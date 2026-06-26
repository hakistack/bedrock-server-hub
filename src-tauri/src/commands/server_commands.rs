//! Tauri commands for registering and controlling Bedrock servers.

use std::path::{Path, PathBuf};

use tauri::{AppHandle, State};

use crate::core::server_process::{self, ServerStatus};
use crate::core::server_validator::{self, ServerValidationResult};
use crate::db::repositories::{servers_repository, settings_repository};
use crate::error::{AppError, AppResult};
use crate::models::server::Server;
use crate::state::AppState;

/// Per-server preferences surfaced to the UI.
#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ServerSettings {
    pub auto_restart: bool,
}

fn now_iso() -> String {
    chrono::Utc::now().to_rfc3339()
}

/// Build a `Server` for a freshly resolved installation directory.
/// `dir` must already be validated. Used by import and the download wizard.
pub(crate) fn build_server_record(
    dir: &Path,
    name: Option<String>,
    resolved: &server_validator::ResolvedPaths,
) -> Server {
    let canonical = dir
        .canonicalize()
        .unwrap_or_else(|_| dir.to_path_buf())
        .to_string_lossy()
        .into_owned();

    let display_name = name.filter(|n| !n.trim().is_empty()).unwrap_or_else(|| {
        dir.file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_else(|| "Bedrock Server".into())
    });

    let now = now_iso();
    Server {
        id: uuid::Uuid::new_v4().to_string(),
        name: display_name,
        path: canonical,
        executable_path: resolved.executable.to_string_lossy().into_owned(),
        properties_path: resolved.properties.to_string_lossy().into_owned(),
        worlds_path: resolved.worlds.to_string_lossy().into_owned(),
        created_at: now.clone(),
        updated_at: now,
        server_version: None,
        install_source: Some("import".into()),
        platform: None,
        channel: None,
        created_from_download: false,
    }
}

/// Inspect a folder and report whether it looks like a valid BDS install.
#[tauri::command]
pub fn validate_server_folder(path: String) -> ServerValidationResult {
    server_validator::validate(Path::new(&path))
}

/// Register a folder as a managed server. Validates layout and persists it.
#[tauri::command]
pub fn import_server(
    state: State<AppState>,
    path: String,
    name: Option<String>,
) -> AppResult<Server> {
    let dir = PathBuf::from(&path);
    let canonical = dir
        .canonicalize()
        .unwrap_or_else(|_| dir.clone())
        .to_string_lossy()
        .into_owned();

    let conn = state.db.lock().unwrap();
    if servers_repository::find_by_path(&conn, &canonical)?.is_some() {
        return Err(AppError::Validation(
            "Este servidor ya está registrado.".into(),
        ));
    }

    let resolved = server_validator::resolve(&dir)?;
    let server = build_server_record(&dir, name, &resolved);
    servers_repository::insert(&conn, &server)?;
    Ok(server)
}

#[tauri::command]
pub fn list_servers(state: State<AppState>) -> AppResult<Vec<Server>> {
    let conn = state.db.lock().unwrap();
    servers_repository::list(&conn)
}

#[tauri::command]
pub fn get_server(state: State<AppState>, server_id: String) -> AppResult<Server> {
    let conn = state.db.lock().unwrap();
    servers_repository::get(&conn, &server_id)?
        .ok_or_else(|| AppError::NotFound("Servidor no encontrado.".into()))
}

#[tauri::command]
pub fn rename_server(state: State<AppState>, server_id: String, name: String) -> AppResult<Server> {
    if name.trim().is_empty() {
        return Err(AppError::Validation("El nombre no puede estar vacío.".into()));
    }
    let conn = state.db.lock().unwrap();
    servers_repository::rename(&conn, &server_id, name.trim(), &now_iso())?;
    servers_repository::get(&conn, &server_id)?
        .ok_or_else(|| AppError::NotFound("Servidor no encontrado.".into()))
}

#[tauri::command]
pub fn remove_server(
    app: AppHandle,
    state: State<AppState>,
    server_id: String,
) -> AppResult<()> {
    // Stop it first if it is running, then forget it.
    let _ = server_process::stop(&app, &server_id);
    let conn = state.db.lock().unwrap();
    servers_repository::delete(&conn, &server_id)?;
    Ok(())
}

fn load_server(state: &AppState, server_id: &str) -> AppResult<Server> {
    let conn = state.db.lock().unwrap();
    servers_repository::get(&conn, server_id)?
        .ok_or_else(|| AppError::NotFound("Servidor no encontrado.".into()))
}

#[tauri::command]
pub fn start_server(
    app: AppHandle,
    state: State<AppState>,
    server_id: String,
) -> AppResult<()> {
    let server = load_server(&state, &server_id)?;
    server_process::start(&app, &server)
}

#[tauri::command]
pub fn stop_server(app: AppHandle, server_id: String) -> AppResult<()> {
    server_process::stop(&app, &server_id)
}

#[tauri::command]
pub fn restart_server(
    app: AppHandle,
    state: State<AppState>,
    server_id: String,
) -> AppResult<()> {
    let server = load_server(&state, &server_id)?;
    server_process::restart(&app, &server)
}

#[tauri::command]
pub fn get_server_status(state: State<AppState>, server_id: String) -> ServerStatus {
    server_process::status(&state, &server_id)
}

#[tauri::command]
pub fn get_server_settings(state: State<AppState>, server_id: String) -> AppResult<ServerSettings> {
    let conn = state.db.lock().unwrap();
    let auto_restart =
        settings_repository::get_bool(&conn, &settings_repository::auto_restart_key(&server_id))?;
    Ok(ServerSettings { auto_restart })
}

#[tauri::command]
pub fn set_auto_restart(
    state: State<AppState>,
    server_id: String,
    enabled: bool,
) -> AppResult<()> {
    let conn = state.db.lock().unwrap();
    settings_repository::set_bool(
        &conn,
        &settings_repository::auto_restart_key(&server_id),
        enabled,
    )
}

/// Send a raw console command to a running server's stdin.
#[tauri::command]
pub fn send_server_command(
    state: State<AppState>,
    server_id: String,
    command: String,
) -> AppResult<()> {
    server_process::send_command(&state, &server_id, &command)
}
