use crate::eve::ui::models::ui_tree_node::UITreeNodeWithDisplayRegion;
use lazy_static::lazy_static;
use std::collections::{HashMap, HashSet};
use std::rc::Rc;

pub struct UiConstants;

impl UiConstants {
    pub fn overview_window() -> HashSet<&'static str> {
        let overview_window: HashSet<&str> = ["OverView", "OverviewWindow", "OverviewWindowOld"]
            .iter()
            .cloned()
            .collect();
        overview_window
    }

    pub fn initialize_mapper() -> HashMap<UiZonesEnum, Vec<Rc<UITreeNodeWithDisplayRegion>>> {
        let mut mapper = HashMap::new();
        mapper.insert(UiZonesEnum::Overview, Vec::new());
        mapper
    }
}

#[derive(Hash, Eq, PartialEq, Debug, Clone)]
pub enum UiZonesEnum {
    Overview,
}

lazy_static! {
    pub static ref UI_ZONES: HashMap<&'static str, UiZonesEnum> = {
    let mut hash_map = HashMap::new();
        hash_map.insert("OverView",UiZonesEnum::Overview);
        hash_map.insert("_left",UiZonesEnum::Overview);
        hash_map.insert("_width",UiZonesEnum::Overview);
        hash_map
    };
}

