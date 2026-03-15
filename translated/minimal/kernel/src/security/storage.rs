










use super::{CapabilityId, CapabilityRights, CapabilityType, SecurityError};
use spin::Mutex;
use alloc::collections::BTreeSet;


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StorageOperation {
    
    Alx,
    
    Aof,
    
    Axo,
    
    Aqi,
    
    Aqt,
    
    Aya,
    
    Asn,
    
    Ajw,
    
    Aym,
}

impl StorageOperation {
    
    pub fn zjv(&self) -> CapabilityType {
        match self {
            Self::Alx | Self::Axo => CapabilityType::Agr,
            Self::Aof => CapabilityType::Apn,
            Self::Aqi | Self::Aqt | Self::Aya => {
                CapabilityType::Awq
            }
            Self::Asn | Self::Ajw | Self::Aym => {
                CapabilityType::Aqy
            }
        }
    }

    
    pub fn bao(&self) -> CapabilityRights {
        match self {
            Self::Alx | Self::Axo => CapabilityRights::Cm,
            Self::Aof => CapabilityRights::Db,
            Self::Aqi => CapabilityRights::Vx,
            Self::Aqt | Self::Aym => CapabilityRights::Bea,
            Self::Aya | Self::Asn | Self::Ajw => {
                CapabilityRights::Mr
            }
        }
    }

    
    pub fn eom(&self) -> u8 {
        match self {
            Self::Alx | Self::Axo => 0,
            Self::Aof => 2,
            Self::Aqi => 3,
            Self::Aya => 4,
            Self::Aqt | Self::Asn => 4,
            Self::Ajw | Self::Aym => 5,
        }
    }
}


#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Ik(pub u8);


static AEP_: Mutex<BTreeSet<Ik>> = Mutex::new(BTreeSet::new());


static AGQ_: Mutex<BTreeSet<u64>> = Mutex::new(BTreeSet::new());


pub fn init() {
    
    crate::log_debug!("[STORAGE_SEC] Storage security initialized");
}


pub fn zbq(disk: Ik) {
    AEP_.lock().insert(disk);
    crate::log!("[STORAGE_SEC] Disk {} locked", disk.0);
}


pub fn zua(disk: Ik, mh: CapabilityId) -> Result<(), SecurityError> {
    
    super::dxi(mh, CapabilityRights::Mr)?;
    
    AEP_.lock().remove(&disk);
    crate::log_warn!("[STORAGE_SEC] Disk {} UNLOCKED for dangerous operations", disk.0);
    Ok(())
}


pub fn txg(disk: Ik) -> bool {
    AEP_.lock().contains(&disk)
}


pub fn yvl(aod: u64, mh: CapabilityId) -> Result<(), SecurityError> {
    super::dxi(mh, CapabilityRights::Bhs)?;
    AGQ_.lock().insert(aod);
    crate::log!("[STORAGE_SEC] Task {} granted storage privileges", aod);
    Ok(())
}


pub fn zkd(aod: u64) {
    AGQ_.lock().remove(&aod);
    crate::log!("[STORAGE_SEC] Task {} storage privileges revoked", aod);
}


pub fn tnh(aod: u64) -> bool {
    AGQ_.lock().contains(&aod)
}




pub fn khe(
    disk: Ik,
    ayh: StorageOperation,
    aod: u64,
) -> Result<(), StorageSecurityError> {
    let koa = ayh.eom();
    
    
    if koa <= 1 {
        return Ok(());
    }
    
    
    if !tnh(aod) {
        
        if aod != 0 {
            return Err(StorageSecurityError::Bjp {
                ayh,
                cbj: "storage privilege or root",
            });
        }
    }
    
    
    if koa >= 3 && txg(disk) {
        return Err(StorageSecurityError::Beq { disk, ayh });
    }
    
    
    if koa >= 5 {
        crate::log_warn!(
            "[STORAGE_SEC] DANGER: Task {} performing {:?} on disk {}",
            aod, ayh, disk.0
        );
    }
    
    Ok(())
}


#[derive(Debug, Clone, Copy)]
pub enum StorageSecurityError {
    
    Bjp {
        ayh: StorageOperation,
        cbj: &'static str,
    },
    
    Beq {
        disk: Ik,
        ayh: StorageOperation,
    },
    
    Ckl {
        ayh: StorageOperation,
    },
    
    Tb,
}

impl core::fmt::Display for StorageSecurityError {
    fn fmt(&self, bb: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Bjp { ayh, cbj } => {
                write!(bb, "{:?} requires {}", ayh, cbj)
            }
            Self::Beq { disk, ayh } => {
                write!(bb, "Disk {} is locked, cannot perform {:?}", disk.0, ayh)
            }
            Self::Ckl { ayh } => {
                write!(bb, "{:?} requires explicit confirmation", ayh)
            }
            Self::Tb => write!(bb, "Invalid or expired capability"),
        }
    }
}


static RF_: Mutex<AuditLog> = Mutex::new(AuditLog::new());

struct AuditLog {
    ch: [Option<Ke>; 64],
    next: usize,
}

impl AuditLog {
    const fn new() -> Self {
        Self {
            ch: [None; 64],
            next: 0,
        }
    }
    
    fn log(&mut self, bt: Ke) {
        self.ch[self.next] = Some(bt);
        self.next = (self.next + 1) % 64;
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Ke {
    pub aea: u64,
    pub aod: u64,
    pub disk: Ik,
    pub ayh: StorageOperation,
    pub emd: bool,
}


pub fn gzg(
    aod: u64,
    disk: Ik,
    ayh: StorageOperation,
    emd: bool,
) {
    let bt = Ke {
        aea: crate::logger::fjp(),
        aod,
        disk,
        ayh,
        emd,
    };
    RF_.lock().log(bt);
    
    if !emd {
        crate::log_warn!(
            "[STORAGE_AUDIT] DENIED: task={} disk={} op={:?}",
            aod, disk.0, ayh
        );
    }
}
