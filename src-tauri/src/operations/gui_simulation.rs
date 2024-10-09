use std::collections::HashMap;
use crate::eve::interop::gui::windows_utils::WindowsUtils;
use std::os::raw::c_void;
use std::ptr::NonNull;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};
use log::debug;
use windows::Win32::Foundation::HWND;

pub struct GuiSimulation {
    process_id: u32,
    windows_found: Arc<Mutex<Vec<SafeHWND>>>,
    key_states: Arc<Mutex<HashMap<u32, KeyState>>>, // u32 for virtual key code
    key_simulation_active: bool,
}

struct SafeHWND(NonNull<c_void>);

#[derive(Clone)]
struct KeyState {
    active: bool,
    delay: Duration,
    last_pressed: Option<Instant>,
}


unsafe impl Send for SafeHWND {}

impl GuiSimulation {
    pub fn new(process_id: u32) -> GuiSimulation {
        let windows_found = WindowsUtils::get_window_from_process_id(process_id);
        let key_states = Arc::new(Mutex::new(HashMap::new()));
        let safes_hwnd = Arc::new(Mutex::new(
            windows_found
                .iter()
                .map(|hwnd| SafeHWND(NonNull::new(hwnd.0).unwrap()))
                .collect::<Vec<SafeHWND>>()));
        
        GuiSimulation {
            process_id,
            windows_found: safes_hwnd,
            key_states,
            key_simulation_active: false,
        }
    }

    pub fn activate_key(&mut self, key_code: u32, delay: Duration) {
        self.simulate_key_presses();
        let mut key_states = self.key_states.lock().unwrap();
        key_states.insert(key_code, KeyState {
            active: true,
            delay,
            last_pressed: None,
        });
    }

    pub fn deactivate_key(&mut self, key_code: u32) {
        self.simulate_key_presses();
        let mut key_states = self.key_states.lock().unwrap();
        if let Some(state) = key_states.get_mut(&key_code) {
            state.active = false;
        }
    }

    fn simulate_key_presses(&mut self) {
        if (self.key_simulation_active) {
            return;
        }else { 
            self.key_simulation_active = true;
        }
        
        let key_states = self.key_states.clone();
        let windows_found = self.windows_found.clone();
        
        thread::spawn(move || loop {
            let keys_to_update: Vec<(u32,KeyState)>;
            {
                let key_states = key_states.lock().unwrap();
                keys_to_update = key_states
                    .iter()
                    .filter_map(|(&key_code, state)| {
                        if state.active {
                            
                            Some((key_code, state.clone())) // Clonamos el KeyState aquí
                        } else {
                            None
                        }
                    })
                    .collect::<Vec<(u32, KeyState)>>();
            }
            {
                let mut key_states = key_states.lock().unwrap();
                for (key_code, state) in keys_to_update {
                    if state.active {
                        if let Some(last) = state.last_pressed {
                            if last.elapsed() < state.delay {
                                continue; // Skip if the delay has not passed
                            }
                        }

                        // Send key press event (replace this with actual code to send the key)
                        for hwnd in windows_found.lock().unwrap().iter() {
                            let hwnd = hwnd.0.as_ptr();
                            debug!("Simulating key press for {:?} and key_code: {:?}",hwnd, key_code);
                            WindowsUtils::simulate_key_press(HWND(hwnd), key_code);
                        }
                        
                        if let Some(state) = key_states.get_mut(&key_code) {
                            state.last_pressed = Some(Instant::now());
                        }
                    }
                }
            }
            

            thread::sleep(Duration::from_millis(100)); // Small sleep to prevent busy-waiting
        });

                      
    }
}
