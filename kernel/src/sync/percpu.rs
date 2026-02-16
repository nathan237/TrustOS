//! Per-CPU Data Storage
//!
//! Provides per-CPU variables for lock-free, cache-friendly data access.
//! Critical for SMP performance - avoids cache line bouncing.

use core::sync::atomic::{AtomicU64, AtomicBool, AtomicUsize, Ordering};
use core::cell::UnsafeCell;
use alloc::vec::Vec;
use alloc::boxed::Box;

/// Maximum number of CPUs supported
pub const MAX_CPUS: usize = 256;

/// Per-CPU block - contains all per-CPU data for one CPU
///
/// IMPORTANT: `gs_base` MUST be the first field (offset 0) so that
/// `mov reg, gs:[0]` reads the self-pointer for PercpuBlock::current().
#[repr(C, align(64))] // Cache line aligned
pub struct PercpuBlock {
    /// Self-pointer — MUST be at offset 0 for fast gs:[0] access
    pub gs_base: u64,
    /// CPU ID (sequential: 0 = BSP)
    pub cpu_id: u32,
    _pad0: u32,
    /// Current running thread ID
    pub current_tid: AtomicU64,
    /// Whether CPU is currently inside a syscall
    pub inside_syscall: AtomicBool,
    /// Whether CPU is in interrupt handler
    pub in_interrupt: AtomicBool,
    /// Interrupt nesting depth
    pub interrupt_depth: AtomicUsize,
    /// Preemption disabled count
    pub preempt_disabled: AtomicUsize,
    /// Context switch pending flag
    pub need_reschedule: AtomicBool,
    /// CPU is idle
    pub is_idle: AtomicBool,
    /// Number of context switches on this CPU
    pub context_switches: AtomicU64,
    /// Number of syscalls handled
    pub syscall_count: AtomicU64,
    /// Number of interrupts handled
    pub interrupt_count: AtomicU64,
    /// Last timer tick timestamp (TSC)
    pub last_tick_tsc: AtomicU64,
    /// CPU-local scratch space (for syscall handler, etc.)
    pub scratch: [u64; 8],
    /// Kernel stack pointer for this CPU
    pub kernel_stack: u64,
    /// User stack pointer (saved during syscall)
    pub user_stack: u64,
}

impl PercpuBlock {
    pub const fn new(cpu_id: u32) -> Self {
        Self {
            gs_base: 0, // Set during init_bsp/init_ap
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
    
    /// Get current per-CPU block (via GS segment)
    ///
    /// gs_base (offset 0) holds a self-pointer set by init_bsp/init_ap.
    /// Reading gs:[0] gives us the PercpuBlock address directly.
    #[inline]
    pub fn current() -> &'static Self {
        unsafe {
            let self_ptr: u64;
            core::arch::asm!(
                "mov {}, gs:[0]",
                out(reg) self_ptr,
                options(pure, nomem, nostack)
            );
            
            // Fallback if GS not set up yet (early boot)
            if self_ptr == 0 {
                return &PERCPU_BLOCKS[0];
            }
            
            &*(self_ptr as *const Self)
        }
    }
    
    /// Enter interrupt context
    #[inline]
    pub fn enter_interrupt(&self) {
        self.interrupt_depth.fetch_add(1, Ordering::Relaxed);
        self.in_interrupt.store(true, Ordering::Release);
        self.interrupt_count.fetch_add(1, Ordering::Relaxed);
    }
    
    /// Leave interrupt context
    #[inline]
    pub fn leave_interrupt(&self) {
        let depth = self.interrupt_depth.fetch_sub(1, Ordering::Relaxed);
        if depth == 1 {
            self.in_interrupt.store(false, Ordering::Release);
        }
    }
    
    /// Disable preemption
    #[inline]
    pub fn preempt_disable(&self) {
        self.preempt_disabled.fetch_add(1, Ordering::Relaxed);
    }
    
    /// Enable preemption
    #[inline]
    pub fn preempt_enable(&self) {
        let count = self.preempt_disabled.fetch_sub(1, Ordering::Relaxed);
        if count == 1 && self.need_reschedule.load(Ordering::Relaxed) {
            // Preemption now enabled and reschedule pending
            // Trigger scheduler
            crate::scheduler::schedule();
        }
    }
    
    /// Check if preemption is enabled
    #[inline]
    pub fn preempt_enabled(&self) -> bool {
        self.preempt_disabled.load(Ordering::Relaxed) == 0
    }
    
    /// Request a reschedule
    #[inline]
    pub fn set_need_reschedule(&self) {
        self.need_reschedule.store(true, Ordering::Release);
    }
    
    /// Clear reschedule flag (called by scheduler)
    #[inline]
    pub fn clear_need_reschedule(&self) {
        self.need_reschedule.store(false, Ordering::Release);
    }
}

/// Global per-CPU blocks array
static mut PERCPU_BLOCKS: [PercpuBlock; MAX_CPUS] = {
    const INIT: PercpuBlock = PercpuBlock::new(0);
    [INIT; MAX_CPUS]
};

/// Number of active CPUs
static NUM_CPUS: AtomicUsize = AtomicUsize::new(1);

/// Initialize per-CPU data for BSP (Bootstrap Processor)
pub fn init_bsp() {
    unsafe {
        PERCPU_BLOCKS[0].cpu_id = 0;
        
        // Set up GS base to point to our block
        let block_ptr = &PERCPU_BLOCKS[0] as *const _ as u64;
        
        // Write to KERNEL_GS_BASE MSR
        let msr = 0xC0000102u32; // IA32_KERNEL_GS_BASE
        let low = block_ptr as u32;
        let high = (block_ptr >> 32) as u32;
        
        core::arch::asm!(
            "wrmsr",
            in("ecx") msr,
            in("eax") low,
            in("edx") high,
            options(nostack)
        );
        
        // Also set GS_BASE for immediate use
        let gs_msr = 0xC0000101u32; // IA32_GS_BASE  
        core::arch::asm!(
            "wrmsr",
            in("ecx") gs_msr,
            in("eax") low,
            in("edx") high,
            options(nostack)
        );
        
        PERCPU_BLOCKS[0].gs_base = block_ptr;
    }
    
    crate::log!("Per-CPU data initialized for BSP");
}

/// Initialize per-CPU data for an AP (Application Processor)
pub fn init_ap(cpu_id: u32) {
    if cpu_id as usize >= MAX_CPUS {
        return;
    }
    
    unsafe {
        PERCPU_BLOCKS[cpu_id as usize].cpu_id = cpu_id;
        
        // Set up GS base
        let block_ptr = &PERCPU_BLOCKS[cpu_id as usize] as *const _ as u64;
        
        let msr = 0xC0000102u32;
        let low = block_ptr as u32;
        let high = (block_ptr >> 32) as u32;
        
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
        
        PERCPU_BLOCKS[cpu_id as usize].gs_base = block_ptr;
    }
    
    NUM_CPUS.fetch_add(1, Ordering::Relaxed);
}

/// Get per-CPU block for specific CPU
pub fn get_cpu(cpu_id: u32) -> Option<&'static PercpuBlock> {
    if (cpu_id as usize) < NUM_CPUS.load(Ordering::Relaxed) {
        unsafe { Some(&PERCPU_BLOCKS[cpu_id as usize]) }
    } else {
        None
    }
}

/// Get number of active CPUs
pub fn num_cpus() -> usize {
    NUM_CPUS.load(Ordering::Relaxed)
}

/// Get current CPU ID
#[inline]
pub fn current_cpu_id() -> u32 {
    PercpuBlock::current().cpu_id
}

/// Iterator over all active CPUs
pub fn iter_cpus() -> impl Iterator<Item = &'static PercpuBlock> {
    let n = NUM_CPUS.load(Ordering::Relaxed);
    unsafe { PERCPU_BLOCKS[..n].iter() }
}

/// Per-CPU variable wrapper
pub struct PerCpu<T> {
    data: UnsafeCell<[Option<T>; MAX_CPUS]>,
}

unsafe impl<T: Send> Send for PerCpu<T> {}
unsafe impl<T: Send + Sync> Sync for PerCpu<T> {}

impl<T> PerCpu<T> {
    pub const fn new() -> Self {
        const NONE: Option<()> = None;
        Self {
            data: UnsafeCell::new([const { None }; MAX_CPUS]),
        }
    }
    
    /// Get value for current CPU
    pub fn get(&self) -> Option<&T> {
        let cpu = current_cpu_id() as usize;
        if cpu < MAX_CPUS {
            unsafe { (*self.data.get())[cpu].as_ref() }
        } else {
            None
        }
    }
    
    /// Get mutable value for current CPU
    pub fn get_mut(&self) -> Option<&mut T> {
        let cpu = current_cpu_id() as usize;
        if cpu < MAX_CPUS {
            unsafe { (*self.data.get())[cpu].as_mut() }
        } else {
            None
        }
    }
    
    /// Set value for current CPU
    pub fn set(&self, value: T) {
        let cpu = current_cpu_id() as usize;
        if cpu < MAX_CPUS {
            unsafe { (*self.data.get())[cpu] = Some(value) };
        }
    }
    
    /// Get value for specific CPU
    pub fn get_cpu(&self, cpu_id: u32) -> Option<&T> {
        let cpu = cpu_id as usize;
        if cpu < MAX_CPUS {
            unsafe { (*self.data.get())[cpu].as_ref() }
        } else {
            None
        }
    }
}

/// CPU statistics
#[derive(Debug, Clone)]
pub struct CpuStats {
    pub cpu_id: u32,
    pub context_switches: u64,
    pub syscalls: u64,
    pub interrupts: u64,
    pub is_idle: bool,
    pub current_tid: u64,
}

/// Get stats for all CPUs
pub fn all_cpu_stats() -> Vec<CpuStats> {
    iter_cpus()
        .map(|block| CpuStats {
            cpu_id: block.cpu_id,
            context_switches: block.context_switches.load(Ordering::Relaxed),
            syscalls: block.syscall_count.load(Ordering::Relaxed),
            interrupts: block.interrupt_count.load(Ordering::Relaxed),
            is_idle: block.is_idle.load(Ordering::Relaxed),
            current_tid: block.current_tid.load(Ordering::Relaxed),
        })
        .collect()
}
