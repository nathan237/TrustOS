//! Boot Timing Profiler — Display boot checkpoint timeline
//!
//! `hwdbg timing`       — Show boot timeline with deltas
//! `hwdbg timing slow`  — Show only checkpoints that took > 10ms

use alloc::format;
use super::dbg_out;

/// Dispatcher
pub fn run(args: &[&str]) {
    let subcmd = args.first().copied().unwrap_or("all");

    match subcmd {
        "all" | "full" => show_timeline(false),
        "slow" => show_timeline(true),
        _ => {
            dbg_out!("Usage:");
            dbg_out!("  hwdbg timing          Full boot timeline");
            dbg_out!("  hwdbg timing slow     Only slow phases (>10ms)");
        }
    }
}

fn show_timeline(slow_only: bool) {
    super::section_header("BOOT TIMING PROFILE");

    let checkpoints = crate::debug::get_checkpoints();

    if checkpoints.is_empty() {
        dbg_out!("No boot checkpoints recorded.");
        dbg_out!("Add crate::debug::checkpoint(code, \"name\") calls to init functions.");
        return;
    }

    // Get TSC frequency estimate
    let tsc_freq = estimate_tsc_freq();
    let freq_mhz = tsc_freq / 1_000_000;

    dbg_out!("TSC freq: ~{} MHz  ({} checkpoints)", freq_mhz, checkpoints.len());
    dbg_out!("");

    let first_tsc = checkpoints[0].0;
    let mut prev_tsc = first_tsc;
    let mut slowest_delta_ms: u64 = 0;
    let mut slowest_name = "";
    let mut filtered = 0u32;

    dbg_out!("{:>8} {:>8} {:>4}  {}", "Abs(ms)", "Delta", "Code", "Checkpoint");
    dbg_out!("{}", "-".repeat(60));

    for &(tsc, code, name) in &checkpoints {
        let abs_ms = tsc_to_ms(tsc - first_tsc, tsc_freq);
        let delta_ms = tsc_to_ms(tsc - prev_tsc, tsc_freq);

        if slow_only && delta_ms < 10 {
            filtered += 1;
            prev_tsc = tsc;
            continue;
        }

        let bar = make_bar(delta_ms);

        dbg_out!("{:>7}ms {:>6}ms  [{:02X}]  {} {}",
            abs_ms, delta_ms, code, name, bar);

        if delta_ms > slowest_delta_ms {
            slowest_delta_ms = delta_ms;
            slowest_name = name;
        }

        prev_tsc = tsc;
    }

    let total_ms = tsc_to_ms(checkpoints.last().map_or(0, |c| c.0) - first_tsc, tsc_freq);

    dbg_out!("{}", "-".repeat(60));
    dbg_out!("Total boot: {}ms", total_ms);
    if !slowest_name.is_empty() {
        dbg_out!("Slowest:    {} ({}ms)", slowest_name, slowest_delta_ms);
    }

    if slow_only && filtered > 0 {
        dbg_out!("({} fast checkpoints hidden)", filtered);
    }

    // Current uptime
    let uptime = crate::time::uptime_ms();
    dbg_out!("Current uptime: {}ms", uptime);
}

/// Estimate TSC frequency using PIT channel 2
/// Returns approximate Hz
fn estimate_tsc_freq() -> u64 {
    #[cfg(target_arch = "x86_64")]
    {
        // Try to use a known TSC frequency if stored
        // Otherwise, rough estimate: count TSC ticks over ~1ms using PIT
        let t1 = crate::debug::read_tsc();

        // PIT channel 2: count down from 1193 (~1ms at 1.193182 MHz)
        crate::debug::outb(0x61, (crate::debug::inb(0x61) & 0xFD) | 0x01); // Gate on
        crate::debug::outb(0x43, 0xB0); // Channel 2, mode 0, lobyte/hibyte
        crate::debug::outb(0x42, 0xA9); // Low byte (1193 & 0xFF)
        crate::debug::outb(0x42, 0x04); // High byte (1193 >> 8)

        // Wait for output to go high
        loop {
            if crate::debug::inb(0x61) & 0x20 != 0 {
                break;
            }
        }

        let t2 = crate::debug::read_tsc();
        let ticks_per_ms = t2 - t1;

        // ticks_per_ms * 1000 = Hz
        ticks_per_ms * 1000
    }

    #[cfg(not(target_arch = "x86_64"))]
    {
        // Default fallback: assume ~2 GHz
        2_000_000_000
    }
}

fn tsc_to_ms(ticks: u64, freq_hz: u64) -> u64 {
    if freq_hz == 0 { return 0; }
    (ticks * 1000) / freq_hz
}

/// Simple ASCII bar proportional to time
fn make_bar(ms: u64) -> &'static str {
    match ms {
        0 => "",
        1..=5 => ".",
        6..=20 => "..",
        21..=50 => "...",
        51..=100 => ".....",
        101..=500 => "=======",
        501..=1000 => "============",
        _ => "==================== SLOW!",
    }
}
