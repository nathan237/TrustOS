

























use core::fmt;






pub const BLJ_: &[u8; 8] = b"ANDROID!";


pub const BLK_: u32 = 4096;



#[repr(C)]
#[derive(Clone)]
pub struct Byp {
    
    pub sj: [u8; 8],
    
    pub bvc: u32,
    
    pub eed: u32,
    
    pub hwp: u32,
    
    pub vpz: u32,
    
    pub wfu: u32,
    
    pub wft: u32,
    
    pub xao: u32,
    
    pub aus: u32,
    
    pub obi: u32,
    
    pub otc: u32,
    
    pub j: [u8; 16],
    
    pub wx: [u8; 512],
    
    pub ad: [u8; 32],
    
    pub spz: [u8; 1024],
    
    
    pub vti: u32,
    
    pub vth: u64,
    
    pub drp: u32,
    
    
    pub dgv: u32,
    
    pub bqh: u64,
}

impl Byp {
    
    pub fn zdn(bvc: u32, eed: u32) -> Self {
        let mut dh = Self {
            sj: *BLJ_,
            bvc,
            eed,
            hwp: 0,
            vpz: 0x01000000,      
            wfu: 0,
            wft: 0,
            xao: 0x00000100,
            aus: BLK_,
            obi: 2,
            otc: Self::van(1, 0, 0, 2026, 2), 
            j: [0u8; 16],
            wx: [0u8; 512],
            ad: [0u8; 32],
            spz: [0u8; 1024],
            vti: 0,
            vth: 0,
            drp: core::mem::size_of::<Self>() as u32,
            dgv: 0,
            bqh: 0,
        };

        
        let j = b"TrustOS";
        dh.j[..j.len()].dg(j);

        
        let wx = b"trustos.mode=desktop trustos.serial=ttyS0";
        dh.wx[..wx.len()].dg(wx);

        dh
    }

    
    
    fn van(q: u32, o: u32, r: u32, ccq: u32, caw: u32) -> u32 {
        (q << 25) | (o << 18) | (r << 11) | ((ccq - 2000) << 4) | caw
    }

    
    pub fn as_bytes(&self) -> &[u8] {
        unsafe {
            core::slice::anh(
                self as *const Self as *const u8,
                core::mem::size_of::<Self>(),
            )
        }
    }
}








#[repr(C)]
#[derive(Clone)]
pub struct Cse {
    
    pub sj: [u8; 8],
    
    pub bvc: u32,
    
    pub hwp: u32,
    
    pub otc: u32,
    
    pub drp: u32,
    
    pub awt: [u32; 4],
    
    pub obi: u32,
    
    pub wx: [u8; 1536],
    
    pub zok: u32,
}






pub const JZ_: u32 = 0xD00DFEED;


#[repr(C)]
pub struct FdtHeader {
    pub sj: u32,
    pub fac: u32,
    pub uxd: u32,
    pub uxc: u32,
    pub uxe: u32,
    pub dk: u32,
    pub uce: u32,
    pub qqv: u32,
    pub wpb: u32,
    pub wpc: u32,
}

impl FdtHeader {
    
    pub unsafe fn dxi(ptr: *const u8) -> bool {
        if ptr.abq() {
            return false;
        }
        let sj = u32::eqv((ptr as *const u32).md());
        sj == JZ_
    }

    
    pub unsafe fn nwg(ptr: *const u8) -> Option<&'static Self> {
        if !Self::dxi(ptr) {
            return None;
        }
        Some(&*(ptr as *const Self))
    }

    
    pub fn aay(&self) -> u32 {
        u32::eqv(self.fac)
    }
}








pub struct DtbInfo {
    
    pub epj: u64,
    
    pub dgv: u32,
    
    pub model: [u8; 64],
    pub uph: usize,
    
    pub umv: u64,
    
    pub czr: u64,
    
    pub whw: u64,
}

impl Default for DtbInfo {
    fn default() -> Self {
        Self {
            epj: 0,
            dgv: 0,
            model: [0u8; 64],
            uph: 0,
            umv: 0,
            czr: 0,
            whw: 0,
        }
    }
}

impl DtbInfo {
    
    pub unsafe fn sxt(ptr: *const u8) -> Option<Self> {
        let dh = FdtHeader::nwg(ptr)?;
        let mut co = DtbInfo::default();
        co.epj = ptr as u64;
        co.dgv = dh.aay();

        
        
        
        Some(co)
    }
}






#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SocFamily {
    
    F,
    
    Aeb,
    
    Ali,
    
    Ahz,
    
    Akf,
    
    Anm,
    
    Agu,
}

impl fmt::Display for SocFamily {
    fn fmt(&self, bb: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::F => write!(bb, "Unknown"),
            Self::Aeb => write!(bb, "QEMU virt"),
            Self::Ali => write!(bb, "Qualcomm Snapdragon"),
            Self::Ahz => write!(bb, "Samsung Exynos"),
            Self::Akf => write!(bb, "MediaTek"),
            Self::Anm => write!(bb, "Google Tensor"),
            Self::Agu => write!(bb, "Broadcom (RPi)"),
        }
    }
}


pub static mut XF_: SocFamily = SocFamily::F;


pub fn zow() -> SocFamily {
    unsafe { XF_ }
}






pub struct Cne {
    pub hpk: u64,
    pub epj: u64,
    pub hwo: u64,
    pub cnl: u64,
}

impl Cne {
    
    pub const fn yrg(jqr: SocFamily) -> Self {
        match jqr {
            SocFamily::Aeb => Self {
                hpk: 0x4008_0000,
                epj: 0x4000_0000,
                hwo: 0x4400_0000,
                cnl: 0x0900_0000,  
            },
            SocFamily::Ali => Self {
                hpk: 0x8008_0000,
                epj: 0x8300_0000,
                hwo: 0x8200_0000,
                cnl: 0x0078_AF00,  
            },
            SocFamily::Anm => Self {
                hpk: 0x8008_0000,
                epj: 0x8300_0000,
                hwo: 0x8200_0000,
                cnl: 0x10A0_0000,  
            },
            SocFamily::Agu => Self {
                hpk: 0x0008_0000,
                epj: 0x0000_0100,
                hwo: 0x0200_0000,
                cnl: 0xFE20_1000,  
            },
            SocFamily::Ahz | SocFamily::Akf | SocFamily::F => Self {
                hpk: 0x8008_0000,
                epj: 0x8300_0000,
                hwo: 0x8200_0000,
                cnl: 0x1102_0000,  
            },
        }
    }
}
