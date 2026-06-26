//! Resolves the current official Bedrock Dedicated Server download links.
//!
//! Primary source: the official Minecraft services JSON API, which returns the
//! up-to-date links Mojang generates for each platform/channel. If that fails,
//! we fall back to scraping the public download page for the same link pattern.
//! If both fail the caller surfaces a manual-paste fallback to the user.
//!
//! We never bundle or mirror the server binaries — only point at official URLs.

use regex::Regex;
use serde::Deserialize;

use crate::error::{AppError, AppResult};
use crate::models::download::{ServerChannel, ServerDownloadOption, ServerPlatform};

const API_URL: &str =
    "https://net-secondary.web.minecraft-services.net/api/v1.0/download/links";
const PAGE_URL: &str = "https://www.minecraft.net/en-us/download/server/bedrock";
const USER_AGENT: &str =
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64) BedrockServerManager/0.1";

#[derive(Deserialize)]
struct ApiResponse {
    result: ApiResult,
}
#[derive(Deserialize)]
struct ApiResult {
    links: Vec<ApiLink>,
}
#[derive(Deserialize)]
struct ApiLink {
    #[serde(rename = "downloadType")]
    download_type: String,
    #[serde(rename = "downloadUrl")]
    download_url: String,
}

fn client() -> AppResult<reqwest::Client> {
    reqwest::Client::builder()
        .user_agent(USER_AGENT)
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .map_err(|e| AppError::Internal(format!("No se pudo crear el cliente HTTP: {e}")))
}

fn extract_version(url: &str) -> Option<String> {
    let re = Regex::new(r"bedrock-server-([0-9][0-9.]*)\.zip").ok()?;
    re.captures(url)
        .and_then(|c| c.get(1))
        .map(|m| m.as_str().to_string())
}

fn map_download_type(dt: &str) -> Option<(ServerPlatform, ServerChannel)> {
    match dt {
        "serverBedrockWindows" => Some((ServerPlatform::Windows, ServerChannel::Stable)),
        "serverBedrockPreviewWindows" => Some((ServerPlatform::Windows, ServerChannel::Preview)),
        "serverBedrockLinux" => Some((ServerPlatform::Linux, ServerChannel::Stable)),
        "serverBedrockPreviewLinux" => Some((ServerPlatform::Linux, ServerChannel::Preview)),
        _ => None,
    }
}

fn make_option(platform: ServerPlatform, channel: ServerChannel, url: String) -> ServerDownloadOption {
    let version = extract_version(&url);
    let platform_label = match platform {
        ServerPlatform::Windows => "Windows",
        ServerPlatform::Linux => "Linux",
    };
    let channel_label = match channel {
        ServerChannel::Stable => "Stable",
        ServerChannel::Preview => "Preview",
    };
    let label = match &version {
        Some(v) => format!("{platform_label} · {channel_label} · v{v}"),
        None => format!("{platform_label} · {channel_label}"),
    };
    ServerDownloadOption {
        id: format!("{platform_label}-{channel_label}").to_lowercase(),
        label,
        platform,
        channel,
        url,
        version,
    }
}

async fn resolve_from_api(client: &reqwest::Client) -> AppResult<Vec<ServerDownloadOption>> {
    let resp = client
        .get(API_URL)
        .send()
        .await
        .map_err(|e| AppError::Internal(format!("Fallo al consultar la API oficial: {e}")))?
        .error_for_status()
        .map_err(|e| AppError::Internal(format!("La API oficial respondió con error: {e}")))?;

    let body: ApiResponse = resp
        .json()
        .await
        .map_err(|e| AppError::Internal(format!("Respuesta de la API ilegible: {e}")))?;

    let mut options = Vec::new();
    for link in body.result.links {
        if let Some((platform, channel)) = map_download_type(&link.download_type) {
            options.push(make_option(platform, channel, link.download_url));
        }
    }
    Ok(options)
}

async fn resolve_from_html(client: &reqwest::Client) -> AppResult<Vec<ServerDownloadOption>> {
    let html = client
        .get(PAGE_URL)
        .send()
        .await
        .map_err(|e| AppError::Internal(format!("Fallo al abrir la página oficial: {e}")))?
        .text()
        .await
        .map_err(|e| AppError::Internal(format!("Página oficial ilegible: {e}")))?;

    let re = Regex::new(
        r"https://www\.minecraft\.net/bedrockdedicatedserver/bin-(win|linux)(-preview)?/bedrock-server-[0-9][0-9.]*\.zip",
    )
    .map_err(|e| AppError::Internal(e.to_string()))?;

    let mut seen = std::collections::HashSet::new();
    let mut options = Vec::new();
    for m in re.find_iter(&html) {
        let url = m.as_str().to_string();
        if !seen.insert(url.clone()) {
            continue;
        }
        let platform = if url.contains("bin-win") {
            ServerPlatform::Windows
        } else {
            ServerPlatform::Linux
        };
        let channel = if url.contains("-preview") {
            ServerChannel::Preview
        } else {
            ServerChannel::Stable
        };
        options.push(make_option(platform, channel, url));
    }
    Ok(options)
}

/// Sort options for a stable, friendly presentation order.
fn sort_options(options: &mut [ServerDownloadOption]) {
    options.sort_by_key(|o| {
        let p = match o.platform {
            ServerPlatform::Windows => 0,
            ServerPlatform::Linux => 1,
        };
        let c = match o.channel {
            ServerChannel::Stable => 0,
            ServerChannel::Preview => 1,
        };
        (p, c)
    });
}

/// Resolve the available download options, trying the API then the HTML page.
pub async fn resolve_options() -> AppResult<Vec<ServerDownloadOption>> {
    let client = client()?;

    if let Ok(mut opts) = resolve_from_api(&client).await {
        if !opts.is_empty() {
            sort_options(&mut opts);
            return Ok(opts);
        }
    }

    let mut opts = resolve_from_html(&client).await?;
    if opts.is_empty() {
        return Err(AppError::Internal(
            "No se pudieron resolver los enlaces de descarga oficiales. \
             Usa la opción de pegar la URL manualmente."
                .into(),
        ));
    }
    sort_options(&mut opts);
    Ok(opts)
}

/// Build a download option from a user-pasted URL (manual fallback).
pub fn option_from_manual_url(url: &str) -> AppResult<ServerDownloadOption> {
    let url = url.trim();
    let valid = url.starts_with("https://www.minecraft.net/bedrockdedicatedserver/")
        || url.starts_with("https://minecraft.net/bedrockdedicatedserver/");
    if !valid || !url.ends_with(".zip") {
        return Err(AppError::Validation(
            "La URL no parece un enlace oficial de Bedrock Dedicated Server (.zip).".into(),
        ));
    }
    let platform = if url.contains("bin-win") {
        ServerPlatform::Windows
    } else {
        ServerPlatform::Linux
    };
    let channel = if url.contains("-preview") {
        ServerChannel::Preview
    } else {
        ServerChannel::Stable
    };
    Ok(make_option(platform, channel, url.to_string()))
}
