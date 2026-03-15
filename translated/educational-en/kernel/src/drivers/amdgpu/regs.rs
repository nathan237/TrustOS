//! AMD GPU Register Definitions — Navi 10 (RDNA 1)
//!
//! Register offsets for direct MMIO and indirect (SMN) access.
//! Based on Linux amdgpu driver headers and AMD register documentation.
//!
//! Register namespaces:
//! - MM_*     : MMIO window control (index/data for indirect access)
//! - GC_*     : Graphics Core (shader engines, CUs)
//! - GRBM_*   : Graphics Register Bus Manager (status, soft reset)
//! - MC_*     : Memory Controller (VRAM, VM, arbitration)
//! - BIF_*    : Bus Interface (PCIe, BAR, GART)
//! - SMU/MP1_*: System Management Unit (clocks, power, firmware)
//! - DCN_*    : Display Core Next (scanout, CRTC, encoders) [Phase 2+]
//! - SDMA_*   : System DMA engines [Phase 3+]
//! - GFX_*    : Graphics command processor [Phase 4+]

// ═══════════════════════════════════════════════════════════════════════════════
// MMIO Index/Data — Indirect Register Access
// ═══════════════════════════════════════════════════════════════════════════════

/// MMIO Index register — write target register address here
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const MM_INDEX: u32 = 0x0000;
/// MMIO Data register — read/write register value here
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const MM_DATA: u32 = 0x0004;
/// MMIO Index Hi — upper 32 bits for 64-bit indexed access
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const MM_INDEX_HI: u32 = 0x0008;

// ═══════════════════════════════════════════════════════════════════════════════
// BIF / NBIO — Bus Interface / North Bridge I/O
// ═══════════════════════════════════════════════════════════════════════════════

/// BIF BX PF0 — device identification
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const BIF_BX_PF0_GPU_HDP_FLUSH_REQUEST: u32 = 0x0106;
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const BIF_BX_PF0_GPU_HDP_FLUSH_DONE: u32 = 0x0107;

/// NBIO scratch registers (programmed by VBIOS)
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const SCRATCH_REG0: u32 = 0x2040;
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const SCRATCH_REG1: u32 = 0x2044;
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const SCRATCH_REG2: u32 = 0x2048;
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const SCRATCH_REG3: u32 = 0x204C;
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const SCRATCH_REG4: u32 = 0x2050;
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const SCRATCH_REG5: u32 = 0x2054;
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const SCRATCH_REG6: u32 = 0x2058;
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const SCRATCH_REG7: u32 = 0x205C;

// ═══════════════════════════════════════════════════════════════════════════════
// GRBM — Graphics Register Bus Manager
// ═══════════════════════════════════════════════════════════════════════════════

/// GRBM status — tells you if the graphics engine is idle/busy
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const GRBM_STATUS: u32 = 0x8010;
/// GRBM status 2
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const GRBM_STATUS2: u32 = 0x8014;
/// GRBM soft reset — write bits to reset individual blocks
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const GRBM_SOFT_RESET: u32 = 0x8020;
/// GRBM GFX index — select SE/SA/instance for register reads
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const GRBM_GFX_INDEX: u32 = 0x9000;

// GRBM_STATUS bits
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const GRBM_STATUS_GUI_ACTIVE: u32 = 1 << 31;
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const GRBM_STATUS_CP_BUSY: u32 = 1 << 29;
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const GRBM_STATUS_CP_COHERENCY_BUSY: u32 = 1 << 28;
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const GRBM_STATUS_GDS_BUSY: u32 = 1 << 27;
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const GRBM_STATUS_BCI_BUSY: u32 = 1 << 23;
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const GRBM_STATUS_SPI_BUSY: u32 = 1 << 22;
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const GRBM_STATUS_TA_BUSY: u32 = 1 << 14;
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const GRBM_STATUS_DB_BUSY: u32 = 1 << 12;
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const GRBM_STATUS_CALLBACK_BUSY: u32 = 1 << 11;
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const GRBM_STATUS_PA_BUSY: u32 = 1 << 7;

// ═══════════════════════════════════════════════════════════════════════════════
// GC — Graphics Core (GFX10 / RDNA 1)
// ═══════════════════════════════════════════════════════════════════════════════

/// GC version/identification register
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const GC_VERSION: u32 = 0x9000;

/// Shader Array configuration
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const CC_GC_SHADER_ARRAY_CONFIG: u32 = 0x9830;
/// User shader array config (active CU mask)
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const GC_USER_SHADER_ARRAY_CONFIG: u32 = 0x9834;

/// GC CAC (dynamic power) weight for CUs
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const GC_CAC_WEIGHT_CU_0: u32 = 0x9838;

/// Compute Pipe control
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const CP_ME_CNTL: u32 = 0x86D8;
/// Ring buffer base/size for GFX ring
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const CP_RB0_BASE: u32 = 0x8040;
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const CP_RB0_CNTL: u32 = 0x8044;
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const CP_RB0_RPTR: u32 = 0x8048;
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const CP_RB0_WPTR: u32 = 0x804C;

/// RLC (Run List Controller) Power Gating control
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const RLC_PAGE_CNTL: u32 = 0x4E00;
/// RLC firmware version
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const RLC_GPM_UCODE_DATA: u32 = 0x4E24;
/// RLC safe mode
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const RLC_SAFE_MODE: u32 = 0x4E0C;

// ═══════════════════════════════════════════════════════════════════════════════
// MC / MMHUB — Memory Controller / Memory Hub
// ═══════════════════════════════════════════════════════════════════════════════

/// Memory controller VM framebuffer location (base, in 1MB units)
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const MC_VM_FRAMEBUFFER_LOCATION_BASE: u32 = 0x2024;
/// Memory controller VM framebuffer location (top, in 1MB units)
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const MC_VM_FRAMEBUFFER_LOCATION_TOP: u32 = 0x2028;
/// Memory controller VM AGP base
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const MC_VM_AGP_BASE: u32 = 0x202C;
/// Memory controller VM AGP top
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const MC_VM_AGP_TOP: u32 = 0x2030;
/// Memory controller VM AGP bot
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const MC_VM_AGP_BOT: u32 = 0x2034;

/// Legacy memory size register (in MB)
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const CONFIG_MEMSIZE: u32 = 0x5428;

/// MC ARB RAM configuration (memory type, width, etc.)
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const MC_ARB_RAMCFG: u32 = 0x9D8;

/// MC SEQ misc — memory type identification
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const MC_SEQUENCE_MISC0: u32 = 0xA80;

/// MMHUB VM configuration
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const MMHUB_VM_FRAMEBUFFER_OFFSET: u32 = 0x31B4;

// ═══════════════════════════════════════════════════════════════════════════════
// SMU / MP1 — System Management Unit
// ═══════════════════════════════════════════════════════════════════════════════

/// SMU firmware version (via MP1 mailbox)
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const MP1_SMN_C2PMSG_58: u32 = 0x3B8E8;
/// SMU message interface
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const MP1_SMN_C2PMSG_66: u32 = 0x3B908;
/// SMU response
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const MP1_SMN_C2PMSG_90: u32 = 0x3B968;
/// SMU argument
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const MP1_SMN_C2PMSG_82: u32 = 0x3B948;

/// Clock pin control (reference clock info)
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const CG_CLKPIN_CNTL_2: u32 = 0x0168;

// ═══════════════════════════════════════════════════════════════════════════════
// HDP — Host Data Path
// ═══════════════════════════════════════════════════════════════════════════════

/// HDP Host Path Enable
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const HDP_HOST_PATH_CNTL: u32 = 0x2C00;
/// HDP nonsurface base
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const HDP_NONSURFACE_BASE: u32 = 0x2C04;
/// HDP memory coherency flush control
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const HDP_MEMORY_COHERENCY_FLUSH_CNTL: u32 = 0x2C14;

// ═══════════════════════════════════════════════════════════════════════════════
// DCN 2.0 — Display Core Next (Phase 2: display output)
// ═══════════════════════════════════════════════════════════════════════════════

/// DCN register block — full register set for display engine
pub mod dcn {
    // ── DCN Version ──────────────────────────────────────────────────────────
    /// DCN version identification register
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const DCN_VERSION: u32 = 0x0001_2000;
    
    // ── DMCUB (Display Micro Controller Unit B) ─────────────────────────────
    /// DMCUB firmware status
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const DMCUB_STATUS: u32 = 0x0003_1000;
    /// DMCUB scratch registers
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const DMCUB_SCRATCH0: u32 = 0x0003_1010;
    
    // ── OTG (Output Timing Generator) — 6 pipes ────────────────────────────
    // Each OTG pipe is spaced 0x400 apart in the register map
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const OTG0_BASE: u32 = 0x0001_B000;
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const OTG_PIPE_STRIDE: u32 = 0x400;
    
    // Offsets from OTGx base:
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const OTG_CONTROL_OFFSET: u32 = 0x00;
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const OTG_H_TOTAL_OFFSET: u32 = 0x04;
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const OTG_H_BLANK_START_END_OFFSET: u32 = 0x08;
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const OTG_H_SYNC_A_OFFSET: u32 = 0x0C;
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const OTG_H_SYNC_A_CNTL_OFFSET: u32 = 0x10;
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const OTG_V_TOTAL_OFFSET: u32 = 0x1C;
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const OTG_V_BLANK_START_END_OFFSET: u32 = 0x20;
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const OTG_V_SYNC_A_OFFSET: u32 = 0x24;
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const OTG_V_SYNC_A_CNTL_OFFSET: u32 = 0x28;
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const OTG_INTERLACE_CONTROL_OFFSET: u32 = 0x2C;
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const OTG_BLANK_CONTROL_OFFSET: u32 = 0x38;
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const OTG_PIXEL_RATE_CNTL_OFFSET: u32 = 0x60;
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const OTG_STATUS_OFFSET: u32 = 0x70;
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const OTG_STATUS_POSITION_OFFSET: u32 = 0x74;
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const OTG_NOM_VERT_POSITION_OFFSET: u32 = 0x78;
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const OTG_BLACK_COLOR_OFFSET: u32 = 0x80;
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const OTG_CLOCK_CONTROL_OFFSET: u32 = 0xA0;
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const OTG_VERTICAL_INTERRUPT0_POSITION_OFFSET: u32 = 0xB0;
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const OTG_VERTICAL_INTERRUPT1_POSITION_OFFSET: u32 = 0xB4;
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const OTG_VERTICAL_INTERRUPT2_POSITION_OFFSET: u32 = 0xB8;
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const OTG_MASTER_EN_OFFSET: u32 = 0xFC;
    
    // ── HUBP (Hub Pipe) — 6 pipes ──────────────────────────────────────────
    // Each HUBP pipe is spaced 0x400 apart  
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const HUBP0_BASE: u32 = 0x0001_A000;
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const HUBP_PIPE_STRIDE: u32 = 0x400;
    
    // Offsets from HUBPx base:
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const HUBP_SURFACE_CONFIG_OFFSET: u32 = 0x00;
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const HUBP_SURFACE_ADDRESS_OFFSET: u32 = 0x04;
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const HUBP_SURFACE_ADDRESS_HIGH_OFFSET: u32 = 0x08;
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const HUBP_SURFACE_PITCH_OFFSET: u32 = 0x0C;
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const HUBP_SURFACE_SIZE_OFFSET: u32 = 0x10;
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const HUBP_SURFACE_ADDRESS_C_OFFSET: u32 = 0x14;   // Chroma plane address
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const HUBP_SURFACE_ADDRESS_HIGH_C_OFFSET: u32 = 0x18;
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const HUBP_SURFACE_PITCH_C_OFFSET: u32 = 0x1C;
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const HUBP_DCSURF_TILING_CONFIG_OFFSET: u32 = 0x30;
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const HUBP_DCSURF_PRI_VIEWPORT_DIMENSION_OFFSET: u32 = 0x40;
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const HUBP_DCSURF_PRI_VIEWPORT_START_OFFSET: u32 = 0x44;
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const HUBP_DCHUBP_CNTL_OFFSET: u32 = 0x60;
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const HUBP_DCHUBP_REQUEST_SIZE_CONFIG_OFFSET: u32 = 0x64;
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const HUBP_DCSURF_FLIP_CONTROL_OFFSET: u32 = 0x68;
    
    // ── DPP (Display Pipe & Plane) — 6 pipes ───────────────────────────────
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const DPP0_BASE: u32 = 0x0001_9000;
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const DPP_PIPE_STRIDE: u32 = 0x400;
    
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const DPP_CONTROL_OFFSET: u32 = 0x00;
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const DPP_CM_ICSC_CONTROL_OFFSET: u32 = 0x40;
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const DPP_CM_DGAM_CONTROL_OFFSET: u32 = 0x50;
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const DPP_CM_RGAM_CONTROL_OFFSET: u32 = 0x60;
    
    // ── MPC (Multi-Pipe Combiner) ───────────────────────────────────────────
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const MPC_BASE: u32 = 0x0001_8000;
    
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const MPC_OUT_MUX_OFFSET: u32 = 0x00;   // Output MUX — maps pipes to OPP
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const MPC_OCSC_MODE_OFFSET: u32 = 0x10;
    
    // ── OPP (Output Pixel Processing) ───────────────────────────────────────
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const OPP0_BASE: u32 = 0x0001_C000;
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const OPP_STRIDE: u32 = 0x400;
    
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const OPP_PIPE_CONTROL_OFFSET: u32 = 0x00;
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const OPP_FORMATTER_BIT_DEPTH_CONTROL_OFFSET: u32 = 0x10;
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const OPP_FORMATTER_DITHER_CONTROL_OFFSET: u32 = 0x14;
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const OPP_FORMATTER_CLAMP_CONTROL_OFFSET: u32 = 0x18;
    
    // ── DIG (Display Interface Generator) / Encoders ────────────────────────
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const DIG0_BASE: u32 = 0x0001_D000;
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const DIG_STRIDE: u32 = 0x400;
    
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const DIG_FE_CNTL_OFFSET: u32 = 0x00;
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const DIG_BE_CNTL_OFFSET: u32 = 0x04;
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const DIG_BE_EN_CNTL_OFFSET: u32 = 0x08;
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const DIG_OUTPUT_FORMAT_OFFSET: u32 = 0x10;
    
    // DP-specific encoder registers
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const DP_LINK_CNTL_OFFSET: u32 = 0x40;
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const DP_PIXEL_FORMAT_OFFSET: u32 = 0x44;
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const DP_MSA_COLORIMETRY_OFFSET: u32 = 0x48;
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const DP_CONFIG_OFFSET: u32 = 0x4C;
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const DP_VID_STREAM_CNTL_OFFSET: u32 = 0x50;
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const DP_STEER_FIFO_OFFSET: u32 = 0x54;
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const DP_VID_TIMING_OFFSET: u32 = 0x58;
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const DP_SECTOR_CNTL_OFFSET: u32 = 0x60;
    
    // HDMI-specific encoder registers
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const HDMI_CONTROL_OFFSET: u32 = 0x80;
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const HDMI_STATUS_OFFSET: u32 = 0x84;
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const HDMI_AUDIO_PACKET_CONTROL_OFFSET: u32 = 0x88;
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const HDMI_GC_OFFSET: u32 = 0x90;
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const HDMI_INFOFRAME_CONTROL0_OFFSET: u32 = 0xA0;
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const HDMI_INFOFRAME_CONTROL1_OFFSET: u32 = 0xA4;
    
    // ── HPD (Hot Plug Detect) ───────────────────────────────────────────────
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const HPD0_BASE: u32 = 0x0001_E000;
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const HPD_STRIDE: u32 = 0x20;
    
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const HPD_INT_STATUS_OFFSET: u32 = 0x00;
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const HPD_INT_CONTROL_OFFSET: u32 = 0x04;
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const HPD_CONTROL_OFFSET: u32 = 0x08;
    
    // ── AUX (Auxiliary Channel for DisplayPort) ─────────────────────────────
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const AUX0_BASE: u32 = 0x0001_E100;
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const AUX_STRIDE: u32 = 0x100;
    
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const AUX_CONTROL_OFFSET: u32 = 0x00;
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const AUX_SOFTWARE_CONTROL_OFFSET: u32 = 0x04;
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const AUX_SOFTWARE_STATUS_OFFSET: u32 = 0x08;
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const AUX_SOFTWARE_DATA_OFFSET: u32 = 0x0C;
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const AUX_LS_STATUS_OFFSET: u32 = 0x14;
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const AUX_DPHY_TRANSMIT_REF_CONTROL_OFFSET: u32 = 0x20;
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const AUX_DPHY_TRANSMIT_CONTROL_OFFSET: u32 = 0x24;
    
    // ── PHY / UNIPHY (Physical Layer) ───────────────────────────────────────
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const UNIPHY0_BASE: u32 = 0x0001_F000;
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const UNIPHY_STRIDE: u32 = 0x400;
    
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const UNIPHY_CHANNEL_XBAR_CNTL_OFFSET: u32 = 0x00;
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const UNIPHY_PLL_CONTROL1_OFFSET: u32 = 0x10;
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const UNIPHY_PLL_CONTROL2_OFFSET: u32 = 0x14;
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const UNIPHY_PLL_SS_CNTL_OFFSET: u32 = 0x20;
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const UNIPHY_PLL_FBDIV_OFFSET: u32 = 0x28;
    
    // ── DCCG (Display Clock Controller / Generator) ─────────────────────────
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const DCCG_BASE: u32 = 0x0001_2100;
    
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const DCCG_DISPCLK_CNTL_OFFSET: u32 = 0x00;
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const DCCG_DPPCLK_CNTL_OFFSET: u32 = 0x04;
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const DCCG_REFCLK_CNTL_OFFSET: u32 = 0x10;
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const DCCG_DPSTREAMCLK_CNTL_OFFSET: u32 = 0x14;
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const DCCG_HDMICHARCLK_CNTL_OFFSET: u32 = 0x18;
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const DCCG_SYMCLK_CNTL_OFFSET: u32 = 0x20;
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const DCCG_OTGCLK_CNTL_OFFSET: u32 = 0x24;
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
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const SDMA0_BASE: u32 = 0x0000_4980;
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const SDMA1_BASE: u32 = 0x0000_4A80;

/// Stride between SDMA0 and SDMA1 register sets
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const SDMA_ENGINE_STRIDE: u32 = SDMA1_BASE - SDMA0_BASE;

/// Number of SDMA engines on Navi 10
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const SDMA_NUMBER_ENGINES: usize = 2;

// ── SDMA GFX Ring Buffer Registers (offsets from SDMAx_BASE) ────────────────

/// Ring buffer control: size (log2), enable, etc.
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const SDMA_GFX_RB_CNTL: u32 = 0x00;
/// Ring buffer base address [31:0] (256-byte aligned → bits [31:8] valid)
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const SDMA_GFX_RB_BASE: u32 = 0x04;
/// Ring buffer base address [63:32]
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const SDMA_GFX_RB_BASE_HI: u32 = 0x08;
/// Ring buffer read pointer (hardware-managed)
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const SDMA_GFX_RB_RPTR: u32 = 0x18;
/// Ring buffer read pointer [63:32]
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const SDMA_GFX_RB_RPTR_HI: u32 = 0x1C;
/// Ring buffer write pointer (software-managed, doorbell-writeable)
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const SDMA_GFX_RB_WPTR: u32 = 0x20;
/// Ring buffer write pointer [63:32]
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const SDMA_GFX_RB_WPTR_HI: u32 = 0x24;
/// Write pointer poll address [31:0] — GPU writes RPTR here for CPU to read
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const SDMA_GFX_RB_WPTR_POLL_ADDRESS_LO: u32 = 0x28;
/// Write pointer poll address [63:32]
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const SDMA_GFX_RB_WPTR_POLL_ADDRESS_HI: u32 = 0x2C;
/// Ring buffer read pointer report address [31:0]
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const SDMA_GFX_RB_RPTR_ADDRESS_LO: u32 = 0x30;
/// Ring buffer read pointer report address [63:32]
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const SDMA_GFX_RB_RPTR_ADDRESS_HI: u32 = 0x34;
/// Doorbell control for this ring
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const SDMA_GFX_DOORBELL: u32 = 0x38;
/// Doorbell offset within the BAR4 doorbell page
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const SDMA_GFX_DOORBELL_OFFSET: u32 = 0x3C;

// ── SDMA Engine-Level Status/Control (absolute offsets from MMIO base) ──────

/// SDMA0 status register — engine idle/busy + error flags
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const SDMA0_STATUS_REGISTER: u32 = 0x4D68;
/// SDMA0 chicken bits (enable/disable hardware features)
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const SDMA0_CHICKEN_BITS: u32 = 0x4D6C;
/// SDMA0 clock gating control
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const SDMA0_CLK_CONTROLLER: u32 = 0x4D70;
/// SDMA0 power cntl
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const SDMA0_POWER_CNTL: u32 = 0x4D74;
/// SDMA0 freeze — halt engine for programming
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const SDMA0_F32_CNTL: u32 = 0x4D78;
/// SDMA0 version
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const SDMA0_VERSION: u32 = 0x4D80;

/// SDMA1 status register
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const SDMA1_STATUS_REGISTER: u32 = 0x4E68;
/// SDMA1 chicken bits
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const SDMA1_CHICKEN_BITS: u32 = 0x4E6C;
/// SDMA1 freeze
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const SDMA1_F32_CNTL: u32 = 0x4E78;

// ── SDMA_GFX_RB_CNTL bit definitions ───────────────────────────────────────

/// Ring buffer size in log2 DWORDs (bits [6:1])
/// E.g., 12 → 4096 DWORDs = 16KB ring
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const SDMA_RB_CNTL_RB_SIZE_SHIFT: u32 = 1;
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const SDMA_RB_CNTL_RB_SIZE_MASK: u32 = 0x3F << 1;
/// Ring buffer read pointer writeback enable (bit 12)
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const SDMA_RB_CNTL_RPTR_WRITEBACK_ENABLE: u32 = 1 << 12;
/// Ring buffer enable (bit 0)
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const SDMA_RB_CNTL_RB_ENABLE: u32 = 1 << 0;
/// Ring VMID (bits [19:16]) — 0 for bare-metal (no IOMMU)
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const SDMA_RB_CNTL_RB_VMID_SHIFT: u32 = 16;
/// RPTR writeback interval timer (bits [27:20])
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const SDMA_RB_CNTL_RPTR_WRITEBACK_TIMER_SHIFT: u32 = 20;

// ── SDMA_STATUS_REG bit definitions ─────────────────────────────────────────

/// SDMA engine idle (all queues drained)
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const SDMA_STATUS_IDLE: u32 = 1 << 0;
/// SDMA had a context-switch
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const SDMA_STATUS_CTXSW: u32 = 1 << 4;

// ── SDMA Packet Opcodes ─────────────────────────────────────────────────────
//
// SDMA packets differ from PM4. Format:
//   DW0: [31:28]=0, [27:26]=sub_op, [25:8]=op, [7:0]=extra_info
//
// Reference: drivers/gpu/drm/amd/amdgpu/navi10_sdma_pkt_open.h

/// NOP — padding / alignment
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const SDMA_OPERATION_NOP: u32 = 0;
/// COPY — linear DMA copy (sub_op=0=linear, sub_op=1=tiled, sub_op=3=SOA, sub_op=4=dirty_page)
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const SDMA_OPERATION_COPY: u32 = 1;
/// WRITE — write immediate data to GPU address
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const SDMA_OPERATION_WRITE: u32 = 2;
/// INDIRECT_BUFFER — jump to indirect buffer
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const SDMA_OPERATION_INDIRECT: u32 = 4;
/// FENCE — write a fence value to memory
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const SDMA_OPERATION_FENCE: u32 = 5;
/// TRAP — generate interrupt on completion
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const SDMA_OPERATION_TRAP: u32 = 6;
/// POLL_REGMEM — poll a register/memory location
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const SDMA_OPERATION_POLL_REGMEM: u32 = 8;
/// TIMESTAMP — write GPU timestamp to memory
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const SDMA_OPERATION_TIMESTAMP: u32 = 13;
/// SRBM_WRITE — write a register via SRBM bridge
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const SDMA_OPERATION_SRBM_WRITE: u32 = 14;
/// CONST_FILL — fill memory with a constant value (32-bit)
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const SDMA_OPERATION_CONST_FILL: u32 = 11;
/// GCR — Global Cache Request (flush/invalidate)
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const SDMA_OPERATION_GCR: u32 = 17;

/// SDMA COPY sub-operations
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const SDMA_COPY_SUB_LINEAR: u32 = 0;
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const SDMA_COPY_SUB_TILED: u32 = 1;
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const SDMA_COPY_SUB_SOA: u32 = 3;
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const SDMA_COPY_SUB_DIRTY_PAGE: u32 = 4;

/// SDMA WRITE sub-operations
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const SDMA_WRITE_SUB_LINEAR: u32 = 0;
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const SDMA_WRITE_SUB_TILED: u32 = 1;

// ═══════════════════════════════════════════════════════════════════════════════
// PM4 / CP — Command Processor packet types (for Phase 4: command submission)
// ═══════════════════════════════════════════════════════════════════════════════

/// PM4 packet header types
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const PM4_TYPE0: u32 = 0;
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const PM4_TYPE2: u32 = 2;
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const PM4_TYPE3: u32 = 3;

/// Common PM4 opcodes
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const PM4_NOP: u32 = 0x10;
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const PM4_SET_SH_REGISTER: u32 = 0x76;
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const PM4_SET_CONTEXT_REGISTER: u32 = 0x69;
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const PM4_INDIRECT_BUFFER: u32 = 0x3F;
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const PM4_DMA_DATA: u32 = 0x50;
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const PM4_ACQUIRE_MEMORY: u32 = 0x58;
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const PM4_RELEASE_MEMORY: u32 = 0x49;
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const PM4_EVENT_WRITE: u32 = 0x46;
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const PM4_DISPATCH_DIRECT: u32 = 0x15;
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const PM4_DRAW_INDEX_AUTO: u32 = 0x2D;

// ═══════════════════════════════════════════════════════════════════════════════
// Compute Queue Registers (GFX10 / RDNA 1) — Phase 3/4
// ═══════════════════════════════════════════════════════════════════════════════

/// Compute ring 0 registers (MEC pipe 0, queue 0)
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const CP_HQD_ACTIVE: u32 = 0x3E54;
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const CP_HQD_PQ_BASE_LO: u32 = 0x3E58;
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const CP_HQD_PQ_BASE_HI: u32 = 0x3E5C;
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const CP_HQD_PQ_RPTR: u32 = 0x3E60;
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const CP_HQD_PQ_WPTR_LO: u32 = 0x3E64;
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const CP_HQD_PQ_WPTR_HI: u32 = 0x3E68;
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const CP_HQD_PQ_CONTROL: u32 = 0x3E6C;
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const CP_HQD_PQ_DOORBELL_CONTROL: u32 = 0x3E74;
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const CP_HQD_DEQUEUE_REQUEST: u32 = 0x3E80;

/// MEC (Micro Engine Compute) engine control
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const CP_MEC_CNTL: u32 = 0x8234;
/// MEC doorbell range
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const CP_MEC_DOORBELL_RANGE_LOWER: u32 = 0x8260;
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const CP_MEC_DOORBELL_RANGE_UPPER: u32 = 0x8264;

/// Compute shader SH registers (set via PM4_SET_SH_REG, base 0x2C00)
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const SH_REGISTER_BASE: u32 = 0x2C00;
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const COMPUTE_PGM_LO: u32 = 0x2E0C;      // Shader program address low
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const COMPUTE_PGM_HI: u32 = 0x2E10;      // Shader program address high
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const COMPUTE_PGM_RSRC1: u32 = 0x2E14;   // VGPR/SGPR counts, float mode
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const COMPUTE_PGM_RSRC2: u32 = 0x2E18;   // LDS, scratch, user SGPR count
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const COMPUTE_PGM_RSRC3: u32 = 0x2E1C;   // Wave limit, shared VGPR count
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const COMPUTE_NUMBER_THREAD_X: u32 = 0x2E20; // Threads per workgroup X
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const COMPUTE_NUMBER_THREAD_Y: u32 = 0x2E24; // Threads per workgroup Y
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const COMPUTE_NUMBER_THREAD_Z: u32 = 0x2E28; // Threads per workgroup Z
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const COMPUTE_USER_DATA_0: u32 = 0x2E40;  // User SGPR data (buffer descriptors, etc.)
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const COMPUTE_USER_DATA_1: u32 = 0x2E44;
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const COMPUTE_USER_DATA_2: u32 = 0x2E48;
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const COMPUTE_USER_DATA_3: u32 = 0x2E4C;
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const COMPUTE_USER_DATA_4: u32 = 0x2E50;
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const COMPUTE_USER_DATA_5: u32 = 0x2E54;
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const COMPUTE_USER_DATA_6: u32 = 0x2E58;
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const COMPUTE_USER_DATA_7: u32 = 0x2E5C;
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const COMPUTE_USER_DATA_8: u32 = 0x2E60;
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const COMPUTE_USER_DATA_9: u32 = 0x2E64;
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const COMPUTE_USER_DATA_10: u32 = 0x2E68;
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const COMPUTE_USER_DATA_11: u32 = 0x2E6C;
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const COMPUTE_USER_DATA_12: u32 = 0x2E70;  // GEMM: s12 = M
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const COMPUTE_USER_DATA_13: u32 = 0x2E74;  // GEMM: s13 = N
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const COMPUTE_USER_DATA_14: u32 = 0x2E78;  // GEMM: s14 = K
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const COMPUTE_USER_DATA_15: u32 = 0x2E7C;  // GEMM: reserved
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const COMPUTE_RESOURCE_LIMITS: u32 = 0x2E30;
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const COMPUTE_DISPATCH_INITIATOR: u32 = 0x2E34;

/// Fence / event registers
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const CP_COHER_CNTL: u32 = 0xA0A0;
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const CP_COHER_SIZE: u32 = 0xA0A4;
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const CP_COHER_BASE: u32 = 0xA0A8;
