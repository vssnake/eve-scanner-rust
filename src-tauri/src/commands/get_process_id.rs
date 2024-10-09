use std::cell::OnceCell;
use std::rc::Rc;
use std::sync::{Arc, Mutex, OnceLock};
use tauri::Window;
use crate::operations::eve_ui_tracker::EveUiTracker;
use crate::operations::obtain_pid_process::ObtainPidProcess;

#[tauri::command]
pub fn get_process_ids() -> Vec<u32> {
    let processes = ObtainPidProcess::execute("exefile").unwrap();
    processes
}

static WINDOW_INSTANCE: OnceLock<Arc<Mutex<EveUiTracker>>> = OnceLock::new();

pub fn init_window_instance(window: Arc<Window>) {
    WINDOW_INSTANCE.set(Arc::new(Mutex::new(EveUiTracker::new(window)))).ok();
}
    
#[tauri::command]
pub fn start_tracker(pid: String) -> bool {
    if let Some(tracker) = WINDOW_INSTANCE.get() {
        let test = pid.parse::<u32>().ok().unwrap();
        let mut tracker_lock = tracker.lock().unwrap();
        tracker_lock.start_tracker(test, Arc::clone(tracker)); // Pasa el Arc
        true
    } else {
        false // Retorna false si no está inicializado
    }
}

#[tauri::command]
pub fn stop_tracker(pid: String) -> bool {
    let test = pid.parse::<u32>().ok().unwrap();
    WINDOW_INSTANCE.get().unwrap().lock().unwrap().stop_tracker(test);    
    true
}

