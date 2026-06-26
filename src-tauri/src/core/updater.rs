//! Portable self-updater.
//!
//! Checks GitHub Releases for a newer version, downloads the portable `.exe`
//! asset, and replaces the running executable in place (Windows): the running
//! exe is renamed aside (allowed on Windows), the new one takes its place, and
//! the app relaunches. Leftover `.old` files are cleaned up on startup.
//!
//! No cryptographic signature — integrity rests on HTTPS + the official repo.

use std::io::Write;
use std::path::{Path, PathBuf};

use futures_util::StreamExt;
use serde_json::Value;
use tauri::{AppHandle, Emitter};

use crate::error::{AppError, AppResult};
use crate::models::update::{UpdateInfo, UpdateProgress};

const GITHUB_REPO: &str = "hakistack/bedrock-server-hub";
/// The release asset that is the portable executable.
const ASSET_NAME: &str = "BedrockServerManager.exe";
const USER_AGENT: &str = "BedrockServerManager-Updater";
pub const EVENT_PROGRESS: &str = "update://progress";

/// Parse a version string ("v1.2.3" / "1.2.3") into comparable numbers.
fn parse_version(v: &str) -> Vec<u64> {
    v.trim()
        .trim_start_matches('v')
        .split('.')
        .map(|part| {
            part.chars()
                .take_while(|c| c.is_ascii_digit())
                .collect::<String>()
                .parse()
                .unwrap_or(0)
        })
        .collect()
}

fn is_newer(latest: &str, current: &str) -> bool {
    let l = parse_version(latest);
    let c = parse_version(current);
    for i in 0..l.len().max(c.len()) {
        let a = l.get(i).copied().unwrap_or(0);
        let b = c.get(i).copied().unwrap_or(0);
        if a != b {
            return a > b;
        }
    }
    false
}

fn client() -> AppResult<reqwest::Client> {
    reqwest::Client::builder()
        .user_agent(USER_AGENT)
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .map_err(|e| AppError::Internal(format!("No se pudo crear el cliente HTTP: {e}")))
}

/// Query the latest release: returns (tag, notes, Some((asset_url, size))).
async fn fetch_latest() -> AppResult<(String, String, Option<(String, u64)>)> {
    let url = format!("https://api.github.com/repos/{GITHUB_REPO}/releases/latest");
    let resp = client()?
        .get(url)
        .header("Accept", "application/vnd.github+json")
        .send()
        .await
        .map_err(|e| AppError::Internal(format!("No se pudo consultar GitHub: {e}")))?
        .error_for_status()
        .map_err(|e| AppError::Internal(format!("GitHub respondió error: {e}")))?;

    let json: Value = resp
        .json()
        .await
        .map_err(|e| AppError::Internal(format!("Respuesta de GitHub ilegible: {e}")))?;

    let tag = json["tag_name"].as_str().unwrap_or_default().to_string();
    let notes = json["body"].as_str().unwrap_or_default().to_string();
    let asset = json["assets"].as_array().and_then(|arr| {
        arr.iter()
            .find(|a| a["name"].as_str() == Some(ASSET_NAME))
            .map(|a| {
                (
                    a["browser_download_url"].as_str().unwrap_or_default().to_string(),
                    a["size"].as_u64().unwrap_or(0),
                )
            })
    });

    Ok((tag, notes, asset))
}

/// Check for an available update.
pub async fn check(app: &AppHandle) -> AppResult<UpdateInfo> {
    let current = app.package_info().version.to_string();
    let (tag, notes, asset) = fetch_latest().await?;
    let latest = tag.trim_start_matches('v').to_string();

    let (download_url, asset_size) = match asset {
        Some((u, s)) => (Some(u), Some(s)),
        None => (None, None),
    };

    let available = !latest.is_empty()
        && is_newer(&latest, &current)
        && download_url.is_some()
        && cfg!(windows);

    Ok(UpdateInfo {
        available,
        current_version: current,
        latest_version: latest,
        notes,
        download_url,
        asset_size,
        supported: cfg!(windows),
    })
}

fn emit_progress(app: &AppHandle, downloaded: u64, total: Option<u64>) {
    let percentage = total.filter(|t| *t > 0).map(|t| (downloaded as f64 / t as f64) * 100.0);
    let _ = app.emit(
        EVENT_PROGRESS,
        UpdateProgress {
            downloaded_bytes: downloaded,
            total_bytes: total,
            percentage,
        },
    );
}

async fn download_to(app: &AppHandle, url: &str, dest: &Path) -> AppResult<()> {
    let resp = client()?
        .get(url)
        .send()
        .await
        .map_err(|e| AppError::Internal(format!("Fallo al descargar la actualización: {e}")))?
        .error_for_status()
        .map_err(|e| AppError::Internal(format!("Descarga rechazada: {e}")))?;

    let total = resp.content_length();
    let mut file = std::fs::File::create(dest)
        .map_err(|e| AppError::Io(format!("No se pudo crear el archivo de actualización: {e}")))?;

    let mut downloaded: u64 = 0;
    let mut last_emit: u64 = 0;
    let mut stream = resp.bytes_stream();
    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|e| AppError::Internal(format!("Error de red: {e}")))?;
        file.write_all(&chunk)
            .map_err(|e| AppError::Io(format!("No se pudo escribir la actualización: {e}")))?;
        downloaded += chunk.len() as u64;
        if downloaded - last_emit >= 512 * 1024 {
            last_emit = downloaded;
            emit_progress(app, downloaded, total);
        }
    }
    file.flush().ok();
    emit_progress(app, downloaded, total);

    if let Some(total) = total {
        if downloaded < total {
            let _ = std::fs::remove_file(dest);
            return Err(AppError::Internal(
                "La actualización quedó incompleta.".into(),
            ));
        }
    }
    Ok(())
}

fn sibling(exe: &Path, suffix: &str) -> AppResult<PathBuf> {
    let dir = exe
        .parent()
        .ok_or_else(|| AppError::Internal("No se pudo resolver la carpeta del ejecutable.".into()))?;
    let stem = exe
        .file_stem()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_else(|| "app".into());
    Ok(dir.join(format!("{stem}.{suffix}.exe")))
}

/// Download the new exe and swap it in, then relaunch. Windows only.
pub async fn download_and_install(app: &AppHandle, url: &str) -> AppResult<()> {
    if !cfg!(windows) {
        return Err(AppError::Validation(
            "La auto-actualización solo está disponible en la versión portable de Windows.".into(),
        ));
    }

    let current_exe = std::env::current_exe()
        .map_err(|e| AppError::Io(format!("No se pudo resolver el ejecutable actual: {e}")))?;
    let new_path = sibling(&current_exe, "new")?;

    download_to(app, url, &new_path).await?;
    apply_update(app, &current_exe, &new_path)?;
    Ok(())
}

#[cfg(windows)]
fn apply_update(app: &AppHandle, current_exe: &Path, new_path: &Path) -> AppResult<()> {
    let old_path = sibling(current_exe, "old")?;
    let _ = std::fs::remove_file(&old_path);

    // Renaming a running executable is allowed on Windows.
    std::fs::rename(current_exe, &old_path)
        .map_err(|e| AppError::Io(format!("No se pudo apartar el ejecutable actual: {e}")))?;

    if let Err(e) = std::fs::rename(new_path, current_exe) {
        // Roll back so the app still works.
        let _ = std::fs::rename(&old_path, current_exe);
        return Err(AppError::Io(format!(
            "No se pudo colocar la nueva versión: {e}"
        )));
    }

    // Launch the freshly placed exe and quit this instance.
    std::process::Command::new(current_exe)
        .spawn()
        .map_err(|e| AppError::Process(format!("No se pudo relanzar la app: {e}")))?;
    app.exit(0);
    Ok(())
}

#[cfg(not(windows))]
fn apply_update(_app: &AppHandle, _current_exe: &Path, _new_path: &Path) -> AppResult<()> {
    Err(AppError::Validation(
        "La auto-actualización solo está disponible en Windows.".into(),
    ))
}

/// Remove leftover `.old`/`.new` files from a previous update (startup).
pub fn cleanup_leftovers() {
    if let Ok(exe) = std::env::current_exe() {
        for suffix in ["old", "new"] {
            if let Ok(p) = sibling(&exe, suffix) {
                let _ = std::fs::remove_file(p);
            }
        }
    }
}
