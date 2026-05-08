




use alloc::string::{String, ToString};
use alloc::vec::Vec;
use alloc::vec;
use alloc::sync::Arc;
use alloc::format;
use spin::RwLock;

use super::{
    Au, Bx, Bv, Stat, Ap, FileType,
    K, E, VfsError
};


#[derive(Clone, Copy, Debug)]
enum ProcFileType {
    Fk,
    MemInfo,
    Uptime,
    Version,
    Mounts,
    Cmdline,
    Stat,
}


#[derive(Clone, Copy, Debug)]
enum PidFileType {
    Status,
    Comm,
    Maps,
    Cmdline,
    Cwd,
    Environ,
}









fn ewr(pid: u32) -> K { 1000 + (pid as u64) * 10 }
fn ius(pid: u32, idx: u64) -> K { 1000 + (pid as u64) * 10 + idx }


fn iie(ino: K) -> Option<u32> {
    if ino >= 1000 && (ino - 1000) % 10 == 0 {
        Some(((ino - 1000) / 10) as u32)
    } else {
        None
    }
}


fn iif(ino: K) -> Option<(u32, u64)> {
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

const BFI_: &[&str] = &["status", "comm", "maps", "cmdline", "cwd", "environ"];


struct Pz {
    file_type: ProcFileType,
    ino: K,
}

impl Pz {
    fn generate_content(&self) -> Vec<u8> {
        match self.file_type {
            ProcFileType::Fk => self.gen_cpuinfo(),
            ProcFileType::MemInfo => self.gen_meminfo(),
            ProcFileType::Uptime => self.gen_uptime(),
            ProcFileType::Version => self.gen_version(),
            ProcFileType::Mounts => self.gen_mounts(),
            ProcFileType::Cmdline => self.gen_cmdline(),
            ProcFileType::Stat => self.gen_stat(),
        }
    }
    
    fn gen_cpuinfo(&self) -> Vec<u8> {
        let num_cpus = crate::cpu::smp::ail();
        let mut j = String::new();
        for i in 0..num_cpus {
            j.push_str(&format!(
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
        j.into_bytes()
    }
    
    fn gen_meminfo(&self) -> Vec<u8> {
        let baa = 1024u64;
        let used = crate::memory::heap::used() as u64 / 1024;
        let free = baa.saturating_sub(used);
        
        format!(
            "MemTotal:       {} kB\n\
             MemFree:        {} kB\n\
             MemUsed:        {} kB\n\
             Buffers:        0 kB\n\
             Cached:         0 kB\n\
             SwapTotal:      0 kB\n\
             SwapFree:       0 kB\n",
            baa, free, used
        ).into_bytes()
    }
    
    fn gen_uptime(&self) -> Vec<u8> {
        let gx = crate::logger::eg();
        let im = gx / 100;
        let yt = (gx % 100) as f32 / 100.0;
        format!("{}.{:02} 0.00\n", im, (yt * 100.0) as u32).into_bytes()
    }
    
    fn gen_version(&self) -> Vec<u8> {
        format!(
            "TrustOS version 0.1.0 (rustc) #1 SMP PREEMPT {}\n",
            "Jan 30 2026"
        ).into_bytes()
    }
    
    fn gen_mounts(&self) -> Vec<u8> {
        let mut content = String::new();
        for (path, caa) in crate::vfs::dtl() {
            content.push_str(&format!("{} {} {} rw 0 0\n", caa, path, caa));
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
        let gx = crate::logger::eg();
        let num_cpus = crate::cpu::smp::ail();
        let mut j = format!(
            "cpu  {} 0 {} 0 0 0 0 0 0 0\n",
            gx / 2, gx / 2
        );
        for i in 0..num_cpus {
            j.push_str(&format!(
                "cpu{} {} 0 {} 0 0 0 0 0 0 0\n",
                i, gx / (2 * num_cpus as u64), gx / (2 * num_cpus as u64)
            ));
        }
        j.push_str(&format!(
            "intr 0\nctxt 0\nbtime 0\nprocesses {}\nprocs_running {}\nprocs_blocked 0\n",
            crate::process::count(),
            num_cpus
        ));
        j.into_bytes()
    }
}

impl Bx for Pz {
    fn read(&self, offset: u64, buf: &mut [u8]) -> E<usize> {
        let content = self.generate_content();
        if offset >= content.len() as u64 {
            return Ok(0);
        }
        let start = offset as usize;
        let rz = core::cmp::min(buf.len(), content.len() - start);
        buf[..rz].copy_from_slice(&content[start..start + rz]);
        Ok(rz)
    }
    
    fn write(&self, bkm: u64, _buf: &[u8]) -> E<usize> {
        Err(VfsError::ReadOnly)
    }
    
    fn stat(&self) -> E<Stat> {
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






struct Py {
    pid: u32,
    file_type: PidFileType,
    ino: K,
}

impl Py {
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
        let mut j = String::new();
        crate::process::bwz(self.pid, |aa| {
            let acr = match aa.state {
                crate::process::ProcessState::Created => "N (created)",
                crate::process::ProcessState::Ready => "R (running)",
                crate::process::ProcessState::Running => "R (running)",
                crate::process::ProcessState::Blocked => "S (sleeping)",
                crate::process::ProcessState::Waiting => "S (sleeping)",
                crate::process::ProcessState::Stopped => "T (stopped)",
                crate::process::ProcessState::Zombie => "Z (zombie)",
                crate::process::ProcessState::Dead => "X (dead)",
            };
            j.push_str(&format!("Name:\t{}\n", aa.name));
            j.push_str(&format!("State:\t{}\n", acr));
            j.push_str(&format!("Pid:\t{}\n", aa.pid));
            j.push_str(&format!("PPid:\t{}\n", aa.ppid));
            j.push_str(&format!("Uid:\t{}\t{}\t{}\t{}\n", aa.uid, aa.euid, aa.uid, aa.uid));
            j.push_str(&format!("Gid:\t{}\t{}\t{}\t{}\n", aa.gid, aa.egid, aa.gid, aa.gid));
            j.push_str(&format!("FDSize:\t{}\n", aa.fd_table.len()));
            j.push_str(&format!("VmSize:\t{} kB\n", 
                (aa.memory.heap_end.saturating_sub(aa.memory.code_start)) / 1024));
            j.push_str(&format!("Threads:\t{}\n", aa.children.len() + 1));
            j.push_str(&format!("SigPnd:\t0000000000000000\n"));
            j.push_str(&format!("Cpus_allowed:\tff\n"));
        });
        if j.is_empty() {
            j.push_str("(no such process)\n");
        }
        j.into_bytes()
    }
    
    fn gen_comm(&self) -> Vec<u8> {
        let mut name = String::new();
        crate::process::bwz(self.pid, |aa| {
            name = aa.name.clone();
        });
        format!("{}\n", name).into_bytes()
    }
    
    fn gen_maps(&self) -> Vec<u8> {
        let mut j = String::new();
        crate::process::bwz(self.pid, |aa| {
            let m = &aa.memory;
            if m.code_end > m.code_start {
                j.push_str(&format!("{:016x}-{:016x} r-xp 00000000 00:00 0  [code]\n", 
                    m.code_start, m.code_end));
            }
            if m.data_end > m.data_start {
                j.push_str(&format!("{:016x}-{:016x} rw-p 00000000 00:00 0  [data]\n", 
                    m.data_start, m.data_end));
            }
            if m.heap_end > m.heap_start {
                j.push_str(&format!("{:016x}-{:016x} rw-p 00000000 00:00 0  [heap]\n", 
                    m.heap_start, m.heap_end));
            }
            if m.stack_end > m.stack_start {
                j.push_str(&format!("{:016x}-{:016x} rw-p 00000000 00:00 0  [stack]\n", 
                    m.stack_start, m.stack_end));
            }
            
            if let Some(cr3) = aa.cr3.checked_add(0).filter(|&c| c != 0) {
                for vma in crate::memory::vma::mzk(cr3) {
                    let r = if vma.prot & 1 != 0 { 'r' } else { '-' };
                    let w = if vma.prot & 2 != 0 { 'w' } else { '-' };
                    let x = if vma.prot & 4 != 0 { 'x' } else { '-' };
                    j.push_str(&format!("{:016x}-{:016x} {}{}{}p 00000000 00:00 0  [mmap]\n",
                        vma.start, vma.end, r, w, x));
                }
            }
        });
        if j.is_empty() {
            j.push_str("(no such process)\n");
        }
        j.into_bytes()
    }
    
    fn gen_cmdline(&self) -> Vec<u8> {
        let mut name = String::new();
        crate::process::bwz(self.pid, |aa| {
            name = aa.name.clone();
        });
        format!("{}\0", name).into_bytes()
    }
    
    fn gen_cwd(&self) -> Vec<u8> {
        let mut cwd = String::from("/");
        crate::process::bwz(self.pid, |aa| {
            cwd = aa.cwd.clone();
        });
        format!("{}\n", cwd).into_bytes()
    }
    
    fn gen_environ(&self) -> Vec<u8> {
        let mut j = String::new();
        crate::process::bwz(self.pid, |aa| {
            for (k, v) in &aa.env {
                j.push_str(&format!("{}={}\0", k, v));
            }
        });
        j.into_bytes()
    }
}

impl Bx for Py {
    fn read(&self, offset: u64, buf: &mut [u8]) -> E<usize> {
        let content = self.generate_content();
        if offset >= content.len() as u64 {
            return Ok(0);
        }
        let start = offset as usize;
        let rz = core::cmp::min(buf.len(), content.len() - start);
        buf[..rz].copy_from_slice(&content[start..start + rz]);
        Ok(rz)
    }
    
    fn write(&self, bkm: u64, _buf: &[u8]) -> E<usize> {
        Err(VfsError::ReadOnly)
    }
    
    fn stat(&self) -> E<Stat> {
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





struct Act {
    pid: u32,
}

impl Bv for Act {
    fn lookup(&self, name: &str) -> E<K> {
        for (i, bsr) in BFI_.iter().enumerate() {
            if name == *bsr {
                return Ok(ius(self.pid, (i + 1) as u64));
            }
        }
        Err(VfsError::NotFound)
    }
    
    fn readdir(&self) -> E<Vec<Ap>> {
        let mut entries = vec![
            Ap { name: String::from("."), ino: ewr(self.pid), file_type: FileType::Directory },
            Ap { name: String::from(".."), ino: 1, file_type: FileType::Directory },
        ];
        for (i, bsr) in BFI_.iter().enumerate() {
            entries.push(Ap {
                name: String::from(*bsr),
                ino: ius(self.pid, (i + 1) as u64),
                file_type: FileType::Regular,
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
            ino: ewr(self.pid),
            file_type: FileType::Directory,
            mode: 0o555,
            ..Default::default()
        })
    }
}






#[derive(Clone)]
struct Ez {
    name: String,
    file_type: ProcFileType,
    ino: K,
}


struct Acu {
    entries: Vec<Ez>,
}

impl Bv for Acu {
    fn lookup(&self, name: &str) -> E<K> {
        
        for entry in &self.entries {
            if entry.name == name {
                return Ok(entry.ino);
            }
        }
        
        if let Ok(pid) = name.parse::<u32>() {
            
            let exists = crate::process::list().iter().any(|(aa, _, _)| *aa == pid);
            if exists {
                return Ok(ewr(pid));
            }
        }
        Err(VfsError::NotFound)
    }
    
    fn readdir(&self) -> E<Vec<Ap>> {
        let mut entries = vec![
            Ap { name: String::from("."), ino: 1, file_type: FileType::Directory },
            Ap { name: String::from(".."), ino: 1, file_type: FileType::Directory },
        ];
        
        
        for entry in &self.entries {
            entries.push(Ap {
                name: entry.name.clone(),
                ino: entry.ino,
                file_type: FileType::Regular,
            });
        }
        
        
        for (pid, _name, _state) in crate::process::list() {
            entries.push(Ap {
                name: format!("{}", pid),
                ino: ewr(pid),
                file_type: FileType::Directory,
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
            mode: 0o555,
            ..Default::default()
        })
    }
}


pub struct ProcFs {
    entries: Vec<Ez>,
}

impl ProcFs {
    pub fn new() -> E<Self> {
        let entries = vec![
            Ez { name: String::from("cpuinfo"), file_type: ProcFileType::Fk, ino: 2 },
            Ez { name: String::from("meminfo"), file_type: ProcFileType::MemInfo, ino: 3 },
            Ez { name: String::from("uptime"), file_type: ProcFileType::Uptime, ino: 4 },
            Ez { name: String::from("version"), file_type: ProcFileType::Version, ino: 5 },
            Ez { name: String::from("mounts"), file_type: ProcFileType::Mounts, ino: 6 },
            Ez { name: String::from("cmdline"), file_type: ProcFileType::Cmdline, ino: 7 },
            Ez { name: String::from("stat"), file_type: ProcFileType::Stat, ino: 8 },
        ];
        
        Ok(Self { entries })
    }
    
    fn find_entry(&self, ino: K) -> Option<&Ez> {
        self.entries.iter().find(|e| e.ino == ino)
    }
}

impl Au for ProcFs {
    fn name(&self) -> &str {
        "proc"
    }
    
    fn root_inode(&self) -> K {
        1
    }
    
    fn get_file(&self, ino: K) -> E<Arc<dyn Bx>> {
        
        if let Some(entry) = self.find_entry(ino) {
            return Ok(Arc::new(Pz {
                file_type: entry.file_type,
                ino: entry.ino,
            }));
        }
        
        if let Some((pid, idx)) = iif(ino) {
            let qk = match idx {
                1 => PidFileType::Status,
                2 => PidFileType::Comm,
                3 => PidFileType::Maps,
                4 => PidFileType::Cmdline,
                5 => PidFileType::Cwd,
                6 => PidFileType::Environ,
                _ => return Err(VfsError::NotFound),
            };
            return Ok(Arc::new(Py { pid, file_type: qk, ino }));
        }
        Err(VfsError::NotFound)
    }
    
    fn get_dir(&self, ino: K) -> E<Arc<dyn Bv>> {
        if ino == 1 {
            return Ok(Arc::new(Acu {
                entries: self.entries.clone(),
            }));
        }
        
        if let Some(pid) = iie(ino) {
            return Ok(Arc::new(Act { pid }));
        }
        Err(VfsError::NotDirectory)
    }
    
    fn stat(&self, ino: K) -> E<Stat> {
        if ino == 1 {
            return Ok(Stat {
                ino: 1,
                file_type: FileType::Directory,
                mode: 0o555,
                ..Default::default()
            });
        }
        
        if let Some(entry) = self.find_entry(ino) {
            let anw = Pz { file_type: entry.file_type, ino: entry.ino }
                .generate_content().len() as u64;
            return Ok(Stat {
                ino: entry.ino,
                file_type: FileType::Regular,
                size: anw,
                mode: 0o444,
                ..Default::default()
            });
        }
        
        if iie(ino).is_some() {
            return Ok(Stat {
                ino,
                file_type: FileType::Directory,
                mode: 0o555,
                ..Default::default()
            });
        }
        
        if let Some((pid, idx)) = iif(ino) {
            let qk = match idx {
                1 => PidFileType::Status,
                2 => PidFileType::Comm,
                3 => PidFileType::Maps,
                4 => PidFileType::Cmdline,
                5 => PidFileType::Cwd,
                6 => PidFileType::Environ,
                _ => return Err(VfsError::NotFound),
            };
            let size = Py { pid, file_type: qk, ino }.generate_content().len() as u64;
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
