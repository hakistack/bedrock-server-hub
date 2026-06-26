//! CRUD access for the `backups` table.

use rusqlite::{Connection, OptionalExtension, Row};

use crate::error::AppResult;
use crate::models::backup::BackupRecord;

fn map_row(row: &Row) -> rusqlite::Result<BackupRecord> {
    Ok(BackupRecord {
        id: row.get("id")?,
        server_id: row.get("server_id")?,
        world_name: row.get("world_name")?,
        reason: row.get("reason")?,
        path: row.get("path")?,
        created_at: row.get("created_at")?,
        size_bytes: None,
    })
}

pub fn insert(conn: &Connection, b: &BackupRecord) -> AppResult<()> {
    conn.execute(
        "INSERT INTO backups (id, server_id, world_name, reason, path, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        rusqlite::params![b.id, b.server_id, b.world_name, b.reason, b.path, b.created_at],
    )?;
    Ok(())
}

pub fn list_for_server(conn: &Connection, server_id: &str) -> AppResult<Vec<BackupRecord>> {
    let mut stmt = conn.prepare(
        "SELECT id, server_id, world_name, reason, path, created_at
         FROM backups WHERE server_id = ?1 ORDER BY created_at DESC",
    )?;
    let rows = stmt.query_map([server_id], map_row)?;
    let mut out = Vec::new();
    for r in rows {
        let mut rec = r?;
        rec.size_bytes = Some(crate::core::archive::dir_size(std::path::Path::new(&rec.path)));
        out.push(rec);
    }
    Ok(out)
}

pub fn get(conn: &Connection, id: &str) -> AppResult<Option<BackupRecord>> {
    let row = conn
        .query_row(
            "SELECT id, server_id, world_name, reason, path, created_at
             FROM backups WHERE id = ?1",
            [id],
            map_row,
        )
        .optional()?;
    Ok(row)
}

pub fn delete(conn: &Connection, id: &str) -> AppResult<()> {
    conn.execute("DELETE FROM backups WHERE id = ?1", [id])?;
    Ok(())
}
