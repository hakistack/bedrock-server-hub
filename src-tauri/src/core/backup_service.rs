//! Creates and restores backups before any modifying operation.
//!
//! A backup is a self-contained folder under `<server>/backups/`:
//! ```text
//! backups/2026-06-25_1530_world_import/
//!   metadata.json
//!   server.properties     (copy, when present)
//!   world.zip             (zip of the affected world, when world_name is set)
//! ```

use std::path::{Path, PathBuf};

use tauri::{AppHandle, Emitter};

use crate::core::archive;
use crate::error::{AppError, AppResult};
use crate::models::backup::{BackupProgress, BackupRecord, RestoreOptions};
use crate::models::server::Server;

pub const EVENT_PROGRESS: &str = "backup://progress";

fn emit_progress(app: &AppHandle, backup_id: &str, phase: &str, done: u64, total: u64) {
    let _ = app.emit(
        EVENT_PROGRESS,
        BackupProgress {
            backup_id: backup_id.to_string(),
            phase: phase.to_string(),
            done,
            total,
        },
    );
}

fn timestamp() -> String {
    // Filesystem-safe: no colons.
    chrono::Local::now().format("%Y-%m-%d_%H%M%S").to_string()
}

fn backups_root(server: &Server) -> PathBuf {
    Path::new(&server.path).join("backups")
}

/// Create a backup of the current state, emitting `backup://progress`.
pub fn create_backup(
    app: &AppHandle,
    server: &Server,
    world_name: Option<&str>,
    reason: &str,
) -> AppResult<BackupRecord> {
    let root = backups_root(server);
    std::fs::create_dir_all(&root)?;

    let id = uuid::Uuid::new_v4().to_string();
    let created_at = chrono::Utc::now().to_rfc3339();
    let folder = root.join(format!("{}_{}", timestamp(), reason));
    std::fs::create_dir_all(&folder)
        .map_err(|e| AppError::Io(format!("No se pudo crear la carpeta de backup: {e}")))?;

    emit_progress(app, &id, "starting", 0, 0);

    // 1. server.properties copy.
    let props = Path::new(&server.properties_path);
    if props.is_file() {
        std::fs::copy(props, folder.join("server.properties"))?;
    }

    // 2. The affected world (if any), with progress.
    if let Some(name) = world_name {
        let world_dir = Path::new(&server.worlds_path).join(name);
        if world_dir.is_dir() {
            let id_for_cb = id.clone();
            let app_for_cb = app.clone();
            archive::zip_dir_with_progress(&world_dir, &folder.join("world.zip"), move |done, total| {
                emit_progress(&app_for_cb, &id_for_cb, "zipping", done, total);
            })
            .map_err(|e| AppError::Io(format!("El backup del mundo falló: {e}")))?;
        }
    }

    let record = BackupRecord {
        id: id.clone(),
        server_id: server.id.clone(),
        world_name: world_name.map(|s| s.to_string()),
        reason: reason.to_string(),
        path: folder.to_string_lossy().into_owned(),
        created_at,
        size_bytes: None,
    };

    // 3. metadata.json (self-describing backup, independent of the DB).
    let metadata = serde_json::to_string_pretty(&record)?;
    std::fs::write(folder.join("metadata.json"), metadata)?;

    emit_progress(app, &id, "completed", 1, 1);
    Ok(record)
}

/// Restore a backup. `options` selects what is restored (world and/or
/// server.properties). The caller is expected to have taken a pre_restore
/// safety backup first.
pub fn restore_backup(
    server: &Server,
    record: &BackupRecord,
    options: RestoreOptions,
) -> AppResult<()> {
    let folder = Path::new(&record.path);
    if !folder.is_dir() {
        return Err(AppError::NotFound(
            "La carpeta del backup ya no existe en disco.".into(),
        ));
    }

    // Restore the world from world.zip.
    if options.restore_world {
        let world_zip = folder.join("world.zip");
        if let Some(name) = &record.world_name {
            if world_zip.is_file() {
                let target = Path::new(&server.worlds_path).join(name);
                if target.exists() {
                    std::fs::remove_dir_all(&target)?;
                }
                archive::unzip(&world_zip, &target)?;
            }
        }
    }

    // Restore server.properties.
    if options.restore_properties {
        let props_backup = folder.join("server.properties");
        if props_backup.is_file() {
            std::fs::copy(&props_backup, &server.properties_path)?;
        }
    }

    Ok(())
}

/// Delete a backup folder from disk. Missing folder is treated as success.
pub fn delete_backup(record: &BackupRecord) -> AppResult<()> {
    let folder = Path::new(&record.path);
    if folder.is_dir() {
        std::fs::remove_dir_all(folder)
            .map_err(|e| AppError::Io(format!("No se pudo borrar el backup: {e}")))?;
    }
    Ok(())
}
