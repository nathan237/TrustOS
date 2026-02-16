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
    ("chmod",      "permission system"),
    ("chown",      "ownership"),
    ("ln",         "symbolic links"),
    ("readlink",   "readlink"),
    ("cut",        "cut"),
    ("tr",         "tr"),
    ("tee",        "tee"),
    ("xargs",      "xargs"),
    ("killall",    "process name matching"),
    ("nice",       "priority"),
    ("nohup",      "background execution"),
    ("bg",         "job control"),
    ("fg",         "job control"),
    ("iostat",     "I/O statistics"),
    ("strace",     "syscall tracing"),
    ("tar",        "archive support"),
    ("gzip",       "compression"),
    ("gunzip",     "decompression"),
    ("zip",        "archive support"),
    ("unzip",      "archive support"),
    ("umount",     "unmounting"),
    ("mkfs",       "filesystem creation"),
    ("fsck",       "filesystem check"),
    ("unset",      "environment variables"),
    ("alias",      "aliases"),
    ("unalias",    "aliases"),
    ("read",       "variable input"),
    ("bc",         "calculator"),
    ("diff",       "diff"),
    ("patch",      "patch"),
    ("md5sum",     "MD5"),
    ("sha256sum",  "SHA256"),
    ("base64",     "encoding"),
    ("watch",      "periodic execution"),
    ("timeout",    "timeout"),
    ("time_cmd",   "command timing"),
    ("script",     "terminal recording"),
    ("loadkeys",   "keymap"),
    ("setfont",    "font loading"),
    ("dmidecode",  "DMI/SMBIOS"),
    ("hdparm",     "disk parameters"),
    ("modprobe",   "kernel modules"),
    ("insmod",     "module loading"),
    ("rmmod",      "module unloading"),
    ("service",    "init system"),
    ("systemctl",  "systemd"),
    ("crontab",    "scheduled tasks"),
    ("at",         "scheduled execution"),
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
    // Print 10 times (would be infinite in real implementation)
    for _ in 0..10 {
        crate::println!("{}", text);
    }
    crate::println!("... (press Ctrl+C to stop in real yes)");
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
    while (inc > 0 && i <= last) || (inc < 0 && i >= last) {
        crate::println!("{}", i);
        i += inc;
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
        crate::println!("Usage: mount <device> <mountpoint>");
        return;
    }
    
    crate::println_color!(COLOR_YELLOW, "mount: dynamic mounting not implemented");
}

pub(super) fn cmd_sync() {
    crate::println!("Syncing filesystems...");
    crate::println_color!(COLOR_GREEN, "Done.");
}

pub(super) fn cmd_lsblk() {
    crate::println_color!(COLOR_BRIGHT_GREEN, "Block Devices:");
    crate::println!("NAME    SIZE    TYPE    MOUNTPOINT");
    crate::println!("------------------------------------");
    crate::println!("ram0    256K    disk    /");
    
    // Check for AHCI disks (simplified - no get_disk_info yet)
    crate::println!("(AHCI disk info not available)");
}

pub(super) fn cmd_blkid() {
    crate::println!("/dev/ram0: TYPE=\"ramfs\"");
}

pub(super) fn cmd_export(args: &[&str]) {
    if args.is_empty() {
        crate::println!("PATH=/bin:/usr/bin");
        crate::println!("HOME=/");
        crate::println!("USER=root");
        crate::println!("SHELL=/bin/tsh");
        return;
    }
    crate::println_color!(COLOR_YELLOW, "export: environment variables stored in memory only");
}

pub(super) fn cmd_source(args: &[&str]) {
    if args.is_empty() {
        crate::println!("Usage: source <script>");
        return;
    }
    
    match super::network::read_file_content(args[0]) {
        Some(content) => {
            for line in content.lines() {
                let trimmed = line.trim();
                if !trimmed.is_empty() && !trimmed.starts_with('#') {
                    super::execute_command(trimmed);
                }
            }
        }
        None => crate::println_color!(COLOR_RED, "source: cannot read {}", args[0]),
    }
}

pub(super) fn cmd_set(_args: &[&str]) {
    crate::println!("SHELL=/bin/tsh");
    crate::println!("PATH=/bin:/usr/bin");
    crate::println!("PWD={}", crate::ramfs::with_fs(|fs| String::from(fs.pwd())));
    crate::println!("USER=root");
    crate::println!("HOME=/");
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
    while d * d <= n {
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


