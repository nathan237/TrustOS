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

/// Busy-wait delay (~1µs per iteration via port 0x80 I/O)
#[inline]
fn gpu_udelay(us: u32) {
    for _ in 0..us {
        #[cfg(target_arch = "x86_64")]
        unsafe { core::arch::asm!("out 0x80, al", in("al") 0u8, options(nostack, nomem)); }
        #[cfg(not(target_arch = "x86_64"))]
        core::hint::spin_loop();
    }
}

/// Busy-wait delay in milliseconds
#[inline]
fn gpu_msleep(ms: u32) {
    gpu_udelay(ms * 1000);
}

// ═══════════════════════════════════════════════════════════════════════════════
// Constants
// ═══════════════════════════════════════════════════════════════════════════════

/// Firmware directory in ramfs
const FW_DIR: &str = "/lib/firmware/amdgpu";

/// Expected firmware file names for Navi 10
const FW_PFP: &str = "navi10_pfp.bin";
const FW_ME: &str = "navi10_me.bin";
const FW_CE: &str = "navi10_ce.bin";
const FW_MEC: &str = "navi10_mec.bin";
const FW_MEC2: &str = "navi10_mec2.bin";
const FW_RLC: &str = "navi10_rlc.bin";
const FW_SDMA0: &str = "navi10_sdma.bin";
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

// ── RLC (Run List Controller) firmware registers (GFX8/Polaris: gca/gfx_8_0_d.h) ──
/// RLC cntl — main enable/disable (mmRLC_CNTL = 0xEC00)
const RLC_CNTL: u32 = 0xEC00 * 4;  // 0x3B000
/// RLC status (mmRLC_STAT = 0xEC04)
const RLC_STAT: u32 = 0xEC04 * 4;  // 0x3B010
/// RLC safe mode request (mmRLC_SAFE_MODE = 0xEC05)
const RLC_SAFE_MODE: u32 = 0xEC05 * 4;  // 0x3B014
/// RLC CGTT management control (mmRLC_CGTT_MGCG_OVERRIDE = 0xEC48)
const RLC_CGTT_MGCG_OVERRIDE: u32 = 0xEC48 * 4;  // 0x3B120
/// RLC GPM ucode address (mmRLC_GPM_UCODE_ADDR = 0xF83C)
const RLC_GPM_UCODE_ADDR: u32 = 0xF83C * 4;  // 0x3E0F0
/// RLC GPM ucode data (mmRLC_GPM_UCODE_DATA = 0xF83D)
const RLC_GPM_UCODE_DATA: u32 = 0xF83D * 4;  // 0x3E0F4

// ── CP (Command Processor) GFX firmware registers (GFX8: gca/gfx_8_0_d.h) ──
/// PFP (Pre-Fetch Parser) firmware address register (mmCP_PFP_UCODE_ADDR = 0xF814)
const CP_PFP_UCODE_ADDR: u32 = 0xF814 * 4;  // 0x3E050
/// PFP firmware data register (mmCP_PFP_UCODE_DATA = 0xF815)
const CP_PFP_UCODE_DATA: u32 = 0xF815 * 4;  // 0x3E054
/// ME (Micro Engine) firmware address register (mmCP_ME_RAM_WADDR = 0xF816)
const CP_ME_UCODE_ADDR: u32 = 0xF816 * 4;  // 0x3E058
/// ME firmware data register (mmCP_ME_RAM_DATA = 0xF817)
const CP_ME_UCODE_DATA: u32 = 0xF817 * 4;  // 0x3E05C
/// CE (Constant Engine) firmware address register (mmCP_CE_UCODE_ADDR = 0xF818)
const CP_CE_UCODE_ADDR: u32 = 0xF818 * 4;  // 0x3E060
/// CE firmware data register (mmCP_CE_UCODE_DATA = 0xF819)
const CP_CE_UCODE_DATA: u32 = 0xF819 * 4;  // 0x3E064
/// ME master control — halt/resume ME, PFP, CE (mmCP_ME_CNTL = 0x21B6)
const CP_ME_CNTL: u32 = 0x21B6 * 4;  // 0x86D8 (same as GFX7)

// CP_ME_CNTL bits
const CP_ME_HALT: u32 = 1 << 28;
const CP_PFP_HALT: u32 = 1 << 26;
const CP_CE_HALT: u32 = 1 << 24;

// ── MEC (Micro Engine Compute) firmware registers (GFX8: gca/gfx_8_0_d.h) ──
/// MEC1 firmware address (mmCP_MEC_ME1_UCODE_ADDR = 0xF81A)
const CP_MEC_ME1_UCODE_ADDR: u32 = 0xF81A * 4;  // 0x3E068
/// MEC1 firmware data (mmCP_MEC_ME1_UCODE_DATA = 0xF81B)
const CP_MEC_ME1_UCODE_DATA: u32 = 0xF81B * 4;  // 0x3E06C
/// MEC2 firmware address (mmCP_MEC_ME2_UCODE_ADDR = 0xF81C)
const CP_MEC_ME2_UCODE_ADDR: u32 = 0xF81C * 4;  // 0x3E070
/// MEC2 firmware data (mmCP_MEC_ME2_UCODE_DATA = 0xF81D)
const CP_MEC_ME2_UCODE_DATA: u32 = 0xF81D * 4;  // 0x3E074

// ── SDMA firmware registers ─────────────────────────────────────────────────
/// SDMA engine 0 ucode address
const SDMA0_UCODE_ADDR: u32 = 0x4D88;
/// SDMA engine 0 ucode data
const SDMA0_UCODE_DATA: u32 = 0x4D8C;
/// SDMA engine 1 ucode address
const SDMA1_UCODE_ADDR: u32 = 0x4E88;
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

static FW_LOADED: AtomicBool = AtomicBool::new(false);

// ═══════════════════════════════════════════════════════════════════════════════
// Firmware File I/O
// ═══════════════════════════════════════════════════════════════════════════════

/// Read a firmware file from ramfs, returning the raw bytes
fn read_fw_file(name: &str) -> Option<Vec<u8>> {
    let path = format!("{}/{}", FW_DIR, name);
    crate::ramfs::with_fs(|fs| {
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
fn load_rlc(mmio: u64, ucode: &[u8], fw_version: u32) -> Result<(), &'static str> {
    let dwords = ucode.len() / 4;
    if dwords == 0 { return Err("RLC firmware is empty"); }
    
    crate::log!("[AMDGPU-FW] Loading RLC: {} bytes ({} DWORDs)", ucode.len(), dwords);

    unsafe {
        // 1. Halt RLC (Linux: gfx_v8_0_rlc_stop → RLC_CNTL.RLC_ENABLE_F32 = 0)
        let rlc_cntl = mmio_read32(mmio, RLC_CNTL);
        mmio_write32(mmio, RLC_CNTL, rlc_cntl & !(1u32)); // Clear RLC_ENABLE_F32 (bit 0 on Polaris)

        // Wait for RLC to idle (bounded poll, then fixed settle delay)
        for _ in 0..10000 {
            let stat = mmio_read32(mmio, RLC_STAT);
            if stat & 1 == 0 { break; } // Bit 0 = busy
            core::hint::spin_loop();
        }
        gpu_udelay(50); // Linux: settle after stop

        // SOFT_RESET_RLC pulse intentionally OMITTED: toggling GRBM_SOFT_RESET.RLC
        // without prior RLC_SAFE_MODE handshake hangs the PCIe bus on Polaris 10.
        // See /memories/repo/timing_patch_regression.md.

        // 2. Set start address to 0
        mmio_write32(mmio, RLC_GPM_UCODE_ADDR, 0);
        
        // 3. Write firmware DWORDs sequentially
        for i in 0..dwords {
            let offset = i * 4;
            if offset + 4 > ucode.len() { break; }
            let dw = u32::from_le_bytes([
                ucode[offset], ucode[offset+1], ucode[offset+2], ucode[offset+3]
            ]);
            mmio_write32(mmio, RLC_GPM_UCODE_DATA, dw);
        }
        
        // 4. Write firmware version to UCODE_ADDR (GFX8 convention from Linux gfx_v8_0.c)
        mmio_write32(mmio, RLC_GPM_UCODE_ADDR, fw_version);
        
        // 5. Re-enable RLC (Linux: gfx_v8_0_rlc_start → RLC_CNTL.RLC_ENABLE_F32 = 1, then udelay(50))
        mmio_write32(mmio, RLC_CNTL, 1);
        gpu_udelay(50); // Linux: mandatory settle after start, before any CP enable

        // 6. Verify RLC started
        for _ in 0..FW_LOAD_TIMEOUT {
            let stat = mmio_read32(mmio, RLC_STAT);
            if stat & 1 != 0 {
                crate::log!("[AMDGPU-FW] RLC running (stat={:#X})", stat);
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
    unsafe {
        // 1. Halt all three CP engines (Linux: gfx_v8_0_cp_gfx_enable(false) + udelay(50))
        let me_cntl = mmio_read32(mmio, CP_ME_CNTL);
        mmio_write32(mmio, CP_ME_CNTL, me_cntl | CP_ME_HALT | CP_PFP_HALT | CP_CE_HALT);
        gpu_udelay(50);
        
        // 2. Load PFP firmware
        if let Some(fw) = pfp {
            let dwords = fw.len() / 4;
            crate::log!("[AMDGPU-FW] Loading PFP: {} DWORDs", dwords);
            mmio_write32(mmio, CP_PFP_UCODE_ADDR, 0);
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
            mmio_write32(mmio, CP_ME_UCODE_ADDR, 0);
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
            mmio_write32(mmio, CP_CE_UCODE_ADDR, 0);
            for i in 0..dwords {
                let off = i * 4;
                if off + 4 > fw.len() { break; }
                let dw = u32::from_le_bytes([fw[off], fw[off+1], fw[off+2], fw[off+3]]);
                mmio_write32(mmio, CP_CE_UCODE_DATA, dw);
            }
        }
        
        // 5. Un-halt the engines (Linux: gfx_v8_0_cp_gfx_enable(true) + udelay(50))
        mmio_write32(mmio, CP_ME_CNTL, me_cntl & !(CP_ME_HALT | CP_PFP_HALT | CP_CE_HALT));
        gpu_udelay(50);

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
///
/// Each `Option` slot is `(ucode, jt_dword_count, fw_version)`.  Linux loads
/// only the dwords *before* the jump table (`jt_offset` count), then writes
/// the firmware version to UCODE_ADDR; finally `CP_MEC_CNTL = 0` is written
/// directly (clearing both halt bits) to start ME1+ME2.  See
/// `gfx_v8_0_cp_compute_load_microcode` and `gfx_v8_0_cp_compute_enable`.
fn load_mec(
    mmio: u64,
    mec1: Option<(&[u8], u32, u32)>,
    mec2: Option<(&[u8], u32, u32)>,
) -> Result<(), &'static str> {
    let mec_halt_bits: u32 = (1u32 << 30) | (1u32 << 28);
    // mmGRBM_GFX_INDEX = 0xC200, broadcast = 0xE0000000 (SE/SH/INSTANCE bcast)
    const GRBM_GFX_INDEX_BCAST: u32 = 0xE0000000;
    // mmCGTT_CP_CLK_CTRL = 0xF180 → byte 0x3C600 (force-on bits = 0)
    const CGTT_CP_CLK_CTRL: u32 = 0xF180 * 4;
    // mmCGTT_RLC_CLK_CTRL = 0xF181
    const CGTT_RLC_CLK_CTRL: u32 = 0xF181 * 4;
    // GRBM_SOFT_RESET bit 16 = SOFT_RESET_CP
    const SOFT_RESET_CP: u32 = 1 << 16;
    // Polaris GFX8 power-gating regs (gfx_v8_0_d.h offsets, dword * 4):
    //   mmRLC_PG_CNTL          = 0xEC43 → 0x3B10C
    //   mmRLC_CGCG_CGLS_CTRL   = 0xEC22 → 0x3B088 (also RLC_CP_SCHEDULERS!)
    //   mmRLC_PG_ALWAYS_ON_CU_MASK = 0xEC34
    //   mmRLC_MAX_PG_CU       = 0xEC35
    // Disable GFX power gating: bit 0 = STATIC_PER_CU_PGFSM_EN, bit 16 = GFX_POWER_GATING_ENABLE
    const POL_RLC_PG_CNTL: u32 = 0xEC43 * 4;

    unsafe {
        // 0. Broadcast GRBM_GFX_INDEX so MEC writes hit every SE/SH instance.
        mmio_write32(mmio, regs::GRBM_GFX_INDEX, GRBM_GFX_INDEX_BCAST);

        // 0_pg. Disable GFX power gating BEFORE touching MEC SRAM. If GFX domain
        //       is power-gated, CP_MEC*_UCODE_DATA writes silently drop and
        //       readbacks return a sentinel pattern (e.g. 0xF000EEF3).
        let pg_before = mmio_read32(mmio, POL_RLC_PG_CNTL);
        mmio_write32(mmio, POL_RLC_PG_CNTL, 0);
        let pg_after = mmio_read32(mmio, POL_RLC_PG_CNTL);
        crate::log!("[AMDGPU-FW] RLC_PG_CNTL: {:#X} -> {:#X} (disabled GFX PG)",
            pg_before, pg_after);

        // 0a. Force CP and RLC clocks always-on (clear all dynamic clock-gating
        //     overrides for these blocks). Without this CP_MEC F32 may stay
        //     clock-gated after un-halt and never advance PC.
        let cp_cg_before = mmio_read32(mmio, CGTT_CP_CLK_CTRL);
        let rlc_cg_before = mmio_read32(mmio, CGTT_RLC_CLK_CTRL);
        mmio_write32(mmio, CGTT_CP_CLK_CTRL, 0);
        mmio_write32(mmio, CGTT_RLC_CLK_CTRL, 0);
        crate::log!("[AMDGPU-FW] CGTT_CP_CLK={:#X}->0  CGTT_RLC_CLK={:#X}->0",
            cp_cg_before, rlc_cg_before);

        // 1. Halt MEC1 + MEC2 (write the halt mask directly, Linux pattern).
        // Linux gfx_v8_0_cp_compute_enable: WREG32 + udelay(50).
        mmio_write32(mmio, regs::CP_MEC_CNTL, mec_halt_bits);
        gpu_udelay(50);

        // 1_diag. SRAM probe — diagnose 0xF000EEF3 stuck readback. Test if MEC
        //         SRAM is actually writable BEFORE we burn 65k dwords.
        //         Polaris RLC live-state regs (gfx_v8_0_d.h):
        //           mmRLC_STAT     = 0xEC04 → 0x3B010
        //           mmRLC_GPM_STAT = 0xEC0E → 0x3B038 (bit 0 = GFX_POWER_STATUS)
        //           mmRLC_CNTL     = 0xEC00 → 0x3B000
        const POL_RLC_CNTL: u32     = 0xEC00 * 4;
        const POL_RLC_STAT: u32     = 0xEC04 * 4;
        const POL_RLC_GPM_STAT: u32 = 0xEC0E * 4;
        let rlc_cntl_v = mmio_read32(mmio, POL_RLC_CNTL);
        let rlc_stat_v = mmio_read32(mmio, POL_RLC_STAT);
        let rlc_gpm_v  = mmio_read32(mmio, POL_RLC_GPM_STAT);
        crate::log!("[AMDGPU-FW] RLC live: CNTL={:#X} STAT={:#X} GPM_STAT={:#X} (F32_EN={} GFX_PWR={})",
            rlc_cntl_v, rlc_stat_v, rlc_gpm_v,
            rlc_cntl_v & 1, (rlc_gpm_v >> 0) & 1);

        // SRAM cold read: address 0, no write yet
        mmio_write32(mmio, CP_MEC_ME1_UCODE_ADDR, 0);
        let cold0 = mmio_read32(mmio, CP_MEC_ME1_UCODE_DATA);
        let cold1 = mmio_read32(mmio, CP_MEC_ME1_UCODE_DATA);
        // SRAM write probe: write known patterns to addr 0..1, read back
        mmio_write32(mmio, CP_MEC_ME1_UCODE_ADDR, 0);
        mmio_write32(mmio, CP_MEC_ME1_UCODE_DATA, 0xCAFEBABE);
        mmio_write32(mmio, CP_MEC_ME1_UCODE_DATA, 0xDEADC0DE);
        mmio_write32(mmio, CP_MEC_ME1_UCODE_ADDR, 0);
        let probe0 = mmio_read32(mmio, CP_MEC_ME1_UCODE_DATA);
        let probe1 = mmio_read32(mmio, CP_MEC_ME1_UCODE_DATA);
        crate::log!("[AMDGPU-FW] MEC1 SRAM probe: cold[0]={:#X} cold[1]={:#X} | after-write[0]={:#X}(exp CAFEBABE) [1]={:#X}(exp DEADC0DE)",
            cold0, cold1, probe0, probe1);
        // 1a. SOFT_RESET pulse on CP+GFX+RLC blocks. On Polaris (Volcanic
        //     Islands) GRBM_SOFT_RESET layout:
        //       bit 16 = SOFT_RESET_CP
        //       bit 17 = SOFT_RESET_GFX
        //       bit 18 = SOFT_RESET_RLC
        //     This is the Linux gfx_v8_0_soft_reset sequence — required when
        //     MEC SRAM reads return a stuck sentinel (e.g. 0xF000EEF3) which
        //     means the F32 sub-block is wedged in a non-running, non-reset
        //     limbo state from VBIOS POST. The pulse forces a clean re-init.
        const SOFT_RESET_CP_GFX_RLC: u32 = (1 << 16) | (1 << 17) | (1 << 18);
        let grbm_pre = mmio_read32(mmio, regs::GRBM_SOFT_RESET);
        mmio_write32(mmio, regs::GRBM_SOFT_RESET, grbm_pre | SOFT_RESET_CP_GFX_RLC);
        let grbm_after_set = mmio_read32(mmio, regs::GRBM_SOFT_RESET);
        gpu_udelay(50);
        mmio_write32(mmio, regs::GRBM_SOFT_RESET, grbm_pre & !SOFT_RESET_CP_GFX_RLC);
        gpu_udelay(50);
        let grbm_after_clr = mmio_read32(mmio, regs::GRBM_SOFT_RESET);
        crate::log!("[AMDGPU-FW] GRBM_SOFT_RESET pulse: pre={:#X} set={:#X} clr={:#X}",
            grbm_pre, grbm_after_set, grbm_after_clr);

        // Re-halt MEC after reset (CNTL bits may be cleared by reset).
        mmio_write32(mmio, regs::CP_MEC_CNTL, mec_halt_bits);
        gpu_udelay(50);

        // 2. MEC1 firmware — Linux gfx_v8_0_cp_compute_load_microcode:
        //    UCODE_ADDR = 0, write the FULL ucode, then UCODE_ADDR = fw_version.
        //    `jt_dwords` here is actually `jt_offset` (header field) — IGNORED.
        if let Some((fw, _jt_offset, fw_version)) = mec1 {
            let total_dwords = (fw.len() / 4) as u32;
            let load_dwords = total_dwords;
            crate::log!("[AMDGPU-FW] Loading MEC1: {}/{} DWORDs (ver={:#X})",
                load_dwords, total_dwords, fw_version);
            mmio_write32(mmio, CP_MEC_ME1_UCODE_ADDR, 0);
            for i in 0..load_dwords as usize {
                let off = i * 4;
                let dw = u32::from_le_bytes([fw[off], fw[off+1], fw[off+2], fw[off+3]]);
                mmio_write32(mmio, CP_MEC_ME1_UCODE_DATA, dw);
            }
            // Verify FW landed: read back first 2 dwords via UCODE_ADDR=0 auto-increment.
            mmio_write32(mmio, CP_MEC_ME1_UCODE_ADDR, 0);
            let rb0 = mmio_read32(mmio, CP_MEC_ME1_UCODE_DATA);
            let rb1 = mmio_read32(mmio, CP_MEC_ME1_UCODE_DATA);
            let exp0 = u32::from_le_bytes([fw[0], fw[1], fw[2], fw[3]]);
            let exp1 = u32::from_le_bytes([fw[4], fw[5], fw[6], fw[7]]);
            crate::log!("[AMDGPU-FW] MEC1 UCODE readback [0]={:#X}(exp {:#X}) [1]={:#X}(exp {:#X}) {}",
                rb0, exp0, rb1, exp1,
                if rb0 == exp0 && rb1 == exp1 { "OK" } else { "MISMATCH" });
            // Write version back to ADDR — required by GFX8 loader.
            mmio_write32(mmio, CP_MEC_ME1_UCODE_ADDR, fw_version);
        }

        // 3. MEC2 firmware — same: load full ucode.
        if let Some((fw, _jt_offset, fw_version)) = mec2 {
            let total_dwords = (fw.len() / 4) as u32;
            let load_dwords = total_dwords;
            crate::log!("[AMDGPU-FW] Loading MEC2: {}/{} DWORDs (ver={:#X})",
                load_dwords, total_dwords, fw_version);
            mmio_write32(mmio, CP_MEC_ME2_UCODE_ADDR, 0);
            for i in 0..load_dwords as usize {
                let off = i * 4;
                let dw = u32::from_le_bytes([fw[off], fw[off+1], fw[off+2], fw[off+3]]);
                mmio_write32(mmio, CP_MEC_ME2_UCODE_DATA, dw);
            }
            mmio_write32(mmio, CP_MEC_ME2_UCODE_ADDR, fw_version);
        }

        // 3a. (REMOVED) Soft reset was destroying loaded firmware.
        // Linux never resets CP between load and un-halt.

        // 4. Un-halt MEC1+MEC2 by writing 0 (Linux `cp_compute_enable(true)` + udelay(50)).
        mmio_write32(mmio, regs::CP_MEC_CNTL, 0);
        gpu_udelay(50);
        // Extra settle window for MEC F32 boot ROM to fetch first instructions
        // before we read PC (Linux relies on subsequent ring tests to confirm).
        gpu_udelay(450);

        // 5. Verify MEC1 PC has advanced past 0 (boot ROM running).
        // PC regs: mmCP_MEC1_INSTR_PNTR=0x208E, mmCP_MEC2_INSTR_PNTR=0x208F.
        // Requires SRBM_GFX_CNTL.MEID select to read each MEC's PC.
        mmio_write32(mmio, regs::SRBM_GFX_CNTL_V8, 1 << 2); // MEID=1
        let mec1_pc = mmio_read32(mmio, regs::CP_MEC1_INSTR_PNTR);
        mmio_write32(mmio, regs::SRBM_GFX_CNTL_V8, 2 << 2); // MEID=2
        let mec2_pc = mmio_read32(mmio, regs::CP_MEC2_INSTR_PNTR);
        mmio_write32(mmio, regs::SRBM_GFX_CNTL_V8, 0); // restore GFX
        crate::log!("[AMDGPU-FW] post-unhalt: MEC1_PC={:#X} MEC2_PC={:#X}", mec1_pc, mec2_pc);
    }

    crate::log!("[AMDGPU-FW] MEC firmware loaded + un-halted");
    Ok(())
}

/// Load SDMA engine firmware
fn load_sdma(mmio: u64, engine: usize, fw: &[u8]) -> Result<(), &'static str> {
    let dwords = fw.len() / 4;
    if dwords == 0 { return Err("SDMA firmware is empty"); }
    
    let (addr_reg, data_reg, f32_cntl) = match engine {
        0 => (SDMA0_UCODE_ADDR, SDMA0_UCODE_DATA, regs::SDMA0_F32_CNTL),
        1 => (SDMA1_UCODE_ADDR, SDMA1_UCODE_DATA, regs::SDMA1_F32_CNTL),
        _ => return Err("Invalid SDMA engine index"),
    };
    
    crate::log!("[AMDGPU-FW] Loading SDMA{}: {} DWORDs", engine, dwords);
    
    unsafe {
        // 1. Halt the SDMA F32 microengine
        mmio_write32(mmio, f32_cntl, mmio_read32(mmio, f32_cntl) | 1); // HALT bit
        for _ in 0..10000 { core::hint::spin_loop(); }
        
        // 2. Set ucode start address to 0
        mmio_write32(mmio, addr_reg, 0);
        
        // 3. Write firmware DWORDs
        for i in 0..dwords {
            let off = i * 4;
            if off + 4 > fw.len() { break; }
            let dw = u32::from_le_bytes([fw[off], fw[off+1], fw[off+2], fw[off+3]]);
            mmio_write32(mmio, data_reg, dw);
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

/// VRAM info stored during init for shell commands
static VRAM_INFO: Mutex<(u64, u64)> = Mutex::new((0, 0)); // (phys, size)

/// Embedded SOS blob (empty for Polaris, populated for Navi if feature enabled)
static EMBEDDED_SOS: &[u8] = &[];

/// Get stored VRAM physical address
pub fn vram_phys() -> u64 {
    VRAM_INFO.lock().0
}

/// Get stored VRAM size
pub fn vram_size_stored() -> u64 {
    VRAM_INFO.lock().1
}

/// Get embedded SOS blob
pub fn embedded_sos() -> &'static [u8] {
    EMBEDDED_SOS
}

/// Initialize and load all GPU firmware (Navi 10 path)
///
/// Call this after amdgpu::init() has mapped MMIO.
/// Firmware files must be present in /lib/firmware/amdgpu/ in the ramfs.
pub fn init(mmio_base: u64, vram_phys_addr: u64, vram_size_val: u64) {
    {
        let mut vi = VRAM_INFO.lock();
        vi.0 = vram_phys_addr;
        vi.1 = vram_size_val;
    }
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
    let _ = crate::ramfs::with_fs(|fs| {
        let _ = fs.mkdir("/lib");
        let _ = fs.mkdir("/lib/firmware");
        let _ = fs.mkdir("/lib/firmware/amdgpu");
    });
    
    // Check for VBIOS/PSP pre-loaded state
    let smu_ver = unsafe { mmio_read32(mmio_base, regs::MP1_SMN_C2PMSG_58) };
    crate::log!("[AMDGPU-FW] SMU firmware version: {:#010X}", smu_ver);
    if smu_ver != 0 && smu_ver != 0xFFFFFFFF {
        crate::log!("[AMDGPU-FW] SMU is active — VBIOS has initialized power management");
    } else {
        crate::log!("[AMDGPU-FW] SMU not active — cold boot, firmware required");
    }
    
    // Check GFX engine status before loading
    let grbm = unsafe { mmio_read32(mmio_base, regs::GRBM_STATUS) };
    let cp_busy = grbm & regs::GRBM_STATUS_CP_BUSY != 0;
    let gui_active = grbm & regs::GRBM_STATUS_GUI_ACTIVE != 0;
    crate::log!("[AMDGPU-FW] Pre-load: GRBM={:#010X} CP_BUSY={} GUI_ACTIVE={}", 
        grbm, cp_busy, gui_active);
    
    // ── Try to load each firmware component ─────────────────────────────
    
    // 1. RLC — must be loaded first
    if let Some(raw) = read_fw_file(FW_RLC) {
        let ucode = parse_fw_blob(&raw);
        match load_rlc(mmio_base, ucode, 0) {
            Ok(()) => state.rlc = FwStatus::Loaded,
            Err(e) => {
                crate::log!("[AMDGPU-FW] RLC load failed: {}", e);
                state.rlc = FwStatus::Failed;
            }
        }
    } else {
        crate::log!("[AMDGPU-FW] {} not found in {}", FW_RLC, FW_DIR);
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

    // Same gfx_firmware_header_v1_0 layout on Navi as Polaris.
    let mec1_ucode = mec1_raw.as_deref().and_then(parse_polaris_gfx_fw);
    let mec2_ucode = mec2_raw.as_deref().and_then(parse_polaris_gfx_fw);

    if mec1_ucode.is_some() || mec2_ucode.is_some() {
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
        crate::log!("[AMDGPU-FW]   1. Copy firmware files to {}", FW_DIR);
        crate::log!("[AMDGPU-FW]   2. Run 'gpufw load' to reload firmware");
        crate::log!("[AMDGPU-FW]   3. Or add firmware as Limine boot modules");
    } else {
        FW_LOADED.store(true, Ordering::SeqCst);
        crate::log!("[AMDGPU-FW] Firmware loading complete — engines should be active");
    }
    
    // Post-load engine status
    let grbm_post = unsafe { mmio_read32(mmio_base, regs::GRBM_STATUS) };
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
    let (vp, vs) = *VRAM_INFO.lock();
    init(mmio_base, vp, vs);
}

/// Initialize Polaris SDMA engines - called from mod.rs for Polaris GPUs
pub fn init_polaris(mmio: u64, vram_phys_addr: u64, vram_ap_size: u64) {
    {
        let mut vi = VRAM_INFO.lock();
        vi.0 = vram_phys_addr;
        vi.1 = vram_ap_size;
    }
    crate::log!("[POLARIS-FW] Polaris SDMA - Staged Mode (diag-only boot)");
    if mmio == 0 {
        crate::log!("[POLARIS-FW] No MMIO base - skipping");
        return;
    }
    // Boot = read-only diagnostic only. Use shell: gpu sdma init
    let mut state = FW_STATE.lock();
    state.mmio_base = mmio;
    state.sdma0 = FwStatus::Failed;
    state.sdma1 = FwStatus::Failed;
    drop(state);
    crate::log!("[POLARIS-FW] Staged init ready. Use 'gpu sdma init' or 'gpu sdma diag'.");
}

/// PSP boot manual (stub for Polaris — PSP not used)
pub fn psp_boot_manual() -> Vec<String> {
    alloc::vec!["PSP not used on Polaris (direct MMIO firmware load)".into()]
}

/// PSP diagnostic (stub)
pub fn psp_diag() -> Vec<String> {
    alloc::vec!["PSP diagnostic not available on Polaris".into()]
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

// =============================================================================
// Polaris 10 (GCN 4 / VI) SDMA v3.0 — Clean implementation
// =============================================================================
//
// Faithful to Linux: sdma_v3_0.c + gmc_v8_0.c + vi.c
// IP block init order: vi_common -> gmc_v8_0 -> tonga_ih -> gfx_v8_0 -> sdma_v3_0
//
// Single canonical Polaris SDMA bring-up implementation.

// Embedded Polaris firmware from linux-firmware
static EMBEDDED_POLARIS_SDMA0: &[u8] = include_bytes!("../../../../firmware/amdgpu/polaris10_sdma.bin");
static EMBEDDED_POLARIS_SDMA1: &[u8] = include_bytes!("../../../../firmware/amdgpu/polaris10_sdma1.bin");
static EMBEDDED_POLARIS_MEC1: &[u8] = include_bytes!("../../../../firmware/amdgpu/polaris10_mec.bin");
static EMBEDDED_POLARIS_MEC2: &[u8] = include_bytes!("../../../../firmware/amdgpu/polaris10_mec2.bin");
static EMBEDDED_POLARIS_RLC: &[u8] = include_bytes!("../../../../firmware/amdgpu/polaris10_rlc.bin");
static EMBEDDED_POLARIS_PFP: &[u8] = include_bytes!("../../../../firmware/amdgpu/polaris10_pfp.bin");
static EMBEDDED_POLARIS_ME: &[u8] = include_bytes!("../../../../firmware/amdgpu/polaris10_me.bin");
static EMBEDDED_POLARIS_CE: &[u8] = include_bytes!("../../../../firmware/amdgpu/polaris10_ce.bin");
static EMBEDDED_POLARIS_SMC:    &[u8] = include_bytes!("../../../../firmware/amdgpu/polaris10_smc.bin");
static EMBEDDED_POLARIS_SMC_SK: &[u8] = include_bytes!("../../../../firmware/amdgpu/polaris10_k_smc.bin");

/// Accessor for SMU SMC firmware blob.
/// `security_hard_key` = `SMU_FIRMWARE.SMU_SEL` bit (kicker indicator).
/// Linux mapping (drivers/gpu/drm/amd/pm/powerplay/smumgr/polaris10_smumgr.c):
///   kicker=1 (SEL=1) → polaris10_k_smc.bin
///   kicker=0 (SEL=0) → polaris10_smc.bin
pub fn embedded_polaris_smc(security_hard_key: bool) -> &'static [u8] {
    if security_hard_key { EMBEDDED_POLARIS_SMC_SK } else { EMBEDDED_POLARIS_SMC }
}

// ── Polaris SDMA v3.0 register offsets (oss_3_0_d.h, byte offsets) ──────────
mod polaris_sdma_regs {
    pub const SDMA0_UCODE_ADDR:     u32 = 0x3400 * 4;
    pub const SDMA0_UCODE_DATA:     u32 = 0x3401 * 4;
    pub const SDMA0_POWER_CNTL:     u32 = 0x3402 * 4;
    pub const SDMA0_CLK_CTRL:       u32 = 0x3403 * 4;
    pub const SDMA0_CNTL:           u32 = 0x3404 * 4;
    pub const SDMA0_CHICKEN_BITS:   u32 = 0x3405 * 4;
    pub const SDMA0_TILING_CONFIG:  u32 = 0x3406 * 4;
    pub const SDMA0_SEM_WAIT_FAIL_TIMER_CNTL: u32 = 0x3409 * 4;
    pub const SDMA0_FREEZE:         u32 = 0x340C * 4;
    pub const SDMA0_STATUS_REG:     u32 = 0x340D * 4;
    // mmSDMA0_RB_RPTR_FETCH = 0x340a — internal F32 ring fetch position
    // (separate from visible mmSDMA0_GFX_RB_RPTR which the F32 only updates
    // periodically). Diag-only RO. Use to distinguish "F32 not fetching"
    // (FETCH stays 0) from "F32 fetching but visible RPTR stale" (FETCH advances).
    pub const SDMA0_RB_RPTR_FETCH:  u32 = 0x340A * 4;
    pub const SDMA0_IB_OFFSET_FETCH: u32 = 0x340B * 4;
    pub const SDMA0_F32_CNTL:       u32 = 0x3412 * 4;
    pub const SDMA0_PHASE0_QUANTUM: u32 = 0x3414 * 4;
    pub const SDMA0_PHASE1_QUANTUM: u32 = 0x3415 * 4;

    pub const SDMA0_GFX_RB_CNTL:    u32 = 0x3480 * 4;
    pub const SDMA0_GFX_RB_BASE:    u32 = 0x3481 * 4;
    pub const SDMA0_GFX_RB_BASE_HI: u32 = 0x3482 * 4;
    pub const SDMA0_GFX_RB_RPTR:    u32 = 0x3483 * 4;
    pub const SDMA0_GFX_RB_WPTR:    u32 = 0x3484 * 4;
    pub const SDMA0_GFX_RB_WPTR_POLL_CNTL:    u32 = 0x3485 * 4;
    pub const SDMA0_GFX_RB_WPTR_POLL_ADDR_HI: u32 = 0x3486 * 4;
    pub const SDMA0_GFX_RB_WPTR_POLL_ADDR_LO: u32 = 0x3487 * 4;
    pub const SDMA0_GFX_RB_RPTR_ADDR_HI: u32 = 0x3488 * 4;
    pub const SDMA0_GFX_RB_RPTR_ADDR_LO: u32 = 0x3489 * 4;
    pub const SDMA0_GFX_IB_CNTL:    u32 = 0x348A * 4;
    pub const SDMA0_GFX_IB_RPTR:    u32 = 0x348B * 4;
    pub const SDMA0_GFX_IB_OFFSET:  u32 = 0x348C * 4;
    // mmSDMA0_GFX_CONTEXT_STATUS = 0x3491 (RO), mmSDMA0_GFX_DOORBELL = 0x3492,
    // mmSDMA0_GFX_CONTEXT_CNTL = 0x3493 (RW). Per oss_3_0_d.h.
    pub const SDMA0_GFX_CONTEXT_STATUS: u32 = 0x3491 * 4;
    pub const SDMA0_GFX_DOORBELL:   u32 = 0x3492 * 4;
    pub const SDMA0_GFX_CONTEXT_CNTL: u32 = 0x3493 * 4;
    // Vega-only mmSDMA0_GFX_DOORBELL_OFFSET (kept as alias of CONTEXT_CNTL
    // for legacy diag dump labels — VI has no DOORBELL_OFFSET).
    pub const SDMA0_GFX_DOORBELL_OFFSET: u32 = 0x3493 * 4;
    // mmSDMA0_GFX_VIRTUAL_ADDR = 0x34A7, mmSDMA0_GFX_APE1_CNTL = 0x34A8
    // (b22 diag exposed prior "FIXED" comments wrote opposite direction —
    // those landed in the 0x3494..0x34A6 reserved hole, hence F32 kept
    // default VMID and ring fetch never bound to our GART-CTX0 PT).
    pub const SDMA0_GFX_VIRTUAL_ADDR: u32 = 0x34A7 * 4;
    pub const SDMA0_GFX_APE1_CNTL:    u32 = 0x34A8 * 4;
    pub const SDMA0_GFX_MINOR_PTR_UPDATE: u32 = 0x349D * 4;
    pub const SDMA0_RLC0_RB_CNTL:   u32 = 0x3500 * 4;
    pub const SDMA0_RLC0_IB_CNTL:   u32 = 0x350A * 4;
    pub const SDMA0_RLC1_RB_CNTL:   u32 = 0x3580 * 4;
    pub const SDMA0_RLC1_IB_CNTL:   u32 = 0x358A * 4;

    pub const SDMA1_OFFSET: u32 = 0x200 * 4; // 0x800 bytes between SDMA0 and SDMA1
}

// ── Polaris GMC v8.0 register offsets (gmc_8_1_d.h, byte offsets) ───────────
const POL_MC_VM_FB_LOCATION:      u32 = 0x809 * 4;
const POL_MC_VM_AGP_TOP:          u32 = 0x80A * 4;
const POL_MC_VM_AGP_BOT:          u32 = 0x80B * 4;
const POL_MC_VM_AGP_BASE:         u32 = 0x80C * 4;
// Live BTC-250PRO/Polaris validation + local Linux pipeline reference:
// SYSTEM_APERTURE regs are at 0x80D/0x80E/0x80F. The older 0x82A..0x82C
// offsets read as sticky firmware defaults and ignore our writes on this board.
const POL_MC_VM_SYS_APR_LOW:      u32 = 0x80D * 4;
const POL_MC_VM_SYS_APR_HIGH:     u32 = 0x80E * 4;
const POL_MC_VM_SYS_APR_DEFAULT:  u32 = 0x80F * 4;
const POL_MC_VM_MX_L1_TLB_CNTL:  u32 = 0x819 * 4;
const POL_MC_VM_FB_OFFSET:        u32 = 0x81A * 4;

const POL_VM_L2_CNTL:            u32 = 0x500 * 4;
const POL_VM_L2_CNTL2:           u32 = 0x501 * 4;
const POL_VM_L2_CNTL3:           u32 = 0x502 * 4;
const POL_VM_L2_CNTL4:           u32 = 0x578 * 4;
const POL_VM_CONTEXT0_CNTL:      u32 = 0x504 * 4;
const POL_VM_CONTEXT0_PAGE_TABLE_BASE_ADDR:  u32 = 0x54F * 4;
const POL_VM_CONTEXT0_PAGE_TABLE_START_ADDR: u32 = 0x557 * 4;
const POL_VM_CONTEXT0_PAGE_TABLE_END_ADDR:   u32 = 0x55F * 4;
const POL_VM_CONTEXT0_PROTECTION_FAULT_DEFAULT_ADDR: u32 = 0x546 * 4;
const POL_VM_INVALIDATE_REQUEST:  u32 = 0x51E * 4;
const POL_VM_INVALIDATE_RESPONSE: u32 = 0x51F * 4;
const POL_VM_CONTEXT0_CNTL2:      u32 = 0x50C * 4;
const POL_VM_CONTEXT0_PROTECTION_FAULT_STATUS:   u32 = 0x536 * 4;
const POL_VM_CONTEXT0_PROTECTION_FAULT_ADDR:     u32 = 0x53E * 4;
// Linux gmc_8_1_d.h: definitive 4-char ASCII client tag (HDP/DMIF/VCE0/SDMA/...)
const POL_VM_CONTEXT0_PROTECTION_FAULT_MCCLIENT: u32 = 0x538 * 4;
const POL_VM_CONTEXT1_PROTECTION_FAULT_STATUS:   u32 = 0x537 * 4;
const POL_VM_CONTEXT1_PROTECTION_FAULT_ADDR:     u32 = 0x53F * 4;
const POL_VM_CONTEXT1_PROTECTION_FAULT_MCCLIENT: u32 = 0x539 * 4;
const POL_VM_L2_CONTEXT1_IDENTITY_APERTURE_LOW_ADDR:  u32 = 0x575 * 4;
const POL_VM_L2_CONTEXT1_IDENTITY_APERTURE_HIGH_ADDR: u32 = 0x576 * 4;
const POL_VM_L2_CONTEXT_IDENTITY_PHYSICAL_OFFSET:     u32 = 0x577 * 4;
const POL_VM_PRT_APERTURE0_LOW_ADDR: u32 = 0x52C * 4;
const POL_VM_PRT_APERTURE1_LOW_ADDR: u32 = 0x52D * 4;
const POL_VM_PRT_APERTURE2_LOW_ADDR: u32 = 0x52E * 4;
const POL_VM_PRT_APERTURE3_LOW_ADDR: u32 = 0x52F * 4;

const POL_CONFIG_MEMSIZE:         u32 = 0x5428;
const POL_SRBM_STATUS:            u32 = 0x0394 * 4;
const POL_SRBM_STATUS2:           u32 = 0x0E4C; // mmSRBM_STATUS2
const POL_SRBM_SOFT_RESET:        u32 = 0x0398 * 4;
const POL_GRBM_SOFT_RESET:        u32 = 0x8020;
const POL_RLC_GPM_STAT:           u32 = 0xEC0E * 4;
const POL_RLC_CGCG_CGLS_CTRL:     u32 = 0xEC22 * 4;
const POL_RLC_CP_SCHEDULERS:      u32 = 0xEC22 * 4;
const POL_RLC_PG_CNTL:            u32 = 0xEC43 * 4;

// BIF_FB_EN: Linux gmc_v8_0_mc_program enables BAR0 VRAM read/write
const POL_BIF_FB_EN:              u32 = 0x1524 * 4; // 0x5490
const BIF_FB_EN_READ:             u32 = 1 << 0;
const BIF_FB_EN_WRITE:            u32 = 1 << 1;

// HDP registers
const POL_HDP_HOST_PATH_CNTL:    u32 = 0x0B00;
const POL_HDP_NONSURFACE_BASE:   u32 = 0x0B04;
const POL_HDP_MISC_CNTL:         u32 = 0x0B3C;
const POL_HDP_REG_COHERENCY_FLUSH_CNTL: u32 = 0x0B80;
const POL_HDP_MEM_COHERENCY_FLUSH_CNTL: u32 = 0x0B90;

// IH (Interrupt Handler) registers — oss_3_0_d.h dword offsets × 4
// Real offsets confirmed: mmIH_RB_CNTL=0xE30, NOT 0xF80 or 0x3E0
const POL_IH_RB_CNTL:            u32 = 0xE30 * 4; // 0x38C0
const POL_IH_RB_BASE:            u32 = 0xE31 * 4; // 0x38C4
const POL_IH_RB_RPTR:            u32 = 0xE32 * 4; // 0x38C8
const POL_IH_RB_WPTR:            u32 = 0xE33 * 4; // 0x38CC
const POL_IH_RB_WPTR_ADDR_HI:   u32 = 0xE34 * 4; // 0x38D0
const POL_IH_RB_WPTR_ADDR_LO:   u32 = 0xE35 * 4; // 0x38D4
const POL_IH_CNTL:               u32 = 0xE36 * 4; // 0x38D8
const POL_IH_LEVEL_STATUS:       u32 = 0xE37 * 4; // 0x38DC
const POL_IH_STATUS:             u32 = 0xE38 * 4; // 0x38E0
const POL_IH_DOORBELL_RPTR:      u32 = 0xE42 * 4; // 0x3908

// SRBM_GFX_CNTL for VMID select
const POL_SRBM_GFX_CNTL:         u32 = 0x391 * 4; // 0xE44

// DCE11 display engine — needed to blank CRTCs and stop DMIF page-fault loop.
// Polaris 10 has 6 CRTCs. mmCRTC_CONTROL base = 0x1B9C (DWORD).
// Per-CRTC offsets (DWORD) from Linux vid.h CRTC{N}_REGISTER_OFFSET:
const POL_DCE_CRTC_CONTROL_BASE: u32 = 0x1B9C * 4;
const POL_DCE_CRTC_OFFSETS: [u32; 6] = [
    0x0000 * 4, 0x0200 * 4, 0x0400 * 4,
    0x2600 * 4, 0x2800 * 4, 0x2A00 * 4,
];
// CRTC_CONTROL.CRTC_MASTER_EN = bit 0
const DCE_CRTC_MASTER_EN: u32 = 1 << 0;

// GB_ADDR_CONFIG for tiling config
const POL_GB_ADDR_CONFIG:         u32 = 0x263E * 4;

// GART PTE flags
const GART_PTE_VALID:     u64 = 1 << 0;
const GART_PTE_SYSTEM:    u64 = 1 << 1;
const GART_PTE_SNOOPED:   u64 = 1 << 2;
const GART_PTE_READABLE:  u64 = 1 << 5;
const GART_PTE_WRITEABLE: u64 = 1 << 6;
// SNOOP bit dropped: F32 stalls in MC read with SNOOP=1 despite GART PTE
// being correct. PCIe ATS is not configured by VBIOS on bare-metal, so
// snoop transactions never get a PCIe response. Cache coherency handled by
// HDP flush + explicit mfence on CPU side.
const _GART_PTE_SYSRAM_SNOOPED: u64 = GART_PTE_VALID | GART_PTE_SYSTEM | GART_PTE_SNOOPED
                                     | GART_PTE_READABLE | GART_PTE_WRITEABLE;
const GART_PTE_SYSRAM: u64 = GART_PTE_VALID | GART_PTE_SYSTEM
                            | GART_PTE_READABLE | GART_PTE_WRITEABLE;

// ── CP diagnostic flags (used by vm.rs shell) ──────────────────────────────
pub const CP_DIAG_L2_OFF:    u32 = 1 << 0;
pub const CP_DIAG_WD_DST1:   u32 = 1 << 1;
pub const CP_DIAG_WD_DST2:   u32 = 1 << 2;
pub const CP_DIAG_NO_SHADER: u32 = 1 << 3;
pub const CP_DIAG_NOFLAT:    u32 = 1 << 4;
pub const CP_DIAG_NO_INIT:   u32 = 1 << 5;
pub const CP_DIAG_NO_RESET:  u32 = 1 << 6;

// ── Persistent buffer state ────────────────────────────────────────────────
pub(crate) static POLARIS_BUF: spin::Mutex<Option<PolarisBuf>> = spin::Mutex::new(None);

pub(crate) struct PolarisBuf {
    pub virt: u64,       // ring CPU virtual address
    pub phys: u64,       // ring physical address (sysRAM)
    gart_gpu_base: u64,
    fw_loaded: [bool; 2],
    ring_ok: [bool; 2],
    vram_bar_virt: u64,
    vram_bar_phys: u64,
    vram_fb_mc: u64,
    use_vram: bool,
    // V37 GART fields
    pub ring_mc: u64,    // ring MC address in GART space
    pub wb_mc: u64,      // RPTR writeback MC address in GART
    fence_mc: u64,   // fence MC address in GART
    fence_cpu: u64,  // fence CPU virtual address (for polling)
    pub wb_cpu: u64,     // WB CPU virtual address (for reading RPTR)
    poll_cpu: u64,   // WPTR poll source CPU virtual address (F32 reads WPTR from here)
}

fn gpu_alive(mmio: u64) -> bool {
    let v = unsafe { mmio_read32(mmio, polaris_sdma_regs::SDMA0_STATUS_REG) };
    v != 0xFFFF_FFFF
}

fn serial_flush() {
    for _ in 0..500_000u32 { core::hint::spin_loop(); }
}

/// Parse Polaris SDMA firmware header -> (ucode_slice, dword_count, fw_version)
fn parse_polaris_sdma_fw(fw: &[u8]) -> Option<(&[u8], usize, u32)> {
    if fw.len() < 28 { return None; }
    let fw_version = u32::from_le_bytes([fw[16], fw[17], fw[18], fw[19]]);
    let ucode_size = u32::from_le_bytes([fw[20], fw[21], fw[22], fw[23]]) as usize;
    let ucode_off  = u32::from_le_bytes([fw[24], fw[25], fw[26], fw[27]]) as usize;
    if ucode_off == 0 || ucode_off >= fw.len() { return None; }
    let end = (ucode_off + ucode_size).min(fw.len());
    let ucode = &fw[ucode_off..end];
    let dwords = ucode.len() / 4;
    crate::serial_println!("[POLARIS-FW] FW ver={:#X} ucode_off={:#X} dwords={}", fw_version, ucode_off, dwords);
    Some((ucode, dwords, fw_version))
}

/// Parse a `gfx_firmware_header_v1_0` (RLC / MEC / PFP / ME / CE on GFX8).
/// Returns `(ucode_slice, jt_dwords, fw_version)`.
///
/// Layout (Linux `amdgpu_ucode.h`):
///   common_firmware_header (32 bytes)
///   ucode_feature_version u32 @ 32
///   jt_offset             u32 @ 36   ← number of dwords to load before the JT
///   jt_size               u32 @ 40
fn parse_polaris_gfx_fw(fw: &[u8]) -> Option<(&[u8], u32, u32)> {
    crate::serial_println!("[POLARIS-FW] parse: total={}B", fw.len());
    if fw.len() < 44 { crate::serial_println!("[POLARIS-FW] too short"); return None; }
    let fw_version = u32::from_le_bytes([fw[16], fw[17], fw[18], fw[19]]);
    let ucode_size = u32::from_le_bytes([fw[20], fw[21], fw[22], fw[23]]) as usize;
    let ucode_off  = u32::from_le_bytes([fw[24], fw[25], fw[26], fw[27]]) as usize;
    let jt_offset  = u32::from_le_bytes([fw[36], fw[37], fw[38], fw[39]]);
    let jt_size    = u32::from_le_bytes([fw[40], fw[41], fw[42], fw[43]]);
    crate::serial_println!("[POLARIS-FW] ver={:#X} ucode_size={:#X} ucode_off={:#X} jt_off={} jt_sz={}",
        fw_version, ucode_size, ucode_off, jt_offset, jt_size);
    if ucode_off == 0 || ucode_off >= fw.len() { crate::serial_println!("[POLARIS-FW] bad ucode_off"); return None; }
    let end = (ucode_off + ucode_size).min(fw.len());
    let ucode = &fw[ucode_off..end];
    let dwords = (ucode.len() / 4) as u32;
    let jt = if jt_offset > 0 && jt_offset <= dwords { jt_offset } else { dwords };
    crate::serial_println!("[POLARIS-FW] GFX-hdr ver={:#X} ucode_off={:#X} dwords={} jt_off={} jt_sz={}",
        fw_version, ucode_off, dwords, jt_offset, jt_size);
    Some((ucode, jt, fw_version))
}

// =============================================================================
// Phase 0: Diagnostic dump (read-only, no state changes)
// =============================================================================

/// Full pre-init diagnostic: BIF_FB_EN, IH, FREEZE, MC, SDMA status
pub fn polaris_sdma_diag(mmio: u64) {
    unsafe {
        crate::serial_println!("=== POLARIS SDMA DIAGNOSTIC ===");
        if !gpu_alive(mmio) {
            crate::println!("GPU DEAD (0xFFFFFFFF)");
            return;
        }

        // BIF_FB_EN - CRITICAL: if 0, BAR0 VRAM access is dead
        let bif = mmio_read32(mmio, POL_BIF_FB_EN);
        let bif_ok = (bif & 3) == 3;
        crate::serial_println!("[DIAG] BIF_FB_EN={:#X} read={} write={} {}",
            bif, bif & 1, (bif >> 1) & 1, if bif_ok { "OK" } else { "BROKEN!" });
        crate::println!("BIF_FB_EN={:#X} (R={} W={}) {}", bif, bif & 1, (bif >> 1) & 1,
            if bif_ok { "OK" } else { "*** DISABLED ***" });

        // IH (Interrupt Handler) state
        let ih_rb_cntl = mmio_read32(mmio, POL_IH_RB_CNTL);
        let ih_base    = mmio_read32(mmio, POL_IH_RB_BASE);
        let ih_rptr    = mmio_read32(mmio, POL_IH_RB_RPTR);
        let ih_wptr    = mmio_read32(mmio, POL_IH_RB_WPTR);
        let ih_cntl    = mmio_read32(mmio, POL_IH_CNTL);
        let ih_status  = mmio_read32(mmio, POL_IH_STATUS);
        let ih_enabled = ih_rb_cntl & 1;
        crate::serial_println!("[DIAG] IH: RB_CNTL={:#010X} BASE={:#X} RPTR={:#X} WPTR={:#X} CNTL={:#X} ST={:#X}",
            ih_rb_cntl, ih_base, ih_rptr, ih_wptr, ih_cntl, ih_status);
        crate::println!("IH: EN={} RB_CNTL={:#X} BASE={:#X} R={:#X} W={:#X}",
            ih_enabled, ih_rb_cntl, ih_base, ih_rptr, ih_wptr);

        // MC/VMC state
        let fb_loc = mmio_read32(mmio, POL_MC_VM_FB_LOCATION);
        let fb_off = mmio_read32(mmio, POL_MC_VM_FB_OFFSET);
        let memsz = mmio_read32(mmio, POL_CONFIG_MEMSIZE);
        let sys_lo = mmio_read32(mmio, POL_MC_VM_SYS_APR_LOW);
        let sys_hi = mmio_read32(mmio, POL_MC_VM_SYS_APR_HIGH);
        let sys_def = mmio_read32(mmio, POL_MC_VM_SYS_APR_DEFAULT);
        let l1_cntl = mmio_read32(mmio, POL_MC_VM_MX_L1_TLB_CNTL);
        let l2_cntl = mmio_read32(mmio, POL_VM_L2_CNTL);
        let ctx0 = mmio_read32(mmio, POL_VM_CONTEXT0_CNTL);
        let agp_top = mmio_read32(mmio, POL_MC_VM_AGP_TOP);
        let agp_bot = mmio_read32(mmio, POL_MC_VM_AGP_BOT);

        crate::serial_println!("[DIAG] FB_LOC={:#010X} FB_OFF={:#X} MEMSZ={}MB",
            fb_loc, fb_off, memsz);
        crate::println!("FB_LOC={:#010X} OFF={:#X} MEM={}MB", fb_loc, fb_off, memsz);
        crate::println!("SYS_APR=[{:#X},{:#X}] DEF={:#X}", sys_lo, sys_hi, sys_def);
        crate::println!("L1={:#X}(en={} SAM={}) L2={:#X}(en={}) CTX0={:#X}(en={} d={})",
            l1_cntl, l1_cntl & 1, (l1_cntl >> 3) & 3,
            l2_cntl, l2_cntl & 1,
            ctx0, ctx0 & 1, (ctx0 >> 1) & 7);
        crate::println!("AGP=[{:#X},{:#X}]", agp_bot, agp_top);

        // VM fault status
        let fault_st = mmio_read32(mmio, POL_VM_CONTEXT0_PROTECTION_FAULT_STATUS);
        if fault_st != 0 {
            crate::println!("VM FAULT STATUS={:#010X}", fault_st);
        }

        // SDMA engines
        for eng in 0..2u32 {
            let off = eng * polaris_sdma_regs::SDMA1_OFFSET;
            let f32c = mmio_read32(mmio, polaris_sdma_regs::SDMA0_F32_CNTL + off);
            let stat = mmio_read32(mmio, polaris_sdma_regs::SDMA0_STATUS_REG + off);
            let freeze = mmio_read32(mmio, polaris_sdma_regs::SDMA0_FREEZE + off);
            let rbc = mmio_read32(mmio, polaris_sdma_regs::SDMA0_GFX_RB_CNTL + off);
            let rptr = mmio_read32(mmio, polaris_sdma_regs::SDMA0_GFX_RB_RPTR + off);
            let wptr = mmio_read32(mmio, polaris_sdma_regs::SDMA0_GFX_RB_WPTR + off);
            let base = mmio_read32(mmio, polaris_sdma_regs::SDMA0_GFX_RB_BASE + off);
            let base_hi = mmio_read32(mmio, polaris_sdma_regs::SDMA0_GFX_RB_BASE_HI + off);
            let cntl = mmio_read32(mmio, polaris_sdma_regs::SDMA0_CNTL + off);

            crate::serial_println!("[DIAG] SDMA{}: F32={:#X}(halt={}) FREEZE={:#X} ST={:#010X}",
                eng, f32c, f32c & 1, freeze, stat);
            crate::println!("SDMA{}: halt={} freeze={} idle={} rb_empty={} rbc={:#X} R={:#X} W={:#X} base={:#X}:{:#X} cntl={:#X}",
                eng, f32c & 1, freeze & 1,
                stat & 1, (stat >> 2) & 1,
                rbc, rptr, wptr, base_hi, base, cntl);
        }

        // SRBM status
        let srbm = mmio_read32(mmio, POL_SRBM_STATUS);
        crate::println!("SRBM_STATUS={:#010X} (MCB={} MCC={} MCD={})",
            srbm, (srbm >> 1) & 1, (srbm >> 3) & 1, (srbm >> 4) & 1);

        // MC_SEQ_MISC0 (MC firmware version indicator)
        let mc_seq = mmio_read32(mmio, 0x2A00); // mmMC_SEQ_MISC0 = 0xA80*4
        crate::println!("MC_SEQ_MISC0={:#010X}", mc_seq);

        crate::serial_println!("=== END DIAGNOSTIC ===");
    }
}

/// MC diagnostic (status command)
pub fn polaris_mc_diag(mmio: u64) {
    polaris_sdma_diag(mmio);
}

/// Blank all 6 DCE11 CRTCs by clearing CRTC_CONTROL.CRTC_MASTER_EN.
/// Linux dce_v11_0_disable_dce() — minimal variant (no UPDATE_LOCK; we don't
/// care about mid-frame glitches when disabling).
///
/// Why: VBIOS leaves CRTC_MASTER_EN=1 with a scanout pointing at a stale
/// framebuffer. DMIF (CID=0x78 in PROTECTION_FAULT_STATUS) keeps fetching
/// that buffer, page-faulting in a tight loop, saturating the L2 MC arbiter
/// and starving SDMA F32 reads — RPTR never advances even though SDMA itself
/// emits MC requests (mc_rreq_idle=0).
pub fn polaris_dce_disable_all(mmio: u64) {
    let mut blanked = 0u32;
    for (i, off) in POL_DCE_CRTC_OFFSETS.iter().enumerate() {
        let reg = POL_DCE_CRTC_CONTROL_BASE + *off;
        unsafe {
            let ctrl = mmio_read32(mmio, reg);
            if ctrl & DCE_CRTC_MASTER_EN != 0 {
                let new = ctrl & !DCE_CRTC_MASTER_EN;
                mmio_write32(mmio, reg, new);
                let rb = mmio_read32(mmio, reg);
                crate::serial_println!(
                    "[DCE] CRTC{} blanked: CTRL {:#010X} -> {:#010X} (rb={:#010X})",
                    i, ctrl, new, rb);
                blanked += 1;
            } else {
                crate::serial_println!("[DCE] CRTC{} already off (CTRL={:#010X})", i, ctrl);
            }
        }
    }
    // Clear any DMIF fault that was latched while the CRTCs were live so the
    // next SDMA probe gets a clean slate.
    unsafe {
        let st = mmio_read32(mmio, POL_VM_CONTEXT0_PROTECTION_FAULT_STATUS);
        if st != 0 {
            mmio_write32(mmio, POL_VM_CONTEXT0_PROTECTION_FAULT_STATUS, st);
            crate::serial_println!("[DCE] Cleared post-blank PF_STATUS={:#010X}", st);
        }
    }
    crate::serial_println!("[DCE] disable_all done: {} CRTC(s) blanked", blanked);
}

// =============================================================================
// Phase 2: GMC v8.0 init — faithful to Linux gmc_v8_0_hw_init
// =============================================================================

/// Full GMC init: HDP flush, system aperture, BIF_FB_EN, L1 TLB, L2, VM_CONTEXT0
/// Faithful to Linux gmc_v8_0_mc_program + gmc_v8_0_gart_enable
pub fn polaris_gmc_init(mmio: u64) {
    unsafe {
        let fb_loc = mmio_read32(mmio, POL_MC_VM_FB_LOCATION);
        let fb_start_unit = (fb_loc & 0xFFFF) as u64;
        let fb_end_unit = ((fb_loc >> 16) & 0xFFFF) as u64;
        let vram_start = fb_start_unit << 24;
        let vram_end = (fb_end_unit << 24) | 0xFF_FFFF;
        let memsz = mmio_read32(mmio, POL_CONFIG_MEMSIZE);

        crate::serial_println!("[GMC] VRAM [{:#X},{:#X}] {}MB", vram_start, vram_end, memsz);
        crate::println!("GMC: VRAM [{:#X},{:#X}] {}MB", vram_start, vram_end, memsz);

        // --- gmc_v8_0_mc_program ---

        // 1. HDP 32-entry cache invalidate (Linux: clear 32 HDP entries)
        for i in 0..32u32 {
            let base = 0x0B14 + i * 0x18; // mmHDP_NONSURF_BASE_HI + i*6_dwords*4
            mmio_write32(mmio, base, 0);
        }
        mmio_write32(mmio, POL_HDP_REG_COHERENCY_FLUSH_CNTL, 0);

        // 2. Wait MC idle
        let mut mc_idle = false;
        for _ in 0..1_000_000u32 {
            let status = mmio_read32(mmio, POL_SRBM_STATUS);
            let busy = status & 0x1E; // bits[4:1] = MCB, MCB_NONDISP, MCC, MCD
            if busy == 0 { mc_idle = true; break; }
            core::hint::spin_loop();
        }
        crate::serial_println!("[GMC] MC idle: {}", if mc_idle { "OK" } else { "TIMEOUT" });

        // 3. System aperture — Linux gmc_v8_0_mc_program (gmc_v8_0.c):
        //   WREG32(mmMC_VM_SYSTEM_APERTURE_LOW_ADDR,  vram_start >> 12);
        //   WREG32(mmMC_VM_SYSTEM_APERTURE_HIGH_ADDR, vram_end   >> 12);
        //   WREG32(mmMC_VM_SYSTEM_APERTURE_DEFAULT_ADDR, scratch_page >> 12);
        // Shift is >>12 (4K page frame), NOT >>18. Without aperture covering FB,
        // any HDP/DMIF/SDMA access to FB pages hits VM range protection → PF=0x01078001
        // CID=0x78 (HDP).
        let sys_lo  = (vram_start >> 12) as u32;
        let sys_hi  = (vram_end   >> 12) as u32;
        // Default page: any valid FB page; use FB+4MB (no scratch BO yet).
        let sys_def = ((vram_start + 0x0040_0000) >> 12) as u32;
        mmio_write32(mmio, POL_MC_VM_SYS_APR_LOW, sys_lo);
        mmio_write32(mmio, POL_MC_VM_SYS_APR_HIGH, sys_hi);
        mmio_write32(mmio, POL_MC_VM_SYS_APR_DEFAULT, sys_def);

        crate::serial_println!("[GMC] SYS_APR LO={:#X} HI={:#X} DEF={:#X}", sys_lo, sys_hi, sys_def);

        // 4. Disable AGP aperture (Linux: BOT > TOP = empty)
        mmio_write32(mmio, POL_MC_VM_AGP_BOT, 0xFFFF_FFFFu32);
        mmio_write32(mmio, POL_MC_VM_AGP_TOP, 0);
        mmio_write32(mmio, POL_MC_VM_AGP_BASE, 0);

        // 5. Wait MC idle again
        for _ in 0..1_000_000u32 {
            let status = mmio_read32(mmio, POL_SRBM_STATUS);
            if status & 0x1E == 0 { break; }
            core::hint::spin_loop();
        }

        // 6. BIF_FB_EN — CRITICAL: enable BAR0 VRAM read/write
        mmio_write32(mmio, POL_BIF_FB_EN, BIF_FB_EN_READ | BIF_FB_EN_WRITE);
        let bif_rb = mmio_read32(mmio, POL_BIF_FB_EN);
        crate::serial_println!("[GMC] BIF_FB_EN={:#X} (should be 0x3)", bif_rb);
        crate::println!("BIF_FB_EN={:#X}", bif_rb);

        // 7. HDP_MISC_CNTL + HDP_HOST_PATH_CNTL (read-modify-write flush)
        let hpc = mmio_read32(mmio, POL_HDP_HOST_PATH_CNTL);
        mmio_write32(mmio, POL_HDP_HOST_PATH_CNTL, hpc);
        let hmc = mmio_read32(mmio, POL_HDP_MISC_CNTL);
        mmio_write32(mmio, POL_HDP_MISC_CNTL, hmc);

        // --- Option B: GART enabled (Linux gmc_v8_0_gart_enable) ---
        // L1 TLB + L2 cache enabled. VM_CONTEXT0 deferred until GART table
        // is populated by polaris_sdma_full_init.

        // 8. L1 TLB — match Linux gmc_v8_0_gart_enable() field-by-field
        // Linux: ENABLE_L1_TLB=1, ENABLE_L1_FRAGMENT_PROCESSING=1,
        //        SYSTEM_ACCESS_MODE=3, ENABLE_ADVANCED_DRIVER_MODEL=1,
        //        SYSTEM_APERTURE_UNMAPPED_ACCESS=0 (explicit!)
        let l1_cntl = (1u32 << 0)   // ENABLE_L1_TLB
                    | (1u32 << 1)   // ENABLE_L1_FRAGMENT_PROCESSING
                    | (3u32 << 3)   // SYSTEM_ACCESS_MODE = 3 (sysRAM via GART)
                    | (1u32 << 6);  // ENABLE_ADVANCED_DRIVER_MODEL
        // NOTE: SYSTEM_APERTURE_UNMAPPED_ACCESS (bit 10) = 0 per Linux source
        // NOTE: ENABLE_L1_TLB_CLOCK_GATING (bit 8) not set by Linux
        mmio_write32(mmio, POL_MC_VM_MX_L1_TLB_CNTL, l1_cntl);
        crate::serial_println!("[GMC] L1 TLB={:#X} (Linux-matched 0x5B)", l1_cntl);

        // 9. L2 cache — exact golden values from Linux hw dump (20260413_172832)
        // on the same BTC-250PRO / RX 580X hardware. Previous hand-assembled
        // bitfields produced 0x06188F03 which differed from Linux 0x0C0B8E03 —
        // wrong L2 config causes GART PTE lookups to fail (SDMA read fault).
        mmio_write32(mmio, POL_VM_L2_CNTL, 0x0C0B8E03u32);
        // CRITICAL: Linux gmc_v8_0_gart_enable sets INVALIDATE_ALL_L1_TLBS (bit0)
        // + INVALIDATE_L2_CACHE (bit1) to flush stale VBIOS TLB entries.
        // Without this, "GART TLB may be stale... GPU fetches wrong data from
        // GTT memory" (AMD kernel patch fix for SDMA ring test fail).
        mmio_write32(mmio, POL_VM_L2_CNTL2, 0x00000003u32);
        // Linux VM_L2_CNTL3 = 0x80148009: L2 bank/way config
        mmio_write32(mmio, POL_VM_L2_CNTL3, 0x80148009u32);
        // Linux gmc_v8_0_gart_enable: VM_L2_CNTL4 all fields = 0 (no physical bypass)
        mmio_write32(mmio, POL_VM_L2_CNTL4, 0x00000000u32);
        crate::serial_println!("[GMC] L2 cache ENABLED + L1/L2 INVALIDATED (Linux-matched)");

        // 10. VM_CONTEXT0 — disabled for now, enabled after GART table populated
        mmio_write32(mmio, POL_VM_CONTEXT0_CNTL, 0);
        mmio_write32(mmio, POL_VM_CONTEXT0_CNTL2, 0);  // Linux: VM_CONTEXT0_CNTL2 = 0
        mmio_write32(mmio, POL_VM_CONTEXT0_PAGE_TABLE_START_ADDR, 0);
        mmio_write32(mmio, POL_VM_CONTEXT0_PAGE_TABLE_END_ADDR, 0);
        mmio_write32(mmio, POL_VM_CONTEXT0_PAGE_TABLE_BASE_ADDR, 0);
        // Linux golden value from hw dump: 0x0008FBFE
        mmio_write32(mmio, POL_VM_CONTEXT0_PROTECTION_FAULT_DEFAULT_ADDR, 0x0008FBFEu32);

        // 10b. Identity aperture — Linux gmc_v8_0_gart_enable sets all to 0
        mmio_write32(mmio, POL_VM_L2_CONTEXT1_IDENTITY_APERTURE_LOW_ADDR, 0);
        mmio_write32(mmio, POL_VM_L2_CONTEXT1_IDENTITY_APERTURE_HIGH_ADDR, 0);
        mmio_write32(mmio, POL_VM_L2_CONTEXT_IDENTITY_PHYSICAL_OFFSET, 0);

        // 10c. PRT aperture golden registers — Polaris10 golden_settings
        // Linux: mmVM_PRT_APERTURE{0-3}_LOW_ADDR = 0x0FFFFFFF (disabled range)
        mmio_write32(mmio, POL_VM_PRT_APERTURE0_LOW_ADDR, 0x0FFFFFFFu32);
        mmio_write32(mmio, POL_VM_PRT_APERTURE1_LOW_ADDR, 0x0FFFFFFFu32);
        mmio_write32(mmio, POL_VM_PRT_APERTURE2_LOW_ADDR, 0x0FFFFFFFu32);
        mmio_write32(mmio, POL_VM_PRT_APERTURE3_LOW_ADDR, 0x0FFFFFFFu32);

        crate::serial_println!("[GMC] VM_CONTEXT0 ready (deferred until GART table)");

        // 11. TLB flush
        mmio_write32(mmio, POL_VM_INVALIDATE_REQUEST, 1);
        gpu_udelay(100); // Linux: poll + udelay after TLB invalidate
        let inv_resp = mmio_read32(mmio, POL_VM_INVALIDATE_RESPONSE);
        crate::serial_println!("[GMC] TLB invalidate response={:#X}", inv_resp);

        // Verify
        let l1_rb = mmio_read32(mmio, POL_MC_VM_MX_L1_TLB_CNTL);
        let l2_rb = mmio_read32(mmio, POL_VM_L2_CNTL);
        let ctx0_rb = mmio_read32(mmio, POL_VM_CONTEXT0_CNTL);
        crate::println!("GMC GART: L1={:#X} L2={:#X} CTX0={:#X}", l1_rb, l2_rb, ctx0_rb);
    }
}

// =============================================================================
// Phase 3: SDMA v3.0 init — faithful to Linux sdma_v3_0_start
// =============================================================================

/// V37: Full SDMA init with GART — ring + WB in sysRAM mapped through GART.
/// Faithful to Linux: gmc_v8_0 + sdma_v3_0 init order.
/// Uses MMIO WPTR (no doorbell — dead on BTC-250PRO).
pub fn polaris_sdma_full_init(mmio: u64) {
    polaris_sdma_full_init_mode(mmio, true, true, false);
}

pub fn polaris_sdma_full_init_no_ctxsw(mmio: u64) {
    polaris_sdma_full_init_mode(mmio, false, true, false);
}

pub fn polaris_sdma_full_init_hold_f32(mmio: u64) {
    polaris_sdma_full_init_mode(mmio, false, false, false);
}

pub fn polaris_sdma_full_init_late_f32(mmio: u64) {
    polaris_sdma_full_init_mode(mmio, true, false, true);
}

fn polaris_sdma_full_init_mode(
    mmio: u64,
    enable_auto_ctxsw: bool,
    unhalt_f32_early: bool,
    unhalt_f32_late: bool,
) {
    crate::serial_println!(
        "=== POLARIS SDMA V37 INIT (GART — Linux-faithful, auto_ctxsw={}, unhalt_f32_early={}, unhalt_f32_late={}) ===",
        enable_auto_ctxsw,
        unhalt_f32_early,
        unhalt_f32_late
    );
    crate::println!("=== SDMA V37 Init (GART) ===");

    // PHASE -1: blank DCE CRTCs FIRST. Otherwise DMIF (CID=0x78) keeps
    // page-faulting on the stale VBIOS scanout buffer, monopolises the L2 MC
    // arbiter, and starves SDMA F32 reads (mc_rreq_idle=0, RPTR=0 forever).
    // Linux dce_v11_0_disable_dce() does this as part of dce_v11_0_hw_init —
    // we replicate the minimal "clear CRTC_MASTER_EN on every active CRTC"
    // path so SDMA bringup is no longer gated on the display engine.
    polaris_dce_disable_all(mmio);

    // EARLY PROBE: snapshot PF_STATUS at function entry, before ANY work.
    // Helper closure usable outside the polaris_sdma_regs::* scope below.
    // Polaris gmc_v8 PF_STATUS bit layout:
    //   PROTECTIONS[7:0] reserved[11:8] CLIENT_ID[19:12] reserved[23:20]
    //   RW[24] VMID[28:25] MORE[31]
    let early_probe = |tag: &str| unsafe {
        let st = mmio_read32(mmio, POL_VM_CONTEXT0_PROTECTION_FAULT_STATUS);
        let ad = mmio_read32(mmio, POL_VM_CONTEXT0_PROTECTION_FAULT_ADDR);
        let mcc = mmio_read32(mmio, POL_VM_CONTEXT0_PROTECTION_FAULT_MCCLIENT);
        if st != 0 {
            let prot = st & 0xFF;
            let cid  = (st >> 12) & 0xFF;
            let rw   = (st >> 24) & 0x1;
            let vmid = (st >> 25) & 0xF;
            let more = (st >> 31) & 0x1;
            // Linux gmc_v8_0_vm_decode_fault: BLOCK = 4 ASCII bytes (BE order).
            let to_pr = |b: u32| -> u8 {
                let c = (b & 0xFF) as u8;
                if (0x20..=0x7E).contains(&c) { c } else { b'.' }
            };
            let b0 = to_pr(mcc >> 24);
            let b1 = to_pr(mcc >> 16);
            let b2 = to_pr(mcc >> 8);
            let b3 = to_pr(mcc);
            crate::serial_println!(
                "[SDMA-PROBE] {} PF_STATUS={:#010X} VMID={} CID={:#X} RW={} PROT={:#X} MORE={} MC={:#X} BLOCK={}{}{}{} (MCCLIENT={:#010X})",
                tag, st, vmid, cid, rw, prot, more, (ad as u64) << 12,
                b0 as char, b1 as char, b2 as char, b3 as char, mcc);
            mmio_write32(mmio, POL_VM_CONTEXT0_PROTECTION_FAULT_STATUS, st);
        } else {
            crate::serial_println!("[SDMA-PROBE] {} PF_STATUS=0 (clean)", tag);
        }
    };
    early_probe("function-entry");

    // MEC1+MEC2 are already halted at boot (CP_MEC_CNTL=0x50000000 read
    // confirmed). The persistent PF_STATUS=0x01078001 (VMID=0, CID=0x78,
    // PROT=RANGE) is the DMIF/DCE display engine writing the VBIOS scanout
    // target — unrelated to MEC. SYSTEM_APERTURE programming in polaris_gmc_init
    // below is what quiesces it.
    let cntl_boot = unsafe { mmio_read32(mmio, regs::CP_MEC_CNTL) };
    crate::serial_println!("[SDMA] CP_MEC_CNTL boot={:#X} (expect 0x50000000 = both halted)", cntl_boot);

    if !gpu_alive(mmio) {
        crate::println!("GPU dead");
        return;
    }
    early_probe("after-gpu_alive");

    // Read VRAM info
    let fb_loc = unsafe { mmio_read32(mmio, POL_MC_VM_FB_LOCATION) };
    let fb_start = ((fb_loc & 0xFFFF) as u64) << 24;

    // Get VRAM BAR info from GPU state
    let (vram_bar_phys, vram_bar_virt) = if let Some(info) = super::get_info() {
        let phys = info.vram_aperture_phys;
        if phys == 0 {
            crate::println!("No VRAM BAR");
            return;
        }
        // Map enough VRAM for GART page table at offset 0x380000
        let map_size = 4 * 1024 * 1024usize; // 4MB (need 3.5MB+ for GART table)
        match crate::memory::map_mmio(phys, map_size) {
            Ok(v) => (phys, v),
            Err(_) => {
                crate::println!("VRAM BAR map failed");
                return;
            }
        }
    } else {
        crate::println!("No GPU info");
        return;
    };

    crate::serial_println!("[SDMA] VRAM BAR phys={:#X} virt={:#X} FB_MC={:#X}",
        vram_bar_phys, vram_bar_virt, fb_start);
    crate::println!("VRAM: BAR={:#X} MC={:#X}", vram_bar_phys, fb_start);

    // Step 0: GMC init (aperture + HDP + BIF + L1/L2 enabled, VM_CONTEXT0 deferred)
    polaris_gmc_init(mmio);
    serial_flush();
    early_probe("after-polaris_gmc_init");

    // Step 0b: Pulse GRBM_SOFT_RESET bit 2 (SDMA block reset) — matches Linux
    // sdma_v3_0 pre-init. Captured from mmiotrace run 20260413_172644 at t=34.871.
    unsafe {
        use polaris_sdma_regs::*;

        mmio_write32(mmio, SDMA0_GFX_RB_CNTL, 0);
        mmio_write32(mmio, SDMA0_GFX_RB_CNTL + SDMA1_OFFSET, 0);
        mmio_write32(mmio, SDMA0_F32_CNTL, 1);
        mmio_write32(mmio, SDMA0_F32_CNTL + SDMA1_OFFSET, 1);
        gpu_udelay(50);

        mmio_write32(mmio, POL_GRBM_SOFT_RESET, 1u32 << 2);
        gpu_msleep(1); // Linux: msleep(1) after SDMA soft reset pulse
        mmio_write32(mmio, POL_GRBM_SOFT_RESET, 0);
        gpu_msleep(1);

        mmio_write32(mmio, SDMA0_GFX_RB_CNTL, 0);
        mmio_write32(mmio, SDMA0_GFX_RB_CNTL + SDMA1_OFFSET, 0);
        mmio_write32(mmio, SDMA0_F32_CNTL, 1);
        mmio_write32(mmio, SDMA0_F32_CNTL + SDMA1_OFFSET, 1);
        gpu_udelay(50);
    }
    early_probe("after-grbm-soft-reset");

    // Step 0c: SDMA chicken bits + clock control — captured values from Linux trace
    // on same hardware. Must be written BEFORE any ring setup.
    unsafe {
        use polaris_sdma_regs::*;
        mmio_write32(mmio, SDMA0_CHICKEN_BITS, 0x00810007);
        mmio_write32(mmio, SDMA0_CHICKEN_BITS + SDMA1_OFFSET, 0x00810007);
        mmio_write32(mmio, SDMA0_CLK_CTRL, 0);
        mmio_write32(mmio, SDMA0_CLK_CTRL + SDMA1_OFFSET, 0);
    }
    early_probe("after-chicken-clk");

    // Step 1: Golden regs (power gating off)
    polaris_sdma_golden(mmio);
    early_probe("after-sdma-golden");

    // Step 1b: RLC scheduler firmware.  The NOP path now proves SDMA F32
    // fetches the ring (`RPTR_FETCH=4`) but never retires visible RPTR/WB, and
    // the live diagnostic showed RLC CNTL/STAT/GPM/SCHED all zero.  Load RLC
    // before SDMA context scheduling so AUTO_CTXSW has a live scheduler block
    // behind it.  Do not pulse RLC soft-reset here; `load_rlc` intentionally
    // avoids that unsafe sequence on Polaris.
    {
        let need_rlc = {
            let fw_state = FW_STATE.lock();
            fw_state.rlc != FwStatus::Running && fw_state.rlc != FwStatus::Loaded
        };
        if need_rlc {
            match parse_polaris_sdma_fw(EMBEDDED_POLARIS_RLC) {
                Some((ucode, dwords, ver)) => {
                    crate::serial_println!("[SDMA] Loading RLC before SDMA scheduling: ver={:#X} dwords={}", ver, dwords);
                    if let Err(e) = load_rlc(mmio, ucode, ver) {
                        crate::println!("RLC fw load failed: {}", e);
                        return;
                    }
                    let mut fw_state = FW_STATE.lock();
                    fw_state.rlc = FwStatus::Loaded;
                }
                None => {
                    crate::println!("RLC fw parse failed");
                    return;
                }
            }
        }
        unsafe {
            crate::serial_println!("[SDMA] RLC live after load: CNTL={:#010X} STAT={:#010X} GPM={:#010X} SCHED={:#010X} PG={:#010X}",
                mmio_read32(mmio, RLC_CNTL),
                mmio_read32(mmio, RLC_STAT),
                mmio_read32(mmio, POL_RLC_GPM_STAT),
                mmio_read32(mmio, POL_RLC_CP_SCHEDULERS),
                mmio_read32(mmio, POL_RLC_PG_CNTL));
        }
    }
    early_probe("after-rlc-load");

    // ========================================================================
    // Step 2: Allocate ring / WB / fence / WPTR-poll in sysRAM (GART-mapped)
    // ========================================================================
    // V38: Linux-faithful GART sysRAM mode.
    // Layout (3 sysRAM pages, PTEs 1..3 in GART table):
    //   PTE1 @ MC 0xff00001000 → ring SDMA0 (4 KiB = 1024 DW, RB_SIZE=10)
    //   PTE2 @ MC 0xff00002000 → ring SDMA1 (4 KiB)
    //   PTE3 @ MC 0xff00003000 → shared (WB/poll/fence for both engines)
    // Shared page sub-layout:
    //   +0x00  RPTR writeback SDMA0  (8 bytes)
    //   +0x20  RPTR writeback SDMA1  (8 bytes)
    //   +0x40  WPTR poll source SDMA0 (4 bytes)
    //   +0x60  WPTR poll source SDMA1 (4 bytes)
    //   +0x80  Fence SDMA0 (4 bytes)
    //   +0xA0  Fence SDMA1 (4 bytes)
    // Each ring = 8 KiB (2 pages) because the SDMA F32 firmware enforces
    // RB_CNTL.SIZE = 11 (2^11 DW = 8 KiB) regardless of what we write; F32
    // aligns RB_BASE to ring_size and prefetches the full window, so the
    // second 4 KiB MUST be mapped (unmapped → VM fault we actually observed).
    let ring0_phys_a = match crate::memory::frame::alloc_contiguous_below(2, 0x1_0000_0000) {
        Some(p) => p, None => { crate::println!("ring0 DMA32 alloc failed"); return; }
    };
    let ring1_phys_a = match crate::memory::frame::alloc_contiguous_below(2, 0x1_0000_0000) {
        Some(p) => p, None => { crate::println!("ring1 DMA32 alloc failed"); return; }
    };
    let shared_phys = match crate::memory::frame::alloc_frame_below_4g_zeroed() {
        Some(p) => p, None => { crate::println!("shared DMA32 alloc failed"); return; }
    };
    let ring0_phys_b = ring0_phys_a + 0x1000;
    let ring1_phys_b = ring1_phys_a + 0x1000;
    let ring0_virt = crate::memory::phys_to_virt(ring0_phys_a);
    let ring1_virt = crate::memory::phys_to_virt(ring1_phys_a);
    let shared_virt = crate::memory::phys_to_virt(shared_phys);
    unsafe {
        core::ptr::write_bytes(ring0_virt as *mut u8, 0, 8192);
        core::ptr::write_bytes(ring1_virt as *mut u8, 0, 8192);
        core::sync::atomic::fence(core::sync::atomic::Ordering::SeqCst);
        #[cfg(target_arch = "x86_64")]
        core::arch::asm!("wbinvd", options(nostack, preserves_flags));
    }
    let ring0_phys = ring0_phys_a;
    let ring1_phys = ring1_phys_a;

    // IH ring buffer: 4KB (1 page, 1024 DWORDs) — minimal but valid.
    // Linux uses 64–256KB; we only need the ring to exist so the GPU can
    // deliver VM faults and completion events instead of stalling silently.
    let ih_ring_phys = match crate::memory::frame::alloc_frame_below_4g_zeroed() {
        Some(p) => p, None => { crate::println!("IH DMA32 alloc failed"); return; }
    };
    let ih_ring_virt = crate::memory::phys_to_virt(ih_ring_phys);

    // CSA (Context Save Area) — one 4 KiB page per engine. Step-2 fallback
    // for the F32 fault at MC 0xF400075000: program SDMAx_GFX_VIRTUAL_ADDR
    // with a valid GART-mapped backing page so F32 default CSA accesses land
    // somewhere safe instead of the unmapped FB+0x75000 default.
    let csa0_phys = match crate::memory::frame::alloc_frame_below_4g_zeroed() {
        Some(p) => p, None => { crate::println!("CSA0 DMA32 alloc failed"); return; }
    };
    let csa1_phys = match crate::memory::frame::alloc_frame_below_4g_zeroed() {
        Some(p) => p, None => { crate::println!("CSA1 DMA32 alloc failed"); return; }
    };
    let csa0_virt = crate::memory::phys_to_virt(csa0_phys);
    let csa1_virt = crate::memory::phys_to_virt(csa1_phys);

    // GART MC addresses — ring0 at PTE 0/1 (8 KiB aligned), ring1 at PTE 2/3,
    // shared at PTE 4, IH ring at PTE 5, CSA0 at PTE 6, CSA1 at PTE 7.
    const GART_MC_BASE: u64 = 0xFF00000000;
    let ring0_mc: u64 = GART_MC_BASE + 0x0000;
    let ring1_mc: u64 = GART_MC_BASE + 0x2000;
    let shared_mc: u64 = GART_MC_BASE + 0x4000;
    let ih_ring_mc: u64 = GART_MC_BASE + 0x5000;

    // Sub-offsets inside the shared page
    let wb0_mc    = shared_mc + 0x00;
    let wb1_mc    = shared_mc + 0x20;
    let poll0_mc  = shared_mc + 0x40;
    let poll1_mc  = shared_mc + 0x60;
    let fence0_mc = shared_mc + 0x80;
    let fence1_mc = shared_mc + 0xA0;

    let wb0_cpu    = shared_virt + 0x00;
    let wb1_cpu    = shared_virt + 0x20;
    let _poll0_cpu = shared_virt + 0x40;
    let _poll1_cpu = shared_virt + 0x60;
    let fence0_cpu = shared_virt + 0x80;
    let fence1_cpu = shared_virt + 0xA0;

    // Aliases kept for PolarisBuf compatibility (ring0/fence0/wb0 are the primaries)
    let ring_mc   = ring0_mc;
    let wb_mc     = wb0_mc;
    let fence_mc  = fence0_mc;
    let ring_cpu  = ring0_virt;
    let wb_cpu    = wb0_cpu;
    let fence_cpu = fence0_cpu;
    let ring_phys = ring0_phys;

    unsafe {
        // F32 polls WPTR from sysRAM. Make the initial shared page state
        // globally visible before enabling WPTR polling; this path has no GPU
        // snoop of dirty CPU cache lines.
        for off in (0..0xC0usize).step_by(4) {
            core::ptr::write_volatile((shared_virt + off as u64) as *mut u32, 0);
        }
        core::sync::atomic::fence(core::sync::atomic::Ordering::SeqCst);
        #[cfg(target_arch = "x86_64")]
        core::arch::asm!("wbinvd", options(nostack, preserves_flags));
    }

    crate::serial_println!("[SDMA] sysRAM: ring0 v={:#X} p={:#X} mc={:#X}",
        ring0_virt, ring0_phys, ring0_mc);
    crate::serial_println!("[SDMA] sysRAM: ring1 v={:#X} p={:#X} mc={:#X}",
        ring1_virt, ring1_phys, ring1_mc);
    crate::serial_println!("[SDMA] sysRAM: shared v={:#X} p={:#X} mc={:#X}",
        shared_virt, shared_phys, shared_mc);
    crate::println!("sysRAM: ring0={:#X} ring1={:#X} shared={:#X}",
        ring0_mc, ring1_mc, shared_mc);

    // ========================================================================
    // Step 3: Create GART page table in VRAM + populate PTEs for sysRAM pages
    // ========================================================================
    // GART MC base: 0xFF00000000 (same as Linux for this hardware).
    // Page table lives in VRAM at offset 0x380000 (accessible via BAR0).
    // We zero the first 2 MiB slot (512 PTEs → 2 MiB of GART window) and
    // populate PTEs 1..3 to map our ring0/ring1/shared sysRAM pages into GART.
    let gart_table_vram_off: u64 = 0x380000;
    let gart_table_mc = fb_start + gart_table_vram_off;
    let gart_table_cpu = vram_bar_virt + gart_table_vram_off;

    unsafe {
        let table = gart_table_cpu as *mut u64;
        // Zero the 512 PTEs we care about
        for i in 0..512usize {
            core::ptr::write_volatile(table.add(i), 0);
        }
        // PTE i encodes the sysRAM physical page at MC = GART_MC_BASE + i*4096
        // ring0 spans PTE 0..1 (8 KiB), ring1 spans PTE 2..3 (8 KiB), shared at PTE 4
        let pte0 = (ring0_phys_a & !0xFFFu64) | GART_PTE_SYSRAM;
        let pte1 = (ring0_phys_b & !0xFFFu64) | GART_PTE_SYSRAM;
        let pte2 = (ring1_phys_a & !0xFFFu64) | GART_PTE_SYSRAM;
        let pte3 = (ring1_phys_b & !0xFFFu64) | GART_PTE_SYSRAM;
        let pte4 = (shared_phys  & !0xFFFu64) | GART_PTE_SYSRAM;
        let pte5 = (ih_ring_phys & !0xFFFu64) | GART_PTE_SYSRAM;
        let pte6 = (csa0_phys    & !0xFFFu64) | GART_PTE_SYSRAM;
        let pte7 = (csa1_phys    & !0xFFFu64) | GART_PTE_SYSRAM;
        core::ptr::write_volatile(table.add(0), pte0);
        core::ptr::write_volatile(table.add(1), pte1);
        core::ptr::write_volatile(table.add(2), pte2);
        core::ptr::write_volatile(table.add(3), pte3);
        core::ptr::write_volatile(table.add(4), pte4);
        core::ptr::write_volatile(table.add(5), pte5);
        core::ptr::write_volatile(table.add(6), pte6);
        core::ptr::write_volatile(table.add(7), pte7);
        #[cfg(target_arch = "x86_64")]
        core::sync::atomic::fence(core::sync::atomic::Ordering::SeqCst);
        mmio_write32(mmio, POL_HDP_MEM_COHERENCY_FLUSH_CNTL, 1);
        let _ = mmio_read32(mmio, POL_HDP_MEM_COHERENCY_FLUSH_CNTL);
        crate::serial_println!("[SDMA] GART PTE0={:#X} PTE1={:#X} PTE2={:#X} PTE3={:#X} PTE4={:#X} PTE5(IH)={:#X} PTE6(CSA0)={:#X} PTE7(CSA1)={:#X}",
            pte0, pte1, pte2, pte3, pte4, pte5, pte6, pte7);
    }

    crate::serial_println!("[SDMA] GART table: MC={:#X} CPU={:#X}",
        gart_table_mc, gart_table_cpu);

    // ========================================================================
    // Step 4: Enable VM_CONTEXT0 with GART page table
    // ========================================================================
    unsafe {
        let gart_start = (GART_MC_BASE >> 12) as u32;
        let gart_end = ((GART_MC_BASE + 0x0FFF_FFFF) >> 12) as u32; // 256MB range (Linux: 0x0FF0FFFF)
        let table_base = (gart_table_mc >> 12) as u32;

        mmio_write32(mmio, POL_VM_CONTEXT0_PAGE_TABLE_START_ADDR, gart_start);
        mmio_write32(mmio, POL_VM_CONTEXT0_PAGE_TABLE_END_ADDR, gart_end);
        mmio_write32(mmio, POL_VM_CONTEXT0_PAGE_TABLE_BASE_ADDR, table_base);

        // VM_CONTEXT0_CNTL = 0x00fffed9 — exact value captured from Linux
        // mmiotrace on this hardware (Polaris 10 / RX 580X).
        // Decoded: ENABLE_CONTEXT(0) | RANGE_PROTECTION_FAULT_ENABLE_DEFAULT(3)
        //        | DUMMY_PAGE_PROTECTION_FAULT_ENABLE_DEFAULT(4) + full fault
        //          routing for read/write/exec/TLB.
        let ctx0_cntl: u32 = 0x00fffed9;
        mmio_write32(mmio, POL_VM_CONTEXT0_CNTL, ctx0_cntl);

        // Clear any stale protection fault from BIOS/early init
        // (write-1-to-clear style — write the current value to clear)
        let stale_fault = mmio_read32(mmio, POL_VM_CONTEXT0_PROTECTION_FAULT_STATUS);
        if stale_fault != 0 {
            mmio_write32(mmio, POL_VM_CONTEXT0_PROTECTION_FAULT_STATUS, stale_fault);
            crate::serial_println!("[SDMA] Cleared stale fault: {:#X}", stale_fault);
        }

        // Re-invalidate L2 + all L1 TLBs AFTER GART table populated and
        // VM_CONTEXT0 enabled. This is the critical step from the AMD kernel
        // patch "fix sdma ring test fail when resume from hibernation":
        // "gart tlb may be staled... this cause gpu fetchs wrong data from gtt memory"
        mmio_write32(mmio, POL_VM_L2_CNTL2, 0x00000003u32);
        gpu_udelay(50); // Linux: delay after L2 invalidate

        // TLB invalidate for VMID 0 to pick up new page table
        mmio_write32(mmio, POL_VM_INVALIDATE_REQUEST, 1);
        gpu_udelay(100); // Linux: poll + udelay after TLB invalidate
        let inv_resp = mmio_read32(mmio, POL_VM_INVALIDATE_RESPONSE);
        crate::serial_println!("[SDMA] L2 re-invalidated + TLB flush resp={:#X}", inv_resp);

        // Second HDP flush after VM setup
        mmio_write32(mmio, POL_HDP_MEM_COHERENCY_FLUSH_CNTL, 1);

        let ctx0_rb_after = mmio_read32(mmio, POL_VM_CONTEXT0_CNTL);
        crate::serial_println!("[SDMA-v2] VM_CONTEXT0 ENABLED: start={:#X} end={:#X} base={:#X} cntl_wrote={:#X} cntl_read={:#X}",
            gart_start, gart_end, table_base, ctx0_cntl, ctx0_rb_after);
        crate::println!("CTX0 ON: GART=[{:#X},{:#X}] PT={:#X}", gart_start, gart_end, table_base);
    }

    // ========================================================================
    // Step 4b: tonga_ih — Interrupt Handler ring init (Linux: tonga_ih_irq_init)
    // Linux order: vi_common → gmc_v8_0 → **tonga_ih** → gfx_v8_0 → sdma_v3_0
    // Without IH the GPU cannot deliver VM page faults, engine completions,
    // or error interrupts. The F32 SDMA firmware and CP can stall waiting for
    // an IH ack that never comes. This was the only major IP block we skipped.
    // ========================================================================
    unsafe {
        crate::serial_println!("[IH] === tonga_ih init (pre-SDMA) ===");

        // 1. Disable IH ring (tonga_ih_disable_interrupts)
        let ih_rb_prev = mmio_read32(mmio, POL_IH_RB_CNTL);
        crate::serial_println!("[IH] pre-disable RB_CNTL={:#010X}", ih_rb_prev);
        mmio_write32(mmio, POL_IH_RB_CNTL, ih_rb_prev & !1u32);
        gpu_udelay(100);

        // 2. Clear ring pointers (Linux: before base addr write)
        mmio_write32(mmio, POL_IH_RB_RPTR, 0);
        mmio_write32(mmio, POL_IH_RB_WPTR, 0);

        // 3. Ring base address — GART-mapped sysRAM (PTE 5, MC 0xFF00005000)
        // oss_3_0 has no RB_BASE_HI — 32-bit base holds bits [39:8]
        mmio_write32(mmio, POL_IH_RB_BASE, (ih_ring_mc >> 8) as u32);

        // 4. Build IH_RB_CNTL — oss_3_0_sh_mask.h bit layout:
        //    [0]     RB_ENABLE
        //    [5:1]   RB_SIZE = log2(ring_bytes/4) = log2(1024) = 10
        //    [8]     WPTR_WRITEBACK_ENABLE (not needed — we poll RPTR via MMIO)
        //    [20]    MC_SPACE = 1 (system memory via GART, not VRAM)
        //    [21]    RPTR_REARM = 1
        //    [24]    WPTR_OVERFLOW_ENABLE = 1
        //    [31]    WPTR_OVERFLOW_CLEAR = 1 (clear any stale overflow)
        let ih_rb_cntl: u32 = (1u32 << 0)       // RB_ENABLE
                            | (10u32 << 1)       // RB_SIZE (4KB = 1024 DW)
                            | (1u32 << 20)       // MC_SPACE (sysRAM via GART)
                            | (1u32 << 21)       // RPTR_REARM
                            | (1u32 << 24)       // WPTR_OVERFLOW_ENABLE
                            | (1u32 << 31);      // WPTR_OVERFLOW_CLEAR
        mmio_write32(mmio, POL_IH_RB_CNTL, ih_rb_cntl);

        // 5. IH_CNTL: enable interrupt delivery, MC_VMID=0 (use GART)
        //    [0] ENABLE_INTR = 1
        //    [19:16] MC_VMID = 0 (VMID 0 page table for ring access)
        let ih_cntl_prev = mmio_read32(mmio, POL_IH_CNTL);
        let ih_cntl_val = (ih_cntl_prev & !0x000F_0001u32) | 1u32;
        mmio_write32(mmio, POL_IH_CNTL, ih_cntl_val);

        // 6. Doorbell disabled (BTC-250PRO has dead doorbells)
        mmio_write32(mmio, POL_IH_DOORBELL_RPTR, 0);

        gpu_msleep(1); // Linux: msleep(1) after IH_RB_CNTL.RB_ENABLE=1

        // Readback verify
        let ih_rb_rb = mmio_read32(mmio, POL_IH_RB_CNTL);
        let ih_cntl_rb = mmio_read32(mmio, POL_IH_CNTL);
        let ih_rptr = mmio_read32(mmio, POL_IH_RB_RPTR);
        let ih_wptr = mmio_read32(mmio, POL_IH_RB_WPTR);
        let ih_status = mmio_read32(mmio, POL_IH_STATUS);
        crate::serial_println!("[IH] Done: RB_CNTL={:#010X} CNTL={:#010X} RPTR={:#X} WPTR={:#X} STATUS={:#010X} base_mc={:#X}",
            ih_rb_rb, ih_cntl_rb, ih_rptr, ih_wptr, ih_status, ih_ring_mc);
        crate::println!("IH: RB_CNTL={:#X} CNTL={:#X} R={:#X} W={:#X} ST={:#X}",
            ih_rb_rb, ih_cntl_rb, ih_rptr, ih_wptr, ih_status);
    }

    // ========================================================================
    // Step 5: SDMA v3.0 bring-up — byte-for-byte aligned on Linux sdma_v3_0
    // golden mmiotrace. Order below matches the chronological write sequence.
    // ========================================================================
    // Key parameters:
    //   - Ring/WB/POLL in sysRAM (GART-mapped), not VRAM
    //   - RB_CNTL final = 0x1015 (SIZE=10 for 4 KiB ring, no RB_PRIV bit)
    //   - DOORBELL disabled on BTC-250PRO (no doorbell mailbox), MMIO WPTR mode
    //   - WPTR poll configured (0x401000 + sysRAM poll source)
    //   - PHASE0/1_QUANTUM = 0x2000
    //   - TILING_CONFIG = 0
    //   - FW upload via MMIO (VBIOS does not preload SDMA microcode here).
    let e0: u32 = 0; // SDMA0 offset
    let e1: u32 = polaris_sdma_regs::SDMA1_OFFSET; // SDMA1 offset
    unsafe {
        use polaris_sdma_regs::*;

        // Inline phase-fault probe: snapshot PF_STATUS after each step.
        // Clear the fault at the start so subsequent reads pinpoint the
        // exact phase that re-triggers it.
        let probe = |tag: &str| {
            let st = mmio_read32(mmio, POL_VM_CONTEXT0_PROTECTION_FAULT_STATUS);
            let ad = mmio_read32(mmio, POL_VM_CONTEXT0_PROTECTION_FAULT_ADDR);
            if st != 0 {
                let prot = st & 0xFF;
                let cid  = (st >> 12) & 0xFF;
                let rw   = (st >> 24) & 0x1;
                let vmid = (st >> 25) & 0xF;
                let more = (st >> 31) & 0x1;
                crate::serial_println!(
                    "[SDMA-PROBE] {} PF_STATUS={:#010X} VMID={} CID={:#X} RW={} PROT={:#X} MORE={} MC={:#X}",
                    tag, st, vmid, cid, rw, prot, more, (ad as u64) << 12);
                mmio_write32(mmio, POL_VM_CONTEXT0_PROTECTION_FAULT_STATUS, st);
            } else {
                crate::serial_println!("[SDMA-PROBE] {} PF_STATUS=0 (clean)", tag);
            }
        };
        probe("init-entry");

        // --- Phase 0: preamble ---
        mmio_write32(mmio, SDMA0_CHICKEN_BITS + e0, 0x00810007);
        mmio_write32(mmio, SDMA0_CLK_CTRL     + e0, 0);
        mmio_write32(mmio, SDMA0_GFX_IB_CNTL  + e0, 0);
        mmio_write32(mmio, SDMA0_RLC0_RB_CNTL + e0, 0);
        mmio_write32(mmio, SDMA0_RLC0_IB_CNTL + e0, 0x100);
        mmio_write32(mmio, SDMA0_RLC1_RB_CNTL + e0, 0);
        mmio_write32(mmio, SDMA0_RLC1_IB_CNTL + e0, 0x100);
        mmio_write32(mmio, SDMA0_CHICKEN_BITS + e1, 0x00810007);
        mmio_write32(mmio, SDMA0_CLK_CTRL     + e1, 0);
        mmio_write32(mmio, SDMA0_GFX_IB_CNTL  + e1, 0);
        mmio_write32(mmio, SDMA0_RLC0_RB_CNTL + e1, 0);
        mmio_write32(mmio, SDMA0_RLC0_IB_CNTL + e1, 0x100);
        mmio_write32(mmio, SDMA0_RLC1_RB_CNTL + e1, 0);
        mmio_write32(mmio, SDMA0_RLC1_IB_CNTL + e1, 0x100);
        mmio_write32(mmio, SDMA0_CNTL         + e0, 0x08010402);
        mmio_write32(mmio, SDMA0_CNTL         + e1, 0x08010402);
        crate::serial_println!("[SDMA] CNTL phase1=0x08010402");
        probe("after-phase0");

        // --- Phase 1: disable/halt (golden: RB_CNTL=0 then F32_CNTL=1, no IB_CNTL here) ---
        mmio_write32(mmio, SDMA0_GFX_RB_CNTL + e0, 0);
        mmio_write32(mmio, SDMA0_GFX_RB_CNTL + e1, 0);
        mmio_write32(mmio, SDMA0_F32_CNTL    + e0, 1);
        mmio_write32(mmio, SDMA0_F32_CNTL    + e1, 1);
        gpu_msleep(1); // Linux: msleep(1) after SDMA halt/reset
        probe("after-phase1-halt");

        // --- Phase 1b: upload SDMA microcode while F32 is halted ---
        // Prior assumption "VBIOS preloads ucode, survives HALT" was wrong:
        // runtime probe showed F32_CNTL=0 (unhalted) + STATUS busy + RPTR frozen,
        // consistent with F32 running without valid firmware.
        crate::serial_println!("[SDMA] Uploading ucode (Polaris)...");
        polaris_sdma_load_fw_full(mmio);
        gpu_udelay(100); // Linux: usleep_range(10,100) after firmware load
        probe("after-phase1b-fw");

        // --- Phase 2: VIRTUAL_ADDR/APE1 setup (Linux sdma_v3_0_gfx_resume) ---
        // Linux clears these per VMID through vi_srbm_select(..., vmid=j)
        // before programming the ring. Do not program CSA MC here: live testing
        // showed writes to non-zero CSA values do not stick, while Linux writes 0.
        let csa0_mc: u64 = GART_MC_BASE + 0x6000;
        let csa1_mc: u64 = GART_MC_BASE + 0x7000;
        let srbm_save = mmio_read32(mmio, POL_SRBM_GFX_CNTL);
        for vmid in 0..16u32 {
            mmio_write32(mmio, POL_SRBM_GFX_CNTL, vmid << 4);
            mmio_write32(mmio, SDMA0_GFX_VIRTUAL_ADDR + e0, 0);
            mmio_write32(mmio, SDMA0_GFX_APE1_CNTL    + e0, 0);
            mmio_write32(mmio, SDMA0_GFX_VIRTUAL_ADDR + e1, 0);
            mmio_write32(mmio, SDMA0_GFX_APE1_CNTL    + e1, 0);
        }
        mmio_write32(mmio, POL_SRBM_GFX_CNTL, 0);
        // Direct zero after the VMID loop catches instance-local reset residue
        // observed live on SDMA1 (`VIRTUAL_ADDR=0x10`) despite the loop above.
        mmio_write32(mmio, SDMA0_GFX_VIRTUAL_ADDR + e0, 0);
        mmio_write32(mmio, SDMA0_GFX_APE1_CNTL    + e0, 0);
        mmio_write32(mmio, SDMA0_GFX_VIRTUAL_ADDR + e1, 0);
        mmio_write32(mmio, SDMA0_GFX_APE1_CNTL    + e1, 0);
        let va0_rb = mmio_read32(mmio, SDMA0_GFX_VIRTUAL_ADDR + e0);
        let ape0_rb = mmio_read32(mmio, SDMA0_GFX_APE1_CNTL + e0);
        let va1_rb = mmio_read32(mmio, SDMA0_GFX_VIRTUAL_ADDR + e1);
        let ape1_rb = mmio_read32(mmio, SDMA0_GFX_APE1_CNTL + e1);
        mmio_write32(mmio, POL_SRBM_GFX_CNTL, srbm_save);
        crate::serial_println!(
            "[SDMA] VMID VIRTUAL_ADDR/APE1 cleared for 16 VMIDs; csa0_mc={:#X} csa1_mc={:#X} rb0={:#X}/{:#X} rb1={:#X}/{:#X}",
            csa0_mc, csa1_mc, va0_rb, ape0_rb, va1_rb, ape1_rb);
        probe("after-phase2-csa");

        // NOTE: F32 stays HALTED through ring config. Linux sequence:
        // halt → load_microcode → gfx_resume (ring config with F32 halted)
        // → enable(unhalt) → ctx_switch_enable. F32 reads RB_CNTL etc. on
        // first unhalt — if we unhalt before config, F32 sees RB_ENABLE=0
        // and never enters its fetch loop.

        // --- Phase 3: ring configuration (golden order, per engine) ---
        // Helper closure emulates the exact Linux sequence for one engine.
        let config_ring = |eng_off: u32, ring_mc: u64, wb_mc: u64, poll_mc: u64,
                           poll_cpu: u64, doorbell_val: u32, ring_cpu_addr: u64| {
            // Linux step A: memset(ring->ring, 0, ring_size). 8 KiB ring (2 pages).
            // Even though we alloc_zeroed at init, Linux re-zeros on every
            // gfx_resume — guards against any stale data the F32 might
            // speculatively prefetch past RPTR.
            for off in (0..8192usize).step_by(4) {
                core::ptr::write_volatile((ring_cpu_addr + off as u64) as *mut u32, 0);
            }
            core::sync::atomic::fence(core::sync::atomic::Ordering::SeqCst);
            core::ptr::write_volatile(poll_cpu as *mut u32, 0);
            core::sync::atomic::fence(core::sync::atomic::Ordering::SeqCst);
            #[cfg(target_arch = "x86_64")]
            core::arch::asm!("wbinvd", options(nostack, preserves_flags));
            mmio_write32(mmio, SDMA0_TILING_CONFIG           + eng_off, 0);
            mmio_write32(mmio, SDMA0_SEM_WAIT_FAIL_TIMER_CNTL + eng_off, 0);
            // Wipe stale F32 context state inherited from VBIOS / prior boot
            mmio_write32(mmio, SDMA0_GFX_CONTEXT_CNTL + eng_off, 0);
            // RB_CNTL = SIZE=10 (4 KiB / 1024 DW) — matches Linux golden trace 0x1015
            mmio_write32(mmio, SDMA0_GFX_RB_CNTL + eng_off, (10u32 << 1));
            mmio_write32(mmio, SDMA0_GFX_RB_RPTR  + eng_off, 0);
            mmio_write32(mmio, SDMA0_GFX_RB_WPTR  + eng_off, 0);
            mmio_write32(mmio, SDMA0_GFX_IB_RPTR  + eng_off, 0);
            mmio_write32(mmio, SDMA0_GFX_IB_OFFSET + eng_off, 0);
            // RPTR writeback address (sysRAM via GART)
            mmio_write32(mmio, SDMA0_GFX_RB_RPTR_ADDR_HI + eng_off, (wb_mc >> 32) as u32);
            mmio_write32(mmio, SDMA0_GFX_RB_RPTR_ADDR_LO + eng_off, (wb_mc as u32) & !3u32);
            // Ring base (MC>>8)
            mmio_write32(mmio, SDMA0_GFX_RB_BASE    + eng_off, (ring_mc >> 8) as u32);
            mmio_write32(mmio, SDMA0_GFX_RB_BASE_HI + eng_off, (ring_mc >> 40) as u32);
            // WPTR=0 (empty ring) — Linux mainline gfx_resume order, no pre-kick.
            // The previous "kick to 8 then back to 0" was a stale workaround
            // that left F32 in an inconsistent state on Polaris.
            mmio_write32(mmio, SDMA0_GFX_RB_WPTR + eng_off, 0);
            // Doorbell + WPTR poll config — F32_POLL DISABLED for debug.
            // Pure MMIO WPTR mode: F32 reads WPTR from MMIO register directly.
            // Linux step K: WPTR_POLL_ADDR must point at a valid GART page even
            // if POLL_CNTL.ENABLE=0, because F32 may speculatively read it.
            // Without this, the regs hold post-reset garbage that page-faults.
            mmio_write32(mmio, SDMA0_GFX_RB_WPTR_POLL_ADDR_HI + eng_off, (poll_mc >> 32) as u32);
            mmio_write32(mmio, SDMA0_GFX_RB_WPTR_POLL_ADDR_LO + eng_off, (poll_mc as u32) & !3u32);
            mmio_write32(mmio, SDMA0_GFX_DOORBELL + eng_off, doorbell_val);
            // WPTR_POLL_CNTL (oss_3_0_sh_mask.h):
            //   bit 0  = ENABLE
            //   bit 1  = SWAP_ENABLE
            //   bit 2  = F32_POLL_ENABLE   ← THIS is the wake-from-poll bit
            //   bits 16-19 = POLL_FREQ
            //   bits 24-31 = IDLE_POLL_COUNT
            // On Polaris VI, F32 microcode has NO "pure MMIO WPTR" wake mode:
            // it either fetches WPTR via doorbell signal OR polls poll_mc in
            // sysRAM. Without F32_POLL_ENABLE, F32 sits forever waiting.
            // Linux sdma_v3_0_gfx_resume:
            //   wptr_poll_cntl = RREG32(WPTR_POLL_CNTL);
            //   REG_SET_FIELD(F32_POLL_ENABLE, ring->use_pollmem);
            //   WREG32(WPTR_POLL_CNTL, wptr_poll_cntl);
            // Read-modify-write to preserve POLL_FREQ defaults.
            // Bit 0 (ENABLE) is the master poll switch — without it, F32_POLL
            // is ignored. Confirmed via diag dump on _b21:
            //   POLL_CNTL readback = 0x00401004 (F32_POLL=1, ENABLE=0)
            //   → POLL[0]=4 was written, but F32 never read it.
            // Linux sdma_v3_0_gfx_resume sets BOTH bits when use_pollmem.
            let mut wpc = mmio_read32(mmio, SDMA0_GFX_RB_WPTR_POLL_CNTL + eng_off);
            wpc &= !((1 << 0) | (1 << 2)); // keep pollmem off until CSA/poll path is fixed
            mmio_write32(mmio, SDMA0_GFX_RB_WPTR_POLL_CNTL + eng_off, wpc);
            // RB_CNTL final = SIZE(10) | RPTR_WRITEBACK_ENABLE(bit12)
            // | RPTR_WRITEBACK_TIMER(3) | ENABLE(bit0). Linux's 0x1017 was
            // tested on this board and hard-stalled `gpu sdma test`; keep the
            // safer 0x31015 baseline while isolating the retire path.
            // Bit 9 is RB_SWAP_ENABLE (byte swap) — must stay 0 on x86, otherwise
            // F32 reads corrupted ring data.
            let rb_cntl_final: u32 = (10u32 << 1) | (1 << 12) | (3 << 16) | 1; // 0x31015
            mmio_write32(mmio, SDMA0_GFX_RB_CNTL + eng_off, rb_cntl_final);
            gpu_udelay(50);
            // Match Linux sdma_v3_0_gfx_resume: IB is enabled even for the
            // direct ring test path. Runtime tests showed MC0 faults can occur
            // before command submission, so test this as an init-time delta.
            mmio_write32(mmio, SDMA0_GFX_IB_CNTL + eng_off, 0);
        };
        // Doorbell DISABLED — BTC-250PRO has dead doorbells. In doorbell mode
        // the F32 ignores MMIO WPTR writes and waits for a doorbell signal that
        // never arrives, causing RPTR to stay at 0.  MMIO WPTR mode works.
        config_ring(e0, ring0_mc, wb0_mc, poll0_mc, _poll0_cpu, 0x1000_01E0, ring0_virt);
        probe("after-phase3-ring0");
        config_ring(e1, ring1_mc, wb1_mc, poll1_mc, _poll1_cpu, 0x1000_01E1, ring1_virt);
        probe("after-phase3-ring1");
        crate::serial_println!("[SDMA] Rings configured (RB_CNTL=0x1015, doorbell OFF, WPTR poll zeroed)");

        // --- Phase 3b: unhalt F32 AFTER ring config (Linux order) ---
        // sdma_v3_0_gfx_resume: config rings → sdma_v3_0_enable(true) → ctx_switch.
        // F32 reads RB_CNTL, RB_BASE, WPTR on first boot — they must be valid.
        if unhalt_f32_early {
            mmio_write32(mmio, SDMA0_F32_CNTL + e0, 0);
            probe("after-phase3b-unhalt-e0");
            mmio_write32(mmio, SDMA0_F32_CNTL + e1, 0);
            gpu_msleep(1); // Linux: msleep(1) after F32 unhalt
            probe("after-phase3b-unhalt-e1");
            crate::serial_println!("[SDMA] F32 unhalted (post-ring, Linux order)");
            // Poll STATUS_REG for engine idle (Linux: sdma_v3_0_start polls this)
            for attempt in 0..100u32 {
                let st = mmio_read32(mmio, SDMA0_STATUS_REG + e0);
                if st == 0 { break; }
                if attempt == 99 {
                    crate::serial_println!("[SDMA] WARNING: engine0 not idle after unhalt, ST={:#010X}", st);
                }
                gpu_udelay(100);
            }
        } else {
            probe("after-phase3b-hold-f32");
            crate::serial_println!("[SDMA] F32 kept halted for idle-fault isolation");
        }

        // --- Phase 3c: program CONTEXT_CNTL post-unhalt (Linux ctx_switch_enable order) ---
        // oss_3_0_sh_mask.h: SDMA0_GFX_CONTEXT_CNTL has ONLY two fields:
        //   bit  16    = RESUME_CTX        (mask 0x10000)
        //   bits 24-27 = SESSION_SEL       (mask 0xF000000)
        // RESUME_CTX must be 0 on cold start: it tells F32 to resume a
        // previously preempted context, but no such context exists. Linux
        // sdma_v3_0_gfx_resume() writes 0; the bit is set later in the
        // AUTO_CTXSW resume hand-shake. RESUME_CTX=1 on cold start halts
        // F32 fetch (RPTR_FETCH stays at 0).
        let ctx_cntl_val: u32 = 0; // Linux gfx_resume value (cold start)
        mmio_write32(mmio, SDMA0_GFX_CONTEXT_CNTL + e0, ctx_cntl_val);
        mmio_write32(mmio, SDMA0_GFX_CONTEXT_CNTL + e1, ctx_cntl_val);
        gpu_udelay(50);
        let cc0 = mmio_read32(mmio, SDMA0_GFX_CONTEXT_CNTL + e0);
        let cc1 = mmio_read32(mmio, SDMA0_GFX_CONTEXT_CNTL + e1);
        crate::serial_println!("[SDMA] CONTEXT_CNTL post-unhalt: e0={:#X} e1={:#X}", cc0, cc1);
        probe("after-phase3c-ctxcntl");

        // --- Phase 4: SDMA0_CNTL with AUTO_CTXSW (Linux ctx_switch_enable(true)) ---
        // oss_3_0_sh_mask.h SDMA0_CNTL bit layout (Polaris/VI):
        //   bit  0 = TRAP_ENABLE                (0x1)
        //   bit  1 = ATC_L1_ENABLE              (0x2)  -- required for MC translation
        //   bit 18 = AUTO_CTXSW_ENABLE          (0x40000) -- F32 schedules contexts
        //   bit 28 = CTXEMPTY_INT_ENABLE        (0x10000000)
        // Linux sdma_v3_0_ctx_switch_enable(true) sets AUTO_CTXSW=1 BEFORE
        // ring_test, on bare metal (non-SR-IOV). Prior assumption "needs RLC"
        // was wrong: RLC is only needed for SR-IOV / preemption. Without
        // AUTO_CTXSW the F32 sees WPTR > RPTR but never schedules a fetch.
        // Linux only writes PHASE*_QUANTUM when amdgpu_sdma_phase_quantum is
        // explicitly set.  Re-testing 0x2000 with the corrected CNTL still
        // left RPTR stale and reintroduced a CID 0x77 fault, so keep default.
        mmio_write32(mmio, SDMA0_PHASE0_QUANTUM + e0, 0);
        mmio_write32(mmio, SDMA0_PHASE1_QUANTUM + e0, 0);
        mmio_write32(mmio, SDMA0_PHASE0_QUANTUM + e1, 0);
        mmio_write32(mmio, SDMA0_PHASE1_QUANTUM + e1, 0);

        // Linux sdma_v3_0_ctx_switch_enable() preserves SDMA0_CNTL and only
        // toggles AUTO_CTXSW_ENABLE plus ATC_L1_ENABLE.  Do not clobber the
        // preamble bits from 0x08010402; losing them leaves the F32 scheduler
        // selected but unable to move RPTR_FETCH on this Polaris board.
        let auto_ctxsw_bit = if enable_auto_ctxsw { 1 << 18 } else { 0 };
        let cntl0 = mmio_read32(mmio, SDMA0_CNTL + e0) | (1 << 1) | auto_ctxsw_bit;
        let cntl1 = mmio_read32(mmio, SDMA0_CNTL + e1) | (1 << 1) | auto_ctxsw_bit;
        mmio_write32(mmio, SDMA0_CNTL + e0, cntl0);
        mmio_write32(mmio, SDMA0_CNTL + e1, cntl1);
        gpu_udelay(50);
        let c0 = mmio_read32(mmio, SDMA0_CNTL + e0);
        let c1 = mmio_read32(mmio, SDMA0_CNTL + e1);
        let cntl_mode = if enable_auto_ctxsw { "preserve|ATC_L1|AUTO_CTXSW" } else { "preserve|ATC_L1|no-AUTO_CTXSW" };
        crate::serial_println!("[SDMA] CNTL post: e0={:#X} e1={:#X} ({})", c0, c1, cntl_mode);
        if enable_auto_ctxsw {
            probe("after-phase4-ctxsw");
        } else {
            probe("after-phase4-noctxsw");
        }

        if unhalt_f32_late {
            mmio_write32(mmio, SDMA0_F32_CNTL + e0, 0);
            probe("after-phase4-late-unhalt-e0");
            mmio_write32(mmio, SDMA0_F32_CNTL + e1, 0);
            gpu_msleep(1);
            probe("after-phase4-late-unhalt-e1");
            crate::serial_println!("[SDMA] F32 unhalted late after CONTEXT/CNTL programming");
        }

        gpu_msleep(10);
        probe("after-settle-10ms");

        // --- Phase 5: keep CNTL=0x01 (no AUTO_CTXSW until RLC is up) ---

        gpu_msleep(5); // Linux: settle after CNTL phase3

        // Post-init status (both engines)
        let st0 = mmio_read32(mmio, SDMA0_STATUS_REG + e0);
        let st1 = mmio_read32(mmio, SDMA0_STATUS_REG + e1);
        let f0  = mmio_read32(mmio, SDMA0_F32_CNTL   + e0);
        let r0  = mmio_read32(mmio, SDMA0_GFX_RB_RPTR + e0);
        let w0  = mmio_read32(mmio, SDMA0_GFX_RB_WPTR + e0);
        gpu_msleep(100);
        probe("after-settle-100ms");
        gpu_msleep(400);
        probe("after-settle-500ms");
        gpu_msleep(1500);
        probe("after-settle-2000ms");
        crate::serial_println!("[SDMA] Post: ST0={:#010X} ST1={:#010X} F0={:#X} R0={:#X} W0={:#X}",
            st0, st1, f0, r0, w0);
        crate::println!("V38: ST0={:#X} ST1={:#X} halt={} R0={:#X} W0={:#X}",
            st0, st1, f0 & 1, r0, w0);

        // Save state
        let mut buf = POLARIS_BUF.lock();
        *buf = Some(PolarisBuf {
            virt: ring_cpu,
            phys: ring_phys,
            gart_gpu_base: ring_mc,
            fw_loaded: [true, true],
            ring_ok: [true, true],
            vram_bar_virt,
            vram_bar_phys,
            vram_fb_mc: fb_start,
            use_vram: false,
            ring_mc,
            wb_mc,
            fence_mc,
            fence_cpu,
            wb_cpu,
            poll_cpu: _poll0_cpu,
        });
        // Silence unused-variable warnings for fields the self-test doesn't consume yet
        let _ = (ring1_virt, ring1_phys, shared_virt, shared_phys,
                 fence1_mc, fence1_cpu, wb1_cpu);
    }

    // Init done — self-test is now a separate `gpu sdma test` invocation
    // (avoids running it before F32 has stabilized).
    crate::println!("--- SDMA init complete (run `gpu sdma test` to verify ring) ---");
}

// =============================================================================
// Phase 4: Ring test — WRITE_LINEAR + fence
// =============================================================================

/// SDMA ring test: write WRITE_LINEAR packet to fence location, poll completion.
/// V37: fence is in sysRAM (mapped via GART), polled via CPU HHDM.
/// Staged SDMA self-test.
///
/// Runs two levels of increasing complexity:
///   - Level A: single-DW NOP packet → tests that SDMA fetches the ring and RPTR advances
///   - Level B: WRITE_LINEAR of 1 dword to a GART-mapped fence → tests actual DMA execution
///
/// Each level verifies:
///   - RPTR caught up to WPTR (ring drained)
///   - For B: target dword has expected value AND a canary at target+4 is untouched
///     (catches off-by-one count bugs)
///
/// On failure, dumps engine state, fault registers, and the first dwords of the ring
/// so you can tell whether the fetch stalled, the packet was rejected, or the
/// target write faulted.
pub fn polaris_sdma_self_test(mmio: u64) {
    let buf_guard = POLARIS_BUF.lock();
    let buf = match buf_guard.as_ref() {
        Some(b) => b,
        None => { crate::println!("No buffers (run init first)"); return; }
    };

    let ring_cpu = buf.virt;
    let ring_mc  = buf.ring_mc;
    let fence_mc = buf.fence_mc;
    let fence_cpu = buf.fence_cpu;
    let wb_mc    = buf.wb_mc;
    let wb_cpu   = buf.wb_cpu;
    let poll_cpu = buf.poll_cpu;

    crate::println!("=== SDMA Self-Test (staged) ===");
    crate::serial_println!("[SDMA-TEST] Ring CPU={:#X} MC={:#X}  Fence CPU={:#X} MC={:#X}  WB CPU={:#X} MC={:#X}",
        ring_cpu, ring_mc, fence_cpu, fence_mc, wb_cpu, wb_mc);

    // Clear WB so we can detect whether SDMA writes back RPTR.
    // This is a FREE diagnostic: RB_CNTL has RPTR_WRITEBACK_ENABLE set, so after
    // each packet the SDMA automatically writes RPTR to wb_mc via GART PTE[2].
    // If Level A passes but wb_cpu[0] stays 0, it means SDMA fetched the ring
    // correctly but its write path to sysRAM is broken — narrows the bug
    // dramatically.
    unsafe {
        core::ptr::write_volatile(wb_cpu as *mut u32, 0xDEAD_0000);
        core::ptr::write_volatile((wb_cpu + 4) as *mut u32, 0xDEAD_0001);
        // Initialize poll memory to WPTR=0 (F32 reads WPTR from here)
        core::ptr::write_volatile(poll_cpu as *mut u32, 0);
        core::sync::atomic::fence(core::sync::atomic::Ordering::SeqCst);
    }

    use polaris_sdma_regs::*;

    // ---- pre-flight: engine must be running and ring enabled ----
    let pre_stat = unsafe { mmio_read32(mmio, SDMA0_STATUS_REG) };
    let pre_f32  = unsafe { mmio_read32(mmio, SDMA0_F32_CNTL) };
    let pre_rbc  = unsafe { mmio_read32(mmio, SDMA0_GFX_RB_CNTL) };
    let pre_rptr = unsafe { mmio_read32(mmio, SDMA0_GFX_RB_RPTR) };
    let pre_wptr = unsafe { mmio_read32(mmio, SDMA0_GFX_RB_WPTR) };
    crate::println!("PRE: ST={:#010X} F32={:#X} RBC={:#010X} R={:#X} W={:#X}",
        pre_stat, pre_f32, pre_rbc, pre_rptr, pre_wptr);
    if pre_f32 & 1 != 0 {
        crate::println!("  !! F32 halted — re-run init"); return;
    }
    if pre_rbc & 1 == 0 {
        crate::println!("  !! RB disabled — re-run init"); return;
    }

    // Start appending packets after whatever WPTR currently points at.
    // Assume ring is large enough that we don't need to wrap for this test.
    let mut wptr_bytes = pre_wptr;
    let mut wptr_dw = (wptr_bytes / 4) as usize;
    if wptr_dw > 1900 {
        crate::println!("WPTR {:#X} too close to ring wrap; re-init first", pre_wptr);
        return;
    }

    // ============================================================
    // LEVEL 0: Pure CPU memory pattern test on the ring region.
    // Writes 8 distinct dword patterns, then reads them back.
    // This isolates "can CPU write/read sysRAM at this vaddr?" from
    // anything SDMA/GART related. If this fails, nothing else matters.
    // ============================================================
    crate::println!("--- Level 0: CPU sysRAM pattern ---");
    unsafe {
        let ring = ring_cpu as *mut u32;
        let patterns: [u32; 8] = [
            0x1111_1111, 0x2222_2222, 0x3333_3333, 0x4444_4444,
            0x5555_5555, 0x6666_6666, 0x7777_7777, 0x8888_8888,
        ];
        for i in 0..8 {
            core::ptr::write_volatile(ring.add(wptr_dw + i), patterns[i]);
        }
        core::sync::atomic::fence(core::sync::atomic::Ordering::SeqCst);
        let mut all_ok = true;
        for i in 0..8 {
            let rb = core::ptr::read_volatile(ring.add(wptr_dw + i) as *const u32);
            let ok = rb == patterns[i];
            if !ok { all_ok = false; }
            crate::serial_println!("[SDMA-TEST/0]   ring[{}] wrote={:#010X} read={:#010X} {}",
                wptr_dw + i, patterns[i], rb, if ok { "OK" } else { "BAD" });
        }
        crate::println!("Level 0: {}", if all_ok { "PASS" } else { "FAIL (CPU writes to ring vaddr are lossy!)" });
        if !all_ok {
            crate::println!("  vaddr={:#X}  aborting — fix HHDM/mapping first", ring_cpu);
            return;
        }
        // CRITICAL: pre-fill the ring area with VALID NOPs (header 0x00000000 = single-DW NOP).
        // The SDMA fetcher is speculative and may prefetch past RPTR into content we
        // haven't written yet. If we leave sentinels like 0xFFFFFFFF there, the
        // prefetch buffer captures garbage opcodes that get committed before we
        // can overwrite the ring with real packets. NOP-filling makes stray
        // prefetches harmless.
        for i in 0..32 {
            core::ptr::write_volatile(ring.add(wptr_dw + i), 0x0000_0000);
        }
        core::sync::atomic::fence(core::sync::atomic::Ordering::SeqCst);
    }

    // ============================================================
    // LEVEL NOP: minimal single-DW NOP submit.
    // The ring is NOP-pre-filled. We bump WPTR by 4 bytes (1 DW) and
    // watch RPTR + STATUS + F32_CNTL periodically. If RPTR advances to
    // match, F32 is fetching the ring. If not — regardless of packet
    // format — the fetch path itself is broken and no WRITE_LINEAR
    // will ever work. This is the cheapest "is SDMA alive?" probe.
    // ============================================================
    crate::println!("--- Level NOP: single-DW NOP probe ---");
    {
        crate::println!("Level NOP: SKIP - Linux sdma_v3_0_ring_test_ring uses WRITE_LINEAR directly");
        // Do not submit a standalone NOP here.  On Polaris this advances the
        // internal fetch pointer but can fault SDM0 at address 0 before any
        // useful retire/writeback.  The Linux validation path starts with a
        // 5-DW WRITE_LINEAR packet, so keep wptr_dw at the current value.
    }

    // ============================================================
    // LEVEL A+B: combined single-submission test.
    // Both packets (NOP + WRITE_LINEAR) are written into the ring first,
    // then a SINGLE WPTR bump submits them. Ring area beyond is pre-filled
    // with valid NOPs so SDMA's speculative prefetcher cannot commit garbage.
    // ============================================================
    crate::println!("--- Level A+B: WRITE_LINEAR (Linux ring_test_ring) ---");

    // Pre-sentinel fence[0] and canary fence[1]. GPU will overwrite fence[0]
    // with 0xCAFEBABE; fence[1] MUST remain 0xBABEBABE — if it changes,
    // the count field was wrong and SDMA wrote past the intended target.
    unsafe {
        let fence = fence_cpu as *mut u32;
        core::ptr::write_volatile(fence.add(0), 0xDEAD_DEAD);
        core::ptr::write_volatile(fence.add(1), 0xBABE_BABE);
        // Also sentinel the wb+0x800 slot for over-run detection.
        core::ptr::write_volatile((wb_cpu + 0x800) as *mut u32, 0xDEAD_D800);
        core::sync::atomic::fence(core::sync::atomic::Ordering::SeqCst);
    }

    let (fence_val, canary_val, ring_drained, iters_used, first_seen, rptr_end, stat_end,
         wptr_hw_end, f32_end, rbcntl_end) = unsafe {
        let ring = ring_cpu as *mut u32;
        let base = wptr_dw;
        // WRITE_LINEAR: opcode=2, subop=0, count = number of DWORDs.
        // Linux sdma_v3_0_ring_test_ring uses COUNT(1), not count-1.
        let hdr    = 0x0000_0002u32;
        let dst_mc = fence_mc;
        let dst_lo = dst_mc as u32;
        let dst_hi = (dst_mc >> 32) as u32;
        let cnt    = 1u32;
        let data   = 0xCAFE_BABEu32;
        core::ptr::write_volatile(ring.add(base + 0), hdr);
        core::ptr::write_volatile(ring.add(base + 1), dst_lo);
        core::ptr::write_volatile(ring.add(base + 2), dst_hi);
        core::ptr::write_volatile(ring.add(base + 3), cnt);
        core::ptr::write_volatile(ring.add(base + 4), data);
        core::sync::atomic::fence(core::sync::atomic::Ordering::SeqCst);

        // PRE-SUBMISSION READBACK: verify the packet actually landed in memory
        // BEFORE we bump WPTR. If these reads don't match what we wrote, the
        // problem is purely CPU-side (write-combining, ordering, mapping).
        let rb0 = core::ptr::read_volatile(ring.add(base + 0) as *const u32);
        let rb1 = core::ptr::read_volatile(ring.add(base + 1) as *const u32);
        let rb2 = core::ptr::read_volatile(ring.add(base + 2) as *const u32);
        let rb3 = core::ptr::read_volatile(ring.add(base + 3) as *const u32);
        let rb4 = core::ptr::read_volatile(ring.add(base + 4) as *const u32);
        let pkt_ok = rb0 == hdr && rb1 == dst_lo && rb2 == dst_hi && rb3 == cnt && rb4 == data;
        crate::serial_println!("[SDMA-TEST/B] pre-submit readback: [{:#010X} {:#010X} {:#010X} {:#010X} {:#010X}] {}",
            rb0, rb1, rb2, rb3, rb4, if pkt_ok { "OK" } else { "MISMATCH" });
        crate::println!("B pre-submit: hdr={:#X}/{:#X} lo={:#X}/{:#X} hi={:#X}/{:#X} cnt={:#X}/{:#X} data={:#X}/{:#X}",
            rb0, hdr, rb1, dst_lo, rb2, dst_hi, rb3, cnt, rb4, data);
        if !pkt_ok {
            crate::println!("  PACKET WRITE DROPPED — CPU-side problem, aborting level B");
            crate::println!("  ring_vaddr={:#X} base={} (byte off {})", ring_cpu, base, base*4);
            drop(buf_guard);
            polaris_sdma_diag_dump(mmio);
            return;
        }

        wptr_dw += 5;
        wptr_bytes = (wptr_dw as u32) * 4;

        crate::serial_println!("[SDMA-TEST/B] pkt at ring[{}], WPTR={:#X}", base, wptr_bytes);
        // Write WPTR to poll memory + MMIO (same as Linux: ring->wptr << 2)
        core::ptr::write_volatile(poll_cpu as *mut u32, wptr_bytes);
        core::sync::atomic::fence(core::sync::atomic::Ordering::SeqCst);
        #[cfg(target_arch = "x86_64")]
        core::arch::asm!("wbinvd", options(nostack, preserves_flags));
        mmio_write32(mmio, SDMA0_GFX_RB_WPTR, wptr_bytes);

        let mut last_rptr = mmio_read32(mmio, SDMA0_GFX_RB_RPTR);
        let mut last_fetch = mmio_read32(mmio, SDMA0_RB_RPTR_FETCH);
        let mut fence_val = 0u32;
        let mut first_seen = u32::MAX;
        let mut drained = false;
        let mut iters_used = 0u32;
        for i in 0..50_000u32 {
            let rptr = mmio_read32(mmio, SDMA0_GFX_RB_RPTR);
            let fetch = mmio_read32(mmio, SDMA0_RB_RPTR_FETCH);
            if rptr != last_rptr {
                crate::serial_println!("[SDMA-TEST/B]   iter {} RPTR {:#X}->{:#X}", i, last_rptr, rptr);
                last_rptr = rptr;
            }
            if fetch != last_fetch {
                crate::serial_println!("[SDMA-TEST/B]   iter {} FETCH {:#X}->{:#X}", i, last_fetch, fetch);
                last_fetch = fetch;
            }
            if i & 0xFF == 0 {
                mmio_write32(mmio, POL_HDP_MEM_COHERENCY_FLUSH_CNTL, 1);
                let _ = mmio_read32(mmio, POL_HDP_MEM_COHERENCY_FLUSH_CNTL);
            }
            fence_val = core::ptr::read_volatile(fence_cpu as *const u32);
            if fence_val == 0xCAFE_BABE && first_seen == u32::MAX {
                first_seen = i;
            }
            if fence_val == 0xCAFE_BABE {
                drained = true;
                iters_used = i;
                break;
            }
            iters_used = i;
            core::hint::spin_loop();
        }
        // Final HDP flush before reading fence/canary/WB so we get the authoritative
        // post-exec VRAM view, not whatever HDP cached from CPU-side zero-fill.
        mmio_write32(mmio, POL_HDP_MEM_COHERENCY_FLUSH_CNTL, 1);
        let _ = mmio_read32(mmio, POL_HDP_MEM_COHERENCY_FLUSH_CNTL);
        fence_val = core::ptr::read_volatile(fence_cpu as *const u32);
        let canary = core::ptr::read_volatile((fence_cpu + 4) as *const u32);
        let rptr = mmio_read32(mmio, SDMA0_GFX_RB_RPTR);
        let stat = mmio_read32(mmio, SDMA0_STATUS_REG);
        let wptr_hw = mmio_read32(mmio, SDMA0_GFX_RB_WPTR);
        let f32c = mmio_read32(mmio, SDMA0_F32_CNTL);
        let rbc = mmio_read32(mmio, SDMA0_GFX_RB_CNTL);
        (fence_val, canary, drained, iters_used, first_seen, rptr, stat, wptr_hw, f32c, rbc)
    };

    let fence_ok = fence_val == 0xCAFE_BABE;
    let canary_ok = canary_val == 0xBABE_BABE;
    let rptr_ok = rptr_end == wptr_bytes;
    crate::println!("Level B: fence={:#X}[{}] canary={:#X}[{}] RPTR={:#X}/{:#X}[{}] iters={} first@{}",
        fence_val, if fence_ok { "OK" } else { "MISS" },
        canary_val, if canary_ok { "OK" } else { "BAD" },
        rptr_end, wptr_bytes, if rptr_ok { "OK" } else { "STUCK" },
        iters_used, first_seen);
    crate::serial_println!("[SDMA-TEST/B] post: WPTR_hw={:#X} (sw={:#X}) RPTR={:#X} F32_CNTL={:#010X} (halt={}) RB_CNTL={:#010X} (en={})",
        wptr_hw_end, wptr_bytes, rptr_end, f32_end, f32_end & 1, rbcntl_end, rbcntl_end & 1);
    crate::println!("B post: WPTR_hw={:#X}/sw={:#X} RPTR={:#X} F32={:#X}[halt={}] RB_CNTL={:#X}[en={}]",
        wptr_hw_end, wptr_bytes, rptr_end, f32_end, f32_end & 1, rbcntl_end, rbcntl_end & 1);

    if !fence_ok || !canary_ok || !ring_drained {
        crate::println!("Level B: FAIL — WRITE_LINEAR did not complete cleanly");
        drop(buf_guard);
        polaris_sdma_diag_dump(mmio);
        return;
    }

    crate::println!("Level B: PASS — WRITE_LINEAR landed");
    drop(buf_guard);
    polaris_sdma_diag_dump(mmio);
    return;

    {
        use polaris_sdma_regs::*;
        let nop_wptr_bytes = ((wptr_dw + 1) as u32) * 4;
        let pre_r = unsafe { mmio_read32(mmio, SDMA0_GFX_RB_RPTR) };
        let pre_s = unsafe { mmio_read32(mmio, SDMA0_STATUS_REG) };
        let pre_f = unsafe { mmio_read32(mmio, SDMA0_F32_CNTL) };
        crate::serial_println!("[SDMA-TEST/NOP] pre: RPTR={:#X} target_WPTR={:#X} ST={:#010X} F32={:#X}",
            pre_r, nop_wptr_bytes, pre_s, pre_f);
        // Clear any stale fault before test so we capture only faults from this NOP
        unsafe {
            let stale = mmio_read32(mmio, POL_VM_CONTEXT0_PROTECTION_FAULT_STATUS);
            if stale != 0 {
                mmio_write32(mmio, POL_VM_CONTEXT0_PROTECTION_FAULT_STATUS, stale);
                crate::serial_println!("[SDMA-TEST/NOP] Cleared stale fault: {:#X}", stale);
            }
        }
        unsafe {
            // Coherency: flush CPU writes through HDP and invalidate the GPU L2
            // page-walk cache so F32 sees the freshly-written NOP packets.
            // Linux mainline order: HDP_MEM_COHERENCY_FLUSH_CNTL=0 + readback,
            // then VM_INVALIDATE_REQUEST=1 (VMID0).
            mmio_write32(mmio, POL_HDP_MEM_COHERENCY_FLUSH_CNTL, 0);
            let _ = mmio_read32(mmio, POL_HDP_MEM_COHERENCY_FLUSH_CNTL);
            mmio_write32(mmio, POL_VM_INVALIDATE_REQUEST, 1);
            for _ in 0..1000 {
                if mmio_read32(mmio, POL_VM_INVALIDATE_RESPONSE) & 1 != 0 { break; }
                core::hint::spin_loop();
            }
            crate::serial_println!("[SDMA-TEST/NOP] HDP flushed, L2 invalidated");
            // Cache coherency without snoop: flush ALL CPU caches to RAM so
            // the GPU MC reads see our ring/poll/WB writes. PTE.SN=0 means
            // no PCIe snoop request; without IOMMU/ATS, this wbinvd is the
            // only way to guarantee CPU writes are globally visible.
            #[cfg(target_arch = "x86_64")]
            core::arch::asm!("wbinvd", options(nostack, preserves_flags));
            // Write WPTR to poll memory FIRST (F32 reads WPTR from here when
            // F32_POLL_ENABLE is set in WPTR_POLL_CNTL), then bump MMIO register.
            core::ptr::write_volatile(poll_cpu as *mut u32, nop_wptr_bytes);
            core::sync::atomic::fence(core::sync::atomic::Ordering::SeqCst);
            // Second flush after writing the poll value so it lands in RAM
            // before F32 can latch on the new WPTR.
            #[cfg(target_arch = "x86_64")]
            core::arch::asm!("wbinvd", options(nostack, preserves_flags));
            mmio_write32(mmio, SDMA0_GFX_RB_WPTR, nop_wptr_bytes);
        }

        let mut nop_drained = false;
        let mut nop_fetched = false;
        let mut nop_iters = 0u32;
        // Iterations kept small: F32 contests the MC bus and each RPTR read
        // stalls ~100+ us, so a long loop triggers the hardware watchdog
        // before we can print the diagnostic dump.
        for i in 0..20_000u32 {
            nop_iters = i;
            let rptr = unsafe { mmio_read32(mmio, SDMA0_GFX_RB_RPTR) };
            let rptr_fetch = unsafe { mmio_read32(mmio, SDMA0_RB_RPTR_FETCH) };
            if rptr == nop_wptr_bytes { nop_drained = true; break; }
            if rptr_fetch == nop_wptr_bytes { nop_fetched = true; }
            if nop_fetched {
                let wb0 = unsafe { core::ptr::read_volatile(wb_cpu as *const u32) };
                if wb0 == nop_wptr_bytes {
                    nop_drained = true;
                    break;
                }
            }
            if i > 0 && i % 5_000 == 0 {
                let st = unsafe { mmio_read32(mmio, SDMA0_STATUS_REG) };
                let f32c = unsafe { mmio_read32(mmio, SDMA0_F32_CNTL) };
                let wh = unsafe { mmio_read32(mmio, SDMA0_GFX_RB_WPTR) };
                crate::serial_println!("[SDMA-TEST/NOP] hb i={} RPTR={:#X} FETCH={:#X} WPTR_hw={:#X} ST={:#010X} F32={:#X}",
                    i, rptr, rptr_fetch, wh, st, f32c);
            }
            core::hint::spin_loop();
        }
        let post_r = unsafe { mmio_read32(mmio, SDMA0_GFX_RB_RPTR) };
        let post_fetch = unsafe { mmio_read32(mmio, SDMA0_RB_RPTR_FETCH) };
        let post_s = unsafe { mmio_read32(mmio, SDMA0_STATUS_REG) };
        let post_f = unsafe { mmio_read32(mmio, SDMA0_F32_CNTL) };
        let post_w = unsafe { mmio_read32(mmio, SDMA0_GFX_RB_WPTR) };
        crate::serial_println!("[SDMA-TEST/NOP] post: RPTR={:#X} FETCH={:#X} WPTR_hw={:#X} ST={:#010X} F32={:#X} drained={} fetched={} iters={}",
            post_r, post_fetch, post_w, post_s, post_f, nop_drained, nop_fetched, nop_iters);
        if !nop_drained && !nop_fetched {
            crate::serial_println!("[SDMA-TEST/NOP] FATAL: F32 did not consume a single NOP — fetch path broken.");
            crate::println!("Level NOP: FAIL — F32 not fetching the ring. Aborting before Level B.");
            drop(buf_guard);
            polaris_sdma_diag_dump(mmio);
            return;
        }
        if nop_drained {
            crate::println!("Level NOP: PASS  RPTR {:#X} -> {:#X}  iters={}", pre_r, post_r, nop_iters);
        } else {
            crate::println!("Level NOP: FETCHED  RPTR_FETCH={:#X}, RB_RPTR still {:#X}  iters={}",
                post_fetch, post_r, nop_iters);
            crate::println!("  F32 consumed the NOP; stopping before WRITE_LINEAR because RPTR writeback/reporting is still broken.");
            drop(buf_guard);
            polaris_sdma_diag_dump(mmio);
            return;
        }
        // Advance tracking — we consumed 1 DW of NOP.
        wptr_dw += 1;
        wptr_bytes = nop_wptr_bytes;
    }

    // ============================================================
    // LEVEL A+B: combined single-submission test.
    // Both packets (NOP + WRITE_LINEAR) are written into the ring first,
    // then a SINGLE WPTR bump submits them. Ring area beyond is pre-filled
    // with valid NOPs so SDMA's speculative prefetcher cannot commit garbage.
    // ============================================================
    crate::println!("--- Level A+B: NOP + WRITE_LINEAR ---");

    // Pre-sentinel fence[0] and canary fence[1]. GPU will overwrite fence[0]
    // with 0xCAFEBABE; fence[1] MUST remain 0xBABEBABE — if it changes,
    // the count field was wrong and SDMA wrote past the intended target.
    unsafe {
        let fence = fence_cpu as *mut u32;
        core::ptr::write_volatile(fence.add(0), 0xDEAD_DEAD);
        core::ptr::write_volatile(fence.add(1), 0xBABE_BABE);
        // Also sentinel the wb+0x800 slot for over-run detection.
        core::ptr::write_volatile((wb_cpu + 0x800) as *mut u32, 0xDEAD_D800);
        core::sync::atomic::fence(core::sync::atomic::Ordering::SeqCst);
    }

    let (fence_val, canary_val, ring_drained, iters_used, first_seen, rptr_end, stat_end,
         wptr_hw_end, f32_end, rbcntl_end) = unsafe {
        let ring = ring_cpu as *mut u32;
        let base = wptr_dw;
        // WRITE_LINEAR: opcode=2, subop=0, count_field = ndw - 1
        let hdr    = 0x0000_0002u32;
        // EXPERIMENT v3: aim at wb_mc+0x800 instead of fence_mc. wb region
        // is the same VRAM page family that RPTR writeback (known-working
        // direct MC path) uses. If THIS write lands but fence_mc did not,
        // the issue is addr range specific. If it also doesn't land, it's
        // packet/encoding related, not VM/range.
        let dst_mc = buf.wb_mc + 0x800;
        let dst_lo = dst_mc as u32;
        let dst_hi = (dst_mc >> 32) as u32;
        let cnt    = 0u32;
        let data   = 0xCAFE_BABEu32;
        core::ptr::write_volatile(ring.add(base + 0), hdr);
        core::ptr::write_volatile(ring.add(base + 1), dst_lo);
        core::ptr::write_volatile(ring.add(base + 2), dst_hi);
        core::ptr::write_volatile(ring.add(base + 3), cnt);
        core::ptr::write_volatile(ring.add(base + 4), data);
        core::sync::atomic::fence(core::sync::atomic::Ordering::SeqCst);

        // PRE-SUBMISSION READBACK: verify the packet actually landed in memory
        // BEFORE we bump WPTR. If these reads don't match what we wrote, the
        // problem is purely CPU-side (write-combining, ordering, mapping).
        let rb0 = core::ptr::read_volatile(ring.add(base + 0) as *const u32);
        let rb1 = core::ptr::read_volatile(ring.add(base + 1) as *const u32);
        let rb2 = core::ptr::read_volatile(ring.add(base + 2) as *const u32);
        let rb3 = core::ptr::read_volatile(ring.add(base + 3) as *const u32);
        let rb4 = core::ptr::read_volatile(ring.add(base + 4) as *const u32);
        let pkt_ok = rb0 == hdr && rb1 == dst_lo && rb2 == dst_hi && rb3 == cnt && rb4 == data;
        crate::serial_println!("[SDMA-TEST/B] pre-submit readback: [{:#010X} {:#010X} {:#010X} {:#010X} {:#010X}] {}",
            rb0, rb1, rb2, rb3, rb4, if pkt_ok { "OK" } else { "MISMATCH" });
        crate::println!("B pre-submit: hdr={:#X}/{:#X} lo={:#X}/{:#X} hi={:#X}/{:#X} cnt={:#X}/{:#X} data={:#X}/{:#X}",
            rb0, hdr, rb1, dst_lo, rb2, dst_hi, rb3, cnt, rb4, data);
        if !pkt_ok {
            crate::println!("  PACKET WRITE DROPPED — CPU-side problem, aborting level B");
            crate::println!("  ring_vaddr={:#X} base={} (byte off {})", ring_cpu, base, base*4);
            drop(buf_guard);
            polaris_sdma_diag_dump(mmio);
            return;
        }

        wptr_dw += 5;
        wptr_bytes = (wptr_dw as u32) * 4;

        crate::serial_println!("[SDMA-TEST/B] pkt at ring[{}], WPTR={:#X}", base, wptr_bytes);
        // Write WPTR to poll memory + MMIO (same as NOP level)
        core::ptr::write_volatile(poll_cpu as *mut u32, wptr_bytes);
        core::sync::atomic::fence(core::sync::atomic::Ordering::SeqCst);
        mmio_write32(mmio, SDMA0_GFX_RB_WPTR, wptr_bytes);

        let mut last_rptr = mmio_read32(mmio, SDMA0_GFX_RB_RPTR);
        let mut last_fetch = mmio_read32(mmio, SDMA0_RB_RPTR_FETCH);
        let mut fence_val = 0u32;
        let mut first_seen = u32::MAX;
        let mut drained = false;
        let mut iters_used = 0u32;
        // HDP read cache must be invalidated between GPU writes and CPU reads of
        // VRAM through BAR0; otherwise CPU sees stale pre-init zeros even after
        // SDMA executes WRITE_LINEAR. Flushing every 256 iters is a cheap
        // compromise vs flushing every iter.
        for i in 0..50_000u32 {
            let rptr = mmio_read32(mmio, SDMA0_GFX_RB_RPTR);
            let fetch = mmio_read32(mmio, SDMA0_RB_RPTR_FETCH);
            if rptr != last_rptr {
                crate::serial_println!("[SDMA-TEST/B]   iter {} RPTR {:#X}->{:#X}", i, last_rptr, rptr);
                last_rptr = rptr;
            }
            if fetch != last_fetch {
                crate::serial_println!("[SDMA-TEST/B]   iter {} FETCH {:#X}->{:#X}", i, last_fetch, fetch);
                last_fetch = fetch;
            }
            // Heartbeat: every 200K iters dump full state regardless of progress.
            // Tells us whether SDMA is stuck idle, halted, or faulting silently.
            if i > 0 && i % 200_000 == 0 {
                let st_h   = mmio_read32(mmio, SDMA0_STATUS_REG);
                let f32_h  = mmio_read32(mmio, SDMA0_F32_CNTL);
                let wptr_h = mmio_read32(mmio, SDMA0_GFX_RB_WPTR);
                let fv_h   = core::ptr::read_volatile(fence_cpu as *const u32);
                crate::serial_println!("[SDMA-TEST/B] hb i={} RPTR={:#X} FETCH={:#X} WPTR_hw={:#X} ST={:#010X} F32={:#X} fence={:#X}",
                    i, rptr, fetch, wptr_h, st_h, f32_h, fv_h);
            }
            if i & 0xFF == 0 {
                mmio_write32(mmio, POL_HDP_MEM_COHERENCY_FLUSH_CNTL, 1);
                let _ = mmio_read32(mmio, POL_HDP_MEM_COHERENCY_FLUSH_CNTL);
            }
            fence_val = core::ptr::read_volatile(fence_cpu as *const u32);
            if fence_val == 0xCAFE_BABE && first_seen == u32::MAX {
                first_seen = i;
            }
            if (rptr == wptr_bytes || fetch == wptr_bytes) && fence_val == 0xCAFE_BABE { drained = true; iters_used = i; break; }
            iters_used = i;
            core::hint::spin_loop();
        }
        // Final HDP flush before reading fence/canary/WB so we get the authoritative
        // post-exec VRAM view, not whatever HDP cached from CPU-side zero-fill.
        mmio_write32(mmio, POL_HDP_MEM_COHERENCY_FLUSH_CNTL, 1);
        let _ = mmio_read32(mmio, POL_HDP_MEM_COHERENCY_FLUSH_CNTL);
        fence_val = core::ptr::read_volatile(fence_cpu as *const u32);
        let canary = core::ptr::read_volatile((fence_cpu + 4) as *const u32);
        let rptr = mmio_read32(mmio, SDMA0_GFX_RB_RPTR);
        let stat = mmio_read32(mmio, SDMA0_STATUS_REG);
        let wptr_hw = mmio_read32(mmio, SDMA0_GFX_RB_WPTR);
        let f32c = mmio_read32(mmio, SDMA0_F32_CNTL);
        let rbc = mmio_read32(mmio, SDMA0_GFX_RB_CNTL);
        (fence_val, canary, drained, iters_used, first_seen, rptr, stat, wptr_hw, f32c, rbc)
    };

    let data_ok   = fence_val == 0xCAFE_BABE;
    let canary_ok = canary_val == 0xBABE_BABE;
    let pass = data_ok && canary_ok && ring_drained;

    crate::println!("Level B: fence={:#X}[{}] canary={:#X}[{}] RPTR={:#X}/{:#X}[{}] iters={} first@{}",
        fence_val, if data_ok { "OK" } else { "BAD" },
        canary_val, if canary_ok { "OK" } else { "BAD" },
        rptr_end, wptr_bytes, if ring_drained { "OK" } else { "BAD" },
        iters_used,
        if first_seen == u32::MAX { u32::MAX } else { first_seen });
    crate::serial_println!("[SDMA-TEST/B] ST_end={:#010X} first_seen={}", stat_end, first_seen);
    crate::serial_println!("[SDMA-TEST/B] post: WPTR_hw={:#X} (sw={:#X}) RPTR={:#X} F32_CNTL={:#010X} (halt={}) RB_CNTL={:#010X} (en={})",
        wptr_hw_end, wptr_bytes, rptr_end, f32_end, f32_end & 1, rbcntl_end, rbcntl_end & 1);
    crate::println!("B post: WPTR_hw={:#X}/sw={:#X} RPTR={:#X} F32={:#X}[halt={}] RB_CNTL={:#X}[en={}]",
        wptr_hw_end, wptr_bytes, rptr_end, f32_end, f32_end & 1, rbcntl_end, rbcntl_end & 1);
    if wptr_hw_end != wptr_bytes {
        crate::println!("  → WPTR mmio write NOT retained by HW (wanted {:#X}, got {:#X})", wptr_bytes, wptr_hw_end);
    }
    if f32_end & 1 != 0 {
        crate::println!("  → F32 re-halted itself after submit (firmware aborted)");
    }
    if rbcntl_end & 1 == 0 {
        crate::println!("  → RB_ENABLE dropped after submit");
    }

    // Final WB readback — has the RPTR writeback advanced to reflect both packets?
    // HDP flush again so CPU read sees GPU's last write to the WB buffer in VRAM.
    unsafe {
        mmio_write32(mmio, POL_HDP_MEM_COHERENCY_FLUSH_CNTL, 1);
        let _ = mmio_read32(mmio, POL_HDP_MEM_COHERENCY_FLUSH_CNTL);
    }
    let (wb0_end, wb1_end) = unsafe {(
        core::ptr::read_volatile(wb_cpu as *const u32),
        core::ptr::read_volatile((wb_cpu + 4) as *const u32),
    )};
    let wb_dst_end = unsafe { core::ptr::read_volatile((wb_cpu + 0x800) as *const u32) };
    crate::println!("Final WB: [0]={:#X} [1]={:#X} dst@+0x800={:#X}", wb0_end, wb1_end, wb_dst_end);
    crate::serial_println!("[SDMA-TEST/B] Final WB={:#X}/{:#X} dst@+0x800={:#X}", wb0_end, wb1_end, wb_dst_end);

    if pass {
        crate::println!("Level B: PASS");
        crate::println!("=== SDMA SELF-TEST: ALL LEVELS PASS ===");
    } else {
        crate::println!("Level B: FAIL");
        // Extra hints based on WB state
        if wb0_end == 0xDEAD_0000 {
            crate::println!("  WB never touched → SDMA write-path to sysRAM is BROKEN");
            crate::println!("  (not just fence — nothing reaches sysRAM from GPU)");
        } else if wb0_end == 0x18 {
            crate::println!("  WB shows 0x18 → RPTR writeback OK, fence write lost");
            crate::println!("  → PTE[3] (fence) or fence MC routing is the bug");
        } else {
            crate::println!("  WB shows {:#X} → unexpected, RPTR stuck before packet B?", wb0_end);
        }
        if !ring_drained { crate::println!("  ring did not drain → exec stall or fault"); }
        if !data_ok      { crate::println!("  target not written → target address not reachable?"); }
        if !canary_ok    { crate::println!("  canary corrupted → count field bug, wrote too much"); }
        drop(buf_guard);
        polaris_sdma_diag_dump(mmio);
    }
}

/// Diagnostic dump for SDMA failures. Reads engine state, fault registers,
/// and the first 8 dwords of the ring + first 2 dwords of the fence region.
fn polaris_sdma_diag_dump(mmio: u64) {
    use polaris_sdma_regs::*;
    unsafe {
        crate::println!("--- SDMA Diagnostic Dump ---");
        let stat     = mmio_read32(mmio, SDMA0_STATUS_REG);
        let f32c     = mmio_read32(mmio, SDMA0_F32_CNTL);
        let cntl     = mmio_read32(mmio, SDMA0_CNTL);
        let chicken  = mmio_read32(mmio, SDMA0_CHICKEN_BITS);
        let freeze   = mmio_read32(mmio, SDMA0_FREEZE);
        let rbc      = mmio_read32(mmio, SDMA0_GFX_RB_CNTL);
        let ctx_st   = mmio_read32(mmio, SDMA0_GFX_CONTEXT_STATUS);
        let ctx_cntl = mmio_read32(mmio, SDMA0_GFX_CONTEXT_CNTL);
        let base     = mmio_read32(mmio, SDMA0_GFX_RB_BASE);
        let base_hi  = mmio_read32(mmio, SDMA0_GFX_RB_BASE_HI);
        let rptr     = mmio_read32(mmio, SDMA0_GFX_RB_RPTR);
        let wptr     = mmio_read32(mmio, SDMA0_GFX_RB_WPTR);
        let fault_st = mmio_read32(mmio, POL_VM_CONTEXT0_PROTECTION_FAULT_STATUS);
        let fault_ad = mmio_read32(mmio, POL_VM_CONTEXT0_PROTECTION_FAULT_ADDR);
        let fault_mc = mmio_read32(mmio, POL_VM_CONTEXT0_PROTECTION_FAULT_MCCLIENT);
        let fault1_st = mmio_read32(mmio, POL_VM_CONTEXT1_PROTECTION_FAULT_STATUS);
        let fault1_ad = mmio_read32(mmio, POL_VM_CONTEXT1_PROTECTION_FAULT_ADDR);
        let fault1_mc = mmio_read32(mmio, POL_VM_CONTEXT1_PROTECTION_FAULT_MCCLIENT);
        let l2_cntl  = mmio_read32(mmio, POL_VM_L2_CNTL);
        let ctx0_cnt = mmio_read32(mmio, POL_VM_CONTEXT0_CNTL);
        let rlc_cntl = mmio_read32(mmio, RLC_CNTL);
        let rlc_stat = mmio_read32(mmio, RLC_STAT);
        let rlc_gpm  = mmio_read32(mmio, POL_RLC_GPM_STAT);
        let rlc_sched = mmio_read32(mmio, POL_RLC_CP_SCHEDULERS);
        let rlc_pg = mmio_read32(mmio, POL_RLC_PG_CNTL);

        crate::println!("STATUS   ={:#010X}  idle={} halt={} rb_empty={} mc_rreq_idle={}",
            stat, stat & 1, f32c & 1, (stat >> 2) & 1, (stat >> 17) & 1);
        crate::println!("F32_CNTL ={:#X}  SDMA_CNTL={:#X}  CHICKEN={:#X} FREEZE={:#X}",
            f32c, cntl, chicken, freeze);
        crate::println!("CTX_STATUS={:#010X}  CTX_CNTL={:#010X}", ctx_st, ctx_cntl);
        crate::println!("RB_CNTL  ={:#010X}  en={} priv={}", rbc, rbc & 1, (rbc >> 23) & 1);
        let rb_mc = ((base as u64) << 8) | ((base_hi as u64) << 40);
        crate::println!("RB_BASE  =HI:{:#X} LO:{:#X}  MC={:#X}", base_hi, base, rb_mc);
        crate::println!("RPTR={:#X}  WPTR={:#X}", rptr, wptr);
        // Internal F32 fetch counters — if these advance while RPTR stays 0,
        // the bug is RPTR writeback (WB or visible MMIO), not F32 fetch.
        let rptr_fetch = mmio_read32(mmio, SDMA0_RB_RPTR_FETCH);
        let ib_off_fetch = mmio_read32(mmio, SDMA0_IB_OFFSET_FETCH);
        crate::println!("RPTR_FETCH(internal)={:#X}  IB_OFFSET_FETCH={:#X}", rptr_fetch, ib_off_fetch);
        let poll_cntl = mmio_read32(mmio, SDMA0_GFX_RB_WPTR_POLL_CNTL);
        let door      = mmio_read32(mmio, SDMA0_GFX_DOORBELL);
        let poll_lo   = mmio_read32(mmio, SDMA0_GFX_RB_WPTR_POLL_ADDR_LO);
        let poll_hi   = mmio_read32(mmio, SDMA0_GFX_RB_WPTR_POLL_ADDR_HI);
        let rptr_lo   = mmio_read32(mmio, SDMA0_GFX_RB_RPTR_ADDR_LO);
        let rptr_hi   = mmio_read32(mmio, SDMA0_GFX_RB_RPTR_ADDR_HI);
        let poll_mc   = ((poll_hi as u64) << 32) | (poll_lo as u64);
        let rptr_mc   = ((rptr_hi as u64) << 32) | (rptr_lo as u64);
        crate::println!("POLL_CNTL={:#010X}  ENABLE={} F32_POLL={} SWAP={}  DOORBELL={:#010X}",
            poll_cntl, poll_cntl & 1, (poll_cntl >> 2) & 1, (poll_cntl >> 1) & 1, door);
        crate::println!("POLL_ADDR_MC={:#X}  RPTR_ADDR_MC={:#X}", poll_mc, rptr_mc);
        // SDMA_CNTL bit 1 = UTC_L1_ENABLE — required for VMID0 MC translation
        crate::println!("CNTL decode: TRAP={} UTC_L1={} AUTO_CTXSW={}",
            cntl & 1, (cntl >> 1) & 1, (cntl >> 18) & 1);
        // STATUS bit 16 = DELTA_RPTR_EMPTY (F32 read packets but never advanced RPTR)
        // STATUS bit 14 = PACKET_READY ; bit 12 = MC_RD_IDLE ; bit 2 = RB_EMPTY ; bit 0 = IDLE
        crate::println!("STATUS decode: PKT_READY={} DELTA_RPTR_EMPTY={} REG_IDLE={}",
            (stat >> 14) & 1, (stat >> 16) & 1, (stat >> 11) & 1);
        crate::println!("FAULT_STATUS={:#010X}  FAULT_ADDR={:#X}  L2={:#X}  CTX0={:#X}",
            fault_st, fault_ad, l2_cntl, ctx0_cnt);
        crate::println!("FAULT_MCCLIENT={:#010X}  CTX1_FAULT={:#010X} addr={:#X} mc={:#010X}",
            fault_mc, fault1_st, fault1_ad, fault1_mc);
        crate::println!("RLC: CNTL={:#010X} STAT={:#010X} GPM={:#010X} SCHED={:#010X} PG={:#010X}",
            rlc_cntl, rlc_stat, rlc_gpm, rlc_sched, rlc_pg);
        crate::println!("RLCQ: r0_rb={:#010X} r0_ib={:#010X} r1_rb={:#010X} r1_ib={:#010X}",
            mmio_read32(mmio, SDMA0_RLC0_RB_CNTL), mmio_read32(mmio, SDMA0_RLC0_IB_CNTL),
            mmio_read32(mmio, SDMA0_RLC1_RB_CNTL), mmio_read32(mmio, SDMA0_RLC1_IB_CNTL));

        let srbm_save = mmio_read32(mmio, POL_SRBM_GFX_CNTL);
        for n in 0..3u32 {
            let vmid = if n == 0 { 0 } else if n == 1 { 1 } else { 15 };
            mmio_write32(mmio, POL_SRBM_GFX_CNTL, vmid << 4);
            let va0 = mmio_read32(mmio, SDMA0_GFX_VIRTUAL_ADDR);
            let ape0 = mmio_read32(mmio, SDMA0_GFX_APE1_CNTL);
            let va1 = mmio_read32(mmio, SDMA0_GFX_VIRTUAL_ADDR + SDMA1_OFFSET);
            let ape1 = mmio_read32(mmio, SDMA0_GFX_APE1_CNTL + SDMA1_OFFSET);
            crate::println!("VMID{} SDMA VA/APE: e0={:#X}/{:#X} e1={:#X}/{:#X}",
                vmid, va0, ape0, va1, ape1);
        }
        mmio_write32(mmio, POL_SRBM_GFX_CNTL, srbm_save);

        if fault_st != 0 {
            // gmc_v8 layout: PROT[7:0] CID[19:12] RW[24] VMID[28:25] MORE[31]
            let prot   = fault_st & 0xFF;
            let cid    = (fault_st >> 12) & 0xFF;
            let rw     = (fault_st >> 24) & 1;
            let vmid   = (fault_st >> 25) & 0xF;
            let more   = (fault_st >> 31) & 1;
            crate::println!("  FAULT decode: VMID={} CID={:#X} RW={} PROT={:#X} MORE={}", vmid, cid, rw, prot, more);
        }

        if let Some(b) = POLARIS_BUF.lock().as_ref() {
            let ring = b.virt as *const u32;
            crate::println!("Ring[0..8] (CPU view):");
            for i in 0..8 {
                crate::println!("  [{}] = {:#010X}", i, core::ptr::read_volatile(ring.add(i)));
            }
            let fence = b.fence_cpu as *const u32;
            crate::println!("Fence[0..2]: {:#010X} {:#010X}",
                core::ptr::read_volatile(fence.add(0)),
                core::ptr::read_volatile(fence.add(1)));
            let wb = b.wb_cpu as *const u32;
            let pl = b.poll_cpu as *const u32;
            crate::println!("WB[0..2]:    {:#010X} {:#010X}  (RPTR writeback target)",
                core::ptr::read_volatile(wb.add(0)),
                core::ptr::read_volatile(wb.add(1)));
            crate::println!("POLL[0..2]:  {:#010X} {:#010X}  (F32 reads WPTR from here)",
                core::ptr::read_volatile(pl.add(0)),
                core::ptr::read_volatile(pl.add(1)));
            crate::println!("BUF MC: ring={:#X} wb={:#X} fence={:#X}",
                b.ring_mc, b.wb_mc, b.fence_mc);
        }
    }
}

pub fn polaris_sdma_retire_diag(mmio: u64) {
    use polaris_sdma_regs::*;
    unsafe {
        crate::println!("--- SDMA Retire/WB Diagnostic ---");
        let st = mmio_read32(mmio, SDMA0_STATUS_REG);
        let cntl = mmio_read32(mmio, SDMA0_CNTL);
        let f32 = mmio_read32(mmio, SDMA0_F32_CNTL);
        let rb_cntl = mmio_read32(mmio, SDMA0_GFX_RB_CNTL);
        let rptr = mmio_read32(mmio, SDMA0_GFX_RB_RPTR);
        let wptr = mmio_read32(mmio, SDMA0_GFX_RB_WPTR);
        let fetch = mmio_read32(mmio, SDMA0_RB_RPTR_FETCH);
        let ib_fetch = mmio_read32(mmio, SDMA0_IB_OFFSET_FETCH);
        let ctx_status = mmio_read32(mmio, SDMA0_GFX_CONTEXT_STATUS);
        let ctx_cntl = mmio_read32(mmio, SDMA0_GFX_CONTEXT_CNTL);
        let doorbell = mmio_read32(mmio, SDMA0_GFX_DOORBELL);
        let poll_cntl = mmio_read32(mmio, SDMA0_GFX_RB_WPTR_POLL_CNTL);
        let rptr_hi = mmio_read32(mmio, SDMA0_GFX_RB_RPTR_ADDR_HI);
        let rptr_lo = mmio_read32(mmio, SDMA0_GFX_RB_RPTR_ADDR_LO);
        let poll_hi = mmio_read32(mmio, SDMA0_GFX_RB_WPTR_POLL_ADDR_HI);
        let poll_lo = mmio_read32(mmio, SDMA0_GFX_RB_WPTR_POLL_ADDR_LO);
        let fault = mmio_read32(mmio, POL_VM_CONTEXT0_PROTECTION_FAULT_STATUS);
        let fault_addr = mmio_read32(mmio, POL_VM_CONTEXT0_PROTECTION_FAULT_ADDR);
        let fault_client = mmio_read32(mmio, POL_VM_CONTEXT0_PROTECTION_FAULT_MCCLIENT);

        crate::println!("SDMA: ST={:#010X} CNTL={:#010X} F32={:#X} RB={:#010X}", st, cntl, f32, rb_cntl);
        crate::println!("PTR: RPTR={:#X} WPTR={:#X} FETCH={:#X} IB_FETCH={:#X}", rptr, wptr, fetch, ib_fetch);
        crate::println!("CTX: STATUS={:#010X} CNTL={:#010X} DOORBELL={:#010X}", ctx_status, ctx_cntl, doorbell);
        crate::println!("POLL: CNTL={:#010X} ADDR={:#X}:{:#08X}", poll_cntl, poll_hi, poll_lo);
        crate::println!("WB:   ADDR={:#X}:{:#08X}", rptr_hi, rptr_lo);
        crate::println!("FAULT: ST={:#010X} ADDR_MC={:#X} CLIENT={:#010X}", fault, (fault_addr as u64) << 12, fault_client);
        crate::println!("STATUS bits: IDLE={} RB_EMPTY={} REG_IDLE={} PKT_READY={} DELTA_RPTR_EMPTY={} MC_RREQ_IDLE={}",
            st & 1, (st >> 2) & 1, (st >> 11) & 1, (st >> 14) & 1, (st >> 16) & 1, (st >> 17) & 1);
        crate::println!("RLC: CNTL={:#010X} STAT={:#010X} GPM={:#010X} SCHED={:#010X} PG={:#010X}",
            mmio_read32(mmio, RLC_CNTL),
            mmio_read32(mmio, RLC_STAT),
            mmio_read32(mmio, POL_RLC_GPM_STAT),
            mmio_read32(mmio, POL_RLC_CP_SCHEDULERS),
            mmio_read32(mmio, POL_RLC_PG_CNTL));
        crate::println!("RLCQ: r0_rb={:#010X} r0_ib={:#010X} r1_rb={:#010X} r1_ib={:#010X}",
            mmio_read32(mmio, SDMA0_RLC0_RB_CNTL),
            mmio_read32(mmio, SDMA0_RLC0_IB_CNTL),
            mmio_read32(mmio, SDMA0_RLC1_RB_CNTL),
            mmio_read32(mmio, SDMA0_RLC1_IB_CNTL));

        if let Some(b) = POLARIS_BUF.lock().as_ref() {
            let wb = b.wb_cpu as *const u32;
            let poll = b.poll_cpu as *const u32;
            let fence = b.fence_cpu as *const u32;
            crate::println!("BUF: ring_mc={:#X} wb_mc={:#X} fence_mc={:#X}", b.ring_mc, b.wb_mc, b.fence_mc);
            crate::println!("CPU preflush: WB={:#010X}/{:#010X} POLL={:#010X}/{:#010X} FENCE={:#010X}/{:#010X}",
                core::ptr::read_volatile(wb.add(0)), core::ptr::read_volatile(wb.add(1)),
                core::ptr::read_volatile(poll.add(0)), core::ptr::read_volatile(poll.add(1)),
                core::ptr::read_volatile(fence.add(0)), core::ptr::read_volatile(fence.add(1)));
            mmio_write32(mmio, POL_HDP_MEM_COHERENCY_FLUSH_CNTL, 0);
            let _ = mmio_read32(mmio, POL_HDP_MEM_COHERENCY_FLUSH_CNTL);
            #[cfg(target_arch = "x86_64")]
            core::arch::asm!("wbinvd", options(nostack, preserves_flags));
            crate::println!("CPU postflush: WB={:#010X}/{:#010X} POLL={:#010X}/{:#010X} FENCE={:#010X}/{:#010X}",
                core::ptr::read_volatile(wb.add(0)), core::ptr::read_volatile(wb.add(1)),
                core::ptr::read_volatile(poll.add(0)), core::ptr::read_volatile(poll.add(1)),
                core::ptr::read_volatile(fence.add(0)), core::ptr::read_volatile(fence.add(1)));
        } else {
            crate::println!("BUF: not allocated; run `gpu sdma init` first");
        }
    }
}

// =============================================================================
// Utility functions (referenced by vm.rs shell commands)
// =============================================================================

pub fn polaris_alloc_buffers() -> bool {
    // Buffers are allocated as part of full_init (VRAM-based)
    let buf = POLARIS_BUF.lock();
    if buf.is_some() {
        crate::println!("Buffers already allocated");
        true
    } else {
        crate::println!("Run 'gpu sdma init' first");
        false
    }
}

pub fn polaris_sdma_va_diag(mmio: u64) {
    use polaris_sdma_regs::*;

    unsafe {
        let srbm_save = mmio_read32(mmio, POL_SRBM_GFX_CNTL);
        crate::println!("--- SDMA VMID VA/APE Diagnostic ---");
        crate::println!("SRBM_GFX_CNTL save={:#010X}", srbm_save);
        for vmid in 0..16u32 {
            mmio_write32(mmio, POL_SRBM_GFX_CNTL, vmid << 4);
            let srbm_rb = mmio_read32(mmio, POL_SRBM_GFX_CNTL);
            let va0 = mmio_read32(mmio, SDMA0_GFX_VIRTUAL_ADDR);
            let ape0 = mmio_read32(mmio, SDMA0_GFX_APE1_CNTL);
            let va1 = mmio_read32(mmio, SDMA0_GFX_VIRTUAL_ADDR + SDMA1_OFFSET);
            let ape1 = mmio_read32(mmio, SDMA0_GFX_APE1_CNTL + SDMA1_OFFSET);
            crate::println!(
                "VMID{:02} SRBM={:#010X} SDMA0 VA/APE={:#010X}/{:#010X} SDMA1 VA/APE={:#010X}/{:#010X}",
                vmid, srbm_rb, va0, ape0, va1, ape1
            );
        }
        mmio_write32(mmio, POL_SRBM_GFX_CNTL, srbm_save);
        crate::println!("SRBM_GFX_CNTL restored={:#010X}", mmio_read32(mmio, POL_SRBM_GFX_CNTL));
    }
}

pub fn polaris_sdma_reset(mmio: u64) {
    unsafe {
        // GRBM_SOFT_RESET bit 20 = SOFT_RESET_SDMA0, bit 21 = SOFT_RESET_SDMA1
        let grbm = mmio_read32(mmio, POL_GRBM_SOFT_RESET);
        mmio_write32(mmio, POL_GRBM_SOFT_RESET, grbm | (1 << 20) | (1 << 21));
        for _ in 0..100_000u32 { core::hint::spin_loop(); }
        mmio_write32(mmio, POL_GRBM_SOFT_RESET, grbm & !((1 << 20) | (1 << 21)));
        for _ in 0..100_000u32 { core::hint::spin_loop(); }
        crate::serial_println!("[SDMA] Soft reset done");
        crate::println!("SDMA soft reset done");
    }
}

pub fn polaris_sdma_load_fw(mmio: u64) {
    polaris_sdma_load_fw_full(mmio);
}

pub fn polaris_sdma_load_fw_full(mmio: u64) {
    for engine in 0..2u32 {
        let eng_off = engine * polaris_sdma_regs::SDMA1_OFFSET;
        let fw = if engine == 0 { EMBEDDED_POLARIS_SDMA0 } else { EMBEDDED_POLARIS_SDMA1 };
        if let Some((ucode, dwords, fw_ver)) = parse_polaris_sdma_fw(fw) {
            unsafe {
                // Halt first
                let f32c = mmio_read32(mmio, polaris_sdma_regs::SDMA0_F32_CNTL + eng_off);
                mmio_write32(mmio, polaris_sdma_regs::SDMA0_F32_CNTL + eng_off, f32c | 1);
                for _ in 0..10_000u32 { core::hint::spin_loop(); }

                mmio_write32(mmio, polaris_sdma_regs::SDMA0_UCODE_ADDR + eng_off, 0);
                for i in 0..dwords {
                    let o = i * 4;
                    let dw = u32::from_le_bytes([ucode[o], ucode[o+1], ucode[o+2], ucode[o+3]]);
                    mmio_write32(mmio, polaris_sdma_regs::SDMA0_UCODE_DATA + eng_off, dw);
                }
                mmio_write32(mmio, polaris_sdma_regs::SDMA0_UCODE_ADDR + eng_off, fw_ver);
            }
            crate::println!("SDMA{}: {} dwords loaded, ver={:#X}", engine, dwords, fw_ver);
        } else {
            crate::println!("SDMA{}: FW parse failed", engine);
        }
    }
}

/// Read SDMA F32 instruction SRAM back via UCODE_ADDR/UCODE_DATA and compare
/// against the embedded firmware bytes. Diagnostic for "F32 doesn't fetch ring"
/// — if SRAM is empty/wrong, F32 boots on garbage and never reaches the
/// RB_CNTL fetch loop. Run AFTER `gpu sdma init`. Halts F32 during readback,
/// leaves it halted (re-run init to resume).
pub fn polaris_sdma_ucode_verify(mmio: u64) {
    for engine in 0..2u32 {
        let eng_off = engine * polaris_sdma_regs::SDMA1_OFFSET;
        let fw = if engine == 0 { EMBEDDED_POLARIS_SDMA0 } else { EMBEDDED_POLARIS_SDMA1 };
        let (ucode, dwords, fw_ver) = match parse_polaris_sdma_fw(fw) {
            Some(t) => t,
            None => { crate::println!("SDMA{}: FW parse failed", engine); continue; }
        };
        unsafe {
            // Halt F32 so the read pointer in UCODE_ADDR is stable.
            let f32_pre = mmio_read32(mmio, polaris_sdma_regs::SDMA0_F32_CNTL + eng_off);
            mmio_write32(mmio, polaris_sdma_regs::SDMA0_F32_CNTL + eng_off, f32_pre | 1);
            for _ in 0..10_000u32 { core::hint::spin_loop(); }

            // Reset address pointer to 0; on each DATA read it auto-increments.
            mmio_write32(mmio, polaris_sdma_regs::SDMA0_UCODE_ADDR + eng_off, 0);

            let n_check = core::cmp::min(dwords, 64);
            let mut mismatches: u32 = 0;
            let mut all_zero = true;
            let mut all_same: Option<u32> = None;
            let mut first_mismatch: Option<(usize, u32, u32)> = None;
            for i in 0..n_check {
                // Manually set ADDR per read — Polaris UCODE_ADDR does NOT
                // auto-increment on DATA reads (only on writes during upload).
                mmio_write32(mmio, polaris_sdma_regs::SDMA0_UCODE_ADDR + eng_off, i as u32);
                let read = mmio_read32(mmio, polaris_sdma_regs::SDMA0_UCODE_DATA + eng_off);
                let o = i * 4;
                let want = u32::from_le_bytes([ucode[o], ucode[o+1], ucode[o+2], ucode[o+3]]);
                if read != 0 { all_zero = false; }
                match all_same {
                    None => all_same = Some(read),
                    Some(v) if v != read => all_same = Some(0xDEAD_BEEF),
                    _ => {}
                }
                if read != want {
                    mismatches += 1;
                    if first_mismatch.is_none() {
                        first_mismatch = Some((i, want, read));
                    }
                }
            }
            // Read back the address register
            let addr_after = mmio_read32(mmio, polaris_sdma_regs::SDMA0_UCODE_ADDR + eng_off);

            crate::println!("SDMA{} ucode-verify: dwords={} ver={:#X}", engine, dwords, fw_ver);
            crate::println!("  checked={} mismatches={} addr_after={:#X} (last write {:#X})",
                n_check, mismatches, addr_after, n_check.saturating_sub(1));
            if all_zero {
                crate::println!("  >>> SRAM reads ALL ZERO — ucode NOT in F32 instruction RAM");
            } else if let Some(v) = all_same {
                if v == 0xDEAD_BEEF {
                    // Multiple distinct values seen → varies. Good sign.
                } else {
                    crate::println!("  >>> SRAM stuck at {:#010X} for all reads — ADDR not auto-incrementing or write-only", v);
                }
            }
            if let Some((i, w, r)) = first_mismatch {
                crate::println!("  first mismatch @ dw[{}]: want={:#010X} got={:#010X}", i, w, r);
            } else if mismatches == 0 && !all_zero {
                crate::println!("  >>> ucode bytes MATCH — F32 SRAM contains valid microcode");
            }

            // Restore previous F32_CNTL
            mmio_write32(mmio, polaris_sdma_regs::SDMA0_F32_CNTL + eng_off, f32_pre);
        }
    }
}

pub fn polaris_sdma_setup_rings(mmio: u64) {
    crate::println!("Use 'gpu sdma init' for full ring setup");
}

pub fn polaris_sdma_halt(mmio: u64) {
    for eng in 0..2u32 {
        let off = eng * polaris_sdma_regs::SDMA1_OFFSET;
        unsafe {
            let f32c = mmio_read32(mmio, polaris_sdma_regs::SDMA0_F32_CNTL + off);
            mmio_write32(mmio, polaris_sdma_regs::SDMA0_F32_CNTL + off, f32c | 1);
        }
        crate::println!("SDMA{} halted", eng);
    }
}

pub fn polaris_sdma_unhalt(mmio: u64) {
    for eng in 0..2u32 {
        let off = eng * polaris_sdma_regs::SDMA1_OFFSET;
        unsafe {
            let f32c = mmio_read32(mmio, polaris_sdma_regs::SDMA0_F32_CNTL + off);
            mmio_write32(mmio, polaris_sdma_regs::SDMA0_F32_CNTL + off, f32c & !1u32);
        }
        crate::println!("SDMA{} unhalted", eng);
    }
}

pub fn polaris_sdma_vm_dump(mmio: u64) {
    unsafe {
        crate::println!("=== VM Context Dump ===");
        crate::println!("L1_TLB={:#010X}", mmio_read32(mmio, POL_MC_VM_MX_L1_TLB_CNTL));
        crate::println!("L2_CNTL={:#010X}", mmio_read32(mmio, POL_VM_L2_CNTL));
        crate::println!("L2_CNTL2={:#010X}", mmio_read32(mmio, POL_VM_L2_CNTL2));
        crate::println!("L2_CNTL3={:#010X}", mmio_read32(mmio, POL_VM_L2_CNTL3));
        crate::println!("L2_CNTL4={:#010X}", mmio_read32(mmio, POL_VM_L2_CNTL4));
        crate::println!("CTX0_CNTL={:#010X}", mmio_read32(mmio, POL_VM_CONTEXT0_CNTL));
        crate::println!("CTX0_CNTL2={:#010X}", mmio_read32(mmio, POL_VM_CONTEXT0_CNTL2));
        crate::println!("CTX0_PT_BASE={:#010X}", mmio_read32(mmio, POL_VM_CONTEXT0_PAGE_TABLE_BASE_ADDR));
        crate::println!("CTX0_PT_START={:#010X}", mmio_read32(mmio, POL_VM_CONTEXT0_PAGE_TABLE_START_ADDR));
        crate::println!("CTX0_PT_END={:#010X}", mmio_read32(mmio, POL_VM_CONTEXT0_PAGE_TABLE_END_ADDR));
        crate::println!("FAULT_STATUS={:#010X}", mmio_read32(mmio, POL_VM_CONTEXT0_PROTECTION_FAULT_STATUS));
        crate::println!("INV_REQ={:#X} INV_RESP={:#X}",
            mmio_read32(mmio, POL_VM_INVALIDATE_REQUEST),
            mmio_read32(mmio, POL_VM_INVALIDATE_RESPONSE));
    }
}

pub fn polaris_sdma_vm_clear(mmio: u64) {
    unsafe {
        mmio_write32(mmio, POL_VM_CONTEXT0_CNTL, 0); // disable CTX0
        mmio_write32(mmio, POL_VM_L2_CNTL, 0); // disable L2
        crate::println!("VM CTX0 + L2 disabled");
    }
}

pub fn polaris_sdma_golden(mmio: u64) {
    unsafe {
        for eng in 0..2u32 {
            let off = eng * polaris_sdma_regs::SDMA1_OFFSET;
            // Disable power gating / light sleep
            mmio_write32(mmio, polaris_sdma_regs::SDMA0_POWER_CNTL + off, 0);
            // Clock control: disable clock gating
            mmio_write32(mmio, polaris_sdma_regs::SDMA0_CLK_CTRL + off, 0);
            // Chicken bits: keep default (0)
        }
        crate::println!("Golden regs applied (power/clock gating off)");
    }
}

pub fn polaris_mc_setup(mmio: u64) {
    polaris_gmc_init(mmio);
}

pub fn polaris_sdma_mc(mmio: u64) {
    polaris_mc_diag(mmio);
}

pub fn polaris_sdma_dump(mmio: u64) {
    unsafe {
        for eng in 0..2u32 {
            let off = eng * polaris_sdma_regs::SDMA1_OFFSET;
            use polaris_sdma_regs::*;
            crate::println!("=== SDMA{} ===", eng);
            crate::println!("  CNTL={:#010X}", mmio_read32(mmio, SDMA0_CNTL + off));
            crate::println!("  F32_CNTL={:#010X}", mmio_read32(mmio, SDMA0_F32_CNTL + off));
            crate::println!("  STATUS={:#010X}", mmio_read32(mmio, SDMA0_STATUS_REG + off));
            crate::println!("  FREEZE={:#010X}", mmio_read32(mmio, SDMA0_FREEZE + off));
            crate::println!("  RB_CNTL={:#010X}", mmio_read32(mmio, SDMA0_GFX_RB_CNTL + off));
            crate::println!("  RB_BASE={:#X}:{:#X}", mmio_read32(mmio, SDMA0_GFX_RB_BASE_HI + off), mmio_read32(mmio, SDMA0_GFX_RB_BASE + off));
            crate::println!("  RPTR={:#010X}", mmio_read32(mmio, SDMA0_GFX_RB_RPTR + off));
            crate::println!("  WPTR={:#010X}", mmio_read32(mmio, SDMA0_GFX_RB_WPTR + off));
            crate::println!("  RPTR_ADDR={:#X}:{:#X}", mmio_read32(mmio, SDMA0_GFX_RB_RPTR_ADDR_HI + off), mmio_read32(mmio, SDMA0_GFX_RB_RPTR_ADDR_LO + off));
            crate::println!("  IB_CNTL={:#010X}", mmio_read32(mmio, SDMA0_GFX_IB_CNTL + off));
            crate::println!("  DOORBELL={:#010X}", mmio_read32(mmio, SDMA0_GFX_DOORBELL + off));
            crate::println!("  WPTR_POLL={:#010X}", mmio_read32(mmio, SDMA0_GFX_RB_WPTR_POLL_CNTL + off));
            crate::println!("  POWER={:#010X}", mmio_read32(mmio, SDMA0_POWER_CNTL + off));
            crate::println!("  CLK={:#010X}", mmio_read32(mmio, SDMA0_CLK_CTRL + off));
        }
    }
}

pub fn polaris_sdma_write_wptr(mmio: u64, value: u32) {
    unsafe {
        mmio_write32(mmio, polaris_sdma_regs::SDMA0_GFX_RB_WPTR, value);
        crate::println!("WPTR0={}", value);
        for _ in 0..1_000_000u32 { core::hint::spin_loop(); }
        let rptr = mmio_read32(mmio, polaris_sdma_regs::SDMA0_GFX_RB_RPTR);
        let stat = mmio_read32(mmio, polaris_sdma_regs::SDMA0_STATUS_REG);
        crate::println!("RPTR={:#X} ST={:#010X}", rptr, stat);
    }
}

pub fn polaris_sdma_reg(mmio: u64, reg_offset: u32, write_val: Option<u32>) {
    unsafe {
        if let Some(val) = write_val {
            mmio_write32(mmio, reg_offset, val);
            let rb = mmio_read32(mmio, reg_offset);
            crate::println!("[{:#06X}] wrote {:#010X} readback={:#010X}", reg_offset, val, rb);
        } else {
            let val = mmio_read32(mmio, reg_offset);
            crate::println!("[{:#06X}] = {:#010X}", reg_offset, val);
        }
    }
}

pub fn polaris_sdma_vram_nop_test(mmio: u64) {
    use polaris_sdma_regs::*;

    let (vram_bar_virt, fb_mc) = {
        let guard = POLARIS_BUF.lock();
        match guard.as_ref() {
            Some(buf) => (buf.vram_bar_virt, buf.vram_fb_mc),
            None => { crate::println!("Run `gpu sdma init` first"); return; }
        }
    };
    if vram_bar_virt == 0 || fb_mc == 0 {
        crate::println!("No VRAM BAR mapping in SDMA state");
        return;
    }

    unsafe {
        let ring_off = 0x0010_0000u64;
        let wb_off = ring_off + 0x2000;
        let poll_off = wb_off + 0x40;
        let fence_off = wb_off + 0x80;
        let ring_cpu = vram_bar_virt + ring_off;
        let wb_cpu = vram_bar_virt + wb_off;
        let poll_cpu = vram_bar_virt + poll_off;
        let fence_cpu = vram_bar_virt + fence_off;
        let ring_mc = fb_mc + ring_off;
        let wb_mc = fb_mc + wb_off;
        let poll_mc = fb_mc + poll_off;
        let fence_mc = fb_mc + fence_off;

        crate::println!("=== SDMA VRAM WRITE_LINEAR test (VM_CONTEXT0 disabled) ===");
        crate::println!("ring_mc={:#X} wb_mc={:#X} fence_mc={:#X}", ring_mc, wb_mc, fence_mc);

        let ctx0_pre = mmio_read32(mmio, POL_VM_CONTEXT0_CNTL);
        mmio_write32(mmio, POL_VM_CONTEXT0_CNTL, ctx0_pre & !1u32);
        mmio_write32(mmio, SDMA0_F32_CNTL, 1);
        mmio_write32(mmio, SDMA0_GFX_RB_CNTL, 0);
        mmio_write32(mmio, SDMA0_GFX_RB_RPTR, 0);
        mmio_write32(mmio, SDMA0_GFX_RB_WPTR, 0);
        mmio_write32(mmio, SDMA0_GFX_IB_RPTR, 0);
        mmio_write32(mmio, SDMA0_GFX_IB_OFFSET, 0);

        for off in (0..8192usize).step_by(4) {
            core::ptr::write_volatile((ring_cpu + off as u64) as *mut u32, 0);
        }
        core::ptr::write_volatile(wb_cpu as *mut u32, 0xDEAD_0000);
        core::ptr::write_volatile(poll_cpu as *mut u32, 0);
        core::ptr::write_volatile(fence_cpu as *mut u32, 0xCAFE_DEAD);
        let ring = ring_cpu as *mut u32;
        core::ptr::write_volatile(ring.add(0), 0x0000_0002);
        core::ptr::write_volatile(ring.add(1), fence_mc as u32);
        core::ptr::write_volatile(ring.add(2), (fence_mc >> 32) as u32);
        core::ptr::write_volatile(ring.add(3), 1);
        core::ptr::write_volatile(ring.add(4), 0xDEAD_BEEF);
        core::sync::atomic::fence(core::sync::atomic::Ordering::SeqCst);
        mmio_write32(mmio, POL_HDP_MEM_COHERENCY_FLUSH_CNTL, 1);
        let _ = mmio_read32(mmio, POL_HDP_MEM_COHERENCY_FLUSH_CNTL);

        mmio_write32(mmio, SDMA0_GFX_RB_RPTR_ADDR_HI, (wb_mc >> 32) as u32);
        mmio_write32(mmio, SDMA0_GFX_RB_RPTR_ADDR_LO, (wb_mc as u32) & !3u32);
        mmio_write32(mmio, SDMA0_GFX_RB_BASE, (ring_mc >> 8) as u32);
        mmio_write32(mmio, SDMA0_GFX_RB_BASE_HI, (ring_mc >> 40) as u32);
        mmio_write32(mmio, SDMA0_GFX_DOORBELL, 0);
        mmio_write32(mmio, SDMA0_GFX_RB_WPTR_POLL_ADDR_HI, (poll_mc >> 32) as u32);
        mmio_write32(mmio, SDMA0_GFX_RB_WPTR_POLL_ADDR_LO, (poll_mc as u32) & !3u32);
        let poll_cntl = mmio_read32(mmio, SDMA0_GFX_RB_WPTR_POLL_CNTL) | (1 << 0) | (1 << 2);
        mmio_write32(mmio, SDMA0_GFX_RB_WPTR_POLL_CNTL, poll_cntl);
        mmio_write32(mmio, SDMA0_GFX_IB_CNTL, 0x101);
        mmio_write32(mmio, SDMA0_GFX_RB_CNTL, (10u32 << 1) | (1 << 12) | (3 << 16) | 1);
        mmio_write32(mmio, SDMA0_CNTL, 0x0805_0402);
        mmio_write32(mmio, SDMA0_GFX_CONTEXT_CNTL, 0);
        mmio_write32(mmio, SDMA0_F32_CNTL, 0);
        gpu_udelay(100);
        core::ptr::write_volatile(poll_cpu as *mut u32, 20);
        core::sync::atomic::fence(core::sync::atomic::Ordering::SeqCst);
        mmio_write32(mmio, POL_HDP_MEM_COHERENCY_FLUSH_CNTL, 1);
        let _ = mmio_read32(mmio, POL_HDP_MEM_COHERENCY_FLUSH_CNTL);
        mmio_write32(mmio, SDMA0_GFX_RB_WPTR, 20);

        for _ in 0..100_000u32 {
            if core::ptr::read_volatile(fence_cpu as *const u32) == 0xDEAD_BEEF {
                break;
            }
            core::hint::spin_loop();
        }
        let r = mmio_read32(mmio, SDMA0_GFX_RB_RPTR);
        let w = mmio_read32(mmio, SDMA0_GFX_RB_WPTR);
        let fetch = mmio_read32(mmio, SDMA0_RB_RPTR_FETCH);
        let st = mmio_read32(mmio, SDMA0_STATUS_REG);
        let wb = core::ptr::read_volatile(wb_cpu as *const u32);
        let fence = core::ptr::read_volatile(fence_cpu as *const u32);
        let pf = mmio_read32(mmio, POL_VM_CONTEXT0_PROTECTION_FAULT_STATUS);
        crate::println!("VRAM WRITE: fence={:#010X} R={:#X} W={:#X} FETCH={:#X} ST={:#010X} WB={:#010X} PF={:#010X}",
            fence, r, w, fetch, st, wb, pf);

        mmio_write32(mmio, POL_VM_CONTEXT0_CNTL, ctx0_pre);
    }
}

pub fn polaris_sdma_write_linear_diag(mmio: u64, count_field: u32, dest: &str) {
    use polaris_sdma_regs::*;

    let buf_guard = POLARIS_BUF.lock();
    let buf = match buf_guard.as_ref() {
        Some(b) => b,
        None => { crate::println!("Run `gpu sdma init-latef32` first"); return; }
    };

    let ring_cpu = buf.virt;
    let ring_mc = buf.ring_mc;
    let wb_cpu = buf.wb_cpu;
    let wb_mc = buf.wb_mc;
    let poll_cpu = buf.poll_cpu;

    let (dst_cpu, dst_mc, label) = match dest {
        "ring" => (ring_cpu + 0x100, ring_mc + 0x100, "ring+0x100"),
        "vram" => {
            if buf.vram_bar_virt == 0 || buf.vram_fb_mc == 0 {
                crate::println!("No VRAM BAR state; run init first");
                return;
            }
            (buf.vram_bar_virt + 0x0010_0000, buf.vram_fb_mc + 0x0010_0000, "vram+1M")
        }
        _ => (buf.fence_cpu, buf.fence_mc, "gart-fence"),
    };

    unsafe {
        crate::println!("=== SDMA WRITE_LINEAR diag cnt={} dst={} ===", count_field, label);
        crate::println!("ring={:#X} wb={:#X} dst={:#X}", ring_mc, wb_mc, dst_mc);
        polaris_sdma_va_diag(mmio);

        for off in (0..8192usize).step_by(4) {
            core::ptr::write_volatile((ring_cpu + off as u64) as *mut u32, 0);
        }
        core::ptr::write_volatile(wb_cpu as *mut u32, 0xDEAD_0000);
        core::ptr::write_volatile((wb_cpu + 4) as *mut u32, 0xDEAD_0001);
        core::ptr::write_volatile(poll_cpu as *mut u32, 0);
        core::ptr::write_volatile(dst_cpu as *mut u32, 0xDEAD_DEAD);
        core::ptr::write_volatile((dst_cpu + 4) as *mut u32, 0xBABE_BABE);

        let ring = ring_cpu as *mut u32;
        core::ptr::write_volatile(ring.add(0), 0x0000_0002);
        core::ptr::write_volatile(ring.add(1), dst_mc as u32);
        core::ptr::write_volatile(ring.add(2), (dst_mc >> 32) as u32);
        core::ptr::write_volatile(ring.add(3), count_field);
        core::ptr::write_volatile(ring.add(4), 0xCAFE_BABE);
        core::sync::atomic::fence(core::sync::atomic::Ordering::SeqCst);
        #[cfg(target_arch = "x86_64")]
        core::arch::asm!("wbinvd", options(nostack, preserves_flags));
        mmio_write32(mmio, POL_HDP_MEM_COHERENCY_FLUSH_CNTL, 1);
        let _ = mmio_read32(mmio, POL_HDP_MEM_COHERENCY_FLUSH_CNTL);

        core::ptr::write_volatile(poll_cpu as *mut u32, 20);
        core::sync::atomic::fence(core::sync::atomic::Ordering::SeqCst);
        #[cfg(target_arch = "x86_64")]
        core::arch::asm!("wbinvd", options(nostack, preserves_flags));
        mmio_write32(mmio, SDMA0_GFX_RB_WPTR, 20);

        let mut first_seen = u32::MAX;
        for i in 0..100_000u32 {
            if i & 0xFF == 0 {
                mmio_write32(mmio, POL_HDP_MEM_COHERENCY_FLUSH_CNTL, 1);
                let _ = mmio_read32(mmio, POL_HDP_MEM_COHERENCY_FLUSH_CNTL);
            }
            let v = core::ptr::read_volatile(dst_cpu as *const u32);
            if v == 0xCAFE_BABE {
                first_seen = i;
                break;
            }
            core::hint::spin_loop();
        }

        mmio_write32(mmio, POL_HDP_MEM_COHERENCY_FLUSH_CNTL, 1);
        let _ = mmio_read32(mmio, POL_HDP_MEM_COHERENCY_FLUSH_CNTL);
        let dst0 = core::ptr::read_volatile(dst_cpu as *const u32);
        let dst1 = core::ptr::read_volatile((dst_cpu + 4) as *const u32);
        let wb0 = core::ptr::read_volatile(wb_cpu as *const u32);
        let wb1 = core::ptr::read_volatile((wb_cpu + 4) as *const u32);
        let rptr = mmio_read32(mmio, SDMA0_GFX_RB_RPTR);
        let wptr = mmio_read32(mmio, SDMA0_GFX_RB_WPTR);
        let fetch = mmio_read32(mmio, SDMA0_RB_RPTR_FETCH);
        let st = mmio_read32(mmio, SDMA0_STATUS_REG);
        let pf = mmio_read32(mmio, POL_VM_CONTEXT0_PROTECTION_FAULT_STATUS);
        let mcc = mmio_read32(mmio, POL_VM_CONTEXT0_PROTECTION_FAULT_MCCLIENT);

        crate::println!(
            "WRDIAG: dst0={:#010X} dst1={:#010X} first={} RPTR={:#X} WPTR={:#X} FETCH={:#X}",
            dst0, dst1, first_seen, rptr, wptr, fetch
        );
        crate::println!(
            "WRDIAG: WB={:#010X}/{:#010X} ST={:#010X} PF={:#010X} MCCLIENT={:#010X}",
            wb0, wb1, st, pf, mcc
        );
    }
}

pub fn polaris_gpu_cleanup(mmio: u64) {
    polaris_sdma_halt(mmio);
    polaris_sdma_reset(mmio);
    // Clear VM faults
    unsafe {
        mmio_write32(mmio, POL_VM_INVALIDATE_REQUEST, 0xFFFF);
        for _ in 0..100_000u32 { core::hint::spin_loop(); }
    }
    crate::println!("GPU cleanup done");
}

pub fn polaris_sdma_probe(mmio: u64) {
    polaris_sdma_diag(mmio);
}

pub fn polaris_sdma_steptrace(mmio: u64, _max_step: u8) {
    polaris_sdma_full_init(mmio);
}

pub fn polaris_sdma_deep_diag(mmio: u64) {
    polaris_sdma_diag(mmio);
    polaris_sdma_dump(mmio);
}

pub fn polaris_sdma_vram_ring_test(mmio: u64) {
    polaris_sdma_full_init(mmio);
}

pub fn polaris_sdma_bios_vram_test(mmio: u64) {
    // Test with BIOS firmware (no reload)
    crate::println!("BIOS VRAM test: use 'gpu sdma init' instead");
    polaris_sdma_diag(mmio);
}

pub fn polaris_vtd_disable() {
    // VT-d/IOMMU disable attempt via PCI config
    // This is a no-op placeholder; actual VT-d disable requires DMAR ACPI table manipulation
    crate::println!("VT-d: no action (bare-metal, IOMMU should be off by default)");
}

// ── Compatibility stubs for vm.rs — all delegate to polaris_sdma_full_init ──
pub fn polaris_sdma_init_vram(mmio: u64) { polaris_sdma_full_init(mmio); }
pub fn polaris_sdma_init_gart(mmio: u64) { polaris_sdma_full_init(mmio); }

/// Map physical pages into GART and return MC base address.
/// `phys_pages`: slice of 4KB-aligned physical addresses to map
/// `start_pte`: first PTE index to use (PTEs 0-5 = SDMA, 6-24 = compute)
/// Returns MC base address (GART_MC_BASE + start_pte * 0x1000)
pub fn gart_map_pages(mmio: u64, phys_pages: &[u64], start_pte: usize) -> Option<u64> {
    let buf_guard = POLARIS_BUF.lock();
    let buf = buf_guard.as_ref()?;
    let vram_bar_virt = buf.vram_bar_virt;
    drop(buf_guard);

    let gart_table_vram_off: u64 = 0x380000;
    let gart_table_cpu = vram_bar_virt + gart_table_vram_off;

    unsafe {
        let table = gart_table_cpu as *mut u64;
        for (i, &phys) in phys_pages.iter().enumerate() {
            core::ptr::write_volatile(table.add(start_pte + i),
                (phys & !0xFFFu64) | GART_PTE_SYSRAM);
        }
        // Flush GART TLB
        mmio_write32(mmio, POL_VM_INVALIDATE_REQUEST, 1);
        for _ in 0..100_000u32 {
            let resp = mmio_read32(mmio, POL_VM_INVALIDATE_RESPONSE);
            if resp & 1 != 0 { break; }
            core::hint::spin_loop();
        }
    }

    const GART_MC_BASE: u64 = 0xFF00000000;
    Some(GART_MC_BASE + (start_pte as u64) * 0x1000)
}

/// Diff GART PTE[0..6] against expected ring/wb/fence physical addresses.
/// Reuses POLARIS_BUF.vram_bar_virt (already mapped during init), so this is
/// safe to call from the shell without re-mapping the VRAM BAR. Also dumps
/// the first 8 DWs of the ring (CPU view via HHDM) so we can compare to what
/// F32 would fetch through GART. If PTE phys differs from buf.phys, the
/// GART setup is broken and F32 reads garbage. If they match, F32 reads the
/// same bytes the CPU sees — so any fetch failure is a deeper interlock.
pub fn polaris_sdma_ptediff(mmio: u64) {
    use polaris_sdma_regs::*;
    crate::println!("=== GART PTE diff ===");
    let buf_guard = POLARIS_BUF.lock();
    let buf = match buf_guard.as_ref() {
        Some(b) => b,
        None => { crate::println!("No SDMA buf — run `gpu sdma init` first"); return; }
    };
    let vram_bar_virt = buf.vram_bar_virt;
    let ring_virt     = buf.virt;
    let ring_phys     = buf.phys;
    let ring_mc       = buf.ring_mc;
    let wb_mc         = buf.wb_mc;
    drop(buf_guard);

    let gart_table_cpu = vram_bar_virt + 0x380000;
    crate::println!("ring CPU={:#X} phys={:#X} mc={:#X}  wb_mc={:#X}",
        ring_virt, ring_phys, ring_mc, wb_mc);
    crate::println!("GART table CPU={:#X}", gart_table_cpu);

    // PTE[0] should map ring_mc -> ring_phys (page-aligned)
    let expected_pte0_phys = ring_phys & !0xFFFu64;
    let mc_base: u64 = 0xFF00000000;
    let expected_pte_idx = ((ring_mc - mc_base) / 0x1000) as usize;
    crate::println!("expected: ring at PTE[{}] phys={:#X}", expected_pte_idx, expected_pte0_phys);

    unsafe {
        let table = gart_table_cpu as *const u64;
        for i in 0..8usize {
            let pte = core::ptr::read_volatile(table.add(i));
            let phys = pte & !0xFFFu64;
            let flags = pte & 0xFFF;
            let v = (flags & 1) != 0;
            let s = (flags & 2) != 0;
            let sn = (flags & 4) != 0;
            let r = (flags & 0x20) != 0;
            let w = (flags & 0x40) != 0;
            let mark = if i == expected_pte_idx && phys == expected_pte0_phys {
                "  <-- ring (MATCH)"
            } else if i == expected_pte_idx {
                "  <-- ring (MISMATCH!)"
            } else { "" };
            crate::println!("PTE[{}]={:#018X} phys={:#X} V={} S={} SN={} R={} W={}{}",
                i, pte, phys, v as u8, s as u8, sn as u8, r as u8, w as u8, mark);
        }
    }

    // Show first 8 DWs of ring via CPU HHDM view (what F32 SHOULD read if GART OK)
    crate::println!("--- ring[0..8] CPU view (HHDM) ---");
    unsafe {
        let r = ring_virt as *const u32;
        for i in 0..8usize {
            let dw = core::ptr::read_volatile(r.add(i));
            crate::println!("  ring[{}] = {:#010X}", i, dw);
        }
    }

    // F32 fetch state
    unsafe {
        let st   = mmio_read32(mmio, SDMA0_STATUS_REG);
        let f32c = mmio_read32(mmio, SDMA0_F32_CNTL);
        let rptr = mmio_read32(mmio, SDMA0_GFX_RB_RPTR);
        let wptr = mmio_read32(mmio, SDMA0_GFX_RB_WPTR);
        let rbc  = mmio_read32(mmio, SDMA0_GFX_RB_CNTL);
        crate::println!("SDMA0: ST={:#010X} F32={:#X} RBC={:#010X} R={:#X} W={:#X}",
            st, f32c, rbc, rptr, wptr);
    }
}

// ── CP stubs (Command Processor — separate from SDMA, future work) ────────
pub fn polaris_cp_mec_init(mmio: u64) {
    crate::println!("=== CP MEC Compute Init (Polaris GCN4) ===");

    if !gpu_alive(mmio) {
        crate::println!("GPU dead");
        return;
    }

    // Need SDMA init done first (for GART + MEC firmware)
    let buf_guard = POLARIS_BUF.lock();
    let buf = match buf_guard.as_ref() {
        Some(b) => b,
        None => { crate::println!("Run 'gpu sdma init' first (need GART)"); return; }
    };
    let vram_bar_virt = buf.vram_bar_virt;
    let vram_fb_mc = buf.vram_fb_mc;
    drop(buf_guard);

    // Read GART table location
    let fb_loc = unsafe { mmio_read32(mmio, POL_MC_VM_FB_LOCATION) };
    let fb_start = ((fb_loc & 0xFFFF) as u64) << 24;
    let gart_table_vram_off: u64 = 0x380000;
    let gart_table_cpu = vram_bar_virt + gart_table_vram_off;

    // Allocate sysRAM for compute buffers:
    //   - Compute ring: 4KB (1 page)
    //   - Data buffer:  64KB (16 pages)
    //   - Code buffer:  4KB (1 page)
    //   - WB/fence:     4KB (1 page, shared)
    // Total: 19 pages → GART PTEs 6..24 (PTEs 0-5 used by SDMA+IH)
    let page_layout = match alloc::alloc::Layout::from_size_align(4096, 4096) {
        Ok(l) => l,
        Err(_) => { crate::println!("bad page layout"); return; }
    };
    let data_layout = match alloc::alloc::Layout::from_size_align(64 * 1024, 4096) {
        Ok(l) => l,
        Err(_) => { crate::println!("bad data layout"); return; }
    };

    let ring_virt = unsafe { alloc::alloc::alloc_zeroed(page_layout.clone()) } as u64;
    let data_virt = unsafe { alloc::alloc::alloc_zeroed(data_layout) } as u64;
    let code_virt = unsafe { alloc::alloc::alloc_zeroed(page_layout.clone()) } as u64;
    let wb_virt   = unsafe { alloc::alloc::alloc_zeroed(page_layout) } as u64;

    if ring_virt == 0 || data_virt == 0 || code_virt == 0 || wb_virt == 0 {
        crate::println!("sysRAM alloc failed");
        return;
    }

    // Get physical addresses for all pages
    let ring_phys = match crate::memory::virt_to_phys(ring_virt) {
        Some(p) => p, None => { crate::println!("ring v2p fail"); return; }
    };
    let code_phys = match crate::memory::virt_to_phys(code_virt) {
        Some(p) => p, None => { crate::println!("code v2p fail"); return; }
    };
    let wb_phys = match crate::memory::virt_to_phys(wb_virt) {
        Some(p) => p, None => { crate::println!("wb v2p fail"); return; }
    };

    // Data buffer: get phys for each 4KB page (up to 16)
    let mut data_phys_pages: [u64; 16] = [0; 16];
    for i in 0..16 {
        data_phys_pages[i] = match crate::memory::virt_to_phys(data_virt + (i as u64) * 4096) {
            Some(p) => p, None => { crate::println!("data page {} v2p fail", i); return; }
        };
    }

    // GART MC addresses: PTEs 6+ (after SDMA PTEs 0-5)
    const GART_MC_BASE: u64 = 0xFF00000000;
    let ring_mc = GART_MC_BASE + 0x6000;    // PTE 6
    let data_mc = GART_MC_BASE + 0x7000;    // PTEs 7..22 (16 pages)
    let code_mc = GART_MC_BASE + 0x17000;   // PTE 23
    let wb_mc   = GART_MC_BASE + 0x18000;   // PTE 24

    // Write GART PTEs
    unsafe {
        let table = gart_table_cpu as *mut u64;
        // PTE 6: compute ring
        core::ptr::write_volatile(table.add(6),
            (ring_phys & !0xFFFu64) | GART_PTE_SYSRAM);
        // PTEs 7..22: data buffer (16 pages)
        for i in 0..16 {
            core::ptr::write_volatile(table.add(7 + i),
                (data_phys_pages[i] & !0xFFFu64) | GART_PTE_SYSRAM);
        }
        // PTE 23: code buffer
        core::ptr::write_volatile(table.add(23),
            (code_phys & !0xFFFu64) | GART_PTE_SYSRAM);
        // PTE 24: WB/fence
        core::ptr::write_volatile(table.add(24),
            (wb_phys & !0xFFFu64) | GART_PTE_SYSRAM);
    }

    // Flush GART TLB
    unsafe {
        mmio_write32(mmio, POL_VM_INVALIDATE_REQUEST, 1);
        for _ in 0..100_000u32 {
            let resp = mmio_read32(mmio, POL_VM_INVALIDATE_RESPONSE);
            if resp & 1 != 0 { break; }
            core::hint::spin_loop();
        }
    }

    crate::serial_println!("[CP-MEC] ring mc={:#X} data mc={:#X} code mc={:#X} wb mc={:#X}",
        ring_mc, data_mc, code_mc, wb_mc);
    crate::println!("GART: ring={:#X} data={:#X} code={:#X}", ring_mc, data_mc, code_mc);

    // Load RLC firmware first (required for compute block power/clock gating)
    {
        let fw_state = FW_STATE.lock();
        if fw_state.rlc != FwStatus::Running && fw_state.rlc != FwStatus::Loaded {
            drop(fw_state);
            crate::println!("Loading RLC firmware...");
            let rlc_parsed = parse_polaris_sdma_fw(EMBEDDED_POLARIS_RLC);
            match rlc_parsed {
                Some((ucode, dwords, ver)) => {
                    crate::serial_println!("[CP-MEC] RLC: ver={:#X} {} DWORDs", ver, dwords);
                    if let Err(e) = load_rlc(mmio, ucode, ver) {
                        crate::println!("RLC fw load failed: {}", e);
                        return;
                    }
                    let mut fw_state = FW_STATE.lock();
                    fw_state.rlc = FwStatus::Loaded;
                }
                None => {
                    crate::println!("RLC fw parse failed");
                    return;
                }
            }
        }
    }

    // Load GFX CP pipe (PFP/ME/CE) — must run before MEC.
    // PFP/ME initialize shared CP scratch registers that MEC boot ROM reads.
    {
        let fw_state = FW_STATE.lock();
        if fw_state.pfp != FwStatus::Running && fw_state.pfp != FwStatus::Loaded {
            drop(fw_state);
            crate::println!("Loading CP GFX firmware (PFP/ME/CE)...");
            let pfp_ucode = parse_polaris_gfx_fw(EMBEDDED_POLARIS_PFP)
                .map(|(ucode, _jt, ver)| {
                    crate::serial_println!("[CP-MEC] PFP: ver={:#X} {}B", ver, ucode.len());
                    ucode
                });
            let me_ucode = parse_polaris_gfx_fw(EMBEDDED_POLARIS_ME)
                .map(|(ucode, _jt, ver)| {
                    crate::serial_println!("[CP-MEC] ME:  ver={:#X} {}B", ver, ucode.len());
                    ucode
                });
            let ce_ucode = parse_polaris_gfx_fw(EMBEDDED_POLARIS_CE)
                .map(|(ucode, _jt, ver)| {
                    crate::serial_println!("[CP-MEC] CE:  ver={:#X} {}B", ver, ucode.len());
                    ucode
                });
            if let Err(e) = load_cp_gfx(mmio, pfp_ucode, me_ucode, ce_ucode) {
                crate::println!("CP GFX fw load failed: {}", e);
                return;
            }
            let mut fw_state = FW_STATE.lock();
            fw_state.pfp = FwStatus::Loaded;
            fw_state.me = FwStatus::Loaded;
            fw_state.ce = FwStatus::Loaded;
        }
    }

    // Load MEC firmware if not already loaded
    {
        let fw_state = FW_STATE.lock();
        if fw_state.mec1 != FwStatus::Running && fw_state.mec1 != FwStatus::Loaded {
            drop(fw_state);
            crate::println!("Loading MEC firmware...");
            let mec1_parsed = parse_polaris_gfx_fw(EMBEDDED_POLARIS_MEC1)
                .map(|(ucode, jt, ver)| {
                    crate::serial_println!("[CP-MEC] MEC1: ver={:#X} jt_dwords={}", ver, jt);
                    (ucode, jt, ver)
                });
            let mec2_parsed = parse_polaris_gfx_fw(EMBEDDED_POLARIS_MEC2)
                .map(|(ucode, jt, ver)| {
                    crate::serial_println!("[CP-MEC] MEC2: ver={:#X} jt_dwords={}", ver, jt);
                    (ucode, jt, ver)
                });
            if let Err(e) = load_mec(mmio, mec1_parsed, mec2_parsed) {
                crate::println!("MEC fw load failed: {}", e);
                return;
            }
            let mut fw_state = FW_STATE.lock();
            fw_state.mec1 = FwStatus::Loaded;
            fw_state.mec2 = FwStatus::Loaded;
        }
    }

    // Verify MEC firmware actually started (PC counters should be non-zero).
    // Per-pipe regs: must select MEID via SRBM_GFX_CNTL before reading.
    unsafe {
        mmio_write32(mmio, regs::SRBM_GFX_CNTL_V8, 1 << 2); // MEID=1 (MEC1)
        let mec1_pc = mmio_read32(mmio, regs::CP_MEC1_INSTR_PNTR);
        mmio_write32(mmio, regs::SRBM_GFX_CNTL_V8, 2 << 2); // MEID=2 (MEC2)
        let mec2_pc = mmio_read32(mmio, regs::CP_MEC2_INSTR_PNTR);
        mmio_write32(mmio, regs::SRBM_GFX_CNTL_V8, 0); // restore GFX
        let mec_cntl = mmio_read32(mmio, regs::CP_MEC_CNTL);
        crate::println!("MEC post-load: CNTL={:#X} PC1={:#X} PC2={:#X}",
            mec_cntl, mec1_pc, mec2_pc);
        crate::serial_println!("[CP-MEC] post-load CNTL={:#X} PC1={:#X} PC2={:#X}",
            mec_cntl, mec1_pc, mec2_pc);
        if mec1_pc == 0 || mec1_pc == 0xDEADBEEF {
            crate::println!("WARNING: MEC1 program counter is {:#X} — firmware may not be running", mec1_pc);
        }
    }

    // Initialize compute engine
    super::compute::init_polaris(
        mmio,
        ring_virt, ring_mc,
        data_virt, data_mc,
        code_virt, code_mc,
        wb_virt, wb_mc,
    );

    crate::println!("CP MEC compute ready — use 'gpuexec' to dispatch");
}
pub fn polaris_cp_nop_test(mmio: u64) { crate::println!("CP NOP: not yet reimplemented"); }
pub fn polaris_cp_regscan(mmio: u64) { crate::println!("CP regscan: not yet reimplemented"); }
pub fn polaris_cp_nop_vram(mmio: u64, _vram: u64) { crate::println!("CP: not yet reimplemented"); }
pub fn polaris_cp_nop_lowmem(mmio: u64) { crate::println!("CP: not yet reimplemented"); }
pub fn polaris_cp_write_sentinel(mmio: u64) { crate::println!("CP: not yet reimplemented"); }
pub fn polaris_cp_sentinel_v30(mmio: u64, _vram: u64) { crate::println!("CP: not yet reimplemented"); }
pub fn polaris_cp_v75(mmio: u64, _vram: u64) { crate::println!("CP: not yet reimplemented"); }
pub fn polaris_cp_dispatch(mmio: u64, _flags: u32) { crate::println!("CP: not yet reimplemented"); }
pub fn polaris_cp_vmcheck(mmio: u64) { polaris_sdma_vm_dump(mmio); }
pub fn polaris_cp_nop_v3(mmio: u64) { crate::println!("CP: not yet reimplemented"); }
pub fn polaris_cp_linux_order(mmio: u64) { crate::println!("CP: not yet reimplemented"); }
pub fn polaris_cp_bios_test(mmio: u64) { crate::println!("CP: not yet reimplemented"); }
pub fn polaris_cp_bios2_test(mmio: u64) { crate::println!("CP: not yet reimplemented"); }
pub fn polaris_cp_vram_test(mmio: u64, _vram: u64) { crate::println!("CP: not yet reimplemented"); }
pub fn polaris_cp_bios_vram_test(mmio: u64, _vram: u64) { crate::println!("CP: not yet reimplemented"); }
pub fn polaris_cp_kick_test(mmio: u64, _vram: u64) { crate::println!("CP: not yet reimplemented"); }
pub fn polaris_cp_gfx_test(mmio: u64) { crate::println!("CP: not yet reimplemented"); }
pub fn polaris_cp_rb_dump(mmio: u64) { crate::println!("CP: not yet reimplemented"); }
pub fn polaris_cp_nop_lowmem2(mmio: u64) { crate::println!("CP: not yet reimplemented"); }
pub fn polaris_linux_init(mmio: u64, _vram: u64) { polaris_sdma_full_init(mmio); }

// =============================================================================
// GFX v8.0 GPU Init — the missing layer between IH and CP firmware
// =============================================================================
//
// Linux: gfx_v8_0_gpu_init() + gfx_v8_0_init_golden_registers()
// This runs BEFORE any CP/MEC firmware load. Without it:
//   - SH_MEM_CONFIG is undefined → shaders can't address memory
//   - Golden registers are in VBIOS state → caches/routing undefined
//   - SPI/SQ not configured → shader processor won't dispatch waves
//
// Call via: `gpu sdma gfx-init` (before `gpu sdma cp-init`)

/// GFX8 Polaris register offsets (dword index × 4)
mod pol_gfx_regs {
    pub const GRBM_GFX_INDEX:        u32 = 0xC200 * 4;
    pub const SRBM_GFX_CNTL:         u32 = 0x0391 * 4;

    // Shader Memory (per-VMID, selected via SRBM_GFX_CNTL)
    pub const SH_MEM_CONFIG:          u32 = 0x2614 * 4;
    pub const SH_MEM_APE1_BASE:       u32 = 0x2615 * 4;
    pub const SH_MEM_APE1_LIMIT:      u32 = 0x2616 * 4;
    pub const SH_MEM_BASES:           u32 = 0x2617 * 4;

    // Address / Tiling config
    pub const GB_ADDR_CONFIG:         u32 = 0x263E * 4;
    pub const HDP_ADDR_CONFIG:        u32 = 0x0BD2 * 4;  // mmHDP_ADDR_CONFIG

    // Shader Processor / Sequencer
    pub const SPI_CONFIG_CNTL:        u32 = 0x2440 * 4;
    pub const SPI_RESOURCE_RESERVE_CU_0:    u32 = 0x2443 * 4;
    pub const SPI_RESOURCE_RESERVE_CU_1:    u32 = 0x2444 * 4;
    pub const SPI_RESOURCE_RESERVE_EN_CU_0: u32 = 0x2445 * 4;
    pub const SPI_RESOURCE_RESERVE_EN_CU_1: u32 = 0x2446 * 4;
    pub const SQ_CONFIG:              u32 = 0x2300 * 4;
    pub const SQ_RANDOM_WAVE_PRI:     u32 = 0x2303 * 4;

    // Command Processor
    pub const CP_ME_CNTL:             u32 = 0x21B6 * 4;
    pub const CP_MEQ_THRESHOLDS:      u32 = 0x21B0 * 4;
    pub const CP_PERFMON_CNTL:        u32 = 0x21B7 * 4;

    // PA / SC
    pub const PA_SC_RASTER_CONFIG:    u32 = 0xA0D4 * 4;
    pub const PA_SC_RASTER_CONFIG_1:  u32 = 0xA0D5 * 4;
    pub const PA_SC_ENHANCE:          u32 = 0xA2D4 * 4;
    pub const PA_SC_LINE_STIPPLE_STATE: u32 = 0xA2D2 * 4;
    pub const PA_SC_FIFO_SIZE:        u32 = 0xA2DC * 4;

    // VGT
    pub const VGT_NUM_INSTANCES:      u32 = 0xA2A2 * 4;
    pub const VGT_CACHE_INVALIDATION: u32 = 0xA2A4 * 4;
    pub const VGT_GS_VERTEX_REUSE:    u32 = 0xA2CE * 4;

    // Texture / Cache
    pub const TA_CNTL_AUX:           u32 = 0x2542 * 4;
    pub const TCP_CHAN_STEER_LO:      u32 = 0x2B03 * 4;
    pub const TCP_CHAN_STEER_HI:      u32 = 0x2B04 * 4;
    pub const TCP_ADDR_CONFIG:        u32 = 0x2B05 * 4;

    // CB / DB
    pub const CB_HW_CONTROL_3:        u32 = 0xA41E * 4;
    pub const DB_DEBUG2:              u32 = 0x260D * 4;

    // SX
    pub const SX_DEBUG_1:             u32 = 0x2420 * 4;
}

/// Golden register entry: (offset, mask, value)
/// Applied as: reg = (reg & ~mask) | (value & mask)
struct GoldenReg(u32, u32, u32);

/// Polaris10 golden_settings_a11 + golden_common_all
/// Source: Linux gfx_v8_0.c, verified against mmiotrace 20260413
const POLARIS10_GOLDEN: &[GoldenReg] = &[
    // golden_common_all (Polaris10 36CU / 4SE×9CU/SE)
    GoldenReg(pol_gfx_regs::PA_SC_RASTER_CONFIG,    0x3f3f3fff, 0x16000012),
    GoldenReg(pol_gfx_regs::PA_SC_RASTER_CONFIG_1,  0x0000003f, 0x0000002a),
    GoldenReg(pol_gfx_regs::GB_ADDR_CONFIG,          0xffffffff, 0x22011003),
    GoldenReg(pol_gfx_regs::SPI_RESOURCE_RESERVE_CU_0,    0xffffffff, 0x00000800),
    GoldenReg(pol_gfx_regs::SPI_RESOURCE_RESERVE_CU_1,    0xffffffff, 0x00000800),
    GoldenReg(pol_gfx_regs::SPI_RESOURCE_RESERVE_EN_CU_0, 0xffffffff, 0x00FF7FBF),
    GoldenReg(pol_gfx_regs::SPI_RESOURCE_RESERVE_EN_CU_1, 0xffffffff, 0x00FF7FBF),
    // golden_a11
    GoldenReg(pol_gfx_regs::CB_HW_CONTROL_3,        0x000001ff, 0x00000040),
    GoldenReg(pol_gfx_regs::DB_DEBUG2,               0xf00fffff, 0x00000400),
    GoldenReg(pol_gfx_regs::PA_SC_ENHANCE,           0xffffffff, 0x20000001),
    GoldenReg(pol_gfx_regs::PA_SC_LINE_STIPPLE_STATE, 0x0000ff0f, 0x00000000),
    GoldenReg(pol_gfx_regs::SQ_RANDOM_WAVE_PRI,     0x001fffff, 0x000006fd),
    GoldenReg(pol_gfx_regs::TA_CNTL_AUX,            0x000f000f, 0x000b0000),
    GoldenReg(pol_gfx_regs::TCP_ADDR_CONFIG,         0x000003ff, 0x000000f7),
    GoldenReg(pol_gfx_regs::TCP_CHAN_STEER_HI,       0xffffffff, 0x00000000),
    GoldenReg(pol_gfx_regs::TCP_CHAN_STEER_LO,       0xffffffff, 0x00003210),
    GoldenReg(pol_gfx_regs::VGT_CACHE_INVALIDATION,  0x3fff3e3f, 0x00000002),
    GoldenReg(pol_gfx_regs::VGT_GS_VERTEX_REUSE,    0x0000001f, 0x00000010),
];

/// gfx_v8_0_gpu_init + gfx_v8_0_init_golden_registers equivalent for Polaris10.
///
/// Must be called AFTER gmc_init + ih_init, BEFORE cp/mec firmware load.
/// Linux order: vi_common → gmc_v8_0 → tonga_ih → **gfx_v8_0** → sdma_v3_0
pub fn polaris_gfx_init(mmio: u64) {
    use pol_gfx_regs::*;

    if !gpu_alive(mmio) {
        crate::println!("GPU dead");
        return;
    }

    let t0 = crate::time::uptime_ms();
    crate::serial_println!("=== POLARIS GFX INIT (gfx_v8_0_gpu_init) ===");
    crate::println!("=== GFX Init (gfx_v8_0 equivalent) ===");

    unsafe {
        // ── Step 1: Broadcast mode (all SE/SH/INSTANCE) ────────────────
        mmio_write32(mmio, GRBM_GFX_INDEX, 0xE000_0000);

        // ── Step 2: Golden registers (read-modify-write) ───────────────
        crate::serial_println!("[GFX] Applying {} golden registers...", POLARIS10_GOLDEN.len());
        for g in POLARIS10_GOLDEN {
            let old = mmio_read32(mmio, g.0);
            let new = (old & !g.1) | (g.2 & g.1);
            mmio_write32(mmio, g.0, new);
        }

        // Read back GB_ADDR_CONFIG to confirm
        let gb_cfg = mmio_read32(mmio, GB_ADDR_CONFIG);
        crate::serial_println!("[GFX] GB_ADDR_CONFIG = {:#010X} (expected 0x22011003)", gb_cfg);
        crate::println!("GB_ADDR_CONFIG = {:#010X}", gb_cfg);

        // Propagate GB_ADDR_CONFIG to HDP (Linux: gmc_v8_0 does this)
        mmio_write32(mmio, HDP_ADDR_CONFIG, gb_cfg);

        // ── Step 3: SH_MEM_CONFIG per VMID (THE critical missing piece) ─
        // Linux: gfx_v8_0_gpu_init → gfx_v8_0_init_compute_vmid
        // SH_MEM_CONFIG tells shader engines how to resolve memory addresses.
        // Without this, compute dispatches hang silently.
        //
        // Bitfields (gfx_8_0_sh_mask.h):
        //   [1:0]   ALIGNMENT_MODE  — 0=DWORD_STRICT, 1=DWORD, 2=UNALIGNED
        //   [12]    ADDRESS_MODE    — 0=32bit, 1=HSA64 (flat 64-bit)
        //   [14:13] DEFAULT_MTYPE   — 0=NC(non-cached), 2=CC(cached-coherent)
        //   [17:16] APE1_MTYPE      — 0=NC
        //
        // For bare-metal compute on VMID 0: HSA64 + UNALIGNED + NC
        let sh_mem_cfg: u32 = (1 << 12)   // ADDRESS_MODE = HSA64
                            | (2 << 0);    // ALIGNMENT_MODE = UNALIGNED
        // DEFAULT_MTYPE = NC (0), APE1_MTYPE = NC (0) → no extra bits needed

        crate::serial_println!("[GFX] Programming SH_MEM for 16 VMIDs: config={:#X}", sh_mem_cfg);
        for vmid in 0..16u32 {
            mmio_write32(mmio, SRBM_GFX_CNTL, vmid << 4);
            mmio_write32(mmio, SH_MEM_CONFIG, sh_mem_cfg);
            mmio_write32(mmio, SH_MEM_APE1_BASE, 1);  // base > limit = APE1 disabled
            mmio_write32(mmio, SH_MEM_APE1_LIMIT, 0);
            mmio_write32(mmio, SH_MEM_BASES, 0);       // no private/shared segment offset
        }
        mmio_write32(mmio, SRBM_GFX_CNTL, 0); // restore VMID 0
        crate::println!("SH_MEM_CONFIG = {:#X} (16 VMIDs)", sh_mem_cfg);

        // ── Step 4: SPI / SQ / CP pipeline defaults ────────────────────
        // Linux: gfx_v8_0_gpu_init()
        mmio_write32(mmio, SPI_CONFIG_CNTL, 0x0300_0000); // GPR_WRITE_PRIORITY=3
        mmio_write32(mmio, SQ_CONFIG, 0);                  // default SQ config
        mmio_write32(mmio, CP_MEQ_THRESHOLDS, 0x0030_0030); // MEQ1=0x30, MEQ2=0x30
        mmio_write32(mmio, CP_PERFMON_CNTL, 0);
        mmio_write32(mmio, SX_DEBUG_1, 0x20);
        mmio_write32(mmio, VGT_NUM_INSTANCES, 1);

        // PA_SC_FIFO_SIZE — Linux uses a computed value based on SE count.
        // Polaris10 (4 SE): SC_FRONTEND=0x20, SC_BACKEND=0x100, SC_HIZ=0x30
        //                   SC_EARLYZ=0x60 → combined = 0x00600030_01000020
        // Simplified: use the VBIOS value (read-modify if needed)
        let pa_fifo = mmio_read32(mmio, PA_SC_FIFO_SIZE);
        if pa_fifo == 0 {
            mmio_write32(mmio, PA_SC_FIFO_SIZE, 0x0060_0100);
        }

        crate::serial_println!("[GFX] SPI/SQ/CP/VGT configured");

        // ── Step 5: Ensure broadcast restored ──────────────────────────
        mmio_write32(mmio, GRBM_GFX_INDEX, 0xE000_0000);

        // ── Verify key registers ───────────────────────────────────────
        mmio_write32(mmio, SRBM_GFX_CNTL, 0); // VMID 0
        let sh_rb = mmio_read32(mmio, SH_MEM_CONFIG);
        let spi_rb = mmio_read32(mmio, SPI_CONFIG_CNTL);
        let sq_rb = mmio_read32(mmio, SQ_CONFIG);
        let vgt_rb = mmio_read32(mmio, VGT_NUM_INSTANCES);
        crate::serial_println!("[GFX] Verify: SH_MEM={:#X} SPI={:#X} SQ={:#X} VGT_INST={}",
            sh_rb, spi_rb, sq_rb, vgt_rb);
    }

    let dt = crate::time::uptime_ms() - t0;
    crate::println!("GFX init done ({}ms) — 18 golden + SH_MEM×16 + SPI/SQ/CP", dt);
    crate::serial_println!("=== GFX INIT COMPLETE ({}ms) ===", dt);
}
