//! Tauri commands for creating a server from an official download.

use std::path::Path;

use tauri::{AppHandle, Manager};

use crate::commands::{network_commands, server_commands};
use crate::core::{server_downloader, server_installer};
use crate::db::repositories::servers_repository;
use crate::error::{AppError, AppResult};
use crate::models::download::{
    DownloadedServerPackage, ServerChannel, ServerDownloadOption, ServerPlatform,
};
use crate::models::server::Server;
use crate::state::AppState;

fn platform_label(p: ServerPlatform) -> String {
    match p {
        ServerPlatform::Windows => "Windows",
        ServerPlatform::Linux => "Linux",
    }
    .to_string()
}

fn channel_label(c: ServerChannel) -> String {
    match c {
        ServerChannel::Stable => "Stable",
        ServerChannel::Preview => "Preview",
    }
    .to_string()
}

/// Extract → validate → register a downloaded package as a managed server.
/// Shared by `install_downloaded_server` and `create_server_from_official_download`.
fn register_installed(
    app: &AppHandle,
    package: &DownloadedServerPackage,
    target_directory: &str,
    server_name: String,
) -> AppResult<Server> {
    let target = Path::new(target_directory);
    let resolved = server_installer::install(Path::new(&package.path), target)?;

    let mut server = server_commands::build_server_record(target, Some(server_name), &resolved);
    server.install_source = Some("official_download".into());
    server.created_from_download = true;
    server.platform = Some(platform_label(package.option.platform));
    server.channel = Some(channel_label(package.option.channel));
    server.server_version = package.option.version.clone();

    let state = app.state::<AppState>();
    let conn = state.db.lock().unwrap();
    if servers_repository::find_by_path(&conn, &server.path)?.is_some() {
        return Err(AppError::Validation(
            "Ya hay un servidor registrado en esa carpeta.".into(),
        ));
    }
    servers_repository::insert(&conn, &server)?;

    // A freshly downloaded BDS defaults to port 19132 — give each new server a
    // free, non-conflicting port pair. Best-effort: never fail creation over it.
    let used = network_commands::collect_used_ports(&conn, &server.id);
    let (v4, v6) = network_commands::free_port_pair(&used);
    let _ = network_commands::write_ports(&server, v4, v6);

    drop(conn);

    // Best-effort cleanup of the downloaded zip.
    let _ = std::fs::remove_file(&package.path);

    Ok(server)
}

/// Install an already-downloaded package into a target folder.
#[tauri::command]
pub fn install_downloaded_server(
    app: AppHandle,
    package: DownloadedServerPackage,
    target_directory: String,
    server_name: String,
) -> AppResult<Server> {
    register_installed(&app, &package, &target_directory, server_name)
}

/// One-shot: download the official server then install & register it.
#[tauri::command]
pub async fn create_server_from_official_download(
    app: AppHandle,
    option: ServerDownloadOption,
    target_directory: String,
    server_name: String,
    accepted_eula: bool,
) -> AppResult<Server> {
    let download_id = uuid::Uuid::new_v4().to_string();
    let package = server_downloader::download(&app, &option, &download_id, accepted_eula).await?;
    register_installed(&app, &package, &target_directory, server_name)
}
