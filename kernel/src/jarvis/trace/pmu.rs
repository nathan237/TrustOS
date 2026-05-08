//! PMU (Performance Monitoring Unit) — programmable counters on x86_64.
//!
//! Skylake architectural PMC: 4 general counters (40-bit), accessible via
//! `RDPMC` once enabled in `IA32_PERFEVTSELx` MSRs. Designed as a passive
//! data source for JARVIS — read once per timer tick and emit a sample.
//!
//! Encoding of `IA32_PERFEVTSELx` (MSR 0x186 + n):
//!   bits[7:0]   = event_select
//!   bits[15:8]  = umask
//!   bit 16      = USR (count user mode)
//!   bit 17      = OS  (count kernel mode)
//!   bit 22      = EN  (enable counter)
//!
//! Events programmed (Skylake, INTEL Vol. 3B 19.3):
//!   PMC0 = 0x3C 0x00 — UnHalted Core Cycles
//!   PMC1 = 0xC0 0x00 — Instructions Retired
//!   PMC2 = 0xD1 0x01 — MEM_LOAD_RETIRED.L1_MISS
//!   PMC3 = 0xC5 0x00 — Branch Mispredict Retired

use core::sync::atomic::{AtomicBool, Ordering};

static PMU_ENABLED: AtomicBool = AtomicBool::new(false);

const MSR_PERFEVTSEL0: u32 = 0x186;
const MSR_PMC0:        u32 = 0xC1;     // RDMSR address (RDPMC uses index 0..3)

#[inline(always)]
unsafe fn wrmsr(msr: u32, val: u64) {
    let lo = val as u32;
    let hi = (val >> 32) as u32;
    core::arch::asm!("wrmsr", in("ecx") msr, in("eax") lo, in("edx") hi);
}

#[inline(always)]
unsafe fn rdmsr(msr: u32) -> u64 {
    let lo: u32;
    let hi: u32;
    core::arch::asm!("rdmsr", in("ecx") msr, out("eax") lo, out("edx") hi);
    ((hi as u64) << 32) | (lo as u64)
}

#[inline(always)]
unsafe fn rdpmc(idx: u32) -> u64 {
    let lo: u32;
    let hi: u32;
    core::arch::asm!("rdpmc", in("ecx") idx, out("eax") lo, out("edx") hi);
    ((hi as u64) << 32) | (lo as u64)
}

/// Encode an EVENTSEL value with USR+OS+EN set.
#[inline]
const fn evtsel(event: u8, umask: u8) -> u64 {
    (event as u64)
        | ((umask as u64) << 8)
        | (1 << 16)   // USR
        | (1 << 17)   // OS
        | (1 << 22)   // EN
}

/// Program the 4 general PMCs and enable user-mode RDPMC.
///
/// Returns `true` on success. Safe to call multiple times.
pub fn init() -> bool {
    // Sanity: CPUID.0AH must report ≥4 GP counters; bail if not Skylake-like.
    let (eax, _ebx, _ecx, _edx): (u32, u32, u32, u32);
    unsafe {
        core::arch::asm!(
            "push rbx",
            "cpuid",
            "mov rsi, rbx",
            "pop rbx",
            inout("eax") 0x0Au32 => eax,
            out("esi") _ebx,
            out("ecx") _ecx,
            out("edx") _edx,
        );
    }
    // EAX[15:8] = number of GP counters per logical CPU
    let n_gp = ((eax >> 8) & 0xff) as u8;
    if n_gp < 4 {
        crate::serial_println!("[PMU] only {} GP counters — disabled", n_gp);
        return false;
    }

    unsafe {
        // Enable RDPMC in user mode via CR4.PCE (bit 8)
        let cr4: u64;
        core::arch::asm!("mov {}, cr4", out(reg) cr4);
        core::arch::asm!("mov cr4, {}", in(reg) cr4 | (1 << 8));

        wrmsr(MSR_PERFEVTSEL0 + 0, evtsel(0x3C, 0x00)); // cycles
        wrmsr(MSR_PERFEVTSEL0 + 1, evtsel(0xC0, 0x00)); // insn retired
        wrmsr(MSR_PERFEVTSEL0 + 2, evtsel(0xD1, 0x01)); // L1 miss (Skylake)
        wrmsr(MSR_PERFEVTSEL0 + 3, evtsel(0xC5, 0x00)); // branch mispredict
    }

    PMU_ENABLED.store(true, Ordering::Release);
    crate::serial_println!("[PMU] {} GP counters armed (cycles, insn, L1miss, brmiss)", n_gp);
    true
}

#[inline]
pub fn enabled() -> bool {
    PMU_ENABLED.load(Ordering::Relaxed)
}

#[derive(Clone, Copy, Default, Debug)]
pub struct Snapshot {
    pub cycles:      u64,
    pub instructions: u64,
    pub l1_miss:     u64,
    pub br_miss:     u64,
}

/// Read all 4 counters. Cheap (~50 cycles total).
#[inline]
pub fn snapshot() -> Snapshot {
    if !enabled() {
        return Snapshot::default();
    }
    unsafe {
        Snapshot {
            cycles:       rdpmc(0),
            instructions: rdpmc(1),
            l1_miss:      rdpmc(2),
            br_miss:      rdpmc(3),
        }
    }
}

/// Compute deltas + emit a `PmuSample` trace event. Cheap, idempotent on disable.
pub fn sample_into_trace() {
    if !enabled() { return; }
    let s = snapshot();
    crate::jarvis::trace::trace_pmu(s.cycles, s.instructions, s.l1_miss, s.br_miss);
}

/// Read the IA32_PERF_GLOBAL_STATUS MSR for diagnostics (best-effort).
pub fn perf_global_status() -> u64 {
    if !enabled() { return 0; }
    unsafe { rdmsr(0x38E) }
}

#[allow(dead_code)]
const _: u32 = MSR_PMC0; // suppress unused
