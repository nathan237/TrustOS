






use alloc::string::String;
use alloc::vec::Vec;


#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum Reg {
    Bt = 0,   
    Oq = 1,   
    Ds = 2,   
    Bxa = 3,   
    Bbb = 4,   
    Bg = 5,   
    Mp = 6,   
    Aom = 7,   
    Aon = 8,   
    Bbd = 9,   
    Je = 10, 
    Zo = 11, 
    Afw = 12, 
    Va = 13, 
    Vb = 14, 
    Vc = 15, 
    Bav = 16, 
    Vd = 17, 
    Afx = 18, 
    Afy = 19, 
    Afz = 20, 
    Aog = 21, 
    Aoh = 22, 
    Aoi = 23, 
    Aoj = 24, 
    Aok = 25, 
    Baw = 26, 
    Bax = 27, 
    Aol = 28, 
    Bay = 29, 
    Baz = 30, 
    Bba = 31, 

    
    
    Bvo = 32,  
    Bvp = 33,     
}

impl Reg {
    pub fn ivy(a: u8) -> Self {
        match a {
            0 => Reg::Bt, 1 => Reg::Oq, 2 => Reg::Ds, 3 => Reg::Bxa,
            4 => Reg::Bbb, 5 => Reg::Bg, 6 => Reg::Mp, 7 => Reg::Aom,
            8 => Reg::Aon, 9 => Reg::Bbd, 10 => Reg::Je, 11 => Reg::Zo,
            12 => Reg::Afw, 13 => Reg::Va, 14 => Reg::Vb, 15 => Reg::Vc,
            16 => Reg::Bav, 17 => Reg::Vd, 18 => Reg::Afx, 19 => Reg::Afy,
            20 => Reg::Afz, 21 => Reg::Aog, 22 => Reg::Aoh, 23 => Reg::Aoi,
            24 => Reg::Aoj, 25 => Reg::Aok, 26 => Reg::Baw, 27 => Reg::Bax,
            28 => Reg::Aol, 29 => Reg::Bay, 30 => Reg::Baz, 31 => Reg::Bba,
            32 => Reg::Bvo, 33 => Reg::Bvp,
            _ => Reg::Bt,
        }
    }

    
    pub fn kj(&self) -> &'static str {
        match self {
            Reg::Bt => "zero", Reg::Oq => "ra", Reg::Ds => "sp", Reg::Bxa => "gp",
            Reg::Bbb => "tp", Reg::Bg => "t0", Reg::Mp => "t1", Reg::Aom => "t2",
            Reg::Aon => "fp", Reg::Bbd => "s1", Reg::Je => "a0", Reg::Zo => "a1",
            Reg::Afw => "a2", Reg::Va => "a3", Reg::Vb => "a4", Reg::Vc => "a5",
            Reg::Bav => "a6", Reg::Vd => "a7", Reg::Afx => "s2", Reg::Afy => "s3",
            Reg::Afz => "s4", Reg::Aog => "s5", Reg::Aoh => "s6", Reg::Aoi => "s7",
            Reg::Aoj => "s8", Reg::Aok => "s9", Reg::Baw => "s10", Reg::Bax => "s11",
            Reg::Aol => "t3", Reg::Bay => "t4", Reg::Baz => "t5", Reg::Bba => "t6",
            Reg::Bvo => "vflags", Reg::Bvp => "vpc",
        }
    }
}





#[derive(Debug, Clone)]
pub enum RvInst {
    
    Add { ck: Reg, cp: Reg, et: Reg },
    Sub { ck: Reg, cp: Reg, et: Reg },
    Ex { ck: Reg, cp: Reg, et: Reg },
    Fx  { ck: Reg, cp: Reg, et: Reg },
    Aga { ck: Reg, cp: Reg, et: Reg },
    Amt { ck: Reg, cp: Reg, et: Reg },  
    Amx { ck: Reg, cp: Reg, et: Reg },  
    Azc { ck: Reg, cp: Reg, et: Reg },  
    Btb { ck: Reg, cp: Reg, et: Reg },  
    Bte { ck: Reg, cp: Reg, et: Reg }, 

    
    Mul    { ck: Reg, cp: Reg, et: Reg },
    Bms   { ck: Reg, cp: Reg, et: Reg }, 
    Div    { ck: Reg, cp: Reg, et: Reg },
    Arb   { ck: Reg, cp: Reg, et: Reg },
    Rem    { ck: Reg, cp: Reg, et: Reg },
    Bqx   { ck: Reg, cp: Reg, et: Reg },

    
    Gf  { ck: Reg, cp: Reg, gf: i64 },
    Ou  { ck: Reg, cp: Reg, gf: i64 },
    Akw   { ck: Reg, cp: Reg, gf: i64 },
    Aoq  { ck: Reg, cp: Reg, gf: i64 },
    Ayv  { ck: Reg, cp: Reg, bcp: u8 },
    Aze  { ck: Reg, cp: Reg, bcp: u8 },
    Azd  { ck: Reg, cp: Reg, bcp: u8 },
    Btc  { ck: Reg, cp: Reg, gf: i64 },
    Btd { ck: Reg, cp: Reg, gf: i64 },

    
    Blq   { ck: Reg, gf: i64 },           
    Bce { ck: Reg, gf: i64 },           

    
    Bky  { ck: Reg, cp: Reg, l: i64 },  
    Ajr { ck: Reg, cp: Reg, l: i64 },  
    Bla  { ck: Reg, cp: Reg, l: i64 },  
    Ajs { ck: Reg, cp: Reg, l: i64 },  
    Blr  { ck: Reg, cp: Reg, l: i64 },  
    Aka { ck: Reg, cp: Reg, l: i64 },  
    Pt  { ck: Reg, cp: Reg, l: i64 },  
    Amf  { et: Reg, cp: Reg, l: i64 }, 
    Amo  { et: Reg, cp: Reg, l: i64 }, 
    Ang  { et: Reg, cp: Reg, l: i64 }, 
    Mi  { et: Reg, cp: Reg, l: i64 }, 

    
    Agp  { cp: Reg, et: Reg, l: i64 },  
    Ags  { cp: Reg, et: Reg, l: i64 },  
    Bcr  { cp: Reg, et: Reg, l: i64 },  
    Bcp  { cp: Reg, et: Reg, l: i64 },  
    Bcs { cp: Reg, et: Reg, l: i64 },  
    Bcq { cp: Reg, et: Reg, l: i64 },  

    
    Xh  { ck: Reg, l: i64 },              
    Xi { ck: Reg, cp: Reg, l: i64 },    

    
    Wk,                                        
    Bfr,                                       
    Bgw,                                        

    
    Bbr { ck: Reg, et: Reg, cp: Reg },   
    Bbq  { ck: Reg, et: Reg, cp: Reg },   

    
    
    Hu { ck: Reg, gf: i64 },
    
    Gl { ck: Reg, acl: Reg },
    
    Fq,
    
    Ama,
    
    En { l: i64 },

    
    
    
    Ed { cp: Reg, et: Reg },
    
    Aad { mo: FlagCond, l: i64 },
    
    Od { arch: SourceArch, ag: u64, text: String },
}


#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FlagCond {
    Eq,    
    Adl,    
    Lt,    
    Wr,    
    Auz,   
    Atb,   
    Te,    
    Jn,    
    Neg,   
    Pos,   
    Awn,   
    Awc, 
}


#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SourceArch {
    BT_,
    Fg,
    Jy,  
    Acz,
    Aod,     
}

impl SourceArch {
    pub fn j(&self) -> &'static str {
        match self {
            SourceArch::BT_ => "x86_64",
            SourceArch::Fg => "aarch64",
            SourceArch::Jy => "riscv64",
            SourceArch::Acz => "mips64",
            SourceArch::Aod => "wasm",
        }
    }
}



#[derive(Debug, Clone)]
pub struct TranslatedBlock {
    
    pub cbz: u64,
    
    pub gsy: SourceArch,
    
    pub instructions: Vec<RvInst>,
    
    pub jrg: usize,
    
    pub bil: Vec<u64>,
}

impl TranslatedBlock {
    pub fn new(cbz: u64, gsy: SourceArch) -> Self {
        Self {
            cbz,
            gsy,
            instructions: Vec::new(),
            jrg: 0,
            bil: Vec::new(),
        }
    }

    pub fn fj(&mut self, fi: RvInst) {
        self.instructions.push(fi);
    }

    pub fn yoy(&mut self, edl: &[RvInst]) {
        self.instructions.bk(edl);
    }
}


#[derive(Debug, Default, Clone)]
pub struct TranslationStats {
    pub ilv: u64,
    pub esv: u64,
    pub hyi: u64,
    pub yhb: u64,
    pub yhc: u64,
    pub ddf: u64,
}

impl TranslationStats {
    pub fn nrx(&self) -> f64 {
        if self.esv == 0 {
            return 0.0;
        }
        self.hyi as f64 / self.esv as f64
    }
}
