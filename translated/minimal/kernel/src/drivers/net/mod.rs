




use alloc::boxed::Box;
use alloc::vec::Vec;
use spin::Mutex;
use core::sync::atomic::{AtomicBool, Ordering};

use super::{Gi, Co, DriverStatus, DriverCategory, nw};
use crate::pci::S;


pub trait Ha: Gi {
    
    fn csg(&self) -> [u8; 6];
    
    
    fn aik(&self) -> bool;
    
    
    fn gll(&self) -> u32 { 0 }
    
    
    fn baq(&mut self, f: &[u8]) -> Result<(), &'static str>;
    
    
    fn chb(&mut self) -> Option<Vec<u8>>;
    
    
    fn poll(&mut self);
    
    
    fn cm(&self) -> NetStats;
    
    
    fn pjd(&mut self, qbs: bool) -> Result<(), &'static str> {
        Err("Not supported")
    }
    
    
    fn yel(&mut self, yat: [u8; 6]) -> Result<(), &'static str> {
        Err("Not supported")
    }
}


#[derive(Debug, Clone, Copy, Default)]
pub struct NetStats {
    pub cuz: u64,
    pub dbo: u64,
    pub bpc: u64,
    pub bsc: u64,
    pub dmv: u64,
    pub dbn: u64,
    pub mnn: u64,
    pub mbk: u64,
}


static HJ_: Mutex<Option<Box<dyn Ha>>> = Mutex::new(None);
static AQP_: AtomicBool = AtomicBool::new(false);


struct Bqw {
    co: Co,
    fig: fn() -> Box<dyn Ha>,
}


static BBI_: Mutex<Vec<Bqw>> = Mutex::new(Vec::new());





mod virtio;





mod e1000;





mod rtl8139;





mod rtl8169;





pub mod wifi;
pub mod iwl4965;






pub fn vuc() {
    virtio::nw();
    e1000::nw();
    rtl8139::nw();
    rtl8169::nw();
}


pub fn jly(co: Co, fig: fn() -> Box<dyn Ha>) {
    let mut chc = BBI_.lock();
    chc.push(Bqw { co, fig });
}


pub fn lvo(sq: &S) -> bool {
    let chc = BBI_.lock();
    for bt in chc.iter() {
        for &(acs, de) in bt.co.fye {
            if sq.ml == acs && (de == 0xFFFF || sq.mx == de) {
                let mut rj = (bt.fig)();
                match rj.probe(sq) {
                    Ok(()) => {
                        if let Err(aa) = rj.ay() {
                            crate::log_warn!("[DRIVERS] Failed to start {}: {}", bt.co.j, aa);
                            return false;
                        }
                        *HJ_.lock() = Some(rj);
                        AQP_.store(true, Ordering::SeqCst);
                        return true;
                    }
                    Err(aa) => {
                        crate::log_debug!("[DRIVERS] {} probe failed: {}", bt.co.j, aa);
                    }
                }
            }
        }
    }
    false
}


pub fn bzy() -> bool {
    AQP_.load(Ordering::Relaxed)
}


pub fn cez() -> Option<[u8; 6]> {
    HJ_.lock().as_ref().map(|bc| bc.csg())
}


pub fn aik() -> bool {
    HJ_.lock().as_ref().map(|bc| bc.aik()).unwrap_or(false)
}


pub fn baq(f: &[u8]) -> Result<(), &'static str> {
    let mut adb = HJ_.lock();
    let rj = adb.as_mut().ok_or("No network driver")?;
    rj.baq(f)
}


pub fn chb() -> Option<Vec<u8>> {
    let mut adb = HJ_.lock();
    adb.as_mut().and_then(|bc| bc.chb())
}


pub fn poll() {
    if let Some(rj) = HJ_.lock().as_mut() {
        rj.poll();
    }
}


pub fn cm() -> NetStats {
    HJ_.lock().as_ref()
        .map(|bc| bc.cm())
        .age()
}
