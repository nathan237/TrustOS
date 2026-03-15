

#[derive(Debug, Clone, Copy)]
pub enum Feature {
    Anv, Anw, Uc, Amy, Qu, Amz, Anc,
    Ana, Anb, Agk, Ow, Agl, Agb, Alb,
    Amp, Alv, Alw, Aoc, Anf,
}

impl core::fmt::Display for Feature {
    fn fmt(&self, bb: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(bb, "{:?}", self)
    }
}

pub fn oam(xzf: Feature) -> bool { false }
pub fn vlg() {
    crate::serial_println!("CPU features: none (non-x86_64 stub)");
}
