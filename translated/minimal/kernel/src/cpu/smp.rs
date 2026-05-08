




use core::sync::atomic::{AtomicU32, AtomicBool, Ordering};
use alloc::vec::Vec;
use spin::Mutex;


pub const AR_: usize = 64;


static AQZ_: AtomicU32 = AtomicU32::new(1);


static AIR_: AtomicU32 = AtomicU32::new(1);






const BAE_: u64 = 0xFEE0_0000;

const DUF_: u64 = 0x300;

const DUE_: u64 = 0x310;



fn ons(target_apic_id: u32, vector: u8) {
    if crate::apic::lq() {
        crate::apic::gtx(target_apic_id, vector);
    }
}


pub fn ptm() {
    if crate::apic::lq() {
        crate::apic::jel(0xFE);
    }
}



pub fn jeo(target_cpu: u32) {
    let apic_id = unsafe { CI_[target_cpu as usize].apic_id };
    if crate::apic::lq() && apic_id != 0 || target_cpu != 0 {
        ons(apic_id, 0xFD);
    }
}


static ACF_: [AtomicBool; AR_] = {
    const Bm: AtomicBool = AtomicBool::new(false);
    [Bm; AR_]
};


static AOD_: AtomicU32 = AtomicU32::new(0);


#[repr(C)]
pub struct PerCpuData {
    
    pub cpu_id: u32,
    
    pub apic_id: u32,
    
    pub byk: u64,
    
    pub interrupt_depth: u32,
    
    pub kernel_stack: u64,
    
    pub tsc_last: u64,
    
    pub work_completed: u64,
}

impl PerCpuData {
    pub const fn new(cpu_id: u32, apic_id: u32) -> Self {
        Self {
            cpu_id,
            apic_id,
            byk: 0,
            interrupt_depth: 0,
            kernel_stack: 0,
            tsc_last: 0,
            work_completed: 0,
        }
    }
}


static mut CI_: [PerCpuData; AR_] = {
    const Bm: PerCpuData = PerCpuData::new(0, 0);
    [Bm; AR_]
};


pub fn bll() -> u32 {
    let st = unsafe { core::arch::x86_64::__cpuid(1) };
    let apic_id = ((st.ebx >> 24) & 0xFF) as u32;
    
    
    for i in 0..cpu_count() as usize {
        if unsafe { CI_[i].apic_id == apic_id } {
            return i as u32;
        }
    }
    0 
}


pub fn current() -> &'static PerCpuData {
    let id = bll() as usize;
    unsafe { &CI_[id.min(AR_ - 1)] }
}


pub fn lam() -> &'static mut PerCpuData {
    let id = bll() as usize;
    unsafe { &mut CI_[id.min(AR_ - 1)] }
}


pub fn cpu_count() -> u32 {
    AQZ_.load(Ordering::Relaxed)
}


pub fn jfb(count: u32) {
    AQZ_.store(count, Ordering::Release);
}


pub fn ail() -> u32 {
    AIR_.load(Ordering::Acquire)
}


pub fn gds(cpu_id: u32) -> bool {
    if (cpu_id as usize) < AR_ {
        ACF_[cpu_id as usize].load(Ordering::Relaxed)
    } else {
        false
    }
}


pub fn init() {
    
    ACF_[0].store(true, Ordering::Release);
    AIR_.store(1, Ordering::Release);
    
    
    unsafe {
        CI_[0].cpu_id = 0;
        CI_[0].apic_id = ibf();
        CI_[0].work_completed = 0;
    }
    
    AOD_.store(unsafe { CI_[0].apic_id }, Ordering::Release);
    
    crate::serial_println!("[SMP] BSP initialized (APIC ID: {})", unsafe { CI_[0].apic_id });
}


fn ibf() -> u32 {
    let st = unsafe { core::arch::x86_64::__cpuid(1) };
    ((st.ebx >> 24) & 0xFF) as u32
}


pub struct Ve {
    pub cpu_count: u32,
    pub cuj: u32,
    pub ap_apic_ids: Vec<u32>,
}

impl Ve {
    
    pub fn bfx() -> Self {
        let cuj = ibf();
        
        
        if let Some(ctg) = crate::acpi::rk() {
            let mut fhd = Vec::new();
            for lapic in &ctg.local_apics {
                if lapic.enabled && lapic.apic_id != cuj {
                    fhd.push(lapic.apic_id);
                }
            }
            
            Self {
                cpu_count: (1 + fhd.len()) as u32,
                cuj,
                ap_apic_ids: fhd,
            }
        } else {
            
            Self {
                cpu_count: 1,
                cuj,
                ap_apic_ids: Vec::new(),
            }
        }
    }
}


pub fn gof() {
    let npw = if eru() { "ON " } else { "OFF" };
    crate::println!("╔══════════════════════════════════════╗");
    crate::println!("║          SMP STATUS                  ║");
    crate::println!("╠══════════════════════════════════════╣");
    crate::println!("║ Parallel:    {}                      ║", npw);
    crate::println!("║ BSP APIC ID: {:3}                     ║", AOD_.load(Ordering::Relaxed));
    crate::println!("║ Total CPUs:  {:3}                     ║", cpu_count());
    crate::println!("║ Ready CPUs:  {:3}                     ║", ail());
    crate::println!("╠══════════════════════════════════════╣");
    
    for i in 0..cpu_count().min(AR_ as u32) as usize {
        let ready = if gds(i as u32) { "✓" } else { "✗" };
        let bka = unsafe { CI_[i].work_completed };
        crate::println!("║ CPU {:2}: {} APIC {:3}  Work: {:8}  ║", 
            i, ready, unsafe { CI_[i].apic_id }, bka);
    }
    crate::println!("╚══════════════════════════════════════╝");
}


pub fn get_stats() -> (u32, u32, u64) {
    let av = cpu_count();
    let ready = ail();
    let mut joe = 0u64;
    
    for i in 0..av as usize {
        joe += unsafe { CI_[i].work_completed };
    }
    
    (av, ready, joe)
}






pub type Nz = fn(usize, usize, *mut u8);


#[derive(Clone, Copy)]
pub struct WorkItem {
    pub func: Option<Nz>,
    pub start: usize,
    pub end: usize,
    pub data: *mut u8,
}

unsafe impl Send for WorkItem {}
unsafe impl Sync for WorkItem {}

impl WorkItem {
    pub const fn empty() -> Self {
        Self { func: None, start: 0, end: 0, data: core::ptr::null_mut() }
    }
}


static BLG_: [Mutex<WorkItem>; AR_] = {
    const Bm: Mutex<WorkItem> = Mutex::new(WorkItem::empty());
    [Bm; AR_]
};


static ZV_: [AtomicBool; AR_] = {
    const Bm: AtomicBool = AtomicBool::new(false);
    [Bm; AR_]
};


static RO_: AtomicU32 = AtomicU32::new(0);


static YL_: AtomicBool = AtomicBool::new(true);


const CLW_: usize = 32;



const CJG_: u32 = 1_000_000;


pub fn elh() {
    YL_.store(true, Ordering::Release);
    crate::serial_println!("[SMP] Parallelism ENABLED");
}


pub fn fsj() {
    YL_.store(false, Ordering::Release);
    crate::serial_println!("[SMP] Parallelism DISABLED");
}


pub fn eru() -> bool {
    YL_.load(Ordering::Relaxed)
}







pub fn bcz(total_items: usize, func: Nz, data: *mut u8) {
    
    if !YL_.load(Ordering::Relaxed) {
        func(0, total_items, data);
        return;
    }
    
    let num_cpus = ail() as usize;
    
    
    
    
    if num_cpus <= 1 || total_items < CLW_ {
        func(0, total_items, data);
        return;
    }
    
    
    let rs = (total_items + num_cpus - 1) / num_cpus;
    RO_.store(0, Ordering::Release);
    
    
    let mut fsk = 0usize;
    for cpu_id in 1..num_cpus {
        if !gds(cpu_id as u32) { continue; }
        
        let start = cpu_id * rs;
        if start >= total_items { break; }
        let end = ((cpu_id + 1) * rs).min(total_items);
        
        
        {
            let mut bka = BLG_[cpu_id].lock();
            bka.func = Some(func);
            bka.start = start;
            bka.end = end;
            bka.data = data;
        }
        
        ZV_[cpu_id].store(true, Ordering::Release);
        fsk += 1;
    }
    
    
    
    
    
    func(0, rs.min(total_items), data);
    RO_.fetch_add(1, Ordering::Release);
    
    
    if fsk > 0 {
        let expected = fsk as u32 + 1;
        let mut eab = 0u32;
        
        while RO_.load(Ordering::Acquire) < expected {
            core::hint::spin_loop();
            eab += 1;
            
            
            if eab > CJG_ {
                crate::serial_println!("[SMP] WARNING: Timeout waiting for APs, completed {}/{}", 
                    RO_.load(Ordering::Relaxed), expected);
                
                
                for cancel_id in 1..num_cpus {
                    ZV_[cancel_id].store(false, Ordering::Release);
                }
                break;
            }
        }
    }
}


fn kis(cpu_id: usize) {
    if ZV_[cpu_id].load(Ordering::Acquire) {
        let bka = { *BLG_[cpu_id].lock() };
        ZV_[cpu_id].store(false, Ordering::Release);
        
        if let Some(func) = bka.func {
            func(bka.start, bka.end, bka.data);
            unsafe { CI_[cpu_id].work_completed += 1; }
        }
        
        RO_.fetch_add(1, Ordering::Release);
    }
}
















pub unsafe extern "C" fn hfk(smp_info: &limine::smp::Cpu) -> ! {
    
    let processor_id = smp_info.id as usize;
    let lapic_id = smp_info.lapic_id;
    
    
    if processor_id < AR_ {
        CI_[processor_id].cpu_id = processor_id as u32;
        CI_[processor_id].apic_id = lapic_id;
        CI_[processor_id].work_completed = 0;
    }
    
    
    crate::cpu::simd::fuo();
    
    
    
    crate::gdt::cau(processor_id as u32);
    
    
    
    crate::interrupts::nad();
    
    
    
    
    crate::sync::percpu::cau(processor_id as u32);
    
    
    
    crate::thread::mow(processor_id as u32);
    
    
    
    crate::apic::cau();
    
    
    ACF_[processor_id].store(true, Ordering::Release);
    AIR_.fetch_add(1, Ordering::Release);
    
    crate::serial_println!("[SMP] AP {} fully online (LAPIC ID: {})", processor_id, lapic_id);
    
    
    
    
    loop {
        
        kis(processor_id);
        
        
        
        
        core::arch::asm!("sti; hlt", options(nomem, nostack));
    }
}
