//! Streams the official server zip to disk, reporting progress via events.

use std::io::Write;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use futures_util::StreamExt;
use tauri::{AppHandle, Emitter, Manager};

use crate::error::{AppError, AppResult};
use crate::models::download::{
    DownloadProgress, DownloadStatus, DownloadedServerPackage, ServerDownloadOption,
};
use crate::state::AppState;

pub const EVENT_PROGRESS: &str = "download://progress";

const USER_AGENT: &str =
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64) BedrockServerManager/0.1";
/// Emit a progress event at most every ~512 KB to avoid flooding the frontend.
const EMIT_EVERY_BYTES: u64 = 512 * 1024;

fn emit(app: &AppHandle, progress: DownloadProgress) {
    let _ = app.emit(EVENT_PROGRESS, progress);
}

fn progress(
    download_id: &str,
    bytes: u64,
    total: Option<u64>,
    status: DownloadStatus,
) -> DownloadProgress {
    let percentage = total.filter(|t| *t > 0).map(|t| (bytes as f64 / t as f64) * 100.0);
    DownloadProgress {
        download_id: download_id.to_string(),
        bytes_downloaded: bytes,
        total_bytes: total,
        percentage,
        status,
    }
}

fn register_cancel(app: &AppHandle, download_id: &str) -> Arc<AtomicBool> {
    let flag = Arc::new(AtomicBool::new(false));
    if let Some(state) = app.try_state::<AppState>() {
        state
            .downloads
            .lock()
            .unwrap()
            .insert(download_id.to_string(), flag.clone());
    }
    flag
}

fn unregister_cancel(app: &AppHandle, download_id: &str) {
    if let Some(state) = app.try_state::<AppState>() {
        state.downloads.lock().unwrap().remove(download_id);
    }
}

fn downloads_dir(app: &AppHandle) -> AppResult<std::path::PathBuf> {
    let dir = app
        .state::<AppState>()
        .data_dir
        .join("downloads");
    std::fs::create_dir_all(&dir)?;
    Ok(dir)
}

/// Download `option.url` to a temp file, emitting progress on `download_id`.
/// Refuses to run unless the EULA was explicitly accepted.
pub async fn download(
    app: &AppHandle,
    option: &ServerDownloadOption,
    download_id: &str,
    accepted_eula: bool,
) -> AppResult<DownloadedServerPackage> {
    if !accepted_eula {
        return Err(AppError::Validation(
            "Debes aceptar el EULA de Minecraft y la Política de Privacidad antes de descargar."
                .into(),
        ));
    }

    let cancel = register_cancel(app, download_id);
    let result = download_inner(app, option, download_id, &cancel).await;
    unregister_cancel(app, download_id);

    if let Err(ref e) = result {
        // Distinguish a user cancel from a real failure for the UI.
        let status = if cancel.load(Ordering::SeqCst) {
            DownloadStatus::Cancelled
        } else {
            DownloadStatus::Failed
        };
        emit(app, progress(download_id, 0, None, status));
        let _ = e;
    }

    result
}

async fn download_inner(
    app: &AppHandle,
    option: &ServerDownloadOption,
    download_id: &str,
    cancel: &Arc<AtomicBool>,
) -> AppResult<DownloadedServerPackage> {
    emit(app, progress(download_id, 0, None, DownloadStatus::Starting));

    let client = reqwest::Client::builder()
        .user_agent(USER_AGENT)
        .build()
        .map_err(|e| AppError::Internal(format!("No se pudo crear el cliente HTTP: {e}")))?;

    let resp = client
        .get(&option.url)
        .send()
        .await
        .map_err(|e| AppError::Internal(format!("Fallo al iniciar la descarga: {e}")))?
        .error_for_status()
        .map_err(|e| AppError::Internal(format!("El servidor de descargas respondió error: {e}")))?;

    let total = resp.content_length();

    let dest = downloads_dir(app)?.join(format!("{download_id}.zip"));
    let mut file = std::fs::File::create(&dest)
        .map_err(|e| AppError::Io(format!("No se pudo crear el archivo temporal: {e}")))?;

    let mut downloaded: u64 = 0;
    let mut last_emit: u64 = 0;
    let mut stream = resp.bytes_stream();

    while let Some(chunk) = stream.next().await {
        if cancel.load(Ordering::SeqCst) {
            drop(file);
            let _ = std::fs::remove_file(&dest);
            return Err(AppError::Validation("Descarga cancelada.".into()));
        }

        let chunk = chunk.map_err(|e| AppError::Internal(format!("Error de red: {e}")))?;
        file.write_all(&chunk)
            .map_err(|e| AppError::Io(format!("No se pudo escribir el archivo: {e}")))?;
        downloaded += chunk.len() as u64;

        if downloaded - last_emit >= EMIT_EVERY_BYTES {
            last_emit = downloaded;
            emit(
                app,
                progress(download_id, downloaded, total, DownloadStatus::Downloading),
            );
        }
    }

    file.flush().ok();

    // If the server told us a size, make sure we got all of it.
    if let Some(total) = total {
        if downloaded < total {
            let _ = std::fs::remove_file(&dest);
            return Err(AppError::Internal(
                "La descarga quedó incompleta (tamaño no coincide).".into(),
            ));
        }
    }

    emit(
        app,
        progress(download_id, downloaded, total, DownloadStatus::Completed),
    );

    Ok(DownloadedServerPackage {
        download_id: download_id.to_string(),
        path: dest.to_string_lossy().into_owned(),
        size_bytes: downloaded,
        option: option.clone(),
    })
}

/// Signal an in-flight download to cancel.
pub fn cancel(app: &AppHandle, download_id: &str) {
    if let Some(state) = app.try_state::<AppState>() {
        if let Some(flag) = state.downloads.lock().unwrap().get(download_id) {
            flag.store(true, Ordering::SeqCst);
        }
    }
}
