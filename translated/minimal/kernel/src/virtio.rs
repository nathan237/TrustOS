





use core::sync::atomic::Ordering;
use alloc::vec::Vec;
use alloc::boxed::Box;
use crate::arch::Port;


pub mod status {
    pub const Gf: u8 = 1;
    pub const Cl: u8 = 2;
    pub const IQ_: u8 = 4;
    pub const NY_: u8 = 8;
    pub const BUH_: u8 = 64;
    pub const Sa: u8 = 128;
}


pub mod cap_type {
    pub const AQU_: u8 = 1;
    pub const BDR_: u8 = 2;
    pub const AZL_: u8 = 3;
    pub const ASD_: u8 = 4;
    pub const ECV_: u8 = 5;
}


pub mod legacy_reg {
    pub const BUG_: u16 = 0x00;      
    pub const BVA_: u16 = 0x04;      
    pub const CQE_: u16 = 0x08;        
    pub const XV_: u16 = 0x0C;           
    pub const AIO_: u16 = 0x0E;         
    pub const CQK_: u16 = 0x10;         
    pub const ES_: u16 = 0x12;        
    pub const CFZ_: u16 = 0x13;           
    
    pub const AHQ_: u16 = 0x14;              
    pub const DYT_: u16 = 0x1A;           
}


pub mod desc_flags {
    pub const Pn: u16 = 1;       
    pub const Bh: u16 = 2;      
    pub const Axc: u16 = 4;   
}


#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct VirtqDesc {
    pub addr: u64,    
    pub len: u32,     
    pub flags: u16,   
    pub next: u16,    
}


#[repr(C)]
#[derive(Debug)]
pub struct Kx {
    pub flags: u16,
    pub idx: u16,
    
}


#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct Qv {
    pub id: u32,    
    pub len: u32,   
}


#[repr(C)]
#[derive(Debug)]
pub struct Ky {
    pub flags: u16,
    pub idx: u16,
    
}


pub struct Virtqueue {
    
    pub size: u16,
    
    pub phys_addr: u64,
    
    pub desc: *mut VirtqDesc,
    
    pub avail: *mut Kx,
    
    pub used: *mut Ky,
    
    pub last_used_idx: u16,
    
    pub free_head: u16,
    
    pub num_free: u16,
    
    pub free_list: Vec<u16>,
}



unsafe impl Send for Virtqueue {}
unsafe impl Sync for Virtqueue {}

impl Virtqueue {
    
    pub fn hjs(queue_size: u16) -> usize {
        let frr = core::mem::size_of::<VirtqDesc>() * queue_size as usize;
        let jyq = 6 + 2 * queue_size as usize; 
        let used_size = 6 + 8 * queue_size as usize;  
        
        
        let cfz = frr;
        let bps = ((cfz + jyq) + 4095) & !4095; 
        
        bps + used_size
    }
    
    
    pub fn alloc_desc(&mut self) -> Option<u16> {
        if self.num_free == 0 {
            return None;
        }
        
        let idx = self.free_head;
        self.free_head = self.free_list[idx as usize];
        self.num_free -= 1;
        Some(idx)
    }
    
    
    pub fn free_desc(&mut self, idx: u16) {
        self.free_list[idx as usize] = self.free_head;
        self.free_head = idx;
        self.num_free += 1;
    }
    
    
    pub unsafe fn add_available(&mut self, su: u16) {
        let avail = &mut *self.avail;
        let cpm = (self.avail as *mut u8).add(4) as *mut u16;
        let idx = avail.idx;
        *cpm.add((idx % self.size) as usize) = su;
        
        
        core::sync::atomic::fence(Ordering::Release);
        
        avail.idx = idx.wrapping_add(1);
    }


    pub fn new(size: u16) -> Result<Box<Self>, &'static str> {
        use alloc::alloc::{alloc_zeroed, Layout};
        
        let total_size = Self::hjs(size);
        let layout = Layout::from_size_align(total_size, 4096)
            .map_err(|_| "Invalid layout")?;
        
        let ptr = unsafe { alloc_zeroed(layout) };
        if ptr.is_null() {
            return Err("Failed to allocate virtqueue");
        }
        
        let phys_addr = ptr as u64; 
        
        
        let frr = core::mem::size_of::<VirtqDesc>() * size as usize;
        let cfz = frr;
        let bps = ((cfz + 6 + 2 * size as usize) + 4095) & !4095;
        
        let desc = ptr as *mut VirtqDesc;
        let avail = unsafe { ptr.add(cfz) as *mut Kx };
        let used = unsafe { ptr.add(bps) as *mut Ky };
        
        
        let mut free_list = alloc::vec::Vec::with_capacity(size as usize);
        for i in 0..size {
            free_list.push(i + 1);
        }
        if size > 0 {
            free_list[size as usize - 1] = 0;
        }
        
        Ok(Box::new(Self {
            size,
            phys_addr,
            desc,
            avail,
            used,
            last_used_idx: 0,
            free_head: 0,
            num_free: size,
            free_list,
        }))
    }
    
    
    pub fn set_desc(&mut self, idx: u16, addr: u64, len: u32, flags: u16, next: u16) {
        unsafe {
            let desc = &mut *self.desc.add(idx as usize);
            desc.addr = addr;
            desc.len = len;
            desc.flags = flags;
            desc.next = next;
        }
    }
    
    
    pub fn submit(&mut self, su: u16) {
        unsafe { self.add_available(su) }
    }
    
    
    pub fn has_used(&self) -> bool {
        unsafe { 
            let used = &*self.used;
            core::sync::atomic::fence(Ordering::Acquire);
            used.idx != self.last_used_idx
        }
    }
    
    
    pub fn pop_used(&mut self) -> Option<(u32, u32)> {
        unsafe {
            let used = &*self.used;
            core::sync::atomic::fence(Ordering::Acquire);
            
            if used.idx == self.last_used_idx {
                return None;
            }
            
            let cpm = (self.used as *mut u8).add(4) as *mut Qv;
            let cit = *cpm.add((self.last_used_idx % self.size) as usize);
            self.last_used_idx = self.last_used_idx.wrapping_add(1);
            
            Some((cit.id, cit.len))
        }
    }
}


pub struct VirtioDevice {
    
    pub iobase: u16,
    
    pub device_features: u32,
    
    pub driver_features: u32,
}

impl VirtioDevice {
    
    pub fn new(iobase: u16) -> Self {
        Self {
            iobase,
            device_features: 0,
            driver_features: 0,
        }
    }
    
    
    pub fn read_status(&self) -> u8 {
        unsafe {
            let mut port = Port::<u8>::new(self.iobase + legacy_reg::ES_);
            port.read()
        }
    }
    
    
    pub fn write_status(&mut self, status: u8) {
        unsafe {
            let mut port = Port::<u8>::new(self.iobase + legacy_reg::ES_);
            port.write(status);
        }
    }
    
    
    pub fn add_status(&mut self, bits: u8) {
        let current = self.read_status();
        self.write_status(current | bits);
    }
    
    
    pub fn reset(&mut self) {
        self.write_status(0);
    }
    
    
    pub fn read_device_features(&mut self) -> u32 {
        unsafe {
            let mut port = Port::<u32>::new(self.iobase + legacy_reg::BUG_);
            self.device_features = port.read();
            self.device_features
        }
    }
    
    
    pub fn write_driver_features(&mut self, features: u32) {
        self.driver_features = features;
        unsafe {
            let mut port = Port::<u32>::new(self.iobase + legacy_reg::BVA_);
            port.write(features);
        }
    }
    
    
    pub fn select_queue(&mut self, queue: u16) {
        unsafe {
            let mut port = Port::<u16>::new(self.iobase + legacy_reg::AIO_);
            port.write(queue);
        }
    }
    
    
    pub fn get_queue_size(&self) -> u16 {
        unsafe {
            let mut port = Port::<u16>::new(self.iobase + legacy_reg::XV_);
            port.read()
        }
    }
    
    
    pub fn set_queue_address(&mut self, bog: u32) {
        unsafe {
            let mut port = Port::<u32>::new(self.iobase + legacy_reg::CQE_);
            port.write(bog);
        }
    }
    
    
    pub fn notify_queue(&self, queue: u16) {
        unsafe {
            let mut port = Port::<u16>::new(self.iobase + legacy_reg::CQK_);
            port.write(queue);
        }
    }
    
    
    pub fn qsh(&self) -> u8 {
        unsafe {
            let mut port = Port::<u8>::new(self.iobase + legacy_reg::CFZ_);
            port.read()
        }
    }
    
    
    pub fn read_config8(&self, offset: u16) -> u8 {
        unsafe {
            let mut port = Port::<u8>::new(self.iobase + legacy_reg::AHQ_ + offset);
            port.read()
        }
    }
    
    
    pub fn read_config16(&self, offset: u16) -> u16 {
        unsafe {
            let mut port = Port::<u16>::new(self.iobase + legacy_reg::AHQ_ + offset);
            port.read()
        }
    }
    
    
    pub fn read_config32(&self, offset: u16) -> u32 {
        unsafe {
            let mut port = Port::<u32>::new(self.iobase + legacy_reg::AHQ_ + offset);
            port.read()
        }
    }
}
