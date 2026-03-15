//! Android Boot Support for aarch64
//!
//! Enables TrustOS to boot on Android devices via `fastboot flash boot`.
//!
//! ## Boot Protocol
//! Android bootloader (ABL/U-Boot) loads the kernel at a physical address
//! and jumps to it with:
//! - x0 = physical address of Device Tree Blob (DTB)
//! - x1 = 0 (reserved)
//! - CPU in EL1 (normal world), MMU off, caches off
//! - BL31 (ARM Trusted Firmware) running in EL3 for SMC services
//!
//! ## Android Boot Image Format
//! The kernel is packaged in a `boot.img` with this header:
//! ```text
//! +------------------+
//! | Boot Header      |  (page-aligned, 4096 bytes)
//! +------------------+
//! | Kernel           |  (page-aligned)  ← TrustOS binary
//! +------------------+
//! | Ramdisk          |  (page-aligned, optional)
//! +------------------+
//! | DTB              |  (page-aligned, v2+)
//! +------------------+
//! ```

use core::fmt;

// ═════════════════════════════════════════════════════════════════════════════
// Android Boot Image Header v2 (most compatible — works on Pixel 3+)
// ═════════════════════════════════════════════════════════════════════════════

/// ANDROID! magic bytes
pub const BOOT_MAGIC: &[u8; 8] = b"ANDROID!";

/// Standard Android boot image page size
pub const BOOT_PAGE_SIZE: u32 = 4096;

/// Android Boot Image Header v2
/// Based on: https://source.android.com/docs/core/architecture/bootloader/boot-image-header
#[repr(C)]
#[derive(Clone)]
pub struct BootImgHeaderV2 {
    /// Magic: "ANDROID!" (8 bytes)
    pub magic: [u8; 8],
    /// Size of the kernel in bytes
    pub kernel_size: u32,
    /// Physical load address of the kernel
    pub kernel_addr: u32,
    /// Size of the ramdisk in bytes (0 for TrustOS)
    pub ramdisk_size: u32,
    /// Physical load address of the ramdisk
    pub ramdisk_addr: u32,
    /// Size of second-stage bootloader (0)
    pub second_size: u32,
    /// Physical load address of second-stage (unused)
    pub second_addr: u32,
    /// Physical address of device tree / tags
    pub tags_addr: u32,
    /// Flash page size (usually 4096)
    pub page_size: u32,
    /// Header version (2)
    pub header_version: u32,
    /// OS version (packed: A.B.C + month/year)
    pub os_version: u32,
    /// Product name string (16 bytes)
    pub name: [u8; 16],
    /// Kernel command line (512 bytes)
    pub cmdline: [u8; 512],
    /// SHA-1 hash of kernel + ramdisk + second (32 bytes)
    pub id: [u8; 32],
    /// Extra command line (1024 bytes)
    pub extra_cmdline: [u8; 1024],
    // ── Header v1 extensions ──
    /// Size of recovery DTBO (0 for normal boot)
    pub recovery_dtbo_size: u32,
    /// Offset of recovery DTBO in boot image
    pub recovery_dtbo_offset: u64,
    /// Total header size including extensions
    pub header_size: u32,
    // ── Header v2 extensions ──
    /// Size of DTB (Device Tree Blob) appended after ramdisk
    pub dtb_size: u32,
    /// Physical load address for DTB
    pub dtb_addr: u64,
}

impl BootImgHeaderV2 {
    /// Create a new boot image header for TrustOS
    pub fn new_trustos(kernel_size: u32, kernel_addr: u32) -> Self {
        let mut header = Self {
            magic: *BOOT_MAGIC,
            kernel_size,
            kernel_addr,
            ramdisk_size: 0,
            ramdisk_addr: 0x01000000,      // Standard ramdisk addr (unused)
            second_size: 0,
            second_addr: 0,
            tags_addr: 0x00000100,
            page_size: BOOT_PAGE_SIZE,
            header_version: 2,
            os_version: Self::pack_os_version(1, 0, 0, 2026, 2), // TrustOS v1.0.0
            name: [0u8; 16],
            cmdline: [0u8; 512],
            id: [0u8; 32],
            extra_cmdline: [0u8; 1024],
            recovery_dtbo_size: 0,
            recovery_dtbo_offset: 0,
            header_size: core::mem::size_of::<Self>() as u32,
            dtb_size: 0,
            dtb_addr: 0,
        };

        // Set product name
        let name = b"TrustOS";
        header.name[..name.len()].copy_from_slice(name);

        // Set kernel command line
        let cmdline = b"trustos.mode=desktop trustos.serial=ttyS0";
        header.cmdline[..cmdline.len()].copy_from_slice(cmdline);

        header
    }

    /// Pack OS version: A.B.C and YYYY-MM
    /// Format: (A << 25) | (B << 18) | (C << 11) | ((Y-2000) << 4) | M
    fn pack_os_version(a: u32, b: u32, c: u32, year: u32, month: u32) -> u32 {
        (a << 25) | (b << 18) | (c << 11) | ((year - 2000) << 4) | month
    }

    /// Serialize to bytes (for boot.img generation)
    pub fn as_bytes(&self) -> &[u8] {
        unsafe {
            core::slice::from_raw_parts(
                self as *const Self as *const u8,
                core::mem::size_of::<Self>(),
            )
        }
    }
}

// ═════════════════════════════════════════════════════════════════════════════
// Android Boot Image Header v4 (for Pixel 6+ / GKI 2.0)
// ═════════════════════════════════════════════════════════════════════════════

/// Android Boot Image Header v4
/// Used on modern devices with Generic Kernel Image (GKI).
/// The vendor_boot partition carries the DTB separately.
#[repr(C)]
#[derive(Clone)]
pub struct BootImgHeaderV4 {
    /// Magic: "ANDROID!" (8 bytes)
    pub magic: [u8; 8],
    /// Size of the kernel in bytes
    pub kernel_size: u32,
    /// Size of the ramdisk in bytes (0 for TrustOS)
    pub ramdisk_size: u32,
    /// OS version (packed)
    pub os_version: u32,
    /// Total header size
    pub header_size: u32,
    /// Reserved (4 * u32)
    pub reserved: [u32; 4],
    /// Header version (4)
    pub header_version: u32,
    /// Kernel command line (1536 bytes)
    pub cmdline: [u8; 1536],
    /// Signature size (0 if unsigned, for AVB)
    pub signature_size: u32,
}

// ═════════════════════════════════════════════════════════════════════════════
// Device Tree Blob (DTB) minimal parser
// ═════════════════════════════════════════════════════════════════════════════

/// FDT (Flattened Device Tree) magic number
pub const FDT_MAGIC: u32 = 0xD00DFEED;

/// Minimal FDT header for validation
#[repr(C)]
pub struct FdtHeader {
    pub magic: u32,
    pub totalsize: u32,
    pub off_dt_struct: u32,
    pub off_dt_strings: u32,
    pub off_mem_rsvmap: u32,
    pub version: u32,
    pub last_comp_version: u32,
    pub boot_cpuid_phys: u32,
    pub size_dt_strings: u32,
    pub size_dt_struct: u32,
}

impl FdtHeader {
    /// Validate that a memory region looks like a valid DTB
    pub unsafe fn validate(ptr: *const u8) -> bool {
        if ptr.is_null() {
            return false;
        }
        let magic = u32::from_be((ptr as *const u32).read_unaligned());
        magic == FDT_MAGIC
    }

    /// Read header from a physical address
    pub unsafe fn from_ptr(ptr: *const u8) -> Option<&'static Self> {
        if !Self::validate(ptr) {
            return None;
        }
        Some(&*(ptr as *const Self))
    }

    /// Total size of the DTB (big-endian)
    pub fn total_size(&self) -> u32 {
        u32::from_be(self.totalsize)
    }
}

// ═════════════════════════════════════════════════════════════════════════════
// DTB Property extraction (minimal — no full libfdt dependency)
// ═════════════════════════════════════════════════════════════════════════════

/// Extract a simple string property from a FDT
/// This is a minimal parser — doesn't handle nested nodes fully.
/// For full DTB support, we'd use a proper `fdt` crate.
pub struct DtbInfo {
    /// Physical base of DTB
    pub dtb_base: u64,
    /// Total size of DTB
    pub dtb_size: u32,
    /// Detected SoC model (from /model or /compatible)
    pub model: [u8; 64],
    pub model_len: usize,
    /// Memory base address (from /memory reg)
    pub mem_base: u64,
    /// Memory size (from /memory reg)
    pub mem_size: u64,
    /// Serial port base address (from /chosen stdout-path)
    pub serial_base: u64,
}

impl Default for DtbInfo {
    fn default() -> Self {
        Self {
            dtb_base: 0,
            dtb_size: 0,
            model: [0u8; 64],
            model_len: 0,
            mem_base: 0,
            mem_size: 0,
            serial_base: 0,
        }
    }
}

impl DtbInfo {
    /// Quick parse: extract basic info from DTB at given physical address
    pub unsafe fn from_dtb_ptr(ptr: *const u8) -> Option<Self> {
        let header = FdtHeader::from_ptr(ptr)?;
        let mut info = DtbInfo::default();
        info.dtb_base = ptr as u64;
        info.dtb_size = header.total_size();

        // DTB is valid but full parsing requires more code.
        // For now, we just record the base/size so the kernel can
        // pass it to a proper DTB parser later.
        Some(info)
    }
}

// ═════════════════════════════════════════════════════════════════════════════
// SoC Detection (for hardware-specific init)
// ═════════════════════════════════════════════════════════════════════════════

/// Known Android SoC families
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SocFamily {
    /// Unknown SoC
    Unknown,
    /// QEMU virt platform (for testing)
    QemuVirt,
    /// Qualcomm Snapdragon (Pixel, OnePlus, Samsung Galaxy)
    Qualcomm,
    /// Samsung Exynos
    Exynos,
    /// MediaTek Dimensity
    MediaTek,
    /// Google Tensor (Pixel 6+, actually Samsung-fabbed)
    Tensor,
    /// Raspberry Pi (Broadcom BCM2711/BCM2712)
    Broadcom,
}

impl fmt::Display for SocFamily {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unknown => write!(f, "Unknown"),
            Self::QemuVirt => write!(f, "QEMU virt"),
            Self::Qualcomm => write!(f, "Qualcomm Snapdragon"),
            Self::Exynos => write!(f, "Samsung Exynos"),
            Self::MediaTek => write!(f, "MediaTek"),
            Self::Tensor => write!(f, "Google Tensor"),
            Self::Broadcom => write!(f, "Broadcom (RPi)"),
        }
    }
}

/// Detected SoC information (populated during early boot)
pub static mut SOC_INFO: SocFamily = SocFamily::Unknown;

/// Get the detected SoC family
pub fn soc_family() -> SocFamily {
    unsafe { SOC_INFO }
}

// ═════════════════════════════════════════════════════════════════════════════
// Android-specific kernel load addresses by SoC
// ═════════════════════════════════════════════════════════════════════════════

/// Standard kernel load addresses per SoC vendor
pub struct SocAddresses {
    pub kernel_base: u64,
    pub dtb_base: u64,
    pub ramdisk_base: u64,
    pub uart_base: u64,
}

impl SocAddresses {
    /// Get standard addresses for a SoC family
    pub const fn for_soc(soc: SocFamily) -> Self {
        match soc {
            SocFamily::QemuVirt => Self {
                kernel_base: 0x4008_0000,
                dtb_base: 0x4000_0000,
                ramdisk_base: 0x4400_0000,
                uart_base: 0x0900_0000,  // PL011
            },
            SocFamily::Qualcomm => Self {
                kernel_base: 0x8008_0000,
                dtb_base: 0x8300_0000,
                ramdisk_base: 0x8200_0000,
                uart_base: 0x0078_AF00,  // GENI UART (Snapdragon 8xx)
            },
            SocFamily::Tensor => Self {
                kernel_base: 0x8008_0000,
                dtb_base: 0x8300_0000,
                ramdisk_base: 0x8200_0000,
                uart_base: 0x10A0_0000,  // Samsung USI UART
            },
            SocFamily::Broadcom => Self {
                kernel_base: 0x0008_0000,
                dtb_base: 0x0000_0100,
                ramdisk_base: 0x0200_0000,
                uart_base: 0xFE20_1000,  // BCM2711 mini UART
            },
            SocFamily::Exynos | SocFamily::MediaTek | SocFamily::Unknown => Self {
                kernel_base: 0x8008_0000,
                dtb_base: 0x8300_0000,
                ramdisk_base: 0x8200_0000,
                uart_base: 0x1102_0000,  // Generic
            },
        }
    }
}
