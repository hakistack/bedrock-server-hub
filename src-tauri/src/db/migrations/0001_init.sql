-- Initial schema for Bedrock Server Manager.

CREATE TABLE IF NOT EXISTS servers (
    id              TEXT PRIMARY KEY,
    name            TEXT NOT NULL,
    path            TEXT NOT NULL UNIQUE,
    executable_path TEXT NOT NULL,
    properties_path TEXT NOT NULL,
    worlds_path     TEXT NOT NULL,
    created_at      TEXT NOT NULL,
    updated_at      TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS installed_addons (
    id           TEXT PRIMARY KEY,
    server_id    TEXT NOT NULL REFERENCES servers(id) ON DELETE CASCADE,
    world_name   TEXT NOT NULL,
    name         TEXT NOT NULL,
    uuid         TEXT NOT NULL,
    version      TEXT NOT NULL,
    pack_type    TEXT NOT NULL,
    installed_at TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_installed_addons_server
    ON installed_addons(server_id);

CREATE TABLE IF NOT EXISTS backups (
    id         TEXT PRIMARY KEY,
    server_id  TEXT NOT NULL REFERENCES servers(id) ON DELETE CASCADE,
    world_name TEXT,
    reason     TEXT NOT NULL,
    path       TEXT NOT NULL,
    created_at TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_backups_server
    ON backups(server_id);

CREATE TABLE IF NOT EXISTS app_settings (
    key   TEXT PRIMARY KEY,
    value TEXT NOT NULL
);
