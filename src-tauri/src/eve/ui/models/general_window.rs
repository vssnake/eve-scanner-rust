use std::collections::HashMap;
use std::rc::Rc;
use serde::Serialize;
use crate::eve::ui::models::directional_scanner::DirectionalScanner;
use crate::eve::ui::models::overview_window::OverviewWindow;
use crate::eve::ui::models::probe_scanner::ProbeScanner;
use crate::eve::ui_tree_node::models::ui_tree_node::UITreeNodeWithDisplayRegion;
use crate::eve::ui_tree_node::ui_constants::UiZonesEnum;

#[derive(Debug, Serialize, Clone)]
pub struct GeneralWindow {
    pub overview_windows: Vec<Rc<OverviewWindow>>,
    pub directional_scanner: Option<Rc<DirectionalScanner>>,
    pub probe_scanner: Option<Rc<ProbeScanner>>
    
}

impl GeneralWindow {
    pub fn parse_general_window(defined_zones: HashMap<UiZonesEnum, Vec<Rc<UITreeNodeWithDisplayRegion>>>) -> GeneralWindow {
        GeneralWindow {
            overview_windows: OverviewWindow::parse_overview_windows(&defined_zones),
            directional_scanner: DirectionalScanner::parse_directional_scanner(&defined_zones),
            probe_scanner: ProbeScanner::parse_probe_scanner(&defined_zones)
        }
    }
}