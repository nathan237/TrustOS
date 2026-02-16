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
pub const COW_BIT: u64 = 1 << 9;

/// Reference counts for shared physical frames (phys_page_addr -> count)
static REFCOUNTS: Mutex<BTreeMap<u64, u32>> = Mutex::new(BTreeMap::new());

/// Increment reference count for a physical frame
pub fn ref_increment(phys: u64) {
    let page = phys & !0xFFF;
    let mut rc = REFCOUNTS.lock();
    let count = rc.entry(page).or_insert(1);
    *count += 1;
}

/// Decrement reference count. Returns true if frame should be freed.
pub fn ref_decrement(phys: u64) -> bool {
    let page = phys & !0xFFF;
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
pub fn ref_count(phys: u64) -> u32 {
    let page = phys & !0xFFF;
    REFCOUNTS.lock().get(&page).copied().unwrap_or(1)
}

/// Handle a COW page fault at the given address.
/// Called from page fault handler when a user-mode write protection violation occurs.
/// Returns `true` if the fault was a COW fault and was resolved.
pub fn handle_cow_fault(fault_addr: u64) -> bool {
    let page_addr = fault_addr & !0xFFF;
    let hhdm = super::hhdm_offset();

    let cr3: u64;
    unsafe { core::arch::asm!("mov {}, cr3", out(reg) cr3, options(nostack, preserves_flags)); }

    let pml4_idx = ((page_addr >> 39) & 0x1FF) as usize;
    let pdpt_idx = ((page_addr >> 30) & 0x1FF) as usize;
    let pd_idx   = ((page_addr >> 21) & 0x1FF) as usize;
    let pt_idx   = ((page_addr >> 12) & 0x1FF) as usize;

    let pml4 = unsafe { &*((cr3 + hhdm) as *const PageTable) };
    if !pml4.entries[pml4_idx].is_present() { return false; }
    let pdpt = unsafe { &*((pml4.entries[pml4_idx].phys_addr() + hhdm) as *const PageTable) };
    if !pdpt.entries[pdpt_idx].is_present() { return false; }
    let pd = unsafe { &*((pdpt.entries[pdpt_idx].phys_addr() + hhdm) as *const PageTable) };
    if !pd.entries[pd_idx].is_present() { return false; }
    let pt = unsafe { &mut *((pd.entries[pd_idx].phys_addr() + hhdm) as *mut PageTable) };
    if !pt.entries[pt_idx].is_present() { return false; }

    let flags = pt.entries[pt_idx].flags().bits();

    // Not a COW page → not our problem
    if flags & COW_BIT == 0 {
        return false;
    }

    let old_phys = pt.entries[pt_idx].phys_addr();
    let rc = ref_count(old_phys);

    if rc > 1 {
        // Shared page → allocate a private copy
        let new_phys = match frame::alloc_frame() {
            Some(p) => p,
            None => return false, // OOM
        };
        unsafe {
            core::ptr::copy_nonoverlapping(
                (old_phys + hhdm) as *const u8,
                (new_phys + hhdm) as *mut u8,
                PAGE_SIZE,
            );
        }
        ref_decrement(old_phys);
        let new_flags = (flags & !COW_BIT) | PageFlags::WRITABLE;
        pt.entries[pt_idx].set(new_phys, PageFlags::new(new_flags));
    } else {
        // Sole owner → just restore write permission
        let new_flags = (flags & !COW_BIT) | PageFlags::WRITABLE;
        pt.entries[pt_idx].set(old_phys, PageFlags::new(new_flags));
    }

    // Flush TLB entry
    unsafe { core::arch::asm!("invlpg [{}]", in(reg) page_addr, options(nostack, preserves_flags)); }
    true
}

/// Clone an address space with COW semantics.
///
/// All writable user-space pages in the parent become read-only + COW.
/// The child gets a fresh address space sharing the same physical pages.
/// On first write, the page fault handler allocates a private copy.
pub fn clone_cow(parent_cr3: u64) -> Option<AddressSpace> {
    let hhdm = super::hhdm_offset();
    let mut child = AddressSpace::new_with_kernel()?;
    let parent_pml4 = unsafe { &*((parent_cr3 + hhdm) as *const PageTable) };

    for pml4_idx in 0..256 {
        if !parent_pml4.entries[pml4_idx].is_present() { continue; }
        let pdpt = unsafe {
            &*((parent_pml4.entries[pml4_idx].phys_addr() + hhdm) as *const PageTable)
        };

        for pdpt_idx in 0..ENTRIES_PER_TABLE {
            if !pdpt.entries[pdpt_idx].is_present() { continue; }
            if pdpt.entries[pdpt_idx].flags().bits() & PageFlags::HUGE_PAGE != 0 { continue; }
            let pd = unsafe {
                &*((pdpt.entries[pdpt_idx].phys_addr() + hhdm) as *const PageTable)
            };

            for pd_idx in 0..ENTRIES_PER_TABLE {
                if !pd.entries[pd_idx].is_present() { continue; }
                if pd.entries[pd_idx].flags().bits() & PageFlags::HUGE_PAGE != 0 { continue; }
                let parent_pt = unsafe {
                    &mut *((pd.entries[pd_idx].phys_addr() + hhdm) as *mut PageTable)
                };

                for pt_idx in 0..ENTRIES_PER_TABLE {
                    if !parent_pt.entries[pt_idx].is_present() { continue; }

                    let phys  = parent_pt.entries[pt_idx].phys_addr();
                    let flags = parent_pt.entries[pt_idx].flags().bits();

                    // COW flags: clear WRITABLE, set COW bit
                    let cow_flags = (flags & !PageFlags::WRITABLE) | COW_BIT;

                    // Mark parent page as COW (read-only)
                    parent_pt.entries[pt_idx].set(phys, PageFlags::new(cow_flags));

                    let virt = ((pml4_idx as u64) << 39)
                             | ((pdpt_idx as u64) << 30)
                             | ((pd_idx   as u64) << 21)
                             | ((pt_idx   as u64) << 12);

                    // Map same physical page in child (also COW)
                    child.map_page(virt, phys, PageFlags::new(cow_flags))?;

                    // Track the sharing
                    ref_increment(phys);

                    // Flush parent TLB for this page
                    unsafe {
                        core::arch::asm!("invlpg [{}]", in(reg) virt,
                                         options(nostack, preserves_flags));
                    }
                }
            }
        }
    }

    Some(child)
}
