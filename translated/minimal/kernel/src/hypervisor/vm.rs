









use alloc::string::String;
use alloc::vec::Vec;
use alloc::boxed::Box;
use spin::Mutex;
use super::{HypervisorError, Result};
use super::vmcs::{Vmcs, fields, exit_reason};
use super::ept::EptManager;
use super::vmx;


#[derive(Debug, Clone, Copy, PartialEq)]
pub enum VmState {
    Cu,
    Ai,
    Cl,
    Af,
    Gu,
}


#[derive(Debug, Clone, Default)]
pub struct VmStats {
    pub gwg: u64,
    pub bmp: u64,
    pub ank: u64,
    pub bkn: u64,
    pub axz: u64,
    pub fhx: u64,
}


#[derive(Debug, Clone, Default)]
#[repr(C)]
pub struct GuestRegs {
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
}


pub struct VirtualMachine {
    pub ad: u64,
    pub j: String,
    pub g: VmState,
    pub apy: usize,
    pub cm: VmStats,
    vmcs: Option<Vmcs>,
    ept: Option<EptManager>,
    fe: Vec<u8>,
    ej: GuestRegs,
    
    bjq: Option<usize>,
    
    vpid: Option<u16>,
}

impl VirtualMachine {
    pub fn new(ad: u64, j: &str, afc: usize) -> Result<Self> {
        let apy = afc * 1024 * 1024;
        
        
        let fe = alloc::vec![0u8; apy];
        
        
        let bjq = super::console::fgb(ad, j);
        
        
        super::virtfs::nhj(ad);
        
        
        let vpid = super::vpid::ijo();
        if vpid.is_some() {
            crate::serial_println!("[VM {}] Allocated VPID {} for TLB isolation", ad, vpid.unwrap());
        }
        
        
        super::api::eps(
            super::api::VmEventType::Cu,
            ad,
            super::api::VmEventData::Cj(alloc::format!("VM '{}' created", j)),
        );
        
        Ok(VirtualMachine {
            ad,
            j: String::from(j),
            g: VmState::Cu,
            apy,
            cm: VmStats::default(),
            vmcs: None,
            ept: None,
            fe,
            ej: GuestRegs::default(),
            bjq: Some(bjq),
            vpid,
        })
    }
    
    
    pub fn elx(&mut self, cac: &str, bqx: &str, awr: bool) {
        super::virtfs::elx(self.ad, cac, bqx, awr);
    }
    
    
    pub fn cfp(&mut self) -> Result<()> {
        crate::serial_println!("[VM {}] Initializing VMCS and EPT", self.ad);
        
        
        let fyj = vmx::bcg(vmx::TZ_);
        let cty = (fyj & 0x7FFF_FFFF) as u32;
        
        
        let mut vmcs = Vmcs::new(cty)?;
        vmcs.load()?;
        
        
        vmcs.wks()?;
        vmcs.wkt()?;
        vmcs.wkq()?;
        
        
        vmcs.wlm(self.vpid)?;
        
        
        let mut ept = EptManager::new(self.apy)?;
        ept.wky(&self.fe)?;
        
        
        vmcs.write(fields::BUH_, ept.sna().cvr())?;
        
        self.vmcs = Some(vmcs);
        self.ept = Some(ept);
        
        crate::serial_println!("[VM {}] Initialization complete (VPID={:?})", self.ad, self.vpid);
        
        Ok(())
    }
    
    
    pub fn diy(&mut self, f: &[u8], dst: u64) -> Result<()> {
        let l = dst as usize;
        
        if l + f.len() > self.fe.len() {
            return Err(HypervisorError::Ns);
        }
        
        self.fe[l..l + f.len()].dg(f);
        
        crate::serial_println!("[VM {}] Loaded {} bytes at 0x{:X}", 
                              self.ad, f.len(), dst);
        
        Ok(())
    }
    
    
    pub fn ay(&mut self, mi: u64, ahu: u64) -> Result<()> {
        if self.vmcs.is_none() {
            self.cfp()?;
        }
        
        let vmcs = self.vmcs.as_ref().unwrap();
        
        
        vmcs.wla(mi, ahu)?;
        
        
        let fic = pyn as *const () as u64;
        
        
        let hnb = alloc::vec![0u8; 16384];
        let iyr = (hnb.fq() as u64 + 16384) & !0xF;
        core::mem::forget(hnb); 
        
        vmcs.pjo(fic, iyr)?;
        
        self.g = VmState::Ai;
        
        crate::serial_println!("[VM {}] Starting at RIP=0x{:X}, RSP=0x{:X}", 
                              self.ad, mi, ahu);
        crate::serial_println!("[VM {}] Host exit handler=0x{:X}, host stack=0x{:X}",
                              self.ad, fic, iyr);
        
        
        self.hyg()?;
        
        Ok(())
    }
    
    
    
    
    
    pub fn fvn(
        &mut self,
        cwc: &[u8],
        wx: &str,
        apw: Option<&[u8]>,
    ) -> Result<()> {
        use super::linux_loader;
        
        crate::serial_println!("[VM {}] Starting Linux kernel ({} bytes)", self.ad, cwc.len());
        
        
        let aeq = linux_loader::vks(
            &mut self.fe,
            cwc,
            wx,
            apw,
        )?;
        
        
        if self.vmcs.is_none() {
            self.cfp()?;
        }
        
        let vmcs = self.vmcs.as_ref().unwrap();
        
        
        linux_loader::rnu(vmcs, &aeq)?;
        
        
        self.ej.rsi = aeq.avg;
        self.pfl();
        
        
        let fic = pyn as *const () as u64;
        let hnb = alloc::vec![0u8; 16384];
        let iyr = (hnb.fq() as u64 + 16384) & !0xF;
        core::mem::forget(hnb);
        
        vmcs.pjo(fic, iyr)?;
        
        self.g = VmState::Ai;
        
        crate::serial_println!("[VM {}] Linux: RIP=0x{:X} RSP=0x{:X} RSI(boot_params)=0x{:X} CR3=0x{:X}",
                              self.ad, aeq.mi, aeq.ahu,
                              aeq.avg, aeq.jm);
        
        self.hyg()?;
        
        Ok(())
    }
    
    
    
    
    
    fn hyg(&mut self) -> Result<()> {
        let mut lib = false;
        
        loop {
            
            let result = xso(lib);
            
            if result != 0 {
                
                let rq = vmx::igs(fields::DBI_).unwrap_or(0xFFFF);
                crate::serial_println!("[VM {}] VM entry failed! error={}", self.ad, rq);
                self.g = VmState::Gu;
                return if lib {
                    Err(HypervisorError::Bwb)
                } else {
                    Err(HypervisorError::Bvz)
                };
            }
            
            lib = true;
            
            
            self.cm.gwg += 1;
            
            
            self.ugy();
            
            
            let ipf = self.tlm()?;
            
            if !ipf {
                break;
            }
            
            
            self.pfl();
        }
        
        self.g = VmState::Af;
        crate::serial_println!("[VM {}] Stopped after {} exits (cpuid={} io={} hlt={} ept={})",
                              self.ad, self.cm.gwg, self.cm.bmp,
                              self.cm.ank, self.cm.axz,
                              self.cm.fhx);
        Ok(())
    }
    
    
    fn ugy(&mut self) {
        unsafe {
            let ahy = &YL_;
            self.ej.rax = ahy.rax;
            self.ej.rbx = ahy.rbx;
            self.ej.rcx = ahy.rcx;
            self.ej.rdx = ahy.rdx;
            self.ej.rsi = ahy.rsi;
            self.ej.rdi = ahy.rdi;
            self.ej.rbp = ahy.rbp;
            self.ej.r8  = ahy.r8;
            self.ej.r9  = ahy.r9;
            self.ej.r10 = ahy.r10;
            self.ej.r11 = ahy.r11;
            self.ej.r12 = ahy.r12;
            self.ej.r13 = ahy.r13;
            self.ej.r14 = ahy.r14;
            self.ej.r15 = ahy.r15;
        }
    }
    
    
    fn pfl(&self) {
        unsafe {
            let ahy = &mut YL_;
            ahy.rax = self.ej.rax;
            ahy.rbx = self.ej.rbx;
            ahy.rcx = self.ej.rcx;
            ahy.rdx = self.ej.rdx;
            ahy.rsi = self.ej.rsi;
            ahy.rdi = self.ej.rdi;
            ahy.rbp = self.ej.rbp;
            ahy.r8  = self.ej.r8;
            ahy.r9  = self.ej.r9;
            ahy.r10 = self.ej.r10;
            ahy.r11 = self.ej.r11;
            ahy.r12 = self.ej.r12;
            ahy.r13 = self.ej.r13;
            ahy.r14 = self.ej.r14;
            ahy.r15 = self.ej.r15;
        }
    }
    
    
    fn tlm(&mut self) -> Result<bool> {
        
        let (exit_reason, dqq, wb, hof) = {
            let vmcs = self.vmcs.as_ref().unwrap();
            let ctt = vmcs.read(fields::DBG_)? as u32 & 0xFFFF;
            let vow = vmcs.read(fields::BUR_)?;
            let pc = vmcs.read(fields::FG_)?;
            let len = vmcs.read(fields::DBD_).unwrap_or(0);
            (ctt, vow, pc, len)
        };
        
        match exit_reason {
            exit_reason::Apr => {
                self.cm.bmp += 1;
                crate::lab_mode::trace_bus::ept(
                    self.ad, "CPUID", wb,
                    &alloc::format!("EAX=0x{:X}", self.ej.rax)
                );
                self.lat()?;
                
                let vmcs = self.vmcs.as_ref().unwrap();
                vmcs.write(fields::FG_, wb + 2)?;
                Ok(true)
            }
            
            exit_reason::Atl => {
                self.cm.axz += 1;
                
                
                let vmcs = self.vmcs.as_ref().unwrap();
                vmcs.write(fields::FG_, wb + hof)?;
                
                
                if self.cm.axz > 50000 {
                    crate::serial_println!("[VM {}] Too many HLTs ({}), stopping", self.ad, self.cm.axz);
                    Ok(false)
                } else {
                    Ok(true)
                }
            }
            
            exit_reason::CCI_ => {
                self.cm.ank += 1;
                let port = ((dqq >> 16) & 0xFFFF) as u16;
                let te = if (dqq & 8) == 0 { "OUT" } else { "IN" };
                crate::lab_mode::trace_bus::npq(self.ad, te, port, self.ej.rax);
                self.lav(dqq)?;
                let vmcs = self.vmcs.as_ref().unwrap();
                vmcs.write(fields::FG_, wb + hof)?;
                Ok(true)
            }
            
            exit_reason::Cjh | exit_reason::Bwl => {
                self.cm.bkn += 1;
                self.laz(exit_reason == exit_reason::Bwl)?;
                let vmcs = self.vmcs.as_ref().unwrap();
                vmcs.write(fields::FG_, wb + hof)?;
                Ok(true)
            }
            
            exit_reason::BUI_ => {
                self.cm.fhx += 1;
                let vmcs = self.vmcs.as_ref().unwrap();
                let axy = vmcs.read(fields::BYR_)?;
                let hmb = vmcs.read(fields::BYP_).bq();
                
                crate::lab_mode::trace_bus::npr(
                    self.ad, "EPT_VIOLATION", axy, dqq
                );
                
                
                super::isolation::pau(
                    self.ad,
                    axy,
                    hmb,
                    dqq,
                    wb,
                );
                
                
                super::api::eps(
                    super::api::VmEventType::Lj,
                    self.ad,
                    super::api::VmEventData::Bxs(axy),
                );
                
                Ok(false)
            }
            
            exit_reason::Cpd => {
                crate::lab_mode::trace_bus::ept(
                    self.ad, "VMCALL", wb,
                    &alloc::format!("func=0x{:X}", self.ej.rax)
                );
                
                let result = self.tln()?;
                let vmcs = self.vmcs.as_ref().unwrap();
                vmcs.write(fields::FG_, wb + hof)?;
                Ok(result)
            }
            
            exit_reason::CYQ_ => {
                crate::lab_mode::trace_bus::epu(self.ad, "TRIPLE FAULT (crashed)");
                crate::serial_println!("[VM {}] TRIPLE FAULT! Guest crashed.", self.ad);
                self.g = VmState::Gu;
                Ok(false)
            }
            
            exit_reason::Bbf => {
                
                
                let qao = self.ej.rcx as u32;
                let qap = (self.ej.rdx << 32) | (self.ej.rax & 0xFFFF_FFFF);
                
                if qao == 0 {
                    
                    let grf = qap | 1; 
                    unsafe {
                        core::arch::asm!(
                            "xsetbv",
                            in("ecx") 0u32,
                            in("edx") (grf >> 32) as u32,
                            in("eax") grf as u32,
                        );
                    }
                    crate::serial_println!("[VM {}] XSETBV XCR0=0x{:X}", self.ad, grf);
                } else {
                    crate::serial_println!("[VM {}] XSETBV ignored XCR{}=0x{:X}", self.ad, qao, qap);
                }
                
                let vmcs = self.vmcs.as_ref().unwrap();
                vmcs.write(fields::FG_, wb + hof)?;
                Ok(true)
            }
            
            exit_reason::CBZ_ => {
                crate::serial_println!("[VM {}] Invalid guest state! Check VMCS.", self.ad);
                self.g = VmState::Gu;
                Ok(false)
            }
            
            _ => {
                crate::serial_println!("[VM {}] Unhandled VM exit reason: {} at RIP=0x{:X}", 
                                      self.ad, exit_reason, wb);
                Ok(false)
            }
        }
    }
    
    
    fn lat(&mut self) -> Result<()> {
        let awa = self.ej.rax as u32;
        let bxj = self.ej.rcx as u32;
        
        
        match awa {
            0x4000_0000 => {
                
                self.ej.rax = 0x4000_0001;
                self.ej.rbx = 0x7473_7254; 
                self.ej.rcx = 0x7254_534F; 
                self.ej.rdx = 0x534F_7473; 
                return Ok(());
            }
            0x4000_0001 => {
                self.ej.rax = 0;
                self.ej.rbx = 0;
                self.ej.rcx = 0;
                self.ej.rdx = 0;
                return Ok(());
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
        
        let (mut eax, ebx, mut ecx, edx) = (htx, fpy, hty, htz);
        
        match awa {
            0x0000_0001 => {
                ecx &= !(1 << 5);   
                ecx |= 1 << 31;     
                ecx &= !(1 << 21);  
                ecx &= !(1 << 3);   
            }
            0x0000_000A => {
                eax = 0; ecx = 0; 
            }
            _ => {}
        }
        
        self.ej.rax = eax as u64;
        self.ej.rbx = ebx as u64;
        self.ej.rcx = ecx as u64;
        self.ej.rdx = edx as u64;
        
        Ok(())
    }
    
    
    fn lav(&mut self, dqq: u64) -> Result<()> {
        let port = ((dqq >> 16) & 0xFFFF) as u16;
        let tym = (dqq & 8) == 0;
        let yaf = (dqq & 16) != 0;
        let dds = (dqq & 7) as u8 + 1; 
        
        if tym {
            
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
                
                0x3F9..=0x3FF | 0x2F9..=0x2FF => {}
                
                0x20 | 0x21 | 0xA0 | 0xA1 => {} 
                0x40..=0x43 | 0x61 => {} 
                0x60 | 0x64 => {} 
                0x70 | 0x71 => {} 
                0x80..=0x8F => {} 
                0x00..=0x0F | 0xC0..=0xDF => {} 
                0x92 => {} 
                0xCF8 | 0xCFC..=0xCFF => {} 
                0xB000..=0xB03F => {} 
                0xE9 => {
                    let bm = (bn & 0xFF) as u8;
                    crate::serial_print!("{}", bm as char);
                }
                0xED => {} 
                _ => {
                    if self.cm.ank < 50 {
                        crate::serial_println!("[VM {}] OUT port 0x{:X} val=0x{:X}", self.ad, port, bn);
                    }
                }
            }
        } else {
            
            let bn: u32 = match port {
                
                0x3F8 => 0,         
                0x3F9 => 0,         
                0x3FA => 0xC1,      
                0x3FB => 0x03,      
                0x3FC => 0x03,      
                0x3FD => 0x60,      
                0x3FE => 0xB0,      
                0x3FF => 0,         
                
                0x2F8..=0x2FF => match port & 0x7 {
                    5 => 0x60,
                    _ => 0,
                },
                
                0x60 => 0,
                0x64 => 0x1C,       
                
                0x20 => 0,          
                0x21 => 0xFF,       
                0xA0 => 0,          
                0xA1 => 0xFF,       
                
                0x40..=0x42 => 0,
                0x43 => 0,
                0x61 => 0x20,       
                
                0x70 => 0,
                0x71 => 0,
                
                0x3B0..=0x3DF => 0,
                
                0x00..=0x0F | 0x80..=0x8F | 0xC0..=0xDF => 0,
                
                0xCF8 => 0,
                0xCFC..=0xCFF => 0xFFFF_FFFF,
                
                0xB008 => {
                    let qb = self.cm.gwg.hx(4);
                    (qb & 0xFFFF_FFFF) as u32
                }
                0xB000..=0xB03F => 0,
                
                0x92 => 0x02,
                
                0xE9 => 0,
                0xED => 0,
                _ => {
                    if self.cm.ank < 50 {
                        crate::serial_println!("[VM {}] IN port 0x{:X}", self.ad, port);
                    }
                    0xFF
                }
            };
            self.ej.rax = (self.ej.rax & !0xFFFF_FFFF) | (bn as u64);
        }
        
        Ok(())
    }
    
    
    fn laz(&mut self, rm: bool) -> Result<()> {
        let msr = self.ej.rcx as u32;
        
        
        const KK_: u32 = 0x001B;
        const NN_: u32 = 0x01A0;
        const KL_: u32 = 0x0277;
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
            let msy = (self.ej.rdx << 32) | (self.ej.rax & 0xFFFF_FFFF);
            
            
            match msr {
                VQ_ | VO_ | VK_ | VP_ |
                VL_ | VM_ | VN_ |
                KL_ | CN_ | KK_ | NN_ |
                OI_ => {}
                0x0174..=0x0176 => {} 
                0x0200..=0x020F => {} 
                0x0400..=0x047F => {} 
                _ => {
                    if self.cm.gwg < 100 {
                        crate::serial_println!("[VM {}] WRMSR 0x{:X} (ignored)", self.ad, msr);
                    }
                }
            }
        } else {
            
            let bn: u64 = match msr {
                KK_ => 0xFEE0_0900, 
                NN_ => 1,          
                KL_ => 0x0007040600070406, 
                CN_ => 0x501,             
                OI_ => 0,
                0x00FE => 0,   
                0x0179 => 0,   
                0x017A => 0,   
                0x02FF => 0x06, 
                0x0200..=0x020F => 0, 
                0x0400..=0x047F => 0, 
                _ => {
                    if self.cm.gwg < 100 {
                        crate::serial_println!("[VM {}] RDMSR 0x{:X} = 0", self.ad, msr);
                    }
                    0
                }
            };
            self.ej.rax = bn & 0xFFFF_FFFF;
            self.ej.rdx = bn >> 32;
        }
        
        Ok(())
    }
    
    
    fn tln(&mut self) -> Result<bool> {
        let gw = self.ej.rax;
        
        match gw {
            
            0 => {
                crate::serial_println!("[VM {}] Hypercall: print", self.ad);
                Ok(true)
            }
            
            
            1 => {
                let nz = self.ej.rbx;
                crate::serial_println!("[VM {}] Hypercall: exit (code={})", self.ad, nz);
                Ok(false)
            }
            
            
            2 => {
                let qb = crate::time::lc();
                self.ej.rax = qb;
                Ok(true)
            }
            
            
            3 => {
                let r = (self.ej.rbx & 0xFF) as u8;
                super::console::oac(self.ad, 0xE9, true, r);
                Ok(true)
            }
            
            
            4 => {
                let r = super::console::oac(self.ad, 0x3F8, false, 0);
                self.ej.rax = r as u64;
                Ok(true)
            }
            
            
            0x100..=0x1FF => {
                let mpi = (gw - 0x100) as u32;
                let n = [
                    self.ej.rbx,
                    self.ej.rcx,
                    self.ej.rdx,
                    self.ej.rsi,
                ];
                let (result, iia) = super::virtfs::lau(self.ad, mpi, &n);
                self.ej.rax = result as u64;
                Ok(true)
            }
            
            
            0x200..=0x3FF => {
                let n = [
                    self.ej.rbx,
                    self.ej.rcx,
                    self.ej.rdx,
                    self.ej.rsi,
                ];
                let (result, f) = super::api::tix(self.ad, gw, &n);
                
                
                if result == -1 && gw == super::api::hypercall::Uf {
                    
                    return Ok(false);
                }
                if result == -2 && gw == super::api::hypercall::Axh {
                    
                    return Ok(false);
                }
                
                self.ej.rax = f;
                Ok(true)
            }
            
            _ => {
                crate::serial_println!("[VM {}] Unknown hypercall: 0x{:X}", self.ad, gw);
                self.ej.rax = u64::O; 
                Ok(true)
            }
        }
    }
}


static Bai: Mutex<Vec<VirtualMachine>> = Mutex::new(Vec::new());


pub fn dpg(ad: u64, j: &str, afc: usize) -> Result<()> {
    let vm = VirtualMachine::new(ad, j, afc)?;
    Bai.lock().push(vm);
    Ok(())
}


pub fn poi(ad: u64) -> Result<()> {
    gte(ad, "hello")
}


pub fn gte(ad: u64, bzw: &str) -> Result<()> {
    let mut bfr = Bai.lock();
    
    for vm in bfr.el() {
        if vm.ad == ad {
            
            if bzw == "linux-test" || bzw.pp(".bzimage") {
                let uz = super::guests::iwr(bzw)
                    .unwrap_or_else(|| super::linux_loader::klw());
                crate::serial_println!("[VM {}] Loading Linux guest '{}' ({} bytes)", 
                                      ad, bzw, uz.len());
                vm.fvn(&uz, "console=ttyS0 earlyprintk=serial nokaslr", None)?;
                return Ok(());
            }
            
            
            let aj = super::guests::iwr(bzw)
                .unwrap_or_else(|| super::guests::obp());
            
            crate::serial_println!("[VM {}] Loading guest '{}' ({} bytes)", ad, bzw, aj.len());
            
            vm.diy(&aj, 0x1000)?;
            vm.ay(0x1000, 0x8000)?;
            return Ok(());
        }
    }
    
    Err(HypervisorError::Mo)
}


pub fn jru(ad: u64) -> Result<()> {
    let mut bfr = Bai.lock();
    
    for vm in bfr.el() {
        if vm.ad == ad {
            vm.g = VmState::Af;
            return Ok(());
        }
    }
    
    Err(HypervisorError::Mo)
}





use core::sync::atomic::{AtomicBool, AtomicU8, Ordering};


static mut YL_: GuestRegs = GuestRegs {
    rax: 0, rbx: 0, rcx: 0, rdx: 0,
    rsi: 0, rdi: 0, rbp: 0,
    r8: 0, r9: 0, r10: 0, r11: 0,
    r12: 0, r13: 0, r14: 0, r15: 0,
};



static mut AVW_: u64 = 0;


static BIO_: AtomicU8 = AtomicU8::new(0);













fn xso(xpn: bool) -> u64 {
    BIO_.store(xpn as u8, Ordering::SeqCst);
    unsafe { xsp() }
}



#[unsafe(evb)]
unsafe extern "C" fn xsp() -> u64 {
    core::arch::evc!(
        
        "push rbx",
        "push rbp",
        "push r12",
        "push r13",
        "push r14",
        "push r15",
        "pushfq",
        
        
        "lea rax, [{host_rsp}]",
        "mov [rax], rsp",
        
        
        "lea rax, [{flag}]",
        "movzx eax, byte ptr [rax]",
        "push rax",   
        
        
        "lea rcx, [{gregs}]",
        "mov rbx, [rcx + 8]",
        "mov rdx, [rcx + 24]",
        "mov rsi, [rcx + 32]",
        "mov rdi, [rcx + 40]",
        "mov rbp, [rcx + 48]",
        "mov r8,  [rcx + 56]",
        "mov r9,  [rcx + 64]",
        "mov r10, [rcx + 72]",
        "mov r11, [rcx + 80]",
        "mov r12, [rcx + 88]",
        "mov r13, [rcx + 96]",
        "mov r14, [rcx + 104]",
        "mov r15, [rcx + 112]",
        "mov rax, [rcx + 0]",    
        "mov rcx, [rcx + 16]",   
        
        
        
        
        
        
        
        
        
        
        
        "cmp qword ptr [rsp], 0",
        "jne 20f",
        
        
        "add rsp, 8",    
        "vmlaunch",
        "jmp 30f",       
        
        
        "20:",
        "add rsp, 8",    
        "vmresume",
        
        
        
        "30:",
        
        "lea rax, [{host_rsp}]",
        "mov rsp, [rax]",
        
        "popfq",
        "pop r15",
        "pop r14",
        "pop r13",
        "pop r12",
        "pop rbp",
        "pop rbx",
        "mov rax, 1",
        "ret",
        
        tpw = aaw AVW_,
        thn = aaw YL_,
        cqp = aaw BIO_,
    );
}











#[unsafe(evb)]
extern "C" fn pyn() {
    core::arch::evc!(
        
        
        "push rax",
        "lea rax, [{gregs}]",
        
        
        "mov [rax + 8],   rbx",
        "mov [rax + 16],  rcx",
        "mov [rax + 24],  rdx",
        "mov [rax + 32],  rsi",
        "mov [rax + 40],  rdi",
        "mov [rax + 48],  rbp",
        "mov [rax + 56],  r8",
        "mov [rax + 64],  r9",
        "mov [rax + 72],  r10",
        "mov [rax + 80],  r11",
        "mov [rax + 88],  r12",
        "mov [rax + 96],  r13",
        "mov [rax + 104], r14",
        "mov [rax + 112], r15",
        
        
        "pop rbx",
        "mov [rax], rbx",
        
        
        "lea rax, [{host_rsp}]",
        "mov rsp, [rax]",
        
        
        "popfq",
        "pop r15",
        "pop r14",
        "pop r13",
        "pop r12",
        "pop rbp",
        "pop rbx",
        
        
        "xor eax, eax",
        "ret",
        
        thn = aaw YL_,
        tpw = aaw AVW_,
    );
}
