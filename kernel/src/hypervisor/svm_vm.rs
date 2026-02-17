//! SVM VM (Virtual Machine) Management for AMD
//!
//! AMD SVM equivalent of vm.rs for Intel VMX:
//! - VM creation and destruction
//! - VMCB management
//! - NPT (Nested Page Tables) setup
//! - #VMEXIT handling

use alloc::string::String;
use alloc::vec::Vec;
use alloc::boxed::Box;
use alloc::format;
use core::sync::atomic::{AtomicU64, Ordering};
use spin::Mutex;

use super::{HypervisorError, Result};
use super::svm::{self, SvmExitCode, SvmFeatures, VmrunGuestRegs};
use super::svm::vmcb::{Vmcb, control_offsets, state_offsets};
use super::svm::npt::{Npt, flags as npt_flags};

/// VM ID counter
static NEXT_VM_ID: AtomicU64 = AtomicU64::new(1);

/// SVM VM state
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SvmVmState {
    Created,
    Running,
    Paused,
    Stopped,
    Crashed,
}

/// SVM VM statistics
#[derive(Debug, Clone, Default)]
pub struct SvmVmStats {
    pub vmexits: u64,
    pub cpuid_exits: u64,
    pub io_exits: u64,
    pub msr_exits: u64,
    pub hlt_exits: u64,
    pub npf_exits: u64,     // Nested Page Faults
    pub vmmcall_exits: u64, // Hypercalls
    pub intr_exits: u64,    // Interrupt exits
}

/// Guest registers saved/restored on VM exit/entry
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
    pub rsp: u64,  // Saved separately from VMCB
}

/// AMD SVM Virtual Machine
pub struct SvmVirtualMachine {
    /// Unique VM ID
    pub id: u64,
    /// VM name
    pub name: String,
    /// Current state
    pub state: SvmVmState,
    /// Memory size in bytes
    pub memory_size: usize,
    /// Statistics
    pub stats: SvmVmStats,
    /// VMCB (Virtual Machine Control Block)
    pub(crate) vmcb: Option<Box<Vmcb>>,
    /// NPT (Nested Page Tables)
    npt: Option<Npt>,
    /// Guest physical memory
    guest_memory: Vec<u8>,
    /// Guest general-purpose registers
    pub(crate) guest_regs: SvmGuestRegs,
    /// ASID for TLB isolation
    pub asid: u32,
    /// Console ID for output
    console_id: Option<usize>,
    /// SVM features available
    features: SvmFeatures,
}

impl SvmVirtualMachine {
    /// Create a new SVM VM
    pub fn new(name: &str, memory_mb: usize) -> Result<Self> {
        // Check SVM support
        if !svm::is_supported() {
            return Err(HypervisorError::SvmNotSupported);
        }
        
        let id = NEXT_VM_ID.fetch_add(1, Ordering::SeqCst);
        let memory_size = memory_mb * 1024 * 1024;
        let features = svm::get_features();
        
        // Allocate guest memory
        let guest_memory = alloc::vec![0u8; memory_size];
        
        // Allocate ASID (Address Space ID) for TLB isolation
        let asid = super::svm::npt::allocate_asid().unwrap_or(1);
        
        // Create virtual console
        let console_id = super::console::create_console(id, name);
        
        // Create VirtFS for this VM
        super::virtfs::create_virtfs(id);
        
        crate::serial_println!("[SVM-VM {}] Created '{}' with {} MB RAM, ASID={}", 
                              id, name, memory_mb, asid);
        
        // Emit to TrustLab trace bus
        crate::lab_mode::trace_bus::emit_vm_lifecycle(
            id, &format!("CREATED '{}' mem={}MB ASID={}", name, memory_mb, asid)
        );
        
        // Emit creation event
        super::api::emit_event(
            super::api::VmEventType::Created,
            id,
            super::api::VmEventData::Message(format!("SVM VM '{}' created", name)),
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
        })
    }
    
    /// Initialize VMCB and NPT
    pub fn initialize(&mut self) -> Result<()> {
        crate::serial_println!("[SVM-VM {}] Initializing VMCB and NPT...", self.id);
        
        // Create and setup VMCB
        let mut vmcb = Vmcb::new();
        
        // Setup basic intercepts (includes IOIO now)
        vmcb.setup_basic_intercepts();
        
        // Set ASID (must be non-zero for guest)
        vmcb.write_control(control_offsets::GUEST_ASID, self.asid as u64);
        
        // Allocate IOPM (I/O Permission Map) — 12 KB (3 pages)
        // All bits = 1 means intercept all I/O ports
        let iopm = alloc::vec![0xFFu8; 12288]; // 12KB, all 1s = intercept everything
        let iopm_ptr = iopm.as_ptr() as u64;
        let iopm_phys = iopm_ptr - crate::memory::hhdm_offset();
        vmcb.set_iopm_base(iopm_phys);
        // Leak the IOPM allocation so it lives for the duration of the VM
        core::mem::forget(iopm);
        crate::serial_println!("[SVM-VM {}] IOPM allocated at HPA=0x{:X}", self.id, iopm_phys);
        
        // Setup NPT if supported
        if self.features.npt {
            let mut npt = Npt::new(self.asid);
            
            // Map guest memory 1:1
            let guest_mem_ptr = self.guest_memory.as_ptr() as u64;
            let guest_mem_phys = guest_mem_ptr - crate::memory::hhdm_offset();
            
            if let Err(e) = npt.map_range(
                0,                          // Guest physical address
                guest_mem_phys,             // Host physical address
                self.memory_size as u64,    // Size
                npt_flags::RWX,             // Permissions
            ) {
                crate::serial_println!("[SVM-VM {}] NPT mapping failed: {}", self.id, e);
                return Err(HypervisorError::NptViolation);
            }
            
            // Get NPT CR3 (PML4 physical address)
            let npt_cr3 = npt.cr3();
            
            // Enable NPT in VMCB
            // N_CR3 = NPT base address
            vmcb.write_control(control_offsets::N_CR3, npt_cr3);
            
            // Enable NPT bit in VMCB control area
            let mut np_enable = vmcb.read_control(control_offsets::NP_ENABLE);
            np_enable |= 1; // Bit 0 = enable nested paging
            vmcb.write_control(control_offsets::NP_ENABLE, np_enable);
            
            crate::serial_println!("[SVM-VM {}] NPT enabled, N_CR3=0x{:X}", self.id, npt_cr3);
            
            self.npt = Some(npt);
        } else {
            // Shadow paging mode (legacy)
            crate::serial_println!("[SVM-VM {}] NPT not available, using shadow paging", self.id);
        }
        
        // Setup TLB control for ASID
        vmcb.write_control(control_offsets::TLB_CONTROL, 0); // No flush on VMRUN
        
        self.vmcb = Some(Box::new(vmcb));
        
        crate::serial_println!("[SVM-VM {}] Initialization complete", self.id);
        
        Ok(())
    }
    
    /// Load binary into guest memory
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
    
    /// Setup guest state in VMCB for real mode boot
    pub fn setup_real_mode(&mut self, entry_point: u64) -> Result<()> {
        let vmcb = self.vmcb.as_mut().ok_or(HypervisorError::VmcbNotLoaded)?;
        vmcb.setup_real_mode();
        
        // Set entry point (CS:IP style for real mode)
        let cs_base = (entry_point >> 4) << 4;
        let ip = entry_point & 0xF;
        
        vmcb.write_state(state_offsets::CS_BASE, cs_base);
        vmcb.write_state(state_offsets::CS_SELECTOR, (cs_base >> 4) as u64);
        vmcb.write_state(state_offsets::RIP, ip);
        
        crate::serial_println!("[SVM-VM {}] Real mode: CS=0x{:X}, IP=0x{:X}", 
                              self.id, cs_base >> 4, ip);
        
        Ok(())
    }
    
    /// Setup guest state in VMCB for protected mode
    pub fn setup_protected_mode(&mut self, entry_point: u64, stack_ptr: u64) -> Result<()> {
        let vmcb = self.vmcb.as_mut().ok_or(HypervisorError::VmcbNotLoaded)?;
        vmcb.setup_protected_mode(entry_point);
        
        vmcb.write_state(state_offsets::RIP, entry_point);
        vmcb.write_state(state_offsets::RSP, stack_ptr);
        
        crate::serial_println!("[SVM-VM {}] Protected mode: RIP=0x{:X}, RSP=0x{:X}", 
                              self.id, entry_point, stack_ptr);
        
        Ok(())
    }
    
    /// Setup guest state for Linux kernel boot (protected mode with boot_params)
    pub fn setup_protected_mode_for_linux(&mut self, entry_point: u64, stack_ptr: u64, boot_params_addr: u64) -> Result<()> {
        let vmcb = self.vmcb.as_mut().ok_or(HypervisorError::VmcbNotLoaded)?;
        vmcb.setup_protected_mode(entry_point);
        
        vmcb.write_state(state_offsets::RIP, entry_point);
        vmcb.write_state(state_offsets::RSP, stack_ptr);
        
        // Linux kernel expects boot_params pointer in RSI
        // Since RSI is not in VMCB state save area, we set it in guest_regs
        // which will be loaded before VMRUN
        self.guest_regs.rsi = boot_params_addr;
        
        // Also set some other commonly expected values
        self.guest_regs.rbp = 0;
        self.guest_regs.rdi = 0;
        
        crate::serial_println!("[SVM-VM {}] Linux protected mode: RIP=0x{:X}, RSP=0x{:X}, RSI(boot_params)=0x{:X}", 
                              self.id, entry_point, stack_ptr, boot_params_addr);
        
        Ok(())
    }

    /// Setup guest state in VMCB for long mode (64-bit)
    pub fn setup_long_mode(&mut self, entry_point: u64, stack_ptr: u64) -> Result<()> {
        // Get NPT CR3 for guest paging
        let guest_cr3 = self.npt.as_ref().map(|n| n.cr3()).unwrap_or(0);
        
        let vmcb = self.vmcb.as_mut().ok_or(HypervisorError::VmcbNotLoaded)?;
        vmcb.setup_long_mode(entry_point, guest_cr3);
        
        vmcb.write_state(state_offsets::RIP, entry_point);
        vmcb.write_state(state_offsets::RSP, stack_ptr);
        
        crate::serial_println!("[SVM-VM {}] Long mode: RIP=0x{:X}, RSP=0x{:X}", 
                              self.id, entry_point, stack_ptr);
        
        Ok(())
    }
    
    /// Start the VM
    pub fn start(&mut self) -> Result<()> {
        if self.vmcb.is_none() {
            self.initialize()?;
        }
        
        self.state = SvmVmState::Running;
        
        crate::serial_println!("[SVM-VM {}] Starting execution...", self.id);
        crate::lab_mode::trace_bus::emit_vm_lifecycle(self.id, "STARTED");
        
        // Run the VM loop
        self.run_loop()?;
        
        Ok(())
    }
    
    /// VM execution loop
    fn run_loop(&mut self) -> Result<()> {
        // Get VMCB physical address first
        let vmcb_phys = {
            let vmcb = self.vmcb.as_ref().ok_or(HypervisorError::VmcbNotLoaded)?;
            vmcb.phys_addr()
        };
        
        // Create VmrunGuestRegs from our guest_regs for vmrun_with_regs
        let mut vmrun_regs = VmrunGuestRegs {
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
        
        crate::serial_println!("[SVM-VM {}] Entering VM loop, RSI=0x{:X}", self.id, vmrun_regs.rsi);
        
        loop {
            if self.state != SvmVmState::Running {
                break;
            }
            
            // Sync RAX and RSP with VMCB (these are stored in VMCB state save area)
            {
                let vmcb = self.vmcb.as_mut().ok_or(HypervisorError::VmcbNotLoaded)?;
                // RAX is saved/loaded by VMCB automatically
                vmcb.write_state(state_offsets::RSP, self.guest_regs.rsp);
            }
            
            // Execute VMRUN with guest registers
            // CLGI - Clear Global Interrupt Flag (disable interrupts during VM entry)
            unsafe { svm::clgi(); }
            
            // VMRUN - Enter guest with all GPRs loaded
            unsafe {
                svm::vmrun_with_regs(vmcb_phys, &mut vmrun_regs);
            }
            
            // STGI - Set Global Interrupt Flag (re-enable interrupts)
            unsafe { svm::stgi(); }
            
            // Copy back registers from vmrun_regs to guest_regs
            self.guest_regs.rax = vmrun_regs.rax;
            self.guest_regs.rbx = vmrun_regs.rbx;
            self.guest_regs.rcx = vmrun_regs.rcx;
            self.guest_regs.rdx = vmrun_regs.rdx;
            self.guest_regs.rsi = vmrun_regs.rsi;
            self.guest_regs.rdi = vmrun_regs.rdi;
            self.guest_regs.rbp = vmrun_regs.rbp;
            self.guest_regs.r8 = vmrun_regs.r8;
            self.guest_regs.r9 = vmrun_regs.r9;
            self.guest_regs.r10 = vmrun_regs.r10;
            self.guest_regs.r11 = vmrun_regs.r11;
            self.guest_regs.r12 = vmrun_regs.r12;
            self.guest_regs.r13 = vmrun_regs.r13;
            self.guest_regs.r14 = vmrun_regs.r14;
            self.guest_regs.r15 = vmrun_regs.r15;
            
            // Get RSP from VMCB
            {
                let vmcb = self.vmcb.as_ref().ok_or(HypervisorError::VmcbNotLoaded)?;
                self.guest_regs.rsp = vmcb.read_state(state_offsets::RSP);
            }
            
            self.stats.vmexits += 1;
            
            // Handle #VMEXIT
            let continue_running = self.handle_vmexit_inline()?;
            
            // Emit register snapshot to trace bus periodically (for VM Inspector)
            if self.stats.vmexits % 50 == 0 || !continue_running {
                let rip = {
                    let vmcb = self.vmcb.as_ref().ok_or(HypervisorError::VmcbNotLoaded)?;
                    vmcb.read_state(state_offsets::RIP)
                };
                crate::lab_mode::trace_bus::emit_vm_regs(
                    self.id,
                    self.guest_regs.rax,
                    self.guest_regs.rbx,
                    self.guest_regs.rcx,
                    self.guest_regs.rdx,
                    rip,
                    self.guest_regs.rsp,
                );
            }
            
            if !continue_running {
                break;
            }
            
            // Sync back to vmrun_regs for next iteration
            vmrun_regs.rax = self.guest_regs.rax;
            vmrun_regs.rbx = self.guest_regs.rbx;
            vmrun_regs.rcx = self.guest_regs.rcx;
            vmrun_regs.rdx = self.guest_regs.rdx;
            vmrun_regs.rsi = self.guest_regs.rsi;
            vmrun_regs.rdi = self.guest_regs.rdi;
            vmrun_regs.rbp = self.guest_regs.rbp;
            vmrun_regs.r8 = self.guest_regs.r8;
            vmrun_regs.r9 = self.guest_regs.r9;
            vmrun_regs.r10 = self.guest_regs.r10;
            vmrun_regs.r11 = self.guest_regs.r11;
            vmrun_regs.r12 = self.guest_regs.r12;
            vmrun_regs.r13 = self.guest_regs.r13;
            vmrun_regs.r14 = self.guest_regs.r14;
            vmrun_regs.r15 = self.guest_regs.r15;
        }
        
        if self.state == SvmVmState::Running {
            self.state = SvmVmState::Stopped;
        }
        
        crate::serial_println!("[SVM-VM {}] Stopped after {} VMEXITs", self.id, self.stats.vmexits);
        crate::lab_mode::trace_bus::emit_vm_lifecycle(
            self.id, &format!("STOPPED after {} exits (cpuid={} io={} msr={} hlt={} vmcall={})",
                self.stats.vmexits, self.stats.cpuid_exits, self.stats.io_exits,
                self.stats.msr_exits, self.stats.hlt_exits, self.stats.vmmcall_exits)
        );
        
        Ok(())
    }
    
    /// Handle #VMEXIT (inline version to avoid borrow issues)
    fn handle_vmexit_inline(&mut self) -> Result<bool> {
        // Read exit information
        let (exit_code, exit_info1, exit_info2, guest_rip, next_rip) = {
            let vmcb = self.vmcb.as_ref().ok_or(HypervisorError::VmcbNotLoaded)?;
            let ec = vmcb.read_control(control_offsets::EXITCODE);
            let ei1 = vmcb.read_control(control_offsets::EXITINFO1);
            let ei2 = vmcb.read_control(control_offsets::EXITINFO2);
            let rip = vmcb.read_state(state_offsets::RIP);
            let nrip = if self.features.nrip_save {
                vmcb.read_control(control_offsets::NEXT_RIP)
            } else {
                rip + 2 // Assume 2-byte instruction as fallback
            };
            (ec, ei1, ei2, rip, nrip)
        };
        
        let exit = SvmExitCode::from(exit_code);
        
        match exit {
            SvmExitCode::Cpuid => {
                self.stats.cpuid_exits += 1;
                // Emit to TrustLab before handling
                crate::lab_mode::trace_bus::emit_vm_exit(
                    self.id, "CPUID", guest_rip,
                    &alloc::format!("EAX=0x{:X} ECX=0x{:X}", self.guest_regs.rax, self.guest_regs.rcx)
                );
                self.handle_cpuid();
                let vmcb = self.vmcb.as_mut().ok_or(HypervisorError::VmcbNotLoaded)?;
                vmcb.write_state(state_offsets::RIP, next_rip);
                Ok(true)
            }
            
            SvmExitCode::Hlt => {
                self.stats.hlt_exits += 1;
                crate::lab_mode::trace_bus::emit_vm_exit(self.id, "HLT", guest_rip, "");
                // For Linux, HLT is used for idle loop - inject a timer interrupt
                // to wake the guest up and skip the HLT
                {
                    let vmcb = self.vmcb.as_mut().ok_or(HypervisorError::VmcbNotLoaded)?;
                    vmcb.write_state(state_offsets::RIP, next_rip);
                    // Inject virtual timer interrupt via EVENT_INJ
                    // Format: [31:0] = vector:8 | type:3 | EV:1 | rsvd:19 | V:1
                    // type 0 = external interrupt, V=1 = valid
                    // Only inject periodically to avoid flooding
                    if self.stats.hlt_exits % 100 == 0 {
                        let event_inj: u64 = 0x20        // Vector 0x20 (timer)
                                           | (0 << 8)    // Type 0 = external interrupt
                                           | (1u64 << 31); // V = valid
                        vmcb.write_control(control_offsets::EVENT_INJ, event_inj);
                    }
                }
                
                // If we've done too many HLTs without progress, stop
                if self.stats.hlt_exits > 100_000 {
                    crate::serial_println!("[SVM-VM {}] Too many HLT exits ({}), stopping", self.id, self.stats.hlt_exits);
                    self.state = SvmVmState::Stopped;
                    Ok(false)
                } else {
                    Ok(true)
                }
            }
            
            SvmExitCode::IoioIn | SvmExitCode::IoioOut => {
                self.stats.io_exits += 1;
                let port = ((exit_info1 >> 16) & 0xFFFF) as u16;
                let dir = if matches!(exit, SvmExitCode::IoioIn) { "IN" } else { "OUT" };
                crate::lab_mode::trace_bus::emit_vm_io(self.id, dir, port, self.guest_regs.rax);
                self.handle_io(exit_info1);
                let vmcb = self.vmcb.as_mut().ok_or(HypervisorError::VmcbNotLoaded)?;
                vmcb.write_state(state_offsets::RIP, next_rip);
                Ok(true)
            }
            
            SvmExitCode::MsrRead | SvmExitCode::MsrWrite => {
                self.stats.msr_exits += 1;
                let is_write = matches!(exit, SvmExitCode::MsrWrite);
                let msr_dir = if is_write { "WRMSR" } else { "RDMSR" };
                crate::lab_mode::trace_bus::emit_vm_exit(
                    self.id, msr_dir, guest_rip,
                    &alloc::format!("MSR=0x{:X}", self.guest_regs.rcx)
                );
                self.handle_msr(is_write);
                let vmcb = self.vmcb.as_mut().ok_or(HypervisorError::VmcbNotLoaded)?;
                vmcb.write_state(state_offsets::RIP, next_rip);
                Ok(true)
            }
            
            SvmExitCode::NpfFault => {
                self.stats.npf_exits += 1;
                let guest_phys = exit_info2;
                let error_code = exit_info1;
                crate::lab_mode::trace_bus::emit_vm_memory(
                    self.id, "NPF_VIOLATION", guest_phys, error_code
                );
                crate::serial_println!("[SVM-VM {}] NPF: GPA=0x{:X}, Error=0x{:X}", 
                                      self.id, guest_phys, error_code);
                
                // Record violation
                super::isolation::record_violation(
                    self.id,
                    guest_phys,
                    None,
                    error_code,
                    guest_rip,
                );
                
                self.state = SvmVmState::Crashed;
                Ok(false)
            }
            
            SvmExitCode::Vmmcall => {
                self.stats.vmmcall_exits += 1;
                crate::lab_mode::trace_bus::emit_vm_exit(
                    self.id, "VMMCALL", guest_rip,
                    &alloc::format!("func=0x{:X} args=({:X},{:X},{:X})",
                        self.guest_regs.rax, self.guest_regs.rbx,
                        self.guest_regs.rcx, self.guest_regs.rdx)
                );
                let should_continue = self.handle_vmmcall();
                let vmcb = self.vmcb.as_mut().ok_or(HypervisorError::VmcbNotLoaded)?;
                vmcb.write_state(state_offsets::RIP, next_rip);
                Ok(should_continue)
            }
            
            SvmExitCode::Shutdown => {
                crate::lab_mode::trace_bus::emit_vm_lifecycle(self.id, "TRIPLE FAULT (shutdown)");
                crate::serial_println!("[SVM-VM {}] Guest SHUTDOWN (triple fault)", self.id);
                self.state = SvmVmState::Crashed;
                Ok(false)
            }
            
            SvmExitCode::Intr => {
                self.stats.intr_exits += 1;
                // External interrupt - just continue (don't emit, too noisy)
                Ok(true)
            }
            
            // CR write intercepts — allow the write and continue
            SvmExitCode::WriteCr0 => {
                // Guest wants to write CR0 (e.g., enabling paging, FPU setup)
                // The new value is in exit_info1 (for MOV-to-CR) or we need to decode
                // With NPT enabled, most CR0 writes are safe to allow
                let new_cr0 = exit_info1;
                crate::lab_mode::trace_bus::emit_vm_exit(
                    self.id, "WRITE_CR0", guest_rip,
                    &alloc::format!("val=0x{:X}", new_cr0)
                );
                {
                    let vmcb = self.vmcb.as_mut().ok_or(HypervisorError::VmcbNotLoaded)?;
                    // Write the value the guest wants into the VMCB CR0
                    // Ensure PE (bit 0) stays set in protected mode
                    let safe_cr0 = new_cr0 | 0x10; // Keep ET bit set
                    vmcb.set_cr0(safe_cr0);
                    vmcb.write_state(state_offsets::RIP, next_rip);
                }
                Ok(true)
            }
            
            SvmExitCode::WriteCr3 => {
                // Guest is switching page tables (e.g., context switch)
                let new_cr3 = exit_info1;
                {
                    let vmcb = self.vmcb.as_mut().ok_or(HypervisorError::VmcbNotLoaded)?;
                    vmcb.set_cr3(new_cr3);
                    vmcb.write_state(state_offsets::RIP, next_rip);
                }
                Ok(true)
            }
            
            SvmExitCode::WriteCr4 => {
                // Guest writing CR4 (PAE, PSE, etc.)
                let new_cr4 = exit_info1;
                crate::lab_mode::trace_bus::emit_vm_exit(
                    self.id, "WRITE_CR4", guest_rip,
                    &alloc::format!("val=0x{:X}", new_cr4)
                );
                {
                    let vmcb = self.vmcb.as_mut().ok_or(HypervisorError::VmcbNotLoaded)?;
                    vmcb.set_cr4(new_cr4);
                    vmcb.write_state(state_offsets::RIP, next_rip);
                }
                Ok(true)
            }
            
            // CR read intercepts — let them read the current VMCB value
            SvmExitCode::ReadCr0 | SvmExitCode::ReadCr3 | SvmExitCode::ReadCr4 => {
                // The hardware handles CR reads via VMCB state save area
                // Just advance RIP
                let vmcb = self.vmcb.as_mut().ok_or(HypervisorError::VmcbNotLoaded)?;
                vmcb.write_state(state_offsets::RIP, next_rip);
                Ok(true)
            }
            
            SvmExitCode::Cr0SelWrite => {
                // Selective CR0 write (LMSW, CLTS) — allow it
                let vmcb = self.vmcb.as_mut().ok_or(HypervisorError::VmcbNotLoaded)?;
                vmcb.write_state(state_offsets::RIP, next_rip);
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
    
    /// Handle CPUID exit
    fn handle_cpuid(&mut self) {
        let eax = self.guest_regs.rax as u32;
        let ecx = self.guest_regs.rcx as u32;
        
        // Execute real CPUID
        let (out_eax, out_ebx, out_ecx, out_edx): (u32, u32, u32, u32);
        
        unsafe {
            core::arch::asm!(
                "push rbx",
                "cpuid",
                "mov {out_ebx:e}, ebx",
                "pop rbx",
                inout("eax") eax => out_eax,
                inout("ecx") ecx => out_ecx,
                out_ebx = out(reg) out_ebx,
                out("edx") out_edx,
            );
        }
        
        // Hide SVM from guest (optional - for security)
        let out_ecx = if eax == 0x8000_0001 {
            out_ecx & !(1 << 2) // Clear SVM bit
        } else {
            out_ecx
        };
        
        // Write results back to guest
        self.guest_regs.rax = out_eax as u64;
        self.guest_regs.rbx = out_ebx as u64;
        self.guest_regs.rcx = out_ecx as u64;
        self.guest_regs.rdx = out_edx as u64;
    }
    
    /// Handle I/O instruction exit
    fn handle_io(&mut self, info1: u64) {
        let is_in = (info1 & 1) != 0;
        let port = ((info1 >> 16) & 0xFFFF) as u16;
        let _size = match (info1 >> 4) & 0x7 {
            0 => 1, // Byte
            1 => 2, // Word
            2 => 4, // Dword
            _ => 1,
        };
        
        if is_in {
            // IN instruction - return appropriate data for each port
            let value: u32 = match port {
                // ── COM1 serial port (0x3F8-0x3FF) ───────────────────
                0x3F8 => 0,                     // Data register - no data available
                0x3F9 => 0,                     // Interrupt enable register
                0x3FA => 0xC1,                  // IIR: FIFO enabled, no interrupt pending
                0x3FB => 0x03,                  // LCR: 8N1
                0x3FC => 0x03,                  // MCR: DTR + RTS
                0x3FD => 0x60,                  // LSR: TX empty + TX holding empty
                0x3FE => 0xB0,                  // MSR: CTS + DSR
                0x3FF => 0,                     // Scratch register
                
                // ── COM2 serial (0x2F8-0x2FF) ─────────────────────────
                0x2F8..=0x2FF => match port & 0x7 {
                    5 => 0x60, // LSR: TX empty
                    _ => 0,
                },
                
                // ── Keyboard controller (8042) ────────────────────────
                0x60 => 0,                      // Keyboard data
                0x64 => 0x1C,                   // Status: input buffer empty, self-test passed
                
                // ── PIC (8259A) ───────────────────────────────────
                0x20 => 0,                      // Master PIC: command/status
                0x21 => 0xFF,                   // Master PIC: data (all IRQs masked)
                0xA0 => 0,                      // Slave PIC: command/status
                0xA1 => 0xFF,                   // Slave PIC: data (all IRQs masked)
                
                // ── PIT (8254) timer ──────────────────────────────
                0x40 => 0,                      // Counter 0 (system timer)
                0x41 => 0,                      // Counter 1 (refresh)
                0x42 => 0,                      // Counter 2 (speaker)
                0x43 => 0,                      // Control word
                0x61 => 0x20,                   // NMI status / speaker control
                
                // ── CMOS/RTC ──────────────────────────────────────
                0x70 => 0,
                0x71 => {
                    // Return sensible CMOS values
                    0x00 // Default: 0
                }
                
                // ── DMA controllers ───────────────────────────────
                0x00..=0x0F | 0x80..=0x8F | 0xC0..=0xDF => 0,
                
                // ── VGA / display ─────────────────────────────────
                0x3B0..=0x3DF => 0,             // VGA registers
                
                // ── PCI config space ──────────────────────────────
                0xCF8 => 0xFFFF_FFFF,           // PCI config addr (no devices)
                0xCFC..=0xCFF => 0xFFFF_FFFF,   // PCI config data (no devices)
                
                // ── ACPI / power management ───────────────────────
                0xB000..=0xB03F => 0,           // ACPI PM base
                
                // ── Debug / misc ──────────────────────────────────
                0xE9 => 0,                      // Bochs debug port
                0xED => 0,                      // I/O delay port
                0x92 => 0x02,                   // Fast A20 gate (A20 enabled)
                
                _ => {
                    // Unknown port — log first few then be silent
                    if self.stats.io_exits < 50 {
                        crate::serial_println!("[SVM-VM {}] Unhandled IN port 0x{:X}", self.id, port);
                    }
                    0xFF
                }
            };
            self.guest_regs.rax = (self.guest_regs.rax & !0xFFFF_FFFF) | (value as u64);
        } else {
            // OUT instruction
            let value = self.guest_regs.rax as u32;
            
            match port {
                // Serial output — write directly to physical serial port
                0x3F8 => {
                    let ch = (value & 0xFF) as u8;
                    crate::serial_print!("{}", ch as char);
                    if let Some(console_id) = self.console_id {
                        super::console::write_char(console_id, ch as char);
                    }
                }
                // COM2 serial output
                0x2F8 => {
                    let ch = (value & 0xFF) as u8;
                    crate::serial_print!("{}", ch as char);
                }
                // Serial config writes — accept silently
                0x3F9..=0x3FF | 0x2F9..=0x2FF => {}
                
                // PIC programming — accept silently (ICW1-ICW4, OCW1-OCW3)
                0x20 | 0x21 | 0xA0 | 0xA1 => {}
                
                // PIT programming — accept silently
                0x40..=0x43 => {}
                
                // CMOS — accept silently
                0x70 | 0x71 => {}
                
                // NMI / speaker control
                0x61 => {}
                
                // Keyboard controller commands
                0x60 | 0x64 => {}
                
                // DMA
                0x00..=0x0F | 0x80..=0x8F | 0xC0..=0xDF => {}
                
                // VGA
                0x3B0..=0x3DF => {}
                
                // PCI config
                0xCF8..=0xCFF => {}
                
                // Fast A20 gate
                0x92 => {}
                
                // Debug ports
                0xE9 => {
                    let ch = (value & 0xFF) as u8;
                    crate::serial_print!("{}", ch as char);
                }
                0xED => {} // I/O delay, do nothing
                
                // ACPI
                0xB000..=0xB03F => {}
                
                _ => {
                    if self.stats.io_exits < 50 {
                        crate::serial_println!("[SVM-VM {}] Unhandled OUT port 0x{:X} val=0x{:X}", self.id, port, value);
                    }
                }
            }
        }
    }
    
    /// Handle MSR access exit
    fn handle_msr(&mut self, is_write: bool) {
        let msr = self.guest_regs.rcx as u32;
        
        if is_write {
            let value = (self.guest_regs.rdx << 32) | (self.guest_regs.rax & 0xFFFF_FFFF);
            crate::serial_println!("[SVM-VM {}] WRMSR 0x{:X} = 0x{:X}", self.id, msr, value);
            // Silently ignore writes
        } else {
            // RDMSR - return zeros for unknown MSRs
            self.guest_regs.rax = 0;
            self.guest_regs.rdx = 0;
        }
    }
    
    /// Handle VMMCALL (hypercall) exit
    fn handle_vmmcall(&mut self) -> bool {
        let function = self.guest_regs.rax;
        let arg1 = self.guest_regs.rbx;
        let arg2 = self.guest_regs.rcx;
        let arg3 = self.guest_regs.rdx;
        
        crate::serial_println!("[SVM-VM {}] VMMCALL: func=0x{:X}, args=({:X}, {:X}, {:X})", 
                              self.id, function, arg1, arg2, arg3);
        
        let (result, should_continue): (i64, bool) = match function {
            // Exit VM
            0x00 => {
                self.state = SvmVmState::Stopped;
                (0, false)
            }
            
            // Print string (arg1 = GPA of string)
            0x01 => {
                self.hypercall_print(arg1);
                (0, true)
            }
            
            // Get time (returns TSC)
            0x02 => {
                (unsafe { core::arch::x86_64::_rdtsc() as i64 }, true)
            }
            
            _ => (-1, true), // Unknown hypercall
        };
        
        // Return result in RAX
        self.guest_regs.rax = result as u64;
        
        // Emit event
        super::api::emit_event(
            super::api::VmEventType::Hypercall,
            self.id,
            super::api::VmEventData::HypercallInfo { function, result },
        );
        
        should_continue
    }
    
    /// Hypercall: print string from guest memory
    fn hypercall_print(&self, gpa: u64) {
        let offset = gpa as usize;
        if offset < self.guest_memory.len() {
            // Find null terminator
            let max_len = (self.guest_memory.len() - offset).min(256);
            let slice = &self.guest_memory[offset..offset + max_len];
            
            if let Some(null_pos) = slice.iter().position(|&c| c == 0) {
                if let Ok(s) = core::str::from_utf8(&slice[..null_pos]) {
                    crate::serial_println!("[SVM-VM {} PRINT] {}", self.id, s);
                    if let Some(console_id) = self.console_id {
                        for ch in s.chars() {
                            super::console::write_char(console_id, ch);
                        }
                    }
                }
            }
        }
    }
    
    /// Pause the VM
    pub fn pause(&mut self) -> Result<()> {
        if self.state == SvmVmState::Running {
            self.state = SvmVmState::Paused;
            crate::serial_println!("[SVM-VM {}] Paused", self.id);
        }
        Ok(())
    }
    
    /// Resume the VM
    pub fn resume(&mut self) -> Result<()> {
        if self.state == SvmVmState::Paused {
            self.state = SvmVmState::Running;
            crate::serial_println!("[SVM-VM {}] Resumed", self.id);
        }
        Ok(())
    }
    
    /// Get VM statistics
    pub fn get_stats(&self) -> &SvmVmStats {
        &self.stats
    }
    
    /// Get VM state
    pub fn get_state(&self) -> SvmVmState {
        self.state
    }
    
    /// Read guest memory
    pub fn read_guest_memory(&self, gpa: u64, len: usize) -> Option<&[u8]> {
        let offset = gpa as usize;
        if offset + len <= self.guest_memory.len() {
            Some(&self.guest_memory[offset..offset + len])
        } else {
            None
        }
    }
    
    /// Write to guest memory
    pub fn write_guest_memory(&mut self, gpa: u64, data: &[u8]) -> Result<()> {
        let offset = gpa as usize;
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
        // Free ASID
        super::svm::npt::free_asid(self.asid);
        
        // Remove console
        if let Some(_console_id) = self.console_id {
            // console cleanup if needed
        }
        
        // Remove VirtFS
        super::virtfs::remove_virtfs(self.id);
        
        crate::serial_println!("[SVM-VM {}] Destroyed", self.id);
    }
}

/// Global VM manager for SVM VMs
static SVM_VMS: Mutex<Vec<SvmVirtualMachine>> = Mutex::new(Vec::new());

/// Create a new SVM VM
pub fn create_vm(name: &str, memory_mb: usize) -> Result<u64> {
    let vm = SvmVirtualMachine::new(name, memory_mb)?;
    let id = vm.id;
    SVM_VMS.lock().push(vm);
    Ok(id)
}

/// Get mutable access to a VM by ID
pub fn with_vm<F, R>(id: u64, f: F) -> Option<R>
where
    F: FnOnce(&mut SvmVirtualMachine) -> R,
{
    let mut vms = SVM_VMS.lock();
    vms.iter_mut().find(|vm| vm.id == id).map(f)
}

/// List all SVM VMs
pub fn list_vms() -> Vec<(u64, String, SvmVmState)> {
    SVM_VMS.lock()
        .iter()
        .map(|vm| (vm.id, vm.name.clone(), vm.state))
        .collect()
}

/// Destroy a VM
pub fn destroy_vm(id: u64) -> Result<()> {
    let mut vms = SVM_VMS.lock();
    if let Some(pos) = vms.iter().position(|vm| vm.id == id) {
        vms.remove(pos);
        Ok(())
    } else {
        Err(HypervisorError::VmNotFound)
    }
}
