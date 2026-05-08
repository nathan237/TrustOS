//! JARVIS data-gathering pipeline.
//!
//! Layered design:
//!   L1 — `trace_*()` inline helpers on the hot path (≤ 20 cycles when on,
//!         ≤ 2 cycles when off).
//!   L2 — Per-CPU lock-free ring buffers in `ring.rs`.
//!   L3 — Drainer (currently shell-driven via `trace dump`).
//!   L4 — Sinks (memory print today; UDP/file/JARVIS later).
//!
//! Safety contract for tracepoints:
//!   - Never allocate.
//!   - Never lock the heap.
//!   - Never recurse into another tracepoint.
//!   - Never block.

pub mod event;
pub mod ring;
#[cfg(target_arch = "x86_64")]
pub mod pmu;

pub use event::{TraceEvent, TraceKind, flag, EVENT_SIZE};
pub use ring::{PerCpuRing, RINGS, RING_CAPACITY, MAX_TRACE_CPUS};

use core::sync::atomic::{AtomicBool, AtomicU32, AtomicU64, Ordering};

/// Master enable switch. Default OFF — enable explicitly via `trace on`.
static TRACING_ENABLED: AtomicBool = AtomicBool::new(false);

/// Reentrancy guard per CPU (avoids tracepoint→alloc→tracepoint loops).
static REENTRY: [AtomicBool; MAX_TRACE_CPUS] = {
    const A: AtomicBool = AtomicBool::new(false);
    [A; MAX_TRACE_CPUS]
};

/// Per-kind sampling rate. 1 = full trace, N = keep 1 every N.
/// Indexed by `TraceKind as u16` (range 0..=100, we cap at 128).
static SAMPLE_RATES: [AtomicU32; 128] = {
    const A: AtomicU32 = AtomicU32::new(1);
    [A; 128]
};

/// Total events emitted across all CPUs (incl. dropped).
static TOTAL_EMITTED: AtomicU64 = AtomicU64::new(0);

#[inline(always)]
pub fn is_enabled() -> bool {
    TRACING_ENABLED.load(Ordering::Relaxed)
}

pub fn enable() {
    TRACING_ENABLED.store(true, Ordering::Release);
    crate::serial_println!("[TRACE] enabled");
}

pub fn disable() {
    TRACING_ENABLED.store(false, Ordering::Release);
    crate::serial_println!("[TRACE] disabled");
}

pub fn set_sample_rate(kind: TraceKind, rate: u32) {
    let idx = (kind as u16) as usize;
    if idx < SAMPLE_RATES.len() {
        SAMPLE_RATES[idx].store(rate.max(1), Ordering::Relaxed);
    }
}

#[inline(always)]
fn cpu_index() -> usize {
    let id = crate::sync::percpu::current_cpu_id() as usize;
    if id >= MAX_TRACE_CPUS { 0 } else { id }
}

#[inline(always)]
fn rdtsc() -> u64 {
    crate::arch::timestamp()
}

/// Decide whether this event should be kept under sampling.
#[inline(always)]
fn should_sample(kind: TraceKind) -> bool {
    let idx = (kind as u16) as usize;
    let rate = if idx < SAMPLE_RATES.len() {
        SAMPLE_RATES[idx].load(Ordering::Relaxed)
    } else {
        1
    };
    if rate <= 1 { return true; }
    // Cheap pseudo-random: low bits of TSC modulo rate.
    (rdtsc() as u32).wrapping_mul(2654435761) % rate == 0
}

/// Core push helper. Inlined into the hot path.
#[inline(always)]
fn push(kind: TraceKind, data: [u64; 4], flags: u8) {
    if !is_enabled() { return; }
    if !should_sample(kind) { return; }

    let cpu = cpu_index();

    // Reentrancy guard: if a tracepoint already firing on this CPU
    // (e.g. an alloc inside tracing init), drop silently.
    if REENTRY[cpu].swap(true, Ordering::Acquire) {
        return;
    }

    let ev = TraceEvent {
        tsc:   rdtsc(),
        kind:  kind as u16,
        cpu:   cpu as u8,
        flags,
        _pad:  0,
        data,
    };
    let _ = RINGS[cpu].push(ev);
    TOTAL_EMITTED.fetch_add(1, Ordering::Relaxed);

    REENTRY[cpu].store(false, Ordering::Release);
}

// ---------------------------------------------------------------------------
// Public hot-path helpers — keep tiny, all `#[inline(always)]`.
// ---------------------------------------------------------------------------

#[inline(always)]
pub fn trace_alloc(size: u32, class: u8, ptr: usize, latency_cyc: u32) {
    push(TraceKind::Alloc,
         [size as u64, class as u64, ptr as u64, latency_cyc as u64],
         0);
}

#[inline(always)]
pub fn trace_free(size: u32, class: u8, ptr: usize, latency_cyc: u32) {
    push(TraceKind::Free,
         [size as u64, class as u64, ptr as u64, latency_cyc as u64],
         0);
}

#[inline(always)]
pub fn trace_sched_switch(from_tid: u64, to_tid: u64, reason: u32, runtime_ns: u64) {
    push(TraceKind::SchedSwitch,
         [from_tid, to_tid, reason as u64, runtime_ns],
         0);
}

#[inline(always)]
pub fn trace_irq(vector: u8, latency_cyc: u32, entry: bool) {
    let kind = if entry { TraceKind::IrqEntry } else { TraceKind::IrqExit };
    push(kind, [vector as u64, latency_cyc as u64, 0, 0], flag::IRQ_CTX);
}

#[inline(always)]
pub fn trace_marker(tag0: u64, tag1: u64) {
    push(TraceKind::Marker, [tag0, tag1, 0, 0], 0);
}

#[inline(always)]
pub fn trace_pmu(cycles: u64, instructions: u64, l1_miss: u64, br_miss: u64) {
    push(TraceKind::PmuSample, [cycles, instructions, l1_miss, br_miss], 0);
}

// ---------------------------------------------------------------------------
// Init + introspection
// ---------------------------------------------------------------------------

pub fn init() {
    // Default sampling: full trace for everything. User can downsample at
    // runtime via the `trace rate <kind> <n>` shell command.
    for r in SAMPLE_RATES.iter() {
        r.store(1, Ordering::Relaxed);
    }
    // Pre-downsample very chatty kinds so a casual `trace on` doesn't
    // drown the rings instantly.
    set_sample_rate(TraceKind::Alloc, 32);
    set_sample_rate(TraceKind::Free,  32);
    set_sample_rate(TraceKind::IrqEntry, 8);
    set_sample_rate(TraceKind::IrqExit,  8);
    crate::serial_println!(
        "[TRACE] init: {} CPUs × {} events × {} B = {} KB",
        MAX_TRACE_CPUS, RING_CAPACITY, EVENT_SIZE,
        (MAX_TRACE_CPUS * RING_CAPACITY * EVENT_SIZE) / 1024
    );
}

pub struct Stats {
    pub enabled:       bool,
    pub total_emitted: u64,
    pub per_cpu_used:  [u64; MAX_TRACE_CPUS],
    pub per_cpu_drop:  [u64; MAX_TRACE_CPUS],
    pub per_cpu_writ:  [u64; MAX_TRACE_CPUS],
}

pub fn stats() -> Stats {
    let mut s = Stats {
        enabled:       is_enabled(),
        total_emitted: TOTAL_EMITTED.load(Ordering::Relaxed),
        per_cpu_used:  [0; MAX_TRACE_CPUS],
        per_cpu_drop:  [0; MAX_TRACE_CPUS],
        per_cpu_writ:  [0; MAX_TRACE_CPUS],
    };
    for i in 0..MAX_TRACE_CPUS {
        s.per_cpu_used[i] = RINGS[i].len();
        s.per_cpu_drop[i] = RINGS[i].dropped.load(Ordering::Relaxed);
        s.per_cpu_writ[i] = RINGS[i].written.load(Ordering::Relaxed);
    }
    s
}

/// Drain all rings into a borrowed slice; returns count drained.
/// Intended for shell `trace dump` and offline export.
pub fn drain_all(out: &mut [TraceEvent]) -> usize {
    let mut total = 0usize;
    for i in 0..MAX_TRACE_CPUS {
        if total >= out.len() { break; }
        let n = RINGS[i].drain(&mut out[total..]);
        total += n;
    }
    total
}

/// Reset counters and discard buffered events.
pub fn clear() {
    for i in 0..MAX_TRACE_CPUS {
        let h = RINGS[i].head.load(Ordering::Relaxed);
        RINGS[i].tail.store(h, Ordering::Release);
        RINGS[i].dropped.store(0, Ordering::Relaxed);
        RINGS[i].written.store(0, Ordering::Relaxed);
    }
    TOTAL_EMITTED.store(0, Ordering::Relaxed);
}
