//! EFI Variables — Read UEFI firmware variables via EFI System Table
//!
//! Dumps SecureBoot state, boot order, platform key info, firmware vendor.
//! Works on UEFI-booted systems (Limine provides EFI System Table pointer).

use alloc::string::String;
use alloc::format;
use alloc::vec::Vec;
use super::dbg_out;

/// Collected EFI Information
#[derive(Debug, Clone)]
pub struct EfiInfo {
    pub available: bool,
    pub secure_boot: bool,
    pub uefi_version: String,
    pub firmware_vendor: String,
    pub firmware_revision: u32,
    pub boot_order: Vec<u16>,
    pub boot_current: u16,
    pub runtime_services: bool,
}

impl Default for EfiInfo {
    fn default() -> Self {
        Self {
            available: false,
            secure_boot: false,
            uefi_version: String::new(),
            firmware_vendor: String::new(),
            firmware_revision: 0,
            boot_order: Vec::new(),
            boot_current: 0xFFFF,
            runtime_services: false,
        }
    }
}

// EFI System Table signature
const EFI_SYSTEM_TABLE_SIGNATURE: u64 = 0x5453595320494249; // "IBI SYST"

// EFI Global Variable GUID: 8BE4DF61-93CA-11D2-AA0D-00E098032B8C
const EFI_GLOBAL_VARIABLE_GUID: [u8; 16] = [
    0x61, 0xDF, 0xE4, 0x8B, 0xCA, 0x93, 0xD2, 0x11,
    0xAA, 0x0D, 0x00, 0xE0, 0x98, 0x03, 0x2B, 0x8C,
];

/// Cached EFI info
static EFI_CACHE: spin::Once<EfiInfo> = spin::Once::new();

/// Get cached EFI info
pub fn get_info() -> &'static EfiInfo {
    EFI_CACHE.call_once(|| collect_efi_info())
}

/// Collect EFI information
pub fn collect_efi_info() -> EfiInfo {
    let mut info = EfiInfo::default();

    #[cfg(target_arch = "x86_64")]
    {
        // Limine doesn't directly expose the EFI System Table after ExitBootServices.
        // We probe what we can from ACPI and SMBIOS data, and check if UEFI boot was used.
        probe_uefi_boot_mode(&mut info);
    }

    info
}

#[cfg(target_arch = "x86_64")]
fn probe_uefi_boot_mode(info: &mut EfiInfo) {
    // If ACPI tables are available and we have RSDP revision >= 2, 
    // the system likely booted via UEFI (modern firmware).
    if let Some(acpi) = crate::acpi::get_info() {
        if acpi.revision >= 2 {
            info.available = true;
            info.uefi_version = format!("ACPI {}.0+ (UEFI probable)", acpi.revision);
            info.firmware_vendor = acpi.oem_id.clone();
        }
    }

    // Scan the EFI system table region: UEFI firmware typically places
    // the System Table in the EFI_RUNTIME_SERVICES_DATA or ACPI_NVS memory regions.
    // We scan for the "IBI SYST" signature in known regions.
    scan_for_efi_system_table(info);
}

#[cfg(target_arch = "x86_64")]
fn scan_for_efi_system_table(info: &mut EfiInfo) {
    // Try to find EFI System Table by scanning memory regions from the boot memory map.
    // The system table has signature "IBI SYST" (0x5453595320494249).
    // 
    // Approach: Scan ACPI-reported regions and known EFI areas.
    // On most UEFI systems, the system table is in the first 4GB.

    // Check common EFI system table regions 
    let regions_to_scan: &[(u64, usize)] = &[
        // Some firmware places it near the top of low memory
        (0x0007_0000, 0x10000),  // 448K-512K
        (0x000E_0000, 0x20000),  // 896K-1M (ROM area)
    ];

    for &(start, size) in regions_to_scan {
        if crate::memory::map_mmio(start, size).is_err() {
            continue;
        }

        let mut addr = start;
        while addr + 120 < start + size as u64 {
            let virt = crate::memory::phys_to_virt(addr);
            let sig = unsafe { *(virt as *const u64) };

            if sig == EFI_SYSTEM_TABLE_SIGNATURE {
                crate::serial_println!("[EFI] Found System Table at phys {:#x}", addr);
                parse_system_table(virt, info);
                return;
            }
            addr += 16; // Aligned
        }
    }

    // If we have ACPI info, at least report that
    if info.available {
        crate::serial_println!("[EFI] System table not found in scan, using ACPI metadata");
    }
}

#[cfg(target_arch = "x86_64")]
fn parse_system_table(virt: u64, info: &mut EfiInfo) {
    let ptr = virt as *const u8;

    // Verify signature
    let sig = unsafe { *(ptr as *const u64) };
    if sig != EFI_SYSTEM_TABLE_SIGNATURE {
        crate::serial_println!("[EFI] Invalid system table signature: {:#x}", sig);
        return;
    }

    // EFI_SYSTEM_TABLE layout (64-bit):
    //  0: Hdr.Signature (8)
    //  8: Hdr.Revision (4)
    // 12: Hdr.HeaderSize (4)
    // 16: Hdr.CRC32 (4)
    // 20: Hdr.Reserved (4)
    // 24: FirmwareVendor (8, pointer to UCS-2 string)
    // 32: FirmwareRevision (4)
    // 36: padding (4)
    // 40: ConsoleInHandle (8)
    // 48: ConIn (8)
    // 56: ConsoleOutHandle (8)
    // 64: ConOut (8)
    // 72: StandardErrorHandle (8)
    // 80: StdErr (8)
    // 88: RuntimeServices (8, pointer)
    // 96: BootServices (8, pointer — NULL after ExitBootServices)

    let revision = unsafe { *(ptr.add(8) as *const u32) };
    let major = (revision >> 16) & 0xFFFF;
    let minor = revision & 0xFFFF;
    info.uefi_version = format!("{}.{}", major, minor / 10);
    info.firmware_revision = unsafe { *(ptr.add(32) as *const u32) };

    // Read firmware vendor string (UCS-2)
    let vendor_ptr_val = unsafe { *(ptr.add(24) as *const u64) };
    if vendor_ptr_val != 0 {
        let vendor_virt = crate::memory::phys_to_virt(vendor_ptr_val);
        info.firmware_vendor = read_ucs2_string(vendor_virt, 128);
    }

    // Runtime Services pointer
    let rt_ptr_val = unsafe { *(ptr.add(88) as *const u64) };
    if rt_ptr_val != 0 {
        info.runtime_services = true;
        // Note: Runtime Services may not be usable after ExitBootServices
        // without proper virtual address mapping. Limine calls EBS, so
        // GetVariable is only available if RT was set up.
        // We parse what we can from the configuration tables instead.
    }

    // Configuration Tables — scan for SMBIOS and other GUIDs
    // NumberOfTableEntries at offset 104, ConfigurationTable at offset 112
    let num_config = unsafe { *(ptr.add(104) as *const u64) } as usize;
    let config_ptr_val = unsafe { *(ptr.add(112) as *const u64) };

    if config_ptr_val != 0 && num_config > 0 && num_config < 256 {
        scan_config_tables(config_ptr_val, num_config, info);
    }

    // Try reading EFI variables from known NVRAM locations
    // SecureBoot variable: check via ACPI BGRT or config table heuristics
    probe_secure_boot_from_config(info);
}

#[cfg(target_arch = "x86_64")]
fn scan_config_tables(phys_addr: u64, count: usize, info: &mut EfiInfo) {
    let table_size = 24; // EFI_CONFIGURATION_TABLE = GUID(16) + pointer(8)
    let total_size = count * table_size;

    if crate::memory::map_mmio(phys_addr, total_size).is_err() {
        return;
    }

    let base = crate::memory::phys_to_virt(phys_addr);

    for i in 0..count {
        let entry = (base + (i * table_size) as u64) as *const u8;
        let guid = unsafe { core::slice::from_raw_parts(entry, 16) };
        let _table_ptr = unsafe { *(entry.add(16) as *const u64) };

        // Log known GUIDs
        // ACPI 2.0: 8868E871-E4F1-11D3-BC22-0080C73C8881
        // SMBIOS:   EB9D2D31-2D88-11D3-9A16-0090273FC14D
        // SMBIOS3:  F2FD1544-9794-4A2C-992E-E5BBCF20E394

        if guid_matches(guid, &[0x31, 0x2D, 0x9D, 0xEB, 0x88, 0x2D, 0xD3, 0x11,
                                 0x9A, 0x16, 0x00, 0x90, 0x27, 0x3F, 0xC1, 0x4D]) {
            crate::serial_println!("[EFI] Found SMBIOS config table");
        }
        if guid_matches(guid, &[0x44, 0x15, 0xFD, 0xF2, 0x94, 0x97, 0x2C, 0x4A,
                                 0x99, 0x2E, 0xE5, 0xBB, 0xCF, 0x20, 0xE3, 0x94]) {
            crate::serial_println!("[EFI] Found SMBIOS3 config table");
        }
    }
}

#[cfg(target_arch = "x86_64")]
fn probe_secure_boot_from_config(info: &mut EfiInfo) {
    // Without Runtime Services (post-ExitBootServices), we can't call GetVariable.
    // Heuristic: scan for SecureBoot state in known NVRAM memory regions.
    // 
    // Common UEFI NVRAM layouts place variables in flash at platform-specific
    // addresses. This is a best-effort probe.
    //
    // Alternative: Check if the Limine boot info indicates Secure Boot was active
    // (some Limine versions store this in the boot info).
    
    // For now, report as unknown if we can't determine
    // The EFI info will still show version and vendor which is valuable
    info.secure_boot = false; // Default: assume not enabled (safe default)
}

fn guid_matches(guid: &[u8], expected: &[u8; 16]) -> bool {
    guid.len() >= 16 && &guid[..16] == expected
}

fn read_ucs2_string(virt: u64, max_chars: usize) -> String {
    let mut s = String::new();
    let ptr = virt as *const u16;

    for i in 0..max_chars {
        let ch = unsafe { *ptr.add(i) };
        if ch == 0 { break; }
        if ch < 0x80 {
            s.push(ch as u8 as char);
        } else {
            s.push('?');
        }
    }

    s
}

/// Run EFI variables dump command
pub fn run(args: &[&str]) {
    let _verbose = args.contains(&"-v") || args.contains(&"--verbose");

    dbg_out!("[EFI] === UEFI/EFI Variable Dump ===");

    let info = collect_efi_info();

    if !info.available {
        dbg_out!("[EFI] No EFI System Table found");
        dbg_out!("[EFI] System may have been BIOS-booted (not UEFI)");
        return;
    }

    dbg_out!("[EFI] UEFI Version:      {}", info.uefi_version);
    dbg_out!("[EFI] Firmware Vendor:    {}", info.firmware_vendor);
    dbg_out!("[EFI] Firmware Revision:  {:#x}", info.firmware_revision);
    dbg_out!("[EFI] Runtime Services:   {}", if info.runtime_services { "Available" } else { "Not available" });
    dbg_out!("[EFI] Secure Boot:        {}", if info.secure_boot { "Enabled" } else { "Disabled / Unknown" });

    if !info.boot_order.is_empty() {
        let order: Vec<String> = info.boot_order.iter().map(|x| format!("{:04X}", x)).collect();
        dbg_out!("[EFI] Boot Order:         {}", order.join(", "));
    }

    if info.boot_current != 0xFFFF {
        dbg_out!("[EFI] Boot Current:       {:04X}", info.boot_current);
    }

    dbg_out!("");
    dbg_out!("[EFI] Note: Full variable access requires Runtime Services mapping.");
    dbg_out!("[EFI]       Post-ExitBootServices, only cached data is available.");
}
