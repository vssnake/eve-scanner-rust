use std::rc::Rc;
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use tauri::{Emitter, Manager};
use crate::commands::get_process_id::init_window_instance;
use crate::operations::obtain_pid_process::ObtainPidProcess;

pub fn process_watcher(app: &tauri::App) {
    let windows =  app.get_window("main").unwrap();
    let windows = Arc::new(windows);
    init_window_instance(Arc::clone(&windows));
    thread::spawn(move || {
        loop {
            let processes = ObtainPidProcess::execute("exefile").unwrap();

            windows.emit("processes", Some(processes)).unwrap();
            
            thread::sleep(Duration::from_secs(3));
        }
    });
}