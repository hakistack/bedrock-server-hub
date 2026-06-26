//! Key/value access for the `app_settings` table.

use rusqlite::{Connection, OptionalExtension};

use crate::error::AppResult;

pub fn get(conn: &Connection, key: &str) -> AppResult<Option<String>> {
    let value = conn
        .query_row(
            "SELECT value FROM app_settings WHERE key = ?1",
            [key],
            |row| row.get::<_, String>(0),
        )
        .optional()?;
    Ok(value)
}

pub fn set(conn: &Connection, key: &str, value: &str) -> AppResult<()> {
    conn.execute(
        "INSERT INTO app_settings (key, value) VALUES (?1, ?2)
         ON CONFLICT(key) DO UPDATE SET value = excluded.value",
        rusqlite::params![key, value],
    )?;
    Ok(())
}

pub fn get_bool(conn: &Connection, key: &str) -> AppResult<bool> {
    Ok(get(conn, key)?.as_deref() == Some("true"))
}

pub fn set_bool(conn: &Connection, key: &str, value: bool) -> AppResult<()> {
    set(conn, key, if value { "true" } else { "false" })
}

/// Settings key for a server's crash auto-restart preference.
pub fn auto_restart_key(server_id: &str) -> String {
    format!("server.{server_id}.autoRestart")
}

/// Settings key for a server's backup schedule (stored as JSON).
pub fn backup_schedule_key(server_id: &str) -> String {
    format!("server.{server_id}.backupSchedule")
}
