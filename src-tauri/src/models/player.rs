use serde::{Deserialize, Serialize};

/// A player currently connected to a server (derived from console logs).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Player {
    pub name: String,
    pub xuid: String,
    pub connected_at: String,
}

/// A connect/disconnect event emitted on `server://player`.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlayerEvent {
    pub server_id: String,
    /// "connected" | "disconnected"
    pub event: String,
    pub name: String,
    pub xuid: String,
    pub at: String,
}
