//! NVIDIA NV50 (Tesla) Register Definitions
//!
//! Register offsets for G80/G84/G86 family GPUs (BAR0 MMIO space).
//! Source: envytools rnndb, nouveau kernel driver.

// ═══════════════════════════════════════════════════════════════════════════════
// PMC — Master Control (0x000000)
// ═══════════════════════════════════════════════════════════════════════════════

/// GPU boot/identification register
/// Bits 20-27: chipset id, bits 0-7: stepping
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const PMC_BOOT_0: u32 = 0x000000;

/// Endian switch / test register
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const PMC_ENDIAN: u32 = 0x000004;

/// Interrupt status (host)
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const PMC_INTR_HOST: u32 = 0x000100;

/// Interrupt enable (host)
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const PMC_INTR_EN_HOST: u32 = 0x000140;

/// Engine master enable
/// bit 0: all engines, bit 4: PFIFO, bit 8: PTIMER
/// bit 12: PGRAPH, bit 20: PFB, bit 24: PCOPY
/// bit 26: PVDEC, bit 30: PDISPLAY
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const PMC_ENABLE: u32 = 0x000200;

// PMC_ENABLE bits
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const PMC_ENABLE_ALL: u32 = 1 << 0;
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const PMC_ENABLE_PFIFO: u32 = 1 << 8;
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const PMC_ENABLE_PGRAPH: u32 = 1 << 12;
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const PMC_ENABLE_PFB: u32 = 1 << 20;
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const PMC_ENABLE_PDISPLAY: u32 = 1 << 30;

// ═══════════════════════════════════════════════════════════════════════════════
// PBUS — Bus Interface (0x001000)
// ═══════════════════════════════════════════════════════════════════════════════

pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const PBUS_INTR: u32 = 0x001100;
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const PBUS_INTR_EN: u32 = 0x001140;

// ═══════════════════════════════════════════════════════════════════════════════
// PTIMER — Timer (0x009000)
// ═══════════════════════════════════════════════════════════════════════════════

/// Timer low 32 bits (nanoseconds)
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const PTIMER_TIME_0: u32 = 0x009400;
/// Timer high 32 bits
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const PTIMER_TIME_1: u32 = 0x009410;
/// Timer numerator (clock config)
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const PTIMER_NUMERATOR: u32 = 0x009200;
/// Timer denominator
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const PTIMER_DENOMINATOR: u32 = 0x009210;

// ═══════════════════════════════════════════════════════════════════════════════
// PFIFO — Command FIFO (0x002000)
// ═══════════════════════════════════════════════════════════════════════════════

/// FIFO enable
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const PFIFO_ENABLE: u32 = 0x002200;
/// FIFO interrupt status
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const PFIFO_INTR: u32 = 0x002100;
/// FIFO interrupt enable
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const PFIFO_INTR_EN: u32 = 0x002140;

/// FIFO mode (PIO vs DMA per channel)
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const PFIFO_MODE: u32 = 0x002504;
/// FIFO DMA config
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const PFIFO_DMA: u32 = 0x002508;

/// FIFO channel control (per-channel base + 0x40*ch)
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const PFIFO_CHAN_BASE: u32 = 0x800000;
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const PFIFO_CHAN_STRIDE: u32 = 0x2000;

/// FIFO RAMHT (hash table for object handles)
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const PFIFO_RAMHT: u32 = 0x002210;

// ═══════════════════════════════════════════════════════════════════════════════
// PFB — Frame Buffer Controller (0x100000)
// ═══════════════════════════════════════════════════════════════════════════════

/// PFB config 0 — VRAM type, width
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const PFB_CFG0: u32 = 0x100200;
/// PFB config 1 — VRAM size
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const PFB_CFG1: u32 = 0x100204;

// ═══════════════════════════════════════════════════════════════════════════════
// PGRAPH — Graphics Engine (0x400000)
// ═══════════════════════════════════════════════════════════════════════════════

/// PGRAPH interrupt status
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const PGRAPH_INTR: u32 = 0x400100;
/// PGRAPH interrupt enable
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const PGRAPH_INTR_EN: u32 = 0x40013C;
/// PGRAPH channel control
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const PGRAPH_CHANNEL_CONTEXT: u32 = 0x400500;
/// PGRAPH status
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const PGRAPH_STATUS: u32 = 0x400700;

/// PGRAPH trap status (error info)
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const PGRAPH_TRAP: u32 = 0x400108;
/// PGRAPH grctx control
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const PGRAPH_CTXCTL: u32 = 0x400824;
/// PGRAPH DISPATCH (engine enable/class binding)
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const PGRAPH_DISPATCH: u32 = 0x400804;

// ═══════════════════════════════════════════════════════════════════════════════
// PDISPLAY — Display Engine (0x610000)
// ═══════════════════════════════════════════════════════════════════════════════

/// Display interrupt status
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const PDISP_INTR: u32 = 0x610020;
/// Display interrupt enable
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const PDISP_INTR_EN: u32 = 0x610024;

/// Display master control
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const PDISP_MASTER_CONTROLLER: u32 = 0x610200;
/// Display master state
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const PDISP_MASTER_STATE: u32 = 0x610300;

/// CRTC head 0 control base
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const PDISP_HEAD0_BASE: u32 = 0x610B58;
/// CRTC head 0 surface address
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const PDISP_HEAD0_SURFACE: u32 = 0x610B60;
/// CRTC head 0 display size
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const PDISP_HEAD0_SIZE: u32 = 0x610B68;

// ═══════════════════════════════════════════════════════════════════════════════
// NV50 2D Engine Methods (class 0x502D)
// Used via PFIFO command submission (DMA push buffer)
// ═══════════════════════════════════════════════════════════════════════════════

/// Set object/class on subchannel
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const NV50_2D_SET_OBJECT: u32 = 0x0000;

// DMA objects
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const NV50_2D_DMA_NOTIFY: u32 = 0x0180;
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const NV50_2D_DMA_DESTINATION: u32 = 0x0184;
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const NV50_2D_DMA_SOURCE: u32 = 0x0188;

// Destination surface
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const NV50_2D_DESTINATION_FORMAT: u32 = 0x0200;
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const NV50_2D_DESTINATION_LINEAR: u32 = 0x0204;
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const NV50_2D_DESTINATION_PITCH: u32 = 0x0214;
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const NV50_2D_DESTINATION_WIDTH: u32 = 0x0218;
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const NV50_2D_DESTINATION_HEIGHT: u32 = 0x021C;
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const NV50_2D_DESTINATION_ADDRESS_HIGH: u32 = 0x0220;
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const NV50_2D_DESTINATION_ADDRESS_LOW: u32 = 0x0224;

// Source surface
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const NV50_2D_SOURCE_FORMAT: u32 = 0x0230;
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const NV50_2D_SOURCE_LINEAR: u32 = 0x0234;
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const NV50_2D_SOURCE_PITCH: u32 = 0x0244;
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const NV50_2D_SOURCE_WIDTH: u32 = 0x0248;
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const NV50_2D_SOURCE_HEIGHT: u32 = 0x024C;
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const NV50_2D_SOURCE_ADDRESS_HIGH: u32 = 0x0250;
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const NV50_2D_SOURCE_ADDRESS_LOW: u32 = 0x0254;

// Clip rectangle
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const NV50_2D_CLIP_X: u32 = 0x0280;
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const NV50_2D_CLIP_Y: u32 = 0x0284;
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const NV50_2D_CLIP_W: u32 = 0x0288;
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const NV50_2D_CLIP_H: u32 = 0x028C;
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const NV50_2D_CLIP_ENABLE: u32 = 0x0290;

// ROP / blend operations
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const NV50_2D_OPERATION: u32 = 0x02AC;

// Solid drawing
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const NV50_2D_DRAW_SHAPE: u32 = 0x0580;
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const NV50_2D_DRAW_COLOR_FORMAT: u32 = 0x0584;
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const NV50_2D_DRAW_COLOR: u32 = 0x0588;
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const NV50_2D_DRAW_POINT32_X: u32 = 0x0600;
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const NV50_2D_DRAW_POINT32_Y: u32 = 0x0604;

// Blit (copy) operations
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const NV50_2D_BLIT_CONTROL: u32 = 0x088C;
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const NV50_2D_BLIT_DESTINATION_X: u32 = 0x08B0;
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const NV50_2D_BLIT_DESTINATION_Y: u32 = 0x08B4;
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const NV50_2D_BLIT_DESTINATION_W: u32 = 0x08B8;
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const NV50_2D_BLIT_DESTINATION_H: u32 = 0x08BC;
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const NV50_2D_BLIT_DU_DX_FRACT: u32 = 0x08C0;
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const NV50_2D_BLIT_DU_DX_INT: u32 = 0x08C4;
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const NV50_2D_BLIT_DV_DY_FRACT: u32 = 0x08C8;
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const NV50_2D_BLIT_DV_DY_INT: u32 = 0x08CC;
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const NV50_2D_BLIT_SOURCE_X_FRACT: u32 = 0x08D0;
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const NV50_2D_BLIT_SOURCE_X_INT: u32 = 0x08D4;
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const NV50_2D_BLIT_SOURCE_Y_FRACT: u32 = 0x08D8;
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const NV50_2D_BLIT_SOURCE_Y_INT: u32 = 0x08DC;

// SIFC (Scaled Image From CPU)
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const NV50_2D_SIFC_BITMAP_ENABLE: u32 = 0x0800;
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const NV50_2D_SIFC_FORMAT: u32 = 0x0804;
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const NV50_2D_SIFC_WIDTH: u32 = 0x0838;
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const NV50_2D_SIFC_HEIGHT: u32 = 0x083C;
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const NV50_2D_SIFC_DATA: u32 = 0x0860;

// ═══════════════════════════════════════════════════════════════════════════════
// Surface format constants
// ═══════════════════════════════════════════════════════════════════════════════

/// A8R8G8B8 (32-bit ARGB)
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const SURFACE_FORMAT_A8R8G8B8: u32 = 0xCF;
/// X8R8G8B8 (32-bit XRGB, no alpha)
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const SURFACE_FORMAT_X8R8G8B8: u32 = 0xE6;
/// R5G6B5 (16-bit RGB)
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const SURFACE_FORMAT_R5G6B5: u32 = 0xE8;
/// A8 (8-bit alpha only)
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const SURFACE_FORMAT_A8: u32 = 0xF3;

// ═══════════════════════════════════════════════════════════════════════════════
// Operation modes
// ═══════════════════════════════════════════════════════════════════════════════

/// DST = SRC (if alpha then copy)
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const OPERATION_SRCCOPY_AND: u32 = 0;
/// DST = ROP(DST, SRC, PAT) masked by alpha
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const OPERATION_ROP_AND: u32 = 1;
/// DST = blend(DST, SRC) using beta
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const OPERATION_BLEND: u32 = 2;
/// DST = SRC (direct copy)
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const OPERATION_SRCCOPY: u32 = 3;
/// DST = ROP(DST, SRC, PAT)
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const OPERATION_ROP: u32 = 4;

// ═══════════════════════════════════════════════════════════════════════════════
// Draw shapes
// ═══════════════════════════════════════════════════════════════════════════════

pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const DRAW_SHAPE_POINTS: u32 = 0;
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const DRAW_SHAPE_LINES: u32 = 1;
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const DRAW_SHAPE_LINE_STRIP: u32 = 2;
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const DRAW_SHAPE_TRIANGLES: u32 = 3;
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const DRAW_SHAPE_RECTANGLES: u32 = 4;

// ═══════════════════════════════════════════════════════════════════════════════
// NV50 class IDs (for FIFO object binding)
// ═══════════════════════════════════════════════════════════════════════════════

/// 2D engine class
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const NV50_2D_CLASS: u32 = 0x502D;
/// Memory-to-memory format (M2MF)
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const NV50_M2MF_CLASS: u32 = 0x5039;
/// 3D engine (Tesla)
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const NV50_TESLA_CLASS: u32 = 0x5097;
/// Compute engine
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const NV50_COMPUTE_CLASS: u32 = 0x50C0;
/// FIFO channel (GPFIFO)
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const NV50_CHAN_CLASS: u32 = 0x506F;
/// Display class
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const NV50_DISPLAY_CLASS: u32 = 0x5070;

// ═══════════════════════════════════════════════════════════════════════════════
// FIFO push buffer control
// ═══════════════════════════════════════════════════════════════════════════════

/// User-space FIFO register base (per-channel)
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const NV50_USER_BASE: u32 = 0x00C00000;
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const NV50_USER_STRIDE: u32 = 0x2000;

/// DMA PUT offset (write pointer — GPU reads from here)
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const NV50_USER_DMA_PUT: u32 = 0x40;
/// DMA GET offset (read pointer — GPU writes here)
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const NV50_USER_DMA_GET: u32 = 0x44;
/// Channel REF counter (for synchronization)
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const NV50_USER_REF: u32 = 0x48;
