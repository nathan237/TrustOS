//! ACPI (Advanced Configuration and Power Interface) Support
//!
//! This module parses ACPI tables to detect:
//! - MADT: Multi-processor configuration (APIC, I/O APIC, CPUs)
//! - FADT: Power management features
//! - MCFG: PCI Express configuration
//! - HPET: High Precision Event Timer

pub mod tables;
pub mod madt;
pub mod fadt;
pub mod mcfg;
pub mod hpet;

use alloc::vec::Vec;
use alloc::string::String;
use spin::Once;
use core::ptr;

/// ACPI information collected from tables
#[derive(Debug, Clone)]
pub struct AcpiInfo {
    /// ACPI revision (1 = ACPI 1.0, 2+ = ACPI 2.0+)
    pub revision: u8,
    /// OEM ID from RSDP
    pub oem_id: String,
    /// List of CPU Local APICs
    pub local_apics: Vec<madt::LocalApic>,
    /// List of I/O APICs
    pub io_apics: Vec<madt::IoApic>,
    /// List of interrupt source overrides
    pub int_overrides: Vec<madt::IntSourceOverride>,
    /// Local APIC NMI configurations (MADT type 4)
    pub local_apic_nmis: Vec<madt::LocalApicNmiInfo>,
    /// Local APIC address
    pub local_apic_addr: u64,
    /// FADT information
    pub fadt: Option<fadt::FadtInfo>,
    /// PCIe MCFG regions
    pub mcfg_regions: Vec<mcfg::McfgEntry>,
    /// HPET information
    pub hpet: Option<hpet::HpetInfo>,
    /// Number of CPUs detected
    pub cpu_count: usize,
}

impl Default for AcpiInfo {
    fn default() -> Self {
        Self {
            revision: 0,
            oem_id: String::new(),
            local_apics: Vec::new(),
            io_apics: Vec::new(),
            int_overrides: Vec::new(),
            local_apic_nmis: Vec::new(),
            local_apic_addr: 0xFEE0_0000,
            fadt: None,
            mcfg_regions: Vec::new(),
            hpet: None,
            cpu_count: 1,
        }
    }
}

static ACPI_INFO: Once<AcpiInfo> = Once::new();

/// Get ACPI information (must call init first)
pub fn get_info() -> Option<&'static AcpiInfo> {
    ACPI_INFO.get()
}

/// Initialize ACPI directly from Limine's pointer (already mapped)
pub fn init_direct(rsdp_ptr: u64) -> bool {
    if rsdp_ptr == 0 {
        crate::serial_println!("[ACPI] No RSDP provided");
        return false;
    }
    
    let hhdm = crate::memory::hhdm_offset();
    
    // Limine gives physical address - map the RSDP page before accessing
    crate::serial_println!("[ACPI] RSDP phys={:#x}, mapping...", rsdp_ptr);
    
    // Map the RSDP region (at least 36 bytes for RSDP, or 64 for XSDP)
    match crate::memory::map_mmio(rsdp_ptr, 4096) {
        Ok(rsdp_virt) => {
            crate::serial_println!("[ACPI] RSDP mapped at virt={:#x}", rsdp_virt);
            init_internal(rsdp_virt, hhdm)
        }
        Err(e) => {
            crate::serial_println!("[ACPI] Failed to map RSDP: {}", e);
            false
        }
    }
}

/// Initialize ACPI subsystem from RSDP address (from Limine)
/// Limine may give either a physical address or a HHDM-mapped virtual address
pub fn init_from_virt(rsdp_addr: u64) -> bool {
    if rsdp_addr == 0 {
        crate::serial_println!("[ACPI] No RSDP provided");
        return false;
    }
    
    let hhdm = crate::memory::hhdm_offset();
    
    // Check if this is already a virtual address (starts with HHDM prefix)
    // or a physical address that needs HHDM mapping
    let rsdp_virt = if rsdp_addr >= hhdm {
        // Already a virtual address in HHDM
        rsdp_addr
    } else {
        // Physical address - add HHDM offset
        rsdp_addr + hhdm
    };
    
    let rsdp_phys = rsdp_virt - hhdm;
    crate::serial_println!("[ACPI] RSDP at phys={:#x} virt={:#x}", rsdp_phys, rsdp_virt);
    
    init_internal(rsdp_virt, hhdm)
}

/// Initialize ACPI subsystem from physical RSDP address
pub fn init(rsdp_phys: u64) -> bool {
    if rsdp_phys == 0 {
        crate::serial_println!("[ACPI] No RSDP provided");
        return false;
    }
    
    let hhdm = crate::memory::hhdm_offset();
    let rsdp_virt = rsdp_phys + hhdm;
    
    crate::serial_println!("[ACPI] RSDP at phys={:#x} virt={:#x}", rsdp_phys, rsdp_virt);
    
    init_internal(rsdp_virt, hhdm)
}

/// Internal initialization
fn init_internal(rsdp_virt: u64, hhdm: u64) -> bool {
    crate::serial_println!("[ACPI] About to read RSDP...");
    
    // Read first byte to test memory access
    let first_byte = unsafe { core::ptr::read_volatile(rsdp_virt as *const u8) };
    crate::serial_println!("[ACPI] First byte: {:#x}", first_byte);
    
    // Read signature bytes
    let mut sig_bytes = [0u8; 8];
    for i in 0..8 {
        sig_bytes[i] = unsafe { core::ptr::read_volatile((rsdp_virt as *const u8).add(i)) };
    }
    
    // Check signature manually
    let expected = b"RSD PTR ";
    let sig_ok = sig_bytes == *expected;
    crate::serial_println!("[ACPI] Sig OK: {}", sig_ok);
    
    // Parse RSDP
    let rsdp = unsafe { &*(rsdp_virt as *const tables::Rsdp) };
    
    if !sig_ok {
        crate::serial_println!("[ACPI] Invalid RSDP signature");
        return false;
    }
    
    // Validate checksum
    let sum: u8 = unsafe {
        let bytes = core::slice::from_raw_parts(rsdp_virt as *const u8, 20);
        bytes.iter().fold(0u8, |acc, &b| acc.wrapping_add(b))
    };
    if sum != 0 {
        crate::serial_println!("[ACPI] Invalid RSDP checksum (sum={})", sum);
        return false;
    }
    
    let mut info = AcpiInfo::default();
    info.revision = rsdp.revision;
    info.oem_id = core::str::from_utf8(&rsdp.oem_id)
        .unwrap_or("Unknown")
        .trim()
        .into();
    
    crate::serial_println!("[ACPI] Revision: {}, OEM: {}", info.revision, info.oem_id);
    
    // Use XSDT if ACPI 2.0+, otherwise RSDT
    let table_addrs = if info.revision >= 2 {
        let xsdp = unsafe { &*(rsdp_virt as *const tables::Xsdp) };
        
        // Read fields from packed struct
        let xsdp_length = unsafe { ptr::read_unaligned(ptr::addr_of!(xsdp.length)) };
        let xsdt_address = unsafe { ptr::read_unaligned(ptr::addr_of!(xsdp.xsdt_address)) };
        let rsdt_address = unsafe { ptr::read_unaligned(ptr::addr_of!(rsdp.rsdt_address)) };
        
        // Validate extended checksum
        let ext_sum: u8 = unsafe {
            let bytes = core::slice::from_raw_parts(rsdp_virt as *const u8, xsdp_length as usize);
            bytes.iter().fold(0u8, |acc, &b| acc.wrapping_add(b))
        };
        if ext_sum != 0 {
            crate::serial_println!("[ACPI] Invalid XSDP extended checksum, falling back to RSDT");
            // Map RSDT before accessing
            if let Err(e) = crate::memory::map_mmio(rsdt_address as u64, 4096) {
                crate::serial_println!("[ACPI] Failed to map RSDT: {}", e);
                return false;
            }
            parse_rsdt(rsdt_address as u64 + hhdm)
        } else {
            crate::serial_println!("[ACPI] Using XSDT at {:#x}", xsdt_address);
            // Map XSDT before accessing
            if let Err(e) = crate::memory::map_mmio(xsdt_address, 4096) {
                crate::serial_println!("[ACPI] Failed to map XSDT: {}", e);
                return false;
            }
            parse_xsdt(xsdt_address + hhdm)
        }
    } else {
        let rsdt_address = unsafe { ptr::read_unaligned(ptr::addr_of!(rsdp.rsdt_address)) };
        crate::serial_println!("[ACPI] Using RSDT at {:#x}", rsdt_address);
        // Map RSDT before accessing
        if let Err(e) = crate::memory::map_mmio(rsdt_address as u64, 4096) {
            crate::serial_println!("[ACPI] Failed to map RSDT: {}", e);
            return false;
        }
        parse_rsdt(rsdt_address as u64 + hhdm)
    };
    
    crate::serial_println!("[ACPI] Found {} tables", table_addrs.len());
    
    // Parse each table
    for &table_phys in &table_addrs {
        // Map the table before accessing it
        if let Err(e) = crate::memory::map_mmio(table_phys, 4096) {
            crate::serial_println!("[ACPI] Failed to map table at {:#x}: {}", table_phys, e);
            continue;
        }
        
        let table_virt = table_phys + hhdm;
        let header = unsafe { &*(table_virt as *const tables::SdtHeader) };
        
        let sig = core::str::from_utf8(&header.signature).unwrap_or("????");
        let header_length = unsafe { ptr::read_unaligned(ptr::addr_of!(header.length)) };
        
        // Map additional pages if table is larger than 4KB
        if header_length > 4096 {
            let extra_size = header_length as usize - 4096;
            if let Err(e) = crate::memory::map_mmio(table_phys + 4096, extra_size) {
                crate::serial_println!("[ACPI] Failed to map extended table: {}", e);
            }
        }
        
        crate::serial_println!("[ACPI] Table: {} at {:#x}, len={}", sig, table_phys, header_length);
        
        match &header.signature {
            b"APIC" => {
                // MADT - Multiple APIC Description Table
                if let Some((lapic_addr, lapics, ioapics, overrides, nmis)) = madt::parse(table_virt) {
                    info.local_apic_addr = lapic_addr;
                    info.local_apics = lapics;
                    info.io_apics = ioapics;
                    info.int_overrides = overrides;
                    info.local_apic_nmis = nmis;
                    info.cpu_count = info.local_apics.len();
                    crate::serial_println!("[ACPI] MADT: {} CPUs, {} I/O APICs, {} NMI entries", 
                        info.cpu_count, info.io_apics.len(), info.local_apic_nmis.len());
                }
            }
            b"FACP" => {
                // FADT - Fixed ACPI Description Table
                if let Some(fadt_info) = fadt::parse(table_virt, hhdm) {
                    crate::serial_println!("[ACPI] FADT: PM1a={:#x}, century={}", 
                        fadt_info.pm1a_evt_blk, fadt_info.century_reg);
                    info.fadt = Some(fadt_info);
                }
            }
            b"MCFG" => {
                // MCFG - PCI Express configuration
                if let Some(regions) = mcfg::parse(table_virt) {
                    crate::serial_println!("[ACPI] MCFG: {} PCIe regions", regions.len());
                    info.mcfg_regions = regions;
                }
            }
            b"HPET" => {
                // HPET - High Precision Event Timer
                if let Some(hpet_info) = hpet::parse(table_virt) {
                    crate::serial_println!("[ACPI] HPET: base={:#x}, min_tick={}", 
                        hpet_info.base_address, hpet_info.min_tick);
                    info.hpet = Some(hpet_info);
                }
            }
            _ => {
                // Other tables (SSDT, DSDT, etc.) - log but don't parse
            }
        }
    }
    
    ACPI_INFO.call_once(|| info);
    
    true
}

/// Parse RSDT (32-bit table addresses)
fn parse_rsdt(rsdt_virt: u64) -> Vec<u64> {
    let header = unsafe { &*(rsdt_virt as *const tables::SdtHeader) };
    
    // Validate signature
    if &header.signature != b"RSDT" {
        crate::serial_println!("[ACPI] Invalid RSDT signature");
        return Vec::new();
    }
    
    let entries_start = rsdt_virt + core::mem::size_of::<tables::SdtHeader>() as u64;
    let entries_len = (header.length as usize - core::mem::size_of::<tables::SdtHeader>()) / 4;
    
    let mut addrs = Vec::with_capacity(entries_len);
    for i in 0..entries_len {
        let addr = unsafe { ptr::read_unaligned((entries_start + i as u64 * 4) as *const u32) };
        addrs.push(addr as u64);
    }
    
    addrs
}

/// Parse XSDT (64-bit table addresses)
fn parse_xsdt(xsdt_virt: u64) -> Vec<u64> {
    let header = unsafe { &*(xsdt_virt as *const tables::SdtHeader) };
    
    // Validate signature
    if &header.signature != b"XSDT" {
        crate::serial_println!("[ACPI] Invalid XSDT signature");
        return Vec::new();
    }
    
    let entries_start = xsdt_virt + core::mem::size_of::<tables::SdtHeader>() as u64;
    let entries_len = (header.length as usize - core::mem::size_of::<tables::SdtHeader>()) / 8;
    
    let mut addrs = Vec::with_capacity(entries_len);
    for i in 0..entries_len {
        let addr = unsafe { ptr::read_unaligned((entries_start + i as u64 * 8) as *const u64) };
        addrs.push(addr);
    }
    
    addrs
}

/// Get the number of CPUs detected
pub fn cpu_count() -> usize {
    ACPI_INFO.get().map(|i| i.cpu_count).unwrap_or(1)
}

/// Get Local APIC base address
pub fn local_apic_address() -> u64 {
    ACPI_INFO.get().map(|i| i.local_apic_addr).unwrap_or(0xFEE0_0000)
}

/// Check if ACPI is initialized
pub fn is_initialized() -> bool {
    ACPI_INFO.get().is_some()
}

/// Shutdown the system using ACPI
pub fn shutdown() -> ! {
    if let Some(info) = ACPI_INFO.get() {
        if let Some(ref fadt) = info.fadt {
            fadt::shutdown(fadt);
        }
    }
    
    // Fallback: triple fault or halt
    crate::serial_println!("[ACPI] Shutdown failed, halting");
    loop {
        unsafe { core::arch::asm!("hlt"); }
    }
}

/// Suspend to S3 (sleep-to-RAM). Returns true if wake-up occurred.
pub fn suspend() -> bool {
    if let Some(info) = ACPI_INFO.get() {
        if let Some(ref fadt) = info.fadt {
            return fadt::suspend_s3(fadt);
        }
    }
    crate::serial_println!("[ACPI] No FADT available for S3 suspend");
    false
}

/// Reboot the system
pub fn reboot() -> ! {
    // Try keyboard controller reset first (works on most systems)
    unsafe {
        // Wait for keyboard controller
        for _ in 0..10000 {
            let status = x86_64::instructions::port::Port::<u8>::new(0x64).read();
            if (status & 0x02) == 0 {
                break;
            }
        }
        // Send reset command
        x86_64::instructions::port::Port::<u8>::new(0x64).write(0xFE);
    }
    
    // If that didn't work, try ACPI reset
    if let Some(info) = ACPI_INFO.get() {
        if let Some(ref fadt) = info.fadt {
            fadt::reset(fadt);
        }
    }
    
    // Fallback: triple fault
    crate::serial_println!("[ACPI] Reboot failed, triple faulting");
    unsafe {
        // Load null IDT and trigger interrupt
        core::arch::asm!(
            "lidt [{}]",
            "int3",
            in(reg) &[0u64; 2],
            options(noreturn)
        );
    }
}
