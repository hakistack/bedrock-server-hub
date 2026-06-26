-- Optional provenance metadata for servers (download wizard).

ALTER TABLE servers ADD COLUMN server_version TEXT;
ALTER TABLE servers ADD COLUMN install_source TEXT;
ALTER TABLE servers ADD COLUMN platform TEXT;
ALTER TABLE servers ADD COLUMN channel TEXT;
ALTER TABLE servers ADD COLUMN created_from_download INTEGER NOT NULL DEFAULT 0;
