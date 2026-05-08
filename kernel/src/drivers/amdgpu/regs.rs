//! AMD GPU Register Definitions — Navi 10 (RDNA 1) — SOC15
//!
//! Register byte offsets for direct MMIO and indirect (SMN) access.
//! Based on Linux amdgpu driver: navi10_ip_offset.h, gc_10_1_0_offset.h,
//! nbio_2_3_offset.h, mmhub_2_0_0_offset.h
//!
//! SOC15 register addressing:
//!   byte_offset = (IP_BASE_SEGMENT + register_index) * 4
//!
//! IP block base segments for Navi 10:
//!   NBIO  SEG0=0x0000  SEG1=0x0014  SEG2=0x0D20  SEG3=0x10400
//!   GC    SEG0=0x1260  SEG1=0x12440
//!   HDP   SEG0=0x0D80
//!   MMHUB SEG0=0x1A000 (indirect access only, beyond 256KB BAR)
//!   MP1   SEG0=0x16000 (indirect access only)
//!   OSSSYS SEG0=0x000A
//!
/// Registers within 2MB BAR2 (0x1FFFFF) → direct MMIO
/// Registers beyond 2MB → indirect via PCIE_INDEX2/DATA2 (SMN address)

// ═══════════════════════════════════════════════════════════════════════════════
// MMIO Index/Data — Indirect Register Access
// ═══════════════════════════════════════════════════════════════════════════════

/// MMIO Index register — write target register address here
pub const MM_INDEX: u32 = 0x0000;
/// MMIO Data register — read/write register value here
pub const MM_DATA: u32 = 0x0004;
/// MMIO Index Hi — upper bits for extended register addressing
/// mmBIF_BX_DEV0_EPF0_VF0_MM_INDEX_HI = DWORD 0x0006, BASE_IDX=0 → byte 0x0018
pub const MM_INDEX_HI: u32 = 0x0018;

/// Navi 10 (SOC15 / NBIO 2.3): PCIE_INDEX2 — preferred indirect index port
/// mmPCIE_INDEX2 = NBIO_BASE(0) + 0x000E → byte offset 0x38
pub const PCIE_INDEX2: u32 = 0x0038;
/// Navi 10 (SOC15 / NBIO 2.3): PCIE_DATA2 — preferred indirect data port
/// mmPCIE_DATA2 = NBIO_BASE(0) + 0x000F → byte offset 0x3C
pub const PCIE_DATA2: u32 = 0x003C;

// ═══════════════════════════════════════════════════════════════════════════════
// BIF / NBIO — Bus Interface / North Bridge I/O
// NBIO_BASE_SEG0 = 0x0000, SEG1 = 0x0014, SEG2 = 0x0D20
// ═══════════════════════════════════════════════════════════════════════════════

/// BIF BX PF0 — HDP flush (NBIO SEG0 + reg)
pub const BIF_BX_PF0_GPU_HDP_FLUSH_REQ: u32 = 0x0106 * 4;
pub const BIF_BX_PF0_GPU_HDP_FLUSH_DONE: u32 = 0x0107 * 4;

/// BIOS scratch registers (NBIO SEG1=0x0014 + regBIOS_SCRATCH_x)
/// regBIOS_SCRATCH_0 = 0x05C9, byte = (0x0014 + 0x05C9) * 4 = 0x1774
pub const SCRATCH_REG0: u32 = (0x0014 + 0x05C9) * 4; // 0x1774
pub const SCRATCH_REG1: u32 = (0x0014 + 0x05CA) * 4; // 0x1778
pub const SCRATCH_REG2: u32 = (0x0014 + 0x05CB) * 4; // 0x177C
pub const SCRATCH_REG3: u32 = (0x0014 + 0x05CC) * 4; // 0x1780
pub const SCRATCH_REG4: u32 = (0x0014 + 0x05CD) * 4; // 0x1784
pub const SCRATCH_REG5: u32 = (0x0014 + 0x05CE) * 4; // 0x1788
pub const SCRATCH_REG6: u32 = (0x0014 + 0x05CF) * 4; // 0x178C
pub const SCRATCH_REG7: u32 = (0x0014 + 0x05D0) * 4; // 0x1790

// ═══════════════════════════════════════════════════════════════════════════════
// GRBM — Graphics Register Bus Manager
// GFX8 (Polaris): registers are at absolute dword index * 4, NO base offset.
// The GC_BASE_SEG0 = 0x1260 base only applies to GFX9+ (Vega/Navi).
// ═══════════════════════════════════════════════════════════════════════════════

/// Helper: GC register byte offset (dword index → byte offset)
const fn gc0(reg: u32) -> u32 { reg * 4 }

/// GRBM status — tells you if the graphics engine is idle/busy
/// regGRBM_STATUS = 0x2004, byte = 0x2004*4 = 0x8010
pub const GRBM_STATUS: u32 = gc0(0x2004);
/// GRBM status 2
pub const GRBM_STATUS2: u32 = gc0(0x2005);
/// GRBM soft reset — write bits to reset individual blocks
pub const GRBM_SOFT_RESET: u32 = gc0(0x2008);
/// GRBM GFX index — select SE/SA/instance for register reads
/// GFX8: mmGRBM_GFX_INDEX = 0xC200, byte = 0x30800
pub const GRBM_GFX_INDEX: u32 = 0xC200 * 4;

// GRBM_STATUS bits
pub const GRBM_STATUS_GUI_ACTIVE: u32 = 1 << 31;
pub const GRBM_STATUS_CP_BUSY: u32 = 1 << 29;
pub const GRBM_STATUS_CP_COHERENCY_BUSY: u32 = 1 << 28;
pub const GRBM_STATUS_GDS_BUSY: u32 = 1 << 27;
pub const GRBM_STATUS_BCI_BUSY: u32 = 1 << 23;
pub const GRBM_STATUS_SPI_BUSY: u32 = 1 << 22;
pub const GRBM_STATUS_TA_BUSY: u32 = 1 << 14;
pub const GRBM_STATUS_DB_BUSY: u32 = 1 << 12;
pub const GRBM_STATUS_CB_BUSY: u32 = 1 << 11;
pub const GRBM_STATUS_PA_BUSY: u32 = 1 << 7;

// ═══════════════════════════════════════════════════════════════════════════════
// GC — Graphics Core (GFX10 / RDNA 1)
// GC_BASE_SEG0 = 0x1260, GC_BASE_SEG1 = 0x12440
// ═══════════════════════════════════════════════════════════════════════════════

/// Helper: GC SEG1 register byte offset (many GC regs use SEG1)
const fn gc1(reg: u32) -> u32 { (0x12440 + reg) * 4 }

/// GC version/identification register
/// regGRBM_CHIP_REVISION = 0x2001, BASE_IDX=0
pub const GC_VERSION: u32 = gc0(0x2001);

/// Shader Array configuration
/// regCC_GC_SHADER_ARRAY_CONFIG = 0x260C, BASE_IDX=0
pub const CC_GC_SHADER_ARRAY_CONFIG: u32 = gc0(0x260C);
/// User shader array config (active CU mask)
/// regGC_USER_SHADER_ARRAY_CONFIG = 0x260D, BASE_IDX=0
pub const GC_USER_SHADER_ARRAY_CONFIG: u32 = gc0(0x260D);

/// GC CAC (dynamic power) weight for CUs
pub const GC_CAC_WEIGHT_CU_0: u32 = gc0(0x260E);

/// Compute Pipe control — CP_ME_CNTL
/// regCP_ME_CNTL = 0x21B6, BASE_IDX=0
pub const CP_ME_CNTL: u32 = gc0(0x21B6);
/// Ring buffer for GFX ring
/// regCP_RB0_BASE = 0x2010, BASE_IDX=0
pub const CP_RB0_BASE: u32 = gc0(0x2010);
pub const CP_RB0_CNTL: u32 = gc0(0x2011);
pub const CP_RB0_RPTR: u32 = gc0(0x2012);
pub const CP_RB0_WPTR: u32 = gc0(0x2013);

/// RLC (Run List Controller) registers — many in GC SEG1
/// regRLC_CNTL = 0x4C00, BASE_IDX=1
pub const RLC_CNTL: u32 = gc1(0x4C00);
/// regRLC_STAT = 0x4C04, BASE_IDX=1
pub const RLC_STAT: u32 = gc1(0x4C04);
pub const RLC_PG_CNTL: u32 = gc1(0x4C44);
/// regRLC_GPM_UCODE_DATA = 0x5B69, BASE_IDX=1 (indirect: beyond 256KB)
pub const RLC_GPM_UCODE_DATA: u32 = gc1(0x5B69);
/// RLC safe mode → regRLC_SAFE_MODE = 0x4C02, BASE_IDX=1
pub const RLC_SAFE_MODE: u32 = gc1(0x4C02);

// ═══════════════════════════════════════════════════════════════════════════════
// MC / MMHUB — Memory Controller / Memory Hub
// ═══════════════════════════════════════════════════════════════════════════════
// NOTE: On Navi 10 (MMHUB 2.0, SOC15), MMHUB registers are at register indices
// beyond the 256KB direct MMIO window. These MUST use indirect access
// (MM_INDEX/MM_DATA). The byte offsets are computed as:
//   (MMHUB_BASE_SEG0 + reg_offset) * 4 = (0x1A000 + offset) * 4

/// Navi 10 SOC15: MMMMC_VM_FB_LOCATION_BASE — byte offset for indirect access
/// MMHUB_BASE(0x1A000) + 0x086C = reg 0x1A86C, byte 0x6A1B0
pub const MMHUB_FB_LOCATION_BASE: u32 = 0x6A1B0;
/// Navi 10 SOC15: MMMMC_VM_FB_LOCATION_TOP
/// MMHUB_BASE + 0x086D = byte 0x6A1B4
pub const MMHUB_FB_LOCATION_TOP: u32 = 0x6A1B4;
/// Navi 10 SOC15: MMMMC_VM_AGP_TOP
/// MMHUB_BASE + 0x086E = byte 0x6A1B8
pub const MMHUB_AGP_TOP: u32 = 0x6A1B8;
/// Navi 10 SOC15: MMMMC_VM_AGP_BOT
/// MMHUB_BASE + 0x086F = byte 0x6A1BC
pub const MMHUB_AGP_BOT: u32 = 0x6A1BC;
/// Navi 10 SOC15: MMMMC_VM_AGP_BASE
/// MMHUB_BASE + 0x0870 = byte 0x6A1C0
pub const MMHUB_AGP_BASE: u32 = 0x6A1C0;
/// Navi 10 SOC15: MMMMC_VM_FB_OFFSET
/// MMHUB_BASE + 0x0857 = byte 0x6A15C
pub const MMHUB_FB_OFFSET: u32 = 0x6A15C;

// Legacy GFX9/Vega offsets (kept for reference, DO NOT use on Navi 10)
/// Memory controller VM framebuffer location (base, in 1MB units) — GFX9 ONLY
pub const MC_VM_FB_LOCATION_BASE: u32 = 0x2024;
/// Memory controller VM framebuffer location (top, in 1MB units) — GFX9 ONLY
pub const MC_VM_FB_LOCATION_TOP: u32 = 0x2028;
/// Memory controller VM AGP base — GFX9 ONLY
pub const MC_VM_AGP_BASE: u32 = 0x202C;
/// Memory controller VM AGP top — GFX9 ONLY
pub const MC_VM_AGP_TOP: u32 = 0x2030;
/// Memory controller VM AGP bot — GFX9 ONLY
pub const MC_VM_AGP_BOT: u32 = 0x2034;

/// Navi 10 SOC15: mmRCC_DEV0_EPF0_RCC_CONFIG_MEMSIZE (NBIO 2.3)
/// DWORD 0x00C3, BASE_IDX=2 → (NBIO_SEG2(0x0D20) + 0x00C3) * 4 = 0x378C
/// Returns VRAM size in MB
/// NOTE: On some hardware, direct MMIO at this offset returns garbage.
/// Use indirect access via MM_INDEX/MM_DATA with DWORD offset 0x0DE3.
pub const RCC_CONFIG_MEMSIZE: u32 = 0x378C;
/// DWORD index for indirect access via MM_INDEX/MM_DATA
pub const RCC_CONFIG_MEMSIZE_DWORD: u32 = 0x0D20 + 0x00C3; // = 0x0DE3

/// Legacy GFX9: CONFIG_MEMSIZE (in MB) — GFX9 ONLY
pub const CONFIG_MEMSIZE: u32 = 0x5428;

/// MC ARB RAM configuration (memory type, width, etc.) — GFX9
pub const MC_ARB_RAMCFG: u32 = 0x9D8;

/// MC SEQ misc — memory type identification — GFX9
pub const MC_SEQ_MISC0: u32 = 0xA80;

/// MMHUB VM configuration — GFX9
pub const MMHUB_VM_FB_OFFSET: u32 = 0x31B4;

// ═══════════════════════════════════════════════════════════════════════════════
// SMU / MP1 — System Management Unit
// ═══════════════════════════════════════════════════════════════════════════════

// ── SMU v11 (Navi 10 / RDNA 1) — Indirect via PCIE_INDEX2/DATA2 ────────────
/// SMU firmware version (via MP1 mailbox) — SMN address
pub const MP1_SMN_C2PMSG_58: u32 = 0x3B8E8;
/// SMU message interface — SMN address
pub const MP1_SMN_C2PMSG_66: u32 = 0x3B908;
/// SMU response — SMN address
pub const MP1_SMN_C2PMSG_90: u32 = 0x3B968;
/// SMU argument — SMN address
pub const MP1_SMN_C2PMSG_82: u32 = 0x3B948;

// ── SMU v7 (Polaris / GCN 4) — Direct MMIO ────────────────────────────────
// Source: Linux smu_7_1_3_d.h — dword indices × 4 = byte offsets
// Protocol: Wait RESP!=0 → Write 0→RESP → [Write param→ARG] → Write msg→MESSAGE → Wait RESP!=0

/// SMU v7 message register — write SMU command ID here (uint16_t)
/// mmSMC_MESSAGE_0 = dword 0x94 → byte 0x250
pub const SMC_MESSAGE_0: u32 = 0x94 * 4;
/// SMU v7 response register — 0x01=OK, 0xFF=fail, 0xFE=unknown
/// mmSMC_RESP_0 = dword 0x95 → byte 0x254
pub const SMC_RESP_0: u32 = 0x95 * 4;
/// SMU v7 message argument / return value register
/// mmSMC_MSG_ARG_0 = dword 0xA4 → byte 0x290
pub const SMC_MSG_ARG_0: u32 = 0xA4 * 4;
/// SMC indirect index register (SRAM access bank 11)
/// mmSMC_IND_INDEX_11 = dword 0x1AC → byte 0x6B0
pub const SMC_IND_INDEX_11: u32 = 0x1AC * 4;
/// SMC indirect data register (SRAM access bank 11)
/// mmSMC_IND_DATA_11 = dword 0x1AD → byte 0x6B4
pub const SMC_IND_DATA_11: u32 = 0x1AD * 4;
/// SMC indirect access control
/// mmSMC_IND_ACCESS_CNTL = dword 0x92 → byte 0x248
pub const SMC_IND_ACCESS_CNTL: u32 = 0x92 * 4;
/// SMC indirect index register (bank 0 — for SYSCON / full address space)
/// mmSMC_IND_INDEX_0 = dword 0x80 → byte 0x200
pub const SMC_IND_INDEX_0: u32 = 0x80 * 4;
/// SMC indirect data register (bank 0 — for SYSCON / full address space)
/// mmSMC_IND_DATA_0 = dword 0x81 → byte 0x204
pub const SMC_IND_DATA_0: u32 = 0x81 * 4;
/// SMC indirect index register (bank 1 — used by AtomBIOS IIO sub-program 5)
/// mmSMC_IND_INDEX_1 = dword 0x82 → byte 0x208
pub const SMC_IND_INDEX_1: u32 = 0x82 * 4;
/// SMC indirect data register (bank 1 — used by AtomBIOS IIO sub-program 5)
/// mmSMC_IND_DATA_1 = dword 0x83 → byte 0x20C
pub const SMC_IND_DATA_1: u32 = 0x83 * 4;

// SMU v7 response codes
pub const SMU7_RESP_OK: u32 = 0x01;
pub const SMU7_RESP_CMD_FAIL: u32 = 0xFF;
pub const SMU7_RESP_CMD_UNKNOWN: u32 = 0xFE;
pub const SMU7_RESP_CMD_BAD_PREREQ: u32 = 0xFD;

// SMU v7 message IDs (from smu7_ppsmc.h — uint16_t values)
pub const PPSMC_MSG_API_GetSclkFrequency: u16 = 0x200;
pub const PPSMC_MSG_API_GetMclkFrequency: u16 = 0x201;
pub const PPSMC_MSG_API_GetSclkBusy: u16 = 0x202;
pub const PPSMC_MSG_API_GetMclkBusy: u16 = 0x203;
pub const PPSMC_MSG_API_GetAsicPower: u16 = 0x204;
pub const PPSMC_MSG_SetFanRpmMax: u16 = 0x205;
pub const PPSMC_MSG_DPM_Enable: u16 = 0x14E;
pub const PPSMC_MSG_DPM_Disable: u16 = 0x14F;
pub const PPSMC_MSG_SCLKDPM_SetEnabledMask: u16 = 0x145;
pub const PPSMC_MSG_MCLKDPM_SetEnabledMask: u16 = 0x146;
pub const PPSMC_MSG_EnableClockGatingFeature: u16 = 0x260;
pub const PPSMC_MSG_DisableClockGatingFeature: u16 = 0x261;

// ── GFX power-gating control (Polaris SMU7) ─────────────────────────────────
// Linux: drivers/gpu/drm/amd/pm/legacy-dpm/smu7_ppsmc.h
/// Wake all GFX CUs out of power-gating (param = 0 → all CUs).
/// Required before MEC1 can execute its boot trampoline on power-managed boards.
pub const PPSMC_MSG_GFX_CU_PG_DISABLE: u16 = 0x281;
/// Re-enable GFX CU power-gating (param = num_active_cus).
pub const PPSMC_MSG_GFX_CU_PG_ENABLE:  u16 = 0x280;
/// Legacy power-up GFX block (sent by Linux on resume).
pub const PPSMC_MSG_PowerUpGfx:        u16 = 0x06;
/// Legacy power-down GFX block.
pub const PPSMC_MSG_PowerDownGfx:      u16 = 0x05;
/// Force GFX DPM out of low-power forced level.
pub const PPSMC_MSG_NoForcedLevel:     u16 = 0x18;
/// Disable Cache Auto-Cleaning (often required before MEC ucode load).
pub const PPSMC_MSG_DisableCac:        u16 = 0x14;
pub const PPSMC_MSG_EnableCac:         u16 = 0x13;

// Clock-gating feature mask bits (param to EnableClockGatingFeature)
/// Mask covering the entire GFX clock-gating tree (MGCG/CGCG/CGLS).
pub const PWR_CG_FLAG_GFX:             u32 = 0x0000_03E0;

// SMC SRAM indirect addresses (accessed via IND_INDEX_11/IND_DATA_11)
/// SMC syscon reset control — bit 0 (rst_reg): 1=assert reset, 0=de-assert
pub const IX_SMC_SYSCON_RESET_CNTL: u32 = 0x80000000;
/// SMC syscon clock control 0 — bit 0 (ck_disable): 0=clock enabled
pub const IX_SMC_SYSCON_CLOCK_CNTL_0: u32 = 0x80000004;
/// SMC syscon misc control — bit 1 (pre_fetcher_en): MUST be 1 before reset
/// for the SMC instruction prefetcher to fetch ucode after de-assert.
/// Reference: drivers/gpu/drm/amd/pm/powerplay/smumgr/polaris10_smumgr.c
pub const IX_SMC_SYSCON_MISC_CNTL: u32 = 0x80000010;
pub const SMC_SYSCON_MISC_CNTL_PRE_FETCHER_EN_MASK: u32 = 0x2;
/// SMC program counter — >= 0x20100 means SMC firmware is running
pub const IX_SMC_PC_C: u32 = 0x80000370;
/// Firmware flags in SMC SRAM — bit 0 = INTERRUPTS_ENABLED
pub const IX_FIRMWARE_FLAGS: u32 = 0x3f000;
/// RCU_UC_EVENTS — bit 7 (boot_seq_done)
pub const IX_RCU_UC_EVENTS: u32 = 0xC0000004;
/// AUTO_INCREMENT_IND_11 bit in SMC_IND_ACCESS_CNTL
pub const SMC_IND_ACCESS_AUTO_INCREMENT_11: u32 = 1 << 11;
/// AUTO_INCREMENT_IND_0 bit in SMC_IND_ACCESS_CNTL
pub const SMC_IND_ACCESS_AUTO_INCREMENT_0: u32 = 1 << 0;
/// SMC RAM end address (address space limit)
pub const SMC_RAM_END: u32 = 0x40000;

// ── SMU v7 Protection Mode Registers (indirect via bank 0) ─────────────────
// Source: Linux smu_7_1_3_d.h + smu_7_1_3_sh_mask.h

/// ixSMU_STATUS — protection mode validation result
/// Bit 0: SMU_DONE, Bit 1: SMU_PASS
pub const IX_SMU_STATUS: u32 = 0xe0003088;
/// ixSMU_FIRMWARE — firmware mode/status register
/// Bit 16: SMU_MODE (1=protection mode), Bit 17: SMU_SEL (security key)
pub const IX_SMU_FIRMWARE: u32 = 0xe00030a4;
/// ixSMU_INPUT_DATA — DMA address for FW upload (low 32 bits)
/// Boot ROM reads FW from this MC address during protection mode start
pub const IX_SMU_INPUT_DATA: u32 = 0xe00030b8;

// SMU_STATUS bit masks
pub const SMU_STATUS_DONE_MASK: u32 = 0x1;
pub const SMU_STATUS_PASS_MASK: u32 = 0x2;
// SMU_FIRMWARE bit masks (smu_7_1_3_sh_mask.h)
//   bits  0..7   = SMU_NUMBER_OF_DPMS
//   bits  8..15  = SMU_MINOR_VERSION
//   bits 16..23  = SMU_MAJOR_VERSION
//   bits 24..27  = SMU_MODE     (0 = non-protected, 1 = protected)
//   bits 28..31  = SMU_SEL      (0 = SMU_SK,        1 = SMU)
pub const SMU_FIRMWARE_MODE_MASK: u32 = 0x0F000000;
pub const SMU_FIRMWARE_SEL_MASK: u32 = 0xF0000000;
// RCU_UC_EVENTS bit masks
pub const RCU_UC_EVENTS_BOOT_SEQ_DONE_MASK: u32 = 0x80;
pub const RCU_UC_EVENTS_INTERRUPTS_ENABLED_MASK: u32 = 0x10000;
// FIRMWARE_FLAGS bit masks
pub const FIRMWARE_FLAGS_INTERRUPTS_ENABLED_MASK: u32 = 0x1;
// SMC_SYSCON_CLOCK_CNTL_0 bit masks
pub const SMC_SYSCON_CLOCK_CNTL_0_CK_DISABLE_MASK: u32 = 0x1;

// SMU v7 message IDs — protection mode
/// PPSMC_MSG_Test — trigger FW validation in protection mode
#[allow(non_upper_case_globals)]
pub const PPSMC_MSG_Test: u16 = 0x01;
/// SMU firmware header location in SRAM (protection mode entry offset)
pub const SMU7_FIRMWARE_HEADER_LOCATION: u32 = 0x20000;

/// Clock pin control (reference clock info)
pub const CG_CLKPIN_CNTL_2: u32 = 0x0168;

// ═══════════════════════════════════════════════════════════════════════════════
// HDP — Host Data Path
// ═══════════════════════════════════════════════════════════════════════════════

/// HDP byte base for Navi10 (SOC15): HDP_BASE_INST0_SEG0 = 0x0F20 → byte 0x3C80
const fn hdp0(reg: u32) -> u32 { 0x3C80 + reg * 4 }

/// HDP Host Path Control (Navi10 SOC15: HDP_BASE + 0x00CC)
pub const HDP_HOST_PATH_CNTL: u32 = hdp0(0x00CC);
/// HDP nonsurface base (Navi10 SOC15: HDP_BASE + 0x0040)
pub const HDP_NONSURFACE_BASE: u32 = hdp0(0x0040);
/// HDP read cache invalidate (Navi10 SOC15: HDP_BASE + 0x00D1)
/// Writing 1 invalidates the HDP read cache (forces re-read from memory)
pub const HDP_READ_CACHE_INVALIDATE: u32 = hdp0(0x00D1);

// ═══════════════════════════════════════════════════════════════════════════════
// DCN 2.0 — Display Core Next (Phase 2: display output)
// ═══════════════════════════════════════════════════════════════════════════════

/// DCN register block — full register set for display engine
pub mod dcn {
    // ── DCN Version ──────────────────────────────────────────────────────────
    /// DCN version identification register
    pub const DCN_VERSION: u32 = 0x0001_2000;
    
    // ── DMCUB (Display Micro Controller Unit B) ─────────────────────────────
    /// DMCUB firmware status
    pub const DMCUB_STATUS: u32 = 0x0003_1000;
    /// DMCUB scratch registers
    pub const DMCUB_SCRATCH0: u32 = 0x0003_1010;
    
    // ── OTG (Output Timing Generator) — 6 pipes ────────────────────────────
    // Each OTG pipe is spaced 0x400 apart in the register map
    pub const OTG0_BASE: u32 = 0x0001_B000;
    pub const OTG_PIPE_STRIDE: u32 = 0x400;
    
    // Offsets from OTGx base:
    pub const OTG_CONTROL_OFFSET: u32 = 0x00;
    pub const OTG_H_TOTAL_OFFSET: u32 = 0x04;
    pub const OTG_H_BLANK_START_END_OFFSET: u32 = 0x08;
    pub const OTG_H_SYNC_A_OFFSET: u32 = 0x0C;
    pub const OTG_H_SYNC_A_CNTL_OFFSET: u32 = 0x10;
    pub const OTG_V_TOTAL_OFFSET: u32 = 0x1C;
    pub const OTG_V_BLANK_START_END_OFFSET: u32 = 0x20;
    pub const OTG_V_SYNC_A_OFFSET: u32 = 0x24;
    pub const OTG_V_SYNC_A_CNTL_OFFSET: u32 = 0x28;
    pub const OTG_INTERLACE_CONTROL_OFFSET: u32 = 0x2C;
    pub const OTG_BLANK_CONTROL_OFFSET: u32 = 0x38;
    pub const OTG_PIXEL_RATE_CNTL_OFFSET: u32 = 0x60;
    pub const OTG_STATUS_OFFSET: u32 = 0x70;
    pub const OTG_STATUS_POSITION_OFFSET: u32 = 0x74;
    pub const OTG_NOM_VERT_POSITION_OFFSET: u32 = 0x78;
    pub const OTG_BLACK_COLOR_OFFSET: u32 = 0x80;
    pub const OTG_CLOCK_CONTROL_OFFSET: u32 = 0xA0;
    pub const OTG_VERTICAL_INTERRUPT0_POSITION_OFFSET: u32 = 0xB0;
    pub const OTG_VERTICAL_INTERRUPT1_POSITION_OFFSET: u32 = 0xB4;
    pub const OTG_VERTICAL_INTERRUPT2_POSITION_OFFSET: u32 = 0xB8;
    pub const OTG_MASTER_EN_OFFSET: u32 = 0xFC;
    
    // ── HUBP (Hub Pipe) — 6 pipes ──────────────────────────────────────────
    // Each HUBP pipe is spaced 0x400 apart  
    pub const HUBP0_BASE: u32 = 0x0001_A000;
    pub const HUBP_PIPE_STRIDE: u32 = 0x400;
    
    // Offsets from HUBPx base:
    pub const HUBP_SURFACE_CONFIG_OFFSET: u32 = 0x00;
    pub const HUBP_SURFACE_ADDR_OFFSET: u32 = 0x04;
    pub const HUBP_SURFACE_ADDR_HIGH_OFFSET: u32 = 0x08;
    pub const HUBP_SURFACE_PITCH_OFFSET: u32 = 0x0C;
    pub const HUBP_SURFACE_SIZE_OFFSET: u32 = 0x10;
    pub const HUBP_SURFACE_ADDR_C_OFFSET: u32 = 0x14;   // Chroma plane address
    pub const HUBP_SURFACE_ADDR_HIGH_C_OFFSET: u32 = 0x18;
    pub const HUBP_SURFACE_PITCH_C_OFFSET: u32 = 0x1C;
    pub const HUBP_DCSURF_TILING_CONFIG_OFFSET: u32 = 0x30;
    pub const HUBP_DCSURF_PRI_VIEWPORT_DIMENSION_OFFSET: u32 = 0x40;
    pub const HUBP_DCSURF_PRI_VIEWPORT_START_OFFSET: u32 = 0x44;
    pub const HUBP_DCHUBP_CNTL_OFFSET: u32 = 0x60;
    pub const HUBP_DCHUBP_REQ_SIZE_CONFIG_OFFSET: u32 = 0x64;
    pub const HUBP_DCSURF_FLIP_CONTROL_OFFSET: u32 = 0x68;
    
    // ── DPP (Display Pipe & Plane) — 6 pipes ───────────────────────────────
    pub const DPP0_BASE: u32 = 0x0001_9000;
    pub const DPP_PIPE_STRIDE: u32 = 0x400;
    
    pub const DPP_CONTROL_OFFSET: u32 = 0x00;
    pub const DPP_CM_ICSC_CONTROL_OFFSET: u32 = 0x40;
    pub const DPP_CM_DGAM_CONTROL_OFFSET: u32 = 0x50;
    pub const DPP_CM_RGAM_CONTROL_OFFSET: u32 = 0x60;
    
    // ── MPC (Multi-Pipe Combiner) ───────────────────────────────────────────
    pub const MPC_BASE: u32 = 0x0001_8000;
    
    pub const MPC_OUT_MUX_OFFSET: u32 = 0x00;   // Output MUX — maps pipes to OPP
    pub const MPC_OCSC_MODE_OFFSET: u32 = 0x10;
    
    // ── OPP (Output Pixel Processing) ───────────────────────────────────────
    pub const OPP0_BASE: u32 = 0x0001_C000;
    pub const OPP_STRIDE: u32 = 0x400;
    
    pub const OPP_PIPE_CONTROL_OFFSET: u32 = 0x00;
    pub const OPP_FMT_BIT_DEPTH_CONTROL_OFFSET: u32 = 0x10;
    pub const OPP_FMT_DITHER_CONTROL_OFFSET: u32 = 0x14;
    pub const OPP_FMT_CLAMP_CONTROL_OFFSET: u32 = 0x18;
    
    // ── DIG (Display Interface Generator) / Encoders ────────────────────────
    pub const DIG0_BASE: u32 = 0x0001_D000;
    pub const DIG_STRIDE: u32 = 0x400;
    
    pub const DIG_FE_CNTL_OFFSET: u32 = 0x00;
    pub const DIG_BE_CNTL_OFFSET: u32 = 0x04;
    pub const DIG_BE_EN_CNTL_OFFSET: u32 = 0x08;
    pub const DIG_OUTPUT_FORMAT_OFFSET: u32 = 0x10;
    
    // DP-specific encoder registers
    pub const DP_LINK_CNTL_OFFSET: u32 = 0x40;
    pub const DP_PIXEL_FORMAT_OFFSET: u32 = 0x44;
    pub const DP_MSA_COLORIMETRY_OFFSET: u32 = 0x48;
    pub const DP_CONFIG_OFFSET: u32 = 0x4C;
    pub const DP_VID_STREAM_CNTL_OFFSET: u32 = 0x50;
    pub const DP_STEER_FIFO_OFFSET: u32 = 0x54;
    pub const DP_VID_TIMING_OFFSET: u32 = 0x58;
    pub const DP_SEC_CNTL_OFFSET: u32 = 0x60;
    
    // HDMI-specific encoder registers
    pub const HDMI_CONTROL_OFFSET: u32 = 0x80;
    pub const HDMI_STATUS_OFFSET: u32 = 0x84;
    pub const HDMI_AUDIO_PACKET_CONTROL_OFFSET: u32 = 0x88;
    pub const HDMI_GC_OFFSET: u32 = 0x90;
    pub const HDMI_INFOFRAME_CONTROL0_OFFSET: u32 = 0xA0;
    pub const HDMI_INFOFRAME_CONTROL1_OFFSET: u32 = 0xA4;
    
    // ── HPD (Hot Plug Detect) ───────────────────────────────────────────────
    pub const HPD0_BASE: u32 = 0x0001_E000;
    pub const HPD_STRIDE: u32 = 0x20;
    
    pub const HPD_INT_STATUS_OFFSET: u32 = 0x00;
    pub const HPD_INT_CONTROL_OFFSET: u32 = 0x04;
    pub const HPD_CONTROL_OFFSET: u32 = 0x08;
    
    // ── AUX (Auxiliary Channel for DisplayPort) ─────────────────────────────
    pub const AUX0_BASE: u32 = 0x0001_E100;
    pub const AUX_STRIDE: u32 = 0x100;
    
    pub const AUX_CONTROL_OFFSET: u32 = 0x00;
    pub const AUX_SW_CONTROL_OFFSET: u32 = 0x04;
    pub const AUX_SW_STATUS_OFFSET: u32 = 0x08;
    pub const AUX_SW_DATA_OFFSET: u32 = 0x0C;
    pub const AUX_LS_STATUS_OFFSET: u32 = 0x14;
    pub const AUX_DPHY_TX_REF_CONTROL_OFFSET: u32 = 0x20;
    pub const AUX_DPHY_TX_CONTROL_OFFSET: u32 = 0x24;
    
    // ── PHY / UNIPHY (Physical Layer) ───────────────────────────────────────
    pub const UNIPHY0_BASE: u32 = 0x0001_F000;
    pub const UNIPHY_STRIDE: u32 = 0x400;
    
    pub const UNIPHY_CHANNEL_XBAR_CNTL_OFFSET: u32 = 0x00;
    pub const UNIPHY_PLL_CONTROL1_OFFSET: u32 = 0x10;
    pub const UNIPHY_PLL_CONTROL2_OFFSET: u32 = 0x14;
    pub const UNIPHY_PLL_SS_CNTL_OFFSET: u32 = 0x20;
    pub const UNIPHY_PLL_FBDIV_OFFSET: u32 = 0x28;
    
    // ── DCCG (Display Clock Controller / Generator) ─────────────────────────
    pub const DCCG_BASE: u32 = 0x0001_2100;
    
    pub const DCCG_DISPCLK_CNTL_OFFSET: u32 = 0x00;
    pub const DCCG_DPPCLK_CNTL_OFFSET: u32 = 0x04;
    pub const DCCG_REFCLK_CNTL_OFFSET: u32 = 0x10;
    pub const DCCG_DPSTREAMCLK_CNTL_OFFSET: u32 = 0x14;
    pub const DCCG_HDMICHARCLK_CNTL_OFFSET: u32 = 0x18;
    pub const DCCG_SYMCLK_CNTL_OFFSET: u32 = 0x20;
    pub const DCCG_OTGCLK_CNTL_OFFSET: u32 = 0x24;
}

// ═══════════════════════════════════════════════════════════════════════════════
// SDMA — System DMA Engines (Navi 10 — 2 engines: SDMA0 + SDMA1)
// ═══════════════════════════════════════════════════════════════════════════════
//
// SDMA engines perform asynchronous DMA transfers independently of the
// Graphics/Compute command processors. Each engine has its own ring buffer
// and can run in parallel with GFX/Compute workloads.
//
// Navi 10 SDMA register layout (per engine):
//   SDMA0: base 0x4980 (GFX ring) / 0x4A00 (Page ring)
//   SDMA1: base 0x4A80 (GFX ring) / 0x4B00 (Page ring)
//
// Each ring has: CNTL, BASE, BASE_HI, RPTR, RPTR_HI, WPTR, WPTR_HI,
//                WPTR_POLL_ADDR, DOORBELL, etc.
//
// SDMA packet opcodes use a different format than PM4:
//   DW0: [31:28]=0 (always), [27:26]=sub_op, [25:8]=op, [7:0]=extra
//

/// SDMA engine instance offsets (from MMIO base)
/// SDMA0_SEG0=0x1260, byte = 0x4980 | SDMA1_SEG0=0x1860, byte = 0x6180
pub const SDMA0_BASE: u32 = 0x1260 * 4; // 0x4980
pub const SDMA1_BASE: u32 = 0x1860 * 4; // 0x6180

/// Stride between SDMA0 and SDMA1 register sets
pub const SDMA_ENGINE_STRIDE: u32 = SDMA1_BASE - SDMA0_BASE; // 0x1800

/// Number of SDMA engines on Navi 10
pub const SDMA_NUM_ENGINES: usize = 2;

// ── SDMA GFX Ring Buffer Registers (offsets from SDMAx_BASE) ────────────────
// Source: gc_10_1_0_offset.h — mmSDMA0_GFX_* dword indices, *4 for byte offset
// All offsets are relative to SDMAx_BASE (GC_SEG0 base, NOT the first SDMA reg)

/// Ring buffer control — mmSDMA0_GFX_RB_CNTL = 0x0080
pub const SDMA_GFX_RB_CNTL: u32 = 0x0080 * 4;
/// Ring buffer base address [31:0] — mmSDMA0_GFX_RB_BASE = 0x0081
pub const SDMA_GFX_RB_BASE: u32 = 0x0081 * 4;
/// Ring buffer base address [63:32] — mmSDMA0_GFX_RB_BASE_HI = 0x0082
pub const SDMA_GFX_RB_BASE_HI: u32 = 0x0082 * 4;
/// Ring buffer read pointer — mmSDMA0_GFX_RB_RPTR = 0x0083
pub const SDMA_GFX_RB_RPTR: u32 = 0x0083 * 4;
/// Ring buffer read pointer [63:32] — mmSDMA0_GFX_RB_RPTR_HI = 0x0084
pub const SDMA_GFX_RB_RPTR_HI: u32 = 0x0084 * 4;
/// Ring buffer write pointer — mmSDMA0_GFX_RB_WPTR = 0x0085
pub const SDMA_GFX_RB_WPTR: u32 = 0x0085 * 4;
/// Ring buffer write pointer [63:32] — mmSDMA0_GFX_RB_WPTR_HI = 0x0086
pub const SDMA_GFX_RB_WPTR_HI: u32 = 0x0086 * 4;
/// Write pointer poll control — mmSDMA0_GFX_RB_WPTR_POLL_CNTL = 0x0087
pub const SDMA_GFX_RB_WPTR_POLL_CNTL: u32 = 0x0087 * 4;
/// RPTR report address [63:32] — mmSDMA0_GFX_RB_RPTR_ADDR_HI = 0x0088
pub const SDMA_GFX_RB_RPTR_ADDR_HI: u32 = 0x0088 * 4;
/// RPTR report address [31:0] — mmSDMA0_GFX_RB_RPTR_ADDR_LO = 0x0089
pub const SDMA_GFX_RB_RPTR_ADDR_LO: u32 = 0x0089 * 4;
/// Indirect buffer control — mmSDMA0_GFX_IB_CNTL = 0x008a
pub const SDMA_GFX_IB_CNTL: u32 = 0x008a * 4;
/// Doorbell control — mmSDMA0_GFX_DOORBELL = 0x0092
pub const SDMA_GFX_DOORBELL: u32 = 0x0092 * 4;
/// Doorbell offset — mmSDMA0_GFX_DOORBELL_OFFSET = 0x00ab
pub const SDMA_GFX_DOORBELL_OFFSET: u32 = 0x00ab * 4;
/// WPTR poll address [63:32] — mmSDMA0_GFX_RB_WPTR_POLL_ADDR_HI = 0x00b2
pub const SDMA_GFX_RB_WPTR_POLL_ADDR_HI: u32 = 0x00b2 * 4;
/// WPTR poll address [31:0] — mmSDMA0_GFX_RB_WPTR_POLL_ADDR_LO = 0x00b3
pub const SDMA_GFX_RB_WPTR_POLL_ADDR_LO: u32 = 0x00b3 * 4;
/// Minor pointer update — mmSDMA0_GFX_MINOR_PTR_UPDATE = 0x00b5
pub const SDMA_GFX_MINOR_PTR_UPDATE: u32 = 0x00b5 * 4;

// ── SDMA Engine-Level Status/Control (SOC15 Navi 10) ────────────────────────
// Source: gc_10_1_0_offset.h — mmSDMA0_* dword indices
// SDMA0 GC_SEG0 base = 0x1260, SDMA1 = 0x1860 (stride = 0x600 dwords)

/// SDMA0 status register — mmSDMA0_STATUS_REG = 0x0025
pub const SDMA0_STATUS_REG: u32 = (0x1260 + 0x0025) * 4;
/// SDMA0 chicken bits — mmSDMA0_CHICKEN_BITS = 0x001d
pub const SDMA0_CHICKEN_BITS: u32 = (0x1260 + 0x001d) * 4;
/// SDMA0 clock gating control — mmSDMA0_CLK_CTRL = 0x001b
pub const SDMA0_CLK_CTRL: u32 = (0x1260 + 0x001b) * 4;
/// SDMA0 power cntl — mmSDMA0_POWER_CNTL = 0x001a
pub const SDMA0_POWER_CNTL: u32 = (0x1260 + 0x001a) * 4;
/// SDMA0 F32 microengine halt — mmSDMA0_F32_CNTL = 0x002a
pub const SDMA0_F32_CNTL: u32 = (0x1260 + 0x002a) * 4;
/// SDMA0 version — mmSDMA0_VERSION = 0x0035
pub const SDMA0_VERSION: u32 = (0x1260 + 0x0035) * 4;

/// SDMA0 top-level control — mmSDMA0_CNTL = 0x001C
pub const SDMA0_CNTL: u32 = (0x1260 + 0x001C) * 4;
/// SDMA1 top-level control
pub const SDMA1_CNTL: u32 = (0x1860 + 0x001C) * 4;

/// SDMA UTCL1 control — relative offset from SDMAx_BASE
/// mmSDMA0_UTCL1_CNTL = 0x003c (gc_10_1_0_offset.h)
pub const SDMA_UTCL1_CNTL: u32 = 0x003c * 4;
/// SDMA UTCL1 page — mmSDMA0_UTCL1_PAGE = 0x003d
pub const SDMA_UTCL1_PAGE: u32 = 0x003d * 4;

/// SDMA1 status register — mmSDMA1_STATUS_REG = 0x0625 → relative 0x0025
pub const SDMA1_STATUS_REG: u32 = (0x1860 + 0x0025) * 4;
/// SDMA1 chicken bits — mmSDMA1_CHICKEN_BITS = 0x061d → relative 0x001d
pub const SDMA1_CHICKEN_BITS: u32 = (0x1860 + 0x001d) * 4;
/// SDMA1 F32 halt — mmSDMA1_F32_CNTL = 0x062a → relative 0x002a
pub const SDMA1_F32_CNTL: u32 = (0x1860 + 0x002a) * 4;

// ── SDMA_GFX_RB_CNTL bit definitions ───────────────────────────────────────

/// Ring buffer size in log2 DWORDs (bits [6:1])
/// E.g., 12 → 4096 DWORDs = 16KB ring
pub const SDMA_RB_CNTL_RB_SIZE_SHIFT: u32 = 1;
pub const SDMA_RB_CNTL_RB_SIZE_MASK: u32 = 0x3F << 1;
/// Ring buffer read pointer writeback enable (bit 12)
pub const SDMA_RB_CNTL_RPTR_WRITEBACK_ENABLE: u32 = 1 << 12;
/// Ring buffer enable (bit 0)
pub const SDMA_RB_CNTL_RB_ENABLE: u32 = 1 << 0;
/// Ring buffer privilege (bit 23) — REQUIRED for bare-metal without IOMMU
pub const SDMA_RB_CNTL_RB_PRIV: u32 = 1 << 23;
/// Ring VMID (bits [19:16]) — 0 for bare-metal (no IOMMU)
pub const SDMA_RB_CNTL_RB_VMID_SHIFT: u32 = 16;
/// RPTR writeback interval timer (bits [27:20])
pub const SDMA_RB_CNTL_RPTR_WRITEBACK_TIMER_SHIFT: u32 = 20;

// ── SDMA_STATUS_REG bit definitions ─────────────────────────────────────────

/// SDMA engine idle (all queues drained)
pub const SDMA_STATUS_IDLE: u32 = 1 << 0;
/// SDMA had a context-switch
pub const SDMA_STATUS_CTXSW: u32 = 1 << 4;

// ── SDMA Packet Opcodes ─────────────────────────────────────────────────────
//
// SDMA v5.0 (Navi 10) packet format — DIFFERENT from PM4:
//   DW0: [7:0]=opcode, [15:8]=sub_opcode, [31:16]=extra (varies by opcode)
//
// Reference: drivers/gpu/drm/amd/amdgpu/navi10_sdma_pkt_open.h

/// NOP — padding / alignment
pub const SDMA_OP_NOP: u32 = 0;
/// COPY — linear DMA copy (sub_op=0=linear, sub_op=1=tiled, sub_op=3=SOA, sub_op=4=dirty_page)
pub const SDMA_OP_COPY: u32 = 1;
/// WRITE — write immediate data to GPU address
pub const SDMA_OP_WRITE: u32 = 2;
/// INDIRECT_BUFFER — jump to indirect buffer
pub const SDMA_OP_INDIRECT: u32 = 4;
/// FENCE — write a fence value to memory
pub const SDMA_OP_FENCE: u32 = 5;
/// TRAP — generate interrupt on completion
pub const SDMA_OP_TRAP: u32 = 6;
/// POLL_REGMEM — poll a register/memory location
pub const SDMA_OP_POLL_REGMEM: u32 = 8;
/// TIMESTAMP — write GPU timestamp to memory
pub const SDMA_OP_TIMESTAMP: u32 = 13;
/// SRBM_WRITE — write a register via SRBM bridge
pub const SDMA_OP_SRBM_WRITE: u32 = 14;
/// CONST_FILL — fill memory with a constant value (32-bit)
pub const SDMA_OP_CONST_FILL: u32 = 11;
/// GCR — Global Cache Request (flush/invalidate)
pub const SDMA_OP_GCR: u32 = 17;

/// SDMA COPY sub-operations
pub const SDMA_COPY_SUB_LINEAR: u32 = 0;
pub const SDMA_COPY_SUB_TILED: u32 = 1;
pub const SDMA_COPY_SUB_SOA: u32 = 3;
pub const SDMA_COPY_SUB_DIRTY_PAGE: u32 = 4;

/// SDMA WRITE sub-operations
pub const SDMA_WRITE_SUB_LINEAR: u32 = 0;
pub const SDMA_WRITE_SUB_TILED: u32 = 1;

// ═══════════════════════════════════════════════════════════════════════════════
// PM4 / CP — Command Processor packet types (for Phase 4: command submission)
// ═══════════════════════════════════════════════════════════════════════════════

/// PM4 packet header types
pub const PM4_TYPE0: u32 = 0;
pub const PM4_TYPE2: u32 = 2;
pub const PM4_TYPE3: u32 = 3;

/// Common PM4 opcodes
pub const PM4_NOP: u32 = 0x10;
pub const PM4_SET_SH_REG: u32 = 0x76;
pub const PM4_SET_CONTEXT_REG: u32 = 0x69;
pub const PM4_INDIRECT_BUFFER: u32 = 0x3F;
pub const PM4_DMA_DATA: u32 = 0x50;
pub const PM4_ACQUIRE_MEM: u32 = 0x58;
pub const PM4_RELEASE_MEM: u32 = 0x49;
pub const PM4_EVENT_WRITE: u32 = 0x46;
pub const PM4_EVENT_WRITE_EOP: u32 = 0x47;
pub const PM4_DISPATCH_DIRECT: u32 = 0x15;
pub const PM4_DRAW_INDEX_AUTO: u32 = 0x2D;

// ═══════════════════════════════════════════════════════════════════════════════
// Compute Queue Registers (GFX10 / RDNA 1) — Phase 3/4
// ═══════════════════════════════════════════════════════════════════════════════

/// Compute ring 0 registers (MEC pipe 0, queue 0)
pub const CP_HQD_ACTIVE: u32 = 0x3E54;
pub const CP_HQD_PQ_BASE_LO: u32 = 0x3E58;
pub const CP_HQD_PQ_BASE_HI: u32 = 0x3E5C;
pub const CP_HQD_PQ_RPTR: u32 = 0x3E60;
pub const CP_HQD_PQ_WPTR_LO: u32 = 0x3E64;
pub const CP_HQD_PQ_WPTR_HI: u32 = 0x3E68;
pub const CP_HQD_PQ_CONTROL: u32 = 0x3E6C;
pub const CP_HQD_PQ_DOORBELL_CONTROL: u32 = 0x3E74;
pub const CP_HQD_DEQUEUE_REQUEST: u32 = 0x3E80;

// ═══════════════════════════════════════════════════════════════════════════════
// Compute Queue Registers (GFX8 / Polaris / GCN 4)
// Source: gfx_8_0_d.h — dword offsets × 4 = byte offsets
// Accessed after selecting pipe/queue via SRBM_GFX_CNTL
// ═══════════════════════════════════════════════════════════════════════════════

pub const SRBM_GFX_CNTL_V8: u32 = 0x0391 * 4;  // 0x0E44 — mmSRBM_GFX_CNTL (oss_3_0_d.h)

// GFX7/8 (GCN 3/4, Polaris) HQD register offsets from Linux gfx_7_0_d.h
// These are dword offsets multiplied by 4 for byte addressing.
// Accessed per-pipe/queue via SRBM_GFX_CNTL muxing.
pub const CP_HQD_ACTIVE_V8: u32            = 0x3247 * 4;  // 0xC91C
pub const CP_HQD_VMID_V8: u32              = 0x3248 * 4;  // 0xC920
pub const CP_HQD_PQ_BASE_V8: u32           = 0x324D * 4;  // 0xC934
pub const CP_HQD_PQ_BASE_HI_V8: u32        = 0x324E * 4;  // 0xC938
pub const CP_HQD_PQ_RPTR_V8: u32           = 0x324F * 4;  // 0xC93C
pub const CP_HQD_PQ_RPTR_REPORT_ADDR_V8: u32    = 0x3250 * 4;  // 0xC940
pub const CP_HQD_PQ_RPTR_REPORT_ADDR_HI_V8: u32 = 0x3251 * 4;  // 0xC944
pub const CP_HQD_PQ_DOORBELL_CONTROL_V8: u32 = 0x3254 * 4;  // 0xC950
pub const CP_HQD_PQ_QUANTUM_V8: u32        = 0x324C * 4;  // 0xC930
pub const CP_HQD_PQ_WPTR_V8: u32           = 0x3255 * 4;  // 0xC954
pub const CP_HQD_PQ_CONTROL_V8: u32        = 0x3256 * 4;  // 0xC958
pub const CP_HQD_DEQUEUE_REQUEST_V8: u32    = 0x325D * 4;  // 0xC974
// EOP: use HPD (Hardware Pipe Descriptor) EOP registers from gfx_7_0_d.h
pub const CP_HQD_EOP_BASE_ADDR_V8: u32     = 0x3241 * 4;  // 0xC904 (CP_HPD_EOP_BASE_ADDR)
pub const CP_HQD_EOP_BASE_ADDR_HI_V8: u32  = 0x3242 * 4;  // 0xC908 (CP_HPD_EOP_BASE_ADDR_HI)
pub const CP_HQD_EOP_CONTROL_V8: u32       = 0x3244 * 4;  // 0xC910 (CP_HPD_EOP_CONTROL);

/// MEC (Micro Engine Compute) engine control
/// Linux: mmCP_MEC_CNTL = 0x208D, byte offset 0x8234
/// bit 30 = MEC_ME1_HALT, bit 28 = MEC_ME2_HALT
pub const CP_MEC_CNTL: u32 = gc0(0x208D);
/// MEC1 F32 program counter — non-zero & changing → MEC1 ucode is executing.
/// Linux: mmCP_MEC1_INSTR_PNTR = 0x208E
pub const CP_MEC1_INSTR_PNTR: u32 = gc0(0x208E);
/// MEC2 F32 program counter (parity check vs MEC1).
/// Linux: mmCP_MEC2_INSTR_PNTR = 0x208F
pub const CP_MEC2_INSTR_PNTR: u32 = gc0(0x208F);
/// MEC doorbell range
pub const CP_MEC_DOORBELL_RANGE_LOWER: u32 = 0x8260;
pub const CP_MEC_DOORBELL_RANGE_UPPER: u32 = 0x8264;

/// Compute shader SH registers (set via PM4_SET_SH_REG, base 0x2C00)
pub const SH_REG_BASE: u32 = 0x2C00;
pub const COMPUTE_PGM_LO: u32 = 0x2E0C;      // Shader program address low
pub const COMPUTE_PGM_HI: u32 = 0x2E10;      // Shader program address high
pub const COMPUTE_PGM_RSRC1: u32 = 0x2E14;   // VGPR/SGPR counts, float mode
pub const COMPUTE_PGM_RSRC2: u32 = 0x2E18;   // LDS, scratch, user SGPR count
pub const COMPUTE_PGM_RSRC3: u32 = 0x2E1C;   // Wave limit, shared VGPR count
pub const COMPUTE_NUM_THREAD_X: u32 = 0x2E20; // Threads per workgroup X
pub const COMPUTE_NUM_THREAD_Y: u32 = 0x2E24; // Threads per workgroup Y
pub const COMPUTE_NUM_THREAD_Z: u32 = 0x2E28; // Threads per workgroup Z
pub const COMPUTE_USER_DATA_0: u32 = 0x2E40;  // User SGPR data (buffer descriptors, etc.)
pub const COMPUTE_USER_DATA_1: u32 = 0x2E44;
pub const COMPUTE_USER_DATA_2: u32 = 0x2E48;
pub const COMPUTE_USER_DATA_3: u32 = 0x2E4C;
pub const COMPUTE_USER_DATA_4: u32 = 0x2E50;
pub const COMPUTE_USER_DATA_5: u32 = 0x2E54;
pub const COMPUTE_USER_DATA_6: u32 = 0x2E58;
pub const COMPUTE_USER_DATA_7: u32 = 0x2E5C;
pub const COMPUTE_USER_DATA_8: u32 = 0x2E60;
pub const COMPUTE_USER_DATA_9: u32 = 0x2E64;
pub const COMPUTE_USER_DATA_10: u32 = 0x2E68;
pub const COMPUTE_USER_DATA_11: u32 = 0x2E6C;
pub const COMPUTE_USER_DATA_12: u32 = 0x2E70;  // GEMM: s12 = M
pub const COMPUTE_USER_DATA_13: u32 = 0x2E74;  // GEMM: s13 = N
pub const COMPUTE_USER_DATA_14: u32 = 0x2E78;  // GEMM: s14 = K
pub const COMPUTE_USER_DATA_15: u32 = 0x2E7C;  // GEMM: reserved
pub const COMPUTE_RESOURCE_LIMITS: u32 = 0x2E30;
pub const COMPUTE_DISPATCH_INITIATOR: u32 = 0x2E34;

/// Fence / event registers
pub const CP_COHER_CNTL: u32 = 0xA0A0;
pub const CP_COHER_SIZE: u32 = 0xA0A4;
pub const CP_COHER_BASE: u32 = 0xA0A8;
