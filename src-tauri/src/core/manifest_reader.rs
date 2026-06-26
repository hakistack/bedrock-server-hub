//! Parses a Bedrock `manifest.json` into a structured [`Manifest`].
//!
//! Bedrock manifests vary between `format_version` 1 and 2 and across pack
//! types, so we navigate the JSON defensively rather than deserializing into a
//! rigid struct: a missing optional field must never fail the whole read.

use std::path::Path;

use serde_json::Value;

use crate::error::{AppError, AppResult};
use crate::models::manifest::{Dependency, Manifest, PackType};

/// Normalize a version field (array `[1,0,0]` or string `"1.0.0"`) to `Vec<i64>`.
fn parse_version(value: Option<&Value>) -> Vec<i64> {
    match value {
        Some(Value::Array(arr)) => arr
            .iter()
            .map(|v| v.as_i64().or_else(|| v.as_f64().map(|f| f as i64)).unwrap_or(0))
            .collect(),
        Some(Value::String(s)) => s.split('.').filter_map(|p| p.parse::<i64>().ok()).collect(),
        _ => vec![1, 0, 0],
    }
}

/// Classify the pack from its module type strings.
fn determine_pack_type(module_types: &[String]) -> PackType {
    if module_types.iter().any(|t| t == "resources") {
        PackType::Resource
    } else if module_types
        .iter()
        .any(|t| matches!(t.as_str(), "data" | "script" | "client_data"))
    {
        PackType::Behavior
    } else if module_types.iter().any(|t| t.contains("skin")) {
        PackType::Skin
    } else {
        PackType::Unknown
    }
}

pub fn read(manifest_path: &Path) -> AppResult<Manifest> {
    let content = std::fs::read_to_string(manifest_path)
        .map_err(|e| AppError::Io(format!("No se pudo leer manifest.json: {e}")))?;
    let json: Value = serde_json::from_str(&content)
        .map_err(|e| AppError::Validation(format!("manifest.json inválido: {e}")))?;

    let header = json
        .get("header")
        .ok_or_else(|| AppError::Validation("El manifest no tiene 'header'.".into()))?;

    let uuid = header
        .get("uuid")
        .and_then(|v| v.as_str())
        .filter(|s| !s.is_empty())
        .ok_or_else(|| AppError::Validation("El manifest no tiene UUID (header.uuid).".into()))?
        .to_string();

    let name = header
        .get("name")
        .and_then(|v| v.as_str())
        .unwrap_or("Pack sin nombre")
        .to_string();

    let description = header
        .get("description")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    let version = parse_version(header.get("version"));

    let module_types: Vec<String> = json
        .get("modules")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|m| m.get("type").and_then(|t| t.as_str()).map(String::from))
                .collect()
        })
        .unwrap_or_default();

    let dependencies: Vec<Dependency> = json
        .get("dependencies")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|d| {
                    let uuid = d.get("uuid").and_then(|u| u.as_str())?.to_string();
                    Some(Dependency {
                        uuid,
                        version: parse_version(d.get("version")),
                    })
                })
                .collect()
        })
        .unwrap_or_default();

    Ok(Manifest {
        name,
        description,
        uuid,
        version,
        pack_type: determine_pack_type(&module_types),
        module_types,
        dependencies,
    })
}
