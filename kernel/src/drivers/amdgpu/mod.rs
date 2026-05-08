//! AMD GPU Driver — Phase 1: PCIe Discovery & MMIO
//!
//! Native AMD Radeon GPU driver for TrustOS.
//! Targets AMD Navi 10 (RDNA 1) — RX 5600 XT / RX 5700 / RX 5700 XT.
//!
//! Phase 1: PCIe enumeration, BAR mapping, GPU identity register reads.
//!
//! Architecture:
//! - Discover AMD GPU via PCI bus scan (vendor 0x1002)
//! - Map MMIO BARs (BAR5 = registers, BAR0 = VRAM aperture)
//! - Read GPU identity registers (ASIC family, revision, VRAM info)
//! - Report hardware capabilities to the kernel
//!
//! References:
//! - AMD GPU register headers: https://github.com/torvalds/linux/tree/master/drivers/gpu/drm/amd
//! - AMDGPU kernel driver docs: https://docs.kernel.org/gpu/amdgpu/

pub mod regs;
pub mod dcn;
pub mod compute;
pub mod sdma;
pub mod smu;
pub mod neural;
pub mod firmware;
pub mod psp;
pub mod regscan;
pub mod atom;
#[cfg(feature = "jarvis")]
pub mod gpu_train;
pub mod shaders;
pub mod pipeline_audit;

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

/// Known Polaris 10/11 (GCN 4) device IDs — RX 470/480/570/580
pub const POLARIS_DEVICE_IDS: &[u16] = &[
    0x67DF, // Polaris 10 — RX 470/480/570/580
    0x67C4, // Polaris 10 (Pro WX 7100)
    0x67C7, // Polaris 10 (Pro WX 5100)
    0x67E0, // Polaris 10 (unused)
    0x67E3, // Polaris 10 (unused)
    0x67E8, // Polaris 10 (unused)
    0x67EB, // Polaris 10 (unused)
    0x67EF, // Polaris 10 — RX 460/560
    0x67FF, // Polaris 11 — RX 550/560
    0x6FDF, // Polaris 20 — RX 580 2048SP
];

/// GPU generation — determines register layout and init sequence
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GpuGen {
    /// Polaris / GCN 4 (RX 470/480/570/580) — SDMA v3.0, direct MMIO fw load
    Polaris,
    /// Navi 10 / RDNA 1 (RX 5600/5700) — SDMA v5.0, PSP fw load
    Navi10,
    /// Unknown AMD GPU
    Unknown,
}

/// Determine GPU generation from PCI device ID
pub fn gpu_gen_from_id(device_id: u16) -> GpuGen {
    if NAVI10_DEVICE_IDS.contains(&device_id) {
        GpuGen::Navi10
    } else if POLARIS_DEVICE_IDS.contains(&device_id) {
        GpuGen::Polaris
    } else {
        GpuGen::Unknown
    }
}

/// AMD GPU family identifiers from ASIC registers
pub mod family {
    pub const FAMILY_UNKNOWN:  u32 = 0;
    pub const FAMILY_NV:       u32 = 143; // Navi 10 / RDNA 1
    pub const FAMILY_NV14:     u32 = 144; // Navi 14 / RDNA 1 (cut-down)
    pub const FAMILY_AI:       u32 = 141; // Vega 10 / GCN 5
    pub const FAMILY_RV:       u32 = 142; // Raven Ridge APU
    pub const FAMILY_POLARIS:  u32 = 130; // Polaris 10/11 / GCN 4
}

/// PCI BAR indices for AMD GPUs (Navi 10 / GCN >= Bonaire layout)
/// Linux amdgpu_device.c: adev->rmmio_base = pci_resource_start(pdev, 5) for asic >= CHIP_BONAIRE
/// BAR0 = VRAM aperture (256 MB), BAR2 = Doorbell (2 MB), BAR5 = MMIO registers (~512 KB)
pub mod bar {
    /// BAR5: MMIO register space (32-bit non-prefetchable, ~512 KB at 0xF7C00000)
    pub const MMIO_REGISTERS: usize = 5;
    /// BAR0: VRAM aperture (64-bit prefetchable, 256 MB)
    pub const VRAM_APERTURE: usize = 0;
    /// BAR2: Doorbell space (64-bit prefetchable, 2 MB)
    pub const DOORBELL: usize = 2;
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
    /// GPU generation (Polaris / Navi10 / Unknown)
    pub gpu_gen: GpuGen,
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
    /// BAR5 MMIO base (physical)
    pub mmio_base_phys: u64,
    /// BAR5 MMIO base (virtual, mapped)
    pub mmio_base_virt: u64,
    /// BAR5 MMIO size
    pub mmio_size: u64,
    /// BAR0 VRAM aperture base (physical)
    pub vram_aperture_phys: u64,
    /// BAR0 VRAM aperture size
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
            0x67DF => match self.revision {
                0xE7 => "AMD Radeon RX 580X",
                0xC7 | 0xCF => "AMD Radeon RX 580",
                0xC4 | 0xC5 => "AMD Radeon RX 570",
                0xEF => "AMD Radeon RX 480",
                0xE3 | 0xE1 => "AMD Radeon RX 470",
                _ => "AMD Radeon Polaris 10",
            },
            0x67EF => "AMD Radeon RX 560",
            0x67FF => "AMD Radeon RX 550",
            0x6FDF => "AMD Radeon RX 580 2048SP",
            0x67C4 => "AMD Radeon Pro WX 7100",
            _ => match self.gpu_gen {
                GpuGen::Polaris => "AMD Radeon (Polaris)",
                GpuGen::Navi10 => "AMD Radeon (Navi)",
                GpuGen::Unknown => "AMD Radeon (Unknown)",
            },
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
    /// Detected GPU information (currently selected)
    gpu_info: Option<GpuInfo>,
    /// All detected AMD GPUs (populated by probe_all)
    all_gpus: Vec<GpuInfo>,
}

static GPU_STATE: Mutex<AmdGpuState> = Mutex::new(AmdGpuState {
    initialized: false,
    gpu_info: None,
    all_gpus: Vec::new(),
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
    let v = core::ptr::read_volatile(addr as *const u32);
    crate::mmio_trace!("R", offset, v);
    v
}

/// Write a 32-bit GPU register via MMIO
///
/// # Safety
/// `mmio_base` must be a valid mapped virtual address for GPU MMIO space
#[inline]
pub unsafe fn mmio_write32(mmio_base: u64, offset: u32, value: u32) {
    let addr = mmio_base + offset as u64;
    core::ptr::write_volatile(addr as *mut u32, value);
    crate::mmio_trace!("W", offset, value);
}

/// Read a 32-bit GPU register using indirect (indexed) access via PCIE_INDEX2/DATA2
/// On SOC15 (Navi 10+), PCIE_INDEX2/DATA2 is the correct path for indirect access.
/// The index register takes a BYTE OFFSET (e.g. 0x6A1B0 for MMHUB_FB_LOCATION_BASE).
///
/// # Safety
/// `mmio_base` must be a valid mapped virtual address
#[inline]
pub unsafe fn mmio_read_indirect(mmio_base: u64, reg: u32) -> u32 {
    // Write byte-offset to PCIE_INDEX2 port
    mmio_write32(mmio_base, regs::PCIE_INDEX2, reg);
    // Dummy readback to flush PCIe posted write (matches Linux amdgpu_device_indirect_rreg)
    let _ = mmio_read32(mmio_base, regs::PCIE_INDEX2);
    // Read data
    mmio_read32(mmio_base, regs::PCIE_DATA2)
}

/// Write a 32-bit GPU register using indirect (indexed) access via PCIE_INDEX2/DATA2
///
/// # Safety
/// `mmio_base` must be a valid mapped virtual address
#[inline]
pub unsafe fn mmio_write_indirect(mmio_base: u64, reg: u32, value: u32) {
    mmio_write32(mmio_base, regs::PCIE_INDEX2, reg);
    let _ = mmio_read32(mmio_base, regs::PCIE_INDEX2);
    mmio_write32(mmio_base, regs::PCIE_DATA2, value);
    let _ = mmio_read32(mmio_base, regs::PCIE_DATA2);
}

/// Read indirect via legacy MM_INDEX/MM_DATA
/// `reg` is a BYTE offset. MM_INDEX takes the byte offset directly.
/// MM_INDEX_HI carries bit 31 (for addresses >= 2GB).
#[inline]
pub unsafe fn mmio_read_indirect_legacy(mmio_base: u64, reg: u32) -> u32 {
    mmio_write32(mmio_base, regs::MM_INDEX, reg);
    let _ = mmio_read32(mmio_base, regs::MM_INDEX);
    mmio_write32(mmio_base, regs::MM_INDEX_HI, reg >> 31);
    let _ = mmio_read32(mmio_base, regs::MM_INDEX_HI);
    mmio_read32(mmio_base, regs::MM_DATA)
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
            
            // For 64-bit BARs: combine upper + lower masked bits
            // mask already has lower 4 bits cleared
            let mask_hi = readback_hi as u64;
            if mask_hi == 0xFFFFFFFF || mask_hi == 0 {
                // Upper bits all set or zero → size fits in 32 bits
                return size;
            }
            let full_mask = (mask_hi << 32) | (mask as u64);
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

/// Read PCIe Link Capabilities (max supported speed, max width).
/// Bits 3:0 = max link speed, bits 9:4 = max link width.
pub fn read_pcie_link_caps(dev: &PciDevice) -> (u8, u8) {
    if let Some(cap) = pci::find_capability(dev, pcie_cap::PCI_EXPRESS) {
        let lc = pci::config_read(dev.bus, dev.device, dev.function, cap + 0x0C);
        let max_speed = (lc & 0xF) as u8;
        let max_width = ((lc >> 4) & 0x3F) as u8;
        (max_speed, max_width)
    } else {
        (0, 0)
    }
}

/// Read PCIe Link Control 2 Target Link Speed (bits 3:0).
pub fn read_pcie_target_speed(dev: &PciDevice) -> u8 {
    if let Some(cap) = pci::find_capability(dev, pcie_cap::PCI_EXPRESS) {
        let lc2 = pci::config_read16(dev.bus, dev.device, dev.function, cap + 0x30);
        (lc2 & 0xF) as u8
    } else {
        0
    }
}

/// Find the parent PCI-to-PCI bridge (root port / switch downstream port)
/// whose secondary bus matches the given device's bus.
pub fn find_parent_bridge(dev: &PciDevice) -> Option<PciDevice> {
    let target_bus = dev.bus;
    for d in pci::get_devices() {
        if d.class_code == pci::class::BRIDGE && d.subclass == pci::bridge::PCI_TO_PCI {
            let sec_bus = pci::config_read8(d.bus, d.device, d.function, 0x19);
            if sec_bus == target_bus {
                return Some(d);
            }
        }
    }
    None
}

/// Force PCIe link to a target generation by retraining via the parent bridge.
/// `target_gen` must be 1, 2, or 3. Returns the (speed, width) read after retrain.
///
/// SAFETY: Mutates PCIe Link Control 2 / Link Control on the parent bridge and
/// triggers a link retrain. A failed retrain on the wrong port can hang the
/// system; callers should validate target speed against both endpoints' caps
/// (this function does so before issuing the retrain).
pub unsafe fn force_pcie_gen(dev: &PciDevice, target_gen: u8) -> Result<(u8, u8), &'static str> {
    if !(1..=3).contains(&target_gen) {
        return Err("target_gen must be 1, 2, or 3");
    }
    let (dev_max, _) = read_pcie_link_caps(dev);
    if dev_max == 0 {
        return Err("device has no PCIe capability");
    }
    if dev_max < target_gen {
        return Err("device does not support requested speed");
    }
    let bridge = find_parent_bridge(dev).ok_or("no parent bridge found")?;
    let (br_max, _) = read_pcie_link_caps(&bridge);
    if br_max == 0 {
        return Err("bridge has no PCIe capability");
    }
    if br_max < target_gen {
        return Err("bridge does not support requested speed");
    }
    let cap = pci::find_capability(&bridge, pcie_cap::PCI_EXPRESS)
        .ok_or("bridge PCIe cap not found")?;

    // 1) Set Target Link Speed in Link Control 2 (cap + 0x30, bits 3:0).
    let lc2_off = cap + 0x30;
    let mut lc2 = pci::config_read16(bridge.bus, bridge.device, bridge.function, lc2_off);
    lc2 = (lc2 & !0xF) | (target_gen as u16 & 0xF);
    pci::config_write16(bridge.bus, bridge.device, bridge.function, lc2_off, lc2);

    // 2) Trigger Retrain Link bit (Link Control cap + 0x10, bit 5).
    let lc_off = cap + 0x10;
    let lc = pci::config_read16(bridge.bus, bridge.device, bridge.function, lc_off);
    pci::config_write16(bridge.bus, bridge.device, bridge.function, lc_off, lc | (1 << 5));

    // 3) Poll Link Training bit (Link Status cap + 0x12, bit 11) until clear.
    let ls_off = cap + 0x12;
    let mut tries: u32 = 0;
    loop {
        let ls = pci::config_read16(bridge.bus, bridge.device, bridge.function, ls_off);
        if ls & (1 << 11) == 0 { break; }
        tries += 1;
        if tries > 1_000_000 {
            return Err("link retrain timeout");
        }
        core::hint::spin_loop();
    }

    // Brief settle delay before reading device-side link status.
    for _ in 0..200_000 { core::hint::spin_loop(); }

    Ok(read_pcie_link_info(dev))
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
            // GC_VERSION may read 0 if firmware not loaded — fallback to PCI device ID
            crate::serial_println!("[AMDGPU] GC_VERSION not valid (major={} minor={}), using PCI ID fallback", gc_major, gc_minor);
            family::FAMILY_UNKNOWN
        };
        
        external_rev = gc_info; // Store full GC version as external rev
        
        // CU count from shader array config (if available)
        // Only trust this register if GC_VERSION was valid (firmware loaded)
        if family != family::FAMILY_UNKNOWN {
            let cu_disable_mask = cc_gc_shader_array & 0xFFFF;
            compute_units = if cu_disable_mask != 0xFFFF && cu_disable_mask != 0xFFFFFFFF {
                // Navi 10: 2 shader engines, 2 SAs per SE, up to 10 CUs per SA = 40 max
                let disabled = cu_disable_mask.count_ones();
                40u32.saturating_sub(disabled)
            } else {
                0
            };
        } else {
            // No firmware — CU registers are unreliable, will use PCI ID fallback
            compute_units = 0;
        }

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
/// `firmware_loaded`: if false, MMIO registers are unreliable (skip reads)
fn read_vram_size(mmio_base: u64, firmware_loaded: bool) -> u64 {
    if !firmware_loaded {
        crate::serial_println!("[AMDGPU] Firmware not loaded — skipping MMIO VRAM register reads");
        return 0; // Caller should use PCI ID fallback
    }
    unsafe {
        // Method 1: Try VRAM_INFO registers (Navi 10)
        // MC_VM_FB_LOCATION_BASE/TOP: bits [23:0] represent 1MB-aligned addresses
        // Size = (top - base + 1) << 20  (each unit = 1 MB)
        let mc_vm_fb_location = mmio_read32(mmio_base, regs::MC_VM_FB_LOCATION_BASE);
        let mc_vm_fb_top = mmio_read32(mmio_base, regs::MC_VM_FB_LOCATION_TOP);
        crate::serial_println!("[AMDGPU] MC_VM_FB_LOCATION BASE={:#010X} TOP={:#010X}", 
            mc_vm_fb_location, mc_vm_fb_top);
        
        if mc_vm_fb_top > mc_vm_fb_location && mc_vm_fb_location != 0xFFFFFFFF {
            // Bits [23:0] are 1MB-granularity addresses
            let base = (mc_vm_fb_location & 0x00FFFFFF) as u64;
            let top = (mc_vm_fb_top & 0x00FFFFFF) as u64;
            let size = (top - base + 1) << 20; // Convert to bytes
            crate::serial_println!("[AMDGPU] VRAM from FB_LOCATION: base={} top={} size={} MB",
                base, top, size >> 20);
            if size > 0 && size <= 32 * 1024 * 1024 * 1024 {
                return size;
            }
        }
        
        // Method 2: Try reading CONFIG_MEMSIZE (often contains VRAM size in bytes directly)
        let config_memsize = mmio_read32(mmio_base, regs::CONFIG_MEMSIZE);
        crate::serial_println!("[AMDGPU] CONFIG_MEMSIZE: {:#010X}", config_memsize);
        if config_memsize != 0 && config_memsize != 0xFFFFFFFF {
            // CONFIG_MEMSIZE is in bytes on Navi (not MB)
            let size = config_memsize as u64;
            if size >= 64 * 1024 * 1024 && size <= 32 * 1024 * 1024 * 1024 {
                // Already in bytes (>= 64MB makes sense)
                crate::serial_println!("[AMDGPU] VRAM from CONFIG_MEMSIZE: {} MB (bytes)", size >> 20);
                return size;
            }
            // Maybe in MB for older chips
            let size_from_mb = size * 1024 * 1024;
            if size_from_mb > 0 && size_from_mb <= 32 * 1024 * 1024 * 1024 {
                crate::serial_println!("[AMDGPU] VRAM from CONFIG_MEMSIZE: {} MB (units)", size);
                return size_from_mb;
            }
        }
        
        // Method 3: Navi10 specific — try reading RLC_GPU_CLOCK_COUNT area for VRAM info
        // Fallback for known GPUs based on device ID
        crate::serial_println!("[AMDGPU] VRAM size: could not determine from registers");
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
// PCI Power Management
// ═══════════════════════════════════════════════════════════════════════════════

/// Ensure GPU is in PCI D0 (full power) state.
/// Linux PCI core does this before driver probe. We must do it manually.
fn ensure_pci_d0(dev: &PciDevice) {
    if let Some(pm_cap) = pci::find_capability(dev, pcie_cap::POWER_MGMT) {
        let pmcsr = pci::config_read16(dev.bus, dev.device, dev.function, pm_cap + 4);
        let current_state = pmcsr & 0x3;
        let state_name = match current_state {
            0 => "D0",
            1 => "D1",
            2 => "D2",
            3 => "D3hot",
            _ => "?",
        };
        crate::log!("[AMDGPU] PCI PM: cap@{:#X} PMCSR={:#06X} state={}", pm_cap, pmcsr, state_name);
        
        if current_state != 0 {
            // Transition to D0: clear PowerState bits (1:0), keep other bits
            let new_pmcsr = pmcsr & !0x3; // Set state to D0 (00)
            pci::config_write16(dev.bus, dev.device, dev.function, pm_cap + 4, new_pmcsr);
            
            // PCI spec: D3hot→D0 requires minimum 10ms recovery time
            // Spin-wait ~20ms (conservative)
            for _ in 0..20_000_000u32 {
                core::hint::spin_loop();
            }
            
            let verify = pci::config_read16(dev.bus, dev.device, dev.function, pm_cap + 4);
            crate::log!("[AMDGPU] PCI PM: Transitioned {} → D0 (PMCSR now {:#06X})", state_name, verify);
        }
    } else {
        crate::log!("[AMDGPU] PCI PM: No PM capability found");
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

    // Second: try Polaris device IDs
    for &dev_id in POLARIS_DEVICE_IDS {
        if let Some(dev) = pci::find_by_id(AMD_VENDOR_ID, dev_id) {
            crate::log!("[AMDGPU] Found Polaris GPU: {:04X}:{:04X} at {:02X}:{:02X}.{}", 
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

/// Probe PCI bus for ALL AMD GPUs (not just the first)
pub fn probe_all() -> Vec<PciDevice> {
    let mut gpus = Vec::new();
    let display_devs = pci::find_by_class(pci::class::DISPLAY);
    for dev in display_devs {
        if dev.vendor_id == AMD_VENDOR_ID {
            gpus.push(dev);
        }
    }
    gpus
}

/// Initialize a specific GPU by PCI bus number. Performs full BAR mapping + identity read.
/// Returns GpuInfo on success.
pub fn init_gpu(dev: &PciDevice) -> Option<GpuInfo> {
    pci::enable_bus_master(dev);
    pci::enable_memory_space(dev);
    ensure_pci_d0(dev);

    // Arm PCIe bus-lock auto-recovery: CTO on the GPU + AER fatal-severity
    // on GPU and its Root Port. On a fatal PCIe error, SERR# -> NMI ->
    // 0xCF9 reset (handled by the existing NMI handler). On a soft
    // completion timeout, callers using mmio_read32_safe() detect the
    // synthetic 0xFFFFFFFF and reboot.
    crate::pcie_recovery::init_for_gpu(dev);

    let mmio_phys = dev.bar_address(bar::MMIO_REGISTERS).unwrap_or(0);
    let mmio_size = detect_bar_size(dev, bar::MMIO_REGISTERS);
    let vram_phys = dev.bar_address(bar::VRAM_APERTURE).unwrap_or(0);
    let vram_ap_size = detect_bar_size(dev, bar::VRAM_APERTURE);

    if mmio_phys == 0 { return None; }

    let map_size = if mmio_size > 0 { mmio_size as usize } else { 512 * 1024 };
    let mmio_virt = match memory::map_mmio(mmio_phys, map_size) {
        Ok(v) => v,
        Err(_) => return None,
    };

    let gen = gpu_gen_from_id(dev.device_id);
    let (link_speed, link_width) = read_pcie_link_info(dev);
    let has_msi = pci::find_capability(dev, pcie_cap::MSI).is_some();
    let has_msix = pci::find_capability(dev, pcie_cap::MSIX).is_some();

    let vram_size = match dev.device_id {
        0x67DF => 8 * 1024 * 1024 * 1024u64,
        0x67EF => 4 * 1024 * 1024 * 1024u64,
        0x67FF => 2 * 1024 * 1024 * 1024u64,
        0x6FDF => 8 * 1024 * 1024 * 1024u64,
        _ => 0,
    };
    let compute_units = match dev.device_id {
        0x67DF => 36,
        0x67EF => 16,
        0x67FF => 8,
        0x6FDF => 32,
        _ => 0,
    };

    Some(GpuInfo {
        gpu_gen: gen,
        vendor_id: dev.vendor_id,
        device_id: dev.device_id,
        bus: dev.bus,
        device: dev.device,
        function: dev.function,
        revision: dev.revision,
        asic_family: if gen == GpuGen::Polaris { family::FAMILY_POLARIS } else { family::FAMILY_UNKNOWN },
        chip_external_rev: 0,
        family_name: if gen == GpuGen::Polaris { "Polaris 10 (GCN 4)" } else { "Unknown" },
        vram_size,
        vram_type: if gen == GpuGen::Polaris { "GDDR5" } else { "Unknown" },
        mmio_base_phys: mmio_phys,
        mmio_base_virt: mmio_virt,
        mmio_size,
        vram_aperture_phys: vram_phys,
        vram_aperture_size: vram_ap_size,
        pcie_link_speed: link_speed,
        pcie_link_width: link_width,
        has_msi,
        has_msix,
        compute_units,
        gpu_clock_mhz: 0,
    })
}

/// List all AMD GPUs found on PCI bus with basic info
pub fn list_all_gpus() -> Vec<GpuInfo> {
    let devs = probe_all();
    let mut result = Vec::new();
    for dev in &devs {
        if let Some(info) = init_gpu(dev) {
            result.push(info);
        }
    }
    result
}

/// Select a GPU by PCI bus number as the active GPU
pub fn select_gpu_by_bus(bus: u8) -> bool {
    let devs = probe_all();
    for dev in &devs {
        if dev.bus == bus {
            if let Some(info) = init_gpu(dev) {
                let mut state = GPU_STATE.lock();
                state.gpu_info = Some(info);
                state.initialized = true;
                GPU_DETECTED.store(true, Ordering::SeqCst);
                return true;
            }
        }
    }
    false
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
    crate::debug::checkpoint(crate::debug::POST_GPU_PCI, "GPU PCI discovery");
    crate::serial_println!("[AMDGPU] heap before init: free={} KB, used={} KB",
        crate::memory::heap::free() / 1024, crate::memory::heap::used() / 1024);
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
    
    // Step 2: Enable PCI bus mastering, memory space, and ensure D0 power state
    crate::log!("[AMDGPU] Enabling PCI bus mastering and memory space...");
    pci::enable_bus_master(&dev);
    pci::enable_memory_space(&dev);
    
    // Ensure GPU is in PCI D0 power state (not D3hot)
    ensure_pci_d0(&dev);
    
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
    
    crate::log!("[AMDGPU] BAR5 (MMIO):     phys={:#014X} size={:#X} ({} KB)", 
        mmio_phys, mmio_size, mmio_size / 1024);
    crate::log!("[AMDGPU] BAR0 (VRAM):     phys={:#014X} size={:#X} ({} MB)", 
        vram_phys, vram_ap_size, if vram_ap_size > 0 { vram_ap_size / (1024 * 1024) } else { 0 });
    crate::log!("[AMDGPU] BAR2 (Doorbell): phys={:#014X} size={:#X} ({} KB)", 
        doorbell_phys, doorbell_size, doorbell_size / 1024);
    
    if mmio_phys == 0 {
        crate::log!("[AMDGPU] ERROR: BAR5 (MMIO registers) not available!");
        return;
    }
    
    // Step 4: Map MMIO BAR into kernel virtual address space
    let map_size = if mmio_size > 0 { mmio_size as usize } else { 512 * 1024 }; // Default 512KB
    crate::log!("[AMDGPU] Mapping MMIO: {:#X} -> {} pages...", mmio_phys, map_size / 4096);
    
    let mmio_virt = match memory::map_mmio(mmio_phys, map_size) {
        Ok(virt) => {
            crate::debug::checkpoint(crate::debug::POST_GPU_MMIO, "GPU MMIO mapped");
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
    
    // Determine GPU generation FIRST from PCI ID (needed to select register set)
    let gen = gpu_gen_from_id(dev.device_id);
    crate::log!("[AMDGPU] GPU generation: {:?}", gen);

    // Step 6: Read GPU identity registers (only for Navi — Polaris has different offsets)
    let (asic_family_raw, chip_ext_rev, compute_units, gpu_clock, _vram_width) = match gen {
        GpuGen::Polaris => {
            crate::log!("[AMDGPU] Polaris detected — skipping Navi10 register reads");
            (family::FAMILY_UNKNOWN, 0, 0, 0, 0)
        }
        _ => {
            crate::log!("[AMDGPU] Reading GPU identity registers...");
            read_gpu_identity(mmio_virt)
        }
    };
    
    // firmware_loaded: GC_VERSION returned a valid family → firmware is running
    let firmware_loaded = asic_family_raw != family::FAMILY_UNKNOWN;

    // If family from registers is unknown, try PCI device ID fallback
    let asic_family = if asic_family_raw == family::FAMILY_UNKNOWN {
        match gen {
            GpuGen::Navi10 => match dev.device_id {
                0x7340 | 0x7341 | 0x7347 => {
                    crate::log!("[AMDGPU] Family from PCI ID {:04X} → Navi 14", dev.device_id);
                    family::FAMILY_NV14
                }
                _ => {
                    crate::log!("[AMDGPU] Family from PCI ID {:04X} → Navi 10", dev.device_id);
                    family::FAMILY_NV
                }
            },
            GpuGen::Polaris => {
                crate::log!("[AMDGPU] Family from PCI ID {:04X} → Polaris (GCN 4)", dev.device_id);
                family::FAMILY_POLARIS
            }
            GpuGen::Unknown => family::FAMILY_UNKNOWN,
        }
    } else {
        asic_family_raw
    };
    
    let family_name = match asic_family {
        family::FAMILY_NV => "Navi 10 (RDNA 1)",
        family::FAMILY_NV14 => "Navi 14 (RDNA 1)",
        family::FAMILY_AI => "Vega 10 (GCN 5)",
        family::FAMILY_RV => "Raven (Vega APU)",
        family::FAMILY_POLARIS => "Polaris 10 (GCN 4)",
        _ => "Unknown",
    };
    
    // Step 7: Read VRAM size (skip MMIO reads for Polaris — uses different regs)
    let vram_size = if gen == GpuGen::Polaris { 0 } else { read_vram_size(mmio_virt, firmware_loaded) };
    // Fallback: if registers don't report VRAM, use known sizes for common GPUs
    let vram_size = if vram_size == 0 {
        let fallback = match dev.device_id {
            0x731F => 8 * 1024 * 1024 * 1024u64,  // RX 5700 XT = 8 GB
            0x731E => 8 * 1024 * 1024 * 1024u64,  // RX 5700 = 8 GB
            0x7340 => 8 * 1024 * 1024 * 1024u64,  // RX 5500 XT = 8 GB
            0x7341 => 4 * 1024 * 1024 * 1024u64,  // RX 5500 = 4 GB
            0x67DF => 8 * 1024 * 1024 * 1024u64,  // RX 580 = 8 GB (4 GB variant exists)
            0x67EF => 4 * 1024 * 1024 * 1024u64,  // RX 560 = 4 GB
            0x67FF => 2 * 1024 * 1024 * 1024u64,  // RX 550 = 2 GB
            0x6FDF => 8 * 1024 * 1024 * 1024u64,  // RX 580 2048SP = 8 GB
            _ => 0,
        };
        if fallback > 0 {
            crate::log!("[AMDGPU] VRAM from PCI ID fallback: {} MB", fallback >> 20);
        }
        fallback
    } else {
        vram_size
    };
    let vram_type = if gen == GpuGen::Polaris { "GDDR5 (Polaris)" } else { vram_type_string(mmio_virt) };
    
    // CU count fallback by PCI device ID + revision (when firmware not loaded)
    let compute_units = if compute_units == 0 {
        let fallback = match (dev.device_id, dev.revision) {
            (0x731F, 0xC1) => 40,  // RX 5700 XT
            (0x731F, 0xC0 | 0xC4) => 36, // RX 5700
            (0x731F, 0xC2 | 0xC3) => 36, // RX 5600 XT
            (0x731E, _) => 36,     // RX 5700 (alt ID)
            (0x7340, _) => 22,     // RX 5500 XT
            (0x7341, _) => 22,     // RX 5500
            (0x67DF, _) => 36,     // RX 580 (Polaris 10)
            (0x67EF, _) => 16,     // RX 560
            (0x67FF, _) => 8,      // RX 550
            (0x6FDF, _) => 32,     // RX 580 2048SP
            _ => 0,
        };
        if fallback > 0 {
            crate::log!("[AMDGPU] CUs from PCI ID fallback: {}", fallback);
        }
        fallback
    } else {
        compute_units
    };
    
    // Build GPU info struct  
    let info = GpuInfo {
        gpu_gen: gen,
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
    // Inline VRAM formatting — no String allocation
    if info.vram_size >= 1024 * 1024 * 1024 {
        crate::log!("[AMDGPU] VRAM: {} GB ({})", info.vram_size / (1024 * 1024 * 1024), vram_type);
    } else if info.vram_size >= 1024 * 1024 {
        crate::log!("[AMDGPU] VRAM: {} MB ({})", info.vram_size / (1024 * 1024), vram_type);
    } else if info.vram_size > 0 {
        crate::log!("[AMDGPU] VRAM: {} KB ({})", info.vram_size / 1024, vram_type);
    } else {
        crate::log!("[AMDGPU] VRAM: Unknown ({})", vram_type);
    }
    // Inline PCIe formatting — no String allocation
    let pcie_speed_str = match info.pcie_link_speed {
        1 => "2.5 GT/s (Gen1)",
        2 => "5.0 GT/s (Gen2)",
        3 => "8.0 GT/s (Gen3)",
        4 => "16.0 GT/s (Gen4)",
        _ => "Unknown",
    };
    crate::log!("[AMDGPU] PCIe x{} {}", info.pcie_link_width, pcie_speed_str);
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
    
    match gen {
        GpuGen::Polaris => {
            // Polaris: direct MMIO firmware loading — no PSP needed
            crate::debug::checkpoint(crate::debug::POST_GPU_FW, "GPU Polaris FW init");
            crate::log!("[AMDGPU] Polaris path: direct SDMA firmware load (no PSP)");

            // Best-effort SMU bring-up + GFX power-up before firmware load.
            // On power-managed mining boards (BTC-250PRO etc.) VBIOS leaves GFX
            // CU power-gated, which prevents MEC1 from running its boot
            // trampoline. The SMU PowerUpGfx sequence wakes the CUs so that
            // subsequent CP/MEC firmware uploads can actually execute.
            // Failures here are non-fatal: legacy code paths still run.
            // Auto-start is disabled so the VBIOS-pre-init SMU state can be
            // observed via `gpu smu diag` and started manually with `gpu smu start`.
            // Set TRUSTOS_AMDGPU_SMU_AUTO_START to true to restore auto-start.
            const TRUSTOS_AMDGPU_SMU_AUTO_START: bool = false;
            if TRUSTOS_AMDGPU_SMU_AUTO_START {
            if let Some(probe_info) = get_info() {
                unsafe {
                    match smu::smu7_start_smu(mmio_virt) {
                        Ok(()) => crate::log!("[AMDGPU] SMU started"),
                        Err(e) => {
                            crate::log!("[AMDGPU] SMU start failed: {} — issuing PCI config reset and retrying", e);
                            smu::smu7_pci_config_reset(
                                probe_info.bus,
                                probe_info.device,
                                probe_info.function,
                            );
                            match smu::smu7_start_smu(mmio_virt) {
                                Ok(()) => crate::log!("[AMDGPU] SMU started after PCI reset"),
                                Err(e2) => crate::log!("[AMDGPU] SMU start still failed after reset: {}", e2),
                            }
                        }
                    }
                    match smu::smu7_powerup_gfx(&probe_info) {
                        Ok(rep) => crate::log!(
                            "[AMDGPU] PowerUpGfx: MEC1_PC 0x{:08X} -> 0x{:08X}",
                            rep.mec1_pc_pre, rep.mec1_pc_post
                        ),
                        Err(e) => crate::log!("[AMDGPU] PowerUpGfx skipped: {}", e),
                    }
                }
            }
            } else {
                crate::log!("[AMDGPU] SMU auto-start DISABLED — use `gpu smu start` to bring up");
            }

            firmware::init_polaris(mmio_virt, vram_phys, vram_ap_size);
        }
        _ => {
            // Navi 10: full firmware init via PSP
            crate::debug::checkpoint(crate::debug::POST_GPU_PSP, "GPU PSP init");
            firmware::init(mmio_virt, vram_phys, vram_ap_size);
            crate::debug::checkpoint(crate::debug::POST_GPU_SDMA, "GPU SDMA init");
            dcn::init(mmio_virt);
            sdma::init(mmio_virt);
            compute::init(mmio_virt);
        }
    }
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

/// Dump raw VRAM-related MMIO registers for debugging
/// Uses correct Navi 10 SOC15 offsets (MMHUB 2.0, NBIO 2.3, GC 10.1)
pub fn dump_vram_regs() -> Vec<String> {
    let mut lines = Vec::new();
    if let Some(info) = get_info() {
        let mmio = info.mmio_base_virt;
        let (bus, dev, func) = (info.bus, info.device, info.function);
        // Reconstruct a PciDevice for capability queries
        let pci_dev = PciDevice {
            bus, device: dev, function: func,
            vendor_id: info.vendor_id, device_id: info.device_id,
            class_code: 3, subclass: 0, prog_if: 0,
            revision: info.revision, header_type: 0,
            bar: [0; 6], interrupt_line: 0, interrupt_pin: 0,
        };
        unsafe {
            lines.push(format!("=== AMD GPU Deep Diagnostic (Navi 10 SOC15) ==="));
            lines.push(format!("MMIO virt={:#018X} phys={:#014X} size={}KB",
                mmio, info.mmio_base_phys, info.mmio_size / 1024));

            // ── 1. PCI Config Space ──
            let pci_cmd = pci::config_read16(bus, dev, func, 0x04);
            let pci_status = pci::config_read16(bus, dev, func, 0x06);
            lines.push(format!("PCI CMD={:#06X} STS={:#06X} MemEn={} BusMst={}",
                pci_cmd, pci_status, (pci_cmd >> 1) & 1, (pci_cmd >> 2) & 1));

            // PCI Power Management state
            if let Some(pm_cap) = pci::find_capability(&pci_dev, pcie_cap::POWER_MGMT) {
                let pmcsr = pci::config_read16(bus, dev, func, pm_cap + 4);
                let pmc = pci::config_read16(bus, dev, func, pm_cap + 2);
                lines.push(format!("PCI PM: cap@{:#X} PMC={:#06X} PMCSR={:#06X} state=D{}",
                    pm_cap, pmc, pmcsr, pmcsr & 3));
            }

            // PCIe link status
            if let Some(pcie_cap) = pci::find_capability(&pci_dev, pcie_cap::PCI_EXPRESS) {
                let link_status = pci::config_read16(bus, dev, func, pcie_cap + 0x12);
                let link_cap = pci::config_read(bus, dev, func, pcie_cap + 0x0C);
                let speed = link_status & 0xF;
                let width = (link_status >> 4) & 0x3F;
                let max_speed = link_cap & 0xF;
                let max_width = (link_cap >> 4) & 0x3F;
                lines.push(format!("PCIe Link: Gen{} x{} (max Gen{} x{})",
                    speed, width, max_speed, max_width));
            }

            // BARs from config space
            let bar0_lo = pci::config_read(bus, dev, func, 0x10);
            let bar0_hi = pci::config_read(bus, dev, func, 0x14);
            let bar5 = pci::config_read(bus, dev, func, 0x24);
            lines.push(format!("BAR0(VRAM)={:#010X}_{:08X} BAR5(MMIO)={:#010X}",
                bar0_hi, bar0_lo, bar5));

            // ── 2. Page Table Flags Check ──
            let cr3: u64;
            cr3 = crate::arch::read_page_table_root();
            let hhdm = crate::memory::hhdm_offset();
            let pml4 = &*((cr3 + hhdm) as *const [u64; 512]);
            let pml4_idx = ((mmio >> 39) & 0x1FF) as usize;
            let pdpt_idx = ((mmio >> 30) & 0x1FF) as usize;
            let pd_idx = ((mmio >> 21) & 0x1FF) as usize;
            let pt_idx = ((mmio >> 12) & 0x1FF) as usize;
            if pml4[pml4_idx] & 1 != 0 {
                let pdpt = &*(((pml4[pml4_idx] & 0x000F_FFFF_FFFF_F000) + hhdm) as *const [u64; 512]);
                if pdpt[pdpt_idx] & 1 != 0 && pdpt[pdpt_idx] & (1 << 7) == 0 {
                    let pd = &*(((pdpt[pdpt_idx] & 0x000F_FFFF_FFFF_F000) + hhdm) as *const [u64; 512]);
                    if pd[pd_idx] & 1 != 0 && pd[pd_idx] & (1 << 7) == 0 {
                        let pt = &*(((pd[pd_idx] & 0x000F_FFFF_FFFF_F000) + hhdm) as *const [u64; 512]);
                        let pte = pt[pt_idx];
                        let pcd = (pte >> 4) & 1;
                        let pwt = (pte >> 3) & 1;
                        let pat = (pte >> 7) & 1;
                        let nx = (pte >> 63) & 1;
                        lines.push(format!("PTE: {:#018X} PCD={} PWT={} PAT={} NX={} → {}",
                            pte, pcd, pwt, pat, nx,
                            match (pat, pcd, pwt) {
                                (0, 0, 0) => "WB",
                                (0, 0, 1) => "WC/WT",
                                (0, 1, 0) => "UC-",
                                (0, 1, 1) => "UC",
                                _ => "custom",
                            }));
                    }
                }
            }

            // ── 3. SCRATCH write/read test ──
            let scratch_orig = mmio_read32(mmio, regs::SCRATCH_REG0);
            mmio_write32(mmio, regs::SCRATCH_REG0, 0xDEAD_BEEF);
            let scratch_rb = mmio_read32(mmio, regs::SCRATCH_REG0);
            mmio_write32(mmio, regs::SCRATCH_REG0, scratch_orig);
            lines.push(format!("SCRATCH w/r: {:#010X} {}", scratch_rb,
                if scratch_rb == 0xDEAD_BEEF { "OK" } else { "FAIL" }));

            // ── 4. Index register readback verification ──
            mmio_write32(mmio, regs::PCIE_INDEX2, 0x12345678);
            core::sync::atomic::fence(core::sync::atomic::Ordering::SeqCst);
            let idx2_rb = mmio_read32(mmio, regs::PCIE_INDEX2);
            lines.push(format!("PCIE_IDX2 w/r: wrote 0x12345678 read {:#010X} {}",
                idx2_rb, if idx2_rb == 0x12345678 { "OK" } else { "FAIL" }));

            mmio_write32(mmio, regs::MM_INDEX, 0xABCD_0000);
            core::sync::atomic::fence(core::sync::atomic::Ordering::SeqCst);
            let mmidx_rb = mmio_read32(mmio, regs::MM_INDEX);
            lines.push(format!("MM_INDEX w/r: wrote 0xABCD0000 read {:#010X} {}",
                mmidx_rb, if mmidx_rb == 0xABCD_0000 { "OK" } else { "FAIL" }));

            // ── 5. Indirect vs Direct comparison ──
            let grbm = mmio_read32(mmio, regs::GRBM_STATUS);
            let grbm_pcie = mmio_read_indirect(mmio, regs::GRBM_STATUS);
            let grbm_mm = mmio_read_indirect_legacy(mmio, regs::GRBM_STATUS);
            lines.push(format!("GRBM dir={:#010X} pcie={:#010X} mm={:#010X} {}",
                grbm, grbm_pcie, grbm_mm,
                if grbm == grbm_pcie && grbm == grbm_mm { "ALL MATCH" }
                else if grbm == grbm_pcie { "pcie OK" }
                else if grbm == grbm_mm { "mm OK" }
                else { "ALL MISMATCH" }));

            // ── 6. Direct MMIO decode at various offsets ──
            let rcc_config = mmio_read32(mmio, regs::RCC_CONFIG_MEMSIZE);
            lines.push(format!("RCC_CONFIG @{:#X}: {:#010X}", regs::RCC_CONFIG_MEMSIZE, rcc_config));

            lines.push(format!("SCRATCH7: {:#010X}", mmio_read32(mmio, regs::SCRATCH_REG7)));

            // ── 7. MMHUB decode test ──
            let fb_base = mmio_read32(mmio, regs::MMHUB_FB_LOCATION_BASE);
            let fb_top = mmio_read32(mmio, regs::MMHUB_FB_LOCATION_TOP);
            lines.push(format!("FB_BASE @{:#X}: {:#010X}", regs::MMHUB_FB_LOCATION_BASE, fb_base));
            lines.push(format!("FB_TOP  @{:#X}: {:#010X}", regs::MMHUB_FB_LOCATION_TOP, fb_top));

            // ── 7b. PSP Sign of Life (MP0_SMN_C2PMSG_81) ──
            // MP0_BASE_SEG0=0x16000, mmMP0_SMN_C2PMSG_81=0x0091
            let psp_sol_offset: u32 = (0x16000 + 0x0091) * 4; // 0x58244
            let psp_sol = mmio_read32(mmio, psp_sol_offset);
            lines.push(format!("PSP_SOL @{:#X}: {:#010X} {}", psp_sol_offset, psp_sol,
                if psp_sol == 0xFFFF_FFFF { "DEAD" }
                else if psp_sol == 0x80000000 { "READY" }
                else if psp_sol != 0 { "ALIVE" }
                else { "ZERO" }));

            // ── 7c. PCI config space dump (first 64 bytes) ──
            lines.push(format!("PCI_CFG:"));
            for row in 0..4u8 {
                let o = row * 16;
                let d0 = pci::config_read(bus, dev, func, o);
                let d1 = pci::config_read(bus, dev, func, o + 4);
                let d2 = pci::config_read(bus, dev, func, o + 8);
                let d3 = pci::config_read(bus, dev, func, o + 12);
                lines.push(format!("  [{:02X}] {:08X} {:08X} {:08X} {:08X}", o, d0, d1, d2, d3));
            }

            // ── 8. RLC/CP status ──
            let rlc = mmio_read32(mmio, regs::RLC_CNTL);
            lines.push(format!("RLC_CNTL: {:#010X} EN_F32={}", rlc, (rlc >> 1) & 1));
            let cp = mmio_read32(mmio, regs::CP_ME_CNTL);
            lines.push(format!("CP_ME_CNTL: {:#010X} ME_H={} PFP_H={} CE_H={}",
                cp, (cp >> 28) & 1, (cp >> 26) & 1, (cp >> 24) & 1));

            // ── 9. SMU firmware version ──
            let smu_ver = mmio_read32(mmio, regs::MP1_SMN_C2PMSG_58);
            lines.push(format!("SMU_FW @{:#X}: {:#010X}", regs::MP1_SMN_C2PMSG_58, smu_ver));

            // ── 10. Expansion ROM BAR ──
            let exp_rom = pci::config_read(bus, dev, func, 0x30);
            lines.push(format!("EXP_ROM: {:#010X} en={}", exp_rom, exp_rom & 1));

            // ── 11. NBIO SEG2 scan (0x3480+) ──
            let seg2_base: u32 = 0x0D20 * 4;
            lines.push(format!("NBIO_SEG2 @{:#X}:", seg2_base));
            for i in 0..8u32 {
                let off = seg2_base + i * 4;
                let v = mmio_read32(mmio, off);
                lines.push(format!("  [{:#06X}]: {:#010X}", off, v));
            }

            // ── 12. HDP probe ──
            let hdp_base: u32 = 0x0D80 * 4;
            lines.push(format!("HDP @{:#X}: {:#010X}", hdp_base, mmio_read32(mmio, hdp_base)));
        }
    } else {
        lines.push(String::from("No AMD GPU detected"));
    }
    lines
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
