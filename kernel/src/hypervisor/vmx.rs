//! Intel VT-x (VMX) Operations
//!
//! Implémentation des opérations VMX de base:
//! - Détection du support VMX
//! - VMXON / VMXOFF
//! - Instructions VMX

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
        let mut region = VmxonRegion {
            revision_id,
            data: [0; 4092],
        };
        region
    }
}

/// Région VMXON allouée
static mut VMXON_REGION: Option<Box<VmxonRegion>> = None;

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
    let vmx_basic = read_msr(0x480); // IA32_VMX_BASIC
    caps.vmcs_revision_id = (vmx_basic & 0x7FFF_FFFF) as u32;
    
    // Vérifier EPT et autres fonctionnalités via IA32_VMX_PROCBASED_CTLS2
    // D'abord vérifier si secondary controls sont supportés
    let procbased_ctls = read_msr(0x482); // IA32_VMX_PROCBASED_CTLS
    let secondary_ctls_allowed = (procbased_ctls >> 32) & (1 << 31) != 0;
    
    if secondary_ctls_allowed {
        let procbased_ctls2 = read_msr(0x48B); // IA32_VMX_PROCBASED_CTLS2
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
    let feature_control = read_msr(0x3A); // IA32_FEATURE_CONTROL
    
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
    let vmx_basic = read_msr(0x480);
    let revision_id = (vmx_basic & 0x7FFF_FFFF) as u32;
    
    // Allouer la région VMXON (4KB alignée)
    let region = Box::new(VmxonRegion::new(revision_id));
    let region_phys = region.as_ref() as *const VmxonRegion as u64;
    
    crate::serial_println!("[VMX] VMXON region at 0x{:016X}, revision 0x{:08X}", 
                          region_phys, revision_id);
    
    // Sauvegarder la région
    unsafe {
        VMXON_REGION = Some(region);
    }
    
    // Exécuter VMXON
    let result: u8;
    unsafe {
        asm!(
            "vmxon [{0}]",
            "setc {1}",      // CF=1 si échec
            "setz {1}",      // ZF=1 si échec avec erreur
            in(reg) &region_phys,
            out(reg_byte) result,
        );
    }
    
    if result != 0 {
        crate::serial_println!("[VMX] VMXON failed!");
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
pub fn vmclear(vmcs_phys: u64) -> Result<()> {
    let result: u8;
    unsafe {
        asm!(
            "vmclear [{0}]",
            "setc {1}",
            in(reg) &vmcs_phys,
            out(reg_byte) result,
        );
    }
    
    if result != 0 {
        return Err(HypervisorError::VmclearFailed);
    }
    
    Ok(())
}

/// VMPTRLD - Charger une VMCS comme courante
pub fn vmptrld(vmcs_phys: u64) -> Result<()> {
    let result: u8;
    unsafe {
        asm!(
            "vmptrld [{0}]",
            "setc {1}",
            in(reg) &vmcs_phys,
            out(reg_byte) result,
        );
    }
    
    if result != 0 {
        return Err(HypervisorError::VmptrldFailed);
    }
    
    Ok(())
}

/// VMREAD - Lire un champ de la VMCS
pub fn vmread(field: u64) -> Result<u64> {
    let value: u64;
    let result: u8;
    
    unsafe {
        asm!(
            "vmread {0}, {1}",
            "setc {2}",
            out(reg) value,
            in(reg) field,
            out(reg_byte) result,
        );
    }
    
    if result != 0 {
        return Err(HypervisorError::VmreadFailed);
    }
    
    Ok(value)
}

/// VMWRITE - Écrire un champ dans la VMCS
pub fn vmwrite(field: u64, value: u64) -> Result<()> {
    let result: u8;
    
    unsafe {
        asm!(
            "vmwrite {0}, {1}",
            "setc {2}",
            in(reg) field,
            in(reg) value,
            out(reg_byte) result,
        );
    }
    
    if result != 0 {
        return Err(HypervisorError::VmwriteFailed);
    }
    
    Ok(())
}

/// VMLAUNCH - Lancer une VM pour la première fois
pub fn vmlaunch() -> Result<()> {
    let result: u8;
    
    unsafe {
        asm!(
            "vmlaunch",
            "setc {0}",
            out(reg_byte) result,
        );
    }
    
    // Si on arrive ici, c'est que VMLAUNCH a échoué
    // (sinon on serait dans le guest)
    
    if result != 0 {
        let error_code = vmread(0x4400)?; // VM_INSTRUCTION_ERROR
        crate::serial_println!("[VMX] VMLAUNCH failed with error: {}", error_code);
        return Err(HypervisorError::VmlaunchFailed);
    }
    
    Ok(())
}

/// VMRESUME - Reprendre une VM après un VM exit
pub fn vmresume() -> Result<()> {
    let result: u8;
    
    unsafe {
        asm!(
            "vmresume",
            "setc {0}",
            out(reg_byte) result,
        );
    }
    
    if result != 0 {
        let error_code = vmread(0x4400)?;
        crate::serial_println!("[VMX] VMRESUME failed with error: {}", error_code);
        return Err(HypervisorError::VmresumeFailed);
    }
    
    Ok(())
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
