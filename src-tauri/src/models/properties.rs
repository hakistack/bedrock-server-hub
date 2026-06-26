use serde::{Deserialize, Serialize};

/// A single `key=value` pair from `server.properties`, surfaced to the UI.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PropertyEntry {
    pub key: String,
    pub value: String,
}

/// A change requested by the frontend for a given property key.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PropertyUpdate {
    pub key: String,
    pub value: String,
}
