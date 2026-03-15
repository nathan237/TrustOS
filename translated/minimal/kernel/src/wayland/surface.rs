




use alloc::string::String;
use alloc::vec::Vec;


#[derive(Debug, Clone, Copy)]
pub struct Ahh {
    pub b: i32,
    pub c: i32,
    pub z: i32,
    pub ac: i32,
}


#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum BufferTransform {
    #[default]
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
pub struct Surface {
    
    pub ad: u32,
    
    
    pub b: i32,
    pub c: i32,
    
    
    pub z: u32,
    pub ac: u32,
    
    
    pub bi: Vec<u32>,
    
    
    pub ltl: Option<Vec<u32>>,
    pub lto: u32,
    pub ltn: u32,
    
    
    pub qst: i32,
    pub qsu: i32,
    
    
    pub qsv: i32,
    
    
    pub qsw: BufferTransform,
    
    
    pub cpt: Vec<Ahh>,
    
    
    pub gda: bool,
    
    
    pub iw: bool,
    
    
    pub dq: String,
    
    
    pub ijv: String,
    
    
    pub hmn: bool,
    
    
    pub ogx: bool,
    
    
    pub tu: Option<u32>,
    
    
    pub uys: Option<Ahh>,
    
    
    pub tvb: Option<Ahh>,
    
    
    pub swv: Option<u32>,
    
    
    pub g: SurfaceState,
}


#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct SurfaceState {
    pub bkk: bool,
    pub szf: bool,
    pub dlg: bool,
    pub qey: bool,
    pub zsq: bool,
    pub zsr: bool,
    pub zss: bool,
    pub zsp: bool,
}

impl Surface {
    pub fn new(ad: u32) -> Self {
        Self {
            ad,
            b: 100,
            c: 100,
            z: 0,
            ac: 0,
            bi: Vec::new(),
            ltl: None,
            lto: 0,
            ltn: 0,
            qst: 0,
            qsu: 0,
            qsv: 1,
            qsw: BufferTransform::M,
            cpt: Vec::new(),
            gda: false,
            iw: true,
            dq: String::new(),
            ijv: String::new(),
            hmn: true,
            ogx: false,
            tu: None,
            uys: None,
            tvb: None,
            swv: None,
            g: SurfaceState::default(),
        }
    }
    
    
    pub fn dyl(&mut self, bi: Vec<u32>, z: u32, ac: u32) {
        self.ltl = Some(bi);
        self.lto = z;
        self.ltn = ac;
    }
    
    
    pub fn cpt(&mut self, b: i32, c: i32, z: i32, ac: i32) {
        self.cpt.push(Ahh { b, c, z, ac });
    }
    
    
    pub fn dfc(&mut self) {
        
        if let Some(bi) = self.ltl.take() {
            self.bi = bi;
            self.z = self.lto;
            self.ac = self.ltn;
        }
        
        
        self.cpt.clear();
        
        self.gda = true;
    }
    
    
    pub fn hzz(&mut self, dq: &str) {
        self.dq = String::from(dq);
    }
    
    
    pub fn zmm(&mut self, ijv: &str) {
        self.ijv = String::from(ijv);
    }
    
    
    pub fn eyk(&mut self, b: i32, c: i32) {
        self.b = b;
        self.c = c;
    }
    
    
    pub fn contains(&self, y: i32, x: i32) -> bool {
        let fwu = if self.hmn { 28 } else { 0 };
        let dn = self.b;
        let dp = self.c - fwu;
        let hy = self.b + self.z as i32;
        let jz = self.c + self.ac as i32;
        
        y >= dn && y < hy && x >= dp && x < jz
    }
    
    
    pub fn odw(&self, y: i32, x: i32) -> bool {
        if !self.hmn {
            return false;
        }
        let fwu = 28;
        y >= self.b 
            && y < self.b + self.z as i32 
            && x >= self.c - fwu 
            && x < self.c
    }
    
    
    pub fn yjx(&self) -> (i32, i32, u32, u32) {
        (self.b, self.c, self.z, self.ac)
    }
    
    
    pub fn hqr(&mut self) {
        self.ogx = true;
        self.hmn = true;
        self.g.qey = true;
    }
    
    
    pub fn zcl(&mut self, anv: u32, akr: u32) {
        self.g.bkk = true;
        self.b = 0;
        self.c = 28; 
        
    }
    
    
    pub fn zud(&mut self, bwb: i32, dur: i32) {
        self.g.bkk = false;
        self.b = bwb;
        self.c = dur;
    }
}


pub struct Bto {
    bcb: u32,
}

impl Bto {
    pub fn new() -> Self {
        Self { bcb: 1 }
    }
    
    pub fn avp(&mut self) -> Surface {
        let ad = self.bcb;
        self.bcb += 1;
        Surface::new(ad)
    }
}

impl Default for Bto {
    fn default() -> Self {
        Self::new()
    }
}
