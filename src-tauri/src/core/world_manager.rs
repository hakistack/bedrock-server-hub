//! Listing, importing and activating Bedrock worlds.

use std::path::{Path, PathBuf};

use walkdir::WalkDir;

use crate::core::archive;
use crate::error::{AppError, AppResult};
use crate::models::world::World;

/// A `.mcworld` extracted to a temp location, ready to be moved into `worlds/`.
pub struct ExtractedWorld {
    /// Temp directory to remove once the import finishes (success or failure).
    pub temp_dir: PathBuf,
    /// The folder inside `temp_dir` that actually holds `level.dat`.
    pub root: PathBuf,
    /// Filesystem-safe folder name suggested for this world.
    pub suggested_name: String,
}

fn read_levelname(world_dir: &Path) -> Option<String> {
    std::fs::read_to_string(world_dir.join("levelname.txt"))
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
}

/// Replace characters that are awkward in folder names.
fn sanitize_folder_name(raw: &str) -> String {
    let cleaned: String = raw
        .chars()
        .map(|c| match c {
            'a'..='z' | 'A'..='Z' | '0'..='9' | ' ' | '-' | '_' | '.' => c,
            _ => '_',
        })
        .collect();
    let trimmed = cleaned.trim().trim_matches('.').trim();
    if trimmed.is_empty() {
        "Imported World".to_string()
    } else {
        trimmed.to_string()
    }
}

/// List every world folder under `worlds_path`, flagging the active one.
pub fn list_worlds(worlds_path: &Path, active_level_name: &str) -> AppResult<Vec<World>> {
    let mut worlds = Vec::new();
    if !worlds_path.is_dir() {
        return Ok(worlds);
    }

    for entry in std::fs::read_dir(worlds_path)? {
        let entry = entry?;
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let name = entry.file_name().to_string_lossy().into_owned();

        let modified_at = std::fs::metadata(&path)
            .and_then(|m| m.modified())
            .ok()
            .map(|t| chrono::DateTime::<chrono::Utc>::from(t).to_rfc3339());

        worlds.push(World {
            display_name: read_levelname(&path),
            is_active: name == active_level_name,
            size_bytes: archive::dir_size(&path),
            modified_at,
            has_behavior_packs: path.join("world_behavior_packs.json").is_file(),
            has_resource_packs: path.join("world_resource_packs.json").is_file(),
            path: path.to_string_lossy().into_owned(),
            name,
        });
    }

    worlds.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    Ok(worlds)
}

/// Locate the directory containing `level.dat` within an extracted archive.
fn find_world_root(base: &Path) -> Option<PathBuf> {
    for entry in WalkDir::new(base).max_depth(3).into_iter().filter_map(|e| e.ok()) {
        if entry.file_type().is_file()
            && entry.file_name().to_string_lossy().eq_ignore_ascii_case("level.dat")
        {
            return entry.path().parent().map(|p| p.to_path_buf());
        }
    }
    None
}

/// Extract a `.mcworld`/`.zip` to a temp dir and validate it is a real world.
pub fn extract_world(mcworld_path: &Path) -> AppResult<ExtractedWorld> {
    if !mcworld_path.is_file() {
        return Err(AppError::Validation(
            "El archivo de mundo seleccionado no existe.".into(),
        ));
    }

    let temp_dir = std::env::temp_dir().join(format!("bsm_world_{}", uuid::Uuid::new_v4()));
    archive::unzip(mcworld_path, &temp_dir)?;

    let root = find_world_root(&temp_dir).ok_or_else(|| {
        // Clean up before bailing out.
        let _ = std::fs::remove_dir_all(&temp_dir);
        AppError::Validation(
            "El archivo no parece un mundo válido (no se encontró level.dat).".into(),
        )
    })?;

    let stem = mcworld_path
        .file_stem()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_else(|| "Imported World".into());
    let suggested_name = sanitize_folder_name(&stem);

    Ok(ExtractedWorld {
        temp_dir,
        root,
        suggested_name,
    })
}

/// Pick a non-colliding folder name under `worlds_path` based on `desired`.
pub fn unique_world_name(worlds_path: &Path, desired: &str) -> String {
    if !worlds_path.join(desired).exists() {
        return desired.to_string();
    }
    for n in 2..1000 {
        let candidate = format!("{desired} ({n})");
        if !worlds_path.join(&candidate).exists() {
            return candidate;
        }
    }
    format!("{desired}_{}", uuid::Uuid::new_v4())
}
