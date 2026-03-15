



use super::tables::Ei;


#[repr(C, packed)]
struct Cfc {
    dh: Ei,
    
    
    sns: u32,
    
    
    yfn: u8,
    yfq: u8,
    yfp: u8,
    yfr: u8,
    bps: u64,
    
    
    lct: u8,
    
    onn: u16,
    
    zeq: u8,
}


#[derive(Debug, Clone)]
pub struct Wy {
    
    pub bps: u64,
    
    pub lct: u8,
    
    pub llx: u16,
    
    pub lph: u8,
    
    pub eoc: bool,
    
    pub lij: bool,
    
    pub ml: u16,
    
    pub ewo: u32,
}


pub mod regs {
    
    pub const BLU_: u64 = 0x000;
    
    pub const Pa: u64 = 0x010;
    
    pub const DRI_: u64 = 0x020;
    
    pub const Bze: u64 = 0x0F0;
    
    pub const EHQ_: u64 = 0x100;
    
    pub const EHP_: u64 = 0x108;
    
    pub const EHR_: u64 = 0x110;
}


pub fn parse(oci: u64) -> Option<Wy> {
    let dh = unsafe { &*(oci as *const Ei) };
    
    
    if &dh.signature != b"HPET" {
        return None;
    }
    
    let hpet = unsafe { &*(oci as *const Cfc) };
    
    let itd = unsafe { core::ptr::md(core::ptr::vf!(hpet.sns)) };
    let sm = unsafe { core::ptr::md(core::ptr::vf!(hpet.bps)) };
    let llx = unsafe { core::ptr::md(core::ptr::vf!(hpet.onn)) };
    
    
    let lph = ((itd >> 8) & 0x1F) as u8 + 1;
    let eoc = (itd & (1 << 13)) != 0;
    let lij = (itd & (1 << 15)) != 0;
    let ml = (itd >> 16) as u16;
    
    
    let ewo = if sm != 0 {
        
        match crate::memory::bki(sm, 4096) {
            Ok(vd) => {
                let mh = unsafe { core::ptr::read_volatile((vd + regs::BLU_) as *const u64) };
                (mh >> 32) as u32
            }
            Err(aa) => {
                crate::serial_println!("[HPET] Failed to map HPET MMIO at {:#x}: {}", sm, aa);
                0
            }
        }
    } else {
        0
    };
    
    Some(Wy {
        bps: sm,
        lct: hpet.lct,
        llx,
        lph,
        eoc,
        lij,
        ml,
        ewo,
    })
}

impl Wy {
    
    pub fn fjc(&self) -> u64 {
        if self.ewo == 0 {
            return 0;
        }
        
        1_000_000_000_000_000u64 / self.ewo as u64
    }
    
    
    pub fn vrl(&self) -> u64 {
        let hp = crate::memory::lr();
        let ag = self.bps + hp + regs::Bze;
        unsafe { core::ptr::read_volatile(ag as *const u64) }
    }
    
    
    pub fn cuf(&self, iq: bool) {
        let hp = crate::memory::lr();
        let dfe = self.bps + hp + regs::Pa;
        
        unsafe {
            let mut config = core::ptr::read_volatile(dfe as *const u64);
            if iq {
                config |= 1; 
            } else {
                config &= !1;
            }
            core::ptr::write_volatile(dfe as *mut u64, config);
        }
    }
    
    
    pub fn zni(&self, iq: bool) {
        if !self.lij {
            return;
        }
        
        let hp = crate::memory::lr();
        let dfe = self.bps + hp + regs::Pa;
        
        unsafe {
            let mut config = core::ptr::read_volatile(dfe as *const u64);
            if iq {
                config |= 2; 
            } else {
                config &= !2;
            }
            core::ptr::write_volatile(dfe as *mut u64, config);
        }
    }
    
    
    pub fn zsn(&self, qb: u64) -> u64 {
        
        if self.ewo == 0 {
            return 0;
        }
        (qb as u128 * self.ewo as u128 / 1_000_000) as u64
    }
    
    
    pub fn zdh(&self, efq: u64) -> u64 {
        if self.ewo == 0 {
            return 0;
        }
        (efq as u128 * 1_000_000 / self.ewo as u128) as u64
    }
}


pub fn init() -> bool {
    let co = match super::ani() {
        Some(a) => a,
        None => return false,
    };
    
    let hpet = match &co.hpet {
        Some(i) => i,
        None => {
            crate::serial_println!("[HPET] No HPET table found");
            return false;
        }
    };
    
    crate::serial_println!("[HPET] Initializing: base={:#x}, freq={} Hz", 
        hpet.bps, hpet.fjc());
    
    
    hpet.cuf(true);
    
    true
}
