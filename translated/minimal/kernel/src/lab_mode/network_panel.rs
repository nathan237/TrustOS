








extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;
use alloc::format;

use crate::keyboard::{T_, S_, AM_, AO_};


pub struct NetworkPanelState {
    
    pub scroll: usize,
    
    pub tab: usize,
    
    pub last_refresh: u64,
}

impl NetworkPanelState {
    pub fn new() -> Self {
        Self { scroll: 0, tab: 0, last_refresh: 0 }
    }

    pub fn handle_key(&mut self, key: u8) {
        match key {
            k if k == T_ => {
                self.scroll = self.scroll.saturating_sub(1);
            }
            k if k == S_ => {
                self.scroll += 1;
            }
            k if k == AM_ => {
                self.scroll = self.scroll.saturating_sub(10);
            }
            k if k == AO_ => {
                self.scroll += 10;
            }
            b'1' => self.tab = 0,
            b'2' => self.tab = 1,
            b'3' => self.tab = 2,
            _ => {}
        }
    }

    pub fn handle_click(&mut self, _lx: i32, ly: i32, _w: u32, _h: u32) {
        let ch = super::qu();
        if ch <= 0 { return; }
        
        if ly < ch {
            
            let col = (_lx / 80).max(0) as usize;
            if col < 3 {
                self.tab = col;
            }
        }
    }
}


pub fn draw(state: &NetworkPanelState, x: i32, y: i32, w: u32, h: u32) {
    let ch = super::qu();
    let blm = super::ew();
    if ch <= 0 || blm <= 0 || w < 60 || h < 40 {
        return;
    }
    let bco = (w as i32 / blm) as usize;
    let xw = (h as i32 / ch) as usize;
    if xw < 3 { return; }

    
    let tabs = ["[1] Overview", "[2] Connections", "[3] Packets"];
    let mut bu = x;
    for (i, label) in tabs.iter().enumerate() {
        let color = if i == state.tab { super::M_ } else { super::F_ };
        super::eh(bu, y, label, color);
        bu += (label.len() as i32 + 2) * blm;
    }

    let bn = y + ch + 4;
    let foi = xw.saturating_sub(2);

    match state.tab {
        0 => dnq(x, bn, w, foi, ch, blm, bco),
        1 => lij(state, x, bn, w, foi, ch, blm, bco),
        2 => lkc(state, x, bn, w, foi, ch, blm, bco),
        _ => {}
    }
}

fn dnq(x: i32, y: i32, _w: u32, rows: usize, ch: i32, _cw: i32, bco: usize) {
    let mut row = 0;
    let mut o = y;

    
    let (auc, gw_str, bhv, link) = mdg();
    let lines: Vec<String> = alloc::vec![
        format!("Interface: virtio-net  Link: {}", if link { "UP" } else { "DOWN" }),
        format!("IPv4: {}  GW: {}", auc, gw_str),
        format!("MAC:  {}", bhv),
        String::new(),
        format!("TCP connections: {}", crate::netstack::tcp::kxd()),
        format!("Sniffer packets: {}",
            crate::netscan::sniffer::npc()),
        format!("Injected packets: {}",
            crate::netscan::replay::plz()),
        format!("CSPRNG: {}", if crate::rng::mjq() { "RDRAND" } else { "SW fallback" }),
    ];

    for line in &lines {
        if row >= rows { break; }
        let display = if line.len() > bco { &line[..bco] } else { line.as_str() };
        let color = if line.is_empty() { super::F_ } else { super::P_ };
        super::eh(x, o, display, color);
        o += ch;
        row += 1;
    }
}

fn lij(state: &NetworkPanelState, x: i32, y: i32, _w: u32, rows: usize, ch: i32, _cw: i32, bco: usize) {
    let nc = crate::netstack::tcp::mza();
    let header = "SRC_PORT  DST_IP:PORT         STATE";
    super::eh(x, y, header, super::M_);

    let mut o = y + ch;
    let mut row = 0;
    for (i, info) in nc.iter().enumerate() {
        if i < state.scroll { continue; }
        if row >= rows.saturating_sub(1) { break; }
        let display = if info.len() > bco { &info[..bco] } else { info.as_str() };
        super::eh(x, o, display, super::P_);
        o += ch;
        row += 1;
    }
    if nc.is_empty() {
        super::eh(x, o, "(no active connections)", super::F_);
    }
}

fn lkc(state: &NetworkPanelState, x: i32, y: i32, _w: u32, rows: usize, ch: i32, _cw: i32, bco: usize) {
    let captured = crate::netscan::sniffer::fyk();
    let header = "#    PROTO  SRC -> DST          INFO";
    super::eh(x, y, header, super::M_);

    let mut o = y + ch;
    let mut row = 0;
    for (i, fj) in captured.iter().enumerate().rev() {
        if row < state.scroll { row += 1; continue; }
        if row >= rows.saturating_sub(1) + state.scroll { break; }
        let line = format!("{:<4} {:?}  {}", i, fj.protocol, &fj.info);
        let display = if line.len() > bco { &line[..bco] } else { line.as_str() };
        let color = match fj.protocol {
            crate::netscan::sniffer::Protocol::Tcp => super::AC_,
            crate::netscan::sniffer::Protocol::Udp => super::AU_,
            crate::netscan::sniffer::Protocol::Icmp => super::AK_,
            crate::netscan::sniffer::Protocol::Arp => super::DN_,
            _ => super::P_,
        };
        super::eh(x, o, display, color);
        o += ch;
        row += 1;
    }
    if captured.is_empty() {
        super::eh(x, o, "(no captured packets — run 'sniff start')", super::F_);
    }
}



fn mdg() -> (String, String, String, bool) {
    let (auc, gw_str) = if let Some((ip, _mask, fz)) = crate::network::rd() {
        let czj = ip.as_bytes();
        let mgt = if let Some(g) = fz {
            let cab = g.as_bytes();
            format!("{}.{}.{}.{}", cab[0], cab[1], cab[2], cab[3])
        } else {
            String::from("-")
        };
        (
            format!("{}.{}.{}.{}", czj[0], czj[1], czj[2], czj[3]),
            mgt,
        )
    } else {
        (String::from("unconfigured"), String::from("-"))
    };

    let mac = crate::network::aqu().unwrap_or([0; 6]);
    let bhv = format!("{:02X}:{:02X}:{:02X}:{:02X}:{:02X}:{:02X}",
        mac[0], mac[1], mac[2], mac[3], mac[4], mac[5]);

    let link = crate::network::rd().is_some();
    (auc, gw_str, bhv, link)
}
