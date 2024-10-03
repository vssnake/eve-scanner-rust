use std::collections::HashSet;
use std::rc::Rc;
use crate::eve::ui::models::overview_window::{OverviewWindow, OverviewWindowEntry, OverviewWindowEntryCommonIndications};
use crate::eve::ui_tree_node::models::ui_tree_node::{UITreeNodeWithDisplayRegion, UiTreeNode};
use crate::eve::ui_tree_node::utils::display_region_utils::DisplayRegionUtils;
use crate::eve::ui::parser_utils::ParserUtils;

impl OverviewWindow {
    pub fn parse(overview_window_node: Rc<UITreeNodeWithDisplayRegion>) -> OverviewWindow {
        let descendant_display_region =
            DisplayRegionUtils::list_descendants_with_display_region(&overview_window_node.child_with_region);

        let scroll_node = descendant_display_region.iter().find(|node| {
            node.node.ui_node.object_type_name
                .to_lowercase()
                .contains("basicdynamicscroll")
        });

        let scroll_controllers = scroll_node
            .map(|node| ParserUtils::parse_scroll_controls(&node.node));

        let headers_container_node = descendant_display_region.iter().find(|node| {
            node.node.ui_node.object_type_name
                .to_lowercase()
                .contains("headers")
        });

        let entries_headers = headers_container_node
            .map(|node| ParserUtils::get_all_contained_display_texts_with_region(Some(Rc::clone(&node.node))))
            .unwrap_or_default();

        let children_with_region =
            DisplayRegionUtils::list_descendants_with_display_region(&overview_window_node.child_with_region);

        let mut entries = Vec::new();

        for child_with_region in children_with_region {
            if child_with_region.node.ui_node.object_type_name != "OverviewScrollEntry" {
                continue;
            }
            let parsed_entry =
                OverviewWindow::parse_overview_window_entry(&entries_headers, Rc::clone(&child_with_region.node));
            entries.push(parsed_entry);
        }

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
        let mut all_text_with_regions = ParserUtils::get_all_contained_display_texts_with_region(
            Some(Rc::clone(&overview_window_entry_node)),
        );

        all_text_with_regions.sort_by(|a, b| {
            a.1.total_display_region.x
                .partial_cmp(&b.1.total_display_region.x)
                .unwrap()
        });

        let texts_left_to_right: Vec<String> = all_text_with_regions
            .into_iter()
            .map(|(text, _)| text)
            .collect();

        let list_view_entry = ParserUtils::parse_list_view_entry(entry_headers,
                                                                 Rc::clone(&overview_window_entry_node));
        
        let test = t!("distance").as_ref();
        //TODO locale Distance
        let object_distance = list_view_entry.get(t!("distance").as_ref()).map(|s| s.to_string());
        let object_distance_in_meters =
            ParserUtils::parse_overview_entry_distance_in_meters_from_text(&object_distance);

        let list_descendants_with_display_region =
            DisplayRegionUtils::list_descendants_with_display_region(&overview_window_entry_node.child_with_region);

        let space_object_icon_node = list_descendants_with_display_region
            .iter()
            .find(|child_with_region| {
                child_with_region.node.ui_node.object_type_name
                    .eq_ignore_ascii_case("spaceObjectIcon")
            });
        
        let object_name = list_view_entry.get(t!("name").as_ref()).map(|s| s.to_string());
        let object_type = list_view_entry.get(t!("type").as_ref()).map(|s| s.to_string());
        let object_alliance = list_view_entry.get(t!("alliance").as_ref()).map(|s| s.to_string());
        
        if  space_object_icon_node.is_none() {
            return OverviewWindowEntry {
                ui_node: Rc::clone(&overview_window_entry_node),
                texts_left_to_right,
                cells_texts: list_view_entry,
                object_distance,
                object_distance_in_meters,
                object_name,
                object_type,
                object_alliance,
                is_player: false,
                icon_sprite_color_percent: None,
                names_under_space_object_icon: HashSet::new(),
                bg_color_fills_percent: Vec::new(),
                right_aligned_icons_hints: Vec::new(),
                common_indications: OverviewWindowEntryCommonIndications {
                    targeting: false,
                    targeted_by_me: false,
                    is_jamming_me: false,
                    is_warp_disrupting_me: false,
                },
                opacity_percent: None,
            };
        }

        let space_object_icon_descendants =
            DisplayRegionUtils::list_descendants_with_display_region(&space_object_icon_node.unwrap().node.child_with_region);

        let mut icon_sprite = None;

        for child_with_region in &space_object_icon_descendants {
            let name = ParserUtils::get_name_from_dict_entries(&child_with_region.node.ui_node);
            if let Some(name_str) = name.as_deref() {
                if name_str == "iconSprite" {
                    icon_sprite = Some(child_with_region.clone());
                    break;
                }
            }
        }

        let is_player = space_object_icon_descendants
            .iter()
            .any(|child_with_region| {
                child_with_region.node.ui_node.object_type_name == "FlagIconWithState"
            });
        
        
        
        let icon_sprite_color_percent = icon_sprite
            .map(|icon| ParserUtils::get_color_percentage_from_dict_entries(&icon.node.ui_node))
            .flatten();

        let names_under_space_object_icon: HashSet<String> =
            UiTreeNode::list_descendants_in_ui_tree_node(&space_object_icon_node.unwrap().node.ui_node)
                .into_iter()
                .filter_map(|descendant| ParserUtils::get_name_from_dict_entries(&descendant))
                .collect();

        let mut bg_color_fills_percent = Vec::new();
        let descendants_with_region =
            DisplayRegionUtils::list_descendants_with_display_region(&overview_window_entry_node.child_with_region);

        for child_with_region in descendants_with_region {
            let name = ParserUtils::get_name_from_dict_entries(&child_with_region.node.ui_node);
            let object_type = &child_with_region.node.ui_node.object_type_name;


            if let Some(name_str) = name.as_deref() {
                if name_str == "bgColor" && object_type == "Fill" {
                    if let Some(color) =
                        ParserUtils::get_color_percentage_from_dict_entries(&child_with_region.node.ui_node)
                    {
                        bg_color_fills_percent.push(color);
                    }
                }
            }
        }

        let mut right_aligned_icons_hints = Vec::new();
        let initial_descendants =
            DisplayRegionUtils::list_descendants_with_display_region(&overview_window_entry_node.child_with_region);

        for child_with_region in initial_descendants {

            if let Some(name_str) = ParserUtils::get_name_from_dict_entries(&child_with_region.node.ui_node).as_deref() {
                if name_str == "rightAlignedIconContainer" {
                    let inner_descendants =
                        DisplayRegionUtils::list_descendants_with_display_region(&child_with_region.node.child_with_region);

                    for inner_child_with_region in inner_descendants {
                        if let Some(hint) =
                            ParserUtils::get_hint_text_from_dict_entries(&inner_child_with_region.node.ui_node)
                        {
                            if !hint.is_empty() {
                                right_aligned_icons_hints.push(hint);
                            }
                        }
                    }
                }
            }
        }

        let is_targeting = names_under_space_object_icon.iter().any(|name| name.as_str() == "targeting");
        let is_targeted_by_me = names_under_space_object_icon.iter().any(|name| name.as_str() == "targetedByMeIndicator");
        let common_indicator = OverviewWindowEntryCommonIndications {
            targeting: is_targeting,
            targeted_by_me: is_targeted_by_me,
            is_jamming_me: OverviewWindow::right_aligned_icons_hints_contains_text_ignoring_case(
                &right_aligned_icons_hints,
                t!("is_jamming_me").as_ref(),
            ),
            is_warp_disrupting_me: OverviewWindow::right_aligned_icons_hints_contains_text_ignoring_case(
                &right_aligned_icons_hints,
                t!("is_warp_disrupting_me").as_ref(),
            ),
        };

        let opacity_percent =
            ParserUtils::get_opacity_from_dict_entries(&overview_window_entry_node.ui_node);
        
        
        OverviewWindowEntry {
            ui_node: Rc::clone(&overview_window_entry_node),
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
            common_indications: common_indicator,
            opacity_percent,
        }
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