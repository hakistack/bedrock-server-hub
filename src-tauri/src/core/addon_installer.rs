//! Installs prepared packs into a server and wires them into the active world.
//!
//! Behavior packs go to `<server>/behavior_packs/<folder>` and resource packs
//! to `<server>/resource_packs/<folder>`. The world then references them via
//! `world_behavior_packs.json` / `world_resource_packs.json`, deduplicated by
//! pack UUID so re-installing never produces duplicate entries.

use std::collections::{HashMap, HashSet};
use std::path::Path;

use serde::{Deserialize, Serialize};
use serde_json::Value;
use walkdir::WalkDir;

use crate::core::addon_parser::PreparedAddon;
use crate::core::{archive, manifest_reader};
use crate::error::{AppError, AppResult};
use crate::models::addon::{InstalledPack, WorldPack, WorldPacks};
use crate::models::manifest::PackType;
use crate::models::server::Server;

const BEHAVIOR_JSON: &str = "world_behavior_packs.json";
const RESOURCE_JSON: &str = "world_resource_packs.json";

/// One entry of a `world_*_packs.json` file.
#[derive(Debug, Serialize, Deserialize)]
struct PackRef {
    pack_id: String,
    version: Vec<i64>,
    /// Preserve any extra keys some packs carry (e.g. "subpack").
    #[serde(flatten)]
    extra: serde_json::Map<String, Value>,
}

fn sanitize(name: &str) -> String {
    let s: String = name
        .chars()
        .map(|c| match c {
            'a'..='z' | 'A'..='Z' | '0'..='9' | '-' | '_' => c,
            _ => '_',
        })
        .collect();
    let trimmed = s.trim_matches('_');
    if trimmed.is_empty() {
        "pack".to_string()
    } else {
        trimmed.to_string()
    }
}

fn read_pack_refs(path: &Path) -> Vec<PackRef> {
    std::fs::read_to_string(path)
        .ok()
        .and_then(|c| serde_json::from_str::<Vec<PackRef>>(&c).ok())
        .unwrap_or_default()
}

/// Add a pack reference to a world json file, deduped by uuid.
/// Returns true if newly added, false if it was already present.
fn add_pack_ref(world_json: &Path, uuid: &str, version: &[i64]) -> AppResult<bool> {
    let mut refs = read_pack_refs(world_json);
    if refs.iter().any(|r| r.pack_id == uuid) {
        return Ok(false);
    }
    refs.push(PackRef {
        pack_id: uuid.to_string(),
        version: version.to_vec(),
        extra: serde_json::Map::new(),
    });
    write_pack_refs(world_json, &refs)
}

fn write_pack_refs(world_json: &Path, refs: &[PackRef]) -> AppResult<bool> {
    let json = serde_json::to_string_pretty(refs)?;
    std::fs::write(world_json, json)
        .map_err(|e| AppError::Io(format!("No se pudo escribir {}: {e}", world_json.display())))?;
    Ok(true)
}

/// Remove a pack reference (by uuid) from a world json file. Returns true if
/// an entry was removed.
fn remove_pack_ref(world_json: &Path, uuid: &str) -> AppResult<bool> {
    if !world_json.is_file() {
        return Ok(false);
    }
    let mut refs = read_pack_refs(world_json);
    let before = refs.len();
    refs.retain(|r| r.pack_id != uuid);
    if refs.len() == before {
        return Ok(false);
    }
    write_pack_refs(world_json, &refs)?;
    Ok(true)
}

/// All pack UUIDs already referenced by a world (both json files).
pub fn world_pack_ids(server: &Server, world_name: &str) -> HashSet<String> {
    let world_dir = Path::new(&server.worlds_path).join(world_name);
    let mut ids = HashSet::new();
    for file in [BEHAVIOR_JSON, RESOURCE_JSON] {
        for r in read_pack_refs(&world_dir.join(file)) {
            ids.insert(r.pack_id);
        }
    }
    ids
}

/// Find and delete the pack folder whose manifest UUID matches, scanning the
/// server's `behavior_packs` and `resource_packs`. Returns true if removed.
fn delete_pack_folder(server: &Server, uuid: &str) -> bool {
    let server_root = Path::new(&server.path);
    for sub in ["behavior_packs", "resource_packs"] {
        let dir = server_root.join(sub);
        if !dir.is_dir() {
            continue;
        }
        let Ok(entries) = std::fs::read_dir(&dir) else {
            continue;
        };
        for entry in entries.filter_map(|e| e.ok()) {
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }
            // The manifest may be nested one level in; scan shallowly.
            let found = WalkDir::new(&path)
                .max_depth(2)
                .into_iter()
                .filter_map(|e| e.ok())
                .any(|e| {
                    e.file_name().to_string_lossy().eq_ignore_ascii_case("manifest.json")
                        && manifest_reader::read(e.path())
                            .map(|m| m.uuid == uuid)
                            .unwrap_or(false)
                });
            if found {
                let _ = std::fs::remove_dir_all(&path);
                return true;
            }
        }
    }
    false
}

/// Remove a pack from a world: drop its json reference and delete its folder.
pub fn uninstall_pack(server: &Server, world_name: &str, uuid: &str) -> AppResult<bool> {
    let world_dir = Path::new(&server.worlds_path).join(world_name);
    let removed_b = remove_pack_ref(&world_dir.join(BEHAVIOR_JSON), uuid)?;
    let removed_r = remove_pack_ref(&world_dir.join(RESOURCE_JSON), uuid)?;
    let removed_folder = delete_pack_folder(server, uuid);
    Ok(removed_b || removed_r || removed_folder)
}

/// Map of pack uuid → display name for every pack folder under `dir`.
fn pack_names_in(dir: &Path) -> HashMap<String, String> {
    let mut map = HashMap::new();
    let Ok(entries) = std::fs::read_dir(dir) else {
        return map;
    };
    for entry in entries.filter_map(|e| e.ok()) {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        // manifest.json may sit one level in.
        for e in WalkDir::new(&path).max_depth(2).into_iter().filter_map(|e| e.ok()) {
            if e.file_name().to_string_lossy().eq_ignore_ascii_case("manifest.json") {
                if let Ok(m) = manifest_reader::read(e.path()) {
                    map.insert(m.uuid, m.name);
                }
                break;
            }
        }
    }
    map
}

fn world_packs_of(world_dir: &Path, json: &str, names: &HashMap<String, String>, kind: &str) -> Vec<WorldPack> {
    read_pack_refs(&world_dir.join(json))
        .into_iter()
        .map(|r| WorldPack {
            name: names.get(&r.pack_id).cloned().unwrap_or_else(|| r.pack_id.clone()),
            present: names.contains_key(&r.pack_id),
            uuid: r.pack_id,
            version: r.version,
            pack_type: kind.to_string(),
        })
        .collect()
}

/// List the packs a world actually references, in application order, resolving
/// names from the installed pack folders.
pub fn list_world_packs(server: &Server, world_name: &str) -> WorldPacks {
    let server_root = Path::new(&server.path);
    let world_dir = Path::new(&server.worlds_path).join(world_name);
    let bnames = pack_names_in(&server_root.join("behavior_packs"));
    let rnames = pack_names_in(&server_root.join("resource_packs"));
    WorldPacks {
        behavior: world_packs_of(&world_dir, BEHAVIOR_JSON, &bnames, "behavior"),
        resource: world_packs_of(&world_dir, RESOURCE_JSON, &rnames, "resource"),
    }
}

/// Rewrite a world's pack json in the given uuid order (versions/extra kept).
/// Unknown uuids are appended at the end preserving their relative order.
pub fn reorder_world_packs(
    server: &Server,
    world_name: &str,
    pack_type: &str,
    ordered_uuids: &[String],
) -> AppResult<()> {
    let json_name = match pack_type {
        "behavior" => BEHAVIOR_JSON,
        "resource" => RESOURCE_JSON,
        _ => return Err(AppError::Validation("Tipo de pack inválido.".into())),
    };
    let path = Path::new(&server.worlds_path).join(world_name).join(json_name);
    if !path.is_file() {
        return Ok(());
    }
    let mut refs = read_pack_refs(&path);
    refs.sort_by_key(|r| {
        ordered_uuids
            .iter()
            .position(|u| u == &r.pack_id)
            .unwrap_or(usize::MAX)
    });
    write_pack_refs(&path, &refs)?;
    Ok(())
}

/// Install supported packs from `prepared` into `server` / `world_name`.
/// If `selected` is `Some`, only packs whose UUID is in the set are installed.
pub fn install_packs(
    server: &Server,
    world_name: &str,
    prepared: &PreparedAddon,
    selected: Option<&HashSet<String>>,
) -> AppResult<Vec<InstalledPack>> {
    let server_root = Path::new(&server.path);
    let world_dir = Path::new(&server.worlds_path).join(world_name);
    // The world may not exist yet (brand-new server): pre-seed its folder so
    // BDS picks up the pack json files when it generates the world on first
    // start — important for addons that affect world generation.
    std::fs::create_dir_all(&world_dir).map_err(|e| {
        AppError::Io(format!("No se pudo preparar la carpeta del mundo '{world_name}': {e}"))
    })?;

    let mut results = Vec::new();

    for pack in &prepared.packs {
        let m = &pack.manifest;

        if let Some(sel) = selected {
            if !sel.contains(&m.uuid) {
                continue;
            }
        }

        let (packs_subdir, world_json_name) = match m.pack_type {
            PackType::Behavior => ("behavior_packs", "world_behavior_packs.json"),
            PackType::Resource => ("resource_packs", "world_resource_packs.json"),
            _ => {
                results.push(InstalledPack {
                    name: m.name.clone(),
                    uuid: m.uuid.clone(),
                    version: m.version.clone(),
                    pack_type: m.pack_type,
                    status: "unsupported".into(),
                    message: Some(
                        "Tipo de pack no soportado todavía (solo behavior y resource).".into(),
                    ),
                });
                continue;
            }
        };

        // Copy the pack folder into the server's packs directory.
        let uuid_short = m.uuid.chars().take(8).collect::<String>();
        let folder = format!("{}_{}", sanitize(&m.name), uuid_short);
        let target = server_root.join(packs_subdir).join(&folder);
        if target.exists() {
            let _ = std::fs::remove_dir_all(&target);
        }
        archive::copy_dir(&pack.dir, &target)?;

        // Reference it from the world, deduped by uuid.
        let world_json = world_dir.join(world_json_name);
        let added = add_pack_ref(&world_json, &m.uuid, &m.version)?;

        results.push(InstalledPack {
            name: m.name.clone(),
            uuid: m.uuid.clone(),
            version: m.version.clone(),
            pack_type: m.pack_type,
            status: if added { "installed" } else { "updated" }.into(),
            message: if added {
                None
            } else {
                Some("El pack ya estaba referenciado en el mundo; archivos actualizados.".into())
            },
        });
    }

    Ok(results)
}
