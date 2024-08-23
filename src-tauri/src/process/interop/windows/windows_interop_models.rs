#[cfg(windows)] extern crate winapi;

use bitflags::bitflags;
use winapi::um::winnt::{
    PAGE_EXECUTE, PAGE_EXECUTE_READ, PAGE_EXECUTE_READWRITE, PAGE_EXECUTE_WRITECOPY, PAGE_NOACCESS,
    PAGE_READONLY, PAGE_READWRITE, PAGE_WRITECOPY, PAGE_GUARD, PAGE_NOCACHE, PAGE_WRITECOMBINE,
    PROCESS_TERMINATE, PROCESS_CREATE_THREAD, PROCESS_VM_OPERATION, PROCESS_VM_READ, PROCESS_VM_WRITE,
    PROCESS_DUP_HANDLE, PROCESS_CREATE_PROCESS, PROCESS_SET_QUOTA, PROCESS_SET_INFORMATION,
    PROCESS_QUERY_INFORMATION, PROCESS_QUERY_LIMITED_INFORMATION, SYNCHRONIZE,
    PROCESS_ALL_ACCESS,
    MEM_COMMIT, MEM_FREE, MEM_RESERVE, MEM_IMAGE, MEM_MAPPED, MEM_PRIVATE,
};
bitflags! {
    pub struct ProcessAccessFlags: u32 {
        const ALL = PROCESS_ALL_ACCESS;
        const TERMINATE = PROCESS_TERMINATE;
        const CREATE_THREAD = PROCESS_CREATE_THREAD;
        const VM_OPERATION = PROCESS_VM_OPERATION;
        const VM_READ = PROCESS_VM_READ;
        const VM_WRITE = PROCESS_VM_WRITE;
        const DUPLICATE_HANDLE = PROCESS_DUP_HANDLE;
        const CREATE_PROCESS = PROCESS_CREATE_PROCESS;
        const SET_QUOTA = PROCESS_SET_QUOTA;
        const SET_INFORMATION = PROCESS_SET_INFORMATION;
        const QUERY_INFORMATION = PROCESS_QUERY_INFORMATION;
        const QUERY_LIMITED_INFORMATION = PROCESS_QUERY_LIMITED_INFORMATION;
        const SYNCHRONIZE = SYNCHRONIZE;
    }
}

#[repr(C)]
pub struct MemoryBasicInformation64 {
    pub base_address: u64,
    pub allocation_base: u64,
    pub allocation_protect: i32,
    pub alignment1: i32,
    pub region_size: u64,
    pub state: i32,
    pub protect: i32,
    pub type_: i32,
    pub alignment2: i32,
}

bitflags! {
    pub struct MemoryInformationProtection: u32 {
        const PAGE_EXECUTE = PAGE_EXECUTE;
        const PAGE_EXECUTE_READ = PAGE_EXECUTE_READ;
        const PAGE_EXECUTE_READWRITE = PAGE_EXECUTE_READWRITE;
        const PAGE_EXECUTE_WRITECOPY = PAGE_EXECUTE_WRITECOPY;
        const PAGE_NOACCESS = PAGE_NOACCESS;
        const PAGE_READONLY = PAGE_READONLY;
        const PAGE_READWRITE = PAGE_READWRITE;
        const PAGE_WRITECOPY = PAGE_WRITECOPY;
        const PAGE_GUARD = PAGE_GUARD;
        const PAGE_NOCACHE = PAGE_NOCACHE;
        const PAGE_WRITECOMBINE = PAGE_WRITECOMBINE;
    }
}

#[repr(u32)]
pub enum MemoryInformationState {
    MemCommit = MEM_COMMIT,
    MemFree = MEM_FREE,
    MemReserve = MEM_RESERVE,
}

#[repr(u32)]
pub enum MemoryInformationType {
    MemImage = MEM_IMAGE,
    MemMapped = MEM_MAPPED,
    MemPrivate = MEM_PRIVATE,
}
