use std::cell::RefCell;
use crate::eve::ui::common::common::{ChildOfNodeWithDisplayRegion, ChildWithRegion, ChildWithoutRegion, DisplayRegion, UITreeNodeWithDisplayRegion};
use lazy_static::lazy_static;
use std::collections::{HashMap, HashSet};
use std::rc::Rc;
use crate::eve::ui::ui_constants::{UiConstants, UiZonesEnum};
use crate::process::interop::ui::ui_tree_node::UiTreeNode;



pub fn get_display_region_from_dict_entries(
    entries_of_interest: &HashMap<String, serde_json::Value>,
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

pub fn get_display_region_from_ui_node(ui_node: &UiTreeNode) -> Option<DisplayRegion> {
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

fn fixed_number_from_ui_node(property_name: &str, ui_node: &UiTreeNode) -> Option<i32> {
    ui_node
        .dict_entries_of_interest
        .get(property_name)
        .and_then(|boxed_any| {
            boxed_any
                .downcast_ref::<serde_json::Value>() // Intenta hacer downcast a &serde_json::Value
                .and_then(|json_value| extract_int_from_int_or_string(json_value))
        })
}



pub fn as_ui_tree_node_with_inherited_offset<'a>(
    inherited_offset: (i32, i32),
    occluded_regions: &mut Vec<DisplayRegion>,
    raw_node: &'a UiTreeNode,
) -> Rc<RefCell<dyn ChildOfNodeWithDisplayRegion<'a> + 'a>> {
    if let Some(self_region) = get_display_region_from_ui_node(&raw_node) {
        let total_display_region = DisplayRegion {
            x: self_region.x + inherited_offset.0,
            y: self_region.y + inherited_offset.1,
            width: self_region.width,
            height: self_region.height,
        };

        let tree_node_with_display_region = parse_child_of_node_with_display_region(
            raw_node,
            &self_region,
            &total_display_region,
            occluded_regions,
        );
        let child_of_node: Rc<RefCell<dyn ChildOfNodeWithDisplayRegion>> = Rc::new(RefCell::new(ChildWithRegion {
            node: tree_node_with_display_region,
        }));
        child_of_node
    } else {
        let child_of_node: Rc<RefCell<dyn ChildOfNodeWithDisplayRegion>> = Rc::new(RefCell::new(ChildWithoutRegion {
            node: raw_node,
        }));
        child_of_node
    }
}

pub fn parse_child_of_node_with_display_region<'a>(
    ui_tree_node: &'a UiTreeNode,
    self_display_region: &DisplayRegion,
    total_display_region: &DisplayRegion,
    occluded_regions: &mut Vec<DisplayRegion>,
) -> UITreeNodeWithDisplayRegion<'a> {
    let mut mapped_siblings: Vec<Rc<RefCell<dyn ChildOfNodeWithDisplayRegion>>> = Vec::new();
    let mut occluded_regions_from_siblings: Vec<DisplayRegion> = Vec::new();

    let test_flatten = ui_tree_node.children.iter().flatten();
    for child in test_flatten {
        let child_result = as_ui_tree_node_with_inherited_offset(
            (total_display_region.x, total_display_region.y),
            &mut occluded_regions_from_siblings,
            child,
        );


        if let child_with_region = just_case_with_display_region(&child_result) {

           let data =child_with_region.unwrap();
            mapped_siblings.insert(0, Rc::clone(&child_result));
            let descendants_with_display_region: Vec<&'a ChildWithRegion> = list_descendants_with_display_region(data.node.children.as_ref());
            let new_occluded_regions = descendants_with_display_region
                .iter()
                .filter(|cwr| node_occludes_following_nodes(&cwr.node))
                .map(|cwr| cwr.node.total_display_region.clone())
                .collect::<Vec<DisplayRegion>>();

            occluded_regions_from_siblings.extend(new_occluded_regions);

            occluded_regions.extend(occluded_regions_from_siblings.iter().cloned());
        }
    }

    mapped_siblings.reverse();
    let total_display_region_visible = DisplayRegion { x: -1, y: -1, width: 0, height: 0 };

    UITreeNodeWithDisplayRegion {
        ui_node: ui_tree_node,
        children: Option::from(mapped_siblings),
        self_display_region: self_display_region.clone(),
        total_display_region: total_display_region.clone(),
        total_display_region_visible,
    }
}

pub fn just_case_with_display_region<'a>(
    child: &Rc<RefCell<dyn ChildOfNodeWithDisplayRegion<'a> + 'a>>,
) -> Option<&'a ChildWithRegion<'a>> {

    None
}

pub fn node_occludes_following_nodes(node: &UITreeNodeWithDisplayRegion) -> bool {
    let known_occluding_types = [
        "SortHeaders", "ContextMenu", "OverviewWindow", "DronesWindow",
        "SelectedItemWnd", "InventoryPrimary", "ChatWindowStack",
    ];

    known_occluding_types.contains(&node.ui_node.object_type_name.as_str())
}

pub fn list_descendants_in_ui_tree_node(parent: &UiTreeNode) -> Vec<&UiTreeNode> {
    let mut descendants: Vec<&UiTreeNode> = Vec::new();

    for child in parent.children.iter().flatten() {
        descendants.push(child);
        descendants.extend(list_descendants_in_ui_tree_node(child));
    }

    descendants
}

fn fixed_number_from_property_name(
    property_name: &str,
    entries_of_interest: &HashMap<String, serde_json::Value>,
) -> Option<i32> {
    entries_of_interest
        .get(property_name)
        .and_then(|json_value| extract_int_from_int_or_string(json_value))
}

pub fn list_descendants_with_display_region<'a>(
    children_of_node: Option<&'a Vec<Rc<RefCell<dyn ChildOfNodeWithDisplayRegion<'a> + 'a>>>>,
) -> Vec<&'a ChildWithRegion<'a>> {
    let mut all_descendants: Vec<&'a ChildWithRegion<'a>> = Vec::new();


    let children_with_regions: Vec<&'a ChildWithRegion<'a>> = if let Some(children) = children_of_node {
        list_children_with_display_region(Some(children))
    } else {
        Vec::new()
    };

    for child_with_region in children_with_regions.iter() {

        all_descendants.push(child_with_region);

        // Recurse to get the descendants of the current child
        let descendants = list_descendants_with_display_region(child_with_region.node.children.as_ref());
        all_descendants.extend(descendants);
    }

    all_descendants
}

pub fn list_children_with_display_region<'a>(
    children_of_node: Option<&'a Vec<Rc<RefCell<dyn ChildOfNodeWithDisplayRegion<'a> + 'a>>>>,
) -> Vec<&'a ChildWithRegion<'a>> {
    if let Some(children) = children_of_node {
        children
            .iter()
            .filter_map(|child| just_case_with_display_region(child))
            .collect()
    } else {
        Vec::new()
    }
}



fn extract_int_from_int_or_string(object_value: &serde_json::Value) -> Option<i32> {
    match object_value {
        serde_json::Value::Number(num) => num.as_i64().map(|n| n as i32),
        serde_json::Value::String(string_value) => {
            if let Ok(parsed_int) = string_value.parse::<i32>() {
                Some(parsed_int)
            } else {
                // Log o manejo del error si falla la conversión
                println!("Failed to parse integer from string '{}'", string_value);
                None
            }
        }
        _ => None,
    }
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

pub struct EvePythonUtils;

impl EvePythonUtils {
    pub fn is_key_of_interest(key: &str) -> bool {
        DICT_ENTRIES_OF_INTEREST_KEYS.contains(key)
    }
}


lazy_static! {
    static ref DICT_ENTRIES_OF_INTEREST_KEYS: HashSet<&'static str> = {
        let mut set = HashSet::new();
        set.insert("_top");
        set.insert("_left");
        set.insert("_width");
        set.insert("_height");
        set.insert("_displayX");
        set.insert("_displayY");
        set.insert("_displayHeight");
        set.insert("_displayWidth");
        set.insert("_name");
        set.insert("_text");
        set.insert("_setText");
        set.insert("children");
        set.insert("texturePath");
        set.insert("_bgTexturePath");
        set.insert("_hint");
        set.insert("_display");
        set.insert("lastShield");
        set.insert("lastArmor");
        set.insert("lastStructure");
        set.insert("_lastValue");
        set.insert("ramp_active");
        set.insert("_rotation");
        set.insert("_color");
        set.insert("_sr");
        set.insert("htmlstr");
        set.insert("_texturePath");
        set.insert("_opacity");
        set.insert("_bgColor");
        set.insert("isExpanded");
        set
    };


}
