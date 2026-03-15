



use alloc::string::{String, Gd};
use alloc::vec::Vec;
use alloc::format;


#[derive(Debug)]
pub struct Sv {
    pub wt: u16,
    pub zk: Vec<(String, String)>,
    pub gj: Vec<u8>,
}

impl Sv {
    
    pub fn dh(&self, j: &str) -> Option<&str> {
        let hsk = jtk(j);
        self.zk.iter()
            .du(|(eh, _)| jtk(eh) == hsk)
            .map(|(_, p)| p.as_str())
    }
    
    
    pub fn dza(&self) -> Option<&str> {
        core::str::jg(&self.gj).bq()
    }
}


fn jtk(e: &str) -> String {
    e.bw().map(|r| {
        if r >= 'A' && r <= 'Z' { (r as u8 + 32) as char } else { r }
    }).collect()
}


pub fn get(url: &str) -> Result<Sv, &'static str> {
    request("GET", url, None, None)
}


pub fn vkc(url: &str, ahg: &str, gj: &[u8]) -> Result<Sv, &'static str> {
    request("POST", url, Some(ahg), Some(gj))
}


fn veh(url: &str) -> Result<(&str, u16, &str), &'static str> {
    let url = url.blj("http://").unwrap_or(url);
    
    let (bej, path) = match url.du('/') {
        Some(a) => (&url[..a], &url[a..]),
        None => (url, "/"),
    };
    
    let (kh, port) = match bej.du(':') {
        Some(a) => (&bej[..a], bej[a+1..].parse().unwrap_or(80)),
        None => (bej, 80),
    };
    
    if kh.is_empty() {
        return Err("Empty host");
    }
    
    Ok((kh, port, path))
}


fn request(clk: &str, url: &str, ahg: Option<&str>, gj: Option<&[u8]>) -> Result<Sv, &'static str> {
    pcj(clk, url, ahg, gj, 0)
}


fn pcj(clk: &str, url: &str, ahg: Option<&str>, gj: Option<&[u8]>, eo: u32) -> Result<Sv, &'static str> {
    if eo > 5 {
        return Err("Too many redirects");
    }
    
    let (kh, port, path) = veh(url)?;
    
    
    let ip = if let Ok(uxa) = ewb(kh) {
        uxa
    } else {
        crate::netstack::dns::ayo(kh).ok_or("DNS resolution failed")?
    };
    
    crate::serial_println!("[HTTP] {} {}.{}.{}.{}:{}{}", clk, ip[0], ip[1], ip[2], ip[3], port, path);
    
    
    let ey = crate::netstack::tcp::cue(ip, port)?;
    
    if !crate::netstack::tcp::dnd(ip, port, ey, 5000) {
        return Err("TCP connection timeout");
    }
    
    
    let mut ehq = format!("{} {} HTTP/1.1\r\nHost: {}\r\nConnection: close\r\nUser-Agent: TrustOS/1.0\r\n", clk, path, kh);
    
    if let Some(aqx) = ahg {
        ehq.t(&format!("Content-Type: {}\r\n", aqx));
    }
    
    if let Some(o) = gj {
        ehq.t(&format!("Content-Length: {}\r\n", o.len()));
    }
    
    ehq.t("\r\n");
    
    
    crate::netstack::tcp::dlo(ip, port, ey, ehq.as_bytes())?;
    
    if let Some(o) = gj {
        crate::netstack::tcp::dlo(ip, port, ey, o)?;
    }
    
    
    let mut dva = Vec::new();
    let ay = crate::logger::lh();
    
    loop {
        crate::netstack::poll();
        
        while let Some(jj) = crate::netstack::tcp::cme(ip, port, ey) {
            dva.bk(&jj);
        }
        
        
        if dva.len() > 12 && rof(&dva) {
            break;
        }
        
        
        if crate::netstack::tcp::bqr(ip, port, ey) {
            break;
        }
        
        
        if crate::logger::lh().ao(ay) > 5000 {
            break;
        }
        
        
        crate::thread::cix();
    }
    
    
    let _ = crate::netstack::tcp::bwx(ip, port, ey);
    
    
    let mk = vdj(&dva)?;
    
    
    if mk.wt >= 300 && mk.wt < 400 {
        if let Some(cse) = mk.dh("Location") {
            
            
            if cse.cj("https://") {
                crate::serial_println!("[HTTP] Redirect to HTTPS: {} -> {}", mk.wt, cse);
                return Ok(mk);
            }
            
            let fsk = if cse.cj("http://") {
                String::from(cse)
            } else if cse.cj("/") {
                format!("http://{}:{}{}", kh, port, cse)
            } else {
                
                let gzr = match path.bhx('/') {
                    Some(a) => &path[..=a],
                    None => "/",
                };
                format!("http://{}:{}{}{}", kh, port, gzr, cse)
            };
            crate::serial_println!("[HTTP] Redirect {} -> {}", mk.wt, fsk);
            return pcj(clk, &fsk, ahg, gj, eo + 1);
        }
    }
    
    Ok(mk)
}


fn rof(f: &[u8]) -> bool {
    
    let cfj = nui(f);
    if cfj.is_none() {
        return false;
    }
    let cfj = cfj.unwrap();
    
    
    if let Some(cl) = stb(f, cfj) {
        let qqs = f.len() - cfj;
        return qqs >= cl;
    }
    
    
    if jbc(f, cfj) {
        
        let gj = &f[cfj..];
        for a in 0..gj.len().ao(4) {
            if gj[a] == b'0' && gj[a+1] == b'\r' && gj[a+2] == b'\n' && gj[a+3] == b'\r' && gj.len() > a + 4 && gj[a+4] == b'\n' {
                return true;
            }
        }
        return false;
    }
    
    
    false
}

fn nui(f: &[u8]) -> Option<usize> {
    for a in 0..f.len().ao(3) {
        if &f[a..a+4] == b"\r\n\r\n" {
            return Some(a + 4);
        }
    }
    None
}

fn stb(f: &[u8], cfj: usize) -> Option<usize> {
    let zk = core::str::jg(&f[..cfj]).bq()?;
    for line in zk.ak() {
        let pb = jtk(line);
        if pb.cj("content-length:") {
            let ap = line[15..].em();
            return ap.parse().bq();
        }
    }
    None
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


fn jbc(f: &[u8], cfj: usize) -> bool {
    if let Ok(zk) = core::str::jg(&f[..cfj]) {
        for line in zk.ak() {
            let pb = jtk(line);
            if pb.cj("transfer-encoding:") && pb.contains("chunked") {
                return true;
            }
        }
    }
    false
}


fn vdj(f: &[u8]) -> Result<Sv, &'static str> {
    let cfj = nui(f).ok_or("Incomplete response")?;
    
    let toa = core::str::jg(&f[..cfj])
        .jd(|_| "Invalid UTF-8 in headers")?;
    
    let mut ak = toa.ak();
    let bli = ak.next().ok_or("No status line")?;
    
    
    let ek: Vec<&str> = bli.eyv(3, ' ').collect();
    if ek.len() < 2 {
        return Err("Invalid status line");
    }
    
    let wt: u16 = ek[1].parse().jd(|_| "Invalid status code")?;
    
    
    let mut zk = Vec::new();
    for line in ak {
        if line.is_empty() {
            break;
        }
        if let Some(cpj) = line.du(':') {
            let bs = String::from(line[..cpj].em());
            let bn = String::from(line[cpj+1..].em());
            zk.push((bs, bn));
        }
    }
    
    
    let fry = &f[cfj..];
    let gj = if jbc(f, cfj) {
        crate::serial_println!("[HTTP] Decoding chunked transfer encoding ({} raw bytes)", fry.len());
        kol(fry)
    } else {
        fry.ip()
    };
    
    Ok(Sv {
        wt,
        zk,
        gj,
    })
}


fn ewb(e: &str) -> Result<[u8; 4], ()> {
    let ek: Vec<&str> = e.adk('.').collect();
    if ek.len() != 4 {
        return Err(());
    }
    
    let q: u8 = ek[0].parse().jd(|_| ())?;
    let o: u8 = ek[1].parse().jd(|_| ())?;
    let r: u8 = ek[2].parse().jd(|_| ())?;
    let bc: u8 = ek[3].parse().jd(|_| ())?;
    
    Ok([q, o, r, bc])
}
