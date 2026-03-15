












pub mod html_parser;
pub mod css_parser;
pub mod js_engine;
pub mod renderer;
pub mod url;

pub use html_parser::*;
pub use css_parser::*;
pub use js_engine::*;
pub use renderer::*;
pub use url::*;

use alloc::string::String;
use alloc::vec::Vec;
use alloc::collections::BTreeMap;


#[derive(Clone)]
pub struct Aqe {
    pub j: String,
    pub bn: String,
    pub vh: String,
    pub path: String,
    pub hzi: bool,
    pub ocj: bool,
}


pub struct BrowserTab {
    pub url: String,
    pub dq: String,
    pub ama: Option<Su>,
    pub ug: i32,
    pub czh: Vec<Wz>,
    pub bfc: String,
    pub dca: bool,
    pub adv: Vec<String>,
    pub ari: usize,
    pub status: BrowserStatus,
    pub bhw: BTreeMap<String, Nx>,
    pub dug: Vec<String>,
}

impl BrowserTab {
    pub fn new() -> Self {
        Self {
            url: String::new(),
            dq: String::from("New Tab"),
            ama: None,
            ug: 0,
            czh: Vec::new(),
            bfc: String::new(),
            dca: false,
            adv: Vec::new(),
            ari: 0,
            status: BrowserStatus::Cv,
            bhw: BTreeMap::new(),
            dug: Vec::new(),
        }
    }
}


pub struct Browser {
    pub bdv: String,
    pub adv: Vec<String>,
    pub ari: usize,
    pub ama: Option<Su>,
    pub ug: i32,
    pub faz: u32,
    pub fyg: u32,
    pub status: BrowserStatus,
    pub czh: Vec<Wz>,
    
    pub bfc: String,
    
    pub dca: bool,
    
    pub bhw: BTreeMap<String, Nx>,
    
    pub dug: Vec<String>,
    
    pub ipg: Vec<Aqe>,
    
    pub bio: Vec<BrowserTab>,
    
    pub ahd: usize,
    
    pub hay: Vec<(String, String)>, 
    
    pub ohh: Vec<String>,
    
    pub ghi: BTreeMap<String, String>,
    
    pub kwq: Option<String>,
}


#[derive(Clone)]
pub struct Nx {
    pub ahg: ResourceType,
    pub f: Vec<u8>,
}


#[derive(Clone, PartialEq)]
pub enum ResourceType {
    Image,
    Mj,
    Cmq,
    Qg,
}


#[derive(Clone, PartialEq)]
pub enum BrowserStatus {
    Cv,
    Py,
    Q(String),
    At,
}


#[derive(Clone)]
pub struct Wz {
    pub cae: String,
    pub b: i32,
    pub c: i32,
    pub z: u32,
    pub ac: u32,
}

impl Browser {
    pub fn new(z: u32, ac: u32) -> Self {
        let sue = BrowserTab::new();
        Self {
            bdv: String::new(),
            adv: Vec::new(),
            ari: 0,
            ama: None,
            ug: 0,
            faz: z,
            fyg: ac,
            status: BrowserStatus::Cv,
            czh: Vec::new(),
            bfc: String::new(),
            dca: false,
            bhw: BTreeMap::new(),
            dug: Vec::new(),
            ipg: Vec::new(),
            bio: alloc::vec![sue],
            ahd: 0,
            hay: Vec::new(),
            ohh: Vec::new(),
            ghi: BTreeMap::new(),
            kwq: None,
        }
    }
    
    
    pub fn xja(&mut self) {
        self.dca = !self.dca;
        crate::serial_println!("[BROWSER] View mode: {}", if self.dca { "RAW" } else { "PARSED" });
    }
    
    
    pub fn bvn(&mut self, url: &str) -> Result<(), &'static str> {
        self.ooz(url, 0)
    }
    
    
    fn ooz(&mut self, url: &str, eo: u32) -> Result<(), &'static str> {
        if eo > 5 {
            self.status = BrowserStatus::Q(alloc::format!("Too many redirects"));
            return Err("Too many redirects");
        }
        
        self.status = BrowserStatus::Py;
        
        
        let amf = gnx(url, &self.bdv);
        
        crate::serial_println!("[BROWSER] Navigating to: {}", amf);
        
        
        let (wt, zk, gj) = if amf.cj("https://") {
            
            match crate::netstack::https::get(&amf) {
                Ok(m) => (m.wt, m.zk, m.gj),
                Err(aa) => {
                    crate::serial_println!("[BROWSER] HTTPS error: {}", aa);
                    self.status = BrowserStatus::Q(alloc::format!("HTTPS error: {}", aa));
                    self.bfc = alloc::format!(
                        "<html><body><h1>HTTPS Error</h1><p>Could not load {}</p><p>{}</p></body></html>",
                        amf, aa
                    );
                    self.ama = Some(due(&self.bfc));
                    self.bdv = amf;
                    self.ug = 0;
                    return Err("HTTPS error");
                }
            }
        } else {
            
            match crate::netstack::http::get(&amf) {
                Ok(m) => (m.wt, m.zk, m.gj),
                Err(aa) => {
                    crate::serial_println!("[BROWSER] Network error: {}", aa);
                    self.status = BrowserStatus::Q(alloc::format!("Network error: {}", aa));
                    self.bfc = alloc::format!(
                        "<html><body><h1>Error</h1><p>Could not load {}</p><p>{}</p></body></html>",
                        amf, aa
                    );
                    self.ama = Some(due(&self.bfc));
                    self.bdv = amf;
                    self.ug = 0;
                    return Err(aa);
                }
            }
        };
        
        if wt >= 400 {
            let fr = alloc::format!("HTTP {}", wt);
            self.status = BrowserStatus::Q(fr.clone());
            self.bfc = alloc::format!(
                "<html><body><h1>HTTP Error {}</h1><p>The server returned an error for {}</p></body></html>",
                wt, amf
            );
            self.ama = Some(due(&self.bfc));
            self.bdv = amf;
            self.ug = 0;
            return Err("HTTP error");
        }
        
        
        if wt >= 300 && wt < 400 {
            
            let cse = zk.iter()
                .du(|(eh, _)| eh.aqn() == "location")
                .map(|(_, p)| p.clone());
            if let Some(euf) = cse {
                crate::serial_println!("[BROWSER] Redirect {} -> {}", wt, euf);
                return self.ooz(&euf, eo + 1);
            }
        }
        
        
        let brb = core::str::jg(&gj).unwrap_or("");
        self.bfc = brb.to_string();
        
        
        self.lvt(&zk, &amf);
        
        
        self.ama = Some(due(brb));
        
        
        self.nrp();
        
        
        self.nsm(&amf);
        
        
        if self.ari < self.adv.len() {
            self.adv.dmu(self.ari);
        }
        self.adv.push(amf.clone());
        self.ari = self.adv.len();
        
        self.bdv = amf;
        self.ug = 0;
        self.status = BrowserStatus::At;
        
        Ok(())
    }
    
    
    pub fn nsm(&mut self, qnr: &str) {
        self.dug.clear();
        self.bhw.clear();
        
        if let Some(ref doc) = self.ama {
            let bhw = net(&doc.xq);
            for (ll, url) in bhw {
                let amf = gnx(&url, qnr);
                crate::serial_println!("[BROWSER] Found {} resource: {}", ll, amf);
                self.dug.push(amf);
            }
        }
    }
    
    
    pub fn zbe(&mut self) {
        let vxx: Vec<String> = self.dug.bbk(..).collect();
        
        for url in vxx {
            if self.bhw.bgm(&url) {
                continue; 
            }
            
            crate::serial_println!("[BROWSER] Loading resource: {}", url);
            
            match crate::netstack::http::get(&url) {
                Ok(mk) => {
                    if mk.wt == 200 {
                        let ahg = mk.dh("Content-Type").unwrap_or("");
                        let bsa = if ahg.contains("image") {
                            ResourceType::Image
                        } else if ahg.contains("css") {
                            ResourceType::Mj
                        } else if ahg.contains("javascript") {
                            ResourceType::Cmq
                        } else {
                            ResourceType::Qg
                        };
                        
                        crate::serial_println!("[BROWSER] Loaded {} ({} bytes)", url, mk.gj.len());
                        self.bhw.insert(url, Nx {
                            ahg: bsa,
                            f: mk.gj,
                        });
                    }
                }
                Err(aa) => {
                    crate::serial_println!("[BROWSER] Failed to load {}: {}", url, aa);
                }
            }
        }
    }
    
    
    pub fn qmf(&mut self) -> Result<(), &'static str> {
        if self.ari > 1 {
            self.ari -= 1;
            let url = self.adv[self.ari - 1].clone();
            self.bvn(&url)
        } else {
            Err("No previous page")
        }
    }
    
    
    pub fn fiz(&mut self) -> Result<(), &'static str> {
        if self.ari < self.adv.len() {
            self.ari += 1;
            let url = self.adv[self.ari - 1].clone();
            self.bvn(&url)
        } else {
            Err("No next page")
        }
    }
    
    
    pub fn gqr(&mut self) -> Result<(), &'static str> {
        let url = self.bdv.clone();
        self.bvn(&url)
    }
    
    
    pub fn jc(&mut self, aaq: i32) {
        self.ug = (self.ug + aaq).am(0);
    }
    
    
    pub fn tpb(&self, b: i32, c: i32) -> Option<&str> {
        let mue = c + self.ug;
        for arl in &self.czh {
            if b >= arl.b && b < arl.b + arl.z as i32 &&
               mue >= arl.c && mue < arl.c + arl.ac as i32 {
                return Some(&arl.cae);
            }
        }
        None
    }

    
    
    

    
    pub fn zdm(&mut self) {
        
        self.pfm();
        let acp = BrowserTab::new();
        self.bio.push(acp);
        self.ahd = self.bio.len() - 1;
        self.lje();
        crate::serial_println!("[BROWSER] New tab #{}", self.ahd);
    }

    
    pub fn zqj(&mut self, index: usize) {
        if index >= self.bio.len() || index == self.ahd {
            return;
        }
        self.pfm();
        self.ahd = index;
        self.lje();
        crate::serial_println!("[BROWSER] Switched to tab #{}", index);
    }

    
    pub fn yiy(&mut self) {
        if self.bio.len() <= 1 {
            return; 
        }
        self.bio.remove(self.ahd);
        if self.ahd >= self.bio.len() {
            self.ahd = self.bio.len() - 1;
        }
        self.lje();
    }

    
    pub fn zqv(&self) -> usize {
        self.bio.len()
    }

    
    pub fn zqw(&self) -> Vec<(String, bool)> {
        self.bio.iter().cf()
            .map(|(a, acp)| (acp.dq.clone(), a == self.ahd))
            .collect()
    }

    fn pfm(&mut self) {
        if let Some(acp) = self.bio.ds(self.ahd) {
            acp.url = self.bdv.clone();
            acp.ama = self.ama.take();
            acp.ug = self.ug;
            acp.czh = core::mem::take(&mut self.czh);
            acp.bfc = core::mem::take(&mut self.bfc);
            acp.dca = self.dca;
            acp.adv = self.adv.clone();
            acp.ari = self.ari;
            acp.status = self.status.clone();
            acp.bhw = core::mem::take(&mut self.bhw);
            acp.dug = core::mem::take(&mut self.dug);
            
            if let Some(ref doc) = acp.ama {
                if !doc.dq.is_empty() {
                    acp.dq = doc.dq.clone();
                }
            }
        }
    }

    fn lje(&mut self) {
        if let Some(acp) = self.bio.ds(self.ahd) {
            self.bdv = acp.url.clone();
            self.ama = acp.ama.take();
            self.ug = acp.ug;
            self.czh = core::mem::take(&mut acp.czh);
            self.bfc = core::mem::take(&mut acp.bfc);
            self.dca = acp.dca;
            self.adv = acp.adv.clone();
            self.ari = acp.ari;
            self.status = acp.status.clone();
            self.bhw = core::mem::take(&mut acp.bhw);
            self.dug = core::mem::take(&mut acp.dug);
        }
    }

    
    
    

    
    pub fn lvt(&mut self, zk: &[(String, String)], url: &str) {
        let vh = ggn(url);
        for (bs, bn) in zk {
            if bs.aqn() == "set-cookie" {
                if let Some(dzu) = vdr(bn, &vh) {
                    
                    self.ipg.ajm(|r| !(r.j == dzu.j && r.vh == dzu.vh));
                    self.ipg.push(dzu);
                }
            }
        }
    }

    
    pub fn yjz(&self, url: &str) -> Option<String> {
        let vh = ggn(url);
        let tyw = url.cj("https://");
        let path = sqe(url);

        let olh: Vec<String> = self.ipg.iter()
            .hi(|r| {
                vh.pp(&r.vh) &&
                path.cj(&r.path) &&
                (!r.hzi || tyw)
            })
            .map(|r| alloc::format!("{}={}", r.j, r.bn))
            .collect();

        if olh.is_empty() { None } else { Some(olh.rr("; ")) }
    }

    
    
    

    
    pub fn yej(&mut self) {
        let dq = self.ama.as_ref()
            .map(|bc| bc.dq.clone())
            .unwrap_or_else(|| self.bdv.clone());
        if !self.bdv.is_empty() {
            self.hay.push((self.bdv.clone(), dq));
            crate::serial_println!("[BROWSER] Bookmarked: {}", self.bdv);
        }
    }

    
    pub fn zjg(&mut self, index: usize) {
        if index < self.hay.len() {
            self.hay.remove(index);
        }
    }

    
    pub fn yzc(&self) -> bool {
        self.hay.iter().any(|(url, _)| url == &self.bdv)
    }

    
    
    

    
    pub fn zps(&mut self, hr: &str, clk: &str) -> Result<(), &'static str> {
        let amf = gnx(hr, &self.bdv);
        
        
        let mut dqw = String::new();
        for (j, bn) in &self.ghi {
            if !dqw.is_empty() { dqw.push('&'); }
            dqw.t(&moh(j));
            dqw.push('=');
            dqw.t(&moh(bn));
        }

        crate::serial_println!("[BROWSER] Form submit: {} {} data={}", clk, amf, dqw);

        if clk.dha("post") {
            
            match crate::netstack::http::vkc(&amf, "application/x-www-form-urlencoded", dqw.as_bytes()) {
                Ok(m) => {
                    
                    self.lvt(&m.zk, &amf);

                    if m.wt >= 300 && m.wt < 400 {
                        if let Some(euf) = m.zk.iter()
                            .du(|(eh, _)| eh.aqn() == "location")
                            .map(|(_, p)| p.clone())
                        {
                            return self.bvn(&euf);
                        }
                    }

                    let brb = core::str::jg(&m.gj).unwrap_or("");
                    self.bfc = brb.to_string();
                    self.ama = Some(due(brb));
                    self.bdv = amf;
                    self.ug = 0;
                    self.status = BrowserStatus::At;
                    self.ghi.clear();
                    Ok(())
                }
                Err(aa) => {
                    self.status = BrowserStatus::Q(alloc::format!("POST error: {}", aa));
                    Err("POST failed")
                }
            }
        } else {
            
            let xpc = if dqw.is_empty() {
                amf
            } else if amf.contains('?') {
                alloc::format!("{}&{}", amf, dqw)
            } else {
                alloc::format!("{}?{}", amf, dqw)
            };
            self.bvn(&xpc)
        }
    }

    
    pub fn zne(&mut self, j: &str, bn: &str) {
        self.ghi.insert(j.to_string(), bn.to_string());
    }

    
    pub fn ztt(&mut self, r: char) {
        if let Some(ref j) = self.kwq.clone() {
            let ap = self.ghi.bt(j.clone()).clq(String::new);
            ap.push(r);
        }
    }

    
    pub fn yfg(&mut self) {
        if let Some(ref j) = self.kwq.clone() {
            if let Some(ap) = self.ghi.ds(j) {
                ap.pop();
            }
        }
    }

    
    
    

    
    pub fn nrp(&mut self) {
        if let Some(ref doc) = self.ama {
            let gro = neu(&doc.xq);
            if !gro.is_empty() {
                let mut ohf = js_engine::JsContext::new();
                for eib in gro {
                    if let Err(aa) = ohf.bna(&eib) {
                        crate::serial_println!("[BROWSER JS ERROR] {}", aa);
                    }
                }
                self.ohh.lg(ohf.ffp);
            }
        }
    }
}


pub fn gnx(url: &str, ar: &str) -> String {
    let url = url.em();
    
    
    if url.cj("http://") || url.cj("https://") {
        return url.to_string();
    }
    
    
    if url.cj("//") {
        return alloc::format!("http:{}", url);
    }
    
    
    
    let tmg = url.contains('.');
    let sud = url.du('/');
    let stz = url.du('.');
    let twt = tmg && match (stz, sud) {
        (Some(bc), Some(e)) => bc < e,  
        (Some(_), None) => true,       
        _ => false,
    };
    
    if twt {
        return alloc::format!("http://{}", url);
    }
    
    
    let ar = if ar.is_empty() { "http://localhost/" } else { ar };
    let ar = ar.blj("http://").unwrap_or(ar);
    
    let (kh, fdd) = match ar.du('/') {
        Some(a) => (&ar[..a], &ar[a..]),
        None => (ar, "/"),
    };
    
    
    if url.cj('/') {
        return alloc::format!("http://{}{}", kh, url);
    }
    
    
    let gzr = match fdd.bhx('/') {
        Some(a) => &fdd[..=a],
        None => "/",
    };
    
    alloc::format!("http://{}{}{}", kh, gzr, url)
}



fn net(xq: &[HtmlNode]) -> Vec<(String, String)> {
    let mut bhw = Vec::new();
    
    for anq in xq {
        if let HtmlNode::Na(ij) = anq {
            match ij.ll.as_str() {
                "img" => {
                    if let Some(cy) = ij.qn("src") {
                        if !cy.is_empty() && !cy.cj("data:") {
                            bhw.push(("img".to_string(), cy.to_string()));
                        }
                    }
                }
                "link" => {
                    
                    let adj = ij.qn("rel").unwrap_or("");
                    if adj == "stylesheet" {
                        if let Some(cae) = ij.qn("href") {
                            if !cae.is_empty() {
                                bhw.push(("css".to_string(), cae.to_string()));
                            }
                        }
                    }
                    
                    if adj.contains("icon") {
                        if let Some(cae) = ij.qn("href") {
                            if !cae.is_empty() {
                                bhw.push(("icon".to_string(), cae.to_string()));
                            }
                        }
                    }
                }
                "script" => {
                    if let Some(cy) = ij.qn("src") {
                        if !cy.is_empty() {
                            bhw.push(("script".to_string(), cy.to_string()));
                        }
                    }
                }
                _ => {}
            }
            
            
            bhw.lg(net(&ij.zf));
        }
    }
    
    bhw
}

use alloc::string::Gd;


fn ggn(url: &str) -> String {
    let fbq = url.blj("https://")
        .or_else(|| url.blj("http://"))
        .unwrap_or(url);
    fbq.adk('/').next().unwrap_or("").adk(':').next().unwrap_or("").to_string()
}


fn sqe(url: &str) -> String {
    let fbq = url.blj("https://")
        .or_else(|| url.blj("http://"))
        .unwrap_or(url);
    match fbq.du('/') {
        Some(a) => fbq[a..].adk('?').next().unwrap_or("/").to_string(),
        None => "/".to_string(),
    }
}


fn moh(e: &str) -> String {
    let mut ckd = String::new();
    for o in e.bf() {
        if o.bvb() || b"-_.~".contains(&o) {
            ckd.push(o as char);
        } else if o == b' ' {
            ckd.push('+');
        } else {
            ckd.push('%');
            ckd.push(char::from(b"0123456789ABCDEF"[(o >> 4) as usize]));
            ckd.push(char::from(b"0123456789ABCDEF"[(o & 0xF) as usize]));
        }
    }
    ckd
}


fn vdr(dh: &str, rvb: &str) -> Option<Aqe> {
    let mut ek = dh.adk(';');
    let lnl = ek.next()?.em();
    let eq = lnl.du('=')?;
    let j = lnl[..eq].em().to_string();
    let bn = lnl[eq + 1..].em().to_string();
    
    if j.is_empty() { return None; }

    let mut dzu = Aqe {
        j,
        bn,
        vh: rvb.to_string(),
        path: "/".to_string(),
        hzi: false,
        ocj: false,
    };

    for qn in ek {
        let qn = qn.em().aqn();
        if qn.cj("domain=") {
            dzu.vh = qn[7..].tl('.').to_string();
        } else if qn.cj("path=") {
            dzu.path = qn[5..].to_string();
        } else if qn == "secure" {
            dzu.hzi = true;
        } else if qn == "httponly" {
            dzu.ocj = true;
        }
        
    }

    Some(dzu)
}


fn neu(xq: &[HtmlNode]) -> Vec<String> {
    let mut gro = Vec::new();
    for anq in xq {
        if let HtmlNode::Na(ij) = anq {
            if ij.ll == "script" && ij.qn("src").is_none() {
                let mut text = String::new();
                for aeh in &ij.zf {
                    if let HtmlNode::Text(ab) = aeh {
                        text.t(ab);
                    }
                }
                if !text.em().is_empty() {
                    gro.push(text);
                }
            }
            gro.lg(neu(&ij.zf));
        }
    }
    gro
}
