












use alloc::string::String;
use alloc::vec::Vec;
use alloc::format;
use core::sync::atomic::{AtomicBool, AtomicU64, Ordering};






static AJQ_: AtomicBool = AtomicBool::new(false);


static CTP_: AtomicU64 = AtomicU64::new(0);


pub fn aiy() {
    AJQ_.store(true, Ordering::SeqCst);
    crate::serial_println!("[VMI] Introspection engine enabled");
    crate::lab_mode::trace_bus::epu(0, "VMI engine ENABLED");
}


pub fn cwz() {
    AJQ_.store(false, Ordering::SeqCst);
    crate::serial_println!("[VMI] Introspection engine disabled");
    crate::lab_mode::trace_bus::epu(0, "VMI engine DISABLED");
}


pub fn zu() -> bool {
    AJQ_.load(Ordering::Relaxed)
}






pub fn jlj(fk: u64, pe: u64, len: usize) -> Option<Vec<u8>> {
    
    if let Some(f) = vrt(fk, pe, len) {
        return Some(f);
    }
    vru(fk, pe, len)
}


fn vrt(fk: u64, pe: u64, len: usize) -> Option<Vec<u8>> {
    super::svm_vm::coa(fk, |vm| {
        vm.duy(pe, len).map(|e| e.ip())
    }).yqy()
}


fn vru(qeb: u64, qbz: u64, jxx: usize) -> Option<Vec<u8>> {
    
    
    None
}


pub fn jlk(fk: u64, pe: u64) -> Option<u64> {
    let f = jlj(fk, pe, 8)?;
    if f.len() < 8 { return None; }
    Some(u64::dj([
        f[0], f[1], f[2], f[3],
        f[4], f[5], f[6], f[7],
    ]))
}


pub fn zhu(fk: u64, pe: u64) -> Option<u32> {
    let f = jlj(fk, pe, 4)?;
    if f.len() < 4 { return None; }
    Some(u32::dj([f[0], f[1], f[2], f[3]]))
}


pub fn zht(fk: u64, pe: u64, cat: usize) -> Option<String> {
    let am = if cat > 256 { 256 } else { cat };
    let f = jlj(fk, pe, am)?;
    let ci = f.iter().qf(|&o| o == 0).unwrap_or(f.len());
    String::jg(f[..ci].ip()).bq()
}









pub fn tif(fk: u64, bnd: u64, uy: u64) -> Option<u64> {
    
    let vjo = bnd & 0x000F_FFFF_FFFF_F000;
    
    let wd = ((uy >> 39) & 0x1FF) as u64;
    let ru = ((uy >> 30) & 0x1FF) as u64;
    let rn   = ((uy >> 21) & 0x1FF) as u64;
    let yf   = ((uy >> 12) & 0x1FF) as u64;
    let l   = uy & 0xFFF;
    
    
    let owg = jlk(fk, vjo + wd * 8)?;
    if owg & 1 == 0 { return None; } 
    
    
    let vgi = owg & 0x000F_FFFF_FFFF_F000;
    let jiz = jlk(fk, vgi + ru * 8)?;
    if jiz & 1 == 0 { return None; }
    
    
    if jiz & (1 << 7) != 0 {
        let ht = (jiz & 0x000F_FFFF_C000_0000) | (uy & 0x3FFF_FFFF);
        return Some(ht);
    }
    
    
    let vgc = jiz & 0x000F_FFFF_FFFF_F000;
    let gpd = jlk(fk, vgc + rn * 8)?;
    if gpd & 1 == 0 { return None; }
    
    
    if gpd & (1 << 7) != 0 {
        let ht = (gpd & 0x000F_FFFF_FFE0_0000) | (uy & 0x1F_FFFF);
        return Some(ht);
    }
    
    
    let frn = gpd & 0x000F_FFFF_FFFF_F000;
    let oyh = jlk(fk, frn + yf * 8)?;
    if oyh & 1 == 0 { return None; }
    
    let ht = (oyh & 0x000F_FFFF_FFFF_F000) | l;
    Some(ht)
}


pub fn exi(fk: u64, bnd: u64, uy: u64, len: usize) -> Option<Vec<u8>> {
    
    let mut result = Vec::fc(len);
    let mut ia = len;
    let mut kmq = uy;
    
    while ia > 0 {
        let pe = tif(fk, bnd, kmq)?;
        let huc = (kmq & 0xFFF) as usize;
        let jj = core::cmp::v(ia, 4096 - huc);
        
        let f = jlj(fk, pe, jj)?;
        result.bk(&f);
        
        ia -= jj;
        kmq += jj as u64;
    }
    
    Some(result)
}






#[derive(Debug, Clone, Default)]
pub struct RegisterSnapshot {
    
    pub rax: u64,
    pub rbx: u64,
    pub rcx: u64,
    pub rdx: u64,
    pub rsi: u64,
    pub rdi: u64,
    pub rbp: u64,
    pub rsp: u64,
    pub r8:  u64,
    pub r9:  u64,
    pub r10: u64,
    pub r11: u64,
    pub r12: u64,
    pub r13: u64,
    pub r14: u64,
    pub r15: u64,
    
    pub pc: u64,
    pub rflags: u64,
    
    pub akb: u64,
    pub jm: u64,
    pub cr4: u64,
    
    pub aap: u16,
    pub bjw: u16,
    pub rv: u16,
    pub cqf: u16,
}


pub fn wqi(fk: u64) -> Option<RegisterSnapshot> {
    super::svm_vm::coa(fk, |vm| {
        let mut xj = RegisterSnapshot::default();
        
        
        let regs = &vm.ej;
        xj.rax = regs.rax;
        xj.rbx = regs.rbx;
        xj.rcx = regs.rcx;
        xj.rdx = regs.rdx;
        xj.rsi = regs.rsi;
        xj.rdi = regs.rdi;
        xj.rbp = regs.rbp;
        xj.r8  = regs.r8;
        xj.r9  = regs.r9;
        xj.r10 = regs.r10;
        xj.r11 = regs.r11;
        xj.r12 = regs.r12;
        xj.r13 = regs.r13;
        xj.r14 = regs.r14;
        xj.r15 = regs.r15;
        
        
        if let Some(ref vmcb) = vm.vmcb {
            use super::svm::vmcb::state_offsets;
            xj.pc = vmcb.xs(state_offsets::Aw);
            xj.rsp = vmcb.xs(state_offsets::Hc);
            xj.rflags = vmcb.xs(state_offsets::Kv);
            xj.akb = vmcb.xs(state_offsets::Vu);
            xj.jm = vmcb.xs(state_offsets::Vv);
            xj.cr4 = vmcb.xs(state_offsets::Vw);
            xj.aap = vmcb.xs(state_offsets::JU_) as u16;
            xj.bjw = vmcb.xs(state_offsets::MV_) as u16;
            xj.rv = vmcb.xs(state_offsets::XH_) as u16;
            xj.cqf = vmcb.xs(state_offsets::SZ_) as u16;
        }
        
        xj
    })
}


pub fn wqh(fk: u64) -> Option<RegisterSnapshot> {
    CTP_.fetch_add(1, Ordering::Relaxed);
    
    
    if let Some(xj) = wqi(fk) {
        crate::lab_mode::trace_bus::nps(
            fk,
            xj.pc, xj.rsp, xj.rax, xj.rbx, xj.rcx, xj.rdx,
        );
        return Some(xj);
    }
    
    
    None
}






#[derive(Debug, Clone)]
pub struct Ati {
    
    pub ce: u32,
    
    pub gcy: String,
    
    pub g: u8,
    
    pub bfb: u32,
    
    pub xbb: u64,
    
    pub pyo: u64,
}



#[derive(Debug, Clone, Copy)]
pub struct LinuxOffsets {
    
    pub jsq: usize,
    
    pub ce: usize,
    
    pub gcy: usize,
    
    pub g: usize,
    
    pub tu: usize,
    
    pub lmd: usize,
    
    pub lme: usize,
    
    pub izy: u64,
}

impl LinuxOffsets {
    
    pub fn ufj() -> Self {
        LinuxOffsets {
            jsq: 0x498,    
            ce: 0x560,           
            gcy: 0x6F0,          
            g: 0x00,          
            tu: 0x568,        
            lmd: 0x478,            
            lme: 0x80,    
            izy: 0,    
        }
    }
    
    
    pub fn ufi() -> Self {
        LinuxOffsets {
            jsq: 0x3F0,
            ce: 0x4C8,
            gcy: 0x670,
            g: 0x00,
            tu: 0x4D0,
            lmd: 0x458,
            lme: 0x80,
            izy: 0,
        }
    }
}





pub fn nqr(
    fk: u64,
    bnd: u64,
    bkr: &LinuxOffsets,
) -> Vec<Ati> {
    let mut ye = Vec::new();
    let ulv = 512; 
    
    if bkr.izy == 0 {
        return ye;
    }
    
    let oeo = bkr.izy;
    let mut cv = oeo;
    
    for _ in 0..ulv {
        
        let ce = match exi(fk, bnd, cv + bkr.ce as u64, 4) {
            Some(f) if f.len() >= 4 => {
                u32::dj([f[0], f[1], f[2], f[3]])
            }
            _ => break,
        };
        
        
        let gcy = exi(fk, bnd, cv + bkr.gcy as u64, 16)
            .and_then(|f| {
                let ci = f.iter().qf(|&o| o == 0).unwrap_or(f.len());
                String::jg(f[..ci].ip()).bq()
            })
            .unwrap_or_else(|| String::from("?"));
        
        
        let g = exi(fk, bnd, cv + bkr.g as u64, 1)
            .map(|bc| bc[0])
            .unwrap_or(0);
        
        
        let bfb = exi(fk, bnd, cv + bkr.tu as u64, 8)
            .and_then(|f| {
                if f.len() < 8 { return None; }
                let lsd = u64::dj([
                    f[0], f[1], f[2], f[3],
                    f[4], f[5], f[6], f[7],
                ]);
                
                exi(fk, bnd, lsd + bkr.ce as u64, 4)
                    .map(|bc| u32::dj([bc[0], bc[1], bc[2], bc[3]]))
            })
            .unwrap_or(0);
        
        
        let pyo = exi(fk, bnd, cv + bkr.lmd as u64, 8)
            .and_then(|f| {
                if f.len() < 8 { return None; }
                let onp = u64::dj([
                    f[0], f[1], f[2], f[3],
                    f[4], f[5], f[6], f[7],
                ]);
                if onp == 0 { return Some(0u64); } 
                exi(fk, bnd, onp + bkr.lme as u64, 8)
                    .map(|bc| u64::dj([
                        bc[0], bc[1], bc[2], bc[3], bc[4], bc[5], bc[6], bc[7],
                    ]))
            })
            .unwrap_or(0);
        
        ye.push(Ati {
            ce,
            gcy,
            g,
            bfb,
            xbb: cv,
            pyo,
        });
        
        
        let uuk = match exi(
            fk, bnd,
            cv + bkr.jsq as u64, 8
        ) {
            Some(f) if f.len() >= 8 => {
                u64::dj([
                    f[0], f[1], f[2], f[3],
                    f[4], f[5], f[6], f[7],
                ])
            }
            _ => break,
        };
        
        
        cv = uuk.nj(bkr.jsq as u64);
        
        
        if cv == oeo {
            break;
        }
    }
    
    ye
}






#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MemoryRegionType {
    
    Jw,
    
    Nn,
    
    Bre,
    
    Afg,
    
    Nw,
    
    Bbk,
}


#[derive(Debug, Clone)]
pub struct Ne {
    pub ar: u64,
    pub aw: u64,
    pub bwo: MemoryRegionType,
    pub cu: &'static str,
}


pub fn kfk(afc: usize) -> Vec<Ne> {
    let jfr = (afc * 1024 * 1024) as u64;
    let mut afx = Vec::new();
    
    
    
    afx.push(Ne {
        ar: 0,
        aw: 0xA_0000,
        bwo: MemoryRegionType::Jw,
        cu: "Conventional",
    });
    
    
    afx.push(Ne {
        ar: 0xA_0000,
        aw: 0x6_0000,
        bwo: MemoryRegionType::Nn,
        cu: "VGA+ROM",
    });
    
    
    let spy = if jfr > 0x1_0000_0000 {
        
        0xC000_0000u64 
    } else {
        core::cmp::v(jfr, 0xC000_0000)
    };
    
    afx.push(Ne {
        ar: 0x10_0000,
        aw: spy - 0x10_0000,
        bwo: MemoryRegionType::Jw,
        cu: "Extended",
    });
    
    
    afx.push(Ne {
        ar: 0xC000_0000,
        aw: 0x4000_0000,
        bwo: MemoryRegionType::Nn,
        cu: "PCI MMIO",
    });
    
    
    if jfr > 0x1_0000_0000 {
        let qel = jfr - 0xC000_0000; 
        afx.push(Ne {
            ar: 0x1_0000_0000,
            aw: qel,
            bwo: MemoryRegionType::Jw,
            cu: "High Memory",
        });
    }
    
    
    afx.push(Ne {
        ar: 0xFEC0_0000,
        aw: 0x1000,
        bwo: MemoryRegionType::Nn,
        cu: "IO-APIC",
    });
    
    afx.push(Ne {
        ar: 0xFEE0_0000,
        aw: 0x1000,
        bwo: MemoryRegionType::Nn,
        cu: "Local APIC",
    });
    
    afx
}






#[derive(Debug, Clone)]
pub struct Btw {
    
    pub hi: Vec<u64>,
    
    pub gh: bool,
    
    pub bjm: Vec<Apt>,
    
    pub hqy: usize,
}


#[derive(Debug, Clone)]
pub struct Apt {
    pub fk: u64,
    pub fvy: u64,
    pub qki: u64,
    pub aai: u64,
    pub agf: u64,
    pub pc: u64,
    pub aea: u64,
}

impl Btw {
    pub fn new() -> Self {
        Btw {
            hi: Vec::new(),
            gh: false,
            bjm: Vec::new(),
            hqy: 1024,
        }
    }
    
    
    pub fn record(&mut self, fk: u64, nr: u64, bfv: u64, km: u64, oe: u64, pc: u64) {
        if !self.gh { return; }
        
        
        if !self.hi.is_empty() && !self.hi.contains(&nr) {
            return;
        }
        
        if self.bjm.len() >= self.hqy {
            self.bjm.remove(0); 
        }
        
        self.bjm.push(Apt {
            fk,
            fvy: nr,
            qki: bfv,
            aai: km,
            agf: oe,
            pc,
            aea: crate::time::lc(),
        });
        
        
        crate::lab_mode::trace_bus::ept(
            fk,
            "SYSCALL",
            pc,
            &format!("nr={} a0=0x{:X} a1=0x{:X}", nr, bfv, km),
        );
    }
    
    
    pub fn fsj(&self, az: usize) -> &[Apt] {
        let ay = self.bjm.len().ao(az);
        &self.bjm[ay..]
    }
    
    
    pub fn clear(&mut self) {
        self.bjm.clear();
    }
}






#[derive(Debug, Clone)]
pub struct Bvy {
    pub fk: u64,
    pub jvx: String,
    pub regs: Option<RegisterSnapshot>,
    pub ye: Vec<Ati>,
    pub memory_map: Vec<Ne>,
    pub afc: usize,
    pub g: &'static str,
}


pub fn zqy(fk: u64) -> Option<Bvy> {
    if !zu() { return None; }
    
    
    let bfr = super::svm_vm::hqc();
    let (j, boo, afc) = {
        let aig = bfr.iter().du(|(ad, _, _)| *ad == fk)?;
        let g = match aig.2 {
            super::svm_vm::SvmVmState::Cu => "created",
            super::svm_vm::SvmVmState::Ai => "running",
            super::svm_vm::SvmVmState::Af => "stopped",
            super::svm_vm::SvmVmState::Cl => "paused",
            _ => "unknown",
        };
        
        (aig.1.clone(), g, 0usize) 
    };
    
    
    let regs = wqh(fk);
    
    
    let ye = if let Some(ref m) = regs {
        if m.jm != 0 {
            
            let uxk = LinuxOffsets::ufj();
            let jke = nqr(fk, m.jm, &uxk);
            if jke.is_empty() {
                let uxj = LinuxOffsets::ufi();
                nqr(fk, m.jm, &uxj)
            } else {
                jke
            }
        } else {
            Vec::new()
        }
    } else {
        Vec::new()
    };
    
    
    let memory_map = kfk(if afc > 0 { afc } else { 64 });
    
    Some(Bvy {
        fk,
        jvx: j,
        regs,
        ye,
        memory_map,
        afc,
        g: boo,
    })
}


pub fn ojm() -> Vec<(u64, String, &'static str)> {
    let mut result = Vec::new();
    
    
    for (ad, j, g) in super::svm_vm::hqc() {
        let boo = match g {
            super::svm_vm::SvmVmState::Cu => "created",
            super::svm_vm::SvmVmState::Ai => "running",
            super::svm_vm::SvmVmState::Af => "stopped",
            super::svm_vm::SvmVmState::Cl => "paused",
            _ => "unknown",
        };
        result.push((ad, j, boo));
    }
    
    result
}
