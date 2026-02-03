//! Window management

pub struct Window {
    pub id: u64,
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
    pub title: [u8; 64],
}

impl Window {
    pub fn new(id: u64) -> Self {
        Self {
            id,
            x: 0,
            y: 0,
            width: 640,
            height: 480,
            title: [0; 64],
        }
    }
}
