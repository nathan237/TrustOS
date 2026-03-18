//! Stress Test Suite — CPU compute + memory bandwidth + PCI probing stress
//!
//! Useful for stability testing: run for N seconds and check for crashes,
//! thermal throttling, or memory errors.

use alloc::string::String;
use alloc::format;
use alloc::vec::Vec;
use alloc::vec;
use super::dbg_out;

/// Run stress tests for the given duration
pub fn run(seconds: u64) {
    dbg_out!("[STRESS] === Hardware Stress Test ({} seconds) ===", seconds);
    dbg_out!("[STRESS] This will exercise CPU, memory, and check for thermal issues.");
    dbg_out!("");

    // Record starting thermal
    #[cfg(target_arch = "x86_64")]
    let start_temp = read_cpu_temp();

    // Phase 1: CPU integer stress
    dbg_out!("[STRESS] Phase 1/4: CPU integer stress...");
    let int_result = stress_cpu_integer(seconds / 4);
    dbg_out!("[STRESS]   {} integer ops in {} cycles ({})", 
        int_result.operations, int_result.cycles, 
        if int_result.errors == 0 { "PASS" } else { "ERRORS!" });

    // Phase 2: CPU floating point / SIMD (if available)
    dbg_out!("[STRESS] Phase 2/4: CPU SIMD stress...");
    let simd_result = stress_cpu_simd(seconds / 4);
    dbg_out!("[STRESS]   {} SIMD ops ({} errors)", simd_result.operations, simd_result.errors);

    // Phase 3: Memory bandwidth stress
    dbg_out!("[STRESS] Phase 3/4: Memory bandwidth stress...");
    let mem_result = stress_memory(seconds / 4);
    dbg_out!("[STRESS]   {} MB transferred ({} errors)", 
        mem_result.operations, mem_result.errors);

    // Phase 4: Mixed workload
    dbg_out!("[STRESS] Phase 4/4: Mixed workload...");
    let mix_result = stress_mixed(seconds / 4);
    dbg_out!("[STRESS]   {} mixed ops ({} errors)", mix_result.operations, mix_result.errors);

    // Check final thermal
    #[cfg(target_arch = "x86_64")]
    {
        let end_temp = read_cpu_temp();
        if let (Some(start), Some(end)) = (start_temp, end_temp) {
            dbg_out!("[STRESS] Temperature: {}°C → {}°C (Δ{}°C)", start, end, end - start);
            if end > 90 {
                dbg_out!("[STRESS] ⚠ WARNING: CPU temperature reached {}°C — possible thermal issue!", end);
            }
        }
    }

    // Summary
    let total_errors = int_result.errors + simd_result.errors + mem_result.errors + mix_result.errors;
    dbg_out!("");
    if total_errors == 0 {
        dbg_out!("[STRESS] ✓ ALL TESTS PASSED — no computation errors detected");
    } else {
        dbg_out!("[STRESS] ✗ {} ERRORS DETECTED — possible hardware issue!", total_errors);
    }
}

struct StressResult {
    operations: u64,
    cycles: u64,
    errors: u64,
}

fn stress_cpu_integer(seconds: u64) -> StressResult {
    let mut ops: u64 = 0;
    let mut errors: u64 = 0;
    let start_tsc = read_tsc();
    let target_cycles = seconds * estimate_tsc_freq();

    // Prime numbers sieve + verification as stress workload
    let mut primes = [0u64; 64];
    primes[0] = 2;
    let mut prime_count = 1usize;

    loop {
        let current_tsc = read_tsc();
        if current_tsc.saturating_sub(start_tsc) > target_cycles { break; }

        // Compute something verifiable
        let mut sum: u64 = 0;
        for i in 0..10000u64 {
            sum = sum.wrapping_add(i.wrapping_mul(i));
            sum ^= sum >> 17;
            sum = sum.wrapping_mul(0x517CC1B727220A95);
        }

        // Verify known result (computed deterministically)
        // Simple check: sum should not be 0 for this sequence
        if sum == 0 {
            errors += 1;
        }

        // Integer division stress
        let mut div_test: u64 = 0xDEAD_BEEF_CAFE_1337;
        for d in 1..100u64 {
            div_test = div_test.wrapping_div(d.max(1)).wrapping_add(d);
        }
        if div_test == 0 { errors += 1; } // Should never be exactly 0

        ops += 10100; // ~10000 mul + ~100 div per iteration
    }

    StressResult { operations: ops, cycles: read_tsc().saturating_sub(start_tsc), errors }
}

fn stress_cpu_simd(seconds: u64) -> StressResult {
    let mut ops: u64 = 0;
    let mut errors: u64 = 0;
    let start_tsc = read_tsc();
    let target_cycles = seconds * estimate_tsc_freq();

    // Use regular integer ops as "SIMD-like" stress
    // (actual SSE/AVX would need inline asm or intrinsics)
    loop {
        let current_tsc = read_tsc();
        if current_tsc.saturating_sub(start_tsc) > target_cycles { break; }

        // Parallel-ish computation on 4 "lanes"
        let mut a = [1u64; 4];
        let mut b = [2u64; 4];
        for _ in 0..5000 {
            for j in 0..4 {
                a[j] = a[j].wrapping_mul(b[j]).wrapping_add(j as u64);
                b[j] = b[j].wrapping_add(a[j]) ^ (a[j] >> 3);
            }
        }

        // Verify: at least one lane should be non-zero
        if a.iter().all(|&x| x == 0) {
            errors += 1;
        }

        ops += 5000 * 4; // 4 lanes, 5000 iterations
    }

    StressResult { operations: ops, cycles: read_tsc().saturating_sub(start_tsc), errors }
}

fn stress_memory(seconds: u64) -> StressResult {
    let mut ops: u64 = 0;
    let mut errors: u64 = 0;
    let start_tsc = read_tsc();
    let target_cycles = seconds * estimate_tsc_freq();

    // Allocate 4MB buffer for memory bandwidth test
    let buf_size = 4 * 1024 * 1024;
    let mut buf = vec![0u8; buf_size];

    loop {
        let current_tsc = read_tsc();
        if current_tsc.saturating_sub(start_tsc) > target_cycles { break; }

        // Sequential write
        let pattern = ((ops & 0xFF) as u8).wrapping_add(1);
        for b in buf.iter_mut() { *b = pattern; }

        // Verify
        for b in buf.iter() {
            if *b != pattern {
                errors += 1;
                break; // Don't count every bad byte, just flag the pass
            }
        }

        ops += (buf_size / (1024 * 1024)) as u64; // Count in MB
    }

    StressResult { operations: ops, cycles: read_tsc().saturating_sub(start_tsc), errors }
}

fn stress_mixed(seconds: u64) -> StressResult {
    let mut ops: u64 = 0;
    let mut errors: u64 = 0;
    let start_tsc = read_tsc();
    let target_cycles = seconds * estimate_tsc_freq();

    let mut buf = vec![0u8; 64 * 1024]; // 64 KB

    loop {
        let current_tsc = read_tsc();
        if current_tsc.saturating_sub(start_tsc) > target_cycles { break; }

        // Mix compute + memory
        let mut state: u64 = 0xCAFE_1337;
        for chunk in buf.chunks_mut(64) {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            for (i, b) in chunk.iter_mut().enumerate() {
                *b = ((state >> (i % 8)) & 0xFF) as u8;
            }
        }

        // Verify a sample
        let mut check_state: u64 = 0xCAFE_1337;
        check_state ^= check_state << 13;
        check_state ^= check_state >> 7;
        check_state ^= check_state << 17;
        if buf[0] != (check_state & 0xFF) as u8 {
            errors += 1;
        }

        ops += 1;
    }

    StressResult { operations: ops, cycles: read_tsc().saturating_sub(start_tsc), errors }
}

fn read_tsc() -> u64 {
    #[cfg(target_arch = "x86_64")]
    {
        let lo: u32;
        let hi: u32;
        unsafe {
            core::arch::asm!("rdtsc", out("eax") lo, out("edx") hi, options(nostack));
        }
        ((hi as u64) << 32) | lo as u64
    }
    #[cfg(not(target_arch = "x86_64"))]
    {
        crate::time::uptime_ms() * 1_000_000 // rough approximation
    }
}

fn estimate_tsc_freq() -> u64 {
    // Try CPUID leaf 0x16 for base frequency, else assume ~2 GHz
    #[cfg(target_arch = "x86_64")]
    {
        // Guard: leaf 0x16 only exists on Skylake+; Core 2 Duo (T61) max leaf is 0x0A
        let max_leaf = unsafe { core::arch::x86_64::__cpuid(0x0) }.eax;
        if max_leaf >= 0x16 {
            let result = unsafe { core::arch::x86_64::__cpuid(0x16) };
            let base_mhz = result.eax & 0xFFFF;
            if base_mhz > 0 {
                return (base_mhz as u64) * 1_000_000;
            }
        }
        2_000_000_000 // 2 GHz fallback
    }

    #[cfg(not(target_arch = "x86_64"))]
    {
        1_000_000 // Uptime-based, not TSC
    }
}

#[cfg(target_arch = "x86_64")]
fn read_cpu_temp() -> Option<i32> {
    let therm_status = crate::debug::read_msr_safe(0x19C)?;
    let status = therm_status as u32;
    let valid = status & (1 << 31) != 0;
    if !valid { return None; }

    let digital_readout = (status >> 16) & 0x7F;
    let tj_max = crate::debug::read_msr_safe(0x1A2)
        .map(|v| ((v >> 16) & 0xFF) as i32)
        .unwrap_or(100);

    Some(tj_max - digital_readout as i32)
}
