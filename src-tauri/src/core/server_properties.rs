//! A non-destructive `server.properties` parser.
//!
//! `server.properties` is a line-oriented `key=value` file that also carries
//! comments (`#`) the user may have written. We model every line so that
//! updating a value never reorders entries, drops comments, or mangles blanks.

use std::path::Path;

use crate::error::{AppError, AppResult};
use crate::models::properties::{PropertyEntry, PropertyUpdate};

/// One physical line of the file, preserved verbatim where it carries no value.
#[derive(Debug, Clone)]
enum Line {
    Comment(String),
    Blank,
    Entry { key: String, value: String },
}

#[derive(Debug, Clone, Default)]
pub struct PropertiesFile {
    lines: Vec<Line>,
}

impl PropertiesFile {
    pub fn parse(content: &str) -> Self {
        let mut lines = Vec::new();
        for raw in content.lines() {
            let trimmed = raw.trim();
            if trimmed.is_empty() {
                lines.push(Line::Blank);
            } else if trimmed.starts_with('#') || trimmed.starts_with('!') {
                lines.push(Line::Comment(raw.to_string()));
            } else if let Some((key, value)) = raw.split_once('=') {
                lines.push(Line::Entry {
                    key: key.trim().to_string(),
                    value: value.trim().to_string(),
                });
            } else {
                // No '=' and not a comment: keep it verbatim as a comment-like
                // line so we never lose user content.
                lines.push(Line::Comment(raw.to_string()));
            }
        }
        Self { lines }
    }

    /// All key/value entries, in file order.
    pub fn entries(&self) -> Vec<PropertyEntry> {
        self.lines
            .iter()
            .filter_map(|l| match l {
                Line::Entry { key, value } => Some(PropertyEntry {
                    key: key.clone(),
                    value: value.clone(),
                }),
                _ => None,
            })
            .collect()
    }

    /// Update an existing key, or append it if it does not exist yet.
    pub fn set(&mut self, key: &str, value: &str) {
        for line in &mut self.lines {
            if let Line::Entry { key: k, value: v } = line {
                if k == key {
                    *v = value.to_string();
                    return;
                }
            }
        }
        self.lines.push(Line::Entry {
            key: key.to_string(),
            value: value.to_string(),
        });
    }

    pub fn to_text(&self) -> String {
        let mut out = String::new();
        for line in &self.lines {
            match line {
                Line::Comment(c) => out.push_str(c),
                Line::Blank => {}
                Line::Entry { key, value } => {
                    out.push_str(key);
                    out.push('=');
                    out.push_str(value);
                }
            }
            out.push('\n');
        }
        out
    }
}

/// Read and parse all entries from a `server.properties` file.
pub fn read(path: &Path) -> AppResult<Vec<PropertyEntry>> {
    let content = std::fs::read_to_string(path).map_err(|e| {
        AppError::Io(format!("No se pudo leer server.properties: {e}"))
    })?;
    Ok(PropertiesFile::parse(&content).entries())
}

/// Apply a set of updates and write the file back, preserving structure.
/// Returns the resulting entries.
pub fn update(path: &Path, updates: &[PropertyUpdate]) -> AppResult<Vec<PropertyEntry>> {
    let content = std::fs::read_to_string(path).map_err(|e| {
        AppError::Io(format!("No se pudo leer server.properties: {e}"))
    })?;
    let mut file = PropertiesFile::parse(&content);
    for update in updates {
        file.set(&update.key, &update.value);
    }
    std::fs::write(path, file.to_text()).map_err(|e| {
        AppError::Io(format!("No se pudo escribir server.properties: {e}"))
    })?;
    Ok(file.entries())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn preserves_comments_and_updates_in_place() {
        let input = "# header comment\nserver-name=Dedicated Server\n\ngamemode=survival\n";
        let mut f = PropertiesFile::parse(input);
        f.set("gamemode", "creative");
        let out = f.to_text();
        assert!(out.contains("# header comment"));
        assert!(out.contains("server-name=Dedicated Server"));
        assert!(out.contains("gamemode=creative"));
        assert!(!out.contains("gamemode=survival"));
    }

    #[test]
    fn appends_missing_key() {
        let mut f = PropertiesFile::parse("server-name=Test\n");
        f.set("max-players", "20");
        assert!(f.to_text().contains("max-players=20"));
    }
}
