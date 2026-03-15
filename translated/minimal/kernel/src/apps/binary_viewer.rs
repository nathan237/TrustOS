





use alloc::string::String;
use alloc::vec::Vec;
use alloc::format;
use alloc::collections::BTreeMap;

use crate::binary_analysis::{Rn, Ga, Go, Dc, Sb, XrefType};


const J_: u32 = 28;



const JJ_: u32       = 0xFF0D1117;  
const HN_: u32      = 0xFF161B22;  
const JK_: u32     = 0xFF21262D;  
const ZK_: u32   = 0xFF1F3A5F;  
const ZJ_: u32      = 0xFF1C2333;  
const DDM_: u32       = 0xFF0D1117;  

const HR_: u32      = 0xFF8B949E;  
const BNV_: u32       = 0xFFC9D1D9;  
const BNP_: u32     = 0xFF7EE787;  
const BNX_: u32  = 0xFF79C0FF;  
const AOP_: u32  = 0xFFD2A8FF;  
const AON_: u32 = 0xFFFFA657;  
const AOL_: u32   = 0xFF8B949E;  
const BNW_: u32     = 0xFFFF7B72;  
const AOQ_: u32    = 0xFF7EE787;  
const AOM_: u32    = 0xFFFFFFFF;  
const AOT_: u32      = 0xFFC9D1D9;  
const BOE_: u32 = 0xFFFFA657;  
const AAJ_: u32    = 0xFF8B949E;  
const AAG_: u32      = 0xFFFF7B72;  
const AOK_: u32      = 0xFF79C0FF;  
const AOO_: u32      = 0xFFFFA657;  
const BOA_: u32 = 0xFF30363D;  
const BOK_: u32      = 0xFFD2A8FF;  




#[derive(Clone, Copy, PartialEq)]
pub enum ActivePanel {
    Js,
    Ir,
    Hn,
    V,
}


#[derive(Clone, Copy, PartialEq)]
pub enum NavItem {
    Ato,           
    Axc,   
    Qm(usize),
    Ayk,   
    Ga(usize),
    Azo,          
    Go(usize),
    Asq,        
    Bs(usize),
    Azi,          
    Azh(usize),
    DynamicInfo,      
    Aud,          
    Alz,      
}




pub struct BinaryViewerState {
    
    pub ln: Rn,
    
    pub bjd: ActivePanel,
    
    
    pub bag: Vec<(NavItem, u8, String)>,  
    pub gnk: usize,
    pub dad: usize,
    pub arm: [bool; 8],  

    
    pub cyl: usize,       
    pub cfk: usize,     
    pub toj: usize,       

    
    pub eow: usize,     
    pub bze: usize,  
    
    
    pub esn: usize,
    pub zl: Vec<String>,
    
    
    pub cwt: u64,
    
    
    pub wn: String,
    
    
    pub bcn: bool,
    pub bla: String,
    pub chq: Vec<u64>,
    pub wfq: usize,
}

impl BinaryViewerState {
    
    pub fn new(ln: Rn, path: &str) -> Self {
        let mut g = BinaryViewerState {
            ln,
            bjd: ActivePanel::Hn,
            bag: Vec::new(),
            gnk: 0,
            dad: 0,
            arm: [true, false, true, false, true, false, false, false], 
            cyl: 0,
            cfk: 0,
            toj: 0,
            eow: 0,
            bze: 0,
            esn: 0,
            zl: Vec::new(),
            cwt: 0,
            wn: String::from(path),
            bcn: false,
            bla: String::new(),
            chq: Vec::new(),
            wfq: 0,
        };
        
        
        g.cwt = g.ln.elf.co.bt;
        
        
        if let Some(w) = g.ln.instructions.iter().qf(|a| a.re == g.cwt) {
            g.bze = w;
            g.eow = w.ao(5); 
        }
        
        
        if let Some(l) = g.ln.mot(g.cwt) {
            g.cyl = (l as usize) & !0xF; 
            g.cfk = l as usize;
        }
        
        
        g.exl();
        
        
        g.gvp();
        
        g
    }

    
    pub fn exl(&mut self) {
        self.bag.clear();
        
        
        self.bag.push((NavItem::Ato, 0, format!(
            "[H] ELF Header — {} {} {}",
            self.ln.elf.co.gfz,
            self.ln.elf.co.czk,
            self.ln.elf.co.class
        )));
        
        
        let egp = self.ln.elf.dku.len();
        let vhc = if self.arm[1] { "[-]" } else { "[+]" };
        self.bag.push((NavItem::Axc, 0, format!(
            "{} Program Headers ({})", vhc, egp
        )));
        if self.arm[1] {
            for (a, afv) in self.ln.elf.dku.iter().cf() {
                self.bag.push((NavItem::Qm(a), 1, format!(
                    "  {:6} 0x{:08X} {:>6} {}",
                    afv.ddc(),
                    afv.uy,
                    afv.jfv,
                    afv.kwm(),
                )));
            }
        }
        
        
        let wlo = self.ln.elf.aeo.len();
        let wlp = if self.arm[2] { "[-]" } else { "[+]" };
        self.bag.push((NavItem::Ayk, 0, format!(
            "{} Sections ({})", wlp, wlo
        )));
        if self.arm[2] {
            for (a, zw) in self.ln.elf.aeo.iter().cf() {
                let j = if zw.j.is_empty() { "(null)" } else { &zw.j };
                self.bag.push((NavItem::Ga(a), 1, format!(
                    "  {:<16} {:8} 0x{:08X} {:>6}",
                    j,
                    zw.ddc(),
                    zw.ag,
                    zw.aw,
                )));
            }
        }
        
        
        let mil = self.ln.elf.bot.len() + self.ln.elf.dqj.len();
        let wwv = if self.arm[3] { "[-]" } else { "[+]" };
        self.bag.push((NavItem::Azo, 0, format!(
            "{} Symbols ({})", wwv, mil
        )));
        if self.arm[3] {
            
            let mut pqm: Vec<(usize, &Go)> = self.ln.elf.bot.iter()
                .rh(self.ln.elf.dqj.iter())
                .cf()
                .hi(|(_, e)| !e.j.is_empty() && e.bn != 0)
                .collect();
            pqm.bxf(|(_, e)| e.bn);
            for (a, aaw) in pqm.iter().take(200) {
                let pa = match aaw.gtt {
                    2 => "fn",  
                    1 => "obj", 
                    _ => "  ",
                };
                self.bag.push((NavItem::Go(*a), 1, format!(
                    "  {} {:<24} 0x{:08X} {}",
                    pa, 
                    if aaw.j.len() > 24 { &aaw.j[..24] } else { &aaw.j },
                    aaw.bn,
                    aaw.qpo(),
                )));
            }
        }
        
        
        let szg = self.ln.xrefs.ajb.len();
        let svh = if self.arm[4] { "[-]" } else { "[+]" };
        self.bag.push((NavItem::Asq, 0, format!(
            "{} Functions ({})", svh, szg
        )));
        if self.arm[4] {
            for (a, ke) in self.ln.xrefs.ajb.iter().cf().take(200) {
                let j = if ke.j.is_empty() {
                    format!("sub_{:X}", ke.bt)
                } else {
                    ke.j.clone()
                };
                self.bag.push((NavItem::Bs(a), 1, format!(
                    "  fn {:<24} 0x{:08X} ({} insns)",
                    if j.len() > 24 { &j[..24] } else { &j },
                    ke.bt,
                    ke.jak,
                )));
            }
        }
        
        
        let wuw = self.ln.elf.pd.len();
        let wux = if self.arm[5] { "[-]" } else { "[+]" };
        self.bag.push((NavItem::Azi, 0, format!(
            "{} Strings ({})", wux, wuw
        )));
        if self.arm[5] {
            for (a, e) in self.ln.elf.pd.iter().cf().take(100) {
                let display = if e.ca.len() > 30 {
                    format!("\"{}...\"", &e.ca[..30])
                } else {
                    format!("\"{}\"", e.ca)
                };
                let xqf = e.uy.unwrap_or(0);
                self.bag.push((NavItem::Azh(a), 1, format!(
                    "  0x{:08X} {}", xqf, display
                )));
            }
        }
        
        
        if !self.ln.elf.djt.is_empty() || !self.ln.elf.hhn.is_empty() {
            let shw = if self.arm[6] { "[-]" } else { "[+]" };
            self.bag.push((NavItem::DynamicInfo, 0, format!(
                "{} Dynamic Linking", shw
            )));
            if self.arm[6] {
                for uek in &self.ln.elf.djt {
                    self.bag.push((NavItem::Aud, 1, format!("  NEEDED: {}", uek)));
                }
                if let Some(ahp) = &self.ln.elf.interpreter {
                    self.bag.push((NavItem::Aud, 1, format!("  INTERP: {}", ahp)));
                }
            }
        }
        
        
        if !self.ln.elf.bwp.is_empty() {
            let vuj = self.ln.elf.bwp.len();
            let vuk = if self.arm[7] { "[-]" } else { "[+]" };
            self.bag.push((NavItem::Alz, 0, format!(
                "{} Relocations ({})", vuk, vuj
            )));
            if self.arm[7] {
                for dbb in self.ln.elf.bwp.iter().take(100) {
                    let ezj = if dbb.ezj.is_empty() { "-" } else { &dbb.ezj };
                    self.bag.push((NavItem::Alz, 1, format!(
                        "  0x{:08X} {} + 0x{:X}",
                        dbb.l, ezj, dbb.fcn
                    )));
                }
            }
        }
    }

    
    pub fn dhq(&mut self, ag: u64) {
        self.cwt = ag;
        
        
        if let Some(w) = self.ln.instructions.iter().qf(|a| a.re >= ag) {
            self.bze = w;
            self.eow = w.ao(5);
        }
        
        
        if let Some(l) = self.ln.mot(ag) {
            let dz = l as usize;
            self.cfk = dz;
            self.cyl = dz & !0xF;
        }
        
        
        self.gvp();
    }

    
    pub fn gvp(&mut self) {
        self.zl.clear();
        let ag = self.cwt;
        
        
        self.zl.push(format!("Address: 0x{:016X}", ag));
        self.zl.push(String::new());
        
        
        if let Some(zw) = self.ln.elf.wfz(ag) {
            self.zl.push(format!("Section: {} [{}]", zw.j, zw.ddc()));
            self.zl.push(format!("  Range: 0x{:X}..0x{:X}", zw.ag, zw.ag + zw.aw));
            self.zl.push(format!("  Flags: {}", zw.kwm()));
        }
        
        
        if let Some(ezj) = self.ln.elf.blw.get(&ag) {
            self.zl.push(String::new());
            self.zl.push(format!("Symbol: {}", ezj));
        }
        
        
        if let Some(ke) = self.ln.xrefs.szk(ag) {
            self.zl.push(String::new());
            let j = if ke.j.is_empty() {
                format!("sub_{:X}", ke.bt)
            } else {
                ke.j.clone()
            };
            self.zl.push(format!("Function: {}", j));
            self.zl.push(format!("  Entry: 0x{:X}", ke.bt));
            self.zl.push(format!("  End:   0x{:X}", ke.ci));
            self.zl.push(format!("  Instructions: {}", ke.jak));
            self.zl.push(format!("  Basic blocks: {}", ke.ikx));
            
            if !ke.imr.is_empty() {
                self.zl.push(String::new());
                self.zl.push(String::from("Calls to:"));
                for cd in &ke.imr {
                    let j = self.ln.elf.blw.get(cd)
                        .abn()
                        .unwrap_or_else(|| format!("0x{:X}", cd));
                    self.zl.push(format!("  -> {}", j));
                }
            }
            if !ke.imq.is_empty() {
                self.zl.push(String::new());
                self.zl.push(String::from("Called from:"));
                for nbn in &ke.imq {
                    let j = self.ln.elf.blw.get(nbn)
                        .abn()
                        .unwrap_or_else(|| format!("0x{:X}", nbn));
                    self.zl.push(format!("  <- {}", j));
                }
            }
        }
        
        
        let ihw = self.ln.xrefs.ihw(ag);
        if !ihw.is_empty() {
            self.zl.push(String::new());
            self.zl.push(format!("Xrefs TO 0x{:X} ({}):", ag, ihw.len()));
            for bta in ihw.iter().take(20) {
                let bde = match bta.dnl {
                    XrefType::En => "CALL",
                    XrefType::Nh => "JMP ",
                    XrefType::Ahd => "Jcc ",
                    XrefType::Aaw => "DATA",
                };
                self.zl.push(format!("  {} from 0x{:X}", bde, bta.from));
            }
        }
        
        
        let gxa = self.ln.xrefs.gxa(ag);
        if !gxa.is_empty() {
            self.zl.push(String::new());
            self.zl.push(format!("Xrefs FROM 0x{:X} ({}):", ag, gxa.len()));
            for bta in gxa.iter().take(20) {
                let bde = match bta.dnl {
                    XrefType::En => "CALL",
                    XrefType::Nh => "JMP ",
                    XrefType::Ahd => "Jcc ",
                    XrefType::Aaw => "DATA",
                };
                let fwf = self.ln.elf.blw.get(&bta.wh)
                    .abn()
                    .unwrap_or_else(|| format!("0x{:X}", bta.wh));
                self.zl.push(format!("  {} -> {}", bde, fwf));
            }
        }
        
        
        if let Some(fi) = self.ln.tvj(ag) {
            self.zl.push(String::new());
            self.zl.push(String::from("Instruction:"));
            self.zl.push(format!("  {} {}", fi.bes, fi.bvs));
            let toi: Vec<String> = fi.bf.iter().map(|o| format!("{:02X}", o)).collect();
            self.zl.push(format!("  Bytes: {}", toi.rr(" ")));
            self.zl.push(format!("  Size: {} bytes", fi.bf.len()));
            if let Some(ref byv) = fi.byv {
                self.zl.push(format!("  Note: {}", byv));
            }
            if let Some(cd) = fi.ena {
                self.zl.push(format!("  Target: 0x{:X}", cd));
            }
        }
    }

    
    pub fn vr(&mut self, bs: char) {
        match bs {
            
            '\t' => {
                self.bjd = match self.bjd {
                    ActivePanel::Js => ActivePanel::Ir,
                    ActivePanel::Ir => ActivePanel::Hn,
                    ActivePanel::Hn => ActivePanel::V,
                    ActivePanel::V => ActivePanel::Js,
                };
            },
            
            'U' => self.dlm(),   
            'D' => self.eid(), 
            'L' => self.mco(), 
            'R' => self.mcq(), 
            
            '\n' | '\r' => self.lqh(),
            
            'g' | 'G' => {
                self.bcn = !self.bcn;
                if self.bcn {
                    self.bla.clear();
                }
            },
            
            '0'..='9' | 'a'..='f' | 'A'..='F' if self.bcn => {
                self.bla.push(bs);
            },
            
            '\x08' if self.bcn => {
                self.bla.pop();
            },
            
            'x' | 'X' => self.svl(),
            _ => {}
        }
    }
    
    
    pub fn crc(&mut self, scancode: u8) {
        match scancode {
            0x48 => self.dlm(),    
            0x50 => self.eid(),  
            0x4B => self.mco(),  
            0x4D => self.mcq(), 
            0x49 => {                    
                for _ in 0..20 { self.dlm(); }
            },
            0x51 => {                    
                for _ in 0..20 { self.eid(); }
            },
            0x47 => self.tgp(), 
            0x4F => self.tgo(),   
            0x0F => {                    
                self.bjd = match self.bjd {
                    ActivePanel::Js => ActivePanel::Ir,
                    ActivePanel::Ir => ActivePanel::Hn,
                    ActivePanel::Hn => ActivePanel::V,
                    ActivePanel::V => ActivePanel::Js,
                };
            },
            0x1C => self.lqh(),     
            _ => {}
        }
    }

    fn dlm(&mut self) {
        match self.bjd {
            ActivePanel::Js => {
                if self.dad > 0 {
                    self.dad -= 1;
                    if self.dad < self.gnk {
                        self.gnk = self.dad;
                    }
                }
            },
            ActivePanel::Ir => {
                if self.cyl >= 16 {
                    self.cyl -= 16;
                }
                if self.cfk >= 16 {
                    self.cfk -= 16;
                }
            },
            ActivePanel::Hn => {
                if self.bze > 0 {
                    self.bze -= 1;
                    if self.bze < self.eow {
                        self.eow = self.bze;
                    }
                    
                    if let Some(fi) = self.ln.instructions.get(self.bze) {
                        self.cwt = fi.re;
                        self.gvp();
                    }
                }
            },
            ActivePanel::V => {
                if self.esn > 0 {
                    self.esn -= 1;
                }
            },
        }
    }

    fn eid(&mut self) {
        match self.bjd {
            ActivePanel::Js => {
                if self.dad + 1 < self.bag.len() {
                    self.dad += 1;
                }
            },
            ActivePanel::Ir => {
                if self.cyl + 16 < self.ln.f.len() {
                    self.cyl += 16;
                }
                self.cfk += 16;
                if self.cfk >= self.ln.f.len() {
                    self.cfk = self.ln.f.len().ao(1);
                }
            },
            ActivePanel::Hn => {
                if self.bze + 1 < self.ln.instructions.len() {
                    self.bze += 1;
                    
                    if let Some(fi) = self.ln.instructions.get(self.bze) {
                        self.cwt = fi.re;
                        self.gvp();
                    }
                }
            },
            ActivePanel::V => {
                if self.esn + 1 < self.zl.len() {
                    self.esn += 1;
                }
            },
        }
    }

    fn mco(&mut self) {
        
        for _ in 0..10 { self.dlm(); }
    }

    fn mcq(&mut self) {
        
        for _ in 0..10 { self.eid(); }
    }

    fn tgp(&mut self) {
        match self.bjd {
            ActivePanel::Js => { self.dad = 0; self.gnk = 0; },
            ActivePanel::Ir => { self.cyl = 0; self.cfk = 0; },
            ActivePanel::Hn => { self.eow = 0; self.bze = 0; },
            ActivePanel::V => { self.esn = 0; },
        }
    }

    fn tgo(&mut self) {
        match self.bjd {
            ActivePanel::Js => {
                self.dad = self.bag.len().ao(1);
            },
            ActivePanel::Ir => {
                let qv = self.ln.f.len().ao(16);
                self.cyl = qv & !0xF;
                self.cfk = qv;
            },
            ActivePanel::Hn => {
                self.bze = self.ln.instructions.len().ao(1);
            },
            ActivePanel::V => {
                self.esn = self.zl.len().ao(1);
            },
        }
    }

    fn lqh(&mut self) {
        match self.bjd {
            ActivePanel::Js => {
                if let Some((item, _, _)) = self.bag.get(self.dad) {
                    match *item {
                        NavItem::Ato => {
                            
                            self.dhq(self.ln.elf.co.bt);
                        },
                        NavItem::Axc => {
                            self.arm[1] = !self.arm[1];
                            self.exl();
                        },
                        NavItem::Qm(a) => {
                            if let Some(afv) = self.ln.elf.dku.get(a) {
                                self.dhq(afv.uy);
                            }
                        },
                        NavItem::Ayk => {
                            self.arm[2] = !self.arm[2];
                            self.exl();
                        },
                        NavItem::Ga(a) => {
                            if let Some(zw) = self.ln.elf.aeo.get(a) {
                                if zw.ag != 0 {
                                    self.dhq(zw.ag);
                                } else {
                                    
                                    self.cyl = zw.l as usize & !0xF;
                                    self.cfk = zw.l as usize;
                                    self.bjd = ActivePanel::Ir;
                                }
                            }
                        },
                        NavItem::Azo => {
                            self.arm[3] = !self.arm[3];
                            self.exl();
                        },
                        NavItem::Go(a) => {
                            let qgq: Vec<&Go> = self.ln.elf.bot.iter()
                                .rh(self.ln.elf.dqj.iter())
                                .hi(|e| !e.j.is_empty() && e.bn != 0)
                                .collect();
                            if let Some(aaw) = qgq.get(a) {
                                self.dhq(aaw.bn);
                            }
                        },
                        NavItem::Asq => {
                            self.arm[4] = !self.arm[4];
                            self.exl();
                        },
                        NavItem::Bs(a) => {
                            if let Some(ke) = self.ln.xrefs.ajb.get(a) {
                                self.dhq(ke.bt);
                            }
                        },
                        NavItem::Azi => {
                            self.arm[5] = !self.arm[5];
                            self.exl();
                        },
                        NavItem::Azh(a) => {
                            if let Some(e) = self.ln.elf.pd.get(a) {
                                if let Some(uy) = e.uy {
                                    self.dhq(uy);
                                }
                            }
                        },
                        NavItem::DynamicInfo => {
                            self.arm[6] = !self.arm[6];
                            self.exl();
                        },
                        NavItem::Alz => {
                            self.arm[7] = !self.arm[7];
                            self.exl();
                        },
                        NavItem::Aud => {},
                    }
                }
            },
            ActivePanel::Hn => {
                
                if let Some(fi) = self.ln.instructions.get(self.bze) {
                    if let Some(cd) = fi.ena {
                        self.dhq(cd);
                    }
                }
            },
            ActivePanel::Ir => {
                
                if let Some(uy) = self.ln.osf(self.cfk as u64) {
                    self.dhq(uy);
                    self.bjd = ActivePanel::Hn;
                }
            },
            ActivePanel::V => {},
        }

        
        if self.bcn && !self.bla.is_empty() {
            if let Ok(ag) = u64::wa(&self.bla, 16) {
                self.dhq(ag);
                self.bcn = false;
            }
        }
    }

    fn svl(&mut self) {
        
        let xrefs = self.ln.xrefs.gxa(self.cwt);
        if let Some(bta) = xrefs.fv() {
            self.dhq(bta.wh);
        }
    }

    
    pub fn ago(&mut self, amr: i32, aio: i32, aog: u32, biz: u32) {
        let nd = biz.ao(J_ + 24) as i32; 
        let eve = (aog as i32 * 25) / 100; 
        let fko = (aog as i32 * 25) / 100; 
        let geu = (aog as i32 * 30) / 100; 
        

        let bbs = 20i32;
        let bfm = 20i32;
        let nfr = (J_ as i32) + bbs;

        
        if aio < nfr || aio > biz as i32 - bfm {
            return; 
        }

        let gy = 14i32;
        let atd = ((aio - nfr) / gy) as usize;

        if amr < eve {
            
            self.bjd = ActivePanel::Js;
            let dew = self.gnk + atd;
            if dew < self.bag.len() {
                self.dad = dew;
                self.lqh(); 
            }
        } else if amr < eve + fko {
            
            self.bjd = ActivePanel::Ir;
            let kia = self.cyl + atd * 16;
            if kia < self.ln.f.len() {
                self.cfk = kia;
                if let Some(uy) = self.ln.osf(kia as u64) {
                    self.cwt = uy;
                    self.gvp();
                }
            }
        } else if amr < eve + fko + geu {
            
            self.bjd = ActivePanel::Hn;
            let khz = self.eow + atd;
            if khz < self.ln.instructions.len() {
                self.bze = khz;
                self.cwt = self.ln.instructions[khz].re;
                self.gvp();
            }
        } else {
            
            self.bjd = ActivePanel::V;
        }
    }
}





pub fn kqv(
    g: &BinaryViewerState,
    fx: i32, lw: i32, hk: u32, mg: u32,
    acd: &dyn Fn(i32, i32, &str, u32),
) {
    let bxn = J_ as i32;
    let bfm = 20i32;
    let bbs = 20i32;
    
    let tc = fx + 1;
    let gl = lw + bxn;
    let ur = hk.ao(2) as i32;
    let nd = (mg as i32) - bxn - bfm;
    
    if ur < 200 || nd < 100 {
        return;
    }

    
    let eve = (ur * 25) / 100;
    let fko = (ur * 25) / 100;
    let geu = (ur * 30) / 100;
    let izw = ur - eve - fko - geu;

    let lno = tc;
    let fkp = lno + eve;
    let ire = fkp + fko;
    let leb = ire + geu;

    
    crate::framebuffer::ah(fx as u32, (lw + bxn) as u32, hk, mg - bxn as u32, JJ_);

    
    let zk = [
        (lno, eve, "Navigation", ActivePanel::Js),
        (fkp, fko, "Hex View", ActivePanel::Ir),
        (ire, geu, "Disassembly", ActivePanel::Hn),
        (leb, izw, "Info / Xrefs", ActivePanel::V),
    ];

    for (y, ars, cu, vbh) in &zk {
        let ei = if *vbh == g.bjd { 0xFF1F6FEB } else { JK_ };
        crate::framebuffer::ah(*y as u32, gl as u32, *ars as u32, bbs as u32, ei);
        acd(*y + 4, gl + 3, cu, AOM_);
    }

    
    for y in &[fkp, ire, leb] {
        crate::framebuffer::ah(*y as u32, (gl + bbs) as u32, 1, nd as u32, BOA_);
    }

    let atg = gl + bbs;
    let ans = nd - bbs;
    let gy = 14i32;
    let act = (ans / gy) as usize;

    
    seg(g, lno, atg, eve, act, gy, acd);

    
    sdn(g, fkp + 2, atg, fko - 4, act, gy, acd);

    
    scp(g, ire + 2, atg, geu - 4, act, gy, acd);

    
    krc(g, leb + 2, atg, izw - 4, act, gy, acd);

    
    let uo = lw + mg as i32 - bfm;
    crate::framebuffer::ah(fx as u32, uo as u32, hk, bfm as u32, JK_);

    
    let awz = format!(
        " {} | {} | {} insns | {} syms | {} funcs",
        g.wn,
        g.ln.elf.co.gfz,
        g.ln.instructions.len(),
        g.ln.elf.bot.len() + g.ln.elf.dqj.len(),
        g.ln.xrefs.ajb.len(),
    );
    acd(fx + 4, uo + 3, &awz, AAJ_);

    
    let elz = format!("0x{:016X} ", g.cwt);
    let qfs = fx + hk as i32 - (elz.len() as i32 * 8) - 4;
    acd(qfs, uo + 3, &elz, HR_);

    
    if g.bcn {
        let blb = lw + bxn + 2;
        let kp = 250i32;
        let kl = 20i32;
        let cr = fx + hk as i32 / 2 - kp / 2;
        crate::framebuffer::ah(cr as u32, blb as u32, kp as u32, kl as u32, 0xFF1F6FEB);
        let aau = format!("Go to: 0x{}_", g.bla);
        acd(cr + 4, blb + 3, &aau, 0xFFFFFFFF);
    }
}


fn seg(
    g: &BinaryViewerState,
    b: i32, c: i32, d: i32,
    iw: usize, gy: i32,
    acd: &dyn Fn(i32, i32, &str, u32),
) {
    let ay = g.gnk;
    let ci = (ay + iw).v(g.bag.len());

    for (afj, w) in (ay..ci).cf() {
        let ct = c + (afj as i32) * gy;
        let (item, crn, text) = &g.bag[w];

        
        if w == g.dad {
            let ei = if g.bjd == ActivePanel::Js { ZK_ } else { ZJ_ };
            crate::framebuffer::ah(b as u32, ct as u32, d as u32, gy as u32, ei);
        }

        
        let s = match item {
            NavItem::Ato | NavItem::Axc | NavItem::Ayk |
            NavItem::Azo | NavItem::Asq | NavItem::Azi |
            NavItem::DynamicInfo | NavItem::Alz => BOE_,
            NavItem::Bs(_) => AAG_,
            NavItem::Go(_) => AOP_,
            NavItem::Azh(_) => AOQ_,
            _ => AOT_,
        };

        
        let aem = (d / 8).am(4) as usize;
        let display = if text.len() > aem {
            &text[..aem]
        } else {
            text
        };

        acd(b + 2 + (*crn as i32 * 8), ct + 1, display, s);
    }
}


fn sdn(
    g: &BinaryViewerState,
    b: i32, c: i32, dxx: i32,
    iw: usize, gy: i32,
    acd: &dyn Fn(i32, i32, &str, u32),
) {
    let f = &g.ln.f;
    let cun = g.cyl;

    for line in 0..iw {
        let l = cun + line * 16;
        if l >= f.len() { break; }

        let ct = c + (line as i32) * gy;
        let ci = (l + 16).v(f.len());
        let jj = &f[l..ci];

        
        if l <= g.cfk && g.cfk < l + 16 {
            let ei = if g.bjd == ActivePanel::Ir { ZK_ } else { ZJ_ };
            crate::framebuffer::ah(b as u32, ct as u32, 400, gy as u32, ei);
        }

        
        let dyd = format!("{:06X}", l);
        acd(b, ct + 1, &dyd, HR_);

        
        let mut bng = b + 56;
        for (a, &o) in jj.iter().cf() {
            if a == 8 { bng += 4; } 
            let quz = format!("{:02X}", o);
            
            
            let bj = if o == 0 {
                0xFF484F58 
            } else if o >= 0x20 && o < 0x7F {
                BNV_ 
            } else {
                AON_ 
            };
            acd(bng, ct + 1, &quz, bj);
            bng += 20;
        }

        
        let ikb = b + 56 + 16 * 20 + 12;
        let mut ax = ikb;
        for &o in jj {
            let bm = if o >= 0x20 && o < 0x7F { o as char } else { '.' };
            let mut k = [0u8; 4];
            let e = bm.hia(&mut k);
            let bj = if o >= 0x20 && o < 0x7F { BNP_ } else { 0xFF484F58 };
            acd(ax, ct + 1, e, bj);
            ax += 8;
        }
    }
}


fn scp(
    g: &BinaryViewerState,
    b: i32, c: i32, d: i32,
    iw: usize, gy: i32,
    acd: &dyn Fn(i32, i32, &str, u32),
) {
    let edl = &g.ln.instructions;
    if edl.is_empty() {
        acd(b + 4, c + 20, "No code to display", AOL_);
        return;
    }

    
    let ay = g.eow;
    let ci = (ay + iw).v(edl.len());

    for (afj, w) in (ay..ci).cf() {
        let fi = &edl[w];
        let ct = c + (afj as i32) * gy;

        
        if w == g.bze {
            let ei = if g.bjd == ActivePanel::Hn { ZK_ } else { ZJ_ };
            crate::framebuffer::ah(b as u32, ct as u32, d as u32, gy as u32, ei);
        }

        
        if g.ln.xrefs.txn(fi.re) {
            if let Some(j) = g.ln.elf.blw.get(&fi.re) {
                
                let cu = format!("<{}>:", j);
                let aem = (d / 8).am(4) as usize;
                let display = if cu.len() > aem { &cu[..aem] } else { &cu };
                acd(b + 2, ct + 1, display, BNW_);
                continue; 
            }
        }

        let mut cx = b + 2;

        
        let dyd = format!("{:08X}", fi.re);
        acd(cx, ct + 1, &dyd, HR_);
        cx += 72;

        
        let aal = fi.bf.len().v(6);
        let mut hcb = String::new();
        for o in &fi.bf[..aal] {
            hcb.t(&format!("{:02X}", o));
        }
        if fi.bf.len() > 6 { hcb.t(".."); }
        
        while hcb.len() < 14 { hcb.push(' '); }
        acd(cx, ct + 1, &hcb, 0xFF484F58);
        cx += 116;

        
        let uoz = if fi.etc {
            AOK_
        } else if fi.etg || fi.etd {
            AOO_
        } else if fi.edy {
            AAG_
        } else {
            BNX_
        };
        let uoy = format!("{:<7}", fi.bes);
        acd(cx, ct + 1, &uoy, uoz);
        cx += 60;

        
        let oma = ((d - (cx - b)) / 8).am(1) as usize;
        let bvr = if fi.bvs.len() > oma {
            &fi.bvs[..oma]
        } else {
            &fi.bvs
        };

        
        let uyq = if fi.bvs.cj("0x") || fi.bvs.contains("0x") {
            AON_
        } else {
            AOP_
        };
        acd(cx, ct + 1, bvr, uyq);

        
        if let Some(ref byv) = fi.byv {
            let kjz = format!(" ; {}", byv);
            let pbq = d - (cx - b) - (bvr.len() as i32 * 8);
            if pbq > 24 {
                let olo = (pbq / 8) as usize;
                let display = if kjz.len() > olo { &kjz[..olo] } else { &kjz };
                acd(cx + (bvr.len() as i32 * 8), ct + 1, display, AOL_);
            }
        }
    }
}


fn krc(
    g: &BinaryViewerState,
    b: i32, c: i32, dxx: i32,
    iw: usize, gy: i32,
    acd: &dyn Fn(i32, i32, &str, u32),
) {
    let ay = g.esn;
    let ci = (ay + iw).v(g.zl.len());

    for (afj, w) in (ay..ci).cf() {
        let ct = c + (afj as i32) * gy;
        let line = &g.zl[w];

        
        let bj = if line.cj("Address:") || line.cj("Section:") ||
                     line.cj("Symbol:") || line.cj("Function:") ||
                     line.cj("Instruction:") {
            AOM_
        } else if line.cj("Xrefs") {
            BOK_
        } else if line.cj("  ->") || line.cj("  <-") {
            AOK_
        } else if line.cj("  CALL") || line.cj("  JMP") || line.cj("  Jcc") {
            AOO_
        } else if line.cj("  DATA") {
            AOQ_
        } else if line.cj("Calls to:") || line.cj("Called from:") {
            AAG_
        } else {
            AOT_
        };

        
        let aem = 40usize; 
        let display = if line.len() > aem { &line[..aem] } else { line };
        acd(b + 2, ct + 1, display, bj);
    }
}
