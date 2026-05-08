


























use alloc::string::String;
use alloc::format;
use alloc::vec::Vec;
use core::sync::atomic::{AtomicBool, Ordering};
use spin::Mutex;

use super::{kj, ib, inz, Hz};
use super::regs::dcn;






#[derive(Debug, Clone, Copy)]
pub struct Cv {
    
    pub h_active: u32,
    
    pub h_front_porch: u32,
    
    pub h_sync_width: u32,
    
    pub h_back_porch: u32,
    
    pub v_active: u32,
    
    pub v_front_porch: u32,
    
    pub v_sync_width: u32,
    
    pub v_back_porch: u32,
    
    pub pixel_clock_khz: u32,
    
    pub refresh_hz: u32,
    
    pub h_sync_positive: bool,
    
    pub v_sync_positive: bool,
}

impl Cv {
    
    pub fn h_total(&self) -> u32 {
        self.h_active + self.h_front_porch + self.h_sync_width + self.h_back_porch
    }

    
    pub fn v_total(&self) -> u32 {
        self.v_active + self.v_front_porch + self.v_sync_width + self.v_back_porch
    }

    
    pub fn h_sync_start(&self) -> u32 {
        self.h_active + self.h_front_porch
    }

    
    pub fn h_sync_end(&self) -> u32 {
        self.h_active + self.h_front_porch + self.h_sync_width
    }

    
    pub fn v_sync_start(&self) -> u32 {
        self.v_active + self.v_front_porch
    }

    
    pub fn v_sync_end(&self) -> u32 {
        self.v_active + self.v_front_porch + self.v_sync_width
    }

    
    pub fn modeline(&self) -> String {
        format!("{}x{}@{}Hz pclk={}kHz htotal={} vtotal={}",
            self.h_active, self.v_active, self.refresh_hz,
            self.pixel_clock_khz, self.h_total(), self.v_total())
    }
}


pub const CKI_: Cv = Cv {
    h_active: 640, h_front_porch: 16, h_sync_width: 96, h_back_porch: 48,
    v_active: 480, v_front_porch: 10, v_sync_width: 2, v_back_porch: 33,
    pixel_clock_khz: 25175, refresh_hz: 60,
    h_sync_positive: false, v_sync_positive: false,
};

pub const CKE_: Cv = Cv {
    h_active: 1280, h_front_porch: 110, h_sync_width: 40, h_back_porch: 220,
    v_active: 720, v_front_porch: 5, v_sync_width: 5, v_back_porch: 20,
    pixel_clock_khz: 74250, refresh_hz: 60,
    h_sync_positive: true, v_sync_positive: true,
};

pub const CKF_: Cv = Cv {
    h_active: 1920, h_front_porch: 88, h_sync_width: 44, h_back_porch: 148,
    v_active: 1080, v_front_porch: 4, v_sync_width: 5, v_back_porch: 36,
    pixel_clock_khz: 148500, refresh_hz: 60,
    h_sync_positive: true, v_sync_positive: true,
};

pub const CKG_: Cv = Cv {
    h_active: 2560, h_front_porch: 48, h_sync_width: 32, h_back_porch: 80,
    v_active: 1440, v_front_porch: 3, v_sync_width: 5, v_back_porch: 33,
    pixel_clock_khz: 241500, refresh_hz: 60,
    h_sync_positive: true, v_sync_positive: false,
};

pub const CKH_: Cv = Cv {
    h_active: 3840, h_front_porch: 176, h_sync_width: 88, h_back_porch: 296,
    v_active: 2160, v_front_porch: 8, v_sync_width: 10, v_back_porch: 72,
    pixel_clock_khz: 533250, refresh_hz: 60,
    h_sync_positive: true, v_sync_positive: false,
};


pub fn ovz() -> &'static [Cv] {
    &[CKI_, CKE_, CKF_, CKG_, CKH_]
}






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


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectorStatus {
    Disconnected,
    Connected,
    Unknown,
}


#[derive(Debug, Clone)]
pub struct Lm {
    
    pub index: u8,
    
    pub connector_type: ConnectorType,
    
    pub status: ConnectorStatus,
    
    pub dig_encoder: u8,
    
    pub phy_index: u8,
    
    pub hpd_pin: u8,
    
    pub current_mode: Option<Cv>,
    
    pub dpcd_rev: u8,
    
    pub max_link_rate: u8,
    
    pub max_lane_count: u8,
}


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SurfaceFormat {
    
    Argb8888,
    
    Xrgb8888,
    
    Abgr8888,
    
    Rgb565,
}

impl SurfaceFormat {
    
    pub fn bpp(&self) -> u32 {
        match self {
            SurfaceFormat::Argb8888 | SurfaceFormat::Xrgb8888 | SurfaceFormat::Abgr8888 => 4,
            SurfaceFormat::Rgb565 => 2,
        }
    }

    
    pub fn dcn_format_code(&self) -> u32 {
        match self {
            SurfaceFormat::Argb8888 => 0x0A, 
            SurfaceFormat::Xrgb8888 => 0x08, 
            SurfaceFormat::Abgr8888 => 0x0C, 
            SurfaceFormat::Rgb565 => 0x04,   
        }
    }
}


#[derive(Debug, Clone)]
pub struct Qf {
    
    pub fb_phys_addr: u64,
    
    pub width: u32,
    
    pub height: u32,
    
    pub pitch: u32,
    
    pub format: SurfaceFormat,
}






pub struct Xq {
    
    pub initialized: bool,
    
    pub connectors: Vec<Lm>,
    
    pub active_displays: u8,
    
    pub dcn_version: (u8, u8),  
    
    pub max_pipes: u8,
    
    pub scanouts: [Option<Qf>; 6],
}

static KP_: Mutex<Xq> = Mutex::new(Xq {
    initialized: false,
    connectors: Vec::new(),
    active_displays: 0,
    dcn_version: (0, 0),
    max_pipes: 0,
    scanouts: [None, None, None, None, None, None],
});

static ARW_: AtomicBool = AtomicBool::new(false);






unsafe fn boa(mmio: u64, pipe: u8, abg: u32) -> u32 {
    let base = dcn::BEA_ + (pipe as u32) * dcn::BEE_;
    kj(mmio, base + abg)
}


unsafe fn buo(mmio: u64, pipe: u8, abg: u32, value: u32) {
    let base = dcn::BEA_ + (pipe as u32) * dcn::BEE_;
    ib(mmio, base + abg, value);
}


unsafe fn drn(mmio: u64, pipe: u8, abg: u32) -> u32 {
    let base = dcn::AYF_ + (pipe as u32) * dcn::AYG_;
    kj(mmio, base + abg)
}


unsafe fn dro(mmio: u64, pipe: u8, abg: u32, value: u32) {
    let base = dcn::AYF_ + (pipe as u32) * dcn::AYG_;
    ib(mmio, base + abg, value);
}


#[allow(dead_code)]
unsafe fn lep(mmio: u64, dnc: u8, abg: u32) -> u32 {
    let base = dcn::BUO_ + (dnc as u32) * dcn::BUQ_;
    kj(mmio, base + abg)
}


unsafe fn ife(mmio: u64, czg: u8, abg: u32) -> u32 {
    let base = dcn::CDV_ + (czg as u32) * dcn::CDY_;
    kj(mmio, base + abg)
}







pub fn ldu(mmio: u64) -> Vec<Lm> {
    let mut connectors = Vec::new();
    
    crate::serial_println!("[DCN] Scanning display connectors (6 HPD pins)...");
    
    for i in 0..6u8 {
        unsafe {
            
            let gbd = ife(mmio, i, dcn::CDX_);
            let gbc = ife(mmio, i, dcn::CDW_);
            
            crate::serial_println!("[DCN]   HPD{}: INT_STATUS={:#010X} INT_CONTROL={:#010X}", 
                i, gbd, gbc);
            
            
            let bfn = (gbd & 1) != 0;
            
            
            let hse = lep(mmio, i, dcn::BUP_);
            
            crate::serial_println!("[DCN]   DIG{}: FE_CNTL={:#010X} connected={}", 
                i, hse, bfn);
            
            
            
            let poy = (hse >> 16) & 0xF;
            let connector_type = match poy {
                0 => ConnectorType::DisplayPort,
                1 => ConnectorType::HDMI,
                2 => ConnectorType::DVI,
                _ => {
                    
                    if gbc != 0 && gbc != 0xFFFFFFFF {
                        ConnectorType::DisplayPort 
                    } else {
                        ConnectorType::Unknown
                    }
                }
            };
            
            let status = if bfn {
                ConnectorStatus::Connected
            } else if gbd == 0xFFFFFFFF {
                ConnectorStatus::Unknown 
            } else {
                ConnectorStatus::Disconnected
            };
            
            connectors.push(Lm {
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



pub fn ocp(mmio: u64, connector: &mut Lm) {
    if connector.connector_type != ConnectorType::DisplayPort {
        return;
    }
    
    unsafe {
        
        let dnc = connector.dig_encoder;
        let hge = dcn::BNH_ + (dnc as u32) * dcn::BNK_;
        
        
        let jyn = kj(mmio, hge + dcn::BNI_);
        crate::serial_println!("[DCN]   AUX{}: CONTROL={:#010X}", dnc, jyn);
        
        
        
        
        
        
        
        
        
        let lhi = kj(mmio, hge + dcn::BNJ_);
        crate::serial_println!("[DCN]   AUX{}: DPHY_TX_REF={:#010X}", dnc, lhi);
    }
}






pub fn ocn(mmio: u64, pipe: u8) -> Option<Cv> {
    unsafe {
        
        let isv = boa(mmio, pipe, dcn::LN_);
        crate::serial_println!("[DCN] OTG{}: CONTROL={:#010X}", pipe, isv);
        
        
        if isv & 1 == 0 {
            return None;
        }
        
        
        let h_total = boa(mmio, pipe, dcn::BED_);
        let fzr = boa(mmio, pipe, dcn::BEB_);
        let drc = boa(mmio, pipe, dcn::BEC_);
        let v_total = boa(mmio, pipe, dcn::BEH_);
        let hbb = boa(mmio, pipe, dcn::BEF_);
        let edm = boa(mmio, pipe, dcn::BEG_);
        
        crate::serial_println!("[DCN] OTG{}: H_TOTAL={:#010X} H_BLANK={:#010X} H_SYNC={:#010X}", 
            pipe, h_total, fzr, drc);
        crate::serial_println!("[DCN] OTG{}: V_TOTAL={:#010X} V_BLANK={:#010X} V_SYNC={:#010X}", 
            pipe, v_total, hbb, edm);
        
        
        let cal = h_total & 0x7FFF;
        let cfa = v_total & 0x7FFF;
        let qkb = (fzr >> 16) & 0x7FFF;
        let ide = fzr & 0x7FFF;
        let rbu = (hbb >> 16) & 0x7FFF;
        let jpr = hbb & 0x7FFF;
        let h_sync_start = (drc >> 16) & 0x7FFF;
        let h_sync_end = drc & 0x7FFF;
        let v_sync_start = (edm >> 16) & 0x7FFF;
        let v_sync_end = edm & 0x7FFF;
        
        
        if cal == 0 || cfa == 0 || cal > 8192 || cfa > 8192 {
            return None;
        }
        
        let h_active = if ide <= cal { ide } else { cal };
        let v_active = if jpr <= cfa { jpr } else { cfa };
        
        if h_active == 0 || v_active == 0 {
            return None;
        }
        
        let mgy = h_sync_start.saturating_sub(h_active);
        let mhc = h_sync_end.saturating_sub(h_sync_start);
        let mgw = cal.saturating_sub(h_sync_end);
        
        let pqt = v_sync_start.saturating_sub(v_active);
        let pqx = v_sync_end.saturating_sub(v_sync_start);
        let pqs = cfa.saturating_sub(v_sync_end);
        
        
        let fmb = boa(mmio, pipe, dcn::CLS_);
        let pixel_clock_khz = if fmb != 0 && fmb != 0xFFFFFFFF {
            
            (fmb & 0xFFFF) * 10 
        } else {
            
            ((cal as u64) * (cfa as u64) * 60 / 1000) as u32
        };
        
        let refresh = if cal > 0 && cfa > 0 && pixel_clock_khz > 0 {
            (pixel_clock_khz as u64 * 1000) / (cal as u64 * cfa as u64)
        } else {
            60 
        };
        
        Some(Cv {
            h_active, h_front_porch: mgy, h_sync_width: mhc, h_back_porch: mgw,
            v_active, v_front_porch: pqt, v_sync_width: pqx, v_back_porch: pqs,
            pixel_clock_khz,
            refresh_hz: refresh as u32,
            h_sync_positive: true,
            v_sync_positive: true,
        })
    }
}


pub fn qre(mmio: u64, pipe: u8, mode: &Cv) {
    crate::serial_println!("[DCN] Programming OTG{} for {}", pipe, mode.modeline());
    
    unsafe {
        
        buo(mmio, pipe, dcn::LN_, 0);
        
        
        buo(mmio, pipe, dcn::BED_, mode.h_total() - 1);
        
        let mgv = ((mode.h_sync_start()) << 16) | mode.h_active;
        buo(mmio, pipe, dcn::BEB_, mgv);
        
        let drc = ((mode.h_sync_start()) << 16) | mode.h_sync_end();
        buo(mmio, pipe, dcn::BEC_, drc);
        
        
        buo(mmio, pipe, dcn::BEH_, mode.v_total() - 1);
        
        let pqr = ((mode.v_sync_start()) << 16) | mode.v_active;
        buo(mmio, pipe, dcn::BEF_, pqr);
        
        let edm = ((mode.v_sync_start()) << 16) | mode.v_sync_end();
        buo(mmio, pipe, dcn::BEG_, edm);
        
        crate::serial_println!("[DCN] OTG{} timing programmed: {}x{} htotal={} vtotal={}",
            pipe, mode.h_active, mode.v_active, mode.h_total(), mode.v_total());
    }
}


pub fn qfa(mmio: u64, pipe: u8) {
    unsafe {
        
        let mut ajj = boa(mmio, pipe, dcn::LN_);
        ajj |= 1; 
        buo(mmio, pipe, dcn::LN_, ajj);
        crate::serial_println!("[DCN] OTG{} enabled", pipe);
    }
}


pub fn qdc(mmio: u64, pipe: u8) {
    unsafe {
        let mut ajj = boa(mmio, pipe, dcn::LN_);
        ajj &= !1; 
        buo(mmio, pipe, dcn::LN_, ajj);
        crate::serial_println!("[DCN] OTG{} disabled", pipe);
    }
}






pub fn qbc(mmio: u64, pipe: u8, surface: &Qf) {
    crate::serial_println!("[DCN] Configuring HUBP{} for {}x{} @ {:#X}", 
        pipe, surface.width, surface.height, surface.fb_phys_addr);
    
    unsafe {
        
        let hdz = (surface.fb_phys_addr >> 32) as u32;
        let dhe = (surface.fb_phys_addr & 0xFFFFFFFF) as u32;
        
        dro(mmio, pipe, dcn::AYH_, hdz);
        dro(mmio, pipe, dcn::AYI_, dhe);
        
        
        dro(mmio, pipe, dcn::AYK_, surface.pitch / surface.format.bpp());
        
        
        let otm = (surface.height << 16) | surface.width;
        dro(mmio, pipe, dcn::AYL_, otm);
        
        
        dro(mmio, pipe, dcn::AYJ_, surface.format.dcn_format_code());
        
        crate::serial_println!("[DCN] HUBP{} configured: addr={:#010X}:{:#010X} pitch={} fmt={:?}",
            pipe, hdz, dhe, surface.pitch, surface.format);
    }
}







pub fn init(mmio_base: u64) {
    crate::log!("[DCN] ═══════════════════════════════════════════════════════");
    crate::log!("[DCN] Display Core Next 2.0 — Phase 2: Display Configuration");
    crate::log!("[DCN] ═══════════════════════════════════════════════════════");
    
    
    let fqw = unsafe { kj(mmio_base, dcn::BTU_) };
    crate::serial_println!("[DCN] DCN_VERSION raw: {:#010X}", fqw);
    
    let fqu = (fqw >> 8) & 0xFF;
    let fqv = fqw & 0xFF;
    crate::log!("[DCN] DCN version: {}.{}", fqu, fqv);
    
    
    let lgi = unsafe { kj(mmio_base, dcn::BUU_) };
    crate::serial_println!("[DCN] DMCUB_STATUS: {:#010X}", lgi);
    
    
    crate::log!("[DCN] Detecting display connectors...");
    let mut connectors = ldu(mmio_base);
    
    let mut fof = 0u8;
    for et in &connectors {
        let bvz = match et.status {
            ConnectorStatus::Connected => {
                fof += 1;
                "CONNECTED"
            },
            ConnectorStatus::Disconnected => "disconnected",
            ConnectorStatus::Unknown => "unknown",
        };
        crate::log!("[DCN]   Connector {}: {} — {}", 
            et.index, et.connector_type.name(), bvz);
    }
    crate::log!("[DCN] Found {} connected display(s)", fof);
    
    
    for et in &mut connectors {
        if et.status == ConnectorStatus::Connected {
            ocp(mmio_base, et);
        }
    }
    
    
    crate::log!("[DCN] Reading active display modes...");
    let mut active_displays = 0u8;
    
    for pipe in 0..6u8 {
        if let Some(mode) = ocn(mmio_base, pipe) {
            crate::log!("[DCN]   OTG{}: {} (active)", pipe, mode.modeline());
            
            if (pipe as usize) < connectors.len() {
                connectors[pipe as usize].current_mode = Some(mode);
            }
            active_displays += 1;
        } else {
            crate::serial_println!("[DCN]   OTG{}: inactive", pipe);
        }
    }
    
    
    crate::log!("[DCN] Reading HUBP surface configurations...");
    let mut scanouts: [Option<Qf>; 6] = [None, None, None, None, None, None];
    
    for pipe in 0..6u8 {
        unsafe {
            let oyq = drn(mmio_base, pipe, dcn::AYH_);
            let oyr = drn(mmio_base, pipe, dcn::AYI_);
            let jjw = drn(mmio_base, pipe, dcn::AYJ_);
            let jjx = drn(mmio_base, pipe, dcn::AYK_);
            let jjy = drn(mmio_base, pipe, dcn::AYL_);
            
            let addr = ((oyq as u64) << 32) | (oyr as u64);
            
            if addr != 0 && addr != 0xFFFFFFFFFFFFFFFF && jjw != 0xFFFFFFFF {
                let width = jjy & 0xFFFF;
                let height = (jjy >> 16) & 0xFFFF;
                
                crate::serial_println!("[DCN]   HUBP{}: addr={:#014X} size={}x{} pitch={} config={:#010X}", 
                    pipe, addr, width, height, jjx, jjw);
                
                if width > 0 && height > 0 && width < 16384 && height < 16384 {
                    scanouts[pipe as usize] = Some(Qf {
                        fb_phys_addr: addr,
                        width,
                        height,
                        pitch: jjx * 4, 
                        format: SurfaceFormat::Xrgb8888,
                    });
                    crate::log!("[DCN]   HUBP{}: {}x{} surface at {:#014X}", pipe, width, height, addr);
                }
            }
        }
    }
    
    
    crate::log!("[DCN] ───────────────────────────────────────────────────────");
    crate::log!("[DCN] DCN {}.{} — {} connector(s), {} active display(s)",
        fqu, fqv, fof, active_displays);
    for et in &connectors {
        if et.status == ConnectorStatus::Connected {
            if let Some(ref mode) = et.current_mode {
                crate::log!("[DCN]   Output {}: {} {}x{}@{}Hz", 
                    et.index, et.connector_type.name(),
                    mode.h_active, mode.v_active, mode.refresh_hz);
            } else {
                crate::log!("[DCN]   Output {}: {} (connected, no active mode)", 
                    et.index, et.connector_type.name());
            }
        }
    }
    crate::log!("[DCN] ───────────────────────────────────────────────────────");
    crate::log!("[DCN] Phase 2 complete — Display engine probed");
    
    
    let mut state = KP_.lock();
    state.initialized = true;
    state.connectors = connectors;
    state.active_displays = active_displays;
    state.dcn_version = (fqu as u8, fqv as u8);
    state.max_pipes = 6;
    state.scanouts = scanouts;
    ARW_.store(true, Ordering::SeqCst);
}






pub fn is_ready() -> bool {
    ARW_.load(Ordering::Relaxed)
}


pub fn qhi() -> Vec<Lm> {
    KP_.lock().connectors.clone()
}


pub fn pxq() -> u8 {
    KP_.lock().active_displays
}


pub fn dcn_version() -> (u8, u8) {
    KP_.lock().dcn_version
}


pub fn summary() -> String {
    let state = KP_.lock();
    if state.initialized {
        let bfn = state.connectors.iter()
            .filter(|c| c.status == ConnectorStatus::Connected)
            .count();
        format!("DCN {}.{} — {} display(s), {} connected", 
            state.dcn_version.0, state.dcn_version.1,
            state.active_displays, bfn)
    } else {
        String::from("DCN not initialized")
    }
}


pub fn info_lines() -> Vec<String> {
    let mut lines = Vec::new();
    let state = KP_.lock();
    
    if state.initialized {
        lines.push(format!("DCN {}.{} Display Engine", state.dcn_version.0, state.dcn_version.1));
        lines.push(format!("  Pipes: {} max, {} active", state.max_pipes, state.active_displays));
        lines.push(String::new());
        
        for et in &state.connectors {
            let status = match et.status {
                ConnectorStatus::Connected => "CONNECTED",
                ConnectorStatus::Disconnected => "disconnected",
                ConnectorStatus::Unknown => "unknown",
            };
            
            let mut line = format!("  Connector {}: {} [{}]", 
                et.index, et.connector_type.name(), status);
            
            if let Some(ref mode) = et.current_mode {
                line.push_str(&format!(" — {}x{}@{}Hz", mode.h_active, mode.v_active, mode.refresh_hz));
            }
            
            lines.push(line);
        }
        
        
        lines.push(String::new());
        lines.push(String::from("  Active Surfaces:"));
        for (i, scanout) in state.scanouts.iter().enumerate() {
            if let Some(ref j) = scanout {
                lines.push(format!("    HUBP{}: {}x{} {:?} @ {:#X}", 
                    i, j.width, j.height, j.format, j.fb_phys_addr));
            }
        }
    } else {
        lines.push(String::from("DCN display engine not initialized"));
    }
    
    lines
}
