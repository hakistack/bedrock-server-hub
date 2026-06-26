use serde::{Deserialize, Serialize};

/// The kind of Bedrock pack, derived from a manifest's module types.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PackType {
    Behavior,
    Resource,
    Skin,
    Unknown,
}

impl PackType {
    pub fn as_db_str(&self) -> &'static str {
        match self {
            PackType::Behavior => "behavior",
            PackType::Resource => "resource",
            PackType::Skin => "skin",
            PackType::Unknown => "unknown",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Dependency {
    pub uuid: String,
    pub version: Vec<i64>,
}

/// A parsed `manifest.json`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Manifest {
    pub name: String,
    pub description: Option<String>,
    pub uuid: String,
    pub version: Vec<i64>,
    pub pack_type: PackType,
    /// Raw module type strings found in the manifest (for transparency).
    pub module_types: Vec<String>,
    pub dependencies: Vec<Dependency>,
}
