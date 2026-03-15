//! Intel VT-x (VMX) Operations
//!
//! Implémentation des opérations VMX de base:
//! - Détection du support VMX
//! - VMXON / VMXOFF
//! - Instructions VMX
//! - MSR-based control field adjustment
//! - Physical address translation for hardware structures

use core::arch::asm;
use alloc::boxed::Box;
use super::{HypervisorError, Result, VmxCapabilities};

/// Région VMXON (4KB alignée)
#[repr(C, align(4096))]
pub struct VmxonRegion {
    pub revision_id: u32,
    pub data: [u8; 4092],
}

impl VmxonRegion {
    pub fn new(revision_id: u32) -> Self {
        VmxonRegion {
            revision_id,
            data: [0; 4092],
        }
    }
}

/// Région VMXON allouée (keep Box alive so memory isn't freed)
static mut VMXON_REGION: Option<Box<VmxonRegion>> = None;

// ============================================================================
// MSR CONSTANTS
// ============================================================================

pub const IA32_VMX_BASIC: u32 = 0x480;
pub const IA32_VMX_PINBASED_CTLS: u32 = 0x481;
pub const IA32_VMX_PROCBASED_CTLS: u32 = 0x482;
pub const IA32_VMX_EXIT_CTLS: u32 = 0x483;
pub const IA32_VMX_ENTRY_CTLS: u32 = 0x484;
pub const IA32_VMX_PROCBASED_CTLS2: u32 = 0x48B;
pub const IA32_VMX_TRUE_PINBASED_CTLS: u32 = 0x48D;
pub const IA32_VMX_TRUE_PROCBASED_CTLS: u32 = 0x48E;
pub const IA32_VMX_TRUE_EXIT_CTLS: u32 = 0x48F;
pub const IA32_VMX_TRUE_ENTRY_CTLS: u32 = 0x490;
pub const IA32_FEATURE_CONTROL: u32 = 0x3A;

/// Adjust a VMX control field value according to the allowed 0/1 settings MSR.
/// Low 32 bits of the MSR = "allowed 0-settings" (bits that MUST be 1)
/// High 32 bits of the MSR = "allowed 1-settings" (bits that MAY be 1)
pub fn adjust_vmx_control(msr: u32, desired: u32) -> u32 {
    let msr_val = read_msr(msr);
    let must_be_1 = msr_val as u32;        // Low 32 bits
    let may_be_1 = (msr_val >> 32) as u32; // High 32 bits
    // Set all bits that must be 1, clear bits that must be 0
    (desired | must_be_1) & may_be_1
}

/// Check if "true" MSR controls are supported (bit 55 of IA32_VMX_BASIC)
pub fn has_true_msrs() -> bool {
    let vmx_basic = read_msr(IA32_VMX_BASIC);
    (vmx_basic & (1u64 << 55)) != 0
}

/// Get the appropriate pin-based controls MSR
pub fn pinbased_ctls_msr() -> u32 {
    if has_true_msrs() { IA32_VMX_TRUE_PINBASED_CTLS } else { IA32_VMX_PINBASED_CTLS }
}

/// Get the appropriate primary proc-based controls MSR
pub fn procbased_ctls_msr() -> u32 {
    if has_true_msrs() { IA32_VMX_TRUE_PROCBASED_CTLS } else { IA32_VMX_PROCBASED_CTLS }
}

/// Get the appropriate exit controls MSR
pub fn exit_ctls_msr() -> u32 {
    if has_true_msrs() { IA32_VMX_TRUE_EXIT_CTLS } else { IA32_VMX_EXIT_CTLS }
}

/// Get the appropriate entry controls MSR
pub fn entry_ctls_msr() -> u32 {
    if has_true_msrs() { IA32_VMX_TRUE_ENTRY_CTLS } else { IA32_VMX_ENTRY_CTLS }
}

/// Convert a virtual (HHDM) address to physical for VMX hardware structures
pub fn virt_to_phys_vmx(virt: u64) -> u64 {
    crate::memory::virt_to_phys(virt).unwrap_or(virt)
}

/// Vérifier le support VMX via CPUID
pub fn check_vmx_support() -> Result<VmxCapabilities> {
    let mut caps = VmxCapabilities {
        supported: false,
        ept_supported: false,
        unrestricted_guest: false,
        vpid_supported: false,
        vmcs_revision_id: 0,
    };
    
    // CPUID.1:ECX.VMX[bit 5] = 1 si VMX supporté
    let cpuid_result: u32;
    unsafe {
        asm!(
            "push rbx",       // Sauvegarder rbx
            "mov eax, 1",
            "cpuid",
            "mov {0:e}, ecx",
            "pop rbx",        // Restaurer rbx
            out(reg) cpuid_result,
            out("eax") _,
            out("ecx") _,
            out("edx") _,
        );
    }
    
    caps.supported = (cpuid_result & (1 << 5)) != 0;
    
    if !caps.supported {
        return Ok(caps);
    }
    
    // Lire IA32_VMX_BASIC MSR pour obtenir le revision ID
    let vmx_basic = read_msr(IA32_VMX_BASIC);
    caps.vmcs_revision_id = (vmx_basic & 0x7FFF_FFFF) as u32;
    
    // Vérifier EPT et autres fonctionnalités via IA32_VMX_PROCBASED_CTLS2
    // D'abord vérifier si secondary controls sont supportés
    let procbased_ctls = read_msr(IA32_VMX_PROCBASED_CTLS);
    let secondary_ctls_allowed = (procbased_ctls >> 32) & (1 << 31) != 0;
    
    if secondary_ctls_allowed {
        let procbased_ctls2 = read_msr(IA32_VMX_PROCBASED_CTLS2);
        let allowed_1 = (procbased_ctls2 >> 32) as u32;
        
        caps.ept_supported = (allowed_1 & (1 << 1)) != 0;           // Enable EPT
        caps.unrestricted_guest = (allowed_1 & (1 << 7)) != 0;      // Unrestricted guest
        caps.vpid_supported = (allowed_1 & (1 << 5)) != 0;          // Enable VPID
    }
    
    Ok(caps)
}

/// Activer VMX dans CR4
pub fn enable_vmx() -> Result<()> {
    // Vérifier IA32_FEATURE_CONTROL MSR
    let feature_control = read_msr(IA32_FEATURE_CONTROL);
    
    // Bit 0: Lock bit
    // Bit 2: Enable VMX outside SMX
    if (feature_control & 1) != 0 {
        // Lock bit set, vérifier si VMX est activé
        if (feature_control & (1 << 2)) == 0 {
            crate::serial_println!("[VMX] ERROR: VMX disabled by BIOS (locked)");
            return Err(HypervisorError::VmxNotSupported);
        }
    } else {
        // Lock bit not set, on peut configurer
        let new_value = feature_control | (1 << 2) | 1; // Enable VMX + Lock
        write_msr(0x3A, new_value);
        crate::serial_println!("[VMX] Enabled VMX in IA32_FEATURE_CONTROL");
    }
    
    // Activer VMX dans CR4
    unsafe {
        asm!(
            "mov rax, cr4",
            "or rax, 0x2000",  // CR4.VMXE (bit 13)
            "mov cr4, rax",
            out("rax") _,
        );
    }
    
    crate::serial_println!("[VMX] CR4.VMXE enabled");
    
    Ok(())
}

/// Exécuter VMXON
pub fn vmxon() -> Result<()> {
    // Lire le revision ID
    let vmx_basic = read_msr(IA32_VMX_BASIC);
    let revision_id = (vmx_basic & 0x7FFF_FFFF) as u32;
    
    // Allouer la région VMXON (4KB alignée)
    let region = Box::new(VmxonRegion::new(revision_id));
    let region_virt = region.as_ref() as *const VmxonRegion as u64;
    let region_phys = virt_to_phys_vmx(region_virt);
    
    crate::serial_println!("[VMX] VMXON region virt=0x{:016X} phys=0x{:016X}, revision 0x{:08X}", 
                          region_virt, region_phys, revision_id);
    
    // Sauvegarder la région (keep alive)
    unsafe {
        VMXON_REGION = Some(region);
    }
    
    // Exécuter VMXON — operand is pointer to memory containing the physical address
    let cf: u8;
    let zf: u8;
    unsafe {
        asm!(
            "vmxon [{addr}]",
            "setc {cf}",
            "setz {zf}",
            addr = in(reg) &region_phys,
            cf = out(reg_byte) cf,
            zf = out(reg_byte) zf,
        );
    }
    
    if cf != 0 || zf != 0 {
        crate::serial_println!("[VMX] VMXON failed! CF={} ZF={}", cf, zf);
        return Err(HypervisorError::VmxonFailed);
    }
    
    crate::serial_println!("[VMX] VMXON successful - Now in VMX root operation");
    
    Ok(())
}

/// Exécuter VMXOFF
pub fn vmxoff() -> Result<()> {
    unsafe {
        asm!("vmxoff");
    }
    
    // Désactiver VMX dans CR4
    unsafe {
        asm!(
            "mov rax, cr4",
            "and rax, ~0x2000",  // Clear CR4.VMXE
            "mov cr4, rax",
            out("rax") _,
        );
    }
    
    // Libérer la région VMXON
    unsafe {
        VMXON_REGION = None;
    }
    
    crate::serial_println!("[VMX] VMXOFF complete - Left VMX operation");
    
    Ok(())
}

/// VMCLEAR - Initialiser/nettoyer une VMCS
/// `vmcs_phys` is the physical address of the VMCS region.
pub fn vmclear(vmcs_phys: u64) -> Result<()> {
    let cf: u8;
    let zf: u8;
    unsafe {
        asm!(
            "vmclear [{addr}]",
            "setc {cf}",
            "setz {zf}",
            addr = in(reg) &vmcs_phys,
            cf = out(reg_byte) cf,
            zf = out(reg_byte) zf,
        );
    }
    
    if cf != 0 || zf != 0 {
        crate::serial_println!("[VMX] VMCLEAR failed for phys=0x{:X} CF={} ZF={}", vmcs_phys, cf, zf);
        return Err(HypervisorError::VmclearFailed);
    }
    
    Ok(())
}

/// VMPTRLD - Charger une VMCS comme courante
/// `vmcs_phys` is the physical address of the VMCS region.
pub fn vmptrld(vmcs_phys: u64) -> Result<()> {
    let cf: u8;
    let zf: u8;
    unsafe {
        asm!(
            "vmptrld [{addr}]",
            "setc {cf}",
            "setz {zf}",
            addr = in(reg) &vmcs_phys,
            cf = out(reg_byte) cf,
            zf = out(reg_byte) zf,
        );
    }
    
    if cf != 0 || zf != 0 {
        crate::serial_println!("[VMX] VMPTRLD failed for phys=0x{:X} CF={} ZF={}", vmcs_phys, cf, zf);
        return Err(HypervisorError::VmptrldFailed);
    }
    
    Ok(())
}

/// VMREAD - Lire un champ de la VMCS
pub fn vmread(field: u64) -> Result<u64> {
    let value: u64;
    let cf: u8;
    let zf: u8;
    
    unsafe {
        asm!(
            "vmread {val}, {field}",
            "setc {cf}",
            "setz {zf}",
            val = out(reg) value,
            field = in(reg) field,
            cf = out(reg_byte) cf,
            zf = out(reg_byte) zf,
        );
    }
    
    if cf != 0 || zf != 0 {
        return Err(HypervisorError::VmreadFailed);
    }
    
    Ok(value)
}

/// VMWRITE - Écrire un champ dans la VMCS
pub fn vmwrite(field: u64, value: u64) -> Result<()> {
    let cf: u8;
    let zf: u8;
    
    unsafe {
        asm!(
            "vmwrite {field}, {val}",
            "setc {cf}",
            "setz {zf}",
            field = in(reg) field,
            val = in(reg) value,
            cf = out(reg_byte) cf,
            zf = out(reg_byte) zf,
        );
    }
    
    if cf != 0 || zf != 0 {
        return Err(HypervisorError::VmwriteFailed);
    }
    
    Ok(())
}

/// VMLAUNCH attempt — returns Ok(()) only if it fails (because success = we're in the guest).
/// On a real VM exit, HOST_RIP takes control, not this function.
/// This is used in the run loop: vmlaunch → guest runs → VM exit → host_rip handler.
pub fn vmlaunch() -> Result<()> {
    let cf: u8;
    let zf: u8;
    
    unsafe {
        asm!(
            "vmlaunch",
            "setc {cf}",
            "setz {zf}",
            cf = out(reg_byte) cf,
            zf = out(reg_byte) zf,
        );
    }
    
    // If we get here, VMLAUNCH failed (on success, we'd be in the guest)
    let error_code = vmread(0x4400).unwrap_or(0xFFFF); // VM_INSTRUCTION_ERROR
    crate::serial_println!("[VMX] VMLAUNCH failed! CF={} ZF={} error={}", cf, zf, error_code);
    Err(HypervisorError::VmlaunchFailed)
}

/// VMRESUME attempt — same semantics as vmlaunch
pub fn vmresume() -> Result<()> {
    let cf: u8;
    let zf: u8;
    
    unsafe {
        asm!(
            "vmresume",
            "setc {cf}",
            "setz {zf}",
            cf = out(reg_byte) cf,
            zf = out(reg_byte) zf,
        );
    }
    
    let error_code = vmread(0x4400).unwrap_or(0xFFFF);
    crate::serial_println!("[VMX] VMRESUME failed! CF={} ZF={} error={}", cf, zf, error_code);
    Err(HypervisorError::VmresumeFailed)
}

/// Lire un MSR
pub fn read_msr(msr: u32) -> u64 {
    let low: u32;
    let high: u32;
    
    unsafe {
        asm!(
            "rdmsr",
            in("ecx") msr,
            out("eax") low,
            out("edx") high,
        );
    }
    
    ((high as u64) << 32) | (low as u64)
}

/// Écrire un MSR
pub fn write_msr(msr: u32, value: u64) {
    let low = value as u32;
    let high = (value >> 32) as u32;
    
    unsafe {
        asm!(
            "wrmsr",
            in("ecx") msr,
            in("eax") low,
            in("edx") high,
        );
    }
}
