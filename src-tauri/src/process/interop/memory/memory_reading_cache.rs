use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::hash::Hash;
use winapi::um::winnt::FirmwareTypeUnknown;

pub struct MemoryReadingCache {
    python_type_name_from_python_object_address: Arc<Mutex<HashMap<u64, String>>>,
    python_string_value_max_length_4000: Arc<Mutex<HashMap<u64, String>>>,
    dict_entry_value_representation: Arc<Mutex<HashMap<u64, Arc<Box<dyn std::any::Any>>>>>,
}

impl MemoryReadingCache {
    pub fn new() -> Self {
        Self {
            python_type_name_from_python_object_address: Arc::new(Mutex::new(HashMap::new())),
            python_string_value_max_length_4000: Arc::new(Mutex::new(HashMap::new())),
            dict_entry_value_representation: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn get_python_type_name_from_python_object_address<F>(&self, address: u64, get_fresh: F) -> Option<String>
    where
        F: FnOnce() -> Result<String, &'static str>,
    {
        self.get_from_cache_or_update(&self.python_type_name_from_python_object_address, address, get_fresh)
    }

    pub fn get_python_string_value_max_length_4000<F>(&self, address: u64, get_fresh: F) -> Option<String>
    where
        F: FnOnce() -> Result<String, &'static str>,
    {
        self.get_from_cache_or_update(&self.python_string_value_max_length_4000, address, get_fresh)
    }

    pub fn get_dict_entry_value_representation<F>(&self, address: u64, get_fresh: F) -> Option<Arc<Box<dyn std::any::Any>>>
    where
        F: FnOnce() -> Result<Arc<Box<dyn std::any::Any>>, &'static str>,
    {
        self.get_from_cache_or_update(&self.dict_entry_value_representation, address, get_fresh)
    }

    fn get_from_cache_or_update<K, V, F>(
        &self,
        cache: &Arc<Mutex<HashMap<K, V>>>,
        key: K,
        get_fresh: F,
    ) -> Option<V>
    where
        K: Eq + Hash + Copy,
        V: Clone,
        F: FnOnce() -> Result<V, &'static str>,
    {
        let mut cache_lock = cache.lock().unwrap();

        if let Some(from_cache) = cache_lock.get(&key) {
            return Some(from_cache.clone());
        }

        let fresh = get_fresh();
        
        if fresh.is_ok() {
            let result = fresh.unwrap();
            cache_lock.insert(key, result.clone());
            return Some(result);
        }

        None
    }
}
