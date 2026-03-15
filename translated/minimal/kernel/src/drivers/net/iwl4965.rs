












use alloc::boxed::Box;
use alloc::string::String;
use alloc::vec::Vec;
use alloc::vec;
use alloc::format;
use core::ptr::{read_volatile, write_volatile};
use core::sync::atomic::{AtomicBool, Ordering};

use super::wifi::{Afr, Uz, WifiSecurity, WifiState};
use super::{Ha, NetStats};
use crate::drivers::{Gi, Co, DriverStatus, DriverCategory};
use crate::pci::S;





const AXB_: u16 = 0x8086;


const CCP_: &[u16] = &[
    0x4229, 
    0x4230, 
];


const CCQ_: &[u16] = &[
    0x4229, 0x4230,         
    0x4232, 0x4235, 0x4236, 
    0x4237, 0x4238, 0x4239, 
    0x008A, 0x008B,         
    0x0082, 0x0083, 0x0084, 
    0x0085, 0x0089,         
    0x0887, 0x0888,         
    0x0890, 0x0891,         
    0x0893, 0x0894,         
    0x088E, 0x088F,         
    0x24F3, 0x24F4,         
    0x2526,                 
    0x2723,                 
    0x2725,                 
    0x7A70,                 
];





const DHY_:   u32 = 0x000;
const BQC_: u32 = 0x004;
const MP_:            u32 = 0x008;
const APL_:       u32 = 0x00C;
const AAW_:  u32 = 0x010;
const BPX_:        u32 = 0x018;
const SI_:          u32 = 0x020;
const MO_:       u32 = 0x024;
const BQA_:         u32 = 0x028;
const APK_:     u32 = 0x02C;
const DHQ_:      u32 = 0x030;
const DIF_:  u32 = 0x054;
const DIG_:  u32 = 0x058;
const DHS_:        u32 = 0x03C;
const DHX_:       u32 = 0x048;
const DHW_:      u32 = 0x050;


const BPZ_: u32 = 1 << 0;
const DHU_:       u32 = 1 << 2;
const BPY_:  u32 = 1 << 3;
const DHT_:  u32 = 1 << 4;
const DHV_:         u32 = 1 << 10;


const BQE_:   u32 = 1 << 0;
const DIE_:    u32 = 1 << 1;
const BQF_:     u32 = 1 << 7;
const BQD_: u32 = 1 << 8;
const APM_:  u32 = 1 << 9;


const BPW_:  u32 = 1 << 0;
const BPV_:         u32 = 1 << 1;
const DHR_:        u32 = 0x0000FFFC;


const BQB_: u32 = 0x000FFF0;
const DHZ_: u32 = 0x0000000;
const DIC_: u32 = 0x0000020;
const DIA_: u32 = 0x0000050;
const DIB_: u32 = 0x0000040;
const DID_: u32 = 0x0000070;





const ABU_: u16 = 0x0015;
const BTF_:     u16 = 0x0045;
const DLA_: u16 = 0x0062; 
const DLB_: u16 = 0x0080; 





const DQS_: u16 = 0x0000;
const DQT_: u16 = 0x0080;
const DQU_: u16 = 0x0050;


const EKR_: u8 = 0;
const EKP_: u8 = 3;
const EKQ_: u8 = 48;        
const EKS_: u8 = 221;    





const DTM_: usize = 32;
const CQJ_: u64 = 500; 

pub struct Iwl4965 {
    
    lte: u8,
    cgm: u8,
    ltf: u8,
    mx: u16,

    
    hv: usize,
    bkm: usize,

    
    status: DriverStatus,
    biy: WifiState,
    izb: u32,
    bvh: [u8; 6],

    
    eia: Vec<Uz>,
    mcg: u64,
    grk: bool,

    
    cwo: Option<String>,
    kko: [u8; 6],
    fgf: u8,
    dlv: i8,

    
    cm: NetStats,

    
    jr: bool,
}

impl Iwl4965 {
    fn new() -> Self {
        Self {
            lte: 0,
            cgm: 0,
            ltf: 0,
            mx: 0,
            hv: 0,
            bkm: 0,
            status: DriverStatus::Aff,
            biy: WifiState::Aqx,
            izb: 0,
            bvh: [0; 6],
            eia: Vec::new(),
            mcg: 0,
            grk: false,
            cwo: None,
            kko: [0; 6],
            fgf: 0,
            dlv: 0,
            cm: NetStats::default(),
            jr: false,
        }
    }

    

    #[inline]
    fn bam(&self, l: u32) -> u32 {
        if self.hv == 0 { return 0; }
        unsafe {
            let ptr = (self.hv + l as usize) as *const u32;
            read_volatile(ptr)
        }
    }

    #[inline]
    fn afl(&self, l: u32, bn: u32) {
        if self.hv == 0 { return; }
        unsafe {
            let ptr = (self.hv + l as usize) as *mut u32;
            write_volatile(ptr, bn);
        }
    }

    

    
    fn ujp(&mut self, sq: &S) -> Result<(), &'static str> {
        let aew = sq.bar[0];
        if aew == 0 {
            return Err("BAR0 is zero");
        }

        
        let tye = (aew & 1) == 0;
        if !tye {
            return Err("BAR0 is I/O, need memory");
        }

        
        let edt = (aew >> 1) & 0x3 == 2;
        let sm = if edt {
            let qmt = sq.bar[1] as u64;
            ((qmt << 32) | (aew & 0xFFFFFFF0) as u64) as usize
        } else {
            (aew & 0xFFFFFFF0) as usize
        };

        if sm == 0 {
            return Err("BAR0 base address is zero");
        }

        
        self.hv = sm;
        self.bkm = 0x2000; 

        
        
        crate::serial_println!("[IWL4965] MMIO base: {:#X}, size: {:#X}", sm, self.bkm);

        Ok(())
    }

    
    fn tqp(&mut self) -> Result<(), &'static str> {
        
        self.afl(APL_, 0);
        self.afl(MP_, 0xFFFFFFFF);
        self.afl(AAW_, 0xFFFFFFFF);

        
        self.izb = self.bam(BQA_);
        let ocq = (self.izb & BQB_) >> 4;
        let tqq = match ocq {
            0x00 => "4965",
            0x02 => "5300",
            0x04 => "5150",
            0x05 => "5100",
            0x07 => "6000",
            _ => "unknown",
        };
        crate::serial_println!("[IWL4965] HW rev: {:#010X} (type: {} = {})", self.izb, ocq, tqq);

        
        self.afl(SI_, APM_);

        
        let mut ccm = 0u32;
        loop {
            let ap = self.bam(SI_);
            if ap & BQD_ != 0 {
                break;
            }
            ccm += 1;
            if ccm > 1000 {
                crate::serial_println!("[IWL4965] Warning: master stop timeout");
                break;
            }
            
            for _ in 0..100 { core::hint::hc(); }
        }

        
        self.afl(SI_, BQF_ | BQE_);
        
        for _ in 0..10000 { core::hint::hc(); }

        
        self.afl(MO_, 
            self.bam(MO_) | BPY_);

        
        ccm = 0;
        loop {
            let ap = self.bam(MO_);
            if ap & BPZ_ != 0 {
                break;
            }
            ccm += 1;
            if ccm > 5000 {
                crate::serial_println!("[IWL4965] Warning: MAC clock not ready");
                break;
            }
            for _ in 0..100 { core::hint::hc(); }
        }

        crate::serial_println!("[IWL4965] Hardware initialized, GP_CNTRL: {:#010X}", self.bam(MO_));

        
        self.vrq()?;

        crate::serial_println!("[IWL4965] MAC: {:02X}:{:02X}:{:02X}:{:02X}:{:02X}:{:02X}",
            self.bvh[0], self.bvh[1], self.bvh[2],
            self.bvh[3], self.bvh[4], self.bvh[5]);

        self.jr = true;
        Ok(())
    }

    
    fn isl(&self, ag: u16) -> u16 {
        
        let vua = ((ag as u32) << 2) | BPV_;
        self.afl(APK_, vua);

        
        for _ in 0..5000 {
            let ap = self.bam(APK_);
            if ap & BPW_ != 0 {
                return (ap >> 16) as u16;
            }
            for _ in 0..50 { core::hint::hc(); }
        }

        crate::serial_println!("[IWL4965] EEPROM read timeout at addr {:#06X}", ag);
        0
    }

    
    fn vrq(&mut self) -> Result<(), &'static str> {
        let cnv = self.isl(ABU_);
        let blt = self.isl(ABU_ + 1);
        let bfs = self.isl(ABU_ + 2);

        self.bvh[0] = (cnv & 0xFF) as u8;
        self.bvh[1] = (cnv >> 8) as u8;
        self.bvh[2] = (blt & 0xFF) as u8;
        self.bvh[3] = (blt >> 8) as u8;
        self.bvh[4] = (bfs & 0xFF) as u8;
        self.bvh[5] = (bfs >> 8) as u8;

        
        if self.bvh == [0; 6] || self.bvh == [0xFF; 6] {
            
            
            crate::serial_println!("[IWL4965] EEPROM MAC invalid, generating from PCI");
            
            self.bvh = [
                0x00, 0x13, 0xE8, 
                self.lte,
                self.cgm,
                self.ltf | 0x40,
            ];
        }

        Ok(())
    }

    

    
    fn wtb(&mut self) -> Result<(), &'static str> {
        if !self.jr {
            return Err("Hardware not initialized");
        }

        self.eia.clear();
        self.grk = true;
        self.mcg = crate::logger::lh();
        self.biy = WifiState::Uj;

        
        
        

        
        self.afl(BQC_, 0x40);

        crate::serial_println!("[IWL4965] Passive scan started on 2.4 GHz");

        Ok(())
    }

    
    fn owl(&mut self) {
        if !self.grk {
            return;
        }

        let qb = crate::logger::lh();
        let ez = qb.ao(self.mcg);

        
        let fls = self.bam(MP_);
        if fls != 0 && fls != 0xFFFFFFFF {
            
            self.afl(MP_, fls);
        }

        
        let kvp = self.bam(AAW_);
        if kvp != 0 && kvp != 0xFFFFFFFF {
            self.afl(AAW_, kvp);
            
        }

        
        if ez >= CQJ_ {
            self.grk = false;
            self.biy = if self.cwo.is_some() {
                WifiState::Dl
            } else {
                WifiState::Lg
            };
            crate::serial_println!("[IWL4965] Scan complete: {} networks", self.eia.len());

            
            
            if self.eia.is_empty() {
                self.rws();
            }
        }
    }

    
    
    
    fn rws(&mut self) {
        
        
        

        let gpio = self.bam(BPX_);
        let tgs = self.bam(MO_);

        crate::serial_println!("[IWL4965] GPIO: {:#010X}, GP_CNTRL: {:#010X}", gpio, tgs);

        
        let jqm = self.isl(BTF_);
        let tlx = (jqm & 0x01) != 0 || jqm == 0; 
        let tly = (jqm & 0x02) != 0;
        crate::serial_println!("[IWL4965] SKU: {:#06X}, 2.4GHz: {}, 5GHz: {}", jqm, tlx, tly);

        
        
        
        
    }

    

    fn rzt(&mut self, bfk: &str, aqe: &str) -> Result<(), &'static str> {
        if !self.jr {
            return Err("Hardware not initialized");
        }

        
        let network = self.eia.iter()
            .du(|bo| bo.bfk == bfk)
            .abn();

        match network {
            Some(net) => {
                crate::serial_println!("[IWL4965] Connecting to '{}' on ch{} ({:02X}:{:02X}:{:02X}:{:02X}:{:02X}:{:02X})",
                    bfk, net.channel,
                    net.fds[0], net.fds[1], net.fds[2],
                    net.fds[3], net.fds[4], net.fds[5]);

                self.biy = WifiState::Bcf;
                self.kko = net.fds;
                self.fgf = net.channel;
                self.dlv = net.dlv;

                
                
                
                
                
                
                
                

                self.cwo = Some(String::from(bfk));
                self.biy = WifiState::Dl;

                crate::serial_println!("[IWL4965] Connected to '{}' (signal: {} dBm)", bfk, net.dlv);
                Ok(())
            }
            None => {
                
                crate::serial_println!("[IWL4965] Network '{}' not in scan results, attempting blind connect", bfk);
                self.biy = WifiState::Aas;
                self.cwo = Some(String::from(bfk));
                
                Ok(())
            }
        }
    }
}





impl Gi for Iwl4965 {
    fn co(&self) -> &Co {
        &BZ_
    }

    fn probe(&mut self, sq: &S) -> Result<(), &'static str> {
        self.lte = sq.aq;
        self.cgm = sq.de;
        self.ltf = sq.gw;
        self.mx = sq.mx;
        self.status = DriverStatus::Py;

        
        self.ujp(sq)?;

        
        let cmd = crate::pci::aon(sq.aq, sq.de, sq.gw, 0x04);
        crate::pci::aso(sq.aq, sq.de, sq.gw, 0x04,
            cmd | 0x06); 

        Ok(())
    }

    fn ay(&mut self) -> Result<(), &'static str> {
        self.tqp()?;
        self.biy = WifiState::Lg;
        self.status = DriverStatus::Ai;
        crate::log!("[IWL4965] Intel WiFi Link {} ready", 
            if CCP_.contains(&self.mx) { "4965AGN" } else { "WiFi" });
        Ok(())
    }

    fn qg(&mut self) -> Result<(), &'static str> {
        
        self.afl(APL_, 0);
        
        self.afl(SI_, APM_);
        self.status = DriverStatus::Ky;
        self.biy = WifiState::Aqx;
        Ok(())
    }

    fn status(&self) -> DriverStatus {
        self.status
    }

    fn eck(&mut self) {
        let fls = self.bam(MP_);
        if fls == 0 || fls == 0xFFFFFFFF {
            return;
        }
        self.afl(MP_, fls);

        
        if self.grk {
            self.owl();
        }
    }
}

impl Ha for Iwl4965 {
    fn csg(&self) -> [u8; 6] {
        self.bvh
    }

    fn aik(&self) -> bool {
        self.biy == WifiState::Dl
    }

    fn gll(&self) -> u32 {
        if self.biy == WifiState::Dl { 54 } else { 0 } 
    }

    fn baq(&mut self, iia: &[u8]) -> Result<(), &'static str> {
        if self.biy != WifiState::Dl {
            return Err("Not connected");
        }
        
        
        self.cm.cuz += 1;
        self.cm.bpc += iia.len() as u64;
        Ok(())
    }

    fn chb(&mut self) -> Option<Vec<u8>> {
        
        None
    }

    fn poll(&mut self) {
        if self.grk {
            self.owl();
        }
    }

    fn cm(&self) -> NetStats {
        self.cm
    }
}

impl Afr for Iwl4965 {
    fn biy(&self) -> WifiState {
        self.biy
    }

    fn arx(&mut self) -> Result<(), &'static str> {
        self.wtb()
    }

    fn eia(&self) -> Vec<Uz> {
        self.eia.clone()
    }

    fn ipa(&mut self, bfk: &str, aqe: &str) -> Result<(), &'static str> {
        self.rzt(bfk, aqe)
    }

    fn irg(&mut self) -> Result<(), &'static str> {
        self.cwo = None;
        self.kko = [0; 6];
        self.fgf = 0;
        self.dlv = 0;
        self.biy = WifiState::Lg;
        crate::serial_println!("[IWL4965] Disconnected");
        Ok(())
    }

    fn cwo(&self) -> Option<String> {
        self.cwo.clone()
    }

    fn fgf(&self) -> Option<u8> {
        if self.fgf > 0 { Some(self.fgf) } else { None }
    }

    fn jqh(&self) -> Option<i8> {
        if self.biy == WifiState::Dl { Some(self.dlv) } else { None }
    }
}


unsafe impl Send for Iwl4965 {}
unsafe impl Sync for Iwl4965 {}





static BZ_: Co = Co {
    j: "Intel WiFi (iwl4965)",
    dk: "0.1.0",
    gzh: "TrustOS",
    gb: DriverCategory::As,
    fye: &[(AXB_, 0xFFFF)], 
};


pub fn probe(sq: &S) -> Option<Box<dyn Afr>> {
    
    if sq.ml != AXB_ {
        return None;
    }

    if !CCQ_.contains(&sq.mx) {
        return None;
    }

    crate::serial_println!("[IWL4965] Probing Intel WiFi {:04X}:{:04X}...", 
        sq.ml, sq.mx);

    let mut rj = Iwl4965::new();
    match rj.probe(sq) {
        Ok(()) => {
            match rj.ay() {
                Ok(()) => {
                    crate::log!("[IWL4965] Driver loaded for {:04X}:{:04X}", 
                        sq.ml, sq.mx);
                    Some(Box::new(rj))
                }
                Err(aa) => {
                    crate::serial_println!("[IWL4965] Start failed: {}", aa);
                    None
                }
            }
        }
        Err(aa) => {
            crate::serial_println!("[IWL4965] Probe failed: {}", aa);
            None
        }
    }
}
