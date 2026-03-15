













#[cfg(target_arch = "x86_64")]
#[path = "x86_64/mod.rs"]
pub mod platform;

#[cfg(target_arch = "aarch64")]
#[path = "aarch64/mod.rs"]
pub mod platform;

#[cfg(target_arch = "riscv64")]
#[path = "riscv64/mod.rs"]
pub mod platform;





pub use platform::cpu;
pub use platform::interrupts;
pub use platform::serial;
pub use platform::memory;
pub use platform::context;
pub use platform::timer;
pub use platform::boot;
pub use platform::syscall_arch;






#[inline(always)]
pub fn bhd() {
    platform::bhd();
}


#[inline(always)]
pub fn hmj() -> ! {
    loop {
        platform::bhd();
    }
}


#[inline(always)]
pub fn ofa() {
    platform::interrupts::aiy();
}


#[inline(always)]
pub fn tvq() {
    platform::interrupts::cwz();
}


#[inline(always)]
pub fn cvh<G, Ac>(bb: G) -> Ac
where
    G: FnOnce() -> Ac,
{
    platform::interrupts::cvh(bb)
}


#[inline(always)]
pub fn kaw() -> bool {
    platform::interrupts::gag()
}


#[inline(always)]
pub fn ghg(ag: u64) {
    platform::memory::ghg(ag);
}


#[inline(always)]
pub fn ivc() {
    platform::memory::ivc();
}


#[inline(always)]
pub fn dle() -> u64 {
    platform::memory::dle()
}


#[inline(always)]
pub fn dnj(ap: u64) {
    platform::memory::dnj(ap);
}


#[inline(always)]
pub fn jln() -> u64 {
    platform::cpu::jln()
}


#[inline(always)]
pub fn jat() {
    platform::cpu::jat();
}


#[inline(always)]
pub fn hbf() {
    platform::cpu::hbf();
}


#[inline(always)]
pub fn aea() -> u64 {
    platform::timer::aea()
}


pub const fn kav() -> &'static str {
    #[cfg(target_arch = "x86_64")]
    { "x86_64" }
    #[cfg(target_arch = "aarch64")]
    { "aarch64" }
    #[cfg(target_arch = "riscv64")]
    { "riscv64gc" }
}


pub const BM_: usize = 4096;


pub const DSD_: u64 = {
    #[cfg(target_arch = "x86_64")]
    { 0xFFFF_FFFF_8000_0000 } 
    #[cfg(target_arch = "aarch64")]
    { 0xFFFF_0000_0000_0000 } 
    #[cfg(target_arch = "riscv64")]
    { 0xFFFF_FFFF_C000_0000 } 
};


pub const DZU_: u32 = {
    #[cfg(target_arch = "x86_64")]
    { 52 }
    #[cfg(target_arch = "aarch64")]
    { 48 }
    #[cfg(target_arch = "riscv64")]
    { 56 } 
};








#[cfg(target_arch = "x86_64")]
pub type Port<T> = x86_64::instructions::port::Port<T>;

#[cfg(not(target_arch = "x86_64"))]
pub mod port_stub {
    use core::marker::PhantomData;

    
    pub struct Port<T: Adw> {
        qdi: PhantomData<T>,
        port: u16,
    }

    
    pub trait Adw: Copy + Default {}
    impl Adw for u8 {}
    impl Adw for u16 {}
    impl Adw for u32 {}

    impl<T: Adw> Port<T> {
        pub const fn new(port: u16) -> Self {
            Self { qdi: PhantomData, port }
        }
        pub unsafe fn read(&mut self) -> T { T::default() }
        pub unsafe fn write(&mut self, msy: T) {}
    }
}

#[cfg(not(target_arch = "x86_64"))]
pub type Port<T> = port_stub::Port<T>;

