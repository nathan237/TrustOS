
















































use alloc::string::String;
use alloc::format;
use alloc::vec::Vec;
use core::sync::atomic::{AtomicBool, Ordering};
use spin::Mutex;

use super::{wr, sk, onq, uox};
use super::regs;
use crate::memory;






const ACP_: &str = "/lib/firmware/amdgpu";


const BWB_: &str = "navi10_pfp.bin";
const BVY_: &str = "navi10_me.bin";
const BVW_: &str = "navi10_ce.bin";
const BVZ_: &str = "navi10_mec.bin";
const BWA_: &str = "navi10_mec2.bin";
const ASO_: &str = "navi10_rlc.bin";
const BWC_: &str = "navi10_sdma.bin";
const BWD_: &str = "navi10_sdma1.bin";


const DCN_: u32 = 0x4D_44_41; 








const DMM_: usize = 256; 


const BVX_: u64 = 5_000_000;







const ECQ_: u32 = 0x4E08;

const COT_: u32 = 0x4E20;

const BEP_: u32 = 0x4E24;

const ECO_: u32 = 0x4E28;

const ECP_: u32 = 0x4E2C;

const COW_: u32 = 0x4E0C;

const AHH_: u32 = 0x4E00;

const BEQ_: u32 = 0x4E04;

const COU_: u32 = 0x4E30;



const BPJ_: u32 = 0x8A14;

const BPK_: u32 = 0x8A18;

const BPH_: u32 = 0x8A1C;

const BPI_: u32 = 0x8A20;

const BOY_: u32 = 0x8A24;

const BOZ_: u32 = 0x8A28;

const MN_: u32 = 0x86D8;


const APG_: u32 = 1 << 28;
const APH_: u32 = 1 << 26;
const APB_: u32 = 1 << 24;



const BPD_: u32 = 0x8A30;

const BPE_: u32 = 0x8A34;

const BPF_: u32 = 0x8A38;

const BPG_: u32 = 0x8A3C;



const CQZ_: u32 = 0x4D88;

const CRA_: u32 = 0x4D8C;

const CRD_: u32 = 0x4E88;

const CRE_: u32 = 0x4E8C;



const BPL_: u32 = 0x8044;

const DHI_: u32 = 0x8054;






#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FwStatus {
    
    N,
    
    Ls,
    
    Ai,
    
    Kk,
}


pub struct Bhg {
    pub eht: FwStatus,
    pub ego: FwStatus,
    pub efj: FwStatus,
    pub cdr: FwStatus,
    pub dtc: FwStatus,
    pub euu: FwStatus,
    pub eyg: FwStatus,
    pub fts: FwStatus,
    pub hv: u64,
}

static ACQ_: Mutex<Bhg> = Mutex::new(Bhg {
    eht: FwStatus::N,
    ego: FwStatus::N,
    efj: FwStatus::N,
    cdr: FwStatus::N,
    dtc: FwStatus::N,
    euu: FwStatus::N,
    eyg: FwStatus::N,
    fts: FwStatus::N,
    hv: 0,
});

static ASN_: AtomicBool = AtomicBool::new(false);






fn exh(j: &str) -> Option<Vec<u8>> {
    let path = format!("{}/{}", ACP_, j);
    crate::ramfs::fh(|fs| {
        fs.mq(&path).bq().map(|f| f.ip())
    })
}




fn ewa(f: &[u8]) -> &[u8] {
    if f.len() < 16 {
        return f;
    }
    
    
    
    
    let sho = u32::dj([f[0], f[1], f[2], f[3]]);
    let epl = u32::dj([f[4], f[5], f[6], f[7]]);
    
    
    
    
    
    
    
    
    
    
    
    
    
    
    
    
    if epl >= 4 && epl <= 1024 {
        let giy = (epl as usize) * 4;
        if giy < f.len() {
            
            if f.len() >= 24 {
                let juo = u32::dj([
                    f[20], f[21], f[22], f[23]
                ]) as usize;
                if juo > 0 && juo < f.len() {
                    crate::serial_println!("[AMDGPU-FW] Header: size={}dw, ucode_offset={:#X}", 
                        epl, juo);
                    return &f[juo..];
                }
            }
            crate::serial_println!("[AMDGPU-FW] Header: {}dw ({}B), ucode starts at {:#X}", 
                epl, giy, giy);
            return &f[giy..];
        }
    }
    
    
    crate::serial_println!("[AMDGPU-FW] No header detected, using raw blob");
    f
}









fn uhf(mmio: u64, cva: &[u8]) -> Result<(), &'static str> {
    let azr = cva.len() / 4;
    if azr == 0 { return Err("RLC firmware is empty"); }
    
    crate::log!("[AMDGPU-FW] Loading RLC: {} bytes ({} DWORDs)", cva.len(), azr);
    
    unsafe {
        
        let pdp = wr(mmio, AHH_);
        sk(mmio, AHH_, pdp & !(1u32)); 
        
        
        for _ in 0..10000 {
            let hm = wr(mmio, BEQ_);
            if hm & 1 == 0 { break; } 
            core::hint::hc();
        }
        
        
        sk(mmio, COU_, azr as u32);
        
        
        sk(mmio, COT_, 0);
        
        
        for a in 0..azr {
            let l = a * 4;
            if l + 4 > cva.len() { break; }
            let aix = u32::dj([
                cva[l], cva[l+1], cva[l+2], cva[l+3]
            ]);
            sk(mmio, BEP_, aix);
        }
        
        
        sk(mmio, AHH_, pdp | 1); 
        
        
        for _ in 0..BVX_ {
            let hm = wr(mmio, BEQ_);
            if hm & 1 != 0 {
                crate::log!("[AMDGPU-FW] RLC running (stat={:#X})", hm);
                return Ok(());
            }
            core::hint::hc();
        }
    }
    
    crate::log!("[AMDGPU-FW] RLC loaded ({} DWORDs) — verifying...", azr);
    Ok(())
}







fn ugo(mmio: u64, ego: Option<&[u8]>, efj: Option<&[u8]>, cdr: Option<&[u8]>) -> Result<(), &'static str> {
    unsafe {
        
        let omo = wr(mmio, MN_);
        sk(mmio, MN_, omo | APG_ | APH_ | APB_);
        
        
        for _ in 0..10000 { core::hint::hc(); }
        
        
        if let Some(ua) = ego {
            let azr = ua.len() / 4;
            crate::log!("[AMDGPU-FW] Loading PFP: {} DWORDs", azr);
            sk(mmio, BPJ_, 0);
            for a in 0..azr {
                let dz = a * 4;
                if dz + 4 > ua.len() { break; }
                let aix = u32::dj([ua[dz], ua[dz+1], ua[dz+2], ua[dz+3]]);
                sk(mmio, BPK_, aix);
            }
        }
        
        
        if let Some(ua) = efj {
            let azr = ua.len() / 4;
            crate::log!("[AMDGPU-FW] Loading ME: {} DWORDs", azr);
            sk(mmio, BPH_, 0);
            for a in 0..azr {
                let dz = a * 4;
                if dz + 4 > ua.len() { break; }
                let aix = u32::dj([ua[dz], ua[dz+1], ua[dz+2], ua[dz+3]]);
                sk(mmio, BPI_, aix);
            }
        }
        
        
        if let Some(ua) = cdr {
            let azr = ua.len() / 4;
            crate::log!("[AMDGPU-FW] Loading CE: {} DWORDs", azr);
            sk(mmio, BOY_, 0);
            for a in 0..azr {
                let dz = a * 4;
                if dz + 4 > ua.len() { break; }
                let aix = u32::dj([ua[dz], ua[dz+1], ua[dz+2], ua[dz+3]]);
                sk(mmio, BOZ_, aix);
            }
        }
        
        
        sk(mmio, MN_, omo & !(APG_ | APH_ | APB_));
        
        
        for _ in 0..100000 {
            let fjx = wr(mmio, regs::KI_);
            if fjx & regs::ADC_ == 0 {
                crate::log!("[AMDGPU-FW] CP GFX engines started");
                return Ok(());
            }
            core::hint::hc();
        }
    }
    
    crate::log!("[AMDGPU-FW] CP GFX loaded — engine busy (may need time)");
    Ok(())
}





fn uhe(mmio: u64, dtc: Option<&[u8]>, euu: Option<&[u8]>) -> Result<(), &'static str> {
    unsafe {
        
        let omr = wr(mmio, regs::AAU_);
        sk(mmio, regs::AAU_, omr | (1 << 28)); 
        
        for _ in 0..10000 { core::hint::hc(); }
        
        
        if let Some(ua) = dtc {
            let azr = ua.len() / 4;
            crate::log!("[AMDGPU-FW] Loading MEC1: {} DWORDs", azr);
            sk(mmio, BPD_, 0);
            for a in 0..azr {
                let dz = a * 4;
                if dz + 4 > ua.len() { break; }
                let aix = u32::dj([ua[dz], ua[dz+1], ua[dz+2], ua[dz+3]]);
                sk(mmio, BPE_, aix);
            }
        }
        
        
        if let Some(ua) = euu {
            let azr = ua.len() / 4;
            crate::log!("[AMDGPU-FW] Loading MEC2: {} DWORDs", azr);
            sk(mmio, BPF_, 0);
            for a in 0..azr {
                let dz = a * 4;
                if dz + 4 > ua.len() { break; }
                let aix = u32::dj([ua[dz], ua[dz+1], ua[dz+2], ua[dz+3]]);
                sk(mmio, BPG_, aix);
            }
        }
        
        
        sk(mmio, regs::AAU_, omr & !(1u32 << 28));
    }
    
    crate::log!("[AMDGPU-FW] MEC firmware loaded");
    Ok(())
}


fn ojx(mmio: u64, engine: usize, ua: &[u8]) -> Result<(), &'static str> {
    let azr = ua.len() / 4;
    if azr == 0 { return Err("SDMA firmware is empty"); }
    
    let (qfr, rtm, itp) = match engine {
        0 => (CQZ_, CRA_, regs::BFD_),
        1 => (CRD_, CRE_, regs::BFF_),
        _ => return Err("Invalid SDMA engine index"),
    };
    
    crate::log!("[AMDGPU-FW] Loading SDMA{}: {} DWORDs", engine, azr);
    
    unsafe {
        
        sk(mmio, itp, wr(mmio, itp) | 1); 
        for _ in 0..10000 { core::hint::hc(); }
        
        
        sk(mmio, qfr, 0);
        
        
        for a in 0..azr {
            let dz = a * 4;
            if dz + 4 > ua.len() { break; }
            let aix = u32::dj([ua[dz], ua[dz+1], ua[dz+2], ua[dz+3]]);
            sk(mmio, rtm, aix);
        }
        
        
        sk(mmio, itp, wr(mmio, itp) & !1u32);
    }
    
    crate::log!("[AMDGPU-FW] SDMA{} firmware loaded", engine);
    Ok(())
}









pub fn init(hv: u64) {
    crate::log!("[AMDGPU-FW] ═══════════════════════════════════════════════");
    crate::log!("[AMDGPU-FW] AMD GPU Firmware Loader — Navi 10");
    crate::log!("[AMDGPU-FW] ═══════════════════════════════════════════════");
    
    if hv == 0 {
        crate::log!("[AMDGPU-FW] No MMIO base — skipping firmware init");
        return;
    }
    
    let mut g = ACQ_.lock();
    g.hv = hv;
    
    
    let _ = crate::ramfs::fh(|fs| {
        let _ = fs.ut("/lib");
        let _ = fs.ut("/lib/firmware");
        let _ = fs.ut("/lib/firmware/amdgpu");
    });
    
    
    let mgf = unsafe { wr(hv, regs::BAZ_) };
    crate::log!("[AMDGPU-FW] SMU firmware version: {:#010X}", mgf);
    if mgf != 0 && mgf != 0xFFFFFFFF {
        crate::log!("[AMDGPU-FW] SMU is active — VBIOS has initialized power management");
    } else {
        crate::log!("[AMDGPU-FW] SMU not active — cold boot, firmware required");
    }
    
    
    let fjx = unsafe { wr(hv, regs::KI_) };
    let kky = fjx & regs::ADC_ != 0;
    let laq = fjx & regs::ATR_ != 0;
    crate::log!("[AMDGPU-FW] Pre-load: GRBM={:#010X} CP_BUSY={} GUI_ACTIVE={}", 
        fjx, kky, laq);
    
    
    
    
    if let Some(js) = exh(ASO_) {
        let cva = ewa(&js);
        match uhf(hv, cva) {
            Ok(()) => g.eht = FwStatus::Ls,
            Err(aa) => {
                crate::log!("[AMDGPU-FW] RLC load failed: {}", aa);
                g.eht = FwStatus::Kk;
            }
        }
    } else {
        crate::log!("[AMDGPU-FW] {} not found in {}", ASO_, ACP_);
        g.eht = FwStatus::N;
    }
    
    
    let vgx = exh(BWB_);
    let umr = exh(BVY_);
    let qxe = exh(BVW_);
    
    let ltr = vgx.ahz().map(ewa);
    let llj = umr.ahz().map(ewa);
    let kgw = qxe.ahz().map(ewa);
    
    if ltr.is_some() || llj.is_some() || kgw.is_some() {
        match ugo(hv, ltr, llj, kgw) {
            Ok(()) => {
                if ltr.is_some() { g.ego = FwStatus::Ls; }
                if llj.is_some() { g.efj = FwStatus::Ls; }
                if kgw.is_some() { g.cdr = FwStatus::Ls; }
            }
            Err(aa) => {
                crate::log!("[AMDGPU-FW] CP GFX load failed: {}", aa);
                g.ego = FwStatus::Kk;
                g.efj = FwStatus::Kk;
                g.cdr = FwStatus::Kk;
            }
        }
    } else {
        crate::log!("[AMDGPU-FW] No GFX CP firmware found (PFP/ME/CE)");
    }
    
    
    let ums = exh(BVZ_);
    let umt = exh(BWA_);
    
    let lll = ums.ahz().map(ewa);
    let llm = umt.ahz().map(ewa);
    
    if lll.is_some() || llm.is_some() {
        match uhe(hv, lll, llm) {
            Ok(()) => {
                if lll.is_some() { g.dtc = FwStatus::Ls; }
                if llm.is_some() { g.euu = FwStatus::Ls; }
            }
            Err(aa) => {
                crate::log!("[AMDGPU-FW] MEC load failed: {}", aa);
                g.dtc = FwStatus::Kk;
            }
        }
    } else {
        crate::log!("[AMDGPU-FW] No MEC firmware found");
    }
    
    
    if let Some(js) = exh(BWC_) {
        let cva = ewa(&js);
        match ojx(hv, 0, cva) {
            Ok(()) => g.eyg = FwStatus::Ls,
            Err(aa) => {
                crate::log!("[AMDGPU-FW] SDMA0 load failed: {}", aa);
                g.eyg = FwStatus::Kk;
            }
        }
    }
    
    if let Some(js) = exh(BWD_) {
        let cva = ewa(&js);
        match ojx(hv, 1, cva) {
            Ok(()) => g.fts = FwStatus::Ls,
            Err(aa) => {
                crate::log!("[AMDGPU-FW] SDMA1 load failed: {}", aa);
                g.fts = FwStatus::Kk;
            }
        }
    }
    
    
    let ljl = [g.eht, g.ego, g.efj, g.cdr, 
                        g.dtc, g.euu, g.eyg, g.fts]
        .iter().hi(|&&e| e == FwStatus::Ls).az();
    
    crate::log!("[AMDGPU-FW] ───────────────────────────────────────────────");
    crate::log!("[AMDGPU-FW] Firmware status ({}/8 loaded):", ljl);
    crate::log!("[AMDGPU-FW]   RLC:   {:?}", g.eht);
    crate::log!("[AMDGPU-FW]   PFP:   {:?}", g.ego);
    crate::log!("[AMDGPU-FW]   ME:    {:?}", g.efj);
    crate::log!("[AMDGPU-FW]   CE:    {:?}", g.cdr);
    crate::log!("[AMDGPU-FW]   MEC1:  {:?}", g.dtc);
    crate::log!("[AMDGPU-FW]   MEC2:  {:?}", g.euu);
    crate::log!("[AMDGPU-FW]   SDMA0: {:?}", g.eyg);
    crate::log!("[AMDGPU-FW]   SDMA1: {:?}", g.fts);
    crate::log!("[AMDGPU-FW] ───────────────────────────────────────────────");
    
    if ljl == 0 {
        crate::log!("[AMDGPU-FW] No firmware loaded — GPU compute will use CPU fallback");
        crate::log!("[AMDGPU-FW] To enable GPU compute:");
        crate::log!("[AMDGPU-FW]   1. Copy firmware files to {}", ACP_);
        crate::log!("[AMDGPU-FW]   2. Run 'gpufw load' to reload firmware");
        crate::log!("[AMDGPU-FW]   3. Or add firmware as Limine boot modules");
    } else {
        ASN_.store(true, Ordering::SeqCst);
        crate::log!("[AMDGPU-FW] Firmware loading complete — engines should be active");
    }
    
    
    let thl = unsafe { wr(hv, regs::KI_) };
    crate::log!("[AMDGPU-FW] Post-load GRBM_STATUS: {:#010X}", thl);
    
    drop(g);
}






pub fn tyc() -> bool {
    ASN_.load(Ordering::Relaxed)
}


pub fn awz() -> String {
    let g = ACQ_.lock();
    let diz = [g.eht, g.ego, g.efj, g.cdr,
                  g.dtc, g.euu, g.eyg, g.fts]
        .iter().hi(|&&e| e == FwStatus::Ls || e == FwStatus::Ai).az();
    format!("GPU Firmware: {}/8 loaded (RLC:{:?} MEC:{:?} SDMA:{:?})",
        diz, g.eht, g.dtc, g.eyg)
}


pub fn ahs(hv: u64) {
    crate::log!("[AMDGPU-FW] Reloading firmware...");
    init(hv);
}


pub fn wtw() -> Vec<String> {
    let g = ACQ_.lock();
    let mut ak = Vec::new();
    ak.push(format!("RLC  (Run List Controller):  {:?}", g.eht));
    ak.push(format!("PFP  (Pre-Fetch Parser):     {:?}", g.ego));
    ak.push(format!("ME   (Micro Engine):         {:?}", g.efj));
    ak.push(format!("CE   (Constant Engine):      {:?}", g.cdr));
    ak.push(format!("MEC1 (Compute Engine 1):     {:?}", g.dtc));
    ak.push(format!("MEC2 (Compute Engine 2):     {:?}", g.euu));
    ak.push(format!("SDMA0 (DMA Engine 0):        {:?}", g.eyg));
    ak.push(format!("SDMA1 (DMA Engine 1):        {:?}", g.fts));
    ak
}
