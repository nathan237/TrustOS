




use core::sync::atomic::{AtomicU64, AtomicBool, AtomicUsize, Ordering};
use core::cell::UnsafeCell;
use alloc::vec::Vec;
use alloc::boxed::Box;


pub const AN_: usize = 32;





#[repr(C, align(64))] 
pub struct PercpuBlock {
    
    pub fjy: u64,
    
    pub qq: u32,
    iig: u32,
    
    pub bqd: AtomicU64,
    
    pub tve: AtomicBool,
    
    pub ldu: AtomicBool,
    
    pub flu: AtomicUsize,
    
    pub jjt: AtomicUsize,
    
    pub jgo: AtomicBool,
    
    pub edw: AtomicBool,
    
    pub gdf: AtomicU64,
    
    pub gtx: AtomicU64,
    
    pub lfd: AtomicU64,
    
    pub ucp: AtomicU64,
    
    pub pgn: [u64; 8],
    
    pub bhg: u64,
    
    pub ais: u64,
}

impl PercpuBlock {
    pub const fn new(qq: u32) -> Self {
        Self {
            fjy: 0, 
            qq,
            iig: 0,
            bqd: AtomicU64::new(0),
            tve: AtomicBool::new(false),
            ldu: AtomicBool::new(false),
            flu: AtomicUsize::new(0),
            jjt: AtomicUsize::new(0),
            jgo: AtomicBool::new(false),
            edw: AtomicBool::new(true),
            gdf: AtomicU64::new(0),
            gtx: AtomicU64::new(0),
            lfd: AtomicU64::new(0),
            ucp: AtomicU64::new(0),
            pgn: [0; 8],
            bhg: 0,
            ais: 0,
        }
    }
    
    
    
    
    
    #[inline]
    pub fn cv() -> &'static Self {
        #[cfg(target_arch = "x86_64")]
        unsafe {
            let mdm: u64;
            core::arch::asm!(
                "mov {}, gs:[0]",
                bd(reg) mdm,
                options(zgw, nomem, nostack)
            );
            
            
            if mdm == 0 {
                return &EO_[0];
            }
            
            return &*(mdm as *const Self);
        }
        #[cfg(not(target_arch = "x86_64"))]
        unsafe { &EO_[0] }
    }
    
    
    #[inline]
    pub fn ypn(&self) {
        self.flu.fetch_add(1, Ordering::Relaxed);
        self.ldu.store(true, Ordering::Release);
        self.lfd.fetch_add(1, Ordering::Relaxed);
    }
    
    
    #[inline]
    pub fn zao(&self) {
        let eo = self.flu.fetch_sub(1, Ordering::Relaxed);
        if eo == 1 {
            self.ldu.store(false, Ordering::Release);
        }
    }
    
    
    #[inline]
    pub fn zgf(&self) {
        self.jjt.fetch_add(1, Ordering::Relaxed);
    }
    
    
    #[inline]
    pub fn zgg(&self) {
        let az = self.jjt.fetch_sub(1, Ordering::Relaxed);
        if az == 1 && self.jgo.load(Ordering::Relaxed) {
            
            
            crate::scheduler::dvk();
        }
    }
    
    
    #[inline]
    pub fn zgh(&self) -> bool {
        self.jjt.load(Ordering::Relaxed) == 0
    }
    
    
    #[inline]
    pub fn znn(&self) {
        self.jgo.store(true, Ordering::Release);
    }
    
    
    #[inline]
    pub fn yil(&self) {
        self.jgo.store(false, Ordering::Release);
    }
}


static mut EO_: [PercpuBlock; AN_] = {
    const Dm: PercpuBlock = PercpuBlock::new(0);
    [Dm; AN_]
};


static VV_: AtomicUsize = AtomicUsize::new(1);


pub fn yxn() {
    unsafe {
        EO_[0].qq = 0;
        
        
        let fdn = &EO_[0] as *const _ as u64;
        
        #[cfg(target_arch = "x86_64")]
        {
            
            let msr = 0xC0000102u32; 
            let ail = fdn as u32;
            let afq = (fdn >> 32) as u32;
            
            core::arch::asm!(
                "wrmsr",
                in("ecx") msr,
                in("eax") ail,
                in("edx") afq,
                options(nostack)
            );
            
            
            let lap = 0xC0000101u32; 
            core::arch::asm!(
                "wrmsr",
                in("ecx") lap,
                in("eax") ail,
                in("edx") afq,
                options(nostack)
            );
        }
        
        EO_[0].fjy = fdn;
    }
    
    crate::log!("Per-CPU data initialized for BSP");
}


pub fn eso(qq: u32) {
    if qq as usize >= AN_ {
        return;
    }
    
    unsafe {
        EO_[qq as usize].qq = qq;
        
        
        let fdn = &EO_[qq as usize] as *const _ as u64;
        
        #[cfg(target_arch = "x86_64")]
        {
            let msr = 0xC0000102u32;
            let ail = fdn as u32;
            let afq = (fdn >> 32) as u32;
            
            core::arch::asm!(
                "wrmsr",
                in("ecx") msr,
                in("eax") ail,
                in("edx") afq,
                options(nostack)
            );
            
            let lap = 0xC0000101u32;
            core::arch::asm!(
                "wrmsr",
                in("ecx") lap,
                in("eax") ail,
                in("edx") afq,
                options(nostack)
            );
        }
        
        EO_[qq as usize].fjy = fdn;
    }
    
    VV_.fetch_add(1, Ordering::Relaxed);
}


pub fn tde(qq: u32) -> Option<&'static PercpuBlock> {
    if (qq as usize) < VV_.load(Ordering::Relaxed) {
        unsafe { Some(&EO_[qq as usize]) }
    } else {
        None
    }
}


pub fn bcc() -> usize {
    VV_.load(Ordering::Relaxed)
}


#[inline]
pub fn ead() -> u32 {
    PercpuBlock::cv().qq
}


pub fn tzz() -> impl Iterator<Item = &'static PercpuBlock> {
    let bo = VV_.load(Ordering::Relaxed);
    unsafe { EO_[..bo].iter() }
}


pub struct Awt<T> {
    f: UnsafeCell<[Option<T>; AN_]>,
}

unsafe impl<T: Send> Send for Awt<T> {}
unsafe impl<T: Send + Sync> Sync for Awt<T> {}

impl<T> Awt<T> {
    pub const fn new() -> Self {
        const Cq: Option<()> = None;
        Self {
            f: UnsafeCell::new([const { None }; AN_]),
        }
    }
    
    
    pub fn get(&self) -> Option<&T> {
        let cpu = ead() as usize;
        if cpu < AN_ {
            unsafe { (*self.f.get())[cpu].as_ref() }
        } else {
            None
        }
    }
    
    
    pub fn ds(&self) -> Option<&mut T> {
        let cpu = ead() as usize;
        if cpu < AN_ {
            unsafe { (*self.f.get())[cpu].as_mut() }
        } else {
            None
        }
    }
    
    
    pub fn oj(&self, bn: T) {
        let cpu = ead() as usize;
        if cpu < AN_ {
            unsafe { (*self.f.get())[cpu] = Some(bn) };
        }
    }
    
    
    pub fn tde(&self, qq: u32) -> Option<&T> {
        let cpu = qq as usize;
        if cpu < AN_ {
            unsafe { (*self.f.get())[cpu].as_ref() }
        } else {
            None
        }
    }
}


#[derive(Debug, Clone)]
pub struct Aqf {
    pub qq: u32,
    pub gdf: u64,
    pub apd: u64,
    pub interrupts: u64,
    pub edw: bool,
    pub bqd: u64,
}


pub fn gyf() -> Vec<Aqf> {
    tzz()
        .map(|block| Aqf {
            qq: block.qq,
            gdf: block.gdf.load(Ordering::Relaxed),
            apd: block.gtx.load(Ordering::Relaxed),
            interrupts: block.lfd.load(Ordering::Relaxed),
            edw: block.edw.load(Ordering::Relaxed),
            bqd: block.bqd.load(Ordering::Relaxed),
        })
        .collect()
}
