use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ServerPlatform {
    Windows,
    Linux,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ServerChannel {
    Stable,
    Preview,
}

/// A resolved official download option presented to the user.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ServerDownloadOption {
    pub id: String,
    pub label: String,
    pub platform: ServerPlatform,
    pub channel: ServerChannel,
    pub url: String,
    /// Parsed from the file name, e.g. "1.26.32.2".
    pub version: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DownloadStatus {
    Starting,
    Downloading,
    Completed,
    Failed,
    Cancelled,
}

/// Progress event emitted on `download://progress`.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DownloadProgress {
    pub download_id: String,
    pub bytes_downloaded: u64,
    pub total_bytes: Option<u64>,
    pub percentage: Option<f64>,
    pub status: DownloadStatus,
}

/// Result of a successful download, fed into the installer.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DownloadedServerPackage {
    pub download_id: String,
    pub path: String,
    pub size_bytes: u64,
    pub option: ServerDownloadOption,
}
