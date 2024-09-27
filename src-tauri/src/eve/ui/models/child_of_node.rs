use crate::eve::ui::models::ui_tree_node::{UITreeNodeWithDisplayRegion, UiTreeNode};
use std::any::Any;
use std::rc::Rc;

pub trait ChildOfNodeWithDisplayRegion {
    fn has_region(&self) -> bool;
    fn as_any(&self) -> &dyn Any;
    fn as_any_rc(self: Rc<Self>) -> Rc<dyn Any>;
}

pub struct ChildWithRegion {
    pub node: UITreeNodeWithDisplayRegion,
}

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