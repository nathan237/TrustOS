

























use core::fmt;






pub const BOB_: &[u8; 8] = b"ANDROID!";


pub const BOC_: u32 = 4096;



#[repr(C)]
#[derive(Clone)]
pub struct Ahm {
    
    pub magic: [u8; 8],
    
    pub kernel_size: u32,
    
    pub kernel_addr: u32,
    
    pub ramdisk_size: u32,
    
    pub ramdisk_addr: u32,
    
    pub second_size: u32,
    
    pub second_addr: u32,
    
    pub tags_addr: u32,
    
    pub xy: u32,
    
    pub header_version: u32,
    
    pub os_version: u32,
    
    pub name: [u8; 16],
    
    pub cmdline: [u8; 512],
    
    pub id: [u8; 32],
    
    pub extra_cmdline: [u8; 1024],
    
    
    pub recovery_dtbo_size: u32,
    
    pub recovery_dtbo_offset: u64,
    
    pub bms: u32,
    
    
    pub dtb_size: u32,
    
    pub dtb_addr: u64,
}

impl Ahm {
    
    pub fn qpn(kernel_size: u32, kernel_addr: u32) -> Self {
        let mut header = Self {
            magic: *BOB_,
            kernel_size,
            kernel_addr,
            ramdisk_size: 0,
            ramdisk_addr: 0x01000000,      
            second_size: 0,
            second_addr: 0,
            tags_addr: 0x00000100,
            xy: BOC_,
            header_version: 2,
            os_version: Self::npb(1, 0, 0, 2026, 2), 
            name: [0u8; 16],
            cmdline: [0u8; 512],
            id: [0u8; 32],
            extra_cmdline: [0u8; 1024],
            recovery_dtbo_size: 0,
            recovery_dtbo_offset: 0,
            bms: core::mem::size_of::<Self>() as u32,
            dtb_size: 0,
            dtb_addr: 0,
        };

        
        let name = b"TrustOS";
        header.name[..name.len()].copy_from_slice(name);

        
        let cmdline = b"trustos.mode=desktop trustos.serial=ttyS0";
        header.cmdline[..cmdline.len()].copy_from_slice(cmdline);

        header
    }

    
    
    fn npb(a: u32, b: u32, c: u32, year: u32, month: u32) -> u32 {
        (a << 25) | (b << 18) | (c << 11) | ((year - 2000) << 4) | month
    }

    
    pub fn as_bytes(&self) -> &[u8] {
        unsafe {
            core::slice::from_raw_parts(
                self as *const Self as *const u8,
                core::mem::size_of::<Self>(),
            )
        }
    }
}








#[repr(C)]
#[derive(Clone)]
pub struct Ate {
    
    pub magic: [u8; 8],
    
    pub kernel_size: u32,
    
    pub ramdisk_size: u32,
    
    pub os_version: u32,
    
    pub bms: u32,
    
    pub reserved: [u32; 4],
    
    pub header_version: u32,
    
    pub cmdline: [u8; 1536],
    
    pub signature_size: u32,
}






pub const KT_: u32 = 0xD00DFEED;


#[repr(C)]
pub struct FdtHeader {
    pub magic: u32,
    pub totalsize: u32,
    pub off_dt_struct: u32,
    pub off_dt_strings: u32,
    pub off_mem_rsvmap: u32,
    pub version: u32,
    pub last_comp_version: u32,
    pub boot_cpuid_phys: u32,
    pub size_dt_strings: u32,
    pub size_dt_struct: u32,
}

impl FdtHeader {
    
    pub unsafe fn bpu(ptr: *const u8) -> bool {
        if ptr.is_null() {
            return false;
        }
        let magic = u32::from_be((ptr as *const u32).read_unaligned());
        magic == KT_
    }

    
    pub unsafe fn from_ptr(ptr: *const u8) -> Option<&'static Self> {
        if !Self::bpu(ptr) {
            return None;
        }
        Some(&*(ptr as *const Self))
    }

    
    pub fn total_size(&self) -> u32 {
        u32::from_be(self.totalsize)
    }
}








pub struct DtbInfo {
    
    pub dtb_base: u64,
    
    pub dtb_size: u32,
    
    pub model: [u8; 64],
    pub model_len: usize,
    
    pub mem_base: u64,
    
    pub bcr: u64,
    
    pub serial_base: u64,
}

impl Default for DtbInfo {
    fn default() -> Self {
        Self {
            dtb_base: 0,
            dtb_size: 0,
            model: [0u8; 64],
            model_len: 0,
            mem_base: 0,
            bcr: 0,
            serial_base: 0,
        }
    }
}

impl DtbInfo {
    
    pub unsafe fn lzd(ptr: *const u8) -> Option<Self> {
        let header = FdtHeader::from_ptr(ptr)?;
        let mut info = DtbInfo::default();
        info.dtb_base = ptr as u64;
        info.dtb_size = header.total_size();

        
        
        
        Some(info)
    }
}






#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SocFamily {
    
    Unknown,
    
    QemuVirt,
    
    Qualcomm,
    
    Exynos,
    
    MediaTek,
    
    Tensor,
    
    Broadcom,
}

impl fmt::Display for SocFamily {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unknown => write!(f, "Unknown"),
            Self::QemuVirt => write!(f, "QEMU virt"),
            Self::Qualcomm => write!(f, "Qualcomm Snapdragon"),
            Self::Exynos => write!(f, "Samsung Exynos"),
            Self::MediaTek => write!(f, "MediaTek"),
            Self::Tensor => write!(f, "Google Tensor"),
            Self::Broadcom => write!(f, "Broadcom (RPi)"),
        }
    }
}


pub static mut YM_: SocFamily = SocFamily::Unknown;


pub fn qxg() -> SocFamily {
    unsafe { YM_ }
}






pub struct Aqe {
    pub kernel_base: u64,
    pub dtb_base: u64,
    pub ramdisk_base: u64,
    pub uart_base: u64,
}

impl Aqe {
    
    pub const fn qgb(fbe: SocFamily) -> Self {
        match fbe {
            SocFamily::QemuVirt => Self {
                kernel_base: 0x4008_0000,
                dtb_base: 0x4000_0000,
                ramdisk_base: 0x4400_0000,
                uart_base: 0x0900_0000,  
            },
            SocFamily::Qualcomm => Self {
                kernel_base: 0x8008_0000,
                dtb_base: 0x8300_0000,
                ramdisk_base: 0x8200_0000,
                uart_base: 0x0078_AF00,  
            },
            SocFamily::Tensor => Self {
                kernel_base: 0x8008_0000,
                dtb_base: 0x8300_0000,
                ramdisk_base: 0x8200_0000,
                uart_base: 0x10A0_0000,  
            },
            SocFamily::Broadcom => Self {
                kernel_base: 0x0008_0000,
                dtb_base: 0x0000_0100,
                ramdisk_base: 0x0200_0000,
                uart_base: 0xFE20_1000,  
            },
            SocFamily::Exynos | SocFamily::MediaTek | SocFamily::Unknown => Self {
                kernel_base: 0x8008_0000,
                dtb_base: 0x8300_0000,
                ramdisk_base: 0x8200_0000,
                uart_base: 0x1102_0000,  
            },
        }
    }
}
