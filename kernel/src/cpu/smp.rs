//! SMP - Symmetric Multi-Processing
//!
//! Multi-core CPU support using Limine SMP protocol.
//! Provides work distribution across all available cores.

use core::sync::atomic::{AtomicU32, AtomicBool, Ordering};
use alloc::vec::Vec;
use spin::Mutex;

/// Maximum supported CPUs
pub const MAX_CPUS: usize = 64;

/// Number of active CPUs
static CPU_COUNT: AtomicU32 = AtomicU32::new(1);

/// Number of ready CPUs  
static READY_COUNT: AtomicU32 = AtomicU32::new(1);

/// Per-CPU initialization complete flags
static CPU_READY: [AtomicBool; MAX_CPUS] = {
    const INIT: AtomicBool = AtomicBool::new(false);
    [INIT; MAX_CPUS]
};

/// BSP (Bootstrap Processor) ID
static BSP_ID: AtomicU32 = AtomicU32::new(0);

/// Per-CPU data structure
#[repr(C)]
pub struct PerCpuData {
    /// CPU ID (0 = BSP)
    pub cpu_id: u32,
    /// APIC ID  
    pub apic_id: u32,
    /// Current running task ID
    pub current_task: u64,
    /// Interrupt nesting level
    pub interrupt_depth: u32,
    /// Kernel stack pointer
    pub kernel_stack: u64,
    /// TSC at last calibration
    pub tsc_last: u64,
    /// Work items completed
    pub work_completed: u64,
}

impl PerCpuData {
    pub const fn new(cpu_id: u32, apic_id: u32) -> Self {
        Self {
            cpu_id,
            apic_id,
            current_task: 0,
            interrupt_depth: 0,
            kernel_stack: 0,
            tsc_last: 0,
            work_completed: 0,
        }
    }
}

/// Per-CPU data array
static mut PER_CPU: [PerCpuData; MAX_CPUS] = {
    const INIT: PerCpuData = PerCpuData::new(0, 0);
    [INIT; MAX_CPUS]
};

/// Get current CPU ID (from APIC)
pub fn current_cpu_id() -> u32 {
    let cpuid = unsafe { core::arch::x86_64::__cpuid(1) };
    let apic_id = ((cpuid.ebx >> 24) & 0xFF) as u32;
    
    // Find which CPU has this APIC ID
    for i in 0..cpu_count() as usize {
        if unsafe { PER_CPU[i].apic_id == apic_id } {
            return i as u32;
        }
    }
    0 // Default to BSP
}

/// Get per-CPU data for current CPU
pub fn current() -> &'static PerCpuData {
    let id = current_cpu_id() as usize;
    unsafe { &PER_CPU[id.min(MAX_CPUS - 1)] }
}

/// Get mutable per-CPU data for current CPU
pub fn current_mut() -> &'static mut PerCpuData {
    let id = current_cpu_id() as usize;
    unsafe { &mut PER_CPU[id.min(MAX_CPUS - 1)] }
}

/// Get CPU count
pub fn cpu_count() -> u32 {
    CPU_COUNT.load(Ordering::Relaxed)
}

/// Set CPU count (called from main after SMP init)
pub fn set_cpu_count(count: u32) {
    CPU_COUNT.store(count, Ordering::Release);
}

/// Get ready CPU count
pub fn ready_cpu_count() -> u32 {
    READY_COUNT.load(Ordering::Acquire)
}

/// Check if CPU is ready
pub fn is_cpu_ready(cpu_id: u32) -> bool {
    if (cpu_id as usize) < MAX_CPUS {
        CPU_READY[cpu_id as usize].load(Ordering::Relaxed)
    } else {
        false
    }
}

/// Initialize SMP (called from BSP)
pub fn init() {
    // Mark BSP as ready
    CPU_READY[0].store(true, Ordering::Release);
    READY_COUNT.store(1, Ordering::Release);
    
    // Initialize BSP per-CPU data
    unsafe {
        PER_CPU[0].cpu_id = 0;
        PER_CPU[0].apic_id = get_apic_id();
        PER_CPU[0].work_completed = 0;
    }
    
    BSP_ID.store(unsafe { PER_CPU[0].apic_id }, Ordering::Release);
    
    crate::serial_println!("[SMP] BSP initialized (APIC ID: {})", unsafe { PER_CPU[0].apic_id });
}

/// Get local APIC ID
fn get_apic_id() -> u32 {
    let cpuid = unsafe { core::arch::x86_64::__cpuid(1) };
    ((cpuid.ebx >> 24) & 0xFF) as u32
}

/// SMP information structure
pub struct SmpInfo {
    pub cpu_count: u32,
    pub bsp_apic_id: u32,
    pub ap_apic_ids: Vec<u32>,
}

impl SmpInfo {
    /// Detect SMP configuration from ACPI MADT
    pub fn detect() -> Self {
        let bsp_apic_id = get_apic_id();
        
        // Try to get CPU info from ACPI
        if let Some(acpi_info) = crate::acpi::get_info() {
            let mut ap_ids = Vec::new();
            for lapic in &acpi_info.local_apics {
                if lapic.enabled && lapic.apic_id != bsp_apic_id {
                    ap_ids.push(lapic.apic_id);
                }
            }
            
            Self {
                cpu_count: (1 + ap_ids.len()) as u32,
                bsp_apic_id,
                ap_apic_ids: ap_ids,
            }
        } else {
            // Fallback: single CPU
            Self {
                cpu_count: 1,
                bsp_apic_id,
                ap_apic_ids: Vec::new(),
            }
        }
    }
}

/// Print SMP status
pub fn print_status() {
    let par_mode = if is_smp_enabled() { "ON " } else { "OFF" };
    crate::println!("╔══════════════════════════════════════╗");
    crate::println!("║          SMP STATUS                  ║");
    crate::println!("╠══════════════════════════════════════╣");
    crate::println!("║ Parallel:    {}                      ║", par_mode);
    crate::println!("║ BSP APIC ID: {:3}                     ║", BSP_ID.load(Ordering::Relaxed));
    crate::println!("║ Total CPUs:  {:3}                     ║", cpu_count());
    crate::println!("║ Ready CPUs:  {:3}                     ║", ready_cpu_count());
    crate::println!("╠══════════════════════════════════════╣");
    
    for i in 0..cpu_count().min(MAX_CPUS as u32) as usize {
        let ready = if is_cpu_ready(i as u32) { "✓" } else { "✗" };
        let work = unsafe { PER_CPU[i].work_completed };
        crate::println!("║ CPU {:2}: {} APIC {:3}  Work: {:8}  ║", 
            i, ready, unsafe { PER_CPU[i].apic_id }, work);
    }
    crate::println!("╚══════════════════════════════════════╝");
}

/// Get statistics for all CPUs
pub fn get_stats() -> (u32, u32, u64) {
    let total = cpu_count();
    let ready = ready_cpu_count();
    let mut total_work = 0u64;
    
    for i in 0..total as usize {
        total_work += unsafe { PER_CPU[i].work_completed };
    }
    
    (total, ready, total_work)
}

// ============================================================================
// PARALLEL WORK QUEUE
// ============================================================================

/// Work item function type
pub type WorkFn = fn(usize, usize, *mut u8);

/// Work item for parallel execution
#[derive(Clone, Copy)]
pub struct WorkItem {
    pub func: Option<WorkFn>,
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

/// Per-CPU work queues
static WORK_QUEUES: [Mutex<WorkItem>; MAX_CPUS] = {
    const INIT: Mutex<WorkItem> = Mutex::new(WorkItem::empty());
    [INIT; MAX_CPUS]
};

/// Signal work pending
static WORK_PENDING: [AtomicBool; MAX_CPUS] = {
    const INIT: AtomicBool = AtomicBool::new(false);
    [INIT; MAX_CPUS]
};

/// Global work completion counter
static WORK_COMPLETION: AtomicU32 = AtomicU32::new(0);

/// SMP parallelism toggle (can be disabled at runtime for safety)
static SMP_ENABLED: AtomicBool = AtomicBool::new(true);

/// Minimum items to consider parallel execution (avoid overhead for small work)
const PARALLEL_THRESHOLD: usize = 32;

/// Maximum spin iterations to wait for APs (prevents infinite hang)
const MAX_WAIT_SPINS: u32 = 500_000;

/// Enable SMP parallelism
pub fn enable_smp() {
    SMP_ENABLED.store(true, Ordering::Release);
    crate::serial_println!("[SMP] Parallelism ENABLED");
}

/// Disable SMP parallelism (fallback to single-core)
pub fn disable_smp() {
    SMP_ENABLED.store(false, Ordering::Release);
    crate::serial_println!("[SMP] Parallelism DISABLED");
}

/// Check if SMP is enabled
pub fn is_smp_enabled() -> bool {
    SMP_ENABLED.load(Ordering::Relaxed)
}

/// Execute a function in parallel across all available cores
/// 
/// # Safety
/// - `func` must be safe to call from any CPU core
/// - `data` must remain valid until all cores complete
/// - `func` must not assume single-threaded execution
pub fn parallel_for(total_items: usize, func: WorkFn, data: *mut u8) {
    // Check if SMP parallelism is enabled
    if !SMP_ENABLED.load(Ordering::Relaxed) {
        func(0, total_items, data);
        return;
    }
    
    let num_cpus = ready_cpu_count() as usize;
    
    // Fallback to single-thread if:
    // - Only 1 CPU ready
    // - Workload too small (overhead not worth it)
    if num_cpus <= 1 || total_items < PARALLEL_THRESHOLD {
        func(0, total_items, data);
        return;
    }
    
    // Calculate chunk size - each core gets equal portion
    let chunk_size = (total_items + num_cpus - 1) / num_cpus;
    WORK_COMPLETION.store(0, Ordering::Release);
    
    // Distribute work to APs (Application Processors)
    let mut dispatched = 0usize;
    for cpu_id in 1..num_cpus {
        if !is_cpu_ready(cpu_id as u32) { continue; }
        
        let start = cpu_id * chunk_size;
        if start >= total_items { break; }
        let end = ((cpu_id + 1) * chunk_size).min(total_items);
        
        // Queue work for this AP
        {
            let mut work = WORK_QUEUES[cpu_id].lock();
            work.func = Some(func);
            work.start = start;
            work.end = end;
            work.data = data;
        }
        
        WORK_PENDING[cpu_id].store(true, Ordering::Release);
        dispatched += 1;
    }
    
    // BSP (Bootstrap Processor) handles chunk 0
    func(0, chunk_size.min(total_items), data);
    WORK_COMPLETION.fetch_add(1, Ordering::Release);
    
    // Wait for APs to complete with timeout
    if dispatched > 0 {
        let expected = dispatched as u32 + 1;
        let mut spin_count = 0u32;
        
        while WORK_COMPLETION.load(Ordering::Acquire) < expected {
            core::hint::spin_loop();
            spin_count += 1;
            
            // Timeout protection - avoid infinite hang
            if spin_count > MAX_WAIT_SPINS {
                crate::serial_println!("[SMP] WARNING: Timeout waiting for APs, completed {}/{}", 
                    WORK_COMPLETION.load(Ordering::Relaxed), expected);
                break;
            }
        }
    }
}

/// Check and execute work (called from AP loop)
fn check_and_execute_work(cpu_id: usize) {
    if WORK_PENDING[cpu_id].load(Ordering::Acquire) {
        let work = { *WORK_QUEUES[cpu_id].lock() };
        WORK_PENDING[cpu_id].store(false, Ordering::Release);
        
        if let Some(func) = work.func {
            func(work.start, work.end, work.data);
            unsafe { PER_CPU[cpu_id].work_completed += 1; }
        }
        
        WORK_COMPLETION.fetch_add(1, Ordering::Release);
    }
}

// ============================================================================
// Future: AP (Application Processor) startup code
// ============================================================================

/// AP entry point (called by Limine for each AP)
/// This function will be called with a unique stack for each AP
pub unsafe extern "C" fn ap_entry(smp_info: &limine::smp::Cpu) -> ! {
    // Read AP info
    let processor_id = smp_info.id as usize;
    let lapic_id = smp_info.lapic_id;
    
    // Initialize per-CPU data
    if processor_id < MAX_CPUS {
        PER_CPU[processor_id].cpu_id = processor_id as u32;
        PER_CPU[processor_id].apic_id = lapic_id;
        PER_CPU[processor_id].work_completed = 0;
        
        // Enable SSE for this CPU
        super::simd::enable_sse();
        
        // Mark CPU as ready
        CPU_READY[processor_id].store(true, Ordering::Release);
        READY_COUNT.fetch_add(1, Ordering::Release);
        
        crate::serial_println!("[SMP] AP {} online (LAPIC ID: {})", processor_id, lapic_id);
    }
    
    // AP work loop - check for work, then yield CPU
    loop {
        check_and_execute_work(processor_id);
        
        // Longer pause between work checks (~0.1ms)
        // Reduces CPU waste and fixes display refresh on VirtualBox/Hyper-V
        // (tight spin loops starve the hypervisor's display refresh thread)
        for _ in 0..100_000 {
            core::hint::spin_loop();
        }
    }
}
