//! Capability-Based Security Subsystem
//! 
//! All resource access is mediated through unforgeable capability tokens.
//! Minimal Trusted Computing Base (TCB).
//! Also provides CPU security feature management (SMAP, SMEP, etc.)

mod capability;
mod policy;
pub mod cpu_features;
pub mod storage;

use spin::Mutex;
use alloc::collections::BTreeMap;
use core::sync::atomic::{AtomicU64, Ordering};

pub use capability::{Capability, CapabilityId, CapabilityType, CapabilityRights};
pub use cpu_features::{enable_smep, enable_smap, enable_umip, disable_smap_for_user_access};
pub use storage::{StorageOperation, StorageSecurityError, DiskId};

/// Global capability registry
static CAPABILITIES: Mutex<BTreeMap<CapabilityId, Capability>> = Mutex::new(BTreeMap::new());

/// Next capability ID
static NEXT_CAP_ID: AtomicU64 = AtomicU64::new(1);

/// Security violation counter
static VIOLATIONS: AtomicU64 = AtomicU64::new(0);

/// Initialize security subsystem
pub fn init() {
    // Create root capability with all rights
    let root_cap = Capability::root();
    CAPABILITIES.lock().insert(root_cap.id, root_cap);
    
    // Initialize CPU security features (SMEP, SMAP, UMIP)
    let features = cpu_features::init();
    
    crate::log!("[SECURITY] Initialized - SMEP:{} SMAP:{} UMIP:{}",
        if features.smep { "ON" } else { "off" },
        if features.smap { "avail" } else { "off" },  // SMAP disabled for now
        if features.umip { "ON" } else { "off" }
    );
}

/// Create new capability
pub fn create_capability(
    cap_type: CapabilityType,
    rights: CapabilityRights,
    owner: u64,
) -> CapabilityId {
    let id = CapabilityId(NEXT_CAP_ID.fetch_add(1, Ordering::Relaxed));
    let cap = Capability::new(id, cap_type, rights, owner);
    
    CAPABILITIES.lock().insert(id, cap);
    
    crate::log_debug!("Created capability {:?} for task {}", id, owner);
    
    id
}

/// Validate capability for operation
pub fn validate(cap_id: CapabilityId, required_rights: CapabilityRights) -> Result<(), SecurityError> {
    let caps = CAPABILITIES.lock();
    let cap = caps.get(&cap_id).ok_or(SecurityError::InvalidCapability)?;
    
    if !cap.has_rights(required_rights) {
        VIOLATIONS.fetch_add(1, Ordering::Relaxed);
        crate::log_warn!("Security violation: capability {:?} lacks rights {:?}", cap_id, required_rights);
        
        crate::trace::record_event(
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

/// Revoke capability
pub fn revoke(cap_id: CapabilityId) -> Result<(), SecurityError> {
    CAPABILITIES.lock()
        .remove(&cap_id)
        .ok_or(SecurityError::InvalidCapability)?;
    
    crate::log_debug!("Revoked capability {:?}", cap_id);
    
    Ok(())
}

/// Derive new capability with reduced rights
pub fn derive(
    parent: CapabilityId,
    new_rights: CapabilityRights,
    new_owner: u64,
) -> Result<CapabilityId, SecurityError> {
    let cap_type: CapabilityType;
    {
        let caps = CAPABILITIES.lock();
        let parent_cap = caps.get(&parent).ok_or(SecurityError::InvalidCapability)?;
        
        // Can only derive with same or fewer rights
        if !parent_cap.has_rights(new_rights) {
            return Err(SecurityError::InsufficientRights);
        }
        cap_type = parent_cap.cap_type;
    }
    
    let id = create_capability(cap_type, new_rights, new_owner);
    
    Ok(id)
}

/// Get security statistics
pub fn stats() -> SecurityStats {
    SecurityStats {
        active_capabilities: CAPABILITIES.lock().len(),
        violations: VIOLATIONS.load(Ordering::Relaxed),
    }
}

/// Security statistics
#[derive(Debug, Clone)]
pub struct SecurityStats {
    pub active_capabilities: usize,
    pub violations: u64,
}

/// Security error types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SecurityError {
    /// Capability ID not found
    InvalidCapability,
    /// Capability lacks required rights
    InsufficientRights,
    /// Capability has expired
    ExpiredCapability,
    /// Operation not permitted
    NotPermitted,
}
