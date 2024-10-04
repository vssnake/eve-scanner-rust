// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

/*fn main() {
    tauri_app_lib::run()
}*/
extern crate core;

use std::collections::HashMap;
use crate::operations::extract_possible_root_address::ExtractPossibleRootAddress;
use crate::operations::obtain_pid_process::ObtainPidProcess;
use crate::operations::ui_tree_node_extractor::UiTreeNodeExtractor;
use eve::interop::memory::windows_memory_reader::WindowsMemoryReader;
use std::rc::Rc;
use std::time::{Duration, Instant};
use std::io::{self, Write};
use std::thread;
use log::{debug, error, info, warn, LevelFilter};
use simple_logger::SimpleLogger;

#[cfg(target_os = "windows")]
extern crate winapi;
use winapi::um::consoleapi::AllocConsole;
use crate::eve::ui::models::general_window::GeneralWindow;
use crate::eve::ui_tree_node::models::ui_tree_node::UITreeNodeWithDisplayRegion;
use crate::eve::ui_tree_node::ui_constants::UiZonesEnum;
use crate::operations::eve_ui_tracker::EveUiTracker;

mod operations;
mod eve;
mod db;

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

    EveUiTracker::start_tracker();
}

