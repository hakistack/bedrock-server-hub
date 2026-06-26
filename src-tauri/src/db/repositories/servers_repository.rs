//! CRUD access for the `servers` table.

use rusqlite::{Connection, OptionalExtension, Row};

use crate::error::AppResult;
use crate::models::server::Server;

fn map_row(row: &Row) -> rusqlite::Result<Server> {
    Ok(Server {
        id: row.get("id")?,
        name: row.get("name")?,
        path: row.get("path")?,
        executable_path: row.get("executable_path")?,
        properties_path: row.get("properties_path")?,
        worlds_path: row.get("worlds_path")?,
        created_at: row.get("created_at")?,
        updated_at: row.get("updated_at")?,
        server_version: row.get("server_version")?,
        install_source: row.get("install_source")?,
        platform: row.get("platform")?,
        channel: row.get("channel")?,
        created_from_download: row.get::<_, i64>("created_from_download")? != 0,
    })
}

const SELECT_COLS: &str = "id, name, path, executable_path, properties_path, worlds_path,
        created_at, updated_at, server_version, install_source, platform, channel,
        created_from_download";

pub fn insert(conn: &Connection, server: &Server) -> AppResult<()> {
    conn.execute(
        "INSERT INTO servers
            (id, name, path, executable_path, properties_path, worlds_path, created_at, updated_at,
             server_version, install_source, platform, channel, created_from_download)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)",
        rusqlite::params![
            server.id,
            server.name,
            server.path,
            server.executable_path,
            server.properties_path,
            server.worlds_path,
            server.created_at,
            server.updated_at,
            server.server_version,
            server.install_source,
            server.platform,
            server.channel,
            server.created_from_download as i64,
        ],
    )?;
    Ok(())
}

pub fn list(conn: &Connection) -> AppResult<Vec<Server>> {
    let mut stmt =
        conn.prepare(&format!("SELECT {SELECT_COLS} FROM servers ORDER BY created_at ASC"))?;
    let rows = stmt.query_map([], map_row)?;
    let mut servers = Vec::new();
    for row in rows {
        servers.push(row?);
    }
    Ok(servers)
}

pub fn get(conn: &Connection, id: &str) -> AppResult<Option<Server>> {
    let server = conn
        .query_row(
            &format!("SELECT {SELECT_COLS} FROM servers WHERE id = ?1"),
            [id],
            map_row,
        )
        .optional()?;
    Ok(server)
}

pub fn find_by_path(conn: &Connection, path: &str) -> AppResult<Option<Server>> {
    let server = conn
        .query_row(
            &format!("SELECT {SELECT_COLS} FROM servers WHERE path = ?1"),
            [path],
            map_row,
        )
        .optional()?;
    Ok(server)
}

pub fn rename(conn: &Connection, id: &str, name: &str, updated_at: &str) -> AppResult<()> {
    conn.execute(
        "UPDATE servers SET name = ?2, updated_at = ?3 WHERE id = ?1",
        rusqlite::params![id, name, updated_at],
    )?;
    Ok(())
}

pub fn delete(conn: &Connection, id: &str) -> AppResult<()> {
    conn.execute("DELETE FROM servers WHERE id = ?1", [id])?;
    Ok(())
}
