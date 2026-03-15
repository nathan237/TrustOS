

















use alloc::string::String;
use alloc::vec::Vec;
use alloc::boxed::Box;
use alloc::format;
use core::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use spin::Mutex;

use super::{HypervisorError, Result, CpuVendor, avo};
use super::linux_subsystem::{Bli, Byq, Arj, e820_type};


pub mod boot_proto {
    
    pub const DO_: u64 = 0x10000;
    
    
    pub const HQ_: u64 = 0x20000;
    
    
    pub const BMP_: usize = 2048;
    
    
    pub const FJ_: u64 = 0x100000;
    
    
    pub const AWY_: u64 = 0x8000;
    
    
    pub const IB_: u64 = 0x1000;
    
    
    pub const CBQ_: u64 = 0x1000000;  
    
    
    pub const EFL_: usize = 0x1F1;
    
    
    pub const BYZ_: u32 = 0x53726448;
    
    
    pub const DUF_: u16 = 0x0200;
    
    
    pub const UT_: u8 = 0x01;
    pub const ZU_: u8 = 0x80;
    
    
    pub const CER_: u8 = 0xFF;  
    
    
    pub const AQX_: u32 = 1;
    pub const AQY_: u32 = 2;
    pub const DKZ_: u32 = 3;
}


#[derive(Debug, Clone)]
pub struct LinuxKernelInfo {
    
    pub ewy: u16,
    
    pub boi: u8,
    
    pub eet: u8,
    
    pub ffh: u32,
    
    pub czd: u16,
    
    pub gjy: u32,
    
    pub hpj: u32,
    
    pub pbn: bool,
    
    pub kjm: u32,
    
    pub gpq: u64,
    
    pub gjx: u32,
}

impl LinuxKernelInfo {
    
    pub fn sxr(uz: &[u8]) -> Option<Self> {
        if uz.len() < 0x250 {
            return None;
        }
        
        
        let sj = u32::dj([
            uz[0x202],
            uz[0x203],
            uz[0x204],
            uz[0x205],
        ]);
        
        if sj != boot_proto::BYZ_ {
            return None;
        }
        
        let ewy = u16::dj([uz[0x206], uz[0x207]]);
        let boi = uz[0x1F1];
        let eet = uz[0x211];
        let ffh = u32::dj([
            uz[0x214],
            uz[0x215],
            uz[0x216],
            uz[0x217],
        ]);
        let czd = u16::dj([uz[0x20E], uz[0x20F]]);
        
        
        let gjy = if ewy >= 0x0200 {
            u32::dj([
                uz[0x22C],
                uz[0x22D],
                uz[0x22E],
                uz[0x22F],
            ])
        } else {
            0x37FFFFFF
        };
        
        
        let (hpj, pbn) = if ewy >= 0x0205 {
            let align = u32::dj([
                uz[0x230],
                uz[0x231],
                uz[0x232],
                uz[0x233],
            ]);
            let dbb = uz[0x234] != 0;
            (align, dbb)
        } else {
            (0x100000, false)
        };
        
        
        let kjm = if ewy >= 0x0206 {
            u32::dj([
                uz[0x238],
                uz[0x239],
                uz[0x23A],
                uz[0x23B],
            ])
        } else {
            255
        };
        
        
        let (gpq, gjx) = if ewy >= 0x020A {
            let bwa = u64::dj([
                uz[0x258],
                uz[0x259],
                uz[0x25A],
                uz[0x25B],
                uz[0x25C],
                uz[0x25D],
                uz[0x25E],
                uz[0x25F],
            ]);
            let init = u32::dj([
                uz[0x260],
                uz[0x261],
                uz[0x262],
                uz[0x263],
            ]);
            (bwa, init)
        } else {
            (0x100000, 0)
        };
        
        Some(Self {
            ewy,
            boi,
            eet,
            ffh,
            czd,
            gjy,
            hpj,
            pbn,
            kjm,
            gpq,
            gjx,
        })
    }
    
    
    pub fn tfb<'a>(&self, uz: &'a [u8]) -> Option<&'a str> {
        if self.czd == 0 {
            return None;
        }
        
        let l = self.czd as usize + 0x200;
        if l >= uz.len() {
            return None;
        }
        
        
        let ci = uz[l..].iter()
            .qf(|&o| o == 0)
            .unwrap_or(64);
        
        core::str::jg(&uz[l..l + ci]).bq()
    }
}


#[derive(Debug, Clone)]
pub struct Pw {
    
    pub afc: usize,
    
    pub wx: String,
    
    pub jvj: u32,
    
    pub gsc: bool,
    
    pub virtio_console: bool,
}

impl Default for Pw {
    fn default() -> Self {
        Self {
            afc: 64,
            wx: String::from("console=ttyS0 earlyprintk=serial quiet"),
            jvj: 1,
            gsc: true,
            virtio_console: true,
        }
    }
}


pub struct LinuxVm {
    
    ad: u64,
    
    config: Pw,
    
    bke: Option<LinuxKernelInfo>,
    
    aqk: AtomicBool,
    
    fe: Vec<u8>,
    
    dzs: Mutex<Vec<u8>>,
}

impl LinuxVm {
    
    pub fn new(config: Pw) -> Result<Self> {
        static AFX_: AtomicU64 = AtomicU64::new(0x10000);
        let ad = AFX_.fetch_add(1, Ordering::SeqCst);
        
        
        let apy = config.afc * 1024 * 1024;
        let fe = alloc::vec![0u8; apy];
        
        crate::serial_println!("[LINUX-VM {}] Created with {} MB RAM", ad, config.afc);
        
        Ok(Self {
            ad,
            config,
            bke: None,
            aqk: AtomicBool::new(false),
            fe,
            dzs: Mutex::new(Vec::new()),
        })
    }
    
    
    pub fn uhc(&mut self, uz: &[u8]) -> Result<()> {
        
        let bke = LinuxKernelInfo::sxr(uz)
            .ok_or(HypervisorError::Cgc)?;
        
        crate::serial_println!("[LINUX-VM {}] Kernel: protocol v{}.{}, setup_sects={}", 
            self.ad,
            bke.ewy >> 8,
            bke.ewy & 0xFF,
            bke.boi);
        
        if let Some(dk) = bke.tfb(uz) {
            crate::serial_println!("[LINUX-VM {}] Kernel version: {}", self.ad, dk);
        }
        
        
        let boi = if bke.boi == 0 { 4 } else { bke.boi };
        let pal = (boi as usize + 1) * 512;
        
        
        let owf = pal;
        let jjl = uz.len() - owf;
        
        crate::serial_println!("[LINUX-VM {}] Real mode: {} bytes, Protected mode: {} bytes", 
            self.ad, pal, jjl);
        
        
        let dix = boot_proto::FJ_ as usize;
        if dix + jjl > self.fe.len() {
            return Err(HypervisorError::Ns);
        }
        
        self.fe[dix..dix + jjl]
            .dg(&uz[owf..]);
        
        crate::serial_println!("[LINUX-VM {}] Loaded kernel at 0x{:X} ({} KB)", 
            self.ad, dix, jjl / 1024);
        
        self.bke = Some(bke);
        
        Ok(())
    }
    
    
    pub fn uha(&mut self, buz: &[u8]) -> Result<u64> {
        let bke = self.bke.as_ref()
            .ok_or(HypervisorError::Acg)?;
        
        
        
        let fns = bke.gjy as u64;
        let mut dix = boot_proto::CBQ_;
        
        
        if dix + buz.len() as u64 > fns {
            
            dix = fns - buz.len() as u64;
            dix &= !0xFFF; 
        }
        
        let l = dix as usize;
        if l + buz.len() > self.fe.len() {
            return Err(HypervisorError::Ns);
        }
        
        self.fe[l..l + buz.len()].dg(buz);
        
        crate::serial_println!("[LINUX-VM {}] Loaded initramfs at 0x{:X} ({} KB)", 
            self.ad, dix, buz.len() / 1024);
        
        Ok(dix)
    }
    
    
    pub fn mez(&mut self, lem: u64, jaa: u32) -> Result<()> {
        let bke = self.bke.as_ref()
            .ok_or(HypervisorError::Acg)?;
        
        
        let kjl = boot_proto::HQ_ as usize;
        let dzn = self.config.wx.as_bytes();
        let ffd = dzn.len().v(boot_proto::BMP_ - 1);
        
        self.fe[kjl..kjl + ffd]
            .dg(&dzn[..ffd]);
        self.fe[kjl + ffd] = 0; 
        
        crate::serial_println!("[LINUX-VM {}] Command line: {}", self.ad, self.config.wx);
        
        
        let avg = boot_proto::DO_ as usize;
        
        
        for a in 0..4096 {
            self.fe[avg + a] = 0;
        }
        
        
        let tnu = avg + 0x1F1;
        
        
        self.fe[tnu] = bke.boi;
        
        
        self.fe[avg + 0x210] = boot_proto::CER_;
        
        
        let eet = boot_proto::UT_ | boot_proto::ZU_;
        self.fe[avg + 0x211] = eet;
        
        
        let ecv: u16 = 0xFE00;
        self.fe[avg + 0x224] = (ecv & 0xFF) as u8;
        self.fe[avg + 0x225] = (ecv >> 8) as u8;
        
        
        let rkx = boot_proto::HQ_ as u32;
        let dzn = rkx.ho();
        self.fe[avg + 0x228..avg + 0x22C]
            .dg(&dzn);
        
        
        let tug = (lem as u32).ho();
        self.fe[avg + 0x218..avg + 0x21C]
            .dg(&tug);
        
        
        let tum = jaa.ho();
        self.fe[avg + 0x21C..avg + 0x220]
            .dg(&tum);
        
        
        self.mfa(avg)?;
        
        crate::serial_println!("[LINUX-VM {}] Boot params at 0x{:X}", 
            self.ad, boot_proto::DO_);
        
        Ok(())
    }
    
    
    fn mfa(&mut self, avg: usize) -> Result<()> {
        
        let isg = avg + 0x2D0;
        let mut ame: u8 = 0;
        
        
        self.dxs(isg, 0, 0, 0x9FC00, boot_proto::AQX_);
        ame += 1;
        
        
        self.dxs(isg, 1, 0x9FC00, 0x400, boot_proto::AQY_);
        ame += 1;
        
        
        self.dxs(isg, 2, 0xA0000, 0x60000, boot_proto::AQY_);
        ame += 1;
        
        
        let okt = 0x100000u64;
        let jeo = (self.fe.len() as u64).ao(okt);
        self.dxs(isg, 3, okt, jeo, boot_proto::AQX_);
        ame += 1;
        
        
        self.fe[avg + 0x1E8] = ame;
        
        crate::serial_println!("[LINUX-VM {}] E820 map: {} entries, {} MB usable", 
            self.ad, ame, jeo / (1024 * 1024));
        
        Ok(())
    }
    
    
    fn dxs(&mut self, qnp: usize, index: usize, 
                        ag: u64, aw: u64, avt: u32) {
        let bql = qnp + index * 20;  
        
        
        self.fe[bql..bql + 8].dg(&ag.ho());
        
        
        self.fe[bql + 8..bql + 16].dg(&aw.ho());
        
        
        self.fe[bql + 16..bql + 20].dg(&avt.ho());
    }
    
    
    fn wkv(&mut self) -> Result<u64> {
        let eqy = boot_proto::IB_ as usize;
        
        
        
        
        
        
        
        self.fe[eqy..eqy + 8].dg(&[0u8; 8]);
        
        
        let rli: u64 = 0x00CF9A000000FFFF;
        self.fe[eqy + 8..eqy + 16].dg(&rli.ho());
        
        
        let rtq: u64 = 0x00CF92000000FFFF;
        self.fe[eqy + 16..eqy + 24].dg(&rtq.ho());
        
        
        Ok(boot_proto::IB_)
    }
    
    
    pub fn boot(&mut self, uz: &[u8], buz: &[u8]) -> Result<()> {
        
        self.uhc(uz)?;
        
        
        let lem = self.uha(buz)?;
        
        
        self.mez(lem, buz.len() as u32)?;
        
        
        let eqy = self.wkv()?;
        
        
        let bke = self.bke.as_ref()
            .ok_or(HypervisorError::Acg)?;
        
        let mi = if bke.ffh != 0 {
            bke.ffh as u64
        } else {
            boot_proto::FJ_
        };
        
        crate::serial_println!("[LINUX-VM {}] Entry point: 0x{:X}", self.ad, mi);
        crate::serial_println!("[LINUX-VM {}] GDT at: 0x{:X}", self.ad, eqy);
        crate::serial_println!("[LINUX-VM {}] Boot params: 0x{:X}", self.ad, boot_proto::DO_);
        
        
        match avo() {
            CpuVendor::Ef => {
                crate::serial_println!("[LINUX-VM {}] Using Intel VMX...", self.ad);
                self.qrh(mi)?;
            }
            CpuVendor::Ct => {
                crate::serial_println!("[LINUX-VM {}] Using AMD SVM...", self.ad);
                self.qrg(mi)?;
            }
            CpuVendor::F => {
                crate::serial_println!("[LINUX-VM {}] No hardware virtualization available", self.ad);
                crate::serial_println!("[LINUX-VM {}] Running in simulated mode", self.ad);
                return Ok(());
            }
        }
        
        Ok(())
    }
    
    
    fn qrh(&mut self, mi: u64) -> Result<()> {
        use super::vm::VirtualMachine;
        
        crate::serial_println!("[LINUX-VM {}] VMX boot: creating Intel VT-x VM...", self.ad);
        
        
        let mut vm = VirtualMachine::new(self.ad + 100, "linux-vmx-guest", self.config.afc)?;
        
        
        vm.cfp()?;
        
        crate::serial_println!("[LINUX-VM {}] VMX VM initialized, loading {} MB...", 
            self.ad, self.fe.len() / (1024 * 1024));
        
        
        vm.diy(&self.fe, 0)?;
        
        crate::serial_println!("[LINUX-VM {}] Starting Linux kernel via VMX...", self.ad);
        crate::serial_println!("[LINUX-VM {}] Entry: 0x{:X}, Boot params: 0x{:X}", 
            self.ad, mi, boot_proto::DO_);
        
        self.aqk.store(true, Ordering::SeqCst);
        
        
        
        
        
        match vm.ay(mi, boot_proto::AWY_) {
            Ok(()) => {
                crate::serial_println!("[LINUX-VM {}] VMX execution completed", self.ad);
            }
            Err(aa) => {
                crate::serial_println!("[LINUX-VM {}] VMX execution failed: {:?}", self.ad, aa);
                crate::serial_println!("[LINUX-VM {}] Note: VMX requires Intel CPU with VT-x. QEMU TCG does not support nested VMX.", self.ad);
                return Err(aa);
            }
        }
        
        self.aqk.store(false, Ordering::SeqCst);
        
        Ok(())
    }
    
    
    fn qrg(&mut self, mi: u64) -> Result<()> {
        use super::svm_vm;
        
        
        let fk = svm_vm::dpg("linux-guest", self.config.afc)?;
        
        crate::serial_println!("[LINUX-VM {}] SVM VM #{} created, loading {} MB...", 
            self.ad, fk, self.fe.len() / (1024 * 1024));
        
        
        let qra = svm_vm::coa(fk, |vm| -> Result<()> {
            
            vm.cfp()?;
            
            
            vm.diy(&self.fe, 0)?;
            
            
            vm.pjq(
                mi, 
                boot_proto::AWY_,
                boot_proto::DO_
            )?;
            
            crate::serial_println!("[LINUX-VM {}] Starting Linux kernel execution...", self.ad);
            crate::serial_println!("[LINUX-VM {}] Entry: 0x{:X}, Boot params: 0x{:X}", 
                self.ad, mi, boot_proto::DO_);
            
            
            vm.ay()
        });
        
        match qra {
            Some(Ok(())) => {
                crate::serial_println!("[LINUX-VM {}] VM execution completed", self.ad);
            }
            Some(Err(aa)) => {
                crate::serial_println!("[LINUX-VM {}] VM execution failed: {:?}", self.ad, aa);
                return Err(aa);
            }
            None => {
                crate::serial_println!("[LINUX-VM {}] Could not find VM #{}", self.ad, fk);
                return Err(HypervisorError::Mo);
            }
        }
        
        self.aqk.store(false, Ordering::SeqCst);
        
        Ok(())
    }
    
    
    pub fn dsi(&self) -> bool {
        self.aqk.load(Ordering::SeqCst)
    }
    
    
    pub fn iwo(&self) -> Vec<u8> {
        self.dzs.lock().clone()
    }
    
    
    pub fn ad(&self) -> u64 {
        self.ad
    }
}


static AEN_: Mutex<Option<LinuxVm>> = Mutex::new(None);


pub fn ima(uz: &[u8], buz: &[u8], wx: &str) -> Result<u64> {
    let config = Pw {
        afc: 128,
        wx: String::from(wx),
        ..Default::default()
    };
    
    let mut vm = LinuxVm::new(config)?;
    let ad = vm.ad();
    
    vm.boot(uz, buz)?;
    
    *AEN_.lock() = Some(vm);
    
    Ok(ad)
}


pub fn dsi() -> bool {
    AEN_.lock().as_ref().map(|vm| vm.dsi()).unwrap_or(false)
}


pub fn yuh() -> Option<u64> {
    AEN_.lock().as_ref().map(|vm| vm.ad())
}
