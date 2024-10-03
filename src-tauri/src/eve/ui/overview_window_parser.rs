use crate::eve::ui::models::overview_window::{
    OverviewWindow, OverviewWindowEntry, OverviewWindowEntryCommonIndications,
};
use crate::eve::ui::parser_utils::ParserUtils;
use crate::eve::ui_tree_node::models::ui_tree_node::{UITreeNodeWithDisplayRegion, UiTreeNode};
use crate::eve::ui_tree_node::utils::display_region_utils::DisplayRegionUtils;
use std::collections::HashSet;
use std::rc::Rc;
use crate::eve::ui_tree_node::common::common::ColorComponents;
use crate::eve::ui_tree_node::models::child_of_node::ChildWithRegion;

impl OverviewWindow {
    pub fn parse(overview_window_node: Rc<UITreeNodeWithDisplayRegion>) -> OverviewWindow {
        let descendant_display_region = DisplayRegionUtils::list_descendants_with_display_region(
            &overview_window_node.child_with_region,
        );

        let scroll_node = descendant_display_region.iter().find(|node| {
            node.node
                .ui_node
                .object_type_name
                .to_lowercase()
                .contains("basicdynamicscroll")
        });

        let scroll_controllers =
            scroll_node.map(|node| ParserUtils::parse_scroll_controls(&node.node));

        let headers_container_node = descendant_display_region.iter().find(|node| {
            node.node
                .ui_node
                .object_type_name
                .to_lowercase()
                .contains("headers")
        });

        let entries_headers = headers_container_node
            .map(|node| ParserUtils::get_all_contained_display_texts_with_region(&node.node))
            .unwrap_or_default();

        let children_with_region = DisplayRegionUtils::list_descendants_with_display_region(
            &overview_window_node.child_with_region,
        );

        let entries: Vec<_> = children_with_region
            .iter()
            .filter(|child| child.node.ui_node.object_type_name == "OverviewScrollEntry")
            .map(|child_with_region| {
                OverviewWindow::parse_overview_window_entry(
                    &entries_headers,
                    Rc::clone(&child_with_region.node),
                )
            })
            .collect();

        OverviewWindow {
            ui_node: overview_window_node,
            entries_headers,
            entries,
            scroll_controls: scroll_controllers,
        }
    }

    pub fn parse_overview_window_entry(
        entry_headers: &Vec<(String, Rc<UITreeNodeWithDisplayRegion>)>,
        overview_window_entry_node: Rc<UITreeNodeWithDisplayRegion>,
    ) -> OverviewWindowEntry {
        

        let list_view_entry = ParserUtils::parse_list_view_entry(
            entry_headers,
            Rc::clone(&overview_window_entry_node),
        );
        let texts_left_to_right = Self::extract_text_left_to_right(&overview_window_entry_node);
        
        let object_distance = list_view_entry
            .get(t!("distance").as_ref())
            .map(|s| s.to_string());
        let object_distance_in_meters =
            ParserUtils::parse_overview_entry_distance_in_meters_from_text(&object_distance);
        
        let object_name = list_view_entry
            .get(t!("name").as_ref())
            .map(|s| s.to_string());
        
        let object_type = list_view_entry
            .get(t!("type").as_ref())
            .map(|s| s.to_string());
        
        let object_alliance = list_view_entry
            .get(t!("alliance").as_ref())
            .map(|s| s.to_string());

        let bg_color_fills_percent = Self::get_bg_color_fills_percent(&overview_window_entry_node);

        let right_aligned_icons_hints = Self::get_right_aligned_icons_hints(&overview_window_entry_node);
        

        let opacity_percent =
            ParserUtils::get_opacity_from_dict_entries(&overview_window_entry_node.ui_node);

        let list_descendants_with_display_region =
            DisplayRegionUtils::list_descendants_with_display_region(
                &overview_window_entry_node.child_with_region,
            );
        
        let space_object_icon_node =
            list_descendants_with_display_region
                .iter()
                .find(|child_with_region| {
                    child_with_region
                        .node
                        .ui_node
                        .object_type_name
                        .eq_ignore_ascii_case("spaceObjectIcon")
                });

        let space_object_icon_descendants = space_object_icon_node.as_ref().map(|node| {
            DisplayRegionUtils::list_descendants_with_display_region(&node.node.child_with_region)
        });

        let icon_sprite_color_percent = Self::get_icon_sprite_color_perfect(&space_object_icon_descendants);

        let is_player = Self::is_player(space_object_icon_descendants);

        let names_under_space_object_icon: HashSet<String> =
            UiTreeNode::list_descendants_in_ui_tree_node(
                &space_object_icon_node.unwrap().node.ui_node,
            )
            .into_iter()
            .filter_map(|descendant| ParserUtils::get_name_from_dict_entries(&descendant))
            .collect();
        
        let common_indications = Self::extract_common_indicators(&names_under_space_object_icon, &right_aligned_icons_hints);
        

        OverviewWindowEntry {
            ui_node: overview_window_entry_node,
            texts_left_to_right,
            cells_texts: list_view_entry,
            object_distance,
            object_distance_in_meters,
            object_name,
            object_type,
            object_alliance,
            is_player,
            icon_sprite_color_percent,
            names_under_space_object_icon,
            bg_color_fills_percent,
            right_aligned_icons_hints,
            common_indications,
            opacity_percent,
        }
    }

    fn get_bg_color_fills_percent(overview_window_entry_node: &Rc<UITreeNodeWithDisplayRegion>) -> Vec<ColorComponents> {
        let bg_color_fills_percent: Vec<_> = DisplayRegionUtils::list_descendants_with_display_region(
            &overview_window_entry_node.child_with_region,
        )
            .iter()
            .filter_map(|child_with_region| {
                let binding = ParserUtils::get_name_from_dict_entries(&child_with_region.node.ui_node);
                let name = binding.as_deref();
                let object_type = &child_with_region.node.ui_node.object_type_name;

                if name == Some("bgColor") && object_type == "Fill" {
                    ParserUtils::get_color_percentage_from_dict_entries(&child_with_region.node.ui_node)
                } else {
                    None
                }
            })
            .collect();
        bg_color_fills_percent
    }

    fn get_right_aligned_icons_hints(overview_window_entry_node: &Rc<UITreeNodeWithDisplayRegion>) -> Vec<String> {
        let right_aligned_icons_hints: Vec<_> = DisplayRegionUtils::list_descendants_with_display_region(
            &overview_window_entry_node.child_with_region,
        )
            .iter()
            .filter_map(|child_with_region| {
                let binding = ParserUtils::get_name_from_dict_entries(&child_with_region.node.ui_node);
                let name_str = binding.as_deref();
                if name_str == Some("rightAlignedIconContainer") {
                    Some(DisplayRegionUtils::list_descendants_with_display_region(
                        &child_with_region.node.child_with_region,
                    ))
                } else {
                    None
                }
            })
            .flat_map(|inner_descendants| inner_descendants)
            .filter_map(|inner_child_with_region| {
                let hint = ParserUtils::get_hint_text_from_dict_entries(&inner_child_with_region.node.ui_node);
                if hint.is_some() && !hint.as_ref().unwrap().is_empty() {
                    hint
                } else {
                    None
                }
            })
            .collect();
        right_aligned_icons_hints
    }

    fn get_icon_sprite_color_perfect(space_object_icon_descendants: &Option<Vec<Rc<ChildWithRegion>>>) -> Option<ColorComponents> {
        let icon_sprite_color_percent = space_object_icon_descendants
            .as_ref() // Nos aseguramos de trabajar con una referencia opcional.
            .and_then(|descendants| {
                descendants.iter().find_map(|child_with_region| {
                    let binding = ParserUtils::get_name_from_dict_entries(&child_with_region.node.ui_node);
                    let name = binding.as_deref();
                    if name == Some("iconSprite") {
                        Some(child_with_region) // Encontramos el iconSprite
                    } else {
                        None
                    }
                })
            })
            .and_then(|icon| ParserUtils::get_color_percentage_from_dict_entries(&icon.node.ui_node));
        icon_sprite_color_percent
    }

    fn is_player(space_object_icon_descendants: Option<Vec<Rc<ChildWithRegion>>>) -> bool {
        if space_object_icon_descendants.is_none() {
            return false;
        }
        let is_player = space_object_icon_descendants.unwrap()
            .iter()
            .any(|child_with_region| {
                child_with_region.node.ui_node.object_type_name == "FlagIconWithState"
            });
        is_player
    }

    fn extract_common_indicators(names_under_space_object_icon: &HashSet<String>, right_aligned_icons_hints: &Vec<String>) -> OverviewWindowEntryCommonIndications {
        let is_targeting = names_under_space_object_icon
            .iter()
            .any(|name| name.as_str() == "targeting");
        let is_targeted_by_me = names_under_space_object_icon
            .iter()
            .any(|name| name.as_str() == "targetedByMeIndicator");
        let common_indicator = OverviewWindowEntryCommonIndications {
            targeting: is_targeting,
            targeted_by_me: is_targeted_by_me,
            is_jamming_me: OverviewWindow::right_aligned_icons_hints_contains_text_ignoring_case(
                &right_aligned_icons_hints,
                t!("is_jamming_me").as_ref(),
            ),
            is_warp_disrupting_me:
            OverviewWindow::right_aligned_icons_hints_contains_text_ignoring_case(
                &right_aligned_icons_hints,
                t!("is_warp_disrupting_me").as_ref(),
            ),
        };
        common_indicator
    }

    fn extract_text_left_to_right(
        overview_window_entry_node: &Rc<UITreeNodeWithDisplayRegion>,
    ) -> Vec<String> {
        let mut all_text_with_regions =
            ParserUtils::get_all_contained_display_texts_with_region(&overview_window_entry_node);

        all_text_with_regions.sort_by(|a, b| {
            a.1.total_display_region
                .x
                .partial_cmp(&b.1.total_display_region.x)
                .unwrap()
        });

        let texts_left_to_right: Vec<String> = all_text_with_regions
            .into_iter()
            .map(|(text, _)| text)
            .collect();
        texts_left_to_right
    }

    fn right_aligned_icons_hints_contains_text_ignoring_case(
        hints: &Vec<String>,
        text_to_search: &str,
    ) -> bool {
        let search_text_lower = text_to_search.to_lowercase();
        hints
            .iter()
            .any(|hint| hint.to_lowercase().contains(&search_text_lower))
    }
}
