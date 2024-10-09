use std::rc::Rc;
use serde::Serialize;
use crate::eve::ui_tree_node::models::ui_tree_node::UITreeNodeWithDisplayRegion;

#[derive(Debug, Serialize)]
pub struct ProbeScanner {
    #[serde(skip_serializing)]
    pub ui_node: Rc<UITreeNodeWithDisplayRegion>,
    pub entries: Vec<ProbeScannerEntry>,
}

#[derive(Debug, Serialize)]
pub struct ProbeScannerEntry {
    pub distance_unformatted: String,
    pub distance: Option<i32>,
    pub id: String,
    pub name: String,
    pub signal_strength: String,
    pub type_emplacement: String,
}