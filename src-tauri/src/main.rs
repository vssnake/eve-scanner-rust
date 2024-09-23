// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

/*fn main() {
    tauri_app_lib::run()
}*/
extern crate core;

use std::rc::Rc;
use log::info;
use crate::eve::ui::common::common::{ChildWithRegion, ChildWithoutRegion};
use crate::operations::extract_possible_root_address::ExtractPossibleRootAddress;
use crate::operations::obtain_pid_process::ObtainPidProcess;
use crate::operations::ui_tree_node_extractor::UiTreeNodeExtractor;
use crate::process::interop::memory::windows_memory_reader::WindowsMemoryReader;
use crate::process::interop::ui::ui_tree_node::{UITreeNodeWithDisplayRegion, UiTreeNode};

mod process;
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

    for ui_tree_node in ui_tree_nodes {
        /*for children in ui_tree_node.children.iter() {
            match children.as_any() {
                any if any.is::<ChildWithRegion>() => {
                    let child_with_region = any.downcast_ref::<ChildWithRegion>().unwrap();
                    let node = &child_with_region.node;
                    println!(
                        "Es un ChildWithRegion con región. Datos: con X: {}",
                        child_with_region.node.self_display_region.x
                    );
                }
                any if any.is::<ChildWithoutRegion>() => {
                    let child_without_region = any.downcast_ref::<ChildWithoutRegion>().unwrap();
                    let node = &child_without_region.node;
                    println!("Es un ChildWithoutRegion sin región.");
                }
                _ => {
                    println!("Tipo desconocido.");
                }
            }
           
        } */
    }


    info!("It works!");

}
