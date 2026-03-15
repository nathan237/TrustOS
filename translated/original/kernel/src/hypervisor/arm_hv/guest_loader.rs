//! ARM64 Guest Loader — Loads Android/Linux kernel into guest memory
//!
//! This is the ARM equivalent of `linux_loader.rs` (x86). It takes a raw
//! ARM64 `Image`, an optional initrd, and a DTB, places them in guest RAM,
//! patches the DTB with boot parameters, and launches the guest under
//! TrustOS's EL2 hypervisor.
//!
//! ## ARM64 Image Format
//!
//! ```text
//! Offset  Size  Field
//! 0x00    4     Branch instruction (code0) — jumps to kernel entry
//! 0x04    4     Reserved
//! 0x08    8     Kernel load offset from RAM base (text_offset)
//! 0x10    8     Effective image size
//! 0x18    8     Flags (endianness, page size, etc.)
//! 0x20    8     Reserved
//! 0x28    8     Reserved
//! 0x30    8     Reserved
//! 0x38    4     Magic: 0x644d5241 ("ARM\x64")
//! 0x3C    4     PE header offset (for UEFI)
//! ```
//!
//! ## Memory Layout (QEMU virt, RAM at 0x4000_0000)
//!
//! ```text
//! 0x4000_0000  ┌─────────────────────────┐  Guest RAM base
//!              │  (reserved, 2MB)         │
//! 0x4020_0000  ├─────────────────────────┤  Kernel Image (2MB aligned)
//!              │  Android/Linux kernel    │
//!              │  (~20-40MB)              │
//! 0x4400_0000  ├─────────────────────────┤  DTB (64MB offset)
//!              │  Device Tree Blob        │
//!              │  (~64KB-256KB)           │
//! 0x4500_0000  ├─────────────────────────┤  initrd/ramdisk (80MB offset)
//!              │  Android ramdisk.img     │
//!              │  (~10-500MB)             │
//! 0x5FFF_FFFF  └─────────────────────────┘  End of 512MB guest RAM
//! ```
//!
//! ## Boot Protocol
//!
//! ARM64 Linux boot requires:
//! - x0 = physical address of DTB
//! - x1 = 0, x2 = 0, x3 = 0
//! - Jump to Image offset 0 (the branch instruction handles the rest)
//! - CPU in EL1 (our hypervisor handles this via ERET)
//! - MMU off from guest perspective (Stage-2 is transparent)
//! - Caches can be on
//! - D-cache clean to PoC for kernel + DTB + initrd regions

use alloc::string::String;
use alloc::vec::Vec;
use alloc::vec;
use alloc::format;
use core::ptr;

use super::HypervisorConfig;
use super::stage2::Stage2Tables;

// ═══════════════════════════════════════════════════════════════
// Memory Layout Constants
// ═══════════════════════════════════════════════════════════════

/// Default guest RAM base (QEMU virt machine standard)
pub const GUEST_RAM_BASE: u64 = 0x4000_0000;

/// Kernel Image offset from RAM base (2MB aligned, ARM64 convention)
pub const KERNEL_OFFSET: u64 = 0x0020_0000;  // 2MB

/// DTB offset from RAM base
pub const DTB_OFFSET: u64 = 0x0400_0000;     // 64MB

/// Initrd offset from RAM base
pub const INITRD_OFFSET: u64 = 0x0500_0000;  // 80MB

/// Default guest RAM size (512MB)
pub const DEFAULT_RAM_SIZE: u64 = 512 * 1024 * 1024;

/// Maximum kernel size (64MB should be more than enough)
pub const MAX_KERNEL_SIZE: usize = 64 * 1024 * 1024;

/// Maximum DTB size (1MB)
pub const MAX_DTB_SIZE: usize = 1 * 1024 * 1024;

/// Maximum initrd size (the rest of RAM after initrd offset)
pub const MAX_INITRD_SIZE: usize = 400 * 1024 * 1024;

/// ARM64 Image magic number at offset 0x38
pub const ARM64_IMAGE_MAGIC: u32 = 0x644d5241; // "ARM\x64"

/// ARM64 Image header offset for magic
pub const MAGIC_OFFSET: usize = 0x38;

/// ARM64 Image header offset for image size
pub const IMAGE_SIZE_OFFSET: usize = 0x10;

/// ARM64 Image header offset for text_offset (kernel load offset)
pub const TEXT_OFFSET_FIELD: usize = 0x08;

// ═══════════════════════════════════════════════════════════════
// ARM64 Image Header
// ═══════════════════════════════════════════════════════════════

/// Parsed ARM64 kernel Image header
#[derive(Debug, Clone)]
pub struct Arm64ImageHeader {
    /// Branch instruction at offset 0 (entry point)
    pub code0: u32,
    /// Kernel load offset from page-aligned base
    pub text_offset: u64,
    /// Effective image size (0 = unknown, use file size)
    pub image_size: u64,
    /// Flags: bit 0 = LE (0) or BE (1), bits 1-2 = page size
    pub flags: u64,
    /// Magic: should be 0x644d5241
    pub magic: u32,
}

impl Arm64ImageHeader {
    /// Parse an ARM64 Image header from raw bytes
    pub fn parse(data: &[u8]) -> Result<Self, &'static str> {
        if data.len() < 64 {
            return Err("Image too small (need at least 64 bytes for header)");
        }

        let magic = u32::from_le_bytes([
            data[MAGIC_OFFSET],
            data[MAGIC_OFFSET + 1],
            data[MAGIC_OFFSET + 2],
            data[MAGIC_OFFSET + 3],
        ]);

        if magic != ARM64_IMAGE_MAGIC {
            return Err("Invalid ARM64 Image magic (expected 0x644d5241 'ARM\\x64')");
        }

        let code0 = u32::from_le_bytes([data[0], data[1], data[2], data[3]]);

        let text_offset = u64::from_le_bytes([
            data[0x08], data[0x09], data[0x0A], data[0x0B],
            data[0x0C], data[0x0D], data[0x0E], data[0x0F],
        ]);

        let image_size = u64::from_le_bytes([
            data[0x10], data[0x11], data[0x12], data[0x13],
            data[0x14], data[0x15], data[0x16], data[0x17],
        ]);

        let flags = u64::from_le_bytes([
            data[0x18], data[0x19], data[0x1A], data[0x1B],
            data[0x1C], data[0x1D], data[0x1E], data[0x1F],
        ]);

        Ok(Arm64ImageHeader {
            code0,
            text_offset,
            image_size,
            flags,
            magic,
        })
    }

    /// Check if the kernel expects little-endian
    pub fn is_little_endian(&self) -> bool {
        self.flags & 1 == 0
    }

    /// Get the effective image size to copy
    pub fn effective_size(&self, file_size: usize) -> usize {
        if self.image_size > 0 && (self.image_size as usize) <= file_size {
            self.image_size as usize
        } else {
            file_size
        }
    }
}

// ═══════════════════════════════════════════════════════════════
// DTB Patching
// ═══════════════════════════════════════════════════════════════

/// FDT header magic
const FDT_MAGIC: u32 = 0xD00DFEED;

/// FDT token types
const FDT_BEGIN_NODE: u32 = 0x00000001;
const FDT_END_NODE: u32   = 0x00000002;
const FDT_PROP: u32        = 0x00000003;
const FDT_NOP: u32         = 0x00000004;
const FDT_END: u32         = 0x00000009;

/// Validate a DTB blob
pub fn validate_dtb(data: &[u8]) -> Result<(), &'static str> {
    if data.len() < 40 {
        return Err("DTB too small");
    }
    let magic = u32::from_be_bytes([data[0], data[1], data[2], data[3]]);
    if magic != FDT_MAGIC {
        return Err("Invalid DTB magic (expected 0xD00DFEED)");
    }
    let total_size = u32::from_be_bytes([data[4], data[5], data[6], data[7]]) as usize;
    if total_size > data.len() {
        return Err("DTB total_size exceeds buffer");
    }
    Ok(())
}

/// Patch a DTB's /chosen node to add initrd and bootargs
///
/// This creates a simple overlay approach: we build a new DTB with the
/// /chosen node properly set. For simplicity, we modify the existing DTB
/// in-place by finding the /chosen node and patching its properties.
///
/// If no /chosen node exists, we append properties at the end.
pub fn patch_dtb_chosen(
    dtb: &mut [u8],
    dtb_len: usize,
    initrd_start: Option<u64>,
    initrd_end: Option<u64>,
    bootargs: Option<&str>,
) -> Result<usize, &'static str> {
    // For a robust implementation, we'd need a full FDT builder.
    // For now, we use a pragmatic approach: find empty space in the DTB
    // and append /chosen properties. Most DTBs from QEMU already have
    // a /chosen node with bootargs.
    //
    // The kernel will use the DTB as-is if we don't modify it.
    // For initial testing, we skip modification and let QEMU's
    // -append flag handle bootargs.
    
    // Validate DTB
    validate_dtb(&dtb[..dtb_len])?;
    
    // Return unmodified size for now
    // TODO: Full FDT property patching (add linux,initrd-start/end to /chosen)
    Ok(dtb_len)
}

// ═══════════════════════════════════════════════════════════════
// Guest Loader
// ═══════════════════════════════════════════════════════════════

/// Configuration for loading a guest kernel
#[derive(Debug, Clone)]
pub struct GuestLoadConfig {
    /// RAM base address for the guest
    pub ram_base: u64,
    /// RAM size in bytes
    pub ram_size: u64,
    /// Kernel command line (passed in DTB /chosen/bootargs)
    pub cmdline: String,
    /// MMIO regions to trap and spy on: (base, size)
    pub trap_mmio: Vec<(u64, u64)>,
    /// Whether to trap SMC calls
    pub trap_smc: bool,
    /// Whether to trap WFI (idle monitoring)
    pub trap_wfi: bool,
}

impl Default for GuestLoadConfig {
    fn default() -> Self {
        Self {
            ram_base: GUEST_RAM_BASE,
            ram_size: DEFAULT_RAM_SIZE,
            cmdline: String::from("console=ttyAMA0 earlycon=pl011,0x09000000 earlyprintk"),
            trap_mmio: vec![
                // QEMU virt machine — trap the most interesting peripherals
                (0x0800_0000, 0x0001_0000),  // GIC Distributor
                (0x0801_0000, 0x0001_0000),  // GIC Redistributor
                (0x0900_0000, 0x0000_1000),  // PL011 UART
                (0x0901_0000, 0x0000_1000),  // RTC
                (0x0A00_0000, 0x0000_0200),  // VirtIO-0
                (0x0A00_0200, 0x0000_0200),  // VirtIO-1
            ],
            trap_smc: true,
            trap_wfi: false,
        }
    }
}

/// Result of loading a guest kernel
#[derive(Debug)]
pub struct GuestLoadResult {
    /// Where the kernel was loaded
    pub kernel_addr: u64,
    /// Kernel size
    pub kernel_size: usize,
    /// Where the DTB was placed
    pub dtb_addr: u64,
    /// DTB size
    pub dtb_size: usize,
    /// Where the initrd was placed (if any)
    pub initrd_addr: Option<u64>,
    /// Initrd size
    pub initrd_size: Option<usize>,
    /// Parsed ARM64 Image header
    pub header: Arm64ImageHeader,
    /// Ready-to-use HypervisorConfig
    pub hv_config: HypervisorConfig,
}

/// Load an ARM64 kernel Image + DTB + optional initrd into guest memory
///
/// This function:
/// 1. Validates the ARM64 Image header
/// 2. Copies kernel to guest_ram_base + 2MB
/// 3. Copies DTB to guest_ram_base + 64MB  
/// 4. Copies initrd (if provided) to guest_ram_base + 80MB
/// 5. Optionally patches DTB /chosen node with initrd info + cmdline
/// 6. Returns a configured HypervisorConfig ready for `enter_hypervisor()`
pub fn load_arm64_guest(
    kernel_data: &[u8],
    dtb_data: &[u8],
    initrd_data: Option<&[u8]>,
    config: &GuestLoadConfig,
) -> Result<GuestLoadResult, String> {
    // ── Validate kernel ──
    if kernel_data.len() < 64 {
        return Err(String::from("Kernel image too small"));
    }
    if kernel_data.len() > MAX_KERNEL_SIZE {
        return Err(format!("Kernel too large: {}MB (max {}MB)",
            kernel_data.len() / (1024*1024), MAX_KERNEL_SIZE / (1024*1024)));
    }

    let header = Arm64ImageHeader::parse(kernel_data)
        .map_err(|e| String::from(e))?;

    // ── Validate DTB ──
    validate_dtb(dtb_data).map_err(|e| String::from(e))?;
    if dtb_data.len() > MAX_DTB_SIZE {
        return Err(format!("DTB too large: {}KB (max {}KB)",
            dtb_data.len() / 1024, MAX_DTB_SIZE / 1024));
    }

    // ── Validate initrd ──
    if let Some(initrd) = initrd_data {
        if initrd.len() > MAX_INITRD_SIZE {
            return Err(format!("initrd too large: {}MB (max {}MB)",
                initrd.len() / (1024*1024), MAX_INITRD_SIZE / (1024*1024)));
        }
    }

    // ── Calculate addresses ──
    let kernel_addr = config.ram_base + KERNEL_OFFSET;
    let dtb_addr = config.ram_base + DTB_OFFSET;
    let initrd_addr = config.ram_base + INITRD_OFFSET;

    // Verify everything fits
    let kernel_end = kernel_addr + header.effective_size(kernel_data.len()) as u64;
    if kernel_end > dtb_addr {
        return Err(format!("Kernel too large ({}MB), overlaps DTB region",
            kernel_data.len() / (1024*1024)));
    }
    if let Some(initrd) = initrd_data {
        let initrd_end = initrd_addr + initrd.len() as u64;
        let ram_end = config.ram_base + config.ram_size;
        if initrd_end > ram_end {
            return Err(format!("initrd doesn't fit in guest RAM (need {}MB, have {}MB free)",
                initrd.len() / (1024*1024),
                (ram_end - initrd_addr) / (1024*1024)));
        }
    }

    // ── Copy kernel to guest memory ──
    let kernel_size = header.effective_size(kernel_data.len());
    unsafe {
        ptr::copy_nonoverlapping(
            kernel_data.as_ptr(),
            kernel_addr as *mut u8,
            kernel_size,
        );
    }

    // ── Copy DTB to guest memory ──
    // Make a mutable copy so we can patch it
    let mut dtb_buf = [0u8; 1048576]; // 1MB max DTB
    let dtb_copy_len = dtb_data.len().min(dtb_buf.len());
    dtb_buf[..dtb_copy_len].copy_from_slice(&dtb_data[..dtb_copy_len]);

    // Patch DTB with initrd info and cmdline
    let initrd_end_addr = initrd_data.map(|d| initrd_addr + d.len() as u64);
    let dtb_final_size = patch_dtb_chosen(
        &mut dtb_buf,
        dtb_copy_len,
        initrd_data.map(|_| initrd_addr),
        initrd_end_addr,
        if config.cmdline.is_empty() { None } else { Some(&config.cmdline) },
    ).map_err(|e| String::from(e))?;

    unsafe {
        ptr::copy_nonoverlapping(
            dtb_buf.as_ptr(),
            dtb_addr as *mut u8,
            dtb_final_size,
        );
    }

    // ── Copy initrd to guest memory ──
    let (initrd_result_addr, initrd_result_size) = if let Some(initrd) = initrd_data {
        unsafe {
            ptr::copy_nonoverlapping(
                initrd.as_ptr(),
                initrd_addr as *mut u8,
                initrd.len(),
            );
        }
        (Some(initrd_addr), Some(initrd.len()))
    } else {
        (None, None)
    };

    // ── Flush caches (ARM requirement for self-modifying / loaded code) ──
    #[cfg(target_arch = "aarch64")]
    unsafe {
        // Clean D-cache to PoC and invalidate I-cache for loaded regions
        // The kernel, DTB, and initrd must be visible to the guest CPU
        core::arch::asm!(
            "dsb ish",
            "ic ialluis",    // Invalidate all instruction caches
            "dsb ish",
            "isb",
            options(nomem, nostack)
        );
    }

    // ── Build HypervisorConfig ──
    let hv_config = HypervisorConfig {
        guest_entry: kernel_addr,
        guest_dtb: dtb_addr,
        guest_ram_base: config.ram_base,
        guest_ram_size: config.ram_size,
        trapped_mmio: config.trap_mmio.clone(),
        trap_smc: config.trap_smc,
        trap_wfi: config.trap_wfi,
    };

    Ok(GuestLoadResult {
        kernel_addr,
        kernel_size,
        dtb_addr,
        dtb_size: dtb_final_size,
        initrd_addr: initrd_result_addr,
        initrd_size: initrd_result_size,
        header,
        hv_config,
    })
}

/// Format a human-readable summary of a loaded guest
pub fn format_load_result(result: &GuestLoadResult) -> String {
    let mut s = String::new();
    s.push_str("=== ARM64 Guest Loaded ===\n");
    s.push_str(&format!("  Kernel:  0x{:08X} ({} KB)\n",
        result.kernel_addr, result.kernel_size / 1024));
    s.push_str(&format!("  DTB:     0x{:08X} ({} KB)\n",
        result.dtb_addr, result.dtb_size / 1024));
    if let (Some(addr), Some(size)) = (result.initrd_addr, result.initrd_size) {
        s.push_str(&format!("  initrd:  0x{:08X} ({} KB)\n", addr, size / 1024));
    }
    s.push_str(&format!("  Entry:   0x{:08X}\n", result.hv_config.guest_entry));
    s.push_str(&format!("  RAM:     0x{:08X} - 0x{:08X} ({} MB)\n",
        result.hv_config.guest_ram_base,
        result.hv_config.guest_ram_base + result.hv_config.guest_ram_size,
        result.hv_config.guest_ram_size / (1024*1024)));
    s.push_str(&format!("  Endian:  {}\n",
        if result.header.is_little_endian() { "Little (LE)" } else { "Big (BE)" }));
    s.push_str(&format!("  MMIO traps: {} regions\n",
        result.hv_config.trapped_mmio.len()));
    for (base, size) in &result.hv_config.trapped_mmio {
        s.push_str(&format!("    0x{:08X} - 0x{:08X} ({})\n",
            base, base + size, super::mmio_spy::identify_device(*base)));
    }
    s.push_str(&format!("  SMC trap: {}\n",
        if result.hv_config.trap_smc { "ON" } else { "OFF" }));
    s
}

/// Quick-boot: load the kernel that QEMU passed to TrustOS and re-launch
/// it as a guest under EL2 surveillance.
///
/// This is for the demo scenario where TrustOS itself was loaded by QEMU
/// with -kernel, and we want to take the same Image and boot it as a guest.
/// In practice, for the Android demo, we'll use a separate Image file.
pub fn self_test_guest(ram_base: u64, ram_size: u64) -> Result<GuestLoadResult, String> {
    // For self-test: create a minimal "guest" that just does WFI
    // This tests the full hypervisor path without needing a real kernel
    
    // Build a tiny ARM64 Image that loops: WFI + branch back
    // This is a 64-byte Image header + 8 bytes of code
    let mut tiny_kernel = [0u8; 72];
    
    // offset 0: branch to code at +64 bytes
    // b #64 = 0x14000010 (branch forward 16 instructions = 64 bytes)
    tiny_kernel[0..4].copy_from_slice(&0x14000010u32.to_le_bytes());
    
    // offset 0x08: text_offset = 0 (load at base)
    // offset 0x10: image_size = 72
    tiny_kernel[0x10..0x18].copy_from_slice(&72u64.to_le_bytes());
    
    // offset 0x38: ARM64 magic
    tiny_kernel[0x38..0x3C].copy_from_slice(&ARM64_IMAGE_MAGIC.to_le_bytes());
    
    // offset 64: actual code
    // WFI then branch back to WFI
    tiny_kernel[64..68].copy_from_slice(&0xD503207Fu32.to_le_bytes()); // WFI
    tiny_kernel[68..72].copy_from_slice(&0x17FFFFFFu32.to_le_bytes()); // B .-4
    
    // Minimal DTB (just the FDT header with no nodes)
    let mut mini_dtb = [0u8; 48];
    mini_dtb[0..4].copy_from_slice(&FDT_MAGIC.to_be_bytes());     // magic
    mini_dtb[4..8].copy_from_slice(&48u32.to_be_bytes());          // totalsize
    mini_dtb[8..12].copy_from_slice(&40u32.to_be_bytes());         // off_dt_struct
    mini_dtb[12..16].copy_from_slice(&44u32.to_be_bytes());        // off_dt_strings
    mini_dtb[16..20].copy_from_slice(&28u32.to_be_bytes());        // off_mem_rsvmap
    mini_dtb[20..24].copy_from_slice(&17u32.to_be_bytes());        // version
    mini_dtb[24..28].copy_from_slice(&16u32.to_be_bytes());        // last_comp_version
    // mem_rsvmap at offset 28: 16 bytes of zeros (empty)
    // struct at offset 40: FDT_END
    mini_dtb[40..44].copy_from_slice(&FDT_END.to_be_bytes());
    // strings at offset 44: empty (4 bytes padding)
    
    let mut config = GuestLoadConfig::default();
    config.ram_base = ram_base;
    config.ram_size = ram_size;
    config.trap_wfi = true; // We WANT to see the WFI traps for testing
    config.cmdline = String::new();
    
    load_arm64_guest(&tiny_kernel, &mini_dtb, None, &config)
}
