use std::rc::Rc;
use std::sync::Arc;

// Define a struct similar to PyDictEntry
#[derive(Debug)]
pub struct PyDictEntry {
    pub hash: u64,
    pub key: u64,
    pub value: u64,
}

// Define a struct similar to DictEntryValueGenericRepresentation
#[derive(Debug)]
pub struct DictEntryValueGenericRepresentation {
    pub address: u64,
    pub python_object_type_name: Option<String>,
}

// Define a struct similar to DictEntry
#[derive(Debug)]
pub struct DictEntry {
    pub key: String,
    pub value: Arc<Box<dyn std::any::Any>>, // Rust doesn't have a direct equivalent to C#'s `object`, so we use a trait object
}

// Define a struct similar to LongInt
#[derive(Debug)]
pub struct LongInt {
    pub int: i64,
    pub int_low32: i32,
}