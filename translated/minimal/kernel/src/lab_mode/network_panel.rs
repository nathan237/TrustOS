








extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;
use alloc::format;

use crate::keyboard::{V_, U_, AM_, AQ_};


pub struct NetworkPanelState {
    
    pub jc: usize,
    
    pub acp: usize,
    
    pub ucl: u64,
}

impl NetworkPanelState {
    pub fn new() -> Self {
        Self { jc: 0, acp: 0, ucl: 0 }
    }

    pub fn vr(&mut self, bs: u8) {
        match bs {
            eh if eh == V_ => {
                self.jc = self.jc.ao(1);
            }
            eh if eh == U_ => {
                self.jc += 1;
            }
            eh if eh == AM_ => {
                self.jc = self.jc.ao(10);
            }
            eh if eh == AQ_ => {
                self.jc += 10;
            }
            b'1' => self.acp = 0,
            b'2' => self.acp = 1,
            b'3' => self.acp = 2,
            _ => {}
        }
    }

    pub fn ago(&mut self, qcm: i32, ct: i32, dxx: u32, dxv: u32) {
        let bm = super::apm();
        if bm <= 0 { return; }
        
        if ct < bm {
            
            let bj = (qcm / 80).am(0) as usize;
            if bj < 3 {
                self.acp = bj;
            }
        }
    }
}


pub fn po(g: &NetworkPanelState, b: i32, c: i32, d: u32, i: u32) {
    let bm = super::apm();
    let dpm = super::nk();
    if bm <= 0 || dpm <= 0 || d < 60 || i < 40 {
        return;
    }
    let czo = (d as i32 / dpm) as usize;
    let brh = (i as i32 / bm) as usize;
    if brh < 3 { return; }

    
    let bio = ["[1] Overview", "[2] Connections", "[3] Packets"];
    let mut gx = b;
    for (a, cu) in bio.iter().cf() {
        let s = if a == g.acp { super::O_ } else { super::F_ };
        super::kw(gx, c, cu, s);
        gx += (cu.len() as i32 + 2) * dpm;
    }

    let gl = c + bm + 4;
    let kkr = brh.ao(2);

    match g.acp {
        0 => krf(b, gl, d, kkr, bm, dpm, czo),
        1 => scj(g, b, gl, d, kkr, bm, dpm, czo),
        2 => seq(g, b, gl, d, kkr, bm, dpm, czo),
        _ => {}
    }
}

fn krf(b: i32, c: i32, dxx: u32, lk: usize, bm: i32, jxv: i32, czo: usize) {
    let mut br = 0;
    let mut x = c;

    
    let (dil, lar, djg, arl) = tdu();
    let ak: Vec<String> = alloc::vec![
        format!("Interface: virtio-net  Link: {}", if arl { "UP" } else { "DOWN" }),
        format!("IPv4: {}  GW: {}", dil, lar),
        format!("MAC:  {}", djg),
        String::new(),
        format!("TCP connections: {}", crate::netstack::tcp::rnz()),
        format!("Sniffer packets: {}",
            crate::netscan::sniffer::vao()),
        format!("Injected packets: {}",
            crate::netscan::replay::xkh()),
        format!("CSPRNG: {}", if crate::rng::tmr() { "RDRAND" } else { "SW fallback" }),
    ];

    for line in &ak {
        if br >= lk { break; }
        let display = if line.len() > czo { &line[..czo] } else { line.as_str() };
        let s = if line.is_empty() { super::F_ } else { super::T_ };
        super::kw(b, x, display, s);
        x += bm;
        br += 1;
    }
}

fn scj(g: &NetworkPanelState, b: i32, c: i32, dxx: u32, lk: usize, bm: i32, jxv: i32, czo: usize) {
    let aan = crate::netstack::tcp::ufo();
    let dh = "SRC_PORT  DST_IP:PORT         STATE";
    super::kw(b, c, dh, super::O_);

    let mut x = c + bm;
    let mut br = 0;
    for (a, co) in aan.iter().cf() {
        if a < g.jc { continue; }
        if br >= lk.ao(1) { break; }
        let display = if co.len() > czo { &co[..czo] } else { co.as_str() };
        super::kw(b, x, display, super::T_);
        x += bm;
        br += 1;
    }
    if aan.is_empty() {
        super::kw(b, x, "(no active connections)", super::F_);
    }
}

fn seq(g: &NetworkPanelState, b: i32, c: i32, dxx: u32, lk: usize, bm: i32, jxv: i32, czo: usize) {
    let bjm = crate::netscan::sniffer::kyk();
    let dh = "#    PROTO  SRC -> DST          INFO";
    super::kw(b, c, dh, super::O_);

    let mut x = c + bm;
    let mut br = 0;
    for (a, mt) in bjm.iter().cf().vv() {
        if br < g.jc { br += 1; continue; }
        if br >= lk.ao(1) + g.jc { break; }
        let line = format!("{:<4} {:?}  {}", a, mt.protocol, &mt.co);
        let display = if line.len() > czo { &line[..czo] } else { line.as_str() };
        let s = match mt.protocol {
            crate::netscan::sniffer::Protocol::Mk => super::AK_,
            crate::netscan::sniffer::Protocol::Ic => super::BB_,
            crate::netscan::sniffer::Protocol::Pq => super::AO_,
            crate::netscan::sniffer::Protocol::Vj => super::EZ_,
            _ => super::T_,
        };
        super::kw(b, x, display, s);
        x += bm;
        br += 1;
    }
    if bjm.is_empty() {
        super::kw(b, x, "(no captured packets — run 'sniff start')", super::F_);
    }
}



fn tdu() -> (String, String, String, bool) {
    let (dil, lar) = if let Some((ip, elo, nt)) = crate::network::aou() {
        let gji = ip.as_bytes();
        let tik = if let Some(at) = nt {
            let eqx = at.as_bytes();
            format!("{}.{}.{}.{}", eqx[0], eqx[1], eqx[2], eqx[3])
        } else {
            String::from("-")
        };
        (
            format!("{}.{}.{}.{}", gji[0], gji[1], gji[2], gji[3]),
            tik,
        )
    } else {
        (String::from("unconfigured"), String::from("-"))
    };

    let ed = crate::network::ckt().unwrap_or([0; 6]);
    let djg = format!("{:02X}:{:02X}:{:02X}:{:02X}:{:02X}:{:02X}",
        ed[0], ed[1], ed[2], ed[3], ed[4], ed[5]);

    let arl = crate::network::aou().is_some();
    (dil, lar, djg, arl)
}
