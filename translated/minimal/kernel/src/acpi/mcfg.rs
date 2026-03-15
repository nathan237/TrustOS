



use alloc::vec::Vec;
use super::tables::Ei;


#[repr(C, packed)]
struct Dcj {
    dh: Ei,
    
    asi: u64,
    
}


#[repr(C, packed)]
#[derive(Clone, Copy)]
struct Bma {
    
    bps: u64,
    
    wgo: u16,
    
    cca: u8,
    
    cej: u8,
    
    asi: u32,
}


#[derive(Debug, Clone)]
pub struct Tl {
    
    pub bps: u64,
    
    pub ie: u16,
    
    pub cca: u8,
    
    pub cej: u8,
}

impl Tl {
    
    pub fn nfk(&self, aq: u8, de: u8, gw: u8) -> Option<u64> {
        if aq < self.cca || aq > self.cej {
            return None;
        }
        if de > 31 || gw > 7 {
            return None;
        }
        
        
        
        let l = ((aq.ao(self.cca)) as u64) << 20 
                   | (de as u64) << 15 
                   | (gw as u64) << 12;
        
        Some(self.bps + l)
    }
    
    
    pub fn aw(&self) -> u64 {
        let kfo = (self.cej.ao(self.cca) as u64).akq(1);
        kfo << 20  
    }
}


pub fn parse(omk: u64) -> Option<Vec<Tl>> {
    let dh = unsafe { &*(omk as *const Ei) };
    
    
    if &dh.signature != b"MCFG" {
        return None;
    }
    
    
    let drp = core::mem::size_of::<Ei>() + 8; 
    let acy = core::mem::size_of::<Bma>();
    let ebf = (dh.go as usize - drp) / acy;
    
    if ebf == 0 {
        return None;
    }
    
    let mut ch = Vec::fc(ebf);
    let fhu = omk + drp as u64;
    
    for a in 0..ebf {
        let ggi = fhu + (a * acy) as u64;
        let js = unsafe { &*(ggi as *const Bma) };
        
        let ar = unsafe { core::ptr::md(core::ptr::vf!(js.bps)) };
        let pk = unsafe { core::ptr::md(core::ptr::vf!(js.wgo)) };
        
        ch.push(Tl {
            bps: ar,
            ie: pk,
            cca: js.cca,
            cej: js.cej,
        });
    }
    
    Some(ch)
}


pub fn ytl(ie: u16, aq: u8, de: u8, gw: u8) -> Option<u64> {
    let co = super::ani()?;
    
    for bt in &co.eut {
        if bt.ie == ie && aq >= bt.cca && aq <= bt.cej {
            return bt.nfk(aq, de, gw);
        }
    }
    
    None
}


pub fn anl() -> bool {
    super::ani()
        .map(|a| !a.eut.is_empty())
        .unwrap_or(false)
}
