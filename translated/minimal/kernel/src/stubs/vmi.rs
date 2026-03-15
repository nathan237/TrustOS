
use alloc::string::String;
use alloc::vec::Vec;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MemoryRegionType {
    Jw,
    Nn,
    Bre,
    Nw,
    Bbk,
    Afg,
}

pub struct MemoryRegion {
    pub ar: u64,
    pub aw: u64,
    pub bwo: MemoryRegionType,
    pub cu: String,
}

pub fn zu() -> bool { false }
pub fn ojm() -> Vec<(u64, String, &'static str)> { Vec::new() }
pub fn kfk(yav: usize) -> Vec<MemoryRegion> { Vec::new() }
