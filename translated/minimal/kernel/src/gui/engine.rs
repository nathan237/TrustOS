










use alloc::string::String;
use alloc::vec::Vec;
use alloc::vec;
use alloc::boxed::Box;
use core::sync::atomic::{AtomicBool, AtomicU64, AtomicI32, Ordering};
use spin::Mutex;






const BGX_: u64 = 16_666;


static FW_: AtomicU64 = AtomicU64::new(3_000_000_000); 


static ASI_: AtomicU64 = AtomicU64::new(0);
static AYT_: AtomicU64 = AtomicU64::new(0);
static APQ_: AtomicU64 = AtomicU64::new(0);


pub fn oep() {
    
    let kx = crate::cpu::mnh();
    FW_.store(kx, Ordering::SeqCst);
    crate::serial_println!("[GUI] Frame timing init: TSC {} MHz", kx / 1_000_000);
}


#[inline]
fn xnb(qb: u64) -> u64 {
    let kx = FW_.load(Ordering::Relaxed);
    if kx == 0 { return 0; }
    (qb * 1_000_000) / kx
}


#[inline]
pub fn awf() -> u64 {
    xnb(ow())
}


#[inline]
fn ow() -> u64 {
    crate::arch::aea()
}


pub fn zvv(ivr: u64) {
    let ez = awf().ao(ivr);
    
    if ez < BGX_ {
        let cd = ivr + BGX_;
        let mut gzn = 0u32;
        while awf() < cd {
            gzn += 1;
            if gzn >= 2_000_000 { break; } 
            core::hint::hc();
        }
    }
    
    
    let az = ASI_.fetch_add(1, Ordering::Relaxed);
    let iu = awf();
    let qv = AYT_.load(Ordering::Relaxed);
    if iu - qv >= 1_000_000 {
        APQ_.store(az, Ordering::Relaxed);
        ASI_.store(0, Ordering::Relaxed);
        AYT_.store(iu, Ordering::Relaxed);
    }
}


pub fn kyp() -> u64 {
    APQ_.load(Ordering::Relaxed)
}






static BAW_: AtomicBool = AtomicBool::new(false);
static BAX_: AtomicBool = AtomicBool::new(false);
static CHA_: AtomicBool = AtomicBool::new(false);
static BAY_: AtomicBool = AtomicBool::new(false);


#[derive(Clone, Copy, Debug, PartialEq)]
pub enum HotkeyAction {
    None,
    
    Bzo,      
    Cnp,     
    Cnc,         
    Cnd,        
    Chg,         
    Chl,         
    Cmx,      
    
    Aks,  
    Tu,     
    Cil,          
    
    Cgw,       
    Ddm,    
    Dil,       
    Coc,   
}


pub mod scancode {
    pub const Bbi: u8 = 0x38;
    pub const Bdf: u8 = 0x1D;
    pub const Bri: u8 = 0x2A;
    pub const Bwh: u8 = 0x5B;  
    pub const Cnq: u8 = 0x0F;
    pub const Ccs: u8 = 0x3E;
    pub const Cgm: u8 = 0x4B;
    pub const Cju: u8 = 0x4D;
    pub const Afc: u8 = 0x48;
    pub const Cam: u8 = 0x50;
    pub const Bdy: u8 = 0x20;
    pub const Se: u8 = 0x12;
    pub const T: u8 = 0x14;
    pub const Ac: u8 = 0x13;
    pub const Aur: u8 = 0x26;
    pub const Cvk: u8 = 0x01;
    pub const Ccr: u8 = 0x58;
}


pub fn zuv(scancode: u8, vn: bool) {
    match scancode {
        scancode::Bbi => BAW_.store(vn, Ordering::Relaxed),
        scancode::Bdf => BAX_.store(vn, Ordering::Relaxed),
        scancode::Bri => CHA_.store(vn, Ordering::Relaxed),
        scancode::Bwh => BAY_.store(vn, Ordering::Relaxed),
        _ => {}
    }
}


pub fn yhw(scancode: u8) -> HotkeyAction {
    let bdj = BAW_.load(Ordering::Relaxed);
    let db = BAX_.load(Ordering::Relaxed);
    let ep = BAY_.load(Ordering::Relaxed);
    
    
    if bdj && scancode == scancode::Ccs {
        return HotkeyAction::Bzo;
    }
    
    
    if bdj && scancode == scancode::Cnq {
        return HotkeyAction::Cnp;
    }
    
    
    if ep {
        match scancode {
            scancode::Cgm => return HotkeyAction::Cnc,
            scancode::Cju => return HotkeyAction::Cnd,
            scancode::Afc => return HotkeyAction::Chg,
            scancode::Cam => return HotkeyAction::Chl,
            scancode::Bdy => return HotkeyAction::Cmx,
            scancode::Se => return HotkeyAction::Aks,
            scancode::T => return HotkeyAction::Tu,
            scancode::Ac => return HotkeyAction::Cil,
            scancode::Aur => return HotkeyAction::Cgw,
            _ => {}
        }
    }
    
    
    if db && bdj && scancode == scancode::T {
        return HotkeyAction::Tu;
    }
    
    
    if scancode == scancode::Ccr {
        return HotkeyAction::Coc;
    }
    
    HotkeyAction::None
}


static YP_: AtomicBool = AtomicBool::new(false);

pub fn yib(scancode: u8, vn: bool) -> bool {
    if scancode == scancode::Bwh {
        if vn {
            YP_.store(true, Ordering::Relaxed);
        } else {
            
            if YP_.load(Ordering::Relaxed) {
                YP_.store(false, Ordering::Relaxed);
                return true; 
            }
        }
    } else if vn {
        
        YP_.store(false, Ordering::Relaxed);
    }
    false
}






#[derive(Clone, Copy, Debug, PartialEq)]
pub enum SnapPosition {
    None,
    Ap,       
    Ca,      
    Chh,  
    Dp,    
    Dq,
    Dt,
    Du,
}


pub fn yhf(
    xj: SnapPosition,
    wf: u32,
    aav: u32,
    xbc: u32,
) -> (i32, i32, u32, u32) {
    let bxw = aav - xbc;
    
    match xj {
        SnapPosition::Ap => (0, 0, wf / 2, bxw),
        SnapPosition::Ca => ((wf / 2) as i32, 0, wf / 2, bxw),
        SnapPosition::Chh => (0, 0, wf, bxw),
        SnapPosition::Dp => (0, 0, wf / 2, bxw / 2),
        SnapPosition::Dq => ((wf / 2) as i32, 0, wf / 2, bxw / 2),
        SnapPosition::Dt => (0, (bxw / 2) as i32, wf / 2, bxw / 2),
        SnapPosition::Du => ((wf / 2) as i32, (bxw / 2) as i32, wf / 2, bxw / 2),
        SnapPosition::None => (0, 0, 400, 300), 
    }
}






static YZ_: AtomicBool = AtomicBool::new(false);
static LQ_: AtomicI32 = AtomicI32::new(0);


pub fn pny() {
    YZ_.store(true, Ordering::Relaxed);
    LQ_.store(0, Ordering::Relaxed);
}


pub fn mvg() {
    LQ_.fetch_add(1, Ordering::Relaxed);
}


pub fn yeu() {
    LQ_.fetch_sub(1, Ordering::Relaxed);
}


pub fn stv() -> i32 {
    YZ_.store(false, Ordering::Relaxed);
    LQ_.load(Ordering::Relaxed)
}


pub fn hot() -> bool {
    YZ_.load(Ordering::Relaxed)
}


pub fn qhl() -> i32 {
    LQ_.load(Ordering::Relaxed)
}






#[derive(Clone, Copy, Debug, PartialEq)]
pub enum CursorType {
    Ov,
    Cev,           
    Text,           
    Cko,       
    Ckn,       
    Aeh,     
    Aeg,     
    Fw,           
    Bwm,           
    Bdw,      
}

static APP_: Mutex<CursorType> = Mutex::new(CursorType::Ov);


pub fn bld(gi: CursorType) {
    *APP_.lock() = gi;
}


pub fn gia() -> CursorType {
    *APP_.lock()
}


pub fn ysx(gi: CursorType) -> &'static [u8; 64] {
    match gi {
        CursorType::Ov => &BQL_,
        CursorType::Cev => &BQN_,
        CursorType::Text => &BQT_,
        CursorType::Cko => &BQR_,
        CursorType::Ckn => &BQP_,
        CursorType::Aeh => &BQS_,
        CursorType::Aeg => &BQQ_,
        CursorType::Fw => &BQO_,
        CursorType::Bwm => &BQU_,
        CursorType::Bdw => &BQM_,
    }
}


static BQL_: [u8; 64] = [
    2,0,0,0,0,0,0,0,
    2,2,0,0,0,0,0,0,
    2,1,2,0,0,0,0,0,
    2,1,1,2,0,0,0,0,
    2,1,1,1,2,0,0,0,
    2,1,1,1,1,2,0,0,
    2,1,1,2,2,0,0,0,
    2,2,2,0,0,0,0,0,
];

static BQN_: [u8; 64] = [
    0,0,2,2,0,0,0,0,
    0,2,1,1,2,0,0,0,
    0,2,1,1,2,0,0,0,
    0,2,1,1,2,2,2,0,
    2,2,1,1,1,1,1,2,
    2,1,1,1,1,1,1,2,
    2,1,1,1,1,1,1,2,
    0,2,2,2,2,2,2,0,
];

static BQT_: [u8; 64] = [
    0,2,2,2,2,2,0,0,
    0,0,0,2,0,0,0,0,
    0,0,0,2,0,0,0,0,
    0,0,0,2,0,0,0,0,
    0,0,0,2,0,0,0,0,
    0,0,0,2,0,0,0,0,
    0,0,0,2,0,0,0,0,
    0,2,2,2,2,2,0,0,
];

static BQR_: [u8; 64] = [
    0,0,0,2,0,0,0,0,
    0,0,2,1,2,0,0,0,
    0,2,1,1,1,2,0,0,
    0,0,0,2,0,0,0,0,
    0,0,0,2,0,0,0,0,
    0,2,1,1,1,2,0,0,
    0,0,2,1,2,0,0,0,
    0,0,0,2,0,0,0,0,
];

static BQP_: [u8; 64] = [
    0,0,0,0,0,0,0,0,
    0,0,2,0,0,2,0,0,
    0,2,1,2,2,1,2,0,
    2,1,1,1,1,1,1,2,
    0,2,1,2,2,1,2,0,
    0,0,2,0,0,2,0,0,
    0,0,0,0,0,0,0,0,
    0,0,0,0,0,0,0,0,
];

static BQS_: [u8; 64] = [
    2,2,2,2,0,0,0,0,
    2,1,1,2,0,0,0,0,
    2,1,2,0,0,0,0,0,
    2,2,0,2,0,0,0,0,
    0,0,0,0,2,0,2,2,
    0,0,0,0,0,2,1,2,
    0,0,0,0,2,1,1,2,
    0,0,0,0,2,2,2,2,
];

static BQQ_: [u8; 64] = [
    0,0,0,0,2,2,2,2,
    0,0,0,0,2,1,1,2,
    0,0,0,0,0,2,1,2,
    0,0,0,0,2,0,2,2,
    2,2,0,2,0,0,0,0,
    2,1,2,0,0,0,0,0,
    2,1,1,2,0,0,0,0,
    2,2,2,2,0,0,0,0,
];

static BQO_: [u8; 64] = [
    0,0,0,2,0,0,0,0,
    0,0,2,1,2,0,0,0,
    0,0,0,2,0,0,0,0,
    2,2,2,2,2,2,2,0,
    0,0,0,2,0,0,0,0,
    0,0,2,1,2,0,0,0,
    0,0,0,2,0,0,0,0,
    0,0,0,0,0,0,0,0,
];

static BQU_: [u8; 64] = [
    2,2,2,2,2,2,0,0,
    2,1,1,1,1,2,0,0,
    0,2,1,1,2,0,0,0,
    0,0,2,2,0,0,0,0,
    0,0,2,2,0,0,0,0,
    0,2,1,1,2,0,0,0,
    2,1,1,1,1,2,0,0,
    2,2,2,2,2,2,0,0,
];

static BQM_: [u8; 64] = [
    0,0,0,2,0,0,0,0,
    0,0,0,2,0,0,0,0,
    0,0,0,2,0,0,0,0,
    2,2,2,1,2,2,2,0,
    0,0,0,2,0,0,0,0,
    0,0,0,2,0,0,0,0,
    0,0,0,2,0,0,0,0,
    0,0,0,0,0,0,0,0,
];






#[derive(Clone, Copy, Debug, PartialEq)]
pub enum NotifyPriority {
    V,
    Oo,
    Q,
    Hf,
}


pub struct Toast {
    pub dq: String,
    pub message: String,
    pub abv: NotifyPriority,
    pub cju: u64,
    pub uk: u64,
    pub li: Option<u8>, 
}

impl Toast {
    pub fn new(dq: &str, message: &str, abv: NotifyPriority) -> Self {
        Self {
            dq: String::from(dq),
            message: String::from(message),
            abv,
            cju: awf(),
            uk: 5000,
            li: None,
        }
    }
    
    pub fn xuw(mut self, jn: u64) -> Self {
        self.uk = jn;
        self
    }
    
    pub fn xux(mut self, egl: u8) -> Self {
        self.li = Some(egl.v(100));
        self
    }
    
    pub fn hox(&self) -> bool {
        let ez = (awf() - self.cju) / 1000;
        ez >= self.uk
    }
    
    
    pub fn oz(&self) -> u64 {
        (awf() - self.cju) / 1000
    }
    
    
    pub fn adh(&self) -> u8 {
        let ez = self.oz();
        
        if ez < 300 {
            return ((ez * 255) / 300).v(255) as u8;
        }
        
        if self.uk > 500 && ez > self.uk - 500 {
            let ia = self.uk.ao(ez);
            return ((ia * 255) / 500).v(255) as u8;
        }
        255
    }
    
    pub fn tda(&self) -> u32 {
        match self.abv {
            NotifyPriority::V => 0xFF3498DB,    
            NotifyPriority::Oo => 0xFFF39C12, 
            NotifyPriority::Q => 0xFFE74C3C,   
            NotifyPriority::Hf => 0xFF27AE60, 
        }
    }
}


static Avw: Mutex<Vec<Toast>> = Mutex::new(Vec::new());
const CFL_: usize = 5;


pub fn wnq(dq: &str, message: &str, abv: NotifyPriority) {
    let mut csv = Avw.lock();
    
    
    csv.ajm(|bo| !bo.hox());
    
    
    while csv.len() >= CFL_ {
        csv.remove(0);
    }
    
    csv.push(Toast::new(dq, message, abv));
}


pub fn zoh(dq: &str, message: &str, egl: u8) {
    let mut csv = Avw.lock();
    
    
    for bo in csv.el() {
        if bo.dq == dq && bo.li.is_some() {
            bo.li = Some(egl.v(100));
            bo.message = String::from(message);
            return;
        }
    }
    
    
    csv.push(Toast::new(dq, message, NotifyPriority::V)
        .xux(egl)
        .xuw(30000)); 
}


pub fn nyg() -> Vec<Toast> {
    let mut csv = Avw.lock();
    csv.ajm(|bo| !bo.hox());
    csv.clone()
}

impl Clone for Toast {
    fn clone(&self) -> Self {
        Self {
            dq: self.dq.clone(),
            message: self.message.clone(),
            abv: self.abv,
            cju: self.cju,
            uk: self.uk,
            li: self.li,
        }
    }
}






static XI_: AtomicBool = AtomicBool::new(false);


#[derive(Clone)]
pub struct Of {
    pub j: String,
    pub pa: u8,      
    pub hr: StartAction,
}

#[derive(Clone, Copy, Debug)]
pub enum StartAction {
    Akr(&'static str),
    Tu,
    Cih,
    Akt,
    Akq,
    Qt,
    Cks,
    Cgv,
}


pub fn zta() {
    let cv = XI_.load(Ordering::Relaxed);
    XI_.store(!cv, Ordering::Relaxed);
}


pub fn yix() {
    XI_.store(false, Ordering::Relaxed);
}


pub fn zaa() -> bool {
    XI_.load(Ordering::Relaxed)
}


pub fn nyp() -> Vec<Of> {
    vec![
        Of { j: String::from("Terminal"), pa: 0, hr: StartAction::Tu },
        Of { j: String::from("Files"), pa: 1, hr: StartAction::Cih },
        Of { j: String::from("Settings"), pa: 2, hr: StartAction::Akt },
        Of { j: String::from("About"), pa: 3, hr: StartAction::Akq },
        Of { j: String::from("───────────"), pa: 255, hr: StartAction::Akq },
        Of { j: String::from("Lock"), pa: 4, hr: StartAction::Cgv },
        Of { j: String::from("Restart"), pa: 5, hr: StartAction::Cks },
        Of { j: String::from("Shutdown"), pa: 6, hr: StartAction::Qt },
    ]
}







static JL_: [[u8; 256]; 256] = {
    let mut gg = [[0u8; 256]; 256];
    let mut dw = 0usize;
    while dw < 256 {
        let mut bn = 0usize;
        while bn < 256 {
            gg[dw][bn] = ((bn * dw + 127) / 255) as u8;
            bn += 1;
        }
        dw += 1;
    }
    gg
};


#[inline(always)]
pub fn kds(cy: u32, cs: u32) -> u32 {
    let dw = ((cy >> 24) & 0xFF) as usize;
    if dw == 0 { return cs; }
    if dw == 255 { return cy; }
    
    let akg = 255 - dw;
    
    let adz = ((cy >> 16) & 0xFF) as usize;
    let bsi = ((cy >> 8) & 0xFF) as usize;
    let is = (cy & 0xFF) as usize;
    
    let ahh = ((cs >> 16) & 0xFF) as usize;
    let bgs = ((cs >> 8) & 0xFF) as usize;
    let ng = (cs & 0xFF) as usize;
    
    let m = JL_[dw][adz] as u32 + JL_[akg][ahh] as u32;
    let at = JL_[dw][bsi] as u32 + JL_[akg][bgs] as u32;
    let o = JL_[dw][is] as u32 + JL_[akg][ng] as u32;
    
    0xFF000000 | (m << 16) | (at << 8) | o
}






#[derive(Clone, Copy, Default, Debug)]
pub struct Rect {
    pub b: i32,
    pub c: i32,
    pub d: u32,
    pub i: u32,
}

impl Rect {
    pub const fn new(b: i32, c: i32, d: u32, i: u32) -> Self {
        Self { b, c, d, i }
    }
    
    pub fn jao(&self, gq: &Rect) -> bool {
        self.b < gq.b + gq.d as i32 &&
        self.b + self.d as i32 > gq.b &&
        self.c < gq.c + gq.i as i32 &&
        self.c + self.i as i32 > gq.c
    }
    
    pub fn far(&self, gq: &Rect) -> Rect {
        let dn = self.b.v(gq.b);
        let dp = self.c.v(gq.c);
        let hy = (self.b + self.d as i32).am(gq.b + gq.d as i32);
        let jz = (self.c + self.i as i32).am(gq.c + gq.i as i32);
        Rect {
            b: dn,
            c: dp,
            d: (hy - dn) as u32,
            i: (jz - dp) as u32,
        }
    }
    
    pub fn ahy(&self) -> u32 {
        self.d * self.i
    }
}


pub struct Caw {
    akn: Vec<Rect>,
    asw: bool,
    wf: u32,
    aav: u32,
}

impl Caw {
    pub fn new(d: u32, i: u32) -> Self {
        Self {
            akn: Vec::fc(64),
            asw: true,
            wf: d,
            aav: i,
        }
    }
    
    
    pub fn zbz(&mut self, ha: Rect) {
        if self.asw { return; }
        if ha.d == 0 || ha.i == 0 { return; }
        
        
        for a in 0..self.akn.len() {
            if self.akn[a].jao(&ha) {
                
                let hrj = self.akn[a].far(&ha);
                let xtt = hrj.ahy() as i64 - 
                    (self.akn[a].ahy() + ha.ahy()) as i64;
                
                
                let wpu = self.akn[a].ahy().v(ha.ahy());
                if xtt < (wpu / 2) as i64 {
                    self.akn[a] = hrj;
                    return;
                }
            }
        }
        
        
        if self.akn.len() < 64 {
            self.akn.push(ha);
        } else {
            
            self.asw = true;
        }
    }
    
    
    pub fn olb(&mut self) {
        self.asw = true;
    }
    
    
    pub fn clear(&mut self) {
        self.akn.clear();
        self.asw = false;
    }
    
    
    pub fn tdi(&self) -> &[Rect] {
        &self.akn
    }
    
    
    pub fn bex(&self) -> bool {
        self.asw
    }
}
