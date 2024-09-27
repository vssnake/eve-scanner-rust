#[derive(Debug)]
pub struct PyDictEntryRepresentation {
    pub address: u64,
    pub python_object_type_name: Option<String>,
}