//! DCN 2.0 — Display Core Next for Navi 10 (RDNA 1)
//!
//! Phase 2 of the AMD GPU driver: Display output configuration.
//!
//! DCN 2.0 Pipeline (Navi 10):
//! ```
//! HUBP → DPP → MPC → OPP → OPTC(OTG) → DIO(DP/HDMI) → Monitor
//! ```
//!
//! Components:
//! - **HUBP** (Hub Pipe): Reads surface data from memory (VRAM/framebuffer)
//! - **DPP** (Display Pipe & Plane): Scaling, color conversion, format  
//! - **MPC** (Multi-Pipe Combiner): Blending, cursor overlay
//! - **OPP** (Output Pixel Processing): Dithering, bit-depth reduction
//! - **OPTC/OTG** (Output Timing Generator): CRTC timing, sync signals
//! - **DIO** (Display I/O): DP 1.4 / HDMI 2.0 encoders & PHY
//!
//! Navi 10 has:
//! - 6 display pipes (HUBP0–5, DPP0–5)
//! - 6 OTGs (OTG0–5) 
//! - 6 encoders (DIG0–5) with DP 1.4 + HDMI 2.0
//! - Max 6 simultaneous displays
//!
//! References:
//! - Linux: drivers/gpu/drm/amd/display/dc/dcn20/
//! - AMD Display Core Next docs

use alloc::string::String;
use alloc::format;
use alloc::vec::Vec;
use core::sync::atomic::{AtomicBool, Ordering};
use spin::Mutex;

use super::{mmio_read32, mmio_write32, mmio_read_indirect, GpuInfo};
use super::regs::dcn;

// ═══════════════════════════════════════════════════════════════════════════════
// Display Mode / Timing
// ═══════════════════════════════════════════════════════════════════════════════

/// Display mode timing (CRTC modeline)
#[derive(Debug, Clone, Copy)]
pub struct DisplayMode {
    /// Horizontal active pixels
    pub h_active: u32,
    /// Horizontal front porch
    pub h_front_porch: u32,
    /// Horizontal sync width
    pub h_sync_width: u32,
    /// Horizontal back porch
    pub h_back_porch: u32,
    /// Vertical active lines
    pub v_active: u32,
    /// Vertical front porch
    pub v_front_porch: u32,
    /// Vertical sync width
    pub v_sync_width: u32,
    /// Vertical back porch
    pub v_back_porch: u32,
    /// Pixel clock in kHz
    pub pixel_clock_khz: u32,
    /// Refresh rate in Hz
    pub refresh_hz: u32,
    /// H sync polarity (true = positive)
    pub h_sync_positive: bool,
    /// V sync polarity (true = positive)
    pub v_sync_positive: bool,
}

impl DisplayMode {
    /// H total = active + front + sync + back
    pub fn h_total(&self) -> u32 {
        self.h_active + self.h_front_porch + self.h_sync_width + self.h_back_porch
    }

    /// V total = active + front + sync + back
    pub fn v_total(&self) -> u32 {
        self.v_active + self.v_front_porch + self.v_sync_width + self.v_back_porch
    }

    /// H sync start
    pub fn h_sync_start(&self) -> u32 {
        self.h_active + self.h_front_porch
    }

    /// H sync end
    pub fn h_sync_end(&self) -> u32 {
        self.h_active + self.h_front_porch + self.h_sync_width
    }

    /// V sync start
    pub fn v_sync_start(&self) -> u32 {
        self.v_active + self.v_front_porch
    }

    /// V sync end
    pub fn v_sync_end(&self) -> u32 {
        self.v_active + self.v_front_porch + self.v_sync_width
    }

    /// Format as modeline string
    pub fn modeline(&self) -> String {
        format!("{}x{}@{}Hz pclk={}kHz htotal={} vtotal={}",
            self.h_active, self.v_active, self.refresh_hz,
            self.pixel_clock_khz, self.h_total(), self.v_total())
    }
}

/// Standard display modes (common EDID modes)
pub const MODE_640X480_60: DisplayMode = DisplayMode {
    h_active: 640, h_front_porch: 16, h_sync_width: 96, h_back_porch: 48,
    v_active: 480, v_front_porch: 10, v_sync_width: 2, v_back_porch: 33,
    pixel_clock_khz: 25175, refresh_hz: 60,
    h_sync_positive: false, v_sync_positive: false,
};

pub const MODE_1280X720_60: DisplayMode = DisplayMode {
    h_active: 1280, h_front_porch: 110, h_sync_width: 40, h_back_porch: 220,
    v_active: 720, v_front_porch: 5, v_sync_width: 5, v_back_porch: 20,
    pixel_clock_khz: 74250, refresh_hz: 60,
    h_sync_positive: true, v_sync_positive: true,
};

pub const MODE_1920X1080_60: DisplayMode = DisplayMode {
    h_active: 1920, h_front_porch: 88, h_sync_width: 44, h_back_porch: 148,
    v_active: 1080, v_front_porch: 4, v_sync_width: 5, v_back_porch: 36,
    pixel_clock_khz: 148500, refresh_hz: 60,
    h_sync_positive: true, v_sync_positive: true,
};

pub const MODE_2560X1440_60: DisplayMode = DisplayMode {
    h_active: 2560, h_front_porch: 48, h_sync_width: 32, h_back_porch: 80,
    v_active: 1440, v_front_porch: 3, v_sync_width: 5, v_back_porch: 33,
    pixel_clock_khz: 241500, refresh_hz: 60,
    h_sync_positive: true, v_sync_positive: false,
};

pub const MODE_3840X2160_60: DisplayMode = DisplayMode {
    h_active: 3840, h_front_porch: 176, h_sync_width: 88, h_back_porch: 296,
    v_active: 2160, v_front_porch: 8, v_sync_width: 10, v_back_porch: 72,
    pixel_clock_khz: 533250, refresh_hz: 60,
    h_sync_positive: true, v_sync_positive: false,
};

/// Get all standard modes
pub fn standard_modes() -> &'static [DisplayMode] {
    &[MODE_640X480_60, MODE_1280X720_60, MODE_1920X1080_60, MODE_2560X1440_60, MODE_3840X2160_60]
}

// ═══════════════════════════════════════════════════════════════════════════════
// Connector Types
// ═══════════════════════════════════════════════════════════════════════════════

/// Display connector type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectorType {
    None,
    DisplayPort,
    HDMI,
    DVI,
    VGA,
    Unknown,
}

impl ConnectorType {
    pub fn name(&self) -> &'static str {
        match self {
            ConnectorType::None => "None",
            ConnectorType::DisplayPort => "DisplayPort",
            ConnectorType::HDMI => "HDMI",
            ConnectorType::DVI => "DVI",
            ConnectorType::VGA => "VGA",
            ConnectorType::Unknown => "Unknown",
        }
    }
}

/// Display connector status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectorStatus {
    Disconnected,
    Connected,
    Unknown,
}

/// Display output/connector info
#[derive(Debug, Clone)]
pub struct DisplayConnector {
    /// Connector index (0–5 for Navi 10)
    pub index: u8,
    /// Type of connector
    pub connector_type: ConnectorType,
    /// Connection status
    pub status: ConnectorStatus,
    /// DIG (Display Interface Generator) encoder index
    pub dig_encoder: u8,
    /// PHY index
    pub phy_index: u8,
    /// HPD (Hot Plug Detect) pin
    pub hpd_pin: u8,
    /// Current mode (if active)
    pub current_mode: Option<DisplayMode>,
    /// DPCD revision (for DP connectors) 
    pub dpcd_rev: u8,
    /// Max link rate in 270MHz units (for DP)
    pub max_link_rate: u8,
    /// Max lane count (for DP)
    pub max_lane_count: u8,
}

/// Surface pixel format
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SurfaceFormat {
    /// ARGB 8:8:8:8 (32bpp, standard desktop format)
    Argb8888,
    /// XRGB 8:8:8:8 (32bpp, alpha ignored)
    Xrgb8888,
    /// ABGR 8:8:8:8 (32bpp, reversed channel order)
    Abgr8888,
    /// RGB 5:6:5 (16bpp)
    Rgb565,
}

impl SurfaceFormat {
    /// Bytes per pixel
    pub fn bpp(&self) -> u32 {
        match self {
            SurfaceFormat::Argb8888 | SurfaceFormat::Xrgb8888 | SurfaceFormat::Abgr8888 => 4,
            SurfaceFormat::Rgb565 => 2,
        }
    }

    /// DCN HUBP surface format code
    pub fn dcn_format_code(&self) -> u32 {
        match self {
            SurfaceFormat::Argb8888 => 0x0A, // SURFACE_PIXEL_FORMAT_GRPH_ARGB8888
            SurfaceFormat::Xrgb8888 => 0x08, // SURFACE_PIXEL_FORMAT_GRPH_ARGB8888 (no alpha)
            SurfaceFormat::Abgr8888 => 0x0C, // SURFACE_PIXEL_FORMAT_GRPH_ABGR8888
            SurfaceFormat::Rgb565 => 0x04,   // SURFACE_PIXEL_FORMAT_GRPH_RGB565
        }
    }
}

/// Scanout surface configuration
#[derive(Debug, Clone)]
pub struct ScanoutSurface {
    /// Physical address of the framebuffer
    pub fb_phys_addr: u64,
    /// Width in pixels
    pub width: u32,
    /// Height in pixels
    pub height: u32,
    /// Pitch (bytes per row, may be > width * bpp due to alignment)
    pub pitch: u32,
    /// Pixel format
    pub format: SurfaceFormat,
}

// ═══════════════════════════════════════════════════════════════════════════════
// DCN State  
// ═══════════════════════════════════════════════════════════════════════════════

/// DCN display engine state
pub struct DcnState {
    /// Whether DCN has been initialized
    pub initialized: bool,
    /// Detected connectors
    pub connectors: Vec<DisplayConnector>,
    /// Active display count
    pub active_displays: u8,
    /// DCN version detected
    pub dcn_version: (u8, u8),  // (major, minor)
    /// Max number of pipes
    pub max_pipes: u8,
    /// Current scanout configuration per pipe
    pub scanouts: [Option<ScanoutSurface>; 6],
}

static DCN_STATE: Mutex<DcnState> = Mutex::new(DcnState {
    initialized: false,
    connectors: Vec::new(),
    active_displays: 0,
    dcn_version: (0, 0),
    max_pipes: 0,
    scanouts: [None, None, None, None, None, None],
});

static DCN_READY: AtomicBool = AtomicBool::new(false);

// ═══════════════════════════════════════════════════════════════════════════════
// DCN Register Access Helpers
// ═══════════════════════════════════════════════════════════════════════════════

/// Read OTG register for a specific pipe
unsafe fn otg_read(mmio: u64, pipe: u8, reg_offset: u32) -> u32 {
    let base = dcn::OTG0_BASE + (pipe as u32) * dcn::OTG_PIPE_STRIDE;
    mmio_read32(mmio, base + reg_offset)
}

/// Write OTG register for a specific pipe
unsafe fn otg_write(mmio: u64, pipe: u8, reg_offset: u32, value: u32) {
    let base = dcn::OTG0_BASE + (pipe as u32) * dcn::OTG_PIPE_STRIDE;
    mmio_write32(mmio, base + reg_offset, value);
}

/// Read HUBP register for a specific pipe
unsafe fn hubp_read(mmio: u64, pipe: u8, reg_offset: u32) -> u32 {
    let base = dcn::HUBP0_BASE + (pipe as u32) * dcn::HUBP_PIPE_STRIDE;
    mmio_read32(mmio, base + reg_offset)
}

/// Write HUBP register for a specific pipe
unsafe fn hubp_write(mmio: u64, pipe: u8, reg_offset: u32, value: u32) {
    let base = dcn::HUBP0_BASE + (pipe as u32) * dcn::HUBP_PIPE_STRIDE;
    mmio_write32(mmio, base + reg_offset, value);
}

/// Read DIG encoder register
#[allow(dead_code)]
unsafe fn dig_read(mmio: u64, dig: u8, reg_offset: u32) -> u32 {
    let base = dcn::DIG0_BASE + (dig as u32) * dcn::DIG_STRIDE;
    mmio_read32(mmio, base + reg_offset)
}

/// Read HPD register
unsafe fn hpd_read(mmio: u64, hpd: u8, reg_offset: u32) -> u32 {
    let base = dcn::HPD0_BASE + (hpd as u32) * dcn::HPD_STRIDE;
    mmio_read32(mmio, base + reg_offset)
}

// ═══════════════════════════════════════════════════════════════════════════════
// Connector Detection  
// ═══════════════════════════════════════════════════════════════════════════════

/// Detect display connectors by reading HPD (Hot Plug Detect) status
/// and DIG encoder configuration registers
pub fn detect_connectors(mmio: u64) -> Vec<DisplayConnector> {
    let mut connectors = Vec::new();
    
    crate::serial_println!("[DCN] Scanning display connectors (6 HPD pins)...");
    
    for i in 0..6u8 {
        unsafe {
            // Read HPD status register
            let hpd_int_status = hpd_read(mmio, i, dcn::HPD_INT_STATUS_OFFSET);
            let hpd_int_control = hpd_read(mmio, i, dcn::HPD_INT_CONTROL_OFFSET);
            
            crate::serial_println!("[DCN]   HPD{}: INT_STATUS={:#010X} INT_CONTROL={:#010X}", 
                i, hpd_int_status, hpd_int_control);
            
            // Bit 0 of HPD_INT_STATUS = HPD sense (1 = connected)
            let connected = (hpd_int_status & 1) != 0;
            
            // Read DIG encoder type to determine connector type
            let dig_status = dig_read(mmio, i, dcn::DIG_FE_CNTL_OFFSET);
            
            crate::serial_println!("[DCN]   DIG{}: FE_CNTL={:#010X} connected={}", 
                i, dig_status, connected);
            
            // Determine connector type from DIG configuration
            // Bits 19:16 of DIG_FE_CNTL encode the transmitter type
            let tx_type = (dig_status >> 16) & 0xF;
            let connector_type = match tx_type {
                0 => ConnectorType::DisplayPort,
                1 => ConnectorType::HDMI,
                2 => ConnectorType::DVI,
                _ => {
                    // If we can't read DIG, try HPD control register for hints
                    if hpd_int_control != 0 && hpd_int_control != 0xFFFFFFFF {
                        ConnectorType::DisplayPort // Most Navi 10 outputs are DP
                    } else {
                        ConnectorType::Unknown
                    }
                }
            };
            
            let status = if connected {
                ConnectorStatus::Connected
            } else if hpd_int_status == 0xFFFFFFFF {
                ConnectorStatus::Unknown // Register inaccessible
            } else {
                ConnectorStatus::Disconnected
            };
            
            connectors.push(DisplayConnector {
                index: i,
                connector_type,
                status,
                dig_encoder: i,
                phy_index: i,
                hpd_pin: i,
                current_mode: None,
                dpcd_rev: 0,
                max_link_rate: 0,
                max_lane_count: 0,
            });
        }
    }
    
    connectors
}

/// Read DPCD (DisplayPort Configuration Data) via AUX channel
/// This identifies DP monitors and their capabilities
pub fn read_dpcd(mmio: u64, connector: &mut DisplayConnector) {
    if connector.connector_type != ConnectorType::DisplayPort {
        return;
    }
    
    unsafe {
        // AUX channel registers for this DIG encoder
        let dig = connector.dig_encoder;
        let aux_base = dcn::AUX0_BASE + (dig as u32) * dcn::AUX_STRIDE;
        
        // Read AUX control to check if the AUX engine is ready
        let aux_control = mmio_read32(mmio, aux_base + dcn::AUX_CONTROL_OFFSET);
        crate::serial_println!("[DCN]   AUX{}: CONTROL={:#010X}", dig, aux_control);
        
        // In a full implementation, we would:
        // 1. Set up AUX transaction (address = 0x0000 for DPCD rev)
        // 2. Start AUX read
        // 3. Wait for completion
        // 4. Read back DPCD data
        // For now, we just log what's visible
        
        // Try to read DPCD mailbox if firmware has cached it
        let dpcd_data = mmio_read32(mmio, aux_base + dcn::AUX_DPHY_TX_REF_CONTROL_OFFSET);
        crate::serial_println!("[DCN]   AUX{}: DPHY_TX_REF={:#010X}", dig, dpcd_data);
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// OTG (Output Timing Generator) Programming
// ═══════════════════════════════════════════════════════════════════════════════

/// Read the current mode from an OTG pipe
pub fn read_current_mode(mmio: u64, pipe: u8) -> Option<DisplayMode> {
    unsafe {
        // Read OTG status to check if it's enabled
        let otg_control = otg_read(mmio, pipe, dcn::OTG_CONTROL_OFFSET);
        crate::serial_println!("[DCN] OTG{}: CONTROL={:#010X}", pipe, otg_control);
        
        // Bit 0 = OTG enabled
        if otg_control & 1 == 0 {
            return None;
        }
        
        // Read timing registers
        let h_total = otg_read(mmio, pipe, dcn::OTG_H_TOTAL_OFFSET);
        let h_blank_start_end = otg_read(mmio, pipe, dcn::OTG_H_BLANK_START_END_OFFSET);
        let h_sync = otg_read(mmio, pipe, dcn::OTG_H_SYNC_A_OFFSET);
        let v_total = otg_read(mmio, pipe, dcn::OTG_V_TOTAL_OFFSET);
        let v_blank_start_end = otg_read(mmio, pipe, dcn::OTG_V_BLANK_START_END_OFFSET);
        let v_sync = otg_read(mmio, pipe, dcn::OTG_V_SYNC_A_OFFSET);
        
        crate::serial_println!("[DCN] OTG{}: H_TOTAL={:#010X} H_BLANK={:#010X} H_SYNC={:#010X}", 
            pipe, h_total, h_blank_start_end, h_sync);
        crate::serial_println!("[DCN] OTG{}: V_TOTAL={:#010X} V_BLANK={:#010X} V_SYNC={:#010X}", 
            pipe, v_total, v_blank_start_end, v_sync);
        
        // Parse timing values
        let ht = h_total & 0x7FFF;
        let vt = v_total & 0x7FFF;
        let h_blank_start = (h_blank_start_end >> 16) & 0x7FFF;
        let h_blank_end = h_blank_start_end & 0x7FFF;
        let v_blank_start = (v_blank_start_end >> 16) & 0x7FFF;
        let v_blank_end = v_blank_start_end & 0x7FFF;
        let h_sync_start = (h_sync >> 16) & 0x7FFF;
        let h_sync_end = h_sync & 0x7FFF;
        let v_sync_start = (v_sync >> 16) & 0x7FFF;
        let v_sync_end = v_sync & 0x7FFF;
        
        // Validate timing values
        if ht == 0 || vt == 0 || ht > 8192 || vt > 8192 {
            return None;
        }
        
        let h_active = if h_blank_end <= ht { h_blank_end } else { ht };
        let v_active = if v_blank_end <= vt { v_blank_end } else { vt };
        
        if h_active == 0 || v_active == 0 {
            return None;
        }
        
        let h_fp = h_sync_start.saturating_sub(h_active);
        let h_sw = h_sync_end.saturating_sub(h_sync_start);
        let h_bp = ht.saturating_sub(h_sync_end);
        
        let v_fp = v_sync_start.saturating_sub(v_active);
        let v_sw = v_sync_end.saturating_sub(v_sync_start);
        let v_bp = vt.saturating_sub(v_sync_end);
        
        // Estimate pixel clock from OTG clock divider (if readable)
        let clock_status = otg_read(mmio, pipe, dcn::OTG_PIXEL_RATE_CNTL_OFFSET);
        let pixel_clock_khz = if clock_status != 0 && clock_status != 0xFFFFFFFF {
            // Try to extract from register
            (clock_status & 0xFFFF) * 10 // Rough estimate
        } else {
            // Calculate from standard timing
            ((ht as u64) * (vt as u64) * 60 / 1000) as u32
        };
        
        let refresh = if ht > 0 && vt > 0 && pixel_clock_khz > 0 {
            (pixel_clock_khz as u64 * 1000) / (ht as u64 * vt as u64)
        } else {
            60 // Default
        };
        
        Some(DisplayMode {
            h_active, h_front_porch: h_fp, h_sync_width: h_sw, h_back_porch: h_bp,
            v_active, v_front_porch: v_fp, v_sync_width: v_sw, v_back_porch: v_bp,
            pixel_clock_khz,
            refresh_hz: refresh as u32,
            h_sync_positive: true,
            v_sync_positive: true,
        })
    }
}

/// Program OTG timing for a display mode
pub fn program_otg_timing(mmio: u64, pipe: u8, mode: &DisplayMode) {
    crate::serial_println!("[DCN] Programming OTG{} for {}", pipe, mode.modeline());
    
    unsafe {
        // Disable OTG first
        otg_write(mmio, pipe, dcn::OTG_CONTROL_OFFSET, 0);
        
        // Program H timing
        otg_write(mmio, pipe, dcn::OTG_H_TOTAL_OFFSET, mode.h_total() - 1);
        
        let h_blank = ((mode.h_sync_start()) << 16) | mode.h_active;
        otg_write(mmio, pipe, dcn::OTG_H_BLANK_START_END_OFFSET, h_blank);
        
        let h_sync = ((mode.h_sync_start()) << 16) | mode.h_sync_end();
        otg_write(mmio, pipe, dcn::OTG_H_SYNC_A_OFFSET, h_sync);
        
        // Program V timing
        otg_write(mmio, pipe, dcn::OTG_V_TOTAL_OFFSET, mode.v_total() - 1);
        
        let v_blank = ((mode.v_sync_start()) << 16) | mode.v_active;
        otg_write(mmio, pipe, dcn::OTG_V_BLANK_START_END_OFFSET, v_blank);
        
        let v_sync = ((mode.v_sync_start()) << 16) | mode.v_sync_end();
        otg_write(mmio, pipe, dcn::OTG_V_SYNC_A_OFFSET, v_sync);
        
        crate::serial_println!("[DCN] OTG{} timing programmed: {}x{} htotal={} vtotal={}",
            pipe, mode.h_active, mode.v_active, mode.h_total(), mode.v_total());
    }
}

/// Enable OTG output
pub fn enable_otg(mmio: u64, pipe: u8) {
    unsafe {
        // Set OTG_CONTROL bit 0 = enable
        let mut ctl = otg_read(mmio, pipe, dcn::OTG_CONTROL_OFFSET);
        ctl |= 1; // Enable
        otg_write(mmio, pipe, dcn::OTG_CONTROL_OFFSET, ctl);
        crate::serial_println!("[DCN] OTG{} enabled", pipe);
    }
}

/// Disable OTG output
pub fn disable_otg(mmio: u64, pipe: u8) {
    unsafe {
        let mut ctl = otg_read(mmio, pipe, dcn::OTG_CONTROL_OFFSET);
        ctl &= !1; // Disable
        otg_write(mmio, pipe, dcn::OTG_CONTROL_OFFSET, ctl);
        crate::serial_println!("[DCN] OTG{} disabled", pipe);
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// HUBP — Surface Configuration
// ═══════════════════════════════════════════════════════════════════════════════

/// Configure HUBP pipe for a scanout surface
pub fn configure_hubp(mmio: u64, pipe: u8, surface: &ScanoutSurface) {
    crate::serial_println!("[DCN] Configuring HUBP{} for {}x{} @ {:#X}", 
        pipe, surface.width, surface.height, surface.fb_phys_addr);
    
    unsafe {
        // Set surface address (split into high/low 32-bit)
        let addr_hi = (surface.fb_phys_addr >> 32) as u32;
        let addr_lo = (surface.fb_phys_addr & 0xFFFFFFFF) as u32;
        
        hubp_write(mmio, pipe, dcn::HUBP_SURFACE_ADDR_HIGH_OFFSET, addr_hi);
        hubp_write(mmio, pipe, dcn::HUBP_SURFACE_ADDR_OFFSET, addr_lo);
        
        // Set surface pitch (in pixels, or bytes on some revisions)
        hubp_write(mmio, pipe, dcn::HUBP_SURFACE_PITCH_OFFSET, surface.pitch / surface.format.bpp());
        
        // Set surface size
        let size_reg = (surface.height << 16) | surface.width;
        hubp_write(mmio, pipe, dcn::HUBP_SURFACE_SIZE_OFFSET, size_reg);
        
        // Set pixel format
        hubp_write(mmio, pipe, dcn::HUBP_SURFACE_CONFIG_OFFSET, surface.format.dcn_format_code());
        
        crate::serial_println!("[DCN] HUBP{} configured: addr={:#010X}:{:#010X} pitch={} fmt={:?}",
            pipe, addr_hi, addr_lo, surface.pitch, surface.format);
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// DCN Initialization
// ═══════════════════════════════════════════════════════════════════════════════

/// Initialize the DCN display engine
/// Called after Phase 1 has mapped MMIO
pub fn init(mmio_base: u64) {
    crate::log!("[DCN] ═══════════════════════════════════════════════════════");
    crate::log!("[DCN] Display Core Next 2.0 — Phase 2: Display Configuration");
    crate::log!("[DCN] ═══════════════════════════════════════════════════════");
    
    // Step 1: Read DCN version
    let dcn_ver = unsafe { mmio_read32(mmio_base, dcn::DCN_VERSION) };
    crate::serial_println!("[DCN] DCN_VERSION raw: {:#010X}", dcn_ver);
    
    let dcn_major = (dcn_ver >> 8) & 0xFF;
    let dcn_minor = dcn_ver & 0xFF;
    crate::log!("[DCN] DCN version: {}.{}", dcn_major, dcn_minor);
    
    // Step 2: Read DMCUB (Display Micro-Controller Unit) status
    let dmcub_status = unsafe { mmio_read32(mmio_base, dcn::DMCUB_STATUS) };
    crate::serial_println!("[DCN] DMCUB_STATUS: {:#010X}", dmcub_status);
    
    // Step 3: Detect connectors
    crate::log!("[DCN] Detecting display connectors...");
    let mut connectors = detect_connectors(mmio_base);
    
    let mut connected_count = 0u8;
    for conn in &connectors {
        let status_str = match conn.status {
            ConnectorStatus::Connected => {
                connected_count += 1;
                "CONNECTED"
            },
            ConnectorStatus::Disconnected => "disconnected",
            ConnectorStatus::Unknown => "unknown",
        };
        crate::log!("[DCN]   Connector {}: {} — {}", 
            conn.index, conn.connector_type.name(), status_str);
    }
    crate::log!("[DCN] Found {} connected display(s)", connected_count);
    
    // Step 4: Read DPCD for DP connectors
    for conn in &mut connectors {
        if conn.status == ConnectorStatus::Connected {
            read_dpcd(mmio_base, conn);
        }
    }
    
    // Step 5: Read active OTGs to discover current display modes
    crate::log!("[DCN] Reading active display modes...");
    let mut active_displays = 0u8;
    
    for pipe in 0..6u8 {
        if let Some(mode) = read_current_mode(mmio_base, pipe) {
            crate::log!("[DCN]   OTG{}: {} (active)", pipe, mode.modeline());
            // Associate mode with corresponding connector
            if (pipe as usize) < connectors.len() {
                connectors[pipe as usize].current_mode = Some(mode);
            }
            active_displays += 1;
        } else {
            crate::serial_println!("[DCN]   OTG{}: inactive", pipe);
        }
    }
    
    // Step 6: Read HUBP surface info
    crate::log!("[DCN] Reading HUBP surface configurations...");
    let mut scanouts: [Option<ScanoutSurface>; 6] = [None, None, None, None, None, None];
    
    for pipe in 0..6u8 {
        unsafe {
            let surf_addr_hi = hubp_read(mmio_base, pipe, dcn::HUBP_SURFACE_ADDR_HIGH_OFFSET);
            let surf_addr_lo = hubp_read(mmio_base, pipe, dcn::HUBP_SURFACE_ADDR_OFFSET);
            let surf_config = hubp_read(mmio_base, pipe, dcn::HUBP_SURFACE_CONFIG_OFFSET);
            let surf_pitch = hubp_read(mmio_base, pipe, dcn::HUBP_SURFACE_PITCH_OFFSET);
            let surf_size = hubp_read(mmio_base, pipe, dcn::HUBP_SURFACE_SIZE_OFFSET);
            
            let addr = ((surf_addr_hi as u64) << 32) | (surf_addr_lo as u64);
            
            if addr != 0 && addr != 0xFFFFFFFFFFFFFFFF && surf_config != 0xFFFFFFFF {
                let width = surf_size & 0xFFFF;
                let height = (surf_size >> 16) & 0xFFFF;
                
                crate::serial_println!("[DCN]   HUBP{}: addr={:#014X} size={}x{} pitch={} config={:#010X}", 
                    pipe, addr, width, height, surf_pitch, surf_config);
                
                if width > 0 && height > 0 && width < 16384 && height < 16384 {
                    scanouts[pipe as usize] = Some(ScanoutSurface {
                        fb_phys_addr: addr,
                        width,
                        height,
                        pitch: surf_pitch * 4, // Assume 32bpp
                        format: SurfaceFormat::Xrgb8888,
                    });
                    crate::log!("[DCN]   HUBP{}: {}x{} surface at {:#014X}", pipe, width, height, addr);
                }
            }
        }
    }
    
    // Summary
    crate::log!("[DCN] ───────────────────────────────────────────────────────");
    crate::log!("[DCN] DCN {}.{} — {} connector(s), {} active display(s)",
        dcn_major, dcn_minor, connected_count, active_displays);
    for conn in &connectors {
        if conn.status == ConnectorStatus::Connected {
            if let Some(ref mode) = conn.current_mode {
                crate::log!("[DCN]   Output {}: {} {}x{}@{}Hz", 
                    conn.index, conn.connector_type.name(),
                    mode.h_active, mode.v_active, mode.refresh_hz);
            } else {
                crate::log!("[DCN]   Output {}: {} (connected, no active mode)", 
                    conn.index, conn.connector_type.name());
            }
        }
    }
    crate::log!("[DCN] ───────────────────────────────────────────────────────");
    crate::log!("[DCN] Phase 2 complete — Display engine probed");
    
    // Store state
    let mut state = DCN_STATE.lock();
    state.initialized = true;
    state.connectors = connectors;
    state.active_displays = active_displays;
    state.dcn_version = (dcn_major as u8, dcn_minor as u8);
    state.max_pipes = 6;
    state.scanouts = scanouts;
    DCN_READY.store(true, Ordering::SeqCst);
}

// ═══════════════════════════════════════════════════════════════════════════════
// Public API
// ═══════════════════════════════════════════════════════════════════════════════

/// Check if DCN is initialized
pub fn is_ready() -> bool {
    DCN_READY.load(Ordering::Relaxed)
}

/// Get detected connectors
pub fn get_connectors() -> Vec<DisplayConnector> {
    DCN_STATE.lock().connectors.clone()
}

/// Get active display count
pub fn active_display_count() -> u8 {
    DCN_STATE.lock().active_displays
}

/// Get DCN version
pub fn dcn_version() -> (u8, u8) {
    DCN_STATE.lock().dcn_version
}

/// Get summary string
pub fn summary() -> String {
    let state = DCN_STATE.lock();
    if state.initialized {
        let connected = state.connectors.iter()
            .filter(|c| c.status == ConnectorStatus::Connected)
            .count();
        format!("DCN {}.{} — {} display(s), {} connected", 
            state.dcn_version.0, state.dcn_version.1,
            state.active_displays, connected)
    } else {
        String::from("DCN not initialized")
    }
}

/// Get detailed info lines for terminal display
pub fn info_lines() -> Vec<String> {
    let mut lines = Vec::new();
    let state = DCN_STATE.lock();
    
    if state.initialized {
        lines.push(format!("DCN {}.{} Display Engine", state.dcn_version.0, state.dcn_version.1));
        lines.push(format!("  Pipes: {} max, {} active", state.max_pipes, state.active_displays));
        lines.push(String::new());
        
        for conn in &state.connectors {
            let status = match conn.status {
                ConnectorStatus::Connected => "CONNECTED",
                ConnectorStatus::Disconnected => "disconnected",
                ConnectorStatus::Unknown => "unknown",
            };
            
            let mut line = format!("  Connector {}: {} [{}]", 
                conn.index, conn.connector_type.name(), status);
            
            if let Some(ref mode) = conn.current_mode {
                line.push_str(&format!(" — {}x{}@{}Hz", mode.h_active, mode.v_active, mode.refresh_hz));
            }
            
            lines.push(line);
        }
        
        // Show scanout surfaces
        lines.push(String::new());
        lines.push(String::from("  Active Surfaces:"));
        for (i, scanout) in state.scanouts.iter().enumerate() {
            if let Some(ref s) = scanout {
                lines.push(format!("    HUBP{}: {}x{} {:?} @ {:#X}", 
                    i, s.width, s.height, s.format, s.fb_phys_addr));
            }
        }
    } else {
        lines.push(String::from("DCN display engine not initialized"));
    }
    
    lines
}
