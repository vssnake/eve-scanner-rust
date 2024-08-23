// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

/*fn main() {
    tauri_app_lib::run()
}*/

use log::info;
use crate::operations::extract_possible_root_address::ExtractPossibleRootAddress;
use crate::operations::obtain_pid_process::ObtainPidProcess;


mod process;
mod operations;

fn main() {


    let process_id = ObtainPidProcess::execute("exefile").unwrap();

    let possible_root_address = ExtractPossibleRootAddress::new().execute(process_id).unwrap();


    info!("It works!");

}
