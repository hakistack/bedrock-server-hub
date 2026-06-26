use serde::{Deserialize, Serialize};

use crate::models::manifest::PackType;

/// One addon to install in a batch run.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AddonInstallItem {
    pub source_path: String,
    /// Restrict to these pack UUIDs; `None` installs all supported packs.
    pub selected_uuids: Option<Vec<String>>,
}

/// A single pack detected inside an addon archive (for the preview UI).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AddonPack {
    pub name: String,
    pub description: Option<String>,
    pub uuid: String,
    pub version: Vec<i64>,
    pub pack_type: PackType,
    /// Path within the extracted archive (informational).
    pub source_extracted_path: String,
}

/// The result of inspecting an addon file before installation.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AddonPackage {
    pub id: String,
    /// The original archive path on disk.
    pub source_path: String,
    pub display_name: String,
    pub packs: Vec<AddonPack>,
}

/// Per-pack outcome of an install.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstalledPack {
    pub name: String,
    pub uuid: String,
    pub version: Vec<i64>,
    pub pack_type: PackType,
    /// "installed" | "updated" | "skipped" | "unsupported"
    pub status: String,
    pub message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AddonInstallReport {
    pub world_name: String,
    pub results: Vec<InstalledPack>,
    /// Non-fatal warnings, e.g. unsatisfied dependency UUIDs.
    pub warnings: Vec<String>,
}

/// A pack actually referenced by a world (read from world_*_packs.json),
/// in application order.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WorldPack {
    pub uuid: String,
    pub version: Vec<i64>,
    /// Resolved from the pack's manifest when its folder is found; else the uuid.
    pub name: String,
    /// "behavior" | "resource"
    pub pack_type: String,
    /// Whether the referenced pack folder exists on disk.
    pub present: bool,
}

/// The ordered packs of a single world, split by type.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WorldPacks {
    pub behavior: Vec<WorldPack>,
    pub resource: Vec<WorldPack>,
}

/// A record of an installed addon pack (from the database).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstalledAddon {
    pub id: String,
    pub server_id: String,
    pub world_name: String,
    pub name: String,
    pub uuid: String,
    pub version: String,
    pub pack_type: String,
    pub installed_at: String,
}
