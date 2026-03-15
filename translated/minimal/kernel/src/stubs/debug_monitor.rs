

use alloc::string::String;
use alloc::vec::Vec;

#[derive(Debug, Clone, Copy)]
pub enum DebugCategory {
    Iu,
    Lr,
    Hx,
    Jr,
    Nn,
    Ctn,
    Fv,
    Qg,
}

#[derive(Debug, Clone, Copy)]
pub enum HandleStatus {
    Gw,
    Id,
}

pub fn init() {}
pub fn qg() {}
pub fn apa() {}
pub fn ky() -> bool { false }
pub fn rl() -> bool { false }
pub fn bry(
    qeb: u64,
    xyf: DebugCategory,
    xxs: u64,
    yda: HandleStatus,
    fcc: u64,
    dds: usize,
    xys: &str,
) {}
pub fn jtr() -> u64 { 0 }
pub fn jup() -> u64 { 0 }
pub fn kym() -> String { String::from("Debug monitor not available") }
pub fn kyr() -> String { String::new() }
pub fn nyd() -> String { String::new() }
pub fn nyf() -> String { String::new() }
pub fn nys(jxu: usize) -> String { String::new() }
pub fn pje(qbs: bool) {}
