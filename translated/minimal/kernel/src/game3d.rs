















use alloc::vec::Vec;
use alloc::format;
use micromath::Wo;






const QC_: u8 = 0;
const BHC_: u8 = 1;
const CYC_: u8 = 2;
const CYB_: u8 = 3;
const CYA_: u8 = 4;
const AIT_: u8 = 5;
const QD_: u8 = 9;


const DFF_: u32 = 0xFF0A1A0A;  
const DFH_: u32 = 0xFF0F0F0F; 
const DFJ_: u32 = 0xCC0A0F0A;
const SA_: u32 = 0xFF00DD55;
const AAE_: u32 = 0xFF006633;
const BNI_: u32 = 0xFF00AA44;
const BNG_: u32 = 0xFF050A05;
const AOJ_: u32 = 0xFF00FF88;
const BNH_: u32 = 0xFFFFFF00;
const AOG_: u32 = 0xFF00FFAA;
const AAD_: u32 = 0xFF44FF44;
const AOD_: u32 = 0xFFFF4444;


const AA_: usize = 64;






struct WallTexture {
    hz: Vec<u32>,
}

impl WallTexture {
    
    fn qrx() -> Self {
        let mut hz = alloc::vec![0u32; AA_ * AA_];
        let hbg = 16;
        let hbh = 32;
        let hrw = 2;

        for c in 0..AA_ {
            for b in 0..AA_ {
                let br = c / hbg;
                let l = if br % 2 == 0 { 0 } else { hbh / 2 };
                let bx = (b + l) % hbh;
                let je = c % hbg;

                if je < hrw || bx < hrw {
                    
                    hz[c * AA_ + b] = 0xFF333333;
                } else {
                    
                    let bnq = ((b * 7 + c * 13) % 20) as u32;
                    let m = 140u32.akq(bnq).v(180);
                    let at = 60u32.akq(bnq / 2).v(80);
                    let o = 30u32.akq(bnq / 3).v(50);
                    hz[c * AA_ + b] = 0xFF000000 | (m << 16) | (at << 8) | o;
                }
            }
        }
        WallTexture { hz }
    }

    
    fn wuk() -> Self {
        let mut hz = alloc::vec![0u32; AA_ * AA_];
        for c in 0..AA_ {
            for b in 0..AA_ {
                
                let bnq = ((b * 31 + c * 17 + (b ^ c) * 7) % 40) as u32;
                let ar = 90u32;
                let p = ar.akq(bnq).v(140);
                hz[c * AA_ + b] = 0xFF000000 | (p << 16) | (p << 8) | p;
            }
        }
        
        for a in 0..AA_ {
            let cx = (a * 3 + 7) % AA_;
            let ae = a;
            if cx < AA_ && ae < AA_ {
                hz[ae * AA_ + cx] = 0xFF222222;
            }
        }
        WallTexture { hz }
    }

    
    fn unu() -> Self {
        let mut hz = alloc::vec![0u32; AA_ * AA_];
        for c in 0..AA_ {
            for b in 0..AA_ {
                
                let bti = c % 16;
                let txh = bti == 0 || bti == 15;
                let tyu = (b % 16 == 8) && (c % 16 == 8);

                if txh {
                    hz[c * AA_ + b] = 0xFF556655;
                } else if tyu {
                    hz[c * AA_ + b] = 0xFF889988;
                } else {
                    let bnq = ((b * 11 + c * 23) % 15) as u32;
                    let p = 50u32 + bnq;
                    hz[c * AA_ + b] = 0xFF000000 | (p / 2 << 16) | (p << 8) | (p / 2);
                }
            }
        }
        WallTexture { hz }
    }

    
    fn uko() -> Self {
        let mut hz = alloc::vec![0u32; AA_ * AA_];
        for c in 0..AA_ {
            for b in 0..AA_ {
                let bnq = ((b * 37 + c * 53 + (b * c) % 97) % 50) as u32;
                let rlq = ((b * 13) % AA_) < 4;
                let at = if rlq {
                    80u32 + bnq * 2
                } else {
                    10u32 + bnq / 2
                };
                hz[c * AA_ + b] = 0xFF000000 | ((at / 4) << 16) | (at.v(200) << 8) | (at / 6);
            }
        }
        WallTexture { hz }
    }

    
    fn sae() -> Self {
        let mut hz = alloc::vec![0u32; AA_ * AA_];
        for c in 0..AA_ {
            for b in 0..AA_ {
                let acu = b < 3 || b >= AA_ - 3 || c < 3 || c >= AA_ - 3;
                let tiz = b > 48 && b < 56 && c > 26 && c < 38;

                if acu {
                    hz[c * AA_ + b] = 0xFF008844;
                } else if tiz {
                    hz[c * AA_ + b] = 0xFF00FFAA;
                } else {
                    
                    let bnq = ((b * 3 + c * 7) % 20) as u32;
                    let m = 60u32 + bnq;
                    let at = 40u32 + bnq / 2;
                    let o = 20u32;
                    hz[c * AA_ + b] = 0xFF000000 | (m << 16) | (at << 8) | o;
                }
            }
        }
        WallTexture { hz }
    }

    #[inline]
    fn yr(&self, tm: usize, p: usize) -> u32 {
        self.hz[(p & (AA_ - 1)) * AA_ + (tm & (AA_ - 1))]
    }
}





#[derive(Clone, Copy, PartialEq)]
pub enum ItemType {
    Acc,
    Ki,   
    Acl,    
}

#[derive(Clone, Copy)]
pub struct Item {
    pub b: f32,
    pub c: f32,
    pub cft: ItemType,
    pub byu: bool,
}





#[derive(Clone, Copy, PartialEq)]
pub enum EnemyState {
    Cv,
    Aal,
    Apf,
    Ez,
}

#[derive(Clone, Copy)]
pub struct Nb {
    pub b: f32,
    pub c: f32,
    pub arh: i32,
    pub efe: i32,
    pub g: EnemyState,
    pub ddx: u32,
    pub cpt: i32,
    pub ig: f32,
    pub eyn: f32,
    pub emh: f32,
}





const DW_: usize = 16;
const DV_: usize = 16;


struct Pv {
    map: [[u8; DW_]; DV_],
    jqv: f32,
    jqw: f32,
    jqt: f32,
    pj: Vec<Item>,
    anf: Vec<Nb>,
}

fn nhe() -> Pv {
    #[rustfmt::chz]
    let map: [[u8; DW_]; DV_] = [
        [1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1],
        [1,0,0,0,1,0,0,0,0,0,0,0,0,0,0,1],
        [1,0,0,0,1,0,0,0,0,0,3,0,0,0,0,1],
        [1,0,0,0,5,0,0,0,0,0,3,0,0,0,0,1],
        [1,1,1,1,1,0,0,0,0,0,3,0,0,0,0,1],
        [1,0,0,0,0,0,0,0,0,0,0,0,0,0,0,1],
        [1,0,2,2,0,0,0,4,4,4,0,0,0,0,0,1],
        [1,0,2,2,0,0,0,4,0,4,0,0,0,0,0,1],
        [1,0,0,0,0,0,0,4,0,4,0,0,2,2,0,1],
        [1,0,0,0,0,0,0,0,0,0,0,0,2,0,0,1],
        [1,0,0,0,1,1,5,1,1,0,0,0,2,0,0,1],
        [1,0,0,0,1,0,0,0,1,0,0,0,0,0,0,1],
        [1,0,0,0,1,0,0,0,1,0,0,0,0,0,0,1],
        [1,0,0,0,1,0,0,9,1,0,0,0,3,3,3,1],
        [1,0,0,0,1,1,1,1,1,0,0,0,0,0,0,1],
        [1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1],
    ];

    let pj = alloc::vec![
        Item { b: 1.5, c: 1.5, cft: ItemType::Acc, byu: false },
        Item { b: 5.5, c: 2.5, cft: ItemType::Ki, byu: false },
        Item { b: 8.5, c: 8.5, cft: ItemType::Ki, byu: false },
        Item { b: 14.5, c: 1.5, cft: ItemType::Ki, byu: false },
        Item { b: 1.5, c: 13.5, cft: ItemType::Acl, byu: false },
        Item { b: 13.5, c: 9.5, cft: ItemType::Ki, byu: false },
    ];

    Pv {
        map,
        jqv: 2.5,
        jqw: 2.5,
        jqt: 0.0,
        pj,
        anf: alloc::vec![
            Nb { b: 8.5, c: 3.5, arh: 30, efe: 30, g: EnemyState::Cv, ddx: 0, cpt: 8, ig: 0.02, eyn: 6.0, emh: 1.5 },
            Nb { b: 3.5, c: 8.5, arh: 30, efe: 30, g: EnemyState::Cv, ddx: 0, cpt: 8, ig: 0.02, eyn: 6.0, emh: 1.5 },
            Nb { b: 13.5, c: 5.5, arh: 40, efe: 40, g: EnemyState::Cv, ddx: 0, cpt: 12, ig: 0.025, eyn: 8.0, emh: 1.5 },
        ],
    }
}

fn rqq() -> Pv {
    #[rustfmt::chz]
    let map: [[u8; DW_]; DV_] = [
        [2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2],
        [2,0,0,0,0,0,0,0,0,0,0,0,0,0,0,2],
        [2,0,3,3,3,0,0,0,0,0,3,3,3,0,0,2],
        [2,0,3,0,0,0,0,0,0,0,0,0,3,0,0,2],
        [2,0,3,0,4,4,4,0,4,4,4,0,3,0,0,2],
        [2,0,0,0,4,0,0,0,0,0,4,0,0,0,0,2],
        [2,0,0,0,4,0,1,1,1,0,4,0,0,0,0,2],
        [2,0,0,0,0,0,1,0,1,0,0,0,0,0,0,2],
        [2,0,0,0,0,0,1,9,1,0,0,0,0,0,0,2],
        [2,0,0,0,4,0,1,1,1,0,4,0,0,0,0,2],
        [2,0,0,0,4,0,0,0,0,0,4,0,0,0,0,2],
        [2,0,3,0,4,4,4,0,4,4,4,0,3,0,0,2],
        [2,0,3,0,0,0,0,0,0,0,0,0,3,0,0,2],
        [2,0,3,3,3,0,0,0,0,0,3,3,3,0,0,2],
        [2,0,0,0,0,0,0,0,0,0,0,0,0,0,0,2],
        [2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2],
    ];

    let pj = alloc::vec![
        Item { b: 1.5, c: 1.5, cft: ItemType::Ki, byu: false },
        Item { b: 14.5, c: 1.5, cft: ItemType::Ki, byu: false },
        Item { b: 1.5, c: 14.5, cft: ItemType::Ki, byu: false },
        Item { b: 14.5, c: 14.5, cft: ItemType::Ki, byu: false },
        Item { b: 7.5, c: 5.5, cft: ItemType::Acc, byu: false },
        Item { b: 7.5, c: 10.5, cft: ItemType::Acl, byu: false },
    ];

    Pv {
        map,
        jqv: 1.5,
        jqw: 1.5,
        jqt: 0.0,
        pj,
        anf: alloc::vec![
            Nb { b: 5.5, c: 5.5, arh: 40, efe: 40, g: EnemyState::Cv, ddx: 0, cpt: 10, ig: 0.025, eyn: 7.0, emh: 1.5 },
            Nb { b: 10.5, c: 5.5, arh: 40, efe: 40, g: EnemyState::Cv, ddx: 0, cpt: 10, ig: 0.025, eyn: 7.0, emh: 1.5 },
            Nb { b: 5.5, c: 10.5, arh: 40, efe: 40, g: EnemyState::Cv, ddx: 0, cpt: 10, ig: 0.025, eyn: 7.0, emh: 1.5 },
            Nb { b: 10.5, c: 10.5, arh: 50, efe: 50, g: EnemyState::Cv, ddx: 0, cpt: 15, ig: 0.03, eyn: 8.0, emh: 1.5 },
        ],
    }
}





pub struct Game3DState {
    
    pub brq: f32,
    pub brr: f32,
    pub bkw: f32,  
    pub ewt: i32,
    pub hvg: u32,
    pub hmp: bool,

    
    gnc: bool,
    gnb: bool,
    gtl: bool,
    gtm: bool,
    jul: bool,
    jum: bool,

    
    map: [[u8; DW_]; DV_],
    pj: Vec<Item>,
    anf: Vec<Nb>,
    gdz: u32,

    
    mkk: WallTexture,
    psq: WallTexture,
    psp: WallTexture,
    mkl: WallTexture,
    pso: WallTexture,

    
    frame: u32,
    pub hkv: bool,
    pub cev: bool,
    ghf: u32,          
    gph: u32,   
    message: Option<(alloc::string::String, u32)>, 

    
    iaf: u32,
    eir: u32,
    jwp: f32,
    lhm: u32,
    cob: Vec<f32>,  

    
    ajn: u32,

    
    jlf: Vec<f32>,
    jlg: Vec<f32>,
    lhx: usize,
    lhw: f32,
}

impl Game3DState {
    pub fn new() -> Self {
        let jy = nhe();
        Self {
            brq: jy.jqv,
            brr: jy.jqw,
            bkw: jy.jqt,
            ewt: 100,
            hvg: 0,
            hmp: false,

            gnc: false,
            gnb: false,
            gtl: false,
            gtm: false,
            jul: false,
            jum: false,

            map: jy.map,
            pj: jy.pj,
            anf: jy.anf,
            gdz: 1,

            mkk: WallTexture::qrx(),
            psq: WallTexture::wuk(),
            psp: WallTexture::unu(),
            mkl: WallTexture::uko(),
            pso: WallTexture::sae(),

            frame: 0,
            hkv: false,
            cev: false,
            ghf: 0,
            gph: 0,
            message: None,

            iaf: 0,
            eir: 0,
            jwp: 0.0,
            lhm: 0,
            cob: Vec::new(),

            ajn: 12345,

            jlf: Vec::new(),
            jlg: Vec::new(),
            lhx: 0,
            lhw: f32::Lx,
        }
    }

    
    fn vsy(&mut self, d: usize) {
        if d == self.lhx && self.bkw == self.lhw {
            return; 
        }
        let ckm = core::f32::consts::ACI_;
        self.jlf.cmg(d, 0.0);
        self.jlg.cmg(d, 0.0);
        for bj in 0..d {
            let vqm = (bj as f32 / d as f32 - 0.5) * 2.0;
            let ozq = self.bkw + vqm * (ckm / 2.0);
            self.jlf[bj] = ozq.cjt();
            self.jlg[bj] = ozq.ayq();
        }
        self.lhx = d;
        self.lhw = self.bkw;
    }

    fn lop(&mut self) -> u32 {
        self.ajn ^= self.ajn << 13;
        self.ajn ^= self.ajn >> 17;
        self.ajn ^= self.ajn << 5;
        self.ajn
    }

    
    fn uhd(&mut self, lim: u32) {
        let jy = match lim {
            2 => rqq(),
            _ => nhe(),
        };
        self.map = jy.map;
        self.pj = jy.pj;
        self.anf = jy.anf;
        self.brq = jy.jqv;
        self.brr = jy.jqw;
        self.bkw = jy.jqt;
        self.gdz = lim;
        self.hmp = false;
        self.message = Some((format!("Level {}", lim), 120));
    }

    
    pub fn vr(&mut self, bs: u8) {
        use crate::keyboard::{V_, U_, AH_, AI_};

        if self.hkv || self.cev {
            if bs == b' ' || bs == 0x0D {
                
                *self = Game3DState::new();
            }
            return;
        }

        match bs {
            
            b'w' | b'W' | V_ => self.gnc = true,
            b's' | b'S' | U_ => self.gnb = true,
            b'a' | b'A' => self.gtl = true,
            b'd' | b'D' => self.gtm = true,
            AH_ => self.jul = true,
            AI_ => self.jum = true,
            b'e' | b'E' => self.pwf(),
            b' ' => self.wmz(), 
            _ => {}
        }
    }

    
    pub fn avy(&mut self, bs: u8) {
        use crate::keyboard::{V_, U_, AH_, AI_};
        match bs {
            b'w' | b'W' | V_ => self.gnc = false,
            b's' | b'S' | U_ => self.gnb = false,
            b'a' | b'A' => self.gtl = false,
            b'd' | b'D' => self.gtm = false,
            AH_ => self.jul = false,
            AI_ => self.jum = false,
            _ => {}
        }
    }

    
    fn pwf(&mut self) {
        
        let khg = self.brq + self.bkw.cjt() * 1.2;
        let khh = self.brr + self.bkw.ayq() * 1.2;
        let hl = khg as usize;
        let ir = khh as usize;

        if hl < DW_ && ir < DV_ {
            match self.map[ir][hl] {
                AIT_ => {
                    if self.hmp {
                        self.map[ir][hl] = QC_;
                        self.message = Some((alloc::string::String::from("Door opened!"), 90));
                    } else {
                        self.message = Some((alloc::string::String::from("Need keycard!"), 90));
                    }
                }
                QD_ => {
                    if self.gdz < 2 {
                        self.gdz += 1;
                        self.uhd(self.gdz);
                        self.hvg += 500;
                    } else {
                        self.hkv = true;
                        self.message = Some((alloc::string::String::from("YOU WIN!"), 9999));
                    }
                }
                _ => {}
            }
        }
    }

    
    pub fn or(&mut self) {
        if self.hkv || self.cev {
            return;
        }

        self.frame += 1;

        
        let pwm = 0.06;
        if self.jul { self.bkw -= pwm; }
        if self.jum { self.bkw += pwm; }

        
        let euy = 0.06;
        let apn = self.bkw.cjt();
        let aql = self.bkw.ayq();
        let mut dx = 0.0f32;
        let mut bg = 0.0f32;

        if self.gnc { dx += apn * euy; bg += aql * euy; }
        if self.gnb { dx -= apn * euy; bg -= aql * euy; }
        if self.gtl { dx += aql * euy; bg -= apn * euy; }
        if self.gtm { dx -= aql * euy; bg += apn * euy; }

        
        let adf = 0.25;
        let evh = self.brq + dx;
        let bhn = self.brr + bg;

        
        if !self.lgq(evh + adf * dx.wof(), self.brr) {
            self.brq = evh;
        }
        
        if !self.lgq(self.brq, bhn + adf * bg.wof()) {
            self.brr = bhn;
        }

        
        self.qzl();

        
        self.xou();

        
        if self.iaf > 0 { self.iaf -= 1; }
        if self.eir > 0 { self.eir -= 1; }

        
        if self.gnc || self.gnb || self.gtl || self.gtm {
            self.jwp += 0.12;
        } else {
            self.jwp *= 0.9; 
        }

        
        if self.ewt <= 0 {
            self.cev = true;
            self.message = Some((alloc::string::String::from("YOU DIED"), 9999));
        }

        
        let hl = self.brq as usize;
        let ir = self.brr as usize;
        if hl < DW_ && ir < DV_ && self.map[ir][hl] == QD_ {
            self.pwf();
        }

        
        if self.ghf > 0 { self.ghf -= 1; }
        if self.gph > 0 { self.gph -= 1; }
        if let Some((_, ref mut vj)) = self.message {
            if *vj > 0 { *vj -= 1; }
            else { self.message = None; }
        }
    }

    #[inline]
    fn lgq(&self, b: f32, c: f32) -> bool {
        let hl = b as usize;
        let ir = c as usize;
        if hl >= DW_ || ir >= DV_ { return true; }
        self.map[ir][hl] != QC_
    }

    fn qzl(&mut self) {
        for item in &mut self.pj {
            if item.byu { continue; }
            let dx = item.b - self.brq;
            let bg = item.c - self.brr;
            if dx * dx + bg * bg < 0.5 {
                item.byu = true;
                self.gph = 15;
                match item.cft {
                    ItemType::Acc => {
                        self.ewt = (self.ewt + 25).v(100);
                        self.message = Some((alloc::string::String::from("+25 HP"), 60));
                    }
                    ItemType::Ki => {
                        self.hvg += 100;
                        self.message = Some((alloc::string::String::from("+100 pts"), 60));
                    }
                    ItemType::Acl => {
                        self.hmp = true;
                        self.message = Some((alloc::string::String::from("KEYCARD acquired!"), 90));
                    }
                }
            }
        }
    }

    
    fn wmz(&mut self) {
        if self.iaf > 0 { return; }
        self.iaf = 15; 
        self.eir = 4;    

        
        let fsb = self.bkw.cjt();
        let fsc = self.bkw.ayq();
        let mut kb = self.brq;
        let mut ix = self.brr;
        let gu = 0.1;

        for _ in 0..120 { 
            kb += fsb * gu;
            ix += fsc * gu;

            
            let hl = kb as usize;
            let ir = ix as usize;
            if hl >= DW_ || ir >= DV_ { break; }
            if self.map[ir][hl] != QC_ { break; }

            
            for bjx in &mut self.anf {
                if bjx.g == EnemyState::Ez { continue; }
                let edx = bjx.b - kb;
                let npf = bjx.c - ix;
                if edx * edx + npf * npf < 0.3 {
                    
                    bjx.arh -= 25;
                    if bjx.arh <= 0 {
                        bjx.g = EnemyState::Ez;
                        self.lhm += 1;
                        self.hvg += 200;
                        self.message = Some((alloc::string::String::from("Enemy eliminated!"), 60));
                    } else {
                        bjx.g = EnemyState::Aal; 
                        self.message = Some((alloc::string::String::from("Hit!"), 30));
                    }
                    return;
                }
            }
        }
    }

    
    fn xou(&mut self) {
        let y = self.brq;
        let x = self.brr;

        for a in 0..self.anf.len() {
            if self.anf[a].g == EnemyState::Ez { continue; }

            let bqp = self.anf[a].b;
            let ahm = self.anf[a].c;
            let dx = y - bqp;
            let bg = x - ahm;
            let la = (dx * dx + bg * bg).ibi();

            
            if la < self.anf[a].emh {
                self.anf[a].g = EnemyState::Apf;
            } else if la < self.anf[a].eyn {
                self.anf[a].g = EnemyState::Aal;
            } else if self.anf[a].g == EnemyState::Aal {
                
                self.anf[a].g = EnemyState::Cv;
            }

            match self.anf[a].g {
                EnemyState::Aal => {
                    
                    if la > 0.1 {
                        let vt = dx / la * self.anf[a].ig;
                        let ahr = bg / la * self.anf[a].ig;
                        let opq = self.anf[a].b + vt;
                        let opr = self.anf[a].c + ahr;
                        
                        if !self.lgq(opq, opr) {
                            self.anf[a].b = opq;
                            self.anf[a].c = opr;
                        }
                    }
                }
                EnemyState::Apf => {
                    if self.anf[a].ddx == 0 {
                        
                        self.ewt -= self.anf[a].cpt;
                        self.ghf = 10;
                        self.anf[a].ddx = 45; 
                    }
                }
                _ => {}
            }

            
            if self.anf[a].ddx > 0 {
                self.anf[a].ddx -= 1;
            }
        }
    }

    
    
    

    
    pub fn tj(&mut self, k: &mut [u32], d: usize, i: usize) {
        if d < 80 || i < 60 { return; }

        let tqo = 40; 
        let azb = i.ao(tqo);

        
        #[cfg(target_arch = "x86_64")]
        unsafe {
            crate::graphics::simd::bed(k.mw(), d * i, 0xFF000000);
        }
        #[cfg(not(target_arch = "x86_64"))]
        {
            k[..d * i].vi(0xFF000000);
        }

        
        self.cob.clear();
        self.cob.cmg(d, f32::O);

        
        self.vsy(d);

        
        self.vvt(k, d, azb);

        
        self.vww(k, d, azb);

        
        self.vvr(k, d, azb);

        
        self.vvz(k, d, azb);

        
        self.vvn(k, d, azb);

        
        self.vwx(k, d, azb);

        
        self.vwc(k, d, i);

        
        self.lzd(k, d, i, azb);

        
        if self.ghf > 0 {
            let dw = (self.ghf as u32 * 8).v(80);
            for a in 0..d * azb {
                let m = ((k[a] >> 16) & 0xFF).akq(dw);
                k[a] = (k[a] & 0xFF00FFFF) | (m.v(255) << 16);
            }
        }

        
        if self.gph > 0 {
            let dw = (self.gph as u32 * 5).v(40);
            for a in 0..d * azb {
                let at = ((k[a] >> 8) & 0xFF).akq(dw);
                k[a] = (k[a] & 0xFFFF00FF) | (at.v(255) << 8);
            }
        }

        
        if self.hkv || self.cev {
            
            for a in 0..d * i {
                let m = ((k[a] >> 16) & 0xFF) / 3;
                let at = ((k[a] >> 8) & 0xFF) / 3;
                let o = (k[a] & 0xFF) / 3;
                k[a] = 0xFF000000 | (m << 16) | (at << 8) | o;
            }
        }

        
        if let Some((ref fr, _)) = self.message {
            self.np(k, d, azb / 4, fr, SA_);
        }
    }

    fn vvt(&self, k: &mut [u32], d: usize, i: usize) {
        let iv = i / 2;

        for c in 0..i {
            let s = if c < iv {
                
                let ab = c as u32 * 20 / iv as u32;
                let at = 8u32 + ab;
                0xFF000000 | ((at / 3) << 16) | (at << 8) | (at / 4)
            } else {
                
                let la = (c - iv) as u32;
                let lkv = iv as u32;
                let ab = la * 25 / lkv.am(1);
                let ar = 6u32 + ab;
                0xFF000000 | ((ar / 2) << 16) | ((ar / 2) << 8) | (ar / 2)
            };
            
            #[cfg(target_arch = "x86_64")]
            unsafe {
                crate::graphics::simd::bed(
                    k.mw().add(c * d),
                    d,
                    s,
                );
            }
            #[cfg(not(target_arch = "x86_64"))]
            {
                for b in 0..d {
                    k[c * d + b] = s;
                }
            }
        }
    }

    fn vww(&mut self, k: &mut [u32], d: usize, i: usize) {
        let ckm = core::f32::consts::ACI_; 
        let wp = i as f32 / 2.0;

        for bj in 0..d {
            
            let fsb = self.jlf[bj];
            let fsc = self.jlg[bj];

            
            let mut fno = self.brq as i32;
            let mut fnp = self.brr as i32;

            let kow = if fsb.gp() < 1e-8 { 1e8 } else { (1.0 / fsb).gp() };
            let kox = if fsc.gp() < 1e-8 { 1e8 } else { (1.0 / fsc).gp() };

            let (dcj, mut pkq) = if fsb < 0.0 {
                (-1i32, (self.brq - fno as f32) * kow)
            } else {
                (1i32, ((fno + 1) as f32 - self.brq) * kow)
            };

            let (ejd, mut pkr) = if fsc < 0.0 {
                (-1i32, (self.brr - fnp as f32) * kox)
            } else {
                (1i32, ((fnp + 1) as f32 - self.brr) * kox)
            };

            
            let mut agp = false;
            let mut gso = 0; 
            let mut mqg = BHC_;

            for _ in 0..64 {
                if pkq < pkr {
                    pkq += kow;
                    fno += dcj;
                    gso = 0;
                } else {
                    pkr += kox;
                    fnp += ejd;
                    gso = 1;
                }

                if fno < 0 || fnp < 0 || fno >= DW_ as i32 || fnp >= DV_ as i32 {
                    break;
                }

                let ccd = self.map[fnp as usize][fno as usize];
                if ccd != QC_ {
                    agp = true;
                    mqg = ccd;
                    break;
                }
            }

            if !agp { continue; }

            
            let gpg = if gso == 0 {
                (fno as f32 - self.brq + (1.0 - dcj as f32) / 2.0) / fsb
            } else {
                (fnp as f32 - self.brr + (1.0 - ejd as f32) / 2.0) / fsc
            };

            if gpg <= 0.0 { continue; }

            
            let jwi = (i as f32 / gpg).v(i as f32 * 4.0);
            let sfr = ((wp - jwi / 2.0) as i32).am(0) as usize;
            let scs = ((wp + jwi / 2.0) as i32).v(i as i32 - 1) as usize;

            
            let cnz = if gso == 0 {
                self.brr + gpg * fsc
            } else {
                self.brq + gpg * fsb
            };
            let cnz = cnz - cnz.hjw();  
            let xfn = (cnz * AA_ as f32) as usize;

            
            let dco = match mqg {
                BHC_ => &self.mkk,
                CYC_ => &self.psq,
                CYB_ => &self.psp,
                CYA_ => &self.mkl,
                AIT_ => &self.pso,
                QD_ => &self.mkl,
                _ => &self.mkk,
            };

            
            let tvx = AA_ as f32 / jwi;
            for c in sfr..=scs {
                let xfo = ((c as f32 - (wp - jwi / 2.0)) * tvx) as usize;
                let mut il = dco.yr(xfn, xfo);

                
                let cer = (1.0 - (gpg / 12.0).v(1.0)).am(0.15);
                let m = (((il >> 16) & 0xFF) as f32 * cer) as u32;
                let at = (((il >> 8) & 0xFF) as f32 * cer) as u32;
                let o = ((il & 0xFF) as f32 * cer) as u32;
                il = 0xFF000000 | (m << 16) | (at << 8) | o;

                
                if gso == 1 {
                    let m = ((il >> 16) & 0xFF) * 3 / 4;
                    let at = ((il >> 8) & 0xFF) * 3 / 4;
                    let o = (il & 0xFF) * 3 / 4;
                    il = 0xFF000000 | (m << 16) | (at << 8) | o;
                }

                
                if mqg == QD_ {
                    let xg = ((self.frame as f32 * 0.1).ayq() * 30.0 + 30.0) as u32;
                    let ght = ((il >> 8) & 0xFF).akq(xg).v(255);
                    il = (il & 0xFFFF00FF) | (ght << 8);
                }

                k[c * d + bj] = il;
            }

            
            if bj < self.cob.len() {
                self.cob[bj] = gpg;
            }
        }
    }

    fn vvz(&self, k: &mut [u32], d: usize, i: usize) {
        let wp = i as f32 / 2.0;

        for item in &self.pj {
            if item.byu { continue; }

            
            let dx = item.b - self.brq;
            let bg = item.c - self.brr;

            
            let apn = self.bkw.cjt();
            let aql = self.bkw.ayq();
            let gx = dx * apn + bg * aql;
            let ty = -dx * aql + bg * apn;

            
            if ty < 0.2 { continue; }

            
            let ckm = core::f32::consts::ACI_;
            let xu = (0.5 + gx / (ty * (ckm / 2.0).mjs() * 2.0)) * d as f32;
            let jrd = (i as f32 / ty * 0.3) as i32;

            if jrd < 2 { continue; }

            let cr = xu as i32 - jrd / 2;
            let cq = (wp as i32) - jrd / 2;

            let s = match item.cft {
                ItemType::Acc => AAD_,
                ItemType::Ki => AOG_,
                ItemType::Acl => 0xFFFFAA00,
            };

            
            let iv = jrd / 2;
            for hhl in -iv..=iv {
                let br = cq + iv + hhl;
                if br < 0 || br >= i as i32 { continue; }
                let cml = iv - hhl.gp();
                for isc in -cml..=cml {
                    let cx = cr + iv + isc;
                    if cx < 0 || cx >= d as i32 { continue; }

                    
                    let xg = ((self.frame as f32 * 0.15 + item.b * 3.0).ayq() * 0.3 + 0.7) as f32;
                    let m = (((s >> 16) & 0xFF) as f32 * xg) as u32;
                    let at = (((s >> 8) & 0xFF) as f32 * xg) as u32;
                    let o = ((s & 0xFF) as f32 * xg) as u32;

                    k[br as usize * d + cx as usize] = 0xFF000000 | (m << 16) | (at << 8) | o;
                }
            }
        }
    }

    fn vvr(&self, k: &mut [u32], d: usize, i: usize) {
        let wp = i as f32 / 2.0;
        let ckm = core::f32::consts::ACI_;

        for bjx in &self.anf {
            if bjx.g == EnemyState::Ez { continue; }

            
            let dx = bjx.b - self.brq;
            let bg = bjx.c - self.brr;

            
            let apn = self.bkw.cjt();
            let aql = self.bkw.ayq();
            let gx = dx * apn + bg * aql;
            let ty = -dx * aql + bg * apn;

            
            if ty < 0.3 { continue; }

            
            let xu = (0.5 + gx / (ty * (ckm / 2.0).mjs() * 2.0)) * d as f32;
            let cby = (i as f32 / ty * 0.6) as i32;
            let fvf = (cby as f32 * 0.5) as i32;

            if cby < 2 { continue; }

            let cr = xu as i32 - fvf / 2;
            let cq = wp as i32 - cby / 2;

            
            let agg: u32 = match bjx.g {
                EnemyState::Cv => 0xFFCC2222,      
                EnemyState::Aal => 0xFFFF3333,   
                EnemyState::Apf => 0xFFFF6600,  
                EnemyState::Ez => continue,
            };

            
            let cer = (1.0 - (ty / 12.0).v(1.0)).am(0.2);

            
            for hhl in 0..cby {
                let br = cq + hhl;
                if br < 0 || br >= i as i32 { continue; }

                
                let ab = hhl as f32 / cby as f32; 
                let mqp = if ab < 0.2 {
                    
                    0.4 + ab
                } else if ab < 0.7 {
                    
                    0.7
                } else {
                    
                    0.5
                };
                let pef = (fvf as f32 * mqp * 0.5) as i32;

                for isc in -pef..=pef {
                    let cx = cr + fvf / 2 + isc;
                    if cx < 0 || cx >= d as i32 { continue; }

                    
                    if (cx as usize) < self.cob.len() && ty >= self.cob[cx as usize] {
                        continue; 
                    }

                    
                    if ab > 0.75 && isc.gp() < 2 {
                        continue; 
                    }

                    
                    let m = (((agg >> 16) & 0xFF) as f32 * cer) as u32;
                    let at = (((agg >> 8) & 0xFF) as f32 * cer) as u32;
                    let o = ((agg & 0xFF) as f32 * cer) as u32;
                    let il = 0xFF000000 | (m << 16) | (at << 8) | o;

                    k[br as usize * d + cx as usize] = il;
                }
            }

            
            if cby > 8 {
                let ckh = cq + cby / 8;
                let nso = fvf / 6;
                for &snz in &[-nso, nso] {
                    let fif = cr + fvf / 2 + snz;
                    if fif >= 0 && fif < d as i32 && ckh >= 0 && ckh < i as i32 {
                        if (fif as usize) < self.cob.len() && ty < self.cob[fif as usize] {
                            k[ckh as usize * d + fif as usize] = 0xFFFFFF00; 
                        }
                    }
                }
            }

            
            if bjx.arh < bjx.efe && cby > 10 {
                let pl = cq - 4;
                if pl >= 0 && pl < i as i32 {
                    let lo = fvf.v(20) as usize;
                    let akd = (bjx.arh as f32 / bjx.efe as f32 * lo as f32) as usize;
                    let ikt = (cr + fvf / 2 - lo as i32 / 2).am(0) as usize;
                    for bx in 0..lo {
                        let y = ikt + bx;
                        if y >= d { break; }
                        if (y as usize) < self.cob.len() && ty >= self.cob[y] {
                            continue;
                        }
                        let s = if bx < akd { 0xFFFF0000 } else { 0xFF440000 };
                        k[pl as usize * d + y] = s;
                    }
                }
            }
        }
    }

    fn vvn(&self, k: &mut [u32], d: usize, i: usize) {
        let cx = d / 2;
        let ae = i / 2;
        let aw = 4;
        let s = if self.eir > 0 { 0xFFFFFF00 } else { 0xAA00FF88 };

        
        for b in (cx.ao(aw))..=(cx + aw).v(d - 1) {
            if b != cx { 
                k[ae * d + b] = s;
            }
        }
        
        for c in (ae.ao(aw))..=(ae + aw).v(i - 1) {
            if c != ae {
                k[c * d + cx] = s;
            }
        }
        
        k[ae * d + cx] = if self.eir > 0 { 0xFFFFFFFF } else { 0xFF00FF88 };
    }

    fn vwx(&self, k: &mut [u32], d: usize, i: usize) {
        
        let qqq = if self.gnc || self.gnb || self.gtl || self.gtm {
            (self.jwp.ayq() * 3.0) as i32
        } else {
            0
        };

        let fde = (d as i32 / 2 + 30) as usize;
        let dec = (i as i32 - 20 + qqq) as usize;

        
        let tih = if self.eir > 0 { 0xFFFFDD44 } else { 0xFF666666 };
        let qna = if self.eir > 0 { 0xFFFFFF88 } else { 0xFF444444 };

        
        for c in 0..3usize {
            for b in 0..12usize {
                let y = fde + b;
                let x = dec.ao(8) + c;
                if y < d && x < i {
                    k[x * d + y] = qna;
                }
            }
        }
        
        for c in 0..8usize {
            for b in 0..6usize {
                let y = fde + 3 + b;
                let x = dec.ao(5) + c;
                if y < d && x < i {
                    k[x * d + y] = tih;
                }
            }
        }

        
        if self.eir > 0 {
            let suq = fde + 12;
            let sur = dec.ao(7);
            for bg in 0..5usize {
                for dx in 0..4usize {
                    let y = suq + dx;
                    let x = sur.ao(1) + bg;
                    if y < d && x < i {
                        k[x * d + y] = 0xFFFFFF88;
                    }
                }
            }
        }
    }

    fn vwc(&self, k: &mut [u32], d: usize, i: usize) {
        let cell = 5;
        let ola = DW_ * cell;
        let ujq = DV_ * cell;
        let dtw = d - ola - 8;
        let dtx = 8;

        
        for c in 0..ujq + 4 {
            for b in 0..ola + 4 {
                let y = dtw - 2 + b;
                let x = dtx - 2 + c;
                if y < d && x < i {
                    k[x * d + y] = 0xAA000000;
                }
            }
        }

        
        for ir in 0..DV_ {
            for hl in 0..DW_ {
                let s = match self.map[ir][hl] {
                    QC_ => BNG_,
                    QD_ => BNH_,
                    AIT_ => 0xFF884400,
                    _ => BNI_,
                };
                for bg in 0..cell {
                    for dx in 0..cell {
                        let y = dtw + hl * cell + dx;
                        let x = dtx + ir * cell + bg;
                        if y < d && x < i {
                            k[x * d + y] = s;
                        }
                    }
                }
            }
        }

        
        let y = dtw + (self.brq * cell as f32) as usize;
        let x = dtx + (self.brr * cell as f32) as usize;
        for bg in 0..3usize {
            for dx in 0..3usize {
                let b = y + dx;
                let c = x + bg;
                if b < d && c < i {
                    k[c * d + b] = AOJ_;
                }
            }
        }

        
        let nlo = 6.0;
        let bqp = y as f32 + self.bkw.cjt() * nlo;
        let ahm = x as f32 + self.bkw.ayq() * nlo;
        let au = 8;
        for a in 0..au {
            let ab = a as f32 / au as f32;
            let mj = (y as f32 + (bqp - y as f32) * ab) as usize;
            let ct = (x as f32 + (ahm - x as f32) * ab) as usize;
            if mj < d && ct < i {
                k[ct * d + mj] = AOJ_;
            }
        }

        
        for item in &self.pj {
            if item.byu { continue; }
            let fg = dtw + (item.b * cell as f32) as usize;
            let og = dtx + (item.c * cell as f32) as usize;
            let ize = match item.cft {
                ItemType::Acc => AAD_,
                ItemType::Ki => AOG_,
                ItemType::Acl => 0xFFFFAA00,
            };
            if fg > 0 && fg + 1 < d && og > 0 && og + 1 < i {
                k[og * d + fg] = ize;
                k[og * d + fg + 1] = ize;
                k[(og + 1) * d + fg] = ize;
                k[(og + 1) * d + fg + 1] = ize;
            }
        }

        
        for bjx in &self.anf {
            if bjx.g == EnemyState::Ez { continue; }
            let bqp = dtw + (bjx.b * cell as f32) as usize;
            let ahm = dtx + (bjx.c * cell as f32) as usize;
            let ec = 0xFFFF2222;
            if bqp > 0 && bqp + 1 < d && ahm > 0 && ahm + 1 < i {
                k[ahm * d + bqp] = ec;
                k[ahm * d + bqp + 1] = ec;
                k[(ahm + 1) * d + bqp] = ec;
                k[(ahm + 1) * d + bqp + 1] = ec;
            }
        }
    }

    fn lzd(&self, k: &mut [u32], d: usize, i: usize, azb: usize) {
        
        for c in azb..i {
            for b in 0..d {
                k[c * d + b] = 0xFF0A120A;
            }
        }

        
        for b in 0..d {
            if azb < i {
                k[azb * d + b] = SA_;
            }
        }

        let ecy = azb + 4;
        let ezq = 1;

        
        self.ri(k, d, i, 8, ecy, "HP", AAE_);
        let ajx = 28;
        let lo = 80;
        let tn = 10;
        
        for c in 0..tn {
            for b in 0..lo {
                let y = ajx + b;
                let x = ecy + 4 + c;
                if y < d && x < i {
                    k[x * d + y] = 0xFF1A1A1A;
                }
            }
        }
        
        let akd = (self.ewt as usize * lo / 100).v(lo);
        let tqf = if self.ewt > 60 { AAD_ }
                       else if self.ewt > 30 { 0xFFAAAA00 }
                       else { AOD_ };
        for c in 0..tn {
            for b in 0..akd {
                let y = ajx + b;
                let x = ecy + 4 + c;
                if y < d && x < i {
                    k[x * d + y] = tqf;
                }
            }
        }

        
        let hzb = format!("SCORE:{}", self.hvg);
        self.ri(k, d, i, 120, ecy, &hzb, SA_);

        
        let ueh = format!("LVL:{}", self.gdz);
        self.ri(k, d, i, 240, ecy, &ueh, AAE_);

        
        if self.hmp {
            self.ri(k, d, i, 310, ecy, "[KEY]", 0xFFFFAA00);
        }

        
        let ubk = format!("KILLS:{}", self.lhm);
        self.ri(k, d, i, 370, ecy, &ubk, AOD_);

        
        let rmv = d - 60;
        let dgh = ["N", "E", "S", "W"];
        let qik = (self.bkw + core::f32::consts::Eu * 2.0) % (core::f32::consts::Eu * 2.0);
        let rxp = ((qik + core::f32::consts::DMF_) / core::f32::consts::DME_) as usize % 4;
        self.ri(k, d, i, rmv, ecy, dgh[rxp], SA_);

        
        self.ri(k, d, i, 8, ecy + 18, "WASD:Move Arrows:Turn E:Use Space:Shoot", AAE_);
    }

    
    
    

    fn ri(&self, k: &mut [u32], d: usize, i: usize, b: usize, c: usize, text: &str, s: u32) {
        for (a, bm) in text.bw().cf() {
            let cx = b + a * 7;
            if cx + 6 >= d { break; }
            self.ahi(k, d, i, cx, c, bm, s);
        }
    }

    fn np(&self, k: &mut [u32], d: usize, c: usize, text: &str, s: u32) {
        let bda = text.len() * 7;
        let b = if bda < d { (d - bda) / 2 } else { 0 };
        self.ri(k, d, d * (d / d), b, c, text, s); 
        
        for (a, bm) in text.bw().cf() {
            let cx = b + a * 7;
            if cx + 6 >= d { break; }
            
            let csl = k.len() / d;
            self.ahi(k, d, csl, cx, c, bm, s);
        }
    }

    fn ahi(&self, k: &mut [u32], d: usize, i: usize, b: usize, c: usize, bm: char, s: u32) {
        
        let cdf = match bm {
            'A' => [0b01110, 0b10001, 0b10001, 0b11111, 0b10001, 0b10001, 0b10001],
            'B' => [0b11110, 0b10001, 0b10001, 0b11110, 0b10001, 0b10001, 0b11110],
            'C' => [0b01110, 0b10001, 0b10000, 0b10000, 0b10000, 0b10001, 0b01110],
            'D' => [0b11110, 0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b11110],
            'E' => [0b11111, 0b10000, 0b10000, 0b11110, 0b10000, 0b10000, 0b11111],
            'F' => [0b11111, 0b10000, 0b10000, 0b11110, 0b10000, 0b10000, 0b10000],
            'G' => [0b01110, 0b10001, 0b10000, 0b10111, 0b10001, 0b10001, 0b01110],
            'H' => [0b10001, 0b10001, 0b10001, 0b11111, 0b10001, 0b10001, 0b10001],
            'I' => [0b01110, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100, 0b01110],
            'J' => [0b00111, 0b00010, 0b00010, 0b00010, 0b10010, 0b10010, 0b01100],
            'K' => [0b10001, 0b10010, 0b10100, 0b11000, 0b10100, 0b10010, 0b10001],
            'L' => [0b10000, 0b10000, 0b10000, 0b10000, 0b10000, 0b10000, 0b11111],
            'M' => [0b10001, 0b11011, 0b10101, 0b10101, 0b10001, 0b10001, 0b10001],
            'N' => [0b10001, 0b11001, 0b10101, 0b10011, 0b10001, 0b10001, 0b10001],
            'O' => [0b01110, 0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b01110],
            'P' => [0b11110, 0b10001, 0b10001, 0b11110, 0b10000, 0b10000, 0b10000],
            'Q' => [0b01110, 0b10001, 0b10001, 0b10001, 0b10101, 0b10010, 0b01101],
            'R' => [0b11110, 0b10001, 0b10001, 0b11110, 0b10100, 0b10010, 0b10001],
            'S' => [0b01110, 0b10001, 0b10000, 0b01110, 0b00001, 0b10001, 0b01110],
            'T' => [0b11111, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100],
            'U' => [0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b01110],
            'V' => [0b10001, 0b10001, 0b10001, 0b10001, 0b01010, 0b01010, 0b00100],
            'W' => [0b10001, 0b10001, 0b10001, 0b10101, 0b10101, 0b11011, 0b10001],
            'X' => [0b10001, 0b10001, 0b01010, 0b00100, 0b01010, 0b10001, 0b10001],
            'Y' => [0b10001, 0b10001, 0b01010, 0b00100, 0b00100, 0b00100, 0b00100],
            'Z' => [0b11111, 0b00001, 0b00010, 0b00100, 0b01000, 0b10000, 0b11111],
            '0' => [0b01110, 0b10011, 0b10101, 0b10101, 0b11001, 0b10001, 0b01110],
            '1' => [0b00100, 0b01100, 0b00100, 0b00100, 0b00100, 0b00100, 0b01110],
            '2' => [0b01110, 0b10001, 0b00001, 0b00110, 0b01000, 0b10000, 0b11111],
            '3' => [0b01110, 0b10001, 0b00001, 0b00110, 0b00001, 0b10001, 0b01110],
            '4' => [0b00010, 0b00110, 0b01010, 0b10010, 0b11111, 0b00010, 0b00010],
            '5' => [0b11111, 0b10000, 0b11110, 0b00001, 0b00001, 0b10001, 0b01110],
            '6' => [0b01110, 0b10000, 0b10000, 0b11110, 0b10001, 0b10001, 0b01110],
            '7' => [0b11111, 0b00001, 0b00010, 0b00100, 0b01000, 0b01000, 0b01000],
            '8' => [0b01110, 0b10001, 0b10001, 0b01110, 0b10001, 0b10001, 0b01110],
            '9' => [0b01110, 0b10001, 0b10001, 0b01111, 0b00001, 0b00001, 0b01110],
            ':' => [0b00000, 0b00100, 0b00100, 0b00000, 0b00100, 0b00100, 0b00000],
            '+' => [0b00000, 0b00100, 0b00100, 0b11111, 0b00100, 0b00100, 0b00000],
            '-' => [0b00000, 0b00000, 0b00000, 0b11111, 0b00000, 0b00000, 0b00000],
            '!' => [0b00100, 0b00100, 0b00100, 0b00100, 0b00100, 0b00000, 0b00100],
            '.' => [0b00000, 0b00000, 0b00000, 0b00000, 0b00000, 0b00000, 0b00100],
            '[' => [0b01100, 0b01000, 0b01000, 0b01000, 0b01000, 0b01000, 0b01100],
            ']' => [0b00110, 0b00010, 0b00010, 0b00010, 0b00010, 0b00010, 0b00110],
            ' ' => [0b00000, 0b00000, 0b00000, 0b00000, 0b00000, 0b00000, 0b00000],
            _   => [0b11111, 0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b11111],
        };

        for br in 0..7 {
            for bj in 0..5 {
                if cdf[br] & (1 << (4 - bj)) != 0 {
                    let y = b + bj;
                    let x = c + br;
                    if y < d && x < i {
                        k[x * d + y] = s;
                    }
                }
            }
        }
    }
}
