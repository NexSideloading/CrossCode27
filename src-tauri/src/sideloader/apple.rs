use futures::FutureExt;
use isideload::{
    anisette::{remote_v3::RemoteV3AnisetteProvider, AnisetteClientInfo},
    auth::apple_account::{AppleAccount, TwoFactorCallbackParams, TwoFactorCallbackResponse},
    dev::developer_session::DeveloperSession,
    util::fs_storage::FsStorage,
    util::storage::SideloadingStorage,
};
use once_cell::sync::OnceCell;
use serde_json::Value;
use std::{
    sync::{Arc, Mutex},
    time::Duration,
};
use tauri::{Emitter, Listener, Manager};
use tokio::sync::Mutex as TokioMutex;

use crate::sideloader::apple_commands::{get_stored_credentials, store_credentials};

pub static APPLE_ACCOUNT: OnceCell<Mutex<Option<Arc<TokioMutex<AppleAccount>>>>> =
    OnceCell::new();

fn storage(handle: &tauri::AppHandle) -> Result<Box<dyn SideloadingStorage>, String> {
    Ok(Box::new(FsStorage::new(
        handle
            .path()
            .app_data_dir()
            .map_err(|e| format!("Failed to get app data directory: {e}"))?,
    )))
}

pub async fn get_account(
    handle: &tauri::AppHandle,
    window: &tauri::Window,
    anisette_server: String,
) -> Result<Arc<TokioMutex<AppleAccount>>, String> {
    let cell = APPLE_ACCOUNT.get_or_init(|| Mutex::new(None));
    if let Some(account) = cell.lock().unwrap().as_ref() {
        return Ok(account.clone());
    }

    let account = login(handle, window, anisette_server).await?;
    let account = Arc::new(TokioMutex::new(account));
    *cell.lock().unwrap() = Some(account.clone());
    Ok(account)
}

pub async fn get_developer_session(
    handle: &tauri::AppHandle,
    window: &tauri::Window,
    anisette_server: String,
) -> Result<DeveloperSession, String> {
    let account = get_account(handle, window, anisette_server).await?;
    let mut account = account.lock().await;
    DeveloperSession::from_account(&mut *account)
        .await
        .map_err(|e| format!("{e:?}"))
}

pub async fn login(
    handle: &tauri::AppHandle,
    window: &tauri::Window,
    anisette_server: String,
) -> Result<AppleAccount, String> {
    let (email, password) = if let Some(credentials) = get_stored_credentials() {
        window
            .emit("build-output", "Using stored Apple ID credentials".to_string())
            .ok();
        credentials
    } else {
        let (tx, rx) = std::sync::mpsc::channel::<String>();
        window
            .emit("apple-id-required", ())
            .map_err(|e| e.to_string())?;
        let tx_login = tx.clone();
        let received = window.listen("apple-id-recieved", move |event| {
            let _ = tx_login.send(format!("apple-id-recieved:{}", event.payload()));
        });
        let tx_cancel = tx;
        let cancelled = window.listen("login-cancelled", move |_| {
            let _ = tx_cancel.send("login-cancelled".to_string());
        });
        let result = rx.recv_timeout(Duration::from_secs(120));
        window.unlisten(received);
        window.unlisten(cancelled);
        match result {
            Ok(message) if message.starts_with("apple-id-recieved:") => {
                let json: Value = serde_json::from_str(
                    &message["apple-id-recieved:".len()..],
                )
                .map_err(|e| format!("Failed to parse Apple ID response: {e}"))?;
                let email = json
                    .get("appleId")
                    .and_then(Value::as_str)
                    .ok_or("Apple ID was not provided")?
                    .to_string();
                let password = json
                    .get("applePass")
                    .and_then(Value::as_str)
                    .ok_or("Apple ID password was not provided")?
                    .to_string();
                if json
                    .get("saveCredentials")
                    .and_then(Value::as_bool)
                    .unwrap_or(false)
                {
                    store_credentials(&email, &password).map_err(|e| e.to_string())?;
                }
                (email, password)
            }
            Ok(_) => return Err("Login cancelled by user".to_string()),
            Err(_) => return Err("Login cancelled or timed out".to_string()),
        }
    };

    let anisette_url = if anisette_server.starts_with("http") {
        anisette_server
    } else {
        format!("https://{anisette_server}")
    };
    let window_clone = window.clone();
    let tfa = move |params: TwoFactorCallbackParams| {
        let window = window_clone.clone();
        async move {
            window
                .emit("2fa-required", params)
                .map_err(|e| rootcause::report!("{e}"))?;
            let (tx, rx) = std::sync::mpsc::channel::<String>();
            let handler = window.listen("2fa-recieved", move |event| {
                let _ = tx.send(event.payload().trim_matches('"').to_string());
            });
            let result = rx.recv_timeout(Duration::from_secs(120));
            window.unlisten(handler);
            Ok(match result {
                Ok(code) => TwoFactorCallbackResponse::SubmitCode(code),
                Err(_) => TwoFactorCallbackResponse::Abort,
            })
        }
        .boxed()
    };

    window
        .emit("build-output", "Logging in...".to_string())
        .map_err(|e| e.to_string())?;
    let provider = RemoteV3AnisetteProvider::default()
        .map_err(|e| format!("{e:?}"))?
        .set_serial_number("0".to_string())
        .set_storage(storage(handle)?)
        .set_client_info_override(Some(AnisetteClientInfo {
            client_info:
                "<Mac15,7> <macOS;27.0;26A5378j> <com.apple.AuthKit/1 (com.apple.akd/1.0)>"
                    .to_string(),
            user_agent: "akd/1.0 CFNetwork/811.5.4".to_string(),
        }))
        .set_url(&anisette_url);
    let account = AppleAccount::builder(&email.to_lowercase())
        .anisette_provider(provider)
        .login(&password, Box::new(tfa))
        .await
        .map_err(|e| format!("{e:?}"))?;
    window
        .emit("build-output", "Successfully logged in".to_string())
        .map_err(|e| e.to_string())?;
    Ok(account)
}

pub fn invalidate_account() {
    if let Some(cell) = APPLE_ACCOUNT.get() {
        *cell.lock().unwrap() = None;
    }
}
