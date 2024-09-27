use crate::eve::ui::models::child_of_node::{ChildWithRegion, ChildWithoutRegion};
use crate::eve::ui::models::display_region::DisplayRegion;
use std::any::Any;
use std::collections::HashMap;
use std::rc::Rc;

#[derive(Debug)]
pub struct UiTreeNode {
    pub object_address: u64,
    pub object_type_name: String,
    pub dict_entries_of_interest: HashMap<String, Rc<Box<dyn Any>>>,
    pub other_dict_entries_keys: Vec<String>,
    pub children: Vec<Rc<UiTreeNode>>,
}

impl UiTreeNode {
    pub fn enumerate_self_and_descendants(&self) -> Vec<&Rc<UiTreeNode>> {
        let mut result: Vec<&Rc<UiTreeNode>> = Vec::new();
        let test = &self.children;
        for child in test {
            result.extend(child.enumerate_self_and_descendants())
        }
        result
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

    pub fn list_descendants_in_ui_tree_node(children: Vec<Rc<UiTreeNode>>) -> Vec<Rc<UiTreeNode>> {
        let mut descendants: Vec<Rc<UiTreeNode>> = Vec::new();

        for child in children {
            descendants.push(Rc::clone(&child));
            descendants.extend(UiTreeNode::list_descendants_in_ui_tree_node(
                child.children.iter().map(|c| Rc::clone(c)).collect(),
            ));
        }

        descendants
    }
}

pub struct UITreeNodeWithDisplayRegion {
    pub ui_node: Rc<UiTreeNode>,
    pub child_with_region: Vec<Rc<ChildWithRegion>>,
    pub child_without_region: Vec<Rc<ChildWithoutRegion>>,
    pub self_display_region: Rc<DisplayRegion>,
    pub total_display_region: Rc<DisplayRegion>,
    pub total_display_region_visible: DisplayRegion,
}

impl UITreeNodeWithDisplayRegion {}

pub struct ScrollControls {
    pub ui_node: Rc<UITreeNodeWithDisplayRegion>,
    pub scroll_handle: Option<Rc<UITreeNodeWithDisplayRegion>>,
}
