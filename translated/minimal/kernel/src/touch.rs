











use core::sync::atomic::{AtomicBool, AtomicI32, AtomicU8, AtomicU16, AtomicU32, AtomicU64, Ordering};






pub const GU_: usize = 10;


const CV_: usize = 64;
const ARI_: usize = CV_ - 1;






#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum TouchPhase {
    
    Fm = 0,
    
    Avu = 1,
    
    Ek = 2,
    
    Aai = 3,
}

impl TouchPhase {
    fn ckp(p: u8) -> Self {
        match p {
            0 => Self::Fm,
            1 => Self::Avu,
            2 => Self::Ek,
            3 => Self::Aai,
            _ => Self::Aai,
        }
    }
}


#[derive(Debug, Clone, Copy)]
pub struct TouchPoint {
    
    pub ad: u16,
    
    pub b: i32,
    
    pub c: i32,
    
    pub cgr: u8,
    
    pub ib: TouchPhase,
    
    pub bsp: u64,
}

impl Default for TouchPoint {
    fn default() -> Self {
        Self {
            ad: 0,
            b: 0,
            c: 0,
            cgr: 0,
            ib: TouchPhase::Ek,
            bsp: 0,
        }
    }
}


#[derive(Debug, Clone)]
pub struct TouchState {
    
    pub egw: [TouchPoint; GU_],
    
    pub az: u8,
    
    pub bsp: u64,
}

impl Default for TouchState {
    fn default() -> Self {
        Self {
            egw: [TouchPoint::default(); GU_],
            az: 0,
            bsp: 0,
        }
    }
}


#[derive(Debug, Clone, Copy)]
pub struct Zd {
    pub nl: TouchPoint,
}






static Be: AtomicBool = AtomicBool::new(false);


static AQF_: AtomicBool = AtomicBool::new(false);


static IW_: AtomicU32 = AtomicU32::new(1280);
static IV_: AtomicU32 = AtomicU32::new(800);


static AQD_: AtomicU32 = AtomicU32::new(4096);
static AQE_: AtomicU32 = AtomicU32::new(4096);


static YT_: AtomicU8 = AtomicU8::new(0);



struct AtomicTouchSlot {
    gh: AtomicBool,
    ad: AtomicU16,
    b: AtomicI32,
    c: AtomicI32,
    cgr: AtomicU8,
    ib: AtomicU8,
    bsp: AtomicU64,
}

impl AtomicTouchSlot {
    const fn new() -> Self {
        Self {
            gh: AtomicBool::new(false),
            ad: AtomicU16::new(0),
            b: AtomicI32::new(0),
            c: AtomicI32::new(0),
            cgr: AtomicU8::new(0),
            ib: AtomicU8::new(TouchPhase::Ek as u8),
            bsp: AtomicU64::new(0),
        }
    }

    fn load(&self) -> TouchPoint {
        TouchPoint {
            ad: self.ad.load(Ordering::Relaxed),
            b: self.b.load(Ordering::Relaxed),
            c: self.c.load(Ordering::Relaxed),
            cgr: self.cgr.load(Ordering::Relaxed),
            ib: TouchPhase::ckp(self.ib.load(Ordering::Relaxed)),
            bsp: self.bsp.load(Ordering::Relaxed),
        }
    }

    fn store(&self, nl: &TouchPoint) {
        self.ad.store(nl.ad, Ordering::Relaxed);
        self.b.store(nl.b, Ordering::Relaxed);
        self.c.store(nl.c, Ordering::Relaxed);
        self.cgr.store(nl.cgr, Ordering::Relaxed);
        self.ib.store(nl.ib as u8, Ordering::Relaxed);
        self.bsp.store(nl.bsp, Ordering::Relaxed);
        self.gh.store(nl.ib != TouchPhase::Ek && nl.ib != TouchPhase::Aai, Ordering::Relaxed);
    }

    fn clear(&self) {
        self.gh.store(false, Ordering::Relaxed);
        self.ib.store(TouchPhase::Ek as u8, Ordering::Relaxed);
    }
}


static HG_: [AtomicTouchSlot; GU_] = [
    AtomicTouchSlot::new(), AtomicTouchSlot::new(),
    AtomicTouchSlot::new(), AtomicTouchSlot::new(),
    AtomicTouchSlot::new(), AtomicTouchSlot::new(),
    AtomicTouchSlot::new(), AtomicTouchSlot::new(),
    AtomicTouchSlot::new(), AtomicTouchSlot::new(),
];






struct Arr {
    
    
    esg: [AtomicU16; CV_],
    mrt: [AtomicI32; CV_],
    fzf: [AtomicI32; CV_],
    lvd: [AtomicU8; CV_],
    lts: [AtomicU8; CV_],
    mlc: [AtomicU64; CV_],
    
    dxt: AtomicU32,
    
    lxp: AtomicU32,
}


macro_rules! gam {
    ($type:ty, $ap:expr, $bo:expr) => {{
        
        
        const Dm: $type = $ap;
        [Dm; $bo]
    }};
}

static GK_: Arr = Arr {
    esg: gam!(AtomicU16, AtomicU16::new(0), CV_),
    mrt: gam!(AtomicI32, AtomicI32::new(0), CV_),
    fzf: gam!(AtomicI32, AtomicI32::new(0), CV_),
    lvd: gam!(AtomicU8, AtomicU8::new(0), CV_),
    lts: gam!(AtomicU8, AtomicU8::new(0), CV_),
    mlc: gam!(AtomicU64, AtomicU64::new(0), CV_),
    dxt: AtomicU32::new(0),
    lxp: AtomicU32::new(0),
};

impl Arr {
    
    fn push(&self, nl: &TouchPoint) {
        let d = self.dxt.load(Ordering::Relaxed);
        let w = (d as usize) & ARI_;

        self.esg[w].store(nl.ad, Ordering::Relaxed);
        self.mrt[w].store(nl.b, Ordering::Relaxed);
        self.fzf[w].store(nl.c, Ordering::Relaxed);
        self.lvd[w].store(nl.cgr, Ordering::Relaxed);
        self.lts[w].store(nl.ib as u8, Ordering::Relaxed);
        self.mlc[w].store(nl.bsp, Ordering::Relaxed);

        
        self.dxt.store(d.cn(1), Ordering::Release);
    }

    
    fn pop(&self) -> Option<Zd> {
        let m = self.lxp.load(Ordering::Relaxed);
        let d = self.dxt.load(Ordering::Acquire);

        if m == d {
            return None; 
        }

        let w = (m as usize) & ARI_;
        let nl = TouchPoint {
            ad: self.esg[w].load(Ordering::Relaxed),
            b: self.mrt[w].load(Ordering::Relaxed),
            c: self.fzf[w].load(Ordering::Relaxed),
            cgr: self.lvd[w].load(Ordering::Relaxed),
            ib: TouchPhase::ckp(self.lts[w].load(Ordering::Relaxed)),
            bsp: self.mlc[w].load(Ordering::Relaxed),
        };

        self.lxp.store(m.cn(1), Ordering::Release);
        Some(Zd { nl })
    }
}






pub fn init() {
    
    for gk in &HG_ {
        gk.clear();
    }
    YT_.store(0, Ordering::Relaxed);
    Be.store(true, Ordering::Relaxed);
    crate::serial_println!("[TOUCH] Touch subsystem initialized (max {} points)", GU_);
}


pub fn dbw(z: u32, ac: u32) {
    IW_.store(z, Ordering::Relaxed);
    IV_.store(ac, Ordering::Relaxed);
}


pub fn zmv(bvj: u32, csl: u32) {
    AQD_.store(bvj, Ordering::Relaxed);
    AQE_.store(csl, Ordering::Relaxed);
}


pub fn zmu(brs: bool) {
    AQF_.store(brs, Ordering::Relaxed);
    if brs {
        crate::serial_println!("[TOUCH] Touchscreen device detected");
    }
}


pub fn anl() -> bool {
    Be.load(Ordering::Relaxed) && AQF_.load(Ordering::Relaxed)
}


pub fn ky() -> bool {
    Be.load(Ordering::Relaxed)
}


pub fn gxu() -> u8 {
    YT_.load(Ordering::Relaxed)
}


pub fn drd() -> TouchState {
    let mut g = TouchState::default();
    let mut az = 0u8;

    for gk in &HG_ {
        if gk.gh.load(Ordering::Relaxed) && (az as usize) < GU_ {
            g.egw[az as usize] = gk.load();
            az += 1;
        }
    }

    g.az = az;
    g.bsp = crate::gui::engine::awf();
    g
}


pub fn dks() -> Option<Zd> {
    GK_.pop()
}


pub fn say<G: FnMut(Zd)>(mut bb: G) {
    while let Some(ebi) = GK_.pop() {
        bb(ebi);
    }
}









pub fn lep(ad: u16, lxf: u32, fsa: u32, cgr: u8, ib: TouchPhase) {
    let wf = IW_.load(Ordering::Relaxed);
    let aav = IV_.load(Ordering::Relaxed);
    let rxa = AQD_.load(Ordering::Relaxed).am(1);
    let rxb = AQE_.load(Ordering::Relaxed).am(1);

    
    let b = ((lxf as u64 * wf as u64) / rxa as u64) as i32;
    let c = ((fsa as u64 * aav as u64) / rxb as u64) as i32;

    tuo(ad, b, c, cgr, ib);
}





pub fn tuo(ad: u16, b: i32, c: i32, cgr: u8, ib: TouchPhase) {
    let wf = IW_.load(Ordering::Relaxed) as i32;
    let aav = IV_.load(Ordering::Relaxed) as i32;

    let nl = TouchPoint {
        ad,
        b: b.qp(0, wf - 1),
        c: c.qp(0, aav - 1),
        cgr,
        ib,
        bsp: crate::gui::engine::awf(),
    };

    
    let wpr = stl(ad, ib);
    if let Some(w) = wpr {
        HG_[w].store(&nl);
    }

    
    let mut az = 0u8;
    for gk in &HG_ {
        if gk.gh.load(Ordering::Relaxed) {
            az += 1;
        }
    }
    YT_.store(az, Ordering::Relaxed);

    
    GK_.push(&nl);
}



fn stl(ad: u16, ib: TouchPhase) -> Option<usize> {
    
    for (a, gk) in HG_.iter().cf() {
        if gk.gh.load(Ordering::Relaxed) && gk.ad.load(Ordering::Relaxed) == ad {
            return Some(a);
        }
    }

    
    if ib == TouchPhase::Fm {
        for (a, gk) in HG_.iter().cf() {
            if !gk.gh.load(Ordering::Relaxed) {
                return Some(a);
            }
        }
    }

    
    if ib == TouchPhase::Ek || ib == TouchPhase::Aai {
        
        for (a, gk) in HG_.iter().cf() {
            if gk.ad.load(Ordering::Relaxed) == ad {
                return Some(a);
            }
        }
    }

    None
}








pub fn yoz() -> Option<(i32, i32, bool)> {
    
    for gk in &HG_ {
        if gk.gh.load(Ordering::Relaxed) {
            let b = gk.b.load(Ordering::Relaxed);
            let c = gk.c.load(Ordering::Relaxed);
            let ib = TouchPhase::ckp(gk.ib.load(Ordering::Relaxed));
            let vn = ib == TouchPhase::Fm || ib == TouchPhase::Avu;
            return Some((b, c, vn));
        }
    }
    None
}
