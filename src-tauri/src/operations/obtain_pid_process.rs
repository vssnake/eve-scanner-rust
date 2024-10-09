use sysinfo::System;

pub struct ObtainPidProcess {}

impl ObtainPidProcess {
    pub fn execute(process_name: &str) -> Result<Vec<u32>, String> {
        let mut system = System::new_all();
        let mut processes: Vec<u32> = Vec::new();

        system.refresh_all();

        for (pid, process) in system.processes() {
            if process.name().to_str().unwrap().contains(process_name) {
                processes.push(pid.as_u32());
            }
        }
        
        Ok(processes)
    }
}