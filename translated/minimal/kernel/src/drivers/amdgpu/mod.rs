
















pub mod regs;
pub mod dcn;
pub mod compute;
pub mod sdma;
pub mod neural;
pub mod firmware;

use alloc::string::String;
use alloc::format;
use alloc::vec::Vec;
use core::sync::atomic::{AtomicBool, Ordering};
use spin::Mutex;

use crate::pci::{self, L};
use crate::memory;






pub const AMH_: u16 = 0x1002;


pub const CKS_: &[u16] = &[
    0x7310, 
    0x7312, 
    0x7318, 
    0x7319, 
    0x731A, 
    0x731B, 
    0x731E, 
    0x731F, 
    0x7340, 
    0x7341, 
    0x7347, 
    0x734F, 
];


pub mod family {
    pub const BXV_:  u32 = 0;
    pub const ATQ_:       u32 = 143; 
    pub const ATR_:     u32 = 144; 
    pub const BXT_:       u32 = 141; 
    pub const BXU_:       u32 = 142; 
}


pub mod bar {
    
    pub const BCU_: usize = 0;
    
    pub const BKZ_: usize = 2;
    
    pub const Xn: usize = 4;
}


pub mod pcie_cap {
    pub const CMC_: u8 = 0x10;
    pub const Mp: u8 = 0x05;
    pub const Pl: u8 = 0x11;
    pub const EEP_: u8 = 0x01;
}






#[derive(Debug, Clone)]
pub struct Hz {
    
    pub vendor_id: u16,
    pub device_id: u16,
    
    pub bus: u8,
    pub device: u8,
    pub function: u8,
    
    pub revision: u8,
    
    pub asic_family: u32,
    
    pub chip_external_rev: u32,
    
    pub family_name: &'static str,
    
    pub vram_size: u64,
    
    pub vram_type: &'static str,
    
    pub mmio_base_phys: u64,
    
    pub mmio_base_virt: u64,
    
    pub mmio_size: u64,
    
    pub vram_aperture_phys: u64,
    
    pub vram_aperture_size: u64,
    
    pub pcie_link_speed: u8,
    
    pub pcie_link_width: u8,
    
    pub has_msi: bool,
    
    pub has_msix: bool,
    
    pub compute_units: u32,
    
    pub gpu_clock_mhz: u32,
}

impl Hz {
    
    pub fn gpu_name(&self) -> &'static str {
        match self.device_id {
            0x731F => match self.revision {
                0xC1 => "AMD Radeon RX 5700 XT",
                0xC0 | 0xC4 => "AMD Radeon RX 5700",
                0xC2 | 0xC3 => "AMD Radeon RX 5600 XT",
                _ => "AMD Radeon Navi 10",
            },
            0x7340 => "AMD Radeon RX 5500 XT",
            0x7341 => "AMD Radeon Pro W5500",
            0x7312 => "AMD Radeon Pro W5700X",
            _ => "AMD Radeon (Unknown Navi)",
        }
    }

    
    pub fn vram_string(&self) -> String {
        if self.vram_size >= 1024 * 1024 * 1024 {
            format!("{} GB", self.vram_size / (1024 * 1024 * 1024))
        } else if self.vram_size >= 1024 * 1024 {
            format!("{} MB", self.vram_size / (1024 * 1024))
        } else if self.vram_size > 0 {
            format!("{} KB", self.vram_size / 1024)
        } else {
            String::from("Unknown")
        }
    }

    
    pub fn pcie_link_string(&self) -> String {
        let speed = match self.pcie_link_speed {
            1 => "2.5 GT/s (Gen1)",
            2 => "5.0 GT/s (Gen2)",
            3 => "8.0 GT/s (Gen3)",
            4 => "16.0 GT/s (Gen4)",
            _ => "Unknown",
        };
        format!("PCIe x{} {}", self.pcie_link_width, speed)
    }
}


struct Wg {
    
    initialized: bool,
    
    gpu_info: Option<Hz>,
}

static AVU_: Mutex<Wg> = Mutex::new(Wg {
    initialized: false,
    gpu_info: None,
});

static OF_: AtomicBool = AtomicBool::new(false);









#[inline]
pub unsafe fn kj(mmio_base: u64, offset: u32) -> u32 {
    let addr = mmio_base + offset as u64;
    core::ptr::read_volatile(addr as *const u32)
}





#[inline]
pub unsafe fn ib(mmio_base: u64, offset: u32, value: u32) {
    let addr = mmio_base + offset as u64;
    core::ptr::write_volatile(addr as *mut u32, value);
}






#[inline]
pub unsafe fn inz(mmio_base: u64, reg: u32) -> u32 {
    
    ib(mmio_base, regs::BCX_, reg);
    
    kj(mmio_base, regs::BCW_)
}





#[inline]
pub unsafe fn nfq(mmio_base: u64, reg: u32, value: u32) {
    ib(mmio_base, regs::BCX_, reg);
    ib(mmio_base, regs::BCW_, value);
}







fn fru(s: &L, bar_index: usize) -> u64 {
    let bku = 0x10 + (bar_index as u8 * 4);
    
    
    let ccb = pci::ms(s.bus, s.device, s.function, bku);
    
    
    pci::qj(s.bus, s.device, s.function, bku, 0xFFFFFFFF);
    
    
    let agx = pci::ms(s.bus, s.device, s.function, bku);
    
    
    pci::qj(s.bus, s.device, s.function, bku, ccb);
    
    if agx == 0 {
        return 0;
    }
    
    
    if ccb & 1 == 0 {
        
        let mask = agx & 0xFFFFFFF0;
        if mask == 0 {
            return 0;
        }
        let size = (!mask).wrapping_add(1) as u64;
        
        
        let bqj = (ccb >> 1) & 0x3;
        if bqj == 2 && bar_index < 5 {
            
            let egd = bku + 4;
            let gle = pci::ms(s.bus, s.device, s.function, egd);
            pci::qj(s.bus, s.device, s.function, egd, 0xFFFFFFFF);
            let gqk = pci::ms(s.bus, s.device, s.function, egd);
            pci::qj(s.bus, s.device, s.function, egd, gle);
            
            let iak = ((gqk as u64) << 32) | (mask as u64);
            if iak == 0 {
                return size;
            }
            (!iak).wrapping_add(1)
        } else {
            size
        }
    } else {
        
        let mask = agx & 0xFFFFFFFC;
        if mask == 0 {
            return 0;
        }
        ((!mask).wrapping_add(1) & 0xFFFF) as u64
    }
}






fn ocx(s: &L) -> (u8, u8) {
    if let Some(pcie_cap_offset) = pci::bsq(s, pcie_cap::CMC_) {
        
        let cbl = pci::vf(s.bus, s.device, s.function, pcie_cap_offset + 0x12);
        let speed = (cbl & 0xF) as u8;        
        let width = ((cbl >> 4) & 0x3F) as u8; 
        (speed, width)
    } else {
        (0, 0)
    }
}







fn gqe(mmio_base: u64) -> (u32, u32, u32, u32, u32) {
    let family;
    let hxi;
    let compute_units;
    let cyw;
    let jqq;

    unsafe {
        
        
        
        let enw = kj(mmio_base, regs::BZU_);
        crate::serial_println!("[AMDGPU] GC_VERSION raw: {:#010X}", enw);
        
        
        let oho = kj(mmio_base, regs::CSK_);
        crate::serial_println!("[AMDGPU] RLC_PG_CNTL raw: {:#010X}", oho);
        
        
        let mbg = kj(mmio_base, regs::BZS_);
        crate::serial_println!("[AMDGPU] GC_CAC raw: {:#010X}", mbg);
        
        
        let hjy = kj(mmio_base, regs::BOS_);
        crate::serial_println!("[AMDGPU] CC_GC_SHADER_ARRAY_CONFIG: {:#010X}", hjy);
        
        
        let mbh = kj(mmio_base, regs::BZT_);
        crate::serial_println!("[AMDGPU] GC_USER_SHADER_ARRAY_CONFIG: {:#010X}", mbh);
        
        
        let dqz = kj(mmio_base, regs::LB_);
        crate::serial_println!("[AMDGPU] GRBM_STATUS: {:#010X}", dqz);
        
        
        let oty = kj(mmio_base, regs::BDB_);
        crate::serial_println!("[AMDGPU] MP1 firmware version: {:#010X}", oty);
        
        
        let ols = kj(mmio_base, regs::CUG_);
        let olt = kj(mmio_base, regs::CUH_);
        let olu = kj(mmio_base, regs::CUI_);
        crate::serial_println!("[AMDGPU] SCRATCH[0]={:#010X} [1]={:#010X} [7]={:#010X}", 
            ols, olt, olu);
        
        
        let imw = kj(mmio_base, regs::CJK_);
        crate::serial_println!("[AMDGPU] MC_ARB_RAMCFG: {:#010X}", imw);
        jqq = ((imw >> 8) & 0xFF) as u32;
        
        
        
        let iau = (enw >> 16) & 0xFF;
        let iav = (enw >> 8) & 0xFF;
        
        family = if iau == 10 && iav == 1 {
            family::ATQ_ 
        } else if iau == 10 && iav == 3 {
            family::ATR_ 
        } else {
            family::BXV_
        };
        
        hxi = enw; 
        
        
        
        let fpi = hjy & 0xFFFF;
        compute_units = if fpi != 0xFFFF && fpi != 0xFFFFFFFF {
            
            let disabled = fpi.count_ones();
            40u32.saturating_sub(disabled)
        } else {
            0 
        };

        
        let ezn = kj(mmio_base, regs::BOY_);
        crate::serial_println!("[AMDGPU] CG_CLKPIN_CNTL_2: {:#010X}", ezn);
        cyw = if ezn != 0 && ezn != 0xFFFFFFFF {
            ezn & 0xFFFF 
        } else {
            0
        };
    }

    (family, hxi, compute_units, cyw, jqq)
}


fn gqj(mmio_base: u64) -> u64 {
    unsafe {
        
        let euf = kj(mmio_base, regs::CJM_);
        let ghd = kj(mmio_base, regs::CJN_);
        crate::serial_println!("[AMDGPU] MC_VM_FB_LOCATION BASE={:#010X} TOP={:#010X}", 
            euf, ghd);
        
        if ghd > euf && euf != 0xFFFFFFFF {
            
            let kaf = (euf & 0xFFFF) as u64;
            let plh = (ghd & 0xFFFF) as u64;
            let size = (plh - kaf + 1) * 1024 * 1024;
            if size > 0 && size <= 32 * 1024 * 1024 * 1024 {
                return size;
            }
        }
        
        
        let eis = kj(mmio_base, regs::BRI_);
        crate::serial_println!("[AMDGPU] CONFIG_MEMSIZE: {:#010X}", eis);
        if eis != 0 && eis != 0xFFFFFFFF {
            return (eis as u64) * 1024 * 1024; 
        }
        
        
        0
    }
}


fn ptf(mmio_base: u64) -> &'static str {
    unsafe {
        let imx = kj(mmio_base, regs::CJL_);
        crate::serial_println!("[AMDGPU] MC_SEQ_MISC0: {:#010X}", imx);
        
        
        match (imx >> 28) & 0xF {
            1 => "DDR2",
            2 => "DDR3",
            3 => "GDDR3",
            4 => "GDDR4",
            5 => "GDDR5",
            6 => "HBM",
            7 => "HBM2",
            8 | 9 => "GDDR6",
            _ => {
                
                "GDDR6 (assumed)"
            }
        }
    }
}







pub fn probe() -> Option<L> {
    
    for &dev_id in CKS_ {
        if let Some(s) = pci::lvs(AMH_, dev_id) {
            crate::log!("[AMDGPU] Found Navi GPU: {:04X}:{:04X} at {:02X}:{:02X}.{}", 
                s.vendor_id, s.device_id, s.bus, s.device, s.function);
            return Some(s);
        }
    }
    
    
    let bbd = pci::bsp(pci::class::Du);
    for s in bbd {
        if s.vendor_id == AMH_ {
            crate::log!("[AMDGPU] Found AMD Display: {:04X}:{:04X} at {:02X}:{:02X}.{}", 
                s.vendor_id, s.device_id, s.bus, s.device, s.function);
            return Some(s);
        }
    }
    
    None
}









pub fn init() {
    crate::log!("[AMDGPU] ═══════════════════════════════════════════════════");
    crate::log!("[AMDGPU] AMD GPU Driver — Phase 1: PCIe Discovery & MMIO");
    crate::log!("[AMDGPU] ═══════════════════════════════════════════════════");
    
    
    let s = match probe() {
        Some(d) => d,
        None => {
            crate::log!("[AMDGPU] No AMD GPU detected on PCI bus");
            crate::log!("[AMDGPU] (This is normal in VMs without GPU passthrough)");
            
            let bbd = pci::bsp(pci::class::Du);
            if bbd.is_empty() {
                crate::log!("[AMDGPU] No display controllers found at all");
            } else {
                for d in &bbd {
                    crate::log!("[AMDGPU] Display: {:04X}:{:04X} {} at {:02X}:{:02X}.{}", 
                        d.vendor_id, d.device_id, d.vendor_name(),
                        d.bus, d.device, d.function);
                }
            }
            return;
        }
    };
    
    
    crate::log!("[AMDGPU] Enabling PCI bus mastering and memory space...");
    pci::bzi(&s);
    pci::bzj(&s);
    
    
    let cmd = pci::vf(s.bus, s.device, s.function, 0x04);
    let status = pci::vf(s.bus, s.device, s.function, 0x06);
    crate::log!("[AMDGPU] PCI Command: {:#06X}  Status: {:#06X}", cmd, status);
    
    
    crate::log!("[AMDGPU] Detecting BARs...");
    
    let cbs = s.bar_address(bar::BCU_).unwrap_or(0);
    let mmio_size = fru(&s, bar::BCU_);
    
    let dgk = s.bar_address(bar::BKZ_).unwrap_or(0);
    let csn = fru(&s, bar::BKZ_);
    
    let lgx = s.bar_address(bar::Xn).unwrap_or(0);
    let htd = fru(&s, bar::Xn);
    
    crate::log!("[AMDGPU] BAR0 (MMIO):     phys={:#014X} size={:#X} ({} KB)", 
        cbs, mmio_size, mmio_size / 1024);
    crate::log!("[AMDGPU] BAR2 (VRAM):     phys={:#014X} size={:#X} ({} MB)", 
        dgk, csn, csn / (1024 * 1024));
    crate::log!("[AMDGPU] BAR4 (Doorbell): phys={:#014X} size={:#X} ({} KB)", 
        lgx, htd, htd / 1024);
    
    if cbs == 0 {
        crate::log!("[AMDGPU] ERROR: BAR0 (MMIO registers) not available!");
        return;
    }
    
    
    let daw = if mmio_size > 0 { mmio_size as usize } else { 512 * 1024 }; 
    crate::log!("[AMDGPU] Mapping MMIO: {:#X} -> {} pages...", cbs, daw / 4096);
    
    let akb = match memory::yv(cbs, daw) {
        Ok(virt) => {
            crate::log!("[AMDGPU] MMIO mapped at virtual {:#014X}", virt);
            virt
        }
        Err(e) => {
            crate::log!("[AMDGPU] ERROR: Failed to map MMIO: {}", e);
            return;
        }
    };
    
    
    let (cbj, dth) = ocx(&s);
    let has_msi = pci::bsq(&s, pcie_cap::Mp).is_some();
    let has_msix = pci::bsq(&s, pcie_cap::Pl).is_some();
    
    crate::log!("[AMDGPU] PCIe: Gen{} x{}  MSI:{}  MSI-X:{}", 
        cbj, dth,
        if has_msi { "yes" } else { "no" },
        if has_msix { "yes" } else { "no" });
    
    
    crate::log!("[AMDGPU] Reading GPU identity registers...");
    let (asic_family, chip_ext_rev, compute_units, cyw, _vram_width) = 
        gqe(akb);
    
    let family_name = match asic_family {
        family::ATQ_ => "Navi 10 (RDNA 1)",
        family::ATR_ => "Navi 14 (RDNA 1)",
        family::BXT_ => "Vega 10 (GCN 5)",
        family::BXU_ => "Raven (Vega APU)",
        _ => "Unknown",
    };
    
    
    let vram_size = gqj(akb);
    let vram_type = ptf(akb);
    
    
    let info = Hz {
        vendor_id: s.vendor_id,
        device_id: s.device_id,
        bus: s.bus,
        device: s.device,
        function: s.function,
        revision: s.revision,
        asic_family,
        chip_external_rev: chip_ext_rev,
        family_name,
        vram_size,
        vram_type,
        mmio_base_phys: cbs,
        mmio_base_virt: akb,
        mmio_size: mmio_size,
        vram_aperture_phys: dgk,
        vram_aperture_size: csn,
        pcie_link_speed: cbj,
        pcie_link_width: dth,
        has_msi,
        has_msix,
        compute_units,
        gpu_clock_mhz: cyw,
    };
    
    
    crate::log!("[AMDGPU] ───────────────────────────────────────────────────");
    crate::log!("[AMDGPU] GPU: {}", info.gpu_name());
    crate::log!("[AMDGPU] PCI: {:04X}:{:04X} rev {:02X} at {:02X}:{:02X}.{}", 
        info.vendor_id, info.device_id, info.revision,
        info.bus, info.device, info.function);
    crate::log!("[AMDGPU] Family: {}", family_name);
    crate::log!("[AMDGPU] VRAM: {} ({})", info.vram_string(), vram_type);
    crate::log!("[AMDGPU] {}", info.pcie_link_string());
    if compute_units > 0 {
        crate::log!("[AMDGPU] Compute Units: {}", compute_units);
    }
    if cyw > 0 {
        crate::log!("[AMDGPU] GPU Clock: {} MHz", cyw);
    }
    crate::log!("[AMDGPU] MMIO: {:#X} ({} KB mapped)", akb, daw / 1024);
    crate::log!("[AMDGPU] ───────────────────────────────────────────────────");
    crate::log!("[AMDGPU] Phase 1 complete — GPU discovered and identified");
    
    
    let mut state = AVU_.lock();
    state.initialized = true;
    state.gpu_info = Some(info);
    OF_.store(true, Ordering::SeqCst);
    drop(state);
    
    
    firmware::init(akb);
    
    
    dcn::init(akb);
    
    
    sdma::init(akb);
    
    
    compute::init(akb);
}






pub fn aud() -> bool {
    OF_.load(Ordering::Relaxed)
}


pub fn rk() -> Option<Hz> {
    AVU_.lock().gpu_info.clone()
}


pub fn summary() -> String {
    if let Some(info) = rk() {
        format!("{} | {} {} | {} | CU:{}", 
            info.gpu_name(),
            info.vram_string(),
            info.vram_type,
            info.pcie_link_string(),
            if info.compute_units > 0 { 
                format!("{}", info.compute_units) 
            } else { 
                String::from("?") 
            })
    } else {
        String::from("No AMD GPU detected")
    }
}


pub fn info_lines() -> Vec<String> {
    let mut lines = Vec::new();
    
    if let Some(info) = rk() {
        lines.push(format!("╔══════════════════════════════════════════════╗"));
        lines.push(format!("║        AMD GPU — {}        ║", info.gpu_name()));
        lines.push(format!("╠══════════════════════════════════════════════╣"));
        lines.push(format!("║ PCI ID:    {:04X}:{:04X} rev {:02X}                 ║", 
            info.vendor_id, info.device_id, info.revision));
        lines.push(format!("║ Location:  {:02X}:{:02X}.{}                          ║",
            info.bus, info.device, info.function));
        lines.push(format!("║ Family:    {}               ║", info.family_name));
        lines.push(format!("║ VRAM:      {} ({})            ║", info.vram_string(), info.vram_type));
        lines.push(format!("║ PCIe:      {}              ║", info.pcie_link_string()));
        if info.compute_units > 0 {
            lines.push(format!("║ CUs:       {}                               ║", info.compute_units));
        }
        lines.push(format!("║ MMIO:      {:#X} ({} KB)        ║", 
            info.mmio_base_virt, info.mmio_size / 1024));
        lines.push(format!("║ VRAM Apt:  {:#X} ({} MB)     ║",
            info.vram_aperture_phys, info.vram_aperture_size / (1024 * 1024)));
        lines.push(format!("║ MSI: {}  MSI-X: {}                        ║",
            if info.has_msi { "Yes" } else { "No " },
            if info.has_msix { "Yes" } else { "No " }));
        lines.push(format!("╚══════════════════════════════════════════════╝"));
    } else {
        lines.push(String::from("No AMD GPU detected"));
        lines.push(String::from("(GPU passthrough or bare metal required)"));
    }
    
    lines
}
