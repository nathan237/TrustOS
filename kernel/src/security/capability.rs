//! Capability types and structures
//! 
//! Unforgeable capability tokens for resource access control.

use core::sync::atomic::{AtomicU64, Ordering};

/// Unique capability identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CapabilityId(pub u64);

impl CapabilityId {
    pub const ROOT: CapabilityId = CapabilityId(0);
}

/// Type of resource this capability grants access to
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CapabilityType {
    /// Memory region access
    Memory,
    /// IPC channel access
    Channel,
    /// Device access
    Device,
    /// Process control
    Process,
    /// Filesystem access (files/directories)
    Filesystem,
    /// Network access
    Network,
    /// Kernel control (privileged)
    Kernel,
    
    // === Storage security levels ===
    /// Block device read access (safe)
    BlockDeviceRead,
    /// Block device write access (dangerous)
    BlockDeviceWrite,
    /// Partition table management (very dangerous)
    PartitionManagement,
    /// Low-level disk format (destructive)
    DiskFormat,
}

/// Rights that can be granted
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CapabilityRights(u32);

impl CapabilityRights {
    pub const NONE: Self = Self(0);
    pub const READ: Self = Self(1 << 0);
    pub const WRITE: Self = Self(1 << 1);
    pub const EXECUTE: Self = Self(1 << 2);
    pub const DELETE: Self = Self(1 << 3);
    pub const CREATE: Self = Self(1 << 4);
    pub const GRANT: Self = Self(1 << 5);
    pub const ALL: Self = Self(0x3F);
    pub const READ_WRITE: Self = Self(0x03); // READ | WRITE
    
    /// Combine rights
    pub const fn union(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }
    
    /// Check if contains rights
    pub const fn contains(self, other: Self) -> bool {
        (self.0 & other.0) == other.0
    }
}

/// A capability token
#[derive(Debug)]
pub struct Capability {
    /// Unique ID
    pub id: CapabilityId,
    /// Type of resource
    pub cap_type: CapabilityType,
    /// Granted rights
    pub rights: CapabilityRights,
    /// Owning task ID
    pub owner: u64,
    /// Parent capability (for derivation chain)
    pub parent: Option<CapabilityId>,
    /// Creation timestamp
    pub created_at: u64,
    /// Expiration timestamp (0 = never)
    pub expires_at: u64,
    /// Usage counter
    usage_count: AtomicU64,
}

impl Capability {
    /// Create new capability
    pub fn new(
        id: CapabilityId,
        cap_type: CapabilityType,
        rights: CapabilityRights,
        owner: u64,
    ) -> Self {
        Self {
            id,
            cap_type,
            rights,
            owner,
            parent: None,
            created_at: crate::logger::get_timestamp(),
            expires_at: 0,
            usage_count: AtomicU64::new(0),
        }
    }
    
    /// Create root capability with all rights
    pub fn root() -> Self {
        Self {
            id: CapabilityId::ROOT,
            cap_type: CapabilityType::Kernel,
            rights: CapabilityRights::ALL,
            owner: 0,
            parent: None,
            created_at: 0,
            expires_at: 0,
            usage_count: AtomicU64::new(0),
        }
    }
    
    /// Check if capability has required rights
    pub fn has_rights(&self, required: CapabilityRights) -> bool {
        self.rights.contains(required)
    }
    
    /// Check if capability is expired
    pub fn is_expired(&self) -> bool {
        if self.expires_at == 0 {
            return false;
        }
        crate::logger::get_timestamp() > self.expires_at
    }
    
    /// Increment usage counter
    pub fn use_once(&self) {
        self.usage_count.fetch_add(1, Ordering::Relaxed);
    }
    
    /// Get usage count
    pub fn usage(&self) -> u64 {
        self.usage_count.load(Ordering::Relaxed)
    }
}
