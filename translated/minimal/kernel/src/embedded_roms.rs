

#![allow(bgr)]


pub fn usa() -> Option<&'static [u8]> {
    #[cfg(tmt)]
    {
        static CHK_: &[u8] = include_bytes!(env!("NES_ROM_PATH"));
        Some(CHK_)
    }
    #[cfg(not(tmt))]
    {
        None
    }
}


pub fn tag() -> Option<&'static [u8]> {
    #[cfg(tmn)]
    {
        static BWK_: &[u8] = include_bytes!(env!("GB_ROM_PATH"));
        Some(BWK_)
    }
    #[cfg(not(tmn))]
    {
        None
    }
}
