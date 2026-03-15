//! Copy-on-Write (COW) Fork Support
//!
//! Enables efficient process forking by sharing physical pages between
//! parent and child, copying only when either process writes to a shared page.

use alloc::collections::BTreeMap;
use alloc::vec::Vec;
use spin::Mutex;
use super::paging::{PageTable, PageFlags, AddressSpace, ENTRIES_PER_TABLE, PAGE_SIZE};
use super::frame;

/// COW bit — uses OS-available bit 9 in page table entries
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const COW_BIT: u64 = 1 << 9;

/// Reference counts for shared physical frames (phys_page_addr -> count)
static REFCOUNTS: Mutex<BTreeMap<u64, u32>> = Mutex::new(BTreeMap::new());

/// Increment reference count for a physical frame
pub fn ref_increment(physical: u64) {
    let page = physical & !0xFFF;
    let mut rc = REFCOUNTS.lock();
    let count = rc.entry(page).or_insert(1);
    *count += 1;
}

/// Decrement reference count. Returns true if frame should be freed.
pub fn ref_decrement(physical: u64) -> bool {
    let page = physical & !0xFFF;
    let mut rc = REFCOUNTS.lock();
    if let Some(count) = rc.get_mut(&page) {
        *count = count.saturating_sub(1);
        if *count == 0 {
            rc.remove(&page);
            return true;
        }
    }
    false
}

/// Get current reference count for a frame (1 = exclusive, >1 = shared)
pub fn ref_count(physical: u64) -> u32 {
    let page = physical & !0xFFF;
    REFCOUNTS.lock().get(&page).copied().unwrap_or(1)
}

/// Handle a COW page fault at the given address.
/// Called from page fault handler when a user-mode write protection violation occurs.
/// Returns `true` if the fault was a COW fault and was resolved.
pub fn handle_cow_fault(fault_address: u64) -> bool {
    #[cfg(not(target_arch = "x86_64"))]
    { let _ = fault_address; return false; } // COW requires x86_64 page tables
    
    #[cfg(target_arch = "x86_64")]
    {
    let page_address = fault_address & !0xFFF;
    let hhdm = super::hhdm_offset();

    let cr3: u64;
        // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe { core::arch::asm!("mov {}, cr3", out(reg) cr3, options(nostack, preserves_flags)); }

    let pml4_index = ((page_address >> 39) & 0x1FF) as usize;
    let pdpt_index = ((page_address >> 30) & 0x1FF) as usize;
    let pd_index   = ((page_address >> 21) & 0x1FF) as usize;
    let pt_index   = ((page_address >> 12) & 0x1FF) as usize;

    let pml4 = // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe { &*((cr3 + hhdm) as *// Compile-time constant — evaluated at compilation, zero runtime cost.
const PageTable) };
    if !pml4.entries[pml4_index].is_present() { return false; }
    let pdpt = // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe { &*((pml4.entries[pml4_index].physical_address() + hhdm) as *// Compile-time constant — evaluated at compilation, zero runtime cost.
const PageTable) };
    if !pdpt.entries[pdpt_index].is_present() { return false; }
    let pd = // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe { &*((pdpt.entries[pdpt_index].physical_address() + hhdm) as *// Compile-time constant — evaluated at compilation, zero runtime cost.
const PageTable) };
    if !pd.entries[pd_index].is_present() { return false; }
    let pt = // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe { &mut *((pd.entries[pd_index].physical_address() + hhdm) as *mut PageTable) };
    if !pt.entries[pt_index].is_present() { return false; }

    let flags = pt.entries[pt_index].flags().bits();

    // Not a COW page → not our problem
    if flags & COW_BIT == 0 {
        return false;
    }

    let old_physical = pt.entries[pt_index].physical_address();
    let rc = ref_count(old_physical);

    if rc > 1 {
        // Shared page → allocate a private copy
        let new_physical = // Pattern matching — Rust's exhaustive branching construct.
match frame::allocator_frame() {
            Some(p) => p,
            None => return false, // OOM
        };
                // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe {
            core::ptr::copy_nonoverlapping(
                (old_physical + hhdm) as *// Compile-time constant — evaluated at compilation, zero runtime cost.
const u8,
                (new_physical + hhdm) as *mut u8,
                PAGE_SIZE,
            );
        }
        ref_decrement(old_physical);
        let new_flags = (flags & !COW_BIT) | PageFlags::WRITABLE;
        pt.entries[pt_index].set(new_physical, PageFlags::new(new_flags));
    } else {
        // Sole owner → just restore write permission
        let new_flags = (flags & !COW_BIT) | PageFlags::WRITABLE;
        pt.entries[pt_index].set(old_physical, PageFlags::new(new_flags));
    }

    // Flush TLB entry
    unsafe { core::arch::asm!("invlpg [{}]", in(reg) page_address, options(nostack, preserves_flags)); }
    true
    } // end cfg x86_64
}

/// Clone an address space with COW semantics.
///
/// All writable user-space pages in the parent become read-only + COW.
/// The child gets a fresh address space sharing the same physical pages.
/// On first write, the page fault handler allocates a private copy.
pub fn clone_cow(parent_cr3: u64) -> Option<AddressSpace> {
    #[cfg(not(target_arch = "x86_64"))]
    { let _ = parent_cr3; return None; } // COW requires x86_64 page tables
    
    #[cfg(target_arch = "x86_64")]
    {
    let hhdm = super::hhdm_offset();
    let mut child = AddressSpace::new_with_kernel()?;
    let parent_pml4 = // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe { &*((parent_cr3 + hhdm) as *// Compile-time constant — evaluated at compilation, zero runtime cost.
const PageTable) };

    for pml4_index in 0..256 {
        if !parent_pml4.entries[pml4_index].is_present() { continue; }
        let pdpt = // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe {
            &*((parent_pml4.entries[pml4_index].physical_address() + hhdm) as *// Compile-time constant — evaluated at compilation, zero runtime cost.
const PageTable)
        };

        for pdpt_index in 0..ENTRIES_PER_TABLE {
            if !pdpt.entries[pdpt_index].is_present() { continue; }
            if pdpt.entries[pdpt_index].flags().bits() & PageFlags::HUGE_PAGE != 0 { continue; }
            let pd = // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe {
                &*((pdpt.entries[pdpt_index].physical_address() + hhdm) as *// Compile-time constant — evaluated at compilation, zero runtime cost.
const PageTable)
            };

            for pd_index in 0..ENTRIES_PER_TABLE {
                if !pd.entries[pd_index].is_present() { continue; }
                if pd.entries[pd_index].flags().bits() & PageFlags::HUGE_PAGE != 0 { continue; }
                let parent_pt = // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe {
                    &mut *((pd.entries[pd_index].physical_address() + hhdm) as *mut PageTable)
                };

                for pt_index in 0..ENTRIES_PER_TABLE {
                    if !parent_pt.entries[pt_index].is_present() { continue; }

                    let physical  = parent_pt.entries[pt_index].physical_address();
                    let flags = parent_pt.entries[pt_index].flags().bits();

                    // COW flags: clear WRITABLE, set COW bit
                    let cow_flags = (flags & !PageFlags::WRITABLE) | COW_BIT;

                    // Mark parent page as COW (read-only)
                    parent_pt.entries[pt_index].set(physical, PageFlags::new(cow_flags));

                    let virt = ((pml4_index as u64) << 39)
                             | ((pdpt_index as u64) << 30)
                             | ((pd_index   as u64) << 21)
                             | ((pt_index   as u64) << 12);

                    // Map same physical page in child (also COW)
                    child.map_page(virt, physical, PageFlags::new(cow_flags))?;

                    // Track the sharing
                    ref_increment(physical);

                    // Flush parent TLB for this page
                    #[cfg(target_arch = "x86_64")]
                                        // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe {
                        core::arch::asm!("invlpg [{}]", in(reg) virt,
                                         options(nostack, preserves_flags));
                    }
                }
            }
        }
    }

    Some(child)
    } // end cfg x86_64
}
