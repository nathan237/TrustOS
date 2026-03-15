




mod channel;
mod message;

use spin::Mutex;
use alloc::collections::BTreeMap;
use core::sync::atomic::{AtomicU64, Ordering};

pub use channel::{Channel, Kg, Rt};
pub use message::{Cj, MessageHeader, MessagePayload};


static Dv: Mutex<BTreeMap<Kg, Channel>> = Mutex::new(BTreeMap::new());


static CHN_: AtomicU64 = AtomicU64::new(1);


static BAK_: AtomicU64 = AtomicU64::new(0);
static AFI_: AtomicU64 = AtomicU64::new(0);


pub fn init() {
    crate::log!("IPC ready");
}

pub fn whk(bm: u64) {
    crate::log_debug!("IPC send {}", bm);
}

pub fn vtc(bm: u64) -> u64 {
    crate::log_debug!("IPC recv {}", bm);
    0
}

pub fn rqk() -> u64 {
    crate::log!("Create IPC channel");
    1
}


pub fn ipp() -> (Rt, Rt) {
    let ad = Kg(CHN_.fetch_add(1, Ordering::Relaxed));
    let channel = Channel::new(ad);
    
    let bsg = channel.bsg();
    let afw = channel.afw();
    
    Dv.lock().insert(ad, channel);
    
    crate::log_debug!("Created IPC channel {:?}", ad);
    
    (bsg, afw)
}


pub fn baq(channel: Kg, message: Cj) -> Result<(), IpcError> {
    let lq = Dv.lock();
    let channel = lq.get(&channel).ok_or(IpcError::Apw)?;
    
    channel.baq(message)?;
    BAK_.fetch_add(1, Ordering::Relaxed);
    
    Ok(())
}


pub fn chb(channel: Kg) -> Result<Cj, IpcError> {
    let lq = Dv.lock();
    let channel = lq.get(&channel).ok_or(IpcError::Apw)?;
    
    let fr = channel.chb()?;
    AFI_.fetch_add(1, Ordering::Relaxed);
    
    Ok(fr)
}


pub fn pwh(channel: Kg) -> Result<Option<Cj>, IpcError> {
    let lq = Dv.lock();
    let channel = lq.get(&channel).ok_or(IpcError::Apw)?;
    
    match channel.pwh() {
        Ok(fr) => {
            AFI_.fetch_add(1, Ordering::Relaxed);
            Ok(Some(fr))
        }
        Err(IpcError::Zn) => Ok(None),
        Err(aa) => Err(aa),
    }
}


pub fn yit(channel: Kg) {
    Dv.lock().remove(&channel);
    crate::log_debug!("Closed IPC channel {:?}", channel);
}


pub fn cm() -> Bka {
    Bka {
        qye: Dv.lock().len(),
        unt: BAK_.load(Ordering::Relaxed),
        uns: AFI_.load(Ordering::Relaxed),
    }
}


#[derive(Debug, Clone)]
pub struct Bka {
    pub qye: usize,
    pub unt: u64,
    pub uns: u64,
}


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IpcError {
    
    Apw,
    
    Aak,
    
    Byu,
    
    Zn,
    
    Jt,
    
    Czy,
}
