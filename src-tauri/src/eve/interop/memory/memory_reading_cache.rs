use std::cell::RefCell;
use std::collections::HashMap;
use std::hash::Hash;
use std::rc::Rc;

pub struct MemoryReadingCache {
    python_type_name_from_python_object_address: Rc<RefCell<HashMap<u64, String>>>,
    python_string_value_max_length_4000: Rc<RefCell<HashMap<u64, String>>>,
    dict_entry_value_representation: Rc<RefCell<HashMap<u64, Rc<Box<dyn std::any::Any>>>>>,
}


impl MemoryReadingCache {
    pub fn new() -> Self {
        Self {
            python_type_name_from_python_object_address: Rc::new(RefCell::new(HashMap::new())),
            python_string_value_max_length_4000: Rc::new(RefCell::new(HashMap::new())),
            dict_entry_value_representation: Rc::new(RefCell::new(HashMap::new())),
        }
    }

    pub fn clear(&self) {
        self.python_string_value_max_length_4000.borrow_mut().clear();
        self.python_type_name_from_python_object_address.borrow_mut().clear();
        self.dict_entry_value_representation.borrow_mut().clear();
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

    pub fn get_dict_entry_value_representation<F>(&self, address: u64, get_fresh: F) -> Option<Rc<Box<dyn std::any::Any>>>
    where
        F: FnOnce() -> Result<Rc<Box<dyn std::any::Any>>, &'static str>,
    {
        self.get_from_cache_or_update(&self.dict_entry_value_representation, address, get_fresh)
    }

    fn get_from_cache_or_update<K, V, F>(
        &self,
        cache: &Rc<RefCell<HashMap<K, V>>>,
        key: K,
        get_fresh: F,
    ) -> Option<V>
    where
        K: Eq + Hash + Copy,
        V: Clone,
        F: FnOnce() -> Result<V, &'static str>,
    {
        {
            let cache_lock = cache.borrow();
            if let Some(from_cache) = cache_lock.get(&key) {
                return Some(from_cache.clone());
            }
        }

        let fresh = get_fresh();

        if fresh.is_ok() {
            let result = fresh.unwrap();
            let mut cache_lock = cache.borrow_mut();
            cache_lock.insert(key, result.clone());
            return Some(result);
        }

        None
    }
}
