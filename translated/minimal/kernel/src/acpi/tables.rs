




#[repr(C, packed)]
pub struct Cky {
    
    pub signature: [u8; 8],
    
    pub bmj: u8,
    
    pub clo: [u8; 6],
    
    pub afe: u8,
    
    pub dvi: u32,
}


#[repr(C, packed)]
pub struct Cqr {
    
    pub signature: [u8; 8],
    pub bmj: u8,
    pub clo: [u8; 6],
    pub afe: u8,
    pub dvi: u32,
    
    
    
    pub go: u32,
    
    pub ihx: u64,
    
    pub yqb: u8,
    
    pub awt: [u8; 3],
}


#[repr(C, packed)]
pub struct Ei {
    
    pub signature: [u8; 4],
    
    pub go: u32,
    
    pub afe: u8,
    
    pub bmj: u8,
    
    pub clo: [u8; 6],
    
    pub zee: [u8; 8],
    
    pub zed: u32,
    
    pub yku: u32,
    
    pub ykv: u32,
}

impl Ei {
    
    pub fn dxi(&self) -> bool {
        let ptr = self as *const _ as *const u8;
        let len = self.go as usize;
        
        if len < core::mem::size_of::<Ei>() {
            return false;
        }
        
        let sum: u8 = unsafe {
            (0..len).map(|a| *ptr.add(a)).cqs(0u8, |q, o| q.cn(o))
        };
        
        sum == 0
    }
    
    
    pub fn zol(&self) -> &str {
        core::str::jg(&self.signature).unwrap_or("????")
    }
}


#[repr(C, packed)]
#[derive(Clone, Copy, Debug)]
pub struct Gj {
    
    pub ze: u8,
    
    pub gbd: u8,
    
    pub mzf: u8,
    
    pub cct: u8,
    
    pub re: u64,
}

impl Gj {
    
    pub const BGB_: u8 = 0;
    
    pub const BGA_: u8 = 1;
    
    pub const EGO_: u8 = 2;
    
    pub const EGM_: u8 = 3;
    
    pub const EGP_: u8 = 4;
    
    pub const EGN_: u8 = 0x7F;
    
    
    pub fn cld(&self) -> bool {
        self.re != 0
    }
    
    
    pub unsafe fn read(&self) -> u64 {
        match self.ze {
            Self::BGA_ => {
                let port = self.re as u16;
                match self.gbd {
                    8 => x86_64::instructions::port::Port::<u8>::new(port).read() as u64,
                    16 => x86_64::instructions::port::Port::<u16>::new(port).read() as u64,
                    32 => x86_64::instructions::port::Port::<u32>::new(port).read() as u64,
                    _ => 0,
                }
            }
            Self::BGB_ => {
                let ag = self.re + crate::memory::lr();
                match self.gbd {
                    8 => core::ptr::read_volatile(ag as *const u8) as u64,
                    16 => core::ptr::read_volatile(ag as *const u16) as u64,
                    32 => core::ptr::read_volatile(ag as *const u32) as u64,
                    64 => core::ptr::read_volatile(ag as *const u64),
                    _ => 0,
                }
            }
            _ => 0,
        }
    }
    
    
    pub unsafe fn write(&self, bn: u64) {
        match self.ze {
            Self::BGA_ => {
                let port = self.re as u16;
                match self.gbd {
                    8 => x86_64::instructions::port::Port::<u8>::new(port).write(bn as u8),
                    16 => x86_64::instructions::port::Port::<u16>::new(port).write(bn as u16),
                    32 => x86_64::instructions::port::Port::<u32>::new(port).write(bn as u32),
                    _ => {}
                }
            }
            Self::BGB_ => {
                let ag = self.re + crate::memory::lr();
                match self.gbd {
                    8 => core::ptr::write_volatile(ag as *mut u8, bn as u8),
                    16 => core::ptr::write_volatile(ag as *mut u16, bn as u16),
                    32 => core::ptr::write_volatile(ag as *mut u32, bn as u32),
                    64 => core::ptr::write_volatile(ag as *mut u64, bn),
                    _ => {}
                }
            }
            _ => {}
        }
    }
}
