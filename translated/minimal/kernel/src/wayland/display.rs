



use alloc::vec::Vec;
use alloc::string::String;
use super::protocol::{Op, Afu, kys};


pub struct Display {
    
    pub nzc: Vec<Op>,
    
    
    pub ltm: Vec<Afu>,
    
    
    pub jct: u32,
}

impl Display {
    pub fn new() -> Self {
        Self {
            nzc: kys(),
            ltm: Vec::new(),
            jct: 0,
        }
    }
    
    
    pub fn sync(&mut self, qvp: u32) {
        
        self.jct = qvp;
    }
    
    
    pub fn ytq(&self) -> &[Op] {
        &self.nzc
    }
    
    
    pub fn zhd(&mut self, id: Afu) {
        self.ltm.push(id);
    }
    
    
    pub fn hjx(&mut self) -> Vec<Afu> {
        core::mem::take(&mut self.ltm)
    }
}

impl Default for Display {
    fn default() -> Self {
        Self::new()
    }
}


#[derive(Debug, Clone)]
pub struct Dd {
    pub ad: u32,
    pub j: String,
    pub ujk: String,
    pub model: String,
    pub b: i32,
    pub c: i32,
    pub vho: i32,  
    pub vhn: i32, 
    pub wvw: Subpixel,
    pub xlt: Transform,
    pub bv: i32,
    pub gmv: Vec<Awm>,
    pub eog: usize,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Subpixel {
    F = 0,
    None = 1,
    Cyl = 2,
    Cyk = 3,
    Dlm = 4,
    Dll = 5,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Transform {
    M = 0,
    Ckw = 1,
    Cku = 2,
    Ckv = 3,
    Cdr = 4,
    Cdu = 5,
    Cds = 6,
    Cdt = 7,
}

#[derive(Debug, Clone)]
pub struct Awm {
    pub z: i32,
    pub ac: i32,
    pub gqr: i32, 
    pub flags: u32,   
}

impl Dd {
    pub fn new(ad: u32, z: u32, ac: u32) -> Self {
        Self {
            ad,
            j: String::from("TrustOS-1"),
            ujk: String::from("TrustOS"),
            model: String::from("Virtual Display"),
            b: 0,
            c: 0,
            vho: (z * 254 / 96) as i32, 
            vhn: (ac * 254 / 96) as i32,
            wvw: Subpixel::F,
            xlt: Transform::M,
            bv: 1,
            gmv: alloc::vec![
                Awm {
                    z: z as i32,
                    ac: ac as i32,
                    gqr: 60000, 
                    flags: 3, 
                }
            ],
            eog: 0,
        }
    }
    
    pub fn eog(&self) -> &Awm {
        &self.gmv[self.eog]
    }
}
