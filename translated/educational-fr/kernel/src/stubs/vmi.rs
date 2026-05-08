//! VMI (Virtual Machine Introspection) stub
use alloc::string::String;
use alloc::vec::Vec;

// #[derive] — génère automatiquement les implémentations de traits à la compilation.
#[derive(Debug, Clone, Copy, PartialEq)]
// Énumération — un type qui peut être l'une de plusieurs variantes.
pub enum MemoryRegionType {
    Ram,
    Mmio,
    Rom,
    Reserved,
    AcpiReclaimable,
    Unmapped,
}

// Structure publique — visible à l'extérieur de ce module.
pub struct MemoryRegion {
    pub base: u64,
    pub size: u64,
    pub region_type: MemoryRegionType,
    pub label: String,
}

// Fonction publique — appelable depuis d'autres modules.
pub fn is_enabled() -> bool { false }
// Fonction publique — appelable depuis d'autres modules.
pub fn list_all_vms() -> Vec<(u64, String, &'static str)> { Vec::new() }
// Fonction publique — appelable depuis d'autres modules.
pub fn build_guest_memory_map(_max_regions: usize) -> Vec<MemoryRegion> { Vec::new() }
