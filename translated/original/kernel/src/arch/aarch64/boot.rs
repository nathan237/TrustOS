//! aarch64 Boot / Early Init
//!
//! Platform-specific early initialization for ARM64.
//! Includes MMIO identity mapping setup for Device-nGnRnE UART access.

use core::sync::atomic::{AtomicU64, Ordering};

/// Early platform init for aarch64
/// Sets up basic CPU features before the generic kernel runs.
pub fn early_init() {
    // On ARM64, Limine already sets up EL1 with MMU enabled.
    // We just need to ensure caches are on and alignment check is off.
    unsafe {
        let mut sctlr = super::cpu::read_sctlr_el1();
        sctlr |= 1 << 2;   // C — enable data cache
        sctlr |= 1 << 12;  // I — enable instruction cache
        sctlr &= !(1 << 1); // A — disable alignment check (for unaligned access)
        super::cpu::write_sctlr_el1(sctlr);
    }
}

/// Limine boot protocol is used on aarch64 (UEFI)
pub const BOOTLOADER: &str = "Limine";

// ============================================================================
// MMIO Identity Mapping — Enables PL011 UART access after Limine boot
// ============================================================================
//
// Problem: Limine sets TTBR0 to an empty table (no lower-half mappings).
// The PL011 UART is at physical 0x0900_0000, which is a lower-half address.
// Limine's HHDM maps all physical memory but with Normal Cacheable attributes,
// which doesn't work for MMIO (writes get cached, never reach the device).
//
// Solution: Create a minimal identity mapping in TTBR0 for the first 1GB:
//   PA 0x0000_0000 → VA 0x0000_0000 (1GB block, Device-nGnRnE)
// This allows direct physical address access to UART, GIC, VirtIO, etc.
//
// Page table layout (4KB granule, 48-bit VA):
//   L0 table (512 entries, each covers 512GB) → entry[0] → L1 table
//   L1 table (512 entries, each covers 1GB)   → entry[0] → 1GB block @ 0x0

/// 4KB-aligned page table for TTBR0 Level 0
#[repr(C, align(4096))]
struct PageTable4K([u64; 512]);

/// Static page tables in kernel BSS (will be zeroed by Limine)
static mut TTBR0_L0: PageTable4K = PageTable4K([0u64; 512]);
static mut TTBR0_L1: PageTable4K = PageTable4K([0u64; 512]);

/// Whether the MMIO identity map has been set up
static MMIO_MAP_READY: AtomicU64 = AtomicU64::new(0);

/// AArch64 page table descriptor bits
const PT_VALID: u64    = 1 << 0;     // Valid entry
const PT_TABLE: u64    = 1 << 1;     // Table descriptor (vs block)
const PT_BLOCK: u64    = 0 << 1;     // Block descriptor (1GB at L1)
const PT_AF: u64       = 1 << 10;    // Access Flag (must be set)
const PT_SH_ISH: u64   = 0b11 << 8;  // Inner Shareable
const PT_AP_RW_EL1: u64 = 0b00 << 6; // R/W at EL1
const PT_UXN: u64      = 1 << 54;    // Unprivileged Execute Never
const PT_PXN: u64      = 1 << 53;    // Privileged Execute Never

/// MAIR attribute index for Device-nGnRnE
/// We use index 4 (an unused slot) to avoid disturbing Limine's existing MAIR entries.
/// Limine typically uses: attr0=Normal WB (0xFF), attr1=Device-nGnRnE (0x00)
/// We set attr4=Device-nGnRnE and reference it from our TTBR0 block descriptor.
const MAIR_DEVICE_IDX: u64 = 4; // AttrIndx[2:0] = 4 → bits[4:2]

/// Set up a minimal identity mapping in TTBR0_EL1 for MMIO access.
///
/// This creates a 1GB block mapping at VA 0x0000_0000 → PA 0x0000_0000
/// with Device-nGnRnE attributes, enabling direct MMIO access to PL011 UART,
/// GIC distributor, VirtIO devices, and other peripherals.
///
/// # Arguments
/// * `kernel_virt_base` — Virtual base address of kernel (from KernelAddressRequest)
/// * `kernel_phys_base` — Physical base address of kernel (from KernelAddressRequest)
///
/// # Safety
/// Must be called early in boot, before any MMIO access through TTBR0 addresses.
pub unsafe fn setup_mmio_identity_map(kernel_virt_base: u64, kernel_phys_base: u64) {
    // Compute virt-to-phys offset for kernel statics (BSS)
    // phys = virt - kernel_virt_base + kernel_phys_base
    let virt_to_phys = |virt: u64| -> u64 {
        virt.wrapping_sub(kernel_virt_base).wrapping_add(kernel_phys_base)
    };

    // Get physical addresses of our static page tables
    let l0_phys = virt_to_phys(&raw const TTBR0_L0 as u64);
    let l1_phys = virt_to_phys(&raw const TTBR0_L1 as u64);

    // Set MAIR_EL1 attr4 = 0x00 (Device-nGnRnE) WITHOUT touching existing entries
    // Limine's existing attrs (0-3) are preserved — only attr4 is modified
    let mair: u64;
    core::arch::asm!("mrs {}, MAIR_EL1", out(reg) mair, options(nomem, nostack));
    let attr4_mask: u64 = 0xFF << 32; // attr4 occupies bits[39:32]
    let new_mair = (mair & !attr4_mask) | (0x00u64 << 32); // Device-nGnRnE at index 4
    core::arch::asm!("msr MAIR_EL1, {}", in(reg) new_mair, options(nomem, nostack));
    core::arch::asm!("isb", options(nomem, nostack));

    // Build L1 table: entry[0] = 1GB block at PA 0x0, Device-nGnRnE, R/W, no-exec
    let block_desc: u64 = 0x0000_0000  // Output address (PA 0x0, 1GB aligned)
        | PT_VALID
        | PT_BLOCK     // Block descriptor (not table)
        | PT_AF        // Access flag
        | PT_SH_ISH    // Inner shareable
        | PT_AP_RW_EL1 // R/W at EL1
        | (MAIR_DEVICE_IDX << 2)  // AttrIndx = 0 (Device-nGnRnE)
        | PT_UXN       // No unprivileged execute
        | PT_PXN;      // No privileged execute (it's MMIO, not code)

    TTBR0_L1.0[0] = block_desc;

    // Build L0 table: entry[0] = table descriptor pointing to L1
    let table_desc: u64 = l1_phys
        | PT_VALID
        | PT_TABLE;    // Table descriptor

    TTBR0_L0.0[0] = table_desc;

    // Ensure writes are visible
    core::arch::asm!("dsb ishst", options(nomem, nostack)); // Ensure table writes complete
    core::arch::asm!("isb", options(nomem, nostack));

    // Install our page table in TTBR0_EL1
    // TTBR0_EL1 format: BADDR[47:1] | CnP[0]
    core::arch::asm!("msr TTBR0_EL1, {}", in(reg) l0_phys, options(nomem, nostack));

    // Invalidate TLB for TTBR0 address space
    core::arch::asm!("tlbi vmalle1is", options(nomem, nostack));
    core::arch::asm!("dsb ish", options(nomem, nostack));
    core::arch::asm!("isb", options(nomem, nostack));

    MMIO_MAP_READY.store(1, Ordering::Release);
}

/// Check if MMIO identity mapping is ready
pub fn is_mmio_mapped() -> bool {
    MMIO_MAP_READY.load(Ordering::Acquire) != 0
}
