//! IPC Message types
//! 
//! Message structures for inter-process communication.

use alloc::vec::Vec;

/// Maximum message payload size
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const MAXIMUM_PAYLOAD_SIZE: usize = 4096;

/// Message header
#[derive(Debug, Clone)]
// Public structure — visible outside this module.
pub struct MessageHeader {
    /// Message type ID
    pub msg_type: u32,
    /// Sender task ID
    pub sender: u64,
    /// Sequence number for ordering
    pub sequence: u64,
    /// Timestamp (kernel ticks)
    pub timestamp: u64,
    /// Capability token for authorization
    pub capability: u64,
}

// Implementation block — defines methods for the type above.
impl MessageHeader {
    /// Create new header
    pub fn new(msg_type: u32, sender: u64, capability: u64) -> Self {
        Self {
            msg_type,
            sender,
            sequence: 0,
            timestamp: crate::logger::get_timestamp(),
            capability,
        }
    }
}

/// Message payload
#[derive(Debug, Clone)]
// Enumeration — a type that can be one of several variants.
pub enum MessagePayload {
    /// Empty message (signal)
    Empty,
    /// Small inline data
    Inline([u8; 64]),
    /// Heap-allocated data
    Heap(Vec<u8>),
    /// Zero-copy buffer reference
    Buffer {
        /// Physical address of buffer
        physical_address: u64,
        /// Buffer size
        size: usize,
    },
}

// Implementation block — defines methods for the type above.
impl MessagePayload {
    /// Get payload size
    pub fn size(&self) -> usize {
                // Pattern matching — Rust's exhaustive branching construct.
match self {
            MessagePayload::Empty => 0,
            MessagePayload::Inline(data) => data.len(),
            MessagePayload::Heap(data) => data.len(),
            MessagePayload::Buffer { size, .. } => *size,
        }
    }
    
    /// Create inline payload from bytes
    pub fn from_bytes(data: &[u8]) -> Self {
        if data.is_empty() {
            MessagePayload::Empty
        } else if data.len() <= 64 {
            let mut inline = [0u8; 64];
            inline[..data.len()].copy_from_slice(data);
            MessagePayload::Inline(inline)
        } else {
            MessagePayload::Heap(data.to_vec())
        }
    }
}

/// Complete IPC message
#[derive(Debug, Clone)]
// Public structure — visible outside this module.
pub struct Message {
    /// Message header
    pub header: MessageHeader,
    /// Message payload
    pub payload: MessagePayload,
}

// Implementation block — defines methods for the type above.
impl Message {
    /// Create new message
    pub fn new(msg_type: u32, sender: u64, capability: u64, payload: MessagePayload) -> Self {
        Self {
            header: MessageHeader::new(msg_type, sender, capability),
            payload,
        }
    }
    
    /// Create empty signal message
    pub fn signal(msg_type: u32, sender: u64, capability: u64) -> Self {
        Self::new(msg_type, sender, capability, MessagePayload::Empty)
    }
    
    /// Create message with inline data
    pub fn with_data(msg_type: u32, sender: u64, capability: u64, data: &[u8]) -> Self {
        Self::new(msg_type, sender, capability, MessagePayload::from_bytes(data))
    }
}

/// Well-known message types
pub mod msg_types {
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const REQUEST: u32 = 1;
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const RESPONSE: u32 = 2;
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const SIGNAL: u32 = 3;
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const ERROR: u32 = 4;
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const SHUTDOWN: u32 = 5;
}
