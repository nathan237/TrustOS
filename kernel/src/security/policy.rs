//! Security Policy Engine
//! 
//! Defines and enforces security policies.

use super::{CapabilityType, CapabilityRights};

/// Security policy rule
#[derive(Debug, Clone, Copy)]
pub struct PolicyRule {
    /// Resource type this rule applies to
    pub resource_type: CapabilityType,
    /// Minimum required rights
    pub required_rights: CapabilityRights,
    /// Allow or deny
    pub allow: bool,
}

/// Default kernel security policies
pub const DEFAULT_POLICIES: &[PolicyRule] = &[
    // Kernel access requires all rights
    PolicyRule {
        resource_type: CapabilityType::Kernel,
        required_rights: CapabilityRights::ALL,
        allow: true,
    },
    // Memory read is commonly allowed
    PolicyRule {
        resource_type: CapabilityType::Memory,
        required_rights: CapabilityRights::READ,
        allow: true,
    },
    // IPC requires at least read/write
    PolicyRule {
        resource_type: CapabilityType::Channel,
        required_rights: CapabilityRights::READ_WRITE,
        allow: true,
    },
];

/// Get default kernel security policies
pub fn default_policies() -> &'static [PolicyRule] {
    DEFAULT_POLICIES
}

/// Invariant checks for security
pub mod invariants {
    use super::*;
    
    /// Invariant: No capability escalation
    pub fn check_no_escalation(
        parent_rights: CapabilityRights,
        child_rights: CapabilityRights,
    ) -> bool {
        parent_rights.contains(child_rights)
    }
    
    /// Invariant: All IPC calls validated
    pub fn check_ipc_capability(cap_rights: CapabilityRights) -> bool {
        cap_rights.contains(CapabilityRights::READ) || 
        cap_rights.contains(CapabilityRights::WRITE)
    }
    
    /// Invariant: Kernel operations require kernel capability
    pub fn check_kernel_access(cap_type: CapabilityType) -> bool {
        matches!(cap_type, CapabilityType::Kernel)
    }
}
