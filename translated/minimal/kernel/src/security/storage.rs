










use super::{CapabilityId, CapabilityRights, CapabilityType, SecurityError};
use spin::Mutex;
use alloc::collections::BTreeSet;


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StorageOperation {
    
    ReadSectors,
    
    WriteSectors,
    
    ReadPartitionTable,
    
    CreatePartition,
    
    DeletePartition,
    
    ResizePartition,
    
    FormatFilesystem,
    
    LowLevelFormat,
    
    SecureErase,
}

impl StorageOperation {
    
    pub fn qud(&self) -> CapabilityType {
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

    
    pub fn abh(&self) -> CapabilityRights {
        match self {
            Self::ReadSectors | Self::ReadPartitionTable => CapabilityRights::Ba,
            Self::WriteSectors => CapabilityRights::Bh,
            Self::CreatePartition => CapabilityRights::Jq,
            Self::DeletePartition | Self::SecureErase => CapabilityRights::Xl,
            Self::ResizePartition | Self::FormatFilesystem | Self::LowLevelFormat => {
                CapabilityRights::Fi
            }
        }
    }

    
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


#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Dn(pub u8);


static AGJ_: Mutex<BTreeSet<Dn>> = Mutex::new(BTreeSet::new());


static AIK_: Mutex<BTreeSet<u64>> = Mutex::new(BTreeSet::new());


pub fn init() {
    
    crate::log_debug!("[STORAGE_SEC] Storage security initialized");
}


pub fn qoe(disk: Dn) {
    AGJ_.lock().insert(disk);
    crate::log!("[STORAGE_SEC] Disk {} locked", disk.0);
}


pub fn rbg(disk: Dn, cap: CapabilityId) -> Result<(), SecurityError> {
    
    super::bpu(cap, CapabilityRights::Fi)?;
    
    AGJ_.lock().remove(&disk);
    crate::log_warn!("[STORAGE_SEC] Disk {} UNLOCKED for dangerous operations", disk.0);
    Ok(())
}


pub fn msj(disk: Dn) -> bool {
    AGJ_.lock().contains(&disk)
}


pub fn qjv(task_id: u64, cap: CapabilityId) -> Result<(), SecurityError> {
    super::bpu(cap, CapabilityRights::Ze)?;
    AIK_.lock().insert(task_id);
    crate::log!("[STORAGE_SEC] Task {} granted storage privileges", task_id);
    Ok(())
}


pub fn quj(task_id: u64) {
    AIK_.lock().remove(&task_id);
    crate::log!("[STORAGE_SEC] Task {} storage privileges revoked", task_id);
}


pub fn mke(task_id: u64) -> bool {
    AIK_.lock().contains(&task_id)
}




pub fn flh(
    disk: Dn,
    operation: StorageOperation,
    task_id: u64,
) -> Result<(), StorageSecurityError> {
    let fqs = operation.danger_level();
    
    
    if fqs <= 1 {
        return Ok(());
    }
    
    
    if !mke(task_id) {
        
        if task_id != 0 {
            return Err(StorageSecurityError::InsufficientPrivileges {
                operation,
                aov: "storage privilege or root",
            });
        }
    }
    
    
    if fqs >= 3 && msj(disk) {
        return Err(StorageSecurityError::DiskLocked { disk, operation });
    }
    
    
    if fqs >= 5 {
        crate::log_warn!(
            "[STORAGE_SEC] DANGER: Task {} performing {:?} on disk {}",
            task_id, operation, disk.0
        );
    }
    
    Ok(())
}


#[derive(Debug, Clone, Copy)]
pub enum StorageSecurityError {
    
    InsufficientPrivileges {
        operation: StorageOperation,
        aov: &'static str,
    },
    
    DiskLocked {
        disk: Dn,
        operation: StorageOperation,
    },
    
    RequiresConfirmation {
        operation: StorageOperation,
    },
    
    InvalidCapability,
}

impl core::fmt::Display for StorageSecurityError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::InsufficientPrivileges { operation, aov } => {
                write!(f, "{:?} requires {}", operation, aov)
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


static SB_: Mutex<AuditLog> = Mutex::new(AuditLog::new());

struct AuditLog {
    entries: [Option<Eb>; 64],
    next: usize,
}

impl AuditLog {
    const fn new() -> Self {
        Self {
            entries: [None; 64],
            next: 0,
        }
    }
    
    fn log(&mut self, entry: Eb) {
        self.entries[self.next] = Some(entry);
        self.next = (self.next + 1) % 64;
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Eb {
    pub timestamp: u64,
    pub task_id: u64,
    pub disk: Dn,
    pub operation: StorageOperation,
    pub bxl: bool,
}


pub fn dib(
    task_id: u64,
    disk: Dn,
    operation: StorageOperation,
    bxl: bool,
) {
    let entry = Eb {
        timestamp: crate::logger::ckc(),
        task_id,
        disk,
        operation,
        bxl,
    };
    SB_.lock().log(entry);
    
    if !bxl {
        crate::log_warn!(
            "[STORAGE_AUDIT] DENIED: task={} disk={} op={:?}",
            task_id, disk.0, operation
        );
    }
}
