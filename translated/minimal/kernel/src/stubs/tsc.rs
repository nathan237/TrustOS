


static mut Ow: u64 = 1_000_000_000;

pub struct Stopwatch {
    start: u64,
}

impl Stopwatch {
    pub fn start() -> Self {
        Self { start: crate::arch::timestamp() }
    }
    pub fn elapsed_nanos(&self) -> u64 {
        let bb = crate::arch::timestamp().wrapping_sub(self.start);
        brn(bb)
    }
    pub fn elapsed_micros(&self) -> u64 { self.elapsed_nanos() / 1_000 }
    pub fn lov(&self) -> u64 { self.elapsed_nanos() / 1_000_000 }
    pub fn lou(&self) -> u64 { crate::arch::timestamp().wrapping_sub(self.start) }
    pub fn mwg(&mut self) -> u64 {
        let cy = crate::arch::timestamp();
        let bb = brn(cy.wrapping_sub(self.start));
        self.start = cy;
        bb
    }
}

pub fn init(we: u64) {
    unsafe { Ow = we; }
}

pub fn ey() -> u64 { crate::arch::timestamp() }
pub fn odh() -> u64 { crate::arch::timestamp() }
pub fn odi() -> (u64, u32) { (crate::arch::timestamp(), 0) }
pub fn we() -> u64 { unsafe { Ow } }

pub fn brn(cycles: u64) -> u64 {
    let freq = unsafe { Ow };
    if freq == 0 { return 0; }
    (cycles as u128 * 1_000_000_000 / freq as u128) as u64
}
pub fn fqf(cycles: u64) -> u64 { brn(cycles) / 1_000 }
pub fn fqg(cycles: u64) -> u64 { brn(cycles) / 1_000_000 }

pub fn dvi() -> u64 { brn(crate::arch::timestamp()) }
pub fn gjt() -> u64 { dvi() / 1_000 }
pub fn nlk() -> u64 { dvi() / 1_000_000 }

pub fn dmq(bul: u64) {
    let start = dvi();
    while dvi().wrapping_sub(start) < bul {
        core::hint::spin_loop();
    }
}
pub fn ldb(micros: u64) { dmq(micros * 1_000); }
pub fn ww(millis: u64) { dmq(millis * 1_000_000); }
pub fn hq(millis: u64) { ww(millis); }
pub fn hju() -> u64 { unsafe { Ow } }
