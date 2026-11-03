use std::path::PathBuf;

use isideload::{
    sideload::{
        builder::{ExtensionsBehavior, ExtensionsBehaviorSelection, MaxCertsBehavior},
        sideloader::Sideloader,
        SideloaderBuilder,
    },
    util::callbacks::{ExtensionsBehaviorCallbackBox, MaxCertsCallbackBox},
};
use tauri::{Emitter, Manager};

use crate::sideloader::device::{get_provider, list_devices, DeviceInfo};

type CrossCodeSideloader = Sideloader<MaxCertsCallbackBox, ExtensionsBehaviorCallbackBox>;

async fn create_sideloader(
    handle: &tauri::AppHandle,
    window: &tauri::Window,
    anisette_server: String,
) -> Result<CrossCodeSideloader, String> {
    let account =
        crate::sideloader::apple::get_account(handle, window, anisette_server.clone()).await?;
    let email = account.lock().await.email.clone();
    let dev_session =
        crate::sideloader::apple::get_developer_session(handle, window, anisette_server).await?;
    let storage = Box::new(isideload::util::fs_storage::FsStorage::new(
        handle
            .path()
            .app_data_dir()
            .map_err(|e| format!("Failed to get app data directory: {e}"))?,
    ));

    Ok(SideloaderBuilder::new(dev_session, email)
        .machine_name("CrossCode".to_string())
        .storage(storage)
        .extensions_behavior(ExtensionsBehaviorSelection::Fixed(
            ExtensionsBehavior::ReuseMain,
        ))
        .max_certs_behavior(MaxCertsBehavior::Revoke)
        .build())
}

pub async fn sideload_app(
    handle: &tauri::AppHandle,
    window: &tauri::Window,
    anisette_server: String,
    device: DeviceInfo,
    app_path: PathBuf,
) -> Result<(), String> {
    let provider = get_provider(&device).await?;
    let mut sideloader = create_sideloader(handle, window, anisette_server).await?;
    sideloader
        .install_app(
            &provider,
            app_path,
            true,
            None::<fn(f32) -> std::future::Ready<()>>,
            None,
        )
        .await
        .map_err(|e| format!("{e:?}"))?;
    Ok(())
}

#[tauri::command]
pub async fn refresh_idevice(window: tauri::Window) {
    match list_devices().await {
        Ok(devices) => {
            window.emit("idevices", devices).ok();
        }
        Err(e) => {
            window.emit("idevices", Vec::<DeviceInfo>::new()).ok();
            eprintln!("Failed to list devices: {e}");
        }
    }
}
