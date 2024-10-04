use windows::Win32::Foundation::HWND;
use crate::eve::interop::gui::windows_utils::WindowsUtils;

pub struct GuiSimulation {
    process_id: u32,
    windows_found: Vec<HWND>
}

impl GuiSimulation {
    pub fn new(process_id: u32) -> GuiSimulation {
        
        let windows_found: WindowsUtils::get_window_from_process_id(process_id);
        GuiSimulation {
            process_id,
            windows_found
        }
    }
}