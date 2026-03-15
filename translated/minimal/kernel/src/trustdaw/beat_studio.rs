




















use alloc::vec::Vec;
use alloc::vec;
use alloc::format;
use alloc::string::String;
use core::sync::atomic::Ordering;

use crate::audio::synth::{Waveform, Envelope, SynthEngine, BR_, Dv};





mod colors {
    
    pub const JJ_: u32        = 0x0A0A14;
    pub const RM_: u32        = 0x0F0F1E;
    pub const HN_: u32       = 0x141428;
    pub const JK_: u32      = 0x1A1A30;
    pub const ALU_: u32    = 0x1E1E38;

    
    pub const Fj: u32         = 0x2A2A4A;
    pub const ZN_: u32  = 0x4A4A6A;
    pub const Aqo: u32        = 0x222240;

    
    pub const AIS_: u32    = 0xEEEEFF;
    pub const AC_: u32   = 0xCCCCDD;
    pub const N_: u32 = 0x8888AA;
    pub const AV_: u32       = 0x555577;
    pub const PY_: u32    = 0x66BBFF;

    
    pub const OY_: u32     = 0x44DD66;
    pub const AII_: u32      = 0x666688;
    pub const WN_: u32        = 0xFF3344;
    pub const XT_: u32   = 0x121228;

    
    pub const CUL_: u32       = 0x1A1A30;
    pub const EGW_: u32        = 0xFF6622;
    pub const EGX_: u32    = 0xFF8844;
    pub const BGJ_: u32    = 0x44FF88;
    pub const BGK_: u32  = 0x66FF66;
    pub const CUK_: u32    = 0x333355;
    pub const CUJ_: u32  = 0x444466;

    
    pub const AFJ_: u32    = 0x44CC44;
    pub const AFL_: u32   = 0xCCCC44;
    pub const AFK_: u32      = 0xCC4444;
    pub const VD_: u32       = 0x0D0D1A;
    pub const DLN_: u32     = 0xBBBBCC;
    pub const CHI_: u32    = 0xFF8800;
    pub const CTR_: u32    = 0xFFDD00;

    
    pub const ADT_: u32      = 0xDDDDEE;
    pub const ADS_: u32      = 0x222233;
    pub const AYB_: u32    = 0xFF6622;
    pub const CDB_: u32      = 0x444455;

    
    pub const CQO_: u32     = 0x44DDFF;
    pub const CQN_: u32       = 0x0A0A18;
    pub const CTS_: u32     = 0x22CC66;
    pub const CTT_: u32     = 0x66DD44;
    pub const CTU_: u32     = 0xCCCC22;
    pub const CTV_: u32     = 0xDD6622;
    pub const CTW_: u32     = 0xDD2222;

    
    pub const S_: [u32; 8] = [
        0xFF4444, 
        0xFFAA22, 
        0xFFDD44, 
        0x44DD66, 
        0x44AAFF, 
        0x8844FF, 
        0xFF44AA, 
        0x44DDDD, 
    ];
}






const GP_: usize = 8;

const FL_: usize = 32;

const BRH_: usize = 16;


#[derive(Clone, Copy)]
pub struct BeatStep {
    
    pub gh: bool,
    
    pub qm: u8,
    
    pub gnz: i8,
}

impl BeatStep {
    pub fn dz() -> Self {
        Self { gh: false, qm: 100, gnz: 0 }
    }

    pub fn ea(qm: u8) -> Self {
        Self { gh: true, qm, gnz: 0 }
    }

    pub fn bz(qm: u8, l: i8) -> Self {
        Self { gh: true, qm, gnz: l }
    }
}


pub struct BeatTrack {
    
    pub j: [u8; 16],
    pub baf: usize,
    
    pub au: [BeatStep; FL_],
    
    pub aml: usize,
    
    pub fdc: u8,
    
    pub ve: Waveform,
    
    pub qr: Envelope,
    
    pub hq: u8,
    
    pub arp: i8,
    
    pub so: bool,
    
    pub cic: bool,
    
    pub s: u32,
    
    pub jbg: bool,
}

impl BeatTrack {
    pub fn new(j: &str, fdc: u8, ve: Waveform, s: u32, jbg: bool) -> Self {
        let mut djr = [0u8; 16];
        let bf = j.as_bytes();
        let len = bf.len().v(16);
        djr[..len].dg(&bf[..len]);

        Self {
            j: djr,
            baf: len,
            au: [BeatStep::dz(); FL_],
            aml: BRH_,
            fdc,
            ve,
            qr: Envelope::hvi(),
            hq: 200,
            arp: 0,
            so: false,
            cic: false,
            s,
            jbg,
        }
    }

    pub fn amj(&self) -> &str {
        core::str::jg(&self.j[..self.baf]).unwrap_or("???")
    }

    
    pub fn xiy(&mut self, gu: usize) {
        if gu < self.aml {
            self.au[gu].gh = !self.au[gu].gh;
        }
    }

    
    pub fn lov(&self, gu: usize) -> u8 {
        if gu < self.aml && self.au[gu].gh {
            let ar = self.fdc as i16;
            let l = self.au[gu].gnz as i16;
            (ar + l).qp(0, 127) as u8
        } else {
            0
        }
    }

    
    pub fn gxu(&self) -> usize {
        self.au[..self.aml].iter().hi(|e| e.gh).az()
    }
}






pub struct BeatStudio {
    
    pub af: [BeatTrack; GP_],
    
    pub kz: u16,
    
    pub ezi: u8,
    
    pub uij: u8,

    
    pub uu: bool,
    pub ehe: bool,
    pub aop: usize,

    
    pub bdw: usize,
    pub bzc: usize,

    
    pub hyy: [i16; 256],
    pub jnu: usize,
    pub mgq: [u8; 16],

    
    pub dsl: [bool; 128],
    pub cgg: i8,
    pub qm: u8,

    
    gz: u32,
    kc: u32,
}

impl BeatStudio {
    
    pub fn new() -> Self {
        let gz = crate::framebuffer::AB_.load(Ordering::Relaxed) as u32;
        let kc = crate::framebuffer::Z_.load(Ordering::Relaxed) as u32;

        let af = [
            BeatTrack::new("Kick",  36, Waveform::Dg,     colors::S_[0], true),
            BeatTrack::new("Snare", 38, Waveform::Cr,    colors::S_[1], true),
            BeatTrack::new("HiHat", 42, Waveform::Cr,    colors::S_[2], true),
            BeatTrack::new("Bass",  36, Waveform::Gb,   colors::S_[3], false),
            BeatTrack::new("Lead",  60, Waveform::Ft, colors::S_[4], false),
            BeatTrack::new("Pad",   60, Waveform::Triangle, colors::S_[5], false),
            BeatTrack::new("FX",    48, Waveform::Ft, colors::S_[6], false),
            BeatTrack::new("Perc",  56, Waveform::Cr,    colors::S_[7], true),
        ];

        let mut er = Self {
            af,
            kz: 128,
            ezi: 50,
            uij: 1,
            uu: false,
            ehe: false,
            aop: 0,
            bdw: 0,
            bzc: 0,
            hyy: [0i16; 256],
            jnu: 0,
            mgq: [0u8; 16],
            dsl: [false; 128],
            cgg: 0,
            qm: 100,
            gz,
            kc,
        };

        
        er.ugq();

        
        er.af[0].qr = Envelope::new(2, 80, 0, 50);    
        er.af[1].qr = Envelope::new(1, 40, 0, 30);    
        er.af[2].qr = Envelope::new(1, 20, 0, 15);    
        er.af[3].qr = Envelope::hvi();                
        er.af[4].qr = Envelope::new(5, 100, 70, 80);  
        er.af[5].qr = Envelope::ov();                  
        er.af[6].qr = Envelope::new(1, 200, 0, 100);  
        er.af[7].qr = Envelope::new(1, 30, 0, 20);    

        er
    }

    
    fn ugq(&mut self) {
        
        for a in [0, 4, 8, 12] {
            self.af[0].au[a] = BeatStep::ea(127);
        }

        
        for a in [4, 12] {
            self.af[1].au[a] = BeatStep::ea(110);
        }

        
        for a in [0, 2, 4, 6, 8, 10, 12, 14] {
            self.af[2].au[a] = BeatStep::ea(80);
        }
        
        for a in [1, 3, 5, 7, 9, 11, 13, 15] {
            self.af[2].au[a] = BeatStep::ea(40);
        }

        
        self.af[3].au[0] = BeatStep::bz(120, 0);   
        self.af[3].au[3] = BeatStep::bz(100, 0);   
        self.af[3].au[6] = BeatStep::bz(110, 3);   
        self.af[3].au[10] = BeatStep::bz(100, 5);  
        self.af[3].au[13] = BeatStep::bz(90, 3);   

        
        self.af[4].au[0] = BeatStep::bz(100, 0);   
        self.af[4].au[2] = BeatStep::bz(90, 3);    
        self.af[4].au[4] = BeatStep::bz(100, 7);   
        self.af[4].au[8] = BeatStep::bz(110, 5);   
        self.af[4].au[11] = BeatStep::bz(90, 3);   

        
        self.af[5].au[0] = BeatStep::bz(70, 0);    

        
        self.af[6].au[6] = BeatStep::ea(60);
        self.af[6].au[14] = BeatStep::ea(50);

        
        self.af[7].au[1] = BeatStep::ea(80);
        self.af[7].au[5] = BeatStep::ea(90);
        self.af[7].au[9] = BeatStep::ea(80);
        self.af[7].au[13] = BeatStep::ea(70);
    }

    
    
    pub fn ljh(&mut self) {
        
        self.kz = 100;
        self.ezi = 56; 

        
        for ab in self.af.el() {
            ab.aml = 32;
            for e in 0..FL_ {
                ab.au[e] = BeatStep::dz();
            }
        }

        
        self.af[0] = BeatTrack::new("Kick",    36, Waveform::Dg,     colors::S_[0], true);
        self.af[1] = BeatTrack::new("Clap",    39, Waveform::Cr,    colors::S_[1], true);
        self.af[2] = BeatTrack::new("HiHat",   42, Waveform::Cr,    colors::S_[2], true);
        self.af[3] = BeatTrack::new("Sub Bass", 24, Waveform::Dg,    colors::S_[3], false);
        self.af[4] = BeatTrack::new("Mid Bass", 36, Waveform::Gb,  colors::S_[4], false);
        self.af[5] = BeatTrack::new("Chords",  60, Waveform::Triangle, colors::S_[5], false);
        self.af[6] = BeatTrack::new("Lead",    72, Waveform::Ft, colors::S_[6], false);
        self.af[7] = BeatTrack::new("Perc",    56, Waveform::Cr,    colors::S_[7], true);

        for ab in self.af.el() {
            ab.aml = 32;
        }

        
        self.af[0].qr = Envelope::new(2, 200, 0, 80);    
        self.af[1].qr = Envelope::new(1, 70, 0, 45);     
        self.af[2].qr = Envelope::new(1, 22, 0, 12);     
        self.af[3].qr = Envelope::new(8, 400, 85, 250);  
        self.af[4].qr = Envelope::new(5, 150, 50, 100);  
        self.af[5].qr = Envelope::ov();                   
        self.af[6].qr = Envelope::new(10, 200, 70, 180); 
        self.af[7].qr = Envelope::new(1, 30, 0, 18);     

        
        
        
        for a in (0..32).akt(4) {
            self.af[0].au[a] = BeatStep::ea(127);
        }
        self.af[0].au[3]  = BeatStep::ea(45);  
        self.af[0].au[15] = BeatStep::ea(40);  
        self.af[0].au[27] = BeatStep::ea(45);  

        
        
        
        self.af[1].au[4]  = BeatStep::ea(120);
        self.af[1].au[12] = BeatStep::ea(120);
        self.af[1].au[20] = BeatStep::ea(120);
        self.af[1].au[28] = BeatStep::ea(120);
        self.af[1].au[11] = BeatStep::ea(50);  
        self.af[1].au[27] = BeatStep::ea(55);  

        
        
        
        for a in 0..16 {
            let bxr = match a % 4 {
                0 => 85,
                2 => 105,  
                1 => 35,
                3 => 50,
                _ => 45,
            };
            self.af[2].au[a] = BeatStep::ea(bxr);
        }
        for a in 16..32 {
            let bxr = match a % 4 {
                0 => 80,
                2 => 110,  
                1 => 30,
                3 => 45,
                _ => 40,
            };
            self.af[2].au[a] = BeatStep::ea(bxr);
        }
        self.af[2].au[23] = BeatStep::dz(); 
        self.af[2].au[31] = BeatStep::dz();

        
        
        
        
        
        
        self.af[3].au[0]  = BeatStep::bz(127, 0);   
        self.af[3].au[6]  = BeatStep::bz(100, 0);   
        self.af[3].au[8]  = BeatStep::bz(120, 0);   
        self.af[3].au[14] = BeatStep::bz(100, 0);   
        
        self.af[3].au[16] = BeatStep::bz(127, 8);   
        self.af[3].au[20] = BeatStep::bz(110, 8);   
        self.af[3].au[24] = BeatStep::bz(127, 10);  
        self.af[3].au[28] = BeatStep::bz(110, 10);  

        
        
        
        
        
        self.af[4].au[0]  = BeatStep::bz(120, 0);   
        self.af[4].au[3]  = BeatStep::bz(110, 0);   
        self.af[4].au[5]  = BeatStep::bz(100, 3);   
        self.af[4].au[7]  = BeatStep::bz(115, 7);   
        self.af[4].au[10] = BeatStep::bz(105, 5);   
        self.af[4].au[13] = BeatStep::bz(95, 3);    
        
        self.af[4].au[16] = BeatStep::bz(120, 8);   
        self.af[4].au[19] = BeatStep::bz(110, 7);   
        self.af[4].au[21] = BeatStep::bz(100, 5);   
        self.af[4].au[24] = BeatStep::bz(120, 10);  
        self.af[4].au[26] = BeatStep::bz(105, 7);   
        self.af[4].au[29] = BeatStep::bz(100, 5);   
        self.af[4].au[31] = BeatStep::bz(90, 3);    

        
        
        
        
        self.af[5].au[0]  = BeatStep::bz(80, 0);    
        self.af[5].au[4]  = BeatStep::bz(70, 3);    
        self.af[5].au[8]  = BeatStep::bz(75, 7);    
        self.af[5].au[12] = BeatStep::bz(70, 3);    
        
        self.af[5].au[16] = BeatStep::bz(80, 8);    
        self.af[5].au[20] = BeatStep::bz(75, 7);    
        self.af[5].au[24] = BeatStep::bz(80, 10);   
        self.af[5].au[28] = BeatStep::bz(75, 7);    

        
        
        
        
        
        self.af[6].au[0]  = BeatStep::bz(100, 7);   
        self.af[6].au[2]  = BeatStep::bz(105, 10);  
        self.af[6].au[3]  = BeatStep::bz(115, 12);  
        self.af[6].au[5]  = BeatStep::bz(100, 10);  
        self.af[6].au[7]  = BeatStep::bz(90, 7);    
        self.af[6].au[8]  = BeatStep::bz(105, 5);   
        self.af[6].au[10] = BeatStep::bz(100, 3);   
        self.af[6].au[12] = BeatStep::bz(110, 5);   
        self.af[6].au[14] = BeatStep::bz(105, 7);   
        
        self.af[6].au[16] = BeatStep::bz(115, 12);  
        self.af[6].au[17] = BeatStep::bz(100, 10);  
        self.af[6].au[19] = BeatStep::bz(110, 12);  
        self.af[6].au[20] = BeatStep::bz(120, 15);  
        self.af[6].au[22] = BeatStep::bz(105, 12);  
        self.af[6].au[24] = BeatStep::bz(100, 10);  
        self.af[6].au[25] = BeatStep::bz(95, 7);    
        self.af[6].au[27] = BeatStep::bz(90, 5);    
        self.af[6].au[29] = BeatStep::bz(85, 3);    
        self.af[6].au[31] = BeatStep::bz(80, 0);    

        
        
        
        for a in (1..32).akt(2) {
            self.af[7].au[a] = BeatStep::ea(55);
        }
        for a in (2..32).akt(4) {
            self.af[7].au[a] = BeatStep::ea(90);
        }
        
        self.af[7].au[26] = BeatStep::ea(90);
        self.af[7].au[27] = BeatStep::ea(100);
        self.af[7].au[28] = BeatStep::ea(105);
        self.af[7].au[29] = BeatStep::ea(110);
        self.af[7].au[30] = BeatStep::ea(120);
        self.af[7].au[31] = BeatStep::ea(127); 

        
        self.af[0].hq = 230; 
        self.af[1].hq = 185; 
        self.af[2].hq = 140; 
        self.af[3].hq = 255; 
        self.af[4].hq = 200; 
        self.af[5].hq = 110; 
        self.af[6].hq = 175; 
        self.af[7].hq = 120; 
    }

    
    
    
    
    

    
    fn gyu(&mut self) {
        self.kz = 106;
        self.ezi = 50; 

        self.af[0] = BeatTrack::new("Kick",  36, Waveform::Dg,     colors::S_[0], true);
        self.af[1] = BeatTrack::new("Snare", 38, Waveform::Cr,    colors::S_[1], true);
        self.af[2] = BeatTrack::new("HiHat", 42, Waveform::Cr,    colors::S_[2], true);
        self.af[3] = BeatTrack::new("Sub",   24, Waveform::Dg,     colors::S_[3], false);
        self.af[4] = BeatTrack::new("Bass",  36, Waveform::Gb,   colors::S_[4], false);
        self.af[5] = BeatTrack::new("Pad",   60, Waveform::Triangle, colors::S_[5], false);
        self.af[6] = BeatTrack::new("Lead",  72, Waveform::Ft, colors::S_[6], false);
        self.af[7] = BeatTrack::new("Arp",   72, Waveform::Triangle, colors::S_[7], false);

        for ab in self.af.el() {
            ab.aml = 32;
            for e in 0..FL_ { ab.au[e] = BeatStep::dz(); }
        }

        
        self.af[0].qr = Envelope::new(2, 200, 0, 80);
        self.af[1].qr = Envelope::new(1, 65, 0, 40);
        self.af[2].qr = Envelope::new(1, 18, 0, 8);
        self.af[3].qr = Envelope::new(15, 600, 90, 350);
        self.af[4].qr = Envelope::new(5, 200, 45, 130);
        self.af[5].qr = Envelope::ov();
        self.af[6].qr = Envelope::new(10, 280, 75, 220);
        self.af[7].qr = Envelope::new(3, 100, 25, 70);

        self.af[0].hq = 200;
        self.af[1].hq = 175;
        self.af[2].hq = 130;
        self.af[3].hq = 240;
        self.af[4].hq = 190;
        self.af[5].hq = 100;
        self.af[6].hq = 180;
        self.af[7].hq = 140;
    }

    
    fn qiz(&mut self) {
        self.gyu();
        
        self.af[0].so = true;  
        self.af[1].so = true;  
        self.af[2].so = true;  
        self.af[4].so = true;  
        self.af[6].so = true;  

        
        self.af[3].hq = 180;
        self.af[3].au[0]  = BeatStep::bz(70, 0);   
        self.af[3].au[16] = BeatStep::bz(50, 0);   

        
        self.af[5].hq = 75;
        self.af[5].au[0]  = BeatStep::bz(45, 0);   
        self.af[5].au[8]  = BeatStep::bz(40, 3);   
        self.af[5].au[16] = BeatStep::bz(45, 7);   
        self.af[5].au[24] = BeatStep::bz(40, 3);   

        
        self.af[7] = BeatTrack::new("Texture", 72, Waveform::Cr, colors::S_[7], true);
        self.af[7].aml = 32;
        self.af[7].qr = Envelope::new(1, 15, 0, 5);
        self.af[7].hq = 50;
        self.af[7].au[4]  = BeatStep::ea(25);
        self.af[7].au[5]  = BeatStep::ea(20);  
        self.af[7].au[11] = BeatStep::ea(30);
        self.af[7].au[18] = BeatStep::ea(18);
        self.af[7].au[19] = BeatStep::ea(25);
        self.af[7].au[26] = BeatStep::ea(28);
    }

    
    fn qix(&mut self) {
        self.gyu();
        self.af[1].so = true;  
        self.af[6].so = true;  

        
        self.af[0].hq = 160;
        for a in (0..32).akt(8) {
            self.af[0].au[a] = BeatStep::ea(80);
        }

        
        self.af[2].hq = 90;
        for a in (0..32).akt(4) {
            self.af[2].au[a] = BeatStep::ea(30);
        }
        for a in (2..32).akt(4) {
            self.af[2].au[a] = BeatStep::ea(55);
        }

        
        self.af[3].hq = 220;
        self.af[3].au[0]  = BeatStep::bz(100, 0);  
        self.af[3].au[8]  = BeatStep::bz(90, 0);   
        self.af[3].au[16] = BeatStep::bz(100, 8);  
        self.af[3].au[24] = BeatStep::bz(95, 10);  

        
        self.af[4].hq = 150;
        self.af[4].au[0]  = BeatStep::bz(90, 0);   
        self.af[4].au[4]  = BeatStep::bz(75, 0);   
        self.af[4].au[8]  = BeatStep::bz(85, 3);   
        self.af[4].au[12] = BeatStep::bz(80, 7);   
        self.af[4].au[16] = BeatStep::bz(90, 8);   
        self.af[4].au[20] = BeatStep::bz(80, 7);   
        self.af[4].au[24] = BeatStep::bz(95, 10);  
        self.af[4].au[28] = BeatStep::bz(85, 7);   

        
        self.af[5].hq = 90;
        self.af[5].au[0]  = BeatStep::bz(55, 0);   
        self.af[5].au[4]  = BeatStep::bz(50, 3);   
        self.af[5].au[8]  = BeatStep::bz(55, 7);   
        self.af[5].au[12] = BeatStep::bz(50, 3);   
        self.af[5].au[16] = BeatStep::bz(55, 8);   
        self.af[5].au[20] = BeatStep::bz(50, 7);   
        self.af[5].au[24] = BeatStep::bz(55, 10);  
        self.af[5].au[28] = BeatStep::bz(50, 7);   

        
        self.af[7].hq = 120;
        
        self.af[7].au[0]  = BeatStep::bz(80, 0);   
        self.af[7].au[2]  = BeatStep::bz(85, 3);   
        self.af[7].au[4]  = BeatStep::bz(90, 7);   
        self.af[7].au[6]  = BeatStep::bz(95, 12);  
        self.af[7].au[8]  = BeatStep::bz(80, 0);   
        self.af[7].au[10] = BeatStep::bz(85, 3);   
        self.af[7].au[12] = BeatStep::bz(90, 7);   
        self.af[7].au[14] = BeatStep::bz(100, 12); 
        
        self.af[7].au[16] = BeatStep::bz(80, 8);   
        self.af[7].au[18] = BeatStep::bz(85, 0);   
        self.af[7].au[20] = BeatStep::bz(90, 3);   
        self.af[7].au[22] = BeatStep::bz(85, 8);   
        self.af[7].au[24] = BeatStep::bz(85, 10);  
        self.af[7].au[26] = BeatStep::bz(90, 2);   
        self.af[7].au[28] = BeatStep::bz(95, 5);   
        self.af[7].au[30] = BeatStep::bz(100, 10); 
    }

    
    fn qiy(&mut self) {
        self.gyu();
        

        
        self.af[0].hq = 225;
        for a in (0..32).akt(4) {
            self.af[0].au[a] = BeatStep::ea(120);
        }
        self.af[0].au[3]  = BeatStep::ea(40);
        self.af[0].au[15] = BeatStep::ea(35);
        self.af[0].au[19] = BeatStep::ea(40);
        self.af[0].au[27] = BeatStep::ea(35);

        
        self.af[1].hq = 180;
        self.af[1].au[4]  = BeatStep::ea(115);
        self.af[1].au[12] = BeatStep::ea(115);
        self.af[1].au[20] = BeatStep::ea(115);
        self.af[1].au[28] = BeatStep::ea(115);
        self.af[1].au[11] = BeatStep::ea(45); 

        
        self.af[2].hq = 140;
        for a in 0..32 {
            let bxr = match a % 4 { 0 => 80, 2 => 100, 1 => 35, _ => 50 };
            self.af[2].au[a] = BeatStep::ea(bxr);
        }
        self.af[2].au[15] = BeatStep::dz();
        self.af[2].au[31] = BeatStep::dz();

        
        self.af[3].hq = 250;
        self.af[3].au[0]  = BeatStep::bz(127, 0);
        self.af[3].au[6]  = BeatStep::bz(100, 0);
        self.af[3].au[8]  = BeatStep::bz(120, 0);
        self.af[3].au[14] = BeatStep::bz(95, 0);
        self.af[3].au[16] = BeatStep::bz(127, 8);  
        self.af[3].au[20] = BeatStep::bz(105, 8);
        self.af[3].au[24] = BeatStep::bz(127, 10); 
        self.af[3].au[28] = BeatStep::bz(105, 10);

        
        self.af[4].hq = 200;
        self.af[4].au[0]  = BeatStep::bz(115, 0);  
        self.af[4].au[3]  = BeatStep::bz(105, 0);  
        self.af[4].au[5]  = BeatStep::bz(95, 3);   
        self.af[4].au[7]  = BeatStep::bz(110, 7);  
        self.af[4].au[10] = BeatStep::bz(100, 5);  
        self.af[4].au[13] = BeatStep::bz(90, 3);   
        self.af[4].au[16] = BeatStep::bz(115, 8);  
        self.af[4].au[19] = BeatStep::bz(105, 7);  
        self.af[4].au[21] = BeatStep::bz(95, 5);   
        self.af[4].au[24] = BeatStep::bz(115, 10); 
        self.af[4].au[26] = BeatStep::bz(100, 7);  
        self.af[4].au[29] = BeatStep::bz(95, 5);   
        self.af[4].au[31] = BeatStep::bz(85, 3);   

        
        self.af[5].hq = 110;
        self.af[5].au[0]  = BeatStep::bz(70, 0);   
        self.af[5].au[4]  = BeatStep::bz(65, 3);   
        self.af[5].au[8]  = BeatStep::bz(70, 7);   
        self.af[5].au[12] = BeatStep::bz(65, 3);   
        self.af[5].au[16] = BeatStep::bz(70, 8);   
        self.af[5].au[20] = BeatStep::bz(65, 7);   
        self.af[5].au[24] = BeatStep::bz(70, 10);  
        self.af[5].au[28] = BeatStep::bz(65, 7);   

        
        self.af[6].hq = 190;
        
        self.af[6].au[0]  = BeatStep::bz(100, 7);  
        self.af[6].au[3]  = BeatStep::bz(110, 10); 
        self.af[6].au[6]  = BeatStep::bz(120, 12); 
        self.af[6].au[10] = BeatStep::bz(105, 10); 
        self.af[6].au[12] = BeatStep::bz(100, 8);  
        self.af[6].au[14] = BeatStep::bz(95, 7);   
        
        self.af[6].au[16] = BeatStep::bz(105, 3);  
        self.af[6].au[18] = BeatStep::bz(110, 7);  
        self.af[6].au[20] = BeatStep::bz(120, 12); 
        self.af[6].au[22] = BeatStep::bz(127, 15); 
        self.af[6].au[24] = BeatStep::bz(110, 12); 
        self.af[6].au[26] = BeatStep::bz(100, 10); 
        self.af[6].au[28] = BeatStep::bz(90, 7);   
        self.af[6].au[30] = BeatStep::bz(85, 0);   

        
        self.af[7].hq = 130;
        self.af[7].au[1]  = BeatStep::bz(70, 12);  
        self.af[7].au[5]  = BeatStep::bz(65, 7);   
        self.af[7].au[9]  = BeatStep::bz(70, 12);  
        self.af[7].au[13] = BeatStep::bz(65, 3);   
        self.af[7].au[17] = BeatStep::bz(70, 8);   
        self.af[7].au[21] = BeatStep::bz(75, 12);  
        self.af[7].au[25] = BeatStep::bz(70, 10);  
        self.af[7].au[29] = BeatStep::bz(65, 7);   
    }

    
    fn qjb(&mut self) {
        self.gyu();

        
        self.af[0].hq = 210;
        for a in (0..32).akt(8) {
            self.af[0].au[a] = BeatStep::ea(110);
        }

        
        self.af[1].hq = 165;
        self.af[1].au[4]  = BeatStep::ea(105);
        self.af[1].au[12] = BeatStep::ea(105);
        self.af[1].au[20] = BeatStep::ea(105);
        self.af[1].au[28] = BeatStep::ea(105);

        
        self.af[2].hq = 100;
        for a in (2..32).akt(4) {
            self.af[2].au[a] = BeatStep::ea(60);
        }

        
        self.af[3].hq = 230;
        self.af[3].au[0]  = BeatStep::bz(110, 0);  
        self.af[3].au[8]  = BeatStep::bz(100, 0);
        self.af[3].au[16] = BeatStep::bz(110, 8);  
        self.af[3].au[24] = BeatStep::bz(105, 10); 

        
        self.af[4].hq = 180;
        self.af[4].au[0]  = BeatStep::bz(100, 0);  
        self.af[4].au[4]  = BeatStep::bz(85, 7);   
        self.af[4].au[8]  = BeatStep::bz(95, 4);   
        self.af[4].au[12] = BeatStep::bz(85, 0);   
        self.af[4].au[16] = BeatStep::bz(100, 8);  
        self.af[4].au[20] = BeatStep::bz(85, 7);   
        self.af[4].au[24] = BeatStep::bz(100, 10); 
        self.af[4].au[28] = BeatStep::bz(90, 5);   

        
        self.af[5].hq = 105;
        self.af[5].au[0]  = BeatStep::bz(60, 0);   
        self.af[5].au[8]  = BeatStep::bz(55, 4);   
        self.af[5].au[16] = BeatStep::bz(60, 7);   
        self.af[5].au[24] = BeatStep::bz(55, 12);  

        
        
        self.af[6].hq = 200;
        
        self.af[6].au[0]  = BeatStep::bz(110, 0);  
        self.af[6].au[4]  = BeatStep::bz(115, 4);  
        self.af[6].au[8]  = BeatStep::bz(120, 7);  
        self.af[6].au[12] = BeatStep::bz(127, 12); 
        
        self.af[6].au[16] = BeatStep::bz(120, 12); 
        self.af[6].au[20] = BeatStep::bz(115, 7);  
        self.af[6].au[24] = BeatStep::bz(110, 4);  
        self.af[6].au[28] = BeatStep::bz(105, 0);  

        
        self.af[7].hq = 115;
        self.af[7].au[2]  = BeatStep::bz(75, 12);  
        self.af[7].au[6]  = BeatStep::bz(70, 16);  
        self.af[7].au[10] = BeatStep::bz(75, 19);  
        self.af[7].au[14] = BeatStep::bz(80, 12);  
        self.af[7].au[18] = BeatStep::bz(70, 19);  
        self.af[7].au[22] = BeatStep::bz(75, 16);  
        self.af[7].au[26] = BeatStep::bz(70, 12);  
        self.af[7].au[30] = BeatStep::bz(65, 7);   
    }

    
    fn qja(&mut self) {
        self.gyu();
        
        self.af[0].so = true;  
        self.af[1].so = true;  
        self.af[2].so = true;  
        self.af[4].so = true;  
        self.af[7].so = true;  

        
        self.af[3].hq = 150;
        self.af[3].au[0]  = BeatStep::bz(60, 0);   
        self.af[3].au[16] = BeatStep::bz(40, 0);   

        
        self.af[5].hq = 80;
        self.af[5].au[0]  = BeatStep::bz(40, 0);   
        self.af[5].au[8]  = BeatStep::bz(35, 7);   
        self.af[5].au[16] = BeatStep::bz(40, 12);  
        self.af[5].au[24] = BeatStep::bz(35, 7);   

        
        self.af[6].hq = 140;
        self.af[6].au[0]  = BeatStep::bz(80, 0);   
        self.af[6].au[4]  = BeatStep::bz(75, 4);   
        self.af[6].au[8]  = BeatStep::bz(80, 7);   
        self.af[6].au[12] = BeatStep::bz(85, 12);  
        
    }

    
    
    
    
    
    
    
    
    
    

    
    
    fn fxi(&mut self) {
        self.kz = 100;   
        self.ezi = 50;

        
        
        self.af[0] = BeatTrack::new("Sub",     27, Waveform::Dg,     colors::S_[0], false);  
        self.af[1] = BeatTrack::new("Snare",   38, Waveform::Cr,    colors::S_[1], true);
        self.af[2] = BeatTrack::new("HiHat",   56, Waveform::Cr,    colors::S_[2], true);
        self.af[3] = BeatTrack::new("OpenHat", 53, Waveform::Cr,    colors::S_[3], true);
        self.af[4] = BeatTrack::new("Synth",   63, Waveform::Gb,   colors::S_[4], false);  
        self.af[5] = BeatTrack::new("Pad",     51, Waveform::Ft, colors::S_[5], false);  
        self.af[6] = BeatTrack::new("Lead",    75, Waveform::Ft, colors::S_[6], false);  
        self.af[7] = BeatTrack::new("Perc",    63, Waveform::Cr,    colors::S_[7], true);

        for ab in self.af.el() {
            ab.aml = 32;
            for e in 0..FL_ { ab.au[e] = BeatStep::dz(); }
            ab.so = false;
        }

        
        self.af[0].qr = Envelope::new(1, 1800, 80, 600);  
        self.af[1].qr = Envelope::new(1, 80, 0, 35);      
        self.af[2].qr = Envelope::new(1, 16, 0, 6);       
        self.af[3].qr = Envelope::new(1, 140, 0, 80);     
        self.af[4].qr = Envelope::new(3, 380, 25, 260);   
        self.af[5].qr = Envelope::ov();                   
        self.af[6].qr = Envelope::new(4, 300, 50, 220);   
        self.af[7].qr = Envelope::new(1, 25, 0, 8);       
    }

    
    
    
    
    
    pub fn uhn(&mut self) {
        self.fxi();
        self.af[1].so = true;   
        self.af[2].so = true;   
        self.af[3].so = true;   
        self.af[7].so = true;   

        
        self.af[0].hq = 120;
        self.af[0].au[0]  = BeatStep::bz(60, 0);     

        
        self.af[5].hq = 50;
        self.af[5].au[0]  = BeatStep::bz(35, 0);     
        self.af[5].au[8]  = BeatStep::bz(30, 5);     
        self.af[5].au[16] = BeatStep::bz(35, 3);     
        self.af[5].au[24] = BeatStep::bz(30, -2);    

        
        self.af[4].hq = 55;
        self.af[4].au[4]  = BeatStep::bz(40, 0);     
        self.af[4].au[12] = BeatStep::bz(35, -3);    
        self.af[4].au[20] = BeatStep::bz(38, 5);     

        
        self.af[6].hq = 35;
        self.af[6].au[16] = BeatStep::bz(30, 7);     
        self.af[6].au[24] = BeatStep::bz(25, 0);     
    }

    
    
    
    
    
    
    pub fn jdv(&mut self) {
        self.fxi();

        
        self.af[0].hq = 255;
        self.af[0].au[0]  = BeatStep::bz(127, 0);    
        self.af[0].au[6]  = BeatStep::bz(90, 0);     
        self.af[0].au[8]  = BeatStep::bz(118, 0);    
        self.af[0].au[16] = BeatStep::bz(125, 5);    
        self.af[0].au[20] = BeatStep::bz(105, 3);    
        self.af[0].au[24] = BeatStep::bz(120, -2);   
        self.af[0].au[28] = BeatStep::bz(100, 0);    

        
        self.af[1].hq = 185;
        self.af[1].au[8]  = BeatStep::ea(120);
        self.af[1].au[24] = BeatStep::ea(118);
        self.af[1].au[6]  = BeatStep::ea(32);             
        self.af[1].au[22] = BeatStep::ea(30);             
        self.af[1].au[15] = BeatStep::ea(40);             

        
        
        self.af[2].hq = 100;
        
        self.af[2].au[0]  = BeatStep::ea(90);
        self.af[2].au[1]  = BeatStep::ea(38);
        self.af[2].au[2]  = BeatStep::ea(72);
        self.af[2].au[3]  = BeatStep::ea(35);
        self.af[2].au[4]  = BeatStep::ea(85);
        self.af[2].au[5]  = BeatStep::ea(32);
        self.af[2].au[6]  = BeatStep::ea(68);
        self.af[2].au[7]  = BeatStep::ea(30);
        self.af[2].au[8]  = BeatStep::ea(88);
        self.af[2].au[9]  = BeatStep::ea(36);
        self.af[2].au[10] = BeatStep::ea(70);
        self.af[2].au[11] = BeatStep::ea(33);
        self.af[2].au[12] = BeatStep::ea(82);
        self.af[2].au[13] = BeatStep::ea(35);
        self.af[2].au[14] = BeatStep::ea(75);
        self.af[2].au[15] = BeatStep::ea(40);
        
        self.af[2].au[16] = BeatStep::ea(90);
        self.af[2].au[17] = BeatStep::ea(38);
        self.af[2].au[18] = BeatStep::ea(72);
        self.af[2].au[19] = BeatStep::ea(35);
        self.af[2].au[20] = BeatStep::ea(85);
        self.af[2].au[21] = BeatStep::ea(32);
        self.af[2].au[22] = BeatStep::ea(68);
        self.af[2].au[23] = BeatStep::ea(30);
        
        self.af[2].au[24] = BeatStep::ea(42);
        self.af[2].au[25] = BeatStep::ea(52);
        self.af[2].au[26] = BeatStep::ea(62);
        self.af[2].au[27] = BeatStep::ea(72);
        self.af[2].au[28] = BeatStep::ea(82);
        self.af[2].au[29] = BeatStep::ea(92);
        self.af[2].au[30] = BeatStep::ea(102);
        self.af[2].au[31] = BeatStep::ea(115);

        
        self.af[3].hq = 70;
        self.af[3].au[4]  = BeatStep::ea(65);
        self.af[3].au[12] = BeatStep::ea(60);
        self.af[3].au[20] = BeatStep::ea(58);

        
        self.af[4].hq = 95;
        
        self.af[4].au[0]  = BeatStep::bz(82, 0);     
        self.af[4].au[2]  = BeatStep::bz(60, 3);     
        self.af[4].au[4]  = BeatStep::bz(70, 7);     
        self.af[4].au[6]  = BeatStep::bz(55, 12);    
        self.af[4].au[8]  = BeatStep::bz(78, -4);    
        self.af[4].au[10] = BeatStep::bz(58, 0);     
        self.af[4].au[12] = BeatStep::bz(68, 3);     
        
        self.af[4].au[16] = BeatStep::bz(80, 5);     
        self.af[4].au[18] = BeatStep::bz(58, 9);     
        self.af[4].au[20] = BeatStep::bz(72, 12);    
        self.af[4].au[24] = BeatStep::bz(75, 3);     
        self.af[4].au[26] = BeatStep::bz(55, 7);     
        self.af[4].au[28] = BeatStep::bz(65, -2);    

        
        self.af[5].hq = 42;
        self.af[5].au[0]  = BeatStep::bz(32, 0);     
        self.af[5].au[8]  = BeatStep::bz(28, 5);     
        self.af[5].au[16] = BeatStep::bz(32, 3);     
        self.af[5].au[24] = BeatStep::bz(28, -2);    

        
        
        self.af[6].hq = 125;
        self.af[6].au[0]  = BeatStep::bz(105, 7);    
        self.af[6].au[2]  = BeatStep::bz(88, 3);     
        self.af[6].au[4]  = BeatStep::bz(95, 7);     
        self.af[6].au[8]  = BeatStep::bz(118, 12);   
        
        self.af[6].au[14] = BeatStep::bz(80, 5);     
        
        self.af[6].au[16] = BeatStep::bz(95, 5);     
        self.af[6].au[19] = BeatStep::bz(85, 3);     
        self.af[6].au[22] = BeatStep::bz(78, -2);    
        self.af[6].au[26] = BeatStep::bz(72, 0);     

        
        self.af[7].hq = 55;
        self.af[7].au[3]  = BeatStep::ea(42);
        self.af[7].au[7]  = BeatStep::ea(38);
        self.af[7].au[11] = BeatStep::ea(48);
        self.af[7].au[15] = BeatStep::ea(35);
        self.af[7].au[19] = BeatStep::ea(45);
        self.af[7].au[23] = BeatStep::ea(40);
        self.af[7].au[27] = BeatStep::ea(50);
        self.af[7].au[31] = BeatStep::ea(55);
    }

    
    
    
    
    
    pub fn uhq(&mut self) {
        self.fxi();

        
        self.af[0].hq = 160;
        self.af[0].au[0]  = BeatStep::bz(85, 0);     
        self.af[0].au[16] = BeatStep::bz(75, 0);     

        
        self.af[1].hq = 140;
        self.af[1].au[8]  = BeatStep::ea(90);
        self.af[1].au[24] = BeatStep::ea(85);

        
        self.af[2].hq = 72;
        self.af[2].au[0]  = BeatStep::ea(65);
        self.af[2].au[4]  = BeatStep::ea(55);
        self.af[2].au[8]  = BeatStep::ea(60);
        self.af[2].au[12] = BeatStep::ea(50);
        self.af[2].au[16] = BeatStep::ea(65);
        self.af[2].au[20] = BeatStep::ea(55);
        self.af[2].au[24] = BeatStep::ea(60);
        self.af[2].au[28] = BeatStep::ea(50);

        
        self.af[3].hq = 55;
        self.af[3].au[14] = BeatStep::ea(55);

        
        self.af[4].hq = 70;
        self.af[4].au[4]  = BeatStep::bz(55, 0);     
        self.af[4].au[12] = BeatStep::bz(48, -3);    
        self.af[4].au[20] = BeatStep::bz(52, 5);     
        self.af[4].au[28] = BeatStep::bz(45, 3);     

        
        self.af[5].hq = 55;
        self.af[5].au[0]  = BeatStep::bz(32, 0);     
        self.af[5].au[16] = BeatStep::bz(28, 7);     

        
        self.af[6].hq = 65;
        self.af[6].au[0]  = BeatStep::bz(55, 7);     
        self.af[6].au[12] = BeatStep::bz(48, 0);     

        
        self.af[7].hq = 35;
        self.af[7].au[7]  = BeatStep::ea(30);
        self.af[7].au[23] = BeatStep::ea(28);
    }

    
    
    
    
    
    pub fn uhl(&mut self) {
        self.fxi();

        
        self.af[0].hq = 235;
        self.af[0].au[0]  = BeatStep::bz(118, 0);    
        self.af[0].au[8]  = BeatStep::bz(105, 0);    
        self.af[0].au[16] = BeatStep::bz(115, 5);    
        self.af[0].au[20] = BeatStep::bz(95, 3);     
        self.af[0].au[24] = BeatStep::bz(110, 0);    
        self.af[0].au[28] = BeatStep::bz(88, -2);    

        
        self.af[1].hq = 175;
        self.af[1].au[8]  = BeatStep::ea(115);
        self.af[1].au[24] = BeatStep::ea(112);
        self.af[1].au[4]  = BeatStep::ea(28);
        self.af[1].au[12] = BeatStep::ea(35);
        self.af[1].au[20] = BeatStep::ea(30);

        
        self.af[2].hq = 90;
        
        self.af[2].au[0]  = BeatStep::ea(82);
        self.af[2].au[2]  = BeatStep::ea(40);
        self.af[2].au[4]  = BeatStep::ea(75);
        self.af[2].au[6]  = BeatStep::ea(38);
        self.af[2].au[8]  = BeatStep::ea(80);
        self.af[2].au[10] = BeatStep::ea(42);
        self.af[2].au[12] = BeatStep::ea(72);
        self.af[2].au[14] = BeatStep::ea(35);
        
        self.af[2].au[16] = BeatStep::ea(85);
        self.af[2].au[17] = BeatStep::ea(35);
        self.af[2].au[18] = BeatStep::ea(70);
        self.af[2].au[19] = BeatStep::ea(32);
        self.af[2].au[20] = BeatStep::ea(80);
        self.af[2].au[21] = BeatStep::ea(38);
        self.af[2].au[22] = BeatStep::ea(68);
        self.af[2].au[23] = BeatStep::ea(30);
        
        self.af[2].au[24] = BeatStep::ea(40);
        self.af[2].au[25] = BeatStep::ea(48);
        self.af[2].au[26] = BeatStep::ea(58);
        self.af[2].au[27] = BeatStep::ea(68);
        self.af[2].au[28] = BeatStep::ea(78);
        self.af[2].au[29] = BeatStep::ea(88);
        self.af[2].au[30] = BeatStep::ea(98);
        self.af[2].au[31] = BeatStep::ea(110);

        
        self.af[3].hq = 60;
        self.af[3].au[6]  = BeatStep::ea(60);
        self.af[3].au[14] = BeatStep::ea(55);
        self.af[3].au[22] = BeatStep::ea(58);

        
        self.af[4].hq = 85;
        self.af[4].au[0]  = BeatStep::bz(72, 0);     
        self.af[4].au[4]  = BeatStep::bz(58, 3);     
        self.af[4].au[8]  = BeatStep::bz(68, 7);     
        self.af[4].au[12] = BeatStep::bz(55, 5);     
        self.af[4].au[16] = BeatStep::bz(75, 0);     
        self.af[4].au[20] = BeatStep::bz(62, 3);     
        self.af[4].au[24] = BeatStep::bz(70, -2);    
        self.af[4].au[28] = BeatStep::bz(60, 0);     

        
        self.af[5].hq = 48;
        self.af[5].au[0]  = BeatStep::bz(35, 0);     
        self.af[5].au[8]  = BeatStep::bz(30, 5);     
        self.af[5].au[16] = BeatStep::bz(35, 3);     
        self.af[5].au[24] = BeatStep::bz(32, 7);     

        
        self.af[6].hq = 100;
        self.af[6].au[0]  = BeatStep::bz(75, 0);     
        self.af[6].au[4]  = BeatStep::bz(82, 3);     
        self.af[6].au[8]  = BeatStep::bz(90, 7);     
        self.af[6].au[16] = BeatStep::bz(80, 5);     
        self.af[6].au[20] = BeatStep::bz(88, 7);     
        self.af[6].au[24] = BeatStep::bz(95, 12);    

        
        self.af[7].hq = 48;
        self.af[7].au[3]  = BeatStep::ea(35);
        self.af[7].au[7]  = BeatStep::ea(32);
        self.af[7].au[11] = BeatStep::ea(40);
        self.af[7].au[15] = BeatStep::ea(30);
        self.af[7].au[19] = BeatStep::ea(38);
        self.af[7].au[23] = BeatStep::ea(35);
        self.af[7].au[27] = BeatStep::ea(42);
        self.af[7].au[31] = BeatStep::ea(48);
    }

    
    
    
    
    
    
    pub fn uhk(&mut self) {
        self.fxi();
        self.af[0].so = true;   
        self.af[1].so = true;   
        self.af[2].so = true;   
        self.af[3].so = true;   
        self.af[7].so = true;   

        
        self.af[5].hq = 55;
        self.af[5].au[0]  = BeatStep::bz(35, 0);     
        self.af[5].au[8]  = BeatStep::bz(28, 7);     
        self.af[5].au[16] = BeatStep::bz(32, -2);    
        self.af[5].au[24] = BeatStep::bz(28, 0);     

        
        self.af[4].hq = 45;
        self.af[4].au[8]  = BeatStep::bz(35, 0);     
        self.af[4].au[20] = BeatStep::bz(30, -3);    

        
        self.af[6].hq = 30;
        self.af[6].au[0]  = BeatStep::bz(28, 7);     
        self.af[6].au[16] = BeatStep::bz(25, 0);     
    }

    
    
    
    
    
    pub fn uhp(&mut self) {
        self.fxi();

        
        self.af[0].hq = 195;
        self.af[0].au[0]  = BeatStep::bz(100, 0);    
        self.af[0].au[8]  = BeatStep::bz(88, 0);     
        self.af[0].au[16] = BeatStep::bz(95, 5);     
        self.af[0].au[24] = BeatStep::bz(85, 0);     

        
        self.af[1].hq = 155;
        self.af[1].au[8]  = BeatStep::ea(105);
        self.af[1].au[24] = BeatStep::ea(100);
        self.af[1].au[7]  = BeatStep::ea(25);
        self.af[1].au[23] = BeatStep::ea(22);

        
        self.af[2].hq = 78;
        self.af[2].au[0]  = BeatStep::ea(72);
        self.af[2].au[2]  = BeatStep::ea(38);
        self.af[2].au[4]  = BeatStep::ea(65);
        self.af[2].au[6]  = BeatStep::ea(35);
        self.af[2].au[8]  = BeatStep::ea(70);
        self.af[2].au[10] = BeatStep::ea(36);
        self.af[2].au[12] = BeatStep::ea(62);
        self.af[2].au[14] = BeatStep::ea(33);
        self.af[2].au[16] = BeatStep::ea(72);
        self.af[2].au[18] = BeatStep::ea(38);
        self.af[2].au[20] = BeatStep::ea(65);
        self.af[2].au[22] = BeatStep::ea(35);
        self.af[2].au[24] = BeatStep::ea(70);
        self.af[2].au[26] = BeatStep::ea(40);
        self.af[2].au[28] = BeatStep::ea(65);
        self.af[2].au[30] = BeatStep::ea(50);

        
        self.af[3].hq = 55;
        self.af[3].au[6]  = BeatStep::ea(52);
        self.af[3].au[22] = BeatStep::ea(48);

        
        self.af[4].hq = 78;
        self.af[4].au[0]  = BeatStep::bz(65, 0);     
        self.af[4].au[4]  = BeatStep::bz(52, 3);     
        self.af[4].au[8]  = BeatStep::bz(60, 7);     
        self.af[4].au[16] = BeatStep::bz(68, 5);     
        self.af[4].au[20] = BeatStep::bz(55, 0);     
        self.af[4].au[24] = BeatStep::bz(62, 3);     

        
        self.af[5].hq = 45;
        self.af[5].au[0]  = BeatStep::bz(30, 0);     
        self.af[5].au[16] = BeatStep::bz(28, 5);     

        
        self.af[6].hq = 90;
        self.af[6].au[0]  = BeatStep::bz(78, 7);     
        self.af[6].au[4]  = BeatStep::bz(65, 3);     
        self.af[6].au[12] = BeatStep::bz(72, 0);     
        self.af[6].au[20] = BeatStep::bz(82, 7);     
        self.af[6].au[24] = BeatStep::bz(70, 5);     

        
        self.af[7].hq = 40;
        self.af[7].au[3]  = BeatStep::ea(28);
        self.af[7].au[11] = BeatStep::ea(35);
        self.af[7].au[19] = BeatStep::ea(30);
        self.af[7].au[27] = BeatStep::ea(38);
    }

    
    
    
    
    
    pub fn uhm(&mut self) {
        self.jdv();

        
        self.af[0].hq = 255;     
        self.af[1].hq = 195;     
        self.af[2].hq = 115;     
        self.af[3].hq = 78;      
        self.af[4].hq = 110;     
        self.af[5].hq = 52;      
        self.af[6].hq = 145;     
        self.af[7].hq = 65;      

        
        self.af[1].au[4]  = BeatStep::ea(55);
        self.af[1].au[12] = BeatStep::ea(50);
        self.af[1].au[20] = BeatStep::ea(58);

        
        self.af[2].au[1]  = BeatStep::ea(52);
        self.af[2].au[3]  = BeatStep::ea(48);

        
        self.af[6].au[8]  = BeatStep::bz(127, 12);   
        self.af[6].au[12] = BeatStep::bz(115, 12);   
    }

    
    
    
    
    
    pub fn uho(&mut self) {
        self.fxi();
        self.af[1].so = true;   
        self.af[2].so = true;   
        self.af[3].so = true;   
        self.af[7].so = true;   

        
        self.af[0].hq = 130;
        self.af[0].au[0]  = BeatStep::bz(55, 0);     

        
        self.af[5].hq = 35;
        self.af[5].au[0]  = BeatStep::bz(22, 0);     
        self.af[5].au[16] = BeatStep::bz(18, 7);     

        
        self.af[4].hq = 48;
        self.af[4].au[4]  = BeatStep::bz(32, 0);     
        self.af[4].au[14] = BeatStep::bz(25, -3);    
        self.af[4].au[22] = BeatStep::bz(28, 0);     

        
        self.af[6].hq = 45;
        self.af[6].au[0]  = BeatStep::bz(32, 7);     
        self.af[6].au[10] = BeatStep::bz(28, 0);     
        self.af[6].au[20] = BeatStep::bz(22, 0);     
    }

    
    pub fn zbg(&mut self) {
        self.jdv();
    }

    
    
    
    
    

    
    fn gvj(&mut self) {
        self.kz = 85;
        self.ezi = 58; 

        self.af[0] = BeatTrack::new("Kick",   36, Waveform::Dg,     colors::S_[0], true);   
        self.af[1] = BeatTrack::new("Snare",  38, Waveform::Cr,    colors::S_[1], true);
        self.af[2] = BeatTrack::new("HiHat",  54, Waveform::Cr,    colors::S_[2], true);
        self.af[3] = BeatTrack::new("Sub",    33, Waveform::Dg,     colors::S_[3], false);  
        self.af[4] = BeatTrack::new("Keys",   69, Waveform::Triangle, colors::S_[4], false);  
        self.af[5] = BeatTrack::new("Pad",    57, Waveform::Ft, colors::S_[5], false);  
        self.af[6] = BeatTrack::new("Lead",   81, Waveform::Gb,   colors::S_[6], false);  
        self.af[7] = BeatTrack::new("Perc",   60, Waveform::Cr,    colors::S_[7], true);

        for ab in self.af.el() {
            ab.aml = 32;
            for e in 0..FL_ { ab.au[e] = BeatStep::dz(); }
            ab.so = false;
        }

        
        self.af[0].qr = Envelope::new(2, 120, 0, 80);     
        self.af[1].qr = Envelope::new(1, 60, 0, 40);      
        self.af[2].qr = Envelope::new(1, 22, 0, 10);      
        self.af[3].qr = Envelope::new(2, 2000, 75, 800);  
        self.af[4].qr = Envelope::new(8, 500, 40, 350);   
        self.af[5].qr = Envelope::ov();                   
        self.af[6].qr = Envelope::new(6, 400, 55, 280);   
        self.af[7].qr = Envelope::new(1, 35, 0, 12);      
    }

    
    
    
    
    pub fn zbk(&mut self) {
        self.gvj();
        self.af[0].so = true;   
        self.af[1].so = true;   
        self.af[2].so = true;   
        self.af[7].so = true;   

        
        self.af[3].hq = 80;
        self.af[3].au[0]  = BeatStep::bz(50, 0);     

        
        self.af[5].hq = 55;
        self.af[5].au[0]  = BeatStep::bz(30, 0);     
        self.af[5].au[8]  = BeatStep::bz(25, 4);     
        self.af[5].au[16] = BeatStep::bz(28, 7);     
        self.af[5].au[24] = BeatStep::bz(25, 5);     

        
        self.af[4].hq = 40;
        self.af[4].au[4]  = BeatStep::bz(32, 0);     
        self.af[4].au[12] = BeatStep::bz(28, 4);     
        self.af[4].au[20] = BeatStep::bz(30, 7);     
        self.af[4].au[28] = BeatStep::bz(25, 10);    

        
        self.af[6].hq = 25;
        self.af[6].au[16] = BeatStep::bz(22, 7);     
    }

    
    
    
    
    
    pub fn zbm(&mut self) {
        self.gvj();
        self.af[1].so = true;    
        self.af[7].so = true;    

        
        self.af[0].hq = 145;
        self.af[0].au[0]  = BeatStep::ea(100);
        self.af[0].au[8]  = BeatStep::ea(85);
        self.af[0].au[16] = BeatStep::ea(95);
        self.af[0].au[24] = BeatStep::ea(80);

        
        self.af[2].hq = 65;
        for a in (0..32).akt(2) {
            self.af[2].au[a] = BeatStep::ea(60 + (a as u8 % 3) * 10);
        }
        
        self.af[2].au[3]  = BeatStep::ea(25);
        self.af[2].au[11] = BeatStep::ea(22);
        self.af[2].au[19] = BeatStep::ea(25);
        self.af[2].au[27] = BeatStep::ea(20);

        
        self.af[3].hq = 160;
        self.af[3].au[0]  = BeatStep::bz(105, 0);    
        self.af[3].au[8]  = BeatStep::bz(95, -4);    
        self.af[3].au[16] = BeatStep::bz(100, -9);   
        self.af[3].au[24] = BeatStep::bz(90, -2);    

        
        self.af[4].hq = 72;
        
        self.af[4].au[0]  = BeatStep::bz(68, 0);     
        self.af[4].au[2]  = BeatStep::bz(55, 4);     
        self.af[4].au[4]  = BeatStep::bz(62, 7);     
        
        self.af[4].au[8]  = BeatStep::bz(65, -4);    
        self.af[4].au[10] = BeatStep::bz(52, 0);     
        self.af[4].au[12] = BeatStep::bz(60, 4);     
        
        self.af[4].au[16] = BeatStep::bz(65, -9);    
        self.af[4].au[18] = BeatStep::bz(50, -5);    
        self.af[4].au[20] = BeatStep::bz(58, -2);    
        
        self.af[4].au[24] = BeatStep::bz(62, -2);    
        self.af[4].au[26] = BeatStep::bz(48, 2);     
        self.af[4].au[28] = BeatStep::bz(55, 5);     

        
        self.af[5].hq = 38;
        self.af[5].au[0]  = BeatStep::bz(28, 0);     
        self.af[5].au[16] = BeatStep::bz(26, -9);    

        
        self.af[6].hq = 50;
        self.af[6].au[0]  = BeatStep::bz(55, 0);     
        self.af[6].au[6]  = BeatStep::bz(48, 7);     
        self.af[6].au[12] = BeatStep::bz(52, 5);     
        self.af[6].au[18] = BeatStep::bz(50, 4);     
        self.af[6].au[24] = BeatStep::bz(48, 2);     
        self.af[6].au[30] = BeatStep::bz(42, 0);     
    }

    
    
    
    
    
    pub fn zbi(&mut self) {
        self.gvj();

        
        self.af[0].hq = 195;
        self.af[0].au[0]  = BeatStep::ea(125);
        self.af[0].au[6]  = BeatStep::ea(40);  
        self.af[0].au[8]  = BeatStep::ea(120);
        self.af[0].au[16] = BeatStep::ea(125);
        self.af[0].au[22] = BeatStep::ea(35);  
        self.af[0].au[24] = BeatStep::ea(115);

        
        self.af[1].hq = 150;
        self.af[1].au[8]  = BeatStep::ea(115);
        self.af[1].au[24] = BeatStep::ea(110);
        self.af[1].au[5]  = BeatStep::ea(28);  
        self.af[1].au[21] = BeatStep::ea(25);  

        
        self.af[2].hq = 75;
        for a in 0..32 {
            let bxr = if a % 4 == 0 { 80 } else if a % 2 == 0 { 55 } else { 30 };
            self.af[2].au[a] = BeatStep::ea(bxr);
        }

        
        self.af[7].hq = 45;
        self.af[7].au[4]  = BeatStep::ea(55);
        self.af[7].au[12] = BeatStep::ea(50);
        self.af[7].au[20] = BeatStep::ea(52);
        self.af[7].au[28] = BeatStep::ea(48);

        
        self.af[3].hq = 220;
        self.af[3].au[0]  = BeatStep::bz(120, 0);    
        self.af[3].au[4]  = BeatStep::bz(80, 0);     
        self.af[3].au[8]  = BeatStep::bz(115, 5);    
        self.af[3].au[16] = BeatStep::bz(118, -4);   
        self.af[3].au[20] = BeatStep::bz(75, -4);    
        self.af[3].au[24] = BeatStep::bz(110, -5);   
        self.af[3].au[28] = BeatStep::bz(90, -5);    

        
        self.af[4].hq = 90;
        
        self.af[4].au[0]  = BeatStep::bz(75, 0);     
        self.af[4].au[2]  = BeatStep::bz(60, 4);     
        self.af[4].au[4]  = BeatStep::bz(68, 7);     
        self.af[4].au[6]  = BeatStep::bz(50, 12);    
        
        self.af[4].au[8]  = BeatStep::bz(72, 5);     
        self.af[4].au[10] = BeatStep::bz(58, 8);     
        self.af[4].au[12] = BeatStep::bz(65, 12);    
        
        self.af[4].au[16] = BeatStep::bz(70, -4);    
        self.af[4].au[18] = BeatStep::bz(55, 0);     
        self.af[4].au[20] = BeatStep::bz(62, 4);     
        
        self.af[4].au[24] = BeatStep::bz(68, -5);    
        self.af[4].au[26] = BeatStep::bz(52, -1);    
        self.af[4].au[28] = BeatStep::bz(60, 2);     
        self.af[4].au[30] = BeatStep::bz(48, 5);     

        
        self.af[5].hq = 35;
        self.af[5].au[0]  = BeatStep::bz(30, 0);     
        self.af[5].au[16] = BeatStep::bz(28, -4);    

        
        self.af[6].hq = 95;
        self.af[6].au[0]  = BeatStep::bz(85, 7);     
        self.af[6].au[4]  = BeatStep::bz(72, 5);     
        self.af[6].au[8]  = BeatStep::bz(90, 4);     
        self.af[6].au[12] = BeatStep::bz(78, 2);     
        self.af[6].au[16] = BeatStep::bz(95, 0);     
        self.af[6].au[20] = BeatStep::bz(70, 4);     
        self.af[6].au[24] = BeatStep::bz(100, 7);    
        self.af[6].au[28] = BeatStep::bz(65, 5);     
    }

    
    
    
    
    pub fn zbh(&mut self) {
        self.gvj();
        self.af[0].so = true;   
        self.af[1].so = true;   
        self.af[7].so = true;   

        
        self.af[2].hq = 35;
        self.af[2].au[0]  = BeatStep::ea(40);
        self.af[2].au[8]  = BeatStep::ea(35);
        self.af[2].au[16] = BeatStep::ea(38);
        self.af[2].au[24] = BeatStep::ea(32);

        
        self.af[3].hq = 70;
        self.af[3].au[0]  = BeatStep::bz(55, 0);     
        self.af[3].au[16] = BeatStep::bz(50, 5);     

        
        self.af[4].hq = 60;
        self.af[4].au[0]  = BeatStep::bz(55, 5);     
        self.af[4].au[4]  = BeatStep::bz(45, 8);     
        self.af[4].au[8]  = BeatStep::bz(52, 0);     
        self.af[4].au[12] = BeatStep::bz(42, 4);     
        self.af[4].au[16] = BeatStep::bz(50, -5);    
        self.af[4].au[20] = BeatStep::bz(40, -1);    
        self.af[4].au[24] = BeatStep::bz(48, 0);     
        self.af[4].au[28] = BeatStep::bz(38, 7);     

        
        self.af[5].hq = 50;
        self.af[5].au[0]  = BeatStep::bz(32, 5);     
        self.af[5].au[16] = BeatStep::bz(30, 0);     

        
        self.af[6].hq = 42;
        self.af[6].au[8]  = BeatStep::bz(50, 12);    
        self.af[6].au[16] = BeatStep::bz(42, 7);     
        self.af[6].au[24] = BeatStep::bz(38, 5);     
    }

    
    
    
    
    pub fn zbj(&mut self) {
        self.gvj();

        
        self.af[0].hq = 210;
        self.af[0].au[0]  = BeatStep::ea(127);
        self.af[0].au[4]  = BeatStep::ea(45);  
        self.af[0].au[8]  = BeatStep::ea(122);
        self.af[0].au[12] = BeatStep::ea(40);  
        self.af[0].au[16] = BeatStep::ea(127);
        self.af[0].au[20] = BeatStep::ea(42);
        self.af[0].au[24] = BeatStep::ea(120);
        self.af[0].au[30] = BeatStep::ea(90);  

        
        self.af[1].hq = 165;
        self.af[1].au[8]  = BeatStep::ea(120);
        self.af[1].au[24] = BeatStep::ea(118);
        self.af[1].au[6]  = BeatStep::ea(30);
        self.af[1].au[22] = BeatStep::ea(28);
        
        self.af[1].au[28] = BeatStep::ea(55);
        self.af[1].au[29] = BeatStep::ea(65);
        self.af[1].au[30] = BeatStep::ea(80);
        self.af[1].au[31] = BeatStep::ea(100);

        
        self.af[2].hq = 82;
        for a in 0..32 {
            let bxr = if a % 4 == 0 { 85 } else if a % 2 == 0 { 60 } else { 35 };
            self.af[2].au[a] = BeatStep::ea(bxr);
        }

        
        self.af[7].hq = 55;
        self.af[7].au[2]  = BeatStep::ea(45);
        self.af[7].au[6]  = BeatStep::ea(55);
        self.af[7].au[14] = BeatStep::ea(50);
        self.af[7].au[18] = BeatStep::ea(42);
        self.af[7].au[26] = BeatStep::ea(52);
        self.af[7].au[30] = BeatStep::ea(60);

        
        self.af[3].hq = 245;
        self.af[3].au[0]  = BeatStep::bz(127, 0);    
        self.af[3].au[4]  = BeatStep::bz(85, 0);     
        self.af[3].au[8]  = BeatStep::bz(120, -4);   
        self.af[3].au[16] = BeatStep::bz(125, -2);   
        self.af[3].au[20] = BeatStep::bz(80, -2);    
        self.af[3].au[24] = BeatStep::bz(118, -5);   
        self.af[3].au[28] = BeatStep::bz(95, -5);    

        
        self.af[4].hq = 100;
        
        self.af[4].au[0]  = BeatStep::bz(80, 0);     
        self.af[4].au[1]  = BeatStep::bz(62, 4);     
        self.af[4].au[2]  = BeatStep::bz(72, 7);     
        self.af[4].au[4]  = BeatStep::bz(65, 12);    
        
        self.af[4].au[8]  = BeatStep::bz(78, -4);    
        self.af[4].au[9]  = BeatStep::bz(60, 0);     
        self.af[4].au[10] = BeatStep::bz(70, 4);     
        self.af[4].au[12] = BeatStep::bz(58, 8);     
        
        self.af[4].au[16] = BeatStep::bz(75, -2);    
        self.af[4].au[17] = BeatStep::bz(58, 2);     
        self.af[4].au[18] = BeatStep::bz(68, 5);     
        self.af[4].au[20] = BeatStep::bz(55, 10);    
        
        self.af[4].au[24] = BeatStep::bz(72, -5);    
        self.af[4].au[25] = BeatStep::bz(55, -1);    
        self.af[4].au[26] = BeatStep::bz(65, 2);     
        self.af[4].au[28] = BeatStep::bz(52, 5);     

        
        self.af[5].hq = 42;
        self.af[5].au[0]  = BeatStep::bz(32, 0);     
        self.af[5].au[8]  = BeatStep::bz(28, -4);    
        self.af[5].au[16] = BeatStep::bz(30, -2);    
        self.af[5].au[24] = BeatStep::bz(28, -5);    

        
        self.af[6].hq = 110;
        self.af[6].au[0]  = BeatStep::bz(95, 12);    
        self.af[6].au[3]  = BeatStep::bz(80, 7);     
        self.af[6].au[6]  = BeatStep::bz(88, 5);     
        self.af[6].au[8]  = BeatStep::bz(100, 4);    
        self.af[6].au[12] = BeatStep::bz(92, 0);     
        self.af[6].au[16] = BeatStep::bz(105, 7);    
        self.af[6].au[18] = BeatStep::bz(85, 5);     
        self.af[6].au[20] = BeatStep::bz(98, 4);     
        self.af[6].au[24] = BeatStep::bz(110, 12);   
        self.af[6].au[28] = BeatStep::bz(75, 7);     
    }

    
    
    
    
    pub fn zbl(&mut self) {
        self.gvj();
        self.af[0].so = true;   
        self.af[1].so = true;   
        self.af[2].so = true;   
        self.af[7].so = true;   

        
        self.af[3].hq = 60;
        self.af[3].au[0]  = BeatStep::bz(40, 0);     

        
        self.af[4].hq = 35;
        self.af[4].au[0]  = BeatStep::bz(35, 0);     
        self.af[4].au[8]  = BeatStep::bz(28, 4);     
        self.af[4].au[20] = BeatStep::bz(30, 7);     

        
        self.af[5].hq = 30;
        self.af[5].au[0]  = BeatStep::bz(22, 0);     
        self.af[5].au[16] = BeatStep::bz(18, 7);     

        
        self.af[6].hq = 40;
        self.af[6].au[0]  = BeatStep::bz(30, 7);     
        self.af[6].au[8]  = BeatStep::bz(25, 5);     
        self.af[6].au[16] = BeatStep::bz(22, 4);     
        self.af[6].au[26] = BeatStep::bz(18, 0);     
    }

    
    
    

    fn pwc(&self) -> u32 { 48 }

    fn fxg(&self) -> u32 { 120 }

    fn pgi(&self) -> u32 { self.gz.ao(self.fxg() + self.auk()).am(120) }

    fn auk(&self) -> u32 {
        
        let gtg = ((self.gz - self.fxg() - 120) / self.af[0].aml as u32).am(20).v(44);
        gtg * self.af[0].aml as u32
    }

    fn gtg(&self) -> u32 {
        self.auk() / self.af[0].aml as u32
    }

    fn mmr(&self) -> u32 { 32 }

    fn bhc(&self) -> u32 {
        
        24 + GP_ as u32 * self.mmr()
    }

    fn jom(&self) -> u32 { self.pwc() }
    fn whu(&self) -> u32 { self.fxg() }
    fn pgj(&self) -> u32 { self.fxg() + self.auk() }

    fn imd(&self) -> u32 { self.jom() + self.bhc() + 2 }
    fn qrl(&self) -> u32 { self.kc.ao(self.imd() + 48) }
    fn uo(&self) -> u32 { self.kc.ao(48) }

    
    
    

    
    pub fn po(&self) {
        if self.gz == 0 || self.kc == 0 { return; }

        
        crate::framebuffer::ah(0, 0, self.gz, self.kc, colors::JJ_);

        self.krn();
        self.sgi();
        self.sfs();
        self.sfi();
        self.sbx();
        self.hgw();
    }

    

    fn krn(&self) {
        let i = self.pwc();
        crate::framebuffer::ah(0, 0, self.gz, i, colors::XT_);

        
        crate::framebuffer::cb("TrustDAW Beat Studio", 8, 4, colors::PY_);

        
        let bx = 220;
        
        let wul = if !self.uu { colors::AIS_ } else { colors::AII_ };
        crate::framebuffer::ah(bx, 4, 14, 14, wul);

        
        let luf = if self.uu { colors::OY_ } else { colors::AII_ };
        crate::framebuffer::ah(bx + 22, 4, 4, 14, luf);
        crate::framebuffer::ah(bx + 26, 6, 3, 10, luf);
        crate::framebuffer::ah(bx + 29, 8, 2, 6, luf);

        
        let vtb = if self.ehe { colors::WN_ } else { colors::AII_ };
        crate::framebuffer::abc(bx + 52, 11, 6, vtb);

        
        let gbk = format!("BPM:{}", self.kz);
        crate::framebuffer::cb(&gbk, bx + 80, 4, colors::AIS_);

        
        let bar = self.aop / 16 + 1;
        let rf = (self.aop % 16) / 4 + 1;
        let sub = self.aop % 4 + 1;
        let dar = format!("{}:{}.{}", bar, rf, sub);
        crate::framebuffer::cb(&dar, bx + 160, 4, colors::OY_);

        
        let wwr = format!("Swing:{}%", self.ezi);
        crate::framebuffer::cb(&wwr, bx + 240, 4, colors::N_);

        
        let wuf = format!("{} steps", self.af[0].aml);
        crate::framebuffer::cb(&wuf, bx + 340, 4, colors::N_);

        
        crate::framebuffer::cb("Key: C# minor", 8, 24, colors::AV_);

        let xll = self.af[self.bdw].amj();
        let wgs = format!("Track: {} [{}]", xll, self.bdw);
        crate::framebuffer::cb(&wgs, 140, 24, colors::N_);

        let lps = format!("Oct:{}", 4i8 + self.cgg);
        crate::framebuffer::cb(&lps, 340, 24, colors::N_);

        let moy = format!("Vel:{}", self.qm);
        crate::framebuffer::cb(&moy, 420, 24, colors::N_);

        
        crate::framebuffer::zs(0, i - 1, self.gz, colors::ZN_);
    }

    

    fn sgi(&self) {
        let b = 0;
        let c = self.jom();
        let d = self.fxg();
        let ph = self.mmr();

        
        crate::framebuffer::ah(b, c, d, 24, colors::JK_);
        crate::framebuffer::cb("TRACKS", 8, c + 4, colors::AV_);
        crate::framebuffer::zs(b, c + 23, d, colors::Fj);

        for a in 0..GP_ {
            let ix = c + 24 + a as u32 * ph;
            let qe = a == self.bdw;

            
            let ei = if qe { colors::ALU_ } else { colors::HN_ };
            crate::framebuffer::ah(b, ix, d, ph, ei);

            
            crate::framebuffer::ah(b, ix, 4, ph, self.af[a].s);

            
            if qe {
                crate::framebuffer::cb(">", 6, ix + 8, colors::BGJ_);
            }

            
            let j = self.af[a].amj();
            let ure = if self.af[a].so { colors::AV_ }
                         else { colors::AC_ };
            crate::framebuffer::cb(j, 18, ix + 8, ure);

            
            if self.af[a].so {
                crate::framebuffer::cb("M", 82, ix + 8, colors::CHI_);
            }
            if self.af[a].cic {
                crate::framebuffer::cb("S", 96, ix + 8, colors::CTR_);
            }

            
            crate::framebuffer::zs(b, ix + ph - 1, d, colors::Aqo);
        }

        
        crate::framebuffer::axt(d - 1, c, self.bhc(), colors::Fj);
    }

    

    fn sfs(&self) {
        let qz = self.whu();
        let ub = self.jom();
        let kp = self.gtg();
        let ph = self.mmr();
        let orw = self.af[0].aml;

        
        crate::framebuffer::ah(qz, ub, self.auk(), 24, colors::JK_);
        for e in 0..orw {
            let cr = qz + e as u32 * kp;
            let ajh = format!("{}", e + 1);
            
            let s = if e % 4 == 0 { colors::AC_ } else { colors::AV_ };
            crate::framebuffer::cb(&ajh, cr + 2, ub + 4, s);

            
            if e % 4 == 0 && e > 0 {
                crate::framebuffer::axt(cr, ub, self.bhc(), colors::CUJ_);
            }
        }
        crate::framebuffer::zs(qz, ub + 23, self.auk(), colors::Fj);

        
        for ab in 0..GP_ {
            let ix = ub + 24 + ab as u32 * ph;

            for e in 0..orw {
                let cr = qz + e as u32 * kp;
                let gu = &self.af[ab].au[e];

                
                let lrr = cr + 2;
                let lrs = ix + 2;
                let lrq = kp.ao(4).am(4);
                let lrp = ph.ao(4).am(4);

                
                let txc = ab == self.bdw && e == self.bzc;
                let ogp = self.uu && e == self.aop;

                let var = if gu.gh {
                    if ogp {
                        colors::BGK_  
                    } else {
                        
                        let kt = gu.qm as u32 * 100 / 127;
                        jzn(self.af[ab].s, kt.am(40))
                    }
                } else if ogp {
                    colors::ALU_  
                } else {
                    colors::CUL_
                };

                
                crate::framebuffer::ah(lrr, lrs, lrq, lrp, var);

                
                if txc {
                    crate::framebuffer::lx(lrr.ao(1), lrs.ao(1),
                        lrq + 2, lrp + 2, colors::BGJ_);
                }

                
                crate::framebuffer::lx(lrr, lrs, lrq, lrp, colors::CUK_);
            }

            
            crate::framebuffer::zs(qz, ix + ph - 1, self.auk(), colors::Aqo);
        }

        
        if self.uu {
            let y = qz + self.aop as u32 * kp + kp / 2;
            crate::framebuffer::axt(y, ub, self.bhc(), colors::BGK_);
        }
    }

    

    fn sfi(&self) {
        let cr = self.pgj();
        let cq = self.jom();
        let kp = self.pgi();
        let kl = self.bhc();

        crate::framebuffer::ah(cr, cq, kp, kl, colors::CQN_);
        crate::framebuffer::axt(cr, cq, kl, colors::Fj);

        
        crate::framebuffer::ah(cr, cq, kp, 24, colors::JK_);
        crate::framebuffer::cb("SCOPE", cr + 8, cq + 4, colors::AV_);
        crate::framebuffer::zs(cr, cq + 23, kp, colors::Fj);

        
        let hza = cq + 24;
        let hyz = kl / 2 - 24;
        let uq = hza + hyz / 2;

        
        crate::framebuffer::zs(cr + 4, uq, kp - 8, colors::Aqo);

        
        let hgy = (kp - 8).v(256) as usize;
        for a in 1..hgy {
            let trt = (self.jnu + a - 1) % 256;
            let tru = (self.jnu + a) % 256;

            let dp = uq as i32 - (self.hyy[trt] as i32 * hyz as i32 / 2) / 32768;
            let jz = uq as i32 - (self.hyy[tru] as i32 * hyz as i32 / 2) / 32768;

            let egz = (dp.am(hza as i32) as u32).v(hza + hyz);
            let eha = (jz.am(hza as i32) as u32).v(hza + hyz);

            
            let onm = egz.v(eha);
            let csl = egz.am(eha);
            let gy = (csl - onm).am(1);
            crate::framebuffer::ah(cr + 4 + a as u32, onm, 1, gy, colors::CQO_);
        }

        
        let dcd = cq + kl / 2;
        let wqr = kl / 2;

        crate::framebuffer::zs(cr, dcd, kp, colors::Fj);
        crate::framebuffer::cb("SPECTRUM", cr + 8, dcd + 4, colors::AV_);

        let mxt = dcd + 20;
        let kbz = wqr - 24;
        let orl = 16usize;
        let lo = ((kp - 16) / orl as u32).am(4);

        for a in 0..orl {
            let bx = cr + 8 + a as u32 * lo;
            let jy = self.mgq[a].v(100) as u32;
            let kvz = kbz * jy / 100;
            let pl = mxt + kbz - kvz;

            
            let emn = if jy > 85 { colors::CTW_ }
                           else if jy > 70 { colors::CTV_ }
                           else if jy > 50 { colors::CTU_ }
                           else if jy > 30 { colors::CTT_ }
                           else { colors::CTS_ };

            
            crate::framebuffer::ah(bx, mxt, lo - 2, kbz, colors::VD_);

            
            if kvz > 0 {
                crate::framebuffer::ah(bx, pl, lo - 2, kvz, emn);
            }
        }
    }

    

    fn sbx(&self) {
        let je = self.imd();
        let adn = self.qrl();

        crate::framebuffer::zs(0, je, self.gz, colors::ZN_);

        
        self.sdx(0, je + 1, self.fxg(), adn);
        self.sgm(self.fxg(), je + 1, self.auk(), adn);
        self.krc(self.pgj(), je + 1, self.pgi(), adn);
    }

    

    fn sdx(&self, b: u32, c: u32, d: u32, i: u32) {
        crate::framebuffer::ah(b, c, d, i, colors::HN_);

        
        crate::framebuffer::cb("MIXER", b + 8, c + 4, colors::AV_);

        let sqt = i.ao(40);
        let alm = GP_;
        let nss = ((d - 8) / alm as u32).am(8);

        for a in 0..alm {
            let jf = b + 4 + a as u32 * nss;
            let sc = c + 24;

            
            let cfo = self.af[a].amj().bw().next().unwrap_or('?');
            let tue = format!("{}", cfo);
            crate::framebuffer::cb(&tue, jf + 2, sc, self.af[a].s);

            
            let kux = jf + 2;
            let kuy = sc + 18;
            let kuw = nss.ao(6).am(3);
            let ggt = sqt.ao(30);

            crate::framebuffer::ah(kux, kuy, kuw, ggt, colors::VD_);

            
            let jy = self.af[a].hq as u32 * ggt / 255;
            if jy > 0 {
                let uei = kuy + ggt - jy;
                let ueg = if self.af[a].so { colors::AV_ }
                    else if jy > ggt * 90 / 100 { colors::AFK_ }
                    else if jy > ggt * 70 / 100 { colors::AFL_ }
                    else { colors::AFJ_ };
                crate::framebuffer::ah(kux, uei, kuw, jy, ueg);
            }

            
            crate::framebuffer::lx(kux, kuy, kuw, ggt, colors::Fj);
        }

        
        crate::framebuffer::axt(b + d - 1, c, i, colors::Fj);
    }

    

    fn sgm(&self, b: u32, c: u32, d: u32, i: u32) {
        crate::framebuffer::ah(b, c, d, i, colors::RM_);

        
        crate::framebuffer::cb("KEYBOARD", b + 8, c + 4, colors::AV_);

        
        let fmg = c + 22;
        let oho = i.ao(26);
        let jwr = oho;
        let mzh = oho * 60 / 100;

        
        let lpp = 14u32;
        let diq = (d - 16) / lpp;
        let ohp = b + 8;

        let kcg = (4 + self.cgg) as u8;

        
        for a in 0..lpp {
            let jcd = ohp + a * diq;

            
            let jhp = a / 7;
            let gkp = a % 7;
            let mdn = match gkp {
                0 => 0,  
                1 => 2,  
                2 => 4,  
                3 => 5,  
                4 => 7,  
                5 => 9,  
                6 => 11, 
                _ => 0,
            };
            let ti = (kcg + jhp as u8) * 12 + mdn as u8;

            let eth = ti < 128 && self.dsl[ti as usize];
            let lhc = if eth { colors::AYB_ } else { colors::ADT_ };

            crate::framebuffer::ah(jcd, fmg, diq - 2, jwr, lhc);
            crate::framebuffer::lx(jcd, fmg, diq - 2, jwr, colors::Fj);

            
            let uvj = ["C", "D", "E", "F", "G", "A", "B"];
            if gkp < 7 {
                let cu = uvj[gkp as usize];
                let bbw = if eth { colors::AIS_ } else { colors::CDB_ };
                crate::framebuffer::cb(cu, jcd + diq / 2 - 4, fmg + jwr - 18, bbw);
            }

            
            if gkp == 0 {
                let lps = format!("{}", kcg + jhp as u8);
                crate::framebuffer::cb(&lps, jcd + 2, fmg + 2, colors::AV_);
            }
        }

        
        for a in 0..lpp {
            let jhp = a / 7;
            let gkp = a % 7;

            
            let mdn = match gkp {
                0 => Some(1),  
                1 => Some(3),  
                
                3 => Some(6),  
                4 => Some(8),  
                5 => Some(10), 
                _ => None,
            };

            if let Some(grz) = mdn {
                let ti = (kcg + jhp as u8) * 12 + grz as u8;
                let eth = ti < 128 && self.dsl[ti as usize];

                let bx = ohp + a * diq + diq * 2 / 3;
                let nm = diq * 2 / 3;
                let lhc = if eth { colors::AYB_ } else { colors::ADS_ };

                crate::framebuffer::ah(bx, fmg, nm, mzh, lhc);
                crate::framebuffer::lx(bx, fmg, nm, mzh, colors::Fj);
            }
        }

        
        let fmn = fmg + jwr + 2;
        if fmn + 16 < c + i {
            crate::framebuffer::cb("[Z X C V B N M] Low  [Q W E R T Y U] High", b + 8, fmn, colors::AV_);
        }
    }

    

    fn krc(&self, b: u32, c: u32, d: u32, i: u32) {
        crate::framebuffer::ah(b, c, d, i, colors::HN_);
        crate::framebuffer::axt(b, c, i, colors::Fj);

        crate::framebuffer::cb("INFO", b + 8, c + 4, colors::AV_);

        let mut ct = c + 24;
        let gy = 18u32;

        
        let ab = &self.af[self.bdw];
        let amj = format!("Track: {}", ab.amj());
        crate::framebuffer::cb(&amj, b + 8, ct, colors::AC_);
        ct += gy;

        let xua = format!("Wave: {}", ab.ve.j());
        crate::framebuffer::cb(&xua, b + 8, ct, colors::N_);
        ct += gy;

        let bde = if ab.jbg { "Type: Drum" } else { "Type: Melodic" };
        crate::framebuffer::cb(bde, b + 8, ct, colors::N_);
        ct += gy;

        let bkp = crate::audio::tables::dtf(ab.fdc);
        let uvk = crate::audio::tables::efk(ab.fdc);
        let uvn = format!("Note: {}{}", bkp, uvk);
        crate::framebuffer::cb(&uvn, b + 8, ct, colors::N_);
        ct += gy;

        let coh = format!("Steps: {}/{}", ab.gxu(), ab.aml);
        crate::framebuffer::cb(&coh, b + 8, ct, colors::N_);
        ct += gy;

        let igu = format!("Vol: {}", ab.hq);
        crate::framebuffer::cb(&igu, b + 8, ct, colors::N_);
        ct += gy;

        let lry = if ab.arp == 0 { String::from("Pan: C") }
                     else if ab.arp > 0 { format!("Pan: R{}", ab.arp) }
                     else { format!("Pan: L{}", -ab.arp) };
        crate::framebuffer::cb(&lry, b + 8, ct, colors::N_);
        ct += gy + 8;

        
        let rrq = format!("Step: {}/{}", self.bzc + 1, ab.aml);
        crate::framebuffer::cb(&rrq, b + 8, ct, colors::PY_);
        ct += gy;

        
        let gu = &ab.au[self.bzc];
        if gu.gh {
            let moy = format!("Hit Vel: {}", gu.qm);
            crate::framebuffer::cb(&moy, b + 8, ct, colors::AC_);
        } else {
            crate::framebuffer::cb("Hit: ---", b + 8, ct, colors::AV_);
        }
    }

    

    fn hgw(&self) {
        let cq = self.uo();
        crate::framebuffer::ah(0, cq, self.gz, 48, colors::XT_);
        crate::framebuffer::zs(0, cq, self.gz, colors::ZN_);

        crate::framebuffer::cb(
            "[Space] Play/Stop  [Enter] Toggle Step  [R] Record  [Tab] Track  [+/-] BPM",
            8, cq + 6, colors::AV_
        );
        crate::framebuffer::cb(
            "[Arrows] Navigate  [Z-M] Low Piano  [Q-P] High Piano  [F8] Export  [Esc] Exit",
            8, cq + 24, colors::AV_
        );
    }

    
    
    

    
    
    
    pub fn ehn(&self) -> Vec<i16> {
        let dwk = (60 * BR_) / (self.kz as u32 * 4); 
        let tk = self.af[0].aml;
        let agc = dwk as usize * tk;
        let ayz = agc * Dv as usize;

        let mut bno = vec![0i32; ayz];

        
        let mut rng: u32 = 0xCAFE_B0BA;

        
        let qjc = self.af.iter().any(|ab| ab.cic);

        for (zx, ab) in self.af.iter().cf() {
            if ab.so { continue; }
            if qjc && !ab.cic { continue; }

            let mut engine = SynthEngine::new();
            engine.dvs(ab.ve);
            engine.qr = ab.qr;

            let api = ab.hq as i32;

            
            let mut e = 0usize;
            while e < tk {
                let gu = &ab.au[e];
                if !gu.gh {
                    e += 1;
                    continue;
                }

                let ayg = ab.lov(e);
                if ayg == 0 { e += 1; continue; }

                
                let orc = if ab.jbg {
                    1usize
                } else {
                    let mut dqi = 1usize;
                    while e + dqi < tk && !ab.au[e + dqi].gh {
                        dqi += 1;
                    }
                    dqi
                };

                
                let qnq = e * dwk as usize;
                rng ^= rng << 13; rng ^= rng >> 17; rng ^= rng << 5;
                let uae = ((rng % 769) as i32 - 384) as isize; 
                let uvm = (qnq as isize + uae).am(0) as usize;

                let uvg = dwk as usize * orc;

                let bxr = gu.qm;
                let uvl = engine.lzf(ayg, bxr,
                    (uvg as u32 * 1000) / BR_);

                for (fb, &yr) in uvl.iter().cf() {
                    let w = uvm * Dv as usize + fb;
                    if w < bno.len() {
                        bno[w] += (yr as i32 * api) / 255;
                    }
                }

                e += orc;
            }
        }

        
        let hft = (dwk as usize * 3) * Dv as usize;
        let ntb = 50i32;

        if hft > 0 && hft < bno.len() {
            for a in hft..bno.len() {
                let kot = bno[a - hft];
                bno[a] += (kot * ntb) / 100;
            }
            let mju = hft * 2;
            if mju < bno.len() {
                for a in mju..bno.len() {
                    let kot = bno[a - mju];
                    bno[a] += (kot * ntb / 3) / 100;
                }
            }
        }

        
        let mut jhd: u16 = 0xACE1;
        for yr in bno.el() {
            let ga = jhd & 1;
            jhd >>= 1;
            if ga == 1 { jhd ^= 0xB400; }
            let bnq = (jhd as i16 as i32) / 180; 
            *yr += bnq;
        }

        
        bno.iter().map(|&e| {
            let e = e.qp(-48000, 48000);
            if e > 24000 {
                (24000 + (e - 24000) / 3) as i16
            } else if e < -24000 {
                (-24000 + (e + 24000) / 3) as i16
            } else {
                e as i16
            }
        }).collect()
    }

    
    pub fn dwh(&self) -> u32 {
        60_000 / (self.kz as u32 * 4) 
    }

    
    pub fn fat(&mut self, un: &[i16]) {
        
        let gu = (un.len() / 256).am(1);
        for a in 0..256 {
            let w = (a * gu).v(un.len().ao(1));
            self.hyy[a] = un[w];
        }
        self.jnu = 0;
    }

    
    pub fn fxu(&mut self) {
        
        for a in 0..16 {
            let mut jy: u32 = 0;
            for ab in &self.af {
                if !ab.so && self.aop < ab.aml {
                    let gu = &ab.au[self.aop];
                    if gu.gh {
                        
                        let bti = (ab.fdc as u32 / 8).v(15);
                        let eoy = (bti as i32 - a as i32).eki();
                        if eoy < 4 {
                            jy += gu.qm as u32 * (4 - eoy) / 4;
                        }
                    }
                }
            }
            self.mgq[a] = jy.v(100) as u8;
        }
    }

    
    pub fn xmh(&mut self, ti: u8, qm: u8) {
        if ti < 128 {
            self.dsl[ti as usize] = true;
        }
        
        let azd = self.af[self.bdw].ve;
        let _ = crate::audio::dvs(azd);
        let _ = crate::audio::owb(ti, qm, 150);
    }

    
    pub fn vum(&mut self, ti: u8) {
        if ti < 128 {
            self.dsl[ti as usize] = false;
        }
    }
}






pub fn ucu() -> Result<(), &'static str> {
    
    crate::audio::init().bq(); 

    let mut er = BeatStudio::new();
    er.po();

    crate::serial_println!("[BEAT_STUDIO] Launched — press Esc to exit");

    oih(&mut er)
}


pub fn ucw() -> Result<(), &'static str> {
    crate::audio::init().bq();

    let mut er = BeatStudio::new();
    er.ljh();
    er.po();

    crate::serial_println!("[BEAT_STUDIO] Funky House loaded — press Esc to exit");

    oih(&mut er)
}


fn oih(er: &mut BeatStudio) -> Result<(), &'static str> {
    loop {
        if let Some(scancode) = crate::keyboard::xw() {
            let bep = scancode & 0x80 != 0;
            let vla = scancode & 0x7F;

            
            if bep {
                if let Some(ayg) = super::keyboard_midi::hyv(vla) {
                    er.vum(ayg);
                    er.po();
                }
                continue;
            }

            let mut paz = true;

            match scancode {
                
                0x01 => break,

                
                0x39 => {
                    if er.uu {
                        er.uu = false;
                        er.aop = 0;
                        let _ = crate::audio::qg();
                    } else {
                        er.uu = true;
                        
                        let audio = er.ehn();
                        er.fat(&audio);
                        let _ = crate::drivers::hda::dcg(&audio);
                        
                        qit(er);
                    }
                }

                
                0x1C => {
                    er.af[er.bdw].xiy(er.bzc);
                }

                
                0x0F => {
                    er.bdw = (er.bdw + 1) % GP_;
                }

                
                0x4D => { 
                    let am = er.af[er.bdw].aml;
                    er.bzc = (er.bzc + 1) % am;
                }
                0x4B => { 
                    let am = er.af[er.bdw].aml;
                    if er.bzc == 0 {
                        er.bzc = am - 1;
                    } else {
                        er.bzc -= 1;
                    }
                }
                0x50 => { 
                    er.bdw = (er.bdw + 1) % GP_;
                }
                0x48 => { 
                    if er.bdw == 0 {
                        er.bdw = GP_ - 1;
                    } else {
                        er.bdw -= 1;
                    }
                }

                
                0x0D => { 
                    er.kz = (er.kz + 5).v(300);
                }
                0x0C => { 
                    er.kz = er.kz.ao(5).am(40);
                }

                
                0x49 => { 
                    er.cgg = (er.cgg + 1).v(4);
                }
                0x51 => { 
                    er.cgg = (er.cgg - 1).am(-4);
                }

                
                0x32 => {
                    let aqx = er.bdw;
                    er.af[aqx].so = !er.af[aqx].so;
                }

                
                0x3B => {
                    let aqx = er.bdw;
                    er.af[aqx].ve = match er.af[aqx].ve {
                        Waveform::Dg => Waveform::Gb,
                        Waveform::Gb => Waveform::Ft,
                        Waveform::Ft => Waveform::Triangle,
                        Waveform::Triangle => Waveform::Cr,
                        Waveform::Cr => Waveform::Dg,
                    };
                }

                
                0x3C => {
                    for ab in er.af.el() {
                        ab.aml = if ab.aml == 16 { 32 } else { 16 };
                    }
                    if er.bzc >= er.af[0].aml {
                        er.bzc = 0;
                    }
                }

                
                0x42 => {
                    let audio = er.ehn();
                    let _ = super::wav_export::hio(
                        "/home/beat.wav", &audio, BR_, Dv as u16
                    );
                    crate::serial_println!("[BEAT_STUDIO] Exported to /home/beat.wav");
                }

                
                0x13 => {
                    er.ehe = !er.ehe;
                }

                
                0x0E => {
                    let aqx = er.bdw;
                    for e in 0..er.af[aqx].aml {
                        er.af[aqx].au[e] = BeatStep::dz();
                    }
                }

                
                _ => {
                    if let Some(ayg) = super::keyboard_midi::hyv(scancode) {
                        er.xmh(ayg, er.qm);

                        
                        if er.ehe {
                            let aqx = er.bdw;
                            let aap = er.bzc;
                            let ar = er.af[aqx].fdc;
                            let l = ayg as i8 - ar as i8;
                            er.af[aqx].au[aap] = BeatStep::bz(er.qm, l);
                            
                            let am = er.af[aqx].aml;
                            er.bzc = (er.bzc + 1) % am;
                        }
                    } else {
                        paz = false;
                    }
                }
            }

            if paz {
                er.fxu();
                er.po();
            }
        }

        
        for _ in 0..3000 {
            core::hint::hc();
        }
    }

    
    let _ = crate::audio::qg();
    crate::serial_println!("[BEAT_STUDIO] Exited");
    Ok(())
}


fn qit(er: &mut BeatStudio) {
    let tk = er.af[0].aml;
    let bop = er.dwh();

    for e in 0..tk {
        er.aop = e;
        er.fxu();
        er.po();

        
        match eks(bop as u64) {
            1 | 2 => {  
                er.uu = false;
                er.aop = 0;
                let _ = crate::audio::qg();
                return;
            }
            _ => {}
        }
    }

    if er.uu {
        
        er.uu = false;
        er.aop = 0;
    }
}






fn jzn(s: u32, kt: u32) -> u32 {
    let m = ((s >> 16) & 0xFF) * kt / 100;
    let at = ((s >> 8) & 0xFF) * kt / 100;
    let o = (s & 0xFF) * kt / 100;
    (m.v(255) << 16) | (at.v(255) << 8) | o.v(255)
}













struct Blz {
    
    buu: i32,
    
    ig: u8,
    
    acr: u8,
    
    gh: bool,
    
    hcq: u8,
    
    ceq: u8,
}


struct MatrixState {
    cpm: Vec<Blz>,
    ajg: usize,
    bnr: usize,
    gz: u32,
    kc: u32,
    
    frame: u32,
    
    cam: u32,
}

impl MatrixState {
    
    
    fn new() -> Self {
        let gz = crate::framebuffer::AB_.load(Ordering::Relaxed) as u32;
        let kc = crate::framebuffer::Z_.load(Ordering::Relaxed) as u32;

        const R_: usize = 160;
        let bnr = (kc / 16) as usize;

        let mut cpm = Vec::fc(R_);
        let mut cam: u32 = 0xDEAD_BEEF;

        for a in 0..R_ {
            
            let dv = (a as u32).hx(2654435761) ^ 0xDEADBEEF;
            cam = lin(cam);
            let ig = (dv % 3) as u8 + 1;        
            let ase = 30u8;                         
            let vc = -((dv % (bnr as u32 / 2)) as i32);
            let qyg = (dv.hx(7919) % 94) as u8;

            cpm.push(Blz {
                buu: vc,
                ig,
                acr: ase,
                gh: true,
                hcq: qyg,
                ceq: 100,
            });
        }

        Self {
            cpm,
            ajg: R_,
            bnr,
            gz,
            kc,
            frame: 0,
            cam,
        }
    }

    
    fn or(&mut self) {
        self.frame += 1;

        for (a, bj) in self.cpm.el().cf() {
            bj.buu += bj.ig as i32;

            
            let xks = bj.acr as i32 * 16;
            if bj.buu > (self.kc as i32 + xks) {
                let dv = (a as u32).hx(1103515245).cn(self.frame);
                bj.buu = -((dv % (self.kc / 2)) as i32);
                bj.ig = (dv % 3) as u8 + 1;
                bj.hcq = ((dv.hx(7919)) % 94) as u8;
            }
        }
    }

    
    fn nuz(&mut self, hj: u8) {
        
        let az = (self.ajg * hj as usize / 255).am(3);
        for _ in 0..az {
            self.cam = lin(self.cam);
            let adq = (self.cam as usize) % self.ajg;
            self.cpm[adq].ceq = 255;
            self.cpm[adq].gh = true;
            self.cpm[adq].buu = 0;
            self.cam = lin(self.cam);
            self.cpm[adq].ig = (self.cam % 3) as u8 + 3; 
        }
    }

    
    fn po(&self, gu: usize, tk: usize, pvq: &str, kz: u16, qmu: &str) {
        
        crate::framebuffer::ah(0, 0, self.gz, self.kc, 0x000000);

        
        const Pm: &[u8] = b"@#$%&*0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz!?<>{}[]|/\\~^";

        
        for (adq, bj) in self.cpm.iter().cf() {
            if !bj.gh { continue; }

            let b = adq as u32 * 8;

            for afg in 0..(bj.acr as i32 + 1) {
                let br = bj.buu - afg;
                if br < 0 || br >= self.bnr as i32 { continue; }

                let c = br as u32 * 16;

                
                let gcl = if afg == 0 {
                    
                    ((bj.hcq as u32 + self.frame * 3 + adq as u32) % Pm.len() as u32) as usize
                } else {
                    
                    ((bj.hcq as u32 + br as u32 * 7 + adq as u32 * 13) % Pm.len() as u32) as usize
                };
                let bm = Pm[gcl] as char;

                
                let kt = if afg == 0 {
                    
                    255u32
                } else {
                    
                    let yx = 255u32.ao(afg as u32 * 255 / bj.acr as u32);
                    yx.am(20)
                };

                
                let sup = bj.ceq as u32;
                let hhv = (kt * sup / 100).v(255);

                
                let m = if afg == 0 { hhv * 80 / 100 } else { hhv * 10 / 100 };
                let at = hhv;
                let o = if afg == 0 { hhv * 60 / 100 } else { hhv * 20 / 100 };
                let s = ((m.v(255)) << 16) | ((at.v(255)) << 8) | o.v(255);

                crate::framebuffer::afn(b, c, bm, s);
            }
        }

        
        let pl = self.kc - 32;
        let tn = 8;
        let lo = self.gz - 40;
        let ajx = 20;

        
        crate::framebuffer::ah(ajx, pl, lo, tn, 0x002200);
        
        crate::framebuffer::lx(ajx, pl, lo, tn, 0x00AA00);

        
        if tk > 0 {
            let adu = lo * gu as u32 / tk as u32;
            crate::framebuffer::ah(ajx + 1, pl + 1, adu, tn - 2, 0x00FF44);

            
            for a in 1..tk {
                if a % 4 == 0 {
                    let hl = ajx + lo * a as u32 / tk as u32;
                    crate::framebuffer::axt(hl, pl, tn, 0x00CC00);
                }
            }
        }

        
        let dq = "TRUSTDAW // BEAT MATRIX";
        let dcs = dq.len() as u32 * 8 + 16;
        let cnf = (self.gz - dcs) / 2;
        crate::framebuffer::ah(cnf, 8, dcs, 24, 0x001100);
        crate::framebuffer::lx(cnf, 8, dcs, 24, 0x00CC00);
        crate::framebuffer::cb(dq, cnf + 8, 12, 0x00FF66);

        
        let edf = 40;
        let izw = pvq.len() as u32 * 8 + 16;
        crate::framebuffer::ah(8, edf, izw.v(self.gz - 16), 20, 0x000800);
        crate::framebuffer::cb(pvq, 16, edf + 2, 0x00AA44);

        
        let gbk = format!("{} BPM  {}", kz, qmu);
        let mzz = gbk.len() as u32 * 8 + 16;
        let naa = self.gz - mzz - 8;
        crate::framebuffer::ah(naa, edf, mzz, 20, 0x000800);
        crate::framebuffer::cb(&gbk, naa + 8, edf + 2, 0x00CC66);

        
        let pov = format!("{:02}/{:02}", gu + 1, tk);
        let gtg = pov.len() as u32 * 8 + 12;
        let dcj = (self.gz - gtg) / 2;
        let ejd = pl - 24;
        crate::framebuffer::ah(dcj, ejd, gtg, 20, 0x001100);
        crate::framebuffer::cb(&pov, dcj + 6, ejd + 2, 0x44FF88);

        
        let gjv = 70;
        let dxb = ["Ki", "Cl", "HH", "SB", "MB", "Ch", "Ld", "Pc"];
        for (a, j) in dxb.iter().cf() {
            let ty = gjv + a as u32 * 20;
            let s = if a < 8 { colors::S_[a] } else { 0x00FF00 };
            crate::framebuffer::cb(j, 8, ty, s);
        }
    }

    
    
    fn eba(&self) {
        crate::framebuffer::ah(0, 0, self.gz, self.kc, 0xFF000000);

        const Pm: &[u8] = b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ@#$%&*+=<>[]{}|";
        const CYL_: i32 = 30;
        const BMK_: i32 = 16;

        let byt = self.gz / self.ajg as u32;

        for (adq, bj) in self.cpm.iter().cf() {
            let b = (adq as u32 * byt) + byt / 2;
            let buu = bj.buu;

            
            let nkn = ((bj.ig as u32).ao(1)) * 50; 
            let fdr = 30 + nkn * 70 / 100; 
            let pfj = 20 + nkn * 80 / 100;      

            for a in 0..CYL_ {
                let avl = buu - (a * BMK_);
                if avl < 0 || avl >= self.kc as i32 { continue; }

                
                let ar: u32 = if a == 0 { 255 }
                    else if a == 1 { 200 }
                    else { 160u32.ao(a as u32 * 7) };
                if ar < 15 { continue; }

                let kt = ar * fdr / 100;

                
                let (m, at, o) = if a == 0 {
                    
                    let d = 140 * fdr / 100;
                    (d, kt.am(d), d)
                } else {
                    
                    let thk = (15 * (100 - pfj) / 100).v(40);
                    let qqn = (30 * (100 - pfj) / 100).v(50);
                    (thk, kt, qqn)
                };

                let s = ((m.v(255)) << 16) | ((at.v(255)) << 8) | o.v(255);

                
                let des = bj.hcq as u32
                    + (a as u32 * 7919)
                    ^ (self.frame / 12);
                let bm = Pm[(des as usize) % Pm.len()] as char;

                crate::framebuffer::afn(b, avl as u32, bm, s);
            }
        }
    }
}


pub fn ucy() -> Result<(), &'static str> {
    crate::audio::init().bq();

    let mut er = BeatStudio::new();
    er.ljh();

    let mut matrix = MatrixState::new();

    
    matrix.po(0, er.af[0].aml, "> INITIALIZING BEAT MATRIX...", er.kz, "1:1.1");

    
    let audio = er.ehn();
    er.fat(&audio);

    let tk = er.af[0].aml;
    let bop = er.dwh();
    let zth = bop * tk as u32;

    crate::serial_println!("[MATRIX] Funky House: {} BPM, {} steps, {}ms per step", er.kz, tk, bop);

    
    for bb in 0..30 {
        matrix.or();
        let lfj = match bb {
            0..=5   => "> LOADING BEAT DATA...",
            6..=12  => "> DECODING FREQUENCY MATRIX...",
            13..=20 => "> SYNTH ENGINES ONLINE...",
            _       => "> READY. ENTERING THE BEAT.",
        };
        matrix.po(0, tk, lfj, er.kz, "---");

        if cnw(100) { return Ok(()); } 
    }

    
    let ulm = 4u32;

    
    let _ = crate::drivers::hda::dcg(&audio);

    'outer: for yaq in 0..ulm {

        
        for e in 0..tk {
            er.aop = e;

            
            for ab in 0..8 {
                if er.af[ab].au[e].gh && !er.af[ab].so {
                    let bxr = er.af[ab].au[e].qm;
                    matrix.nuz(bxr);
                }
            }

            
            let mut coh = String::from("> ");
            for ab in 0..8 {
                if er.af[ab].au[e].gh && !er.af[ab].so {
                    coh.t(er.af[ab].amj());
                    coh.push(' ');
                }
            }
            if coh.len() <= 2 {
                coh.t("...");
            }

            
            let bar = e / 16 + 1;
            let rf = (e % 16) / 4 + 1;
            let sub = e % 4 + 1;
            let dar = format!("{}:{}.{}", bar, rf, sub);

            
            matrix.or();
            matrix.po(e, tk, &coh, er.kz, &dar);

            
            match eks(bop as u64) {
                1 | 2 => { break 'outer; } 
                _ => {}
            }
        }
    }

    
    let _ = crate::audio::qg();

    for bb in 0..40 {
        matrix.or();
        let lrd = match bb {
            0..=10  => "> DISCONNECTING...",
            11..=25 => "> SIGNAL LOST",
            _       => "> TRUSTDAW // SYSTEM OFFLINE",
        };
        matrix.po(0, tk, lrd, er.kz, "---");

        
        let hfk = matrix.ajg / 40;
        for r in 0..hfk {
            let w = (bb as usize * hfk + r) % matrix.ajg;
            matrix.cpm[w].gh = false;
        }

        crate::cpu::tsc::asq(80); 
    }

    
    crate::framebuffer::ah(0, 0, matrix.gz, matrix.kc, 0x000000);
    let nua = "TRUSTDAW BEAT MATRIX // BUILT ON TRUSTOS";
    let ua = nua.len() as u32 * 8;
    let jf = (matrix.gz - ua) / 2;
    let sc = matrix.kc / 2 - 8;
    crate::framebuffer::cb(nua, jf, sc, 0x00FF44);

    let ppk = "Bare-metal. No OS. Pure Rust.";
    let kp = ppk.len() as u32 * 8;
    let cr = (matrix.gz - kp) / 2;
    crate::framebuffer::cb(ppk, cr, sc + 24, 0x008822);

    
    loop {
        if let Some(jt) = crate::keyboard::xw() {
            if jt & 0x80 == 0 { break; }
        }
        crate::cpu::tsc::asq(20);
    }

    crate::serial_println!("[MATRIX] Showcase complete");
    Ok(())
}




















struct Gz {
    dq: &'static str,
    atp: &'static str,
    eu: &'static str,
    vj: u32,
}


fn epe(btp: &Gz, gz: u32, kc: u32, ib: &str, li: u32, es: u32) {
    
    let del = 200u32;
    let bjk = kc.ao(del + 52); 
    let btm = 16u32;
    let dom = gz.ao(32);

    crate::framebuffer::ih(btm, bjk, dom, del, 0x000000, 230);

    
    crate::framebuffer::lx(btm, bjk, dom, del, 0x00EEFF);
    crate::framebuffer::lx(btm + 1, bjk + 1, dom - 2, del - 2, 0x00EEFF);

    
    let bv = 2u32;
    let fg = (btm + 16) as i32;

    
    crate::graphics::scaling::azp(fg, (bjk + 12) as i32, ib, 0x00DDFF, bv);

    
    crate::graphics::scaling::azp(fg, (bjk + 48) as i32, btp.dq, 0xFFFFFF, bv);

    
    crate::graphics::scaling::azp(fg, (bjk + 88) as i32, btp.atp, 0x55FF99, bv);

    
    crate::graphics::scaling::azp(fg, (bjk + 128) as i32, btp.eu, 0xAADDFF, bv);

    
    let ewi = btm + 16;
    let ewj = bjk + del - 20;
    let fqm = dom - 32;
    let ewh = 8u32;
    crate::framebuffer::ah(ewi, ewj, fqm, ewh, 0x112233);
    if es > 0 {
        let adu = fqm * li / es;
        crate::framebuffer::ah(ewi, ewj, adu, ewh, 0x00EEFF);
    }
}


fn dgs(gz: u32, kc: u32, ojc: &str, ojd: &str, oje: &str, mm: u32) {
    crate::framebuffer::ah(0, 0, gz, kc, 0x050510);

    let bv = 2u32;
    let nk = 8 * bv; 

    
    let bkl = kc / 2;
    crate::framebuffer::ah(0, bkl - 80, gz, 2, mm);
    crate::framebuffer::ah(0, bkl + 80, gz, 2, mm);

    
    let blt = ojc.len() as u32 * nk;
    crate::graphics::scaling::azp(
        ((gz.ao(blt)) / 2) as i32, (bkl - 52) as i32,
        ojc, 0xFFFFFF, bv);

    
    let bfs = ojd.len() as u32 * nk;
    crate::graphics::scaling::azp(
        ((gz.ao(bfs)) / 2) as i32, (bkl - 10) as i32,
        ojd, mm, bv);

    
    let bxu = oje.len() as u32 * nk;
    crate::graphics::scaling::azp(
        ((gz.ao(bxu)) / 2) as i32, (bkl + 36) as i32,
        oje, 0x99AABB, bv);
}



fn cnw(alu: u64) -> bool {
    
    let jj = 50u64; 
    let mut ia = alu;
    while ia > 0 {
        let bmv = ia.v(jj);
        crate::cpu::tsc::asq(bmv);
        ia -= bmv;
        
        while let Some(jt) = crate::keyboard::xw() {
            if jt & 0x80 != 0 { continue; }
            if jt == 0x01 { return true; } 
        }
    }
    false
}



fn eks(alu: u64) -> u8 {
    let jj = 50u64;
    let mut ia = alu;
    while ia > 0 {
        let bmv = ia.v(jj);
        crate::cpu::tsc::asq(bmv);
        ia -= bmv;
        while let Some(jt) = crate::keyboard::xw() {
            if jt & 0x80 != 0 { continue; }
            if jt == 0x01 { return 1; } 
            if jt == 0x39 { return 2; } 
        }
    }
    0
}






fn rng(audio: &[i16], ay: usize, ci: usize) -> u32 {
    let e = ay.v(audio.len());
    let aa = ci.v(audio.len());
    if aa <= e { return 0; }
    let slice = &audio[e..aa];
    let wvx: u64 = slice.iter().map(|p| p.eki() as u64).sum();
    let abl = (wvx / slice.len().am(1) as u64) as u32;
    (abl * 100 / 8000).v(100)
}






fn sdh(
    gz: u32, kc: u32,
    wej: &[i16; 256],
    abo: u32, 
    frame: u32,
) {
    let gwn = gz * 72 / 100;
    let ddk = kc * 34 / 100;
    let gwo = (gz - gwn) / 2;
    let uq = kc / 2;

    
    let xg = 35 + abo * 65 / 100;

    
    let keh = (frame % 40) as u32;
    let qrv = if keh < 20 { keh } else { 40 - keh }; 
    let qrw = 92 + qrv * 16 / 20; 

    let qhs = xg * qrw / 100;

    let lpl: usize = 256;
    let jxp = (gwn / lpl as u32).am(1);

    
    let mut fzf = [0i32; 256];
    for a in 0..256 {
        let yr = wej[a] as i32;
        let mrv = yr * (ddk as i32 / 2) * qhs as i32 / (32768 * 100);
        fzf[a] = uq as i32 - mrv;
    }

    
    for a in 0..lpl {
        let b = gwo + a as u32 * jxp;
        let c = fzf[a];
        let ae = uq as i32;
        let (qc, i) = if c < ae {
            (c.am(0) as u32, (ae - c).am(1) as u32)
        } else {
            (ae.am(0) as u32, (c - ae).am(1) as u32)
        };
        crate::framebuffer::ih(b, qc, jxp, i, 0x00DDCC, 18);
    }

    
    let tq: [(i32, u32, u32); 5] = [
        (14, 10,  0x6622FF), 
        (8,  20,  0x4444FF), 
        (4,  45,  0x00AAEE), 
        (2,  100, 0x00DDCC), 
        (1,  220, 0x44FFDD), 
    ];

    for &(wp, dw, s) in &tq {
        for a in 0..lpl {
            let b = gwo + a as u32 * jxp;
            let c = fzf[a];
            let qc = (c - wp).am(0) as u32;
            let bjj = (c + wp).v(kc as i32) as u32;
            let i = bjj.ao(qc).am(1);
            crate::framebuffer::ih(b, qc, jxp, i, s, dw);
        }
    }

    
    crate::framebuffer::ih(gwo, uq.ao(1), gwn, 2, 0x00FFCC, 12);

    
    let mzr = uq.ao(ddk / 2);
    let qqp = uq + ddk / 2;
    crate::framebuffer::ih(gwo, mzr, gwn, 1, 0x00FFCC, 10);
    crate::framebuffer::ih(gwo, qqp, gwn, 1, 0x00FFCC, 10);

    
    let wdy = mzr + (frame % ddk.am(1));
    crate::framebuffer::ih(gwo, wdy, gwn, 2, 0x00FFCC, 18);
}


fn scm(
    gz: u32, kc: u32,
    phe: &str,
    hzg: usize,
    djd: u32, xki: u32,
    gu: usize, tk: usize,
    kz: u16,
) {
    let bv = 2u32;
    let nk = 8 * bv;

    
    let dq = "NEON PROTOCOL";
    let qd = dq.len() as u32 * nk;
    let gx = (gz.ao(qd)) / 2;
    crate::framebuffer::ih(gx.ao(12), 10, qd + 24, 36, 0x000000, 160);
    crate::graphics::scaling::azp(gx as i32, 14, dq, 0x00FFCC, bv);

    
    let pls = phe.len() as u32 * 8 + 16;
    let plt = (gz.ao(pls)) / 2;
    crate::framebuffer::ih(plt.ao(4), 50, pls + 8, 20, 0x000000, 140);
    crate::framebuffer::cb(phe, plt, 52, 0xBB44FF);

    
    let mzy = format!("{} BPM", kz);
    let nm = mzy.len() as u32 * 8 + 16;
    crate::framebuffer::ih(6, 8, nm, 20, 0x000000, 140);
    crate::framebuffer::cb(&mzy, 14, 12, 0x00AA88);

    
    let phb = format!("{}/8 L{}/{}", hzg + 1, djd + 1, xki);
    let kp = phb.len() as u32 * 8 + 16;
    let cr = gz.ao(kp + 8);
    crate::framebuffer::ih(cr, 8, kp, 20, 0x000000, 140);
    crate::framebuffer::cb(&phb, cr + 8, 12, 0x00AA88);

    
    let ewj = kc.ao(28);
    let ewh = 4u32;
    let fqm = gz.ao(60);
    let ewi = 30u32;
    
    crate::framebuffer::ih(ewi.ao(6), ewj.ao(6), fqm + 12, ewh + 12, 0x00FFCC, 6);
    
    crate::framebuffer::ah(ewi, ewj, fqm, ewh, 0x001111);
    if tk > 0 {
        let adu = fqm * gu as u32 / tk as u32;
        crate::framebuffer::ah(ewi, ewj, adu, ewh, 0x00FFCC);
        
        crate::framebuffer::ih(ewi, ewj.ao(2), adu, ewh + 4, 0x00FFCC, 30);
    }
    
    for a in 1..tk {
        if a % 8 == 0 {
            let hl = ewi + fqm * a as u32 / tk as u32;
            crate::framebuffer::axt(hl, ewj, ewh, 0x005555);
        }
    }

    
    let ohu = "Eb minor";
    let yo = ohu.len() as u32 * 8 + 8;
    crate::framebuffer::ih(gz.ao(yo + 8), kc.ao(48), yo, 16, 0x000000, 120);
    crate::framebuffer::cb(ohu, gz.ao(yo + 4), kc.ao(46), 0x665588);
}


pub fn uda() -> Result<(), &'static str> {
    crate::audio::init().bq();
    crate::serial_println!("[SHOWCASE] Starting narrated showcase...");

    let gz = crate::framebuffer::AB_.load(Ordering::Relaxed) as u32;
    let kc = crate::framebuffer::Z_.load(Ordering::Relaxed) as u32;

    
    crate::framebuffer::beo();
    crate::framebuffer::afi(true);

    
    
    

    dgs(gz, kc,
        "T R U S T D A W",
        "Building a Funky House Track from Scratch",
        "Bare-Metal  //  No OS  //  Pure Rust  //  Real-Time Audio",
        0x00CCFF,
    );
    crate::framebuffer::sv();
    if cnw(6500) { bcq(); return Ok(()); }

    dgs(gz, kc,
        "PHASE 1: BUILDING THE BEAT",
        "Watch each layer come to life, one track at a time",
        "100 BPM  //  C Minor  //  32 Steps (2 Bars)  //  Echo FX",
        0x44FF88,
    );
    crate::framebuffer::sv();
    if cnw(5500) { bcq(); return Ok(()); }

    
    
    
    crate::serial_println!("[SHOWCASE] Phase 1: Building the beat");

    
    let mut fsl = BeatStudio::new();
    fsl.ljh();

    
    let mut er = BeatStudio::new();
    er.kz = fsl.kz;
    er.ezi = fsl.ezi;
    
    for ab in er.af.el() {
        ab.aml = 32;
        for e in 0..FL_ {
            ab.au[e] = BeatStep::dz();
        }
    }
    er.af[0] = BeatTrack::new("Kick",     36, Waveform::Dg,     colors::S_[0], true);
    er.af[1] = BeatTrack::new("Clap",     39, Waveform::Cr,    colors::S_[1], true);
    er.af[2] = BeatTrack::new("HiHat",    42, Waveform::Cr,    colors::S_[2], true);
    er.af[3] = BeatTrack::new("Sub Bass", 24, Waveform::Dg,     colors::S_[3], false);
    er.af[4] = BeatTrack::new("Mid Bass", 36, Waveform::Gb,   colors::S_[4], false);
    er.af[5] = BeatTrack::new("Chords",   60, Waveform::Triangle, colors::S_[5], false);
    er.af[6] = BeatTrack::new("Lead",     72, Waveform::Ft, colors::S_[6], false);
    er.af[7] = BeatTrack::new("Perc",     56, Waveform::Cr,    colors::S_[7], true);
    for ab in er.af.el() {
        ab.aml = 32;
    }
    
    for a in 0..8 {
        er.af[a].qr = fsl.af[a].qr;
        er.af[a].hq = fsl.af[a].hq;
        er.af[a].so = false;
    }

    let bop = er.dwh();
    let tk = 32usize;
    let zbr = bop * tk as u32;

    
    let xlg: [Gz; 8] = [
        Gz {
            dq: "KICK -- The Foundation",
            atp: "Four-on-the-floor kicks + ghost notes",
            eu: "Sine wave @ C2  |  Deep 808 thump  |  150ms decay",
            vj: 0,
        },
        Gz {
            dq: "CLAP -- The Backbeat",
            atp: "Beats 2 & 4 with ghost flams",
            eu: "Noise burst  |  Tight snap  |  55ms decay",
            vj: 0,
        },
        Gz {
            dq: "HI-HAT -- The Groove Engine",
            atp: "16th note groove with velocity dynamics",
            eu: "Noise  |  Crispy short  |  Off-beat accents for the funk",
            vj: 0,
        },
        Gz {
            dq: "SUB BASS -- The Rumble",
            atp: "Deep sine sub following Cm -> Ab -> Bb",
            eu: "Sine wave @ C1 (33Hz!)  |  Long sustain  |  Feel it in your chest",
            vj: 0,
        },
        Gz {
            dq: "MID BASS -- The Funk",
            atp: "Syncopated pluck riding on top of the sub",
            eu: "Square wave @ C2  |  Punchy pluck  |  Funky syncopation",
            vj: 0,
        },
        Gz {
            dq: "CHORDS -- The Atmosphere",
            atp: "Lush pads: Cm -> Ab -> Bb progression",
            eu: "Triangle wave @ C4  |  Pad envelope  |  Harmonic movement",
            vj: 0,
        },
        Gz {
            dq: "LEAD -- The Hook",
            atp: "Catchy melody: G5-Bb5-C6 rising, Eb6 peak!",
            eu: "Sawtooth @ C5  |  Singing melody  |  Call & response over 2 bars",
            vj: 0,
        },
        Gz {
            dq: "PERCUSSION -- The Energy",
            atp: "Shakers + fill buildup into the drop",
            eu: "Noise burst  |  Snap envelope  |  Crescendo at bar 2 end",
            vj: 0,
        },
    ];

    
    for zx in 0..8 {
        er.bdw = zx;

        
        let btp = &xlg[zx];
        dgs(gz, kc,
            &format!("TRACK {}/8", zx + 1),
            btp.dq,
            btp.eu,
            colors::S_[zx],
        );
        crate::framebuffer::sv();
        if cnw(5000) { bcq(); return Ok(()); }

        
        let mut jrt: Vec<usize> = Vec::new();
        for e in 0..tk {
            if fsl.af[zx].au[e].gh {
                jrt.push(e);
            }
        }

        
        let hva = format!("PHASE 1  //  TRACK {}/8  //  PLACING PATTERN", zx + 1);

        
        er.po();
        epe(btp, gz, kc, &hva, 0, jrt.len() as u32);
        crate::framebuffer::sv();
        crate::cpu::tsc::asq(1200);

        
        for (vin, &mhq) in jrt.iter().cf() {
            
            er.bzc = mhq;

            
            er.po();
            let li = vin as u32;
            let es = jrt.len() as u32;
            epe(btp, gz, kc, &hva, li, es);
            crate::framebuffer::sv();

            
            crate::cpu::tsc::asq(200);

            
            er.af[zx].au[mhq] = fsl.af[zx].au[mhq];

            
            er.po();
            epe(btp, gz, kc, &hva, li + 1, es);
            crate::framebuffer::sv();

            
            crate::cpu::tsc::asq(280);

            
            while let Some(jt) = crate::keyboard::xw() {
                if jt & 0x80 != 0 { continue; }
                if jt == 0x01 { bcq(); return Ok(()); }
                if jt == 0x39 { break; } 
            }
        }

        
        let ugc = format!("PHASE 1  //  TRACK {}/8  //  LISTEN", zx + 1);

        let uga = Gz {
            dq: btp.dq,
            atp: if zx == 0 {
                "Listening to the kick pattern..."
            } else {
                "Hear how this layer adds to the mix..."
            },
            eu: btp.eu,
            vj: 0,
        };

        
        let audio = er.ehn();
        er.fat(&audio);

        
        let _ = crate::drivers::hda::dcg(&audio);
        er.uu = true;

        
        let mut ara = false;
        for yar in 0..3u32 {
            for e in 0..tk {
                er.aop = e;
                er.fxu();
                er.po();
                let li = (e as u32 * 100) / tk as u32;
                epe(&uga, gz, kc, &ugc, li, 100);
                crate::framebuffer::sv();

                match eks(bop as u64) {
                    1 => { ara = true; break; }
                    2 => { break; } 
                    _ => {}
                }
            }
            if ara { break; }
        }

        
        let _ = crate::drivers::hda::qg();
        er.uu = false;
        er.aop = 0;

        if ara { bcq(); return Ok(()); }

        
        crate::cpu::tsc::asq(1200);
    }

    
    
    
    crate::serial_println!("[SHOWCASE] Phase 2: Full mix playback");

    dgs(gz, kc,
        "PHASE 2: THE FULL MIX",
        "All 8 tracks together -- the complete Deep House groove",
        "Listen to how the layers combine with echo and sustain",
        0xFF6622,
    );
    crate::framebuffer::sv();
    if cnw(5500) { bcq(); return Ok(()); }

    
    let dqy = er.ehn();
    er.fat(&dqy);

    let uop = Gz {
        dq: "FULL MIX -- All 8 Tracks",
        atp: "Kick + Clap + HiHat + Sub + Bass + Chords + Lead + Perc",
        eu: "100 BPM  |  C Minor  |  Deep House  |  Echo FX  |  Bare-Metal Audio",
        vj: 0,
    };

    
    let _ = crate::drivers::hda::dcg(&dqy);
    er.uu = true;

    let mut ara = false;
    for djd in 0..3u32 {
        for e in 0..tk {
            er.aop = e;
            er.fxu();
            er.po();
            let uik = format!("PHASE 2  //  LOOP {}/3", djd + 1);
            let li = (e as u32 * 100) / tk as u32;
            epe(&uop, gz, kc, &uik, li, 100);
            crate::framebuffer::sv();

            match eks(bop as u64) {
                1 => { ara = true; break; }
                2 => { break; }
                _ => {}
            }
        }
        if ara { break; }
    }

    let _ = crate::drivers::hda::qg();
    er.uu = false;
    er.aop = 0;
    if ara { bcq(); return Ok(()); }

    
    
    
    crate::serial_println!("[SHOWCASE] Phase 3: Matrix visualizer");

    dgs(gz, kc,
        "PHASE 3: ENTER THE MATRIX",
        "The same beat, visualized as a living data stream",
        "Matrix rain  //  Beat-reactive  //  Pure framebuffer rendering",
        0x00FF44,
    );
    crate::framebuffer::sv();
    if cnw(5500) { bcq(); return Ok(()); }

    let mut matrix = MatrixState::new();

    
    for bb in 0..25 {
        matrix.or();
        let lfj = match bb {
            0..=6   => "> LOADING BEAT DATA...",
            7..=14  => "> DECODING FREQUENCY MATRIX...",
            15..=20 => "> SYNTH ENGINES ONLINE...",
            _       => "> ENTERING THE BEAT MATRIX...",
        };
        matrix.po(0, tk, lfj, er.kz, "---");
        crate::framebuffer::sv();
        if cnw(150) { bcq(); return Ok(()); }
    }

    
    let _ = crate::drivers::hda::dcg(&dqy);

    let olk = 3u32;
    ara = false;
    for djd in 0..olk {
        for e in 0..tk {
            er.aop = e;

            
            for ab in 0..8 {
                if er.af[ab].au[e].gh && !er.af[ab].so {
                    matrix.nuz(er.af[ab].au[e].qm);
                }
            }

            
            let mut coh = format!("LOOP {}/{}  > ", djd + 1, olk);
            for ab in 0..8 {
                if er.af[ab].au[e].gh && !er.af[ab].so {
                    coh.t(er.af[ab].amj());
                    coh.push(' ');
                }
            }
            if coh.pp("> ") { coh.t("..."); }

            let bar = e / 16 + 1;
            let rf = (e % 16) / 4 + 1;
            let sub = e % 4 + 1;
            let dar = format!("{}:{}.{}", bar, rf, sub);

            matrix.or();
            matrix.po(e, tk, &coh, er.kz, &dar);
            crate::framebuffer::sv();

            match eks(bop as u64) {
                1 => { ara = true; break; }
                2 => { break; }
                _ => {}
            }
        }
        if ara { break; }
    }

    let _ = crate::drivers::hda::qg();

    
    
    
    crate::serial_println!("[SHOWCASE] Outro");

    
    for bb in 0..35 {
        matrix.or();
        let lrd = match bb {
            0..=8   => "> SIGNAL FADING...",
            9..=20  => "> DISCONNECTING FROM THE MATRIX...",
            _       => "> TRUSTDAW // SYSTEM OFFLINE",
        };
        matrix.po(0, tk, lrd, er.kz, "---");
        crate::framebuffer::sv();

        
        let ptv = matrix.ajg / 30;
        for r in 0..ptv {
            let w = (bb as usize * ptv + r) % matrix.ajg;
            matrix.cpm[w].gh = false;
        }

        crate::cpu::tsc::asq(100); 
    }

    
    crate::framebuffer::ah(0, 0, gz, kc, 0x020208);

    let vs = kc / 2;

    
    crate::framebuffer::ah(gz / 4, vs - 80, gz / 2, 1, 0x00CCFF);
    crate::framebuffer::ah(gz / 4, vs + 80, gz / 2, 1, 0x00CCFF);

    let rqx: [(&str, u32); 8] = [
        ("T R U S T D A W",                          0x00FF66),
        ("",                                          0x000000),
        ("A bare-metal beat production studio",       0xCCCCDD),
        ("running on TrustOS -- written in Rust",     0xCCCCDD),
        ("",                                          0x000000),
        ("No operating system. No libraries.",         0x88AACC),
        ("Just raw hardware, a framebuffer, and HDA audio.", 0x88AACC),
        ("",                                          0x000000),
    ];

    let vc = vs - 60;
    for (a, (text, s)) in rqx.iter().cf() {
        if text.is_empty() { continue; }
        let qd = text.len() as u32 * 8;
        let gx = (gz - qd) / 2;
        crate::framebuffer::cb(text, gx, vc + a as u32 * 20, *s);
    }

    
    let ll = "Press any key to exit";
    let qd = ll.len() as u32 * 8;
    crate::framebuffer::cb(ll, (gz - qd) / 2, vs + 60, 0x556677);
    crate::framebuffer::sv();

    
    loop {
        if let Some(jt) = crate::keyboard::xw() {
            if jt & 0x80 == 0 { break; }
        }
        crate::cpu::tsc::asq(20);
    }

    bcq();
    crate::serial_println!("[SHOWCASE] Narrated showcase complete");
    Ok(())
}


fn bcq() {
    crate::framebuffer::afi(false);
}


fn lin(g: u32) -> u32 {
    let mut e = g;
    if e == 0 { e = 0xDEAD_BEEF; }
    e ^= e << 13;
    e ^= e >> 17;
    e ^= e << 5;
    e
}






pub fn ucv() -> Result<(), &'static str> {
    crate::audio::init().bq();
    crate::serial_println!("[ANTHEM] Starting TrustOS Anthem...");

    let gz = crate::framebuffer::AB_.load(Ordering::Relaxed) as u32;
    let kc = crate::framebuffer::Z_.load(Ordering::Relaxed) as u32;

    crate::framebuffer::beo();
    crate::framebuffer::afi(true);

    
    
    
    dgs(gz, kc,
        "T R U S T O S    A N T H E M",
        "Renaissance Numerique",
        "Cm -> C Major  //  106 BPM  //  Tension -> Revelation -> Maitrise",
        0x00CCFF,
    );
    crate::framebuffer::sv();
    if cnw(6000) { bcq(); return Ok(()); }

    
    
    
    struct Zu {
        dq: &'static str,
        atp: &'static str,
        eu: &'static str,
        s: u32,
        bkh: u32,
    }

    let aeo = [
        Zu {
            dq: "INTRO -- L'EVEIL",
            atp: "Quelque chose s'eveille...",
            eu: "Pad drone  |  Heartbeat sub  |  Texture digitale",
            s: 0x4466CC, bkh: 6,
        },
        Zu {
            dq: "BUILD -- L'ESPOIR",
            atp: "L'espoir nait, le rythme s'installe",
            eu: "Kick doux  |  Arpege montant  |  Basse chaude",
            s: 0x44AAFF, bkh: 8,
        },
        Zu {
            dq: "DROP -- LA REVELATION",
            atp: "Explosion controlee. Le controle est repris.",
            eu: "Full mix  |  Lead lumineux  |  Groove electro-funk",
            s: 0xFF6622, bkh: 10,
        },
        Zu {
            dq: "STABLE -- LA MAITRISE",
            atp: "Le theme TrustOS. Souverain. Reconnaissable.",
            eu: "Motif C-E-G-C  |  Cm -> C Major!  |  Identite sonore",
            s: 0x00FF66, bkh: 10,
        },
        Zu {
            dq: "OUTRO -- FUTUR SOUVERAIN",
            atp: "Le signal s'estompe... le motif reste.",
            eu: "Pad + motif  |  Serenite  |  Un futur souverain",
            s: 0x8844FF, bkh: 6,
        },
    ];

    
    
    
    for (hzg, zw) in aeo.iter().cf() {
        
        dgs(gz, kc,
            &format!("SECTION {}/5", hzg + 1),
            zw.dq,
            zw.eu,
            zw.s,
        );
        crate::framebuffer::sv();
        if cnw(4500) { bcq(); return Ok(()); }

        
        let mut er = BeatStudio::new();
        match hzg {
            0 => er.qiz(),
            1 => er.qix(),
            2 => er.qiy(),
            3 => er.qjb(),
            _ => er.qja(),
        }

        
        let audio = er.ehn();
        er.fat(&audio);
        let _ = crate::drivers::hda::dcg(&audio);
        er.uu = true;

        let bop = er.dwh();
        let tk = 32usize;

        let btp = Gz {
            dq: zw.dq,
            atp: zw.atp,
            eu: zw.eu,
            vj: 0,
        };

        
        let mut ara = false;
        for djd in 0..zw.bkh {
            for e in 0..tk {
                er.aop = e;
                er.fxu();
                er.po();
                let hva = format!(
                    "SECTION {}/5  //  LOOP {}/{}",
                    hzg + 1, djd + 1, zw.bkh
                );
                let li = (e as u32 * 100) / tk as u32;
                epe(&btp, gz, kc, &hva, li, 100);
                crate::framebuffer::sv();

                match eks(bop as u64) {
                    1 => { ara = true; break; } 
                    2 => { break; }                  
                    _ => {}
                }
            }
            if ara { break; }
        }

        let _ = crate::drivers::hda::qg();
        er.uu = false;
        if ara { bcq(); return Ok(()); }

        
        crate::cpu::tsc::asq(800);
    }

    
    
    
    crate::framebuffer::ah(0, 0, gz, kc, 0x020208);
    let vs = kc / 2;
    crate::framebuffer::ah(gz / 4, vs - 80, gz / 2, 2, 0x00CCFF);
    crate::framebuffer::ah(gz / 4, vs + 80, gz / 2, 2, 0x00CCFF);

    let bv = 2u32;
    let nk = 8 * bv;

    let dq = "T R U S T O S   A N T H E M";
    let blt = dq.len() as u32 * nk;
    crate::graphics::scaling::azp(
        ((gz.ao(blt)) / 2) as i32, (vs - 55) as i32,
        dq, 0x00FF66, bv);

    let sub = "Renaissance Numerique  --  Un futur souverain.";
    let bfs = sub.len() as u32 * nk;
    crate::graphics::scaling::azp(
        ((gz.ao(bfs)) / 2) as i32, (vs - 10) as i32,
        sub, 0xCCCCDD, bv);

    let co = "Composed on TrustOS  //  Bare-metal Rust  //  Native HDA Audio";
    let bxu = co.len() as u32 * nk;
    crate::graphics::scaling::azp(
        ((gz.ao(bxu)) / 2) as i32, (vs + 30) as i32,
        co, 0x88AACC, bv);

    let ll = "Press any key to exit";
    let qd = ll.len() as u32 * 8;
    crate::framebuffer::cb(ll, (gz - qd) / 2, vs + 65, 0x556677);
    crate::framebuffer::sv();

    loop {
        if let Some(jt) = crate::keyboard::xw() {
            if jt & 0x80 == 0 { break; }
        }
        crate::cpu::tsc::asq(20);
    }

    bcq();
    crate::serial_println!("[ANTHEM] TrustOS Anthem complete");
    Ok(())
}







pub fn udb() -> Result<(), &'static str> {
    crate::audio::init().bq();
    crate::serial_println!("[CYBER] 'Neon Protocol' — Creative Process + Full Song");

    let gz = crate::framebuffer::AB_.load(Ordering::Relaxed) as u32;
    let kc = crate::framebuffer::Z_.load(Ordering::Relaxed) as u32;

    crate::framebuffer::beo();
    crate::framebuffer::afi(true);

    let tk = 32usize;
    let mut er = BeatStudio::new();

    
    
    
    

    dgs(gz, kc,
        "T R U S T D A W",
        "\"NEON PROTOCOL\" — Creative Process",
        "Watch the beat come alive  |  100 BPM  |  Eb minor",
        0x00FFCC,
    );
    crate::framebuffer::sv();
    if cnw(5000) { bcq(); return Ok(()); }

    
    er.jdv();
    let bop = er.dwh();
    for ab in 0..8 { er.af[ab].so = true; }

    
    let my: [(usize, &str, &str, u32, u32); 8] = [
        (0, "+ SUB BASS",  "The foundation — 43 Hz Eb1 rumble",    3, 0xFF4444),
        (1, "+ SNARE",     "Hard mechanical crack — beats 3 & 7",  2, 0xFFAA22),
        (2, "+ HI-HAT",    "Aggressive 16th-note machine gun",     2, 0xFFFF44),
        (3, "+ OPEN HAT",  "Digital sizzle — off-beat accents",    1, 0x88FF44),
        (4, "+ SYNTH",     "Neon arpeggio: Eb > B > Ab > Gb",      2, 0x44DDFF),
        (5, "+ PAD",       "Cold digital atmosphere",               2, 0x8844FF),
        (6, "+ LEAD",      "The hook — cuts through the noise",     2, 0xFF44CC),
        (7, "+ PERC",      "Glitch percussion accents",             1, 0xCCCCCC),
    ];

    for &(zx, j, desc, bkh, mm) in &my {
        
        er.af[zx].so = false;

        
        dgs(gz, kc, j, desc, "", mm);
        crate::framebuffer::sv();
        if cnw(1800) { bcq(); return Ok(()); }

        
        let audio = er.ehn();
        er.fat(&audio);
        let _ = crate::drivers::hda::dcg(&audio);
        er.uu = true;

        let btp = Gz {
            dq: j,
            atp: desc,
            eu: "Building the beat...",
            vj: 0,
        };

        let mut ara = false;
        for djd in 0..bkh {
            for e in 0..tk {
                er.aop = e;
                er.fxu();

                for jp in 0..128 { er.dsl[jp] = false; }
                for fvz in 0..8 {
                    if er.af[fvz].so { continue; }
                    if er.af[fvz].au[e].gh {
                        let ayg = er.af[fvz].lov(e);
                        if ayg > 0 && ayg < 128 {
                            er.dsl[ayg as usize] = true;
                        }
                    }
                }

                er.po();
                let cu = format!("{} — Loop {}/{}", j, djd + 1, bkh);
                let li = (e as u32 * 100) / tk as u32;
                epe(&btp, gz, kc, &cu, li, 100);
                crate::framebuffer::sv();

                match eks(bop as u64) {
                    1 => { ara = true; break; }
                    2 => { break; }
                    _ => {}
                }
            }
            if ara { break; }
        }

        let _ = crate::drivers::hda::qg();
        er.uu = false;
        er.aop = 0;
        for jp in 0..128 { er.dsl[jp] = false; }
        if ara { bcq(); return Ok(()); }
    }

    
    dgs(gz, kc,
        "ALL LAYERS ACTIVE",
        "The complete beat — \"Neon Protocol\"",
        "8 tracks  |  100 BPM  |  Eb minor",
        0x00FFCC,
    );
    crate::framebuffer::sv();
    if cnw(2500) { bcq(); return Ok(()); }

    {
        let audio = er.ehn();
        er.fat(&audio);
        let _ = crate::drivers::hda::dcg(&audio);
        er.uu = true;

        let btp = Gz {
            dq: "FULL MIX",
            atp: "All 8 layers combined",
            eu: "Neon Protocol — complete beat",
            vj: 0,
        };

        let mut ara = false;
        for djd in 0..3u32 {
            for e in 0..tk {
                er.aop = e;
                er.fxu();
                for jp in 0..128 { er.dsl[jp] = false; }
                for fvz in 0..8 {
                    if !er.af[fvz].so && er.af[fvz].au[e].gh {
                        let ayg = er.af[fvz].lov(e);
                        if ayg > 0 && ayg < 128 { er.dsl[ayg as usize] = true; }
                    }
                }
                er.po();
                let cu = format!("FULL MIX — Loop {}/3", djd + 1);
                epe(&btp, gz, kc, &cu, (e as u32 * 100) / tk as u32, 100);
                crate::framebuffer::sv();
                match eks(bop as u64) {
                    1 => { ara = true; break; }
                    2 => { break; }
                    _ => {}
                }
            }
            if ara { break; }
        }

        let _ = crate::drivers::hda::qg();
        er.uu = false;
        er.aop = 0;
        if ara { bcq(); return Ok(()); }
    }

    
    
    
    
    
    

    dgs(gz, kc,
        "ENTERING THE MATRIX",
        "\"NEON PROTOCOL\" — Full Song",
        "8 sections  |  Pulsing waveform  |  [Esc] Exit",
        0x00FFCC,
    );
    crate::framebuffer::sv();
    if cnw(4000) { bcq(); return Ok(()); }

    let mut matrix = MatrixState::new();

    let wfs: [&str; 8] = [
        "INTRO — System Boot",
        "DROP — Neon Protocol",
        "BREAKDOWN — Signal Lost",
        "BUILD — Recompile",
        "BRIDGE — Blackout",
        "REBUILD — Reboot Sequence",
        "FINAL DROP — Full Override",
        "OUTRO — Shutdown",
    ];
    let pha: [u32; 8] = [3, 5, 1, 4, 1, 3, 3, 1];

    
    for bb in 0..30u32 {
        matrix.or();
        matrix.eba();
        let fr = match bb {
            0..=8   => "INITIALIZING NEON PROTOCOL...",
            9..=18  => "LOADING WAVEFORM ENGINE...",
            _       => "READY.",
        };
        let hsf = fr.len() as u32 * 16 + 32;
        let hl = (gz.ao(hsf)) / 2;
        let ir = kc / 2 - 16;
        crate::framebuffer::ih(hl.ao(8), ir.ao(8), hsf + 16, 48, 0x000000, 180);
        crate::graphics::scaling::azp(hl as i32, ir as i32, fr, 0x00FFCC, 2);
        crate::framebuffer::sv();
        if cnw(80) { bcq(); return Ok(()); }
    }

    
    
    let mut dqy: Vec<i16> = Vec::new();
    
    let mut mcx: Vec<(usize, usize)> = Vec::new(); 
    let mut kzf: usize = 0;

    for zw in 0..8usize {
        match zw {
            0 => er.uhn(),
            1 => er.jdv(),
            2 => er.uhq(),
            3 => er.uhl(),
            4 => er.uhk(),
            5 => er.uhp(),
            6 => er.uhm(),
            _ => er.uho(),
        }
        let wfx = er.ehn();
        for yas in 0..pha[zw] {
            mcx.push((zw, kzf));
            dqy.bk(&wfx);
            kzf += tk;
        }
    }

    let bop = er.dwh();
    let wuc = (60u32 * 48000) / (er.kz as u32 * 4);
    let pff = wuc as usize * 2; 
    let xkf = kzf;

    
    let _ = crate::drivers::hda::dcg(&dqy);

    
    let mut ara = false;
    let mut ipx = 0usize;
    let mut nib = 0u32;

    for at in 0..xkf {
        
        
        for (cvv, &(si, ckx)) in mcx.iter().cf() {
            if ckx <= at {
                ipx = si;
                
                nib = mcx[..=cvv].iter().hi(|&&(e, _)| e == si).az() as u32 - 1;
            }
        }
        let e = at % tk; 

        
        let bcm = at * pff;
        let is = ((at + 1) * pff).v(dqy.len());
        if bcm < dqy.len() {
            er.fat(&dqy[bcm..is]);
        }
        let abo = rng(&dqy, bcm, is);

        
        matrix.or();

        
        matrix.eba();
        sdh(gz, kc, &er.hyy, abo, matrix.frame);
        scm(gz, kc, wfs[ipx], ipx,
            nib, pha[ipx], e, tk, er.kz);

        crate::framebuffer::sv();

        match eks(bop as u64) {
            1 => { ara = true; break; }
            2 => {} 
            _ => {}
        }
    }

    let _ = crate::drivers::hda::qg();
    if ara { bcq(); return Ok(()); }

    
    for bb in 0..50u32 {
        matrix.or();
        let hfk = matrix.ajg / 50;
        for r in 0..hfk {
            let w = (bb as usize * hfk + r) % matrix.ajg;
            matrix.cpm[w].gh = false;
        }
        matrix.eba();

        let fr = match bb {
            0..=15  => "DISCONNECTING...",
            16..=30 => "SIGNAL LOST",
            _       => "NEON PROTOCOL // OFFLINE",
        };
        let hsf = fr.len() as u32 * 16 + 32;
        let hl = (gz.ao(hsf)) / 2;
        let ir = kc / 2 - 16;
        crate::framebuffer::ih(hl.ao(8), ir.ao(8), hsf + 16, 48, 0x000000, 180);
        crate::graphics::scaling::azp(hl as i32, ir as i32, fr, 0x00FFCC, 2);
        crate::framebuffer::sv();
        crate::cpu::tsc::asq(80);
    }

    
    crate::framebuffer::ah(0, 0, gz, kc, 0x050510);
    let vs = kc / 2;
    crate::framebuffer::ah(gz / 4, vs - 80, gz / 2, 2, 0x00FFCC);
    crate::framebuffer::ah(gz / 4, vs + 80, gz / 2, 2, 0x00FFCC);

    let bv = 2u32;
    let nk = 8 * bv;

    let aax = "\"NEON PROTOCOL\"";
    let blt = aax.len() as u32 * nk;
    crate::graphics::scaling::azp(
        ((gz.ao(blt)) / 2) as i32, (vs - 55) as i32,
        aax, 0x00FFCC, bv);

    let aco = "Cyberpunk Trap — 100 BPM — Eb minor";
    let bfs = aco.len() as u32 * nk;
    crate::graphics::scaling::azp(
        ((gz.ao(bfs)) / 2) as i32, (vs - 10) as i32,
        aco, 0xCCCCDD, bv);

    let bcx = "Creative Process + Full Song — Bare Metal Rust";
    let bxu = bcx.len() as u32 * nk;
    crate::graphics::scaling::azp(
        ((gz.ao(bxu)) / 2) as i32, (vs + 30) as i32,
        bcx, 0x8844CC, bv);

    let ll = "Press any key to exit";
    let qd = ll.len() as u32 * 8;
    crate::framebuffer::cb(ll, (gz - qd) / 2, vs + 65, 0x446688);
    crate::framebuffer::sv();

    loop {
        if let Some(jt) = crate::keyboard::xw() {
            if jt & 0x80 == 0 { break; }
        }
        crate::cpu::tsc::asq(20);
    }

    bcq();
    crate::serial_println!("[CYBER] 'Neon Protocol' complete");
    Ok(())
}
