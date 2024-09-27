use std::rc::Rc;

#[derive(Debug)]
pub struct DictEntry {
    pub key: String,
    pub value: Rc<Box<dyn std::any::Any>>, // Rust doesn't have a direct equivalent to C#'s `object`, so we use a trait object
}