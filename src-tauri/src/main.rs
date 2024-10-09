// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

#[cfg(target_os = "windows")]
extern crate winapi;

use log::{info, LevelFilter};
use simple_logger::SimpleLogger;
use winapi::um::consoleapi::AllocConsole;
use crate::commands::process_watcher::process_watcher;
use crate::operations::eve_ui_tracker::EveUiTracker;

mod operations;
mod eve;
mod db;
mod commands;

#[macro_use]
extern crate rust_i18n;

i18n!("locales", fallback = "es");
fn main() {

    #[cfg(target_os = "windows")]
    unsafe {
        AllocConsole();
    }

    rust_i18n::set_locale("es");

    SimpleLogger::new().with_level(LevelFilter::Debug).init().unwrap();

    info!("Starting Eve Tracker");
    
    
    
    tauri_app_lib::run();
}


