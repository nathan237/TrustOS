







use alloc::vec::Vec;
use alloc::collections::BTreeMap;
use spin::Mutex;
use core::sync::atomic::{AtomicU64, Ordering};






#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(u64)]
pub enum MemoryProtection {
    
    None = 0,
    
    Bz = 0b001,
    
    Jx = 0b011,
    
    Cwg = 0b100,
    
    Ckg = 0b101,
    
    Bqt = 0b111,
}

impl MemoryProtection {
    
    pub fn ogr(&self) -> bool {
        (*self as u64) & 0b001 != 0
    }
    
    
    pub fn edz(&self) -> bool {
        (*self as u64) & 0b010 != 0
    }
    
    
    pub fn clc(&self) -> bool {
        (*self as u64) & 0b100 != 0
    }
}






#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RegionType {
    
    Jw,
    
    Aqc,
    
    Ckt,
    
    Ckz,
    
    Azf,
    
    Nn,
    
    Nw,
    
    Cmv,
}


#[derive(Debug, Clone)]
pub struct MemoryRegion {
    pub ay: u64,
    pub aw: u64,
    pub bwo: RegionType,
    pub ewx: MemoryProtection,
    pub j: &'static str,
}

impl MemoryRegion {
    pub fn new(ay: u64, aw: u64, bwo: RegionType, j: &'static str) -> Self {
        let ewx = match bwo {
            RegionType::Jw => MemoryProtection::Bqt,
            RegionType::Aqc => MemoryProtection::Ckg,
            RegionType::Ckt => MemoryProtection::Bz,
            RegionType::Ckz => MemoryProtection::Jx,
            RegionType::Azf => MemoryProtection::Jx,
            RegionType::Nn => MemoryProtection::Jx,
            RegionType::Nw => MemoryProtection::None,
            RegionType::Cmv => MemoryProtection::Jx,
        };
        
        MemoryRegion {
            ay,
            aw,
            bwo,
            ewx,
            j,
        }
    }
    
    pub fn ci(&self) -> u64 {
        self.ay + self.aw
    }
    
    pub fn contains(&self, ag: u64) -> bool {
        ag >= self.ay && ag < self.ci()
    }
}






pub struct Bam {
    pub fk: u64,
    pub afx: Vec<MemoryRegion>,
    pub mmi: u64,
}

impl Bam {
    
    pub fn new(fk: u64, afc: usize) -> Self {
        let mmi = (afc * 1024 * 1024) as u64;
        let mut afx = Vec::new();
        
        
        
        
        
        
        
        afx.push(MemoryRegion::new(0x0000, 0x1000, RegionType::Nw, "null_guard"));
        afx.push(MemoryRegion::new(0x1000, 0x7000, RegionType::Aqc, "code"));
        afx.push(MemoryRegion::new(0x8000, 0x8000, RegionType::Azf, "stack"));
        
        let bjt = 0x10000u64;
        let cpv = mmi.ao(bjt);
        if cpv > 0 {
            afx.push(MemoryRegion::new(bjt, cpv, RegionType::Jw, "data"));
        }
        
        Bam {
            fk,
            afx,
            mmi,
        }
    }
    
    
    pub fn sto(&self, ag: u64) -> Option<&MemoryRegion> {
        self.afx.iter().du(|m| m.contains(ag))
    }
    
    
    pub fn yen(&mut self, aoz: MemoryRegion) {
        
        self.afx.push(aoz);
    }
    
    
    pub fn yhv(&self, ag: u64, rm: bool, jbh: bool) -> bool {
        if let Some(aoz) = self.sto(ag) {
            if rm && !aoz.ewx.edz() {
                return false;
            }
            if jbh && !aoz.ewx.clc() {
                return false;
            }
            if !rm && !jbh && !aoz.ewx.ogr() {
                return false;
            }
            true
        } else {
            false 
        }
    }
}






#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ViolationType {
    Read,
    Write,
    Ahw,
    Jx,
    Cqp,
}


#[derive(Debug, Clone)]
pub struct Lj {
    pub fk: u64,
    pub hmc: u64,
    pub hmb: Option<u64>,
    pub igm: ViolationType,
    pub aet: u64,
    pub wb: u64,
}

static BIJ_: Mutex<Vec<Lj>> = Mutex::new(Vec::new());
static BII_: AtomicU64 = AtomicU64::new(0);


pub fn pau(
    fk: u64,
    hmc: u64,
    hmb: Option<u64>,
    spa: u64,
    wb: u64,
) {
    let igm = vel(spa);
    
    let xrr = Lj {
        fk,
        hmc,
        hmb,
        igm,
        aet: crate::time::lc(),
        wb,
    };
    
    BII_.fetch_add(1, Ordering::SeqCst);
    
    let mut log = BIJ_.lock();
    if log.len() >= 100 {
        log.remove(0); 
    }
    log.push(xrr);
    
    crate::serial_println!(
        "[EPT] Violation: VM {} GPA=0x{:X} type={:?} at RIP=0x{:X}",
        fk, hmc, igm, wb
    );
}


fn vel(lwp: u64) -> ViolationType {
    let read = (lwp & 1) != 0;
    let write = (lwp & 2) != 0;
    let bna = (lwp & 4) != 0;
    
    match (read, write, bna) {
        (true, true, _) => ViolationType::Jx,
        (_, true, true) => ViolationType::Cqp,
        (_, true, _) => ViolationType::Write,
        (_, _, true) => ViolationType::Ahw,
        _ => ViolationType::Read,
    }
}


pub fn pyh() -> u64 {
    BII_.load(Ordering::SeqCst)
}


pub fn vte(az: usize) -> Vec<Lj> {
    let log = BIJ_.lock();
    let ay = if log.len() > az { log.len() - az } else { 0 };
    log[ay..].ip()
}






#[derive(Debug, Clone)]
pub struct Aer {
    pub cg: bool,
    pub message: &'static str,
    pub qj: SecuritySeverity,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SecuritySeverity {
    V,
    Oo,
    Aj,
}


pub fn yic(layout: &Bam) -> Vec<Aer> {
    let mut fer = Vec::new();
    
    
    let tmv = layout.afx.iter()
        .any(|m| m.ay == 0 && m.bwo == RegionType::Nw);
    
    fer.push(Aer {
        cg: tmv,
        message: "Null pointer guard page",
        qj: SecuritySeverity::Aj,
    });
    
    
    let tnd = layout.afx.iter()
        .any(|m| m.ewx == MemoryProtection::Bqt && 
             m.bwo != RegionType::Jw);
    
    fer.push(Aer {
        cg: !tnd,
        message: "W^X (no writable+executable regions)",
        qj: SecuritySeverity::Oo,
    });
    
    
    let wsc = layout.afx.iter()
        .hi(|m| m.bwo == RegionType::Azf)
        .xx(|m| !m.ewx.clc());
    
    fer.push(Aer {
        cg: wsc,
        message: "Stack is non-executable",
        qj: SecuritySeverity::Aj,
    });
    
    
    let rlh = layout.afx.iter()
        .hi(|m| m.bwo == RegionType::Aqc)
        .xx(|m| !m.ewx.edz());
    
    fer.push(Aer {
        cg: rlh,
        message: "Code sections are read-only",
        qj: SecuritySeverity::Oo,
    });
    
    fer
}






#[derive(Debug, Clone, Default)]
pub struct Aup {
    pub jtu: u64,
    pub zby: u64,
    pub zkn: u64,
    pub zoe: u64,
    pub cnt: u64,
}

static BIP_: Mutex<BTreeMap<u64, Aup>> = Mutex::new(BTreeMap::new());


pub fn yti(fk: u64) -> Aup {
    BIP_.lock().get(&fk).abn().age()
}


pub fn zuu(fk: u64, unx: Aup) {
    BIP_.lock().insert(fk, unx);
}






pub fn ppw() -> bool {
    
    let mh = super::vmx::bcg(0x48C);
    
    (mh & 1) != 0
}


pub fn zqa() -> bool {
    let mh = super::vmx::bcg(0x48C);
    
    (mh & (1 << 21)) != 0
}


pub fn zpz() -> bool {
    let mh = super::vmx::bcg(0x48C);
    
    (mh & (1 << 17)) != 0
}


pub fn ytd() -> u8 {
    let mh = super::vmx::bcg(0x48C);
    ((mh >> 8) & 0xFF) as u8
}
