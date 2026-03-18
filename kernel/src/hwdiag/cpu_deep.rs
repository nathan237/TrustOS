//! Deep CPU Diagnostics — comprehensive CPU profiling for any machine
//!
//! Dumps all CPUID leaves, cache topology, TLB structure, microcode revision,
//! frequency estimation, feature flags, and security vulnerability mitigations.

use alloc::string::String;
use alloc::format;
use alloc::vec::Vec;
use super::dbg_out;

/// Run deep CPU diagnostics
pub fn run(args: &[&str]) {
    let verbose = args.contains(&"-v") || args.contains(&"--verbose");

    dbg_out!("[CPU] === Deep CPU Analysis ===");

    #[cfg(target_arch = "x86_64")]
    {
        dump_vendor_brand();
        dump_topology();
        dump_features();
        dump_cache_info();
        dump_tlb_info();
        dump_frequency();
        dump_microcode();
        dump_security_mitigations();
        dump_power_management();
        if verbose {
            dump_all_cpuid_leaves();
        }
    }

    #[cfg(target_arch = "aarch64")]
    {
        dump_aarch64_info();
    }

    #[cfg(target_arch = "riscv64")]
    {
        dbg_out!("[CPU] RISC-V — CPUID not available, checking ISA extensions via CSR");
        dbg_out!("[CPU] (ISA extension probing not yet implemented for riscv64)");
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// x86_64 CPUID-based diagnostics
// ═══════════════════════════════════════════════════════════════════════════════
#[cfg(target_arch = "x86_64")]
fn cpuid(leaf: u32, subleaf: u32) -> (u32, u32, u32, u32) {
    let result = unsafe { core::arch::x86_64::__cpuid_count(leaf, subleaf) };
    (result.eax, result.ebx, result.ecx, result.edx)
}

#[cfg(target_arch = "x86_64")]
fn cpuid_brand_string() -> String {
    let mut brand = [0u8; 48];
    for i in 0..3u32 {
        let (eax, ebx, ecx, edx) = cpuid(0x8000_0002 + i, 0);
        let offset = (i * 16) as usize;
        brand[offset..offset+4].copy_from_slice(&eax.to_le_bytes());
        brand[offset+4..offset+8].copy_from_slice(&ebx.to_le_bytes());
        brand[offset+8..offset+12].copy_from_slice(&ecx.to_le_bytes());
        brand[offset+12..offset+16].copy_from_slice(&edx.to_le_bytes());
    }
    let s = core::str::from_utf8(&brand).unwrap_or("?").trim_end_matches('\0').trim();
    String::from(s)
}

#[cfg(target_arch = "x86_64")]
fn dump_vendor_brand() {
    let (max_leaf, ebx, ecx, edx) = cpuid(0, 0);
    let mut vendor = [0u8; 12];
    vendor[0..4].copy_from_slice(&ebx.to_le_bytes());
    vendor[4..8].copy_from_slice(&edx.to_le_bytes());
    vendor[8..12].copy_from_slice(&ecx.to_le_bytes());
    let vendor_str = core::str::from_utf8(&vendor).unwrap_or("?");

    let (ext_max, _, _, _) = cpuid(0x8000_0000, 0);
    let brand = if ext_max >= 0x8000_0004 {
        cpuid_brand_string()
    } else {
        String::from("<unavailable>")
    };

    let (eax1, _, _, _) = cpuid(1, 0);
    let stepping = eax1 & 0xF;
    let model = ((eax1 >> 4) & 0xF) | (((eax1 >> 16) & 0xF) << 4);
    let family = ((eax1 >> 8) & 0xF) + ((eax1 >> 20) & 0xFF);

    dbg_out!("[CPU] Vendor:   {}", vendor_str);
    dbg_out!("[CPU] Brand:    {}", brand);
    dbg_out!("[CPU] Family: {} Model: {} Stepping: {}", family, model, stepping);
    dbg_out!("[CPU] Max CPUID leaf: 0x{:X}  Max extended: 0x{:X}", max_leaf, ext_max);
}

#[cfg(target_arch = "x86_64")]
fn dump_topology() {
    let (_, ebx1, _, _) = cpuid(1, 0);
    let max_logical = (ebx1 >> 16) & 0xFF;
    let initial_apic = (ebx1 >> 24) & 0xFF;
    dbg_out!("[CPU] Initial APIC ID: {}  Max logical processors: {}", initial_apic, max_logical);

    // Extended topology (leaf 0xB)
    let (max_leaf, _, _, _) = cpuid(0, 0);
    if max_leaf >= 0xB {
        dbg_out!("[CPU] Extended Topology (CPUID.0Bh):");
        for level in 0..4u32 {
            let (eax, ebx, ecx, _edx) = cpuid(0xB, level);
            let shift = eax & 0x1F;
            let num_procs = ebx & 0xFFFF;
            let level_type = (ecx >> 8) & 0xFF;
            if level_type == 0 && level > 0 { break; }
            let type_str = match level_type {
                0 => "Invalid",
                1 => "SMT (threads)",
                2 => "Core",
                _ => "Unknown",
            };
            if num_procs > 0 {
                dbg_out!("[CPU]   Level {}: type={} ({})  processors={}, shift={}",
                    level, level_type, type_str, num_procs, shift);
            }
        }
    }

    // Try leaf 0x1F (V2 Extended Topology) for Intel Hybrid
    if max_leaf >= 0x1F {
        let (eax, ebx, ecx, _) = cpuid(0x1F, 0);
        if (ecx >> 8) & 0xFF != 0 {
            dbg_out!("[CPU] V2 Extended Topology (CPUID.1Fh) — Hybrid core info available");
        }
    }
}

#[cfg(target_arch = "x86_64")]
fn dump_features() {
    let (_, _, ecx1, edx1) = cpuid(1, 0);
    let (_, ebx7, ecx7, edx7) = cpuid(7, 0);
    let (_, _, _, edx81) = cpuid(0x8000_0001, 0);

    dbg_out!("[CPU] Feature Flags:");

    // Basic features (ECX of leaf 1)
    let ecx1_features = [
        (0, "SSE3"), (1, "PCLMULQDQ"), (5, "VMX"), (9, "SSSE3"),
        (12, "FMA"), (19, "SSE4.1"), (20, "SSE4.2"), (23, "POPCNT"),
        (25, "AES-NI"), (26, "XSAVE"), (27, "OSXSAVE"), (28, "AVX"),
        (29, "F16C"), (30, "RDRAND"),
    ];
    let mut feat_line = String::from("  ECX.1: ");
    for (bit, name) in &ecx1_features {
        if ecx1 & (1 << bit) != 0 {
            feat_line.push_str(name);
            feat_line.push(' ');
        }
    }
    dbg_out!("[CPU] {}", feat_line);

    // EDX of leaf 1
    let edx1_features = [
        (0, "FPU"), (4, "TSC"), (5, "MSR"), (6, "PAE"), (8, "CX8"),
        (9, "APIC"), (15, "CMOV"), (19, "CLFLUSH"), (23, "MMX"),
        (24, "FXSR"), (25, "SSE"), (26, "SSE2"), (28, "HTT"),
    ];
    let mut feat_line = String::from("  EDX.1: ");
    for (bit, name) in &edx1_features {
        if edx1 & (1 << bit) != 0 {
            feat_line.push_str(name);
            feat_line.push(' ');
        }
    }
    dbg_out!("[CPU] {}", feat_line);

    // Structured extended features (leaf 7)
    let ebx7_features = [
        (0, "FSGSBASE"), (2, "SGX"), (3, "BMI1"), (4, "HLE"), (5, "AVX2"),
        (7, "SMEP"), (8, "BMI2"), (10, "INVPCID"), (11, "RTM"),
        (16, "AVX512F"), (18, "RDSEED"), (19, "ADX"), (20, "SMAP"),
        (26, "AVX512PF"), (27, "AVX512ER"), (28, "AVX512CD"),
        (29, "SHA"), (30, "AVX512BW"), (31, "AVX512VL"),
    ];
    let mut feat_line = String::from("  EBX.7: ");
    for (bit, name) in &ebx7_features {
        if ebx7 & (1 << bit) != 0 {
            feat_line.push_str(name);
            feat_line.push(' ');
        }
    }
    dbg_out!("[CPU] {}", feat_line);

    // ECX/EDX of leaf 7
    let ecx7_features = [
        (1, "AVX512_VBMI"), (8, "GFNI"), (9, "VAES"), (10, "VPCLMULQDQ"),
    ];
    let edx7_features = [
        (2, "AVX512_4VNNIW"), (3, "AVX512_4FMAPS"),
        (10, "MD_CLEAR"), (18, "PCONFIG"), (20, "CET_IBT"),
        (26, "IBRS/IBPB"), (27, "STIBP"), (28, "L1D_FLUSH"),
        (29, "IA32_ARCH_CAPS"), (31, "SSBD"),
    ];
    let mut feat_line = String::from("  ECX.7: ");
    for (bit, name) in &ecx7_features {
        if ecx7 & (1 << bit) != 0 { feat_line.push_str(name); feat_line.push(' '); }
    }
    dbg_out!("[CPU] {}", feat_line);
    let mut feat_line = String::from("  EDX.7: ");
    for (bit, name) in &edx7_features {
        if edx7 & (1 << bit) != 0 { feat_line.push_str(name); feat_line.push(' '); }
    }
    dbg_out!("[CPU] {}", feat_line);

    // Extended features (0x80000001)
    let ext_features = [
        (11, "SYSCALL"), (20, "NX"), (26, "1GB-pages"), (27, "RDTSCP"), (29, "Long-Mode"),
    ];
    let mut feat_line = String::from("  Ext:   ");
    for (bit, name) in &ext_features {
        if edx81 & (1 << bit) != 0 { feat_line.push_str(name); feat_line.push(' '); }
    }
    dbg_out!("[CPU] {}", feat_line);
}

#[cfg(target_arch = "x86_64")]
fn dump_cache_info() {
    let (max_leaf, _, _, _) = cpuid(0, 0);
    if max_leaf < 4 { return; }

    dbg_out!("[CPU] Cache Hierarchy (CPUID.4h):");
    for index in 0..16u32 {
        let (eax, ebx, ecx, _) = cpuid(4, index);
        let cache_type = eax & 0x1F;
        if cache_type == 0 { break; }

        let level = (eax >> 5) & 0x7;
        let sets = ecx + 1;
        let line_size = (ebx & 0xFFF) + 1;
        let partitions = ((ebx >> 12) & 0x3FF) + 1;
        let ways = ((ebx >> 22) & 0x3FF) + 1;
        let size_kb = (ways * partitions * line_size * sets) / 1024;

        let type_str = match cache_type {
            1 => "Data",
            2 => "Instruction",
            3 => "Unified",
            _ => "?",
        };

        dbg_out!("[CPU]   L{} {} : {} KB, {}-way, {}-byte line, {} sets",
            level, type_str, size_kb, ways, line_size, sets);
    }
}

#[cfg(target_arch = "x86_64")]
fn dump_tlb_info() {
    let (ext_max, _, _, _) = cpuid(0x8000_0000, 0);
    if ext_max < 0x8000_0005 { return; }

    // AMD-style TLB info (leaf 0x80000005 / 0x80000006)
    let (eax5, ebx5, ecx5, edx5) = cpuid(0x8000_0005, 0);
    if eax5 != 0 || ebx5 != 0 {
        dbg_out!("[CPU] TLB Info (AMD-style, CPUID.80000005h):");
        // L1 data TLB
        let dtlb_2m_entries = (eax5 >> 16) & 0xFF;
        let dtlb_2m_assoc = (eax5 >> 24) & 0xFF;
        if dtlb_2m_entries > 0 {
            dbg_out!("[CPU]   L1 DTLB 2M/4M: {} entries, {}-way", dtlb_2m_entries, dtlb_2m_assoc);
        }
        let dtlb_4k_entries = (ebx5 >> 16) & 0xFF;
        let dtlb_4k_assoc = (ebx5 >> 24) & 0xFF;
        if dtlb_4k_entries > 0 {
            dbg_out!("[CPU]   L1 DTLB 4K: {} entries, {}-way", dtlb_4k_entries, dtlb_4k_assoc);
        }
    }

    if ext_max >= 0x8000_0006 {
        let (_, _, ecx6, _) = cpuid(0x8000_0006, 0);
        let l2_size = (ecx6 >> 16) & 0xFFFF;
        let l2_assoc = (ecx6 >> 12) & 0xF;
        let l2_line = ecx6 & 0xFF;
        if l2_size > 0 {
            dbg_out!("[CPU]   L2 cache: {} KB, assoc={}, line={} bytes", l2_size, l2_assoc, l2_line);
        }
    }
}

#[cfg(target_arch = "x86_64")]
fn dump_frequency() {
    let (max_leaf, _, _, _) = cpuid(0, 0);

    // Try leaf 0x15: TSC/core crystal clock
    if max_leaf >= 0x15 {
        let (eax, ebx, ecx, _) = cpuid(0x15, 0);
        if eax != 0 && ebx != 0 {
            let crystal_hz = if ecx != 0 { ecx as u64 } else { 0 };
            dbg_out!("[CPU] TSC Ratio: {} / {} (crystal: {} Hz)", ebx, eax,
                if crystal_hz > 0 { alloc::format!("{}", crystal_hz) } else { String::from("unknown") });
            if crystal_hz > 0 && eax > 0 {
                let tsc_freq = crystal_hz * ebx as u64 / eax as u64;
                dbg_out!("[CPU] TSC Frequency: {} MHz", tsc_freq / 1_000_000);
            }
        }
    }

    // Try leaf 0x16: Processor frequency info
    if max_leaf >= 0x16 {
        let (eax, ebx, ecx, _) = cpuid(0x16, 0);
        let base_mhz = eax & 0xFFFF;
        let max_mhz = ebx & 0xFFFF;
        let bus_mhz = ecx & 0xFFFF;
        if base_mhz > 0 {
            dbg_out!("[CPU] Frequency: base={} MHz, max={} MHz, bus={} MHz", base_mhz, max_mhz, bus_mhz);
        }
    }

    // Check invariant TSC
    let (ext_max, _, _, _) = cpuid(0x8000_0000, 0);
    if ext_max >= 0x8000_0007 {
        let (_, _, _, edx) = cpuid(0x8000_0007, 0);
        let invariant_tsc = edx & (1 << 8) != 0;
        dbg_out!("[CPU] Invariant TSC: {}", if invariant_tsc { "YES (reliable)" } else { "NO (may drift)" });
    }

    // Measure TSC by busy-loop (rough estimate if no CPUID frequency)
    dbg_out!("[CPU] TSC live measurement...");
    let start: u64;
    unsafe { core::arch::asm!("rdtsc", out("eax") _, out("edx") _, options(nostack)); }
    let t0: u64;
    unsafe {
        let lo: u32;
        let hi: u32;
        core::arch::asm!("rdtsc", out("eax") lo, out("edx") hi, options(nostack));
        t0 = ((hi as u64) << 32) | lo as u64;
    }
    // ~10ms busy wait using PIT or simple loop
    for _ in 0..10_000_000u64 {
        unsafe { core::arch::asm!("nop", options(nostack)); }
    }
    let t1: u64;
    unsafe {
        let lo: u32;
        let hi: u32;
        core::arch::asm!("rdtsc", out("eax") lo, out("edx") hi, options(nostack));
        t1 = ((hi as u64) << 32) | lo as u64;
    }
    let delta = t1.saturating_sub(t0);
    dbg_out!("[CPU] TSC delta ({} cycles over ~10M NOPs)", delta);
}

#[cfg(target_arch = "x86_64")]
fn dump_microcode() {
    // IA32_BIOS_SIGN_ID (MSR 0x8B): microcode revision
    if let Some(val) = crate::debug::read_msr_safe(0x8B) {
        let ucode_rev = (val >> 32) as u32;
        dbg_out!("[CPU] Microcode revision: 0x{:08X}", ucode_rev);
    } else {
        dbg_out!("[CPU] Microcode revision: <MSR 0x8B read failed>");
    }

    // IA32_PLATFORM_ID (MSR 0x17)
    if let Some(val) = crate::debug::read_msr_safe(0x17) {
        let platform_id = (val >> 50) & 0x7;
        dbg_out!("[CPU] Platform ID: {}", platform_id);
    }

    // PAT (MSR 0x277)
    if let Some(val) = crate::debug::read_msr_safe(0x277) {
        dbg_out!("[CPU] PAT: 0x{:016X}", val);
    }
}

#[cfg(target_arch = "x86_64")]
fn dump_security_mitigations() {
    dbg_out!("[CPU] Security Mitigations:");

    // Check IA32_ARCH_CAPABILITIES (MSR 0x10A) if supported
    let (_, _, _, edx7) = cpuid(7, 0);
    if edx7 & (1 << 29) != 0 {
        if let Some(val) = crate::debug::read_msr_safe(0x10A) {
            let rdcl_no = val & 1 != 0;          // Not vulnerable to Meltdown
            let ibrs_all = val & (1 << 1) != 0;  // IBRS enhanced
            let rsba = val & (1 << 2) != 0;      // RSB Alternate
            let ssb_no = val & (1 << 4) != 0;    // Not vulnerable to SSB
            let mds_no = val & (1 << 5) != 0;    // Not vulnerable to MDS
            let taa_no = val & (1 << 8) != 0;    // Not vulnerable to TAA

            dbg_out!("[CPU]   IA32_ARCH_CAPABILITIES: 0x{:016X}", val);
            dbg_out!("[CPU]     Meltdown (RDCL):    {}", if rdcl_no { "NOT VULNERABLE" } else { "VULNERABLE or mitigated" });
            dbg_out!("[CPU]     IBRS enhanced:       {}", ibrs_all);
            dbg_out!("[CPU]     RSB Alternate:       {}", rsba);
            dbg_out!("[CPU]     Spectre SSB:         {}", if ssb_no { "NOT VULNERABLE" } else { "VULNERABLE or mitigated" });
            dbg_out!("[CPU]     MDS:                 {}", if mds_no { "NOT VULNERABLE" } else { "VULNERABLE or mitigated" });
            dbg_out!("[CPU]     TAA:                 {}", if taa_no { "NOT VULNERABLE" } else { "VULNERABLE or mitigated" });
        }
    } else {
        dbg_out!("[CPU]   IA32_ARCH_CAPABILITIES: not supported (older CPU)");
    }

    // SPEC_CTRL (MSR 0x48)
    if edx7 & (1 << 26) != 0 { // IBRS/IBPB
        if let Some(val) = crate::debug::read_msr_safe(0x48) {
            dbg_out!("[CPU]   IA32_SPEC_CTRL: 0x{:016X} (IBRS={}, STIBP={}, SSBD={})",
                val, val & 1, (val >> 1) & 1, (val >> 2) & 1);
        }
    }

    // Check SMEP, SMAP, UMIP via CR4
    let cr4: u64;
    unsafe { core::arch::asm!("mov {}, cr4", out(reg) cr4, options(nostack)); }
    dbg_out!("[CPU]   CR4: 0x{:016X}", cr4);
    dbg_out!("[CPU]     SMEP (bit 20): {}",  if cr4 & (1 << 20) != 0 { "ENABLED" } else { "disabled" });
    dbg_out!("[CPU]     SMAP (bit 21): {}",  if cr4 & (1 << 21) != 0 { "ENABLED" } else { "disabled" });
    dbg_out!("[CPU]     UMIP (bit 11): {}",  if cr4 & (1 << 11) != 0 { "ENABLED" } else { "disabled" });

    // EFER MSR
    if let Some(efer) = crate::debug::read_msr_safe(0xC000_0080) {
        dbg_out!("[CPU]   IA32_EFER: 0x{:016X} (SCE={}, LME={}, LMA={}, NXE={})",
            efer, efer & 1, (efer >> 8) & 1, (efer >> 10) & 1, (efer >> 11) & 1);
    }
}

#[cfg(target_arch = "x86_64")]
fn dump_power_management() {
    let (ext_max, _, _, _) = cpuid(0x8000_0000, 0);
    if ext_max >= 0x8000_0007 {
        let (eax, _, _, edx) = cpuid(0x8000_0007, 0);
        dbg_out!("[CPU] Power Management (CPUID.80000007h):");
        let features = [
            (0, "TS (temp sensor)"), (1, "FID (freq ID)"), (2, "VID (voltage ID)"),
            (3, "TTP"), (4, "TM (thermal monitor)"), (6, "100MHz steps"),
            (7, "HwPstate"), (8, "InvariantTSC"), (9, "CPB (core perf boost)"),
        ];
        for (bit, name) in &features {
            if edx & (1 << bit) != 0 {
                dbg_out!("[CPU]   {}", name);
            }
        }
    }

    // Check if CPU supports MPERF/APERF (for actual vs. effective frequency)
    let (_, _, ecx1, _) = cpuid(1, 0);
    if ecx1 & (1 << 31) != 0 {
        // Hypervisor present — skip MPERF/APERF, might not be real
        dbg_out!("[CPU] Hypervisor detected — MPERF/APERF may be virtualized");
    }
}

#[cfg(target_arch = "x86_64")]
fn dump_all_cpuid_leaves() {
    dbg_out!("[CPU] === Full CPUID Dump (verbose) ===");
    let (max_leaf, _, _, _) = cpuid(0, 0);
    for leaf in 0..=max_leaf.min(0x20) {
        let (eax, ebx, ecx, edx) = cpuid(leaf, 0);
        dbg_out!("[CPU] CPUID({:08X},0) = EAX:{:08X} EBX:{:08X} ECX:{:08X} EDX:{:08X}",
            leaf, eax, ebx, ecx, edx);
    }
    let (ext_max, _, _, _) = cpuid(0x8000_0000, 0);
    for leaf in 0x8000_0000..=ext_max.min(0x8000_0020) {
        let (eax, ebx, ecx, edx) = cpuid(leaf, 0);
        dbg_out!("[CPU] CPUID({:08X},0) = EAX:{:08X} EBX:{:08X} ECX:{:08X} EDX:{:08X}",
            leaf, eax, ebx, ecx, edx);
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// aarch64 diagnostics
// ═══════════════════════════════════════════════════════════════════════════════
#[cfg(target_arch = "aarch64")]
fn dump_aarch64_info() {
    dbg_out!("[CPU] AArch64 CPU identification:");

    let midr: u64;
    unsafe { core::arch::asm!("mrs {}, MIDR_EL1", out(reg) midr); }
    let implementer = (midr >> 24) & 0xFF;
    let variant = (midr >> 20) & 0xF;
    let arch = (midr >> 16) & 0xF;
    let part = (midr >> 4) & 0xFFF;
    let rev = midr & 0xF;

    let impl_name = match implementer {
        0x41 => "ARM",
        0x42 => "Broadcom",
        0x43 => "Cavium",
        0x44 => "DEC",
        0x4E => "NVIDIA",
        0x50 => "APM",
        0x51 => "Qualcomm",
        0x53 => "Samsung",
        0x56 => "Marvell",
        0x61 => "Apple",
        0x69 => "Intel",
        0xC0 => "Ampere",
        _ => "Unknown",
    };

    dbg_out!("[CPU] MIDR_EL1: 0x{:016X}", midr);
    dbg_out!("[CPU] Implementer: 0x{:02X} ({})", implementer, impl_name);
    dbg_out!("[CPU] Variant: {}  Architecture: {}  Part: 0x{:03X}  Revision: {}",
        variant, arch, part, rev);

    // ID_AA64ISAR0_EL1 — instruction set features
    let isar0: u64;
    unsafe { core::arch::asm!("mrs {}, ID_AA64ISAR0_EL1", out(reg) isar0); }
    dbg_out!("[CPU] ID_AA64ISAR0_EL1: 0x{:016X}", isar0);
    let aes = (isar0 >> 4) & 0xF;
    let sha1 = (isar0 >> 8) & 0xF;
    let sha2 = (isar0 >> 12) & 0xF;
    let crc32 = (isar0 >> 16) & 0xF;
    let atomic = (isar0 >> 20) & 0xF;
    let rdm = (isar0 >> 28) & 0xF;
    dbg_out!("[CPU]   AES:{} SHA1:{} SHA2:{} CRC32:{} Atomic:{} RDM:{}",
        aes, sha1, sha2, crc32, atomic, rdm);

    // ID_AA64MMFR0_EL1 — memory model features
    let mmfr0: u64;
    unsafe { core::arch::asm!("mrs {}, ID_AA64MMFR0_EL1", out(reg) mmfr0); }
    let pa_range = mmfr0 & 0xF;
    let pa_bits = match pa_range {
        0 => 32, 1 => 36, 2 => 40, 3 => 42, 4 => 44, 5 => 48, 6 => 52, _ => 0,
    };
    dbg_out!("[CPU] Physical address bits: {}", pa_bits);

    // Current exception level
    let current_el: u64;
    unsafe { core::arch::asm!("mrs {}, CurrentEL", out(reg) current_el); }
    let el = (current_el >> 2) & 0x3;
    dbg_out!("[CPU] Current Exception Level: EL{}", el);
}
