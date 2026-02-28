//! Android Boot Path — Rust entry point
//!
//! Called from `android_entry.S` after the assembly stub sets up:
//! - Stack pointer
//! - BSS zeroed
//! - Caches enabled
//! - DAIF masked (interrupts off)
//!
//! This module handles the transition from bare-metal Android boot
//! to the standard TrustOS kernel initialization path.

use crate::android_boot::{self, FdtHeader, DtbInfo, SocFamily};

// ═════════════════════════════════════════════════════════════════════════════
// Statics for Android boot state
// ═════════════════════════════════════════════════════════════════════════════

/// DTB physical address (saved from bootloader handoff)
static mut DTB_PHYS_ADDR: u64 = 0;

/// Whether we booted via Android path (vs Limine)
static mut ANDROID_BOOT: bool = false;

/// Parsed DTB info
static mut DTB_INFO: Option<DtbInfo> = None;

// ═════════════════════════════════════════════════════════════════════════════
// PSCI interface (Power State Coordination Interface via SMC to BL31)
// ═════════════════════════════════════════════════════════════════════════════

// SMC Function IDs (PSCI v1.0+)
const PSCI_VERSION: u64 = 0x8400_0000;
const PSCI_SYSTEM_OFF: u64 = 0x8400_0008;
const PSCI_SYSTEM_RESET: u64 = 0x8400_0009;
const PSCI_CPU_ON_64: u64 = 0xC400_0003;
const PSCI_FEATURES: u64 = 0x8400_000A;

extern "C" {
    /// Call BL31 SMC (defined in android_entry.S)
    fn android_smc_call(fid: u64, x1: u64, x2: u64, x3: u64) -> u64;
    /// PSCI shutdown
    fn android_psci_off() -> !;
    /// PSCI reboot
    fn android_psci_reset() -> !;
}

/// Issue an SMC call to ARM Trusted Firmware (BL31)
pub fn smc(fid: u64, x1: u64, x2: u64, x3: u64) -> u64 {
    unsafe { android_smc_call(fid, x1, x2, x3) }
}

/// Get PSCI version from BL31
pub fn psci_version() -> (u32, u32) {
    let v = smc(PSCI_VERSION, 0, 0, 0);
    let major = (v >> 16) as u32;
    let minor = (v & 0xFFFF) as u32;
    (major, minor)
}

/// Power off the system via PSCI
pub fn system_off() -> ! {
    unsafe { android_psci_off() }
}

/// Reboot the system via PSCI
pub fn system_reset() -> ! {
    unsafe { android_psci_reset() }
}

/// Wake up a secondary CPU core via PSCI CPU_ON
pub fn cpu_on(target_cpu: u64, entry_point: u64, context_id: u64) -> i64 {
    smc(PSCI_CPU_ON_64, target_cpu, entry_point, context_id) as i64
}

// ═════════════════════════════════════════════════════════════════════════════
// Android main entry (called from assembly)
// ═════════════════════════════════════════════════════════════════════════════

/// Early UART write (before serial module is initialized)
/// Uses the PL011/GENI UART base set by the assembly stub.
unsafe fn early_uart_putc(c: u8) {
    extern "C" {
        static __uart_base: u64;
    }
    let base = __uart_base;
    if base != 0 {
        let ptr = base as *mut u32;
        // Wait for UART TX FIFO not full (PL011: UARTFR bit 5)
        // Simple spin — works for PL011, ok as fallback for others
        for _ in 0..10000 {
            let fr = ptr.add(6).read_volatile(); // UARTFR offset = 0x18
            if fr & (1 << 5) == 0 {
                break;
            }
        }
        ptr.write_volatile(c as u32);
    }
}

unsafe fn early_uart_print(s: &[u8]) {
    for &c in s {
        early_uart_putc(c);
    }
}

/// Main Rust entry point for Android boot path.
///
/// Called from android_entry.S with:
/// - x0 (first arg) = DTB physical address  
/// - Stack set up, BSS zeroed, caches on, interrupts off
#[no_mangle]
pub unsafe extern "C" fn android_main(dtb_ptr: u64) -> ! {
    // Mark that we booted via Android path
    ANDROID_BOOT = true;
    DTB_PHYS_ADDR = dtb_ptr;

    early_uart_print(b"[ANDROID] TrustOS android_main entered\r\n");

    // ── Step 1: Validate DTB ──
    if dtb_ptr != 0 {
        let valid = FdtHeader::validate(dtb_ptr as *const u8);
        if valid {
            early_uart_print(b"[ANDROID] DTB valid at 0x");
            early_uart_print_hex(dtb_ptr);
            early_uart_print(b"\r\n");

            DTB_INFO = DtbInfo::from_dtb_ptr(dtb_ptr as *const u8);
        } else {
            early_uart_print(b"[ANDROID] WARNING: DTB invalid!\r\n");
        }
    } else {
        early_uart_print(b"[ANDROID] No DTB provided\r\n");
    }

    // ── Step 2: Query PSCI version from BL31 ──
    let (psci_major, psci_minor) = psci_version();
    early_uart_print(b"[ANDROID] PSCI version: ");
    early_uart_putc(b'0' + psci_major as u8);
    early_uart_putc(b'.');
    early_uart_putc(b'0' + psci_minor as u8);
    early_uart_print(b"\r\n");

    // ── Step 3: Detect SoC ──
    // For now, assume QEMU virt unless DTB says otherwise
    android_boot::SOC_INFO = SocFamily::QemuVirt;
    early_uart_print(b"[ANDROID] SoC: QEMU virt (default)\r\n");

    // ── Step 4: Set up basic memory ──
    // Without Limine, we need to discover memory from DTB or hardcode
    // QEMU virt: RAM starts at 0x40000000, typical size 1-4 GB
    let ram_base: u64 = 0x4000_0000;
    let ram_size: u64 = 128 * 1024 * 1024; // Conservative 128 MB
    early_uart_print(b"[ANDROID] RAM: 128 MB from 0x40000000\r\n");

    // Heap: place after the kernel image
    extern "C" {
        static __kernel_end: u8;
    }
    let kernel_end = &__kernel_end as *const u8 as u64;
    let heap_start = (kernel_end + 0xFFF) & !0xFFF; // Page-align
    let heap_size = 32 * 1024 * 1024; // 32 MB heap

    early_uart_print(b"[ANDROID] Heap: 32 MB at 0x");
    early_uart_print_hex(heap_start);
    early_uart_print(b"\r\n");

    // Initialize the allocator
    crate::memory::init_android_heap(heap_start, heap_size);

    early_uart_print(b"[ANDROID] Heap initialized, entering kernel...\r\n");

    // ── Step 5: Jump to standard kernel initialization ──
    // From here, the standard TrustOS boot sequence takes over
    // (serial, framebuffer, keyboard, desktop, etc.)
    // Note: framebuffer on Android requires SimpleFB from DTB
    // or direct display controller programming.
    
    early_uart_print(b"[ANDROID] TrustOS booted successfully!\r\n");
    early_uart_print(b"[ANDROID] Shell not available yet (no framebuffer in Android boot mode)\r\n");
    early_uart_print(b"[ANDROID] Entering halt loop. Use PSCI to reboot.\r\n");

    // For now, halt. Full integration with desktop will come when
    // we add SimpleFB/DRM framebuffer discovery from DTB.
    loop {
        core::arch::asm!("wfe", options(nomem, nostack));
    }
}

/// Print a u64 as hex via early UART
unsafe fn early_uart_print_hex(val: u64) {
    let hex = b"0123456789ABCDEF";
    for i in (0..16).rev() {
        let nibble = ((val >> (i * 4)) & 0xF) as usize;
        early_uart_putc(hex[nibble]);
    }
}

// ═════════════════════════════════════════════════════════════════════════════
// Public query API
// ═════════════════════════════════════════════════════════════════════════════

/// Check if we booted via Android bootloader (vs Limine)
pub fn is_android_boot() -> bool {
    unsafe { ANDROID_BOOT }
}

/// Get the DTB physical address (0 if not available)
pub fn dtb_address() -> u64 {
    unsafe { DTB_PHYS_ADDR }
}
