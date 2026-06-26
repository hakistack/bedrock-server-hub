//! Management of the Bedrock Dedicated Server child process.
//!
//! Design notes (phase 1 baseline — see TODOs for the future supervisor):
//! - One running process per server id, tracked in `AppState.processes`.
//! - stdout/stderr are streamed line-by-line to the frontend via the
//!   `server://log` event; status transitions go out on `server://status`.
//! - Graceful stop writes `stop` to the server's stdin (BDS understands it),
//!   then a killer thread force-kills if it does not exit within a grace window.
//! - Crash vs. clean exit is inferred from the shared status flag at EOF.

use std::io::{BufRead, BufReader, Write};
use std::process::{Child, ChildStdin, Command, Stdio};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use serde::Serialize;
use sysinfo::{Pid, ProcessesToUpdate, System};
use tauri::{AppHandle, Emitter, Manager};

use crate::error::{AppError, AppResult};
use crate::models::player::{Player, PlayerEvent};
use crate::models::server::Server;
use crate::state::AppState;

pub const EVENT_LOG: &str = "server://log";
pub const EVENT_STATUS: &str = "server://status";
pub const EVENT_METRICS: &str = "server://metrics";
pub const EVENT_PLAYER: &str = "server://player";

/// How often to sample process CPU/RAM.
const METRICS_INTERVAL: Duration = Duration::from_secs(2);

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ServerMetrics {
    pub server_id: String,
    /// CPU usage percent (can exceed 100% across cores).
    pub cpu: f32,
    pub memory_bytes: u64,
}

/// Grace period before a graceful stop escalates to a force kill.
const STOP_GRACE: Duration = Duration::from_secs(8);
const POLL_INTERVAL: Duration = Duration::from_millis(200);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ServerStatus {
    Offline,
    Starting,
    Online,
    Stopping,
    Crashed,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LogLine {
    pub server_id: String,
    /// "stdout" | "stderr"
    pub stream: String,
    pub line: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StatusEvent {
    pub server_id: String,
    pub status: ServerStatus,
}

/// A live server process held in the registry.
pub struct RunningServer {
    child: Child,
    stdin: Option<ChildStdin>,
    /// Shared with the reader threads so they can observe/transition status.
    status: Arc<Mutex<ServerStatus>>,
}

fn emit_status(app: &AppHandle, server_id: &str, status: ServerStatus) {
    let _ = app.emit(
        EVENT_STATUS,
        StatusEvent {
            server_id: server_id.to_string(),
            status,
        },
    );
}

fn emit_log(app: &AppHandle, server_id: &str, stream: &str, line: String) {
    let _ = app.emit(
        EVENT_LOG,
        LogLine {
            server_id: server_id.to_string(),
            stream: stream.to_string(),
            line,
        },
    );
}

/// Update the online-players registry and emit a `server://player` event.
fn handle_player(app: &AppHandle, server_id: &str, event: &str, name: &str, xuid: &str) {
    let at = chrono::Local::now().to_rfc3339();
    if let Some(state) = app.try_state::<AppState>() {
        let mut map = state.players.lock().unwrap();
        let list = map.entry(server_id.to_string()).or_default();
        if event == "connected" {
            if !list.iter().any(|p| p.xuid == xuid) {
                list.push(Player {
                    name: name.to_string(),
                    xuid: xuid.to_string(),
                    connected_at: at.clone(),
                });
            }
        } else {
            list.retain(|p| p.xuid != xuid);
        }
    }
    let _ = app.emit(
        EVENT_PLAYER,
        PlayerEvent {
            server_id: server_id.to_string(),
            event: event.to_string(),
            name: name.to_string(),
            xuid: xuid.to_string(),
            at,
        },
    );
}

/// Sample the server process CPU/RAM until it leaves the registry, emitting
/// `server://metrics`. CPU usage needs two samples spaced apart.
fn spawn_metrics(app: AppHandle, server_id: String, pid: u32) {
    thread::spawn(move || {
        let pid = Pid::from_u32(pid);
        let mut sys = System::new();
        loop {
            // Stop once the server is no longer tracked.
            match app.try_state::<AppState>() {
                Some(state) if state.processes.lock().unwrap().contains_key(&server_id) => {}
                _ => break,
            }
            sys.refresh_processes(ProcessesToUpdate::Some(&[pid]), true);
            thread::sleep(METRICS_INTERVAL);
            sys.refresh_processes(ProcessesToUpdate::Some(&[pid]), true);
            match sys.process(pid) {
                Some(p) => {
                    let _ = app.emit(
                        EVENT_METRICS,
                        ServerMetrics {
                            server_id: server_id.clone(),
                            cpu: p.cpu_usage(),
                            memory_bytes: p.memory(),
                        },
                    );
                }
                None => break,
            }
        }
    });
}

/// Spawn a thread that streams a child's stdout/stderr to the frontend.
///
/// The stdout reader (`primary == true`) additionally owns the lifecycle:
/// it detects readiness ("Server started") and, on EOF, publishes the final
/// status (Offline if we were stopping, Crashed otherwise) and clears the
/// registry entry.
fn spawn_reader<R: std::io::Read + Send + 'static>(
    app: AppHandle,
    server_id: String,
    reader: R,
    stream: &'static str,
    status: Arc<Mutex<ServerStatus>>,
    primary: bool,
) {
    // Compiled once per reader thread for player connect/disconnect parsing.
    let connect_re = regex::Regex::new(r"Player connected:\s*(.+?),\s*xuid:\s*(\d+)").ok();
    let disconnect_re = regex::Regex::new(r"Player disconnected:\s*(.+?),\s*xuid:\s*(\d+)").ok();

    thread::spawn(move || {
        let buf = BufReader::new(reader);
        for line in buf.lines() {
            let Ok(line) = line else { break };

            if primary && line.contains("Server started") {
                let mut s = status.lock().unwrap();
                if *s == ServerStatus::Starting {
                    *s = ServerStatus::Online;
                    drop(s);
                    emit_status(&app, &server_id, ServerStatus::Online);
                    // Stable start → give a fresh crash budget.
                    reset_crash_counter(&app, &server_id);
                }
            }

            if primary {
                if let Some(re) = &connect_re {
                    if let Some(c) = re.captures(&line) {
                        handle_player(&app, &server_id, "connected", &c[1], &c[2]);
                    }
                }
                if let Some(re) = &disconnect_re {
                    if let Some(c) = re.captures(&line) {
                        handle_player(&app, &server_id, "disconnected", &c[1], &c[2]);
                    }
                }
            }

            emit_log(&app, &server_id, stream, line);
        }

        if primary {
            let final_status = {
                let current = *status.lock().unwrap();
                if current == ServerStatus::Stopping {
                    ServerStatus::Offline
                } else {
                    ServerStatus::Crashed
                }
            };
            *status.lock().unwrap() = final_status;

            // Drop the registry entry if it is still the process we started.
            if let Some(state) = app.try_state::<AppState>() {
                state.processes.lock().unwrap().remove(&server_id);
                state.players.lock().unwrap().remove(&server_id);
            }
            emit_status(&app, &server_id, final_status);

            if final_status == ServerStatus::Crashed {
                maybe_auto_restart(&app, &server_id);
            }
        }
    });
}

/// Maximum crash-triggered restarts within `CRASH_WINDOW` before giving up.
const MAX_CRASH_RESTARTS: u32 = 3;
const CRASH_WINDOW: Duration = Duration::from_secs(60);
const CRASH_RESTART_DELAY: Duration = Duration::from_secs(3);

/// Load a server record from the database (best-effort, for supervisor use).
fn load_server(app: &AppHandle, server_id: &str) -> Option<Server> {
    let state = app.try_state::<AppState>()?;
    let conn = state.db.lock().unwrap();
    crate::db::repositories::servers_repository::get(&conn, server_id)
        .ok()
        .flatten()
}

/// Clear the crash counter for a server (called on intentional start/stop).
fn reset_crash_counter(app: &AppHandle, server_id: &str) {
    if let Some(state) = app.try_state::<AppState>() {
        state.crash_restarts.lock().unwrap().remove(server_id);
    }
}

/// After a crash, restart the server if auto-restart is enabled and we have
/// not exceeded the crash budget within the rolling window.
fn maybe_auto_restart(app: &AppHandle, server_id: &str) {
    let Some(state) = app.try_state::<AppState>() else {
        return;
    };

    // Is auto-restart enabled for this server?
    let enabled = {
        let conn = state.db.lock().unwrap();
        crate::db::repositories::settings_repository::get_bool(
            &conn,
            &crate::db::repositories::settings_repository::auto_restart_key(server_id),
        )
        .unwrap_or(false)
    };
    if !enabled {
        return;
    }

    // Rate-limit restarts to avoid a crash loop.
    let attempt = {
        let mut map = state.crash_restarts.lock().unwrap();
        let entry = map
            .entry(server_id.to_string())
            .or_insert((0, Instant::now()));
        if entry.1.elapsed() > CRASH_WINDOW {
            *entry = (0, Instant::now());
        }
        if entry.0 >= MAX_CRASH_RESTARTS {
            None
        } else {
            entry.0 += 1;
            Some(entry.0)
        }
    };

    let Some(attempt) = attempt else {
        emit_log(
            app,
            server_id,
            "stderr",
            format!(
                "[manager] El servidor crasheó repetidamente ({MAX_CRASH_RESTARTS} veces en {}s). \
                 Auto-reinicio detenido.",
                CRASH_WINDOW.as_secs()
            ),
        );
        return;
    };

    if let Some(server) = load_server(app, server_id) {
        emit_log(
            app,
            server_id,
            "stdout",
            format!("[manager] Crash detectado. Reiniciando ({attempt}/{MAX_CRASH_RESTARTS})…"),
        );
        let app = app.clone();
        thread::spawn(move || {
            thread::sleep(CRASH_RESTART_DELAY);
            if let Err(e) = start(&app, &server) {
                emit_log(&app, &server.id, "stderr", format!("[manager] {e}"));
            }
        });
    }
}

/// Validate the executable path and resolve a `Command` for the server.
fn build_command(server: &Server) -> AppResult<Command> {
    let exe = std::path::Path::new(&server.executable_path);
    if !exe.exists() {
        return Err(AppError::Validation(format!(
            "No se encontró el ejecutable del servidor en {}",
            server.executable_path
        )));
    }
    let mut cmd = Command::new(exe);
    cmd.current_dir(&server.path)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    // On Windows, prevent a console window from flashing when launching BDS.
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        cmd.creation_flags(CREATE_NO_WINDOW);
    }

    Ok(cmd)
}

/// Start the server. Errors if it is already running.
pub fn start(app: &AppHandle, server: &Server) -> AppResult<()> {
    let state = app.state::<AppState>();

    if state.processes.lock().unwrap().contains_key(&server.id) {
        return Err(AppError::Validation(
            "El servidor ya está en ejecución.".into(),
        ));
    }

    let mut cmd = build_command(server)?;
    let mut child = cmd
        .spawn()
        .map_err(|e| AppError::Process(format!("No se pudo iniciar el servidor: {e}")))?;

    let pid = child.id();
    let stdin = child.stdin.take();
    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| AppError::Process("No se pudo capturar stdout del servidor.".into()))?;
    let stderr = child
        .stderr
        .take()
        .ok_or_else(|| AppError::Process("No se pudo capturar stderr del servidor.".into()))?;

    let status = Arc::new(Mutex::new(ServerStatus::Starting));
    emit_status(app, &server.id, ServerStatus::Starting);

    spawn_reader(
        app.clone(),
        server.id.clone(),
        stdout,
        "stdout",
        status.clone(),
        true,
    );
    spawn_reader(
        app.clone(),
        server.id.clone(),
        stderr,
        "stderr",
        status.clone(),
        false,
    );

    state.processes.lock().unwrap().insert(
        server.id.clone(),
        RunningServer {
            child,
            stdin,
            status,
        },
    );

    spawn_metrics(app.clone(), server.id.clone(), pid);

    Ok(())
}

/// Gracefully stop a running server, escalating to a kill after the grace
/// window. Returns Ok even if the server was not running (idempotent).
pub fn stop(app: &AppHandle, server_id: &str) -> AppResult<()> {
    let state = app.state::<AppState>();

    // A user-initiated stop cancels any pending crash auto-restart budget.
    reset_crash_counter(app, server_id);

    let running = state.processes.lock().unwrap().remove(server_id);
    let Some(mut running) = running else {
        return Ok(());
    };

    *running.status.lock().unwrap() = ServerStatus::Stopping;
    emit_status(app, server_id, ServerStatus::Stopping);

    // Take ownership into a killer thread so we never block the command thread.
    thread::spawn(move || {
        stop_blocking(&mut running);
    });

    Ok(())
}

/// Send `stop` to BDS and wait for it to exit, force-killing on timeout.
/// Blocks the calling thread — only call this off the main thread.
fn stop_blocking(running: &mut RunningServer) {
    if let Some(mut stdin) = running.stdin.take() {
        let _ = writeln!(stdin, "stop");
        let _ = stdin.flush();
        // Dropping stdin closes the pipe, signalling EOF to the server.
    }

    let deadline_polls = (STOP_GRACE.as_millis() / POLL_INTERVAL.as_millis()) as u32;
    for _ in 0..deadline_polls {
        match running.child.try_wait() {
            Ok(Some(_)) => return,
            Ok(None) => thread::sleep(POLL_INTERVAL),
            Err(_) => break,
        }
    }

    let _ = running.child.kill();
    let _ = running.child.wait();
}

/// Restart: stop (if running) then start again once the port is released.
pub fn restart(app: &AppHandle, server: &Server) -> AppResult<()> {
    let state = app.state::<AppState>();
    let running = state.processes.lock().unwrap().remove(&server.id);

    let app = app.clone();
    let server = server.clone();
    thread::spawn(move || {
        if let Some(mut running) = running {
            *running.status.lock().unwrap() = ServerStatus::Stopping;
            emit_status(&app, &server.id, ServerStatus::Stopping);
            stop_blocking(&mut running);
            // Give the OS a moment to release the bound port.
            thread::sleep(Duration::from_millis(1500));
        }
        if let Err(e) = start(&app, &server) {
            emit_log(&app, &server.id, "stderr", format!("[manager] {e}"));
            emit_status(&app, &server.id, ServerStatus::Crashed);
        }
    });

    Ok(())
}

/// Current status of a server (Offline if not in the registry).
pub fn status(state: &AppState, server_id: &str) -> ServerStatus {
    state
        .processes
        .lock()
        .unwrap()
        .get(server_id)
        .map(|r| *r.status.lock().unwrap())
        .unwrap_or(ServerStatus::Offline)
}

/// Send a raw command line to a running server's stdin.
/// Useful for the console; exposed now so the UI can grow into it.
pub fn send_command(state: &AppState, server_id: &str, line: &str) -> AppResult<()> {
    let mut map = state.processes.lock().unwrap();
    let running = map
        .get_mut(server_id)
        .ok_or_else(|| AppError::Validation("El servidor no está en ejecución.".into()))?;
    let stdin = running
        .stdin
        .as_mut()
        .ok_or_else(|| AppError::Process("stdin del servidor no disponible.".into()))?;
    writeln!(stdin, "{line}").map_err(|e| AppError::Process(e.to_string()))?;
    stdin.flush().map_err(|e| AppError::Process(e.to_string()))?;
    Ok(())
}
