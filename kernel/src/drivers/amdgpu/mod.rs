//! AMD GPU Driver — Phase 1: PCIe Discovery & MMIO
//!
//! Native AMD Radeon GPU driver for TrustOS.
//! Targets AMD Navi 10 (RDNA 1) — RX 5600 XT / RX 5700 / RX 5700 XT.
//!
//! Phase 1: PCIe enumeration, BAR mapping, GPU identity register reads.
//!
//! Architecture:
//! - Discover AMD GPU via PCI bus scan (vendor 0x1002)
//! - Map MMIO BARs (BAR0 = registers, BAR2 = VRAM aperture)
//! - Read GPU identity registers (ASIC family, revision, VRAM info)
//! - Report hardware capabilities to the kernel
//!
//! References:
//! - AMD GPU register headers: https://github.com/torvalds/linux/tree/master/drivers/gpu/drm/amd
//! - AMDGPU kernel driver docs: https://docs.kernel.org/gpu/amdgpu/

pub mod regs;
pub mod dcn;

use alloc::string::String;
use alloc::format;
use alloc::vec::Vec;
use core::sync::atomic::{AtomicBool, Ordering};
use spin::Mutex;

use crate::pci::{self, PciDevice};
use crate::memory;

// ═══════════════════════════════════════════════════════════════════════════════
// AMD GPU Constants
// ═══════════════════════════════════════════════════════════════════════════════

/// AMD/ATI PCI vendor ID
pub const AMD_VENDOR_ID: u16 = 0x1002;

/// Known Navi 10 (RDNA 1) device IDs
pub const NAVI10_DEVICE_IDS: &[u16] = &[
    0x7310, // Navi 10 (unused/engineering)
    0x7312, // Navi 10 (Pro W5700X)
    0x7318, // Navi 10 (unused)
    0x7319, // Navi 10 (unused)
    0x731A, // Navi 10 (unused)
    0x731B, // Navi 10 (unused)
    0x731E, // Navi 10 (unused)
    0x731F, // Navi 10 — RX 5600 XT / RX 5700 / RX 5700 XT
    0x7340, // Navi 14 — RX 5500 XT
    0x7341, // Navi 14 (Pro W5500)
    0x7347, // Navi 14 (Pro W5500M)
    0x734F, // Navi 14 (unused)
];

/// AMD GPU family identifiers from ASIC registers
pub mod family {
    pub const FAMILY_UNKNOWN:  u32 = 0;
    pub const FAMILY_NV:       u32 = 143; // Navi 10 / RDNA 1
    pub const FAMILY_NV14:     u32 = 144; // Navi 14 / RDNA 1 (cut-down)
    pub const FAMILY_AI:       u32 = 141; // Vega 10 / GCN 5
    pub const FAMILY_RV:       u32 = 142; // Raven Ridge APU
}

/// PCI BAR indices for AMD GPUs
pub mod bar {
    /// BAR0: Register MMIO (256KB–512KB)  — GPU control registers
    pub const MMIO_REGISTERS: usize = 0;
    /// BAR2: VRAM aperture / doorbell (256MB) — GPU memory window
    pub const VRAM_APERTURE: usize = 2;
    /// BAR4: Doorbell (for some chips) — queue notifications
    pub const DOORBELL: usize = 4;
}

/// PCIe capability IDs
pub mod pcie_cap {
    pub const PCI_EXPRESS: u8 = 0x10;
    pub const MSI: u8 = 0x05;
    pub const MSIX: u8 = 0x11;
    pub const POWER_MGMT: u8 = 0x01;
}

// ═══════════════════════════════════════════════════════════════════════════════
// AMD GPU State
// ═══════════════════════════════════════════════════════════════════════════════

/// GPU hardware information gathered from registers
#[derive(Debug, Clone)]
pub struct GpuInfo {
    /// PCI vendor:device
    pub vendor_id: u16,
    pub device_id: u16,
    /// PCI location
    pub bus: u8,
    pub device: u8,
    pub function: u8,
    /// PCI revision
    pub revision: u8,
    /// ASIC family (from registers)
    pub asic_family: u32,
    /// Chip external revision
    pub chip_external_rev: u32,
    /// GPU family name string
    pub family_name: &'static str,
    /// VRAM size in bytes (from register probing)
    pub vram_size: u64,
    /// VRAM type description
    pub vram_type: &'static str,
    /// BAR0 MMIO base (physical)
    pub mmio_base_phys: u64,
    /// BAR0 MMIO base (virtual, mapped)
    pub mmio_base_virt: u64,
    /// BAR0 MMIO size
    pub mmio_size: u64,
    /// BAR2 VRAM aperture base (physical)
    pub vram_aperture_phys: u64,
    /// BAR2 VRAM aperture size
    pub vram_aperture_size: u64,
    /// PCIe link speed
    pub pcie_link_speed: u8,
    /// PCIe link width (lanes)
    pub pcie_link_width: u8,
    /// Has MSI capability
    pub has_msi: bool,
    /// Has MSI-X capability
    pub has_msix: bool,
    /// Number of compute units (if readable)
    pub compute_units: u32,
    /// GPU clock in MHz (if readable)
    pub gpu_clock_mhz: u32,
}

impl GpuInfo {
    /// Get a human-readable GPU name
    pub fn gpu_name(&self) -> &'static str {
        match self.device_id {
            0x731F => match self.revision {
                0xC1 => "AMD Radeon RX 5700 XT",
                0xC0 | 0xC4 => "AMD Radeon RX 5700",
                0xC2 | 0xC3 => "AMD Radeon RX 5600 XT",
                _ => "AMD Radeon Navi 10",
            },
            0x7340 => "AMD Radeon RX 5500 XT",
            0x7341 => "AMD Radeon Pro W5500",
            0x7312 => "AMD Radeon Pro W5700X",
            _ => "AMD Radeon (Unknown Navi)",
        }
    }

    /// Format VRAM size nicely
    pub fn vram_string(&self) -> String {
        if self.vram_size >= 1024 * 1024 * 1024 {
            format!("{} GB", self.vram_size / (1024 * 1024 * 1024))
        } else if self.vram_size >= 1024 * 1024 {
            format!("{} MB", self.vram_size / (1024 * 1024))
        } else if self.vram_size > 0 {
            format!("{} KB", self.vram_size / 1024)
        } else {
            String::from("Unknown")
        }
    }

    /// Format PCIe link info
    pub fn pcie_link_string(&self) -> String {
        let speed = match self.pcie_link_speed {
            1 => "2.5 GT/s (Gen1)",
            2 => "5.0 GT/s (Gen2)",
            3 => "8.0 GT/s (Gen3)",
            4 => "16.0 GT/s (Gen4)",
            _ => "Unknown",
        };
        format!("PCIe x{} {}", self.pcie_link_width, speed)
    }
}

/// Driver state
struct AmdGpuState {
    /// Whether the driver has been initialized
    initialized: bool,
    /// Detected GPU information
    gpu_info: Option<GpuInfo>,
}

static GPU_STATE: Mutex<AmdGpuState> = Mutex::new(AmdGpuState {
    initialized: false,
    gpu_info: None,
});

static GPU_DETECTED: AtomicBool = AtomicBool::new(false);

// ═══════════════════════════════════════════════════════════════════════════════
// MMIO Helpers
// ═══════════════════════════════════════════════════════════════════════════════

/// Read a 32-bit GPU register via MMIO
/// 
/// # Safety
/// `mmio_base` must be a valid mapped virtual address for GPU MMIO space
#[inline]
pub unsafe fn mmio_read32(mmio_base: u64, offset: u32) -> u32 {
    let addr = mmio_base + offset as u64;
    core::ptr::read_volatile(addr as *const u32)
}

/// Write a 32-bit GPU register via MMIO
///
/// # Safety
/// `mmio_base` must be a valid mapped virtual address for GPU MMIO space
#[inline]
pub unsafe fn mmio_write32(mmio_base: u64, offset: u32, value: u32) {
    let addr = mmio_base + offset as u64;
    core::ptr::write_volatile(addr as *mut u32, value);
}

/// Read a 32-bit GPU register using indirect (indexed) access
/// This uses MMIO index/data registers for accessing registers beyond direct MMIO window
///
/// # Safety
/// `mmio_base` must be a valid mapped virtual address
#[inline]
pub unsafe fn mmio_read_indirect(mmio_base: u64, reg: u32) -> u32 {
    // Write register address to index port
    mmio_write32(mmio_base, regs::MM_INDEX, reg);
    // Read data from data port
    mmio_read32(mmio_base, regs::MM_DATA)
}

/// Write a 32-bit GPU register using indirect (indexed) access
///
/// # Safety
/// `mmio_base` must be a valid mapped virtual address
#[inline]
pub unsafe fn mmio_write_indirect(mmio_base: u64, reg: u32, value: u32) {
    mmio_write32(mmio_base, regs::MM_INDEX, reg);
    mmio_write32(mmio_base, regs::MM_DATA, value);
}

// ═══════════════════════════════════════════════════════════════════════════════
// BAR Size Detection
// ═══════════════════════════════════════════════════════════════════════════════

/// Detect the size of a PCI BAR by writing all-ones and reading back
/// This is the standard PCI BAR sizing mechanism.
fn detect_bar_size(dev: &PciDevice, bar_index: usize) -> u64 {
    let bar_offset = 0x10 + (bar_index as u8 * 4);
    
    // Save original BAR value
    let original = pci::config_read(dev.bus, dev.device, dev.function, bar_offset);
    
    // Write all ones
    pci::config_write(dev.bus, dev.device, dev.function, bar_offset, 0xFFFFFFFF);
    
    // Read back
    let readback = pci::config_read(dev.bus, dev.device, dev.function, bar_offset);
    
    // Restore original
    pci::config_write(dev.bus, dev.device, dev.function, bar_offset, original);
    
    if readback == 0 {
        return 0;
    }
    
    // Check memory vs I/O
    if original & 1 == 0 {
        // Memory BAR — mask lower 4 bits
        let mask = readback & 0xFFFFFFF0;
        if mask == 0 {
            return 0;
        }
        let size = (!mask).wrapping_add(1) as u64;
        
        // Check for 64-bit BAR
        let bar_type = (original >> 1) & 0x3;
        if bar_type == 2 && bar_index < 5 {
            // 64-bit BAR: also check upper 32 bits
            let bar_hi_offset = bar_offset + 4;
            let original_hi = pci::config_read(dev.bus, dev.device, dev.function, bar_hi_offset);
            pci::config_write(dev.bus, dev.device, dev.function, bar_hi_offset, 0xFFFFFFFF);
            let readback_hi = pci::config_read(dev.bus, dev.device, dev.function, bar_hi_offset);
            pci::config_write(dev.bus, dev.device, dev.function, bar_hi_offset, original_hi);
            
            let full_mask = ((readback_hi as u64) << 32) | (mask as u64);
            if full_mask == 0 {
                return size;
            }
            (!full_mask).wrapping_add(1)
        } else {
            size
        }
    } else {
        // I/O BAR — mask lower 2 bits
        let mask = readback & 0xFFFFFFFC;
        if mask == 0 {
            return 0;
        }
        ((!mask).wrapping_add(1) & 0xFFFF) as u64
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// PCIe Link Info
// ═══════════════════════════════════════════════════════════════════════════════

/// Read PCIe link speed and width from PCI Express capability
fn read_pcie_link_info(dev: &PciDevice) -> (u8, u8) {
    if let Some(pcie_cap_offset) = pci::find_capability(dev, pcie_cap::PCI_EXPRESS) {
        // Link Status register is at PCI Express cap + 0x12
        let link_status = pci::config_read16(dev.bus, dev.device, dev.function, pcie_cap_offset + 0x12);
        let speed = (link_status & 0xF) as u8;        // bits 3:0 = current link speed
        let width = ((link_status >> 4) & 0x3F) as u8; // bits 9:4 = negotiated width
        (speed, width)
    } else {
        (0, 0)
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// GPU Register Reading
// ═══════════════════════════════════════════════════════════════════════════════

/// Read GPU identity registers from MMIO
/// This reads chip family, revision, and basic capabilities
fn read_gpu_identity(mmio_base: u64) -> (u32, u32, u32, u32, u32) {
    let family;
    let external_rev;
    let compute_units;
    let gpu_clock;
    let vram_width;

    unsafe {
        // Try direct register access first — these are Navi10 / GFX10 register offsets
        // If these read as 0xFFFFFFFF or 0, the GPU might need initialization first
        
        let gc_info = mmio_read32(mmio_base, regs::GC_VERSION);
        crate::serial_println!("[AMDGPU] GC_VERSION raw: {:#010X}", gc_info);
        
        // Read chip identity from BIF/NBIO (should work even pre-init)
        let rlc_gpu_id = mmio_read32(mmio_base, regs::RLC_PG_CNTL);
        crate::serial_println!("[AMDGPU] RLC_PG_CNTL raw: {:#010X}", rlc_gpu_id);
        
        // Read the GC (Graphics Core) identification
        let gc_cac = mmio_read32(mmio_base, regs::GC_CAC_WEIGHT_CU_0);
        crate::serial_println!("[AMDGPU] GC_CAC raw: {:#010X}", gc_cac);
        
        // Try to read golden register — should give us CU count
        let cc_gc_shader_array = mmio_read32(mmio_base, regs::CC_GC_SHADER_ARRAY_CONFIG);
        crate::serial_println!("[AMDGPU] CC_GC_SHADER_ARRAY_CONFIG: {:#010X}", cc_gc_shader_array);
        
        // Read GC_USER_SHADER_ARRAY_CONFIG for active shader engines
        let gc_user_sa = mmio_read32(mmio_base, regs::GC_USER_SHADER_ARRAY_CONFIG);
        crate::serial_println!("[AMDGPU] GC_USER_SHADER_ARRAY_CONFIG: {:#010X}", gc_user_sa);
        
        // Try GRBM status to see if GFX engine is alive
        let grbm_status = mmio_read32(mmio_base, regs::GRBM_STATUS);
        crate::serial_println!("[AMDGPU] GRBM_STATUS: {:#010X}", grbm_status);
        
        // Try reading SMC/MP1 firmware version (may indicate if firmware is loaded)
        let smc_ver = mmio_read32(mmio_base, regs::MP1_SMN_C2PMSG_58);
        crate::serial_println!("[AMDGPU] MP1 firmware version: {:#010X}", smc_ver);
        
        // NBIO/BIF scratch registers — often contain VBIOS-programmed info
        let scratch0 = mmio_read32(mmio_base, regs::SCRATCH_REG0);
        let scratch1 = mmio_read32(mmio_base, regs::SCRATCH_REG1);
        let scratch7 = mmio_read32(mmio_base, regs::SCRATCH_REG7);
        crate::serial_println!("[AMDGPU] SCRATCH[0]={:#010X} [1]={:#010X} [7]={:#010X}", 
            scratch0, scratch1, scratch7);
        
        // Read memory controller config for VRAM width
        let mc_arb = mmio_read32(mmio_base, regs::MC_ARB_RAMCFG);
        crate::serial_println!("[AMDGPU] MC_ARB_RAMCFG: {:#010X}", mc_arb);
        vram_width = ((mc_arb >> 8) & 0xFF) as u32;
        
        // Determine family from GC version or PCI info
        // GC version format: major.minor.stepping (packed)
        let gc_major = (gc_info >> 16) & 0xFF;
        let gc_minor = (gc_info >> 8) & 0xFF;
        
        family = if gc_major == 10 && gc_minor == 1 {
            family::FAMILY_NV // Navi 10 confirmed via GC version
        } else if gc_major == 10 && gc_minor == 3 {
            family::FAMILY_NV14 // Navi 14
        } else {
            family::FAMILY_UNKNOWN
        };
        
        external_rev = gc_info; // Store full GC version as external rev
        
        // CU count from shader array config (if available)
        // Each bit cleared = one CU active (inverted mask)
        let cu_disable_mask = cc_gc_shader_array & 0xFFFF;
        compute_units = if cu_disable_mask != 0xFFFF && cu_disable_mask != 0xFFFFFFFF {
            // Navi 10: 2 shader engines, 2 SAs per SE, up to 10 CUs per SA = 40 max
            let disabled = cu_disable_mask.count_ones();
            40u32.saturating_sub(disabled)
        } else {
            0 // Can't determine
        };

        // GPU clock — try SMU mailbox or golden PLL register
        let sclk = mmio_read32(mmio_base, regs::CG_CLKPIN_CNTL_2);
        crate::serial_println!("[AMDGPU] CG_CLKPIN_CNTL_2: {:#010X}", sclk);
        gpu_clock = if sclk != 0 && sclk != 0xFFFFFFFF {
            sclk & 0xFFFF // Lower bits often contain clock in MHz
        } else {
            0
        };
    }

    (family, external_rev, compute_units, gpu_clock, vram_width)
}

/// Try to determine VRAM size from memory controller registers
fn read_vram_size(mmio_base: u64) -> u64 {
    unsafe {
        // Method 1: Try VRAM_INFO registers (Navi 10)
        let mc_vm_fb_location = mmio_read32(mmio_base, regs::MC_VM_FB_LOCATION_BASE);
        let mc_vm_fb_top = mmio_read32(mmio_base, regs::MC_VM_FB_LOCATION_TOP);
        crate::serial_println!("[AMDGPU] MC_VM_FB_LOCATION BASE={:#010X} TOP={:#010X}", 
            mc_vm_fb_location, mc_vm_fb_top);
        
        if mc_vm_fb_top > mc_vm_fb_location && mc_vm_fb_location != 0xFFFFFFFF {
            // These are in 1MB units (shifted by 24 bits in the register)
            let base_mb = (mc_vm_fb_location & 0xFFFF) as u64;
            let top_mb = (mc_vm_fb_top & 0xFFFF) as u64;
            let size = (top_mb - base_mb + 1) * 1024 * 1024;
            if size > 0 && size <= 32 * 1024 * 1024 * 1024 {
                return size;
            }
        }
        
        // Method 2: Try reading CONFIG_MEMSIZE (older method, sometimes works)
        let config_memsize = mmio_read32(mmio_base, regs::CONFIG_MEMSIZE);
        crate::serial_println!("[AMDGPU] CONFIG_MEMSIZE: {:#010X}", config_memsize);
        if config_memsize != 0 && config_memsize != 0xFFFFFFFF {
            return (config_memsize as u64) * 1024 * 1024; // In MB if small, or bytes if large
        }
        
        // Method 3: Use BAR2 aperture size as approximation
        0
    }
}

/// Determine VRAM type string from MC registers
fn vram_type_string(mmio_base: u64) -> &'static str {
    unsafe {
        let mc_seq = mmio_read32(mmio_base, regs::MC_SEQ_MISC0);
        crate::serial_println!("[AMDGPU] MC_SEQ_MISC0: {:#010X}", mc_seq);
        
        // Bits 31:28 encode memory type on some chips
        match (mc_seq >> 28) & 0xF {
            1 => "DDR2",
            2 => "DDR3",
            3 => "GDDR3",
            4 => "GDDR4",
            5 => "GDDR5",
            6 => "HBM",
            7 => "HBM2",
            8 | 9 => "GDDR6",
            _ => {
                // Navi 10 uses GDDR6, this is known from the hardware
                "GDDR6 (assumed)"
            }
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Main Driver Interface
// ═══════════════════════════════════════════════════════════════════════════════

/// Probe the PCI bus for AMD GPUs
/// Returns the first AMD display device found, if any
pub fn probe() -> Option<PciDevice> {
    // First: try known Navi 10 device IDs
    for &dev_id in NAVI10_DEVICE_IDS {
        if let Some(dev) = pci::find_by_id(AMD_VENDOR_ID, dev_id) {
            crate::log!("[AMDGPU] Found Navi GPU: {:04X}:{:04X} at {:02X}:{:02X}.{}", 
                dev.vendor_id, dev.device_id, dev.bus, dev.device, dev.function);
            return Some(dev);
        }
    }
    
    // Fallback: find any AMD display controller
    let display_devs = pci::find_by_class(pci::class::DISPLAY);
    for dev in display_devs {
        if dev.vendor_id == AMD_VENDOR_ID {
            crate::log!("[AMDGPU] Found AMD Display: {:04X}:{:04X} at {:02X}:{:02X}.{}", 
                dev.vendor_id, dev.device_id, dev.bus, dev.device, dev.function);
            return Some(dev);
        }
    }
    
    None
}

/// Initialize the AMD GPU driver
/// 
/// This performs:
/// 1. PCI device discovery
/// 2. Enable PCI bus mastering + memory space
/// 3. BAR mapping (MMIO registers + VRAM aperture)
/// 4. GPU identity register reads
/// 5. Report detected hardware info
pub fn init() {
    crate::log!("[AMDGPU] ═══════════════════════════════════════════════════");
    crate::log!("[AMDGPU] AMD GPU Driver — Phase 1: PCIe Discovery & MMIO");
    crate::log!("[AMDGPU] ═══════════════════════════════════════════════════");
    
    // Step 1: Find AMD GPU on PCI bus
    let dev = match probe() {
        Some(d) => d,
        None => {
            crate::log!("[AMDGPU] No AMD GPU detected on PCI bus");
            crate::log!("[AMDGPU] (This is normal in VMs without GPU passthrough)");
            // List all display controllers found for diagnostic
            let display_devs = pci::find_by_class(pci::class::DISPLAY);
            if display_devs.is_empty() {
                crate::log!("[AMDGPU] No display controllers found at all");
            } else {
                for d in &display_devs {
                    crate::log!("[AMDGPU] Display: {:04X}:{:04X} {} at {:02X}:{:02X}.{}", 
                        d.vendor_id, d.device_id, d.vendor_name(),
                        d.bus, d.device, d.function);
                }
            }
            return;
        }
    };
    
    // Step 2: Enable PCI bus mastering and memory space access
    crate::log!("[AMDGPU] Enabling PCI bus mastering and memory space...");
    pci::enable_bus_master(&dev);
    pci::enable_memory_space(&dev);
    
    // Read and display PCI command/status
    let cmd = pci::config_read16(dev.bus, dev.device, dev.function, 0x04);
    let status = pci::config_read16(dev.bus, dev.device, dev.function, 0x06);
    crate::log!("[AMDGPU] PCI Command: {:#06X}  Status: {:#06X}", cmd, status);
    
    // Step 3: Detect BARs
    crate::log!("[AMDGPU] Detecting BARs...");
    
    let mmio_phys = dev.bar_address(bar::MMIO_REGISTERS).unwrap_or(0);
    let mmio_size = detect_bar_size(&dev, bar::MMIO_REGISTERS);
    
    let vram_phys = dev.bar_address(bar::VRAM_APERTURE).unwrap_or(0);
    let vram_ap_size = detect_bar_size(&dev, bar::VRAM_APERTURE);
    
    let doorbell_phys = dev.bar_address(bar::DOORBELL).unwrap_or(0);
    let doorbell_size = detect_bar_size(&dev, bar::DOORBELL);
    
    crate::log!("[AMDGPU] BAR0 (MMIO):     phys={:#014X} size={:#X} ({} KB)", 
        mmio_phys, mmio_size, mmio_size / 1024);
    crate::log!("[AMDGPU] BAR2 (VRAM):     phys={:#014X} size={:#X} ({} MB)", 
        vram_phys, vram_ap_size, vram_ap_size / (1024 * 1024));
    crate::log!("[AMDGPU] BAR4 (Doorbell): phys={:#014X} size={:#X} ({} KB)", 
        doorbell_phys, doorbell_size, doorbell_size / 1024);
    
    if mmio_phys == 0 {
        crate::log!("[AMDGPU] ERROR: BAR0 (MMIO registers) not available!");
        return;
    }
    
    // Step 4: Map MMIO BAR into kernel virtual address space
    let map_size = if mmio_size > 0 { mmio_size as usize } else { 512 * 1024 }; // Default 512KB
    crate::log!("[AMDGPU] Mapping MMIO: {:#X} -> {} pages...", mmio_phys, map_size / 4096);
    
    let mmio_virt = match memory::map_mmio(mmio_phys, map_size) {
        Ok(virt) => {
            crate::log!("[AMDGPU] MMIO mapped at virtual {:#014X}", virt);
            virt
        }
        Err(e) => {
            crate::log!("[AMDGPU] ERROR: Failed to map MMIO: {}", e);
            return;
        }
    };
    
    // Step 5: Read PCIe link info
    let (link_speed, link_width) = read_pcie_link_info(&dev);
    let has_msi = pci::find_capability(&dev, pcie_cap::MSI).is_some();
    let has_msix = pci::find_capability(&dev, pcie_cap::MSIX).is_some();
    
    crate::log!("[AMDGPU] PCIe: Gen{} x{}  MSI:{}  MSI-X:{}", 
        link_speed, link_width,
        if has_msi { "yes" } else { "no" },
        if has_msix { "yes" } else { "no" });
    
    // Step 6: Read GPU identity registers
    crate::log!("[AMDGPU] Reading GPU identity registers...");
    let (asic_family, chip_ext_rev, compute_units, gpu_clock, _vram_width) = 
        read_gpu_identity(mmio_virt);
    
    let family_name = match asic_family {
        family::FAMILY_NV => "Navi 10 (RDNA 1)",
        family::FAMILY_NV14 => "Navi 14 (RDNA 1)",
        family::FAMILY_AI => "Vega 10 (GCN 5)",
        family::FAMILY_RV => "Raven (Vega APU)",
        _ => "Unknown",
    };
    
    // Step 7: Read VRAM size
    let vram_size = read_vram_size(mmio_virt);
    let vram_type = vram_type_string(mmio_virt);
    
    // Build GPU info struct  
    let info = GpuInfo {
        vendor_id: dev.vendor_id,
        device_id: dev.device_id,
        bus: dev.bus,
        device: dev.device,
        function: dev.function,
        revision: dev.revision,
        asic_family,
        chip_external_rev: chip_ext_rev,
        family_name,
        vram_size,
        vram_type,
        mmio_base_phys: mmio_phys,
        mmio_base_virt: mmio_virt,
        mmio_size: mmio_size,
        vram_aperture_phys: vram_phys,
        vram_aperture_size: vram_ap_size,
        pcie_link_speed: link_speed,
        pcie_link_width: link_width,
        has_msi,
        has_msix,
        compute_units,
        gpu_clock_mhz: gpu_clock,
    };
    
    // Step 8: Print summary
    crate::log!("[AMDGPU] ───────────────────────────────────────────────────");
    crate::log!("[AMDGPU] GPU: {}", info.gpu_name());
    crate::log!("[AMDGPU] PCI: {:04X}:{:04X} rev {:02X} at {:02X}:{:02X}.{}", 
        info.vendor_id, info.device_id, info.revision,
        info.bus, info.device, info.function);
    crate::log!("[AMDGPU] Family: {}", family_name);
    crate::log!("[AMDGPU] VRAM: {} ({})", info.vram_string(), vram_type);
    crate::log!("[AMDGPU] {}", info.pcie_link_string());
    if compute_units > 0 {
        crate::log!("[AMDGPU] Compute Units: {}", compute_units);
    }
    if gpu_clock > 0 {
        crate::log!("[AMDGPU] GPU Clock: {} MHz", gpu_clock);
    }
    crate::log!("[AMDGPU] MMIO: {:#X} ({} KB mapped)", mmio_virt, map_size / 1024);
    crate::log!("[AMDGPU] ───────────────────────────────────────────────────");
    crate::log!("[AMDGPU] Phase 1 complete — GPU discovered and identified");
    
    // Store state
    let mut state = GPU_STATE.lock();
    state.initialized = true;
    state.gpu_info = Some(info);
    GPU_DETECTED.store(true, Ordering::SeqCst);
    drop(state);
    
    // Phase 2: Initialize DCN display engine
    dcn::init(mmio_virt);
}

// ═══════════════════════════════════════════════════════════════════════════════
// Public API
// ═══════════════════════════════════════════════════════════════════════════════

/// Check if an AMD GPU was detected
pub fn is_detected() -> bool {
    GPU_DETECTED.load(Ordering::Relaxed)
}

/// Get GPU info (if detected)
pub fn get_info() -> Option<GpuInfo> {
    GPU_STATE.lock().gpu_info.clone()
}

/// Get a summary string for display in terminal/desktop
pub fn summary() -> String {
    if let Some(info) = get_info() {
        format!("{} | {} {} | {} | CU:{}", 
            info.gpu_name(),
            info.vram_string(),
            info.vram_type,
            info.pcie_link_string(),
            if info.compute_units > 0 { 
                format!("{}", info.compute_units) 
            } else { 
                String::from("?") 
            })
    } else {
        String::from("No AMD GPU detected")
    }
}

/// Get detailed info lines for terminal display
pub fn info_lines() -> Vec<String> {
    let mut lines = Vec::new();
    
    if let Some(info) = get_info() {
        lines.push(format!("╔══════════════════════════════════════════════╗"));
        lines.push(format!("║        AMD GPU — {}        ║", info.gpu_name()));
        lines.push(format!("╠══════════════════════════════════════════════╣"));
        lines.push(format!("║ PCI ID:    {:04X}:{:04X} rev {:02X}                 ║", 
            info.vendor_id, info.device_id, info.revision));
        lines.push(format!("║ Location:  {:02X}:{:02X}.{}                          ║",
            info.bus, info.device, info.function));
        lines.push(format!("║ Family:    {}               ║", info.family_name));
        lines.push(format!("║ VRAM:      {} ({})            ║", info.vram_string(), info.vram_type));
        lines.push(format!("║ PCIe:      {}              ║", info.pcie_link_string()));
        if info.compute_units > 0 {
            lines.push(format!("║ CUs:       {}                               ║", info.compute_units));
        }
        lines.push(format!("║ MMIO:      {:#X} ({} KB)        ║", 
            info.mmio_base_virt, info.mmio_size / 1024));
        lines.push(format!("║ VRAM Apt:  {:#X} ({} MB)     ║",
            info.vram_aperture_phys, info.vram_aperture_size / (1024 * 1024)));
        lines.push(format!("║ MSI: {}  MSI-X: {}                        ║",
            if info.has_msi { "Yes" } else { "No " },
            if info.has_msix { "Yes" } else { "No " }));
        lines.push(format!("╚══════════════════════════════════════════════╝"));
    } else {
        lines.push(String::from("No AMD GPU detected"));
        lines.push(String::from("(GPU passthrough or bare metal required)"));
    }
    
    lines
}
