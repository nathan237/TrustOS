//! TrustVM - Hyperviseur intégré pour TrustOS
//!
//! Implémentation d'un hyperviseur Type-1 multi-architecture:
//! - Intel VT-x (VMX) pour processeurs Intel
//! - AMD-V (SVM) pour processeurs AMD
//!
//! Permet d'exécuter des systèmes invités (Linux, etc.) dans des VMs isolées
//!
//! Architecture:
//! - VMX Root Mode / SVM Host Mode: TrustOS (l'hôte)
//! - VMX Non-Root Mode / SVM Guest Mode: Guests (Linux, etc.)
//!
//! Phase 4 Features:
//! - AMD SVM support with VMCB and NPT
//! - Unified API for both Intel and AMD
//! - Automatic CPU detection

// Core Intel VT-x virtualization
pub mod vmx;
pub mod vmcs;
pub mod ept;
pub mod vm;

// AMD SVM virtualization (Phase 4)
pub mod svm;
pub mod svm_vm;

// Phase 2: I/O and guests
pub mod console;
pub mod virtfs;
pub mod guests;

// Phase 3: API, isolation, branding
pub mod api;
pub mod vpid;
pub mod isolation;
pub mod branding;

// Phase 6: Virtual Machine Introspection
pub mod vmi;

// Phase 7: Linux Boot Protocol
pub mod linux_loader;

// Phase 5: Linux Subsystem (TSL)
pub mod virtio_console;
pub mod linux_subsystem;
pub mod linux_vm;

use core::sync::atomic::{AtomicBool, AtomicU64, AtomicU8, Ordering};

/// État global de l'hyperviseur
static HYPERVISOR_ENABLED: AtomicBool = AtomicBool::new(false);
static VM_COUNT: AtomicU64 = AtomicU64::new(0);

/// CPU Vendor type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum CpuVendor {
    Unknown = 0,
    Intel = 1,
    Amd = 2,
}

/// Current CPU vendor
static CPU_VENDOR: AtomicU8 = AtomicU8::new(0);

/// Virtualization backend in use
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VirtBackend {
    None,
    IntelVmx,
    AmdSvm,
}

/// Erreurs de l'hyperviseur
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum HypervisorError {
    /// CPU ne supporte pas VMX
    VmxNotSupported,
    /// CPU ne supporte pas SVM (AMD-V)
    SvmNotSupported,
    /// No virtualization support
    NoVirtualizationSupport,
    /// VMX déjà activé
    VmxAlreadyEnabled,
    /// Échec de VMXON
    VmxonFailed,
    /// Échec de VMCLEAR
    VmclearFailed,
    /// Échec de VMPTRLD
    VmptrldFailed,
    /// Échec de VMLAUNCH
    VmlaunchFailed,
    /// Échec de VMRESUME
    VmresumeFailed,
    /// Échec de VMREAD
    VmreadFailed,
    /// Échec de VMWRITE
    VmwriteFailed,
    /// Mémoire insuffisante
    OutOfMemory,
    /// VM non trouvée
    VmNotFound,
    /// Configuration invalide
    InvalidConfiguration,
    /// EPT violation
    EptViolation,
    /// NPT (AMD) violation
    NptViolation,
    /// SVM initialization failed
    SvmInitFailed,
    /// VMCB not loaded (AMD SVM)
    VmcbNotLoaded,
    /// Already running (Linux Subsystem)
    AlreadyRunning,
    /// Invalid state (Linux Subsystem)
    InvalidState,
    /// Invalid binary/kernel format
    InvalidBinary,
    /// Invalid guest type/format
    InvalidGuest,
}

pub type Result<T> = core::result::Result<T, HypervisorError>;

/// Detect CPU vendor
pub fn detect_cpu_vendor() -> CpuVendor {
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
    
    // Intel: "GenuineIntel" (EBX=Genu, EDX=ineI, ECX=ntel)
    let is_intel = vendor_ebx == 0x756E_6547 
        && vendor_edx == 0x4965_6E69 
        && vendor_ecx == 0x6C65_746E;
    
    // AMD: "AuthenticAMD" (EBX=Auth, EDX=enti, ECX=cAMD)
    let is_amd = vendor_ebx == 0x6874_7541 
        && vendor_edx == 0x6974_6E65 
        && vendor_ecx == 0x444D_4163;
    
    if is_intel {
        CPU_VENDOR.store(CpuVendor::Intel as u8, Ordering::SeqCst);
        CpuVendor::Intel
    } else if is_amd {
        CPU_VENDOR.store(CpuVendor::Amd as u8, Ordering::SeqCst);
        CpuVendor::Amd
    } else {
        CpuVendor::Unknown
    }
}

/// Get current CPU vendor (cached)
pub fn cpu_vendor() -> CpuVendor {
    match CPU_VENDOR.load(Ordering::SeqCst) {
        1 => CpuVendor::Intel,
        2 => CpuVendor::Amd,
        _ => CpuVendor::Unknown,
    }
}

/// Get current virtualization backend
pub fn virt_backend() -> VirtBackend {
    if !is_enabled() {
        return VirtBackend::None;
    }
    match cpu_vendor() {
        CpuVendor::Intel => VirtBackend::IntelVmx,
        CpuVendor::Amd => VirtBackend::AmdSvm,
        CpuVendor::Unknown => VirtBackend::None,
    }
}

/// Informations sur les capacités VMX
#[derive(Debug, Clone)]
pub struct VmxCapabilities {
    pub supported: bool,
    pub ept_supported: bool,
    pub unrestricted_guest: bool,
    pub vpid_supported: bool,
    pub vmcs_revision_id: u32,
}

/// Initialiser l'hyperviseur (auto-détecte Intel/AMD)
pub fn init() -> Result<()> {
    crate::serial_println!("{}", branding::startup_banner());
    crate::serial_println!("[HV] Initializing TrustVM hypervisor...");
    
    // Detect CPU vendor
    let vendor = detect_cpu_vendor();
    crate::serial_println!("[HV] CPU Vendor: {:?}", vendor);
    
    match vendor {
        CpuVendor::Intel => init_intel_vmx(),
        CpuVendor::Amd => init_amd_svm(),
        CpuVendor::Unknown => {
            crate::serial_println!("[HV] Unknown CPU vendor - virtualization not supported");
            Err(HypervisorError::NoVirtualizationSupport)
        }
    }
}

/// Initialize Intel VT-x (VMX)
fn init_intel_vmx() -> Result<()> {
    crate::serial_println!("[HV] Initializing Intel VT-x (VMX)...");
    
    // Vérifier le support VMX
    let caps = vmx::check_vmx_support()?;
    
    crate::serial_println!("[HV] VMX supported: {}", caps.supported);
    crate::serial_println!("[HV] EPT supported: {}", caps.ept_supported);
    crate::serial_println!("[HV] VPID supported: {}", caps.vpid_supported);
    crate::serial_println!("[HV] Unrestricted guest: {}", caps.unrestricted_guest);
    crate::serial_println!("[HV] VMCS revision: 0x{:08X}", caps.vmcs_revision_id);
    
    if !caps.supported {
        return Err(HypervisorError::VmxNotSupported);
    }
    
    // Activer VMX
    vmx::enable_vmx()?;
    
    // Exécuter VMXON
    vmx::vmxon()?;
    
    // Initialize VPID support
    let vpid_enabled = vpid::init();
    
    HYPERVISOR_ENABLED.store(true, Ordering::SeqCst);
    
    crate::serial_println!("[HV] TrustVM hypervisor initialized successfully (Intel VT-x)!");
    crate::serial_println!("{}", branding::welcome_message(vpid_enabled, caps.ept_supported));
    
    // Emit initialization event
    api::emit_event(
        api::VmEventType::Created,
        0, // Host/hypervisor
        api::VmEventData::Message(alloc::string::String::from("TrustVM initialized (Intel VT-x)")),
    );
    
    Ok(())
}

/// Initialize AMD SVM
fn init_amd_svm() -> Result<()> {
    crate::serial_println!("[HV] Initializing AMD-V (SVM)...");
    
    // Check SVM support
    if !svm::is_supported() {
        crate::serial_println!("[HV] SVM not supported or disabled in BIOS");
        return Err(HypervisorError::SvmNotSupported);
    }
    
    // Get SVM features
    let features = svm::get_features();
    
    crate::serial_println!("[HV] SVM Revision: {}", features.revision);
    crate::serial_println!("[HV] NPT supported: {}", features.npt);
    crate::serial_println!("[HV] NRIP Save: {}", features.nrip_save);
    crate::serial_println!("[HV] Flush by ASID: {}", features.flush_by_asid);
    crate::serial_println!("[HV] Decode Assists: {}", features.decode_assists);
    crate::serial_println!("[HV] Available ASIDs: {}", features.num_asids);
    
    // Initialize SVM
    svm::init().map_err(|_| HypervisorError::SvmInitFailed)?;
    
    HYPERVISOR_ENABLED.store(true, Ordering::SeqCst);
    
    crate::serial_println!("[HV] TrustVM hypervisor initialized successfully (AMD SVM)!");
    crate::serial_println!("{}", branding::welcome_message(true, features.npt));
    
    // Emit initialization event
    api::emit_event(
        api::VmEventType::Created,
        0, // Host/hypervisor
        api::VmEventData::Message(alloc::string::String::from("TrustVM initialized (AMD SVM)")),
    );
    
    Ok(())
}

/// Vérifier si l'hyperviseur est actif
pub fn is_enabled() -> bool {
    HYPERVISOR_ENABLED.load(Ordering::SeqCst)
}

/// Créer une nouvelle VM
pub fn create_vm(name: &str, memory_mb: usize) -> Result<u64> {
    if !is_enabled() {
        return Err(HypervisorError::NoVirtualizationSupport);
    }
    
    crate::serial_println!("[HV] Creating VM '{}' (Memory: {}MB)", name, memory_mb);
    
    // Use appropriate backend based on CPU vendor
    match cpu_vendor() {
        CpuVendor::Amd => {
            svm_vm::create_vm(name, memory_mb)
        }
        CpuVendor::Intel => {
            let vm_id = VM_COUNT.fetch_add(1, Ordering::SeqCst);
            vm::create_vm(vm_id, name, memory_mb)?;
            Ok(vm_id)
        }
        CpuVendor::Unknown => {
            Err(HypervisorError::NoVirtualizationSupport)
        }
    }
}

/// Lancer une VM
pub fn start_vm(vm_id: u64) -> Result<()> {
    match cpu_vendor() {
        CpuVendor::Amd => {
            svm_vm::with_vm(vm_id, |vm| vm.start()).unwrap_or(Err(HypervisorError::VmNotFound))?;
            Ok(())
        }
        CpuVendor::Intel => vm::start_vm(vm_id),
        CpuVendor::Unknown => Err(HypervisorError::NoVirtualizationSupport),
    }
}

/// Lancer une VM avec un guest spécifique
pub fn start_vm_with_guest(vm_id: u64, guest_name: &str) -> Result<()> {
    match cpu_vendor() {
        CpuVendor::Amd => {
            // Check if this is a Linux guest
            let is_linux = guest_name == "linux-test" 
                || guest_name.ends_with(".bzimage") 
                || guest_name.ends_with(".bzImage");
            
            if is_linux {
                // Linux boot protocol path
                let bzimage_data = if guest_name == "linux-test" {
                    linux_loader::create_test_linux_kernel()
                } else {
                    guests::get_guest(guest_name)
                        .ok_or(HypervisorError::VmNotFound)?
                        .to_vec()
                };
                
                svm_vm::with_vm(vm_id, |vm| {
                    vm.start_linux(
                        &bzimage_data,
                        "console=ttyS0 earlyprintk=serial nokaslr",
                        None,
                    )
                }).unwrap_or(Err(HypervisorError::VmNotFound))?;
                Ok(())
            } else {
                // Legacy binary guest path
                let guest_data = guests::get_guest(guest_name)
                    .ok_or(HypervisorError::VmNotFound)?;
                
                let is_protected = guest_name == "pm-test" || guest_name == "protected";
                
                svm_vm::with_vm(vm_id, |vm| {
                    // Initialize if needed
                    if vm.vmcb.is_none() {
                        vm.initialize()?;
                    }
                    // Load binary at 0x1000
                    vm.load_binary(&guest_data, 0x1000)?;
                    if is_protected {
                        // Protected mode: entry=0x1000, stack=0x8000
                        vm.setup_protected_mode(0x1000, 0x8000)?;
                    } else {
                        // Real mode (default)
                        vm.setup_real_mode(0x1000)?;
                    }
                    vm.start()
                }).unwrap_or(Err(HypervisorError::VmNotFound))?;
                Ok(())
            }
        }
        CpuVendor::Intel => vm::start_vm_with_guest(vm_id, guest_name),
        CpuVendor::Unknown => Err(HypervisorError::NoVirtualizationSupport),
    }
}

/// Arrêter une VM
pub fn stop_vm(vm_id: u64) -> Result<()> {
    match cpu_vendor() {
        CpuVendor::Amd => {
            svm_vm::with_vm(vm_id, |vm| vm.pause()).unwrap_or(Err(HypervisorError::VmNotFound))?;
            Ok(())
        }
        CpuVendor::Intel => vm::stop_vm(vm_id),
        CpuVendor::Unknown => Err(HypervisorError::NoVirtualizationSupport),
    }
}

/// Ajouter un partage de fichiers à une VM
pub fn add_mount(vm_id: u64, host_path: &str, guest_path: &str, readonly: bool) {
    virtfs::add_mount(vm_id, host_path, guest_path, readonly);
}

/// Injecter une entrée dans la console d'une VM
pub fn inject_console_input(vm_id: u64, data: &[u8]) {
    console::inject_input(vm_id, data);
}

/// Récupérer la sortie de la console d'une VM
pub fn get_console_output(vm_id: u64) -> alloc::string::String {
    console::get_output(vm_id)
}

/// Lister les guests disponibles
pub fn list_guests() -> &'static [&'static str] {
    guests::list_guests()
}

/// Obtenir le nombre de VMs
pub fn vm_count() -> u64 {
    VM_COUNT.load(Ordering::SeqCst)
}

/// Désactiver l'hyperviseur
pub fn shutdown() -> Result<()> {
    if !is_enabled() {
        return Ok(());
    }
    
    crate::serial_println!("[HV] Shutting down TrustVM hypervisor...");
    
    // Arrêter toutes les VMs
    // TODO: Itérer sur les VMs et les arrêter
    
    // VMXOFF
    vmx::vmxoff()?;
    
    HYPERVISOR_ENABLED.store(false, Ordering::SeqCst);
    
    crate::serial_println!("[HV] Hypervisor shutdown complete");
    
    Ok(())
}

// ============================================================================
// PHASE 3: TRUSTVM API
// ============================================================================

/// Get TrustVM capabilities
pub fn get_capabilities() -> api::Capabilities {
    api::get_capabilities()
}

/// Get TrustVM version string
pub fn version() -> &'static str {
    api::VERSION_STRING
}

/// Get recent VM events
pub fn get_events(count: usize) -> alloc::vec::Vec<api::VmEvent> {
    api::get_recent_events(count)
}

/// Create a communication channel with a VM
pub fn create_channel(vm_id: u64, name: &str) -> u64 {
    api::create_channel(vm_id, name, 4096)
}

/// Send data on a channel
pub fn channel_send(channel_id: u64, data: &[u8]) -> core::result::Result<usize, &'static str> {
    api::channel_send(channel_id, data)
}

/// Set resource quota for a VM
pub fn set_vm_quota(vm_id: u64, max_memory: usize, max_vcpus: usize) {
    let quota = api::ResourceQuota {
        max_memory,
        max_vcpus,
        ..api::ResourceQuota::default()
    };
    api::set_quota(vm_id, quota);
}

// ============================================================================
// PHASE 3: ISOLATION API
// ============================================================================

/// Check if VPID is enabled
pub fn vpid_enabled() -> bool {
    vpid::is_enabled()
}

/// Get number of allocated VPIDs
pub fn vpid_count() -> usize {
    vpid::allocated_count()
}

/// Get EPT violation count
pub fn ept_violations() -> u64 {
    isolation::violation_count()
}

/// Get recent EPT violations
pub fn recent_ept_violations(count: usize) -> alloc::vec::Vec<isolation::EptViolation> {
    isolation::recent_violations(count)
}

/// Check if execute-only EPT pages are supported
pub fn supports_execute_only_ept() -> bool {
    isolation::supports_execute_only()
}

// ============================================================================
// PHASE 3: BRANDING
// ============================================================================

/// Get the TrustVM logo (small)
pub fn logo() -> &'static str {
    branding::LOGO_SMALL
}

/// Render capabilities display
pub fn render_capabilities() -> alloc::string::String {
    let caps = get_capabilities();
    branding::render_capabilities(caps.as_u64())
}

/// Render security status
pub fn render_security_status() -> alloc::string::String {
    branding::render_security_status(
        vpid::is_enabled(),
        true, // EPT always enabled when hypervisor is on
        isolation::supports_execute_only(),
        isolation::violation_count(),
    )
}

// ============================================================================
// PHASE 4: BACKEND INFO
// ============================================================================

/// Get backend description string
pub fn backend_info() -> alloc::string::String {
    match virt_backend() {
        VirtBackend::IntelVmx => {
            alloc::format!("Intel VT-x (VMX) - VMCS, EPT, VPID")
        }
        VirtBackend::AmdSvm => {
            let features = svm::get_features();
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

/// Check if we're using AMD SVM
pub fn is_amd_svm() -> bool {
    cpu_vendor() == CpuVendor::Amd && is_enabled()
}

/// Check if we're using Intel VMX
pub fn is_intel_vmx() -> bool {
    cpu_vendor() == CpuVendor::Intel && is_enabled()
}

/// Get SVM features (if on AMD)
pub fn svm_features() -> Option<svm::SvmFeatures> {
    if cpu_vendor() == CpuVendor::Amd {
        Some(svm::get_features())
    } else {
        None
    }
}


