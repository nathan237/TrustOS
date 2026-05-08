












use alloc::string::String;
use alloc::format;
use alloc::vec::Vec;


fn iyi() -> u64 {
    #[cfg(target_arch = "aarch64")]
    {
        let cnt: u64;
        unsafe {
            core::arch::asm!(
                "mrs {}, cntvct_el0",
                out(reg) cnt,
                options(nomem, nostack)
            );
        }
        cnt
    }
    
    #[cfg(target_arch = "x86_64")]
    {
        let lo: u32;
        let hi: u32;
        unsafe {
            core::arch::asm!(
                "rdtsc",
                out("eax") lo,
                out("edx") hi,
                options(nomem, nostack)
            );
        }
        ((hi as u64) << 32) | (lo as u64)
    }
    
    #[cfg(target_arch = "riscv64")]
    {
        let cnt: u64;
        unsafe {
            core::arch::asm!(
                "rdcycle {}",
                out(reg) cnt,
                options(nomem, nostack)
            );
        }
        cnt
    }
}


fn mdx() -> u64 {
    #[cfg(target_arch = "aarch64")]
    {
        let freq: u64;
        unsafe {
            core::arch::asm!(
                "mrs {}, cntfrq_el0",
                out(reg) freq,
                options(nomem, nostack)
            );
        }
        freq
    }
    
    #[cfg(target_arch = "x86_64")]
    {
        
        2_000_000_000
    }
    
    #[cfg(target_arch = "riscv64")]
    {
        
        10_000_000
    }
}


fn ind(addr: u64, xe: usize) -> (u64, u64, u64) {
    let mut ghs = u64::MAX;
    let mut ggu = 0u64;
    let mut jnt = 0u64;
    
    for _ in 0..xe {
        
        #[cfg(target_arch = "aarch64")]
        unsafe {
            core::arch::asm!("dsb sy", "isb", options(nomem, nostack));
        }
        #[cfg(target_arch = "x86_64")]
        unsafe {
            core::arch::asm!("mfence", options(nomem, nostack));
        }
        #[cfg(target_arch = "riscv64")]
        unsafe {
            core::arch::asm!("fence", options(nomem, nostack));
        }
        
        let start = iyi();
        
        
        unsafe {
            let ptr = addr as *const u32;
            let _ = core::ptr::read_volatile(ptr);
        }
        
        
        #[cfg(target_arch = "aarch64")]
        unsafe {
            core::arch::asm!("dsb sy", options(nomem, nostack));
        }
        #[cfg(target_arch = "x86_64")]
        unsafe {
            core::arch::asm!("mfence", options(nomem, nostack));
        }
        #[cfg(target_arch = "riscv64")]
        unsafe {
            core::arch::asm!("fence", options(nomem, nostack));
        }
        
        let end = iyi();
        let bb = end.wrapping_sub(start);
        
        if bb < ghs { ghs = bb; }
        if bb > ggu { ggu = bb; }
        jnt += bb;
    }
    
    let ns = jnt / xe as u64;
    (ghs, ns, ggu)
}


fn kkq(avg_cycles: u64, baseline_cycles: u64) -> (&'static str, &'static str) {
    let zi = if baseline_cycles > 0 {
        (avg_cycles * 100) / baseline_cycles
    } else {
        100
    };
    
    match zi {
        0..=80 => ("FAST", "\x01G"),    
        81..=120 => ("NORMAL", "\x01W"), 
        121..=300 => ("SLOW", "\x01Y"),  
        301..=1000 => ("VERY SLOW", "\x01R"), 
        _ => ("ANOMALOUS", "\x01R"),     
    }
}


pub fn jbw(args: &str) -> String {
    let mut output = String::new();
    
    output.push_str("\x01C== TrustProbe: Timing Side-Channel Analyzer ==\x01W\n\n");
    
    let freq = mdx();
    output.push_str(&format!("Timer frequency: {} Hz ({} MHz)\n", freq, freq / 1_000_000));
    
    let xe = 10;
    
    
    output.push_str("\n\x01Y--- Baseline Measurement ---\x01W\n");
    
    
    let kai = &output as *const String as u64;
    let kaj = kai & !0xFFF;
    let (bl_min, bl_avg, bl_max) = ind(kaj, xe);
    
    output.push_str(&format!("Baseline (kernel memory): min={} avg={} max={} cycles\n",
        bl_min, bl_avg, bl_max));
    
    let ird = if freq > 0 { 1_000_000_000 / freq } else { 1 };
    output.push_str(&format!("  ~{} ns per access (avg)\n", bl_avg * ird));
    
    
    output.push_str("\n\x01Y--- Region Timing Comparison ---\x01W\n");
    output.push_str(&format!("{:<16} {:<10} {:<10} {:<10} {:<10} {}\n",
        "ADDRESS", "MIN", "AVG", "MAX", "RATIO", "CLASS"));
    output.push_str(&format!("{}\n", "-".repeat(70)));
    
    
    #[cfg(target_arch = "aarch64")]
    let ewz: Vec<(u64, &str)> = alloc::vec![
        (0x0800_0000, "GIC"),
        (0x0900_0000, "UART"),
        (0x0A00_0000, "VirtIO"),
        (0x0E00_0000, "Secure SRAM"),
        (0x4000_0000, "RAM (low)"),
        (0x8000_0000, "RAM (high)"),
    ];
    
    #[cfg(target_arch = "x86_64")]
    let ewz: Vec<(u64, &str)> = alloc::vec![
        (0x000A_0000, "VGA/SMRAM"),
        (0x000F_0000, "BIOS area"),
        (0xFEC0_0000, "I/O APIC"),
        (0xFEE0_0000, "Local APIC"),
        (0xFED0_0000, "HPET"),
    ];
    
    #[cfg(target_arch = "riscv64")]
    let ewz: Vec<(u64, &str)> = alloc::vec![
        (0x0200_0000, "CLINT"),
        (0x0C00_0000, "PLIC"),
        (0x1000_0000, "UART"),
        (0x8000_0000, "RAM"),
    ];
    
    let mut bqg = Vec::new();
    
    for (addr, name) in &ewz {
        let (p_min, p_avg, p_max) = ind(*addr, xe);
        let zi = if bl_avg > 0 { (p_avg * 100) / bl_avg } else { 0 };
        let (class, color) = kkq(p_avg, bl_avg);
        
        output.push_str(&format!("0x{:010X}   {:<10} {:<10} {:<10} {:<10} {}{}\x01W ({})\n",
            addr, p_min, p_avg, p_max,
            format!("{}%", zi), color, class, name));
        
        if zi > 200 || zi < 50 {
            bqg.push((*addr, *name, zi, class));
        }
    }
    
    
    if !bqg.is_empty() {
        output.push_str(&format!("\n\x01Y--- Anomaly Details ---\x01W\n"));
        output.push_str(&format!("Found {} timing anomalies:\n\n", bqg.len()));
        
        for (addr, name, zi, class) in &bqg {
            output.push_str(&format!("\x01R[{}]\x01W {} @ 0x{:010X} ({}% of baseline)\n",
                class, name, addr, zi));
            
            if *zi > 300 {
                output.push_str("    Interpretation: This region likely triggers a fault/exception.\n");
                output.push_str("    This could indicate secure memory, MMIO, or unmapped region.\n");
            } else if *zi < 50 {
                output.push_str("    Interpretation: Faster than DRAM — could be SRAM or cached.\n");
                output.push_str("    This might be a tightly-coupled memory or L1 cache hit.\n");
            }
        }
    }
    
    
    output.push_str(&format!("\n\x01C== Timing Analysis Summary ==\x01W\n"));
    output.push_str(&format!("  Regions tested: {}\n", ewz.len()));
    output.push_str(&format!("  Anomalies: {}\n", bqg.len()));
    output.push_str(&format!("  Baseline: {} cycles ({} ns)\n", bl_avg, bl_avg * ird));
    
    if bqg.iter().any(|(_, _, r, _)| *r > 500) {
        output.push_str("\n\x01R[!] High-latency regions detected — possible secure boundaries\x01W\n");
        output.push_str("    Run 'hwscan trustzone' for detailed boundary mapping\n");
    }
    
    output
}
