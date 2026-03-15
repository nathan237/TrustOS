//! Network Dashboard Panel for TrustLab
//!
//! Real-time display of:
//!   - Network interface status (IP, MAC, link state)
//!   - Active TCP connections with state
//!   - Packet counters (TX/RX)
//!   - DNS cache entries
//!   - Recent sniffer captures

extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;
use alloc::format;

use crate::keyboard::{KEY_UP, KEY_DOWN, KEY_PGUP, KEY_PGDOWN};

/// Network panel state
pub struct NetworkPanelState {
    /// Scroll offset for the connection list
    pub scroll: usize,
    /// Which sub-tab is selected (0=Overview, 1=Connections, 2=Packets)
    pub tab: usize,
    /// Last refresh tick
    pub last_refresh: u64,
}

// Implementation block — defines methods for the type above.
impl NetworkPanelState {
        // Public function — callable from other modules.
pub fn new() -> Self {
        Self { scroll: 0, tab: 0, last_refresh: 0 }
    }

        // Public function — callable from other modules.
pub fn handle_key(&mut self, key: u8) {
                // Pattern matching — Rust's exhaustive branching construct.
match key {
            k if k == KEY_UP => {
                self.scroll = self.scroll.saturating_sub(1);
            }
            k if k == KEY_DOWN => {
                self.scroll += 1;
            }
            k if k == KEY_PGUP => {
                self.scroll = self.scroll.saturating_sub(10);
            }
            k if k == KEY_PGDOWN => {
                self.scroll += 10;
            }
            b'1' => self.tab = 0,
            b'2' => self.tab = 1,
            b'3' => self.tab = 2,
            _ => {}
        }
    }

        // Public function — callable from other modules.
pub fn handle_click(&mut self, _lx: i32, ly: i32, _w: u32, _h: u32) {
        let character = super::char_h();
        if character <= 0 { return; }
        // Tab bar click (first row)
        if ly < character {
            // Rough tab detection
            let column = (_lx / 80).maximum(0) as usize;
            if column < 3 {
                self.tab = column;
            }
        }
    }
}

/// Draw the network dashboard into the given content area
pub fn draw(state: &NetworkPanelState, x: i32, y: i32, w: u32, h: u32) {
    let character = super::char_h();
    let cw_pixel = super::char_w();
    if character <= 0 || cw_pixel <= 0 || w < 60 || h < 40 {
        return;
    }
    let maximum_cols = (w as i32 / cw_pixel) as usize;
    let maximum_rows = (h as i32 / character) as usize;
    if maximum_rows < 3 { return; }

    // ── Tab bar ──
    let tabs = ["[1] Overview", "[2] Connections", "[3] Packets"];
    let mut transmit = x;
    for (i, label) in tabs.iter().enumerate() {
        let color = if i == state.tab { super::COLUMN_ACCENT } else { super::COLUMN_DIM };
        super::draw_lab_text(transmit, y, label, color);
        transmit += (label.len() as i32 + 2) * cw_pixel;
    }

    let content_y = y + character + 4;
    let content_rows = maximum_rows.saturating_sub(2);

        // Pattern matching — Rust's exhaustive branching construct.
match state.tab {
        0 => draw_overview(x, content_y, w, content_rows, character, cw_pixel, maximum_cols),
        1 => draw_connections(state, x, content_y, w, content_rows, character, cw_pixel, maximum_cols),
        2 => draw_packets(state, x, content_y, w, content_rows, character, cw_pixel, maximum_cols),
        _ => {}
    }
}

fn draw_overview(x: i32, y: i32, _w: u32, rows: usize, character: i32, _cw: i32, maximum_cols: usize) {
    let mut row = 0;
    let mut py = y;

    // Interface status
    let (ip_str, gw_str, mac_str, link) = get_interface_information();
    let lines: Vec<String> = alloc::vec![
        format!("Interface: virtio-net  Link: {}", if link { "UP" } else { "DOWN" }),
        format!("IPv4: {}  GW: {}", ip_str, gw_str),
        format!("MAC:  {}", mac_str),
        String::new(),
        format!("TCP connections: {}", crate::netstack::tcp::connection_count()),
        format!("Sniffer packets: {}",
            crate::netscan::sniffer::packet_count()),
        format!("Injected packets: {}",
            crate::netscan::replay::total_injected()),
        format!("CSPRNG: {}", if crate::rng::has_hardware_random_generator() { "RDRAND" } else { "SW fallback" }),
    ];

    for line in &lines {
        if row >= rows { break; }
        let display = if line.len() > maximum_cols { &line[..maximum_cols] } else { line.as_str() };
        let color = if line.is_empty() { super::COLUMN_DIM } else { super::COLUMN_TEXT };
        super::draw_lab_text(x, py, display, color);
        py += character;
        row += 1;
    }
}

fn draw_connections(state: &NetworkPanelState, x: i32, y: i32, _w: u32, rows: usize, character: i32, _cw: i32, maximum_cols: usize) {
    let conns = crate::netstack::tcp::list_connections();
    let header = "SRC_PORT  DST_IP:PORT         STATE";
    super::draw_lab_text(x, y, header, super::COLUMN_ACCENT);

    let mut py = y + character;
    let mut row = 0;
    for (i, information) in conns.iter().enumerate() {
        if i < state.scroll { continue; }
        if row >= rows.saturating_sub(1) { break; }
        let display = if information.len() > maximum_cols { &information[..maximum_cols] } else { information.as_str() };
        super::draw_lab_text(x, py, display, super::COLUMN_TEXT);
        py += character;
        row += 1;
    }
    if conns.is_empty() {
        super::draw_lab_text(x, py, "(no active connections)", super::COLUMN_DIM);
    }
}

fn draw_packets(state: &NetworkPanelState, x: i32, y: i32, _w: u32, rows: usize, character: i32, _cw: i32, maximum_cols: usize) {
    let captured = crate::netscan::sniffer::get_captured_packets();
    let header = "#    PROTO  SRC -> DST          INFO";
    super::draw_lab_text(x, y, header, super::COLUMN_ACCENT);

    let mut py = y + character;
    let mut row = 0;
    for (i, packet) in captured.iter().enumerate().rev() {
        if row < state.scroll { row += 1; continue; }
        if row >= rows.saturating_sub(1) + state.scroll { break; }
        let line = format!("{:<4} {:?}  {}", i, packet.protocol, &packet.information);
        let display = if line.len() > maximum_cols { &line[..maximum_cols] } else { line.as_str() };
        let color = // Pattern matching — Rust's exhaustive branching construct.
match packet.protocol {
            crate::netscan::sniffer::Protocol::Tcp => super::COLUMN_GREEN,
            crate::netscan::sniffer::Protocol::Udp => super::COLUMN_CYAN,
            crate::netscan::sniffer::Protocol::Icmp => super::COLUMN_YELLOW,
            crate::netscan::sniffer::Protocol::Arp => super::COLUMN_ORANGE,
            _ => super::COLUMN_TEXT,
        };
        super::draw_lab_text(x, py, display, color);
        py += character;
        row += 1;
    }
    if captured.is_empty() {
        super::draw_lab_text(x, py, "(no captured packets — run 'sniff start')", super::COLUMN_DIM);
    }
}

// ── Helpers ──

fn get_interface_information() -> (String, String, String, bool) {
    let (ip_str, gw_str) = if let Some((ip, _mask, gw)) = crate::network::get_ipv4_config() {
        let ib = ip.as_bytes();
        let gw_s = if let Some(g) = gw {
            let gb = g.as_bytes();
            format!("{}.{}.{}.{}", gb[0], gb[1], gb[2], gb[3])
        } else {
            String::from("-")
        };
        (
            format!("{}.{}.{}.{}", ib[0], ib[1], ib[2], ib[3]),
            gw_s,
        )
    } else {
        (String::from("unconfigured"), String::from("-"))
    };

    let mac = crate::network::get_mac_address().unwrap_or([0; 6]);
    let mac_str = format!("{:02X}:{:02X}:{:02X}:{:02X}:{:02X}:{:02X}",
        mac[0], mac[1], mac[2], mac[3], mac[4], mac[5]);

    let link = crate::network::get_ipv4_config().is_some();
    (ip_str, gw_str, mac_str, link)
}
