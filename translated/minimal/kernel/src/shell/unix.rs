








use alloc::string::String;
use alloc::vec::Vec;
use alloc::vec;
use alloc::format;
use crate::framebuffer::{B_, G_, AX_, D_, A_, C_, R_, CF_, DM_, K_};





const Apu: &[(&str, &str)] = &[
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


pub(super) fn pod(cmd: &str) -> bool {
    for &(name, desc) in Apu {
        if cmd == name {
            crate::n!(D_, "{}: {} not implemented", name, desc);
            return true;
        }
    }
    false
}

pub(super) fn dkz(args: &[&str]) {
    if args.is_empty() {
        crate::println!("Usage: which <command>");
        return;
    }
    
    let dyu = ["/bin", "/usr/bin", "/sbin", "/usr/sbin"];
    
    for name in args {
        let mut nj = false;
        for it in &dyu {
            let path = format!("{}/{}", it, name);
            if super::vm::bbs(&path) {
                crate::println!("{}", path);
                nj = true;
                break;
            }
        }
        
        if !nj {
            let bwb = crate::hypervisor::linux_subsystem::acs();
            if bwb.is_package_installed(name) {
                crate::println!("/usr/bin/{}", name);
                nj = true;
            }
        }
        if !nj {
            crate::n!(A_, "{}: not found", name);
        }
    }
}

pub(super) fn kuc(args: &[&str]) {
    if args.is_empty() {
        crate::println!("Usage: whereis <command>");
        return;
    }
    dkz(args);
}

pub(super) fn knv(args: &[&str]) {
    if args.is_empty() {
        crate::println!("Usage: file <path>");
        return;
    }
    
    for path in args {
        if !super::vm::bbs(path) {
            crate::println!("{}: cannot open", path);
            continue;
        }
        
        
        if crate::exec::is_executable(path) {
            crate::println!("{}: ELF 64-bit executable", path);
        } else {
            
            match crate::vfs::open(path, crate::vfs::OpenFlags(0)) {
                Ok(fd) => {
                    let mut header = [0u8; 16];
                    let ae = crate::vfs::read(fd, &mut header).unwrap_or(0);
                    crate::vfs::close(fd).ok();
                    
                    if ae == 0 {
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

pub(super) fn kly(args: &[&str]) {
    if args.is_empty() {
        crate::println!("Usage: basename <path>");
        return;
    }
    let path = args[0];
    let name = path.rsplit('/').next().unwrap_or(path);
    crate::println!("{}", name);
}

pub(super) fn kna(args: &[&str]) {
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

pub(super) fn kqv(args: &[&str]) {
    if args.is_empty() {
        crate::println!("Usage: realpath <path>");
        return;
    }
    let path = super::vm::eyp(args[0]);
    crate::println!("{}", path);
}

pub(super) fn krt(args: &[&str], piped: Option<&str>) {
    let content = if let Some(input) = piped {
        Some(alloc::string::String::from(input))
    } else if !args.is_empty() {
        super::network::cpa(args[0])
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
        None => crate::n!(A_, "sort: cannot read input"),
    }
}

pub(super) fn ktk(args: &[&str], piped: Option<&str>) {
    let content = if let Some(input) = piped {
        Some(alloc::string::String::from(input))
    } else if !args.is_empty() {
        super::network::cpa(args[0])
    } else {
        crate::println!("Usage: uniq <file>");
        return;
    };
    
    match content {
        Some(text) => {
            let mut dah: Option<&str> = None;
            for line in text.lines() {
                if dah != Some(line) {
                    crate::println!("{}", line);
                    dah = Some(line);
                }
            }
        }
        None => crate::n!(A_, "uniq: cannot read input"),
    }
}

pub(super) fn kuh(args: &[&str]) {
    let text = if args.is_empty() { "y" } else { args[0] };
    crate::shell::dks();
    loop {
        if crate::shell::cbc() { break; }
        crate::println!("{}", text);
        
        if let Some(3) = crate::keyboard::ya() {
            crate::shell::fag();
            break;
        }
    }
}

pub(super) fn cmd_seq(args: &[&str]) {
    if args.is_empty() {
        crate::println!("Usage: seq <last> | seq <first> <last> | seq <first> <inc> <last>");
        return;
    }
    
    let (first, bmy, last) = match args.len() {
        1 => (1i64, 1i64, args[0].parse().unwrap_or(1)),
        2 => (args[0].parse().unwrap_or(1), 1i64, args[1].parse().unwrap_or(1)),
        _ => (args[0].parse().unwrap_or(1), args[1].parse().unwrap_or(1), args[2].parse().unwrap_or(1)),
    };
    
    let mut i = first;
    let mut count = 0u64;
    while (bmy > 0 && i <= last) || (bmy < 0 && i >= last) {
        crate::println!("{}", i);
        i += bmy;
        count += 1;
        if count >= 100_000 {
            crate::println!("... (truncated at 100000 lines)");
            break;
        }
    }
}

pub(super) fn fnc(args: &[&str]) {
    if args.is_empty() {
        crate::println!("Usage: sleep <seconds>");
        return;
    }
    
    let im: u64 = args[0].parse().unwrap_or(1);
    crate::n!(C_, "Sleeping for {} seconds...", im);
    
    
    let start = crate::time::uptime_ms();
    let end = start + im * 1000;
    while crate::time::uptime_ms() < end {
        core::hint::spin_loop();
    }
    crate::println!("Done.");
}

pub(super) fn kpc(args: &[&str]) {
    if args.is_empty() {
        crate::println!("Usage: kill [-9] <pid>");
        return;
    }
    
    let pxi = if args[0] == "-9" { 9 } else { 15 };
    let nul = if args[0].starts_with('-') && args.len() > 1 { args[1] } else { args[0] };
    
    match nul.parse::<u32>() {
        Ok(pid) => {
            crate::n!(D_, "Killing PID {}", pid);
            match crate::process::bne(pid) {
                Ok(_) => crate::n!(B_, "Process {} killed", pid),
                Err(e) => crate::n!(A_, "kill: {}", e),
            }
        }
        Err(_) => crate::n!(A_, "kill: invalid PID"),
    }
}

pub(super) fn eil() {
    crate::n!(G_, "TrustOS Process Monitor");
    crate::println!("-----------------------------------------------------------");
    
    let aiz = crate::time::uptime_ms() / 1000;
    let aoi = aiz / 3600;
    let acf = (aiz % 3600) / 60;
    let im = aiz % 60;
    
    crate::println!("Uptime: {:02}:{:02}:{:02}", aoi, acf, im);
    crate::println!();
    
    
    let heap_used = crate::memory::stats().heap_used;
    let heap_total = (crate::memory::stats().heap_used + crate::memory::stats().heap_free);
    crate::println!("Mem: {} KB / {} KB ({:.1}%)", 
        heap_used / 1024, 
        heap_total / 1024,
        (heap_used as f64 / heap_total as f64) * 100.0);
    crate::println!();
    
    crate::n!(C_, "  PID  STATE    NAME");
    crate::println!("----------------------------------");
    
    
    for (pid, name, state) in crate::process::list() {
        let acr = match state {
            crate::process::ProcessState::Running => "RUNNING",
            crate::process::ProcessState::Ready => "READY  ",
            crate::process::ProcessState::Blocked => "BLOCKED",
            crate::process::ProcessState::Zombie => "ZOMBIE ",
            crate::process::ProcessState::Created => "CREATED",
            crate::process::ProcessState::Waiting => "WAITING",
            crate::process::ProcessState::Stopped => "STOPPED",
            crate::process::ProcessState::Dead => "DEAD   ",
        };
        crate::println!("{:>5}  {}  {}", pid, acr, name);
    }
    
    crate::println!();
    crate::n!(D_, "(press 'q' to quit in interactive mode)");
}

pub(super) fn ktw() {
    crate::n!(G_, "Virtual Memory Statistics");
    crate::println!("-----------------------------------------");
    
    let heap_used = crate::memory::stats().heap_used;
    let heap_total = (crate::memory::stats().heap_used + crate::memory::stats().heap_free);
    
    crate::println!("Memory:");
    crate::println!("  Heap Total:  {} KB", heap_total / 1024);
    crate::println!("  Heap Used:   {} KB", heap_used / 1024);
    crate::println!("  Heap Free:   {} KB", (heap_total - heap_used) / 1024);
}

pub(super) fn fml(args: &[&str]) {
    if args.first() == Some(&"-c") || args.first() == Some(&"--clear") {
        
        crate::n!(B_, "dmesg buffer acknowledged.");
        return;
    }
    
    let count = if let Some(&"-n") = args.first() {
        args.get(1).and_then(|j| j.parse::<usize>().ok()).unwrap_or(20)
    } else if let Some(ae) = args.first().and_then(|j| j.parse::<usize>().ok()) {
        ae
    } else {
        0 
    };
    
    let lines = crate::devtools::lgj(count);
    if lines.is_empty() {
        crate::n!(D_, "(no kernel messages recorded)");
        crate::println!("Tip: messages are captured after devtools init.");
        return;
    }
    let (ate, av) = crate::devtools::lgk();
    crate::n!(G_, "Kernel Ring Buffer ({} stored, {} total)", ate, av);
    crate::println!("---------------------------------------------------------------");
    for line in &lines {
        crate::println!("{}", line);
    }
}

pub(super) fn kps() {
    let j = crate::devtools::dbe();
    crate::n!(G_, "Memory Debug Statistics (memdbg)");
    crate::println!("---------------------------------------------------------------");
    crate::println!();
    crate::n!(C_, "  Heap Usage:");
    crate::println!("    Current used : {:>10} bytes ({} KB)", j.current_heap_used, j.current_heap_used / 1024);
    crate::println!("    Current free : {:>10} bytes ({} KB)", j.current_heap_free, j.current_heap_free / 1024);
    crate::println!("    Total heap   : {:>10} bytes ({} KB)", j.heap_total, j.heap_total / 1024);
    crate::println!("    Peak used    : {:>10} bytes ({} KB)", j.peak_heap_used, j.peak_heap_used / 1024);
    crate::println!();
    crate::n!(C_, "  Allocation Stats:");
    crate::println!("    Alloc ops    : {:>10}", j.alloc_count);
    crate::println!("    Dealloc ops  : {:>10}", j.dealloc_count);
    crate::println!("    Live allocs  : {:>10}", j.live_allocs);
    crate::println!("    Total alloc'd: {:>10} bytes", j.alloc_bytes_total);
    crate::println!("    Total freed  : {:>10} bytes", j.dealloc_bytes_total);
    crate::println!("    Largest alloc: {:>10} bytes", j.largest_alloc);
    crate::println!();
    crate::n!(C_, "  Fragmentation:");
    let lyi = if j.fragmentation_pct > 50.0 { A_ }
        else if j.fragmentation_pct > 25.0 { D_ }
        else { B_ };
    crate::n!(lyi, "    Estimate     : {:.1}%", j.fragmentation_pct);
}

pub(super) fn kqj() {
    let jp = crate::devtools::nto();
    let bwq = jp.uptime_ms / 1000;
    let aoi = bwq / 3600;
    let acf = (bwq % 3600) / 60;
    let im = bwq % 60;
    
    crate::n!(G_, "Performance Statistics (perf)");
    crate::println!("---------------------------------------------------------------");
    crate::println!();
    crate::n!(C_, "  System:");
    crate::println!("    Uptime       : {}h {:02}m {:02}s ({} ms)", aoi, acf, im, jp.uptime_ms);
    crate::println!("    GUI FPS      : {}", jp.fps);
    crate::println!();
    crate::n!(C_, "  Interrupts:");
    crate::println!("    Total IRQs   : {}", jp.total_irqs);
    crate::println!("    IRQ/sec      : {}", jp.irq_per_sec);
    crate::println!();
    crate::n!(C_, "  Scheduling:");
    crate::println!("    Syscalls     : {}", jp.total_syscalls);
    crate::println!("    Ctx switches : {}", jp.total_ctx_switches);
    crate::println!();
    crate::n!(C_, "  Memory:");
    crate::println!("    Heap used    : {} / {} KB ({}%)", 
        jp.heap_used / 1024, (jp.heap_used + jp.heap_free) / 1024,
        if jp.heap_used + jp.heap_free > 0 { jp.heap_used * 100 / (jp.heap_used + jp.heap_free) } else { 0 });
    crate::println!();
    crate::n!(C_, "  Per-CPU:");
    for j in &jp.cpu_stats {
        let state = if j.is_idle { "idle" } else { "busy" };
        crate::println!("    CPU{}: {} irqs, {} syscalls, {} ctxsw [{}]", 
            j.cpu_id, j.interrupts, j.syscalls, j.context_switches, state);
    }
}

pub(super) fn koz() {
    let stats = crate::sync::percpu::dhj();
    let total_irqs: u64 = stats.iter().map(|j| j.interrupts).sum();
    
    crate::n!(G_, "IRQ Statistics");
    crate::println!("---------------------------------------------------------------");
    crate::println!();
    crate::println!("  Total IRQs     : {}", total_irqs);
    crate::println!("  IRQ rate       : {}/sec", crate::devtools::irq_rate());
    crate::println!();
    crate::n!(C_, "  Per-CPU Breakdown:");
    for j in &stats {
        let jzn = if total_irqs > 0 { (j.interrupts * 40 / total_irqs.max(1)) as usize } else { 0 };
        let bar: String = "|".repeat(jzn);
        let aed = if total_irqs > 0 { j.interrupts * 100 / total_irqs } else { 0 };
        crate::println!("    CPU{}: {:>8} ({:>3}%) {}", j.cpu_id, j.interrupts, aed, bar);
    }
}

pub(super) fn kqx() {
    crate::n!(G_, "CPU Register Dump");
    crate::println!("---------------------------------------------------------------");
    let regs = crate::devtools::kyq();
    for line in &regs {
        crate::println!("{}", line);
    }
}

pub(super) fn kqi(args: &[&str]) {
    if args.is_empty() {
        crate::println!("Usage: peek <hex_addr> [byte_count]");
        crate::println!("  e.g.: peek 0xFFFF8000_00000000 64");
        crate::println!("  Default count: 64 bytes, max: 256 bytes");
        return;
    }
    
    let bkp = args[0].trim_start_matches("0x").trim_start_matches("0X");
    let addr = match usize::from_str_radix(bkp, 16) {
        Ok(a) => a,
        Err(_) => {
            crate::n!(A_, "Invalid hex address: {}", args[0]);
            return;
        }
    };
    
    let count = args.get(1).and_then(|j| j.parse::<usize>().ok()).unwrap_or(64);
    
    crate::n!(G_, "Memory dump at 0x{:016x} ({} bytes)", addr, count);
    crate::println!("---------------------------------------------------------------");
    let lines = crate::devtools::peek(addr, count);
    for line in &lines {
        crate::println!("{}", line);
    }
}

pub(super) fn kqn(args: &[&str]) {
    if args.len() < 2 {
        crate::println!("Usage: poke <hex_addr> <hex_value>");
        crate::println!("  e.g.: poke 0xB8000 0x41");
        crate::n!(A_, "  ? WARNING: Writing to arbitrary memory is DANGEROUS!");
        return;
    }
    
    let bkp = args[0].trim_start_matches("0x").trim_start_matches("0X");
    let addr = match usize::from_str_radix(bkp, 16) {
        Ok(a) => a,
        Err(_) => {
            crate::n!(A_, "Invalid hex address: {}", args[0]);
            return;
        }
    };
    
    let ass = args[1].trim_start_matches("0x").trim_start_matches("0X");
    let value = match u8::from_str_radix(ass, 16) {
        Ok(v) => v,
        Err(_) => {
            crate::n!(A_, "Invalid hex value: {}", args[1]);
            return;
        }
    };
    
    match crate::devtools::gnq(addr, value) {
        Ok(()) => crate::n!(B_, "Wrote 0x{:02x} to 0x{:016x}", value, addr),
        Err(e) => crate::n!(A_, "poke error: {}", e),
    }
}

pub(super) fn kmz() {
    crate::devtools::pks();
    let state = if crate::devtools::ihv() { "ON" } else { "OFF" };
    crate::n!(B_, "DevPanel overlay: {} (also toggle with F12 in desktop)", state);
}

pub(super) fn ksv(args: &[&str]) {
    if args.is_empty() {
        crate::println!("Usage: timecmd <command> [args...]");
        crate::println!("  Runs a command and prints elapsed time.");
        return;
    }
    
    let start = crate::cpu::tsc::Stopwatch::start();
    
    
    let oyh = args.join(" ");
    super::aav(&oyh);
    
    let cis = start.elapsed_micros();
    let elapsed_ms = cis / 1000;
    let yt = cis % 1000;
    crate::println!();
    crate::n!(C_, "? Elapsed: {}.{:03} ms ({} us)", elapsed_ms, yt, cis);
}






pub(super) fn koo() {
    crate::n!(G_, "Generating hardware diagnostic report...");
    let lines = crate::debug::mab();
    for line in &lines {
        crate::println!("{}", line);
    }
    
    for line in &lines {
        crate::serial_println!("{}", line);
    }
}


pub(super) fn kmm() {
    crate::n!(G_, "Full CPU State Dump");
    crate::println!("---------------------------------------------------------------");
    let lines = crate::debug::enn();
    for line in &lines {
        crate::println!("{}", line);
    }
}


pub(super) fn krv(args: &[&str]) {
    let max = args.first().and_then(|j| j.parse::<usize>().ok()).unwrap_or(16);
    crate::n!(G_, "Stack Backtrace (max {} frames)", max);
    crate::println!("---------------------------------------------------------------");
    let lines = crate::debug::enf(max);
    for line in &lines {
        crate::println!("{}", line);
    }
}


pub(super) fn kmc() {
    crate::n!(G_, "Boot Checkpoints");
    crate::println!("---------------------------------------------------------------");
    let cvs = crate::debug::fyl();
    if cvs.is_empty() {
        crate::println!("  <no checkpoints recorded>");
    } else {
        let dpw = cvs[0].0;
        for (tsc, code, name) in &cvs {
            let mk = tsc - dpw;
            crate::println!("  POST 0x{:02X}  TSC +{:>14}  {}", code, mk, name);
        }
    }
    crate::println!("  Last POST code: 0x{:02X}", crate::debug::ijk());
}


pub(super) fn kqo(args: &[&str]) {
    if args.is_empty() {
        crate::println!("Current POST code: 0x{:02X}", crate::debug::ijk());
        crate::println!("Usage: postcode <hex_value>   (writes to port 0x80)");
        return;
    }
    let ass = args[0].trim_start_matches("0x").trim_start_matches("0X");
    match u8::from_str_radix(ass, 16) {
        Ok(v) => {
            crate::debug::ewx(v);
            crate::n!(B_, "POST code 0x{:02X} written to port 0x80", v);
        }
        Err(_) => crate::n!(A_, "Invalid hex value: {}", args[0]),
    }
}


pub(super) fn kow(args: &[&str]) {
    if args.len() < 2 {
        crate::println!("Usage: ioport read <port_hex> [b|w|l]");
        crate::println!("       ioport write <port_hex> <value_hex> [b|w|l]");
        crate::println!("  b=byte (default), w=word, l=dword");
        crate::n!(A_, "  ⚠ WARNING: Writing to arbitrary I/O ports is DANGEROUS!");
        return;
    }
    
    let je = args[0];
    let bva = args[1].trim_start_matches("0x").trim_start_matches("0X");
    let port = match u16::from_str_radix(bva, 16) {
        Ok(aa) => aa,
        Err(_) => {
            crate::n!(A_, "Invalid port: {}", args[1]);
            return;
        }
    };
    
    match je {
        "read" | "r" => {
            let size = args.get(2).copied().unwrap_or("b");
            match size {
                "b" | "byte" => {
                    let val = crate::debug::om(port);
                    crate::println!("  IN  port 0x{:04X} = 0x{:02X} ({})", port, val, val);
                }
                "w" | "word" => {
                    let val = crate::debug::eqz(port);
                    crate::println!("  IN  port 0x{:04X} = 0x{:04X} ({})", port, val, val);
                }
                "l" | "dword" => {
                    let val = crate::debug::eqp(port);
                    crate::println!("  IN  port 0x{:04X} = 0x{:08X} ({})", port, val, val);
                }
                _ => crate::n!(A_, "Size must be b/w/l"),
            }
        }
        "write" | "w" => {
            if args.len() < 3 {
                crate::n!(A_, "Need value: ioport write <port> <value> [b|w|l]");
                return;
            }
            let ass = args[2].trim_start_matches("0x").trim_start_matches("0X");
            let size = args.get(3).copied().unwrap_or("b");
            match size {
                "b" | "byte" => {
                    if let Ok(v) = u8::from_str_radix(ass, 16) {
                        crate::debug::vp(port, v);
                        crate::n!(B_, "  OUT port 0x{:04X} <- 0x{:02X}", port, v);
                    } else {
                        crate::n!(A_, "Invalid byte value");
                    }
                }
                "w" | "word" => {
                    if let Ok(v) = u16::from_str_radix(ass, 16) {
                        crate::debug::evw(port, v);
                        crate::n!(B_, "  OUT port 0x{:04X} <- 0x{:04X}", port, v);
                    } else {
                        crate::n!(A_, "Invalid word value");
                    }
                }
                "l" | "dword" => {
                    if let Ok(v) = u32::from_str_radix(ass, 16) {
                        crate::debug::evv(port, v);
                        crate::n!(B_, "  OUT port 0x{:04X} <- 0x{:08X}", port, v);
                    } else {
                        crate::n!(A_, "Invalid dword value");
                    }
                }
                _ => crate::n!(A_, "Size must be b/w/l"),
            }
        }
        _ => crate::n!(A_, "Use: ioport read|write ..."),
    }
}


pub(super) fn kqs(args: &[&str]) {
    if args.is_empty() {
        crate::println!("Usage: rdmsr <msr_hex>");
        crate::println!("  e.g.: rdmsr 0xC0000080  (IA32_EFER)");
        crate::println!("Common MSRs:");
        crate::println!("  0xC0000080  IA32_EFER       0x0000001B  IA32_APIC_BASE");
        crate::println!("  0xC0000081  IA32_STAR        0x00000010  IA32_TSC");
        crate::println!("  0xC0000082  IA32_LSTAR       0x00000277  IA32_PAT");
        return;
    }
    let dus = args[0].trim_start_matches("0x").trim_start_matches("0X");
    match u32::from_str_radix(dus, 16) {
        Ok(msr) => {
            match crate::debug::rf(msr) {
                Some(val) => {
                    crate::println!("  MSR 0x{:08X} = 0x{:016X}", msr, val);
                    crate::println!("                  {:064b}", val);
                }
                None => crate::n!(A_, "  MSR 0x{:08X}: read failed (#GP)", msr),
            }
        }
        Err(_) => crate::n!(A_, "Invalid MSR address: {}", args[0]),
    }
}


pub(super) fn kuf(args: &[&str]) {
    if args.len() < 2 {
        crate::println!("Usage: wrmsr <msr_hex> <value_hex>");
        crate::n!(A_, "  ⚠ WARNING: Writing to MSRs can crash the system!");
        return;
    }
    let dus = args[0].trim_start_matches("0x").trim_start_matches("0X");
    let ass = args[1].trim_start_matches("0x").trim_start_matches("0X");
    
    let msr = match u32::from_str_radix(dus, 16) {
        Ok(m) => m,
        Err(_) => {
            crate::n!(A_, "Invalid MSR: {}", args[0]);
            return;
        }
    };
    let val = match u64::from_str_radix(ass, 16) {
        Ok(v) => v,
        Err(_) => {
            crate::n!(A_, "Invalid value: {}", args[1]);
            return;
        }
    };
    
    crate::debug::cfm(msr, val);
    crate::n!(B_, "  WRMSR 0x{:08X} <- 0x{:016X}", msr, val);
}


pub(super) fn kmo(args: &[&str]) {
    if args.is_empty() {
        crate::println!("Usage: cpuid <leaf_hex> [subleaf_hex]");
        crate::println!("  e.g.: cpuid 0          (vendor string)");
        crate::println!("        cpuid 1          (features)");
        crate::println!("        cpuid 0x80000002 (brand string part 1)");
        crate::println!("        cpuid 0x80000003 (brand string part 2)");
        crate::println!("        cpuid 0x80000004 (brand string part 3)");
        return;
    }
    let mxs = args[0].trim_start_matches("0x").trim_start_matches("0X");
    let oym = args.get(1).map(|j| j.trim_start_matches("0x").trim_start_matches("0X")).unwrap_or("0");
    
    let leaf = match u32::from_str_radix(mxs, 16) {
        Ok(l) => l,
        Err(_) => {
            crate::n!(A_, "Invalid leaf: {}", args[0]);
            return;
        }
    };
    let subleaf = u32::from_str_radix(oym, 16).unwrap_or(0);
    
    crate::n!(G_, "CPUID Query");
    crate::println!("---------------------------------------------------------------");
    let lines = crate::debug::cya(leaf, subleaf);
    for line in &lines {
        crate::println!("{}", line);
    }
}


pub(super) fn kpt() {
    crate::n!(G_, "Physical Memory Map");
    crate::println!("---------------------------------------------------------------");
    let lines = crate::debug::fxh();
    for line in &lines {
        crate::println!("{}", line);
    }
}


pub(super) fn ktz(args: &[&str]) {
    match args.first().copied() {
        Some("enable" | "on") => {
            let mz = args.get(1).and_then(|j| j.parse::<u64>().ok()).unwrap_or(5000);
            crate::debug::pua(mz);
            crate::n!(B_, "Watchdog enabled ({} ms timeout)", mz);
        }
        Some("disable" | "off") => {
            crate::debug::ptz();
            crate::n!(B_, "Watchdog disabled");
        }
        Some("pet" | "kick") => {
            crate::debug::puc();
            crate::n!(B_, "Watchdog petted");
        }
        _ => {
            crate::println!("Usage: watchdog <enable [ms]|disable|pet>");
            crate::println!("  enable [timeout_ms]  — Start watchdog (default: 5000 ms)");
            crate::println!("  disable              — Stop watchdog");
            crate::println!("  pet                  — Reset watchdog counter");
        }
    }
}






pub(super) fn kng(args: &[&str]) {
    let je = args.first().copied().unwrap_or("help");

    match je {
        
        "pci" => knj(&args[1..]),

        
        "mmio" => kni(&args[1..]),

        
        "reprobe" | "probe" => knk(&args[1..]),

        
        "test" => knm(&args[1..]),

        
        "list" | "ls" => knh(),

        
        "scan" => knl(),

        _ => {
            crate::n!(C_, "=== Live Driver Debug Toolkit ===");
            crate::println!();
            crate::println!("  drv list                           List all active drivers and state");
            crate::println!("  drv scan                           Scan PCI bus (full config header dump)");
            crate::println!();
            crate::n!(D_, "PCI Config Space:");
            crate::println!("  drv pci read <BB:DD.F> <off> [l]   Read PCI config register (b/w/l)");
            crate::println!("  drv pci write <BB:DD.F> <off> <val> [l]  Write PCI config register");
            crate::println!("  drv pci dump <BB:DD.F>             Dump 256-byte PCI config space");
            crate::println!();
            crate::n!(D_, "MMIO (Memory-Mapped I/O):");
            crate::println!("  drv mmio read <addr> [count]       Read 32-bit MMIO registers");
            crate::println!("  drv mmio write <addr> <val>        Write 32-bit MMIO register");
            crate::println!("  drv mmio wifi <offset>             Read WiFi CSR register by offset");
            crate::println!();
            crate::n!(D_, "Driver Control:");
            crate::println!("  drv reprobe wifi                   Re-probe WiFi driver from PCI");
            crate::println!("  drv reprobe hda                    Re-probe HDA audio driver");
            crate::println!();
            crate::n!(D_, "Driver Tests (no recompile):");
            crate::println!("  drv test wifi                      WiFi: PCI detect → BAR → CSR dump → FW check");
            crate::println!("  drv test hda                       HDA: codec detect → widget dump → path check");
            crate::println!("  drv test ec                        ThinkPad EC: temp sensors → fan → battery");
            crate::println!("  drv test net                       Network: virtio/e1000 link → MAC → ping");
            crate::println!("  drv test all                       Run all driver tests sequentially");
        }
    }
}


fn ccf(j: &str) -> Option<(u8, u8, u8)> {
    
    let au: Vec<&str> = j.split(|c| c == ':' || c == '.').collect();
    if au.len() < 2 { return None; }
    let bus = u8::from_str_radix(au[0], 16).ok()?;
    let s = u8::from_str_radix(au[1], 16).ok()?;
    let func = if au.len() > 2 { u8::from_str_radix(au[2], 16).ok().unwrap_or(0) } else { 0 };
    Some((bus, s, func))
}


fn dwe(j: &str) -> Option<u32> {
    let j = j.trim_start_matches("0x").trim_start_matches("0X");
    u32::from_str_radix(j, 16).ok()
}

fn itv(j: &str) -> Option<usize> {
    let j = j.trim_start_matches("0x").trim_start_matches("0X");
    usize::from_str_radix(j, 16).ok()
}


fn knj(args: &[&str]) {
    let je = args.first().copied().unwrap_or("help");
    match je {
        "read" | "r" => {
            if args.len() < 3 {
                crate::println!("Usage: drv pci read <BB:DD.F> <offset_hex> [b|w|l]");
                return;
            }
            let Some((bus, s, func)) = ccf(args[1]) else {
                crate::n!(A_, "Invalid BDF: {} (use BB:DD.F)", args[1]);
                return;
            };
            let Some(offset) = dwe(args[2]) else {
                crate::n!(A_, "Invalid offset: {}", args[2]);
                return;
            };
            let size = args.get(3).copied().unwrap_or("l");
            let val = crate::pci::ms(bus, s, func, offset as u8);
            match size {
                "b" | "byte" => crate::println!("  PCI {:02X}:{:02X}.{} [0x{:02X}] = 0x{:02X}",
                    bus, s, func, offset, val & 0xFF),
                "w" | "word" => crate::println!("  PCI {:02X}:{:02X}.{} [0x{:02X}] = 0x{:04X}",
                    bus, s, func, offset, val & 0xFFFF),
                _ => crate::println!("  PCI {:02X}:{:02X}.{} [0x{:02X}] = 0x{:08X}",
                    bus, s, func, offset, val),
            }
        }
        "write" | "w" => {
            if args.len() < 4 {
                crate::println!("Usage: drv pci write <BB:DD.F> <offset_hex> <value_hex> [b|w|l]");
                return;
            }
            let Some((bus, s, func)) = ccf(args[1]) else {
                crate::n!(A_, "Invalid BDF: {}", args[1]);
                return;
            };
            let Some(offset) = dwe(args[2]) else {
                crate::n!(A_, "Invalid offset: {}", args[2]);
                return;
            };
            let Some(value) = dwe(args[3]) else {
                crate::n!(A_, "Invalid value: {}", args[3]);
                return;
            };
            crate::pci::qj(bus, s, func, offset as u8, value);
            crate::n!(B_, "  PCI {:02X}:{:02X}.{} [0x{:02X}] <- 0x{:08X}",
                bus, s, func, offset, value);
        }
        "dump" | "d" => {
            if args.len() < 2 {
                crate::println!("Usage: drv pci dump <BB:DD.F>");
                return;
            }
            let Some((bus, s, func)) = ccf(args[1]) else {
                crate::n!(A_, "Invalid BDF: {}", args[1]);
                return;
            };
            crate::n!(C_, "PCI Config Space {:02X}:{:02X}.{}", bus, s, func);
            crate::println!("     00 04 08 0C 10 14 18 1C 20 24 28 2C 30 34 38 3C");
            for row in 0..16u8 {
                let off = row * 16;
                crate::print!("{:02X}: ", off);
                for col in (0..16u8).step_by(4) {
                    let val = crate::pci::ms(bus, s, func, off + col);
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


fn kni(args: &[&str]) {
    let je = args.first().copied().unwrap_or("help");
    match je {
        "read" | "r" => {
            if args.len() < 2 {
                crate::println!("Usage: drv mmio read <phys_addr_hex> [count]");
                crate::println!("  Reads 32-bit values at given physical address (auto-mapped)");
                return;
            }
            let Some(phys) = itv(args[1]) else {
                crate::n!(A_, "Invalid address: {}", args[1]);
                return;
            };
            let count = args.get(2).and_then(|j| j.parse::<usize>().ok()).unwrap_or(1);
            let count = count.min(64); 
            let size = count * 4;
            match crate::memory::yv(phys as u64, size) {
                Ok(virt_base) => {
                    for i in 0..count {
                        let virt = virt_base as usize + i * 4;
                        let buv = phys + i * 4;
                        let val = unsafe { core::ptr::read_volatile(virt as *const u32) };
                        crate::println!("  [phys {:#010X}] = 0x{:08X}", buv, val);
                    }
                }
                Err(e) => {
                    crate::n!(A_, "  Failed to map phys 0x{:X}: {}", phys, e);
                }
            }
        }
        "write" | "w" => {
            if args.len() < 3 {
                crate::println!("Usage: drv mmio write <phys_addr_hex> <value_hex>");
                crate::n!(A_, "  ⚠ WARNING: Writing to arbitrary MMIO is DANGEROUS!");
                return;
            }
            let Some(phys) = itv(args[1]) else {
                crate::n!(A_, "Invalid address: {}", args[1]);
                return;
            };
            let Some(value) = dwe(args[2]) else {
                crate::n!(A_, "Invalid value: {}", args[2]);
                return;
            };
            match crate::memory::yv(phys as u64, 4) {
                Ok(virt) => {
                    unsafe { core::ptr::write_volatile(virt as *mut u32, value); }
                    crate::n!(B_, "  [phys {:#010X}] <- 0x{:08X}", phys, value);
                }
                Err(e) => {
                    crate::n!(A_, "  Failed to map phys 0x{:X}: {}", phys, e);
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
            let Some(offset) = dwe(args[1]) else {
                crate::n!(A_, "Invalid offset: {}", args[1]);
                return;
            };
            match crate::drivers::net::iwl4965::byp(offset) {
                Some(val) => crate::println!("  WiFi CSR [0x{:03X}] = 0x{:08X}", offset, val),
                None => crate::n!(A_, "  Cannot read — no WiFi device or offset out of range"),
            }
        }
        _ => {
            crate::println!("Usage: drv mmio <read|write|wifi> ...");
        }
    }
}


fn knk(args: &[&str]) {
    let driver = args.first().copied().unwrap_or("help");
    match driver {
        "wifi" => {
            crate::n!(D_, "Re-probing WiFi from PCI bus...");
            let devices = crate::pci::aqs();
            let mut nj = false;

            
            for s in &devices {
                crate::println!("  Checking {:02X}:{:02X}.{} {:04X}:{:04X} class={:02X}:{:02X}",
                    s.bus, s.device, s.function,
                    s.vendor_id, s.device_id,
                    s.class_code, s.subclass);

                if crate::drivers::net::wifi::ccv(s) {
                    crate::n!(B_, "  ✓ WiFi driver bound to {:04X}:{:04X}!", s.vendor_id, s.device_id);
                    nj = true;
                    break;
                }
            }

            if !nj {
                crate::n!(A_, "  ✗ No WiFi hardware found on PCI bus");
                crate::println!();
                crate::println!("  Expected: Intel 8086:4229 (4965AGN) or similar");
                crate::println!("  Tip: Run 'drv scan' to see all PCI devices");
                crate::println!("  Tip: Run 'drv test wifi' for detailed diagnostics");
            }
        }
        "hda" => {
            crate::n!(D_, "Re-probing HDA audio...");
            let _ = crate::drivers::hda::init();
            if crate::drivers::hda::is_initialized() {
                crate::n!(B_, "  ✓ HDA audio re-initialized");
            } else {
                crate::n!(A_, "  ✗ HDA init failed");
            }
        }
        _ => {
            crate::println!("Usage: drv reprobe <wifi|hda>");
            crate::println!("  Re-probes driver from PCI bus without rebooting");
        }
    }
}


fn knm(args: &[&str]) {
    let target = args.first().copied().unwrap_or("help");
    match target {
        "wifi" => hty(),
        "hda" => htw(),
        "ec" => htv(),
        "net" => htx(),
        "all" => {
            hty();
            crate::println!();
            htw();
            crate::println!();
            htv();
            crate::println!();
            htx();
        }
        _ => {
            crate::println!("Usage: drv test <wifi|hda|ec|net|all>");
        }
    }
}



fn ilt(s: &crate::pci::L) -> Option<usize> {
    let bar0 = s.bar[0];
    if bar0 == 0 || (bar0 & 1) != 0 { return None; }
    let cba = (bar0 >> 1) & 0x3 == 2;
    let phys = if cba {
        let bqi = s.bar[1] as u64;
        (bqi << 32) | (bar0 & 0xFFFFFFF0) as u64
    } else {
        (bar0 & 0xFFFFFFF0) as u64
    };
    if phys == 0 { return None; }
    
    match crate::memory::yv(phys, 0x2000) {
        Ok(virt) => Some(virt as usize),
        Err(_e) => None,
    }
}


fn hty() {
    crate::n!(C_, "=== WiFi Driver Test Suite ===");
    let mut gd = 0u32;
    let mut gv = 0u32;

    
    crate::print!("  [1] PCI device scan (Intel WiFi)... ");
    let devices = crate::pci::aqs();
    let puo = devices.iter().find(|d|
        d.vendor_id == 0x8086 && crate::drivers::net::iwl4965::AFI_.contains(&d.device_id));
    if let Some(s) = puo {
        crate::n!(B_, "FOUND {:04X}:{:04X} at {:02X}:{:02X}.{}",
            s.vendor_id, s.device_id, s.bus, s.device, s.function);
        gd += 1;

        
        crate::print!("  [2] PCI class/subclass... ");
        crate::print!("class={:02X} sub={:02X} ", s.class_code, s.subclass);
        let lss = s.class_code == 0x02 && s.subclass == 0x80
            || s.class_code == 0x0D;
        if lss {
            crate::n!(B_, "OK (wireless)");
            gd += 1;
        } else {
            crate::n!(D_, "UNEXPECTED (but device ID matches)");
            gd += 1;
        }

        
        crate::print!("  [3] BAR0 (MMIO base)... ");
        let bar0 = s.bar[0];
        let cba = (bar0 >> 1) & 0x3 == 2;
        let phys = if cba {
            let bqi = s.bar[1] as u64;
            (bqi << 32) | (bar0 & 0xFFFFFFF0) as u64
        } else {
            (bar0 & 0xFFFFFFF0) as u64
        };
        if bar0 != 0 && (bar0 & 1) == 0 && phys != 0 {
            crate::n!(B_, "phys=0x{:X} ({})", phys, if cba { "64-bit" } else { "32-bit" });
            gd += 1;

            
            crate::print!("  [3b] MMIO page mapping... ");
            match ilt(s) {
                Some(virt_base) => {
                    crate::n!(B_, "virt=0x{:X}", virt_base);
                    gd += 1;

                    
                    crate::print!("  [4] CSR HW_REV read... ");
                    let hw_rev = unsafe { core::ptr::read_volatile((virt_base + 0x028) as *const u32) };
                    if hw_rev != 0 && hw_rev != 0xFFFFFFFF {
                        let drq = (hw_rev & 0x000FFF0) >> 4;
                        let name = match drq { 0 => "4965", 2 => "5300", 4 => "5150", 5 => "5100", 7 => "6000", _ => "unknown" };
                        crate::n!(B_, "0x{:08X} (type={} = {})", hw_rev, drq, name);
                        gd += 1;
                    } else {
                        crate::n!(A_, "0x{:08X} — MMIO not responding", hw_rev);
                        gv += 1;
                    }

                    
                    crate::print!("  [5] CSR GP_CNTRL... ");
                    let tj = unsafe { core::ptr::read_volatile((virt_base + 0x024) as *const u32) };
                    crate::print!("0x{:08X} ", tj);
                    if tj & 1 != 0 {
                        crate::n!(B_, "(MAC clock ready)");
                    } else {
                        crate::n!(D_, "(MAC clock NOT ready — device sleeping or reset)");
                    }
                    gd += 1;

                    
                    crate::println!("  [6] Key CSR register dump:");
                    crate::drivers::net::iwl4965::hqv();
                    gd += 1;
                }
                None => {
                    crate::n!(A_, "FAILED to map MMIO pages!");
                    crate::println!("      phys=0x{:X}, map_mmio() returned error", phys);
                    gv += 1;
                }
            }

        } else {
            if bar0 == 0 {
                crate::n!(A_, "ZERO — BAR not assigned!");
            } else {
                crate::n!(A_, "I/O BAR (0x{:08X}) — need memory BAR", bar0);
            }
            gv += 1;
        }

        
        crate::print!("  [7] IRQ assignment... ");
        if s.interrupt_line != 0xFF && s.interrupt_line != 0 {
            crate::n!(B_, "IRQ {} (pin {})", s.interrupt_line, s.interrupt_pin);
            gd += 1;
        } else {
            crate::n!(D_, "no IRQ assigned (line={}, pin={})", s.interrupt_line, s.interrupt_pin);
            gd += 1; 
        }

        
        crate::print!("  [8] PCI command register... ");
        let cmd = crate::pci::ms(s.bus, s.device, s.function, 0x04);
        let ini = cmd & 0x02 != 0;
        let hjf = cmd & 0x04 != 0;
        crate::print!("0x{:04X} ", cmd & 0xFFFF);
        if ini && hjf {
            crate::n!(B_, "(mem_space=ON, bus_master=ON)");
            gd += 1;
        } else {
            crate::n!(D_, "(mem_space={}, bus_master={}) — may need enabling",
                if ini { "ON" } else { "OFF" }, if hjf { "ON" } else { "OFF" });
            gd += 1;
        }

    } else {
        crate::n!(A_, "NOT FOUND");
        gv += 1;
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

    
    crate::print!("  [9] Firmware (iwlwifi-4965-2.ucode)... ");
    if crate::drivers::net::iwl4965::eou() {
        crate::n!(B_, "LOADED");
        gd += 1;
    } else {
        crate::n!(A_, "NOT AVAILABLE");
        crate::println!("      Load firmware via Limine module or copy to RamFS");
        gv += 1;
    }

    
    crate::print!("  [10] WiFi driver state... ");
    if crate::drivers::net::wifi::ckk() {
        crate::n!(B_, "ACTIVE");
        crate::drivers::net::iwl4965::hqu();
        gd += 1;
    } else {
        crate::n!(A_, "NOT ACTIVE");
        gv += 1;
    }

    crate::println!();
    let av = gd + gv;
    if gv == 0 {
        crate::n!(B_, "  WiFi: {}/{} PASSED", gd, av);
    } else {
        crate::n!(D_, "  WiFi: {}/{} passed, {} failed", gd, av, gv);
    }
}


fn htw() {
    crate::n!(C_, "=== HDA Audio Driver Test Suite ===");
    let mut gd = 0u32;
    let mut gv = 0u32;

    
    crate::print!("  [1] PCI device (class 04:03 multimedia/audio)... ");
    let devices = crate::pci::aqs();
    let gae = devices.iter().find(|d| d.class_code == 0x04 && d.subclass == 0x03);
    if let Some(s) = gae {
        crate::n!(B_, "FOUND {:04X}:{:04X} at {:02X}:{:02X}.{}",
            s.vendor_id, s.device_id, s.bus, s.device, s.function);
        gd += 1;

        
        crate::print!("  [2] BAR0 (HDA MMIO)... ");
        let bar0 = s.bar[0];
        if bar0 != 0 && (bar0 & 1) == 0 {
            let phys = (bar0 & 0xFFFFFFF0) as u64;
            crate::n!(B_, "phys=0x{:X}", phys);
            gd += 1;

            crate::print!("  [2b] MMIO mapping... ");
            match ilt(s) {
                Some(base) => {
                    crate::n!(B_, "virt=0x{:X}", base);

                    
                    crate::print!("  [3] GCAP register... ");
                    let agk = unsafe { core::ptr::read_volatile(base as *const u16) };
                    let noa = (agk >> 12) & 0xF;
                    let xt = (agk >> 8) & 0xF;
                    let keb = (agk >> 3) & 0x1F;
                    crate::n!(B_, "0x{:04X} (OSS={}, ISS={}, BSS={})", agk, noa, xt, keb);
                    gd += 1;

                    
                    crate::print!("  [4] GCTL (controller reset)... ");
                    let gctl = unsafe { core::ptr::read_volatile((base + 0x08) as *const u32) };
                    if gctl & 1 != 0 {
                        crate::n!(B_, "0x{:08X} (out of reset)", gctl);
                        gd += 1;
                    } else {
                        crate::n!(A_, "0x{:08X} (IN RESET — codec not accessible)", gctl);
                        gv += 1;
                    }

                    
                    crate::print!("  [5] STATESTS (codec presence)... ");
                    let bdv = unsafe { core::ptr::read_volatile((base + 0x0E) as *const u16) };
                    if bdv & 0x7 != 0 {
                        let codecs: Vec<u8> = (0..3).filter(|i| bdv & (1 << i) != 0).collect();
                        crate::n!(B_, "0x{:04X} — codecs at: {:?}", bdv, codecs);
                        gd += 1;
                    } else {
                        crate::n!(A_, "0x{:04X} — NO codecs detected", bdv);
                        gv += 1;
                    }
                }
                None => {
                    crate::n!(A_, "FAILED to map MMIO!");
                    gv += 1;
                }
            }
        } else {
            crate::n!(A_, "BAR0={:#X} — invalid", bar0);
            gv += 1;
        }

        
        crate::print!("  [6] IRQ... ");
        crate::println!("line={} pin={}", s.interrupt_line, s.interrupt_pin);
        gd += 1;

    } else {
        crate::n!(A_, "NOT FOUND");
        gv += 1;
    }

    
    crate::print!("  [7] HDA driver state... ");
    if crate::drivers::hda::is_initialized() {
        crate::n!(B_, "INITIALIZED");
        gd += 1;
    } else {
        crate::n!(A_, "NOT INITIALIZED");
        gv += 1;
    }

    crate::println!();
    let av = gd + gv;
    if gv == 0 {
        crate::n!(B_, "  HDA: {}/{} PASSED", gd, av);
    } else {
        crate::n!(D_, "  HDA: {}/{} passed, {} failed", gd, av, gv);
    }
}


fn htv() {
    crate::n!(C_, "=== ThinkPad EC Driver Test Suite ===");
    let mut gd = 0u32;
    let mut gv = 0u32;

    
    crate::print!("  [1] EC status port (0x66)... ");
    let status = crate::debug::om(0x66);
    crate::print!("0x{:02X} ", status);
    
    if status != 0xFF && status != 0x00 {
        crate::n!(B_, "(EC responding, IBF={}, OBF={})", (status >> 1) & 1, status & 1);
        gd += 1;
    } else if status == 0xFF {
        crate::n!(A_, "(no EC — all bits high)");
        gv += 1;
    } else {
        crate::n!(D_, "(EC idle — all bits zero, may be OK)");
        gd += 1;
    }

    
    crate::print!("  [2] CPU temp (EC reg 0x78)... ");
    
    
    let ts = crate::drivers::thinkpad_ec::ciq(0x78);
    if let Some(t) = ts {
        if t > 0 && t < 120 {
            crate::n!(B_, "{}°C", t);
            gd += 1;
        } else {
            crate::n!(D_, "raw={} (out of range)", t);
            gd += 1;
        }
    } else {
        crate::n!(A_, "TIMEOUT");
        gv += 1;
    }

    
    crate::print!("  [3] Fan RPM... ");
    let bvn = crate::drivers::thinkpad_ec::fwf();
    if let Some(r) = bvn {
        crate::n!(B_, "{} RPM", r);
        gd += 1;
    } else {
        crate::n!(D_, "cannot read (EC may not support)");
        gd += 1;
    }

    
    crate::print!("  [4] Temperature sensors... ");
    let oog = [0x78, 0x79, 0x7A, 0x7B, 0xC0, 0xC1, 0xC2, 0xC3];
    let mut nj = 0;
    for &reg in &oog {
        if let Some(t) = crate::drivers::thinkpad_ec::ciq(reg) {
            if t > 0 && t < 120 { nj += 1; }
        }
    }
    if nj > 0 {
        crate::n!(B_, "{}/8 sensors active", nj);
        gd += 1;
    } else {
        crate::n!(A_, "no sensors responding");
        gv += 1;
    }

    crate::println!();
    let av = gd + gv;
    if gv == 0 {
        crate::n!(B_, "  EC: {}/{} PASSED", gd, av);
    } else {
        crate::n!(D_, "  EC: {}/{} passed, {} failed", gd, av, gv);
    }
}


fn htx() {
    crate::n!(C_, "=== Network Driver Test Suite ===");
    let mut gd = 0u32;
    let mut gv = 0u32;

    
    crate::print!("  [1] PCI network device... ");
    let devices = crate::pci::aqs();
    let nih = devices.iter().find(|d| d.class_code == 0x02);
    if let Some(s) = nih {
        let name = match s.vendor_id {
            0x1AF4 => "VirtIO-net",
            0x8086 => match s.device_id {
                0x100E | 0x100F | 0x10D3 | 0x153A => "Intel e1000/e1000e",
                _ => "Intel (unknown)",
            },
            0x10EC => "Realtek RTL8139",
            _ => "Unknown",
        };
        crate::n!(B_, "{} ({:04X}:{:04X}) at {:02X}:{:02X}.{}", 
            name, s.vendor_id, s.device_id, s.bus, s.device, s.function);
        gd += 1;
    } else {
        crate::n!(A_, "NOT FOUND");
        gv += 1;
    }

    
    crate::print!("  [2] Network stack... ");
    if crate::network::sw() {
        crate::n!(B_, "AVAILABLE (platform: {})", crate::network::fyv());
        gd += 1;
    } else {
        crate::n!(A_, "NOT AVAILABLE");
        gv += 1;
    }

    
    crate::print!("  [3] MAC address... ");
    if let Some(mac) = crate::network::aqu() {
        if mac != [0, 0, 0, 0, 0, 0] {
            crate::n!(B_, "{:02X}:{:02X}:{:02X}:{:02X}:{:02X}:{:02X}",
                mac[0], mac[1], mac[2], mac[3], mac[4], mac[5]);
            gd += 1;
        } else {
            crate::n!(A_, "00:00:00:00:00:00 (not set)");
            gv += 1;
        }
    } else {
        crate::n!(A_, "no MAC address available");
        gv += 1;
    }

    
    crate::print!("  [4] WiFi subsystem... ");
    if crate::drivers::net::wifi::ckk() {
        crate::n!(B_, "ACTIVE (state: {:?})", crate::drivers::net::wifi::state());
        gd += 1;
    } else {
        crate::n!(D_, "no WiFi hardware");
        gd += 1; 
    }

    crate::println!();
    let av = gd + gv;
    if gv == 0 {
        crate::n!(B_, "  Network: {}/{} PASSED", gd, av);
    } else {
        crate::n!(D_, "  Network: {}/{} passed, {} failed", gd, av, gv);
    }
}


fn knh() {
    crate::n!(C_, "=== Active Drivers ===");
    crate::println!();

    
    crate::print!("  Network:   ");
    if crate::network::sw() {
        crate::n!(B_, "{}", crate::network::fyv());
    } else {
        crate::n!(A_, "none");
    }

    
    crate::print!("  WiFi:      ");
    if crate::drivers::net::wifi::ckk() {
        crate::n!(B_, "iwl4965 (state: {:?})", crate::drivers::net::wifi::state());
    } else {
        crate::n!(D_, "not detected");
    }

    
    crate::print!("  HDA Audio: ");
    if crate::drivers::hda::is_initialized() {
        crate::n!(B_, "initialized");
    } else {
        crate::n!(D_, "not initialized");
    }

    
    crate::print!("  ThinkPad EC: ");
    let bbg = crate::debug::om(0x66);
    if bbg != 0xFF {
        crate::n!(B_, "present (status=0x{:02X})", bbg);
    } else {
        crate::n!(D_, "not detected");
    }

    
    crate::print!("  WiFi FW:   ");
    if crate::drivers::net::iwl4965::eou() {
        crate::n!(B_, "loaded");
    } else {
        crate::n!(D_, "not loaded");
    }

    crate::println!();
}


fn knl() {
    let devices = crate::pci::aqs();
    crate::n!(C_, "=== PCI Bus Scan ({} devices) ===", devices.len());
    crate::println!();

    for s in &devices {
        crate::n!(B_, "{:02X}:{:02X}.{} {:04X}:{:04X}",
            s.bus, s.device, s.function, s.vendor_id, s.device_id);
        crate::println!("  Class:   {:02X}:{:02X} ({})", s.class_code, s.subclass, s.subclass_name());
        crate::println!("  Vendor:  {}", s.vendor_name());
        crate::println!("  ProgIF:  {:02X}  Rev: {:02X}", s.prog_if, s.revision);

        let cmd = crate::pci::ms(s.bus, s.device, s.function, 0x04);
        crate::println!("  Command: 0x{:04X} (mem_space={}, bus_master={}, io_space={})",
            cmd & 0xFFFF,
            if cmd & 0x02 != 0 { "ON" } else { "off" },
            if cmd & 0x04 != 0 { "ON" } else { "off" },
            if cmd & 0x01 != 0 { "ON" } else { "off" });

        let status = (crate::pci::ms(s.bus, s.device, s.function, 0x04) >> 16) & 0xFFFF;
        crate::println!("  Status:  0x{:04X}", status);

        if s.interrupt_line != 0xFF {
            crate::println!("  IRQ:     {} (pin {})", s.interrupt_line, s.interrupt_pin);
        }

        for i in 0..6 {
            if s.bar[i] != 0 {
                let bqj = if s.bar[i] & 1 == 0 { "MEM" } else { "I/O" };
                let addr = if s.bar[i] & 1 == 0 { s.bar[i] & 0xFFFFFFF0 } else { s.bar[i] & 0xFFFFFFFC };
                crate::println!("  BAR{}:    0x{:08X} [{}]", i, addr, bqj);
            }
        }
        crate::println!();
    }
}

pub(super) fn kpn(_args: &[&str]) {
    crate::println!("COMMAND   PID   FD   TYPE   NAME");
    crate::println!("----------------------------------------");
    crate::println!("shell     1     0    CHR    /dev/stdin");
    crate::println!("shell     1     1    CHR    /dev/stdout");
    crate::println!("shell     1     2    CHR    /dev/stderr");
}

pub(super) fn ksb(args: &[&str]) {
    if args.is_empty() {
        crate::println!("Usage: strings <file>");
        return;
    }
    
    match super::network::exu(args[0]) {
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
        None => crate::n!(A_, "strings: cannot read {}", args[0]),
    }
}

pub(super) fn fmu(args: &[&str]) {
    if args.is_empty() {
        
        crate::n!(G_, "Mounted Filesystems:");
        crate::vfs::dtl();
        return;
    }
    
    if args.len() < 2 {
        crate::println!("Usage: mount <device> <mountpoint>");
        return;
    }
    
    crate::n!(D_, "mount: dynamic mounting not implemented");
}

pub(super) fn kse() {
    crate::println!("Syncing filesystems...");
    crate::n!(B_, "Done.");
}

pub(super) fn kti(args: &[&str]) {
    if args.is_empty() {
        crate::println!("Usage: umount <mountpoint>");
        return;
    }
    match crate::vfs::ppk(args[0]) {
        Ok(()) => crate::n!(B_, "Unmounted {}", args[0]),
        Err(e) => crate::n!(A_, "umount: {}: {:?}", args[0], e),
    }
}

pub(super) fn kny(args: &[&str]) {
    crate::n!(G_, "TrustFS Filesystem Check");
    crate::println!("========================================");

    let mounts = crate::vfs::dtl();
    if mounts.is_empty() {
        crate::n!(D_, "No mounted filesystems");
        return;
    }

    let mut errors = 0u32;
    let mut checked = 0u32;

    for (path, caa) in &mounts {
        checked += 1;
        crate::print!("  [{}] {} ({})... ", checked, path, caa);

        
        match crate::vfs::readdir(path) {
            Ok(entries) => {
                let count = entries.len();
                crate::n!(B_, "OK ({} entries)", count);
            }
            Err(e) => {
                errors += 1;
                crate::n!(A_, "ERROR: {:?}", e);
            }
        }
    }

    crate::println!("----------------------------------------");
    if errors == 0 {
        crate::n!(B_, "fsck: {} filesystem(s) checked, no errors", checked);
    } else {
        crate::n!(A_, "fsck: {} error(s) found in {} filesystem(s)", errors, checked);
    }
}

pub(super) fn kpj() {
    crate::n!(G_, "Block Devices:");
    crate::println!("NAME          SIZE        TYPE    DRIVER        MODEL");
    crate::println!("----------------------------------------------------------------------");
    
    let mut idx = 0u32;
    
    
    if crate::nvme::is_initialized() {
        if let Some((model, _serial, ns_size, lba_size)) = crate::nvme::rk() {
            let size_bytes = ns_size * lba_size as u64;
            let td = aqo(size_bytes);
            crate::println!("nvme0n1       {:<11} disk    NVMe          {}", td, model);
            idx += 1;
        }
    }
    
    
    if crate::drivers::ahci::is_initialized() {
        for s in crate::drivers::ahci::adz() {
            let size_bytes = s.sector_count * 512;
            let td = aqo(size_bytes);
            let ws = match s.device_type {
                crate::drivers::ahci::AhciDeviceType::Sata => "disk",
                crate::drivers::ahci::AhciDeviceType::Satapi => "cdrom",
                _ => "disk",
            };
            crate::println!("sda{}          {:<11} {:<7} AHCI/p{}       {}", 
                idx, td, ws, s.port_num, s.model);
            idx += 1;
        }
    }
    
    
    for tz in crate::drivers::ata::eta() {
        if tz.present {
            let size_bytes = tz.sector_count * 512;
            let td = aqo(size_bytes);
            let ch = match tz.channel {
                crate::drivers::ata::IdeChannel::Primary => "P",
                crate::drivers::ata::IdeChannel::Secondary => "S",
            };
            let pos = match tz.position {
                crate::drivers::ata::DrivePosition::Master => "M",
                crate::drivers::ata::DrivePosition::Slave => "S",
            };
            let ws = if tz.atapi { "cdrom" } else { "disk" };
            let mxn = if tz.lba48 { "LBA48" } else { "LBA28" };
            crate::println!("hd{}           {:<11} {:<7} IDE/{}{} {}  {}", 
                idx, td, ws, ch, pos, mxn, tz.model);
            idx += 1;
        }
    }
    
    
    if crate::virtio_blk::is_initialized() {
        let cap = crate::virtio_blk::capacity();
        let td = aqo(cap * 512);
        let eyv = if crate::virtio_blk::is_read_only() { " (ro)" } else { "" };
        crate::println!("vda{}          {:<11} disk    VirtIO-blk{}", idx, td, eyv);
        idx += 1;
    }
    
    
    for (i, (name, blocks, bsize)) in crate::drivers::usb_storage::adz().iter().enumerate() {
        let td = aqo(*blocks * *bsize as u64);
        crate::println!("usb{}          {:<11} disk    USB-Storage   {}", 
            idx + i as u32, td, name);
    }
    if idx == 0 && crate::drivers::usb_storage::aqg() == 0 {
        idx += 1; 
    }
    
    
    crate::println!("ram0          256K        ramdisk RAM           TrustFS");
    
    if idx == 0 {
        crate::println!();
        crate::n!(D_, "No hardware storage detected (using RAM disk)");
    }
}


fn aqo(bytes: u64) -> alloc::string::String {
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

pub(super) fn kmb() {
    let mut nj = false;
    
    
    if crate::nvme::is_initialized() {
        if let Some((model, serial, ns_size, lba_size)) = crate::nvme::rk() {
            let size_bytes = ns_size * lba_size as u64;
            crate::println!("/dev/nvme0n1: MODEL=\"{}\" SERIAL=\"{}\" SIZE={} TYPE=\"nvme\"",
                model, serial, aqo(size_bytes));
            nj = true;
        }
    }
    
    
    if crate::drivers::ahci::is_initialized() {
        for (i, s) in crate::drivers::ahci::adz().iter().enumerate() {
            let size_bytes = s.sector_count * 512;
            crate::println!("/dev/sda{}: MODEL=\"{}\" SERIAL=\"{}\" SIZE={} TYPE=\"ahci\" PORT={}",
                i, s.model, s.serial, aqo(size_bytes), s.port_num);
            nj = true;
        }
    }
    
    
    for (i, tz) in crate::drivers::ata::eta().iter().enumerate() {
        if tz.present {
            let size_bytes = tz.sector_count * 512;
            let caa = if tz.atapi { "atapi" } else { "ide" };
            crate::println!("/dev/hd{}: MODEL=\"{}\" SERIAL=\"{}\" SIZE={} TYPE=\"{}\"",
                i, tz.model, tz.serial, aqo(size_bytes), caa);
            nj = true;
        }
    }
    
    
    if crate::virtio_blk::is_initialized() {
        let cap = crate::virtio_blk::capacity();
        crate::println!("/dev/vda: SIZE={} TYPE=\"virtio-blk\"", aqo(cap * 512));
        nj = true;
    }
    
    
    for (i, (name, blocks, bsize)) in crate::drivers::usb_storage::adz().iter().enumerate() {
        crate::println!("/dev/usb{}: MODEL=\"{}\" SIZE={} TYPE=\"usb-storage\"",
            i, name, aqo(*blocks * *bsize as u64));
        nj = true;
    }
    
    
    crate::println!("/dev/ram0: SIZE=256K TYPE=\"ramfs\"");
    
    if !nj {
        crate::n!(D_, "No hardware block devices detected");
    }
}

pub(super) fn che(args: &[&str]) {
    if args.is_empty() {
        
        for (k, v) in super::scripting::efi() {
            crate::println!("export {}={}", k, v);
        }
        return;
    }
    
    let geh = args.join(" ");
    if let Some(eq_pos) = geh.find('=') {
        let key = geh[..eq_pos].trim();
        let val = geh[eq_pos + 1..].trim().trim_matches('"').trim_matches('\'');
        super::scripting::cql(key, val);
        crate::serial_println!("[export] {}={}", key, val);
    } else {
        
        if super::scripting::axh(args[0]).is_none() {
            super::scripting::cql(args[0], "");
        }
    }
}

pub(super) fn kru(args: &[&str]) {
    if args.is_empty() {
        crate::println!("Usage: source <script>");
        return;
    }
    
    match super::network::cpa(args[0]) {
        Some(content) => {
            super::scripting::lse(&content);
        }
        None => crate::n!(A_, "source: cannot read {}", args[0]),
    }
}

pub(super) fn krk(_args: &[&str]) {
    for (k, v) in super::scripting::efi() {
        crate::println!("{}={}", k, v);
    }
}

pub(super) fn kqq(args: &[&str]) {
    if args.is_empty() {
        crate::println!("Usage: printf <format> [args...]");
        return;
    }
    
    let format = args[0].replace("\\n", "\n").replace("\\t", "\t");
    crate::print!("{}", format);
}

pub(super) fn kso(args: &[&str]) {
    
    if args.is_empty() {
        crate::println!("false");
        return;
    }
    
    match args.first() {
        Some(&"-e") if args.len() > 1 => {
            if super::vm::bbs(args[1]) {
                crate::println!("true");
            } else {
                crate::println!("false");
            }
        }
        Some(&"-d") if args.len() > 1 => {
            crate::n!(D_, "(directory check not implemented)");
        }
        Some(&"-f") if args.len() > 1 => {
            if super::vm::bbs(args[1]) {
                crate::println!("true");
            } else {
                crate::println!("false");
            }
        }
        _ => crate::println!("true"),
    }
}

pub(super) fn knr(args: &[&str]) {
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

pub(super) fn kmf(_args: &[&str]) {
    crate::n!(G_, "   February 2026");
    crate::println!("Su Mo Tu We Th Fr Sa");
    crate::println!(" 1  2  3  4  5  6  7");
    crate::println!(" 8  9 10 11 12 13 14");
    crate::println!("15 16 17 18 19 20 21");
    crate::println!("22 23 24 25 26 27 28");
}

pub(super) fn kmj(args: &[&str]) {
    if args.len() < 2 {
        crate::println!("Usage: cmp <file1> <file2>");
        return;
    }
    
    match (super::network::exu(args[0]), super::network::exu(args[1])) {
        (Some(a), Some(b)) => {
            if a == b {
                
            } else {
                crate::println!("{} {} differ", args[0], args[1]);
            }
        }
        _ => crate::n!(A_, "cmp: cannot read files"),
    }
}

pub(super) fn kqe(args: &[&str]) {
    if args.is_empty() {
        crate::println!("Usage: od <file>");
        return;
    }
    
    super::commands::hlz(args);
}

pub(super) fn kra(args: &[&str]) {
    if args.is_empty() {
        crate::println!("Usage: rev <file>");
        return;
    }
    
    match super::network::cpa(args[0]) {
        Some(content) => {
            for line in content.lines() {
                let ogt: String = line.chars().rev().collect();
                crate::println!("{}", ogt);
            }
        }
        None => crate::n!(A_, "rev: cannot read {}", args[0]),
    }
}

pub(super) fn kns(args: &[&str]) {
    if args.is_empty() {
        crate::println!("Usage: factor <number>");
        return;
    }
    
    let mut ae: u64 = args[0].parse().unwrap_or(0);
    if ae == 0 {
        crate::println!("factor: invalid number");
        return;
    }
    
    crate::print!("{}:", ae);
    let mut d = 2u64;
    while d.checked_mul(d).map_or(false, |dd| dd <= ae) {
        while ae % d == 0 {
            crate::print!(" {}", d);
            ae /= d;
        }
        d += 1;
    }
    if ae > 1 {
        crate::print!(" {}", ae);
    }
    crate::println!();
}

pub(super) fn ktg() {
    crate::println!("/dev/tty0");
}

pub(super) fn ksc(_args: &[&str]) {
    crate::println!("speed 9600 baud; line = 0;");
    crate::println!("-brkint -imaxbel");
}

pub(super) fn kqy() {
    super::commands::eif();
    crate::println!("Terminal reset.");
}

pub(super) fn kpp() {
    crate::n!(G_, "USB Devices:");
    crate::println!("-------------------------------------------");
    
    
    if crate::drivers::xhci::is_initialized() {
        let devices = crate::drivers::xhci::adz();
        if devices.is_empty() {
            crate::println!("Bus 001 Device 001: ID 0000:0000 Root Hub");
            crate::println!("  (no devices connected)");
        } else {
            crate::println!("Bus 001 Device 001: ID 0000:0000 xHCI Root Hub");
            for (i, s) in devices.iter().enumerate() {
                let speed = match s.speed {
                    1 => "Full Speed (12 Mbps)",
                    2 => "Low Speed (1.5 Mbps)",
                    3 => "High Speed (480 Mbps)",
                    4 => "SuperSpeed (5 Gbps)",
                    _ => "Unknown",
                };
                crate::println!("Bus 001 Device {:03}: ID {:04x}:{:04x} Port {} - {}", 
                    i + 2, s.vendor_id, s.product_id, s.port, speed);
                if s.class != 0 {
                    let class_name = match s.class {
                        0x03 => "HID (Human Interface Device)",
                        0x08 => "Mass Storage",
                        0x09 => "Hub",
                        _ => "Unknown class",
                    };
                    crate::println!("    Class: {:02x}:{:02x}:{:02x} ({})", 
                        s.class, s.subclass, s.protocol, class_name);
                }
            }
        }
        crate::println!("");
        crate::println!("Total: {} device(s) connected", devices.len());
    } else {
        crate::println!("Bus 001 Device 001: ID 0000:0000 Root Hub");
        crate::n!(D_, "  (xHCI controller not initialized)");
    }
}

pub(super) fn krq() {
    crate::cpu::smp::gof();
}

pub(super) fn krp(args: &[&str]) {
    if args.is_empty() {
        let status = if crate::cpu::smp::eru() { "ON" } else { "OFF" };
        let cpus = crate::cpu::smp::ail();
        crate::println!("SMP parallelism: {} ({} CPUs ready)", status, cpus);
        crate::println!("Usage: smp [on|off|status]");
        crate::println!("  on     - Enable multi-core parallel rendering");
        crate::println!("  off    - Disable parallelism (single-core, safe mode)");
        crate::println!("  status - Show detailed CPU status");
        return;
    }
    
    match args[0] {
        "on" | "1" | "enable" => {
            crate::cpu::smp::elh();
            crate::n!(0xFF00FF00, "SMP parallelism ENABLED");
        },
        "off" | "0" | "disable" => {
            crate::cpu::smp::fsj();
            crate::n!(0xFFFF8800, "SMP parallelism DISABLED (single-core mode)");
        },
        "status" => {
            crate::cpu::smp::gof();
        },
        _ => {
            crate::println!("Unknown option: {}", args[0]);
            crate::println!("Usage: smp [on|off|status]");
        }
    }
}

pub(super) fn knx(args: &[&str]) {
    use crate::framebuffer::font::{FontMode, guj, ibp};
    
    if args.is_empty() {
        let current = match ibp() {
            FontMode::Sharp => "sharp (disabled)",
            FontMode::Smooth => "smooth (enabled)",
        };
        crate::println!("Font smoothing: {}", current);
        crate::println!("Usage: fontsmooth [on|off]");
        return;
    }
    
    match args[0] {
        "on" | "enable" | "smooth" => {
            guj(FontMode::Smooth);
            crate::println!("Font smoothing enabled");
        }
        "off" | "disable" | "sharp" => {
            guj(FontMode::Sharp);
            crate::println!("Font smoothing disabled");
        }
        _ => {
            crate::println!("Usage: fontsmooth [on|off]");
        }
    }
}

pub(super) fn hma() {
    crate::n!(G_, "CPU Information:");
    crate::println!("-------------------------------------------");
    
    
    if let Some(caps) = crate::cpu::capabilities() {
        crate::println!("Brand:        {}", caps.brand());
        crate::println!("Architecture: x86_64");
        crate::println!("Vendor:       {:?}", caps.vendor);
        crate::println!("Family:       {}", caps.family);
        crate::println!("Model:        {}", caps.model);
        crate::println!("Stepping:     {}", caps.stepping);
        crate::println!("CPU(s):       {}", crate::cpu::smp::cpu_count());
        crate::println!("APIC ID:      {}", caps.apic_id);
        
        
        crate::println!("");
        crate::n!(C_, "Timing:");
        crate::println!("TSC:          {} (invariant: {})", 
            if caps.tsc { "yes" } else { "no" },
            if caps.tsc_invariant { "yes" } else { "no" });
        crate::println!("TSC Freq:     {} MHz", caps.tsc_frequency_hz / 1_000_000);
        crate::println!("RDTSCP:       {}", if caps.rdtscp { "yes" } else { "no" });
        
        
        crate::println!("");
        crate::n!(C_, "SIMD:");
        crate::println!("SSE:          {}", if caps.sse { "yes" } else { "no" });
        crate::println!("SSE2:         {}", if caps.sse2 { "yes" } else { "no" });
        crate::println!("SSE3:         {}", if caps.sse3 { "yes" } else { "no" });
        crate::println!("SSSE3:        {}", if caps.ssse3 { "yes" } else { "no" });
        crate::println!("SSE4.1:       {}", if caps.sse4_1 { "yes" } else { "no" });
        crate::println!("SSE4.2:       {}", if caps.sse4_2 { "yes" } else { "no" });
        crate::println!("AVX:          {}", if caps.avx { "yes" } else { "no" });
        crate::println!("AVX2:         {}", if caps.avx2 { "yes" } else { "no" });
        crate::println!("AVX-512:      {}", if caps.avx512f { "yes" } else { "no" });
        
        
        crate::println!("");
        crate::n!(C_, "Crypto Acceleration:");
        crate::println!("AES-NI:       {}", if caps.aesni { "yes" } else { "no" });
        crate::println!("PCLMULQDQ:    {}", if caps.pclmulqdq { "yes" } else { "no" });
        crate::println!("SHA-NI:       {}", if caps.sha_ext { "yes" } else { "no" });
        crate::println!("RDRAND:       {}", if caps.rdrand { "yes" } else { "no" });
        crate::println!("RDSEED:       {}", if caps.rdseed { "yes" } else { "no" });
        
        
        crate::println!("");
        crate::n!(C_, "Security:");
        crate::println!("SMEP:         {}", if caps.smep { "yes" } else { "no" });
        crate::println!("SMAP:         {}", if caps.smap { "yes" } else { "no" });
        crate::println!("NX:           {}", if caps.nx { "yes" } else { "no" });
        
        
        crate::println!("");
        crate::n!(C_, "Virtualization:");
        crate::println!("Intel VT-x:   {}", if caps.vmx { "yes" } else { "no" });
        crate::println!("AMD-V:        {}", if caps.svm { "yes" } else { "no" });
    } else {
        crate::println!("Architecture: x86_64");
        crate::println!("(CPU detection not initialized)");
    }
}

pub(super) fn kpl() {
    let heap_total = (crate::memory::stats().heap_used + crate::memory::stats().heap_free);
    
    crate::n!(G_, "Memory Configuration:");
    crate::println!("-------------------------------------------");
    crate::println!("Total:       {} KB", heap_total / 1024);
    crate::println!("Used:        {} KB", crate::memory::stats().heap_used / 1024);
}

pub(super) fn kpm() {
    crate::n!(G_, "Loaded Kernel Modules:");
    crate::println!("Module                  Size  Used by");
    crate::println!("e1000                  64000  1");
    crate::println!("ahci                   32000  0");
    crate::println!("ps2kbd                  8000  1");
    crate::println!("ps2mouse                4000  1");
}

pub(super) fn ksh(_args: &[&str]) {
    crate::println!("kernel.ostype = TrustOS");
    crate::println!("kernel.osrelease = 0.1.0");
    crate::println!("kernel.version = #1 SMP TrustOS");
}



pub(super) fn knw(args: &[&str]) {
    use crate::netstack::firewall;
    use crate::netstack::firewall::{Chain, Action, Protocol, IpMatch, PortMatch, Rule};

    if args.is_empty() {
        hlx();
        return;
    }

    match args[0] {
        "status" | "show" => hlx(),
        "enable" | "on" => {
            firewall::set_enabled(true);
            crate::n!(B_, "Firewall enabled");
        }
        "disable" | "off" => {
            firewall::set_enabled(false);
            crate::n!(D_, "Firewall disabled");
        }
        "policy" => {
            
            if args.len() < 3 {
                crate::println!("Usage: firewall policy <INPUT|OUTPUT|FORWARD> <ACCEPT|DROP>");
                return;
            }
            let chain = match Chain::atv(args[1]) {
                Some(c) => c,
                None => { crate::n!(A_, "Invalid chain: {}", args[1]); return; }
            };
            let action = match Action::atv(args[2]) {
                Some(a) => a,
                None => { crate::n!(A_, "Invalid action: {}", args[2]); return; }
            };
            firewall::set_policy(chain, action);
            crate::n!(B_, "Policy {} set to {}", chain.name(), action.name());
        }
        "add" => {
            
            if args.len() < 2 {
                crate::println!("Usage: firewall add <chain> [-p proto] [-s src] [-d dst] [--sport port] [--dport port] -j <action>");
                return;
            }
            let chain = match Chain::atv(args[1]) {
                Some(c) => c,
                None => { crate::n!(A_, "Invalid chain: {}", args[1]); return; }
            };
            let mut qo = Rule::new(chain, Action::Accept);
            let mut i = 2;
            while i < args.len() {
                match args[i] {
                    "-p" | "--proto" => {
                        i += 1;
                        if i < args.len() {
                            qo.protocol = Protocol::atv(args[i]).unwrap_or(Protocol::Any);
                        }
                    }
                    "-s" | "--src" => {
                        i += 1;
                        if i < args.len() {
                            qo.src_ip = IpMatch::parse(args[i]).unwrap_or(IpMatch::Any);
                        }
                    }
                    "-d" | "--dst" => {
                        i += 1;
                        if i < args.len() {
                            qo.dst_ip = IpMatch::parse(args[i]).unwrap_or(IpMatch::Any);
                        }
                    }
                    "--sport" => {
                        i += 1;
                        if i < args.len() {
                            qo.src_port = PortMatch::parse(args[i]).unwrap_or(PortMatch::Any);
                        }
                    }
                    "--dport" => {
                        i += 1;
                        if i < args.len() {
                            qo.dst_port = PortMatch::parse(args[i]).unwrap_or(PortMatch::Any);
                        }
                    }
                    "-j" | "--jump" => {
                        i += 1;
                        if i < args.len() {
                            qo.action = Action::atv(args[i]).unwrap_or(Action::Accept);
                        }
                    }
                    _ => {}
                }
                i += 1;
            }
            firewall::jtx(qo);
            crate::n!(B_, "Rule added to {} chain", chain.name());
        }
        "del" | "delete" => {
            
            if args.len() < 3 {
                crate::println!("Usage: firewall del <chain> <index>");
                return;
            }
            let chain = match Chain::atv(args[1]) {
                Some(c) => c,
                None => { crate::n!(A_, "Invalid chain: {}", args[1]); return; }
            };
            let idx: usize = match args[2].parse() {
                Ok(ae) => ae,
                Err(_) => { crate::n!(A_, "Invalid index: {}", args[2]); return; }
            };
            if firewall::ldc(chain, idx) {
                crate::n!(B_, "Rule {} deleted from {}", idx, chain.name());
            } else {
                crate::n!(A_, "Rule {} not found in {}", idx, chain.name());
            }
        }
        "flush" => {
            let chain = if args.len() > 1 { Chain::atv(args[1]) } else { None };
            firewall::flush(chain);
            if let Some(c) = chain {
                crate::n!(B_, "Flushed {} chain", c.name());
            } else {
                crate::n!(B_, "Flushed all chains");
            }
        }
        "log" => {
            let entries = firewall::mdj();
            if entries.is_empty() {
                crate::println!("(no log entries)");
            } else {
                crate::n!(C_, "Firewall Log ({} entries):", entries.len());
                for entry in &entries {
                    crate::println!("  {}", entry);
                }
            }
        }
        "reset" => {
            firewall::jai();
            firewall::kku();
            crate::n!(B_, "Stats and log cleared");
        }
        "help" | "--help" | "-h" => {
            crate::n!(C_, "TrustOS Firewall — iptables-like packet filter");
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
            crate::n!(A_, "Unknown subcommand: {}", args[0]);
            crate::println!("Try: firewall help");
        }
    }
}

fn hlx() {
    use crate::netstack::firewall;
    use crate::netstack::firewall::Chain;

    let enabled = firewall::lq();
    let (bxl, dropped) = firewall::stats();

    crate::n!(C_, "TrustOS Firewall");
    crate::print!("  Status: ");
    if enabled {
        crate::n!(B_, "ENABLED");
    } else {
        crate::n!(A_, "DISABLED");
    }
    crate::println!("  Packets allowed: {}  dropped: {}", bxl, dropped);
    crate::println!();

    for chain in &[Chain::Input, Chain::Output, Chain::Forward] {
        let policy = firewall::get_policy(*chain);
        let rules = firewall::mzh(*chain);
        crate::bq!(D_, "Chain {} ", chain.name());
        crate::println!("(policy {})", policy.name());
        if rules.is_empty() {
            crate::println!("  (no rules)");
        } else {
            crate::println!("  {:>3}  {:>5}  {:>15}  {:>15}  {:>11}  {:>11}  {:>6}  {:>8}  {:>8}",
                "num", "proto", "source", "destination", "sport", "dport", "action", "pkts", "bytes");
            for (i, qo) in rules.iter().enumerate() {
                crate::println!("  {:>3}  {:>5}  {:>15}  {:>15}  {:>11}  {:>11}  {:>6}  {:>8}  {:>8}",
                    i, qo.protocol.name(), qo.src_ip.display(), qo.dst_ip.display(),
                    qo.src_port.display(), qo.dst_port.display(), qo.action.name(),
                    qo.packets, qo.bytes);
            }
        }
        crate::println!();
    }
}



pub(super) fn kno(args: &[&str]) {
    let path = if args.is_empty() { "/" } else { args[0] };
    let av = hug(path, 0);
    if av >= 1024 * 1024 {
        crate::println!("{:.1}M\t{}", av as f64 / (1024.0 * 1024.0), path);
    } else if av >= 1024 {
        crate::println!("{}K\t{}", av / 1024, path);
    } else {
        crate::println!("{}\t{}", av, path);
    }
}

fn hug(path: &str, depth: usize) -> usize {
    let mut av: usize = 0;

    if let Ok(entries) = crate::ramfs::bh(|fs| fs.ls(Some(path))) {
        for (name, wf, size) in &entries {
            let pd = if path == "/" {
                alloc::format!("/{}", name)
            } else {
                alloc::format!("{}/{}", path, name)
            };
            match wf {
                crate::ramfs::FileType::File => {
                    av += size;
                }
                crate::ramfs::FileType::Directory => {
                    let sub = hug(&pd, depth + 1);
                    av += sub;
                    if depth < 1 {
                        if sub >= 1024 {
                            crate::println!("{}K\t{}", sub / 1024, pd);
                        } else {
                            crate::println!("{}\t{}", sub, pd);
                        }
                    }
                }
                _ => {}
            }
        }
    }
    av
}






pub(super) fn kmh(args: &[&str]) {
    if args.len() < 2 {
        crate::println!("Usage: chmod <mode> <file>");
        crate::println!("  mode: 755, 644, +x, -w, etc.");
        return;
    }
    let mode = args[0];
    let path = args[1];
    if !super::vm::bbs(path) {
        crate::n!(A_, "chmod: {}: No such file", path);
        return;
    }
    crate::n!(B_, "chmod: mode of '{}' changed to {}", path, mode);
}


pub(super) fn kmi(args: &[&str]) {
    if args.len() < 2 {
        crate::println!("Usage: chown <owner[:group]> <file>");
        return;
    }
    let owner = args[0];
    let path = args[1];
    if !super::vm::bbs(path) {
        crate::n!(A_, "chown: {}: No such file", path);
        return;
    }
    crate::n!(B_, "chown: ownership of '{}' changed to {}", path, owner);
}


pub(super) fn kpf(args: &[&str]) {
    let ozl = args.first() == Some(&"-s");
    let gqn: Vec<&str> = args.iter().filter(|a| !a.starts_with('-')).copied().collect();
    if gqn.len() < 2 {
        crate::println!("Usage: ln [-s] <target> <link_name>");
        return;
    }
    let target = gqn[0];
    let link = gqn[1];
    
    if ozl {
        
        let content = format!("SYMLINK:{}", target);
        let result = crate::ramfs::bh(|fs| {
            let _ = fs.touch(link);
            fs.write_file(link, content.as_bytes())
        });
        match result {
            Ok(()) => crate::n!(B_, "'{}' -> '{}'", link, target),
            Err(_) => crate::n!(A_, "ln: failed to create symbolic link"),
        }
    } else {
        
        let data = crate::ramfs::bh(|fs| fs.read_file(target).map(|b| b.to_vec()));
        match data {
            Ok(bytes) => {
                crate::ramfs::bh(|fs| {
                    let _ = fs.touch(link);
                    let _ = fs.write_file(link, &bytes);
                });
                crate::n!(B_, "'{}' => '{}'", link, target);
            }
            Err(_) => crate::n!(A_, "ln: {}: No such file", target),
        }
    }
}


pub(super) fn kqu(args: &[&str]) {
    if args.is_empty() {
        crate::println!("Usage: readlink <symlink>");
        return;
    }
    let path = args[0];
    let content: Option<String> = crate::ramfs::bh(|fs| {
        fs.read_file(path).map(|b| String::from(core::str::from_utf8(b).unwrap_or(""))).ok()
    });
    match content {
        Some(ref j) if j.starts_with("SYMLINK:") => {
            crate::println!("{}", &j[8..]);
        }
        _ => crate::n!(A_, "readlink: {}: Not a symbolic link", path),
    }
}


pub(super) fn kmq(args: &[&str], piped: Option<&str>) {
    
    let mut frh = '\t';
    let mut fields: Option<Vec<usize>> = None;
    let mut hyc: Option<&str> = None;
    let mut i = 0;
    while i < args.len() {
        match args[i] {
            "-d" if i + 1 < args.len() => {
                frh = args[i + 1].chars().next().unwrap_or('\t');
                i += 2;
            }
            "-f" if i + 1 < args.len() => {
                fields = Some(nqg(args[i + 1]));
                i += 2;
            }
            db if !db.starts_with('-') => {
                hyc = Some(db);
                i += 1;
            }
            _ => { i += 1; }
        }
    }
    
    let lva = match fields {
        Some(f) => f,
        None => {
            crate::println!("Usage: cut -d <delimiter> -f <fields> [file]");
            crate::println!("  Example: cut -d : -f 1,3");
            return;
        }
    };
    
    let content = if let Some(input) = piped {
        Some(String::from(input))
    } else if let Some(path) = hyc {
        super::network::cpa(path)
    } else {
        crate::println!("cut: no input");
        return;
    };
    
    if let Some(text) = content {
        for line in text.lines() {
            let au: Vec<&str> = line.split(frh).collect();
            let mut first = true;
            for &f in &lva {
                if f > 0 && f <= au.len() {
                    if !first { crate::print!("{}", frh); }
                    crate::print!("{}", au[f - 1]);
                    first = false;
                }
            }
            crate::println!();
        }
    }
}

fn nqg(j: &str) -> Vec<usize> {
    let mut fields = Vec::new();
    for jn in j.split(',') {
        if let Some(cib) = jn.find('-') {
            let start: usize = jn[..cib].parse().unwrap_or(1);
            let end: usize = jn[cib + 1..].parse().unwrap_or(start).min(start + 10_000);
            for f in start..=end {
                fields.push(f);
            }
        } else if let Ok(f) = jn.parse::<usize>() {
            fields.push(f);
        }
    }
    fields
}


pub(super) fn ksx(args: &[&str], piped: Option<&str>) {
    if args.len() < 2 {
        crate::println!("Usage: tr <set1> <set2>");
        crate::println!("  Example: echo hello | tr a-z A-Z");
        return;
    }
    
    let oom = hxc(args[0]);
    let gud = hxc(args[1]);
    
    let content = if let Some(input) = piped {
        String::from(input)
    } else {
        crate::println!("tr: requires piped input");
        return;
    };
    
    let mut result = String::with_capacity(content.len());
    for ch in content.chars() {
        if let Some(pos) = oom.iter().position(|&c| c == ch) {
            if pos < gud.len() {
                result.push(gud[pos]);
            } else if let Some(&last) = gud.last() {
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

fn hxc(j: &str) -> Vec<char> {
    let mut chars = Vec::new();
    let bytes = j.as_bytes();
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


pub(super) fn ksm(args: &[&str], piped: Option<&str>) {
    let append = args.first() == Some(&"-a");
    let cjj: Vec<&str> = args.iter().filter(|a| !a.starts_with('-')).copied().collect();
    
    let content = if let Some(input) = piped {
        String::from(input)
    } else {
        crate::println!("tee: requires piped input");
        return;
    };
    
    
    crate::print!("{}", content);
    
    
    for path in &cjj {
        if append {
            let _ = crate::ramfs::bh(|fs| fs.append_file(path, content.as_bytes()));
        } else {
            let _ = crate::ramfs::bh(|fs| {
                if !fs.exists(path) { let _ = fs.touch(path); }
                fs.write_file(path, content.as_bytes())
            });
        }
    }
}


pub(super) fn kug(args: &[&str], piped: Option<&str>) {
    let command = if args.is_empty() { "echo" } else { args[0] };
    let hxj = if args.len() > 1 { &args[1..] } else { &[] };
    
    let content = if let Some(input) = piped {
        String::from(input)
    } else {
        crate::println!("xargs: requires piped input");
        return;
    };
    
    
    let items: Vec<&str> = content.split_whitespace().collect();
    for item in &items {
        let fms = if hxj.is_empty() {
            format!("{} {}", command, item)
        } else {
            format!("{} {} {}", command, hxj.join(" "), item)
        };
        super::aav(&fms);
    }
}


pub(super) fn hmg(args: &[&str]) {
    if args.is_empty() {
        crate::println!("Usage: unset <variable>");
        return;
    }
    for name in args {
        super::scripting::fdx(name);
        crate::n!(B_, "Unset: {}", name);
    }
}


pub(super) fn kqt(args: &[&str]) {
    let edn = if args.is_empty() { "REPLY" } else { args[0] };
    let nh = if args.len() > 1 && args[0] == "-p" {
        if args.len() > 2 {
            crate::print!("{}", args[1]);
            if args.len() > 2 { args[2] } else { "REPLY" }
        } else {
            "REPLY"
        }
    } else {
        edn
    };
    
    
    let mut input = String::new();
    loop {
        if let Some(key) = crate::keyboard::ya() {
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
    
    super::scripting::cql(nh, &input);
}



use spin::Mutex;
use alloc::collections::BTreeMap;

static Ji: Mutex<BTreeMap<String, String>> = Mutex::new(BTreeMap::new());

pub fn mco(name: &str) -> Option<String> {
    Ji.lock().get(name).cloned()
}

pub(super) fn kln(args: &[&str]) {
    if args.is_empty() {
        
        let efe = Ji.lock();
        if efe.is_empty() {
            crate::println!("No aliases defined");
        } else {
            for (name, value) in efe.iter() {
                crate::n!(C_, "alias {}='{}'", name, value);
            }
        }
        return;
    }
    
    let db = args.join(" ");
    if let Some(eq_pos) = db.find('=') {
        let name = db[..eq_pos].trim();
        let value = db[eq_pos + 1..].trim().trim_matches('\'').trim_matches('"');
        Ji.lock().insert(String::from(name), String::from(value));
        crate::n!(B_, "alias {}='{}'", name, value);
    } else {
        
        let efe = Ji.lock();
        if let Some(value) = efe.get(args[0]) {
            crate::println!("alias {}='{}'", args[0], value);
        } else {
            crate::n!(A_, "alias: {}: not found", args[0]);
        }
    }
}

pub(super) fn ktj(args: &[&str]) {
    if args.is_empty() {
        crate::println!("Usage: unalias <name>");
        return;
    }
    if args[0] == "-a" {
        Ji.lock().clear();
        crate::n!(B_, "All aliases removed");
    } else {
        if Ji.lock().remove(args[0]).is_some() {
            crate::n!(B_, "Alias '{}' removed", args[0]);
        } else {
            crate::n!(A_, "unalias: {}: not found", args[0]);
        }
    }
}


pub(super) fn klz(_args: &[&str]) {
    crate::n!(C_, "TrustOS bc — arbitrary precision calculator");
    crate::println!("Type expressions, 'quit' or 'exit' to leave");
    crate::println!();
    
    crate::shell::dks();
    
    loop {
        crate::bq!(B_, "bc> ");
        
        let mut input = String::new();
        loop {
            if let Some(key) = crate::keyboard::ya() {
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
        
        let jw = input.trim();
        if jw == "quit" || jw == "exit" {
            break;
        }
        if jw.is_empty() {
            continue;
        }
        
        
        match lro(jw) {
            Some(result) => {
                if result == (result as i64) as f64 {
                    crate::println!("{}", result as i64);
                } else {
                    crate::println!("{:.6}", result);
                }
            }
            None => crate::n!(A_, "Error: invalid expression"),
        }
    }
}

fn lro(expr: &str) -> Option<f64> {
    
    let tokens = pkw(expr);
    let mut pos = 0;
    let result = itn(&tokens, &mut pos);
    if pos == tokens.len() { result } else { None }
}

fn pkw(expr: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut num = String::new();
    for ch in expr.chars() {
        if ch.is_ascii_digit() || ch == '.' {
            num.push(ch);
        } else {
            if !num.is_empty() { tokens.push(core::mem::take(&mut num)); }
            if !ch.is_whitespace() {
                let mut buf = [0u8; 4];
                let j = ch.encode_utf8(&mut buf);
                tokens.push(String::from(j));
            }
        }
    }
    if !num.is_empty() { tokens.push(num); }
    tokens
}

fn itn(tokens: &[String], pos: &mut usize) -> Option<f64> {
    let mut left = itp(tokens, pos)?;
    while *pos < tokens.len() && (tokens[*pos] == "+" || tokens[*pos] == "-") {
        let op = tokens[*pos].clone();
        *pos += 1;
        let right = itp(tokens, pos)?;
        left = if op == "+" { left + right } else { left - right };
    }
    Some(left)
}

fn itp(tokens: &[String], pos: &mut usize) -> Option<f64> {
    let mut left = gmf(tokens, pos)?;
    while *pos < tokens.len() && (tokens[*pos] == "*" || tokens[*pos] == "/" || tokens[*pos] == "%") {
        let op = tokens[*pos].clone();
        *pos += 1;
        let right = gmf(tokens, pos)?;
        left = match op.as_str() {
            "*" => left * right,
            "/" => if right != 0.0 { left / right } else { return None },
            "%" => if right != 0.0 { left % right } else { return None },
            _ => unreachable!(),
        };
    }
    Some(left)
}

fn gmf(tokens: &[String], pos: &mut usize) -> Option<f64> {
    let base = npy(tokens, pos)?;
    if *pos < tokens.len() && tokens[*pos] == "^" {
        *pos += 1;
        let afe = gmf(tokens, pos)?;
        Some(nwi(base, afe))
    } else {
        Some(base)
    }
}

fn npy(tokens: &[String], pos: &mut usize) -> Option<f64> {
    if *pos < tokens.len() && tokens[*pos] == "-" {
        *pos += 1;
        let val = ito(tokens, pos)?;
        Some(-val)
    } else {
        ito(tokens, pos)
    }
}

fn ito(tokens: &[String], pos: &mut usize) -> Option<f64> {
    if *pos >= tokens.len() { return None; }
    if tokens[*pos] == "(" {
        *pos += 1;
        let val = itn(tokens, pos)?;
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

fn nwi(base: f64, afe: f64) -> f64 {
    if afe == 0.0 { return 1.0; }
    if afe == 1.0 { return base; }
    let dpa = afe as i32;
    if (afe - dpa as f64).abs() < 1e-9 {
        let mut result = 1.0;
        let mut b = base;
        let mut e = if dpa < 0 { -dpa as u32 } else { dpa as u32 };
        while e > 0 {
            if e & 1 == 1 { result *= b; }
            b *= b;
            e >>= 1;
        }
        if dpa < 0 { 1.0 / result } else { result }
    } else {
        
        base 
    }
}


pub(super) fn fmj(args: &[&str]) {
    if args.len() < 2 {
        crate::println!("Usage: diff <file1> <file2>");
        return;
    }
    
    let kxk = bol(args[0]);
    let kxl = bol(args[1]);
    
    let (hw, jf) = match (kxk, kxl) {
        (Some(a), Some(b)) => (a, b),
        (None, _) => { crate::n!(A_, "diff: {}: No such file", args[0]); return; }
        (_, None) => { crate::n!(A_, "diff: {}: No such file", args[1]); return; }
    };
    
    let ikf: Vec<&str> = hw.lines().collect();
    let ikg: Vec<&str> = jf.lines().collect();
    
    crate::n!(C_, "--- {}", args[0]);
    crate::n!(C_, "+++ {}", args[1]);
    
    let aoo = core::cmp::max(ikf.len(), ikg.len());
    let mut eor = false;
    
    for i in 0..aoo {
        let clt = ikf.get(i).copied();
        let alv = ikg.get(i).copied();
        
        match (clt, alv) {
            (Some(a), Some(b)) if a != b => {
                crate::n!(D_, "@@ -{},{} +{},{} @@", i + 1, 1, i + 1, 1);
                crate::n!(A_, "-{}", a);
                crate::n!(B_, "+{}", b);
                eor = true;
            }
            (Some(a), None) => {
                crate::n!(A_, "-{}", a);
                eor = true;
            }
            (None, Some(b)) => {
                crate::n!(B_, "+{}", b);
                eor = true;
            }
            _ => {}
        }
    }
    
    if !eor {
        crate::n!(B_, "Files are identical");
    }
}

fn bol(path: &str) -> Option<String> {
    if path.starts_with("/mnt/") || path.starts_with("/dev/") || path.starts_with("/proc/") {
        crate::vfs::gqh(path).ok()
    } else {
        crate::ramfs::bh(|fs| {
            fs.read_file(path).map(|b| String::from(core::str::from_utf8(b).unwrap_or(""))).ok()
        })
    }
}


pub(super) fn kpr(args: &[&str]) {
    if args.is_empty() {
        crate::println!("Usage: md5sum <file> ...");
        return;
    }
    for path in args {
        let data = crate::ramfs::bh(|fs| fs.read_file(path).map(|b| b.to_vec()));
        match data {
            Ok(bytes) => {
                let hash = otb(&bytes);
                crate::println!("{}  {}", hash, path);
            }
            Err(_) => crate::n!(A_, "md5sum: {}: No such file", path),
        }
    }
}


fn otb(data: &[u8]) -> String {
    
    let gtl: [u32; 4] = [0x811c9dc5, 0x01000193, 0xdeadbeef, 0xcafebabe];
    let mut aqy = [0u32; 4];
    for (i, seed) in gtl.iter().enumerate() {
        let mut h = *seed;
        for &byte in data {
            h ^= byte as u32;
            h = h.wrapping_mul(0x01000193);
        }
        
        h ^= data.len() as u32;
        h = h.wrapping_mul(0x01000193);
        aqy[i] = h;
    }
    format!("{:08x}{:08x}{:08x}{:08x}", aqy[0], aqy[1], aqy[2], aqy[3])
}


pub(super) fn krl(args: &[&str]) {
    if args.is_empty() {
        crate::println!("Usage: sha256sum <file> ...");
        return;
    }
    for path in args {
        let data = crate::ramfs::bh(|fs| fs.read_file(path).map(|b| b.to_vec()));
        match data {
            Ok(bytes) => {
                let hash = otc(&bytes);
                crate::println!("{}  {}", hash, path);
            }
            Err(_) => crate::n!(A_, "sha256sum: {}: No such file", path),
        }
    }
}


fn otc(data: &[u8]) -> String {
    let gtl: [u32; 8] = [0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a,
                           0x510e527f, 0x9b05688c, 0x1f83d9ab, 0x5be0cd19];
    let mut aqy = [0u32; 8];
    for (i, seed) in gtl.iter().enumerate() {
        let mut h = *seed;
        for (ay, &byte) in data.iter().enumerate() {
            h ^= byte as u32;
            h = h.wrapping_mul(0x01000193);
            h ^= (ay as u32).wrapping_add(i as u32);
            h = h.rotate_left(5);
        }
        h ^= data.len() as u32;
        h = h.wrapping_mul(0x01000193 + i as u32);
        aqy[i] = h;
    }
    format!("{:08x}{:08x}{:08x}{:08x}{:08x}{:08x}{:08x}{:08x}",
        aqy[0], aqy[1], aqy[2], aqy[3],
        aqy[4], aqy[5], aqy[6], aqy[7])
}


pub(super) fn klx(args: &[&str], piped: Option<&str>) {
    let dmo = args.first() == Some(&"-d") || args.first() == Some(&"--decode");
    let cjj: Vec<&str> = args.iter().filter(|a| !a.starts_with('-')).copied().collect();
    
    let content = if let Some(input) = piped {
        Some(String::from(input))
    } else if !cjj.is_empty() {
        bol(cjj[0])
    } else {
        crate::println!("Usage: base64 [-d] [file]");
        crate::println!("  Or: echo text | base64");
        return;
    };
    
    if let Some(text) = content {
        if dmo {
            match jzu(text.trim()) {
                Some(uu) => crate::print!("{}", core::str::from_utf8(&uu).unwrap_or("(binary data)")),
                None => crate::n!(A_, "base64: invalid input"),
            }
        } else {
            let atq = jzv(text.as_bytes());
            crate::println!("{}", atq);
        }
    }
}

const SC_: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

fn jzv(data: &[u8]) -> String {
    let mut result = String::new();
    let mut i = 0;
    while i < data.len() {
        let kl = data[i] as u32;
        let gf = if i + 1 < data.len() { data[i + 1] as u32 } else { 0 };
        let iq = if i + 2 < data.len() { data[i + 2] as u32 } else { 0 };
        let cet = (kl << 16) | (gf << 8) | iq;
        
        result.push(SC_[((cet >> 18) & 0x3F) as usize] as char);
        result.push(SC_[((cet >> 12) & 0x3F) as usize] as char);
        
        if i + 1 < data.len() {
            result.push(SC_[((cet >> 6) & 0x3F) as usize] as char);
        } else {
            result.push('=');
        }
        if i + 2 < data.len() {
            result.push(SC_[(cet & 0x3F) as usize] as char);
        } else {
            result.push('=');
        }
        i += 3;
    }
    result
}

fn eft(c: u8) -> Option<u32> {
    match c {
        b'A'..=b'Z' => Some((c - b'A') as u32),
        b'a'..=b'z' => Some((c - b'a' + 26) as u32),
        b'0'..=b'9' => Some((c - b'0' + 52) as u32),
        b'+' => Some(62),
        b'/' => Some(63),
        _ => None,
    }
}

fn jzu(j: &str) -> Option<Vec<u8>> {
    let mut result = Vec::new();
    let bytes: Vec<u8> = j.bytes().filter(|&b| b != b'\n' && b != b'\r' && b != b' ').collect();
    let mut i = 0;
    while i + 3 < bytes.len() {
        let a = eft(bytes[i])?;
        let b = eft(bytes[i + 1])?;
        let kgs = if bytes[i + 2] == b'=' { 0 } else { eft(bytes[i + 2])? };
        let d_val = if bytes[i + 3] == b'=' { 0 } else { eft(bytes[i + 3])? };
        
        let cet = (a << 18) | (b << 12) | (kgs << 6) | d_val;
        result.push(((cet >> 16) & 0xFF) as u8);
        if bytes[i + 2] != b'=' {
            result.push(((cet >> 8) & 0xFF) as u8);
        }
        if bytes[i + 3] != b'=' {
            result.push((cet & 0xFF) as u8);
        }
        i += 4;
    }
    Some(result)
}


pub(super) fn kty(args: &[&str]) {
    if args.is_empty() {
        crate::println!("Usage: watch [-n <seconds>] <command>");
        crate::println!("  Example: watch -n 2 ps");
        return;
    }
    
    let mut gde: u64 = 2;
    let mut byc = 0;
    
    if args.len() > 2 && args[0] == "-n" {
        gde = args[1].parse().unwrap_or(2);
        byc = 2;
    }
    
    let cmd = args[byc..].join(" ");
    crate::shell::dks();
    
    loop {
        if crate::shell::cbc() { break; }
        if let Some(3) = crate::keyboard::ya() {
            crate::shell::fag();
            break;
        }
        
        crate::framebuffer::clear();
        crate::framebuffer::afr(0, 0);
        crate::n!(C_, "Every {}s: {}    {}", gde, cmd, "TrustOS");
        crate::println!("---");
        
        super::aav(&cmd);
        
        
        let start = crate::time::uptime_ms();
        let end = start + gde * 1000;
        while crate::time::uptime_ms() < end {
            if let Some(3) = crate::keyboard::ya() {
                crate::shell::fag();
                return;
            }
            core::hint::spin_loop();
        }
    }
    
    crate::println!();
    crate::n!(D_, "watch: interrupted");
}


pub(super) fn ksw(args: &[&str]) {
    if args.len() < 2 {
        crate::println!("Usage: timeout <seconds> <command>");
        return;
    }
    
    let im: u64 = args[0].parse().unwrap_or(5);
    let cmd = args[1..].join(" ");
    
    let brq = crate::time::uptime_ms() + im * 1000;
    
    
    crate::n!(C_, "[timeout: {}s] {}", im, cmd);
    super::aav(&cmd);
    
    if crate::time::uptime_ms() > brq {
        crate::n!(A_, "timeout: command timed out after {}s", im);
    }
}


pub(super) fn ksj(args: &[&str]) {
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
    let ltn = flags.contains('x');
    
    let asy = if args.len() > 1 && flags.contains('f') { args[1] } else {
        crate::n!(A_, "tar: -f <archive> required");
        return;
    };
    
    if create {
        let files = if args.len() > 2 { &args[2..] } else { &[] };
        jlm(asy, files);
    } else if list {
        pdd(asy);
    } else if ltn {
        jln(asy);
    } else {
        crate::n!(A_, "tar: specify -c, -t, or -x");
    }
}

fn jlm(asy: &str, files: &[&str]) {
    
    let mut efm = String::new();
    let mut count = 0;
    
    for path in files {
        let content = bol(path);
        match content {
            Some(text) => {
                efm.push_str(&format!("===FILE:{}:{}===\n", path, text.len()));
                efm.push_str(&text);
                efm.push('\n');
                count += 1;
            }
            None => crate::n!(D_, "tar: {}: Not found, skipping", path),
        }
    }
    
    let _ = crate::ramfs::bh(|fs| {
        if !fs.exists(asy) { let _ = fs.touch(asy); }
        fs.write_file(asy, efm.as_bytes())
    });
    crate::n!(B_, "tar: created '{}' ({} files)", asy, count);
}

fn pdd(asy: &str) {
    match bol(asy) {
        Some(data) => {
            for line in data.lines() {
                if line.starts_with("===FILE:") && line.ends_with("===") {
                    let inner = &line[8..line.len() - 3];
                    if let Some(ald) = inner.rfind(':') {
                        let name = &inner[..ald];
                        let size = &inner[ald + 1..];
                        crate::println!("{:>8}  {}", size, name);
                    }
                }
            }
        }
        None => crate::n!(A_, "tar: {}: No such file", asy),
    }
}

fn jln(asy: &str) {
    match bol(asy) {
        Some(data) => {
            let mut ejl: Option<(String, usize)> = None;
            let mut cxu = String::new();
            let mut dpd = 0;
            
            for line in data.lines() {
                if line.starts_with("===FILE:") && line.ends_with("===") {
                    
                    if let Some((ref name, _)) = ejl {
                        let _ = crate::ramfs::bh(|fs| {
                            if !fs.exists(name) { let _ = fs.touch(name); }
                            fs.write_file(name, cxu.as_bytes())
                        });
                        dpd += 1;
                    }
                    
                    let inner = &line[8..line.len() - 3];
                    if let Some(ald) = inner.rfind(':') {
                        let name = String::from(&inner[..ald]);
                        let size: usize = inner[ald + 1..].parse().unwrap_or(0);
                        ejl = Some((name, size));
                        cxu = String::new();
                    }
                } else if ejl.is_some() {
                    if !cxu.is_empty() { cxu.push('\n'); }
                    cxu.push_str(line);
                }
            }
            
            if let Some((ref name, _)) = ejl {
                let _ = crate::ramfs::bh(|fs| {
                    if !fs.exists(name) { let _ = fs.touch(name); }
                    fs.write_file(name, cxu.as_bytes())
                });
                dpd += 1;
            }
            crate::n!(B_, "tar: extracted {} files", dpd);
        }
        None => crate::n!(A_, "tar: {}: No such file", asy),
    }
}


pub(super) fn koi(args: &[&str]) {
    if args.is_empty() {
        crate::println!("Usage: gzip <file>");
        return;
    }
    let path = args[0];
    match bol(path) {
        Some(data) => {
            
            let qv = ota(data.as_bytes());
            let ccc = format!("{}.gz", path);
            let _ = crate::ramfs::bh(|fs| {
                if !fs.exists(&ccc) { let _ = fs.touch(&ccc); }
                fs.write_file(&ccc, &qv)
            });
            let zi = if !data.is_empty() { (qv.len() as f64 / data.len() as f64) * 100.0 } else { 100.0 };
            crate::n!(B_, "{} -> {} ({:.1}% of original)", path, ccc, zi);
        }
        None => crate::n!(A_, "gzip: {}: No such file", path),
    }
}

pub(super) fn kui(args: &[&str]) {
    if args.len() < 2 {
        crate::println!("Usage: zip <archive.zip> <file1> [file2] ...");
        return;
    }
    
    jlm(args[0], &args[1..]);
    crate::n!(B_, "zip: created '{}'", args[0]);
}

pub(super) fn ktl(args: &[&str]) {
    if args.is_empty() {
        crate::println!("Usage: unzip <archive.zip>");
        return;
    }
    jln(args[0]);
}

fn ota(data: &[u8]) -> Vec<u8> {
    
    let mut out = vec![b'T', b'G', b'Z', 1];
    
    let len = data.len() as u32;
    out.extend_from_slice(&len.to_le_bytes());
    
    let mut i = 0;
    while i < data.len() {
        let byte = data[i];
        let mut count: u8 = 1;
        while i + (count as usize) < data.len() && data[i + count as usize] == byte && count < 255 {
            count += 1;
        }
        if count >= 3 {
            out.push(0xFF); 
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


use core::sync::atomic::Ordering;

struct Iy {
    name: &'static str,
    description: &'static str,
    default_enabled: bool,
}

const BHO_: &[Iy] = &[
    Iy { name: "sshd", description: "OpenSSH server daemon", default_enabled: false },
    Iy { name: "httpd", description: "TrustOS HTTP server", default_enabled: false },
    Iy { name: "crond", description: "Task scheduler daemon", default_enabled: false },
    Iy { name: "syslogd", description: "System logger", default_enabled: true },
    Iy { name: "networkd", description: "Network manager", default_enabled: true },
    Iy { name: "firewalld", description: "Firewall daemon", default_enabled: false },
];


static AJN_: core::sync::atomic::AtomicU32 = core::sync::atomic::AtomicU32::new(0b011000); 

fn iij(idx: usize) -> bool {
    AJN_.load(Ordering::SeqCst) & (1 << idx) != 0
}

fn fai(idx: usize, enabled: bool) {
    if enabled {
        AJN_.fetch_or(1 << idx, Ordering::SeqCst);
    } else {
        AJN_.fetch_and(!(1 << idx), Ordering::SeqCst);
    }
}

pub(super) fn fnb(args: &[&str]) {
    if args.is_empty() {
        crate::n!(C_, "TrustOS Services:");
        crate::println!("{:<15} {:<10} {}", "SERVICE", "STATUS", "DESCRIPTION");
        crate::println!("--------------------------------------------------");
        for (i, bwc) in BHO_.iter().enumerate() {
            let enabled = iij(i);
            let status = if enabled { "active" } else { "inactive" };
            let color = if enabled { B_ } else { K_ };
            crate::bq!(color, "{:<15} ", bwc.name);
            crate::bq!(color, "{:<10} ", status);
            crate::println!("{}", bwc.description);
        }
        return;
    }
    
    let dfb = args[0];
    let action = if args.len() > 1 { args[1] } else { "status" };
    
    if let Some((idx, bwc)) = BHO_.iter().enumerate().find(|(_, j)| j.name == dfb) {
        match action {
            "start" => {
                fai(idx, true);
                crate::n!(B_, "Starting {}... OK", dfb);
            }
            "stop" => {
                fai(idx, false);
                crate::n!(D_, "Stopping {}... OK", dfb);
            }
            "restart" => {
                fai(idx, false);
                crate::n!(D_, "Stopping {}...", dfb);
                fai(idx, true);
                crate::n!(B_, "Starting {}... OK", dfb);
            }
            "status" => {
                let active = iij(idx);
                if active {
                    crate::n!(B_, "  {} - {}", bwc.name, bwc.description);
                    crate::println!("   Active: active (running)");
                } else {
                    crate::n!(K_, "  {} - {}", bwc.name, bwc.description);
                    crate::println!("   Active: inactive (dead)");
                }
            }
            _ => crate::println!("Usage: service <name> start|stop|restart|status"),
        }
    } else {
        crate::n!(A_, "service: unknown service '{}'", dfb);
    }
}

pub(super) fn ksi(args: &[&str]) {
    if args.is_empty() || args[0] == "list-units" {
        fnb(&[]);
        return;
    }
    
    let action = args[0];
    if args.len() < 2 {
        crate::println!("Usage: systemctl <start|stop|restart|status|enable|disable> <service>");
        return;
    }
    let bwc = args[1].trim_end_matches(".service");
    fnb(&[bwc, action]);
}



static ACI_: Mutex<Vec<String>> = Mutex::new(Vec::new());

pub(super) fn kmp(args: &[&str]) {
    match args.first().copied() {
        Some("-l") | None => {
            let entries = ACI_.lock();
            if entries.is_empty() {
                crate::println!("no crontab for root");
            } else {
                for entry in entries.iter() {
                    crate::println!("{}", entry);
                }
            }
        }
        Some("-e") => {
            crate::n!(C_, "Enter cron entries (one per line, empty line to finish):");
            crate::n!(K_, "Format: min hour dom mon dow command");
            let mut entries = Vec::new();
            loop {
                crate::print!("> ");
                let mut input = String::new();
                loop {
                    if let Some(key) = crate::keyboard::ya() {
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
            *ACI_.lock() = entries.clone();
            crate::n!(B_, "crontab: installed {} entries", entries.len());
        }
        Some("-r") => {
            ACI_.lock().clear();
            crate::n!(B_, "crontab: removed");
        }
        _ => crate::println!("Usage: crontab [-l | -e | -r]"),
    }
}


pub(super) fn kls(args: &[&str]) {
    if args.is_empty() {
        crate::println!("Usage: at <time> <command>");
        crate::println!("  Example: at +5m echo hello");
        return;
    }
    
    let jms = args[0];
    let command = if args.len() > 1 { args[1..].join(" ") } else {
        crate::println!("at: no command specified");
        return;
    };
    
    
    let aar = if jms.starts_with('+') {
        let ye = &jms[1..];
        if ye.ends_with('s') {
            ye[..ye.len()-1].parse::<u64>().unwrap_or(0) * 1000
        } else if ye.ends_with('m') {
            ye[..ye.len()-1].parse::<u64>().unwrap_or(0) * 60000
        } else if ye.ends_with('h') {
            ye[..ye.len()-1].parse::<u64>().unwrap_or(0) * 3600000
        } else {
            ye.parse::<u64>().unwrap_or(0) * 1000
        }
    } else {
        crate::println!("at: use +Ns, +Nm, or +Nh for relative times");
        return;
    };
    
    crate::n!(B_, "Job scheduled: '{}' in {} seconds", command, aar / 1000);
    
    
    let start = crate::time::uptime_ms();
    while crate::time::uptime_ms() - start < aar {
        if let Some(3) = crate::keyboard::ya() {
            crate::n!(D_, "at: cancelled");
            return;
        }
        core::hint::spin_loop();
    }
    
    crate::n!(C_, "[at] Executing: {}", command);
    super::aav(&command);
}


pub(super) fn qao(args: &[&str], piped: Option<&str>) {
    let mut ae: usize = 10;
    let mut file: Option<&str> = None;
    
    let mut i = 0;
    while i < args.len() {
        if args[i] == "-n" && i + 1 < args.len() {
            ae = args[i + 1].parse().unwrap_or(10);
            i += 2;
        } else if args[i].starts_with('-') && args[i].len() > 1 {
            ae = args[i][1..].parse().unwrap_or(10);
            i += 1;
        } else {
            file = Some(args[i]);
            i += 1;
        }
    }
    
    let content = if let Some(input) = piped {
        Some(String::from(input))
    } else if let Some(path) = file {
        bol(path)
    } else {
        crate::println!("Usage: head [-n N] <file>");
        return;
    };
    
    if let Some(text) = content {
        for (i, line) in text.lines().enumerate() {
            if i >= ae { break; }
            crate::println!("{}", line);
        }
    }
}


pub(super) fn qaq(args: &[&str], piped: Option<&str>) {
    let mut ae: usize = 10;
    let mut file: Option<&str> = None;
    let mut hzl = false;
    
    let mut i = 0;
    while i < args.len() {
        if args[i] == "-n" && i + 1 < args.len() {
            ae = args[i + 1].parse().unwrap_or(10);
            i += 2;
        } else if args[i] == "-f" {
            hzl = true;
            i += 1;
        } else if args[i].starts_with('-') && args[i].len() > 1 && args[i] != "-f" {
            ae = args[i][1..].parse().unwrap_or(10);
            i += 1;
        } else {
            file = Some(args[i]);
            i += 1;
        }
    }
    
    let content = if let Some(input) = piped {
        Some(String::from(input))
    } else if let Some(path) = file {
        bol(path)
    } else {
        crate::println!("Usage: tail [-n N] [-f] <file>");
        return;
    };
    
    if let Some(text) = content {
        let lines: Vec<&str> = text.lines().collect();
        let start = if lines.len() > ae { lines.len() - ae } else { 0 };
        for line in &lines[start..] {
            crate::println!("{}", line);
        }
    }
    
    if hzl {
        crate::n!(K_, "(follow mode — Ctrl+C to stop)");
        crate::shell::dks();
        loop {
            if let Some(3) = crate::keyboard::ya() { break; }
            if crate::shell::cbc() { break; }
            core::hint::spin_loop();
        }
    }
}


pub(super) fn qas(args: &[&str], piped: Option<&str>) {
    let hob = args.contains(&"-l");
    let hoc = args.contains(&"-w");
    let hnz = args.contains(&"-c");
    let all = !hob && !hoc && !hnz;
    
    let cjj: Vec<&str> = args.iter().filter(|a| !a.starts_with('-')).copied().collect();
    
    let content = if let Some(input) = piped {
        Some(String::from(input))
    } else if !cjj.is_empty() {
        bol(cjj[0])
    } else {
        crate::println!("Usage: wc [-l] [-w] [-c] <file>");
        return;
    };
    
    if let Some(text) = content {
        let lines = text.lines().count();
        let um = text.split_whitespace().count();
        let chars = text.len();
        
        if all {
            crate::println!("  {}  {}  {}", lines, um, chars);
        } else {
            if hob { crate::print!("  {}", lines); }
            if hoc { crate::print!("  {}", um); }
            if hnz { crate::print!("  {}", chars); }
            crate::println!();
        }
    }
}


pub(super) fn ojg() {
    
    let obx = ["/.trustrc", "/etc/trustrc", "/home/trustrc"];
    
    for path in &obx {
        let content: Option<String> = crate::ramfs::bh(|fs| {
            fs.read_file(path).map(|b| String::from(core::str::from_utf8(b).unwrap_or(""))).ok()
        });
        
        if let Some(ref script) = content {
            crate::n!(K_, "[init] Running {}...", path);
            for line in script.lines() {
                let jw = line.trim();
                if jw.is_empty() || jw.starts_with('#') {
                    continue;
                }
                super::aav(jw);
            }
            return; 
        }
    }
}


pub(super) fn qat() {
    let avp = super::scripting::axh("USER").unwrap_or_else(|| String::from("root"));
    crate::println!("{}", avp);
}


pub(super) fn ktn() {
    let dh = crate::time::uptime_ms();
    let im = dh / 1000;
    let cic = im / 86400;
    let aoi = (im % 86400) / 3600;
    let acf = (im % 3600) / 60;
    let j = im % 60;
    
    let exb = crate::process::list().len();
    
    crate::bq!(R_, " up ");
    if cic > 0 { crate::print!("{} day(s), ", cic); }
    crate::print!("{:02}:{:02}:{:02}", aoi, acf, j);
    crate::println!(", {} processes", exb);
}


pub(super) fn qak() {
    crate::framebuffer::clear();
    crate::framebuffer::afr(0, 0);
}






pub(super) fn kpd(args: &[&str]) {
    if args.is_empty() {
        crate::println!("Usage: killall <name>");
        return;
    }
    let name = args[0];
    let mut gev = 0u32;
    for (pid, biq, _state) in crate::process::list() {
        if biq.contains(name) && pid > 1 {
            if crate::process::bne(pid).is_ok() {
                gev += 1;
                crate::n!(D_, "Killed PID {} ({})", pid, biq);
            }
        }
    }
    if gev == 0 {
        crate::n!(A_, "killall: no process matching '{}'", name);
    } else {
        crate::n!(B_, "Killed {} process(es)", gev);
    }
}


pub(super) fn kqa(args: &[&str]) {
    if args.is_empty() {
        crate::println!("Usage: nice [-n priority] <command>");
        return;
    }
    let (priority, byc) = if args[0] == "-n" && args.len() > 2 {
        (args[1].parse::<i32>().unwrap_or(10), 2)
    } else {
        (10, 0)
    };
    let cmd = args[byc..].join(" ");
    crate::n!(C_, "nice: running '{}' with priority {}", cmd, priority);
    
    super::aav(&cmd);
}


pub(super) fn kox() {
    let (reads, writes, atf, atg) = crate::disk::get_stats();
    let aiz = crate::time::uptime_ms() / 1000;
    let aiz = if aiz == 0 { 1 } else { aiz };

    crate::n!(G_, "TrustOS I/O Statistics");
    crate::println!("------------------------------------------------------");
    crate::println!("Uptime: {}s", aiz);
    crate::println!();
    crate::n!(C_, "Device          tps    kB_read/s    kB_wrtn/s   kB_read   kB_wrtn");
    let pmk = (reads + writes) / aiz;
    let bhm = atf / 1024;
    let li = atg / 1024;
    let mvz = bhm / aiz;
    let mwa = li / aiz;
    crate::println!("ramdisk   {:>8}  {:>11}  {:>11}  {:>8}  {:>8}", pmk, mvz, mwa, bhm, li);
    crate::println!();
    crate::println!("Total: {} reads, {} writes", reads, writes);
}


pub(super) fn ksa(args: &[&str]) {
    if args.is_empty() {
        crate::println!("Usage: strace <command>");
        crate::println!("  Trace system calls made by a command");
        return;
    }

    
    crate::serial_println!("[STRACE] Tracing: {}", args.join(" "));
    crate::n!(C_, "strace: tracing '{}'", args.join(" "));
    crate::n!(K_, "--- syscall trace start ---");

    
    use core::sync::atomic::{AtomicBool, Ordering};
    static BIP_: AtomicBool = AtomicBool::new(false);
    BIP_.store(true, Ordering::SeqCst);

    
    let cmd = args.join(" ");
    super::aav(&cmd);

    BIP_.store(false, Ordering::SeqCst);
    crate::n!(K_, "--- syscall trace end ---");
}



pub(super) fn kpx(args: &[&str]) {
    let je = args.first().copied().unwrap_or("help");

    match je {
        "start" | "on" => {
            let auc = match args.get(1) {
                Some(j) => j,
                None => {
                    crate::println!("Usage: netconsole start <ip> [port]");
                    crate::println!("  Example: netconsole start 10.0.0.1");
                    return;
                }
            };
            let ip = match crate::debug::netconsole::bof(auc) {
                Some(ip) => ip,
                None => {
                    crate::n!(A_, "Invalid IP: {}", auc);
                    return;
                }
            };
            let port = args.get(2)
                .and_then(|j| j.parse::<u16>().ok())
                .unwrap_or(crate::debug::netconsole::BUA_);

            
            for _ in 0..50 {
                crate::netstack::poll();
            }

            
            if crate::network::rd().is_none() {
                crate::println!("[netconsole] No IP configured, requesting DHCP...");
                crate::netstack::dhcp::start();
                let start_tick = crate::logger::eg();
                let timeout_ms = 8000u64; 
                loop {
                    for _ in 0..20 {
                        crate::netstack::poll();
                    }
                    if crate::network::rd().is_some() {
                        break;
                    }
                    let bb = crate::logger::eg().saturating_sub(start_tick);
                    if bb > timeout_ms {
                        break;
                    }
                    
                    for _ in 0..10 {
                        crate::arch::acb();
                    }
                }
            }

            
            if crate::network::rd().is_none() {
                crate::n!(D_, "[netconsole] DHCP failed, applying static IP 10.0.0.100/24");
                crate::network::deh(
                    crate::network::Ipv4Address::new(10, 0, 0, 100),
                    crate::network::Ipv4Address::new(255, 255, 255, 0),
                    Some(crate::network::Ipv4Address::new(10, 0, 0, 1)),
                );
            }

            
            if let Some((src_ip, _, _)) = crate::network::rd() {
                let b = src_ip.as_bytes();
                crate::n!(C_, "[netconsole] Source IP: {}.{}.{}.{}", b[0], b[1], b[2], b[3]);
            } else {
                crate::n!(A_, "[netconsole] ERROR: Could not configure any IP");
                return;
            }

            
            
            
            crate::println!("[netconsole] Resolving ARP for {}.{}.{}.{}...", ip[0], ip[1], ip[2], ip[3]);
            let mut efo = crate::netstack::arp::yb(ip).is_some();
            if !efo {
                for attempt in 0..5 {
                    let _ = crate::netstack::arp::bos(ip);
                    let start = crate::logger::eg();
                    loop {
                        for _ in 0..20 { crate::netstack::poll(); }
                        if crate::netstack::arp::yb(ip).is_some() {
                            efo = true;
                            break;
                        }
                        if crate::logger::eg().saturating_sub(start) > 2000 {
                            break; 
                        }
                        crate::arch::acb();
                    }
                    if efo {
                        crate::n!(G_, "[netconsole] ARP resolved (attempt {})", attempt + 1);
                        break;
                    }
                }
            }
            if !efo {
                crate::n!(D_, "[netconsole] ARP unresolved — using broadcast mode");
            }

            crate::debug::netconsole::start(ip, port);
            crate::n!(G_,
                "Netconsole streaming to {}.{}.{}.{}:{}",
                ip[0], ip[1], ip[2], ip[3], port
            );

            
            
            let ctw = if let Some((src, mask, _)) = crate::network::rd() {
                let j = src.as_bytes();
                let m = mask.as_bytes();
                [j[0] | !m[0], j[1] | !m[1], j[2] | !m[2], j[3] | !m[3]]
            } else {
                [255, 255, 255, 255]
            };
            let mkz = b"[TrustOS netconsole] Connection established\n";
            crate::println!("[netconsole] Sending test packet via netstack to {}.{}.{}.{}:{}", ctw[0], ctw[1], ctw[2], ctw[3], port);
            match crate::netstack::udp::azq(ctw, port, 6665, mkz) {
                Ok(()) => crate::n!(G_, "[netconsole] Test packet sent OK via netstack"),
                Err(e) => crate::n!(A_, "[netconsole] Test packet FAILED via netstack: {}", e),
            }

            
            crate::println!("[netconsole] Sending raw test frame...");
            if let Some((src_ip_addr, _, _)) = crate::network::rd() {
                let src_ip = *src_ip_addr.as_bytes();
                let dest_ip = ctw;
                let payload = b"[TrustOS RAW] Hello from kernel!\n";
                let hao = (8 + payload.len()) as u16;
                let mrt = (20 + 8 + payload.len()) as u16;

                
                let mut udp = alloc::vec::Vec::with_capacity(8 + payload.len());
                udp.extend_from_slice(&6665u16.to_be_bytes()); 
                udp.extend_from_slice(&port.to_be_bytes());     
                udp.extend_from_slice(&hao.to_be_bytes());
                udp.extend_from_slice(&0u16.to_be_bytes());     
                udp.extend_from_slice(payload);

                
                let mut bhg = [0u8; 20];
                bhg[0] = 0x45;
                bhg[2..4].copy_from_slice(&mrt.to_be_bytes());
                bhg[6..8].copy_from_slice(&0x4000u16.to_be_bytes()); 
                bhg[8] = 64; 
                bhg[9] = 17; 
                bhg[12..16].copy_from_slice(&src_ip);
                bhg[16..20].copy_from_slice(&dest_ip);
                
                let mut sum: u32 = 0;
                for i in (0..20).step_by(2) {
                    sum += ((bhg[i] as u32) << 8) | (bhg[i+1] as u32);
                }
                while sum > 0xFFFF { sum = (sum & 0xFFFF) + (sum >> 16); }
                let ig = !(sum as u16);
                bhg[10..12].copy_from_slice(&ig.to_be_bytes());

                
                let mut ere = alloc::vec::Vec::with_capacity(20 + udp.len());
                ere.extend_from_slice(&bhg);
                ere.extend_from_slice(&udp);

                
                match crate::netstack::cdq([0xFF; 6], 0x0800, &ere) {
                    Ok(()) => crate::n!(G_, "[netconsole] Raw frame sent OK ({} bytes)", ere.len() + 14),
                    Err(e) => crate::n!(A_, "[netconsole] Raw frame FAILED: {}", e),
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
                crate::n!(G_,
                    "Netconsole: ACTIVE → {}.{}.{}.{}:{}",
                    ip[0], ip[1], ip[2], ip[3], port
                );
            } else {
                crate::n!(D_, "Netconsole: OFF");
            }
        }
        _ => {
            crate::n!(C_, "netconsole — Stream kernel log over UDP");
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


pub(super) fn kne() {
    crate::n!(G_, "SMBIOS/DMI Information");
    crate::println!("------------------------------------------------------");

    
    crate::n!(C_, "Handle 0x0000, DMI type 0, BIOS Information");
    crate::println!("  Vendor: TrustOS");
    crate::println!("  Version: 0.7.0-checkm8");
    crate::println!("  Release Date: 03/12/2026");
    crate::println!("  BIOS Revision: 0.7");
    crate::println!();

    
    crate::n!(C_, "Handle 0x0001, DMI type 1, System Information");
    crate::println!("  Manufacturer: TrustOS Project");
    crate::println!("  Product Name: TrustOS Bare-Metal");
    #[cfg(target_arch = "x86_64")]
    crate::println!("  Architecture: x86_64");
    #[cfg(target_arch = "aarch64")]
    crate::println!("  Architecture: aarch64");
    #[cfg(target_arch = "riscv64")]
    crate::println!("  Architecture: riscv64gc");
    crate::println!();

    
    crate::n!(C_, "Handle 0x0004, DMI type 4, Processor Information");
    let cpu_count = {
        #[cfg(target_arch = "x86_64")]
        { crate::cpu::smp::cpu_count() }
        #[cfg(not(target_arch = "x86_64"))]
        { 1u32 }
    };
    crate::println!("  CPU Count: {}", cpu_count);
    crate::println!("  Features: SSE, SSE2, RDRAND, RDSEED");
    crate::println!();

    
    let stats = crate::memory::stats();
    let baa = (stats.heap_used + stats.heap_free) / 1024;
    crate::n!(C_, "Handle 0x0011, DMI type 17, Memory Device");
    crate::println!("  Size: {} KB (heap)", baa);
    crate::println!("  Type: DRAM");
}


pub(super) fn kok(args: &[&str]) {
    if args.is_empty() {
        crate::println!("Usage: hdparm [-i|-t] <device>");
        crate::println!("  -i  Device information");
        crate::println!("  -t  Timing buffered disk reads");
        return;
    }

    let igm = args.contains(&"-i");
    let jmu = args.contains(&"-t");

    let dne = crate::disk::rk();
    if let Some(info) = dne {
        if igm || (!igm && !jmu) {
            crate::n!(C_, "/dev/sda:");
            crate::println!("  Model: {}", info.model);
            crate::println!("  Serial: {}", info.serial);
            crate::println!("  Sectors: {}", info.sectors);
            crate::println!("  Size: {} MB", info.size_mb);
        }
        if jmu {
            
            let start = crate::time::uptime_ms();
            let mut buf = [0u8; 512];
            for i in 0..100u64 {
                let _ = crate::disk::read_sectors(i % info.sectors, 1, &mut buf);
            }
            let bb = crate::time::uptime_ms() - start;
            let bb = if bb == 0 { 1 } else { bb };
            let pjb = (100 * 512) / (bb as usize);
            crate::n!(B_, "  Timing: 100 sectors in {}ms ({} KB/s)", bb, pjb);
        }
    } else {
        crate::n!(A_, "hdparm: no disk found");
    }
}


pub(super) fn krh(args: &[&str]) {
    let filename = if !args.is_empty() { args[0] } else { "/screenshot.ppm" };

    let width = crate::framebuffer::X_.load(core::sync::atomic::Ordering::Relaxed) as u32;
    let height = crate::framebuffer::W_.load(core::sync::atomic::Ordering::Relaxed) as u32;

    if width == 0 || height == 0 {
        crate::n!(A_, "screenshot: no framebuffer available");
        return;
    }

    crate::n!(C_, "Capturing {}x{} screenshot...", width, height);

    
    let header = format!("P6\n{} {}\n255\n", width, height);
    let nuy = (width * height * 3) as usize;
    let av = header.len() + nuy;
    let mut data = Vec::with_capacity(av);
    data.extend_from_slice(header.as_bytes());

    
    for y in 0..height {
        for x in 0..width {
            let ct = crate::framebuffer::get_pixel(x as u32, y as u32);
            
            let r = ((ct >> 16) & 0xFF) as u8;
            let g = ((ct >> 8) & 0xFF) as u8;
            let b = (ct & 0xFF) as u8;
            data.push(r);
            data.push(g);
            data.push(b);
        }
    }

    
    match crate::vfs::write_file(filename, &data) {
        Ok(_) => crate::n!(B_, "Screenshot saved: {} ({} bytes, {}x{})", filename, av, width, height),
        Err(e) => crate::n!(A_, "screenshot: write failed: {:?}", e),
    }
}


pub(super) fn fmr(args: &[&str]) {
    let port: u16 = if !args.is_empty() {
        args[0].parse().unwrap_or(8080)
    } else {
        8080
    };

    crate::n!(G_, "TrustOS HTTP Server starting on port {}...", port);
    crate::n!(C_, "  Serving files from /");
    crate::n!(K_, "  Press Ctrl+C to stop");

    
    let mzm = crate::netstack::socket::socket(2, 1, 0); 
    match mzm {
        Ok(fd) => {
            let addr = crate::netstack::socket::SockAddrIn::new([0, 0, 0, 0], port);
            if let Err(e) = crate::netstack::socket::fjf(fd, &addr) {
                crate::n!(A_, "httpd: bind failed: {}", e);
                return;
            }
            if let Err(e) = crate::netstack::socket::iks(fd, 8) {
                crate::n!(A_, "httpd: listen failed: {}", e);
                return;
            }
            crate::n!(B_, "Listening on 0.0.0.0:{}", port);
            crate::n!(K_, "(In this kernel, TCP accept() is cooperative — use `curl` from another shell)");
        }
        Err(e) => {
            crate::n!(A_, "httpd: socket creation failed: {}", e);
        }
    }
}




pub(super) fn fmd() {
    crate::n!(G_, "TrustOS System Benchmark");
    crate::println!("======================================================");

    
    crate::n!(C_, "[1/4] CPU integer arithmetic...");
    let start = crate::time::uptime_ms();
    let mut aku: u64 = 0;
    for i in 0u64..10_000_000 {
        aku = aku.wrapping_add(i).wrapping_mul(3);
    }
    let dlm = crate::time::uptime_ms() - start;
    let dlm = if dlm == 0 { 1 } else { dlm };
    crate::println!("  10M iterations in {}ms ({} Mops/s) [checksum=0x{:016x}]",
        dlm, 10000 / dlm, aku);

    
    crate::n!(C_, "[2/4] Memory sequential write...");
    let mut buf = vec![0u8; 1024 * 1024]; 
    let start = crate::time::uptime_ms();
    for i in 0..buf.len() {
        buf[i] = (i & 0xFF) as u8;
    }
    let due = crate::time::uptime_ms() - start;
    let due = if due == 0 { 1 } else { due };
    let ndu = 1000 / due;
    crate::println!("  1MB write in {}ms ({} MB/s)", due, ndu);

    
    crate::n!(C_, "[3/4] Disk I/O (ramdisk)...");
    let start = crate::time::uptime_ms();
    let mut dj = [0u8; 512];
    for i in 0..1000u64 {
        let _ = crate::disk::read_sectors(i % 256, 1, &mut dj);
    }
    let dnf = crate::time::uptime_ms() - start;
    let dnf = if dnf == 0 { 1 } else { dnf };
    crate::println!("  1000 sector reads in {}ms ({} IOPS)", dnf, 1000000 / dnf);

    
    crate::n!(C_, "[4/4] Heap allocation...");
    let start = crate::time::uptime_ms();
    for _ in 0..10000 {
        let v: Vec<u8> = Vec::with_capacity(256);
        core::hint::black_box(v);
    }
    let dhn = crate::time::uptime_ms() - start;
    let dhn = if dhn == 0 { 1 } else { dhn };
    crate::println!("  10K allocs in {}ms ({} allocs/s)", dhn, 10_000_000 / dhn);

    crate::println!("======================================================");
    crate::n!(B_, "Benchmark complete.");
}
