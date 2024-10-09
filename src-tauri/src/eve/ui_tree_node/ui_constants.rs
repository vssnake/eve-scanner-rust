use std::cell::RefCell;
use lazy_static::lazy_static;
use std::collections::{HashMap, HashSet};
use std::rc::Rc;
use crate::eve::ui_tree_node::models::ui_tree_node::UITreeNodeWithDisplayRegion;

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
    
    pub fn check_and_insert_inportant_zone(important_zones: &RefCell<HashMap<UiZonesEnum, Vec<Rc<UITreeNodeWithDisplayRegion>>>>, type_id: &String, node_ref: Rc<UITreeNodeWithDisplayRegion>) {
        let ui_zones = UI_ZONES.get(&type_id.as_str());
        match ui_zones {
            Some(zone) => {
                let mut important_zones = important_zones.borrow_mut();
                let zone = important_zones.entry(zone.clone()).or_insert(Vec::new());
                zone.push(node_ref.clone());
            },
            None => {
                //println!("Zone not found for type_id: {}", type_id);
            }
        }
    }
}

#[derive(Hash, Eq, PartialEq, Debug, Clone)]
pub enum UiZonesEnum {
    Overview,
    DirectionalScanner,
    ProbeScanner,
}

lazy_static! {
    pub static ref UI_ZONES: HashMap<&'static str, UiZonesEnum> = {
        let mut hash_map = HashMap::new();
        
        hash_map.insert("OverView", UiZonesEnum::Overview);
        hash_map.insert("OverviewWindow", UiZonesEnum::Overview);
        hash_map.insert("OverviewWindowOld", UiZonesEnum::Overview);
        hash_map.insert("DirectionalScanner", UiZonesEnum::DirectionalScanner);
        hash_map.insert("ProbeScannerWindow", UiZonesEnum::ProbeScanner);
        hash_map
    };
    
   
}

