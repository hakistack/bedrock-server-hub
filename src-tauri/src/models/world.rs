use serde::Serialize;

/// A world folder living under a server's `worlds/` directory.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct World {
    /// Folder name — this is what `level-name` in server.properties refers to.
    pub name: String,
    /// Friendly name from `levelname.txt`, if present.
    pub display_name: Option<String>,
    pub path: String,
    /// True when this is the server's active world (`level-name`).
    pub is_active: bool,
    pub size_bytes: u64,
    /// Whether the world already carries pack manifests (relevant for addons).
    pub has_behavior_packs: bool,
    pub has_resource_packs: bool,
}
