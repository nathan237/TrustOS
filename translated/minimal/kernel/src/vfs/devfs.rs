



use alloc::string::String;
use alloc::vec::Vec;
use alloc::vec;
use alloc::sync::Arc;
use core::sync::atomic::{AtomicU64, Ordering};

use super::{
    Au, Bx, Bv, Stat, Ap, FileType, 
    K, E, VfsError
};


#[derive(Clone, Copy, Debug)]
enum DeviceType {
    Null,       
    Zero,       
    Random,     
    Hk,    
    Vda,        
}


struct Xu {
    dev_type: DeviceType,
    ino: K,
}

impl Bx for Xu {
    fn read(&self, offset: u64, buf: &mut [u8]) -> E<usize> {
        match self.dev_type {
            DeviceType::Null => Ok(0), 
            DeviceType::Zero => {
                for b in buf.iter_mut() {
                    *b = 0;
                }
                Ok(buf.len())
            }
            DeviceType::Random => {
                
                for b in buf.iter_mut() {
                    *b = crate::rng::dvf() as u8;
                }
                Ok(buf.len())
            }
            DeviceType::Hk => {
                
                
                let mut av = 0usize;
                let ndl = 500_000_000u64; 
                let mut my = 0u64;
                
                while av == 0 && my < ndl {
                    while av < buf.len() {
                        if let Some(ch) = crate::keyboard::ya() {
                            buf[av] = ch;
                            av += 1;
                            
                            if ch == b'\n' || ch == b'\r' {
                                if ch == b'\r' {
                                    buf[av - 1] = b'\n';
                                }
                                return Ok(av);
                            }
                        } else {
                            break; 
                        }
                    }
                    if av == 0 {
                        
                        core::hint::spin_loop();
                        my += 1;
                    }
                }
                Ok(av)
            }
            DeviceType::Vda => {
                
                if !crate::virtio_blk::is_initialized() {
                    return Err(VfsError::IoError);
                }
                let sector_size = 512u64;
                let start_sector = offset / sector_size;
                let afl = (offset % sector_size) as usize;
                let mut dfr = 0usize;
                let mut dec = start_sector;
                let mut pos = afl;
                let mut mx = [0u8; 512];
                while dfr < buf.len() {
                    if crate::virtio_blk::read_sector(dec, &mut mx).is_err() {
                        break;
                    }
                    let avail = 512 - pos;
                    let od = core::cmp::min(avail, buf.len() - dfr);
                    buf[dfr..dfr + od].copy_from_slice(&mx[pos..pos + od]);
                    dfr += od;
                    dec += 1;
                    pos = 0;
                }
                Ok(dfr)
            }
        }
    }
    
    fn write(&self, offset: u64, buf: &[u8]) -> E<usize> {
        match self.dev_type {
            DeviceType::Null => Ok(buf.len()), 
            DeviceType::Zero => Err(VfsError::ReadOnly),
            DeviceType::Random => Err(VfsError::ReadOnly),
            DeviceType::Hk => {
                
                for &b in buf {
                    crate::serial_print!("{}", b as char);
                }
                Ok(buf.len())
            }
            DeviceType::Vda => {
                if !crate::virtio_blk::is_initialized() {
                    return Err(VfsError::IoError);
                }
                let sector_size = 512u64;
                let start_sector = offset / sector_size;
                let afl = (offset % sector_size) as usize;
                let mut cry = 0usize;
                let mut dec = start_sector;
                let mut pos = afl;
                let mut mx = [0u8; 512];
                while cry < buf.len() {
                    
                    if pos != 0 || (buf.len() - cry) < 512 {
                        let _ = crate::virtio_blk::read_sector(dec, &mut mx);
                    }
                    let avail = 512 - pos;
                    let od = core::cmp::min(avail, buf.len() - cry);
                    mx[pos..pos + od].copy_from_slice(&buf[cry..cry + od]);
                    if crate::virtio_blk::write_sector(dec, &mx).is_err() {
                        break;
                    }
                    cry += od;
                    dec += 1;
                    pos = 0;
                }
                Ok(cry)
            }
        }
    }
    
    fn stat(&self) -> E<Stat> {
        let file_type = match self.dev_type {
            DeviceType::Vda => FileType::Ak,
            _ => FileType::CharDevice,
        };
        
        Ok(Stat {
            ino: self.ino,
            file_type,
            size: 0,
            blocks: 0,
            block_size: 512,
            mode: 0o666,
            ..Default::default()
        })
    }
}


struct Ll {
    name: String,
    dev_type: DeviceType,
    ino: K,
}


struct Xt {
    devices: Vec<Ll>,
}

impl Bv for Xt {
    fn lookup(&self, name: &str) -> E<K> {
        for s in &self.devices {
            if s.name == name {
                return Ok(s.ino);
            }
        }
        Err(VfsError::NotFound)
    }
    
    fn readdir(&self) -> E<Vec<Ap>> {
        let mut entries = vec![
            Ap { name: String::from("."), ino: 1, file_type: FileType::Directory },
            Ap { name: String::from(".."), ino: 1, file_type: FileType::Directory },
        ];
        
        for s in &self.devices {
            entries.push(Ap {
                name: s.name.clone(),
                ino: s.ino,
                file_type: match s.dev_type {
                    DeviceType::Vda => FileType::Ak,
                    _ => FileType::CharDevice,
                },
            });
        }
        
        Ok(entries)
    }
    
    fn create(&self, _name: &str, _file_type: FileType) -> E<K> {
        Err(VfsError::ReadOnly) 
    }
    
    fn unlink(&self, _name: &str) -> E<()> {
        Err(VfsError::ReadOnly)
    }
    
    fn stat(&self) -> E<Stat> {
        Ok(Stat {
            ino: 1,
            file_type: FileType::Directory,
            size: 0,
            blocks: 0,
            block_size: 512,
            mode: 0o755,
            ..Default::default()
        })
    }
}


pub struct DevFs {
    devices: Vec<Ll>,
    next_ino: AtomicU64,
}

impl DevFs {
    pub fn new() -> E<Self> {
        let mut fs = Self {
            devices: Vec::new(),
            next_ino: AtomicU64::new(2), 
        };
        
        
        fs.add_device("null", DeviceType::Null);
        fs.add_device("zero", DeviceType::Zero);
        fs.add_device("random", DeviceType::Random);
        fs.add_device("urandom", DeviceType::Random);
        fs.add_device("console", DeviceType::Hk);
        fs.add_device("tty", DeviceType::Hk);
        
        
        if crate::virtio_blk::is_initialized() {
            fs.add_device("vda", DeviceType::Vda);
        }
        
        Ok(fs)
    }
    
    fn add_device(&mut self, name: &str, dev_type: DeviceType) {
        let ino = self.next_ino.fetch_add(1, Ordering::SeqCst);
        self.devices.push(Ll {
            name: String::from(name),
            dev_type,
            ino,
        });
    }
    
    fn find_device(&self, ino: K) -> Option<&Ll> {
        self.devices.iter().find(|d| d.ino == ino)
    }
}

impl Au for DevFs {
    fn name(&self) -> &str {
        "devfs"
    }
    
    fn root_inode(&self) -> K {
        1
    }
    
    fn get_file(&self, ino: K) -> E<Arc<dyn Bx>> {
        let s = self.find_device(ino).ok_or(VfsError::NotFound)?;
        Ok(Arc::new(Xu {
            dev_type: s.dev_type,
            ino: s.ino,
        }))
    }
    
    fn get_dir(&self, ino: K) -> E<Arc<dyn Bv>> {
        if ino == 1 {
            Ok(Arc::new(Xt {
                devices: self.devices.clone(),
            }))
        } else {
            Err(VfsError::NotDirectory)
        }
    }
    
    fn stat(&self, ino: K) -> E<Stat> {
        if ino == 1 {
            Ok(Stat {
                ino: 1,
                file_type: FileType::Directory,
                mode: 0o755,
                ..Default::default()
            })
        } else if let Some(s) = self.find_device(ino) {
            Ok(Stat {
                ino: s.ino,
                file_type: match s.dev_type {
                    DeviceType::Vda => FileType::Ak,
                    _ => FileType::CharDevice,
                },
                mode: 0o666,
                ..Default::default()
            })
        } else {
            Err(VfsError::NotFound)
        }
    }
}

impl Clone for Ll {
    fn clone(&self) -> Self {
        Self {
            name: self.name.clone(),
            dev_type: self.dev_type,
            ino: self.ino,
        }
    }
}
