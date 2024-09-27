use sysinfo::System;

pub struct ObtainPidProcess {}

impl ObtainPidProcess {
    pub fn execute(process_name: &str) -> Result<u32, String> {
        let mut system = System::new_all();

        system.refresh_all();

        for (pid, process) in system.processes() {
            if process.name().to_str().unwrap().contains(process_name) {
                return Ok(pid.as_u32());
            }
        }

        Err(format!("Process {} not found", process_name))
    }
}