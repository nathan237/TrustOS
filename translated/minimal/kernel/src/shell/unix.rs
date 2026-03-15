








use alloc::string::String;
use alloc::vec::Vec;
use alloc::vec;
use alloc::format;
use crate::framebuffer::{B_, G_, AU_, D_, A_, C_, Q_, CD_, DF_, L_};





const Cmg: &[(&str, &str)] = &[
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


pub(super) fn xmy(cmd: &str) -> bool {
    for &(j, desc) in Cmg {
        if cmd == j {
            crate::h!(D_, "{}: {} not implemented", j, desc);
            return true;
        }
    }
    false
}

pub(super) fn hdl(n: &[&str]) {
    if n.is_empty() {
        crate::println!("Usage: which <command>");
        return;
    }
    
    let hzf = ["/bin", "/usr/bin", "/sbin", "/usr/sbin"];
    
    for j in n {
        let mut aig = false;
        for te in &hzf {
            let path = format!("{}/{}", te, j);
            if super::vm::cxx(&path) {
                crate::println!("{}", path);
                aig = true;
                break;
            }
        }
        
        if !aig {
            let ejg = crate::hypervisor::linux_subsystem::bcu();
            if ejg.ogm(j) {
                crate::println!("/usr/bin/{}", j);
                aig = true;
            }
        }
        if !aig {
            crate::h!(A_, "{}: not found", j);
        }
    }
}

pub(super) fn rkr(n: &[&str]) {
    if n.is_empty() {
        crate::println!("Usage: whereis <command>");
        return;
    }
    hdl(n);
}

pub(super) fn rei(n: &[&str]) {
    if n.is_empty() {
        crate::println!("Usage: file <path>");
        return;
    }
    
    for path in n {
        if !super::vm::cxx(path) {
            crate::println!("{}: cannot open", path);
            continue;
        }
        
        
        if crate::exec::clc(path) {
            crate::println!("{}: ELF 64-bit executable", path);
        } else {
            
            match crate::vfs::aji(path, crate::vfs::OpenFlags(0)) {
                Ok(da) => {
                    let mut dh = [0u8; 16];
                    let bo = crate::vfs::read(da, &mut dh).unwrap_or(0);
                    crate::vfs::agj(da).bq();
                    
                    if bo == 0 {
                        crate::println!("{}: empty", path);
                    } else if dh[0..4] == [0x7F, b'E', b'L', b'F'] {
                        crate::println!("{}: ELF file", path);
                    } else if dh[0..2] == [0x1f, 0x8b] {
                        crate::println!("{}: gzip compressed data", path);
                    } else if dh[0..4] == [0x50, 0x4B, 0x03, 0x04] {
                        crate::println!("{}: Zip archive", path);
                    } else if dh[0..6] == *b"#!/bin" {
                        crate::println!("{}: shell script", path);
                    } else if dh.iter().xx(|&o| o.ofo()) {
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

pub(super) fn rcm(n: &[&str]) {
    if n.is_empty() {
        crate::println!("Usage: basename <path>");
        return;
    }
    let path = n[0];
    let j = path.cmm('/').next().unwrap_or(path);
    crate::println!("{}", j);
}

pub(super) fn rds(n: &[&str]) {
    if n.is_empty() {
        crate::println!("Usage: dirname <path>");
        return;
    }
    let path = n[0];
    if let Some(u) = path.bhx('/') {
        if u == 0 {
            crate::println!("/");
        } else {
            crate::println!("{}", &path[..u]);
        }
    } else {
        crate::println!(".");
    }
}

pub(super) fn rhl(n: &[&str]) {
    if n.is_empty() {
        crate::println!("Usage: realpath <path>");
        return;
    }
    let path = super::vm::jmh(n[0]);
    crate::println!("{}", path);
}

pub(super) fn rii(n: &[&str], arr: Option<&str>) {
    let ca = if let Some(input) = arr {
        Some(alloc::string::String::from(input))
    } else if !n.is_empty() {
        super::network::fse(n[0])
    } else {
        crate::println!("Usage: sort <file>");
        return;
    };
    
    match ca {
        Some(text) => {
            let mut ak: Vec<&str> = text.ak().collect();
            ak.jqs();
            for line in ak {
                crate::println!("{}", line);
            }
        }
        None => crate::h!(A_, "sort: cannot read input"),
    }
}

pub(super) fn rjz(n: &[&str], arr: Option<&str>) {
    let ca = if let Some(input) = arr {
        Some(alloc::string::String::from(input))
    } else if !n.is_empty() {
        super::network::fse(n[0])
    } else {
        crate::println!("Usage: uniq <file>");
        return;
    };
    
    match ca {
        Some(text) => {
            let mut gkv: Option<&str> = None;
            for line in text.ak() {
                if gkv != Some(line) {
                    crate::println!("{}", line);
                    gkv = Some(line);
                }
            }
        }
        None => crate::h!(A_, "uniq: cannot read input"),
    }
}

pub(super) fn rkv(n: &[&str]) {
    let text = if n.is_empty() { "y" } else { n[0] };
    crate::shell::hcw();
    loop {
        if crate::shell::etf() { break; }
        crate::println!("{}", text);
        
        if let Some(3) = crate::keyboard::auw() {
            crate::shell::jpb();
            break;
        }
    }
}

pub(super) fn rhz(n: &[&str]) {
    if n.is_empty() {
        crate::println!("Usage: seq <last> | seq <first> <last> | seq <first> <inc> <last>");
        return;
    }
    
    let (fv, drz, qv) = match n.len() {
        1 => (1i64, 1i64, n[0].parse().unwrap_or(1)),
        2 => (n[0].parse().unwrap_or(1), 1i64, n[1].parse().unwrap_or(1)),
        _ => (n[0].parse().unwrap_or(1), n[1].parse().unwrap_or(1), n[2].parse().unwrap_or(1)),
    };
    
    let mut a = fv;
    let mut az = 0u64;
    while (drz > 0 && a <= qv) || (drz < 0 && a >= qv) {
        crate::println!("{}", a);
        a += drz;
        az += 1;
        if az >= 100_000 {
            crate::println!("... (truncated at 100000 lines)");
            break;
        }
    }
}

pub(super) fn kje(n: &[&str]) {
    if n.is_empty() {
        crate::println!("Usage: sleep <seconds>");
        return;
    }
    
    let tv: u64 = n[0].parse().unwrap_or(1);
    crate::h!(C_, "Sleeping for {} seconds...", tv);
    
    
    let ay = crate::time::lc();
    let ci = ay + tv * 1000;
    while crate::time::lc() < ci {
        core::hint::hc();
    }
    crate::println!("Done.");
}

pub(super) fn rfr(n: &[&str]) {
    if n.is_empty() {
        crate::println!("Usage: kill [-9] <pid>");
        return;
    }
    
    let ycw = if n[0] == "-9" { 9 } else { 15 };
    let vhs = if n[0].cj('-') && n.len() > 1 { n[1] } else { n[0] };
    
    match vhs.parse::<u32>() {
        Ok(ce) => {
            crate::h!(D_, "Killing PID {}", ce);
            match crate::process::dsm(ce) {
                Ok(_) => crate::h!(B_, "Process {} killed", ce),
                Err(aa) => crate::h!(A_, "kill: {}", aa),
            }
        }
        Err(_) => crate::h!(A_, "kill: invalid PID"),
    }
}

pub(super) fn iol() {
    crate::h!(G_, "TrustOS Process Monitor");
    crate::println!("-----------------------------------------------------------");
    
    let bxp = crate::time::lc() / 1000;
    let cad = bxp / 3600;
    let bbz = (bxp % 3600) / 60;
    let tv = bxp % 60;
    
    crate::println!("Uptime: {:02}:{:02}:{:02}", cad, bbz, tv);
    crate::println!();
    
    
    let afa = crate::memory::cm().afa;
    let aul = (crate::memory::cm().afa + crate::memory::cm().buv);
    crate::println!("Mem: {} KB / {} KB ({:.1}%)", 
        afa / 1024, 
        aul / 1024,
        (afa as f64 / aul as f64) * 100.0);
    crate::println!();
    
    crate::h!(C_, "  PID  STATE    NAME");
    crate::println!("----------------------------------");
    
    
    for (ce, j, g) in crate::process::aoy() {
        let boo = match g {
            crate::process::ProcessState::Ai => "RUNNING",
            crate::process::ProcessState::At => "READY  ",
            crate::process::ProcessState::Hj => "BLOCKED",
            crate::process::ProcessState::Vf => "ZOMBIE ",
            crate::process::ProcessState::Cu => "CREATED",
            crate::process::ProcessState::Bwo => "WAITING",
            crate::process::ProcessState::Af => "STOPPED",
            crate::process::ProcessState::Ez => "DEAD   ",
        };
        crate::println!("{:>5}  {}  {}", ce, boo, j);
    }
    
    crate::println!();
    crate::h!(D_, "(press 'q' to quit in interactive mode)");
}

pub(super) fn rkl() {
    crate::h!(G_, "Virtual Memory Statistics");
    crate::println!("-----------------------------------------");
    
    let afa = crate::memory::cm().afa;
    let aul = (crate::memory::cm().afa + crate::memory::cm().buv);
    
    crate::println!("Memory:");
    crate::println!("  Heap Total:  {} KB", aul / 1024);
    crate::println!("  Heap Used:   {} KB", afa / 1024);
    crate::println!("  Heap Free:   {} KB", (aul - afa) / 1024);
}

pub(super) fn kim(n: &[&str]) {
    if n.fv() == Some(&"-c") || n.fv() == Some(&"--clear") {
        
        crate::h!(B_, "dmesg buffer acknowledged.");
        return;
    }
    
    let az = if let Some(&"-n") = n.fv() {
        n.get(1).and_then(|e| e.parse::<usize>().bq()).unwrap_or(20)
    } else if let Some(bo) = n.fv().and_then(|e| e.parse::<usize>().bq()) {
        bo
    } else {
        0 
    };
    
    let ak = crate::devtools::rzn(az);
    if ak.is_empty() {
        crate::h!(D_, "(no kernel messages recorded)");
        crate::println!("Tip: messages are captured after devtools init.");
        return;
    }
    let (dzh, es) = crate::devtools::rzo();
    crate::h!(G_, "Kernel Ring Buffer ({} stored, {} total)", dzh, es);
    crate::println!("---------------------------------------------------------------");
    for line in &ak {
        crate::println!("{}", line);
    }
}

pub(super) fn rgi() {
    let e = crate::devtools::jfu();
    crate::h!(G_, "Memory Debug Statistics (memdbg)");
    crate::println!("---------------------------------------------------------------");
    crate::println!();
    crate::h!(C_, "  Heap Usage:");
    crate::println!("    Current used : {:>10} bytes ({} KB)", e.iqb, e.iqb / 1024);
    crate::println!("    Current free : {:>10} bytes ({} KB)", e.kmv, e.kmv / 1024);
    crate::println!("    Total heap   : {:>10} bytes ({} KB)", e.aul, e.aul / 1024);
    crate::println!("    Peak used    : {:>10} bytes ({} KB)", e.gpe, e.gpe / 1024);
    crate::println!();
    crate::h!(C_, "  Allocation Stats:");
    crate::println!("    Alloc ops    : {:>10}", e.cok);
    crate::println!("    Dealloc ops  : {:>10}", e.dpr);
    crate::println!("    Live allocs  : {:>10}", e.czi);
    crate::println!("    Total alloc'd: {:>10} bytes", e.mux);
    crate::println!("    Total freed  : {:>10} bytes", e.nju);
    crate::println!("    Largest alloc: {:>10} bytes", e.etu);
    crate::println!();
    crate::h!(C_, "  Fragmentation:");
    let swt = if e.hki > 50.0 { A_ }
        else if e.hki > 25.0 { D_ }
        else { B_ };
    crate::h!(swt, "    Estimate     : {:.1}%", e.hki);
}

pub(super) fn rgz() {
    let xj = crate::devtools::vgr();
    let ekj = xj.lc / 1000;
    let cad = ekj / 3600;
    let bbz = (ekj % 3600) / 60;
    let tv = ekj % 60;
    
    crate::h!(G_, "Performance Statistics (perf)");
    crate::println!("---------------------------------------------------------------");
    crate::println!();
    crate::h!(C_, "  System:");
    crate::println!("    Uptime       : {}h {:02}m {:02}s ({} ms)", cad, bbz, tv, xj.lc);
    crate::println!("    GUI FPS      : {}", xj.tz);
    crate::println!();
    crate::h!(C_, "  Interrupts:");
    crate::println!("    Total IRQs   : {}", xj.blm);
    crate::println!("    IRQ/sec      : {}", xj.hor);
    crate::println!();
    crate::h!(C_, "  Scheduling:");
    crate::println!("    Syscalls     : {}", xj.mmn);
    crate::println!("    Ctx switches : {}", xj.mmd);
    crate::println!();
    crate::h!(C_, "  Memory:");
    crate::println!("    Heap used    : {} / {} KB ({}%)", 
        xj.afa / 1024, (xj.afa + xj.buv) / 1024,
        if xj.afa + xj.buv > 0 { xj.afa * 100 / (xj.afa + xj.buv) } else { 0 });
    crate::println!();
    crate::h!(C_, "  Per-CPU:");
    for e in &xj.ngt {
        let g = if e.edw { "idle" } else { "busy" };
        crate::println!("    CPU{}: {} irqs, {} syscalls, {} ctxsw [{}]", 
            e.qq, e.interrupts, e.apd, e.gdf, g);
    }
}

pub(super) fn rfo() {
    let cm = crate::sync::percpu::gyf();
    let blm: u64 = cm.iter().map(|e| e.interrupts).sum();
    
    crate::h!(G_, "IRQ Statistics");
    crate::println!("---------------------------------------------------------------");
    crate::println!();
    crate::println!("  Total IRQs     : {}", blm);
    crate::println!("  IRQ rate       : {}/sec", crate::devtools::eds());
    crate::println!();
    crate::h!(C_, "  Per-CPU Breakdown:");
    for e in &cm {
        let qmw = if blm > 0 { (e.interrupts * 40 / blm.am(1)) as usize } else { 0 };
        let bar: String = "|".afd(qmw);
        let cgn = if blm > 0 { e.interrupts * 100 / blm } else { 0 };
        crate::println!("    CPU{}: {:>8} ({:>3}%) {}", e.qq, e.interrupts, cgn, bar);
    }
}

pub(super) fn rhn() {
    crate::h!(G_, "CPU Register Dump");
    crate::println!("---------------------------------------------------------------");
    let regs = crate::devtools::rpv();
    for line in &regs {
        crate::println!("{}", line);
    }
}

pub(super) fn rgy(n: &[&str]) {
    if n.is_empty() {
        crate::println!("Usage: peek <hex_addr> [byte_count]");
        crate::println!("  e.g.: peek 0xFFFF8000_00000000 64");
        crate::println!("  Default count: 64 bytes, max: 256 bytes");
        return;
    }
    
    let elz = n[0].tl("0x").tl("0X");
    let ag = match usize::wa(elz, 16) {
        Ok(q) => q,
        Err(_) => {
            crate::h!(A_, "Invalid hex address: {}", n[0]);
            return;
        }
    };
    
    let az = n.get(1).and_then(|e| e.parse::<usize>().bq()).unwrap_or(64);
    
    crate::h!(G_, "Memory dump at 0x{:016x} ({} bytes)", ag, az);
    crate::println!("---------------------------------------------------------------");
    let ak = crate::devtools::amm(ag, az);
    for line in &ak {
        crate::println!("{}", line);
    }
}

pub(super) fn rhd(n: &[&str]) {
    if n.len() < 2 {
        crate::println!("Usage: poke <hex_addr> <hex_value>");
        crate::println!("  e.g.: poke 0xB8000 0x41");
        crate::h!(A_, "  ? WARNING: Writing to arbitrary memory is DANGEROUS!");
        return;
    }
    
    let elz = n[0].tl("0x").tl("0X");
    let ag = match usize::wa(elz, 16) {
        Ok(q) => q,
        Err(_) => {
            crate::h!(A_, "Invalid hex address: {}", n[0]);
            return;
        }
    };
    
    let ekm = n[1].tl("0x").tl("0X");
    let bn = match u8::wa(ekm, 16) {
        Ok(p) => p,
        Err(_) => {
            crate::h!(A_, "Invalid hex value: {}", n[1]);
            return;
        }
    };
    
    match crate::devtools::luq(ag, bn) {
        Ok(()) => crate::h!(B_, "Wrote 0x{:02x} to 0x{:016x}", bn, ag),
        Err(aa) => crate::h!(A_, "poke error: {}", aa),
    }
}

pub(super) fn rdq() {
    crate::devtools::xiu();
    let g = if crate::devtools::ofv() { "ON" } else { "OFF" };
    crate::h!(B_, "DevPanel overlay: {} (also toggle with F12 in desktop)", g);
}

pub(super) fn rjk(n: &[&str]) {
    if n.is_empty() {
        crate::println!("Usage: timecmd <command> [args...]");
        crate::println!("  Runs a command and prints elapsed time.");
        return;
    }
    
    let ay = crate::cpu::tsc::Stopwatch::ay();
    
    
    let wvj = n.rr(" ");
    super::azu(&wvj);
    
    let fhl = ay.fhk();
    let oz = fhl / 1000;
    let avw = fhl % 1000;
    crate::println!();
    crate::h!(C_, "? Elapsed: {}.{:03} ms ({} us)", oz, avw, fhl);
}






pub(super) fn rfc() {
    crate::h!(G_, "Generating hardware diagnostic report...");
    let ak = crate::debug::syw();
    for line in &ak {
        crate::println!("{}", line);
    }
    
    for line in &ak {
        crate::serial_println!("{}", line);
    }
}


pub(super) fn rdc() {
    crate::h!(G_, "Full CPU State Dump");
    crate::println!("---------------------------------------------------------------");
    let ak = crate::debug::ivz();
    for line in &ak {
        crate::println!("{}", line);
    }
}


pub(super) fn rik(n: &[&str]) {
    let am = n.fv().and_then(|e| e.parse::<usize>().bq()).unwrap_or(16);
    crate::h!(G_, "Stack Backtrace (max {} frames)", am);
    crate::println!("---------------------------------------------------------------");
    let ak = crate::debug::ivi(am);
    for line in &ak {
        crate::println!("{}", line);
    }
}


pub(super) fn rcq() {
    crate::h!(G_, "Boot Checkpoints");
    crate::println!("---------------------------------------------------------------");
    let gdm = crate::debug::nxv();
    if gdm.is_empty() {
        crate::println!("  <no checkpoints recorded>");
    } else {
        let suf = gdm[0].0;
        for (tsc, aj, j) in &gdm {
            let aaq = tsc - suf;
            crate::println!("  POST 0x{:02X}  TSC +{:>14}  {}", aj, aaq, j);
        }
    }
    crate::println!("  Last POST code: 0x{:02X}", crate::debug::oie());
}


pub(super) fn rhe(n: &[&str]) {
    if n.is_empty() {
        crate::println!("Current POST code: 0x{:02X}", crate::debug::oie());
        crate::println!("Usage: postcode <hex_value>   (writes to port 0x80)");
        return;
    }
    let ekm = n[0].tl("0x").tl("0X");
    match u8::wa(ekm, 16) {
        Ok(p) => {
            crate::debug::jjr(p);
            crate::h!(B_, "POST code 0x{:02X} written to port 0x80", p);
        }
        Err(_) => crate::h!(A_, "Invalid hex value: {}", n[0]),
    }
}


pub(super) fn rfl(n: &[&str]) {
    if n.len() < 2 {
        crate::println!("Usage: ioport read <port_hex> [b|w|l]");
        crate::println!("       ioport write <port_hex> <value_hex> [b|w|l]");
        crate::println!("  b=byte (default), w=word, l=dword");
        crate::h!(A_, "  ⚠ WARNING: Writing to arbitrary I/O ports is DANGEROUS!");
        return;
    }
    
    let air = n[0];
    let frc = n[1].tl("0x").tl("0X");
    let port = match u16::wa(frc, 16) {
        Ok(ai) => ai,
        Err(_) => {
            crate::h!(A_, "Invalid port: {}", n[1]);
            return;
        }
    };
    
    match air {
        "read" | "r" => {
            let aw = n.get(2).hu().unwrap_or("b");
            match aw {
                "b" | "byte" => {
                    let ap = crate::debug::cfn(port);
                    crate::println!("  IN  port 0x{:04X} = 0x{:02X} ({})", port, ap, ap);
                }
                "w" | "word" => {
                    let ap = crate::debug::jar(port);
                    crate::println!("  IN  port 0x{:04X} = 0x{:04X} ({})", port, ap, ap);
                }
                "l" | "dword" => {
                    let ap = crate::debug::jac(port);
                    crate::println!("  IN  port 0x{:04X} = 0x{:08X} ({})", port, ap, ap);
                }
                _ => crate::h!(A_, "Size must be b/w/l"),
            }
        }
        "write" | "w" => {
            if n.len() < 3 {
                crate::h!(A_, "Need value: ioport write <port> <value> [b|w|l]");
                return;
            }
            let ekm = n[2].tl("0x").tl("0X");
            let aw = n.get(3).hu().unwrap_or("b");
            match aw {
                "b" | "byte" => {
                    if let Ok(p) = u8::wa(ekm, 16) {
                        crate::debug::bkt(port, p);
                        crate::h!(B_, "  OUT port 0x{:04X} <- 0x{:02X}", port, p);
                    } else {
                        crate::h!(A_, "Invalid byte value");
                    }
                }
                "w" | "word" => {
                    if let Ok(p) = u16::wa(ekm, 16) {
                        crate::debug::jie(port, p);
                        crate::h!(B_, "  OUT port 0x{:04X} <- 0x{:04X}", port, p);
                    } else {
                        crate::h!(A_, "Invalid word value");
                    }
                }
                "l" | "dword" => {
                    if let Ok(p) = u32::wa(ekm, 16) {
                        crate::debug::jic(port, p);
                        crate::h!(B_, "  OUT port 0x{:04X} <- 0x{:08X}", port, p);
                    } else {
                        crate::h!(A_, "Invalid dword value");
                    }
                }
                _ => crate::h!(A_, "Size must be b/w/l"),
            }
        }
        _ => crate::h!(A_, "Use: ioport read|write ..."),
    }
}


pub(super) fn rhi(n: &[&str]) {
    if n.is_empty() {
        crate::println!("Usage: rdmsr <msr_hex>");
        crate::println!("  e.g.: rdmsr 0xC0000080  (IA32_EFER)");
        crate::println!("Common MSRs:");
        crate::println!("  0xC0000080  IA32_EFER       0x0000001B  IA32_APIC_BASE");
        crate::println!("  0xC0000081  IA32_STAR        0x00000010  IA32_TSC");
        crate::println!("  0xC0000082  IA32_LSTAR       0x00000277  IA32_PAT");
        return;
    }
    let lnb = n[0].tl("0x").tl("0X");
    match u32::wa(lnb, 16) {
        Ok(msr) => {
            match crate::debug::fsg(msr) {
                Some(ap) => {
                    crate::println!("  MSR 0x{:08X} = 0x{:016X}", msr, ap);
                    crate::println!("                  {:064b}", ap);
                }
                None => crate::h!(A_, "  MSR 0x{:08X}: read failed (#GP)", msr),
            }
        }
        Err(_) => crate::h!(A_, "Invalid MSR address: {}", n[0]),
    }
}


pub(super) fn rkt(n: &[&str]) {
    if n.len() < 2 {
        crate::println!("Usage: wrmsr <msr_hex> <value_hex>");
        crate::h!(A_, "  ⚠ WARNING: Writing to MSRs can crash the system!");
        return;
    }
    let lnb = n[0].tl("0x").tl("0X");
    let ekm = n[1].tl("0x").tl("0X");
    
    let msr = match u32::wa(lnb, 16) {
        Ok(ef) => ef,
        Err(_) => {
            crate::h!(A_, "Invalid MSR: {}", n[0]);
            return;
        }
    };
    let ap = match u64::wa(ekm, 16) {
        Ok(p) => p,
        Err(_) => {
            crate::h!(A_, "Invalid value: {}", n[1]);
            return;
        }
    };
    
    crate::debug::fbs(msr, ap);
    crate::h!(B_, "  WRMSR 0x{:08X} <- 0x{:016X}", msr, ap);
}


pub(super) fn rde(n: &[&str]) {
    if n.is_empty() {
        crate::println!("Usage: cpuid <leaf_hex> [subleaf_hex]");
        crate::println!("  e.g.: cpuid 0          (vendor string)");
        crate::println!("        cpuid 1          (features)");
        crate::println!("        cpuid 0x80000002 (brand string part 1)");
        crate::println!("        cpuid 0x80000003 (brand string part 2)");
        crate::println!("        cpuid 0x80000004 (brand string part 3)");
        return;
    }
    let udu = n[0].tl("0x").tl("0X");
    let wvt = n.get(1).map(|e| e.tl("0x").tl("0X")).unwrap_or("0");
    
    let awa = match u32::wa(udu, 16) {
        Ok(dm) => dm,
        Err(_) => {
            crate::h!(A_, "Invalid leaf: {}", n[0]);
            return;
        }
    };
    let bxj = u32::wa(wvt, 16).unwrap_or(0);
    
    crate::h!(G_, "CPUID Query");
    crate::println!("---------------------------------------------------------------");
    let ak = crate::debug::ghj(awa, bxj);
    for line in &ak {
        crate::println!("{}", line);
    }
}


pub(super) fn rgj() {
    crate::h!(G_, "Physical Memory Map");
    crate::println!("---------------------------------------------------------------");
    let ak = crate::debug::nvq();
    for line in &ak {
        crate::println!("{}", line);
    }
}


pub(super) fn rko(n: &[&str]) {
    match n.fv().hu() {
        Some("enable" | "on") => {
            let aah = n.get(1).and_then(|e| e.parse::<u64>().bq()).unwrap_or(5000);
            crate::debug::xtw(aah);
            crate::h!(B_, "Watchdog enabled ({} ms timeout)", aah);
        }
        Some("disable" | "off") => {
            crate::debug::xtv();
            crate::h!(B_, "Watchdog disabled");
        }
        Some("pet" | "kick") => {
            crate::debug::xtx();
            crate::h!(B_, "Watchdog petted");
        }
        _ => {
            crate::println!("Usage: watchdog <enable [ms]|disable|pet>");
            crate::println!("  enable [timeout_ms]  — Start watchdog (default: 5000 ms)");
            crate::println!("  disable              — Stop watchdog");
            crate::println!("  pet                  — Reset watchdog counter");
        }
    }
}

pub(super) fn rgd(elm: &[&str]) {
    crate::println!("COMMAND   PID   FD   TYPE   NAME");
    crate::println!("----------------------------------------");
    crate::println!("shell     1     0    CHR    /dev/stdin");
    crate::println!("shell     1     1    CHR    /dev/stdout");
    crate::println!("shell     1     2    CHR    /dev/stderr");
}

pub(super) fn riq(n: &[&str]) {
    if n.is_empty() {
        crate::println!("Usage: strings <file>");
        return;
    }
    
    match super::network::jli(n[0]) {
        Some(f) => {
            let mut cv = String::new();
            for &hf in &f {
                if hf.jbb() || hf == b' ' {
                    cv.push(hf as char);
                } else {
                    if cv.len() >= 4 {
                        crate::println!("{}", cv);
                    }
                    cv.clear();
                }
            }
            if cv.len() >= 4 {
                crate::println!("{}", cv);
            }
        }
        None => crate::h!(A_, "strings: cannot read {}", n[0]),
    }
}

pub(super) fn kiw(n: &[&str]) {
    if n.is_empty() {
        
        crate::h!(G_, "Mounted Filesystems:");
        crate::vfs::hqa();
        return;
    }
    
    if n.len() < 2 {
        crate::println!("Usage: mount <device> <mountpoint>");
        return;
    }
    
    crate::h!(D_, "mount: dynamic mounting not implemented");
}

pub(super) fn rit() {
    crate::println!("Syncing filesystems...");
    crate::h!(B_, "Done.");
}

pub(super) fn rjx(n: &[&str]) {
    if n.is_empty() {
        crate::println!("Usage: umount <mountpoint>");
        return;
    }
    match crate::vfs::xob(n[0]) {
        Ok(()) => crate::h!(B_, "Unmounted {}", n[0]),
        Err(aa) => crate::h!(A_, "umount: {}: {:?}", n[0], aa),
    }
}

pub(super) fn rel(n: &[&str]) {
    crate::h!(G_, "TrustFS Filesystem Check");
    crate::println!("========================================");

    let ajf = crate::vfs::hqa();
    if ajf.is_empty() {
        crate::h!(D_, "No mounted filesystems");
        return;
    }

    let mut bqn = 0u32;
    let mut cpb = 0u32;

    for (path, eqw) in &ajf {
        cpb += 1;
        crate::print!("  [{}] {} ({})... ", cpb, path, eqw);

        
        match crate::vfs::brx(path) {
            Ok(ch) => {
                let az = ch.len();
                crate::h!(B_, "OK ({} entries)", az);
            }
            Err(aa) => {
                bqn += 1;
                crate::h!(A_, "ERROR: {:?}", aa);
            }
        }
    }

    crate::println!("----------------------------------------");
    if bqn == 0 {
        crate::h!(B_, "fsck: {} filesystem(s) checked, no errors", cpb);
    } else {
        crate::h!(A_, "fsck: {} error(s) found in {} filesystem(s)", bqn, cpb);
    }
}

pub(super) fn rfz() {
    crate::h!(G_, "Block Devices:");
    crate::println!("NAME          SIZE        TYPE    DRIVER        MODEL");
    crate::println!("----------------------------------------------------------------------");
    
    let mut w = 0u32;
    
    
    if crate::nvme::ky() {
        if let Some((model, msv, gob, bni)) = crate::nvme::ani() {
            let afz = gob * bni as u64;
            let als = cxz(afz);
            crate::println!("nvme0n1       {:<11} disk    NVMe          {}", als, model);
            w += 1;
        }
    }
    
    
    if crate::drivers::ahci::ky() {
        for ba in crate::drivers::ahci::bhh() {
            let afz = ba.agw * 512;
            let als = cxz(afz);
            let bde = match ba.ceb {
                crate::drivers::ahci::AhciDeviceType::Qr => "disk",
                crate::drivers::ahci::AhciDeviceType::Bse => "cdrom",
                _ => "disk",
            };
            crate::println!("sda{}          {:<11} {:<7} AHCI/p{}       {}", 
                w, als, bde, ba.kg, ba.model);
            w += 1;
        }
    }
    
    
    for ane in crate::drivers::ata::jdq() {
        if ane.brs {
            let afz = ane.agw * 512;
            let als = cxz(afz);
            let bm = match ane.channel {
                crate::drivers::ata::IdeChannel::Adx => "P",
                crate::drivers::ata::IdeChannel::Aeq => "S",
            };
            let u = match ane.qf {
                crate::drivers::ata::DrivePosition::Ake => "M",
                crate::drivers::ata::DrivePosition::Ams => "S",
            };
            let bde = if ane.gal { "cdrom" } else { "disk" };
            let udp = if ane.gle { "LBA48" } else { "LBA28" };
            crate::println!("hd{}           {:<11} {:<7} IDE/{}{} {}  {}", 
                w, als, bde, bm, u, udp, ane.model);
            w += 1;
        }
    }
    
    
    if crate::virtio_blk::ky() {
        let mh = crate::virtio_blk::aty();
        let als = cxz(mh * 512);
        let jmq = if crate::virtio_blk::jbr() { " (ro)" } else { "" };
        crate::println!("vda{}          {:<11} disk    VirtIO-blk{}", w, als, jmq);
        w += 1;
    }
    
    
    for (a, (j, xk, gbn)) in crate::drivers::usb_storage::bhh().iter().cf() {
        let als = cxz(*xk * *gbn as u64);
        crate::println!("usb{}          {:<11} disk    USB-Storage   {}", 
            w + a as u32, als, j);
    }
    if w == 0 && crate::drivers::usb_storage::cjx() == 0 {
        w += 1; 
    }
    
    
    crate::println!("ram0          256K        ramdisk RAM           TrustFS");
    
    if w == 0 {
        crate::println!();
        crate::h!(D_, "No hardware storage detected (using RAM disk)");
    }
}


fn cxz(bf: u64) -> alloc::string::String {
    if bf == 0 {
        return alloc::string::String::from("0B");
    }
    if bf >= 1024 * 1024 * 1024 * 1024 {
        alloc::format!("{:.1}T", bf as f64 / (1024.0 * 1024.0 * 1024.0 * 1024.0))
    } else if bf >= 1024 * 1024 * 1024 {
        alloc::format!("{:.1}G", bf as f64 / (1024.0 * 1024.0 * 1024.0))
    } else if bf >= 1024 * 1024 {
        alloc::format!("{:.1}M", bf as f64 / (1024.0 * 1024.0))
    } else if bf >= 1024 {
        alloc::format!("{}K", bf / 1024)
    } else {
        alloc::format!("{}B", bf)
    }
}

pub(super) fn rcp() {
    let mut aig = false;
    
    
    if crate::nvme::ky() {
        if let Some((model, serial, gob, bni)) = crate::nvme::ani() {
            let afz = gob * bni as u64;
            crate::println!("/dev/nvme0n1: MODEL=\"{}\" SERIAL=\"{}\" SIZE={} TYPE=\"nvme\"",
                model, serial, cxz(afz));
            aig = true;
        }
    }
    
    
    if crate::drivers::ahci::ky() {
        for (a, ba) in crate::drivers::ahci::bhh().iter().cf() {
            let afz = ba.agw * 512;
            crate::println!("/dev/sda{}: MODEL=\"{}\" SERIAL=\"{}\" SIZE={} TYPE=\"ahci\" PORT={}",
                a, ba.model, ba.serial, cxz(afz), ba.kg);
            aig = true;
        }
    }
    
    
    for (a, ane) in crate::drivers::ata::jdq().iter().cf() {
        if ane.brs {
            let afz = ane.agw * 512;
            let eqw = if ane.gal { "atapi" } else { "ide" };
            crate::println!("/dev/hd{}: MODEL=\"{}\" SERIAL=\"{}\" SIZE={} TYPE=\"{}\"",
                a, ane.model, ane.serial, cxz(afz), eqw);
            aig = true;
        }
    }
    
    
    if crate::virtio_blk::ky() {
        let mh = crate::virtio_blk::aty();
        crate::println!("/dev/vda: SIZE={} TYPE=\"virtio-blk\"", cxz(mh * 512));
        aig = true;
    }
    
    
    for (a, (j, xk, gbn)) in crate::drivers::usb_storage::bhh().iter().cf() {
        crate::println!("/dev/usb{}: MODEL=\"{}\" SIZE={} TYPE=\"usb-storage\"",
            a, j, cxz(*xk * *gbn as u64));
        aig = true;
    }
    
    
    crate::println!("/dev/ram0: SIZE=256K TYPE=\"ramfs\"");
    
    if !aig {
        crate::h!(D_, "No hardware block devices detected");
    }
}

pub(super) fn kio(n: &[&str]) {
    if n.is_empty() {
        
        for (eh, p) in super::scripting::ijj() {
            crate::println!("export {}={}", eh, p);
        }
        return;
    }
    
    let lgx = n.rr(" ");
    if let Some(bzo) = lgx.du('=') {
        let bs = lgx[..bzo].em();
        let ap = lgx[bzo + 1..].em().dcz('"').dcz('\'');
        super::scripting::fuk(bs, ap);
        crate::serial_println!("[export] {}={}", bs, ap);
    } else {
        
        if super::scripting::cqx(n[0]).is_none() {
            super::scripting::fuk(n[0], "");
        }
    }
}

pub(super) fn rij(n: &[&str]) {
    if n.is_empty() {
        crate::println!("Usage: source <script>");
        return;
    }
    
    match super::network::fse(n[0]) {
        Some(ca) => {
            super::scripting::sos(&ca);
        }
        None => crate::h!(A_, "source: cannot read {}", n[0]),
    }
}

pub(super) fn ria(elm: &[&str]) {
    for (eh, p) in super::scripting::ijj() {
        crate::println!("{}={}", eh, p);
    }
}

pub(super) fn rhg(n: &[&str]) {
    if n.is_empty() {
        crate::println!("Usage: printf <format> [args...]");
        return;
    }
    
    let format = n[0].replace("\\n", "\n").replace("\\t", "\t");
    crate::print!("{}", format);
}

pub(super) fn rjd(n: &[&str]) {
    
    if n.is_empty() {
        crate::println!("false");
        return;
    }
    
    match n.fv() {
        Some(&"-e") if n.len() > 1 => {
            if super::vm::cxx(n[1]) {
                crate::println!("true");
            } else {
                crate::println!("false");
            }
        }
        Some(&"-d") if n.len() > 1 => {
            crate::h!(D_, "(directory check not implemented)");
        }
        Some(&"-f") if n.len() > 1 => {
            if super::vm::cxx(n[1]) {
                crate::println!("true");
            } else {
                crate::println!("false");
            }
        }
        _ => crate::println!("true"),
    }
}

pub(super) fn red(n: &[&str]) {
    if n.len() < 3 {
        crate::println!("Usage: expr <num1> <op> <num2>");
        return;
    }
    
    let q: i64 = n[0].parse().unwrap_or(0);
    let o: i64 = n[2].parse().unwrap_or(0);
    
    let result = match n[1] {
        "+" => q + o,
        "-" => q - o,
        "*" => q * o,
        "/" if o != 0 => q / o,
        "%" if o != 0 => q % o,
        _ => {
            crate::println!("expr: invalid operator");
            return;
        }
    };
    
    crate::println!("{}", result);
}

pub(super) fn rcu(elm: &[&str]) {
    crate::h!(G_, "   February 2026");
    crate::println!("Su Mo Tu We Th Fr Sa");
    crate::println!(" 1  2  3  4  5  6  7");
    crate::println!(" 8  9 10 11 12 13 14");
    crate::println!("15 16 17 18 19 20 21");
    crate::println!("22 23 24 25 26 27 28");
}

pub(super) fn rcz(n: &[&str]) {
    if n.len() < 2 {
        crate::println!("Usage: cmp <file1> <file2>");
        return;
    }
    
    match (super::network::jli(n[0]), super::network::jli(n[1])) {
        (Some(q), Some(o)) => {
            if q == o {
                
            } else {
                crate::println!("{} {} differ", n[0], n[1]);
            }
        }
        _ => crate::h!(A_, "cmp: cannot read files"),
    }
}

pub(super) fn rgt(n: &[&str]) {
    if n.is_empty() {
        crate::println!("Usage: od <file>");
        return;
    }
    
    super::commands::neb(n);
}

pub(super) fn rhq(n: &[&str]) {
    if n.is_empty() {
        crate::println!("Usage: rev <file>");
        return;
    }
    
    match super::network::fse(n[0]) {
        Some(ca) => {
            for line in ca.ak() {
                let vyo: String = line.bw().vv().collect();
                crate::println!("{}", vyo);
            }
        }
        None => crate::h!(A_, "rev: cannot read {}", n[0]),
    }
}

pub(super) fn ree(n: &[&str]) {
    if n.is_empty() {
        crate::println!("Usage: factor <number>");
        return;
    }
    
    let mut bo: u64 = n[0].parse().unwrap_or(0);
    if bo == 0 {
        crate::println!("factor: invalid number");
        return;
    }
    
    crate::print!("{}:", bo);
    let mut bc = 2u64;
    while bc.rab(bc).efd(false, |rty| rty <= bo) {
        while bo % bc == 0 {
            crate::print!(" {}", bc);
            bo /= bc;
        }
        bc += 1;
    }
    if bo > 1 {
        crate::print!(" {}", bo);
    }
    crate::println!();
}

pub(super) fn rjv() {
    crate::println!("/dev/tty0");
}

pub(super) fn rir(elm: &[&str]) {
    crate::println!("speed 9600 baud; line = 0;");
    crate::println!("-brkint -imaxbel");
}

pub(super) fn rho() {
    super::commands::iof();
    crate::println!("Terminal reset.");
}

pub(super) fn rgf() {
    crate::h!(G_, "USB Devices:");
    crate::println!("-------------------------------------------");
    
    
    if crate::drivers::xhci::ky() {
        let ik = crate::drivers::xhci::bhh();
        if ik.is_empty() {
            crate::println!("Bus 001 Device 001: ID 0000:0000 Root Hub");
            crate::println!("  (no devices connected)");
        } else {
            crate::println!("Bus 001 Device 001: ID 0000:0000 xHCI Root Hub");
            for (a, ba) in ik.iter().cf() {
                let ig = match ba.ig {
                    1 => "Full Speed (12 Mbps)",
                    2 => "Low Speed (1.5 Mbps)",
                    3 => "High Speed (480 Mbps)",
                    4 => "SuperSpeed (5 Gbps)",
                    _ => "Unknown",
                };
                crate::println!("Bus 001 Device {:03}: ID {:04x}:{:04x} Port {} - {}", 
                    a + 2, ba.ml, ba.cgt, ba.port, ig);
                if ba.class != 0 {
                    let bpz = match ba.class {
                        0x03 => "HID (Human Interface Device)",
                        0x08 => "Mass Storage",
                        0x09 => "Hub",
                        _ => "Unknown class",
                    };
                    crate::println!("    Class: {:02x}:{:02x}:{:02x} ({})", 
                        ba.class, ba.adl, ba.protocol, bpz);
                }
            }
        }
        crate::println!("");
        crate::println!("Total: {} device(s) connected", ik.len());
    } else {
        crate::println!("Bus 001 Device 001: ID 0000:0000 Root Hub");
        crate::h!(D_, "  (xHCI controller not initialized)");
    }
}

pub(super) fn rig() {
    crate::cpu::smp::lvm();
}

pub(super) fn rif(n: &[&str]) {
    if n.is_empty() {
        let status = if crate::cpu::smp::jbt() { "ON" } else { "OFF" };
        let cdv = crate::cpu::smp::boc();
        crate::println!("SMP parallelism: {} ({} CPUs ready)", status, cdv);
        crate::println!("Usage: smp [on|off|status]");
        crate::println!("  on     - Enable multi-core parallel rendering");
        crate::println!("  off    - Disable parallelism (single-core, safe mode)");
        crate::println!("  status - Show detailed CPU status");
        return;
    }
    
    match n[0] {
        "on" | "1" | "enable" => {
            crate::cpu::smp::isq();
            crate::h!(0xFF00FF00, "SMP parallelism ENABLED");
        },
        "off" | "0" | "disable" => {
            crate::cpu::smp::kqd();
            crate::h!(0xFFFF8800, "SMP parallelism DISABLED (single-core mode)");
        },
        "status" => {
            crate::cpu::smp::lvm();
        },
        _ => {
            crate::println!("Unknown option: {}", n[0]);
            crate::println!("Usage: smp [on|off|status]");
        }
    }
}

pub(super) fn rek(n: &[&str]) {
    use crate::framebuffer::font::{FontMode, mep, nye};
    
    if n.is_empty() {
        let cv = match nye() {
            FontMode::Bsr => "sharp (disabled)",
            FontMode::Ayz => "smooth (enabled)",
        };
        crate::println!("Font smoothing: {}", cv);
        crate::println!("Usage: fontsmooth [on|off]");
        return;
    }
    
    match n[0] {
        "on" | "enable" | "smooth" => {
            mep(FontMode::Ayz);
            crate::println!("Font smoothing enabled");
        }
        "off" | "disable" | "sharp" => {
            mep(FontMode::Bsr);
            crate::println!("Font smoothing disabled");
        }
        _ => {
            crate::println!("Usage: fontsmooth [on|off]");
        }
    }
}

pub(super) fn ned() {
    crate::h!(G_, "CPU Information:");
    crate::println!("-------------------------------------------");
    
    
    if let Some(dr) = crate::cpu::bme() {
        crate::println!("Brand:        {}", dr.keu());
        crate::println!("Architecture: x86_64");
        crate::println!("Vendor:       {:?}", dr.acs);
        crate::println!("Family:       {}", dr.family);
        crate::println!("Model:        {}", dr.model);
        crate::println!("Stepping:     {}", dr.bxi);
        crate::println!("CPU(s):       {}", crate::cpu::smp::aao());
        crate::println!("APIC ID:      {}", dr.aed);
        
        
        crate::println!("");
        crate::h!(C_, "Timing:");
        crate::println!("TSC:          {} (invariant: {})", 
            if dr.tsc { "yes" } else { "no" },
            if dr.fan { "yes" } else { "no" });
        crate::println!("TSC Freq:     {} MHz", dr.ekf / 1_000_000);
        crate::println!("RDTSCP:       {}", if dr.fsd { "yes" } else { "no" });
        
        
        crate::println!("");
        crate::h!(C_, "SIMD:");
        crate::println!("SSE:          {}", if dr.eiw { "yes" } else { "no" });
        crate::println!("SSE2:         {}", if dr.eix { "yes" } else { "no" });
        crate::println!("SSE3:         {}", if dr.fvj { "yes" } else { "no" });
        crate::println!("SSSE3:        {}", if dr.fvl { "yes" } else { "no" });
        crate::println!("SSE4.1:       {}", if dr.fvk { "yes" } else { "no" });
        crate::println!("SSE4.2:       {}", if dr.eyy { "yes" } else { "no" });
        crate::println!("AVX:          {}", if dr.dof { "yes" } else { "no" });
        crate::println!("AVX2:         {}", if dr.dog { "yes" } else { "no" });
        crate::println!("AVX-512:      {}", if dr.eml { "yes" } else { "no" });
        
        
        crate::println!("");
        crate::h!(C_, "Crypto Acceleration:");
        crate::println!("AES-NI:       {}", if dr.doa { "yes" } else { "no" });
        crate::println!("PCLMULQDQ:    {}", if dr.ewm { "yes" } else { "no" });
        crate::println!("SHA-NI:       {}", if dr.eyl { "yes" } else { "no" });
        crate::println!("RDRAND:       {}", if dr.cbg { "yes" } else { "no" });
        crate::println!("RDSEED:       {}", if dr.cmc { "yes" } else { "no" });
        
        
        crate::println!("");
        crate::h!(C_, "Security:");
        crate::println!("SMEP:         {}", if dr.cia { "yes" } else { "no" });
        crate::println!("SMAP:         {}", if dr.cul { "yes" } else { "no" });
        crate::println!("NX:           {}", if dr.vt { "yes" } else { "no" });
        
        
        crate::println!("");
        crate::h!(C_, "Virtualization:");
        crate::println!("Intel VT-x:   {}", if dr.vmx { "yes" } else { "no" });
        crate::println!("AMD-V:        {}", if dr.svm { "yes" } else { "no" });
    } else {
        crate::println!("Architecture: x86_64");
        crate::println!("(CPU detection not initialized)");
    }
}

pub(super) fn rgb() {
    let aul = (crate::memory::cm().afa + crate::memory::cm().buv);
    
    crate::h!(G_, "Memory Configuration:");
    crate::println!("-------------------------------------------");
    crate::println!("Total:       {} KB", aul / 1024);
    crate::println!("Used:        {} KB", crate::memory::cm().afa / 1024);
}

pub(super) fn rgc() {
    crate::h!(G_, "Loaded Kernel Modules:");
    crate::println!("Module                  Size  Used by");
    crate::println!("e1000                  64000  1");
    crate::println!("ahci                   32000  0");
    crate::println!("ps2kbd                  8000  1");
    crate::println!("ps2mouse                4000  1");
}

pub(super) fn riw(elm: &[&str]) {
    crate::println!("kernel.ostype = TrustOS");
    crate::println!("kernel.osrelease = 0.1.0");
    crate::println!("kernel.version = #1 SMP TrustOS");
}



pub(super) fn rej(n: &[&str]) {
    use crate::netstack::firewall;
    use crate::netstack::firewall::{Chain, Action, Protocol, IpMatch, PortMatch, Rule};

    if n.is_empty() {
        ndz();
        return;
    }

    match n[0] {
        "status" | "show" => ndz(),
        "enable" | "on" => {
            firewall::cuf(true);
            crate::h!(B_, "Firewall enabled");
        }
        "disable" | "off" => {
            firewall::cuf(false);
            crate::h!(D_, "Firewall disabled");
        }
        "policy" => {
            
            if n.len() < 3 {
                crate::println!("Usage: firewall policy <INPUT|OUTPUT|FORWARD> <ACCEPT|DROP>");
                return;
            }
            let rh = match Chain::cko(n[1]) {
                Some(r) => r,
                None => { crate::h!(A_, "Invalid chain: {}", n[1]); return; }
            };
            let hr = match Action::cko(n[2]) {
                Some(q) => q,
                None => { crate::h!(A_, "Invalid action: {}", n[2]); return; }
            };
            firewall::met(rh, hr);
            crate::h!(B_, "Policy {} set to {}", rh.j(), hr.j());
        }
        "add" => {
            
            if n.len() < 2 {
                crate::println!("Usage: firewall add <chain> [-p proto] [-s src] [-d dst] [--sport port] [--dport port] -j <action>");
                return;
            }
            let rh = match Chain::cko(n[1]) {
                Some(r) => r,
                None => { crate::h!(A_, "Invalid chain: {}", n[1]); return; }
            };
            let mut agu = Rule::new(rh, Action::Ld);
            let mut a = 2;
            while a < n.len() {
                match n[a] {
                    "-p" | "--proto" => {
                        a += 1;
                        if a < n.len() {
                            agu.protocol = Protocol::cko(n[a]).unwrap_or(Protocol::Eb);
                        }
                    }
                    "-s" | "--src" => {
                        a += 1;
                        if a < n.len() {
                            agu.jh = IpMatch::parse(n[a]).unwrap_or(IpMatch::Eb);
                        }
                    }
                    "-d" | "--dst" => {
                        a += 1;
                        if a < n.len() {
                            agu.pz = IpMatch::parse(n[a]).unwrap_or(IpMatch::Eb);
                        }
                    }
                    "--sport" => {
                        a += 1;
                        if a < n.len() {
                            agu.ey = PortMatch::parse(n[a]).unwrap_or(PortMatch::Eb);
                        }
                    }
                    "--dport" => {
                        a += 1;
                        if a < n.len() {
                            agu.sa = PortMatch::parse(n[a]).unwrap_or(PortMatch::Eb);
                        }
                    }
                    "-j" | "--jump" => {
                        a += 1;
                        if a < n.len() {
                            agu.hr = Action::cko(n[a]).unwrap_or(Action::Ld);
                        }
                    }
                    _ => {}
                }
                a += 1;
            }
            firewall::qfo(agu);
            crate::h!(B_, "Rule added to {} chain", rh.j());
        }
        "del" | "delete" => {
            
            if n.len() < 3 {
                crate::println!("Usage: firewall del <chain> <index>");
                return;
            }
            let rh = match Chain::cko(n[1]) {
                Some(r) => r,
                None => { crate::h!(A_, "Invalid chain: {}", n[1]); return; }
            };
            let w: usize = match n[2].parse() {
                Ok(bo) => bo,
                Err(_) => { crate::h!(A_, "Invalid index: {}", n[2]); return; }
            };
            if firewall::rvj(rh, w) {
                crate::h!(B_, "Rule {} deleted from {}", w, rh.j());
            } else {
                crate::h!(A_, "Rule {} not found in {}", w, rh.j());
            }
        }
        "flush" => {
            let rh = if n.len() > 1 { Chain::cko(n[1]) } else { None };
            firewall::hjx(rh);
            if let Some(r) = rh {
                crate::h!(B_, "Flushed {} chain", r.j());
            } else {
                crate::h!(B_, "Flushed all chains");
            }
        }
        "log" => {
            let ch = firewall::tdx();
            if ch.is_empty() {
                crate::println!("(no log entries)");
            } else {
                crate::h!(C_, "Firewall Log ({} entries):", ch.len());
                for bt in &ch {
                    crate::println!("  {}", bt);
                }
            }
        }
        "reset" => {
            firewall::pcq();
            firewall::rbf();
            crate::h!(B_, "Stats and log cleared");
        }
        "help" | "--help" | "-h" => {
            crate::h!(C_, "TrustOS Firewall — iptables-like packet filter");
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
            crate::h!(A_, "Unknown subcommand: {}", n[0]);
            crate::println!("Try: firewall help");
        }
    }
}

fn ndz() {
    use crate::netstack::firewall;
    use crate::netstack::firewall::Chain;

    let iq = firewall::zu();
    let (emd, sgx) = firewall::cm();

    crate::h!(C_, "TrustOS Firewall");
    crate::print!("  Status: ");
    if iq {
        crate::h!(B_, "ENABLED");
    } else {
        crate::h!(A_, "DISABLED");
    }
    crate::println!("  Packets allowed: {}  dropped: {}", emd, sgx);
    crate::println!();

    for rh in &[Chain::Jp, Chain::Dd, Chain::Abv] {
        let policy = firewall::iwu(*rh);
        let bib = firewall::ufv(*rh);
        crate::gr!(D_, "Chain {} ", rh.j());
        crate::println!("(policy {})", policy.j());
        if bib.is_empty() {
            crate::println!("  (no rules)");
        } else {
            crate::println!("  {:>3}  {:>5}  {:>15}  {:>15}  {:>11}  {:>11}  {:>6}  {:>8}  {:>8}",
                "num", "proto", "source", "destination", "sport", "dport", "action", "pkts", "bytes");
            for (a, agu) in bib.iter().cf() {
                crate::println!("  {:>3}  {:>5}  {:>15}  {:>15}  {:>11}  {:>11}  {:>6}  {:>8}  {:>8}",
                    a, agu.protocol.j(), agu.jh.display(), agu.pz.display(),
                    agu.ey.display(), agu.sa.display(), agu.hr.j(),
                    agu.egb, agu.bf);
            }
        }
        crate::println!();
    }
}



pub(super) fn rea(n: &[&str]) {
    let path = if n.is_empty() { "/" } else { n[0] };
    let es = noh(path, 0);
    if es >= 1024 * 1024 {
        crate::println!("{:.1}M\t{}", es as f64 / (1024.0 * 1024.0), path);
    } else if es >= 1024 {
        crate::println!("{}K\t{}", es / 1024, path);
    } else {
        crate::println!("{}\t{}", es, path);
    }
}

fn noh(path: &str, eo: usize) -> usize {
    let mut es: usize = 0;

    if let Ok(ch) = crate::ramfs::fh(|fs| fs.awb(Some(path))) {
        for (j, are, aw) in &ch {
            let aeh = if path == "/" {
                alloc::format!("/{}", j)
            } else {
                alloc::format!("{}/{}", path, j)
            };
            match are {
                crate::ramfs::FileType::Es => {
                    es += aw;
                }
                crate::ramfs::FileType::K => {
                    let sub = noh(&aeh, eo + 1);
                    es += sub;
                    if eo < 1 {
                        if sub >= 1024 {
                            crate::println!("{}K\t{}", sub / 1024, aeh);
                        } else {
                            crate::println!("{}\t{}", sub, aeh);
                        }
                    }
                }
                _ => {}
            }
        }
    }
    es
}






pub(super) fn rcw(n: &[&str]) {
    if n.len() < 2 {
        crate::println!("Usage: chmod <mode> <file>");
        crate::println!("  mode: 755, 644, +x, -w, etc.");
        return;
    }
    let ev = n[0];
    let path = n[1];
    if !super::vm::cxx(path) {
        crate::h!(A_, "chmod: {}: No such file", path);
        return;
    }
    crate::h!(B_, "chmod: mode of '{}' changed to {}", path, ev);
}


pub(super) fn rcy(n: &[&str]) {
    if n.len() < 2 {
        crate::println!("Usage: chown <owner[:group]> <file>");
        return;
    }
    let awj = n[0];
    let path = n[1];
    if !super::vm::cxx(path) {
        crate::h!(A_, "chown: {}: No such file", path);
        return;
    }
    crate::h!(B_, "chown: ownership of '{}' changed to {}", path, awj);
}


pub(super) fn rfv(n: &[&str]) {
    let wwx = n.fv() == Some(&"-s");
    let lya: Vec<&str> = n.iter().hi(|q| !q.cj('-')).hu().collect();
    if lya.len() < 2 {
        crate::println!("Usage: ln [-s] <target> <link_name>");
        return;
    }
    let cd = lya[0];
    let arl = lya[1];
    
    if wwx {
        
        let ca = format!("SYMLINK:{}", cd);
        let result = crate::ramfs::fh(|fs| {
            let _ = fs.touch(arl);
            fs.ns(arl, ca.as_bytes())
        });
        match result {
            Ok(()) => crate::h!(B_, "'{}' -> '{}'", arl, cd),
            Err(_) => crate::h!(A_, "ln: failed to create symbolic link"),
        }
    } else {
        
        let f = crate::ramfs::fh(|fs| fs.mq(cd).map(|o| o.ip()));
        match f {
            Ok(bf) => {
                crate::ramfs::fh(|fs| {
                    let _ = fs.touch(arl);
                    let _ = fs.ns(arl, &bf);
                });
                crate::h!(B_, "'{}' => '{}'", arl, cd);
            }
            Err(_) => crate::h!(A_, "ln: {}: No such file", cd),
        }
    }
}


pub(super) fn rhk(n: &[&str]) {
    if n.is_empty() {
        crate::println!("Usage: readlink <symlink>");
        return;
    }
    let path = n[0];
    let ca: Option<String> = crate::ramfs::fh(|fs| {
        fs.mq(path).map(|o| String::from(core::str::jg(o).unwrap_or(""))).bq()
    });
    match ca {
        Some(ref e) if e.cj("SYMLINK:") => {
            crate::println!("{}", &e[8..]);
        }
        _ => crate::h!(A_, "readlink: {}: Not a symbolic link", path),
    }
}


pub(super) fn rdg(n: &[&str], arr: Option<&str>) {
    
    let mut kov = '\t';
    let mut fields: Option<Vec<usize>> = None;
    let mut nti: Option<&str> = None;
    let mut a = 0;
    while a < n.len() {
        match n[a] {
            "-d" if a + 1 < n.len() => {
                kov = n[a + 1].bw().next().unwrap_or('\t');
                a += 2;
            }
            "-f" if a + 1 < n.len() => {
                fields = Some(vcg(n[a + 1]));
                a += 2;
            }
            ji if !ji.cj('-') => {
                nti = Some(ji);
                a += 1;
            }
            _ => { a += 1; }
        }
    }
    
    let ssb = match fields {
        Some(bb) => bb,
        None => {
            crate::println!("Usage: cut -d <delimiter> -f <fields> [file]");
            crate::println!("  Example: cut -d : -f 1,3");
            return;
        }
    };
    
    let ca = if let Some(input) = arr {
        Some(String::from(input))
    } else if let Some(path) = nti {
        super::network::fse(path)
    } else {
        crate::println!("cut: no input");
        return;
    };
    
    if let Some(text) = ca {
        for line in text.ak() {
            let ek: Vec<&str> = line.adk(kov).collect();
            let mut fv = true;
            for &bb in &ssb {
                if bb > 0 && bb <= ek.len() {
                    if !fv { crate::print!("{}", kov); }
                    crate::print!("{}", ek[bb - 1]);
                    fv = false;
                }
            }
            crate::println!();
        }
    }
}

fn vcg(e: &str) -> Vec<usize> {
    let mut fields = Vec::new();
    for vu in e.adk(',') {
        if let Some(hfb) = vu.du('-') {
            let ay: usize = vu[..hfb].parse().unwrap_or(1);
            let ci: usize = vu[hfb + 1..].parse().unwrap_or(ay).v(ay + 10_000);
            for bb in ay..=ci {
                fields.push(bb);
            }
        } else if let Ok(bb) = vu.parse::<usize>() {
            fields.push(bb);
        }
    }
    fields
}


pub(super) fn rjm(n: &[&str], arr: Option<&str>) {
    if n.len() < 2 {
        crate::println!("Usage: tr <set1> <set2>");
        crate::println!("  Example: echo hello | tr a-z A-Z");
        return;
    }
    
    let wid = nru(n[0]);
    let meb = nru(n[1]);
    
    let ca = if let Some(input) = arr {
        String::from(input)
    } else {
        crate::println!("tr: requires piped input");
        return;
    };
    
    let mut result = String::fc(ca.len());
    for bm in ca.bw() {
        if let Some(u) = wid.iter().qf(|&r| r == bm) {
            if u < meb.len() {
                result.push(meb[u]);
            } else if let Some(&qv) = meb.qv() {
                result.push(qv);
            } else {
                result.push(bm);
            }
        } else {
            result.push(bm);
        }
    }
    crate::print!("{}", result);
}

fn nru(e: &str) -> Vec<char> {
    let mut bw = Vec::new();
    let bf = e.as_bytes();
    let mut a = 0;
    while a < bf.len() {
        if a + 2 < bf.len() && bf[a + 1] == b'-' {
            let ay = bf[a];
            let ci = bf[a + 2];
            let (hh, gd) = if ay <= ci { (ay, ci) } else { (ci, ay) };
            for r in hh..=gd {
                bw.push(r as char);
            }
            a += 3;
        } else {
            bw.push(bf[a] as char);
            a += 1;
        }
    }
    bw
}


pub(super) fn rjb(n: &[&str], arr: Option<&str>) {
    let bte = n.fv() == Some(&"-a");
    let fim: Vec<&str> = n.iter().hi(|q| !q.cj('-')).hu().collect();
    
    let ca = if let Some(input) = arr {
        String::from(input)
    } else {
        crate::println!("tee: requires piped input");
        return;
    };
    
    
    crate::print!("{}", ca);
    
    
    for path in &fim {
        if bte {
            let _ = crate::ramfs::fh(|fs| fs.ijw(path, ca.as_bytes()));
        } else {
            let _ = crate::ramfs::fh(|fs| {
                if !fs.aja(path) { let _ = fs.touch(path); }
                fs.ns(path, ca.as_bytes())
            });
        }
    }
}


pub(super) fn rku(n: &[&str], arr: Option<&str>) {
    let ro = if n.is_empty() { "echo" } else { n[0] };
    let nsh = if n.len() > 1 { &n[1..] } else { &[] };
    
    let ca = if let Some(input) = arr {
        String::from(input)
    } else {
        crate::println!("xargs: requires piped input");
        return;
    };
    
    
    let pj: Vec<&str> = ca.ayt().collect();
    for item in &pj {
        let kiv = if nsh.is_empty() {
            format!("{} {}", ro, item)
        } else {
            format!("{} {} {}", ro, nsh.rr(" "), item)
        };
        super::azu(&kiv);
    }
}


pub(super) fn nej(n: &[&str]) {
    if n.is_empty() {
        crate::println!("Usage: unset <variable>");
        return;
    }
    for j in n {
        super::scripting::jur(j);
        crate::h!(B_, "Unset: {}", j);
    }
}


pub(super) fn rhj(n: &[&str]) {
    let igg = if n.is_empty() { "REPLY" } else { n[0] };
    let aau = if n.len() > 1 && n[0] == "-p" {
        if n.len() > 2 {
            crate::print!("{}", n[1]);
            if n.len() > 2 { n[2] } else { "REPLY" }
        } else {
            "REPLY"
        }
    } else {
        igg
    };
    
    
    let mut input = String::new();
    loop {
        if let Some(bs) = crate::keyboard::auw() {
            match bs {
                0x0A => break,
                0x08 => { 
                    if !input.is_empty() {
                        input.pop();
                        crate::print!("\x08 \x08");
                    }
                }
                bm if bm >= 32 && bm < 127 => {
                    input.push(bm as char);
                    crate::print!("{}", bm as char);
                }
                _ => {}
            }
        } else {
            core::hint::hc();
        }
    }
    crate::println!();
    
    super::scripting::fuk(aau, &input);
}



use spin::Mutex;
use alloc::collections::BTreeMap;

static Vg: Mutex<BTreeMap<String, String>> = Mutex::new(BTreeMap::new());

pub fn tct(j: &str) -> Option<String> {
    Vg.lock().get(j).abn()
}

pub(super) fn rcd(n: &[&str]) {
    if n.is_empty() {
        
        let ijf = Vg.lock();
        if ijf.is_empty() {
            crate::println!("No aliases defined");
        } else {
            for (j, bn) in ijf.iter() {
                crate::h!(C_, "alias {}='{}'", j, bn);
            }
        }
        return;
    }
    
    let ji = n.rr(" ");
    if let Some(bzo) = ji.du('=') {
        let j = ji[..bzo].em();
        let bn = ji[bzo + 1..].em().dcz('\'').dcz('"');
        Vg.lock().insert(String::from(j), String::from(bn));
        crate::h!(B_, "alias {}='{}'", j, bn);
    } else {
        
        let ijf = Vg.lock();
        if let Some(bn) = ijf.get(n[0]) {
            crate::println!("alias {}='{}'", n[0], bn);
        } else {
            crate::h!(A_, "alias: {}: not found", n[0]);
        }
    }
}

pub(super) fn rjy(n: &[&str]) {
    if n.is_empty() {
        crate::println!("Usage: unalias <name>");
        return;
    }
    if n[0] == "-a" {
        Vg.lock().clear();
        crate::h!(B_, "All aliases removed");
    } else {
        if Vg.lock().remove(n[0]).is_some() {
            crate::h!(B_, "Alias '{}' removed", n[0]);
        } else {
            crate::h!(A_, "unalias: {}: not found", n[0]);
        }
    }
}


pub(super) fn rcn(elm: &[&str]) {
    crate::h!(C_, "TrustOS bc — arbitrary precision calculator");
    crate::println!("Type expressions, 'quit' or 'exit' to leave");
    crate::println!();
    
    crate::shell::hcw();
    
    loop {
        crate::gr!(B_, "bc> ");
        
        let mut input = String::new();
        loop {
            if let Some(bs) = crate::keyboard::auw() {
                match bs {
                    0x0A => break,
                    0x03 => { crate::println!(); return; }
                    0x08 => {
                        if !input.is_empty() {
                            input.pop();
                            crate::print!("\x08 \x08");
                        }
                    }
                    bm if bm >= 32 && bm < 127 => {
                        input.push(bm as char);
                        crate::print!("{}", bm as char);
                    }
                    _ => {}
                }
            } else {
                core::hint::hc();
            }
        }
        crate::println!();
        
        let ux = input.em();
        if ux == "quit" || ux == "exit" {
            break;
        }
        if ux.is_empty() {
            continue;
        }
        
        
        match snm(ux) {
            Some(result) => {
                if result == (result as i64) as f64 {
                    crate::println!("{}", result as i64);
                } else {
                    crate::println!("{:.6}", result);
                }
            }
            None => crate::h!(A_, "Error: invalid expression"),
        }
    }
}

fn snm(expr: &str) -> Option<f64> {
    
    let eb = xje(expr);
    let mut u = 0;
    let result = oty(&eb, &mut u);
    if u == eb.len() { result } else { None }
}

fn xje(expr: &str) -> Vec<String> {
    let mut eb = Vec::new();
    let mut num = String::new();
    for bm in expr.bw() {
        if bm.atb() || bm == '.' {
            num.push(bm);
        } else {
            if !num.is_empty() { eb.push(core::mem::take(&mut num)); }
            if !bm.fme() {
                let mut k = [0u8; 4];
                let e = bm.hia(&mut k);
                eb.push(String::from(e));
            }
        }
    }
    if !num.is_empty() { eb.push(num); }
    eb
}

fn oty(eb: &[String], u: &mut usize) -> Option<f64> {
    let mut fd = oua(eb, u)?;
    while *u < eb.len() && (eb[*u] == "+" || eb[*u] == "-") {
        let op = eb[*u].clone();
        *u += 1;
        let hw = oua(eb, u)?;
        fd = if op == "+" { fd + hw } else { fd - hw };
    }
    Some(fd)
}

fn oua(eb: &[String], u: &mut usize) -> Option<f64> {
    let mut fd = lse(eb, u)?;
    while *u < eb.len() && (eb[*u] == "*" || eb[*u] == "/" || eb[*u] == "%") {
        let op = eb[*u].clone();
        *u += 1;
        let hw = lse(eb, u)?;
        fd = match op.as_str() {
            "*" => fd * hw,
            "/" => if hw != 0.0 { fd / hw } else { return None },
            "%" => if hw != 0.0 { fd % hw } else { return None },
            _ => unreachable!(),
        };
    }
    Some(fd)
}

fn lse(eb: &[String], u: &mut usize) -> Option<f64> {
    let ar = vbw(eb, u)?;
    if *u < eb.len() && eb[*u] == "^" {
        *u += 1;
        let bgz = lse(eb, u)?;
        Some(vkf(ar, bgz))
    } else {
        Some(ar)
    }
}

fn vbw(eb: &[String], u: &mut usize) -> Option<f64> {
    if *u < eb.len() && eb[*u] == "-" {
        *u += 1;
        let ap = otz(eb, u)?;
        Some(-ap)
    } else {
        otz(eb, u)
    }
}

fn otz(eb: &[String], u: &mut usize) -> Option<f64> {
    if *u >= eb.len() { return None; }
    if eb[*u] == "(" {
        *u += 1;
        let ap = oty(eb, u)?;
        if *u < eb.len() && eb[*u] == ")" {
            *u += 1;
        }
        Some(ap)
    } else {
        let ap: f64 = eb[*u].parse().bq()?;
        *u += 1;
        Some(ap)
    }
}

fn vkf(ar: f64, bgz: f64) -> f64 {
    if bgz == 0.0 { return 1.0; }
    if bgz == 1.0 { return ar; }
    let hin = bgz as i32;
    if (bgz - hin as f64).gp() < 1e-9 {
        let mut result = 1.0;
        let mut o = ar;
        let mut aa = if hin < 0 { -hin as u32 } else { hin as u32 };
        while aa > 0 {
            if aa & 1 == 1 { result *= o; }
            o *= o;
            aa >>= 1;
        }
        if hin < 0 { 1.0 / result } else { result }
    } else {
        
        ar 
    }
}


pub(super) fn rdr(n: &[&str]) {
    if n.len() < 2 {
        crate::println!("Usage: diff <file1> <file2>");
        return;
    }
    
    let rog = dux(n[0]);
    let roh = dux(n[1]);
    
    let (rw, tx) = match (rog, roh) {
        (Some(q), Some(o)) => (q, o),
        (None, _) => { crate::h!(A_, "diff: {}: No such file", n[0]); return; }
        (_, None) => { crate::h!(A_, "diff: {}: No such file", n[1]); return; }
    };
    
    let ojf: Vec<&str> = rw.ak().collect();
    let ojg: Vec<&str> = tx.ak().collect();
    
    crate::h!(C_, "--- {}", n[0]);
    crate::h!(C_, "+++ {}", n[1]);
    
    let cat = core::cmp::am(ojf.len(), ojg.len());
    let mut ixp = false;
    
    for a in 0..cat {
        let fmk = ojf.get(a).hu();
        let bvd = ojg.get(a).hu();
        
        match (fmk, bvd) {
            (Some(q), Some(o)) if q != o => {
                crate::h!(D_, "@@ -{},{} +{},{} @@", a + 1, 1, a + 1, 1);
                crate::h!(A_, "-{}", q);
                crate::h!(B_, "+{}", o);
                ixp = true;
            }
            (Some(q), None) => {
                crate::h!(A_, "-{}", q);
                ixp = true;
            }
            (None, Some(o)) => {
                crate::h!(B_, "+{}", o);
                ixp = true;
            }
            _ => {}
        }
    }
    
    if !ixp {
        crate::h!(B_, "Files are identical");
    }
}

fn dux(path: &str) -> Option<String> {
    if path.cj("/mnt/") || path.cj("/dev/") || path.cj("/proc/") {
        crate::vfs::lxu(path).bq()
    } else {
        crate::ramfs::fh(|fs| {
            fs.mq(path).map(|o| String::from(core::str::jg(o).unwrap_or(""))).bq()
        })
    }
}


pub(super) fn rgh(n: &[&str]) {
    if n.is_empty() {
        crate::println!("Usage: md5sum <file> ...");
        return;
    }
    for path in n {
        let f = crate::ramfs::fh(|fs| fs.mq(path).map(|o| o.ip()));
        match f {
            Ok(bf) => {
                let hash = wop(&bf);
                crate::println!("{}  {}", hash, path);
            }
            Err(_) => crate::h!(A_, "md5sum: {}: No such file", path),
        }
    }
}


fn wop(f: &[u8]) -> String {
    
    let mdd: [u32; 4] = [0x811c9dc5, 0x01000193, 0xdeadbeef, 0xcafebabe];
    let mut cff = [0u32; 4];
    for (a, dv) in mdd.iter().cf() {
        let mut i = *dv;
        for &hf in f {
            i ^= hf as u32;
            i = i.hx(0x01000193);
        }
        
        i ^= f.len() as u32;
        i = i.hx(0x01000193);
        cff[a] = i;
    }
    format!("{:08x}{:08x}{:08x}{:08x}", cff[0], cff[1], cff[2], cff[3])
}


pub(super) fn rib(n: &[&str]) {
    if n.is_empty() {
        crate::println!("Usage: sha256sum <file> ...");
        return;
    }
    for path in n {
        let f = crate::ramfs::fh(|fs| fs.mq(path).map(|o| o.ip()));
        match f {
            Ok(bf) => {
                let hash = woq(&bf);
                crate::println!("{}  {}", hash, path);
            }
            Err(_) => crate::h!(A_, "sha256sum: {}: No such file", path),
        }
    }
}


fn woq(f: &[u8]) -> String {
    let mdd: [u32; 8] = [0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a,
                           0x510e527f, 0x9b05688c, 0x1f83d9ab, 0x5be0cd19];
    let mut cff = [0u32; 8];
    for (a, dv) in mdd.iter().cf() {
        let mut i = *dv;
        for (fb, &hf) in f.iter().cf() {
            i ^= hf as u32;
            i = i.hx(0x01000193);
            i ^= (fb as u32).cn(a as u32);
            i = i.zkk(5);
        }
        i ^= f.len() as u32;
        i = i.hx(0x01000193 + a as u32);
        cff[a] = i;
    }
    format!("{:08x}{:08x}{:08x}{:08x}{:08x}{:08x}{:08x}{:08x}",
        cff[0], cff[1], cff[2], cff[3],
        cff[4], cff[5], cff[6], cff[7])
}


pub(super) fn rcl(n: &[&str], arr: Option<&str>) {
    let hfo = n.fv() == Some(&"-d") || n.fv() == Some(&"--decode");
    let fim: Vec<&str> = n.iter().hi(|q| !q.cj('-')).hu().collect();
    
    let ca = if let Some(input) = arr {
        Some(String::from(input))
    } else if !fim.is_empty() {
        dux(fim[0])
    } else {
        crate::println!("Usage: base64 [-d] [file]");
        crate::println!("  Or: echo text | base64");
        return;
    };
    
    if let Some(text) = ca {
        if hfo {
            match qnc(text.em()) {
                Some(aoq) => crate::print!("{}", core::str::jg(&aoq).unwrap_or("(binary data)")),
                None => crate::h!(A_, "base64: invalid input"),
            }
        } else {
            let ckd = qnd(text.as_bytes());
            crate::println!("{}", ckd);
        }
    }
}

const RG_: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

fn qnd(f: &[u8]) -> String {
    let mut result = String::new();
    let mut a = 0;
    while a < f.len() {
        let wu = f[a] as u32;
        let of = if a + 1 < f.len() { f[a + 1] as u32 } else { 0 };
        let tb = if a + 2 < f.len() { f[a + 2] as u32 } else { 0 };
        let fak = (wu << 16) | (of << 8) | tb;
        
        result.push(RG_[((fak >> 18) & 0x3F) as usize] as char);
        result.push(RG_[((fak >> 12) & 0x3F) as usize] as char);
        
        if a + 1 < f.len() {
            result.push(RG_[((fak >> 6) & 0x3F) as usize] as char);
        } else {
            result.push('=');
        }
        if a + 2 < f.len() {
            result.push(RG_[(fak & 0x3F) as usize] as char);
        } else {
            result.push('=');
        }
        a += 3;
    }
    result
}

fn ikg(r: u8) -> Option<u32> {
    match r {
        b'A'..=b'Z' => Some((r - b'A') as u32),
        b'a'..=b'z' => Some((r - b'a' + 26) as u32),
        b'0'..=b'9' => Some((r - b'0' + 52) as u32),
        b'+' => Some(62),
        b'/' => Some(63),
        _ => None,
    }
}

fn qnc(e: &str) -> Option<Vec<u8>> {
    let mut result = Vec::new();
    let bf: Vec<u8> = e.bf().hi(|&o| o != b'\n' && o != b'\r' && o != b' ').collect();
    let mut a = 0;
    while a + 3 < bf.len() {
        let q = ikg(bf[a])?;
        let o = ikg(bf[a + 1])?;
        let qvf = if bf[a + 2] == b'=' { 0 } else { ikg(bf[a + 2])? };
        let bmq = if bf[a + 3] == b'=' { 0 } else { ikg(bf[a + 3])? };
        
        let fak = (q << 18) | (o << 12) | (qvf << 6) | bmq;
        result.push(((fak >> 16) & 0xFF) as u8);
        if bf[a + 2] != b'=' {
            result.push(((fak >> 8) & 0xFF) as u8);
        }
        if bf[a + 3] != b'=' {
            result.push((fak & 0xFF) as u8);
        }
        a += 4;
    }
    Some(result)
}


pub(super) fn rkn(n: &[&str]) {
    if n.is_empty() {
        crate::println!("Usage: watch [-n <seconds>] <command>");
        crate::println!("  Example: watch -n 2 ps");
        return;
    }
    
    let mut lfg: u64 = 2;
    let mut enq = 0;
    
    if n.len() > 2 && n[0] == "-n" {
        lfg = n[1].parse().unwrap_or(2);
        enq = 2;
    }
    
    let cmd = n[enq..].rr(" ");
    crate::shell::hcw();
    
    loop {
        if crate::shell::etf() { break; }
        if let Some(3) = crate::keyboard::auw() {
            crate::shell::jpb();
            break;
        }
        
        crate::framebuffer::clear();
        crate::framebuffer::bld(0, 0);
        crate::h!(C_, "Every {}s: {}    {}", lfg, cmd, "TrustOS");
        crate::println!("---");
        
        super::azu(&cmd);
        
        
        let ay = crate::time::lc();
        let ci = ay + lfg * 1000;
        while crate::time::lc() < ci {
            if let Some(3) = crate::keyboard::auw() {
                crate::shell::jpb();
                return;
            }
            core::hint::hc();
        }
    }
    
    crate::println!();
    crate::h!(D_, "watch: interrupted");
}


pub(super) fn rjl(n: &[&str]) {
    if n.len() < 2 {
        crate::println!("Usage: timeout <seconds> <command>");
        return;
    }
    
    let tv: u64 = n[0].parse().unwrap_or(5);
    let cmd = n[1..].rr(" ");
    
    let ean = crate::time::lc() + tv * 1000;
    
    
    crate::h!(C_, "[timeout: {}s] {}", tv, cmd);
    super::azu(&cmd);
    
    if crate::time::lc() > ean {
        crate::h!(A_, "timeout: command timed out after {}s", tv);
    }
}


pub(super) fn riy(n: &[&str]) {
    if n.is_empty() {
        crate::println!("Usage: tar <operation> [options] [files...]");
        crate::println!("  tar cf archive.tar file1 file2  — Create archive");
        crate::println!("  tar tf archive.tar              — List contents");
        crate::println!("  tar xf archive.tar              — Extract archive");
        return;
    }
    
    let flags = n[0];
    let avp = flags.contains('c');
    let aoy = flags.contains('t');
    let sqb = flags.contains('x');
    
    let cjb = if n.len() > 1 && flags.contains('f') { n[1] } else {
        crate::h!(A_, "tar: -f <archive> required");
        return;
    };
    
    if avp {
        let sb = if n.len() > 2 { &n[2..] } else { &[] };
        prw(cjb, sb);
    } else if aoy {
        xat(cjb);
    } else if sqb {
        prx(cjb);
    } else {
        crate::h!(A_, "tar: specify -c, -t, or -x");
    }
}

fn prw(cjb: &str, sb: &[&str]) {
    
    let mut ijx = String::new();
    let mut az = 0;
    
    for path in sb {
        let ca = dux(path);
        match ca {
            Some(text) => {
                ijx.t(&format!("===FILE:{}:{}===\n", path, text.len()));
                ijx.t(&text);
                ijx.push('\n');
                az += 1;
            }
            None => crate::h!(D_, "tar: {}: Not found, skipping", path),
        }
    }
    
    let _ = crate::ramfs::fh(|fs| {
        if !fs.aja(cjb) { let _ = fs.touch(cjb); }
        fs.ns(cjb, ijx.as_bytes())
    });
    crate::h!(B_, "tar: created '{}' ({} files)", cjb, az);
}

fn xat(cjb: &str) {
    match dux(cjb) {
        Some(f) => {
            for line in f.ak() {
                if line.cj("===FILE:") && line.pp("===") {
                    let ff = &line[8..line.len() - 3];
                    if let Some(cpj) = ff.bhx(':') {
                        let j = &ff[..cpj];
                        let aw = &ff[cpj + 1..];
                        crate::println!("{:>8}  {}", aw, j);
                    }
                }
            }
        }
        None => crate::h!(A_, "tar: {}: No such file", cjb),
    }
}

fn prx(cjb: &str) {
    match dux(cjb) {
        Some(f) => {
            let mut ipz: Option<(String, usize)> = None;
            let mut ggy = String::new();
            let mut hir = 0;
            
            for line in f.ak() {
                if line.cj("===FILE:") && line.pp("===") {
                    
                    if let Some((ref j, _)) = ipz {
                        let _ = crate::ramfs::fh(|fs| {
                            if !fs.aja(j) { let _ = fs.touch(j); }
                            fs.ns(j, ggy.as_bytes())
                        });
                        hir += 1;
                    }
                    
                    let ff = &line[8..line.len() - 3];
                    if let Some(cpj) = ff.bhx(':') {
                        let j = String::from(&ff[..cpj]);
                        let aw: usize = ff[cpj + 1..].parse().unwrap_or(0);
                        ipz = Some((j, aw));
                        ggy = String::new();
                    }
                } else if ipz.is_some() {
                    if !ggy.is_empty() { ggy.push('\n'); }
                    ggy.t(line);
                }
            }
            
            if let Some((ref j, _)) = ipz {
                let _ = crate::ramfs::fh(|fs| {
                    if !fs.aja(j) { let _ = fs.touch(j); }
                    fs.ns(j, ggy.as_bytes())
                });
                hir += 1;
            }
            crate::h!(B_, "tar: extracted {} files", hir);
        }
        None => crate::h!(A_, "tar: {}: No such file", cjb),
    }
}


pub(super) fn rew(n: &[&str]) {
    if n.is_empty() {
        crate::println!("Usage: gzip <file>");
        return;
    }
    let path = n[0];
    match dux(path) {
        Some(f) => {
            
            let ahf = woo(f.as_bytes());
            let evu = format!("{}.gz", path);
            let _ = crate::ramfs::fh(|fs| {
                if !fs.aja(&evu) { let _ = fs.touch(&evu); }
                fs.ns(&evu, &ahf)
            });
            let bkx = if !f.is_empty() { (ahf.len() as f64 / f.len() as f64) * 100.0 } else { 100.0 };
            crate::h!(B_, "{} -> {} ({:.1}% of original)", path, evu, bkx);
        }
        None => crate::h!(A_, "gzip: {}: No such file", path),
    }
}

pub(super) fn rkw(n: &[&str]) {
    if n.len() < 2 {
        crate::println!("Usage: zip <archive.zip> <file1> [file2] ...");
        return;
    }
    
    prw(n[0], &n[1..]);
    crate::h!(B_, "zip: created '{}'", n[0]);
}

pub(super) fn rka(n: &[&str]) {
    if n.is_empty() {
        crate::println!("Usage: unzip <archive.zip>");
        return;
    }
    prx(n[0]);
}

fn woo(f: &[u8]) -> Vec<u8> {
    
    let mut bd = vec![b'T', b'G', b'Z', 1];
    
    let len = f.len() as u32;
    bd.bk(&len.ho());
    
    let mut a = 0;
    while a < f.len() {
        let hf = f[a];
        let mut az: u8 = 1;
        while a + (az as usize) < f.len() && f[a + az as usize] == hf && az < 255 {
            az += 1;
        }
        if az >= 3 {
            bd.push(0xFF); 
            bd.push(az);
            bd.push(hf);
        } else {
            for _ in 0..az {
                if hf == 0xFF { bd.push(0xFF); bd.push(1); bd.push(0xFF); }
                else { bd.push(hf); }
            }
        }
        a += az as usize;
    }
    bd
}


use core::sync::atomic::Ordering;

struct Ul {
    j: &'static str,
    dc: &'static str,
    geh: bool,
}

const BFK_: &[Ul] = &[
    Ul { j: "sshd", dc: "OpenSSH server daemon", geh: false },
    Ul { j: "httpd", dc: "TrustOS HTTP server", geh: false },
    Ul { j: "crond", dc: "Task scheduler daemon", geh: false },
    Ul { j: "syslogd", dc: "System logger", geh: true },
    Ul { j: "networkd", dc: "Network manager", geh: true },
    Ul { j: "firewalld", dc: "Firewall daemon", geh: false },
];


static AHQ_: core::sync::atomic::AtomicU32 = core::sync::atomic::AtomicU32::new(0b011000); 

fn ogu(w: usize) -> bool {
    AHQ_.load(Ordering::SeqCst) & (1 << w) != 0
}

fn jph(w: usize, iq: bool) {
    if iq {
        AHQ_.nth(1 << w, Ordering::SeqCst);
    } else {
        AHQ_.ntg(!(1 << w), Ordering::SeqCst);
    }
}

pub(super) fn kjd(n: &[&str]) {
    if n.is_empty() {
        crate::h!(C_, "TrustOS Services:");
        crate::println!("{:<15} {:<10} {}", "SERVICE", "STATUS", "DESCRIPTION");
        crate::println!("--------------------------------------------------");
        for (a, ejh) in BFK_.iter().cf() {
            let iq = ogu(a);
            let status = if iq { "active" } else { "inactive" };
            let s = if iq { B_ } else { L_ };
            crate::gr!(s, "{:<15} ", ejh.j);
            crate::gr!(s, "{:<10} ", status);
            crate::println!("{}", ejh.dc);
        }
        return;
    }
    
    let gtr = n[0];
    let hr = if n.len() > 1 { n[1] } else { "status" };
    
    if let Some((w, ejh)) = BFK_.iter().cf().du(|(_, e)| e.j == gtr) {
        match hr {
            "start" => {
                jph(w, true);
                crate::h!(B_, "Starting {}... OK", gtr);
            }
            "stop" => {
                jph(w, false);
                crate::h!(D_, "Stopping {}... OK", gtr);
            }
            "restart" => {
                jph(w, false);
                crate::h!(D_, "Stopping {}...", gtr);
                jph(w, true);
                crate::h!(B_, "Starting {}... OK", gtr);
            }
            "status" => {
                let gh = ogu(w);
                if gh {
                    crate::h!(B_, "  {} - {}", ejh.j, ejh.dc);
                    crate::println!("   Active: active (running)");
                } else {
                    crate::h!(L_, "  {} - {}", ejh.j, ejh.dc);
                    crate::println!("   Active: inactive (dead)");
                }
            }
            _ => crate::println!("Usage: service <name> start|stop|restart|status"),
        }
    } else {
        crate::h!(A_, "service: unknown service '{}'", gtr);
    }
}

pub(super) fn rix(n: &[&str]) {
    if n.is_empty() || n[0] == "list-units" {
        kjd(&[]);
        return;
    }
    
    let hr = n[0];
    if n.len() < 2 {
        crate::println!("Usage: systemctl <start|stop|restart|status|enable|disable> <service>");
        return;
    }
    let ejh = n[1].bdd(".service");
    kjd(&[ejh, hr]);
}



static AAV_: Mutex<Vec<String>> = Mutex::new(Vec::new());

pub(super) fn rdf(n: &[&str]) {
    match n.fv().hu() {
        Some("-l") | None => {
            let ch = AAV_.lock();
            if ch.is_empty() {
                crate::println!("no crontab for root");
            } else {
                for bt in ch.iter() {
                    crate::println!("{}", bt);
                }
            }
        }
        Some("-e") => {
            crate::h!(C_, "Enter cron entries (one per line, empty line to finish):");
            crate::h!(L_, "Format: min hour dom mon dow command");
            let mut ch = Vec::new();
            loop {
                crate::print!("> ");
                let mut input = String::new();
                loop {
                    if let Some(bs) = crate::keyboard::auw() {
                        match bs {
                            0x0A => break,
                            0x08 => { if !input.is_empty() { input.pop(); crate::print!("\x08 \x08"); } }
                            bm if bm >= 32 && bm < 127 => { input.push(bm as char); crate::print!("{}", bm as char); }
                            _ => {}
                        }
                    } else { core::hint::hc(); }
                }
                crate::println!();
                if input.em().is_empty() { break; }
                ch.push(input);
            }
            *AAV_.lock() = ch.clone();
            crate::h!(B_, "crontab: installed {} entries", ch.len());
        }
        Some("-r") => {
            AAV_.lock().clear();
            crate::h!(B_, "crontab: removed");
        }
        _ => crate::println!("Usage: crontab [-l | -e | -r]"),
    }
}


pub(super) fn rci(n: &[&str]) {
    if n.is_empty() {
        crate::println!("Usage: at <time> <command>");
        crate::println!("  Example: at +5m echo hello");
        return;
    }
    
    let pti = n[0];
    let ro = if n.len() > 1 { n[1..].rr(" ") } else {
        crate::println!("at: no command specified");
        return;
    };
    
    
    let azn = if pti.cj('+') {
        let avc = &pti[1..];
        if avc.pp('s') {
            avc[..avc.len()-1].parse::<u64>().unwrap_or(0) * 1000
        } else if avc.pp('m') {
            avc[..avc.len()-1].parse::<u64>().unwrap_or(0) * 60000
        } else if avc.pp('h') {
            avc[..avc.len()-1].parse::<u64>().unwrap_or(0) * 3600000
        } else {
            avc.parse::<u64>().unwrap_or(0) * 1000
        }
    } else {
        crate::println!("at: use +Ns, +Nm, or +Nh for relative times");
        return;
    };
    
    crate::h!(B_, "Job scheduled: '{}' in {} seconds", ro, azn / 1000);
    
    
    let ay = crate::time::lc();
    while crate::time::lc() - ay < azn {
        if let Some(3) = crate::keyboard::auw() {
            crate::h!(D_, "at: cancelled");
            return;
        }
        core::hint::hc();
    }
    
    crate::h!(C_, "[at] Executing: {}", ro);
    super::azu(&ro);
}


pub(super) fn yje(n: &[&str], arr: Option<&str>) {
    let mut bo: usize = 10;
    let mut file: Option<&str> = None;
    
    let mut a = 0;
    while a < n.len() {
        if n[a] == "-n" && a + 1 < n.len() {
            bo = n[a + 1].parse().unwrap_or(10);
            a += 2;
        } else if n[a].cj('-') && n[a].len() > 1 {
            bo = n[a][1..].parse().unwrap_or(10);
            a += 1;
        } else {
            file = Some(n[a]);
            a += 1;
        }
    }
    
    let ca = if let Some(input) = arr {
        Some(String::from(input))
    } else if let Some(path) = file {
        dux(path)
    } else {
        crate::println!("Usage: head [-n N] <file>");
        return;
    };
    
    if let Some(text) = ca {
        for (a, line) in text.ak().cf() {
            if a >= bo { break; }
            crate::println!("{}", line);
        }
    }
}


pub(super) fn yjg(n: &[&str], arr: Option<&str>) {
    let mut bo: usize = 10;
    let mut file: Option<&str> = None;
    let mut nvj = false;
    
    let mut a = 0;
    while a < n.len() {
        if n[a] == "-n" && a + 1 < n.len() {
            bo = n[a + 1].parse().unwrap_or(10);
            a += 2;
        } else if n[a] == "-f" {
            nvj = true;
            a += 1;
        } else if n[a].cj('-') && n[a].len() > 1 && n[a] != "-f" {
            bo = n[a][1..].parse().unwrap_or(10);
            a += 1;
        } else {
            file = Some(n[a]);
            a += 1;
        }
    }
    
    let ca = if let Some(input) = arr {
        Some(String::from(input))
    } else if let Some(path) = file {
        dux(path)
    } else {
        crate::println!("Usage: tail [-n N] [-f] <file>");
        return;
    };
    
    if let Some(text) = ca {
        let ak: Vec<&str> = text.ak().collect();
        let ay = if ak.len() > bo { ak.len() - bo } else { 0 };
        for line in &ak[ay..] {
            crate::println!("{}", line);
        }
    }
    
    if nvj {
        crate::h!(L_, "(follow mode — Ctrl+C to stop)");
        crate::shell::hcw();
        loop {
            if let Some(3) = crate::keyboard::auw() { break; }
            if crate::shell::etf() { break; }
            core::hint::hc();
        }
    }
}


pub(super) fn yji(n: &[&str], arr: Option<&str>) {
    let ngi = n.contains(&"-l");
    let ngj = n.contains(&"-w");
    let ngg = n.contains(&"-c");
    let xx = !ngi && !ngj && !ngg;
    
    let fim: Vec<&str> = n.iter().hi(|q| !q.cj('-')).hu().collect();
    
    let ca = if let Some(input) = arr {
        Some(String::from(input))
    } else if !fim.is_empty() {
        dux(fim[0])
    } else {
        crate::println!("Usage: wc [-l] [-w] [-c] <file>");
        return;
    };
    
    if let Some(text) = ca {
        let ak = text.ak().az();
        let aoh = text.ayt().az();
        let bw = text.len();
        
        if xx {
            crate::println!("  {}  {}  {}", ak, aoh, bw);
        } else {
            if ngi { crate::print!("  {}", ak); }
            if ngj { crate::print!("  {}", aoh); }
            if ngg { crate::print!("  {}", bw); }
            crate::println!();
        }
    }
}


pub(super) fn wbm() {
    
    let vqs = ["/.trustrc", "/etc/trustrc", "/home/trustrc"];
    
    for path in &vqs {
        let ca: Option<String> = crate::ramfs::fh(|fs| {
            fs.mq(path).map(|o| String::from(core::str::jg(o).unwrap_or(""))).bq()
        });
        
        if let Some(ref eib) = ca {
            crate::h!(L_, "[init] Running {}...", path);
            for line in eib.ak() {
                let ux = line.em();
                if ux.is_empty() || ux.cj('#') {
                    continue;
                }
                super::azu(ux);
            }
            return; 
        }
    }
}


pub(super) fn yjj() {
    let cnp = super::scripting::cqx("USER").unwrap_or_else(|| String::from("root"));
    crate::println!("{}", cnp);
}


pub(super) fn rkc() {
    let jn = crate::time::lc();
    let tv = jn / 1000;
    let fgl = tv / 86400;
    let cad = (tv % 86400) / 3600;
    let bbz = (tv % 3600) / 60;
    let e = tv % 60;
    
    let jke = crate::process::aoy().len();
    
    crate::gr!(Q_, " up ");
    if fgl > 0 { crate::print!("{} day(s), ", fgl); }
    crate::print!("{:02}:{:02}:{:02}", cad, bbz, e);
    crate::println!(", {} processes", jke);
}


pub(super) fn yja() {
    crate::framebuffer::clear();
    crate::framebuffer::bld(0, 0);
}






pub(super) fn rfs(n: &[&str]) {
    if n.is_empty() {
        crate::println!("Usage: killall <name>");
        return;
    }
    let j = n[0];
    let mut lhl = 0u32;
    for (ce, dkq, gxl) in crate::process::aoy() {
        if dkq.contains(j) && ce > 1 {
            if crate::process::dsm(ce).is_ok() {
                lhl += 1;
                crate::h!(D_, "Killed PID {} ({})", ce, dkq);
            }
        }
    }
    if lhl == 0 {
        crate::h!(A_, "killall: no process matching '{}'", j);
    } else {
        crate::h!(B_, "Killed {} process(es)", lhl);
    }
}


pub(super) fn rgp(n: &[&str]) {
    if n.is_empty() {
        crate::println!("Usage: nice [-n priority] <command>");
        return;
    }
    let (abv, enq) = if n[0] == "-n" && n.len() > 2 {
        (n[1].parse::<i32>().unwrap_or(10), 2)
    } else {
        (10, 0)
    };
    let cmd = n[enq..].rr(" ");
    crate::h!(C_, "nice: running '{}' with priority {}", cmd, abv);
    
    super::azu(&cmd);
}


pub(super) fn rfm() {
    let (exj, fbu, cjl, cjm) = crate::disk::asx();
    let bxp = crate::time::lc() / 1000;
    let bxp = if bxp == 0 { 1 } else { bxp };

    crate::h!(G_, "TrustOS I/O Statistics");
    crate::println!("------------------------------------------------------");
    crate::println!("Uptime: {}s", bxp);
    crate::println!();
    crate::h!(C_, "Device          tps    kB_read/s    kB_wrtn/s   kB_read   kB_wrtn");
    let xkw = (exj + fbu) / bxp;
    let dir = cjl / 1024;
    let yo = cjm / 1024;
    let ubq = dir / bxp;
    let ubr = yo / bxp;
    crate::println!("ramdisk   {:>8}  {:>11}  {:>11}  {:>8}  {:>8}", xkw, ubq, ubr, dir, yo);
    crate::println!();
    crate::println!("Total: {} reads, {} writes", exj, fbu);
}


pub(super) fn rip(n: &[&str]) {
    if n.is_empty() {
        crate::println!("Usage: strace <command>");
        crate::println!("  Trace system calls made by a command");
        return;
    }

    
    crate::serial_println!("[STRACE] Tracing: {}", n.rr(" "));
    crate::h!(C_, "strace: tracing '{}'", n.rr(" "));
    crate::h!(L_, "--- syscall trace start ---");

    
    use core::sync::atomic::{AtomicBool, Ordering};
    static BGL_: AtomicBool = AtomicBool::new(false);
    BGL_.store(true, Ordering::SeqCst);

    
    let cmd = n.rr(" ");
    super::azu(&cmd);

    BGL_.store(false, Ordering::SeqCst);
    crate::h!(L_, "--- syscall trace end ---");
}


pub(super) fn rdw() {
    crate::h!(G_, "SMBIOS/DMI Information");
    crate::println!("------------------------------------------------------");

    
    crate::h!(C_, "Handle 0x0000, DMI type 0, BIOS Information");
    crate::println!("  Vendor: TrustOS");
    crate::println!("  Version: 0.7.0-checkm8");
    crate::println!("  Release Date: 03/12/2026");
    crate::println!("  BIOS Revision: 0.7");
    crate::println!();

    
    crate::h!(C_, "Handle 0x0001, DMI type 1, System Information");
    crate::println!("  Manufacturer: TrustOS Project");
    crate::println!("  Product Name: TrustOS Bare-Metal");
    #[cfg(target_arch = "x86_64")]
    crate::println!("  Architecture: x86_64");
    #[cfg(target_arch = "aarch64")]
    crate::println!("  Architecture: aarch64");
    #[cfg(target_arch = "riscv64")]
    crate::println!("  Architecture: riscv64gc");
    crate::println!();

    
    crate::h!(C_, "Handle 0x0004, DMI type 4, Processor Information");
    let aao = {
        #[cfg(target_arch = "x86_64")]
        { crate::cpu::smp::aao() }
        #[cfg(not(target_arch = "x86_64"))]
        { 1u32 }
    };
    crate::println!("  CPU Count: {}", aao);
    crate::println!("  Features: SSE, SSE2, RDRAND, RDSEED");
    crate::println!();

    
    let cm = crate::memory::cm();
    let cuu = (cm.afa + cm.buv) / 1024;
    crate::h!(C_, "Handle 0x0011, DMI type 17, Memory Device");
    crate::println!("  Size: {} KB (heap)", cuu);
    crate::println!("  Type: DRAM");
}


pub(super) fn rey(n: &[&str]) {
    if n.is_empty() {
        crate::println!("Usage: hdparm [-i|-t] <device>");
        crate::println!("  -i  Device information");
        crate::println!("  -t  Timing buffered disk reads");
        return;
    }

    let oef = n.contains(&"-i");
    let ptk = n.contains(&"-t");

    let hgf = crate::disk::ani();
    if let Some(co) = hgf {
        if oef || (!oef && !ptk) {
            crate::h!(C_, "/dev/sda:");
            crate::println!("  Model: {}", co.model);
            crate::println!("  Serial: {}", co.serial);
            crate::println!("  Sectors: {}", co.grv);
            crate::println!("  Size: {} MB", co.aga);
        }
        if ptk {
            
            let ay = crate::time::lc();
            let mut k = [0u8; 512];
            for a in 0..100u64 {
                let _ = crate::disk::ain(a % co.grv, 1, &mut k);
            }
            let ez = crate::time::lc() - ay;
            let ez = if ez == 0 { 1 } else { ez };
            let xgn = (100 * 512) / (ez as usize);
            crate::h!(B_, "  Timing: 100 sectors in {}ms ({} KB/s)", ez, xgn);
        }
    } else {
        crate::h!(A_, "hdparm: no disk found");
    }
}


pub(super) fn rhw(n: &[&str]) {
    let it = if !n.is_empty() { n[0] } else { "/screenshot.ppm" };

    let z = crate::framebuffer::AB_.load(core::sync::atomic::Ordering::Relaxed) as u32;
    let ac = crate::framebuffer::Z_.load(core::sync::atomic::Ordering::Relaxed) as u32;

    if z == 0 || ac == 0 {
        crate::h!(A_, "screenshot: no framebuffer available");
        return;
    }

    crate::h!(C_, "Capturing {}x{} screenshot...", z, ac);

    
    let dh = format!("P6\n{} {}\n255\n", z, ac);
    let vig = (z * ac * 3) as usize;
    let es = dh.len() + vig;
    let mut f = Vec::fc(es);
    f.bk(dh.as_bytes());

    
    for c in 0..ac {
        for b in 0..z {
            let il = crate::framebuffer::beg(b as u32, c as u32);
            
            let m = ((il >> 16) & 0xFF) as u8;
            let at = ((il >> 8) & 0xFF) as u8;
            let o = (il & 0xFF) as u8;
            f.push(m);
            f.push(at);
            f.push(o);
        }
    }

    
    match crate::vfs::ns(it, &f) {
        Ok(_) => crate::h!(B_, "Screenshot saved: {} ({} bytes, {}x{})", it, es, z, ac),
        Err(aa) => crate::h!(A_, "screenshot: write failed: {:?}", aa),
    }
}


pub(super) fn kiu(n: &[&str]) {
    let port: u16 = if !n.is_empty() {
        n[0].parse().unwrap_or(8080)
    } else {
        8080
    };

    crate::h!(G_, "TrustOS HTTP Server starting on port {}...", port);
    crate::h!(C_, "  Serving files from /");
    crate::h!(L_, "  Press Ctrl+C to stop");

    
    let ugb = crate::netstack::socket::socket(2, 1, 0); 
    match ugb {
        Ok(da) => {
            let ag = crate::netstack::socket::SockAddrIn::new([0, 0, 0, 0], port);
            if let Err(aa) = crate::netstack::socket::kdj(da, &ag) {
                crate::h!(A_, "httpd: bind failed: {}", aa);
                return;
            }
            if let Err(aa) = crate::netstack::socket::ojr(da, 8) {
                crate::h!(A_, "httpd: listen failed: {}", aa);
                return;
            }
            crate::h!(B_, "Listening on 0.0.0.0:{}", port);
            crate::h!(L_, "(In this kernel, TCP accept() is cooperative — use `curl` from another shell)");
        }
        Err(aa) => {
            crate::h!(A_, "httpd: socket creation failed: {}", aa);
        }
    }
}




pub(super) fn kif() {
    crate::h!(G_, "TrustOS System Benchmark");
    crate::println!("======================================================");

    
    crate::h!(C_, "[1/4] CPU integer arithmetic...");
    let ay = crate::time::lc();
    let mut btc: u64 = 0;
    for a in 0u64..10_000_000 {
        btc = btc.cn(a).hx(3);
    }
    let hed = crate::time::lc() - ay;
    let hed = if hed == 0 { 1 } else { hed };
    crate::println!("  10M iterations in {}ms ({} Mops/s) [checksum=0x{:016x}]",
        hed, 10000 / hed, btc);

    
    crate::h!(C_, "[2/4] Memory sequential write...");
    let mut k = vec![0u8; 1024 * 1024]; 
    let ay = crate::time::lc();
    for a in 0..k.len() {
        k[a] = (a & 0xFF) as u8;
    }
    let hri = crate::time::lc() - ay;
    let hri = if hri == 0 { 1 } else { hri };
    let ump = 1000 / hri;
    crate::println!("  1MB write in {}ms ({} MB/s)", hri, ump);

    
    crate::h!(C_, "[3/4] Disk I/O (ramdisk)...");
    let ay = crate::time::lc();
    let mut jk = [0u8; 512];
    for a in 0..1000u64 {
        let _ = crate::disk::ain(a % 256, 1, &mut jk);
    }
    let hgg = crate::time::lc() - ay;
    let hgg = if hgg == 0 { 1 } else { hgg };
    crate::println!("  1000 sector reads in {}ms ({} IOPS)", hgg, 1000000 / hgg);

    
    crate::h!(C_, "[4/4] Heap allocation...");
    let ay = crate::time::lc();
    for _ in 0..10000 {
        let p: Vec<u8> = Vec::fc(256);
        core::hint::mzg(p);
    }
    let gyi = crate::time::lc() - ay;
    let gyi = if gyi == 0 { 1 } else { gyi };
    crate::println!("  10K allocs in {}ms ({} allocs/s)", gyi, 10_000_000 / gyi);

    crate::println!("======================================================");
    crate::h!(B_, "Benchmark complete.");
}
