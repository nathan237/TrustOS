




mod channel;
mod message;

use spin::Mutex;
use alloc::collections::BTreeMap;
use core::sync::atomic::{AtomicU64, Ordering};

pub use channel::{Channel, Ed, Hj};
pub use message::{Az, MessageHeader, MessagePayload};


static Bq: Mutex<BTreeMap<Ed, Channel>> = Mutex::new(BTreeMap::new());


static CKW_: AtomicU64 = AtomicU64::new(1);


static BCM_: AtomicU64 = AtomicU64::new(0);
static AHC_: AtomicU64 = AtomicU64::new(0);


pub fn init() {
    crate::log!("IPC ready");
}

pub fn onx(ch: u64) {
    crate::log_debug!("IPC send {}", ch);
}

pub fn odo(ch: u64) -> u64 {
    crate::log_debug!("IPC recv {}", ch);
    0
}

pub fn kzf() -> u64 {
    crate::log!("Create IPC channel");
    1
}


pub fn ejc() -> (Hj, Hj) {
    let id = Ed(CKW_.fetch_add(1, Ordering::Relaxed));
    let channel = Channel::new(id);
    
    let sender = channel.sender();
    let receiver = channel.receiver();
    
    Bq.lock().insert(id, channel);
    
    crate::log_debug!("Created IPC channel {:?}", id);
    
    (sender, receiver)
}


pub fn send(channel: Ed, message: Az) -> Result<(), IpcError> {
    let channels = Bq.lock();
    let channel = channels.get(&channel).ok_or(IpcError::ChannelNotFound)?;
    
    channel.send(message)?;
    BCM_.fetch_add(1, Ordering::Relaxed);
    
    Ok(())
}


pub fn receive(channel: Ed) -> Result<Az, IpcError> {
    let channels = Bq.lock();
    let channel = channels.get(&channel).ok_or(IpcError::ChannelNotFound)?;
    
    let bk = channel.receive()?;
    AHC_.fetch_add(1, Ordering::Relaxed);
    
    Ok(bk)
}


pub fn try_receive(channel: Ed) -> Result<Option<Az>, IpcError> {
    let channels = Bq.lock();
    let channel = channels.get(&channel).ok_or(IpcError::ChannelNotFound)?;
    
    match channel.try_receive() {
        Ok(bk) => {
            AHC_.fetch_add(1, Ordering::Relaxed);
            Ok(Some(bk))
        }
        Err(IpcError::WouldBlock) => Ok(None),
        Err(e) => Err(e),
    }
}


pub fn qae(channel: Ed) {
    Bq.lock().remove(&channel);
    crate::log_debug!("Closed IPC channel {:?}", channel);
}


pub fn stats() -> Aaf {
    Aaf {
        channels_active: Bq.lock().len(),
        messages_sent: BCM_.load(Ordering::Relaxed),
        messages_received: AHC_.load(Ordering::Relaxed),
    }
}


#[derive(Debug, Clone)]
pub struct Aaf {
    pub channels_active: usize,
    pub messages_sent: u64,
    pub messages_received: u64,
}


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IpcError {
    
    ChannelNotFound,
    
    ChannelClosed,
    
    BufferFull,
    
    WouldBlock,
    
    PermissionDenied,
    
    InvalidMessage,
}
