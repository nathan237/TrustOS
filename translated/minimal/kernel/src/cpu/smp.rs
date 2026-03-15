




use core::sync::atomic::{AtomicU32, AtomicBool, Ordering};
use alloc::vec::Vec;
use spin::Mutex;


pub const AN_: usize = 64;


static AOZ_: AtomicU32 = AtomicU32::new(1);


static AGX_: AtomicU32 = AtomicU32::new(1);






const AYD_: u64 = 0xFEE0_0000;

const DQL_: u64 = 0x300;

const DQK_: u64 = 0x310;



fn whg(mjw: u32, wj: u8) {
    if crate::apic::zu() {
        crate::apic::mds(mjw, wj);
    }
}


pub fn xtm() {
    if crate::apic::zu() {
        crate::apic::phu(0xFE);
    }
}



pub fn phx(cih: u32) {
    let aed = unsafe { CG_[cih as usize].aed };
    if crate::apic::zu() && aed != 0 || cih != 0 {
        whg(aed, 0xFD);
    }
}


static AAS_: [AtomicBool; AN_] = {
    const Dm: AtomicBool = AtomicBool::new(false);
    [Dm; AN_]
};


static ALZ_: AtomicU32 = AtomicU32::new(0);


#[repr(C)]
pub struct PerCpuData {
    
    pub qq: u32,
    
    pub aed: u32,
    
    pub eoh: u64,
    
    pub flu: u32,
    
    pub bhg: u64,
    
    pub mni: u64,
    
    pub dng: u64,
}

impl PerCpuData {
    pub const fn new(qq: u32, aed: u32) -> Self {
        Self {
            qq,
            aed,
            eoh: 0,
            flu: 0,
            bhg: 0,
            mni: 0,
            dng: 0,
        }
    }
}


static mut CG_: [PerCpuData; AN_] = {
    const Dm: PerCpuData = PerCpuData::new(0, 0);
    [Dm; AN_]
};


pub fn ead() -> u32 {
    let ipl = unsafe { core::arch::x86_64::ddo(1) };
    let aed = ((ipl.ebx >> 24) & 0xFF) as u32;
    
    
    for a in 0..aao() as usize {
        if unsafe { CG_[a].aed == aed } {
            return a as u32;
        }
    }
    0 
}


pub fn cv() -> &'static PerCpuData {
    let ad = ead() as usize;
    unsafe { &CG_[ad.v(AN_ - 1)] }
}


pub fn rry() -> &'static mut PerCpuData {
    let ad = ead() as usize;
    unsafe { &mut CG_[ad.v(AN_ - 1)] }
}


pub fn aao() -> u32 {
    AOZ_.load(Ordering::Relaxed)
}


pub fn piv(az: u32) {
    AOZ_.store(az, Ordering::Release);
}


pub fn boc() -> u32 {
    AGX_.load(Ordering::Acquire)
}


pub fn lga(qq: u32) -> bool {
    if (qq as usize) < AN_ {
        AAS_[qq as usize].load(Ordering::Relaxed)
    } else {
        false
    }
}


pub fn init() {
    
    AAS_[0].store(true, Ordering::Release);
    AGX_.store(1, Ordering::Release);
    
    
    unsafe {
        CG_[0].qq = 0;
        CG_[0].aed = nxq();
        CG_[0].dng = 0;
    }
    
    ALZ_.store(unsafe { CG_[0].aed }, Ordering::Release);
    
    crate::serial_println!("[SMP] BSP initialized (APIC ID: {})", unsafe { CG_[0].aed });
}


fn nxq() -> u32 {
    let ipl = unsafe { core::arch::x86_64::ddo(1) };
    ((ipl.ebx >> 24) & 0xFF) as u32
}


pub struct Aza {
    pub aao: u32,
    pub gbo: u32,
    pub iju: Vec<u32>,
}

impl Aza {
    
    pub fn dgf() -> Self {
        let gbo = nxq();
        
        
        if let Some(fzr) = crate::acpi::ani() {
            let mut kaq = Vec::new();
            for ku in &fzr.dja {
                if ku.iq && ku.aed != gbo {
                    kaq.push(ku.aed);
                }
            }
            
            Self {
                aao: (1 + kaq.len()) as u32,
                gbo,
                iju: kaq,
            }
        } else {
            
            Self {
                aao: 1,
                gbo,
                iju: Vec::new(),
            }
        }
    }
}


pub fn lvm() {
    let vbn = if jbt() { "ON " } else { "OFF" };
    crate::println!("╔══════════════════════════════════════╗");
    crate::println!("║          SMP STATUS                  ║");
    crate::println!("╠══════════════════════════════════════╣");
    crate::println!("║ Parallel:    {}                      ║", vbn);
    crate::println!("║ BSP APIC ID: {:3}                     ║", ALZ_.load(Ordering::Relaxed));
    crate::println!("║ Total CPUs:  {:3}                     ║", aao());
    crate::println!("║ Ready CPUs:  {:3}                     ║", boc());
    crate::println!("╠══════════════════════════════════════╣");
    
    for a in 0..aao().v(AN_ as u32) as usize {
        let ack = if lga(a as u32) { "✓" } else { "✗" };
        let dnf = unsafe { CG_[a].dng };
        crate::println!("║ CPU {:2}: {} APIC {:3}  Work: {:8}  ║", 
            a, ack, unsafe { CG_[a].aed }, dnf);
    }
    crate::println!("╚══════════════════════════════════════╝");
}


pub fn asx() -> (u32, u32, u64) {
    let es = aao();
    let ack = boc();
    let mut pvk = 0u64;
    
    for a in 0..es as usize {
        pvk += unsafe { CG_[a].dng };
    }
    
    (es, ack, pvk)
}






pub type Afv = fn(usize, usize, *mut u8);


#[derive(Clone, Copy)]
pub struct WorkItem {
    pub ke: Option<Afv>,
    pub ay: usize,
    pub ci: usize,
    pub f: *mut u8,
}

unsafe impl Send for WorkItem {}
unsafe impl Sync for WorkItem {}

impl WorkItem {
    pub const fn azs() -> Self {
        Self { ke: None, ay: 0, ci: 0, f: core::ptr::null_mut() }
    }
}


static BJB_: [Mutex<WorkItem>; AN_] = {
    const Dm: Mutex<WorkItem> = Mutex::new(WorkItem::azs());
    [Dm; AN_]
};


static YQ_: [AtomicBool; AN_] = {
    const Dm: AtomicBool = AtomicBool::new(false);
    [Dm; AN_]
};


static QT_: AtomicU32 = AtomicU32::new(0);


static XE_: AtomicBool = AtomicBool::new(true);


const CIN_: usize = 32;



const CFW_: u32 = 1_000_000;


pub fn isq() {
    XE_.store(true, Ordering::Release);
    crate::serial_println!("[SMP] Parallelism ENABLED");
}


pub fn kqd() {
    XE_.store(false, Ordering::Release);
    crate::serial_println!("[SMP] Parallelism DISABLED");
}


pub fn jbt() -> bool {
    XE_.load(Ordering::Relaxed)
}







pub fn daj(ejz: usize, ke: Afv, f: *mut u8) {
    
    if !XE_.load(Ordering::Relaxed) {
        ke(0, ejz, f);
        return;
    }
    
    let bcc = boc() as usize;
    
    
    
    
    if bcc <= 1 || ejz < CIN_ {
        ke(0, ejz, f);
        return;
    }
    
    
    let aiw = (ejz + bcc - 1) / bcc;
    QT_.store(0, Ordering::Release);
    
    
    let mut kqe = 0usize;
    for qq in 1..bcc {
        if !lga(qq as u32) { continue; }
        
        let ay = qq * aiw;
        if ay >= ejz { break; }
        let ci = ((qq + 1) * aiw).v(ejz);
        
        
        {
            let mut dnf = BJB_[qq].lock();
            dnf.ke = Some(ke);
            dnf.ay = ay;
            dnf.ci = ci;
            dnf.f = f;
        }
        
        YQ_[qq].store(true, Ordering::Release);
        kqe += 1;
    }
    
    
    
    
    
    ke(0, aiw.v(ejz), f);
    QT_.fetch_add(1, Ordering::Release);
    
    
    if kqe > 0 {
        let qy = kqe as u32 + 1;
        let mut ibf = 0u32;
        
        while QT_.load(Ordering::Acquire) < qy {
            core::hint::hc();
            ibf += 1;
            
            
            if ibf > CFW_ {
                crate::serial_println!("[SMP] WARNING: Timeout waiting for APs, completed {}/{}", 
                    QT_.load(Ordering::Relaxed), qy);
                
                
                for qvw in 1..bcc {
                    YQ_[qvw].store(false, Ordering::Release);
                }
                break;
            }
        }
    }
}


fn qyk(qq: usize) {
    if YQ_[qq].load(Ordering::Acquire) {
        let dnf = { *BJB_[qq].lock() };
        YQ_[qq].store(false, Ordering::Release);
        
        if let Some(ke) = dnf.ke {
            ke(dnf.ay, dnf.ci, dnf.f);
            unsafe { CG_[qq].dng += 1; }
        }
        
        QT_.fetch_add(1, Ordering::Release);
    }
}
















pub unsafe extern "C" fn mvx(plp: &limine::smp::Cpu) -> ! {
    
    let bny = plp.ad as usize;
    let ett = plp.ett;
    
    
    if bny < AN_ {
        CG_[bny].qq = bny as u32;
        CG_[bny].aed = ett;
        CG_[bny].dng = 0;
    }
    
    
    crate::cpu::simd::ktg();
    
    
    
    crate::gdt::eso(bny as u32);
    
    
    
    crate::interrupts::ugz();
    
    
    
    
    crate::sync::percpu::eso(bny as u32);
    
    
    
    crate::thread::ttb(bny as u32);
    
    
    
    crate::apic::eso();
    
    
    AAS_[bny].store(true, Ordering::Release);
    AGX_.fetch_add(1, Ordering::Release);
    
    crate::serial_println!("[SMP] AP {} fully online (LAPIC ID: {})", bny, ett);
    
    
    
    
    loop {
        
        qyk(bny);
        
        
        
        
        core::arch::asm!("sti; hlt", options(nomem, nostack));
    }
}
