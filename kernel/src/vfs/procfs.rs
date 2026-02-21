//! Process Filesystem (/proc)
//!
//! Virtual filesystem providing system information like Linux /proc,
//! including per-process directories (/proc/<pid>/{status,comm,maps,...}).

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

/// Proc file types (system-wide)
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

/// Per-PID file types
#[derive(Clone, Copy, Debug)]
enum PidFileType {
    Status,
    Comm,
    Maps,
    Cmdline,
    Cwd,
    Environ,
}

// ============================================================================
// Inode scheme:
//   1           = /proc root
//   2..8        = system-wide files (cpuinfo, meminfo, ...)
//   1000+pid*10 = /proc/<pid>  (directory)
//   1000+pid*10+1..6 = per-pid files
// ============================================================================

fn pid_dir_ino(pid: u32) -> Ino { 1000 + (pid as u64) * 10 }
fn pid_file_ino(pid: u32, idx: u64) -> Ino { 1000 + (pid as u64) * 10 + idx }

/// Check if an inode is a per-PID directory
fn is_pid_dir_ino(ino: Ino) -> Option<u32> {
    if ino >= 1000 && (ino - 1000) % 10 == 0 {
        Some(((ino - 1000) / 10) as u32)
    } else {
        None
    }
}

/// Check if an inode is a per-PID file
fn is_pid_file_ino(ino: Ino) -> Option<(u32, u64)> {
    if ino >= 1001 {
        let offset = ino - 1000;
        let pid = (offset / 10) as u32;
        let idx = offset % 10;
        if idx >= 1 && idx <= 6 {
            Some((pid, idx))
        } else {
            None
        }
    } else {
        None
    }
}

const PID_FILE_NAMES: &[&str] = &["status", "comm", "maps", "cmdline", "cwd", "environ"];

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
        let num_cpus = crate::cpu::smp::ready_cpu_count();
        let mut s = String::new();
        for i in 0..num_cpus {
            s.push_str(&format!(
                "processor\t: {}\n\
                 vendor_id\t: TrustOS\n\
                 cpu family\t: 6\n\
                 model name\t: TrustOS Virtual CPU\n\
                 cpu MHz\t\t: 1000.000\n\
                 cache size\t: 4096 KB\n\
                 flags\t\t: fpu vme de pse tsc msr pae cx8 apic\n\
                 bogomips\t: 2000.00\n\n",
                i
            ));
        }
        s.into_bytes()
    }
    
    fn gen_meminfo(&self) -> Vec<u8> {
        let total_kb = 1024u64;
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
        let num_cpus = crate::cpu::smp::ready_cpu_count();
        let mut s = format!(
            "cpu  {} 0 {} 0 0 0 0 0 0 0\n",
            ticks / 2, ticks / 2
        );
        for i in 0..num_cpus {
            s.push_str(&format!(
                "cpu{} {} 0 {} 0 0 0 0 0 0 0\n",
                i, ticks / (2 * num_cpus as u64), ticks / (2 * num_cpus as u64)
            ));
        }
        s.push_str(&format!(
            "intr 0\nctxt 0\nbtime 0\nprocesses {}\nprocs_running {}\nprocs_blocked 0\n",
            crate::process::count(),
            num_cpus
        ));
        s.into_bytes()
    }
}

impl FileOps for ProcFile {
    fn read(&self, offset: u64, buf: &mut [u8]) -> VfsResult<usize> {
        let content = self.generate_content();
        if offset >= content.len() as u64 {
            return Ok(0);
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

// ============================================================================
// Per-PID files
// ============================================================================

/// A per-PID virtual file
struct PidFile {
    pid: u32,
    file_type: PidFileType,
    ino: Ino,
}

impl PidFile {
    fn generate_content(&self) -> Vec<u8> {
        match self.file_type {
            PidFileType::Status => self.gen_status(),
            PidFileType::Comm => self.gen_comm(),
            PidFileType::Maps => self.gen_maps(),
            PidFileType::Cmdline => self.gen_cmdline(),
            PidFileType::Cwd => self.gen_cwd(),
            PidFileType::Environ => self.gen_environ(),
        }
    }
    
    fn gen_status(&self) -> Vec<u8> {
        let mut s = String::new();
        crate::process::with_process(self.pid, |p| {
            let state_str = match p.state {
                crate::process::ProcessState::Created => "N (created)",
                crate::process::ProcessState::Ready => "R (running)",
                crate::process::ProcessState::Running => "R (running)",
                crate::process::ProcessState::Blocked => "S (sleeping)",
                crate::process::ProcessState::Waiting => "S (sleeping)",
                crate::process::ProcessState::Stopped => "T (stopped)",
                crate::process::ProcessState::Zombie => "Z (zombie)",
                crate::process::ProcessState::Dead => "X (dead)",
            };
            s.push_str(&format!("Name:\t{}\n", p.name));
            s.push_str(&format!("State:\t{}\n", state_str));
            s.push_str(&format!("Pid:\t{}\n", p.pid));
            s.push_str(&format!("PPid:\t{}\n", p.ppid));
            s.push_str(&format!("Uid:\t{}\t{}\t{}\t{}\n", p.uid, p.euid, p.uid, p.uid));
            s.push_str(&format!("Gid:\t{}\t{}\t{}\t{}\n", p.gid, p.egid, p.gid, p.gid));
            s.push_str(&format!("FDSize:\t{}\n", p.fd_table.len()));
            s.push_str(&format!("VmSize:\t{} kB\n", 
                (p.memory.heap_end.saturating_sub(p.memory.code_start)) / 1024));
            s.push_str(&format!("Threads:\t{}\n", p.children.len() + 1));
            s.push_str(&format!("SigPnd:\t0000000000000000\n"));
            s.push_str(&format!("Cpus_allowed:\tff\n"));
        });
        if s.is_empty() {
            s.push_str("(no such process)\n");
        }
        s.into_bytes()
    }
    
    fn gen_comm(&self) -> Vec<u8> {
        let mut name = String::new();
        crate::process::with_process(self.pid, |p| {
            name = p.name.clone();
        });
        format!("{}\n", name).into_bytes()
    }
    
    fn gen_maps(&self) -> Vec<u8> {
        let mut s = String::new();
        crate::process::with_process(self.pid, |p| {
            let m = &p.memory;
            if m.code_end > m.code_start {
                s.push_str(&format!("{:016x}-{:016x} r-xp 00000000 00:00 0  [code]\n", 
                    m.code_start, m.code_end));
            }
            if m.data_end > m.data_start {
                s.push_str(&format!("{:016x}-{:016x} rw-p 00000000 00:00 0  [data]\n", 
                    m.data_start, m.data_end));
            }
            if m.heap_end > m.heap_start {
                s.push_str(&format!("{:016x}-{:016x} rw-p 00000000 00:00 0  [heap]\n", 
                    m.heap_start, m.heap_end));
            }
            if m.stack_end > m.stack_start {
                s.push_str(&format!("{:016x}-{:016x} rw-p 00000000 00:00 0  [stack]\n", 
                    m.stack_start, m.stack_end));
            }
            // Also show mmap'd VMAs
            if let Some(cr3) = p.cr3.checked_add(0).filter(|&c| c != 0) {
                for vma in crate::memory::vma::list_vmas(cr3) {
                    let r = if vma.prot & 1 != 0 { 'r' } else { '-' };
                    let w = if vma.prot & 2 != 0 { 'w' } else { '-' };
                    let x = if vma.prot & 4 != 0 { 'x' } else { '-' };
                    s.push_str(&format!("{:016x}-{:016x} {}{}{}p 00000000 00:00 0  [mmap]\n",
                        vma.start, vma.end, r, w, x));
                }
            }
        });
        if s.is_empty() {
            s.push_str("(no such process)\n");
        }
        s.into_bytes()
    }
    
    fn gen_cmdline(&self) -> Vec<u8> {
        let mut name = String::new();
        crate::process::with_process(self.pid, |p| {
            name = p.name.clone();
        });
        format!("{}\0", name).into_bytes()
    }
    
    fn gen_cwd(&self) -> Vec<u8> {
        let mut cwd = String::from("/");
        crate::process::with_process(self.pid, |p| {
            cwd = p.cwd.clone();
        });
        format!("{}\n", cwd).into_bytes()
    }
    
    fn gen_environ(&self) -> Vec<u8> {
        let mut s = String::new();
        crate::process::with_process(self.pid, |p| {
            for (k, v) in &p.env {
                s.push_str(&format!("{}={}\0", k, v));
            }
        });
        s.into_bytes()
    }
}

impl FileOps for PidFile {
    fn read(&self, offset: u64, buf: &mut [u8]) -> VfsResult<usize> {
        let content = self.generate_content();
        if offset >= content.len() as u64 {
            return Ok(0);
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

// ============================================================================
// Per-PID directory
// ============================================================================

struct ProcPidDir {
    pid: u32,
}

impl DirOps for ProcPidDir {
    fn lookup(&self, name: &str) -> VfsResult<Ino> {
        for (i, fname) in PID_FILE_NAMES.iter().enumerate() {
            if name == *fname {
                return Ok(pid_file_ino(self.pid, (i + 1) as u64));
            }
        }
        Err(VfsError::NotFound)
    }
    
    fn readdir(&self) -> VfsResult<Vec<DirEntry>> {
        let mut entries = vec![
            DirEntry { name: String::from("."), ino: pid_dir_ino(self.pid), file_type: FileType::Directory },
            DirEntry { name: String::from(".."), ino: 1, file_type: FileType::Directory },
        ];
        for (i, fname) in PID_FILE_NAMES.iter().enumerate() {
            entries.push(DirEntry {
                name: String::from(*fname),
                ino: pid_file_ino(self.pid, (i + 1) as u64),
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
            ino: pid_dir_ino(self.pid),
            file_type: FileType::Directory,
            mode: 0o555,
            ..Default::default()
        })
    }
}

// ============================================================================
// Root directory and filesystem
// ============================================================================

/// Proc entry info (system-wide files)
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
        // Check system-wide files first
        for entry in &self.entries {
            if entry.name == name {
                return Ok(entry.ino);
            }
        }
        // Check if it's a PID
        if let Ok(pid) = name.parse::<u32>() {
            // Verify process exists
            let exists = crate::process::list().iter().any(|(p, _, _)| *p == pid);
            if exists {
                return Ok(pid_dir_ino(pid));
            }
        }
        Err(VfsError::NotFound)
    }
    
    fn readdir(&self) -> VfsResult<Vec<DirEntry>> {
        let mut entries = vec![
            DirEntry { name: String::from("."), ino: 1, file_type: FileType::Directory },
            DirEntry { name: String::from(".."), ino: 1, file_type: FileType::Directory },
        ];
        
        // System-wide files
        for entry in &self.entries {
            entries.push(DirEntry {
                name: entry.name.clone(),
                ino: entry.ino,
                file_type: FileType::Regular,
            });
        }
        
        // Per-process directories
        for (pid, _name, _state) in crate::process::list() {
            entries.push(DirEntry {
                name: format!("{}", pid),
                ino: pid_dir_ino(pid),
                file_type: FileType::Directory,
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
        // System-wide file?
        if let Some(entry) = self.find_entry(ino) {
            return Ok(Arc::new(ProcFile {
                file_type: entry.file_type,
                ino: entry.ino,
            }));
        }
        // Per-PID file?
        if let Some((pid, idx)) = is_pid_file_ino(ino) {
            let ft = match idx {
                1 => PidFileType::Status,
                2 => PidFileType::Comm,
                3 => PidFileType::Maps,
                4 => PidFileType::Cmdline,
                5 => PidFileType::Cwd,
                6 => PidFileType::Environ,
                _ => return Err(VfsError::NotFound),
            };
            return Ok(Arc::new(PidFile { pid, file_type: ft, ino }));
        }
        Err(VfsError::NotFound)
    }
    
    fn get_dir(&self, ino: Ino) -> VfsResult<Arc<dyn DirOps>> {
        if ino == 1 {
            return Ok(Arc::new(ProcRootDir {
                entries: self.entries.clone(),
            }));
        }
        // Per-PID directory?
        if let Some(pid) = is_pid_dir_ino(ino) {
            return Ok(Arc::new(ProcPidDir { pid }));
        }
        Err(VfsError::NotDirectory)
    }
    
    fn stat(&self, ino: Ino) -> VfsResult<Stat> {
        if ino == 1 {
            return Ok(Stat {
                ino: 1,
                file_type: FileType::Directory,
                mode: 0o555,
                ..Default::default()
            });
        }
        // System-wide file?
        if let Some(entry) = self.find_entry(ino) {
            let content_len = ProcFile { file_type: entry.file_type, ino: entry.ino }
                .generate_content().len() as u64;
            return Ok(Stat {
                ino: entry.ino,
                file_type: FileType::Regular,
                size: content_len,
                mode: 0o444,
                ..Default::default()
            });
        }
        // PID directory?
        if is_pid_dir_ino(ino).is_some() {
            return Ok(Stat {
                ino,
                file_type: FileType::Directory,
                mode: 0o555,
                ..Default::default()
            });
        }
        // PID file?
        if let Some((pid, idx)) = is_pid_file_ino(ino) {
            let ft = match idx {
                1 => PidFileType::Status,
                2 => PidFileType::Comm,
                3 => PidFileType::Maps,
                4 => PidFileType::Cmdline,
                5 => PidFileType::Cwd,
                6 => PidFileType::Environ,
                _ => return Err(VfsError::NotFound),
            };
            let size = PidFile { pid, file_type: ft, ino }.generate_content().len() as u64;
            return Ok(Stat {
                ino,
                file_type: FileType::Regular,
                size,
                mode: 0o444,
                ..Default::default()
            });
        }
        Err(VfsError::NotFound)
    }
}
