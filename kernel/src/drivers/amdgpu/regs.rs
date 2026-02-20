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
pub const MM_INDEX: u32 = 0x0000;
/// MMIO Data register — read/write register value here
pub const MM_DATA: u32 = 0x0004;
/// MMIO Index Hi — upper 32 bits for 64-bit indexed access
pub const MM_INDEX_HI: u32 = 0x0008;

// ═══════════════════════════════════════════════════════════════════════════════
// BIF / NBIO — Bus Interface / North Bridge I/O
// ═══════════════════════════════════════════════════════════════════════════════

/// BIF BX PF0 — device identification
pub const BIF_BX_PF0_GPU_HDP_FLUSH_REQ: u32 = 0x0106;
pub const BIF_BX_PF0_GPU_HDP_FLUSH_DONE: u32 = 0x0107;

/// NBIO scratch registers (programmed by VBIOS)
pub const SCRATCH_REG0: u32 = 0x2040;
pub const SCRATCH_REG1: u32 = 0x2044;
pub const SCRATCH_REG2: u32 = 0x2048;
pub const SCRATCH_REG3: u32 = 0x204C;
pub const SCRATCH_REG4: u32 = 0x2050;
pub const SCRATCH_REG5: u32 = 0x2054;
pub const SCRATCH_REG6: u32 = 0x2058;
pub const SCRATCH_REG7: u32 = 0x205C;

// ═══════════════════════════════════════════════════════════════════════════════
// GRBM — Graphics Register Bus Manager
// ═══════════════════════════════════════════════════════════════════════════════

/// GRBM status — tells you if the graphics engine is idle/busy
pub const GRBM_STATUS: u32 = 0x8010;
/// GRBM status 2
pub const GRBM_STATUS2: u32 = 0x8014;
/// GRBM soft reset — write bits to reset individual blocks
pub const GRBM_SOFT_RESET: u32 = 0x8020;
/// GRBM GFX index — select SE/SA/instance for register reads
pub const GRBM_GFX_INDEX: u32 = 0x9000;

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
// ═══════════════════════════════════════════════════════════════════════════════

/// GC version/identification register
pub const GC_VERSION: u32 = 0x9000;

/// Shader Array configuration
pub const CC_GC_SHADER_ARRAY_CONFIG: u32 = 0x9830;
/// User shader array config (active CU mask)
pub const GC_USER_SHADER_ARRAY_CONFIG: u32 = 0x9834;

/// GC CAC (dynamic power) weight for CUs
pub const GC_CAC_WEIGHT_CU_0: u32 = 0x9838;

/// Compute Pipe control
pub const CP_ME_CNTL: u32 = 0x86D8;
/// Ring buffer base/size for GFX ring
pub const CP_RB0_BASE: u32 = 0x8040;
pub const CP_RB0_CNTL: u32 = 0x8044;
pub const CP_RB0_RPTR: u32 = 0x8048;
pub const CP_RB0_WPTR: u32 = 0x804C;

/// RLC (Run List Controller) Power Gating control
pub const RLC_PG_CNTL: u32 = 0x4E00;
/// RLC firmware version
pub const RLC_GPM_UCODE_DATA: u32 = 0x4E24;
/// RLC safe mode
pub const RLC_SAFE_MODE: u32 = 0x4E0C;

// ═══════════════════════════════════════════════════════════════════════════════
// MC / MMHUB — Memory Controller / Memory Hub
// ═══════════════════════════════════════════════════════════════════════════════

/// Memory controller VM framebuffer location (base, in 1MB units)
pub const MC_VM_FB_LOCATION_BASE: u32 = 0x2024;
/// Memory controller VM framebuffer location (top, in 1MB units)
pub const MC_VM_FB_LOCATION_TOP: u32 = 0x2028;
/// Memory controller VM AGP base
pub const MC_VM_AGP_BASE: u32 = 0x202C;
/// Memory controller VM AGP top
pub const MC_VM_AGP_TOP: u32 = 0x2030;
/// Memory controller VM AGP bot
pub const MC_VM_AGP_BOT: u32 = 0x2034;

/// Legacy memory size register (in MB)
pub const CONFIG_MEMSIZE: u32 = 0x5428;

/// MC ARB RAM configuration (memory type, width, etc.)
pub const MC_ARB_RAMCFG: u32 = 0x9D8;

/// MC SEQ misc — memory type identification
pub const MC_SEQ_MISC0: u32 = 0xA80;

/// MMHUB VM configuration
pub const MMHUB_VM_FB_OFFSET: u32 = 0x31B4;

// ═══════════════════════════════════════════════════════════════════════════════
// SMU / MP1 — System Management Unit
// ═══════════════════════════════════════════════════════════════════════════════

/// SMU firmware version (via MP1 mailbox)
pub const MP1_SMN_C2PMSG_58: u32 = 0x3B8E8;
/// SMU message interface
pub const MP1_SMN_C2PMSG_66: u32 = 0x3B908;
/// SMU response
pub const MP1_SMN_C2PMSG_90: u32 = 0x3B968;
/// SMU argument
pub const MP1_SMN_C2PMSG_82: u32 = 0x3B948;

/// Clock pin control (reference clock info)
pub const CG_CLKPIN_CNTL_2: u32 = 0x0168;

// ═══════════════════════════════════════════════════════════════════════════════
// HDP — Host Data Path
// ═══════════════════════════════════════════════════════════════════════════════

/// HDP Host Path Enable
pub const HDP_HOST_PATH_CNTL: u32 = 0x2C00;
/// HDP nonsurface base
pub const HDP_NONSURFACE_BASE: u32 = 0x2C04;
/// HDP memory coherency flush control
pub const HDP_MEM_COHERENCY_FLUSH_CNTL: u32 = 0x2C14;

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
pub const SDMA0_BASE: u32 = 0x0000_4980;
pub const SDMA1_BASE: u32 = 0x0000_4A80;

/// Stride between SDMA0 and SDMA1 register sets
pub const SDMA_ENGINE_STRIDE: u32 = SDMA1_BASE - SDMA0_BASE;

/// Number of SDMA engines on Navi 10
pub const SDMA_NUM_ENGINES: usize = 2;

// ── SDMA GFX Ring Buffer Registers (offsets from SDMAx_BASE) ────────────────

/// Ring buffer control: size (log2), enable, etc.
pub const SDMA_GFX_RB_CNTL: u32 = 0x00;
/// Ring buffer base address [31:0] (256-byte aligned → bits [31:8] valid)
pub const SDMA_GFX_RB_BASE: u32 = 0x04;
/// Ring buffer base address [63:32]
pub const SDMA_GFX_RB_BASE_HI: u32 = 0x08;
/// Ring buffer read pointer (hardware-managed)
pub const SDMA_GFX_RB_RPTR: u32 = 0x18;
/// Ring buffer read pointer [63:32]
pub const SDMA_GFX_RB_RPTR_HI: u32 = 0x1C;
/// Ring buffer write pointer (software-managed, doorbell-writeable)
pub const SDMA_GFX_RB_WPTR: u32 = 0x20;
/// Ring buffer write pointer [63:32]
pub const SDMA_GFX_RB_WPTR_HI: u32 = 0x24;
/// Write pointer poll address [31:0] — GPU writes RPTR here for CPU to read
pub const SDMA_GFX_RB_WPTR_POLL_ADDR_LO: u32 = 0x28;
/// Write pointer poll address [63:32]
pub const SDMA_GFX_RB_WPTR_POLL_ADDR_HI: u32 = 0x2C;
/// Ring buffer read pointer report address [31:0]
pub const SDMA_GFX_RB_RPTR_ADDR_LO: u32 = 0x30;
/// Ring buffer read pointer report address [63:32]
pub const SDMA_GFX_RB_RPTR_ADDR_HI: u32 = 0x34;
/// Doorbell control for this ring
pub const SDMA_GFX_DOORBELL: u32 = 0x38;
/// Doorbell offset within the BAR4 doorbell page
pub const SDMA_GFX_DOORBELL_OFFSET: u32 = 0x3C;

// ── SDMA Engine-Level Status/Control (absolute offsets from MMIO base) ──────

/// SDMA0 status register — engine idle/busy + error flags
pub const SDMA0_STATUS_REG: u32 = 0x4D68;
/// SDMA0 chicken bits (enable/disable hardware features)
pub const SDMA0_CHICKEN_BITS: u32 = 0x4D6C;
/// SDMA0 clock gating control
pub const SDMA0_CLK_CTRL: u32 = 0x4D70;
/// SDMA0 power cntl
pub const SDMA0_POWER_CNTL: u32 = 0x4D74;
/// SDMA0 freeze — halt engine for programming
pub const SDMA0_F32_CNTL: u32 = 0x4D78;
/// SDMA0 version
pub const SDMA0_VERSION: u32 = 0x4D80;

/// SDMA1 status register
pub const SDMA1_STATUS_REG: u32 = 0x4E68;
/// SDMA1 chicken bits
pub const SDMA1_CHICKEN_BITS: u32 = 0x4E6C;
/// SDMA1 freeze
pub const SDMA1_F32_CNTL: u32 = 0x4E78;

// ── SDMA_GFX_RB_CNTL bit definitions ───────────────────────────────────────

/// Ring buffer size in log2 DWORDs (bits [6:1])
/// E.g., 12 → 4096 DWORDs = 16KB ring
pub const SDMA_RB_CNTL_RB_SIZE_SHIFT: u32 = 1;
pub const SDMA_RB_CNTL_RB_SIZE_MASK: u32 = 0x3F << 1;
/// Ring buffer read pointer writeback enable (bit 12)
pub const SDMA_RB_CNTL_RPTR_WRITEBACK_ENABLE: u32 = 1 << 12;
/// Ring buffer enable (bit 0)
pub const SDMA_RB_CNTL_RB_ENABLE: u32 = 1 << 0;
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
// SDMA packets differ from PM4. Format:
//   DW0: [31:28]=0, [27:26]=sub_op, [25:8]=op, [7:0]=extra_info
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

/// MEC (Micro Engine Compute) engine control
pub const CP_MEC_CNTL: u32 = 0x8234;
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
