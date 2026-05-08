







use alloc::string::String;
use alloc::vec::Vec;
use alloc::boxed::Box;
use alloc::format;
use alloc::collections::VecDeque;
use core::sync::atomic::{AtomicU64, Ordering};
use spin::Mutex;

use super::{HypervisorError, Result};
use super::svm::{self, SvmExitCode, SvmFeatures, Vv};
use super::svm::vmcb::{Vmcb, control_offsets, state_offsets, clean_bits};
use super::svm::npt::{Npt, flags as npt_flags};
use super::mmio::{self, Bt};
use super::ioapic::IoApicState;
use super::hpet::HpetState;
use super::pci::PciBus;
use super::virtio_blk::{VirtioBlkState, VirtioConsoleState};


static CLF_: AtomicU64 = AtomicU64::new(1);


#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SvmVmState {
    Created,
    Running,
    Paused,
    Stopped,
    Crashed,
}


#[derive(Debug, Clone, Default)]
pub struct SvmVmStats {
    pub vmexits: u64,
    pub cpuid_exits: u64,
    pub io_exits: u64,
    pub msr_exits: u64,
    pub hlt_exits: u64,
    pub npf_exits: u64,     
    pub vmmcall_exits: u64, 
    pub intr_exits: u64,    
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
    
    pub id: u64,
    
    pub name: String,
    
    pub state: SvmVmState,
    
    pub memory_size: usize,
    
    pub stats: SvmVmStats,
    
    pub(crate) vmcb: Option<Box<Vmcb>>,
    
    npt: Option<Npt>,
    
    guest_memory: Vec<u8>,
    
    pub(crate) guest_regs: SvmGuestRegs,
    
    pub asid: u32,
    
    console_id: Option<usize>,
    
    features: SvmFeatures,
    
    pub lapic: LapicState,
    
    pub pic: PicState,
    
    pub pit: PitState,
    
    pub pm_timer_start: u64,
    
    cmos_index: u8,
    
    pub ioapic: IoApicState,
    
    pub hpet: HpetState,
    
    pub pci: PciBus,
    
    pub pit_last_inject: u64,
    
    pub serial_input_buffer: VecDeque<u8>,
    
    pub serial_ier: u8,
    
    pub serial_fcr: u8,
    
    pub virtio_blk_storage: Vec<u8>,
    
    pub virtio_blk_status: u8,
    
    pub virtio_console_status: u8,
    
    pub virtio_blk_state: VirtioBlkState,
    
    pub virtio_console_state: VirtioConsoleState,
}


#[derive(Debug, Clone)]
pub struct LapicState {
    
    pub icr: u32,
    
    pub ccr: u32,
    
    pub dcr: u32,
    
    pub timer_lvt: u32,
    
    pub svr: u32,
    
    pub tpr: u32,
    
    pub enabled: bool,
    
    pub last_tick_exit: u64,
}

impl Default for LapicState {
    fn default() -> Self {
        Self {
            icr: 0,
            ccr: 0,
            dcr: 0,
            timer_lvt: 0x0001_0000, 
            svr: 0x1FF,             
            tpr: 0,
            enabled: false,
            last_tick_exit: 0,
        }
    }
}


#[derive(Debug, Clone)]
pub struct PicState {
    
    pub master_icw_phase: u8,
    
    pub slave_icw_phase: u8,
    
    pub master_imr: u8,
    
    pub slave_imr: u8,
    
    pub master_vector_base: u8,
    
    pub slave_vector_base: u8,
    
    pub master_isr: u8,
    
    pub master_irr: u8,
    
    pub initialized: bool,
}

impl Default for PicState {
    fn default() -> Self {
        Self {
            master_icw_phase: 0,
            slave_icw_phase: 0,
            master_imr: 0xFF, 
            slave_imr: 0xFF,
            master_vector_base: 0x08, 
            slave_vector_base: 0x70,
            master_isr: 0,
            master_irr: 0,
            initialized: false,
        }
    }
}


#[derive(Debug, Clone)]
pub struct PitChannel {
    
    pub reload: u16,
    
    pub count: u16,
    
    pub mode: u8,
    
    pub access: u8,
    
    pub latched: bool,
    pub latch_value: u16,
    
    pub write_hi_pending: bool,
    
    pub output: bool,
}

impl Default for PitChannel {
    fn default() -> Self {
        Self {
            reload: 0xFFFF,
            count: 0xFFFF,
            mode: 0,
            access: 3, 
            latched: false,
            latch_value: 0,
            write_hi_pending: false,
            output: false,
        }
    }
}


#[derive(Debug, Clone)]
pub struct PitState {
    pub channels: [PitChannel; 3],
}

impl Default for PitState {
    fn default() -> Self {
        Self {
            channels: [
                PitChannel::default(),
                PitChannel::default(),
                PitChannel::default(),
            ],
        }
    }
}

impl SvmVirtualMachine {
    
    pub fn new(name: &str, memory_mb: usize) -> Result<Self> {
        
        if !svm::is_supported() {
            return Err(HypervisorError::SvmNotSupported);
        }
        
        let id = CLF_.fetch_add(1, Ordering::SeqCst);
        let memory_size = memory_mb * 1024 * 1024;
        let features = svm::ckb();
        
        
        let guest_memory = alloc::vec![0u8; memory_size];
        
        
        let asid = super::svm::npt::hev().unwrap_or(1);
        
        
        let console_id = super::console::create_console(id, name);
        
        
        super::virtfs::hot(id);
        
        crate::serial_println!("[SVM-VM {}] Created '{}' with {} MB RAM, ASID={}", 
                              id, name, memory_mb, asid);
        
        
        crate::lab_mode::trace_bus::bzh(
            id, &format!("CREATED '{}' mem={}MB ASID={}", name, memory_mb, asid)
        );
        
        
        super::api::bzf(
            super::api::VmEventType::Created,
            id,
            super::api::VmEventData::Az(format!("SVM VM '{}' created", name)),
        );
        
        Ok(SvmVirtualMachine {
            id,
            name: String::from(name),
            state: SvmVmState::Created,
            memory_size,
            stats: SvmVmStats::default(),
            vmcb: None,
            npt: None,
            guest_memory,
            guest_regs: SvmGuestRegs::default(),
            asid,
            console_id: Some(console_id),
            features,
            lapic: LapicState::default(),
            pic: PicState::default(),
            pit: PitState::default(),
            pm_timer_start: 0,
            cmos_index: 0,
            ioapic: IoApicState::default(),
            hpet: HpetState::default(),
            pci: PciBus::default(),
            pit_last_inject: 0,
            serial_input_buffer: VecDeque::with_capacity(256),
            serial_ier: 0,
            serial_fcr: 0,
            virtio_blk_storage: alloc::vec![0u8; 64 * 512], 
            virtio_blk_status: 0,
            virtio_console_status: 0,
            virtio_blk_state: VirtioBlkState::with_capacity(64 * 512),
            virtio_console_state: VirtioConsoleState::default(),
        })
    }
    
    
    pub fn initialize(&mut self) -> Result<()> {
        crate::serial_println!("[SVM-VM {}] Initializing VMCB and NPT...", self.id);
        
        
        let mut vmcb = Vmcb::new();
        
        
        vmcb.setup_basic_intercepts();
        
        
        vmcb.write_control(control_offsets::AWB_, self.asid as u64);
        
        
        
        let ihm = alloc::vec![0xFFu8; 12288]; 
        let mrr = ihm.as_ptr() as u64;
        let ihn = mrr - crate::memory::hhdm_offset();
        vmcb.set_iopm_base(ihn);
        
        core::mem::forget(ihm);
        crate::serial_println!("[SVM-VM {}] IOPM allocated at HPA=0x{:X}", self.id, ihn);
        
        
        
        let iox = alloc::vec![0xFFu8; 8192]; 
        let ngv = iox.as_ptr() as u64;
        let ioy = ngv - crate::memory::hhdm_offset();
        vmcb.set_msrpm_base(ioy);
        core::mem::forget(iox);
        crate::serial_println!("[SVM-VM {}] MSRPM allocated at HPA=0x{:X}", self.id, ioy);
        
        
        if self.features.npt {
            let mut npt = Npt::new(self.asid);
            
            
            let mgj = self.guest_memory.as_ptr() as u64;
            let eop = mgj - crate::memory::hhdm_offset();
            
            if let Err(e) = npt.map_range(
                0,                          
                eop,             
                self.memory_size as u64,    
                npt_flags::Uq,             
            ) {
                crate::serial_println!("[SVM-VM {}] NPT mapping failed: {}", self.id, e);
                return Err(HypervisorError::NptViolation);
            }
            
            
            let irb = npt.cr3();
            
            
            
            vmcb.write_control(control_offsets::BDX_, irb);
            
            
            let mut ira = vmcb.read_control(control_offsets::XD_);
            ira |= 1; 
            vmcb.write_control(control_offsets::XD_, ira);
            
            crate::serial_println!("[SVM-VM {}] NPT enabled, N_CR3=0x{:X}", self.id, irb);
            
            self.npt = Some(npt);
        } else {
            
            crate::serial_println!("[SVM-VM {}] NPT not available, using shadow paging", self.id);
        }
        
        
        vmcb.write_control(control_offsets::BJK_, 0); 
        
        self.vmcb = Some(Box::new(vmcb));
        
        crate::serial_println!("[SVM-VM {}] Initialization complete", self.id);
        
        Ok(())
    }
    
    
    pub fn load_binary(&mut self, data: &[u8], load_address: u64) -> Result<()> {
        let offset = load_address as usize;
        
        if offset + data.len() > self.guest_memory.len() {
            return Err(HypervisorError::OutOfMemory);
        }
        
        self.guest_memory[offset..offset + data.len()].copy_from_slice(data);
        
        crate::serial_println!("[SVM-VM {}] Loaded {} bytes at GPA 0x{:X}", 
                              self.id, data.len(), load_address);
        
        Ok(())
    }
    
    
    pub fn setup_real_mode(&mut self, entry_point: u64) -> Result<()> {
        let vmcb = self.vmcb.as_mut().ok_or(HypervisorError::VmcbNotLoaded)?;
        vmcb.setup_real_mode();
        
        
        let fpe = (entry_point >> 4) << 4;
        let ip = entry_point & 0xF;
        
        vmcb.write_state(state_offsets::TP_, fpe);
        vmcb.write_state(state_offsets::KO_, (fpe >> 4) as u64);
        vmcb.write_state(state_offsets::Af, ip);
        
        crate::serial_println!("[SVM-VM {}] Real mode: CS=0x{:X}, IP=0x{:X}", 
                              self.id, fpe >> 4, ip);
        
        Ok(())
    }
    
    
    
    
    pub fn qlk(&mut self, data: &[u8]) {
        for &b in data {
            self.serial_input_buffer.push_back(b);
        }
    }
    
    
    pub fn setup_protected_mode(&mut self, entry_point: u64, stack_ptr: u64) -> Result<()> {
        let vmcb = self.vmcb.as_mut().ok_or(HypervisorError::VmcbNotLoaded)?;
        vmcb.setup_protected_mode(entry_point);
        
        vmcb.write_state(state_offsets::Af, entry_point);
        vmcb.write_state(state_offsets::De, stack_ptr);
        
        crate::serial_println!("[SVM-VM {}] Protected mode: RIP=0x{:X}, RSP=0x{:X}", 
                              self.id, entry_point, stack_ptr);
        
        Ok(())
    }
    
    
    pub fn setup_protected_mode_for_linux(&mut self, entry_point: u64, stack_ptr: u64, boot_params_addr: u64) -> Result<()> {
        let vmcb = self.vmcb.as_mut().ok_or(HypervisorError::VmcbNotLoaded)?;
        vmcb.setup_protected_mode(entry_point);
        
        vmcb.write_state(state_offsets::Af, entry_point);
        vmcb.write_state(state_offsets::De, stack_ptr);
        
        
        
        
        self.guest_regs.rsi = boot_params_addr;
        
        
        self.guest_regs.rbp = 0;
        self.guest_regs.rdi = 0;
        
        crate::serial_println!("[SVM-VM {}] Linux protected mode: RIP=0x{:X}, RSP=0x{:X}, RSI(boot_params)=0x{:X}", 
                              self.id, entry_point, stack_ptr, boot_params_addr);
        
        Ok(())
    }

    
    pub fn setup_long_mode(&mut self, entry_point: u64, stack_ptr: u64, guest_cr3: u64) -> Result<()> {
        let vmcb = self.vmcb.as_mut().ok_or(HypervisorError::VmcbNotLoaded)?;
        vmcb.setup_long_mode(entry_point, guest_cr3);
        
        vmcb.write_state(state_offsets::Af, entry_point);
        vmcb.write_state(state_offsets::De, stack_ptr);
        
        crate::serial_println!("[SVM-VM {}] Long mode: RIP=0x{:X}, RSP=0x{:X}, CR3=0x{:X}", 
                              self.id, entry_point, stack_ptr, guest_cr3);
        
        Ok(())
    }
    
    
    pub fn start_linux(
        &mut self,
        bas: &[u8],
        cmdline: &str,
        initrd: Option<&[u8]>,
    ) -> Result<()> {
        use super::linux_loader;
        
        
        if self.vmcb.is_none() {
            self.initialize()?;
        }
        
        crate::serial_println!("[SVM-VM {}] Loading Linux kernel ({} bytes)...", self.id, bas.len());
        
        
        let ny = linux_loader::itr(bas)
            .map_err(|e| {
                crate::serial_println!("[SVM-VM {}] bzImage parse error: {:?}", self.id, e);
                HypervisorError::InvalidGuest
            })?;
        
        crate::serial_println!("[SVM-VM {}] Kernel: protocol={}.{}, 64-bit={}, entry=0x{:X}",
            self.id, ny.header.version >> 8, ny.header.version & 0xFF,
            ny.supports_64bit, ny.entry_64);
        
        
        let config = linux_loader::Ml {
            cmdline: alloc::string::String::from(cmdline),
            memory_size: self.memory_size as u64,
            initrd: initrd.map(|d| d.to_vec()),
        };
        
        let pk = linux_loader::iku(&mut self.guest_memory, &ny, &config)
            .map_err(|e| {
                crate::serial_println!("[SVM-VM {}] Linux load error: {:?}", self.id, e);
                HypervisorError::InvalidGuest
            })?;
        
        crate::serial_println!("[SVM-VM {}] Linux loaded: entry=0x{:X}, stack=0x{:X}, cr3=0x{:X}, gdt=0x{:X}",
            self.id, pk.entry_point, pk.stack_ptr, pk.cr3, pk.gdt_base);
        
        
        {
            let vmcb = self.vmcb.as_mut().ok_or(HypervisorError::VmcbNotLoaded)?;
            vmcb.setup_long_mode_for_linux(
                pk.entry_point,
                pk.stack_ptr,
                pk.cr3,
                pk.gdt_base,
                39, 
            );
        }
        
        
        self.guest_regs.rsi = pk.boot_params_addr;
        self.guest_regs.rbp = 0;
        self.guest_regs.rdi = 0;
        
        crate::serial_println!("[SVM-VM {}] Starting Linux with RSI=0x{:X} (boot_params)", 
            self.id, pk.boot_params_addr);
        
        
        self.state = SvmVmState::Running;
        crate::lab_mode::trace_bus::bzh(self.id, "LINUX_STARTED");
        self.run_loop()?;
        
        Ok(())
    }
    
    
    pub fn start(&mut self) -> Result<()> {
        if self.vmcb.is_none() {
            self.initialize()?;
        }
        
        self.state = SvmVmState::Running;
        
        crate::serial_println!("[SVM-VM {}] Starting execution...", self.id);
        crate::lab_mode::trace_bus::bzh(self.id, "STARTED");
        
        
        self.run_loop()?;
        
        Ok(())
    }
    
    
    fn run_loop(&mut self) -> Result<()> {
        
        let vmcb_phys = {
            let vmcb = self.vmcb.as_ref().ok_or(HypervisorError::VmcbNotLoaded)?;
            vmcb.phys_addr()
        };
        
        
        let mut vc = Vv {
            rax: self.guest_regs.rax,
            rbx: self.guest_regs.rbx,
            rcx: self.guest_regs.rcx,
            rdx: self.guest_regs.rdx,
            rsi: self.guest_regs.rsi,
            rdi: self.guest_regs.rdi,
            rbp: self.guest_regs.rbp,
            r8: self.guest_regs.r8,
            r9: self.guest_regs.r9,
            r10: self.guest_regs.r10,
            r11: self.guest_regs.r11,
            r12: self.guest_regs.r12,
            r13: self.guest_regs.r13,
            r14: self.guest_regs.r14,
            r15: self.guest_regs.r15,
        };
        
        crate::serial_println!("[SVM-VM {}] Entering VM loop, RSI=0x{:X}", self.id, vc.rsi);
        
        loop {
            if self.state != SvmVmState::Running {
                break;
            }
            
            
            
            let mut eqo = false;
            
            
            {
                let vmcb = self.vmcb.as_mut().ok_or(HypervisorError::VmcbNotLoaded)?;
                
                vmcb.write_state(state_offsets::De, self.guest_regs.rsp);
                
                
                
                
                
                let blc = clean_bits::Alp      
                          | clean_bits::Agx      
                          | clean_bits::Amw        
                          | clean_bits::Ame       
                          | clean_bits::Agz;     
                vmcb.set_clean_bits(blc);
                
                
                if self.lapic.enabled && self.lapic.icr > 0 {
                    let masked = (self.lapic.timer_lvt >> 16) & 1;
                    if masked == 0 {
                        let gyw = (self.lapic.timer_lvt >> 17) & 0x3;
                        let vector = (self.lapic.timer_lvt & 0xFF) as u64;
                        let cws = match self.lapic.dcr & 0xB {
                            0x0 => 2u64, 0x1 => 4, 0x2 => 8, 0x3 => 16,
                            0x8 => 32, 0x9 => 64, 0xA => 128, 0xB => 1,
                            _ => 1,
                        };
                        let bb = self.stats.vmexits.saturating_sub(self.lapic.last_tick_exit);
                        let gx = (bb * 256) / cws;
                        
                        if gx >= self.lapic.icr as u64 {
                            let rflags = vmcb.read_state(state_offsets::Ek);
                            if (rflags & 0x200) != 0 && vector > 0 {
                                let bsi: u64 = vector
                                                   | (0u64 << 8)
                                                   | (1u64 << 31);
                                vmcb.write_control(control_offsets::HA_, bsi);
                                eqo = true;
                            }
                            match gyw {
                                1 => {
                                    self.lapic.last_tick_exit = self.stats.vmexits;
                                    self.lapic.ccr = self.lapic.icr;
                                }
                                _ => {
                                    self.lapic.icr = 0;
                                    self.lapic.ccr = 0;
                                }
                            }
                        }
                    }
                }
                
                
                
                
                
                if !eqo {
                    let nuw = if self.pit.channels[0].reload > 0 {
                        
                        
                        let reload = self.pit.channels[0].reload as u64;
                        
                        (reload / 24).max(100).min(2000)
                    } else {
                        500 
                    };
                    
                    let ote = self.stats.vmexits.saturating_sub(self.pit_last_inject);
                    if ote >= nuw {
                        let rflags = vmcb.read_state(state_offsets::Ek);
                        if (rflags & 0x200) != 0 {
                            
                            let vector = if let Some(afo) = self.ioapic.get_irq_route(0) {
                                if !afo.masked && afo.vector > 0 {
                                    afo.vector as u64
                                } else {
                                    
                                    self.pic.master_vector_base as u64
                                }
                            } else {
                                self.pic.master_vector_base as u64
                            };
                            
                            if vector > 0 {
                                let bsi: u64 = vector
                                                   | (0u64 << 8)
                                                   | (1u64 << 31);
                                vmcb.write_control(control_offsets::HA_, bsi);
                                eqo = true;
                            }
                        }
                        self.pit_last_inject = self.stats.vmexits;
                    }
                }
                
            } 
                
            
            if !eqo {
                
                let mmn = self.check_hpet_interrupts();
                if let Some(vector) = mmn {
                    let vmcb = self.vmcb.as_mut().unwrap();
                    let fpo = vmcb.read_control(control_offsets::HA_);
                    let fgw = (fpo & (1u64 << 31)) != 0;
                    if !fgw {
                        let rflags = vmcb.read_state(state_offsets::Ek);
                        if (rflags & 0x200) != 0 { 
                            let bsi: u64 = vector
                                               | (0u64 << 8)    
                                               | (1u64 << 31);  
                            vmcb.write_control(control_offsets::HA_, bsi);
                        }
                    }
                }
            }
            
            
            if (self.serial_ier & 0x01) != 0 && !self.serial_input_buffer.is_empty() {
                let vector = if let Some(afo) = self.ioapic.get_irq_route(4) {
                    if !afo.masked && afo.vector > 0 {
                        afo.vector as u64
                    } else {
                        (self.pic.master_vector_base + 4) as u64
                    }
                } else {
                    (self.pic.master_vector_base + 4) as u64
                };
                
                if vector > 0 {
                    let vmcb = self.vmcb.as_mut().unwrap();
                    let fpo = vmcb.read_control(control_offsets::HA_);
                    let fgw = (fpo & (1u64 << 31)) != 0;
                    if !fgw {
                        let rflags = vmcb.read_state(state_offsets::Ek);
                        if (rflags & 0x200) != 0 {
                            let bsi: u64 = vector
                                               | (0u64 << 8)
                                               | (1u64 << 31);
                            vmcb.write_control(control_offsets::HA_, bsi);
                        }
                    }
                }
            }
            
            
            
            unsafe { svm::kky(); }
            
            
            unsafe {
                svm::psu(vmcb_phys, &mut vc);
            }
            
            
            unsafe { svm::oxh(); }
            
            
            self.guest_regs.rbx = vc.rbx;
            self.guest_regs.rcx = vc.rcx;
            self.guest_regs.rdx = vc.rdx;
            self.guest_regs.rsi = vc.rsi;
            self.guest_regs.rdi = vc.rdi;
            self.guest_regs.rbp = vc.rbp;
            self.guest_regs.r8 = vc.r8;
            self.guest_regs.r9 = vc.r9;
            self.guest_regs.r10 = vc.r10;
            self.guest_regs.r11 = vc.r11;
            self.guest_regs.r12 = vc.r12;
            self.guest_regs.r13 = vc.r13;
            self.guest_regs.r14 = vc.r14;
            self.guest_regs.r15 = vc.r15;
            
            
            {
                let vmcb = self.vmcb.as_ref().ok_or(HypervisorError::VmcbNotLoaded)?;
                self.guest_regs.rax = vmcb.read_state(state_offsets::Fa);
                self.guest_regs.rsp = vmcb.read_state(state_offsets::De);
            }
            
            self.stats.vmexits += 1;
            
            
            let eix = self.handle_vmexit_inline()?;
            
            
            if self.stats.vmexits % 50 == 0 || !eix {
                let rip = {
                    let vmcb = self.vmcb.as_ref().ok_or(HypervisorError::VmcbNotLoaded)?;
                    vmcb.read_state(state_offsets::Af)
                };
                crate::lab_mode::trace_bus::hvm(
                    self.id,
                    self.guest_regs.rax,
                    self.guest_regs.rbx,
                    self.guest_regs.rcx,
                    self.guest_regs.rdx,
                    rip,
                    self.guest_regs.rsp,
                );
            }
            
            if !eix {
                break;
            }
            
            
            vc.rax = self.guest_regs.rax;
            vc.rbx = self.guest_regs.rbx;
            vc.rcx = self.guest_regs.rcx;
            vc.rdx = self.guest_regs.rdx;
            vc.rsi = self.guest_regs.rsi;
            vc.rdi = self.guest_regs.rdi;
            vc.rbp = self.guest_regs.rbp;
            vc.r8 = self.guest_regs.r8;
            vc.r9 = self.guest_regs.r9;
            vc.r10 = self.guest_regs.r10;
            vc.r11 = self.guest_regs.r11;
            vc.r12 = self.guest_regs.r12;
            vc.r13 = self.guest_regs.r13;
            vc.r14 = self.guest_regs.r14;
            vc.r15 = self.guest_regs.r15;
        }
        
        if self.state == SvmVmState::Running {
            self.state = SvmVmState::Stopped;
        }
        
        crate::serial_println!("[SVM-VM {}] Stopped after {} VMEXITs", self.id, self.stats.vmexits);
        crate::lab_mode::trace_bus::bzh(
            self.id, &format!("STOPPED after {} exits (cpuid={} io={} msr={} hlt={} vmcall={})",
                self.stats.vmexits, self.stats.cpuid_exits, self.stats.io_exits,
                self.stats.msr_exits, self.stats.hlt_exits, self.stats.vmmcall_exits)
        );
        
        Ok(())
    }
    
    
    fn handle_vmexit_inline(&mut self) -> Result<bool> {
        
        let (exit_code, bmb, fvp, guest_rip, vo) = {
            let vmcb = self.vmcb.as_ref().ok_or(HypervisorError::VmcbNotLoaded)?;
            let ec = vmcb.read_control(control_offsets::Lv);
            let lon = vmcb.read_control(control_offsets::Lx);
            let loo = vmcb.read_control(control_offsets::Ly);
            let rip = vmcb.read_state(state_offsets::Af);
            let dvj = if self.features.nrip_save {
                vmcb.read_control(control_offsets::AHS_)
            } else {
                rip + 2 
            };
            (ec, lon, loo, rip, dvj)
        };
        
        let exit = SvmExitCode::from(exit_code);
        
        match exit {
            SvmExitCode::Cpuid => {
                self.stats.cpuid_exits += 1;
                
                crate::lab_mode::trace_bus::bzg(
                    self.id, "CPUID", guest_rip,
                    &alloc::format!("EAX=0x{:X} ECX=0x{:X}", self.guest_regs.rax, self.guest_regs.rcx)
                );
                
                super::debug_monitor::akj(
                    self.id, super::debug_monitor::DebugCategory::CpuidLeaf,
                    self.guest_regs.rax, super::debug_monitor::HandleStatus::Handled,
                    guest_rip, self.stats.vmexits,
                    &alloc::format!("ECX=0x{:X}", self.guest_regs.rcx),
                );
                self.handle_cpuid();
                let vmcb = self.vmcb.as_mut().ok_or(HypervisorError::VmcbNotLoaded)?;
                vmcb.write_state(state_offsets::Af, vo);
                Ok(true)
            }
            
            SvmExitCode::Hlt => {
                self.stats.hlt_exits += 1;
                
                
                {
                    let vmcb = self.vmcb.as_mut().ok_or(HypervisorError::VmcbNotLoaded)?;
                    vmcb.write_state(state_offsets::Af, vo);
                    
                    
                    let rflags = vmcb.read_state(state_offsets::Ek);
                    if (rflags & 0x200) != 0 { 
                        
                        let vector = if self.lapic.enabled && (self.lapic.timer_lvt & 0xFF) > 0 
                            && ((self.lapic.timer_lvt >> 16) & 1) == 0 {
                            (self.lapic.timer_lvt & 0xFF) as u64
                        } else {
                            0x20
                        };
                        let bsi: u64 = vector
                                           | (0u64 << 8)    
                                           | (1u64 << 31);  
                        vmcb.write_control(control_offsets::HA_, bsi);
                    }
                }
                
                
                
                if self.stats.hlt_exits > 5_000_000 {
                    crate::serial_println!("[SVM-VM {}] Too many HLT exits ({}), stopping", self.id, self.stats.hlt_exits);
                    self.state = SvmVmState::Stopped;
                    Ok(false)
                } else {
                    
                    if self.stats.hlt_exits % 10000 == 0 {
                        crate::serial_println!("[SVM-VM {}] HLT count: {}", self.id, self.stats.hlt_exits);
                    }
                    Ok(true)
                }
            }
            
            SvmExitCode::IoioIn | SvmExitCode::IoioOut => {
                self.stats.io_exits += 1;
                let port = ((bmb >> 16) & 0xFFFF) as u16;
                let it = if matches!(exit, SvmExitCode::IoioIn) { "IN" } else { "OUT" };
                crate::lab_mode::trace_bus::hvk(self.id, it, port, self.guest_regs.rax);
                self.handle_io(bmb);
                let vmcb = self.vmcb.as_mut().ok_or(HypervisorError::VmcbNotLoaded)?;
                vmcb.write_state(state_offsets::Af, vo);
                Ok(true)
            }
            
            SvmExitCode::MsrRead | SvmExitCode::MsrWrite => {
                self.stats.msr_exits += 1;
                let is_write = matches!(exit, SvmExitCode::MsrWrite);
                let ngs = if is_write { "WRMSR" } else { "RDMSR" };
                crate::lab_mode::trace_bus::bzg(
                    self.id, ngs, guest_rip,
                    &alloc::format!("MSR=0x{:X}", self.guest_regs.rcx)
                );
                self.handle_msr(is_write);
                let vmcb = self.vmcb.as_mut().ok_or(HypervisorError::VmcbNotLoaded)?;
                vmcb.write_state(state_offsets::Af, vo);
                Ok(true)
            }
            
            SvmExitCode::NpfFault => {
                self.stats.npf_exits += 1;
                let zy = fvp;
                let error_code = bmb;
                
                
                let psq = self.vmcb.as_ref().ok_or(HypervisorError::VmcbNotLoaded)?;
                let (fetched, insn_bytes) = psq.guest_insn_bytes();
                
                
                let uu = mmio::awu(&insn_bytes, fetched, true);
                
                
                let miv = self.handle_npf(zy, error_code, guest_rip, uu.as_ref());
                
                if miv {
                    
                    if let Some(ref ox) = uu {
                        let vmcb = self.vmcb.as_mut().ok_or(HypervisorError::VmcbNotLoaded)?;
                        vmcb.write_state(state_offsets::Af, guest_rip + ox.insn_len as u64);
                    } else if self.features.nrip_save {
                        
                        let vmcb = self.vmcb.as_ref().ok_or(HypervisorError::VmcbNotLoaded)?;
                        let dvj = vmcb.read_control(control_offsets::AHS_);
                        if dvj > guest_rip && dvj < guest_rip + 16 {
                            let vmcb = self.vmcb.as_mut().ok_or(HypervisorError::VmcbNotLoaded)?;
                            vmcb.write_state(state_offsets::Af, dvj);
                        } else {
                            
                            crate::serial_println!("[SVM-VM {}] NPF: decode failed, skipping 3 bytes at RIP=0x{:X}", 
                                self.id, guest_rip);
                            let vmcb = self.vmcb.as_mut().ok_or(HypervisorError::VmcbNotLoaded)?;
                            vmcb.write_state(state_offsets::Af, guest_rip + 3);
                        }
                    } else {
                        
                        crate::serial_println!("[SVM-VM {}] NPF: no decode/nrip, skipping 3 bytes", self.id);
                        let vmcb = self.vmcb.as_mut().ok_or(HypervisorError::VmcbNotLoaded)?;
                        vmcb.write_state(state_offsets::Af, guest_rip + 3);
                    }
                    Ok(true)
                } else {
                    crate::lab_mode::trace_bus::hvl(
                        self.id, "NPF_VIOLATION", zy, error_code
                    );
                    super::debug_monitor::akj(
                        self.id, super::debug_monitor::DebugCategory::NpfFault,
                        zy, super::debug_monitor::HandleStatus::Fatal,
                        guest_rip, self.stats.vmexits,
                        &alloc::format!("err=0x{:X}", error_code),
                    );
                    crate::serial_println!("[SVM-VM {}] FATAL NPF: GPA=0x{:X}, Error=0x{:X}, RIP=0x{:X}", 
                                          self.id, zy, error_code, guest_rip);
                    
                    super::isolation::iyv(
                        self.id,
                        zy,
                        None,
                        error_code,
                        guest_rip,
                    );
                    
                    self.state = SvmVmState::Crashed;
                    Ok(false)
                }
            }
            
            SvmExitCode::Vmmcall => {
                self.stats.vmmcall_exits += 1;
                crate::lab_mode::trace_bus::bzg(
                    self.id, "VMMCALL", guest_rip,
                    &alloc::format!("func=0x{:X} args=({:X},{:X},{:X})",
                        self.guest_regs.rax, self.guest_regs.rbx,
                        self.guest_regs.rcx, self.guest_regs.rdx)
                );
                let gux = self.handle_vmmcall();
                let vmcb = self.vmcb.as_mut().ok_or(HypervisorError::VmcbNotLoaded)?;
                vmcb.write_state(state_offsets::Af, vo);
                Ok(gux)
            }
            
            SvmExitCode::Shutdown => {
                crate::lab_mode::trace_bus::bzh(self.id, "TRIPLE FAULT (shutdown)");
                super::debug_monitor::akj(
                    self.id, super::debug_monitor::DebugCategory::Exception,
                    0xFF, super::debug_monitor::HandleStatus::Fatal,
                    guest_rip, self.stats.vmexits, "TRIPLE FAULT",
                );
                crate::serial_println!("[SVM-VM {}] Guest SHUTDOWN (triple fault)", self.id);
                self.state = SvmVmState::Crashed;
                Ok(false)
            }
            
            SvmExitCode::Intr => {
                self.stats.intr_exits += 1;
                
                Ok(true)
            }
            
            
            SvmExitCode::WriteCr0 => {
                
                
                
                let ips = bmb;
                crate::lab_mode::trace_bus::bzg(
                    self.id, "WRITE_CR0", guest_rip,
                    &alloc::format!("val=0x{:X}", ips)
                );
                {
                    let vmcb = self.vmcb.as_mut().ok_or(HypervisorError::VmcbNotLoaded)?;
                    
                    
                    let oju = ips | 0x10; 
                    vmcb.set_cr0(oju);
                    vmcb.write_state(state_offsets::Af, vo);
                }
                Ok(true)
            }
            
            SvmExitCode::WriteCr3 => {
                
                let niv = bmb;
                {
                    let vmcb = self.vmcb.as_mut().ok_or(HypervisorError::VmcbNotLoaded)?;
                    vmcb.set_cr3(niv);
                    vmcb.write_state(state_offsets::Af, vo);
                }
                Ok(true)
            }
            
            SvmExitCode::WriteCr4 => {
                
                let ipt = bmb;
                crate::lab_mode::trace_bus::bzg(
                    self.id, "WRITE_CR4", guest_rip,
                    &alloc::format!("val=0x{:X}", ipt)
                );
                {
                    let vmcb = self.vmcb.as_mut().ok_or(HypervisorError::VmcbNotLoaded)?;
                    vmcb.set_cr4(ipt);
                    vmcb.write_state(state_offsets::Af, vo);
                }
                Ok(true)
            }
            
            
            SvmExitCode::ReadCr0 | SvmExitCode::ReadCr3 | SvmExitCode::ReadCr4 => {
                
                
                let vmcb = self.vmcb.as_mut().ok_or(HypervisorError::VmcbNotLoaded)?;
                vmcb.write_state(state_offsets::Af, vo);
                Ok(true)
            }
            
            SvmExitCode::Cr0SelWrite => {
                
                let vmcb = self.vmcb.as_mut().ok_or(HypervisorError::VmcbNotLoaded)?;
                vmcb.write_state(state_offsets::Af, vo);
                Ok(true)
            }
            
            
            
            SvmExitCode::Xsetbv => {
                
                
                let pvs = self.guest_regs.rcx as u32;
                let value = (self.guest_regs.rdx << 32) | (self.guest_regs.rax & 0xFFFF_FFFF);
                if pvs == 0 {
                    
                    
                    let safe_value = value | 1; 
                    unsafe {
                        core::arch::asm!(
                            "xsetbv",
                            in("ecx") 0u32,
                            in("edx") (safe_value >> 32) as u32,
                            in("eax") safe_value as u32,
                        );
                    }
                }
                let vmcb = self.vmcb.as_mut().ok_or(HypervisorError::VmcbNotLoaded)?;
                vmcb.write_state(state_offsets::Af, vo);
                Ok(true)
            }
            
            SvmExitCode::Invd => {
                
                
                let vmcb = self.vmcb.as_mut().ok_or(HypervisorError::VmcbNotLoaded)?;
                vmcb.write_state(state_offsets::Af, vo);
                Ok(true)
            }
            
            SvmExitCode::Invlpg => {
                
                
                
                let vmcb = self.vmcb.as_mut().ok_or(HypervisorError::VmcbNotLoaded)?;
                vmcb.write_state(state_offsets::Af, vo);
                Ok(true)
            }
            
            SvmExitCode::Wbinvd => {
                
                
                unsafe { core::arch::asm!("wbinvd", options(nomem, nostack)); }
                let vmcb = self.vmcb.as_mut().ok_or(HypervisorError::VmcbNotLoaded)?;
                vmcb.write_state(state_offsets::Af, vo);
                Ok(true)
            }
            
            SvmExitCode::Rdtsc => {
                
                let tsc = unsafe { core::arch::x86_64::_rdtsc() };
                self.guest_regs.rax = tsc & 0xFFFF_FFFF;
                self.guest_regs.rdx = tsc >> 32;
                let vmcb = self.vmcb.as_mut().ok_or(HypervisorError::VmcbNotLoaded)?;
                vmcb.write_state(state_offsets::Af, vo);
                Ok(true)
            }
            
            SvmExitCode::Rdtscp => {
                
                let tsc = unsafe { core::arch::x86_64::_rdtsc() };
                self.guest_regs.rax = tsc & 0xFFFF_FFFF;
                self.guest_regs.rdx = tsc >> 32;
                self.guest_regs.rcx = 0; 
                let vmcb = self.vmcb.as_mut().ok_or(HypervisorError::VmcbNotLoaded)?;
                vmcb.write_state(state_offsets::Af, vo);
                Ok(true)
            }
            
            SvmExitCode::Pause => {
                
                let vmcb = self.vmcb.as_mut().ok_or(HypervisorError::VmcbNotLoaded)?;
                vmcb.write_state(state_offsets::Af, vo);
                Ok(true)
            }
            
            SvmExitCode::Monitor | SvmExitCode::Mwait | SvmExitCode::MwaitConditional => {
                
                
                let vmcb = self.vmcb.as_mut().ok_or(HypervisorError::VmcbNotLoaded)?;
                vmcb.write_state(state_offsets::Af, vo);
                Ok(true)
            }
            
            SvmExitCode::Vintr => {
                
                Ok(true)
            }
            
            SvmExitCode::TaskSwitch => {
                
                
                crate::serial_println!("[SVM-VM {}] TaskSwitch at RIP=0x{:X}", self.id, guest_rip);
                let vmcb = self.vmcb.as_mut().ok_or(HypervisorError::VmcbNotLoaded)?;
                vmcb.write_state(state_offsets::Af, vo);
                Ok(true)
            }
            
            
            SvmExitCode::ExceptionDE | SvmExitCode::ExceptionDB |
            SvmExitCode::ExceptionBP | SvmExitCode::ExceptionOF |
            SvmExitCode::ExceptionBR | SvmExitCode::ExceptionUD |
            SvmExitCode::ExceptionNM | SvmExitCode::ExceptionMF |
            SvmExitCode::ExceptionAC | SvmExitCode::ExceptionXF => {
                
                let vector = (exit_code - 0x40) as u8;
                if self.stats.vmexits < 200 {
                    crate::serial_println!("[SVM-VM {}] Exception #{} at RIP=0x{:X} — re-injecting", 
                        self.id, vector, guest_rip);
                }
                let vmcb = self.vmcb.as_mut().ok_or(HypervisorError::VmcbNotLoaded)?;
                
                vmcb.inject_event(vector, 3, None);
                Ok(true)
            }
            
            SvmExitCode::ExceptionGP => {
                
                let error_code = bmb as u32;
                if self.stats.vmexits < 200 {
                    crate::serial_println!("[SVM-VM {}] #GP(0x{:X}) at RIP=0x{:X}", 
                        self.id, error_code, guest_rip);
                }
                let vmcb = self.vmcb.as_mut().ok_or(HypervisorError::VmcbNotLoaded)?;
                vmcb.inject_event(13, 3, Some(error_code));
                Ok(true)
            }
            
            SvmExitCode::ExceptionPF => {
                
                let error_code = bmb as u32;
                let aff = fvp;
                if self.stats.vmexits < 200 {
                    crate::serial_println!("[SVM-VM {}] #PF at 0x{:X} (err=0x{:X}) RIP=0x{:X}", 
                        self.id, aff, error_code, guest_rip);
                }
                let vmcb = self.vmcb.as_mut().ok_or(HypervisorError::VmcbNotLoaded)?;
                vmcb.write_state(state_offsets::Og, aff);
                vmcb.inject_event(14, 3, Some(error_code));
                Ok(true)
            }
            
            SvmExitCode::ExceptionDF => {
                
                crate::serial_println!("[SVM-VM {}] DOUBLE FAULT at RIP=0x{:X}", self.id, guest_rip);
                self.state = SvmVmState::Crashed;
                Ok(false)
            }
            
            SvmExitCode::ExceptionTS | SvmExitCode::ExceptionNP |
            SvmExitCode::ExceptionSS => {
                
                let vector = (exit_code - 0x40) as u8;
                let error_code = bmb as u32;
                if self.stats.vmexits < 200 {
                    crate::serial_println!("[SVM-VM {}] Exception #{} (err=0x{:X}) at RIP=0x{:X}", 
                        self.id, vector, error_code, guest_rip);
                }
                let vmcb = self.vmcb.as_mut().ok_or(HypervisorError::VmcbNotLoaded)?;
                vmcb.inject_event(vector, 3, Some(error_code));
                Ok(true)
            }
            
            SvmExitCode::ExceptionMC => {
                
                let vmcb = self.vmcb.as_mut().ok_or(HypervisorError::VmcbNotLoaded)?;
                vmcb.inject_event(18, 3, None);
                Ok(true)
            }
            
            _ => {
                crate::serial_println!("[SVM-VM {}] Unhandled #VMEXIT: {:?} (0x{:X}) at RIP=0x{:X}", 
                                      self.id, exit, exit_code, guest_rip);
                self.state = SvmVmState::Crashed;
                Ok(false)
            }
        }
    }
    
    
    fn handle_cpuid(&mut self) {
        let leaf = self.guest_regs.rax as u32;
        let subleaf = self.guest_regs.rcx as u32;
        
        
        match leaf {
            
            0x4000_0000 => {
                
                self.guest_regs.rax = 0x4000_0001;
                self.guest_regs.rbx = 0x7473_7254; 
                self.guest_regs.rcx = 0x7254_534F; 
                self.guest_regs.rdx = 0x534F_7473; 
                return;
            }
            0x4000_0001 => {
                
                self.guest_regs.rax = 0; 
                self.guest_regs.rbx = 0;
                self.guest_regs.rcx = 0;
                self.guest_regs.rdx = 0;
                return;
            }
            _ => {}
        }
        
        
        let (out_eax, out_ebx, out_ecx, out_edx): (u32, u32, u32, u32);
        
        unsafe {
            core::arch::asm!(
                "push rbx",
                "cpuid",
                "mov {out_ebx:e}, ebx",
                "pop rbx",
                inout("eax") leaf => out_eax,
                inout("ecx") subleaf => out_ecx,
                out_ebx = out(reg) out_ebx,
                out("edx") out_edx,
            );
        }
        
        let (mut eax, mut ebx, mut ecx, mut edx) = (out_eax, out_ebx, out_ecx, out_edx);
        
        match leaf {
            0x0000_0000 => {
                
                
                
            }
            0x0000_0001 => {
                
                ecx &= !(1 << 5);   
                ecx |= 1 << 31;     
                
                ecx &= !(1 << 21);  
                
                ecx &= !(1 << 3);   
            }
            0x0000_0007 => {
                
                if subleaf == 0 {
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
                
                if subleaf == 0 {
                    eax = 0; 
                    ebx = 1; 
                    ecx = (1 << 8) | subleaf; 
                } else if subleaf == 1 {
                    eax = 0;
                    ebx = 1; 
                    ecx = (2 << 8) | subleaf; 
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
        
        self.guest_regs.rax = eax as u64;
        self.guest_regs.rbx = ebx as u64;
        self.guest_regs.rcx = ecx as u64;
        self.guest_regs.rdx = edx as u64;
    }
    
    
    
    fn handle_npf(&mut self, zy: u64, error_code: u64, guest_rip: u64, uu: Option<&Bt>) -> bool {
        
        const BAE_: u64 = 0xFEE0_0000;
        const CGQ_: u64 = 0xFEE0_1000;
        const AFG_: u64 = 0xFEC0_0000;
        const CFL_: u64 = 0xFEC0_1000;
        const AFA_: u64 = 0xFED0_0000;
        const CDZ_: u64 = 0xFED0_1000;
        
        match zy {
            
            BAE_..=CGQ_ => {
                self.handle_lapic_mmio(zy, error_code, uu);
                true
            }
            
            AFG_..=CFL_ => {
                self.handle_ioapic_mmio(zy, error_code, uu);
                true
            }
            
            AFA_..=CDZ_ => {
                self.handle_hpet_mmio(zy, error_code, uu);
                true
            }
            
            0xA0000..=0xBFFFF => {
                if self.stats.npf_exits < 20 {
                    crate::serial_println!("[SVM-VM {}] VGA FB access at 0x{:X}", self.id, zy);
                }
                self.mmio_complete_read(uu, 0);
                true
            }
            
            0xC0000..=0xFFFFF => {
                self.mmio_complete_read(uu, 0);
                true
            }
            
            gm if gm < self.memory_size as u64 => {
                crate::serial_println!("[SVM-VM {}] NPF in guest RAM at 0x{:X}, err=0x{:X}, RIP=0x{:X}", 
                    self.id, gm, error_code, guest_rip);
                false
            }
            
            gm if gm >= 0x1_0000_0000 => {
                if self.stats.npf_exits < 50 {
                    crate::serial_println!("[SVM-VM {}] High MMIO access at 0x{:X} (absorbed)", self.id, gm);
                }
                self.mmio_complete_read(uu, 0xFFFF_FFFF);
                true
            }
            _ => {
                if self.stats.npf_exits < 50 {
                    crate::serial_println!("[SVM-VM {}] NPF: GPA=0x{:X}, err=0x{:X}, RIP=0x{:X}", 
                        self.id, zy, error_code, guest_rip);
                }
                false
            }
        }
    }
    
    
    
    fn mmio_get_write_value(&self, uu: Option<&Bt>) -> u32 {
        if let Some(ox) = uu {
            if let Some(imm) = ox.immediate {
                return mmio::ilw(imm, ox.operand_size) as u32;
            }
            if let Some(tb) = ox.register {
                let val = mmio::iyl(&self.guest_regs, tb);
                return mmio::ilw(val, ox.operand_size) as u32;
            }
        }
        
        self.guest_regs.rax as u32
    }
    
    
    
    fn mmio_complete_read(&mut self, uu: Option<&Bt>, value: u32) {
        if let Some(ox) = uu {
            if !ox.is_write {
                if let Some(tb) = ox.register {
                    
                    
                    
                    mmio::jro(&mut self.guest_regs, tb, value as u64);
                    return;
                }
            }
        }
        
        self.guest_regs.rax = value as u64;
    }
    
    
    fn handle_lapic_mmio(&mut self, gm: u64, error_code: u64, uu: Option<&Bt>) {
        let offset = (gm & 0xFFF) as u32;
        let is_write = (error_code & 0x2) != 0; 
        
        
        const AFQ_: u32 = 0x020;
        const AFS_: u32 = 0x030;
        const VV_: u32 = 0x080;
        const AFO_: u32 = 0x0B0;
        const OU_: u32 = 0x0F0;
        const CGS_: u32 = 0x100;
        const CGV_: u32 = 0x180;
        const CGR_: u32 = 0x200;
        const BAF_: u32 = 0x280;
        const BAJ_: u32 = 0x300;
        const BAH_: u32 = 0x310;
        const IX_: u32 = 0x320;
        const BAO_: u32 = 0x330;
        const BAN_: u32 = 0x340;
        const BAK_: u32 = 0x350;
        const BAL_: u32 = 0x360;
        const AFP_: u32 = 0x370;
        const LG_: u32 = 0x380;
        const AFR_: u32 = 0x390;
        const OV_: u32 = 0x3E0;
        
        if is_write {
            let value = self.mmio_get_write_value(uu);
            match offset {
                VV_ => {
                    self.lapic.tpr = value & 0xFF;
                    
                    if let Some(ref mut vmcb) = self.vmcb {
                        vmcb.write_u32(control_offsets::DFC_, value & 0x0F);
                    }
                }
                AFO_ => {
                    
                }
                OU_ => {
                    self.lapic.svr = value;
                    self.lapic.enabled = (value & 0x100) != 0;
                    if self.stats.vmexits < 1000 {
                        crate::serial_println!("[SVM-VM {}] LAPIC SVR=0x{:X} enabled={}", 
                            self.id, value, self.lapic.enabled);
                    }
                }
                BAJ_ => {
                    
                    if self.stats.vmexits < 1000 {
                        crate::serial_println!("[SVM-VM {}] LAPIC ICR write: 0x{:X} (IPI ignored, single vCPU)", 
                            self.id, value);
                    }
                }
                BAH_ => {} 
                IX_ => {
                    self.lapic.timer_lvt = value;
                    if self.stats.vmexits < 1000 {
                        let vector = value & 0xFF;
                        let masked = (value >> 16) & 1;
                        let mode = (value >> 17) & 0x3;
                        let nfu = match mode {
                            0 => "one-shot",
                            1 => "periodic",
                            2 => "TSC-deadline",
                            _ => "reserved",
                        };
                        crate::serial_println!("[SVM-VM {}] LAPIC timer LVT: vec={} mode={} masked={}", 
                            self.id, vector, nfu, masked);
                    }
                }
                BAK_ | BAL_ | BAO_ | BAN_ | AFP_ => {
                    
                }
                LG_ => {
                    self.lapic.icr = value;
                    self.lapic.ccr = value; 
                    self.lapic.last_tick_exit = self.stats.vmexits;
                    if self.stats.vmexits < 1000 && value > 0 {
                        crate::serial_println!("[SVM-VM {}] LAPIC timer ICR={} (timer armed)", self.id, value);
                    }
                }
                OV_ => {
                    self.lapic.dcr = value;
                }
                BAF_ => {} 
                _ => {
                    if self.stats.vmexits < 200 {
                        crate::serial_println!("[SVM-VM {}] LAPIC write offset=0x{:X} val=0x{:X}", 
                            self.id, offset, value);
                    }
                }
            }
        } else {
            
            let value: u32 = match offset {
                AFQ_ => 0,                    
                AFS_ => 0x0005_0014,     
                VV_ => self.lapic.tpr,
                OU_ => self.lapic.svr,
                CGS_..=0x170 => 0,
                CGV_..=0x1F0 => 0,
                CGR_..=0x270 => 0,
                BAF_ => 0,
                BAJ_ => 0,               
                BAH_ => 0,
                IX_ => self.lapic.timer_lvt,
                BAO_ => 0x0001_0000, 
                BAN_ => 0x0001_0000,    
                BAK_ => 0x0001_0000,        
                BAL_ => 0x0001_0000,        
                AFP_ => 0x0001_0000,    
                LG_ => self.lapic.icr,
                AFR_ => {
                    
                    if self.lapic.icr > 0 {
                        let bb = self.stats.vmexits.saturating_sub(self.lapic.last_tick_exit);
                        let cws = match self.lapic.dcr & 0xB {
                            0x0 => 2u64, 0x1 => 4, 0x2 => 8, 0x3 => 16,
                            0x8 => 32, 0x9 => 64, 0xA => 128, 0xB => 1,
                            _ => 1,
                        };
                        let gx = (bb * 256) / cws;
                        let ck = (self.lapic.icr as u64).saturating_sub(gx);
                        ck as u32
                    } else {
                        0
                    }
                }
                OV_ => self.lapic.dcr,
                _ => 0,
            };
            self.mmio_complete_read(uu, value);
        }
    }
    
    
    fn handle_ioapic_mmio(&mut self, gm: u64, error_code: u64, uu: Option<&Bt>) {
        let offset = gm - super::ioapic::AFG_;
        let is_write = (error_code & 0x2) != 0;
        
        if is_write {
            let value = self.mmio_get_write_value(uu);
            self.ioapic.write(offset, value);
            if self.stats.npf_exits < 100 {
                crate::serial_println!("[SVM-VM {}] IOAPIC write offset=0x{:X} val=0x{:X}", 
                    self.id, offset, value);
            }
        } else {
            let value = self.ioapic.read(offset);
            if self.stats.npf_exits < 100 {
                crate::serial_println!("[SVM-VM {}] IOAPIC read offset=0x{:X} -> 0x{:X}", 
                    self.id, offset, value);
            }
            self.mmio_complete_read(uu, value);
        }
    }
    
    
    fn handle_hpet_mmio(&mut self, gm: u64, error_code: u64, uu: Option<&Bt>) {
        let offset = gm - super::hpet::AFA_;
        let is_write = (error_code & 0x2) != 0;
        
        
        let size = uu.map(|d| d.operand_size).unwrap_or(4);
        
        if is_write {
            let value = self.mmio_get_write_value(uu) as u64;
            self.hpet.write(offset, value, size);
            if self.stats.npf_exits < 50 {
                crate::serial_println!("[SVM-VM {}] HPET write offset=0x{:X} val=0x{:X} size={}", 
                    self.id, offset, value, size);
            }
        } else {
            let value = self.hpet.read(offset, size);
            if self.stats.npf_exits < 50 {
                crate::serial_println!("[SVM-VM {}] HPET read offset=0x{:X} -> 0x{:X} size={}", 
                    self.id, offset, value, size);
            }
            self.mmio_complete_read(uu, value as u32);
        }
    }
    
    
    
    fn check_hpet_interrupts(&mut self) -> Option<u64> {
        let pju = self.hpet.check_timers();
        
        for (i, &(fired, gdo)) in pju.iter().enumerate() {
            if !fired {
                continue;
            }
            
            
            if let Some(afo) = self.ioapic.get_irq_route(gdo) {
                if !afo.masked && afo.vector > 0 {
                    
                    self.hpet.isr |= 1 << i;
                    
                    
                    let config = self.hpet.timers[i].config;
                    let ntp = (config >> 3) & 1 != 0;
                    if ntp {
                        
                        let bfm = self.hpet.timers[i].comparator;
                        if bfm > 0 {
                            self.hpet.timers[i].comparator = self.hpet.timers[i].comparator.wrapping_add(bfm);
                        }
                    } else {
                        
                        self.hpet.timers[i].config &= !(1 << 2);
                    }
                    
                    return Some(afo.vector as u64);
                }
            }
        }
        None
    }
    
    
    fn handle_io(&mut self, drx: u64) {
        let msq = (drx & 1) != 0;
        let port = ((drx >> 16) & 0xFFFF) as u16;
        let bek = match (drx >> 4) & 0x7 {
            0 => 1, 
            1 => 2, 
            2 => 4, 
            _ => 1,
        };
        
        if msq {
            
            let value: u32 = match port {
                
                0x3F8 => {
                    
                    if let Some(byte) = self.serial_input_buffer.pop_front() {
                        byte as u32
                    } else {
                        0
                    }
                }
                0x3F9 => self.serial_ier as u32, 
                0x3FA => {
                    
                    if (self.serial_ier & 0x01) != 0 && !self.serial_input_buffer.is_empty() {
                        0xC4 
                    } else {
                        0xC1 
                    }
                }
                0x3FB => 0x03,                  
                0x3FC => 0x03,                  
                0x3FD => {
                    
                    let mut bht = 0x60u32; 
                    if !self.serial_input_buffer.is_empty() {
                        bht |= 0x01; 
                    }
                    bht
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
                    
                    self.pic.master_isr as u32
                }
                0x21 => self.pic.master_imr as u32,  
                0xA0 => 0,                           
                0xA1 => self.pic.slave_imr as u32,   
                
                
                0x40 | 0x41 | 0x42 => {
                    let ch = (port - 0x40) as usize;
                    let ze = &mut self.pit.channels[ch];
                    if ze.latched {
                        ze.latched = false;
                        ze.latch_value as u32
                    } else {
                        
                        let otd = ze.count.wrapping_sub(
                            (self.stats.vmexits & 0xFFFF) as u16
                        );
                        otd as u32
                    }
                }
                0x43 => 0,                      
                0x61 => 0x20,                   
                
                
                0x70 => self.cmos_index as u32,
                0x71 => {
                    
                    (match self.cmos_index {
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
                
                
                0xCF8 => self.pci.read_config_address(),
                0xCFC..=0xCFF => {
                    let uo = (port - 0xCFC) as u8;
                    self.pci.read_config_data(uo)
                }
                
                
                0xC000..=0xC03F => {
                    let offset = port - 0xC000;
                    self.virtio_console_state.io_read(offset)
                }
                
                
                0xC040..=0xC07F => {
                    let offset = port - 0xC040;
                    self.virtio_blk_state.io_read(offset)
                }
                
                
                0xB000 => 0,           
                0xB002 => 0,           
                0xB004 => 0,           
                0xB008 => {
                    
                    
                    let gx = self.stats.vmexits.wrapping_mul(4); 
                    (gx & 0xFFFF_FFFF) as u32
                }
                0xB009..=0xB00B => {
                    
                    let gx = self.stats.vmexits.wrapping_mul(4);
                    let uo = (port - 0xB008) as u32;
                    ((gx >> (uo * 8)) & 0xFF) as u32
                }
                0xB00C..=0xB03F => 0,  
                
                
                0xE9 => 0,                      
                0xED => 0,                      
                0x92 => 0x02,                   
                
                _ => {
                    
                    if self.stats.io_exits < 50 {
                        crate::serial_println!("[SVM-VM {}] Unhandled IN port 0x{:X}", self.id, port);
                    }
                    super::debug_monitor::akj(
                        self.id, super::debug_monitor::DebugCategory::IoPortIn,
                        port as u64, super::debug_monitor::HandleStatus::Unhandled,
                        self.vmcb.as_ref().map(|v| v.read_state(state_offsets::Af)).unwrap_or(0),
                        self.stats.vmexits, "",
                    );
                    0xFF
                }
            };
            self.guest_regs.rax = (self.guest_regs.rax & !0xFFFF_FFFF) | (value as u64);
        } else {
            
            let value = self.guest_regs.rax as u32;
            
            match port {
                
                0x3F8 => {
                    let ch = (value & 0xFF) as u8;
                    crate::serial_print!("{}", ch as char);
                    if let Some(console_id) = self.console_id {
                        super::console::write_char(console_id, ch as char);
                    }
                }
                
                0x2F8 => {
                    let ch = (value & 0xFF) as u8;
                    crate::serial_print!("{}", ch as char);
                }
                
                0x3F9 => {
                    
                    self.serial_ier = value as u8;
                }
                0x3FA => {
                    
                    self.serial_fcr = value as u8;
                    if (value & 0x02) != 0 {
                        
                        self.serial_input_buffer.clear();
                    }
                }
                0x3FB..=0x3FF => {} 
                
                0x2F9..=0x2FF => {}
                
                
                0x20 => {
                    
                    let v = value as u8;
                    if v & 0x10 != 0 {
                        
                        self.pic.master_icw_phase = 1;
                        self.pic.master_isr = 0;
                        self.pic.master_irr = 0;
                        if self.stats.io_exits < 200 {
                            crate::serial_println!("[SVM-VM {}] PIC master: ICW1=0x{:02X}", self.id, v);
                        }
                    } else if v & 0x08 != 0 {
                        
                    } else {
                        
                        if v == 0x20 {
                            
                            self.pic.master_isr = 0;
                        }
                    }
                }
                0x21 => {
                    
                    let v = value as u8;
                    match self.pic.master_icw_phase {
                        1 => {
                            
                            self.pic.master_vector_base = v & 0xF8;
                            self.pic.master_icw_phase = 2;
                            if self.stats.io_exits < 200 {
                                crate::serial_println!("[SVM-VM {}] PIC master: ICW2 vector_base=0x{:02X}", self.id, v);
                            }
                        }
                        2 => {
                            
                            self.pic.master_icw_phase = 3;
                        }
                        3 => {
                            
                            self.pic.master_icw_phase = 0;
                            self.pic.initialized = true;
                            if self.stats.io_exits < 200 {
                                crate::serial_println!("[SVM-VM {}] PIC master initialized: base=0x{:02X}", 
                                    self.id, self.pic.master_vector_base);
                            }
                        }
                        _ => {
                            
                            self.pic.master_imr = v;
                        }
                    }
                }
                0xA0 => {
                    
                    let v = value as u8;
                    if v & 0x10 != 0 {
                        self.pic.slave_icw_phase = 1;
                    } else if v == 0x20 {
                        
                    }
                }
                0xA1 => {
                    
                    let v = value as u8;
                    match self.pic.slave_icw_phase {
                        1 => {
                            self.pic.slave_vector_base = v & 0xF8;
                            self.pic.slave_icw_phase = 2;
                            if self.stats.io_exits < 200 {
                                crate::serial_println!("[SVM-VM {}] PIC slave: ICW2 vector_base=0x{:02X}", self.id, v);
                            }
                        }
                        2 => { self.pic.slave_icw_phase = 3; }
                        3 => { self.pic.slave_icw_phase = 0; }
                        _ => { self.pic.slave_imr = v; }
                    }
                }
                
                
                0x40 | 0x41 | 0x42 => {
                    let ch = (port - 0x40) as usize;
                    let v = value as u8;
                    let ze = &mut self.pit.channels[ch];
                    match ze.access {
                        1 => {
                            
                            ze.reload = (ze.reload & 0xFF00) | v as u16;
                            ze.count = ze.reload;
                        }
                        2 => {
                            
                            ze.reload = (ze.reload & 0x00FF) | ((v as u16) << 8);
                            ze.count = ze.reload;
                        }
                        3 => {
                            
                            if ze.write_hi_pending {
                                ze.reload = (ze.reload & 0x00FF) | ((v as u16) << 8);
                                ze.count = ze.reload;
                                ze.write_hi_pending = false;
                                if ch == 0 && self.stats.io_exits < 200 {
                                    crate::serial_println!("[SVM-VM {}] PIT ch0: reload={} ({} Hz)", 
                                        self.id, ze.reload,
                                        if ze.reload > 0 { 1193182 / ze.reload as u32 } else { 0 });
                                }
                            } else {
                                ze.reload = (ze.reload & 0xFF00) | v as u16;
                                ze.write_hi_pending = true;
                            }
                        }
                        _ => {}
                    }
                }
                0x43 => {
                    
                    let v = value as u8;
                    let channel = ((v >> 6) & 0x3) as usize;
                    let access = (v >> 4) & 0x3;
                    let mode = (v >> 1) & 0x7;
                    
                    if channel < 3 {
                        if access == 0 {
                            
                            self.pit.channels[channel].latched = true;
                            self.pit.channels[channel].latch_value = self.pit.channels[channel].count;
                        } else {
                            self.pit.channels[channel].access = access;
                            self.pit.channels[channel].mode = mode;
                            self.pit.channels[channel].write_hi_pending = false;
                        }
                    }
                }
                
                
                0x70 => {
                    self.cmos_index = (value as u8) & 0x7F; 
                }
                0x71 => {} 
                
                
                0x61 => {}
                
                
                0x60 | 0x64 => {}
                
                
                0x00..=0x0F | 0x80..=0x8F | 0xC0..=0xDF => {}
                
                
                0x3B0..=0x3DF => {}
                
                
                0xCF8 => {
                    self.pci.write_config_address(value);
                    if self.stats.io_exits < 200 {
                        crate::serial_println!("[SVM-VM {}] PCI CFG ADDR = 0x{:08X}", self.id, value);
                    }
                }
                0xCFC..=0xCFF => {
                    let uo = (port - 0xCFC) as u8;
                    self.pci.write_config_data(uo, value);
                    if self.stats.io_exits < 200 {
                        let (_, bus, s, func, reg) = {
                            let addr = self.pci.config_addr;
                            (addr >> 31 != 0, (addr >> 16) as u8 & 0xFF, (addr >> 11) as u8 & 0x1F, (addr >> 8) as u8 & 0x7, addr as u8 & 0xFC)
                        };
                        crate::serial_println!("[SVM-VM {}] PCI CFG WRITE {:02X}:{:02X}.{} reg=0x{:02X} val=0x{:X}", 
                            self.id, bus, s, func, reg, value);
                    }
                }
                
                
                0xC000..=0xC03F => {
                    let offset = port - 0xC000;
                    let bif = self.virtio_console_state.io_write(offset, value);
                    if bif {
                        
                        self.virtio_console_state.process_transmitq(&self.guest_memory);
                    }
                }
                
                
                0xC040..=0xC07F => {
                    let offset = port - 0xC040;
                    let bif = self.virtio_blk_state.io_write(offset, value);
                    if bif {
                        
                        
                        let oxr = self.virtio_blk_storage.len();
                        let oxs = self.virtio_blk_storage.as_mut_ptr();
                        let ned = self.guest_memory.as_mut_ptr();
                        let nec = self.guest_memory.len();
                        
                        unsafe {
                            let storage = core::slice::from_raw_parts_mut(oxs, oxr);
                            let mgi = core::slice::from_raw_parts_mut(ned, nec);
                            self.virtio_blk_state.process_queue(mgi, storage);
                        }
                    }
                }
                
                
                0x92 => {}
                
                
                0xE9 => {
                    let ch = (value & 0xFF) as u8;
                    crate::serial_print!("{}", ch as char);
                }
                0xED => {} 
                
                
                0xB000..=0xB003 => {
                    
                    if self.stats.io_exits < 100 {
                        crate::serial_println!("[SVM-VM {}] ACPI PM1a_EVT write: port=0x{:X} val=0x{:X}", self.id, port, value);
                    }
                }
                0xB004..=0xB005 => {
                    
                    if self.stats.io_exits < 100 {
                        crate::serial_println!("[SVM-VM {}] ACPI PM1a_CNT write: port=0x{:X} val=0x{:X}", self.id, port, value);
                    }
                    
                    if port == 0xB004 && (value & 0x2000) != 0 {
                        let bpa = (value >> 10) & 0x7;
                        crate::serial_println!("[SVM-VM {}] ACPI shutdown request: SLP_TYP={}", self.id, bpa);
                        if bpa == 5 {
                            self.state = SvmVmState::Stopped;
                        }
                    }
                }
                0xB006..=0xB03F => {} 
                
                _ => {
                    if self.stats.io_exits < 50 {
                        crate::serial_println!("[SVM-VM {}] Unhandled OUT port 0x{:X} val=0x{:X}", self.id, port, value);
                    }
                    super::debug_monitor::akj(
                        self.id, super::debug_monitor::DebugCategory::IoPortOut,
                        port as u64, super::debug_monitor::HandleStatus::Unhandled,
                        self.vmcb.as_ref().map(|v| v.read_state(state_offsets::Af)).unwrap_or(0),
                        self.stats.vmexits, &alloc::format!("val=0x{:X}", value),
                    );
                }
            }
        }
    }
    
    
    fn handle_msr(&mut self, is_write: bool) {
        let msr = self.guest_regs.rcx as u32;
        
        
        const LD_: u32 = 0x001B;
        const CEH_: u32 = 0x00FE;
        const AYO_: u32 = 0x0174;
        const AYQ_: u32 = 0x0175;
        const AYP_: u32 = 0x0176;
        const CEG_: u32 = 0x0179;
        const AYM_: u32 = 0x017A;
        const OO_: u32 = 0x01A0;
        const LE_: u32 = 0x0277;
        const AYN_: u32 = 0x02FF;
        const IA32_EFER: u32 = 0xC000_0080;
        const WZ_: u32 = 0xC000_0081;
        const WX_: u32 = 0xC000_0082;
        const WT_: u32 = 0xC000_0083;
        const WY_: u32 = 0xC000_0084;
        const WU_: u32 = 0xC000_0100;
        const WV_: u32 = 0xC000_0101;
        const WW_: u32 = 0xC000_0102;
        const PG_: u32 = 0xC000_0103;
        
        if is_write {
            let value = (self.guest_regs.rdx << 32) | (self.guest_regs.rax & 0xFFFF_FFFF);
            
            match msr {
                
                WZ_ => {
                    let vmcb = self.vmcb.as_mut().unwrap();
                    vmcb.write_state(state_offsets::Aek, value);
                }
                WX_ => {
                    let vmcb = self.vmcb.as_mut().unwrap();
                    vmcb.write_state(state_offsets::Aar, value);
                }
                WT_ => {
                    let vmcb = self.vmcb.as_mut().unwrap();
                    vmcb.write_state(state_offsets::Xb, value);
                }
                WY_ => {
                    let vmcb = self.vmcb.as_mut().unwrap();
                    vmcb.write_state(state_offsets::Adq, value);
                }
                WW_ => {
                    let vmcb = self.vmcb.as_mut().unwrap();
                    vmcb.write_state(state_offsets::AZU_, value);
                }
                WU_ => {
                    let vmcb = self.vmcb.as_mut().unwrap();
                    vmcb.write_state(state_offsets::AEA_, value);
                }
                WV_ => {
                    let vmcb = self.vmcb.as_mut().unwrap();
                    vmcb.write_state(state_offsets::AEU_, value);
                }
                AYO_ => {
                    let vmcb = self.vmcb.as_mut().unwrap();
                    vmcb.write_state(state_offsets::BIX_, value);
                }
                AYQ_ => {
                    let vmcb = self.vmcb.as_mut().unwrap();
                    vmcb.write_state(state_offsets::BIZ_, value);
                }
                AYP_ => {
                    let vmcb = self.vmcb.as_mut().unwrap();
                    vmcb.write_state(state_offsets::BIY_, value);
                }
                LE_ => {
                    let vmcb = self.vmcb.as_mut().unwrap();
                    vmcb.write_state(state_offsets::Uf, value);
                }
                IA32_EFER => {
                    let vmcb = self.vmcb.as_mut().unwrap();
                    
                    let ojv = value | 0x1000; 
                    vmcb.write_state(state_offsets::Eu, ojv);
                }
                
                LD_ | OO_ | AYM_ |
                AYN_ | PG_ => {}
                
                0x0200..=0x020F => {}
                
                0x0400..=0x047F => {}
                _ => {
                    if self.stats.msr_exits < 100 {
                        crate::serial_println!("[SVM-VM {}] WRMSR 0x{:X} = 0x{:X} (ignored)", self.id, msr, value);
                    }
                    super::debug_monitor::akj(
                        self.id, super::debug_monitor::DebugCategory::MsrWrite,
                        msr as u64, super::debug_monitor::HandleStatus::Unhandled,
                        self.vmcb.as_ref().map(|v| v.read_state(state_offsets::Af)).unwrap_or(0),
                        self.stats.vmexits, &alloc::format!("val=0x{:X}", value),
                    );
                }
            }
        } else {
            
            let value: u64 = match msr {
                
                WZ_ => {
                    let vmcb = self.vmcb.as_ref().unwrap();
                    vmcb.read_state(state_offsets::Aek)
                }
                WX_ => {
                    let vmcb = self.vmcb.as_ref().unwrap();
                    vmcb.read_state(state_offsets::Aar)
                }
                WT_ => {
                    let vmcb = self.vmcb.as_ref().unwrap();
                    vmcb.read_state(state_offsets::Xb)
                }
                WY_ => {
                    let vmcb = self.vmcb.as_ref().unwrap();
                    vmcb.read_state(state_offsets::Adq)
                }
                WW_ => {
                    let vmcb = self.vmcb.as_ref().unwrap();
                    vmcb.read_state(state_offsets::AZU_)
                }
                WU_ => {
                    let vmcb = self.vmcb.as_ref().unwrap();
                    vmcb.read_state(state_offsets::AEA_)
                }
                WV_ => {
                    let vmcb = self.vmcb.as_ref().unwrap();
                    vmcb.read_state(state_offsets::AEU_)
                }
                AYO_ => {
                    let vmcb = self.vmcb.as_ref().unwrap();
                    vmcb.read_state(state_offsets::BIX_)
                }
                AYQ_ => {
                    let vmcb = self.vmcb.as_ref().unwrap();
                    vmcb.read_state(state_offsets::BIZ_)
                }
                AYP_ => {
                    let vmcb = self.vmcb.as_ref().unwrap();
                    vmcb.read_state(state_offsets::BIY_)
                }
                LE_ => {
                    let vmcb = self.vmcb.as_ref().unwrap();
                    vmcb.read_state(state_offsets::Uf)
                }
                IA32_EFER => {
                    let vmcb = self.vmcb.as_ref().unwrap();
                    vmcb.read_state(state_offsets::Eu)
                }
                
                LD_ => 0xFEE0_0900,
                
                CEH_ => 0,
                
                CEG_ => 0,
                AYM_ => 0,
                
                OO_ => 1, 
                
                AYN_ => 0x06,
                
                PG_ => 0,
                
                0x0200..=0x020F => 0,
                
                0x0400..=0x047F => 0,
                _ => {
                    if self.stats.msr_exits < 100 {
                        crate::serial_println!("[SVM-VM {}] RDMSR 0x{:X} = 0 (default)", self.id, msr);
                    }
                    super::debug_monitor::akj(
                        self.id, super::debug_monitor::DebugCategory::MsrRead,
                        msr as u64, super::debug_monitor::HandleStatus::Unhandled,
                        self.vmcb.as_ref().map(|v| v.read_state(state_offsets::Af)).unwrap_or(0),
                        self.stats.vmexits, "returned 0",
                    );
                    0
                }
            };
            
            self.guest_regs.rax = value & 0xFFFF_FFFF;
            self.guest_regs.rdx = value >> 32;
        }
    }
    
    
    fn handle_vmmcall(&mut self) -> bool {
        let function = self.guest_regs.rax;
        let arg1 = self.guest_regs.rbx;
        let arg2 = self.guest_regs.rcx;
        let aer = self.guest_regs.rdx;
        
        crate::serial_println!("[SVM-VM {}] VMMCALL: func=0x{:X}, args=({:X}, {:X}, {:X})", 
                              self.id, function, arg1, arg2, aer);
        
        let (result, gux): (i64, bool) = match function {
            
            0x00 => {
                self.state = SvmVmState::Stopped;
                (0, false)
            }
            
            
            0x01 => {
                self.hypercall_print(arg1);
                (0, true)
            }
            
            
            0x02 => {
                (unsafe { core::arch::x86_64::_rdtsc() as i64 }, true)
            }
            
            _ => (-1, true), 
        };
        
        
        self.guest_regs.rax = result as u64;
        
        
        super::api::bzf(
            super::api::VmEventType::Hypercall,
            self.id,
            super::api::VmEventData::HypercallInfo { function, result },
        );
        
        gux
    }
    
    
    fn hypercall_print(&self, gm: u64) {
        let offset = gm as usize;
        if offset < self.guest_memory.len() {
            
            let aoo = (self.guest_memory.len() - offset).min(256);
            let slice = &self.guest_memory[offset..offset + aoo];
            
            if let Some(null_pos) = slice.iter().position(|&c| c == 0) {
                if let Ok(j) = core::str::from_utf8(&slice[..null_pos]) {
                    crate::serial_println!("[SVM-VM {} PRINT] {}", self.id, j);
                    if let Some(console_id) = self.console_id {
                        for ch in j.chars() {
                            super::console::write_char(console_id, ch);
                        }
                    }
                }
            }
        }
    }
    
    
    pub fn pause(&mut self) -> Result<()> {
        if self.state == SvmVmState::Running {
            self.state = SvmVmState::Paused;
            crate::serial_println!("[SVM-VM {}] Paused", self.id);
        }
        Ok(())
    }
    
    
    pub fn resume(&mut self) -> Result<()> {
        if self.state == SvmVmState::Paused {
            self.state = SvmVmState::Running;
            crate::serial_println!("[SVM-VM {}] Resumed", self.id);
        }
        Ok(())
    }
    
    
    pub fn get_stats(&self) -> &SvmVmStats {
        &self.stats
    }
    
    
    pub fn get_state(&self) -> SvmVmState {
        self.state
    }
    
    
    pub fn read_guest_memory(&self, gm: u64, len: usize) -> Option<&[u8]> {
        let offset = gm as usize;
        if offset + len <= self.guest_memory.len() {
            Some(&self.guest_memory[offset..offset + len])
        } else {
            None
        }
    }
    
    
    pub fn write_guest_memory(&mut self, gm: u64, data: &[u8]) -> Result<()> {
        let offset = gm as usize;
        if offset + data.len() <= self.guest_memory.len() {
            self.guest_memory[offset..offset + data.len()].copy_from_slice(data);
            Ok(())
        } else {
            Err(HypervisorError::OutOfMemory)
        }
    }
}

impl Drop for SvmVirtualMachine {
    fn drop(&mut self) {
        
        super::svm::npt::lyo(self.asid);
        
        
        if let Some(_console_id) = self.console_id {
            
        }
        
        
        super::virtfs::ofb(self.id);
        
        crate::serial_println!("[SVM-VM {}] Destroyed", self.id);
    }
}


static YW_: Mutex<Vec<SvmVirtualMachine>> = Mutex::new(Vec::new());


pub fn blh(name: &str, memory_mb: usize) -> Result<u64> {
    let vm = SvmVirtualMachine::new(name, memory_mb)?;
    let id = vm.id;
    YW_.lock().push(vm);
    Ok(id)
}


pub fn avv<F, U>(id: u64, f: F) -> Option<U>
where
    F: FnOnce(&mut SvmVirtualMachine) -> U,
{
    let mut aen = YW_.lock();
    aen.iter_mut().find(|vm| vm.id == id).map(f)
}


pub fn dtn() -> Vec<(u64, String, SvmVmState)> {
    YW_.lock()
        .iter()
        .map(|vm| (vm.id, vm.name.clone(), vm.state))
        .collect()
}


pub fn qcv(id: u64) -> Result<()> {
    let mut aen = YW_.lock();
    if let Some(pos) = aen.iter().position(|vm| vm.id == id) {
        aen.remove(pos);
        Ok(())
    } else {
        Err(HypervisorError::VmNotFound)
    }
}





#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn qzw() {
        let pic = PicState::default();
        assert_eq!(pic.master_imr, 0xFF);
        assert_eq!(pic.slave_imr, 0xFF);
        assert_eq!(pic.master_vector_base, 0x08);
        assert_eq!(pic.slave_vector_base, 0x70);
        assert_eq!(pic.master_icw_phase, 0);
        assert!(!pic.initialized);
    }

    #[test]
    fn qzx() {
        let mut pic = PicState::default();
        
        pic.master_icw_phase = 1;
        pic.master_isr = 0;
        
        pic.master_vector_base = 0x20;
        pic.master_icw_phase = 2;
        
        pic.master_icw_phase = 3;
        
        pic.master_icw_phase = 0;
        pic.initialized = true;
        assert_eq!(pic.master_vector_base, 0x20);
        assert!(pic.initialized);
    }

    #[test]
    fn qzy() {
        let pit = PitState::default();
        assert_eq!(pit.channels[0].reload, 0xFFFF);
        assert_eq!(pit.channels[0].count, 0xFFFF);
        assert_eq!(pit.channels[0].access, 3);
        assert!(!pit.channels[0].latched);
        assert_eq!(pit.channels.len(), 3);
    }

    #[test]
    fn qzz() {
        let mut ch = PitChannel::default();
        ch.access = 3;
        
        ch.reload = (ch.reload & 0xFF00) | 0x9C;
        ch.write_hi_pending = true;
        
        ch.reload = (ch.reload & 0x00FF) | (0x2E << 8);
        ch.count = ch.reload;
        ch.write_hi_pending = false;
        assert_eq!(ch.reload, 0x2E9C);
        assert_eq!(ch.count, 0x2E9C);
    }

    #[test]
    fn qzq() {
        let lapic = LapicState::default();
        assert_eq!(lapic.icr, 0);
        assert!(!lapic.enabled);
        assert_ne!(lapic.timer_lvt & 0x0001_0000, 0); 
        assert_eq!(lapic.svr, 0x1FF);
    }

    #[test]
    fn qzs() {
        let mut lapic = LapicState::default();
        lapic.svr = 0x1FF;
        lapic.enabled = (lapic.svr & 0x100) != 0;
        assert!(lapic.enabled);
        lapic.svr = 0x0FF;
        lapic.enabled = (lapic.svr & 0x100) != 0;
        assert!(!lapic.enabled);
    }

    #[test]
    fn qzr() {
        let map = [(0x0u32, 2u64), (0x1, 4), (0x2, 8), (0x3, 16),
                    (0x8, 32), (0x9, 64), (0xA, 128), (0xB, 1)];
        for &(dcr, expected) in &map {
            let d = match dcr & 0xB {
                0x0 => 2u64, 0x1 => 4, 0x2 => 8, 0x3 => 16,
                0x8 => 32, 0x9 => 64, 0xA => 128, 0xB => 1, _ => 1,
            };
            assert_eq!(d, expected, "dcr=0x{:X}", dcr);
        }
    }
}
