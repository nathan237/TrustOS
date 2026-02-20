//! RNG utilities — non-cryptographic fast path + CSPRNG via RDRAND/RDSEED

use core::sync::atomic::{AtomicU64, AtomicBool, Ordering};

static RNG_STATE: AtomicU64 = AtomicU64::new(0);
static HAS_RDRAND: AtomicBool = AtomicBool::new(false);
static HAS_RDSEED: AtomicBool = AtomicBool::new(false);

/// Detect CPU support for RDRAND/RDSEED (call once at boot)
pub fn init() {
    let rdrand = unsafe {
        let ecx: u32;
        // cpuid leaf 1: RDRAND is bit 30 of ECX.  Save/restore rbx since LLVM reserves it.
        core::arch::asm!(
            "push rbx",
            "cpuid",
            "pop rbx",
            in("eax") 1u32,
            lateout("eax") _,
            lateout("ecx") ecx,
            lateout("edx") _,
        );
        (ecx >> 30) & 1 == 1
    };
    let rdseed = unsafe {
        let ebx_out: u32;
        // cpuid leaf 7, sub-leaf 0: RDSEED is bit 18 of EBX.
        core::arch::asm!(
            "push rbx",
            "cpuid",
            "mov {out:e}, ebx",
            "pop rbx",
            in("eax") 7u32,
            in("ecx") 0u32,
            out = out(reg) ebx_out,
            lateout("eax") _,
            lateout("edx") _,
        );
        (ebx_out >> 18) & 1 == 1
    };
    HAS_RDRAND.store(rdrand, Ordering::Relaxed);
    HAS_RDSEED.store(rdseed, Ordering::Relaxed);
    crate::serial_println!("[RNG] RDRAND={} RDSEED={}", rdrand, rdseed);
}

fn seed_from_system() -> u64 {
    let ticks = crate::logger::get_ticks();
    let dt = crate::rtc::read_rtc();
    let rtc_mix = ((dt.year as u64) << 48)
        ^ ((dt.month as u64) << 40)
        ^ ((dt.day as u64) << 32)
        ^ ((dt.hour as u64) << 24)
        ^ ((dt.minute as u64) << 16)
        ^ ((dt.second as u64) << 8);
    let rdtsc = unsafe { core::arch::x86_64::_rdtsc() } as u64;
    rdtsc ^ ticks ^ rtc_mix ^ 0x9E3779B97F4A7C15
}

// ── Non-cryptographic fast PRNG (xorshift64*) ─────────────────────

pub fn next_u64() -> u64 {
    let mut state = RNG_STATE.load(Ordering::Relaxed);
    if state == 0 {
        state = seed_from_system();
    }

    // xorshift64*
    state ^= state >> 12;
    state ^= state << 25;
    state ^= state >> 27;
    state = state.wrapping_mul(0x2545F4914F6CDD1D);

    RNG_STATE.store(state, Ordering::Relaxed);
    state
}

pub fn fill_bytes(buf: &mut [u8]) {
    let mut i = 0;
    while i < buf.len() {
        let chunk = next_u64().to_le_bytes();
        for &b in &chunk {
            if i >= buf.len() {
                break;
            }
            buf[i] = b;
            i += 1;
        }
    }
}

/// Get a random u8
pub fn random_u8() -> u8 {
    next_u64() as u8
}

/// Get a random u32
pub fn random_u32() -> u32 {
    next_u64() as u32
}

// ── CSPRNG: hardware-backed RDRAND / RDSEED ───────────────────────

/// Read a single 64-bit value from RDRAND (retries up to 10 times).
/// Returns `None` if RDRAND not supported or all retries fail.
fn rdrand64_raw() -> Option<u64> {
    if !HAS_RDRAND.load(Ordering::Relaxed) {
        return None;
    }
    for _ in 0..10 {
        let val: u64;
        let ok: u8;
        unsafe {
            core::arch::asm!(
                "rdrand {v}",
                "setc {ok}",
                v = out(reg) val,
                ok = out(reg_byte) ok,
            );
        }
        if ok != 0 {
            return Some(val);
        }
    }
    None
}

/// Read a single 64-bit value from RDSEED (retries up to 10 times).
fn rdseed64_raw() -> Option<u64> {
    if !HAS_RDSEED.load(Ordering::Relaxed) {
        return None;
    }
    for _ in 0..10 {
        let val: u64;
        let ok: u8;
        unsafe {
            core::arch::asm!(
                "rdseed {v}",
                "setc {ok}",
                v = out(reg) val,
                ok = out(reg_byte) ok,
            );
        }
        if ok != 0 {
            return Some(val);
        }
    }
    None
}

/// Cryptographically secure random u64 — prefers RDSEED, falls back to RDRAND,
/// then to xorshift mixed with RDTSC.
pub fn secure_random_u64() -> u64 {
    if let Some(v) = rdseed64_raw() {
        return v;
    }
    if let Some(v) = rdrand64_raw() {
        return v;
    }
    // Fallback: mix PRNG with RDTSC for some entropy
    let ts = unsafe { core::arch::x86_64::_rdtsc() } as u64;
    next_u64() ^ ts
}

/// Cryptographically secure random u32.
pub fn secure_random_u32() -> u32 {
    secure_random_u64() as u32
}

/// Fill a buffer with cryptographically secure random bytes.
pub fn secure_fill_bytes(buf: &mut [u8]) {
    let mut i = 0;
    while i < buf.len() {
        let chunk = secure_random_u64().to_le_bytes();
        for &b in &chunk {
            if i >= buf.len() {
                break;
            }
            buf[i] = b;
            i += 1;
        }
    }
}

/// Returns true if hardware CSPRNG (RDRAND) is available.
pub fn has_hw_rng() -> bool {
    HAS_RDRAND.load(Ordering::Relaxed)
}
