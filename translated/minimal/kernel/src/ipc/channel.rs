



use alloc::collections::VecDeque;
use spin::Mutex;
use core::sync::atomic::{AtomicBool, Ordering};
use super::{Az, IpcError};


const ABM_: usize = 256;


const CIX_: u32 = 100_000;


#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Ed(pub u64);


pub struct Channel {
    
    id: Ed,
    
    buffer: Mutex<VecDeque<Az>>,
    
    closed: AtomicBool,
}

impl Channel {
    
    pub fn new(id: Ed) -> Self {
        Self {
            id,
            buffer: Mutex::new(VecDeque::with_capacity(ABM_)),
            closed: AtomicBool::new(false),
        }
    }
    
    
    pub fn id(&self) -> Ed {
        self.id
    }
    
    
    pub fn send(&self, message: Az) -> Result<(), IpcError> {
        if self.closed.load(Ordering::Acquire) {
            return Err(IpcError::ChannelClosed);
        }
        
        let mut buffer = self.buffer.lock();
        if buffer.len() >= ABM_ {
            return Err(IpcError::BufferFull);
        }
        
        buffer.push_back(message);
        Ok(())
    }
    
    
    pub fn qvg(&self, dug: &[Az]) -> Result<usize, IpcError> {
        if self.closed.load(Ordering::Acquire) {
            return Err(IpcError::ChannelClosed);
        }
        
        let mut buffer = self.buffer.lock();
        let available = ABM_.saturating_sub(buffer.len());
        let count = dug.len().min(available);
        
        for bk in &dug[..count] {
            buffer.push_back(bk.clone());
        }
        
        Ok(count)
    }
    
    
    pub fn receive(&self) -> Result<Az, IpcError> {
        let mut my: u32 = 0;
        loop {
            if let Some(bk) = self.buffer.lock().pop_front() {
                return Ok(bk);
            }
            
            if self.closed.load(Ordering::Acquire) {
                return Err(IpcError::ChannelClosed);
            }
            
            my += 1;
            if my > CIX_ {
                return Err(IpcError::WouldBlock);
            }
            
            
            crate::scheduler::dgw();
        }
    }
    
    
    pub fn try_receive(&self) -> Result<Az, IpcError> {
        if let Some(bk) = self.buffer.lock().pop_front() {
            return Ok(bk);
        }
        
        if self.closed.load(Ordering::Acquire) {
            return Err(IpcError::ChannelClosed);
        }
        
        Err(IpcError::WouldBlock)
    }
    
    
    pub fn qta(&self, max: usize) -> Result<alloc::vec::Vec<Az>, IpcError> {
        let mut buffer = self.buffer.lock();
        let count = buffer.len().min(max);
        
        let dug: alloc::vec::Vec<Az> = buffer.drain(..count).collect();
        
        if dug.is_empty() && self.closed.load(Ordering::Acquire) {
            return Err(IpcError::ChannelClosed);
        }
        
        Ok(dug)
    }
    
    
    pub fn close(&self) {
        self.closed.store(true, Ordering::Release);
    }
    
    
    pub fn sender(&self) -> Hj {
        Hj {
            ath: self.id,
            is_sender: true,
        }
    }
    
    
    pub fn receiver(&self) -> Hj {
        Hj {
            ath: self.id,
            is_sender: false,
        }
    }
}


#[derive(Debug, Clone)]
pub struct Hj {
    pub ath: Ed,
    pub is_sender: bool,
}
