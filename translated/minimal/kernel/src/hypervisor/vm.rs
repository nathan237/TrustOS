









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
    Created,
    Running,
    Paused,
    Stopped,
    Crashed,
}


#[derive(Debug, Clone, Default)]
pub struct VmStats {
    pub vm_exits: u64,
    pub cpuid_exits: u64,
    pub io_exits: u64,
    pub msr_exits: u64,
    pub hlt_exits: u64,
    pub ept_violations: u64,
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
    pub id: u64,
    pub name: String,
    pub state: VmState,
    pub memory_size: usize,
    pub stats: VmStats,
    vmcs: Option<Vmcs>,
    ept: Option<EptManager>,
    guest_memory: Vec<u8>,
    guest_regs: GuestRegs,
    
    console_id: Option<usize>,
    
    vpid: Option<u16>,
}

impl VirtualMachine {
    pub fn new(id: u64, name: &str, memory_mb: usize) -> Result<Self> {
        let memory_size = memory_mb * 1024 * 1024;
        
        
        let guest_memory = alloc::vec![0u8; memory_size];
        
        
        let console_id = super::console::create_console(id, name);
        
        
        super::virtfs::hot(id);
        
        
        let vpid = super::vpid::allocate();
        if vpid.is_some() {
            crate::serial_println!("[VM {}] Allocated VPID {} for TLB isolation", id, vpid.unwrap());
        }
        
        
        super::api::bzf(
            super::api::VmEventType::Created,
            id,
            super::api::VmEventData::Az(alloc::format!("VM '{}' created", name)),
        );
        
        Ok(VirtualMachine {
            id,
            name: String::from(name),
            state: VmState::Created,
            memory_size,
            stats: VmStats::default(),
            vmcs: None,
            ept: None,
            guest_memory,
            guest_regs: GuestRegs::default(),
            console_id: Some(console_id),
            vpid,
        })
    }
    
    
    pub fn add_mount(&mut self, host_path: &str, guest_path: &str, readonly: bool) {
        super::virtfs::add_mount(self.id, host_path, guest_path, readonly);
    }
    
    
    pub fn initialize(&mut self) -> Result<()> {
        crate::serial_println!("[VM {}] Initializing VMCS and EPT", self.id);
        
        
        let csl = vmx::ach(vmx::VH_);
        let azj = (csl & 0x7FFF_FFFF) as u32;
        
        
        let mut vmcs = Vmcs::new(azj)?;
        vmcs.load()?;
        
        
        vmcs.setup_execution_controls()?;
        vmcs.setup_exit_controls()?;
        vmcs.setup_entry_controls()?;
        
        
        vmcs.setup_vpid(self.vpid)?;
        
        
        let mut ept = EptManager::new(self.memory_size)?;
        ept.setup_guest_memory_mapping(&self.guest_memory)?;
        
        
        vmcs.write(fields::BXD_, ept.ept_pointer().as_u64())?;
        
        self.vmcs = Some(vmcs);
        self.ept = Some(ept);
        
        crate::serial_println!("[VM {}] Initialization complete (VPID={:?})", self.id, self.vpid);
        
        Ok(())
    }
    
    
    pub fn load_binary(&mut self, data: &[u8], load_address: u64) -> Result<()> {
        let offset = load_address as usize;
        
        if offset + data.len() > self.guest_memory.len() {
            return Err(HypervisorError::OutOfMemory);
        }
        
        self.guest_memory[offset..offset + data.len()].copy_from_slice(data);
        
        crate::serial_println!("[VM {}] Loaded {} bytes at 0x{:X}", 
                              self.id, data.len(), load_address);
        
        Ok(())
    }
    
    
    pub fn start(&mut self, entry_point: u64, stack_ptr: u64) -> Result<()> {
        if self.vmcs.is_none() {
            self.initialize()?;
        }
        
        let vmcs = self.vmcs.as_ref().unwrap();
        
        
        vmcs.setup_guest_state(entry_point, stack_ptr)?;
        
        
        let cjc = jqi as *const () as u64;
        
        
        let drm = alloc::vec![0u8; 16384];
        let epn = (drm.as_ptr() as u64 + 16384) & !0xF;
        core::mem::forget(drm); 
        
        vmcs.setup_host_state(cjc, epn)?;
        
        self.state = VmState::Running;
        
        crate::serial_println!("[VM {}] Starting at RIP=0x{:X}, RSP=0x{:X}", 
                              self.id, entry_point, stack_ptr);
        crate::serial_println!("[VM {}] Host exit handler=0x{:X}, host stack=0x{:X}",
                              self.id, cjc, epn);
        
        
        self.run_loop()?;
        
        Ok(())
    }
    
    
    
    
    
    pub fn start_linux(
        &mut self,
        bas: &[u8],
        cmdline: &str,
        initrd: Option<&[u8]>,
    ) -> Result<()> {
        use super::linux_loader;
        
        crate::serial_println!("[VM {}] Starting Linux kernel ({} bytes)", self.id, bas.len());
        
        
        let pk = linux_loader::nwu(
            &mut self.guest_memory,
            bas,
            cmdline,
            initrd,
        )?;
        
        
        if self.vmcs.is_none() {
            self.initialize()?;
        }
        
        let vmcs = self.vmcs.as_ref().unwrap();
        
        
        linux_loader::kwy(vmcs, &pk)?;
        
        
        self.guest_regs.rsi = pk.boot_params_addr;
        self.save_guest_regs_for_entry();
        
        
        let cjc = jqi as *const () as u64;
        let drm = alloc::vec![0u8; 16384];
        let epn = (drm.as_ptr() as u64 + 16384) & !0xF;
        core::mem::forget(drm);
        
        vmcs.setup_host_state(cjc, epn)?;
        
        self.state = VmState::Running;
        
        crate::serial_println!("[VM {}] Linux: RIP=0x{:X} RSP=0x{:X} RSI(boot_params)=0x{:X} CR3=0x{:X}",
                              self.id, pk.entry_point, pk.stack_ptr,
                              pk.boot_params_addr, pk.cr3);
        
        self.run_loop()?;
        
        Ok(())
    }
    
    
    
    
    
    fn run_loop(&mut self) -> Result<()> {
        let mut gfb = false;
        
        loop {
            
            let result = psv(gfb);
            
            if result != 0 {
                
                let err = vmx::edv(fields::DFA_).unwrap_or(0xFFFF);
                crate::serial_println!("[VM {}] VM entry failed! error={}", self.id, err);
                self.state = VmState::Crashed;
                return if gfb {
                    Err(HypervisorError::VmresumeFailed)
                } else {
                    Err(HypervisorError::VmlaunchFailed)
                };
            }
            
            gfb = true;
            
            
            self.stats.vm_exits += 1;
            
            
            self.load_guest_regs_from_exit();
            
            
            let eix = self.handle_vm_exit()?;
            
            if !eix {
                break;
            }
            
            
            self.save_guest_regs_for_entry();
        }
        
        self.state = VmState::Stopped;
        crate::serial_println!("[VM {}] Stopped after {} exits (cpuid={} io={} hlt={} ept={})",
                              self.id, self.stats.vm_exits, self.stats.cpuid_exits,
                              self.stats.io_exits, self.stats.hlt_exits,
                              self.stats.ept_violations);
        Ok(())
    }
    
    
    fn load_guest_regs_from_exit(&mut self) {
        unsafe {
            let area = &VM_EXIT_GUEST_REGS;
            self.guest_regs.rax = area.rax;
            self.guest_regs.rbx = area.rbx;
            self.guest_regs.rcx = area.rcx;
            self.guest_regs.rdx = area.rdx;
            self.guest_regs.rsi = area.rsi;
            self.guest_regs.rdi = area.rdi;
            self.guest_regs.rbp = area.rbp;
            self.guest_regs.r8  = area.r8;
            self.guest_regs.r9  = area.r9;
            self.guest_regs.r10 = area.r10;
            self.guest_regs.r11 = area.r11;
            self.guest_regs.r12 = area.r12;
            self.guest_regs.r13 = area.r13;
            self.guest_regs.r14 = area.r14;
            self.guest_regs.r15 = area.r15;
        }
    }
    
    
    fn save_guest_regs_for_entry(&self) {
        unsafe {
            let area = &mut VM_EXIT_GUEST_REGS;
            area.rax = self.guest_regs.rax;
            area.rbx = self.guest_regs.rbx;
            area.rcx = self.guest_regs.rcx;
            area.rdx = self.guest_regs.rdx;
            area.rsi = self.guest_regs.rsi;
            area.rdi = self.guest_regs.rdi;
            area.rbp = self.guest_regs.rbp;
            area.r8  = self.guest_regs.r8;
            area.r9  = self.guest_regs.r9;
            area.r10 = self.guest_regs.r10;
            area.r11 = self.guest_regs.r11;
            area.r12 = self.guest_regs.r12;
            area.r13 = self.guest_regs.r13;
            area.r14 = self.guest_regs.r14;
            area.r15 = self.guest_regs.r15;
        }
    }
    
    
    fn handle_vm_exit(&mut self) -> Result<bool> {
        
        let (exit_reason, exit_qual, guest_rip, instr_len) = {
            let vmcs = self.vmcs.as_ref().unwrap();
            let azg = vmcs.read(fields::DEY_)? as u32 & 0xFFFF;
            let oac = vmcs.read(fields::BXN_)?;
            let rip = vmcs.read(fields::FV_)?;
            let len = vmcs.read(fields::DEV_).unwrap_or(0);
            (azg, oac, rip, len)
        };
        
        match exit_reason {
            exit_reason::Rh => {
                self.stats.cpuid_exits += 1;
                crate::lab_mode::trace_bus::bzg(
                    self.id, "CPUID", guest_rip,
                    &alloc::format!("EAX=0x{:X}", self.guest_regs.rax)
                );
                self.handle_cpuid()?;
                
                let vmcs = self.vmcs.as_ref().unwrap();
                vmcs.write(fields::FV_, guest_rip + 2)?;
                Ok(true)
            }
            
            exit_reason::Su => {
                self.stats.hlt_exits += 1;
                
                
                let vmcs = self.vmcs.as_ref().unwrap();
                vmcs.write(fields::FV_, guest_rip + instr_len)?;
                
                
                if self.stats.hlt_exits > 50000 {
                    crate::serial_println!("[VM {}] Too many HLTs ({}), stopping", self.id, self.stats.hlt_exits);
                    Ok(false)
                } else {
                    Ok(true)
                }
            }
            
            exit_reason::CFT_ => {
                self.stats.io_exits += 1;
                let port = ((exit_qual >> 16) & 0xFFFF) as u16;
                let it = if (exit_qual & 8) == 0 { "OUT" } else { "IN" };
                crate::lab_mode::trace_bus::hvk(self.id, it, port, self.guest_regs.rax);
                self.handle_io(exit_qual)?;
                let vmcs = self.vmcs.as_ref().unwrap();
                vmcs.write(fields::FV_, guest_rip + instr_len)?;
                Ok(true)
            }
            
            exit_reason::Ann | exit_reason::Agn => {
                self.stats.msr_exits += 1;
                self.handle_msr(exit_reason == exit_reason::Agn)?;
                let vmcs = self.vmcs.as_ref().unwrap();
                vmcs.write(fields::FV_, guest_rip + instr_len)?;
                Ok(true)
            }
            
            exit_reason::BXE_ => {
                self.stats.ept_violations += 1;
                let vmcs = self.vmcs.as_ref().unwrap();
                let zy = vmcs.read(fields::CBX_)?;
                let drb = vmcs.read(fields::CBV_).ok();
                
                crate::lab_mode::trace_bus::hvl(
                    self.id, "EPT_VIOLATION", zy, exit_qual
                );
                
                
                super::isolation::iyv(
                    self.id,
                    zy,
                    drb,
                    exit_qual,
                    guest_rip,
                );
                
                
                super::api::bzf(
                    super::api::VmEventType::Ev,
                    self.id,
                    super::api::VmEventData::Address(zy),
                );
                
                Ok(false)
            }
            
            exit_reason::Arm => {
                crate::lab_mode::trace_bus::bzg(
                    self.id, "VMCALL", guest_rip,
                    &alloc::format!("func=0x{:X}", self.guest_regs.rax)
                );
                
                let result = self.handle_vmcall()?;
                let vmcs = self.vmcs.as_ref().unwrap();
                vmcs.write(fields::FV_, guest_rip + instr_len)?;
                Ok(result)
            }
            
            exit_reason::DCI_ => {
                crate::lab_mode::trace_bus::bzh(self.id, "TRIPLE FAULT (crashed)");
                crate::serial_println!("[VM {}] TRIPLE FAULT! Guest crashed.", self.id);
                self.state = VmState::Crashed;
                Ok(false)
            }
            
            exit_reason::Wb => {
                
                
                let jrw = self.guest_regs.rcx as u32;
                let jrx = (self.guest_regs.rdx << 32) | (self.guest_regs.rax & 0xFFFF_FFFF);
                
                if jrw == 0 {
                    
                    let safe_value = jrx | 1; 
                    unsafe {
                        core::arch::asm!(
                            "xsetbv",
                            in("ecx") 0u32,
                            in("edx") (safe_value >> 32) as u32,
                            in("eax") safe_value as u32,
                        );
                    }
                    crate::serial_println!("[VM {}] XSETBV XCR0=0x{:X}", self.id, safe_value);
                } else {
                    crate::serial_println!("[VM {}] XSETBV ignored XCR{}=0x{:X}", self.id, jrw, jrx);
                }
                
                let vmcs = self.vmcs.as_ref().unwrap();
                vmcs.write(fields::FV_, guest_rip + instr_len)?;
                Ok(true)
            }
            
            exit_reason::CFK_ => {
                crate::serial_println!("[VM {}] Invalid guest state! Check VMCS.", self.id);
                self.state = VmState::Crashed;
                Ok(false)
            }
            
            _ => {
                crate::serial_println!("[VM {}] Unhandled VM exit reason: {} at RIP=0x{:X}", 
                                      self.id, exit_reason, guest_rip);
                Ok(false)
            }
        }
    }
    
    
    fn handle_cpuid(&mut self) -> Result<()> {
        let leaf = self.guest_regs.rax as u32;
        let subleaf = self.guest_regs.rcx as u32;
        
        
        match leaf {
            0x4000_0000 => {
                
                self.guest_regs.rax = 0x4000_0001;
                self.guest_regs.rbx = 0x7473_7254; 
                self.guest_regs.rcx = 0x7254_534F; 
                self.guest_regs.rdx = 0x534F_7473; 
                return Ok(());
            }
            0x4000_0001 => {
                self.guest_regs.rax = 0;
                self.guest_regs.rbx = 0;
                self.guest_regs.rcx = 0;
                self.guest_regs.rdx = 0;
                return Ok(());
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
        
        let (mut eax, ebx, mut ecx, edx) = (out_eax, out_ebx, out_ecx, out_edx);
        
        match leaf {
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
        
        self.guest_regs.rax = eax as u64;
        self.guest_regs.rbx = ebx as u64;
        self.guest_regs.rcx = ecx as u64;
        self.guest_regs.rdx = edx as u64;
        
        Ok(())
    }
    
    
    fn handle_io(&mut self, exit_qual: u64) -> Result<()> {
        let port = ((exit_qual >> 16) & 0xFFFF) as u16;
        let mtj = (exit_qual & 8) == 0;
        let pxa = (exit_qual & 16) != 0;
        let bek = (exit_qual & 7) as u8 + 1; 
        
        if mtj {
            
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
                    let ch = (value & 0xFF) as u8;
                    crate::serial_print!("{}", ch as char);
                }
                0xED => {} 
                _ => {
                    if self.stats.io_exits < 50 {
                        crate::serial_println!("[VM {}] OUT port 0x{:X} val=0x{:X}", self.id, port, value);
                    }
                }
            }
        } else {
            
            let value: u32 = match port {
                
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
                    let gx = self.stats.vm_exits.wrapping_mul(4);
                    (gx & 0xFFFF_FFFF) as u32
                }
                0xB000..=0xB03F => 0,
                
                0x92 => 0x02,
                
                0xE9 => 0,
                0xED => 0,
                _ => {
                    if self.stats.io_exits < 50 {
                        crate::serial_println!("[VM {}] IN port 0x{:X}", self.id, port);
                    }
                    0xFF
                }
            };
            self.guest_regs.rax = (self.guest_regs.rax & !0xFFFF_FFFF) | (value as u64);
        }
        
        Ok(())
    }
    
    
    fn handle_msr(&mut self, is_write: bool) -> Result<()> {
        let msr = self.guest_regs.rcx as u32;
        
        
        const LD_: u32 = 0x001B;
        const OO_: u32 = 0x01A0;
        const LE_: u32 = 0x0277;
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
            let hdm = (self.guest_regs.rdx << 32) | (self.guest_regs.rax & 0xFFFF_FFFF);
            
            
            match msr {
                WZ_ | WX_ | WT_ | WY_ |
                WU_ | WV_ | WW_ |
                LE_ | IA32_EFER | LD_ | OO_ |
                PG_ => {}
                0x0174..=0x0176 => {} 
                0x0200..=0x020F => {} 
                0x0400..=0x047F => {} 
                _ => {
                    if self.stats.vm_exits < 100 {
                        crate::serial_println!("[VM {}] WRMSR 0x{:X} (ignored)", self.id, msr);
                    }
                }
            }
        } else {
            
            let value: u64 = match msr {
                LD_ => 0xFEE0_0900, 
                OO_ => 1,          
                LE_ => 0x0007040600070406, 
                IA32_EFER => 0x501,             
                PG_ => 0,
                0x00FE => 0,   
                0x0179 => 0,   
                0x017A => 0,   
                0x02FF => 0x06, 
                0x0200..=0x020F => 0, 
                0x0400..=0x047F => 0, 
                _ => {
                    if self.stats.vm_exits < 100 {
                        crate::serial_println!("[VM {}] RDMSR 0x{:X} = 0", self.id, msr);
                    }
                    0
                }
            };
            self.guest_regs.rax = value & 0xFFFF_FFFF;
            self.guest_regs.rdx = value >> 32;
        }
        
        Ok(())
    }
    
    
    fn handle_vmcall(&mut self) -> Result<bool> {
        let function = self.guest_regs.rax;
        
        match function {
            
            0 => {
                crate::serial_println!("[VM {}] Hypercall: print", self.id);
                Ok(true)
            }
            
            
            1 => {
                let exit_code = self.guest_regs.rbx;
                crate::serial_println!("[VM {}] Hypercall: exit (code={})", self.id, exit_code);
                Ok(false)
            }
            
            
            2 => {
                let gx = crate::time::uptime_ms();
                self.guest_regs.rax = gx;
                Ok(true)
            }
            
            
            3 => {
                let c = (self.guest_regs.rbx & 0xFF) as u8;
                super::console::idg(self.id, 0xE9, true, c);
                Ok(true)
            }
            
            
            4 => {
                let c = super::console::idg(self.id, 0x3F8, false, 0);
                self.guest_regs.rax = c as u64;
                Ok(true)
            }
            
            
            0x100..=0x1FF => {
                let hbo = (function - 0x100) as u32;
                let args = [
                    self.guest_regs.rbx,
                    self.guest_regs.rcx,
                    self.guest_regs.rdx,
                    self.guest_regs.rsi,
                ];
                let (result, _data) = super::virtfs::fzs(self.id, hbo, &args);
                self.guest_regs.rax = result as u64;
                Ok(true)
            }
            
            
            0x200..=0x3FF => {
                let args = [
                    self.guest_regs.rbx,
                    self.guest_regs.rcx,
                    self.guest_regs.rdx,
                    self.guest_regs.rsi,
                ];
                let (result, data) = super::api::mhg(self.id, function, &args);
                
                
                if result == -1 && function == super::api::hypercall::Iu {
                    
                    return Ok(false);
                }
                if result == -2 && function == super::api::hypercall::Um {
                    
                    return Ok(false);
                }
                
                self.guest_regs.rax = data;
                Ok(true)
            }
            
            _ => {
                crate::serial_println!("[VM {}] Unknown hypercall: 0x{:X}", self.id, function);
                self.guest_regs.rax = u64::MAX; 
                Ok(true)
            }
        }
    }
}


static Vs: Mutex<Vec<VirtualMachine>> = Mutex::new(Vec::new());


pub fn blh(id: u64, name: &str, memory_mb: usize) -> Result<()> {
    let vm = VirtualMachine::new(id, name, memory_mb)?;
    Vs.lock().push(vm);
    Ok(())
}


pub fn jil(id: u64) -> Result<()> {
    dev(id, "hello")
}


pub fn dev(id: u64, guest_name: &str) -> Result<()> {
    let mut aen = Vs.lock();
    
    for vm in aen.iter_mut() {
        if vm.id == id {
            
            if guest_name == "linux-test" || guest_name.ends_with(".bzimage") {
                let jx = super::guests::eoc(guest_name)
                    .unwrap_or_else(|| super::linux_loader::fpb());
                crate::serial_println!("[VM {}] Loading Linux guest '{}' ({} bytes)", 
                                      id, guest_name, jx.len());
                vm.start_linux(&jx, "console=ttyS0 earlyprintk=serial nokaslr", None)?;
                return Ok(());
            }
            
            
            let code = super::guests::eoc(guest_name)
                .unwrap_or_else(|| super::guests::ieq());
            
            crate::serial_println!("[VM {}] Loading guest '{}' ({} bytes)", id, guest_name, code.len());
            
            vm.load_binary(&code, 0x1000)?;
            vm.start(0x1000, 0x8000)?;
            return Ok(());
        }
    }
    
    Err(HypervisorError::VmNotFound)
}


pub fn fbu(id: u64) -> Result<()> {
    let mut aen = Vs.lock();
    
    for vm in aen.iter_mut() {
        if vm.id == id {
            vm.state = VmState::Stopped;
            return Ok(());
        }
    }
    
    Err(HypervisorError::VmNotFound)
}





use core::sync::atomic::{AtomicBool, AtomicU8, Ordering};


static mut VM_EXIT_GUEST_REGS: GuestRegs = GuestRegs {
    rax: 0, rbx: 0, rcx: 0, rdx: 0,
    rsi: 0, rdi: 0, rbp: 0,
    r8: 0, r9: 0, r10: 0, r11: 0,
    r12: 0, r13: 0, r14: 0, r15: 0,
};



static mut HOST_SAVED_RSP: u64 = 0;


static VMX_USE_RESUME: AtomicU8 = AtomicU8::new(0);













fn psv(use_resume: bool) -> u64 {
    VMX_USE_RESUME.store(use_resume as u8, Ordering::SeqCst);
    unsafe { psw() }
}



#[unsafe(naked)]
unsafe extern "C" fn psw() -> u64 {
    core::arch::naked_asm!(
        
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
        
        host_rsp = sym HOST_SAVED_RSP,
        gregs = sym VM_EXIT_GUEST_REGS,
        flag = sym VMX_USE_RESUME,
    );
}











#[unsafe(naked)]
extern "C" fn jqi() {
    core::arch::naked_asm!(
        
        
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
        
        gregs = sym VM_EXIT_GUEST_REGS,
        host_rsp = sym HOST_SAVED_RSP,
    );
}
