use std::any::Any;
use std::collections::{HashMap, HashSet};
use std::rc::Rc;
use log::debug;
use serde::ser::SerializeMap;
use serde::Serialize;
use crate::eve::ui_tree_node::common::common::ColorComponents;
use crate::eve::ui_tree_node::models::child_of_node::{ChildWithRegion, ChildWithoutRegion};
use crate::eve::ui_tree_node::models::display_region::DisplayRegion;

#[derive(Debug,Serialize)]
pub struct UiTreeNode {
    pub object_address: u64,
    pub object_type_name: String,
    #[serde(serialize_with = "serialize_dict_entries")]
    pub dict_entries_of_interest: HashMap<String, Rc<Box<dyn Any>>>,
    #[serde(skip)]
    pub other_dict_entries_keys: Vec<String>,
    pub children: Vec<Rc<UiTreeNode>>,
}

fn serialize_dict_entries<S>(
    entries: &HashMap<String, Rc<Box<dyn Any>>>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    let mut map = serializer.serialize_map(Some(entries.len()))?;
    for (key, value) in entries {
        // Intentamos hacer un downcast a String, si es posible lo incluimos
        if let Some(val) = value.downcast_ref::<String>() {
            map.serialize_entry(key, val)?;
        }
    }
    map.end()
}

impl UiTreeNode {
    pub fn count_descendants(&self) -> usize {
        let mut count = 1;
        
        for child in &self.children {
            count += child.count_descendants();
        }

        count
    }
    
    pub fn extract_types(&self) -> Vec<String> {
        let mut types = HashSet::new();
        types.insert(self.object_type_name.clone());

        for child in &self.children {
            types.extend(child.extract_types());
        }

        let mut vec: Vec<String> = types.iter().cloned().collect();
        
        vec.sort();
        
        vec
    }


    pub fn new(
        object_address: u64,
        object_type_name: String,
        dict_entries_of_interest: HashMap<String, Rc<Box<dyn std::any::Any>>>,
        other_dict_entries_keys: Vec<String>,
        children: Vec<Rc<UiTreeNode>>,
    ) -> UiTreeNode {
        UiTreeNode {
            object_address,
            object_type_name,
            dict_entries_of_interest,
            other_dict_entries_keys,
            children,
        }
    }

    pub fn list_descendants_in_ui_tree_node(parent: &Rc<UiTreeNode>) -> Vec<Rc<UiTreeNode>> {
        let mut descendants: Vec<Rc<UiTreeNode>> = Vec::new();

        let parent_iter = Rc::clone(parent);
        for child in &parent_iter.children {
            descendants.push(Rc::clone(child));
            
            descendants.extend(UiTreeNode::list_descendants_in_ui_tree_node(
                &child
            ));
        }

        descendants
    }

    pub fn get_display_text(ui_node: &Rc<UiTreeNode>) -> String {
        let keys_to_search = vec!["_setText", "_text"];
        let mut longest_text = String::new();

        for key in keys_to_search {
            if let Some(text_value) = ui_node.dict_entries_of_interest.get(key) {
                let down_cast = text_value.downcast_ref::<String>();
                if down_cast.is_none() {
                    continue;
                }else { 
                    let text = down_cast.unwrap();
                    if text.len() > longest_text.len() {
                        longest_text = text.to_string();
                    }
                }
                /*let string_value_result = Rc::downcast::<String>(text_value.clone());
                if  string_value_result.is_err() {
                    continue;
                }else{
                    if let Ok(text) =  string_value_result {
                        if text.len() > longest_text.len() {
                            longest_text = text.to_string();
                        }
                    }
                }*/
                
            }
        }

        longest_text
    }
}
#[derive(Debug, Serialize)]
pub struct UITreeNodeWithDisplayRegion {
    pub ui_node: Rc<UiTreeNode>,
    #[serde(skip)]
    pub child_with_region: Vec<Rc<ChildWithRegion>>,
    #[serde(skip)]
    pub child_without_region: Vec<Rc<ChildWithoutRegion>>,
    #[serde(skip)]
    pub self_display_region: Rc<DisplayRegion>,
    #[serde(skip)]
    pub total_display_region: Rc<DisplayRegion>,
    #[serde(skip)]
    pub total_display_region_visible: DisplayRegion,
}

impl UITreeNodeWithDisplayRegion {}

#[derive(Debug)]
pub struct ScrollControls {
    pub ui_node: Rc<UITreeNodeWithDisplayRegion>,
    pub scroll_handle: Option<Rc<UITreeNodeWithDisplayRegion>>,
}
