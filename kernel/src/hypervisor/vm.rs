//! VM (Virtual Machine) Management
//!
//! Gestion des machines virtuelles:
//! - Création et destruction
//! - État et contrôle
//! - Handler de VM exit
//!
//! Architecture: The run loop uses a proper VM exit handler that saves
//! guest registers, returns to Rust code for handling, and then resumes.

use alloc::string::String;
use alloc::vec::Vec;
use alloc::boxed::Box;
use spin::Mutex;
use super::{HypervisorError, Result};
use super::vmcs::{Vmcs, fields, exit_reason};
use super::ept::EptManager;
use super::vmx;

/// État d'une VM
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum VmState {
    Created,
    Running,
    Paused,
    Stopped,
    Crashed,
}

/// Statistiques d'une VM
#[derive(Debug, Clone, Default)]
pub struct VmStats {
    pub vm_exits: u64,
    pub cpuid_exits: u64,
    pub io_exits: u64,
    pub msr_exits: u64,
    pub hlt_exits: u64,
    pub ept_violations: u64,
}

/// Registres du guest sauvegardés lors d'un VM exit
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

/// Structure représentant une VM
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
    /// Console ID
    console_id: Option<usize>,
    /// VPID for TLB isolation
    vpid: Option<u16>,
}

impl VirtualMachine {
    pub fn new(id: u64, name: &str, memory_mb: usize) -> Result<Self> {
        let memory_size = memory_mb * 1024 * 1024;
        
        // Allouer la mémoire pour le guest
        let guest_memory = alloc::vec![0u8; memory_size];
        
        // Créer la console virtuelle
        let console_id = super::console::create_console(id, name);
        
        // Créer le VirtFS
        super::virtfs::create_virtfs(id);
        
        // Allocate VPID for TLB isolation
        let vpid = super::vpid::allocate();
        if vpid.is_some() {
            crate::serial_println!("[VM {}] Allocated VPID {} for TLB isolation", id, vpid.unwrap());
        }
        
        // Emit VM created event
        super::api::emit_event(
            super::api::VmEventType::Created,
            id,
            super::api::VmEventData::Message(alloc::format!("VM '{}' created", name)),
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
    
    /// Ajouter un partage de fichiers
    pub fn add_mount(&mut self, host_path: &str, guest_path: &str, readonly: bool) {
        super::virtfs::add_mount(self.id, host_path, guest_path, readonly);
    }
    
    /// Initialiser la VMCS et l'EPT
    pub fn initialize(&mut self) -> Result<()> {
        crate::serial_println!("[VM {}] Initializing VMCS and EPT", self.id);
        
        // Obtenir le revision ID
        let vmx_basic = vmx::read_msr(vmx::IA32_VMX_BASIC);
        let revision_id = (vmx_basic & 0x7FFF_FFFF) as u32;
        
        // Créer la VMCS
        let mut vmcs = Vmcs::new(revision_id)?;
        vmcs.load()?;
        
        // Configurer les contrôles (includes EPT and VPID enables)
        vmcs.setup_execution_controls()?;
        vmcs.setup_exit_controls()?;
        vmcs.setup_entry_controls()?;
        
        // Configure VPID if allocated
        vmcs.setup_vpid(self.vpid)?;
        
        // Créer l'EPT
        let ept = EptManager::new(self.memory_size)?;
        
        // Configurer l'EPT pointer dans la VMCS
        vmcs.write(fields::EPT_POINTER, ept.ept_pointer().as_u64())?;
        
        self.vmcs = Some(vmcs);
        self.ept = Some(ept);
        
        crate::serial_println!("[VM {}] Initialization complete (VPID={:?})", self.id, self.vpid);
        
        Ok(())
    }
    
    /// Charger un binaire dans la mémoire du guest
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
    
    /// Démarrer la VM
    pub fn start(&mut self, entry_point: u64, stack_ptr: u64) -> Result<()> {
        if self.vmcs.is_none() {
            self.initialize()?;
        }
        
        let vmcs = self.vmcs.as_ref().unwrap();
        
        // Configurer l'état du guest
        vmcs.setup_guest_state(entry_point, stack_ptr)?;
        
        // Configurer l'état de l'host — RIP = vm_exit_stub (naked handler for VM exits)
        let exit_handler = vm_exit_stub as *const () as u64;
        
        // Allocate a host stack for VM exit handler (16KB, 16-byte aligned)
        let host_stack = alloc::vec![0u8; 16384];
        let host_stack_top = (host_stack.as_ptr() as u64 + 16384) & !0xF;
        core::mem::forget(host_stack); // Must not be freed
        
        vmcs.setup_host_state(exit_handler, host_stack_top)?;
        
        self.state = VmState::Running;
        
        crate::serial_println!("[VM {}] Starting at RIP=0x{:X}, RSP=0x{:X}", 
                              self.id, entry_point, stack_ptr);
        crate::serial_println!("[VM {}] Host exit handler=0x{:X}, host stack=0x{:X}",
                              self.id, exit_handler, host_stack_top);
        
        // Lancer la VM
        self.run_loop()?;
        
        Ok(())
    }
    
    /// Start a Linux kernel in this VM.
    ///
    /// Parses the bzImage, loads it into guest memory with boot_params,
    /// page tables, GDT, and configures VMCS for 64-bit Linux boot.
    pub fn start_linux(
        &mut self,
        bzimage_data: &[u8],
        cmdline: &str,
        initrd: Option<&[u8]>,
    ) -> Result<()> {
        use super::linux_loader;
        
        crate::serial_println!("[VM {}] Starting Linux kernel ({} bytes)", self.id, bzimage_data.len());
        
        // Prepare guest memory with kernel + boot structures
        let setup = linux_loader::prepare_linux_vm(
            &mut self.guest_memory,
            bzimage_data,
            cmdline,
            initrd,
        )?;
        
        // Initialize VMCS/EPT if not done
        if self.vmcs.is_none() {
            self.initialize()?;
        }
        
        let vmcs = self.vmcs.as_ref().unwrap();
        
        // Configure VMCS with Linux-specific guest state
        linux_loader::configure_vmcs_for_linux(vmcs, &setup)?;
        
        // Set RSI = boot_params address (Linux boot protocol requirement)
        self.guest_regs.rsi = setup.boot_params_addr;
        self.save_guest_regs_for_entry();
        
        // Set up host state for VM exits
        let exit_handler = vm_exit_stub as *const () as u64;
        let host_stack = alloc::vec![0u8; 16384];
        let host_stack_top = (host_stack.as_ptr() as u64 + 16384) & !0xF;
        core::mem::forget(host_stack);
        
        vmcs.setup_host_state(exit_handler, host_stack_top)?;
        
        self.state = VmState::Running;
        
        crate::serial_println!("[VM {}] Linux: RIP=0x{:X} RSP=0x{:X} RSI(boot_params)=0x{:X} CR3=0x{:X}",
                              self.id, setup.entry_point, setup.stack_ptr,
                              setup.boot_params_addr, setup.cr3);
        
        self.run_loop()?;
        
        Ok(())
    }
    
    /// Boucle d'exécution de la VM
    ///
    /// Architecture: vmx_entry_trampoline (naked) → vmlaunch/vmresume → guest
    /// On VM exit: vm_exit_stub saves guest regs, restores host stack, returns to here.
    fn run_loop(&mut self) -> Result<()> {
        let mut launched = false;
        
        loop {
            // Enter the guest
            let result = vmx_enter_guest(launched);
            
            if result != 0 {
                // VMLAUNCH/VMRESUME failed
                let err = vmx::vmread(fields::VM_INSTRUCTION_ERROR).unwrap_or(0xFFFF);
                crate::serial_println!("[VM {}] VM entry failed! error={}", self.id, err);
                self.state = VmState::Crashed;
                return if launched {
                    Err(HypervisorError::VmresumeFailed)
                } else {
                    Err(HypervisorError::VmlaunchFailed)
                };
            }
            
            launched = true;
            
            // If we get here, a VM exit happened and we're back in host mode
            self.stats.vm_exits += 1;
            
            // Read guest GPRs from the saved area
            self.load_guest_regs_from_exit();
            
            // Traiter le VM exit
            let continue_running = self.handle_vm_exit()?;
            
            if !continue_running {
                break;
            }
            
            // Save guest regs back before resume
            self.save_guest_regs_for_entry();
        }
        
        self.state = VmState::Stopped;
        crate::serial_println!("[VM {}] Stopped after {} exits (cpuid={} io={} hlt={} ept={})",
                              self.id, self.stats.vm_exits, self.stats.cpuid_exits,
                              self.stats.io_exits, self.stats.hlt_exits,
                              self.stats.ept_violations);
        Ok(())
    }
    
    /// Load guest register values from the exit save area
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
    
    /// Save guest register values for the next VM entry
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
    
    /// Gérer un VM exit
    fn handle_vm_exit(&mut self) -> Result<bool> {
        // Lire les infos de base du VMCS
        let (exit_reason, exit_qual, guest_rip, instr_len) = {
            let vmcs = self.vmcs.as_ref().unwrap();
            let reason = vmcs.read(fields::VM_EXIT_REASON)? as u32 & 0xFFFF;
            let qual = vmcs.read(fields::EXIT_QUALIFICATION)?;
            let rip = vmcs.read(fields::GUEST_RIP)?;
            let len = vmcs.read(fields::VM_EXIT_INSTRUCTION_LENGTH).unwrap_or(0);
            (reason, qual, rip, len)
        };
        
        match exit_reason {
            exit_reason::CPUID => {
                self.stats.cpuid_exits += 1;
                crate::lab_mode::trace_bus::emit_vm_exit(
                    self.id, "CPUID", guest_rip,
                    &alloc::format!("EAX=0x{:X}", self.guest_regs.rax)
                );
                self.handle_cpuid()?;
                // Avancer RIP après CPUID (2 bytes)
                let vmcs = self.vmcs.as_ref().unwrap();
                vmcs.write(fields::GUEST_RIP, guest_rip + 2)?;
                Ok(true)
            }
            
            exit_reason::HLT => {
                self.stats.hlt_exits += 1;
                crate::lab_mode::trace_bus::emit_vm_exit(self.id, "HLT", guest_rip, "");
                crate::serial_println!("[VM {}] Guest executed HLT at 0x{:X}", self.id, guest_rip);
                Ok(false) // Arrêter la VM
            }
            
            exit_reason::IO_INSTRUCTION => {
                self.stats.io_exits += 1;
                let port = ((exit_qual >> 16) & 0xFFFF) as u16;
                let dir = if (exit_qual & 8) == 0 { "OUT" } else { "IN" };
                crate::lab_mode::trace_bus::emit_vm_io(self.id, dir, port, self.guest_regs.rax);
                self.handle_io(exit_qual)?;
                let vmcs = self.vmcs.as_ref().unwrap();
                vmcs.write(fields::GUEST_RIP, guest_rip + instr_len)?;
                Ok(true)
            }
            
            exit_reason::RDMSR | exit_reason::WRMSR => {
                self.stats.msr_exits += 1;
                self.handle_msr(exit_reason == exit_reason::WRMSR)?;
                let vmcs = self.vmcs.as_ref().unwrap();
                vmcs.write(fields::GUEST_RIP, guest_rip + instr_len)?;
                Ok(true)
            }
            
            exit_reason::EPT_VIOLATION => {
                self.stats.ept_violations += 1;
                let vmcs = self.vmcs.as_ref().unwrap();
                let guest_phys = vmcs.read(fields::GUEST_PHYSICAL_ADDRESS)?;
                let guest_linear = vmcs.read(fields::GUEST_LINEAR_ADDRESS).ok();
                
                crate::lab_mode::trace_bus::emit_vm_memory(
                    self.id, "EPT_VIOLATION", guest_phys, exit_qual
                );
                
                // Record the violation in isolation module
                super::isolation::record_violation(
                    self.id,
                    guest_phys,
                    guest_linear,
                    exit_qual,
                    guest_rip,
                );
                
                // Emit event
                super::api::emit_event(
                    super::api::VmEventType::EptViolation,
                    self.id,
                    super::api::VmEventData::Address(guest_phys),
                );
                
                Ok(false)
            }
            
            exit_reason::VMCALL => {
                crate::lab_mode::trace_bus::emit_vm_exit(
                    self.id, "VMCALL", guest_rip,
                    &alloc::format!("func=0x{:X}", self.guest_regs.rax)
                );
                // Hypercall depuis le guest
                let result = self.handle_vmcall()?;
                let vmcs = self.vmcs.as_ref().unwrap();
                vmcs.write(fields::GUEST_RIP, guest_rip + instr_len)?;
                Ok(result)
            }
            
            exit_reason::TRIPLE_FAULT => {
                crate::lab_mode::trace_bus::emit_vm_lifecycle(self.id, "TRIPLE FAULT (crashed)");
                crate::serial_println!("[VM {}] TRIPLE FAULT! Guest crashed.", self.id);
                self.state = VmState::Crashed;
                Ok(false)
            }
            
            exit_reason::INVALID_GUEST_STATE => {
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
    
    /// Émuler CPUID
    fn handle_cpuid(&mut self) -> Result<()> {
        let leaf = self.guest_regs.rax as u32;
        let subleaf = self.guest_regs.rcx as u32;
        
        // Handle hypervisor-specific leaves
        match leaf {
            0x4000_0000 => {
                // Hypervisor identification
                self.guest_regs.rax = 0x4000_0001;
                self.guest_regs.rbx = 0x7473_7254; // "Trst"
                self.guest_regs.rcx = 0x7254_534F; // "OSTr"
                self.guest_regs.rdx = 0x534F_7473; // "stOS"
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
        
        // Execute real CPUID
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
                ecx &= !(1 << 5);   // Hide VMX
                ecx |= 1 << 31;     // Hypervisor present
                ecx &= !(1 << 21);  // Hide x2APIC
                ecx &= !(1 << 3);   // Hide MONITOR/MWAIT
            }
            0x0000_000A => {
                eax = 0; ecx = 0; // Hide perf monitoring
            }
            _ => {}
        }
        
        self.guest_regs.rax = eax as u64;
        self.guest_regs.rbx = ebx as u64;
        self.guest_regs.rcx = ecx as u64;
        self.guest_regs.rdx = edx as u64;
        
        Ok(())
    }
    
    /// Gérer une instruction I/O
    fn handle_io(&mut self, exit_qual: u64) -> Result<()> {
        let port = ((exit_qual >> 16) & 0xFFFF) as u16;
        let is_out = (exit_qual & 8) == 0;
        let _is_string = (exit_qual & 16) != 0;
        let _size = (exit_qual & 7) as u8 + 1; // 1, 2, or 4 bytes
        
        if is_out {
            // OUT instruction
            let value = (self.guest_regs.rax & 0xFF) as u8;
            
            // Use virtual console for I/O
            let result = super::console::handle_console_io(self.id, port, true, value);
            self.guest_regs.rax = (self.guest_regs.rax & !0xFF) | (result as u64);
        } else {
            // IN instruction
            let value = super::console::handle_console_io(self.id, port, false, 0);
            self.guest_regs.rax = (self.guest_regs.rax & !0xFF) | (value as u64);
        }
        
        Ok(())
    }
    
    /// Gérer RDMSR/WRMSR
    fn handle_msr(&mut self, is_write: bool) -> Result<()> {
        let msr = self.guest_regs.rcx as u32;
        
        // Common MSR constants
        const IA32_APIC_BASE: u32 = 0x001B;
        const IA32_MISC_ENABLE: u32 = 0x01A0;
        const IA32_PAT: u32 = 0x0277;
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
            let _value = (self.guest_regs.rdx << 32) | (self.guest_regs.rax & 0xFFFF_FFFF);
            // For VT-x, many MSRs are handled by VMCS guest state or MSR load/store areas.
            // Silently accept writes to common MSRs.
            match msr {
                MSR_STAR | MSR_LSTAR | MSR_CSTAR | MSR_SFMASK |
                MSR_FS_BASE | MSR_GS_BASE | MSR_KERNEL_GS_BASE |
                IA32_PAT | IA32_EFER | IA32_APIC_BASE | IA32_MISC_ENABLE |
                MSR_TSC_AUX => {}
                0x0174..=0x0176 => {} // SYSENTER_CS/ESP/EIP
                0x0200..=0x020F => {} // MTRRs
                0x0400..=0x047F => {} // MC banks
                _ => {
                    if self.stats.vm_exits < 100 {
                        crate::serial_println!("[VM {}] WRMSR 0x{:X} (ignored)", self.id, msr);
                    }
                }
            }
        } else {
            // RDMSR — return sensible values
            let value: u64 = match msr {
                IA32_APIC_BASE => 0xFEE0_0900, // Default + enabled + BSP
                IA32_MISC_ENABLE => 1,          // Fast string enable
                IA32_PAT => 0x0007040600070406, // Default PAT
                IA32_EFER => 0x501,             // SCE + LME + LMA
                MSR_TSC_AUX => 0,
                0x00FE => 0,   // MTRRcap
                0x0179 => 0,   // MCG_CAP
                0x017A => 0,   // MCG_STATUS
                0x02FF => 0x06, // MTRR_DEF_TYPE
                0x0200..=0x020F => 0, // MTRRs
                0x0400..=0x047F => 0, // MC banks
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
    
    /// Gérer VMCALL (hypercall)
    fn handle_vmcall(&mut self) -> Result<bool> {
        let function = self.guest_regs.rax;
        
        match function {
            // Hypercall 0: Print string (RBX = ptr, RCX = len)
            0 => {
                crate::serial_println!("[VM {}] Hypercall: print", self.id);
                Ok(true)
            }
            
            // Hypercall 1: Exit VM (RBX = exit code)
            1 => {
                let exit_code = self.guest_regs.rbx;
                crate::serial_println!("[VM {}] Hypercall: exit (code={})", self.id, exit_code);
                Ok(false)
            }
            
            // Hypercall 2: Get time
            2 => {
                let ticks = crate::time::uptime_ms();
                self.guest_regs.rax = ticks;
                Ok(true)
            }
            
            // Hypercall 3: Console write (RBX = char)
            3 => {
                let c = (self.guest_regs.rbx & 0xFF) as u8;
                super::console::handle_console_io(self.id, 0xE9, true, c);
                Ok(true)
            }
            
            // Hypercall 4: Console read
            4 => {
                let c = super::console::handle_console_io(self.id, 0x3F8, false, 0);
                self.guest_regs.rax = c as u64;
                Ok(true)
            }
            
            // Hypercall 0x100+: VirtFS operations
            0x100..=0x1FF => {
                let virtfs_op = (function - 0x100) as u32;
                let args = [
                    self.guest_regs.rbx,
                    self.guest_regs.rcx,
                    self.guest_regs.rdx,
                    self.guest_regs.rsi,
                ];
                let (result, _data) = super::virtfs::handle_hypercall(self.id, virtfs_op, &args);
                self.guest_regs.rax = result as u64;
                Ok(true)
            }
            
            // Hypercall 0x200+: TrustVM API (Phase 3)
            0x200..=0x3FF => {
                let args = [
                    self.guest_regs.rbx,
                    self.guest_regs.rcx,
                    self.guest_regs.rdx,
                    self.guest_regs.rsi,
                ];
                let (result, data) = super::api::handle_api_hypercall(self.id, function, &args);
                
                // Check for special return codes
                if result == -1 && function == super::api::hypercall::SHUTDOWN {
                    // Guest requested shutdown
                    return Ok(false);
                }
                if result == -2 && function == super::api::hypercall::REBOOT {
                    // Guest requested reboot - for now just stop
                    return Ok(false);
                }
                
                self.guest_regs.rax = data;
                Ok(true)
            }
            
            _ => {
                crate::serial_println!("[VM {}] Unknown hypercall: 0x{:X}", self.id, function);
                self.guest_regs.rax = u64::MAX; // Error
                Ok(true)
            }
        }
    }
}

/// Liste globale des VMs
static VMS: Mutex<Vec<VirtualMachine>> = Mutex::new(Vec::new());

/// Créer une nouvelle VM
pub fn create_vm(id: u64, name: &str, memory_mb: usize) -> Result<()> {
    let vm = VirtualMachine::new(id, name, memory_mb)?;
    VMS.lock().push(vm);
    Ok(())
}

/// Démarrer une VM avec un guest prédéfini
pub fn start_vm(id: u64) -> Result<()> {
    start_vm_with_guest(id, "hello")
}

/// Démarrer une VM avec un guest spécifique
pub fn start_vm_with_guest(id: u64, guest_name: &str) -> Result<()> {
    let mut vms = VMS.lock();
    
    for vm in vms.iter_mut() {
        if vm.id == id {
            // Check if this is a Linux-type guest
            if guest_name == "linux-test" || guest_name.ends_with(".bzimage") {
                let bzimage = super::guests::get_guest(guest_name)
                    .unwrap_or_else(|| super::linux_loader::create_test_linux_kernel());
                crate::serial_println!("[VM {}] Loading Linux guest '{}' ({} bytes)", 
                                      id, guest_name, bzimage.len());
                vm.start_linux(&bzimage, "console=ttyS0 earlyprintk=serial nokaslr", None)?;
                return Ok(());
            }
            
            // Standard flat-binary guest
            let code = super::guests::get_guest(guest_name)
                .unwrap_or_else(|| super::guests::hello_guest());
            
            crate::serial_println!("[VM {}] Loading guest '{}' ({} bytes)", id, guest_name, code.len());
            
            vm.load_binary(&code, 0x1000)?;
            vm.start(0x1000, 0x8000)?;
            return Ok(());
        }
    }
    
    Err(HypervisorError::VmNotFound)
}

/// Arrêter une VM
pub fn stop_vm(id: u64) -> Result<()> {
    let mut vms = VMS.lock();
    
    for vm in vms.iter_mut() {
        if vm.id == id {
            vm.state = VmState::Stopped;
            return Ok(());
        }
    }
    
    Err(HypervisorError::VmNotFound)
}

// ============================================================================
// VMX LAUNCH/RESUME + VM EXIT MECHANISM
// ============================================================================

use core::sync::atomic::{AtomicBool, AtomicU8, Ordering};

/// Guest register save area (filled by vm_exit_stub on VM exit)
static mut VM_EXIT_GUEST_REGS: GuestRegs = GuestRegs {
    rax: 0, rbx: 0, rcx: 0, rdx: 0,
    rsi: 0, rdi: 0, rbp: 0,
    r8: 0, r9: 0, r10: 0, r11: 0,
    r12: 0, r13: 0, r14: 0, r15: 0,
};

/// Saved host RSP (from vmx_entry_trampoline, used by vm_exit_stub to return).
/// This points into the trampoline's stack frame where callee-saved regs are pushed.
static mut HOST_SAVED_RSP: u64 = 0;

/// Flag: 0 = VMLAUNCH, 1 = VMRESUME
static VMX_USE_RESUME: AtomicU8 = AtomicU8::new(0);

/// Enter the guest VM via VMLAUNCH or VMRESUME.
///
/// This is the clean entry point called from the Rust run loop.
/// Returns 0 on successful VM exit, 1 on vmlaunch/vmresume failure.
///
/// Architecture:
/// 1. vmx_entry_trampoline (naked): saves host callee-saved regs on stack,
///    saves RSP to HOST_SAVED_RSP, loads guest GPRs, does vmlaunch/vmresume.
/// 2. On success: CPU enters guest. On VM exit, CPU jumps to HOST_RIP = vm_exit_stub.
/// 3. vm_exit_stub (naked): saves guest GPRs to VM_EXIT_GUEST_REGS, restores
///    host RSP from HOST_SAVED_RSP, pops callee-saved, returns 0.
/// 4. On failure: trampoline pops callee-saved, returns 1.
fn vmx_enter_guest(use_resume: bool) -> u64 {
    VMX_USE_RESUME.store(use_resume as u8, Ordering::SeqCst);
    unsafe { vmx_entry_trampoline() }
}

/// Naked trampoline that performs the actual VMX entry.
/// Returns 0 if a VM exit occurred (success), 1 if vmlaunch/vmresume failed.
#[unsafe(naked)]
unsafe extern "C" fn vmx_entry_trampoline() -> u64 {
    core::arch::naked_asm!(
        // ── Save host callee-saved registers on stack ──
        "push rbx",
        "push rbp",
        "push r12",
        "push r13",
        "push r14",
        "push r15",
        "pushfq",
        
        // Save current RSP so vm_exit_stub can restore it
        "lea rax, [{host_rsp}]",
        "mov [rax], rsp",
        
        // ── Check VMLAUNCH vs VMRESUME BEFORE loading guest regs ──
        "lea rax, [{flag}]",
        "movzx eax, byte ptr [rax]",
        "push rax",   // save flag on stack (will pop after loading guest regs)
        
        // ── Load guest GPRs from save area ──
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
        "mov rax, [rcx + 0]",    // guest rax
        "mov rcx, [rcx + 16]",   // guest rcx (must be last — we used it as pointer)
        
        // ── Pop flag and branch ──
        // Problem: popping destroys RSP alignment and a guest register.
        // Instead, let's read the saved flag from the stack without popping,
        // using a different approach.
        //
        // Actually, the flag is still on the stack at [rsp]. But all guest
        // regs are loaded. We can't use any register to test without losing
        // a guest value. Solution: use the stack-based test.
        //
        // Use: compare the value at [rsp] and conditional jump, then adjust RSP.
        "cmp qword ptr [rsp], 0",
        "jne 20f",
        
        // ── VMLAUNCH path ──
        "add rsp, 8",    // pop the flag (clean stack for VMLAUNCH)
        "vmlaunch",
        "jmp 30f",       // failure
        
        // ── VMRESUME path ──
        "20:",
        "add rsp, 8",    // pop the flag
        "vmresume",
        // fall-through to failure
        
        // ── VMLAUNCH/VMRESUME failed ──
        "30:",
        // We need to restore host stack. HOST_SAVED_RSP still has our saved value.
        "lea rax, [{host_rsp}]",
        "mov rsp, [rax]",
        // Pop callee-saved and return 1.
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

/// VM exit stub — HOST_RIP points here.
///
/// Called by CPU after a VM exit. CPU has loaded host segment selectors,
/// CR0/CR3/CR4, RSP=HOST_RSP, RIP=this function.
///
/// We:
/// 1. Save all guest GPRs to VM_EXIT_GUEST_REGS
/// 2. Switch RSP to the trampoline's saved RSP (HOST_SAVED_RSP)
/// 3. Pop callee-saved registers (mirror of trampoline's pushes)
/// 4. Return 0 (back to caller of vmx_entry_trampoline)
#[unsafe(naked)]
extern "C" fn vm_exit_stub() {
    core::arch::naked_asm!(
        // ── Save guest GPRs ──
        // We're on the HOST_RSP stack. Use it temporarily.
        "push rax",
        "lea rax, [{gregs}]",
        
        // Save all guest GPRs except rax (saved on stack)
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
        
        // Save guest rax (was pushed on HOST_RSP stack)
        "pop rbx",
        "mov [rax], rbx",
        
        // ── Switch to trampoline's stack ──
        "lea rax, [{host_rsp}]",
        "mov rsp, [rax]",
        
        // ── Pop callee-saved registers (mirror of trampoline's pushes) ──
        "popfq",
        "pop r15",
        "pop r14",
        "pop r13",
        "pop r12",
        "pop rbp",
        "pop rbx",
        
        // ── Return 0 (success: VM exit handled) ──
        "xor eax, eax",
        "ret",
        
        gregs = sym VM_EXIT_GUEST_REGS,
        host_rsp = sym HOST_SAVED_RSP,
    );
}
