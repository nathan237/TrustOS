//! Shared Filesystem for VMs (VirtFS)
//!
//! Système de fichiers partagé entre l'host et les guests
//! Utilise un protocole simple de type 9P simplifié

use alloc::string::String;
use alloc::vec::Vec;
use alloc::vec;
use alloc::format;
use alloc::collections::BTreeMap;
use spin::Mutex;

/// VirtFS operation codes
#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum VirtFsOp {
    Version = 0,
    Attach = 1,
    Walk = 2,
    Open = 3,
    Read = 4,
    Write = 5,
    Close = 6,
    Stat = 7,
    ReadDir = 8,
    Create = 9,
    Remove = 10,
    Mkdir = 11,
}

impl TryFrom<u32> for VirtFsOp {
    type Error = ();
    
    fn try_from(value: u32) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(VirtFsOp::Version),
            1 => Ok(VirtFsOp::Attach),
            2 => Ok(VirtFsOp::Walk),
            3 => Ok(VirtFsOp::Open),
            4 => Ok(VirtFsOp::Read),
            5 => Ok(VirtFsOp::Write),
            6 => Ok(VirtFsOp::Close),
            7 => Ok(VirtFsOp::Stat),
            8 => Ok(VirtFsOp::ReadDir),
            9 => Ok(VirtFsOp::Create),
            10 => Ok(VirtFsOp::Remove),
            11 => Ok(VirtFsOp::Mkdir),
            _ => Err(()),
        }
    }
}

/// VirtFS error codes
#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum VirtFsError {
    Success = 0,
    NotFound = 1,
    PermissionDenied = 2,
    IoError = 3,
    InvalidOp = 4,
    NotDir = 5,
    IsDir = 6,
    NotEmpty = 7,
    Exists = 8,
    NoSpace = 9,
    InvalidFd = 10,
}

/// File descriptor info
#[derive(Debug, Clone)]
struct FdInfo {
    path: String,
    is_dir: bool,
    offset: u64,
}

/// Shared mount point
#[derive(Debug, Clone)]
pub struct SharedMount {
    /// Host path (in TrustOS VFS)
    pub host_path: String,
    /// Guest mount point
    pub guest_path: String,
    /// Read-only
    pub readonly: bool,
}

/// VirtFS instance for a VM
pub struct VirtFs {
    vm_id: u64,
    /// Shared mounts
    mounts: Vec<SharedMount>,
    /// Open file descriptors
    fds: BTreeMap<u32, FdInfo>,
    /// Next FD number
    next_fd: u32,
}

impl VirtFs {
    pub fn new(vm_id: u64) -> Self {
        VirtFs {
            vm_id,
            mounts: Vec::new(),
            fds: BTreeMap::new(),
            next_fd: 3, // 0, 1, 2 reserved for stdin/stdout/stderr
        }
    }
    
    /// Add a shared mount
    pub fn add_mount(&mut self, host_path: &str, guest_path: &str, readonly: bool) {
        self.mounts.push(SharedMount {
            host_path: String::from(host_path),
            guest_path: String::from(guest_path),
            readonly,
        });
        crate::serial_println!("[VirtFS] VM {} mounted {} -> {} (ro={})", 
                              self.vm_id, host_path, guest_path, readonly);
    }
    
    /// Resolve guest path to host path
    fn resolve_path(&self, guest_path: &str) -> Option<(String, bool)> {
        for mount in &self.mounts {
            if guest_path.starts_with(&mount.guest_path) {
                let relative = &guest_path[mount.guest_path.len()..];
                let relative = relative.trim_start_matches('/');
                let host_full = if mount.host_path.ends_with('/') || relative.is_empty() {
                    format!("{}{}", mount.host_path, relative)
                } else {
                    format!("{}/{}", mount.host_path, relative)
                };
                return Some((host_full, mount.readonly));
            }
        }
        None
    }
    
    /// Handle a VirtFS request
    pub fn handle_request(&mut self, op: VirtFsOp, args: &[u64]) -> (u32, Vec<u8>) {
        match op {
            VirtFsOp::Version => {
                // Return version info
                let version = b"VirtFS 1.0\0";
                (VirtFsError::Success as u32, version.to_vec())
            }
            
            VirtFsOp::Attach => {
                // Attach to root
                (VirtFsError::Success as u32, vec![])
            }
            
            VirtFsOp::Open => {
                if args.is_empty() {
                    return (VirtFsError::InvalidOp as u32, vec![]);
                }
                
                // args[0] = pointer to path in shared memory
                // For now, return a dummy FD
                let fd = self.next_fd;
                self.next_fd += 1;
                
                self.fds.insert(fd, FdInfo {
                    path: String::from("/"),
                    is_dir: false,
                    offset: 0,
                });
                
                (VirtFsError::Success as u32, fd.to_le_bytes().to_vec())
            }
            
            VirtFsOp::Read => {
                if args.len() < 3 {
                    return (VirtFsError::InvalidOp as u32, vec![]);
                }
                
                let fd = args[0] as u32;
                let _offset = args[1];
                let count = args[2] as usize;
                
                if let Some(fd_info) = self.fds.get(&fd) {
                    // Try to read from VFS - simplified, return empty for now
                    // A full implementation would use crate::vfs::open/read
                    if let Some((_host_path, _readonly)) = self.resolve_path(&fd_info.path) {
                        // Placeholder: return empty data
                        let data: Vec<u8> = vec![];
                        let to_read = core::cmp::min(count, data.len());
                        (VirtFsError::Success as u32, data[..to_read].to_vec())
                    } else {
                        (VirtFsError::NotFound as u32, vec![])
                    }
                } else {
                    (VirtFsError::InvalidFd as u32, vec![])
                }
            }
            
            VirtFsOp::Write => {
                if args.len() < 2 {
                    return (VirtFsError::InvalidOp as u32, vec![]);
                }
                
                let fd = args[0] as u32;
                
                if let Some(fd_info) = self.fds.get(&fd) {
                    if let Some((_host_path, readonly)) = self.resolve_path(&fd_info.path) {
                        if readonly {
                            return (VirtFsError::PermissionDenied as u32, vec![]);
                        }
                        // Write data (simplified)
                        (VirtFsError::Success as u32, vec![])
                    } else {
                        (VirtFsError::NotFound as u32, vec![])
                    }
                } else {
                    (VirtFsError::InvalidFd as u32, vec![])
                }
            }
            
            VirtFsOp::Close => {
                if args.is_empty() {
                    return (VirtFsError::InvalidOp as u32, vec![]);
                }
                
                let fd = args[0] as u32;
                if self.fds.remove(&fd).is_some() {
                    (VirtFsError::Success as u32, vec![])
                } else {
                    (VirtFsError::InvalidFd as u32, vec![])
                }
            }
            
            VirtFsOp::Stat => {
                // Return file stat (simplified)
                let stat = [0u64; 4]; // size, mode, mtime, type
                let bytes: Vec<u8> = stat.iter()
                    .flat_map(|&v| v.to_le_bytes())
                    .collect();
                (VirtFsError::Success as u32, bytes)
            }
            
            VirtFsOp::ReadDir => {
                // Return directory entries
                (VirtFsError::Success as u32, vec![])
            }
            
            _ => (VirtFsError::InvalidOp as u32, vec![]),
        }
    }
}

/// VirtFS manager
pub struct VirtFsManager {
    instances: BTreeMap<u64, VirtFs>,
}

impl VirtFsManager {
    pub const fn new() -> Self {
        VirtFsManager {
            instances: BTreeMap::new(),
        }
    }
    
    pub fn create(&mut self, vm_id: u64) -> &mut VirtFs {
        self.instances.entry(vm_id).or_insert_with(|| VirtFs::new(vm_id))
    }
    
    pub fn get(&mut self, vm_id: u64) -> Option<&mut VirtFs> {
        self.instances.get_mut(&vm_id)
    }
    
    pub fn remove(&mut self, vm_id: u64) {
        self.instances.remove(&vm_id);
    }
}

static VIRTFS_MGR: Mutex<VirtFsManager> = Mutex::new(VirtFsManager::new());

/// Create a VirtFS for a VM
pub fn create_virtfs(vm_id: u64) -> () {
    VIRTFS_MGR.lock().create(vm_id);
}

/// Add a mount point
pub fn add_mount(vm_id: u64, host_path: &str, guest_path: &str, readonly: bool) {
    let mut mgr = VIRTFS_MGR.lock();
    if let Some(vfs) = mgr.get(vm_id) {
        vfs.add_mount(host_path, guest_path, readonly);
    }
}

/// Remove VirtFS for a VM
pub fn remove_virtfs(vm_id: u64) {
    VIRTFS_MGR.lock().remove(vm_id);
}

/// Handle VirtFS hypercall
pub fn handle_hypercall(vm_id: u64, op: u32, args: &[u64]) -> (u32, Vec<u8>) {
    let mut mgr = VIRTFS_MGR.lock();
    
    if let Some(vfs) = mgr.get(vm_id) {
        if let Ok(virtfs_op) = VirtFsOp::try_from(op) {
            vfs.handle_request(virtfs_op, args)
        } else {
            (VirtFsError::InvalidOp as u32, vec![])
        }
    } else {
        (VirtFsError::NotFound as u32, vec![])
    }
}
