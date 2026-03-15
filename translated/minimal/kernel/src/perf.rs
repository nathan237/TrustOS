

#[inline(always)]
pub fn zpd() {
    core::hint::hc();
}

#[inline(always)]
pub fn zas(o: bool) -> bool {
    o
}

#[inline(always)]
pub fn ztz(o: bool) -> bool {
    o
}
