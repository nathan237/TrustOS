



use alloc::vec::Vec;


pub const AZW_: usize = 4096;


#[derive(Debug, Clone)]
pub struct MessageHeader {
    
    pub msg_type: u32,
    
    pub bsg: u64,
    
    pub eil: u64,
    
    pub aea: u64,
    
    pub capability: u64,
}

impl MessageHeader {
    
    pub fn new(msg_type: u32, bsg: u64, capability: u64) -> Self {
        Self {
            msg_type,
            bsg,
            eil: 0,
            aea: crate::logger::fjp(),
            capability,
        }
    }
}


#[derive(Debug, Clone)]
pub enum MessagePayload {
    
    Jl,
    
    Aug([u8; 64]),
    
    Bir(Vec<u8>),
    
    Byt {
        
        ki: u64,
        
        aw: usize,
    },
}

impl MessagePayload {
    
    pub fn aw(&self) -> usize {
        match self {
            MessagePayload::Jl => 0,
            MessagePayload::Aug(f) => f.len(),
            MessagePayload::Bir(f) => f.len(),
            MessagePayload::Byt { aw, .. } => *aw,
        }
    }
    
    
    pub fn eca(f: &[u8]) -> Self {
        if f.is_empty() {
            MessagePayload::Jl
        } else if f.len() <= 64 {
            let mut inline = [0u8; 64];
            inline[..f.len()].dg(f);
            MessagePayload::Aug(inline)
        } else {
            MessagePayload::Bir(f.ip())
        }
    }
}


#[derive(Debug, Clone)]
pub struct Cj {
    
    pub dh: MessageHeader,
    
    pub ew: MessagePayload,
}

impl Cj {
    
    pub fn new(msg_type: u32, bsg: u64, capability: u64, ew: MessagePayload) -> Self {
        Self {
            dh: MessageHeader::new(msg_type, bsg, capability),
            ew,
        }
    }
    
    
    pub fn cug(msg_type: u32, bsg: u64, capability: u64) -> Self {
        Self::new(msg_type, bsg, capability, MessagePayload::Jl)
    }
    
    
    pub fn zwc(msg_type: u32, bsg: u64, capability: u64, f: &[u8]) -> Self {
        Self::new(msg_type, bsg, capability, MessagePayload::eca(f))
    }
}


pub mod msg_types {
    pub const Ua: u32 = 1;
    pub const Dfp: u32 = 2;
    pub const Ayg: u32 = 3;
    pub const Sf: u32 = 4;
    pub const Uf: u32 = 5;
}
