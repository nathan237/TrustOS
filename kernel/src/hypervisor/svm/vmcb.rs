//! AMD VMCB (Virtual Machine Control Block)
//!
//! The VMCB is the AMD equivalent of Intel's VMCS.
//! It's a 4KB structure split into control area and state save area.

use core::mem::size_of;

/// VMCB Control Area offsets
pub mod control_offsets {
    pub const CR_RD_INTERCEPTS: usize = 0x000;
    pub const CR_WR_INTERCEPTS: usize = 0x004;
    pub const DR_RD_INTERCEPTS: usize = 0x008;
    pub const DR_WR_INTERCEPTS: usize = 0x00C;
    pub const EXCEPTION_INTERCEPTS: usize = 0x010;
    pub const INTERCEPT_MISC1: usize = 0x014;
    pub const INTERCEPT_MISC2: usize = 0x018;
    pub const PAUSE_FILTER_THRESH: usize = 0x03C;
    pub const PAUSE_FILTER_COUNT: usize = 0x03E;
    pub const IOPM_BASE_PA: usize = 0x040;
    pub const MSRPM_BASE_PA: usize = 0x048;
    pub const TSC_OFFSET: usize = 0x050;
    pub const GUEST_ASID: usize = 0x058;
    pub const TLB_CONTROL: usize = 0x05C;
    pub const V_TPR: usize = 0x060;
    pub const V_IRQ: usize = 0x061;
    pub const V_INTR: usize = 0x062;
    pub const V_INTR_MASKING: usize = 0x063;
    pub const V_INTR_VECTOR: usize = 0x064;
    pub const INTERRUPT_SHADOW: usize = 0x068;
    pub const EXITCODE: usize = 0x070;
    pub const EXITINFO1: usize = 0x078;
    pub const EXITINFO2: usize = 0x080;
    pub const EXITINTINFO: usize = 0x088;
    pub const NP_ENABLE: usize = 0x090;
    pub const AVIC_APIC_BAR: usize = 0x098;
    pub const GHCB_PA: usize = 0x0A0;
    pub const EVENT_INJ: usize = 0x0A8;
    pub const N_CR3: usize = 0x0B0;
    pub const LBR_CONTROL: usize = 0x0B8;
    pub const VMCB_CLEAN: usize = 0x0C0;
    pub const NEXT_RIP: usize = 0x0C8;
    pub const BYTES_FETCHED: usize = 0x0D0;
    pub const GUEST_INST_BYTES: usize = 0x0D1;
    pub const AVIC_APIC_BACKING: usize = 0x0E0;
    pub const AVIC_LOGICAL_TABLE: usize = 0x0F0;
    pub const AVIC_PHYSICAL_TABLE: usize = 0x0F8;
    pub const VMSA_PTR: usize = 0x108;
}

/// VMCB State Save Area offsets (starts at 0x400)
pub mod state_offsets {
    pub const BASE: usize = 0x400;
    
    pub const ES_SELECTOR: usize = BASE + 0x000;
    pub const ES_ATTRIB: usize = BASE + 0x002;
    pub const ES_LIMIT: usize = BASE + 0x004;
    pub const ES_BASE: usize = BASE + 0x008;
    
    pub const CS_SELECTOR: usize = BASE + 0x010;
    pub const CS_ATTRIB: usize = BASE + 0x012;
    pub const CS_LIMIT: usize = BASE + 0x014;
    pub const CS_BASE: usize = BASE + 0x018;
    
    pub const SS_SELECTOR: usize = BASE + 0x020;
    pub const SS_ATTRIB: usize = BASE + 0x022;
    pub const SS_LIMIT: usize = BASE + 0x024;
    pub const SS_BASE: usize = BASE + 0x028;
    
    pub const DS_SELECTOR: usize = BASE + 0x030;
    pub const DS_ATTRIB: usize = BASE + 0x032;
    pub const DS_LIMIT: usize = BASE + 0x034;
    pub const DS_BASE: usize = BASE + 0x038;
    
    pub const FS_SELECTOR: usize = BASE + 0x040;
    pub const FS_ATTRIB: usize = BASE + 0x042;
    pub const FS_LIMIT: usize = BASE + 0x044;
    pub const FS_BASE: usize = BASE + 0x048;
    
    pub const GS_SELECTOR: usize = BASE + 0x050;
    pub const GS_ATTRIB: usize = BASE + 0x052;
    pub const GS_LIMIT: usize = BASE + 0x054;
    pub const GS_BASE: usize = BASE + 0x058;
    
    pub const GDTR_SELECTOR: usize = BASE + 0x060;
    pub const GDTR_ATTRIB: usize = BASE + 0x062;
    pub const GDTR_LIMIT: usize = BASE + 0x064;
    pub const GDTR_BASE: usize = BASE + 0x068;
    
    pub const LDTR_SELECTOR: usize = BASE + 0x070;
    pub const LDTR_ATTRIB: usize = BASE + 0x072;
    pub const LDTR_LIMIT: usize = BASE + 0x074;
    pub const LDTR_BASE: usize = BASE + 0x078;
    
    pub const IDTR_SELECTOR: usize = BASE + 0x080;
    pub const IDTR_ATTRIB: usize = BASE + 0x082;
    pub const IDTR_LIMIT: usize = BASE + 0x084;
    pub const IDTR_BASE: usize = BASE + 0x088;
    
    pub const TR_SELECTOR: usize = BASE + 0x090;
    pub const TR_ATTRIB: usize = BASE + 0x092;
    pub const TR_LIMIT: usize = BASE + 0x094;
    pub const TR_BASE: usize = BASE + 0x098;
    
    pub const CPL: usize = BASE + 0x0CB;
    pub const EFER: usize = BASE + 0x0D0;
    pub const CR4: usize = BASE + 0x148;
    pub const CR3: usize = BASE + 0x150;
    pub const CR0: usize = BASE + 0x158;
    pub const DR7: usize = BASE + 0x160;
    pub const DR6: usize = BASE + 0x168;
    pub const RFLAGS: usize = BASE + 0x170;
    pub const RIP: usize = BASE + 0x178;
    pub const RSP: usize = BASE + 0x1D8;
    pub const RAX: usize = BASE + 0x1F8;
    pub const STAR: usize = BASE + 0x200;
    pub const LSTAR: usize = BASE + 0x208;
    pub const CSTAR: usize = BASE + 0x210;
    pub const SFMASK: usize = BASE + 0x218;
    pub const KERNEL_GS_BASE: usize = BASE + 0x220;
    pub const SYSENTER_CS: usize = BASE + 0x228;
    pub const SYSENTER_ESP: usize = BASE + 0x230;
    pub const SYSENTER_EIP: usize = BASE + 0x238;
    pub const CR2: usize = BASE + 0x240;
    pub const PAT: usize = BASE + 0x268;
    pub const DBGCTL: usize = BASE + 0x270;
    pub const BR_FROM: usize = BASE + 0x278;
    pub const BR_TO: usize = BASE + 0x280;
    pub const LASTEXCPFROM: usize = BASE + 0x288;
    pub const LASTEXCPTO: usize = BASE + 0x290;
}

/// Intercept control bits (INTERCEPT_MISC1)
pub mod intercepts {
    pub const INTR: u32 = 1 << 0;
    pub const NMI: u32 = 1 << 1;
    pub const SMI: u32 = 1 << 2;
    pub const INIT: u32 = 1 << 3;
    pub const VINTR: u32 = 1 << 4;
    pub const CR0_SEL_WRITE: u32 = 1 << 5;
    pub const IDTR_READ: u32 = 1 << 6;
    pub const GDTR_READ: u32 = 1 << 7;
    pub const LDTR_READ: u32 = 1 << 8;
    pub const TR_READ: u32 = 1 << 9;
    pub const IDTR_WRITE: u32 = 1 << 10;
    pub const GDTR_WRITE: u32 = 1 << 11;
    pub const LDTR_WRITE: u32 = 1 << 12;
    pub const TR_WRITE: u32 = 1 << 13;
    pub const RDTSC: u32 = 1 << 14;
    pub const RDPMC: u32 = 1 << 15;
    pub const PUSHF: u32 = 1 << 16;
    pub const POPF: u32 = 1 << 17;
    pub const CPUID: u32 = 1 << 18;
    pub const RSM: u32 = 1 << 19;
    pub const IRET: u32 = 1 << 20;
    pub const SWINT: u32 = 1 << 21;
    pub const INVD: u32 = 1 << 22;
    pub const PAUSE: u32 = 1 << 23;
    pub const HLT: u32 = 1 << 24;
    pub const INVLPG: u32 = 1 << 25;
    pub const INVLPGA: u32 = 1 << 26;
    pub const IOIO: u32 = 1 << 27;
    pub const MSR: u32 = 1 << 28;
    pub const TASK_SWITCH: u32 = 1 << 29;
    pub const FERR_FREEZE: u32 = 1 << 30;
    pub const SHUTDOWN: u32 = 1 << 31;
}

/// Intercept control bits (INTERCEPT_MISC2)
pub mod intercepts2 {
    pub const VMRUN: u32 = 1 << 0;
    pub const VMMCALL: u32 = 1 << 1;
    pub const VMLOAD: u32 = 1 << 2;
    pub const VMSAVE: u32 = 1 << 3;
    pub const STGI: u32 = 1 << 4;
    pub const CLGI: u32 = 1 << 5;
    pub const SKINIT: u32 = 1 << 6;
    pub const RDTSCP: u32 = 1 << 7;
    pub const ICEBP: u32 = 1 << 8;
    pub const WBINVD: u32 = 1 << 9;
    pub const MONITOR: u32 = 1 << 10;
    pub const MWAIT: u32 = 1 << 11;
    pub const MWAIT_CONDITIONAL: u32 = 1 << 12;
    pub const XSETBV: u32 = 1 << 13;
}

/// VMCB Clean Bits
pub mod clean_bits {
    pub const INTERCEPTS: u32 = 1 << 0;
    pub const IOPM: u32 = 1 << 1;
    pub const ASID: u32 = 1 << 2;
    pub const TPR: u32 = 1 << 3;
    pub const NP: u32 = 1 << 4;
    pub const CR: u32 = 1 << 5;
    pub const DR: u32 = 1 << 6;
    pub const DT: u32 = 1 << 7;
    pub const SEG: u32 = 1 << 8;
    pub const CR2: u32 = 1 << 9;
    pub const LBR: u32 = 1 << 10;
    pub const AVIC: u32 = 1 << 11;
}

/// TLB Control values
pub mod tlb_control {
    pub const DO_NOTHING: u32 = 0;
    pub const FLUSH_ALL: u32 = 1;
    pub const FLUSH_THIS_ASID: u32 = 3;
    pub const FLUSH_NON_GLOBAL_THIS_ASID: u32 = 7;
}

/// VMCB structure (4KB aligned)
#[repr(C, align(4096))]
pub struct Vmcb {
    data: [u8; 4096],
}

impl Vmcb {
    /// Create a new zeroed VMCB
    pub const fn new() -> Self {
        Self { data: [0; 4096] }
    }
    
    /// Read a single byte from the VMCB
    #[inline]
    pub fn read_u8(&self, offset: usize) -> u8 {
        self.data[offset]
    }
    
    /// Get the guest instruction bytes fetched by the CPU on #NPF/#VMEXIT.
    /// Returns (bytes_fetched_count, instruction_bytes_array).
    /// The VMCB provides up to 15 bytes at GUEST_INST_BYTES (offset 0x0D1),
    /// with the valid count at BYTES_FETCHED (offset 0x0D0).
    pub fn guest_insn_bytes(&self) -> (usize, [u8; 15]) {
        let count = self.data[control_offsets::BYTES_FETCHED] as usize;
        let mut buf = [0u8; 15];
        let n = count.min(15);
        for i in 0..n {
            buf[i] = self.data[control_offsets::GUEST_INST_BYTES + i];
        }
        (n, buf)
    }
    
    /// Read a u16 from the VMCB
    #[inline]
    pub fn read_u16(&self, offset: usize) -> u16 {
        u16::from_le_bytes([self.data[offset], self.data[offset + 1]])
    }
    
    /// Read a u32 from the VMCB
    #[inline]
    pub fn read_u32(&self, offset: usize) -> u32 {
        u32::from_le_bytes([
            self.data[offset],
            self.data[offset + 1],
            self.data[offset + 2],
            self.data[offset + 3],
        ])
    }
    
    /// Read a u64 from the VMCB
    #[inline]
    pub fn read_u64(&self, offset: usize) -> u64 {
        u64::from_le_bytes([
            self.data[offset],
            self.data[offset + 1],
            self.data[offset + 2],
            self.data[offset + 3],
            self.data[offset + 4],
            self.data[offset + 5],
            self.data[offset + 6],
            self.data[offset + 7],
        ])
    }
    
    /// Write a u16 to the VMCB
    #[inline]
    pub fn write_u16(&mut self, offset: usize, value: u16) {
        let bytes = value.to_le_bytes();
        self.data[offset] = bytes[0];
        self.data[offset + 1] = bytes[1];
    }
    
    /// Write a u32 to the VMCB
    #[inline]
    pub fn write_u32(&mut self, offset: usize, value: u32) {
        let bytes = value.to_le_bytes();
        self.data[offset] = bytes[0];
        self.data[offset + 1] = bytes[1];
        self.data[offset + 2] = bytes[2];
        self.data[offset + 3] = bytes[3];
    }
    
    /// Write a u64 to the VMCB
    #[inline]
    pub fn write_u64(&mut self, offset: usize, value: u64) {
        let bytes = value.to_le_bytes();
        for i in 0..8 {
            self.data[offset + i] = bytes[i];
        }
    }
    
    /// Get physical address of VMCB (for VMRUN)
    pub fn phys_addr(&self) -> u64 {
        let virt = self as *const _ as u64;
        virt.wrapping_sub(crate::memory::hhdm_offset())
    }
    
    // ========================================================================
    // Generic Control/State Accessors
    // ========================================================================
    
    /// Read a u64 from the control area by offset
    #[inline]
    pub fn read_control(&self, offset: usize) -> u64 {
        self.read_u64(offset)
    }
    
    /// Write a u64 to the control area by offset
    #[inline]
    pub fn write_control(&mut self, offset: usize, value: u64) {
        self.write_u64(offset, value);
    }
    
    /// Read a u64 from the state save area by offset
    #[inline]
    pub fn read_state(&self, offset: usize) -> u64 {
        self.read_u64(offset)
    }
    
    /// Write a u64 to the state save area by offset
    #[inline]
    pub fn write_state(&mut self, offset: usize, value: u64) {
        self.write_u64(offset, value);
    }
    
    // ========================================================================
    // Control Area Accessors
    // ========================================================================
    
    /// Set CR read intercepts
    pub fn set_cr_read_intercepts(&mut self, mask: u32) {
        self.write_u32(control_offsets::CR_RD_INTERCEPTS, mask);
    }
    
    /// Set CR write intercepts
    pub fn set_cr_write_intercepts(&mut self, mask: u32) {
        self.write_u32(control_offsets::CR_WR_INTERCEPTS, mask);
    }
    
    /// Set exception intercepts (bitmap of vectors 0-31)
    pub fn set_exception_intercepts(&mut self, mask: u32) {
        self.write_u32(control_offsets::EXCEPTION_INTERCEPTS, mask);
    }
    
    /// Set misc intercepts 1
    pub fn set_intercepts1(&mut self, mask: u32) {
        self.write_u32(control_offsets::INTERCEPT_MISC1, mask);
    }
    
    /// Set misc intercepts 2
    pub fn set_intercepts2(&mut self, mask: u32) {
        self.write_u32(control_offsets::INTERCEPT_MISC2, mask);
    }
    
    /// Set IOPM base physical address
    pub fn set_iopm_base(&mut self, phys: u64) {
        self.write_u64(control_offsets::IOPM_BASE_PA, phys);
    }
    
    /// Set MSRPM base physical address
    pub fn set_msrpm_base(&mut self, phys: u64) {
        self.write_u64(control_offsets::MSRPM_BASE_PA, phys);
    }
    
    /// Set TSC offset
    pub fn set_tsc_offset(&mut self, offset: u64) {
        self.write_u64(control_offsets::TSC_OFFSET, offset);
    }
    
    /// Set guest ASID (Address Space ID)
    pub fn set_guest_asid(&mut self, asid: u32) {
        self.write_u32(control_offsets::GUEST_ASID, asid);
    }
    
    /// Set TLB control
    pub fn set_tlb_control(&mut self, control: u32) {
        self.write_u32(control_offsets::TLB_CONTROL, control);
    }
    
    /// Get exit code
    pub fn exit_code(&self) -> u64 {
        self.read_u64(control_offsets::EXITCODE)
    }
    
    /// Get exit info 1
    pub fn exit_info1(&self) -> u64 {
        self.read_u64(control_offsets::EXITINFO1)
    }
    
    /// Get exit info 2
    pub fn exit_info2(&self) -> u64 {
        self.read_u64(control_offsets::EXITINFO2)
    }
    
    /// Enable Nested Paging (NPT)
    pub fn enable_npt(&mut self, ncr3: u64) {
        self.write_u64(control_offsets::NP_ENABLE, 1);
        self.write_u64(control_offsets::N_CR3, ncr3);
    }
    
    /// Disable Nested Paging
    pub fn disable_npt(&mut self) {
        self.write_u64(control_offsets::NP_ENABLE, 0);
    }
    
    /// Get VMCB clean bits
    pub fn clean_bits(&self) -> u32 {
        self.read_u32(control_offsets::VMCB_CLEAN)
    }
    
    /// Set VMCB clean bits
    pub fn set_clean_bits(&mut self, bits: u32) {
        self.write_u32(control_offsets::VMCB_CLEAN, bits);
    }
    
    /// Get next RIP (if NRIP_SAVE feature available)
    pub fn next_rip(&self) -> u64 {
        self.read_u64(control_offsets::NEXT_RIP)
    }
    
    /// Inject an event (interrupt/exception)
    pub fn inject_event(&mut self, vector: u8, event_type: u8, error_code: Option<u32>) {
        let mut event: u64 = (vector as u64) | ((event_type as u64) << 8) | (1u64 << 31); // Valid bit
        if let Some(ec) = error_code {
            event |= 1u64 << 11; // Error code valid
            event |= (ec as u64) << 32;
        }
        self.write_u64(control_offsets::EVENT_INJ, event);
    }
    
    // ========================================================================
    // State Save Area Accessors
    // ========================================================================
    
    /// Set guest RIP
    pub fn set_rip(&mut self, rip: u64) {
        self.write_u64(state_offsets::RIP, rip);
    }
    
    /// Get guest RIP
    pub fn rip(&self) -> u64 {
        self.read_u64(state_offsets::RIP)
    }
    
    /// Set guest RSP
    pub fn set_rsp(&mut self, rsp: u64) {
        self.write_u64(state_offsets::RSP, rsp);
    }
    
    /// Get guest RSP
    pub fn rsp(&self) -> u64 {
        self.read_u64(state_offsets::RSP)
    }
    
    /// Set guest RAX
    pub fn set_rax(&mut self, rax: u64) {
        self.write_u64(state_offsets::RAX, rax);
    }
    
    /// Get guest RAX
    pub fn rax(&self) -> u64 {
        self.read_u64(state_offsets::RAX)
    }
    
    /// Set guest RFLAGS
    pub fn set_rflags(&mut self, rflags: u64) {
        self.write_u64(state_offsets::RFLAGS, rflags);
    }
    
    /// Get guest RFLAGS
    pub fn rflags(&self) -> u64 {
        self.read_u64(state_offsets::RFLAGS)
    }
    
    /// Set guest CR0
    pub fn set_cr0(&mut self, cr0: u64) {
        self.write_u64(state_offsets::CR0, cr0);
    }
    
    /// Get guest CR0
    pub fn cr0(&self) -> u64 {
        self.read_u64(state_offsets::CR0)
    }
    
    /// Set guest CR3
    pub fn set_cr3(&mut self, cr3: u64) {
        self.write_u64(state_offsets::CR3, cr3);
    }
    
    /// Get guest CR3
    pub fn cr3(&self) -> u64 {
        self.read_u64(state_offsets::CR3)
    }
    
    /// Set guest CR4
    pub fn set_cr4(&mut self, cr4: u64) {
        self.write_u64(state_offsets::CR4, cr4);
    }
    
    /// Get guest CR4
    pub fn cr4(&self) -> u64 {
        self.read_u64(state_offsets::CR4)
    }
    
    /// Set guest EFER
    pub fn set_efer(&mut self, efer: u64) {
        self.write_u64(state_offsets::EFER, efer);
    }
    
    /// Get guest EFER
    pub fn efer(&self) -> u64 {
        self.read_u64(state_offsets::EFER)
    }
    
    /// Set CS segment
    pub fn set_cs(&mut self, selector: u16, attrib: u16, limit: u32, base: u64) {
        self.write_u16(state_offsets::CS_SELECTOR, selector);
        self.write_u16(state_offsets::CS_ATTRIB, attrib);
        self.write_u32(state_offsets::CS_LIMIT, limit);
        self.write_u64(state_offsets::CS_BASE, base);
    }
    
    /// Set DS segment
    pub fn set_ds(&mut self, selector: u16, attrib: u16, limit: u32, base: u64) {
        self.write_u16(state_offsets::DS_SELECTOR, selector);
        self.write_u16(state_offsets::DS_ATTRIB, attrib);
        self.write_u32(state_offsets::DS_LIMIT, limit);
        self.write_u64(state_offsets::DS_BASE, base);
    }
    
    /// Set ES segment
    pub fn set_es(&mut self, selector: u16, attrib: u16, limit: u32, base: u64) {
        self.write_u16(state_offsets::ES_SELECTOR, selector);
        self.write_u16(state_offsets::ES_ATTRIB, attrib);
        self.write_u32(state_offsets::ES_LIMIT, limit);
        self.write_u64(state_offsets::ES_BASE, base);
    }
    
    /// Set SS segment
    pub fn set_ss(&mut self, selector: u16, attrib: u16, limit: u32, base: u64) {
        self.write_u16(state_offsets::SS_SELECTOR, selector);
        self.write_u16(state_offsets::SS_ATTRIB, attrib);
        self.write_u32(state_offsets::SS_LIMIT, limit);
        self.write_u64(state_offsets::SS_BASE, base);
    }
    
    /// Set FS segment
    pub fn set_fs(&mut self, selector: u16, attrib: u16, limit: u32, base: u64) {
        self.write_u16(state_offsets::FS_SELECTOR, selector);
        self.write_u16(state_offsets::FS_ATTRIB, attrib);
        self.write_u32(state_offsets::FS_LIMIT, limit);
        self.write_u64(state_offsets::FS_BASE, base);
    }
    
    /// Set GS segment
    pub fn set_gs(&mut self, selector: u16, attrib: u16, limit: u32, base: u64) {
        self.write_u16(state_offsets::GS_SELECTOR, selector);
        self.write_u16(state_offsets::GS_ATTRIB, attrib);
        self.write_u32(state_offsets::GS_LIMIT, limit);
        self.write_u64(state_offsets::GS_BASE, base);
    }
    
    /// Set GDTR
    pub fn set_gdtr(&mut self, limit: u32, base: u64) {
        self.write_u32(state_offsets::GDTR_LIMIT, limit);
        self.write_u64(state_offsets::GDTR_BASE, base);
    }
    
    /// Set IDTR
    pub fn set_idtr(&mut self, limit: u32, base: u64) {
        self.write_u32(state_offsets::IDTR_LIMIT, limit);
        self.write_u64(state_offsets::IDTR_BASE, base);
    }
    
    /// Set TR segment
    pub fn set_tr(&mut self, selector: u16, attrib: u16, limit: u32, base: u64) {
        self.write_u16(state_offsets::TR_SELECTOR, selector);
        self.write_u16(state_offsets::TR_ATTRIB, attrib);
        self.write_u32(state_offsets::TR_LIMIT, limit);
        self.write_u64(state_offsets::TR_BASE, base);
    }
    
    /// Set LDTR
    pub fn set_ldtr(&mut self, selector: u16, attrib: u16, limit: u32, base: u64) {
        self.write_u16(state_offsets::LDTR_SELECTOR, selector);
        self.write_u16(state_offsets::LDTR_ATTRIB, attrib);
        self.write_u32(state_offsets::LDTR_LIMIT, limit);
        self.write_u64(state_offsets::LDTR_BASE, base);
    }
    
    /// Set CPL (Current Privilege Level)
    pub fn set_cpl(&mut self, cpl: u8) {
        self.data[state_offsets::CPL] = cpl;
    }
    
    /// Get CPL
    pub fn cpl(&self) -> u8 {
        self.data[state_offsets::CPL]
    }
    
    /// Set DR6
    pub fn set_dr6(&mut self, dr6: u64) {
        self.write_u64(state_offsets::DR6, dr6);
    }
    
    /// Set DR7
    pub fn set_dr7(&mut self, dr7: u64) {
        self.write_u64(state_offsets::DR7, dr7);
    }
    
    /// Set PAT MSR
    pub fn set_pat(&mut self, pat: u64) {
        self.write_u64(state_offsets::PAT, pat);
    }
    
    // ========================================================================
    // Helper Methods
    // ========================================================================
    
    /// Configure for real mode guest
    pub fn setup_real_mode(&mut self) {
        // Real mode segment attributes: present, read/write, accessed
        let data_attrib: u16 = 0x0093; // Present + S + Type(Data RW Accessed)
        let code_attrib: u16 = 0x009B; // Present + S + Type(Code RX Accessed)
        
        // CS at 0xF000 (reset vector area)
        self.set_cs(0xF000, code_attrib, 0xFFFF, 0xF0000);
        
        // Data segments at 0
        self.set_ds(0, data_attrib, 0xFFFF, 0);
        self.set_es(0, data_attrib, 0xFFFF, 0);
        self.set_fs(0, data_attrib, 0xFFFF, 0);
        self.set_gs(0, data_attrib, 0xFFFF, 0);
        self.set_ss(0, data_attrib, 0xFFFF, 0);
        
        // TR (required even in real mode)
        self.set_tr(0, 0x008B, 0xFFFF, 0);
        
        // LDTR
        self.set_ldtr(0, 0x0082, 0xFFFF, 0);
        
        // GDTR/IDTR
        self.set_gdtr(0xFFFF, 0);
        self.set_idtr(0x3FF, 0);  // Real mode IVT
        
        // CR0: Not in protected mode, but cache enabled
        self.set_cr0(0x60000010);  // CD=0, NW=0, ET=1
        
        // RFLAGS: Interrupts disabled, reserved bit set
        self.set_rflags(0x00000002);
        
        // RIP: Reset vector offset
        self.set_rip(0xFFF0);
        
        // DR6/DR7 defaults
        self.set_dr6(0xFFFF0FF0);
        self.set_dr7(0x00000400);
        
        // EFER: Long mode disabled
        self.set_efer(0);
        
        // PAT: Default value
        self.set_pat(0x0007040600070406);
    }
    
    /// Configure for protected mode guest (32-bit)
    pub fn setup_protected_mode(&mut self, entry_point: u64) {
        let data_attrib: u16 = 0x00C3; // G + DB + Present + Data RW
        let code_attrib: u16 = 0x00CB; // G + DB + Present + Code RX
        
        self.set_cs(0x08, code_attrib, 0xFFFFFFFF, 0);
        self.set_ds(0x10, data_attrib, 0xFFFFFFFF, 0);
        self.set_es(0x10, data_attrib, 0xFFFFFFFF, 0);
        self.set_fs(0x10, data_attrib, 0xFFFFFFFF, 0);
        self.set_gs(0x10, data_attrib, 0xFFFFFFFF, 0);
        self.set_ss(0x10, data_attrib, 0xFFFFFFFF, 0);
        
        self.set_tr(0x18, 0x008B, 0xFFFF, 0);
        self.set_ldtr(0, 0x0082, 0xFFFF, 0);
        
        // GDTR: point to GDT at GPA 0x1000 (3 entries × 8 bytes = 24 bytes)
        self.set_gdtr(23, 0x1000);
        // IDTR: no IDT initially (Linux will set up its own)
        self.set_idtr(0, 0);
        
        self.set_cr0(0x60000011);  // PE=1 (Protected mode)
        self.set_rflags(0x00000002);
        self.set_rip(entry_point);
        self.set_efer(0);
    }
    
    /// Configure for long mode guest (64-bit)
    /// 
    /// AMD VMCB segment attribute format (16-bit):
    ///   [3:0]  = Type (from GDT descriptor type field)
    ///   [4]    = S (1=code/data, 0=system)
    ///   [6:5]  = DPL
    ///   [7]    = P (Present)
    ///   [8]    = AVL
    ///   [9]    = L (Long mode, 64-bit code)
    ///   [10]   = D/B (Default operand size)
    ///   [11]   = G (Granularity)
    pub fn setup_long_mode(&mut self, entry_point: u64, guest_cr3: u64) {
        // CS: 64-bit code segment — type=0xB (code, exec/read, accessed), S=1, P=1, L=1, G=1
        // attrib = 0x0A9B: G=1(bit11), L=1(bit9), P=1(bit7), S=1(bit4), type=0xB
        let code64_attrib: u16 = 0x0A9B;
        // Data segments: type=0x3 (data, read/write, accessed), S=1, P=1, G=1
        let data64_attrib: u16 = 0x0C93;
        
        self.set_cs(0x08, code64_attrib, 0xFFFFFFFF, 0);
        self.set_ds(0x10, data64_attrib, 0xFFFFFFFF, 0);
        self.set_es(0x10, data64_attrib, 0xFFFFFFFF, 0);
        self.set_fs(0x10, data64_attrib, 0xFFFFFFFF, 0);
        self.set_gs(0x10, data64_attrib, 0xFFFFFFFF, 0);
        self.set_ss(0x10, data64_attrib, 0xFFFFFFFF, 0);
        
        // TR: busy 64-bit TSS (type=0xB), P=1
        self.set_tr(0, 0x008B, 0xFFFF, 0);
        // LDTR: LDT descriptor (type=0x2), P=1
        self.set_ldtr(0, 0x0082, 0xFFFF, 0);
        
        // GDTR/IDTR
        self.set_gdtr(23, 0x1000);
        self.set_idtr(0, 0);
        
        // CR0: PE + ET + PG (protected mode + paging)
        self.set_cr0(0x8001_0031); // PG + WP + NE + ET + PE
        
        // CR3: Guest page tables (NOT the NPT CR3!)
        self.set_cr3(guest_cr3);
        
        // CR4: PAE + PGE + OSFXSR + OSXMMEXCPT
        self.set_cr4(0x00000620); // PAE=0x20 + PGE=0x80 + OSFXSR=0x200 + OSXMMEXCPT=0x400
        
        // EFER: SCE + LME + LMA + SVME (SVME required for guest EFER on AMD)
        self.set_efer(0x00001501); // SVME(12) + LMA(10) + LME(8) + SCE(0)
        
        self.set_rflags(0x00000002);
        self.set_rip(entry_point);
        
        // CPL 0 (ring 0)
        self.set_cpl(0);
        
        self.set_dr6(0xFFFF0FF0);
        self.set_dr7(0x00000400);
        self.set_pat(0x0007040600070406);
    }
    
    /// Configure for Linux kernel boot in 64-bit long mode
    /// Uses the Linux boot protocol layout from linux_loader
    pub fn setup_long_mode_for_linux(
        &mut self,
        entry_point: u64,
        stack_ptr: u64,
        guest_cr3: u64,
        gdt_base: u64,
        gdt_limit: u32,
    ) {
        // CS: 64-bit code, selector 0x08
        let code64_attrib: u16 = 0x0A9B;
        // Data: 64-bit data, selector 0x10
        let data64_attrib: u16 = 0x0C93;
        
        self.set_cs(0x08, code64_attrib, 0xFFFFFFFF, 0);
        self.set_ds(0x10, data64_attrib, 0xFFFFFFFF, 0);
        self.set_es(0x10, data64_attrib, 0xFFFFFFFF, 0);
        self.set_fs(0x10, data64_attrib, 0xFFFFFFFF, 0);
        self.set_gs(0x10, data64_attrib, 0xFFFFFFFF, 0);
        self.set_ss(0x10, data64_attrib, 0xFFFFFFFF, 0);
        
        // TR: busy 64-bit TSS
        self.set_tr(0, 0x008B, 0x67, 0);
        // LDTR: unusable
        self.set_ldtr(0, 0x0082, 0, 0);
        
        // Use the Linux guest's own GDT
        self.set_gdtr(gdt_limit, gdt_base);
        self.set_idtr(0, 0);
        
        // CR0: PG + WP + NE + ET + PE
        self.set_cr0(0x8001_0033);
        
        // CR3: Guest page tables
        self.set_cr3(guest_cr3);
        
        // CR4: PAE + PGE + OSFXSR + OSXMMEXCPT
        self.set_cr4(0x00000620);
        
        // EFER: SCE + LME + LMA + SVME
        self.set_efer(0x00001501);
        
        self.set_rflags(0x00000002);
        self.set_rip(entry_point);
        self.set_rsp(stack_ptr);
        self.set_cpl(0);
        
        self.set_dr6(0xFFFF0FF0);
        self.set_dr7(0x00000400);
        self.set_pat(0x0007040600070406);
    }
    
    /// Set up basic intercepts for hypercall-based VM
    pub fn setup_basic_intercepts(&mut self) {
        // Intercept: CPUID, HLT, I/O, MSR, shutdown, INVD
        let intercepts1 = intercepts::CPUID 
            | intercepts::HLT 
            | intercepts::IOIO
            | intercepts::MSR 
            | intercepts::SHUTDOWN
            | intercepts::INVD;  // Cache invalidation — need to emulate
        
        let intercepts2 = intercepts2::VMMCALL 
            | intercepts2::VMRUN
            | intercepts2::XSETBV    // Extended state set — must intercept
            | intercepts2::WBINVD    // Write-back invalidate
            | intercepts2::MONITOR   // MONITOR/MWAIT
            | intercepts2::MWAIT;
        
        self.set_intercepts1(intercepts1);
        self.set_intercepts2(intercepts2);
        
        // Intercept writes to CR0, CR3, CR4
        self.set_cr_write_intercepts(0x19);  // CR0, CR3, CR4
    }
}

/// Segment attribute helpers
pub mod seg_attrib {
    /// Create segment attributes from access rights
    pub fn from_access_rights(access: u16, granularity: bool, db: bool, long: bool) -> u16 {
        let mut attrib = access & 0xFF;
        if granularity { attrib |= 1 << 8; }
        if db { attrib |= 1 << 10; }
        if long { attrib |= 1 << 9; }
        attrib
    }
    
    pub const PRESENT: u16 = 1 << 7;
    pub const DPL_0: u16 = 0 << 5;
    pub const DPL_3: u16 = 3 << 5;
    pub const S_CODE_DATA: u16 = 1 << 4;
    pub const TYPE_CODE_RX: u16 = 0x0A;
    pub const TYPE_DATA_RW: u16 = 0x02;
    pub const GRANULARITY: u16 = 1 << 8;
    pub const DB: u16 = 1 << 10;
    pub const LONG: u16 = 1 << 9;
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_vmcb_size() {
        assert_eq!(size_of::<Vmcb>(), 4096);
    }
}
