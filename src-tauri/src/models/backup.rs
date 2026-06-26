use serde::{Deserialize, Serialize};

/// Reasons a backup is created. Kept as string constants so the value stored
/// in SQLite stays stable and human-readable.
#[allow(dead_code)] // ADDON_INSTALL/WORLD_IMPORT are reserved for phase 3 / future flows.
pub mod reason {
    pub const ADDON_INSTALL: &str = "addon_install";
    pub const WORLD_IMPORT: &str = "world_import";
    pub const PROPERTIES_EDIT: &str = "properties_edit";
    pub const MANUAL: &str = "manual";
    pub const PRE_RESTORE: &str = "pre_restore";
    pub const ADDON_UNINSTALL: &str = "addon_uninstall";
    pub const SCHEDULED: &str = "scheduled";
}

/// Per-server automated backup configuration (stored as JSON in app_settings).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BackupSchedule {
    pub enabled: bool,
    /// "interval" | "daily"
    pub mode: String,
    pub interval_minutes: u32,
    /// "HH:MM" local time for daily mode.
    pub daily_time: String,
    /// Keep at most this many scheduled backups (0 = unlimited).
    pub retention: u32,
}

impl Default for BackupSchedule {
    fn default() -> Self {
        Self {
            enabled: false,
            mode: "interval".into(),
            interval_minutes: 60,
            daily_time: "04:00".into(),
            retention: 7,
        }
    }
}

/// A backup taken before a modifying operation (or manually).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BackupRecord {
    pub id: String,
    pub server_id: String,
    pub world_name: Option<String>,
    pub reason: String,
    pub path: String,
    pub created_at: String,
}

/// Progress event emitted on `backup://progress` while zipping a world.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BackupProgress {
    pub backup_id: String,
    /// "starting" | "zipping" | "completed"
    pub phase: String,
    pub done: u64,
    pub total: u64,
}

/// Flags controlling what a restore touches.
#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RestoreOptions {
    pub restore_world: bool,
    pub restore_properties: bool,
}

impl Default for RestoreOptions {
    fn default() -> Self {
        Self {
            restore_world: true,
            restore_properties: true,
        }
    }
}
