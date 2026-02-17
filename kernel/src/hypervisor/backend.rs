//! Unified Hypervisor Backend Trait
//!
//! Abstracts the differences between Intel VT-x (VMX) and AMD-V (SVM)
//! behind a common trait, so higher layers (linux_vm, shell, API) don't
//! need to specialize on the CPU vendor.
//!
//! The trait covers the entire VM lifecycle:
//!   create → initialize → load → configure → run → pause/resume → destroy
//!
//! Each backend implementation wraps its native VM type (VirtualMachine or
//! SvmVirtualMachine) and translates trait methods into backend-specific calls.

use alloc::string::String;
use alloc::vec::Vec;

use super::{HypervisorError, Result};

// ============================================================================
// COMMON TYPES
// ============================================================================

/// Unified VM state across backends
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VmState {
    Created,
    Running,
    Paused,
    Stopped,
    Crashed,
}

/// Unified VM statistics
#[derive(Debug, Clone, Default)]
pub struct VmStats {
    pub vmexits: u64,
    pub cpuid_exits: u64,
    pub io_exits: u64,
    pub msr_exits: u64,
    pub hlt_exits: u64,
    pub memory_faults: u64,  // EPT violations / NPF
    pub hypercall_exits: u64,
    pub interrupt_exits: u64,
}

/// Guest execution mode for setup
#[derive(Debug, Clone, Copy)]
pub enum GuestMode {
    /// 16-bit real mode (entry_point only)
    RealMode { entry_point: u64 },
    /// 32-bit protected mode (entry_point + stack)
    ProtectedMode { entry_point: u64, stack_ptr: u64 },
    /// 32-bit protected mode for Linux boot protocol (with boot_params GPA)
    LinuxProtectedMode { entry_point: u64, stack_ptr: u64, boot_params: u64 },
    /// 64-bit long mode (entry_point + stack + page table CR3)
    LongMode { entry_point: u64, stack_ptr: u64, cr3: u64 },
}

/// Backend type identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BackendType {
    IntelVmx,
    AmdSvm,
}

// ============================================================================
// THE TRAIT
// ============================================================================

/// Unified hypervisor backend trait.
///
/// Implemented by both Intel VT-x and AMD SVM backends to provide a
/// common interface for VM management.
pub trait HypervisorBackend {
    /// Backend type
    fn backend_type(&self) -> BackendType;
    
    /// Get the VM's unique ID
    fn vm_id(&self) -> u64;
    
    /// Get the VM's name
    fn vm_name(&self) -> &str;
    
    /// Get current VM state
    fn state(&self) -> VmState;
    
    /// Get VM statistics
    fn stats(&self) -> VmStats;
    
    /// Get guest memory size in bytes
    fn memory_size(&self) -> usize;
    
    // ── Lifecycle ───────────────────────────────────────────────
    
    /// Initialize hardware structures (VMCS/EPT or VMCB/NPT)
    fn initialize(&mut self) -> Result<()>;
    
    /// Load raw binary data into guest physical memory
    fn load_binary(&mut self, data: &[u8], load_address: u64) -> Result<()>;
    
    /// Configure the guest execution mode (real/protected/long/Linux)
    fn setup_guest_mode(&mut self, mode: GuestMode) -> Result<()>;
    
    /// Start executing the VM (enters the run loop, blocks until exit)
    fn run(&mut self) -> Result<()>;
    
    /// Boot a Linux kernel via the boot protocol
    fn boot_linux(&mut self, bzimage: &[u8], cmdline: &str, initrd: Option<&[u8]>) -> Result<()>;
    
    /// Pause VM execution
    fn pause(&mut self) -> Result<()>;
    
    /// Resume VM execution
    fn resume(&mut self) -> Result<()>;
    
    // ── Guest memory access ─────────────────────────────────────
    
    /// Read from guest physical memory
    fn read_guest_memory(&self, gpa: u64, len: usize) -> Option<&[u8]>;
    
    /// Write to guest physical memory
    fn write_guest_memory(&mut self, gpa: u64, data: &[u8]) -> Result<()>;
}

// ============================================================================
// BACKEND CREATION
// ============================================================================

/// Create a new VM using the appropriate backend for the current CPU.
/// Returns a boxed trait object.
pub fn create_vm(name: &str, memory_mb: usize) -> Result<alloc::boxed::Box<dyn HypervisorBackend>> {
    match super::cpu_vendor() {
        super::CpuVendor::Amd => {
            let vm = super::svm_vm::SvmVirtualMachine::new(name, memory_mb)?;
            Ok(alloc::boxed::Box::new(SvmBackend { vm }))
        }
        super::CpuVendor::Intel => {
            let id = super::VM_COUNT.fetch_add(1, core::sync::atomic::Ordering::SeqCst);
            let vm = super::vm::VirtualMachine::new(id, name, memory_mb)?;
            Ok(alloc::boxed::Box::new(VmxBackend { vm }))
        }
        super::CpuVendor::Unknown => {
            Err(HypervisorError::NoVirtualizationSupport)
        }
    }
}

// ============================================================================
// AMD SVM BACKEND
// ============================================================================

/// AMD SVM backend wrapping SvmVirtualMachine
pub struct SvmBackend {
    pub vm: super::svm_vm::SvmVirtualMachine,
}

impl HypervisorBackend for SvmBackend {
    fn backend_type(&self) -> BackendType { BackendType::AmdSvm }
    
    fn vm_id(&self) -> u64 { self.vm.id }
    
    fn vm_name(&self) -> &str { &self.vm.name }
    
    fn state(&self) -> VmState {
        match self.vm.get_state() {
            super::svm_vm::SvmVmState::Created => VmState::Created,
            super::svm_vm::SvmVmState::Running => VmState::Running,
            super::svm_vm::SvmVmState::Paused  => VmState::Paused,
            super::svm_vm::SvmVmState::Stopped => VmState::Stopped,
            super::svm_vm::SvmVmState::Crashed => VmState::Crashed,
        }
    }
    
    fn stats(&self) -> VmStats {
        let s = self.vm.get_stats();
        VmStats {
            vmexits: s.vmexits,
            cpuid_exits: s.cpuid_exits,
            io_exits: s.io_exits,
            msr_exits: s.msr_exits,
            hlt_exits: s.hlt_exits,
            memory_faults: s.npf_exits,
            hypercall_exits: s.vmmcall_exits,
            interrupt_exits: s.intr_exits,
        }
    }
    
    fn memory_size(&self) -> usize { self.vm.memory_size }
    
    fn initialize(&mut self) -> Result<()> { self.vm.initialize() }
    
    fn load_binary(&mut self, data: &[u8], load_address: u64) -> Result<()> {
        self.vm.load_binary(data, load_address)
    }
    
    fn setup_guest_mode(&mut self, mode: GuestMode) -> Result<()> {
        match mode {
            GuestMode::RealMode { entry_point } => self.vm.setup_real_mode(entry_point),
            GuestMode::ProtectedMode { entry_point, stack_ptr } => {
                self.vm.setup_protected_mode(entry_point, stack_ptr)
            }
            GuestMode::LinuxProtectedMode { entry_point, stack_ptr, boot_params } => {
                self.vm.setup_protected_mode_for_linux(entry_point, stack_ptr, boot_params)
            }
            GuestMode::LongMode { entry_point, stack_ptr, cr3 } => {
                self.vm.setup_long_mode(entry_point, stack_ptr, cr3)
            }
        }
    }
    
    fn run(&mut self) -> Result<()> { self.vm.start() }
    
    fn boot_linux(&mut self, bzimage: &[u8], cmdline: &str, initrd: Option<&[u8]>) -> Result<()> {
        self.vm.start_linux(bzimage, cmdline, initrd)
    }
    
    fn pause(&mut self) -> Result<()> { self.vm.pause() }
    fn resume(&mut self) -> Result<()> { self.vm.resume() }
    
    fn read_guest_memory(&self, gpa: u64, len: usize) -> Option<&[u8]> {
        self.vm.read_guest_memory(gpa, len)
    }
    
    fn write_guest_memory(&mut self, gpa: u64, data: &[u8]) -> Result<()> {
        self.vm.write_guest_memory(gpa, data)
    }
}

// ============================================================================
// INTEL VT-x BACKEND
// ============================================================================

/// Intel VT-x backend wrapping VirtualMachine
pub struct VmxBackend {
    pub vm: super::vm::VirtualMachine,
}

impl HypervisorBackend for VmxBackend {
    fn backend_type(&self) -> BackendType { BackendType::IntelVmx }
    
    fn vm_id(&self) -> u64 { self.vm.id }
    
    fn vm_name(&self) -> &str { &self.vm.name }
    
    fn state(&self) -> VmState {
        match self.vm.state {
            super::vm::VmState::Created => VmState::Created,
            super::vm::VmState::Running => VmState::Running,
            super::vm::VmState::Paused  => VmState::Paused,
            super::vm::VmState::Stopped => VmState::Stopped,
            super::vm::VmState::Crashed => VmState::Crashed,
        }
    }
    
    fn stats(&self) -> VmStats {
        VmStats {
            vmexits: self.vm.stats.vm_exits,
            cpuid_exits: self.vm.stats.cpuid_exits,
            io_exits: self.vm.stats.io_exits,
            msr_exits: self.vm.stats.msr_exits,
            hlt_exits: self.vm.stats.hlt_exits,
            memory_faults: self.vm.stats.ept_violations,
            hypercall_exits: 0,
            interrupt_exits: 0,
        }
    }
    
    fn memory_size(&self) -> usize { self.vm.memory_size }
    
    fn initialize(&mut self) -> Result<()> { self.vm.initialize() }
    
    fn load_binary(&mut self, data: &[u8], load_address: u64) -> Result<()> {
        self.vm.load_binary(data, load_address)
    }
    
    fn setup_guest_mode(&mut self, mode: GuestMode) -> Result<()> {
        match mode {
            GuestMode::ProtectedMode { entry_point, stack_ptr } |
            GuestMode::LinuxProtectedMode { entry_point, stack_ptr, .. } |
            GuestMode::LongMode { entry_point, stack_ptr, .. } => {
                // VT-x uses VMCS guest state — configure via start()
                // Store entry/stack for later use by run()
                self.vm.start(entry_point, stack_ptr)?;
                Ok(())
            }
            GuestMode::RealMode { entry_point } => {
                self.vm.start(entry_point, 0x8000)?;
                Ok(())
            }
        }
    }
    
    fn run(&mut self) -> Result<()> {
        // VT-x start is done in setup_guest_mode; start() enters run loop
        // If already started, nothing to do (it ran when setup_guest_mode was called)
        Ok(())
    }
    
    fn boot_linux(&mut self, bzimage: &[u8], cmdline: &str, initrd: Option<&[u8]>) -> Result<()> {
        self.vm.start_linux(bzimage, cmdline, initrd)
    }
    
    fn pause(&mut self) -> Result<()> {
        self.vm.state = super::vm::VmState::Paused;
        Ok(())
    }
    
    fn resume(&mut self) -> Result<()> {
        self.vm.state = super::vm::VmState::Running;
        Ok(())
    }
    
    fn read_guest_memory(&self, gpa: u64, len: usize) -> Option<&[u8]> {
        let offset = gpa as usize;
        if offset + len <= self.vm.memory_size {
            // Access via guest_memory field
            None // TODO: Add guest_memory accessor to VirtualMachine
        } else {
            None
        }
    }
    
    fn write_guest_memory(&mut self, gpa: u64, data: &[u8]) -> Result<()> {
        self.vm.load_binary(data, gpa)
    }
}
