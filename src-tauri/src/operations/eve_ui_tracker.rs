use std::collections::HashMap;
use std::{fs, process, thread};
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use std::sync::atomic::AtomicBool;
use std::thread::JoinHandle;
use std::time::{Duration, Instant};
use log::{debug, error, info};
use serde_json::to_string;
use tauri::{Emitter, Window};
use crate::db;
use crate::eve::interop::gui::windows_utils::WindowsUtils;
use crate::eve::ui::models::general_window::GeneralWindow;
use crate::eve::ui_tree_node::models::ui_tree_node::UITreeNodeWithDisplayRegion;
use crate::eve::ui_tree_node::ui_constants::UiZonesEnum;
use crate::eve::ui_tree_node::utils::utils::UiUtils;
use crate::operations::extract_possible_root_address::ExtractPossibleRootAddress;
use crate::operations::gui_simulation::GuiSimulation;
use crate::operations::obtain_pid_process::ObtainPidProcess;
use crate::operations::ui_tree_node_extractor::UiTreeNodeExtractor;
use serde::Serialize;

#[derive(Debug)]
pub struct EveUiTracker {
    eve_ui_status: HashMap<u32,EveUiStatus>,
    running: Arc<AtomicBool>,
    window: Arc<Mutex<Window>>
}

#[derive(Debug, Serialize, Clone)]
enum EveUiTrackerStatus {
    Running,
    Stopped
}

#[derive(Debug, Serialize, Clone)]
pub struct EveUiStatus {
    pub process_id: u32,
    pub status: EveUiTrackerStatus,
    pub error: Option<String>,
    pub general_window: Option<String>,
    pub ms_processing: u32,
}

impl EveUiTracker {
    
    pub fn new(window: Arc<Mutex<Window>>, running: Arc<AtomicBool>) -> Self {
        EveUiTracker {
            eve_ui_status: HashMap::new(),
            running,
            window
        }
    }
    
    fn send_error(&mut self, process: u32, error: String){
        let eve_status = self.eve_ui_status.get_mut(&process).unwrap();
        
        eve_status.error = Some(error);
        
        self.send_event(process);
    }
   
    pub fn start_tracker(&mut self, process: u32) {
        
        
        self.running.store(true, std::sync::atomic::Ordering::Relaxed);

        self.eve_ui_status.insert(process, EveUiStatus {
            process_id: process,
            status: EveUiTrackerStatus::Running,
            error: None,
            general_window: None,
            ms_processing: 0
        });

        info!("Starting tracker for process: {:?}", process);
        
        
        self.extract_ui_from_process(process);


        let eve_status = self.eve_ui_status.get_mut(&process).unwrap();

        eve_status.status = EveUiTrackerStatus::Running;

        self.stop_tracker(process);

        info!("Tracker for process: {:?} finished", process);
        
    }
    

    fn stop_tracker(&mut self, process: u32){

        info!("the handle to stop");
        
        let eve_status = self.eve_ui_status.get_mut(&process).unwrap();

        eve_status.status = EveUiTrackerStatus::Stopped;

        self.send_event(process);
    }
    
    fn modify_eve_ui_status(&mut self,
                            process: u32, 
                            general_window: GeneralWindow){

        let eve_status = self.eve_ui_status.get_mut(&process).unwrap();
        
        eve_status.general_window = Some(to_string(&general_window).unwrap());
        
    }
    
    fn modify_eve_duration_processing(&mut self,
                                        process: u32,
                                        ms_processing: u32){
            let eve_status = self.eve_ui_status.get_mut(&process).unwrap();
            
            eve_status.ms_processing = ms_processing;
        
    }

    fn send_event(&mut self, process: u32){

        let eve_status = self.eve_ui_status.get(&process).unwrap();

        {
            let mut window = self.window.lock().unwrap();
            window.emit("eve_ui_status", eve_status).unwrap();
        }
    }
    
    fn extract_ui_from_process(&mut self, process: u32){

        let time_per_second: i32 = 2;
        let interval = Duration::from_secs_f64(1.0 / time_per_second as f64);
        
        let ui_tree_address = extract_ui_tree_address(process);

        if ui_tree_address.is_err() {
            self.send_error(process, "Could not find ui tree address".to_string());
            info!("Could not find ui tree address");
            return;
        }
        let mut total_duration = Duration::new(0, 0);
        let mut max_duration = Duration::new(0, 0);
        let mut min_duration = Duration::new(u64::MAX, 0);
        let mut count = 0;

        let mut last_print_time = Instant::now();
        
        let ui_tree_node_extractor = UiTreeNodeExtractor::new(process);
        
        //let mut gui_simulation = GuiSimulation::new(process);
        
        //gui_simulation.activate_key(0x56, Duration::from_secs(2));

        loop {
            
            if (self.running.load(std::sync::atomic::Ordering::Relaxed) == false) {
                return;
            }
            let start = Instant::now();

            let ui_tree = ui_tree_node_extractor.extract_ui_tree_from_address(ui_tree_address.unwrap(), 99);
            let duration = start.elapsed();
            if ui_tree.is_err() {
                return;
            }

            let ui_tree = ui_tree.unwrap();
            let zones_with_ui = ui_tree.1;
            
            let general_window = GeneralWindow::parse_general_window(zones_with_ui.clone());
            
            self.modify_eve_ui_status(process, general_window);

            //let types = ui_tree.0.ui_node.extract_types();

           // let json = serde_json::to_string(&zones_with_ui.get(&UiZonesEnum::ProbeScanner)).unwrap();

           // fs::write("archivo.txt", json).expect("TODO: panic message");

            count += 1;
            total_duration += duration;

            // Update maximum and minimum durations
            if duration > max_duration {
                max_duration = duration;
            }
            if duration < min_duration {
                min_duration = duration;
            }

            // Print statistics every 3 seconds
            if last_print_time.elapsed() >= Duration::from_secs(3) {
                let avg_duration = total_duration / count as u32;
                info!("ProcessId: {:?}, Max: {:?}, Min: {:?}, Avg: {:?}, Times: {:?}",process, max_duration, min_duration, avg_duration, count);

                // Reset statistics
                total_duration = Duration::new(0, 0);
                max_duration = Duration::new(0, 0);
                min_duration = Duration::new(u64::MAX, 0);
                count = 0;

                last_print_time = Instant::now();
            }
            
            self.send_event(process);
            
            if (duration < interval) {
                thread::sleep(interval - duration);
            }
        }
    }
}



fn extract_ui_tree_address(process_id: u32) -> Result<u64, &'static str> {
    let database = db::database::Database::new("eve.db").unwrap();



    let database_process_info = database.get_process_info(process_id).unwrap();

    if (database_process_info.is_none()) {
        let root_address_optional = get_root_address(process_id);
        if root_address_optional.is_none() {
            return Err("Could not find root address for process")
        }
        let root_address = root_address_optional.unwrap();
        database.add_process_info(process_id, root_address.to_string()).unwrap();
        Ok(root_address)
    }else{
        let database_ui_address = database_process_info.unwrap().1.parse::<u64>().unwrap();
        let ui_extractor = UiTreeNodeExtractor::new(process_id);
        let ui_tree = ui_extractor.extract_ui_tree_from_address(database_ui_address, 99);
        if ui_tree.is_err() {
            let root_address_optional = get_root_address(process_id);
            if root_address_optional.is_none() {
                return Err("Could not find root address for process")
            }
            database.delete_process_info(process_id).unwrap();

            let root_address = root_address_optional.unwrap();
            database.add_process_info(process_id, root_address.to_string()).unwrap();
            Ok(root_address)
        } else {
            Ok(database_ui_address)
        }
    }
}

fn get_root_address(process_id: u32) -> Option<u64> {
    let possible_root_address = ExtractPossibleRootAddress::new().execute(process_id).unwrap();
    let mut ui_tree_nodes: Vec<(Rc<UITreeNodeWithDisplayRegion>,HashMap<UiZonesEnum, Vec<Rc<UITreeNodeWithDisplayRegion>>>)> = Vec::new();
    let ui_extractor = UiTreeNodeExtractor::new(process_id);
    for address in possible_root_address {
        info!("Possible root address: {:#X}", address);
        let ui_tree = ui_extractor.extract_ui_tree_from_address(address, 99);
        if ui_tree.is_err() {
            continue;
        }

        ui_tree_nodes.push(ui_tree.unwrap());
    }
    let ui_address_with_count_nodes = ui_tree_nodes.iter().map(|(ui_tree_node, _)| {
        let size = ui_tree_node.ui_node.count_descendants();
        return (size, ui_tree_node);
    }).collect::<Vec<_>>();

    let largest_ui_address = ui_address_with_count_nodes.iter().max_by(|(size1, _), (size2, _)| size1.cmp(size2));

    Some(largest_ui_address?.1.ui_node.object_address)
}