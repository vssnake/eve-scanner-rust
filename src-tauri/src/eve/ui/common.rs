pub mod common {
    use crate::process::interop::ui::ui_tree_node::{UITreeNodeWithDisplayRegion, UiTreeNode};
    use std::any::Any;
    use std::rc::Rc;

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


    pub trait ChildOfNodeWithDisplayRegion {
        fn has_region(&self) -> bool;
        fn as_any(& self) -> &dyn Any;
        fn as_any_rc(self: Rc<Self>) -> Rc<dyn Any>;
    }

    pub struct ChildWithRegion {
        pub node: UITreeNodeWithDisplayRegion,
    }

    pub struct ChildWithoutRegion {
        pub node: Rc<UiTreeNode>,
    }

    impl ChildOfNodeWithDisplayRegion for ChildWithRegion {
        fn has_region(&self) -> bool {
            true
        }
        fn as_any(&self) -> &dyn Any {
            self
        }

        fn as_any_rc(self: Rc<Self>) -> Rc<dyn Any> {
            self
        }
    }

    impl ChildOfNodeWithDisplayRegion for ChildWithoutRegion {
        fn has_region(&self) -> bool {
            false
        }
        fn as_any(&self) -> &dyn Any {
            self
        }

        fn as_any_rc(self: Rc<Self>) -> Rc<dyn Any> {
            self
        }
    }
    
}
