




use alloc::string::String;
use alloc::vec::Vec;
use alloc::vec;
use alloc::format;
use alloc::collections::BTreeMap;
use spin::Mutex;


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


#[derive(Debug, Clone)]
struct Yu {
    path: String,
    is_dir: bool,
    offset: u64,
}


#[derive(Debug, Clone)]
pub struct Aeu {
    
    pub host_path: String,
    
    pub guest_path: String,
    
    pub readonly: bool,
}


pub struct VirtFs {
    vm_id: u64,
    
    mounts: Vec<Aeu>,
    
    fds: BTreeMap<u32, Yu>,
    
    next_fd: u32,
}

impl VirtFs {
    pub fn new(vm_id: u64) -> Self {
        VirtFs {
            vm_id,
            mounts: Vec::new(),
            fds: BTreeMap::new(),
            next_fd: 3, 
        }
    }
    
    
    pub fn add_mount(&mut self, host_path: &str, guest_path: &str, readonly: bool) {
        self.mounts.push(Aeu {
            host_path: String::from(host_path),
            guest_path: String::from(guest_path),
            readonly,
        });
        crate::serial_println!("[VirtFS] VM {} mounted {} -> {} (ro={})", 
                              self.vm_id, host_path, guest_path, readonly);
    }
    
    
    fn resolve_path(&self, guest_path: &str) -> Option<(String, bool)> {
        for abd in &self.mounts {
            if guest_path.starts_with(&abd.guest_path) {
                let xj = &guest_path[abd.guest_path.len()..];
                let xj = xj.trim_start_matches('/');
                let mme = if abd.host_path.ends_with('/') || xj.is_empty() {
                    format!("{}{}", abd.host_path, xj)
                } else {
                    format!("{}/{}", abd.host_path, xj)
                };
                return Some((mme, abd.readonly));
            }
        }
        None
    }
    
    
    pub fn handle_request(&mut self, op: VirtFsOp, args: &[u64]) -> (u32, Vec<u8>) {
        match op {
            VirtFsOp::Version => {
                
                let version = b"VirtFS 1.0\0";
                (VirtFsError::Success as u32, version.to_vec())
            }
            
            VirtFsOp::Attach => {
                
                (VirtFsError::Success as u32, vec![])
            }
            
            VirtFsOp::Open => {
                if args.is_empty() {
                    return (VirtFsError::InvalidOp as u32, vec![]);
                }
                
                
                
                let fd = self.next_fd;
                self.next_fd += 1;
                
                self.fds.insert(fd, Yu {
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
                let bkm = args[1];
                let count = args[2] as usize;
                
                if let Some(fd_info) = self.fds.get(&fd) {
                    
                    
                    if let Some((_host_path, _readonly)) = self.resolve_path(&fd_info.path) {
                        
                        let data: Vec<u8> = vec![];
                        let rz = core::cmp::min(count, data.len());
                        (VirtFsError::Success as u32, data[..rz].to_vec())
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
                
                let stat = [0u64; 4]; 
                let bytes: Vec<u8> = stat.iter()
                    .flat_map(|&v| v.to_le_bytes())
                    .collect();
                (VirtFsError::Success as u32, bytes)
            }
            
            VirtFsOp::ReadDir => {
                
                (VirtFsError::Success as u32, vec![])
            }
            
            _ => (VirtFsError::InvalidOp as u32, vec![]),
        }
    }
}


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

static ZL_: Mutex<VirtFsManager> = Mutex::new(VirtFsManager::new());


pub fn hot(vm_id: u64) -> () {
    ZL_.lock().create(vm_id);
}


pub fn add_mount(vm_id: u64, host_path: &str, guest_path: &str, readonly: bool) {
    let mut ng = ZL_.lock();
    if let Some(vfs) = ng.get(vm_id) {
        vfs.add_mount(host_path, guest_path, readonly);
    }
}


pub fn ofb(vm_id: u64) {
    ZL_.lock().remove(vm_id);
}


pub fn fzs(vm_id: u64, op: u32, args: &[u64]) -> (u32, Vec<u8>) {
    let mut ng = ZL_.lock();
    
    if let Some(vfs) = ng.get(vm_id) {
        if let Ok(hbo) = VirtFsOp::try_from(op) {
            vfs.handle_request(hbo, args)
        } else {
            (VirtFsError::InvalidOp as u32, vec![])
        }
    } else {
        (VirtFsError::NotFound as u32, vec![])
    }
}
