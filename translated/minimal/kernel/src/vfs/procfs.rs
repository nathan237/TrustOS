




use alloc::string::{String, Gd};
use alloc::vec::Vec;
use alloc::vec;
use alloc::sync::Arc;
use alloc::format;
use spin::RwLock;

use super::{
    Cc, Et, Ep, Stat, Br, FileType,
    I, B, VfsError
};


#[derive(Clone, Copy, Debug)]
enum ProcFileType {
    Aat,
    Bmd,
    Afh,
    Re,
    Bmq,
    Vz,
    Stat,
}


#[derive(Clone, Copy, Debug)]
enum PidFileType {
    Status,
    Aqd,
    Avh,
    Vz,
    Aqm,
    Arp,
}









fn jjh(ce: u32) -> I { 1000 + (ce as u64) * 10 }
fn ovn(ce: u32, w: u64) -> I { 1000 + (ce as u64) * 10 + w }


fn ogn(dd: I) -> Option<u32> {
    if dd >= 1000 && (dd - 1000) % 10 == 0 {
        Some(((dd - 1000) / 10) as u32)
    } else {
        None
    }
}


fn ogo(dd: I) -> Option<(u32, u64)> {
    if dd >= 1001 {
        let l = dd - 1000;
        let ce = (l / 10) as u32;
        let w = l % 10;
        if w >= 1 && w <= 6 {
            Some((ce, w))
        } else {
            None
        }
    } else {
        None
    }
}

const BDF_: &[&str] = &["status", "comm", "maps", "cmdline", "cwd", "environ"];


struct Alh {
    kd: ProcFileType,
    dd: I,
}

impl Alh {
    fn fjj(&self) -> Vec<u8> {
        match self.kd {
            ProcFileType::Aat => self.tat(),
            ProcFileType::Bmd => self.tbe(),
            ProcFileType::Afh => self.tbt(),
            ProcFileType::Re => self.tbu(),
            ProcFileType::Bmq => self.tbf(),
            ProcFileType::Vz => self.kxz(),
            ProcFileType::Stat => self.tbn(),
        }
    }
    
    fn tat(&self) -> Vec<u8> {
        let bcc = crate::cpu::smp::boc();
        let mut e = String::new();
        for a in 0..bcc {
            e.t(&format!(
                "processor\t: {}\n\
                 vendor_id\t: TrustOS\n\
                 cpu family\t: 6\n\
                 model name\t: TrustOS Virtual CPU\n\
                 cpu MHz\t\t: 1000.000\n\
                 cache size\t: 4096 KB\n\
                 flags\t\t: fpu vme de pse tsc msr pae cx8 apic\n\
                 bogomips\t: 2000.00\n\n",
                a
            ));
        }
        e.cfq()
    }
    
    fn tbe(&self) -> Vec<u8> {
        let cuu = 1024u64;
        let mr = crate::memory::heap::mr() as u64 / 1024;
        let aez = cuu.ao(mr);
        
        format!(
            "MemTotal:       {} kB\n\
             MemFree:        {} kB\n\
             MemUsed:        {} kB\n\
             Buffers:        0 kB\n\
             Cached:         0 kB\n\
             SwapTotal:      0 kB\n\
             SwapFree:       0 kB\n",
            cuu, aez, mr
        ).cfq()
    }
    
    fn tbt(&self) -> Vec<u8> {
        let qb = crate::logger::lh();
        let tv = qb / 100;
        let avw = (qb % 100) as f32 / 100.0;
        format!("{}.{:02} 0.00\n", tv, (avw * 100.0) as u32).cfq()
    }
    
    fn tbu(&self) -> Vec<u8> {
        format!(
            "TrustOS version 0.1.0 (rustc) #1 SMP PREEMPT {}\n",
            "Jan 30 2026"
        ).cfq()
    }
    
    fn tbf(&self) -> Vec<u8> {
        let mut ca = String::new();
        for (path, eqw) in crate::vfs::hqa() {
            ca.t(&format!("{} {} {} rw 0 0\n", eqw, path, eqw));
        }
        if ca.is_empty() {
            ca.t("none / rootfs rw 0 0\n");
        }
        ca.cfq()
    }
    
    fn kxz(&self) -> Vec<u8> {
        b"BOOT_IMAGE=/boot/trustos root=/dev/vda\n".ip()
    }
    
    fn tbn(&self) -> Vec<u8> {
        let qb = crate::logger::lh();
        let bcc = crate::cpu::smp::boc();
        let mut e = format!(
            "cpu  {} 0 {} 0 0 0 0 0 0 0\n",
            qb / 2, qb / 2
        );
        for a in 0..bcc {
            e.t(&format!(
                "cpu{} {} 0 {} 0 0 0 0 0 0 0\n",
                a, qb / (2 * bcc as u64), qb / (2 * bcc as u64)
            ));
        }
        e.t(&format!(
            "intr 0\nctxt 0\nbtime 0\nprocesses {}\nprocs_running {}\nprocs_blocked 0\n",
            crate::process::az(),
            bcc
        ));
        e.cfq()
    }
}

impl Et for Alh {
    fn read(&self, l: u64, k: &mut [u8]) -> B<usize> {
        let ca = self.fjj();
        if l >= ca.len() as u64 {
            return Ok(0);
        }
        let ay = l as usize;
        let ajp = core::cmp::v(k.len(), ca.len() - ay);
        k[..ajp].dg(&ca[ay..ay + ajp]);
        Ok(ajp)
    }
    
    fn write(&self, dnv: u64, ihz: &[u8]) -> B<usize> {
        Err(VfsError::Bz)
    }
    
    fn hm(&self) -> B<Stat> {
        let aw = self.fjj().len() as u64;
        Ok(Stat {
            dd: self.dd,
            kd: FileType::Ea,
            aw,
            ev: 0o444,
            ..Default::default()
        })
    }
}






struct Ale {
    ce: u32,
    kd: PidFileType,
    dd: I,
}

impl Ale {
    fn fjj(&self) -> Vec<u8> {
        match self.kd {
            PidFileType::Status => self.tbo(),
            PidFileType::Aqd => self.tas(),
            PidFileType::Avh => self.tbb(),
            PidFileType::Vz => self.kxz(),
            PidFileType::Aqm => self.tav(),
            PidFileType::Arp => self.tay(),
        }
    }
    
    fn tbo(&self) -> Vec<u8> {
        let mut e = String::new();
        crate::process::ela(self.ce, |ai| {
            let boo = match ai.g {
                crate::process::ProcessState::Cu => "N (created)",
                crate::process::ProcessState::At => "R (running)",
                crate::process::ProcessState::Ai => "R (running)",
                crate::process::ProcessState::Hj => "S (sleeping)",
                crate::process::ProcessState::Bwo => "S (sleeping)",
                crate::process::ProcessState::Af => "T (stopped)",
                crate::process::ProcessState::Vf => "Z (zombie)",
                crate::process::ProcessState::Ez => "X (dead)",
            };
            e.t(&format!("Name:\t{}\n", ai.j));
            e.t(&format!("State:\t{}\n", boo));
            e.t(&format!("Pid:\t{}\n", ai.ce));
            e.t(&format!("PPid:\t{}\n", ai.bfb));
            e.t(&format!("Uid:\t{}\t{}\t{}\t{}\n", ai.pi, ai.ahl, ai.pi, ai.pi));
            e.t(&format!("Gid:\t{}\t{}\t{}\t{}\n", ai.pw, ai.bqj, ai.pw, ai.pw));
            e.t(&format!("FDSize:\t{}\n", ai.buf.len()));
            e.t(&format!("VmSize:\t{} kB\n", 
                (ai.memory.ecv.ao(ai.memory.dez)) / 1024));
            e.t(&format!("Threads:\t{}\n", ai.zf.len() + 1));
            e.t(&format!("SigPnd:\t0000000000000000\n"));
            e.t(&format!("Cpus_allowed:\tff\n"));
        });
        if e.is_empty() {
            e.t("(no such process)\n");
        }
        e.cfq()
    }
    
    fn tas(&self) -> Vec<u8> {
        let mut j = String::new();
        crate::process::ela(self.ce, |ai| {
            j = ai.j.clone();
        });
        format!("{}\n", j).cfq()
    }
    
    fn tbb(&self) -> Vec<u8> {
        let mut e = String::new();
        crate::process::ela(self.ce, |ai| {
            let ef = &ai.memory;
            if ef.kjr > ef.dez {
                e.t(&format!("{:016x}-{:016x} r-xp 00000000 00:00 0  [code]\n", 
                    ef.dez, ef.kjr));
            }
            if ef.njm > ef.bjt {
                e.t(&format!("{:016x}-{:016x} rw-p 00000000 00:00 0  [data]\n", 
                    ef.bjt, ef.njm));
            }
            if ef.ecv > ef.caa {
                e.t(&format!("{:016x}-{:016x} rw-p 00000000 00:00 0  [heap]\n", 
                    ef.caa, ef.ecv));
            }
            if ef.ibm > ef.ibo {
                e.t(&format!("{:016x}-{:016x} rw-p 00000000 00:00 0  [stack]\n", 
                    ef.ibo, ef.ibm));
            }
            
            if let Some(jm) = ai.jm.ink(0).hi(|&r| r != 0) {
                for vma in crate::memory::vma::ufz(jm) {
                    let m = if vma.prot & 1 != 0 { 'r' } else { '-' };
                    let d = if vma.prot & 2 != 0 { 'w' } else { '-' };
                    let b = if vma.prot & 4 != 0 { 'x' } else { '-' };
                    e.t(&format!("{:016x}-{:016x} {}{}{}p 00000000 00:00 0  [mmap]\n",
                        vma.ay, vma.ci, m, d, b));
                }
            }
        });
        if e.is_empty() {
            e.t("(no such process)\n");
        }
        e.cfq()
    }
    
    fn kxz(&self) -> Vec<u8> {
        let mut j = String::new();
        crate::process::ela(self.ce, |ai| {
            j = ai.j.clone();
        });
        format!("{}\0", j).cfq()
    }
    
    fn tav(&self) -> Vec<u8> {
        let mut jv = String::from("/");
        crate::process::ela(self.ce, |ai| {
            jv = ai.jv.clone();
        });
        format!("{}\n", jv).cfq()
    }
    
    fn tay(&self) -> Vec<u8> {
        let mut e = String::new();
        crate::process::ela(self.ce, |ai| {
            for (eh, p) in &ai.env {
                e.t(&format!("{}={}\0", eh, p));
            }
        });
        e.cfq()
    }
}

impl Et for Ale {
    fn read(&self, l: u64, k: &mut [u8]) -> B<usize> {
        let ca = self.fjj();
        if l >= ca.len() as u64 {
            return Ok(0);
        }
        let ay = l as usize;
        let ajp = core::cmp::v(k.len(), ca.len() - ay);
        k[..ajp].dg(&ca[ay..ay + ajp]);
        Ok(ajp)
    }
    
    fn write(&self, dnv: u64, ihz: &[u8]) -> B<usize> {
        Err(VfsError::Bz)
    }
    
    fn hm(&self) -> B<Stat> {
        let aw = self.fjj().len() as u64;
        Ok(Stat {
            dd: self.dd,
            kd: FileType::Ea,
            aw,
            ev: 0o444,
            ..Default::default()
        })
    }
}





struct Bpi {
    ce: u32,
}

impl Ep for Bpi {
    fn cga(&self, j: &str) -> B<I> {
        for (a, ebt) in BDF_.iter().cf() {
            if j == *ebt {
                return Ok(ovn(self.ce, (a + 1) as u64));
            }
        }
        Err(VfsError::N)
    }
    
    fn brx(&self) -> B<Vec<Br>> {
        let mut ch = vec![
            Br { j: String::from("."), dd: jjh(self.ce), kd: FileType::K },
            Br { j: String::from(".."), dd: 1, kd: FileType::K },
        ];
        for (a, ebt) in BDF_.iter().cf() {
            ch.push(Br {
                j: String::from(*ebt),
                dd: ovn(self.ce, (a + 1) as u64),
                kd: FileType::Ea,
            });
        }
        Ok(ch)
    }
    
    fn avp(&self, blu: &str, gxf: FileType) -> B<I> {
        Err(VfsError::Bz)
    }
    
    fn cnm(&self, blu: &str) -> B<()> {
        Err(VfsError::Bz)
    }
    
    fn hm(&self) -> B<Stat> {
        Ok(Stat {
            dd: jjh(self.ce),
            kd: FileType::K,
            ev: 0o555,
            ..Default::default()
        })
    }
}






#[derive(Clone)]
struct Md {
    j: String,
    kd: ProcFileType,
    dd: I,
}


struct Bpj {
    ch: Vec<Md>,
}

impl Ep for Bpj {
    fn cga(&self, j: &str) -> B<I> {
        
        for bt in &self.ch {
            if bt.j == j {
                return Ok(bt.dd);
            }
        }
        
        if let Ok(ce) = j.parse::<u32>() {
            
            let aja = crate::process::aoy().iter().any(|(ai, _, _)| *ai == ce);
            if aja {
                return Ok(jjh(ce));
            }
        }
        Err(VfsError::N)
    }
    
    fn brx(&self) -> B<Vec<Br>> {
        let mut ch = vec![
            Br { j: String::from("."), dd: 1, kd: FileType::K },
            Br { j: String::from(".."), dd: 1, kd: FileType::K },
        ];
        
        
        for bt in &self.ch {
            ch.push(Br {
                j: bt.j.clone(),
                dd: bt.dd,
                kd: FileType::Ea,
            });
        }
        
        
        for (ce, blu, gxl) in crate::process::aoy() {
            ch.push(Br {
                j: format!("{}", ce),
                dd: jjh(ce),
                kd: FileType::K,
            });
        }
        
        Ok(ch)
    }
    
    fn avp(&self, blu: &str, gxf: FileType) -> B<I> {
        Err(VfsError::Bz)
    }
    
    fn cnm(&self, blu: &str) -> B<()> {
        Err(VfsError::Bz)
    }
    
    fn hm(&self) -> B<Stat> {
        Ok(Stat {
            dd: 1,
            kd: FileType::K,
            ev: 0o555,
            ..Default::default()
        })
    }
}


pub struct ProcFs {
    ch: Vec<Md>,
}

impl ProcFs {
    pub fn new() -> B<Self> {
        let ch = vec![
            Md { j: String::from("cpuinfo"), kd: ProcFileType::Aat, dd: 2 },
            Md { j: String::from("meminfo"), kd: ProcFileType::Bmd, dd: 3 },
            Md { j: String::from("uptime"), kd: ProcFileType::Afh, dd: 4 },
            Md { j: String::from("version"), kd: ProcFileType::Re, dd: 5 },
            Md { j: String::from("mounts"), kd: ProcFileType::Bmq, dd: 6 },
            Md { j: String::from("cmdline"), kd: ProcFileType::Vz, dd: 7 },
            Md { j: String::from("stat"), kd: ProcFileType::Stat, dd: 8 },
        ];
        
        Ok(Self { ch })
    }
    
    fn nug(&self, dd: I) -> Option<&Md> {
        self.ch.iter().du(|aa| aa.dd == dd)
    }
}

impl Cc for ProcFs {
    fn j(&self) -> &str {
        "proc"
    }
    
    fn cbm(&self) -> I {
        1
    }
    
    fn era(&self, dd: I) -> B<Arc<dyn Et>> {
        
        if let Some(bt) = self.nug(dd) {
            return Ok(Arc::new(Alh {
                kd: bt.kd,
                dd: bt.dd,
            }));
        }
        
        if let Some((ce, w)) = ogo(dd) {
            let agm = match w {
                1 => PidFileType::Status,
                2 => PidFileType::Aqd,
                3 => PidFileType::Avh,
                4 => PidFileType::Vz,
                5 => PidFileType::Aqm,
                6 => PidFileType::Arp,
                _ => return Err(VfsError::N),
            };
            return Ok(Arc::new(Ale { ce, kd: agm, dd }));
        }
        Err(VfsError::N)
    }
    
    fn dhl(&self, dd: I) -> B<Arc<dyn Ep>> {
        if dd == 1 {
            return Ok(Arc::new(Bpj {
                ch: self.ch.clone(),
            }));
        }
        
        if let Some(ce) = ogn(dd) {
            return Ok(Arc::new(Bpi { ce }));
        }
        Err(VfsError::Lz)
    }
    
    fn hm(&self, dd: I) -> B<Stat> {
        if dd == 1 {
            return Ok(Stat {
                dd: 1,
                kd: FileType::K,
                ev: 0o555,
                ..Default::default()
            });
        }
        
        if let Some(bt) = self.nug(dd) {
            let byy = Alh { kd: bt.kd, dd: bt.dd }
                .fjj().len() as u64;
            return Ok(Stat {
                dd: bt.dd,
                kd: FileType::Ea,
                aw: byy,
                ev: 0o444,
                ..Default::default()
            });
        }
        
        if ogn(dd).is_some() {
            return Ok(Stat {
                dd,
                kd: FileType::K,
                ev: 0o555,
                ..Default::default()
            });
        }
        
        if let Some((ce, w)) = ogo(dd) {
            let agm = match w {
                1 => PidFileType::Status,
                2 => PidFileType::Aqd,
                3 => PidFileType::Avh,
                4 => PidFileType::Vz,
                5 => PidFileType::Aqm,
                6 => PidFileType::Arp,
                _ => return Err(VfsError::N),
            };
            let aw = Ale { ce, kd: agm, dd }.fjj().len() as u64;
            return Ok(Stat {
                dd,
                kd: FileType::Ea,
                aw,
                ev: 0o444,
                ..Default::default()
            });
        }
        Err(VfsError::N)
    }
}
