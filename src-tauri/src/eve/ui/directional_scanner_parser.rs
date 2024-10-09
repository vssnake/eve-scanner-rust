use std::collections::HashMap;
use std::rc::Rc;
use std::result;
use regex::Regex;
use crate::eve::ui::models::directional_scanner::{DirectionalScanner, DirectionalScannerEntry};
use crate::eve::ui::parser_utils::ParserUtils;
use crate::eve::ui_tree_node::models::ui_tree_node::{UITreeNodeWithDisplayRegion, UiTreeNode};
use crate::eve::ui_tree_node::ui_constants::UiZonesEnum;
use crate::eve::ui_tree_node::utils::display_region_utils::DisplayRegionUtils;
use crate::eve::ui_tree_node::utils::utils::UiUtils;

impl DirectionalScanner {
    
    pub fn parse_directional_scanner(zones: &HashMap<UiZonesEnum, Vec<Rc<UITreeNodeWithDisplayRegion>>>)-> Option<Rc<DirectionalScanner>>{
        let directional_scanner = zones.get(&UiZonesEnum::DirectionalScanner);
        if directional_scanner.is_none() {
            return None;
        }
        let directional_scanner = DirectionalScanner::parse(directional_scanner.unwrap()[0].clone());
        
        Some(Rc::new(directional_scanner))
    }
    pub fn parse(region_node: Rc<UITreeNodeWithDisplayRegion>) -> DirectionalScanner {
        
        let childs_with_region =
            DisplayRegionUtils::list_descendants_with_display_region(&region_node.child_with_region);

        let list_node_with_entries = childs_with_region.iter().flat_map(|child| {
            if let Some(name_entry) = ParserUtils::get_name_from_dict_entries(&child.node.ui_node) {
                if !name_entry.starts_with("entry_") {
                    return None;
                }
            } else {
                return None;
            }

            return DirectionalScanner::extract_entry(&child.node);
            
        }).collect::<Vec<DirectionalScannerEntry>>();

        DirectionalScanner {
            ui_node: region_node,
            entries: list_node_with_entries,
        }
    }
    
    fn extract_entry(node: &Rc<UITreeNodeWithDisplayRegion>)-> Option<DirectionalScannerEntry>{
        
        let ship_icon_node = node.child_with_region.get(3)
            .and_then(|child1| child1.node.child_with_region.get(0))
            .map(|child2| &child2.node.ui_node);
        
        let ship_icon = if let Some(ship_icon_node) = ship_icon_node {
            let result = ParserUtils::get_string_property_from_dict_entries(ship_icon_node, 
                                                    "_texturePath");
            result
        }else { None };
        
        let ship_name_node = node.child_with_region.get(2)
            .and_then(|child1| child1.node.child_with_region.get(0))
            .map(|child2| &child2.node.ui_node);
        
        let ship_name = if let Some(ship_name_node) = ship_name_node {
            UiTreeNode::get_display_text(ship_name_node)
        }else { "".parse().unwrap() };
        
        let ship_type_node = node.child_with_region.get(1)
            .and_then(|child1| child1.node.child_with_region.get(0))
            .map(|child2| &child2.node.ui_node);
        
        let ship_type = if let Some(ship_type_node) = ship_type_node {
            UiTreeNode::get_display_text(ship_type_node)
        }else { "".parse().unwrap() };
        
        let ship_type_localized = ParserUtils::extract_localized_name(&ship_type).unwrap_or("empty_default".parse().unwrap());
        
        let distance_node = node.child_with_region.get(0)
            .and_then(|child1| child1.node.child_with_region.get(0))
            .map(|child2| &child2.node.ui_node);
        
        
        let distance = if let Some(distance_node) = distance_node {
            let distance_string = UiTreeNode::get_display_text(distance_node);

            let distance = ParserUtils::parse_distance_in_meters_from_text(&Some(distance_string));
            distance
        }else { None };
        
        if  ship_icon.is_some()  {
            return  Some(DirectionalScannerEntry {
                distance,
                names: ship_name,
                ship_type: ship_type_localized,
                ship_icon: ship_icon.unwrap(),
            });
        } 
        None
    }
}