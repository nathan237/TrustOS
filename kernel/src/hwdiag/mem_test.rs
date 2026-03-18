//! Memory Diagnostics — walking bit test, pattern test, bad RAM detection
//!
//! Tests heap-allocated memory for reliability. Useful for detecting:
//! - Stuck bits (bits that won't flip)
//! - Address line faults (aliasing)
//! - Pattern sensitivity (data-dependent failures)
//! - Refresh failures (DRAM retention)

use alloc::string::String;
use alloc::format;
use alloc::vec::Vec;
use alloc::vec;
use super::dbg_out;

/// Run memory diagnostics
pub fn run(size_mb: usize) {
    dbg_out!("[MEM] === Memory Diagnostics ===");

    // Report memory map first
    dump_memory_layout();

    // Heap status
    dump_heap_status();

    // Run tests on allocated buffer
    let size_bytes = size_mb * 1024 * 1024;
    dbg_out!("[MEM] Allocating {} MB test buffer...", size_mb);

    // Try to allocate. If it fails, try smaller.
    let mut buf = Vec::new();
    let actual_size = allocate_test_buffer(&mut buf, size_bytes);
    if actual_size == 0 {
        dbg_out!("[MEM] ERROR: Cannot allocate test buffer! Heap exhausted?");
        return;
    }
    let actual_mb = actual_size / (1024 * 1024);
    dbg_out!("[MEM] Allocated {} MB ({} bytes) for testing", actual_mb, actual_size);

    // Test 1: Zero fill
    test_zero_fill(&mut buf);

    // Test 2: All ones
    test_all_ones(&mut buf);

    // Test 3: Walking ones
    test_walking_ones(&mut buf);

    // Test 4: Checkerboard patterns
    test_checkerboard(&mut buf);

    // Test 5: Address-as-data (detects address line faults)
    test_address_as_data(&mut buf);

    // Test 6: Random pattern (pseudo-random for reproducibility)
    test_random_pattern(&mut buf);

    dbg_out!("[MEM] === Memory test complete ===");
}

fn allocate_test_buffer(buf: &mut Vec<u8>, requested: usize) -> usize {
    let mut size = requested;
    while size >= 4096 {
        buf.clear();
        buf.try_reserve(size).ok();
        if buf.capacity() >= size {
            buf.resize(size, 0);
            return size;
        }
        size /= 2;
    }
    0
}

fn dump_memory_layout() {
    dbg_out!("[MEM] Physical Memory Map:");
    let lines = crate::debug::format_memory_map();
    for line in &lines {
        dbg_out!("[MEM]   {}", line);
    }
}

fn dump_heap_status() {
    let free = crate::memory::heap::free();
    let stats = crate::devtools::memdbg_stats();
    dbg_out!("[MEM] Heap Status:");
    dbg_out!("[MEM]   Free: {} KB ({} MB)", free / 1024, free / (1024 * 1024));
    dbg_out!("[MEM]   Live allocations: {}", stats.live_allocs);
    dbg_out!("[MEM]   Peak heap used: {} KB", stats.peak_heap_used / 1024);
    dbg_out!("[MEM]   Largest single alloc: {} KB", stats.largest_alloc / 1024);
}

fn test_zero_fill(buf: &mut [u8]) {
    dbg_out!("[MEM] Test 1/6: Zero fill...");
    let len = buf.len();

    // Write all zeros
    for b in buf.iter_mut() { *b = 0x00; }

    // Verify
    let mut errors = 0u64;
    let mut first_error_offset = 0usize;
    for (i, b) in buf.iter().enumerate() {
        if *b != 0x00 {
            if errors == 0 { first_error_offset = i; }
            errors += 1;
        }
    }
    report_test_result("Zero fill", len, errors, first_error_offset);
}

fn test_all_ones(buf: &mut [u8]) {
    dbg_out!("[MEM] Test 2/6: All ones (0xFF)...");
    let len = buf.len();

    for b in buf.iter_mut() { *b = 0xFF; }

    let mut errors = 0u64;
    let mut first_error_offset = 0usize;
    for (i, b) in buf.iter().enumerate() {
        if *b != 0xFF {
            if errors == 0 { first_error_offset = i; }
            errors += 1;
        }
    }
    report_test_result("All ones", len, errors, first_error_offset);
}

fn test_walking_ones(buf: &mut [u8]) {
    dbg_out!("[MEM] Test 3/6: Walking ones...");
    let len = buf.len();
    let mut total_errors = 0u64;

    for bit in 0..8u8 {
        let pattern = 1u8 << bit;

        for b in buf.iter_mut() { *b = pattern; }

        for (i, b) in buf.iter().enumerate() {
            if *b != pattern {
                total_errors += 1;
                if total_errors <= 3 {
                    dbg_out!("[MEM]   FAIL at offset 0x{:X}: expected 0x{:02X}, got 0x{:02X} (stuck bit?)",
                        i, pattern, *b);
                }
            }
        }
    }

    if total_errors == 0 {
        dbg_out!("[MEM]   PASS: Walking ones ({} bytes, 8 patterns)", len);
    } else {
        dbg_out!("[MEM]   FAIL: Walking ones — {} errors detected!", total_errors);
    }
}

fn test_checkerboard(buf: &mut [u8]) {
    dbg_out!("[MEM] Test 4/6: Checkerboard patterns...");
    let len = buf.len();
    let mut total_errors = 0u64;

    let patterns: [(u8, u8); 2] = [(0xAA, 0x55), (0x55, 0xAA)];
    for (p_even, p_odd) in &patterns {
        for (i, b) in buf.iter_mut().enumerate() {
            *b = if i % 2 == 0 { *p_even } else { *p_odd };
        }

        for (i, b) in buf.iter().enumerate() {
            let expected = if i % 2 == 0 { *p_even } else { *p_odd };
            if *b != expected {
                total_errors += 1;
                if total_errors <= 3 {
                    dbg_out!("[MEM]   FAIL at offset 0x{:X}: expected 0x{:02X}, got 0x{:02X}",
                        i, expected, *b);
                }
            }
        }
    }

    if total_errors == 0 {
        dbg_out!("[MEM]   PASS: Checkerboard ({} bytes, 2 patterns)", len);
    } else {
        dbg_out!("[MEM]   FAIL: Checkerboard — {} errors detected!", total_errors);
    }
}

fn test_address_as_data(buf: &mut [u8]) {
    dbg_out!("[MEM] Test 5/6: Address-as-data...");
    let len = buf.len();
    let mut errors = 0u64;

    // Write low byte of address as data (detects address line faults)
    for i in 0..len {
        buf[i] = (i & 0xFF) as u8;
    }

    for i in 0..len {
        let expected = (i & 0xFF) as u8;
        if buf[i] != expected {
            errors += 1;
            if errors <= 3 {
                dbg_out!("[MEM]   FAIL at offset 0x{:X}: expected 0x{:02X}, got 0x{:02X} (address fault?)",
                    i, expected, buf[i]);
            }
        }
    }

    if errors == 0 {
        dbg_out!("[MEM]   PASS: Address-as-data ({} bytes)", len);
    } else {
        dbg_out!("[MEM]   FAIL: Address-as-data — {} errors (possible address line fault!)", errors);
    }
}

fn test_random_pattern(buf: &mut [u8]) {
    dbg_out!("[MEM] Test 6/6: Pseudo-random pattern...");
    let len = buf.len();
    let mut errors = 0u64;

    // Simple xorshift PRNG for reproducibility
    let mut state: u64 = 0xDEAD_BEEF_CAFE_1337;

    for b in buf.iter_mut() {
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        *b = (state & 0xFF) as u8;
    }

    // Reset PRNG and verify
    state = 0xDEAD_BEEF_CAFE_1337;
    for (i, b) in buf.iter().enumerate() {
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        let expected = (state & 0xFF) as u8;
        if *b != expected {
            errors += 1;
            if errors <= 3 {
                dbg_out!("[MEM]   FAIL at offset 0x{:X}: expected 0x{:02X}, got 0x{:02X}",
                    i, expected, *b);
            }
        }
    }

    if errors == 0 {
        dbg_out!("[MEM]   PASS: Random pattern ({} bytes)", len);
    } else {
        dbg_out!("[MEM]   FAIL: Random pattern — {} errors detected!", errors);
    }
}

fn report_test_result(name: &str, size: usize, errors: u64, first_offset: usize) {
    if errors == 0 {
        dbg_out!("[MEM]   PASS: {} ({} bytes)", name, size);
    } else {
        dbg_out!("[MEM]   FAIL: {} — {} errors! First at offset 0x{:X}", name, errors, first_offset);
    }
}
