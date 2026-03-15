







use alloc::string::String;
use alloc::vec::Vec;
use alloc::format;





pub const Y: i8 = 0;
pub const DBT_: i8 = 1;
pub const AJZ_: i8 = 2;
pub const AJY_: i8 = 3;
pub const AKA_: i8 = 4;
pub const BJE_: i8 = 5;
pub const BJD_: i8 = 6;
pub const BLO_: i8 = -1;
pub const ANO_: i8 = -2;
pub const ANM_: i8 = -3;
pub const ANP_: i8 = -4;
pub const BLP_: i8 = -5;
pub const ANN_: i8 = -6;


fn aun(ai: i8) -> bool { ai > 0 }

fn din(ai: i8) -> bool { ai < 0 }

fn bkv(ai: i8) -> i8 { if ai < 0 { -ai } else { ai } }


fn ovp(ai: i8) -> i32 {
    match bkv(ai) {
        1 => 100,   
        2 => 320,   
        3 => 330,   
        4 => 500,   
        5 => 900,   
        6 => 20000, 
        _ => 0,
    }
}


pub fn zfc(ai: i8) -> char {
    match ai {
        1  => '\u{2659}', 
        2  => '\u{2658}', 
        3  => '\u{2657}', 
        4  => '\u{2656}', 
        5  => '\u{2655}', 
        6  => '\u{2654}', 
        -1 => '\u{265G}', 
        -2 => '\u{265E}', 
        -3 => '\u{265Bdy}', 
        -4 => '\u{265C}', 
        -5 => '\u{265Byb}', 
        -6 => '\u{265Bxd}', 
        _  => ' ',
    }
}


pub fn hvd(ai: i8) -> char {
    match ai {
        1  => 'P', 2  => 'N', 3  => 'B', 4  => 'R', 5  => 'Q', 6  => 'K',
        -1 => 'p', -2 => 'n', -3 => 'b', -4 => 'r', -5 => 'q', -6 => 'k',
        _  => ' ',
    }
}


pub fn zfd(ai: i8) -> u32 {
    if aun(ai) { 0xFFFFFFFF } else { 0xFF1A1A1A }
}





#[derive(Clone, Copy, PartialEq)]
pub enum GamePhase {
    Ce,
    Aam,
    Mw,
    Up,
    Yg,
}

#[derive(Clone, Copy, PartialEq)]
pub enum InputMode {
    Uk,   
    Amj,  
}

pub struct ChessState {
    pub mn: [i8; 64],
    pub axi: bool,
    pub na: Option<usize>,     
    pub gi: usize,               
    pub blr: Vec<usize>,     
    pub ib: GamePhase,
    pub edj: InputMode,
    pub message: String,
    pub gnd: Vec<String>,
    
    pub mqm: bool,
    pub kdo: bool,
    pub mqn: bool,
    pub mqo: bool,
    pub kdp: bool,
    pub kdq: bool,
    
    pub bgx: Option<usize>, 
    
    pub jkh: Option<usize>,
    
    pub mqa: bool,
    pub qga: bool,
    pub fzz: i32,
    pub ajn: u32,
    
    pub jcn: Option<usize>,
    pub jco: Option<usize>,
    
    pub dgo: Option<usize>,     
    pub epb: Option<i8>,   
    pub kqq: i32,           
    pub kqr: i32,           
    
    pub fyr: u64,
    pub fdl: u64,
    pub ezv: bool,
    pub dww: u64,       
    pub uco: u64,           
    pub ezw: bool,         
}

impl ChessState {
    pub fn new() -> Self {
        let mut g = ChessState {
            mn: [Y; 64],
            axi: true,
            na: None,
            gi: 52, 
            blr: Vec::new(),
            ib: GamePhase::Ce,
            edj: InputMode::Uk,
            message: String::from("White to move"),
            gnd: Vec::new(),
            mqm: false,
            kdo: false,
            mqn: false,
            mqo: false,
            kdp: false,
            kdq: false,
            bgx: None,
            jkh: None,
            mqa: true,
            qga: false,
            fzz: 2,
            ajn: 12345,
            jcn: None,
            jco: None,
            
            dgo: None,
            epb: None,
            kqq: 0,
            kqr: 0,
            
            fyr: 600_000,  
            fdl: 600_000,
            ezv: false,
            dww: 600_000,
            uco: 0,
            ezw: false,
        };
        g.ttf();
        g
    }

    fn ttf(&mut self) {
        
        self.mn[0] = ANP_;
        self.mn[1] = ANO_;
        self.mn[2] = ANM_;
        self.mn[3] = BLP_;
        self.mn[4] = ANN_;
        self.mn[5] = ANM_;
        self.mn[6] = ANO_;
        self.mn[7] = ANP_;
        for a in 8..16 {
            self.mn[a] = BLO_;
        }
        
        for a in 48..56 {
            self.mn[a] = DBT_;
        }
        self.mn[56] = AKA_;
        self.mn[57] = AJZ_;
        self.mn[58] = AJY_;
        self.mn[59] = BJE_;
        self.mn[60] = BJD_;
        self.mn[61] = AJY_;
        self.mn[62] = AJZ_;
        self.mn[63] = AKA_;
    }

    

    fn br(im: usize) -> usize { im / 8 }
    fn bj(im: usize) -> usize { im % 8 }
    fn im(br: usize, bj: usize) -> usize { br * 8 + bj }

    fn gsv(im: usize) -> String {
        let file = (b'a' + Self::bj(im) as u8) as char;
        let vqe = (b'1' + (7 - Self::br(im)) as u8) as char;
        format!("{}{}", file, vqe)
    }

    

    pub fn vr(&mut self, bs: u8) {
        use crate::keyboard::{V_, U_, AH_, AI_};

        if self.ib == GamePhase::Mw || self.ib == GamePhase::Up {
            if bs == b'r' || bs == b'R' {
                *self = ChessState::new();
            }
            return;
        }

        
        if self.ib == GamePhase::Yg {
            if let Some(im) = self.jkh {
                let iot: i8 = if Self::br(im) == 0 { 1 } else { -1 };
                match bs {
                    b'q' | b'Q' | b'1' => { self.mn[im] = BJE_ * iot; self.iut(); }
                    b'r' | b'R' | b'2' => { self.mn[im] = AKA_ * iot; self.iut(); }
                    b'b' | b'B' | b'3' => { self.mn[im] = AJY_ * iot; self.iut(); }
                    b'n' | b'N' | b'4' => { self.mn[im] = AJZ_ * iot; self.iut(); }
                    _ => {}
                }
            }
            return;
        }

        match bs {
            V_    => { if self.gi >= 8 { self.gi -= 8; } },
            U_  => { if self.gi < 56 { self.gi += 8; } },
            AH_  => { if self.gi % 8 > 0 { self.gi -= 1; } },
            AI_ => { if self.gi % 8 < 7 { self.gi += 1; } },
            0x0D | b' ' => { 
                self.tkv();
            },
            0x1B => { 
                self.na = None;
                self.blr.clear();
                self.edj = InputMode::Uk;
                self.message = if self.axi {
                    String::from("White to move")
                } else {
                    String::from("Black to move")
                };
            },
            b'r' | b'R' => {
                *self = ChessState::new();
            },
            b't' | b'T' => {
                self.xiz();
            },
            b'+' | b'=' => {
                if self.ezv {
                    self.rsr();
                }
            },
            b'd' | b'D' => {
                
                self.fzz = match self.fzz {
                    1 => 2,
                    2 => 3,
                    _ => 1,
                };
                self.message = match self.fzz {
                    1 => String::from("AI: Easy (depth 1)"),
                    2 => String::from("AI: Medium (depth 2)"),
                    _ => String::from("AI: Hard (depth 3)"),
                };
            },
            _ => {}
        }
    }

    fn tkv(&mut self) {
        match self.edj {
            InputMode::Uk => {
                let xe = self.mn[self.gi];
                if xe == Y { return; }
                
                if self.axi && !aun(xe) { 
                    self.message = String::from("Select a white piece");
                    return; 
                }
                if !self.axi && !din(xe) { 
                    self.message = String::from("Select a black piece");
                    return; 
                }
                
                self.blr = self.erb(self.gi);
                if self.blr.is_empty() {
                    self.message = String::from("No legal moves for this piece");
                    return;
                }
                self.na = Some(self.gi);
                self.edj = InputMode::Amj;
                let j = Self::gsv(self.gi);
                self.message = format!("Move {} from {}", hvd(xe), j);
            },
            InputMode::Amj => {
                let from = match self.na {
                    Some(im) => im,
                    None => { self.edj = InputMode::Uk; return; }
                };
                
                
                let jsn = self.mn[self.gi];
                if jsn != Y {
                    let hpa = (self.axi && aun(jsn)) || 
                                 (!self.axi && din(jsn));
                    if hpa {
                        
                        self.blr = self.erb(self.gi);
                        if !self.blr.is_empty() {
                            self.na = Some(self.gi);
                            let j = Self::gsv(self.gi);
                            self.message = format!("Move {} from {}", hvd(jsn), j);
                        }
                        return;
                    }
                }

                if !self.blr.contains(&self.gi) {
                    self.message = String::from("Invalid move");
                    return;
                }

                
                self.jet(from, self.gi);
            },
        }
    }

    fn jet(&mut self, from: usize, wh: usize) {
        let xe = self.mn[from];
        let bjm = self.mn[wh];
        
        
        let upw = format!("{}{}{}", hvd(xe), Self::gsv(from), Self::gsv(wh));
        self.gnd.push(upw);
        
        
        self.jcn = Some(from);
        self.jco = Some(wh);

        
        if bkv(xe) == 1 && Some(wh) == self.bgx {
            
            if aun(xe) {
                self.mn[wh + 8] = Y; 
            } else {
                self.mn[wh - 8] = Y; 
            }
        }

        
        if bkv(xe) == 6 {
            let nep = Self::bj(wh) as i32 - Self::bj(from) as i32;
            if nep == 2 {
                
                self.mn[wh - 1] = self.mn[wh + 1]; 
                self.mn[wh + 1] = Y;
            } else if nep == -2 {
                
                self.mn[wh + 1] = self.mn[wh - 2]; 
                self.mn[wh - 2] = Y;
            }
        }

        
        self.mn[wh] = xe;
        self.mn[from] = Y;

        
        if bkv(xe) == 6 {
            if aun(xe) { self.mqm = true; }
            else { self.kdo = true; }
        }
        if bkv(xe) == 4 {
            if from == 56 { self.mqn = true; }
            if from == 63 { self.mqo = true; }
            if from == 0 { self.kdp = true; }
            if from == 7 { self.kdq = true; }
        }

        
        self.bgx = None;
        if bkv(xe) == 1 {
            let wal = Self::br(wh) as i32 - Self::br(from) as i32;
            if wal.gp() == 2 {
                
                self.bgx = Some(((from as i32 + wh as i32) / 2) as usize);
            }
        }

        
        if bkv(xe) == 1 && (Self::br(wh) == 0 || Self::br(wh) == 7) {
            self.ib = GamePhase::Yg;
            self.jkh = Some(wh);
            self.message = String::from("Promote: Q/R/B/N");
            self.na = None;
            self.blr.clear();
            self.edj = InputMode::Uk;
            return;
        }

        self.nuq();
    }

    fn iut(&mut self) {
        self.jkh = None;
        self.nuq();
    }

    fn nuq(&mut self) {
        self.axi = !self.axi;
        self.na = None;
        self.blr.clear();
        self.edj = InputMode::Uk;

        
        if self.ezv && !self.ezw {
            self.ezw = true;
        }

        
        let lds = self.txr(self.axi);
        let oat = self.tmb(self.axi);

        if lds && !oat {
            self.ib = GamePhase::Mw;
            self.message = if self.axi {
                String::from("Checkmate! Black wins!")
            } else {
                String::from("Checkmate! White wins!")
            };
        } else if !lds && !oat {
            self.ib = GamePhase::Up;
            self.message = String::from("Stalemate — Draw!");
        } else if lds {
            self.ib = GamePhase::Aam;
            self.message = if self.axi {
                String::from("White in check!")
            } else {
                String::from("Black in check!")
            };
        } else {
            self.ib = GamePhase::Ce;
            self.message = if self.axi {
                String::from("White to move")
            } else {
                String::from("Black to move")
            };
        }

        
        if self.mqa && !self.axi && self.ib == GamePhase::Ce || 
           (self.mqa && !self.axi && self.ib == GamePhase::Aam) {
            self.qfz();
        }
    }

    
    
    

    
    fn tec(&self, im: usize) -> Vec<usize> {
        let xe = self.mn[im];
        if xe == Y { return Vec::new(); }
        let cfs = aun(xe);
        let mut bev = Vec::new();
        let br = Self::br(im);
        let bj = Self::bj(im);

        match bkv(xe) {
            1 => { 
                let te: i32 = if cfs { -1 } else { 1 };
                let dwe = if cfs { 6 } else { 1 };
                
                
                let iwd = im as i32 + te * 8;
                if iwd >= 0 && iwd < 64 && self.mn[iwd as usize] == Y {
                    bev.push(iwd as usize);
                    
                    if br == dwe {
                        let iwe = im as i32 + te * 16;
                        if iwe >= 0 && iwe < 64 && self.mn[iwe as usize] == Y {
                            bev.push(iwe as usize);
                        }
                    }
                }
                
                for bmr in [-1i32, 1] {
                    let djs = bj as i32 + bmr;
                    if djs >= 0 && djs < 8 {
                        let cd = (br as i32 + te) * 8 + djs;
                        if cd >= 0 && cd < 64 {
                            let ab = cd as usize;
                            let aaz = self.mn[ab];
                            if (aaz != Y && aun(aaz) != cfs) || Some(ab) == self.bgx {
                                bev.push(ab);
                            }
                        }
                    }
                }
            },
            2 => { 
                let bkr: [(i32, i32); 8] = [
                    (-2,-1),(-2,1),(-1,-2),(-1,2),(1,-2),(1,2),(2,-1),(2,1)
                ];
                for (ahh, bmr) in bkr {
                    let nr = br as i32 + ahh;
                    let djs = bj as i32 + bmr;
                    if nr >= 0 && nr < 8 && djs >= 0 && djs < 8 {
                        let ab = Self::im(nr as usize, djs as usize);
                        let aaz = self.mn[ab];
                        if aaz == Y || aun(aaz) != cfs {
                            bev.push(ab);
                        }
                    }
                }
            },
            3 => { 
                self.mgc(im, cfs, &[(-1,-1),(-1,1),(1,-1),(1,1)], &mut bev);
            },
            4 => { 
                self.mgc(im, cfs, &[(-1,0),(1,0),(0,-1),(0,1)], &mut bev);
            },
            5 => { 
                self.mgc(im, cfs, &[(-1,-1),(-1,1),(1,-1),(1,1),(-1,0),(1,0),(0,-1),(0,1)], &mut bev);
            },
            6 => { 
                let bkr: [(i32, i32); 8] = [
                    (-1,-1),(-1,0),(-1,1),(0,-1),(0,1),(1,-1),(1,0),(1,1)
                ];
                for (ahh, bmr) in bkr {
                    let nr = br as i32 + ahh;
                    let djs = bj as i32 + bmr;
                    if nr >= 0 && nr < 8 && djs >= 0 && djs < 8 {
                        let ab = Self::im(nr as usize, djs as usize);
                        let aaz = self.mn[ab];
                        if aaz == Y || aun(aaz) != cfs {
                            bev.push(ab);
                        }
                    }
                }
                
                if cfs && !self.mqm && im == 60 {
                    
                    if !self.mqo && self.mn[61] == Y && self.mn[62] == Y {
                        if !self.czb(60, false) && !self.czb(61, false) && !self.czb(62, false) {
                            bev.push(62);
                        }
                    }
                    
                    if !self.mqn && self.mn[59] == Y && self.mn[58] == Y && self.mn[57] == Y {
                        if !self.czb(60, false) && !self.czb(59, false) && !self.czb(58, false) {
                            bev.push(58);
                        }
                    }
                }
                if !cfs && !self.kdo && im == 4 {
                    
                    if !self.kdq && self.mn[5] == Y && self.mn[6] == Y {
                        if !self.czb(4, true) && !self.czb(5, true) && !self.czb(6, true) {
                            bev.push(6);
                        }
                    }
                    
                    if !self.kdp && self.mn[3] == Y && self.mn[2] == Y && self.mn[1] == Y {
                        if !self.czb(4, true) && !self.czb(3, true) && !self.czb(2, true) {
                            bev.push(2);
                        }
                    }
                }
            },
            _ => {},
        }
        bev
    }

    fn mgc(&self, im: usize, cfs: bool, dgh: &[(i32, i32)], bev: &mut Vec<usize>) {
        let br = Self::br(im) as i32;
        let bj = Self::bj(im) as i32;
        for &(ahh, bmr) in dgh {
            let mut m = br + ahh;
            let mut r = bj + bmr;
            while m >= 0 && m < 8 && r >= 0 && r < 8 {
                let ab = Self::im(m as usize, r as usize);
                let aaz = self.mn[ab];
                if aaz == Y {
                    bev.push(ab);
                } else {
                    if aun(aaz) != cfs {
                        bev.push(ab); 
                    }
                    break; 
                }
                m += ahh;
                r += bmr;
            }
        }
    }

    
    fn erb(&self, im: usize) -> Vec<usize> {
        let xe = self.mn[im];
        if xe == Y { return Vec::new(); }
        let cfs = aun(xe);
        let dkw = self.tec(im);
        let mut oit = Vec::new();
        for &cd in &dkw {
            
            let mut bdu = self.mn;
            
            if bkv(xe) == 1 && Some(cd) == self.bgx {
                if cfs { bdu[cd + 8] = Y; } else { bdu[cd - 8] = Y; }
            }
            
            if bkv(xe) == 6 {
                let fem = Self::bj(cd) as i32 - Self::bj(im) as i32;
                if fem == 2 { bdu[cd - 1] = bdu[cd + 1]; bdu[cd + 1] = Y; }
                if fem == -2 { bdu[cd + 1] = bdu[cd - 2]; bdu[cd - 2] = Y; }
            }
            bdu[cd] = xe;
            bdu[im] = Y;
            if !Self::ogc(&bdu, cfs) {
                oit.push(cd);
            }
        }
        oit
    }

    
    fn txr(&self, jws: bool) -> bool {
        Self::ogc(&self.mn, jws)
    }

    fn ogc(mn: &[i8; 64], jws: bool) -> bool {
        
        let ubl = if jws { BJD_ } else { ANN_ };
        let ubm = match mn.iter().qf(|&ai| ai == ubl) {
            Some(im) => im,
            None => return false,
        };
        Self::ogv(mn, ubm, !jws)
    }

    
    fn czb(&self, im: usize, kfq: bool) -> bool {
        Self::ogv(&self.mn, im, kfq)
    }

    fn ogv(mn: &[i8; 64], im: usize, kfq: bool) -> bool {
        for a in 0..64 {
            let ai = mn[a];
            if ai == Y { continue; }
            if aun(ai) != kfq { continue; }
            
            let br = a / 8;
            let bj = a % 8;
            let agd = im / 8;
            let asb = im % 8;

            match bkv(ai) {
                1 => { 
                    let te: i32 = if aun(ai) { -1 } else { 1 };
                    if agd as i32 == br as i32 + te && (asb as i32 - bj as i32).gp() == 1 {
                        return true;
                    }
                },
                2 => { 
                    let ahh = (agd as i32 - br as i32).gp();
                    let bmr = (asb as i32 - bj as i32).gp();
                    if (ahh == 2 && bmr == 1) || (ahh == 1 && bmr == 2) {
                        return true;
                    }
                },
                3 => { 
                    if Self::mwn(mn, a, im) { return true; }
                },
                4 => { 
                    if Self::mwo(mn, a, im) { return true; }
                },
                5 => { 
                    if Self::mwn(mn, a, im) || Self::mwo(mn, a, im) {
                        return true;
                    }
                },
                6 => { 
                    let ahh = (agd as i32 - br as i32).gp();
                    let bmr = (asb as i32 - bj as i32).gp();
                    if ahh <= 1 && bmr <= 1 && (ahh + bmr) > 0 {
                        return true;
                    }
                },
                _ => {},
            }
        }
        false
    }

    fn mwn(mn: &[i8; 64], from: usize, wh: usize) -> bool {
        let (xb, gc) = (from / 8, from % 8);
        let (agd, asb) = (wh / 8, wh % 8);
        let ahh = agd as i32 - xb as i32;
        let bmr = asb as i32 - gc as i32;
        if ahh.gp() != bmr.gp() || ahh == 0 { return false; }
        let adz = if ahh > 0 { 1 } else { -1 };
        let jt = if bmr > 0 { 1 } else { -1 };
        let mut m = xb as i32 + adz;
        let mut r = gc as i32 + jt;
        while (m, r) != (agd as i32, asb as i32) {
            if mn[(m * 8 + r) as usize] != Y { return false; }
            m += adz;
            r += jt;
        }
        true
    }

    fn mwo(mn: &[i8; 64], from: usize, wh: usize) -> bool {
        let (xb, gc) = (from / 8, from % 8);
        let (agd, asb) = (wh / 8, wh % 8);
        if xb != agd && gc != asb { return false; }
        if xb == agd {
            let (hh, gd) = if gc < asb { (gc, asb) } else { (asb, gc) };
            for r in (hh + 1)..gd {
                if mn[xb * 8 + r] != Y { return false; }
            }
        } else {
            let (hh, gd) = if xb < agd { (xb, agd) } else { (agd, xb) };
            for m in (hh + 1)..gd {
                if mn[m * 8 + gc] != Y { return false; }
            }
        }
        true
    }

    fn tmb(&self, xui: bool) -> bool {
        for im in 0..64 {
            let ai = self.mn[im];
            if ai == Y { continue; }
            if aun(ai) != xui { continue; }
            if !self.erb(im).is_empty() {
                return true;
            }
        }
        false
    }

    
    
    

    fn snq(&self) -> i32 {
        let mut ol: i32 = 0;
        for im in 0..64 {
            let ai = self.mn[im];
            if ai == Y { continue; }
            let ap = ovp(ai);
            if aun(ai) { ol += ap; } else { ol -= ap; }
            
            
            let m = Self::br(im);
            let r = Self::bj(im);
            let nbz = match (m, r) {
                (3, 3) | (3, 4) | (4, 3) | (4, 4) => 15,
                (2, 2) | (2, 5) | (5, 2) | (5, 5) => 8,
                _ => 0,
            };
            if aun(ai) { ol += nbz; } else { ol -= nbz; }
        }
        ol
    }

    fn lly(&mut self, eo: i32, mut dw: i32, mut dyu: i32, omh: bool) -> i32 {
        if eo == 0 {
            return self.snq();
        }

        let gso = omh; 
        let mut bdn;
        
        if omh {
            bdn = -100000;
            'outer_max: for im in 0..64 {
                let ai = self.mn[im];
                if ai == Y || !aun(ai) { continue; }
                let bev = self.erb(im);
                for &cd in &bev {
                    let ehz = self.mn;
                    let hyq = self.bgx;
                    
                    
                    if bkv(ai) == 1 && Some(cd) == self.bgx {
                        self.mn[cd + 8] = Y;
                    }
                    self.mn[cd] = ai;
                    self.mn[im] = Y;
                    
                    
                    self.bgx = None;
                    if bkv(ai) == 1 && (Self::br(cd) as i32 - Self::br(im) as i32).gp() == 2 {
                        self.bgx = Some(((im + cd) / 2) as usize);
                    }
                    
                    let ol = self.lly(eo - 1, dw, dyu, false);
                    
                    self.mn = ehz;
                    self.bgx = hyq;
                    
                    if ol > bdn { bdn = ol; }
                    if ol > dw { dw = ol; }
                    if dyu <= dw { break 'outer_max; }
                }
            }
        } else {
            bdn = 100000;
            'outer_min: for im in 0..64 {
                let ai = self.mn[im];
                if ai == Y || !din(ai) { continue; }
                let bev = self.erb(im);
                for &cd in &bev {
                    let ehz = self.mn;
                    let hyq = self.bgx;
                    
                    if bkv(ai) == 1 && Some(cd) == self.bgx {
                        self.mn[cd - 8] = Y;
                    }
                    self.mn[cd] = ai;
                    self.mn[im] = Y;
                    
                    self.bgx = None;
                    if bkv(ai) == 1 && (Self::br(cd) as i32 - Self::br(im) as i32).gp() == 2 {
                        self.bgx = Some(((im + cd) / 2) as usize);
                    }
                    
                    let ol = self.lly(eo - 1, dw, dyu, true);
                    
                    self.mn = ehz;
                    self.bgx = hyq;
                    
                    if ol < bdn { bdn = ol; }
                    if ol < dyu { dyu = ol; }
                    if dyu <= dw { break 'outer_min; }
                }
            }
        }
        bdn
    }

    fn qfz(&mut self) {
        let mut myr: Option<usize> = None;
        let mut myu: Option<usize> = None;
        let mut haf = 100000i32; 
        
        for im in 0..64 {
            let ai = self.mn[im];
            if ai == Y || !din(ai) { continue; }
            let bev = self.erb(im);
            for &cd in &bev {
                let ehz = self.mn;
                let hyq = self.bgx;
                
                if bkv(ai) == 1 && Some(cd) == self.bgx {
                    self.mn[cd - 8] = Y;
                }
                self.mn[cd] = ai;
                self.mn[im] = Y;
                
                self.bgx = None;
                if bkv(ai) == 1 && (Self::br(cd) as i32 - Self::br(im) as i32).gp() == 2 {
                    self.bgx = Some(((im + cd) / 2) as usize);
                }
                
                let ol = self.lly(self.fzz, -100000, 100000, true);
                
                self.mn = ehz;
                self.bgx = hyq;
                
                if ol < haf {
                    haf = ol;
                    myr = Some(im);
                    myu = Some(cd);
                }
            }
        }
        
        if let (Some(from), Some(wh)) = (myr, myu) {
            self.jet(from, wh);
        }
    }

    
    
    

    
    pub fn oli(&self) -> i32 {
        let mut pzi = 0i32;
        let mut mzi = 0i32;
        for im in 0..64 {
            let ai = self.mn[im];
            if ai == Y { continue; }
            let ap = ovp(ai);
            
            if bkv(ai) == 6 { continue; }
            if aun(ai) { pzi += ap; } else { mzi += ap; }
        }
        pzi - mzi
    }

    
    
    

    
    
    pub fn oai(&mut self, bj: i32, br: i32) -> bool {
        if bj < 0 || bj > 7 || br < 0 || br > 7 { return false; }
        let im = br as usize * 8 + bj as usize;
        
        if self.ib == GamePhase::Mw || self.ib == GamePhase::Up {
            return false;
        }
        if self.ib == GamePhase::Yg {
            return false; 
        }

        let xe = self.mn[im];
        
        match self.edj {
            InputMode::Uk => {
                if xe == Y { return false; }
                
                let hpa = (self.axi && aun(xe)) || (!self.axi && din(xe));
                if !hpa { return false; }
                
                
                self.blr = self.erb(im);
                if self.blr.is_empty() {
                    self.message = String::from("No legal moves for this piece");
                    return true;
                }
                self.na = Some(im);
                self.gi = im;
                self.edj = InputMode::Amj;
                let j = Self::gsv(im);
                self.message = format!("Move {} from {}", hvd(xe), j);
                
                
                self.dgo = Some(im);
                self.epb = Some(xe);
                return true;
            },
            InputMode::Amj => {
                
                if xe != Y {
                    let hpa = (self.axi && aun(xe)) || (!self.axi && din(xe));
                    if hpa {
                        self.blr = self.erb(im);
                        if !self.blr.is_empty() {
                            self.na = Some(im);
                            self.gi = im;
                            let j = Self::gsv(im);
                            self.message = format!("Move {} from {}", hvd(xe), j);
                            
                            self.dgo = Some(im);
                            self.epb = Some(xe);
                        }
                        return true;
                    }
                }
                
                
                if self.blr.contains(&im) {
                    if let Some(from) = self.na {
                        self.dgo = None;
                        self.epb = None;
                        self.jet(from, im);
                        return true;
                    }
                } else {
                    self.message = String::from("Invalid move");
                }
                return true;
            },
        }
    }

    
    pub fn lay(&mut self, bj: i32, br: i32) {
        if self.dgo.is_none() || self.epb.is_none() {
            return;
        }
        
        let from = self.dgo.unwrap();
        self.dgo = None;
        self.epb = None;
        
        if bj < 0 || bj > 7 || br < 0 || br > 7 {
            
            return;
        }
        
        let im = br as usize * 8 + bj as usize;
        
        
        if im == from { return; }
        
        
        if self.blr.contains(&im) {
            self.jet(from, im);
        }
        
    }

    
    pub fn pxc(&mut self, y: i32, x: i32) {
        self.kqq = y;
        self.kqr = x;
    }

    
    
    

    
    pub fn xiz(&mut self) {
        self.ezv = !self.ezv;
        if self.ezv {
            self.fyr = self.dww;
            self.fdl = self.dww;
            self.ezw = false;
            self.message = format!("Timer ON — {}min/side", self.dww / 60_000);
        } else {
            self.message = String::from("Timer OFF");
        }
    }

    
    pub fn rsr(&mut self) {
        let hvx: [u64; 6] = [60_000, 180_000, 300_000, 600_000, 900_000, 1_800_000];
        let rrw = hvx.iter().qf(|&ab| ab == self.dww).unwrap_or(3);
        let jgx = (rrw + 1) % hvx.len();
        self.dww = hvx[jgx];
        self.fyr = self.dww;
        self.fdl = self.dww;
        self.ezw = false;
        let bbz = self.dww / 60_000;
        let tv = (self.dww % 60_000) / 1000;
        if tv > 0 {
            self.message = format!("Timer: {}m{}s/side", bbz, tv);
        } else {
            self.message = format!("Timer: {}min/side", bbz);
        }
    }

    
    pub fn xgt(&mut self, oz: u64) {
        if !self.ezv || !self.ezw { return; }
        if self.ib == GamePhase::Mw || self.ib == GamePhase::Up { return; }
        
        if self.axi {
            self.fyr = self.fyr.ao(oz);
            if self.fyr == 0 {
                self.ib = GamePhase::Mw; 
                self.message = String::from("Time's up! Black wins!");
            }
        } else {
            self.fdl = self.fdl.ao(oz);
            if self.fdl == 0 {
                self.ib = GamePhase::Mw;
                self.message = String::from("Time's up! White wins!");
            }
        }
    }

    
    pub fn ivj(jn: u64) -> String {
        let jtw = jn / 1000;
        let bbz = jtw / 60;
        let tv = jtw % 60;
        format!("{:02}:{:02}", bbz, tv)
    }
}
