use serde::Serialize;

/// A port the server listens on, with its firewall-rule state.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PortRule {
    /// "IPv4" / "IPv6"
    pub label: String,
    pub port: u16,
    pub protocol: String,
    /// The deterministic firewall rule name we manage.
    pub rule_name: String,
    /// Whether that rule currently exists (always false on non-Windows).
    pub rule_exists: bool,
    /// The properties key this port comes from.
    pub key: String,
}

/// A port clash with another registered server.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PortConflict {
    pub port: u16,
    pub other_server: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NetworkStatus {
    /// True only where firewall management is implemented (Windows).
    pub firewall_supported: bool,
    /// "windows" | "macos" | "linux" | ...
    pub platform: String,
    pub ports: Vec<PortRule>,
    pub conflicts: Vec<PortConflict>,
}
