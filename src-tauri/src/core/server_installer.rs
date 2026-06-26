//! Extracts a downloaded server zip into a target folder and validates it.

use std::path::{Path, PathBuf};

use crate::core::{archive, server_validator};
use crate::core::server_validator::ResolvedPaths;
use crate::error::{AppError, AppResult};

/// True if the directory does not exist or exists and is empty.
fn is_empty_or_absent(dir: &Path) -> bool {
    if !dir.exists() {
        return true;
    }
    match std::fs::read_dir(dir) {
        Ok(mut entries) => entries.next().is_none(),
        Err(_) => false,
    }
}

/// If the zip extracted into a single nested folder, hoist its contents up so
/// the executable sits at `target` root. No-op if already at root.
fn normalize_root(target: &Path) -> AppResult<()> {
    if server_validator::find_executable(target).is_some() {
        return Ok(());
    }

    let subdirs: Vec<PathBuf> = std::fs::read_dir(target)?
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.is_dir())
        .collect();

    for sub in subdirs {
        if server_validator::find_executable(&sub).is_some() {
            for entry in std::fs::read_dir(&sub)? {
                let entry = entry?;
                let to = target.join(entry.file_name());
                std::fs::rename(entry.path(), to)?;
            }
            let _ = std::fs::remove_dir_all(&sub);
            return Ok(());
        }
    }
    Ok(())
}

#[cfg(unix)]
fn make_executable(path: &Path) {
    use std::os::unix::fs::PermissionsExt;
    if let Ok(meta) = std::fs::metadata(path) {
        let mut perms = meta.permissions();
        perms.set_mode(0o755);
        let _ = std::fs::set_permissions(path, perms);
    }
}

#[cfg(not(unix))]
fn make_executable(_path: &Path) {}

/// Install a server zip into `target_dir`.
///
/// Safety: refuses to install into a non-empty existing folder (never
/// overwrites user files). Returns the resolved, validated installation paths.
pub fn install(package_zip: &Path, target_dir: &Path) -> AppResult<ResolvedPaths> {
    if !package_zip.is_file() {
        return Err(AppError::Validation(
            "El paquete descargado no existe.".into(),
        ));
    }
    if !is_empty_or_absent(target_dir) {
        return Err(AppError::Validation(
            "La carpeta de instalación no está vacía. Elige una carpeta vacía o nueva.".into(),
        ));
    }

    std::fs::create_dir_all(target_dir)
        .map_err(|e| AppError::Io(format!("No se pudo crear la carpeta de instalación: {e}")))?;

    archive::unzip(package_zip, target_dir)?;
    normalize_root(target_dir)?;

    // `resolve` validates layout and creates the worlds folder.
    let resolved = server_validator::resolve(target_dir).map_err(|e| match e {
        AppError::Validation(msg) => AppError::Validation(format!(
            "La instalación no es válida tras extraer el servidor: {msg}"
        )),
        other => other,
    })?;

    make_executable(&resolved.executable);
    Ok(resolved)
}
