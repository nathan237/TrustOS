//! Kernel Heap Allocator
//! 
//! Tracked heap allocator for kernel memory allocations.
//! Wraps linked_list_allocator with devtools allocation tracking
//! and a per-CPU magazine cache for small allocations to reduce
//! global lock contention under SMP load.

use linked_list_allocator::LockedHeap;
use core::alloc::{GlobalAlloc, Layout};
use core::cell::UnsafeCell;
use core::sync::atomic::{AtomicU64, AtomicUsize, Ordering};

/// Inner heap allocator (global, mutex-protected)
static INNER: LockedHeap = LockedHeap::empty();

// ── Per-CPU magazine cache ────────────────────────────────────────────────
// Each CPU keeps a small free-list per size class. Allocations / deallocs
// of cached sizes hit the per-CPU cache first, avoiding the global mutex.
//
// Size classes: 16, 32, 64, 128, 256 bytes (covers most Box<small struct>
// and Vec growth patterns in the kernel hot path).

const NUM_SIZE_CLASSES: usize = 5;
const SIZE_CLASSES: [usize; NUM_SIZE_CLASSES] = [16, 32, 64, 128, 256];
const MAGAZINE_CAPACITY: usize = 32;          // entries per class per CPU
const MAX_HEAP_CPUS: usize = 64;              // matches cpu::smp::MAX_CPUS

/// Magazine: simple LIFO stack of free pointers for a single size class.
/// Accessed only by its owning CPU → no locking required, but we wrap in
/// `UnsafeCell` and rely on disabling preemption during access.
#[repr(C, align(64))]                         // cache-line aligned to avoid false sharing
struct Magazine {
    /// Number of entries currently cached
    count: UnsafeCell<usize>,
    /// LIFO stack of cached pointers
    slots: UnsafeCell<[*mut u8; MAGAZINE_CAPACITY]>,
}

unsafe impl Sync for Magazine {}

impl Magazine {
    const fn new() -> Self {
        Self {
            count: UnsafeCell::new(0),
            slots: UnsafeCell::new([core::ptr::null_mut(); MAGAZINE_CAPACITY]),
        }
    }
}

/// Per-CPU array of magazines (one per size class).
#[repr(C, align(64))]
struct CpuMagazines {
    mags: [Magazine; NUM_SIZE_CLASSES],
}

impl CpuMagazines {
    const fn new() -> Self {
        Self {
            mags: [Magazine::new(), Magazine::new(), Magazine::new(),
                   Magazine::new(), Magazine::new()],
        }
    }
}

/// Global per-CPU magazine table
static CPU_MAGS: [CpuMagazines; MAX_HEAP_CPUS] = {
    const INIT: CpuMagazines = CpuMagazines::new();
    [INIT; MAX_HEAP_CPUS]
};

/// Magazine statistics (for `bench mt` / `perf` validation)
static MAG_HITS: AtomicU64 = AtomicU64::new(0);
static MAG_MISSES: AtomicU64 = AtomicU64::new(0);
static MAG_FREES_CACHED: AtomicU64 = AtomicU64::new(0);
static MAG_FREES_GLOBAL: AtomicU64 = AtomicU64::new(0);

/// Whether the magazine cache is armed (set after SMP & per-CPU init).
static MAG_ENABLED: core::sync::atomic::AtomicBool =
    core::sync::atomic::AtomicBool::new(false);

#[inline]
fn size_class_index(size: usize, align: usize) -> Option<usize> {
    // Only cache power-of-2-aligned blocks ≤ 256B with align ≤ 16
    if align > 16 { return None; }
    for (i, &cls) in SIZE_CLASSES.iter().enumerate() {
        if size <= cls { return Some(i); }
    }
    None
}

/// Try to satisfy an allocation from the local CPU magazine.
/// Returns `null` on miss (caller should fall back to global heap).
#[inline]
unsafe fn mag_alloc(class_idx: usize) -> *mut u8 {
    if !MAG_ENABLED.load(Ordering::Relaxed) {
        return core::ptr::null_mut();
    }
    // Disable interrupts to prevent preemption between cpu_id read and
    // magazine access — otherwise a different CPU could trash our slot.
    let flags = irq_save();
    let cpu = crate::cpu::smp::current_cpu_id() as usize;
    if cpu >= MAX_HEAP_CPUS {
        irq_restore(flags);
        return core::ptr::null_mut();
    }

    let mag = &CPU_MAGS[cpu].mags[class_idx];
    let count_ptr = mag.count.get();
    let slots_ptr = mag.slots.get();
    let count = *count_ptr;
    if count == 0 {
        irq_restore(flags);
        MAG_MISSES.fetch_add(1, Ordering::Relaxed);
        return core::ptr::null_mut();
    }
    let new_count = count - 1;
    let p = (*slots_ptr)[new_count];
    *count_ptr = new_count;
    irq_restore(flags);
    MAG_HITS.fetch_add(1, Ordering::Relaxed);
    p
}

/// Try to push a freed block back into the local CPU magazine.
/// Returns `true` if cached, `false` if magazine full (caller frees globally).
#[inline]
unsafe fn mag_free(class_idx: usize, ptr: *mut u8) -> bool {
    if !MAG_ENABLED.load(Ordering::Relaxed) {
        return false;
    }
    let flags = irq_save();
    let cpu = crate::cpu::smp::current_cpu_id() as usize;
    if cpu >= MAX_HEAP_CPUS {
        irq_restore(flags);
        return false;
    }

    let mag = &CPU_MAGS[cpu].mags[class_idx];
    let count_ptr = mag.count.get();
    let slots_ptr = mag.slots.get();
    let count = *count_ptr;
    if count >= MAGAZINE_CAPACITY {
        irq_restore(flags);
        MAG_FREES_GLOBAL.fetch_add(1, Ordering::Relaxed);
        return false;
    }
    (*slots_ptr)[count] = ptr;
    *count_ptr = count + 1;
    irq_restore(flags);
    MAG_FREES_CACHED.fetch_add(1, Ordering::Relaxed);
    true
}

/// Save & disable interrupts. Returns previous IF state.
#[inline(always)]
unsafe fn irq_save() -> u64 {
    #[cfg(target_arch = "x86_64")]
    {
        let mut flags: u64;
        core::arch::asm!(
            "pushfq; pop {}; cli",
            out(reg) flags,
            options(nomem, preserves_flags)
        );
        flags
    }
    #[cfg(not(target_arch = "x86_64"))]
    { 0 }
}

/// Restore interrupt state from `irq_save`.
#[inline(always)]
unsafe fn irq_restore(flags: u64) {
    #[cfg(target_arch = "x86_64")]
    {
        if flags & (1 << 9) != 0 {
            core::arch::asm!("sti", options(nomem, nostack));
        }
    }
    #[cfg(not(target_arch = "x86_64"))]
    { let _ = flags; }
}

/// Enable the per-CPU magazine cache (called from main after SMP init).
pub fn enable_percpu_cache() {
    MAG_ENABLED.store(true, Ordering::Release);
    crate::serial_println!("[HEAP] per-CPU magazine cache enabled ({} CPUs × {} classes × {} slots)",
        MAX_HEAP_CPUS, NUM_SIZE_CLASSES, MAGAZINE_CAPACITY);
}

/// Get magazine cache stats: (hits, misses, frees_cached, frees_global)
pub fn magazine_stats() -> (u64, u64, u64, u64) {
    (MAG_HITS.load(Ordering::Relaxed),
     MAG_MISSES.load(Ordering::Relaxed),
     MAG_FREES_CACHED.load(Ordering::Relaxed),
     MAG_FREES_GLOBAL.load(Ordering::Relaxed))
}

/// Tracked wrapper that forwards to LockedHeap + records stats
struct TrackedAllocator;

unsafe impl GlobalAlloc for TrackedAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        // JARVIS trace gate DISABLED for boot-loop isolation.
        let trace_on = false;
        let t0: u64 = 0;
        let _ = t0;

        let class_opt = size_class_index(layout.size(), layout.align());

        // Fast path: per-CPU magazine for small power-aligned blocks
        let ptr = if let Some(idx) = class_opt {
            let p = unsafe { mag_alloc(idx) };
            if !p.is_null() {
                p
            } else {
                let cls_layout = Layout::from_size_align(SIZE_CLASSES[idx], 16)
                    .unwrap_or(layout);
                unsafe { INNER.alloc(cls_layout) }
            }
        } else {
            // Slow path: global heap
            unsafe { INNER.alloc(layout) }
        };

        if !ptr.is_null() {
            crate::devtools::track_alloc(layout.size());
        }

        if trace_on && !ptr.is_null() {
            let lat = (crate::arch::timestamp().wrapping_sub(t0)) as u32;
            let class = class_opt.map(|c| c as u8).unwrap_or(u8::MAX);
            #[cfg(feature = "jarvis")]
            crate::jarvis::trace::trace_alloc(
                layout.size() as u32, class, ptr as usize, lat,
            );
            let _ = (lat, class);
        }
        ptr
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        // JARVIS trace gate DISABLED for boot-loop isolation.
        let trace_on = false;
        let t0: u64 = 0;
        let _ = t0;

        let class_opt = size_class_index(layout.size(), layout.align());

        // Fast path: try cache the freed block in the per-CPU magazine
        if let Some(idx) = class_opt {
            if unsafe { mag_free(idx, ptr) } {
                crate::devtools::track_dealloc(layout.size());
                if trace_on {
                    let lat = (crate::arch::timestamp().wrapping_sub(t0)) as u32;
                    #[cfg(feature = "jarvis")]
                    crate::jarvis::trace::trace_free(
                        layout.size() as u32, idx as u8, ptr as usize, lat,
                    );
                    let _ = lat;
                }
                return;
            }
            // Magazine full — free at class size to match alloc path
            let cls_layout = Layout::from_size_align(SIZE_CLASSES[idx], 16)
                .unwrap_or(layout);
            unsafe { INNER.dealloc(ptr, cls_layout); }
            crate::devtools::track_dealloc(layout.size());
            if trace_on {
                let lat = (crate::arch::timestamp().wrapping_sub(t0)) as u32;
                #[cfg(feature = "jarvis")]
                crate::jarvis::trace::trace_free(
                    layout.size() as u32, idx as u8, ptr as usize, lat,
                );
                let _ = lat;
            }
            return;
        }

        unsafe { INNER.dealloc(ptr, layout); }
        crate::devtools::track_dealloc(layout.size());
        if trace_on {
            let lat = (crate::arch::timestamp().wrapping_sub(t0)) as u32;
            #[cfg(feature = "jarvis")]
            crate::jarvis::trace::trace_free(
                layout.size() as u32, u8::MAX, ptr as usize, lat,
            );
            let _ = lat;
        }
    }
}

#[global_allocator]
static ALLOCATOR: TrackedAllocator = TrackedAllocator;

/// Heap base address (set by init_at, exposed for diagnostics).
static HEAP_BASE: AtomicUsize = AtomicUsize::new(0);
static HEAP_SIZE: AtomicUsize = AtomicUsize::new(0);

/// Initialize kernel heap at specified address with given size
pub fn init_at(heap_start: usize, heap_size: usize) {
    unsafe {
        INNER.lock().init(heap_start as *mut u8, heap_size);
    }
    HEAP_BASE.store(heap_start, Ordering::Release);
    HEAP_SIZE.store(heap_size, Ordering::Release);
}

/// Get used heap space in bytes
pub fn used() -> usize {
    INNER.lock().used()
}

/// Get free heap space in bytes
pub fn free() -> usize {
    INNER.lock().free()
}

/// Allocate memory with alignment
/// Returns None if allocation fails
pub fn allocate(size: usize, align: usize) -> Option<*mut u8> {
    use core::alloc::Layout;
    
    let layout = Layout::from_size_align(size, align).ok()?;
    let ptr = unsafe {
        alloc::alloc::alloc(layout)
    };
    
    if ptr.is_null() {
        None
    } else {
        Some(ptr)
    }
}

/// Deallocate memory
/// 
/// # Safety
/// - ptr must have been allocated by this allocator
/// - size and align must match the original allocation
pub unsafe fn deallocate(ptr: *mut u8, size: usize, align: usize) {
    use core::alloc::Layout;
    
    if let Ok(layout) = Layout::from_size_align(size, align) {
        alloc::alloc::dealloc(ptr, layout);
    }
}
