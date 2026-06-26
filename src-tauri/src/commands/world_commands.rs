//! Tauri commands for listing, importing and activating worlds.

use std::path::Path;

use rusqlite::Connection;
use tauri::{AppHandle, State};

use crate::commands::backup_commands::make_backup;
use crate::core::{archive, server_properties, world_manager};
use crate::db::repositories::servers_repository;
use crate::error::{AppError, AppResult};
use crate::models::backup::reason;
use crate::models::properties::PropertyUpdate;
use crate::models::server::Server;
use crate::models::world::World;
use crate::state::AppState;

const LEVEL_NAME_KEY: &str = "level-name";

fn load_server(conn: &Connection, server_id: &str) -> AppResult<Server> {
    servers_repository::get(conn, server_id)?
        .ok_or_else(|| AppError::NotFound("Servidor no encontrado.".into()))
}

/// Read the current active world folder name from `server.properties`.
fn active_level_name(server: &Server) -> String {
    server_properties::read(Path::new(&server.properties_path))
        .ok()
        .and_then(|entries| {
            entries
                .into_iter()
                .find(|e| e.key == LEVEL_NAME_KEY)
                .map(|e| e.value)
        })
        .unwrap_or_default()
}

#[tauri::command]
pub fn list_worlds(state: State<AppState>, server_id: String) -> AppResult<Vec<World>> {
    let conn = state.db.lock().unwrap();
    let server = load_server(&conn, &server_id)?;
    drop(conn);

    let active = active_level_name(&server);
    world_manager::list_worlds(Path::new(&server.worlds_path), &active)
}

fn set_active_world(server: &Server, world_name: &str) -> AppResult<()> {
    server_properties::update(
        Path::new(&server.properties_path),
        &[PropertyUpdate {
            key: LEVEL_NAME_KEY.to_string(),
            value: world_name.to_string(),
        }],
    )?;
    Ok(())
}

/// Import a `.mcworld` (or zip). Always imported into a fresh, non-colliding
/// folder so an existing world is never overwritten. Optionally activated.
#[tauri::command]
pub fn import_world(
    app: AppHandle,
    state: State<AppState>,
    server_id: String,
    mcworld_path: String,
    make_active: bool,
) -> AppResult<World> {
    let conn = state.db.lock().unwrap();
    let server = load_server(&conn, &server_id)?;

    let extracted = world_manager::extract_world(Path::new(&mcworld_path))?;
    let worlds_path = Path::new(&server.worlds_path);
    let target_name = world_manager::unique_world_name(worlds_path, &extracted.suggested_name);
    let target = worlds_path.join(&target_name);

    // Copy the validated world into place, then clean up the temp extraction.
    let copy_result = archive::copy_dir(&extracted.root, &target);
    let _ = std::fs::remove_dir_all(&extracted.temp_dir);
    copy_result?;

    if make_active {
        // Activation edits server.properties → back it up first.
        make_backup(&app, &conn, &server, None, reason::PROPERTIES_EDIT)?;
        set_active_world(&server, &target_name)?;
    }
    drop(conn);

    let active = if make_active {
        target_name.clone()
    } else {
        active_level_name(&server)
    };
    let worlds = world_manager::list_worlds(worlds_path, &active)?;
    worlds
        .into_iter()
        .find(|w| w.name == target_name)
        .ok_or_else(|| AppError::Internal("El mundo importado no pudo localizarse.".into()))
}

/// Set a world as the active one (updates `level-name`). Backs up first.
#[tauri::command]
pub fn activate_world(
    app: AppHandle,
    state: State<AppState>,
    server_id: String,
    world_name: String,
) -> AppResult<()> {
    let conn = state.db.lock().unwrap();
    let server = load_server(&conn, &server_id)?;

    let target = Path::new(&server.worlds_path).join(&world_name);
    if !target.is_dir() {
        return Err(AppError::NotFound(
            "El mundo seleccionado no existe.".into(),
        ));
    }

    make_backup(&app, &conn, &server, None, reason::PROPERTIES_EDIT)?;
    set_active_world(&server, &world_name)?;
    Ok(())
}
