//! Background scheduler for automated backups with retention.
//!
//! Ticks once a minute, and for each server with an enabled schedule runs a
//! backup when due (interval or daily time), then prunes old scheduled backups
//! beyond the retention count. Requires the app to be running.

use std::path::Path;
use std::thread;
use std::time::Duration;

use tauri::{AppHandle, Manager};

use crate::core::{backup_service, server_properties};
use crate::db::repositories::{backups_repository, servers_repository, settings_repository};
use crate::error::AppResult;
use crate::models::backup::{reason, BackupSchedule};
use crate::models::server::Server;
use crate::state::AppState;

/// Start the scheduler loop on a background thread.
pub fn start(app: AppHandle) {
    thread::spawn(move || loop {
        thread::sleep(Duration::from_secs(60));
        tick(&app);
    });
}

fn active_world(server: &Server) -> String {
    server_properties::read(Path::new(&server.properties_path))
        .ok()
        .and_then(|entries| {
            entries
                .into_iter()
                .find(|p| p.key == "level-name")
                .map(|p| p.value)
        })
        .filter(|v| !v.trim().is_empty())
        .unwrap_or_else(|| "Bedrock level".into())
}

fn tick(app: &AppHandle) {
    let Some(state) = app.try_state::<AppState>() else {
        return;
    };

    let servers = {
        let conn = state.db.lock().unwrap();
        servers_repository::list(&conn).unwrap_or_default()
    };
    let now = chrono::Local::now();

    for server in servers {
        let sched: BackupSchedule = {
            let conn = state.db.lock().unwrap();
            settings_repository::get(&conn, &settings_repository::backup_schedule_key(&server.id))
                .ok()
                .flatten()
                .and_then(|s| serde_json::from_str(&s).ok())
                .unwrap_or_default()
        };
        if !sched.enabled {
            continue;
        }

        let due = {
            let mut last_map = state.schedule_last_run.lock().unwrap();
            let last = last_map.get(&server.id).copied();
            let due = match sched.mode.as_str() {
                "daily" => {
                    now.format("%H:%M").to_string() == sched.daily_time
                        && last.map(|l| l.date_naive() != now.date_naive()).unwrap_or(true)
                }
                _ => match last {
                    Some(l) => (now - l).num_minutes() >= sched.interval_minutes.max(1) as i64,
                    None => {
                        // First sighting: start the interval clock, don't fire yet.
                        last_map.insert(server.id.clone(), now);
                        false
                    }
                },
            };
            if due {
                last_map.insert(server.id.clone(), now);
            }
            due
        };

        if due {
            if let Err(e) = run_backup(app, &state, &server, &sched) {
                let _ = e; // best-effort; scheduler keeps running
            }
        }
    }
}

fn run_backup(
    app: &AppHandle,
    state: &AppState,
    server: &Server,
    sched: &BackupSchedule,
) -> AppResult<()> {
    let world = active_world(server);
    let record = backup_service::create_backup(app, server, Some(&world), reason::SCHEDULED)?;

    let conn = state.db.lock().unwrap();
    backups_repository::insert(&conn, &record)?;

    // Retention: keep only the newest N scheduled backups.
    if sched.retention > 0 {
        let scheduled: Vec<_> = backups_repository::list_for_server(&conn, &server.id)?
            .into_iter()
            .filter(|b| b.reason == reason::SCHEDULED)
            .collect();
        for old in scheduled.into_iter().skip(sched.retention as usize) {
            let _ = backup_service::delete_backup(&old);
            let _ = backups_repository::delete(&conn, &old.id);
        }
    }
    Ok(())
}
