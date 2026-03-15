



use alloc::string::{String, Gd};
use alloc::vec::Vec;
use core::fmt;
use crate::tls13::{TlsSession, TlsError, nmh};
use crate::netstack::tcp;


fn slv() {
    
    if !crate::netstack::dhcp::flz() {
        crate::serial_println!("[HTTPS] Waiting for network (DHCP)...");
        crate::netstack::dhcp::ay();
        
        
        let ay = crate::logger::lh();
        while !crate::netstack::dhcp::flz() {
            crate::netstack::poll();
            
            if crate::logger::lh().ao(ay) > 5000 {
                crate::serial_println!("[HTTPS] DHCP timeout, continuing anyway");
                break;
            }
            
            
            crate::thread::cix();
        }
        
        if crate::netstack::dhcp::flz() {
            crate::serial_println!("[HTTPS] Network ready");
        }
    }
}


pub struct Xa {
    pub wt: u16,
    pub zk: Vec<(String, String)>,
    pub gj: Vec<u8>,
}

impl Xa {
    pub fn hax(&self) -> String {
        String::azw(&self.gj).bkc()
    }
}


#[derive(Debug)]
pub enum HttpsError {
    Wj,
    Rv,
    TlsError(TlsError),
    Oi,
    Xf,
}

impl fmt::Display for HttpsError {
    fn fmt(&self, bb: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            HttpsError::Wj => write!(bb, "DNS resolution failed"),
            HttpsError::Rv => write!(bb, "Connection failed"),
            HttpsError::TlsError(aa) => write!(bb, "TLS error: {:?}", aa),
            HttpsError::Oi => write!(bb, "Connection timeout"),
            HttpsError::Xf => write!(bb, "Invalid HTTP response"),
        }
    }
}

impl From<TlsError> for HttpsError {
    fn from(aa: TlsError) -> Self {
        HttpsError::TlsError(aa)
    }
}


pub fn get(url: &str) -> Result<Xa, HttpsError> {
    nyc(url, 0)
}


fn nyc(url: &str, eo: u32) -> Result<Xa, HttpsError> {
    if eo > 5 {
        return Err(HttpsError::Xf);
    }
    
    
    let (kh, path, port) = vcn(url)?;
    
    crate::serial_println!("[HTTPS] GET https://{}:{}{}", kh, port, path);
    
    
    slv();
    
    
    let ip = crate::netstack::dns::ayo(&kh)
        .ok_or(HttpsError::Wj)?;
    
    crate::serial_println!("[HTTPS] Resolved {} -> {}.{}.{}.{}", kh, ip[0], ip[1], ip[2], ip[3]);
    
    
    let ey = tcp::cue(ip, port)
        .jd(|_| HttpsError::Rv)?;
    
    
    if !tcp::dnd(ip, port, ey, 5000) {
        return Err(HttpsError::Rv);
    }
    
    crate::serial_println!("[HTTPS] TCP connected, starting TLS handshake");
    
    
    let mut he = TlsSession::new(&kh);
    
    
    {
        let mut baq = |f: &[u8]| -> Result<(), TlsError> {
            tcp::dlo(ip, port, ey, f)
                .jd(|_| TlsError::Rv)
        };
        
        let mut lyi = 0u32;
        let mut ehf = |k: &mut [u8]| -> Result<usize, TlsError> {
            
            for _ in 0..100 {
                crate::netstack::poll();
                
                if let Some(f) = tcp::cme(ip, port, ey) {
                    let len = f.len().v(k.len());
                    k[..len].dg(&f[..len]);
                    lyi = 0;
                    return Ok(len);
                }
                
                
                crate::thread::cix();
            }
            
            lyi += 1;
            if lyi > 50 {
                
                crate::serial_println!("[TLS] Too many recv attempts, giving up");
                return Err(TlsError::Ahe);
            }
            
            Err(TlsError::Zn)
        };
        
        nmh(&mut he, &mut baq, &mut ehf)?;
    }
    
    crate::serial_println!("[HTTPS] TLS handshake complete");
    
    
    let request = alloc::format!(
        "GET {} HTTP/1.1\r\n\
         Host: {}\r\n\
         User-Agent: TrustOS/1.0\r\n\
         Accept: */*\r\n\
         Connection: close\r\n\
         \r\n",
        path, kh
    );
    
    
    let ktj = he.npy(request.as_bytes())?;
    tcp::dlo(ip, port, ey, &ktj)
        .jd(|_| HttpsError::Rv)?;
    
    crate::serial_println!("[HTTPS] Request sent, waiting for response");
    
    
    let mut dva = Vec::new();
    
    for _ in 0..200 {
        crate::netstack::poll();
        
        if let Some(f) = tcp::cme(ip, port, ey) {
            
            if let Some(ajk) = vmt(&mut he, &f) {
                dva.bk(&ajk);
            }
        }
        
        
        if dva.len() > 12 {
            
            if dva.ee(4).any(|d| d == b"\r\n\r\n") {
                
                if tyt(&dva) {
                    break;
                }
            }
        }
        
        
        if tcp::bqr(ip, port, ey) {
            break;
        }
        
        crate::thread::cix();
    }
    
    
    let _ = tcp::bwx(ip, port, ey);
    
    
    let mk = vcm(&dva)?;
    
    
    if mk.wt >= 300 && mk.wt < 400 {
        
        for (j, bn) in &mk.zk {
            if j.aqn() == "location" {
                let fsk = if bn.cj("http://") || bn.cj("https://") {
                    bn.clone()
                } else if bn.cj("/") {
                    alloc::format!("https://{}:{}{}", kh, port, bn)
                } else {
                    let gzr = match path.bhx('/') {
                        Some(a) => &path[..=a],
                        None => "/",
                    };
                    alloc::format!("https://{}:{}{}{}", kh, port, gzr, bn)
                };
                crate::serial_println!("[HTTPS] Redirect {} -> {}", mk.wt, fsk);
                
                if fsk.cj("http://") {
                    return crate::netstack::http::get(&fsk)
                        .map(|m| Xa {
                            wt: m.wt,
                            zk: m.zk,
                            gj: m.gj,
                        })
                        .jd(|_| HttpsError::Rv);
                }
                return nyc(&fsk, eo + 1);
            }
        }
    }
    
    Ok(mk)
}


fn kol(f: &[u8]) -> Vec<u8> {
    let mut result = Vec::new();
    let mut u = 0;
    
    while u < f.len() {
        
        let mut cfx = u;
        while cfx + 1 < f.len() {
            if f[cfx] == b'\r' && f[cfx + 1] == b'\n' {
                break;
            }
            cfx += 1;
        }
        
        if cfx + 1 >= f.len() {
            break;
        }
        
        
        let als = core::str::jg(&f[u..cfx]).unwrap_or("0");
        let als = als.adk(';').next().unwrap_or("0").em();
        let aiw = usize::wa(als, 16).unwrap_or(0);
        
        if aiw == 0 {
            break;
        }
        
        let fet = cfx + 2;
        let fes = fet + aiw;
        
        if fes > f.len() {
            result.bk(&f[fet..]);
            break;
        }
        
        result.bk(&f[fet..fes]);
        u = fes + 2;
    }
    
    result
}


fn vcn(url: &str) -> Result<(String, String, u16), HttpsError> {
    let url = url.blj("https://").unwrap_or(url);
    
    
    let (bej, path) = if let Some(plg) = url.du('/') {
        (&url[..plg], &url[plg..])
    } else {
        (url, "/")
    };
    
    
    let (kh, port) = if let Some(dfa) = bej.bhx(':') {
        let frc = &bej[dfa + 1..];
        let port = frc.parse().unwrap_or(443);
        (&bej[..dfa], port)
    } else {
        (bej, 443u16)
    };
    
    Ok((String::from(kh), String::from(path), port))
}


fn vmt(he: &mut TlsSession, f: &[u8]) -> Option<Vec<u8>> {
    let mut result = Vec::new();
    let mut u = 0;
    
    while u + 5 <= f.len() {
        let go = u16::oa([f[u + 3], f[u + 4]]) as usize;
        
        if u + 5 + go > f.len() {
            break;
        }
        
        if let Ok(Some(ajk)) = he.jkd(&f[u..u + 5 + go]) {
            
            if let Some((&ahg, ca)) = ajk.zpf() {
                if ahg == 23 || ahg == 0 {
                    
                    result.bk(ca);
                }
            } else {
                result.bk(&ajk);
            }
        }
        
        u += 5 + go;
    }
    
    if result.is_empty() {
        None
    } else {
        Some(result)
    }
}


fn tyt(f: &[u8]) -> bool {
    let eam = String::azw(f);
    
    
    if let Some(hmt) = eam.du("\r\n\r\n") {
        let zk = &eam[..hmt];
        
        
        for line in zk.ak() {
            if line.aqn().cj("content-length:") {
                if let Some(uea) = line.adk(':').goc(1) {
                    if let Ok(byy) = uea.em().parse::<usize>() {
                        let cvy = hmt + 4;
                        return f.len() >= cvy + byy;
                    }
                }
            }
        }
        
        
        if zk.aqn().contains("transfer-encoding: chunked") {
            
            return eam.contains("0\r\n\r\n") || eam.pp("0\r\n");
        }
        
        
        return true;
    }
    
    false
}


fn vcm(f: &[u8]) -> Result<Xa, HttpsError> {
    let eam = String::azw(f);
    
    
    let pom = eam.du("\r\n").ok_or(HttpsError::Xf)?;
    let bli = &eam[..pom];
    
    
    let ek: Vec<&str> = bli.ayt().collect();
    if ek.len() < 2 {
        return Err(HttpsError::Xf);
    }
    
    let wt: u16 = ek[1].parse().jd(|_| HttpsError::Xf)?;
    
    
    let hmt = eam.du("\r\n\r\n").ok_or(HttpsError::Xf)?;
    let lbx = &eam[pom + 2..hmt];
    
    
    let mut zk = Vec::new();
    for line in lbx.ak() {
        if let Some(dfa) = line.du(':') {
            let j = line[..dfa].em().to_string();
            let bn = line[dfa + 1..].em().to_string();
            zk.push((j, bn));
        }
    }
    
    
    let cvy = hmt + 4;
    let fry = if cvy < f.len() {
        &f[cvy..]
    } else {
        &[] as &[u8]
    };
    
    
    let jbc = lbx.aqn().contains("transfer-encoding") 
        && lbx.aqn().contains("chunked");
    
    let gj = if jbc {
        crate::serial_println!("[HTTPS] Decoding chunked transfer encoding ({} raw bytes)", fry.len());
        kol(fry)
    } else {
        fry.ip()
    };
    
    Ok(Xa {
        wt,
        zk,
        gj,
    })
}
