//! Small zip helpers shared by the backup service and world import.

use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;

use walkdir::WalkDir;
use zip::write::SimpleFileOptions;

use crate::error::{AppError, AppResult};

/// Recursively zip the contents of `src_dir` into `dest_zip`, invoking
/// `on_progress(done, total)` (file counts) as it works so callers can surface
/// a progress bar for large worlds. Pass `|_, _| {}` to ignore progress.
pub fn zip_dir_with_progress<F: FnMut(u64, u64)>(
    src_dir: &Path,
    dest_zip: &Path,
    mut on_progress: F,
) -> AppResult<()> {
    let file = File::create(dest_zip)
        .map_err(|e| AppError::Io(format!("No se pudo crear el zip: {e}")))?;
    let mut zip = zip::ZipWriter::new(file);
    let options = SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);

    // Count files first so we can report a percentage.
    let total_files = WalkDir::new(src_dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().is_file())
        .count() as u64;
    on_progress(0, total_files);

    let mut done: u64 = 0;
    let mut buffer = Vec::new();
    for entry in WalkDir::new(src_dir).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        let rel = path
            .strip_prefix(src_dir)
            .map_err(|e| AppError::Internal(e.to_string()))?;
        if rel.as_os_str().is_empty() {
            continue;
        }
        let name = rel.to_string_lossy().replace('\\', "/");

        if path.is_file() {
            zip.start_file(name, options)
                .map_err(|e| AppError::Internal(e.to_string()))?;
            let mut f = File::open(path)?;
            buffer.clear();
            f.read_to_end(&mut buffer)?;
            zip.write_all(&buffer)?;
            done += 1;
            on_progress(done, total_files);
        } else if path.is_dir() {
            zip.add_directory(format!("{name}/"), options)
                .map_err(|e| AppError::Internal(e.to_string()))?;
        }
    }

    zip.finish()
        .map_err(|e| AppError::Internal(format!("No se pudo finalizar el zip: {e}")))?;
    Ok(())
}

/// Extract a zip archive into `dest_dir`, creating it if needed.
pub fn unzip(zip_path: &Path, dest_dir: &Path) -> AppResult<()> {
    let file = File::open(zip_path)
        .map_err(|e| AppError::Io(format!("No se pudo abrir el archivo: {e}")))?;
    let mut archive = zip::ZipArchive::new(file)
        .map_err(|e| AppError::Validation(format!("Archivo zip inválido o corrupto: {e}")))?;
    std::fs::create_dir_all(dest_dir)?;
    archive
        .extract(dest_dir)
        .map_err(|e| AppError::Internal(format!("No se pudo extraer el archivo: {e}")))?;
    Ok(())
}

/// Recursively copy a directory tree from `src` to `dst`.
pub fn copy_dir(src: &Path, dst: &Path) -> AppResult<()> {
    std::fs::create_dir_all(dst)?;
    for entry in WalkDir::new(src).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        let rel = path
            .strip_prefix(src)
            .map_err(|e| AppError::Internal(e.to_string()))?;
        if rel.as_os_str().is_empty() {
            continue;
        }
        let target = dst.join(rel);
        if path.is_dir() {
            std::fs::create_dir_all(&target)?;
        } else if path.is_file() {
            if let Some(parent) = target.parent() {
                std::fs::create_dir_all(parent)?;
            }
            std::fs::copy(path, &target)?;
        }
    }
    Ok(())
}

/// Total size in bytes of a directory tree.
pub fn dir_size(dir: &Path) -> u64 {
    WalkDir::new(dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().is_file())
        .filter_map(|e| e.metadata().ok())
        .map(|m| m.len())
        .sum()
}
