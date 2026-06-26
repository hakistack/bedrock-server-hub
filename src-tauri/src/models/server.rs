use serde::{Deserialize, Serialize};

/// A registered Bedrock Dedicated Server installation on disk.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Server {
    pub id: String,
    pub name: String,
    /// Root folder of the server installation (also the process working dir).
    pub path: String,
    /// Absolute path to the `bedrock_server` executable.
    pub executable_path: String,
    /// Absolute path to `server.properties`.
    pub properties_path: String,
    /// Absolute path to the `worlds` directory.
    pub worlds_path: String,
    pub created_at: String,
    pub updated_at: String,

    // --- Provenance metadata (optional; set by the download wizard) ---
    /// e.g. "1.26.32.2" when known.
    pub server_version: Option<String>,
    /// "import" | "official_download".
    pub install_source: Option<String>,
    /// "Windows" | "Linux".
    pub platform: Option<String>,
    /// "Stable" | "Preview".
    pub channel: Option<String>,
    pub created_from_download: bool,
}
