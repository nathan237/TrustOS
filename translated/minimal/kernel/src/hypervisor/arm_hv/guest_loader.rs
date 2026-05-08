


















































use alloc::string::String;
use alloc::vec::Vec;
use alloc::vec;
use alloc::format;
use core::ptr;

use super::Ic;
use super::stage2::Stage2Tables;






pub const CBY_: u64 = 0x4000_0000;


pub const AFL_: u64 = 0x0020_0000;  


pub const ACV_: u64 = 0x0400_0000;     


pub const AFC_: u64 = 0x0500_0000;  


pub const BUC_: u64 = 512 * 1024 * 1024;


pub const BBU_: usize = 64 * 1024 * 1024;


pub const BBO_: usize = 1 * 1024 * 1024;


pub const BBT_: usize = 400 * 1024 * 1024;


pub const AMO_: u32 = 0x644d5241; 


pub const WF_: usize = 0x38;


pub const DUQ_: usize = 0x10;


pub const ELE_: usize = 0x08;






#[derive(Debug, Clone)]
pub struct Arm64ImageHeader {
    
    pub hmj: u32,
    
    pub crm: u64,
    
    pub image_size: u64,
    
    pub flags: u64,
    
    pub magic: u32,
}

impl Arm64ImageHeader {
    
    pub fn parse(data: &[u8]) -> Result<Self, &'static str> {
        if data.len() < 64 {
            return Err("Image too small (need at least 64 bytes for header)");
        }

        let magic = u32::from_le_bytes([
            data[WF_],
            data[WF_ + 1],
            data[WF_ + 2],
            data[WF_ + 3],
        ]);

        if magic != AMO_ {
            return Err("Invalid ARM64 Image magic (expected 0x644d5241 'ARM\\x64')");
        }

        let hmj = u32::from_le_bytes([data[0], data[1], data[2], data[3]]);

        let crm = u64::from_le_bytes([
            data[0x08], data[0x09], data[0x0A], data[0x0B],
            data[0x0C], data[0x0D], data[0x0E], data[0x0F],
        ]);

        let image_size = u64::from_le_bytes([
            data[0x10], data[0x11], data[0x12], data[0x13],
            data[0x14], data[0x15], data[0x16], data[0x17],
        ]);

        let flags = u64::from_le_bytes([
            data[0x18], data[0x19], data[0x1A], data[0x1B],
            data[0x1C], data[0x1D], data[0x1E], data[0x1F],
        ]);

        Ok(Arm64ImageHeader {
            hmj,
            crm,
            image_size,
            flags,
            magic,
        })
    }

    
    pub fn is_little_endian(&self) -> bool {
        self.flags & 1 == 0
    }

    
    pub fn effective_size(&self, file_size: usize) -> usize {
        if self.image_size > 0 && (self.image_size as usize) <= file_size {
            self.image_size as usize
        } else {
            file_size
        }
    }
}






const KT_: u32 = 0xD00DFEED;


const ATT_: u32 = 0x00000001;
const ATU_: u32   = 0x00000002;
const ATW_: u32        = 0x00000003;
const ATV_: u32         = 0x00000004;
const ADQ_: u32         = 0x00000009;


pub fn jpt(data: &[u8]) -> Result<(), &'static str> {
    if data.len() < 40 {
        return Err("DTB too small");
    }
    let magic = u32::from_be_bytes([data[0], data[1], data[2], data[3]]);
    if magic != KT_ {
        return Err("Invalid DTB magic (expected 0xD00DFEED)");
    }
    let total_size = u32::from_be_bytes([data[4], data[5], data[6], data[7]]) as usize;
    if total_size > data.len() {
        return Err("DTB total_size exceeds buffer");
    }
    Ok(())
}








pub fn nrv(
    dtb: &mut [u8],
    dtb_len: usize,
    initrd_start: Option<u64>,
    czt: Option<u64>,
    bootargs: Option<&str>,
) -> Result<usize, &'static str> {
    
    
    
    
    
    
    
    
    
    
    jpt(&dtb[..dtb_len])?;
    
    
    
    Ok(dtb_len)
}






#[derive(Debug, Clone)]
pub struct GuestLoadConfig {
    
    pub ram_base: u64,
    
    pub ram_size: u64,
    
    pub cmdline: String,
    
    pub trap_mmio: Vec<(u64, u64)>,
    
    pub trap_smc: bool,
    
    pub trap_wfi: bool,
}

impl Default for GuestLoadConfig {
    fn default() -> Self {
        Self {
            ram_base: CBY_,
            ram_size: BUC_,
            cmdline: String::from("console=ttyAMA0 earlycon=pl011,0x09000000 earlyprintk"),
            trap_mmio: vec![
                
                (0x0800_0000, 0x0001_0000),  
                (0x0801_0000, 0x0001_0000),  
                (0x0900_0000, 0x0000_1000),  
                (0x0901_0000, 0x0000_1000),  
                (0x0A00_0000, 0x0000_0200),  
                (0x0A00_0200, 0x0000_0200),  
            ],
            trap_smc: true,
            trap_wfi: false,
        }
    }
}


#[derive(Debug)]
pub struct Pb {
    
    pub kernel_addr: u64,
    
    pub kernel_size: usize,
    
    pub dtb_addr: u64,
    
    pub dtb_size: usize,
    
    pub initrd_addr: Option<u64>,
    
    pub initrd_size: Option<usize>,
    
    pub header: Arm64ImageHeader,
    
    pub hv_config: Ic,
}










pub fn mzs(
    kernel_data: &[u8],
    dtb_data: &[u8],
    bck: Option<&[u8]>,
    config: &GuestLoadConfig,
) -> Result<Pb, String> {
    
    if kernel_data.len() < 64 {
        return Err(String::from("Kernel image too small"));
    }
    if kernel_data.len() > BBU_ {
        return Err(format!("Kernel too large: {}MB (max {}MB)",
            kernel_data.len() / (1024*1024), BBU_ / (1024*1024)));
    }

    let header = Arm64ImageHeader::parse(kernel_data)
        .map_err(|e| String::from(e))?;

    
    jpt(dtb_data).map_err(|e| String::from(e))?;
    if dtb_data.len() > BBO_ {
        return Err(format!("DTB too large: {}KB (max {}KB)",
            dtb_data.len() / 1024, BBO_ / 1024));
    }

    
    if let Some(initrd) = bck {
        if initrd.len() > BBT_ {
            return Err(format!("initrd too large: {}MB (max {}MB)",
                initrd.len() / (1024*1024), BBT_ / (1024*1024)));
        }
    }

    
    let kernel_addr = config.ram_base + AFL_;
    let dtb_addr = config.ram_base + ACV_;
    let initrd_addr = config.ram_base + AFC_;

    
    let bhk = kernel_addr + header.effective_size(kernel_data.len()) as u64;
    if bhk > dtb_addr {
        return Err(format!("Kernel too large ({}MB), overlaps DTB region",
            kernel_data.len() / (1024*1024)));
    }
    if let Some(initrd) = bck {
        let czt = initrd_addr + initrd.len() as u64;
        let ixu = config.ram_base + config.ram_size;
        if czt > ixu {
            return Err(format!("initrd doesn't fit in guest RAM (need {}MB, have {}MB free)",
                initrd.len() / (1024*1024),
                (ixu - initrd_addr) / (1024*1024)));
        }
    }

    
    let kernel_size = header.effective_size(kernel_data.len());
    unsafe {
        ptr::copy_nonoverlapping(
            kernel_data.as_ptr(),
            kernel_addr as *mut u8,
            kernel_size,
        );
    }

    
    
    let mut ekw = [0u8; 1048576]; 
    let ftm = dtb_data.len().min(ekw.len());
    ekw[..ftm].copy_from_slice(&dtb_data[..ftm]);

    
    let mpx = bck.map(|d| initrd_addr + d.len() as u64);
    let hud = nrv(
        &mut ekw,
        ftm,
        bck.map(|_| initrd_addr),
        mpx,
        if config.cmdline.is_empty() { None } else { Some(&config.cmdline) },
    ).map_err(|e| String::from(e))?;

    unsafe {
        ptr::copy_nonoverlapping(
            ekw.as_ptr(),
            dtb_addr as *mut u8,
            hud,
        );
    }

    
    let (initrd_result_addr, initrd_result_size) = if let Some(initrd) = bck {
        unsafe {
            ptr::copy_nonoverlapping(
                initrd.as_ptr(),
                initrd_addr as *mut u8,
                initrd.len(),
            );
        }
        (Some(initrd_addr), Some(initrd.len()))
    } else {
        (None, None)
    };

    
    #[cfg(target_arch = "aarch64")]
    unsafe {
        
        
        core::arch::asm!(
            "dsb ish",
            "ic ialluis",    
            "dsb ish",
            "isb",
            options(nomem, nostack)
        );
    }

    
    let hv_config = Ic {
        guest_entry: kernel_addr,
        guest_dtb: dtb_addr,
        guest_ram_base: config.ram_base,
        guest_ram_size: config.ram_size,
        trapped_mmio: config.trap_mmio.clone(),
        trap_smc: config.trap_smc,
        trap_wfi: config.trap_wfi,
    };

    Ok(Pb {
        kernel_addr,
        kernel_size,
        dtb_addr,
        dtb_size: hud,
        initrd_addr: initrd_result_addr,
        initrd_size: initrd_result_size,
        header,
        hv_config,
    })
}


pub fn lxn(result: &Pb) -> String {
    let mut j = String::new();
    j.push_str("=== ARM64 Guest Loaded ===\n");
    j.push_str(&format!("  Kernel:  0x{:08X} ({} KB)\n",
        result.kernel_addr, result.kernel_size / 1024));
    j.push_str(&format!("  DTB:     0x{:08X} ({} KB)\n",
        result.dtb_addr, result.dtb_size / 1024));
    if let (Some(addr), Some(size)) = (result.initrd_addr, result.initrd_size) {
        j.push_str(&format!("  initrd:  0x{:08X} ({} KB)\n", addr, size / 1024));
    }
    j.push_str(&format!("  Entry:   0x{:08X}\n", result.hv_config.guest_entry));
    j.push_str(&format!("  RAM:     0x{:08X} - 0x{:08X} ({} MB)\n",
        result.hv_config.guest_ram_base,
        result.hv_config.guest_ram_base + result.hv_config.guest_ram_size,
        result.hv_config.guest_ram_size / (1024*1024)));
    j.push_str(&format!("  Endian:  {}\n",
        if result.header.is_little_endian() { "Little (LE)" } else { "Big (BE)" }));
    j.push_str(&format!("  MMIO traps: {} regions\n",
        result.hv_config.trapped_mmio.len()));
    for (base, size) in &result.hv_config.trapped_mmio {
        j.push_str(&format!("    0x{:08X} - 0x{:08X} ({})\n",
            base, base + size, super::mmio_spy::btg(*base)));
    }
    j.push_str(&format!("  SMC trap: {}\n",
        if result.hv_config.trap_smc { "ON" } else { "OFF" }));
    j
}







pub fn onl(ram_base: u64, ram_size: u64) -> Result<Pb, String> {
    
    
    
    
    
    let mut dfo = [0u8; 72];
    
    
    
    dfo[0..4].copy_from_slice(&0x14000010u32.to_le_bytes());
    
    
    
    dfo[0x10..0x18].copy_from_slice(&72u64.to_le_bytes());
    
    
    dfo[0x38..0x3C].copy_from_slice(&AMO_.to_le_bytes());
    
    
    
    dfo[64..68].copy_from_slice(&0xD503207Fu32.to_le_bytes()); 
    dfo[68..72].copy_from_slice(&0x17FFFFFFu32.to_le_bytes()); 
    
    
    let mut buj = [0u8; 48];
    buj[0..4].copy_from_slice(&KT_.to_be_bytes());     
    buj[4..8].copy_from_slice(&48u32.to_be_bytes());          
    buj[8..12].copy_from_slice(&40u32.to_be_bytes());         
    buj[12..16].copy_from_slice(&44u32.to_be_bytes());        
    buj[16..20].copy_from_slice(&28u32.to_be_bytes());        
    buj[20..24].copy_from_slice(&17u32.to_be_bytes());        
    buj[24..28].copy_from_slice(&16u32.to_be_bytes());        
    
    
    buj[40..44].copy_from_slice(&ADQ_.to_be_bytes());
    
    
    let mut config = GuestLoadConfig::default();
    config.ram_base = ram_base;
    config.ram_size = ram_size;
    config.trap_wfi = true; 
    config.cmdline = String::new();
    
    mzs(&dfo, &buj, None, &config)
}
