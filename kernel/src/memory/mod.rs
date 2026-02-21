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

use core::sync::atomic::{AtomicUsize, AtomicU64, Ordering};

pub use paging::{AddressSpace, PageFlags, validate_user_ptr, is_user_address, is_kernel_address};

/// Heap start address (set during init)
static HEAP_START: AtomicUsize = AtomicUsize::new(0);
/// HHDM offset (higher half direct map)
static HHDM_OFFSET: AtomicU64 = AtomicU64::new(0xFFFF_8000_0000_0000);
/// Minimum heap size (64 MB) — fallback value
pub const HEAP_SIZE_MIN: usize = 64 * 1024 * 1024;
/// Maximum heap size (512 MB) — cap to leave RAM for framebuffer, page tables, DMA
pub const HEAP_SIZE_MAX: usize = 512 * 1024 * 1024;

/// Actual heap size selected at boot (dynamic: 25% of detected RAM, clamped)
static HEAP_SIZE_ACTUAL: AtomicUsize = AtomicUsize::new(64 * 1024 * 1024);

/// Get the actual heap size (set dynamically at boot)
pub fn heap_size() -> usize {
    HEAP_SIZE_ACTUAL.load(Ordering::Relaxed)
}

/// Kept for backward compatibility — returns the dynamic heap size
#[deprecated(note = "Use memory::heap_size() instead")]
pub const HEAP_SIZE: usize = 64 * 1024 * 1024;

/// Total physical memory detected at boot (set by main.rs)
static TOTAL_PHYS_MEMORY: AtomicU64 = AtomicU64::new(0);

/// Store detected total physical memory
pub fn set_total_physical_memory(bytes: u64) {
    TOTAL_PHYS_MEMORY.store(bytes, Ordering::SeqCst);
}

/// Get total physical memory in bytes
pub fn total_physical_memory() -> u64 {
    TOTAL_PHYS_MEMORY.load(Ordering::Relaxed)
}

/// Compute dynamic heap size: 25% of total RAM, clamped to [64 MB, 512 MB]
pub fn compute_heap_size(total_ram: u64) -> usize {
    let quarter = (total_ram / 4) as usize;
    quarter.clamp(HEAP_SIZE_MIN, HEAP_SIZE_MAX)
}

/// Initialize memory management with HHDM offset and dynamic heap size
pub fn init_with_hhdm_dynamic(hhdm_offset: u64, usable_base: u64, heap_bytes: usize) {
    // Store HHDM offset
    HHDM_OFFSET.store(hhdm_offset, Ordering::SeqCst);
    
    // Store actual heap size
    HEAP_SIZE_ACTUAL.store(heap_bytes, Ordering::SeqCst);
    
    // Use the first usable memory region via HHDM
    let heap_phys = usable_base;
    let heap_virt = hhdm_offset + heap_phys;
    
    HEAP_START.store(heap_virt as usize, Ordering::SeqCst);
    
    // Initialize the heap allocator with dynamic size
    heap::init_at(heap_virt as usize, heap_bytes);
    crate::log!("Heap initialized: {} MB at virt {:#x} (phys {:#x})", 
        heap_bytes / 1024 / 1024, heap_virt, heap_phys);
}

/// Initialize memory management with HHDM offset (legacy, uses HEAP_SIZE_MIN)
pub fn init_with_hhdm(hhdm_offset: u64, usable_base: u64) {
    init_with_hhdm_dynamic(hhdm_offset, usable_base, HEAP_SIZE_MIN);
}

/// Initialize memory management (fallback - uses fixed address)
pub fn init() {
    // Fallback: try a fixed address in higher half
    // This may fail if not mapped!
    let heap_addr = 0xFFFF_8000_0100_0000usize; // 1MB into HHDM
    HEAP_START.store(heap_addr, Ordering::SeqCst);
    let size = HEAP_SIZE_MIN;
    HEAP_SIZE_ACTUAL.store(size, Ordering::SeqCst);
    heap::init_at(heap_addr, size);
    crate::log!("Heap initialized (fallback): {} MB at {:#x}", size / 1024 / 1024, heap_addr);
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
pub fn phys_to_virt(phys: u64) -> u64 {
    hhdm_offset() + phys
}

/// Convert virtual HHDM address to physical
pub fn virt_to_phys(virt: u64) -> Option<u64> {
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
pub fn map_mmio(phys_addr: u64, size: usize) -> Result<u64, &'static str> {
    // For Limine with HHDM, we try to use the HHDM mapping first
    // But MMIO regions may not be covered by HHDM, so we need to 
    // actually map them in the page tables
    
    let virt_addr = phys_to_virt(phys_addr);
    
    // Map pages with MMIO flags (present, writable, no cache, no execute)
    let page_size = 4096u64;
    let start_page = phys_addr & !0xFFF;
    let end_page = (phys_addr + size as u64 + 0xFFF) & !0xFFF;
    let num_pages = ((end_page - start_page) / page_size) as usize;
    
    crate::serial_println!("[MMIO] Mapping {:#x} -> {:#x} ({} pages)", 
        phys_addr, virt_addr, num_pages);
    
    // Map each page
    for i in 0..num_pages {
        let page_phys = start_page + (i as u64 * page_size);
        let page_virt = phys_to_virt(page_phys);
        
        paging::map_kernel_mmio_page(page_virt, page_phys)?;
    }
    
    // Flush TLB for the mapped region
    for i in 0..num_pages {
        let page_virt = phys_to_virt(start_page + (i as u64 * page_size));
        unsafe {
            core::arch::asm!("invlpg [{}]", in(reg) page_virt, options(nostack, preserves_flags));
        }
    }
    
    Ok(virt_addr)
}

/// Unmap an MMIO region (optional, mainly for cleanup)
pub fn unmap_mmio(_virt_addr: u64, _size: usize) {
    // TODO: Implement if needed for hot-unplug support
}

/// Read a u64 from a user process's address space
/// Used by ptrace for PTRACE_PEEK*
pub fn read_user_u64(_pid: u32, addr: u64) -> Result<u64, i32> {
    // For now, just read from the address directly
    // In a real implementation, we would switch to the process's page table
    if !is_user_address(addr) {
        return Err(-14); // EFAULT
    }
    
    // Safety: We've validated this is a user address
    let value = unsafe { core::ptr::read_volatile(addr as *const u64) };
    Ok(value)
}

/// Write a u64 to a user process's address space  
/// Used by ptrace for PTRACE_POKE*
pub fn write_user_u64(_pid: u32, addr: u64, value: u64) -> Result<(), i32> {
    // For now, just write to the address directly
    // In a real implementation, we would switch to the process's page table
    if !is_user_address(addr) {
        return Err(-14); // EFAULT
    }
    
    // Safety: We've validated this is a user address
    unsafe { core::ptr::write_volatile(addr as *mut u64, value) };
    Ok(())
}
