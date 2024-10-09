use crate::eve::utils::extract_int_from_int_or_string;
use lazy_static::lazy_static;
use std::any::Any;
use std::collections::{HashMap, HashSet};
use std::rc::Rc;
use std::result;
use crate::eve::ui_tree_node::models::ui_tree_node::{UITreeNodeWithDisplayRegion, UiTreeNode};

pub struct UiUtils;

impl UiUtils {
    pub fn fixed_number_from_property_name(
        property_name: &str,
        entries_of_interest: &HashMap<String, Rc<Box<dyn Any>>>,
    ) -> Option<i32> {
        entries_of_interest
            .get(property_name)
            .and_then(|json_value| extract_int_from_int_or_string(json_value))
    }

    pub fn fixed_number_from_ui_node(property_name: &str, ui_node: &Rc<UiTreeNode>) -> Option<i32> {
        let property_to_convert_option = ui_node
            .dict_entries_of_interest
            .get(property_name);

        if property_to_convert_option.is_none() {
            return None;
        }

        let property_to_convert = property_to_convert_option?;

        let number = extract_int_from_int_or_string(property_to_convert);


        number
    }

    pub fn node_occludes_following_nodes(node: &UITreeNodeWithDisplayRegion) -> bool {
        PYTHON_OBJECT_TYPES_KNOWN_TO_OCCLUDE.contains(node.ui_node.object_type_name.as_str())
    }
    
    pub fn find_node_in_tree(
        final_node_type: &String,
        parent_node: Rc<UITreeNodeWithDisplayRegion>,
    ) -> Option<Rc<UITreeNodeWithDisplayRegion>> {
        
        if *final_node_type == parent_node.ui_node.object_type_name {
            return Some(parent_node);
        }

        for child in parent_node.child_with_region.iter() {
            
            let result =  UiUtils::find_node_in_tree(
                final_node_type,
                child.node.clone());
            
            if result.is_some() {
                return result;
            }
        }

        None
    }
}


lazy_static! {
    static ref PYTHON_OBJECT_TYPES_KNOWN_TO_OCCLUDE: HashSet<&'static str> = {
        let mut set = HashSet::new();
        set.insert("SortHeaders");
        set.insert("ContextMenu");
        set.insert("OverviewWindow");
        set.insert("DronesWindow");
        set.insert("SelectedItemWnd");
        set.insert("InventoryPrimary");
        set.insert("ChatWindowStack");
        set
    };
}

