use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
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