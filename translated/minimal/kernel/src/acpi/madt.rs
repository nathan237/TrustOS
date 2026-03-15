






use alloc::vec::Vec;
use super::tables::Ei;


#[repr(C, packed)]
struct Chf {
    
    cap: u32,
    
    flags: u32,
}


#[repr(C, packed)]
#[derive(Clone, Copy)]
struct Tj {
    avt: u8,
    go: u8,
}


const BTV_: u8 = 0;
const BTU_: u8 = 1;
const BTT_: u8 = 2;
const DLE_: u8 = 3;
const BTX_: u8 = 4;
const BTW_: u8 = 5;
const BTY_: u8 = 9;


#[repr(C, packed)]
struct Cgt {
    dh: Tj,
    
    qev: u8,
    
    aed: u8,
    
    flags: u32,
}


#[repr(C, packed)]
struct Cgd {
    dh: Tj,
    
    twe: u8,
    
    asi: u8,
    
    twd: u32,
    
    ech: u32,
}


#[repr(C, packed)]
struct Cfz {
    dh: Tj,
    
    aq: u8,
    
    iy: u8,
    
    bup: u32,
    
    flags: u16,
}


#[repr(C, packed)]
struct Cgu {
    dh: Tj,
    
    jza: u8,
    
    flags: u16,
    
    gln: u8,
}


#[repr(C, packed)]
struct Cgs {
    dh: Tj,
    
    asi: u16,
    
    cap: u64,
}


#[repr(C, packed)]
struct Cqq {
    dh: Tj,
    
    asi: u16,
    
    mrm: u32,
    
    flags: u32,
    
    jza: u32,
}


#[derive(Debug, Clone)]
pub struct Xl {
    
    pub aed: u32,
    
    pub bny: u32,
    
    pub iq: bool,
    
    pub htp: bool,
}


#[derive(Debug, Clone)]
pub struct Ach {
    
    pub ad: u8,
    
    pub re: u64,
    
    pub ech: u32,
}


#[derive(Debug, Clone)]
pub struct Xc {
    
    pub iy: u8,
    
    pub bup: u32,
    
    pub dkr: u8,
    
    pub dmt: u8,
}


#[derive(Debug, Clone)]
pub struct Acs {
    
    pub vmx: u8,
    
    pub gln: u8,
    
    pub dkr: u8,
    
    pub dmt: u8,
}


pub fn parse(jen: u64) -> Option<(u64, Vec<Xl>, Vec<Ach>, Vec<Xc>, Vec<Acs>)> {
    let dh = unsafe { &*(jen as *const Ei) };
    
    
    if &dh.signature != b"APIC" {
        return None;
    }
    
    let okr = core::mem::size_of::<Ei>();
    let ujg = unsafe { 
        &*((jen + okr as u64) as *const Chf) 
    };
    
    let mut cap = unsafe { 
        core::ptr::md(core::ptr::vf!(ujg.cap)) 
    } as u64;
    
    let mut dja = Vec::new();
    let mut cyx = Vec::new();
    let mut jif = Vec::new();
    let mut oqq = Vec::new();
    
    
    let fhu = jen + okr as u64 + 8;
    let xaj = jen + dh.go as u64;
    let mut l = fhu;
    
    while l + 2 <= xaj {
        let bzn = unsafe { &*(l as *const Tj) };
        
        if bzn.go < 2 {
            break;
        }
        
        match bzn.avt {
            BTV_ => {
                if bzn.go >= 8 {
                    let bt = unsafe { &*(l as *const Cgt) };
                    let flags = unsafe { core::ptr::md(core::ptr::vf!(bt.flags)) };
                    
                    dja.push(Xl {
                        aed: bt.aed as u32,
                        bny: bt.qev as u32,
                        iq: (flags & 1) != 0,
                        htp: (flags & 2) != 0,
                    });
                }
            }
            BTU_ => {
                if bzn.go >= 12 {
                    let bt = unsafe { &*(l as *const Cgd) };
                    let ag = unsafe { core::ptr::md(core::ptr::vf!(bt.twd)) };
                    let ech = unsafe { core::ptr::md(core::ptr::vf!(bt.ech)) };
                    
                    cyx.push(Ach {
                        ad: bt.twe,
                        re: ag as u64,
                        ech,
                    });
                }
            }
            BTT_ => {
                if bzn.go >= 10 {
                    let bt = unsafe { &*(l as *const Cfz) };
                    let bup = unsafe { core::ptr::md(core::ptr::vf!(bt.bup)) };
                    let flags = unsafe { core::ptr::md(core::ptr::vf!(bt.flags)) };
                    
                    jif.push(Xc {
                        iy: bt.iy,
                        bup,
                        dkr: (flags & 0x03) as u8,
                        dmt: ((flags >> 2) & 0x03) as u8,
                    });
                }
            }
            BTW_ => {
                if bzn.go >= 12 {
                    let bt = unsafe { &*(l as *const Cgs) };
                    cap = unsafe { 
                        core::ptr::md(core::ptr::vf!(bt.cap)) 
                    };
                }
            }
            BTX_ => {
                if bzn.go >= 6 {
                    let bt = unsafe { &*(l as *const Cgu) };
                    let flags = unsafe { core::ptr::md(core::ptr::vf!(bt.flags)) };
                    oqq.push(Acs {
                        vmx: bt.jza,
                        gln: bt.gln,
                        dkr: (flags & 0x03) as u8,
                        dmt: ((flags >> 2) & 0x03) as u8,
                    });
                }
            }
            BTY_ => {
                if bzn.go >= 16 {
                    let bt = unsafe { &*(l as *const Cqq) };
                    let mrm = unsafe { core::ptr::md(core::ptr::vf!(bt.mrm)) };
                    let flags = unsafe { core::ptr::md(core::ptr::vf!(bt.flags)) };
                    let pi = unsafe { core::ptr::md(core::ptr::vf!(bt.jza)) };
                    
                    dja.push(Xl {
                        aed: mrm,
                        bny: pi,
                        iq: (flags & 1) != 0,
                        htp: (flags & 2) != 0,
                    });
                }
            }
            _ => {
                
            }
        }
        
        l += bzn.go as u64;
    }
    
    Some((cap, dja, cyx, jif, oqq))
}
