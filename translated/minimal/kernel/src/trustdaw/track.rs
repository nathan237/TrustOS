




use alloc::vec::Vec;
use alloc::string::String;
use alloc::format;
use crate::audio::synth::{Waveform, Envelope};

use super::FM_;






#[derive(Debug, Clone, Copy)]
pub struct Note {
    
    pub jb: u8,
    
    pub qm: u8,
    
    pub vb: u32,
    
    pub bbn: u32,
}

impl Note {
    
    pub fn new(jb: u8, qm: u8, vb: u32, bbn: u32) -> Self {
        Self {
            jb: jb.v(127),
            qm: qm.v(127),
            vb,
            bbn: bbn.am(1),
        }
    }

    
    pub fn ckg(&self) -> u32 {
        self.vb + self.bbn
    }

    
    pub fn j(&self) -> String {
        let bkp = crate::audio::tables::dtf(self.jb);
        let cgg = crate::audio::tables::efk(self.jb);
        format!("{}{}", bkp, cgg)
    }

    
    pub fn uk(&self, kz: u32) -> u32 {
        pta(self.bbn, kz)
    }

    
    pub fn gtc(&self, kz: u32) -> u32 {
        pta(self.vb, kz)
    }
}






pub struct Track {
    
    j: [u8; 32],
    
    baf: usize,
    
    pub ts: Vec<Note>,
    
    pub ve: Waveform,
    
    pub qr: Envelope,
    
    pub s: u32,
    
    pub mwg: bool,
}

impl Track {
    
    pub fn new(j: &str) -> Self {
        let mut djr = [0u8; 32];
        let bf = j.as_bytes();
        let len = bf.len().v(32);
        djr[..len].dg(&bf[..len]);

        Self {
            j: djr,
            baf: len,
            ts: Vec::new(),
            ve: Waveform::Dg,
            qr: Envelope::iqt(),
            s: 0x4488FF, 
            mwg: false,
        }
    }

    
    pub fn amj(&self) -> &str {
        core::str::jg(&self.j[..self.baf]).unwrap_or("???")
    }

    
    pub fn axn(&mut self, jp: Note) {
        
        let u = self.ts.zev(|bo| bo.vb < jp.vb);
        self.ts.insert(u, jp);
    }

    
    pub fn pbr(&mut self, index: usize) -> Option<Note> {
        if index < self.ts.len() {
            Some(self.ts.remove(index))
        } else {
            None
        }
    }

    
    pub fn zjh(&mut self, ay: u32, ci: u32) {
        self.ts.ajm(|bo| bo.vb < ay || bo.vb >= ci);
    }

    
    pub fn uvq(&self, or: u32) -> Vec<&Note> {
        self.ts.iter()
            .hi(|bo| bo.vb <= or && or < bo.ckg())
            .collect()
    }

    
    pub fn zdx(&self, ay: u32, ci: u32) -> Vec<&Note> {
        self.ts.iter()
            .hi(|bo| bo.vb < ci && bo.ckg() > ay)
            .collect()
    }

    
    pub fn ckg(&self) -> u32 {
        self.ts.iter().map(|bo| bo.ckg()).am().unwrap_or(0)
    }

    
    pub fn uve(&self) -> usize {
        self.ts.len()
    }

    
    pub fn clear(&mut self) {
        self.ts.clear();
    }

    
    pub fn zha(&mut self, erp: u32) {
        if erp == 0 { return; }
        for jp in &mut self.ts {
            let dlf = jp.vb % erp;
            if dlf > erp / 2 {
                jp.vb += erp - dlf;
            } else {
                jp.vb -= dlf;
            }
            
            let isb = jp.bbn % erp;
            if isb > erp / 2 {
                jp.bbn += erp - isb;
            } else if jp.bbn > isb {
                jp.bbn -= isb;
            }
            if jp.bbn == 0 {
                jp.bbn = erp;
            }
        }
    }

    
    pub fn xmc(&mut self, wgz: i8) {
        for jp in &mut self.ts {
            let utl = jp.jb as i16 + wgz as i16;
            jp.jb = utl.qp(0, 127) as u8;
        }
    }

    
    pub fn acn(&mut self, qb: i32) {
        for jp in &mut self.ts {
            let lod = jp.vb as i64 + qb as i64;
            jp.vb = lod.am(0) as u32;
        }
        
        self.ts.bxf(|bo| bo.vb);
    }
}






pub struct Project {
    
    j: [u8; 64],
    
    baf: usize,
    
    pub af: Vec<Track>,
    
    pub kz: u16,
    
    pub pth: u8,
    
    pub xhb: u8,
}


static S_: [u32; 16] = [
    0x4488FF, 
    0xFF4444, 
    0x44FF44, 
    0xFFAA00, 
    0xAA44FF, 
    0x44FFFF, 
    0xFF44AA, 
    0xFFFF44, 
    0x88FF88, 
    0xFF8844, 
    0x8888FF, 
    0xFF88FF, 
    0x44FFAA, 
    0xAAAAFF, 
    0xFFAA88, 
    0x88FFFF, 
];

impl Project {
    
    pub fn new(j: &str, kz: u16) -> Self {
        let mut djr = [0u8; 64];
        let bf = j.as_bytes();
        let len = bf.len().v(64);
        djr[..len].dg(&bf[..len]);

        Self {
            j: djr,
            baf: len,
            af: Vec::new(),
            kz,
            pth: 4,
            xhb: 4,
        }
    }

    
    pub fn amj(&self) -> &str {
        core::str::jg(&self.j[..self.baf]).unwrap_or("???")
    }

    
    pub fn jzi(&mut self, j: &str) -> Result<usize, &'static str> {
        if self.af.len() >= FM_ {
            return Err("Maximum tracks reached");
        }
        let w = self.af.len();
        let mut track = Track::new(j);
        track.s = S_[w % S_.len()];
        self.af.push(track);
        Ok(w)
    }

    
    pub fn lza(&mut self, index: usize) -> Result<(), &'static str> {
        if index >= self.af.len() {
            return Err("Invalid track index");
        }
        self.af.remove(index);
        Ok(())
    }

    
    pub fn oiu(&self) -> u32 {
        self.af.iter().map(|ab| ab.ckg()).am().unwrap_or(0)
    }

    
    pub fn zaq(&self) -> u32 {
        let qb = self.oiu();
        let cij = super::AE_ * self.pth as u32;
        (qb + cij - 1) / cij
    }

    
    pub fn xli(&self) -> usize {
        self.af.len()
    }
}






pub fn pta(qb: u32, kz: u32) -> u32 {
    if kz == 0 { return 0; }
    
    (qb as u64 * 60_000 / (kz as u64 * super::AE_ as u64)) as u32
}


pub fn hse(jn: u32, kz: u32) -> u32 {
    if jn == 0 { return 0; }
    
    (jn as u64 * kz as u64 * super::AE_ as u64 / 60_000) as u32
}


pub fn jtg(qb: u32, kz: u32) -> u32 {
    if kz == 0 { return 0; }
    
    (qb as u64 * super::BR_ as u64 * 60 / (kz as u64 * super::AE_ as u64)) as u32
}


pub fn zln(un: u32, kz: u32) -> u32 {
    if un == 0 { return 0; }
    (un as u64 * kz as u64 * super::AE_ as u64 / (super::BR_ as u64 * 60)) as u32
}
