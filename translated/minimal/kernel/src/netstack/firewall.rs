













use alloc::string::String;
use alloc::vec::Vec;
use alloc::format;
use core::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use spin::Mutex;






#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Chain {
    Jp,
    Dd,
    Abv,
}

impl Chain {
    pub fn j(&self) -> &'static str {
        match self {
            Chain::Jp => "INPUT",
            Chain::Dd => "OUTPUT",
            Chain::Abv => "FORWARD",
        }
    }

    pub fn cko(e: &str) -> Option<Self> {
        match e.idx().as_str() {
            "INPUT" => Some(Chain::Jp),
            "OUTPUT" => Some(Chain::Dd),
            "FORWARD" => Some(Chain::Abv),
            _ => None,
        }
    }
}


#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Action {
    Ld,
    Drop,
    Aly,
    Nl, 
}

impl Action {
    pub fn j(&self) -> &'static str {
        match self {
            Action::Ld => "ACCEPT",
            Action::Drop => "DROP",
            Action::Aly => "REJECT",
            Action::Nl => "LOG",
        }
    }

    pub fn cko(e: &str) -> Option<Self> {
        match e.idx().as_str() {
            "ACCEPT" => Some(Action::Ld),
            "DROP" => Some(Action::Drop),
            "REJECT" => Some(Action::Aly),
            "LOG" => Some(Action::Nl),
            _ => None,
        }
    }
}


#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Protocol {
    Eb,
    Mk,
    Ic,
    Pq,
}

impl Protocol {
    pub fn j(&self) -> &'static str {
        match self {
            Protocol::Eb => "all",
            Protocol::Mk => "tcp",
            Protocol::Ic => "udp",
            Protocol::Pq => "icmp",
        }
    }

    pub fn cko(e: &str) -> Option<Self> {
        match e.aqn().as_str() {
            "all" | "any" | "*" => Some(Protocol::Eb),
            "tcp" => Some(Protocol::Mk),
            "udp" => Some(Protocol::Ic),
            "icmp" => Some(Protocol::Pq),
            _ => None,
        }
    }

    pub fn aqb(&self) -> Option<u8> {
        match self {
            Protocol::Eb => None,
            Protocol::Pq => Some(1),
            Protocol::Mk => Some(6),
            Protocol::Ic => Some(17),
        }
    }
}


#[derive(Debug, Clone, Copy, PartialEq)]
pub enum IpMatch {
    Eb,
    Ho([u8; 4]),
    Azm([u8; 4], u8), 
}

impl IpMatch {
    pub fn oh(&self, ip: [u8; 4]) -> bool {
        match self {
            IpMatch::Eb => true,
            IpMatch::Ho(ag) => *ag == ip,
            IpMatch::Azm(ag, adx) => {
                if *adx == 0 {
                    return true;
                }
                if *adx >= 32 {
                    return *ag == ip;
                }
                let hs = !0u32 << (32 - adx);
                let q = u32::oa(*ag) & hs;
                let o = u32::oa(ip) & hs;
                q == o
            }
        }
    }

    pub fn parse(e: &str) -> Option<Self> {
        if e == "0.0.0.0/0" || e == "any" || e == "*" {
            return Some(IpMatch::Eb);
        }
        if let Some((elz, vkr)) = e.fve('/') {
            let ag = cgl(elz)?;
            let adx: u8 = vkr.parse().bq()?;
            if adx > 32 {
                return None;
            }
            Some(IpMatch::Azm(ag, adx))
        } else {
            let ag = cgl(e)?;
            Some(IpMatch::Ho(ag))
        }
    }

    pub fn display(&self) -> String {
        match self {
            IpMatch::Eb => String::from("0.0.0.0/0"),
            IpMatch::Ho(q) => format!("{}.{}.{}.{}", q[0], q[1], q[2], q[3]),
            IpMatch::Azm(q, ai) => format!("{}.{}.{}.{}/{}", q[0], q[1], q[2], q[3], ai),
        }
    }
}


#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PortMatch {
    Eb,
    Ho(u16),
    Nt(u16, u16),
}

impl PortMatch {
    pub fn oh(&self, port: u16) -> bool {
        match self {
            PortMatch::Eb => true,
            PortMatch::Ho(ai) => *ai == port,
            PortMatch::Nt(hh, gd) => port >= *hh && port <= *gd,
        }
    }

    pub fn parse(e: &str) -> Option<Self> {
        if e == "any" || e == "*" || e == "0" {
            return Some(PortMatch::Eb);
        }
        if let Some((ugi, too)) = e.fve(':') {
            let hh: u16 = ugi.parse().bq()?;
            let gd: u16 = too.parse().bq()?;
            Some(PortMatch::Nt(hh, gd))
        } else {
            let ai: u16 = e.parse().bq()?;
            Some(PortMatch::Ho(ai))
        }
    }

    pub fn display(&self) -> String {
        match self {
            PortMatch::Eb => String::from("*"),
            PortMatch::Ho(ai) => format!("{}", ai),
            PortMatch::Nt(hh, gd) => format!("{}:{}", hh, gd),
        }
    }
}


#[derive(Debug, Clone)]
pub struct Rule {
    pub rh: Chain,
    pub protocol: Protocol,
    pub jh: IpMatch,
    pub pz: IpMatch,
    pub ey: PortMatch,
    pub sa: PortMatch,
    pub hr: Action,
    pub byv: String,
    
    pub egb: u64,
    
    pub bf: u64,
}

impl Rule {
    pub fn new(rh: Chain, hr: Action) -> Self {
        Self {
            rh,
            protocol: Protocol::Eb,
            jh: IpMatch::Eb,
            pz: IpMatch::Eb,
            ey: PortMatch::Eb,
            sa: PortMatch::Eb,
            hr,
            byv: String::new(),
            egb: 0,
            bf: 0,
        }
    }

    
    pub fn oh(&self, cgv: u8, cy: [u8; 4], cs: [u8; 4], bom: u16, bmx: u16) -> bool {
        
        if let Some(ai) = self.protocol.aqb() {
            if ai != cgv {
                return false;
            }
        }
        
        if !self.jh.oh(cy) {
            return false;
        }
        if !self.pz.oh(cs) {
            return false;
        }
        
        if cgv == 6 || cgv == 17 {
            if !self.ey.oh(bom) {
                return false;
            }
            if !self.sa.oh(bmx) {
                return false;
            }
        }
        true
    }
}





struct Asl {
    bib: Vec<Rule>,
    jai: Action,
    jid: Action,
    ivl: Action,
    fnf: Vec<String>,
}

impl Asl {
    fn new() -> Self {
        Self {
            bib: Vec::new(),
            jai: Action::Ld,
            jid: Action::Ld,
            ivl: Action::Drop,
            fnf: Vec::new(),
        }
    }

    fn iwu(&self, rh: Chain) -> Action {
        match rh {
            Chain::Jp => self.jai,
            Chain::Dd => self.jid,
            Chain::Abv => self.ivl,
        }
    }

    fn met(&mut self, rh: Chain, hr: Action) {
        match rh {
            Chain::Jp => self.jai = hr,
            Chain::Dd => self.jid = hr,
            Chain::Abv => self.ivl = hr,
        }
    }

    fn qfk(&mut self, bt: String) {
        if self.fnf.len() >= 256 {
            self.fnf.remove(0);
        }
        self.fnf.push(bt);
    }
}

static Kj: Mutex<Asl> = Mutex::new(Asl {
    bib: Vec::new(),
    jai: Action::Ld,
    jid: Action::Ld,
    ivl: Action::Drop,
    fnf: Vec::new(),
});

static Li: AtomicBool = AtomicBool::new(false);
static OQ_: AtomicU64 = AtomicU64::new(0);
static OR_: AtomicU64 = AtomicU64::new(0);






pub fn zu() -> bool {
    Li.load(Ordering::Relaxed)
}


pub fn cuf(iq: bool) {
    Li.store(iq, Ordering::Release);
}


pub fn ssp(cgv: u8, cy: [u8; 4], cs: [u8; 4], bom: u16, bmx: u16, duk: usize) -> bool {
    if !zu() {
        return true;
    }
    ntv(Chain::Jp, cgv, cy, cs, bom, bmx, duk)
}


pub fn ntw(cgv: u8, cy: [u8; 4], cs: [u8; 4], bom: u16, bmx: u16, duk: usize) -> bool {
    if !zu() {
        return true;
    }
    ntv(Chain::Dd, cgv, cy, cs, bom, bmx, duk)
}


fn ntv(rh: Chain, cgv: u8, cy: [u8; 4], cs: [u8; 4], bom: u16, bmx: u16, duk: usize) -> bool {
    let mut ua = Kj.lock();

    
    for agu in ua.bib.el() {
        if agu.rh != rh {
            continue;
        }
        if agu.oh(cgv, cy, cs, bom, bmx) {
            agu.egb += 1;
            agu.bf += duk as u64;

            match agu.hr {
                Action::Ld => {
                    OQ_.fetch_add(1, Ordering::Relaxed);
                    return true;
                }
                Action::Drop => {
                    OR_.fetch_add(1, Ordering::Relaxed);
                    return false;
                }
                Action::Aly => {
                    OR_.fetch_add(1, Ordering::Relaxed);
                    whl(cgv, cy, cs, bom, bmx);
                    return false;
                }
                Action::Nl => {
                    let vnl = match cgv {
                        1 => "ICMP",
                        6 => "TCP",
                        17 => "UDP",
                        _ => "???",
                    };
                    let bt = format!(
                        "[FW {}] {} {}.{}.{}.{}:{} -> {}.{}.{}.{}:{} len={}",
                        rh.j(), vnl,
                        cy[0], cy[1], cy[2], cy[3], bom,
                        cs[0], cs[1], cs[2], cs[3], bmx,
                        duk,
                    );
                    crate::serial_println!("{}", bt);
                    ua.qfk(bt);
                    OQ_.fetch_add(1, Ordering::Relaxed);
                    return true; 
                }
            }
        }
    }

    
    let policy = ua.iwu(rh);
    match policy {
        Action::Ld | Action::Nl => {
            OQ_.fetch_add(1, Ordering::Relaxed);
            true
        }
        Action::Drop | Action::Aly => {
            OR_.fetch_add(1, Ordering::Relaxed);
            false
        }
    }
}






pub fn qfo(agu: Rule) {
    Kj.lock().bib.push(agu);
}


pub fn yyf(index: usize, agu: Rule) {
    let mut ua = Kj.lock();
    if index <= ua.bib.len() {
        ua.bib.insert(index, agu);
    }
}


pub fn rvj(rh: Chain, index: usize) -> bool {
    let mut ua = Kj.lock();
    let mut nch = 0usize;
    let mut pak = None;
    for (a, agu) in ua.bib.iter().cf() {
        if agu.rh == rh {
            if nch == index {
                pak = Some(a);
                break;
            }
            nch += 1;
        }
    }
    if let Some(a) = pak {
        ua.bib.remove(a);
        true
    } else {
        false
    }
}


pub fn hjx(rh: Option<Chain>) {
    let mut ua = Kj.lock();
    match rh {
        Some(r) => ua.bib.ajm(|m| m.rh != r),
        None => ua.bib.clear(),
    }
}


pub fn met(rh: Chain, hr: Action) {
    Kj.lock().met(rh, hr);
}


pub fn ufv(rh: Chain) -> Vec<Rule> {
    Kj.lock().bib.iter().hi(|m| m.rh == rh).abn().collect()
}


pub fn iwu(rh: Chain) -> Action {
    Kj.lock().iwu(rh)
}


pub fn cm() -> (u64, u64) {
    (OQ_.load(Ordering::Relaxed), OR_.load(Ordering::Relaxed))
}


pub fn tdx() -> Vec<String> {
    Kj.lock().fnf.clone()
}


pub fn rbf() {
    Kj.lock().fnf.clear();
}


pub fn pcq() {
    OQ_.store(0, Ordering::Relaxed);
    OR_.store(0, Ordering::Relaxed);
    let mut ua = Kj.lock();
    for agu in ua.bib.el() {
        agu.egb = 0;
        agu.bf = 0;
    }
}





fn cgl(e: &str) -> Option<[u8; 4]> {
    let ek: Vec<&str> = e.adk('.').collect();
    if ek.len() != 4 {
        return None;
    }
    let q: u8 = ek[0].parse().bq()?;
    let o: u8 = ek[1].parse().bq()?;
    let r: u8 = ek[2].parse().bq()?;
    let bc: u8 = ek[3].parse().bq()?;
    Some([q, o, r, bc])
}


fn whl(cgv: u8, cy: [u8; 4], cs: [u8; 4], bom: u16, bmx: u16) {
    if cgv == 6 {
        
        whq(cy, cs, bom, bmx);
    } else {
        
        whf(cy, cs);
    }
}


fn whq(ams: [u8; 4], uht: [u8; 4], bci: u16, ahq: u16) {
    
    let mut pk = [0u8; 20];
    
    pk[0..2].dg(&ahq.ft());
    
    pk[2..4].dg(&bci.ft());
    
    
    
    pk[12] = 0x50; 
    pk[13] = 0x14; 
    
    
    

    
    let td = super::tcp::psc(uht, ams, &pk);
    pk[16..18].dg(&td.ft());

    let _ = super::ip::blc(ams, 6, &pk);
}


fn whf(ams: [u8; 4], yao: [u8; 4]) {
    
    let mut mt = [0u8; 8];
    mt[0] = 3;  
    mt[1] = 13; 
    
    

    
    let mut sum: u32 = 0;
    for a in (0..mt.len()).akt(2) {
        let od = if a + 1 < mt.len() {
            ((mt[a] as u16) << 8) | (mt[a + 1] as u16)
        } else {
            (mt[a] as u16) << 8
        };
        sum += od as u32;
    }
    while sum >> 16 != 0 {
        sum = (sum & 0xFFFF) + (sum >> 16);
    }
    let td = !(sum as u16);
    mt[2..4].dg(&td.ft());

    let _ = super::ip::blc(ams, 1, &mt);
}
