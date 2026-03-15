

use core::sync::atomic::{AtomicU64, AtomicBool, Ordering};

static BER_: AtomicU64 = AtomicU64::new(0);
static ADH_: AtomicBool = AtomicBool::new(false);
static AVO_: AtomicBool = AtomicBool::new(false);


pub fn init() {
    #[cfg(target_arch = "x86_64")]
    let cbg = unsafe {
        let ecx: u32;
        
        core::arch::asm!(
            "push rbx",
            "cpuid",
            "pop rbx",
            in("eax") 1u32,
            lateout("eax") _,
            lateout("ecx") ecx,
            lateout("edx") _,
        );
        (ecx >> 30) & 1 == 1
    };
    #[cfg(not(target_arch = "x86_64"))]
    let cbg = false;
    #[cfg(target_arch = "x86_64")]
    let cmc = unsafe {
        let ish: u32;
        
        core::arch::asm!(
            "push rbx",
            "cpuid",
            "mov {out:e}, ebx",
            "pop rbx",
            in("eax") 7u32,
            in("ecx") 0u32,
            bd = bd(reg) ish,
            lateout("eax") _,
            lateout("edx") _,
        );
        (ish >> 18) & 1 == 1
    };
    #[cfg(not(target_arch = "x86_64"))]
    let cmc = false;
    ADH_.store(cbg, Ordering::Relaxed);
    AVO_.store(cmc, Ordering::Relaxed);
    crate::serial_println!("[RNG] RDRAND={} RDSEED={}", cbg, cmc);
}

fn wgk() -> u64 {
    let qb = crate::logger::lh();
    let os = crate::rtc::cgz();
    let wbc = ((os.ccq as u64) << 48)
        ^ ((os.caw as u64) << 40)
        ^ ((os.cjw as u64) << 32)
        ^ ((os.bek as u64) << 24)
        ^ ((os.bri as u64) << 16)
        ^ ((os.chr as u64) << 8);
    let lxl = crate::arch::aea();
    lxl ^ qb ^ wbc ^ 0x9E3779B97F4A7C15
}



pub fn hsw() -> u64 {
    let mut g = BER_.load(Ordering::Relaxed);
    if g == 0 {
        g = wgk();
    }

    
    g ^= g >> 12;
    g ^= g << 25;
    g ^= g >> 27;
    g = g.hx(0x2545F4914F6CDD1D);

    BER_.store(g, Ordering::Relaxed);
    g
}

pub fn ntq(k: &mut [u8]) {
    let mut a = 0;
    while a < k.len() {
        let jj = hsw().ho();
        for &o in &jj {
            if a >= k.len() {
                break;
            }
            k[a] = o;
            a += 1;
        }
    }
}


pub fn ozi() -> u8 {
    hsw() as u8
}


pub fn zhh() -> u32 {
    hsw() as u32
}





fn vrb() -> Option<u64> {
    #[cfg(not(target_arch = "x86_64"))]
    return None;

    #[cfg(target_arch = "x86_64")]
    {
        if !ADH_.load(Ordering::Relaxed) {
            return None;
        }
        for _ in 0..10 {
            let ap: u64;
            let bq: u8;
            unsafe {
                core::arch::asm!(
                    "rdrand {v}",
                    "setc {ok}",
                    p = bd(reg) ap,
                    bq = bd(reg_byte) bq,
                );
            }
            if bq != 0 {
                return Some(ap);
            }
        }
        None
    }
}


fn vrc() -> Option<u64> {
    #[cfg(not(target_arch = "x86_64"))]
    return None;

    #[cfg(target_arch = "x86_64")]
    {
        if !AVO_.load(Ordering::Relaxed) {
            return None;
        }
        for _ in 0..10 {
            let ap: u64;
            let bq: u8;
            unsafe {
                core::arch::asm!(
                    "rdseed {v}",
                    "setc {ok}",
                    p = bd(reg) ap,
                    bq = bd(reg_byte) bq,
                );
            }
            if bq != 0 {
                return Some(ap);
            }
        }
        None
    }
}



pub fn phj() -> u64 {
    if let Some(p) = vrc() {
        return p;
    }
    if let Some(p) = vrb() {
        return p;
    }
    
    let wi = crate::arch::aea();
    hsw() ^ wi
}


pub fn wgg() -> u32 {
    phj() as u32
}


pub fn phh(k: &mut [u8]) {
    let mut a = 0;
    while a < k.len() {
        let jj = phj().ho();
        for &o in &jj {
            if a >= k.len() {
                break;
            }
            k[a] = o;
            a += 1;
        }
    }
}


pub fn tmr() -> bool {
    ADH_.load(Ordering::Relaxed)
}
