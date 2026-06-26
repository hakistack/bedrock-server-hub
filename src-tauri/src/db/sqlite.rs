//! SQLite connection setup and lightweight versioned migrations.
//!
//! Migrations are embedded at compile time and applied based on the database's
//! `user_version` pragma. Adding a migration = bump `LATEST_VERSION`, append
//! the `(version, sql)` pair, and ship the `.sql` file.

use std::path::Path;

use rusqlite::Connection;

use crate::error::{AppError, AppResult};

const LATEST_VERSION: i64 = 2;

const MIGRATIONS: &[(i64, &str)] = &[
    (1, include_str!("migrations/0001_init.sql")),
    (2, include_str!("migrations/0002_server_metadata.sql")),
];

/// Open (creating if needed) the database and bring it up to the latest schema.
pub fn open(db_path: &Path) -> AppResult<Connection> {
    let conn = Connection::open(db_path)
        .map_err(|e| AppError::Db(format!("No se pudo abrir la base de datos: {e}")))?;

    conn.execute_batch("PRAGMA journal_mode = WAL; PRAGMA foreign_keys = ON;")
        .map_err(|e| AppError::Db(e.to_string()))?;

    run_migrations(&conn)?;
    Ok(conn)
}

fn run_migrations(conn: &Connection) -> AppResult<()> {
    let mut version: i64 = conn.query_row("PRAGMA user_version", [], |r| r.get(0))?;

    for (target, sql) in MIGRATIONS {
        if *target > version {
            conn.execute_batch(sql)
                .map_err(|e| AppError::Db(format!("Migración {target} falló: {e}")))?;
            version = *target;
        }
    }

    if version != LATEST_VERSION {
        // Defensive: keep the pragma aligned even if a no-op path is taken.
        version = LATEST_VERSION;
    }
    conn.execute_batch(&format!("PRAGMA user_version = {version};"))
        .map_err(|e| AppError::Db(e.to_string()))?;
    Ok(())
}
