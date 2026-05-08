




#[repr(C, packed)]
pub struct Aoo {
    
    pub signature: [u8; 8],
    
    pub checksum: u8,
    
    pub oem_id: [u8; 6],
    
    pub revision: u8,
    
    pub rsdt_address: u32,
}


#[repr(C, packed)]
pub struct Ase {
    
    pub signature: [u8; 8],
    pub checksum: u8,
    pub oem_id: [u8; 6],
    pub revision: u8,
    pub rsdt_address: u32,
    
    
    
    pub length: u32,
    
    pub xsdt_address: u64,
    
    pub extended_checksum: u8,
    
    pub reserved: [u8; 3],
}


#[repr(C, packed)]
pub struct Bu {
    
    pub signature: [u8; 4],
    
    pub length: u32,
    
    pub revision: u8,
    
    pub checksum: u8,
    
    pub oem_id: [u8; 6],
    
    pub oem_table_id: [u8; 8],
    
    pub oem_revision: u32,
    
    pub creator_id: u32,
    
    pub creator_revision: u32,
}

impl Bu {
    
    pub fn bpu(&self) -> bool {
        let ptr = self as *const _ as *const u8;
        let len = self.length as usize;
        
        if len < core::mem::size_of::<Bu>() {
            return false;
        }
        
        let sum: u8 = unsafe {
            (0..len).map(|i| *ptr.add(i)).fold(0u8, |a, b| a.wrapping_add(b))
        };
        
        sum == 0
    }
    
    
    pub fn qwz(&self) -> &str {
        core::str::from_utf8(&self.signature).unwrap_or("????")
    }
}


#[repr(C, packed)]
#[derive(Clone, Copy, Debug)]
pub struct Cx {
    
    pub address_space: u8,
    
    pub bit_width: u8,
    
    pub bit_offset: u8,
    
    pub access_size: u8,
    
    pub address: u64,
}

impl Cx {
    
    pub const BIF_: u8 = 0;
    
    pub const BIE_: u8 = 1;
    
    pub const EKH_: u8 = 2;
    
    pub const EKF_: u8 = 3;
    
    pub const EKI_: u8 = 4;
    
    pub const EKG_: u8 = 0x7F;
    
    
    pub fn is_valid(&self) -> bool {
        self.address != 0
    }
    
    
    pub unsafe fn read(&self) -> u64 {
        match self.address_space {
            Self::BIE_ => {
                let port = self.address as u16;
                match self.bit_width {
                    8 => x86_64::instructions::port::Port::<u8>::new(port).read() as u64,
                    16 => x86_64::instructions::port::Port::<u16>::new(port).read() as u64,
                    32 => x86_64::instructions::port::Port::<u32>::new(port).read() as u64,
                    _ => 0,
                }
            }
            Self::BIF_ => {
                let addr = self.address + crate::memory::hhdm_offset();
                match self.bit_width {
                    8 => core::ptr::read_volatile(addr as *const u8) as u64,
                    16 => core::ptr::read_volatile(addr as *const u16) as u64,
                    32 => core::ptr::read_volatile(addr as *const u32) as u64,
                    64 => core::ptr::read_volatile(addr as *const u64),
                    _ => 0,
                }
            }
            _ => 0,
        }
    }
    
    
    pub unsafe fn write(&self, value: u64) {
        match self.address_space {
            Self::BIE_ => {
                let port = self.address as u16;
                match self.bit_width {
                    8 => x86_64::instructions::port::Port::<u8>::new(port).write(value as u8),
                    16 => x86_64::instructions::port::Port::<u16>::new(port).write(value as u16),
                    32 => x86_64::instructions::port::Port::<u32>::new(port).write(value as u32),
                    _ => {}
                }
            }
            Self::BIF_ => {
                let addr = self.address + crate::memory::hhdm_offset();
                match self.bit_width {
                    8 => core::ptr::write_volatile(addr as *mut u8, value as u8),
                    16 => core::ptr::write_volatile(addr as *mut u16, value as u16),
                    32 => core::ptr::write_volatile(addr as *mut u32, value as u32),
                    64 => core::ptr::write_volatile(addr as *mut u64, value),
                    _ => {}
                }
            }
            _ => {}
        }
    }
}
