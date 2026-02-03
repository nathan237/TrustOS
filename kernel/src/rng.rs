//! Simple RNG utilities (non-cryptographic)

use core::sync::atomic::{AtomicU64, Ordering};

static RNG_STATE: AtomicU64 = AtomicU64::new(0);

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
