

































use super::{CapabilityId, CapabilityType, CapabilityRights};
use spin::Mutex;
use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;
use core::sync::atomic::{AtomicU64, Ordering};


#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Subsystem {
    
    Cy,
    
    Og,
    
    As,
    
    Jm,
    
    Tz,
    
    Ee,
    
    Df,
    
    Jh,
    
    Nj,
    
    Yu,
    
    Hb,
    
    Lq,
    
    PciBus,
    
    Ol,
}

impl Subsystem {
    
    pub fn pcl(&self) -> CapabilityType {
        match self {
            Self::Cy => CapabilityType::Cy,
            Self::Og => CapabilityType::Agr,
            Self::As => CapabilityType::As,
            Self::Jm => CapabilityType::Jm,
            Self::Tz => CapabilityType::Process,
            Self::Ee => CapabilityType::Ee,
            Self::Df => CapabilityType::Ayr,
            Self::Jh => CapabilityType::Jh,
            Self::Nj => CapabilityType::Nj,
            Self::Yu => CapabilityType::Amm,
            Self::Hb => CapabilityType::Hb,
            Self::Lq => CapabilityType::Fv,
            Self::PciBus => CapabilityType::PciBus,
            Self::Ol => CapabilityType::Ol,
        }
    }

    
    pub fn j(&self) -> &'static str {
        match self {
            Self::Cy => "Memory",
            Self::Og => "Storage",
            Self::As => "Network",
            Self::Jm => "Graphics",
            Self::Tz => "Process",
            Self::Ee => "Hypervisor",
            Self::Df => "Shell",
            Self::Jh => "Crypto",
            Self::Nj => "LinuxCompat",
            Self::Yu => "Serial/Debug",
            Self::Hb => "Power",
            Self::Lq => "Interrupts",
            Self::PciBus => "PCI Bus",
            Self::Ol => "USB",
        }
    }

    
    fn nkf(&self) -> CapabilityRights {
        match self {
            
            Self::Cy => CapabilityRights::Mr,
            Self::Lq => CapabilityRights::Mr,
            
            Self::Og => CapabilityRights(
                CapabilityRights::Cm.0 | CapabilityRights::Db.0 | CapabilityRights::Mt.0
            ),
            
            Self::As => CapabilityRights(
                CapabilityRights::Cm.0 | CapabilityRights::Db.0 | CapabilityRights::Vx.0
            ),
            
            Self::Jm => CapabilityRights(
                CapabilityRights::Cm.0 | CapabilityRights::Db.0 | CapabilityRights::Blt.0
            ),
            
            Self::Tz => CapabilityRights(
                CapabilityRights::Cm.0 | CapabilityRights::Db.0 |
                CapabilityRights::Vx.0 | CapabilityRights::Mt.0 |
                CapabilityRights::Ayg.0
            ),
            
            Self::Ee => CapabilityRights::Mr,
            
            Self::Df => CapabilityRights(
                CapabilityRights::Cm.0 | CapabilityRights::Mz.0
            ),
            
            Self::Jh => CapabilityRights::AHA_,
            
            Self::Nj => CapabilityRights(
                CapabilityRights::Cm.0 | CapabilityRights::Db.0 | CapabilityRights::Mz.0
            ),
            
            Self::Yu => CapabilityRights::KZ_,
            
            Self::Hb => CapabilityRights(
                CapabilityRights::Mt.0 | CapabilityRights::Xy.0
            ),
            
            Self::PciBus => CapabilityRights(
                CapabilityRights::Cm.0 | CapabilityRights::Mt.0
            ),
            
            Self::Ol => CapabilityRights(
                CapabilityRights::Cm.0 | CapabilityRights::Db.0 | CapabilityRights::Mt.0
            ),
        }
    }

    
    pub fn tzp(&self) -> &'static str {
        match self {
            
            Self::Cy | Self::Lq => "ring0-tcb",
            
            Self::As | Self::Jm | Self::Df |
            Self::Nj | Self::Jh | Self::Ol |
            Self::Yu => "ring3-candidate",
            
            Self::Og | Self::Ee | Self::Tz |
            Self::Hb | Self::PciBus => "ring0-isolated",
        }
    }

    
    pub fn xx() -> &'static [Subsystem] {
        &[
            Self::Cy, Self::Og, Self::As, Self::Jm,
            Self::Tz, Self::Ee, Self::Df, Self::Jh,
            Self::Nj, Self::Yu, Self::Hb, Self::Lq,
            Self::PciBus, Self::Ol,
        ]
    }
}


struct Djb {
    
    capability: CapabilityId,
    
    yed: AtomicU64,
    
    cnt: AtomicU64,
}


static PT_: Mutex<BTreeMap<Subsystem, CapabilityId>> = Mutex::new(BTreeMap::new());


static EGY_: Mutex<BTreeMap<u8, (u64, u64)>> = Mutex::new(BTreeMap::new());


static ACR_: AtomicU64 = AtomicU64::new(0);

static ASQ_: AtomicU64 = AtomicU64::new(0);



pub fn tty() {
    for bcu in Subsystem::xx() {
        let cap_type = bcu.pcl();
        let bap = bcu.nkf();
        
        let awj = 0x5500 + (*bcu as u64);
        let cap_id = super::klu(cap_type, bap, awj);
        PT_.lock().insert(*bcu, cap_id);
    }
    
    crate::serial_println!("[ISOLATION] {} subsystem capability tokens created", 
        Subsystem::xx().len());
}


pub fn yty(bcu: Subsystem) -> Option<CapabilityId> {
    PT_.lock().get(&bcu).hu()
}




pub fn drb(
    bcu: Subsystem,
    bao: CapabilityRights,
) -> Result<(), super::SecurityError> {
    ASQ_.fetch_add(1, Ordering::Relaxed);
    
    let cap_id = match PT_.lock().get(&bcu).hu() {
        Some(ad) => ad,
        None => {
            ACR_.fetch_add(1, Ordering::Relaxed);
            crate::log_warn!("[ISOLATION] Gate denied: subsystem {:?} has no capability token", bcu);
            return Err(super::SecurityError::Tb);
        }
    };
    
    let result = super::xqm(
        cap_id,
        bcu.pcl(),
        bao,
    );
    
    if result.is_err() {
        ACR_.fetch_add(1, Ordering::Relaxed);
        crate::log_warn!("[ISOLATION] Gate denied: {:?} lacks rights for operation", bcu);
    }
    
    result
}


pub fn ysg() -> Result<(), super::SecurityError> {
    drb(Subsystem::Og, CapabilityRights::Cm)
}


pub fn ysh() -> Result<(), super::SecurityError> {
    drb(Subsystem::Og, CapabilityRights::Db)
}


pub fn ysc() -> Result<(), super::SecurityError> {
    drb(Subsystem::As, CapabilityRights::KZ_)
}


pub fn ysa() -> Result<(), super::SecurityError> {
    drb(Subsystem::Jm, CapabilityRights::KZ_)
}


pub fn yse() -> Result<(), super::SecurityError> {
    drb(Subsystem::Tz, CapabilityRights::Vx)
}


pub fn ysb() -> Result<(), super::SecurityError> {
    drb(Subsystem::Ee, CapabilityRights::Mr)
}


pub fn ysf() -> Result<(), super::SecurityError> {
    drb(Subsystem::Df, CapabilityRights::Mz)
}


pub fn yrz() -> Result<(), super::SecurityError> {
    drb(Subsystem::Jh, CapabilityRights::AHA_)
}


pub fn ysd() -> Result<(), super::SecurityError> {
    drb(Subsystem::Hb, CapabilityRights::Mt)
}


pub fn ppp() -> usize {
    PT_.lock().len()
}


pub fn puy() -> u64 {
    ASQ_.load(Ordering::Relaxed)
}


pub fn puz() -> u64 {
    ACR_.load(Ordering::Relaxed)
}


pub fn tzq() -> Vec<String> {
    let mut ak = Vec::new();
    let dr = PT_.lock();
    
    ak.push(String::from("  Subsystem Isolation Status"));
    ak.push(String::from("  ──────────────────────────────────────────────────────────"));
    ak.push(String::from("  Subsystem       │ Isolation     │ Cap ID │ Rights"));
    ak.push(String::from("  ────────────────┼───────────────┼────────┼─────────────"));
    
    for bcu in Subsystem::xx() {
        let j = bcu.j();
        let jy = bcu.tzp();
        let cap_id = dr.get(bcu).map(|ad| ad.0).unwrap_or(0);
        let bap = bcu.nkf();
        
        let vyx = svz(bap);
        
        ak.push(alloc::format!(
            "  {:<16}│ {:<14}│ {:>6} │ {}",
            j, jy, cap_id, vyx
        ));
    }
    
    ak.push(String::from("  ──────────────────────────────────────────────────────────"));
    ak.push(alloc::format!("  Gate checks: {}  |  Violations: {}",
        puy(), puz()));
    
    ak
}


fn svz(bap: CapabilityRights) -> String {
    let mut e = String::new();
    if bap.contains(CapabilityRights::Cm) { e.push('R'); }
    if bap.contains(CapabilityRights::Db) { e.push('W'); }
    if bap.contains(CapabilityRights::Mz) { e.push('X'); }
    if bap.contains(CapabilityRights::Bea) { e.push('D'); }
    if bap.contains(CapabilityRights::Vx) { e.push('C'); }
    if bap.contains(CapabilityRights::Bhs) { e.push('G'); }
    if bap.contains(CapabilityRights::Mt) { e.push('c'); }
    if bap.contains(CapabilityRights::Blt) { e.push('M'); }
    if bap.contains(CapabilityRights::Ayg) { e.push('S'); }
    if bap.contains(CapabilityRights::Xy) { e.push('P'); }
    if e.is_empty() { e.t("none"); }
    e
}
