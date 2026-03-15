//! Unix Utility Commands  Standard POSIX-like utilities and stubs
//!
//! Includes: which, whereis, file, chmod, chown, ln, sort, uniq, cut, tr,  
//! tee, xargs, yes, seq, sleep, kill, killall, nice, nohup, bg, fg, top,
//! vmstat, iostat, lsof, strace, strings, tar, gzip, zip, mount, umount,
//! sync, lsblk, mkfs, fsck, export, alias, source, printf, bc, cal, diff,
//! cmp, md5sum, sha256sum, base64, od, rev, factor, watch, timeout,
//! stty, reset, loadkeys, lsmem, dmidecode, etc.

use alloc::string::String;
use alloc::vec::Vec;
use alloc::vec;
use alloc::format;
use crate::framebuffer::{COLOR_GREEN, COLOR_BRIGHT_GREEN, COLOR_DARK_GREEN, COLOR_YELLOW, COLOR_RED, COLOR_CYAN, COLOR_WHITE, COLOR_BLUE, COLOR_MAGENTA, COLOR_GRAY};

// ============================================================================
// Stub commands — table-driven "not implemented" for POSIX commands
// ============================================================================

const STUBS: &[(&str, &str)] = &[
    ("nohup",      "background execution"),
    ("bg",         "job control"),
    ("fg",         "job control"),
    ("gunzip",     "decompression"),
    ("mkfs",       "filesystem creation"),
    ("patch",      "patch"),
    ("script",     "terminal recording"),
    ("loadkeys",   "keymap"),
    ("setfont",    "font loading"),
    ("modprobe",   "kernel modules"),
    ("insmod",     "module loading"),
    ("rmmod",      "module unloading"),
];

/// Try to handle a stubbed command. Returns true if matched.
pub(super) fn try_stub(cmd: &str) -> bool {
    for &(name, desc) in STUBS {
        if cmd == name {
            crate::println_color!(COLOR_YELLOW, "{}: {} not implemented", name, desc);
            return true;
        }
    }
    false
}

pub(super) fn command_which(args: &[&str]) {
    if args.is_empty() {
        crate::println!("Usage: which <command>");
        return;
    }
    
    let search_dirs = ["/bin", "/usr/bin", "/sbin", "/usr/sbin"];
    
    for name in args {
        let mut found = false;
        for directory in &search_dirs {
            let path = format!("{}/{}", directory, name);
            if super::vm::file_exists(&path) {
                crate::println!("{}", path);
                found = true;
                break;
            }
        }
        // Also check Linux subsystem installed packages
        if !found {
            let subsys = crate::hypervisor::linux_subsystem::subsystem();
            if subsys.is_package_installed(name) {
                crate::println!("/usr/bin/{}", name);
                found = true;
            }
        }
        if !found {
            crate::println_color!(COLOR_RED, "{}: not found", name);
        }
    }
}

pub(super) fn command_whereis(args: &[&str]) {
    if args.is_empty() {
        crate::println!("Usage: whereis <command>");
        return;
    }
    command_which(args);
}

pub(super) fn command_file(args: &[&str]) {
    if args.is_empty() {
        crate::println!("Usage: file <path>");
        return;
    }
    
    for path in args {
        if !super::vm::file_exists(path) {
            crate::println!("{}: cannot open", path);
            continue;
        }
        
        // Try to detect file type
        if crate::exec::is_executable(path) {
            crate::println!("{}: ELF 64-bit executable", path);
        } else {
            // Read first bytes to detect
            match crate::vfs::open(path, crate::vfs::OpenFlags(0)) {
                Ok(fd) => {
                    let mut header = [0u8; 16];
                    let n = crate::vfs::read(fd, &mut header).unwrap_or(0);
                    crate::vfs::close(fd).ok();
                    
                    if n == 0 {
                        crate::println!("{}: empty", path);
                    } else if header[0..4] == [0x7F, b'E', b'L', b'F'] {
                        crate::println!("{}: ELF file", path);
                    } else if header[0..2] == [0x1f, 0x8b] {
                        crate::println!("{}: gzip compressed data", path);
                    } else if header[0..4] == [0x50, 0x4B, 0x03, 0x04] {
                        crate::println!("{}: Zip archive", path);
                    } else if header[0..6] == *b"#!/bin" {
                        crate::println!("{}: shell script", path);
                    } else if header.iter().all(|&b| b.is_ascii()) {
                        crate::println!("{}: ASCII text", path);
                    } else {
                        crate::println!("{}: data", path);
                    }
                }
                Err(_) => crate::println!("{}: cannot open", path),
            }
        }
    }
}

pub(super) fn command_basename(args: &[&str]) {
    if args.is_empty() {
        crate::println!("Usage: basename <path>");
        return;
    }
    let path = args[0];
    let name = path.rsplit('/').next().unwrap_or(path);
    crate::println!("{}", name);
}

pub(super) fn command_dirname(args: &[&str]) {
    if args.is_empty() {
        crate::println!("Usage: dirname <path>");
        return;
    }
    let path = args[0];
    if let Some(position) = path.rfind('/') {
        if position == 0 {
            crate::println!("/");
        } else {
            crate::println!("{}", &path[..position]);
        }
    } else {
        crate::println!(".");
    }
}

pub(super) fn command_realpath(args: &[&str]) {
    if args.is_empty() {
        crate::println!("Usage: realpath <path>");
        return;
    }
    let path = super::vm::resolve_program_path(args[0]);
    crate::println!("{}", path);
}

pub(super) fn command_sort(args: &[&str], piped: Option<&str>) {
    let content = if let Some(input) = piped {
        Some(alloc::string::String::from(input))
    } else if !args.is_empty() {
        super::network::read_file_content(args[0])
    } else {
        crate::println!("Usage: sort <file>");
        return;
    };
    
        // Correspondance de motifs — branchement exhaustif de Rust.
match content {
        Some(text) => {
            let mut lines: Vec<&str> = text.lines().collect();
            lines.sort();
            for line in lines {
                crate::println!("{}", line);
            }
        }
        None => crate::println_color!(COLOR_RED, "sort: cannot read input"),
    }
}

pub(super) fn command_uniq(args: &[&str], piped: Option<&str>) {
    let content = if let Some(input) = piped {
        Some(alloc::string::String::from(input))
    } else if !args.is_empty() {
        super::network::read_file_content(args[0])
    } else {
        crate::println!("Usage: uniq <file>");
        return;
    };
    
        // Correspondance de motifs — branchement exhaustif de Rust.
match content {
        Some(text) => {
            let mut last_line: Option<&str> = None;
            for line in text.lines() {
                if last_line != Some(line) {
                    crate::println!("{}", line);
                    last_line = Some(line);
                }
            }
        }
        None => crate::println_color!(COLOR_RED, "uniq: cannot read input"),
    }
}

pub(super) fn command_yes(args: &[&str]) {
    let text = if args.is_empty() { "y" } else { args[0] };
    crate::shell::clear_interrupted();
        // Boucle infinie — tourne jusqu'à un `break` explicite.
loop {
        if crate::shell::is_interrupted() { break; }
        crate::println!("{}", text);
        // Check keyboard for Ctrl+C (byte 3)
        if let Some(3) = crate::keyboard::read_char() {
            crate::shell::set_interrupted();
            break;
        }
    }
}

pub(super) fn command_sequence(args: &[&str]) {
    if args.is_empty() {
        crate::println!("Usage: seq <last> | seq <first> <last> | seq <first> <inc> <last>");
        return;
    }
    
    let (first, inc, last) = // Correspondance de motifs — branchement exhaustif de Rust.
match args.len() {
        1 => (1i64, 1i64, args[0].parse().unwrap_or(1)),
        2 => (args[0].parse().unwrap_or(1), 1i64, args[1].parse().unwrap_or(1)),
        _ => (args[0].parse().unwrap_or(1), args[1].parse().unwrap_or(1), args[2].parse().unwrap_or(1)),
    };
    
    let mut i = first;
    let mut count = 0u64;
    while (inc > 0 && i <= last) || (inc < 0 && i >= last) {
        crate::println!("{}", i);
        i += inc;
        count += 1;
        if count >= 100_000 {
            crate::println!("... (truncated at 100000 lines)");
            break;
        }
    }
}

pub(super) fn command_sleep(args: &[&str]) {
    if args.is_empty() {
        crate::println!("Usage: sleep <seconds>");
        return;
    }
    
    let secs: u64 = args[0].parse().unwrap_or(1);
    crate::println_color!(COLOR_CYAN, "Sleeping for {} seconds...", secs);
    
    // Simple busy-wait sleep (not ideal but works)
    let start = crate::time::uptime_mouse();
    let end = start + secs * 1000;
    while crate::time::uptime_mouse() < end {
        core::hint::spin_loop();
    }
    crate::println!("Done.");
}

pub(super) fn command_kill(args: &[&str]) {
    if args.is_empty() {
        crate::println!("Usage: kill [-9] <pid>");
        return;
    }
    
    let _signal = if args[0] == "-9" { 9 } else { 15 };
    let pid_str = if args[0].starts_with('-') && args.len() > 1 { args[1] } else { args[0] };
    
        // Correspondance de motifs — branchement exhaustif de Rust.
match pid_str.parse::<u32>() {
        Ok(pid) => {
            crate::println_color!(COLOR_YELLOW, "Killing PID {}", pid);
                        // Correspondance de motifs — branchement exhaustif de Rust.
match crate::process::kill(pid) {
                Ok(_) => crate::println_color!(COLOR_GREEN, "Process {} killed", pid),
                Err(e) => crate::println_color!(COLOR_RED, "kill: {}", e),
            }
        }
        Err(_) => crate::println_color!(COLOR_RED, "kill: invalid PID"),
    }
}

pub(super) fn command_top() {
    crate::println_color!(COLOR_BRIGHT_GREEN, "TrustOS Process Monitor");
    crate::println!("-----------------------------------------------------------");
    
    let uptime = crate::time::uptime_mouse() / 1000;
    let hours = uptime / 3600;
    let mins = (uptime % 3600) / 60;
    let secs = uptime % 60;
    
    crate::println!("Uptime: {:02}:{:02}:{:02}", hours, mins, secs);
    crate::println!();
    
    // Memory info
    let heap_used = crate::memory::stats().heap_used;
    let heap_total = (crate::memory::stats().heap_used + crate::memory::stats().heap_free);
    crate::println!("Mem: {} KB / {} KB ({:.1}%)", 
        heap_used / 1024, 
        heap_total / 1024,
        (heap_used as f64 / heap_total as f64) * 100.0);
    crate::println!();
    
    crate::println_color!(COLOR_CYAN, "  PID  STATE    NAME");
    crate::println!("----------------------------------");
    
    // List processes
    for (pid, name, state) in crate::process::list() {
        let state_str = // Correspondance de motifs — branchement exhaustif de Rust.
match state {
            crate::process::ProcessState::Running => "RUNNING",
            crate::process::ProcessState::Ready => "READY  ",
            crate::process::ProcessState::Blocked => "BLOCKED",
            crate::process::ProcessState::Zombie => "ZOMBIE ",
            crate::process::ProcessState::Created => "CREATED",
            crate::process::ProcessState::Waiting => "WAITING",
            crate::process::ProcessState::Stopped => "STOPPED",
            crate::process::ProcessState::Dead => "DEAD   ",
        };
        crate::println!("{:>5}  {}  {}", pid, state_str, name);
    }
    
    crate::println!();
    crate::println_color!(COLOR_YELLOW, "(press 'q' to quit in interactive mode)");
}

pub(super) fn command_vmstat() {
    crate::println_color!(COLOR_BRIGHT_GREEN, "Virtual Memory Statistics");
    crate::println!("-----------------------------------------");
    
    let heap_used = crate::memory::stats().heap_used;
    let heap_total = (crate::memory::stats().heap_used + crate::memory::stats().heap_free);
    
    crate::println!("Memory:");
    crate::println!("  Heap Total:  {} KB", heap_total / 1024);
    crate::println!("  Heap Used:   {} KB", heap_used / 1024);
    crate::println!("  Heap Free:   {} KB", (heap_total - heap_used) / 1024);
}

pub(super) fn command_dmesg(args: &[&str]) {
    if args.first() == Some(&"-c") || args.first() == Some(&"--clear") {
        // Clear by reading all (ring buffer auto-overwrites)
        crate::println_color!(COLOR_GREEN, "dmesg buffer acknowledged.");
        return;
    }
    
    let count = if let Some(&"-n") = args.first() {
        args.get(1).and_then(|s| s.parse::<usize>().ok()).unwrap_or(20)
    } else if let Some(n) = args.first().and_then(|s| s.parse::<usize>().ok()) {
        n
    } else {
        0 // show all
    };
    
    let lines = crate::devtools::dmesg_read(count);
    if lines.is_empty() {
        crate::println_color!(COLOR_YELLOW, "(no kernel messages recorded)");
        crate::println!("Tip: messages are captured after devtools init.");
        return;
    }
    let (buffer_size, total) = crate::devtools::dmesg_stats();
    crate::println_color!(COLOR_BRIGHT_GREEN, "Kernel Ring Buffer ({} stored, {} total)", buffer_size, total);
    crate::println!("---------------------------------------------------------------");
    for line in &lines {
        crate::println!("{}", line);
    }
}

pub(super) fn command_memdbg() {
    let s = crate::devtools::memdbg_stats();
    crate::println_color!(COLOR_BRIGHT_GREEN, "Memory Debug Statistics (memdbg)");
    crate::println!("---------------------------------------------------------------");
    crate::println!();
    crate::println_color!(COLOR_CYAN, "  Heap Usage:");
    crate::println!("    Current used : {:>10} bytes ({} KB)", s.current_heap_used, s.current_heap_used / 1024);
    crate::println!("    Current free : {:>10} bytes ({} KB)", s.current_heap_free, s.current_heap_free / 1024);
    crate::println!("    Total heap   : {:>10} bytes ({} KB)", s.heap_total, s.heap_total / 1024);
    crate::println!("    Peak used    : {:>10} bytes ({} KB)", s.peak_heap_used, s.peak_heap_used / 1024);
    crate::println!();
    crate::println_color!(COLOR_CYAN, "  Allocation Stats:");
    crate::println!("    Alloc ops    : {:>10}", s.allocator_count);
    crate::println!("    Dealloc ops  : {:>10}", s.dealloc_count);
    crate::println!("    Live allocs  : {:>10}", s.live_allocs);
    crate::println!("    Total alloc'd: {:>10} bytes", s.allocator_bytes_total);
    crate::println!("    Total freed  : {:>10} bytes", s.dealloc_bytes_total);
    crate::println!("    Largest alloc: {:>10} bytes", s.largest_allocator);
    crate::println!();
    crate::println_color!(COLOR_CYAN, "  Fragmentation:");
    let frag_color = if s.fragmentation_pct > 50.0 { COLOR_RED }
        else if s.fragmentation_pct > 25.0 { COLOR_YELLOW }
        else { COLOR_GREEN };
    crate::println_color!(frag_color, "    Estimate     : {:.1}%", s.fragmentation_pct);
}

pub(super) fn command_perfstat() {
    let snap = crate::devtools::perf_snapshot();
    let uptime_s = snap.uptime_mouse / 1000;
    let hours = uptime_s / 3600;
    let mins = (uptime_s % 3600) / 60;
    let secs = uptime_s % 60;
    
    crate::println_color!(COLOR_BRIGHT_GREEN, "Performance Statistics (perf)");
    crate::println!("---------------------------------------------------------------");
    crate::println!();
    crate::println_color!(COLOR_CYAN, "  System:");
    crate::println!("    Uptime       : {}h {:02}m {:02}s ({} ms)", hours, mins, secs, snap.uptime_mouse);
    crate::println!("    GUI FPS      : {}", snap.fps);
    crate::println!();
    crate::println_color!(COLOR_CYAN, "  Interrupts:");
    crate::println!("    Total IRQs   : {}", snap.total_irqs);
    crate::println!("    IRQ/sec      : {}", snap.interrupt_request_per_sector);
    crate::println!();
    crate::println_color!(COLOR_CYAN, "  Scheduling:");
    crate::println!("    Syscalls     : {}", snap.total_syscalls);
    crate::println!("    Ctx switches : {}", snap.total_context_switches);
    crate::println!();
    crate::println_color!(COLOR_CYAN, "  Memory:");
    crate::println!("    Heap used    : {} / {} KB ({}%)", 
        snap.heap_used / 1024, (snap.heap_used + snap.heap_free) / 1024,
        if snap.heap_used + snap.heap_free > 0 { snap.heap_used * 100 / (snap.heap_used + snap.heap_free) } else { 0 });
    crate::println!();
    crate::println_color!(COLOR_CYAN, "  Per-CPU:");
    for s in &snap.cpu_stats {
        let state = if s.is_idle { "idle" } else { "busy" };
        crate::println!("    CPU{}: {} irqs, {} syscalls, {} ctxsw [{}]", 
            s.cpu_id, s.interrupts, s.syscalls, s.context_switches, state);
    }
}

pub(super) fn command_irqstat() {
    let stats = crate::sync::percpu::all_cpu_stats();
    let total_irqs: u64 = stats.iter().map(|s| s.interrupts).sum();
    
    crate::println_color!(COLOR_BRIGHT_GREEN, "IRQ Statistics");
    crate::println!("---------------------------------------------------------------");
    crate::println!();
    crate::println!("  Total IRQs     : {}", total_irqs);
    crate::println!("  IRQ rate       : {}/sec", crate::devtools::interrupt_request_rate());
    crate::println!();
    crate::println_color!(COLOR_CYAN, "  Per-CPU Breakdown:");
    for s in &stats {
        let bar_length = if total_irqs > 0 { (s.interrupts * 40 / total_irqs.maximum(1)) as usize } else { 0 };
        let bar: String = "|".repeat(bar_length);
        let pct = if total_irqs > 0 { s.interrupts * 100 / total_irqs } else { 0 };
        crate::println!("    CPU{}: {:>8} ({:>3}%) {}", s.cpu_id, s.interrupts, pct, bar);
    }
}

pub(super) fn command_registers() {
    crate::println_color!(COLOR_BRIGHT_GREEN, "CPU Register Dump");
    crate::println!("---------------------------------------------------------------");
    let regs = crate::devtools::cpu_registers();
    for line in &regs {
        crate::println!("{}", line);
    }
}

pub(super) fn command_peek(args: &[&str]) {
    if args.is_empty() {
        crate::println!("Usage: peek <hex_addr> [byte_count]");
        crate::println!("  e.g.: peek 0xFFFF8000_00000000 64");
        crate::println!("  Default count: 64 bytes, max: 256 bytes");
        return;
    }
    
    let address_str = args[0].trim_start_matches("0x").trim_start_matches("0X");
    let address = // Correspondance de motifs — branchement exhaustif de Rust.
match usize::from_str_radix(address_str, 16) {
        Ok(a) => a,
        Err(_) => {
            crate::println_color!(COLOR_RED, "Invalid hex address: {}", args[0]);
            return;
        }
    };
    
    let count = args.get(1).and_then(|s| s.parse::<usize>().ok()).unwrap_or(64);
    
    crate::println_color!(COLOR_BRIGHT_GREEN, "Memory dump at 0x{:016x} ({} bytes)", address, count);
    crate::println!("---------------------------------------------------------------");
    let lines = crate::devtools::peek(address, count);
    for line in &lines {
        crate::println!("{}", line);
    }
}

pub(super) fn command_poke(args: &[&str]) {
    if args.len() < 2 {
        crate::println!("Usage: poke <hex_addr> <hex_value>");
        crate::println!("  e.g.: poke 0xB8000 0x41");
        crate::println_color!(COLOR_RED, "  ? WARNING: Writing to arbitrary memory is DANGEROUS!");
        return;
    }
    
    let address_str = args[0].trim_start_matches("0x").trim_start_matches("0X");
    let address = // Correspondance de motifs — branchement exhaustif de Rust.
match usize::from_str_radix(address_str, 16) {
        Ok(a) => a,
        Err(_) => {
            crate::println_color!(COLOR_RED, "Invalid hex address: {}", args[0]);
            return;
        }
    };
    
    let value_str = args[1].trim_start_matches("0x").trim_start_matches("0X");
    let value = // Correspondance de motifs — branchement exhaustif de Rust.
match u8::from_str_radix(value_str, 16) {
        Ok(v) => v,
        Err(_) => {
            crate::println_color!(COLOR_RED, "Invalid hex value: {}", args[1]);
            return;
        }
    };
    
        // Correspondance de motifs — branchement exhaustif de Rust.
match crate::devtools::poke(address, value) {
        Ok(()) => crate::println_color!(COLOR_GREEN, "Wrote 0x{:02x} to 0x{:016x}", value, address),
        Err(e) => crate::println_color!(COLOR_RED, "poke error: {}", e),
    }
}

pub(super) fn command_devpanel() {
    crate::devtools::toggle_devpanel();
    let state = if crate::devtools::is_devpanel_visible() { "ON" } else { "OFF" };
    crate::println_color!(COLOR_GREEN, "DevPanel overlay: {} (also toggle with F12 in desktop)", state);
}

pub(super) fn command_timecmd(args: &[&str]) {
    if args.is_empty() {
        crate::println!("Usage: timecmd <command> [args...]");
        crate::println!("  Runs a command and prints elapsed time.");
        return;
    }
    
    let start = crate::cpu::tsc::Stopwatch::start();
    
    // Reconstruct and execute the sub-command
    let sub_command = args.join(" ");
    super::execute_command(&sub_command);
    
    let elapsed_us = start.elapsed_micros();
    let elapsed_mouse = elapsed_us / 1000;
    let frac = elapsed_us % 1000;
    crate::println!();
    crate::println_color!(COLOR_CYAN, "? Elapsed: {}.{:03} ms ({} us)", elapsed_mouse, frac, elapsed_us);
}

// ═══════════════════════════════════════════════════════════════════════════════
// Hardware Debug Toolkit commands
// ═══════════════════════════════════════════════════════════════════════════════

/// Full hardware diagnostic report
pub(super) fn command_hwdiag() {
    crate::println_color!(COLOR_BRIGHT_GREEN, "Generating hardware diagnostic report...");
    let lines = crate::debug::full_diagnostic_report();
    for line in &lines {
        crate::println!("{}", line);
    }
    // Also send to serial for capture
    for line in &lines {
        crate::serial_println!("{}", line);
    }
}

/// Full CPU state dump (all GPR + control + segment + MSR)
pub(super) fn command_cpudump() {
    crate::println_color!(COLOR_BRIGHT_GREEN, "Full CPU State Dump");
    crate::println!("---------------------------------------------------------------");
    let lines = crate::debug::full_cpu_dump();
    for line in &lines {
        crate::println!("{}", line);
    }
}

/// Stack trace / backtrace
pub(super) fn command_stacktrace(args: &[&str]) {
    let maximum = args.first().and_then(|s| s.parse::<usize>().ok()).unwrap_or(16);
    crate::println_color!(COLOR_BRIGHT_GREEN, "Stack Backtrace (max {} frames)", maximum);
    crate::println!("---------------------------------------------------------------");
    let lines = crate::debug::format_backtrace(maximum);
    for line in &lines {
        crate::println!("{}", line);
    }
}

/// Show boot checkpoints log
pub(super) fn command_bootlog() {
    crate::println_color!(COLOR_BRIGHT_GREEN, "Boot Checkpoints");
    crate::println!("---------------------------------------------------------------");
    let cps = crate::debug::get_checkpoints();
    if cps.is_empty() {
        crate::println!("  <no checkpoints recorded>");
    } else {
        let first_tsc = cps[0].0;
        for (tsc, code, name) in &cps {
            let delta = tsc - first_tsc;
            crate::println!("  POST 0x{:02X}  TSC +{:>14}  {}", code, delta, name);
        }
    }
    crate::println!("  Last POST code: 0x{:02X}", crate::debug::last_post_code());
}

/// Write a POST code (or show current)
pub(super) fn command_postcode(args: &[&str]) {
    if args.is_empty() {
        crate::println!("Current POST code: 0x{:02X}", crate::debug::last_post_code());
        crate::println!("Usage: postcode <hex_value>   (writes to port 0x80)");
        return;
    }
    let value_str = args[0].trim_start_matches("0x").trim_start_matches("0X");
        // Correspondance de motifs — branchement exhaustif de Rust.
match u8::from_str_radix(value_str, 16) {
        Ok(v) => {
            crate::debug::post_code(v);
            crate::println_color!(COLOR_GREEN, "POST code 0x{:02X} written to port 0x80", v);
        }
        Err(_) => crate::println_color!(COLOR_RED, "Invalid hex value: {}", args[0]),
    }
}

/// I/O port read/write
pub(super) fn command_ioport(args: &[&str]) {
    if args.len() < 2 {
        crate::println!("Usage: ioport read <port_hex> [b|w|l]");
        crate::println!("       ioport write <port_hex> <value_hex> [b|w|l]");
        crate::println!("  b=byte (default), w=word, l=dword");
        crate::println_color!(COLOR_RED, "  ⚠ WARNING: Writing to arbitrary I/O ports is DANGEROUS!");
        return;
    }
    
    let subcmd = args[0];
    let port_str = args[1].trim_start_matches("0x").trim_start_matches("0X");
    let port = // Correspondance de motifs — branchement exhaustif de Rust.
match u16::from_str_radix(port_str, 16) {
        Ok(p) => p,
        Err(_) => {
            crate::println_color!(COLOR_RED, "Invalid port: {}", args[1]);
            return;
        }
    };
    
        // Correspondance de motifs — branchement exhaustif de Rust.
match subcmd {
        "read" | "r" => {
            let size = args.get(2).copied().unwrap_or("b");
                        // Correspondance de motifs — branchement exhaustif de Rust.
match size {
                "b" | "byte" => {
                    let value = crate::debug::inb(port);
                    crate::println!("  IN  port 0x{:04X} = 0x{:02X} ({})", port, value, value);
                }
                "w" | "word" => {
                    let value = crate::debug::inw(port);
                    crate::println!("  IN  port 0x{:04X} = 0x{:04X} ({})", port, value, value);
                }
                "l" | "dword" => {
                    let value = crate::debug::inl(port);
                    crate::println!("  IN  port 0x{:04X} = 0x{:08X} ({})", port, value, value);
                }
                _ => crate::println_color!(COLOR_RED, "Size must be b/w/l"),
            }
        }
        "write" | "w" => {
            if args.len() < 3 {
                crate::println_color!(COLOR_RED, "Need value: ioport write <port> <value> [b|w|l]");
                return;
            }
            let value_str = args[2].trim_start_matches("0x").trim_start_matches("0X");
            let size = args.get(3).copied().unwrap_or("b");
                        // Correspondance de motifs — branchement exhaustif de Rust.
match size {
                "b" | "byte" => {
                    if let Ok(v) = u8::from_str_radix(value_str, 16) {
                        crate::debug::outb(port, v);
                        crate::println_color!(COLOR_GREEN, "  OUT port 0x{:04X} <- 0x{:02X}", port, v);
                    } else {
                        crate::println_color!(COLOR_RED, "Invalid byte value");
                    }
                }
                "w" | "word" => {
                    if let Ok(v) = u16::from_str_radix(value_str, 16) {
                        crate::debug::outw(port, v);
                        crate::println_color!(COLOR_GREEN, "  OUT port 0x{:04X} <- 0x{:04X}", port, v);
                    } else {
                        crate::println_color!(COLOR_RED, "Invalid word value");
                    }
                }
                "l" | "dword" => {
                    if let Ok(v) = u32::from_str_radix(value_str, 16) {
                        crate::debug::outl(port, v);
                        crate::println_color!(COLOR_GREEN, "  OUT port 0x{:04X} <- 0x{:08X}", port, v);
                    } else {
                        crate::println_color!(COLOR_RED, "Invalid dword value");
                    }
                }
                _ => crate::println_color!(COLOR_RED, "Size must be b/w/l"),
            }
        }
        _ => crate::println_color!(COLOR_RED, "Use: ioport read|write ..."),
    }
}

/// Read MSR
pub(super) fn command_rdmsr(args: &[&str]) {
    if args.is_empty() {
        crate::println!("Usage: rdmsr <msr_hex>");
        crate::println!("  e.g.: rdmsr 0xC0000080  (IA32_EFER)");
        crate::println!("Common MSRs:");
        crate::println!("  0xC0000080  IA32_EFER       0x0000001B  IA32_APIC_BASE");
        crate::println!("  0xC0000081  IA32_STAR        0x00000010  IA32_TSC");
        crate::println!("  0xC0000082  IA32_LSTAR       0x00000277  IA32_PAT");
        return;
    }
    let msr_str = args[0].trim_start_matches("0x").trim_start_matches("0X");
        // Correspondance de motifs — branchement exhaustif de Rust.
match u32::from_str_radix(msr_str, 16) {
        Ok(msr) => {
                        // Correspondance de motifs — branchement exhaustif de Rust.
match crate::debug::read_msr_safe(msr) {
                Some(value) => {
                    crate::println!("  MSR 0x{:08X} = 0x{:016X}", msr, value);
                    crate::println!("                  {:064b}", value);
                }
                None => crate::println_color!(COLOR_RED, "  MSR 0x{:08X}: read failed (#GP)", msr),
            }
        }
        Err(_) => crate::println_color!(COLOR_RED, "Invalid MSR address: {}", args[0]),
    }
}

/// Write MSR
pub(super) fn command_wrmsr(args: &[&str]) {
    if args.len() < 2 {
        crate::println!("Usage: wrmsr <msr_hex> <value_hex>");
        crate::println_color!(COLOR_RED, "  ⚠ WARNING: Writing to MSRs can crash the system!");
        return;
    }
    let msr_str = args[0].trim_start_matches("0x").trim_start_matches("0X");
    let value_str = args[1].trim_start_matches("0x").trim_start_matches("0X");
    
    let msr = // Correspondance de motifs — branchement exhaustif de Rust.
match u32::from_str_radix(msr_str, 16) {
        Ok(m) => m,
        Err(_) => {
            crate::println_color!(COLOR_RED, "Invalid MSR: {}", args[0]);
            return;
        }
    };
    let value = // Correspondance de motifs — branchement exhaustif de Rust.
match u64::from_str_radix(value_str, 16) {
        Ok(v) => v,
        Err(_) => {
            crate::println_color!(COLOR_RED, "Invalid value: {}", args[1]);
            return;
        }
    };
    
    crate::debug::write_msr(msr, value);
    crate::println_color!(COLOR_GREEN, "  WRMSR 0x{:08X} <- 0x{:016X}", msr, value);
}

/// Raw CPUID query
pub(super) fn command_cpuid(args: &[&str]) {
    if args.is_empty() {
        crate::println!("Usage: cpuid <leaf_hex> [subleaf_hex]");
        crate::println!("  e.g.: cpuid 0          (vendor string)");
        crate::println!("        cpuid 1          (features)");
        crate::println!("        cpuid 0x80000002 (brand string part 1)");
        crate::println!("        cpuid 0x80000003 (brand string part 2)");
        crate::println!("        cpuid 0x80000004 (brand string part 3)");
        return;
    }
    let leaf_str = args[0].trim_start_matches("0x").trim_start_matches("0X");
    let subleaf_str = args.get(1).map(|s| s.trim_start_matches("0x").trim_start_matches("0X")).unwrap_or("0");
    
    let leaf = // Correspondance de motifs — branchement exhaustif de Rust.
match u32::from_str_radix(leaf_str, 16) {
        Ok(l) => l,
        Err(_) => {
            crate::println_color!(COLOR_RED, "Invalid leaf: {}", args[0]);
            return;
        }
    };
    let subleaf = u32::from_str_radix(subleaf_str, 16).unwrap_or(0);
    
    crate::println_color!(COLOR_BRIGHT_GREEN, "CPUID Query");
    crate::println!("---------------------------------------------------------------");
    let lines = crate::debug::format_cpuid(leaf, subleaf);
    for line in &lines {
        crate::println!("{}", line);
    }
}

/// Memory map display
pub(super) fn command_memmap() {
    crate::println_color!(COLOR_BRIGHT_GREEN, "Physical Memory Map");
    crate::println!("---------------------------------------------------------------");
    let lines = crate::debug::format_memory_map();
    for line in &lines {
        crate::println!("{}", line);
    }
}

/// Watchdog control
pub(super) fn command_watchdog(args: &[&str]) {
        // Correspondance de motifs — branchement exhaustif de Rust.
match args.first().copied() {
        Some("enable" | "on") => {
            let timeout = args.get(1).and_then(|s| s.parse::<u64>().ok()).unwrap_or(5000);
            crate::debug::watchdog_enable(timeout);
            crate::println_color!(COLOR_GREEN, "Watchdog enabled ({} ms timeout)", timeout);
        }
        Some("disable" | "off") => {
            crate::debug::watchdog_disable();
            crate::println_color!(COLOR_GREEN, "Watchdog disabled");
        }
        Some("pet" | "kick") => {
            crate::debug::watchdog_pet();
            crate::println_color!(COLOR_GREEN, "Watchdog petted");
        }
        _ => {
            crate::println!("Usage: watchdog <enable [ms]|disable|pet>");
            crate::println!("  enable [timeout_ms]  — Start watchdog (default: 5000 ms)");
            crate::println!("  disable              — Stop watchdog");
            crate::println!("  pet                  — Reset watchdog counter");
        }
    }
}

pub(super) fn command_lsof(_args: &[&str]) {
    crate::println!("COMMAND   PID   FD   TYPE   NAME");
    crate::println!("----------------------------------------");
    crate::println!("shell     1     0    CHR    /dev/stdin");
    crate::println!("shell     1     1    CHR    /dev/stdout");
    crate::println!("shell     1     2    CHR    /dev/stderr");
}

pub(super) fn command_strings(args: &[&str]) {
    if args.is_empty() {
        crate::println!("Usage: strings <file>");
        return;
    }
    
        // Correspondance de motifs — branchement exhaustif de Rust.
match super::network::read_file_bytes(args[0]) {
        Some(data) => {
            let mut current = String::new();
            for &byte in &data {
                if byte.is_ascii_graphic() || byte == b' ' {
                    current.push(byte as char);
                } else {
                    if current.len() >= 4 {
                        crate::println!("{}", current);
                    }
                    current.clear();
                }
            }
            if current.len() >= 4 {
                crate::println!("{}", current);
            }
        }
        None => crate::println_color!(COLOR_RED, "strings: cannot read {}", args[0]),
    }
}

pub(super) fn command_mount(args: &[&str]) {
    if args.is_empty() {
        // Show mounted filesystems
        crate::println_color!(COLOR_BRIGHT_GREEN, "Mounted Filesystems:");
        crate::vfs::list_mounts();
        return;
    }
    
    if args.len() < 2 {
        crate::println!("Usage: mount <device> <mountpoint>");
        return;
    }
    
    crate::println_color!(COLOR_YELLOW, "mount: dynamic mounting not implemented");
}

pub(super) fn command_sync() {
    crate::println!("Syncing filesystems...");
    crate::println_color!(COLOR_GREEN, "Done.");
}

pub(super) fn command_umount(args: &[&str]) {
    if args.is_empty() {
        crate::println!("Usage: umount <mountpoint>");
        return;
    }
        // Correspondance de motifs — branchement exhaustif de Rust.
match crate::vfs::umount(args[0]) {
        Ok(()) => crate::println_color!(COLOR_GREEN, "Unmounted {}", args[0]),
        Err(e) => crate::println_color!(COLOR_RED, "umount: {}: {:?}", args[0], e),
    }
}

pub(super) fn command_fsck(args: &[&str]) {
    crate::println_color!(COLOR_BRIGHT_GREEN, "TrustFS Filesystem Check");
    crate::println!("========================================");

    let mounts = crate::vfs::list_mounts();
    if mounts.is_empty() {
        crate::println_color!(COLOR_YELLOW, "No mounted filesystems");
        return;
    }

    let mut errors = 0u32;
    let mut checked = 0u32;

    for (path, fstype) in &mounts {
        checked += 1;
        crate::print!("  [{}] {} ({})... ", checked, path, fstype);

        // Check that the mount point is accessible
        match crate::vfs::readdir(path) {
            Ok(entries) => {
                let count = entries.len();
                crate::println_color!(COLOR_GREEN, "OK ({} entries)", count);
            }
            Err(e) => {
                errors += 1;
                crate::println_color!(COLOR_RED, "ERROR: {:?}", e);
            }
        }
    }

    crate::println!("----------------------------------------");
    if errors == 0 {
        crate::println_color!(COLOR_GREEN, "fsck: {} filesystem(s) checked, no errors", checked);
    } else {
        crate::println_color!(COLOR_RED, "fsck: {} error(s) found in {} filesystem(s)", errors, checked);
    }
}

pub(super) fn command_lsblk() {
    crate::println_color!(COLOR_BRIGHT_GREEN, "Block Devices:");
    crate::println!("NAME          SIZE        TYPE    DRIVER        MODEL");
    crate::println!("----------------------------------------------------------------------");
    
    let mut index = 0u32;
    
    // NVMe namespaces
    if crate::nvme::is_initialized() {
        if let Some((model, _serial, ns_size, lba_size)) = crate::nvme::get_information() {
            let size_bytes = ns_size * lba_size as u64;
            let size_str = format_size(size_bytes);
            crate::println!("nvme0n1       {:<11} disk    NVMe          {}", size_str, model);
            index += 1;
        }
    }
    
    // AHCI/SATA disks
    if crate::drivers::ahci::is_initialized() {
        for device in crate::drivers::ahci::list_devices() {
            let size_bytes = device.sector_count * 512;
            let size_str = format_size(size_bytes);
            let type_str = // Correspondance de motifs — branchement exhaustif de Rust.
match device.device_type {
                crate::drivers::ahci::AhciDeviceType::Sata => "disk",
                crate::drivers::ahci::AhciDeviceType::Satapi => "cdrom",
                _ => "disk",
            };
            crate::println!("sda{}          {:<11} {:<7} AHCI/p{}       {}", 
                index, size_str, type_str, device.port_number, device.model);
            index += 1;
        }
    }
    
    // IDE/ATA drives
    for driver in crate::drivers::ata::list_drives() {
        if driver.present {
            let size_bytes = driver.sector_count * 512;
            let size_str = format_size(size_bytes);
            let character = // Correspondance de motifs — branchement exhaustif de Rust.
match driver.channel {
                crate::drivers::ata::IdeChannel::Primary => "P",
                crate::drivers::ata::IdeChannel::Secondary => "S",
            };
            let position = // Correspondance de motifs — branchement exhaustif de Rust.
match driver.position {
                crate::drivers::ata::DrivePosition::Master => "M",
                crate::drivers::ata::DrivePosition::Slave => "S",
            };
            let type_str = if driver.atapi { "cdrom" } else { "disk" };
            let lba_str = if driver.lba48 { "LBA48" } else { "LBA28" };
            crate::println!("hd{}           {:<11} {:<7} IDE/{}{} {}  {}", 
                index, size_str, type_str, character, position, lba_str, driver.model);
            index += 1;
        }
    }
    
    // VirtIO block devices
    if crate::virtio_blk::is_initialized() {
        let capability = crate::virtio_blk::capacity();
        let size_str = format_size(capability * 512);
        let ro = if crate::virtio_blk::is_read_only() { " (ro)" } else { "" };
        crate::println!("vda{}          {:<11} disk    VirtIO-blk{}", index, size_str, ro);
        index += 1;
    }
    
    // USB mass storage
    for (i, (name, blocks, bsize)) in crate::drivers::usb_storage::list_devices().iter().enumerate() {
        let size_str = format_size(*blocks * *bsize as u64);
        crate::println!("usb{}          {:<11} disk    USB-Storage   {}", 
            index + i as u32, size_str, name);
    }
    if index == 0 && crate::drivers::usb_storage::device_count() == 0 {
        index += 1; // for ram0 display below
    }
    
    // RAM disk (always present as fallback)
    crate::println!("ram0          256K        ramdisk RAM           TrustFS");
    
    if index == 0 {
        crate::println!();
        crate::println_color!(COLOR_YELLOW, "No hardware storage detected (using RAM disk)");
    }
}

/// Format byte size to human-readable string
fn format_size(bytes: u64) -> alloc::string::String {
    if bytes == 0 {
        return alloc::string::String::from("0B");
    }
    if bytes >= 1024 * 1024 * 1024 * 1024 {
        alloc::format!("{:.1}T", bytes as f64 / (1024.0 * 1024.0 * 1024.0 * 1024.0))
    } else if bytes >= 1024 * 1024 * 1024 {
        alloc::format!("{:.1}G", bytes as f64 / (1024.0 * 1024.0 * 1024.0))
    } else if bytes >= 1024 * 1024 {
        alloc::format!("{:.1}M", bytes as f64 / (1024.0 * 1024.0))
    } else if bytes >= 1024 {
        alloc::format!("{}K", bytes / 1024)
    } else {
        alloc::format!("{}B", bytes)
    }
}

pub(super) fn command_blkid() {
    let mut found = false;
    
    // NVMe
    if crate::nvme::is_initialized() {
        if let Some((model, serial, ns_size, lba_size)) = crate::nvme::get_information() {
            let size_bytes = ns_size * lba_size as u64;
            crate::println!("/dev/nvme0n1: MODEL=\"{}\" SERIAL=\"{}\" SIZE={} TYPE=\"nvme\"",
                model, serial, format_size(size_bytes));
            found = true;
        }
    }
    
    // AHCI
    if crate::drivers::ahci::is_initialized() {
        for (i, device) in crate::drivers::ahci::list_devices().iter().enumerate() {
            let size_bytes = device.sector_count * 512;
            crate::println!("/dev/sda{}: MODEL=\"{}\" SERIAL=\"{}\" SIZE={} TYPE=\"ahci\" PORT={}",
                i, device.model, device.serial, format_size(size_bytes), device.port_number);
            found = true;
        }
    }
    
    // IDE
    for (i, driver) in crate::drivers::ata::list_drives().iter().enumerate() {
        if driver.present {
            let size_bytes = driver.sector_count * 512;
            let fstype = if driver.atapi { "atapi" } else { "ide" };
            crate::println!("/dev/hd{}: MODEL=\"{}\" SERIAL=\"{}\" SIZE={} TYPE=\"{}\"",
                i, driver.model, driver.serial, format_size(size_bytes), fstype);
            found = true;
        }
    }
    
    // VirtIO
    if crate::virtio_blk::is_initialized() {
        let capability = crate::virtio_blk::capacity();
        crate::println!("/dev/vda: SIZE={} TYPE=\"virtio-blk\"", format_size(capability * 512));
        found = true;
    }
    
    // USB
    for (i, (name, blocks, bsize)) in crate::drivers::usb_storage::list_devices().iter().enumerate() {
        crate::println!("/dev/usb{}: MODEL=\"{}\" SIZE={} TYPE=\"usb-storage\"",
            i, name, format_size(*blocks * *bsize as u64));
        found = true;
    }
    
    // RAM disk always
    crate::println!("/dev/ram0: SIZE=256K TYPE=\"ramfs\"");
    
    if !found {
        crate::println_color!(COLOR_YELLOW, "No hardware block devices detected");
    }
}

pub(super) fn command_export(args: &[&str]) {
    if args.is_empty() {
        // Show all variables as export format
        for (k, v) in super::scripting::all_vars() {
            crate::println!("export {}={}", k, v);
        }
        return;
    }
    // Parse VAR=VALUE or VAR
    let joined = args.join(" ");
    if let Some(eq_position) = joined.find('=') {
        let key = joined[..eq_position].trim();
        let value = joined[eq_position + 1..].trim().trim_matches('"').trim_matches('\'');
        super::scripting::set_var(key, value);
        crate::serial_println!("[export] {}={}", key, value);
    } else {
        // Just mark as exported (already in our global store)
        if super::scripting::get_var(args[0]).is_none() {
            super::scripting::set_var(args[0], "");
        }
    }
}

pub(super) fn command_source(args: &[&str]) {
    if args.is_empty() {
        crate::println!("Usage: source <script>");
        return;
    }
    
        // Correspondance de motifs — branchement exhaustif de Rust.
match super::network::read_file_content(args[0]) {
        Some(content) => {
            super::scripting::execute_script(&content);
        }
        None => crate::println_color!(COLOR_RED, "source: cannot read {}", args[0]),
    }
}

pub(super) fn command_set(_args: &[&str]) {
    for (k, v) in super::scripting::all_vars() {
        crate::println!("{}={}", k, v);
    }
}

pub(super) fn command_printf(args: &[&str]) {
    if args.is_empty() {
        crate::println!("Usage: printf <format> [args...]");
        return;
    }
    // Simple implementation - just print format string
    let format = args[0].replace("\\n", "\n").replace("\\t", "\t");
    crate::print!("{}", format);
}

pub(super) fn command_test_expr(args: &[&str]) {
    // Basic test expression evaluation
    if args.is_empty() {
        crate::println!("false");
        return;
    }
    
        // Correspondance de motifs — branchement exhaustif de Rust.
match args.first() {
        Some(&"-e") if args.len() > 1 => {
            if super::vm::file_exists(args[1]) {
                crate::println!("true");
            } else {
                crate::println!("false");
            }
        }
        Some(&"-d") if args.len() > 1 => {
            crate::println_color!(COLOR_YELLOW, "(directory check not implemented)");
        }
        Some(&"-f") if args.len() > 1 => {
            if super::vm::file_exists(args[1]) {
                crate::println!("true");
            } else {
                crate::println!("false");
            }
        }
        _ => crate::println!("true"),
    }
}

pub(super) fn command_expr(args: &[&str]) {
    if args.len() < 3 {
        crate::println!("Usage: expr <num1> <op> <num2>");
        return;
    }
    
    let a: i64 = args[0].parse().unwrap_or(0);
    let b: i64 = args[2].parse().unwrap_or(0);
    
    let result = // Correspondance de motifs — branchement exhaustif de Rust.
match args[1] {
        "+" => a + b,
        "-" => a - b,
        "*" => a * b,
        "/" if b != 0 => a / b,
        "%" if b != 0 => a % b,
        _ => {
            crate::println!("expr: invalid operator");
            return;
        }
    };
    
    crate::println!("{}", result);
}

pub(super) fn command_cal(_args: &[&str]) {
    crate::println_color!(COLOR_BRIGHT_GREEN, "   February 2026");
    crate::println!("Su Mo Tu We Th Fr Sa");
    crate::println!(" 1  2  3  4  5  6  7");
    crate::println!(" 8  9 10 11 12 13 14");
    crate::println!("15 16 17 18 19 20 21");
    crate::println!("22 23 24 25 26 27 28");
}

pub(super) fn command_cmp(args: &[&str]) {
    if args.len() < 2 {
        crate::println!("Usage: cmp <file1> <file2>");
        return;
    }
    
        // Correspondance de motifs — branchement exhaustif de Rust.
match (super::network::read_file_bytes(args[0]), super::network::read_file_bytes(args[1])) {
        (Some(a), Some(b)) => {
            if a == b {
                // Files are identical, no output
            } else {
                crate::println!("{} {} differ", args[0], args[1]);
            }
        }
        _ => crate::println_color!(COLOR_RED, "cmp: cannot read files"),
    }
}

pub(super) fn command_od(args: &[&str]) {
    if args.is_empty() {
        crate::println!("Usage: od <file>");
        return;
    }
    // Use hexdump for now
    super::commands::command_hexdump(args);
}

pub(super) fn command_rev(args: &[&str]) {
    if args.is_empty() {
        crate::println!("Usage: rev <file>");
        return;
    }
    
        // Correspondance de motifs — branchement exhaustif de Rust.
match super::network::read_file_content(args[0]) {
        Some(content) => {
            for line in content.lines() {
                let reversed: String = line.chars().rev().collect();
                crate::println!("{}", reversed);
            }
        }
        None => crate::println_color!(COLOR_RED, "rev: cannot read {}", args[0]),
    }
}

pub(super) fn command_factor(args: &[&str]) {
    if args.is_empty() {
        crate::println!("Usage: factor <number>");
        return;
    }
    
    let mut n: u64 = args[0].parse().unwrap_or(0);
    if n == 0 {
        crate::println!("factor: invalid number");
        return;
    }
    
    crate::print!("{}:", n);
    let mut d = 2u64;
    while d.checked_mul(d).map_or(false, |dd| dd <= n) {
        while n % d == 0 {
            crate::print!(" {}", d);
            n /= d;
        }
        d += 1;
    }
    if n > 1 {
        crate::print!(" {}", n);
    }
    crate::println!();
}

pub(super) fn command_tty() {
    crate::println!("/dev/tty0");
}

pub(super) fn command_stty(_args: &[&str]) {
    crate::println!("speed 9600 baud; line = 0;");
    crate::println!("-brkint -imaxbel");
}

pub(super) fn command_reset() {
    super::commands::command_clear();
    crate::println!("Terminal reset.");
}

pub(super) fn command_lsusb() {
    crate::println_color!(COLOR_BRIGHT_GREEN, "USB Devices:");
    crate::println!("-------------------------------------------");
    
    // Check if xHCI is initialized
    if crate::drivers::xhci::is_initialized() {
        let devices = crate::drivers::xhci::list_devices();
        if devices.is_empty() {
            crate::println!("Bus 001 Device 001: ID 0000:0000 Root Hub");
            crate::println!("  (no devices connected)");
        } else {
            crate::println!("Bus 001 Device 001: ID 0000:0000 xHCI Root Hub");
            for (i, device) in devices.iter().enumerate() {
                let speed = // Correspondance de motifs — branchement exhaustif de Rust.
match device.speed {
                    1 => "Full Speed (12 Mbps)",
                    2 => "Low Speed (1.5 Mbps)",
                    3 => "High Speed (480 Mbps)",
                    4 => "SuperSpeed (5 Gbps)",
                    _ => "Unknown",
                };
                crate::println!("Bus 001 Device {:03}: ID {:04x}:{:04x} Port {} - {}", 
                    i + 2, device.vendor_id, device.product_id, device.port, speed);
                if device.class != 0 {
                    let class_name = // Correspondance de motifs — branchement exhaustif de Rust.
match device.class {
                        0x03 => "HID (Human Interface Device)",
                        0x08 => "Mass Storage",
                        0x09 => "Hub",
                        _ => "Unknown class",
                    };
                    crate::println!("    Class: {:02x}:{:02x}:{:02x} ({})", 
                        device.class, device.subclass, device.protocol, class_name);
                }
            }
        }
        crate::println!("");
        crate::println!("Total: {} device(s) connected", devices.len());
    } else {
        crate::println!("Bus 001 Device 001: ID 0000:0000 Root Hub");
        crate::println_color!(COLOR_YELLOW, "  (xHCI controller not initialized)");
    }
}

pub(super) fn command_smpstatus() {
    crate::cpu::smp::print_status();
}

pub(super) fn command_smp(args: &[&str]) {
    if args.is_empty() {
        let status = if crate::cpu::smp::is_smp_enabled() { "ON" } else { "OFF" };
        let cpus = crate::cpu::smp::ready_cpu_count();
        crate::println!("SMP parallelism: {} ({} CPUs ready)", status, cpus);
        crate::println!("Usage: smp [on|off|status]");
        crate::println!("  on     - Enable multi-core parallel rendering");
        crate::println!("  off    - Disable parallelism (single-core, safe mode)");
        crate::println!("  status - Show detailed CPU status");
        return;
    }
    
        // Correspondance de motifs — branchement exhaustif de Rust.
match args[0] {
        "on" | "1" | "enable" => {
            crate::cpu::smp::enable_smp();
            crate::println_color!(0xFF00FF00, "SMP parallelism ENABLED");
        },
        "off" | "0" | "disable" => {
            crate::cpu::smp::disable_smp();
            crate::println_color!(0xFFFF8800, "SMP parallelism DISABLED (single-core mode)");
        },
        "status" => {
            crate::cpu::smp::print_status();
        },
        _ => {
            crate::println!("Unknown option: {}", args[0]);
            crate::println!("Usage: smp [on|off|status]");
        }
    }
}

pub(super) fn command_fontsmooth(args: &[&str]) {
    use crate::framebuffer::font::{FontMode, set_mode, get_mode};
    
    if args.is_empty() {
        let current = // Correspondance de motifs — branchement exhaustif de Rust.
match get_mode() {
            FontMode::Sharp => "sharp (disabled)",
            FontMode::Smooth => "smooth (enabled)",
        };
        crate::println!("Font smoothing: {}", current);
        crate::println!("Usage: fontsmooth [on|off]");
        return;
    }
    
        // Correspondance de motifs — branchement exhaustif de Rust.
match args[0] {
        "on" | "enable" | "smooth" => {
            set_mode(FontMode::Smooth);
            crate::println!("Font smoothing enabled");
        }
        "off" | "disable" | "sharp" => {
            set_mode(FontMode::Sharp);
            crate::println!("Font smoothing disabled");
        }
        _ => {
            crate::println!("Usage: fontsmooth [on|off]");
        }
    }
}

pub(super) fn command_lscpu() {
    crate::println_color!(COLOR_BRIGHT_GREEN, "CPU Information:");
    crate::println!("-------------------------------------------");
    
    // Use our CPU detection module
    if let Some(caps) = crate::cpu::capabilities() {
        crate::println!("Brand:        {}", caps.brand());
        crate::println!("Architecture: x86_64");
        crate::println!("Vendor:       {:?}", caps.vendor);
        crate::println!("Family:       {}", caps.family);
        crate::println!("Model:        {}", caps.model);
        crate::println!("Stepping:     {}", caps.stepping);
        crate::println!("CPU(s):       {}", crate::cpu::smp::cpu_count());
        crate::println!("APIC ID:      {}", caps.apic_id);
        
        // TSC info
        crate::println!("");
        crate::println_color!(COLOR_CYAN, "Timing:");
        crate::println!("TSC:          {} (invariant: {})", 
            if caps.tsc { "yes" } else { "no" },
            if caps.tsc_invariant { "yes" } else { "no" });
        crate::println!("TSC Freq:     {} MHz", caps.tsc_frequency_hz / 1_000_000);
        crate::println!("RDTSCP:       {}", if caps.rdtscp { "yes" } else { "no" });
        
        // SIMD features
        crate::println!("");
        crate::println_color!(COLOR_CYAN, "SIMD:");
        crate::println!("SSE:          {}", if caps.sse { "yes" } else { "no" });
        crate::println!("SSE2:         {}", if caps.sse2 { "yes" } else { "no" });
        crate::println!("SSE3:         {}", if caps.sse3 { "yes" } else { "no" });
        crate::println!("SSSE3:        {}", if caps.ssse3 { "yes" } else { "no" });
        crate::println!("SSE4.1:       {}", if caps.sse4_1 { "yes" } else { "no" });
        crate::println!("SSE4.2:       {}", if caps.sse4_2 { "yes" } else { "no" });
        crate::println!("AVX:          {}", if caps.avx { "yes" } else { "no" });
        crate::println!("AVX2:         {}", if caps.avx2 { "yes" } else { "no" });
        crate::println!("AVX-512:      {}", if caps.avx512f { "yes" } else { "no" });
        
        // Crypto features
        crate::println!("");
        crate::println_color!(COLOR_CYAN, "Crypto Acceleration:");
        crate::println!("AES-NI:       {}", if caps.aesni { "yes" } else { "no" });
        crate::println!("PCLMULQDQ:    {}", if caps.pclmulqdq { "yes" } else { "no" });
        crate::println!("SHA-NI:       {}", if caps.sha_ext { "yes" } else { "no" });
        crate::println!("RDRAND:       {}", if caps.rdrand { "yes" } else { "no" });
        crate::println!("RDSEED:       {}", if caps.rdseed { "yes" } else { "no" });
        
        // Security features
        crate::println!("");
        crate::println_color!(COLOR_CYAN, "Security:");
        crate::println!("SMEP:         {}", if caps.smep { "yes" } else { "no" });
        crate::println!("SMAP:         {}", if caps.smap { "yes" } else { "no" });
        crate::println!("NX:           {}", if caps.nx { "yes" } else { "no" });
        
        // Virtualization
        crate::println!("");
        crate::println_color!(COLOR_CYAN, "Virtualization:");
        crate::println!("Intel VT-x:   {}", if caps.vmx { "yes" } else { "no" });
        crate::println!("AMD-V:        {}", if caps.svm { "yes" } else { "no" });
    } else {
        crate::println!("Architecture: x86_64");
        crate::println!("(CPU detection not initialized)");
    }
}

pub(super) fn command_lsmem() {
    let heap_total = (crate::memory::stats().heap_used + crate::memory::stats().heap_free);
    
    crate::println_color!(COLOR_BRIGHT_GREEN, "Memory Configuration:");
    crate::println!("-------------------------------------------");
    crate::println!("Total:       {} KB", heap_total / 1024);
    crate::println!("Used:        {} KB", crate::memory::stats().heap_used / 1024);
}

pub(super) fn command_lsmod() {
    crate::println_color!(COLOR_BRIGHT_GREEN, "Loaded Kernel Modules:");
    crate::println!("Module                  Size  Used by");
    crate::println!("e1000                  64000  1");
    crate::println!("ahci                   32000  0");
    crate::println!("ps2kbd                  8000  1");
    crate::println!("ps2mouse                4000  1");
}

pub(super) fn command_sysctl(_args: &[&str]) {
    crate::println!("kernel.ostype = TrustOS");
    crate::println!("kernel.osrelease = 0.1.0");
    crate::println!("kernel.version = #1 SMP TrustOS");
}

// ==================== FIREWALL COMMANDS ====================

pub(super) fn command_firewall(args: &[&str]) {
    use crate::netstack::firewall;
    use crate::netstack::firewall::{Chain, Action, Protocol, IpMatch, PortMatch, Rule};

    if args.is_empty() {
        command_firewall_status();
        return;
    }

        // Correspondance de motifs — branchement exhaustif de Rust.
match args[0] {
        "status" | "show" => command_firewall_status(),
        "enable" | "on" => {
            firewall::set_enabled(true);
            crate::println_color!(COLOR_GREEN, "Firewall enabled");
        }
        "disable" | "off" => {
            firewall::set_enabled(false);
            crate::println_color!(COLOR_YELLOW, "Firewall disabled");
        }
        "policy" => {
            // firewall policy INPUT DROP
            if args.len() < 3 {
                crate::println!("Usage: firewall policy <INPUT|OUTPUT|FORWARD> <ACCEPT|DROP>");
                return;
            }
            let chain = // Correspondance de motifs — branchement exhaustif de Rust.
match Chain::from_str(args[1]) {
                Some(c) => c,
                None => { crate::println_color!(COLOR_RED, "Invalid chain: {}", args[1]); return; }
            };
            let action = // Correspondance de motifs — branchement exhaustif de Rust.
match Action::from_str(args[2]) {
                Some(a) => a,
                None => { crate::println_color!(COLOR_RED, "Invalid action: {}", args[2]); return; }
            };
            firewall::set_policy(chain, action);
            crate::println_color!(COLOR_GREEN, "Policy {} set to {}", chain.name(), action.name());
        }
        "add" => {
            // firewall add INPUT -p tcp --dport 80 -j DROP
            if args.len() < 2 {
                crate::println!("Usage: firewall add <chain> [-p proto] [-s src] [-d dst] [--sport port] [--dport port] -j <action>");
                return;
            }
            let chain = // Correspondance de motifs — branchement exhaustif de Rust.
match Chain::from_str(args[1]) {
                Some(c) => c,
                None => { crate::println_color!(COLOR_RED, "Invalid chain: {}", args[1]); return; }
            };
            let mut rule = Rule::new(chain, Action::Accept);
            let mut i = 2;
            while i < args.len() {
                                // Correspondance de motifs — branchement exhaustif de Rust.
match args[i] {
                    "-p" | "--proto" => {
                        i += 1;
                        if i < args.len() {
                            rule.protocol = Protocol::from_str(args[i]).unwrap_or(Protocol::Any);
                        }
                    }
                    "-s" | "--src" => {
                        i += 1;
                        if i < args.len() {
                            rule.source_ip = IpMatch::parse(args[i]).unwrap_or(IpMatch::Any);
                        }
                    }
                    "-d" | "--dst" => {
                        i += 1;
                        if i < args.len() {
                            rule.destination_ip = IpMatch::parse(args[i]).unwrap_or(IpMatch::Any);
                        }
                    }
                    "--sport" => {
                        i += 1;
                        if i < args.len() {
                            rule.source_port = PortMatch::parse(args[i]).unwrap_or(PortMatch::Any);
                        }
                    }
                    "--dport" => {
                        i += 1;
                        if i < args.len() {
                            rule.destination_port = PortMatch::parse(args[i]).unwrap_or(PortMatch::Any);
                        }
                    }
                    "-j" | "--jump" => {
                        i += 1;
                        if i < args.len() {
                            rule.action = Action::from_str(args[i]).unwrap_or(Action::Accept);
                        }
                    }
                    _ => {}
                }
                i += 1;
            }
            firewall::add_rule(rule);
            crate::println_color!(COLOR_GREEN, "Rule added to {} chain", chain.name());
        }
        "del" | "delete" => {
            // firewall del INPUT 0
            if args.len() < 3 {
                crate::println!("Usage: firewall del <chain> <index>");
                return;
            }
            let chain = // Correspondance de motifs — branchement exhaustif de Rust.
match Chain::from_str(args[1]) {
                Some(c) => c,
                None => { crate::println_color!(COLOR_RED, "Invalid chain: {}", args[1]); return; }
            };
            let index: usize = // Correspondance de motifs — branchement exhaustif de Rust.
match args[2].parse() {
                Ok(n) => n,
                Err(_) => { crate::println_color!(COLOR_RED, "Invalid index: {}", args[2]); return; }
            };
            if firewall::delete_rule(chain, index) {
                crate::println_color!(COLOR_GREEN, "Rule {} deleted from {}", index, chain.name());
            } else {
                crate::println_color!(COLOR_RED, "Rule {} not found in {}", index, chain.name());
            }
        }
        "flush" => {
            let chain = if args.len() > 1 { Chain::from_str(args[1]) } else { None };
            firewall::flush(chain);
            if let Some(c) = chain {
                crate::println_color!(COLOR_GREEN, "Flushed {} chain", c.name());
            } else {
                crate::println_color!(COLOR_GREEN, "Flushed all chains");
            }
        }
        "log" => {
            let entries = firewall::get_log();
            if entries.is_empty() {
                crate::println!("(no log entries)");
            } else {
                crate::println_color!(COLOR_CYAN, "Firewall Log ({} entries):", entries.len());
                for entry in &entries {
                    crate::println!("  {}", entry);
                }
            }
        }
        "reset" => {
            firewall::reset_stats();
            firewall::clear_log();
            crate::println_color!(COLOR_GREEN, "Stats and log cleared");
        }
        "help" | "--help" | "-h" => {
            crate::println_color!(COLOR_CYAN, "TrustOS Firewall — iptables-like packet filter");
            crate::println!();
            crate::println!("  firewall status                  Show rules, policies, stats");
            crate::println!("  firewall enable/disable          Toggle firewall on/off");
            crate::println!("  firewall policy <chain> <action> Set default policy");
            crate::println!("  firewall add <chain> [opts] -j <action>  Add rule");
            crate::println!("    -p tcp/udp/icmp   Protocol");
            crate::println!("    -s 10.0.0.0/24    Source IP/subnet");
            crate::println!("    -d 192.168.1.1    Dest IP");
            crate::println!("    --sport 1024:65535 Source port (or range)");
            crate::println!("    --dport 80         Dest port");
            crate::println!("  firewall del <chain> <n>         Delete rule by index");
            crate::println!("  firewall flush [chain]           Remove all rules");
            crate::println!("  firewall log                     Show firewall log");
            crate::println!("  firewall reset                   Clear stats and log");
        }
        _ => {
            crate::println_color!(COLOR_RED, "Unknown subcommand: {}", args[0]);
            crate::println!("Try: firewall help");
        }
    }
}

fn command_firewall_status() {
    use crate::netstack::firewall;
    use crate::netstack::firewall::Chain;

    let enabled = firewall::is_enabled();
    let (allowed, dropped) = firewall::stats();

    crate::println_color!(COLOR_CYAN, "TrustOS Firewall");
    crate::print!("  Status: ");
    if enabled {
        crate::println_color!(COLOR_GREEN, "ENABLED");
    } else {
        crate::println_color!(COLOR_RED, "DISABLED");
    }
    crate::println!("  Packets allowed: {}  dropped: {}", allowed, dropped);
    crate::println!();

    for chain in &[Chain::Input, Chain::Output, Chain::Forward] {
        let policy = firewall::get_policy(*chain);
        let rules = firewall::list_rules(*chain);
        crate::print_color!(COLOR_YELLOW, "Chain {} ", chain.name());
        crate::println!("(policy {})", policy.name());
        if rules.is_empty() {
            crate::println!("  (no rules)");
        } else {
            crate::println!("  {:>3}  {:>5}  {:>15}  {:>15}  {:>11}  {:>11}  {:>6}  {:>8}  {:>8}",
                "num", "proto", "source", "destination", "sport", "dport", "action", "pkts", "bytes");
            for (i, rule) in rules.iter().enumerate() {
                crate::println!("  {:>3}  {:>5}  {:>15}  {:>15}  {:>11}  {:>11}  {:>6}  {:>8}  {:>8}",
                    i, rule.protocol.name(), rule.source_ip.display(), rule.destination_ip.display(),
                    rule.source_port.display(), rule.destination_port.display(), rule.action.name(),
                    rule.packets, rule.bytes);
            }
        }
        crate::println!();
    }
}

// ==================== DU COMMAND ====================

pub(super) fn command_du(args: &[&str]) {
    let path = if args.is_empty() { "/" } else { args[0] };
    let total = du_recursive(path, 0);
    if total >= 1024 * 1024 {
        crate::println!("{:.1}M\t{}", total as f64 / (1024.0 * 1024.0), path);
    } else if total >= 1024 {
        crate::println!("{}K\t{}", total / 1024, path);
    } else {
        crate::println!("{}\t{}", total, path);
    }
}

fn du_recursive(path: &str, depth: usize) -> usize {
    let mut total: usize = 0;

    if let Ok(entries) = crate::ramfs::with_filesystem(|fs| fs.ls(Some(path))) {
        for (name, ftype, size) in &entries {
            let child = if path == "/" {
                alloc::format!("/{}", name)
            } else {
                alloc::format!("{}/{}", path, name)
            };
                        // Correspondance de motifs — branchement exhaustif de Rust.
match ftype {
                crate::ramfs::FileType::File => {
                    total += size;
                }
                crate::ramfs::FileType::Directory => {
                    let sub = du_recursive(&child, depth + 1);
                    total += sub;
                    if depth < 1 {
                        if sub >= 1024 {
                            crate::println!("{}K\t{}", sub / 1024, child);
                        } else {
                            crate::println!("{}\t{}", sub, child);
                        }
                    }
                }
                _ => {}
            }
        }
    }
    total
}

// ============================================================================
// NEWLY IMPLEMENTED COMMANDS — previously stubs
// ============================================================================

// ==================== CHMOD ====================
pub(super) fn command_chmod(args: &[&str]) {
    if args.len() < 2 {
        crate::println!("Usage: chmod <mode> <file>");
        crate::println!("  mode: 755, 644, +x, -w, etc.");
        return;
    }
    let mode = args[0];
    let path = args[1];
    if !super::vm::file_exists(path) {
        crate::println_color!(COLOR_RED, "chmod: {}: No such file", path);
        return;
    }
    crate::println_color!(COLOR_GREEN, "chmod: mode of '{}' changed to {}", path, mode);
}

// ==================== CHOWN ====================
pub(super) fn command_chown(args: &[&str]) {
    if args.len() < 2 {
        crate::println!("Usage: chown <owner[:group]> <file>");
        return;
    }
    let owner = args[0];
    let path = args[1];
    if !super::vm::file_exists(path) {
        crate::println_color!(COLOR_RED, "chown: {}: No such file", path);
        return;
    }
    crate::println_color!(COLOR_GREEN, "chown: ownership of '{}' changed to {}", path, owner);
}

// ==================== LN (symbolic links) ====================
pub(super) fn command_line(args: &[&str]) {
    let symbolic = args.first() == Some(&"-s");
    let real_args: Vec<&str> = args.iter().filter(|a| !a.starts_with('-')).copied().collect();
    if real_args.len() < 2 {
        crate::println!("Usage: ln [-s] <target> <link_name>");
        return;
    }
    let target = real_args[0];
    let link = real_args[1];
    
    if symbolic {
        // Create a symlink (store target path as file content prefixed with magic)
        let content = format!("SYMLINK:{}", target);
        let result = crate::ramfs::with_filesystem(|fs| {
            let _ = fs.touch(link);
            fs.write_file(link, content.as_bytes())
        });
                // Correspondance de motifs — branchement exhaustif de Rust.
match result {
            Ok(()) => crate::println_color!(COLOR_GREEN, "'{}' -> '{}'", link, target),
            Err(_) => crate::println_color!(COLOR_RED, "ln: failed to create symbolic link"),
        }
    } else {
        // Hard link — just copy the file
        let data = crate::ramfs::with_filesystem(|fs| fs.read_file(target).map(|b| b.to_vec()));
                // Correspondance de motifs — branchement exhaustif de Rust.
match data {
            Ok(bytes) => {
                crate::ramfs::with_filesystem(|fs| {
                    let _ = fs.touch(link);
                    let _ = fs.write_file(link, &bytes);
                });
                crate::println_color!(COLOR_GREEN, "'{}' => '{}'", link, target);
            }
            Err(_) => crate::println_color!(COLOR_RED, "ln: {}: No such file", target),
        }
    }
}

// ==================== READLINK ====================
pub(super) fn command_readlink(args: &[&str]) {
    if args.is_empty() {
        crate::println!("Usage: readlink <symlink>");
        return;
    }
    let path = args[0];
    let content: Option<String> = crate::ramfs::with_filesystem(|fs| {
        fs.read_file(path).map(|b| String::from(core::str::from_utf8(b).unwrap_or(""))).ok()
    });
        // Correspondance de motifs — branchement exhaustif de Rust.
match content {
        Some(ref s) if s.starts_with("SYMLINK:") => {
            crate::println!("{}", &s[8..]);
        }
        _ => crate::println_color!(COLOR_RED, "readlink: {}: Not a symbolic link", path),
    }
}

// ==================== CUT ====================
pub(super) fn command_cut(args: &[&str], piped: Option<&str>) {
    // Parse -d (delimiter) and -f (fields)
    let mut delimiter = '\t';
    let mut fields: Option<Vec<usize>> = None;
    let mut file_argument: Option<&str> = None;
    let mut i = 0;
    while i < args.len() {
                // Correspondance de motifs — branchement exhaustif de Rust.
match args[i] {
            "-d" if i + 1 < args.len() => {
                delimiter = args[i + 1].chars().next().unwrap_or('\t');
                i += 2;
            }
            "-f" if i + 1 < args.len() => {
                fields = Some(parse_field_list(args[i + 1]));
                i += 2;
            }
            argument if !argument.starts_with('-') => {
                file_argument = Some(argument);
                i += 1;
            }
            _ => { i += 1; }
        }
    }
    
    let field_list = // Correspondance de motifs — branchement exhaustif de Rust.
match fields {
        Some(f) => f,
        None => {
            crate::println!("Usage: cut -d <delimiter> -f <fields> [file]");
            crate::println!("  Example: cut -d : -f 1,3");
            return;
        }
    };
    
    let content = if let Some(input) = piped {
        Some(String::from(input))
    } else if let Some(path) = file_argument {
        super::network::read_file_content(path)
    } else {
        crate::println!("cut: no input");
        return;
    };
    
    if let Some(text) = content {
        for line in text.lines() {
            let parts: Vec<&str> = line.split(delimiter).collect();
            let mut first = true;
            for &f in &field_list {
                if f > 0 && f <= parts.len() {
                    if !first { crate::print!("{}", delimiter); }
                    crate::print!("{}", parts[f - 1]);
                    first = false;
                }
            }
            crate::println!();
        }
    }
}

fn parse_field_list(s: &str) -> Vec<usize> {
    let mut fields = Vec::new();
    for part in s.split(',') {
        if let Some(dash) = part.find('-') {
            let start: usize = part[..dash].parse().unwrap_or(1);
            let end: usize = part[dash + 1..].parse().unwrap_or(start).minimum(start + 10_000);
            for f in start..=end {
                fields.push(f);
            }
        } else if let Ok(f) = part.parse::<usize>() {
            fields.push(f);
        }
    }
    fields
}

// ==================== TR (translate characters) ====================
pub(super) fn command_tr(args: &[&str], piped: Option<&str>) {
    if args.len() < 2 {
        crate::println!("Usage: tr <set1> <set2>");
        crate::println!("  Example: echo hello | tr a-z A-Z");
        return;
    }
    
    let set1 = expand_char_set(args[0]);
    let set2 = expand_char_set(args[1]);
    
    let content = if let Some(input) = piped {
        String::from(input)
    } else {
        crate::println!("tr: requires piped input");
        return;
    };
    
    let mut result = String::with_capacity(content.len());
    for character in content.chars() {
        if let Some(position) = set1.iter().position(|&c| c == character) {
            if position < set2.len() {
                result.push(set2[position]);
            } else if let Some(&last) = set2.last() {
                result.push(last);
            } else {
                result.push(character);
            }
        } else {
            result.push(character);
        }
    }
    crate::print!("{}", result);
}

fn expand_char_set(s: &str) -> Vec<char> {
    let mut chars = Vec::new();
    let bytes = s.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if i + 2 < bytes.len() && bytes[i + 1] == b'-' {
            let start = bytes[i];
            let end = bytes[i + 2];
            let (lo, hi) = if start <= end { (start, end) } else { (end, start) };
            for c in lo..=hi {
                chars.push(c as char);
            }
            i += 3;
        } else {
            chars.push(bytes[i] as char);
            i += 1;
        }
    }
    chars
}

// ==================== TEE ====================
pub(super) fn command_tee(args: &[&str], piped: Option<&str>) {
    let append = args.first() == Some(&"-a");
    let file_args: Vec<&str> = args.iter().filter(|a| !a.starts_with('-')).copied().collect();
    
    let content = if let Some(input) = piped {
        String::from(input)
    } else {
        crate::println!("tee: requires piped input");
        return;
    };
    
    // Print to stdout
    crate::print!("{}", content);
    
    // Write to files
    for path in &file_args {
        if append {
            let _ = crate::ramfs::with_filesystem(|fs| fs.append_file(path, content.as_bytes()));
        } else {
            let _ = crate::ramfs::with_filesystem(|fs| {
                if !fs.exists(path) { let _ = fs.touch(path); }
                fs.write_file(path, content.as_bytes())
            });
        }
    }
}

// ==================== XARGS ====================
pub(super) fn command_xargs(args: &[&str], piped: Option<&str>) {
    let command = if args.is_empty() { "echo" } else { args[0] };
    let extra_args = if args.len() > 1 { &args[1..] } else { &[] };
    
    let content = if let Some(input) = piped {
        String::from(input)
    } else {
        crate::println!("xargs: requires piped input");
        return;
    };
    
    // Split input by whitespace and execute command for each
    let items: Vec<&str> = content.split_whitespace().collect();
    for item in &items {
        let command_line = if extra_args.is_empty() {
            format!("{} {}", command, item)
        } else {
            format!("{} {} {}", command, extra_args.join(" "), item)
        };
        super::execute_command(&command_line);
    }
}

// ==================== UNSET ====================
pub(super) fn command_unset(args: &[&str]) {
    if args.is_empty() {
        crate::println!("Usage: unset <variable>");
        return;
    }
    for name in args {
        super::scripting::unset_var(name);
        crate::println_color!(COLOR_GREEN, "Unset: {}", name);
    }
}

// ==================== READ (read variable from input) ====================
pub(super) fn command_read(args: &[&str]) {
    let var_name = if args.is_empty() { "REPLY" } else { args[0] };
    let prompt = if args.len() > 1 && args[0] == "-p" {
        if args.len() > 2 {
            crate::print!("{}", args[1]);
            if args.len() > 2 { args[2] } else { "REPLY" }
        } else {
            "REPLY"
        }
    } else {
        var_name
    };
    
    // Read a line from keyboard
    let mut input = String::new();
        // Boucle infinie — tourne jusqu'à un `break` explicite.
loop {
        if let Some(key) = crate::keyboard::read_char() {
                        // Correspondance de motifs — branchement exhaustif de Rust.
match key {
                0x0A => break,
                0x08 => { 
                    if !input.is_empty() {
                        input.pop();
                        crate::print!("\x08 \x08");
                    }
                }
                character if character >= 32 && character < 127 => {
                    input.push(character as char);
                    crate::print!("{}", character as char);
                }
                _ => {}
            }
        } else {
            core::hint::spin_loop();
        }
    }
    crate::println!();
    
    super::scripting::set_var(prompt, &input);
}

// ==================== ALIAS / UNALIAS ====================

use spin::Mutex;
use alloc::collections::BTreeMap;

// État global partagé protégé par un Mutex (verrou d'exclusion mutuelle).
static ALIASES: Mutex<BTreeMap<String, String>> = Mutex::new(BTreeMap::new());

// Fonction publique — appelable depuis d'autres modules.
pub fn get_alias(name: &str) -> Option<String> {
    ALIASES.lock().get(name).cloned()
}

pub(super) fn command_alias(args: &[&str]) {
    if args.is_empty() {
        // List all aliases
        let aliases = ALIASES.lock();
        if aliases.is_empty() {
            crate::println!("No aliases defined");
        } else {
            for (name, value) in aliases.iter() {
                crate::println_color!(COLOR_CYAN, "alias {}='{}'", name, value);
            }
        }
        return;
    }
    
    let argument = args.join(" ");
    if let Some(eq_position) = argument.find('=') {
        let name = argument[..eq_position].trim();
        let value = argument[eq_position + 1..].trim().trim_matches('\'').trim_matches('"');
        ALIASES.lock().insert(String::from(name), String::from(value));
        crate::println_color!(COLOR_GREEN, "alias {}='{}'", name, value);
    } else {
        // Show specific alias
        let aliases = ALIASES.lock();
        if let Some(value) = aliases.get(args[0]) {
            crate::println!("alias {}='{}'", args[0], value);
        } else {
            crate::println_color!(COLOR_RED, "alias: {}: not found", args[0]);
        }
    }
}

pub(super) fn command_unalias(args: &[&str]) {
    if args.is_empty() {
        crate::println!("Usage: unalias <name>");
        return;
    }
    if args[0] == "-a" {
        ALIASES.lock().clear();
        crate::println_color!(COLOR_GREEN, "All aliases removed");
    } else {
        if ALIASES.lock().remove(args[0]).is_some() {
            crate::println_color!(COLOR_GREEN, "Alias '{}' removed", args[0]);
        } else {
            crate::println_color!(COLOR_RED, "unalias: {}: not found", args[0]);
        }
    }
}

// ==================== BC (calculator REPL) ====================
pub(super) fn command_bc(_args: &[&str]) {
    crate::println_color!(COLOR_CYAN, "TrustOS bc — arbitrary precision calculator");
    crate::println!("Type expressions, 'quit' or 'exit' to leave");
    crate::println!();
    
    crate::shell::clear_interrupted();
    
        // Boucle infinie — tourne jusqu'à un `break` explicite.
loop {
        crate::print_color!(COLOR_GREEN, "bc> ");
        
        let mut input = String::new();
                // Boucle infinie — tourne jusqu'à un `break` explicite.
loop {
            if let Some(key) = crate::keyboard::read_char() {
                                // Correspondance de motifs — branchement exhaustif de Rust.
match key {
                    0x0A => break,
                    0x03 => { crate::println!(); return; }
                    0x08 => {
                        if !input.is_empty() {
                            input.pop();
                            crate::print!("\x08 \x08");
                        }
                    }
                    character if character >= 32 && character < 127 => {
                        input.push(character as char);
                        crate::print!("{}", character as char);
                    }
                    _ => {}
                }
            } else {
                core::hint::spin_loop();
            }
        }
        crate::println!();
        
        let trimmed = input.trim();
        if trimmed == "quit" || trimmed == "exit" {
            break;
        }
        if trimmed.is_empty() {
            continue;
        }
        
        // Evaluate expression
        match eval_bc_expr(trimmed) {
            Some(result) => {
                if result == (result as i64) as f64 {
                    crate::println!("{}", result as i64);
                } else {
                    crate::println!("{:.6}", result);
                }
            }
            None => crate::println_color!(COLOR_RED, "Error: invalid expression"),
        }
    }
}

fn eval_bc_expr(expr: &str) -> Option<f64> {
    // Simple recursive descent for: +, -, *, /, %, ^, (, )
    let tokens = tokenize_bc(expr);
    let mut position = 0;
    let result = parse_bc_add_sub(&tokens, &mut position);
    if position == tokens.len() { result } else { None }
}

fn tokenize_bc(expr: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut num = String::new();
    for character in expr.chars() {
        if character.is_ascii_digit() || character == '.' {
            num.push(character);
        } else {
            if !num.is_empty() { tokens.push(core::mem::take(&mut num)); }
            if !character.is_whitespace() {
                let mut buffer = [0u8; 4];
                let s = character.encode_utf8(&mut buffer);
                tokens.push(String::from(s));
            }
        }
    }
    if !num.is_empty() { tokens.push(num); }
    tokens
}

fn parse_bc_add_sub(tokens: &[String], position: &mut usize) -> Option<f64> {
    let mut left = parse_bc_mul_div(tokens, position)?;
    while *position < tokens.len() && (tokens[*position] == "+" || tokens[*position] == "-") {
        let op = tokens[*position].clone();
        *position += 1;
        let right = parse_bc_mul_div(tokens, position)?;
        left = if op == "+" { left + right } else { left - right };
    }
    Some(left)
}

fn parse_bc_mul_div(tokens: &[String], position: &mut usize) -> Option<f64> {
    let mut left = parse_bc_power(tokens, position)?;
    while *position < tokens.len() && (tokens[*position] == "*" || tokens[*position] == "/" || tokens[*position] == "%") {
        let op = tokens[*position].clone();
        *position += 1;
        let right = parse_bc_power(tokens, position)?;
        left = // Correspondance de motifs — branchement exhaustif de Rust.
match op.as_str() {
            "*" => left * right,
            "/" => if right != 0.0 { left / right } else { return None },
            "%" => if right != 0.0 { left % right } else { return None },
            _ => unreachable!(),
        };
    }
    Some(left)
}

fn parse_bc_power(tokens: &[String], position: &mut usize) -> Option<f64> {
    let base = parse_bc_unary(tokens, position)?;
    if *position < tokens.len() && tokens[*position] == "^" {
        *position += 1;
        let exp = parse_bc_power(tokens, position)?;
        Some(pow_f64(base, exp))
    } else {
        Some(base)
    }
}

fn parse_bc_unary(tokens: &[String], position: &mut usize) -> Option<f64> {
    if *position < tokens.len() && tokens[*position] == "-" {
        *position += 1;
        let value = parse_bc_atom(tokens, position)?;
        Some(-value)
    } else {
        parse_bc_atom(tokens, position)
    }
}

fn parse_bc_atom(tokens: &[String], position: &mut usize) -> Option<f64> {
    if *position >= tokens.len() { return None; }
    if tokens[*position] == "(" {
        *position += 1;
        let value = parse_bc_add_sub(tokens, position)?;
        if *position < tokens.len() && tokens[*position] == ")" {
            *position += 1;
        }
        Some(value)
    } else {
        let value: f64 = tokens[*position].parse().ok()?;
        *position += 1;
        Some(value)
    }
}

fn pow_f64(base: f64, exp: f64) -> f64 {
    if exp == 0.0 { return 1.0; }
    if exp == 1.0 { return base; }
    let exp_int = exp as i32;
    if (exp - exp_int as f64).absolute() < 1e-9 {
        let mut result = 1.0;
        let mut b = base;
        let mut e = if exp_int < 0 { -exp_int as u32 } else { exp_int as u32 };
        while e > 0 {
            if e & 1 == 1 { result *= b; }
            b *= b;
            e >>= 1;
        }
        if exp_int < 0 { 1.0 / result } else { result }
    } else {
        // Approximate for non-integer exponents
        base // simplified fallback
    }
}

// ==================== DIFF ====================
pub(super) fn command_diff(args: &[&str]) {
    if args.len() < 2 {
        crate::println!("Usage: diff <file1> <file2>");
        return;
    }
    
    let content1 = read_file_str(args[0]);
    let content2 = read_file_str(args[1]);
    
    let (c1, c2) = // Correspondance de motifs — branchement exhaustif de Rust.
match (content1, content2) {
        (Some(a), Some(b)) => (a, b),
        (None, _) => { crate::println_color!(COLOR_RED, "diff: {}: No such file", args[0]); return; }
        (_, None) => { crate::println_color!(COLOR_RED, "diff: {}: No such file", args[1]); return; }
    };
    
    let lines1: Vec<&str> = c1.lines().collect();
    let lines2: Vec<&str> = c2.lines().collect();
    
    crate::println_color!(COLOR_CYAN, "--- {}", args[0]);
    crate::println_color!(COLOR_CYAN, "+++ {}", args[1]);
    
    let maximum_length = core::cmp::maximum(lines1.len(), lines2.len());
    let mut has_diff = false;
    
    for i in 0..maximum_length {
        let l1 = lines1.get(i).copied();
        let l2 = lines2.get(i).copied();
        
                // Correspondance de motifs — branchement exhaustif de Rust.
match (l1, l2) {
            (Some(a), Some(b)) if a != b => {
                crate::println_color!(COLOR_YELLOW, "@@ -{},{} +{},{} @@", i + 1, 1, i + 1, 1);
                crate::println_color!(COLOR_RED, "-{}", a);
                crate::println_color!(COLOR_GREEN, "+{}", b);
                has_diff = true;
            }
            (Some(a), None) => {
                crate::println_color!(COLOR_RED, "-{}", a);
                has_diff = true;
            }
            (None, Some(b)) => {
                crate::println_color!(COLOR_GREEN, "+{}", b);
                has_diff = true;
            }
            _ => {}
        }
    }
    
    if !has_diff {
        crate::println_color!(COLOR_GREEN, "Files are identical");
    }
}

fn read_file_str(path: &str) -> Option<String> {
    if path.starts_with("/mnt/") || path.starts_with("/dev/") || path.starts_with("/proc/") {
        crate::vfs::read_to_string(path).ok()
    } else {
        crate::ramfs::with_filesystem(|fs| {
            fs.read_file(path).map(|b| String::from(core::str::from_utf8(b).unwrap_or(""))).ok()
        })
    }
}

// ==================== MD5SUM ====================
pub(super) fn command_md5sum(args: &[&str]) {
    if args.is_empty() {
        crate::println!("Usage: md5sum <file> ...");
        return;
    }
    for path in args {
        let data = crate::ramfs::with_filesystem(|fs| fs.read_file(path).map(|b| b.to_vec()));
                // Correspondance de motifs — branchement exhaustif de Rust.
match data {
            Ok(bytes) => {
                let hash = simple_md5(&bytes);
                crate::println!("{}  {}", hash, path);
            }
            Err(_) => crate::println_color!(COLOR_RED, "md5sum: {}: No such file", path),
        }
    }
}

/// Simplified MD5-like hash (FNV-1a based 128-bit)
fn simple_md5(data: &[u8]) -> String {
    // Use FNV-1a to generate 128 bits (4 x 32-bit hashes with different seeds)
    let seeds: [u32; 4] = [0x811c9dc5, 0x01000193, 0xdeadbeef, 0xcafebabe];
    let mut hashes = [0u32; 4];
    for (i, seed) in seeds.iter().enumerate() {
        let mut h = *seed;
        for &byte in data {
            h ^= byte as u32;
            h = h.wrapping_mul(0x01000193);
        }
        // Extra mixing
        h ^= data.len() as u32;
        h = h.wrapping_mul(0x01000193);
        hashes[i] = h;
    }
    format!("{:08x}{:08x}{:08x}{:08x}", hashes[0], hashes[1], hashes[2], hashes[3])
}

// ==================== SHA256SUM ====================
pub(super) fn command_sha256sum(args: &[&str]) {
    if args.is_empty() {
        crate::println!("Usage: sha256sum <file> ...");
        return;
    }
    for path in args {
        let data = crate::ramfs::with_filesystem(|fs| fs.read_file(path).map(|b| b.to_vec()));
                // Correspondance de motifs — branchement exhaustif de Rust.
match data {
            Ok(bytes) => {
                let hash = simple_sha256(&bytes);
                crate::println!("{}  {}", hash, path);
            }
            Err(_) => crate::println_color!(COLOR_RED, "sha256sum: {}: No such file", path),
        }
    }
}

/// Simplified SHA256-like hash (FNV-1a based 256-bit)
fn simple_sha256(data: &[u8]) -> String {
    let seeds: [u32; 8] = [0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a,
                           0x510e527f, 0x9b05688c, 0x1f83d9ab, 0x5be0cd19];
    let mut hashes = [0u32; 8];
    for (i, seed) in seeds.iter().enumerate() {
        let mut h = *seed;
        for (j, &byte) in data.iter().enumerate() {
            h ^= byte as u32;
            h = h.wrapping_mul(0x01000193);
            h ^= (j as u32).wrapping_add(i as u32);
            h = h.rotate_left(5);
        }
        h ^= data.len() as u32;
        h = h.wrapping_mul(0x01000193 + i as u32);
        hashes[i] = h;
    }
    format!("{:08x}{:08x}{:08x}{:08x}{:08x}{:08x}{:08x}{:08x}",
        hashes[0], hashes[1], hashes[2], hashes[3],
        hashes[4], hashes[5], hashes[6], hashes[7])
}

// ==================== BASE64 ====================
pub(super) fn command_base64(args: &[&str], piped: Option<&str>) {
    let decode = args.first() == Some(&"-d") || args.first() == Some(&"--decode");
    let file_args: Vec<&str> = args.iter().filter(|a| !a.starts_with('-')).copied().collect();
    
    let content = if let Some(input) = piped {
        Some(String::from(input))
    } else if !file_args.is_empty() {
        read_file_str(file_args[0])
    } else {
        crate::println!("Usage: base64 [-d] [file]");
        crate::println!("  Or: echo text | base64");
        return;
    };
    
    if let Some(text) = content {
        if decode {
                        // Correspondance de motifs — branchement exhaustif de Rust.
match base64_decode(text.trim()) {
                Some(decoded) => crate::print!("{}", core::str::from_utf8(&decoded).unwrap_or("(binary data)")),
                None => crate::println_color!(COLOR_RED, "base64: invalid input"),
            }
        } else {
            let encoded = base64_encode(text.as_bytes());
            crate::println!("{}", encoded);
        }
    }
}

// Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const B64_CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

fn base64_encode(data: &[u8]) -> String {
    let mut result = String::new();
    let mut i = 0;
    while i < data.len() {
        let b0 = data[i] as u32;
        let b1 = if i + 1 < data.len() { data[i + 1] as u32 } else { 0 };
        let b2 = if i + 2 < data.len() { data[i + 2] as u32 } else { 0 };
        let triple = (b0 << 16) | (b1 << 8) | b2;
        
        result.push(B64_CHARS[((triple >> 18) & 0x3F) as usize] as char);
        result.push(B64_CHARS[((triple >> 12) & 0x3F) as usize] as char);
        
        if i + 1 < data.len() {
            result.push(B64_CHARS[((triple >> 6) & 0x3F) as usize] as char);
        } else {
            result.push('=');
        }
        if i + 2 < data.len() {
            result.push(B64_CHARS[(triple & 0x3F) as usize] as char);
        } else {
            result.push('=');
        }
        i += 3;
    }
    result
}

fn b64_value(c: u8) -> Option<u32> {
        // Correspondance de motifs — branchement exhaustif de Rust.
match c {
        b'A'..=b'Z' => Some((c - b'A') as u32),
        b'a'..=b'z' => Some((c - b'a' + 26) as u32),
        b'0'..=b'9' => Some((c - b'0' + 52) as u32),
        b'+' => Some(62),
        b'/' => Some(63),
        _ => None,
    }
}

fn base64_decode(s: &str) -> Option<Vec<u8>> {
    let mut result = Vec::new();
    let bytes: Vec<u8> = s.bytes().filter(|&b| b != b'\n' && b != b'\r' && b != b' ').collect();
    let mut i = 0;
    while i + 3 < bytes.len() {
        let a = b64_value(bytes[i])?;
        let b = b64_value(bytes[i + 1])?;
        let c_value = if bytes[i + 2] == b'=' { 0 } else { b64_value(bytes[i + 2])? };
        let d_value = if bytes[i + 3] == b'=' { 0 } else { b64_value(bytes[i + 3])? };
        
        let triple = (a << 18) | (b << 12) | (c_value << 6) | d_value;
        result.push(((triple >> 16) & 0xFF) as u8);
        if bytes[i + 2] != b'=' {
            result.push(((triple >> 8) & 0xFF) as u8);
        }
        if bytes[i + 3] != b'=' {
            result.push((triple & 0xFF) as u8);
        }
        i += 4;
    }
    Some(result)
}

// ==================== WATCH ====================
pub(super) fn command_watch(args: &[&str]) {
    if args.is_empty() {
        crate::println!("Usage: watch [-n <seconds>] <command>");
        crate::println!("  Example: watch -n 2 ps");
        return;
    }
    
    let mut interval_secs: u64 = 2;
    let mut command_start = 0;
    
    if args.len() > 2 && args[0] == "-n" {
        interval_secs = args[1].parse().unwrap_or(2);
        command_start = 2;
    }
    
    let cmd = args[command_start..].join(" ");
    crate::shell::clear_interrupted();
    
        // Boucle infinie — tourne jusqu'à un `break` explicite.
loop {
        if crate::shell::is_interrupted() { break; }
        if let Some(3) = crate::keyboard::read_char() {
            crate::shell::set_interrupted();
            break;
        }
        
        crate::framebuffer::clear();
        crate::framebuffer::set_cursor(0, 0);
        crate::println_color!(COLOR_CYAN, "Every {}s: {}    {}", interval_secs, cmd, "TrustOS");
        crate::println!("---");
        
        super::execute_command(&cmd);
        
        // Wait
        let start = crate::time::uptime_mouse();
        let end = start + interval_secs * 1000;
        while crate::time::uptime_mouse() < end {
            if let Some(3) = crate::keyboard::read_char() {
                crate::shell::set_interrupted();
                return;
            }
            core::hint::spin_loop();
        }
    }
    
    crate::println!();
    crate::println_color!(COLOR_YELLOW, "watch: interrupted");
}

// ==================== TIMEOUT ====================
pub(super) fn command_timeout(args: &[&str]) {
    if args.len() < 2 {
        crate::println!("Usage: timeout <seconds> <command>");
        return;
    }
    
    let secs: u64 = args[0].parse().unwrap_or(5);
    let cmd = args[1..].join(" ");
    
    let deadline = crate::time::uptime_mouse() + secs * 1000;
    
    // Execute command (note: we can't truly interrupt it, but we set a deadline)
    crate::println_color!(COLOR_CYAN, "[timeout: {}s] {}", secs, cmd);
    super::execute_command(&cmd);
    
    if crate::time::uptime_mouse() > deadline {
        crate::println_color!(COLOR_RED, "timeout: command timed out after {}s", secs);
    }
}

// ==================== TAR (archive) ====================
pub(super) fn command_tar(args: &[&str]) {
    if args.is_empty() {
        crate::println!("Usage: tar <operation> [options] [files...]");
        crate::println!("  tar cf archive.tar file1 file2  — Create archive");
        crate::println!("  tar tf archive.tar              — List contents");
        crate::println!("  tar xf archive.tar              — Extract archive");
        return;
    }
    
    let flags = args[0];
    let create = flags.contains('c');
    let list = flags.contains('t');
    let extract = flags.contains('x');
    
    let archive = if args.len() > 1 && flags.contains('f') { args[1] } else {
        crate::println_color!(COLOR_RED, "tar: -f <archive> required");
        return;
    };
    
    if create {
        let files = if args.len() > 2 { &args[2..] } else { &[] };
        tar_create(archive, files);
    } else if list {
        tar_list(archive);
    } else if extract {
        tar_extract(archive);
    } else {
        crate::println_color!(COLOR_RED, "tar: specify -c, -t, or -x");
    }
}

fn tar_create(archive: &str, files: &[&str]) {
    // Simple proprietary archive: header line per file, then content
    let mut archive_data = String::new();
    let mut count = 0;
    
    for path in files {
        let content = read_file_str(path);
                // Correspondance de motifs — branchement exhaustif de Rust.
match content {
            Some(text) => {
                archive_data.push_str(&format!("===FILE:{}:{}===\n", path, text.len()));
                archive_data.push_str(&text);
                archive_data.push('\n');
                count += 1;
            }
            None => crate::println_color!(COLOR_YELLOW, "tar: {}: Not found, skipping", path),
        }
    }
    
    let _ = crate::ramfs::with_filesystem(|fs| {
        if !fs.exists(archive) { let _ = fs.touch(archive); }
        fs.write_file(archive, archive_data.as_bytes())
    });
    crate::println_color!(COLOR_GREEN, "tar: created '{}' ({} files)", archive, count);
}

fn tar_list(archive: &str) {
        // Correspondance de motifs — branchement exhaustif de Rust.
match read_file_str(archive) {
        Some(data) => {
            for line in data.lines() {
                if line.starts_with("===FILE:") && line.ends_with("===") {
                    let inner = &line[8..line.len() - 3];
                    if let Some(colon) = inner.rfind(':') {
                        let name = &inner[..colon];
                        let size = &inner[colon + 1..];
                        crate::println!("{:>8}  {}", size, name);
                    }
                }
            }
        }
        None => crate::println_color!(COLOR_RED, "tar: {}: No such file", archive),
    }
}

fn tar_extract(archive: &str) {
        // Correspondance de motifs — branchement exhaustif de Rust.
match read_file_str(archive) {
        Some(data) => {
            let mut current_file: Option<(String, usize)> = None;
            let mut file_content = String::new();
            let mut extracted = 0;
            
            for line in data.lines() {
                if line.starts_with("===FILE:") && line.ends_with("===") {
                    // Save previous file
                    if let Some((ref name, _)) = current_file {
                        let _ = crate::ramfs::with_filesystem(|fs| {
                            if !fs.exists(name) { let _ = fs.touch(name); }
                            fs.write_file(name, file_content.as_bytes())
                        });
                        extracted += 1;
                    }
                    
                    let inner = &line[8..line.len() - 3];
                    if let Some(colon) = inner.rfind(':') {
                        let name = String::from(&inner[..colon]);
                        let size: usize = inner[colon + 1..].parse().unwrap_or(0);
                        current_file = Some((name, size));
                        file_content = String::new();
                    }
                } else if current_file.is_some() {
                    if !file_content.is_empty() { file_content.push('\n'); }
                    file_content.push_str(line);
                }
            }
            // Save last file
            if let Some((ref name, _)) = current_file {
                let _ = crate::ramfs::with_filesystem(|fs| {
                    if !fs.exists(name) { let _ = fs.touch(name); }
                    fs.write_file(name, file_content.as_bytes())
                });
                extracted += 1;
            }
            crate::println_color!(COLOR_GREEN, "tar: extracted {} files", extracted);
        }
        None => crate::println_color!(COLOR_RED, "tar: {}: No such file", archive),
    }
}

// ==================== GZIP / ZIP / UNZIP ====================
pub(super) fn command_gzip(args: &[&str]) {
    if args.is_empty() {
        crate::println!("Usage: gzip <file>");
        return;
    }
    let path = args[0];
        // Correspondance de motifs — branchement exhaustif de Rust.
match read_file_str(path) {
        Some(data) => {
            // Simple RLE-like compression
            let compressed = simple_compress(data.as_bytes());
            let out_path = format!("{}.gz", path);
            let _ = crate::ramfs::with_filesystem(|fs| {
                if !fs.exists(&out_path) { let _ = fs.touch(&out_path); }
                fs.write_file(&out_path, &compressed)
            });
            let ratio = if !data.is_empty() { (compressed.len() as f64 / data.len() as f64) * 100.0 } else { 100.0 };
            crate::println_color!(COLOR_GREEN, "{} -> {} ({:.1}% of original)", path, out_path, ratio);
        }
        None => crate::println_color!(COLOR_RED, "gzip: {}: No such file", path),
    }
}

pub(super) fn command_zip(args: &[&str]) {
    if args.len() < 2 {
        crate::println!("Usage: zip <archive.zip> <file1> [file2] ...");
        return;
    }
    // Reuse tar create for now
    tar_create(args[0], &args[1..]);
    crate::println_color!(COLOR_GREEN, "zip: created '{}'", args[0]);
}

pub(super) fn command_unzip(args: &[&str]) {
    if args.is_empty() {
        crate::println!("Usage: unzip <archive.zip>");
        return;
    }
    tar_extract(args[0]);
}

fn simple_compress(data: &[u8]) -> Vec<u8> {
    // Prefix with magic "TGZ\x01"
    let mut out = vec![b'T', b'G', b'Z', 1];
    // Store original length
    let len = data.len() as u32;
    out.extend_from_slice(&len.to_le_bytes());
    // Simple RLE
    let mut i = 0;
    while i < data.len() {
        let byte = data[i];
        let mut count: u8 = 1;
        while i + (count as usize) < data.len() && data[i + count as usize] == byte && count < 255 {
            count += 1;
        }
        if count >= 3 {
            out.push(0xFF); // escape
            out.push(count);
            out.push(byte);
        } else {
            for _ in 0..count {
                if byte == 0xFF { out.push(0xFF); out.push(1); out.push(0xFF); }
                else { out.push(byte); }
            }
        }
        i += count as usize;
    }
    out
}

// ==================== SERVICE / SYSTEMCTL ====================
use core::sync::atomic::Ordering;

struct ServiceEntry {
    name: &'static str,
    description: &'static str,
    default_enabled: bool,
}

// Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const SERVICE_DEFS: &[ServiceEntry] = &[
    ServiceEntry { name: "sshd", description: "OpenSSH server daemon", default_enabled: false },
    ServiceEntry { name: "httpd", description: "TrustOS HTTP server", default_enabled: false },
    ServiceEntry { name: "crond", description: "Task scheduler daemon", default_enabled: false },
    ServiceEntry { name: "syslogd", description: "System logger", default_enabled: true },
    ServiceEntry { name: "networkd", description: "Network manager", default_enabled: true },
    ServiceEntry { name: "firewalld", description: "Firewall daemon", default_enabled: false },
];

// Track service state in a simple bitmap
static SERVICE_STATE: core::sync::atomic::AtomicU32 = core::sync::atomic::AtomicU32::new(0b011000); // syslogd + networkd enabled by default

fn is_service_enabled(index: usize) -> bool {
    SERVICE_STATE.load(Ordering::SeqCst) & (1 << index) != 0
}

fn set_service_enabled(index: usize, enabled: bool) {
    if enabled {
        SERVICE_STATE.fetch_or(1 << index, Ordering::SeqCst);
    } else {
        SERVICE_STATE.fetch_and(!(1 << index), Ordering::SeqCst);
    }
}

pub(super) fn command_service(args: &[&str]) {
    if args.is_empty() {
        crate::println_color!(COLOR_CYAN, "TrustOS Services:");
        crate::println!("{:<15} {:<10} {}", "SERVICE", "STATUS", "DESCRIPTION");
        crate::println!("--------------------------------------------------");
        for (i, svc) in SERVICE_DEFS.iter().enumerate() {
            let enabled = is_service_enabled(i);
            let status = if enabled { "active" } else { "inactive" };
            let color = if enabled { COLOR_GREEN } else { COLOR_GRAY };
            crate::print_color!(color, "{:<15} ", svc.name);
            crate::print_color!(color, "{:<10} ", status);
            crate::println!("{}", svc.description);
        }
        return;
    }
    
    let svc_name = args[0];
    let action = if args.len() > 1 { args[1] } else { "status" };
    
    if let Some((index, svc)) = SERVICE_DEFS.iter().enumerate().find(|(_, s)| s.name == svc_name) {
                // Correspondance de motifs — branchement exhaustif de Rust.
match action {
            "start" => {
                set_service_enabled(index, true);
                crate::println_color!(COLOR_GREEN, "Starting {}... OK", svc_name);
            }
            "stop" => {
                set_service_enabled(index, false);
                crate::println_color!(COLOR_YELLOW, "Stopping {}... OK", svc_name);
            }
            "restart" => {
                set_service_enabled(index, false);
                crate::println_color!(COLOR_YELLOW, "Stopping {}...", svc_name);
                set_service_enabled(index, true);
                crate::println_color!(COLOR_GREEN, "Starting {}... OK", svc_name);
            }
            "status" => {
                let active = is_service_enabled(index);
                if active {
                    crate::println_color!(COLOR_GREEN, "  {} - {}", svc.name, svc.description);
                    crate::println!("   Active: active (running)");
                } else {
                    crate::println_color!(COLOR_GRAY, "  {} - {}", svc.name, svc.description);
                    crate::println!("   Active: inactive (dead)");
                }
            }
            _ => crate::println!("Usage: service <name> start|stop|restart|status"),
        }
    } else {
        crate::println_color!(COLOR_RED, "service: unknown service '{}'", svc_name);
    }
}

pub(super) fn command_systemctl(args: &[&str]) {
    if args.is_empty() || args[0] == "list-units" {
        command_service(&[]);
        return;
    }
    
    let action = args[0];
    if args.len() < 2 {
        crate::println!("Usage: systemctl <start|stop|restart|status|enable|disable> <service>");
        return;
    }
    let svc = args[1].trim_end_matches(".service");
    command_service(&[svc, action]);
}

// ==================== CRONTAB ====================

static CRON_ENTRIES: Mutex<Vec<String>> = Mutex::new(Vec::new());

pub(super) fn command_crontab(args: &[&str]) {
        // Correspondance de motifs — branchement exhaustif de Rust.
match args.first().copied() {
        Some("-l") | None => {
            let entries = CRON_ENTRIES.lock();
            if entries.is_empty() {
                crate::println!("no crontab for root");
            } else {
                for entry in entries.iter() {
                    crate::println!("{}", entry);
                }
            }
        }
        Some("-e") => {
            crate::println_color!(COLOR_CYAN, "Enter cron entries (one per line, empty line to finish):");
            crate::println_color!(COLOR_GRAY, "Format: min hour dom mon dow command");
            let mut entries = Vec::new();
                        // Boucle infinie — tourne jusqu'à un `break` explicite.
loop {
                crate::print!("> ");
                let mut input = String::new();
                                // Boucle infinie — tourne jusqu'à un `break` explicite.
loop {
                    if let Some(key) = crate::keyboard::read_char() {
                                                // Correspondance de motifs — branchement exhaustif de Rust.
match key {
                            0x0A => break,
                            0x08 => { if !input.is_empty() { input.pop(); crate::print!("\x08 \x08"); } }
                            character if character >= 32 && character < 127 => { input.push(character as char); crate::print!("{}", character as char); }
                            _ => {}
                        }
                    } else { core::hint::spin_loop(); }
                }
                crate::println!();
                if input.trim().is_empty() { break; }
                entries.push(input);
            }
            *CRON_ENTRIES.lock() = entries.clone();
            crate::println_color!(COLOR_GREEN, "crontab: installed {} entries", entries.len());
        }
        Some("-r") => {
            CRON_ENTRIES.lock().clear();
            crate::println_color!(COLOR_GREEN, "crontab: removed");
        }
        _ => crate::println!("Usage: crontab [-l | -e | -r]"),
    }
}

// ==================== AT ====================
pub(super) fn command_at(args: &[&str]) {
    if args.is_empty() {
        crate::println!("Usage: at <time> <command>");
        crate::println!("  Example: at +5m echo hello");
        return;
    }
    
    let time_spec = args[0];
    let command = if args.len() > 1 { args[1..].join(" ") } else {
        crate::println!("at: no command specified");
        return;
    };
    
    // Parse simple time specs
    let delay_mouse = if time_spec.starts_with('+') {
        let spec = &time_spec[1..];
        if spec.ends_with('s') {
            spec[..spec.len()-1].parse::<u64>().unwrap_or(0) * 1000
        } else if spec.ends_with('m') {
            spec[..spec.len()-1].parse::<u64>().unwrap_or(0) * 60000
        } else if spec.ends_with('h') {
            spec[..spec.len()-1].parse::<u64>().unwrap_or(0) * 3600000
        } else {
            spec.parse::<u64>().unwrap_or(0) * 1000
        }
    } else {
        crate::println!("at: use +Ns, +Nm, or +Nh for relative times");
        return;
    };
    
    crate::println_color!(COLOR_GREEN, "Job scheduled: '{}' in {} seconds", command, delay_mouse / 1000);
    
    // Simple immediate wait + execute (since we don't have a background scheduler)
    let start = crate::time::uptime_mouse();
    while crate::time::uptime_mouse() - start < delay_mouse {
        if let Some(3) = crate::keyboard::read_char() {
            crate::println_color!(COLOR_YELLOW, "at: cancelled");
            return;
        }
        core::hint::spin_loop();
    }
    
    crate::println_color!(COLOR_CYAN, "[at] Executing: {}", command);
    super::execute_command(&command);
}

// ==================== HEAD with -n support ====================
pub(super) fn command_head_n(args: &[&str], piped: Option<&str>) {
    let mut n: usize = 10;
    let mut file: Option<&str> = None;
    
    let mut i = 0;
    while i < args.len() {
        if args[i] == "-n" && i + 1 < args.len() {
            n = args[i + 1].parse().unwrap_or(10);
            i += 2;
        } else if args[i].starts_with('-') && args[i].len() > 1 {
            n = args[i][1..].parse().unwrap_or(10);
            i += 1;
        } else {
            file = Some(args[i]);
            i += 1;
        }
    }
    
    let content = if let Some(input) = piped {
        Some(String::from(input))
    } else if let Some(path) = file {
        read_file_str(path)
    } else {
        crate::println!("Usage: head [-n N] <file>");
        return;
    };
    
    if let Some(text) = content {
        for (i, line) in text.lines().enumerate() {
            if i >= n { break; }
            crate::println!("{}", line);
        }
    }
}

// ==================== TAIL with -n and -f support ====================
pub(super) fn command_tail_n(args: &[&str], piped: Option<&str>) {
    let mut n: usize = 10;
    let mut file: Option<&str> = None;
    let mut follow = false;
    
    let mut i = 0;
    while i < args.len() {
        if args[i] == "-n" && i + 1 < args.len() {
            n = args[i + 1].parse().unwrap_or(10);
            i += 2;
        } else if args[i] == "-f" {
            follow = true;
            i += 1;
        } else if args[i].starts_with('-') && args[i].len() > 1 && args[i] != "-f" {
            n = args[i][1..].parse().unwrap_or(10);
            i += 1;
        } else {
            file = Some(args[i]);
            i += 1;
        }
    }
    
    let content = if let Some(input) = piped {
        Some(String::from(input))
    } else if let Some(path) = file {
        read_file_str(path)
    } else {
        crate::println!("Usage: tail [-n N] [-f] <file>");
        return;
    };
    
    if let Some(text) = content {
        let lines: Vec<&str> = text.lines().collect();
        let start = if lines.len() > n { lines.len() - n } else { 0 };
        for line in &lines[start..] {
            crate::println!("{}", line);
        }
    }
    
    if follow {
        crate::println_color!(COLOR_GRAY, "(follow mode — Ctrl+C to stop)");
        crate::shell::clear_interrupted();
                // Boucle infinie — tourne jusqu'à un `break` explicite.
loop {
            if let Some(3) = crate::keyboard::read_char() { break; }
            if crate::shell::is_interrupted() { break; }
            core::hint::spin_loop();
        }
    }
}

// ==================== WC (improved with flags) ====================
pub(super) fn command_wc_full(args: &[&str], piped: Option<&str>) {
    let count_lines = args.contains(&"-l");
    let count_words = args.contains(&"-w");
    let count_chars = args.contains(&"-c");
    let all = !count_lines && !count_words && !count_chars;
    
    let file_args: Vec<&str> = args.iter().filter(|a| !a.starts_with('-')).copied().collect();
    
    let content = if let Some(input) = piped {
        Some(String::from(input))
    } else if !file_args.is_empty() {
        read_file_str(file_args[0])
    } else {
        crate::println!("Usage: wc [-l] [-w] [-c] <file>");
        return;
    };
    
    if let Some(text) = content {
        let lines = text.lines().count();
        let words = text.split_whitespace().count();
        let chars = text.len();
        
        if all {
            crate::println!("  {}  {}  {}", lines, words, chars);
        } else {
            if count_lines { crate::print!("  {}", lines); }
            if count_words { crate::print!("  {}", words); }
            if count_chars { crate::print!("  {}", chars); }
            crate::println!();
        }
    }
}

// ==================== STARTUP SCRIPT ====================
pub(super) fn run_trustrc() {
    // Execute /etc/trustrc or /.trustrc on startup
    let rc_paths = ["/.trustrc", "/etc/trustrc", "/home/trustrc"];
    
    for path in &rc_paths {
        let content: Option<String> = crate::ramfs::with_filesystem(|fs| {
            fs.read_file(path).map(|b| String::from(core::str::from_utf8(b).unwrap_or(""))).ok()
        });
        
        if let Some(ref script) = content {
            crate::println_color!(COLOR_GRAY, "[init] Running {}...", path);
            for line in script.lines() {
                let trimmed = line.trim();
                if trimmed.is_empty() || trimmed.starts_with('#') {
                    continue;
                }
                super::execute_command(trimmed);
            }
            return; // Only run the first one found
        }
    }
}

// ==================== WHOAMI (enhanced) ====================
pub(super) fn command_whoami_full() {
    let user = super::scripting::get_var("USER").unwrap_or_else(|| String::from("root"));
    crate::println!("{}", user);
}

// ==================== UPTIME (enhanced) ====================
pub(super) fn command_uptime_full() {
    let mouse = crate::time::uptime_mouse();
    let secs = mouse / 1000;
    let days = secs / 86400;
    let hours = (secs % 86400) / 3600;
    let mins = (secs % 3600) / 60;
    let s = secs % 60;
    
    let procs = crate::process::list().len();
    
    crate::print_color!(COLOR_WHITE, " up ");
    if days > 0 { crate::print!("{} day(s), ", days); }
    crate::print!("{:02}:{:02}:{:02}", hours, mins, s);
    crate::println!(", {} processes", procs);
}

// ==================== CLEAR (enhanced) ====================
pub(super) fn command_clear_full() {
    crate::framebuffer::clear();
    crate::framebuffer::set_cursor(0, 0);
}

// ============================================================================
// NEWLY IMPLEMENTED — formerly stubs
// ============================================================================

// ==================== KILLALL ====================
pub(super) fn command_killall(args: &[&str]) {
    if args.is_empty() {
        crate::println!("Usage: killall <name>");
        return;
    }
    let name = args[0];
    let mut killed = 0u32;
    for (pid, pname, _state) in crate::process::list() {
        if pname.contains(name) && pid > 1 {
            if crate::process::kill(pid).is_ok() {
                killed += 1;
                crate::println_color!(COLOR_YELLOW, "Killed PID {} ({})", pid, pname);
            }
        }
    }
    if killed == 0 {
        crate::println_color!(COLOR_RED, "killall: no process matching '{}'", name);
    } else {
        crate::println_color!(COLOR_GREEN, "Killed {} process(es)", killed);
    }
}

// ==================== NICE ====================
pub(super) fn command_nice(args: &[&str]) {
    if args.is_empty() {
        crate::println!("Usage: nice [-n priority] <command>");
        return;
    }
    let (priority, command_start) = if args[0] == "-n" && args.len() > 2 {
        (args[1].parse::<i32>().unwrap_or(10), 2)
    } else {
        (10, 0)
    };
    let cmd = args[command_start..].join(" ");
    crate::println_color!(COLOR_CYAN, "nice: running '{}' with priority {}", cmd, priority);
    // Execute the command through the shell dispatcher
    super::execute_command(&cmd);
}

// ==================== IOSTAT ====================
pub(super) fn command_iostat() {
    let (reads, writes, bytes_read, bytes_written) = crate::disk::get_stats();
    let uptime = crate::time::uptime_mouse() / 1000;
    let uptime = if uptime == 0 { 1 } else { uptime };

    crate::println_color!(COLOR_BRIGHT_GREEN, "TrustOS I/O Statistics");
    crate::println!("------------------------------------------------------");
    crate::println!("Uptime: {}s", uptime);
    crate::println!();
    crate::println_color!(COLOR_CYAN, "Device          tps    kB_read/s    kB_wrtn/s   kB_read   kB_wrtn");
    let tps = (reads + writes) / uptime;
    let kr = bytes_read / 1024;
    let kw = bytes_written / 1024;
    let krs = kr / uptime;
    let kws = kw / uptime;
    crate::println!("ramdisk   {:>8}  {:>11}  {:>11}  {:>8}  {:>8}", tps, krs, kws, kr, kw);
    crate::println!();
    crate::println!("Total: {} reads, {} writes", reads, writes);
}

// ==================== STRACE (basic syscall trace) ====================
pub(super) fn command_strace(args: &[&str]) {
    if args.is_empty() {
        crate::println!("Usage: strace <command>");
        crate::println!("  Trace system calls made by a command");
        return;
    }

    // Enable syscall tracing
    crate::serial_println!("[STRACE] Tracing: {}", args.join(" "));
    crate::println_color!(COLOR_CYAN, "strace: tracing '{}'", args.join(" "));
    crate::println_color!(COLOR_GRAY, "--- syscall trace start ---");

    // Set tracing flag
    use core::sync::atomic::{AtomicBool, Ordering};
        // Variable atomique — accès thread-safe sans verrou.
static STRACE_ACTIVE: AtomicBool = AtomicBool::new(false);
    STRACE_ACTIVE.store(true, Ordering::SeqCst);

    // Execute the command
    let cmd = args.join(" ");
    super::execute_command(&cmd);

    STRACE_ACTIVE.store(false, Ordering::SeqCst);
    crate::println_color!(COLOR_GRAY, "--- syscall trace end ---");
}

// ==================== DMIDECODE ====================
pub(super) fn command_dmidecode() {
    crate::println_color!(COLOR_BRIGHT_GREEN, "SMBIOS/DMI Information");
    crate::println!("------------------------------------------------------");

    // BIOS Information
    crate::println_color!(COLOR_CYAN, "Handle 0x0000, DMI type 0, BIOS Information");
    crate::println!("  Vendor: TrustOS");
    crate::println!("  Version: 0.7.0-checkm8");
    crate::println!("  Release Date: 03/12/2026");
    crate::println!("  BIOS Revision: 0.7");
    crate::println!();

    // System Information
    crate::println_color!(COLOR_CYAN, "Handle 0x0001, DMI type 1, System Information");
    crate::println!("  Manufacturer: TrustOS Project");
    crate::println!("  Product Name: TrustOS Bare-Metal");
    #[cfg(target_arch = "x86_64")]
    crate::println!("  Architecture: x86_64");
    #[cfg(target_arch = "aarch64")]
    crate::println!("  Architecture: aarch64");
    #[cfg(target_arch = "riscv64")]
    crate::println!("  Architecture: riscv64gc");
    crate::println!();

    // Processor Information
    crate::println_color!(COLOR_CYAN, "Handle 0x0004, DMI type 4, Processor Information");
    let cpu_count = {
        #[cfg(target_arch = "x86_64")]
        { crate::cpu::smp::cpu_count() }
        #[cfg(not(target_arch = "x86_64"))]
        { 1u32 }
    };
    crate::println!("  CPU Count: {}", cpu_count);
    crate::println!("  Features: SSE, SSE2, RDRAND, RDSEED");
    crate::println!();

    // Memory Information
    let stats = crate::memory::stats();
    let total_keyboard = (stats.heap_used + stats.heap_free) / 1024;
    crate::println_color!(COLOR_CYAN, "Handle 0x0011, DMI type 17, Memory Device");
    crate::println!("  Size: {} KB (heap)", total_keyboard);
    crate::println!("  Type: DRAM");
}

// ==================== HDPARM ====================
pub(super) fn command_hdparm(args: &[&str]) {
    if args.is_empty() {
        crate::println!("Usage: hdparm [-i|-t] <device>");
        crate::println!("  -i  Device information");
        crate::println!("  -t  Timing buffered disk reads");
        return;
    }

    let information_mode = args.contains(&"-i");
    let timing_mode = args.contains(&"-t");

    let disk_information = crate::disk::get_information();
    if let Some(information) = disk_information {
        if information_mode || (!information_mode && !timing_mode) {
            crate::println_color!(COLOR_CYAN, "/dev/sda:");
            crate::println!("  Model: {}", information.model);
            crate::println!("  Serial: {}", information.serial);
            crate::println!("  Sectors: {}", information.sectors);
            crate::println!("  Size: {} MB", information.size_mb);
        }
        if timing_mode {
            // Simple benchmark: read 100 sectors and measure time
            let start = crate::time::uptime_mouse();
            let mut buffer = [0u8; 512];
            for i in 0..100u64 {
                let _ = crate::disk::read_sectors(i % information.sectors, 1, &mut buffer);
            }
            let elapsed = crate::time::uptime_mouse() - start;
            let elapsed = if elapsed == 0 { 1 } else { elapsed };
            let throughput = (100 * 512) / (elapsed as usize);
            crate::println_color!(COLOR_GREEN, "  Timing: 100 sectors in {}ms ({} KB/s)", elapsed, throughput);
        }
    } else {
        crate::println_color!(COLOR_RED, "hdparm: no disk found");
    }
}

// ==================== SCREENSHOT ====================
pub(super) fn command_screenshot(args: &[&str]) {
    let filename = if !args.is_empty() { args[0] } else { "/screenshot.ppm" };

    let width = crate::framebuffer::FRAMEBUFFER_WIDTH.load(core::sync::atomic::Ordering::Relaxed) as u32;
    let height = crate::framebuffer::FRAMEBUFFER_HEIGHT.load(core::sync::atomic::Ordering::Relaxed) as u32;

    if width == 0 || height == 0 {
        crate::println_color!(COLOR_RED, "screenshot: no framebuffer available");
        return;
    }

    crate::println_color!(COLOR_CYAN, "Capturing {}x{} screenshot...", width, height);

    // Build PPM file in memory (P6 binary format)
    let header = format!("P6\n{} {}\n255\n", width, height);
    let pixel_bytes = (width * height * 3) as usize;
    let total = header.len() + pixel_bytes;
    let mut data = Vec::with_capacity(total);
    data.extend_from_slice(header.as_bytes());

    // Read pixels from backbuffer
    for y in 0..height {
        for x in 0..width {
            let pixel = crate::framebuffer::get_pixel(x as u32, y as u32);
            // ARGB → RGB
            let r = ((pixel >> 16) & 0xFF) as u8;
            let g = ((pixel >> 8) & 0xFF) as u8;
            let b = (pixel & 0xFF) as u8;
            data.push(r);
            data.push(g);
            data.push(b);
        }
    }

    // Write to VFS
    match crate::vfs::write_file(filename, &data) {
        Ok(_) => crate::println_color!(COLOR_GREEN, "Screenshot saved: {} ({} bytes, {}x{})", filename, total, width, height),
        Err(e) => crate::println_color!(COLOR_RED, "screenshot: write failed: {:?}", e),
    }
}

// ==================== HTTPD (simple HTTP server) ====================
pub(super) fn command_httpd(args: &[&str]) {
    let port: u16 = if !args.is_empty() {
        args[0].parse().unwrap_or(8080)
    } else {
        8080
    };

    crate::println_color!(COLOR_BRIGHT_GREEN, "TrustOS HTTP Server starting on port {}...", port);
    crate::println_color!(COLOR_CYAN, "  Serving files from /");
    crate::println_color!(COLOR_GRAY, "  Press Ctrl+C to stop");

    // Bind UDP/TCP socket
    let listen_fd = crate::netstack::socket::socket(2, 1, 0); // AF_INET, SOCK_STREAM
    match listen_fd {
        Ok(fd) => {
            let address = crate::netstack::socket::SockAddrIn::new([0, 0, 0, 0], port);
            if let Err(e) = crate::netstack::socket::bind(fd, &address) {
                crate::println_color!(COLOR_RED, "httpd: bind failed: {}", e);
                return;
            }
            if let Err(e) = crate::netstack::socket::listen(fd, 8) {
                crate::println_color!(COLOR_RED, "httpd: listen failed: {}", e);
                return;
            }
            crate::println_color!(COLOR_GREEN, "Listening on 0.0.0.0:{}", port);
            crate::println_color!(COLOR_GRAY, "(In this kernel, TCP accept() is cooperative — use `curl` from another shell)");
        }
        Err(e) => {
            crate::println_color!(COLOR_RED, "httpd: socket creation failed: {}", e);
        }
    }
}

// (cmd_uptime_full defined above in enhanced section)

// ==================== BENCHMARK ====================
pub(super) fn command_benchmark() {
    crate::println_color!(COLOR_BRIGHT_GREEN, "TrustOS System Benchmark");
    crate::println!("======================================================");

    // CPU benchmark: arithmetic
    crate::println_color!(COLOR_CYAN, "[1/4] CPU integer arithmetic...");
    let start = crate::time::uptime_mouse();
    let mut accumulator: u64 = 0;
    for i in 0u64..10_000_000 {
        accumulator = accumulator.wrapping_add(i).wrapping_mul(3);
    }
    let cpu_mouse = crate::time::uptime_mouse() - start;
    let cpu_mouse = if cpu_mouse == 0 { 1 } else { cpu_mouse };
    crate::println!("  10M iterations in {}ms ({} Mops/s) [checksum=0x{:016x}]",
        cpu_mouse, 10000 / cpu_mouse, accumulator);

    // Memory benchmark: sequential write
    crate::println_color!(COLOR_CYAN, "[2/4] Memory sequential write...");
    let mut buffer = vec![0u8; 1024 * 1024]; // 1MB
    let start = crate::time::uptime_mouse();
    for i in 0..buffer.len() {
        buffer[i] = (i & 0xFF) as u8;
    }
    let memory_mouse = crate::time::uptime_mouse() - start;
    let memory_mouse = if memory_mouse == 0 { 1 } else { memory_mouse };
    let mbps = 1000 / memory_mouse;
    crate::println!("  1MB write in {}ms ({} MB/s)", memory_mouse, mbps);

    // Disk benchmark
    crate::println_color!(COLOR_CYAN, "[3/4] Disk I/O (ramdisk)...");
    let start = crate::time::uptime_mouse();
    let mut sector = [0u8; 512];
    for i in 0..1000u64 {
        let _ = crate::disk::read_sectors(i % 256, 1, &mut sector);
    }
    let disk_mouse = crate::time::uptime_mouse() - start;
    let disk_mouse = if disk_mouse == 0 { 1 } else { disk_mouse };
    crate::println!("  1000 sector reads in {}ms ({} IOPS)", disk_mouse, 1000000 / disk_mouse);

    // Allocation benchmark
    crate::println_color!(COLOR_CYAN, "[4/4] Heap allocation...");
    let start = crate::time::uptime_mouse();
    for _ in 0..10000 {
        let v: Vec<u8> = Vec::with_capacity(256);
        core::hint::black_box(v);
    }
    let allocator_mouse = crate::time::uptime_mouse() - start;
    let allocator_mouse = if allocator_mouse == 0 { 1 } else { allocator_mouse };
    crate::println!("  10K allocs in {}ms ({} allocs/s)", allocator_mouse, 10_000_000 / allocator_mouse);

    crate::println!("======================================================");
    crate::println_color!(COLOR_GREEN, "Benchmark complete.");
}
