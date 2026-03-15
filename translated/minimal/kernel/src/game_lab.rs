












extern crate alloc;
use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;
use alloc::vec;
use crate::framebuffer;
use crate::gameboy::GameBoyEmulator;


const LI_: u32 = 28;
const KX_: u32 = 4;
const K_: u32 = 14;
const GB_: u32 = 8;
const JE_: u32 = 24;


const MH_: u32         = 0xFF0A0F14;
const BNY_: u32      = 0xFF111920;
const JR_: u32     = 0xFF1E2A36;
const SE_: u32  = 0xFF142028;
const T_: u32       = 0xFF9CD8B0;   
const F_: u32        = 0xFF4A6A54;   
const BE_: u32     = 0xFF00FF88;   
const O_: u32     = 0xFF58A6FF;   
const AF_: u32      = 0xFFE0F8D0;   
const ED_: u32        = 0xFF80FFAA;   
const DP_: u32    = 0xFF00FF66;   
const SD_: u32   = 0xFF2A3A30;   
const MJ_: u32       = 0xFFD29922;   
const AW_: u32        = 0xFFF85149;   
const BB_: u32       = 0xFF79C0FF;   
const BO_: u32     = 0xFFBC8CFF;   
const HR_: u32       = 0xFF507060;   
const SC_: u32    = 0xFFFF4444;   
const SG_: u32    = 0xFF0E1820;

const BAG_: usize = 16;
const CFQ_: usize = 256;
const CYJ_: usize = 64;
const CFE_: usize = 8;


#[derive(Clone, Copy, PartialEq)]
pub enum LabTab {
    Zt = 0,
    Ys = 1,
    Aoe = 2,
    Oh = 3,
    Ze = 4,
}


#[derive(Clone)]
pub struct Bwr {
    pub ag: u16,
    pub cu: [u8; 8],    
    pub jce: u8,
    pub jjy: u8,
    pub hes: u8,
    pub cpa: bool,
    pub aw: u8,           
}


#[derive(Clone, Copy, PartialEq)]
pub enum SearchMode {
    Ho,      
    Aaj,    
    Afd,  
    Ss,    
    Tg,       
}


#[derive(Clone, Copy)]
pub struct Azx {
    pub fz: u16,
    pub opcode: u8,
    pub q: u8, pub bb: u8,
    pub sp: u16,
}


pub struct SaveState {
    pub kkz: u8, pub klf: u8,
    pub kla: u8, pub klc: u8,
    pub kld: u8, pub kle: u8,
    pub klg: u8, pub klj: u8,
    pub klo: u16, pub klk: u16,
    pub kli: bool, pub klh: bool,
    pub brc: u8, pub bhf: u8,
    pub cfu: u8,
    pub aow: u8, pub aox: u8,
    pub cht: u8, pub chs: u8,
    pub axj: u8, pub beq: u8,
    pub aec: Vec<u8>,
    pub bux: [u8; 127],
    pub kzo: u8, pub kzz: u8,
    pub kzy: u8, pub kzx: u8,
    pub kzp: u8, pub kzq: u8,
    pub kzl: u8, pub kzu: u8, pub kzv: u8,
    pub lag: u8, pub laf: u8,
    pub kzr: u8, pub kzm: u32,
    pub lab: [u8; 8192],
    pub lac: [u8; 8192],
    pub kzs: [u8; 160],
    pub kzk: [u8; 64],
    pub kzt: [u8; 64],
    pub lad: u8,
    pub kzj: u8, pub kzw: u8,
    pub lae: u8,
    pub mkw: u16, pub mla: u8,
    pub mlb: u8, pub mkz: u8,
    pub kgs: u16, pub kgq: u8,
    pub kgr: bool, pub kgp: u8,
    pub imy: Vec<u8>,
    pub blq: bool,
}

impl SaveState {
    pub fn azs() -> Self {
        Self {
            kkz: 0, klf: 0, kla: 0, klc: 0,
            kld: 0, kle: 0, klg: 0, klj: 0,
            klo: 0, klk: 0, kli: false, klh: false,
            brc: 0, bhf: 0, cfu: 0,
            aow: 0xF, aox: 0xF,
            cht: 0, chs: 0,
            axj: 1, beq: 0,
            aec: Vec::new(), bux: [0; 127],
            kzo: 0, kzz: 0, kzy: 0, kzx: 0,
            kzp: 0, kzq: 0, kzl: 0, kzu: 0, kzv: 0,
            lag: 0, laf: 0, kzr: 0, kzm: 0,
            lab: [0; 8192], lac: [0; 8192],
            kzs: [0; 160],
            kzk: [0; 64], kzt: [0; 64],
            lad: 0, kzj: 0, kzw: 0,
            lae: 0,
            mkw: 0, mla: 0, mlb: 0, mkz: 0,
            kgs: 1, kgq: 0,
            kgr: false, kgp: 0,
            imy: Vec::new(),
            blq: false,
        }
    }
}


pub struct GameLabState {
    
    pub fnb: Option<u32>,
    
    pub bnn: u16,
    
    pub eyi: u8,
    
    pub frame: u32,
    
    pub foa: u8,
    
    pub jc: u32,

    
    pub ahd: LabTab,

    
    pub eku: Vec<Bwr>,

    
    pub fty: u16,         
    pub joa: [u8; 6],     
    pub eif: u8,
    pub chq: Vec<u16>,  
    pub ftx: Vec<u8>,  
    pub grs: SearchMode,
    pub bcn: bool,       
    pub wfk: bool,    

    
    pub gbl: Vec<u16>,     
    pub qrr: [u8; 5],        
    pub qrs: u8,
    pub ant: bool,
    pub dwj: bool,            
    pub dwi: bool,          

    
    
    pub cmx: u8,

    
    pub trace: Vec<Azx>,
    pub eka: bool,

    
    pub ejv: u8,   
    pub gum: u32,

    
    pub jfs: [u8; 256], 

    
    pub fto: SaveState,
}

impl GameLabState {
    pub fn new() -> Self {
        Self {
            fnb: None,
            bnn: 0xC000,
            eyi: 0,
            frame: 0,
            foa: 0,
            jc: 0,
            ahd: LabTab::Zt,
            eku: Vec::new(),
            fty: 0,
            joa: [0; 6],
            eif: 0,
            chq: Vec::new(),
            ftx: Vec::new(),
            grs: SearchMode::Ho,
            bcn: false,
            wfk: true,
            gbl: Vec::new(),
            qrr: [0; 5],
            qrs: 0,
            ant: false,
            dwj: false,
            dwi: false,
            cmx: 2, 
            trace: Vec::new(),
            eka: false,
            ejv: 0,
            gum: 0,
            jfs: [0; 256],
            fto: SaveState::azs(),
        }
    }

    pub fn zpc(&self) -> f32 {
        match self.cmx {
            0 => 0.25,
            1 => 0.5,
            2 => 1.0,
            3 => 2.0,
            4 => 4.0,
            _ => 1.0,
        }
    }

    pub fn wqy(&self) -> &'static str {
        match self.cmx {
            0 => "0.25x",
            1 => "0.5x",
            2 => "1x",
            3 => "2x",
            4 => "4x",
            _ => "1x",
        }
    }

    
    pub fn ziv(&mut self, cw: &GameBoyEmulator) {
        if !self.eka { return; }
        let fz = cw.cpu.fz;
        let opcode = boa(cw, fz);
        let bt = Azx {
            fz,
            opcode,
            q: cw.cpu.q,
            bb: cw.cpu.bb,
            sp: cw.cpu.sp,
        };
        if self.trace.len() >= CYJ_ {
            self.trace.remove(0);
        }
        self.trace.push(bt);
    }

    
    pub fn pkj(&self, fz: u16) -> bool {
        self.gbl.iter().any(|&bp| bp == fz)
    }

    
    pub fn wcv(&mut self, cw: &GameBoyEmulator) {
        let e = &mut self.fto;
        e.kkz = cw.cpu.q; e.klf = cw.cpu.bb;
        e.kla = cw.cpu.o; e.klc = cw.cpu.r;
        e.kld = cw.cpu.bc; e.kle = cw.cpu.aa;
        e.klg = cw.cpu.i; e.klj = cw.cpu.dm;
        e.klo = cw.cpu.sp; e.klk = cw.cpu.fz;
        e.kli = cw.cpu.dih; e.klh = cw.cpu.dhv;
        e.brc = cw.brc; e.bhf = cw.bhf;
        e.cfu = cw.cfu;
        e.aow = cw.aow;
        e.aox = cw.aox;
        e.cht = cw.cht;
        e.chs = cw.chs;
        e.axj = cw.axj; e.beq = cw.beq;
        e.aec = cw.aec.clone();
        e.bux = cw.bux;
        e.kzo = cw.gpu.amh; e.kzz = cw.gpu.hm;
        e.kzy = cw.gpu.eyf; e.kzx = cw.gpu.eye;
        e.kzp = cw.gpu.ct; e.kzq = cw.gpu.eey;
        e.kzl = cw.gpu.emt; e.kzu = cw.gpu.fpm; e.kzv = cw.gpu.fpn;
        e.lag = cw.gpu.lw; e.laf = cw.gpu.fx;
        e.kzr = cw.gpu.ev; e.kzm = cw.gpu.yl;
        e.lab = cw.gpu.aof;
        e.lac = cw.gpu.dnb;
        e.kzs = cw.gpu.awh;
        e.kzk = cw.gpu.bdo;
        e.kzt = cw.gpu.fpk;
        e.lad = cw.gpu.fbb;
        e.kzj = cw.gpu.doj; e.kzw = cw.gpu.dtv;
        e.lae = cw.gpu.ekz;
        e.mkw = cw.timer.div; e.mla = cw.timer.ejw;
        e.mlb = cw.timer.fww; e.mkz = cw.timer.ezl;
        e.kgs = cw.on.bwu;
        e.kgq = cw.on.brv;
        e.kgr = cw.on.ctr;
        e.kgp = cw.on.ev;
        e.imy = cw.on.ajl.clone();
        e.blq = true;
    }

    
    pub fn uhb(&self, cw: &mut GameBoyEmulator) {
        let e = &self.fto;
        if !e.blq { return; }
        cw.cpu.q = e.kkz; cw.cpu.bb = e.klf;
        cw.cpu.o = e.kla; cw.cpu.r = e.klc;
        cw.cpu.bc = e.kld; cw.cpu.aa = e.kle;
        cw.cpu.i = e.klg; cw.cpu.dm = e.klj;
        cw.cpu.sp = e.klo; cw.cpu.fz = e.klk;
        cw.cpu.dih = e.kli; cw.cpu.dhv = e.klh;
        cw.brc = e.brc; cw.bhf = e.bhf;
        cw.cfu = e.cfu;
        cw.aow = e.aow;
        cw.aox = e.aox;
        cw.cht = e.cht;
        cw.chs = e.chs;
        cw.axj = e.axj; cw.beq = e.beq;
        if e.aec.len() == cw.aec.len() {
            cw.aec.dg(&e.aec);
        }
        cw.bux = e.bux;
        cw.gpu.amh = e.kzo; cw.gpu.hm = e.kzz;
        cw.gpu.eyf = e.kzy; cw.gpu.eye = e.kzx;
        cw.gpu.ct = e.kzp; cw.gpu.eey = e.kzq;
        cw.gpu.emt = e.kzl; cw.gpu.fpm = e.kzu; cw.gpu.fpn = e.kzv;
        cw.gpu.lw = e.lag; cw.gpu.fx = e.laf;
        cw.gpu.ev = e.kzr; cw.gpu.yl = e.kzm;
        cw.gpu.aof = e.lab;
        cw.gpu.dnb = e.lac;
        cw.gpu.awh = e.kzs;
        cw.gpu.bdo = e.kzk;
        cw.gpu.fpk = e.kzt;
        cw.gpu.fbb = e.lad;
        cw.gpu.doj = e.kzj; cw.gpu.dtv = e.kzw;
        cw.gpu.ekz = e.lae;
        cw.timer.div = e.mkw; cw.timer.ejw = e.mla;
        cw.timer.fww = e.mlb; cw.timer.ezl = e.mkz;
        cw.on.bwu = e.kgs;
        cw.on.brv = e.kgq;
        cw.on.ctr = e.kgr;
        cw.on.ev = e.kgp;
        if e.imy.len() == cw.on.ajl.len() {
            cw.on.ajl.dg(&e.imy);
        }
    }

    
    pub fn pru(&mut self, cw: &GameBoyEmulator) {
        self.ftx.clear();
        self.ftx.pcn(cw.aec.len());
        for &o in cw.aec.iter() {
            self.ftx.push(o);
        }
    }

    
    pub fn wfn(&mut self, cw: &GameBoyEmulator) {
        self.chq.clear();
        let ap = self.fty as u8;
        for (a, &o) in cw.aec.iter().cf() {
            if o == ap {
                let ag = if a < 0x1000 { 0xC000 + a as u16 } else { 0xD000 + (a as u16 - 0x1000) };
                self.chq.push(ag);
                if self.chq.len() >= CFQ_ { break; }
            }
        }
        self.pru(cw);
        self.bcn = true;
    }

    
    pub fn wfm(&mut self, cw: &GameBoyEmulator) {
        if !self.bcn { return; }
        let utq: Vec<u16> = self.chq.iter().hu().hi(|&ag| {
            let heq = boa(cw, ag);
            let vo = self.plv(ag);
            match self.grs {
                SearchMode::Ho => heq == self.fty as u8,
                SearchMode::Aaj => heq != vo,
                SearchMode::Afd => heq == vo,
                SearchMode::Ss => heq > vo,
                SearchMode::Tg => heq < vo,
            }
        }).collect();
        self.chq = utq;
        self.pru(cw);
    }

    fn plv(&self, ag: u16) -> u8 {
        let w = match ag {
            0xC000..=0xCFFF => (ag - 0xC000) as usize,
            0xD000..=0xDFFF => 0x1000 + (ag - 0xD000) as usize,
            _ => return 0xFF,
        };
        if w < self.ftx.len() { self.ftx[w] } else { 0xFF }
    }

    
    pub fn qfq(&mut self, ag: u16) {
        if self.eku.len() >= BAG_ { return; }
        if self.eku.iter().any(|d| d.ag == ag) { return; }
        let mut cu = [0u8; 8];
        let e = format!("{:04X}", ag);
        for (a, o) in e.bf().cf().take(8) { cu[a] = o; }
        self.eku.push(Bwr {
            ag,
            cu,
            jce: e.len().v(8) as u8,
            jjy: 0,
            hes: 0,
            cpa: false,
            aw: 1,
        });
    }

    
    pub fn pxk(&mut self, cw: &GameBoyEmulator) {
        for d in self.eku.el() {
            d.jjy = d.hes;
            d.hes = boa(cw, d.ag);
            d.cpa = d.hes != d.jjy;
        }
    }

    pub fn or(&mut self) {
        self.frame = self.frame.cn(1);
    }

    pub fn vr(&mut self, bs: u8) {
        match self.ahd {
            LabTab::Zt => self.tjz(bs),
            LabTab::Ys => self.tka(bs),
            LabTab::Aoe => self.tkd(bs),
            LabTab::Oh => self.tkb(bs),
            LabTab::Ze => self.tkc(bs),
        }
        
        match bs {
            
            b'p' | b'P' => self.ant = !self.ant,
            
            b'n' | b'N' => { self.dwj = true; self.ant = true; }
            
            b'm' | b'M' => { self.dwi = true; self.ant = true; }
            
            b',' => { if self.cmx > 0 { self.cmx -= 1; } }
            b'.' => { if self.cmx < 4 { self.cmx += 1; } }
            _ => {}
        }
    }

    fn tjz(&mut self, bs: u8) {
        match bs {
            0x09 => { self.eyi = (self.eyi + 1) % 6; }
            0xF0 => { self.bnn = self.bnn.nj(0x10); }
            0xF1 => { self.bnn = self.bnn.cn(0x10); }
            0xF2 => { self.bnn = self.bnn.nj(0x100); }
            0xF3 => { self.bnn = self.bnn.cn(0x100); }
            b'1' => { self.foa = 0; self.bnn = 0xC000; }
            b'2' => { self.foa = 1; self.bnn = 0x8000; }
            b'3' => { self.foa = 2; self.bnn = 0xFF80; }
            b'4' => { self.foa = 3; self.bnn = 0x0000; }
            b'5' => { self.foa = 4; self.bnn = 0xFE00; }
            _ => {}
        }
    }

    fn tka(&mut self, bs: u8) {
        match bs {
            b'0'..=b'9' | b'a'..=b'f' | b'A'..=b'F' => {
                if (self.eif as usize) < 4 {
                    self.joa[self.eif as usize] = bs;
                    self.eif += 1;
                    
                    self.fty = self.oum();
                }
            }
            0x08 | 0x7F => { 
                if self.eif > 0 {
                    self.eif -= 1;
                    self.fty = self.oum();
                }
            }
            
            0x09 => {
                self.grs = match self.grs {
                    SearchMode::Ho => SearchMode::Aaj,
                    SearchMode::Aaj => SearchMode::Afd,
                    SearchMode::Afd => SearchMode::Ss,
                    SearchMode::Ss => SearchMode::Tg,
                    SearchMode::Tg => SearchMode::Ho,
                };
            }
            
            b'r' | b'R' => {
                self.chq.clear();
                self.bcn = false;
                self.eif = 0;
                self.ftx.clear();
            }
            _ => {}
        }
    }

    fn tkd(&mut self, bs: u8) {
        match bs {
            
            0x08 | 0x7F => {
                self.eku.pop();
            }
            _ => {}
        }
    }

    fn tkb(&mut self, bs: u8) {
        match bs {
            0x09 => { self.ejv = (self.ejv + 1) % 3; }
            0xF0 => { self.gum = self.gum.ao(1); }
            0xF1 => { self.gum = self.gum.akq(1); }
            _ => {}
        }
    }

    fn tkc(&mut self, bs: u8) {
        match bs {
            b't' | b'T' => { self.eka = !self.eka; }
            b'r' | b'R' => { self.trace.clear(); }
            _ => {}
        }
    }

    fn oum(&self) -> u16 {
        let mut ap: u16 = 0;
        for a in 0..self.eif as usize {
            let r = self.joa[a];
            let dpy = match r {
                b'0'..=b'9' => r - b'0',
                b'a'..=b'f' => r - b'a' + 10,
                b'A'..=b'F' => r - b'A' + 10,
                _ => 0,
            };
            ap = (ap << 4) | dpy as u16;
        }
        ap
    }

    pub fn ago(&mut self, kb: i32, ix: i32, hk: u32, qec: u32) {
        
        
        let bbs = 22i32;
        let puj = LI_ as i32 + bbs;
        let pui = puj + JE_ as i32;
        if ix >= puj && ix < pui {
            let axb = 72i32;
            let gx = kb - 4;
            if gx >= 0 {
                let xag = gx / axb;
                match xag {
                    0 => self.ahd = LabTab::Zt,
                    1 => self.ahd = LabTab::Ys,
                    2 => self.ahd = LabTab::Aoe,
                    3 => self.ahd = LabTab::Oh,
                    4 => self.ahd = LabTab::Ze,
                    _ => {}
                }
            }
            
            let eyu = hk as i32 - 200;
            if kb >= eyu && kb < eyu + 24 {
                if self.cmx > 0 { self.cmx -= 1; }
            } else if kb >= eyu + 28 && kb < eyu + 52 {
                if self.cmx < 4 { self.cmx += 1; }
            }
            
            let goz = hk as i32 - 130;
            if kb >= goz && kb < goz + 40 {
                self.ant = !self.ant;
            }
            
            let dcj = hk as i32 - 86;
            if kb >= dcj && kb < dcj + 36 {
                self.dwj = true; self.ant = true;
            }
            
            let nwa = hk as i32 - 46;
            if kb >= nwa && kb < nwa + 42 {
                self.dwi = true; self.ant = true;
            }
        }

        
        let gl = pui;

        
        if self.ahd == LabTab::Ys {
            
            let onx = gl + 6 + K_ as i32 + 4 + K_ as i32 + 2;
            if ix >= onx && ix < onx + K_ as i32 {
                let hl = kb - 8 - 48;
                if hl >= 0 {
                    
                    let upl: [i32; 5] = [5*8+10, 7*8+10, 4*8+10, 7*8+10, 4*8+10];
                    let mut jyy = 0i32;
                    for (a, &d) in upl.iter().cf() {
                        if hl >= jyy && hl < jyy + d {
                            self.grs = match a {
                                0 => SearchMode::Ho,
                                1 => SearchMode::Aaj,
                                2 => SearchMode::Afd,
                                3 => SearchMode::Ss,
                                _ => SearchMode::Tg,
                            };
                            break;
                        }
                        jyy += d;
                    }
                }
            }
            
            let hqd = gl + 120;
            if ix >= hqd && kb >= (hk as i32 - 60) {
                let w = ((ix - hqd) / K_ as i32) as usize;
                if w < self.chq.len() {
                    self.qfq(self.chq[w]);
                }
            }
        }

        
        if self.ahd == LabTab::Oh {
            let cgi = gl + 6 + K_ as i32;
            if ix >= cgi && ix < cgi + K_ as i32 {
                let y = kb - 8;
                if y >= 0 && y < 110 { self.ejv = 0; }
                else if y >= 110 && y < 220 { self.ejv = 1; }
                else if y >= 220 && y < 330 { self.ejv = 2; }
            }
        }
    }
}


pub fn sdd(
    g: &GameLabState,
    cw: Option<&GameBoyEmulator>,
    fx: i32, lw: i32, hk: u32, mg: u32,
) {
    let cx = fx as u32;
    let ae = (lw + LI_ as i32) as u32;
    let dt = hk;
    let bm = mg.ao(LI_);

    if dt < 200 || bm < 150 { return; }

    
    framebuffer::ah(cx, ae, dt, bm, MH_);

    
    framebuffer::ah(cx, ae, dt, 22, SE_);
    let ilt = (g.frame / 15) % 2 == 0;
    let sah = if ilt { BE_ } else { F_ };
    framebuffer::ah(cx + 6, ae + 8, 6, 6, sah);
    fu(cx + 16, ae + 4, "GAME LAB", BE_);

    
    if cw.is_some() {
        fu(cx + 100, ae + 4, "[LINKED]", BE_);
    } else {
        fu(cx + 100, ae + 4, "[NO EMU]", AW_);
    }

    
    let dbq = cx + dt - 120;
    bgt(dbq, ae + 2, 48, 16, "SAVE", g.fto.blq, O_);
    bgt(dbq + 54, ae + 2, 48, 16, "LOAD", g.fto.blq, if g.fto.blq { BE_ } else { F_ });

    
    let ty = ae + 22;
    framebuffer::ah(cx, ty, dt, JE_, SG_);
    framebuffer::ah(cx, ty + JE_ - 1, dt, 1, JR_);

    
    let bio: [(&str, LabTab); 5] = [
        ("ANALYZE", LabTab::Zt),
        ("SEARCH", LabTab::Ys),
        ("WATCH", LabTab::Aoe),
        ("TILES", LabTab::Oh),
        ("TRACE", LabTab::Ze),
    ];
    let axb: u32 = 68;
    for (a, (cu, acp)) in bio.iter().cf() {
        let gx = cx + 4 + a as u32 * (axb + 4);
        let gh = g.ahd == *acp;
        let ei = if gh { 0xFF1A3828 } else { SG_ };
        framebuffer::ah(gx, ty + 2, axb, JE_ - 4, ei);
        if gh {
            framebuffer::ah(gx, ty + JE_ - 3, axb, 2, BE_);
        }
        let bj = if gh { BE_ } else { F_ };
        fu(gx + 4, ty + 6, cu, bj);
    }

    
    let eyu = cx + dt - 200;
    bgt(eyu, ty + 3, 22, 16, "<", false, O_);
    fu(eyu + 26, ty + 6, g.wqy(), AF_);
    bgt(eyu + 56, ty + 3, 22, 16, ">", false, O_);

    
    let goz = cx + dt - 130;
    let vfm = if g.ant { AW_ } else { BE_ };
    let vfn = if g.ant { "PLAY" } else { "PAUS" };
    bgt(goz, ty + 3, 38, 16, vfn, g.ant, vfm);
    bgt(goz + 42, ty + 3, 34, 16, "STEP", false, BB_);
    bgt(goz + 80, ty + 3, 42, 16, "FRAME", false, BO_);

    
    let gl = ty + JE_;
    let nd = bm.ao(22 + JE_);

    match g.ahd {
        LabTab::Zt => sfz(g, cw, cx, gl, dt, nd),
        LabTab::Ys => sga(g, cw, cx, gl, dt, nd),
        LabTab::Aoe => sgd(g, cw, cx, gl, dt, nd),
        LabTab::Oh => sgb(g, cw, cx, gl, dt, nd),
        LabTab::Ze => sgc(g, cw, cx, gl, dt, nd),
    }
}





fn sfz(g: &GameLabState, cw: Option<&GameBoyEmulator>, cx: u32, ae: u32, dt: u32, bm: u32) {
    let iea = bm * 60 / 100;
    let keg = bm - iea - 2;
    let oy = (dt - 4) / 3;

    ses(cw, cx + 1, ae, oy, iea, g);
    set(cw, cx + oy + 2, ae, oy, iea, g);
    sew(cw, g, cx + oy * 2 + 3, ae, oy, iea);

    let je = ae + iea + 2;
    sev(cw, cx + 1, je, oy, keg, g);
    ser(cw, cx + oy + 2, je, oy, keg);
    seu(cw, cx + oy * 2 + 3, je, oy, keg, g);
}





fn sga(g: &GameLabState, cw: Option<&GameBoyEmulator>, cx: u32, ae: u32, dt: u32, bm: u32) {
    let y = cx + 8;
    let mut x = ae + 6;

    
    fu(y, x, "MEMORY SEARCH", O_);
    fu(y + 120, x, "(Hex value, Tab=mode, R=reset)", F_);
    x += K_ + 4;

    
    fu(y, x, "VALUE:", ED_);
    let mut hoc = String::new();
    for a in 0..g.eif as usize {
        hoc.push(g.joa[a] as char);
    }
    if hoc.is_empty() { hoc.t("__"); }
    
    if (g.frame / 20) % 2 == 0 { hoc.push('_'); }
    fu(y + 52, x, &hoc, AF_);

    
    let gwl = format!("= {} (0x{:02X})", g.fty, g.fty);
    fu(y + 120, x, &gwl, F_);
    x += K_ + 2;

    
    fu(y, x, "MODE:", ED_);
    let gmv = [
        ("EXACT", SearchMode::Ho),
        ("CHANGED", SearchMode::Aaj),
        ("SAME", SearchMode::Afd),
        ("GREATER", SearchMode::Ss),
        ("LESS", SearchMode::Tg),
    ];
    let mut hl = y + 48;
    for (cu, ev) in &gmv {
        let gh = g.grs == *ev;
        let bj = if gh { BE_ } else { F_ };
        if gh { framebuffer::ah(hl - 2, x - 1, cu.len() as u32 * GB_ + 4, K_, 0xFF1A3020); }
        fu(hl, x, cu, bj);
        hl += cu.len() as u32 * GB_ + 10;
    }
    x += K_ + 2;

    
    fu(y, x, "Enter=Scan/Filter", O_);
    fu(y + 152, x, "R=Reset", MJ_);
    x += K_ + 6;

    
    let status = if !g.bcn {
        String::from("No scan yet. Type value + Enter to scan WRAM.")
    } else {
        format!("Results: {} addresses", g.chq.len())
    };
    fu(y, x, &status, T_);
    x += K_ + 4;

    
    if g.bcn {
        framebuffer::ah(cx + 4, x, dt - 8, 1, JR_);
        x += 4;
        fu(y, x, "ADDR", O_);
        fu(y + 60, x, "VALUE", O_);
        fu(y + 110, x, "DEC", O_);
        if dt > 400 { fu(y + 160, x, "PREV", F_); }
        fu(cx + dt - 68, x, "[+WATCH]", BE_);
        x += K_ + 2;

        let brh = ((bm.ao(x - ae)) / K_).v(32) as usize;
        for (a, &ag) in g.chq.iter().take(brh).cf() {
            let ap = if let Some(aa) = cw { boa(aa, ag) } else { 0 };
            let vo = g.plv(ag);
            let cpa = ap != vo;

            let dyd = format!("{:04X}", ag);
            fu(y, x, &dyd, HR_);

            let mov = format!("{:02X}", ap);
            let xqj = if cpa { SC_ } else { AF_ };
            fu(y + 60, x, &mov, xqj);

            let kok = format!("{:3}", ap);
            fu(y + 110, x, &kok, F_);

            if dt > 400 {
                let lvj = format!("{:02X}", vo);
                fu(y + 160, x, &lvj, F_);
            }

            
            let mqj = cx + dt - 48;
            fu(mqj, x, "+W", BE_);

            x += K_;
            let _ = a;
        }
        if g.chq.len() > brh {
            let upp = format!("... +{} more", g.chq.len() - brh);
            fu(y, x, &upp, F_);
        }
    }
}





fn sgd(g: &GameLabState, cw: Option<&GameBoyEmulator>, cx: u32, ae: u32, dt: u32, bm: u32) {
    let y = cx + 8;
    let mut x = ae + 6;

    fu(y, x, "WATCH LIST", O_);
    let rpg = format!("{}/{}", g.eku.len(), BAG_);
    fu(y + 100, x, &rpg, F_);
    fu(y + 160, x, "(Backspace=remove last)", F_);
    x += K_ + 4;

    if g.eku.is_empty() {
        fu(y, x, "No watches. Add from Search tab with [+W] button.", F_);
        return;
    }

    
    framebuffer::ah(cx + 4, x, dt - 8, 1, JR_);
    x += 4;
    fu(y, x, "LABEL", O_);
    fu(y + 72, x, "ADDR", O_);
    fu(y + 120, x, "HEX", O_);
    fu(y + 160, x, "DEC", O_);
    fu(y + 200, x, "PREV", O_);
    if dt > 500 { fu(y + 250, x, "VISUAL", F_); }
    x += K_ + 2;

    for d in g.eku.iter() {
        let fms: String = d.cu[..d.jce as usize].iter().map(|&o| o as char).collect();
        fu(y, x, &fms, ED_);

        let dyd = format!("{:04X}", d.ag);
        fu(y + 72, x, &dyd, HR_);

        let ap = if let Some(aa) = cw { boa(aa, d.ag) } else { d.hes };
        let mov = format!("{:02X}", ap);
        let bj = if d.cpa { SC_ } else { AF_ };
        fu(y + 120, x, &mov, bj);

        let kok = format!("{:3}", ap);
        fu(y + 160, x, &kok, bj);

        let lvj = format!("{:02X}", d.jjy);
        fu(y + 200, x, &lvj, F_);

        
        if dt > 500 {
            let lo = 100u32;
            let vi = (ap as u32 * lo) / 255;
            framebuffer::ah(y + 250, x + 2, lo, 8, 0xFF0A1A10);
            let kca = if d.cpa { SC_ } else { BE_ };
            framebuffer::ah(y + 250, x + 2, vi, 8, kca);
        }

        x += K_;
    }
}





fn sgb(g: &GameLabState, cw: Option<&GameBoyEmulator>, cx: u32, ae: u32, dt: u32, bm: u32) {
    let y = cx + 8;
    let mut x = ae + 6;

    let bcd = ["TILES $8000", "TILES $8800", "OAM SPRITES"];
    fu(y, x, "TILE VIEWER", O_);
    x += K_;

    
    for (a, cu) in bcd.iter().cf() {
        let gh = a as u8 == g.ejv;
        let bj = if gh { BE_ } else { F_ };
        let gx = y + a as u32 * 110;
        if gh { framebuffer::ah(gx - 2, x - 1, cu.len() as u32 * GB_ + 4, K_, 0xFF1A3020); }
        fu(gx, x, cu, bj);
    }
    fu(y + 340, x, "(Tab=page, Arrows=scroll)", F_);
    x += K_ + 6;

    let cw = match cw {
        Some(aa) => aa,
        None => { fu(y, x, "No emulator linked", F_); return; }
    };

    if g.ejv < 2 {
        
        let sm: u16 = if g.ejv == 0 { 0x8000 } else { 0x8800 };
        let bll = 2u32; 
        let idm = 8 * bll;
        let ec = ((dt - 20) / (idm + 1)).v(16);
        let brh = ((bm - (x - ae) - 4) / (idm + 1)).v(16);
        let jc = g.gum.v(16u32.ao(brh));

        for br in 0..brh {
            for bj in 0..ec {
                let ptb = (jc + br) * 16 + bj;
                if ptb >= 256 { break; }
                let bsn = sm.cn(ptb as u16 * 16);
                let dx = y + bj * (idm + 1);
                let bg = x + br * (idm + 1);

                
                for mnp in 0..8u32 {
                    let hh = boa(cw, bsn.cn(mnp as u16 * 2));
                    let gd = boa(cw, bsn.cn(mnp as u16 * 2 + 1));
                    for pwq in 0..8u32 {
                        let ga = 7 - pwq;
                        let bts = ((gd >> ga) & 1) << 1 | ((hh >> ga) & 1);
                        let dlr = match bts {
                            0 => 0xFF0A1510,
                            1 => 0xFF346856,
                            2 => 0xFF88C070,
                            3 => 0xFFE0F8D0,
                            _ => 0xFF000000,
                        };
                        framebuffer::ah(
                            dx + pwq * bll,
                            bg + mnp * bll,
                            bll, bll, dlr,
                        );
                    }
                }
            }
        }

        
        let fmn = x + brh * (idm + 1) + 4;
        let vqd = format!("Tiles {}-{}", jc * 16, ((jc + brh) * 16).v(256) - 1);
        fu(y, fmn, &vqd, F_);
    } else {
        
        fu(y, x, "#  Y   X   TILE FLAGS", O_);
        x += K_;

        let uly = ((bm - (x - ae)) / K_).v(40);
        for a in 0..uly {
            let dkd = 0xFE00u16 + a as u16 * 4;
            let cq = boa(cw, dkd);
            let cr = boa(cw, dkd + 1);
            let ccd = boa(cw, dkd + 2);
            let flags = boa(cw, dkd + 3);

            let iw = cq > 0 && cq < 160 && cr > 0 && cr < 168;
            let bj = if iw { AF_ } else { F_ };

            let e = format!("{:2} {:3} {:3}  {:02X}   {:02X}", a, cq, cr, ccd, flags);
            fu(y, x, &e, bj);

            
            let iux = y + 200;
            if flags & 0x80 != 0 { fu(iux, x, "P", MJ_); }
            if flags & 0x40 != 0 { fu(iux + 12, x, "Y", O_); }
            if flags & 0x20 != 0 { fu(iux + 24, x, "X", O_); }
            if cw.atz {
                let jil = flags & 0x07;
                let om = (flags >> 3) & 1;
                let jkk = format!("P{} B{}", jil, om);
                fu(iux + 40, x, &jkk, BB_);
            }

            x += K_;
        }
    }
}





fn sgc(g: &GameLabState, cw: Option<&GameBoyEmulator>, cx: u32, ae: u32, dt: u32, bm: u32) {
    let y = cx + 8;
    let mut x = ae + 6;

    fu(y, x, "TRACE LOG", O_);
    let sli = if g.eka { "[ON]" } else { "[OFF]" };
    let slh = if g.eka { BE_ } else { AW_ };
    fu(y + 88, x, sli, slh);
    fu(y + 132, x, "(T=toggle, R=clear)", F_);
    x += K_ + 2;

    
    if let Some(cw) = cw {
        fu(y, x, "DISASSEMBLY @ PC", O_);
        x += K_;
        let fz = cw.cpu.fz;
        
        let mut ag = fz.nj(8);
        let ryb = 12u32.v((bm / 3) / K_);
        for _ in 0..ryb {
            let opcode = boa(cw, ag);
            let afb = ag == fz;
            let adx = if afb { ">" } else { " " };
            let (bes, aw) = rya(cw, ag);
            let e = format!("{} {:04X}: {:02X}  {}", adx, ag, opcode, bes);
            let bj = if afb { BE_ } else { T_ };
            if afb {
                framebuffer::ah(y - 2, x - 1, dt - 16, K_, 0xFF1A3020);
            }
            fu(y, x, &e, bj);

            
            if g.gbl.iter().any(|&bp| bp == ag) {
                framebuffer::ah(y - 6, x + 2, 4, 8, AW_);
            }

            x += K_;
            ag = ag.cn(aw as u16);
        }
    }
    x += 6;
    framebuffer::ah(cx + 4, x, dt - 8, 1, JR_);
    x += 4;

    
    fu(y, x, "BREAKPOINTS", O_);
    let qrq = format!("{}/{}", g.gbl.len(), CFE_);
    fu(y + 100, x, &qrq, F_);
    x += K_;

    if g.gbl.is_empty() {
        fu(y, x, "None (type addr in Search, click to add)", F_);
    } else {
        let mut hbd = y;
        for &bp in g.gbl.iter() {
            let hbc = format!("{:04X}", bp);
            framebuffer::ah(hbd, x, 40, K_ - 2, 0xFF2A0A0A);
            fu(hbd + 2, x, &hbc, AW_);
            hbd += 48;
            if hbd > cx + dt - 60 { x += K_; hbd = y; }
        }
    }
    x += K_ + 4;
    framebuffer::ah(cx + 4, x, dt - 8, 1, JR_);
    x += 4;

    
    fu(y, x, "TRACE HISTORY", O_);
    let xky = format!("({} entries)", g.trace.len());
    fu(y + 120, x, &xky, F_);
    x += K_;

    fu(y, x, "PC    OP  A  F  SP", F_);
    x += K_;

    let brh = ((bm.ao(x - ae)) / K_) as usize;
    let ay = if g.trace.len() > brh { g.trace.len() - brh } else { 0 };
    for bt in g.trace[ay..].iter() {
        let e = format!("{:04X}  {:02X}  {:02X} {:02X} {:04X}", bt.fz, bt.opcode, bt.q, bt.bb, bt.sp);
        fu(y, x, &e, T_);
        x += K_;
        if x + K_ > ae + bm { break; }
    }
}


fn ses(cw: Option<&GameBoyEmulator>, b: u32, c: u32, d: u32, i: u32, g: &GameLabState) {
    epf(b, c, d, i, "CPU REGISTERS", g.eyi == 0);

    let y = b + KX_ + 2;
    let mut x = c + 20;

    if let Some(cw) = cw {
        let cpu = &cw.cpu;

        
        let regs = [
            ("AF", cpu.q, cpu.bb),
            ("BC", cpu.o, cpu.r),
            ("DE", cpu.bc, cpu.aa),
            ("HL", cpu.i, cpu.dm),
        ];
        for (j, gd, hh) in &regs {
            fu(y, x, j, ED_);
            let e = format!("{:02X}{:02X}", gd, hh);
            fu(y + 28, x, &e, AF_);
            
            let cuc = format!("({:3} {:3})", gd, hh);
            fu(y + 72, x, &cuc, F_);
            x += K_;
        }

        x += 4;
        
        fu(y, x, "SP", ED_);
        let wrm = format!("{:04X}", cpu.sp);
        fu(y + 28, x, &wrm, AF_);
        x += K_;

        fu(y, x, "PC", ED_);
        let vfw = format!("{:04X}", cpu.fz);
        fu(y + 28, x, &vfw, O_);

        
        if cw.dvf {
            let opcode = boa(cw, cpu.fz);
            let ops = format!("[{:02X}]", opcode);
            fu(y + 72, x, &ops, BB_);
        }
        x += K_ + 6;

        
        fu(y, x, "FLAGS", F_);
        x += K_;
        let flags = [
            ("Z", cpu.bb & 0x80 != 0),
            ("N", cpu.bb & 0x40 != 0),
            ("H", cpu.bb & 0x20 != 0),
            ("C", cpu.bb & 0x10 != 0),
        ];
        let mut jf = y;
        for (j, oj) in &flags {
            let s = if *oj { DP_ } else { SD_ };
            framebuffer::ah(jf, x, 24, 14, if *oj { 0xFF0A3020 } else { 0xFF0A1510 });
            fu(jf + 4, x + 1, j, s);
            jf += 28;
        }
        x += K_ + 6;

        
        fu(y, x, "IME", F_);
        fu(y + 32, x, if cpu.dih { "ON" } else { "OFF" }, 
            if cpu.dih { DP_ } else { SD_ });
        fu(y + 64, x, "HALT", F_);
        fu(y + 100, x, if cpu.dhv { "YES" } else { "NO" },
            if cpu.dhv { MJ_ } else { F_ });
        x += K_;

        
        fu(y, x, "CYCLES", F_);
        let aap = format!("{}", cpu.yl);
        fu(y + 56, x, &aap, AF_);
        x += K_;

        
        if cw.atz {
            fu(y, x, "MODE", F_);
            fu(y + 40, x, "CGB", BE_);
            let mgn = if cw.beq & 0x80 != 0 { "2x" } else { "1x" };
            fu(y + 72, x, mgn, O_);
        }
    } else {
        fu(y, x, "No emulator linked", F_);
    }
}


fn set(cw: Option<&GameBoyEmulator>, b: u32, c: u32, d: u32, i: u32, g: &GameLabState) {
    epf(b, c, d, i, "GPU / LCD", g.eyi == 1);

    let y = b + KX_ + 2;
    let mut x = c + 20;

    if let Some(cw) = cw {
        let gpu = &cw.gpu;

        
        fu(y, x, "LCDC", ED_);
        let awb = format!("{:02X}", gpu.amh);
        fu(y + 40, x, &awb, AF_);
        let oim = gpu.amh & 0x80 != 0;
        fu(y + 64, x, if oim { "LCD:ON" } else { "LCD:OFF" },
            if oim { DP_ } else { AW_ });
        x += K_;

        
        let fs = [
            ("BG", gpu.amh & 0x01 != 0),
            ("OBJ", gpu.amh & 0x02 != 0),
            ("8x16", gpu.amh & 0x04 != 0),
            ("WIN", gpu.amh & 0x20 != 0),
        ];
        let mut bx = y;
        for (j, ea) in &fs {
            let bj = if *ea { DP_ } else { SD_ };
            fu(bx, x, j, bj);
            bx += (j.len() as u32 + 1) * GB_;
        }
        x += K_ + 4;

        
        fu(y, x, "LY", ED_);
        let ujc = format!("{:3} / 153", gpu.ct);
        fu(y + 28, x, &ujc, AF_);
        
        let ajx = y + 110;
        let lo = d.ao(130);
        framebuffer::ah(ajx, x + 2, lo, 8, 0xFF0A1A10);
        let li = (gpu.ct as u32 * lo) / 154;
        let kca = if gpu.ct < 144 { BE_ } else { MJ_ };
        framebuffer::ah(ajx, x + 2, li.v(lo), 8, kca);
        x += K_;

        fu(y, x, "LYC", F_);
        let ujb = format!("{:3}", gpu.eey);
        fu(y + 32, x, &ujb, AF_);
        if gpu.ct == gpu.eey {
            fu(y + 60, x, "=MATCH", BE_);
        }
        x += K_;

        
        fu(y, x, "MODE", F_);
        let (czz, upa) = match gpu.ev {
            0 => ("HBLANK", F_),
            1 => ("VBLANK", MJ_),
            2 => ("OAM", O_),
            3 => ("DRAW", BE_),
            _ => ("???", AW_),
        };
        fu(y + 40, x, czz, upa);
        let rsv = format!("({} dots)", gpu.yl);
        fu(y + 96, x, &rsv, F_);
        x += K_ + 4;

        
        fu(y, x, "SCX/Y", F_);
        let rv = format!("{:3},{:3}", gpu.eye, gpu.eyf);
        fu(y + 48, x, &rv, AF_);
        x += K_;
        fu(y, x, "WX/Y", F_);
        let ciw = format!("{:3},{:3}", gpu.fx, gpu.lw);
        fu(y + 48, x, &ciw, AF_);
        x += K_ + 4;

        
        fu(y, x, "BGP", F_);
        krg(y + 32, x, gpu.emt);
        x += K_;
        fu(y, x, "OBP0", F_);
        krg(y + 40, x, gpu.fpm);
        x += K_;
        fu(y, x, "OBP1", F_);
        krg(y + 40, x, gpu.fpn);
        x += K_ + 4;

        
        if cw.atz {
            fu(y, x, "CGB BG PALETTES", F_);
            x += K_;
            for jil in 0..8 {
                let vkm = y + jil * 36;
                for r in 0..4u32 {
                    let l = (jil * 8 + r * 2) as usize;
                    if l + 1 < gpu.bdo.len() {
                        let hh = gpu.bdo[l] as u16;
                        let gd = gpu.bdo[l + 1] as u16;
                        let fsv = hh | (gd << 8);
                        let m = (((fsv & 0x1F) as u8) << 3) as u32;
                        let at = ((((fsv >> 5) & 0x1F) as u8) << 3) as u32;
                        let o = ((((fsv >> 10) & 0x1F) as u8) << 3) as u32;
                        let s = 0xFF000000 | (m << 16) | (at << 8) | o;
                        framebuffer::ah(vkm + r * 8, x, 7, 8, s);
                    }
                }
            }
            x += 12;

            
            fu(y, x, "VRAM BANK", F_);
            let xqu = format!("{}", gpu.fbb);
            fu(y + 80, x, &xqu, AF_);
        }
    } else {
        fu(y, x, "No emulator linked", F_);
    }
}


fn sew(cw: Option<&GameBoyEmulator>, g: &GameLabState, b: u32, c: u32, d: u32, i: u32) {
    epf(b, c, d, i, "MEMORY", g.eyi == 2);

    let y = b + KX_ + 2;
    let mut x = c + 20;

    
    let gmv = ["WRAM", "VRAM", "HRAM", "ROM", "OAM"];
    let mut gx = y;
    for (a, j) in gmv.iter().cf() {
        let na = a as u8 == g.foa;
        let bj = if na { BE_ } else { F_ };
        if na {
            framebuffer::ah(gx, x, j.len() as u32 * GB_ + 4, K_, 0xFF1A3020);
        }
        fu(gx + 2, x, j, bj);
        gx += j.len() as u32 * GB_ + 8;
    }
    x += K_ + 4;

    
    let qfu = format!("ADDR  {:04X}", g.bnn);
    fu(y, x, &qfu, O_);
    x += K_;

    if let Some(cw) = cw {
        
        let lk = ((i - 60) / K_).v(16) as u16;
        for br in 0..lk {
            let ag = g.bnn.cn(br * 16);
            
            let dyd = format!("{:04X}", ag);
            fu(y, x, &dyd, HR_);

            
            let mut bng = y + 40;
            let mut mwk = [b'.'; 16];
            for bj in 0..16u16 {
                let q = ag.cn(bj);
                let hf = boa(cw, q);
                let lcu = format!("{:02X}", hf);
                
                let nli = (br * 16 + bj) as usize;
                let vo = if nli < g.jfs.len() { g.jfs[nli] } else { hf };
                let quv = if hf != vo { SC_ } else if hf == 0 { F_ } else { AF_ };
                fu(bng, x, &lcu, quv);
                bng += 20;
                if bj == 7 { bng += 4; } 

                if hf >= 0x20 && hf < 0x7F {
                    mwk[bj as usize] = hf;
                }
            }

            
            if d > 420 {
                let ascii: alloc::string::String = mwk.iter().map(|&o| o as char).collect();
                fu(bng + 8, x, &ascii, F_);
            }

            x += K_;
        }
    } else {
        fu(y, x, "No emulator linked", F_);
    }
}


fn sev(cw: Option<&GameBoyEmulator>, b: u32, c: u32, d: u32, i: u32, g: &GameLabState) {
    epf(b, c, d, i, "I/O REGISTERS", g.eyi == 3);

    let y = b + KX_ + 2;
    let mut x = c + 20;

    if let Some(cw) = cw {
        
        fu(y, x, "INTERRUPTS", O_);
        x += K_;

        fu(y, x, "IE", ED_);
        let trv = format!("{:02X}", cw.brc);
        fu(y + 24, x, &trv, AF_);

        fu(y + 50, x, "IF", ED_);
        let trw = format!("{:02X}", cw.bhf);
        fu(y + 74, x, &trw, AF_);
        x += K_;

        
        let tvk = ["VBL", "STA", "TIM", "SER", "JOY"];
        let mut fg = y;
        for (a, j) in tvk.iter().cf() {
            let hnr = cw.brc & (1 << a) != 0;
            let dry = cw.bhf & (1 << a) != 0;
            let bj = if hnr && dry { AW_ } else if hnr { DP_ } else { SD_ };
            fu(fg, x, j, bj);
            fg += 32;
        }
        x += K_ + 6;

        
        fu(y, x, "TIMER", O_);
        x += K_;

        fu(y, x, "DIV", F_);
        let rzh = format!("{:02X}", cw.timer.pac());
        fu(y + 32, x, &rzh, AF_);

        fu(y + 60, x, "TIMA", F_);
        let xha = format!("{:02X}", cw.timer.ejw);
        fu(y + 100, x, &xha, AF_);
        x += K_;

        fu(y, x, "TMA", F_);
        let xie = format!("{:02X}", cw.timer.fww);
        fu(y + 32, x, &xie, AF_);

        fu(y + 60, x, "TAC", F_);
        let xam = format!("{:02X}", cw.timer.ezl);
        fu(y + 100, x, &xam, AF_);
        x += K_ + 6;

        
        fu(y, x, "SERIAL", O_);
        x += K_;
        fu(y, x, "SB", F_);
        let wdf = format!("{:02X}", cw.cht);
        fu(y + 24, x, &wdf, AF_);
        fu(y + 50, x, "SC", F_);
        let wew = format!("{:02X}", cw.chs);
        fu(y + 74, x, &wew, AF_);

        if cw.atz {
            x += K_ + 6;
            fu(y, x, "CGB I/O", O_);
            x += K_;
            fu(y, x, "KEY1", F_);
            let uay = format!("{:02X}", cw.beq);
            fu(y + 40, x, &uay, AF_);
            fu(y + 68, x, "WRAM", F_);
            let xud = format!("BK{}", cw.axj);
            fu(y + 104, x, &xud, AF_);
        }
    } else {
        fu(y, x, "No emulator linked", F_);
    }
}


fn ser(cw: Option<&GameBoyEmulator>, b: u32, c: u32, d: u32, i: u32) {
    epf(b, c, d, i, "CARTRIDGE", false);

    let y = b + KX_ + 2;
    let mut x = c + 20;

    if let Some(cw) = cw {
        let on = &cw.on;

        
        let xhr: alloc::vec::Vec<u8> = on.dq.iter().hu()
            .fwc(|&r| r != 0 && r >= 0x20).collect();
        let dq = core::str::jg(&xhr).unwrap_or("???");
        fu(y, x, "TITLE", F_);
        fu(y + 48, x, dq, BE_);
        x += K_;

        
        let umg = match on.fnz {
            crate::gameboy::cartridge::MbcType::None => "ROM ONLY",
            crate::gameboy::cartridge::MbcType::Acw => "MBC1",
            crate::gameboy::cartridge::MbcType::Acx => "MBC3",
            crate::gameboy::cartridge::MbcType::Acy => "MBC5",
        };
        fu(y, x, "MBC", F_);
        fu(y + 32, x, umg, AF_);
        x += K_;

        
        let vzx = on.awv.len() / 1024;
        let vpx = on.ajl.len() / 1024;
        fu(y, x, "ROM", F_);
        let acl = format!("{}KB", vzx);
        fu(y + 32, x, &acl, AF_);
        fu(y + 80, x, "RAM", F_);
        let vqf = format!("{}KB", vpx);
        fu(y + 112, x, &vqf, AF_);
        x += K_;

        
        fu(y, x, "ROM BANK", F_);
        let vqr = format!("{:3}", on.bwu);
        fu(y + 72, x, &vqr, AF_);
        let xjt = on.awv.len() / 16384;
        let xbj = format!("/ {}", xjt);
        fu(y + 96, x, &xbj, F_);
        x += K_;

        fu(y, x, "RAM BANK", F_);
        let vzo = format!("{:3}", on.brv);
        fu(y + 72, x, &vzo, AF_);
        x += K_;

        
        fu(y, x, "CGB", F_);
        let qxz = match on.eni {
            0xC0 => "CGB ONLY",
            0x80 => "CGB+DMG",
            _ => "DMG",
        };
        let qxy = if on.eni >= 0x80 { O_ } else { F_ };
        fu(y + 32, x, qxz, qxy);
    } else {
        fu(y, x, "No cartridge", F_);
    }
}


fn seu(cw: Option<&GameBoyEmulator>, b: u32, c: u32, d: u32, i: u32, g: &GameLabState) {
    epf(b, c, d, i, "INPUT STATE", g.eyi == 5);

    let y = b + KX_ + 2;
    let mut x = c + 20;

    if let Some(cw) = cw {
        fu(y, x, "JOYPAD $FF00", O_);
        let uap = format!("{:02X}", cw.cfu);
        fu(y + 104, x, &uap, AF_);
        x += K_ + 4;

        
        fu(y, x, "D-PAD", F_);
        x += K_;
        let bln    = cw.aox & 0x04 == 0;
        let hgr  = cw.aox & 0x08 == 0;
        let fd  = cw.aox & 0x02 == 0;
        let hw = cw.aox & 0x01 == 0;

        
        let dx = y + 16;
        let bg = x;
        let nf: u32 = 16;
        
        framebuffer::ah(dx + nf, bg, nf, nf, if bln { DP_ } else { 0xFF1A2820 });
        fu(dx + nf + 3, bg + 2, "U", if bln { 0xFF000000 } else { F_ });
        
        framebuffer::ah(dx + nf, bg + nf * 2, nf, nf, if hgr { DP_ } else { 0xFF1A2820 });
        fu(dx + nf + 3, bg + nf * 2 + 2, "D", if hgr { 0xFF000000 } else { F_ });
        
        framebuffer::ah(dx, bg + nf, nf, nf, if fd { DP_ } else { 0xFF1A2820 });
        fu(dx + 3, bg + nf + 2, "L", if fd { 0xFF000000 } else { F_ });
        
        framebuffer::ah(dx + nf * 2, bg + nf, nf, nf, if hw { DP_ } else { 0xFF1A2820 });
        fu(dx + nf * 2 + 3, bg + nf + 2, "R", if hw { 0xFF000000 } else { F_ });
        
        framebuffer::ah(dx + nf, bg + nf, nf, nf, 0xFF1A2820);

        
        let bx = y + 100;
        let iil = cw.aow & 0x01 == 0;
        let ikh = cw.aow & 0x02 == 0;
        let joe = cw.aow & 0x04 == 0;
        let jrn = cw.aow & 0x08 == 0;

        
        framebuffer::ah(bx + 32, bg + 4, 22, 22, if iil { DP_ } else { 0xFF1A2820 });
        fu(bx + 38, bg + 8, "A", if iil { 0xFF000000 } else { AF_ });

        
        framebuffer::ah(bx, bg + 16, 22, 22, if ikh { DP_ } else { 0xFF1A2820 });
        fu(bx + 6, bg + 20, "B", if ikh { 0xFF000000 } else { AF_ });

        x = bg + nf * 3 + 6;

        
        let jof = y + 20;
        framebuffer::ah(jof, x, 40, 14, if joe { O_ } else { 0xFF1A2820 });
        fu(jof + 4, x + 1, "SEL", if joe { 0xFF000000 } else { F_ });

        framebuffer::ah(jof + 48, x, 48, 14, if jrn { O_ } else { 0xFF1A2820 });
        fu(jof + 52, x + 1, "START", if jrn { 0xFF000000 } else { F_ });
        x += K_ + 8;

        
        fu(y, x, "DIRS", F_);
        let bjw = format!("{:02X}", cw.aox);
        fu(y + 40, x, &bjw, AF_);
        fu(y + 68, x, "BTNS", F_);
        let kex = format!("{:02X}", cw.aow);
        fu(y + 108, x, &kex, AF_);
    } else {
        fu(y, x, "No emulator linked", F_);
    }
}



fn epf(b: u32, c: u32, d: u32, i: u32, dq: &str, na: bool) {
    
    framebuffer::ah(b, c, d, i, BNY_);
    
    let imb = if na { BE_ } else { JR_ };
    framebuffer::ah(b, c, d, 1, imb);
    framebuffer::ah(b, c + i - 1, d, 1, imb);
    framebuffer::ah(b, c, 1, i, imb);
    framebuffer::ah(b + d - 1, c, 1, i, imb);
    
    framebuffer::ah(b + 1, c + 1, d - 2, 16, SE_);
    fu(b + 6, c + 3, dq, if na { BE_ } else { O_ });
}

fn fu(b: u32, c: u32, text: &str, s: u32) {
    let mut cx = b;
    for bm in text.bw() {
        framebuffer::afn(cx, c, bm, s);
        cx += GB_;
    }
}

fn krg(b: u32, c: u32, aim: u8) {
    
    const BWL_: [u32; 4] = [0xFFE0F8D0, 0xFF88C070, 0xFF346856, 0xFF081820];
    for a in 0..4u32 {
        let dlr = (aim >> (a * 2)) & 3;
        framebuffer::ah(b + a * 16, c + 1, 14, 10, BWL_[dlr as usize]);
    }
}


pub fn boa(cw: &GameBoyEmulator, ag: u16) -> u8 {
    match ag {
        0x0000..=0x7FFF => cw.on.read(ag),
        0x8000..=0x9FFF => cw.gpu.jlp(ag),
        0xA000..=0xBFFF => cw.on.read(ag),
        0xC000..=0xCFFF => {
            let w = (ag as usize) - 0xC000;
            if w < cw.aec.len() { cw.aec[w] } else { 0xFF }
        }
        0xD000..=0xDFFF => {
            let om = cw.axj.am(1) as usize;
            let l = om * 0x1000 + (ag as usize - 0xD000);
            if l < cw.aec.len() { cw.aec[l] } else { 0xFF }
        }
        0xFE00..=0xFE9F => cw.gpu.pai(ag),
        0xFF80..=0xFFFE => {
            let w = (ag - 0xFF80) as usize;
            if w < cw.bux.len() { cw.bux[w] } else { 0xFF }
        }
        0xFFFF => cw.brc,
        _ => 0xFF,
    }
}


pub fn pxh(g: &mut GameLabState, cw: &GameBoyEmulator) {
    for a in 0..256usize {
        let ag = g.bnn.cn(a as u16);
        g.jfs[a] = boa(cw, ag);
    }
}


fn bgt(b: u32, c: u32, d: u32, i: u32, cu: &str, gh: bool, s: u32) {
    let ei = if gh { 0xFF1A3020 } else { 0xFF0E1820 };
    framebuffer::ah(b, c, d, i, ei);
    framebuffer::ah(b, c, d, 1, s & 0x40FFFFFF);
    framebuffer::ah(b, c + i - 1, d, 1, s & 0x40FFFFFF);
    framebuffer::ah(b, c, 1, i, s & 0x40FFFFFF);
    framebuffer::ah(b + d - 1, c, 1, i, s & 0x40FFFFFF);
    let bda = cu.len() as u32 * GB_;
    fu(b + (d.ao(bda)) / 2, c + (i.ao(12)) / 2, cu, s);
}


fn rya(cw: &GameBoyEmulator, ag: u16) -> (String, u8) {
    let op = boa(cw, ag);
    let of = boa(cw, ag.cn(1));
    let tb = boa(cw, ag.cn(2));
    let buy = (tb as u16) << 8 | of as u16;

    match op {
        0x00 => (String::from("NOP"), 1),
        0x01 => (format!("LD BC,${:04X}", buy), 3),
        0x02 => (String::from("LD (BC),A"), 1),
        0x03 => (String::from("INC BC"), 1),
        0x04 => (String::from("INC B"), 1),
        0x05 => (String::from("DEC B"), 1),
        0x06 => (format!("LD B,${:02X}", of), 2),
        0x07 => (String::from("RLCA"), 1),
        0x08 => (format!("LD (${:04X}),SP", buy), 3),
        0x09 => (String::from("ADD HL,BC"), 1),
        0x0A => (String::from("LD A,(BC)"), 1),
        0x0B => (String::from("DEC BC"), 1),
        0x0C => (String::from("INC C"), 1),
        0x0D => (String::from("DEC C"), 1),
        0x0E => (format!("LD C,${:02X}", of), 2),
        0x0F => (String::from("RRCA"), 1),
        0x10 => (String::from("STOP"), 2),
        0x11 => (format!("LD DE,${:04X}", buy), 3),
        0x12 => (String::from("LD (DE),A"), 1),
        0x13 => (String::from("INC DE"), 1),
        0x16 => (format!("LD D,${:02X}", of), 2),
        0x18 => (format!("JR ${:02X}", of), 2),
        0x1A => (String::from("LD A,(DE)"), 1),
        0x1E => (format!("LD E,${:02X}", of), 2),
        0x20 => (format!("JR NZ,${:02X}", of), 2),
        0x21 => (format!("LD HL,${:04X}", buy), 3),
        0x22 => (String::from("LD (HL+),A"), 1),
        0x23 => (String::from("INC HL"), 1),
        0x26 => (format!("LD H,${:02X}", of), 2),
        0x28 => (format!("JR Z,${:02X}", of), 2),
        0x2A => (String::from("LD A,(HL+)"), 1),
        0x2E => (format!("LD L,${:02X}", of), 2),
        0x2F => (String::from("CPL"), 1),
        0x30 => (format!("JR NC,${:02X}", of), 2),
        0x31 => (format!("LD SP,${:04X}", buy), 3),
        0x32 => (String::from("LD (HL-),A"), 1),
        0x33 => (String::from("INC SP"), 1),
        0x36 => (format!("LD (HL),${:02X}", of), 2),
        0x38 => (format!("JR C,${:02X}", of), 2),
        0x3C => (String::from("INC A"), 1),
        0x3D => (String::from("DEC A"), 1),
        0x3E => (format!("LD A,${:02X}", of), 2),
        0x40..=0x7F if op != 0x76 => {
            let cs = ["B","C","D","E","H","L","(HL)","A"][(op as usize >> 3) & 7];
            let cy = ["B","C","D","E","H","L","(HL)","A"][op as usize & 7];
            (format!("LD {},{}", cs, cy), 1)
        }
        0x76 => (String::from("HALT"), 1),
        0x80..=0x87 => {
            let m = ["B","C","D","E","H","L","(HL)","A"][op as usize & 7];
            (format!("ADD A,{}", m), 1)
        }
        0x90..=0x97 => {
            let m = ["B","C","D","E","H","L","(HL)","A"][op as usize & 7];
            (format!("SUB {}", m), 1)
        }
        0xA0..=0xA7 => {
            let m = ["B","C","D","E","H","L","(HL)","A"][op as usize & 7];
            (format!("AND {}", m), 1)
        }
        0xA8..=0xAF => {
            let m = ["B","C","D","E","H","L","(HL)","A"][op as usize & 7];
            (format!("XOR {}", m), 1)
        }
        0xB0..=0xB7 => {
            let m = ["B","C","D","E","H","L","(HL)","A"][op as usize & 7];
            (format!("OR {}", m), 1)
        }
        0xB8..=0xBF => {
            let m = ["B","C","D","E","H","L","(HL)","A"][op as usize & 7];
            (format!("CP {}", m), 1)
        }
        0xC0 => (String::from("RET NZ"), 1),
        0xC1 => (String::from("POP BC"), 1),
        0xC2 => (format!("JP NZ,${:04X}", buy), 3),
        0xC3 => (format!("JP ${:04X}", buy), 3),
        0xC4 => (format!("CALL NZ,${:04X}", buy), 3),
        0xC5 => (String::from("PUSH BC"), 1),
        0xC6 => (format!("ADD A,${:02X}", of), 2),
        0xC8 => (String::from("RET Z"), 1),
        0xC9 => (String::from("RET"), 1),
        0xCA => (format!("JP Z,${:04X}", buy), 3),
        0xCB => (format!("CB {:02X}", of), 2),
        0xCC => (format!("CALL Z,${:04X}", buy), 3),
        0xCD => (format!("CALL ${:04X}", buy), 3),
        0xCE => (format!("ADC A,${:02X}", of), 2),
        0xD0 => (String::from("RET NC"), 1),
        0xD1 => (String::from("POP DE"), 1),
        0xD2 => (format!("JP NC,${:04X}", buy), 3),
        0xD5 => (String::from("PUSH DE"), 1),
        0xD6 => (format!("SUB ${:02X}", of), 2),
        0xD8 => (String::from("RET C"), 1),
        0xD9 => (String::from("RETI"), 1),
        0xDA => (format!("JP C,${:04X}", buy), 3),
        0xE0 => (format!("LDH ($FF{:02X}),A", of), 2),
        0xE1 => (String::from("POP HL"), 1),
        0xE2 => (String::from("LD ($FF00+C),A"), 1),
        0xE5 => (String::from("PUSH HL"), 1),
        0xE6 => (format!("AND ${:02X}", of), 2),
        0xE9 => (String::from("JP (HL)"), 1),
        0xEA => (format!("LD (${:04X}),A", buy), 3),
        0xEE => (format!("XOR ${:02X}", of), 2),
        0xF0 => (format!("LDH A,($FF{:02X})", of), 2),
        0xF1 => (String::from("POP AF"), 1),
        0xF3 => (String::from("DI"), 1),
        0xF5 => (String::from("PUSH AF"), 1),
        0xF6 => (format!("OR ${:02X}", of), 2),
        0xFA => (format!("LD A,(${:04X})", buy), 3),
        0xFB => (String::from("EI"), 1),
        0xFE => (format!("CP ${:02X}", of), 2),
        0xFF => (String::from("RST $38"), 1),
        _ => (format!("DB ${:02X}", op), 1),
    }
}




pub fn sdq(
    cw: Option<&GameBoyEmulator>,
    cx: u32, ae: u32, dt: u32, bm: u32,
) {
    
    framebuffer::ah(cx, ae, dt, bm, 0xFF0A0F14);
    
    if dt < 60 || bm < 40 { return; }
    
    let (bln, hgr, fd, hw, iil, ikh, joe, jrn, rxu, qsm) =
        if let Some(cw) = cw {
            (
                cw.aox & 0x04 == 0,
                cw.aox & 0x08 == 0,
                cw.aox & 0x02 == 0,
                cw.aox & 0x01 == 0,
                cw.aow & 0x01 == 0,
                cw.aow & 0x02 == 0,
                cw.aow & 0x04 == 0,
                cw.aow & 0x08 == 0,
                cw.aox,
                cw.aow,
            )
        } else {
            (false, false, false, false, false, false, false, false, 0xFF, 0xFF)
        };
    
    
    fu(cx + 6, ae + 4, "GAME BOY INPUT", F_);
    
    
    if dt > 300 {
        fu(cx + dt - 200, ae + 4, "WASD=Pad X=A Z=B C=Sel", 0xFF3A5A44);
    }
    
    
    let dqd = cx + 40;
    let dqe = ae + 30;
    let nf: u32 = 26;
    let qi: u32 = 2;
    
    
    irw(dqd, dqe - nf - qi, nf, nf, "W", bln);
    
    irw(dqd, dqe + nf + qi, nf, nf, "S", hgr);
    
    irw(dqd - nf - qi, dqe, nf, nf, "A", fd);
    
    irw(dqd + nf + qi, dqe, nf, nf, "D", hw);
    
    framebuffer::ah(dqd, dqe, nf, nf, 0xFF141E1A);
    
    
    let acv: u32 = 30;
    let iim = cx + dt - 80;
    let iin = ae + 30;
    let ikj = cx + dt - 140;
    let ikk = ae + 48;
    
    nne(iim, iin, acv, "A", iil);
    nne(ikj, ikk, acv, "B", ikh);
    
    
    fu(iim + acv + 4, iin + 8, "(X)", F_);
    fu(ikj - 28, ikk + 8, "(Z)", F_);
    
    
    let cgd = cx + dt / 2;
    let fqu = ae + bm - 36;
    nnf(cgd - 70, fqu, 56, 20, "SELECT", joe);
    nnf(cgd + 14, fqu, 56, 20, "START", jrn);
    
    
    fu(cgd - 70, fqu + 22, "(C)", F_);
    fu(cgd + 14, fqu + 22, "(Enter)", F_);
    
    
    let edf = ae + bm - 16;
    let bjw = alloc::format!("DIRS:{:02X}", rxu);
    let kex = alloc::format!("BTNS:{:02X}", qsm);
    fu(cx + 6, edf, &bjw, 0xFF3A5A44);
    fu(cx + 80, edf, &kex, 0xFF3A5A44);
}


pub fn tdt(cx: u32, ae: u32, dt: u32, bm: u32) -> [(u32, u32, u32, u32, u8); 8] {
    let dqd = cx + 40;
    let dqe = ae + 30;
    let nf: u32 = 26;
    let qi: u32 = 2;
    
    let acv: u32 = 30;
    let iim = cx + dt - 80;
    let iin = ae + 30;
    let ikj = cx + dt - 140;
    let ikk = ae + 48;
    
    let cgd = cx + dt / 2;
    let fqu = ae + bm - 36;
    
    [
        (dqd, dqe - nf - qi, nf, nf, b'w'),           
        (dqd, dqe + nf + qi, nf, nf, b's'),           
        (dqd - nf - qi, dqe, nf, nf, b'a'),           
        (dqd + nf + qi, dqe, nf, nf, b'd'),           
        (iim, iin, acv, acv, b'x'),                    
        (ikj, ikk, acv, acv, b'z'),                    
        (cgd - 70, fqu, 56, 20, b'c'),                  
        (cgd + 14, fqu, 56, 20, b'\r'),                 
    ]
}

fn irw(b: u32, c: u32, d: u32, i: u32, cu: &str, vn: bool) {
    let ei = if vn { 0xFF00CC66 } else { 0xFF1A2E24 };
    framebuffer::ah(b, c, d, i, ei);
    
    let atw = if vn { 0xFF00FF88 } else { 0xFF2A4A38 };
    framebuffer::ah(b, c, d, 1, atw);
    framebuffer::ah(b, c + i - 1, d, 1, atw);
    framebuffer::ah(b, c, 1, i, atw);
    framebuffer::ah(b + d - 1, c, 1, i, atw);
    let fwm = if vn { 0xFF000000 } else { 0xFF00FF88 };
    let gx = b + (d / 2).ao(4);
    let ty = c + (i / 2).ao(6);
    fu(gx, ty, cu, fwm);
}

fn nne(b: u32, c: u32, nf: u32, cu: &str, vn: bool) {
    let ei = if vn { 0xFF00CC66 } else { 0xFF1A2E24 };
    framebuffer::ah(b, c, nf, nf, ei);
    
    framebuffer::ah(b, c, 4, 4, 0xFF0A0F14);
    framebuffer::ah(b + nf - 4, c, 4, 4, 0xFF0A0F14);
    framebuffer::ah(b, c + nf - 4, 4, 4, 0xFF0A0F14);
    framebuffer::ah(b + nf - 4, c + nf - 4, 4, 4, 0xFF0A0F14);
    
    let atw = if vn { 0xFF00FF88 } else { 0xFF2A4A38 };
    framebuffer::ah(b + 3, c, nf - 6, 1, atw);
    framebuffer::ah(b + 3, c + nf - 1, nf - 6, 1, atw);
    framebuffer::ah(b, c + 3, 1, nf - 6, atw);
    framebuffer::ah(b + nf - 1, c + 3, 1, nf - 6, atw);
    let fwm = if vn { 0xFF000000 } else { 0xFF00FF88 };
    fu(b + nf / 2 - 4, c + nf / 2 - 6, cu, fwm);
}

fn nnf(b: u32, c: u32, d: u32, i: u32, cu: &str, vn: bool) {
    let ei = if vn { 0xFF00CC66 } else { 0xFF141E1A };
    framebuffer::ah(b, c, d, i, ei);
    let atw = if vn { 0xFF00FF88 } else { 0xFF2A4A38 };
    framebuffer::ah(b + 2, c, d - 4, 1, atw);
    framebuffer::ah(b + 2, c + i - 1, d - 4, 1, atw);
    framebuffer::ah(b, c + 2, 1, i - 4, atw);
    framebuffer::ah(b + d - 1, c + 2, 1, i - 4, atw);
    let fwm = if vn { 0xFF000000 } else { 0xFF80FFAA };
    let caj = cu.len() as u32 * 8;
    fu(b + (d.ao(caj)) / 2, c + (i / 2).ao(6), cu, fwm);
}
