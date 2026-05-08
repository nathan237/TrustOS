













use alloc::string::String;
use alloc::vec::Vec;
use alloc::format;
use core::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use spin::Mutex;






#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Chain {
    Input,
    Output,
    Forward,
}

impl Chain {
    pub fn name(&self) -> &'static str {
        match self {
            Chain::Input => "INPUT",
            Chain::Output => "OUTPUT",
            Chain::Forward => "FORWARD",
        }
    }

    pub fn atv(j: &str) -> Option<Self> {
        match j.to_uppercase().as_str() {
            "INPUT" => Some(Chain::Input),
            "OUTPUT" => Some(Chain::Output),
            "FORWARD" => Some(Chain::Forward),
            _ => None,
        }
    }
}


#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Action {
    Accept,
    Drop,
    Reject,
    Log, 
}

impl Action {
    pub fn name(&self) -> &'static str {
        match self {
            Action::Accept => "ACCEPT",
            Action::Drop => "DROP",
            Action::Reject => "REJECT",
            Action::Log => "LOG",
        }
    }

    pub fn atv(j: &str) -> Option<Self> {
        match j.to_uppercase().as_str() {
            "ACCEPT" => Some(Action::Accept),
            "DROP" => Some(Action::Drop),
            "REJECT" => Some(Action::Reject),
            "LOG" => Some(Action::Log),
            _ => None,
        }
    }
}


#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Protocol {
    Any,
    Tcp,
    Udp,
    Icmp,
}

impl Protocol {
    pub fn name(&self) -> &'static str {
        match self {
            Protocol::Any => "all",
            Protocol::Tcp => "tcp",
            Protocol::Udp => "udp",
            Protocol::Icmp => "icmp",
        }
    }

    pub fn atv(j: &str) -> Option<Self> {
        match j.to_lowercase().as_str() {
            "all" | "any" | "*" => Some(Protocol::Any),
            "tcp" => Some(Protocol::Tcp),
            "udp" => Some(Protocol::Udp),
            "icmp" => Some(Protocol::Icmp),
            _ => None,
        }
    }

    pub fn number(&self) -> Option<u8> {
        match self {
            Protocol::Any => None,
            Protocol::Icmp => Some(1),
            Protocol::Tcp => Some(6),
            Protocol::Udp => Some(17),
        }
    }
}


#[derive(Debug, Clone, Copy, PartialEq)]
pub enum IpMatch {
    Any,
    Exact([u8; 4]),
    Subnet([u8; 4], u8), 
}

impl IpMatch {
    pub fn matches(&self, ip: [u8; 4]) -> bool {
        match self {
            IpMatch::Any => true,
            IpMatch::Exact(addr) => *addr == ip,
            IpMatch::Subnet(addr, nm) => {
                if *nm == 0 {
                    return true;
                }
                if *nm >= 32 {
                    return *addr == ip;
                }
                let mask = !0u32 << (32 - nm);
                let a = u32::from_be_bytes(*addr) & mask;
                let b = u32::from_be_bytes(ip) & mask;
                a == b
            }
        }
    }

    pub fn parse(j: &str) -> Option<Self> {
        if j == "0.0.0.0/0" || j == "any" || j == "*" {
            return Some(IpMatch::Any);
        }
        if let Some((bkp, prefix_str)) = j.split_once('/') {
            let addr = art(bkp)?;
            let nm: u8 = prefix_str.parse().ok()?;
            if nm > 32 {
                return None;
            }
            Some(IpMatch::Subnet(addr, nm))
        } else {
            let addr = art(j)?;
            Some(IpMatch::Exact(addr))
        }
    }

    pub fn display(&self) -> String {
        match self {
            IpMatch::Any => String::from("0.0.0.0/0"),
            IpMatch::Exact(a) => format!("{}.{}.{}.{}", a[0], a[1], a[2], a[3]),
            IpMatch::Subnet(a, aa) => format!("{}.{}.{}.{}/{}", a[0], a[1], a[2], a[3], aa),
        }
    }
}


#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PortMatch {
    Any,
    Exact(u16),
    Range(u16, u16),
}

impl PortMatch {
    pub fn matches(&self, port: u16) -> bool {
        match self {
            PortMatch::Any => true,
            PortMatch::Exact(aa) => *aa == port,
            PortMatch::Range(lo, hi) => port >= *lo && port <= *hi,
        }
    }

    pub fn parse(j: &str) -> Option<Self> {
        if j == "any" || j == "*" || j == "0" {
            return Some(PortMatch::Any);
        }
        if let Some((lo_str, hi_str)) = j.split_once(':') {
            let lo: u16 = lo_str.parse().ok()?;
            let hi: u16 = hi_str.parse().ok()?;
            Some(PortMatch::Range(lo, hi))
        } else {
            let aa: u16 = j.parse().ok()?;
            Some(PortMatch::Exact(aa))
        }
    }

    pub fn display(&self) -> String {
        match self {
            PortMatch::Any => String::from("*"),
            PortMatch::Exact(aa) => format!("{}", aa),
            PortMatch::Range(lo, hi) => format!("{}:{}", lo, hi),
        }
    }
}


#[derive(Debug, Clone)]
pub struct Rule {
    pub chain: Chain,
    pub protocol: Protocol,
    pub src_ip: IpMatch,
    pub dst_ip: IpMatch,
    pub src_port: PortMatch,
    pub dst_port: PortMatch,
    pub action: Action,
    pub comment: String,
    
    pub packets: u64,
    
    pub bytes: u64,
}

impl Rule {
    pub fn new(chain: Chain, action: Action) -> Self {
        Self {
            chain,
            protocol: Protocol::Any,
            src_ip: IpMatch::Any,
            dst_ip: IpMatch::Any,
            src_port: PortMatch::Any,
            dst_port: PortMatch::Any,
            action,
            comment: String::new(),
            packets: 0,
            bytes: 0,
        }
    }

    
    pub fn matches(&self, arv: u8, src: [u8; 4], dst: [u8; 4], ais: u16, ahv: u16) -> bool {
        
        if let Some(aa) = self.protocol.number() {
            if aa != arv {
                return false;
            }
        }
        
        if !self.src_ip.matches(src) {
            return false;
        }
        if !self.dst_ip.matches(dst) {
            return false;
        }
        
        if arv == 6 || arv == 17 {
            if !self.src_port.matches(ais) {
                return false;
            }
            if !self.dst_port.matches(ahv) {
                return false;
            }
        }
        true
    }
}





struct Si {
    rules: Vec<Rule>,
    input_policy: Action,
    output_policy: Action,
    forward_policy: Action,
    log_entries: Vec<String>,
}

impl Si {
    fn new() -> Self {
        Self {
            rules: Vec::new(),
            input_policy: Action::Accept,
            output_policy: Action::Accept,
            forward_policy: Action::Drop,
            log_entries: Vec::new(),
        }
    }

    fn get_policy(&self, chain: Chain) -> Action {
        match chain {
            Chain::Input => self.input_policy,
            Chain::Output => self.output_policy,
            Chain::Forward => self.forward_policy,
        }
    }

    fn set_policy(&mut self, chain: Chain, action: Action) {
        match chain {
            Chain::Input => self.input_policy = action,
            Chain::Output => self.output_policy = action,
            Chain::Forward => self.forward_policy = action,
        }
    }

    fn add_log(&mut self, entry: String) {
        if self.log_entries.len() >= 256 {
            self.log_entries.remove(0);
        }
        self.log_entries.push(entry);
    }
}

static Ee: Mutex<Si> = Mutex::new(Si {
    rules: Vec::new(),
    input_policy: Action::Accept,
    output_policy: Action::Accept,
    forward_policy: Action::Drop,
    log_entries: Vec::new(),
});

static Cq: AtomicBool = AtomicBool::new(false);
static PO_: AtomicU64 = AtomicU64::new(0);
static PP_: AtomicU64 = AtomicU64::new(0);






pub fn lq() -> bool {
    Cq.load(Ordering::Relaxed)
}


pub fn set_enabled(enabled: bool) {
    Cq.store(enabled, Ordering::Release);
}


pub fn lvk(arv: u8, src: [u8; 4], dst: [u8; 4], ais: u16, ahv: u16, aup: usize) -> bool {
    if !lq() {
        return true;
    }
    hyn(Chain::Input, arv, src, dst, ais, ahv, aup)
}


pub fn hyo(arv: u8, src: [u8; 4], dst: [u8; 4], ais: u16, ahv: u16, aup: usize) -> bool {
    if !lq() {
        return true;
    }
    hyn(Chain::Output, arv, src, dst, ais, ahv, aup)
}


fn hyn(chain: Chain, arv: u8, src: [u8; 4], dst: [u8; 4], ais: u16, ahv: u16, aup: usize) -> bool {
    let mut fo = Ee.lock();

    
    for qo in fo.rules.iter_mut() {
        if qo.chain != chain {
            continue;
        }
        if qo.matches(arv, src, dst, ais, ahv) {
            qo.packets += 1;
            qo.bytes += aup as u64;

            match qo.action {
                Action::Accept => {
                    PO_.fetch_add(1, Ordering::Relaxed);
                    return true;
                }
                Action::Drop => {
                    PP_.fetch_add(1, Ordering::Relaxed);
                    return false;
                }
                Action::Reject => {
                    PP_.fetch_add(1, Ordering::Relaxed);
                    ony(arv, src, dst, ais, ahv);
                    return false;
                }
                Action::Log => {
                    let nza = match arv {
                        1 => "ICMP",
                        6 => "TCP",
                        17 => "UDP",
                        _ => "???",
                    };
                    let entry = format!(
                        "[FW {}] {} {}.{}.{}.{}:{} -> {}.{}.{}.{}:{} len={}",
                        chain.name(), nza,
                        src[0], src[1], src[2], src[3], ais,
                        dst[0], dst[1], dst[2], dst[3], ahv,
                        aup,
                    );
                    crate::serial_println!("{}", entry);
                    fo.add_log(entry);
                    PO_.fetch_add(1, Ordering::Relaxed);
                    return true; 
                }
            }
        }
    }

    
    let policy = fo.get_policy(chain);
    match policy {
        Action::Accept | Action::Log => {
            PO_.fetch_add(1, Ordering::Relaxed);
            true
        }
        Action::Drop | Action::Reject => {
            PP_.fetch_add(1, Ordering::Relaxed);
            false
        }
    }
}






pub fn jtx(qo: Rule) {
    Ee.lock().rules.push(qo);
}


pub fn qln(index: usize, qo: Rule) {
    let mut fo = Ee.lock();
    if index <= fo.rules.len() {
        fo.rules.insert(index, qo);
    }
}


pub fn ldc(chain: Chain, index: usize) -> bool {
    let mut fo = Ee.lock();
    let mut hki = 0usize;
    let mut iyn = None;
    for (i, qo) in fo.rules.iter().enumerate() {
        if qo.chain == chain {
            if hki == index {
                iyn = Some(i);
                break;
            }
            hki += 1;
        }
    }
    if let Some(i) = iyn {
        fo.rules.remove(i);
        true
    } else {
        false
    }
}


pub fn flush(chain: Option<Chain>) {
    let mut fo = Ee.lock();
    match chain {
        Some(c) => fo.rules.retain(|r| r.chain != c),
        None => fo.rules.clear(),
    }
}


pub fn set_policy(chain: Chain, action: Action) {
    Ee.lock().set_policy(chain, action);
}


pub fn mzh(chain: Chain) -> Vec<Rule> {
    Ee.lock().rules.iter().filter(|r| r.chain == chain).cloned().collect()
}


pub fn get_policy(chain: Chain) -> Action {
    Ee.lock().get_policy(chain)
}


pub fn stats() -> (u64, u64) {
    (PO_.load(Ordering::Relaxed), PP_.load(Ordering::Relaxed))
}


pub fn mdj() -> Vec<String> {
    Ee.lock().log_entries.clone()
}


pub fn kku() {
    Ee.lock().log_entries.clear();
}


pub fn jai() {
    PO_.store(0, Ordering::Relaxed);
    PP_.store(0, Ordering::Relaxed);
    let mut fo = Ee.lock();
    for qo in fo.rules.iter_mut() {
        qo.packets = 0;
        qo.bytes = 0;
    }
}





fn art(j: &str) -> Option<[u8; 4]> {
    let au: Vec<&str> = j.split('.').collect();
    if au.len() != 4 {
        return None;
    }
    let a: u8 = au[0].parse().ok()?;
    let b: u8 = au[1].parse().ok()?;
    let c: u8 = au[2].parse().ok()?;
    let d: u8 = au[3].parse().ok()?;
    Some([a, b, c, d])
}


fn ony(arv: u8, src: [u8; 4], dst: [u8; 4], ais: u16, ahv: u16) {
    if arv == 6 {
        
        ood(src, dst, ais, ahv);
    } else {
        
        onr(src, dst);
    }
}


fn ood(tn: [u8; 4], local_ip: [u8; 4], remote_port: u16, local_port: u16) {
    
    let mut gq = [0u8; 20];
    
    gq[0..2].copy_from_slice(&local_port.to_be_bytes());
    
    gq[2..4].copy_from_slice(&remote_port.to_be_bytes());
    
    
    
    gq[12] = 0x50; 
    gq[13] = 0x14; 
    
    
    

    
    let ig = super::tcp::jlr(local_ip, tn, &gq);
    gq[16..18].copy_from_slice(&ig.to_be_bytes());

    let _ = super::ip::aha(tn, 6, &gq);
}


fn onr(tn: [u8; 4], _local_ip: [u8; 4]) {
    
    let mut fj = [0u8; 8];
    fj[0] = 3;  
    fj[1] = 13; 
    
    

    
    let mut sum: u32 = 0;
    for i in (0..fj.len()).step_by(2) {
        let fx = if i + 1 < fj.len() {
            ((fj[i] as u16) << 8) | (fj[i + 1] as u16)
        } else {
            (fj[i] as u16) << 8
        };
        sum += fx as u32;
    }
    while sum >> 16 != 0 {
        sum = (sum & 0xFFFF) + (sum >> 16);
    }
    let ig = !(sum as u16);
    fj[2..4].copy_from_slice(&ig.to_be_bytes());

    let _ = super::ip::aha(tn, 1, &fj);
}
