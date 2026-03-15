




pub fn noy() {
    
    unsafe {
        let wnw = super::cpu::vsl();
        super::cpu::xvs(
            wnw | super::cpu::sie_bits::Cld
                | super::cpu::sie_bits::Cmf
                | super::cpu::sie_bits::Cmd
        );
    }
}


pub const Bcj: &str = "Limine";
