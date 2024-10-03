use std::collections::HashMap;
use std::rc::Rc;
use crate::eve::ui::models::overview_window::OverviewWindow;
use crate::eve::ui_tree_node::models::ui_tree_node::UITreeNodeWithDisplayRegion;
use crate::eve::ui_tree_node::ui_constants::UiZonesEnum;

#[derive(Debug)]
pub struct GeneralWindow {
    pub  overview_windows: Vec<Rc<OverviewWindow>>
}

impl GeneralWindow {
    pub fn parse_general_window(defined_zones: HashMap<UiZonesEnum, Vec<Rc<UITreeNodeWithDisplayRegion>>>) -> GeneralWindow {
        GeneralWindow {
            overview_windows: OverviewWindow::parse_overview_windows(defined_zones),
        }
    }
}