


static mut Aib: u64 = 1_000_000_000;

pub struct Stopwatch {
    ay: u64,
}

impl Stopwatch {
    pub fn ay() -> Self {
        Self { ay: crate::arch::aea() }
    }
    pub fn ksx(&self) -> u64 {
        let ez = crate::arch::aea().nj(self.ay);
        eaj(ez)
    }
    pub fn fhk(&self) -> u64 { self.ksx() / 1_000 }
    pub fn ska(&self) -> u64 { self.ksx() / 1_000_000 }
    pub fn sjz(&self) -> u64 { crate::arch::aea().nj(self.ay) }
    pub fn ubx(&mut self) -> u64 {
        let iu = crate::arch::aea();
        let ez = eaj(iu.nj(self.ay));
        self.ay = iu;
        ez
    }
}

pub fn init(ard: u64) {
    unsafe { Aib = ard; }
}

pub fn ow() -> u64 { crate::arch::aea() }
pub fn vsr() -> u64 { crate::arch::aea() }
pub fn vss() -> (u64, u32) { (crate::arch::aea(), 0) }
pub fn ard() -> u64 { unsafe { Aib } }

pub fn eaj(yl: u64) -> u64 {
    let kx = unsafe { Aib };
    if kx == 0 { return 0; }
    (yl as u128 * 1_000_000_000 / kx as u128) as u64
}
pub fn knl(yl: u64) -> u64 { eaj(yl) / 1_000 }
pub fn knm(yl: u64) -> u64 { eaj(yl) / 1_000_000 }

pub fn hsz() -> u64 { eaj(crate::arch::aea()) }
pub fn loz() -> u64 { hsz() / 1_000 }
pub fn uvu() -> u64 { hsz() / 1_000_000 }

pub fn hfs(efq: u64) {
    let ay = hsz();
    while hsz().nj(ay) < efq {
        core::hint::hc();
    }
}
pub fn rvf(llt: u64) { hfs(llt * 1_000); }
pub fn asq(foh: u64) { hfs(foh * 1_000_000); }
pub fn rd(foh: u64) { asq(foh); }
pub fn nbj() -> u64 { unsafe { Aib } }
