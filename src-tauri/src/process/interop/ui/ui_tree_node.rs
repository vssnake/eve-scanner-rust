use std::collections::HashMap;
use serde::{Serialize, Deserialize};

#[derive(Debug)]
pub struct UiTreeNode {
    pub object_address: u64,
    pub object_type_name: String,
    pub dict_entries_of_interest: HashMap<String, Box<dyn std::any::Any>>,
    pub other_dict_entries_keys: Option<Vec<String>>,
    pub children: Option<Vec<UiTreeNode>>,
}

#[derive(Debug)]
pub struct Bunch {
    pub entries_of_interest: serde_json::Map<String, serde_json::Value>,
}

impl UiTreeNode {
    pub fn enumerate_self_and_descendants(&self) -> Vec<&UiTreeNode> {
        let mut result = vec![self];
        if let Some(children) = &self.children {
            for child in children {
                result.extend(child.enumerate_self_and_descendants());
            }
        }
        result
    }

    pub fn new(
        object_address: u64,
        object_type_name: String,
        dict_entries_of_interest: HashMap<String, Box<dyn std::any::Any>>,
        other_dict_entries_keys: Option<Vec<String>>,
        children: Option<Vec<UiTreeNode>>,
    ) -> UiTreeNode {
        UiTreeNode {
            object_address,
            object_type_name,
            dict_entries_of_interest,
            other_dict_entries_keys,
            children,
        }
    }
}