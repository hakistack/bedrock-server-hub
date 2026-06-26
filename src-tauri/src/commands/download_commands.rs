//! Tauri commands for resolving and downloading the official server.

use tauri::AppHandle;

use crate::core::{official_download_resolver, server_downloader};
use crate::error::AppResult;
use crate::models::download::{DownloadedServerPackage, ServerDownloadOption};

/// Resolve the official download options (API, then HTML fallback).
#[tauri::command]
pub async fn get_official_server_download_options() -> AppResult<Vec<ServerDownloadOption>> {
    official_download_resolver::resolve_options().await
}

/// Build an option from a user-pasted official URL (manual fallback path).
#[tauri::command]
pub fn resolve_manual_download_url(url: String) -> AppResult<ServerDownloadOption> {
    official_download_resolver::option_from_manual_url(&url)
}

/// Download the selected option's zip, emitting progress on `download://progress`.
#[tauri::command]
pub async fn download_bedrock_server(
    app: AppHandle,
    option: ServerDownloadOption,
    download_id: String,
    accepted_eula: bool,
) -> AppResult<DownloadedServerPackage> {
    server_downloader::download(&app, &option, &download_id, accepted_eula).await
}

/// Cancel an in-flight download by id.
#[tauri::command]
pub fn cancel_download(app: AppHandle, download_id: String) {
    server_downloader::cancel(&app, &download_id);
}
