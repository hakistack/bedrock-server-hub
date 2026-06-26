//! Extracts a `.mcaddon`/`.mcpack`/`.zip` and discovers the packs inside it.
//!
//! A `.mcaddon` can bundle several packs (each its own folder with a
//! `manifest.json`); a `.mcpack` is usually a single pack. We extract once and
//! walk for every `manifest.json` to support both uniformly.

use std::path::{Path, PathBuf};

use walkdir::WalkDir;

use crate::core::{archive, manifest_reader};
use crate::error::{AppError, AppResult};
use crate::models::addon::{AddonPack, AddonPackage};
use crate::models::manifest::Manifest;

/// A pack located inside an extracted archive.
pub struct PreparedPack {
    pub manifest: Manifest,
    /// Folder that directly contains the `manifest.json`.
    pub dir: PathBuf,
    /// Path relative to the extraction root (for display).
    pub rel: String,
}

pub struct PreparedAddon {
    pub temp_dir: PathBuf,
    pub packs: Vec<PreparedPack>,
}

impl PreparedAddon {
    /// Remove the temporary extraction directory.
    pub fn cleanup(&self) {
        let _ = std::fs::remove_dir_all(&self.temp_dir);
    }
}

fn is_manifest(path: &Path) -> bool {
    path.file_name()
        .map(|n| n.to_string_lossy().eq_ignore_ascii_case("manifest.json"))
        .unwrap_or(false)
}

fn has_ext(path: &Path, ext: &str) -> bool {
    path.extension()
        .map(|e| e.to_string_lossy().eq_ignore_ascii_case(ext))
        .unwrap_or(false)
}

/// Extract any nested `.mcpack`/`.zip` archives found under `dir` into sibling
/// folders, so their manifests become visible to the scan. A `.mcaddon` is
/// often a zip that *contains* one `.mcpack` per pack.
fn expand_nested_archives(dir: &Path) {
    let nested: Vec<_> = WalkDir::new(dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .map(|e| e.path().to_path_buf())
        .filter(|p| has_ext(p, "mcpack") || has_ext(p, "zip"))
        .collect();

    for (i, archive_path) in nested.iter().enumerate() {
        let stem = archive_path
            .file_stem()
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_else(|| format!("pack_{i}"));
        let dest = dir.join(format!("__nested_{i}_{stem}"));
        // Best-effort: a corrupt nested pack just won't contribute manifests.
        let _ = archive::unzip(archive_path, &dest);
    }
}

/// Extract the archive to a temp dir and read every pack manifest found,
/// including those inside nested `.mcpack`/`.zip` archives.
pub fn extract_and_scan(archive_path: &Path) -> AppResult<PreparedAddon> {
    if !archive_path.is_file() {
        return Err(AppError::Validation("El archivo de addon no existe.".into()));
    }

    let temp_dir = std::env::temp_dir().join(format!("bsm_addon_{}", uuid::Uuid::new_v4()));
    archive::unzip(archive_path, &temp_dir)?;
    expand_nested_archives(&temp_dir);

    let mut packs = Vec::new();
    for entry in WalkDir::new(&temp_dir).into_iter().filter_map(|e| e.ok()) {
        if entry.file_type().is_file() && is_manifest(entry.path()) {
            // Skip a manifest we cannot read, but keep scanning the rest.
            if let Ok(manifest) = manifest_reader::read(entry.path()) {
                let dir = entry
                    .path()
                    .parent()
                    .map(|p| p.to_path_buf())
                    .unwrap_or_else(|| temp_dir.clone());
                let rel = dir
                    .strip_prefix(&temp_dir)
                    .map(|p| p.to_string_lossy().into_owned())
                    .unwrap_or_default();
                packs.push(PreparedPack { manifest, dir, rel });
            }
        }
    }

    if packs.is_empty() {
        let _ = std::fs::remove_dir_all(&temp_dir);
        return Err(AppError::Validation(
            "No se encontró ningún pack válido (manifest.json con UUID) en el archivo.".into(),
        ));
    }

    Ok(PreparedAddon { temp_dir, packs })
}

/// Build the preview model from a prepared addon.
pub fn to_package(archive_path: &Path, prepared: &PreparedAddon) -> AddonPackage {
    let display_name = archive_path
        .file_stem()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_else(|| "Addon".into());

    let packs = prepared
        .packs
        .iter()
        .map(|p| AddonPack {
            name: p.manifest.name.clone(),
            description: p.manifest.description.clone(),
            uuid: p.manifest.uuid.clone(),
            version: p.manifest.version.clone(),
            pack_type: p.manifest.pack_type,
            source_extracted_path: p.rel.clone(),
        })
        .collect();

    AddonPackage {
        id: uuid::Uuid::new_v4().to_string(),
        source_path: archive_path.to_string_lossy().into_owned(),
        display_name,
        packs,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Runs against the real sample addons in `<repo>/addons_tests` if present.
    /// Skips silently when the folder is absent (e.g. clean CI checkout).
    #[test]
    fn parses_sample_addons_including_nested_mcpacks() {
        let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../addons_tests");
        if !dir.is_dir() {
            eprintln!("addons_tests no presente — test omitido");
            return;
        }

        for entry in std::fs::read_dir(&dir).unwrap().filter_map(|e| e.ok()) {
            let path = entry.path();
            if !path.extension().map(|e| e == "mcaddon").unwrap_or(false) {
                continue;
            }
            let prepared = extract_and_scan(&path)
                .unwrap_or_else(|e| panic!("falló al parsear {}: {e}", path.display()));
            assert!(
                !prepared.packs.is_empty(),
                "no se detectaron packs en {}",
                path.display()
            );
            for p in &prepared.packs {
                assert!(!p.manifest.uuid.is_empty());
            }
            prepared.cleanup();
        }
    }
}
