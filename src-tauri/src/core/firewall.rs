//! Windows firewall rule management for Bedrock server ports.
//!
//! Bedrock uses **UDP** (default 19132 IPv4 / 19133 IPv6). We create inbound
//! allow rules named deterministically per server+port so we can detect and
//! manage them. Checking rules needs no elevation; **adding** them does, so we
//! relaunch `netsh` through an elevated PowerShell (one UAC prompt for all).
//!
//! On non-Windows platforms these are graceful no-ops / unsupported errors.

#[cfg(windows)]
use std::process::Command;

use crate::error::AppResult;

/// True only where firewall management is implemented.
pub fn supported() -> bool {
    cfg!(windows)
}

pub fn platform() -> &'static str {
    std::env::consts::OS
}

/// Deterministic, space-free rule name (avoids netsh quoting issues).
pub fn rule_name(server_name: &str, port: u16) -> String {
    let cleaned: String = server_name
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() || c == '-' { c } else { '_' })
        .collect();
    let trimmed = cleaned.trim_matches('_');
    let base = if trimmed.is_empty() { "server" } else { trimmed };
    format!("BSM_{base}_{port}")
}

#[cfg(windows)]
pub fn rule_exists(name: &str) -> bool {
    use std::os::windows::process::CommandExt;
    const CREATE_NO_WINDOW: u32 = 0x0800_0000;
    Command::new("netsh")
        .args(["advfirewall", "firewall", "show", "rule"])
        .arg(format!("name={name}"))
        .creation_flags(CREATE_NO_WINDOW)
        .output()
        .map(|o| o.status.success() && !String::from_utf8_lossy(&o.stdout).contains("No rules"))
        .unwrap_or(false)
}

#[cfg(not(windows))]
pub fn rule_exists(_name: &str) -> bool {
    false
}

/// Add inbound UDP allow rules for the given `(rule_name, port)` pairs.
/// Runs a single elevated PowerShell (one UAC prompt) on Windows.
#[cfg(windows)]
pub fn add_rules(rules: &[(String, u16)]) -> AppResult<()> {
    use std::os::windows::process::CommandExt;
    use crate::error::AppError;
    const CREATE_NO_WINDOW: u32 = 0x0800_0000;

    if rules.is_empty() {
        return Ok(());
    }

    let inner: String = rules
        .iter()
        .map(|(name, port)| {
            format!("netsh advfirewall firewall add rule name={name} dir=in action=allow protocol=UDP localport={port}")
        })
        .collect::<Vec<_>>()
        .join("; ");

    let ps = format!(
        "Start-Process -FilePath powershell -Verb RunAs -WindowStyle Hidden -Wait \
         -ArgumentList '-NoProfile','-Command','{}'",
        inner.replace('\'', "''")
    );

    let status = Command::new("powershell")
        .args(["-NoProfile", "-WindowStyle", "Hidden", "-Command", &ps])
        .creation_flags(CREATE_NO_WINDOW)
        .status()
        .map_err(|e| AppError::Process(format!("No se pudo invocar PowerShell: {e}")))?;

    if !status.success() {
        return Err(AppError::Process(
            "No se pudieron agregar las reglas de firewall (UAC cancelado o error).".into(),
        ));
    }
    Ok(())
}

#[cfg(not(windows))]
pub fn add_rules(_rules: &[(String, u16)]) -> AppResult<()> {
    Err(crate::error::AppError::Validation(
        "La gestión del firewall solo está disponible en Windows.".into(),
    ))
}
