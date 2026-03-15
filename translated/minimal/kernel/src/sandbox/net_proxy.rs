





extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;
use alloc::format;

use super::Ax;
use super::policy::{SandboxPolicy, ggn};




#[derive(Debug, Clone)]
pub struct Fz {
    pub wt: u16,
    pub ahg: String,
    pub gj: Vec<u8>,
    pub zk: Vec<(String, String)>,
    pub mqh: bool,
}

impl Fz {
    
    pub fn hax(&self) -> String {
        String::azw(&self.gj).bkc()
    }

    
    pub fn oga(&self) -> bool {
        self.ahg.contains("text/html") || self.ahg.contains("xhtml")
    }
}


#[derive(Debug)]
pub enum ProxyError {
    Beu(String),
    Bqk,
    Ckr,
    Qd(String),
    Wj,
    TlsError,
    Oi,
    Bjv,
}




struct RateLimiter {
    
    bh: Vec<u64>,
    
    lld: u32,
}

impl RateLimiter {
    fn new(lld: u32) -> Self {
        Self {
            bh: Vec::new(),
            lld,
        }
    }

    
    fn qyl(&mut self) -> bool {
        let iu = crate::time::lc();
        let uyj = iu.ao(60_000);

        
        self.bh.ajm(|&wi| wi > uyj);

        if self.bh.len() as u32 >= self.lld {
            false
        } else {
            self.bh.push(iu);
            true
        }
    }
}





pub struct NetProxy {
    afh: Ax,
    ozj: RateLimiter,
    
    eur: usize,
    fcr: bool,
    fny: u8,
    emx: bool,
    
    dxa: usize,
    
    xv: usize,
}

impl NetProxy {
    pub fn new(afh: Ax, policy: &SandboxPolicy) -> Self {
        Self {
            afh,
            ozj: RateLimiter::new(policy.gqf),
            eur: policy.eur,
            fcr: policy.fcr,
            fny: policy.fny,
            emx: policy.emx,
            dxa: 0,
            xv: 0,
        }
    }

    
    
    pub fn hjd(&mut self, url: &str, ate: usize) -> Result<Fz, ProxyError> {
        
        if url.is_empty() {
            return Err(ProxyError::Bjv);
        }

        
        let dto = self.gnx(url);

        
        if !self.ozj.qyl() {
            crate::serial_println!("[sandbox:{}] RATE LIMITED: {}", self.afh.0, dto);
            return Err(ProxyError::Bqk);
        }

        
        if self.emx {
            if let Some(vh) = ggn(&dto) {
                if self.tyo(vh) {
                    crate::serial_println!("[sandbox:{}] SSRF BLOCKED: {}", self.afh.0, vh);
                    return Err(ProxyError::Beu(format!("private IP: {}", vh)));
                }
            }
        }

        
        let ksu = core::cmp::v(ate, self.eur);
        let fmc = dto.cj("https://");

        crate::serial_println!("[sandbox:{}] FETCH: {} (max {} bytes)", self.afh.0, dto, ksu);

        let mk = if fmc {
            self.srt(&dto, ksu)?
        } else {
            self.srs(&dto, ksu)?
        };

        self.dxa += 1;
        self.xv += mk.gj.len();

        Ok(mk)
    }

    
    fn srs(&self, url: &str, ate: usize) -> Result<Fz, ProxyError> {
        
        
        match crate::netstack::http::get(url) {
            Ok(lj) => {
                let ahg = lj.zk.iter()
                    .du(|(eh, _)| eh.avd() == "content-type")
                    .map(|(_, p)| p.clone())
                    .unwrap_or_else(|| String::from("text/html"));

                let gj = if lj.gj.len() > ate {
                    
                    crate::serial_println!("[sandbox:{}] Response truncated: {} -> {} bytes",
                        self.afh.0, lj.gj.len(), ate);
                    lj.gj[..ate].ip()
                } else {
                    lj.gj
                };

                Ok(Fz {
                    wt: lj.wt,
                    ahg,
                    gj,
                    zk: lj.zk,
                    mqh: false,
                })
            }
            Err(aa) => {
                crate::serial_println!("[sandbox:{}] HTTP error: {}", self.afh.0, aa);
                Err(ProxyError::Qd(String::from(aa)))
            }
        }
    }

    
    fn srt(&self, url: &str, ate: usize) -> Result<Fz, ProxyError> {
        match crate::netstack::https::get(url) {
            Ok(lj) => {
                let ahg = lj.zk.iter()
                    .du(|(eh, _)| eh.avd() == "content-type")
                    .map(|(_, p)| p.clone())
                    .unwrap_or_else(|| String::from("text/html"));

                let gj = if lj.gj.len() > ate {
                    lj.gj[..ate].ip()
                } else {
                    lj.gj
                };

                Ok(Fz {
                    wt: lj.wt,
                    ahg,
                    gj,
                    zk: lj.zk,
                    mqh: false,
                })
            }
            Err(aa) => {
                crate::serial_println!("[sandbox:{}] HTTPS error: {:?}", self.afh.0, aa);
                match aa {
                    crate::netstack::https::HttpsError::Wj => Err(ProxyError::Wj),
                    crate::netstack::https::HttpsError::TlsError(_) => Err(ProxyError::TlsError),
                    crate::netstack::https::HttpsError::Oi => Err(ProxyError::Oi),
                    _ => Err(ProxyError::Qd(format!("{:?}", aa))),
                }
            }
        }
    }

    
    fn gnx(&self, url: &str) -> String {
        if url.cj("http://") || url.cj("https://") {
            String::from(url)
        } else if url.cj("//") {
            format!("http:{}", url)
        } else {
            format!("http://{}", url)
        }
    }

    
    fn tyo(&self, kh: &str) -> bool {
        
        let vln = [
            "localhost", "127.", "10.", "192.168.", "172.16.", "172.17.",
            "172.18.", "172.19.", "172.20.", "172.21.", "172.22.", "172.23.",
            "172.24.", "172.25.", "172.26.", "172.27.", "172.28.", "172.29.",
            "172.30.", "172.31.", "169.254.", "0.0.0.0", "[::]", "[::1]",
            "0.", "fc00:", "fd00:", "fe80:",
        ];
        let pb = kh.avd();
        for pattern in &vln {
            if pb.cj(pattern) || pb == *pattern {
                return true;
            }
        }
        
        
        if kh.parse::<u32>().is_ok() {
            
            return true;
        }
        false
    }

    
    pub fn cm(&self) -> (usize, usize) {
        (self.dxa, self.xv)
    }
}
