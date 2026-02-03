//! Inter-Process Communication Subsystem
//! 
//! Async, batched IPC with zero-copy message passing where possible.
//! All IPC is capability-protected.

mod channel;
mod message;

use spin::Mutex;
use alloc::collections::BTreeMap;
use core::sync::atomic::{AtomicU64, Ordering};

pub use channel::{Channel, ChannelId, ChannelEnd};
pub use message::{Message, MessageHeader, MessagePayload};

/// Global channel registry
static CHANNELS: Mutex<BTreeMap<ChannelId, Channel>> = Mutex::new(BTreeMap::new());

/// Next channel ID
static NEXT_CHANNEL_ID: AtomicU64 = AtomicU64::new(1);

/// IPC statistics
static MESSAGES_SENT: AtomicU64 = AtomicU64::new(0);
static MESSAGES_RECEIVED: AtomicU64 = AtomicU64::new(0);

/// Initialize IPC subsystem
pub fn init() {
    crate::log!("IPC ready");
}

pub fn send_raw(ch: u64) {
    crate::log_debug!("IPC send {}", ch);
}

pub fn receive_raw(ch: u64) -> u64 {
    crate::log_debug!("IPC recv {}", ch);
    0
}

pub fn create_channel_raw() -> u64 {
    crate::log!("Create IPC channel");
    1
}


/// Create new IPC channel
pub fn create_channel() -> (ChannelEnd, ChannelEnd) {
    let id = ChannelId(NEXT_CHANNEL_ID.fetch_add(1, Ordering::Relaxed));
    let channel = Channel::new(id);
    
    let sender = channel.sender();
    let receiver = channel.receiver();
    
    CHANNELS.lock().insert(id, channel);
    
    crate::log_debug!("Created IPC channel {:?}", id);
    
    (sender, receiver)
}

/// Send message through channel
pub fn send(channel: ChannelId, message: Message) -> Result<(), IpcError> {
    let channels = CHANNELS.lock();
    let channel = channels.get(&channel).ok_or(IpcError::ChannelNotFound)?;
    
    channel.send(message)?;
    MESSAGES_SENT.fetch_add(1, Ordering::Relaxed);
    
    Ok(())
}

/// Receive message from channel (blocking)
pub fn receive(channel: ChannelId) -> Result<Message, IpcError> {
    let channels = CHANNELS.lock();
    let channel = channels.get(&channel).ok_or(IpcError::ChannelNotFound)?;
    
    let msg = channel.receive()?;
    MESSAGES_RECEIVED.fetch_add(1, Ordering::Relaxed);
    
    Ok(msg)
}

/// Try receive message (non-blocking)
pub fn try_receive(channel: ChannelId) -> Result<Option<Message>, IpcError> {
    let channels = CHANNELS.lock();
    let channel = channels.get(&channel).ok_or(IpcError::ChannelNotFound)?;
    
    match channel.try_receive() {
        Ok(msg) => {
            MESSAGES_RECEIVED.fetch_add(1, Ordering::Relaxed);
            Ok(Some(msg))
        }
        Err(IpcError::WouldBlock) => Ok(None),
        Err(e) => Err(e),
    }
}

/// Close channel
pub fn close_channel(channel: ChannelId) {
    CHANNELS.lock().remove(&channel);
    crate::log_debug!("Closed IPC channel {:?}", channel);
}

/// Get IPC statistics
pub fn stats() -> IpcStats {
    IpcStats {
        channels_active: CHANNELS.lock().len(),
        messages_sent: MESSAGES_SENT.load(Ordering::Relaxed),
        messages_received: MESSAGES_RECEIVED.load(Ordering::Relaxed),
    }
}

/// IPC statistics
#[derive(Debug, Clone)]
pub struct IpcStats {
    pub channels_active: usize,
    pub messages_sent: u64,
    pub messages_received: u64,
}

/// IPC error types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IpcError {
    /// Channel not found
    ChannelNotFound,
    /// Channel closed
    ChannelClosed,
    /// Buffer full
    BufferFull,
    /// Would block (for non-blocking ops)
    WouldBlock,
    /// Permission denied
    PermissionDenied,
    /// Invalid message
    InvalidMessage,
}
