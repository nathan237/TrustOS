//! CoreMark EEMBC benchmark integration with energy efficiency reporting.
//!
//! Compiles official CoreMark C sources via build.rs and links them into the kernel.
//! Provides FFI glue: TSC timer, serial output, and memory stubs.
//! After the benchmark, reads Intel RAPL MSRs and DTS thermal sensors
//! to produce a detailed energy efficiency report.

extern "C" {
    fn main();
}

const ITERATIONS: u64 = 1_500_000;

// ── RAPL / Thermal MSR addresses ──

const MSR_RAPL_POWER_UNIT: u32 = 0x606;
const MSR_PKG_ENERGY_STATUS: u32 = 0x611;
const MSR_PP0_ENERGY_STATUS: u32 = 0x639;
const MSR_PP1_ENERGY_STATUS: u32 = 0x641;
const MSR_DRAM_ENERGY_STATUS: u32 = 0x619;
const MSR_PKG_POWER_INFO: u32 = 0x614;
const MSR_THERM_STATUS: u32 = 0x19C;
const MSR_TEMPERATURE_TARGET: u32 = 0x1A2;
const MSR_PKG_THERM_STATUS: u32 = 0x1B1;
const MSR_PLATFORM_INFO: u32 = 0xCE;
const MSR_PERF_STATUS: u32 = 0x198;

struct RaplSnapshot {
    pkg: u64,
    pp0: u64,
    pp1: u64,
    dram: u64,
}

struct PreBenchState {
    tsc: u64,
    rapl: RaplSnapshot,
    energy_unit_shift: u32,
    power_unit_shift: u32,
    core_temp: i32,
    pkg_temp: i32,
    tj_max: i32,
    freq_mhz: u64,
    vcore_mv: u64,
    tdp_mw: u64,
}

fn read_msr(addr: u32) -> u64 {
    crate::debug::read_msr_safe(addr).unwrap_or(0)
}

fn read_rapl_snapshot() -> RaplSnapshot {
    RaplSnapshot {
        pkg: read_msr(MSR_PKG_ENERGY_STATUS) & 0xFFFF_FFFF,
        pp0: read_msr(MSR_PP0_ENERGY_STATUS) & 0xFFFF_FFFF,
        pp1: read_msr(MSR_PP1_ENERGY_STATUS) & 0xFFFF_FFFF,
        dram: read_msr(MSR_DRAM_ENERGY_STATUS) & 0xFFFF_FFFF,
    }
}

fn rapl_delta(before: u64, after: u64) -> u64 {
    if after >= before {
        after - before
    } else {
        (0xFFFF_FFFF - before) + after + 1
    }
}

fn read_temps() -> (i32, i32, i32) {
    let tj_max = crate::debug::read_msr_safe(MSR_TEMPERATURE_TARGET)
        .map(|v| ((v >> 16) & 0xFF) as i32)
        .unwrap_or(100);

    let core_temp = crate::debug::read_msr_safe(MSR_THERM_STATUS)
        .and_then(|v| {
            let s = v as u32;
            if s & (1 << 31) != 0 {
                Some(tj_max - ((s >> 16) & 0x7F) as i32)
            } else {
                None
            }
        })
        .unwrap_or(0);

    let pkg_temp = crate::debug::read_msr_safe(MSR_PKG_THERM_STATUS)
        .map(|v| tj_max - (((v as u32) >> 16) & 0x7F) as i32)
        .unwrap_or(0);

    (core_temp, pkg_temp, tj_max)
}

fn read_freq_mhz() -> u64 {
    crate::debug::read_msr_safe(MSR_PLATFORM_INFO)
        .map(|v| ((v >> 8) & 0xFF) * 100)
        .unwrap_or(3300)
}

fn read_vcore_mv() -> u64 {
    crate::debug::read_msr_safe(MSR_PERF_STATUS)
        .map(|v| {
            let vid = (v >> 32) & 0xFFFF;
            if vid > 0 { (vid * 1000) / 8192 } else { 0 }
        })
        .unwrap_or(0)
}

fn read_tdp_mw(power_unit_shift: u32) -> u64 {
    crate::debug::read_msr_safe(MSR_PKG_POWER_INFO)
        .map(|v| {
            let raw = (v & 0x7FFF) as u64;
            (raw * 1000) >> power_unit_shift
        })
        .unwrap_or(0)
}

fn capture_pre_state() -> PreBenchState {
    let rapl_unit = read_msr(MSR_RAPL_POWER_UNIT);
    let energy_unit_shift = ((rapl_unit >> 8) & 0x1F) as u32;
    let power_unit_shift = (rapl_unit & 0xF) as u32;
    let (core_temp, pkg_temp, tj_max) = read_temps();
    let freq_mhz = read_freq_mhz();
    let vcore_mv = read_vcore_mv();
    let tdp_mw = read_tdp_mw(power_unit_shift);
    let rapl = read_rapl_snapshot();
    let tsc = crate::cpu::tsc::read_tsc_serialized();

    PreBenchState {
        tsc,
        rapl,
        energy_unit_shift,
        power_unit_shift,
        core_temp,
        pkg_temp,
        tj_max,
        freq_mhz,
        vcore_mv,
        tdp_mw,
    }
}

/// Fixed-point division: returns (integer, 2-digit fractional part)
fn div2(num: u64, den: u64) -> (u64, u64) {
    if den == 0 { return (0, 0); }
    let int = num / den;
    let frac = (num % den) * 100 / den;
    (int, frac)
}

/// Fixed-point division: returns (integer, 3-digit fractional part)
fn div3(num: u64, den: u64) -> (u64, u64) {
    if den == 0 { return (0, 0); }
    let int = num / den;
    let frac = (num % den) * 1000 / den;
    (int, frac)
}

fn out(s: &str) {
    crate::println!("{}", s);
    crate::serial_println!("{}", s);
}

macro_rules! report {
    ($($arg:tt)*) => {{
        let s = alloc::format!($($arg)*);
        out(&s);
    }};
}

fn print_report(pre: &PreBenchState, tsc_end: u64) {
    let tsc_freq = crate::cpu::tsc::frequency_hz();
    if tsc_freq == 0 { return; }
    let tsc_delta = tsc_end.wrapping_sub(pre.tsc);

    // ── Timing ──
    let (rt_s, rt_cs) = div2(tsc_delta, tsc_freq / 100);
    let runtime_ms = tsc_delta / (tsc_freq / 1000);

    // ── Precise score ──
    let score = ITERATIONS * tsc_freq / tsc_delta;
    let score_frac = (ITERATIONS * tsc_freq % tsc_delta) * 100 / tsc_delta;
    let (cm_mhz, cm_mhz_f) = div2(score * 1000 + score_frac * 10, pre.freq_mhz);

    // ── Post-benchmark thermal ──
    let (core_after, pkg_after, _) = read_temps();
    let vcore_after = read_vcore_mv();

    // ── RAPL energy deltas (millijoules) ──
    let rapl_after = read_rapl_snapshot();
    let eshift = pre.energy_unit_shift;
    let pkg_delta = rapl_delta(pre.rapl.pkg, rapl_after.pkg);
    let pp0_delta = rapl_delta(pre.rapl.pp0, rapl_after.pp0);
    let pp1_delta = rapl_delta(pre.rapl.pp1, rapl_after.pp1);
    let dram_delta = rapl_delta(pre.rapl.dram, rapl_after.dram);

    let pkg_mj = (pkg_delta * 1000) >> eshift;
    let pp0_mj = (pp0_delta * 1000) >> eshift;
    let pp1_mj = (pp1_delta * 1000) >> eshift;
    let dram_mj = (dram_delta * 1000) >> eshift;

    // Joules with 3 decimals
    let (pkg_j, pkg_jf) = div3(pkg_mj, 1000);
    let (pp0_j, pp0_jf) = div3(pp0_mj, 1000);
    let (pp1_j, pp1_jf) = div3(pp1_mj, 1000);
    let (dram_j, dram_jf) = div3(dram_mj, 1000);

    // Average power (milliwatts)
    let avg_pkg_mw = if runtime_ms > 0 { pkg_mj * 1000 / runtime_ms } else { 0 };
    let avg_pp0_mw = if runtime_ms > 0 { pp0_mj * 1000 / runtime_ms } else { 0 };
    let (avg_pkg_w, avg_pkg_wf) = div3(avg_pkg_mw, 1000);
    let (avg_pp0_w, avg_pp0_wf) = div3(avg_pp0_mw, 1000);

    // TDP utilization %
    let tdp_pct = if pre.tdp_mw > 0 { avg_pkg_mw * 100 / pre.tdp_mw } else { 0 };

    // ── Efficiency metrics ──
    // CoreMark/Watt = score / avg_power_watts
    let cm_per_watt = if avg_pkg_mw > 0 { score * 1000 / avg_pkg_mw } else { 0 };
    let (cm_mhz_w, cm_mhz_wf) = if avg_pkg_mw > 0 {
        div2(score * 1_000_000, pre.freq_mhz * avg_pkg_mw)
    } else {
        (0, 0)
    };

    // uJ per iteration = pkg_energy_uj / iterations
    let pkg_uj = (pkg_delta * 1_000_000) >> eshift;
    let uj_per_iter = if ITERATIONS > 0 { pkg_uj / ITERATIONS } else { 0 };

    // Iterations per millijoule
    let iter_per_mj = if pkg_mj > 0 { ITERATIONS / pkg_mj } else { 0 };

    // Thermal delta
    let core_delta = core_after - pre.core_temp;
    let pkg_delta_t = pkg_after - pre.pkg_temp;

    // ── Print report ──
    out("");
    out("================================================================");
    out("   TrustOS CoreMark Energy Efficiency Report");
    out("================================================================");

    out("");
    out("--- Hardware -------------------------------------------------");
    report!("  CPU               : Intel Pentium G4400 (Skylake)");
    report!("  Frequency (spec)  : {} MHz", pre.freq_mhz);
    report!("  TSC Frequency     : {} MHz", tsc_freq / 1_000_000);
    report!("  TjMax             : {}*C", pre.tj_max);
    report!("  TDP               : {}.{} W", pre.tdp_mw / 1000, pre.tdp_mw % 1000);
    if pre.vcore_mv > 0 {
        report!("  VCore (start)     : {} mV", pre.vcore_mv);
    }
    if vcore_after > 0 {
        report!("  VCore (end)       : {} mV", vcore_after);
    }

    out("");
    out("--- CoreMark Score -------------------------------------------");
    report!("  Iterations        : {}", ITERATIONS);
    report!("  Runtime (TSC)     : {}.{:02} s", rt_s, rt_cs);
    report!("  Score (precise)   : {}.{:02} iterations/sec", score, score_frac);
    report!("  CoreMark/MHz      : {}.{:02}", cm_mhz, cm_mhz_f);

    out("");
    out("--- Thermal --------------------------------------------------");
    report!("  Core Temp Before  : {}*C", pre.core_temp);
    report!("  Core Temp After   : {}*C", core_after);
    report!("  Core Temp Rise    : {}{}*C", if core_delta >= 0 { "+" } else { "" }, core_delta);
    report!("  Pkg Temp Before   : {}*C", pre.pkg_temp);
    report!("  Pkg Temp After    : {}*C", pkg_after);
    report!("  Pkg Temp Rise     : {}{}*C", if pkg_delta_t >= 0 { "+" } else { "" }, pkg_delta_t);

    out("");
    out("--- Energy Consumption (Intel RAPL) --------------------------");
    report!("  Package (total)   : {}.{:03} J", pkg_j, pkg_jf);
    report!("    Cores (PP0)     : {}.{:03} J", pp0_j, pp0_jf);
    report!("    iGPU  (PP1)     : {}.{:03} J", pp1_j, pp1_jf);
    report!("    DRAM            : {}.{:03} J", dram_j, dram_jf);
    report!("  Avg Power (Pkg)   : {}.{:03} W", avg_pkg_w, avg_pkg_wf);
    report!("  Avg Power (Cores) : {}.{:03} W", avg_pp0_w, avg_pp0_wf);
    report!("  TDP Utilization   : {}%", tdp_pct);

    out("");
    out("--- Efficiency Metrics ---------------------------------------");
    report!("  CoreMark/Watt     : {}", cm_per_watt);
    report!("  CoreMark/MHz/Watt : {}.{:02}", cm_mhz_w, cm_mhz_wf);
    report!("  uJ/iteration      : {}", uj_per_iter);
    report!("  Iterations/mJ     : {}", iter_per_mj);

    out("");
    out("--- Comparison Reference (single core, ~same class) ---------");
    out("  Platform               CM/MHz   CM/W    uJ/iter");
    out("  TrustOS (bare metal)   ^^^^     ^^^^    ^^^^");
    out("  Linux (Skylake, -O2)   ~4.5     ~500    ~2000");
    out("  RTOS (bare metal)      ~5-8     ~600    ~1600");
    out("  ARM Cortex-A72 Linux   ~5.0     ~2500   ~400");

    out("");
    out("--- Submission Info (EEMBC) ----------------------------------");
    out("  Compiler          : Clang 22.1.3 (LLVM)");
    report!("  Flags             : -O2 -ffreestanding -nostdlib -fno-builtin -mcmodel=large -mno-red-zone -mno-sse");
    out("  OS                : (bare metal)");
    out("  Memory            : STATIC");
    out("  Parallel          : 1 (single thread)");
    out("================================================================");
    out("");
}

/// Entry point — called from kmain when feature "coremark" is active.
pub fn run() -> ! {
    crate::println!("");
    crate::println!("========================================");
    crate::println!("  CoreMark 1.0 - TrustOS Edition");
    crate::println!("========================================");

    let freq = crate::cpu::tsc_frequency();
    crate::println!("[CoreMark] TSC frequency: {} MHz", freq / 1_000_000);
    crate::println!("[CoreMark] Iterations: {} (~60s runtime)", ITERATIONS);
    crate::println!("[CoreMark] Running benchmark...");
    crate::println!("[CoreMark] Please wait ~60 seconds...");
    crate::serial_println!("[CoreMark] TSC={} MHz, ITER={}, calling main()", freq / 1_000_000, ITERATIONS);

    let pre = capture_pre_state();

    unsafe { main(); }

    let tsc_end = crate::cpu::tsc::read_tsc_serialized();

    // Display CoreMark's own output
    crate::println!("");
    crate::println!("========== COREMARK RAW OUTPUT ==========");
    let guard = OUTPUT_BUF.lock();
    let (ref buf, pos) = *guard;
    if let Ok(s) = core::str::from_utf8(&buf[..pos]) {
        for line in s.lines() {
            crate::println!("{}", line);
            crate::serial_println!("{}", line);
        }
    }
    drop(guard);
    crate::println!("=========================================");

    print_report(&pre, tsc_end);

    crate::println!("[CoreMark] Done.");
    loop { core::hint::spin_loop(); }
}

// ── Static buffer for capturing all ee_printf output ──

const OUTPUT_BUF_SIZE: usize = 8192;
static OUTPUT_BUF: spin::Mutex<([u8; OUTPUT_BUF_SIZE], usize)> =
    spin::Mutex::new(([0u8; OUTPUT_BUF_SIZE], 0));

// ── FFI exports called by CoreMark C code ──

#[no_mangle]
pub extern "C" fn trustos_read_tsc() -> u64 {
    crate::cpu::tsc::read_tsc_serialized()
}

#[no_mangle]
pub extern "C" fn trustos_tsc_freq() -> u64 {
    let freq = crate::cpu::tsc::frequency_hz();
    if freq == 0 { 3_300_000_000 } else { freq }
}

#[no_mangle]
pub extern "C" fn trustos_serial_putchar(c: u8) {
    crate::arch::serial::write_byte(c);
    let mut guard = OUTPUT_BUF.lock();
    let (ref mut buf, ref mut pos) = *guard;
    if *pos < buf.len() {
        buf[*pos] = c;
        *pos += 1;
    }
}
