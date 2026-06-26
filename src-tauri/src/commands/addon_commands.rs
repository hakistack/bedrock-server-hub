//! Tauri commands for previewing, installing and uninstalling addons.

use std::collections::HashSet;
use std::path::Path;

use tauri::{AppHandle, State};

use crate::commands::backup_commands::make_backup;
use crate::core::{addon_installer, addon_parser};
use crate::db::repositories::{addons_repository, servers_repository};
use crate::error::{AppError, AppResult};
use crate::models::addon::{
    AddonInstallItem, AddonInstallReport, AddonPackage, InstalledAddon,
};
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

/// Inspect an addon file and return its packs, without installing anything.
#[tauri::command]
pub fn preview_addon(source_path: String) -> AppResult<AddonPackage> {
    let prepared = addon_parser::extract_and_scan(Path::new(&source_path))?;
    let package = addon_parser::to_package(Path::new(&source_path), &prepared);
    prepared.cleanup();
    Ok(package)
}

/// Install one or more addons into a world in a single run (one backup).
/// The world may not exist yet — its folder is created and the pack json files
/// pre-seeded so BDS applies them when it generates the world on first start.
fn run_install(
    app: &AppHandle,
    state: &AppState,
    server_id: &str,
    world_name: &str,
    items: Vec<(String, Option<Vec<String>>)>,
) -> AppResult<AddonInstallReport> {
    if world_name.trim().is_empty() {
        return Err(AppError::Validation(
            "Debes seleccionar un mundo destino.".into(),
        ));
    }

    let server: Server = {
        let conn = state.db.lock().unwrap();
        servers_repository::get(&conn, server_id)?
            .ok_or_else(|| AppError::NotFound("Servidor no encontrado.".into()))?
    };

    // Extract every addon up front (no DB lock held while doing slow IO).
    let mut prepared_list = Vec::new();
    let mut warnings = Vec::new();
    for (path, sel) in items {
        match addon_parser::extract_and_scan(Path::new(&path)) {
            Ok(p) => prepared_list.push((p, sel)),
            Err(e) => warnings.push(format!("No se pudo leer un addon: {e}")),
        }
    }
    if prepared_list.is_empty() {
        return Err(AppError::Validation(
            "No hay ningún addon válido para instalar.".into(),
        ));
    }

    // UUIDs available after this batch (world + every addon being installed),
    // used to flag unsatisfied dependencies.
    let mut available = addon_installer::world_pack_ids(&server, world_name);
    for (p, _) in &prepared_list {
        for pk in &p.packs {
            available.insert(pk.manifest.uuid.clone());
        }
    }

    let now = now_iso();
    let mut results = Vec::new();

    {
        let conn = state.db.lock().unwrap();

        // Single automatic backup of the target world before any change.
        if let Err(e) = make_backup(app, &conn, &server, Some(world_name), reason::ADDON_INSTALL) {
            drop(conn);
            for (p, _) in &prepared_list {
                p.cleanup();
            }
            return Err(e);
        }

        for (p, sel) in &prepared_list {
            let selected: Option<HashSet<String>> =
                sel.as_ref().map(|v| v.iter().cloned().collect());

            match addon_installer::install_packs(&server, world_name, p, selected.as_ref()) {
                Ok(r) => {
                    for item in &r {
                        if item.status == "installed" || item.status == "updated" {
                            let addon = InstalledAddon {
                                id: uuid::Uuid::new_v4().to_string(),
                                server_id: server.id.clone(),
                                world_name: world_name.to_string(),
                                name: item.name.clone(),
                                uuid: item.uuid.clone(),
                                version: version_to_string(&item.version),
                                pack_type: item.pack_type.as_db_str().to_string(),
                                installed_at: now.clone(),
                            };
                            addons_repository::insert(&conn, &addon)?;
                        }
                    }
                    for pk in &p.packs {
                        for dep in &pk.manifest.dependencies {
                            if !available.contains(&dep.uuid) {
                                warnings.push(format!(
                                    "'{}' depende de {} que no está instalado en el mundo.",
                                    pk.manifest.name, dep.uuid
                                ));
                            }
                        }
                    }
                    results.extend(r);
                }
                Err(e) => warnings.push(format!("Fallo instalando un addon: {e}")),
            }
        }
    }

    for (p, _) in &prepared_list {
        p.cleanup();
    }

    Ok(AddonInstallReport {
        world_name: world_name.to_string(),
        results,
        warnings,
    })
}

/// Install a single addon (kept for convenience).
#[tauri::command]
pub fn install_addon(
    app: AppHandle,
    state: State<AppState>,
    server_id: String,
    world_name: String,
    source_path: String,
    selected_uuids: Option<Vec<String>>,
) -> AppResult<AddonInstallReport> {
    run_install(
        &app,
        &state,
        &server_id,
        &world_name,
        vec![(source_path, selected_uuids)],
    )
}

/// Install several addons in one run (one backup, combined report).
#[tauri::command]
pub fn install_addons(
    app: AppHandle,
    state: State<AppState>,
    server_id: String,
    world_name: String,
    items: Vec<AddonInstallItem>,
) -> AppResult<AddonInstallReport> {
    let items = items
        .into_iter()
        .map(|i| (i.source_path, i.selected_uuids))
        .collect();
    run_install(&app, &state, &server_id, &world_name, items)
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
    let conn = state.db.lock().unwrap();
    let server = servers_repository::get(&conn, &server_id)?
        .ok_or_else(|| AppError::NotFound("Servidor no encontrado.".into()))?;
    make_backup(&app, &conn, &server, Some(&world_name), reason::ADDON_UNINSTALL)?;
    let removed = addon_installer::uninstall_pack(&server, &world_name, &uuid)?;
    addons_repository::delete_by_uuid(&conn, &server_id, &world_name, &uuid)?;
    Ok(removed)
}
