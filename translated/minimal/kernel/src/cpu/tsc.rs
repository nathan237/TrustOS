




use core::sync::atomic::{AtomicU64, Ordering};


static FW_: AtomicU64 = AtomicU64::new(0);


static XX_: AtomicU64 = AtomicU64::new(0);


pub fn init(ard: u64) {
    FW_.store(ard, Ordering::Release);
    XX_.store(ow(), Ordering::Release);
    
    crate::serial_println!("[TSC] Initialized: {} Hz ({} GHz)", 
        ard, ard / 1_000_000_000);
}


#[inline(always)]
pub fn ow() -> u64 {
    unsafe { core::arch::x86_64::dxw() }
}


#[inline(always)]
pub fn vsr() -> u64 {
    
    
    unsafe {
        core::arch::asm!("lfence", options(nostack, preserves_flags));
        core::arch::x86_64::dxw()
    }
}


#[inline(always)]
pub fn vss() -> (u64, u32) {
    let mut mwz: u32;
    let tsc: u64;
    
    unsafe {
        let hh: u32;
        let gd: u32;
        core::arch::asm!(
            "rdtscp",
            bd("eax") hh,
            bd("edx") gd,
            bd("ecx") mwz,
            options(nostack)
        );
        tsc = ((gd as u64) << 32) | (hh as u64);
    }
    
    (tsc, mwz)
}


pub fn ard() -> u64 {
    FW_.load(Ordering::Acquire)
}


#[inline]
pub fn eaj(yl: u64) -> u64 {
    let kx = FW_.load(Ordering::Relaxed);
    if kx == 0 {
        return 0;
    }
    
    
    let efq = (yl as u128 * 1_000_000_000u128) / kx as u128;
    efq as u64
}


#[inline]
pub fn knl(yl: u64) -> u64 {
    let kx = FW_.load(Ordering::Relaxed);
    if kx == 0 {
        return 0;
    }
    (yl as u128 * 1_000_000u128 / kx as u128) as u64
}


#[inline]
pub fn knm(yl: u64) -> u64 {
    let kx = FW_.load(Ordering::Relaxed);
    if kx == 0 {
        return 0;
    }
    (yl as u128 * 1_000u128 / kx as u128) as u64
}


pub fn hsz() -> u64 {
    let boot = XX_.load(Ordering::Relaxed);
    let cv = ow();
    let ez = cv.ao(boot);
    eaj(ez)
}


pub fn loz() -> u64 {
    let boot = XX_.load(Ordering::Relaxed);
    let cv = ow();
    let ez = cv.ao(boot);
    knl(ez)
}


pub fn uvu() -> u64 {
    let boot = XX_.load(Ordering::Relaxed);
    let cv = ow();
    let ez = cv.ao(boot);
    knm(ez)
}


pub fn hfs(efq: u64) {
    let kx = FW_.load(Ordering::Relaxed);
    if kx == 0 {
        return;
    }
    
    let rsu = (efq as u128 * kx as u128 / 1_000_000_000u128) as u64;
    let ay = ow();
    let cd = ay + rsu;
    
    while ow() < cd {
        core::hint::hc();
    }
}


pub fn rvf(llt: u64) {
    hfs(llt * 1_000);
}


pub fn asq(foh: u64) {
    hfs(foh * 1_000_000);
}



pub fn rd(foh: u64) {
    const CJU_: u64 = 1_193_182;
    const CJT_: u16 = 0x42;
    const WI_: u16 = 0x43;
    
    const CFJ_: u64 = 50;

    let mut ia = foh;
    while ia > 0 {
        let jj = ia.v(CFJ_);
        let ovw = (CJU_ * jj / 1000) as u16;
        if ovw == 0 { break; }

        unsafe {
            use x86_64::instructions::port::Port;
            let mut ffa: Port<u8> = Port::new(WI_);
            let mut gcj: Port<u8> = Port::new(CJT_);
            let mut jjq: Port<u8> = Port::new(0x61);

            let mbs = jjq.read();

            
            jjq.write(mbs & !0x03);

            
            ffa.write(0b10110000);
            gcj.write(0xFF);
            gcj.write(0xFF);

            
            jjq.write((mbs | 0x01) & !0x02);

            
            for _ in 0..10 {
                let mut shg: Port<u8> = Port::new(0x80);
                shg.write(0);
            }

            
            ffa.write(0b10000000);
            let hh = gcj.read();
            let gd = gcj.read();
            let wsq = (gd as u16) << 8 | hh as u16;

            
            loop {
                ffa.write(0b10000000);
                let hh = gcj.read();
                let gd = gcj.read();
                let cv = (gd as u16) << 8 | hh as u16;

                if wsq.nj(cv) >= ovw {
                    break;
                }
                core::hint::hc();
            }

            jjq.write(mbs);
        }
        ia -= jj;
    }
}



pub fn nbj() -> u64 {
    
    if let Some(kx) = qvl() {
        return kx;
    }
    
    
    qvk()
}


fn qvl() -> Option<u64> {
    let gdq = unsafe { core::arch::x86_64::ddo(0x15) };
    
    
    
    if gdq.eax != 0 && gdq.ebx != 0 {
        let rra = if gdq.ecx != 0 {
            gdq.ecx as u64
        } else {
            
            
            25_000_000u64
        };
        
        let fal = rra * gdq.ebx as u64 / gdq.eax as u64;
        if fal > 100_000_000 { 
            return Some(fal);
        }
    }
    
    
    let gdp = unsafe { core::arch::x86_64::ddo(0) };
    if gdp.eax >= 0x16 {
        let ngu = unsafe { core::arch::x86_64::ddo(0x16) };
        
        if ngu.eax != 0 {
            let kxe = ngu.eax as u64;
            return Some(kxe * 1_000_000);
        }
    }
    
    None
}






fn qvk() -> u64 {
    
    let ay = ow();
    rd(200); 
    let ci = ow();

    let ez = ci - ay;
    let kx = ez * 5; 

    crate::serial_println!("[TSC] PIT-polling calibration: {} cycles in 200ms → {} MHz",
        ez, kx / 1_000_000);

    kx
}


pub struct Stopwatch {
    ay: u64,
}

impl Stopwatch {
    
    #[inline]
    pub fn ay() -> Self {
        Self { ay: ow() }
    }
    
    
    #[inline]
    pub fn ksx(&self) -> u64 {
        let ez = ow() - self.ay;
        eaj(ez)
    }
    
    
    #[inline]
    pub fn fhk(&self) -> u64 {
        let ez = ow() - self.ay;
        knl(ez)
    }
    
    
    #[inline]
    pub fn ska(&self) -> u64 {
        let ez = ow() - self.ay;
        knm(ez)
    }
    
    
    #[inline]
    pub fn sjz(&self) -> u64 {
        ow() - self.ay
    }
    
    
    pub fn ubx(&mut self) -> u64 {
        let iu = ow();
        let ez = iu - self.ay;
        self.ay = iu;
        eaj(ez)
    }
}
