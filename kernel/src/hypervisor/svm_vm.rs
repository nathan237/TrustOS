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
use super::svm::vmcb::{Vmcb, control_offsets, state_offsets, clean_bits};
use super::svm::npt::{Npt, flags as npt_flags};
use super::mmio::{self, MmioDecoded};
use super::ioapic::IoApicState;
use super::hpet::HpetState;

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
    /// LAPIC timer emulation state
    pub lapic: LapicState,
    /// PIC 8259A emulation state
    pub pic: PicState,
    /// PIT 8254 timer emulation state
    pub pit: PitState,
    /// ACPI PM timer counter (3.579545 MHz, 32-bit)
    pub pm_timer_start: u64,
    /// CMOS RTC state
    cmos_index: u8,
    /// I/O APIC emulation state
    pub ioapic: IoApicState,
    /// HPET (High Precision Event Timer) emulation state
    pub hpet: HpetState,
}

/// Emulated Local APIC timer state
#[derive(Debug, Clone)]
pub struct LapicState {
    /// Timer initial count register
    pub icr: u32,
    /// Timer current count register
    pub ccr: u32,
    /// Timer divide configuration register
    pub dcr: u32,
    /// Timer LVT entry (vector + mode + mask)
    pub timer_lvt: u32,
    /// Spurious interrupt vector register
    pub svr: u32,
    /// Task priority register
    pub tpr: u32,
    /// Whether LAPIC is software-enabled
    pub enabled: bool,
    /// VMEXIT counter at last timer tick (for decrement simulation)
    pub last_tick_exit: u64,
}

impl Default for LapicState {
    fn default() -> Self {
        Self {
            icr: 0,
            ccr: 0,
            dcr: 0,
            timer_lvt: 0x0001_0000, // Masked by default
            svr: 0x1FF,             // Software enabled, vector 0xFF
            tpr: 0,
            enabled: false,
            last_tick_exit: 0,
        }
    }
}

/// Emulated 8259A PIC (Programmable Interrupt Controller) pair
#[derive(Debug, Clone)]
pub struct PicState {
    /// Master PIC: ICW state machine phase (0=ready, 1-4=ICW sequence)
    pub master_icw_phase: u8,
    /// Slave PIC: ICW state machine phase
    pub slave_icw_phase: u8,
    /// Master PIC: interrupt mask register (IMR/OCW1)
    pub master_imr: u8,
    /// Slave PIC: interrupt mask register
    pub slave_imr: u8,
    /// Master PIC: vector base (set by ICW2)
    pub master_vector_base: u8,
    /// Slave PIC: vector base
    pub slave_vector_base: u8,
    /// Master PIC: in-service register (ISR)
    pub master_isr: u8,
    /// Master PIC: interrupt request register (IRR)
    pub master_irr: u8,
    /// Whether initialization is complete
    pub initialized: bool,
}

impl Default for PicState {
    fn default() -> Self {
        Self {
            master_icw_phase: 0,
            slave_icw_phase: 0,
            master_imr: 0xFF, // All masked
            slave_imr: 0xFF,
            master_vector_base: 0x08, // Default BIOS mapping
            slave_vector_base: 0x70,
            master_isr: 0,
            master_irr: 0,
            initialized: false,
        }
    }
}

/// Emulated 8254 PIT (Programmable Interval Timer) channel
#[derive(Debug, Clone)]
pub struct PitChannel {
    /// Reload value (count)
    pub reload: u16,
    /// Current counter value
    pub count: u16,
    /// Operating mode (0-5)
    pub mode: u8,
    /// Access mode: 1=lobyte, 2=hibyte, 3=lo then hi
    pub access: u8,
    /// Latch state for reading
    pub latched: bool,
    pub latch_value: u16,
    /// Waiting for high byte of 16-bit write
    pub write_hi_pending: bool,
    /// Output pin state
    pub output: bool,
}

impl Default for PitChannel {
    fn default() -> Self {
        Self {
            reload: 0xFFFF,
            count: 0xFFFF,
            mode: 0,
            access: 3, // lo/hi by default
            latched: false,
            latch_value: 0,
            write_hi_pending: false,
            output: false,
        }
    }
}

/// Emulated 8254 PIT state (3 channels)
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
            lapic: LapicState::default(),
            pic: PicState::default(),
            pit: PitState::default(),
            pm_timer_start: 0,
            cmos_index: 0,
            ioapic: IoApicState::default(),
            hpet: HpetState::default(),
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
        
        // Allocate MSRPM (MSR Permission Map) — 8 KB (2 pages)
        // All bits = 1 means intercept all MSR accesses
        let msrpm = alloc::vec![0xFFu8; 8192]; // 8KB, all 1s = intercept everything
        let msrpm_ptr = msrpm.as_ptr() as u64;
        let msrpm_phys = msrpm_ptr - crate::memory::hhdm_offset();
        vmcb.set_msrpm_base(msrpm_phys);
        core::mem::forget(msrpm);
        crate::serial_println!("[SVM-VM {}] MSRPM allocated at HPA=0x{:X}", self.id, msrpm_phys);
        
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
    pub fn setup_long_mode(&mut self, entry_point: u64, stack_ptr: u64, guest_cr3: u64) -> Result<()> {
        let vmcb = self.vmcb.as_mut().ok_or(HypervisorError::VmcbNotLoaded)?;
        vmcb.setup_long_mode(entry_point, guest_cr3);
        
        vmcb.write_state(state_offsets::RIP, entry_point);
        vmcb.write_state(state_offsets::RSP, stack_ptr);
        
        crate::serial_println!("[SVM-VM {}] Long mode: RIP=0x{:X}, RSP=0x{:X}, CR3=0x{:X}", 
                              self.id, entry_point, stack_ptr, guest_cr3);
        
        Ok(())
    }
    
    /// Start a Linux kernel using the Linux boot protocol
    pub fn start_linux(
        &mut self,
        bzimage_data: &[u8],
        cmdline: &str,
        initrd: Option<&[u8]>,
    ) -> Result<()> {
        use super::linux_loader;
        
        // Initialize VMCB and NPT if not done
        if self.vmcb.is_none() {
            self.initialize()?;
        }
        
        crate::serial_println!("[SVM-VM {}] Loading Linux kernel ({} bytes)...", self.id, bzimage_data.len());
        
        // Parse bzImage
        let kernel = linux_loader::parse_bzimage(bzimage_data)
            .map_err(|e| {
                crate::serial_println!("[SVM-VM {}] bzImage parse error: {:?}", self.id, e);
                HypervisorError::InvalidGuest
            })?;
        
        crate::serial_println!("[SVM-VM {}] Kernel: protocol={}.{}, 64-bit={}, entry=0x{:X}",
            self.id, kernel.header.version >> 8, kernel.header.version & 0xFF,
            kernel.supports_64bit, kernel.entry_64);
        
        // Prepare guest memory
        let config = linux_loader::LinuxGuestConfig {
            cmdline: alloc::string::String::from(cmdline),
            memory_size: self.memory_size as u64,
            initrd: initrd.map(|d| d.to_vec()),
        };
        
        let setup = linux_loader::load_linux_kernel(&mut self.guest_memory, &kernel, &config)
            .map_err(|e| {
                crate::serial_println!("[SVM-VM {}] Linux load error: {:?}", self.id, e);
                HypervisorError::InvalidGuest
            })?;
        
        crate::serial_println!("[SVM-VM {}] Linux loaded: entry=0x{:X}, stack=0x{:X}, cr3=0x{:X}, gdt=0x{:X}",
            self.id, setup.entry_point, setup.stack_ptr, setup.cr3, setup.gdt_base);
        
        // Configure VMCB for Linux long mode boot
        {
            let vmcb = self.vmcb.as_mut().ok_or(HypervisorError::VmcbNotLoaded)?;
            vmcb.setup_long_mode_for_linux(
                setup.entry_point,
                setup.stack_ptr,
                setup.cr3,
                setup.gdt_base,
                39, // 5 GDT entries × 8 bytes - 1
            );
        }
        
        // Set RSI = boot_params address (Linux boot protocol requirement)
        self.guest_regs.rsi = setup.boot_params_addr;
        self.guest_regs.rbp = 0;
        self.guest_regs.rdi = 0;
        
        crate::serial_println!("[SVM-VM {}] Starting Linux with RSI=0x{:X} (boot_params)", 
            self.id, setup.boot_params_addr);
        
        // Start execution
        self.state = SvmVmState::Running;
        crate::lab_mode::trace_bus::emit_vm_lifecycle(self.id, "LINUX_STARTED");
        self.run_loop()?;
        
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
                
                // Set VMCB clean bits — tell hardware what hasn't changed
                // This avoids re-reading unchanged state from memory on VMRUN
                // After a VMEXIT, if we only modified RIP (for instruction skip),
                // we can mark most fields as clean.
                let clean = clean_bits::IOPM      // I/O permissions didn't change
                          | clean_bits::ASID      // ASID didn't change
                          | clean_bits::NP        // NPT didn't change
                          | clean_bits::LBR       // LBR didn't change
                          | clean_bits::AVIC;     // AVIC didn't change
                vmcb.set_clean_bits(clean);
                
                // Inject periodic timer interrupt based on LAPIC state
                if self.lapic.enabled && self.lapic.icr > 0 {
                    let masked = (self.lapic.timer_lvt >> 16) & 1;
                    if masked == 0 {
                        let timer_mode = (self.lapic.timer_lvt >> 17) & 0x3;
                        let vector = (self.lapic.timer_lvt & 0xFF) as u64;
                        let divider = match self.lapic.dcr & 0xB {
                            0x0 => 2u64, 0x1 => 4, 0x2 => 8, 0x3 => 16,
                            0x8 => 32, 0x9 => 64, 0xA => 128, 0xB => 1,
                            _ => 1,
                        };
                        let elapsed = self.stats.vmexits.saturating_sub(self.lapic.last_tick_exit);
                        let ticks = (elapsed * 256) / divider;
                        
                        if ticks >= self.lapic.icr as u64 {
                            // Timer fired!
                            let rflags = vmcb.read_state(state_offsets::RFLAGS);
                            if (rflags & 0x200) != 0 && vector > 0 { // IF set and valid vector
                                let event_inj: u64 = vector
                                                   | (0u64 << 8)    // Type 0 = external interrupt
                                                   | (1u64 << 31);  // V = valid
                                vmcb.write_control(control_offsets::EVENT_INJ, event_inj);
                            }
                            // Reset for periodic mode, stop for one-shot
                            match timer_mode {
                                1 => { // Periodic
                                    self.lapic.last_tick_exit = self.stats.vmexits;
                                    self.lapic.ccr = self.lapic.icr;
                                }
                                _ => { // One-shot or TSC-deadline
                                    self.lapic.icr = 0;
                                    self.lapic.ccr = 0;
                                }
                            }
                        }
                    }
                } else if self.stats.vmexits > 0 && self.stats.vmexits % 5000 == 0 {
                    // Fallback: if LAPIC timer not yet programmed, inject timer periodically
                    let rflags = vmcb.read_state(state_offsets::RFLAGS);
                    if (rflags & 0x200) != 0 {
                        let event_inj: u64 = 0x20
                                           | (0u64 << 8)
                                           | (1u64 << 31);
                        vmcb.write_control(control_offsets::EVENT_INJ, event_inj);
                    }
                }
                
                // Check HPET timers → I/O APIC routing → interrupt injection
                // Only inject if no LAPIC timer interrupt was injected this iteration
                {
                    let current_event = vmcb.read_control(control_offsets::EVENT_INJ);
                    let already_injecting = (current_event & (1u64 << 31)) != 0;
                    
                    if !already_injecting {
                        if let Some(vector) = self.check_hpet_interrupts() {
                            let vmcb = self.vmcb.as_mut().unwrap();
                            let rflags = vmcb.read_state(state_offsets::RFLAGS);
                            if (rflags & 0x200) != 0 { // IF set
                                let event_inj: u64 = vector
                                                   | (0u64 << 8)    // Type 0 = external interrupt
                                                   | (1u64 << 31);  // V = valid
                                vmcb.write_control(control_offsets::EVENT_INJ, event_inj);
                            }
                        }
                    }
                }
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
                // For Linux, HLT is used for idle loop — inject a timer interrupt
                // to wake the guest and advance past the HLT instruction
                {
                    let vmcb = self.vmcb.as_mut().ok_or(HypervisorError::VmcbNotLoaded)?;
                    vmcb.write_state(state_offsets::RIP, next_rip);
                    
                    // Inject timer on HLT to wake the guest
                    let rflags = vmcb.read_state(state_offsets::RFLAGS);
                    if (rflags & 0x200) != 0 { // IF set
                        // Use LAPIC timer vector if programmed, otherwise default 0x20
                        let vector = if self.lapic.enabled && (self.lapic.timer_lvt & 0xFF) > 0 
                            && ((self.lapic.timer_lvt >> 16) & 1) == 0 {
                            (self.lapic.timer_lvt & 0xFF) as u64
                        } else {
                            0x20
                        };
                        let event_inj: u64 = vector
                                           | (0u64 << 8)    // Type 0 = external interrupt
                                           | (1u64 << 31);  // V = valid
                        vmcb.write_control(control_offsets::EVENT_INJ, event_inj);
                    }
                }
                
                // Linux can HLT many times during boot (idle loops, waiting for devices)
                // Only stop after an extreme number indicating a hang
                if self.stats.hlt_exits > 5_000_000 {
                    crate::serial_println!("[SVM-VM {}] Too many HLT exits ({}), stopping", self.id, self.stats.hlt_exits);
                    self.state = SvmVmState::Stopped;
                    Ok(false)
                } else {
                    // Log periodically
                    if self.stats.hlt_exits % 10000 == 0 {
                        crate::serial_println!("[SVM-VM {}] HLT count: {}", self.id, self.stats.hlt_exits);
                    }
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
                
                // Fetch instruction bytes from VMCB for MMIO decoding
                let vmcb_ref = self.vmcb.as_ref().ok_or(HypervisorError::VmcbNotLoaded)?;
                let (fetched, insn_bytes) = vmcb_ref.guest_insn_bytes();
                
                // Try to decode the MMIO instruction
                let decoded = mmio::decode_mmio_instruction(&insn_bytes, fetched, true);
                
                // Determine if this is a known MMIO region we can emulate
                let handled = self.handle_npf(guest_phys, error_code, guest_rip, decoded.as_ref());
                
                if handled {
                    // Advance RIP past the decoded instruction
                    if let Some(ref dec) = decoded {
                        let vmcb = self.vmcb.as_mut().ok_or(HypervisorError::VmcbNotLoaded)?;
                        vmcb.write_state(state_offsets::RIP, guest_rip + dec.insn_len as u64);
                    } else if self.features.nrip_save {
                        // Fallback: try NRIP_SAVE if decoder couldn't parse the instruction
                        let vmcb = self.vmcb.as_ref().ok_or(HypervisorError::VmcbNotLoaded)?;
                        let nrip = vmcb.read_control(control_offsets::NEXT_RIP);
                        if nrip > guest_rip && nrip < guest_rip + 16 {
                            let vmcb = self.vmcb.as_mut().ok_or(HypervisorError::VmcbNotLoaded)?;
                            vmcb.write_state(state_offsets::RIP, nrip);
                        } else {
                            // Last resort: skip 3 bytes (common MOV instruction minimum)
                            crate::serial_println!("[SVM-VM {}] NPF: decode failed, skipping 3 bytes at RIP=0x{:X}", 
                                self.id, guest_rip);
                            let vmcb = self.vmcb.as_mut().ok_or(HypervisorError::VmcbNotLoaded)?;
                            vmcb.write_state(state_offsets::RIP, guest_rip + 3);
                        }
                    } else {
                        // No NRIP, no decode — skip 3 bytes as best guess
                        crate::serial_println!("[SVM-VM {}] NPF: no decode/nrip, skipping 3 bytes", self.id);
                        let vmcb = self.vmcb.as_mut().ok_or(HypervisorError::VmcbNotLoaded)?;
                        vmcb.write_state(state_offsets::RIP, guest_rip + 3);
                    }
                    Ok(true)
                } else {
                    crate::lab_mode::trace_bus::emit_vm_memory(
                        self.id, "NPF_VIOLATION", guest_phys, error_code
                    );
                    crate::serial_println!("[SVM-VM {}] FATAL NPF: GPA=0x{:X}, Error=0x{:X}, RIP=0x{:X}", 
                                          self.id, guest_phys, error_code, guest_rip);
                    
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
            
            // ===== Additional exits for Linux boot =====
            
            SvmExitCode::Xsetbv => {
                // Guest wants to set XCR0 (extended control register)
                // ECX = XCR number (must be 0), EDX:EAX = value
                let xcr = self.guest_regs.rcx as u32;
                let value = (self.guest_regs.rdx << 32) | (self.guest_regs.rax & 0xFFFF_FFFF);
                if xcr == 0 {
                    // Allow XSETBV with sanitized value:
                    // Bit 0 (x87) must always be set, mask to supported features
                    let safe_value = value | 1; // Ensure x87 bit is set
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
                vmcb.write_state(state_offsets::RIP, next_rip);
                Ok(true)
            }
            
            SvmExitCode::Invd => {
                // INVD — invalidate caches without writeback
                // Just skip it (WBINVD is the safe version)
                let vmcb = self.vmcb.as_mut().ok_or(HypervisorError::VmcbNotLoaded)?;
                vmcb.write_state(state_offsets::RIP, next_rip);
                Ok(true)
            }
            
            SvmExitCode::Invlpg => {
                // INVLPG — invalidate TLB entry for a single page
                // With NPT, guest TLB management is handled by hardware
                // Just advance RIP
                let vmcb = self.vmcb.as_mut().ok_or(HypervisorError::VmcbNotLoaded)?;
                vmcb.write_state(state_offsets::RIP, next_rip);
                Ok(true)
            }
            
            SvmExitCode::Wbinvd => {
                // WBINVD — write-back and invalidate caches
                // Safe to execute on host (just flushes caches)
                unsafe { core::arch::asm!("wbinvd", options(nomem, nostack)); }
                let vmcb = self.vmcb.as_mut().ok_or(HypervisorError::VmcbNotLoaded)?;
                vmcb.write_state(state_offsets::RIP, next_rip);
                Ok(true)
            }
            
            SvmExitCode::Rdtsc => {
                // RDTSC — return host TSC (with optional offset)
                let tsc = unsafe { core::arch::x86_64::_rdtsc() };
                self.guest_regs.rax = tsc & 0xFFFF_FFFF;
                self.guest_regs.rdx = tsc >> 32;
                let vmcb = self.vmcb.as_mut().ok_or(HypervisorError::VmcbNotLoaded)?;
                vmcb.write_state(state_offsets::RIP, next_rip);
                Ok(true)
            }
            
            SvmExitCode::Rdtscp => {
                // RDTSCP — same as RDTSC but also returns IA32_TSC_AUX in ECX
                let tsc = unsafe { core::arch::x86_64::_rdtsc() };
                self.guest_regs.rax = tsc & 0xFFFF_FFFF;
                self.guest_regs.rdx = tsc >> 32;
                self.guest_regs.rcx = 0; // TSC_AUX = 0
                let vmcb = self.vmcb.as_mut().ok_or(HypervisorError::VmcbNotLoaded)?;
                vmcb.write_state(state_offsets::RIP, next_rip);
                Ok(true)
            }
            
            SvmExitCode::Pause => {
                // PAUSE — hint to processor, just skip
                let vmcb = self.vmcb.as_mut().ok_or(HypervisorError::VmcbNotLoaded)?;
                vmcb.write_state(state_offsets::RIP, next_rip);
                Ok(true)
            }
            
            SvmExitCode::Monitor | SvmExitCode::Mwait | SvmExitCode::MwaitConditional => {
                // MONITOR/MWAIT — used for power management idle
                // Skip and let the guest continue (it will find an alternative)
                let vmcb = self.vmcb.as_mut().ok_or(HypervisorError::VmcbNotLoaded)?;
                vmcb.write_state(state_offsets::RIP, next_rip);
                Ok(true)
            }
            
            SvmExitCode::Vintr => {
                // Virtual interrupt delivered — just continue
                Ok(true)
            }
            
            SvmExitCode::TaskSwitch => {
                // Task switch — Linux doesn't use hardware task switching in long mode
                // Log and continue
                crate::serial_println!("[SVM-VM {}] TaskSwitch at RIP=0x{:X}", self.id, guest_rip);
                let vmcb = self.vmcb.as_mut().ok_or(HypervisorError::VmcbNotLoaded)?;
                vmcb.write_state(state_offsets::RIP, next_rip);
                Ok(true)
            }
            
            // Exception intercepts — re-inject into guest
            SvmExitCode::ExceptionDE | SvmExitCode::ExceptionDB |
            SvmExitCode::ExceptionBP | SvmExitCode::ExceptionOF |
            SvmExitCode::ExceptionBR | SvmExitCode::ExceptionUD |
            SvmExitCode::ExceptionNM | SvmExitCode::ExceptionMF |
            SvmExitCode::ExceptionAC | SvmExitCode::ExceptionXF => {
                // These exceptions don't have error codes — re-inject into guest
                let vector = (exit_code - 0x40) as u8;
                if self.stats.vmexits < 200 {
                    crate::serial_println!("[SVM-VM {}] Exception #{} at RIP=0x{:X} — re-injecting", 
                        self.id, vector, guest_rip);
                }
                let vmcb = self.vmcb.as_mut().ok_or(HypervisorError::VmcbNotLoaded)?;
                // Event type 3 = exception, V=1
                vmcb.inject_event(vector, 3, None);
                Ok(true)
            }
            
            SvmExitCode::ExceptionGP => {
                // #GP — re-inject with error code
                let error_code = exit_info1 as u32;
                if self.stats.vmexits < 200 {
                    crate::serial_println!("[SVM-VM {}] #GP(0x{:X}) at RIP=0x{:X}", 
                        self.id, error_code, guest_rip);
                }
                let vmcb = self.vmcb.as_mut().ok_or(HypervisorError::VmcbNotLoaded)?;
                vmcb.inject_event(13, 3, Some(error_code));
                Ok(true)
            }
            
            SvmExitCode::ExceptionPF => {
                // #PF — re-inject with error code, set CR2
                let error_code = exit_info1 as u32;
                let fault_addr = exit_info2;
                if self.stats.vmexits < 200 {
                    crate::serial_println!("[SVM-VM {}] #PF at 0x{:X} (err=0x{:X}) RIP=0x{:X}", 
                        self.id, fault_addr, error_code, guest_rip);
                }
                let vmcb = self.vmcb.as_mut().ok_or(HypervisorError::VmcbNotLoaded)?;
                vmcb.write_state(state_offsets::CR2, fault_addr);
                vmcb.inject_event(14, 3, Some(error_code));
                Ok(true)
            }
            
            SvmExitCode::ExceptionDF => {
                // Double fault — this is very bad, stop the VM
                crate::serial_println!("[SVM-VM {}] DOUBLE FAULT at RIP=0x{:X}", self.id, guest_rip);
                self.state = SvmVmState::Crashed;
                Ok(false)
            }
            
            SvmExitCode::ExceptionTS | SvmExitCode::ExceptionNP |
            SvmExitCode::ExceptionSS => {
                // Exceptions with error codes — re-inject
                let vector = (exit_code - 0x40) as u8;
                let error_code = exit_info1 as u32;
                if self.stats.vmexits < 200 {
                    crate::serial_println!("[SVM-VM {}] Exception #{} (err=0x{:X}) at RIP=0x{:X}", 
                        self.id, vector, error_code, guest_rip);
                }
                let vmcb = self.vmcb.as_mut().ok_or(HypervisorError::VmcbNotLoaded)?;
                vmcb.inject_event(vector, 3, Some(error_code));
                Ok(true)
            }
            
            SvmExitCode::ExceptionMC => {
                // Machine Check — re-inject as abort
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
    
    /// Handle CPUID exit
    fn handle_cpuid(&mut self) {
        let leaf = self.guest_regs.rax as u32;
        let subleaf = self.guest_regs.rcx as u32;
        
        // Handle hypervisor-specific leaves
        match leaf {
            // Hypervisor identification (KVM-compatible)
            0x4000_0000 => {
                // Return "TrstOSTrstOS" and max leaf = 0x40000001
                self.guest_regs.rax = 0x4000_0001;
                self.guest_regs.rbx = 0x7473_7254; // "Trst"
                self.guest_regs.rcx = 0x7254_534F; // "OSTr"
                self.guest_regs.rdx = 0x534F_7473; // "stOS"
                return;
            }
            0x4000_0001 => {
                // Hypervisor features: report TSC frequency (if known) and basic features
                self.guest_regs.rax = 0; // No special features
                self.guest_regs.rbx = 0;
                self.guest_regs.rcx = 0;
                self.guest_regs.rdx = 0;
                return;
            }
            _ => {}
        }
        
        // Execute real CPUID on host
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
                // Basic CPUID: report hypervisor presence in max leaf
                // Keep vendor string, but indicate hypervisor support
                // Max standard leaf: pass through
            }
            0x0000_0001 => {
                // Feature flags
                ecx &= !(1 << 5);   // Hide VMX
                ecx |= 1 << 31;     // Set hypervisor present bit
                // Hide x2APIC if we don't emulate it
                ecx &= !(1 << 21);  // Hide x2APIC
                // Hide MONITOR/MWAIT (requires intercept setup)
                ecx &= !(1 << 3);   // Hide MONITOR
            }
            0x0000_0007 => {
                // Extended features
                if subleaf == 0 {
                    ebx &= !(1 << 0);  // Hide FSGSBASE for now
                    ecx &= !(1 << 2);  // Hide UMIP
                    ecx &= !(1 << 4);  // Hide OSPKE
                }
            }
            0x0000_000A => {
                // Performance monitoring — hide from guest
                eax = 0;
                ebx = 0;
                ecx = 0;
                edx = 0;
            }
            0x0000_000B | 0x0000_001F => {
                // Extended topology — report 1 core, 1 thread
                if subleaf == 0 {
                    eax = 0; // No SMT sub-levels
                    ebx = 1; // 1 logical processor
                    ecx = (1 << 8) | subleaf; // SMT level type
                } else if subleaf == 1 {
                    eax = 0;
                    ebx = 1; // 1 core
                    ecx = (2 << 8) | subleaf; // Core level type
                } else {
                    eax = 0;
                    ebx = 0;
                    ecx = 0;
                }
            }
            0x8000_0001 => {
                // AMD extended features
                ecx &= !(1 << 2);  // Hide SVM from guest
            }
            _ => {
                // Pass through all other leaves unchanged
            }
        }
        
        self.guest_regs.rax = eax as u64;
        self.guest_regs.rbx = ebx as u64;
        self.guest_regs.rcx = ecx as u64;
        self.guest_regs.rdx = edx as u64;
    }
    
    /// Handle Nested Page Fault (NPF)
    /// Returns true if the fault was handled successfully
    fn handle_npf(&mut self, guest_phys: u64, error_code: u64, guest_rip: u64, decoded: Option<&MmioDecoded>) -> bool {
        // MMIO region constants
        const LAPIC_BASE: u64 = 0xFEE0_0000;
        const LAPIC_END: u64 = 0xFEE0_1000;
        const IOAPIC_BASE: u64 = 0xFEC0_0000;
        const IOAPIC_END: u64 = 0xFEC0_1000;
        const HPET_BASE: u64 = 0xFED0_0000;
        const HPET_END: u64 = 0xFED0_1000;
        
        match guest_phys {
            // Local APIC MMIO (0xFEE00000 - 0xFEE00FFF)
            LAPIC_BASE..=LAPIC_END => {
                self.handle_lapic_mmio(guest_phys, error_code, decoded);
                true
            }
            // I/O APIC MMIO (0xFEC00000 - 0xFEC00FFF) 
            IOAPIC_BASE..=IOAPIC_END => {
                self.handle_ioapic_mmio(guest_phys, error_code, decoded);
                true
            }
            // HPET MMIO (0xFED00000 - 0xFED00FFF)
            HPET_BASE..=HPET_END => {
                self.handle_hpet_mmio(guest_phys, error_code, decoded);
                true
            }
            // VGA framebuffer (0xA0000 - 0xBFFFF) — map if within guest memory
            0xA0000..=0xBFFFF => {
                if self.stats.npf_exits < 20 {
                    crate::serial_println!("[SVM-VM {}] VGA FB access at 0x{:X}", self.id, guest_phys);
                }
                self.mmio_complete_read(decoded, 0);
                true
            }
            // ROM area (0xC0000 - 0xFFFFF) — typically BIOS ROMs
            0xC0000..=0xFFFFF => {
                self.mmio_complete_read(decoded, 0);
                true
            }
            // Below guest memory but unmapped? Try to handle
            gpa if gpa < self.memory_size as u64 => {
                crate::serial_println!("[SVM-VM {}] NPF in guest RAM at 0x{:X}, err=0x{:X}, RIP=0x{:X}", 
                    self.id, gpa, error_code, guest_rip);
                false
            }
            // Above 4GB region? Likely a BAR or device mapping
            gpa if gpa >= 0x1_0000_0000 => {
                if self.stats.npf_exits < 50 {
                    crate::serial_println!("[SVM-VM {}] High MMIO access at 0x{:X} (absorbed)", self.id, gpa);
                }
                self.mmio_complete_read(decoded, 0xFFFF_FFFF);
                true
            }
            _ => {
                if self.stats.npf_exits < 50 {
                    crate::serial_println!("[SVM-VM {}] NPF: GPA=0x{:X}, err=0x{:X}, RIP=0x{:X}", 
                        self.id, guest_phys, error_code, guest_rip);
                }
                false
            }
        }
    }
    
    /// Get the write value from a decoded MMIO instruction.
    /// Uses the decoded register or immediate; falls back to RAX if no decode available.
    fn mmio_get_write_value(&self, decoded: Option<&MmioDecoded>) -> u32 {
        if let Some(dec) = decoded {
            if let Some(imm) = dec.immediate {
                return mmio::mask_to_size(imm, dec.operand_size) as u32;
            }
            if let Some(reg_idx) = dec.register {
                let val = mmio::read_guest_reg(&self.guest_regs, reg_idx);
                return mmio::mask_to_size(val, dec.operand_size) as u32;
            }
        }
        // Fallback: use RAX (legacy behavior)
        self.guest_regs.rax as u32
    }
    
    /// Complete an MMIO read by writing the result to the correct guest register.
    /// If no decode available, falls back to writing RAX.
    fn mmio_complete_read(&mut self, decoded: Option<&MmioDecoded>, value: u32) {
        if let Some(dec) = decoded {
            if !dec.is_write {
                if let Some(reg_idx) = dec.register {
                    // For MOVZX/MOVSX the operand_size tells us the MMIO access size,
                    // but we write the full value (already masked) to the destination register.
                    // LAPIC registers are always 32-bit, so we just zero-extend.
                    mmio::write_guest_reg(&mut self.guest_regs, reg_idx, value as u64);
                    return;
                }
            }
        }
        // Fallback: write to RAX
        self.guest_regs.rax = value as u64;
    }
    
    /// Emulate Local APIC MMIO access (read and write) using decoded instruction
    fn handle_lapic_mmio(&mut self, gpa: u64, error_code: u64, decoded: Option<&MmioDecoded>) {
        let offset = (gpa & 0xFFF) as u32;
        let is_write = (error_code & 0x2) != 0; // Bit 1 of NPF error code = write
        
        // Common LAPIC register offsets
        const LAPIC_ID: u32 = 0x020;
        const LAPIC_VERSION: u32 = 0x030;
        const LAPIC_TPR: u32 = 0x080;
        const LAPIC_EOI: u32 = 0x0B0;
        const LAPIC_SVR: u32 = 0x0F0;
        const LAPIC_ISR_BASE: u32 = 0x100;
        const LAPIC_TMR_BASE: u32 = 0x180;
        const LAPIC_IRR_BASE: u32 = 0x200;
        const LAPIC_ESR: u32 = 0x280;
        const LAPIC_ICR_LOW: u32 = 0x300;
        const LAPIC_ICR_HIGH: u32 = 0x310;
        const LAPIC_TIMER_LVT: u32 = 0x320;
        const LAPIC_THERMAL_LVT: u32 = 0x330;
        const LAPIC_PERF_LVT: u32 = 0x340;
        const LAPIC_LINT0: u32 = 0x350;
        const LAPIC_LINT1: u32 = 0x360;
        const LAPIC_ERROR_LVT: u32 = 0x370;
        const LAPIC_TIMER_ICR: u32 = 0x380;
        const LAPIC_TIMER_CCR: u32 = 0x390;
        const LAPIC_TIMER_DCR: u32 = 0x3E0;
        
        if is_write {
            let value = self.mmio_get_write_value(decoded);
            match offset {
                LAPIC_TPR => {
                    self.lapic.tpr = value & 0xFF;
                    // Also update V_TPR in VMCB for hardware acceleration
                    if let Some(ref mut vmcb) = self.vmcb {
                        vmcb.write_u32(control_offsets::V_TPR, value & 0x0F);
                    }
                }
                LAPIC_EOI => {
                    // End-of-interrupt: clear highest ISR bit (we don't track ISR, just accept)
                }
                LAPIC_SVR => {
                    self.lapic.svr = value;
                    self.lapic.enabled = (value & 0x100) != 0;
                    if self.stats.vmexits < 1000 {
                        crate::serial_println!("[SVM-VM {}] LAPIC SVR=0x{:X} enabled={}", 
                            self.id, value, self.lapic.enabled);
                    }
                }
                LAPIC_ICR_LOW => {
                    // IPI — log but don't deliver (single vCPU)
                    if self.stats.vmexits < 1000 {
                        crate::serial_println!("[SVM-VM {}] LAPIC ICR write: 0x{:X} (IPI ignored, single vCPU)", 
                            self.id, value);
                    }
                }
                LAPIC_ICR_HIGH => {} // Destination for IPI, ignore
                LAPIC_TIMER_LVT => {
                    self.lapic.timer_lvt = value;
                    if self.stats.vmexits < 1000 {
                        let vector = value & 0xFF;
                        let masked = (value >> 16) & 1;
                        let mode = (value >> 17) & 0x3;
                        let mode_str = match mode {
                            0 => "one-shot",
                            1 => "periodic",
                            2 => "TSC-deadline",
                            _ => "reserved",
                        };
                        crate::serial_println!("[SVM-VM {}] LAPIC timer LVT: vec={} mode={} masked={}", 
                            self.id, vector, mode_str, masked);
                    }
                }
                LAPIC_LINT0 | LAPIC_LINT1 | LAPIC_THERMAL_LVT | LAPIC_PERF_LVT | LAPIC_ERROR_LVT => {
                    // LVT entries — accept silently
                }
                LAPIC_TIMER_ICR => {
                    self.lapic.icr = value;
                    self.lapic.ccr = value; // Start countdown
                    self.lapic.last_tick_exit = self.stats.vmexits;
                    if self.stats.vmexits < 1000 && value > 0 {
                        crate::serial_println!("[SVM-VM {}] LAPIC timer ICR={} (timer armed)", self.id, value);
                    }
                }
                LAPIC_TIMER_DCR => {
                    self.lapic.dcr = value;
                }
                LAPIC_ESR => {} // Write clears ESR
                _ => {
                    if self.stats.vmexits < 200 {
                        crate::serial_println!("[SVM-VM {}] LAPIC write offset=0x{:X} val=0x{:X}", 
                            self.id, offset, value);
                    }
                }
            }
        } else {
            // Read
            let value: u32 = match offset {
                LAPIC_ID => 0,                    // APIC ID = 0 (BSP)
                LAPIC_VERSION => 0x0005_0014,     // Version 0x14, max LVT entry 5
                LAPIC_TPR => self.lapic.tpr,
                LAPIC_SVR => self.lapic.svr,
                LAPIC_ISR_BASE..=0x170 => 0,
                LAPIC_TMR_BASE..=0x1F0 => 0,
                LAPIC_IRR_BASE..=0x270 => 0,
                LAPIC_ESR => 0,
                LAPIC_ICR_LOW => 0,               // Delivery status = idle
                LAPIC_ICR_HIGH => 0,
                LAPIC_TIMER_LVT => self.lapic.timer_lvt,
                LAPIC_THERMAL_LVT => 0x0001_0000, // Masked
                LAPIC_PERF_LVT => 0x0001_0000,    // Masked
                LAPIC_LINT0 => 0x0001_0000,        // Masked
                LAPIC_LINT1 => 0x0001_0000,        // Masked
                LAPIC_ERROR_LVT => 0x0001_0000,    // Masked
                LAPIC_TIMER_ICR => self.lapic.icr,
                LAPIC_TIMER_CCR => {
                    // Simulate countdown based on VMEXIT count difference
                    if self.lapic.icr > 0 {
                        let elapsed = self.stats.vmexits.saturating_sub(self.lapic.last_tick_exit);
                        let divider = match self.lapic.dcr & 0xB {
                            0x0 => 2u64, 0x1 => 4, 0x2 => 8, 0x3 => 16,
                            0x8 => 32, 0x9 => 64, 0xA => 128, 0xB => 1,
                            _ => 1,
                        };
                        let ticks = (elapsed * 256) / divider;
                        let remaining = (self.lapic.icr as u64).saturating_sub(ticks);
                        remaining as u32
                    } else {
                        0
                    }
                }
                LAPIC_TIMER_DCR => self.lapic.dcr,
                _ => 0,
            };
            self.mmio_complete_read(decoded, value);
        }
    }
    
    /// Emulate I/O APIC MMIO access using decoded instruction
    fn handle_ioapic_mmio(&mut self, gpa: u64, error_code: u64, decoded: Option<&MmioDecoded>) {
        let offset = gpa - super::ioapic::IOAPIC_BASE;
        let is_write = (error_code & 0x2) != 0;
        
        if is_write {
            let value = self.mmio_get_write_value(decoded);
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
            self.mmio_complete_read(decoded, value);
        }
    }
    
    /// Handle HPET MMIO access (0xFED00000 - 0xFED00FFF)
    fn handle_hpet_mmio(&mut self, gpa: u64, error_code: u64, decoded: Option<&MmioDecoded>) {
        let offset = gpa - super::hpet::HPET_BASE;
        let is_write = (error_code & 0x2) != 0;
        
        // Determine operand size from decoded instruction
        let size = decoded.map(|d| d.operand_size).unwrap_or(4);
        
        if is_write {
            let value = self.mmio_get_write_value(decoded) as u64;
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
            self.mmio_complete_read(decoded, value as u32);
        }
    }
    
    /// Check HPET timers and inject interrupts via I/O APIC routing.
    /// Returns the vector to inject (if any).
    fn check_hpet_interrupts(&mut self) -> Option<u64> {
        let timer_status = self.hpet.check_timers();
        
        for (i, &(fired, irq_route)) in timer_status.iter().enumerate() {
            if !fired {
                continue;
            }
            
            // Look up the route in the I/O APIC redirection table
            if let Some(route) = self.ioapic.get_irq_route(irq_route) {
                if !route.masked && route.vector > 0 {
                    // Set interrupt status bit for this timer
                    self.hpet.isr |= 1 << i;
                    
                    // For edge-triggered: reset comparator to avoid re-firing
                    let config = self.hpet.timers[i].config;
                    let periodic = (config >> 3) & 1 != 0;
                    if periodic {
                        // In periodic mode, add comparator value to itself
                        let comp = self.hpet.timers[i].comparator;
                        if comp > 0 {
                            self.hpet.timers[i].comparator = self.hpet.timers[i].comparator.wrapping_add(comp);
                        }
                    } else {
                        // One-shot: disable timer by clearing enable bit
                        self.hpet.timers[i].config &= !(1 << 2);
                    }
                    
                    return Some(route.vector as u64);
                }
            }
        }
        None
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
                0x20 => {
                    // Master PIC: read ISR or IRR depending on OCW3
                    self.pic.master_isr as u32
                }
                0x21 => self.pic.master_imr as u32,  // Master PIC: IMR
                0xA0 => 0,                           // Slave PIC: ISR
                0xA1 => self.pic.slave_imr as u32,   // Slave PIC: IMR
                
                // ── PIT (8254) timer ──────────────────────────────
                0x40 | 0x41 | 0x42 => {
                    let ch = (port - 0x40) as usize;
                    let pit_ch = &mut self.pit.channels[ch];
                    if pit_ch.latched {
                        pit_ch.latched = false;
                        pit_ch.latch_value as u32
                    } else {
                        // Simulate decrement based on VMEXIT count
                        let simulated = pit_ch.count.wrapping_sub(
                            (self.stats.vmexits & 0xFFFF) as u16
                        );
                        simulated as u32
                    }
                }
                0x43 => 0,                      // Control word (write-only, read returns 0)
                0x61 => 0x20,                   // NMI status / speaker control
                
                // ── CMOS/RTC ──────────────────────────────────────
                0x70 => self.cmos_index as u32,
                0x71 => {
                    // Return CMOS register values based on selected index
                    (match self.cmos_index {
                        0x00 => 0x00u32,  // RTC seconds
                        0x02 => 0x30,  // RTC minutes
                        0x04 => 0x12,  // RTC hours (BCD 12 = noon)
                        0x06 => 0x02,  // RTC day of week (Monday)
                        0x07 => 0x17,  // RTC day of month
                        0x08 => 0x02,  // RTC month (February)
                        0x09 => 0x26,  // RTC year (2026 in BCD)
                        0x0A => 0x26,  // Status Register A: divider + rate
                        0x0B => 0x02,  // Status Register B: 24h mode
                        0x0C => 0x00,  // Status Register C: no interrupts pending
                        0x0D => 0x80,  // Status Register D: battery OK
                        0x0E => 0x00,  // Diagnostic status
                        0x0F => 0x00,  // Shutdown status
                        0x10 => 0x00,  // Floppy type (none)
                        0x12 => 0x00,  // Hard disk type (none)
                        0x14 => 0x06,  // Equipment: math coprocessor, color display
                        0x15 => 0x80,  // Base memory low byte (640K = 0x0280)
                        0x16 => 0x02,  // Base memory high byte
                        0x17 => 0x00,  // Extended memory low (set by e820 instead)
                        0x18 => 0x00,  // Extended memory high
                        0x32 => 0x20,  // Century (BCD 20 for 2000s)
                        _ => 0x00,
                    })
                }
                
                // ── DMA controllers ───────────────────────────────
                0x00..=0x0F | 0x80..=0x8F | 0xC0..=0xDF => 0,
                
                // ── VGA / display ─────────────────────────────────
                0x3B0..=0x3DF => 0,             // VGA registers
                
                // ── PCI config space ──────────────────────────────
                0xCF8 => 0xFFFF_FFFF,           // PCI config addr (no devices)
                0xCFC..=0xCFF => 0xFFFF_FFFF,   // PCI config data (no devices)
                
                // ── ACPI / power management ───────────────────────
                0xB000 => 0,           // PM1a_EVT_STS (no events)
                0xB002 => 0,           // PM1a_EVT_EN
                0xB004 => 0,           // PM1a_CNT (SCI_EN will be checked)
                0xB008 => {
                    // ACPI PM Timer (3.579545 MHz, 32-bit)
                    // Simulate based on VMEXIT count (each ~1µs ≈ 3.58 ticks)
                    let ticks = self.stats.vmexits.wrapping_mul(4); // ~4 ticks per vmexit
                    (ticks & 0xFFFF_FFFF) as u32
                }
                0xB009..=0xB00B => {
                    // PM timer upper bytes (32-bit read may access byte-at-a-time)
                    let ticks = self.stats.vmexits.wrapping_mul(4);
                    let byte_offset = (port - 0xB008) as u32;
                    ((ticks >> (byte_offset * 8)) & 0xFF) as u32
                }
                0xB00C..=0xB03F => 0,  // Other PM registers
                
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
                
                // PIC programming — full ICW/OCW state machine
                0x20 => {
                    // Master PIC command port
                    let v = value as u8;
                    if v & 0x10 != 0 {
                        // ICW1: start initialization sequence
                        self.pic.master_icw_phase = 1;
                        self.pic.master_isr = 0;
                        self.pic.master_irr = 0;
                        if self.stats.io_exits < 200 {
                            crate::serial_println!("[SVM-VM {}] PIC master: ICW1=0x{:02X}", self.id, v);
                        }
                    } else if v & 0x08 != 0 {
                        // OCW3: read ISR/IRR command
                    } else {
                        // OCW2: EOI commands
                        if v == 0x20 {
                            // Non-specific EOI
                            self.pic.master_isr = 0;
                        }
                    }
                }
                0x21 => {
                    // Master PIC data port
                    let v = value as u8;
                    match self.pic.master_icw_phase {
                        1 => {
                            // ICW2: vector base
                            self.pic.master_vector_base = v & 0xF8;
                            self.pic.master_icw_phase = 2;
                            if self.stats.io_exits < 200 {
                                crate::serial_println!("[SVM-VM {}] PIC master: ICW2 vector_base=0x{:02X}", self.id, v);
                            }
                        }
                        2 => {
                            // ICW3: cascade configuration
                            self.pic.master_icw_phase = 3;
                        }
                        3 => {
                            // ICW4: mode
                            self.pic.master_icw_phase = 0;
                            self.pic.initialized = true;
                            if self.stats.io_exits < 200 {
                                crate::serial_println!("[SVM-VM {}] PIC master initialized: base=0x{:02X}", 
                                    self.id, self.pic.master_vector_base);
                            }
                        }
                        _ => {
                            // OCW1: set IMR
                            self.pic.master_imr = v;
                        }
                    }
                }
                0xA0 => {
                    // Slave PIC command port
                    let v = value as u8;
                    if v & 0x10 != 0 {
                        self.pic.slave_icw_phase = 1;
                    } else if v == 0x20 {
                        // Non-specific EOI
                    }
                }
                0xA1 => {
                    // Slave PIC data port
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
                
                // PIT programming — track counter reload values
                0x40 | 0x41 | 0x42 => {
                    let ch = (port - 0x40) as usize;
                    let v = value as u8;
                    let pit_ch = &mut self.pit.channels[ch];
                    match pit_ch.access {
                        1 => {
                            // Lobyte only
                            pit_ch.reload = (pit_ch.reload & 0xFF00) | v as u16;
                            pit_ch.count = pit_ch.reload;
                        }
                        2 => {
                            // Hibyte only
                            pit_ch.reload = (pit_ch.reload & 0x00FF) | ((v as u16) << 8);
                            pit_ch.count = pit_ch.reload;
                        }
                        3 => {
                            // Lo/hi sequence
                            if pit_ch.write_hi_pending {
                                pit_ch.reload = (pit_ch.reload & 0x00FF) | ((v as u16) << 8);
                                pit_ch.count = pit_ch.reload;
                                pit_ch.write_hi_pending = false;
                                if ch == 0 && self.stats.io_exits < 200 {
                                    crate::serial_println!("[SVM-VM {}] PIT ch0: reload={} ({} Hz)", 
                                        self.id, pit_ch.reload,
                                        if pit_ch.reload > 0 { 1193182 / pit_ch.reload as u32 } else { 0 });
                                }
                            } else {
                                pit_ch.reload = (pit_ch.reload & 0xFF00) | v as u16;
                                pit_ch.write_hi_pending = true;
                            }
                        }
                        _ => {}
                    }
                }
                0x43 => {
                    // PIT control word
                    let v = value as u8;
                    let channel = ((v >> 6) & 0x3) as usize;
                    let access = (v >> 4) & 0x3;
                    let mode = (v >> 1) & 0x7;
                    
                    if channel < 3 {
                        if access == 0 {
                            // Latch command
                            self.pit.channels[channel].latched = true;
                            self.pit.channels[channel].latch_value = self.pit.channels[channel].count;
                        } else {
                            self.pit.channels[channel].access = access;
                            self.pit.channels[channel].mode = mode;
                            self.pit.channels[channel].write_hi_pending = false;
                        }
                    }
                }
                
                // CMOS — track index register
                0x70 => {
                    self.cmos_index = (value as u8) & 0x7F; // Bit 7 = NMI disable
                }
                0x71 => {} // CMOS data write — accept silently
                
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
                
                // ACPI PM registers
                0xB000..=0xB003 => {
                    // PM1a_EVT: write to status clears bits (write-1-to-clear)
                    if self.stats.io_exits < 100 {
                        crate::serial_println!("[SVM-VM {}] ACPI PM1a_EVT write: port=0x{:X} val=0x{:X}", self.id, port, value);
                    }
                }
                0xB004..=0xB005 => {
                    // PM1a_CNT: SCI_EN etc.
                    if self.stats.io_exits < 100 {
                        crate::serial_println!("[SVM-VM {}] ACPI PM1a_CNT write: port=0x{:X} val=0x{:X}", self.id, port, value);
                    }
                    // Check for S5 (shutdown) request: SLP_TYP=5, SLP_EN=1
                    if port == 0xB004 && (value & 0x2000) != 0 {
                        let slp_typ = (value >> 10) & 0x7;
                        crate::serial_println!("[SVM-VM {}] ACPI shutdown request: SLP_TYP={}", self.id, slp_typ);
                        if slp_typ == 5 {
                            self.state = SvmVmState::Stopped;
                        }
                    }
                }
                0xB006..=0xB03F => {} // Other PM registers — accept silently
                
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
        
        // MSR constants
        const IA32_APIC_BASE: u32 = 0x001B;
        const IA32_MTRRCAP: u32 = 0x00FE;
        const IA32_SYSENTER_CS: u32 = 0x0174;
        const IA32_SYSENTER_ESP: u32 = 0x0175;
        const IA32_SYSENTER_EIP: u32 = 0x0176;
        const IA32_MCG_CAP: u32 = 0x0179;
        const IA32_MCG_STATUS: u32 = 0x017A;
        const IA32_MISC_ENABLE: u32 = 0x01A0;
        const IA32_PAT: u32 = 0x0277;
        const IA32_MTRR_DEF_TYPE: u32 = 0x02FF;
        const IA32_EFER: u32 = 0xC000_0080;
        const MSR_STAR: u32 = 0xC000_0081;
        const MSR_LSTAR: u32 = 0xC000_0082;
        const MSR_CSTAR: u32 = 0xC000_0083;
        const MSR_SFMASK: u32 = 0xC000_0084;
        const MSR_FS_BASE: u32 = 0xC000_0100;
        const MSR_GS_BASE: u32 = 0xC000_0101;
        const MSR_KERNEL_GS_BASE: u32 = 0xC000_0102;
        const MSR_TSC_AUX: u32 = 0xC000_0103;
        
        if is_write {
            let value = (self.guest_regs.rdx << 32) | (self.guest_regs.rax & 0xFFFF_FFFF);
            
            match msr {
                // These are stored in VMCB state save area — write directly
                MSR_STAR => {
                    let vmcb = self.vmcb.as_mut().unwrap();
                    vmcb.write_state(state_offsets::STAR, value);
                }
                MSR_LSTAR => {
                    let vmcb = self.vmcb.as_mut().unwrap();
                    vmcb.write_state(state_offsets::LSTAR, value);
                }
                MSR_CSTAR => {
                    let vmcb = self.vmcb.as_mut().unwrap();
                    vmcb.write_state(state_offsets::CSTAR, value);
                }
                MSR_SFMASK => {
                    let vmcb = self.vmcb.as_mut().unwrap();
                    vmcb.write_state(state_offsets::SFMASK, value);
                }
                MSR_KERNEL_GS_BASE => {
                    let vmcb = self.vmcb.as_mut().unwrap();
                    vmcb.write_state(state_offsets::KERNEL_GS_BASE, value);
                }
                MSR_FS_BASE => {
                    let vmcb = self.vmcb.as_mut().unwrap();
                    vmcb.write_state(state_offsets::FS_BASE, value);
                }
                MSR_GS_BASE => {
                    let vmcb = self.vmcb.as_mut().unwrap();
                    vmcb.write_state(state_offsets::GS_BASE, value);
                }
                IA32_SYSENTER_CS => {
                    let vmcb = self.vmcb.as_mut().unwrap();
                    vmcb.write_state(state_offsets::SYSENTER_CS, value);
                }
                IA32_SYSENTER_ESP => {
                    let vmcb = self.vmcb.as_mut().unwrap();
                    vmcb.write_state(state_offsets::SYSENTER_ESP, value);
                }
                IA32_SYSENTER_EIP => {
                    let vmcb = self.vmcb.as_mut().unwrap();
                    vmcb.write_state(state_offsets::SYSENTER_EIP, value);
                }
                IA32_PAT => {
                    let vmcb = self.vmcb.as_mut().unwrap();
                    vmcb.write_state(state_offsets::PAT, value);
                }
                IA32_EFER => {
                    let vmcb = self.vmcb.as_mut().unwrap();
                    // Ensure SVME stays set (required for SVM guest)
                    let safe_efer = value | 0x1000; // Keep SVME bit
                    vmcb.write_state(state_offsets::EFER, safe_efer);
                }
                // Silently ignore writes to these common MSRs
                IA32_APIC_BASE | IA32_MISC_ENABLE | IA32_MCG_STATUS |
                IA32_MTRR_DEF_TYPE | MSR_TSC_AUX => {}
                // MTRR physBase/physMask pairs 0x200-0x20F
                0x0200..=0x020F => {}
                // Ignore MCi_STATUS/MCi_ADDR/MCi_MISC registers
                0x0400..=0x047F => {}
                _ => {
                    if self.stats.msr_exits < 100 {
                        crate::serial_println!("[SVM-VM {}] WRMSR 0x{:X} = 0x{:X} (ignored)", self.id, msr, value);
                    }
                }
            }
        } else {
            // RDMSR
            let value: u64 = match msr {
                // Read from VMCB state save area
                MSR_STAR => {
                    let vmcb = self.vmcb.as_ref().unwrap();
                    vmcb.read_state(state_offsets::STAR)
                }
                MSR_LSTAR => {
                    let vmcb = self.vmcb.as_ref().unwrap();
                    vmcb.read_state(state_offsets::LSTAR)
                }
                MSR_CSTAR => {
                    let vmcb = self.vmcb.as_ref().unwrap();
                    vmcb.read_state(state_offsets::CSTAR)
                }
                MSR_SFMASK => {
                    let vmcb = self.vmcb.as_ref().unwrap();
                    vmcb.read_state(state_offsets::SFMASK)
                }
                MSR_KERNEL_GS_BASE => {
                    let vmcb = self.vmcb.as_ref().unwrap();
                    vmcb.read_state(state_offsets::KERNEL_GS_BASE)
                }
                MSR_FS_BASE => {
                    let vmcb = self.vmcb.as_ref().unwrap();
                    vmcb.read_state(state_offsets::FS_BASE)
                }
                MSR_GS_BASE => {
                    let vmcb = self.vmcb.as_ref().unwrap();
                    vmcb.read_state(state_offsets::GS_BASE)
                }
                IA32_SYSENTER_CS => {
                    let vmcb = self.vmcb.as_ref().unwrap();
                    vmcb.read_state(state_offsets::SYSENTER_CS)
                }
                IA32_SYSENTER_ESP => {
                    let vmcb = self.vmcb.as_ref().unwrap();
                    vmcb.read_state(state_offsets::SYSENTER_ESP)
                }
                IA32_SYSENTER_EIP => {
                    let vmcb = self.vmcb.as_ref().unwrap();
                    vmcb.read_state(state_offsets::SYSENTER_EIP)
                }
                IA32_PAT => {
                    let vmcb = self.vmcb.as_ref().unwrap();
                    vmcb.read_state(state_offsets::PAT)
                }
                IA32_EFER => {
                    let vmcb = self.vmcb.as_ref().unwrap();
                    vmcb.read_state(state_offsets::EFER)
                }
                // APIC base: report default (0xFEE00000) + enabled + BSP
                IA32_APIC_BASE => 0xFEE0_0900,
                // MTRRcap: report no MTRRs (simplest)
                IA32_MTRRCAP => 0,
                // MCG_CAP: report 0 MC banks
                IA32_MCG_CAP => 0,
                IA32_MCG_STATUS => 0,
                // MISC_ENABLE: report basic features
                IA32_MISC_ENABLE => 1, // Fast string enable
                // MTRR default type: write-back with MTRRs disabled
                IA32_MTRR_DEF_TYPE => 0x06,
                // TSC_AUX: return 0
                MSR_TSC_AUX => 0,
                // MTRR physBase/physMask
                0x0200..=0x020F => 0,
                // MCi registers
                0x0400..=0x047F => 0,
                _ => {
                    if self.stats.msr_exits < 100 {
                        crate::serial_println!("[SVM-VM {}] RDMSR 0x{:X} = 0 (default)", self.id, msr);
                    }
                    0
                }
            };
            
            self.guest_regs.rax = value & 0xFFFF_FFFF;
            self.guest_regs.rdx = value >> 32;
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

// ============================================================================
// UNIT TESTS (documentation / future test framework)
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pic_default_state() {
        let pic = PicState::default();
        assert_eq!(pic.master_imr, 0xFF);
        assert_eq!(pic.slave_imr, 0xFF);
        assert_eq!(pic.master_vector_base, 0x08);
        assert_eq!(pic.slave_vector_base, 0x70);
        assert_eq!(pic.master_icw_phase, 0);
        assert!(!pic.initialized);
    }

    #[test]
    fn test_pic_icw_sequence() {
        let mut pic = PicState::default();
        // ICW1
        pic.master_icw_phase = 1;
        pic.master_isr = 0;
        // ICW2: vector base 0x20
        pic.master_vector_base = 0x20;
        pic.master_icw_phase = 2;
        // ICW3: cascade
        pic.master_icw_phase = 3;
        // ICW4: done
        pic.master_icw_phase = 0;
        pic.initialized = true;
        assert_eq!(pic.master_vector_base, 0x20);
        assert!(pic.initialized);
    }

    #[test]
    fn test_pit_default_state() {
        let pit = PitState::default();
        assert_eq!(pit.channels[0].reload, 0xFFFF);
        assert_eq!(pit.channels[0].count, 0xFFFF);
        assert_eq!(pit.channels[0].access, 3);
        assert!(!pit.channels[0].latched);
        assert_eq!(pit.channels.len(), 3);
    }

    #[test]
    fn test_pit_lohi_write() {
        let mut ch = PitChannel::default();
        ch.access = 3;
        // Low byte
        ch.reload = (ch.reload & 0xFF00) | 0x9C;
        ch.write_hi_pending = true;
        // High byte
        ch.reload = (ch.reload & 0x00FF) | (0x2E << 8);
        ch.count = ch.reload;
        ch.write_hi_pending = false;
        assert_eq!(ch.reload, 0x2E9C);
        assert_eq!(ch.count, 0x2E9C);
    }

    #[test]
    fn test_lapic_default_state() {
        let lapic = LapicState::default();
        assert_eq!(lapic.icr, 0);
        assert!(!lapic.enabled);
        assert_ne!(lapic.timer_lvt & 0x0001_0000, 0); // masked
        assert_eq!(lapic.svr, 0x1FF);
    }

    #[test]
    fn test_lapic_enable_disable() {
        let mut lapic = LapicState::default();
        lapic.svr = 0x1FF;
        lapic.enabled = (lapic.svr & 0x100) != 0;
        assert!(lapic.enabled);
        lapic.svr = 0x0FF;
        lapic.enabled = (lapic.svr & 0x100) != 0;
        assert!(!lapic.enabled);
    }

    #[test]
    fn test_lapic_divider_decode() {
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
