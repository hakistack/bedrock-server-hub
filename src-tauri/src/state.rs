use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::atomic::AtomicBool;
use std::sync::{Arc, Mutex};
use std::time::Instant;

use chrono::{DateTime, Local};
use rusqlite::Connection;

use crate::core::server_process::RunningServer;
use crate::models::player::Player;

/// Shared, app-lifetime state managed by Tauri.
///
/// `db` is a single connection behind a mutex — sufficient for this app's
/// concurrency profile (a desktop tool with one user). `processes` is the
/// registry of currently-running server processes, keyed by server id.
/// `downloads` maps an in-flight download id to a cancellation flag.
pub struct AppState {
    pub db: Mutex<Connection>,
    pub processes: Mutex<HashMap<String, RunningServer>>,
    pub downloads: Mutex<HashMap<String, Arc<AtomicBool>>>,
    /// Crash auto-restart bookkeeping: server id → (attempts, window start).
    pub crash_restarts: Mutex<HashMap<String, (u32, Instant)>>,
    /// Currently-connected players per server (derived from console logs).
    pub players: Mutex<HashMap<String, Vec<Player>>>,
    /// Last time a scheduled backup ran, per server.
    pub schedule_last_run: Mutex<HashMap<String, DateTime<Local>>>,
    /// Root folder for app data (portable: next to the exe on Windows).
    pub data_dir: PathBuf,
}

impl AppState {
    pub fn new(db: Connection, data_dir: PathBuf) -> Self {
        Self {
            db: Mutex::new(db),
            processes: Mutex::new(HashMap::new()),
            downloads: Mutex::new(HashMap::new()),
            crash_restarts: Mutex::new(HashMap::new()),
            players: Mutex::new(HashMap::new()),
            schedule_last_run: Mutex::new(HashMap::new()),
            data_dir,
        }
    }
}
