//! Capability-Based Security Subsystem
//! 
//! All resource access is mediated through unforgeable capability tokens.
//! Minimal Trusted Computing Base (TCB).
//! Also provides CPU security feature management (SMAP, SMEP, etc.)
//!
//! Architecture (see GitHub issues #1, #4):
//! - Each subsystem (disk, network, hypervisor, etc.) receives a dedicated
//!   capability token at boot with the minimum rights it needs.
//! - Operations must present a valid capability before accessing resources.
//! - Capabilities can be dynamically registered for extensibility.
//! - The isolation module (`isolation.rs`) enforces boundary checks.

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
pub use capability::{register_dynamic_type, get_dynamic_type_info, list_dynamic_types, dynamic_type_count};
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
    
    // Initialize subsystem isolation boundaries (issue #1)
    isolation::init_subsystem_capabilities();
    
    crate::log!("[SECURITY] Initialized - SMEP:{} SMAP:{} UMIP:{} subsystems:{}",
        if features.smep { "ON" } else { "off" },
        if features.smap { "avail" } else { "off" },
        if features.umip { "ON" } else { "off" },
        isolation::subsystem_count()
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

/// Validate capability for a specific type AND rights (stronger than validate())
pub fn validate_typed(
    cap_id: CapabilityId,
    required_type: CapabilityType,
    required_rights: CapabilityRights,
) -> Result<(), SecurityError> {
    let caps = CAPABILITIES.lock();
    let cap = caps.get(&cap_id).ok_or(SecurityError::InvalidCapability)?;
    
    // Check type match (Kernel type acts as superuser)
    if cap.cap_type != required_type && cap.cap_type != CapabilityType::Kernel {
        VIOLATIONS.fetch_add(1, Ordering::Relaxed);
        crate::log_warn!("Security violation: capability {:?} type mismatch (have {:?}, need {:?})",
            cap_id, cap.cap_type, required_type);
        return Err(SecurityError::NotPermitted);
    }
    
    if !cap.has_rights(required_rights) {
        VIOLATIONS.fetch_add(1, Ordering::Relaxed);
        return Err(SecurityError::InsufficientRights);
    }
    
    if cap.is_expired() {
        return Err(SecurityError::ExpiredCapability);
    }
    
    cap.use_once();
    Ok(())
}

/// List all active capabilities
pub fn list_capabilities() -> Vec<(CapabilityId, CapabilityType, CapabilityRights, u64)> {
    CAPABILITIES.lock()
        .iter()
        .map(|(id, cap)| (*id, cap.cap_type, cap.rights, cap.owner))
        .collect()
}

/// List capabilities by owner
pub fn list_by_owner(owner: u64) -> Vec<(CapabilityId, CapabilityType, CapabilityRights)> {
    CAPABILITIES.lock()
        .iter()
        .filter(|(_, cap)| cap.owner == owner)
        .map(|(id, cap)| (*id, cap.cap_type, cap.rights))
        .collect()
}

/// List capabilities by type
pub fn list_by_type(cap_type: CapabilityType) -> Vec<(CapabilityId, u64, CapabilityRights)> {
    CAPABILITIES.lock()
        .iter()
        .filter(|(_, cap)| cap.cap_type == cap_type)
        .map(|(id, cap)| (*id, cap.owner, cap.rights))
        .collect()
}

/// Revoke all capabilities owned by a specific owner (cascading revocation)
pub fn revoke_by_owner(owner: u64) -> usize {
    let mut caps = CAPABILITIES.lock();
    let to_remove: Vec<CapabilityId> = caps.iter()
        .filter(|(_, cap)| cap.owner == owner)
        .map(|(id, _)| *id)
        .collect();
    let count = to_remove.len();
    for id in to_remove {
        caps.remove(&id);
    }
    if count > 0 {
        crate::log_debug!("Cascade-revoked {} capabilities for owner {}", count, owner);
    }
    count
}

/// Get security statistics
pub fn stats() -> SecurityStats {
    SecurityStats {
        active_capabilities: CAPABILITIES.lock().len(),
        violations: VIOLATIONS.load(Ordering::Relaxed),
        dynamic_types: dynamic_type_count(),
        subsystems: isolation::subsystem_count(),
    }
}

/// Security statistics
#[derive(Debug, Clone)]
pub struct SecurityStats {
    pub active_capabilities: usize,
    pub violations: u64,
    pub dynamic_types: usize,
    pub subsystems: usize,
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
