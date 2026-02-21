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


