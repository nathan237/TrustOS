//! ARM EL2 Hypervisor — TrustOS MMIO Spy
//!
//! This module implements a Type-1 hypervisor that runs at EL2 (Hypervisor
//! Exception Level) on ARMv8-A. It intercepts all hardware access from a
//! guest OS (Android, Linux) running at EL1, while letting it function
//! normally.
//!
//! Architecture:
//! ```text
//!   EL3  ━━  BL31 / ARM Trusted Firmware (untouched)
//!   EL2  ━━  TrustOS Hypervisor ← WE ARE HERE
//!              ├── Stage-2 Page Tables (IPA → PA translation)
//!              ├── HCR_EL2 trap routing
//!              ├── MMIO Spy (logs every device access)
//!              └── vGIC (virtual interrupt forwarding)
//!   EL1  ━━  Guest OS (Android/Linux) — thinks it's running alone
//!   EL0  ━━  Guest apps
//! ```
//!
//! The key insight: ARM Stage-2 translation lets us mark any memory region
//! as "trapped". When the guest accesses it, CPU generates a Synchronous
//! Exception to EL2. We decode the access, log it, perform the real
//! hardware I/O, and return to the guest. The guest never knows.

pub mod stage2;
pub mod trap_handler;
pub mod mmio_spy;
pub mod vgic;
pub mod el2_entry;
pub mod guest_loader;

use alloc::string::String;
use alloc::vec::Vec;
use alloc::format;
use core::sync::atomic::{AtomicBool, AtomicU64, Ordering};

/// Whether EL2 hypervisor mode is active
static HV_ACTIVE: AtomicBool = AtomicBool::new(false);

/// Number of trapped MMIO accesses since boot
static MMIO_TRAP_COUNT: AtomicU64 = AtomicU64::new(0);

/// Number of SMC calls intercepted
static SMC_TRAP_COUNT: AtomicU64 = AtomicU64::new(0);

// ═════════════════════════════════════════════════════════════════════════════
// HCR_EL2 — Hypervisor Configuration Register
// This is the master control for what gets trapped to EL2
// ═════════════════════════════════════════════════════════════════════════════

/// HCR_EL2 bit definitions
pub mod hcr {
    /// VM bit — enable Stage-2 translation for EL1&0
    pub const VM: u64      = 1 << 0;
    /// SWIO — set/way cache operations trapped
    pub const SWIO: u64    = 1 << 1;
    /// FMO — route physical FIQ to EL2
    pub const FMO: u64     = 1 << 3;
    /// IMO — route physical IRQ to EL2
    pub const IMO: u64     = 1 << 4;
    /// AMO — route SError to EL2
    pub const AMO: u64     = 1 << 5;
    /// TWI — trap WFI to EL2
    pub const TWI: u64     = 1 << 13;
    /// TWE — trap WFE to EL2
    pub const TWE: u64     = 1 << 14;
    /// TVM — trap virtual memory controls (SCTLR, TTBR, etc.)
    pub const TVM: u64     = 1 << 26;
    /// TSC — trap SMC to EL2 (so we see secure world calls!)
    pub const TSC: u64     = 1 << 19;
    /// RW — EL1 is AArch64
    pub const RW: u64      = 1 << 31;
    /// APK — trap pointer authentication key access
    pub const APK: u64     = 1 << 40;
    /// API — trap pointer authentication instructions
    pub const API: u64     = 1 << 41;
}

/// ESR_EL2 exception class values  
pub mod esr_class {
    /// Data Abort from lower EL (Stage-2 fault = MMIO trap)
    pub const DATA_ABORT_LOWER: u32 = 0b100100;
    /// HVC from EL1
    pub const HVC64: u32 = 0b010110;
    /// SMC from EL1 (trapped by TSC)
    pub const SMC64: u32 = 0b010111;
    /// System register access (trapped by TVM, etc.)
    pub const MSR_MRS: u32 = 0b011000;
    /// WFI/WFE trapped (TWI/TWE)
    pub const WFX: u32 = 0b000001;
    /// Instruction Abort from lower EL
    pub const INST_ABORT_LOWER: u32 = 0b100000;
}

/// Guest CPU state saved/restored on EL2 entry/exit
#[repr(C)]
#[derive(Debug, Clone, Default)]
pub struct GuestContext {
    /// General-purpose registers x0-x30
    pub x: [u64; 31],
    /// Stack pointer (SP_EL1)
    pub sp_el1: u64,
    /// Exception Link Register (return address)
    pub elr_el1: u64,
    /// Saved Program Status Register
    pub spsr_el1: u64,
    /// System Control Register
    pub sctlr_el1: u64,
    /// Translation Table Base Register 0
    pub ttbr0_el1: u64,
    /// Translation Table Base Register 1
    pub ttbr1_el1: u64,
    /// Translation Control Register
    pub tcr_el1: u64,
    /// Memory Attribute Indirection Register
    pub mair_el1: u64,
    /// Vector Base Address Register
    pub vbar_el1: u64,
    /// Exception Syndrome Register (why we trapped)
    pub esr_el2: u64,
    /// Faulting Address Register (which IPA was accessed)
    pub far_el2: u64,
    /// Hypervisor IPA Fault Address Register (Stage-2 fault IPA)
    pub hpfar_el2: u64,
}

/// Hypervisor configuration for guest boot
#[derive(Debug, Clone)]
pub struct HypervisorConfig {
    /// Guest kernel entry point (physical address)
    pub guest_entry: u64,
    /// Guest DTB address (we can modify it!)
    pub guest_dtb: u64,
    /// RAM base for the guest
    pub guest_ram_base: u64,
    /// RAM size for the guest
    pub guest_ram_size: u64,
    /// MMIO regions to trap (base, size) — these get logged
    pub trapped_mmio: Vec<(u64, u64)>,
    /// Whether to trap SMC calls (see TrustZone traffic)
    pub trap_smc: bool,
    /// Whether to trap WFI (see idle patterns)
    pub trap_wfi: bool,
}

impl Default for HypervisorConfig {
    fn default() -> Self {
        Self {
            guest_entry: 0,
            guest_dtb: 0,
            guest_ram_base: 0x4000_0000,
            guest_ram_size: 512 * 1024 * 1024,
            trapped_mmio: Vec::new(),
            trap_smc: true,
            trap_wfi: false,
        }
    }
}

// ═════════════════════════════════════════════════════════════════════════════
// Public API
// ═════════════════════════════════════════════════════════════════════════════

/// Check if we're currently running at EL2
pub fn is_el2() -> bool {
    #[cfg(target_arch = "aarch64")]
    {
        let el: u64;
        unsafe { core::arch::asm!("mrs {}, CurrentEL", out(reg) el) };
        (el >> 2) & 3 == 2
    }
    #[cfg(not(target_arch = "aarch64"))]
    { false }
}

/// Check if EL2 hypervisor is active
pub fn is_active() -> bool {
    HV_ACTIVE.load(Ordering::Relaxed)
}

/// Get total MMIO trap count
pub fn mmio_trap_count() -> u64 {
    MMIO_TRAP_COUNT.load(Ordering::Relaxed)
}

/// Get total SMC trap count  
pub fn smc_trap_count() -> u64 {
    SMC_TRAP_COUNT.load(Ordering::Relaxed)
}

/// Compute the HCR_EL2 value from config
pub fn compute_hcr(config: &HypervisorConfig) -> u64 {
    let mut hcr: u64 = 0;

    // Always: AArch64 EL1, Stage-2 enabled
    hcr |= hcr::RW;   // EL1 = AArch64
    hcr |= hcr::VM;   // Stage-2 translation ON

    // Route interrupts through EL2 for vGIC
    hcr |= hcr::IMO;  // IRQ → EL2
    hcr |= hcr::FMO;  // FIQ → EL2
    hcr |= hcr::AMO;  // SError → EL2

    // Trap SMC to see TrustZone calls
    if config.trap_smc {
        hcr |= hcr::TSC;
    }

    // Trap WFI/WFE for idle monitoring
    if config.trap_wfi {
        hcr |= hcr::TWI;
        hcr |= hcr::TWE;
    }

    // Trap set/way cache maintenance (for cache coherency)
    hcr |= hcr::SWIO;

    hcr
}

/// Generate a summary report of hypervisor activity
pub fn generate_spy_report() -> String {
    let mut out = String::new();

    out.push_str("\x01C== TrustOS EL2 Hypervisor — MMIO Spy Report ==\x01W\n\n");

    if !is_active() {
        out.push_str("Hypervisor is NOT active.\n");
        out.push_str("Use 'hv start <kernel>' to boot a guest OS under EL2 surveillance.\n");
        return out;
    }

    out.push_str(&format!("Status: \x01GACTIVE\x01W (running at EL2)\n"));
    out.push_str(&format!("MMIO traps: {}\n", mmio_trap_count()));
    out.push_str(&format!("SMC intercepts: {}\n", smc_trap_count()));
    out.push_str("\n");

    // Get MMIO spy data
    let events = mmio_spy::recent_mmio_events(50);
    if events.is_empty() {
        out.push_str("No MMIO events captured yet.\n");
    } else {
        out.push_str(&format!("{:<14} {:<6} {:<14} {:<14} {}\n",
            "ADDRESS", "R/W", "VALUE", "SIZE", "DEVICE"));
        out.push_str(&format!("{}\n", "-".repeat(70)));

        for evt in &events {
            let rw = if evt.is_write { "WRITE" } else { "READ" };
            out.push_str(&format!("0x{:010X}  {:<6} 0x{:010X}  {} bytes   {}\n",
                evt.ipa, rw, evt.value, evt.access_size, evt.device_name));
        }

        out.push_str(&format!("\nTotal: {} events captured\n", events.len()));
    }

    // SMC call log
    let smc_log = mmio_spy::recent_smc_events(20);
    if !smc_log.is_empty() {
        out.push_str("\n\x01Y--- SMC (Secure Monitor Calls) ---\x01W\n");
        out.push_str(&format!("{:<14} {:<14} {:<14} {}\n",
            "FID", "X1", "X2", "MEANING"));
        out.push_str(&format!("{}\n", "-".repeat(60)));
        for call in &smc_log {
            out.push_str(&format!("0x{:08X}   0x{:08X}   0x{:08X}   {}\n",
                call.fid, call.x1, call.x2, call.smc_type_name));
        }
    }

    out
}
