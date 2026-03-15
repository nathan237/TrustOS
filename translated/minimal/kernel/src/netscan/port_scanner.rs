








use alloc::vec::Vec;
use alloc::string::String;
use alloc::format;


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PortState {
    Ck,
    Dk,
    Kl,        
    Xx,    
}

impl PortState {
    pub fn as_str(&self) -> &'static str {
        match self {
            PortState::Ck => "open",
            PortState::Dk => "closed",
            PortState::Kl => "filtered",
            PortState::Xx => "open|filtered",
        }
    }
}


#[derive(Debug, Clone)]
pub struct Fd {
    pub port: u16,
    pub g: PortState,
    pub xi: &'static str,
    pub banner: Option<String>,
}


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScanType {
    Uu,         
    Wa,     
    Ic,         
}


#[derive(Debug, Clone)]
pub struct ScanConfig {
    pub cd: [u8; 4],
    pub xf: Vec<u16>,
    pub cmr: ScanType,
    pub sg: u32,
    pub ern: bool,
}

impl ScanConfig {
    pub fn new(cd: [u8; 4]) -> Self {
        Self {
            cd,
            xf: super::AAL_.ip(),
            cmr: ScanType::Uu,
            sg: 1500,
            ern: false,
        }
    }

    pub fn jxa(mut self, xf: Vec<u16>) -> Self {
        self.xf = xf;
        self
    }

    pub fn xuy(mut self, ay: u16, ci: u16) -> Self {
        self.xf = (ay..=ci).collect();
        self
    }

    pub fn jxd(mut self, cmr: ScanType) -> Self {
        self.cmr = cmr;
        self
    }

    pub fn jxb(mut self, jn: u32) -> Self {
        self.sg = jn;
        self
    }

    pub fn pzo(mut self, thg: bool) -> Self {
        self.ern = thg;
        self
    }

    pub fn jxc(mut self) -> Self {
        self.xf = super::BHM_.ip();
        self
    }
}


#[derive(Debug, Default)]
pub struct Amg {
    pub pvc: usize,
    pub aji: usize,
    pub cwg: usize,
    pub aud: usize,
    pub oz: u64,
}


pub fn arx(config: &ScanConfig) -> (Vec<Fd>, Amg) {
    let ay = crate::logger::lh();
    let mut hd = Vec::new();
    let mut cm = Amg {
        pvc: config.xf.len(),
        ..Default::default()
    };

    for &port in &config.xf {
        let result = match config.cmr {
            ScanType::Uu => wxa(config.cd, port, config.sg),
            ScanType::Wa => rnx(config.cd, port, config.sg),
            ScanType::Ic => xnz(config.cd, port, config.sg),
        };

        match result.g {
            PortState::Ck => cm.aji += 1,
            PortState::Dk => cm.cwg += 1,
            PortState::Kl | PortState::Xx => cm.aud += 1,
        }

        
        if result.g != PortState::Dk {
            hd.push(result);
        }
    }

    cm.oz = crate::logger::lh().ao(ay);
    (hd, cm)
}







fn wxa(cd: [u8; 4], port: u16, sg: u32) -> Fd {
    let xi = super::fui(port);

    
    let ey = match crate::netstack::tcp::cue(cd, port) {
        Ok(ai) => ai,
        Err(_) => {
            return Fd { port, g: PortState::Kl, xi, banner: None };
        }
    };

    
    let ay = crate::logger::lh();
    let mut aaf: u32 = 0;

    loop {
        crate::netstack::poll();

        if let Some(g) = crate::netstack::tcp::nxy(cd, port, ey) {
            match g {
                crate::netstack::tcp::TcpState::Pi => {
                    
                    let _ = who(cd, port, ey);
                    return Fd { port, g: PortState::Ck, xi, banner: None };
                }
                crate::netstack::tcp::TcpState::Dk => {
                    return Fd { port, g: PortState::Dk, xi, banner: None };
                }
                _ => {}
            }
        }

        if crate::logger::lh().ao(ay) > sg as u64 {
            return Fd { port, g: PortState::Kl, xi, banner: None };
        }
        aaf = aaf.cn(1);
        if aaf > 500_000 {
            return Fd { port, g: PortState::Kl, xi, banner: None };
        }
        core::hint::hc();
    }
}


fn who(kv: [u8; 4], rz: u16, ey: u16) -> Result<(), &'static str> {
    let jh = crate::network::aou()
        .map(|(ip, _, _)| *ip.as_bytes())
        .unwrap_or([10, 0, 2, 15]);

    let mut ie = alloc::vec::Vec::fc(20);
    ie.bk(&ey.ft());
    ie.bk(&rz.ft());
    ie.bk(&0u32.ft()); 
    ie.bk(&0u32.ft()); 
    ie.push(0x50); 
    ie.push(crate::netstack::tcp::flags::Bqg);
    ie.bk(&0u16.ft()); 
    ie.bk(&0u16.ft()); 
    ie.bk(&0u16.ft()); 

    
    let mut dkw = alloc::vec::Vec::fc(32);
    dkw.bk(&jh);
    dkw.bk(&kv);
    dkw.push(0);
    dkw.push(6);
    dkw.bk(&(ie.len() as u16).ft());
    dkw.bk(&ie);
    let td = tsu(&dkw);
    ie[16] = (td >> 8) as u8;
    ie[17] = (td & 0xFF) as u8;

    crate::netstack::ip::blc(kv, 6, &ie)
}

fn tsu(f: &[u8]) -> u16 {
    let mut sum: u32 = 0;
    let mut a = 0;
    while a + 1 < f.len() {
        sum += ((f[a] as u32) << 8) | (f[a + 1] as u32);
        a += 2;
    }
    if a < f.len() {
        sum += (f[a] as u32) << 8;
    }
    while (sum >> 16) != 0 {
        sum = (sum & 0xFFFF) + (sum >> 16);
    }
    !(sum as u16)
}


fn rnx(cd: [u8; 4], port: u16, sg: u32) -> Fd {
    let xi = super::fui(port);

    let ey = match crate::netstack::tcp::cue(cd, port) {
        Ok(ai) => ai,
        Err(_) => {
            return Fd { port, g: PortState::Kl, xi, banner: None };
        }
    };

    
    let fhz = crate::netstack::tcp::dnd(cd, port, ey, sg);

    if fhz {
        
        let _ = crate::netstack::tcp::bwx(cd, port, ey);
        Fd { port, g: PortState::Ck, xi, banner: None }
    } else {
        
        match crate::netstack::tcp::nxy(cd, port, ey) {
            Some(crate::netstack::tcp::TcpState::Dk) => {
                Fd { port, g: PortState::Dk, xi, banner: None }
            }
            _ => {
                Fd { port, g: PortState::Kl, xi, banner: None }
            }
        }
    }
}


fn xnz(cd: [u8; 4], port: u16, sg: u32) -> Fd {
    let xi = super::fui(port);

    
    let ew = xny(port);
    let ey = crate::netstack::udp::muy();

    if crate::netstack::udp::dlp(cd, port, ey, &ew).is_err() {
        return Fd { port, g: PortState::Kl, xi, banner: None };
    }

    
    let ay = crate::logger::lh();
    let mut aaf: u32 = 0;

    loop {
        crate::netstack::poll();

        
        if crate::netstack::udp::jlt(ey).is_some() {
            return Fd { port, g: PortState::Ck, xi, banner: None };
        }

        
        if let Some(rq) = crate::netstack::icmp::xti(cd, 0) {
            if rq.hih == crate::netstack::icmp::AWR_ && rq.aj == 3 {
                return Fd { port, g: PortState::Dk, xi, banner: None };
            }
        }

        if crate::logger::lh().ao(ay) > sg as u64 {
            return Fd { port, g: PortState::Xx, xi, banner: None };
        }
        aaf = aaf.cn(1);
        if aaf > 500_000 {
            return Fd { port, g: PortState::Xx, xi, banner: None };
        }
        core::hint::hc();
    }
}


fn xny(port: u16) -> Vec<u8> {
    match port {
        
        53 => {
            let mut dns = Vec::new();
            dns.bk(&[0x00, 0x01]); 
            dns.bk(&[0x01, 0x00]); 
            dns.bk(&[0x00, 0x01]); 
            dns.bk(&[0x00, 0x00, 0x00, 0x00]); 
            dns.bk(&[0x07]); 
            dns.bk(b"version");
            dns.bk(&[0x04]); 
            dns.bk(b"bind");
            dns.bk(&[0x00]); 
            dns.bk(&[0x00, 0x10]); 
            dns.bk(&[0x00, 0x03]); 
            dns
        }
        
        161 => {
            alloc::vec![
                0x30, 0x26, 0x02, 0x01, 0x01, 0x04, 0x06, 0x70, 0x75, 0x62, 0x6C, 0x69, 0x63,
                0xA0, 0x19, 0x02, 0x01, 0x01, 0x02, 0x01, 0x00, 0x02, 0x01, 0x00, 0x30, 0x0E,
                0x30, 0x0C, 0x06, 0x08, 0x2B, 0x06, 0x01, 0x02, 0x01, 0x01, 0x01, 0x00, 0x05, 0x00,
            ]
        }
        
        123 => {
            let mut orj = alloc::vec![0u8; 48];
            orj[0] = 0x1B; 
            orj
        }
        
        137 => {
            alloc::vec![
                0x80, 0x94, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00,
                0x00, 0x00, 0x00, 0x00, 0x20, 0x43, 0x4B, 0x41,
                0x41, 0x41, 0x41, 0x41, 0x41, 0x41, 0x41, 0x41,
                0x41, 0x41, 0x41, 0x41, 0x41, 0x41, 0x41, 0x41,
                0x41, 0x41, 0x41, 0x41, 0x41, 0x41, 0x41, 0x41,
                0x41, 0x41, 0x41, 0x41, 0x41, 0x00, 0x00, 0x21,
                0x00, 0x01,
            ]
        }
        
        1900 => {
            let fr = b"M-SEARCH * HTTP/1.1\r\nHost:239.255.255.250:1900\r\nST:ssdp:all\r\nMAN:\"ssdp:discover\"\r\nMX:1\r\n\r\n";
            fr.ip()
        }
        
        _ => alloc::vec![0x00; 4],
    }
}


pub fn oyv(cd: [u8; 4]) -> (Vec<Fd>, Amg) {
    let config = ScanConfig::new(cd);
    arx(&config)
}


pub fn yyi(cd: [u8; 4]) -> (Vec<Fd>, Amg) {
    let config = ScanConfig::new(cd)
        .jxc()
        .pzo(true)
        .jxb(2000);
    arx(&config)
}
