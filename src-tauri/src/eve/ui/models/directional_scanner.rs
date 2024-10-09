use std::rc::Rc;
use serde::Serialize;
use crate::eve::ui_tree_node::models::ui_tree_node::UITreeNodeWithDisplayRegion;

#[derive(Debug, Serialize)]
pub struct DirectionalScanner {
    #[serde(skip_serializing)]
    pub ui_node: Rc<UITreeNodeWithDisplayRegion>,
    pub entries: Vec<DirectionalScannerEntry>,
}

#[derive(Debug, Serialize)]
pub struct DirectionalScannerEntry {
    pub distance: Option<i32>,
    pub names: String,
    pub ship_type: String,
    pub ship_icon: String,
}