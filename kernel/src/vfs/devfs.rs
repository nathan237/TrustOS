//! Device Filesystem (/dev)
//!
//! Provides virtual device files like /dev/null, /dev/zero, /dev/random, etc.

use alloc::string::String;
use alloc::vec::Vec;
use alloc::vec;
use alloc::sync::Arc;
use core::sync::atomic::{AtomicU64, Ordering};

use super::{
    FileSystem, FileOps, DirOps, Stat, DirEntry, FileType, 
    Ino, VfsResult, VfsError
};

/// Device types
#[derive(Clone, Copy, Debug)]
enum DeviceType {
    Null,       // /dev/null - discards all writes
    Zero,       // /dev/zero - infinite zeros
    Random,     // /dev/random - random bytes
    Console,    // /dev/console - serial output
    Vda,        // /dev/vda - virtio block device
}

/// Device file implementation
struct DeviceFile {
    dev_type: DeviceType,
    ino: Ino,
}

impl FileOps for DeviceFile {
    fn read(&self, offset: u64, buf: &mut [u8]) -> VfsResult<usize> {
        match self.dev_type {
            DeviceType::Null => Ok(0), // EOF
            DeviceType::Zero => {
                for b in buf.iter_mut() {
                    *b = 0;
                }
                Ok(buf.len())
            }
            DeviceType::Random => {
                // Simple PRNG for random bytes
                for b in buf.iter_mut() {
                    *b = crate::rng::next_u64() as u8;
                }
                Ok(buf.len())
            }
            DeviceType::Console => {
                // Read from keyboard buffer (blocking, line-buffered)
                // Spin until at least one byte is available
                let mut total = 0usize;
                let max_spins = 500_000_000u64; // Safety limit to avoid infinite loop
                let mut spins = 0u64;
                
                while total == 0 && spins < max_spins {
                    while total < buf.len() {
                        if let Some(ch) = crate::keyboard::read_char() {
                            buf[total] = ch;
                            total += 1;
                            // Line-buffered: return on newline
                            if ch == b'\n' || ch == b'\r' {
                                if ch == b'\r' {
                                    buf[total - 1] = b'\n';
                                }
                                return Ok(total);
                            }
                        } else {
                            break; // No more chars available right now
                        }
                    }
                    if total == 0 {
                        // Yield CPU to let keyboard interrupts fire
                        core::hint::spin_loop();
                        spins += 1;
                    }
                }
                Ok(total)
            }
            DeviceType::Vda => {
                // Block device read via virtio-blk
                if !crate::virtio_blk::is_initialized() {
                    return Err(VfsError::IoError);
                }
                let sector_size = 512u64;
                let start_sector = offset / sector_size;
                let offset_in_sector = (offset % sector_size) as usize;
                let mut total_read = 0usize;
                let mut sect = start_sector;
                let mut pos = offset_in_sector;
                let mut sector_buf = [0u8; 512];
                while total_read < buf.len() {
                    if crate::virtio_blk::read_sector(sect, &mut sector_buf).is_err() {
                        break;
                    }
                    let avail = 512 - pos;
                    let to_copy = core::cmp::min(avail, buf.len() - total_read);
                    buf[total_read..total_read + to_copy].copy_from_slice(&sector_buf[pos..pos + to_copy]);
                    total_read += to_copy;
                    sect += 1;
                    pos = 0;
                }
                Ok(total_read)
            }
        }
    }
    
    fn write(&self, offset: u64, buf: &[u8]) -> VfsResult<usize> {
        match self.dev_type {
            DeviceType::Null => Ok(buf.len()), // Discard
            DeviceType::Zero => Err(VfsError::ReadOnly),
            DeviceType::Random => Err(VfsError::ReadOnly),
            DeviceType::Console => {
                // Write to serial console
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
                let offset_in_sector = (offset % sector_size) as usize;
                let mut total_written = 0usize;
                let mut sect = start_sector;
                let mut pos = offset_in_sector;
                let mut sector_buf = [0u8; 512];
                while total_written < buf.len() {
                    // Read-modify-write if partial sector
                    if pos != 0 || (buf.len() - total_written) < 512 {
                        let _ = crate::virtio_blk::read_sector(sect, &mut sector_buf);
                    }
                    let avail = 512 - pos;
                    let to_copy = core::cmp::min(avail, buf.len() - total_written);
                    sector_buf[pos..pos + to_copy].copy_from_slice(&buf[total_written..total_written + to_copy]);
                    if crate::virtio_blk::write_sector(sect, &sector_buf).is_err() {
                        break;
                    }
                    total_written += to_copy;
                    sect += 1;
                    pos = 0;
                }
                Ok(total_written)
            }
        }
    }
    
    fn stat(&self) -> VfsResult<Stat> {
        let file_type = match self.dev_type {
            DeviceType::Vda => FileType::BlockDevice,
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

/// Device entry info
struct DevEntry {
    name: String,
    dev_type: DeviceType,
    ino: Ino,
}

/// DevFS root directory
struct DevRootDir {
    devices: Vec<DevEntry>,
}

impl DirOps for DevRootDir {
    fn lookup(&self, name: &str) -> VfsResult<Ino> {
        for dev in &self.devices {
            if dev.name == name {
                return Ok(dev.ino);
            }
        }
        Err(VfsError::NotFound)
    }
    
    fn readdir(&self) -> VfsResult<Vec<DirEntry>> {
        let mut entries = vec![
            DirEntry { name: String::from("."), ino: 1, file_type: FileType::Directory },
            DirEntry { name: String::from(".."), ino: 1, file_type: FileType::Directory },
        ];
        
        for dev in &self.devices {
            entries.push(DirEntry {
                name: dev.name.clone(),
                ino: dev.ino,
                file_type: match dev.dev_type {
                    DeviceType::Vda => FileType::BlockDevice,
                    _ => FileType::CharDevice,
                },
            });
        }
        
        Ok(entries)
    }
    
    fn create(&self, _name: &str, _file_type: FileType) -> VfsResult<Ino> {
        Err(VfsError::ReadOnly) // Can't create devices dynamically
    }
    
    fn unlink(&self, _name: &str) -> VfsResult<()> {
        Err(VfsError::ReadOnly)
    }
    
    fn stat(&self) -> VfsResult<Stat> {
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

/// DevFS filesystem
pub struct DevFs {
    devices: Vec<DevEntry>,
    next_ino: AtomicU64,
}

impl DevFs {
    pub fn new() -> VfsResult<Self> {
        let mut fs = Self {
            devices: Vec::new(),
            next_ino: AtomicU64::new(2), // 1 is root
        };
        
        // Register standard devices
        fs.add_device("null", DeviceType::Null);
        fs.add_device("zero", DeviceType::Zero);
        fs.add_device("random", DeviceType::Random);
        fs.add_device("urandom", DeviceType::Random);
        fs.add_device("console", DeviceType::Console);
        fs.add_device("tty", DeviceType::Console);
        
        // Add block device if available
        if crate::virtio_blk::is_initialized() {
            fs.add_device("vda", DeviceType::Vda);
        }
        
        Ok(fs)
    }
    
    fn add_device(&mut self, name: &str, dev_type: DeviceType) {
        let ino = self.next_ino.fetch_add(1, Ordering::SeqCst);
        self.devices.push(DevEntry {
            name: String::from(name),
            dev_type,
            ino,
        });
    }
    
    fn find_device(&self, ino: Ino) -> Option<&DevEntry> {
        self.devices.iter().find(|d| d.ino == ino)
    }
}

impl FileSystem for DevFs {
    fn name(&self) -> &str {
        "devfs"
    }
    
    fn root_inode(&self) -> Ino {
        1
    }
    
    fn get_file(&self, ino: Ino) -> VfsResult<Arc<dyn FileOps>> {
        let dev = self.find_device(ino).ok_or(VfsError::NotFound)?;
        Ok(Arc::new(DeviceFile {
            dev_type: dev.dev_type,
            ino: dev.ino,
        }))
    }
    
    fn get_dir(&self, ino: Ino) -> VfsResult<Arc<dyn DirOps>> {
        if ino == 1 {
            Ok(Arc::new(DevRootDir {
                devices: self.devices.clone(),
            }))
        } else {
            Err(VfsError::NotDirectory)
        }
    }
    
    fn stat(&self, ino: Ino) -> VfsResult<Stat> {
        if ino == 1 {
            Ok(Stat {
                ino: 1,
                file_type: FileType::Directory,
                mode: 0o755,
                ..Default::default()
            })
        } else if let Some(dev) = self.find_device(ino) {
            Ok(Stat {
                ino: dev.ino,
                file_type: match dev.dev_type {
                    DeviceType::Vda => FileType::BlockDevice,
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

impl Clone for DevEntry {
    fn clone(&self) -> Self {
        Self {
            name: self.name.clone(),
            dev_type: self.dev_type,
            ino: self.ino,
        }
    }
}
