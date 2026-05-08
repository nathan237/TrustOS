




use alloc::boxed::Box;
use alloc::string::String;
use alloc::vec::Vec;
use spin::Mutex;
use core::sync::atomic::{AtomicBool, Ordering};

use super::{Dd, NetStats};
use crate::drivers::{Cw, Bb, DriverStatus, DriverCategory};
use crate::pci::L;






#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WifiSecurity {
    Open,
    WEP,
    WPA,
    WPA2,
    WPA3,
    Unknown,
}

impl WifiSecurity {
    pub fn as_str(&self) -> &'static str {
        match self {
            WifiSecurity::Open => "Open",
            WifiSecurity::WEP => "WEP",
            WifiSecurity::WPA => "WPA",
            WifiSecurity::WPA2 => "WPA2",
            WifiSecurity::WPA3 => "WPA3",
            WifiSecurity::Unknown => "???",
        }
    }
}


#[derive(Debug, Clone)]
pub struct Fg {
    
    pub ssid: String,
    
    pub bssid: [u8; 6],
    
    pub channel: u8,
    
    pub signal_dbm: i8,
    
    pub security: WifiSecurity,
    
    pub frequency_mhz: u16,
}

impl Fg {
    
    pub fn signal_quality(&self) -> u8 {
        
        if self.signal_dbm >= -30 { return 100; }
        if self.signal_dbm <= -90 { return 0; }
        ((self.signal_dbm as i16 + 90) * 100 / 60) as u8
    }

    
    pub fn signal_bars(&self) -> u8 {
        match self.signal_quality() {
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
    
    NoHardware,
    
    Disabled,
    
    Disconnected,
    
    Scanning,
    
    Connecting,
    
    Authenticating,
    
    Connected,
    
    Failed,
}


pub trait Nx: Dd {
    
    fn wifi_state(&self) -> WifiState;

    
    fn scan(&mut self) -> Result<(), &'static str>;

    
    fn scan_results(&self) -> Vec<Fg>;

    
    fn connect(&mut self, ssid: &str, uy: &str) -> Result<(), &'static str>;

    
    fn disconnect(&mut self) -> Result<(), &'static str>;

    
    fn connected_ssid(&self) -> Option<String>;

    
    fn current_channel(&self) -> Option<u8>;

    
    fn signal_strength(&self) -> Option<i8>;
}






pub(crate) static EN_: Mutex<Option<Box<dyn Nx>>> = Mutex::new(None);
static RM_: AtomicBool = AtomicBool::new(false);


static TU_: Mutex<Option<(u8, u8, u8)>> = Mutex::new(None);


static BHE_: Mutex<Vec<Fg>> = Mutex::new(Vec::new());


static FP_: Mutex<WifiState> = Mutex::new(WifiState::NoHardware);


static ACC_: Mutex<Option<String>> = Mutex::new(None);


static AQW_: Mutex<Option<(String, String)>> = Mutex::new(None);






pub fn oot(bus: u8, device: u8, function: u8) {
    *TU_.lock() = Some((bus, device, function));
    crate::serial_println!("[WIFI] Deferred PCI probe stored: {}.{}.{}", bus, device, function);
}


pub fn ckk() -> bool {
    RM_.load(Ordering::Relaxed) || TU_.lock().is_some()
}


pub fn qkk() -> bool {
    RM_.load(Ordering::Relaxed)
}



pub fn mxj() -> Result<(), &'static str> {
    
    if RM_.load(Ordering::Relaxed) {
        return Ok(());
    }

    let nst = TU_.lock().take();
    let (bus, s, func) = nst.ok_or("No WiFi hardware detected during boot")?;

    crate::println!("  WiFi lazy probe: PCI {}.{}.{}", bus, s, func);
    crate::serial_println!("[WIFI] Lazy probe: {}.{}.{}", bus, s, func);

    
    let devices = crate::pci::aqs();
    let go = devices.iter()
        .find(|d| d.bus == bus && d.device == s && d.function == func)
        .ok_or("WiFi PCI device not found")?;

    
    if ccv(go) {
        crate::println!("  WiFi probe OK: iwl4965");
        Ok(())
    } else {
        
        *TU_.lock() = Some((bus, s, func));
        Err("WiFi hardware probe failed")
    }
}


pub fn state() -> WifiState {
    *FP_.lock()
}


pub fn czx() -> bool {
    *FP_.lock() == WifiState::Connected
}


pub fn connected_ssid() -> Option<String> {
    ACC_.lock().clone()
}


pub fn signal_strength() -> Option<i8> {
    EN_.lock().as_ref().and_then(|d| d.signal_strength())
}



pub fn fux() -> Result<(), &'static str> {
    
    if !RM_.load(Ordering::Relaxed) {
        mxj()?;
    }

    let mut jg = EN_.lock();
    let driver = jg.as_mut().ok_or("No WiFi driver")?;
    if driver.status() == crate::drivers::DriverStatus::Running {
        return Ok(());
    }
    crate::serial_println!("[WIFI] Auto-starting driver (hw_init + firmware)...");
    crate::println!("  Starting WiFi hardware...");
    match driver.start() {
        Ok(()) => {
            crate::serial_println!("[WIFI] Driver started successfully");
            crate::println!("  WiFi hardware initialized");
            Ok(())
        }
        Err(e) => {
            crate::serial_println!("[WIFI] Driver start failed: {}", e);
            crate::println!("  WiFi start failed: {}", e);
            Err(e)
        }
    }
}


pub fn eaj() -> Result<(), &'static str> {
    
    fux()?;

    {
        let mut jg = EN_.lock();
        let driver = jg.as_mut().ok_or("No WiFi driver")?;
        *FP_.lock() = WifiState::Scanning;
        driver.scan()
    }
}


pub fn cys() -> Vec<Fg> {
    BHE_.lock().clone()
}


pub fn poll() {
    let mut jg = EN_.lock();
    if let Some(driver) = jg.as_mut() {
        driver.poll();

        
        let euw = driver.wifi_state();
        let gkr = *FP_.lock();

        if euw != gkr {
            *FP_.lock() = euw;
            crate::serial_println!("[WIFI] State: {:?} -> {:?}", gkr, euw);
        }

        
        if gkr == WifiState::Scanning && euw != WifiState::Scanning {
            let results = driver.scan_results();
            crate::serial_println!("[WIFI] Scan complete: {} networks found", results.len());
            *BHE_.lock() = results;
        }

        
        *ACC_.lock() = driver.connected_ssid();

        
        let request = AQW_.lock().take();
        if let Some((ssid, uy)) = request {
            crate::serial_println!("[WIFI] Connecting to '{}'...", ssid);
            match driver.connect(&ssid, &uy) {
                Ok(()) => {
                    *FP_.lock() = WifiState::Connecting;
                }
                Err(e) => {
                    crate::serial_println!("[WIFI] Connect failed: {}", e);
                    *FP_.lock() = WifiState::Failed;
                }
            }
        }
    }
}


pub fn eyl(ssid: &str, uy: &str) {
    
    if let Err(e) = fux() {
        crate::serial_println!("[WIFI] Cannot connect — start failed: {}", e);
        return;
    }
    *AQW_.lock() = Some((String::from(ssid), String::from(uy)));
}


pub fn disconnect() -> Result<(), &'static str> {
    let mut jg = EN_.lock();
    let driver = jg.as_mut().ok_or("No WiFi driver")?;
    driver.disconnect()?;
    *FP_.lock() = WifiState::Disconnected;
    *ACC_.lock() = None;
    Ok(())
}


pub fn oov(driver: Box<dyn Nx>) {
    crate::log!("[WIFI] WiFi driver active: {}", driver.info().name);
    *EN_.lock() = Some(driver);
    RM_.store(true, Ordering::SeqCst);
    *FP_.lock() = WifiState::Disconnected;
}


pub fn ccv(go: &L) -> bool {
    
    crate::serial_println!("[WIFI-PROBE] Checking {:04X}:{:04X} class={:02X} sub={:02X} at {}.{}.{}",
        go.vendor_id, go.device_id,
        go.class_code, go.subclass,
        go.bus, go.device, go.function);

    
    
    
    let mue = go.class_code == crate::pci::class::Agk
        || (go.class_code == crate::pci::class::Gr && go.subclass == 0x80)
        || (go.vendor_id == 0x8086 && super::iwl4965::AFI_.contains(&go.device_id));

    if !mue {
        crate::serial_println!("[WIFI-PROBE] -> Not wireless (class={:02X} sub={:02X} devid={:04X})",
            go.class_code, go.subclass, go.device_id);
        return false;
    }

    crate::serial_println!("[WIFI] Found wireless device: {:04X}:{:04X} at {}.{}.{}",
        go.vendor_id, go.device_id,
        go.bus, go.device, go.function);

    
    if go.vendor_id == 0x8086 {
        if let Some(driver) = super::iwl4965::probe(go) {
            oov(driver);
            return true;
        }
    }

    false
}
