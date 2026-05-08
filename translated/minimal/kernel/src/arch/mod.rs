













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
pub fn acb() {
    platform::acb();
}


#[inline(always)]
pub fn dre() -> ! {
    loop {
        platform::acb();
    }
}


#[inline(always)]
pub fn ihd() {
    platform::interrupts::enable();
}


#[inline(always)]
pub fn mra() {
    platform::interrupts::bbc();
}


#[inline(always)]
pub fn bag<F, U>(f: F) -> U
where
    F: FnOnce() -> U,
{
    platform::interrupts::bag(f)
}


#[inline(always)]
pub fn fhh() -> bool {
    platform::interrupts::ctq()
}


#[inline(always)]
pub fn cxy(addr: u64) {
    platform::memory::cxy(addr);
}


#[inline(always)]
pub fn emz() {
    platform::memory::emz();
}


#[inline(always)]
pub fn biw() -> u64 {
    platform::memory::biw()
}


#[inline(always)]
pub fn bkc(val: u64) {
    platform::memory::bkc(val);
}


#[inline(always)]
pub fn exy() -> u64 {
    platform::cpu::exy()
}


#[inline(always)]
pub fn erb() {
    platform::cpu::erb();
}


#[inline(always)]
pub fn breakpoint() {
    platform::cpu::breakpoint();
}


#[inline(always)]
pub fn timestamp() -> u64 {
    platform::timer::timestamp()
}


pub const fn fhg() -> &'static str {
    #[cfg(target_arch = "x86_64")]
    { "x86_64" }
    #[cfg(target_arch = "aarch64")]
    { "aarch64" }
    #[cfg(target_arch = "riscv64")]
    { "riscv64gc" }
}


pub const BO_: usize = 4096;


pub const DVW_: u64 = {
    #[cfg(target_arch = "x86_64")]
    { 0xFFFF_FFFF_8000_0000 } 
    #[cfg(target_arch = "aarch64")]
    { 0xFFFF_0000_0000_0000 } 
    #[cfg(target_arch = "riscv64")]
    { 0xFFFF_FFFF_C000_0000 } 
};


pub const EDL_: u32 = {
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

    
    pub struct Port<T: Mx> {
        _phantom: PhantomData<T>,
        port: u16,
    }

    
    pub trait Mx: Copy + Default {}
    impl Mx for u8 {}
    impl Mx for u16 {}
    impl Mx for u32 {}

    impl<T: Mx> Port<T> {
        pub const fn new(port: u16) -> Self {
            Self { _phantom: PhantomData, port }
        }
        pub unsafe fn read(&mut self) -> T { T::default() }
        pub unsafe fn write(&mut self, hdm: T) {}
    }
}

#[cfg(not(target_arch = "x86_64"))]
pub type Port<T> = port_stub::Port<T>;

