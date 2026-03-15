












use alloc::string::String;
use alloc::format;
use alloc::vec::Vec;


fn ozz() -> u64 {
    #[cfg(target_arch = "aarch64")]
    {
        let ffg: u64;
        unsafe {
            core::arch::asm!(
                "mrs {}, cntvct_el0",
                bd(reg) ffg,
                options(nomem, nostack)
            );
        }
        ffg
    }
    
    #[cfg(target_arch = "x86_64")]
    {
        let hh: u32;
        let gd: u32;
        unsafe {
            core::arch::asm!(
                "rdtsc",
                bd("eax") hh,
                bd("edx") gd,
                options(nomem, nostack)
            );
        }
        ((gd as u64) << 32) | (hh as u64)
    }
    
    #[cfg(target_arch = "riscv64")]
    {
        let ffg: u64;
        unsafe {
            core::arch::asm!(
                "rdcycle {}",
                bd(reg) ffg,
                options(nomem, nostack)
            );
        }
        ffg
    }
}


fn tew() -> u64 {
    #[cfg(target_arch = "aarch64")]
    {
        let kx: u64;
        unsafe {
            core::arch::asm!(
                "mrs {}, cntfrq_el0",
                bd(reg) kx,
                options(nomem, nostack)
            );
        }
        kx
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


fn omq(ag: u64, atc: usize) -> (u64, u64, u64) {
    let mut llw = u64::O;
    let mut lku = 0u64;
    let mut pux = 0u64;
    
    for _ in 0..atc {
        
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
        
        let ay = ozz();
        
        
        unsafe {
            let ptr = ag as *const u32;
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
        
        let ci = ozz();
        let ez = ci.nj(ay);
        
        if ez < llw { llw = ez; }
        if ez > lku { lku = ez; }
        pux += ez;
    }
    
    let abl = pux / atc as u64;
    (llw, abl, lku)
}


fn rbb(qlw: u64, myd: u64) -> (&'static str, &'static str) {
    let bkx = if myd > 0 {
        (qlw * 100) / myd
    } else {
        100
    };
    
    match bkx {
        0..=80 => ("FAST", "\x01G"),    
        81..=120 => ("NORMAL", "\x01W"), 
        121..=300 => ("SLOW", "\x01Y"),  
        301..=1000 => ("VERY SLOW", "\x01R"), 
        _ => ("ANOMALOUS", "\x01R"),     
    }
}


pub fn per(n: &str) -> String {
    let mut an = String::new();
    
    an.t("\x01C== TrustProbe: Timing Side-Channel Analyzer ==\x01W\n\n");
    
    let kx = tew();
    an.t(&format!("Timer frequency: {} Hz ({} MHz)\n", kx, kx / 1_000_000));
    
    let atc = 10;
    
    
    an.t("\n\x01Y--- Baseline Measurement ---\x01W\n");
    
    
    let qnt = &an as *const String as u64;
    let qnu = qnt & !0xFFF;
    let (qps, fdk, qpr) = omq(qnu, atc);
    
    an.t(&format!("Baseline (kernel memory): min={} avg={} max={} cycles\n",
        qps, fdk, qpr));
    
    let org = if kx > 0 { 1_000_000_000 / kx } else { 1 };
    an.t(&format!("  ~{} ns per access (avg)\n", fdk * org));
    
    
    an.t("\n\x01Y--- Region Timing Comparison ---\x01W\n");
    an.t(&format!("{:<16} {:<10} {:<10} {:<10} {:<10} {}\n",
        "ADDRESS", "MIN", "AVG", "MAX", "RATIO", "CLASS"));
    an.t(&format!("{}\n", "-".afd(70)));
    
    
    #[cfg(target_arch = "aarch64")]
    let jka: Vec<(u64, &str)> = alloc::vec![
        (0x0800_0000, "GIC"),
        (0x0900_0000, "UART"),
        (0x0A00_0000, "VirtIO"),
        (0x0E00_0000, "Secure SRAM"),
        (0x4000_0000, "RAM (low)"),
        (0x8000_0000, "RAM (high)"),
    ];
    
    #[cfg(target_arch = "x86_64")]
    let jka: Vec<(u64, &str)> = alloc::vec![
        (0x000A_0000, "VGA/SMRAM"),
        (0x000F_0000, "BIOS area"),
        (0xFEC0_0000, "I/O APIC"),
        (0xFEE0_0000, "Local APIC"),
        (0xFED0_0000, "HPET"),
    ];
    
    #[cfg(target_arch = "riscv64")]
    let jka: Vec<(u64, &str)> = alloc::vec![
        (0x0200_0000, "CLINT"),
        (0x0C00_0000, "PLIC"),
        (0x1000_0000, "UART"),
        (0x8000_0000, "RAM"),
    ];
    
    let mut dyi = Vec::new();
    
    for (ag, j) in &jka {
        let (val, lrl, vak) = omq(*ag, atc);
        let bkx = if fdk > 0 { (lrl * 100) / fdk } else { 0 };
        let (class, s) = rbb(lrl, fdk);
        
        an.t(&format!("0x{:010X}   {:<10} {:<10} {:<10} {:<10} {}{}\x01W ({})\n",
            ag, val, lrl, vak,
            format!("{}%", bkx), s, class, j));
        
        if bkx > 200 || bkx < 50 {
            dyi.push((*ag, *j, bkx, class));
        }
    }
    
    
    if !dyi.is_empty() {
        an.t(&format!("\n\x01Y--- Anomaly Details ---\x01W\n"));
        an.t(&format!("Found {} timing anomalies:\n\n", dyi.len()));
        
        for (ag, j, bkx, class) in &dyi {
            an.t(&format!("\x01R[{}]\x01W {} @ 0x{:010X} ({}% of baseline)\n",
                class, j, ag, bkx));
            
            if *bkx > 300 {
                an.t("    Interpretation: This region likely triggers a fault/exception.\n");
                an.t("    This could indicate secure memory, MMIO, or unmapped region.\n");
            } else if *bkx < 50 {
                an.t("    Interpretation: Faster than DRAM — could be SRAM or cached.\n");
                an.t("    This might be a tightly-coupled memory or L1 cache hit.\n");
            }
        }
    }
    
    
    an.t(&format!("\n\x01C== Timing Analysis Summary ==\x01W\n"));
    an.t(&format!("  Regions tested: {}\n", jka.len()));
    an.t(&format!("  Anomalies: {}\n", dyi.len()));
    an.t(&format!("  Baseline: {} cycles ({} ns)\n", fdk, fdk * org));
    
    if dyi.iter().any(|(_, _, m, _)| *m > 500) {
        an.t("\n\x01R[!] High-latency regions detected — possible secure boundaries\x01W\n");
        an.t("    Run 'hwscan trustzone' for detailed boundary mapping\n");
    }
    
    an
}
