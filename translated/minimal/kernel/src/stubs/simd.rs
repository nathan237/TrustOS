

pub fn ktg() {}
pub fn ktd() -> bool { false }
pub fn sky() -> bool { false }

pub unsafe fn umz(cs: *mut u8, cy: *const u8, len: usize) {
    core::ptr::copy_nonoverlapping(cy, cs, len);
}

pub unsafe fn unb(cs: *mut u8, bn: u8, len: usize) {
    core::ptr::ahx(cs, bn, len);
}

pub unsafe fn umy(q: *const u8, o: *const u8, len: usize) -> bool {
    for a in 0..len {
        if *q.add(a) != *o.add(a) { return false; }
    }
    true
}

pub unsafe fn xwo(cs: *mut u8, cy: *const u8, len: usize) {
    for a in 0..len {
        *cs.add(a) ^= *cy.add(a);
    }
}
