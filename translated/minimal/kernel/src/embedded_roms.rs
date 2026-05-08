

#![allow(dead_code)]


pub fn nif() -> Option<&'static [u8]> {
    #[cfg(has_nes_rom)]
    {
        static CKT_: &[u8] = include_bytes!(env!("NES_ROM_PATH"));
        Some(CKT_)
    }
    #[cfg(not(has_nes_rom))]
    {
        None
    }
}


pub fn mbe() -> Option<&'static [u8]> {
    #[cfg(has_gb_rom)]
    {
        static BZQ_: &[u8] = include_bytes!(env!("GB_ROM_PATH"));
        Some(BZQ_)
    }
    #[cfg(not(has_gb_rom))]
    {
        None
    }
}
