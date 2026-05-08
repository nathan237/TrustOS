



pub const AKS_: u8 = 48;
pub const CXS_: u8 = 0xFF;
pub const CFY_: u8 = 0xFE;
pub const AFH_: u8 = 49;
pub const VS_: u8 = 50;
pub const WS_: u8 = 61;
pub const HZ_: u8 = 62;

pub fn init() -> bool {
    #[cfg(target_arch = "aarch64")]
    { return crate::arch::platform::gic::is_initialized(); }
    #[cfg(not(target_arch = "aarch64"))]
    { false }
}
pub fn cau() {}
pub fn bng() {
    #[cfg(target_arch = "aarch64")]
    if crate::arch::platform::gic::is_initialized() {
        crate::arch::platform::gic::fvb(0); 
    }
}
pub fn lapic_id() -> u32 { 0 }
pub fn fbp(_interval_ms: u64) {
    #[cfg(target_arch = "aarch64")]
    if crate::arch::platform::gic::is_initialized() {
        crate::arch::platform::gic::gqp(_interval_ms);
    }
}
pub fn jiy() {}
pub fn gtx(_target_apic_id: u32, _vector: u8) {}
pub fn jel(_vector: u8) {}
pub fn lq() -> bool {
    #[cfg(target_arch = "aarch64")]
    { return crate::arch::platform::gic::is_initialized(); }
    #[cfg(not(target_arch = "aarch64"))]
    { false }
}
pub fn gyq() -> u64 { 0 }
pub fn eyz(_irq: u8, _vector: u8) {}
