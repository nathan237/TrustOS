






use alloc::collections::BTreeSet;
use spin::Mutex;
use core::sync::atomic::{AtomicBool, AtomicU16, Ordering};


pub const BAF_: u16 = 65535;


pub const DPJ_: u16 = 0;


static BIS_: AtomicBool = AtomicBool::new(false);


static BBL_: AtomicU16 = AtomicU16::new(1);


static YY_: Mutex<BTreeSet<u16>> = Mutex::new(BTreeSet::new());






#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(u64)]
pub enum InvvpidType {
    
    Cfu = 0,
    
    Cmz = 1,
    
    Bxv = 2,
    
    Dir = 3,
}


#[repr(C, packed)]
#[derive(Clone, Copy)]
pub struct InvvpidDescriptor {
    pub vpid: u16,
    pub awt: [u8; 6],
    pub ufc: u64,
}

impl InvvpidDescriptor {
    pub fn new(vpid: u16, ag: u64) -> Self {
        InvvpidDescriptor {
            vpid,
            awt: [0; 6],
            ufc: ag,
        }
    }
    
    pub fn nvn(vpid: u16) -> Self {
        Self::new(vpid, 0)
    }
}






pub fn init() -> bool {
    
    let dme = raa();
    
    if dme {
        BIS_.store(true, Ordering::SeqCst);
        crate::serial_println!("[VPID] VPID support enabled - TLB isolation active");
    } else {
        crate::serial_println!("[VPID] VPID not supported - TLB flush on every exit");
    }
    
    dme
}


fn raa() -> bool {
    
    let rrd = super::vmx::bcg(0x48B); 
    let qhe = (rrd >> 32) as u32;
    
    
    (qhe & (1 << 5)) != 0
}


pub fn zu() -> bool {
    BIS_.load(Ordering::SeqCst)
}


pub fn ijo() -> Option<u16> {
    if !zu() {
        return None;
    }
    
    let mut gab = YY_.lock();
    
    
    let ay = BBL_.load(Ordering::SeqCst);
    
    for l in 0..BAF_ {
        let vpid = ((ay as u32 + l as u32) % (BAF_ as u32)) as u16;
        
        
        if vpid == 0 {
            continue;
        }
        
        if !gab.contains(&vpid) {
            gab.insert(vpid);
            BBL_.store(vpid.cn(1).am(1), Ordering::SeqCst);
            
            crate::serial_println!("[VPID] Allocated VPID {} for new VM", vpid);
            return Some(vpid);
        }
    }
    
    
    crate::serial_println!("[VPID] ERROR: All VPIDs exhausted!");
    None
}


pub fn aez(vpid: u16) {
    if vpid == 0 {
        return; 
    }
    
    let mut gab = YY_.lock();
    if gab.remove(&vpid) {
        crate::serial_println!("[VPID] Freed VPID {}", vpid);
        
        
        twa(vpid);
    }
}


pub fn qgz() -> usize {
    YY_.lock().len()
}






pub fn twa(vpid: u16) {
    if !zu() {
        return;
    }
    
    let desc = InvvpidDescriptor::nvn(vpid);
    
    unsafe {
        lfk(InvvpidType::Cmz, &desc);
    }
}


pub fn yyp(vpid: u16, ag: u64) {
    if !zu() {
        return;
    }
    
    let desc = InvvpidDescriptor::new(vpid, ag);
    
    unsafe {
        lfk(InvvpidType::Cfu, &desc);
    }
}


pub fn yyq() {
    if !zu() {
        return;
    }
    
    let desc = InvvpidDescriptor::nvn(0);
    
    unsafe {
        lfk(InvvpidType::Bxv, &desc);
    }
}






#[inline]
unsafe fn lfk(ofg: InvvpidType, desc: &InvvpidDescriptor) {
    let result: u8;
    
    core::arch::asm!(
        "invvpid {0}, [{1}]",
        "setc {2}",
        in(reg) ofg as u64,
        in(reg) desc as *const InvvpidDescriptor,
        bd(reg_byte) result,
        options(nostack)
    );
    
    if result != 0 {
        crate::serial_println!("[VPID] INVVPID failed! type={:?}", ofg);
    }
}






pub fn tfc(vpid: Option<u16>) -> u64 {
    match vpid {
        Some(p) if zu() => p as u64,
        _ => 0, 
    }
}


pub fn tep() -> u64 {
    if zu() {
        1 << 5 
    } else {
        0
    }
}






#[derive(Debug, Clone, Default)]
pub struct Bap {
    pub gab: usize,
    pub equ: usize,
    pub twb: usize,
}

static DBJ_: Mutex<Bap> = Mutex::new(Bap {
    gab: 0,
    equ: 0,
    twb: 0,
});


pub fn asx() -> Bap {
    DBJ_.lock().clone()
}
