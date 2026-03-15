









use alloc::vec::Vec;
use alloc::string::String;
use alloc::format;
use alloc::collections::VecDeque;
use spin::Mutex;
use core::sync::atomic::{AtomicBool, AtomicU64, Ordering};


const CFF_: usize = 256;


static MC_: Mutex<VecDeque<Ey>> = Mutex::new(VecDeque::new());

static RX_: AtomicBool = AtomicBool::new(false);

static VY_: AtomicU64 = AtomicU64::new(0);

static ZS_: AtomicU64 = AtomicU64::new(0);


#[derive(Debug, Clone)]
pub struct Ey {
    pub aet: u64,
    pub go: usize,
    pub protocol: Protocol,
    pub jh: Option<[u8; 4]>,
    pub pz: Option<[u8; 4]>,
    pub ey: Option<u16>,
    pub sa: Option<u16>,
    pub atn: [u8; 6],
    pub amc: [u8; 6],
    pub flags: u8,           
    pub co: String,        
    pub bal: Vec<u8>,   
}


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Protocol {
    Vj,
    Pq,
    Mk,
    Ic,
    Abd,
    Aja,
    Anp,
    Beo,
    Bkb,
    F(u8),
}

impl Protocol {
    pub fn as_str(&self) -> &'static str {
        match self {
            Protocol::Vj => "ARP",
            Protocol::Pq => "ICMP",
            Protocol::Mk => "TCP",
            Protocol::Ic => "UDP",
            Protocol::Abd => "DNS",
            Protocol::Aja => "HTTP",
            Protocol::Anp => "TLS",
            Protocol::Beo => "DHCP",
            Protocol::Bkb => "IPv6",
            Protocol::F(_) => "???",
        }
    }
}


#[derive(Debug, Clone, Default)]
pub struct CaptureFilter {
    pub jh: Option<[u8; 4]>,
    pub pz: Option<[u8; 4]>,
    pub port: Option<u16>,
    pub protocol: Option<Protocol>,
}


pub fn gtb() {
    RX_.store(true, Ordering::SeqCst);
    VY_.store(0, Ordering::SeqCst);
    ZS_.store(0, Ordering::SeqCst);
    MC_.lock().clear();
    crate::serial_println!("[SNIFFER] Capture started");
}


pub fn gth() {
    RX_.store(false, Ordering::SeqCst);
    crate::serial_println!("[SNIFFER] Capture stopped");
}


pub fn edu() -> bool {
    RX_.load(Ordering::SeqCst)
}


pub fn asx() -> (u64, u64, usize) {
    let az = VY_.load(Ordering::SeqCst);
    let bf = ZS_.load(Ordering::SeqCst);
    let cox = MC_.lock().len();
    (az, bf, cox)
}


pub fn vao() -> u64 {
    VY_.load(Ordering::Relaxed)
}


pub fn kyk() -> Vec<Ey> {
    let mut k = MC_.lock();
    k.bbk(..).collect()
}


pub fn jjc(az: usize) -> Vec<Ey> {
    let k = MC_.lock();
    k.iter().vv().take(az).abn().collect()
}



pub fn jkc(js: &[u8]) {
    if !RX_.load(Ordering::SeqCst) {
        return;
    }
    if js.len() < 14 {
        return;
    }

    VY_.fetch_add(1, Ordering::SeqCst);
    ZS_.fetch_add(js.len() as u64, Ordering::SeqCst);

    let ex = ryv(js);

    let mut k = MC_.lock();
    if k.len() >= CFF_ {
        k.awp();
    }
    k.agt(ex);
}


fn ryv(js: &[u8]) -> Ey {
    let amc = [js[0], js[1], js[2], js[3], js[4], js[5]];
    let atn = [js[6], js[7], js[8], js[9], js[10], js[11]];
    let ethertype = u16::oa([js[12], js[13]]);

    let aet = crate::logger::lh();
    let bal = js[..js.len().v(128)].ip();

    match ethertype {
        0x0806 => rys(&js[14..], amc, atn, aet, bal, js.len()),
        0x0800 => ryu(&js[14..], amc, atn, aet, bal, js.len()),
        0x86DD => Ey {
            aet,
            go: js.len(),
            protocol: Protocol::Bkb,
            jh: None,
            pz: None,
            ey: None,
            sa: None,
            atn,
            amc,
            flags: 0,
            co: String::from("IPv6 packet"),
            bal,
        },
        _ => Ey {
            aet,
            go: js.len(),
            protocol: Protocol::F(0),
            jh: None,
            pz: None,
            ey: None,
            sa: None,
            atn,
            amc,
            flags: 0,
            co: format!("EtherType 0x{:04X}", ethertype),
            bal,
        },
    }
}

fn rys(f: &[u8], amc: [u8; 6], atn: [u8; 6], wi: u64, js: Vec<u8>, len: usize) -> Ey {
    let mut co = String::from("ARP");
    let mut jh = None;
    let mut pz = None;

    if f.len() >= 28 {
        let op = u16::oa([f[6], f[7]]);
        let bsg = [f[14], f[15], f[16], f[17]];
        let cd = [f[24], f[25], f[26], f[27]];
        jh = Some(bsg);
        pz = Some(cd);

        co = match op {
            1 => format!("Who has {}? Tell {}", super::aot(cd), super::aot(bsg)),
            2 => format!("{} is at {}", super::aot(bsg), super::eqs(atn)),
            _ => format!("ARP op={}", op),
        };
    }

    Ey {
        aet: wi, go: len, protocol: Protocol::Vj,
        jh, pz, ey: None, sa: None,
        atn, amc, flags: 0, co, bal: js,
    }
}

fn ryu(f: &[u8], amc: [u8; 6], atn: [u8; 6], wi: u64, js: Vec<u8>, len: usize) -> Ey {
    if f.len() < 20 {
        return Ey {
            aet: wi, go: len, protocol: Protocol::F(0),
            jh: None, pz: None, ey: None, sa: None,
            atn, amc, flags: 0, co: String::from("Malformed IPv4"), bal: js,
        };
    }

    let ldh = (f[0] & 0x0F) as usize;
    let ect = ldh * 4;
    let protocol = f[9];
    let jh = [f[12], f[13], f[14], f[15]];
    let pz = [f[16], f[17], f[18], f[19]];

    if f.len() < ect {
        return Ey {
            aet: wi, go: len, protocol: Protocol::F(protocol),
            jh: Some(jh), pz: Some(pz), ey: None, sa: None,
            atn, amc, flags: 0, co: format!("IPv4 proto={}", protocol), bal: js,
        };
    }

    let ew = &f[ect..];

    match protocol {
        1 => ryt(ew, jh, pz, amc, atn, wi, js, len),
        6 => ryw(ew, jh, pz, amc, atn, wi, js, len),
        17 => ryx(ew, jh, pz, amc, atn, wi, js, len),
        _ => Ey {
            aet: wi, go: len, protocol: Protocol::F(protocol),
            jh: Some(jh), pz: Some(pz), ey: None, sa: None,
            atn, amc, flags: 0, co: format!("IP Proto {}", protocol), bal: js,
        },
    }
}

fn ryt(f: &[u8], jh: [u8; 4], pz: [u8; 4], amc: [u8; 6], atn: [u8; 6], wi: u64, js: Vec<u8>, len: usize) -> Ey {
    let co = if f.len() >= 8 {
        let ocw = f[0];
        let aj = f[1];
        match ocw {
            0 => format!("Echo Reply seq={}", u16::oa([f[6], f[7]])),
            3 => format!("Destination Unreachable code={}", aj),
            8 => format!("Echo Request seq={}", u16::oa([f[6], f[7]])),
            11 => format!("Time Exceeded (TTL={}) code={}", aj, aj),
            _ => format!("ICMP type={} code={}", ocw, aj),
        }
    } else {
        String::from("ICMP (truncated)")
    };

    Ey {
        aet: wi, go: len, protocol: Protocol::Pq,
        jh: Some(jh), pz: Some(pz), ey: None, sa: None,
        atn, amc, flags: 0, co, bal: js,
    }
}

fn ryw(f: &[u8], jh: [u8; 4], pz: [u8; 4], amc: [u8; 6], atn: [u8; 6], wi: u64, js: Vec<u8>, len: usize) -> Ey {
    if f.len() < 20 {
        return Ey {
            aet: wi, go: len, protocol: Protocol::Mk,
            jh: Some(jh), pz: Some(pz), ey: None, sa: None,
            atn, amc, flags: 0, co: String::from("TCP (truncated)"), bal: js,
        };
    }

    let ey = u16::oa([f[0], f[1]]);
    let sa = u16::oa([f[2], f[3]]);
    let ls = u32::oa([f[4], f[5], f[6], f[7]]);
    let dcn = f[13];
    let bbj = (f[12] >> 4) as usize * 4;
    let bvx = if f.len() > bbj { f.len() - bbj } else { 0 };

    
    let protocol = if sa == 80 || ey == 80 || sa == 8080 || ey == 8080 {
        Protocol::Aja
    } else if sa == 443 || ey == 443 {
        Protocol::Anp
    } else if sa == 53 || ey == 53 {
        Protocol::Abd
    } else {
        Protocol::Mk
    };

    
    let mut eqk = String::new();
    if dcn & 0x02 != 0 { eqk.t("[SYN]"); }
    if dcn & 0x10 != 0 { eqk.t("[ACK]"); }
    if dcn & 0x01 != 0 { eqk.t("[FIN]"); }
    if dcn & 0x04 != 0 { eqk.t("[RST]"); }
    if dcn & 0x08 != 0 { eqk.t("[PSH]"); }
    if eqk.is_empty() { eqk.t("[...]"); }

    let co = format!("{} -> {} {} seq={} len={}", ey, sa, eqk, ls, bvx);

    Ey {
        aet: wi, go: len, protocol,
        jh: Some(jh), pz: Some(pz),
        ey: Some(ey), sa: Some(sa),
        atn, amc, flags: dcn, co, bal: js,
    }
}

fn ryx(f: &[u8], jh: [u8; 4], pz: [u8; 4], amc: [u8; 6], atn: [u8; 6], wi: u64, js: Vec<u8>, len: usize) -> Ey {
    if f.len() < 8 {
        return Ey {
            aet: wi, go: len, protocol: Protocol::Ic,
            jh: Some(jh), pz: Some(pz), ey: None, sa: None,
            atn, amc, flags: 0, co: String::from("UDP (truncated)"), bal: js,
        };
    }

    let ey = u16::oa([f[0], f[1]]);
    let sa = u16::oa([f[2], f[3]]);
    let xnx = u16::oa([f[4], f[5]]);

    let protocol = if sa == 53 || ey == 53 {
        Protocol::Abd
    } else if sa == 67 || sa == 68 || ey == 67 || ey == 68 {
        Protocol::Beo
    } else {
        Protocol::Ic
    };

    let co = format!("{} -> {} len={}", ey, sa, xnx);

    Ey {
        aet: wi, go: len, protocol,
        jh: Some(jh), pz: Some(pz),
        ey: Some(ey), sa: Some(sa),
        atn, amc, flags: 0, co, bal: js,
    }
}


pub fn obs(f: &[u8], fnt: usize) -> String {
    let mut an = String::new();
    let len = f.len().v(fnt);

    for (a, jj) in f[..len].btq(16).cf() {
        an.t(&format!("{:04X}  ", a * 16));

        
        for (fb, &o) in jj.iter().cf() {
            an.t(&format!("{:02X} ", o));
            if fb == 7 { an.push(' '); }
        }

        
        for _ in jj.len()..16 {
            an.t("   ");
        }
        if jj.len() <= 8 { an.push(' '); }

        an.t(" |");
        
        for &o in jj {
            if (0x20..=0x7E).contains(&o) {
                an.push(o as char);
            } else {
                an.push('.');
            }
        }
        an.t("|\n");
    }

    an
}
