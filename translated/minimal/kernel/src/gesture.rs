












use crate::touch::{self, Zd, TouchPhase, TouchPoint, GU_};






const CXJ_: u64 = 200_000;


const CXI_: i32 = 10;


const CEV_: u64 = 500_000;


const CEU_: i32 = 15;


const LD_: i32 = 50;


const CUT_: i32 = 200;


const SY_: i32 = 30;


const CJQ_: i32 = 15;


const BFB_: i32 = 8;


const BSD_: u64 = 300_000;


const BSC_: i32 = 30;






#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SwipeDirection {
    Ap,
    Ca,
    Ek,
    Fm,
}


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EdgeOrigin {
    
    Hk,
    
    Jd,
    
    Ap,
    
    Ca,
}


#[derive(Debug, Clone, Copy)]
pub enum GestureEvent {
    
    Bty { b: i32, c: i32 },

    
    Bey { b: i32, c: i32 },

    
    Blp { b: i32, c: i32 },

    
    Btt {
        sz: SwipeDirection,
        ql: i32,
        vc: i32,
        cqe: i32,
        hic: i32,
        qm: i32, 
    },

    
    Abp {
        atf: EdgeOrigin,
        li: i32, 
    },

    
    Boz {
        yv: i32,
        uq: i32,
        
        bv: i32,
    },

    
    Yq {
        iqw: i32,
        iqx: i32,
    },

    
    Bui {
        sz: SwipeDirection,
    },

    
    Bum { b: i32, c: i32 },

    
    Bun { b: i32, c: i32 },

    
    Qy { b: i32, c: i32 },

    
    Arf {
        b: i32,
        c: i32,
        ql: i32,
        vc: i32,
    },
}






#[derive(Clone, Copy)]
struct FingerTracker {
    gh: bool,
    ad: u16,
    
    ql: i32,
    vc: i32,
    
    aua: i32,
    bbi: i32,
    
    gtd: u64,
    
    gkw: u64,
    
    gmg: i32,
}

impl Default for FingerTracker {
    fn default() -> Self {
        Self {
            gh: false,
            ad: 0,
            ql: 0,
            vc: 0,
            aua: 0,
            bbi: 0,
            gtd: 0,
            gkw: 0,
            gmg: 0,
        }
    }
}

impl FingerTracker {
    const fn new() -> Self {
        Self {
            gh: false,
            ad: 0,
            ql: 0,
            vc: 0,
            aua: 0,
            bbi: 0,
            gtd: 0,
            gkw: 0,
            gmg: 0,
        }
    }

    fn hgj(&self) -> i32 {
        let dx = self.aua - self.ql;
        let bg = self.bbi - self.vc;
        
        dx.gp() + bg.gp()
    }

    fn ypq(&self) -> i32 {
        let dx = self.aua - self.ql;
        let bg = self.bbi - self.vc;
        dx * dx + bg * bg
    }

    fn ynx(&self) -> u64 {
        self.gkw.ao(self.gtd)
    }
}






#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RecogState {
    
    Cv,
    
    Ans,
    
    Axa,
    
    Arg,
    
    Baa,
    
    Azv,
}


pub struct GestureRecognizer {
    g: RecogState,
    
    axx: [FingerTracker; GU_],
    
    cxy: u8,
    
    anv: i32,
    akr: i32,
    
    lhz: i32,
    lia: i32,
    jcu: u64,
    
    hqk: bool,
    
    hnx: i32,
    
    jjw: i32,
    jjx: i32,
}

impl GestureRecognizer {
    
    pub const fn new(anv: i32, akr: i32) -> Self {
        Self {
            g: RecogState::Cv,
            axx: [FingerTracker::new(); GU_],
            cxy: 0,
            anv,
            akr,
            lhz: 0,
            lia: 0,
            jcu: 0,
            hqk: false,
            hnx: 0,
            jjw: 0,
            jjx: 0,
        }
    }

    
    pub fn dbw(&mut self, z: i32, ac: i32) {
        self.anv = z;
        self.akr = ac;
    }

    
    
    
    
    pub fn process(&mut self, id: &Zd) -> Option<GestureEvent> {
        let nl = &id.nl;

        match nl.ib {
            TouchPhase::Fm => self.uyc(nl),
            TouchPhase::Avu => self.uyd(nl),
            TouchPhase::Ek | TouchPhase::Aai => self.uye(nl),
        }
    }

    
    pub fn vml(&mut self, fjl: &mut GestureBuffer) {
        while let Some(id) = touch::dks() {
            if let Some(at) = self.process(&id) {
                fjl.push(at);
            }
        }

        
        if let Some(at) = self.qzd() {
            fjl.push(at);
        }
    }

    
    pub fn qzd(&mut self) -> Option<GestureEvent> {
        if self.g == RecogState::Ans
            && self.cxy == 1
            && !self.hqk
        {
            
            let (jf, sc, wte, ulb) = match self.ssw() {
                Some(bb) => (bb.aua, bb.bbi, bb.gtd, bb.gmg),
                None => return None,
            };
            let iu = crate::gui::engine::awf();
            let avr = iu.ao(wte);

            if avr >= CEV_
                && ulb < CEU_
            {
                self.hqk = true;
                self.g = RecogState::Axa;
                return Some(GestureEvent::Blp {
                    b: jf,
                    c: sc,
                });
            }
        }
        None
    }

    
    
    

    fn uyc(&mut self, nl: &TouchPoint) -> Option<GestureEvent> {
        
        let gk = self.stg()?;

        self.axx[gk] = FingerTracker {
            gh: true,
            ad: nl.ad,
            ql: nl.b,
            vc: nl.c,
            aua: nl.b,
            bbi: nl.c,
            gtd: nl.bsp,
            gkw: nl.bsp,
            gmg: 0,
        };

        self.cxy += 1;

        
        match self.cxy {
            1 => {
                self.g = RecogState::Ans;
                self.hqk = false;
            }
            2 => {
                self.g = RecogState::Baa;
                
                self.hnx = self.pwo();
                let (hl, ir) = self.mnl();
                self.jjw = hl;
                self.jjx = ir;
            }
            3 => {
                self.g = RecogState::Azv;
            }
            _ => {}
        }

        Some(GestureEvent::Bum {
            b: nl.b,
            c: nl.c,
        })
    }

    
    
    

    fn uyd(&mut self, nl: &TouchPoint) -> Option<GestureEvent> {
        
        let gk = self.nul(nl.ad)?;
        let acz = &mut self.axx[gk];
        acz.aua = nl.b;
        acz.bbi = nl.c;
        acz.gkw = nl.bsp;
        let aor = acz.hgj();
        if aor > acz.gmg {
            acz.gmg = aor;
        }

        match self.g {
            RecogState::Ans if self.cxy == 1 => {
                
                Some(GestureEvent::Bun {
                    b: nl.b,
                    c: nl.c,
                })
            }
            RecogState::Axa => {
                
                let acz = &self.axx[gk];
                self.g = RecogState::Arg;
                Some(GestureEvent::Arf {
                    b: nl.b,
                    c: nl.c,
                    ql: acz.ql,
                    vc: acz.vc,
                })
            }
            RecogState::Arg => {
                let acz = &self.axx[gk];
                Some(GestureEvent::Arf {
                    b: nl.b,
                    c: nl.c,
                    ql: acz.ql,
                    vc: acz.vc,
                })
            }
            RecogState::Baa => {
                self.tll()
            }
            RecogState::Azv => {
                
                None
            }
            _ => None,
        }
    }

    
    
    

    fn uye(&mut self, nl: &TouchPoint) -> Option<GestureEvent> {
        let gk = self.nul(nl.ad)
            .or_else(|| self.ssy());

        let gk = match gk {
            Some(e) => e,
            None => {
                self.apa();
                return Some(GestureEvent::Qy { b: nl.b, c: nl.c });
            }
        };

        
        self.axx[gk].aua = nl.b;
        self.axx[gk].bbi = nl.c;
        self.axx[gk].gkw = nl.bsp;

        let acz = self.axx[gk];
        let gesture = match self.g {
            RecogState::Ans if self.cxy == 1 => {
                self.raz(&acz, nl.bsp)
            }
            RecogState::Axa => {
                
                Some(GestureEvent::Qy { b: nl.b, c: nl.c })
            }
            RecogState::Arg => {
                Some(GestureEvent::Qy { b: nl.b, c: nl.c })
            }
            RecogState::Baa if self.cxy <= 2 => {
                
                Some(GestureEvent::Qy { b: nl.b, c: nl.c })
            }
            RecogState::Azv if self.cxy <= 3 => {
                self.rba()
            }
            _ => {
                Some(GestureEvent::Qy { b: nl.b, c: nl.c })
            }
        };

        
        self.axx[gk].gh = false;
        if self.cxy > 0 {
            self.cxy -= 1;
        }

        
        if self.cxy == 0 {
            self.g = RecogState::Cv;
        }

        gesture
    }

    
    
    

    fn raz(&mut self, acz: &FingerTracker, awf: u64) -> Option<GestureEvent> {
        let avr = awf.ao(acz.gtd);
        let dx = acz.aua - acz.ql;
        let bg = acz.bbi - acz.vc;
        let hgj = dx.gp() + bg.gp();

        
        if avr < CXJ_ && hgj < CXI_ {
            
            let wow = awf.ao(self.jcu);
            let xas = (acz.aua - self.lhz).gp()
                + (acz.bbi - self.lia).gp();

            self.lhz = acz.aua;
            self.lia = acz.bbi;
            self.jcu = awf;

            if wow < BSD_ && xas < BSC_ {
                
                self.jcu = 0;
                return Some(GestureEvent::Bey {
                    b: acz.aua,
                    c: acz.bbi,
                });
            }

            return Some(GestureEvent::Bty {
                b: acz.aua,
                c: acz.bbi,
            });
        }

        
        if hgj >= LD_ {
            let shn = (avr / 10_000).am(1) as i32; 
            let qm = (hgj * 100) / shn; 

            if qm >= CUT_ {
                
                if let Some(siv) = self.qys(acz, dx, bg) {
                    return Some(siv);
                }

                
                let sz = if dx.gp() > bg.gp() {
                    if dx > 0 { SwipeDirection::Ca } else { SwipeDirection::Ap }
                } else {
                    if bg > 0 { SwipeDirection::Fm } else { SwipeDirection::Ek }
                };

                return Some(GestureEvent::Btt {
                    sz,
                    ql: acz.ql,
                    vc: acz.vc,
                    cqe: acz.aua,
                    hic: acz.bbi,
                    qm,
                });
            }
        }

        
        Some(GestureEvent::Qy {
            b: acz.aua,
            c: acz.bbi,
        })
    }

    fn qys(&self, acz: &FingerTracker, dx: i32, bg: i32) -> Option<GestureEvent> {
        
        if acz.vc >= self.akr - SY_ && bg < -LD_ {
            return Some(GestureEvent::Abp {
                atf: EdgeOrigin::Hk,
                li: bg.gp(),
            });
        }

        
        if acz.vc <= SY_ && bg > LD_ {
            return Some(GestureEvent::Abp {
                atf: EdgeOrigin::Jd,
                li: bg.gp(),
            });
        }

        
        if acz.ql <= SY_ && dx > LD_ {
            return Some(GestureEvent::Abp {
                atf: EdgeOrigin::Ap,
                li: dx.gp(),
            });
        }

        
        if acz.ql >= self.anv - SY_ && dx < -LD_ {
            return Some(GestureEvent::Abp {
                atf: EdgeOrigin::Ca,
                li: dx.gp(),
            });
        }

        None
    }

    fn tll(&mut self) -> Option<GestureEvent> {
        let (yqd, nsp) = self.kyy()?;

        let nia = self.pwo();
        let rza = nia - self.hnx;

        
        if rza.gp() >= CJQ_ {
            let (cx, ae) = self.mnl();
            
            let bv = if self.hnx > 0 {
                (nia * 100) / self.hnx.am(1)
            } else {
                100
            };
            return Some(GestureEvent::Boz {
                yv: cx,
                uq: ae,
                bv,
            });
        }

        
        let (hl, ir) = self.mnl();
        let pgq = hl - self.jjw;
        let pgr = ir - self.jjx;

        if pgq.gp() >= BFB_ || pgr.gp() >= BFB_ {
            self.jjw = hl;
            self.jjx = ir;
            return Some(GestureEvent::Yq {
                iqw: pgq,
                iqx: pgr,
            });
        }

        None
    }

    fn rba(&self) -> Option<GestureEvent> {
        
        let mut jtq = 0i32;
        let mut az = 0;
        for acz in &self.axx {
            if acz.gh {
                jtq += acz.aua - acz.ql;
                az += 1;
            }
        }
        if az == 0 { return None; }

        let mxb = jtq / az;

        if mxb.gp() >= LD_ {
            let sz = if mxb > 0 {
                SwipeDirection::Ca
            } else {
                SwipeDirection::Ap
            };
            Some(GestureEvent::Bui { sz })
        } else {
            Some(GestureEvent::Qy { b: 0, c: 0 })
        }
    }

    
    
    

    fn stg(&self) -> Option<usize> {
        for (a, bb) in self.axx.iter().cf() {
            if !bb.gh {
                return Some(a);
            }
        }
        None
    }

    fn nul(&self, ad: u16) -> Option<usize> {
        for (a, bb) in self.axx.iter().cf() {
            if bb.gh && bb.ad == ad {
                return Some(a);
            }
        }
        None
    }

    fn ssy(&self) -> Option<usize> {
        for (a, bb) in self.axx.iter().cf() {
            if bb.gh {
                return Some(a);
            }
        }
        None
    }

    fn ssw(&self) -> Option<&FingerTracker> {
        self.axx.iter().du(|bb| bb.gh)
    }

    fn kyy(&self) -> Option<(usize, usize)> {
        let mut aig = [0usize; 2];
        let mut az = 0;
        for (a, bb) in self.axx.iter().cf() {
            if bb.gh && az < 2 {
                aig[az] = a;
                az += 1;
            }
        }
        if az == 2 {
            Some((aig[0], aig[1]))
        } else {
            None
        }
    }

    fn pwo(&self) -> i32 {
        if let Some((q, o)) = self.kyy() {
            let dx = self.axx[q].aua - self.axx[o].aua;
            let bg = self.axx[q].bbi - self.axx[o].bbi;
            
            hpe((dx * dx + bg * bg) as u32) as i32
        } else {
            0
        }
    }

    fn mnl(&self) -> (i32, i32) {
        if let Some((q, o)) = self.kyy() {
            let hl = (self.axx[q].aua + self.axx[o].aua) / 2;
            let ir = (self.axx[q].bbi + self.axx[o].bbi) / 2;
            (hl, ir)
        } else {
            (0, 0)
        }
    }

    fn apa(&mut self) {
        self.g = RecogState::Cv;
        self.cxy = 0;
        self.hqk = false;
        for bb in &mut self.axx {
            bb.gh = false;
        }
    }
}






pub struct GestureBuffer {
    fjl: [Option<GestureEvent>; 8],
    az: usize,
}

impl GestureBuffer {
    pub const fn new() -> Self {
        Self {
            fjl: [None; 8],
            az: 0,
        }
    }

    pub fn push(&mut self, gesture: GestureEvent) {
        if self.az < 8 {
            self.fjl[self.az] = Some(gesture);
            self.az += 1;
        }
    }

    pub fn len(&self) -> usize {
        self.az
    }

    pub fn is_empty(&self) -> bool {
        self.az == 0
    }

    pub fn iter(&self) -> Asx<'_> {
        Asx {
            k: self,
            w: 0,
        }
    }

    pub fn clear(&mut self) {
        self.az = 0;
        self.fjl = [None; 8];
    }
}

pub struct Asx<'a> {
    k: &'a GestureBuffer,
    w: usize,
}

impl<'a> Iterator for Asx<'a> {
    type Item = &'a GestureEvent;
    fn next(&mut self) -> Option<Self::Item> {
        while self.w < self.k.az {
            let a = self.w;
            self.w += 1;
            if let Some(ref at) = self.k.fjl[a] {
                return Some(at);
            }
        }
        None
    }
}






fn hpe(bo: u32) -> u32 {
    if bo == 0 { return 0; }
    let mut b = bo;
    let mut c = (b + 1) / 2;
    while c < b {
        b = c;
        c = (b + bo / b) / 2;
    }
    b
}
