use crate::eve::interop::memory::memory_reading_cache::MemoryReadingCache;

use crate::eve::interop::memory::windows_memory_reader::WindowsMemoryReader;

use crate::eve::interop::memory::models::dict_entry_representation::PyDictEntryRepresentation;
use crate::eve::interop::memory::models::int_wrapper::IntWrapper;
use crate::eve::interop::memory::models::py_dict_entry::PyDictEntry;
use crate::eve::interop::memory::utils::MemoryUtils;
use crate::eve::interop::ui::python_type_extractor::PythonTypeExtractor;
use std::collections::HashMap;
use std::rc::Rc;

pub struct PythonMemoryReader {
    memory_reader: Rc<WindowsMemoryReader>,
}

impl PythonMemoryReader {
    pub fn new(windows_memory_reader: &Rc<WindowsMemoryReader>) -> Self {
        Self {
            memory_reader: Rc::clone(windows_memory_reader),
        }
    }

    pub fn read_bytes(&self, address: u64, size: u64) -> Result<Vec<u8>, &'static str> {
        self.memory_reader.read_bytes(address, size)
    }

    pub fn get_python_type_name_from_object_address(
        &self,
        object_address: u64,
        memory_reading_cache: &MemoryReadingCache,
    ) -> Result<String, &'static str> {
        let python_object_memory_size: usize = 16;

        let object_type_offset: usize = 8;

        let result = memory_reading_cache.get_python_type_name_from_python_object_address(
            object_address,
            || {
                let object_memory = self
                    .memory_reader
                    .read_bytes(object_address, python_object_memory_size as u64)?;

                if object_memory.len() != python_object_memory_size {
                    return Err("Length is not 16");
                }

                let type_object_address = u64::from_le_bytes(
                    object_memory[object_type_offset..object_type_offset + 8]
                        .try_into()
                        .unwrap(),
                );
                self.get_python_type_name_from_type_object_address(type_object_address)
            },
        );

        result.ok_or_else(|| "Failed to get python type name from object address")
    }

    pub fn get_python_type_name_from_type_object_address(
        &self,
        type_object_address: u64,
    ) -> Result<String, &'static str> {
        let python_type_object_memory_size: usize = 32;
        let name_max_length: u64 = 100;
        let type_object_name_offset: usize = 24;

        let type_object_memory = self
            .memory_reader
            .read_bytes(type_object_address, python_type_object_memory_size as u64)?;

        if type_object_memory.len() != python_type_object_memory_size {
            return Err("Length is not 32");
        }

        let tp_name_address = u64::from_le_bytes(
            type_object_memory[type_object_name_offset..type_object_name_offset + 8]
                .try_into()
                .unwrap(),
        );
        let name_bytes = self
            .memory_reader
            .read_bytes(tp_name_address, name_max_length)?;

        let null_terminator_index = name_bytes.iter().position(|&byte| byte == 0);

        if null_terminator_index.is_none() {
            return Err("Failed to find null terminator");
        }

        let result = std::str::from_utf8(&name_bytes[..null_terminator_index.unwrap()])
            .map_err(|_| "Failed to parse utf8")?;

        Ok(result.to_string())
    }

    pub fn read_python_string_value(
        &self,
        string_object_address: u64,
        max_length: i32,
    ) -> Result<String, &'static str> {
        let string_object_memory_size: usize = 32; // 0x20 in hex
        let string_object_ob_size_offset: usize = 16; // 0x10 in hex
        let string_bytes_offset: u64 = 32; // 8 * 4 in decimal

        //  https://github.com/python/cpython/blob/362ede2232107fc54d406bb9de7711ff7574e1d4/Include/stringobject.h

        let string_object_memory = self
            .memory_reader
            .read_bytes(string_object_address, string_object_memory_size as u64)?;

        if string_object_memory.len() != string_object_memory_size {
            return Err("Failed to read string bytes or the length of the string bytes is not equal to the string object size");
        }

        let string_object_ob_size = u64::from_ne_bytes(
            string_object_memory[string_object_ob_size_offset..string_object_ob_size_offset + 8]
                .try_into()
                .map_err(|_| "Failed to parse string object size")?,
        );

        if (max_length > 0 && max_length < string_object_ob_size as i32)
            || (string_object_ob_size > i32::MAX as u64)
        {
            return Err("String too long");
        }

        let string_bytes = self.memory_reader.read_bytes(
            string_object_address + string_bytes_offset,
            string_object_ob_size,
        )?;

        if string_bytes.len() != string_object_ob_size as usize {
            return Err("Failed to read string bytes or the length of the string bytes is not equal to the string object size");
        }

        Ok(String::from_utf8_lossy(&string_bytes).into_owned())
    }

    pub fn reading_from_python_type_unicode(&self, address: u64) -> Result<String, &'static str> {
        let python_object_memory_size: usize = 32; // 0x20 in hex
        let unicode_string_length_offset: usize = 16; // 0x10 in hex
        let unicode_string_max_length: u64 = 4096; // 0x1000 in hex
        let string_bytes_offset: usize = 24; // 0x18 in hex

        let python_object_memory = self
            .memory_reader
            .read_bytes(address, python_object_memory_size as u64)?;

        if python_object_memory.len() != python_object_memory_size {
            return Err("Failed to read python object memory"); // Failed to read python object memory
        }

        let unicode_string_length = u64::from_ne_bytes(
            python_object_memory[unicode_string_length_offset..unicode_string_length_offset + 8]
                .try_into()
                .map_err(|_| "Failed to slide")?,
        );

        if unicode_string_length > unicode_string_max_length {
            return Err("String too long");
        }

        let string_bytes_count = (unicode_string_length * 2) as usize;

        let string_start_address = u64::from_ne_bytes(
            python_object_memory[string_bytes_offset..string_bytes_offset + 8]
                .try_into()
                .map_err(|_| "Failed to slide")?,
        );

        let string_bytes = self
            .memory_reader
            .read_bytes(string_start_address, string_bytes_count as u64)?;

        if string_bytes.len() != string_bytes_count {
            return Err("Failed to read string bytes");
        }

        Ok(String::from_utf8_lossy(&string_bytes).into_owned())
    }

    pub fn reading_from_python_type_bool(&self, address: u64) -> Result<bool, &'static str> {
        let python_object_memory_size: usize = 24; // 0x18 in hex
        let boolean_value_offset: usize = 16; // 0x10 in hex

        let python_object_memory = self
            .memory_reader
            .read_bytes(address, python_object_memory_size as u64)?;

        if python_object_memory.len() != python_object_memory_size {
            return Err("Failed to read python object memory"); // Failed to read python object memory
        }

        let boolean_value = i64::from_ne_bytes(
            python_object_memory[boolean_value_offset..boolean_value_offset + 8]
                .try_into()
                .map_err(|_| "Failed to slide")?,
        );

        Ok(boolean_value != 0)
    }

    pub fn reading_from_python_type_int(&self, address: u64) -> Result<IntWrapper, &'static str> {
        let python_object_memory_size: usize = 24; // 0x18 in hex
        let int_value_offset: usize = 16; // 0x10 in hex

        // Read the memory at the specified address
        let python_object_memory = self
            .memory_reader
            .read_bytes(address, python_object_memory_size as u64)?;

        // Check that the read memory is of the expected size
        if python_object_memory.len() != python_object_memory_size {
            return Err("Failed to read python object memory");
        }

        // Extract the 64-bit integer value from the memory
        let int_value = i64::from_ne_bytes(
            python_object_memory[int_value_offset..int_value_offset + 8]
                .try_into()
                .map_err(|_| "Failed to slide")?,
        );

        // Check if the value can be represented as an i32
        let as_int32 = int_value as i32;

        // If the value fits into an i32, create an IntWrapper with the i32 value
        if as_int32 as i64 == int_value {
            Ok(IntWrapper::new_from_i32(as_int32))
        } else {
            // Otherwise, store it as an i64 in the IntWrapper
            Ok(IntWrapper::new_from_i64(int_value))
        }
    }

    pub fn read_python_float_object_value(
        &self,
        float_object_address: u64,
    ) -> Result<f64, &'static str> {
        //  https://github.com/python/cpython/blob/362ede2232107fc54d406bb9de7711ff7574e1d4/Include/floatobject.h

        let python_object_memory_size: usize = 32; // 0x20 in hex
        let float_value_offset: usize = 16; // 0x10 in hex

        let python_object_memory = self
            .memory_reader
            .read_bytes(float_object_address, python_object_memory_size as u64)?;

        if python_object_memory.len() != python_object_memory_size {
            return Err("Failed to read python object memory"); // Failed to read python object memory
        }

        let float_value = f64::from_ne_bytes(
            python_object_memory[float_value_offset..float_value_offset + 8]
                .try_into()
                .map_err(|_| "Failed to slide")?,
        );

        Ok(float_value)
    }

    pub fn read_python_string_value_max_length_4000(
        &self,
        str_object_address: u64,
        cache: &MemoryReadingCache,
    ) -> Result<String, &'static str> {
        let cache_result =
            cache.get_python_string_value_max_length_4000(str_object_address, || {
                return self.read_python_string_value(str_object_address, 4000);
            });

        cache_result.ok_or_else(|| "Failed to read python string value")
    }

    pub fn get_dictionary_entries_with_string_keys(
        &self,
        dictionary_object_address: u64,
        cache: &MemoryReadingCache,
    ) -> HashMap<String, u64> {
        let dictionary_entries_result =
            self.read_active_dictionary_entries_from_dictionary_address(dictionary_object_address);

        if dictionary_entries_result.is_err() {
            return HashMap::new(); // Return an empty HashMap instead of ImmutableDictionary.Empty
        }

        let dictionary_entries = dictionary_entries_result.unwrap();

        let mut result = HashMap::new();

        for entry in dictionary_entries.iter() {
            let key = self.read_python_string_value_max_length_4000(entry.key, cache);

            if key.is_ok() {
                result.insert(key.unwrap(), entry.value);
            }
        }

        result
    }

    pub fn read_active_dictionary_entries_from_dictionary_address(
        &self,
        dictionary_address: u64,
    ) -> Result<Vec<PyDictEntry>, &'static str> {
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
            return Err("Failed to read dictionary memory"); // Failed to read dictionary memory
        }

        let dict_memory_as_long_memory =
            MemoryUtils::transform_memory_content_as_ulong_memory(&dict_memory);

        //  https://github.com/python/cpython/blob/362ede2232107fc54d406bb9de7711ff7574e1d4/Include/dictobject.h#L60-L89

        let ma_fill = dict_memory_as_long_memory[2];
        let ma_used = dict_memory_as_long_memory[3];
        let ma_mask = dict_memory_as_long_memory[4];
        let ma_table = dict_memory_as_long_memory[5];

        let number_of_slots = (ma_mask + 1) as usize;

        if number_of_slots > 10_000 {
            return Err("Avoid processing dictionaries with potentially corrupted data");
            // Avoid processing dictionaries with potentially corrupted data
        }

        let slots_memory_size = number_of_slots * 8 * 3;

        let slots_memory = self
            .memory_reader
            .read_bytes(ma_table, slots_memory_size as u64)?;

        if slots_memory.len() != slots_memory_size {
            return Err("Failed to read slots memory"); // Failed to read slots memory
        }

        let slots_memory_as_long_memory =
            MemoryUtils::transform_memory_content_as_ulong_memory(&slots_memory);

        let mut entries = Vec::new();

        for slot_index in 0..number_of_slots {
            let hash = slots_memory_as_long_memory[slot_index * 3];
            let key = slots_memory_as_long_memory[slot_index * 3 + 1];
            let value = slots_memory_as_long_memory[slot_index * 3 + 2];

            if key != 0 && value != 0 {
                entries.push(PyDictEntry { hash, key, value });
            }
        }

        Ok(entries)
    }

    pub fn get_dict_entry_value_representation(
        &self,
        value_object_address: u64,
        memory_reading_cache: &MemoryReadingCache,
    ) -> Rc<Box<dyn std::any::Any>> {
        let result_cache =
            memory_reading_cache.get_dict_entry_value_representation(value_object_address, || {
                let value_python_type_name = self.get_python_type_name_from_python_object_address(
                    value_object_address,
                    memory_reading_cache,
                );

                let value_python_type_name_option = value_python_type_name.as_ref().ok();

                let generic_representation = Rc::new(Box::new(PyDictEntryRepresentation {
                    address: value_object_address,
                    python_object_type_name: value_python_type_name_option.cloned(),
                }) as Box<dyn std::any::Any>);

                if (value_python_type_name_option.is_none()) {
                    return Ok(generic_representation);
                }

                let specialized_representation =
                    PythonTypeExtractor::specialized_reading_from_python_type(
                        &self,
                        value_object_address,
                        &value_python_type_name_option.unwrap(),
                        memory_reading_cache,
                    );

                if specialized_representation.is_err() {
                    return Ok(generic_representation);
                }

                specialized_representation.map(|value| Rc::new(value))
            });

        if (result_cache.is_none()) {
            println!("Result cache is none");
        }
        result_cache.unwrap()
    }

    pub fn get_python_type_name_from_python_object_address(
        &self,
        object_address: u64,
        memory_reading_cache: &MemoryReadingCache,
    ) -> Result<String, &'static str> {
        let cache_result = memory_reading_cache.get_python_type_name_from_python_object_address(
            object_address,
            || {
                let object_memory = self.memory_reader.read_bytes(object_address, 0x10)?;

                if object_memory.len() != 0x10 {
                    return Err("Length is not 0x10");
                }

                return self.get_python_type_name_from_type_object_address(u64::from_le_bytes(
                    object_memory[8..].try_into().unwrap(),
                ));
            },
        );

        cache_result.ok_or_else(|| "Failed to get python type name from object address")
    }
}
