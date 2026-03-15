//! Memory Management Subsystem (Limine version)
//! 
//! Simple memory manager for Limine-booted kernel.
//! Includes paging support for process isolation.

pub mod heap;
pub mod frame;
pub mod paging;
pub mod cow;
pub mod swap;
pub mod vma;

use alloc::vec::Vec;
use core::sync::atomic::{AtomicUsize, AtomicU64, Ordering};
use spin::Mutex;

pub use paging::{AddressSpace, PageFlags, validate_user_pointer, is_user_address, is_kernel_address};

// ── Boot memory map storage (for debug diagnostics) ─────────────────────────
/// Stored memory map from Limine: (base, length, type_code)
/// Type codes: 0=USABLE, 1=RESERVED, 2=ACPI_RECLAIM, 3=ACPI_NVS,
///             4=BAD, 5=BOOTLOADER, 6=KERNEL, 7=FRAMEBUFFER
const MAXIMUM_MEMORY_REGIONS: usize = 64;

struct BootMemoryMap {
    entries: [(u64, u64, u8); MAXIMUM_MEMORY_REGIONS],
    count: usize,
}

// Implementation block — defines methods for the type above.
impl BootMemoryMap {
    const fn new() -> Self {
        Self { entries: [(0, 0, 0); MAXIMUM_MEMORY_REGIONS], count: 0 }
    }
    fn push(&mut self, base: u64, length: u64, type_code: u8) {
        if self.count < MAXIMUM_MEMORY_REGIONS {
            self.entries[self.count] = (base, length, type_code);
            self.count += 1;
        }
    }
    fn as_slice(&self) -> &[(u64, u64, u8)] {
        &self.entries[..self.count]
    }
}

// Global shared state guarded by a Mutex (mutual exclusion lock).
static BOOT_MEMORY_MAP: Mutex<BootMemoryMap> = Mutex::new(BootMemoryMap::new());

/// Store a memory map entry (call during boot from main.rs)
pub fn store_memory_region(base: u64, length: u64, type_code: u8) {
    if let Some(mut map) = BOOT_MEMORY_MAP.try_lock() {
        map.push(base, length, type_code);
    }
}

/// Get the stored memory map
pub fn get_memory_regions() -> Vec<(u64, u64, u8)> {
    BOOT_MEMORY_MAP.lock().as_slice().to_vec()
}

/// Heap start address (set during init)
static HEAP_START: AtomicUsize = AtomicUsize::new(0);
/// HHDM offset (higher half direct map)
static HHDM_OFFSET: AtomicU64 = AtomicU64::new(0xFFFF_8000_0000_0000);
/// Minimum heap size (64 MB) — fallback value
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const HEAP_SIZE_MINIMUM: usize = 64 * 1024 * 1024;
/// Maximum heap size (512 MB) — cap to leave RAM for framebuffer, page tables, DMA
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const HEAP_SIZE_MAXIMUM: usize = 512 * 1024 * 1024;

/// Actual heap size selected at boot (dynamic: 25% of detected RAM, clamped)
static HEAP_SIZE_ACTUAL: AtomicUsize = AtomicUsize::new(64 * 1024 * 1024);

/// Get the actual heap size (set dynamically at boot)
pub fn heap_size() -> usize {
    HEAP_SIZE_ACTUAL.load(Ordering::Relaxed)
}

/// Kept for backward compatibility — returns the dynamic heap size
#[deprecated(note = "Use memory::heap_size() instead")]
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const HEAP_SIZE: usize = 64 * 1024 * 1024;

/// Total physical memory detected at boot (set by main.rs)
static TOTAL_PHYSICAL_MEMORY: AtomicU64 = AtomicU64::new(0);

/// Store detected total physical memory
pub fn set_total_physical_memory(bytes: u64) {
    TOTAL_PHYSICAL_MEMORY.store(bytes, Ordering::SeqCst);
}

/// Get total physical memory in bytes
pub fn total_physical_memory() -> u64 {
    TOTAL_PHYSICAL_MEMORY.load(Ordering::Relaxed)
}

/// Compute dynamic heap size: 25% of total RAM, clamped to [64 MB, 512 MB]
pub fn compute_heap_size(total_ram: u64) -> usize {
    let quarter = (total_ram / 4) as usize;
    quarter.clamp(HEAP_SIZE_MINIMUM, HEAP_SIZE_MAXIMUM)
}

/// Initialize memory management with HHDM offset and dynamic heap size
pub fn initialize_with_hhdm_dynamic(hhdm_offset: u64, usable_base: u64, heap_bytes: usize) {
    // Store HHDM offset
    HHDM_OFFSET.store(hhdm_offset, Ordering::SeqCst);
    
    // Store actual heap size
    HEAP_SIZE_ACTUAL.store(heap_bytes, Ordering::SeqCst);
    
    // Use the first usable memory region via HHDM
    let heap_physical = usable_base;
    let heap_virt = hhdm_offset + heap_physical;
    
    HEAP_START.store(heap_virt as usize, Ordering::SeqCst);
    
    // Initialize the heap allocator with dynamic size
    heap::initialize_at(heap_virt as usize, heap_bytes);
    crate::log!("Heap initialized: {} MB at virt {:#x} (phys {:#x})", 
        heap_bytes / 1024 / 1024, heap_virt, heap_physical);
}

/// Initialize memory management with HHDM offset (legacy, uses HEAP_SIZE_MIN)
pub fn initialize_with_hhdm(hhdm_offset: u64, usable_base: u64) {
    initialize_with_hhdm_dynamic(hhdm_offset, usable_base, HEAP_SIZE_MINIMUM);
}

/// Initialize memory management (fallback - uses fixed address)
pub fn init() {
    // Fallback: try a fixed address in higher half
    // This may fail if not mapped!
    let heap_address = 0xFFFF_8000_0100_0000usize; // 1MB into HHDM
    HEAP_START.store(heap_address, Ordering::SeqCst);
    let size = HEAP_SIZE_MINIMUM;
    HEAP_SIZE_ACTUAL.store(size, Ordering::SeqCst);
    heap::initialize_at(heap_address, size);
    crate::log!("Heap initialized (fallback): {} MB at {:#x}", size / 1024 / 1024, heap_address);
}

/// Initialize memory for Android boot (no HHDM — flat physical addressing)
/// Called from android_main when booting without Limine.
pub fn initialize_android_heap(physical_base: u64, heap_bytes: usize) {
    // No HHDM offset — physical address IS the virtual address (MMU off)
    HHDM_OFFSET.store(0, Ordering::SeqCst);
    HEAP_SIZE_ACTUAL.store(heap_bytes, Ordering::SeqCst);
    HEAP_START.store(physical_base as usize, Ordering::SeqCst);
    
    heap::initialize_at(physical_base as usize, heap_bytes);
}

/// Get memory statistics
pub fn stats() -> MemoryStats {
    let (frames_total, frames_used) = frame::stats();
    MemoryStats {
        heap_used: heap::used(),
        heap_free: heap::free(),
        frames_used: frames_used as usize,
        frames_free: (frames_total - frames_used) as usize,
    }
}

/// Memory statistics
#[derive(Debug, Clone, Copy)]
// Public structure — visible outside this module.
pub struct MemoryStats {
    pub heap_used: usize,
    pub heap_free: usize,
    pub frames_used: usize,
    pub frames_free: usize,
}

/// Get the HHDM offset (for physical to virtual address translation)
pub fn hhdm_offset() -> u64 {
    HHDM_OFFSET.load(Ordering::Relaxed)
}

/// Convert physical address to virtual (via HHDM)
pub fn physical_to_virt(physical: u64) -> u64 {
    hhdm_offset() + physical
}

/// Convert virtual HHDM address to physical
pub fn virt_to_physical(virt: u64) -> Option<u64> {
    let hhdm = hhdm_offset();
    if virt >= hhdm {
        Some(virt - hhdm)
    } else {
        None
    }
}

/// Map an MMIO region into kernel space
/// Returns the virtual address to use for accessing the MMIO region
/// 
/// This maps the physical MMIO address directly using HHDM offset,
/// but also ensures the pages are marked with proper MMIO flags (no cache).
pub fn map_mmio(physical_address: u64, size: usize) -> Result<u64, &'static str> {
    // For Limine with HHDM, we try to use the HHDM mapping first
    // But MMIO regions may not be covered by HHDM, so we need to 
    // actually map them in the page tables
    
    let virt_address = physical_to_virt(physical_address);
    
    // Map pages with MMIO flags (present, writable, no cache, no execute)
    let page_size = 4096u64;
    let start_page = physical_address & !0xFFF;
    let end_page = (physical_address + size as u64 + 0xFFF) & !0xFFF;
    let number_pages = ((end_page - start_page) / page_size) as usize;
    
    crate::serial_println!("[MMIO] Mapping {:#x} -> {:#x} ({} pages)", 
        physical_address, virt_address, number_pages);
    
    // Map each page
    for i in 0..number_pages {
        let page_physical = start_page + (i as u64 * page_size);
        let page_virt = physical_to_virt(page_physical);
        
        paging::map_kernel_mmio_page(page_virt, page_physical)?;
    }
    
    // Flush TLB for the mapped region
    for i in 0..number_pages {
        let page_virt = physical_to_virt(start_page + (i as u64 * page_size));
                // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe {
            #[cfg(target_arch = "x86_64")]
            core::arch::asm!("invlpg [{}]", in(reg) page_virt, options(nostack, preserves_flags));
            #[cfg(not(target_arch = "x86_64"))]
            crate::arch::flush_tlb(page_virt);
        }
    }
    
    Ok(virt_address)
}

/// Unmap an MMIO region (optional, mainly for cleanup)
pub fn unmap_mmio(_virt_address: u64, _size: usize) {
    // TODO: Implement if needed for hot-unplug support
}

/// Read a u64 from a user process's address space
/// Used by ptrace for PTRACE_PEEK*
pub fn read_user_u64(_pid: u32, address: u64) -> Result<u64, i32> {
    // For now, just read from the address directly
    // In a real implementation, we would switch to the process's page table
    if !is_user_address(address) {
        return Err(-14); // EFAULT
    }
    
    // Safety: We've validated this is a user address
    let value = // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe { core::ptr::read_volatile(address as *// Compile-time constant — evaluated at compilation, zero runtime cost.
const u64) };
    Ok(value)
}

/// Write a u64 to a user process's address space  
/// Used by ptrace for PTRACE_POKE*
pub fn write_user_u64(_pid: u32, address: u64, value: u64) -> Result<(), i32> {
    // For now, just write to the address directly
    // In a real implementation, we would switch to the process's page table
    if !is_user_address(address) {
        return Err(-14); // EFAULT
    }
    
    // Safety: We've validated this is a user address
    unsafe { core::ptr::write_volatile(address as *mut u64, value) };
    Ok(())
}
