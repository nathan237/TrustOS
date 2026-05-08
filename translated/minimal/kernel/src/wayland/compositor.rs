



use super::surface::Surface;
use super::gjd;
use alloc::vec::Vec;


#[derive(Debug, Clone)]
pub struct Aom {
    pub id: u32,
    pub rects: Vec<Ur>,
}

#[derive(Debug, Clone, Copy)]
pub struct Ur {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
    pub add: bool, 
}

impl Aom {
    pub fn new(id: u32) -> Self {
        Self {
            id,
            rects: Vec::new(),
        }
    }
    
    pub fn add(&mut self, x: i32, y: i32, width: i32, height: i32) {
        self.rects.push(Ur { x, y, width, height, add: true });
    }
    
    pub fn qxv(&mut self, x: i32, y: i32, width: i32, height: i32) {
        self.rects.push(Ur { x, y, width, height, add: false });
    }
    
    
    pub fn contains(&self, p: i32, o: i32) -> bool {
        let mut bmz = false;
        
        for rect in &self.rects {
            let igd = p >= rect.x 
                && p < rect.x + rect.width 
                && o >= rect.y 
                && o < rect.y + rect.height;
            
            if rect.add && igd {
                bmz = true;
            } else if !rect.add && igd {
                bmz = false;
            }
        }
        
        bmz
    }
}


pub trait Atp {
    
    fn create_surface(&mut self) -> u32;
    
    
    fn qbw(&mut self) -> u32;
}
