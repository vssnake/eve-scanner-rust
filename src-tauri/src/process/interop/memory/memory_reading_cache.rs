use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::hash::Hash;

pub struct MemoryReadingCache {
    python_type_name_from_python_object_address: Arc<Mutex<HashMap<u64, Option<String>>>>,
    python_string_value_max_length_4000: Arc<Mutex<HashMap<u64, Option<String>>>>,
    dict_entry_value_representation: Arc<Mutex<HashMap<u64, Option<Arc<Box<dyn std::any::Any>>>>>>,
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
        F: FnOnce() -> Option<String>,
    {
        self.get_from_cache_or_update(&self.python_type_name_from_python_object_address, address, get_fresh)
    }

    pub fn get_python_string_value_max_length_4000<F>(&self, address: u64, get_fresh: F) -> Option<String>
    where
        F: FnOnce() -> Option<String>,
    {
        self.get_from_cache_or_update(&self.python_string_value_max_length_4000, address, get_fresh)
    }

    pub fn get_dict_entry_value_representation<F>(&self, address: u64, get_fresh: F) -> Option<Arc<Box<dyn std::any::Any>>>
    where
        F: FnOnce() -> Option<Arc<Box<dyn std::any::Any>>>,
    {
        self.get_from_cache_or_update(&self.dict_entry_value_representation, address, get_fresh)
    }

    fn get_from_cache_or_update<K, V, F>(
        &self,
        cache: &Arc<Mutex<HashMap<K, Option<V>>>>,
        key: K,
        get_fresh: F,
    ) -> Option<V>
    where
        K: Eq + Hash + Copy,
        V: Clone,
        F: FnOnce() -> Option<V>,
    {
        let mut cache_lock = cache.lock().unwrap();

        if let Some(from_cache) = cache_lock.get(&key) {
            return from_cache.clone();
        }

        let fresh = get_fresh();

        if fresh.is_some() {
            cache_lock.insert(key, fresh.clone());
        }

        fresh
    }
}
