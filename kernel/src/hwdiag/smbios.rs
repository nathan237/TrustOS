//! SMBIOS/DMI — System Management BIOS table parser
//!
//! Scans memory for SM3/_SM_ anchor, parses SMBIOS structures to identify:
//! - BIOS vendor, version, release date
//! - System manufacturer, product name, serial number, UUID
//! - Baseboard (motherboard) manufacturer, product, serial
//! - Chassis type, manufacturer
//! - Processor socket/frequency info
//! - Memory module details (type, speed, manufacturer, size per slot)

use alloc::string::String;
use alloc::format;
use alloc::vec::Vec;
use super::dbg_out;

/// SMBIOS entry point structure info
#[derive(Debug, Clone)]
pub struct SmbiosInfo {
    pub version_major: u8,
    pub version_minor: u8,
    pub bios: BiosInfo,
    pub system: SystemInfo,
    pub baseboard: BaseboardInfo,
    pub chassis: ChassisInfo,
    pub processors: Vec<ProcessorInfo>,
    pub memory_devices: Vec<MemoryDeviceInfo>,
}

#[derive(Debug, Clone, Default)]
pub struct BiosInfo {
    pub vendor: String,
    pub version: String,
    pub release_date: String,
    pub rom_size_kb: u32,
}

#[derive(Debug, Clone, Default)]
pub struct SystemInfo {
    pub manufacturer: String,
    pub product_name: String,
    pub version: String,
    pub serial_number: String,
    pub uuid: [u8; 16],
    pub sku: String,
    pub family: String,
}

#[derive(Debug, Clone, Default)]
pub struct BaseboardInfo {
    pub manufacturer: String,
    pub product: String,
    pub version: String,
    pub serial_number: String,
}

#[derive(Debug, Clone, Default)]
pub struct ChassisInfo {
    pub manufacturer: String,
    pub chassis_type: u8,
    pub serial_number: String,
}

#[derive(Debug, Clone, Default)]
pub struct ProcessorInfo {
    pub socket: String,
    pub manufacturer: String,
    pub version: String,
    pub max_speed_mhz: u16,
    pub current_speed_mhz: u16,
    pub core_count: u8,
    pub thread_count: u8,
}

#[derive(Debug, Clone, Default)]
pub struct MemoryDeviceInfo {
    pub locator: String,
    pub bank: String,
    pub size_mb: u32,
    pub form_factor: u8,
    pub memory_type: u8,
    pub speed_mhz: u16,
    pub manufacturer: String,
    pub serial_number: String,
    pub part_number: String,
}

/// SMBIOS Entry Point signatures
const SM3_ANCHOR: &[u8; 5] = b"_SM3_";
const SM_ANCHOR: &[u8; 4] = b"_SM_";

/// Cached SMBIOS info
static SMBIOS_CACHE: spin::Once<Option<SmbiosInfo>> = spin::Once::new();

/// Get cached SMBIOS info (call init first)
pub fn get_info() -> Option<&'static SmbiosInfo> {
    SMBIOS_CACHE.get().and_then(|opt| opt.as_ref())
}

/// Initialize SMBIOS by scanning memory
pub fn init() {
    SMBIOS_CACHE.call_once(|| scan_and_parse());
}

/// Run hwdiag SMBIOS dump command
pub fn run(args: &[&str]) {
    let verbose = args.contains(&"-v") || args.contains(&"--verbose");

    dbg_out!("[SMBIOS] === SMBIOS / DMI Table Dump ===");

    // Ensure initialized
    init();

    match get_info() {
        Some(info) => dump_info(info, verbose),
        None => {
            dbg_out!("[SMBIOS] No SMBIOS entry point found in memory");
            dbg_out!("[SMBIOS] (Expected on UEFI/BIOS systems — may be absent in VMs)");
        }
    }
}

fn dump_info(info: &SmbiosInfo, verbose: bool) {
    dbg_out!("[SMBIOS] Version: {}.{}", info.version_major, info.version_minor);
    dbg_out!("");

    // BIOS
    dbg_out!("[BIOS] Vendor:       {}", info.bios.vendor);
    dbg_out!("[BIOS] Version:      {}", info.bios.version);
    dbg_out!("[BIOS] Release Date: {}", info.bios.release_date);
    if info.bios.rom_size_kb > 0 {
        dbg_out!("[BIOS] ROM Size:     {} KB", info.bios.rom_size_kb);
    }
    dbg_out!("");

    // System
    dbg_out!("[SYSTEM] Manufacturer: {}", info.system.manufacturer);
    dbg_out!("[SYSTEM] Product:      {}", info.system.product_name);
    dbg_out!("[SYSTEM] Version:      {}", info.system.version);
    dbg_out!("[SYSTEM] Serial:       {}", info.system.serial_number);
    dbg_out!("[SYSTEM] UUID:         {}", format_uuid(&info.system.uuid));
    if !info.system.sku.is_empty() {
        dbg_out!("[SYSTEM] SKU:          {}", info.system.sku);
    }
    if !info.system.family.is_empty() {
        dbg_out!("[SYSTEM] Family:       {}", info.system.family);
    }
    dbg_out!("");

    // Baseboard
    dbg_out!("[BOARD] Manufacturer:  {}", info.baseboard.manufacturer);
    dbg_out!("[BOARD] Product:       {}", info.baseboard.product);
    dbg_out!("[BOARD] Version:       {}", info.baseboard.version);
    dbg_out!("[BOARD] Serial:        {}", info.baseboard.serial_number);
    dbg_out!("");

    // Chassis
    dbg_out!("[CHASSIS] Manufacturer: {}", info.chassis.manufacturer);
    dbg_out!("[CHASSIS] Type:         {} ({})", chassis_type_name(info.chassis.chassis_type), info.chassis.chassis_type);
    dbg_out!("[CHASSIS] Serial:       {}", info.chassis.serial_number);
    dbg_out!("");

    // Processors
    for (i, proc) in info.processors.iter().enumerate() {
        dbg_out!("[CPU #{}] Socket:       {}", i, proc.socket);
        dbg_out!("[CPU #{}] Manufacturer: {}", i, proc.manufacturer);
        dbg_out!("[CPU #{}] Version:      {}", i, proc.version);
        dbg_out!("[CPU #{}] Max Speed:    {} MHz", i, proc.max_speed_mhz);
        dbg_out!("[CPU #{}] Current:      {} MHz", i, proc.current_speed_mhz);
        dbg_out!("[CPU #{}] Cores/Threads:{} / {}", i, proc.core_count, proc.thread_count);
        dbg_out!("");
    }

    // Memory Devices
    if !info.memory_devices.is_empty() {
        dbg_out!("[MEMORY] === Memory Slots ===");
        for (i, mem) in info.memory_devices.iter().enumerate() {
            if mem.size_mb == 0 && !verbose {
                continue; // Skip empty slots unless verbose
            }
            dbg_out!("[DIMM #{}] Locator:    {}", i, mem.locator);
            dbg_out!("[DIMM #{}] Bank:       {}", i, mem.bank);
            if mem.size_mb > 0 {
                dbg_out!("[DIMM #{}] Size:       {} MB", i, mem.size_mb);
                dbg_out!("[DIMM #{}] Type:       {}", i, memory_type_name(mem.memory_type));
                dbg_out!("[DIMM #{}] Speed:      {} MHz", i, mem.speed_mhz);
                dbg_out!("[DIMM #{}] Manufacturer:{}", i, mem.manufacturer);
                dbg_out!("[DIMM #{}] Part:       {}", i, mem.part_number);
                dbg_out!("[DIMM #{}] Serial:     {}", i, mem.serial_number);
            } else {
                dbg_out!("[DIMM #{}] (empty slot)", i);
            }
            dbg_out!("");
        }
    }
}

/// Scan for SMBIOS entry point in standard memory regions
fn scan_and_parse() -> Option<SmbiosInfo> {
    let hhdm = crate::memory::hhdm_offset();

    // SMBIOS 3.0 (64-bit) or 2.x entry point lives in:
    // 1) UEFI: EFI System Configuration Table (Limine passes RSDP but not SMBIOS directly)
    // 2) Legacy BIOS: scan 0xF0000-0xFFFFF (paragraph-aligned, every 16 bytes)
    // 3) Some UEFI: also at 0xF0000

    // Scan legacy BIOS region (0xF0000 - 0xFFFFF)
    let scan_start: u64 = 0xF0000;
    let scan_end: u64 = 0x100000;

    // Map the scan region
    if crate::memory::map_mmio(scan_start, (scan_end - scan_start) as usize).is_err() {
        crate::serial_println!("[SMBIOS] Failed to map scan region");
        return None;
    }

    let mut addr = scan_start;
    while addr < scan_end - 32 {
        let virt = crate::memory::phys_to_virt(addr);
        let ptr = virt as *const u8;

        // Check for _SM3_ (SMBIOS 3.0+)
        if unsafe { core::slice::from_raw_parts(ptr, 5) } == SM3_ANCHOR {
            crate::serial_println!("[SMBIOS] Found SM3 entry point at {:#x}", addr);
            if let Some(info) = parse_sm3_entry(virt) {
                return Some(info);
            }
        }

        // Check for _SM_ (SMBIOS 2.x)
        if unsafe { core::slice::from_raw_parts(ptr, 4) } == SM_ANCHOR {
            crate::serial_println!("[SMBIOS] Found SM entry point at {:#x}", addr);
            if let Some(info) = parse_sm2_entry(virt) {
                return Some(info);
            }
        }

        addr += 16; // Paragraph-aligned
    }

    crate::serial_println!("[SMBIOS] No entry point found in 0xF0000-0xFFFFF");
    None
}

/// Parse SMBIOS 3.0 (64-bit) Entry Point Structure
fn parse_sm3_entry(virt: u64) -> Option<SmbiosInfo> {
    let ptr = virt as *const u8;

    // Structure:
    //  0: anchor "_SM3_" (5 bytes)
    //  5: checksum (1)
    //  6: entry point length (1)
    //  7: major version (1)
    //  8: minor version (1)
    //  9: docrev (1)
    // 10: entry point revision (1)
    // 11: reserved (1)
    // 12: structure table max size (4, LE)
    // 16: structure table address (8, LE)

    let length = unsafe { *ptr.add(6) } as usize;
    if length < 24 { return None; }

    // Verify checksum
    let sum: u8 = (0..length).map(|i| unsafe { *ptr.add(i) }).fold(0u8, |a, b| a.wrapping_add(b));
    if sum != 0 {
        crate::serial_println!("[SMBIOS] SM3 checksum failed");
        return None;
    }

    let major = unsafe { *ptr.add(7) };
    let minor = unsafe { *ptr.add(8) };
    let table_length = unsafe { *(ptr.add(12) as *const u32) } as usize;
    let table_phys = unsafe { *(ptr.add(16) as *const u64) };

    crate::serial_println!("[SMBIOS] v{}.{}, table at {:#x} ({} bytes)", major, minor, table_phys, table_length);

    parse_structure_table(table_phys, table_length, major, minor)
}

/// Parse SMBIOS 2.x Entry Point Structure
fn parse_sm2_entry(virt: u64) -> Option<SmbiosInfo> {
    let ptr = virt as *const u8;

    // Structure:
    //  0: anchor "_SM_" (4 bytes)
    //  4: checksum (1)
    //  5: entry point length (1)
    //  6: major version (1)
    //  7: minor version (1)
    //  8: max structure size (2, LE)
    // 10: entry point revision (1)
    // 11: formatted area (5 bytes)
    // 16: intermediate anchor "_DMI_" (5 bytes)
    // 21: intermediate checksum (1)
    // 22: structure table length (2, LE)
    // 24: structure table address (4, LE)
    // 28: number of structures (2, LE)

    let length = unsafe { *ptr.add(5) } as usize;
    if length < 30 { return None; }

    // Verify checksum
    let sum: u8 = (0..length).map(|i| unsafe { *ptr.add(i) }).fold(0u8, |a, b| a.wrapping_add(b));
    if sum != 0 {
        crate::serial_println!("[SMBIOS] SM2 checksum failed");
        return None;
    }

    let major = unsafe { *ptr.add(6) };
    let minor = unsafe { *ptr.add(7) };
    let table_length = unsafe { *(ptr.add(22) as *const u16) } as usize;
    let table_phys = unsafe { *(ptr.add(24) as *const u32) } as u64;

    crate::serial_println!("[SMBIOS] v{}.{}, table at {:#x} ({} bytes)", major, minor, table_phys, table_length);

    parse_structure_table(table_phys, table_length, major, minor)
}

/// Parse the SMBIOS structure table
fn parse_structure_table(table_phys: u64, table_length: usize, major: u8, minor: u8) -> Option<SmbiosInfo> {
    if table_length == 0 || table_length > 256 * 1024 {
        return None;
    }

    // Map the structure table
    if crate::memory::map_mmio(table_phys, table_length).is_err() {
        crate::serial_println!("[SMBIOS] Failed to map structure table");
        return None;
    }

    let table_virt = crate::memory::phys_to_virt(table_phys);
    let table_base = table_virt as *const u8;

    let mut info = SmbiosInfo {
        version_major: major,
        version_minor: minor,
        bios: BiosInfo::default(),
        system: SystemInfo::default(),
        baseboard: BaseboardInfo::default(),
        chassis: ChassisInfo::default(),
        processors: Vec::new(),
        memory_devices: Vec::new(),
    };

    let mut offset: usize = 0;

    // Walk through all structures
    while offset + 4 < table_length {
        let hdr = unsafe { table_base.add(offset) };
        let struct_type = unsafe { *hdr };
        let struct_length = unsafe { *hdr.add(1) } as usize;

        if struct_length < 4 { break; } // Invalid

        // Collect string table (after formatted area, double-null terminated)
        let string_start = offset + struct_length;
        let strings = extract_strings(table_base, string_start, table_length);

        // Parse known structure types
        match struct_type {
            0 => parse_bios(hdr, struct_length, &strings, &mut info.bios),
            1 => parse_system(hdr, struct_length, &strings, &mut info.system),
            2 => parse_baseboard(hdr, struct_length, &strings, &mut info.baseboard),
            3 => parse_chassis(hdr, struct_length, &strings, &mut info.chassis),
            4 => {
                let mut proc = ProcessorInfo::default();
                parse_processor(hdr, struct_length, &strings, &mut proc);
                info.processors.push(proc);
            }
            17 => {
                let mut mem = MemoryDeviceInfo::default();
                parse_memory_device(hdr, struct_length, &strings, &mut mem);
                info.memory_devices.push(mem);
            }
            127 => break, // End-of-table
            _ => {} // Skip unknown types
        }

        // Advance past formatted area + string table (double-null terminated)
        offset = find_next_structure(table_base, string_start, table_length);
        if offset == 0 { break; }
    }

    Some(info)
}

/// Extract null-terminated strings from the string table
fn extract_strings(base: *const u8, start: usize, limit: usize) -> Vec<String> {
    let mut strings = Vec::new();
    let mut pos = start;

    loop {
        if pos >= limit { break; }

        let mut s = String::new();
        while pos < limit {
            let b = unsafe { *base.add(pos) };
            pos += 1;
            if b == 0 { break; }
            if b >= 0x20 && b < 0x7F {
                s.push(b as char);
            }
        }

        if s.is_empty() {
            break; // Double-null = end of strings
        }
        strings.push(s);
    }

    strings
}

/// Find the start of the next structure (past the string table)
fn find_next_structure(base: *const u8, string_start: usize, limit: usize) -> usize {
    let mut pos = string_start;

    // Walk past strings until double-null
    loop {
        if pos + 1 >= limit { return 0; }
        let b0 = unsafe { *base.add(pos) };
        let b1 = unsafe { *base.add(pos + 1) };
        if b0 == 0 && b1 == 0 {
            return pos + 2;
        }
        pos += 1;
    }
}

/// Get string by 1-based index from string table
fn get_string(strings: &[String], index: u8) -> String {
    if index == 0 { return String::new(); }
    strings.get((index - 1) as usize).cloned().unwrap_or_default()
}

// ─── Structure Parsers ─────────────────────────────────────────────────────

/// Type 0: BIOS Information
fn parse_bios(hdr: *const u8, len: usize, strings: &[String], info: &mut BiosInfo) {
    if len < 18 { return; }
    info.vendor = get_string(strings, unsafe { *hdr.add(4) });
    info.version = get_string(strings, unsafe { *hdr.add(5) });
    info.release_date = get_string(strings, unsafe { *hdr.add(8) });
    let rom_size_raw = unsafe { *hdr.add(9) };
    info.rom_size_kb = (rom_size_raw as u32 + 1) * 64;
}

/// Type 1: System Information
fn parse_system(hdr: *const u8, len: usize, strings: &[String], info: &mut SystemInfo) {
    if len < 8 { return; }
    info.manufacturer = get_string(strings, unsafe { *hdr.add(4) });
    info.product_name = get_string(strings, unsafe { *hdr.add(5) });
    info.version = get_string(strings, unsafe { *hdr.add(6) });
    info.serial_number = get_string(strings, unsafe { *hdr.add(7) });
    if len >= 25 {
        unsafe {
            core::ptr::copy_nonoverlapping(hdr.add(8), info.uuid.as_mut_ptr(), 16);
        }
    }
    if len >= 26 {
        info.sku = get_string(strings, unsafe { *hdr.add(25) });
    }
    if len >= 27 {
        info.family = get_string(strings, unsafe { *hdr.add(26) });
    }
}

/// Type 2: Baseboard Information
fn parse_baseboard(hdr: *const u8, len: usize, strings: &[String], info: &mut BaseboardInfo) {
    if len < 8 { return; }
    info.manufacturer = get_string(strings, unsafe { *hdr.add(4) });
    info.product = get_string(strings, unsafe { *hdr.add(5) });
    info.version = get_string(strings, unsafe { *hdr.add(6) });
    info.serial_number = get_string(strings, unsafe { *hdr.add(7) });
}

/// Type 3: Chassis Information
fn parse_chassis(hdr: *const u8, len: usize, strings: &[String], info: &mut ChassisInfo) {
    if len < 9 { return; }
    info.manufacturer = get_string(strings, unsafe { *hdr.add(4) });
    info.chassis_type = unsafe { *hdr.add(5) } & 0x7F;
    if len >= 8 {
        info.serial_number = get_string(strings, unsafe { *hdr.add(7) });
    }
}

/// Type 4: Processor Information
fn parse_processor(hdr: *const u8, len: usize, strings: &[String], info: &mut ProcessorInfo) {
    if len < 26 { return; }
    info.socket = get_string(strings, unsafe { *hdr.add(4) });
    info.manufacturer = get_string(strings, unsafe { *hdr.add(7) });
    info.version = get_string(strings, unsafe { *hdr.add(16) });
    info.max_speed_mhz = unsafe { *(hdr.add(20) as *const u16) };
    info.current_speed_mhz = unsafe { *(hdr.add(22) as *const u16) };
    if len >= 36 {
        info.core_count = unsafe { *hdr.add(35) };
    }
    if len >= 38 {
        info.thread_count = unsafe { *hdr.add(37) };
    }
}

/// Type 17: Memory Device
fn parse_memory_device(hdr: *const u8, len: usize, strings: &[String], info: &mut MemoryDeviceInfo) {
    if len < 21 { return; }
    info.form_factor = unsafe { *hdr.add(14) };
    info.locator = get_string(strings, unsafe { *hdr.add(16) });
    info.bank = get_string(strings, unsafe { *hdr.add(17) });
    info.memory_type = unsafe { *hdr.add(18) };

    // Size (word at offset 12)
    let size_raw = unsafe { *(hdr.add(12) as *const u16) };
    if size_raw == 0xFFFF || size_raw == 0x7FFF {
        // Extended size at offset 28 (SMBIOS 2.7+)
        if len >= 32 {
            info.size_mb = unsafe { *(hdr.add(28) as *const u32) } & 0x7FFFFFFF;
        }
    } else if size_raw != 0 {
        if size_raw & 0x8000 != 0 {
            // Size in KB
            info.size_mb = (size_raw & 0x7FFF) as u32 / 1024;
        } else {
            info.size_mb = size_raw as u32;
        }
    }

    if len >= 22 {
        info.speed_mhz = unsafe { *(hdr.add(21) as *const u16) };
    }
    if len >= 24 {
        info.manufacturer = get_string(strings, unsafe { *hdr.add(23) });
    }
    if len >= 25 {
        info.serial_number = get_string(strings, unsafe { *hdr.add(24) });
    }
    if len >= 27 {
        info.part_number = get_string(strings, unsafe { *hdr.add(26) });
    }
}

// ─── Name Lookup Tables ────────────────────────────────────────────────────

pub fn chassis_type_name(t: u8) -> &'static str {
    match t {
        1 => "Other",
        2 => "Unknown",
        3 => "Desktop",
        4 => "Low Profile Desktop",
        5 => "Pizza Box",
        6 => "Mini Tower",
        7 => "Tower",
        8 => "Portable",
        9 => "Laptop",
        10 => "Notebook",
        11 => "Hand Held",
        12 => "Docking Station",
        13 => "All in One",
        14 => "Sub Notebook",
        15 => "Space-saving",
        16 => "Lunch Box",
        17 => "Main Server Chassis",
        23 => "Rack Mount",
        24 => "Sealed-case PC",
        30 => "Tablet",
        31 => "Convertible",
        32 => "Detachable",
        _ => "Unknown",
    }
}

pub fn memory_type_name(t: u8) -> &'static str {
    match t {
        1 => "Other",
        2 => "Unknown",
        3 => "DRAM",
        4 => "EDRAM",
        5 => "VRAM",
        6 => "SRAM",
        7 => "RAM",
        8 => "ROM",
        9 => "FLASH",
        10 => "EEPROM",
        11 => "FEPROM",
        12 => "EPROM",
        13 => "CDRAM",
        14 => "3DRAM",
        15 => "SDRAM",
        16 => "SGRAM",
        17 => "RDRAM",
        18 => "DDR",
        19 => "DDR2",
        20 => "DDR2 FB-DIMM",
        24 => "DDR3",
        26 => "DDR4",
        30 => "LPDDR4",
        34 => "DDR5",
        35 => "LPDDR5",
        _ => "Unknown",
    }
}

fn format_uuid(uuid: &[u8; 16]) -> String {
    // SMBIOS encodes UUID in mixed-endian format
    format!("{:02X}{:02X}{:02X}{:02X}-{:02X}{:02X}-{:02X}{:02X}-{:02X}{:02X}-{:02X}{:02X}{:02X}{:02X}{:02X}{:02X}",
        uuid[3], uuid[2], uuid[1], uuid[0],
        uuid[5], uuid[4],
        uuid[7], uuid[6],
        uuid[8], uuid[9],
        uuid[10], uuid[11], uuid[12], uuid[13], uuid[14], uuid[15])
}
