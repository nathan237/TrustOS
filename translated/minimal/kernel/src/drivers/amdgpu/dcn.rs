


























use alloc::string::String;
use alloc::format;
use alloc::vec::Vec;
use core::sync::atomic::{AtomicBool, Ordering};
use spin::Mutex;

use super::{wr, sk, onq, Sr};
use super::regs::dcn;






#[derive(Debug, Clone, Copy)]
pub struct Gh {
    
    pub buq: u32,
    
    pub eci: u32,
    
    pub erq: u32,
    
    pub fjz: u32,
    
    pub bxq: u32,
    
    pub ekk: u32,
    
    pub faw: u32,
    
    pub fxz: u32,
    
    pub duj: u32,
    
    pub ehi: u32,
    
    pub giq: bool,
    
    pub gvx: bool,
}

impl Gh {
    
    pub fn gir(&self) -> u32 {
        self.buq + self.eci + self.erq + self.fjz
    }

    
    pub fn gvy(&self) -> u32 {
        self.bxq + self.ekk + self.faw + self.fxz
    }

    
    pub fn hmh(&self) -> u32 {
        self.buq + self.eci
    }

    
    pub fn ixl(&self) -> u32 {
        self.buq + self.eci + self.erq
    }

    
    pub fn ige(&self) -> u32 {
        self.bxq + self.ekk
    }

    
    pub fn jve(&self) -> u32 {
        self.bxq + self.ekk + self.faw
    }

    
    pub fn lmk(&self) -> String {
        format!("{}x{}@{}Hz pclk={}kHz htotal={} vtotal={}",
            self.buq, self.bxq, self.ehi,
            self.duj, self.gir(), self.gvy())
    }
}


pub const CGY_: Gh = Gh {
    buq: 640, eci: 16, erq: 96, fjz: 48,
    bxq: 480, ekk: 10, faw: 2, fxz: 33,
    duj: 25175, ehi: 60,
    giq: false, gvx: false,
};

pub const CGU_: Gh = Gh {
    buq: 1280, eci: 110, erq: 40, fjz: 220,
    bxq: 720, ekk: 5, faw: 5, fxz: 20,
    duj: 74250, ehi: 60,
    giq: true, gvx: true,
};

pub const CGV_: Gh = Gh {
    buq: 1920, eci: 88, erq: 44, fjz: 148,
    bxq: 1080, ekk: 4, faw: 5, fxz: 36,
    duj: 148500, ehi: 60,
    giq: true, gvx: true,
};

pub const CGW_: Gh = Gh {
    buq: 2560, eci: 48, erq: 32, fjz: 80,
    bxq: 1440, ekk: 3, faw: 5, fxz: 33,
    duj: 241500, ehi: 60,
    giq: true, gvx: false,
};

pub const CGX_: Gh = Gh {
    buq: 3840, eci: 176, erq: 88, fjz: 296,
    bxq: 2160, ekk: 8, faw: 10, fxz: 72,
    duj: 533250, ehi: 60,
    giq: true, gvx: false,
};


pub fn wsi() -> &'static [Gh] {
    &[CGY_, CGU_, CGV_, CGW_, CGX_]
}






#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectorType {
    None,
    Ahm,
    Bil,
    Bef,
    Cpa,
    F,
}

impl ConnectorType {
    pub fn j(&self) -> &'static str {
        match self {
            ConnectorType::None => "None",
            ConnectorType::Ahm => "DisplayPort",
            ConnectorType::Bil => "HDMI",
            ConnectorType::Bef => "DVI",
            ConnectorType::Cpa => "VGA",
            ConnectorType::F => "Unknown",
        }
    }
}


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectorStatus {
    Lg,
    Dl,
    F,
}


#[derive(Debug, Clone)]
pub struct Abc {
    
    pub index: u8,
    
    pub ffo: ConnectorType,
    
    pub status: ConnectorStatus,
    
    pub nlj: u8,
    
    pub vhl: u8,
    
    pub tqh: u8,
    
    pub eog: Option<Gh>,
    
    pub sat: u8,
    
    pub ull: u8,
    
    pub ulj: u8,
}


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SurfaceFormat {
    
    Apd,
    
    Aor,
    
    Bbj,
    
    Bra,
}

impl SurfaceFormat {
    
    pub fn cwa(&self) -> u32 {
        match self {
            SurfaceFormat::Apd | SurfaceFormat::Aor | SurfaceFormat::Bbj => 4,
            SurfaceFormat::Bra => 2,
        }
    }

    
    pub fn rtx(&self) -> u32 {
        match self {
            SurfaceFormat::Apd => 0x0A, 
            SurfaceFormat::Aor => 0x08, 
            SurfaceFormat::Bbj => 0x0C, 
            SurfaceFormat::Bra => 0x04,   
        }
    }
}


#[derive(Debug, Clone)]
pub struct Amh {
    
    pub hja: u64,
    
    pub z: u32,
    
    pub ac: u32,
    
    pub jb: u32,
    
    pub format: SurfaceFormat,
}






pub struct Beg {
    
    pub jr: bool,
    
    pub cdt: Vec<Abc>,
    
    pub dyb: u8,
    
    pub eon: (u8, u8),  
    
    pub lle: u8,
    
    pub grl: [Option<Amh>; 6],
}

static JV_: Mutex<Beg> = Mutex::new(Beg {
    jr: false,
    cdt: Vec::new(),
    dyb: 0,
    eon: (0, 0),
    lle: 0,
    grl: [None, None, None, None, None, None],
});

static APU_: AtomicBool = AtomicBool::new(false);






unsafe fn dtz(mmio: u64, pipe: u8, ban: u32) -> u32 {
    let ar = dcn::BBX_ + (pipe as u32) * dcn::BCB_;
    wr(mmio, ar + ban)
}


unsafe fn efy(mmio: u64, pipe: u8, ban: u32, bn: u32) {
    let ar = dcn::BBX_ + (pipe as u32) * dcn::BCB_;
    sk(mmio, ar + ban, bn);
}


unsafe fn hne(mmio: u64, pipe: u8, ban: u32) -> u32 {
    let ar = dcn::AWC_ + (pipe as u32) * dcn::AWD_;
    wr(mmio, ar + ban)
}


unsafe fn hnf(mmio: u64, pipe: u8, ban: u32, bn: u32) {
    let ar = dcn::AWC_ + (pipe as u32) * dcn::AWD_;
    sk(mmio, ar + ban, bn);
}


#[allow(bgr)]
unsafe fn rxm(mmio: u64, hgc: u8, ban: u32) -> u32 {
    let ar = dcn::BRS_ + (hgc as u32) * dcn::BRU_;
    wr(mmio, ar + ban)
}


unsafe fn och(mmio: u64, gje: u8, ban: u32) -> u32 {
    let ar = dcn::CAK_ + (gje as u32) * dcn::CAN_;
    wr(mmio, ar + ban)
}







pub fn rwo(mmio: u64) -> Vec<Abc> {
    let mut cdt = Vec::new();
    
    crate::serial_println!("[DCN] Scanning display connectors (6 HPD pins)...");
    
    for a in 0..6u8 {
        unsafe {
            
            let lcq = och(mmio, a, dcn::CAM_);
            let lcp = och(mmio, a, dcn::CAL_);
            
            crate::serial_println!("[DCN]   HPD{}: INT_STATUS={:#010X} INT_CONTROL={:#010X}", 
                a, lcq, lcp);
            
            
            let dzr = (lcq & 1) != 0;
            
            
            let nlk = rxm(mmio, a, dcn::BRT_);
            
            crate::serial_println!("[DCN]   DIG{}: FE_CNTL={:#010X} connected={}", 
                a, nlk, dzr);
            
            
            
            let xno = (nlk >> 16) & 0xF;
            let ffo = match xno {
                0 => ConnectorType::Ahm,
                1 => ConnectorType::Bil,
                2 => ConnectorType::Bef,
                _ => {
                    
                    if lcp != 0 && lcp != 0xFFFFFFFF {
                        ConnectorType::Ahm 
                    } else {
                        ConnectorType::F
                    }
                }
            };
            
            let status = if dzr {
                ConnectorStatus::Dl
            } else if lcq == 0xFFFFFFFF {
                ConnectorStatus::F 
            } else {
                ConnectorStatus::Lg
            };
            
            cdt.push(Abc {
                index: a,
                ffo,
                status,
                nlj: a,
                vhl: a,
                tqh: a,
                eog: None,
                sat: 0,
                ull: 0,
                ulj: 0,
            });
        }
    }
    
    cdt
}



pub fn vro(mmio: u64, nfn: &mut Abc) {
    if nfn.ffo != ConnectorType::Ahm {
        return;
    }
    
    unsafe {
        
        let hgc = nfn.nlj;
        let mxa = dcn::BKT_ + (hgc as u32) * dcn::BKW_;
        
        
        let qln = wr(mmio, mxa + dcn::BKU_);
        crate::serial_println!("[DCN]   AUX{}: CONTROL={:#010X}", hgc, qln);
        
        
        
        
        
        
        
        
        
        let sas = wr(mmio, mxa + dcn::BKV_);
        crate::serial_println!("[DCN]   AUX{}: DPHY_TX_REF={:#010X}", hgc, sas);
    }
}






pub fn vrm(mmio: u64, pipe: u8) -> Option<Gh> {
    unsafe {
        
        let otd = dtz(mmio, pipe, dcn::KU_);
        crate::serial_println!("[DCN] OTG{}: CONTROL={:#010X}", pipe, otd);
        
        
        if otd & 1 == 0 {
            return None;
        }
        
        
        let gir = dtz(mmio, pipe, dcn::BCA_);
        let las = dtz(mmio, pipe, dcn::BBY_);
        let hmg = dtz(mmio, pipe, dcn::BBZ_);
        let gvy = dtz(mmio, pipe, dcn::BCE_);
        let mor = dtz(mmio, pipe, dcn::BCC_);
        let igd = dtz(mmio, pipe, dcn::BCD_);
        
        crate::serial_println!("[DCN] OTG{}: H_TOTAL={:#010X} H_BLANK={:#010X} H_SYNC={:#010X}", 
            pipe, gir, las, hmg);
        crate::serial_println!("[DCN] OTG{}: V_TOTAL={:#010X} V_BLANK={:#010X} V_SYNC={:#010X}", 
            pipe, gvy, mor, igd);
        
        
        let esc = gir & 0x7FFF;
        let fbd = gvy & 0x7FFF;
        let yvt = (las >> 16) & 0x7FFF;
        let oaa = las & 0x7FFF;
        let zvb = (mor >> 16) & 0x7FFF;
        let pxw = mor & 0x7FFF;
        let hmh = (hmg >> 16) & 0x7FFF;
        let ixl = hmg & 0x7FFF;
        let ige = (igd >> 16) & 0x7FFF;
        let jve = igd & 0x7FFF;
        
        
        if esc == 0 || fbd == 0 || esc > 8192 || fbd > 8192 {
            return None;
        }
        
        let buq = if oaa <= esc { oaa } else { esc };
        let bxq = if pxw <= fbd { pxw } else { fbd };
        
        if buq == 0 || bxq == 0 {
            return None;
        }
        
        let tip = hmh.ao(buq);
        let tit = ixl.ao(hmh);
        let tin = esc.ao(ixl);
        
        let xqa = ige.ao(bxq);
        let xqe = jve.ao(ige);
        let xpz = fbd.ao(jve);
        
        
        let kic = dtz(mmio, pipe, dcn::CIJ_);
        let duj = if kic != 0 && kic != 0xFFFFFFFF {
            
            (kic & 0xFFFF) * 10 
        } else {
            
            ((esc as u64) * (fbd as u64) * 60 / 1000) as u32
        };
        
        let gqr = if esc > 0 && fbd > 0 && duj > 0 {
            (duj as u64 * 1000) / (esc as u64 * fbd as u64)
        } else {
            60 
        };
        
        Some(Gh {
            buq, eci: tip, erq: tit, fjz: tin,
            bxq, ekk: xqa, faw: xqe, fxz: xpz,
            duj,
            ehi: gqr as u32,
            giq: true,
            gvx: true,
        })
    }
}


pub fn zgp(mmio: u64, pipe: u8, ev: &Gh) {
    crate::serial_println!("[DCN] Programming OTG{} for {}", pipe, ev.lmk());
    
    unsafe {
        
        efy(mmio, pipe, dcn::KU_, 0);
        
        
        efy(mmio, pipe, dcn::BCA_, ev.gir() - 1);
        
        let tim = ((ev.hmh()) << 16) | ev.buq;
        efy(mmio, pipe, dcn::BBY_, tim);
        
        let hmg = ((ev.hmh()) << 16) | ev.ixl();
        efy(mmio, pipe, dcn::BBZ_, hmg);
        
        
        efy(mmio, pipe, dcn::BCE_, ev.gvy() - 1);
        
        let xpy = ((ev.ige()) << 16) | ev.bxq;
        efy(mmio, pipe, dcn::BCC_, xpy);
        
        let igd = ((ev.ige()) << 16) | ev.jve();
        efy(mmio, pipe, dcn::BCD_, igd);
        
        crate::serial_println!("[DCN] OTG{} timing programmed: {}x{} htotal={} vtotal={}",
            pipe, ev.buq, ev.bxq, ev.gir(), ev.gvy());
    }
}


pub fn ypi(mmio: u64, pipe: u8) {
    unsafe {
        
        let mut bqb = dtz(mmio, pipe, dcn::KU_);
        bqb |= 1; 
        efy(mmio, pipe, dcn::KU_, bqb);
        crate::serial_println!("[DCN] OTG{} enabled", pipe);
    }
}


pub fn ymg(mmio: u64, pipe: u8) {
    unsafe {
        let mut bqb = dtz(mmio, pipe, dcn::KU_);
        bqb &= !1; 
        efy(mmio, pipe, dcn::KU_, bqb);
        crate::serial_println!("[DCN] OTG{} disabled", pipe);
    }
}






pub fn yjv(mmio: u64, pipe: u8, surface: &Amh) {
    crate::serial_println!("[DCN] Configuring HUBP{} for {}x{} @ {:#X}", 
        pipe, surface.z, surface.ac, surface.hja);
    
    unsafe {
        
        let mty = (surface.hja >> 32) as u32;
        let mtz = (surface.hja & 0xFFFFFFFF) as u32;
        
        hnf(mmio, pipe, dcn::AWE_, mty);
        hnf(mmio, pipe, dcn::AWF_, mtz);
        
        
        hnf(mmio, pipe, dcn::AWH_, surface.jb / surface.format.cwa());
        
        
        let wpd = (surface.ac << 16) | surface.z;
        hnf(mmio, pipe, dcn::AWI_, wpd);
        
        
        hnf(mmio, pipe, dcn::AWG_, surface.format.rtx());
        
        crate::serial_println!("[DCN] HUBP{} configured: addr={:#010X}:{:#010X} pitch={} fmt={:?}",
            pipe, mty, mtz, surface.jb, surface.format);
    }
}







pub fn init(hv: u64) {
    crate::log!("[DCN] ═══════════════════════════════════════════════════════");
    crate::log!("[DCN] Display Core Next 2.0 — Phase 2: Display Configuration");
    crate::log!("[DCN] ═══════════════════════════════════════════════════════");
    
    
    let koi = unsafe { wr(hv, dcn::BQZ_) };
    crate::serial_println!("[DCN] DCN_VERSION raw: {:#010X}", koi);
    
    let kog = (koi >> 8) & 0xFF;
    let koh = koi & 0xFF;
    crate::log!("[DCN] DCN version: {}.{}", kog, koh);
    
    
    let rzm = unsafe { wr(hv, dcn::BRY_) };
    crate::serial_println!("[DCN] DMCUB_STATUS: {:#010X}", rzm);
    
    
    crate::log!("[DCN] Detecting display connectors...");
    let mut cdt = rwo(hv);
    
    let mut kkp = 0u8;
    for ly in &cdt {
        let ejb = match ly.status {
            ConnectorStatus::Dl => {
                kkp += 1;
                "CONNECTED"
            },
            ConnectorStatus::Lg => "disconnected",
            ConnectorStatus::F => "unknown",
        };
        crate::log!("[DCN]   Connector {}: {} — {}", 
            ly.index, ly.ffo.j(), ejb);
    }
    crate::log!("[DCN] Found {} connected display(s)", kkp);
    
    
    for ly in &mut cdt {
        if ly.status == ConnectorStatus::Dl {
            vro(hv, ly);
        }
    }
    
    
    crate::log!("[DCN] Reading active display modes...");
    let mut dyb = 0u8;
    
    for pipe in 0..6u8 {
        if let Some(ev) = vrm(hv, pipe) {
            crate::log!("[DCN]   OTG{}: {} (active)", pipe, ev.lmk());
            
            if (pipe as usize) < cdt.len() {
                cdt[pipe as usize].eog = Some(ev);
            }
            dyb += 1;
        } else {
            crate::serial_println!("[DCN]   OTG{}: inactive", pipe);
        }
    }
    
    
    crate::log!("[DCN] Reading HUBP surface configurations...");
    let mut grl: [Option<Amh>; 6] = [None, None, None, None, None, None];
    
    for pipe in 0..6u8 {
        unsafe {
            let wvz = hne(hv, pipe, dcn::AWE_);
            let wwa = hne(hv, pipe, dcn::AWF_);
            let ppy = hne(hv, pipe, dcn::AWG_);
            let ppz = hne(hv, pipe, dcn::AWH_);
            let pqa = hne(hv, pipe, dcn::AWI_);
            
            let ag = ((wvz as u64) << 32) | (wwa as u64);
            
            if ag != 0 && ag != 0xFFFFFFFFFFFFFFFF && ppy != 0xFFFFFFFF {
                let z = pqa & 0xFFFF;
                let ac = (pqa >> 16) & 0xFFFF;
                
                crate::serial_println!("[DCN]   HUBP{}: addr={:#014X} size={}x{} pitch={} config={:#010X}", 
                    pipe, ag, z, ac, ppz, ppy);
                
                if z > 0 && ac > 0 && z < 16384 && ac < 16384 {
                    grl[pipe as usize] = Some(Amh {
                        hja: ag,
                        z,
                        ac,
                        jb: ppz * 4, 
                        format: SurfaceFormat::Aor,
                    });
                    crate::log!("[DCN]   HUBP{}: {}x{} surface at {:#014X}", pipe, z, ac, ag);
                }
            }
        }
    }
    
    
    crate::log!("[DCN] ───────────────────────────────────────────────────────");
    crate::log!("[DCN] DCN {}.{} — {} connector(s), {} active display(s)",
        kog, koh, kkp, dyb);
    for ly in &cdt {
        if ly.status == ConnectorStatus::Dl {
            if let Some(ref ev) = ly.eog {
                crate::log!("[DCN]   Output {}: {} {}x{}@{}Hz", 
                    ly.index, ly.ffo.j(),
                    ev.buq, ev.bxq, ev.ehi);
            } else {
                crate::log!("[DCN]   Output {}: {} (connected, no active mode)", 
                    ly.index, ly.ffo.j());
            }
        }
    }
    crate::log!("[DCN] ───────────────────────────────────────────────────────");
    crate::log!("[DCN] Phase 2 complete — Display engine probed");
    
    
    let mut g = JV_.lock();
    g.jr = true;
    g.cdt = cdt;
    g.dyb = dyb;
    g.eon = (kog as u8, koh as u8);
    g.lle = 6;
    g.grl = grl;
    APU_.store(true, Ordering::SeqCst);
}






pub fn uc() -> bool {
    APU_.load(Ordering::Relaxed)
}


pub fn yst() -> Vec<Abc> {
    JV_.lock().cdt.clone()
}


pub fn yeh() -> u8 {
    JV_.lock().dyb
}


pub fn eon() -> (u8, u8) {
    JV_.lock().eon
}


pub fn awz() -> String {
    let g = JV_.lock();
    if g.jr {
        let dzr = g.cdt.iter()
            .hi(|r| r.status == ConnectorStatus::Dl)
            .az();
        format!("DCN {}.{} — {} display(s), {} connected", 
            g.eon.0, g.eon.1,
            g.dyb, dzr)
    } else {
        String::from("DCN not initialized")
    }
}


pub fn zl() -> Vec<String> {
    let mut ak = Vec::new();
    let g = JV_.lock();
    
    if g.jr {
        ak.push(format!("DCN {}.{} Display Engine", g.eon.0, g.eon.1));
        ak.push(format!("  Pipes: {} max, {} active", g.lle, g.dyb));
        ak.push(String::new());
        
        for ly in &g.cdt {
            let status = match ly.status {
                ConnectorStatus::Dl => "CONNECTED",
                ConnectorStatus::Lg => "disconnected",
                ConnectorStatus::F => "unknown",
            };
            
            let mut line = format!("  Connector {}: {} [{}]", 
                ly.index, ly.ffo.j(), status);
            
            if let Some(ref ev) = ly.eog {
                line.t(&format!(" — {}x{}@{}Hz", ev.buq, ev.bxq, ev.ehi));
            }
            
            ak.push(line);
        }
        
        
        ak.push(String::new());
        ak.push(String::from("  Active Surfaces:"));
        for (a, wea) in g.grl.iter().cf() {
            if let Some(ref e) = wea {
                ak.push(format!("    HUBP{}: {}x{} {:?} @ {:#X}", 
                    a, e.z, e.ac, e.format, e.hja));
            }
        }
    } else {
        ak.push(String::from("DCN display engine not initialized"));
    }
    
    ak
}
