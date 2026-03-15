



use super::surface::Surface;
use super::lny;
use alloc::vec::Vec;


#[derive(Debug, Clone)]
pub struct Ckh {
    pub ad: u32,
    pub akn: Vec<Axt>,
}

#[derive(Debug, Clone, Copy)]
pub struct Axt {
    pub b: i32,
    pub c: i32,
    pub z: i32,
    pub ac: i32,
    pub add: bool, 
}

impl Ckh {
    pub fn new(ad: u32) -> Self {
        Self {
            ad,
            akn: Vec::new(),
        }
    }
    
    pub fn add(&mut self, b: i32, c: i32, z: i32, ac: i32) {
        self.akn.push(Axt { b, c, z, ac, add: true });
    }
    
    pub fn zpv(&mut self, b: i32, c: i32, z: i32, ac: i32) {
        self.akn.push(Axt { b, c, z, ac, add: false });
    }
    
    
    pub fn contains(&self, y: i32, x: i32) -> bool {
        let mut dsa = false;
        
        for ha in &self.akn {
            let odu = y >= ha.b 
                && y < ha.b + ha.z 
                && x >= ha.c 
                && x < ha.c + ha.ac;
            
            if ha.add && odu {
                dsa = true;
            } else if !ha.add && odu {
                dsa = false;
            }
        }
        
        dsa
    }
}


pub trait Ctk {
    
    fn fgc(&mut self) -> u32;
    
    
    fn ykp(&mut self) -> u32;
}
