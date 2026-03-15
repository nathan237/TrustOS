//! AMD GPU Firmware Loader — Navi 10 (RDNA 1) bare-metal firmware bootstrap
//!
//! On modern AMD GPUs, the hardware engines (GFX, Compute, SDMA, Display) won't
//! execute commands until their microcontroller firmware is loaded. This module
//! handles loading firmware blobs into the appropriate GPU microcontrollers.
//!
//! # Firmware Components (Navi 10)
//!
//! | Engine   | Firmware  | Microcontroller | Purpose                          |
//! |----------|-----------|-----------------|----------------------------------|
//! | GFX      | PFP       | CP Front-end    | Command parsing / prefetch       |
//! | GFX      | ME        | CP Back-end     | State management / draw dispatch |
//! | GFX      | CE        | Constant Engine  | Pre-loading constants/shaders   |
//! | Compute  | MEC       | MEC 1 & 2       | Compute queue scheduling         |
//! | RLC      | RLC       | Run List Ctrl    | Power gating, context switch     |
//! | SDMA     | SDMA      | DMA Engine 0+1  | Async memory transfers           |
//! | SMU      | SMC       | SMU 11.0        | Clock/voltage/power management   |
//!
//! # Loading Process
//!
//! ```
//! 1. PSP (Platform Security Processor) boots from SPI flash — already done at power-on
//! 2. Load RLC firmware → enables power management + context switching
//! 3. Load ME/PFP/CE/MEC firmware → Command Processor can now parse PM4 packets
//! 4. Load SDMA firmware → SDMA engines can process ring buffer commands
//! 5. SMU firmware is typically loaded by VBIOS — we just verify it's running
//! ```
//!
//! # Firmware Source
//!
//! Firmware blobs are loaded from the ramfs filesystem at `/lib/firmware/amdgpu/`.
//! They can be provided via:
//! - Limine module_path (loaded at boot)
//! - Embedded in the kernel binary (for self-contained ISO)
//! - Uploaded via shell command (`fw_load`)
//!
//! The firmware files match the naming from linux-firmware:
//! - `navi10_pfp.bin`, `navi10_me.bin`, `navi10_ce.bin`
//! - `navi10_mec.bin`, `navi10_mec2.bin`
//! - `navi10_rlc.bin`
//! - `navi10_sdma.bin`, `navi10_sdma1.bin`
//! - `navi10_smc.bin`
//!
//! References:
//! - Linux: drivers/gpu/drm/amd/amdgpu/gfx_v10_0.c (gfx_v10_0_cp_gfx_load_microcode)
//! - Linux: drivers/gpu/drm/amd/amdgpu/sdma_v5_0.c (sdma_v5_0_load_microcode)
//! - Linux: drivers/gpu/drm/amd/amdgpu/gfx_v10_0.c (gfx_v10_0_rlc_load_microcode)
//! - AMD PSP bootstrap: drivers/gpu/drm/amd/amdgpu/psp_v11_0.c

use alloc::string::String;
use alloc::format;
use alloc::vec::Vec;
use core::sync::atomic::{AtomicBool, Ordering};
use spin::Mutex;

use super::{mmio_read32, mmio_write32, mmio_read_indirect, mmio_write_indirect};
use super::regs;
use crate::memory;

// ═══════════════════════════════════════════════════════════════════════════════
// Constants
// ═══════════════════════════════════════════════════════════════════════════════

/// Firmware directory in ramfs
const FW_DIRECTORY: &str = "/lib/firmware/amdgpu";

/// Expected firmware file names for Navi 10
const FW_PFP: &str = "navi10_pfp.bin";
// Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const FW_ME: &str = "navi10_me.bin";
// Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const FW_CE: &str = "navi10_ce.bin";
// Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const FW_MEC: &str = "navi10_mec.bin";
// Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const FW_MEC2: &str = "navi10_mec2.bin";
// Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const FW_RLC: &str = "navi10_rlc.bin";
// Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const FW_SDMA0: &str = "navi10_sdma.bin";
// Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const FW_SDMA1: &str = "navi10_sdma1.bin";

/// AMD firmware header magic
const AMD_FW_MAGIC: u32 = 0x4D_44_41; // "ADM" (little-endian check)

/// Common firmware header (from linux-firmware)
/// Many AMD firmware blobs use a standard header:
/// - DW0: magic / version
/// - DW1: header size in DWORDs
/// - DW2: ucode version
/// - DW3: ucode size in bytes
/// - DW4+: ucode data
const FW_HEADER_SIZE_DW: usize = 256; // Typical: 256 DWORDs = 1024 bytes header

/// Timeout for firmware load operations (polling iterations)
const FW_LOAD_TIMEOUT: u64 = 5_000_000;

// ═══════════════════════════════════════════════════════════════════════════════
// Register Definitions for Firmware Loading
// ═══════════════════════════════════════════════════════════════════════════════

// ── RLC (Run List Controller) firmware registers ────────────────────────────
/// RLC CGTT management control
const RLC_CGTT_MGCG_OVERRIDE: u32 = 0x4E08;
/// RLC clear state (context switch state buffer)
const RLC_GPM_UCODE_ADDRESS: u32 = 0x4E20;
/// RLC ucode data register — write firmware DWORDs here
const RLC_GPM_UCODE_DATA: u32 = 0x4E24;
/// RLC auto-load control (GFX10+)
const RLC_AUTOLOAD_CNTL: u32 = 0x4E28;
/// RLC auto-load status
const RLC_AUTOLOAD_STATUS: u32 = 0x4E2C;
/// RLC safe mode request
const RLC_SAFE_MODE: u32 = 0x4E0C;
/// RLC cntl — main enable/disable
const RLC_CNTL: u32 = 0x4E00;
/// RLC status
const RLC_STATUS: u32 = 0x4E04;
/// RLC firmware size register  
const RLC_GPM_UCODE_SIZE: u32 = 0x4E30;

// ── CP (Command Processor) GFX firmware registers ───────────────────────────
/// PFP (Pre-Fetch Parser) firmware address register
const CP_PFP_UCODE_ADDRESS: u32 = 0x8A14;
/// PFP firmware data register
const CP_PFP_UCODE_DATA: u32 = 0x8A18;
/// ME (Micro Engine) firmware address register
const CP_ME_UCODE_ADDRESS: u32 = 0x8A1C;
/// ME firmware data register
const CP_ME_UCODE_DATA: u32 = 0x8A20;
/// CE (Constant Engine) firmware address register
const CP_CE_UCODE_ADDRESS: u32 = 0x8A24;
/// CE firmware data register
const CP_CE_UCODE_DATA: u32 = 0x8A28;
/// ME master control — halt/resume ME, PFP, CE
const CP_ME_CNTL: u32 = 0x86D8;

// CP_ME_CNTL bits
const CP_ME_HALT: u32 = 1 << 28;
// Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const CP_PFP_HALT: u32 = 1 << 26;
// Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const CP_CE_HALT: u32 = 1 << 24;

// ── MEC (Micro Engine Compute) firmware registers ───────────────────────────
/// MEC1 firmware address
const CP_MEC_ME1_UCODE_ADDRESS: u32 = 0x8A30;
/// MEC1 firmware data
const CP_MEC_ME1_UCODE_DATA: u32 = 0x8A34;
/// MEC2 firmware address
const CP_MEC_ME2_UCODE_ADDRESS: u32 = 0x8A38;
/// MEC2 firmware data
const CP_MEC_ME2_UCODE_DATA: u32 = 0x8A3C;

// ── SDMA firmware registers ─────────────────────────────────────────────────
/// SDMA engine 0 ucode address
const SDMA0_UCODE_ADDRESS: u32 = 0x4D88;
/// SDMA engine 0 ucode data
const SDMA0_UCODE_DATA: u32 = 0x4D8C;
/// SDMA engine 1 ucode address
const SDMA1_UCODE_ADDRESS: u32 = 0x4E88;
/// SDMA engine 1 ucode data
const SDMA1_UCODE_DATA: u32 = 0x4E8C;

// ── GFX10 CP control registers ──────────────────────────────────────────────
/// CP GFX ring buffer control
const CP_RB0_CNTL: u32 = 0x8044;
/// GFX ring enable
const CP_RB_VMID_RESET: u32 = 0x8054;

// ═══════════════════════════════════════════════════════════════════════════════
// Firmware State
// ═══════════════════════════════════════════════════════════════════════════════

/// Individual firmware status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
// Énumération — un type qui peut être l'une de plusieurs variantes.
pub enum FwStatus {
    /// Firmware not available (file not found)
    NotFound,
    /// Firmware loaded but not verified
    Loaded,
    /// Firmware loaded and engine is running
    Running,
    /// Firmware load failed
    Failed,
}

/// Overview of all firmware states
pub struct FirmwareState {
    pub rlc: FwStatus,
    pub pfp: FwStatus,
    pub me: FwStatus,
    pub ce: FwStatus,
    pub mec1: FwStatus,
    pub mec2: FwStatus,
    pub sdma0: FwStatus,
    pub sdma1: FwStatus,
    pub mmio_base: u64,
}

// État global partagé protégé par un Mutex (verrou d'exclusion mutuelle).
static FW_STATE: Mutex<FirmwareState> = Mutex::new(FirmwareState {
    rlc: FwStatus::NotFound,
    pfp: FwStatus::NotFound,
    me: FwStatus::NotFound,
    ce: FwStatus::NotFound,
    mec1: FwStatus::NotFound,
    mec2: FwStatus::NotFound,
    sdma0: FwStatus::NotFound,
    sdma1: FwStatus::NotFound,
    mmio_base: 0,
});

// Variable atomique — accès thread-safe sans verrou.
static FW_LOADED: AtomicBool = AtomicBool::new(false);

// ═══════════════════════════════════════════════════════════════════════════════
// Firmware File I/O
// ═══════════════════════════════════════════════════════════════════════════════

/// Read a firmware file from ramfs, returning the raw bytes
fn read_fw_file(name: &str) -> Option<Vec<u8>> {
    let path = format!("{}/{}", FW_DIRECTORY, name);
    crate::ramfs::with_filesystem(|fs| {
        fs.read_file(&path).ok().map(|data| data.to_vec())
    })
}

/// Parse a firmware blob — skip the header if present, return ucode DWORDs
/// AMD firmware blobs have a variable-size header. The actual microcode
/// starts after the header.
fn parse_fw_blob(data: &[u8]) -> &[u8] {
    if data.len() < 16 {
        return data;
    }
    
    // Try to detect AMD firmware header format:
    // The first DWORD often has a header size field.
    // Common header: legacy = 256 bytes, new common header = varies
    let dw0 = u32::from_le_bytes([data[0], data[1], data[2], data[3]]);
    let dw1 = u32::from_le_bytes([data[4], data[5], data[6], data[7]]);
    
    // GFX10 firmware format (from amdgpu_ucode.h):
    // struct common_firmware_header {
    //   uint32_t size_bytes;    // total header size in bytes
    //   uint32_t header_size_dw; // header size in DWORDs
    //   uint16_t header_version_major;
    //   uint16_t header_version_minor;
    //   uint16_t ip_version_major;
    //   uint16_t ip_version_minor;
    //   uint32_t ucode_version;
    //   uint32_t ucode_size_bytes;
    //   uint32_t ucode_array_offset_bytes;
    //   ...
    // }
    
    // Check if dw1 looks like a valid header size (4-1024 DWORDs)
    if dw1 >= 4 && dw1 <= 1024 {
        let header_bytes = (dw1 as usize) * 4;
        if header_bytes < data.len() {
            // Also check for ucode_array_offset_bytes at offset 20
            if data.len() >= 24 {
                let ucode_offset = u32::from_le_bytes([
                    data[20], data[21], data[22], data[23]
                ]) as usize;
                if ucode_offset > 0 && ucode_offset < data.len() {
                    crate::serial_println!("[AMDGPU-FW] Header: size={}dw, ucode_offset={:#X}", 
                        dw1, ucode_offset);
                    return &data[ucode_offset..];
                }
            }
            crate::serial_println!("[AMDGPU-FW] Header: {}dw ({}B), ucode starts at {:#X}", 
                dw1, header_bytes, header_bytes);
            return &data[header_bytes..];
        }
    }
    
    // No recognized header — treat entire blob as ucode
    crate::serial_println!("[AMDGPU-FW] No header detected, using raw blob");
    data
}

// ═══════════════════════════════════════════════════════════════════════════════
// Engine-Specific Firmware Loading
// ═══════════════════════════════════════════════════════════════════════════════

/// Load RLC (Run List Controller) firmware
/// 
/// RLC manages power gating, clock gating, and context switching.
/// It must be loaded first before GFX/Compute engines can operate.
fn load_rlc(mmio: u64, ucode: &[u8]) -> Result<(), &'static str> {
    let dwords = ucode.len() / 4;
    if dwords == 0 { return Err("RLC firmware is empty"); }
    
    crate::log!("[AMDGPU-FW] Loading RLC: {} bytes ({} DWORDs)", ucode.len(), dwords);
    
        // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe {
        // 1. Halt RLC
        let rlc_cntl = mmio_read32(mmio, RLC_CNTL);
        mmio_write32(mmio, RLC_CNTL, rlc_cntl & !(1u32)); // Clear RLC_ENABLE bit 0
        
        // Wait for RLC to idle
        for _ in 0..10000 {
            let status = mmio_read32(mmio, RLC_STATUS);
            if status & 1 == 0 { break; } // Bit 0 = busy
            core::hint::spin_loop();
        }
        
        // 2. Program RLC firmware size
        mmio_write32(mmio, RLC_GPM_UCODE_SIZE, dwords as u32);
        
        // 3. Set start address to 0
        mmio_write32(mmio, RLC_GPM_UCODE_ADDRESS, 0);
        
        // 4. Write firmware DWORDs sequentially
        for i in 0..dwords {
            let offset = i * 4;
            if offset + 4 > ucode.len() { break; }
            let dw = u32::from_le_bytes([
                ucode[offset], ucode[offset+1], ucode[offset+2], ucode[offset+3]
            ]);
            mmio_write32(mmio, RLC_GPM_UCODE_DATA, dw);
        }
        
        // 5. Re-enable RLC
        mmio_write32(mmio, RLC_CNTL, rlc_cntl | 1); // Set RLC_ENABLE
        
        // 6. Verify RLC started
        for _ in 0..FW_LOAD_TIMEOUT {
            let status = mmio_read32(mmio, RLC_STATUS);
            if status & 1 != 0 {
                crate::log!("[AMDGPU-FW] RLC running (stat={:#X})", status);
                return Ok(());
            }
            core::hint::spin_loop();
        }
    }
    
    crate::log!("[AMDGPU-FW] RLC loaded ({} DWORDs) — verifying...", dwords);
    Ok(())
}

/// Load GFX Command Processor firmware (PFP, ME, CE)
///
/// These three micro-engines work together to parse and execute PM4 packets:
/// - PFP: Pre-Fetch Parser — reads ahead in the command ring buffer
/// - ME: Micro Engine — main draw/dispatch state machine
/// - CE: Constant Engine — pre-fetches constant buffer data
fn load_cp_gfx(mmio: u64, pfp: Option<&[u8]>, me: Option<&[u8]>, ce: Option<&[u8]>) -> Result<(), &'static str> {
        // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe {
        // 1. Halt all three CP engines
        let me_cntl = mmio_read32(mmio, CP_ME_CNTL);
        mmio_write32(mmio, CP_ME_CNTL, me_cntl | CP_ME_HALT | CP_PFP_HALT | CP_CE_HALT);
        
        // Small delay for engines to halt
        for _ in 0..10000 { core::hint::spin_loop(); }
        
        // 2. Load PFP firmware
        if let Some(fw) = pfp {
            let dwords = fw.len() / 4;
            crate::log!("[AMDGPU-FW] Loading PFP: {} DWORDs", dwords);
            mmio_write32(mmio, CP_PFP_UCODE_ADDRESS, 0);
            for i in 0..dwords {
                let off = i * 4;
                if off + 4 > fw.len() { break; }
                let dw = u32::from_le_bytes([fw[off], fw[off+1], fw[off+2], fw[off+3]]);
                mmio_write32(mmio, CP_PFP_UCODE_DATA, dw);
            }
        }
        
        // 3. Load ME firmware
        if let Some(fw) = me {
            let dwords = fw.len() / 4;
            crate::log!("[AMDGPU-FW] Loading ME: {} DWORDs", dwords);
            mmio_write32(mmio, CP_ME_UCODE_ADDRESS, 0);
            for i in 0..dwords {
                let off = i * 4;
                if off + 4 > fw.len() { break; }
                let dw = u32::from_le_bytes([fw[off], fw[off+1], fw[off+2], fw[off+3]]);
                mmio_write32(mmio, CP_ME_UCODE_DATA, dw);
            }
        }
        
        // 4. Load CE firmware
        if let Some(fw) = ce {
            let dwords = fw.len() / 4;
            crate::log!("[AMDGPU-FW] Loading CE: {} DWORDs", dwords);
            mmio_write32(mmio, CP_CE_UCODE_ADDRESS, 0);
            for i in 0..dwords {
                let off = i * 4;
                if off + 4 > fw.len() { break; }
                let dw = u32::from_le_bytes([fw[off], fw[off+1], fw[off+2], fw[off+3]]);
                mmio_write32(mmio, CP_CE_UCODE_DATA, dw);
            }
        }
        
        // 5. Un-halt the engines
        mmio_write32(mmio, CP_ME_CNTL, me_cntl & !(CP_ME_HALT | CP_PFP_HALT | CP_CE_HALT));
        
        // 6. Wait for CP to become idle (not busy)
        for _ in 0..100000 {
            let grbm = mmio_read32(mmio, regs::GRBM_STATUS);
            if grbm & regs::GRBM_STATUS_CP_BUSY == 0 {
                crate::log!("[AMDGPU-FW] CP GFX engines started");
                return Ok(());
            }
            core::hint::spin_loop();
        }
    }
    
    crate::log!("[AMDGPU-FW] CP GFX loaded — engine busy (may need time)");
    Ok(())
}

/// Load MEC (Micro Engine Compute) firmware
///
/// MEC firmware drives the compute queue scheduler (HQD).
/// Without it, compute dispatches will hang.
fn load_mec(mmio: u64, mec1: Option<&[u8]>, mec2: Option<&[u8]>) -> Result<(), &'static str> {
        // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe {
        // 1. Halt MEC
        let mec_cntl = mmio_read32(mmio, regs::CP_MEC_CNTL);
        mmio_write32(mmio, regs::CP_MEC_CNTL, mec_cntl | (1 << 28)); // MEC_HALT
        
        for _ in 0..10000 { core::hint::spin_loop(); }
        
        // 2. Load MEC1 firmware
        if let Some(fw) = mec1 {
            let dwords = fw.len() / 4;
            crate::log!("[AMDGPU-FW] Loading MEC1: {} DWORDs", dwords);
            mmio_write32(mmio, CP_MEC_ME1_UCODE_ADDRESS, 0);
            for i in 0..dwords {
                let off = i * 4;
                if off + 4 > fw.len() { break; }
                let dw = u32::from_le_bytes([fw[off], fw[off+1], fw[off+2], fw[off+3]]);
                mmio_write32(mmio, CP_MEC_ME1_UCODE_DATA, dw);
            }
        }
        
        // 3. Load MEC2 firmware
        if let Some(fw) = mec2 {
            let dwords = fw.len() / 4;
            crate::log!("[AMDGPU-FW] Loading MEC2: {} DWORDs", dwords);
            mmio_write32(mmio, CP_MEC_ME2_UCODE_ADDRESS, 0);
            for i in 0..dwords {
                let off = i * 4;
                if off + 4 > fw.len() { break; }
                let dw = u32::from_le_bytes([fw[off], fw[off+1], fw[off+2], fw[off+3]]);
                mmio_write32(mmio, CP_MEC_ME2_UCODE_DATA, dw);
            }
        }
        
        // 4. Un-halt MEC
        mmio_write32(mmio, regs::CP_MEC_CNTL, mec_cntl & !(1u32 << 28));
    }
    
    crate::log!("[AMDGPU-FW] MEC firmware loaded");
    Ok(())
}

/// Load SDMA engine firmware
fn load_sdma(mmio: u64, engine: usize, fw: &[u8]) -> Result<(), &'static str> {
    let dwords = fw.len() / 4;
    if dwords == 0 { return Err("SDMA firmware is empty"); }
    
    let (address_register, data_register, f32_cntl) = // Correspondance de motifs — branchement exhaustif de Rust.
match engine {
        0 => (SDMA0_UCODE_ADDRESS, SDMA0_UCODE_DATA, regs::SDMA0_F32_CNTL),
        1 => (SDMA1_UCODE_ADDRESS, SDMA1_UCODE_DATA, regs::SDMA1_F32_CNTL),
        _ => return Err("Invalid SDMA engine index"),
    };
    
    crate::log!("[AMDGPU-FW] Loading SDMA{}: {} DWORDs", engine, dwords);
    
        // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe {
        // 1. Halt the SDMA F32 microengine
        mmio_write32(mmio, f32_cntl, mmio_read32(mmio, f32_cntl) | 1); // HALT bit
        for _ in 0..10000 { core::hint::spin_loop(); }
        
        // 2. Set ucode start address to 0
        mmio_write32(mmio, address_register, 0);
        
        // 3. Write firmware DWORDs
        for i in 0..dwords {
            let off = i * 4;
            if off + 4 > fw.len() { break; }
            let dw = u32::from_le_bytes([fw[off], fw[off+1], fw[off+2], fw[off+3]]);
            mmio_write32(mmio, data_register, dw);
        }
        
        // 4. Un-halt the SDMA engine
        mmio_write32(mmio, f32_cntl, mmio_read32(mmio, f32_cntl) & !1u32);
    }
    
    crate::log!("[AMDGPU-FW] SDMA{} firmware loaded", engine);
    Ok(())
}

// ═══════════════════════════════════════════════════════════════════════════════
// Main Firmware Init
// ═══════════════════════════════════════════════════════════════════════════════

/// Initialize and load all GPU firmware
///
/// Call this after amdgpu::init() has mapped MMIO.
/// Firmware files must be present in /lib/firmware/amdgpu/ in the ramfs.
pub fn init(mmio_base: u64) {
    crate::log!("[AMDGPU-FW] ═══════════════════════════════════════════════");
    crate::log!("[AMDGPU-FW] AMD GPU Firmware Loader — Navi 10");
    crate::log!("[AMDGPU-FW] ═══════════════════════════════════════════════");
    
    if mmio_base == 0 {
        crate::log!("[AMDGPU-FW] No MMIO base — skipping firmware init");
        return;
    }
    
    let mut state = FW_STATE.lock();
    state.mmio_base = mmio_base;
    
    // Create firmware directory if it doesn't exist
    let _ = crate::ramfs::with_filesystem(|fs| {
        let _ = fs.mkdir("/lib");
        let _ = fs.mkdir("/lib/firmware");
        let _ = fs.mkdir("/lib/firmware/amdgpu");
    });
    
    // Check for VBIOS/PSP pre-loaded state
    let smu_ver = // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe { mmio_read32(mmio_base, regs::MP1_SMN_C2PMSG_58) };
    crate::log!("[AMDGPU-FW] SMU firmware version: {:#010X}", smu_ver);
    if smu_ver != 0 && smu_ver != 0xFFFFFFFF {
        crate::log!("[AMDGPU-FW] SMU is active — VBIOS has initialized power management");
    } else {
        crate::log!("[AMDGPU-FW] SMU not active — cold boot, firmware required");
    }
    
    // Check GFX engine status before loading
    let grbm = // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe { mmio_read32(mmio_base, regs::GRBM_STATUS) };
    let cp_busy = grbm & regs::GRBM_STATUS_CP_BUSY != 0;
    let gui_active = grbm & regs::GRBM_STATUS_GUI_ACTIVE != 0;
    crate::log!("[AMDGPU-FW] Pre-load: GRBM={:#010X} CP_BUSY={} GUI_ACTIVE={}", 
        grbm, cp_busy, gui_active);
    
    // ── Try to load each firmware component ─────────────────────────────
    
    // 1. RLC — must be loaded first
    if let Some(raw) = read_fw_file(FW_RLC) {
        let ucode = parse_fw_blob(&raw);
                // Correspondance de motifs — branchement exhaustif de Rust.
match load_rlc(mmio_base, ucode) {
            Ok(()) => state.rlc = FwStatus::Loaded,
            Err(e) => {
                crate::log!("[AMDGPU-FW] RLC load failed: {}", e);
                state.rlc = FwStatus::Failed;
            }
        }
    } else {
        crate::log!("[AMDGPU-FW] {} not found in {}", FW_RLC, FW_DIRECTORY);
        state.rlc = FwStatus::NotFound;
    }
    
    // 2. GFX CP — PFP, ME, CE
    let pfp_raw = read_fw_file(FW_PFP);
    let me_raw = read_fw_file(FW_ME);
    let ce_raw = read_fw_file(FW_CE);
    
    let pfp_ucode = pfp_raw.as_deref().map(parse_fw_blob);
    let me_ucode = me_raw.as_deref().map(parse_fw_blob);
    let ce_ucode = ce_raw.as_deref().map(parse_fw_blob);
    
    if pfp_ucode.is_some() || me_ucode.is_some() || ce_ucode.is_some() {
                // Correspondance de motifs — branchement exhaustif de Rust.
match load_cp_gfx(mmio_base, pfp_ucode, me_ucode, ce_ucode) {
            Ok(()) => {
                if pfp_ucode.is_some() { state.pfp = FwStatus::Loaded; }
                if me_ucode.is_some() { state.me = FwStatus::Loaded; }
                if ce_ucode.is_some() { state.ce = FwStatus::Loaded; }
            }
            Err(e) => {
                crate::log!("[AMDGPU-FW] CP GFX load failed: {}", e);
                state.pfp = FwStatus::Failed;
                state.me = FwStatus::Failed;
                state.ce = FwStatus::Failed;
            }
        }
    } else {
        crate::log!("[AMDGPU-FW] No GFX CP firmware found (PFP/ME/CE)");
    }
    
    // 3. MEC — compute engine
    let mec1_raw = read_fw_file(FW_MEC);
    let mec2_raw = read_fw_file(FW_MEC2);
    
    let mec1_ucode = mec1_raw.as_deref().map(parse_fw_blob);
    let mec2_ucode = mec2_raw.as_deref().map(parse_fw_blob);
    
    if mec1_ucode.is_some() || mec2_ucode.is_some() {
                // Correspondance de motifs — branchement exhaustif de Rust.
match load_mec(mmio_base, mec1_ucode, mec2_ucode) {
            Ok(()) => {
                if mec1_ucode.is_some() { state.mec1 = FwStatus::Loaded; }
                if mec2_ucode.is_some() { state.mec2 = FwStatus::Loaded; }
            }
            Err(e) => {
                crate::log!("[AMDGPU-FW] MEC load failed: {}", e);
                state.mec1 = FwStatus::Failed;
            }
        }
    } else {
        crate::log!("[AMDGPU-FW] No MEC firmware found");
    }
    
    // 4. SDMA engines  
    if let Some(raw) = read_fw_file(FW_SDMA0) {
        let ucode = parse_fw_blob(&raw);
                // Correspondance de motifs — branchement exhaustif de Rust.
match load_sdma(mmio_base, 0, ucode) {
            Ok(()) => state.sdma0 = FwStatus::Loaded,
            Err(e) => {
                crate::log!("[AMDGPU-FW] SDMA0 load failed: {}", e);
                state.sdma0 = FwStatus::Failed;
            }
        }
    }
    
    if let Some(raw) = read_fw_file(FW_SDMA1) {
        let ucode = parse_fw_blob(&raw);
                // Correspondance de motifs — branchement exhaustif de Rust.
match load_sdma(mmio_base, 1, ucode) {
            Ok(()) => state.sdma1 = FwStatus::Loaded,
            Err(e) => {
                crate::log!("[AMDGPU-FW] SDMA1 load failed: {}", e);
                state.sdma1 = FwStatus::Failed;
            }
        }
    }
    
    // ── Summary ─────────────────────────────────────────────────────────
    let loaded_count = [state.rlc, state.pfp, state.me, state.ce, 
                        state.mec1, state.mec2, state.sdma0, state.sdma1]
        .iter().filter(|&&s| s == FwStatus::Loaded).count();
    
    crate::log!("[AMDGPU-FW] ───────────────────────────────────────────────");
    crate::log!("[AMDGPU-FW] Firmware status ({}/8 loaded):", loaded_count);
    crate::log!("[AMDGPU-FW]   RLC:   {:?}", state.rlc);
    crate::log!("[AMDGPU-FW]   PFP:   {:?}", state.pfp);
    crate::log!("[AMDGPU-FW]   ME:    {:?}", state.me);
    crate::log!("[AMDGPU-FW]   CE:    {:?}", state.ce);
    crate::log!("[AMDGPU-FW]   MEC1:  {:?}", state.mec1);
    crate::log!("[AMDGPU-FW]   MEC2:  {:?}", state.mec2);
    crate::log!("[AMDGPU-FW]   SDMA0: {:?}", state.sdma0);
    crate::log!("[AMDGPU-FW]   SDMA1: {:?}", state.sdma1);
    crate::log!("[AMDGPU-FW] ───────────────────────────────────────────────");
    
    if loaded_count == 0 {
        crate::log!("[AMDGPU-FW] No firmware loaded — GPU compute will use CPU fallback");
        crate::log!("[AMDGPU-FW] To enable GPU compute:");
        crate::log!("[AMDGPU-FW]   1. Copy firmware files to {}", FW_DIRECTORY);
        crate::log!("[AMDGPU-FW]   2. Run 'gpufw load' to reload firmware");
        crate::log!("[AMDGPU-FW]   3. Or add firmware as Limine boot modules");
    } else {
        FW_LOADED.store(true, Ordering::SeqCst);
        crate::log!("[AMDGPU-FW] Firmware loading complete — engines should be active");
    }
    
    // Post-load engine status
    let grbm_post = // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe { mmio_read32(mmio_base, regs::GRBM_STATUS) };
    crate::log!("[AMDGPU-FW] Post-load GRBM_STATUS: {:#010X}", grbm_post);
    
    drop(state);
}

// ═══════════════════════════════════════════════════════════════════════════════
// Public API
// ═══════════════════════════════════════════════════════════════════════════════

/// Check if any firmware has been loaded
pub fn is_loaded() -> bool {
    FW_LOADED.load(Ordering::Relaxed)
}

/// Get firmware status summary
pub fn summary() -> String {
    let state = FW_STATE.lock();
    let loaded = [state.rlc, state.pfp, state.me, state.ce,
                  state.mec1, state.mec2, state.sdma0, state.sdma1]
        .iter().filter(|&&s| s == FwStatus::Loaded || s == FwStatus::Running).count();
    format!("GPU Firmware: {}/8 loaded (RLC:{:?} MEC:{:?} SDMA:{:?})",
        loaded, state.rlc, state.mec1, state.sdma0)
}

/// Reload firmware from ramfs (called by shell command)
pub fn reload(mmio_base: u64) {
    crate::log!("[AMDGPU-FW] Reloading firmware...");
    init(mmio_base);
}

/// Get detailed status lines for display
pub fn status_lines() -> Vec<String> {
    let state = FW_STATE.lock();
    let mut lines = Vec::new();
    lines.push(format!("RLC  (Run List Controller):  {:?}", state.rlc));
    lines.push(format!("PFP  (Pre-Fetch Parser):     {:?}", state.pfp));
    lines.push(format!("ME   (Micro Engine):         {:?}", state.me));
    lines.push(format!("CE   (Constant Engine):      {:?}", state.ce));
    lines.push(format!("MEC1 (Compute Engine 1):     {:?}", state.mec1));
    lines.push(format!("MEC2 (Compute Engine 2):     {:?}", state.mec2));
    lines.push(format!("SDMA0 (DMA Engine 0):        {:?}", state.sdma0));
    lines.push(format!("SDMA1 (DMA Engine 1):        {:?}", state.sdma1));
    lines
}
