
use alloc::string::String;
use alloc::vec::Vec;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MemoryRegionType {
    Ram,
    Mmio,
    Rom,
    Reserved,
    AcpiReclaimable,
    Unmapped,
}

pub struct MemoryRegion {
    pub base: u64,
    pub size: u64,
    pub region_type: MemoryRegionType,
    pub label: String,
}

pub fn lq() -> bool { false }
pub fn ikn() -> Vec<(u64, String, &'static str)> { Vec::new() }
pub fn fkc(_max_regions: usize) -> Vec<MemoryRegion> { Vec::new() }
