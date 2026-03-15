



use alloc::string::String;
use alloc::vec::Vec;


#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Cgh {
    Cki = 0,
    Alg = 1,
}


#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ButtonState {
    Cki = 0,
    Alg = 1,
}


#[derive(Debug, Clone, Copy, Default)]
pub struct ModifierState {
    pub nkk: u32,
    pub czf: u32,
    pub caq: u32,
    pub cyi: u32,
}


pub struct Cmr {
    
    pub j: String,
    
    
    pub bme: u32,
    
    
    pub vjr: PointerState,
    
    
    pub keyboard: KeyboardState,
    
    
    pub eqq: Option<u32>,
    
    
    serial: u32,
}

impl Cmr {
    pub fn new(j: &str) -> Self {
        Self {
            j: String::from(j),
            bme: 0b11, 
            vjr: PointerState::new(),
            keyboard: KeyboardState::new(),
            eqq: None,
            serial: 1,
        }
    }
    
    
    pub fn zdq(&mut self) -> u32 {
        let e = self.serial;
        self.serial = self.serial.cn(1);
        e
    }
    
    
    pub fn znh(&mut self, cmz: Option<u32>) {
        self.eqq = cmz;
    }
    
    
    pub fn ywl(&self) -> bool {
        self.bme & 1 != 0
    }
    
    
    pub fn oar(&self) -> bool {
        self.bme & 2 != 0
    }
    
    
    pub fn ywn(&self) -> bool {
        self.bme & 4 != 0
    }
}


pub struct PointerState {
    
    pub b: f64,
    
    
    pub c: f64,
    
    
    pub arc: Option<u32>,
    
    
    pub pqb: f64,
    pub pqc: f64,
    
    
    pub cjk: u32,
    
    
    pub nip: Option<u32>,
    pub nim: i32,
    pub nin: i32,
}

impl PointerState {
    pub fn new() -> Self {
        Self {
            b: 0.0,
            c: 0.0,
            arc: None,
            pqb: 0.0,
            pqc: 0.0,
            cjk: 0,
            nip: None,
            nim: 0,
            nin: 0,
        }
    }
    
    
    pub fn hsa(&mut self, b: f64, c: f64) {
        self.b = b;
        self.c = c;
    }
    
    
    pub fn zmo(&mut self, bdp: u32, vn: bool) {
        if vn {
            self.cjk |= 1 << bdp;
        } else {
            self.cjk &= !(1 << bdp);
        }
    }
    
    
    pub fn yzd(&self, bdp: u32) -> bool {
        self.cjk & (1 << bdp) != 0
    }
    
    
    pub fn bld(&mut self, surface: Option<u32>, tpz: i32, tqa: i32) {
        self.nip = surface;
        self.nim = tpz;
        self.nin = tqa;
    }
}

impl Default for PointerState {
    fn default() -> Self {
        Self::new()
    }
}


pub struct KeyboardState {
    
    pub gps: Vec<u32>,
    
    
    pub modifiers: ModifierState,
    
    
    pub vxe: i32,
    
    
    pub vxd: i32,
    
    
    pub arc: Option<u32>,
}

impl KeyboardState {
    pub fn new() -> Self {
        Self {
            gps: Vec::new(),
            modifiers: ModifierState::default(),
            vxe: 25,
            vxd: 400,
            arc: None,
        }
    }
    
    
    pub fn zai(&mut self, bs: u32) {
        if !self.gps.contains(&bs) {
            self.gps.push(bs);
        }
    }
    
    
    pub fn zaj(&mut self, bs: u32) {
        self.gps.ajm(|&eh| eh != bs);
    }
    
    
    pub fn alh(&self, bs: u32) -> bool {
        self.gps.contains(&bs)
    }
    
    
    pub fn znl(&mut self, nkk: u32, czf: u32, caq: u32, cyi: u32) {
        self.modifiers = ModifierState {
            nkk,
            czf,
            caq,
            cyi,
        };
    }
    
    
    pub fn clear(&mut self) {
        self.gps.clear();
    }
}

impl Default for KeyboardState {
    fn default() -> Self {
        Self::new()
    }
}






#[derive(Debug, Clone)]
pub struct Deo {
    pub time: u32,
    pub pqb: f64,
    pub pqc: f64,
}


#[derive(Debug, Clone)]
pub struct Den {
    pub serial: u32,
    pub time: u32,
    pub bdp: u32,
    pub g: ButtonState,
}


#[derive(Debug, Clone)]
pub struct Dem {
    pub time: u32,
    pub gao: u32, 
    pub bn: f64,
}


#[derive(Debug, Clone)]
pub struct Dav {
    pub serial: u32,
    pub time: u32,
    pub bs: u32,
    pub g: Cgh,
}


#[derive(Debug, Clone)]
pub struct Daw {
    pub serial: u32,
    pub zcx: u32,
    pub zcy: u32,
    pub zcz: u32,
    pub cyi: u32,
}
