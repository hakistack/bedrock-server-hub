//! Tauri commands for the portable self-updater.

use tauri::AppHandle;

use crate::core::updater;
use crate::error::AppResult;
use crate::models::update::UpdateInfo;

/// Check GitHub Releases for a newer version.
#[tauri::command]
pub async fn check_for_update(app: AppHandle) -> AppResult<UpdateInfo> {
    updater::check(&app).await
}

/// Download the new portable exe and swap it in, then relaunch (Windows).
/// On success the app exits/relaunches, so this may not return normally.
#[tauri::command]
pub async fn download_and_install_update(app: AppHandle, download_url: String) -> AppResult<()> {
    updater::download_and_install(&app, &download_url).await
}
