//! ATA SMART — Self-Monitoring, Analysis and Reporting Technology
//!
//! Reads SMART data from ATA/SATA drives via both legacy IDE PIO
//! and AHCI (via the existing AHCI command infrastructure).
//! Provides health status, temperature, power-on hours, reallocated sectors, etc.

use alloc::string::String;
use alloc::format;
use alloc::vec::Vec;
use super::dbg_out;

/// ATA SMART command
const ATA_CMD_SMART: u8 = 0xB0;

/// SMART feature register values
const SMART_READ_DATA: u8 = 0xD0;
const SMART_READ_THRESHOLDS: u8 = 0xD1;
const SMART_ENABLE: u8 = 0xD8;
const SMART_RETURN_STATUS: u8 = 0xDA;

/// SMART LBA signature (must set LBA Mid=0x4F, LBA Hi=0xC2)
const SMART_LBA_MID: u8 = 0x4F;
const SMART_LBA_HI: u8 = 0xC2;

/// Known SMART attribute IDs
#[derive(Debug, Clone)]
pub struct SmartAttribute {
    pub id: u8,
    pub name: &'static str,
    pub current: u8,
    pub worst: u8,
    pub threshold: u8,
    pub raw_value: u64,
    pub pre_fail: bool,
}

/// Full SMART data for a drive
#[derive(Debug, Clone)]
pub struct SmartData {
    pub drive_name: String,
    pub smart_supported: bool,
    pub smart_enabled: bool,
    pub health_ok: bool,
    pub attributes: Vec<SmartAttribute>,
    pub temperature_c: Option<i32>,
    pub power_on_hours: Option<u64>,
    pub reallocated_sectors: Option<u64>,
    pub pending_sectors: Option<u64>,
    pub uncorrectable_sectors: Option<u64>,
    pub power_cycle_count: Option<u64>,
}

impl Default for SmartData {
    fn default() -> Self {
        Self {
            drive_name: String::new(),
            smart_supported: false,
            smart_enabled: false,
            health_ok: true,
            attributes: Vec::new(),
            temperature_c: None,
            power_on_hours: None,
            reallocated_sectors: None,
            pending_sectors: None,
            uncorrectable_sectors: None,
            power_cycle_count: None,
        }
    }
}

/// Run SMART diagnostics on all detected drives
pub fn run(args: &[&str]) {
    let verbose = args.contains(&"-v") || args.contains(&"--verbose");

    dbg_out!("[SMART] === ATA SMART Health Report ===");

    let mut found_any = false;

    // Try IDE drives
    #[cfg(target_arch = "x86_64")]
    {
        if let Some(ide_results) = probe_ide_smart() {
            for data in &ide_results {
                dump_smart(data, verbose);
                found_any = true;
            }
        }
    }

    // Try AHCI drives
    if let Some(ahci_results) = probe_ahci_smart() {
        for data in &ahci_results {
            dump_smart(data, verbose);
            found_any = true;
        }
    }

    if !found_any {
        dbg_out!("[SMART] No SMART-capable drives detected");
        dbg_out!("[SMART] (NVMe drives use a different health protocol — see hwdbg storage)");
    }
}

/// Collect SMART data for integration with marionet/probe
pub fn collect_all() -> Vec<SmartData> {
    let mut results = Vec::new();

    #[cfg(target_arch = "x86_64")]
    if let Some(ide) = probe_ide_smart() {
        results.extend(ide);
    }

    if let Some(ahci) = probe_ahci_smart() {
        results.extend(ahci);
    }

    results
}

fn dump_smart(data: &SmartData, verbose: bool) {
    dbg_out!("");
    dbg_out!("[SMART] Drive: {}", data.drive_name);
    dbg_out!("[SMART] SMART Supported: {}  Enabled: {}", 
        if data.smart_supported { "Yes" } else { "No" },
        if data.smart_enabled { "Yes" } else { "No" });

    if !data.smart_supported {
        return;
    }

    // Health Status
    if data.health_ok {
        dbg_out!("[SMART] Health Status: PASSED");
    } else {
        dbg_out!("[SMART] Health Status: *** FAILING *** — Backup data immediately!");
    }

    // Key metrics
    if let Some(temp) = data.temperature_c {
        dbg_out!("[SMART] Temperature:        {}°C", temp);
    }
    if let Some(hours) = data.power_on_hours {
        let days = hours / 24;
        let years = days / 365;
        dbg_out!("[SMART] Power-On Hours:     {} ({} days, ~{} years)", hours, days, years);
    }
    if let Some(cycles) = data.power_cycle_count {
        dbg_out!("[SMART] Power Cycles:       {}", cycles);
    }
    if let Some(realloc) = data.reallocated_sectors {
        let warn = if realloc > 0 { " ⚠" } else { "" };
        dbg_out!("[SMART] Reallocated Sectors: {}{}", realloc, warn);
    }
    if let Some(pending) = data.pending_sectors {
        let warn = if pending > 0 { " ⚠" } else { "" };
        dbg_out!("[SMART] Pending Sectors:    {}{}", pending, warn);
    }
    if let Some(uncorr) = data.uncorrectable_sectors {
        let warn = if uncorr > 0 { " ⚠" } else { "" };
        dbg_out!("[SMART] Uncorrectable:      {}{}", uncorr, warn);
    }

    if verbose && !data.attributes.is_empty() {
        dbg_out!("");
        dbg_out!("[SMART] {:>3} {:<25} {:>4} {:>5} {:>5} {:>12} {}", 
            "ID", "Attribute", "Cur", "Worst", "Thresh", "Raw", "Flag");
        dbg_out!("[SMART] {} {} {} {} {} {} {}",
            "---", "-------------------------", "----", "-----", "-----", "------------", "----");
        for attr in &data.attributes {
            let flag = if attr.pre_fail { "PF" } else { "OC" };
            let warn = if attr.current <= attr.threshold && attr.threshold > 0 { "!" } else { " " };
            dbg_out!("[SMART] {:>3} {:<25} {:>4} {:>5} {:>5} {:>12} {} {}", 
                attr.id, attr.name, attr.current, attr.worst, attr.threshold,
                attr.raw_value, flag, warn);
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// IDE (PIO-mode) SMART
// ═══════════════════════════════════════════════════════════════════════════════

#[cfg(target_arch = "x86_64")]
fn probe_ide_smart() -> Option<Vec<SmartData>> {
    use crate::arch::Port;

    let mut results = Vec::new();

    // Check primary and secondary IDE channels, master and slave
    let channels: &[(u16, u16, &str)] = &[
        (0x1F0, 0x3F6, "Primary"),
        (0x170, 0x376, "Secondary"),
    ];

    for &(base, _ctrl, channel_name) in channels {
        for slave in [false, true] {
            let drive_name = format!("IDE {} {}", channel_name, if slave { "Slave" } else { "Master" });

            // Select drive
            let drive_sel = if slave { 0xB0 } else { 0xA0 };
            unsafe { Port::<u8>::new(base + 6).write(drive_sel); }
            // 400ns delay
            for _ in 0..4 { unsafe { let _ = Port::<u8>::new(base + 7).read(); } }

            // Check drive presence
            let status = unsafe { Port::<u8>::new(base + 7).read() };
            if status == 0 || status == 0xFF { continue; }

            // First check if SMART is supported via IDENTIFY
            let smart_supported = check_ide_smart_support(base, slave);
            if !smart_supported { continue; }

            let mut data = SmartData {
                drive_name,
                smart_supported: true,
                smart_enabled: false,
                ..SmartData::default()
            };

            // Enable SMART
            if ide_smart_command(base, slave, SMART_ENABLE, 0).is_ok() {
                data.smart_enabled = true;
            }

            // Check health status
            if let Ok(healthy) = ide_smart_return_status(base, slave) {
                data.health_ok = healthy;
            }

            // Read SMART data
            if let Ok(smart_buf) = ide_smart_read_data(base, slave) {
                parse_smart_attributes(&smart_buf, &mut data);
            }

            // Read thresholds
            if let Ok(thresh_buf) = ide_smart_read_thresholds(base, slave) {
                apply_thresholds(&thresh_buf, &mut data);
            }

            extract_key_metrics(&mut data);
            results.push(data);
        }
    }

    if results.is_empty() { None } else { Some(results) }
}

#[cfg(target_arch = "x86_64")]
fn check_ide_smart_support(base: u16, slave: bool) -> bool {
    use crate::arch::Port;

    let drive_sel = if slave { 0xB0 } else { 0xA0 };
    unsafe { Port::<u8>::new(base + 6).write(drive_sel); }
    for _ in 0..4 { unsafe { let _ = Port::<u8>::new(base + 7).read(); } }

    // Clear registers
    unsafe {
        Port::<u8>::new(base + 2).write(0);
        Port::<u8>::new(base + 3).write(0);
        Port::<u8>::new(base + 4).write(0);
        Port::<u8>::new(base + 5).write(0);
    }

    // Send IDENTIFY
    unsafe { Port::<u8>::new(base + 7).write(0xEC); }

    // Wait
    for _ in 0..100_000 {
        let s = unsafe { Port::<u8>::new(base + 7).read() };
        if s == 0 || s == 0xFF { return false; }
        if s & 0x80 == 0 { break; }
        core::hint::spin_loop();
    }

    // Wait for DRQ
    for _ in 0..100_000 {
        let s = unsafe { Port::<u8>::new(base + 7).read() };
        if s & 0x01 != 0 { return false; } // Error
        if s & 0x80 == 0 && s & 0x08 != 0 { break; }
        core::hint::spin_loop();
    }

    // Read identify data
    let mut identify = [0u16; 256];
    for i in 0..256 {
        identify[i] = unsafe { Port::<u16>::new(base).read() };
    }

    // Word 82: SMART support (bit 0)
    (identify[82] & 1) != 0
}

#[cfg(target_arch = "x86_64")]
fn ide_smart_command(base: u16, slave: bool, feature: u8, _count: u8) -> Result<(), &'static str> {
    use crate::arch::Port;

    let drive_sel = if slave { 0xB0 } else { 0xA0 };
    unsafe {
        Port::<u8>::new(base + 6).write(drive_sel);
        for _ in 0..4 { let _ = Port::<u8>::new(base + 7).read(); }

        Port::<u8>::new(base + 1).write(feature);      // Feature register
        Port::<u8>::new(base + 2).write(0);             // Sector count
        Port::<u8>::new(base + 3).write(0);             // LBA Low
        Port::<u8>::new(base + 4).write(SMART_LBA_MID); // LBA Mid = 0x4F
        Port::<u8>::new(base + 5).write(SMART_LBA_HI);  // LBA Hi = 0xC2
        Port::<u8>::new(base + 7).write(ATA_CMD_SMART);
    }

    // Wait for completion
    for _ in 0..1_000_000 {
        let s = unsafe { Port::<u8>::new(base + 7).read() };
        if s & 0x01 != 0 { return Err("SMART command error"); }
        if s & 0x80 == 0 { return Ok(()); }
        core::hint::spin_loop();
    }

    Err("SMART command timeout")
}

#[cfg(target_arch = "x86_64")]
fn ide_smart_return_status(base: u16, slave: bool) -> Result<bool, &'static str> {
    use crate::arch::Port;

    ide_smart_command(base, slave, SMART_RETURN_STATUS, 0)?;

    // After SMART RETURN STATUS, check LBA Mid/Hi:
    // 0x4F/0xC2 = healthy, 0xF4/0x2C = failing
    let mid = unsafe { Port::<u8>::new(base + 4).read() };
    let hi = unsafe { Port::<u8>::new(base + 5).read() };

    if mid == 0xF4 && hi == 0x2C {
        Ok(false) // Failing
    } else {
        Ok(true) // Healthy (0x4F/0xC2 or other)
    }
}

#[cfg(target_arch = "x86_64")]
fn ide_smart_read_data(base: u16, slave: bool) -> Result<[u8; 512], &'static str> {
    use crate::arch::Port;

    let drive_sel = if slave { 0xB0 } else { 0xA0 };
    unsafe {
        Port::<u8>::new(base + 6).write(drive_sel);
        for _ in 0..4 { let _ = Port::<u8>::new(base + 7).read(); }

        Port::<u8>::new(base + 1).write(SMART_READ_DATA);
        Port::<u8>::new(base + 2).write(0);
        Port::<u8>::new(base + 3).write(0);
        Port::<u8>::new(base + 4).write(SMART_LBA_MID);
        Port::<u8>::new(base + 5).write(SMART_LBA_HI);
        Port::<u8>::new(base + 7).write(ATA_CMD_SMART);
    }

    // Wait for DRQ
    for _ in 0..1_000_000 {
        let s = unsafe { Port::<u8>::new(base + 7).read() };
        if s & 0x01 != 0 { return Err("SMART read error"); }
        if s & 0x80 == 0 && s & 0x08 != 0 { break; }
        core::hint::spin_loop();
    }

    let mut buf = [0u8; 512];
    for i in (0..512).step_by(2) {
        let word = unsafe { Port::<u16>::new(base).read() };
        buf[i] = (word & 0xFF) as u8;
        buf[i + 1] = ((word >> 8) & 0xFF) as u8;
    }

    Ok(buf)
}

#[cfg(target_arch = "x86_64")]
fn ide_smart_read_thresholds(base: u16, slave: bool) -> Result<[u8; 512], &'static str> {
    use crate::arch::Port;

    let drive_sel = if slave { 0xB0 } else { 0xA0 };
    unsafe {
        Port::<u8>::new(base + 6).write(drive_sel);
        for _ in 0..4 { let _ = Port::<u8>::new(base + 7).read(); }

        Port::<u8>::new(base + 1).write(SMART_READ_THRESHOLDS);
        Port::<u8>::new(base + 2).write(0);
        Port::<u8>::new(base + 3).write(0);
        Port::<u8>::new(base + 4).write(SMART_LBA_MID);
        Port::<u8>::new(base + 5).write(SMART_LBA_HI);
        Port::<u8>::new(base + 7).write(ATA_CMD_SMART);
    }

    for _ in 0..1_000_000 {
        let s = unsafe { Port::<u8>::new(base + 7).read() };
        if s & 0x01 != 0 { return Err("SMART threshold read error"); }
        if s & 0x80 == 0 && s & 0x08 != 0 { break; }
        core::hint::spin_loop();
    }

    let mut buf = [0u8; 512];
    for i in (0..512).step_by(2) {
        let word = unsafe { Port::<u16>::new(base).read() };
        buf[i] = (word & 0xFF) as u8;
        buf[i + 1] = ((word >> 8) & 0xFF) as u8;
    }

    Ok(buf)
}

// ═══════════════════════════════════════════════════════════════════════════════
// AHCI SMART
// ═══════════════════════════════════════════════════════════════════════════════

fn probe_ahci_smart() -> Option<Vec<SmartData>> {
    if !crate::drivers::ahci::is_initialized() {
        return None;
    }

    let devices = crate::drivers::ahci::list_devices();
    if devices.is_empty() {
        return None;
    }

    let mut results = Vec::new();

    for dev in &devices {
        // Only SATA drives (not ATAPI)
        if dev.device_type != crate::drivers::ahci::AhciDeviceType::Sata {
            continue;
        }

        let drive_name = if dev.model.is_empty() || dev.model == "Unknown" {
            format!("AHCI Port {}", dev.port_num)
        } else {
            format!("{} (AHCI Port {})", dev.model, dev.port_num)
        };

        let mut data = SmartData {
            drive_name,
            smart_supported: true, // Assume supported for SATA drives
            smart_enabled: false,
            ..SmartData::default()
        };

        // Enable SMART via AHCI
        if ahci_smart_command(dev.port_num, SMART_ENABLE).is_ok() {
            data.smart_enabled = true;
        }

        // Check health
        if let Ok(healthy) = ahci_smart_return_status(dev.port_num) {
            data.health_ok = healthy;
        }

        // Read SMART data
        if let Ok(smart_buf) = ahci_smart_read_data(dev.port_num) {
            parse_smart_attributes(&smart_buf, &mut data);
        }

        // Read thresholds
        if let Ok(thresh_buf) = ahci_smart_read_thresholds(dev.port_num) {
            apply_thresholds(&thresh_buf, &mut data);
        }

        extract_key_metrics(&mut data);
        results.push(data);
    }

    if results.is_empty() { None } else { Some(results) }
}

/// Send a non-data SMART command via AHCI
fn ahci_smart_command(port_num: u8, feature: u8) -> Result<(), &'static str> {
    crate::drivers::ahci::send_smart_command(port_num, feature, false)
}

/// SMART RETURN STATUS via AHCI — checks LBA Mid/Hi after command
fn ahci_smart_return_status(port_num: u8) -> Result<bool, &'static str> {
    // SMART RETURN STATUS is special — drive sets LBA Mid/Hi to indicate health
    // We rely on the Task File Error bit for now
    match crate::drivers::ahci::send_smart_command(port_num, SMART_RETURN_STATUS, false) {
        Ok(()) => Ok(true),  // No error = healthy
        Err(_) => Ok(false), // Error = failing
    }
}

/// Read 512 bytes of SMART data via AHCI
fn ahci_smart_read_data(port_num: u8) -> Result<[u8; 512], &'static str> {
    crate::drivers::ahci::smart_read_data(port_num, SMART_READ_DATA)
}

/// Read 512 bytes of SMART thresholds via AHCI
fn ahci_smart_read_thresholds(port_num: u8) -> Result<[u8; 512], &'static str> {
    crate::drivers::ahci::smart_read_data(port_num, SMART_READ_THRESHOLDS)
}

// ═══════════════════════════════════════════════════════════════════════════════
// SMART Data Parsing (common to IDE and AHCI)
// ═══════════════════════════════════════════════════════════════════════════════

/// Parse SMART attribute table from 512-byte sector
fn parse_smart_attributes(buf: &[u8; 512], data: &mut SmartData) {
    // SMART data structure:
    // Offset 0-1: SMART data revision
    // Offset 2-361: 30 attribute entries, 12 bytes each
    // Offset 362: Offline data collection status
    // ...

    for i in 0..30 {
        let offset = 2 + i * 12;
        if offset + 12 > 362 { break; }

        let id = buf[offset];
        if id == 0 { continue; } // Empty entry

        let flags = u16::from_le_bytes([buf[offset + 1], buf[offset + 2]]);
        let current = buf[offset + 3];
        let worst = buf[offset + 4];
        let raw = u64::from_le_bytes([
            buf[offset + 5], buf[offset + 6], buf[offset + 7],
            buf[offset + 8], buf[offset + 9], buf[offset + 10],
            0, 0,
        ]);

        let pre_fail = flags & 1 != 0;

        data.attributes.push(SmartAttribute {
            id,
            name: smart_attribute_name(id),
            current,
            worst,
            threshold: 0, // Filled later from thresholds page
            raw_value: raw,
            pre_fail,
        });
    }
}

/// Apply thresholds from the threshold page
fn apply_thresholds(buf: &[u8; 512], data: &mut SmartData) {
    for i in 0..30 {
        let offset = 2 + i * 12;
        if offset + 2 > 362 { break; }

        let id = buf[offset];
        let threshold = buf[offset + 1];

        if id == 0 { continue; }

        if let Some(attr) = data.attributes.iter_mut().find(|a| a.id == id) {
            attr.threshold = threshold;
        }
    }
}

/// Extract key metrics from parsed attributes
fn extract_key_metrics(data: &mut SmartData) {
    for attr in &data.attributes {
        match attr.id {
            194 | 190 => {
                // Temperature — raw value is usually degrees C (low byte)
                data.temperature_c = Some((attr.raw_value & 0xFF) as i32);
            }
            9 => {
                data.power_on_hours = Some(attr.raw_value);
            }
            5 => {
                data.reallocated_sectors = Some(attr.raw_value);
            }
            197 => {
                data.pending_sectors = Some(attr.raw_value);
            }
            198 => {
                data.uncorrectable_sectors = Some(attr.raw_value);
            }
            12 => {
                data.power_cycle_count = Some(attr.raw_value);
            }
            _ => {}
        }
    }
}

/// SMART attribute name lookup
fn smart_attribute_name(id: u8) -> &'static str {
    match id {
        1 => "Raw Read Error Rate",
        2 => "Throughput Performance",
        3 => "Spin-Up Time",
        4 => "Start/Stop Count",
        5 => "Reallocated Sectors",
        7 => "Seek Error Rate",
        8 => "Seek Time Performance",
        9 => "Power-On Hours",
        10 => "Spin Retry Count",
        11 => "Calibration Retry Count",
        12 => "Power Cycle Count",
        13 => "Soft Read Error Rate",
        170 => "Available Reserve Space",
        171 => "Program Fail Count",
        172 => "Erase Fail Count",
        173 => "Wear Leveling Count",
        174 => "Unexpected Power Loss",
        175 => "Power Loss Protection",
        176 => "Erase Fail (Chip)",
        177 => "Wear Range Delta",
        178 => "Used Rsvd Block (Chip)",
        179 => "Used Rsvd Block (Total)",
        180 => "Unused Rsvd Block Total",
        181 => "Program Fail (Total)",
        182 => "Erase Fail (Total)",
        183 => "Runtime Bad Block",
        184 => "End-to-End Error",
        187 => "Reported Uncorrectable",
        188 => "Command Timeout",
        189 => "High Fly Writes",
        190 => "Airflow Temperature",
        191 => "G-Sense Error Rate",
        192 => "Unsafe Shutdown Count",
        193 => "Load Cycle Count",
        194 => "Temperature",
        195 => "HW ECC Recovered",
        196 => "Reallocation Events",
        197 => "Current Pending Sectors",
        198 => "Offline Uncorrectable",
        199 => "UDMA CRC Error Count",
        200 => "Multi-Zone Error Rate",
        201 => "Soft Read/Write Error",
        202 => "Data Address Mark Error",
        220 => "Disk Shift",
        222 => "Loaded Hours",
        223 => "Load/Unload Retry Count",
        224 => "Load Friction",
        226 => "Load-In Time",
        240 => "Head Flying Hours",
        241 => "Total LBAs Written",
        242 => "Total LBAs Read",
        254 => "Free Fall Protection",
        _ => "(Unknown)",
    }
}
