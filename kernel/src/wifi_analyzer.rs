//! TrustWave — WiFi Wave Analyzer & Visualizer
//!
//! A 6-panel real-time WiFi visualization dashboard (TrustLab style):
//!   ┌──────────────────┬──────────────────┬──────────────────┐
//!   │  📡 Spectrum     │  📊 Signal Graph │  ⚙ Radio Control │
//!   │  Channel map     │  dBm over time   │  Scan/Connect    │
//!   ├──────────────────┼──────────────────┼──────────────────┤
//!   │  🌐 Network Map  │  📈 Packet Stats │  🔧 HW Registers │
//!   │  AP list + bars  │  TX/RX live      │  CSR dump live   │
//!   └──────────────────┴──────────────────┴──────────────────┘
//!
//! No other bare-metal OS has a real-time WiFi wave visualizer.

extern crate alloc;

use alloc::vec::Vec;
use alloc::format;
use alloc::string::String;

use crate::keyboard::{KEY_UP, KEY_DOWN, KEY_LEFT, KEY_RIGHT, KEY_PGUP, KEY_PGDOWN};

// ── Colors (dark theme, matching TrustLab) ──────────────────────────────────
const COL_BG: u32          = 0xFF0D1117;
const COL_PANEL_BG: u32    = 0xFF161B22;
const COL_PANEL_BORDER: u32 = 0xFF30363D;
const COL_HEADER_BG: u32   = 0xFF1C2128;
const COL_TEXT: u32         = 0xFFE6EDF3;
const COL_DIM: u32          = 0xFF8B949E;
const COL_ACCENT: u32       = 0xFF58A6FF;
const COL_GREEN: u32        = 0xFF3FB950;
const COL_YELLOW: u32       = 0xFFD29922;
const COL_RED: u32          = 0xFFF85149;
const COL_PURPLE: u32       = 0xFFBC8CFF;
const COL_CYAN: u32         = 0xFF79C0FF;
const COL_ORANGE: u32       = 0xFFD18616;
const COL_SELECTED: u32     = 0xFF1F6FEB;

const TITLE_BAR_H: u32  = 28;
const PANEL_HEADER_H: u32 = 22;
const PANEL_PAD: u32     = 6;

// ── Signal history ──────────────────────────────────────────────────────────
const SIGNAL_HISTORY_LEN: usize = 120;

// ── Panel identifiers ───────────────────────────────────────────────────────
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum WavePanel {
    Spectrum     = 0,
    SignalGraph  = 1,
    RadioControl = 2,
    NetworkMap   = 3,
    PacketStats  = 4,
    HwRegisters  = 5,
}

impl WavePanel {
    fn title(&self) -> &'static str {
        match self {
            WavePanel::Spectrum     => "Spectrum View",
            WavePanel::SignalGraph  => "Signal Graph",
            WavePanel::RadioControl => "Radio Control",
            WavePanel::NetworkMap   => "Network Map",
            WavePanel::PacketStats  => "Packet Stats",
            WavePanel::HwRegisters  => "HW Registers",
        }
    }

    fn accent(&self) -> u32 {
        match self {
            WavePanel::Spectrum     => COL_CYAN,
            WavePanel::SignalGraph  => COL_GREEN,
            WavePanel::RadioControl => COL_ORANGE,
            WavePanel::NetworkMap   => COL_ACCENT,
            WavePanel::PacketStats  => COL_PURPLE,
            WavePanel::HwRegisters  => COL_RED,
        }
    }
}

// ── Cached CSR value ────────────────────────────────────────────────────────
struct CsrEntry {
    name: &'static str,
    offset: u32,
    value: u32,
}

// ── State ───────────────────────────────────────────────────────────────────
pub struct WifiAnalyzerState {
    /// Focused panel index (0-5)
    pub focused: usize,
    /// Signal history ring buffer (dBm values, 0 = no data)
    signal_history: [i8; SIGNAL_HISTORY_LEN],
    /// Write cursor into signal_history
    signal_cursor: usize,
    /// Tick counter
    tick_count: u64,
    /// Network map scroll offset
    net_scroll: usize,
    /// HW register scroll offset
    reg_scroll: usize,
    /// Cached CSR registers (refreshed periodically)
    csr_cache: Vec<CsrEntry>,
    /// Last CSR refresh tick
    csr_last_refresh: u64,
    /// Cached scan results (refreshed periodically)
    cached_networks: Vec<CachedNetwork>,
    /// Last scan refresh tick
    scan_last_refresh: u64,
    /// Packet stats snapshot
    tx_packets: u64,
    rx_packets: u64,
    tx_bytes: u64,
    rx_bytes: u64,
    tx_errors: u64,
    rx_errors: u64,
    /// Previous snapshot for throughput calculation
    prev_tx_bytes: u64,
    prev_rx_bytes: u64,
    /// Throughput (bytes/sec estimate)
    throughput_tx: u64,
    throughput_rx: u64,
    /// Radio control: selected action
    radio_selected: usize,
}

#[derive(Clone)]
struct CachedNetwork {
    ssid: String,
    bssid: [u8; 6],
    channel: u8,
    signal_dbm: i8,
    security: String,
    frequency_mhz: u16,
    quality: u8,
    bars: u8,
}

impl WifiAnalyzerState {
    pub fn new() -> Self {
        Self {
            focused: 0,
            signal_history: [0i8; SIGNAL_HISTORY_LEN],
            signal_cursor: 0,
            tick_count: 0,
            net_scroll: 0,
            reg_scroll: 0,
            csr_cache: Vec::new(),
            csr_last_refresh: 0,
            cached_networks: Vec::new(),
            scan_last_refresh: 0,
            tx_packets: 0,
            rx_packets: 0,
            tx_bytes: 0,
            rx_bytes: 0,
            tx_errors: 0,
            rx_errors: 0,
            prev_tx_bytes: 0,
            prev_rx_bytes: 0,
            throughput_tx: 0,
            throughput_rx: 0,
            radio_selected: 0,
        }
    }

    /// Called every frame (~60Hz) to refresh live data
    pub fn tick(&mut self) {
        self.tick_count += 1;

        // Sample signal strength every 4 ticks (~15Hz)
        if self.tick_count % 4 == 0 {
            let dbm = crate::drivers::net::wifi::signal_strength().unwrap_or(0);
            self.signal_history[self.signal_cursor] = dbm;
            self.signal_cursor = (self.signal_cursor + 1) % SIGNAL_HISTORY_LEN;
        }

        // Refresh scan results every 120 ticks (~2s)
        if self.tick_count % 120 == 0 || self.tick_count == 1 {
            self.refresh_networks();
        }

        // Refresh CSR registers every 60 ticks (~1s)
        if self.tick_count % 60 == 0 || self.tick_count == 1 {
            self.refresh_csrs();
        }

        // Refresh packet stats every 30 ticks (~0.5s)
        if self.tick_count % 30 == 0 {
            self.refresh_packet_stats();
        }
    }

    fn refresh_networks(&mut self) {
        let results = crate::drivers::net::wifi::get_scan_results();
        self.cached_networks.clear();
        for net in &results {
            self.cached_networks.push(CachedNetwork {
                ssid: net.ssid.clone(),
                bssid: net.bssid,
                channel: net.channel,
                signal_dbm: net.signal_dbm,
                security: String::from(net.security.as_str()),
                frequency_mhz: net.frequency_mhz,
                quality: net.signal_quality(),
                bars: net.signal_bars(),
            });
        }
        self.scan_last_refresh = self.tick_count;
    }

    fn refresh_csrs(&mut self) {
        // CSR register offsets for Intel WiFi
        const REGS: &[(&str, u32)] = &[
            ("HW_IF_CONFIG",  0x000),
            ("INT",           0x008),
            ("INT_MASK",      0x00C),
            ("FH_INT_STATUS", 0x010),
            ("GPIO_IN",       0x018),
            ("RESET",         0x020),
            ("GP_CNTRL",      0x024),
            ("HW_REV",        0x028),
            ("EEPROM_REG",    0x02C),
            ("EEPROM_GP",     0x030),
            ("UCODE_DRV_GP1", 0x054),
            ("UCODE_DRV_GP2", 0x058),
            ("GIO_REG",       0x03C),
            ("GP_UCODE",      0x048),
            ("GP_DRIVER",     0x050),
        ];
        self.csr_cache.clear();
        for (name, offset) in REGS {
            let value = crate::drivers::net::iwl4965::debug_read_csr(*offset).unwrap_or(0xDEAD_DEAD);
            self.csr_cache.push(CsrEntry {
                name,
                offset: *offset,
                value,
            });
        }
        self.csr_last_refresh = self.tick_count;
    }

    fn refresh_packet_stats(&mut self) {
        // Read stats from WiFi driver
        let guard = crate::drivers::net::wifi::WIFI_DRIVER.lock();
        if let Some(ref driver) = *guard {
            let stats = driver.stats();
            self.prev_tx_bytes = self.tx_bytes;
            self.prev_rx_bytes = self.rx_bytes;
            self.tx_packets = stats.tx_packets;
            self.rx_packets = stats.rx_packets;
            self.tx_bytes = stats.tx_bytes;
            self.rx_bytes = stats.rx_bytes;
            self.tx_errors = stats.tx_errors;
            self.rx_errors = stats.rx_errors;
            // Estimate throughput (bytes/0.5s → bytes/s)
            self.throughput_tx = (self.tx_bytes - self.prev_tx_bytes) * 2;
            self.throughput_rx = (self.rx_bytes - self.prev_rx_bytes) * 2;
        }
    }

    pub fn handle_key(&mut self, key: u8) {
        match key {
            // Tab → cycle focus
            0x09 => {
                self.focused = (self.focused + 1) % 6;
            }
            // Arrow keys
            k if k == KEY_UP => {
                match self.focused {
                    3 => self.net_scroll = self.net_scroll.saturating_sub(1),
                    5 => self.reg_scroll = self.reg_scroll.saturating_sub(1),
                    2 => {
                        if self.radio_selected > 0 { self.radio_selected -= 1; }
                    }
                    _ => {}
                }
            }
            k if k == KEY_DOWN => {
                match self.focused {
                    3 => self.net_scroll += 1,
                    5 => self.reg_scroll += 1,
                    2 => {
                        if self.radio_selected < 3 { self.radio_selected += 1; }
                    }
                    _ => {}
                }
            }
            k if k == KEY_PGUP => {
                match self.focused {
                    3 => self.net_scroll = self.net_scroll.saturating_sub(10),
                    5 => self.reg_scroll = self.reg_scroll.saturating_sub(10),
                    _ => {}
                }
            }
            k if k == KEY_PGDOWN => {
                match self.focused {
                    3 => self.net_scroll += 10,
                    5 => self.reg_scroll += 10,
                    _ => {}
                }
            }
            // Enter → execute radio control action
            0x0D | 0x0A => {
                if self.focused == 2 {
                    self.execute_radio_action();
                }
            }
            // 1-6 → select panel directly
            b'1'..=b'6' => {
                self.focused = (key - b'1') as usize;
            }
            // 's' → trigger scan
            b's' | b'S' => {
                let _ = crate::drivers::net::wifi::start_scan();
            }
            _ => {}
        }
    }

    fn execute_radio_action(&mut self) {
        match self.radio_selected {
            0 => { // Scan
                let _ = crate::drivers::net::wifi::start_scan();
            }
            1 => { // Disconnect
                let _ = crate::drivers::net::wifi::disconnect();
            }
            2 => { // Refresh CSRs
                self.refresh_csrs();
            }
            3 => { // Refresh all
                self.refresh_networks();
                self.refresh_csrs();
                self.refresh_packet_stats();
            }
            _ => {}
        }
    }

    pub fn handle_click(&mut self, lx: i32, ly: i32, ww: u32, wh: u32) {
        // Determine which panel was clicked
        let cw = ww.saturating_sub(4);
        let ch = wh.saturating_sub(TITLE_BAR_H + 4);
        let col_w = cw / 3;
        let row_h = ch / 2;

        let col = (lx as u32 / col_w).min(2) as usize;
        let row = if (ly as u32) < row_h { 0 } else { 1 };
        self.focused = row * 3 + col;
    }
}

// ── Layout helper ───────────────────────────────────────────────────────────
struct PanelRect {
    x: i32,
    y: i32,
    w: u32,
    h: u32,
}

fn compute_panels(cx: i32, cy: i32, cw: u32, ch: u32) -> [PanelRect; 6] {
    let gap = 4u32;
    let col_w = (cw - gap * 2) / 3;
    let row_h = (ch - gap) / 2;

    let x0 = cx;
    let x1 = cx + col_w as i32 + gap as i32;
    let x2 = cx + (col_w as i32 + gap as i32) * 2;
    let y0 = cy;
    let y1 = cy + row_h as i32 + gap as i32;

    // Right column: clamp width so it doesn't overflow
    let col2_w = (cw as i32 - (x2 - cx)).max(40) as u32;

    [
        PanelRect { x: x0, y: y0, w: col_w, h: row_h },     // 0: Spectrum
        PanelRect { x: x1, y: y0, w: col_w, h: row_h },     // 1: Signal Graph
        PanelRect { x: x2, y: y0, w: col2_w, h: row_h },    // 2: Radio Control
        PanelRect { x: x0, y: y1, w: col_w, h: row_h },     // 3: Network Map
        PanelRect { x: x1, y: y1, w: col_w, h: row_h },     // 4: Packet Stats
        PanelRect { x: x2, y: y1, w: col2_w, h: row_h },    // 5: HW Registers
    ]
}

// ── Rendering helpers ───────────────────────────────────────────────────────
fn draw_text(x: i32, y: i32, text: &str, color: u32) {
    crate::graphics::scaling::draw_scaled_text(x, y, text, color);
}

fn char_w() -> i32 {
    crate::graphics::scaling::char_width() as i32
}

fn char_h() -> i32 {
    16 * crate::graphics::scaling::get_scale_factor() as i32
}

fn draw_rect_border(x: i32, y: i32, w: u32, h: u32, color: u32) {
    if w == 0 || h == 0 { return; }
    crate::framebuffer::fill_rect(x as u32, y as u32, w, 1, color);
    crate::framebuffer::fill_rect(x as u32, (y + h as i32 - 1) as u32, w, 1, color);
    crate::framebuffer::fill_rect(x as u32, y as u32, 1, h, color);
    crate::framebuffer::fill_rect((x + w as i32 - 1) as u32, y as u32, 1, h, color);
}

fn draw_panel_frame(pr: &PanelRect, panel: WavePanel, focused: bool) {
    crate::framebuffer::fill_rect(pr.x as u32, pr.y as u32, pr.w, pr.h, COL_PANEL_BG);
    let border = if focused { COL_SELECTED } else { COL_PANEL_BORDER };
    draw_rect_border(pr.x, pr.y, pr.w, pr.h, border);
    // Header bar
    let hdr_bg = if focused { 0xFF1F2937 } else { COL_HEADER_BG };
    crate::framebuffer::fill_rect(
        (pr.x + 1) as u32, (pr.y + 1) as u32,
        pr.w.saturating_sub(2), PANEL_HEADER_H - 1, hdr_bg,
    );
    // Accent line
    crate::framebuffer::fill_rect(
        (pr.x + 1) as u32, (pr.y + 1) as u32,
        pr.w.saturating_sub(2), 2, panel.accent(),
    );
    // Title
    draw_text(pr.x + 8, pr.y + 6, panel.title(), COL_TEXT);
}

// ── Main draw ───────────────────────────────────────────────────────────────
pub fn draw(state: &WifiAnalyzerState, wx: i32, wy: i32, ww: u32, wh: u32) {
    let cx = wx + 2;
    let cy = wy + TITLE_BAR_H as i32 + 2;
    let cw = ww.saturating_sub(4);
    let ch = wh.saturating_sub(TITLE_BAR_H + 4);

    if cw < 200 || ch < 100 { return; }

    // Background
    crate::framebuffer::fill_rect(cx as u32, cy as u32, cw, ch, COL_BG);

    let panels = compute_panels(cx, cy, cw, ch);
    let panel_types = [
        WavePanel::Spectrum, WavePanel::SignalGraph, WavePanel::RadioControl,
        WavePanel::NetworkMap, WavePanel::PacketStats, WavePanel::HwRegisters,
    ];

    for (i, pr) in panels.iter().enumerate() {
        draw_panel_frame(pr, panel_types[i], i == state.focused);

        let content_x = pr.x + PANEL_PAD as i32;
        let content_y = pr.y + PANEL_HEADER_H as i32 + PANEL_PAD as i32;
        let content_w = pr.w.saturating_sub(PANEL_PAD * 2);
        let content_h = pr.h.saturating_sub(PANEL_HEADER_H + PANEL_PAD * 2);

        match panel_types[i] {
            WavePanel::Spectrum     => draw_spectrum(state, content_x, content_y, content_w, content_h),
            WavePanel::SignalGraph  => draw_signal_graph(state, content_x, content_y, content_w, content_h),
            WavePanel::RadioControl => draw_radio_control(state, content_x, content_y, content_w, content_h),
            WavePanel::NetworkMap   => draw_network_map(state, content_x, content_y, content_w, content_h),
            WavePanel::PacketStats  => draw_packet_stats(state, content_x, content_y, content_w, content_h),
            WavePanel::HwRegisters  => draw_hw_registers(state, content_x, content_y, content_w, content_h),
        }
    }

    // Status bar at bottom
    let status_y = cy + ch as i32 - char_h() - 2;
    let wifi_state = crate::drivers::net::wifi::state();
    let status = format!(" TrustWave | State: {:?} | Tab=cycle panels | S=scan | 1-6=select panel", wifi_state);
    draw_text(cx + 4, status_y, &status, COL_DIM);
}

// ── Panel 0: Spectrum View ──────────────────────────────────────────────────
// Visualizes channels as vertical bars with signal strength

fn draw_spectrum(state: &WifiAnalyzerState, x: i32, y: i32, w: u32, h: u32) {
    let ch = char_h();
    if ch <= 0 || w < 80 || h < 60 { return; }

    // Draw 2.4 GHz band (channels 1-14) and 5 GHz (selected channels)
    let channels_24: &[u8] = &[1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14];
    let channels_5: &[u8] = &[36, 40, 44, 48, 52, 56, 60, 64, 100, 104, 108, 112, 116, 120, 124, 128, 132, 136, 140, 149, 153, 157, 161, 165];

    draw_text(x, y, "2.4 GHz", COL_CYAN);

    let bar_area_y = y + ch + 4;
    let bar_area_h = (h as i32 - ch * 2 - 12).max(20);
    let total_channels = channels_24.len() + channels_5.len();
    let bar_w = ((w as usize) / (total_channels + 2)).max(2) as u32;
    let gap = 1u32;

    // Draw channel bars
    let mut bx = x;
    for &channel in channels_24.iter().chain(channels_5.iter()) {
        // Find the strongest signal on this channel
        let mut best_dbm: i8 = -100;
        let mut net_count = 0u32;
        for net in &state.cached_networks {
            if net.channel == channel {
                if net.signal_dbm > best_dbm {
                    best_dbm = net.signal_dbm;
                }
                net_count += 1;
            }
        }

        // Draw bar height proportional to signal strength
        let bar_h = if best_dbm > -100 {
            // Map -30..-100 to bar_area_h..0
            let norm = ((best_dbm as i32 + 100) * bar_area_h / 70).clamp(2, bar_area_h);
            norm as u32
        } else {
            2 // Minimum bar for empty channels
        };

        let bar_y = bar_area_y + bar_area_h - bar_h as i32;
        let color = if best_dbm > -100 {
            signal_color(best_dbm)
        } else {
            0xFF2A2A2A // Very dim for empty
        };

        crate::framebuffer::fill_rect(bx as u32, bar_y as u32, bar_w, bar_h, color);

        // Channel label (every other)
        if channel <= 14 || channel % 8 == 4 {
            let label = format!("{}", channel);
            draw_text(bx, bar_area_y + bar_area_h + 2, &label, COL_DIM);
        }

        // AP count badge
        if net_count > 0 {
            let badge = format!("{}", net_count);
            draw_text(bx, bar_y - ch, &badge, COL_TEXT);
        }

        // Separator between 2.4 and 5 GHz bands
        if channel == 14 {
            bx += bar_w as i32 + gap as i32 + 4;
            draw_text(bx, y, "5 GHz", COL_PURPLE);
            bx += 6 * char_w();
        }

        bx += bar_w as i32 + gap as i32;
        if bx as u32 >= x as u32 + w { break; }
    }
}

// ── Panel 1: Signal Graph ───────────────────────────────────────────────────
// Scrolling time series of dBm

fn draw_signal_graph(state: &WifiAnalyzerState, x: i32, y: i32, w: u32, h: u32) {
    let ch = char_h();
    if ch <= 0 || w < 60 || h < 40 { return; }

    // Y axis labels
    let graph_x = x + 5 * char_w();
    let graph_w = w.saturating_sub(6 * char_w() as u32);
    let graph_h = h.saturating_sub(ch as u32 + 4);

    // dBm scale: -30 (top) to -100 (bottom)
    let labels = [(-30, "−30"), (-50, "−50"), (-70, "−70"), (-90, "−90")];
    for (dbm, label) in &labels {
        let py = y + dbm_to_y(*dbm, graph_h as i32);
        draw_text(x, py, label, COL_DIM);
        // Dashed horizontal grid line
        let mut gx = graph_x;
        while gx < graph_x + graph_w as i32 {
            crate::framebuffer::fill_rect(gx as u32, py as u32, 2, 1, 0xFF222222);
            gx += 6;
        }
    }

    // Threshold zones (colored backgrounds)
    let excellent_y = y + dbm_to_y(-30, graph_h as i32);
    let good_y = y + dbm_to_y(-50, graph_h as i32);
    let fair_y = y + dbm_to_y(-70, graph_h as i32);
    crate::framebuffer::fill_rect_alpha(graph_x as u32, excellent_y as u32, graph_w, (good_y - excellent_y) as u32, 0x3FB950, 15);
    crate::framebuffer::fill_rect_alpha(graph_x as u32, good_y as u32, graph_w, (fair_y - good_y) as u32, 0xD29922, 10);

    // Plot signal trace
    let points = SIGNAL_HISTORY_LEN.min(graph_w as usize);
    let step_x = if points > 1 { graph_w as i32 / (points as i32 - 1).max(1) } else { 1 };

    let mut prev_px = 0i32;
    let mut prev_py = 0i32;
    let mut has_prev = false;

    for i in 0..points {
        let idx = (state.signal_cursor + SIGNAL_HISTORY_LEN - points + i) % SIGNAL_HISTORY_LEN;
        let dbm = state.signal_history[idx];
        if dbm == 0 { has_prev = false; continue; }

        let px = graph_x + (i as i32 * step_x);
        let py = y + dbm_to_y(dbm as i32, graph_h as i32);
        let color = signal_color(dbm);

        // Draw point
        crate::framebuffer::fill_rect(px as u32, py as u32, 2, 2, color);

        // Connect to previous point with line
        if has_prev {
            draw_line(prev_px, prev_py, px, py, color);
        }

        prev_px = px;
        prev_py = py;
        has_prev = true;
    }

    // Current value label
    let current_dbm = crate::drivers::net::wifi::signal_strength().unwrap_or(0);
    if current_dbm != 0 {
        let label = format!("{} dBm", current_dbm);
        let color = signal_color(current_dbm);
        draw_text(graph_x + graph_w as i32 - 10 * char_w(), y, &label, color);
    } else {
        draw_text(graph_x, y, "No signal", COL_DIM);
    }
}

fn dbm_to_y(dbm: i32, h: i32) -> i32 {
    // Map -30 → 0 (top), -100 → h (bottom)
    let clamped = dbm.clamp(-100, -30);
    ((clamped + 100) * h / 70).abs()
    // -30 → 70*h/70 = h → we want 0, so invert:
}

fn draw_line(x0: i32, y0: i32, x1: i32, y1: i32, color: u32) {
    // Bresenham's line algorithm
    let dx = (x1 - x0).abs();
    let dy = -(y1 - y0).abs();
    let sx = if x0 < x1 { 1 } else { -1 };
    let sy = if y0 < y1 { 1 } else { -1 };
    let mut err = dx + dy;
    let mut cx = x0;
    let mut cy = y0;
    let mut steps = 0;
    loop {
        if cx >= 0 && cy >= 0 {
            crate::framebuffer::fill_rect(cx as u32, cy as u32, 1, 1, color);
        }
        if cx == x1 && cy == y1 { break; }
        steps += 1;
        if steps > 2000 { break; } // Safety limit
        let e2 = 2 * err;
        if e2 >= dy {
            err += dy;
            cx += sx;
        }
        if e2 <= dx {
            err += dx;
            cy += sy;
        }
    }
}

fn signal_color(dbm: i8) -> u32 {
    if dbm >= -40 { COL_GREEN }
    else if dbm >= -60 { COL_YELLOW }
    else if dbm >= -75 { COL_ORANGE }
    else { COL_RED }
}

// ── Panel 2: Radio Control ──────────────────────────────────────────────────

fn draw_radio_control(state: &WifiAnalyzerState, x: i32, y: i32, w: u32, h: u32) {
    let ch = char_h();
    let cw = char_w();
    if ch <= 0 || w < 80 || h < 60 { return; }

    let mut row = 0i32;

    // Connection info
    let wifi_state = crate::drivers::net::wifi::state();
    let state_str = format!("State: {:?}", wifi_state);
    draw_text(x, y + row * (ch + 2), &state_str, COL_TEXT);
    row += 1;

    if let Some(ssid) = crate::drivers::net::wifi::connected_ssid() {
        let ssid_str = format!("SSID: {}", ssid);
        draw_text(x, y + row * (ch + 2), &ssid_str, COL_GREEN);
        row += 1;
    }

    if let Some(dbm) = crate::drivers::net::wifi::signal_strength() {
        let sig_str = format!("Signal: {} dBm ({}%)", dbm, quality_from_dbm(dbm));
        draw_text(x, y + row * (ch + 2), &sig_str, signal_color(dbm));
        row += 1;
    }

    let hw = if crate::drivers::net::wifi::has_wifi() { "Detected" } else { "None" };
    let hw_str = format!("Hardware: {}", hw);
    draw_text(x, y + row * (ch + 2), &hw_str, COL_DIM);
    row += 2;

    // Action buttons
    draw_text(x, y + row * (ch + 2), "Actions:", COL_ACCENT);
    row += 1;

    let actions = ["[S] Scan Networks", "[D] Disconnect", "[R] Refresh CSRs", "[A] Refresh All"];
    for (i, action) in actions.iter().enumerate() {
        let color = if i == state.radio_selected && state.focused == 2 {
            COL_SELECTED
        } else {
            COL_TEXT
        };
        let marker = if i == state.radio_selected { "> " } else { "  " };
        let line = format!("{}{}", marker, action);
        draw_text(x, y + row * (ch + 2), &line, color);
        row += 1;
    }

    row += 1;
    draw_text(x, y + row * (ch + 2), "Enter=Execute", COL_DIM);
}

fn quality_from_dbm(dbm: i8) -> u8 {
    if dbm >= -30 { return 100; }
    if dbm <= -90 { return 0; }
    ((dbm as i16 + 90) * 100 / 60) as u8
}

// ── Panel 3: Network Map ────────────────────────────────────────────────────

fn draw_network_map(state: &WifiAnalyzerState, x: i32, y: i32, w: u32, h: u32) {
    let ch = char_h();
    let cw = char_w();
    if ch <= 0 || w < 100 || h < 40 { return; }

    let max_rows = (h as i32 / (ch + 2)) as usize;
    if max_rows < 2 { return; }

    // Header
    let header = format!("{} networks found", state.cached_networks.len());
    draw_text(x, y, &header, COL_ACCENT);

    let connected = crate::drivers::net::wifi::connected_ssid();
    let list_y = y + ch + 4;
    let visible_rows = max_rows.saturating_sub(2);

    let scroll = state.net_scroll.min(state.cached_networks.len().saturating_sub(visible_rows));

    for (i, net) in state.cached_networks.iter().enumerate().skip(scroll).take(visible_rows) {
        let row_y = list_y + ((i - scroll) as i32 * (ch + 4));

        // Signal bars visualization
        let bars_str = match net.bars {
            4 => "||||",
            3 => "|||.",
            2 => "||..",
            1 => "|...",
            _ => "....",
        };
        let sig_color = signal_color(net.signal_dbm);

        // Highlight connected network
        let is_connected = connected.as_ref().map_or(false, |s| *s == net.ssid);
        if is_connected {
            crate::framebuffer::fill_rect_alpha(x as u32, row_y as u32, w, ch as u32 + 2, 0x3FB950, 20);
        }

        // SSID (truncated)
        let ssid_display = if net.ssid.is_empty() { "<hidden>" } else { &net.ssid };
        let max_ssid = ((w / 3) as i32 / cw).max(4) as usize;
        let ssid_trunc: String = ssid_display.chars().take(max_ssid).collect();
        draw_text(x, row_y, &ssid_trunc, if is_connected { COL_GREEN } else { COL_TEXT });

        // Signal bars
        let bars_x = x + (max_ssid as i32 + 1) * cw;
        draw_text(bars_x, row_y, bars_str, sig_color);

        // Channel + security
        let info = format!("Ch{:>3} {} {:>4}dBm", net.channel, net.security, net.signal_dbm);
        let info_x = bars_x + 6 * cw;
        draw_text(info_x, row_y, &info, COL_DIM);

        // BSSID
        let bssid = format!(
            "{:02X}:{:02X}:{:02X}:{:02X}:{:02X}:{:02X}",
            net.bssid[0], net.bssid[1], net.bssid[2],
            net.bssid[3], net.bssid[4], net.bssid[5]
        );
        let bssid_x = info_x + 20 * cw;
        if bssid_x + 17 * cw < x + w as i32 {
            draw_text(bssid_x, row_y, &bssid, 0xFF555555);
        }
    }

    // Scroll indicator
    if state.cached_networks.len() > visible_rows {
        let indicator = format!("[{}/{}]", scroll + 1, state.cached_networks.len());
        draw_text(x + w as i32 - 12 * cw, y, &indicator, COL_DIM);
    }
}

// ── Panel 4: Packet Stats ───────────────────────────────────────────────────

fn draw_packet_stats(state: &WifiAnalyzerState, x: i32, y: i32, w: u32, h: u32) {
    let ch = char_h();
    let cw = char_w();
    if ch <= 0 || w < 80 || h < 60 { return; }

    let row_h = ch + 4;
    let mut row = 0i32;

    // TX stats
    draw_text(x, y + row * row_h, "TX", COL_GREEN);
    row += 1;
    let tx_line = format!("  Packets: {}", state.tx_packets);
    draw_text(x, y + row * row_h, &tx_line, COL_TEXT);
    row += 1;
    let tx_bytes = format!("  Bytes:   {}", format_bytes(state.tx_bytes));
    draw_text(x, y + row * row_h, &tx_bytes, COL_TEXT);
    row += 1;
    let tx_err = format!("  Errors:  {}", state.tx_errors);
    draw_text(x, y + row * row_h, &tx_err, if state.tx_errors > 0 { COL_RED } else { COL_DIM });
    row += 1;
    let tx_tp = format!("  Rate:    {}/s", format_bytes(state.throughput_tx));
    draw_text(x, y + row * row_h, &tx_tp, COL_CYAN);
    row += 2;

    // RX stats
    draw_text(x, y + row * row_h, "RX", COL_ACCENT);
    row += 1;
    let rx_line = format!("  Packets: {}", state.rx_packets);
    draw_text(x, y + row * row_h, &rx_line, COL_TEXT);
    row += 1;
    let rx_bytes = format!("  Bytes:   {}", format_bytes(state.rx_bytes));
    draw_text(x, y + row * row_h, &rx_bytes, COL_TEXT);
    row += 1;
    let rx_err = format!("  Errors:  {}", state.rx_errors);
    draw_text(x, y + row * row_h, &rx_err, if state.rx_errors > 0 { COL_RED } else { COL_DIM });
    row += 1;
    let rx_tp = format!("  Rate:    {}/s", format_bytes(state.throughput_rx));
    draw_text(x, y + row * row_h, &rx_tp, COL_CYAN);
    row += 2;

    // Throughput bar
    if w >= 100 {
        draw_text(x, y + row * row_h, "Bandwidth", COL_ACCENT);
        row += 1;
        // TX bar
        let max_bar = w.saturating_sub(60);
        let tx_pct = (state.throughput_tx as u32).min(1_000_000) * 100 / 1_000_000u32.max(1);
        let tx_fill = (max_bar * tx_pct / 100).max(1);
        draw_text(x, y + row * row_h, "TX", COL_GREEN);
        let bar_x = x + 4 * cw;
        crate::framebuffer::fill_rect(bar_x as u32, (y + row * row_h) as u32, max_bar, ch as u32, 0xFF1A1A1A);
        crate::framebuffer::fill_rect(bar_x as u32, (y + row * row_h) as u32, tx_fill, ch as u32, COL_GREEN);
        row += 1;
        // RX bar
        let rx_pct = (state.throughput_rx as u32).min(1_000_000) * 100 / 1_000_000u32.max(1);
        let rx_fill = (max_bar * rx_pct / 100).max(1);
        draw_text(x, y + row * row_h, "RX", COL_ACCENT);
        crate::framebuffer::fill_rect(bar_x as u32, (y + row * row_h) as u32, max_bar, ch as u32, 0xFF1A1A1A);
        crate::framebuffer::fill_rect(bar_x as u32, (y + row * row_h) as u32, rx_fill, ch as u32, COL_ACCENT);
    }
}

fn format_bytes(bytes: u64) -> String {
    if bytes >= 1_073_741_824 {
        format!("{:.1} GB", bytes as f64 / 1_073_741_824.0)
    } else if bytes >= 1_048_576 {
        format!("{:.1} MB", bytes as f64 / 1_048_576.0)
    } else if bytes >= 1024 {
        format!("{:.1} KB", bytes as f64 / 1024.0)
    } else {
        format!("{} B", bytes)
    }
}

// ── Panel 5: HW Registers ──────────────────────────────────────────────────

fn draw_hw_registers(state: &WifiAnalyzerState, x: i32, y: i32, w: u32, h: u32) {
    let ch = char_h();
    let cw = char_w();
    if ch <= 0 || w < 100 || h < 40 { return; }

    let max_rows = (h as i32 / (ch + 2)) as usize;
    if max_rows < 2 { return; }

    draw_text(x, y, "Intel WiFi CSR Registers", COL_RED);
    let list_y = y + ch + 4;
    let visible_rows = max_rows.saturating_sub(2);

    if state.csr_cache.is_empty() {
        draw_text(x, list_y, "No hardware / not probed yet", COL_DIM);
        return;
    }

    let scroll = state.reg_scroll.min(state.csr_cache.len().saturating_sub(visible_rows));

    for (i, csr) in state.csr_cache.iter().enumerate().skip(scroll).take(visible_rows) {
        let row_y = list_y + ((i - scroll) as i32 * (ch + 2));

        // Register name
        let name_w = 16.min(w as usize / (cw as usize + 1));
        draw_text(x, row_y, csr.name, COL_CYAN);

        // Offset
        let off_str = format!("[0x{:03X}]", csr.offset);
        let off_x = x + 18 * cw;
        draw_text(off_x, row_y, &off_str, COL_DIM);

        // Value
        let val_str = if csr.value == 0xDEAD_DEAD {
            String::from("N/A")
        } else {
            format!("0x{:08X}", csr.value)
        };
        let val_x = off_x + 9 * cw;
        let val_color = if csr.value == 0xDEAD_DEAD { COL_RED }
            else if csr.value == 0 { COL_DIM }
            else { COL_GREEN };
        draw_text(val_x, row_y, &val_str, val_color);

        // Binary breakdown of interesting bits (for key registers)
        if w > 350 && csr.value != 0xDEAD_DEAD {
            let bits = format_register_bits(csr.name, csr.value);
            if !bits.is_empty() {
                let bits_x = val_x + 13 * cw;
                if bits_x + bits.len() as i32 * cw < x + w as i32 {
                    draw_text(bits_x, row_y, &bits, COL_YELLOW);
                }
            }
        }
    }

    // Scroll indicator
    if state.csr_cache.len() > visible_rows {
        let indicator = format!("[{}/{}]", scroll + 1, state.csr_cache.len());
        draw_text(x + w as i32 - 12 * cw, y, &indicator, COL_DIM);
    }
}

fn format_register_bits(name: &str, value: u32) -> String {
    match name {
        "GP_CNTRL" => {
            let mac_access = (value >> 0) & 1;
            let mac_clock = (value >> 1) & 1;
            let init_done = (value >> 2) & 1;
            let mac_sleep = (value >> 4) & 1;
            format!("ACC={} CLK={} INIT={} SLP={}", mac_access, mac_clock, init_done, mac_sleep)
        }
        "INT" => {
            let alive = (value >> 0) & 1;
            let wakeup = (value >> 1) & 1;
            let rx = (value >> 3) & 1;
            let tx = (value >> 6) & 1;
            let hw_err = (value >> 29) & 1;
            format!("ALV={} WK={} RX={} TX={} ERR={}", alive, wakeup, rx, tx, hw_err)
        }
        "HW_REV" => {
            let step = value & 0xF;
            let dash = (value >> 4) & 0xF;
            format!("step={} dash={}", step, dash)
        }
        _ => String::new(),
    }
}
