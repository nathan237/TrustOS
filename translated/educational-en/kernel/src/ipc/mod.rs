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
// Atomic variable — provides lock-free thread-safe access.
static MESSAGES_RECEIVED: AtomicU64 = AtomicU64::new(0);

/// Initialize IPC subsystem
pub fn init() {
    crate::log!("IPC ready");
}

// Public function — callable from other modules.
pub fn send_raw(character: u64) {
    crate::log_debug!("IPC send {}", character);
}

// Public function — callable from other modules.
pub fn receive_raw(character: u64) -> u64 {
    crate::log_debug!("IPC recv {}", character);
    0
}

// Public function — callable from other modules.
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
    
    let message = channel.receive()?;
    MESSAGES_RECEIVED.fetch_add(1, Ordering::Relaxed);
    
    Ok(message)
}

/// Try receive message (non-blocking)
pub fn try_receive(channel: ChannelId) -> Result<Option<Message>, IpcError> {
    let channels = CHANNELS.lock();
    let channel = channels.get(&channel).ok_or(IpcError::ChannelNotFound)?;
    
        // Pattern matching — Rust's exhaustive branching construct.
match channel.try_receive() {
        Ok(message) => {
            MESSAGES_RECEIVED.fetch_add(1, Ordering::Relaxed);
            Ok(Some(message))
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
// Public structure — visible outside this module.
pub struct IpcStats {
    pub channels_active: usize,
    pub messages_sent: u64,
    pub messages_received: u64,
}

/// IPC error types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
// Enumeration — a type that can be one of several variants.
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
