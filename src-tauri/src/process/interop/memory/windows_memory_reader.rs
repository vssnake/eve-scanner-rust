#[cfg(windows)]

use crate::process::interop::memory::memory_region::MemoryRegion;
use winapi::ctypes::c_void;
use winapi::shared::minwindef::DWORD;
use winapi::um::handleapi::CloseHandle;
use winapi::um::memoryapi::{ReadProcessMemory, VirtualQueryEx};
use winapi::um::processthreadsapi::OpenProcess;
use winapi::um::winnt::{HANDLE, PROCESS_QUERY_INFORMATION, PROCESS_VM_READ,
                        MEMORY_BASIC_INFORMATION, PAGE_GUARD, PAGE_NOACCESS, MEM_COMMIT};

pub struct WindowsMemoryReader {
    process_handle: HANDLE,

}

impl WindowsMemoryReader {
    pub fn new(process_id: u32) -> Option<Self> {
        let process_handle = unsafe {
            OpenProcess(PROCESS_QUERY_INFORMATION | PROCESS_VM_READ, 0, process_id)
        };

        if process_handle.is_null() {
            None
        } else {
            Some(WindowsMemoryReader {
                process_handle,
            })
        }
    }
}

impl WindowsMemoryReader  {

    pub(crate) fn read_bytes(&self, start_address: u64, length: u64) -> Option<Vec<u8>> {

        let mut buffer = vec![0u8; length as usize];
        let mut number_of_bytes_read: usize = 0;

        let success = unsafe {
            ReadProcessMemory(
                self.process_handle,
                start_address as *const c_void,
                buffer.as_mut_ptr() as *mut c_void,
                buffer.len(),
                &mut number_of_bytes_read as *mut usize,
            )
        };

        if success == 0 {
            None
        } else {
            Some(buffer)
        }
    }


    pub(crate) fn read_commited_region(&self) -> Vec<MemoryRegion> {
        let mut committed_regions = Vec::new();
        let mut address = 0;

        loop {
            let mut memory_basic_information: MEMORY_BASIC_INFORMATION = unsafe { std::mem::zeroed() };
            let result = unsafe {
                VirtualQueryEx(
                    self.process_handle,
                    address as *const c_void,
                    &mut memory_basic_information,
                    size_of::<MEMORY_BASIC_INFORMATION>(),
                )
            };

            if result == 0 {
                break;
            }

            const PAGE_PROTECT_MASK: DWORD = PAGE_NOACCESS | PAGE_GUARD;


            if (memory_basic_information.Protect & PAGE_PROTECT_MASK) == 0
                && memory_basic_information.State == MEM_COMMIT {
                committed_regions.push( MemoryRegion {
                    base_address: memory_basic_information.BaseAddress as u64,
                    length: memory_basic_information.RegionSize as u64,
                });
            }

            address = memory_basic_information.BaseAddress as usize + memory_basic_information.RegionSize;
        }

        committed_regions
    }
}

impl Drop for WindowsMemoryReader {
    fn drop(&mut self) {
        unsafe {
            CloseHandle(self.process_handle);
        }
    }
}

unsafe impl Send for WindowsMemoryReader {}
unsafe impl Sync for WindowsMemoryReader {}