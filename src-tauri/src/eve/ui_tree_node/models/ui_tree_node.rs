use std::any::Any;
use std::collections::HashMap;
use std::rc::Rc;
use log::debug;
use crate::eve::ui_tree_node::common::common::ColorComponents;
use crate::eve::ui_tree_node::models::child_of_node::{ChildWithRegion, ChildWithoutRegion};
use crate::eve::ui_tree_node::models::display_region::DisplayRegion;

#[derive(Debug)]
pub struct UiTreeNode {
    pub object_address: u64,
    pub object_type_name: String,
    pub dict_entries_of_interest: HashMap<String, Rc<Box<dyn Any>>>,
    pub other_dict_entries_keys: Vec<String>,
    pub children: Vec<Rc<UiTreeNode>>,
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
        let mut types = Vec::new();
        types.push(self.object_type_name.clone());

        for child in &self.children {
            types.extend(child.extract_types());
        }

        types
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

    pub fn get_display_text(ui_node: Rc<UiTreeNode>) -> String {
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
#[derive(Debug)]
pub struct UITreeNodeWithDisplayRegion {
    pub ui_node: Rc<UiTreeNode>,
    pub child_with_region: Vec<Rc<ChildWithRegion>>,
    pub child_without_region: Vec<Rc<ChildWithoutRegion>>,
    pub self_display_region: Rc<DisplayRegion>,
    pub total_display_region: Rc<DisplayRegion>,
    pub total_display_region_visible: DisplayRegion,
}

impl UITreeNodeWithDisplayRegion {}

#[derive(Debug)]
pub struct ScrollControls {
    pub ui_node: Rc<UITreeNodeWithDisplayRegion>,
    pub scroll_handle: Option<Rc<UITreeNodeWithDisplayRegion>>,
}
