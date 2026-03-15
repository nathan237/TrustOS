












use alloc::string::String;
use alloc::vec::Vec;
use alloc::format;






#[derive(Clone)]
struct Afe {
    ak: Vec<String>,
    gn: usize,
    hn: usize,
}


#[derive(Clone)]
pub struct EditorState {
    
    pub ak: Vec<String>,
    
    pub gn: usize,
    
    pub hn: usize,
    
    pub ug: usize,
    
    pub cms: usize,
    
    pub wn: Option<String>,
    
    pub no: bool,
    
    pub eej: Language,
    
    pub ccb: Option<String>,
    
    pub byk: u32,
    
    bsu: Vec<Afe>,
    
    jlv: Vec<Afe>,
    
    pub amu: Option<(usize, usize)>,
    
    pub cqn: Option<String>,
    
    pub dbe: Option<String>,
    
    pub dhe: bool,
    
    pub bbq: Vec<(usize, usize)>,
    
    pub cep: usize,
    
    pub dri: Option<String>,
    
    pub hqt: Option<(usize, usize)>,
}

#[derive(Clone, Copy, PartialEq)]
pub enum Language {
    Adu,
    Rust,
    Aez,
    Akd,
    C,
    Aea,
    Acj,
}

impl Language {
    pub fn j(&self) -> &'static str {
        match self {
            Language::Adu => "Plain Text",
            Language::Rust => "Rust",
            Language::Aez => "TOML",
            Language::Akd => "Markdown",
            Language::C => "C/C++",
            Language::Aea => "Python",
            Language::Acj => "JavaScript",
        }
    }
    
    
    pub fn sxu(j: &str) -> Self {
        if j.pp(".rs") { Language::Rust }
        else if j.pp(".toml") { Language::Aez }
        else if j.pp(".md") { Language::Akd }
        else if j.pp(".c") || j.pp(".h") || j.pp(".cpp") { Language::C }
        else if j.pp(".py") { Language::Aea }
        else if j.pp(".js") || j.pp(".ts") { Language::Acj }
        else { Language::Adu }
    }
}

impl EditorState {
    pub fn new() -> Self {
        Self {
            ak: alloc::vec![String::new()],
            gn: 0,
            hn: 0,
            ug: 0,
            cms: 0,
            wn: None,
            no: false,
            eej: Language::Adu,
            ccb: None,
            byk: 0,
            bsu: Vec::new(),
            jlv: Vec::new(),
            amu: None,
            cqn: None,
            dbe: None,
            dhe: false,
            bbq: Vec::new(),
            cep: 0,
            dri: None,
            hqt: None,
        }
    }
    
    
    pub fn uhh(&mut self, text: &str) {
        self.ak.clear();
        for line in text.ak() {
            self.ak.push(String::from(line));
        }
        if self.ak.is_empty() {
            self.ak.push(String::new());
        }
        self.gn = 0;
        self.hn = 0;
        self.ug = 0;
        self.no = false;
    }
    
    
    pub fn dsu(&mut self, path: &str) {
        self.wn = Some(String::from(path));
        self.eej = Language::sxu(path);
        
        let wo = if path.cj('/') {
            String::from(path)
        } else {
            format!("/{}", path)
        };
        
        if let Ok(f) = crate::ramfs::fh(|fs| fs.mq(&wo).map(|bc| bc.ip())) {
            if let Ok(text) = core::str::jg(&f) {
                self.uhh(text);
            } else {
                self.ak = alloc::vec![String::from("(binary file — cannot edit)")];
            }
        } else {
            
            self.ak = alloc::vec![String::new()];
        }
        self.no = false;
    }
    
    
    pub fn mbr(&mut self) -> bool {
        if let Some(ref path) = self.wn {
            let wo = if path.cj('/') {
                String::from(path.as_str())
            } else {
                format!("/{}", path)
            };
            
            
            let mut text = String::new();
            for (a, line) in self.ak.iter().cf() {
                text.t(line);
                if a + 1 < self.ak.len() {
                    text.push('\n');
                }
            }
            
            let result = crate::ramfs::fh(|fs| {
                
                if fs.mq(&wo).is_err() {
                    let _ = fs.touch(&wo);
                }
                fs.ns(&wo, text.as_bytes())
            });
            
            if result.is_ok() {
                self.no = false;
                self.ccb = Some(format!("Saved: {}", path));
                crate::serial_println!("[TrustCode] Saved: {}", path);
                
                if let Ok(()) = crate::vfs::ns(&wo, text.as_bytes()) {
                    crate::serial_println!("[TrustCode] Persisted to disk: {}", path);
                }
                return true;
            } else {
                self.ccb = Some(format!("ERROR: Could not save {}", path));
            }
        } else {
            self.ccb = Some(String::from("No file path — use Ctrl+Shift+S"));
        }
        false
    }
    
    
    fn bgp(&self) -> &str {
        if self.gn < self.ak.len() {
            &self.ak[self.gn]
        } else {
            ""
        }
    }
    
    
    fn dow(&mut self) {
        let len = if self.gn < self.ak.len() {
            self.ak[self.gn].len()
        } else {
            0
        };
        if self.hn > len {
            self.hn = len;
        }
    }

    
    fn chn(&mut self) {
        self.bsu.push(Afe {
            ak: self.ak.clone(),
            gn: self.gn,
            hn: self.hn,
        });
        
        if self.bsu.len() > 200 {
            self.bsu.remove(0);
        }
        
        self.jlv.clear();
    }

    
    fn ifu(&mut self) {
        if let Some(cbx) = self.bsu.pop() {
            
            self.jlv.push(Afe {
                ak: self.ak.clone(),
                gn: self.gn,
                hn: self.hn,
            });
            self.ak = cbx.ak;
            self.gn = cbx.gn;
            self.hn = cbx.hn;
            self.no = true;
            self.ccb = Some(String::from("Undo"));
        }
    }

    
    fn vtm(&mut self) {
        if let Some(cbx) = self.jlv.pop() {
            
            self.bsu.push(Afe {
                ak: self.ak.clone(),
                gn: self.gn,
                hn: self.hn,
            });
            self.ak = cbx.ak;
            self.gn = cbx.gn;
            self.hn = cbx.hn;
            self.no = true;
            self.ccb = Some(String::from("Redo"));
        }
    }

    
    
    pub fn hll(&self) -> Option<(usize, usize, usize, usize)> {
        let (al, jyv) = self.amu?;
        let (bl, atw) = (self.gn, self.hn);
        if (al, jyv) <= (bl, atw) {
            Some((al, jyv, bl, atw))
        } else {
            Some((bl, atw, al, jyv))
        }
    }

    
    fn phs(&self) -> Option<String> {
        let (aks, jt, ij, ec) = self.hll()?;
        let mut result = String::new();
        for dm in aks..=ij {
            if dm >= self.ak.len() { break; }
            let line = &self.ak[dm];
            let ay = if dm == aks { jt.v(line.len()) } else { 0 };
            let ci = if dm == ij { ec.v(line.len()) } else { line.len() };
            if ay <= ci {
                result.t(&line[ay..ci]);
            }
            if dm < ij {
                result.push('\n');
            }
        }
        Some(result)
    }

    
    fn gej(&mut self) {
        if let Some((aks, jt, ij, ec)) = self.hll() {
            self.chn();
            if aks == ij {
                
                if aks < self.ak.len() {
                    let ci = ec.v(self.ak[aks].len());
                    let ay = jt.v(ci);
                    self.ak[aks] = format!("{}{}", &self.ak[aks][..ay], &self.ak[aks][ci..]);
                }
            } else {
                
                if ij < self.ak.len() {
                    let ddv = if ec <= self.ak[ij].len() {
                        String::from(&self.ak[ij][ec..])
                    } else {
                        String::new()
                    };
                    
                    let adx = if jt <= self.ak[aks].len() {
                        String::from(&self.ak[aks][..jt])
                    } else {
                        self.ak[aks].clone()
                    };
                    self.ak[aks] = format!("{}{}", adx, ddv);
                    
                    let vuv = ij - aks;
                    for _ in 0..vuv {
                        if aks + 1 < self.ak.len() {
                            self.ak.remove(aks + 1);
                        }
                    }
                }
            }
            self.gn = aks;
            self.hn = jt;
            self.amu = None;
            self.no = true;
        }
    }

    
    fn jus(&mut self) {
        self.bbq.clear();
        if let Some(ref query) = self.cqn {
            if query.is_empty() { return; }
            let fm = query.clone();
            for (atd, line) in self.ak.iter().cf() {
                let mut ay = 0;
                while ay + fm.len() <= line.len() {
                    if &line[ay..ay + fm.len()] == fm.as_str() {
                        self.bbq.push((atd, ay));
                        ay += fm.len().am(1);
                    } else {
                        ay += 1;
                    }
                }
            }
        }
    }

    
    fn kwf(&mut self) {
        if self.bbq.is_empty() { return; }
        self.cep = (self.cep + 1) % self.bbq.len();
        let (line, bj) = self.bbq[self.cep];
        self.gn = line;
        self.hn = bj;
        self.bdz();
    }

    
    fn yqs(&mut self) {
        if self.bbq.is_empty() { return; }
        if self.cep == 0 {
            self.cep = self.bbq.len() - 1;
        } else {
            self.cep -= 1;
        }
        let (line, bj) = self.bbq[self.cep];
        self.gn = line;
        self.hn = bj;
        self.bdz();
    }

    
    fn vxg(&mut self) {
        if self.bbq.is_empty() { return; }
        let w = self.cep.v(self.bbq.len() - 1);
        let (line, bj) = self.bbq[w];
        let query = self.cqn.clone();
        let lzi = self.dbe.clone();
        if let (Some(fm), Some(jmc)) = (query, lzi) {
            if line < self.ak.len() && bj + fm.len() <= self.ak[line].len() {
                self.chn();
                let jkq = fm.len();
                self.ak[line] = format!("{}{}{}", &self.ak[line][..bj], jmc, &self.ak[line][bj + jkq..]);
                self.no = true;
                self.jus();
                self.gn = line;
                self.hn = bj + jmc.len();
            }
        }
    }

    
    fn vxf(&mut self) {
        if self.bbq.is_empty() { return; }
        let query = self.cqn.clone();
        let lzi = self.dbe.clone();
        if let (Some(fm), Some(jmc)) = (query, lzi) {
            self.chn();
            for line in self.ak.el() {
                let mut result = String::new();
                let mut ay = 0;
                while ay < line.len() {
                    if ay + fm.len() <= line.len() && &line[ay..ay + fm.len()] == fm.as_str() {
                        result.t(&jmc);
                        ay += fm.len();
                    } else {
                        if let Some(bm) = line.as_bytes().get(ay) {
                            result.push(*bm as char);
                        }
                        ay += 1;
                    }
                }
                *line = result;
            }
            self.no = true;
            self.jus();
            self.ccb = Some(String::from("Replaced all"));
        }
    }
    
    
    pub fn zat(&self) -> usize {
        self.ak.len()
    }
    
    
    
    
    
    
    pub fn vr(&mut self, bs: u8) -> bool {
        use crate::keyboard::*;
        
        self.byk = 0; 
        
        
        if self.ccb.is_some() && bs != 0x13 { 
            
        }
        
        
        if self.cqn.is_some() {
            match bs {
                0x1B => { 
                    self.cqn = None;
                    self.dbe = None;
                    self.dhe = false;
                    self.bbq.clear();
                    return true;
                }
                0x0D | 0x0A => { 
                    if self.dhe {
                        if let Some(ref ycp) = self.dbe {
                            self.vxg();
                        }
                    } else {
                        self.kwf();
                    }
                    return true;
                }
                0x09 => { 
                    if self.dbe.is_some() {
                        self.dhe = !self.dhe;
                    }
                    return true;
                }
                0x08 => { 
                    if self.dhe {
                        if let Some(ref mut dbl) = self.dbe {
                            dbl.pop();
                        }
                    } else if let Some(ref mut fm) = self.cqn {
                        fm.pop();
                    }
                    self.jus();
                    return true;
                }
                0x01 => { 
                    self.vxf();
                    return true;
                }
                r if r >= 0x20 && r < 0x7F => {
                    if self.dhe {
                        if let Some(ref mut dbl) = self.dbe {
                            dbl.push(r as char);
                        }
                    } else if let Some(ref mut fm) = self.cqn {
                        fm.push(r as char);
                    }
                    self.jus();
                    
                    if !self.bbq.is_empty() {
                        self.cep = 0;
                        let (line, bj) = self.bbq[0];
                        self.gn = line;
                        self.hn = bj;
                        self.bdz();
                    }
                    return true;
                }
                _ => { return true; }
            }
        }
        
        
        if self.dri.is_some() {
            match bs {
                0x1B => { 
                    self.dri = None;
                    return true;
                }
                0x0D | 0x0A => { 
                    if let Some(ref input) = self.dri {
                        if let Ok(csd) = input.parse::<usize>() {
                            if csd > 0 && csd <= self.ak.len() {
                                self.gn = csd - 1;
                                self.hn = 0;
                                self.bdz();
                                self.ccb = Some(format!("Go to line {}", csd));
                            } else {
                                self.ccb = Some(format!("Invalid line (1-{})", self.ak.len()));
                            }
                        }
                    }
                    self.dri = None;
                    return true;
                }
                0x08 => { 
                    if let Some(ref mut input) = self.dri {
                        input.pop();
                    }
                    return true;
                }
                r if r >= b'0' && r <= b'9' => {
                    if let Some(ref mut input) = self.dri {
                        input.push(r as char);
                    }
                    return true;
                }
                _ => { return true; }
            }
        }
        
        let bsk = crate::keyboard::alh(0x2A) || crate::keyboard::alh(0x36);
        
        match bs {
            
            0x13 => {
                self.mbr();
                return true;
            }
            
            
            0x06 => {
                self.cqn = Some(String::new());
                self.dbe = None;
                self.dhe = false;
                self.bbq.clear();
                self.cep = 0;
                return true;
            }
            
            
            0x12 => {
                self.cqn = Some(String::new());
                self.dbe = Some(String::new());
                self.dhe = false;
                self.bbq.clear();
                self.cep = 0;
                return true;
            }
            
            
            0x1A => {
                self.ifu();
                self.bdz();
                return true;
            }
            
            
            0x19 => {
                self.vtm();
                self.bdz();
                return true;
            }
            
            
            0x07 => {
                self.dri = Some(String::new());
                return true;
            }
            
            
            AYA_ => {
                self.xit();
                return true;
            }
            
            
            AXZ_ => {
                self.rvh();
                return true;
            }
            
            
            AXY_ => {
                self.shi();
                return true;
            }
            
            
            AXV_ => {
                self.upv();
                return true;
            }
            
            
            AXU_ => {
                self.upu();
                return true;
            }
            
            
            AXW_ => {
                if bsk && self.amu.is_none() {
                    self.amu = Some((self.gn, self.hn));
                } else if !bsk {
                    self.amu = None;
                }
                self.xvc();
                self.bdz();
                return true;
            }
            
            
            AXX_ => {
                if bsk && self.amu.is_none() {
                    self.amu = Some((self.gn, self.hn));
                } else if !bsk {
                    self.amu = None;
                }
                self.xvd();
                self.bdz();
                return true;
            }
            
            
            0x01 => {
                self.amu = Some((0, 0));
                let gkv = self.ak.len().ao(1);
                let ucd = if gkv < self.ak.len() { self.ak[gkv].len() } else { 0 };
                self.gn = gkv;
                self.hn = ucd;
                self.ccb = Some(String::from("Select All"));
                return true;
            }
            
            
            0x03 => {
                if let Some(text) = self.phs() {
                    crate::keyboard::eno(&text);
                    self.ccb = Some(String::from("Copied"));
                }
                return true;
            }
            
            
            0x18 => {
                if let Some(text) = self.phs() {
                    crate::keyboard::eno(&text);
                    self.gej();
                    self.ccb = Some(String::from("Cut"));
                }
                return true;
            }
            
            
            0x16 => {
                if let Some(text) = crate::keyboard::ndn() {
                    
                    if self.amu.is_some() {
                        self.gej();
                    }
                    self.chn();
                    
                    for bm in text.bw() {
                        if bm == '\n' {
                            
                            if self.gn < self.ak.len() {
                                self.hn = self.hn.v(self.ak[self.gn].len());
                                let kr = self.ak[self.gn].pmk(self.hn);
                                self.gn += 1;
                                self.ak.insert(self.gn, kr);
                                self.hn = 0;
                            }
                        } else if bm >= ' ' && bm as u32 <= 0x7E {
                            if self.gn < self.ak.len() && self.hn <= self.ak[self.gn].len() {
                                self.ak[self.gn].insert(self.hn, bm);
                                self.hn += 1;
                            }
                        }
                    }
                    self.no = true;
                    self.ccb = Some(String::from("Pasted"));
                }
                self.bdz();
                return true;
            }
            
            
            V_ => {
                if bsk && self.amu.is_none() {
                    self.amu = Some((self.gn, self.hn));
                } else if !bsk {
                    self.amu = None;
                }
                if self.gn > 0 {
                    self.gn -= 1;
                    self.dow();
                }
                self.bdz();
                return true;
            }
            U_ => {
                if bsk && self.amu.is_none() {
                    self.amu = Some((self.gn, self.hn));
                } else if !bsk {
                    self.amu = None;
                }
                if self.gn + 1 < self.ak.len() {
                    self.gn += 1;
                    self.dow();
                }
                self.bdz();
                return true;
            }
            AH_ => {
                if bsk && self.amu.is_none() {
                    self.amu = Some((self.gn, self.hn));
                } else if !bsk {
                    self.amu = None;
                }
                if self.hn > 0 {
                    self.hn -= 1;
                } else if self.gn > 0 {
                    self.gn -= 1;
                    self.hn = self.ak[self.gn].len();
                }
                self.bdz();
                return true;
            }
            AI_ => {
                if bsk && self.amu.is_none() {
                    self.amu = Some((self.gn, self.hn));
                } else if !bsk {
                    self.amu = None;
                }
                let ark = if self.gn < self.ak.len() { self.ak[self.gn].len() } else { 0 };
                if self.hn < ark {
                    self.hn += 1;
                } else if self.gn + 1 < self.ak.len() {
                    self.gn += 1;
                    self.hn = 0;
                }
                self.bdz();
                return true;
            }
            CQ_ => {
                if bsk && self.amu.is_none() {
                    self.amu = Some((self.gn, self.hn));
                } else if !bsk {
                    self.amu = None;
                }
                self.hn = 0;
                return true;
            }
            CP_ => {
                if bsk && self.amu.is_none() {
                    self.amu = Some((self.gn, self.hn));
                } else if !bsk {
                    self.amu = None;
                }
                let ark = if self.gn < self.ak.len() { self.ak[self.gn].len() } else { 0 };
                self.hn = ark;
                return true;
            }
            AM_ => {
                let eeb = 20;
                self.gn = self.gn.ao(eeb);
                self.dow();
                self.bdz();
                return true;
            }
            AQ_ => {
                let eeb = 20;
                self.gn = (self.gn + eeb).v(self.ak.len().ao(1));
                self.dow();
                self.bdz();
                return true;
            }
            
            
            0x08 => {
                if self.amu.is_some() {
                    self.gej();
                    self.bdz();
                    return true;
                }
                self.chn();
                if self.hn > 0 && self.gn < self.ak.len() {
                    self.ak[self.gn].remove(self.hn - 1);
                    self.hn -= 1;
                    self.no = true;
                } else if self.gn > 0 {
                    
                    let cv = self.ak.remove(self.gn);
                    self.gn -= 1;
                    self.hn = self.ak[self.gn].len();
                    self.ak[self.gn].t(&cv);
                    self.no = true;
                }
                self.bdz();
                return true;
            }
            
            
            CX_ => {
                if self.amu.is_some() {
                    self.gej();
                    return true;
                }
                self.chn();
                if self.gn < self.ak.len() {
                    let ark = self.ak[self.gn].len();
                    if self.hn < ark {
                        self.ak[self.gn].remove(self.hn);
                        self.no = true;
                    } else if self.gn + 1 < self.ak.len() {
                        
                        let next = self.ak.remove(self.gn + 1);
                        self.ak[self.gn].t(&next);
                        self.no = true;
                    }
                }
                return true;
            }
            
            
            0x0D | 0x0A => {
                if self.amu.is_some() {
                    self.gej();
                }
                self.chn();
                if self.gn < self.ak.len() {
                    
                    self.hn = self.hn.v(self.ak[self.gn].len());
                    
                    let kr = self.ak[self.gn].pmk(self.hn);
                    
                    
                    let crn: String = self.ak[self.gn]
                        .bw()
                        .fwc(|r| *r == ' ' || *r == '\t')
                        .collect();
                    
                    
                    let ang = if self.ak[self.gn].eke().pp('{')
                        || self.ak[self.gn].eke().pp('(') {
                        "    "
                    } else {
                        ""
                    };
                    
                    let usz = format!("{}{}{}", crn, ang, kr);
                    self.gn += 1;
                    self.ak.insert(self.gn, usz);
                    self.hn = crn.len() + ang.len();
                    self.no = true;
                }
                self.bdz();
                return true;
            }
            
            
            0x09 => {
                self.chn();
                if let Some((aks, qdo, ij, qbq)) = self.hll() {
                    
                    if bsk {
                        
                        for dm in aks..=ij.v(self.ak.len().ao(1)) {
                            let eyr = self.ak[dm].bw().take(4).fwc(|r| *r == ' ').az();
                            if eyr > 0 {
                                self.ak[dm] = String::from(&self.ak[dm][eyr..]);
                            }
                        }
                    } else {
                        
                        for dm in aks..=ij.v(self.ak.len().ao(1)) {
                            self.ak[dm] = format!("    {}", self.ak[dm]);
                        }
                    }
                    self.no = true;
                } else if bsk {
                    
                    if self.gn < self.ak.len() {
                        let eyr = self.ak[self.gn].bw().take(4).fwc(|r| *r == ' ').az();
                        if eyr > 0 {
                            self.ak[self.gn] = String::from(&self.ak[self.gn][eyr..]);
                            self.hn = self.hn.ao(eyr);
                            self.no = true;
                        }
                    }
                } else {
                    
                    if self.gn < self.ak.len() {
                        for _ in 0..4 {
                            self.ak[self.gn].insert(self.hn, ' ');
                            self.hn += 1;
                        }
                        self.no = true;
                    }
                }
                return true;
            }
            
            
            r if r >= 0x20 && r < 0x7F => {
                if self.amu.is_some() {
                    self.gej();
                }
                self.chn();
                if self.gn < self.ak.len() {
                    let bm = r as char;
                    if self.hn <= self.ak[self.gn].len() {
                        self.ak[self.gn].insert(self.hn, bm);
                        self.hn += 1;
                        self.no = true;
                        
                        
                        let agj = match bm {
                            '{' => Some('}'),
                            '(' => Some(')'),
                            '[' => Some(']'),
                            '"' => Some('"'),
                            '\'' => Some('\''),
                            _ => None,
                        };
                        if let Some(rbv) = agj {
                            self.ak[self.gn].insert(self.hn, rbv);
                            
                        }
                    }
                }
                return true;
            }
            
            _ => {}
        }
        false
    }
    
    
    fn bdz(&mut self) {
        if self.gn < self.ug {
            self.ug = self.gn;
        }
        
        
        let iw = 30usize;
        if self.gn >= self.ug + iw {
            self.ug = self.gn - iw + 1;
        }
    }

    
    
    

    
    fn xit(&mut self) {
        self.chn();
        let gcz = match self.eej {
            Language::Rust | Language::C | Language::Acj => "// ",
            Language::Aea => "# ",
            Language::Aez => "# ",
            _ => "// ",
        };

        
        let (ay, ci) = if let Some((aks, qdo, ij, qbq)) = self.hll() {
            (aks, ij)
        } else {
            (self.gn, self.gn)
        };

        
        let muq = (ay..=ci.v(self.ak.len().ao(1)))
            .xx(|dm| self.ak[dm].ifa().cj(gcz.eke()));

        for dm in ay..=ci.v(self.ak.len().ao(1)) {
            if muq {
                
                let ux = &self.ak[dm];
                if let Some(u) = ux.du(gcz) {
                    self.ak[dm] = format!("{}{}", &ux[..u], &ux[u + gcz.len()..]);
                } else if let Some(u) = ux.du(gcz.eke()) {
                    
                    let vkq = gcz.eke();
                    self.ak[dm] = format!("{}{}", &ux[..u], &ux[u + vkq.len()..]);
                }
            } else {
                
                let oea = self.ak[dm].bw().fwc(|r| *r == ' ' || *r == '\t').az();
                self.ak[dm] = format!("{}{}{}", &self.ak[dm][..oea], gcz, &self.ak[dm][oea..]);
            }
        }
        self.no = true;
        self.ccb = Some(String::from(if muq { "Uncommented" } else { "Commented" }));
    }

    
    fn rvh(&mut self) {
        if self.ak.len() <= 1 {
            self.chn();
            self.ak[0] = String::new();
            self.hn = 0;
            self.no = true;
            return;
        }
        self.chn();
        self.ak.remove(self.gn);
        if self.gn >= self.ak.len() {
            self.gn = self.ak.len().ao(1);
        }
        self.dow();
        self.no = true;
        self.bdz();
    }

    
    fn shi(&mut self) {
        if self.gn < self.ak.len() {
            self.chn();
            let ksa = self.ak[self.gn].clone();
            self.ak.insert(self.gn + 1, ksa);
            self.gn += 1;
            self.no = true;
            self.bdz();
        }
    }

    
    fn upv(&mut self) {
        if self.gn > 0 && self.gn < self.ak.len() {
            self.chn();
            self.ak.swap(self.gn, self.gn - 1);
            self.gn -= 1;
            self.no = true;
            self.bdz();
        }
    }

    
    fn upu(&mut self) {
        if self.gn + 1 < self.ak.len() {
            self.chn();
            self.ak.swap(self.gn, self.gn + 1);
            self.gn += 1;
            self.no = true;
            self.bdz();
        }
    }

    
    fn xvc(&mut self) {
        if self.gn >= self.ak.len() { return; }
        if self.hn == 0 {
            
            if self.gn > 0 {
                self.gn -= 1;
                self.hn = self.ak[self.gn].len();
            }
            return;
        }
        let line = &self.ak[self.gn];
        let bf = line.as_bytes();
        let mut u = self.hn.v(bf.len());
        
        while u > 0 && bf[u - 1] == b' ' {
            u -= 1;
        }
        
        while u > 0 && bf[u - 1] != b' ' {
            u -= 1;
        }
        self.hn = u;
    }

    
    fn xvd(&mut self) {
        if self.gn >= self.ak.len() { return; }
        let line = &self.ak[self.gn];
        let bf = line.as_bytes();
        let len = bf.len();
        if self.hn >= len {
            
            if self.gn + 1 < self.ak.len() {
                self.gn += 1;
                self.hn = 0;
            }
            return;
        }
        let mut u = self.hn;
        
        while u < len && bf[u] != b' ' {
            u += 1;
        }
        
        while u < len && bf[u] == b' ' {
            u += 1;
        }
        self.hn = u;
    }

    
    pub fn xoy(&mut self) {
        self.hqt = None;
        if self.gn >= self.ak.len() { return; }
        let line = &self.ak[self.gn];
        if self.hn >= line.len() { return; }
        
        let bm = line.as_bytes()[self.hn];
        let (cd, fiz) = match bm {
            b'(' => (b')', true),
            b')' => (b'(', false),
            b'{' => (b'}', true),
            b'}' => (b'{', false),
            b'[' => (b']', true),
            b']' => (b'[', false),
            _ => return,
        };
        
        let mut eo: i32 = 0;
        if fiz {
            let mut dm = self.gn;
            let mut r = self.hn;
            while dm < self.ak.len() {
                let bf = self.ak[dm].as_bytes();
                while r < bf.len() {
                    if bf[r] == bm { eo += 1; }
                    else if bf[r] == cd {
                        eo -= 1;
                        if eo == 0 {
                            self.hqt = Some((dm, r));
                            return;
                        }
                    }
                    r += 1;
                }
                dm += 1;
                r = 0;
            }
        } else {
            let mut dm = self.gn;
            let mut r = self.hn as i32;
            loop {
                let bf = self.ak[dm].as_bytes();
                while r >= 0 {
                    let gdw = r as usize;
                    if gdw < bf.len() {
                        if bf[gdw] == bm { eo += 1; }
                        else if bf[gdw] == cd {
                            eo -= 1;
                            if eo == 0 {
                                self.hqt = Some((dm, gdw));
                                return;
                            }
                        }
                    }
                    r -= 1;
                }
                if dm == 0 { break; }
                dm -= 1;
                r = self.ak[dm].len() as i32 - 1;
            }
        }
    }
}






#[derive(Clone, Copy, PartialEq)]
pub enum TokenKind {
    M,
    Bx,
    Type,
    String,
    Ru,
    L,
    Bs,
    Akc,
    Ms,
    Blb,
    Fb,
    Aab,
}


pub struct W {
    pub ay: usize,
    pub ci: usize,
    pub kk: TokenKind,
}


pub const BNC_: u32   = 0xFF569CD6; 
pub const BNO_: u32      = 0xFF4EC9B0; 
pub const BNM_: u32    = 0xFFCE9178; 
pub const BNA_: u32   = 0xFF6A9955; 
pub const BNJ_: u32    = 0xFFB5CEA8; 
pub const BNB_: u32  = 0xFFDCDCAA; 
pub const BNF_: u32     = 0xFF4FC1FF; 
pub const BMW_: u32 = 0xFFD7BA7D; 
pub const BND_: u32  = 0xFF569CD6; 
pub const BNK_: u32  = 0xFFD4D4D4; 
pub const BMX_: u32   = 0xFFFFD700; 
pub const JQ_: u32    = 0xFFD4D4D4; 
pub const AOH_: u32  = 0xFF858585; 
pub const BMV_: u32 = 0xFF858585; 
pub const MF_: u32        = 0xFF1E1E2E; 
pub const AOE_: u32 = 0xFF1E1E2E; 
pub const AOB_: u32 = 0xFF2A2D3A; 
pub const AAF_: u32 = 0xFF007ACC; 
pub const GD_: u32 = 0xFFFFFFFF; 
pub const AOC_: u32    = 0xFFAEAFAD; 
pub const BMY_: u32 = 0xFF252526; 
pub const BNN_: u32 = 0xFF1E1E2E; 
pub const DFR_: u32 = 0xFF2D2D2D; 


const CPS_: &[&str] = &[
    "as", "async", "await", "break", "const", "continue", "crate", "dyn",
    "else", "enum", "extern", "false", "fn", "for", "if", "impl", "in",
    "let", "loop", "match", "mod", "move", "mut", "pub", "ref", "return",
    "self", "Self", "static", "struct", "super", "trait", "true", "type",
    "unsafe", "use", "where", "while", "yield",
];


const CPT_: &[&str] = &[
    "bool", "char", "f32", "f64", "i8", "i16", "i32", "i64", "i128", "isize",
    "u8", "u16", "u32", "u64", "u128", "usize", "str", "String", "Vec",
    "Option", "Result", "Box", "Rc", "Arc", "Cell", "RefCell", "Mutex",
    "HashMap", "HashSet", "BTreeMap", "BTreeSet", "Cow", "Pin",
    "Some", "None", "Ok", "Err",
];


pub fn puh(line: &str) -> Vec<W> {
    let mut ui = Vec::new();
    let bf = line.as_bytes();
    let len = bf.len();
    let mut a = 0;
    
    while a < len {
        let bm = bf[a];
        
        
        if bm == b'/' && a + 1 < len && bf[a + 1] == b'/' {
            ui.push(W { ay: a, ci: len, kk: TokenKind::Ru });
            break; 
        }
        
        
        if bm == b'#' && a + 1 < len && bf[a + 1] == b'[' {
            let ay = a;
            
            while a < len && bf[a] != b']' { a += 1; }
            if a < len { a += 1; } 
            ui.push(W { ay, ci: a, kk: TokenKind::Ms });
            continue;
        }
        
        
        if bm == b'"' {
            let ay = a;
            a += 1;
            while a < len {
                if bf[a] == b'\\' && a + 1 < len {
                    a += 2; 
                } else if bf[a] == b'"' {
                    a += 1;
                    break;
                } else {
                    a += 1;
                }
            }
            ui.push(W { ay, ci: a, kk: TokenKind::String });
            continue;
        }
        
        
        if bm == b'\'' {
            
            let ay = a;
            a += 1;
            if a < len && bf[a].gke() {
                
                let zwm = a;
                while a < len && (bf[a].bvb() || bf[a] == b'_') {
                    a += 1;
                }
                if a < len && bf[a] == b'\'' {
                    
                    a += 1;
                    ui.push(W { ay, ci: a, kk: TokenKind::String });
                } else {
                    
                    ui.push(W { ay, ci: a, kk: TokenKind::Blb });
                }
            } else if a < len && bf[a] == b'\\' {
                
                while a < len && bf[a] != b'\'' { a += 1; }
                if a < len { a += 1; }
                ui.push(W { ay, ci: a, kk: TokenKind::String });
            } else {
                ui.push(W { ay, ci: ay + 1, kk: TokenKind::M });
            }
            continue;
        }
        
        
        if bm.atb() || (bm == b'0' && a + 1 < len && (bf[a+1] == b'x' || bf[a+1] == b'b' || bf[a+1] == b'o')) {
            let ay = a;
            a += 1;
            while a < len && (bf[a].bvb() || bf[a] == b'_' || bf[a] == b'.') {
                a += 1;
            }
            ui.push(W { ay, ci: a, kk: TokenKind::L });
            continue;
        }
        
        
        if bm.gke() || bm == b'_' {
            let ay = a;
            while a < len && (bf[a].bvb() || bf[a] == b'_') {
                a += 1;
            }
            let od = &line[ay..a];
            
            
            if a < len && bf[a] == b'!' {
                ui.push(W { ay, ci: a + 1, kk: TokenKind::Akc });
                a += 1;
                continue;
            }
            
            
            let lgd = a < len && bf[a] == b'(';
            
            let kk = if CPS_.contains(&od) {
                TokenKind::Bx
            } else if CPT_.contains(&od) {
                TokenKind::Type
            } else if lgd {
                TokenKind::Bs
            } else {
                TokenKind::M
            };
            
            ui.push(W { ay, ci: a, kk });
            continue;
        }
        
        
        if bm == b'{' || bm == b'}' || bm == b'(' || bm == b')' || bm == b'[' || bm == b']' {
            ui.push(W { ay: a, ci: a + 1, kk: TokenKind::Aab });
            a += 1;
            continue;
        }
        
        
        if bm == b'+' || bm == b'-' || bm == b'*' || bm == b'=' || bm == b'!' 
            || bm == b'<' || bm == b'>' || bm == b'&' || bm == b'|' || bm == b'^'
            || bm == b':' || bm == b';' || bm == b',' || bm == b'.' {
            ui.push(W { ay: a, ci: a + 1, kk: TokenKind::Fb });
            a += 1;
            continue;
        }
        
        
        let ay = a;
        a += 1;
        ui.push(W { ay, ci: a, kk: TokenKind::M });
    }
    
    ui
}


pub fn puf(kk: TokenKind) -> u32 {
    match kk {
        TokenKind::M => JQ_,
        TokenKind::Bx => BNC_,
        TokenKind::Type => BNO_,
        TokenKind::String => BNM_,
        TokenKind::Ru => BNA_,
        TokenKind::L => BNJ_,
        TokenKind::Bs => BNB_,
        TokenKind::Akc => BNF_,
        TokenKind::Ms => BMW_,
        TokenKind::Blb => BND_,
        TokenKind::Fb => BNK_,
        TokenKind::Aab => BMX_,
    }
}





const CMT_: &[&str] = &[
    "False", "None", "True", "and", "as", "assert", "async", "await",
    "break", "class", "continue", "def", "del", "elif", "else", "except",
    "finally", "for", "from", "global", "if", "import", "in", "is",
    "lambda", "nonlocal", "not", "or", "pass", "raise", "return", "try",
    "while", "with", "yield",
];

const CMS_: &[&str] = &[
    "int", "float", "str", "bool", "list", "dict", "tuple", "set",
    "frozenset", "bytes", "bytearray", "range", "type", "object",
    "print", "len", "input", "open", "super", "self", "cls",
    "Exception", "ValueError", "TypeError", "KeyError", "IndexError",
    "RuntimeError", "StopIteration", "OSError", "IOError",
];

pub fn xjh(line: &str) -> Vec<W> {
    let mut ui = Vec::new();
    let bf = line.as_bytes();
    let len = bf.len();
    let mut a = 0;
    
    while a < len {
        let bm = bf[a];
        
        
        if bm == b'#' {
            ui.push(W { ay: a, ci: len, kk: TokenKind::Ru });
            break;
        }
        
        
        if bm == b'@' {
            let ay = a;
            a += 1;
            while a < len && (bf[a].bvb() || bf[a] == b'_' || bf[a] == b'.') { a += 1; }
            ui.push(W { ay, ci: a, kk: TokenKind::Ms });
            continue;
        }
        
        
        if bm == b'"' || bm == b'\'' {
            let ay = a;
            let cgw = bm;
            
            if a + 2 < len && bf[a+1] == cgw && bf[a+2] == cgw {
                a += 3;
                while a + 2 < len {
                    if bf[a] == cgw && bf[a+1] == cgw && bf[a+2] == cgw {
                        a += 3;
                        break;
                    }
                    if bf[a] == b'\\' { a += 1; }
                    a += 1;
                }
                if a >= len { a = len; }
            } else {
                a += 1;
                while a < len {
                    if bf[a] == b'\\' && a + 1 < len { a += 2; continue; }
                    if bf[a] == cgw { a += 1; break; }
                    a += 1;
                }
            }
            ui.push(W { ay, ci: a, kk: TokenKind::String });
            continue;
        }
        
        
        if bm.atb() {
            let ay = a;
            a += 1;
            while a < len && (bf[a].bvb() || bf[a] == b'_' || bf[a] == b'.') { a += 1; }
            ui.push(W { ay, ci: a, kk: TokenKind::L });
            continue;
        }
        
        
        if bm.gke() || bm == b'_' {
            let ay = a;
            while a < len && (bf[a].bvb() || bf[a] == b'_') { a += 1; }
            let od = &line[ay..a];
            let lgc = a < len && bf[a] == b'(';
            let kk = if CMT_.contains(&od) {
                TokenKind::Bx
            } else if CMS_.contains(&od) {
                TokenKind::Type
            } else if lgc {
                TokenKind::Bs
            } else {
                TokenKind::M
            };
            ui.push(W { ay, ci: a, kk });
            continue;
        }
        
        
        if bm == b'(' || bm == b')' || bm == b'{' || bm == b'}' || bm == b'[' || bm == b']' {
            ui.push(W { ay: a, ci: a + 1, kk: TokenKind::Aab });
            a += 1;
            continue;
        }
        
        
        if bm == b'+' || bm == b'-' || bm == b'*' || bm == b'/' || bm == b'=' || bm == b'!'
            || bm == b'<' || bm == b'>' || bm == b'&' || bm == b'|' || bm == b'^'
            || bm == b':' || bm == b';' || bm == b',' || bm == b'.' || bm == b'%' {
            ui.push(W { ay: a, ci: a + 1, kk: TokenKind::Fb });
            a += 1;
            continue;
        }
        
        a += 1;
    }
    ui
}





const CCR_: &[&str] = &[
    "break", "case", "catch", "class", "const", "continue", "debugger",
    "default", "delete", "do", "else", "export", "extends", "finally",
    "for", "function", "if", "import", "in", "instanceof", "let", "new",
    "of", "return", "super", "switch", "this", "throw", "try", "typeof",
    "var", "void", "while", "with", "yield", "async", "await", "from",
    "static", "get", "set",
];

const CCS_: &[&str] = &[
    "Array", "Boolean", "Date", "Error", "Function", "JSON", "Map", "Math",
    "Number", "Object", "Promise", "Proxy", "RegExp", "Set", "String",
    "Symbol", "WeakMap", "WeakSet", "console", "document", "window",
    "null", "undefined", "true", "false", "NaN", "Infinity",
];

const BQW_: &[&str] = &[
    "auto", "break", "case", "char", "const", "continue", "default", "do",
    "double", "else", "enum", "extern", "float", "for", "goto", "if",
    "inline", "int", "long", "register", "restrict", "return", "short",
    "signed", "sizeof", "static", "struct", "switch", "typedef", "union",
    "unsigned", "void", "volatile", "while",
    
    "bool", "catch", "class", "constexpr", "delete", "dynamic_cast",
    "explicit", "false", "friend", "mutable", "namespace", "new",
    "noexcept", "nullptr", "operator", "override", "private", "protected",
    "public", "reinterpret_cast", "static_assert", "static_cast",
    "template", "this", "throw", "true", "try", "typeid", "typename",
    "using", "virtual",
];

const BQX_: &[&str] = &[
    "int8_t", "int16_t", "int32_t", "int64_t", "uint8_t", "uint16_t",
    "uint32_t", "uint64_t", "size_t", "ssize_t", "ptrdiff_t", "intptr_t",
    "uintptr_t", "FILE", "NULL", "EOF", "string", "vector", "map",
    "set", "pair", "shared_ptr", "unique_ptr", "weak_ptr",
];

pub fn pug(line: &str, hou: bool) -> Vec<W> {
    let mut ui = Vec::new();
    let bf = line.as_bytes();
    let len = bf.len();
    let mut a = 0;
    let fmj: &[&str] = if hou { BQW_ } else { CCR_ };
    let ifn: &[&str] = if hou { BQX_ } else { CCS_ };
    
    while a < len {
        let bm = bf[a];
        
        
        if bm == b'/' && a + 1 < len && bf[a + 1] == b'/' {
            ui.push(W { ay: a, ci: len, kk: TokenKind::Ru });
            break;
        }
        
        
        if bm == b'/' && a + 1 < len && bf[a + 1] == b'*' {
            let ay = a;
            a += 2;
            while a + 1 < len {
                if bf[a] == b'*' && bf[a+1] == b'/' { a += 2; break; }
                a += 1;
            }
            if a >= len { a = len; }
            ui.push(W { ay, ci: a, kk: TokenKind::Ru });
            continue;
        }
        
        
        if hou && bm == b'#' {
            ui.push(W { ay: a, ci: len, kk: TokenKind::Akc });
            break;
        }
        
        
        if bm == b'"' || bm == b'\'' || bm == b'`' {
            let ay = a;
            let cgw = bm;
            a += 1;
            while a < len {
                if bf[a] == b'\\' && a + 1 < len { a += 2; continue; }
                if bf[a] == cgw { a += 1; break; }
                a += 1;
            }
            ui.push(W { ay, ci: a, kk: TokenKind::String });
            continue;
        }
        
        
        if bm.atb() || (bm == b'.' && a + 1 < len && bf[a+1].atb()) {
            let ay = a;
            a += 1;
            while a < len && (bf[a].bvb() || bf[a] == b'_' || bf[a] == b'.') { a += 1; }
            ui.push(W { ay, ci: a, kk: TokenKind::L });
            continue;
        }
        
        
        if bm.gke() || bm == b'_' || bm == b'$' {
            let ay = a;
            while a < len && (bf[a].bvb() || bf[a] == b'_' || bf[a] == b'$') { a += 1; }
            let od = &line[ay..a];
            let lgc = a < len && bf[a] == b'(';
            let kk = if fmj.contains(&od) {
                TokenKind::Bx
            } else if ifn.contains(&od) {
                TokenKind::Type
            } else if lgc {
                TokenKind::Bs
            } else {
                TokenKind::M
            };
            ui.push(W { ay, ci: a, kk });
            continue;
        }
        
        
        if bm == b'(' || bm == b')' || bm == b'{' || bm == b'}' || bm == b'[' || bm == b']' {
            ui.push(W { ay: a, ci: a + 1, kk: TokenKind::Aab });
            a += 1;
            continue;
        }
        
        
        if bm == b'+' || bm == b'-' || bm == b'*' || bm == b'/' || bm == b'=' || bm == b'!'
            || bm == b'<' || bm == b'>' || bm == b'&' || bm == b'|' || bm == b'^'
            || bm == b':' || bm == b';' || bm == b',' || bm == b'.' || bm == b'?' || bm == b'%' {
            ui.push(W { ay: a, ci: a + 1, kk: TokenKind::Fb });
            a += 1;
            continue;
        }
        
        a += 1;
    }
    ui
}





pub fn xji(line: &str) -> Vec<W> {
    let mut ui = Vec::new();
    let bf = line.as_bytes();
    let len = bf.len();
    let ux = line.ifa();
    
    
    if ux.cj('#') {
        ui.push(W { ay: 0, ci: len, kk: TokenKind::Ru });
        return ui;
    }
    
    
    if ux.cj('[') {
        let l = len - ux.len();
        ui.push(W { ay: l, ci: len, kk: TokenKind::Ms });
        return ui;
    }
    
    
    let mut a = 0;
    
    while a < len && bf[a] != b'=' {
        a += 1;
    }
    if a < len {
        
        ui.push(W { ay: 0, ci: a, kk: TokenKind::Type });
        
        ui.push(W { ay: a, ci: a + 1, kk: TokenKind::Fb });
        a += 1;
        
        while a < len && bf[a] == b' ' { a += 1; }
        
        if a < len {
            let ekl = a;
            let fay = bf[a];
            if fay == b'"' || fay == b'\'' {
                
                ui.push(W { ay: ekl, ci: len, kk: TokenKind::String });
            } else if fay == b't' || fay == b'f' {
                
                ui.push(W { ay: ekl, ci: len, kk: TokenKind::Bx });
            } else if fay.atb() || fay == b'-' || fay == b'+' {
                
                ui.push(W { ay: ekl, ci: len, kk: TokenKind::L });
            } else if fay == b'[' {
                
                ui.push(W { ay: ekl, ci: len, kk: TokenKind::Aab });
            } else {
                ui.push(W { ay: ekl, ci: len, kk: TokenKind::M });
            }
            
            if let Some(tnm) = line[ekl..].du('#') {
                let qep = ekl + tnm;
                ui.push(W { ay: qep, ci: len, kk: TokenKind::Ru });
            }
        }
    } else {
        
        ui.push(W { ay: 0, ci: len, kk: TokenKind::M });
    }
    ui
}





pub const DFN_: u32 = 0xFF569CD6;  
pub const DFL_: u32    = 0xFFD7BA7D;   
pub const DFM_: u32    = 0xFFCE9178;    
pub const DFO_: u32    = 0xFF4EC9B0;    
pub const DFP_: u32    = 0xFF569CD6;    

pub fn xjg(line: &str) -> Vec<W> {
    let mut ui = Vec::new();
    let bf = line.as_bytes();
    let len = bf.len();
    let ux = line.ifa();
    
    
    if ux.cj('#') {
        ui.push(W { ay: 0, ci: len, kk: TokenKind::Bx });
        return ui;
    }
    
    
    if ux.cj("```") {
        ui.push(W { ay: 0, ci: len, kk: TokenKind::String });
        return ui;
    }
    
    
    if ux.cj("- ") || ux.cj("* ") || ux.cj("+ ") {
        let l = len - ux.len();
        ui.push(W { ay: l, ci: l + 2, kk: TokenKind::Akc });
        
        if l + 2 < len {
            ui.push(W { ay: l + 2, ci: len, kk: TokenKind::M });
        }
        return ui;
    }
    
    
    let mut a = 0;
    let mut diu = 0;
    
    while a < len {
        
        if bf[a] == b'`' {
            if diu < a {
                ui.push(W { ay: diu, ci: a, kk: TokenKind::M });
            }
            let ay = a;
            a += 1;
            while a < len && bf[a] != b'`' { a += 1; }
            if a < len { a += 1; }
            ui.push(W { ay, ci: a, kk: TokenKind::String });
            diu = a;
            continue;
        }
        
        
        if bf[a] == b'*' && a + 1 < len && bf[a+1] == b'*' {
            if diu < a {
                ui.push(W { ay: diu, ci: a, kk: TokenKind::M });
            }
            let ay = a;
            a += 2;
            while a + 1 < len {
                if bf[a] == b'*' && bf[a+1] == b'*' { a += 2; break; }
                a += 1;
            }
            ui.push(W { ay, ci: a, kk: TokenKind::Ms });
            diu = a;
            continue;
        }
        
        
        if bf[a] == b'[' {
            if diu < a {
                ui.push(W { ay: diu, ci: a, kk: TokenKind::M });
            }
            let ay = a;
            while a < len && bf[a] != b')' { a += 1; }
            if a < len { a += 1; }
            ui.push(W { ay, ci: a, kk: TokenKind::Type });
            diu = a;
            continue;
        }
        
        a += 1;
    }
    
    if diu < len {
        ui.push(W { ay: diu, ci: len, kk: TokenKind::M });
    }
    
    ui
}


pub fn xjf(line: &str, sh: Language) -> Vec<W> {
    match sh {
        Language::Rust => puh(line),
        Language::Aea => xjh(line),
        Language::Acj => pug(line, false),
        Language::C => pug(line, true),
        Language::Aez => xji(line),
        Language::Akd => xjg(line),
        Language::Adu => Vec::new(),
    }
}







pub fn ehm(
    g: &mut EditorState,
    b: i32, c: i32, d: u32, i: u32,
    acd: &dyn Fn(i32, i32, &str, u32),
    ymw: &dyn Fn(i32, i32, char, u32),
) {
    let nk: i32 = 8;
    let gy: i32 = 16;
    let bfm: i32 = 22;
    let jfw: i32 = 22;
    let dwn: i32 = 28;
    let kev: i32 = 18;
    let jts = jfw + dwn + kev;
    
    
    let nlm = if g.ak.len() >= 10000 { 5 }
        else if g.ak.len() >= 1000 { 4 }
        else if g.ak.len() >= 100 { 3 }
        else { 2 };
    let bqy = (nlm + 2) * nk;
    
    let bds = b + bqy;
    let bdt = c + jts;
    let ior = d as i32 - bqy;
    let byr = i as i32 - jts - bfm;
    let act = (byr / gy).am(1) as usize;
    
    
    if g.gn < g.ug {
        g.ug = g.gn;
    }
    if g.gn >= g.ug + act {
        g.ug = g.gn - act + 1;
    }
    g.byk += 1;
    g.xoy();
    
    
    let xp = c;
    crate::framebuffer::ah(b as u32, xp as u32, d, jfw as u32, 0xFF333333);
    
    crate::framebuffer::ah(b as u32, (xp + jfw - 1) as u32, d, 1, 0xFF252526);
    let une = ["File", "Edit", "Selection", "View", "Go", "Run", "Terminal", "Help"];
    let mut hl = b + 8;
    for cu in &une {
        acd(hl, xp + 4, cu, 0xFFCCCCCC);
        hl += (cu.len() as i32 + 2) * nk;
    }
    
    
    let bxl = c + jfw;
    crate::framebuffer::ah(b as u32, bxl as u32, d, dwn as u32, 0xFF252526);
    
    let mjp = g.wn.as_ref().map(|ai| {
        ai.cmm('/').next().unwrap_or(ai.as_str())
    }).unwrap_or("untitled");
    let kqc = if g.no { " *" } else { "" };
    
    let ubw = match g.eej {
        Language::Rust => "RS",
        Language::Aea => "PY",
        Language::Acj => "JS",
        Language::C => " C",
        Language::Aez => "TL",
        Language::Akd => "MD",
        Language::Adu => "  ",
    };
    let icv = format!(" {} {} {}  x", ubw, mjp, kqc);
    let axb = ((icv.len() as u32) * 8 + 4).v(d);
    
    crate::framebuffer::ah(b as u32, bxl as u32, axb, dwn as u32, MF_);
    
    crate::framebuffer::ah(b as u32, bxl as u32, axb, 2, 0xFF007ACC);
    
    acd(b + 4, bxl + 8, &icv, JQ_);
    
    
    let gzz = bxl + dwn;
    crate::framebuffer::ah(b as u32, gzz as u32, d, kev as u32, 0xFF1E1E1E);
    if let Some(ref path) = g.wn {
        
        let mut bx = b + bqy + 4;
        let mut ay = 0;
        let bf = path.as_bytes();
        for a in 0..=bf.len() {
            if a == bf.len() || bf[a] == b'/' {
                if a > ay {
                    let vu = &path[ay..a];
                    if ay > 0 {
                        acd(bx, gzz + 2, " > ", 0xFF666666);
                        bx += 3 * nk;
                    }
                    acd(bx, gzz + 2, vu, 0xFF858585);
                    bx += vu.len() as i32 * nk;
                }
                ay = a + 1;
            }
        }
    } else {
        acd(b + bqy + 4, gzz + 2, "untitled", 0xFF858585);
    }
    
    crate::framebuffer::ah(b as u32, (gzz + kev - 1) as u32, d, 1, 0xFF333333);
    
    
    crate::framebuffer::ah(b as u32, bdt as u32, d, byr as u32, MF_);
    
    
    crate::framebuffer::ah(b as u32, bdt as u32, bqy as u32, byr as u32, AOE_);
    crate::framebuffer::ah((b + bqy - 1) as u32, bdt as u32, 1, byr as u32, 0xFF333333);
    
    
    for afj in 0..act {
        let atd = g.ug + afj;
        if atd >= g.ak.len() { break; }
        
        let ct = bdt + (afj as i32 * gy);
        if ct + gy > bdt + byr { break; }
        
        let jbd = atd == g.gn;
        
        
        if jbd {
            crate::framebuffer::ah(
                b as u32, ct as u32,
                bqy as u32, gy as u32,
                0xFF1A1D26,
            );
            crate::framebuffer::ah(
                bds as u32, ct as u32,
                ior as u32, gy as u32,
                AOB_,
            );
        }
        
        
        if let Some((aks, jt, ij, ec)) = g.hll() {
            if atd >= aks && atd <= ij {
                let ark = g.ak[atd].len();
                let grx = if atd == aks { jt.v(ark) } else { 0 };
                let jod = if atd == ij { ec.v(ark) } else { ark };
                if grx < jod {
                    let cr = bds + 4 + (grx as i32 * nk);
                    let kp = ((jod - grx) as i32 * nk) as u32;
                    crate::framebuffer::ah(
                        cr as u32, ct as u32, kp, gy as u32, 0xFF264F78,
                    );
                }
            }
        }
        
        
        let ufa = &g.ak[atd];
        let mut oeb = 0usize;
        for o in ufa.bf() {
            if o == b' ' { oeb += 1; } else { break; }
        }
        let tss = oeb / 4;
        for jy in 0..tss {
            let nzz = bds + 4 + (jy as i32 * 4 * nk);
            if nzz < b + d as i32 {
                let tig = if jbd { 0xFF505050 } else { 0xFF404040 };
                crate::framebuffer::ah(nzz as u32, ct as u32, 1, gy as u32, tig);
            }
        }
        
        
        if let Some((gmt, jfp)) = g.hqt {
            if atd == gmt {
                let bx = bds + 4 + (jfp as i32 * nk);
                crate::framebuffer::ah(bx as u32, ct as u32, nk as u32, gy as u32, 0xFF3A3D41);
                crate::framebuffer::ah(bx as u32, ct as u32, nk as u32, 1, 0xFF888888);
                crate::framebuffer::ah(bx as u32, (ct + gy - 1) as u32, nk as u32, 1, 0xFF888888);
                crate::framebuffer::ah(bx as u32, ct as u32, 1, gy as u32, 0xFF888888);
                crate::framebuffer::ah((bx + nk - 1) as u32, ct as u32, 1, gy as u32, 0xFF888888);
            }
            if atd == g.gn && g.hn < g.ak[atd].len() {
                let aiv = g.ak[atd].as_bytes()[g.hn];
                if oh!(aiv, b'(' | b')' | b'{' | b'}' | b'[' | b']') {
                    let hck = bds + 4 + (g.hn as i32 * nk);
                    crate::framebuffer::ah(hck as u32, ct as u32, nk as u32, gy as u32, 0xFF3A3D41);
                    crate::framebuffer::ah(hck as u32, ct as u32, nk as u32, 1, 0xFF888888);
                    crate::framebuffer::ah(hck as u32, (ct + gy - 1) as u32, nk as u32, 1, 0xFF888888);
                    crate::framebuffer::ah(hck as u32, ct as u32, 1, gy as u32, 0xFF888888);
                    crate::framebuffer::ah((hck + nk - 1) as u32, ct as u32, 1, gy as u32, 0xFF888888);
                }
            }
        }
        
        
        let uey = format!("{:>width$} ", atd + 1, z = nlm as usize);
        let htc = if jbd { 0xFFC6C6C6 } else { AOH_ };
        acd(b + 2, ct, &uey, htc);
        
        
        let line = &g.ak[atd];
        let eb = xjf(line, g.eej);
        if !eb.is_empty() {
            for dlx in &eb {
                let s = puf(dlx.kk);
                let xga = &line[dlx.ay..dlx.ci];
                let cr = bds + 4 + (dlx.ay as i32 * nk);
                if cr < b + d as i32 {
                    acd(cr, ct, xga, s);
                }
            }
        } else if !line.is_empty() {
            acd(bds + 4, ct, line, JQ_);
        }
        
        
        if jbd {
            let kdv = (g.byk / 30) % 2 == 0;
            if kdv {
                let cx = bds + 4 + (g.hn as i32 * nk);
                crate::framebuffer::ah(cx as u32, ct as u32, 2, gy as u32, AOC_);
            }
        }
    }
    
    
    if g.ak.len() > act {
        let auz = (b + d as i32 - 10) as u32;
        let dbr = byr as u32;
        let axd = ((act as u32 * dbr) / g.ak.len() as u32).am(20);
        let bsm = (g.ug as u32 * (dbr - axd)) / g.ak.len().ao(act) as u32;
        crate::framebuffer::ah(auz + 3, bdt as u32, 7, dbr, 0xFF252526);
        crate::framebuffer::afp(auz + 3, bdt as u32 + bsm, 7, axd, 3, 0xFF6A6A6A);
    }
    
    
    if g.cqn.is_some() {
        let nud: i32 = if g.dbe.is_some() { 56 } else { 32 };
        let ghb: i32 = 370.v(ior);
        let hjo = b + d as i32 - ghb - 20;
        let hjp = bdt + 4;
        
        crate::framebuffer::ah((hjo + 2) as u32, (hjp + 2) as u32, ghb as u32, nud as u32, 0xFF0A0A0A);
        crate::framebuffer::ah(hjo as u32, hjp as u32, ghb as u32, nud as u32, 0xFF252526);
        crate::framebuffer::ah(hjo as u32, hjp as u32, ghb as u32, 1, 0xFF007ACC);
        
        let query = g.cqn.ahz().unwrap_or("");
        let uki = if g.bbq.is_empty() {
            if query.is_empty() { String::new() } else { String::from(" No results") }
        } else {
            format!(" {}/{}", g.cep + 1, g.bbq.len())
        };
        let nuc = !g.dhe;
        let ggw = hjo + 8;
        let iuh = hjp + 6;
        let iug = ghb - 100;
        crate::framebuffer::ah(ggw as u32, iuh as u32, iug as u32, 18, if nuc { 0xFF3C3C3C } else { 0xFF333333 });
        if nuc {
            crate::framebuffer::ah(ggw as u32, (iuh + 17) as u32, iug as u32, 1, 0xFF007ACC);
        }
        acd(ggw + 4, iuh + 2, query, 0xFFCCCCCC);
        acd(hjo + ghb - 90, iuh + 2, &uki, 0xFF858585);
        
        if let Some(ref replace) = g.dbe {
            let maa = hjp + 30;
            let pcg = g.dhe;
            crate::framebuffer::ah(ggw as u32, maa as u32, iug as u32, 18, if pcg { 0xFF3C3C3C } else { 0xFF333333 });
            if pcg {
                crate::framebuffer::ah(ggw as u32, (maa + 17) as u32, iug as u32, 1, 0xFF007ACC);
            }
            acd(ggw + 4, maa + 2, replace, 0xFFCCCCCC);
        }
        
        
        let jkq = query.len();
        if jkq > 0 {
            for &(gmt, jfp) in &g.bbq {
                if gmt >= g.ug && gmt < g.ug + act {
                    let afj = gmt - g.ug;
                    let lmc = bdt + (afj as i32 * gy);
                    let lmh = bds + 4 + (jfp as i32 * nk);
                    let efp = (jkq as i32 * nk) as u32;
                    crate::framebuffer::ah(lmh as u32, lmc as u32, efp, gy as u32, 0xFF613214);
                    if g.cep < g.bbq.len() && g.bbq[g.cep] == (gmt, jfp) {
                        crate::framebuffer::ah(lmh as u32, lmc as u32, efp, 1, 0xFFE8AB53);
                        crate::framebuffer::ah(lmh as u32, (lmc + gy - 1) as u32, efp, 1, 0xFFE8AB53);
                    }
                }
            }
        }
    }
    
    
    if let Some(ref input) = g.dri {
        let fgr: i32 = 320.v(d as i32 - 40);
        let nlg: i32 = 32;
        let fgs = b + (d as i32 - fgr) / 2;
        let irb = c + jts + 2;
        crate::framebuffer::ah((fgs + 2) as u32, (irb + 2) as u32, fgr as u32, nlg as u32, 0xFF0A0A0A);
        crate::framebuffer::ah(fgs as u32, irb as u32, fgr as u32, nlg as u32, 0xFF252526);
        crate::framebuffer::ah(fgs as u32, irb as u32, fgr as u32, 2, 0xFF007ACC);
        let alf = irb + 6;
        crate::framebuffer::ah((fgs + 8) as u32, alf as u32, (fgr - 16) as u32, 18, 0xFF3C3C3C);
        crate::framebuffer::ah((fgs + 8) as u32, (alf + 17) as u32, (fgr - 16) as u32, 1, 0xFF007ACC);
        let tgr = format!(":{}", input);
        acd(fgs + 12, alf + 2, &tgr, 0xFFCCCCCC);
        let hint = format!("Go to Line (1-{})", g.ak.len());
        let hmy = fgs + fgr - (hint.len() as i32 * nk) - 12;
        acd(hmy, alf + 2, &hint, 0xFF666666);
    }

    
    let uo = c + jts + byr;
    crate::framebuffer::ah(b as u32, uo as u32, d, bfm as u32, AAF_);
    
    
    let mut gsr = b + 8;
    acd(gsr, uo + 4, "@ main", GD_);
    gsr += 7 * nk;
    crate::framebuffer::ah(gsr as u32, (uo + 4) as u32, 1, 14, 0xFF1A6DAA);
    gsr += 6;
    if g.no {
        acd(gsr, uo + 4, "* Modified", 0xFFFFD166);
    } else {
        acd(gsr, uo + 4, "Saved", GD_);
    }
    
    
    let dar = format!("Ln {}, Col {}", g.gn + 1, g.hn + 1);
    let oib = g.eej.j();
    
    let mut bxh = b + d as i32 - 8;
    
    bxh -= dar.len() as i32 * nk;
    acd(bxh, uo + 4, &dar, GD_);
    bxh -= 10;
    crate::framebuffer::ah(bxh as u32, (uo + 4) as u32, 1, 14, 0xFF1A6DAA);
    bxh -= 6;
    
    bxh -= oib.len() as i32 * nk;
    acd(bxh, uo + 4, oib, GD_);
    bxh -= 10;
    crate::framebuffer::ah(bxh as u32, (uo + 4) as u32, 1, 14, 0xFF1A6DAA);
    bxh -= 6;
    
    bxh -= 5 * nk;
    acd(bxh, uo + 4, "UTF-8", GD_);
    bxh -= 10;
    crate::framebuffer::ah(bxh as u32, (uo + 4) as u32, 1, 14, 0xFF1A6DAA);
    bxh -= 6;
    
    bxh -= 9 * nk;
    acd(bxh, uo + 4, "Spaces: 4", GD_);
}
