//! SMP - Symmetric Multi-Processing
//!
//! Multi-core CPU support using Limine SMP protocol.

use core::sync::atomic::{AtomicU32, AtomicBool, Ordering};
use alloc::vec::Vec;

/// Maximum supported CPUs
pub const MAX_CPUS: usize = 64;

/// Number of active CPUs
static CPU_COUNT: AtomicU32 = AtomicU32::new(1);

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
        }
    }
}

/// Per-CPU data array
static mut PER_CPU: [PerCpuData; MAX_CPUS] = {
    const INIT: PerCpuData = PerCpuData::new(0, 0);
    [INIT; MAX_CPUS]
};

/// Get current CPU ID (from APIC or GS segment)
pub fn current_cpu_id() -> u32 {
    // Read APIC ID from local APIC
    // For now, use a simpler method - assume BSP
    0 // TODO: implement proper CPU ID detection
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
    
    // Initialize BSP per-CPU data
    unsafe {
        PER_CPU[0].cpu_id = 0;
        PER_CPU[0].apic_id = get_apic_id();
    }
    
    BSP_ID.store(unsafe { PER_CPU[0].apic_id }, Ordering::Release);
    
    crate::serial_println!("[SMP] BSP APIC ID: {}", unsafe { PER_CPU[0].apic_id });
    crate::serial_println!("[SMP] Single-core mode (AP startup not yet implemented)");
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
    crate::println!("SMP Status:");
    crate::println!("  BSP APIC ID: {}", BSP_ID.load(Ordering::Relaxed));
    crate::println!("  CPU count: {}", cpu_count());
    
    for i in 0..cpu_count() as usize {
        let ready = if is_cpu_ready(i as u32) { "ready" } else { "offline" };
        crate::println!("  CPU {}: {}", i, ready);
    }
}

// ============================================================================
// Future: AP (Application Processor) startup code
// ============================================================================

/// AP entry point (called by Limine for each AP)
/// This function will be called with a unique stack for each AP
#[allow(dead_code)]
extern "C" fn ap_entry(smp_info: *const limine::smp::Cpu) -> ! {
    // Read AP info
    let cpu_info = unsafe { &*smp_info };
    let processor_id = cpu_info.id as usize;
    let lapic_id = cpu_info.lapic_id;
    
    // Initialize per-CPU data
    if processor_id < MAX_CPUS {
        unsafe {
            PER_CPU[processor_id].cpu_id = processor_id as u32;
            PER_CPU[processor_id].apic_id = lapic_id;
        }
        
        // Enable SSE for this CPU
        super::simd::enable_sse();
        
        // Mark CPU as ready
        CPU_READY[processor_id].store(true, Ordering::Release);
        CPU_COUNT.fetch_add(1, Ordering::Release);
        
        crate::serial_println!("[SMP] AP {} online (LAPIC ID: {})", processor_id, lapic_id);
    }
    
    // AP idle loop
    loop {
        // TODO: Check for work in per-CPU run queue
        // For now, just halt
        unsafe {
            core::arch::asm!("hlt");
        }
    }
}
