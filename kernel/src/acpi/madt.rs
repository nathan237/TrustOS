//! MADT (Multiple APIC Description Table) Parser
//!
//! The MADT describes all APICs in the system, including:
//! - Local APICs (one per CPU core)
//! - I/O APICs (for routing interrupts)
//! - Interrupt Source Overrides

use alloc::vec::Vec;
use super::tables::SdtHeader;

/// MADT structure (after SDT header)
#[repr(C, packed)]
struct MadtHeader {
    /// Local APIC physical address
    local_apic_addr: u32,
    /// Flags (bit 0 = dual 8259 PICs present)
    flags: u32,
}

/// MADT entry header
#[repr(C, packed)]
#[derive(Clone, Copy)]
struct MadtEntryHeader {
    entry_type: u8,
    length: u8,
}

/// Entry types
const ENTRY_LOCAL_APIC: u8 = 0;
const ENTRY_IO_APIC: u8 = 1;
const ENTRY_INT_SRC_OVERRIDE: u8 = 2;
const ENTRY_NMI_SOURCE: u8 = 3;
const ENTRY_LOCAL_APIC_NMI: u8 = 4;
const ENTRY_LOCAL_APIC_ADDR_OVERRIDE: u8 = 5;
const ENTRY_LOCAL_X2APIC: u8 = 9;

/// Local APIC entry (type 0)
#[repr(C, packed)]
struct LocalApicEntry {
    header: MadtEntryHeader,
    /// ACPI processor UID
    acpi_processor_id: u8,
    /// Local APIC ID
    apic_id: u8,
    /// Flags (bit 0 = enabled, bit 1 = online capable)
    flags: u32,
}

/// I/O APIC entry (type 1)
#[repr(C, packed)]
struct IoApicEntry {
    header: MadtEntryHeader,
    /// I/O APIC ID
    io_apic_id: u8,
    /// Reserved
    _reserved: u8,
    /// I/O APIC physical address
    io_apic_addr: u32,
    /// Global System Interrupt Base
    gsi_base: u32,
}

/// Interrupt Source Override entry (type 2)
#[repr(C, packed)]
struct IntSourceOverrideEntry {
    header: MadtEntryHeader,
    /// Bus (always 0 = ISA)
    bus: u8,
    /// Source (ISA IRQ)
    source: u8,
    /// Global System Interrupt
    gsi: u32,
    /// Flags (polarity, trigger mode)
    flags: u16,
}

/// Local APIC Address Override entry (type 5)
#[repr(C, packed)]
struct LocalApicAddrOverrideEntry {
    header: MadtEntryHeader,
    /// Reserved
    _reserved: u16,
    /// 64-bit Local APIC address
    local_apic_addr: u64,
}

/// Local x2APIC entry (type 9)
#[repr(C, packed)]
struct X2ApicEntry {
    header: MadtEntryHeader,
    /// Reserved
    _reserved: u16,
    /// x2APIC ID (32-bit)
    x2apic_id: u32,
    /// Flags (bit 0 = enabled, bit 1 = online capable)
    flags: u32,
    /// ACPI processor UID
    acpi_processor_uid: u32,
}

/// Parsed Local APIC information
#[derive(Debug, Clone)]
pub struct LocalApic {
    /// APIC ID
    pub apic_id: u32,
    /// ACPI Processor ID/UID
    pub processor_id: u32,
    /// Is this APIC enabled?
    pub enabled: bool,
    /// Can be brought online?
    pub online_capable: bool,
}

/// Parsed I/O APIC information
#[derive(Debug, Clone)]
pub struct IoApic {
    /// I/O APIC ID
    pub id: u8,
    /// Physical address
    pub address: u64,
    /// Global System Interrupt base
    pub gsi_base: u32,
}

/// Interrupt Source Override
#[derive(Debug, Clone)]
pub struct IntSourceOverride {
    /// ISA IRQ number
    pub source: u8,
    /// Global System Interrupt
    pub gsi: u32,
    /// Polarity (0 = bus default, 1 = active high, 3 = active low)
    pub polarity: u8,
    /// Trigger mode (0 = bus default, 1 = edge, 3 = level)
    pub trigger: u8,
}

/// Parse MADT table
pub fn parse(madt_virt: u64) -> Option<(u64, Vec<LocalApic>, Vec<IoApic>, Vec<IntSourceOverride>)> {
    let header = unsafe { &*(madt_virt as *const SdtHeader) };
    
    // Verify signature
    if &header.signature != b"APIC" {
        return None;
    }
    
    let madt_header_offset = core::mem::size_of::<SdtHeader>();
    let madt_header = unsafe { 
        &*((madt_virt + madt_header_offset as u64) as *const MadtHeader) 
    };
    
    let mut local_apic_addr = unsafe { 
        core::ptr::read_unaligned(core::ptr::addr_of!(madt_header.local_apic_addr)) 
    } as u64;
    
    let mut local_apics = Vec::new();
    let mut io_apics = Vec::new();
    let mut overrides = Vec::new();
    
    // Parse entries
    let entries_start = madt_virt + madt_header_offset as u64 + 8;
    let table_end = madt_virt + header.length as u64;
    let mut offset = entries_start;
    
    while offset + 2 <= table_end {
        let entry_header = unsafe { &*(offset as *const MadtEntryHeader) };
        
        if entry_header.length < 2 {
            break;
        }
        
        match entry_header.entry_type {
            ENTRY_LOCAL_APIC => {
                if entry_header.length >= 8 {
                    let entry = unsafe { &*(offset as *const LocalApicEntry) };
                    let flags = unsafe { core::ptr::read_unaligned(core::ptr::addr_of!(entry.flags)) };
                    
                    local_apics.push(LocalApic {
                        apic_id: entry.apic_id as u32,
                        processor_id: entry.acpi_processor_id as u32,
                        enabled: (flags & 1) != 0,
                        online_capable: (flags & 2) != 0,
                    });
                }
            }
            ENTRY_IO_APIC => {
                if entry_header.length >= 12 {
                    let entry = unsafe { &*(offset as *const IoApicEntry) };
                    let addr = unsafe { core::ptr::read_unaligned(core::ptr::addr_of!(entry.io_apic_addr)) };
                    let gsi_base = unsafe { core::ptr::read_unaligned(core::ptr::addr_of!(entry.gsi_base)) };
                    
                    io_apics.push(IoApic {
                        id: entry.io_apic_id,
                        address: addr as u64,
                        gsi_base,
                    });
                }
            }
            ENTRY_INT_SRC_OVERRIDE => {
                if entry_header.length >= 10 {
                    let entry = unsafe { &*(offset as *const IntSourceOverrideEntry) };
                    let gsi = unsafe { core::ptr::read_unaligned(core::ptr::addr_of!(entry.gsi)) };
                    let flags = unsafe { core::ptr::read_unaligned(core::ptr::addr_of!(entry.flags)) };
                    
                    overrides.push(IntSourceOverride {
                        source: entry.source,
                        gsi,
                        polarity: (flags & 0x03) as u8,
                        trigger: ((flags >> 2) & 0x03) as u8,
                    });
                }
            }
            ENTRY_LOCAL_APIC_ADDR_OVERRIDE => {
                if entry_header.length >= 12 {
                    let entry = unsafe { &*(offset as *const LocalApicAddrOverrideEntry) };
                    local_apic_addr = unsafe { 
                        core::ptr::read_unaligned(core::ptr::addr_of!(entry.local_apic_addr)) 
                    };
                }
            }
            ENTRY_LOCAL_X2APIC => {
                if entry_header.length >= 16 {
                    let entry = unsafe { &*(offset as *const X2ApicEntry) };
                    let x2apic_id = unsafe { core::ptr::read_unaligned(core::ptr::addr_of!(entry.x2apic_id)) };
                    let flags = unsafe { core::ptr::read_unaligned(core::ptr::addr_of!(entry.flags)) };
                    let uid = unsafe { core::ptr::read_unaligned(core::ptr::addr_of!(entry.acpi_processor_uid)) };
                    
                    local_apics.push(LocalApic {
                        apic_id: x2apic_id,
                        processor_id: uid,
                        enabled: (flags & 1) != 0,
                        online_capable: (flags & 2) != 0,
                    });
                }
            }
            _ => {
                // Unknown entry type, skip
            }
        }
        
        offset += entry_header.length as u64;
    }
    
    Some((local_apic_addr, local_apics, io_apics, overrides))
}
