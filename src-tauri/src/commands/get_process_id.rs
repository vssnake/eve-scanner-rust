use std::cell::OnceCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::{Arc, Mutex, OnceLock, RwLock};
use std::sync::atomic::AtomicBool;
use std::thread;
use log::info;
use tauri::Window;
use crate::operations::eve_ui_tracker::EveUiTracker;
use crate::operations::obtain_pid_process::ObtainPidProcess;

#[tauri::command]
pub fn get_process_ids() -> Vec<u32> {
    let processes = ObtainPidProcess::execute("exefile").unwrap();
    processes
}

static WINDOW_INSTANCES: OnceLock<Arc<Mutex<HashMap<u32, Arc<AtomicBool>>>>> = OnceLock::new();
static WINDOW_TEST: OnceLock<Arc<Mutex<Window>>> = OnceLock::new();
pub fn init_window_instance(window: Arc<Mutex<Window>>) {
    WINDOW_INSTANCES.set(Arc::new(Mutex::new(HashMap::new()))).ok();
    WINDOW_TEST.set(window).ok();
}

#[tauri::command]
    pub fn start_tracker(pid: String) -> bool {
        let process_id = pid.parse::<u32>().ok().unwrap();

        if let Some(instances) = WINDOW_INSTANCES.get() {
            {
                let instances_lock = instances.lock().unwrap();

                if instances_lock.contains_key(&process_id) {
                    return false;
                }
            }
            let test = WINDOW_TEST.get().unwrap();

            let atomic_bool = Arc::new(AtomicBool::new(true));
            let mut eve_ui_tracker = EveUiTracker::new(Arc::clone(test),atomic_bool.clone());

            {
                let mut instances_lock = instances.lock().unwrap();
                instances_lock.insert(process_id, atomic_bool);
            }

            thread::spawn(move || {
                eve_ui_tracker.start_tracker(process_id);
                info!("Stopped tracking process {}", process_id);
            });

            true
        } else {
            false
        }
    }

#[tauri::command]
pub fn stop_tracker(pid: String) -> bool {
    let process_id = pid.parse::<u32>().ok().unwrap();
    info!("Stopping tracker for process: {:?}", process_id);

    if let Some(instances) = WINDOW_INSTANCES.get() {
        let mut instances_lock = instances.lock().unwrap();
        if let Some(atomic_bool) = instances_lock.get(&process_id) {
            atomic_bool.store(false, std::sync::atomic::Ordering::Relaxed);
            instances_lock.remove(&process_id);
            true
        } else {
            false
        }
    } else {
        false
    }
}
