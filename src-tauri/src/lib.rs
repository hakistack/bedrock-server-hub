mod commands;
mod core;
mod db;
mod error;
mod models;
mod state;

use std::path::PathBuf;

use tauri::Manager;

use crate::state::AppState;

/// Resolve where the app stores its data.
///
/// On Windows we behave as a **portable app**: data lives in a `data/` folder
/// next to the executable, so the whole thing can be copied/moved and carries
/// its state. If that location is not writable (e.g. Program Files), or on
/// other platforms, we fall back to the OS app-data directory.
fn resolve_data_dir(app: &tauri::AppHandle) -> PathBuf {
    #[cfg(windows)]
    {
        if let Ok(exe) = std::env::current_exe() {
            if let Some(dir) = exe.parent() {
                let data = dir.join("data");
                if std::fs::create_dir_all(&data).is_ok() {
                    return data;
                }
            }
        }
    }
    let dir = app
        .path()
        .app_data_dir()
        .expect("no se pudo resolver el directorio de datos");
    let _ = std::fs::create_dir_all(&dir);
    dir
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .setup(|app| {
            let data_dir = resolve_data_dir(app.handle());
            std::fs::create_dir_all(&data_dir).ok();
            let db_path = data_dir.join("bedrock_manager.db");

            let conn = db::sqlite::open(&db_path).expect("no se pudo inicializar la base de datos");
            app.manage(AppState::new(conn, data_dir));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::server_commands::validate_server_folder,
            commands::server_commands::import_server,
            commands::server_commands::list_servers,
            commands::server_commands::get_server,
            commands::server_commands::rename_server,
            commands::server_commands::remove_server,
            commands::server_commands::start_server,
            commands::server_commands::stop_server,
            commands::server_commands::restart_server,
            commands::server_commands::get_server_status,
            commands::server_commands::get_server_settings,
            commands::server_commands::set_auto_restart,
            commands::server_commands::send_server_command,
            commands::settings_commands::read_properties,
            commands::settings_commands::update_properties,
            commands::world_commands::list_worlds,
            commands::world_commands::import_world,
            commands::world_commands::activate_world,
            commands::backup_commands::list_backups,
            commands::backup_commands::create_backup,
            commands::backup_commands::restore_backup,
            commands::backup_commands::delete_backup,
            commands::download_commands::get_official_server_download_options,
            commands::download_commands::resolve_manual_download_url,
            commands::download_commands::download_bedrock_server,
            commands::download_commands::cancel_download,
            commands::server_create_commands::install_downloaded_server,
            commands::server_create_commands::create_server_from_official_download,
            commands::addon_commands::preview_addon,
            commands::addon_commands::install_addon,
            commands::addon_commands::list_installed_addons,
            commands::addon_commands::uninstall_addon,
            commands::network_commands::get_network_status,
            commands::network_commands::add_firewall_rules,
            commands::network_commands::assign_free_port,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
