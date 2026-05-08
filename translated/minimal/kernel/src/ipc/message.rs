



use alloc::vec::Vec;


pub const BBY_: usize = 4096;


#[derive(Debug, Clone)]
pub struct MessageHeader {
    
    pub msg_type: u32,
    
    pub sender: u64,
    
    pub sequence: u64,
    
    pub timestamp: u64,
    
    pub capability: u64,
}

impl MessageHeader {
    
    pub fn new(msg_type: u32, sender: u64, capability: u64) -> Self {
        Self {
            msg_type,
            sender,
            sequence: 0,
            timestamp: crate::logger::ckc(),
            capability,
        }
    }
}


#[derive(Debug, Clone)]
pub enum MessagePayload {
    
    Empty,
    
    Inline([u8; 64]),
    
    Heap(Vec<u8>),
    
    Buffer {
        
        phys_addr: u64,
        
        size: usize,
    },
}

impl MessagePayload {
    
    pub fn size(&self) -> usize {
        match self {
            MessagePayload::Empty => 0,
            MessagePayload::Inline(data) => data.len(),
            MessagePayload::Heap(data) => data.len(),
            MessagePayload::Buffer { size, .. } => *size,
        }
    }
    
    
    pub fn bsv(data: &[u8]) -> Self {
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


#[derive(Debug, Clone)]
pub struct Az {
    
    pub header: MessageHeader,
    
    pub payload: MessagePayload,
}

impl Az {
    
    pub fn new(msg_type: u32, sender: u64, capability: u64, payload: MessagePayload) -> Self {
        Self {
            header: MessageHeader::new(msg_type, sender, capability),
            payload,
        }
    }
    
    
    pub fn ash(msg_type: u32, sender: u64, capability: u64) -> Self {
        Self::new(msg_type, sender, capability, MessagePayload::Empty)
    }
    
    
    pub fn rco(msg_type: u32, sender: u64, capability: u64, data: &[u8]) -> Self {
        Self::new(msg_type, sender, capability, MessagePayload::bsv(data))
    }
}


pub mod msg_types {
    pub const Iq: u32 = 1;
    pub const Bbi: u32 = 2;
    pub const Uv: u32 = 3;
    pub const Hr: u32 = 4;
    pub const Iu: u32 = 5;
}
