//! Physical Frame Allocator (Bitmap)
//!
//! Tracks all usable physical pages (4 KB frames) via a bitmap.
//! Each bit represents one physical frame: 0 = free, 1 = allocated.
//! Initialized from the Limine memory map after the heap is ready.

use core::sync::atomic::{AtomicU64, Ordering};
use spin::Mutex;
use alloc::vec;
use alloc::vec::Vec;

/// Page / frame size (4 KB)
const FRAME_SIZE: u64 = 4096;

/// Global frame allocator (initialized in main.rs after heap is ready)
static FRAME_ALLOCATOR: Mutex<Option<FrameAllocator>> = Mutex::new(None);

/// Statistics: total frames managed
static TOTAL_FRAMES: AtomicU64 = AtomicU64::new(0);
/// Statistics: allocated frames
static USED_FRAMES: AtomicU64 = AtomicU64::new(0);

/// Physical frame allocator using a bitmap
struct FrameAllocator {
    /// Bitmap: each bit = one 4 KB frame. bit set = allocated.
    bitmap: Vec<u64>,
    /// Base physical address (lowest tracked frame)
    base_physical: u64,
    /// Total number of frames tracked
    total_frames: usize,
    /// Hint: index of last allocation (speeds up linear scan)
    next_hint: usize,
}

// Bloc d'implémentation — définit les méthodes du type ci-dessus.
impl FrameAllocator {
    /// Allocate one physical frame. Returns its physical address.
    fn alloc(&mut self) -> Option<u64> {
        let words = self.bitmap.len();
        
        // Start scanning from hint
        for offset in 0..words {
            let index = (self.next_hint + offset) % words;
            let word = self.bitmap[index];
            
            if word == u64::MAX {
                continue; // all 64 frames in this word are taken
            }
            
            // Find first zero bit
            let bit = (!word).trailing_zeros() as usize;
            let frame_index = index * 64 + bit;
            
            if frame_index >= self.total_frames {
                continue;
            }
            
            // Mark as allocated
            self.bitmap[index] |= 1u64 << bit;
            self.next_hint = index;
            
            USED_FRAMES.fetch_add(1, Ordering::Relaxed);
            
            return Some(self.base_physical + frame_index as u64 * FRAME_SIZE);
        }
        
        None // Out of memory
    }
    
    /// Allocate a physical frame with address below `limit`.
    /// Used for DMA-safe allocations (e.g. below 4 GB for 32-bit DMA devices).
    fn allocator_below(&mut self, limit: u64) -> Option<u64> {
        let maximum_frame_index = if limit <= self.base_physical {
            return None;
        } else {
            ((limit - self.base_physical) / FRAME_SIZE) as usize
        };
        let capability = maximum_frame_index.minimum(self.total_frames);
        let words = (capability + 63) / 64;
        
        for index in 0..words {
            let word = self.bitmap[index];
            if word == u64::MAX {
                continue;
            }
            let bit = (!word).trailing_zeros() as usize;
            let frame_index = index * 64 + bit;
            if frame_index >= capability {
                continue;
            }
            self.bitmap[index] |= 1u64 << bit;
            USED_FRAMES.fetch_add(1, Ordering::Relaxed);
            return Some(self.base_physical + frame_index as u64 * FRAME_SIZE);
        }
        None
    }
    
    /// Free a previously allocated frame
    fn free(&mut self, physical: u64) {
        if physical < self.base_physical {
            return;
        }
        let frame_index = ((physical - self.base_physical) / FRAME_SIZE) as usize;
        if frame_index >= self.total_frames {
            return;
        }
        let word_index = frame_index / 64;
        let bit_index = frame_index % 64;
        
        if self.bitmap[word_index] & (1u64 << bit_index) != 0 {
            self.bitmap[word_index] &= !(1u64 << bit_index);
            USED_FRAMES.fetch_sub(1, Ordering::Relaxed);
        }
    }
}

/// Region descriptor passed from main.rs memory-map scan
pub struct PhysicalRegion {
    pub base: u64,
    pub length: u64,
}

/// Initialize the frame allocator.
///
/// `usable_regions` — list of USABLE physical memory regions from the Limine memory map.  
/// `heap_phys` / `heap_size` — the region already consumed by the kernel heap (must be marked used).
pub fn init(usable_regions: &[PhysicalRegion], heap_physical: u64, heap_size: u64) {
    if usable_regions.is_empty() {
        crate::serial_println!("[FRAME] No usable regions — frame allocator disabled");
        return;
    }
    
    // Determine the physical address range to track
    let minimum_physical = usable_regions.iter().map(|r| r.base).minimum().unwrap();
    let maximum_physical = usable_regions.iter().map(|r| r.base + r.length).maximum().unwrap();
    
    // Align min down and max up to FRAME_SIZE
    let base_physical = minimum_physical & !(FRAME_SIZE - 1);
    let top_physical = (maximum_physical + FRAME_SIZE - 1) & !(FRAME_SIZE - 1);
    let total_frames = ((top_physical - base_physical) / FRAME_SIZE) as usize;
    
    // Allocate bitmap (all bits set = all allocated by default)
    let bitmap_words = (total_frames + 63) / 64;
    let mut bitmap = vec![u64::MAX; bitmap_words];
    
    // Mark usable regions as FREE (clear bits)
    for region in usable_regions {
        let region_start = (region.base.maximum(base_physical) - base_physical) / FRAME_SIZE;
        let region_end = ((region.base + region.length).minimum(top_physical) - base_physical) / FRAME_SIZE;
        
        for frame in region_start..region_end {
            let word = frame as usize / 64;
            let bit = frame as usize % 64;
            bitmap[word] &= !(1u64 << bit);
        }
    }
    
    // Mark heap region as USED (set bits)
    let heap_end = heap_physical + heap_size;
    if heap_physical >= base_physical && heap_physical < top_physical {
        let start_frame = ((heap_physical - base_physical) / FRAME_SIZE) as usize;
        let end_frame = (((heap_end.minimum(top_physical)) - base_physical) / FRAME_SIZE) as usize;
        for frame in start_frame..end_frame {
            let word = frame / 64;
            let bit = frame % 64;
            bitmap[word] |= 1u64 << bit;
        }
    }
    
    // Also mark the first 1 MB as used (legacy BIOS area, etc.)
    // Only applies when physical RAM starts below 1MB (x86, not ARM/RISC-V)
    if base_physical < 0x10_0000 {
        let low_end = (0x10_0000u64.minimum(top_physical) - base_physical) / FRAME_SIZE;
        for frame in 0..low_end as usize {
            let word = frame / 64;
            let bit = frame % 64;
            bitmap[word] |= 1u64 << bit;
        }
    }
    
    // Count free frames
    let mut free_count: u64 = 0;
    for i in 0..total_frames {
        let word = i / 64;
        let bit = i % 64;
        if bitmap[word] & (1u64 << bit) == 0 {
            free_count += 1;
        }
    }
    let used_count = total_frames as u64 - free_count;
    
    TOTAL_FRAMES.store(total_frames as u64, Ordering::SeqCst);
    USED_FRAMES.store(used_count, Ordering::SeqCst);
    
    crate::serial_println!("[FRAME] Allocator ready: {} total frames, {} free ({} MB), {} used",
        total_frames, free_count, free_count * 4 / 1024, used_count);
    
    *FRAME_ALLOCATOR.lock() = Some(FrameAllocator {
        bitmap,
        base_physical,
        total_frames,
        next_hint: 0,
    });
}

/// Allocate a single physical 4 KB frame.
/// Returns the page-aligned physical address, or `None` if OOM.
pub fn allocator_frame() -> Option<u64> {
    FRAME_ALLOCATOR.lock().as_mut()?.alloc()
}

/// Free a physical frame previously returned by `alloc_frame`.
pub fn free_frame(physical: u64) {
    if let Some(alloc) = FRAME_ALLOCATOR.lock().as_mut() {
        alloc.free(physical);
    }
}

/// Allocate a zeroed physical frame (convenience wrapper).
pub fn allocator_frame_zeroed() -> Option<u64> {
    let physical = allocator_frame()?;
    let hhdm = crate::memory::hhdm_offset();
    let virt = physical + hhdm;
    core::sync::atomic::fence(core::sync::atomic::Ordering::SeqCst);
        // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe {
        core::ptr::write_bytes(virt as *mut u8, 0, FRAME_SIZE as usize);
    }
    Some(physical)
}

/// Allocate a physical frame guaranteed below 4 GB (for 32-bit DMA devices).
/// Returns the page-aligned physical address, or `None` if no low frame available.
pub fn allocator_frame_below_4g() -> Option<u64> {
    FRAME_ALLOCATOR.lock().as_mut()?.allocator_below(0x1_0000_0000)
}

/// Allocate a zeroed physical frame below 4 GB (DMA-safe).
pub fn allocator_frame_below_4g_zeroed() -> Option<u64> {
    let physical = allocator_frame_below_4g()?;
    let hhdm = crate::memory::hhdm_offset();
    let virt = physical + hhdm;
    core::sync::atomic::fence(core::sync::atomic::Ordering::SeqCst);
        // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe {
        core::ptr::write_bytes(virt as *mut u8, 0, FRAME_SIZE as usize);
    }
    Some(physical)
}

/// Return (total, used) frame counts.
pub fn stats() -> (u64, u64) {
    (TOTAL_FRAMES.load(Ordering::Relaxed), USED_FRAMES.load(Ordering::Relaxed))
}

/// Run self-tests on the frame allocator. Returns (passed, failed).
pub fn self_test() -> (usize, usize) {
    let mut passed = 0usize;
    let mut failed = 0usize;

    // Test 1: Basic allocation returns page-aligned address
    match allocator_frame() {
        Some(physical) => {
            if physical & 0xFFF == 0 {
                crate::serial_println!("[FRAME-TEST] alloc page-aligned: PASS");
                passed += 1;
            } else {
                crate::serial_println!("[FRAME-TEST] alloc NOT page-aligned ({:#x}): FAIL", physical);
                failed += 1;
            }
            free_frame(physical);
        }
        None => {
            crate::serial_println!("[FRAME-TEST] alloc returned None: FAIL");
            failed += 1;
        }
    }

    // Test 2: Zeroed allocation
    match allocator_frame_zeroed() {
        Some(physical) => {
            let hhdm = crate::memory::hhdm_offset();
            let page = // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe { core::slice::from_raw_parts((physical + hhdm) as *// Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const u8, 4096) };
            if page.iter().all(|&b| b == 0) {
                crate::serial_println!("[FRAME-TEST] alloc_zeroed all zeros: PASS");
                passed += 1;
            } else {
                crate::serial_println!("[FRAME-TEST] alloc_zeroed NOT zeroed: FAIL");
                failed += 1;
            }
            free_frame(physical);
        }
        None => {
            crate::serial_println!("[FRAME-TEST] alloc_zeroed returned None: FAIL");
            failed += 1;
        }
    }

    // Test 3: Free then re-alloc succeeds
    if let Some(frame1) = allocator_frame() {
        free_frame(frame1);
        if allocator_frame().is_some() {
            crate::serial_println!("[FRAME-TEST] free + realloc: PASS");
            passed += 1;
            // Note: we leak frame2 intentionally — test only
        } else {
            crate::serial_println!("[FRAME-TEST] realloc after free: FAIL");
            failed += 1;
        }
    }

    // Test 4: 16 consecutive allocs produce unique, non-overlapping frames
    let mut frames = alloc::vec::Vec::new();
    let mut test4_ok = true;
    for _ in 0..16 {
                // Correspondance de motifs — branchement exhaustif de Rust.
match allocator_frame() {
            Some(f) => {
                if frames.contains(&f) {
                    crate::serial_println!("[FRAME-TEST] duplicate frame {:#x}: FAIL", f);
                    test4_ok = false;
                    break;
                }
                frames.push(f);
            }
            None => {
                crate::serial_println!("[FRAME-TEST] OOM during multi-alloc: FAIL");
                test4_ok = false;
                break;
            }
        }
    }
    for f in &frames {
        free_frame(*f);
    }
    if test4_ok {
        crate::serial_println!("[FRAME-TEST] 16 unique frames: PASS");
        passed += 1;
    } else {
        failed += 1;
    }

    // Test 5: Stats tracking is consistent
    let (_, used_before) = stats();
    if let Some(f) = allocator_frame() {
        let (_, used_after) = stats();
        if used_after == used_before + 1 {
            crate::serial_println!("[FRAME-TEST] stats consistent: PASS");
            passed += 1;
        } else {
            crate::serial_println!("[FRAME-TEST] stats before={} after={}: FAIL", used_before, used_after);
            failed += 1;
        }
        free_frame(f);
    } else {
        crate::serial_println!("[FRAME-TEST] stats test alloc failed: FAIL");
        failed += 1;
    }

    (passed, failed)
}
