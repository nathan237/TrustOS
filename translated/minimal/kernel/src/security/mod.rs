












mod capability;
mod policy;
pub mod cpu_features;
pub mod storage;
pub mod isolation;

use spin::Mutex;
use alloc::collections::BTreeMap;
use alloc::vec::Vec;
use core::sync::atomic::{AtomicU64, Ordering};

pub use capability::{Capability, CapabilityId, CapabilityType, CapabilityRights};
pub use capability::{izk, mda, ikq, huo};
pub use cpu_features::{hvq, lpu, hvr, lez};
pub use storage::{StorageOperation, StorageSecurityError, Dn};


static Dl: Mutex<BTreeMap<CapabilityId, Capability>> = Mutex::new(BTreeMap::new());


static CKV_: AtomicU64 = AtomicU64::new(1);


static Qu: AtomicU64 = AtomicU64::new(0);


pub fn init() {
    
    let jbi = Capability::cdl();
    Dl.lock().insert(jbi.id, jbi);
    
    
    let features = cpu_features::init();
    
    
    isolation::mpn();
    
    crate::log!("[SECURITY] Initialized - SMEP:{} SMAP:{} UMIP:{} subsystems:{}",
        if features.smep { "ON" } else { "off" },
        if features.smap { "avail" } else { "off" },
        if features.umip { "ON" } else { "off" },
        isolation::jjq()
    );
}


pub fn fpa(
    cap_type: CapabilityType,
    rights: CapabilityRights,
    owner: u64,
) -> CapabilityId {
    let id = CapabilityId(CKV_.fetch_add(1, Ordering::Relaxed));
    let cap = Capability::new(id, cap_type, rights, owner);
    
    Dl.lock().insert(id, cap);
    
    crate::log_debug!("Created capability {:?} for task {}", id, owner);
    
    id
}


pub fn bpu(cap_id: CapabilityId, abh: CapabilityRights) -> Result<(), SecurityError> {
    let caps = Dl.lock();
    let cap = caps.get(&cap_id).ok_or(SecurityError::InvalidCapability)?;
    
    if !cap.has_rights(abh) {
        Qu.fetch_add(1, Ordering::Relaxed);
        crate::log_warn!("Security violation: capability {:?} lacks rights {:?}", cap_id, abh);
        
        crate::trace::akj(
            crate::trace::EventType::SecurityViolation,
            cap_id.0
        );
        
        return Err(SecurityError::InsufficientRights);
    }
    
    if cap.is_expired() {
        return Err(SecurityError::ExpiredCapability);
    }
    
    Ok(())
}


pub fn ogu(cap_id: CapabilityId) -> Result<(), SecurityError> {
    Dl.lock()
        .remove(&cap_id)
        .ok_or(SecurityError::InvalidCapability)?;
    
    crate::log_debug!("Revoked capability {:?}", cap_id);
    
    Ok(())
}


pub fn derive(
    parent: CapabilityId,
    new_rights: CapabilityRights,
    new_owner: u64,
) -> Result<CapabilityId, SecurityError> {
    let cap_type: CapabilityType;
    {
        let caps = Dl.lock();
        let itk = caps.get(&parent).ok_or(SecurityError::InvalidCapability)?;
        
        
        if !itk.has_rights(new_rights) {
            return Err(SecurityError::InsufficientRights);
        }
        cap_type = itk.cap_type;
    }
    
    let id = fpa(cap_type, new_rights, new_owner);
    
    Ok(id)
}


pub fn prc(
    cap_id: CapabilityId,
    required_type: CapabilityType,
    abh: CapabilityRights,
) -> Result<(), SecurityError> {
    let caps = Dl.lock();
    let cap = caps.get(&cap_id).ok_or(SecurityError::InvalidCapability)?;
    
    
    if cap.cap_type != required_type && cap.cap_type != CapabilityType::Kernel {
        Qu.fetch_add(1, Ordering::Relaxed);
        crate::log_warn!("Security violation: capability {:?} type mismatch (have {:?}, need {:?})",
            cap_id, cap.cap_type, required_type);
        return Err(SecurityError::NotPermitted);
    }
    
    if !cap.has_rights(abh) {
        Qu.fetch_add(1, Ordering::Relaxed);
        return Err(SecurityError::InsufficientRights);
    }
    
    if cap.is_expired() {
        return Err(SecurityError::ExpiredCapability);
    }
    
    cap.use_once();
    Ok(())
}


pub fn myz() -> Vec<(CapabilityId, CapabilityType, CapabilityRights, u64)> {
    Dl.lock()
        .iter()
        .map(|(id, cap)| (*id, cap.cap_type, cap.rights, cap.owner))
        .collect()
}


pub fn qnl(owner: u64) -> Vec<(CapabilityId, CapabilityType, CapabilityRights)> {
    Dl.lock()
        .iter()
        .filter(|(_, cap)| cap.owner == owner)
        .map(|(id, cap)| (*id, cap.cap_type, cap.rights))
        .collect()
}


pub fn qnm(cap_type: CapabilityType) -> Vec<(CapabilityId, u64, CapabilityRights)> {
    Dl.lock()
        .iter()
        .filter(|(_, cap)| cap.cap_type == cap_type)
        .map(|(id, cap)| (*id, cap.owner, cap.rights))
        .collect()
}


pub fn qui(owner: u64) -> usize {
    let mut caps = Dl.lock();
    let aph: Vec<CapabilityId> = caps.iter()
        .filter(|(_, cap)| cap.owner == owner)
        .map(|(id, _)| *id)
        .collect();
    let count = aph.len();
    for id in aph {
        caps.remove(&id);
    }
    if count > 0 {
        crate::log_debug!("Cascade-revoked {} capabilities for owner {}", count, owner);
    }
    count
}


pub fn stats() -> Aeq {
    Aeq {
        active_capabilities: Dl.lock().len(),
        violations: Qu.load(Ordering::Relaxed),
        dynamic_types: huo(),
        subsystems: isolation::jjq(),
    }
}


#[derive(Debug, Clone)]
pub struct Aeq {
    pub active_capabilities: usize,
    pub violations: u64,
    pub dynamic_types: usize,
    pub subsystems: usize,
}


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SecurityError {
    
    InvalidCapability,
    
    InsufficientRights,
    
    ExpiredCapability,
    
    NotPermitted,
}
