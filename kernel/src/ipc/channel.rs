//! IPC Channel implementation
//! 
//! Lock-free channel with batching support.

use alloc::collections::VecDeque;
use spin::Mutex;
use core::sync::atomic::{AtomicBool, Ordering};
use super::{Message, IpcError};

/// Channel buffer size
const CHANNEL_BUFFER_SIZE: usize = 256;

/// Maximum spin iterations before giving up in blocking receive
const MAX_RECEIVE_SPINS: u32 = 100_000;

/// Unique channel identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ChannelId(pub u64);

/// Bidirectional IPC channel
pub struct Channel {
    /// Channel ID
    id: ChannelId,
    /// Message buffer
    buffer: Mutex<VecDeque<Message>>,
    /// Channel closed flag (AtomicBool avoids extra lock per receive loop)
    closed: AtomicBool,
}

impl Channel {
    /// Create new channel
    pub fn new(id: ChannelId) -> Self {
        Self {
            id,
            buffer: Mutex::new(VecDeque::with_capacity(CHANNEL_BUFFER_SIZE)),
            closed: AtomicBool::new(false),
        }
    }
    
    /// Get channel ID
    pub fn id(&self) -> ChannelId {
        self.id
    }
    
    /// Send message
    pub fn send(&self, message: Message) -> Result<(), IpcError> {
        if self.closed.load(Ordering::Acquire) {
            return Err(IpcError::ChannelClosed);
        }
        
        let mut buffer = self.buffer.lock();
        if buffer.len() >= CHANNEL_BUFFER_SIZE {
            return Err(IpcError::BufferFull);
        }
        
        buffer.push_back(message);
        Ok(())
    }
    
    /// Send batch of messages
    pub fn send_batch(&self, messages: &[Message]) -> Result<usize, IpcError> {
        if self.closed.load(Ordering::Acquire) {
            return Err(IpcError::ChannelClosed);
        }
        
        let mut buffer = self.buffer.lock();
        let available = CHANNEL_BUFFER_SIZE.saturating_sub(buffer.len());
        let count = messages.len().min(available);
        
        for msg in &messages[..count] {
            buffer.push_back(msg.clone());
        }
        
        Ok(count)
    }
    
    /// Receive message (blocking with bounded spin)
    pub fn receive(&self) -> Result<Message, IpcError> {
        let mut spins: u32 = 0;
        loop {
            if let Some(msg) = self.buffer.lock().pop_front() {
                return Ok(msg);
            }
            
            if self.closed.load(Ordering::Acquire) {
                return Err(IpcError::ChannelClosed);
            }
            
            spins += 1;
            if spins > MAX_RECEIVE_SPINS {
                return Err(IpcError::WouldBlock);
            }
            
            // Yield to other tasks
            crate::scheduler::yield_now();
        }
    }
    
    /// Try receive (non-blocking)
    pub fn try_receive(&self) -> Result<Message, IpcError> {
        if let Some(msg) = self.buffer.lock().pop_front() {
            return Ok(msg);
        }
        
        if self.closed.load(Ordering::Acquire) {
            return Err(IpcError::ChannelClosed);
        }
        
        Err(IpcError::WouldBlock)
    }
    
    /// Receive batch of messages
    pub fn receive_batch(&self, max: usize) -> Result<alloc::vec::Vec<Message>, IpcError> {
        let mut buffer = self.buffer.lock();
        let count = buffer.len().min(max);
        
        let messages: alloc::vec::Vec<Message> = buffer.drain(..count).collect();
        
        if messages.is_empty() && self.closed.load(Ordering::Acquire) {
            return Err(IpcError::ChannelClosed);
        }
        
        Ok(messages)
    }
    
    /// Close channel
    pub fn close(&self) {
        self.closed.store(true, Ordering::Release);
    }
    
    /// Create sender endpoint
    pub fn sender(&self) -> ChannelEnd {
        ChannelEnd {
            channel_id: self.id,
            is_sender: true,
        }
    }
    
    /// Create receiver endpoint
    pub fn receiver(&self) -> ChannelEnd {
        ChannelEnd {
            channel_id: self.id,
            is_sender: false,
        }
    }
}

/// Channel endpoint handle
#[derive(Debug, Clone)]
pub struct ChannelEnd {
    pub channel_id: ChannelId,
    pub is_sender: bool,
}
