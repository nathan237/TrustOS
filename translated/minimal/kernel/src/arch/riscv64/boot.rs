




pub fn hut() {
    
    unsafe {
        let osk = super::cpu::odc();
        super::cpu::pvd(
            osk | super::cpu::sie_bits::Aos
                | super::cpu::sie_bits::Apt
                | super::cpu::sie_bits::Apr
        );
    }
}


pub const Wo: &str = "Limine";
