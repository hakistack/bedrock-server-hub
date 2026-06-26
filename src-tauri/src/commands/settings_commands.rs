//! Tauri commands for reading and editing `server.properties`.

use std::path::Path;

use tauri::State;

use crate::core::server_properties;
use crate::db::repositories::servers_repository;
use crate::error::{AppError, AppResult};
use crate::models::properties::{PropertyEntry, PropertyUpdate};
use crate::state::AppState;

fn properties_path(state: &AppState, server_id: &str) -> AppResult<String> {
    let conn = state.db.lock().unwrap();
    let server = servers_repository::get(&conn, server_id)?
        .ok_or_else(|| AppError::NotFound("Servidor no encontrado.".into()))?;
    Ok(server.properties_path)
}

#[tauri::command]
pub fn read_properties(
    state: State<AppState>,
    server_id: String,
) -> AppResult<Vec<PropertyEntry>> {
    let path = properties_path(&state, &server_id)?;
    server_properties::read(Path::new(&path))
}

#[tauri::command]
pub fn update_properties(
    state: State<AppState>,
    server_id: String,
    updates: Vec<PropertyUpdate>,
) -> AppResult<Vec<PropertyEntry>> {
    let path = properties_path(&state, &server_id)?;
    server_properties::update(Path::new(&path), &updates)
}
