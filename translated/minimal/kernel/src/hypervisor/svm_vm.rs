







use alloc::string::String;
use alloc::vec::Vec;
use alloc::boxed::Box;
use alloc::format;
use alloc::collections::VecDeque;
use core::sync::atomic::{AtomicU64, Ordering};
use spin::Mutex;

use super::{HypervisorError, Result};
use super::svm::{self, SvmExitCode, SvmFeatures, Ban};
use super::svm::vmcb::{Vmcb, control_offsets, state_offsets, clean_bits};
use super::svm::npt::{Npt, flags as npt_flags};
use super::mmio::{self, Eh};
use super::ioapic::IoApicState;
use super::hpet::HpetState;
use super::pci::PciBus;
use super::virtio_blk::{VirtioBlkState, VirtioConsoleState};


static CHW_: AtomicU64 = AtomicU64::new(1);


#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SvmVmState {
    Cu,
    Ai,
    Cl,
    Af,
    Gu,
}


#[derive(Debug, Clone, Default)]
pub struct SvmVmStats {
    pub ait: u64,
    pub bmp: u64,
    pub ank: u64,
    pub bkn: u64,
    pub axz: u64,
    pub cay: u64,     
    pub gwh: u64, 
    pub jap: u64,    
}


#[derive(Debug, Clone, Default)]
#[repr(C, align(16))]
pub struct SvmGuestRegs {
    pub rax: u64,
    pub rbx: u64,
    pub rcx: u64,
    pub rdx: u64,
    pub rsi: u64,
    pub rdi: u64,
    pub rbp: u64,
    pub r8: u64,
    pub r9: u64,
    pub r10: u64,
    pub r11: u64,
    pub r12: u64,
    pub r13: u64,
    pub r14: u64,
    pub r15: u64,
    pub rsp: u64,  
}


pub struct SvmVirtualMachine {
    
    pub ad: u64,
    
    pub j: String,
    
    pub g: SvmVmState,
    
    pub apy: usize,
    
    pub cm: SvmVmStats,
    
    pub(crate) vmcb: Option<Box<Vmcb>>,
    
    npt: Option<Npt>,
    
    fe: Vec<u8>,
    
    pub(crate) ej: SvmGuestRegs,
    
    pub ajv: u32,
    
    bjq: Option<usize>,
    
    features: SvmFeatures,
    
    pub ku: LapicState,
    
    pub pic: PicState,
    
    pub abu: PitState,
    
    pub vjn: u64,
    
    ion: u8,
    
    pub ioapic: IoApicState,
    
    pub hpet: HpetState,
    
    pub pci: PciBus,
    
    pub lub: u64,
    
    pub fuh: VecDeque<u8>,
    
    pub hzs: u8,
    
    pub pie: u8,
    
    pub mpj: Vec<u8>,
    
    pub xru: u8,
    
    pub xrv: u8,
    
    pub jvr: VirtioBlkState,
    
    pub jvs: VirtioConsoleState,
}


#[derive(Debug, Clone)]
pub struct LapicState {
    
    pub bnh: u32,
    
    pub fel: u32,
    
    pub dgc: u32,
    
    pub atq: u32,
    
    pub bim: u32,
    
    pub guv: u32,
    
    pub iq: bool,
    
    pub fmr: u64,
}

impl Default for LapicState {
    fn default() -> Self {
        Self {
            bnh: 0,
            fel: 0,
            dgc: 0,
            atq: 0x0001_0000, 
            bim: 0x1FF,             
            guv: 0,
            iq: false,
            fmr: 0,
        }
    }
}


#[derive(Debug, Clone)]
pub struct PicState {
    
    pub ayd: u8,
    
    pub bxd: u8,
    
    pub eun: u8,
    
    pub gsp: u8,
    
    pub cgc: u8,
    
    pub eyo: u8,
    
    pub dji: u8,
    
    pub jfb: u8,
    
    pub jr: bool,
}

impl Default for PicState {
    fn default() -> Self {
        Self {
            ayd: 0,
            bxd: 0,
            eun: 0xFF, 
            gsp: 0xFF,
            cgc: 0x08, 
            eyo: 0x70,
            dji: 0,
            jfb: 0,
            jr: false,
        }
    }
}


#[derive(Debug, Clone)]
pub struct PitChannel {
    
    pub ahs: u16,
    
    pub az: u16,
    
    pub ev: u8,
    
    pub vz: u8,
    
    pub czf: bool,
    pub gkx: u16,
    
    pub ccp: bool,
    
    pub an: bool,
}

impl Default for PitChannel {
    fn default() -> Self {
        Self {
            ahs: 0xFFFF,
            az: 0xFFFF,
            ev: 0,
            vz: 3, 
            czf: false,
            gkx: 0,
            ccp: false,
            an: false,
        }
    }
}


#[derive(Debug, Clone)]
pub struct PitState {
    pub lq: [PitChannel; 3],
}

impl Default for PitState {
    fn default() -> Self {
        Self {
            lq: [
                PitChannel::default(),
                PitChannel::default(),
                PitChannel::default(),
            ],
        }
    }
}

impl SvmVirtualMachine {
    
    pub fn new(j: &str, afc: usize) -> Result<Self> {
        
        if !svm::gkj() {
            return Err(HypervisorError::Btq);
        }
        
        let ad = CHW_.fetch_add(1, Ordering::SeqCst);
        let apy = afc * 1024 * 1024;
        let features = svm::fjn();
        
        
        let fe = alloc::vec![0u8; apy];
        
        
        let ajv = super::svm::npt::mva().unwrap_or(1);
        
        
        let bjq = super::console::fgb(ad, j);
        
        
        super::virtfs::nhj(ad);
        
        crate::serial_println!("[SVM-VM {}] Created '{}' with {} MB RAM, ASID={}", 
                              ad, j, afc, ajv);
        
        
        crate::lab_mode::trace_bus::epu(
            ad, &format!("CREATED '{}' mem={}MB ASID={}", j, afc, ajv)
        );
        
        
        super::api::eps(
            super::api::VmEventType::Cu,
            ad,
            super::api::VmEventData::Cj(format!("SVM VM '{}' created", j)),
        );
        
        Ok(SvmVirtualMachine {
            ad,
            j: String::from(j),
            g: SvmVmState::Cu,
            apy,
            cm: SvmVmStats::default(),
            vmcb: None,
            npt: None,
            fe,
            ej: SvmGuestRegs::default(),
            ajv,
            bjq: Some(bjq),
            features,
            ku: LapicState::default(),
            pic: PicState::default(),
            abu: PitState::default(),
            vjn: 0,
            ion: 0,
            ioapic: IoApicState::default(),
            hpet: HpetState::default(),
            pci: PciBus::default(),
            lub: 0,
            fuh: VecDeque::fc(256),
            hzs: 0,
            pie: 0,
            mpj: alloc::vec![0u8; 64 * 512], 
            xru: 0,
            xrv: 0,
            jvr: VirtioBlkState::fc(64 * 512),
            jvs: VirtioConsoleState::default(),
        })
    }
    
    
    pub fn cfp(&mut self) -> Result<()> {
        crate::serial_println!("[SVM-VM {}] Initializing VMCB and NPT...", self.ad);
        
        
        let mut vmcb = Vmcb::new();
        
        
        vmcb.wki();
        
        
        vmcb.elc(control_offsets::ATX_, self.ajv as u64);
        
        
        
        let ofk = alloc::vec![0xFFu8; 12288]; 
        let twk = ofk.fq() as u64;
        let ofl = twk - crate::memory::lr();
        vmcb.wjd(ofl);
        
        core::mem::forget(ofk);
        crate::serial_println!("[SVM-VM {}] IOPM allocated at HPA=0x{:X}", self.ad, ofl);
        
        
        
        let oon = alloc::vec![0xFFu8; 8192]; 
        let uqj = oon.fq() as u64;
        let ooo = uqj - crate::memory::lr();
        vmcb.wjg(ooo);
        core::mem::forget(oon);
        crate::serial_println!("[SVM-VM {}] MSRPM allocated at HPA=0x{:X}", self.ad, ooo);
        
        
        if self.features.npt {
            let mut npt = Npt::new(self.ajv);
            
            
            let tia = self.fe.fq() as u64;
            let ixk = tia - crate::memory::lr();
            
            if let Err(aa) = npt.jew(
                0,                          
                ixk,             
                self.apy as u64,    
                npt_flags::Axk,             
            ) {
                crate::serial_println!("[SVM-VM {}] NPT mapping failed: {}", self.ad, aa);
                return Err(HypervisorError::Cid);
            }
            
            
            let ore = npt.jm();
            
            
            
            vmcb.elc(control_offsets::BBU_, ore);
            
            
            let mut ord = vmcb.cgx(control_offsets::VU_);
            ord |= 1; 
            vmcb.elc(control_offsets::VU_, ord);
            
            crate::serial_println!("[SVM-VM {}] NPT enabled, N_CR3=0x{:X}", self.ad, ore);
            
            self.npt = Some(npt);
        } else {
            
            crate::serial_println!("[SVM-VM {}] NPT not available, using shadow paging", self.ad);
        }
        
        
        vmcb.elc(control_offsets::BHG_, 0); 
        
        self.vmcb = Some(Box::new(vmcb));
        
        crate::serial_println!("[SVM-VM {}] Initialization complete", self.ad);
        
        Ok(())
    }
    
    
    pub fn diy(&mut self, f: &[u8], dst: u64) -> Result<()> {
        let l = dst as usize;
        
        if l + f.len() > self.fe.len() {
            return Err(HypervisorError::Ns);
        }
        
        self.fe[l..l + f.len()].dg(f);
        
        crate::serial_println!("[SVM-VM {}] Loaded {} bytes at GPA 0x{:X}", 
                              self.ad, f.len(), dst);
        
        Ok(())
    }
    
    
    pub fn jpk(&mut self, mi: u64) -> Result<()> {
        let vmcb = self.vmcb.as_mut().ok_or(HypervisorError::Bd)?;
        vmcb.jpk();
        
        
        let kmb = (mi >> 4) << 4;
        let ip = mi & 0xF;
        
        vmcb.abz(state_offsets::SJ_, kmb);
        vmcb.abz(state_offsets::JU_, (kmb >> 4) as u64);
        vmcb.abz(state_offsets::Aw, ip);
        
        crate::serial_println!("[SVM-VM {}] Real mode: CS=0x{:X}, IP=0x{:X}", 
                              self.ad, kmb >> 4, ip);
        
        Ok(())
    }
    
    
    
    
    pub fn yyb(&mut self, f: &[u8]) {
        for &o in f {
            self.fuh.agt(o);
        }
    }
    
    
    pub fn iab(&mut self, mi: u64, ahu: u64) -> Result<()> {
        let vmcb = self.vmcb.as_mut().ok_or(HypervisorError::Bd)?;
        vmcb.iab(mi);
        
        vmcb.abz(state_offsets::Aw, mi);
        vmcb.abz(state_offsets::Hc, ahu);
        
        crate::serial_println!("[SVM-VM {}] Protected mode: RIP=0x{:X}, RSP=0x{:X}", 
                              self.ad, mi, ahu);
        
        Ok(())
    }
    
    
    pub fn pjq(&mut self, mi: u64, ahu: u64, avg: u64) -> Result<()> {
        let vmcb = self.vmcb.as_mut().ok_or(HypervisorError::Bd)?;
        vmcb.iab(mi);
        
        vmcb.abz(state_offsets::Aw, mi);
        vmcb.abz(state_offsets::Hc, ahu);
        
        
        
        
        self.ej.rsi = avg;
        
        
        self.ej.rbp = 0;
        self.ej.rdi = 0;
        
        crate::serial_println!("[SVM-VM {}] Linux protected mode: RIP=0x{:X}, RSP=0x{:X}, RSI(boot_params)=0x{:X}", 
                              self.ad, mi, ahu, avg);
        
        Ok(())
    }

    
    pub fn mfb(&mut self, mi: u64, ahu: u64, bnd: u64) -> Result<()> {
        let vmcb = self.vmcb.as_mut().ok_or(HypervisorError::Bd)?;
        vmcb.mfb(mi, bnd);
        
        vmcb.abz(state_offsets::Aw, mi);
        vmcb.abz(state_offsets::Hc, ahu);
        
        crate::serial_println!("[SVM-VM {}] Long mode: RIP=0x{:X}, RSP=0x{:X}, CR3=0x{:X}", 
                              self.ad, mi, ahu, bnd);
        
        Ok(())
    }
    
    
    pub fn fvn(
        &mut self,
        cwc: &[u8],
        wx: &str,
        apw: Option<&[u8]>,
    ) -> Result<()> {
        use super::linux_loader;
        
        
        if self.vmcb.is_none() {
            self.cfp()?;
        }
        
        crate::serial_println!("[SVM-VM {}] Loading Linux kernel ({} bytes)...", self.ad, cwc.len());
        
        
        let acf = linux_loader::oud(cwc)
            .jd(|aa| {
                crate::serial_println!("[SVM-VM {}] bzImage parse error: {:?}", self.ad, aa);
                HypervisorError::Bjt
            })?;
        
        crate::serial_println!("[SVM-VM {}] Kernel: protocol={}.{}, 64-bit={}, entry=0x{:X}",
            self.ad, acf.dh.dk >> 8, acf.dh.dk & 0xFF,
            acf.gtp, acf.hie);
        
        
        let config = linux_loader::Acq {
            wx: alloc::string::String::from(wx),
            apy: self.apy as u64,
            apw: apw.map(|bc| bc.ip()),
        };
        
        let aeq = linux_loader::ojt(&mut self.fe, &acf, &config)
            .jd(|aa| {
                crate::serial_println!("[SVM-VM {}] Linux load error: {:?}", self.ad, aa);
                HypervisorError::Bjt
            })?;
        
        crate::serial_println!("[SVM-VM {}] Linux loaded: entry=0x{:X}, stack=0x{:X}, cr3=0x{:X}, gdt=0x{:X}",
            self.ad, aeq.mi, aeq.ahu, aeq.jm, aeq.bun);
        
        
        {
            let vmcb = self.vmcb.as_mut().ok_or(HypervisorError::Bd)?;
            vmcb.wld(
                aeq.mi,
                aeq.ahu,
                aeq.jm,
                aeq.bun,
                39, 
            );
        }
        
        
        self.ej.rsi = aeq.avg;
        self.ej.rbp = 0;
        self.ej.rdi = 0;
        
        crate::serial_println!("[SVM-VM {}] Starting Linux with RSI=0x{:X} (boot_params)", 
            self.ad, aeq.avg);
        
        
        self.g = SvmVmState::Ai;
        crate::lab_mode::trace_bus::epu(self.ad, "LINUX_STARTED");
        self.hyg()?;
        
        Ok(())
    }
    
    
    pub fn ay(&mut self) -> Result<()> {
        if self.vmcb.is_none() {
            self.cfp()?;
        }
        
        self.g = SvmVmState::Ai;
        
        crate::serial_println!("[SVM-VM {}] Starting execution...", self.ad);
        crate::lab_mode::trace_bus::epu(self.ad, "STARTED");
        
        
        self.hyg()?;
        
        Ok(())
    }
    
    
    fn hyg(&mut self) -> Result<()> {
        
        let ekr = {
            let vmcb = self.vmcb.as_ref().ok_or(HypervisorError::Bd)?;
            vmcb.ki()
        };
        
        
        let mut aph = Ban {
            rax: self.ej.rax,
            rbx: self.ej.rbx,
            rcx: self.ej.rcx,
            rdx: self.ej.rdx,
            rsi: self.ej.rsi,
            rdi: self.ej.rdi,
            rbp: self.ej.rbp,
            r8: self.ej.r8,
            r9: self.ej.r9,
            r10: self.ej.r10,
            r11: self.ej.r11,
            r12: self.ej.r12,
            r13: self.ej.r13,
            r14: self.ej.r14,
            r15: self.ej.r15,
        };
        
        crate::serial_println!("[SVM-VM {}] Entering VM loop, RSI=0x{:X}", self.ad, aph.rsi);
        
        loop {
            if self.g != SvmVmState::Ai {
                break;
            }
            
            
            
            let mut jab = false;
            
            
            {
                let vmcb = self.vmcb.as_mut().ok_or(HypervisorError::Bd)?;
                
                vmcb.abz(state_offsets::Hc, self.ej.rsp);
                
                
                
                
                
                let dox = clean_bits::Cfq      
                          | clean_bits::Bxk      
                          | clean_bits::Chs        
                          | clean_bits::Cgk       
                          | clean_bits::Bxm;     
                vmcb.wik(dox);
                
                
                if self.ku.iq && self.ku.bnh > 0 {
                    let bnm = (self.ku.atq >> 16) & 1;
                    if bnm == 0 {
                        let mkx = (self.ku.atq >> 17) & 0x3;
                        let wj = (self.ku.atq & 0xFF) as u64;
                        let gfa = match self.ku.dgc & 0xB {
                            0x0 => 2u64, 0x1 => 4, 0x2 => 8, 0x3 => 16,
                            0x8 => 32, 0x9 => 64, 0xA => 128, 0xB => 1,
                            _ => 1,
                        };
                        let ez = self.cm.ait.ao(self.ku.fmr);
                        let qb = (ez * 256) / gfa;
                        
                        if qb >= self.ku.bnh as u64 {
                            let rflags = vmcb.xs(state_offsets::Kv);
                            if (rflags & 0x200) != 0 && wj > 0 {
                                let ebh: u64 = wj
                                                   | (0u64 << 8)
                                                   | (1u64 << 31);
                                vmcb.elc(control_offsets::GJ_, ebh);
                                jab = true;
                            }
                            match mkx {
                                1 => {
                                    self.ku.fmr = self.cm.ait;
                                    self.ku.fel = self.ku.bnh;
                                }
                                _ => {
                                    self.ku.bnh = 0;
                                    self.ku.fel = 0;
                                }
                            }
                        }
                    }
                }
                
                
                
                
                
                if !jab {
                    let vie = if self.abu.lq[0].ahs > 0 {
                        
                        
                        let ahs = self.abu.lq[0].ahs as u64;
                        
                        (ahs / 24).am(100).v(2000)
                    } else {
                        500 
                    };
                    
                    let wov = self.cm.ait.ao(self.lub);
                    if wov >= vie {
                        let rflags = vmcb.xs(state_offsets::Kv);
                        if (rflags & 0x200) != 0 {
                            
                            let wj = if let Some(bia) = self.ioapic.hli(0) {
                                if !bia.bnm && bia.wj > 0 {
                                    bia.wj as u64
                                } else {
                                    
                                    self.pic.cgc as u64
                                }
                            } else {
                                self.pic.cgc as u64
                            };
                            
                            if wj > 0 {
                                let ebh: u64 = wj
                                                   | (0u64 << 8)
                                                   | (1u64 << 31);
                                vmcb.elc(control_offsets::GJ_, ebh);
                                jab = true;
                            }
                        }
                        self.lub = self.cm.ait;
                    }
                }
                
            } 
                
            
            if !jab {
                
                let tqk = self.qyx();
                if let Some(wj) = tqk {
                    let vmcb = self.vmcb.as_mut().unwrap();
                    let kmt = vmcb.cgx(control_offsets::GJ_);
                    let kah = (kmt & (1u64 << 31)) != 0;
                    if !kah {
                        let rflags = vmcb.xs(state_offsets::Kv);
                        if (rflags & 0x200) != 0 { 
                            let ebh: u64 = wj
                                               | (0u64 << 8)    
                                               | (1u64 << 31);  
                            vmcb.elc(control_offsets::GJ_, ebh);
                        }
                    }
                }
            }
            
            
            if (self.hzs & 0x01) != 0 && !self.fuh.is_empty() {
                let wj = if let Some(bia) = self.ioapic.hli(4) {
                    if !bia.bnm && bia.wj > 0 {
                        bia.wj as u64
                    } else {
                        (self.pic.cgc + 4) as u64
                    }
                } else {
                    (self.pic.cgc + 4) as u64
                };
                
                if wj > 0 {
                    let vmcb = self.vmcb.as_mut().unwrap();
                    let kmt = vmcb.cgx(control_offsets::GJ_);
                    let kah = (kmt & (1u64 << 31)) != 0;
                    if !kah {
                        let rflags = vmcb.xs(state_offsets::Kv);
                        if (rflags & 0x200) != 0 {
                            let ebh: u64 = wj
                                               | (0u64 << 8)
                                               | (1u64 << 31);
                            vmcb.elc(control_offsets::GJ_, ebh);
                        }
                    }
                }
            }
            
            
            
            unsafe { svm::rbl(); }
            
            
            unsafe {
                svm::xsn(ekr, &mut aph);
            }
            
            
            unsafe { svm::wug(); }
            
            
            self.ej.rbx = aph.rbx;
            self.ej.rcx = aph.rcx;
            self.ej.rdx = aph.rdx;
            self.ej.rsi = aph.rsi;
            self.ej.rdi = aph.rdi;
            self.ej.rbp = aph.rbp;
            self.ej.r8 = aph.r8;
            self.ej.r9 = aph.r9;
            self.ej.r10 = aph.r10;
            self.ej.r11 = aph.r11;
            self.ej.r12 = aph.r12;
            self.ej.r13 = aph.r13;
            self.ej.r14 = aph.r14;
            self.ej.r15 = aph.r15;
            
            
            {
                let vmcb = self.vmcb.as_ref().ok_or(HypervisorError::Bd)?;
                self.ej.rax = vmcb.xs(state_offsets::Me);
                self.ej.rsp = vmcb.xs(state_offsets::Hc);
            }
            
            self.cm.ait += 1;
            
            
            let ipf = self.tlo()?;
            
            
            if self.cm.ait % 50 == 0 || !ipf {
                let pc = {
                    let vmcb = self.vmcb.as_ref().ok_or(HypervisorError::Bd)?;
                    vmcb.xs(state_offsets::Aw)
                };
                crate::lab_mode::trace_bus::nps(
                    self.ad,
                    self.ej.rax,
                    self.ej.rbx,
                    self.ej.rcx,
                    self.ej.rdx,
                    pc,
                    self.ej.rsp,
                );
            }
            
            if !ipf {
                break;
            }
            
            
            aph.rax = self.ej.rax;
            aph.rbx = self.ej.rbx;
            aph.rcx = self.ej.rcx;
            aph.rdx = self.ej.rdx;
            aph.rsi = self.ej.rsi;
            aph.rdi = self.ej.rdi;
            aph.rbp = self.ej.rbp;
            aph.r8 = self.ej.r8;
            aph.r9 = self.ej.r9;
            aph.r10 = self.ej.r10;
            aph.r11 = self.ej.r11;
            aph.r12 = self.ej.r12;
            aph.r13 = self.ej.r13;
            aph.r14 = self.ej.r14;
            aph.r15 = self.ej.r15;
        }
        
        if self.g == SvmVmState::Ai {
            self.g = SvmVmState::Af;
        }
        
        crate::serial_println!("[SVM-VM {}] Stopped after {} VMEXITs", self.ad, self.cm.ait);
        crate::lab_mode::trace_bus::epu(
            self.ad, &format!("STOPPED after {} exits (cpuid={} io={} msr={} hlt={} vmcall={})",
                self.cm.ait, self.cm.bmp, self.cm.ank,
                self.cm.bkn, self.cm.axz, self.cm.gwh)
        );
        
        Ok(())
    }
    
    
    fn tlo(&mut self) -> Result<bool> {
        
        let (nz, dqp, kum, wb, aqa) = {
            let vmcb = self.vmcb.as_ref().ok_or(HypervisorError::Bd)?;
            let ec = vmcb.cgx(control_offsets::Abm);
            let sjm = vmcb.cgx(control_offsets::Abn);
            let sjn = vmcb.cgx(control_offsets::Abo);
            let pc = vmcb.xs(state_offsets::Aw);
            let hta = if self.features.evl {
                vmcb.cgx(control_offsets::AFY_)
            } else {
                pc + 2 
            };
            (ec, sjm, sjn, pc, hta)
        };
        
        let cxn = SvmExitCode::from(nz);
        
        match cxn {
            SvmExitCode::Bdu => {
                self.cm.bmp += 1;
                
                crate::lab_mode::trace_bus::ept(
                    self.ad, "CPUID", wb,
                    &alloc::format!("EAX=0x{:X} ECX=0x{:X}", self.ej.rax, self.ej.rcx)
                );
                
                super::debug_monitor::bry(
                    self.ad, super::debug_monitor::DebugCategory::Ahg,
                    self.ej.rax, super::debug_monitor::HandleStatus::Gw,
                    wb, self.cm.ait,
                    &alloc::format!("ECX=0x{:X}", self.ej.rcx),
                );
                self.lat();
                let vmcb = self.vmcb.as_mut().ok_or(HypervisorError::Bd)?;
                vmcb.abz(state_offsets::Aw, aqa);
                Ok(true)
            }
            
            SvmExitCode::Bit => {
                self.cm.axz += 1;
                
                
                {
                    let vmcb = self.vmcb.as_mut().ok_or(HypervisorError::Bd)?;
                    vmcb.abz(state_offsets::Aw, aqa);
                    
                    
                    let rflags = vmcb.xs(state_offsets::Kv);
                    if (rflags & 0x200) != 0 { 
                        
                        let wj = if self.ku.iq && (self.ku.atq & 0xFF) > 0 
                            && ((self.ku.atq >> 16) & 1) == 0 {
                            (self.ku.atq & 0xFF) as u64
                        } else {
                            0x20
                        };
                        let ebh: u64 = wj
                                           | (0u64 << 8)    
                                           | (1u64 << 31);  
                        vmcb.elc(control_offsets::GJ_, ebh);
                    }
                }
                
                
                
                if self.cm.axz > 5_000_000 {
                    crate::serial_println!("[SVM-VM {}] Too many HLT exits ({}), stopping", self.ad, self.cm.axz);
                    self.g = SvmVmState::Af;
                    Ok(false)
                } else {
                    
                    if self.cm.axz % 10000 == 0 {
                        crate::serial_println!("[SVM-VM {}] HLT count: {}", self.ad, self.cm.axz);
                    }
                    Ok(true)
                }
            }
            
            SvmExitCode::Auo | SvmExitCode::Bjz => {
                self.cm.ank += 1;
                let port = ((dqp >> 16) & 0xFFFF) as u16;
                let te = if oh!(cxn, SvmExitCode::Auo) { "IN" } else { "OUT" };
                crate::lab_mode::trace_bus::npq(self.ad, te, port, self.ej.rax);
                self.lav(dqp);
                let vmcb = self.vmcb.as_mut().ok_or(HypervisorError::Bd)?;
                vmcb.abz(state_offsets::Aw, aqa);
                Ok(true)
            }
            
            SvmExitCode::Hx | SvmExitCode::Jr => {
                self.cm.bkn += 1;
                let rm = oh!(cxn, SvmExitCode::Jr);
                let uqi = if rm { "WRMSR" } else { "RDMSR" };
                crate::lab_mode::trace_bus::ept(
                    self.ad, uqi, wb,
                    &alloc::format!("MSR=0x{:X}", self.ej.rcx)
                );
                self.laz(rm);
                let vmcb = self.vmcb.as_mut().ok_or(HypervisorError::Bd)?;
                vmcb.abz(state_offsets::Aw, aqa);
                Ok(true)
            }
            
            SvmExitCode::Qe => {
                self.cm.cay += 1;
                let axy = kum;
                let error_code = dqp;
                
                
                let xsi = self.vmcb.as_ref().ok_or(HypervisorError::Bd)?;
                let (srv, hod) = xsi.thy();
                
                
                let aoq = mmio::cpw(&hod, srv, true);
                
                
                let tlt = self.tko(axy, error_code, wb, aoq.as_ref());
                
                if tlt {
                    
                    if let Some(ref adr) = aoq {
                        let vmcb = self.vmcb.as_mut().ok_or(HypervisorError::Bd)?;
                        vmcb.abz(state_offsets::Aw, wb + adr.ake as u64);
                    } else if self.features.evl {
                        
                        let vmcb = self.vmcb.as_ref().ok_or(HypervisorError::Bd)?;
                        let hta = vmcb.cgx(control_offsets::AFY_);
                        if hta > wb && hta < wb + 16 {
                            let vmcb = self.vmcb.as_mut().ok_or(HypervisorError::Bd)?;
                            vmcb.abz(state_offsets::Aw, hta);
                        } else {
                            
                            crate::serial_println!("[SVM-VM {}] NPF: decode failed, skipping 3 bytes at RIP=0x{:X}", 
                                self.ad, wb);
                            let vmcb = self.vmcb.as_mut().ok_or(HypervisorError::Bd)?;
                            vmcb.abz(state_offsets::Aw, wb + 3);
                        }
                    } else {
                        
                        crate::serial_println!("[SVM-VM {}] NPF: no decode/nrip, skipping 3 bytes", self.ad);
                        let vmcb = self.vmcb.as_mut().ok_or(HypervisorError::Bd)?;
                        vmcb.abz(state_offsets::Aw, wb + 3);
                    }
                    Ok(true)
                } else {
                    crate::lab_mode::trace_bus::npr(
                        self.ad, "NPF_VIOLATION", axy, error_code
                    );
                    super::debug_monitor::bry(
                        self.ad, super::debug_monitor::DebugCategory::Qe,
                        axy, super::debug_monitor::HandleStatus::Nd,
                        wb, self.cm.ait,
                        &alloc::format!("err=0x{:X}", error_code),
                    );
                    crate::serial_println!("[SVM-VM {}] FATAL NPF: GPA=0x{:X}, Error=0x{:X}, RIP=0x{:X}", 
                                          self.ad, axy, error_code, wb);
                    
                    super::isolation::pau(
                        self.ad,
                        axy,
                        None,
                        error_code,
                        wb,
                    );
                    
                    self.g = SvmVmState::Gu;
                    Ok(false)
                }
            }
            
            SvmExitCode::Bwa => {
                self.cm.gwh += 1;
                crate::lab_mode::trace_bus::ept(
                    self.ad, "VMMCALL", wb,
                    &alloc::format!("func=0x{:X} args=({:X},{:X},{:X})",
                        self.ej.rax, self.ej.rbx,
                        self.ej.rcx, self.ej.rdx)
                );
                let mfp = self.tlp();
                let vmcb = self.vmcb.as_mut().ok_or(HypervisorError::Bd)?;
                vmcb.abz(state_offsets::Aw, aqa);
                Ok(mfp)
            }
            
            SvmExitCode::Qt => {
                crate::lab_mode::trace_bus::epu(self.ad, "TRIPLE FAULT (shutdown)");
                super::debug_monitor::bry(
                    self.ad, super::debug_monitor::DebugCategory::Ahu,
                    0xFF, super::debug_monitor::HandleStatus::Nd,
                    wb, self.cm.ait, "TRIPLE FAULT",
                );
                crate::serial_println!("[SVM-VM {}] Guest SHUTDOWN (triple fault)", self.ad);
                self.g = SvmVmState::Gu;
                Ok(false)
            }
            
            SvmExitCode::Bjr => {
                self.cm.jap += 1;
                
                Ok(true)
            }
            
            
            SvmExitCode::Bwx => {
                
                
                
                let opn = dqp;
                crate::lab_mode::trace_bus::ept(
                    self.ad, "WRITE_CR0", wb,
                    &alloc::format!("val=0x{:X}", opn)
                );
                {
                    let vmcb = self.vmcb.as_mut().ok_or(HypervisorError::Bd)?;
                    
                    
                    let wcg = opn | 0x10; 
                    vmcb.hzw(wcg);
                    vmcb.abz(state_offsets::Aw, aqa);
                }
                Ok(true)
            }
            
            SvmExitCode::Bwy => {
                
                let usq = dqp;
                {
                    let vmcb = self.vmcb.as_mut().ok_or(HypervisorError::Bd)?;
                    vmcb.mei(usq);
                    vmcb.abz(state_offsets::Aw, aqa);
                }
                Ok(true)
            }
            
            SvmExitCode::Bwz => {
                
                let opo = dqp;
                crate::lab_mode::trace_bus::ept(
                    self.ad, "WRITE_CR4", wb,
                    &alloc::format!("val=0x{:X}", opo)
                );
                {
                    let vmcb = self.vmcb.as_mut().ok_or(HypervisorError::Bd)?;
                    vmcb.mej(opo);
                    vmcb.abz(state_offsets::Aw, aqa);
                }
                Ok(true)
            }
            
            
            SvmExitCode::Bqo | SvmExitCode::Bqp | SvmExitCode::Bqq => {
                
                
                let vmcb = self.vmcb.as_mut().ok_or(HypervisorError::Bd)?;
                vmcb.abz(state_offsets::Aw, aqa);
                Ok(true)
            }
            
            SvmExitCode::Bdv => {
                
                let vmcb = self.vmcb.as_mut().ok_or(HypervisorError::Bd)?;
                vmcb.abz(state_offsets::Aw, aqa);
                Ok(true)
            }
            
            
            
            SvmExitCode::Bxc => {
                
                
                let xwl = self.ej.rcx as u32;
                let bn = (self.ej.rdx << 32) | (self.ej.rax & 0xFFFF_FFFF);
                if xwl == 0 {
                    
                    
                    let grf = bn | 1; 
                    unsafe {
                        core::arch::asm!(
                            "xsetbv",
                            in("ecx") 0u32,
                            in("edx") (grf >> 32) as u32,
                            in("eax") grf as u32,
                        );
                    }
                }
                let vmcb = self.vmcb.as_mut().ok_or(HypervisorError::Bd)?;
                vmcb.abz(state_offsets::Aw, aqa);
                Ok(true)
            }
            
            SvmExitCode::Bjw => {
                
                
                let vmcb = self.vmcb.as_mut().ok_or(HypervisorError::Bd)?;
                vmcb.abz(state_offsets::Aw, aqa);
                Ok(true)
            }
            
            SvmExitCode::Bjx => {
                
                
                
                let vmcb = self.vmcb.as_mut().ok_or(HypervisorError::Bd)?;
                vmcb.abz(state_offsets::Aw, aqa);
                Ok(true)
            }
            
            SvmExitCode::Bwt => {
                
                
                unsafe { core::arch::asm!("wbinvd", options(nomem, nostack)); }
                let vmcb = self.vmcb.as_mut().ok_or(HypervisorError::Bd)?;
                vmcb.abz(state_offsets::Aw, aqa);
                Ok(true)
            }
            
            SvmExitCode::Bqm => {
                
                let tsc = unsafe { core::arch::x86_64::dxw() };
                self.ej.rax = tsc & 0xFFFF_FFFF;
                self.ej.rdx = tsc >> 32;
                let vmcb = self.vmcb.as_mut().ok_or(HypervisorError::Bd)?;
                vmcb.abz(state_offsets::Aw, aqa);
                Ok(true)
            }
            
            SvmExitCode::Uc => {
                
                let tsc = unsafe { core::arch::x86_64::dxw() };
                self.ej.rax = tsc & 0xFFFF_FFFF;
                self.ej.rdx = tsc >> 32;
                self.ej.rcx = 0; 
                let vmcb = self.vmcb.as_mut().ok_or(HypervisorError::Bd)?;
                vmcb.abz(state_offsets::Aw, aqa);
                Ok(true)
            }
            
            SvmExitCode::Bor => {
                
                let vmcb = self.vmcb.as_mut().ok_or(HypervisorError::Bd)?;
                vmcb.abz(state_offsets::Aw, aqa);
                Ok(true)
            }
            
            SvmExitCode::Bmo | SvmExitCode::Bmz | SvmExitCode::Chq => {
                
                
                let vmcb = self.vmcb.as_mut().ok_or(HypervisorError::Bd)?;
                vmcb.abz(state_offsets::Aw, aqa);
                Ok(true)
            }
            
            SvmExitCode::Bvu => {
                
                Ok(true)
            }
            
            SvmExitCode::Bub => {
                
                
                crate::serial_println!("[SVM-VM {}] TaskSwitch at RIP=0x{:X}", self.ad, wb);
                let vmcb = self.vmcb.as_mut().ok_or(HypervisorError::Bd)?;
                vmcb.abz(state_offsets::Aw, aqa);
                Ok(true)
            }
            
            
            SvmExitCode::Cbx | SvmExitCode::Cbw |
            SvmExitCode::Cbu | SvmExitCode::Cce |
            SvmExitCode::Cbv | SvmExitCode::Cci |
            SvmExitCode::Ccc | SvmExitCode::Ccb |
            SvmExitCode::Cbt | SvmExitCode::Ccj => {
                
                let wj = (nz - 0x40) as u8;
                if self.cm.ait < 200 {
                    crate::serial_println!("[SVM-VM {}] Exception #{} at RIP=0x{:X} — re-injecting", 
                        self.ad, wj, wb);
                }
                let vmcb = self.vmcb.as_mut().ok_or(HypervisorError::Bd)?;
                
                vmcb.hnz(wj, 3, None);
                Ok(true)
            }
            
            SvmExitCode::Cbz => {
                
                let error_code = dqp as u32;
                if self.cm.ait < 200 {
                    crate::serial_println!("[SVM-VM {}] #GP(0x{:X}) at RIP=0x{:X}", 
                        self.ad, error_code, wb);
                }
                let vmcb = self.vmcb.as_mut().ok_or(HypervisorError::Bd)?;
                vmcb.hnz(13, 3, Some(error_code));
                Ok(true)
            }
            
            SvmExitCode::Ccf => {
                
                let error_code = dqp as u32;
                let bha = kum;
                if self.cm.ait < 200 {
                    crate::serial_println!("[SVM-VM {}] #PF at 0x{:X} (err=0x{:X}) RIP=0x{:X}", 
                        self.ad, bha, error_code, wb);
                }
                let vmcb = self.vmcb.as_mut().ok_or(HypervisorError::Bd)?;
                vmcb.abz(state_offsets::Agy, bha);
                vmcb.hnz(14, 3, Some(error_code));
                Ok(true)
            }
            
            SvmExitCode::Cby => {
                
                crate::serial_println!("[SVM-VM {}] DOUBLE FAULT at RIP=0x{:X}", self.ad, wb);
                self.g = SvmVmState::Gu;
                Ok(false)
            }
            
            SvmExitCode::Cch | SvmExitCode::Ccd |
            SvmExitCode::Ccg => {
                
                let wj = (nz - 0x40) as u8;
                let error_code = dqp as u32;
                if self.cm.ait < 200 {
                    crate::serial_println!("[SVM-VM {}] Exception #{} (err=0x{:X}) at RIP=0x{:X}", 
                        self.ad, wj, error_code, wb);
                }
                let vmcb = self.vmcb.as_mut().ok_or(HypervisorError::Bd)?;
                vmcb.hnz(wj, 3, Some(error_code));
                Ok(true)
            }
            
            SvmExitCode::Cca => {
                
                let vmcb = self.vmcb.as_mut().ok_or(HypervisorError::Bd)?;
                vmcb.hnz(18, 3, None);
                Ok(true)
            }
            
            _ => {
                crate::serial_println!("[SVM-VM {}] Unhandled #VMEXIT: {:?} (0x{:X}) at RIP=0x{:X}", 
                                      self.ad, cxn, nz, wb);
                self.g = SvmVmState::Gu;
                Ok(false)
            }
        }
    }
    
    
    fn lat(&mut self) {
        let awa = self.ej.rax as u32;
        let bxj = self.ej.rcx as u32;
        
        
        match awa {
            
            0x4000_0000 => {
                
                self.ej.rax = 0x4000_0001;
                self.ej.rbx = 0x7473_7254; 
                self.ej.rcx = 0x7254_534F; 
                self.ej.rdx = 0x534F_7473; 
                return;
            }
            0x4000_0001 => {
                
                self.ej.rax = 0; 
                self.ej.rbx = 0;
                self.ej.rcx = 0;
                self.ej.rdx = 0;
                return;
            }
            _ => {}
        }
        
        
        let (htx, fpy, hty, htz): (u32, u32, u32, u32);
        
        unsafe {
            core::arch::asm!(
                "push rbx",
                "cpuid",
                "mov {out_ebx:e}, ebx",
                "pop rbx",
                inout("eax") awa => htx,
                inout("ecx") bxj => hty,
                fpy = bd(reg) fpy,
                bd("edx") htz,
            );
        }
        
        let (mut eax, mut ebx, mut ecx, mut edx) = (htx, fpy, hty, htz);
        
        match awa {
            0x0000_0000 => {
                
                
                
            }
            0x0000_0001 => {
                
                ecx &= !(1 << 5);   
                ecx |= 1 << 31;     
                
                ecx &= !(1 << 21);  
                
                ecx &= !(1 << 3);   
            }
            0x0000_0007 => {
                
                if bxj == 0 {
                    ebx &= !(1 << 0);  
                    ecx &= !(1 << 2);  
                    ecx &= !(1 << 4);  
                }
            }
            0x0000_000A => {
                
                eax = 0;
                ebx = 0;
                ecx = 0;
                edx = 0;
            }
            0x0000_000B | 0x0000_001F => {
                
                if bxj == 0 {
                    eax = 0; 
                    ebx = 1; 
                    ecx = (1 << 8) | bxj; 
                } else if bxj == 1 {
                    eax = 0;
                    ebx = 1; 
                    ecx = (2 << 8) | bxj; 
                } else {
                    eax = 0;
                    ebx = 0;
                    ecx = 0;
                }
            }
            0x8000_0001 => {
                
                ecx &= !(1 << 2);  
            }
            _ => {
                
            }
        }
        
        self.ej.rax = eax as u64;
        self.ej.rbx = ebx as u64;
        self.ej.rcx = ecx as u64;
        self.ej.rdx = edx as u64;
    }
    
    
    
    fn tko(&mut self, axy: u64, error_code: u64, wb: u64, aoq: Option<&Eh>) -> bool {
        
        const AYD_: u64 = 0xFEE0_0000;
        const CDH_: u64 = 0xFEE0_1000;
        const ADP_: u64 = 0xFEC0_0000;
        const CCA_: u64 = 0xFEC0_1000;
        const ADK_: u64 = 0xFED0_0000;
        const CAO_: u64 = 0xFED0_1000;
        
        match axy {
            
            AYD_..=CDH_ => {
                self.tkf(axy, error_code, aoq);
                true
            }
            
            ADP_..=CCA_ => {
                self.tjy(axy, error_code, aoq);
                true
            }
            
            ADK_..=CAO_ => {
                self.tju(axy, error_code, aoq);
                true
            }
            
            0xA0000..=0xBFFFF => {
                if self.cm.cay < 20 {
                    crate::serial_println!("[SVM-VM {}] VGA FB access at 0x{:X}", self.ad, axy);
                }
                self.gmu(aoq, 0);
                true
            }
            
            0xC0000..=0xFFFFF => {
                self.gmu(aoq, 0);
                true
            }
            
            pe if pe < self.apy as u64 => {
                crate::serial_println!("[SVM-VM {}] NPF in guest RAM at 0x{:X}, err=0x{:X}, RIP=0x{:X}", 
                    self.ad, pe, error_code, wb);
                false
            }
            
            pe if pe >= 0x1_0000_0000 => {
                if self.cm.cay < 50 {
                    crate::serial_println!("[SVM-VM {}] High MMIO access at 0x{:X} (absorbed)", self.ad, pe);
                }
                self.gmu(aoq, 0xFFFF_FFFF);
                true
            }
            _ => {
                if self.cm.cay < 50 {
                    crate::serial_println!("[SVM-VM {}] NPF: GPA=0x{:X}, err=0x{:X}, RIP=0x{:X}", 
                        self.ad, axy, error_code, wb);
                }
                false
            }
        }
    }
    
    
    
    fn lmg(&self, aoq: Option<&Eh>) -> u32 {
        if let Some(adr) = aoq {
            if let Some(gf) = adr.cag {
                return mmio::old(gf, adr.aqc) as u32;
            }
            if let Some(alq) = adr.nw {
                let ap = mmio::paf(&self.ej, alq);
                return mmio::old(ap, adr.aqc) as u32;
            }
        }
        
        self.ej.rax as u32
    }
    
    
    
    fn gmu(&mut self, aoq: Option<&Eh>, bn: u32) {
        if let Some(adr) = aoq {
            if !adr.rm {
                if let Some(alq) = adr.nw {
                    
                    
                    
                    mmio::pzy(&mut self.ej, alq, bn as u64);
                    return;
                }
            }
        }
        
        self.ej.rax = bn as u64;
    }
    
    
    fn tkf(&mut self, pe: u64, error_code: u64, aoq: Option<&Eh>) {
        let l = (pe & 0xFFF) as u32;
        let rm = (error_code & 0x2) != 0; 
        
        
        const ADW_: u32 = 0x020;
        const ADY_: u32 = 0x030;
        const UM_: u32 = 0x080;
        const ADU_: u32 = 0x0B0;
        const NW_: u32 = 0x0F0;
        const CDJ_: u32 = 0x100;
        const CDM_: u32 = 0x180;
        const CDI_: u32 = 0x200;
        const AYE_: u32 = 0x280;
        const AYI_: u32 = 0x300;
        const AYG_: u32 = 0x310;
        const ID_: u32 = 0x320;
        const AYN_: u32 = 0x330;
        const AYM_: u32 = 0x340;
        const AYJ_: u32 = 0x350;
        const AYK_: u32 = 0x360;
        const ADV_: u32 = 0x370;
        const KN_: u32 = 0x380;
        const ADX_: u32 = 0x390;
        const NX_: u32 = 0x3E0;
        
        if rm {
            let bn = self.lmg(aoq);
            match l {
                UM_ => {
                    self.ku.guv = bn & 0xFF;
                    
                    if let Some(ref mut vmcb) = self.vmcb {
                        vmcb.sx(control_offsets::DBK_, bn & 0x0F);
                    }
                }
                ADU_ => {
                    
                }
                NW_ => {
                    self.ku.bim = bn;
                    self.ku.iq = (bn & 0x100) != 0;
                    if self.cm.ait < 1000 {
                        crate::serial_println!("[SVM-VM {}] LAPIC SVR=0x{:X} enabled={}", 
                            self.ad, bn, self.ku.iq);
                    }
                }
                AYI_ => {
                    
                    if self.cm.ait < 1000 {
                        crate::serial_println!("[SVM-VM {}] LAPIC ICR write: 0x{:X} (IPI ignored, single vCPU)", 
                            self.ad, bn);
                    }
                }
                AYG_ => {} 
                ID_ => {
                    self.ku.atq = bn;
                    if self.cm.ait < 1000 {
                        let wj = bn & 0xFF;
                        let bnm = (bn >> 16) & 1;
                        let ev = (bn >> 17) & 0x3;
                        let upc = match ev {
                            0 => "one-shot",
                            1 => "periodic",
                            2 => "TSC-deadline",
                            _ => "reserved",
                        };
                        crate::serial_println!("[SVM-VM {}] LAPIC timer LVT: vec={} mode={} masked={}", 
                            self.ad, wj, upc, bnm);
                    }
                }
                AYJ_ | AYK_ | AYN_ | AYM_ | ADV_ => {
                    
                }
                KN_ => {
                    self.ku.bnh = bn;
                    self.ku.fel = bn; 
                    self.ku.fmr = self.cm.ait;
                    if self.cm.ait < 1000 && bn > 0 {
                        crate::serial_println!("[SVM-VM {}] LAPIC timer ICR={} (timer armed)", self.ad, bn);
                    }
                }
                NX_ => {
                    self.ku.dgc = bn;
                }
                AYE_ => {} 
                _ => {
                    if self.cm.ait < 200 {
                        crate::serial_println!("[SVM-VM {}] LAPIC write offset=0x{:X} val=0x{:X}", 
                            self.ad, l, bn);
                    }
                }
            }
        } else {
            
            let bn: u32 = match l {
                ADW_ => 0,                    
                ADY_ => 0x0005_0014,     
                UM_ => self.ku.guv,
                NW_ => self.ku.bim,
                CDJ_..=0x170 => 0,
                CDM_..=0x1F0 => 0,
                CDI_..=0x270 => 0,
                AYE_ => 0,
                AYI_ => 0,               
                AYG_ => 0,
                ID_ => self.ku.atq,
                AYN_ => 0x0001_0000, 
                AYM_ => 0x0001_0000,    
                AYJ_ => 0x0001_0000,        
                AYK_ => 0x0001_0000,        
                ADV_ => 0x0001_0000,    
                KN_ => self.ku.bnh,
                ADX_ => {
                    
                    if self.ku.bnh > 0 {
                        let ez = self.cm.ait.ao(self.ku.fmr);
                        let gfa = match self.ku.dgc & 0xB {
                            0x0 => 2u64, 0x1 => 4, 0x2 => 8, 0x3 => 16,
                            0x8 => 32, 0x9 => 64, 0xA => 128, 0xB => 1,
                            _ => 1,
                        };
                        let qb = (ez * 256) / gfa;
                        let ia = (self.ku.bnh as u64).ao(qb);
                        ia as u32
                    } else {
                        0
                    }
                }
                NX_ => self.ku.dgc,
                _ => 0,
            };
            self.gmu(aoq, bn);
        }
    }
    
    
    fn tjy(&mut self, pe: u64, error_code: u64, aoq: Option<&Eh>) {
        let l = pe - super::ioapic::ADP_;
        let rm = (error_code & 0x2) != 0;
        
        if rm {
            let bn = self.lmg(aoq);
            self.ioapic.write(l, bn);
            if self.cm.cay < 100 {
                crate::serial_println!("[SVM-VM {}] IOAPIC write offset=0x{:X} val=0x{:X}", 
                    self.ad, l, bn);
            }
        } else {
            let bn = self.ioapic.read(l);
            if self.cm.cay < 100 {
                crate::serial_println!("[SVM-VM {}] IOAPIC read offset=0x{:X} -> 0x{:X}", 
                    self.ad, l, bn);
            }
            self.gmu(aoq, bn);
        }
    }
    
    
    fn tju(&mut self, pe: u64, error_code: u64, aoq: Option<&Eh>) {
        let l = pe - super::hpet::ADK_;
        let rm = (error_code & 0x2) != 0;
        
        
        let aw = aoq.map(|bc| bc.aqc).unwrap_or(4);
        
        if rm {
            let bn = self.lmg(aoq) as u64;
            self.hpet.write(l, bn, aw);
            if self.cm.cay < 50 {
                crate::serial_println!("[SVM-VM {}] HPET write offset=0x{:X} val=0x{:X} size={}", 
                    self.ad, l, bn, aw);
            }
        } else {
            let bn = self.hpet.read(l, aw);
            if self.cm.cay < 50 {
                crate::serial_println!("[SVM-VM {}] HPET read offset=0x{:X} -> 0x{:X} size={}", 
                    self.ad, l, bn, aw);
            }
            self.gmu(aoq, bn as u32);
        }
    }
    
    
    
    fn qyx(&mut self) -> Option<u64> {
        let xhl = self.hpet.qzy();
        
        for (a, &(stw, lft)) in xhl.iter().cf() {
            if !stw {
                continue;
            }
            
            
            if let Some(bia) = self.ioapic.hli(lft) {
                if !bia.bnm && bia.wj > 0 {
                    
                    self.hpet.cru |= 1 << a;
                    
                    
                    let config = self.hpet.axe[a].config;
                    let vgs = (config >> 3) & 1 != 0;
                    if vgs {
                        
                        let dfd = self.hpet.axe[a].dpb;
                        if dfd > 0 {
                            self.hpet.axe[a].dpb = self.hpet.axe[a].dpb.cn(dfd);
                        }
                    } else {
                        
                        self.hpet.axe[a].config &= !(1 << 2);
                    }
                    
                    return Some(bia.wj as u64);
                }
            }
        }
        None
    }
    
    
    fn lav(&mut self, hnw: u64) {
        let txq = (hnw & 1) != 0;
        let port = ((hnw >> 16) & 0xFFFF) as u16;
        let dds = match (hnw >> 4) & 0x7 {
            0 => 1, 
            1 => 2, 
            2 => 4, 
            _ => 1,
        };
        
        if txq {
            
            let bn: u32 = match port {
                
                0x3F8 => {
                    
                    if let Some(hf) = self.fuh.awp() {
                        hf as u32
                    } else {
                        0
                    }
                }
                0x3F9 => self.hzs as u32, 
                0x3FA => {
                    
                    if (self.hzs & 0x01) != 0 && !self.fuh.is_empty() {
                        0xC4 
                    } else {
                        0xC1 
                    }
                }
                0x3FB => 0x03,                  
                0x3FC => 0x03,                  
                0x3FD => {
                    
                    let mut eum = 0x60u32; 
                    if !self.fuh.is_empty() {
                        eum |= 0x01; 
                    }
                    eum
                }
                0x3FE => 0xB0,                  
                0x3FF => 0,                     
                
                
                0x2F8..=0x2FF => match port & 0x7 {
                    5 => 0x60, 
                    _ => 0,
                },
                
                
                0x60 => 0,                      
                0x64 => 0x1C,                   
                
                
                0x20 => {
                    
                    self.pic.dji as u32
                }
                0x21 => self.pic.eun as u32,  
                0xA0 => 0,                           
                0xA1 => self.pic.gsp as u32,   
                
                
                0x40 | 0x41 | 0x42 => {
                    let bm = (port - 0x40) as usize;
                    let awo = &mut self.abu.lq[bm];
                    if awo.czf {
                        awo.czf = false;
                        awo.gkx as u32
                    } else {
                        
                        let wos = awo.az.nj(
                            (self.cm.ait & 0xFFFF) as u16
                        );
                        wos as u32
                    }
                }
                0x43 => 0,                      
                0x61 => 0x20,                   
                
                
                0x70 => self.ion as u32,
                0x71 => {
                    
                    (match self.ion {
                        0x00 => 0x00u32,  
                        0x02 => 0x30,  
                        0x04 => 0x12,  
                        0x06 => 0x02,  
                        0x07 => 0x17,  
                        0x08 => 0x02,  
                        0x09 => 0x26,  
                        0x0A => 0x26,  
                        0x0B => 0x02,  
                        0x0C => 0x00,  
                        0x0D => 0x80,  
                        0x0E => 0x00,  
                        0x0F => 0x00,  
                        0x10 => 0x00,  
                        0x12 => 0x00,  
                        0x14 => 0x06,  
                        0x15 => 0x80,  
                        0x16 => 0x02,  
                        0x17 => 0x00,  
                        0x18 => 0x00,  
                        0x32 => 0x20,  
                        _ => 0x00,
                    })
                }
                
                
                0x00..=0x0F | 0x80..=0x8F | 0xC0..=0xDF => 0,
                
                
                0x3B0..=0x3DF => 0,             
                
                
                0xCF8 => self.pci.ozx(),
                0xCFC..=0xCFF => {
                    let aok = (port - 0xCFC) as u8;
                    self.pci.duw(aok)
                }
                
                
                0xC000..=0xC03F => {
                    let l = port - 0xC000;
                    self.jvs.crq(l)
                }
                
                
                0xC040..=0xC07F => {
                    let l = port - 0xC040;
                    self.jvr.crq(l)
                }
                
                
                0xB000 => 0,           
                0xB002 => 0,           
                0xB004 => 0,           
                0xB008 => {
                    
                    
                    let qb = self.cm.ait.hx(4); 
                    (qb & 0xFFFF_FFFF) as u32
                }
                0xB009..=0xB00B => {
                    
                    let qb = self.cm.ait.hx(4);
                    let aok = (port - 0xB008) as u32;
                    ((qb >> (aok * 8)) & 0xFF) as u32
                }
                0xB00C..=0xB03F => 0,  
                
                
                0xE9 => 0,                      
                0xED => 0,                      
                0x92 => 0x02,                   
                
                _ => {
                    
                    if self.cm.ank < 50 {
                        crate::serial_println!("[SVM-VM {}] Unhandled IN port 0x{:X}", self.ad, port);
                    }
                    super::debug_monitor::bry(
                        self.ad, super::debug_monitor::DebugCategory::Iu,
                        port as u64, super::debug_monitor::HandleStatus::Id,
                        self.vmcb.as_ref().map(|p| p.xs(state_offsets::Aw)).unwrap_or(0),
                        self.cm.ait, "",
                    );
                    0xFF
                }
            };
            self.ej.rax = (self.ej.rax & !0xFFFF_FFFF) | (bn as u64);
        } else {
            
            let bn = self.ej.rax as u32;
            
            match port {
                
                0x3F8 => {
                    let bm = (bn & 0xFF) as u8;
                    crate::serial_print!("{}", bm as char);
                    if let Some(bjq) = self.bjq {
                        super::console::write_char(bjq, bm as char);
                    }
                }
                
                0x2F8 => {
                    let bm = (bn & 0xFF) as u8;
                    crate::serial_print!("{}", bm as char);
                }
                
                0x3F9 => {
                    
                    self.hzs = bn as u8;
                }
                0x3FA => {
                    
                    self.pie = bn as u8;
                    if (bn & 0x02) != 0 {
                        
                        self.fuh.clear();
                    }
                }
                0x3FB..=0x3FF => {} 
                
                0x2F9..=0x2FF => {}
                
                
                0x20 => {
                    
                    let p = bn as u8;
                    if p & 0x10 != 0 {
                        
                        self.pic.ayd = 1;
                        self.pic.dji = 0;
                        self.pic.jfb = 0;
                        if self.cm.ank < 200 {
                            crate::serial_println!("[SVM-VM {}] PIC master: ICW1=0x{:02X}", self.ad, p);
                        }
                    } else if p & 0x08 != 0 {
                        
                    } else {
                        
                        if p == 0x20 {
                            
                            self.pic.dji = 0;
                        }
                    }
                }
                0x21 => {
                    
                    let p = bn as u8;
                    match self.pic.ayd {
                        1 => {
                            
                            self.pic.cgc = p & 0xF8;
                            self.pic.ayd = 2;
                            if self.cm.ank < 200 {
                                crate::serial_println!("[SVM-VM {}] PIC master: ICW2 vector_base=0x{:02X}", self.ad, p);
                            }
                        }
                        2 => {
                            
                            self.pic.ayd = 3;
                        }
                        3 => {
                            
                            self.pic.ayd = 0;
                            self.pic.jr = true;
                            if self.cm.ank < 200 {
                                crate::serial_println!("[SVM-VM {}] PIC master initialized: base=0x{:02X}", 
                                    self.ad, self.pic.cgc);
                            }
                        }
                        _ => {
                            
                            self.pic.eun = p;
                        }
                    }
                }
                0xA0 => {
                    
                    let p = bn as u8;
                    if p & 0x10 != 0 {
                        self.pic.bxd = 1;
                    } else if p == 0x20 {
                        
                    }
                }
                0xA1 => {
                    
                    let p = bn as u8;
                    match self.pic.bxd {
                        1 => {
                            self.pic.eyo = p & 0xF8;
                            self.pic.bxd = 2;
                            if self.cm.ank < 200 {
                                crate::serial_println!("[SVM-VM {}] PIC slave: ICW2 vector_base=0x{:02X}", self.ad, p);
                            }
                        }
                        2 => { self.pic.bxd = 3; }
                        3 => { self.pic.bxd = 0; }
                        _ => { self.pic.gsp = p; }
                    }
                }
                
                
                0x40 | 0x41 | 0x42 => {
                    let bm = (port - 0x40) as usize;
                    let p = bn as u8;
                    let awo = &mut self.abu.lq[bm];
                    match awo.vz {
                        1 => {
                            
                            awo.ahs = (awo.ahs & 0xFF00) | p as u16;
                            awo.az = awo.ahs;
                        }
                        2 => {
                            
                            awo.ahs = (awo.ahs & 0x00FF) | ((p as u16) << 8);
                            awo.az = awo.ahs;
                        }
                        3 => {
                            
                            if awo.ccp {
                                awo.ahs = (awo.ahs & 0x00FF) | ((p as u16) << 8);
                                awo.az = awo.ahs;
                                awo.ccp = false;
                                if bm == 0 && self.cm.ank < 200 {
                                    crate::serial_println!("[SVM-VM {}] PIT ch0: reload={} ({} Hz)", 
                                        self.ad, awo.ahs,
                                        if awo.ahs > 0 { 1193182 / awo.ahs as u32 } else { 0 });
                                }
                            } else {
                                awo.ahs = (awo.ahs & 0xFF00) | p as u16;
                                awo.ccp = true;
                            }
                        }
                        _ => {}
                    }
                }
                0x43 => {
                    
                    let p = bn as u8;
                    let channel = ((p >> 6) & 0x3) as usize;
                    let vz = (p >> 4) & 0x3;
                    let ev = (p >> 1) & 0x7;
                    
                    if channel < 3 {
                        if vz == 0 {
                            
                            self.abu.lq[channel].czf = true;
                            self.abu.lq[channel].gkx = self.abu.lq[channel].az;
                        } else {
                            self.abu.lq[channel].vz = vz;
                            self.abu.lq[channel].ev = ev;
                            self.abu.lq[channel].ccp = false;
                        }
                    }
                }
                
                
                0x70 => {
                    self.ion = (bn as u8) & 0x7F; 
                }
                0x71 => {} 
                
                
                0x61 => {}
                
                
                0x60 | 0x64 => {}
                
                
                0x00..=0x0F | 0x80..=0x8F | 0xC0..=0xDF => {}
                
                
                0x3B0..=0x3DF => {}
                
                
                0xCF8 => {
                    self.pci.dni(bn);
                    if self.cm.ank < 200 {
                        crate::serial_println!("[SVM-VM {}] PCI CFG ADDR = 0x{:08X}", self.ad, bn);
                    }
                }
                0xCFC..=0xCFF => {
                    let aok = (port - 0xCFC) as u8;
                    self.pci.mra(aok, bn);
                    if self.cm.ank < 200 {
                        let (_, aq, ba, ke, reg) = {
                            let ag = self.pci.dfe;
                            (ag >> 31 != 0, (ag >> 16) as u8 & 0xFF, (ag >> 11) as u8 & 0x1F, (ag >> 8) as u8 & 0x7, ag as u8 & 0xFC)
                        };
                        crate::serial_println!("[SVM-VM {}] PCI CFG WRITE {:02X}:{:02X}.{} reg=0x{:02X} val=0x{:X}", 
                            self.ad, aq, ba, ke, reg, bn);
                    }
                }
                
                
                0xC000..=0xC03F => {
                    let l = port - 0xC000;
                    let djy = self.jvs.edp(l, bn);
                    if djy {
                        
                        self.jvs.vmv(&self.fe);
                    }
                }
                
                
                0xC040..=0xC07F => {
                    let l = port - 0xC040;
                    let djy = self.jvr.edp(l, bn);
                    if djy {
                        
                        
                        let wus = self.mpj.len();
                        let wut = self.mpj.mw();
                        let umx = self.fe.mw();
                        let umw = self.fe.len();
                        
                        unsafe {
                            let storage = core::slice::bef(wut, wus);
                            let thz = core::slice::bef(umx, umw);
                            self.jvr.vmr(thz, storage);
                        }
                    }
                }
                
                
                0x92 => {}
                
                
                0xE9 => {
                    let bm = (bn & 0xFF) as u8;
                    crate::serial_print!("{}", bm as char);
                }
                0xED => {} 
                
                
                0xB000..=0xB003 => {
                    
                    if self.cm.ank < 100 {
                        crate::serial_println!("[SVM-VM {}] ACPI PM1a_EVT write: port=0x{:X} val=0x{:X}", self.ad, port, bn);
                    }
                }
                0xB004..=0xB005 => {
                    
                    if self.cm.ank < 100 {
                        crate::serial_println!("[SVM-VM {}] ACPI PM1a_CNT write: port=0x{:X} val=0x{:X}", self.ad, port, bn);
                    }
                    
                    if port == 0xB004 && (bn & 0x2000) != 0 {
                        let dwb = (bn >> 10) & 0x7;
                        crate::serial_println!("[SVM-VM {}] ACPI shutdown request: SLP_TYP={}", self.ad, dwb);
                        if dwb == 5 {
                            self.g = SvmVmState::Af;
                        }
                    }
                }
                0xB006..=0xB03F => {} 
                
                _ => {
                    if self.cm.ank < 50 {
                        crate::serial_println!("[SVM-VM {}] Unhandled OUT port 0x{:X} val=0x{:X}", self.ad, port, bn);
                    }
                    super::debug_monitor::bry(
                        self.ad, super::debug_monitor::DebugCategory::Lr,
                        port as u64, super::debug_monitor::HandleStatus::Id,
                        self.vmcb.as_ref().map(|p| p.xs(state_offsets::Aw)).unwrap_or(0),
                        self.cm.ait, &alloc::format!("val=0x{:X}", bn),
                    );
                }
            }
        }
    }
    
    
    fn laz(&mut self, rm: bool) {
        let msr = self.ej.rcx as u32;
        
        
        const KK_: u32 = 0x001B;
        const CAW_: u32 = 0x00FE;
        const AWM_: u32 = 0x0174;
        const AWO_: u32 = 0x0175;
        const AWN_: u32 = 0x0176;
        const CAV_: u32 = 0x0179;
        const AWJ_: u32 = 0x017A;
        const NN_: u32 = 0x01A0;
        const KL_: u32 = 0x0277;
        const AWK_: u32 = 0x02FF;
        const CN_: u32 = 0xC000_0080;
        const VQ_: u32 = 0xC000_0081;
        const VO_: u32 = 0xC000_0082;
        const VK_: u32 = 0xC000_0083;
        const VP_: u32 = 0xC000_0084;
        const VL_: u32 = 0xC000_0100;
        const VM_: u32 = 0xC000_0101;
        const VN_: u32 = 0xC000_0102;
        const OI_: u32 = 0xC000_0103;
        
        if rm {
            let bn = (self.ej.rdx << 32) | (self.ej.rax & 0xFFFF_FFFF);
            
            match msr {
                
                VQ_ => {
                    let vmcb = self.vmcb.as_mut().unwrap();
                    vmcb.abz(state_offsets::Bsb, bn);
                }
                VO_ => {
                    let vmcb = self.vmcb.as_mut().unwrap();
                    vmcb.abz(state_offsets::Bko, bn);
                }
                VK_ => {
                    let vmcb = self.vmcb.as_mut().unwrap();
                    vmcb.abz(state_offsets::Bde, bn);
                }
                VP_ => {
                    let vmcb = self.vmcb.as_mut().unwrap();
                    vmcb.abz(state_offsets::Brh, bn);
                }
                VN_ => {
                    let vmcb = self.vmcb.as_mut().unwrap();
                    vmcb.abz(state_offsets::AXR_, bn);
                }
                VL_ => {
                    let vmcb = self.vmcb.as_mut().unwrap();
                    vmcb.abz(state_offsets::ACK_, bn);
                }
                VM_ => {
                    let vmcb = self.vmcb.as_mut().unwrap();
                    vmcb.abz(state_offsets::ADE_, bn);
                }
                AWM_ => {
                    let vmcb = self.vmcb.as_mut().unwrap();
                    vmcb.abz(state_offsets::BGT_, bn);
                }
                AWO_ => {
                    let vmcb = self.vmcb.as_mut().unwrap();
                    vmcb.abz(state_offsets::BGV_, bn);
                }
                AWN_ => {
                    let vmcb = self.vmcb.as_mut().unwrap();
                    vmcb.abz(state_offsets::BGU_, bn);
                }
                KL_ => {
                    let vmcb = self.vmcb.as_mut().unwrap();
                    vmcb.abz(state_offsets::Awo, bn);
                }
                CN_ => {
                    let vmcb = self.vmcb.as_mut().unwrap();
                    
                    let wch = bn | 0x1000; 
                    vmcb.abz(state_offsets::Lh, wch);
                }
                
                KK_ | NN_ | AWJ_ |
                AWK_ | OI_ => {}
                
                0x0200..=0x020F => {}
                
                0x0400..=0x047F => {}
                _ => {
                    if self.cm.bkn < 100 {
                        crate::serial_println!("[SVM-VM {}] WRMSR 0x{:X} = 0x{:X} (ignored)", self.ad, msr, bn);
                    }
                    super::debug_monitor::bry(
                        self.ad, super::debug_monitor::DebugCategory::Jr,
                        msr as u64, super::debug_monitor::HandleStatus::Id,
                        self.vmcb.as_ref().map(|p| p.xs(state_offsets::Aw)).unwrap_or(0),
                        self.cm.ait, &alloc::format!("val=0x{:X}", bn),
                    );
                }
            }
        } else {
            
            let bn: u64 = match msr {
                
                VQ_ => {
                    let vmcb = self.vmcb.as_ref().unwrap();
                    vmcb.xs(state_offsets::Bsb)
                }
                VO_ => {
                    let vmcb = self.vmcb.as_ref().unwrap();
                    vmcb.xs(state_offsets::Bko)
                }
                VK_ => {
                    let vmcb = self.vmcb.as_ref().unwrap();
                    vmcb.xs(state_offsets::Bde)
                }
                VP_ => {
                    let vmcb = self.vmcb.as_ref().unwrap();
                    vmcb.xs(state_offsets::Brh)
                }
                VN_ => {
                    let vmcb = self.vmcb.as_ref().unwrap();
                    vmcb.xs(state_offsets::AXR_)
                }
                VL_ => {
                    let vmcb = self.vmcb.as_ref().unwrap();
                    vmcb.xs(state_offsets::ACK_)
                }
                VM_ => {
                    let vmcb = self.vmcb.as_ref().unwrap();
                    vmcb.xs(state_offsets::ADE_)
                }
                AWM_ => {
                    let vmcb = self.vmcb.as_ref().unwrap();
                    vmcb.xs(state_offsets::BGT_)
                }
                AWO_ => {
                    let vmcb = self.vmcb.as_ref().unwrap();
                    vmcb.xs(state_offsets::BGV_)
                }
                AWN_ => {
                    let vmcb = self.vmcb.as_ref().unwrap();
                    vmcb.xs(state_offsets::BGU_)
                }
                KL_ => {
                    let vmcb = self.vmcb.as_ref().unwrap();
                    vmcb.xs(state_offsets::Awo)
                }
                CN_ => {
                    let vmcb = self.vmcb.as_ref().unwrap();
                    vmcb.xs(state_offsets::Lh)
                }
                
                KK_ => 0xFEE0_0900,
                
                CAW_ => 0,
                
                CAV_ => 0,
                AWJ_ => 0,
                
                NN_ => 1, 
                
                AWK_ => 0x06,
                
                OI_ => 0,
                
                0x0200..=0x020F => 0,
                
                0x0400..=0x047F => 0,
                _ => {
                    if self.cm.bkn < 100 {
                        crate::serial_println!("[SVM-VM {}] RDMSR 0x{:X} = 0 (default)", self.ad, msr);
                    }
                    super::debug_monitor::bry(
                        self.ad, super::debug_monitor::DebugCategory::Hx,
                        msr as u64, super::debug_monitor::HandleStatus::Id,
                        self.vmcb.as_ref().map(|p| p.xs(state_offsets::Aw)).unwrap_or(0),
                        self.cm.ait, "returned 0",
                    );
                    0
                }
            };
            
            self.ej.rax = bn & 0xFFFF_FFFF;
            self.ej.rdx = bn >> 32;
        }
    }
    
    
    fn tlp(&mut self) -> bool {
        let gw = self.ej.rax;
        let aai = self.ej.rbx;
        let agf = self.ej.rcx;
        let bfx = self.ej.rdx;
        
        crate::serial_println!("[SVM-VM {}] VMMCALL: func=0x{:X}, args=({:X}, {:X}, {:X})", 
                              self.ad, gw, aai, agf, bfx);
        
        let (result, mfp): (i64, bool) = match gw {
            
            0x00 => {
                self.g = SvmVmState::Af;
                (0, false)
            }
            
            
            0x01 => {
                self.tqt(aai);
                (0, true)
            }
            
            
            0x02 => {
                (unsafe { core::arch::x86_64::dxw() as i64 }, true)
            }
            
            _ => (-1, true), 
        };
        
        
        self.ej.rax = result as u64;
        
        
        super::api::eps(
            super::api::VmEventType::Acd,
            self.ad,
            super::api::VmEventData::Cfe { gw, result },
        );
        
        mfp
    }
    
    
    fn tqt(&self, pe: u64) {
        let l = pe as usize;
        if l < self.fe.len() {
            
            let cat = (self.fe.len() - l).v(256);
            let slice = &self.fe[l..l + cat];
            
            if let Some(uwd) = slice.iter().qf(|&r| r == 0) {
                if let Ok(e) = core::str::jg(&slice[..uwd]) {
                    crate::serial_println!("[SVM-VM {} PRINT] {}", self.ad, e);
                    if let Some(bjq) = self.bjq {
                        for bm in e.bw() {
                            super::console::write_char(bjq, bm);
                        }
                    }
                }
            }
        }
    }
    
    
    pub fn rb(&mut self) -> Result<()> {
        if self.g == SvmVmState::Ai {
            self.g = SvmVmState::Cl;
            crate::serial_println!("[SVM-VM {}] Paused", self.ad);
        }
        Ok(())
    }
    
    
    pub fn anu(&mut self) -> Result<()> {
        if self.g == SvmVmState::Cl {
            self.g = SvmVmState::Ai;
            crate::serial_println!("[SVM-VM {}] Resumed", self.ad);
        }
        Ok(())
    }
    
    
    pub fn asx(&self) -> &SvmVmStats {
        &self.cm
    }
    
    
    pub fn drd(&self) -> SvmVmState {
        self.g
    }
    
    
    pub fn duy(&self, pe: u64, len: usize) -> Option<&[u8]> {
        let l = pe as usize;
        if l + len <= self.fe.len() {
            Some(&self.fe[l..l + len])
        } else {
            None
        }
    }
    
    
    pub fn jxg(&mut self, pe: u64, f: &[u8]) -> Result<()> {
        let l = pe as usize;
        if l + f.len() <= self.fe.len() {
            self.fe[l..l + f.len()].dg(f);
            Ok(())
        } else {
            Err(HypervisorError::Ns)
        }
    }
}

impl Drop for SvmVirtualMachine {
    fn drop(&mut self) {
        
        super::svm::npt::sxb(self.ajv);
        
        
        if let Some(xyl) = self.bjq {
            
        }
        
        
        super::virtfs::vuz(self.ad);
        
        crate::serial_println!("[SVM-VM {}] Destroyed", self.ad);
    }
}


static XP_: Mutex<Vec<SvmVirtualMachine>> = Mutex::new(Vec::new());


pub fn dpg(j: &str, afc: usize) -> Result<u64> {
    let vm = SvmVirtualMachine::new(j, afc)?;
    let ad = vm.ad;
    XP_.lock().push(vm);
    Ok(ad)
}


pub fn coa<G, Ac>(ad: u64, bb: G) -> Option<Ac>
where
    G: FnOnce(&mut SvmVirtualMachine) -> Ac,
{
    let mut bfr = XP_.lock();
    bfr.el().du(|vm| vm.ad == ad).map(bb)
}


pub fn hqc() -> Vec<(u64, String, SvmVmState)> {
    XP_.lock()
        .iter()
        .map(|vm| (vm.ad, vm.j.clone(), vm.g))
        .collect()
}


pub fn ylw(ad: u64) -> Result<()> {
    let mut bfr = XP_.lock();
    if let Some(u) = bfr.iter().qf(|vm| vm.ad == ad) {
        bfr.remove(u);
        Ok(())
    } else {
        Err(HypervisorError::Mo)
    }
}





#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zsc() {
        let pic = PicState::default();
        assert_eq!(pic.eun, 0xFF);
        assert_eq!(pic.gsp, 0xFF);
        assert_eq!(pic.cgc, 0x08);
        assert_eq!(pic.eyo, 0x70);
        assert_eq!(pic.ayd, 0);
        assert!(!pic.jr);
    }

    #[test]
    fn zsd() {
        let mut pic = PicState::default();
        
        pic.ayd = 1;
        pic.dji = 0;
        
        pic.cgc = 0x20;
        pic.ayd = 2;
        
        pic.ayd = 3;
        
        pic.ayd = 0;
        pic.jr = true;
        assert_eq!(pic.cgc, 0x20);
        assert!(pic.jr);
    }

    #[test]
    fn zse() {
        let abu = PitState::default();
        assert_eq!(abu.lq[0].ahs, 0xFFFF);
        assert_eq!(abu.lq[0].az, 0xFFFF);
        assert_eq!(abu.lq[0].vz, 3);
        assert!(!abu.lq[0].czf);
        assert_eq!(abu.lq.len(), 3);
    }

    #[test]
    fn zsf() {
        let mut bm = PitChannel::default();
        bm.vz = 3;
        
        bm.ahs = (bm.ahs & 0xFF00) | 0x9C;
        bm.ccp = true;
        
        bm.ahs = (bm.ahs & 0x00FF) | (0x2E << 8);
        bm.az = bm.ahs;
        bm.ccp = false;
        assert_eq!(bm.ahs, 0x2E9C);
        assert_eq!(bm.az, 0x2E9C);
    }

    #[test]
    fn zrw() {
        let ku = LapicState::default();
        assert_eq!(ku.bnh, 0);
        assert!(!ku.iq);
        assert_ne!(ku.atq & 0x0001_0000, 0); 
        assert_eq!(ku.bim, 0x1FF);
    }

    #[test]
    fn zry() {
        let mut ku = LapicState::default();
        ku.bim = 0x1FF;
        ku.iq = (ku.bim & 0x100) != 0;
        assert!(ku.iq);
        ku.bim = 0x0FF;
        ku.iq = (ku.bim & 0x100) != 0;
        assert!(!ku.iq);
    }

    #[test]
    fn zrx() {
        let map = [(0x0u32, 2u64), (0x1, 4), (0x2, 8), (0x3, 16),
                    (0x8, 32), (0x9, 64), (0xA, 128), (0xB, 1)];
        for &(dgc, qy) in &map {
            let bc = match dgc & 0xB {
                0x0 => 2u64, 0x1 => 4, 0x2 => 8, 0x3 => 16,
                0x8 => 32, 0x9 => 64, 0xA => 128, 0xB => 1, _ => 1,
            };
            assert_eq!(bc, qy, "dcr=0x{:X}", dgc);
        }
    }
}
