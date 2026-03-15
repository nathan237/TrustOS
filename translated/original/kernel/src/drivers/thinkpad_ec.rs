//! ThinkPad Embedded Controller (EC) Driver
//!
//! Provides fan control, thermal monitoring, and CPU frequency/voltage
//! management for ThinkPad laptops (tested on T61).
//!
//! EC communication: ports 0x66 (command/status) and 0x62 (data).
//! Fan/thermal registers: reverse-engineered from thinkpad_acpi Linux driver.
//! SpeedStep: Intel Enhanced SpeedStep via MSR 0x198/0x199.

use core::sync::atomic::{AtomicBool, Ordering};
use crate::arch::Port;

// ═══════════════════════════════════════════════════════════════════════════════
// EC I/O Ports
// ═══════════════════════════════════════════════════════════════════════════════

const EC_DATA_PORT: u16 = 0x62;
const EC_CMD_PORT: u16 = 0x66;  // also status port

// EC status bits
const EC_STATUS_OBF: u8 = 0x01;  // Output Buffer Full — data ready to read
const EC_STATUS_IBF: u8 = 0x02;  // Input Buffer Full — EC busy

// EC commands
const EC_CMD_READ: u8 = 0x80;
const EC_CMD_WRITE: u8 = 0x81;

// ═══════════════════════════════════════════════════════════════════════════════
// ThinkPad EC Registers (from thinkpad_acpi)
// ═══════════════════════════════════════════════════════════════════════════════

/// Fan control register: 0-7 = speed level, 0x40 = auto, 0x80 = full speed  
const EC_REG_FAN_CONTROL: u8 = 0x2F;

/// Fan RPM registers (big-endian u16)
const EC_REG_FAN_RPM_HI: u8 = 0x84;
const EC_REG_FAN_RPM_LO: u8 = 0x85;

/// Temperature sensor registers (°C, one per sensor)
const EC_REG_TEMP_BASE: u8 = 0x78;
const EC_TEMP_SENSOR_COUNT: usize = 8;  // 0x78..0x7F

/// Sensor labels for T61 (approximate — varies by model)
const TEMP_LABELS: [&str; 8] = [
    "CPU",          // 0x78
    "miniPCI",      // 0x79
    "HDD",          // 0x7A
    "GPU",          // 0x7B
    "Battery",      // 0x7C
    "Sensor 5",     // 0x7D
    "Sensor 6",     // 0x7E
    "Sensor 7",     // 0x7F
];

// ═══════════════════════════════════════════════════════════════════════════════
// Intel SpeedStep MSRs (Core 2 Duo era)
// ═══════════════════════════════════════════════════════════════════════════════

const MSR_IA32_PERF_STATUS: u32 = 0x198;
const MSR_IA32_PERF_CTL: u32 = 0x199;
const MSR_IA32_MISC_ENABLE: u32 = 0x1A0;
const MSR_IA32_THERM_STATUS: u32 = 0x19C;

// ═══════════════════════════════════════════════════════════════════════════════
// State
// ═══════════════════════════════════════════════════════════════════════════════

static EC_AVAILABLE: AtomicBool = AtomicBool::new(false);

// ═══════════════════════════════════════════════════════════════════════════════
// EC Low-Level I/O
// ═══════════════════════════════════════════════════════════════════════════════

/// Wait for EC input buffer to be empty (EC ready to accept data)
fn ec_wait_ibf_clear() -> bool {
    let mut status_port: Port<u8> = Port::new(EC_CMD_PORT);
    let mut delay_port: Port<u8> = Port::new(0x80);
    for _ in 0..100_000u32 {
        let status = unsafe { status_port.read() };
        if status & EC_STATUS_IBF == 0 {
            return true;
        }
        unsafe { delay_port.read(); }
    }
    false
}

/// Wait for EC output buffer to have data
fn ec_wait_obf_set() -> bool {
    let mut status_port: Port<u8> = Port::new(EC_CMD_PORT);
    let mut delay_port: Port<u8> = Port::new(0x80);
    for _ in 0..100_000u32 {
        let status = unsafe { status_port.read() };
        if status & EC_STATUS_OBF != 0 {
            return true;
        }
        unsafe { delay_port.read(); }
    }
    false
}

/// Read one byte from an EC register
pub fn ec_read(reg: u8) -> Option<u8> {
    let mut cmd_port: Port<u8> = Port::new(EC_CMD_PORT);
    let mut data_port: Port<u8> = Port::new(EC_DATA_PORT);
    if !ec_wait_ibf_clear() { return None; }
    unsafe { cmd_port.write(EC_CMD_READ); }
    if !ec_wait_ibf_clear() { return None; }
    unsafe { data_port.write(reg); }
    if !ec_wait_obf_set() { return None; }
    Some(unsafe { data_port.read() })
}

/// Write one byte to an EC register
pub fn ec_write(reg: u8, val: u8) -> bool {
    let mut cmd_port: Port<u8> = Port::new(EC_CMD_PORT);
    let mut data_port: Port<u8> = Port::new(EC_DATA_PORT);
    if !ec_wait_ibf_clear() { return false; }
    unsafe { cmd_port.write(EC_CMD_WRITE); }
    if !ec_wait_ibf_clear() { return false; }
    unsafe { data_port.write(reg); }
    if !ec_wait_ibf_clear() { return false; }
    unsafe { data_port.write(val); }
    true
}

// ═══════════════════════════════════════════════════════════════════════════════
// EC Probe
// ═══════════════════════════════════════════════════════════════════════════════

/// Probe the EC — try reading the CPU temp register. If we get a sane value, it works.
pub fn probe() -> bool {
    if let Some(temp) = ec_read(EC_REG_TEMP_BASE) {
        // Sane temperature: 10-120°C
        if temp >= 10 && temp <= 120 {
            EC_AVAILABLE.store(true, Ordering::Relaxed);
            crate::serial_println!("[EC] ThinkPad EC detected — CPU temp: {}°C", temp);
            return true;
        }
    }
    crate::serial_println!("[EC] ThinkPad EC not detected or not responding");
    false
}

pub fn is_available() -> bool {
    EC_AVAILABLE.load(Ordering::Relaxed)
}

// ═══════════════════════════════════════════════════════════════════════════════
// Fan Control
// ═══════════════════════════════════════════════════════════════════════════════

/// Fan level: 0-7 (manual), Auto, FullSpeed
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FanLevel {
    Level(u8),   // 0-7
    Auto,        // EC automatic control
    FullSpeed,   // Maximum RPM (BIOS override)
}

/// Get current fan level
pub fn fan_get_level() -> Option<u8> {
    ec_read(EC_REG_FAN_CONTROL)
}

/// Set fan level
pub fn fan_set_level(level: FanLevel) -> bool {
    let val = match level {
        FanLevel::Level(l) => {
            if l > 7 { return false; }
            l
        }
        FanLevel::Auto => 0x80,      // bit 7 = auto mode
        FanLevel::FullSpeed => 0x40, // bit 6 = disengaged (full speed)
    };
    ec_write(EC_REG_FAN_CONTROL, val)
}

/// Read fan speed in RPM (returns 0 if fan is stopped or unsupported)
pub fn fan_get_rpm() -> Option<u16> {
    let hi = ec_read(EC_REG_FAN_RPM_HI)?;
    let lo = ec_read(EC_REG_FAN_RPM_LO)?;
    Some(((hi as u16) << 8) | lo as u16)
}

// ═══════════════════════════════════════════════════════════════════════════════
// Temperature Sensors
// ═══════════════════════════════════════════════════════════════════════════════

/// Read temperature from a specific sensor (0-7). Returns °C or None.
pub fn temp_read(sensor: usize) -> Option<u8> {
    if sensor >= EC_TEMP_SENSOR_COUNT { return None; }
    let val = ec_read(EC_REG_TEMP_BASE + sensor as u8)?;
    // 0x00 or 0x80+ usually means sensor not present
    if val == 0 || val >= 128 { return None; }
    Some(val)
}

/// Get label for a temperature sensor
pub fn temp_label(sensor: usize) -> &'static str {
    if sensor < EC_TEMP_SENSOR_COUNT {
        TEMP_LABELS[sensor]
    } else {
        "Unknown"
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// CPU Frequency / Voltage (Intel SpeedStep via MSR)
// ═══════════════════════════════════════════════════════════════════════════════

/// Current CPU performance status: (frequency_mhz, voltage_mv)
#[cfg(target_arch = "x86_64")]
pub fn cpu_perf_status() -> Option<(u32, u32)> {
    let val = crate::debug::read_msr_safe(MSR_IA32_PERF_STATUS)?;
    // Bits [15:0] = Current performance state value
    // Core 2 Duo encoding: bits [15:8] = FID (frequency ID), bits [7:0] = VID (voltage ID)
    let fid = ((val >> 8) & 0xFF) as u32;
    let vid = (val & 0xFF) as u32;
    
    // Core 2 Duo: freq = FID * FSB (usually 200MHz for T61)
    // T61 FSB is 800MHz quad-pumped = 200MHz base
    let freq_mhz = fid * 200;
    
    // Core 2 Duo VID: voltage = 0.7125V + VID * 0.0125V  (approximate)
    // Some models use different tables, but this is the common Merom encoding
    let voltage_mv = if vid > 0 {
        712 + vid * 12  // millivolts (approximate)
    } else {
        0
    };
    
    Some((freq_mhz, voltage_mv))
}

#[cfg(not(target_arch = "x86_64"))]
pub fn cpu_perf_status() -> Option<(u32, u32)> {
    None
}

/// Read the requested P-state (what we asked for)
#[cfg(target_arch = "x86_64")]
pub fn cpu_perf_target() -> Option<u16> {
    let val = crate::debug::read_msr_safe(MSR_IA32_PERF_CTL)?;
    Some((val & 0xFFFF) as u16)
}

#[cfg(not(target_arch = "x86_64"))]
pub fn cpu_perf_target() -> Option<u16> {
    None
}

/// Set CPU P-state (frequency/voltage pair)
/// fid: Frequency ID (multiplier), vid: Voltage ID
/// WARNING: Wrong values can hang or damage the CPU. Use known-good P-states only.
#[cfg(target_arch = "x86_64")]
pub fn cpu_set_pstate(fid: u8, vid: u8) -> bool {
    let val = ((fid as u64) << 8) | (vid as u64);
    crate::debug::write_msr(MSR_IA32_PERF_CTL, val);
    true
}

#[cfg(not(target_arch = "x86_64"))]
pub fn cpu_set_pstate(_fid: u8, _vid: u8) -> bool {
    false
}

/// Check if Enhanced SpeedStep (EIST) is enabled
#[cfg(target_arch = "x86_64")]
pub fn eist_enabled() -> Option<bool> {
    let val = crate::debug::read_msr_safe(MSR_IA32_MISC_ENABLE)?;
    Some((val & (1 << 16)) != 0)
}

#[cfg(not(target_arch = "x86_64"))]
pub fn eist_enabled() -> Option<bool> {
    None
}

/// Read CPU digital thermal sensor (DTS) — degrees below TjMax
#[cfg(target_arch = "x86_64")]
pub fn cpu_therm_status() -> Option<(bool, u8)> {
    let val = crate::debug::read_msr_safe(MSR_IA32_THERM_STATUS)?;
    let valid = (val & (1 << 31)) != 0;  // Reading valid bit
    let readout = ((val >> 16) & 0x7F) as u8;  // Digital readout (degrees below TjMax)
    Some((valid, readout))
}

#[cfg(not(target_arch = "x86_64"))]
pub fn cpu_therm_status() -> Option<(bool, u8)> {
    None
}

// ═══════════════════════════════════════════════════════════════════════════════
// Known P-States for T61 Core 2 Duo T7x00 / T8x00 series
// ═══════════════════════════════════════════════════════════════════════════════

/// (label, FID, VID) — T61 Core 2 Duo T7300 (2.0GHz, FSB 800MHz)
/// These are approximate. Actual values depend on specific CPU stepping.
pub const T61_PSTATES: &[(&str, u8, u8)] = &[
    ("2.0 GHz (max)",  10, 38),  // FID=10 * 200MHz = 2000MHz
    ("1.6 GHz",         8, 30),
    ("1.2 GHz",         6, 22),
    ("800 MHz (min)",   4, 16),
];

// ═══════════════════════════════════════════════════════════════════════════════
// Shell Command Handlers
// ═══════════════════════════════════════════════════════════════════════════════

/// `fan` command — ThinkPad fan control
pub fn cmd_fan(args: &[&str]) {
    use crate::framebuffer::*;

    if !is_available() {
        if !probe() {
            crate::println_color!(COLOR_RED, "EC not available — not a ThinkPad or EC unresponsive");
            return;
        }
    }

    match args.first().copied() {
        None | Some("status") => {
            // Show current fan status
            crate::println_color!(COLOR_CYAN, "=== ThinkPad Fan Status ===");
            
            if let Some(level) = fan_get_level() {
                let desc = match level {
                    0x80 => "auto (EC controlled)",
                    0x40 => "DISENGAGED (full speed)",
                    l if l <= 7 => match l {
                        0 => "0 (off)",
                        1 => "1 (lowest)",
                        2 => "2",
                        3 => "3",
                        4 => "4",
                        5 => "5",
                        6 => "6",
                        7 => "7 (highest manual)",
                        _ => "?",
                    },
                    _ => "unknown",
                };
                crate::println!("  Level: 0x{:02X} — {}", level, desc);
            } else {
                crate::println_color!(COLOR_RED, "  Level: read failed");
            }

            if let Some(rpm) = fan_get_rpm() {
                if rpm == 0 || rpm == 0xFFFF {
                    crate::println!("  RPM:   stopped or N/A");
                } else {
                    crate::println!("  RPM:   {}", rpm);
                }
            } else {
                crate::println!("  RPM:   read failed");
            }
        }

        Some("auto") => {
            if fan_set_level(FanLevel::Auto) {
                crate::println_color!(COLOR_GREEN, "Fan set to AUTO (EC controlled)");
            } else {
                crate::println_color!(COLOR_RED, "Failed to set fan to auto");
            }
        }

        Some("max") | Some("full") => {
            if fan_set_level(FanLevel::FullSpeed) {
                crate::println_color!(COLOR_YELLOW, "Fan set to FULL SPEED (disengaged)");
            } else {
                crate::println_color!(COLOR_RED, "Failed to set fan to full speed");
            }
        }

        Some("off") | Some("0") => {
            crate::println_color!(COLOR_YELLOW, "WARNING: Turning fan off! Monitor temperatures carefully.");
            if fan_set_level(FanLevel::Level(0)) {
                crate::println_color!(COLOR_RED, "Fan OFF");
            } else {
                crate::println_color!(COLOR_RED, "Failed to turn fan off");
            }
        }

        Some(n) => {
            if let Ok(level) = n.parse::<u8>() {
                if level <= 7 {
                    if fan_set_level(FanLevel::Level(level)) {
                        crate::println_color!(COLOR_GREEN, "Fan set to level {}", level);
                    } else {
                        crate::println_color!(COLOR_RED, "Failed to set fan level");
                    }
                } else {
                    crate::println_color!(COLOR_RED, "Fan level must be 0-7, 'auto', 'max', or 'off'");
                }
            } else {
                crate::println!("Usage: fan [status|auto|max|off|0-7]");
                crate::println!("  fan          Show current fan status");
                crate::println!("  fan auto     Let EC control the fan");
                crate::println!("  fan max      Full speed (disengaged)");
                crate::println!("  fan off      Turn fan off (DANGEROUS)");
                crate::println!("  fan 0-7      Set manual speed level");
            }
        }
    }
}

/// `temp` / `sensors` command — Show temperatures
pub fn cmd_temp(_args: &[&str]) {
    use crate::framebuffer::*;

    if !is_available() {
        if !probe() {
            crate::println_color!(COLOR_RED, "EC not available — not a ThinkPad or EC unresponsive");
            return;
        }
    }

    crate::println_color!(COLOR_CYAN, "=== ThinkPad Temperature Sensors ===");
    
    let mut any_sensor = false;
    for i in 0..EC_TEMP_SENSOR_COUNT {
        if let Some(temp) = temp_read(i) {
            any_sensor = true;
            let color = if temp >= 90 {
                COLOR_RED
            } else if temp >= 70 {
                COLOR_YELLOW
            } else {
                COLOR_GREEN
            };
            crate::print!("  {:10} ", temp_label(i));
            crate::println_color!(color, "{}°C", temp);
        }
    }

    if !any_sensor {
        crate::println!("  No temperature sensors responded");
    }

    // Also show CPU DTS if available
    #[cfg(target_arch = "x86_64")]
    if let Some((valid, dts)) = cpu_therm_status() {
        if valid {
            // TjMax for Core 2 Duo is typically 100°C
            let tj_max: u8 = 100;
            let cpu_temp = tj_max.saturating_sub(dts);
            let color = if cpu_temp >= 90 {
                COLOR_RED
            } else if cpu_temp >= 70 {
                COLOR_YELLOW
            } else {
                COLOR_GREEN
            };
            crate::print!("  {:10} ", "CPU (DTS)");
            crate::println_color!(color, "{}°C (TjMax={}, margin={}°C)", cpu_temp, tj_max, dts);
        }
    }

    // Show fan info too
    crate::println!();
    if let Some(rpm) = fan_get_rpm() {
        if rpm > 0 && rpm != 0xFFFF {
            crate::println!("  Fan:       {} RPM", rpm);
        } else {
            crate::println!("  Fan:       stopped");
        }
    }
}

/// `cpufreq` command — CPU frequency/voltage control
pub fn cmd_cpufreq(args: &[&str]) {
    use crate::framebuffer::*;

    match args.first().copied() {
        None | Some("status") => {
            crate::println_color!(COLOR_CYAN, "=== CPU Frequency / Voltage ===");

            // EIST status
            match eist_enabled() {
                Some(true) => crate::println_color!(COLOR_GREEN, "  SpeedStep (EIST): enabled"),
                Some(false) => crate::println_color!(COLOR_YELLOW, "  SpeedStep (EIST): disabled"),
                None => crate::println!("  SpeedStep (EIST): unknown"),
            }

            // Current P-state
            if let Some((freq, voltage)) = cpu_perf_status() {
                crate::println!("  Current freq:     {} MHz", freq);
                if voltage > 0 {
                    crate::println!("  Current voltage:  {}.{:03} V", voltage / 1000, voltage % 1000);
                }
            } else {
                crate::println!("  Current P-state:  read failed");
            }

            // Target P-state
            if let Some(target) = cpu_perf_target() {
                let tfid = (target >> 8) & 0xFF;
                let tvid = target & 0xFF;
                crate::println!("  Target:           FID={} VID={} ({}MHz)", tfid, tvid, tfid as u32 * 200);
            }

            // DTS temperature
            #[cfg(target_arch = "x86_64")]
            if let Some((valid, dts)) = cpu_therm_status() {
                if valid {
                    crate::println!("  CPU temp (DTS):   {}°C (margin: {}°C to TjMax)", 100u8.saturating_sub(dts), dts);
                }
            }

            // Show known P-states
            crate::println!();
            crate::println_color!(COLOR_CYAN, "  Known T61 P-states (Core 2 Duo, FSB 800MHz):");
            for (label, fid, vid) in T61_PSTATES {
                crate::println!("    FID={:2} VID={:2}  → {}", fid, vid, label);
            }
        }

        Some("set") => {
            if args.len() < 3 {
                crate::println!("Usage: cpufreq set <fid> <vid>");
                crate::println!("  Use 'cpufreq status' to see known P-states");
                return;
            }
            let fid = match args[1].parse::<u8>() {
                Ok(f) => f,
                Err(_) => {
                    crate::println_color!(COLOR_RED, "Invalid FID: {}", args[1]);
                    return;
                }
            };
            let vid = match args[2].parse::<u8>() {
                Ok(v) => v,
                Err(_) => {
                    crate::println_color!(COLOR_RED, "Invalid VID: {}", args[2]);
                    return;
                }
            };
            crate::println_color!(COLOR_YELLOW, "Setting P-state: FID={} VID={} ({}MHz)", fid, vid, fid as u32 * 200);
            if cpu_set_pstate(fid, vid) {
                crate::println_color!(COLOR_GREEN, "P-state change requested");
                // Read back
                if let Some((freq, voltage)) = cpu_perf_status() {
                    crate::println!("  Now running at: {} MHz, {}.{:03} V", freq, voltage / 1000, voltage % 1000);
                }
            } else {
                crate::println_color!(COLOR_RED, "Failed to set P-state");
            }
        }

        Some("max") => {
            if let Some(&(label, fid, vid)) = T61_PSTATES.first() {
                crate::println_color!(COLOR_YELLOW, "Setting CPU to {}", label);
                cpu_set_pstate(fid, vid);
                crate::println_color!(COLOR_GREEN, "Done");
            }
        }

        Some("min") | Some("powersave") => {
            if let Some(&(label, fid, vid)) = T61_PSTATES.last() {
                crate::println_color!(COLOR_YELLOW, "Setting CPU to {}", label);
                cpu_set_pstate(fid, vid);
                crate::println_color!(COLOR_GREEN, "Done");
            }
        }

        _ => {
            crate::println!("Usage: cpufreq [status|set|max|min]");
            crate::println!("  cpufreq            Show current frequency/voltage");
            crate::println!("  cpufreq set <f> <v> Set P-state (FID, VID)");
            crate::println!("  cpufreq max        Set maximum performance");
            crate::println!("  cpufreq min        Set minimum (powersave)");
        }
    }
}
