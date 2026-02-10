//! Memory Management Subsystem (Limine version)
//! 
//! Simple memory manager for Limine-booted kernel.
//! Includes paging support for process isolation.

pub mod heap;
pub mod paging;

use core::sync::atomic::{AtomicUsize, AtomicU64, Ordering};

pub use paging::{AddressSpace, PageFlags, validate_user_ptr, is_user_address, is_kernel_address};

/// Heap start address (set during init)
static HEAP_START: AtomicUsize = AtomicUsize::new(0);
/// HHDM offset (higher half direct map)
static HHDM_OFFSET: AtomicU64 = AtomicU64::new(0xFFFF_8000_0000_0000);
/// Kernel heap size — dynamically selected at boot based on available RAM.
/// Default: 64 MB (supports high-res framebuffers and 3D rendering).
/// With VirtualBox (1GB+ RAM), we can use 128 MB+ for better performance.
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

/// Initialize memory management with HHDM offset
pub fn init_with_hhdm(hhdm_offset: u64, usable_base: u64) {
    // Store HHDM offset for later use
    HHDM_OFFSET.store(hhdm_offset, Ordering::SeqCst);
    
    // Use the first usable memory region via HHDM
    let heap_phys = usable_base;
    let heap_virt = hhdm_offset + heap_phys;
    
    HEAP_START.store(heap_virt as usize, Ordering::SeqCst);
    
    // Initialize the heap allocator
    heap::init_at(heap_virt as usize);
    crate::log!("Heap initialized: {} KB at virt {:#x} (phys {:#x})", 
        HEAP_SIZE / 1024, heap_virt, heap_phys);
}

/// Initialize memory management (fallback - uses fixed address)
pub fn init() {
    // Fallback: try a fixed address in higher half
    // This may fail if not mapped!
    let heap_addr = 0xFFFF_8000_0100_0000usize; // 1MB into HHDM
    HEAP_START.store(heap_addr, Ordering::SeqCst);
    heap::init_at(heap_addr);
    crate::log!("Heap initialized (fallback): {} KB at {:#x}", HEAP_SIZE / 1024, heap_addr);
}

/// Get memory statistics
pub fn stats() -> MemoryStats {
    MemoryStats {
        heap_used: heap::used(),
        heap_free: heap::free(),
        frames_used: 0,
        frames_free: 0,
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
