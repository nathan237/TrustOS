
















































use alloc::string::String;
use alloc::format;
use alloc::vec::Vec;
use core::sync::atomic::{AtomicBool, Ordering};
use spin::Mutex;

use super::{kj, ib, inz, nfq};
use super::regs;
use crate::memory;






const AEF_: &str = "/lib/firmware/amdgpu";


const BZH_: &str = "navi10_pfp.bin";
const BZE_: &str = "navi10_me.bin";
const BZC_: &str = "navi10_ce.bin";
const BZF_: &str = "navi10_mec.bin";
const BZG_: &str = "navi10_mec2.bin";
const AUS_: &str = "navi10_rlc.bin";
const BZI_: &str = "navi10_sdma.bin";
const BZJ_: &str = "navi10_sdma1.bin";


const DGH_: u32 = 0x4D_44_41; 








const DQG_: usize = 256; 


const BZD_: u64 = 5_000_000;







const EGI_: u32 = 0x4E08;

const CSI_: u32 = 0x4E20;

const BGR_: u32 = 0x4E24;

const EGG_: u32 = 0x4E28;

const EGH_: u32 = 0x4E2C;

const CSL_: u32 = 0x4E0C;

const AJD_: u32 = 0x4E00;

const BGS_: u32 = 0x4E04;

const CSJ_: u32 = 0x4E30;



const BSA_: u32 = 0x8A14;

const BSB_: u32 = 0x8A18;

const BRY_: u32 = 0x8A1C;

const BRZ_: u32 = 0x8A20;

const BRP_: u32 = 0x8A24;

const BRQ_: u32 = 0x8A28;

const NM_: u32 = 0x86D8;


const ARG_: u32 = 1 << 28;
const ARH_: u32 = 1 << 26;
const ARB_: u32 = 1 << 24;



const BRU_: u32 = 0x8A30;

const BRV_: u32 = 0x8A34;

const BRW_: u32 = 0x8A38;

const BRX_: u32 = 0x8A3C;



const CUQ_: u32 = 0x4D88;

const CUR_: u32 = 0x4D8C;

const CUU_: u32 = 0x4E88;

const CUV_: u32 = 0x4E8C;



const BSC_: u32 = 0x8044;

const DLB_: u32 = 0x8054;






#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FwStatus {
    
    NotFound,
    
    Loaded,
    
    Running,
    
    Failed,
}


pub struct Yx {
    pub rlc: FwStatus,
    pub pfp: FwStatus,
    pub me: FwStatus,
    pub ce: FwStatus,
    pub mec1: FwStatus,
    pub mec2: FwStatus,
    pub sdma0: FwStatus,
    pub sdma1: FwStatus,
    pub mmio_base: u64,
}

static AEG_: Mutex<Yx> = Mutex::new(Yx {
    rlc: FwStatus::NotFound,
    pfp: FwStatus::NotFound,
    me: FwStatus::NotFound,
    ce: FwStatus::NotFound,
    mec1: FwStatus::NotFound,
    mec2: FwStatus::NotFound,
    sdma0: FwStatus::NotFound,
    sdma1: FwStatus::NotFound,
    mmio_base: 0,
});

static AUR_: AtomicBool = AtomicBool::new(false);






fn cdc(name: &str) -> Option<Vec<u8>> {
    let path = format!("{}/{}", AEF_, name);
    crate::ramfs::bh(|fs| {
        fs.read_file(&path).ok().map(|data| data.to_vec())
    })
}




fn ccg(data: &[u8]) -> &[u8] {
    if data.len() < 16 {
        return data;
    }
    
    
    
    
    let lmt = u32::from_le_bytes([data[0], data[1], data[2], data[3]]);
    let bza = u32::from_le_bytes([data[4], data[5], data[6], data[7]]);
    
    
    
    
    
    
    
    
    
    
    
    
    
    
    
    
    if bza >= 4 && bza <= 1024 {
        let czd = (bza as usize) * 4;
        if czd < data.len() {
            
            if data.len() >= 24 {
                let fdv = u32::from_le_bytes([
                    data[20], data[21], data[22], data[23]
                ]) as usize;
                if fdv > 0 && fdv < data.len() {
                    crate::serial_println!("[AMDGPU-FW] Header: size={}dw, ucode_offset={:#X}", 
                        bza, fdv);
                    return &data[fdv..];
                }
            }
            crate::serial_println!("[AMDGPU-FW] Header: {}dw ({}B), ucode starts at {:#X}", 
                bza, czd, czd);
            return &data[czd..];
        }
    }
    
    
    crate::serial_println!("[AMDGPU-FW] No header detected, using raw blob");
    data
}









fn naf(mmio: u64, asp: &[u8]) -> Result<(), &'static str> {
    let aau = asp.len() / 4;
    if aau == 0 { return Err("RLC firmware is empty"); }
    
    crate::log!("[AMDGPU-FW] Loading RLC: {} bytes ({} DWORDs)", asp.len(), aau);
    
    unsafe {
        
        let jba = kj(mmio, AJD_);
        ib(mmio, AJD_, jba & !(1u32)); 
        
        
        for _ in 0..10000 {
            let stat = kj(mmio, BGS_);
            if stat & 1 == 0 { break; } 
            core::hint::spin_loop();
        }
        
        
        ib(mmio, CSJ_, aau as u32);
        
        
        ib(mmio, CSI_, 0);
        
        
        for i in 0..aau {
            let offset = i * 4;
            if offset + 4 > asp.len() { break; }
            let qx = u32::from_le_bytes([
                asp[offset], asp[offset+1], asp[offset+2], asp[offset+3]
            ]);
            ib(mmio, BGR_, qx);
        }
        
        
        ib(mmio, AJD_, jba | 1); 
        
        
        for _ in 0..BZD_ {
            let stat = kj(mmio, BGS_);
            if stat & 1 != 0 {
                crate::log!("[AMDGPU-FW] RLC running (stat={:#X})", stat);
                return Ok(());
            }
            core::hint::spin_loop();
        }
    }
    
    crate::log!("[AMDGPU-FW] RLC loaded ({} DWORDs) — verifying...", aau);
    Ok(())
}







fn mzv(mmio: u64, pfp: Option<&[u8]>, me: Option<&[u8]>, ce: Option<&[u8]>) -> Result<(), &'static str> {
    unsafe {
        
        let inb = kj(mmio, NM_);
        ib(mmio, NM_, inb | ARG_ | ARH_ | ARB_);
        
        
        for _ in 0..10000 { core::hint::spin_loop(); }
        
        
        if let Some(fo) = pfp {
            let aau = fo.len() / 4;
            crate::log!("[AMDGPU-FW] Loading PFP: {} DWORDs", aau);
            ib(mmio, BSA_, 0);
            for i in 0..aau {
                let off = i * 4;
                if off + 4 > fo.len() { break; }
                let qx = u32::from_le_bytes([fo[off], fo[off+1], fo[off+2], fo[off+3]]);
                ib(mmio, BSB_, qx);
            }
        }
        
        
        if let Some(fo) = me {
            let aau = fo.len() / 4;
            crate::log!("[AMDGPU-FW] Loading ME: {} DWORDs", aau);
            ib(mmio, BRY_, 0);
            for i in 0..aau {
                let off = i * 4;
                if off + 4 > fo.len() { break; }
                let qx = u32::from_le_bytes([fo[off], fo[off+1], fo[off+2], fo[off+3]]);
                ib(mmio, BRZ_, qx);
            }
        }
        
        
        if let Some(fo) = ce {
            let aau = fo.len() / 4;
            crate::log!("[AMDGPU-FW] Loading CE: {} DWORDs", aau);
            ib(mmio, BRP_, 0);
            for i in 0..aau {
                let off = i * 4;
                if off + 4 > fo.len() { break; }
                let qx = u32::from_le_bytes([fo[off], fo[off+1], fo[off+2], fo[off+3]]);
                ib(mmio, BRQ_, qx);
            }
        }
        
        
        ib(mmio, NM_, inb & !(ARG_ | ARH_ | ARB_));
        
        
        for _ in 0..100000 {
            let ckf = kj(mmio, regs::LB_);
            if ckf & regs::AES_ == 0 {
                crate::log!("[AMDGPU-FW] CP GFX engines started");
                return Ok(());
            }
            core::hint::spin_loop();
        }
    }
    
    crate::log!("[AMDGPU-FW] CP GFX loaded — engine busy (may need time)");
    Ok(())
}





fn nae(mmio: u64, mec1: Option<&[u8]>, mec2: Option<&[u8]>) -> Result<(), &'static str> {
    unsafe {
        
        let ine = kj(mmio, regs::ACH_);
        ib(mmio, regs::ACH_, ine | (1 << 28)); 
        
        for _ in 0..10000 { core::hint::spin_loop(); }
        
        
        if let Some(fo) = mec1 {
            let aau = fo.len() / 4;
            crate::log!("[AMDGPU-FW] Loading MEC1: {} DWORDs", aau);
            ib(mmio, BRU_, 0);
            for i in 0..aau {
                let off = i * 4;
                if off + 4 > fo.len() { break; }
                let qx = u32::from_le_bytes([fo[off], fo[off+1], fo[off+2], fo[off+3]]);
                ib(mmio, BRV_, qx);
            }
        }
        
        
        if let Some(fo) = mec2 {
            let aau = fo.len() / 4;
            crate::log!("[AMDGPU-FW] Loading MEC2: {} DWORDs", aau);
            ib(mmio, BRW_, 0);
            for i in 0..aau {
                let off = i * 4;
                if off + 4 > fo.len() { break; }
                let qx = u32::from_le_bytes([fo[off], fo[off+1], fo[off+2], fo[off+3]]);
                ib(mmio, BRX_, qx);
            }
        }
        
        
        ib(mmio, regs::ACH_, ine & !(1u32 << 28));
    }
    
    crate::log!("[AMDGPU-FW] MEC firmware loaded");
    Ok(())
}


fn ikx(mmio: u64, engine: usize, fo: &[u8]) -> Result<(), &'static str> {
    let aau = fo.len() / 4;
    if aau == 0 { return Err("SDMA firmware is empty"); }
    
    let (addr_reg, data_reg, f32_cntl) = match engine {
        0 => (CUQ_, CUR_, regs::BHH_),
        1 => (CUU_, CUV_, regs::BHJ_),
        _ => return Err("Invalid SDMA engine index"),
    };
    
    crate::log!("[AMDGPU-FW] Loading SDMA{}: {} DWORDs", engine, aau);
    
    unsafe {
        
        ib(mmio, f32_cntl, kj(mmio, f32_cntl) | 1); 
        for _ in 0..10000 { core::hint::spin_loop(); }
        
        
        ib(mmio, addr_reg, 0);
        
        
        for i in 0..aau {
            let off = i * 4;
            if off + 4 > fo.len() { break; }
            let qx = u32::from_le_bytes([fo[off], fo[off+1], fo[off+2], fo[off+3]]);
            ib(mmio, data_reg, qx);
        }
        
        
        ib(mmio, f32_cntl, kj(mmio, f32_cntl) & !1u32);
    }
    
    crate::log!("[AMDGPU-FW] SDMA{} firmware loaded", engine);
    Ok(())
}









pub fn init(mmio_base: u64) {
    crate::log!("[AMDGPU-FW] ═══════════════════════════════════════════════");
    crate::log!("[AMDGPU-FW] AMD GPU Firmware Loader — Navi 10");
    crate::log!("[AMDGPU-FW] ═══════════════════════════════════════════════");
    
    if mmio_base == 0 {
        crate::log!("[AMDGPU-FW] No MMIO base — skipping firmware init");
        return;
    }
    
    let mut state = AEG_.lock();
    state.mmio_base = mmio_base;
    
    
    let _ = crate::ramfs::bh(|fs| {
        let _ = fs.mkdir("/lib");
        let _ = fs.mkdir("/lib/firmware");
        let _ = fs.mkdir("/lib/firmware/amdgpu");
    });
    
    
    let gvl = unsafe { kj(mmio_base, regs::BDB_) };
    crate::log!("[AMDGPU-FW] SMU firmware version: {:#010X}", gvl);
    if gvl != 0 && gvl != 0xFFFFFFFF {
        crate::log!("[AMDGPU-FW] SMU is active — VBIOS has initialized power management");
    } else {
        crate::log!("[AMDGPU-FW] SMU not active — cold boot, firmware required");
    }
    
    
    let ckf = unsafe { kj(mmio_base, regs::LB_) };
    let fos = ckf & regs::AES_ != 0;
    let fzq = ckf & regs::AVV_ != 0;
    crate::log!("[AMDGPU-FW] Pre-load: GRBM={:#010X} CP_BUSY={} GUI_ACTIVE={}", 
        ckf, fos, fzq);
    
    
    
    
    if let Some(dm) = cdc(AUS_) {
        let asp = ccg(&dm);
        match naf(mmio_base, asp) {
            Ok(()) => state.rlc = FwStatus::Loaded,
            Err(e) => {
                crate::log!("[AMDGPU-FW] RLC load failed: {}", e);
                state.rlc = FwStatus::Failed;
            }
        }
    } else {
        crate::log!("[AMDGPU-FW] {} not found in {}", AUS_, AEF_);
        state.rlc = FwStatus::NotFound;
    }
    
    
    let ntt = cdc(BZH_);
    let ndw = cdc(BZE_);
    let khv = cdc(BZC_);
    
    let gmv = ntt.as_deref().map(ccg);
    let ghf = ndw.as_deref().map(ccg);
    let flb = khv.as_deref().map(ccg);
    
    if gmv.is_some() || ghf.is_some() || flb.is_some() {
        match mzv(mmio_base, gmv, ghf, flb) {
            Ok(()) => {
                if gmv.is_some() { state.pfp = FwStatus::Loaded; }
                if ghf.is_some() { state.me = FwStatus::Loaded; }
                if flb.is_some() { state.ce = FwStatus::Loaded; }
            }
            Err(e) => {
                crate::log!("[AMDGPU-FW] CP GFX load failed: {}", e);
                state.pfp = FwStatus::Failed;
                state.me = FwStatus::Failed;
                state.ce = FwStatus::Failed;
            }
        }
    } else {
        crate::log!("[AMDGPU-FW] No GFX CP firmware found (PFP/ME/CE)");
    }
    
    
    let ndz = cdc(BZF_);
    let nea = cdc(BZG_);
    
    let ghh = ndz.as_deref().map(ccg);
    let ghi = nea.as_deref().map(ccg);
    
    if ghh.is_some() || ghi.is_some() {
        match nae(mmio_base, ghh, ghi) {
            Ok(()) => {
                if ghh.is_some() { state.mec1 = FwStatus::Loaded; }
                if ghi.is_some() { state.mec2 = FwStatus::Loaded; }
            }
            Err(e) => {
                crate::log!("[AMDGPU-FW] MEC load failed: {}", e);
                state.mec1 = FwStatus::Failed;
            }
        }
    } else {
        crate::log!("[AMDGPU-FW] No MEC firmware found");
    }
    
    
    if let Some(dm) = cdc(BZI_) {
        let asp = ccg(&dm);
        match ikx(mmio_base, 0, asp) {
            Ok(()) => state.sdma0 = FwStatus::Loaded,
            Err(e) => {
                crate::log!("[AMDGPU-FW] SDMA0 load failed: {}", e);
                state.sdma0 = FwStatus::Failed;
            }
        }
    }
    
    if let Some(dm) = cdc(BZJ_) {
        let asp = ccg(&dm);
        match ikx(mmio_base, 1, asp) {
            Ok(()) => state.sdma1 = FwStatus::Loaded,
            Err(e) => {
                crate::log!("[AMDGPU-FW] SDMA1 load failed: {}", e);
                state.sdma1 = FwStatus::Failed;
            }
        }
    }
    
    
    let gga = [state.rlc, state.pfp, state.me, state.ce, 
                        state.mec1, state.mec2, state.sdma0, state.sdma1]
        .iter().filter(|&&j| j == FwStatus::Loaded).count();
    
    crate::log!("[AMDGPU-FW] ───────────────────────────────────────────────");
    crate::log!("[AMDGPU-FW] Firmware status ({}/8 loaded):", gga);
    crate::log!("[AMDGPU-FW]   RLC:   {:?}", state.rlc);
    crate::log!("[AMDGPU-FW]   PFP:   {:?}", state.pfp);
    crate::log!("[AMDGPU-FW]   ME:    {:?}", state.me);
    crate::log!("[AMDGPU-FW]   CE:    {:?}", state.ce);
    crate::log!("[AMDGPU-FW]   MEC1:  {:?}", state.mec1);
    crate::log!("[AMDGPU-FW]   MEC2:  {:?}", state.mec2);
    crate::log!("[AMDGPU-FW]   SDMA0: {:?}", state.sdma0);
    crate::log!("[AMDGPU-FW]   SDMA1: {:?}", state.sdma1);
    crate::log!("[AMDGPU-FW] ───────────────────────────────────────────────");
    
    if gga == 0 {
        crate::log!("[AMDGPU-FW] No firmware loaded — GPU compute will use CPU fallback");
        crate::log!("[AMDGPU-FW] To enable GPU compute:");
        crate::log!("[AMDGPU-FW]   1. Copy firmware files to {}", AEF_);
        crate::log!("[AMDGPU-FW]   2. Run 'gpufw load' to reload firmware");
        crate::log!("[AMDGPU-FW]   3. Or add firmware as Limine boot modules");
    } else {
        AUR_.store(true, Ordering::SeqCst);
        crate::log!("[AMDGPU-FW] Firmware loading complete — engines should be active");
    }
    
    
    let mga = unsafe { kj(mmio_base, regs::LB_) };
    crate::log!("[AMDGPU-FW] Post-load GRBM_STATUS: {:#010X}", mga);
    
    drop(state);
}






pub fn msz() -> bool {
    AUR_.load(Ordering::Relaxed)
}


pub fn summary() -> String {
    let state = AEG_.lock();
    let bhq = [state.rlc, state.pfp, state.me, state.ce,
                  state.mec1, state.mec2, state.sdma0, state.sdma1]
        .iter().filter(|&&j| j == FwStatus::Loaded || j == FwStatus::Running).count();
    format!("GPU Firmware: {}/8 loaded (RLC:{:?} MEC:{:?} SDMA:{:?})",
        bhq, state.rlc, state.mec1, state.sdma0)
}


pub fn reload(mmio_base: u64) {
    crate::log!("[AMDGPU-FW] Reloading firmware...");
    init(mmio_base);
}


pub fn owz() -> Vec<String> {
    let state = AEG_.lock();
    let mut lines = Vec::new();
    lines.push(format!("RLC  (Run List Controller):  {:?}", state.rlc));
    lines.push(format!("PFP  (Pre-Fetch Parser):     {:?}", state.pfp));
    lines.push(format!("ME   (Micro Engine):         {:?}", state.me));
    lines.push(format!("CE   (Constant Engine):      {:?}", state.ce));
    lines.push(format!("MEC1 (Compute Engine 1):     {:?}", state.mec1));
    lines.push(format!("MEC2 (Compute Engine 2):     {:?}", state.mec2));
    lines.push(format!("SDMA0 (DMA Engine 0):        {:?}", state.sdma0));
    lines.push(format!("SDMA1 (DMA Engine 1):        {:?}", state.sdma1));
    lines
}
