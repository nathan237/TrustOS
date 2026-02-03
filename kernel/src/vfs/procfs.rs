//! Process Filesystem (/proc)
//!
//! Virtual filesystem providing system information like Linux /proc.

use alloc::string::{String, ToString};
use alloc::vec::Vec;
use alloc::vec;
use alloc::sync::Arc;
use alloc::format;
use spin::RwLock;

use super::{
    FileSystem, FileOps, DirOps, Stat, DirEntry, FileType,
    Ino, VfsResult, VfsError
};

/// Proc file types
#[derive(Clone, Copy, Debug)]
enum ProcFileType {
    CpuInfo,
    MemInfo,
    Uptime,
    Version,
    Mounts,
    Cmdline,
    Stat,
}

/// Virtual proc file - generates content on read
struct ProcFile {
    file_type: ProcFileType,
    ino: Ino,
}

impl ProcFile {
    fn generate_content(&self) -> Vec<u8> {
        match self.file_type {
            ProcFileType::CpuInfo => self.gen_cpuinfo(),
            ProcFileType::MemInfo => self.gen_meminfo(),
            ProcFileType::Uptime => self.gen_uptime(),
            ProcFileType::Version => self.gen_version(),
            ProcFileType::Mounts => self.gen_mounts(),
            ProcFileType::Cmdline => self.gen_cmdline(),
            ProcFileType::Stat => self.gen_stat(),
        }
    }
    
    fn gen_cpuinfo(&self) -> Vec<u8> {
        format!(
            "processor\t: 0\n\
             vendor_id\t: TrustOS\n\
             cpu family\t: 6\n\
             model name\t: TrustOS Virtual CPU\n\
             cpu MHz\t\t: 1000.000\n\
             cache size\t: 4096 KB\n\
             flags\t\t: fpu vme de pse tsc msr pae cx8 apic\n\
             bogomips\t: 2000.00\n"
        ).into_bytes()
    }
    
    fn gen_meminfo(&self) -> Vec<u8> {
        // Get actual memory stats from our allocator
        let total_kb = 1024u64; // Heap size in KB
        let used = crate::memory::heap::used() as u64 / 1024;
        let free = total_kb.saturating_sub(used);
        
        format!(
            "MemTotal:       {} kB\n\
             MemFree:        {} kB\n\
             MemUsed:        {} kB\n\
             Buffers:        0 kB\n\
             Cached:         0 kB\n\
             SwapTotal:      0 kB\n\
             SwapFree:       0 kB\n",
            total_kb, free, used
        ).into_bytes()
    }
    
    fn gen_uptime(&self) -> Vec<u8> {
        let ticks = crate::logger::get_ticks();
        // Approximate: assume 100 ticks/second (depends on timer config)
        let secs = ticks / 100;
        let frac = (ticks % 100) as f32 / 100.0;
        format!("{}.{:02} 0.00\n", secs, (frac * 100.0) as u32).into_bytes()
    }
    
    fn gen_version(&self) -> Vec<u8> {
        format!(
            "TrustOS version 0.1.0 (rustc) #1 SMP PREEMPT {}\n",
            "Jan 30 2026"
        ).into_bytes()
    }
    
    fn gen_mounts(&self) -> Vec<u8> {
        let mut content = String::new();
        for (path, fstype) in crate::vfs::list_mounts() {
            content.push_str(&format!("{} {} {} rw 0 0\n", fstype, path, fstype));
        }
        if content.is_empty() {
            content.push_str("none / rootfs rw 0 0\n");
        }
        content.into_bytes()
    }
    
    fn gen_cmdline(&self) -> Vec<u8> {
        b"BOOT_IMAGE=/boot/trustos root=/dev/vda\n".to_vec()
    }
    
    fn gen_stat(&self) -> Vec<u8> {
        let ticks = crate::logger::get_ticks();
        format!(
            "cpu  {} 0 {} 0 0 0 0 0 0 0\n\
             cpu0 {} 0 {} 0 0 0 0 0 0 0\n\
             intr 0\n\
             ctxt 0\n\
             btime 0\n\
             processes 1\n\
             procs_running 1\n\
             procs_blocked 0\n",
            ticks / 2, ticks / 2,
            ticks / 2, ticks / 2
        ).into_bytes()
    }
}

impl FileOps for ProcFile {
    fn read(&self, offset: u64, buf: &mut [u8]) -> VfsResult<usize> {
        let content = self.generate_content();
        
        if offset >= content.len() as u64 {
            return Ok(0); // EOF
        }
        
        let start = offset as usize;
        let to_read = core::cmp::min(buf.len(), content.len() - start);
        buf[..to_read].copy_from_slice(&content[start..start + to_read]);
        
        Ok(to_read)
    }
    
    fn write(&self, _offset: u64, _buf: &[u8]) -> VfsResult<usize> {
        Err(VfsError::ReadOnly)
    }
    
    fn stat(&self) -> VfsResult<Stat> {
        let size = self.generate_content().len() as u64;
        Ok(Stat {
            ino: self.ino,
            file_type: FileType::Regular,
            size,
            mode: 0o444,
            ..Default::default()
        })
    }
}

/// Proc entry info
#[derive(Clone)]
struct ProcEntry {
    name: String,
    file_type: ProcFileType,
    ino: Ino,
}

/// ProcFS root directory
struct ProcRootDir {
    entries: Vec<ProcEntry>,
}

impl DirOps for ProcRootDir {
    fn lookup(&self, name: &str) -> VfsResult<Ino> {
        for entry in &self.entries {
            if entry.name == name {
                return Ok(entry.ino);
            }
        }
        Err(VfsError::NotFound)
    }
    
    fn readdir(&self) -> VfsResult<Vec<DirEntry>> {
        let mut entries = vec![
            DirEntry { name: String::from("."), ino: 1, file_type: FileType::Directory },
            DirEntry { name: String::from(".."), ino: 1, file_type: FileType::Directory },
        ];
        
        for entry in &self.entries {
            entries.push(DirEntry {
                name: entry.name.clone(),
                ino: entry.ino,
                file_type: FileType::Regular,
            });
        }
        
        Ok(entries)
    }
    
    fn create(&self, _name: &str, _file_type: FileType) -> VfsResult<Ino> {
        Err(VfsError::ReadOnly)
    }
    
    fn unlink(&self, _name: &str) -> VfsResult<()> {
        Err(VfsError::ReadOnly)
    }
    
    fn stat(&self) -> VfsResult<Stat> {
        Ok(Stat {
            ino: 1,
            file_type: FileType::Directory,
            mode: 0o555,
            ..Default::default()
        })
    }
}

/// ProcFS filesystem
pub struct ProcFs {
    entries: Vec<ProcEntry>,
}

impl ProcFs {
    pub fn new() -> VfsResult<Self> {
        let entries = vec![
            ProcEntry { name: String::from("cpuinfo"), file_type: ProcFileType::CpuInfo, ino: 2 },
            ProcEntry { name: String::from("meminfo"), file_type: ProcFileType::MemInfo, ino: 3 },
            ProcEntry { name: String::from("uptime"), file_type: ProcFileType::Uptime, ino: 4 },
            ProcEntry { name: String::from("version"), file_type: ProcFileType::Version, ino: 5 },
            ProcEntry { name: String::from("mounts"), file_type: ProcFileType::Mounts, ino: 6 },
            ProcEntry { name: String::from("cmdline"), file_type: ProcFileType::Cmdline, ino: 7 },
            ProcEntry { name: String::from("stat"), file_type: ProcFileType::Stat, ino: 8 },
        ];
        
        Ok(Self { entries })
    }
    
    fn find_entry(&self, ino: Ino) -> Option<&ProcEntry> {
        self.entries.iter().find(|e| e.ino == ino)
    }
}

impl FileSystem for ProcFs {
    fn name(&self) -> &str {
        "proc"
    }
    
    fn root_inode(&self) -> Ino {
        1
    }
    
    fn get_file(&self, ino: Ino) -> VfsResult<Arc<dyn FileOps>> {
        let entry = self.find_entry(ino).ok_or(VfsError::NotFound)?;
        Ok(Arc::new(ProcFile {
            file_type: entry.file_type,
            ino: entry.ino,
        }))
    }
    
    fn get_dir(&self, ino: Ino) -> VfsResult<Arc<dyn DirOps>> {
        if ino == 1 {
            Ok(Arc::new(ProcRootDir {
                entries: self.entries.clone(),
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
                mode: 0o555,
                ..Default::default()
            })
        } else if let Some(entry) = self.find_entry(ino) {
            let content_len = ProcFile { file_type: entry.file_type, ino: entry.ino }
                .generate_content().len() as u64;
            Ok(Stat {
                ino: entry.ino,
                file_type: FileType::Regular,
                size: content_len,
                mode: 0o444,
                ..Default::default()
            })
        } else {
            Err(VfsError::NotFound)
        }
    }
}
