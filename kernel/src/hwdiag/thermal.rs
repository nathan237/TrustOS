//! Thermal & Power Diagnostics — CPU temperature, power states, fan control
//!
//! Reads thermal sensors via MSRs, ACPI, and embedded controller.

use alloc::string::String;
use alloc::format;
use alloc::vec::Vec;
use super::dbg_out;

/// Run thermal diagnostics
pub fn run() {
    dbg_out!("[THERM] === Thermal & Power Diagnostics ===");

    #[cfg(target_arch = "x86_64")]
    {
        read_cpu_thermal();
        read_power_info();
        read_battery_ec();
        read_thinkpad_ec();
    }

    #[cfg(not(target_arch = "x86_64"))]
    {
        dbg_out!("[THERM] Thermal monitoring via MSR only available on x86_64");
        dbg_out!("[THERM] (ACPI thermal zones would need additional parsing)");
    }
}

#[cfg(target_arch = "x86_64")]
fn read_cpu_thermal() {
    dbg_out!("[THERM] ─── CPU Thermal Sensors ───");

    // IA32_THERM_STATUS (MSR 0x19C)
    if let Some(therm_status) = crate::debug::read_msr_safe(0x19C) {
        let status = therm_status as u32;
        let thermal_status_flag = status & 1 != 0;
        let thermal_status_log = status & 2 != 0;
        let prochot = status & (1 << 2) != 0;
        let prochot_log = status & (1 << 3) != 0;
        let critical = status & (1 << 4) != 0;
        let reading_valid = status & (1 << 31) != 0;
        let digital_readout = (status >> 16) & 0x7F;

        dbg_out!("[THERM] IA32_THERM_STATUS (0x19C): 0x{:08X}", status);
        dbg_out!("[THERM]   Thermal status: {}  Log: {}", thermal_status_flag, thermal_status_log);
        dbg_out!("[THERM]   PROCHOT#: {}  Log: {}  Critical: {}", prochot, prochot_log, critical);

        if reading_valid {
            // Read TjMax from MSR 0x1A2 (IA32_TEMPERATURE_TARGET)
            let tj_max = if let Some(temp_target) = crate::debug::read_msr_safe(0x1A2) {
                ((temp_target >> 16) & 0xFF) as i32
            } else {
                100 // Default TjMax assumption
            };
            let current_temp = tj_max - digital_readout as i32;
            dbg_out!("[THERM]   TjMax: {}°C  Digital readout: {}  Current: ~{}°C",
                tj_max, digital_readout, current_temp);

            // Temperature assessment
            if current_temp > 90 {
                dbg_out!("[THERM]   ⚠ CRITICAL: CPU temperature very high!");
            } else if current_temp > 75 {
                dbg_out!("[THERM]   ⚠ WARNING: CPU temperature elevated");
            } else {
                dbg_out!("[THERM]   ✓ CPU temperature normal");
            }
        } else {
            dbg_out!("[THERM]   Digital readout not valid (reading_valid=false)");
        }
    } else {
        dbg_out!("[THERM]   IA32_THERM_STATUS: not available (non-Intel or restricted)");
    }

    // IA32_PACKAGE_THERM_STATUS (MSR 0x1B1)
    if let Some(pkg_therm) = crate::debug::read_msr_safe(0x1B1) {
        let status = pkg_therm as u32;
        let digital_readout = (status >> 16) & 0x7F;
        let prochot = status & (1 << 2) != 0;
        dbg_out!("[THERM] Package: PROCHOT#={} readout_delta={}", prochot, digital_readout);
    }

    // IA32_THERM_INTERRUPT (MSR 0x19B) — what's configured
    if let Some(therm_int) = crate::debug::read_msr_safe(0x19B) {
        let high_temp_int = therm_int & 1 != 0;
        let low_temp_int = therm_int & 2 != 0;
        let prochot_int = therm_int & (1 << 2) != 0;
        let threshold1 = (therm_int >> 8) & 0x7F;
        let threshold2 = (therm_int >> 16) & 0x7F;
        dbg_out!("[THERM]   Interrupts: high={} low={} prochot={} thresh1={} thresh2={}",
            high_temp_int, low_temp_int, prochot_int, threshold1, threshold2);
    }
}

#[cfg(target_arch = "x86_64")]
fn read_power_info() {
    dbg_out!("[THERM] ─── CPU Power/Frequency ───");

    // IA32_PERF_STATUS (MSR 0x198) — current P-state
    if let Some(perf_status) = crate::debug::read_msr_safe(0x198) {
        let ratio = (perf_status >> 8) & 0xFF;
        dbg_out!("[THERM] IA32_PERF_STATUS: current ratio = {} (x100 MHz = ~{} MHz)",
            ratio, ratio * 100);
    }

    // IA32_PERF_CTL (MSR 0x199) — target P-state
    if let Some(perf_ctl) = crate::debug::read_msr_safe(0x199) {
        let target_ratio = (perf_ctl >> 8) & 0xFF;
        let turbo_disable = perf_ctl & (1 << 32) != 0;
        dbg_out!("[THERM] IA32_PERF_CTL: target ratio = {}  turbo disabled = {}",
            target_ratio, turbo_disable);
    }

    // IA32_MISC_ENABLE (MSR 0x1A0) — various flags
    if let Some(misc) = crate::debug::read_msr_safe(0x1A0) {
        let speedstep = misc & (1 << 16) != 0;
        let turbo_disable = misc & (1 << 38) != 0;
        let thermal_monitor = misc & (1 << 3) != 0;
        dbg_out!("[THERM] IA32_MISC_ENABLE: SpeedStep={} TurboDisable={} ThermalMonitor={}",
            speedstep, turbo_disable, thermal_monitor);
    }

    // MSR_PKG_POWER_INFO (MSR 0x614) — TDP
    if let Some(pwr_info) = crate::debug::read_msr_safe(0x614) {
        let tdp_raw = pwr_info & 0x7FFF;
        // Power units from MSR 0x606
        let power_unit = if let Some(pwr_unit_msr) = crate::debug::read_msr_safe(0x606) {
            1u32 << ((pwr_unit_msr & 0xF) as u32)
        } else {
            8 // Default: 1/8 watt
        };
        let tdp_watts = tdp_raw as f64 / power_unit as f64;
        dbg_out!("[THERM] TDP: ~{} (raw={}, unit=1/{}W)", tdp_raw / (power_unit as u64), tdp_raw, power_unit);
    }

    // MSR_RAPL_POWER_UNIT (MSR 0x606) — RAPL units
    if let Some(rapl_unit) = crate::debug::read_msr_safe(0x606) {
        let pwr_unit = 1u32 << ((rapl_unit & 0xF) as u32);
        let energy_unit = 1u32 << (((rapl_unit >> 8) & 0x1F) as u32);
        let time_unit = 1u32 << (((rapl_unit >> 16) & 0xF) as u32);
        dbg_out!("[THERM] RAPL Units: power=1/{} W  energy=1/{} J  time=1/{} s",
            pwr_unit, energy_unit, time_unit);
    }

    // MSR_PKG_ENERGY_STATUS (MSR 0x611) — accumulated energy
    if let Some(energy) = crate::debug::read_msr_safe(0x611) {
        dbg_out!("[THERM] Package energy counter: {} (raw units)", energy & 0xFFFF_FFFF);
    }
}

#[cfg(target_arch = "x86_64")]
fn read_battery_ec() {
    dbg_out!("[THERM] ─── Embedded Controller (EC) ───");

    // Check EC availability (ACPI EC at ports 0x62/0x66)
    let ec_status = crate::debug::inb(0x66);
    if ec_status == 0xFF {
        dbg_out!("[THERM]   EC not present at standard ports (0x62/0x66)");
        return;
    }

    let obf = ec_status & 1 != 0;         // Output buffer full
    let ibf = ec_status & 2 != 0;         // Input buffer full
    let sci = ec_status & (1 << 5) != 0;  // SCI event pending
    let burst = ec_status & (1 << 4) != 0; // Burst mode

    dbg_out!("[THERM]   EC Status (port 0x66): 0x{:02X} (OBF={} IBF={} SCI={} Burst={})",
        ec_status, obf, ibf, sci, burst);
}

#[cfg(target_arch = "x86_64")]
fn read_thinkpad_ec() {
    // ThinkPad EC registers (port 0x1600-0x161F range via LPC)
    // Check if ThinkPad-specific fan/temp registers are accessible
    // EC RAM offset 0x78 = CPU temperature on many ThinkPads
    // EC RAM offset 0x2F = Fan control on many ThinkPads

    // Read a few EC registers via I/O port 0x62/0x66
    // This is safe to probe even on non-ThinkPad machines
    let ec_status = crate::debug::inb(0x66);
    if ec_status == 0xFF { return; }

    // Don't try to read EC RAM if EC is busy
    if ec_status & 2 != 0 {
        dbg_out!("[THERM]   EC busy, skipping ThinkPad-specific probing");
        return;
    }

    dbg_out!("[THERM] ─── ThinkPad EC (if present) ───");
    dbg_out!("[THERM]   (Reading EC RAM requires ACPI method, probing basic ports only)");

    // Check for ThinkPad CMOS magic at CMOS 0x6C
    crate::debug::outb(0x70, 0x6C);
    let thinkpad_magic = crate::debug::inb(0x71);
    if thinkpad_magic != 0xFF && thinkpad_magic != 0x00 {
        dbg_out!("[THERM]   CMOS[0x6C] = 0x{:02X} (may indicate ThinkPad model info)", thinkpad_magic);
    }
}
