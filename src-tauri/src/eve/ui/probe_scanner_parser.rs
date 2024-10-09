use std::collections::HashMap;
use std::rc::Rc;
use windows::Win32::UI::Shell::PARSEDURLA;
use crate::eve::ui::models::directional_scanner::{DirectionalScanner, DirectionalScannerEntry};
use crate::eve::ui::models::probe_scanner::{ProbeScanner, ProbeScannerEntry};
use crate::eve::ui::parser_utils::ParserUtils;
use crate::eve::ui_tree_node::models::ui_tree_node::{UITreeNodeWithDisplayRegion, UiTreeNode};
use crate::eve::ui_tree_node::ui_constants::UiZonesEnum;
use crate::eve::ui_tree_node::utils::display_region_utils::DisplayRegionUtils;

impl ProbeScanner {

    pub fn parse_probe_scanner(zones: &HashMap<UiZonesEnum, Vec<Rc<UITreeNodeWithDisplayRegion>>>)-> Option<Rc<ProbeScanner>>{
        let probe_scanner = zones.get(&UiZonesEnum::ProbeScanner);
        if probe_scanner.is_none() {
            return None;
        }
        let probe_scanner = ProbeScanner::parse(probe_scanner.unwrap()[0].clone());

        Some(Rc::new(probe_scanner))
    }
    pub fn parse(region_node: Rc<UITreeNodeWithDisplayRegion>) -> ProbeScanner {

        let childs_with_region =
            DisplayRegionUtils::list_descendants_with_display_region(&region_node.child_with_region);


        let list_node_with_entries = childs_with_region.iter().find(|child| {

            let parsed_name = ParserUtils::get_name_from_dict_entries(&child.node.ui_node);
            
            if parsed_name.is_none() {
                return false
            }else if parsed_name.unwrap().eq("__content") {
                return true
            }
            return false
        });
        
        let list_node_with_entries = if let Some(list_node_with_entries) = list_node_with_entries {
            
            list_node_with_entries.node.child_with_region.iter().map(|child| {
                ProbeScanner::extract_entry(&child.node)
            }).filter(|entry| entry.is_some()).map(|entry| entry.unwrap()).collect::<Vec<_>>()
            
        } else {
            return ProbeScanner {
                ui_node: region_node,
                entries: Vec::new(),
            }
        };

        ProbeScanner {
            ui_node: region_node,
            entries: list_node_with_entries,
        }
    }

    fn extract_entry(node: &Rc<UITreeNodeWithDisplayRegion>)-> Option<ProbeScannerEntry>{

        let node_to_extract_info = node.child_with_region.get(1)
            .and_then(|child1| child1.node.child_with_region.get(0))
           // .and_then(|child2| child2.node.child_with_region.get(0))
            .map(|child3| &child3.node);
        
        if node_to_extract_info.is_none() {
            return None;
        }
        
        let childs_with_region = &node_to_extract_info.unwrap().child_with_region;;
        
        
        let type_site_icon_node = childs_with_region.get(4)
            .and_then(|child1| child1.node.child_with_region.get(0))
            .map(|child2| &child2.node.ui_node);
        
        let distance_node = childs_with_region.get(3)
            .and_then(|child1| child1.node.child_with_region.get(0))
            .map(|child2| &child2.node.ui_node);
        
        let id_node = childs_with_region.get(2)
            .and_then(|child1| child1.node.child_with_region.get(0))
            .map(|child2| &child2.node.ui_node);
        
        let name_node = childs_with_region.get(1)
            .and_then(|child1| child1.node.child_with_region.get(0))
            .map(|child2| &child2.node.ui_node);
        
        let type_node = childs_with_region.get(0)
            .and_then(|child1| child1.node.child_with_region.get(0))
            .map(|child2| &child2.node.ui_node);
        
       

        let type_site_icon = if let Some(type_site_icon_node) = type_site_icon_node {
            let result = ParserUtils::get_string_property_from_dict_entries(type_site_icon_node,
                                                                            "_texturePath");
            result
        }else { None };

        let distance = if let Some(distance_node) = distance_node {
            UiTreeNode::get_display_text(distance_node)
        }else { "".parse().unwrap() };

        let id = if let Some(id_node) = id_node {
            UiTreeNode::get_display_text(id_node)
        }else { "".parse().unwrap() };

        let tye_site = if let Some(type_node) = type_node {
            UiTreeNode::get_display_text(type_node)
        }else { "".parse().unwrap() };

        let name = if let Some(name_node) = name_node {
            UiTreeNode::get_display_text(name_node)
        }else { "".parse().unwrap() };

        

        if  type_site_icon.is_some()  {
            return  Some(ProbeScannerEntry {
                distance: ParserUtils::parse_distance_in_meters_from_text(&Some(distance.to_string())),
                distance_unformatted: distance.to_string(),
                id,
                name,
                signal_strength: "".parse().unwrap(),
                type_emplacement: tye_site,
            });
        }
        None
    }
}