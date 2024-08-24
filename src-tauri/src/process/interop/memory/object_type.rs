#[derive(Debug, PartialEq)]
pub enum ObjectType {
    NoneType,
    Bool(bool),
    Int(i64),
    Str(String),
    DictEntryValueGenericRepresentation(Vec<ObjectType>),
}


impl ObjectType {
    pub(crate) fn from_python_type_name(type_name: &str, value: Option<&str>) -> ObjectType {
        match type_name {
            "NoneType" => ObjectType::NoneType,
            "bool" => {
                let bool_value = value.unwrap_or("false") == "true";
                ObjectType::Bool(bool_value)
            },
            "int" => {
                let int_value = value.unwrap_or("0").parse().unwrap_or(0);
                ObjectType::Int(int_value)
            },
            "str" => ObjectType::Str(value.unwrap_or("").to_string()),
            _ => unimplemented!(),

        }
    }
}