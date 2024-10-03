use std::any::Any;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use crate::eve::ui_tree_node::common::common::ColorComponents;
use crate::eve::ui_tree_node::models::display_region::DisplayRegion;
use crate::eve::ui_tree_node::models::ui_tree_node::{ScrollControls, UITreeNodeWithDisplayRegion, UiTreeNode};
use crate::eve::ui_tree_node::utils::display_region_utils::DisplayRegionUtils;

pub struct ParserUtils{}

impl ParserUtils {
    pub fn parse_list_view_entry(
        entry_headers: &Vec<(String, Rc<UITreeNodeWithDisplayRegion>)>,
        list_view_entry_node: Rc<UITreeNodeWithDisplayRegion>,
    ) -> HashMap<String, String> {
        /*
        Observations show two different kinds of representations of the texts in the cells in a list view:
    
        + Each cell text in a dedicated UI element. (Overview entry)
        + All cell texts in a single UI element, separated by a tab-tag (<t>) (Inventory item)
    
        Following is an example of the latter case:
        Condensed Scordite<t><right>200<t>Scordite<t><t><t><right>30 m3<t><right>2.290,00 ISK
        */

        if entry_headers.is_empty() {
            return HashMap::new();
        }

        let mut cells_texts = HashMap::new();;
        let leftmost_header = &entry_headers[0];
        let all_texts_with_regions = ParserUtils::get_all_contained_display_texts_with_region(&list_view_entry_node);

        for (cell_text, cell) in all_texts_with_regions {
            let distance_from_leftmost_header =
                cell.total_display_region.x - leftmost_header.1.total_display_region.x;

            let matched_header = entry_headers.iter().find(|header| {
                ParserUtils::header_region_matches_cell_region(&header.1.total_display_region, &cell.total_display_region)
            });

            if let Some(matched_header) = matched_header {
                cells_texts.insert(matched_header.0.clone(), cell_text);
            } else if distance_from_leftmost_header.abs() < 4 {
                // No operation if distance is too small
            } else {
                // Split cellText by "<t>" and trim the results
                let split_texts: Vec<String> = cell_text
                    .split("<t>")
                    .map(|t| t.trim().to_string())
                    .filter(|t| !t.is_empty())
                    .collect();

                for (i, split_text) in split_texts.iter().enumerate() {
                    if i < entry_headers.len() {
                        cells_texts.insert(entry_headers[i].0.clone(), split_text.clone());
                    }
                }
            }
        }

        cells_texts
    }
    
    pub fn get_all_contained_display_texts_with_region(
        ui_node: &Rc<UITreeNodeWithDisplayRegion>,
    ) -> Vec<(String, Rc<UITreeNodeWithDisplayRegion>)> {
        let mut result = Vec::new();

        
            let descendant_children = DisplayRegionUtils::list_descendants_with_display_region(&ui_node.child_with_region);

            for descendant in descendant_children {
                
                let display_text = UiTreeNode::get_display_text(descendant.node.ui_node.clone());
                if !display_text.is_empty() {
                    result.push((display_text, descendant.node.clone()));
                }
            }
        

        result
    }

    fn header_region_matches_cell_region(
        header_region: &DisplayRegion,
        cell_region: &DisplayRegion,
    ) -> bool {
        (header_region.x < cell_region.x + 3)
            && (header_region.x + header_region.width > cell_region.x + cell_region.width - 3)
    }

    pub fn parse_scroll_controls(scroll_node: &Rc<UITreeNodeWithDisplayRegion>) -> ScrollControls {
        let scroll_handle = DisplayRegionUtils::list_descendants_with_display_region(&scroll_node.child_with_region)
            .into_iter()
            .find(|node| node.node.ui_node.object_type_name == "ScrollHandle");

        ScrollControls {
            ui_node: scroll_node.clone(),
            scroll_handle: scroll_handle.map(|node| node.node.clone()),
        }
    }

    pub fn parse_overview_entry_distance_in_meters_from_text(
        distance_display_text_before_trim: &Option<String>,
    ) -> i32 {
        
        if (distance_display_text_before_trim.is_none()) {
            return -1;
        }

        let trimmed = distance_display_text_before_trim
            .as_deref() 
            .map(|s| s.trim())
            .unwrap_or("");
        
        let parts: Vec<&str> = trimmed.split_whitespace().collect();
        if parts.len() < 2 {
            return -1;
        }

        let unit_text = parts[parts.len() - 1];
        let number_text_parts: Vec<&str> = parts[..parts.len() - 1].to_vec();

        let unit_in_meters = ParserUtils::parse_distance_unit_in_meters(unit_text);
        if unit_in_meters.is_none() {
            // Too far away
            return -1;
        }

        let number_text = number_text_parts.join(" ");
        let number = ParserUtils::parse_number_truncating_after_optional_decimal_separator(&number_text);
        
        if number.is_err(){
            return  -1
        }

        number.unwrap() * unit_in_meters.unwrap()
    }
    
    fn parse_distance_unit_in_meters(unit_text: &str) -> Option<i32> {
        match unit_text {
            "m" => Some(1),          
            "km" => Some(1000),      
            _ => None,           
        }
    }
    
    fn parse_number_truncating_after_optional_decimal_separator(number_text: &str) -> Result<i32, String> {
        let number_text = number_text.replace('.', "");
        number_text.parse::<i32>().map_err(|e| format!("Failed to parse number: {}", e))
    }

    pub fn get_name_from_dict_entries(ui_tree_node: &UiTreeNode) -> Option<String> {
        ParserUtils::get_string_property_from_dict_entries(ui_tree_node, "_name")
    }

    pub fn get_hint_text_from_dict_entries(ui_tree_node: &UiTreeNode) -> Option<String> {
        ParserUtils::get_string_property_from_dict_entries(ui_tree_node, "_hint")
    }

    pub fn get_opacity_from_dict_entries(ui_tree_node: &UiTreeNode) -> Option<i32> {
        if let Some(opacity) = ParserUtils::get_string_property_from_dict_entries(ui_tree_node, "_opacity") {
            if let Ok(opacity_value) = opacity.parse::<f32>() {
                return Some((opacity_value * 100.0).round() as i32);
            }
        }
        None
    }

    pub fn get_color_percentage_from_dict_entries(
        ui_tree_node: &UiTreeNode,
    ) -> Option<ColorComponents>{
        if let Some(object_value) = ui_tree_node.dict_entries_of_interest.get("_color") {
           // if let Some(boxed_any) = object_value.clone().downcast_ref::<Box<dyn Any>>() {
                if let Some(color_components) = object_value.downcast_ref::<ColorComponents>() {
                    return Some(color_components.clone());
                }
           // }
        }
        None
    }

    pub fn get_string_property_from_dict_entries(
        ui_tree_node: &UiTreeNode,
        property_name: &str,
    ) -> Option<String> {
        if let Some(object_value) = ui_tree_node.dict_entries_of_interest.get(property_name) {
            //if let Some(boxed_any) = object_value.clone().downcast_ref::<Box<dyn Any>>() {
                if let Some(string_property) = object_value.downcast_ref::<String>() {
                    return Some(string_property.clone());
                }
           // }
        }
        None
    }
    
    
}