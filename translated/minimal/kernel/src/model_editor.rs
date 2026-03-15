





use alloc::vec::Vec;
use alloc::string::String;
use alloc::format;
use crate::formula3d::{V3, Bh, lz, rk};



const ATW_: f32 = 8.0;
const ADD_: usize = 16;
const BIF_: f32 = 12.0;
const ZG_: f32 = 2.0;


const MH_: u32        = 0xFF1A1A2E;
const BNT_: u32      = 0xFF2A2A3A;
const BNU_: u32 = 0xFF3A3A4A;
const BNQ_: u32    = 0xFFFF4444;
const BNR_: u32    = 0xFF44FF44;
const BNS_: u32    = 0xFF4488FF;
const BOI_: u32      = 0xFF00FFAA;
const BOJ_: u32  = 0xFFFFFF00;
const BOF_: u32    = 0xFFFFFFFF;
const BOH_: u32  = 0xFFFFAA00;
const BOG_: u32 = 0xFF88FFFF;
const SG_: u32   = 0xFF0D0D1A;
const MI_: u32 = 0xFF2A2A4A;
const BOD_: u32 = 0xFF0055AA;
const AAJ_: u32    = 0xFF0D0D1A;
const T_: u32      = 0xFFCCCCCC;
const AOR_: u32   = 0xFFFFFFFF;
const AAK_: u32  = 0xFF777777;
const AOS_: u32    = 0xCC1A1A2E;
const SF_: u32 = 0xFF00AAFF;



#[derive(Clone, Copy, PartialEq)]
pub enum EditTool {
    Qs,     
    Rh,  
    Rg,    
    Fw,       
    Jj,     
}

impl EditTool {
    fn j(self) -> &'static str {
        match self {
            EditTool::Qs => "Select",
            EditTool::Rh => "Add Vertex",
            EditTool::Rg => "Add Edge",
            EditTool::Fw => "Move",
            EditTool::Jj => "Delete",
        }
    }
    fn tpy(self) -> &'static str {
        match self {
            EditTool::Qs => "S",
            EditTool::Rh => "A",
            EditTool::Rg => "E",
            EditTool::Fw => "G",
            EditTool::Jj => "X",
        }
    }
    fn idq(self) -> &'static str {
        match self {
            EditTool::Qs => "Click vertex to select. Shift+click for multi-select.",
            EditTool::Rh => "Click in viewport to place a vertex on the grid.",
            EditTool::Rg => "Click two vertices to connect them with an edge.",
            EditTool::Fw => "Drag selected vertices. Arrows = fine move.",
            EditTool::Jj => "Click a vertex or edge to remove it.",
        }
    }
}



#[derive(Clone)]
pub struct ModelEditorState {
    
    pub lm: Vec<V3>,
    pub bu: Vec<(usize, usize)>,
    
    
    pub bbc: f32,    
    pub atx: f32,    
    pub aab: f32,       
    pub eng: V3,      
    
    
    pub bxo: EditTool,
    pub awx: Vec<usize>,
    pub lcn: Option<usize>,
    pub bgv: Option<usize>, 
    
    
    pub daa: i32,
    pub dab: i32,
    pub jgf: bool,
    pub hrx: bool,
    pub kqs: i32,
    pub kqt: i32,
    pub vyu: bool,  
    
    
    pub pkm: bool,
    pub pkl: bool,
    pub ial: bool,
    pub jqd: bool,
    pub aoc: String,
    pub hrr: String,
    
    
    pub bsu: Vec<(Vec<V3>, Vec<(usize, usize)>)>,
    
    
    frame: u32,
    
    kga: f32,
    kgc: f32,
    kfz: f32,
    kgb: f32,
}

impl ModelEditorState {
    pub fn new() -> Self {
        Self {
            lm: Vec::new(),
            bu: Vec::new(),
            bbc: 0.6,
            atx: 0.4,
            aab: 6.0,
            eng: V3 { b: 0.0, c: 0.0, av: 0.0 },
            bxo: EditTool::Qs,
            awx: Vec::new(),
            lcn: None,
            bgv: None,
            daa: 0,
            dab: 0,
            jgf: false,
            hrx: false,
            kqs: 0,
            kqt: 0,
            vyu: false,
            pkm: true,
            pkl: true,
            ial: true,
            jqd: true,
            aoc: String::from("TrustEdit 3D ready — Press H for help"),
            hrr: String::from("untitled"),
            bsu: Vec::new(),
            frame: 0,
            kga: rk(0.6),
            kgc: lz(0.6),
            kfz: rk(0.4),
            kgb: lz(0.4),
        }
    }
    
    
    pub fn gls(&mut self, j: &str) {
        self.bru();
        let mesh = match j {
            "cube" => crate::formula3d::czt(),
            "pyramid" => crate::formula3d::czv(),
            "diamond" => crate::formula3d::czu(),
            "torus" => crate::formula3d::czw(1.0, 0.4, 16, 8),
            "icosphere" => crate::formula3d::fod(1.2),
            "grid" => crate::formula3d::foc(2.0, 4),
            _ => return,
        };
        self.lm = mesh.lm;
        self.bu = mesh.bu;
        self.awx.clear();
        self.bgv = None;
        self.aoc = format!("Loaded preset: {} ({} verts, {} edges)", 
            j, self.lm.len(), self.bu.len());
    }
    
    
    pub fn clear(&mut self) {
        self.bru();
        self.lm.clear();
        self.bu.clear();
        self.awx.clear();
        self.bgv = None;
        self.aoc = String::from("Scene cleared");
    }
    
    
    fn bru(&mut self) {
        if self.bsu.len() > 20 {
            self.bsu.remove(0);
        }
        self.bsu.push((self.lm.clone(), self.bu.clone()));
    }
    
    
    pub fn ifu(&mut self) {
        if let Some((by, bu)) = self.bsu.pop() {
            self.lm = by;
            self.bu = bu;
            self.awx.clear();
            self.bgv = None;
            self.aoc = String::from("Undo");
        } else {
            self.aoc = String::from("Nothing to undo");
        }
    }
    
    
    pub fn mbr(&mut self) {
        
        let mut f = String::new();
        f.t("TRUSTEDIT3D v1\n");
        f.t(&format!("vertices {}\n", self.lm.len()));
        for p in &self.lm {
            f.t(&format!("v {:.4} {:.4} {:.4}\n", p.b, p.c, p.av));
        }
        f.t(&format!("edges {}\n", self.bu.len()));
        for &(q, o) in &self.bu {
            f.t(&format!("e {} {}\n", q, o));
        }
        
        let path = format!("/{}.t3d", self.hrr);
        let bf = f.as_bytes();
        
        
        let _ = crate::ramfs::fh(|fs| fs.touch(&path));
        match crate::ramfs::fh(|fs| fs.ns(&path, bf)) {
            Ok(_) => {
                self.aoc = format!("Saved to {}", path);
            },
            Err(_) => {
                self.aoc = String::from("Error: Could not save file");
            }
        }
    }
    
    
    pub fn load(&mut self, path: &str) {
        let rto: Result<Vec<u8>, ()> = crate::ramfs::fh(|fs| {
            match fs.mq(path) {
                Ok(bf) => Ok(bf.ip()),
                Err(_) => Err(()),
            }
        });
        
        match rto {
            Ok(f) => {
                self.bru();
                if let Ok(text) = core::str::jg(&f) {
                    self.vdx(text);
                    
                    if let Some(j) = path.blj('/') {
                        if let Some(j) = j.ezc(".t3d") {
                            self.hrr = String::from(j);
                        }
                    }
                    self.aoc = format!("Loaded {} ({} verts, {} edges)", 
                        path, self.lm.len(), self.bu.len());
                } else {
                    self.aoc = String::from("Error: Invalid file data");
                }
            },
            Err(_) => {
                self.aoc = format!("Error: Could not read {}", path);
            }
        }
    }
    
    
    fn vdx(&mut self, text: &str) {
        self.lm.clear();
        self.bu.clear();
        self.awx.clear();
        
        for line in text.ak() {
            let ek: Vec<&str> = line.ayt().collect();
            if ek.is_empty() { continue; }
            match ek[0] {
                "v" if ek.len() >= 4 => {
                    if let (Ok(b), Ok(c), Ok(av)) = (
                        ek[1].parse::<f32>(),
                        ek[2].parse::<f32>(),
                        ek[3].parse::<f32>(),
                    ) {
                        self.lm.push(V3 { b, c, av });
                    }
                },
                "e" if ek.len() >= 3 => {
                    if let (Ok(q), Ok(o)) = (
                        ek[1].parse::<usize>(),
                        ek[2].parse::<usize>(),
                    ) {
                        self.bu.push((q, o));
                    }
                },
                _ => {}
            }
        }
    }
    
    
    pub fn zsw(&self) -> Bh {
        Bh {
            lm: self.lm.clone(),
            bu: self.bu.clone(),
            cqd: None,
            ks: None,
            cxq: None,
        }
    }
    
    
    
    fn dkv(&self, p: V3, gm: usize, me: usize) -> (i32, i32, f32) {
        
        let p = V3 { 
            b: p.b - self.eng.b, 
            c: p.c - self.eng.c, 
            av: p.av - self.eng.av 
        };
        
        let ae = self.kga;
        let cq = self.kgc;
        let kb = V3 { b: p.b * ae + p.av * cq, c: p.c, av: -p.b * cq + p.av * ae };
        
        let cx = self.kfz;
        let cr = self.kgb;
        let ix = V3 { b: kb.b, c: kb.c * cx - kb.av * cr, av: kb.c * cr + kb.av * cx };
        
        
        let ifo = ix.av + self.aab;
        
        
        if ifo < 0.1 { return (0, 0, -999.0); }
        let bv = (gm.v(me) as f32) * 0.45;
        let wwt = (ix.b / ifo * bv) + (gm as f32 * 0.5);
        let wwu = (-ix.c / ifo * bv) + (me as f32 * 0.5);
        (wwt as i32, wwu as i32, ifo)
    }
    
    
    fn xol(&self, cr: i32, cq: i32, gm: usize, me: usize) -> V3 {
        
        let bv = (gm.v(me) as f32) * 0.45;
        let kb = (cr as f32 - gm as f32 * 0.5) / bv;
        let ix = -(cq as f32 - me as f32 * 0.5) / bv;
        
        
        let te = V3 { b: kb, c: ix, av: 1.0 };
        
        
        let cx = rk(-self.atx);
        let pqi = lz(-self.atx);
        let apo = V3 { b: te.b, c: te.c * cx - te.av * pqi, av: te.c * pqi + te.av * cx };
        
        let ae = rk(-self.bbc);
        let pql = lz(-self.bbc);
        let us = V3 { b: apo.b * ae + apo.av * pql, c: apo.c, av: -apo.b * pql + apo.av * ae };
        
        
        let fee = V3 {
            b: self.eng.b - self.aab * lz(self.bbc) * rk(self.atx),
            c: self.eng.c + self.aab * lz(self.atx),
            av: self.eng.av - self.aab * rk(self.bbc) * rk(self.atx),
        };
        
        
        if us.c.gp() < 0.001 {
            
            return V3 { b: fee.b + us.b * 5.0, c: 0.0, av: fee.av + us.av * 5.0 };
        }
        let ab = -fee.c / us.c;
        if ab < 0.0 {
            
            return V3 { b: fee.b + us.b * 5.0, c: 0.0, av: fee.av + us.av * 5.0 };
        }
        
        let dnh = V3 {
            b: fee.b + us.b * ab,
            c: 0.0,
            av: fee.av + us.av * ab,
        };
        
        
        V3 {
            b: (dnh.b * 2.0 + 0.5) as i32 as f32 * 0.5,
            c: 0.0,
            av: (dnh.av * 2.0 + 0.5) as i32 as f32 * 0.5,
        }
    }
    
    
    fn hjn(&self, cr: i32, cq: i32, gm: usize, me: usize) -> Option<usize> {
        let mut bdn: Option<(usize, f32)> = None;
        for (a, p) in self.lm.iter().cf() {
            let (y, x, av) = self.dkv(*p, gm, me);
            if av < 0.1 { continue; }
            let dx = (y - cr) as f32;
            let bg = (x - cq) as f32;
            let la = dx * dx + bg * bg;
            if la < BIF_ * BIF_ {
                if let Some((_, ilj)) = bdn {
                    if la < ilj { bdn = Some((a, la)); }
                } else {
                    bdn = Some((a, la));
                }
            }
        }
        bdn.map(|(a, _)| a)
    }
    
    
    
    pub fn vr(&mut self, bs: u8) {
        use crate::keyboard::*;
        
        match bs {
            
            b's' | b'S' => {
                self.bxo = EditTool::Qs;
                self.bgv = None;
                self.aoc = String::from("Tool: Select");
            },
            b'a' | b'A' => {
                self.bxo = EditTool::Rh;
                self.bgv = None;
                self.aoc = String::from("Tool: Add Vertex — Click to place");
            },
            b'e' | b'E' => {
                self.bxo = EditTool::Rg;
                self.bgv = None;
                self.aoc = String::from("Tool: Add Edge — Click two vertices");
            },
            b'g' | b'G' => {
                self.bxo = EditTool::Fw;
                self.bgv = None;
                self.aoc = String::from("Tool: Move — Drag vertices");
            },
            b'x' | b'X' => {
                if self.bxo == EditTool::Jj {
                    
                    if !self.awx.is_empty() {
                        self.rvk();
                    }
                } else {
                    self.bxo = EditTool::Jj;
                    self.bgv = None;
                    self.aoc = String::from("Tool: Delete — Click to remove");
                }
            },
            
            
            b'z' | b'Z' => self.ifu(),
            
            
            b'h' | b'H' => {
                self.ial = !self.ial;
                self.aoc = format!("Tips: {}", if self.ial { "ON" } else { "OFF" });
            },
            b'v' | b'V' => {
                self.jqd = !self.jqd;
            },
            
            
            b'1' => self.gls("cube"),
            b'2' => self.gls("pyramid"),
            b'3' => self.gls("diamond"),
            b'4' => self.gls("torus"),
            b'5' => self.gls("icosphere"),
            b'6' => self.gls("grid"),
            
            
            b'w' | b'W' => self.mbr(),
            b'l' | b'L' => {
                let path = format!("/{}.t3d", self.hrr);
                self.load(&path);
            },
            
            
            b'c' | b'C' => self.clear(),
            
            
            AH_ => self.bbc -= 0.15,
            AI_ => self.bbc += 0.15,
            V_ => {
                self.atx = (self.atx - 0.15).am(-1.5);
            },
            U_ => {
                self.atx = (self.atx + 0.15).v(1.5);
            },
            
            
            b'd' | b'D' => {
                self.awx.clear();
                self.bgv = None;
                self.aoc = String::from("Deselected all");
            },
            
            
            b'+' | b'=' => {
                self.aab = (self.aab - 0.5).am(1.5);
            },
            b'-' | b'_' => {
                self.aab = (self.aab + 0.5).v(20.0);
            },
            
            
            b'r' | b'R' => {
                self.bbc = 0.6;
                self.atx = 0.4;
                self.aab = 6.0;
                self.eng = V3 { b: 0.0, c: 0.0, av: 0.0 };
                self.aoc = String::from("Camera reset");
            },
            
            _ => {}
        }
    }
    
    
    pub fn ago(&mut self, fp: i32, iz: i32, gm: usize, me: usize, vn: bool) {
        if vn {
            self.jgf = true;
            self.kqs = fp;
            self.kqt = iz;
            self.hrx = false;
            
            match self.bxo {
                EditTool::Qs => {
                    if let Some(w) = self.hjn(fp, iz, gm, me) {
                        if self.awx.contains(&w) {
                            self.awx.ajm(|&p| p != w);
                        } else {
                            self.awx.push(w);
                        }
                        self.aoc = format!("Selected {} vertex(es)", self.awx.len());
                    } else {
                        self.awx.clear();
                        self.aoc = String::from("Selection cleared");
                    }
                },
                EditTool::Rh => {
                    self.bru();
                    let dnh = self.xol(fp, iz, gm, me);
                    self.lm.push(dnh);
                    self.aoc = format!("Added vertex at ({:.1}, {:.1}, {:.1}) — {} total",
                        dnh.b, dnh.c, dnh.av, self.lm.len());
                },
                EditTool::Rg => {
                    if let Some(w) = self.hjn(fp, iz, gm, me) {
                        if let Some(ay) = self.bgv {
                            if ay != w {
                                
                                let aja = self.bu.iter().any(|&(q, o)| 
                                    (q == ay && o == w) || (q == w && o == ay));
                                if !aja {
                                    self.bru();
                                    self.bu.push((ay, w));
                                    self.aoc = format!("Edge {} -> {} created — {} edges total",
                                        ay, w, self.bu.len());
                                } else {
                                    self.aoc = String::from("Edge already exists");
                                }
                            }
                            self.bgv = None;
                        } else {
                            self.bgv = Some(w);
                            self.aoc = format!("Edge start: vertex {} — Click another vertex", w);
                        }
                    } else {
                        self.bgv = None;
                        self.aoc = String::from("No vertex found — click on a vertex");
                    }
                },
                EditTool::Fw => {
                    if let Some(w) = self.hjn(fp, iz, gm, me) {
                        if !self.awx.contains(&w) {
                            self.awx.clear();
                            self.awx.push(w);
                        }
                        self.bru();
                    }
                },
                EditTool::Jj => {
                    if let Some(w) = self.hjn(fp, iz, gm, me) {
                        self.bru();
                        self.rvl(w);
                        self.aoc = format!("Deleted vertex {} — {} remaining", w, self.lm.len());
                    }
                },
            }
        } else {
            self.jgf = false;
            self.hrx = false;
        }
    }
    
    
    pub fn lax(&mut self, fp: i32, iz: i32, gm: usize, me: usize) {
        let dx = fp - self.daa;
        let bg = iz - self.dab;
        self.daa = fp;
        self.dab = iz;
        
        
        self.lcn = self.hjn(fp, iz, gm, me);
        
        if self.jgf {
            let jtq = (fp - self.kqs).gp();
            let xkc = (iz - self.kqt).gp();
            if jtq > 3 || xkc > 3 {
                self.hrx = true;
            }
            
            if self.hrx {
                match self.bxo {
                    EditTool::Fw if !self.awx.is_empty() => {
                        
                        let oob = 0.01 * self.aab;
                        let ooc = dx as f32 * oob;
                        let upx = -bg as f32 * oob;
                        
                        
                        let ae = rk(self.bbc);
                        let cq = lz(self.bbc);
                        
                        for &w in &self.awx.clone() {
                            if w < self.lm.len() {
                                self.lm[w].b += ooc * ae;
                                self.lm[w].av += ooc * cq;
                                self.lm[w].c += upx;
                            }
                        }
                    },
                    EditTool::Qs | EditTool::Rh | EditTool::Rg | EditTool::Jj => {
                        
                        self.bbc += dx as f32 * 0.01;
                        self.atx = (self.atx + bg as f32 * 0.01).qp(-1.5, 1.5);
                    },
                    _ => {}
                }
            }
        }
    }
    
    
    pub fn ers(&mut self, aaq: i8) {
        if aaq > 0 {
            self.aab = (self.aab - 0.5).am(1.5);
        } else {
            self.aab = (self.aab + 0.5).v(20.0);
        }
    }
    
    fn rvl(&mut self, w: usize) {
        if w >= self.lm.len() { return; }
        self.lm.remove(w);
        
        self.bu.ajm(|&(q, o)| q != w && o != w);
        for amd in &mut self.bu {
            if amd.0 > w { amd.0 -= 1; }
            if amd.1 > w { amd.1 -= 1; }
        }
        self.awx.ajm(|&p| p != w);
        for bxk in &mut self.awx {
            if *bxk > w { *bxk -= 1; }
        }
    }
    
    fn rvk(&mut self) {
        if self.awx.is_empty() { return; }
        self.bru();
        
        let mut cik = self.awx.clone();
        cik.zox();
        cik.dbh();
        for w in cik {
            if w < self.lm.len() {
                self.lm.remove(w);
                self.bu.ajm(|&(q, o)| q != w && o != w);
                for amd in &mut self.bu {
                    if amd.0 > w { amd.0 -= 1; }
                    if amd.1 > w { amd.1 -= 1; }
                }
            }
        }
        let az = self.awx.len();
        self.awx.clear();
        self.aoc = format!("Deleted {} vertices", az);
    }
    
    
    
    pub fn tj(&mut self, k: &mut [u32], d: usize, i: usize) {
        self.frame += 1;
        
        
        self.kga = rk(self.bbc);
        self.kgc = lz(self.bbc);
        self.kfz = rk(self.atx);
        self.kgb = lz(self.atx);
        
        let bpb = 32;
        let bfm = 20;
        let fyh = bpb;
        let mpg = i.ao(bpb + bfm);
        
        
        for il in k.el() {
            *il = MH_;
        }
        
        
        self.sgh(k, d, i, bpb);
        
        
        if mpg > 10 {
            self.sgl(k, d, i, 0, fyh, d, mpg);
        }
        
        
        self.hgw(k, d, i, i - bfm, bfm);
        
        
        if self.ial {
            self.sgg(k, d, i, fyh, mpg);
        }
    }
    
    fn sgh(&self, k: &mut [u32], d: usize, i: usize, ejt: usize) {
        
        for c in 0..ejt {
            for b in 0..d {
                k[c * d + b] = SG_;
            }
        }
        
        
        if ejt < i {
            for b in 0..d {
                k[ejt * d + b] = 0xFF333355;
            }
        }
        
        
        let mlu = [EditTool::Qs, EditTool::Rh, EditTool::Rg, EditTool::Fw, EditTool::Jj];
        let pm = 60;
        let qx = 22;
        let kn = 5;
        let mut bx = 8;
        
        for bxo in &mlu {
            let gh = self.bxo == *bxo;
            let ei = if gh { BOD_ } else { MI_ };
            ah(k, d, i, bx, kn, pm, qx, ei);
            
            lx(k, d, i, bx, kn, pm, qx, if gh { 0xFF00AAFF } else { 0xFF444466 });
            
            let cu = format!("{} [{}]", bxo.j(), bxo.tpy());
            bbl(k, d, i, bx + 3, kn + 7, &cu, 
                if gh { AOR_ } else { T_ });
            bx += pm + 4;
        }
        
        
        let hvx = ["1:Cube", "2:Pyr", "3:Dia", "4:Tor", "5:Ico", "6:Grid"];
        let mut y = d - 8;
        for akl in hvx.iter().vv() {
            let ars = akl.len() * 6 + 8;
            y -= ars;
            ah(k, d, i, y, kn, ars, qx, MI_);
            lx(k, d, i, y, kn, ars, qx, 0xFF444466);
            bbl(k, d, i, y + 4, kn + 7, akl, AAK_);
            y -= 4;
        }
        
        
        let dbq = bx + 20;
        ah(k, d, i, dbq, kn, 50, qx, MI_);
        lx(k, d, i, dbq, kn, 50, qx, 0xFF444466);
        bbl(k, d, i, dbq + 4, kn + 7, "Save[W]", T_);
        
        ah(k, d, i, dbq + 54, kn, 50, qx, MI_);
        lx(k, d, i, dbq + 54, kn, 50, qx, 0xFF444466);
        bbl(k, d, i, dbq + 58, kn + 7, "Load[L]", T_);
        
        ah(k, d, i, dbq + 108, kn, 50, qx, MI_);
        lx(k, d, i, dbq + 108, kn, 50, qx, 0xFF444466);
        bbl(k, d, i, dbq + 112, kn + 7, "Undo[Z]", T_);
    }
    
    fn sgl(&self, k: &mut [u32], nm: usize, adn: usize,
                     fp: usize, iz: usize, gm: usize, me: usize) {
        
        if self.pkm {
            self.fgx(k, nm, adn, fp, iz, gm, me);
        }
        
        
        if self.pkl {
            self.sbt(k, nm, adn, fp, iz, gm, me);
        }
        
        
        for (w, &(q, o)) in self.bu.iter().cf() {
            if q >= self.lm.len() || o >= self.lm.len() { continue; }
            let (fy, fo, alw) = self.dkv(self.lm[q], gm, me);
            let (dn, dp, aeu) = self.dkv(self.lm[o], gm, me);
            if alw < 0.1 || aeu < 0.1 { continue; }
            
            
            let fua = self.awx.contains(&q) || self.awx.contains(&o);
            let txi = self.bgv == Some(q) || self.bgv == Some(o);
            
            let s = if txi { 0xFFFF8800 } else if fua { BOJ_ } else { BOI_ };
            
            
            let bth = (alw + aeu) * 0.5;
            let yx = (1.0 - (bth - 3.0) * 0.1).qp(0.3, 1.0);
            let s = sqr(s, yx);
            
            dgr(k, nm, adn, 
                fp as i32 + fy, iz as i32 + fo,
                fp as i32 + dn, iz as i32 + dp, s);
        }
        
        
        if self.jqd {
            for (a, p) in self.lm.iter().cf() {
                let (cr, cq, av) = self.dkv(*p, gm, me);
                if av < 0.1 { continue; }
                
                let y = fp as i32 + cr;
                let x = iz as i32 + cq;
                
                let na = self.awx.contains(&a);
                let asy = self.lcn == Some(a);
                let bgv = self.bgv == Some(a);
                
                let s = if bgv { 0xFFFF8800 }
                    else if na { BOH_ }
                    else if asy { BOG_ }
                    else { BOF_ };
                
                let aw = if na || asy || bgv { 3 } else { 2 };
                
                
                for bg in -aw..=aw {
                    for dx in -aw..=aw {
                        if dx * dx + bg * bg <= aw * aw {
                            oyo(k, nm, adn, y + dx, x + bg, s);
                        }
                    }
                }
                
                
                if na || asy {
                    let cu = format!("{}", a);
                    bbl(k, nm, adn, (y + aw + 3) as usize, (x - 3) as usize, &cu, s);
                }
            }
        }
        
        
        if let Some(dlz) = self.bgv {
            if dlz < self.lm.len() {
                let (cr, cq, av) = self.dkv(self.lm[dlz], gm, me);
                if av > 0.1 {
                    
                    scn(k, nm, adn,
                        fp as i32 + cr, iz as i32 + cq,
                        fp as i32 + self.daa, iz as i32 + self.dab,
                        0xFFFF8800);
                }
            }
        }
    }
    
    fn fgx(&self, k: &mut [u32], nm: usize, adn: usize,
                 fp: usize, iz: usize, gm: usize, me: usize) {
        let iv = ATW_ * 0.5;
        let gu = ATW_ / ADD_ as f32;
        
        for a in 0..=ADD_ {
            let ab = -iv + a as f32 * gu;
            let tww = (a == ADD_ / 2);
            let s = if tww { BNU_ } else { BNT_ };
            
            
            let (fy, fo, alw) = self.dkv(V3 { b: -iv, c: 0.0, av: ab }, gm, me);
            let (dn, dp, aeu) = self.dkv(V3 { b: iv, c: 0.0, av: ab }, gm, me);
            if alw > 0.1 && aeu > 0.1 {
                dgr(k, nm, adn, 
                    fp as i32 + fy, iz as i32 + fo,
                    fp as i32 + dn, iz as i32 + dp, s);
            }
            
            
            let (fy, fo, alw) = self.dkv(V3 { b: ab, c: 0.0, av: -iv }, gm, me);
            let (dn, dp, aeu) = self.dkv(V3 { b: ab, c: 0.0, av: iv }, gm, me);
            if alw > 0.1 && aeu > 0.1 {
                dgr(k, nm, adn, 
                    fp as i32 + fy, iz as i32 + fo,
                    fp as i32 + dn, iz as i32 + dp, s);
            }
        }
    }
    
    fn sbt(&self, k: &mut [u32], nm: usize, adn: usize,
                 fp: usize, iz: usize, gm: usize, me: usize) {
        let atf = V3 { b: 0.0, c: 0.0, av: 0.0 };
        let qly = [
            (V3 { b: ZG_, c: 0.0, av: 0.0 }, BNQ_, "X"),
            (V3 { b: 0.0, c: ZG_, av: 0.0 }, BNR_, "Y"),
            (V3 { b: 0.0, c: 0.0, av: ZG_ }, BNS_, "Z"),
        ];
        
        let (mp, qw, jig) = self.dkv(atf, gm, me);
        if jig < 0.1 { return; }
        
        for (ci, s, cu) in &qly {
            let (bqp, ahm, sqo) = self.dkv(*ci, gm, me);
            if sqo < 0.1 { continue; }
            
            
            for bc in -1..=1i32 {
                dgr(k, nm, adn,
                    fp as i32 + mp + bc, iz as i32 + qw,
                    fp as i32 + bqp + bc, iz as i32 + ahm, *s);
                dgr(k, nm, adn,
                    fp as i32 + mp, iz as i32 + qw + bc,
                    fp as i32 + bqp, iz as i32 + ahm + bc, *s);
            }
            
            
            bbl(k, nm, adn, (fp as i32 + bqp + 5) as usize, (iz as i32 + ahm - 3) as usize,
                cu, *s);
        }
    }
    
    fn hgw(&self, k: &mut [u32], d: usize, i: usize, cq: usize, kl: usize) {
        
        for c in cq..cq + kl {
            if c >= i { break; }
            for b in 0..d {
                k[c * d + b] = AAJ_;
            }
        }
        
        if cq > 0 && cq < i {
            for b in 0..d {
                k[cq * d + b] = 0xFF333355;
            }
        }
        
        
        bbl(k, d, i, 8, cq + 6, &self.aoc, T_);
        
        
        let cm = format!("V:{} E:{} | {} | Zoom:{:.1}", 
            self.lm.len(), self.bu.len(), self.hrr, self.aab);
        let wtq = d.ao(cm.len() * 6 + 8);
        bbl(k, d, i, wtq, cq + 6, &cm, AAK_);
    }
    
    fn sgg(&self, k: &mut [u32], d: usize, i: usize, iz: usize, me: usize) {
        let yd = 200;
        let ans = 180;
        let awm = d.ao(yd + 10);
        let atg = iz + 10;
        
        if atg + ans >= i { return; }
        
        
        for c in atg..atg + ans {
            for b in awm..awm + yd {
                if c < i && b < d {
                    k[c * d + b] = fdm(k[c * d + b], AOS_);
                }
            }
        }
        
        
        lx(k, d, i, awm, atg, yd, ans, SF_);
        
        let mut ty = atg + 6;
        let gx = awm + 8;
        
        bbl(k, d, i, gx, ty, "--- Controls ---", SF_);
        ty += 12;
        bbl(k, d, i, gx, ty, "S: Select  A: Add Vtx", T_);
        ty += 10;
        bbl(k, d, i, gx, ty, "E: Add Edge  G: Move", T_);
        ty += 10;
        bbl(k, d, i, gx, ty, "X: Delete  D: Deselect", T_);
        ty += 10;
        bbl(k, d, i, gx, ty, "Z: Undo  C: Clear", T_);
        ty += 10;
        bbl(k, d, i, gx, ty, "Arrows: Orbit camera", T_);
        ty += 10;
        bbl(k, d, i, gx, ty, "+/-: Zoom  R: Reset cam", T_);
        ty += 14;
        bbl(k, d, i, gx, ty, "--- Presets ---", SF_);
        ty += 12;
        bbl(k, d, i, gx, ty, "1:Cube 2:Pyr 3:Dia", T_);
        ty += 10;
        bbl(k, d, i, gx, ty, "4:Torus 5:Ico 6:Grid", T_);
        ty += 14;
        bbl(k, d, i, gx, ty, "--- File ---", SF_);
        ty += 12;
        bbl(k, d, i, gx, ty, "W: Save  L: Load", T_);
        ty += 10;
        bbl(k, d, i, gx, ty, "H: Toggle this panel", AAK_);
        
        
        let idq = self.bxo.idq();
        if idq.len() > 0 {
            let fwt = atg + ans + 5;
            if fwt + 16 < i {
                let xho = idq.len() * 6 + 12;
                ah(k, d, i, awm, fwt, xho, 14, AOS_ & 0xDDFFFFFF);
                bbl(k, d, i, awm + 6, fwt + 3, idq, AOR_);
            }
        }
    }
}



use crate::draw_utils::{
    sf as oyo,
    ah,
    lx,
    ahj as dgr,
};

fn scn(k: &mut [u32], d: usize, i: usize, fy: i32, fo: i32, dn: i32, dp: i32, s: u32) {
    let mut b = fy;
    let mut c = fo;
    let dx = (dn - fy).gp();
    let bg = -(dp - fo).gp();
    let cr = if fy < dn { 1 } else { -1 };
    let cq = if fo < dp { 1 } else { -1 };
    let mut rq = dx + bg;
    let mut gu = 0;
    let csk = (dx.gp() + bg.gp()) as usize + 1;
    let csk = csk.v(4000);
    
    for _ in 0..csk {
        if gu % 8 < 4 {
            oyo(k, d, i, b, c, s);
        }
        gu += 1;
        if b == dn && c == dp { break; }
        let agl = 2 * rq;
        if agl >= bg { rq += bg; b += cr; }
        if agl <= dx { rq += dx; c += cq; }
    }
}

fn sqr(s: u32, pv: f32) -> u32 {
    let m = ((s >> 16) & 0xFF) as f32 * pv;
    let at = ((s >> 8) & 0xFF) as f32 * pv;
    let o = (s & 0xFF) as f32 * pv;
    0xFF000000 | ((m as u32) << 16) | ((at as u32) << 8) | (o as u32)
}

fn fdm(ei: u32, lp: u32) -> u32 {
    let dw = ((lp >> 24) & 0xFF) as f32 / 255.0;
    let wq = 1.0 - dw;
    let m = ((lp >> 16) & 0xFF) as f32 * dw + ((ei >> 16) & 0xFF) as f32 * wq;
    let at = ((lp >> 8) & 0xFF) as f32 * dw + ((ei >> 8) & 0xFF) as f32 * wq;
    let o = (lp & 0xFF) as f32 * dw + (ei & 0xFF) as f32 * wq;
    0xFF000000 | ((m as u32) << 16) | ((at as u32) << 8) | (o as u32)
}


fn bbl(k: &mut [u32], nm: usize, adn: usize, b: usize, c: usize, text: &str, s: u32) {
    let mut cx = b;
    for bm in text.bf() {
        sfp(k, nm, adn, cx, c, bm, s);
        cx += 6;
    }
}


fn sfp(k: &mut [u32], nm: usize, adn: usize, b: usize, c: usize, bm: u8, s: u32) {
    
    let r = bm as char;
    if r == ' ' { return; }
    
    
    let fs: [u8; 7] = match r {
        '0' => [0b01110, 0b10001, 0b10011, 0b10101, 0b11001, 0b10001, 0b01110],
        '1' => [0b00100, 0b01100, 0b00100, 0b00100, 0b00100, 0b00100, 0b01110],
        '2' => [0b01110, 0b10001, 0b00001, 0b00110, 0b01000, 0b10000, 0b11111],
        '3' => [0b01110, 0b10001, 0b00001, 0b00110, 0b00001, 0b10001, 0b01110],
        '4' => [0b00010, 0b00110, 0b01010, 0b10010, 0b11111, 0b00010, 0b00010],
        '5' => [0b11111, 0b10000, 0b11110, 0b00001, 0b00001, 0b10001, 0b01110],
        '6' => [0b00110, 0b01000, 0b10000, 0b11110, 0b10001, 0b10001, 0b01110],
        '7' => [0b11111, 0b00001, 0b00010, 0b00100, 0b01000, 0b01000, 0b01000],
        '8' => [0b01110, 0b10001, 0b10001, 0b01110, 0b10001, 0b10001, 0b01110],
        '9' => [0b01110, 0b10001, 0b10001, 0b01111, 0b00001, 0b00010, 0b01100],
        'A' | 'a' => [0b01110, 0b10001, 0b10001, 0b11111, 0b10001, 0b10001, 0b10001],
        'B' | 'b' => [0b11110, 0b10001, 0b10001, 0b11110, 0b10001, 0b10001, 0b11110],
        'C' | 'c' => [0b01110, 0b10001, 0b10000, 0b10000, 0b10000, 0b10001, 0b01110],
        'D' | 'd' => [0b11110, 0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b11110],
        'E' | 'e' => [0b11111, 0b10000, 0b10000, 0b11110, 0b10000, 0b10000, 0b11111],
        'F' | 'f' => [0b11111, 0b10000, 0b10000, 0b11110, 0b10000, 0b10000, 0b10000],
        'G' | 'g' => [0b01110, 0b10001, 0b10000, 0b10111, 0b10001, 0b10001, 0b01111],
        'H' | 'h' => [0b10001, 0b10001, 0b10001, 0b11111, 0b10001, 0b10001, 0b10001],
        'I' | 'i' => [0b01110, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100, 0b01110],
        'J' | 'j' => [0b00111, 0b00010, 0b00010, 0b00010, 0b00010, 0b10010, 0b01100],
        'K' | 'k' => [0b10001, 0b10010, 0b10100, 0b11000, 0b10100, 0b10010, 0b10001],
        'L' | 'l' => [0b10000, 0b10000, 0b10000, 0b10000, 0b10000, 0b10000, 0b11111],
        'M' | 'm' => [0b10001, 0b11011, 0b10101, 0b10101, 0b10001, 0b10001, 0b10001],
        'N' | 'n' => [0b10001, 0b11001, 0b10101, 0b10011, 0b10001, 0b10001, 0b10001],
        'O' | 'o' => [0b01110, 0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b01110],
        'P' | 'p' => [0b11110, 0b10001, 0b10001, 0b11110, 0b10000, 0b10000, 0b10000],
        'Q' | 'q' => [0b01110, 0b10001, 0b10001, 0b10001, 0b10101, 0b10010, 0b01101],
        'R' | 'r' => [0b11110, 0b10001, 0b10001, 0b11110, 0b10100, 0b10010, 0b10001],
        'S' | 's' => [0b01110, 0b10001, 0b10000, 0b01110, 0b00001, 0b10001, 0b01110],
        'T' | 't' => [0b11111, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100],
        'U' | 'u' => [0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b01110],
        'V' | 'v' => [0b10001, 0b10001, 0b10001, 0b10001, 0b01010, 0b01010, 0b00100],
        'W' | 'w' => [0b10001, 0b10001, 0b10001, 0b10101, 0b10101, 0b11011, 0b10001],
        'X' | 'x' => [0b10001, 0b10001, 0b01010, 0b00100, 0b01010, 0b10001, 0b10001],
        'Y' | 'y' => [0b10001, 0b10001, 0b01010, 0b00100, 0b00100, 0b00100, 0b00100],
        'Z' | 'z' => [0b11111, 0b00001, 0b00010, 0b00100, 0b01000, 0b10000, 0b11111],
        ':' => [0b00000, 0b00100, 0b00100, 0b00000, 0b00100, 0b00100, 0b00000],
        '.' => [0b00000, 0b00000, 0b00000, 0b00000, 0b00000, 0b00100, 0b00100],
        ',' => [0b00000, 0b00000, 0b00000, 0b00000, 0b00100, 0b00100, 0b01000],
        '-' => [0b00000, 0b00000, 0b00000, 0b11111, 0b00000, 0b00000, 0b00000],
        '+' => [0b00000, 0b00100, 0b00100, 0b11111, 0b00100, 0b00100, 0b00000],
        '=' => [0b00000, 0b00000, 0b11111, 0b00000, 0b11111, 0b00000, 0b00000],
        '/' => [0b00001, 0b00010, 0b00010, 0b00100, 0b01000, 0b01000, 0b10000],
        '(' => [0b00010, 0b00100, 0b01000, 0b01000, 0b01000, 0b00100, 0b00010],
        ')' => [0b01000, 0b00100, 0b00010, 0b00010, 0b00010, 0b00100, 0b01000],
        '[' => [0b01110, 0b01000, 0b01000, 0b01000, 0b01000, 0b01000, 0b01110],
        ']' => [0b01110, 0b00010, 0b00010, 0b00010, 0b00010, 0b00010, 0b01110],
        '>' => [0b10000, 0b01000, 0b00100, 0b00010, 0b00100, 0b01000, 0b10000],
        '<' => [0b00010, 0b00100, 0b01000, 0b10000, 0b01000, 0b00100, 0b00010],
        '|' => [0b00100, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100],
        '_' => [0b00000, 0b00000, 0b00000, 0b00000, 0b00000, 0b00000, 0b11111],
        '!' => [0b00100, 0b00100, 0b00100, 0b00100, 0b00100, 0b00000, 0b00100],
        '?' => [0b01110, 0b10001, 0b00001, 0b00110, 0b00100, 0b00000, 0b00100],
        '#' => [0b01010, 0b01010, 0b11111, 0b01010, 0b11111, 0b01010, 0b01010],
        '*' => [0b00000, 0b00100, 0b10101, 0b01110, 0b10101, 0b00100, 0b00000],
        '~' => [0b00000, 0b00000, 0b01000, 0b10101, 0b00010, 0b00000, 0b00000],
        _ => [0b01110, 0b01010, 0b01010, 0b01010, 0b01010, 0b01010, 0b01110], 
    };
    
    for br in 0..7 {
        for bj in 0..5 {
            if fs[br] & (1 << (4 - bj)) != 0 {
                let y = b + bj;
                let x = c + br;
                if y < nm && x < adn {
                    k[x * nm + y] = s;
                }
            }
        }
    }
}
