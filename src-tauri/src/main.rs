// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

/*fn main() {
    tauri_app_lib::run()
}*/
extern crate core;

use crate::operations::extract_possible_root_address::ExtractPossibleRootAddress;
use crate::operations::obtain_pid_process::ObtainPidProcess;
use crate::operations::ui_tree_node_extractor::UiTreeNodeExtractor;
use eve::interop::memory::windows_memory_reader::WindowsMemoryReader;
use eve::ui::models::ui_tree_node::{UITreeNodeWithDisplayRegion, UiTreeNode};
use log::info;
use std::rc::Rc;

mod operations;
mod eve;


fn main() {
    let process_id = ObtainPidProcess::execute("exefile").unwrap();

    let possible_root_address = ExtractPossibleRootAddress::new().execute(process_id).unwrap();
    
    let memoryReader = WindowsMemoryReader::new(process_id).unwrap();

    let uiExtractor = UiTreeNodeExtractor::new(process_id);

    let mut ui_tree_nodes: Vec<Rc<UITreeNodeWithDisplayRegion>> = Vec::new();
    for address in possible_root_address {
        info!("Possible root address: {:#X}", address);
        let ui_tree = uiExtractor.extract_ui_tree_from_address(address, 99);
        if ui_tree.is_err() {
            continue;
        }
        ui_tree_nodes.push(ui_tree.unwrap());
    }


    info!("It works!");
}
