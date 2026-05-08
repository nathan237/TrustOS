



use super::{Rect, Size};


#[derive(Clone, Copy, Debug)]
pub struct Aib {
    pub min_width: f32,
    pub max_width: f32,
    pub min_height: f32,
    pub max_height: f32,
}

impl Aib {
    pub fn new(min_w: f32, max_w: f32, min_h: f32, dua: f32) -> Self {
        Self {
            min_width: min_w,
            max_width: max_w,
            min_height: min_h,
            max_height: dua,
        }
    }
    
    pub fn rai(size: Size) -> Self {
        Self {
            min_width: size.width,
            max_width: size.width,
            min_height: size.height,
            max_height: size.height,
        }
    }
    
    pub fn qog(size: Size) -> Self {
        Self {
            min_width: 0.0,
            max_width: size.width,
            min_height: 0.0,
            max_height: size.height,
        }
    }
    
    pub fn rbe() -> Self {
        Self {
            min_width: 0.0,
            max_width: f32::INFINITY,
            min_height: 0.0,
            max_height: f32::INFINITY,
        }
    }
    
    pub fn qbd(&self, size: Size) -> Size {
        Size::new(
            size.width.max(self.min_width).min(self.max_width),
            size.height.max(self.min_height).min(self.max_height),
        )
    }
}


#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Ast {
    Start,
    Center,
    End,
    Stretch,
}


#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Azm {
    Start,
    Center,
    End,
    SpaceBetween,
    SpaceAround,
    SpaceEvenly,
}


#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Atr {
    Start,
    Center,
    End,
    Stretch,
}


#[derive(Clone, Copy, Debug, Default)]
pub struct EdgeInsets {
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
    pub left: f32,
}

impl EdgeInsets {
    pub const fn all(value: f32) -> Self {
        Self {
            top: value,
            right: value,
            bottom: value,
            left: value,
        }
    }
    
    pub const fn ozm(vertical: f32, horizontal: f32) -> Self {
        Self {
            top: vertical,
            bottom: vertical,
            left: horizontal,
            right: horizontal,
        }
    }
    
    pub const fn nnf(top: f32, right: f32, bottom: f32, left: f32) -> Self {
        Self { top, right, bottom, left }
    }
    
    pub fn horizontal(&self) -> f32 {
        self.left + self.right
    }
    
    pub fn vertical(&self) -> f32 {
        self.top + self.bottom
    }
    
    pub fn qcm(&self, rect: Rect) -> Rect {
        Rect::new(
            rect.x + self.left,
            rect.y + self.top,
            rect.width - self.horizontal(),
            rect.height - self.vertical(),
        )
    }
    
    pub fn inflate(&self, rect: Rect) -> Rect {
        Rect::new(
            rect.x - self.left,
            rect.y - self.top,
            rect.width + self.horizontal(),
            rect.height + self.vertical(),
        )
    }
}
