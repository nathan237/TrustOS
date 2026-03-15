



extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;
use alloc::format;




#[derive(Debug, Clone)]
pub enum PolicyVerdict {
    
    Zs,
    
    Pf(String),
    
    Nl,
}




#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PolicyPreset {
    
    Aet,
    
    Ade,
    
    Ads,
}




#[derive(Debug, Clone)]
pub struct DomainRule {
    
    pub pattern: String,
    
    pub allow: bool,
}

impl DomainRule {
    pub fn allow(pattern: &str) -> Self {
        Self { pattern: String::from(pattern), allow: true }
    }

    pub fn deny(pattern: &str) -> Self {
        Self { pattern: String::from(pattern), allow: false }
    }

    
    pub fn oh(&self, vh: &str) -> bool {
        if self.pattern.cj("*.") {
            
            let cif = &self.pattern[1..]; 
            vh.pp(cif) || vh == &self.pattern[2..]
        } else {
            vh == self.pattern
        }
    }
}




#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContentFilter {
    
    Age,
    
    Agd,
    
    Agf,
    
    Aox,
    
    Aoy,
    
    Agg,
    
    Aow,
}




const BLH_: &[u16] = &[
    21,   
    22,   
    23,   
    25,   
    110,  
    143,  
    445,  
    3306, 
    3389, 
    5432, 
    5900, 
    6379, 
    27017, 
];




const BQY_: &[&str] = &[
    "file://",
    "javascript:",
    "data:text/html",
    "blob:",
    "about:",
    "localhost",
    "127.0.0.1",
    "0.0.0.0",
    "[::]",
    "[::1]",
    "169.254.",     
    "10.",          
    "192.168.",     
    "172.16.",      
];




fn txd(url: &str) -> Option<&'static str> {
    let pb = url.avd();
    for &pat in BQY_ {
        if pb.contains(pat) {
            return Some(pat);
        }
    }
    
    
    if let Some(tpx) = pb.du("://") {
        let ddv = &pb[tpx + 3..];
        if let Some(fv) = ddv.bw().next() {
            if fv.atb() {
                
                
                if ddv.cj("10.") || ddv.cj("192.168.") 
                    || ddv.cj("172.16.") || ddv.cj("127.")
                    || ddv.cj("0.") {
                    return Some("private IP address");
                }
            }
        }
    }
    None
}


pub fn ggn(url: &str) -> Option<&str> {
    let fbq = if let Some(u) = url.du("://") {
        &url[u + 3..]
    } else {
        url
    };
    
    let kh = fbq.adk('/').next()?;
    
    let vh = if kh.contains(':') {
        kh.adk(':').next()?
    } else {
        kh
    };
    
    let vh = if vh.contains('@') {
        vh.adk('@').qv()?
    } else {
        vh
    };
    if vh.is_empty() { None } else { Some(vh) }
}


fn sqf(url: &str) -> u16 {
    let pb = url.avd();
    let fmc = pb.cj("https://");
    let eaq = if fmc { 443 } else { 80 };

    let fbq = if let Some(u) = url.du("://") {
        &url[u + 3..]
    } else {
        url
    };
    let kh = fbq.adk('/').next().unwrap_or("");
    if let Some(cpj) = kh.bhx(':') {
        kh[cpj + 1..].parse().unwrap_or(eaq)
    } else {
        eaq
    }
}




#[derive(Debug, Clone)]
pub struct SandboxPolicy {
    pub akl: PolicyPreset,
    
    pub epa: Vec<DomainRule>,
    
    pub ijq: Vec<ContentFilter>,
    
    pub gqf: u32,
    
    pub eur: usize,
    
    pub gbf: bool,
    
    pub kdz: bool,
    
    pub kdx: bool,
    
    pub fcr: bool,
    
    pub fny: u8,
    
    pub emx: bool,
    
    pub glv: bool,
    
    pub ilu: Vec<String>,
}

impl SandboxPolicy {
    
    pub fn sye(akl: PolicyPreset) -> Self {
        match akl {
            PolicyPreset::Aet => Self::wvb(),
            PolicyPreset::Ade => Self::upk(),
            PolicyPreset::Ads => Self::vgu(),
        }
    }

    
    fn wvb() -> Self {
        Self {
            akl: PolicyPreset::Aet,
            epa: Vec::new(), 
            ijq: alloc::vec![
                ContentFilter::Age,
                ContentFilter::Agd,
                ContentFilter::Agf,
                ContentFilter::Agg,
            ],
            gqf: 30,
            eur: 512 * 1024, 
            gbf: true,
            kdz: true,
            kdx: true,
            fcr: true,
            fny: 2,
            emx: true,
            glv: true,
            ilu: Vec::new(),
        }
    }

    
    fn upk() -> Self {
        Self {
            akl: PolicyPreset::Ade,
            epa: alloc::vec![
                
                DomainRule::deny("*.doubleclick.net"),
                DomainRule::deny("*.googlesyndication.com"),
                DomainRule::deny("*.analytics.google.com"),
                DomainRule::deny("*.facebook.com"),
                DomainRule::deny("*.fbcdn.net"),
            ],
            ijq: alloc::vec![
                ContentFilter::Age,
                ContentFilter::Agd,
                ContentFilter::Agf,
                ContentFilter::Aoy,
                ContentFilter::Agg,
                ContentFilter::Aox,
                ContentFilter::Aow,
            ],
            gqf: 60,
            eur: 1024 * 1024, 
            gbf: false,
            kdz: false,
            kdx: false,
            fcr: true,
            fny: 5,
            emx: true,
            glv: false,
            ilu: Vec::new(),
        }
    }

    
    fn vgu() -> Self {
        Self {
            akl: PolicyPreset::Ads,
            epa: Vec::new(), 
            ijq: alloc::vec![
                ContentFilter::Age,
                ContentFilter::Agd,
                ContentFilter::Agf,
                ContentFilter::Aoy,
                ContentFilter::Agg,
                ContentFilter::Aox,
                ContentFilter::Aow,
            ],
            gqf: 120,
            eur: 4 * 1024 * 1024, 
            gbf: false,
            kdz: false,
            kdx: false,
            fcr: true,
            fny: 10,
            emx: true, 
            glv: false,
            ilu: Vec::new(),
        }
    }

    
    pub fn kaf(&mut self, pattern: &str) {
        self.epa.insert(0, DomainRule::allow(pattern));
    }

    
    pub fn kpc(&mut self, pattern: &str) {
        self.epa.insert(0, DomainRule::deny(pattern));
    }

    
    pub fn nrf(&self, url: &str) -> PolicyVerdict {
        
        if let Some(pattern) = txd(url) {
            return PolicyVerdict::Pf(format!("blocked dangerous pattern: {}", pattern));
        }

        
        let port = sqf(url);
        if BLH_.contains(&port) {
            return PolicyVerdict::Pf(format!("blocked port: {}", port));
        }

        
        let pb = url.avd();
        for cdg in &self.ilu {
            if pb.contains(&cdg.avd()) {
                return PolicyVerdict::Pf(format!("blocked pattern: {}", cdg));
            }
        }

        
        let vh = match ggn(url) {
            Some(bc) => bc,
            None => return PolicyVerdict::Pf(String::from("cannot extract domain")),
        };

        
        for agu in &self.epa {
            if agu.oh(vh) {
                if agu.allow {
                    return if self.glv { PolicyVerdict::Nl } else { PolicyVerdict::Zs };
                } else {
                    return PolicyVerdict::Pf(format!("domain blocked: {}", vh));
                }
            }
        }

        
        match self.akl {
            PolicyPreset::Aet => {
                
                PolicyVerdict::Pf(format!("domain not in allowlist: {}", vh))
            },
            PolicyPreset::Ade => {
                
                if self.glv { PolicyVerdict::Nl } else { PolicyVerdict::Zs }
            },
            PolicyPreset::Ads => {
                PolicyVerdict::Zs
            },
        }
    }

    
    pub fn ohg(&self) -> bool {
        !self.gbf
    }

    
    pub fn yjy(&self, ahg: &str) -> bool {
        let aqx = ahg.avd();
        for hi in &self.ijq {
            match hi {
                ContentFilter::Age => if aqx.contains("text/html") || aqx.contains("application/xhtml") { return true; },
                ContentFilter::Agd => if aqx.contains("text/css") { return true; },
                ContentFilter::Agf => if aqx.contains("image/") { return true; },
                ContentFilter::Aox => if aqx.contains("javascript") || aqx.contains("ecmascript") { return true; },
                ContentFilter::Aoy => if aqx.contains("json") || aqx.contains("application/xml") { return true; },
                ContentFilter::Agg => if aqx.contains("text/plain") || aqx.contains("text/") { return true; },
                ContentFilter::Aow => if aqx.contains("font/") || aqx.contains("woff") { return true; },
            }
        }
        false
    }

    
    pub fn awz(&self) -> String {
        let qhf: usize = self.epa.iter().hi(|m| m.allow).az();
        let rvp: usize = self.epa.iter().hi(|m| !m.allow).az();
        format!(
            "Policy: {:?}\n  Domain rules: {} allow, {} deny\n  Rate limit: {}/min\n  Max response: {} KB\n  JS: {}\n  Redirects: {} (max depth {})\n  SSRF protection: {}\n  Log all: {}",
            self.akl,
            qhf, rvp,
            self.gqf,
            self.eur / 1024,
            if self.gbf { "BLOCKED" } else { "sandboxed" },
            if self.fcr { "allowed" } else { "blocked" },
            self.fny,
            if self.emx { "ON" } else { "OFF" },
            if self.glv { "yes" } else { "no" },
        )
    }
}
