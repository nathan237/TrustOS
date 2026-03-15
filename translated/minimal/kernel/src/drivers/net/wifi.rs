




use alloc::boxed::Box;
use alloc::string::String;
use alloc::vec::Vec;
use spin::Mutex;
use core::sync::atomic::{AtomicBool, Ordering};

use super::{Ha, NetStats};
use crate::drivers::{Gi, Co, DriverStatus, DriverCategory};
use crate::pci::S;






#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WifiSecurity {
    Ck,
    Cqb,
    Cqd,
    Cqe,
    Cqf,
    F,
}

impl WifiSecurity {
    pub fn as_str(&self) -> &'static str {
        match self {
            WifiSecurity::Ck => "Open",
            WifiSecurity::Cqb => "WEP",
            WifiSecurity::Cqd => "WPA",
            WifiSecurity::Cqe => "WPA2",
            WifiSecurity::Cqf => "WPA3",
            WifiSecurity::F => "???",
        }
    }
}


#[derive(Debug, Clone)]
pub struct Uz {
    
    pub bfk: String,
    
    pub fds: [u8; 6],
    
    pub channel: u8,
    
    pub dlv: i8,
    
    pub security: WifiSecurity,
    
    pub sxk: u16,
}

impl Uz {
    
    pub fn wod(&self) -> u8 {
        
        if self.dlv >= -30 { return 100; }
        if self.dlv <= -90 { return 0; }
        ((self.dlv as i16 + 90) * 100 / 60) as u8
    }

    
    pub fn wob(&self) -> u8 {
        match self.wod() {
            80..=100 => 4,
            60..=79 => 3,
            40..=59 => 2,
            20..=39 => 1,
            _ => 0,
        }
    }
}


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WifiState {
    
    Bni,
    
    Aqx,
    
    Lg,
    
    Uj,
    
    Aas,
    
    Bcf,
    
    Dl,
    
    Kk,
}


pub trait Afr: Ha {
    
    fn biy(&self) -> WifiState;

    
    fn arx(&mut self) -> Result<(), &'static str>;

    
    fn eia(&self) -> Vec<Uz>;

    
    fn ipa(&mut self, bfk: &str, aqe: &str) -> Result<(), &'static str>;

    
    fn irg(&mut self) -> Result<(), &'static str>;

    
    fn cwo(&self) -> Option<String>;

    
    fn fgf(&self) -> Option<u8>;

    
    fn jqh(&self) -> Option<i8>;
}






static QS_: Mutex<Option<Box<dyn Afr>>> = Mutex::new(None);
static BJA_: AtomicBool = AtomicBool::new(false);


static BFA_: Mutex<Vec<Uz>> = Mutex::new(Vec::new());


static FA_: Mutex<WifiState> = Mutex::new(WifiState::Bni);


static AAP_: Mutex<Option<String>> = Mutex::new(None);


static AOW_: Mutex<Option<(String, String)>> = Mutex::new(None);






pub fn ywq() -> bool {
    BJA_.load(Ordering::Relaxed)
}


pub fn g() -> WifiState {
    *FA_.lock()
}


pub fn lfz() -> bool {
    *FA_.lock() == WifiState::Dl
}


pub fn cwo() -> Option<String> {
    AAP_.lock().clone()
}


pub fn jqh() -> Option<i8> {
    QS_.lock().as_ref().and_then(|bc| bc.jqh())
}


pub fn pod() -> Result<(), &'static str> {
    let mut adb = QS_.lock();
    let rj = adb.as_mut().ok_or("No WiFi driver")?;
    *FA_.lock() = WifiState::Uj;
    rj.arx()
}


pub fn nym() -> Vec<Uz> {
    BFA_.lock().clone()
}


pub fn poll() {
    let mut adb = QS_.lock();
    if let Some(rj) = adb.as_mut() {
        rj.poll();

        
        let jgt = rj.biy();
        let lqb = *FA_.lock();

        if jgt != lqb {
            *FA_.lock() = jgt;
            crate::serial_println!("[WIFI] State: {:?} -> {:?}", lqb, jgt);
        }

        
        if lqb == WifiState::Uj && jgt != WifiState::Uj {
            let hd = rj.eia();
            crate::serial_println!("[WIFI] Scan complete: {} networks found", hd.len());
            *BFA_.lock() = hd;
        }

        
        *AAP_.lock() = rj.cwo();

        
        let request = AOW_.lock().take();
        if let Some((bfk, aqe)) = request {
            crate::serial_println!("[WIFI] Connecting to '{}'...", bfk);
            match rj.ipa(&bfk, &aqe) {
                Ok(()) => {
                    *FA_.lock() = WifiState::Aas;
                }
                Err(aa) => {
                    crate::serial_println!("[WIFI] Connect failed: {}", aa);
                    *FA_.lock() = WifiState::Kk;
                }
            }
        }
    }
}


pub fn lzk(bfk: &str, aqe: &str) {
    *AOW_.lock() = Some((String::from(bfk), String::from(aqe)));
}


pub fn irg() -> Result<(), &'static str> {
    let mut adb = QS_.lock();
    let rj = adb.as_mut().ok_or("No WiFi driver")?;
    rj.irg()?;
    *FA_.lock() = WifiState::Lg;
    *AAP_.lock() = None;
    Ok(())
}


pub fn wit(rj: Box<dyn Afr>) {
    crate::log!("[WIFI] WiFi driver active: {}", rj.co().j);
    *QS_.lock() = Some(rj);
    BJA_.store(true, Ordering::SeqCst);
    *FA_.lock() = WifiState::Lg;
}


pub fn gpv(sq: &S) -> bool {
    
    
    let tzl = sq.ajz == crate::pci::class::Bwi
        || (sq.ajz == crate::pci::class::Qa && sq.adl == 0x80);

    if !tzl {
        return false;
    }

    crate::serial_println!("[WIFI] Found wireless device: {:04X}:{:04X} at {}.{}.{}",
        sq.ml, sq.mx,
        sq.aq, sq.de, sq.gw);

    
    if sq.ml == 0x8086 {
        if let Some(rj) = super::iwl4965::probe(sq) {
            wit(rj);
            return true;
        }
    }

    false
}
