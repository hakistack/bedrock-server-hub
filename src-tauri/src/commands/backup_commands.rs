//! Tauri commands for creating, listing, restoring and deleting backups.

use rusqlite::Connection;
use tauri::{AppHandle, State};

use crate::core::backup_service;
use crate::db::repositories::{backups_repository, servers_repository};
use crate::error::{AppError, AppResult};
use crate::models::backup::{reason, BackupRecord, RestoreOptions};
use crate::models::server::Server;
use crate::state::AppState;

fn load_server(conn: &Connection, server_id: &str) -> AppResult<Server> {
    servers_repository::get(conn, server_id)?
        .ok_or_else(|| AppError::NotFound("Servidor no encontrado.".into()))
}

/// Create a backup on disk and record it in the database.
/// Shared by other commands (world import, activate, restore safety backups).
pub(crate) fn make_backup(
    app: &AppHandle,
    conn: &Connection,
    server: &Server,
    world_name: Option<&str>,
    reason: &str,
) -> AppResult<BackupRecord> {
    let record = backup_service::create_backup(app, server, world_name, reason)?;
    backups_repository::insert(conn, &record)?;
    Ok(record)
}

#[tauri::command]
pub fn list_backups(state: State<AppState>, server_id: String) -> AppResult<Vec<BackupRecord>> {
    let conn = state.db.lock().unwrap();
    backups_repository::list_for_server(&conn, &server_id)
}

/// Manual backup, optionally scoped to a specific world.
#[tauri::command]
pub fn create_backup(
    app: AppHandle,
    state: State<AppState>,
    server_id: String,
    world_name: Option<String>,
) -> AppResult<BackupRecord> {
    let conn = state.db.lock().unwrap();
    let server = load_server(&conn, &server_id)?;
    make_backup(&app, &conn, &server, world_name.as_deref(), reason::MANUAL)
}

/// Restore a backup. A safety (pre_restore) backup of the current state is
/// always taken first, so a restore is itself reversible.
#[tauri::command]
pub fn restore_backup(
    app: AppHandle,
    state: State<AppState>,
    backup_id: String,
    options: Option<RestoreOptions>,
) -> AppResult<BackupRecord> {
    let opts = options.unwrap_or_default();
    let conn = state.db.lock().unwrap();
    let record = backups_repository::get(&conn, &backup_id)?
        .ok_or_else(|| AppError::NotFound("Backup no encontrado.".into()))?;
    let server = load_server(&conn, &record.server_id)?;

    make_backup(&app, &conn, &server, record.world_name.as_deref(), reason::PRE_RESTORE)?;
    backup_service::restore_backup(&server, &record, opts)?;
    Ok(record)
}

#[tauri::command]
pub fn delete_backup(state: State<AppState>, backup_id: String) -> AppResult<()> {
    let conn = state.db.lock().unwrap();
    let record = backups_repository::get(&conn, &backup_id)?
        .ok_or_else(|| AppError::NotFound("Backup no encontrado.".into()))?;
    backup_service::delete_backup(&record)?;
    backups_repository::delete(&conn, &backup_id)?;
    Ok(())
}
