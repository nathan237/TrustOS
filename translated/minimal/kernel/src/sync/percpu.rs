




use core::sync::atomic::{AtomicU64, AtomicBool, AtomicUsize, Ordering};
use core::cell::UnsafeCell;
use alloc::vec::Vec;
use alloc::boxed::Box;


pub const AR_: usize = 32;





#[repr(C, align(64))] 
pub struct PercpuBlock {
    
    pub gs_base: u64,
    
    pub cpu_id: u32,
    _pad0: u32,
    
    pub current_tid: AtomicU64,
    
    pub inside_syscall: AtomicBool,
    
    pub in_interrupt: AtomicBool,
    
    pub interrupt_depth: AtomicUsize,
    
    pub preempt_disabled: AtomicUsize,
    
    pub need_reschedule: AtomicBool,
    
    pub is_idle: AtomicBool,
    
    pub context_switches: AtomicU64,
    
    pub syscall_count: AtomicU64,
    
    pub interrupt_count: AtomicU64,
    
    pub last_tick_tsc: AtomicU64,
    
    pub scratch: [u64; 8],
    
    pub kernel_stack: u64,
    
    pub user_stack: u64,
}

impl PercpuBlock {
    pub const fn new(cpu_id: u32) -> Self {
        Self {
            gs_base: 0, 
            cpu_id,
            _pad0: 0,
            current_tid: AtomicU64::new(0),
            inside_syscall: AtomicBool::new(false),
            in_interrupt: AtomicBool::new(false),
            interrupt_depth: AtomicUsize::new(0),
            preempt_disabled: AtomicUsize::new(0),
            need_reschedule: AtomicBool::new(false),
            is_idle: AtomicBool::new(true),
            context_switches: AtomicU64::new(0),
            syscall_count: AtomicU64::new(0),
            interrupt_count: AtomicU64::new(0),
            last_tick_tsc: AtomicU64::new(0),
            scratch: [0; 8],
            kernel_stack: 0,
            user_stack: 0,
        }
    }
    
    
    
    
    
    #[inline]
    pub fn current() -> &'static Self {
        #[cfg(target_arch = "x86_64")]
        unsafe {
            let self_ptr: u64;
            core::arch::asm!(
                "mov {}, gs:[0]",
                out(reg) self_ptr,
                options(pure, nomem, nostack)
            );
            
            
            if self_ptr == 0 {
                return &FD_[0];
            }
            
            return &*(self_ptr as *const Self);
        }
        #[cfg(not(target_arch = "x86_64"))]
        unsafe { &FD_[0] }
    }
    
    
    #[inline]
    pub fn qfe(&self) {
        self.interrupt_depth.fetch_add(1, Ordering::Relaxed);
        self.in_interrupt.store(true, Ordering::Release);
        self.interrupt_count.fetch_add(1, Ordering::Relaxed);
    }
    
    
    #[inline]
    pub fn qnh(&self) {
        let depth = self.interrupt_depth.fetch_sub(1, Ordering::Relaxed);
        if depth == 1 {
            self.in_interrupt.store(false, Ordering::Release);
        }
    }
    
    
    #[inline]
    pub fn qqv(&self) {
        self.preempt_disabled.fetch_add(1, Ordering::Relaxed);
    }
    
    
    #[inline]
    pub fn qqw(&self) {
        let count = self.preempt_disabled.fetch_sub(1, Ordering::Relaxed);
        if count == 1 && self.need_reschedule.load(Ordering::Relaxed) {
            
            
            crate::scheduler::boq();
        }
    }
    
    
    #[inline]
    pub fn qqx(&self) -> bool {
        self.preempt_disabled.load(Ordering::Relaxed) == 0
    }
    
    
    #[inline]
    pub fn qwj(&self) {
        self.need_reschedule.store(true, Ordering::Release);
    }
    
    
    #[inline]
    pub fn pzz(&self) {
        self.need_reschedule.store(false, Ordering::Release);
    }
}


static mut FD_: [PercpuBlock; AR_] = {
    const Bm: PercpuBlock = PercpuBlock::new(0);
    [Bm; AR_]
};


static XE_: AtomicUsize = AtomicUsize::new(1);


pub fn qky() {
    unsafe {
        FD_[0].cpu_id = 0;
        
        
        let cgj = &FD_[0] as *const _ as u64;
        
        #[cfg(target_arch = "x86_64")]
        {
            
            let msr = 0xC0000102u32; 
            let low = cgj as u32;
            let high = (cgj >> 32) as u32;
            
            core::arch::asm!(
                "wrmsr",
                in("ecx") msr,
                in("eax") low,
                in("edx") high,
                options(nostack)
            );
            
            
            let gs_msr = 0xC0000101u32; 
            core::arch::asm!(
                "wrmsr",
                in("ecx") gs_msr,
                in("eax") low,
                in("edx") high,
                options(nostack)
            );
        }
        
        FD_[0].gs_base = cgj;
    }
    
    crate::log!("Per-CPU data initialized for BSP");
}


pub fn cau(cpu_id: u32) {
    if cpu_id as usize >= AR_ {
        return;
    }
    
    unsafe {
        FD_[cpu_id as usize].cpu_id = cpu_id;
        
        
        let cgj = &FD_[cpu_id as usize] as *const _ as u64;
        
        #[cfg(target_arch = "x86_64")]
        {
            let msr = 0xC0000102u32;
            let low = cgj as u32;
            let high = (cgj >> 32) as u32;
            
            core::arch::asm!(
                "wrmsr",
                in("ecx") msr,
                in("eax") low,
                in("edx") high,
                options(nostack)
            );
            
            let gs_msr = 0xC0000101u32;
            core::arch::asm!(
                "wrmsr",
                in("ecx") gs_msr,
                in("eax") low,
                in("edx") high,
                options(nostack)
            );
        }
        
        FD_[cpu_id as usize].gs_base = cgj;
    }
    
    XE_.fetch_add(1, Ordering::Relaxed);
}


pub fn mcu(cpu_id: u32) -> Option<&'static PercpuBlock> {
    if (cpu_id as usize) < XE_.load(Ordering::Relaxed) {
        unsafe { Some(&FD_[cpu_id as usize]) }
    } else {
        None
    }
}


pub fn num_cpus() -> usize {
    XE_.load(Ordering::Relaxed)
}


#[inline]
pub fn bll() -> u32 {
    PercpuBlock::current().cpu_id
}


pub fn mur() -> impl Iterator<Item = &'static PercpuBlock> {
    let ae = XE_.load(Ordering::Relaxed);
    unsafe { FD_[..ae].iter() }
}


pub struct Ui<T> {
    data: UnsafeCell<[Option<T>; AR_]>,
}

unsafe impl<T: Send> Send for Ui<T> {}
unsafe impl<T: Send + Sync> Sync for Ui<T> {}

impl<T> Ui<T> {
    pub const fn new() -> Self {
        const Bc: Option<()> = None;
        Self {
            data: UnsafeCell::new([const { None }; AR_]),
        }
    }
    
    
    pub fn get(&self) -> Option<&T> {
        let cpu = bll() as usize;
        if cpu < AR_ {
            unsafe { (*self.data.get())[cpu].as_ref() }
        } else {
            None
        }
    }
    
    
    pub fn get_mut(&self) -> Option<&mut T> {
        let cpu = bll() as usize;
        if cpu < AR_ {
            unsafe { (*self.data.get())[cpu].as_mut() }
        } else {
            None
        }
    }
    
    
    pub fn set(&self, value: T) {
        let cpu = bll() as usize;
        if cpu < AR_ {
            unsafe { (*self.data.get())[cpu] = Some(value) };
        }
    }
    
    
    pub fn mcu(&self, cpu_id: u32) -> Option<&T> {
        let cpu = cpu_id as usize;
        if cpu < AR_ {
            unsafe { (*self.data.get())[cpu].as_ref() }
        } else {
            None
        }
    }
}


#[derive(Debug, Clone)]
pub struct Rl {
    pub cpu_id: u32,
    pub context_switches: u64,
    pub syscalls: u64,
    pub interrupts: u64,
    pub is_idle: bool,
    pub current_tid: u64,
}


pub fn dhj() -> Vec<Rl> {
    mur()
        .map(|block| Rl {
            cpu_id: block.cpu_id,
            context_switches: block.context_switches.load(Ordering::Relaxed),
            syscalls: block.syscall_count.load(Ordering::Relaxed),
            interrupts: block.interrupt_count.load(Ordering::Relaxed),
            is_idle: block.is_idle.load(Ordering::Relaxed),
            current_tid: block.current_tid.load(Ordering::Relaxed),
        })
        .collect()
}
