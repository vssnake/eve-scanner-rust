use windows::Win32::Foundation::{BOOL, HWND, LPARAM, WPARAM};
use windows::Win32::UI::WindowsAndMessaging::{EnumWindows, GetWindowThreadProcessId, PostMessageA, WM_KEYDOWN, WM_KEYUP};

struct EnumWindowsData {
    process_id: u32,
    windows_found: *mut Vec<HWND>
}
pub struct WindowsUtils {}

impl WindowsUtils {
    
    pub fn get_window_from_process_id(process_id: u32) -> Vec<HWND> {
        
        let mut windows_found: Vec<HWND> = Vec::new();
        
        let mut data = EnumWindowsData {
            process_id,
            windows_found: &mut windows_found as *mut _,
        };
        unsafe {
            let _ = EnumWindows(Some(enum_windows_callback), LPARAM(&mut data as *mut EnumWindowsData as isize));
        }
        
        windows_found
        
    }
    
    pub fn simulate_key_press(hwnd: HWND, key: u32) {
        unsafe {
            let _ = PostMessageA(hwnd, WM_KEYDOWN, WPARAM(key as usize), LPARAM(0isize));
            let _ = PostMessageA(hwnd, WM_KEYUP, WPARAM(key as usize), LPARAM(0isize));
        }
    }
}


unsafe extern "system" fn enum_windows_callback(hwnd: HWND, l_param: LPARAM) -> BOOL {
    let data = &mut *(l_param.0 as *mut EnumWindowsData);
    let mut window_process_id = 0;
    
    GetWindowThreadProcessId(hwnd, Some(&mut window_process_id));
    
    if window_process_id == data.process_id {
        (*data.windows_found).push(hwnd);
        
        
        return BOOL(0);
    }
    
    BOOL(1)
}