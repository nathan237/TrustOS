












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
pub use capability::{pbm, tdn, ojp, nop};
pub use cpu_features::{npw, sle, npx, rxy};
pub use storage::{StorageOperation, StorageSecurityError, Ik};


static Ig: Mutex<BTreeMap<CapabilityId, Capability>> = Mutex::new(BTreeMap::new());


static CHM_: AtomicU64 = AtomicU64::new(1);


static Aoa: AtomicU64 = AtomicU64::new(0);


pub fn init() {
    
    let pdy = Capability::exv();
    Ig.lock().insert(pdy.ad, pdy);
    
    
    let features = cpu_features::init();
    
    
    isolation::tty();
    
    crate::log!("[SECURITY] Initialized - SMEP:{} SMAP:{} UMIP:{} subsystems:{}",
        if features.cia { "ON" } else { "off" },
        if features.cul { "avail" } else { "off" },
        if features.ddd { "ON" } else { "off" },
        isolation::ppp()
    );
}


pub fn klu(
    cap_type: CapabilityType,
    bap: CapabilityRights,
    awj: u64,
) -> CapabilityId {
    let ad = CapabilityId(CHM_.fetch_add(1, Ordering::Relaxed));
    let mh = Capability::new(ad, cap_type, bap, awj);
    
    Ig.lock().insert(ad, mh);
    
    crate::log_debug!("Created capability {:?} for task {}", ad, awj);
    
    ad
}


pub fn dxi(cap_id: CapabilityId, bao: CapabilityRights) -> Result<(), SecurityError> {
    let dr = Ig.lock();
    let mh = dr.get(&cap_id).ok_or(SecurityError::Tb)?;
    
    if !mh.lbf(bao) {
        Aoa.fetch_add(1, Ordering::Relaxed);
        crate::log_warn!("Security violation: capability {:?} lacks rights {:?}", cap_id, bao);
        
        crate::trace::bry(
            crate::trace::EventType::Cms,
            cap_id.0
        );
        
        return Err(SecurityError::Auk);
    }
    
    if mh.hox() {
        return Err(SecurityError::Bgh);
    }
    
    Ok(())
}


pub fn vyp(cap_id: CapabilityId) -> Result<(), SecurityError> {
    Ig.lock()
        .remove(&cap_id)
        .ok_or(SecurityError::Tb)?;
    
    crate::log_debug!("Revoked capability {:?}", cap_id);
    
    Ok(())
}


pub fn derive(
    tu: CapabilityId,
    oqd: CapabilityRights,
    ute: u64,
) -> Result<CapabilityId, SecurityError> {
    let cap_type: CapabilityType;
    {
        let dr = Ig.lock();
        let otu = dr.get(&tu).ok_or(SecurityError::Tb)?;
        
        
        if !otu.lbf(oqd) {
            return Err(SecurityError::Auk);
        }
        cap_type = otu.cap_type;
    }
    
    let ad = klu(cap_type, oqd, ute);
    
    Ok(ad)
}


pub fn xqm(
    cap_id: CapabilityId,
    pcm: CapabilityType,
    bao: CapabilityRights,
) -> Result<(), SecurityError> {
    let dr = Ig.lock();
    let mh = dr.get(&cap_id).ok_or(SecurityError::Tb)?;
    
    
    if mh.cap_type != pcm && mh.cap_type != CapabilityType::Xj {
        Aoa.fetch_add(1, Ordering::Relaxed);
        crate::log_warn!("Security violation: capability {:?} type mismatch (have {:?}, need {:?})",
            cap_id, mh.cap_type, pcm);
        return Err(SecurityError::Cia);
    }
    
    if !mh.lbf(bao) {
        Aoa.fetch_add(1, Ordering::Relaxed);
        return Err(SecurityError::Auk);
    }
    
    if mh.hox() {
        return Err(SecurityError::Bgh);
    }
    
    mh.xpm();
    Ok(())
}


pub fn ufn() -> Vec<(CapabilityId, CapabilityType, CapabilityRights, u64)> {
    Ig.lock()
        .iter()
        .map(|(ad, mh)| (*ad, mh.cap_type, mh.bap, mh.awj))
        .collect()
}


pub fn zaw(awj: u64) -> Vec<(CapabilityId, CapabilityType, CapabilityRights)> {
    Ig.lock()
        .iter()
        .hi(|(_, mh)| mh.awj == awj)
        .map(|(ad, mh)| (*ad, mh.cap_type, mh.bap))
        .collect()
}


pub fn zax(cap_type: CapabilityType) -> Vec<(CapabilityId, u64, CapabilityRights)> {
    Ig.lock()
        .iter()
        .hi(|(_, mh)| mh.cap_type == cap_type)
        .map(|(ad, mh)| (*ad, mh.awj, mh.bap))
        .collect()
}


pub fn zkc(awj: u64) -> usize {
    let mut dr = Ig.lock();
    let cik: Vec<CapabilityId> = dr.iter()
        .hi(|(_, mh)| mh.awj == awj)
        .map(|(ad, _)| *ad)
        .collect();
    let az = cik.len();
    for ad in cik {
        dr.remove(&ad);
    }
    if az > 0 {
        crate::log_debug!("Cascade-revoked {} capabilities for owner {}", az, awj);
    }
    az
}


pub fn cm() -> Bsj {
    Bsj {
        mto: Ig.lock().len(),
        cnt: Aoa.load(Ordering::Relaxed),
        noq: nop(),
        ppr: isolation::ppp(),
    }
}


#[derive(Debug, Clone)]
pub struct Bsj {
    pub mto: usize,
    pub cnt: u64,
    pub noq: usize,
    pub ppr: usize,
}


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SecurityError {
    
    Tb,
    
    Auk,
    
    Bgh,
    
    Cia,
}
