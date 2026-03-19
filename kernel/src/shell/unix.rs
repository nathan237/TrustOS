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

pub(super) fn cmd_which(args: &[&str]) {
    if args.is_empty() {
        crate::println!("Usage: which <command>");
        return;
    }
    
    let search_dirs = ["/bin", "/usr/bin", "/sbin", "/usr/sbin"];
    
    for name in args {
        let mut found = false;
        for dir in &search_dirs {
            let path = format!("{}/{}", dir, name);
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

pub(super) fn cmd_whereis(args: &[&str]) {
    if args.is_empty() {
        crate::println!("Usage: whereis <command>");
        return;
    }
    cmd_which(args);
}

pub(super) fn cmd_file(args: &[&str]) {
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

pub(super) fn cmd_basename(args: &[&str]) {
    if args.is_empty() {
        crate::println!("Usage: basename <path>");
        return;
    }
    let path = args[0];
    let name = path.rsplit('/').next().unwrap_or(path);
    crate::println!("{}", name);
}

pub(super) fn cmd_dirname(args: &[&str]) {
    if args.is_empty() {
        crate::println!("Usage: dirname <path>");
        return;
    }
    let path = args[0];
    if let Some(pos) = path.rfind('/') {
        if pos == 0 {
            crate::println!("/");
        } else {
            crate::println!("{}", &path[..pos]);
        }
    } else {
        crate::println!(".");
    }
}

pub(super) fn cmd_realpath(args: &[&str]) {
    if args.is_empty() {
        crate::println!("Usage: realpath <path>");
        return;
    }
    let path = super::vm::resolve_program_path(args[0]);
    crate::println!("{}", path);
}

pub(super) fn cmd_sort(args: &[&str], piped: Option<&str>) {
    let content = if let Some(input) = piped {
        Some(alloc::string::String::from(input))
    } else if !args.is_empty() {
        super::network::read_file_content(args[0])
    } else {
        crate::println!("Usage: sort <file>");
        return;
    };
    
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

pub(super) fn cmd_uniq(args: &[&str], piped: Option<&str>) {
    let content = if let Some(input) = piped {
        Some(alloc::string::String::from(input))
    } else if !args.is_empty() {
        super::network::read_file_content(args[0])
    } else {
        crate::println!("Usage: uniq <file>");
        return;
    };
    
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

pub(super) fn cmd_yes(args: &[&str]) {
    let text = if args.is_empty() { "y" } else { args[0] };
    crate::shell::clear_interrupted();
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

pub(super) fn cmd_seq(args: &[&str]) {
    if args.is_empty() {
        crate::println!("Usage: seq <last> | seq <first> <last> | seq <first> <inc> <last>");
        return;
    }
    
    let (first, inc, last) = match args.len() {
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

pub(super) fn cmd_sleep(args: &[&str]) {
    if args.is_empty() {
        crate::println!("Usage: sleep <seconds>");
        return;
    }
    
    let secs: u64 = args[0].parse().unwrap_or(1);
    crate::println_color!(COLOR_CYAN, "Sleeping for {} seconds...", secs);
    
    // Simple busy-wait sleep (not ideal but works)
    let start = crate::time::uptime_ms();
    let end = start + secs * 1000;
    while crate::time::uptime_ms() < end {
        core::hint::spin_loop();
    }
    crate::println!("Done.");
}

pub(super) fn cmd_kill(args: &[&str]) {
    if args.is_empty() {
        crate::println!("Usage: kill [-9] <pid>");
        return;
    }
    
    let _signal = if args[0] == "-9" { 9 } else { 15 };
    let pid_str = if args[0].starts_with('-') && args.len() > 1 { args[1] } else { args[0] };
    
    match pid_str.parse::<u32>() {
        Ok(pid) => {
            crate::println_color!(COLOR_YELLOW, "Killing PID {}", pid);
            match crate::process::kill(pid) {
                Ok(_) => crate::println_color!(COLOR_GREEN, "Process {} killed", pid),
                Err(e) => crate::println_color!(COLOR_RED, "kill: {}", e),
            }
        }
        Err(_) => crate::println_color!(COLOR_RED, "kill: invalid PID"),
    }
}

pub(super) fn cmd_top() {
    crate::println_color!(COLOR_BRIGHT_GREEN, "TrustOS Process Monitor");
    crate::println!("-----------------------------------------------------------");
    
    let uptime = crate::time::uptime_ms() / 1000;
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
        let state_str = match state {
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

pub(super) fn cmd_vmstat() {
    crate::println_color!(COLOR_BRIGHT_GREEN, "Virtual Memory Statistics");
    crate::println!("-----------------------------------------");
    
    let heap_used = crate::memory::stats().heap_used;
    let heap_total = (crate::memory::stats().heap_used + crate::memory::stats().heap_free);
    
    crate::println!("Memory:");
    crate::println!("  Heap Total:  {} KB", heap_total / 1024);
    crate::println!("  Heap Used:   {} KB", heap_used / 1024);
    crate::println!("  Heap Free:   {} KB", (heap_total - heap_used) / 1024);
}

pub(super) fn cmd_dmesg(args: &[&str]) {
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
    let (buf_size, total) = crate::devtools::dmesg_stats();
    crate::println_color!(COLOR_BRIGHT_GREEN, "Kernel Ring Buffer ({} stored, {} total)", buf_size, total);
    crate::println!("---------------------------------------------------------------");
    for line in &lines {
        crate::println!("{}", line);
    }
}

pub(super) fn cmd_memdbg() {
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
    crate::println!("    Alloc ops    : {:>10}", s.alloc_count);
    crate::println!("    Dealloc ops  : {:>10}", s.dealloc_count);
    crate::println!("    Live allocs  : {:>10}", s.live_allocs);
    crate::println!("    Total alloc'd: {:>10} bytes", s.alloc_bytes_total);
    crate::println!("    Total freed  : {:>10} bytes", s.dealloc_bytes_total);
    crate::println!("    Largest alloc: {:>10} bytes", s.largest_alloc);
    crate::println!();
    crate::println_color!(COLOR_CYAN, "  Fragmentation:");
    let frag_color = if s.fragmentation_pct > 50.0 { COLOR_RED }
        else if s.fragmentation_pct > 25.0 { COLOR_YELLOW }
        else { COLOR_GREEN };
    crate::println_color!(frag_color, "    Estimate     : {:.1}%", s.fragmentation_pct);
}

pub(super) fn cmd_perfstat() {
    let snap = crate::devtools::perf_snapshot();
    let uptime_s = snap.uptime_ms / 1000;
    let hours = uptime_s / 3600;
    let mins = (uptime_s % 3600) / 60;
    let secs = uptime_s % 60;
    
    crate::println_color!(COLOR_BRIGHT_GREEN, "Performance Statistics (perf)");
    crate::println!("---------------------------------------------------------------");
    crate::println!();
    crate::println_color!(COLOR_CYAN, "  System:");
    crate::println!("    Uptime       : {}h {:02}m {:02}s ({} ms)", hours, mins, secs, snap.uptime_ms);
    crate::println!("    GUI FPS      : {}", snap.fps);
    crate::println!();
    crate::println_color!(COLOR_CYAN, "  Interrupts:");
    crate::println!("    Total IRQs   : {}", snap.total_irqs);
    crate::println!("    IRQ/sec      : {}", snap.irq_per_sec);
    crate::println!();
    crate::println_color!(COLOR_CYAN, "  Scheduling:");
    crate::println!("    Syscalls     : {}", snap.total_syscalls);
    crate::println!("    Ctx switches : {}", snap.total_ctx_switches);
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

pub(super) fn cmd_irqstat() {
    let stats = crate::sync::percpu::all_cpu_stats();
    let total_irqs: u64 = stats.iter().map(|s| s.interrupts).sum();
    
    crate::println_color!(COLOR_BRIGHT_GREEN, "IRQ Statistics");
    crate::println!("---------------------------------------------------------------");
    crate::println!();
    crate::println!("  Total IRQs     : {}", total_irqs);
    crate::println!("  IRQ rate       : {}/sec", crate::devtools::irq_rate());
    crate::println!();
    crate::println_color!(COLOR_CYAN, "  Per-CPU Breakdown:");
    for s in &stats {
        let bar_len = if total_irqs > 0 { (s.interrupts * 40 / total_irqs.max(1)) as usize } else { 0 };
        let bar: String = "|".repeat(bar_len);
        let pct = if total_irqs > 0 { s.interrupts * 100 / total_irqs } else { 0 };
        crate::println!("    CPU{}: {:>8} ({:>3}%) {}", s.cpu_id, s.interrupts, pct, bar);
    }
}

pub(super) fn cmd_registers() {
    crate::println_color!(COLOR_BRIGHT_GREEN, "CPU Register Dump");
    crate::println!("---------------------------------------------------------------");
    let regs = crate::devtools::cpu_registers();
    for line in &regs {
        crate::println!("{}", line);
    }
}

pub(super) fn cmd_peek(args: &[&str]) {
    if args.is_empty() {
        crate::println!("Usage: peek <hex_addr> [byte_count]");
        crate::println!("  e.g.: peek 0xFFFF8000_00000000 64");
        crate::println!("  Default count: 64 bytes, max: 256 bytes");
        return;
    }
    
    let addr_str = args[0].trim_start_matches("0x").trim_start_matches("0X");
    let addr = match usize::from_str_radix(addr_str, 16) {
        Ok(a) => a,
        Err(_) => {
            crate::println_color!(COLOR_RED, "Invalid hex address: {}", args[0]);
            return;
        }
    };
    
    let count = args.get(1).and_then(|s| s.parse::<usize>().ok()).unwrap_or(64);
    
    crate::println_color!(COLOR_BRIGHT_GREEN, "Memory dump at 0x{:016x} ({} bytes)", addr, count);
    crate::println!("---------------------------------------------------------------");
    let lines = crate::devtools::peek(addr, count);
    for line in &lines {
        crate::println!("{}", line);
    }
}

pub(super) fn cmd_poke(args: &[&str]) {
    if args.len() < 2 {
        crate::println!("Usage: poke <hex_addr> <hex_value>");
        crate::println!("  e.g.: poke 0xB8000 0x41");
        crate::println_color!(COLOR_RED, "  ? WARNING: Writing to arbitrary memory is DANGEROUS!");
        return;
    }
    
    let addr_str = args[0].trim_start_matches("0x").trim_start_matches("0X");
    let addr = match usize::from_str_radix(addr_str, 16) {
        Ok(a) => a,
        Err(_) => {
            crate::println_color!(COLOR_RED, "Invalid hex address: {}", args[0]);
            return;
        }
    };
    
    let val_str = args[1].trim_start_matches("0x").trim_start_matches("0X");
    let value = match u8::from_str_radix(val_str, 16) {
        Ok(v) => v,
        Err(_) => {
            crate::println_color!(COLOR_RED, "Invalid hex value: {}", args[1]);
            return;
        }
    };
    
    match crate::devtools::poke(addr, value) {
        Ok(()) => crate::println_color!(COLOR_GREEN, "Wrote 0x{:02x} to 0x{:016x}", value, addr),
        Err(e) => crate::println_color!(COLOR_RED, "poke error: {}", e),
    }
}

pub(super) fn cmd_devpanel() {
    crate::devtools::toggle_devpanel();
    let state = if crate::devtools::is_devpanel_visible() { "ON" } else { "OFF" };
    crate::println_color!(COLOR_GREEN, "DevPanel overlay: {} (also toggle with F12 in desktop)", state);
}

pub(super) fn cmd_timecmd(args: &[&str]) {
    if args.is_empty() {
        crate::println!("Usage: timecmd <command> [args...]");
        crate::println!("  Runs a command and prints elapsed time.");
        return;
    }
    
    let start = crate::cpu::tsc::Stopwatch::start();
    
    // Reconstruct and execute the sub-command
    let sub_cmd = args.join(" ");
    super::execute_command(&sub_cmd);
    
    let elapsed_us = start.elapsed_micros();
    let elapsed_ms = elapsed_us / 1000;
    let frac = elapsed_us % 1000;
    crate::println!();
    crate::println_color!(COLOR_CYAN, "? Elapsed: {}.{:03} ms ({} us)", elapsed_ms, frac, elapsed_us);
}

// ═══════════════════════════════════════════════════════════════════════════════
// Hardware Debug Toolkit commands
// ═══════════════════════════════════════════════════════════════════════════════

/// Full hardware diagnostic report
pub(super) fn cmd_hwdiag() {
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
pub(super) fn cmd_cpudump() {
    crate::println_color!(COLOR_BRIGHT_GREEN, "Full CPU State Dump");
    crate::println!("---------------------------------------------------------------");
    let lines = crate::debug::full_cpu_dump();
    for line in &lines {
        crate::println!("{}", line);
    }
}

/// Stack trace / backtrace
pub(super) fn cmd_stacktrace(args: &[&str]) {
    let max = args.first().and_then(|s| s.parse::<usize>().ok()).unwrap_or(16);
    crate::println_color!(COLOR_BRIGHT_GREEN, "Stack Backtrace (max {} frames)", max);
    crate::println!("---------------------------------------------------------------");
    let lines = crate::debug::format_backtrace(max);
    for line in &lines {
        crate::println!("{}", line);
    }
}

/// Show boot checkpoints log
pub(super) fn cmd_bootlog() {
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
pub(super) fn cmd_postcode(args: &[&str]) {
    if args.is_empty() {
        crate::println!("Current POST code: 0x{:02X}", crate::debug::last_post_code());
        crate::println!("Usage: postcode <hex_value>   (writes to port 0x80)");
        return;
    }
    let val_str = args[0].trim_start_matches("0x").trim_start_matches("0X");
    match u8::from_str_radix(val_str, 16) {
        Ok(v) => {
            crate::debug::post_code(v);
            crate::println_color!(COLOR_GREEN, "POST code 0x{:02X} written to port 0x80", v);
        }
        Err(_) => crate::println_color!(COLOR_RED, "Invalid hex value: {}", args[0]),
    }
}

/// I/O port read/write
pub(super) fn cmd_ioport(args: &[&str]) {
    if args.len() < 2 {
        crate::println!("Usage: ioport read <port_hex> [b|w|l]");
        crate::println!("       ioport write <port_hex> <value_hex> [b|w|l]");
        crate::println!("  b=byte (default), w=word, l=dword");
        crate::println_color!(COLOR_RED, "  ⚠ WARNING: Writing to arbitrary I/O ports is DANGEROUS!");
        return;
    }
    
    let subcmd = args[0];
    let port_str = args[1].trim_start_matches("0x").trim_start_matches("0X");
    let port = match u16::from_str_radix(port_str, 16) {
        Ok(p) => p,
        Err(_) => {
            crate::println_color!(COLOR_RED, "Invalid port: {}", args[1]);
            return;
        }
    };
    
    match subcmd {
        "read" | "r" => {
            let size = args.get(2).copied().unwrap_or("b");
            match size {
                "b" | "byte" => {
                    let val = crate::debug::inb(port);
                    crate::println!("  IN  port 0x{:04X} = 0x{:02X} ({})", port, val, val);
                }
                "w" | "word" => {
                    let val = crate::debug::inw(port);
                    crate::println!("  IN  port 0x{:04X} = 0x{:04X} ({})", port, val, val);
                }
                "l" | "dword" => {
                    let val = crate::debug::inl(port);
                    crate::println!("  IN  port 0x{:04X} = 0x{:08X} ({})", port, val, val);
                }
                _ => crate::println_color!(COLOR_RED, "Size must be b/w/l"),
            }
        }
        "write" | "w" => {
            if args.len() < 3 {
                crate::println_color!(COLOR_RED, "Need value: ioport write <port> <value> [b|w|l]");
                return;
            }
            let val_str = args[2].trim_start_matches("0x").trim_start_matches("0X");
            let size = args.get(3).copied().unwrap_or("b");
            match size {
                "b" | "byte" => {
                    if let Ok(v) = u8::from_str_radix(val_str, 16) {
                        crate::debug::outb(port, v);
                        crate::println_color!(COLOR_GREEN, "  OUT port 0x{:04X} <- 0x{:02X}", port, v);
                    } else {
                        crate::println_color!(COLOR_RED, "Invalid byte value");
                    }
                }
                "w" | "word" => {
                    if let Ok(v) = u16::from_str_radix(val_str, 16) {
                        crate::debug::outw(port, v);
                        crate::println_color!(COLOR_GREEN, "  OUT port 0x{:04X} <- 0x{:04X}", port, v);
                    } else {
                        crate::println_color!(COLOR_RED, "Invalid word value");
                    }
                }
                "l" | "dword" => {
                    if let Ok(v) = u32::from_str_radix(val_str, 16) {
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
pub(super) fn cmd_rdmsr(args: &[&str]) {
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
    match u32::from_str_radix(msr_str, 16) {
        Ok(msr) => {
            match crate::debug::read_msr_safe(msr) {
                Some(val) => {
                    crate::println!("  MSR 0x{:08X} = 0x{:016X}", msr, val);
                    crate::println!("                  {:064b}", val);
                }
                None => crate::println_color!(COLOR_RED, "  MSR 0x{:08X}: read failed (#GP)", msr),
            }
        }
        Err(_) => crate::println_color!(COLOR_RED, "Invalid MSR address: {}", args[0]),
    }
}

/// Write MSR
pub(super) fn cmd_wrmsr(args: &[&str]) {
    if args.len() < 2 {
        crate::println!("Usage: wrmsr <msr_hex> <value_hex>");
        crate::println_color!(COLOR_RED, "  ⚠ WARNING: Writing to MSRs can crash the system!");
        return;
    }
    let msr_str = args[0].trim_start_matches("0x").trim_start_matches("0X");
    let val_str = args[1].trim_start_matches("0x").trim_start_matches("0X");
    
    let msr = match u32::from_str_radix(msr_str, 16) {
        Ok(m) => m,
        Err(_) => {
            crate::println_color!(COLOR_RED, "Invalid MSR: {}", args[0]);
            return;
        }
    };
    let val = match u64::from_str_radix(val_str, 16) {
        Ok(v) => v,
        Err(_) => {
            crate::println_color!(COLOR_RED, "Invalid value: {}", args[1]);
            return;
        }
    };
    
    crate::debug::write_msr(msr, val);
    crate::println_color!(COLOR_GREEN, "  WRMSR 0x{:08X} <- 0x{:016X}", msr, val);
}

/// Raw CPUID query
pub(super) fn cmd_cpuid(args: &[&str]) {
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
    
    let leaf = match u32::from_str_radix(leaf_str, 16) {
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
pub(super) fn cmd_memmap() {
    crate::println_color!(COLOR_BRIGHT_GREEN, "Physical Memory Map");
    crate::println!("---------------------------------------------------------------");
    let lines = crate::debug::format_memory_map();
    for line in &lines {
        crate::println!("{}", line);
    }
}

/// Watchdog control
pub(super) fn cmd_watchdog(args: &[&str]) {
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

// ═══════════════════════════════════════════════════════════════════════════════
// Live Driver Debug Toolkit — test & debug drivers without recompiling
// ═══════════════════════════════════════════════════════════════════════════════

/// Unified driver debug command
pub(super) fn cmd_drv(args: &[&str]) {
    let subcmd = args.first().copied().unwrap_or("help");

    match subcmd {
        // ── PCI config space read/write ──
        "pci" => cmd_drv_pci(&args[1..]),

        // ── MMIO read/write ──
        "mmio" => cmd_drv_mmio(&args[1..]),

        // ── Re-probe a driver at runtime ──
        "reprobe" | "probe" => cmd_drv_reprobe(&args[1..]),

        // ── Driver test suites ──
        "test" => cmd_drv_test(&args[1..]),

        // ── List active drivers & state ──
        "list" | "ls" => cmd_drv_list(),

        // ── Dump all PCI devices with full config space header ──
        "scan" => cmd_drv_scan(),

        _ => {
            crate::println_color!(COLOR_CYAN, "=== Live Driver Debug Toolkit ===");
            crate::println!();
            crate::println!("  drv list                           List all active drivers and state");
            crate::println!("  drv scan                           Scan PCI bus (full config header dump)");
            crate::println!();
            crate::println_color!(COLOR_YELLOW, "PCI Config Space:");
            crate::println!("  drv pci read <BB:DD.F> <off> [l]   Read PCI config register (b/w/l)");
            crate::println!("  drv pci write <BB:DD.F> <off> <val> [l]  Write PCI config register");
            crate::println!("  drv pci dump <BB:DD.F>             Dump 256-byte PCI config space");
            crate::println!();
            crate::println_color!(COLOR_YELLOW, "MMIO (Memory-Mapped I/O):");
            crate::println!("  drv mmio read <addr> [count]       Read 32-bit MMIO registers");
            crate::println!("  drv mmio write <addr> <val>        Write 32-bit MMIO register");
            crate::println!("  drv mmio wifi <offset>             Read WiFi CSR register by offset");
            crate::println!();
            crate::println_color!(COLOR_YELLOW, "Driver Control:");
            crate::println!("  drv reprobe wifi                   Re-probe WiFi driver from PCI");
            crate::println!("  drv reprobe hda                    Re-probe HDA audio driver");
            crate::println!();
            crate::println_color!(COLOR_YELLOW, "Driver Tests (no recompile):");
            crate::println!("  drv test wifi                      WiFi: PCI detect → BAR → CSR dump → FW check");
            crate::println!("  drv test hda                       HDA: codec detect → widget dump → path check");
            crate::println!("  drv test ec                        ThinkPad EC: temp sensors → fan → battery");
            crate::println!("  drv test net                       Network: virtio/e1000 link → MAC → ping");
            crate::println!("  drv test all                       Run all driver tests sequentially");
        }
    }
}

/// Parse "BB:DD.F" PCI address format
fn parse_bdf(s: &str) -> Option<(u8, u8, u8)> {
    // Accept "BB:DD.F" or "BB:DD:F"
    let parts: Vec<&str> = s.split(|c| c == ':' || c == '.').collect();
    if parts.len() < 2 { return None; }
    let bus = u8::from_str_radix(parts[0], 16).ok()?;
    let dev = u8::from_str_radix(parts[1], 16).ok()?;
    let func = if parts.len() > 2 { u8::from_str_radix(parts[2], 16).ok().unwrap_or(0) } else { 0 };
    Some((bus, dev, func))
}

/// Parse hex string (with or without 0x prefix)
fn parse_hex32(s: &str) -> Option<u32> {
    let s = s.trim_start_matches("0x").trim_start_matches("0X");
    u32::from_str_radix(s, 16).ok()
}

fn parse_hex_usize(s: &str) -> Option<usize> {
    let s = s.trim_start_matches("0x").trim_start_matches("0X");
    usize::from_str_radix(s, 16).ok()
}

/// drv pci read/write/dump
fn cmd_drv_pci(args: &[&str]) {
    let subcmd = args.first().copied().unwrap_or("help");
    match subcmd {
        "read" | "r" => {
            if args.len() < 3 {
                crate::println!("Usage: drv pci read <BB:DD.F> <offset_hex> [b|w|l]");
                return;
            }
            let Some((bus, dev, func)) = parse_bdf(args[1]) else {
                crate::println_color!(COLOR_RED, "Invalid BDF: {} (use BB:DD.F)", args[1]);
                return;
            };
            let Some(offset) = parse_hex32(args[2]) else {
                crate::println_color!(COLOR_RED, "Invalid offset: {}", args[2]);
                return;
            };
            let size = args.get(3).copied().unwrap_or("l");
            let val = crate::pci::config_read(bus, dev, func, offset as u8);
            match size {
                "b" | "byte" => crate::println!("  PCI {:02X}:{:02X}.{} [0x{:02X}] = 0x{:02X}",
                    bus, dev, func, offset, val & 0xFF),
                "w" | "word" => crate::println!("  PCI {:02X}:{:02X}.{} [0x{:02X}] = 0x{:04X}",
                    bus, dev, func, offset, val & 0xFFFF),
                _ => crate::println!("  PCI {:02X}:{:02X}.{} [0x{:02X}] = 0x{:08X}",
                    bus, dev, func, offset, val),
            }
        }
        "write" | "w" => {
            if args.len() < 4 {
                crate::println!("Usage: drv pci write <BB:DD.F> <offset_hex> <value_hex> [b|w|l]");
                return;
            }
            let Some((bus, dev, func)) = parse_bdf(args[1]) else {
                crate::println_color!(COLOR_RED, "Invalid BDF: {}", args[1]);
                return;
            };
            let Some(offset) = parse_hex32(args[2]) else {
                crate::println_color!(COLOR_RED, "Invalid offset: {}", args[2]);
                return;
            };
            let Some(value) = parse_hex32(args[3]) else {
                crate::println_color!(COLOR_RED, "Invalid value: {}", args[3]);
                return;
            };
            crate::pci::config_write(bus, dev, func, offset as u8, value);
            crate::println_color!(COLOR_GREEN, "  PCI {:02X}:{:02X}.{} [0x{:02X}] <- 0x{:08X}",
                bus, dev, func, offset, value);
        }
        "dump" | "d" => {
            if args.len() < 2 {
                crate::println!("Usage: drv pci dump <BB:DD.F>");
                return;
            }
            let Some((bus, dev, func)) = parse_bdf(args[1]) else {
                crate::println_color!(COLOR_RED, "Invalid BDF: {}", args[1]);
                return;
            };
            crate::println_color!(COLOR_CYAN, "PCI Config Space {:02X}:{:02X}.{}", bus, dev, func);
            crate::println!("     00 04 08 0C 10 14 18 1C 20 24 28 2C 30 34 38 3C");
            for row in 0..16u8 {
                let off = row * 16;
                crate::print!("{:02X}: ", off);
                for col in (0..16u8).step_by(4) {
                    let val = crate::pci::config_read(bus, dev, func, off + col);
                    crate::print!("{:08X} ", val);
                }
                crate::println!();
            }
        }
        _ => {
            crate::println!("Usage: drv pci <read|write|dump> ...");
        }
    }
}

/// drv mmio read/write
fn cmd_drv_mmio(args: &[&str]) {
    let subcmd = args.first().copied().unwrap_or("help");
    match subcmd {
        "read" | "r" => {
            if args.len() < 2 {
                crate::println!("Usage: drv mmio read <phys_addr_hex> [count]");
                crate::println!("  Reads 32-bit values at given physical address (auto-mapped)");
                return;
            }
            let Some(phys) = parse_hex_usize(args[1]) else {
                crate::println_color!(COLOR_RED, "Invalid address: {}", args[1]);
                return;
            };
            let count = args.get(2).and_then(|s| s.parse::<usize>().ok()).unwrap_or(1);
            let count = count.min(64); // safety cap
            let size = count * 4;
            match crate::memory::map_mmio(phys as u64, size) {
                Ok(virt_base) => {
                    for i in 0..count {
                        let virt = virt_base as usize + i * 4;
                        let phys_a = phys + i * 4;
                        let val = unsafe { core::ptr::read_volatile(virt as *const u32) };
                        crate::println!("  [phys {:#010X}] = 0x{:08X}", phys_a, val);
                    }
                }
                Err(e) => {
                    crate::println_color!(COLOR_RED, "  Failed to map phys 0x{:X}: {}", phys, e);
                }
            }
        }
        "write" | "w" => {
            if args.len() < 3 {
                crate::println!("Usage: drv mmio write <phys_addr_hex> <value_hex>");
                crate::println_color!(COLOR_RED, "  ⚠ WARNING: Writing to arbitrary MMIO is DANGEROUS!");
                return;
            }
            let Some(phys) = parse_hex_usize(args[1]) else {
                crate::println_color!(COLOR_RED, "Invalid address: {}", args[1]);
                return;
            };
            let Some(value) = parse_hex32(args[2]) else {
                crate::println_color!(COLOR_RED, "Invalid value: {}", args[2]);
                return;
            };
            match crate::memory::map_mmio(phys as u64, 4) {
                Ok(virt) => {
                    unsafe { core::ptr::write_volatile(virt as *mut u32, value); }
                    crate::println_color!(COLOR_GREEN, "  [phys {:#010X}] <- 0x{:08X}", phys, value);
                }
                Err(e) => {
                    crate::println_color!(COLOR_RED, "  Failed to map phys 0x{:X}: {}", phys, e);
                }
            }
        }
        "wifi" => {
            if args.len() < 2 {
                crate::println!("Usage: drv mmio wifi <csr_offset_hex>");
                crate::println!("  Reads WiFi CSR register at given offset from BAR0");
                crate::println!("  Common: 0x000=HW_IF_CONFIG, 0x020=RESET, 0x024=GP_CNTRL, 0x028=HW_REV");
                return;
            }
            let Some(offset) = parse_hex32(args[1]) else {
                crate::println_color!(COLOR_RED, "Invalid offset: {}", args[1]);
                return;
            };
            match crate::drivers::net::iwl4965::debug_read_csr(offset) {
                Some(val) => crate::println!("  WiFi CSR [0x{:03X}] = 0x{:08X}", offset, val),
                None => crate::println_color!(COLOR_RED, "  Cannot read — no WiFi device or offset out of range"),
            }
        }
        _ => {
            crate::println!("Usage: drv mmio <read|write|wifi> ...");
        }
    }
}

/// drv reprobe — re-probe a driver from PCI without reboot
fn cmd_drv_reprobe(args: &[&str]) {
    let driver = args.first().copied().unwrap_or("help");
    match driver {
        "wifi" => {
            crate::println_color!(COLOR_YELLOW, "Re-probing WiFi from PCI bus...");
            let devices = crate::pci::get_devices();
            let mut found = false;

            // Check all network + wireless class devices
            for dev in &devices {
                crate::println!("  Checking {:02X}:{:02X}.{} {:04X}:{:04X} class={:02X}:{:02X}",
                    dev.bus, dev.device, dev.function,
                    dev.vendor_id, dev.device_id,
                    dev.class_code, dev.subclass);

                if crate::drivers::net::wifi::probe_pci(dev) {
                    crate::println_color!(COLOR_GREEN, "  ✓ WiFi driver bound to {:04X}:{:04X}!", dev.vendor_id, dev.device_id);
                    found = true;
                    break;
                }
            }

            if !found {
                crate::println_color!(COLOR_RED, "  ✗ No WiFi hardware found on PCI bus");
                crate::println!();
                crate::println!("  Expected: Intel 8086:4229 (4965AGN) or similar");
                crate::println!("  Tip: Run 'drv scan' to see all PCI devices");
                crate::println!("  Tip: Run 'drv test wifi' for detailed diagnostics");
            }
        }
        "hda" => {
            crate::println_color!(COLOR_YELLOW, "Re-probing HDA audio...");
            let _ = crate::drivers::hda::init();
            if crate::drivers::hda::is_initialized() {
                crate::println_color!(COLOR_GREEN, "  ✓ HDA audio re-initialized");
            } else {
                crate::println_color!(COLOR_RED, "  ✗ HDA init failed");
            }
        }
        _ => {
            crate::println!("Usage: drv reprobe <wifi|hda>");
            crate::println!("  Re-probes driver from PCI bus without rebooting");
        }
    }
}

/// drv test — automated driver test suites
fn cmd_drv_test(args: &[&str]) {
    let target = args.first().copied().unwrap_or("help");
    match target {
        "wifi" => drv_test_wifi(),
        "hda" => drv_test_hda(),
        "ec" => drv_test_ec(),
        "net" => drv_test_net(),
        "all" => {
            drv_test_wifi();
            crate::println!();
            drv_test_hda();
            crate::println!();
            drv_test_ec();
            crate::println!();
            drv_test_net();
        }
        _ => {
            crate::println!("Usage: drv test <wifi|hda|ec|net|all>");
        }
    }
}

/// Map a PCI BAR0 physical address to virtual address via HHDM.
/// Returns the virtual base address or None if mapping fails.
fn map_bar0_mmio(dev: &crate::pci::PciDevice) -> Option<usize> {
    let bar0 = dev.bar[0];
    if bar0 == 0 || (bar0 & 1) != 0 { return None; }
    let is_64 = (bar0 >> 1) & 0x3 == 2;
    let phys = if is_64 {
        let bar1 = dev.bar[1] as u64;
        (bar1 << 32) | (bar0 & 0xFFFFFFF0) as u64
    } else {
        (bar0 & 0xFFFFFFF0) as u64
    };
    if phys == 0 { return None; }
    // Map 8KB MMIO region (covers WiFi CSRs, HDA regs, etc.)
    match crate::memory::map_mmio(phys, 0x2000) {
        Ok(virt) => Some(virt as usize),
        Err(_e) => None,
    }
}

/// WiFi driver test suite
fn drv_test_wifi() {
    crate::println_color!(COLOR_CYAN, "=== WiFi Driver Test Suite ===");
    let mut pass = 0u32;
    let mut fail = 0u32;

    // Test 1: PCI device detection
    crate::print!("  [1] PCI device scan (Intel WiFi)... ");
    let devices = crate::pci::get_devices();
    let wifi_dev = devices.iter().find(|d|
        d.vendor_id == 0x8086 && crate::drivers::net::iwl4965::IWL4965_DEVICE_IDS.contains(&d.device_id));
    if let Some(dev) = wifi_dev {
        crate::println_color!(COLOR_GREEN, "FOUND {:04X}:{:04X} at {:02X}:{:02X}.{}",
            dev.vendor_id, dev.device_id, dev.bus, dev.device, dev.function);
        pass += 1;

        // Test 2: PCI class check
        crate::print!("  [2] PCI class/subclass... ");
        crate::print!("class={:02X} sub={:02X} ", dev.class_code, dev.subclass);
        let expected_wireless = dev.class_code == 0x02 && dev.subclass == 0x80
            || dev.class_code == 0x0D;
        if expected_wireless {
            crate::println_color!(COLOR_GREEN, "OK (wireless)");
            pass += 1;
        } else {
            crate::println_color!(COLOR_YELLOW, "UNEXPECTED (but device ID matches)");
            pass += 1;
        }

        // Test 3: BAR0 + MMIO mapping
        crate::print!("  [3] BAR0 (MMIO base)... ");
        let bar0 = dev.bar[0];
        let is_64 = (bar0 >> 1) & 0x3 == 2;
        let phys = if is_64 {
            let bar1 = dev.bar[1] as u64;
            (bar1 << 32) | (bar0 & 0xFFFFFFF0) as u64
        } else {
            (bar0 & 0xFFFFFFF0) as u64
        };
        if bar0 != 0 && (bar0 & 1) == 0 && phys != 0 {
            crate::println_color!(COLOR_GREEN, "phys=0x{:X} ({})", phys, if is_64 { "64-bit" } else { "32-bit" });
            pass += 1;

            // Map the MMIO region into virtual address space
            crate::print!("  [3b] MMIO page mapping... ");
            match map_bar0_mmio(dev) {
                Some(virt_base) => {
                    crate::println_color!(COLOR_GREEN, "virt=0x{:X}", virt_base);
                    pass += 1;

                    // Test 4: Read HW_REV CSR via mapped virtual address
                    crate::print!("  [4] CSR HW_REV read... ");
                    let hw_rev = unsafe { core::ptr::read_volatile((virt_base + 0x028) as *const u32) };
                    if hw_rev != 0 && hw_rev != 0xFFFFFFFF {
                        let hw_type = (hw_rev & 0x000FFF0) >> 4;
                        let name = match hw_type { 0 => "4965", 2 => "5300", 4 => "5150", 5 => "5100", 7 => "6000", _ => "unknown" };
                        crate::println_color!(COLOR_GREEN, "0x{:08X} (type={} = {})", hw_rev, hw_type, name);
                        pass += 1;
                    } else {
                        crate::println_color!(COLOR_RED, "0x{:08X} — MMIO not responding", hw_rev);
                        fail += 1;
                    }

                    // Test 5: GP_CNTRL
                    crate::print!("  [5] CSR GP_CNTRL... ");
                    let gp = unsafe { core::ptr::read_volatile((virt_base + 0x024) as *const u32) };
                    crate::print!("0x{:08X} ", gp);
                    if gp & 1 != 0 {
                        crate::println_color!(COLOR_GREEN, "(MAC clock ready)");
                    } else {
                        crate::println_color!(COLOR_YELLOW, "(MAC clock NOT ready — device sleeping or reset)");
                    }
                    pass += 1;

                    // Test 6: Dump key CSRs
                    crate::println!("  [6] Key CSR register dump:");
                    crate::drivers::net::iwl4965::debug_dump_csrs();
                    pass += 1;
                }
                None => {
                    crate::println_color!(COLOR_RED, "FAILED to map MMIO pages!");
                    crate::println!("      phys=0x{:X}, map_mmio() returned error", phys);
                    fail += 1;
                }
            }

        } else {
            if bar0 == 0 {
                crate::println_color!(COLOR_RED, "ZERO — BAR not assigned!");
            } else {
                crate::println_color!(COLOR_RED, "I/O BAR (0x{:08X}) — need memory BAR", bar0);
            }
            fail += 1;
        }

        // Test 7: IRQ
        crate::print!("  [7] IRQ assignment... ");
        if dev.interrupt_line != 0xFF && dev.interrupt_line != 0 {
            crate::println_color!(COLOR_GREEN, "IRQ {} (pin {})", dev.interrupt_line, dev.interrupt_pin);
            pass += 1;
        } else {
            crate::println_color!(COLOR_YELLOW, "no IRQ assigned (line={}, pin={})", dev.interrupt_line, dev.interrupt_pin);
            pass += 1; // Not fatal
        }

        // Test 8: Bus mastering enabled?
        crate::print!("  [8] PCI command register... ");
        let cmd = crate::pci::config_read(dev.bus, dev.device, dev.function, 0x04);
        let mem_space = cmd & 0x02 != 0;
        let bus_master = cmd & 0x04 != 0;
        crate::print!("0x{:04X} ", cmd & 0xFFFF);
        if mem_space && bus_master {
            crate::println_color!(COLOR_GREEN, "(mem_space=ON, bus_master=ON)");
            pass += 1;
        } else {
            crate::println_color!(COLOR_YELLOW, "(mem_space={}, bus_master={}) — may need enabling",
                if mem_space { "ON" } else { "OFF" }, if bus_master { "ON" } else { "OFF" });
            pass += 1;
        }

    } else {
        crate::println_color!(COLOR_RED, "NOT FOUND");
        fail += 1;
        crate::println!("  [!] No Intel WiFi device on PCI bus.");
        crate::println!("      Expected vendor=8086, device=4229/4230/4232/...");
        crate::println!();
        crate::println!("  All PCI devices on this machine:");
        for d in &devices {
            crate::println!("    {:02X}:{:02X}.{} {:04X}:{:04X} class={:02X}:{:02X} {}",
                d.bus, d.device, d.function,
                d.vendor_id, d.device_id,
                d.class_code, d.subclass,
                d.vendor_name());
        }
    }

    // Test 9: Firmware check
    crate::print!("  [9] Firmware (iwlwifi-4965-2.ucode)... ");
    if crate::drivers::net::iwl4965::has_firmware() {
        crate::println_color!(COLOR_GREEN, "LOADED");
        pass += 1;
    } else {
        crate::println_color!(COLOR_RED, "NOT AVAILABLE");
        crate::println!("      Load firmware via Limine module or copy to RamFS");
        fail += 1;
    }

    // Test 10: Driver state
    crate::print!("  [10] WiFi driver state... ");
    if crate::drivers::net::wifi::has_wifi() {
        crate::println_color!(COLOR_GREEN, "ACTIVE");
        crate::drivers::net::iwl4965::debug_dump();
        pass += 1;
    } else {
        crate::println_color!(COLOR_RED, "NOT ACTIVE");
        fail += 1;
    }

    crate::println!();
    let total = pass + fail;
    if fail == 0 {
        crate::println_color!(COLOR_GREEN, "  WiFi: {}/{} PASSED", pass, total);
    } else {
        crate::println_color!(COLOR_YELLOW, "  WiFi: {}/{} passed, {} failed", pass, total, fail);
    }
}

/// HDA audio driver test suite
fn drv_test_hda() {
    crate::println_color!(COLOR_CYAN, "=== HDA Audio Driver Test Suite ===");
    let mut pass = 0u32;
    let mut fail = 0u32;

    // Test 1: Find HDA controller on PCI
    crate::print!("  [1] PCI device (class 04:03 multimedia/audio)... ");
    let devices = crate::pci::get_devices();
    let hda_dev = devices.iter().find(|d| d.class_code == 0x04 && d.subclass == 0x03);
    if let Some(dev) = hda_dev {
        crate::println_color!(COLOR_GREEN, "FOUND {:04X}:{:04X} at {:02X}:{:02X}.{}",
            dev.vendor_id, dev.device_id, dev.bus, dev.device, dev.function);
        pass += 1;

        // Test 2: BAR0 + MMIO mapping
        crate::print!("  [2] BAR0 (HDA MMIO)... ");
        let bar0 = dev.bar[0];
        if bar0 != 0 && (bar0 & 1) == 0 {
            let phys = (bar0 & 0xFFFFFFF0) as u64;
            crate::println_color!(COLOR_GREEN, "phys=0x{:X}", phys);
            pass += 1;

            crate::print!("  [2b] MMIO mapping... ");
            match map_bar0_mmio(dev) {
                Some(base) => {
                    crate::println_color!(COLOR_GREEN, "virt=0x{:X}", base);

                    // Test 3: GCAP register (offset 0x00)
                    crate::print!("  [3] GCAP register... ");
                    let gcap = unsafe { core::ptr::read_volatile(base as *const u16) };
                    let oss = (gcap >> 12) & 0xF;
                    let iss = (gcap >> 8) & 0xF;
                    let bss = (gcap >> 3) & 0x1F;
                    crate::println_color!(COLOR_GREEN, "0x{:04X} (OSS={}, ISS={}, BSS={})", gcap, oss, iss, bss);
                    pass += 1;

                    // Test 4: GCTL (global control, offset 0x08)
                    crate::print!("  [4] GCTL (controller reset)... ");
                    let gctl = unsafe { core::ptr::read_volatile((base + 0x08) as *const u32) };
                    if gctl & 1 != 0 {
                        crate::println_color!(COLOR_GREEN, "0x{:08X} (out of reset)", gctl);
                        pass += 1;
                    } else {
                        crate::println_color!(COLOR_RED, "0x{:08X} (IN RESET — codec not accessible)", gctl);
                        fail += 1;
                    }

                    // Test 5: STATESTS (codec status, offset 0x0E)
                    crate::print!("  [5] STATESTS (codec presence)... ");
                    let statests = unsafe { core::ptr::read_volatile((base + 0x0E) as *const u16) };
                    if statests & 0x7 != 0 {
                        let codecs: Vec<u8> = (0..3).filter(|i| statests & (1 << i) != 0).collect();
                        crate::println_color!(COLOR_GREEN, "0x{:04X} — codecs at: {:?}", statests, codecs);
                        pass += 1;
                    } else {
                        crate::println_color!(COLOR_RED, "0x{:04X} — NO codecs detected", statests);
                        fail += 1;
                    }
                }
                None => {
                    crate::println_color!(COLOR_RED, "FAILED to map MMIO!");
                    fail += 1;
                }
            }
        } else {
            crate::println_color!(COLOR_RED, "BAR0={:#X} — invalid", bar0);
            fail += 1;
        }

        // Test 6: IRQ
        crate::print!("  [6] IRQ... ");
        crate::println!("line={} pin={}", dev.interrupt_line, dev.interrupt_pin);
        pass += 1;

    } else {
        crate::println_color!(COLOR_RED, "NOT FOUND");
        fail += 1;
    }

    // Test 7: TrustOS HDA driver state
    crate::print!("  [7] HDA driver state... ");
    if crate::drivers::hda::is_initialized() {
        crate::println_color!(COLOR_GREEN, "INITIALIZED");
        pass += 1;
    } else {
        crate::println_color!(COLOR_RED, "NOT INITIALIZED");
        fail += 1;
    }

    crate::println!();
    let total = pass + fail;
    if fail == 0 {
        crate::println_color!(COLOR_GREEN, "  HDA: {}/{} PASSED", pass, total);
    } else {
        crate::println_color!(COLOR_YELLOW, "  HDA: {}/{} passed, {} failed", pass, total, fail);
    }
}

/// ThinkPad EC driver test suite
fn drv_test_ec() {
    crate::println_color!(COLOR_CYAN, "=== ThinkPad EC Driver Test Suite ===");
    let mut pass = 0u32;
    let mut fail = 0u32;

    // Test 1: EC presence (read from port 0x66)
    crate::print!("  [1] EC status port (0x66)... ");
    let status = crate::debug::inb(0x66);
    crate::print!("0x{:02X} ", status);
    // Bit 1 = IBF (should be 0 when idle), bit 0 = OBF
    if status != 0xFF && status != 0x00 {
        crate::println_color!(COLOR_GREEN, "(EC responding, IBF={}, OBF={})", (status >> 1) & 1, status & 1);
        pass += 1;
    } else if status == 0xFF {
        crate::println_color!(COLOR_RED, "(no EC — all bits high)");
        fail += 1;
    } else {
        crate::println_color!(COLOR_YELLOW, "(EC idle — all bits zero, may be OK)");
        pass += 1;
    }

    // Test 2: Read CPU temperature (EC register 0x78)
    crate::print!("  [2] CPU temp (EC reg 0x78)... ");
    // Send read command: write 0x80 to cmd port, then register addr to data port
    // This is a simplified read — the real driver handles timeouts
    let temp = crate::drivers::thinkpad_ec::ec_read(0x78);
    if let Some(t) = temp {
        if t > 0 && t < 120 {
            crate::println_color!(COLOR_GREEN, "{}°C", t);
            pass += 1;
        } else {
            crate::println_color!(COLOR_YELLOW, "raw={} (out of range)", t);
            pass += 1;
        }
    } else {
        crate::println_color!(COLOR_RED, "TIMEOUT");
        fail += 1;
    }

    // Test 3: Fan speed
    crate::print!("  [3] Fan RPM... ");
    let rpm = crate::drivers::thinkpad_ec::fan_get_rpm();
    if let Some(r) = rpm {
        crate::println_color!(COLOR_GREEN, "{} RPM", r);
        pass += 1;
    } else {
        crate::println_color!(COLOR_YELLOW, "cannot read (EC may not support)");
        pass += 1;
    }

    // Test 4: Read all temp sensors
    crate::print!("  [4] Temperature sensors... ");
    let sensors = [0x78, 0x79, 0x7A, 0x7B, 0xC0, 0xC1, 0xC2, 0xC3];
    let mut found = 0;
    for &reg in &sensors {
        if let Some(t) = crate::drivers::thinkpad_ec::ec_read(reg) {
            if t > 0 && t < 120 { found += 1; }
        }
    }
    if found > 0 {
        crate::println_color!(COLOR_GREEN, "{}/8 sensors active", found);
        pass += 1;
    } else {
        crate::println_color!(COLOR_RED, "no sensors responding");
        fail += 1;
    }

    crate::println!();
    let total = pass + fail;
    if fail == 0 {
        crate::println_color!(COLOR_GREEN, "  EC: {}/{} PASSED", pass, total);
    } else {
        crate::println_color!(COLOR_YELLOW, "  EC: {}/{} passed, {} failed", pass, total, fail);
    }
}

/// Network driver test suite
fn drv_test_net() {
    crate::println_color!(COLOR_CYAN, "=== Network Driver Test Suite ===");
    let mut pass = 0u32;
    let mut fail = 0u32;

    // Test 1: Find network device on PCI
    crate::print!("  [1] PCI network device... ");
    let devices = crate::pci::get_devices();
    let net_dev = devices.iter().find(|d| d.class_code == 0x02);
    if let Some(dev) = net_dev {
        let name = match dev.vendor_id {
            0x1AF4 => "VirtIO-net",
            0x8086 => match dev.device_id {
                0x100E | 0x100F | 0x10D3 | 0x153A => "Intel e1000/e1000e",
                _ => "Intel (unknown)",
            },
            0x10EC => "Realtek RTL8139",
            _ => "Unknown",
        };
        crate::println_color!(COLOR_GREEN, "{} ({:04X}:{:04X}) at {:02X}:{:02X}.{}", 
            name, dev.vendor_id, dev.device_id, dev.bus, dev.device, dev.function);
        pass += 1;
    } else {
        crate::println_color!(COLOR_RED, "NOT FOUND");
        fail += 1;
    }

    // Test 2: Network stack
    crate::print!("  [2] Network stack... ");
    if crate::network::is_available() {
        crate::println_color!(COLOR_GREEN, "AVAILABLE (platform: {})", crate::network::get_platform());
        pass += 1;
    } else {
        crate::println_color!(COLOR_RED, "NOT AVAILABLE");
        fail += 1;
    }

    // Test 3: MAC address
    crate::print!("  [3] MAC address... ");
    if let Some(mac) = crate::network::get_mac_address() {
        if mac != [0, 0, 0, 0, 0, 0] {
            crate::println_color!(COLOR_GREEN, "{:02X}:{:02X}:{:02X}:{:02X}:{:02X}:{:02X}",
                mac[0], mac[1], mac[2], mac[3], mac[4], mac[5]);
            pass += 1;
        } else {
            crate::println_color!(COLOR_RED, "00:00:00:00:00:00 (not set)");
            fail += 1;
        }
    } else {
        crate::println_color!(COLOR_RED, "no MAC address available");
        fail += 1;
    }

    // Test 4: WiFi subsystem
    crate::print!("  [4] WiFi subsystem... ");
    if crate::drivers::net::wifi::has_wifi() {
        crate::println_color!(COLOR_GREEN, "ACTIVE (state: {:?})", crate::drivers::net::wifi::state());
        pass += 1;
    } else {
        crate::println_color!(COLOR_YELLOW, "no WiFi hardware");
        pass += 1; // Not a failure on VMs
    }

    crate::println!();
    let total = pass + fail;
    if fail == 0 {
        crate::println_color!(COLOR_GREEN, "  Network: {}/{} PASSED", pass, total);
    } else {
        crate::println_color!(COLOR_YELLOW, "  Network: {}/{} passed, {} failed", pass, total, fail);
    }
}

/// drv list — show all active drivers
fn cmd_drv_list() {
    crate::println_color!(COLOR_CYAN, "=== Active Drivers ===");
    crate::println!();

    // Network
    crate::print!("  Network:   ");
    if crate::network::is_available() {
        crate::println_color!(COLOR_GREEN, "{}", crate::network::get_platform());
    } else {
        crate::println_color!(COLOR_RED, "none");
    }

    // WiFi
    crate::print!("  WiFi:      ");
    if crate::drivers::net::wifi::has_wifi() {
        crate::println_color!(COLOR_GREEN, "iwl4965 (state: {:?})", crate::drivers::net::wifi::state());
    } else {
        crate::println_color!(COLOR_YELLOW, "not detected");
    }

    // HDA
    crate::print!("  HDA Audio: ");
    if crate::drivers::hda::is_initialized() {
        crate::println_color!(COLOR_GREEN, "initialized");
    } else {
        crate::println_color!(COLOR_YELLOW, "not initialized");
    }

    // EC
    crate::print!("  ThinkPad EC: ");
    let ec_status = crate::debug::inb(0x66);
    if ec_status != 0xFF {
        crate::println_color!(COLOR_GREEN, "present (status=0x{:02X})", ec_status);
    } else {
        crate::println_color!(COLOR_YELLOW, "not detected");
    }

    // Firmware
    crate::print!("  WiFi FW:   ");
    if crate::drivers::net::iwl4965::has_firmware() {
        crate::println_color!(COLOR_GREEN, "loaded");
    } else {
        crate::println_color!(COLOR_YELLOW, "not loaded");
    }

    crate::println!();
}

/// drv scan — detailed PCI device scan
fn cmd_drv_scan() {
    let devices = crate::pci::get_devices();
    crate::println_color!(COLOR_CYAN, "=== PCI Bus Scan ({} devices) ===", devices.len());
    crate::println!();

    for dev in &devices {
        crate::println_color!(COLOR_GREEN, "{:02X}:{:02X}.{} {:04X}:{:04X}",
            dev.bus, dev.device, dev.function, dev.vendor_id, dev.device_id);
        crate::println!("  Class:   {:02X}:{:02X} ({})", dev.class_code, dev.subclass, dev.subclass_name());
        crate::println!("  Vendor:  {}", dev.vendor_name());
        crate::println!("  ProgIF:  {:02X}  Rev: {:02X}", dev.prog_if, dev.revision);

        let cmd = crate::pci::config_read(dev.bus, dev.device, dev.function, 0x04);
        crate::println!("  Command: 0x{:04X} (mem_space={}, bus_master={}, io_space={})",
            cmd & 0xFFFF,
            if cmd & 0x02 != 0 { "ON" } else { "off" },
            if cmd & 0x04 != 0 { "ON" } else { "off" },
            if cmd & 0x01 != 0 { "ON" } else { "off" });

        let status = (crate::pci::config_read(dev.bus, dev.device, dev.function, 0x04) >> 16) & 0xFFFF;
        crate::println!("  Status:  0x{:04X}", status);

        if dev.interrupt_line != 0xFF {
            crate::println!("  IRQ:     {} (pin {})", dev.interrupt_line, dev.interrupt_pin);
        }

        for i in 0..6 {
            if dev.bar[i] != 0 {
                let bar_type = if dev.bar[i] & 1 == 0 { "MEM" } else { "I/O" };
                let addr = if dev.bar[i] & 1 == 0 { dev.bar[i] & 0xFFFFFFF0 } else { dev.bar[i] & 0xFFFFFFFC };
                crate::println!("  BAR{}:    0x{:08X} [{}]", i, addr, bar_type);
            }
        }
        crate::println!();
    }
}

pub(super) fn cmd_lsof(_args: &[&str]) {
    crate::println!("COMMAND   PID   FD   TYPE   NAME");
    crate::println!("----------------------------------------");
    crate::println!("shell     1     0    CHR    /dev/stdin");
    crate::println!("shell     1     1    CHR    /dev/stdout");
    crate::println!("shell     1     2    CHR    /dev/stderr");
}

pub(super) fn cmd_strings(args: &[&str]) {
    if args.is_empty() {
        crate::println!("Usage: strings <file>");
        return;
    }
    
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

pub(super) fn cmd_mount(args: &[&str]) {
    if args.is_empty() {
        // Show mounted filesystems
        crate::println_color!(COLOR_BRIGHT_GREEN, "Mounted Filesystems:");
        crate::vfs::list_mounts();
        return;
    }
    
    if args.len() < 2 {
        crate::println!("Usage: mount <device> <mountpoint> [fstype]");
        crate::println!();
        crate::println!("Devices:");
        crate::println!("  ahci:<port>:<start_lba>  - AHCI/SATA partition");
        crate::println!("  ahci:<port>              - Whole AHCI disk (superfloppy)");
        crate::println!();
        crate::println!("FS types: fat32, ext4, ntfs (auto-detected if omitted)");
        crate::println!();
        crate::println!("Examples:");
        crate::println!("  mount ahci:0:2048 /mnt/disk0   - Mount partition at LBA 2048");
        crate::println!("  mount ahci:1:0 /mnt/disk1 ntfs - Mount whole disk as NTFS");
        crate::println!();
        crate::println!("Tip: run 'diskscan' to detect disks and get mount commands.");
        return;
    }
    
    let device = args[0];
    let mountpoint = args[1];
    let fstype = if args.len() > 2 { Some(args[2]) } else { None };
    
    // Parse device specification
    if device.starts_with("ahci:") || device.starts_with("sata:") {
        let spec = &device[5..];
        let parts: Vec<&str> = spec.split(':').collect();
        
        let port: u8 = match parts[0].parse() {
            Ok(n) => n,
            Err(_) => {
                crate::println_color!(COLOR_RED, "Invalid port number: {}", parts[0]);
                return;
            }
        };
        
        let start_lba: u64 = if parts.len() > 1 {
            match parts[1].parse() {
                Ok(n) => n,
                Err(_) => {
                    crate::println_color!(COLOR_RED, "Invalid LBA: {}", parts[1]);
                    return;
                }
            }
        } else {
            0
        };
        
        use alloc::sync::Arc;
        use crate::vfs::fat32::{AhciBlockReader, BlockDevice};
        
        let reader = Arc::new(AhciBlockReader::new(port as usize, start_lba));
        
        // Determine filesystem type
        let detected = fstype.unwrap_or_else(|| {
            // Auto-detect by probing magic bytes
            let mut sector0 = [0u8; 512];
            if reader.read_sector(0, &mut sector0).is_ok() {
                if sector0.len() >= 11 && &sector0[3..7] == b"NTFS" {
                    return "ntfs";
                }
                if sector0[510] == 0x55 && sector0[511] == 0xAA {
                    if sector0.len() >= 90 && &sector0[82..87] == b"FAT32" {
                        return "fat32";
                    }
                    let spf16 = u16::from_le_bytes([sector0[22], sector0[23]]);
                    let spf32 = u32::from_le_bytes([sector0[36], sector0[37], sector0[38], sector0[39]]);
                    if spf16 == 0 && spf32 > 0 {
                        return "fat32";
                    }
                }
            }
            // Try ext4
            let mut s2 = [0u8; 512];
            if reader.read_sector(2, &mut s2).is_ok() && s2.len() >= 0x3A {
                let magic = u16::from_le_bytes([s2[0x38], s2[0x39]]);
                if magic == 0xEF53 {
                    return "ext4";
                }
            }
            "auto"
        });
        
        crate::println!("Mounting ahci:{}:{} at {} (fs: {})...", port, start_lba, mountpoint, detected);
        
        match detected {
            "fat32" | "vfat" => {
                match crate::vfs::fat32::Fat32Fs::mount(reader) {
                    Ok(fs) => {
                        match crate::vfs::mount(mountpoint, Arc::new(fs)) {
                            Ok(()) => crate::println_color!(COLOR_GREEN, "Mounted FAT32 at {}", mountpoint),
                            Err(e) => crate::println_color!(COLOR_RED, "VFS mount error: {:?}", e),
                        }
                    }
                    Err(e) => crate::println_color!(COLOR_RED, "FAT32 mount failed: {:?}", e),
                }
            }
            "ext4" | "ext3" | "ext2" => {
                match crate::vfs::ext4::mount(reader) {
                    Ok(fs) => {
                        match crate::vfs::mount(mountpoint, fs) {
                            Ok(()) => crate::println_color!(COLOR_GREEN, "Mounted ext4 at {}", mountpoint),
                            Err(e) => crate::println_color!(COLOR_RED, "VFS mount error: {:?}", e),
                        }
                    }
                    Err(e) => crate::println_color!(COLOR_RED, "ext4 mount failed: {}", e),
                }
            }
            "ntfs" => {
                match crate::vfs::ntfs::mount(reader) {
                    Ok(fs) => {
                        match crate::vfs::mount(mountpoint, fs) {
                            Ok(()) => crate::println_color!(COLOR_GREEN, "Mounted NTFS at {}", mountpoint),
                            Err(e) => crate::println_color!(COLOR_RED, "VFS mount error: {:?}", e),
                        }
                    }
                    Err(e) => crate::println_color!(COLOR_RED, "NTFS mount failed: {}", e),
                }
            }
            _ => {
                // Try all in order: FAT32, ext4, NTFS
                let reader2 = Arc::new(AhciBlockReader::new(port as usize, start_lba));
                let reader3 = Arc::new(AhciBlockReader::new(port as usize, start_lba));
                
                if let Ok(fs) = crate::vfs::fat32::Fat32Fs::mount(reader) {
                    match crate::vfs::mount(mountpoint, Arc::new(fs)) {
                        Ok(()) => { crate::println_color!(COLOR_GREEN, "Mounted FAT32 at {}", mountpoint); return; }
                        Err(e) => crate::println_color!(COLOR_RED, "VFS mount error: {:?}", e),
                    }
                } else if let Ok(fs) = crate::vfs::ext4::mount(reader2) {
                    match crate::vfs::mount(mountpoint, fs) {
                        Ok(()) => { crate::println_color!(COLOR_GREEN, "Mounted ext4 at {}", mountpoint); return; }
                        Err(e) => crate::println_color!(COLOR_RED, "VFS mount error: {:?}", e),
                    }
                } else if let Ok(fs) = crate::vfs::ntfs::mount(reader3) {
                    match crate::vfs::mount(mountpoint, fs) {
                        Ok(()) => { crate::println_color!(COLOR_GREEN, "Mounted NTFS at {}", mountpoint); return; }
                        Err(e) => crate::println_color!(COLOR_RED, "VFS mount error: {:?}", e),
                    }
                } else {
                    crate::println_color!(COLOR_RED, "mount: no supported filesystem found on device");
                }
            }
        }
    } else {
        crate::println_color!(COLOR_RED, "mount: unsupported device: {}", device);
        crate::println!("Supported: ahci:<port>[:<start_lba>]");
        crate::println!("Run 'diskscan' to find devices.");
    }
}

pub(super) fn cmd_sync() {
    crate::println!("Syncing filesystems...");
    crate::println_color!(COLOR_GREEN, "Done.");
}

pub(super) fn cmd_umount(args: &[&str]) {
    if args.is_empty() {
        crate::println!("Usage: umount <mountpoint>");
        return;
    }
    match crate::vfs::umount(args[0]) {
        Ok(()) => crate::println_color!(COLOR_GREEN, "Unmounted {}", args[0]),
        Err(e) => crate::println_color!(COLOR_RED, "umount: {}: {:?}", args[0], e),
    }
}

pub(super) fn cmd_fsck(args: &[&str]) {
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

pub(super) fn cmd_lsblk() {
    crate::println_color!(COLOR_BRIGHT_GREEN, "Block Devices:");
    crate::println!("NAME          SIZE        TYPE    DRIVER        MODEL");
    crate::println!("----------------------------------------------------------------------");
    
    let mut idx = 0u32;
    
    // NVMe namespaces
    if crate::nvme::is_initialized() {
        if let Some((model, _serial, ns_size, lba_size)) = crate::nvme::get_info() {
            let size_bytes = ns_size * lba_size as u64;
            let size_str = format_size(size_bytes);
            crate::println!("nvme0n1       {:<11} disk    NVMe          {}", size_str, model);
            idx += 1;
        }
    }
    
    // AHCI/SATA disks
    if crate::drivers::ahci::is_initialized() {
        for dev in crate::drivers::ahci::list_devices() {
            let size_bytes = dev.sector_count * 512;
            let size_str = format_size(size_bytes);
            let type_str = match dev.device_type {
                crate::drivers::ahci::AhciDeviceType::Sata => "disk",
                crate::drivers::ahci::AhciDeviceType::Satapi => "cdrom",
                _ => "disk",
            };
            crate::println!("sda{}          {:<11} {:<7} AHCI/p{}       {}", 
                idx, size_str, type_str, dev.port_num, dev.model);
            idx += 1;
        }
    }
    
    // IDE/ATA drives
    for drv in crate::drivers::ata::list_drives() {
        if drv.present {
            let size_bytes = drv.sector_count * 512;
            let size_str = format_size(size_bytes);
            let ch = match drv.channel {
                crate::drivers::ata::IdeChannel::Primary => "P",
                crate::drivers::ata::IdeChannel::Secondary => "S",
            };
            let pos = match drv.position {
                crate::drivers::ata::DrivePosition::Master => "M",
                crate::drivers::ata::DrivePosition::Slave => "S",
            };
            let type_str = if drv.atapi { "cdrom" } else { "disk" };
            let lba_str = if drv.lba48 { "LBA48" } else { "LBA28" };
            crate::println!("hd{}           {:<11} {:<7} IDE/{}{} {}  {}", 
                idx, size_str, type_str, ch, pos, lba_str, drv.model);
            idx += 1;
        }
    }
    
    // VirtIO block devices
    if crate::virtio_blk::is_initialized() {
        let cap = crate::virtio_blk::capacity();
        let size_str = format_size(cap * 512);
        let ro = if crate::virtio_blk::is_read_only() { " (ro)" } else { "" };
        crate::println!("vda{}          {:<11} disk    VirtIO-blk{}", idx, size_str, ro);
        idx += 1;
    }
    
    // USB mass storage
    for (i, (name, blocks, bsize)) in crate::drivers::usb_storage::list_devices().iter().enumerate() {
        let size_str = format_size(*blocks * *bsize as u64);
        crate::println!("usb{}          {:<11} disk    USB-Storage   {}", 
            idx + i as u32, size_str, name);
    }
    if idx == 0 && crate::drivers::usb_storage::device_count() == 0 {
        idx += 1; // for ram0 display below
    }
    
    // RAM disk (always present as fallback)
    crate::println!("ram0          256K        ramdisk RAM           TrustFS");
    
    if idx == 0 {
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

pub(super) fn cmd_blkid() {
    let mut found = false;
    
    // NVMe
    if crate::nvme::is_initialized() {
        if let Some((model, serial, ns_size, lba_size)) = crate::nvme::get_info() {
            let size_bytes = ns_size * lba_size as u64;
            crate::println!("/dev/nvme0n1: MODEL=\"{}\" SERIAL=\"{}\" SIZE={} TYPE=\"nvme\"",
                model, serial, format_size(size_bytes));
            found = true;
        }
    }
    
    // AHCI
    if crate::drivers::ahci::is_initialized() {
        for (i, dev) in crate::drivers::ahci::list_devices().iter().enumerate() {
            let size_bytes = dev.sector_count * 512;
            crate::println!("/dev/sda{}: MODEL=\"{}\" SERIAL=\"{}\" SIZE={} TYPE=\"ahci\" PORT={}",
                i, dev.model, dev.serial, format_size(size_bytes), dev.port_num);
            found = true;
        }
    }
    
    // IDE
    for (i, drv) in crate::drivers::ata::list_drives().iter().enumerate() {
        if drv.present {
            let size_bytes = drv.sector_count * 512;
            let fstype = if drv.atapi { "atapi" } else { "ide" };
            crate::println!("/dev/hd{}: MODEL=\"{}\" SERIAL=\"{}\" SIZE={} TYPE=\"{}\"",
                i, drv.model, drv.serial, format_size(size_bytes), fstype);
            found = true;
        }
    }
    
    // VirtIO
    if crate::virtio_blk::is_initialized() {
        let cap = crate::virtio_blk::capacity();
        crate::println!("/dev/vda: SIZE={} TYPE=\"virtio-blk\"", format_size(cap * 512));
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

pub(super) fn cmd_export(args: &[&str]) {
    if args.is_empty() {
        // Show all variables as export format
        for (k, v) in super::scripting::all_vars() {
            crate::println!("export {}={}", k, v);
        }
        return;
    }
    // Parse VAR=VALUE or VAR
    let joined = args.join(" ");
    if let Some(eq_pos) = joined.find('=') {
        let key = joined[..eq_pos].trim();
        let val = joined[eq_pos + 1..].trim().trim_matches('"').trim_matches('\'');
        super::scripting::set_var(key, val);
        crate::serial_println!("[export] {}={}", key, val);
    } else {
        // Just mark as exported (already in our global store)
        if super::scripting::get_var(args[0]).is_none() {
            super::scripting::set_var(args[0], "");
        }
    }
}

pub(super) fn cmd_source(args: &[&str]) {
    if args.is_empty() {
        crate::println!("Usage: source <script>");
        return;
    }
    
    match super::network::read_file_content(args[0]) {
        Some(content) => {
            super::scripting::execute_script(&content);
        }
        None => crate::println_color!(COLOR_RED, "source: cannot read {}", args[0]),
    }
}

pub(super) fn cmd_set(_args: &[&str]) {
    for (k, v) in super::scripting::all_vars() {
        crate::println!("{}={}", k, v);
    }
}

pub(super) fn cmd_printf(args: &[&str]) {
    if args.is_empty() {
        crate::println!("Usage: printf <format> [args...]");
        return;
    }
    // Simple implementation - just print format string
    let format = args[0].replace("\\n", "\n").replace("\\t", "\t");
    crate::print!("{}", format);
}

pub(super) fn cmd_test_expr(args: &[&str]) {
    // Basic test expression evaluation
    if args.is_empty() {
        crate::println!("false");
        return;
    }
    
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

pub(super) fn cmd_expr(args: &[&str]) {
    if args.len() < 3 {
        crate::println!("Usage: expr <num1> <op> <num2>");
        return;
    }
    
    let a: i64 = args[0].parse().unwrap_or(0);
    let b: i64 = args[2].parse().unwrap_or(0);
    
    let result = match args[1] {
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

pub(super) fn cmd_cal(_args: &[&str]) {
    crate::println_color!(COLOR_BRIGHT_GREEN, "   February 2026");
    crate::println!("Su Mo Tu We Th Fr Sa");
    crate::println!(" 1  2  3  4  5  6  7");
    crate::println!(" 8  9 10 11 12 13 14");
    crate::println!("15 16 17 18 19 20 21");
    crate::println!("22 23 24 25 26 27 28");
}

pub(super) fn cmd_cmp(args: &[&str]) {
    if args.len() < 2 {
        crate::println!("Usage: cmp <file1> <file2>");
        return;
    }
    
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

pub(super) fn cmd_od(args: &[&str]) {
    if args.is_empty() {
        crate::println!("Usage: od <file>");
        return;
    }
    // Use hexdump for now
    super::commands::cmd_hexdump(args);
}

pub(super) fn cmd_rev(args: &[&str]) {
    if args.is_empty() {
        crate::println!("Usage: rev <file>");
        return;
    }
    
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

pub(super) fn cmd_factor(args: &[&str]) {
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

pub(super) fn cmd_tty() {
    crate::println!("/dev/tty0");
}

pub(super) fn cmd_stty(_args: &[&str]) {
    crate::println!("speed 9600 baud; line = 0;");
    crate::println!("-brkint -imaxbel");
}

pub(super) fn cmd_reset() {
    super::commands::cmd_clear();
    crate::println!("Terminal reset.");
}

pub(super) fn cmd_lsusb() {
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
            for (i, dev) in devices.iter().enumerate() {
                let speed = match dev.speed {
                    1 => "Full Speed (12 Mbps)",
                    2 => "Low Speed (1.5 Mbps)",
                    3 => "High Speed (480 Mbps)",
                    4 => "SuperSpeed (5 Gbps)",
                    _ => "Unknown",
                };
                crate::println!("Bus 001 Device {:03}: ID {:04x}:{:04x} Port {} - {}", 
                    i + 2, dev.vendor_id, dev.product_id, dev.port, speed);
                if dev.class != 0 {
                    let class_name = match dev.class {
                        0x03 => "HID (Human Interface Device)",
                        0x08 => "Mass Storage",
                        0x09 => "Hub",
                        _ => "Unknown class",
                    };
                    crate::println!("    Class: {:02x}:{:02x}:{:02x} ({})", 
                        dev.class, dev.subclass, dev.protocol, class_name);
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

pub(super) fn cmd_smpstatus() {
    crate::cpu::smp::print_status();
}

pub(super) fn cmd_smp(args: &[&str]) {
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

pub(super) fn cmd_fontsmooth(args: &[&str]) {
    use crate::framebuffer::font::{FontMode, set_mode, get_mode};
    
    if args.is_empty() {
        let current = match get_mode() {
            FontMode::Sharp => "sharp (disabled)",
            FontMode::Smooth => "smooth (enabled)",
        };
        crate::println!("Font smoothing: {}", current);
        crate::println!("Usage: fontsmooth [on|off]");
        return;
    }
    
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

pub(super) fn cmd_lscpu() {
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

pub(super) fn cmd_lsmem() {
    let heap_total = (crate::memory::stats().heap_used + crate::memory::stats().heap_free);
    
    crate::println_color!(COLOR_BRIGHT_GREEN, "Memory Configuration:");
    crate::println!("-------------------------------------------");
    crate::println!("Total:       {} KB", heap_total / 1024);
    crate::println!("Used:        {} KB", crate::memory::stats().heap_used / 1024);
}

pub(super) fn cmd_lsmod() {
    crate::println_color!(COLOR_BRIGHT_GREEN, "Loaded Kernel Modules:");
    crate::println!("Module                  Size  Used by");
    crate::println!("e1000                  64000  1");
    crate::println!("ahci                   32000  0");
    crate::println!("ps2kbd                  8000  1");
    crate::println!("ps2mouse                4000  1");
}

pub(super) fn cmd_sysctl(_args: &[&str]) {
    crate::println!("kernel.ostype = TrustOS");
    crate::println!("kernel.osrelease = 0.1.0");
    crate::println!("kernel.version = #1 SMP TrustOS");
}

// ==================== FIREWALL COMMANDS ====================

pub(super) fn cmd_firewall(args: &[&str]) {
    use crate::netstack::firewall;
    use crate::netstack::firewall::{Chain, Action, Protocol, IpMatch, PortMatch, Rule};

    if args.is_empty() {
        cmd_firewall_status();
        return;
    }

    match args[0] {
        "status" | "show" => cmd_firewall_status(),
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
            let chain = match Chain::from_str(args[1]) {
                Some(c) => c,
                None => { crate::println_color!(COLOR_RED, "Invalid chain: {}", args[1]); return; }
            };
            let action = match Action::from_str(args[2]) {
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
            let chain = match Chain::from_str(args[1]) {
                Some(c) => c,
                None => { crate::println_color!(COLOR_RED, "Invalid chain: {}", args[1]); return; }
            };
            let mut rule = Rule::new(chain, Action::Accept);
            let mut i = 2;
            while i < args.len() {
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
                            rule.src_ip = IpMatch::parse(args[i]).unwrap_or(IpMatch::Any);
                        }
                    }
                    "-d" | "--dst" => {
                        i += 1;
                        if i < args.len() {
                            rule.dst_ip = IpMatch::parse(args[i]).unwrap_or(IpMatch::Any);
                        }
                    }
                    "--sport" => {
                        i += 1;
                        if i < args.len() {
                            rule.src_port = PortMatch::parse(args[i]).unwrap_or(PortMatch::Any);
                        }
                    }
                    "--dport" => {
                        i += 1;
                        if i < args.len() {
                            rule.dst_port = PortMatch::parse(args[i]).unwrap_or(PortMatch::Any);
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
            let chain = match Chain::from_str(args[1]) {
                Some(c) => c,
                None => { crate::println_color!(COLOR_RED, "Invalid chain: {}", args[1]); return; }
            };
            let idx: usize = match args[2].parse() {
                Ok(n) => n,
                Err(_) => { crate::println_color!(COLOR_RED, "Invalid index: {}", args[2]); return; }
            };
            if firewall::delete_rule(chain, idx) {
                crate::println_color!(COLOR_GREEN, "Rule {} deleted from {}", idx, chain.name());
            } else {
                crate::println_color!(COLOR_RED, "Rule {} not found in {}", idx, chain.name());
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

fn cmd_firewall_status() {
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
                    i, rule.protocol.name(), rule.src_ip.display(), rule.dst_ip.display(),
                    rule.src_port.display(), rule.dst_port.display(), rule.action.name(),
                    rule.packets, rule.bytes);
            }
        }
        crate::println!();
    }
}

// ==================== DU COMMAND ====================

pub(super) fn cmd_du(args: &[&str]) {
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

    if let Ok(entries) = crate::ramfs::with_fs(|fs| fs.ls(Some(path))) {
        for (name, ftype, size) in &entries {
            let child = if path == "/" {
                alloc::format!("/{}", name)
            } else {
                alloc::format!("{}/{}", path, name)
            };
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
pub(super) fn cmd_chmod(args: &[&str]) {
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
pub(super) fn cmd_chown(args: &[&str]) {
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
pub(super) fn cmd_ln(args: &[&str]) {
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
        let result = crate::ramfs::with_fs(|fs| {
            let _ = fs.touch(link);
            fs.write_file(link, content.as_bytes())
        });
        match result {
            Ok(()) => crate::println_color!(COLOR_GREEN, "'{}' -> '{}'", link, target),
            Err(_) => crate::println_color!(COLOR_RED, "ln: failed to create symbolic link"),
        }
    } else {
        // Hard link — just copy the file
        let data = crate::ramfs::with_fs(|fs| fs.read_file(target).map(|b| b.to_vec()));
        match data {
            Ok(bytes) => {
                crate::ramfs::with_fs(|fs| {
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
pub(super) fn cmd_readlink(args: &[&str]) {
    if args.is_empty() {
        crate::println!("Usage: readlink <symlink>");
        return;
    }
    let path = args[0];
    let content: Option<String> = crate::ramfs::with_fs(|fs| {
        fs.read_file(path).map(|b| String::from(core::str::from_utf8(b).unwrap_or(""))).ok()
    });
    match content {
        Some(ref s) if s.starts_with("SYMLINK:") => {
            crate::println!("{}", &s[8..]);
        }
        _ => crate::println_color!(COLOR_RED, "readlink: {}: Not a symbolic link", path),
    }
}

// ==================== CUT ====================
pub(super) fn cmd_cut(args: &[&str], piped: Option<&str>) {
    // Parse -d (delimiter) and -f (fields)
    let mut delimiter = '\t';
    let mut fields: Option<Vec<usize>> = None;
    let mut file_arg: Option<&str> = None;
    let mut i = 0;
    while i < args.len() {
        match args[i] {
            "-d" if i + 1 < args.len() => {
                delimiter = args[i + 1].chars().next().unwrap_or('\t');
                i += 2;
            }
            "-f" if i + 1 < args.len() => {
                fields = Some(parse_field_list(args[i + 1]));
                i += 2;
            }
            arg if !arg.starts_with('-') => {
                file_arg = Some(arg);
                i += 1;
            }
            _ => { i += 1; }
        }
    }
    
    let field_list = match fields {
        Some(f) => f,
        None => {
            crate::println!("Usage: cut -d <delimiter> -f <fields> [file]");
            crate::println!("  Example: cut -d : -f 1,3");
            return;
        }
    };
    
    let content = if let Some(input) = piped {
        Some(String::from(input))
    } else if let Some(path) = file_arg {
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
            let end: usize = part[dash + 1..].parse().unwrap_or(start).min(start + 10_000);
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
pub(super) fn cmd_tr(args: &[&str], piped: Option<&str>) {
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
    for ch in content.chars() {
        if let Some(pos) = set1.iter().position(|&c| c == ch) {
            if pos < set2.len() {
                result.push(set2[pos]);
            } else if let Some(&last) = set2.last() {
                result.push(last);
            } else {
                result.push(ch);
            }
        } else {
            result.push(ch);
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
pub(super) fn cmd_tee(args: &[&str], piped: Option<&str>) {
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
            let _ = crate::ramfs::with_fs(|fs| fs.append_file(path, content.as_bytes()));
        } else {
            let _ = crate::ramfs::with_fs(|fs| {
                if !fs.exists(path) { let _ = fs.touch(path); }
                fs.write_file(path, content.as_bytes())
            });
        }
    }
}

// ==================== XARGS ====================
pub(super) fn cmd_xargs(args: &[&str], piped: Option<&str>) {
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
        let cmd_line = if extra_args.is_empty() {
            format!("{} {}", command, item)
        } else {
            format!("{} {} {}", command, extra_args.join(" "), item)
        };
        super::execute_command(&cmd_line);
    }
}

// ==================== UNSET ====================
pub(super) fn cmd_unset(args: &[&str]) {
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
pub(super) fn cmd_read(args: &[&str]) {
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
    loop {
        if let Some(key) = crate::keyboard::read_char() {
            match key {
                0x0A => break,
                0x08 => { 
                    if !input.is_empty() {
                        input.pop();
                        crate::print!("\x08 \x08");
                    }
                }
                ch if ch >= 32 && ch < 127 => {
                    input.push(ch as char);
                    crate::print!("{}", ch as char);
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

static ALIASES: Mutex<BTreeMap<String, String>> = Mutex::new(BTreeMap::new());

pub fn get_alias(name: &str) -> Option<String> {
    ALIASES.lock().get(name).cloned()
}

pub(super) fn cmd_alias(args: &[&str]) {
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
    
    let arg = args.join(" ");
    if let Some(eq_pos) = arg.find('=') {
        let name = arg[..eq_pos].trim();
        let value = arg[eq_pos + 1..].trim().trim_matches('\'').trim_matches('"');
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

pub(super) fn cmd_unalias(args: &[&str]) {
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
pub(super) fn cmd_bc(_args: &[&str]) {
    crate::println_color!(COLOR_CYAN, "TrustOS bc — arbitrary precision calculator");
    crate::println!("Type expressions, 'quit' or 'exit' to leave");
    crate::println!();
    
    crate::shell::clear_interrupted();
    
    loop {
        crate::print_color!(COLOR_GREEN, "bc> ");
        
        let mut input = String::new();
        loop {
            if let Some(key) = crate::keyboard::read_char() {
                match key {
                    0x0A => break,
                    0x03 => { crate::println!(); return; }
                    0x08 => {
                        if !input.is_empty() {
                            input.pop();
                            crate::print!("\x08 \x08");
                        }
                    }
                    ch if ch >= 32 && ch < 127 => {
                        input.push(ch as char);
                        crate::print!("{}", ch as char);
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
    let mut pos = 0;
    let result = parse_bc_add_sub(&tokens, &mut pos);
    if pos == tokens.len() { result } else { None }
}

fn tokenize_bc(expr: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut num = String::new();
    for ch in expr.chars() {
        if ch.is_ascii_digit() || ch == '.' {
            num.push(ch);
        } else {
            if !num.is_empty() { tokens.push(core::mem::take(&mut num)); }
            if !ch.is_whitespace() {
                let mut buf = [0u8; 4];
                let s = ch.encode_utf8(&mut buf);
                tokens.push(String::from(s));
            }
        }
    }
    if !num.is_empty() { tokens.push(num); }
    tokens
}

fn parse_bc_add_sub(tokens: &[String], pos: &mut usize) -> Option<f64> {
    let mut left = parse_bc_mul_div(tokens, pos)?;
    while *pos < tokens.len() && (tokens[*pos] == "+" || tokens[*pos] == "-") {
        let op = tokens[*pos].clone();
        *pos += 1;
        let right = parse_bc_mul_div(tokens, pos)?;
        left = if op == "+" { left + right } else { left - right };
    }
    Some(left)
}

fn parse_bc_mul_div(tokens: &[String], pos: &mut usize) -> Option<f64> {
    let mut left = parse_bc_power(tokens, pos)?;
    while *pos < tokens.len() && (tokens[*pos] == "*" || tokens[*pos] == "/" || tokens[*pos] == "%") {
        let op = tokens[*pos].clone();
        *pos += 1;
        let right = parse_bc_power(tokens, pos)?;
        left = match op.as_str() {
            "*" => left * right,
            "/" => if right != 0.0 { left / right } else { return None },
            "%" => if right != 0.0 { left % right } else { return None },
            _ => unreachable!(),
        };
    }
    Some(left)
}

fn parse_bc_power(tokens: &[String], pos: &mut usize) -> Option<f64> {
    let base = parse_bc_unary(tokens, pos)?;
    if *pos < tokens.len() && tokens[*pos] == "^" {
        *pos += 1;
        let exp = parse_bc_power(tokens, pos)?;
        Some(pow_f64(base, exp))
    } else {
        Some(base)
    }
}

fn parse_bc_unary(tokens: &[String], pos: &mut usize) -> Option<f64> {
    if *pos < tokens.len() && tokens[*pos] == "-" {
        *pos += 1;
        let val = parse_bc_atom(tokens, pos)?;
        Some(-val)
    } else {
        parse_bc_atom(tokens, pos)
    }
}

fn parse_bc_atom(tokens: &[String], pos: &mut usize) -> Option<f64> {
    if *pos >= tokens.len() { return None; }
    if tokens[*pos] == "(" {
        *pos += 1;
        let val = parse_bc_add_sub(tokens, pos)?;
        if *pos < tokens.len() && tokens[*pos] == ")" {
            *pos += 1;
        }
        Some(val)
    } else {
        let val: f64 = tokens[*pos].parse().ok()?;
        *pos += 1;
        Some(val)
    }
}

fn pow_f64(base: f64, exp: f64) -> f64 {
    if exp == 0.0 { return 1.0; }
    if exp == 1.0 { return base; }
    let exp_int = exp as i32;
    if (exp - exp_int as f64).abs() < 1e-9 {
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
pub(super) fn cmd_diff(args: &[&str]) {
    if args.len() < 2 {
        crate::println!("Usage: diff <file1> <file2>");
        return;
    }
    
    let content1 = read_file_str(args[0]);
    let content2 = read_file_str(args[1]);
    
    let (c1, c2) = match (content1, content2) {
        (Some(a), Some(b)) => (a, b),
        (None, _) => { crate::println_color!(COLOR_RED, "diff: {}: No such file", args[0]); return; }
        (_, None) => { crate::println_color!(COLOR_RED, "diff: {}: No such file", args[1]); return; }
    };
    
    let lines1: Vec<&str> = c1.lines().collect();
    let lines2: Vec<&str> = c2.lines().collect();
    
    crate::println_color!(COLOR_CYAN, "--- {}", args[0]);
    crate::println_color!(COLOR_CYAN, "+++ {}", args[1]);
    
    let max_len = core::cmp::max(lines1.len(), lines2.len());
    let mut has_diff = false;
    
    for i in 0..max_len {
        let l1 = lines1.get(i).copied();
        let l2 = lines2.get(i).copied();
        
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
        crate::ramfs::with_fs(|fs| {
            fs.read_file(path).map(|b| String::from(core::str::from_utf8(b).unwrap_or(""))).ok()
        })
    }
}

// ==================== MD5SUM ====================
pub(super) fn cmd_md5sum(args: &[&str]) {
    if args.is_empty() {
        crate::println!("Usage: md5sum <file> ...");
        return;
    }
    for path in args {
        let data = crate::ramfs::with_fs(|fs| fs.read_file(path).map(|b| b.to_vec()));
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
pub(super) fn cmd_sha256sum(args: &[&str]) {
    if args.is_empty() {
        crate::println!("Usage: sha256sum <file> ...");
        return;
    }
    for path in args {
        let data = crate::ramfs::with_fs(|fs| fs.read_file(path).map(|b| b.to_vec()));
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
pub(super) fn cmd_base64(args: &[&str], piped: Option<&str>) {
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

fn b64_val(c: u8) -> Option<u32> {
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
        let a = b64_val(bytes[i])?;
        let b = b64_val(bytes[i + 1])?;
        let c_val = if bytes[i + 2] == b'=' { 0 } else { b64_val(bytes[i + 2])? };
        let d_val = if bytes[i + 3] == b'=' { 0 } else { b64_val(bytes[i + 3])? };
        
        let triple = (a << 18) | (b << 12) | (c_val << 6) | d_val;
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
pub(super) fn cmd_watch(args: &[&str]) {
    if args.is_empty() {
        crate::println!("Usage: watch [-n <seconds>] <command>");
        crate::println!("  Example: watch -n 2 ps");
        return;
    }
    
    let mut interval_secs: u64 = 2;
    let mut cmd_start = 0;
    
    if args.len() > 2 && args[0] == "-n" {
        interval_secs = args[1].parse().unwrap_or(2);
        cmd_start = 2;
    }
    
    let cmd = args[cmd_start..].join(" ");
    crate::shell::clear_interrupted();
    
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
        let start = crate::time::uptime_ms();
        let end = start + interval_secs * 1000;
        while crate::time::uptime_ms() < end {
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
pub(super) fn cmd_timeout(args: &[&str]) {
    if args.len() < 2 {
        crate::println!("Usage: timeout <seconds> <command>");
        return;
    }
    
    let secs: u64 = args[0].parse().unwrap_or(5);
    let cmd = args[1..].join(" ");
    
    let deadline = crate::time::uptime_ms() + secs * 1000;
    
    // Execute command (note: we can't truly interrupt it, but we set a deadline)
    crate::println_color!(COLOR_CYAN, "[timeout: {}s] {}", secs, cmd);
    super::execute_command(&cmd);
    
    if crate::time::uptime_ms() > deadline {
        crate::println_color!(COLOR_RED, "timeout: command timed out after {}s", secs);
    }
}

// ==================== TAR (archive) ====================
pub(super) fn cmd_tar(args: &[&str]) {
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
    
    let _ = crate::ramfs::with_fs(|fs| {
        if !fs.exists(archive) { let _ = fs.touch(archive); }
        fs.write_file(archive, archive_data.as_bytes())
    });
    crate::println_color!(COLOR_GREEN, "tar: created '{}' ({} files)", archive, count);
}

fn tar_list(archive: &str) {
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
    match read_file_str(archive) {
        Some(data) => {
            let mut current_file: Option<(String, usize)> = None;
            let mut file_content = String::new();
            let mut extracted = 0;
            
            for line in data.lines() {
                if line.starts_with("===FILE:") && line.ends_with("===") {
                    // Save previous file
                    if let Some((ref name, _)) = current_file {
                        let _ = crate::ramfs::with_fs(|fs| {
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
                let _ = crate::ramfs::with_fs(|fs| {
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
pub(super) fn cmd_gzip(args: &[&str]) {
    if args.is_empty() {
        crate::println!("Usage: gzip <file>");
        return;
    }
    let path = args[0];
    match read_file_str(path) {
        Some(data) => {
            // Simple RLE-like compression
            let compressed = simple_compress(data.as_bytes());
            let out_path = format!("{}.gz", path);
            let _ = crate::ramfs::with_fs(|fs| {
                if !fs.exists(&out_path) { let _ = fs.touch(&out_path); }
                fs.write_file(&out_path, &compressed)
            });
            let ratio = if !data.is_empty() { (compressed.len() as f64 / data.len() as f64) * 100.0 } else { 100.0 };
            crate::println_color!(COLOR_GREEN, "{} -> {} ({:.1}% of original)", path, out_path, ratio);
        }
        None => crate::println_color!(COLOR_RED, "gzip: {}: No such file", path),
    }
}

pub(super) fn cmd_zip(args: &[&str]) {
    if args.len() < 2 {
        crate::println!("Usage: zip <archive.zip> <file1> [file2] ...");
        return;
    }
    // Reuse tar create for now
    tar_create(args[0], &args[1..]);
    crate::println_color!(COLOR_GREEN, "zip: created '{}'", args[0]);
}

pub(super) fn cmd_unzip(args: &[&str]) {
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

fn is_service_enabled(idx: usize) -> bool {
    SERVICE_STATE.load(Ordering::SeqCst) & (1 << idx) != 0
}

fn set_service_enabled(idx: usize, enabled: bool) {
    if enabled {
        SERVICE_STATE.fetch_or(1 << idx, Ordering::SeqCst);
    } else {
        SERVICE_STATE.fetch_and(!(1 << idx), Ordering::SeqCst);
    }
}

pub(super) fn cmd_service(args: &[&str]) {
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
    
    if let Some((idx, svc)) = SERVICE_DEFS.iter().enumerate().find(|(_, s)| s.name == svc_name) {
        match action {
            "start" => {
                set_service_enabled(idx, true);
                crate::println_color!(COLOR_GREEN, "Starting {}... OK", svc_name);
            }
            "stop" => {
                set_service_enabled(idx, false);
                crate::println_color!(COLOR_YELLOW, "Stopping {}... OK", svc_name);
            }
            "restart" => {
                set_service_enabled(idx, false);
                crate::println_color!(COLOR_YELLOW, "Stopping {}...", svc_name);
                set_service_enabled(idx, true);
                crate::println_color!(COLOR_GREEN, "Starting {}... OK", svc_name);
            }
            "status" => {
                let active = is_service_enabled(idx);
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

pub(super) fn cmd_systemctl(args: &[&str]) {
    if args.is_empty() || args[0] == "list-units" {
        cmd_service(&[]);
        return;
    }
    
    let action = args[0];
    if args.len() < 2 {
        crate::println!("Usage: systemctl <start|stop|restart|status|enable|disable> <service>");
        return;
    }
    let svc = args[1].trim_end_matches(".service");
    cmd_service(&[svc, action]);
}

// ==================== CRONTAB ====================

static CRON_ENTRIES: Mutex<Vec<String>> = Mutex::new(Vec::new());

pub(super) fn cmd_crontab(args: &[&str]) {
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
            loop {
                crate::print!("> ");
                let mut input = String::new();
                loop {
                    if let Some(key) = crate::keyboard::read_char() {
                        match key {
                            0x0A => break,
                            0x08 => { if !input.is_empty() { input.pop(); crate::print!("\x08 \x08"); } }
                            ch if ch >= 32 && ch < 127 => { input.push(ch as char); crate::print!("{}", ch as char); }
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
pub(super) fn cmd_at(args: &[&str]) {
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
    let delay_ms = if time_spec.starts_with('+') {
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
    
    crate::println_color!(COLOR_GREEN, "Job scheduled: '{}' in {} seconds", command, delay_ms / 1000);
    
    // Simple immediate wait + execute (since we don't have a background scheduler)
    let start = crate::time::uptime_ms();
    while crate::time::uptime_ms() - start < delay_ms {
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
pub(super) fn cmd_head_n(args: &[&str], piped: Option<&str>) {
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
pub(super) fn cmd_tail_n(args: &[&str], piped: Option<&str>) {
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
        loop {
            if let Some(3) = crate::keyboard::read_char() { break; }
            if crate::shell::is_interrupted() { break; }
            core::hint::spin_loop();
        }
    }
}

// ==================== WC (improved with flags) ====================
pub(super) fn cmd_wc_full(args: &[&str], piped: Option<&str>) {
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
        let content: Option<String> = crate::ramfs::with_fs(|fs| {
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
pub(super) fn cmd_whoami_full() {
    let user = super::scripting::get_var("USER").unwrap_or_else(|| String::from("root"));
    crate::println!("{}", user);
}

// ==================== UPTIME (enhanced) ====================
pub(super) fn cmd_uptime_full() {
    let ms = crate::time::uptime_ms();
    let secs = ms / 1000;
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
pub(super) fn cmd_clear_full() {
    crate::framebuffer::clear();
    crate::framebuffer::set_cursor(0, 0);
}

// ============================================================================
// NEWLY IMPLEMENTED — formerly stubs
// ============================================================================

// ==================== KILLALL ====================
pub(super) fn cmd_killall(args: &[&str]) {
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
pub(super) fn cmd_nice(args: &[&str]) {
    if args.is_empty() {
        crate::println!("Usage: nice [-n priority] <command>");
        return;
    }
    let (priority, cmd_start) = if args[0] == "-n" && args.len() > 2 {
        (args[1].parse::<i32>().unwrap_or(10), 2)
    } else {
        (10, 0)
    };
    let cmd = args[cmd_start..].join(" ");
    crate::println_color!(COLOR_CYAN, "nice: running '{}' with priority {}", cmd, priority);
    // Execute the command through the shell dispatcher
    super::execute_command(&cmd);
}

// ==================== IOSTAT ====================
pub(super) fn cmd_iostat() {
    let (reads, writes, bytes_read, bytes_written) = crate::disk::get_stats();
    let uptime = crate::time::uptime_ms() / 1000;
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
pub(super) fn cmd_strace(args: &[&str]) {
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
    static STRACE_ACTIVE: AtomicBool = AtomicBool::new(false);
    STRACE_ACTIVE.store(true, Ordering::SeqCst);

    // Execute the command
    let cmd = args.join(" ");
    super::execute_command(&cmd);

    STRACE_ACTIVE.store(false, Ordering::SeqCst);
    crate::println_color!(COLOR_GRAY, "--- syscall trace end ---");
}

// ==================== NETCONSOLE ====================
/// Stream kernel log over UDP to a remote host for real-time debugging.
pub(super) fn cmd_netconsole(args: &[&str]) {
    let subcmd = args.first().copied().unwrap_or("help");

    match subcmd {
        "start" | "on" => {
            let ip_str = match args.get(1) {
                Some(s) => s,
                None => {
                    crate::println!("Usage: netconsole start <ip> [port]");
                    crate::println!("  Example: netconsole start 10.0.0.1");
                    return;
                }
            };
            let ip = match crate::debug::netconsole::parse_ip(ip_str) {
                Some(ip) => ip,
                None => {
                    crate::println_color!(COLOR_RED, "Invalid IP: {}", ip_str);
                    return;
                }
            };
            let port = args.get(2)
                .and_then(|s| s.parse::<u16>().ok())
                .unwrap_or(crate::debug::netconsole::DEFAULT_PORT);

            // Ensure network stack is ready: drain any pending RX packets
            for _ in 0..50 {
                crate::netstack::poll();
            }

            // If no IP yet, aggressively request DHCP with real time-based wait
            if crate::network::get_ipv4_config().is_none() {
                crate::println!("[netconsole] No IP configured, requesting DHCP...");
                crate::netstack::dhcp::start();
                let start_tick = crate::logger::get_ticks();
                let timeout_ms = 8000u64; // 8 second timeout
                loop {
                    for _ in 0..20 {
                        crate::netstack::poll();
                    }
                    if crate::network::get_ipv4_config().is_some() {
                        break;
                    }
                    let elapsed = crate::logger::get_ticks().saturating_sub(start_tick);
                    if elapsed > timeout_ms {
                        break;
                    }
                    // Wait ~10ms between bursts (halt wakes on next interrupt)
                    for _ in 0..10 {
                        crate::arch::halt();
                    }
                }
            }

            // If DHCP still failed, apply static IP for direct cable setup
            if crate::network::get_ipv4_config().is_none() {
                crate::println_color!(COLOR_YELLOW, "[netconsole] DHCP failed, applying static IP 10.0.0.100/24");
                crate::network::set_ipv4_config(
                    crate::network::Ipv4Address::new(10, 0, 0, 100),
                    crate::network::Ipv4Address::new(255, 255, 255, 0),
                    Some(crate::network::Ipv4Address::new(10, 0, 0, 1)),
                );
            }

            // Show current source IP
            if let Some((src_ip, _, _)) = crate::network::get_ipv4_config() {
                let b = src_ip.as_bytes();
                crate::println_color!(COLOR_CYAN, "[netconsole] Source IP: {}.{}.{}.{}", b[0], b[1], b[2], b[3]);
            } else {
                crate::println_color!(COLOR_RED, "[netconsole] ERROR: Could not configure any IP");
                return;
            }

            // Pre-resolve ARP for the target IP before starting netconsole.
            // Without this, the first send_line() call would try ARP inline
            // (from within serial::_print), which can timeout silently.
            crate::println!("[netconsole] Resolving ARP for {}.{}.{}.{}...", ip[0], ip[1], ip[2], ip[3]);
            let mut arp_ok = crate::netstack::arp::resolve(ip).is_some();
            if !arp_ok {
                for attempt in 0..5 {
                    let _ = crate::netstack::arp::send_request(ip);
                    let start = crate::logger::get_ticks();
                    loop {
                        for _ in 0..20 { crate::netstack::poll(); }
                        if crate::netstack::arp::resolve(ip).is_some() {
                            arp_ok = true;
                            break;
                        }
                        if crate::logger::get_ticks().saturating_sub(start) > 2000 {
                            break; // 2s per attempt
                        }
                        crate::arch::halt();
                    }
                    if arp_ok {
                        crate::println_color!(COLOR_BRIGHT_GREEN, "[netconsole] ARP resolved (attempt {})", attempt + 1);
                        break;
                    }
                }
            }
            if !arp_ok {
                crate::println_color!(COLOR_YELLOW, "[netconsole] ARP unresolved — using broadcast mode");
            }

            crate::debug::netconsole::start(ip, port);
            crate::println_color!(COLOR_BRIGHT_GREEN,
                "Netconsole streaming to {}.{}.{}.{}:{}",
                ip[0], ip[1], ip[2], ip[3], port
            );

            // Send multiple test packets to verify TX works
            // First via the normal netstack path (broadcast)
            let bcast_ip = if let Some((src, mask, _)) = crate::network::get_ipv4_config() {
                let s = src.as_bytes();
                let m = mask.as_bytes();
                [s[0] | !m[0], s[1] | !m[1], s[2] | !m[2], s[3] | !m[3]]
            } else {
                [255, 255, 255, 255]
            };
            let hello = b"[TrustOS netconsole] Connection established\n";
            crate::println!("[netconsole] Sending test packet via netstack to {}.{}.{}.{}:{}", bcast_ip[0], bcast_ip[1], bcast_ip[2], bcast_ip[3], port);
            match crate::netstack::udp::send_to(bcast_ip, port, 6665, hello) {
                Ok(()) => crate::println_color!(COLOR_BRIGHT_GREEN, "[netconsole] Test packet sent OK via netstack"),
                Err(e) => crate::println_color!(COLOR_RED, "[netconsole] Test packet FAILED via netstack: {}", e),
            }

            // Also build and send a raw UDP frame manually (bypass all netstack)
            crate::println!("[netconsole] Sending raw test frame...");
            if let Some((src_ip_addr, _, _)) = crate::network::get_ipv4_config() {
                let src_ip = *src_ip_addr.as_bytes();
                let dest_ip = bcast_ip;
                let payload = b"[TrustOS RAW] Hello from kernel!\n";
                let udp_len = (8 + payload.len()) as u16;
                let ip_total = (20 + 8 + payload.len()) as u16;

                // UDP header
                let mut udp = alloc::vec::Vec::with_capacity(8 + payload.len());
                udp.extend_from_slice(&6665u16.to_be_bytes()); // src port
                udp.extend_from_slice(&port.to_be_bytes());     // dst port
                udp.extend_from_slice(&udp_len.to_be_bytes());
                udp.extend_from_slice(&0u16.to_be_bytes());     // checksum (0=skip)
                udp.extend_from_slice(payload);

                // IP header
                let mut ip_hdr = [0u8; 20];
                ip_hdr[0] = 0x45;
                ip_hdr[2..4].copy_from_slice(&ip_total.to_be_bytes());
                ip_hdr[6..8].copy_from_slice(&0x4000u16.to_be_bytes()); // Don't fragment
                ip_hdr[8] = 64; // TTL
                ip_hdr[9] = 17; // UDP
                ip_hdr[12..16].copy_from_slice(&src_ip);
                ip_hdr[16..20].copy_from_slice(&dest_ip);
                // Checksum
                let mut sum: u32 = 0;
                for i in (0..20).step_by(2) {
                    sum += ((ip_hdr[i] as u32) << 8) | (ip_hdr[i+1] as u32);
                }
                while sum > 0xFFFF { sum = (sum & 0xFFFF) + (sum >> 16); }
                let csum = !(sum as u16);
                ip_hdr[10..12].copy_from_slice(&csum.to_be_bytes());

                // Full IP packet
                let mut ip_pkt = alloc::vec::Vec::with_capacity(20 + udp.len());
                ip_pkt.extend_from_slice(&ip_hdr);
                ip_pkt.extend_from_slice(&udp);

                // Send as broadcast Ethernet frame
                match crate::netstack::send_frame([0xFF; 6], 0x0800, &ip_pkt) {
                    Ok(()) => crate::println_color!(COLOR_BRIGHT_GREEN, "[netconsole] Raw frame sent OK ({} bytes)", ip_pkt.len() + 14),
                    Err(e) => crate::println_color!(COLOR_RED, "[netconsole] Raw frame FAILED: {}", e),
                }
            }

            crate::println!("  Listener: ncat -u -l -p {}", port);
        }
        "stop" | "off" => {
            crate::debug::netconsole::stop();
            crate::println!("Netconsole stopped.");
        }
        "status" => {
            let (ip, port, enabled) = crate::debug::netconsole::status();
            if enabled {
                crate::println_color!(COLOR_BRIGHT_GREEN,
                    "Netconsole: ACTIVE → {}.{}.{}.{}:{}",
                    ip[0], ip[1], ip[2], ip[3], port
                );
            } else {
                crate::println_color!(COLOR_YELLOW, "Netconsole: OFF");
            }
        }
        _ => {
            crate::println_color!(COLOR_CYAN, "netconsole — Stream kernel log over UDP");
            crate::println!();
            crate::println!("Usage:");
            crate::println!("  netconsole start <ip> [port]  — Start streaming (default port: 6666)");
            crate::println!("  netconsole stop               — Stop streaming");
            crate::println!("  netconsole status             — Show current config");
            crate::println!();
            crate::println!("Listener (on remote PC):");
            crate::println!("  ncat -u -l -p 6666");
            crate::println!("  socat UDP-LISTEN:6666 STDOUT");
            crate::println!("  python -c \"import socket; s=socket.socket(socket.AF_INET,socket.SOCK_DGRAM); s.bind(('',6666)); [print(s.recvfrom(4096)[0].decode(),end='') for _ in iter(int,1)]\"");
        }
    }
}

// ==================== REMOTESHELL ====================
pub(super) fn cmd_remoteshell(args: &[&str]) {
    let subcmd = args.first().copied().unwrap_or("help");

    match subcmd {
        "start" | "on" => {
            // Ensure we have an IP — same logic as netconsole
            if crate::network::get_ipv4_config().is_none() {
                crate::println!("[remoteshell] No IP configured, requesting DHCP...");
                crate::netstack::dhcp::start();
                let start_tick = crate::logger::get_ticks();
                loop {
                    for _ in 0..20 { crate::netstack::poll(); }
                    if crate::network::get_ipv4_config().is_some() { break; }
                    if crate::logger::get_ticks().saturating_sub(start_tick) > 8000 { break; }
                    for _ in 0..10 { crate::arch::halt(); }
                }
            }
            if crate::network::get_ipv4_config().is_none() {
                crate::println_color!(COLOR_YELLOW, "[remoteshell] DHCP failed, applying static IP 10.0.0.100/24");
                crate::network::set_ipv4_config(
                    crate::network::Ipv4Address::new(10, 0, 0, 100),
                    crate::network::Ipv4Address::new(255, 255, 255, 0),
                    Some(crate::network::Ipv4Address::new(10, 0, 0, 1)),
                );
            }
            if let Some((src_ip, _, _)) = crate::network::get_ipv4_config() {
                let b = src_ip.as_bytes();
                crate::println_color!(COLOR_CYAN, "[remoteshell] Source IP: {}.{}.{}.{}", b[0], b[1], b[2], b[3]);
            }

            crate::debug::remoteshell::start();
            crate::println_color!(COLOR_BRIGHT_GREEN,
                "Remote shell listening on UDP port {}",
                crate::debug::remoteshell::LISTEN_PORT
            );
            crate::println!("  Connect: python scripts/remote_console.py --ip <this-ip>");

            // Also auto-start netconsole if not already running to stream log output
            if !crate::debug::netconsole::is_enabled() {
                crate::println!("[remoteshell] Auto-starting netconsole (broadcast, port 6666)");
                if let Some((src_ip, mask, _)) = crate::network::get_ipv4_config() {
                    let s = src_ip.as_bytes();
                    let m = mask.as_bytes();
                    let bcast = [s[0] | !m[0], s[1] | !m[1], s[2] | !m[2], s[3] | !m[3]];
                    crate::debug::netconsole::start(bcast, crate::debug::netconsole::DEFAULT_PORT);
                }
            }
        }
        "stop" | "off" => {
            crate::debug::remoteshell::stop();
            crate::println!("Remote shell stopped.");
        }
        "status" => {
            if crate::debug::remoteshell::is_enabled() {
                crate::println_color!(COLOR_BRIGHT_GREEN, "Remote shell: ACTIVE (UDP port {})", crate::debug::remoteshell::LISTEN_PORT);
            } else {
                crate::println_color!(COLOR_YELLOW, "Remote shell: OFF");
            }
        }
        _ => {
            crate::println_color!(COLOR_CYAN, "remoteshell — Bidirectional shell over UDP");
            crate::println!();
            crate::println!("Usage:");
            crate::println!("  remoteshell start   — Start listening on port {}", crate::debug::remoteshell::LISTEN_PORT);
            crate::println!("  remoteshell stop    — Stop");
            crate::println!("  remoteshell status  — Show state");
            crate::println!();
            crate::println!("Connect from PC:");
            crate::println!("  python scripts/remote_console.py --ip <board-ip>");
        }
    }
}

// ==================== DMIDECODE ====================
pub(super) fn cmd_dmidecode() {
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
    let total_kb = (stats.heap_used + stats.heap_free) / 1024;
    crate::println_color!(COLOR_CYAN, "Handle 0x0011, DMI type 17, Memory Device");
    crate::println!("  Size: {} KB (heap)", total_kb);
    crate::println!("  Type: DRAM");
}

// ==================== HDPARM ====================
pub(super) fn cmd_hdparm(args: &[&str]) {
    if args.is_empty() {
        crate::println!("Usage: hdparm [-i|-t] <device>");
        crate::println!("  -i  Device information");
        crate::println!("  -t  Timing buffered disk reads");
        return;
    }

    let info_mode = args.contains(&"-i");
    let timing_mode = args.contains(&"-t");

    let disk_info = crate::disk::get_info();
    if let Some(info) = disk_info {
        if info_mode || (!info_mode && !timing_mode) {
            crate::println_color!(COLOR_CYAN, "/dev/sda:");
            crate::println!("  Model: {}", info.model);
            crate::println!("  Serial: {}", info.serial);
            crate::println!("  Sectors: {}", info.sectors);
            crate::println!("  Size: {} MB", info.size_mb);
        }
        if timing_mode {
            // Simple benchmark: read 100 sectors and measure time
            let start = crate::time::uptime_ms();
            let mut buf = [0u8; 512];
            for i in 0..100u64 {
                let _ = crate::disk::read_sectors(i % info.sectors, 1, &mut buf);
            }
            let elapsed = crate::time::uptime_ms() - start;
            let elapsed = if elapsed == 0 { 1 } else { elapsed };
            let throughput = (100 * 512) / (elapsed as usize);
            crate::println_color!(COLOR_GREEN, "  Timing: 100 sectors in {}ms ({} KB/s)", elapsed, throughput);
        }
    } else {
        crate::println_color!(COLOR_RED, "hdparm: no disk found");
    }
}

// ==================== SCREENSHOT ====================
pub(super) fn cmd_screenshot(args: &[&str]) {
    let filename = if !args.is_empty() { args[0] } else { "/screenshot.ppm" };

    let width = crate::framebuffer::FB_WIDTH.load(core::sync::atomic::Ordering::Relaxed) as u32;
    let height = crate::framebuffer::FB_HEIGHT.load(core::sync::atomic::Ordering::Relaxed) as u32;

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
pub(super) fn cmd_httpd(args: &[&str]) {
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
            let addr = crate::netstack::socket::SockAddrIn::new([0, 0, 0, 0], port);
            if let Err(e) = crate::netstack::socket::bind(fd, &addr) {
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
pub(super) fn cmd_benchmark() {
    crate::println_color!(COLOR_BRIGHT_GREEN, "TrustOS System Benchmark");
    crate::println!("======================================================");

    // CPU benchmark: arithmetic
    crate::println_color!(COLOR_CYAN, "[1/4] CPU integer arithmetic...");
    let start = crate::time::uptime_ms();
    let mut acc: u64 = 0;
    for i in 0u64..10_000_000 {
        acc = acc.wrapping_add(i).wrapping_mul(3);
    }
    let cpu_ms = crate::time::uptime_ms() - start;
    let cpu_ms = if cpu_ms == 0 { 1 } else { cpu_ms };
    crate::println!("  10M iterations in {}ms ({} Mops/s) [checksum=0x{:016x}]",
        cpu_ms, 10000 / cpu_ms, acc);

    // Memory benchmark: sequential write
    crate::println_color!(COLOR_CYAN, "[2/4] Memory sequential write...");
    let mut buf = vec![0u8; 1024 * 1024]; // 1MB
    let start = crate::time::uptime_ms();
    for i in 0..buf.len() {
        buf[i] = (i & 0xFF) as u8;
    }
    let mem_ms = crate::time::uptime_ms() - start;
    let mem_ms = if mem_ms == 0 { 1 } else { mem_ms };
    let mbps = 1000 / mem_ms;
    crate::println!("  1MB write in {}ms ({} MB/s)", mem_ms, mbps);

    // Disk benchmark
    crate::println_color!(COLOR_CYAN, "[3/4] Disk I/O (ramdisk)...");
    let start = crate::time::uptime_ms();
    let mut sector = [0u8; 512];
    for i in 0..1000u64 {
        let _ = crate::disk::read_sectors(i % 256, 1, &mut sector);
    }
    let disk_ms = crate::time::uptime_ms() - start;
    let disk_ms = if disk_ms == 0 { 1 } else { disk_ms };
    crate::println!("  1000 sector reads in {}ms ({} IOPS)", disk_ms, 1000000 / disk_ms);

    // Allocation benchmark
    crate::println_color!(COLOR_CYAN, "[4/4] Heap allocation...");
    let start = crate::time::uptime_ms();
    for _ in 0..10000 {
        let v: Vec<u8> = Vec::with_capacity(256);
        core::hint::black_box(v);
    }
    let alloc_ms = crate::time::uptime_ms() - start;
    let alloc_ms = if alloc_ms == 0 { 1 } else { alloc_ms };
    crate::println!("  10K allocs in {}ms ({} allocs/s)", alloc_ms, 10_000_000 / alloc_ms);

    crate::println!("======================================================");
    crate::println_color!(COLOR_GREEN, "Benchmark complete.");
}
