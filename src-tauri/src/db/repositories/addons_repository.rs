//! CRUD access for the `installed_addons` table.

use rusqlite::{Connection, Row};

use crate::error::AppResult;
use crate::models::addon::InstalledAddon;

fn map_row(row: &Row) -> rusqlite::Result<InstalledAddon> {
    Ok(InstalledAddon {
        id: row.get("id")?,
        server_id: row.get("server_id")?,
        world_name: row.get("world_name")?,
        name: row.get("name")?,
        uuid: row.get("uuid")?,
        version: row.get("version")?,
        pack_type: row.get("pack_type")?,
        installed_at: row.get("installed_at")?,
    })
}

pub fn insert(conn: &Connection, addon: &InstalledAddon) -> AppResult<()> {
    conn.execute(
        "INSERT INTO installed_addons
            (id, server_id, world_name, name, uuid, version, pack_type, installed_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        rusqlite::params![
            addon.id,
            addon.server_id,
            addon.world_name,
            addon.name,
            addon.uuid,
            addon.version,
            addon.pack_type,
            addon.installed_at,
        ],
    )?;
    Ok(())
}

pub fn delete_by_uuid(
    conn: &Connection,
    server_id: &str,
    world_name: &str,
    uuid: &str,
) -> AppResult<()> {
    conn.execute(
        "DELETE FROM installed_addons
         WHERE server_id = ?1 AND world_name = ?2 AND uuid = ?3",
        rusqlite::params![server_id, world_name, uuid],
    )?;
    Ok(())
}

pub fn list_for_server(conn: &Connection, server_id: &str) -> AppResult<Vec<InstalledAddon>> {
    let mut stmt = conn.prepare(
        "SELECT id, server_id, world_name, name, uuid, version, pack_type, installed_at
         FROM installed_addons WHERE server_id = ?1 ORDER BY installed_at DESC",
    )?;
    let rows = stmt.query_map([server_id], map_row)?;
    let mut out = Vec::new();
    for r in rows {
        out.push(r?);
    }
    Ok(out)
}
