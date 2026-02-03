//! MCFG (PCI Express Memory-mapped Configuration) Parser
//!
//! The MCFG table describes PCIe configuration space base addresses.

use alloc::vec::Vec;
use super::tables::SdtHeader;

/// MCFG table structure
#[repr(C, packed)]
struct McfgTable {
    header: SdtHeader,
    /// Reserved
    _reserved: u64,
    // Followed by McfgEntry structures
}

/// MCFG entry (configuration space allocation)
#[repr(C, packed)]
#[derive(Clone, Copy)]
struct McfgEntryRaw {
    /// Base address of enhanced configuration mechanism
    base_address: u64,
    /// PCI Segment Group Number
    segment_group: u16,
    /// Start PCI bus number
    start_bus: u8,
    /// End PCI bus number
    end_bus: u8,
    /// Reserved
    _reserved: u32,
}

/// Parsed MCFG entry
#[derive(Debug, Clone)]
pub struct McfgEntry {
    /// Base address for PCIe configuration space
    pub base_address: u64,
    /// PCI segment group
    pub segment: u16,
    /// Start bus number
    pub start_bus: u8,
    /// End bus number
    pub end_bus: u8,
}

impl McfgEntry {
    /// Calculate the address for a specific device's configuration space
    pub fn config_address(&self, bus: u8, device: u8, function: u8) -> Option<u64> {
        if bus < self.start_bus || bus > self.end_bus {
            return None;
        }
        if device > 31 || function > 7 {
            return None;
        }
        
        // PCIe configuration space: 4KB per function
        // Address = Base + ((bus - start_bus) << 20 | device << 15 | function << 12)
        let offset = ((bus - self.start_bus) as u64) << 20 
                   | (device as u64) << 15 
                   | (function as u64) << 12;
        
        Some(self.base_address + offset)
    }
    
    /// Size of this segment's configuration space
    pub fn size(&self) -> u64 {
        let buses = (self.end_bus - self.start_bus + 1) as u64;
        buses << 20  // 1MB per bus (32 devices * 8 functions * 4KB)
    }
}

/// Parse MCFG table
pub fn parse(mcfg_virt: u64) -> Option<Vec<McfgEntry>> {
    let header = unsafe { &*(mcfg_virt as *const SdtHeader) };
    
    // Verify signature
    if &header.signature != b"MCFG" {
        return None;
    }
    
    // Calculate number of entries
    let header_size = core::mem::size_of::<SdtHeader>() + 8; // + reserved
    let entry_size = core::mem::size_of::<McfgEntryRaw>();
    let entries_len = (header.length as usize - header_size) / entry_size;
    
    if entries_len == 0 {
        return None;
    }
    
    let mut entries = Vec::with_capacity(entries_len);
    let entries_start = mcfg_virt + header_size as u64;
    
    for i in 0..entries_len {
        let entry_addr = entries_start + (i * entry_size) as u64;
        let raw = unsafe { &*(entry_addr as *const McfgEntryRaw) };
        
        let base = unsafe { core::ptr::read_unaligned(core::ptr::addr_of!(raw.base_address)) };
        let seg = unsafe { core::ptr::read_unaligned(core::ptr::addr_of!(raw.segment_group)) };
        
        entries.push(McfgEntry {
            base_address: base,
            segment: seg,
            start_bus: raw.start_bus,
            end_bus: raw.end_bus,
        });
    }
    
    Some(entries)
}

/// Get PCIe configuration space address for a device
pub fn get_pcie_config_addr(segment: u16, bus: u8, device: u8, function: u8) -> Option<u64> {
    let info = super::get_info()?;
    
    for entry in &info.mcfg_regions {
        if entry.segment == segment && bus >= entry.start_bus && bus <= entry.end_bus {
            return entry.config_address(bus, device, function);
        }
    }
    
    None
}

/// Check if PCIe MMCONFIG is available
pub fn is_available() -> bool {
    super::get_info()
        .map(|i| !i.mcfg_regions.is_empty())
        .unwrap_or(false)
}
