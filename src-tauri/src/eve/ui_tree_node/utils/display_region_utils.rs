use std::any::Any;
use std::collections::HashMap;
use std::rc::Rc;
use crate::eve::ui_tree_node::models::child_of_node::{ChildOfNodeWithDisplayRegion, ChildWithRegion, ChildWithoutRegion};
use crate::eve::ui_tree_node::models::display_region::DisplayRegion;
use crate::eve::ui_tree_node::models::ui_tree_node::{UITreeNodeWithDisplayRegion, UiTreeNode};
use crate::eve::ui_tree_node::utils::utils::UiUtils;

pub struct DisplayRegionUtils;

impl DisplayRegionUtils {
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
            let child_result = DisplayRegionUtils::create_display_region_node_with_offset(
                (total_display_region.x, total_display_region.y),
                &mut occluded_regions_from_siblings,
                x,
            );

            if let Some(child_with_region) =
                DisplayRegionUtils::just_case_with_display_region(Rc::clone(&child_result))
            {
                childs_with_region.push(Rc::clone(&child_with_region));
                let descendants_with_display_region: Vec<Rc<ChildWithRegion>> =
                    DisplayRegionUtils::list_descendants_with_display_region(
                        &child_with_region.node.child_with_region,
                    );
                let new_occluded_regions = descendants_with_display_region
                    .iter()
                    .filter(|cwr| DisplayRegionUtils::node_occludes_following_nodes(&cwr.node))
                    .map(|cwr| Rc::clone(&cwr.node.total_display_region))
                    .collect::<Vec<Rc<DisplayRegion>>>();

                occluded_regions_from_siblings.extend(new_occluded_regions);

                occluded_regions.extend(occluded_regions_from_siblings.iter().cloned());
            } else {
                childs_without_region.push(
                    Rc::downcast::<ChildWithoutRegion>(child_result.as_any_rc()).unwrap(),
                );
            }
        }

        childs_with_region.reverse();
        childs_without_region.reverse();
        let total_display_region_visible = DisplayRegion {
            x: -1,
            y: -1,
            width: 0,
            height: 0,
        };

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
            let child_with_region = child_as_any_rc.downcast::<ChildWithRegion>();

            if (child_with_region.is_err()) {
                return None;
            }
            Some(child_with_region.unwrap())
        } else {
            None
        }
    }

    pub fn list_descendants_with_display_region(
        children: &Vec<Rc<ChildWithRegion>>,
    ) -> Vec<Rc<ChildWithRegion>> {
        let mut all_descendants: Vec<Rc<ChildWithRegion>> = Vec::new();

        for child_with_region in children.iter() {
            all_descendants.push(Rc::clone(child_with_region));

            // Recurse to get the descendants of the current child
            let descendants = DisplayRegionUtils::list_descendants_with_display_region(
                &child_with_region.node.child_with_region,
            );
            all_descendants.extend(descendants);
        }

        all_descendants
    }

    pub fn list_children_with_display_region(
        &self,
        children_of_node: &Vec<Rc<dyn ChildOfNodeWithDisplayRegion>>,
    ) -> Vec<Rc<ChildWithRegion>> {
        children_of_node
            .iter()
            .filter_map(|child| DisplayRegionUtils::just_case_with_display_region(Rc::clone(child)))
            .collect()
    }

    pub fn create_display_region_node_with_offset(
        inherited_offset: (i32, i32),
        occluded_regions: &mut Vec<Rc<DisplayRegion>>,
        raw_node: &Rc<UiTreeNode>,
    ) -> Rc<dyn ChildOfNodeWithDisplayRegion> {
        if let Some(self_region) = DisplayRegionUtils::create_display_region_from_ui_node(&raw_node)
        {
            let total_display_region = Rc::new(DisplayRegion {
                x: self_region.x + inherited_offset.0,
                y: self_region.y + inherited_offset.1,
                width: self_region.width,
                height: self_region.height,
            });

            let tree_node_with_display_region =
                DisplayRegionUtils::parse_child_of_node_with_display_region(
                    raw_node,
                    &Rc::new(self_region),
                    &total_display_region,
                    occluded_regions,
                );
            let child_of_node = Rc::new(ChildWithRegion {
                node: Rc::new(tree_node_with_display_region),
            });
            child_of_node
        } else {
            let child_of_node = Rc::new(ChildWithoutRegion {
                node: Rc::clone(raw_node),
            });
            child_of_node
        }
    }

    pub fn node_occludes_following_nodes(node: &UITreeNodeWithDisplayRegion) -> bool {
        let known_occluding_types = [
            "SortHeaders",
            "ContextMenu",
            "OverviewWindow",
            "DronesWindow",
            "SelectedItemWnd",
            "InventoryPrimary",
            "ChatWindowStack",
        ];

        known_occluding_types.contains(&node.ui_node.object_type_name.as_str())
    }

    pub fn get_display_region_from_dict_entries(
        entries_of_interest: &HashMap<String, Rc<Box<dyn Any>>>,
    ) -> Option<DisplayRegion> {
        let display_x = UiUtils::fixed_number_from_property_name("_displayX", entries_of_interest);
        let display_y = UiUtils::fixed_number_from_property_name("_displayY", entries_of_interest);
        let display_width =
            UiUtils::fixed_number_from_property_name("_displayWidth", entries_of_interest);
        let display_height =
            UiUtils::fixed_number_from_property_name("_displayHeight", entries_of_interest);

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

    pub fn create_display_region_from_ui_node(ui_node: &Rc<UiTreeNode>) -> Option<DisplayRegion> {
        let display_x = UiUtils::fixed_number_from_ui_node("_displayX", ui_node);
        let display_y = UiUtils::fixed_number_from_ui_node("_displayY", ui_node);
        let display_width = UiUtils::fixed_number_from_ui_node("_displayWidth", ui_node);
        let display_height = UiUtils::fixed_number_from_ui_node("_displayHeight", ui_node);

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
}
