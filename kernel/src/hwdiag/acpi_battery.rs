//! ACPI Battery & Thermal Zone Diagnostics
//!
//! Probes battery status via ACPI EC registers and thermal zones
//! from FADT/EC. Since TrustOS has no AML interpreter, we use
//! direct EC register reads (ThinkPad-style) and ACPI fixed registers.

use alloc::string::String;
use alloc::format;
use alloc::vec::Vec;
use super::dbg_out;

/// Battery status information
#[derive(Debug, Clone, Default)]
pub struct BatteryInfo {
    pub present: bool,
    pub charging: bool,
    pub discharging: bool,
    pub charge_percent: Option<u8>,
    pub voltage_mv: Option<u16>,
    pub current_ma: Option<i16>,
    pub temperature_c: Option<i8>,
    pub details: Vec<String>,
}

/// Thermal zone information
#[derive(Debug, Clone)]
pub struct ThermalZone {
    pub name: String,
    pub temperature_c: i32,
    pub critical_c: Option<i32>,
    pub source: String,
}

/// Combined ACPI power/thermal data
#[derive(Debug, Clone, Default)]
pub struct AcpiPowerInfo {
    pub battery: Option<BatteryInfo>,
    pub thermal_zones: Vec<ThermalZone>,
    pub ac_present: Option<bool>,
    pub pm_profile: Option<&'static str>,
    pub sci_enabled: bool,
    pub sleep_states: Vec<String>,
}

/// Run ACPI battery & thermal diagnostics
pub fn run(args: &[&str]) {
    let verbose = args.contains(&"-v") || args.contains(&"--verbose");

    dbg_out!("[ACPI-POWER] === Battery & Thermal Zones ===");
    dbg_out!("");

    let info = collect_all();

    // PM Profile
    if let Some(profile) = info.pm_profile {
        dbg_out!("[ACPI] PM Profile: {}", profile);
    }
    dbg_out!("[ACPI] SCI Enabled: {}", info.sci_enabled);
    if !info.sleep_states.is_empty() {
        dbg_out!("[ACPI] Sleep States: {}", info.sleep_states.join(", "));
    }
    dbg_out!("");

    // AC power
    if let Some(ac) = info.ac_present {
        dbg_out!("[AC] AC Adapter: {}", if ac { "CONNECTED" } else { "DISCONNECTED" });
    }

    // Battery
    match &info.battery {
        Some(bat) if bat.present => {
            dbg_out!("[BAT] Battery: PRESENT");
            let state = if bat.charging { "CHARGING" }
                else if bat.discharging { "DISCHARGING" }
                else { "IDLE/FULL" };
            dbg_out!("[BAT] State: {}", state);
            if let Some(pct) = bat.charge_percent {
                dbg_out!("[BAT] Charge: {}%", pct);
            }
            if let Some(mv) = bat.voltage_mv {
                dbg_out!("[BAT] Voltage: {}.{:03} V", mv / 1000, mv % 1000);
            }
            if let Some(ma) = bat.current_ma {
                dbg_out!("[BAT] Current: {} mA", ma);
            }
            if let Some(t) = bat.temperature_c {
                dbg_out!("[BAT] Temperature: {}°C", t);
            }
            for d in &bat.details {
                dbg_out!("[BAT]   {}", d);
            }
        }
        _ => {
            dbg_out!("[BAT] Battery: NOT DETECTED (desktop or not readable)");
        }
    }
    dbg_out!("");

    // Thermal zones
    if info.thermal_zones.is_empty() {
        dbg_out!("[THERMAL] No thermal zones detected");
    } else {
        dbg_out!("[THERMAL] Thermal Zones ({}):", info.thermal_zones.len());
        for tz in &info.thermal_zones {
            let crit = match tz.critical_c {
                Some(c) => format!(" (critical: {}°C)", c),
                None => String::new(),
            };
            dbg_out!("[THERMAL]   {}: {}°C{} [{}]", tz.name, tz.temperature_c, crit, tz.source);
        }
    }

    // Verbose: dump raw EC registers
    if verbose {
        dbg_out!("");
        dump_raw_ec();
    }
}

/// Collect all battery & thermal info
pub fn collect_all() -> AcpiPowerInfo {
    let mut info = AcpiPowerInfo::default();

    // ACPI FADT analysis
    if let Some(acpi) = crate::acpi::get_info() {
        if let Some(ref fadt) = acpi.fadt {
            info.pm_profile = Some(pm_profile_name(fadt.flags));
            info.sci_enabled = check_sci_enabled(fadt);
            info.sleep_states = detect_sleep_states(fadt);
        }
    }

    // Probe battery & thermal via EC (ThinkPad/laptop)
    #[cfg(target_arch = "x86_64")]
    {
        info.battery = probe_battery_ec();
        info.thermal_zones = probe_thermal_zones();
        info.ac_present = probe_ac_adapter();
    }

    info
}

// ═══════════════════════════════════════════════════════════════════════════════
// FADT / Power Management
// ═══════════════════════════════════════════════════════════════════════════════

fn pm_profile_name(flags: u32) -> &'static str {
    // FADT flags bits for PM profile aren't in flags; PM profile is byte at offset 45
    // We approximate from FADT flags
    if flags & (1 << 20) != 0 { return "HW-Reduced ACPI"; }
    "Full ACPI"
}

fn check_sci_enabled(fadt: &crate::acpi::fadt::FadtInfo) -> bool {
    if fadt.pm1a_cnt_blk == 0 { return false; }

    #[cfg(target_arch = "x86_64")]
    {
        let val: u16 = unsafe { x86_64::instructions::port::Port::new(fadt.pm1a_cnt_blk as u16).read() };
        // Bit 0 = SCI_EN
        val & 1 != 0
    }
    #[cfg(not(target_arch = "x86_64"))]
    { false }
}

fn detect_sleep_states(fadt: &crate::acpi::fadt::FadtInfo) -> Vec<String> {
    let mut states = Vec::new();
    states.push(String::from("S0 (Working)"));

    // S3 (Suspend to RAM) requires PM1a_CNT
    if fadt.pm1a_cnt_blk != 0 {
        states.push(String::from("S3 (Suspend)"));
    }

    // S4 needs S4BIOS support flag
    if fadt.flags & (1 << 4) != 0 {
        states.push(String::from("S4 (Hibernate)"));
    }

    // S5 (Soft Off) always available if ACPI works
    states.push(String::from("S5 (Soft Off)"));

    states
}

// ═══════════════════════════════════════════════════════════════════════════════
// EC-based Battery Probe (Laptop / ThinkPad)
// ═══════════════════════════════════════════════════════════════════════════════

// Standard ACPI EC battery registers (common across many laptops)
// These are the embedded controller register addresses used by ACPI _BST/_BIF methods

// ThinkPad EC battery registers
const EC_BAT_STATUS: u8 = 0x38;     // Battery status flags
const EC_BAT_CHARGE: u8 = 0x39;     // Remaining charge (coarse)
const EC_BAT_PRESENT: u8 = 0x34;    // Battery presence flags

#[cfg(target_arch = "x86_64")]
fn probe_battery_ec() -> Option<BatteryInfo> {
    use crate::drivers::thinkpad_ec;

    if !thinkpad_ec::probe() {
        // Try generic ACPI EC if ThinkPad EC not available
        return probe_battery_generic_ec();
    }

    let mut bat = BatteryInfo::default();

    // Check battery presence via EC register
    if let Some(present_flags) = thinkpad_ec::ec_read(EC_BAT_PRESENT) {
        bat.present = present_flags & 0x01 != 0;
        if !bat.present {
            return Some(bat);
        }
        if present_flags & 0x02 != 0 {
            bat.details.push(String::from("Second battery detected"));
        }
    } else {
        // Try alternate detection: battery temp sensor
        if let Some(temp) = thinkpad_ec::temp_read(4) {
            // Sensor 4 (0x7C) = battery temp on ThinkPads
            if temp > 0 && temp < 128 {
                bat.present = true;
                bat.temperature_c = Some(temp as i8);
            }
        }
    }

    if !bat.present {
        return Some(bat);
    }

    // Read battery status
    if let Some(status) = thinkpad_ec::ec_read(EC_BAT_STATUS) {
        bat.discharging = status & 0x01 != 0;
        bat.charging = status & 0x02 != 0;
    }

    // Read charge level
    if let Some(charge) = thinkpad_ec::ec_read(EC_BAT_CHARGE) {
        if charge <= 100 {
            bat.charge_percent = Some(charge);
        }
    }

    // Battery temperature from ThinkPad EC sensor 4
    if bat.temperature_c.is_none() {
        if let Some(temp) = thinkpad_ec::temp_read(4) {
            if temp > 0 && temp < 128 {
                bat.temperature_c = Some(temp as i8);
            }
        }
    }

    Some(bat)
}

/// Generic ACPI EC battery probe (non-ThinkPad)
#[cfg(target_arch = "x86_64")]
fn probe_battery_generic_ec() -> Option<BatteryInfo> {
    // Standard ACPI EC at ports 0x62/0x66
    let ec_status: u8 = unsafe { x86_64::instructions::port::Port::new(0x66u16).read() };

    // If EC doesn't respond (all 0xFF), no EC present
    if ec_status == 0xFF {
        return None;
    }

    // EC exists but we can't read battery without ACPI AML methods
    // Report what we can detect
    let mut bat = BatteryInfo::default();
    bat.details.push(format!("EC status: {:#04x} (OBF={}, IBF={})",
        ec_status, ec_status & 1, (ec_status >> 1) & 1));
    bat.details.push(String::from("Battery reading requires ACPI AML interpreter (not yet implemented)"));

    Some(bat)
}

// ═══════════════════════════════════════════════════════════════════════════════
// Thermal Zone Probe
// ═══════════════════════════════════════════════════════════════════════════════

#[cfg(target_arch = "x86_64")]
fn probe_thermal_zones() -> Vec<ThermalZone> {
    let mut zones = Vec::new();

    // CPU thermal via MSR
    if let Some(tz) = probe_cpu_msr_thermal() {
        zones.push(tz);
    }

    // Package thermal via MSR
    if let Some(tz) = probe_package_msr_thermal() {
        zones.push(tz);
    }

    // EC temperature sensors (ThinkPad)
    if crate::drivers::thinkpad_ec::probe() {
        let labels = ["CPU", "miniPCI", "HDD", "GPU", "Battery", "Sensor5", "Sensor6", "Sensor7"];
        for i in 0..8 {
            if let Some(temp) = crate::drivers::thinkpad_ec::temp_read(i) {
                if temp > 0 && temp < 128 {
                    zones.push(ThermalZone {
                        name: format!("EC/{}", labels[i]),
                        temperature_c: temp as i32,
                        critical_c: Some(if i == 0 { 105 } else { 90 }), // typical laptop limits
                        source: String::from("Embedded Controller"),
                    });
                }
            }
        }
    }

    zones
}

#[cfg(target_arch = "x86_64")]
fn probe_cpu_msr_thermal() -> Option<ThermalZone> {
    use crate::debug::read_msr_safe;

    // IA32_THERM_STATUS (0x19C)
    let therm = read_msr_safe(0x19C)?;
    let valid = therm & (1 << 31) != 0; // Digital readout valid

    if !valid { return None; }

    let digital_readout = ((therm >> 16) & 0x7F) as i32;

    // TjMax from MSR 0x1A2 (IA32_TEMPERATURE_TARGET)
    let tj_max = read_msr_safe(0x1A2)
        .map(|v| ((v >> 16) & 0xFF) as i32)
        .unwrap_or(100);

    let temp = tj_max - digital_readout;

    Some(ThermalZone {
        name: String::from("CPU Core"),
        temperature_c: temp,
        critical_c: Some(tj_max),
        source: String::from("MSR IA32_THERM_STATUS"),
    })
}

#[cfg(target_arch = "x86_64")]
fn probe_package_msr_thermal() -> Option<ThermalZone> {
    use crate::debug::read_msr_safe;

    // IA32_PACKAGE_THERM_STATUS (0x1B1)
    let therm = read_msr_safe(0x1B1)?;
    let valid = therm & (1 << 31) != 0;

    if !valid { return None; }

    let digital_readout = ((therm >> 16) & 0x7F) as i32;
    let tj_max = read_msr_safe(0x1A2)
        .map(|v| ((v >> 16) & 0xFF) as i32)
        .unwrap_or(100);
    let temp = tj_max - digital_readout;

    Some(ThermalZone {
        name: String::from("CPU Package"),
        temperature_c: temp,
        critical_c: Some(tj_max),
        source: String::from("MSR IA32_PACKAGE_THERM_STATUS"),
    })
}

// ═══════════════════════════════════════════════════════════════════════════════
// AC Adapter Detection
// ═══════════════════════════════════════════════════════════════════════════════

#[cfg(target_arch = "x86_64")]
fn probe_ac_adapter() -> Option<bool> {
    use crate::drivers::thinkpad_ec;

    if !thinkpad_ec::probe() {
        return None;
    }

    // ThinkPad EC register for AC adapter
    // Register 0x46 typically holds power source info on ThinkPads
    if let Some(val) = thinkpad_ec::ec_read(0x46) {
        return Some(val & 0x01 != 0);
    }

    None
}

// ═══════════════════════════════════════════════════════════════════════════════
// Verbose: Raw EC register dump
// ═══════════════════════════════════════════════════════════════════════════════

fn dump_raw_ec() {
    #[cfg(target_arch = "x86_64")]
    {
        use crate::drivers::thinkpad_ec;

        if !thinkpad_ec::probe() {
            dbg_out!("[EC-RAW] ThinkPad EC not available");
            return;
        }

        dbg_out!("[EC-RAW] EC Register Dump (0x00-0xFF):");
        dbg_out!("[EC-RAW]      00 01 02 03 04 05 06 07 08 09 0A 0B 0C 0D 0E 0F");
        for row in 0..16 {
            let base = row * 16u8;
            let mut line = format!("[EC-RAW] {:02X}: ", base);
            for col in 0..16 {
                match thinkpad_ec::ec_read(base.wrapping_add(col)) {
                    Some(v) => line.push_str(&format!("{:02X} ", v)),
                    None => line.push_str("-- "),
                }
            }
            dbg_out!("{}", line);
        }
    }
}
