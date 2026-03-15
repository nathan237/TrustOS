//! Embedded ROM data — compiled into the kernel binary
//! ROMs are detected at build time from kernel/roms/ directory
#![allow(dead_code)]

/// Get embedded NES ROM data (if a .nes file was in roms/ at build time)
pub fn nes_rom() -> Option<&'static [u8]> {
    #[cfg(has_nes_rom)]
    {
        static NES_DATA: &[u8] = include_bytes!(env!("NES_ROM_PATH"));
        Some(NES_DATA)
    }
    #[cfg(not(has_nes_rom))]
    {
        None
    }
}

/// Get embedded Game Boy ROM data (if a .gb file was in roms/ at build time)
pub fn gb_rom() -> Option<&'static [u8]> {
    #[cfg(has_gb_rom)]
    {
        static GB_DATA: &[u8] = include_bytes!(env!("GB_ROM_PATH"));
        Some(GB_DATA)
    }
    #[cfg(not(has_gb_rom))]
    {
        None
    }
}
