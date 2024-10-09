use crate::commands::get_process_id::{get_process_ids, start_tracker, stop_tracker};
use crate::commands::process_watcher::process_watcher;

// Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
mod commands;
mod operations;
mod eve;
mod db;

#[macro_use]
extern crate rust_i18n;

i18n!("locales", fallback = "es");

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    //let test = obtain_process_command();
    tauri::Builder::default()
        .setup(|app| {
            process_watcher(app);
            Ok(())
        })
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![get_process_ids,start_tracker,stop_tracker])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
