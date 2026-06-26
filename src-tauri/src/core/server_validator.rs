//! Validation of a Bedrock Dedicated Server folder layout.
//!
//! Shared by the "import existing server" flow and the download/install wizard
//! so both judge a valid installation the exact same way.

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::error::{AppError, AppResult};

/// Candidate executable file names across platforms.
pub const EXECUTABLE_NAMES: &[&str] = &["bedrock_server.exe", "bedrock_server"];

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ServerValidationResult {
    pub is_valid: bool,
    pub executable_path: Option<String>,
    pub properties_path: Option<String>,
    pub worlds_path: Option<String>,
    pub issues: Vec<String>,
}

/// Concrete resolved paths for a valid installation.
pub struct ResolvedPaths {
    pub executable: PathBuf,
    pub properties: PathBuf,
    pub worlds: PathBuf,
}

pub fn find_executable(dir: &Path) -> Option<PathBuf> {
    EXECUTABLE_NAMES
        .iter()
        .map(|name| dir.join(name))
        .find(|p| p.is_file())
}

/// Non-fatal inspection — collects issues instead of erroring, so the UI can
/// show the user exactly what is missing.
pub fn validate(dir: &Path) -> ServerValidationResult {
    let mut issues = Vec::new();

    if !dir.is_dir() {
        return ServerValidationResult {
            is_valid: false,
            executable_path: None,
            properties_path: None,
            worlds_path: None,
            issues: vec!["La ruta seleccionada no es una carpeta accesible.".into()],
        };
    }

    let executable = find_executable(dir);
    if executable.is_none() {
        issues.push(
            "No se encontró el ejecutable del servidor (bedrock_server / bedrock_server.exe)."
                .into(),
        );
    }

    let properties = dir.join("server.properties");
    if !properties.is_file() {
        issues.push("No se encontró server.properties.".into());
    }

    // A missing `worlds` folder is not fatal — it can be created on import.
    let worlds = dir.join("worlds");

    ServerValidationResult {
        is_valid: issues.is_empty(),
        executable_path: executable.as_ref().map(|p| p.to_string_lossy().into_owned()),
        properties_path: properties
            .is_file()
            .then(|| properties.to_string_lossy().into_owned()),
        worlds_path: Some(worlds.to_string_lossy().into_owned()),
        issues,
    }
}

/// Strict resolution used before registering a server. Creates the `worlds`
/// folder if missing (also confirming write access) and errors on any problem.
pub fn resolve(dir: &Path) -> AppResult<ResolvedPaths> {
    if !dir.is_dir() {
        return Err(AppError::Validation(
            "La ruta seleccionada no es una carpeta accesible.".into(),
        ));
    }

    let executable = find_executable(dir).ok_or_else(|| {
        AppError::Validation(
            "No se encontró el ejecutable del servidor (bedrock_server / bedrock_server.exe)."
                .into(),
        )
    })?;

    let properties = dir.join("server.properties");
    if !properties.is_file() {
        return Err(AppError::Validation(
            "No se encontró server.properties en la carpeta.".into(),
        ));
    }

    let worlds = dir.join("worlds");
    if !worlds.exists() {
        std::fs::create_dir_all(&worlds).map_err(|e| {
            AppError::Validation(format!(
                "No se pudo crear/escribir la carpeta worlds (¿permisos?): {e}"
            ))
        })?;
    }

    Ok(ResolvedPaths {
        executable,
        properties,
        worlds,
    })
}
