//! Swap Subsystem for TrustOS
//!
//! Provides page eviction (LRU) and swap-to-disk support.
//! Pages can be swapped out to a swap file/partition and paged back in
//! on demand via the page fault handler.
//!
//! Swap layout: a simple block map where each 4 KB page maps to a slot.

use alloc::boxed::Box;
use alloc::collections::BTreeMap;
use alloc::vec::Vec;
use spin::Mutex;
use core::sync::atomic::{AtomicBool, AtomicU64, AtomicU32, Ordering};

/// Page size
const PAGE_SIZE: u64 = 4096;

/// Maximum number of swap slots (256 MB of swap = 65536 pages)
const MAX_SWAP_SLOTS: usize = 65536;

/// Swap slot index (0 = not in swap)
pub type SwapSlot = u32;

/// Swap entry: maps a virtual page to a swap slot
#[derive(Clone, Copy, Debug)]
struct SwapEntry {
    /// Physical address of the page (when in memory), 0 if swapped out
    phys_addr: u64,
    /// Swap slot number (0 = never swapped)
    slot: SwapSlot,
    /// Owning CR3 (address space)
    cr3: u64,
    /// Virtual address
    virt_addr: u64,
    /// Access counter for LRU approximation (incremented on access)
    access_count: u32,
    /// Last access tick
    last_access: u64,
}

/// Global swap state
struct SwapState {
    /// Whether swap is enabled
    enabled: bool,
    /// Swap file path (on disk)
    swap_path: Option<&'static str>,
    /// Bitmap of free swap slots (true = in use)
    slot_bitmap: Vec<bool>,
    /// Total swap slots
    total_slots: usize,
    /// Used swap slots
    used_slots: usize,
    /// Map of swapped-out pages: (cr3, virt_page) -> swap slot
    swap_map: BTreeMap<(u64, u64), SwapSlot>,
    /// Page tracking for LRU eviction: (cr3, virt_page) -> SwapEntry
    page_tracker: BTreeMap<(u64, u64), SwapEntry>,
}

static SWAP: Mutex<SwapState> = Mutex::new(SwapState {
    enabled: false,
    swap_path: None,
    slot_bitmap: Vec::new(),
    total_slots: 0,
    used_slots: 0,
    swap_map: BTreeMap::new(),
    page_tracker: BTreeMap::new(),
});

/// Statistics
static PAGES_SWAPPED_OUT: AtomicU64 = AtomicU64::new(0);
static PAGES_SWAPPED_IN: AtomicU64 = AtomicU64::new(0);
static SWAP_ENABLED: AtomicBool = AtomicBool::new(false);

/// Initialize the swap subsystem with a swap file/partition
pub fn init(swap_path: &'static str, size_bytes: u64) {
    let slots = (size_bytes / PAGE_SIZE) as usize;
    let slots = slots.min(MAX_SWAP_SLOTS);
    
    let mut state = SWAP.lock();
    state.slot_bitmap = alloc::vec![false; slots];
    state.total_slots = slots;
    state.used_slots = 0;
    state.enabled = true;
    state.swap_path = Some(swap_path);
    
    SWAP_ENABLED.store(true, Ordering::SeqCst);
    crate::serial_println!("[SWAP] Initialized: {} slots ({} MB), path={}",
        slots, (slots * 4096) / (1024 * 1024), swap_path);
}

/// Enable swap without a backing file (in-memory page cache eviction only)
pub fn enable_anonymous(max_pages: usize) {
    let slots = max_pages.min(MAX_SWAP_SLOTS);
    let mut state = SWAP.lock();
    state.slot_bitmap = alloc::vec![false; slots];
    state.total_slots = slots;
    state.used_slots = 0;
    state.enabled = true;
    SWAP_ENABLED.store(true, Ordering::SeqCst);
    crate::serial_println!("[SWAP] Anonymous swap: {} slots ({} KB)", slots, slots * 4);
}

/// swapon syscall — enable swap on a file
pub fn swapon(path: &str) -> Result<(), &'static str> {
    // For now, use a fixed 64 MB swap area
    let size = 64 * 1024 * 1024u64;
    // Leak the string for 'static lifetime (OS never frees swap anyway)
    let static_path: &'static str = Box::leak(alloc::string::String::from(path).into_boxed_str());
    init(static_path, size);
    Ok(())
}

/// swapoff syscall — disable swap
pub fn swapoff(_path: &str) -> Result<(), &'static str> {
    let mut state = SWAP.lock();
    if !state.enabled {
        return Err("Swap not enabled");
    }
    // Swap in all pages before disabling
    // (in a real OS we'd need to page everything back in)
    state.enabled = false;
    state.used_slots = 0;
    state.swap_map.clear();
    SWAP_ENABLED.store(false, Ordering::SeqCst);
    crate::serial_println!("[SWAP] Disabled");
    Ok(())
}

/// Check if swap is enabled
pub fn is_enabled() -> bool {
    SWAP_ENABLED.load(Ordering::Relaxed)
}

/// Allocate a swap slot
fn alloc_slot(state: &mut SwapState) -> Option<SwapSlot> {
    for (i, used) in state.slot_bitmap.iter_mut().enumerate() {
        if !*used {
            *used = true;
            state.used_slots += 1;
            return Some((i + 1) as SwapSlot); // slot 0 reserved for "not swapped"
        }
    }
    None
}

/// Free a swap slot
fn free_slot(state: &mut SwapState, slot: SwapSlot) {
    if slot == 0 { return; }
    let idx = (slot - 1) as usize;
    if idx < state.slot_bitmap.len() {
        state.slot_bitmap[idx] = false;
        state.used_slots = state.used_slots.saturating_sub(1);
    }
}

/// Track a page for potential eviction
pub fn track_page(cr3: u64, virt_addr: u64, phys_addr: u64) {
    if !SWAP_ENABLED.load(Ordering::Relaxed) { return; }
    
    let key = (cr3, virt_addr & !0xFFF);
    let entry = SwapEntry {
        phys_addr,
        slot: 0,
        cr3,
        virt_addr: virt_addr & !0xFFF,
        access_count: 1,
        last_access: crate::logger::get_ticks(),
    };
    
    SWAP.lock().page_tracker.insert(key, entry);
}

/// Record page access (called on page fault handler for accessed pages)
pub fn touch_page(cr3: u64, virt_addr: u64) {
    if !SWAP_ENABLED.load(Ordering::Relaxed) { return; }
    
    let key = (cr3, virt_addr & !0xFFF);
    let mut state = SWAP.lock();
    if let Some(entry) = state.page_tracker.get_mut(&key) {
        entry.access_count = entry.access_count.saturating_add(1);
        entry.last_access = crate::logger::get_ticks();
    }
}

/// Untrack a page (when freed)
pub fn untrack_page(cr3: u64, virt_addr: u64) {
    if !SWAP_ENABLED.load(Ordering::Relaxed) { return; }
    
    let key = (cr3, virt_addr & !0xFFF);
    let mut state = SWAP.lock();
    if let Some(entry) = state.page_tracker.remove(&key) {
        if entry.slot != 0 {
            free_slot(&mut state, entry.slot);
        }
    }
    state.swap_map.remove(&key);
}

/// Select the least-recently-used page for eviction.
/// Returns (cr3, virt_addr, phys_addr) of the victim page.
fn select_lru_victim(state: &SwapState) -> Option<(u64, u64, u64)> {
    let mut best: Option<(&(u64, u64), &SwapEntry)> = None;
    let mut best_score = u64::MAX;
    
    for (key, entry) in state.page_tracker.iter() {
        // Only consider pages that are currently in memory
        if entry.phys_addr == 0 { continue; }
        // Skip kernel pages (cr3 = 0 convention)
        if entry.cr3 == 0 { continue; }
        
        // LRU score: lower = better victim
        // Combine access count and last access time
        let score = entry.last_access.saturating_mul(entry.access_count as u64 + 1);
        if score < best_score {
            best_score = score;
            best = Some((key, entry));
        }
    }
    
    best.map(|(_, entry)| (entry.cr3, entry.virt_addr, entry.phys_addr))
}

/// Try to evict a page to make room for a new allocation.
/// Returns the freed physical frame address, or None.
pub fn try_evict_page() -> Option<u64> {
    let mut state = SWAP.lock();
    if !state.enabled { return None; }
    
    let (cr3, virt_addr, phys_addr) = select_lru_victim(&state)?;
    let slot = alloc_slot(&mut state)?;
    
    // Write page data to swap slot (in-memory swap buffer for now)
    // In a real OS, this would write to disk
    write_swap_slot(&state, slot, phys_addr);
    
    // Update tracking
    let key = (cr3, virt_addr);
    if let Some(entry) = state.page_tracker.get_mut(&key) {
        entry.phys_addr = 0; // No longer in memory
        entry.slot = slot;
    }
    state.swap_map.insert(key, slot);
    
    // Unmap the page from the address space (mark as not present)
    // We modify the PTE to store the swap slot in bits 12..31
    unmap_for_swap(cr3, virt_addr, slot);
    
    PAGES_SWAPPED_OUT.fetch_add(1, Ordering::Relaxed);
    
    crate::log_debug!("[SWAP] Evicted page cr3={:#x} virt={:#x} -> slot {}",
        cr3, virt_addr, slot);
    
    Some(phys_addr)
}

/// Handle a page fault for a swapped-out page.
/// Returns true if the page was swapped back in.
pub fn handle_swap_fault(cr3: u64, fault_addr: u64) -> bool {
    let virt_page = fault_addr & !0xFFF;
    let key = (cr3, virt_page);
    
    let mut state = SWAP.lock();
    let slot = match state.swap_map.get(&key) {
        Some(&s) => s,
        None => return false,
    };
    
    if slot == 0 { return false; }
    
    // Allocate a new physical frame
    let new_phys = match crate::memory::frame::alloc_frame_zeroed() {
        Some(f) => f,
        None => return false, // OOM even after trying swap
    };
    
    // Read swap data back into the frame
    read_swap_slot(&state, slot, new_phys);
    
    // Update tracking
    if let Some(entry) = state.page_tracker.get_mut(&key) {
        entry.phys_addr = new_phys;
        let old_slot = entry.slot;
        entry.slot = 0;
        entry.access_count = 1;
        entry.last_access = crate::logger::get_ticks();
        free_slot(&mut state, old_slot);
    }
    state.swap_map.remove(&key);
    
    drop(state);
    
    // Re-map the page in the address space
    remap_after_swap(cr3, virt_page, new_phys);
    
    PAGES_SWAPPED_IN.fetch_add(1, Ordering::Relaxed);
    
    crate::log_debug!("[SWAP] Paged in cr3={:#x} virt={:#x} phys={:#x}",
        cr3, virt_page, new_phys);
    
    true
}

/// Swap statistics
pub fn stats() -> SwapStats {
    let state = SWAP.lock();
    SwapStats {
        enabled: state.enabled,
        total_slots: state.total_slots,
        used_slots: state.used_slots,
        pages_swapped_out: PAGES_SWAPPED_OUT.load(Ordering::Relaxed),
        pages_swapped_in: PAGES_SWAPPED_IN.load(Ordering::Relaxed),
        tracked_pages: state.page_tracker.len(),
    }
}

#[derive(Clone, Debug)]
pub struct SwapStats {
    pub enabled: bool,
    pub total_slots: usize,
    pub used_slots: usize,
    pub pages_swapped_out: u64,
    pub pages_swapped_in: u64,
    pub tracked_pages: usize,
}

// ============================================================================
// Swap I/O — in-memory swap buffer
// ============================================================================
// For a real disk-backed swap, these would do block I/O.
// If NVMe is available, swap I/O goes to disk. Otherwise, falls back to
// an in-memory buffer (still useful for page eviction and reclaiming frames).

/// In-memory swap storage fallback (one Vec<u8> per slot, used when NVMe unavailable)
static SWAP_PAGES: Mutex<BTreeMap<SwapSlot, Vec<u8>>> = Mutex::new(BTreeMap::new());

/// Base LBA on NVMe where the swap partition starts.
/// We reserve the last portion of the NVMe drive for swap.
/// Each page = 8 sectors (4096 / 512).
const SECTORS_PER_PAGE: u64 = 8;

/// Get the swap base LBA (last 64 MB of NVMe)
fn swap_base_lba() -> u64 {
    let cap = crate::nvme::capacity();
    let swap_sectors = (MAX_SWAP_SLOTS as u64) * SECTORS_PER_PAGE;
    if cap > swap_sectors {
        cap - swap_sectors
    } else {
        0 // Drive too small — use start (or fallback in-memory)
    }
}

fn write_swap_slot(_state: &SwapState, slot: SwapSlot, phys_addr: u64) {
    let hhdm = crate::memory::hhdm_offset();
    let src = unsafe { core::slice::from_raw_parts((phys_addr + hhdm) as *const u8, PAGE_SIZE as usize) };
    
    // Try NVMe first
    if crate::nvme::is_initialized() {
        let lba = swap_base_lba() + ((slot as u64 - 1) * SECTORS_PER_PAGE);
        if crate::nvme::write_sectors(lba, SECTORS_PER_PAGE as usize, src).is_ok() {
            return;
        }
    }
    
    // Fallback: in-memory
    let mut pages = SWAP_PAGES.lock();
    pages.insert(slot, src.to_vec());
}

fn read_swap_slot(_state: &SwapState, slot: SwapSlot, phys_addr: u64) {
    let hhdm = crate::memory::hhdm_offset();
    let dst = unsafe { core::slice::from_raw_parts_mut((phys_addr + hhdm) as *mut u8, PAGE_SIZE as usize) };
    
    // Try NVMe first
    if crate::nvme::is_initialized() {
        let lba = swap_base_lba() + ((slot as u64 - 1) * SECTORS_PER_PAGE);
        if crate::nvme::read_sectors(lba, SECTORS_PER_PAGE as usize, dst).is_ok() {
            return;
        }
    }
    
    // Fallback: in-memory
    let pages = SWAP_PAGES.lock();
    if let Some(data) = pages.get(&slot) {
        dst[..data.len()].copy_from_slice(data);
    } else {
        dst.fill(0);
    }
}

// ============================================================================
// PTE manipulation for swap
// ============================================================================

/// Unmap a page and store swap slot in the PTE (not-present, bits 1..31 = slot)
fn unmap_for_swap(cr3: u64, virt_addr: u64, slot: SwapSlot) {
    let hhdm = crate::memory::hhdm_offset();
    
    // Walk page tables to find the PTE
    let pml4 = unsafe { &mut *((cr3 + hhdm) as *mut [u64; 512]) };
    let pml4_idx = ((virt_addr >> 39) & 0x1FF) as usize;
    if pml4[pml4_idx] & 1 == 0 { return; }
    
    let pdpt_phys = pml4[pml4_idx] & !0xFFF;
    let pdpt = unsafe { &mut *((pdpt_phys + hhdm) as *mut [u64; 512]) };
    let pdpt_idx = ((virt_addr >> 30) & 0x1FF) as usize;
    if pdpt[pdpt_idx] & 1 == 0 { return; }
    
    let pd_phys = pdpt[pdpt_idx] & !0xFFF;
    let pd = unsafe { &mut *((pd_phys + hhdm) as *mut [u64; 512]) };
    let pd_idx = ((virt_addr >> 21) & 0x1FF) as usize;
    if pd[pd_idx] & 1 == 0 { return; }
    if pd[pd_idx] & (1 << 7) != 0 { return; } // Huge page, skip
    
    let pt_phys = pd[pd_idx] & !0xFFF;
    let pt = unsafe { &mut *((pt_phys + hhdm) as *mut [u64; 512]) };
    let pt_idx = ((virt_addr >> 12) & 0x1FF) as usize;
    
    // Store swap slot encoded in the PTE (not-present):
    // Bit 0 = 0 (not present)
    // Bits 1..31 = swap slot
    // Bit 62 = 1 (marker: "this is a swap PTE")
    pt[pt_idx] = ((slot as u64) << 1) | (1u64 << 62);
    
    // Flush TLB for this page
    unsafe { core::arch::asm!("invlpg [{}]", in(reg) virt_addr, options(nostack, preserves_flags)); }
}

/// Re-map a page after swapping it back in
fn remap_after_swap(cr3: u64, virt_addr: u64, phys_addr: u64) {
    let hhdm = crate::memory::hhdm_offset();
    
    let pml4 = unsafe { &mut *((cr3 + hhdm) as *mut [u64; 512]) };
    let pml4_idx = ((virt_addr >> 39) & 0x1FF) as usize;
    if pml4[pml4_idx] & 1 == 0 { return; }
    
    let pdpt_phys = pml4[pml4_idx] & !0xFFF;
    let pdpt = unsafe { &mut *((pdpt_phys + hhdm) as *mut [u64; 512]) };
    let pdpt_idx = ((virt_addr >> 30) & 0x1FF) as usize;
    if pdpt[pdpt_idx] & 1 == 0 { return; }
    
    let pd_phys = pdpt[pdpt_idx] & !0xFFF;
    let pd = unsafe { &mut *((pd_phys + hhdm) as *mut [u64; 512]) };
    let pd_idx = ((virt_addr >> 21) & 0x1FF) as usize;
    if pd[pd_idx] & 1 == 0 { return; }
    
    let pt_phys = pd[pd_idx] & !0xFFF;
    let pt = unsafe { &mut *((pt_phys + hhdm) as *mut [u64; 512]) };
    let pt_idx = ((virt_addr >> 12) & 0x1FF) as usize;
    
    // Restore as USER_DATA page (present, writable, user)
    let flags: u64 = 1 | (1 << 1) | (1 << 2); // PRESENT | WRITABLE | USER
    pt[pt_idx] = (phys_addr & !0xFFF) | flags;
    
    unsafe { core::arch::asm!("invlpg [{}]", in(reg) virt_addr, options(nostack, preserves_flags)); }
}

/// Check if a PTE is a swap entry (not present + swap marker)
pub fn is_swap_pte(pte_value: u64) -> bool {
    (pte_value & 1) == 0 && (pte_value & (1u64 << 62)) != 0
}

/// Extract swap slot from a swap PTE
pub fn swap_slot_from_pte(pte_value: u64) -> SwapSlot {
    ((pte_value >> 1) & 0x7FFF_FFFF) as SwapSlot
}
