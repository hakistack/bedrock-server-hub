//! Tauri commands for previewing, installing and uninstalling addons.

use std::collections::HashSet;
use std::path::Path;

use tauri::{AppHandle, State};

use crate::commands::backup_commands::make_backup;
use crate::core::{addon_installer, addon_parser};
use crate::db::repositories::{addons_repository, servers_repository};
use crate::error::{AppError, AppResult};
use crate::models::addon::{AddonInstallReport, AddonPackage, InstalledAddon};
use crate::models::backup::reason;
use crate::models::server::Server;
use crate::state::AppState;

fn now_iso() -> String {
    chrono::Utc::now().to_rfc3339()
}

fn version_to_string(version: &[i64]) -> String {
    version
        .iter()
        .map(|n| n.to_string())
        .collect::<Vec<_>>()
        .join(".")
}

fn load_server(state: &AppState, server_id: &str) -> AppResult<Server> {
    let conn = state.db.lock().unwrap();
    servers_repository::get(&conn, server_id)?
        .ok_or_else(|| AppError::NotFound("Servidor no encontrado.".into()))
}

/// Inspect an addon file and return its packs, without installing anything.
#[tauri::command]
pub fn preview_addon(source_path: String) -> AppResult<AddonPackage> {
    let prepared = addon_parser::extract_and_scan(Path::new(&source_path))?;
    let package = addon_parser::to_package(Path::new(&source_path), &prepared);
    prepared.cleanup();
    Ok(package)
}

/// Install supported packs from `source_path` into the given world.
/// `selected_uuids` (optional) limits which packs are installed.
/// Creates an automatic backup of the world first.
#[tauri::command]
pub fn install_addon(
    app: AppHandle,
    state: State<AppState>,
    server_id: String,
    world_name: String,
    source_path: String,
    selected_uuids: Option<Vec<String>>,
) -> AppResult<AddonInstallReport> {
    if world_name.trim().is_empty() {
        return Err(AppError::Validation(
            "Debes seleccionar un mundo destino.".into(),
        ));
    }

    let server = load_server(&state, &server_id)?;

    let world_dir = Path::new(&server.worlds_path).join(&world_name);
    if !world_dir.is_dir() {
        return Err(AppError::NotFound(format!(
            "El mundo destino '{world_name}' no existe."
        )));
    }

    // Extract & validate the addon before touching anything.
    let prepared = addon_parser::extract_and_scan(Path::new(&source_path))?;
    let selected: Option<HashSet<String>> =
        selected_uuids.map(|v| v.into_iter().collect());

    // UUIDs this addon provides — used for dependency checks.
    let addon_uuids: HashSet<String> =
        prepared.packs.iter().map(|p| p.manifest.uuid.clone()).collect();

    let conn = state.db.lock().unwrap();

    // Automatic backup of the target world before modifying it.
    if let Err(e) = make_backup(&app, &conn, &server, Some(&world_name), reason::ADDON_INSTALL) {
        drop(conn);
        prepared.cleanup();
        return Err(e);
    }

    let results = match addon_installer::install_packs(
        &server,
        &world_name,
        &prepared,
        selected.as_ref(),
    ) {
        Ok(r) => r,
        Err(e) => {
            drop(conn);
            prepared.cleanup();
            return Err(e);
        }
    };

    // Record successfully wired packs.
    let now = now_iso();
    for r in &results {
        if r.status == "installed" || r.status == "updated" {
            let addon = InstalledAddon {
                id: uuid::Uuid::new_v4().to_string(),
                server_id: server.id.clone(),
                world_name: world_name.clone(),
                name: r.name.clone(),
                uuid: r.uuid.clone(),
                version: version_to_string(&r.version),
                pack_type: r.pack_type.as_db_str().to_string(),
                installed_at: now.clone(),
            };
            addons_repository::insert(&conn, &addon)?;
        }
    }
    drop(conn);

    // Dependency check: warn about UUIDs not provided by this addon and not
    // already present in the world.
    let mut available = addon_installer::world_pack_ids(&server, &world_name);
    available.extend(addon_uuids);
    let mut warnings = Vec::new();
    for pack in &prepared.packs {
        for dep in &pack.manifest.dependencies {
            if !available.contains(&dep.uuid) {
                warnings.push(format!(
                    "'{}' depende de {} que no está instalado en el mundo.",
                    pack.manifest.name, dep.uuid
                ));
            }
        }
    }

    prepared.cleanup();

    Ok(AddonInstallReport {
        world_name,
        results,
        warnings,
    })
}

#[tauri::command]
pub fn list_installed_addons(
    state: State<AppState>,
    server_id: String,
) -> AppResult<Vec<InstalledAddon>> {
    let conn = state.db.lock().unwrap();
    addons_repository::list_for_server(&conn, &server_id)
}

/// Remove a pack from a world: backup, drop its json reference, delete its
/// folder and forget it in the database.
#[tauri::command]
pub fn uninstall_addon(
    app: AppHandle,
    state: State<AppState>,
    server_id: String,
    world_name: String,
    uuid: String,
) -> AppResult<bool> {
    let server = load_server(&state, &server_id)?;

    let conn = state.db.lock().unwrap();
    make_backup(&app, &conn, &server, Some(&world_name), reason::ADDON_UNINSTALL)?;
    let removed = addon_installer::uninstall_pack(&server, &world_name, &uuid)?;
    addons_repository::delete_by_uuid(&conn, &server_id, &world_name, &uuid)?;
    Ok(removed)
}
