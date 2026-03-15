


























use alloc::string::String;
use alloc::vec::Vec;
use alloc::vec;
use alloc::format;
use alloc::collections::VecDeque;






const JI_: u32 = 0xFF050606;

const DLT_: u32 = 0xFF00FF66;

const NA_: u32 = 0xFF00CC55;

const DLU_: u32 = 0xFF00AA44;

const DLW_: u32 = 0xFF008844;

const DLV_: u32 = 0xFF003B1A;


const APS_: u32 = 0xFF00FF88;

const EFK_: u32 = 0xFF1A3A2A;


const LR_: [u32; 16] = [
    0xFF050606, 
    0xFF882222, 
    0xFF00CC55, 
    0xFF888822, 
    0xFF4466AA, 
    0xFF884488, 
    0xFF448888, 
    0xFFAAAAAA, 
    0xFF666666, 
    0xFFFF5555, 
    0xFF00FF66, 
    0xFFFFFF00, 
    0xFF6688CC, 
    0xFFCC66CC, 
    0xFF66CCCC, 
    0xFFE0E8E4, 
];






#[derive(Debug, Clone, Copy)]
pub struct Cell {
    
    pub bm: char,
    
    pub lp: u32,
    
    pub ei: u32,
    
    pub qn: CellAttr,
}

impl Default for Cell {
    fn default() -> Self {
        Self {
            bm: ' ',
            lp: NA_,
            ei: JI_,
            qn: CellAttr::default(),
        }
    }
}


#[derive(Debug, Clone, Copy, Default)]
pub struct CellAttr {
    pub bpt: bool,
    pub tp: bool,
    pub gkl: bool,
    pub dde: bool,
    pub ilt: bool,
    pub dbh: bool,
    pub hidden: bool,
    pub dmb: bool,
}






#[derive(Debug, Clone, Copy, PartialEq)]
enum ParseState {
    
    M,
    
    Bgc,
    
    Bdx,
    
    Boh,
    
    Beh,
}


#[derive(Debug, Clone)]
pub enum Crl {
    
    Dew(char),
    
    Ctv(u16),
    
    Ctr(u16),
    
    Ctu(u16),
    
    Cts(u16),
    
    Ctt(u16, u16),
    
    Dik,
    
    Dgf,
    
    Cwc,
    
    Cwd,
    
    Cwb,
    
    Cvz,
    
    Cwa,
    
    Cvy,
    
    Dip(Vec<u16>),
    
    Din(u16),
    
    Dim(u16),
    
    Dio(String),
    
    Crz,
    
    Zz,
    
    Djr,
    
    Ddb,
    
    Cst,
    
    Diq,
    
    Cyj,
    
    F,
}






pub struct GraphicsTerminal {
    
    pub ec: u16,
    pub lk: u16,
    
    
    pub bpy: u16,
    pub bpx: u16,
    
    
    pub lf: u16,
    pub ot: u16,
    
    
    hyo: u16,
    hyp: u16,
    
    
    pub cwu: bool,
    
    
    btx: bool,
    byk: u32,
    
    
    bbr: Vec<Cell>,
    
    
    bsf: VecDeque<Vec<Cell>>,
    pgt: usize,
    
    
    px: usize,
    
    
    azl: CellAttr,
    btw: u32,
    dpj: u32,
    
    
    cba: ParseState,
    eod: Vec<u16>,
    gdv: String,
    goo: String,
    
    
    pub dq: String,
    
    
    pub cmz: Option<u32>,
    
    
    no: bool,
    
    
    pub nzd: bool,
    
    
    pub mci: u8,
}

impl GraphicsTerminal {
    
    pub fn new(z: u32, ac: u32) -> Self {
        
        let bpy = 8u16;
        let bpx = 16u16;
        let ec = (z / bpy as u32) as u16;
        let lk = (ac / bpx as u32) as u16;
        
        let ixh = (ec as usize) * (lk as usize);
        let bbr = vec![Cell::default(); ixh];
        
        let mut asc = Self {
            ec,
            lk,
            bpy,
            bpx,
            lf: 0,
            ot: 0,
            hyo: 0,
            hyp: 0,
            cwu: true,
            btx: true,
            byk: 0,
            bbr,
            bsf: VecDeque::new(),
            pgt: 1000,
            px: 0,
            azl: CellAttr::default(),
            btw: NA_,
            dpj: JI_,
            cba: ParseState::M,
            eod: Vec::new(),
            gdv: String::new(),
            goo: String::new(),
            dq: String::from("TrustOS Terminal"),
            cmz: None,
            no: true,
            nzd: true,
            mci: 20,
        };
        
        
        asc.write_str("\x1b[1;32m"); 
        asc.write_str("╔══════════════════════════════════════════════════════════╗\r\n");
        asc.write_str("║  \x1b[1;97mTrustOS\x1b[1;32m Graphical Terminal                             ║\r\n");
        asc.write_str("║  Matrix Edition v1.0                                     ║\r\n");
        asc.write_str("╚══════════════════════════════════════════════════════════╝\r\n");
        asc.write_str("\x1b[0;32m"); 
        asc.write_str("\r\n");
        
        asc
    }
    
    
    pub fn write_str(&mut self, e: &str) {
        for r in e.bw() {
            self.write_char(r);
        }
    }
    
    
    pub fn write_char(&mut self, r: char) {
        match self.cba {
            ParseState::M => self.tkn(r),
            ParseState::Bgc => self.tjm(r),
            ParseState::Bdx => self.tjg(r),
            ParseState::Boh => self.tkp(r),
            ParseState::Beh => self.tji(r),
        }
        self.no = true;
    }
    
    fn tkn(&mut self, r: char) {
        match r {
            '\x1b' => {
                self.cba = ParseState::Bgc;
            }
            '\n' => {
                self.agr();
            }
            '\r' => {
                self.lf = 0;
            }
            '\x08' => {
                
                if self.lf > 0 {
                    self.lf -= 1;
                }
            }
            '\t' => {
                
                let uun = ((self.lf / 8) + 1) * 8;
                self.lf = uun.v(self.ec - 1);
            }
            '\x07' => {
                
            }
            _ if r >= ' ' => {
                self.vol(r);
            }
            _ => {
                
            }
        }
    }
    
    fn tjm(&mut self, r: char) {
        match r {
            '[' => {
                self.cba = ParseState::Bdx;
                self.eod.clear();
                self.gdv.clear();
            }
            ']' => {
                self.cba = ParseState::Boh;
                self.goo.clear();
            }
            'P' => {
                self.cba = ParseState::Beh;
            }
            'c' => {
                
                self.apa();
                self.cba = ParseState::M;
            }
            '7' => {
                
                self.hyo = self.lf;
                self.hyp = self.ot;
                self.cba = ParseState::M;
            }
            '8' => {
                
                self.lf = self.hyo;
                self.ot = self.hyp;
                self.cba = ParseState::M;
            }
            'D' => {
                
                self.agr();
                self.cba = ParseState::M;
            }
            'M' => {
                
                if self.ot > 0 {
                    self.ot -= 1;
                }
                self.cba = ParseState::M;
            }
            'E' => {
                
                self.lf = 0;
                self.agr();
                self.cba = ParseState::M;
            }
            _ => {
                
                self.cba = ParseState::M;
            }
        }
    }
    
    fn tjg(&mut self, r: char) {
        if r.atb() || r == ';' {
            self.gdv.push(r);
        } else {
            
            self.eod.clear();
            for vu in self.gdv.adk(';') {
                if let Ok(bo) = vu.parse::<u16>() {
                    self.eod.push(bo);
                } else {
                    self.eod.push(0);
                }
            }
            
            
            self.som(r);
            self.cba = ParseState::M;
        }
    }
    
    fn som(&mut self, cmd: char) {
        let oi = &self.eod;
        let ags = oi.fv().hu().unwrap_or(1).am(1);
        let pr = oi.get(1).hu().unwrap_or(1).am(1);
        
        match cmd {
            'A' => {
                
                self.ot = self.ot.ao(ags);
            }
            'B' => {
                
                self.ot = (self.ot + ags).v(self.lk - 1);
            }
            'C' => {
                
                self.lf = (self.lf + ags).v(self.ec - 1);
            }
            'D' => {
                
                self.lf = self.lf.ao(ags);
            }
            'E' => {
                
                self.lf = 0;
                self.ot = (self.ot + ags).v(self.lk - 1);
            }
            'F' => {
                
                self.lf = 0;
                self.ot = self.ot.ao(ags);
            }
            'G' => {
                
                self.lf = (ags - 1).v(self.ec - 1);
            }
            'H' | 'f' => {
                
                let br = oi.fv().hu().unwrap_or(1).am(1);
                let bj = oi.get(1).hu().unwrap_or(1).am(1);
                self.ot = (br - 1).v(self.lk - 1);
                self.lf = (bj - 1).v(self.ec - 1);
            }
            'J' => {
                
                let ev = oi.fv().hu().unwrap_or(0);
                match ev {
                    0 => self.sne(),
                    1 => self.snf(),
                    2 | 3 => self.nqy(),
                    _ => {}
                }
            }
            'K' => {
                
                let ev = oi.fv().hu().unwrap_or(0);
                match ev {
                    0 => self.nqz(),
                    1 => self.nra(),
                    2 => self.snd(),
                    _ => {}
                }
            }
            'S' => {
                
                for _ in 0..ags {
                    self.dlm();
                }
            }
            'T' => {
                
                for _ in 0..ags {
                    self.eid();
                }
            }
            'm' => {
                
                self.sot();
            }
            's' => {
                
                self.hyo = self.lf;
                self.hyp = self.ot;
            }
            'u' => {
                
                self.lf = self.hyo;
                self.ot = self.hyp;
            }
            '?' if !oi.is_empty() => {
                
            }
            'h' => {
                
                if self.gdv.cj('?') {
                    let ev = oi.fv().hu().unwrap_or(0);
                    if ev == 25 {
                        self.cwu = true;
                    }
                }
            }
            'l' => {
                
                if self.gdv.cj('?') {
                    let ev = oi.fv().hu().unwrap_or(0);
                    if ev == 25 {
                        self.cwu = false;
                    }
                }
            }
            _ => {
                
            }
        }
    }
    
    fn sot(&mut self) {
        let oi = if self.eod.is_empty() {
            vec![0] 
        } else {
            self.eod.clone()
        };
        
        let mut a = 0;
        while a < oi.len() {
            let ai = oi[a];
            match ai {
                0 => {
                    
                    self.azl = CellAttr::default();
                    self.btw = NA_;
                    self.dpj = JI_;
                }
                1 => self.azl.bpt = true,
                2 => self.azl.tp = true,
                3 => self.azl.gkl = true,
                4 => self.azl.dde = true,
                5 => self.azl.ilt = true,
                7 => self.azl.dbh = true,
                8 => self.azl.hidden = true,
                9 => self.azl.dmb = true,
                21 => self.azl.bpt = false,
                22 => {
                    self.azl.bpt = false;
                    self.azl.tp = false;
                }
                23 => self.azl.gkl = false,
                24 => self.azl.dde = false,
                25 => self.azl.ilt = false,
                27 => self.azl.dbh = false,
                28 => self.azl.hidden = false,
                29 => self.azl.dmb = false,
                
                30..=37 => {
                    let w = (ai - 30) as usize;
                    self.btw = LR_[w];
                }
                38 => {
                    
                    if a + 2 < oi.len() && oi[a + 1] == 5 {
                        
                        let w = oi[a + 2] as usize;
                        self.btw = self.ney(w);
                        a += 2;
                    } else if a + 4 < oi.len() && oi[a + 1] == 2 {
                        
                        let m = oi[a + 2] as u32;
                        let at = oi[a + 3] as u32;
                        let o = oi[a + 4] as u32;
                        self.btw = 0xFF000000 | (m << 16) | (at << 8) | o;
                        a += 4;
                    }
                }
                39 => self.btw = NA_, 
                
                40..=47 => {
                    let w = (ai - 40) as usize;
                    self.dpj = LR_[w];
                }
                48 => {
                    
                    if a + 2 < oi.len() && oi[a + 1] == 5 {
                        let w = oi[a + 2] as usize;
                        self.dpj = self.ney(w);
                        a += 2;
                    } else if a + 4 < oi.len() && oi[a + 1] == 2 {
                        let m = oi[a + 2] as u32;
                        let at = oi[a + 3] as u32;
                        let o = oi[a + 4] as u32;
                        self.dpj = 0xFF000000 | (m << 16) | (at << 8) | o;
                        a += 4;
                    }
                }
                49 => self.dpj = JI_, 
                
                90..=97 => {
                    let w = (ai - 90 + 8) as usize;
                    self.btw = LR_[w];
                }
                
                100..=107 => {
                    let w = (ai - 100 + 8) as usize;
                    self.dpj = LR_[w];
                }
                _ => {}
            }
            a += 1;
        }
    }
    
    
    fn ney(&self, w: usize) -> u32 {
        if w < 16 {
            LR_[w]
        } else if w < 232 {
            
            let w = w - 16;
            let m = (w / 36) % 6;
            let at = (w / 6) % 6;
            let o = w % 6;
            let m = if m > 0 { m * 40 + 55 } else { 0 };
            let at = if at > 0 { at * 40 + 55 } else { 0 };
            let o = if o > 0 { o * 40 + 55 } else { 0 };
            0xFF000000 | ((m as u32) << 16) | ((at as u32) << 8) | (o as u32)
        } else {
            
            let laj = (w - 232) * 10 + 8;
            0xFF000000 | ((laj as u32) << 16) | ((laj as u32) << 8) | (laj as u32)
        }
    }
    
    fn tkp(&mut self, r: char) {
        if r == '\x07' || r == '\x1b' {
            
            self.soq();
            self.cba = ParseState::M;
        } else {
            self.goo.push(r);
        }
    }
    
    fn soq(&mut self) {
        
        if let Some(w) = self.goo.du(';') {
            let cmd = &self.goo[..w];
            let f = &self.goo[w + 1..];
            
            match cmd {
                "0" | "2" => {
                    
                    self.dq = String::from(f);
                }
                _ => {}
            }
        }
    }
    
    fn tji(&mut self, r: char) {
        
        if r == '\x1b' || r == '\\' {
            self.cba = ParseState::M;
        }
    }
    
    
    fn vol(&mut self, r: char) {
        if self.lf >= self.ec {
            self.lf = 0;
            self.agr();
        }
        
        let w = self.ot as usize * self.ec as usize + self.lf as usize;
        if w < self.bbr.len() {
            
            let (lp, ei) = if self.azl.dbh {
                (self.dpj, self.btw)
            } else {
                (self.btw, self.dpj)
            };
            
            let lp = if self.azl.bpt {
                self.qrz(lp)
            } else if self.azl.tp {
                self.eot(lp)
            } else {
                lp
            };
            
            self.bbr[w] = Cell {
                bm: r,
                lp,
                ei,
                qn: self.azl,
            };
        }
        
        self.lf += 1;
    }
    
    
    fn qrz(&self, s: u32) -> u32 {
        let m = ((s >> 16) & 0xFF).v(255);
        let at = ((s >> 8) & 0xFF).v(255);
        let o = (s & 0xFF).v(255);
        
        let m = (m + (255 - m) / 3).v(255);
        let at = (at + (255 - at) / 3).v(255);
        let o = (o + (255 - o) / 3).v(255);
        
        0xFF000000 | (m << 16) | (at << 8) | o
    }
    
    
    fn eot(&self, s: u32) -> u32 {
        let m = ((s >> 16) & 0xFF) * 2 / 3;
        let at = ((s >> 8) & 0xFF) * 2 / 3;
        let o = (s & 0xFF) * 2 / 3;
        
        0xFF000000 | (m << 16) | (at << 8) | o
    }
    
    
    fn agr(&mut self) {
        if self.ot >= self.lk - 1 {
            self.dlm();
        } else {
            self.ot += 1;
        }
    }
    
    
    fn dlm(&mut self) {
        
        let xjo: Vec<Cell> = self.bbr[..self.ec as usize].ip();
        self.bsf.agt(xjo);
        
        
        while self.bsf.len() > self.pgt {
            self.bsf.awp();
        }
        
        
        let ec = self.ec as usize;
        for c in 0..self.lk as usize - 1 {
            let big = (c + 1) * ec;
            let dqh = c * ec;
            for b in 0..ec {
                self.bbr[dqh + b] = self.bbr[big + b];
            }
        }
        
        
        let ucm = (self.lk as usize - 1) * ec;
        for b in 0..ec {
            self.bbr[ucm + b] = Cell::default();
        }
    }
    
    
    fn eid(&mut self) {
        let ec = self.ec as usize;
        
        
        for c in (1..self.lk as usize).vv() {
            let big = (c - 1) * ec;
            let dqh = c * ec;
            for b in 0..ec {
                self.bbr[dqh + b] = self.bbr[big + b];
            }
        }
        
        
        for b in 0..ec {
            self.bbr[b] = Cell::default();
        }
    }
    
    
    fn sne(&mut self) {
        
        self.nqz();
        
        
        let ec = self.ec as usize;
        for c in (self.ot + 1) as usize..self.lk as usize {
            let mu = c * ec;
            for b in 0..ec {
                self.bbr[mu + b] = Cell::default();
            }
        }
    }
    
    
    fn snf(&mut self) {
        
        self.nra();
        
        
        let ec = self.ec as usize;
        for c in 0..self.ot as usize {
            let mu = c * ec;
            for b in 0..ec {
                self.bbr[mu + b] = Cell::default();
            }
        }
    }
    
    
    fn nqy(&mut self) {
        for cell in &mut self.bbr {
            *cell = Cell::default();
        }
    }
    
    
    fn nqz(&mut self) {
        let mu = self.ot as usize * self.ec as usize;
        for b in self.lf as usize..self.ec as usize {
            self.bbr[mu + b] = Cell::default();
        }
    }
    
    
    fn nra(&mut self) {
        let mu = self.ot as usize * self.ec as usize;
        for b in 0..=self.lf as usize {
            if mu + b < self.bbr.len() {
                self.bbr[mu + b] = Cell::default();
            }
        }
    }
    
    
    fn snd(&mut self) {
        let mu = self.ot as usize * self.ec as usize;
        for b in 0..self.ec as usize {
            self.bbr[mu + b] = Cell::default();
        }
    }
    
    
    pub fn apa(&mut self) {
        self.lf = 0;
        self.ot = 0;
        self.azl = CellAttr::default();
        self.btw = NA_;
        self.dpj = JI_;
        self.cwu = true;
        self.nqy();
    }
    
    
    pub fn vr(&mut self, r: char) {
        
        
        if r == '\n' {
            self.write_str("\r\n");
        } else {
            self.write_char(r);
        }
    }
    
    
    pub fn tj(&mut self) -> Vec<u32> {
        let z = self.ec as usize * self.bpy as usize;
        let ac = self.lk as usize * self.bpx as usize;
        
        
        let mut bi = vec![0u32; z * ac];
        crate::graphics::simd::ntp(&mut bi, JI_);
        
        
        self.byk = self.byk.cn(1);
        if self.byk % 30 == 0 {
            self.btx = !self.btx;
        }
        
        
        for c in 0..self.lk as usize {
            for b in 0..self.ec as usize {
                let w = c * self.ec as usize + b;
                let cell = &self.bbr[w];
                
                
                self.scd(&mut bi, z, b as u32, c as u32, cell.ei);
                
                
                if cell.bm != ' ' {
                    self.ahi(&mut bi, z as u32, b as u32, c as u32, cell.bm, cell.lp);
                }
                
                
                if cell.qn.dde {
                    self.sgk(&mut bi, z, b as u32, c as u32, cell.lp);
                }
            }
        }
        
        
        if self.cwu && self.btx {
            self.scl(&mut bi, z);
        }
        
        
        if self.nzd {
            self.qju(&mut bi, z as u32, ac as u32);
        }
        
        
        if self.mci > 0 {
            self.qka(&mut bi, z as u32, ac as u32);
        }
        
        self.no = false;
        bi
    }
    
    
    fn scd(&self, bi: &mut [u32], z: usize, cx: u32, ae: u32, ei: u32) {
        let bwi = cx as usize * self.bpy as usize;
        let cto = ae as usize * self.bpx as usize;
        let acc = self.bpy as usize;
        
        
        for bg in 0..self.bpx as usize {
            let mu = (cto + bg) * z + bwi;
            if mu + acc <= bi.len() {
                #[cfg(target_arch = "x86_64")]
                unsafe {
                    crate::graphics::simd::bed(
                        bi.mw().add(mu),
                        acc,
                        ei
                    );
                }
                #[cfg(not(target_arch = "x86_64"))]
                {
                    bi[mu..mu + acc].vi(ei);
                }
            }
        }
    }

    fn ymv(&self, bi: &mut [u32], z: u32, cx: u32, ae: u32, ei: u32) {
        let bwi = cx * self.bpy as u32;
        let cto = ae * self.bpx as u32;
        
        for bg in 0..self.bpx as u32 {
            for dx in 0..self.bpy as u32 {
                let w = ((cto + bg) * z + bwi + dx) as usize;
                if w < bi.len() {
                    bi[w] = ei;
                }
            }
        }
    }
    
    fn ahi(&self, bi: &mut [u32], z: u32, cx: u32, ae: u32, r: char, lp: u32) {
        let ka = crate::framebuffer::font::ada(r);
        let bwi = cx * self.bpy as u32;
        let cto = ae * self.bpx as u32;
        
        for (bwv, &br) in ka.iter().cf() {
            for ga in 0..8 {
                if (br >> (7 - ga)) & 1 == 1 {
                    let b = bwi + ga;
                    let c = cto + bwv as u32;
                    let w = (c * z + b) as usize;
                    if w < bi.len() {
                        bi[w] = lp;
                    }
                }
            }
        }
    }
    
    fn yno(&self, bi: &mut [u32], z: u32, cx: u32, ae: u32, lp: u32) {
        let bwi = cx * self.bpy as u32;
        let cto = ae * self.bpx as u32 + self.bpx as u32 - 2;
        
        for dx in 0..self.bpy as u32 {
            let w = (cto * z + bwi + dx) as usize;
            if w < bi.len() {
                bi[w] = lp;
            }
        }
    }
    
    
    fn sgk(&self, bi: &mut [u32], z: usize, cx: u32, ae: u32, lp: u32) {
        let bwi = cx as usize * self.bpy as usize;
        let cto = ae as usize * self.bpx as usize + self.bpx as usize - 2;
        let acc = self.bpy as usize;
        
        let ay = cto * z + bwi;
        if ay + acc <= bi.len() {
            #[cfg(target_arch = "x86_64")]
            unsafe {
                crate::graphics::simd::bed(
                    bi.mw().add(ay),
                    acc,
                    lp
                );
            }
            #[cfg(not(target_arch = "x86_64"))]
            {
                bi[ay..ay + acc].vi(lp);
            }
        }
    }
    
    fn dqf(&self, bi: &mut [u32], z: u32) {
        let bwi = self.lf as u32 * self.bpy as u32;
        let cto = self.ot as u32 * self.bpx as u32;
        
        
        for bg in 0..self.bpx as u32 {
            for dx in 0..self.bpy as u32 {
                let w = ((cto + bg) * z + bwi + dx) as usize;
                if w < bi.len() {
                    
                    let xy = bi[w];
                    bi[w] = xy ^ APS_;
                }
            }
        }
    }
    
    
    fn scl(&self, bi: &mut [u32], z: usize) {
        let bwi = self.lf as usize * self.bpy as usize;
        let cto = self.ot as usize * self.bpx as usize;
        let acc = self.bpy as usize;
        
        
        for bg in 0..self.bpx as usize {
            let mu = (cto + bg) * z + bwi;
            if mu + acc <= bi.len() {
                for dx in 0..acc {
                    let xy = bi[mu + dx];
                    bi[mu + dx] = xy ^ APS_;
                }
            }
        }
    }
    
    
    fn qju(&self, bi: &mut [u32], z: u32, ac: u32) {
        
        
        
        let mut bcz = bi.ip();
        
        for c in 0..ac {
            for b in 1..(z - 1) {
                let w = (c * z + b) as usize;
                let fd = bi[(c * z + b - 1) as usize];
                let pn = bi[w];
                let hw = bi[(c * z + b + 1) as usize];
                
                
                let nxa = (fd >> 8) & 0xFF;
                let nwz = (pn >> 8) & 0xFF;
                let nxb = (hw >> 8) & 0xFF;
                
                if nwz > 100 || nxa > 100 || nxb > 100 {
                    let tq = ((nxa + nwz * 2 + nxb) / 4).v(255);
                    let m = (pn >> 16) & 0xFF;
                    let o = pn & 0xFF;
                    bcz[w] = 0xFF000000 | (m << 16) | (tq << 8) | o;
                }
            }
        }
        
        bi.dg(&bcz);
    }
    
    
    fn qka(&self, bi: &mut [u32], z: u32, ac: u32) {
        let hj = 255 - self.mci as u32;
        
        for c in (1..ac).akt(2) {
            let mu = (c * z) as usize;
            let cub = ((c + 1) * z) as usize;
            
            if cub <= bi.len() {
                for w in mu..cub.v(mu + z as usize) {
                    let il = bi[w];
                    let m = ((il >> 16) & 0xFF) * hj / 255;
                    let at = ((il >> 8) & 0xFF) * hj / 255;
                    let o = (il & 0xFF) * hj / 255;
                    bi[w] = 0xFF000000 | (m << 16) | (at << 8) | o;
                }
            }
        }
    }
    
    
    pub fn yzi(&self) -> bool {
        self.no
    }
    
    
    pub fn vii(&self) -> (u32, u32) {
        (
            self.ec as u32 * self.bpy as u32,
            self.lk as u32 * self.bpx as u32,
        )
    }
}





use spin::Mutex;

static NI_: Mutex<Option<GraphicsTerminal>> = Mutex::new(None);


pub fn init(z: u32, ac: u32) -> Result<(), &'static str> {
    let mut asc = NI_.lock();
    if asc.is_some() {
        return Err("Graphics terminal already initialized");
    }
    
    *asc = Some(GraphicsTerminal::new(z, ac));
    crate::serial_println!("[GTERM] Graphics terminal initialized ({}x{})", z, ac);
    Ok(())
}


pub fn write(e: &str) {
    if let Some(asc) = NI_.lock().as_mut() {
        asc.write_str(e);
    }
}


pub fn tj() -> Option<Vec<u32>> {
    NI_.lock().as_mut().map(|asc| asc.tj())
}


pub fn vr(r: char) {
    if let Some(asc) = NI_.lock().as_mut() {
        asc.vr(r);
    }
}


pub fn gii() -> Option<(u32, u32)> {
    NI_.lock().as_ref().map(|asc| asc.vii())
}
