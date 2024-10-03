use std::collections::{HashMap, HashSet};
use std::rc::Rc;
use crate::eve::ui_tree_node::common::common::ColorComponents;
use crate::eve::ui_tree_node::models::child_of_node::ChildWithRegion;
use crate::eve::ui_tree_node::models::ui_tree_node::{ScrollControls, UITreeNodeWithDisplayRegion};
use crate::eve::ui_tree_node::ui_constants::{UiZonesEnum, UI_ZONES};

#[derive(Debug)]
pub struct OverviewWindow {
    pub ui_node: Rc<UITreeNodeWithDisplayRegion>,
    pub entries_headers: Vec<(String, Rc<UITreeNodeWithDisplayRegion>)>,
    pub entries: Vec<OverviewWindowEntry>,
    pub scroll_controls: Option<ScrollControls>,
}

impl OverviewWindow {
    pub fn parse_overview_windows(
        zones: HashMap<UiZonesEnum, Vec<Rc<UITreeNodeWithDisplayRegion>>>,
    ) -> Vec<Rc<OverviewWindow>> {
        let overview_windows = zones.get(&UiZonesEnum::Overview).unwrap();
        let overview_windows_with_region = overview_windows
            .iter()
            .map(|node| OverviewWindow::parse_overview_window(Rc::clone(node)))
            .collect::<Vec<Rc<OverviewWindow>>>();
        
        overview_windows_with_region
        
    }

    fn parse_overview_window(region_node: Rc<UITreeNodeWithDisplayRegion>) -> Rc<OverviewWindow> {
        Rc::new(OverviewWindow::parse(region_node))
    }
}

#[derive(Debug)]
pub struct OverviewWindowEntry {
    pub ui_node: Rc<UITreeNodeWithDisplayRegion>,
    pub texts_left_to_right: Vec<String>,
    pub cells_texts: HashMap<String, String>,
    pub object_distance: Option<String>,
    pub object_distance_in_meters: i32,
    pub object_name: Option<String>,
    pub object_type: Option<String>,
    pub object_alliance: Option<String>,
    pub is_player: bool,
    pub icon_sprite_color_percent: Option<ColorComponents>,
    pub names_under_space_object_icon: HashSet<String>,
    pub bg_color_fills_percent: Vec<ColorComponents>,
    pub right_aligned_icons_hints: Vec<String>,
    pub common_indications: OverviewWindowEntryCommonIndications,
    pub opacity_percent: Option<i32>,
}

#[derive(Debug)]
pub struct OverviewWindowEntryCommonIndications {
    pub targeting: bool,
    pub targeted_by_me: bool,
    pub is_jamming_me: bool,
    pub is_warp_disrupting_me: bool,
}