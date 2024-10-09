use std::any::Any;
use std::rc::Rc;
use serde::Serialize;
use crate::eve::ui_tree_node::models::ui_tree_node::{UITreeNodeWithDisplayRegion, UiTreeNode};

pub trait ChildOfNodeWithDisplayRegion {
    fn has_region(&self) -> bool;
    fn as_any(&self) -> &dyn Any;
    fn as_any_rc(self: Rc<Self>) -> Rc<dyn Any>;
}
#[derive(Debug,Serialize)]
pub struct ChildWithRegion {
    pub node: Rc<UITreeNodeWithDisplayRegion>,
}
#[derive(Debug ,Serialize)]
pub struct ChildWithoutRegion {
    pub node: Rc<UiTreeNode>,
}

impl ChildOfNodeWithDisplayRegion for ChildWithRegion {
    fn has_region(&self) -> bool {
        true
    }
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_rc(self: Rc<Self>) -> Rc<dyn Any> {
        self
    }
}

impl ChildOfNodeWithDisplayRegion for ChildWithoutRegion {
    fn has_region(&self) -> bool {
        false
    }
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_rc(self: Rc<Self>) -> Rc<dyn Any> {
        self
    }
}