











use alloc::string::String;
use alloc::vec::Vec;

use super::{HypervisorError, Result};






#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VmState {
    Created,
    Running,
    Paused,
    Stopped,
    Crashed,
}


#[derive(Debug, Clone, Default)]
pub struct VmStats {
    pub vmexits: u64,
    pub cpuid_exits: u64,
    pub io_exits: u64,
    pub msr_exits: u64,
    pub hlt_exits: u64,
    pub memory_faults: u64,  
    pub hypercall_exits: u64,
    pub interrupt_exits: u64,
}


#[derive(Debug, Clone, Copy)]
pub enum GuestMode {
    
    RealMode { entry_point: u64 },
    
    ProtectedMode { entry_point: u64, stack_ptr: u64 },
    
    LinuxProtectedMode { entry_point: u64, stack_ptr: u64, boot_params: u64 },
    
    LongMode { entry_point: u64, stack_ptr: u64, cr3: u64 },
}


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BackendType {
    IntelVmx,
    AmdSvm,
}









pub trait Sv {
    
    fn hgm(&self) -> BackendType;
    
    
    fn vm_id(&self) -> u64;
    
    
    fn fep(&self) -> &str;
    
    
    fn state(&self) -> VmState;
    
    
    fn stats(&self) -> VmStats;
    
    
    fn memory_size(&self) -> usize;
    
    
    
    
    fn initialize(&mut self) -> Result<()>;
    
    
    fn load_binary(&mut self, data: &[u8], load_address: u64) -> Result<()>;
    
    
    fn jfs(&mut self, mode: GuestMode) -> Result<()>;
    
    
    fn run(&mut self) -> Result<()>;
    
    
    fn ehd(&mut self, jx: &[u8], cmdline: &str, initrd: Option<&[u8]>) -> Result<()>;
    
    
    fn pause(&mut self) -> Result<()>;
    
    
    fn resume(&mut self) -> Result<()>;
    
    
    
    
    fn read_guest_memory(&self, gm: u64, len: usize) -> Option<&[u8]>;
    
    
    fn write_guest_memory(&mut self, gm: u64, data: &[u8]) -> Result<()>;
}







pub fn blh(name: &str, memory_mb: usize) -> Result<alloc::boxed::Box<dyn Sv>> {
    match super::cpu_vendor() {
        super::CpuVendor::Amd => {
            let vm = super::svm_vm::SvmVirtualMachine::new(name, memory_mb)?;
            Ok(alloc::boxed::Box::new(Afb { vm }))
        }
        super::CpuVendor::Intel => {
            let id = super::ALM_.fetch_add(1, core::sync::atomic::Ordering::SeqCst);
            let vm = super::vm::VirtualMachine::new(id, name, memory_mb)?;
            Ok(alloc::boxed::Box::new(Agg { vm }))
        }
        super::CpuVendor::Unknown => {
            Err(HypervisorError::NoVirtualizationSupport)
        }
    }
}






pub struct Afb {
    pub vm: super::svm_vm::SvmVirtualMachine,
}

impl Sv for Afb {
    fn hgm(&self) -> BackendType { BackendType::AmdSvm }
    
    fn vm_id(&self) -> u64 { self.vm.id }
    
    fn fep(&self) -> &str { &self.vm.name }
    
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
        let j = self.vm.get_stats();
        VmStats {
            vmexits: j.vmexits,
            cpuid_exits: j.cpuid_exits,
            io_exits: j.io_exits,
            msr_exits: j.msr_exits,
            hlt_exits: j.hlt_exits,
            memory_faults: j.npf_exits,
            hypercall_exits: j.vmmcall_exits,
            interrupt_exits: j.intr_exits,
        }
    }
    
    fn memory_size(&self) -> usize { self.vm.memory_size }
    
    fn initialize(&mut self) -> Result<()> { self.vm.initialize() }
    
    fn load_binary(&mut self, data: &[u8], load_address: u64) -> Result<()> {
        self.vm.load_binary(data, load_address)
    }
    
    fn jfs(&mut self, mode: GuestMode) -> Result<()> {
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
    
    fn ehd(&mut self, jx: &[u8], cmdline: &str, initrd: Option<&[u8]>) -> Result<()> {
        self.vm.start_linux(jx, cmdline, initrd)
    }
    
    fn pause(&mut self) -> Result<()> { self.vm.pause() }
    fn resume(&mut self) -> Result<()> { self.vm.resume() }
    
    fn read_guest_memory(&self, gm: u64, len: usize) -> Option<&[u8]> {
        self.vm.read_guest_memory(gm, len)
    }
    
    fn write_guest_memory(&mut self, gm: u64, data: &[u8]) -> Result<()> {
        self.vm.write_guest_memory(gm, data)
    }
}






pub struct Agg {
    pub vm: super::vm::VirtualMachine,
}

impl Sv for Agg {
    fn hgm(&self) -> BackendType { BackendType::IntelVmx }
    
    fn vm_id(&self) -> u64 { self.vm.id }
    
    fn fep(&self) -> &str { &self.vm.name }
    
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
    
    fn jfs(&mut self, mode: GuestMode) -> Result<()> {
        match mode {
            GuestMode::ProtectedMode { entry_point, stack_ptr } |
            GuestMode::LinuxProtectedMode { entry_point, stack_ptr, .. } |
            GuestMode::LongMode { entry_point, stack_ptr, .. } => {
                
                
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
        
        
        Ok(())
    }
    
    fn ehd(&mut self, jx: &[u8], cmdline: &str, initrd: Option<&[u8]>) -> Result<()> {
        self.vm.start_linux(jx, cmdline, initrd)
    }
    
    fn pause(&mut self) -> Result<()> {
        self.vm.state = super::vm::VmState::Paused;
        Ok(())
    }
    
    fn resume(&mut self) -> Result<()> {
        self.vm.state = super::vm::VmState::Running;
        Ok(())
    }
    
    fn read_guest_memory(&self, gm: u64, len: usize) -> Option<&[u8]> {
        let offset = gm as usize;
        if offset + len <= self.vm.memory_size {
            
            None 
        } else {
            None
        }
    }
    
    fn write_guest_memory(&mut self, gm: u64, data: &[u8]) -> Result<()> {
        self.vm.load_binary(data, gm)
    }
}
