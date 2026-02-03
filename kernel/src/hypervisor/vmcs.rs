//! VMCS (Virtual Machine Control Structure)
//!
//! La VMCS contient l'état complet d'une VM:
//! - État du guest (registres, segments, etc.)
//! - État de l'host
//! - Contrôles d'exécution
//! - Informations sur les VM exits

use super::{HypervisorError, Result};
use super::vmx::{vmclear, vmptrld, vmread, vmwrite};
use alloc::boxed::Box;

/// VMCS Field Encodings (Intel SDM Vol. 3, Appendix B)
pub mod fields {
    // 16-bit control fields
    pub const VPID: u64 = 0x0000;
    
    // 16-bit guest state
    pub const GUEST_ES_SELECTOR: u64 = 0x0800;
    pub const GUEST_CS_SELECTOR: u64 = 0x0802;
    pub const GUEST_SS_SELECTOR: u64 = 0x0804;
    pub const GUEST_DS_SELECTOR: u64 = 0x0806;
    pub const GUEST_FS_SELECTOR: u64 = 0x0808;
    pub const GUEST_GS_SELECTOR: u64 = 0x080A;
    pub const GUEST_LDTR_SELECTOR: u64 = 0x080C;
    pub const GUEST_TR_SELECTOR: u64 = 0x080E;
    
    // 16-bit host state
    pub const HOST_ES_SELECTOR: u64 = 0x0C00;
    pub const HOST_CS_SELECTOR: u64 = 0x0C02;
    pub const HOST_SS_SELECTOR: u64 = 0x0C04;
    pub const HOST_DS_SELECTOR: u64 = 0x0C06;
    pub const HOST_FS_SELECTOR: u64 = 0x0C08;
    pub const HOST_GS_SELECTOR: u64 = 0x0C0A;
    pub const HOST_TR_SELECTOR: u64 = 0x0C0C;
    
    // 64-bit control fields
    pub const IO_BITMAP_A: u64 = 0x2000;
    pub const IO_BITMAP_B: u64 = 0x2002;
    pub const MSR_BITMAP: u64 = 0x2004;
    pub const EPT_POINTER: u64 = 0x201A;
    
    // 64-bit guest state
    pub const VMCS_LINK_POINTER: u64 = 0x2800;
    pub const GUEST_IA32_DEBUGCTL: u64 = 0x2802;
    pub const GUEST_IA32_PAT: u64 = 0x2804;
    pub const GUEST_IA32_EFER: u64 = 0x2806;
    
    // 64-bit host state
    pub const HOST_IA32_PAT: u64 = 0x2C00;
    pub const HOST_IA32_EFER: u64 = 0x2C02;
    
    // 32-bit control fields
    pub const PIN_BASED_VM_EXEC_CONTROLS: u64 = 0x4000;
    pub const CPU_BASED_VM_EXEC_CONTROLS: u64 = 0x4002;
    pub const EXCEPTION_BITMAP: u64 = 0x4004;
    pub const PAGE_FAULT_ERROR_CODE_MASK: u64 = 0x4006;
    pub const PAGE_FAULT_ERROR_CODE_MATCH: u64 = 0x4008;
    pub const CR3_TARGET_COUNT: u64 = 0x400A;
    pub const VM_EXIT_CONTROLS: u64 = 0x400C;
    pub const VM_EXIT_MSR_STORE_COUNT: u64 = 0x400E;
    pub const VM_EXIT_MSR_LOAD_COUNT: u64 = 0x4010;
    pub const VM_ENTRY_CONTROLS: u64 = 0x4012;
    pub const VM_ENTRY_MSR_LOAD_COUNT: u64 = 0x4014;
    pub const VM_ENTRY_INTERRUPTION_INFO: u64 = 0x4016;
    pub const VM_ENTRY_EXCEPTION_ERROR_CODE: u64 = 0x4018;
    pub const VM_ENTRY_INSTRUCTION_LENGTH: u64 = 0x401A;
    pub const SECONDARY_VM_EXEC_CONTROLS: u64 = 0x401E;
    
    // 32-bit guest state
    pub const GUEST_ES_LIMIT: u64 = 0x4800;
    pub const GUEST_CS_LIMIT: u64 = 0x4802;
    pub const GUEST_SS_LIMIT: u64 = 0x4804;
    pub const GUEST_DS_LIMIT: u64 = 0x4806;
    pub const GUEST_FS_LIMIT: u64 = 0x4808;
    pub const GUEST_GS_LIMIT: u64 = 0x480A;
    pub const GUEST_LDTR_LIMIT: u64 = 0x480C;
    pub const GUEST_TR_LIMIT: u64 = 0x480E;
    pub const GUEST_GDTR_LIMIT: u64 = 0x4810;
    pub const GUEST_IDTR_LIMIT: u64 = 0x4812;
    pub const GUEST_ES_ACCESS_RIGHTS: u64 = 0x4814;
    pub const GUEST_CS_ACCESS_RIGHTS: u64 = 0x4816;
    pub const GUEST_SS_ACCESS_RIGHTS: u64 = 0x4818;
    pub const GUEST_DS_ACCESS_RIGHTS: u64 = 0x481A;
    pub const GUEST_FS_ACCESS_RIGHTS: u64 = 0x481C;
    pub const GUEST_GS_ACCESS_RIGHTS: u64 = 0x481E;
    pub const GUEST_LDTR_ACCESS_RIGHTS: u64 = 0x4820;
    pub const GUEST_TR_ACCESS_RIGHTS: u64 = 0x4822;
    pub const GUEST_INTERRUPTIBILITY_STATE: u64 = 0x4824;
    pub const GUEST_ACTIVITY_STATE: u64 = 0x4826;
    pub const GUEST_SYSENTER_CS: u64 = 0x482A;
    
    // 32-bit host state
    pub const HOST_SYSENTER_CS: u64 = 0x4C00;
    
    // Natural-width control fields
    pub const CR0_GUEST_HOST_MASK: u64 = 0x6000;
    pub const CR4_GUEST_HOST_MASK: u64 = 0x6002;
    pub const CR0_READ_SHADOW: u64 = 0x6004;
    pub const CR4_READ_SHADOW: u64 = 0x6006;
    
    // Natural-width guest state
    pub const GUEST_CR0: u64 = 0x6800;
    pub const GUEST_CR3: u64 = 0x6802;
    pub const GUEST_CR4: u64 = 0x6804;
    pub const GUEST_ES_BASE: u64 = 0x6806;
    pub const GUEST_CS_BASE: u64 = 0x6808;
    pub const GUEST_SS_BASE: u64 = 0x680A;
    pub const GUEST_DS_BASE: u64 = 0x680C;
    pub const GUEST_FS_BASE: u64 = 0x680E;
    pub const GUEST_GS_BASE: u64 = 0x6810;
    pub const GUEST_LDTR_BASE: u64 = 0x6812;
    pub const GUEST_TR_BASE: u64 = 0x6814;
    pub const GUEST_GDTR_BASE: u64 = 0x6816;
    pub const GUEST_IDTR_BASE: u64 = 0x6818;
    pub const GUEST_DR7: u64 = 0x681A;
    pub const GUEST_RSP: u64 = 0x681C;
    pub const GUEST_RIP: u64 = 0x681E;
    pub const GUEST_RFLAGS: u64 = 0x6820;
    pub const GUEST_PENDING_DEBUG_EXCEPTIONS: u64 = 0x6822;
    pub const GUEST_SYSENTER_ESP: u64 = 0x6824;
    pub const GUEST_SYSENTER_EIP: u64 = 0x6826;
    
    // Natural-width host state
    pub const HOST_CR0: u64 = 0x6C00;
    pub const HOST_CR3: u64 = 0x6C02;
    pub const HOST_CR4: u64 = 0x6C04;
    pub const HOST_FS_BASE: u64 = 0x6C06;
    pub const HOST_GS_BASE: u64 = 0x6C08;
    pub const HOST_TR_BASE: u64 = 0x6C0A;
    pub const HOST_GDTR_BASE: u64 = 0x6C0C;
    pub const HOST_IDTR_BASE: u64 = 0x6C0E;
    pub const HOST_SYSENTER_ESP: u64 = 0x6C10;
    pub const HOST_SYSENTER_EIP: u64 = 0x6C12;
    pub const HOST_RSP: u64 = 0x6C14;
    pub const HOST_RIP: u64 = 0x6C16;
    
    // Read-only fields
    pub const VM_INSTRUCTION_ERROR: u64 = 0x4400;
    pub const VM_EXIT_REASON: u64 = 0x4402;
    pub const VM_EXIT_INTERRUPTION_INFO: u64 = 0x4404;
    pub const VM_EXIT_INTERRUPTION_ERROR_CODE: u64 = 0x4406;
    pub const IDT_VECTORING_INFO: u64 = 0x4408;
    pub const IDT_VECTORING_ERROR_CODE: u64 = 0x440A;
    pub const VM_EXIT_INSTRUCTION_LENGTH: u64 = 0x440C;
    pub const VM_EXIT_INSTRUCTION_INFO: u64 = 0x440E;
    pub const EXIT_QUALIFICATION: u64 = 0x6400;
    pub const GUEST_LINEAR_ADDRESS: u64 = 0x640A;
    pub const GUEST_PHYSICAL_ADDRESS: u64 = 0x2400;
}

/// VM Exit reasons
pub mod exit_reason {
    pub const EXCEPTION_NMI: u32 = 0;
    pub const EXTERNAL_INTERRUPT: u32 = 1;
    pub const TRIPLE_FAULT: u32 = 2;
    pub const INIT_SIGNAL: u32 = 3;
    pub const SIPI: u32 = 4;
    pub const IO_SMI: u32 = 5;
    pub const OTHER_SMI: u32 = 6;
    pub const INTERRUPT_WINDOW: u32 = 7;
    pub const NMI_WINDOW: u32 = 8;
    pub const TASK_SWITCH: u32 = 9;
    pub const CPUID: u32 = 10;
    pub const GETSEC: u32 = 11;
    pub const HLT: u32 = 12;
    pub const INVD: u32 = 13;
    pub const INVLPG: u32 = 14;
    pub const RDPMC: u32 = 15;
    pub const RDTSC: u32 = 16;
    pub const RSM: u32 = 17;
    pub const VMCALL: u32 = 18;
    pub const VMCLEAR: u32 = 19;
    pub const VMLAUNCH: u32 = 20;
    pub const VMPTRLD: u32 = 21;
    pub const VMPTRST: u32 = 22;
    pub const VMREAD: u32 = 23;
    pub const VMRESUME: u32 = 24;
    pub const VMWRITE: u32 = 25;
    pub const VMXOFF: u32 = 26;
    pub const VMXON: u32 = 27;
    pub const CR_ACCESS: u32 = 28;
    pub const DR_ACCESS: u32 = 29;
    pub const IO_INSTRUCTION: u32 = 30;
    pub const RDMSR: u32 = 31;
    pub const WRMSR: u32 = 32;
    pub const INVALID_GUEST_STATE: u32 = 33;
    pub const MSR_LOADING: u32 = 34;
    pub const MWAIT: u32 = 36;
    pub const MONITOR_TRAP_FLAG: u32 = 37;
    pub const MONITOR: u32 = 39;
    pub const PAUSE: u32 = 40;
    pub const MCE_DURING_ENTRY: u32 = 41;
    pub const TPR_BELOW_THRESHOLD: u32 = 43;
    pub const APIC_ACCESS: u32 = 44;
    pub const VIRTUALIZED_EOI: u32 = 45;
    pub const GDTR_IDTR_ACCESS: u32 = 46;
    pub const LDTR_TR_ACCESS: u32 = 47;
    pub const EPT_VIOLATION: u32 = 48;
    pub const EPT_MISCONFIGURATION: u32 = 49;
    pub const INVEPT: u32 = 50;
    pub const RDTSCP: u32 = 51;
    pub const PREEMPTION_TIMER: u32 = 52;
    pub const INVVPID: u32 = 53;
    pub const WBINVD: u32 = 54;
    pub const XSETBV: u32 = 55;
    pub const APIC_WRITE: u32 = 56;
    pub const RDRAND: u32 = 57;
    pub const INVPCID: u32 = 58;
    pub const VMFUNC: u32 = 59;
    pub const ENCLS: u32 = 60;
    pub const RDSEED: u32 = 61;
    pub const PML_FULL: u32 = 62;
    pub const XSAVES: u32 = 63;
    pub const XRSTORS: u32 = 64;
}

/// Région VMCS (4KB alignée)
#[repr(C, align(4096))]
pub struct VmcsRegion {
    pub revision_id: u32,
    pub abort_indicator: u32,
    pub data: [u8; 4088],
}

impl VmcsRegion {
    pub fn new(revision_id: u32) -> Self {
        VmcsRegion {
            revision_id,
            abort_indicator: 0,
            data: [0; 4088],
        }
    }
}

/// Structure VMCS avec méthodes d'accès
pub struct Vmcs {
    region: Box<VmcsRegion>,
    phys_addr: u64,
    is_current: bool,
}

impl Vmcs {
    /// Créer une nouvelle VMCS
    pub fn new(revision_id: u32) -> Result<Self> {
        let region = Box::new(VmcsRegion::new(revision_id));
        let phys_addr = region.as_ref() as *const VmcsRegion as u64;
        
        // VMCLEAR pour initialiser
        vmclear(phys_addr)?;
        
        Ok(Vmcs {
            region,
            phys_addr,
            is_current: false,
        })
    }
    
    /// Charger cette VMCS comme courante
    pub fn load(&mut self) -> Result<()> {
        vmptrld(self.phys_addr)?;
        self.is_current = true;
        Ok(())
    }
    
    /// Écrire un champ
    pub fn write(&self, field: u64, value: u64) -> Result<()> {
        if !self.is_current {
            return Err(HypervisorError::InvalidConfiguration);
        }
        vmwrite(field, value)
    }
    
    /// Lire un champ
    pub fn read(&self, field: u64) -> Result<u64> {
        if !self.is_current {
            return Err(HypervisorError::InvalidConfiguration);
        }
        vmread(field)
    }
    
    /// Configurer les contrôles d'exécution de base
    pub fn setup_execution_controls(&self) -> Result<()> {
        use fields::*;
        
        // Pin-based controls
        let pin_based = 0u64; // Minimal pour commencer
        self.write(PIN_BASED_VM_EXEC_CONTROLS, pin_based)?;
        
        // CPU-based controls
        let cpu_based = (1 << 7)   // HLT exiting
                      | (1 << 31); // Activate secondary controls
        self.write(CPU_BASED_VM_EXEC_CONTROLS, cpu_based)?;
        
        // Secondary controls with EPT and VPID
        let mut secondary = 0u64;
        
        // Enable EPT (bit 1) - always if supported
        secondary |= 1 << 1;
        
        // Enable VPID (bit 5) - if supported and enabled
        secondary |= super::vpid::get_secondary_controls_vpid();
        
        // Enable unrestricted guest (bit 7) - allows real mode
        // secondary |= 1 << 7; // Optional
        
        self.write(SECONDARY_VM_EXEC_CONTROLS, secondary)?;
        
        // Exception bitmap (intercepter aucune exception pour l'instant)
        self.write(EXCEPTION_BITMAP, 0)?;
        
        Ok(())
    }
    
    /// Configure VPID for this VMCS
    pub fn setup_vpid(&self, vpid: Option<u16>) -> Result<()> {
        use fields::*;
        
        let vpid_value = super::vpid::get_vmcs_vpid(vpid);
        self.write(VPID, vpid_value)?;
        
        if vpid.is_some() {
            crate::serial_println!("[VMCS] VPID set to {}", vpid_value);
        }
        
        Ok(())
    }
    
    /// Configurer les contrôles de sortie VM
    pub fn setup_exit_controls(&self) -> Result<()> {
        use fields::*;
        
        let exit_controls = (1 << 9)   // Host address-space size (64-bit host)
                          | (1 << 15); // Acknowledge interrupt on exit
        self.write(VM_EXIT_CONTROLS, exit_controls)?;
        
        Ok(())
    }
    
    /// Configurer les contrôles d'entrée VM
    pub fn setup_entry_controls(&self) -> Result<()> {
        use fields::*;
        
        let entry_controls = (1 << 9); // IA-32e mode guest
        self.write(VM_ENTRY_CONTROLS, entry_controls)?;
        
        Ok(())
    }
    
    /// Configurer l'état du guest
    pub fn setup_guest_state(&self, entry_point: u64, stack_ptr: u64) -> Result<()> {
        use fields::*;
        
        // Segments (mode 64-bit)
        // CS
        self.write(GUEST_CS_SELECTOR, 0x08)?;
        self.write(GUEST_CS_BASE, 0)?;
        self.write(GUEST_CS_LIMIT, 0xFFFFFFFF)?;
        self.write(GUEST_CS_ACCESS_RIGHTS, 0xA09B)?; // 64-bit code, present, DPL0
        
        // DS, ES, SS
        for (sel, base, limit, ar) in [
            (GUEST_DS_SELECTOR, GUEST_DS_BASE, GUEST_DS_LIMIT, GUEST_DS_ACCESS_RIGHTS),
            (GUEST_ES_SELECTOR, GUEST_ES_BASE, GUEST_ES_LIMIT, GUEST_ES_ACCESS_RIGHTS),
            (GUEST_SS_SELECTOR, GUEST_SS_BASE, GUEST_SS_LIMIT, GUEST_SS_ACCESS_RIGHTS),
        ] {
            self.write(sel, 0x10)?;
            self.write(base, 0)?;
            self.write(limit, 0xFFFFFFFF)?;
            self.write(ar, 0xC093)?; // Data, present, DPL0
        }
        
        // FS, GS (base 0 pour l'instant)
        self.write(GUEST_FS_SELECTOR, 0)?;
        self.write(GUEST_FS_BASE, 0)?;
        self.write(GUEST_FS_LIMIT, 0xFFFF)?;
        self.write(GUEST_FS_ACCESS_RIGHTS, 0x10000)?; // Unusable
        
        self.write(GUEST_GS_SELECTOR, 0)?;
        self.write(GUEST_GS_BASE, 0)?;
        self.write(GUEST_GS_LIMIT, 0xFFFF)?;
        self.write(GUEST_GS_ACCESS_RIGHTS, 0x10000)?;
        
        // LDTR (unusable)
        self.write(GUEST_LDTR_SELECTOR, 0)?;
        self.write(GUEST_LDTR_BASE, 0)?;
        self.write(GUEST_LDTR_LIMIT, 0)?;
        self.write(GUEST_LDTR_ACCESS_RIGHTS, 0x10000)?;
        
        // TR (Task Register - requis)
        self.write(GUEST_TR_SELECTOR, 0)?;
        self.write(GUEST_TR_BASE, 0)?;
        self.write(GUEST_TR_LIMIT, 0x67)?;
        self.write(GUEST_TR_ACCESS_RIGHTS, 0x8B)?; // 64-bit TSS, busy
        
        // Control registers
        self.write(GUEST_CR0, 0x80050033)?; // PE, MP, ET, NE, WP, AM, PG
        self.write(GUEST_CR3, 0)?; // À configurer avec les page tables du guest
        self.write(GUEST_CR4, 0x2620)?; // PAE, MCE, PGE, OSFXSR, OSXMMEXCPT
        
        // RIP, RSP, RFLAGS
        self.write(GUEST_RIP, entry_point)?;
        self.write(GUEST_RSP, stack_ptr)?;
        self.write(GUEST_RFLAGS, 0x2)?; // Reserved bit 1 = 1
        
        // VMCS link pointer (required, -1 for no nested virtualization)
        self.write(VMCS_LINK_POINTER, 0xFFFFFFFF_FFFFFFFF)?;
        
        // Activity state (active)
        self.write(GUEST_ACTIVITY_STATE, 0)?;
        self.write(GUEST_INTERRUPTIBILITY_STATE, 0)?;
        
        // DR7
        self.write(GUEST_DR7, 0x400)?;
        
        Ok(())
    }
    
    /// Configurer l'état de l'host (pour le retour après VM exit)
    pub fn setup_host_state(&self, exit_handler: u64, stack: u64) -> Result<()> {
        use fields::*;
        use core::arch::asm;
        
        // Lire les valeurs actuelles de l'host
        let cr0: u64;
        let cr3: u64;
        let cr4: u64;
        
        unsafe {
            asm!("mov {}, cr0", out(reg) cr0);
            asm!("mov {}, cr3", out(reg) cr3);
            asm!("mov {}, cr4", out(reg) cr4);
        }
        
        self.write(HOST_CR0, cr0)?;
        self.write(HOST_CR3, cr3)?;
        self.write(HOST_CR4, cr4)?;
        
        // Segments (CS, SS, DS, ES, FS, GS, TR)
        // Lire les sélecteurs actuels
        let cs: u16;
        let ss: u16;
        let ds: u16;
        let es: u16;
        let fs: u16;
        let gs: u16;
        let tr: u16;
        
        unsafe {
            asm!("mov {:x}, cs", out(reg) cs);
            asm!("mov {:x}, ss", out(reg) ss);
            asm!("mov {:x}, ds", out(reg) ds);
            asm!("mov {:x}, es", out(reg) es);
            asm!("mov {:x}, fs", out(reg) fs);
            asm!("mov {:x}, gs", out(reg) gs);
            asm!("str {:x}", out(reg) tr);
        }
        
        self.write(HOST_CS_SELECTOR, cs as u64)?;
        self.write(HOST_SS_SELECTOR, ss as u64)?;
        self.write(HOST_DS_SELECTOR, ds as u64)?;
        self.write(HOST_ES_SELECTOR, es as u64)?;
        self.write(HOST_FS_SELECTOR, fs as u64)?;
        self.write(HOST_GS_SELECTOR, gs as u64)?;
        self.write(HOST_TR_SELECTOR, tr as u64)?;
        
        // Bases
        self.write(HOST_FS_BASE, 0)?;
        self.write(HOST_GS_BASE, 0)?;
        
        // GDTR et IDTR bases
        let mut gdtr: [u64; 2] = [0; 2];
        let mut idtr: [u64; 2] = [0; 2];
        
        unsafe {
            asm!("sgdt [{}]", in(reg) gdtr.as_mut_ptr());
            asm!("sidt [{}]", in(reg) idtr.as_mut_ptr());
        }
        
        self.write(HOST_GDTR_BASE, gdtr[1])?;
        self.write(HOST_IDTR_BASE, idtr[1])?;
        self.write(HOST_TR_BASE, 0)?; // À configurer correctement
        
        // RIP et RSP pour le handler de sortie
        self.write(HOST_RIP, exit_handler)?;
        self.write(HOST_RSP, stack)?;
        
        Ok(())
    }
}

impl Drop for Vmcs {
    fn drop(&mut self) {
        // VMCLEAR avant de libérer la mémoire
        let _ = vmclear(self.phys_addr);
    }
}
