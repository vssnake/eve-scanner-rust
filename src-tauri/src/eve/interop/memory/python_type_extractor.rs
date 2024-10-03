use crate::eve::interop::memory::memory_reading_cache::MemoryReadingCache;
use crate::eve::interop::memory::models::dict_entry::DictEntry;
use crate::eve::interop::memory::python_memory_reader::PythonMemoryReader;
use lazy_static::lazy_static;
use serde::Serialize;
use serde_json::{to_value, Value};
use std::any::Any;
use std::collections::{HashMap, HashSet};
use std::rc::Rc;
use crate::eve::ui_tree_node::common::common::{Bunch, ColorComponents};
use crate::eve::ui_tree_node::models::ui_tree_node::UiTreeNode;

pub struct PythonTypeExtractor;

impl PythonTypeExtractor {
    pub fn specialized_reading_from_python_type(
        memory_reader: &PythonMemoryReader,
        address: u64,
        value_python_type: &str,
        memory_reading_cache: &MemoryReadingCache,
    ) -> Result<Box<dyn Any>, &'static str> {
        let handler = TYPE_HANDLERS.get(value_python_type);
        if handler.is_none() {
            return Err("Failed to find handler for python type");
        }
        handler.unwrap()(memory_reader, address, memory_reading_cache)
    }

    pub fn is_key_of_interest(key: &str) -> bool {
        DICT_ENTRIES_OF_INTEREST_KEYS.contains(key)
    }
}

fn reading_from_python_type_str(
    memory_reader: &PythonMemoryReader,
    address: u64,
) -> Result<String, &'static str> {
    memory_reader.read_python_string_value(address, 0x1000)
}

fn reading_from_python_type_pycolor(
    memory_reader: &PythonMemoryReader,
    address: u64,
    cache: &MemoryReadingCache,
) -> Result<Box<ColorComponents>, &'static str> {
    let py_color_object_memory = memory_reader.read_bytes(address, 0x18);

    if py_color_object_memory
        .as_ref()
        .map_or(true, |bytes| bytes.len() != 0x18)
    {
        return Err("Failed to read pyColorObjectMemory.");
    }

    let dictionary_address =
        u64::from_le_bytes(py_color_object_memory?[0x10..0x18].try_into().unwrap());

    let dictionary_entries =
        memory_reader.get_dictionary_entries_with_string_keys(dictionary_address, cache);

    if dictionary_entries.is_empty()
        || (!dictionary_entries.contains_key("_r")
        && !dictionary_entries.contains_key("_g")
        && !dictionary_entries.contains_key("_b")
        && !dictionary_entries.contains_key("_a"))
    {
        return Err("Failed to read dictionary entries.");
    }

    let read_value_percent_from_dict_entry_key = |key: &str| -> Result<i32, &'static str> {
        if let Some(&value_address) = dictionary_entries.get(key) {
            let value_as_float = memory_reader.read_python_float_object_value(value_address)?;
            Ok((value_as_float * 255.0) as i32)
        } else {
            Err("Key not found.")
        }
    };

    Ok(Box::new(ColorComponents {
        alpha: read_value_percent_from_dict_entry_key("_a")?,
        red: read_value_percent_from_dict_entry_key("_r")?,
        green: read_value_percent_from_dict_entry_key("_g")?,
        blue: read_value_percent_from_dict_entry_key("_b")?,
    }))
}

fn reading_from_python_type_bunch(
    memory_reader: &PythonMemoryReader,
    address: u64,
    cache: &MemoryReadingCache,
) -> Result<Box<Bunch>, &'static str> {
    let dictionary_entries = memory_reader.get_dictionary_entries_with_string_keys(address, cache);

    if dictionary_entries.is_empty() {
        // Failed to read dictionary entries.
        return Err("Failed to read dictionary entries.");
    }

    let mut entries_of_interest = Vec::new();

    for (key, value) in dictionary_entries {
        if DICT_ENTRIES_OF_INTEREST_KEYS.contains(key.as_str()) {
            let dict_entry = DictEntry {
                key: key.clone(),
                value: memory_reader.get_dict_entry_value_representation(value, cache),
            };
            entries_of_interest.push(dict_entry);
        }
    }

    let mut entries_of_interest_map = serde_json::Map::new();
    for dict_entry in entries_of_interest {
        let serialized_value = serialize_memory_reading_node_to_json(dict_entry.value);
        if serialized_value.is_ok() {
            entries_of_interest_map.insert(dict_entry.key, serialized_value?);
        }
    }

    Ok(Box::new(Bunch {
        entries_of_interest: entries_of_interest_map,
    }))
}

fn reading_from_python_type_link(
    memory_reader: &PythonMemoryReader,
    address: u64,
    cache: &MemoryReadingCache,
) -> Result<Box<UiTreeNode>, &'static str> {
    let python_object_type_name =
        memory_reader.get_python_type_name_from_python_object_address(address, cache)?;

    let link_memory = memory_reader.read_bytes(address, 0x40)?;

    let link_memory_as_long_memory: Vec<u64> = link_memory
        .chunks(8)
        .map(|chunk| u64::from_le_bytes(chunk.try_into().unwrap()))
        .collect();

    /*
     * 2024-05-26 observed a reference to a dictionary object at offset 6 * 4 bytes.
     * */

    let first_dict_reference = link_memory_as_long_memory
        .iter()
        .find(|&&reference| {
            let result =
                memory_reader.get_python_type_name_from_python_object_address(reference, cache);
            result.is_ok() && result.unwrap() == "dict"
        })
        .cloned();

    if (first_dict_reference.is_none()) {
        return Err("Failed to find dictionary reference.");
    }

    let dict_entries = memory_reader
        .get_dictionary_entries_with_string_keys(first_dict_reference.unwrap(), cache)
        .into_iter()
        .map(|(key, value)| {
            (
                key,
                memory_reader.get_dict_entry_value_representation(value, cache),
            )
        })
        .collect();

    Ok(Box::new(UiTreeNode::new(
        address,
        python_object_type_name,
        dict_entries,
        vec![],
        vec![],
    )))
}

trait SerializeBox: Any + Send + Sync {
    fn serialize_to_value(&self) -> Result<Value, serde_json::Error>;
}

impl<T: Serialize + Any + Send + Sync> SerializeBox for T {
    fn serialize_to_value(&self) -> Result<Value, serde_json::Error> {
        to_value(self)
    }
}

fn serialize_memory_reading_node_to_json(value: Rc<Box<dyn Any>>) -> Result<Value, &'static str> {
    if let Some(serialize_box) = value.downcast_ref::<Box<dyn SerializeBox>>() {
        serialize_box
            .serialize_to_value()
            .map_err(|_| "Error al serializar")
    } else {
        Err("El valor no es serializable.")
    }
}

type HandlerFn = fn(PythonMemoryReader, u64, MemoryReadingCache) -> Result<dyn Any, &'static str>;

lazy_static! {
    static ref TYPE_HANDLERS: HashMap<
        String,
        fn(&PythonMemoryReader, u64, &MemoryReadingCache) -> Result<Box<dyn Any>, &'static str>,
    > = {
        let mut m: HashMap<
            String,
            fn(&PythonMemoryReader, u64, &MemoryReadingCache) -> Result<Box<dyn Any>, &'static str>,
        > = HashMap::new();
        m.insert(
            String::from("str"),
            |mr: &PythonMemoryReader, addr, _cache| {
                reading_from_python_type_str(mr, addr).map(|value| Box::new(value) as Box<dyn Any>)
            },
        );
        m.insert(
            String::from("unicode"),
            |mr: &PythonMemoryReader, addr, _cache| {
                mr.reading_from_python_type_unicode(addr)
                    .map(|value| Box::new(value) as Box<dyn Any>)
            },
        );
        m.insert(
            String::from("int"),
            |mr: &PythonMemoryReader, addr, _cache| {
                mr.reading_from_python_type_int(addr)
                    .map(|value| Box::new(value) as Box<dyn Any>)
            },
        );
        m.insert(
            String::from("bool"),
            |mr: &PythonMemoryReader, addr, _cache| {
                mr.reading_from_python_type_bool(addr)
                    .map(|value| Box::new(value) as Box<dyn Any>)
            },
        );
        m.insert(
            String::from("float"),
            |mr: &PythonMemoryReader, addr, _cache| {
                mr.read_python_float_object_value(addr)
                    .map(|value| Box::new(value) as Box<dyn Any>)
            },
        );
        m.insert(
            String::from("PyColor"),
            |mr: &PythonMemoryReader, addr, cache| {
                reading_from_python_type_pycolor(mr, addr, cache).map(|value| value as Box<dyn Any>)
            },
        );
        m.insert(
            String::from("Bunch"),
            |mr: &PythonMemoryReader, addr, cache| {
                reading_from_python_type_bunch(mr, addr, cache).map(|value| value as Box<dyn Any>)
            },
        );
        m.insert(
            String::from("Link"),
            |mr: &PythonMemoryReader, addr, cache| {
                reading_from_python_type_link(mr, addr, cache).map(|value| value as Box<dyn Any>)
            },
        );
        m
    };
    static ref DICT_ENTRIES_OF_INTEREST_KEYS: HashSet<&'static str> = {
        let mut set = HashSet::new();
        set.insert("_top");
        set.insert("_left");
        set.insert("_width");
        set.insert("_height");
        set.insert("_displayX");
        set.insert("_displayY");
        set.insert("_displayHeight");
        set.insert("_displayWidth");
        set.insert("_name");
        set.insert("_text");
        set.insert("_setText");
        set.insert("children");
        set.insert("texturePath");
        set.insert("_bgTexturePath");
        set.insert("_hint");
        set.insert("_display");
        set.insert("lastShield");
        set.insert("lastArmor");
        set.insert("lastStructure");
        set.insert("_lastValue");
        set.insert("ramp_active");
        set.insert("_rotation");
        set.insert("_color");
        set.insert("_sr");
        set.insert("htmlstr");
        set.insert("_texturePath");
        set.insert("_opacity");
        set.insert("_bgColor");
        set.insert("isExpanded");
        set
    };
}
