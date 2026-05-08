






use alloc::collections::BTreeSet;
use spin::Mutex;
use core::sync::atomic::{AtomicBool, AtomicU16, Ordering};


pub const BCH_: u16 = 65535;


pub const DTD_: u16 = 0;


static BKY_: AtomicBool = AtomicBool::new(false);


static BDO_: AtomicU16 = AtomicU16::new(1);


static AAD_: Mutex<BTreeSet<u16>> = Mutex::new(BTreeSet::new());






#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(u64)]
pub enum InvvpidType {
    
    IndividualAddress = 0,
    
    SingleContext = 1,
    
    AllContext = 2,
    
    SingleContextRetainGlobal = 3,
}


#[repr(C, packed)]
#[derive(Clone, Copy)]
pub struct InvvpidDescriptor {
    pub vpid: u16,
    pub reserved: [u8; 6],
    pub linear_address: u64,
}

impl InvvpidDescriptor {
    pub fn new(vpid: u16, addr: u64) -> Self {
        InvvpidDescriptor {
            vpid,
            reserved: [0; 6],
            linear_address: addr,
        }
    }
    
    pub fn hzn(vpid: u16) -> Self {
        Self::new(vpid, 0)
    }
}






pub fn init() -> bool {
    
    let supported = kjy();
    
    if supported {
        BKY_.store(true, Ordering::SeqCst);
        crate::serial_println!("[VPID] VPID support enabled - TLB isolation active");
    } else {
        crate::serial_println!("[VPID] VPID not supported - TLB flush on every exit");
    }
    
    supported
}


fn kjy() -> bool {
    
    let kzu = super::vmx::ach(0x48B); 
    let jvc = (kzu >> 32) as u32;
    
    
    (jvc & (1 << 5)) != 0
}


pub fn lq() -> bool {
    BKY_.load(Ordering::SeqCst)
}


pub fn allocate() -> Option<u16> {
    if !lq() {
        return None;
    }
    
    let mut bkr = AAD_.lock();
    
    
    let start = BDO_.load(Ordering::SeqCst);
    
    for offset in 0..BCH_ {
        let vpid = ((start as u32 + offset as u32) % (BCH_ as u32)) as u16;
        
        
        if vpid == 0 {
            continue;
        }
        
        if !bkr.contains(&vpid) {
            bkr.insert(vpid);
            BDO_.store(vpid.wrapping_add(1).max(1), Ordering::SeqCst);
            
            crate::serial_println!("[VPID] Allocated VPID {} for new VM", vpid);
            return Some(vpid);
        }
    }
    
    
    crate::serial_println!("[VPID] ERROR: All VPIDs exhausted!");
    None
}


pub fn free(vpid: u16) {
    if vpid == 0 {
        return; 
    }
    
    let mut bkr = AAD_.lock();
    if bkr.remove(&vpid) {
        crate::serial_println!("[VPID] Freed VPID {}", vpid);
        
        
        mrm(vpid);
    }
}


pub fn jvb() -> usize {
    AAD_.lock().len()
}






pub fn mrm(vpid: u16) {
    if !lq() {
        return;
    }
    
    let desc = InvvpidDescriptor::hzn(vpid);
    
    unsafe {
        gdg(InvvpidType::SingleContext, &desc);
    }
}


pub fn qlu(vpid: u16, addr: u64) {
    if !lq() {
        return;
    }
    
    let desc = InvvpidDescriptor::new(vpid, addr);
    
    unsafe {
        gdg(InvvpidType::IndividualAddress, &desc);
    }
}


pub fn qlv() {
    if !lq() {
        return;
    }
    
    let desc = InvvpidDescriptor::hzn(0);
    
    unsafe {
        gdg(InvvpidType::AllContext, &desc);
    }
}






#[inline]
unsafe fn gdg(inv_type: InvvpidType, desc: &InvvpidDescriptor) {
    let result: u8;
    
    core::arch::asm!(
        "invvpid {0}, [{1}]",
        "setc {2}",
        in(reg) inv_type as u64,
        in(reg) desc as *const InvvpidDescriptor,
        out(reg_byte) result,
        options(nostack)
    );
    
    if result != 0 {
        crate::serial_println!("[VPID] INVVPID failed! type={:?}", inv_type);
    }
}






pub fn meb(vpid: Option<u16>) -> u64 {
    match vpid {
        Some(v) if lq() => v as u64,
        _ => 0, 
    }
}


pub fn mdt() -> u64 {
    if lq() {
        1 << 5 
    } else {
        0
    }
}






#[derive(Debug, Clone, Default)]
pub struct Vx {
    pub bkr: usize,
    pub bzz: usize,
    pub invalidations: usize,
}

static DFB_: Mutex<Vx> = Mutex::new(Vx {
    bkr: 0,
    bzz: 0,
    invalidations: 0,
});


pub fn get_stats() -> Vx {
    DFB_.lock().clone()
}
