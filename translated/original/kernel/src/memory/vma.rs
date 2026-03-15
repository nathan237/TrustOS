//! Virtual Memory Area (VMA) tracking for demand paging
//!
//! Tracks per-process memory mappings so the page fault handler can lazily
//! allocate physical frames for mmap'd regions on first access.

use alloc::collections::BTreeMap;
use alloc::vec::Vec;
use spin::Mutex;

/// VMA protection flags (matches Linux PROT_*)
pub mod prot {
    pub const PROT_NONE: u32 = 0;
    pub const PROT_READ: u32 = 1;
    pub const PROT_WRITE: u32 = 2;
    pub const PROT_EXEC: u32 = 4;
}

/// VMA mapping flags
pub mod flags {
    pub const MAP_ANONYMOUS: u32 = 0x20;
    pub const MAP_PRIVATE: u32 = 0x02;
    pub const MAP_SHARED: u32 = 0x01;
}

/// A virtual memory area (one contiguous mapping)
#[derive(Clone, Debug)]
pub struct Vma {
    /// Page-aligned start address
    pub start: u64,
    /// Page-aligned end address (exclusive)
    pub end: u64,
    /// Protection: PROT_READ | PROT_WRITE | PROT_EXEC
    pub prot: u32,
    /// Mapping flags: MAP_ANONYMOUS | MAP_PRIVATE etc.
    pub flags: u32,
}

impl Vma {
    /// Check if this VMA contains the given address
    pub fn contains(&self, addr: u64) -> bool {
        addr >= self.start && addr < self.end
    }
    
    /// Size in bytes
    pub fn size(&self) -> u64 {
        self.end - self.start
    }
}

/// Per-process VMA list, keyed by CR3 (page table root physical address)
static VMA_TABLE: Mutex<BTreeMap<u64, Vec<Vma>>> = Mutex::new(BTreeMap::new());

/// Register a new VMA for the current address space (identified by CR3)
pub fn add_vma(cr3: u64, vma: Vma) {
    let mut table = VMA_TABLE.lock();
    table.entry(cr3).or_insert_with(Vec::new).push(vma);
}

/// Look up a VMA containing the given fault address for the given CR3
pub fn lookup_vma(cr3: u64, addr: u64) -> Option<Vma> {
    let table = VMA_TABLE.lock();
    if let Some(vmas) = table.get(&cr3) {
        for vma in vmas {
            if vma.contains(addr) {
                return Some(vma.clone());
            }
        }
    }
    None
}

/// Remove VMAs overlapping a given range [start, end)
pub fn remove_vma_range(cr3: u64, start: u64, end: u64) {
    let mut table = VMA_TABLE.lock();
    if let Some(vmas) = table.get_mut(&cr3) {
        // Remove fully-contained VMAs, split partial overlaps
        let mut new_vmas = Vec::new();
        for vma in vmas.drain(..) {
            if vma.end <= start || vma.start >= end {
                // No overlap — keep
                new_vmas.push(vma);
            } else {
                // Overlap — split if partial
                if vma.start < start {
                    new_vmas.push(Vma {
                        start: vma.start,
                        end: start,
                        prot: vma.prot,
                        flags: vma.flags,
                    });
                }
                if vma.end > end {
                    new_vmas.push(Vma {
                        start: end,
                        end: vma.end,
                        prot: vma.prot,
                        flags: vma.flags,
                    });
                }
            }
        }
        *vmas = new_vmas;
    }
}

/// Update protection on VMAs overlapping [start, end)
pub fn update_vma_prot(cr3: u64, start: u64, end: u64, new_prot: u32) {
    let mut table = VMA_TABLE.lock();
    if let Some(vmas) = table.get_mut(&cr3) {
        for vma in vmas.iter_mut() {
            if vma.start < end && vma.end > start {
                vma.prot = new_prot;
            }
        }
    }
}

/// Clone VMAs from one address space to another (used by fork)
pub fn clone_vmas(src_cr3: u64, dst_cr3: u64) {
    let mut table = VMA_TABLE.lock();
    if let Some(vmas) = table.get(&src_cr3) {
        let cloned = vmas.clone();
        table.insert(dst_cr3, cloned);
    }
}

/// Release all VMAs for an address space
pub fn release_vmas(cr3: u64) {
    VMA_TABLE.lock().remove(&cr3);
}

/// List all VMAs for an address space (for /proc/[pid]/maps)
pub fn list_vmas(cr3: u64) -> Vec<Vma> {
    let table = VMA_TABLE.lock();
    table.get(&cr3).cloned().unwrap_or_default()
}

/// Convert VMA prot to PageFlags
pub fn prot_to_page_flags(prot_flags: u32) -> crate::memory::paging::PageFlags {
    use crate::memory::paging::PageFlags;
    
    let mut f = PageFlags::PRESENT | PageFlags::USER | PageFlags::NO_EXECUTE;
    if (prot_flags & prot::PROT_WRITE) != 0 {
        f |= PageFlags::WRITABLE;
    }
    if (prot_flags & prot::PROT_EXEC) != 0 {
        // Remove NX: present + user (no NO_EXECUTE)
        f = PageFlags::PRESENT | PageFlags::USER;
        if (prot_flags & prot::PROT_WRITE) != 0 {
            f |= PageFlags::WRITABLE;
        }
    }
    PageFlags::new(f)
}
