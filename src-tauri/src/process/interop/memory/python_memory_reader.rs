use crate::process::interop::memory::memory_reading_cache::MemoryReadingCache;
use crate::process::interop::memory::memory_utils;
use crate::process::interop::memory::python_models::{
    DictEntryValueGenericRepresentation, PyDictEntry,
};
use crate::process::interop::memory::windows_memory_reader::WindowsMemoryReader;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::Arc;

pub struct PythonMemoryReader {
    memory_reader: Rc<WindowsMemoryReader>,
    cache: MemoryReadingCache,
}

impl PythonMemoryReader {
    pub fn new(
        windows_memory_reader: &Rc<WindowsMemoryReader>,
        memory_reading_cache: MemoryReadingCache,
    ) -> Self {
        Self {
            memory_reader: Rc::clone(windows_memory_reader),
            cache: memory_reading_cache,
        }
    }


    pub fn get_python_type_name_from_object_address(
        &self,
        object_address: u64,
    ) -> Option<String> {

        let python_object_memory_size: usize = 16;

        let object_type_offset: usize = 8;

        self.cache.get_python_type_name_from_python_object_address(object_address, || {
            let object_memory = self.memory_reader.read_bytes(object_address, python_object_memory_size as u64)?;

            if object_memory.len() != python_object_memory_size {
                return Some(String::from("Length is not 16"));
            }

            let type_object_address = u64::from_le_bytes(object_memory[object_type_offset..object_type_offset + 8].try_into().unwrap());
            self.get_python_type_name_from_type_object_address(type_object_address)
        })
    }

    pub fn get_python_type_name_from_type_object_address(
        &self,
        type_object_address: u64,
    ) -> Option<String> {
        let python_type_object_memory_size: usize = 32;
        let name_max_length: u64 = 100;
        let type_object_name_offset: usize = 24;

        let type_object_memory = self.memory_reader.read_bytes(type_object_address, python_type_object_memory_size as u64)?;

        if type_object_memory.len() != python_type_object_memory_size {
            return Some(String::from("Length is not 32"));
        }

        let tp_name_address = u64::from_le_bytes(type_object_memory[type_object_name_offset..type_object_name_offset + 8].try_into().unwrap());
        let name_bytes = self.memory_reader.read_bytes(tp_name_address, name_max_length)?;

        let null_terminator_index = name_bytes.iter().position(|&byte| byte == 0)?;

        Some(std::str::from_utf8(&name_bytes[..null_terminator_index]).ok()?.to_string())
    }

    pub fn read_python_string_value(
        &self,
        string_object_address: u64,
        max_length: i32,
    ) -> Option<String> {

        let string_object_memory_size: usize = 32; // 0x20 in hex
        let string_object_ob_size_offset: usize = 16; // 0x10 in hex
        let string_bytes_offset: u64 = 32; // 8 * 4 in decimal

        //  https://github.com/python/cpython/blob/362ede2232107fc54d406bb9de7711ff7574e1d4/Include/stringobject.h

        let string_object_memory = self
            .memory_reader
            .read_bytes(string_object_address, string_object_memory_size as u64)?;

        if string_object_memory.len() != string_object_memory_size {
            return None; // Failed to read string object memory
        }

        let string_object_ob_size = u64::from_ne_bytes(
            string_object_memory[string_object_ob_size_offset..string_object_ob_size_offset + 8]
                .try_into()
                .ok()?,
        );

        if (max_length > 0 && max_length < string_object_ob_size as i32)
            || (string_object_ob_size > i32::MAX as u64)
        {
            return None; // String too long
        }

        let string_bytes = self.memory_reader.read_bytes(
            string_object_address + string_bytes_offset,
            string_object_ob_size,
        )?;

        if string_bytes.len() != string_object_ob_size as usize {
            return None; // Failed to read string bytes
        }

        Some(String::from_utf8_lossy(&string_bytes).into_owned())
    }

    pub fn reading_from_python_type_unicode(&self, address: u64) -> Option<Rc<String>> {
        let python_object_memory_size: usize = 32; // 0x20 in hex
        let unicode_string_length_offset: usize = 16; // 0x10 in hex
        let unicode_string_max_length: u64 = 4096; // 0x1000 in hex
        let string_bytes_offset: usize = 24; // 0x18 in hex

        let python_object_memory = self
            .memory_reader
            .read_bytes(address, python_object_memory_size as u64)?;

        if python_object_memory.len() != python_object_memory_size {
            return None; // Failed to read python object memory
        }

        let unicode_string_length = u64::from_ne_bytes(
            python_object_memory[unicode_string_length_offset..unicode_string_length_offset + 8]
                .try_into()
                .ok()?,
        );

        if unicode_string_length > unicode_string_max_length {
            return None; // String too long
        }

        let string_bytes_count = (unicode_string_length * 2) as usize;

        let string_start_address = u64::from_ne_bytes(
            python_object_memory[string_bytes_offset..string_bytes_offset + 8]
                .try_into()
                .ok()?,
        );

        let string_bytes = self
            .memory_reader
            .read_bytes(string_start_address, string_bytes_count as u64)?;

        if string_bytes.len() != string_bytes_count {
            return None; // Failed to read string bytes
        }

        Some(Rc::new(String::from_utf8_lossy(&string_bytes).into_owned()))
    }

    pub fn reading_from_python_type_bool(&self, address: u64) -> Option<Rc<bool>> {
        let python_object_memory_size: usize = 24; // 0x18 in hex
        let boolean_value_offset: usize = 16; // 0x10 in hex

        let python_object_memory = self
            .memory_reader
            .read_bytes(address, python_object_memory_size as u64)?;

        if python_object_memory.len() != python_object_memory_size {
            return None; // Failed to read python object memory
        }

        let boolean_value = i64::from_ne_bytes(
            python_object_memory[boolean_value_offset..boolean_value_offset + 8]
                .try_into()
                .ok()?,
        );

        Some(Rc::new(boolean_value != 0))
    }

    pub fn read_python_float_object_value(&self, float_object_address: u64) -> Option<Rc<f64>> {
        //  https://github.com/python/cpython/blob/362ede2232107fc54d406bb9de7711ff7574e1d4/Include/floatobject.h

        let python_object_memory_size: usize = 32; // 0x20 in hex
        let float_value_offset: usize = 16; // 0x10 in hex

        let python_object_memory = self
            .memory_reader
            .read_bytes(float_object_address, python_object_memory_size as u64)?;

        if python_object_memory.len() != python_object_memory_size {
            return None; // Failed to read python object memory
        }

        let float_value = f64::from_ne_bytes(
            python_object_memory[float_value_offset..float_value_offset + 8]
                .try_into()
                .ok()?,
        );

        Some(Rc::new(float_value))
    }

    pub fn read_python_string_value_max_length_4000(
        &self,
        str_object_address: u64,
    ) -> Option<String> {
        self.cache
            .get_python_string_value_max_length_4000(str_object_address, || {
                self.read_python_string_value(str_object_address, 4000)
            })
    }

    pub fn get_dictionary_entries_with_string_keys(
        &self,
        dictionary_object_address: u64,
    ) -> HashMap<String, u64> {
        let dictionary_entries =
            self.read_active_dictionary_entries_from_dictionary_address(dictionary_object_address);

        if dictionary_entries.is_none() {
            return HashMap::new(); // Return an empty HashMap instead of ImmutableDictionary.Empty
        }

        let mut result = HashMap::new();

        for entry in dictionary_entries.unwrap().iter() {
            if let Some(key) = self.read_python_string_value_max_length_4000(entry.key) {
                result.insert(key, entry.value);
            }
        }

        result
    }

    pub fn read_active_dictionary_entries_from_dictionary_address(
        &self,
        dictionary_address: u64,
    ) -> Option<Rc<Vec<PyDictEntry>>> {
        let dict_memory_size: usize = 48; // 0x30 in hex

        /*
        Sources:
        https://github.com/python/cpython/blob/362ede2232107fc54d406bb9de7711ff7574e1d4/Include/dictobject.h
        https://github.com/python/cpython/blob/362ede2232107fc54d406bb9de7711ff7574e1d4/Objects/dictobject.c
        */

        let dict_memory = self
            .memory_reader
            .read_bytes(dictionary_address, dict_memory_size as u64)?;

        if dict_memory.len() != dict_memory_size {
            return None; // Failed to read dictionary memory
        }

        let dict_memory_as_long_memory =
            memory_utils::transform_memory_content_as_ulong_memory(&dict_memory);

        //  https://github.com/python/cpython/blob/362ede2232107fc54d406bb9de7711ff7574e1d4/Include/dictobject.h#L60-L89

        let ma_fill = dict_memory_as_long_memory[2];
        let ma_used = dict_memory_as_long_memory[3];
        let ma_mask = dict_memory_as_long_memory[4];
        let ma_table = dict_memory_as_long_memory[5];

        let number_of_slots = (ma_mask + 1) as usize;

        if number_of_slots > 10_000 {
            return None; // Avoid processing dictionaries with potentially corrupted data
        }

        let slots_memory_size = number_of_slots * 8 * 3;

        let slots_memory = self
            .memory_reader
            .read_bytes(ma_table, slots_memory_size as u64)?;

        if slots_memory.len() != slots_memory_size {
            return None; // Failed to read slots memory
        }

        let slots_memory_as_long_memory =
            memory_utils::transform_memory_content_as_ulong_memory(&slots_memory);

        let mut entries = Vec::new();

        for slot_index in 0..number_of_slots {
            let hash = slots_memory_as_long_memory[slot_index * 3];
            let key = slots_memory_as_long_memory[slot_index * 3 + 1];
            let value = slots_memory_as_long_memory[slot_index * 3 + 2];

            if key != 0 && value != 0 {
                entries.push(PyDictEntry { hash, key, value });
            }
        }

        Some(Rc::new(entries))
    }

    pub fn get_dict_entry_value_representation(
        &self,
        value_object_address: u64,
    ) -> Option<Arc<Box<dyn std::any::Any>>> {
        self.cache
            .get_dict_entry_value_representation(value_object_address, || {
                let value_python_type_name =
                    self.get_python_type_name_from_python_object_address(value_object_address);

                let generic_representation = Box::new(DictEntryValueGenericRepresentation {
                    address: value_object_address,
                    python_object_type_name: None,
                }) as Box<dyn std::any::Any>;

                match value_python_type_name {
                    None => {
                        return None
                    }
                    _ => {}
                }

                //TODO remove this when all work is done
                /*if value_python_type_name.is_none() {
                    return Some(Rc::new(generic_representation));
                }*/


                //TODO implement specialized_reading_from_python_type
                /*let specialized_representation = self.specialized_reading_from_python_type(
                    value_object_address,
                    &value_python_type_name.unwrap(),
                );*/
                let specialized_representation = None;

                if specialized_representation.is_none() {
                    return Some(Arc::new(generic_representation));
                }

                specialized_representation
            })
    }

    fn get_python_type_name_from_python_object_address(
        &self,
        object_address: u64) -> Option<String> {
        self.cache
            .get_python_type_name_from_python_object_address(object_address, || {
                let object_memory = self.memory_reader.read_bytes(object_address, 0x10)?;

                if object_memory.len() != 0x10 {
                    return Some(String::from("Length is not 0x10"));
                }

                return self.get_python_type_name_from_type_object_address(
                    u64::from_le_bytes(object_memory[8..].try_into().unwrap())
                );
            })
    }
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommonColorComponents {
    pub alpha: u8,
    pub red: u8,
    pub green: u8,
    pub blue: u8,
}
