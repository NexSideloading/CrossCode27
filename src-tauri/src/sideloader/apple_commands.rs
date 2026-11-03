use isideload::dev::{
    app_ids::{AppIdsApi, ListAppIdsResponse},
    certificates::{CertificatesApi, DevelopmentCertificate},
    developer_session::{DeveloperDeviceType, DeveloperSession},
    teams::TeamsApi,
};
use keyring::{Entry, Error as KeyringError};
use serde::{Deserialize, Serialize};
use tauri::Manager;

use crate::sideloader::apple::APPLE_ACCOUNT;

pub fn store_credentials(email: &str, password: &str) -> Result<(), KeyringError> {
    Entry::new("crosscode", "apple_id_email")?.set_password(email)?;
    Entry::new("crosscode", email)?.set_password(password)
}

pub fn get_stored_credentials() -> Option<(String, String)> {
    let email = Entry::new("crosscode", "apple_id_email")
        .ok()?
        .get_password()
        .ok()?;
    let password = Entry::new("crosscode", &email).ok()?.get_password().ok()?;
    Some((email, password))
}

#[tauri::command]
pub fn get_apple_email() -> String {
    get_stored_credentials()
        .map(|(email, _)| email)
        .unwrap_or_default()
}

#[tauri::command]
pub fn delete_stored_credentials() -> Result<(), String> {
    let email_entry =
        Entry::new("crosscode", "apple_id_email").map_err(|e| format!("Keyring error: {e:?}"))?;
    let email = match email_entry.get_password() {
        Ok(email) => email,
        Err(keyring::Error::NoEntry) => return Ok(()),
        Err(e) => return Err(format!("Keyring error: {e:?}")),
    };
    Entry::new("crosscode", &email)
        .map_err(|e| format!("Keyring error: {e:?}"))?
        .delete_credential()
        .map_err(|e| format!("Keyring error: {e:?}"))?;
    email_entry
        .delete_credential()
        .map_err(|e| format!("Keyring error: {e:?}"))?;
    if let Some(account) = APPLE_ACCOUNT.get() {
        *account.lock().unwrap() = None;
    }
    Ok(())
}

#[tauri::command]
pub async fn reset_anisette(handle: tauri::AppHandle) -> Result<(), String> {
    let data_dir = handle.path().app_data_dir().map_err(|e| e.to_string())?;
    let config_dir = handle.path().app_config_dir().map_err(|e| e.to_string())?;
    for state_path in [data_dir.join("anisette_state"), config_dir.join("state.plist")] {
        if state_path.exists() {
            tokio::fs::remove_file(state_path)
                .await
                .map_err(|e| e.to_string())?;
        }
    }
    crate::sideloader::apple::invalidate_account();
    Ok(())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CertificateInfo {
    pub name: String,
    pub certificate_id: String,
    pub serial_number: String,
    pub machine_name: String,
}

fn certificate_info(cert: DevelopmentCertificate) -> CertificateInfo {
    CertificateInfo {
        name: cert.name.unwrap_or_default(),
        certificate_id: cert.certificate_id.unwrap_or_default(),
        serial_number: cert.serial_number.unwrap_or_default(),
        machine_name: cert.machine_name.unwrap_or_default(),
    }
}

async fn team(
    session: &mut DeveloperSession,
) -> Result<isideload::dev::teams::DeveloperTeam, String> {
    session
        .list_teams()
        .await
        .map_err(|e| format!("{e:?}"))?
        .into_iter()
        .next()
        .ok_or_else(|| "No developer teams available".to_string())
}

#[tauri::command]
pub async fn get_certificates(
    handle: tauri::AppHandle,
    window: tauri::Window,
    anisette_server: String,
) -> Result<Vec<CertificateInfo>, String> {
    let mut session =
        crate::sideloader::apple::get_developer_session(&handle, &window, anisette_server).await?;
    let selected_team = team(&mut session).await?;
    session
        .list_all_development_certs(&selected_team, DeveloperDeviceType::Ios)
        .await
        .map(|certs| certs.into_iter().map(certificate_info).collect())
        .map_err(|e| format!("{e:?}"))
}

#[tauri::command]
pub async fn revoke_certificate(
    handle: tauri::AppHandle,
    window: tauri::Window,
    anisette_server: String,
    serial_number: String,
) -> Result<(), String> {
    let mut session =
        crate::sideloader::apple::get_developer_session(&handle, &window, anisette_server).await?;
    let selected_team = team(&mut session).await?;
    session
        .revoke_development_cert(&selected_team, &serial_number, DeveloperDeviceType::Ios)
        .await
        .map_err(|e| format!("{e:?}"))
}

#[tauri::command]
pub async fn list_app_ids(
    handle: tauri::AppHandle,
    window: tauri::Window,
    anisette_server: String,
) -> Result<ListAppIdsResponse, String> {
    let mut session =
        crate::sideloader::apple::get_developer_session(&handle, &window, anisette_server).await?;
    let selected_team = team(&mut session).await?;
    session
        .list_app_ids(&selected_team, None)
        .await
        .map_err(|e| format!("{e:?}"))
}

#[tauri::command]
pub async fn delete_app_id(
    handle: tauri::AppHandle,
    window: tauri::Window,
    anisette_server: String,
    app_id_id: String,
) -> Result<(), String> {
    let mut session =
        crate::sideloader::apple::get_developer_session(&handle, &window, anisette_server).await?;
    let selected_team = team(&mut session).await?;
    session
        .delete_app_id(&selected_team, &app_id_id, None)
        .await
        .map_err(|e| format!("{e:?}"))
}
