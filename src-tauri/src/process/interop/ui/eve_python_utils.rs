
use std::any::Any;
use crate::eve::ui::common::common::{ChildOfNodeWithDisplayRegion, ChildWithRegion, ChildWithoutRegion, DisplayRegion};
use crate::process::interop::ui::ui_tree_node::{UITreeNodeWithDisplayRegion, UiTreeNode};
use lazy_static::lazy_static;
use std::collections::{HashMap, HashSet};
use std::rc::Rc;
use crate::process::interop::ui::int_wrapper::IntWrapper;

pub fn get_display_region_from_dict_entries(
    entries_of_interest: &HashMap<String, Rc<Box<dyn Any>>>,
) -> Option<DisplayRegion> {
    let display_x = fixed_number_from_property_name("_displayX", entries_of_interest);
    let display_y = fixed_number_from_property_name("_displayY", entries_of_interest);
    let display_width =
        fixed_number_from_property_name("_displayWidth", entries_of_interest);
    let display_height =
        fixed_number_from_property_name("_displayHeight", entries_of_interest);

    if let (Some(x), Some(y), Some(width), Some(height)) =
        (display_x, display_y, display_width, display_height)
    {
        return Some(DisplayRegion {
            x,
            y,
            width,
            height,
        });
    }

    None
}

pub fn get_display_region_from_ui_node(ui_node: &Rc<UiTreeNode>) -> Option<DisplayRegion> {
    let display_x = fixed_number_from_ui_node("_displayX", ui_node);
    let display_y = fixed_number_from_ui_node("_displayY", ui_node);
    let display_width = fixed_number_from_ui_node("_displayWidth", ui_node);
    let display_height = fixed_number_from_ui_node("_displayHeight", ui_node);

    if let (Some(x), Some(y), Some(width), Some(height)) = (display_x, display_y, display_width, display_height) {
        return Some(DisplayRegion {
            x,
            y,
            width,
            height,
        });
    }

    None
}

fn fixed_number_from_ui_node(property_name: &str, ui_node: &Rc<UiTreeNode>) -> Option<i32> {
    let property_to_convert_option = ui_node
        .dict_entries_of_interest
        .get(property_name);
    
    if (property_to_convert_option.is_none()) {
        return None;
    }
    
    let property_to_convert = property_to_convert_option?;
    
    let number = extract_int_from_int_or_string(property_to_convert);
    
    
   number
}



pub fn as_ui_tree_node_with_inherited_offset(
    inherited_offset: (i32, i32),
    occluded_regions: &mut Vec<Rc<DisplayRegion>>,
    raw_node: &Rc<UiTreeNode>,
) -> Rc<dyn ChildOfNodeWithDisplayRegion> {
    if let Some(self_region) = get_display_region_from_ui_node(&raw_node) {
        let total_display_region = Rc::new(DisplayRegion {
            x: self_region.x + inherited_offset.0,
            y: self_region.y + inherited_offset.1,
            width: self_region.width,
            height: self_region.height,
        });

        let tree_node_with_display_region = parse_child_of_node_with_display_region(
            raw_node,
            &Rc::new(self_region),
            &total_display_region,
            occluded_regions,
        );
        let child_of_node = Rc::new(ChildWithRegion {
            node: tree_node_with_display_region,
        });
        child_of_node
    } else {
        let child_of_node = Rc::new(ChildWithoutRegion {
            node: Rc::clone(raw_node),
        });
        child_of_node
    }
}

pub fn parse_child_of_node_with_display_region(
    ui_tree_node: &Rc<UiTreeNode>,
    self_display_region: &Rc<DisplayRegion>,
    total_display_region: &Rc<DisplayRegion>,
    occluded_regions: &mut Vec<Rc<DisplayRegion>>,
) -> UITreeNodeWithDisplayRegion {
    let mut childs_with_region: Vec<Rc<ChildWithRegion>> = Vec::new();
    let mut childs_without_region: Vec<Rc<ChildWithoutRegion>> = Vec::new();
    let mut occluded_regions_from_siblings: Vec<Rc<DisplayRegion>> = Vec::new();

    for x in &ui_tree_node.children {
        let child_result = as_ui_tree_node_with_inherited_offset(
            (total_display_region.x, total_display_region.y),
            &mut occluded_regions_from_siblings,
            x,
        );

        if let Some(child_with_region) = just_case_with_display_region(Rc::clone(&child_result)) {
            childs_with_region.push(Rc::clone(&child_with_region));
            let descendants_with_display_region: Vec<Rc<ChildWithRegion>> = list_descendants_with_display_region(&child_with_region.node.child_with_region);
            let new_occluded_regions = descendants_with_display_region
                .iter()
                .filter(|cwr| node_occludes_following_nodes(&cwr.node))
                .map(|cwr| Rc::clone(&cwr.node.total_display_region))
                .collect::<Vec<Rc<DisplayRegion>>>();

            occluded_regions_from_siblings.extend(new_occluded_regions);

            occluded_regions.extend(occluded_regions_from_siblings.iter().cloned());
        }else {
            childs_without_region.push(child_result.as_any_rc().downcast::<ChildWithoutRegion>().unwrap());
        }
    }


    childs_with_region.reverse();
    childs_without_region.reverse();
    let total_display_region_visible = DisplayRegion { x: -1, y: -1, width: 0, height: 0 };

    UITreeNodeWithDisplayRegion {
        ui_node: Rc::clone(ui_tree_node),
        child_with_region: childs_with_region,
        child_without_region: childs_without_region,
        self_display_region: Rc::clone(self_display_region),
        total_display_region: total_display_region.clone(),
        total_display_region_visible,
    }
}

pub fn just_case_with_display_region(
    child: Rc<dyn ChildOfNodeWithDisplayRegion>,
) -> Option<Rc<ChildWithRegion>> {
    
    if child.has_region() {
       
        let child_as_any_rc = child.as_any_rc();
        let child_with_region=  child_as_any_rc.downcast::<ChildWithRegion>();
        
        if (child_with_region.is_err()) {
            return None;
        }
        Some(child_with_region.unwrap())
    } else {
        None
    }
    
}

pub fn node_occludes_following_nodes(node: &UITreeNodeWithDisplayRegion) -> bool {
    let known_occluding_types = [
        "SortHeaders", "ContextMenu", "OverviewWindow", "DronesWindow",
        "SelectedItemWnd", "InventoryPrimary", "ChatWindowStack",
    ];

    known_occluding_types.contains(&node.ui_node.object_type_name.as_str())
}

pub fn list_descendants_in_ui_tree_node(children:Vec<Rc<UiTreeNode>>) -> Vec<Rc<UiTreeNode>> {
    let mut descendants: Vec<Rc<UiTreeNode>> = Vec::new();

    for child in children {
        descendants.push(Rc::clone(&child));
        descendants.extend(list_descendants_in_ui_tree_node(child.children.iter().map(|c| Rc::clone(c)).collect()));
    }

    descendants
}

fn fixed_number_from_property_name(
    property_name: &str,
    entries_of_interest: &HashMap<String, Rc<Box<dyn Any>>>,
) -> Option<i32> {
    entries_of_interest
        .get(property_name)
        .and_then(|json_value| extract_int_from_int_or_string(json_value))
}

pub fn list_descendants_with_display_region(
    children: &Vec<Rc<ChildWithRegion>>,
) -> Vec<Rc<ChildWithRegion>> {
    let mut all_descendants: Vec<Rc<ChildWithRegion>> = Vec::new();

    for child_with_region in children.iter() {

        all_descendants.push(Rc::clone(child_with_region));

        // Recurse to get the descendants of the current child
        let descendants = list_descendants_with_display_region(&child_with_region.node.child_with_region);
        all_descendants.extend(descendants);
    }

    all_descendants
}

pub fn list_children_with_display_region(
    children_of_node: &Vec<Rc<dyn ChildOfNodeWithDisplayRegion>>,
) -> Vec<Rc<ChildWithRegion>> {
    children_of_node.iter().filter_map(|child| just_case_with_display_region(Rc::clone(child))).collect()
}



fn extract_int_from_int_or_string(object_value: &Box<dyn Any>) -> Option<i32> {
    if let Some(long_int) = object_value.downcast_ref::<IntWrapper>() {
        return long_int.get_i32();
    } else if let Some(&int_value) = object_value.downcast_ref::<i32>() {
        return Some(int_value);
    } else if let Some(string_value) = object_value.downcast_ref::<String>() {
        if let Ok(parsed_int) = string_value.parse::<i32>() {
            return Some(parsed_int);
        } else {
            // Log or handle error if parsing fails
            println!("Failed to parse integer from string '{}'", string_value);
        }
    }

    // Return None if the value cannot be decoded
    None
}


pub struct NodeOcclusion;

lazy_static! {
    static ref PYTHON_OBJECT_TYPES_KNOWN_TO_OCCLUDE: HashSet<&'static str> = {
        let mut set = HashSet::new();
        set.insert("SortHeaders");
        set.insert("ContextMenu");
        set.insert("OverviewWindow");
        set.insert("DronesWindow");
        set.insert("SelectedItemWnd");
        set.insert("InventoryPrimary");
        set.insert("ChatWindowStack");
        set
    };
}

impl NodeOcclusion {
    pub fn node_occludes_following_nodes(node: &UITreeNodeWithDisplayRegion) -> bool {
        PYTHON_OBJECT_TYPES_KNOWN_TO_OCCLUDE.contains(node.ui_node.object_type_name.as_str())
    }
}
