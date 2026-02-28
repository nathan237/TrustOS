//! EL2 Trap Handler — Synchronous Exception Decoding
//!
//! When the guest at EL1 triggers a trapped operation, the CPU takes a
//! synchronous exception to EL2. ESR_EL2 tells us what happened:
//!
//! ```text
//! ESR_EL2 bits:
//!   [31:26] EC  = Exception Class (what kind of trap)
//!   [25]    IL  = Instruction Length (0=16bit, 1=32bit)
//!   [24:0]  ISS = Instruction Specific Syndrome (details)
//! ```
//!
//! For Data Aborts (MMIO spy):
//! ```text
//!   ISS bits:
//!     [24]    ISV  = Instruction Syndrome Valid
//!     [23:22] SAS  = Syndrome Access Size (0=byte,1=hw,2=word,3=dword)
//!     [21]    SSE  = Syndrome Sign Extend
//!     [20:16] SRT  = Syndrome Register Transfer (which Xn register)
//!     [15]    SF   = Sixty-Four bit register (1=X, 0=W)
//!     [6]     WnR  = Write not Read (1=write, 0=read)
//! ```

use super::mmio_spy;

/// ESR_EL2 field extraction
pub mod esr {
    /// Extract Exception Class
    #[inline(always)]
    pub fn ec(esr: u64) -> u32 {
        ((esr >> 26) & 0x3F) as u32
    }

    /// Extract Instruction Length
    #[inline(always)]
    pub fn il(esr: u64) -> bool {
        (esr >> 25) & 1 != 0
    }

    /// Extract ISS (Instruction Specific Syndrome)
    #[inline(always)]
    pub fn iss(esr: u64) -> u32 {
        (esr & 0x01FF_FFFF) as u32
    }
}

/// Data Abort ISS field extraction
pub mod dabt {
    /// ISV: Instruction Syndrome Valid
    #[inline(always)]
    pub fn isv(iss: u32) -> bool {
        (iss >> 24) & 1 != 0
    }

    /// SAS: Syndrome Access Size (0=byte, 1=halfword, 2=word, 3=doubleword)
    #[inline(always)]
    pub fn sas(iss: u32) -> u32 {
        (iss >> 22) & 0x3
    }

    /// Access size in bytes
    #[inline(always)]
    pub fn access_size(iss: u32) -> u32 {
        1 << sas(iss)
    }

    /// SRT: Syndrome Register Transfer (which Xn)
    #[inline(always)]
    pub fn srt(iss: u32) -> u32 {
        (iss >> 16) & 0x1F
    }

    /// SF: Sixty-Four bit register
    #[inline(always)]
    pub fn sf(iss: u32) -> bool {
        (iss >> 15) & 1 != 0
    }

    /// WnR: Write not Read
    #[inline(always)]
    pub fn is_write(iss: u32) -> bool {
        (iss >> 6) & 1 != 0
    }
}

/// SMC/HVC argument extraction from guest context
pub mod smc {
    /// SMCCC Function ID classification
    #[derive(Debug, Clone, Copy)]
    pub enum SmcType {
        /// PSCI call (power management)
        Psci,
        /// Secure service
        SecureService,
        /// Platform-specific (OEM)
        OemService,
        /// Standard Hypervisor
        HypService,
        /// Unknown
        Unknown,
    }

    /// Classify an SMC Function ID
    pub fn classify(fid: u64) -> SmcType {
        let owning_entity = (fid >> 24) & 0x3F;
        match owning_entity {
            0x04 => SmcType::Psci,          // Standard Secure - PSCI
            0x00..=0x01 => SmcType::SecureService,
            0x02..=0x03 => SmcType::SecureService,
            0x30..=0x31 => SmcType::OemService,
            0x05 => SmcType::HypService,
            _ => SmcType::Unknown,
        }
    }

    /// Decode PSCI function name
    pub fn psci_name(fid: u64) -> &'static str {
        match fid & 0xFFFF_FFFF {
            0x8400_0000 => "PSCI_VERSION",
            0x8400_0001 => "CPU_SUSPEND (32)",
            0xC400_0001 => "CPU_SUSPEND (64)",
            0x8400_0002 => "CPU_OFF",
            0x8400_0003 => "CPU_ON (32)",
            0xC400_0003 => "CPU_ON (64)",
            0x8400_0004 => "AFFINITY_INFO (32)",
            0xC400_0004 => "AFFINITY_INFO (64)",
            0x8400_0005 => "MIGRATE (32)",
            0x8400_0008 => "SYSTEM_OFF",
            0x8400_0009 => "SYSTEM_RESET",
            0x8400_000A => "FEATURES",
            0x8400_000C => "SYSTEM_RESET2",
            _ => "UNKNOWN_PSCI",
        }
    }
}

/// Exception Classes
mod ec {
    pub const DATA_ABORT_LOWER: u32  = 0b10_0100;  // 0x24
    pub const INST_ABORT_LOWER: u32  = 0b10_0000;  // 0x20
    pub const HVC64: u32             = 0b01_0110;   // 0x16
    pub const SMC64: u32             = 0b01_0111;   // 0x17
    pub const MSR_MRS: u32           = 0b01_1000;   // 0x18
    pub const WFX: u32              = 0b00_0001;   // 0x01
    pub const SVC64: u32             = 0b01_0101;   // 0x15
    pub const FP_EXC: u32            = 0b10_1100;   // 0x2C
}

/// Result of handling a trap
#[derive(Debug)]
pub enum TrapAction {
    /// Trap handled, advance guest PC and resume
    Handled,
    /// Trap is an SMC that should be forwarded to real firmware
    ForwardSmc,
    /// Trap cannot be handled — inject fault to guest
    InjectFault,
    /// Guest requested poweroff
    GuestHalt,
}

/// Main trap handler — called from EL2 exception vector
///
/// # Arguments
/// * `esr` — ESR_EL2 value (exception syndrome)
/// * `far` — FAR_EL2 value (faulting virtual address)
/// * `hpfar` — HPFAR_EL2 value (faulting IPA, bits [39:12] in [35:4])
/// * `guest_regs` — pointer to saved guest registers x0-x30
///
/// # Returns
/// `TrapAction` telling the assembly stub what to do next
pub fn handle_sync_trap(
    esr: u64,
    far: u64,
    hpfar: u64,
    guest_regs: &mut [u64; 31],
) -> TrapAction {
    let exception_class = esr::ec(esr);
    let iss = esr::iss(esr);

    match exception_class {
        ec::DATA_ABORT_LOWER => {
            handle_data_abort(iss, far, hpfar, guest_regs)
        }
        ec::INST_ABORT_LOWER => {
            // Instruction fetch from trapped MMIO is unusual — log and fault
            let ipa = (hpfar & 0x0000_000F_FFFF_FFF0) << 8;
            mmio_spy::log_event(mmio_spy::MmioEvent {
                ipa,
                va: far,
                value: 0,
                access_size: 4,
                is_write: false,
                was_inst_fetch: true,
                device_name: mmio_spy::identify_device(ipa),
            });
            TrapAction::InjectFault
        }
        ec::SMC64 => {
            handle_smc(guest_regs)
        }
        ec::HVC64 => {
            // HVC from guest — this is a direct hypervisor call
            // We can use this for TrustOS guest services
            let hvc_id = guest_regs[0];
            handle_hypercall(hvc_id, guest_regs)
        }
        ec::MSR_MRS => {
            // System register access trap (TVM in HCR_EL2)
            handle_sysreg_trap(iss, guest_regs)
        }
        ec::WFX => {
            // WFI/WFE trapped — just skip it
            TrapAction::Handled
        }
        _ => {
            // Unknown exception class
            TrapAction::InjectFault
        }
    }
}

/// Handle a Data Abort from lower EL (the MMIO spy's main path)
fn handle_data_abort(
    iss: u32,
    far: u64,
    hpfar: u64,
    guest_regs: &mut [u64; 31],
) -> TrapAction {
    // Calculate the faulting IPA
    // HPFAR_EL2[39:4] contains IPA[47:12]
    let ipa_page = (hpfar & 0x0000_000F_FFFF_FFF0) << 8;
    let ipa = ipa_page | (far & 0xFFF);

    if !dabt::isv(iss) {
        // ISS not valid - can't decode the access
        // This happens with certain load/store instructions
        // Log it and try to skip
        mmio_spy::log_event(mmio_spy::MmioEvent {
            ipa,
            va: far,
            value: 0,
            access_size: 0,
            is_write: false,
            was_inst_fetch: false,
            device_name: mmio_spy::identify_device(ipa),
        });
        return TrapAction::Handled;
    }

    let is_write = dabt::is_write(iss);
    let access_size = dabt::access_size(iss);
    let reg_idx = dabt::srt(iss) as usize;

    if is_write {
        // Guest is writing to MMIO — get the value from guest register
        let value = if reg_idx < 31 { guest_regs[reg_idx] } else { 0 };

        // Log the write
        mmio_spy::log_event(mmio_spy::MmioEvent {
            ipa,
            va: far,
            value,
            access_size,
            is_write: true,
            was_inst_fetch: false,
            device_name: mmio_spy::identify_device(ipa),
        });

        // Perform the REAL write
        do_mmio_write(ipa, value, access_size);
    } else {
        // Guest is reading from MMIO — do the real read
        let value = do_mmio_read(ipa, access_size);

        // Log the read
        mmio_spy::log_event(mmio_spy::MmioEvent {
            ipa,
            va: far,
            value,
            access_size,
            is_write: false,
            was_inst_fetch: false,
            device_name: mmio_spy::identify_device(ipa),
        });

        // Write the value into guest's register
        if reg_idx < 31 {
            guest_regs[reg_idx] = value;
        }
    }

    TrapAction::Handled
}

/// Handle an SMC from the guest (SMC64)
fn handle_smc(guest_regs: &mut [u64; 31]) -> TrapAction {
    let fid = guest_regs[0];
    let x1 = guest_regs[1];
    let x2 = guest_regs[2];
    let x3 = guest_regs[3];

    let smc_type = smc::classify(fid);

    // Log this SMC
    mmio_spy::log_smc(mmio_spy::SmcEvent {
        fid,
        x1,
        x2,
        x3,
        smc_type_name: match smc_type {
            smc::SmcType::Psci => smc::psci_name(fid),
            smc::SmcType::SecureService => "SECURE_SVC",
            smc::SmcType::OemService => "OEM_SVC",
            smc::SmcType::HypService => "HYP_SVC",
            smc::SmcType::Unknown => "UNKNOWN",
        },
    });

    // Check if guest wants to halt
    if let smc::SmcType::Psci = smc_type {
        match fid & 0xFFFF_FFFF {
            0x8400_0008 => return TrapAction::GuestHalt,  // SYSTEM_OFF
            0x8400_0009 => return TrapAction::GuestHalt,  // SYSTEM_RESET
            _ => {}
        }
    }

    // Forward SMC to real firmware
    TrapAction::ForwardSmc
}

/// Handle a hypercall from the guest (HVC #imm)
fn handle_hypercall(hvc_id: u64, guest_regs: &mut [u64; 31]) -> TrapAction {
    match hvc_id {
        // TrustOS spy report request
        0x5452_5553 => {
            // "TRUS" — guest is asking TrustOS to report spy data
            guest_regs[0] = mmio_spy::total_mmio_events() as u64;
            guest_regs[1] = mmio_spy::total_smc_events() as u64;
            TrapAction::Handled
        }
        _ => TrapAction::Handled,
    }
}

/// Handle a system register access trap
fn handle_sysreg_trap(_iss: u32, _guest_regs: &mut [u64; 31]) -> TrapAction {
    // When TVM is set in HCR_EL2, writes to certain EL1 registers trap
    // For now, just skip them — can be extended to track page table changes
    TrapAction::Handled
}

/// Perform a real MMIO read at a physical address
fn do_mmio_read(pa: u64, size: u32) -> u64 {
    #[cfg(target_arch = "aarch64")]
    unsafe {
        let val: u64;
        match size {
            1 => {
                let v: u8;
                core::arch::asm!(
                    "ldrb {val:w}, [{addr}]",
                    addr = in(reg) pa,
                    val = out(reg) v,
                    options(nostack, readonly)
                );
                return v as u64;
            }
            2 => {
                let v: u16;
                core::arch::asm!(
                    "ldrh {val:w}, [{addr}]",
                    addr = in(reg) pa,
                    val = out(reg) v,
                    options(nostack, readonly)
                );
                return v as u64;
            }
            4 => {
                let v: u32;
                core::arch::asm!(
                    "ldr {val:w}, [{addr}]",
                    addr = in(reg) pa,
                    val = out(reg) v,
                    options(nostack, readonly)
                );
                return v as u64;
            }
            8 => {
                core::arch::asm!(
                    "ldr {val}, [{addr}]",
                    addr = in(reg) pa,
                    val = out(reg) val,
                    options(nostack, readonly)
                );
                return val;
            }
            _ => return 0,
        }
    }
    #[cfg(not(target_arch = "aarch64"))]
    {
        let _ = (pa, size);
        0
    }
}

/// Perform a real MMIO write at a physical address
fn do_mmio_write(pa: u64, value: u64, size: u32) {
    #[cfg(target_arch = "aarch64")]
    unsafe {
        match size {
            1 => {
                core::arch::asm!(
                    "strb {val:w}, [{addr}]",
                    addr = in(reg) pa,
                    val = in(reg) value as u32,
                    options(nostack)
                );
            }
            2 => {
                core::arch::asm!(
                    "strh {val:w}, [{addr}]",
                    addr = in(reg) pa,
                    val = in(reg) value as u32,
                    options(nostack)
                );
            }
            4 => {
                core::arch::asm!(
                    "str {val:w}, [{addr}]",
                    addr = in(reg) pa,
                    val = in(reg) value as u32,
                    options(nostack)
                );
            }
            8 => {
                core::arch::asm!(
                    "str {val}, [{addr}]",
                    addr = in(reg) pa,
                    val = in(reg) value,
                    options(nostack)
                );
            }
            _ => {}
        }
    }
    #[cfg(not(target_arch = "aarch64"))]
    {
        let _ = (pa, value, size);
    }
}
