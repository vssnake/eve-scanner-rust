use std::collections::HashMap;
use std::{process, thread};
use std::rc::Rc;
use std::time::{Duration, Instant};
use log::info;
use crate::db;
use crate::eve::interop::gui::windows_utils::WindowsUtils;
use crate::eve::ui::models::general_window::GeneralWindow;
use crate::eve::ui_tree_node::models::ui_tree_node::UITreeNodeWithDisplayRegion;
use crate::eve::ui_tree_node::ui_constants::UiZonesEnum;
use crate::operations::extract_possible_root_address::ExtractPossibleRootAddress;
use crate::operations::obtain_pid_process::ObtainPidProcess;
use crate::operations::ui_tree_node_extractor::UiTreeNodeExtractor;

pub struct EveUiTracker {
     
}

impl EveUiTracker {

   
    pub fn start_tracker()  {
        let processes = ObtainPidProcess::execute("exefile").unwrap();

        let mut handles = vec![];
        
        for process in processes {
            
            let handle = thread::spawn(move || {
                info!("Starting tracker for process: {:?}", process);

                let windowsAttached = WindowsUtils::get_window_from_process_id(process);
                
                if windowsAttached.len() == 0 {
                    info!("No windows attached to process: {:?}", process);
                    return;
                }else{
                    info!("Windows attached to process: {:?}", windowsAttached);
                }
                EveUiTracker::extract_ui_from_process(process);
            });

            handles.push(handle);
            
        }

        for handle in handles {
            handle.join().unwrap();
        }
        
        
    }
    
    fn extract_ui_from_process(process: u32) {

        let time_per_second: i32 = 2;
        let interval = Duration::from_secs_f64(1.0 / time_per_second as f64);
        
        let ui_tree_address = extract_ui_tree_address(process);

        if ui_tree_address.is_err() {
            info!("Could not find ui tree address");
            return;
        }
        let mut total_duration = Duration::new(0, 0);
        let mut max_duration = Duration::new(0, 0);
        let mut min_duration = Duration::new(u64::MAX, 0);
        let mut count = 0;

        let mut last_print_time = Instant::now();



        let ui_tree_node_extractor = UiTreeNodeExtractor::new(process);

        loop {
            let start = Instant::now();

            let ui_tree = ui_tree_node_extractor.extract_ui_tree_from_address(ui_tree_address.unwrap(), 99);
            let duration = start.elapsed();
            if ui_tree.is_err() {
                return;
            }

            let ui_tree = ui_tree.unwrap();
            let zones_with_ui = ui_tree.1;

            let overview_windows = GeneralWindow::parse_general_window(zones_with_ui);

            //let types = ui_tree.0.ui_node.extract_types();


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