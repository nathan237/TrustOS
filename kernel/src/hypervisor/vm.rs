//! VM (Virtual Machine) Management
//!
//! Gestion des machines virtuelles:
//! - Création et destruction
//! - État et contrôle
//! - Handler de VM exit

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
        let vmx_basic = vmx::read_msr(0x480);
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
        
        // Configurer l'état de l'host
        let exit_handler = vm_exit_handler as *const () as u64;
        let host_stack = alloc::vec![0u8; 8192];
        let host_stack_ptr = host_stack.as_ptr() as u64 + 8192;
        core::mem::forget(host_stack); // Ne pas libérer le stack
        
        vmcs.setup_host_state(exit_handler, host_stack_ptr)?;
        
        self.state = VmState::Running;
        
        crate::serial_println!("[VM {}] Starting at RIP=0x{:X}, RSP=0x{:X}", 
                              self.id, entry_point, stack_ptr);
        
        // Lancer la VM !
        self.run_loop()?;
        
        Ok(())
    }
    
    /// Boucle d'exécution de la VM
    fn run_loop(&mut self) -> Result<()> {
        loop {
            // VM Entry
            let first_launch = self.stats.vm_exits == 0;
            
            if first_launch {
                vmx::vmlaunch()?;
            } else {
                vmx::vmresume()?;
            }
            
            // Si on arrive ici, c'est un VM exit
            self.stats.vm_exits += 1;
            
            // Traiter le VM exit
            let continue_running = self.handle_vm_exit()?;
            
            if !continue_running {
                break;
            }
        }
        
        self.state = VmState::Stopped;
        Ok(())
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
                self.handle_cpuid()?;
                // Avancer RIP après CPUID (2 bytes)
                let vmcs = self.vmcs.as_ref().unwrap();
                vmcs.write(fields::GUEST_RIP, guest_rip + 2)?;
                Ok(true)
            }
            
            exit_reason::HLT => {
                self.stats.hlt_exits += 1;
                crate::serial_println!("[VM {}] Guest executed HLT at 0x{:X}", self.id, guest_rip);
                Ok(false) // Arrêter la VM
            }
            
            exit_reason::IO_INSTRUCTION => {
                self.stats.io_exits += 1;
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
                // Hypercall depuis le guest
                let result = self.handle_vmcall()?;
                let vmcs = self.vmcs.as_ref().unwrap();
                vmcs.write(fields::GUEST_RIP, guest_rip + instr_len)?;
                Ok(result)
            }
            
            exit_reason::TRIPLE_FAULT => {
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
        // Lire EAX (function) et ECX (sub-function) du guest
        let eax = self.guest_regs.rax as u32;
        let ecx = self.guest_regs.rcx as u32;
        
        // Exécuter CPUID réel et retourner le résultat
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
        
        // Optionnel: masquer certaines fonctionnalités
        // Par exemple, cacher VMX au guest
        let out_ecx = if eax == 1 {
            out_ecx & !(1 << 5) // Cacher VMX
        } else {
            out_ecx
        };
        
        self.guest_regs.rax = out_eax as u64;
        self.guest_regs.rbx = out_ebx as u64;
        self.guest_regs.rcx = out_ecx as u64;
        self.guest_regs.rdx = out_edx as u64;
        
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
        
        if is_write {
            // Le guest veut écrire un MSR
            // On peut soit l'ignorer, soit le passer au hardware
            crate::serial_println!("[VM {}] Guest WRMSR 0x{:X} (ignored)", self.id, msr);
        } else {
            // Le guest veut lire un MSR
            // Retourner une valeur fictive ou réelle
            self.guest_regs.rax = 0;
            self.guest_regs.rdx = 0;
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
            // Charger le guest demandé
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

/// Handler de VM exit (appelé par le CPU après un VM exit)
#[unsafe(naked)]
extern "C" fn vm_exit_handler() {
    // Sauvegarder tous les registres du guest
    core::arch::naked_asm!(
        // Sauvegarder les registres généraux
        "push rax",
        "push rcx",  // Note: Pas de rbx qui est réservé par LLVM
        "push rdx",
        "push rsi",
        "push rdi",
        "push rbp",
        "push r8",
        "push r9",
        "push r10",
        "push r11",
        "push r12",
        "push r13",
        "push r14",
        "push r15",
        
        // Appeler le handler Rust
        "mov rdi, rsp",  // Pointeur vers les registres sauvegardés
        "call {handler}",
        
        // Restaurer les registres
        "pop r15",
        "pop r14",
        "pop r13",
        "pop r12",
        "pop r11",
        "pop r10",
        "pop r9",
        "pop r8",
        "pop rbp",
        "pop rdi",
        "pop rsi",
        "pop rdx",
        "pop rcx",
        "pop rax",
        
        // VMRESUME pour retourner dans le guest
        "vmresume",
        
        // Si vmresume échoue, on arrive ici
        "jmp {error}",
        
        handler = sym vm_exit_handler_rust,
        error = sym vm_exit_error,
    );
}

/// Handler Rust pour les VM exits
extern "C" fn vm_exit_handler_rust(_regs: *mut GuestRegs) {
    // Cette fonction sera appelée à chaque VM exit
    // Le traitement réel est fait dans VirtualMachine::handle_vm_exit
}

/// Gestion des erreurs de VMRESUME
extern "C" fn vm_exit_error() {
    crate::serial_println!("[HV] VMRESUME failed in exit handler!");
    loop {
        unsafe { core::arch::asm!("hlt"); }
    }
}
