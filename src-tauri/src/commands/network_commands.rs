//! Tauri commands for port assignment and firewall rule management.

use std::collections::HashSet;
use std::path::Path;

use rusqlite::Connection;
use tauri::{AppHandle, State};

use crate::commands::backup_commands::make_backup;
use crate::core::{firewall, server_properties};
use crate::db::repositories::servers_repository;
use crate::error::{AppError, AppResult};
use crate::models::backup::reason;
use crate::models::network::{NetworkStatus, PortConflict, PortRule};
use crate::models::properties::PropertyUpdate;
use crate::models::server::Server;
use crate::state::AppState;

const PORT_KEY: &str = "server-port";
const PORTV6_KEY: &str = "server-portv6";
const DEFAULT_PORT: u16 = 19132;
const DEFAULT_PORTV6: u16 = 19133;

fn load_server(conn: &Connection, server_id: &str) -> AppResult<Server> {
    servers_repository::get(conn, server_id)?
        .ok_or_else(|| AppError::NotFound("Servidor no encontrado.".into()))
}

/// Ports used by all servers except `exclude_id` (read from their properties).
pub(crate) fn collect_used_ports(conn: &Connection, exclude_id: &str) -> HashSet<u16> {
    let mut used = HashSet::new();
    if let Ok(servers) = servers_repository::list(conn) {
        for s in servers {
            if s.id == exclude_id {
                continue;
            }
            let (a, b) = server_ports(&s);
            used.insert(a);
            used.insert(b);
        }
    }
    used
}

/// Lowest free even base where `base` and `base+1` are both unused.
pub(crate) fn free_port_pair(used: &HashSet<u16>) -> (u16, u16) {
    let mut base = DEFAULT_PORT;
    while base < 65000 && (used.contains(&base) || used.contains(&(base + 1))) {
        base += 2;
    }
    (base, base + 1)
}

/// Write the v4/v6 ports into a server's properties (no backup).
pub(crate) fn write_ports(server: &Server, v4: u16, v6: u16) -> AppResult<()> {
    server_properties::update(
        Path::new(&server.properties_path),
        &[
            PropertyUpdate {
                key: PORT_KEY.into(),
                value: v4.to_string(),
            },
            PropertyUpdate {
                key: PORTV6_KEY.into(),
                value: v6.to_string(),
            },
        ],
    )?;
    Ok(())
}

/// Read (server-port, server-portv6) from a server's properties, with defaults.
fn server_ports(server: &Server) -> (u16, u16) {
    let entries = server_properties::read(Path::new(&server.properties_path)).unwrap_or_default();
    let get = |key: &str, default: u16| -> u16 {
        entries
            .iter()
            .find(|e| e.key == key)
            .and_then(|e| e.value.trim().parse().ok())
            .unwrap_or(default)
    };
    (get(PORT_KEY, DEFAULT_PORT), get(PORTV6_KEY, DEFAULT_PORTV6))
}

fn build_status(server: &Server, others: &[Server]) -> NetworkStatus {
    let (v4, v6) = server_ports(server);

    let ports = vec![
        PortRule {
            label: "IPv4".into(),
            port: v4,
            protocol: "UDP".into(),
            rule_name: firewall::rule_name(&server.name, v4),
            rule_exists: firewall::rule_exists(&firewall::rule_name(&server.name, v4)),
            key: PORT_KEY.into(),
        },
        PortRule {
            label: "IPv6".into(),
            port: v6,
            protocol: "UDP".into(),
            rule_name: firewall::rule_name(&server.name, v6),
            rule_exists: firewall::rule_exists(&firewall::rule_name(&server.name, v6)),
            key: PORTV6_KEY.into(),
        },
    ];

    // Conflicts: any other server sharing one of our ports.
    let mine = [v4, v6];
    let mut conflicts = Vec::new();
    for other in others {
        if other.id == server.id {
            continue;
        }
        let (o4, o6) = server_ports(other);
        for p in [o4, o6] {
            if mine.contains(&p) {
                conflicts.push(PortConflict {
                    port: p,
                    other_server: other.name.clone(),
                });
            }
        }
    }

    NetworkStatus {
        firewall_supported: firewall::supported(),
        platform: firewall::platform().to_string(),
        ports,
        conflicts,
    }
}

#[tauri::command]
pub fn get_network_status(state: State<AppState>, server_id: String) -> AppResult<NetworkStatus> {
    let conn = state.db.lock().unwrap();
    let server = load_server(&conn, &server_id)?;
    let others = servers_repository::list(&conn)?;
    drop(conn);
    Ok(build_status(&server, &others))
}

/// Create the missing inbound UDP firewall rules for this server's ports.
#[tauri::command]
pub fn add_firewall_rules(
    state: State<AppState>,
    server_id: String,
) -> AppResult<NetworkStatus> {
    if !firewall::supported() {
        return Err(AppError::Validation(
            "La gestión del firewall solo está disponible en Windows.".into(),
        ));
    }

    let conn = state.db.lock().unwrap();
    let server = load_server(&conn, &server_id)?;
    let others = servers_repository::list(&conn)?;
    drop(conn);

    let (v4, v6) = server_ports(&server);
    let mut missing = Vec::new();
    for port in [v4, v6] {
        let name = firewall::rule_name(&server.name, port);
        if !firewall::rule_exists(&name) {
            missing.push((name, port));
        }
    }

    firewall::add_rules(&missing)?;
    Ok(build_status(&server, &others))
}

/// Assign the lowest free port pair (UDP v4 + v6) not used by any other
/// registered server, writing it to this server's properties (backed up first).
#[tauri::command]
pub fn assign_free_port(
    app: AppHandle,
    state: State<AppState>,
    server_id: String,
) -> AppResult<NetworkStatus> {
    let conn = state.db.lock().unwrap();
    let server = load_server(&conn, &server_id)?;
    let others = servers_repository::list(&conn)?;

    let used = collect_used_ports(&conn, &server.id);
    let (v4, v6) = free_port_pair(&used);

    make_backup(&app, &conn, &server, None, reason::PROPERTIES_EDIT)?;
    write_ports(&server, v4, v6)?;
    drop(conn);

    Ok(build_status(&server, &others))
}
