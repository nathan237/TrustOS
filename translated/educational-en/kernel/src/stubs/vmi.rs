//! VMI (Virtual Machine Introspection) stub
use alloc::string::String;
use alloc::vec::Vec;

// #[derive] — auto-generates trait implementations at compile time.
#[derive(Debug, Clone, Copy, PartialEq)]
// Enumeration — a type that can be one of several variants.
pub enum MemoryRegionType {
    Ram,
    Mmio,
    Rom,
    Reserved,
    AcpiReclaimable,
    Unmapped,
}

// Public structure — visible outside this module.
pub struct MemoryRegion {
    pub base: u64,
    pub size: u64,
    pub region_type: MemoryRegionType,
    pub label: String,
}

// Public function — callable from other modules.
pub fn is_enabled() -> bool { false }
// Public function — callable from other modules.
pub fn list_all_vms() -> Vec<(u64, String, &'static str)> { Vec::new() }
// Public function — callable from other modules.
pub fn build_guest_memory_map(_max_regions: usize) -> Vec<MemoryRegion> { Vec::new() }
