



pub const AIV_: u8 = 48;
pub const CUB_: u8 = 0xFF;
pub const CCN_: u8 = 0xFE;
pub const ADQ_: u8 = 49;
pub const UJ_: u8 = 50;
pub const VJ_: u8 = 61;
pub const HH_: u8 = 62;

pub fn init() -> bool {
    #[cfg(target_arch = "aarch64")]
    { return crate::arch::platform::gic::ky(); }
    #[cfg(not(target_arch = "aarch64"))]
    { false }
}
pub fn eso() {}
pub fn dsp() {
    #[cfg(target_arch = "aarch64")]
    if crate::arch::platform::gic::ky() {
        crate::arch::platform::gic::ktu(0); 
    }
}
pub fn ett() -> u32 { 0 }
pub fn jro(qch: u64) {
    #[cfg(target_arch = "aarch64")]
    if crate::arch::platform::gic::ky() {
        crate::arch::platform::gic::lye(qch);
    }
}
pub fn pox() {}
pub fn mds(ydj: u32, msz: u8) {}
pub fn phu(msz: u8) {}
pub fn zu() -> bool {
    #[cfg(target_arch = "aarch64")]
    { return crate::arch::platform::gic::ky(); }
    #[cfg(not(target_arch = "aarch64"))]
    { false }
}
pub fn xgv() -> u64 { 0 }
pub fn jmw(yac: u8, msz: u8) {}
