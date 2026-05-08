

















pub mod vmx;
pub mod vmcs;
pub mod ept;
pub mod vm;


pub mod svm;
pub mod svm_vm;


pub mod console;
pub mod virtfs;
pub mod guests;


pub mod api;
pub mod vpid;
pub mod isolation;
pub mod branding;


pub mod vmi;


pub mod linux_loader;
pub mod acpi;


pub mod mmio;
pub mod ioapic;


pub mod backend;
pub mod hpet;


pub mod pci;


pub mod virtio_blk;


pub mod tests;


pub mod debug_monitor;


pub mod virtio_console;
pub mod linux_subsystem;
pub mod linux_vm;

use core::sync::atomic::{AtomicBool, AtomicU64, AtomicU8, Ordering};


static VG_: AtomicBool = AtomicBool::new(false);
static ALM_: AtomicU64 = AtomicU64::new(0);


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum CpuVendor {
    Unknown = 0,
    Intel = 1,
    Amd = 2,
}


static ACG_: AtomicU8 = AtomicU8::new(0);


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VirtBackend {
    None,
    IntelVmx,
    AmdSvm,
}


#[derive(Debug, Clone, Copy, PartialEq)]
pub enum HypervisorError {
    
    VmxNotSupported,
    
    SvmNotSupported,
    
    NoVirtualizationSupport,
    
    VmxAlreadyEnabled,
    
    VmxonFailed,
    
    VmclearFailed,
    
    VmptrldFailed,
    
    VmlaunchFailed,
    
    VmresumeFailed,
    
    VmreadFailed,
    
    VmwriteFailed,
    
    OutOfMemory,
    
    VmNotFound,
    
    InvalidConfiguration,
    
    Ev,
    
    NptViolation,
    
    SvmInitFailed,
    
    VmcbNotLoaded,
    
    AlreadyRunning,
    
    InvalidState,
    
    InvalidBinary,
    
    InvalidGuest,
}

pub type Result<T> = core::result::Result<T, HypervisorError>;


pub fn blt() -> CpuVendor {
    let vendor_ebx: u32;
    let vendor_ecx: u32;
    let vendor_edx: u32;
    
    unsafe {
        core::arch::asm!(
            "push rbx",
            "cpuid",
            "mov {0:e}, ebx",
            "pop rbx",
            out(reg) vendor_ebx,
            inout("eax") 0u32 => _,
            lateout("ecx") vendor_ecx,
            lateout("edx") vendor_edx,
            options(nostack, preserves_flags)
        );
    }
    
    
    let mst = vendor_ebx == 0x756E_6547 
        && vendor_edx == 0x4965_6E69 
        && vendor_ecx == 0x6C65_746E;
    
    
    let erg = vendor_ebx == 0x6874_7541 
        && vendor_edx == 0x6974_6E65 
        && vendor_ecx == 0x444D_4163;
    
    if mst {
        ACG_.store(CpuVendor::Intel as u8, Ordering::SeqCst);
        CpuVendor::Intel
    } else if erg {
        ACG_.store(CpuVendor::Amd as u8, Ordering::SeqCst);
        CpuVendor::Amd
    } else {
        CpuVendor::Unknown
    }
}


pub fn cpu_vendor() -> CpuVendor {
    match ACG_.load(Ordering::SeqCst) {
        1 => CpuVendor::Intel,
        2 => CpuVendor::Amd,
        _ => CpuVendor::Unknown,
    }
}


pub fn psd() -> VirtBackend {
    if !lq() {
        return VirtBackend::None;
    }
    match cpu_vendor() {
        CpuVendor::Intel => VirtBackend::IntelVmx,
        CpuVendor::Amd => VirtBackend::AmdSvm,
        CpuVendor::Unknown => VirtBackend::None,
    }
}


#[derive(Debug, Clone)]
pub struct Nv {
    pub supported: bool,
    pub ept_supported: bool,
    pub unrestricted_guest: bool,
    pub vpid_supported: bool,
    pub vmcs_revision_id: u32,
}


pub fn init() -> Result<()> {
    crate::serial_println!("{}", branding::owm());
    crate::serial_println!("[HV] Initializing TrustVM hypervisor...");
    
    
    let vendor = blt();
    crate::serial_println!("[HV] CPU Vendor: {:?}", vendor);
    
    match vendor {
        CpuVendor::Intel => mpi(),
        CpuVendor::Amd => mou(),
        CpuVendor::Unknown => {
            crate::serial_println!("[HV] Unknown CPU vendor - virtualization not supported");
            Err(HypervisorError::NoVirtualizationSupport)
        }
    }
}


fn mpi() -> Result<()> {
    crate::serial_println!("[HV] Initializing Intel VT-x (VMX)...");
    
    
    let caps = vmx::ehv()?;
    
    crate::serial_println!("[HV] VMX supported: {}", caps.supported);
    crate::serial_println!("[HV] EPT supported: {}", caps.ept_supported);
    crate::serial_println!("[HV] VPID supported: {}", caps.vpid_supported);
    crate::serial_println!("[HV] Unrestricted guest: {}", caps.unrestricted_guest);
    crate::serial_println!("[HV] VMCS revision: 0x{:08X}", caps.vmcs_revision_id);
    
    if !caps.supported {
        return Err(HypervisorError::VmxNotSupported);
    }
    
    
    vmx::lpv()?;
    
    
    vmx::psy()?;
    
    
    let csm = vpid::init();
    
    VG_.store(true, Ordering::SeqCst);
    
    crate::serial_println!("[HV] TrustVM hypervisor initialized successfully (Intel VT-x)!");
    crate::serial_println!("{}", branding::jqz(csm, caps.ept_supported));
    
    
    api::bzf(
        api::VmEventType::Created,
        0, 
        api::VmEventData::Az(alloc::string::String::from("TrustVM initialized (Intel VT-x)")),
    );
    
    Ok(())
}


fn mou() -> Result<()> {
    crate::serial_println!("[HV] Initializing AMD-V (SVM)...");
    
    
    if !svm::is_supported() {
        crate::serial_println!("[HV] SVM not supported or disabled in BIOS");
        return Err(HypervisorError::SvmNotSupported);
    }
    
    
    let features = svm::ckb();
    
    crate::serial_println!("[HV] SVM Revision: {}", features.revision);
    crate::serial_println!("[HV] NPT supported: {}", features.npt);
    crate::serial_println!("[HV] NRIP Save: {}", features.nrip_save);
    crate::serial_println!("[HV] Flush by ASID: {}", features.flush_by_asid);
    crate::serial_println!("[HV] Decode Assists: {}", features.decode_assists);
    crate::serial_println!("[HV] Available ASIDs: {}", features.num_asids);
    
    
    svm::init().map_err(|_| HypervisorError::SvmInitFailed)?;
    
    VG_.store(true, Ordering::SeqCst);
    
    crate::serial_println!("[HV] TrustVM hypervisor initialized successfully (AMD SVM)!");
    crate::serial_println!("{}", branding::jqz(true, features.npt));
    
    
    api::bzf(
        api::VmEventType::Created,
        0, 
        api::VmEventData::Az(alloc::string::String::from("TrustVM initialized (AMD SVM)")),
    );
    
    Ok(())
}


pub fn lq() -> bool {
    VG_.load(Ordering::SeqCst)
}


pub fn blh(name: &str, memory_mb: usize) -> Result<u64> {
    if !lq() {
        return Err(HypervisorError::NoVirtualizationSupport);
    }
    
    crate::serial_println!("[HV] Creating VM '{}' (Memory: {}MB)", name, memory_mb);
    
    
    match cpu_vendor() {
        CpuVendor::Amd => {
            svm_vm::blh(name, memory_mb)
        }
        CpuVendor::Intel => {
            let vm_id = ALM_.fetch_add(1, Ordering::SeqCst);
            vm::blh(vm_id, name, memory_mb)?;
            Ok(vm_id)
        }
        CpuVendor::Unknown => {
            Err(HypervisorError::NoVirtualizationSupport)
        }
    }
}


pub fn jil(vm_id: u64) -> Result<()> {
    match cpu_vendor() {
        CpuVendor::Amd => {
            svm_vm::avv(vm_id, |vm| vm.start()).unwrap_or(Err(HypervisorError::VmNotFound))?;
            Ok(())
        }
        CpuVendor::Intel => vm::jil(vm_id),
        CpuVendor::Unknown => Err(HypervisorError::NoVirtualizationSupport),
    }
}


pub fn dev(vm_id: u64, guest_name: &str) -> Result<()> {
    match cpu_vendor() {
        CpuVendor::Amd => {
            
            let msx = guest_name == "linux-test" 
                || guest_name.ends_with(".bzimage") 
                || guest_name.ends_with(".bzImage");
            
            if msx {
                
                let bas = if guest_name == "linux-test" {
                    linux_loader::fpb()
                } else {
                    guests::eoc(guest_name)
                        .ok_or(HypervisorError::VmNotFound)?
                        .to_vec()
                };
                
                svm_vm::avv(vm_id, |vm| {
                    vm.start_linux(
                        &bas,
                        "console=ttyS0 earlyprintk=serial nokaslr",
                        None,
                    )
                }).unwrap_or(Err(HypervisorError::VmNotFound))?;
                Ok(())
            } else {
                
                let mgh = guests::eoc(guest_name)
                    .ok_or(HypervisorError::VmNotFound)?;
                
                let mtl = guest_name == "pm-test" || guest_name == "protected";
                
                svm_vm::avv(vm_id, |vm| {
                    
                    if vm.vmcb.is_none() {
                        vm.initialize()?;
                    }
                    
                    vm.load_binary(&mgh, 0x1000)?;
                    if mtl {
                        
                        vm.setup_protected_mode(0x1000, 0x8000)?;
                    } else {
                        
                        vm.setup_real_mode(0x1000)?;
                    }
                    vm.start()
                }).unwrap_or(Err(HypervisorError::VmNotFound))?;
                Ok(())
            }
        }
        CpuVendor::Intel => vm::dev(vm_id, guest_name),
        CpuVendor::Unknown => Err(HypervisorError::NoVirtualizationSupport),
    }
}


pub fn fbu(vm_id: u64) -> Result<()> {
    match cpu_vendor() {
        CpuVendor::Amd => {
            svm_vm::avv(vm_id, |vm| vm.pause()).unwrap_or(Err(HypervisorError::VmNotFound))?;
            Ok(())
        }
        CpuVendor::Intel => vm::fbu(vm_id),
        CpuVendor::Unknown => Err(HypervisorError::NoVirtualizationSupport),
    }
}


pub fn add_mount(vm_id: u64, host_path: &str, guest_path: &str, readonly: bool) {
    virtfs::add_mount(vm_id, host_path, guest_path, readonly);
}


pub fn gct(vm_id: u64, data: &[u8]) {
    console::inject_input(vm_id, data);
}


pub fn eoa(vm_id: u64) -> alloc::string::String {
    console::mdo(vm_id)
}


pub fn dtj() -> &'static [&'static str] {
    guests::dtj()
}


pub fn vm_count() -> u64 {
    ALM_.load(Ordering::SeqCst)
}


pub fn shutdown() -> Result<()> {
    if !lq() {
        return Ok(());
    }
    
    crate::serial_println!("[HV] Shutting down TrustVM hypervisor...");
    
    
    
    
    
    vmx::psx()?;
    
    VG_.store(false, Ordering::SeqCst);
    
    crate::serial_println!("[HV] Hypervisor shutdown complete");
    
    Ok(())
}






pub fn enz() -> api::Capabilities {
    api::enz()
}


pub fn version() -> &'static str {
    api::DED_
}


pub fn ibl(count: usize) -> alloc::vec::Vec<api::Je> {
    api::mdr(count)
}


pub fn ejc(vm_id: u64, name: &str) -> u64 {
    api::ejc(vm_id, name, 4096)
}


pub fn hkj(ath: u64, data: &[u8]) -> core::result::Result<usize, &'static str> {
    api::hkj(ath, data)
}


pub fn qwq(vm_id: u64, max_memory: usize, max_vcpus: usize) {
    let gph = api::ResourceQuota {
        max_memory,
        max_vcpus,
        ..api::ResourceQuota::default()
    };
    api::opj(vm_id, gph);
}






pub fn csm() -> bool {
    vpid::lq()
}


pub fn jqo() -> usize {
    vpid::jvb()
}


pub fn ept_violations() -> u64 {
    isolation::jqd()
}


pub fn iys(count: usize) -> alloc::vec::Vec<isolation::Ev> {
    isolation::odq(count)
}


pub fn qxz() -> bool {
    isolation::jjv()
}






pub fn logo() -> &'static str {
    branding::BBD_
}


pub fn eyj() -> alloc::string::String {
    let caps = enz();
    branding::eyj(caps.as_u64())
}


pub fn eyk() -> alloc::string::String {
    branding::eyk(
        vpid::lq(),
        true, 
        isolation::jjv(),
        isolation::jqd(),
    )
}






pub fn fhy() -> alloc::string::String {
    match psd() {
        VirtBackend::IntelVmx => {
            alloc::format!("Intel VT-x (VMX) - VMCS, EPT, VPID")
        }
        VirtBackend::AmdSvm => {
            let features = svm::ckb();
            alloc::format!(
                "AMD-V (SVM) Rev{} - VMCB, NPT:{}, ASIDs:{}",
                features.revision,
                if features.npt { "Yes" } else { "No" },
                features.num_asids
            )
        }
        VirtBackend::None => {
            alloc::format!("No virtualization backend active")
        }
    }
}


pub fn qly() -> bool {
    cpu_vendor() == CpuVendor::Amd && lq()
}


pub fn qmk() -> bool {
    cpu_vendor() == CpuVendor::Intel && lq()
}


pub fn qyd() -> Option<svm::SvmFeatures> {
    if cpu_vendor() == CpuVendor::Amd {
        Some(svm::ckb())
    } else {
        None
    }
}

