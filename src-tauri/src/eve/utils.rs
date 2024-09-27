use crate::eve::interop::memory::models::int_wrapper::IntWrapper;
use std::any::Any;

pub fn extract_int_from_int_or_string(object_value: &Box<dyn Any>) -> Option<i32> {
    if let Some(long_int) = object_value.downcast_ref::<IntWrapper>() {
        return long_int.get_i32();
    } else if let Some(&int_value) = object_value.downcast_ref::<i32>() {
        return Some(int_value);
    } else if let Some(string_value) = object_value.downcast_ref::<String>() {
        if let Ok(parsed_int) = string_value.parse::<i32>() {
            return Some(parsed_int);
        } else {
            // Log or handle error if parsing fails
            println!("Failed to parse integer from string '{}'", string_value);
        }
    }
    // Return None if the value cannot be decoded
    None
}