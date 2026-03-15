//! Storage Security Module
//! 
//! Controls access to block devices, partitions, and formatting operations.
//! These are dangerous operations that can destroy data.
//!
//! Security Levels:
//! 1. BlockDeviceRead  - Read sectors (safe)
//! 2. BlockDeviceWrite - Write sectors (can corrupt data)
//! 3. PartitionManagement - Create/delete partitions (can lose all data)
//! 4. DiskFormat - Low-level format (destroys everything)

use super::{CapabilityId, CapabilityRights, CapabilityType, SecurityError};
use spin::Mutex;
use alloc::collections::BTreeSet;

/// Storage operation types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StorageOperation {
    /// Read sectors from disk
    ReadSectors,
    /// Write sectors to disk
    WriteSectors,
    /// Read partition table
    ReadPartitionTable,
    /// Create partition
    CreatePartition,
    /// Delete partition
    DeletePartition,
    /// Resize partition
    ResizePartition,
    /// Format filesystem on partition
    FormatFilesystem,
    /// Low-level disk format (zeros entire disk)
    LowLevelFormat,
    /// Secure erase (cryptographic wipe)
    SecureErase,
}

impl StorageOperation {
    /// Get required capability type for this operation
    pub fn required_capability(&self) -> CapabilityType {
        match self {
            Self::ReadSectors | Self::ReadPartitionTable => CapabilityType::BlockDeviceRead,
            Self::WriteSectors => CapabilityType::BlockDeviceWrite,
            Self::CreatePartition | Self::DeletePartition | Self::ResizePartition => {
                CapabilityType::PartitionManagement
            }
            Self::FormatFilesystem | Self::LowLevelFormat | Self::SecureErase => {
                CapabilityType::DiskFormat
            }
        }
    }

    /// Get required rights for this operation
    pub fn required_rights(&self) -> CapabilityRights {
        match self {
            Self::ReadSectors | Self::ReadPartitionTable => CapabilityRights::READ,
            Self::WriteSectors => CapabilityRights::WRITE,
            Self::CreatePartition => CapabilityRights::CREATE,
            Self::DeletePartition | Self::SecureErase => CapabilityRights::DELETE,
            Self::ResizePartition | Self::FormatFilesystem | Self::LowLevelFormat => {
                CapabilityRights::ALL
            }
        }
    }

    /// Get danger level (0 = safe, 5 = catastrophic)
    pub fn danger_level(&self) -> u8 {
        match self {
            Self::ReadSectors | Self::ReadPartitionTable => 0,
            Self::WriteSectors => 2,
            Self::CreatePartition => 3,
            Self::ResizePartition => 4,
            Self::DeletePartition | Self::FormatFilesystem => 4,
            Self::LowLevelFormat | Self::SecureErase => 5,
        }
    }
}

/// Disk device identifier (port number for AHCI)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DiskId(pub u8);

/// Locked disks that require explicit unlock for dangerous operations
static LOCKED_DISKS: Mutex<BTreeSet<DiskId>> = Mutex::new(BTreeSet::new());

/// Tasks with elevated storage privileges
static PRIVILEGED_TASKS: Mutex<BTreeSet<u64>> = Mutex::new(BTreeSet::new());

/// Initialize storage security
pub fn init() {
    // By default, lock all disks for dangerous operations
    crate::log_debug!("[STORAGE_SEC] Storage security initialized");
}

/// Lock a disk (prevent dangerous operations without explicit unlock)
pub fn lock_disk(disk: DiskId) {
    LOCKED_DISKS.lock().insert(disk);
    crate::log!("[STORAGE_SEC] Disk {} locked", disk.0);
}

/// Unlock a disk for dangerous operations (requires kernel capability)
pub fn unlock_disk(disk: DiskId, cap: CapabilityId) -> Result<(), SecurityError> {
    // Verify caller has kernel-level access
    super::validate(cap, CapabilityRights::ALL)?;
    
    LOCKED_DISKS.lock().remove(&disk);
    crate::log_warn!("[STORAGE_SEC] Disk {} UNLOCKED for dangerous operations", disk.0);
    Ok(())
}

/// Check if disk is locked
pub fn is_disk_locked(disk: DiskId) -> bool {
    LOCKED_DISKS.lock().contains(&disk)
}

/// Grant storage privilege to a task (root only)
pub fn grant_storage_privilege(task_id: u64, cap: CapabilityId) -> Result<(), SecurityError> {
    super::validate(cap, CapabilityRights::GRANT)?;
    PRIVILEGED_TASKS.lock().insert(task_id);
    crate::log!("[STORAGE_SEC] Task {} granted storage privileges", task_id);
    Ok(())
}

/// Revoke storage privilege from a task
pub fn revoke_storage_privilege(task_id: u64) {
    PRIVILEGED_TASKS.lock().remove(&task_id);
    crate::log!("[STORAGE_SEC] Task {} storage privileges revoked", task_id);
}

/// Check if task has storage privileges
pub fn has_storage_privilege(task_id: u64) -> bool {
    PRIVILEGED_TASKS.lock().contains(&task_id)
}

/// Check if operation is allowed on disk
/// 
/// Returns Ok(()) if allowed, Err with reason if denied.
pub fn check_operation(
    disk: DiskId,
    operation: StorageOperation,
    task_id: u64,
) -> Result<(), StorageSecurityError> {
    let danger = operation.danger_level();
    
    // Level 0-1: Always allowed
    if danger <= 1 {
        return Ok(());
    }
    
    // Level 2+: Requires task to have storage privileges
    if !has_storage_privilege(task_id) {
        // Check if task is kernel (task 0)
        if task_id != 0 {
            return Err(StorageSecurityError::InsufficientPrivileges {
                operation,
                required: "storage privilege or root",
            });
        }
    }
    
    // Level 3+: Disk must be explicitly unlocked
    if danger >= 3 && is_disk_locked(disk) {
        return Err(StorageSecurityError::DiskLocked { disk, operation });
    }
    
    // Level 5: Require confirmation (in userspace this would be a dialog)
    if danger >= 5 {
        crate::log_warn!(
            "[STORAGE_SEC] DANGER: Task {} performing {:?} on disk {}",
            task_id, operation, disk.0
        );
    }
    
    Ok(())
}

/// Storage security errors
#[derive(Debug, Clone, Copy)]
pub enum StorageSecurityError {
    /// Task doesn't have required privileges
    InsufficientPrivileges {
        operation: StorageOperation,
        required: &'static str,
    },
    /// Disk is locked for this operation
    DiskLocked {
        disk: DiskId,
        operation: StorageOperation,
    },
    /// Operation requires confirmation
    RequiresConfirmation {
        operation: StorageOperation,
    },
    /// Invalid capability
    InvalidCapability,
}

impl core::fmt::Display for StorageSecurityError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::InsufficientPrivileges { operation, required } => {
                write!(f, "{:?} requires {}", operation, required)
            }
            Self::DiskLocked { disk, operation } => {
                write!(f, "Disk {} is locked, cannot perform {:?}", disk.0, operation)
            }
            Self::RequiresConfirmation { operation } => {
                write!(f, "{:?} requires explicit confirmation", operation)
            }
            Self::InvalidCapability => write!(f, "Invalid or expired capability"),
        }
    }
}

/// Audit log for storage operations
static AUDIT_LOG: Mutex<AuditLog> = Mutex::new(AuditLog::new());

struct AuditLog {
    entries: [Option<AuditEntry>; 64],
    next: usize,
}

impl AuditLog {
    const fn new() -> Self {
        Self {
            entries: [None; 64],
            next: 0,
        }
    }
    
    fn log(&mut self, entry: AuditEntry) {
        self.entries[self.next] = Some(entry);
        self.next = (self.next + 1) % 64;
    }
}

#[derive(Debug, Clone, Copy)]
pub struct AuditEntry {
    pub timestamp: u64,
    pub task_id: u64,
    pub disk: DiskId,
    pub operation: StorageOperation,
    pub allowed: bool,
}

/// Record a storage operation in the audit log
pub fn audit_operation(
    task_id: u64,
    disk: DiskId,
    operation: StorageOperation,
    allowed: bool,
) {
    let entry = AuditEntry {
        timestamp: crate::logger::get_timestamp(),
        task_id,
        disk,
        operation,
        allowed,
    };
    AUDIT_LOG.lock().log(entry);
    
    if !allowed {
        crate::log_warn!(
            "[STORAGE_AUDIT] DENIED: task={} disk={} op={:?}",
            task_id, disk.0, operation
        );
    }
}
