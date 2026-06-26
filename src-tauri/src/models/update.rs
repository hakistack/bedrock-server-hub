use serde::Serialize;

/// Result of checking GitHub Releases for a newer version.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateInfo {
    /// True only when a newer version exists AND self-update is possible here.
    pub available: bool,
    pub current_version: String,
    pub latest_version: String,
    /// Release notes (markdown).
    pub notes: String,
    pub download_url: Option<String>,
    pub asset_size: Option<u64>,
    /// Whether in-app self-replace is supported on this platform (Windows).
    pub supported: bool,
}

/// Progress event emitted on `update://progress` while downloading the update.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateProgress {
    pub downloaded_bytes: u64,
    pub total_bytes: Option<u64>,
    pub percentage: Option<f64>,
}
