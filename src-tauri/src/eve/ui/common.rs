pub mod common {
    use std::any::Any;
    use std::cell::RefCell;
    use std::collections::HashMap;
    use std::rc::Rc;
    use crate::eve::ui::ui_constants::{UiConstants, UiZonesEnum, UI_ZONES};
    use crate::process::interop::ui::ui_tree_node::UiTreeNode;

    #[derive(Debug, Clone)]
    pub struct ColorComponents {
        pub alpha: i32,
        pub red: i32,
        pub green: i32,
        pub blue: i32,
    }

    #[derive(Debug, Clone)]
    pub struct DisplayRegion {
        pub x: i32,
        pub y: i32,
        pub width: i32,
        pub height: i32,
    }

    impl DisplayRegion {
        pub fn new(x: i32, y: i32, width: i32, height: i32) -> DisplayRegion {
            DisplayRegion {
                x,
                y,
                width,
                height,
            }
        }

        pub fn right(&self) -> i32 {
            self.x + self.width
        }

        pub fn bottom(&self) -> i32 {
            self.y + self.height
        }
    }


    pub trait ChildOfNodeWithDisplayRegion<'a> {
        fn has_region(&self) -> bool;
        fn as_any(&'a self) -> &dyn Any;
    }

    pub struct ChildWithRegion<'a> {
        pub node: UITreeNodeWithDisplayRegion<'a>,
    }

    impl<'a> ChildOfNodeWithDisplayRegion<'a> for ChildWithRegion<'a> {
        fn has_region(&self) -> bool {
            true
        }
        fn as_any(&self) -> &dyn Any {
            self
        }
    }

    pub struct ChildWithoutRegion<'a> {
        pub node: &'a UiTreeNode,
    }

    impl<'a> ChildOfNodeWithDisplayRegion<'a> for ChildWithoutRegion<'a> {
        fn has_region(&self) -> bool {
            false
        }
        fn as_any(&self) -> &dyn Any {
            self
        }
    }

    pub struct UITreeNodeWithDisplayRegion<'a> {
        pub ui_node: &'a UiTreeNode,
        pub children: Option<Vec<Rc<RefCell<dyn ChildOfNodeWithDisplayRegion<'a> +'a>>>>,
        pub self_display_region: DisplayRegion,
        pub total_display_region: DisplayRegion,
        pub total_display_region_visible: DisplayRegion,
    }

    impl<'a> UITreeNodeWithDisplayRegion<'a> {

        pub fn search_and_add_ui_zone_in_node(&'a self, ui_zones: &mut HashMap<&'a UiZonesEnum, Vec<&'a UITreeNodeWithDisplayRegion<'a>>>) {
            if let Some(zone) = UI_ZONES.get(&self.ui_node.object_type_name.as_str()) {
                ui_zones.entry(zone).or_insert_with(Vec::new).push(self);
            }
        }
    }

    pub struct ScrollControls<'a> {
        pub ui_node: &'a UITreeNodeWithDisplayRegion<'a>,
        pub scroll_handle: Option<&'a UITreeNodeWithDisplayRegion<'a>>,
    }
}
