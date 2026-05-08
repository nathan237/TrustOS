







use alloc::boxed::Box;
use alloc::collections::BTreeMap;
use alloc::vec::Vec;
use spin::Mutex;
use core::sync::atomic::{AtomicBool, AtomicU64, AtomicU32, Ordering};


const BO_: u64 = 4096;


const AGZ_: usize = 65536;


pub type Ep = u32;


#[derive(Clone, Copy, Debug)]
struct Vg {
    
    phys_addr: u64,
    
    slot: Ep,
    
    cr3: u64,
    
    virt_addr: u64,
    
    access_count: u32,
    
    last_access: u64,
}


struct Jc {
    
    enabled: bool,
    
    swap_path: Option<&'static str>,
    
    slot_bitmap: Vec<bool>,
    
    total_slots: usize,
    
    used_slots: usize,
    
    swap_map: BTreeMap<(u64, u64), Ep>,
    
    page_tracker: BTreeMap<(u64, u64), Vg>,
}

static Ga: Mutex<Jc> = Mutex::new(Jc {
    enabled: false,
    swap_path: None,
    slot_bitmap: Vec::new(),
    total_slots: 0,
    used_slots: 0,
    swap_map: BTreeMap::new(),
    page_tracker: BTreeMap::new(),
});


static BER_: AtomicU64 = AtomicU64::new(0);
static BEQ_: AtomicU64 = AtomicU64::new(0);
static JU_: AtomicBool = AtomicBool::new(false);


pub fn init(swap_path: &'static str, size_bytes: u64) {
    let azs = (size_bytes / BO_) as usize;
    let azs = azs.min(AGZ_);
    
    let mut state = Ga.lock();
    state.slot_bitmap = alloc::vec![false; azs];
    state.total_slots = azs;
    state.used_slots = 0;
    state.enabled = true;
    state.swap_path = Some(swap_path);
    
    JU_.store(true, Ordering::SeqCst);
    crate::serial_println!("[SWAP] Initialized: {} slots ({} MB), path={}",
        azs, (azs * 4096) / (1024 * 1024), swap_path);
}


pub fn qet(max_pages: usize) {
    let azs = max_pages.min(AGZ_);
    let mut state = Ga.lock();
    state.slot_bitmap = alloc::vec![false; azs];
    state.total_slots = azs;
    state.used_slots = 0;
    state.enabled = true;
    JU_.store(true, Ordering::SeqCst);
    crate::serial_println!("[SWAP] Anonymous swap: {} slots ({} KB)", azs, azs * 4);
}


pub fn ozc(path: &str) -> Result<(), &'static str> {
    
    let size = 64 * 1024 * 1024u64;
    
    let owt: &'static str = Box::leak(alloc::string::String::from(path).into_boxed_str());
    init(owt, size);
    Ok(())
}


pub fn ozb(jsq: &str) -> Result<(), &'static str> {
    let mut state = Ga.lock();
    if !state.enabled {
        return Err("Swap not enabled");
    }
    
    
    state.enabled = false;
    state.used_slots = 0;
    state.swap_map.clear();
    JU_.store(false, Ordering::SeqCst);
    crate::serial_println!("[SWAP] Disabled");
    Ok(())
}


pub fn lq() -> bool {
    JU_.load(Ordering::Relaxed)
}


fn jux(state: &mut Jc) -> Option<Ep> {
    for (i, used) in state.slot_bitmap.iter_mut().enumerate() {
        if !*used {
            *used = true;
            state.used_slots += 1;
            return Some((i + 1) as Ep); 
        }
    }
    None
}


fn iaa(state: &mut Jc, slot: Ep) {
    if slot == 0 { return; }
    let idx = (slot - 1) as usize;
    if idx < state.slot_bitmap.len() {
        state.slot_bitmap[idx] = false;
        state.used_slots = state.used_slots.saturating_sub(1);
    }
}


pub fn pmx(cr3: u64, virt_addr: u64, phys_addr: u64) {
    if !JU_.load(Ordering::Relaxed) { return; }
    
    let key = (cr3, virt_addr & !0xFFF);
    let entry = Vg {
        phys_addr,
        slot: 0,
        cr3,
        virt_addr: virt_addr & !0xFFF,
        access_count: 1,
        last_access: crate::logger::eg(),
    };
    
    Ga.lock().page_tracker.insert(key, entry);
}


pub fn rau(cr3: u64, virt_addr: u64) {
    if !JU_.load(Ordering::Relaxed) { return; }
    
    let key = (cr3, virt_addr & !0xFFF);
    let mut state = Ga.lock();
    if let Some(entry) = state.page_tracker.get_mut(&key) {
        entry.access_count = entry.access_count.saturating_add(1);
        entry.last_access = crate::logger::eg();
    }
}


pub fn rbo(cr3: u64, virt_addr: u64) {
    if !JU_.load(Ordering::Relaxed) { return; }
    
    let key = (cr3, virt_addr & !0xFFF);
    let mut state = Ga.lock();
    if let Some(entry) = state.page_tracker.remove(&key) {
        if entry.slot != 0 {
            iaa(&mut state, entry.slot);
        }
    }
    state.swap_map.remove(&key);
}



fn onh(state: &Jc) -> Option<(u64, u64, u64)> {
    let mut adj: Option<(&(u64, u64), &Vg)> = None;
    let mut djb = u64::MAX;
    
    for (key, entry) in state.page_tracker.iter() {
        
        if entry.phys_addr == 0 { continue; }
        
        if entry.cr3 == 0 { continue; }
        
        
        
        let score = entry.last_access.saturating_mul(entry.access_count as u64 + 1);
        if score < djb {
            djb = score;
            adj = Some((key, entry));
        }
    }
    
    adj.map(|(_, entry)| (entry.cr3, entry.virt_addr, entry.phys_addr))
}



pub fn pnu() -> Option<u64> {
    let mut state = Ga.lock();
    if !state.enabled { return None; }
    
    let (cr3, virt_addr, phys_addr) = onh(&state)?;
    let slot = jux(&mut state)?;
    
    
    
    pve(&state, slot, phys_addr);
    
    
    let key = (cr3, virt_addr);
    if let Some(entry) = state.page_tracker.get_mut(&key) {
        entry.phys_addr = 0; 
        entry.slot = slot;
    }
    state.swap_map.insert(key, slot);
    
    
    
    ppr(cr3, virt_addr, slot);
    
    BER_.fetch_add(1, Ordering::Relaxed);
    
    crate::log_debug!("[SWAP] Evicted page cr3={:#x} virt={:#x} -> slot {}",
        cr3, virt_addr, slot);
    
    Some(phys_addr)
}



pub fn mim(cr3: u64, aff: u64) -> bool {
    let hbn = aff & !0xFFF;
    let key = (cr3, hbn);
    
    let mut state = Ga.lock();
    let slot = match state.swap_map.get(&key) {
        Some(&j) => j,
        None => return false,
    };
    
    if slot == 0 { return false; }
    
    
    let cnd = match crate::memory::frame::aan() {
        Some(f) => f,
        None => return false, 
    };
    
    
    odf(&state, slot, cnd);
    
    
    if let Some(entry) = state.page_tracker.get_mut(&key) {
        entry.phys_addr = cnd;
        let nmv = entry.slot;
        entry.slot = 0;
        entry.access_count = 1;
        entry.last_access = crate::logger::eg();
        iaa(&mut state, nmv);
    }
    state.swap_map.remove(&key);
    
    drop(state);
    
    
    oet(cr3, hbn, cnd);
    
    BEQ_.fetch_add(1, Ordering::Relaxed);
    
    crate::log_debug!("[SWAP] Paged in cr3={:#x} virt={:#x} phys={:#x}",
        cr3, hbn, cnd);
    
    true
}


pub fn stats() -> Afd {
    let state = Ga.lock();
    Afd {
        enabled: state.enabled,
        total_slots: state.total_slots,
        used_slots: state.used_slots,
        pages_swapped_out: BER_.load(Ordering::Relaxed),
        pages_swapped_in: BEQ_.load(Ordering::Relaxed),
        tracked_pages: state.page_tracker.len(),
    }
}

#[derive(Clone, Debug)]
pub struct Afd {
    pub enabled: bool,
    pub total_slots: usize,
    pub used_slots: usize,
    pub pages_swapped_out: u64,
    pub pages_swapped_in: u64,
    pub tracked_pages: usize,
}









static BIU_: Mutex<BTreeMap<Ep, Vec<u8>>> = Mutex::new(BTreeMap::new());




const QF_: u64 = 8;


fn jkb() -> u64 {
    let cap = crate::nvme::capacity();
    let jkc = (AGZ_ as u64) * QF_;
    if cap > jkc {
        cap - jkc
    } else {
        0 
    }
}

fn pve(_state: &Jc, slot: Ep, phys_addr: u64) {
    let bz = crate::memory::hhdm_offset();
    let src = unsafe { core::slice::from_raw_parts((phys_addr + bz) as *const u8, BO_ as usize) };
    
    
    if crate::nvme::is_initialized() {
        let hb = jkb() + ((slot as u64 - 1) * QF_);
        if crate::nvme::write_sectors(hb, QF_ as usize, src).is_ok() {
            return;
        }
    }
    
    
    let mut acg = BIU_.lock();
    acg.insert(slot, src.to_vec());
}

fn odf(_state: &Jc, slot: Ep, phys_addr: u64) {
    let bz = crate::memory::hhdm_offset();
    let dst = unsafe { core::slice::from_raw_parts_mut((phys_addr + bz) as *mut u8, BO_ as usize) };
    
    
    if crate::nvme::is_initialized() {
        let hb = jkb() + ((slot as u64 - 1) * QF_);
        if crate::nvme::read_sectors(hb, QF_ as usize, dst).is_ok() {
            return;
        }
    }
    
    
    let acg = BIU_.lock();
    if let Some(data) = acg.get(&slot) {
        dst[..data.len()].copy_from_slice(data);
    } else {
        dst.fill(0);
    }
}






fn ppr(cr3: u64, virt_addr: u64, slot: Ep) {
    let bz = crate::memory::hhdm_offset();
    
    
    let pml4 = unsafe { &mut *((cr3 + bz) as *mut [u64; 512]) };
    let lu = ((virt_addr >> 39) & 0x1FF) as usize;
    if pml4[lu] & 1 == 0 { return; }
    
    let xz = pml4[lu] & !0xFFF;
    let jt = unsafe { &mut *((xz + bz) as *mut [u64; 512]) };
    let jc = ((virt_addr >> 30) & 0x1FF) as usize;
    if jt[jc] & 1 == 0 { return; }
    
    let aae = jt[jc] & !0xFFF;
    let js = unsafe { &mut *((aae + bz) as *mut [u64; 512]) };
    let iw = ((virt_addr >> 21) & 0x1FF) as usize;
    if js[iw] & 1 == 0 { return; }
    if js[iw] & (1 << 7) != 0 { return; } 
    
    let amj = js[iw] & !0xFFF;
    let jd = unsafe { &mut *((amj + bz) as *mut [u64; 512]) };
    let mw = ((virt_addr >> 12) & 0x1FF) as usize;
    
    
    
    
    
    jd[mw] = ((slot as u64) << 1) | (1u64 << 62);
    
    
    #[cfg(target_arch = "x86_64")]
    unsafe { core::arch::asm!("invlpg [{}]", in(reg) virt_addr, options(nostack, preserves_flags)); }
}


fn oet(cr3: u64, virt_addr: u64, phys_addr: u64) {
    let bz = crate::memory::hhdm_offset();
    
    let pml4 = unsafe { &mut *((cr3 + bz) as *mut [u64; 512]) };
    let lu = ((virt_addr >> 39) & 0x1FF) as usize;
    if pml4[lu] & 1 == 0 { return; }
    
    let xz = pml4[lu] & !0xFFF;
    let jt = unsafe { &mut *((xz + bz) as *mut [u64; 512]) };
    let jc = ((virt_addr >> 30) & 0x1FF) as usize;
    if jt[jc] & 1 == 0 { return; }
    
    let aae = jt[jc] & !0xFFF;
    let js = unsafe { &mut *((aae + bz) as *mut [u64; 512]) };
    let iw = ((virt_addr >> 21) & 0x1FF) as usize;
    if js[iw] & 1 == 0 { return; }
    
    let amj = js[iw] & !0xFFF;
    let jd = unsafe { &mut *((amj + bz) as *mut [u64; 512]) };
    let mw = ((virt_addr >> 12) & 0x1FF) as usize;
    
    
    let flags: u64 = 1 | (1 << 1) | (1 << 2); 
    jd[mw] = (phys_addr & !0xFFF) | flags;
    
    #[cfg(target_arch = "x86_64")]
    unsafe { core::arch::asm!("invlpg [{}]", in(reg) virt_addr, options(nostack, preserves_flags)); }
}


pub fn qmz(pte_value: u64) -> bool {
    (pte_value & 1) == 0 && (pte_value & (1u64 << 62)) != 0
}


pub fn qyf(pte_value: u64) -> Ep {
    ((pte_value >> 1) & 0x7FFF_FFFF) as Ep
}
