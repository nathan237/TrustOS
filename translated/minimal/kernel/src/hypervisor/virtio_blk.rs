































use alloc::vec::Vec;


pub mod req_type {
    pub const DEK_: u32 = 0;     
    pub const DEL_: u32 = 1;    
    pub const DEI_: u32 = 4;  
    pub const DEJ_: u32 = 8; 
}


pub mod status {
    pub const ZM_: u8 = 0;
    pub const ENN_: u8 = 1;
    pub const DEH_: u8 = 2;
}


pub mod device_status {
    pub const Gf: u8 = 1;
    pub const Cl: u8 = 2;
    pub const IQ_: u8 = 4;
    pub const NY_: u8 = 8;
    pub const BUH_: u8 = 64;
    pub const Sa: u8 = 128;
}


pub mod features {
    pub const DEG_: u32 = 1 << 1;
    pub const DEF_: u32 = 1 << 2;
    pub const ENL_: u32 = 1 << 4;
    pub const ENM_: u32 = 1 << 5;
    pub const ENK_: u32 = 1 << 6;
    pub const DEE_: u32 = 1 << 9;
}


pub const H_: usize = 512;


#[derive(Debug, Clone, Copy, Default)]
#[repr(C)]
pub struct VirtqDesc {
    
    pub addr: u64,
    
    pub len: u32,
    
    pub flags: u16,
    
    pub next: u16,
}


#[derive(Debug, Clone)]
pub struct VirtioBlkState {
    
    pub device_features: u32,
    
    pub guest_features: u32,
    
    pub queue_select: u16,
    
    pub queue_pfn: u32,
    
    pub queue_size: u16,
    
    pub device_status: u8,
    
    pub isr_status: u8,
    
    pub capacity_sectors: u64,
    
    pub last_avail_idx: u16,
}

impl Default for VirtioBlkState {
    fn default() -> Self {
        Self {
            device_features: features::DEG_
                           | features::DEF_
                           | features::DEE_,
            guest_features: 0,
            queue_select: 0,
            queue_pfn: 0,
            queue_size: 128, 
            device_status: 0,
            isr_status: 0,
            capacity_sectors: 64, 
            last_avail_idx: 0,
        }
    }
}

impl VirtioBlkState {
    
    pub fn with_capacity(storage_bytes: usize) -> Self {
        let mut state = Self::default();
        state.capacity_sectors = (storage_bytes / H_) as u64;
        state
    }
    
    
    
    pub fn io_read(&mut self, offset: u16) -> u32 {
        match offset {
            
            0x00 => self.device_features,
            
            0x04 => self.guest_features,
            
            0x08 => self.queue_pfn,
            
            0x0C => self.queue_size as u32,
            
            0x0E => self.queue_select as u32,
            
            0x12 => self.device_status as u32,
            
            0x13 => {
                let val = self.isr_status as u32;
                self.isr_status = 0;
                val
            }
            
            0x14 => (self.capacity_sectors & 0xFFFF_FFFF) as u32,
            
            0x18 => ((self.capacity_sectors >> 32) & 0xFFFF_FFFF) as u32,
            
            0x1C => 0x1000, 
            
            0x20 => 128,   
            _ => 0,
        }
    }
    
    
    
    
    pub fn io_write(&mut self, offset: u16, value: u32) -> bool {
        match offset {
            
            0x04 => {
                self.guest_features = value;
            }
            
            0x08 => {
                self.queue_pfn = value;
            }
            
            0x0E => {
                self.queue_select = value as u16;
            }
            
            0x10 => {
                
                return true;
            }
            
            0x12 => {
                self.device_status = value as u8;
                if value == 0 {
                    
                    self.guest_features = 0;
                    self.queue_pfn = 0;
                    self.queue_select = 0;
                    self.isr_status = 0;
                    self.last_avail_idx = 0;
                }
            }
            _ => {}
        }
        false
    }
    
    
    
    
    pub fn process_queue(&mut self, guest_memory: &mut [u8], storage: &mut [u8]) -> usize {
        if self.queue_pfn == 0 || self.device_status & device_status::IQ_ == 0 {
            return 0;
        }
        
        let gpg = (self.queue_pfn as u64) * 4096; 
        let queue_size = self.queue_size as usize;
        
        
        
        
        
        let bls = gpg as usize;
        let fhr = bls + queue_size * 16;
        let bkt = (fhr + 1) & !1; 
        let pqm = bkt + 4 + queue_size * 2 + 2; 
        let dge = (pqm + 4095) & !4095; 
        
        
        if bkt + 2 >= guest_memory.len() {
            return 0;
        }
        let fht = u16::from_le_bytes([
            guest_memory[bkt + 2],
            guest_memory[bkt + 3],
        ]);
        
        let mut iwv = 0usize;
        
        while self.last_avail_idx != fht {
            let eys = (self.last_avail_idx as usize) % queue_size;
            let cpl = bkt + 4 + eys * 2;
            
            if cpl + 2 > guest_memory.len() {
                break;
            }
            
            let tx = u16::from_le_bytes([
                guest_memory[cpl],
                guest_memory[cpl + 1],
            ]) as usize;
            
            
            let pqo = self.process_request(guest_memory, storage, bls, tx, queue_size);
            
            
            if dge + 4 >= guest_memory.len() {
                break;
            }
            let jpp = u16::from_le_bytes([
                guest_memory[dge + 2],
                guest_memory[dge + 3],
            ]);
            let edl = dge + 4 + (jpp as usize % queue_size) * 8;
            
            if edl + 8 <= guest_memory.len() {
                
                let mnj = (tx as u32).to_le_bytes();
                let mxv = (pqo as u32).to_le_bytes();
                guest_memory[edl..edl + 4].copy_from_slice(&mnj);
                guest_memory[edl + 4..edl + 8].copy_from_slice(&mxv);
                
                
                let nju = jpp.wrapping_add(1);
                let bytes = nju.to_le_bytes();
                guest_memory[dge + 2] = bytes[0];
                guest_memory[dge + 3] = bytes[1];
            }
            
            self.last_avail_idx = self.last_avail_idx.wrapping_add(1);
            iwv += 1;
            
            
            self.isr_status |= 1;
        }
        
        iwv
    }
    
    
    fn process_request(
        &self,
        guest_memory: &mut [u8],
        storage: &mut [u8],
        bls: usize,
        first_desc: usize,
        queue_size: usize,
    ) -> usize {
        
        let header = self.read_desc(guest_memory, bls, first_desc);
        if header.addr as usize + 16 > guest_memory.len() {
            return 0;
        }
        
        
        let req_type = u32::from_le_bytes([
            guest_memory[header.addr as usize],
            guest_memory[header.addr as usize + 1],
            guest_memory[header.addr as usize + 2],
            guest_memory[header.addr as usize + 3],
        ]);
        let dj = u64::from_le_bytes([
            guest_memory[header.addr as usize + 8],
            guest_memory[header.addr as usize + 9],
            guest_memory[header.addr as usize + 10],
            guest_memory[header.addr as usize + 11],
            guest_memory[header.addr as usize + 12],
            guest_memory[header.addr as usize + 13],
            guest_memory[header.addr as usize + 14],
            guest_memory[header.addr as usize + 15],
        ]);
        
        
        let mut total_len = 0usize;
        let mut current = first_desc;
        let mut dmh: [(u64, u32, u16); 16] = [(0, 0, 0); 16];
        let mut blp = 0usize;
        let mut ave: Option<(u64, u32)> = None;
        let mut ihx = true;
        
        loop {
            let desc = self.read_desc(guest_memory, bls, current);
            
            if ihx {
                ihx = false;
            } else if desc.flags & 2 != 0 {
                
                
                if desc.len == 1 {
                    ave = Some((desc.addr, desc.len));
                } else {
                    if blp < 16 {
                        dmh[blp] = (desc.addr, desc.len, desc.flags);
                        blp += 1;
                    }
                }
            } else {
                
                if blp < 16 {
                    dmh[blp] = (desc.addr, desc.len, desc.flags);
                    blp += 1;
                }
            }
            
            if desc.flags & 1 == 0 {
                
                
                if ave.is_none() && desc.len == 1 {
                    ave = Some((desc.addr, desc.len));
                    if blp > 0 {
                        blp -= 1; 
                    }
                }
                break;
            }
            
            current = desc.next as usize;
            if current >= queue_size {
                break;
            }
        }
        
        
        let ogp = match req_type {
            req_type::DEK_ => {
                
                let mut offset = dj as usize * H_;
                for i in 0..blp {
                    let (addr, len, _) = dmh[i];
                    let addr = addr as usize;
                    let len = len as usize;
                    if offset + len <= storage.len() && addr + len <= guest_memory.len() {
                        guest_memory[addr..addr + len].copy_from_slice(&storage[offset..offset + len]);
                        total_len += len;
                    }
                    offset += len;
                }
                status::ZM_
            }
            req_type::DEL_ => {
                
                let mut offset = dj as usize * H_;
                for i in 0..blp {
                    let (addr, len, _) = dmh[i];
                    let addr = addr as usize;
                    let len = len as usize;
                    if offset + len <= storage.len() && addr + len <= guest_memory.len() {
                        storage[offset..offset + len].copy_from_slice(&guest_memory[addr..addr + len]);
                        total_len += len;
                    }
                    offset += len;
                }
                status::ZM_
            }
            req_type::DEI_ => {
                
                status::ZM_
            }
            req_type::DEJ_ => {
                
                let eqa = b"trustos-virtio-blk\0";
                if let Some((addr, _, _)) = dmh.first() {
                    let addr = *addr as usize;
                    let mb = eqa.len().min(20);
                    if addr + mb <= guest_memory.len() {
                        guest_memory[addr..addr + mb].copy_from_slice(&eqa[..mb]);
                        total_len += mb;
                    }
                }
                status::ZM_
            }
            _ => status::DEH_,
        };
        
        
        if let Some((addr, _)) = ave {
            let addr = addr as usize;
            if addr < guest_memory.len() {
                guest_memory[addr] = ogp;
                total_len += 1;
            }
        }
        
        total_len
    }
    
    
    fn read_desc(&self, guest_memory: &[u8], bls: usize, index: usize) -> VirtqDesc {
        let offset = bls + index * 16;
        if offset + 16 > guest_memory.len() {
            return VirtqDesc::default();
        }
        
        VirtqDesc {
            addr: u64::from_le_bytes([
                guest_memory[offset], guest_memory[offset + 1],
                guest_memory[offset + 2], guest_memory[offset + 3],
                guest_memory[offset + 4], guest_memory[offset + 5],
                guest_memory[offset + 6], guest_memory[offset + 7],
            ]),
            len: u32::from_le_bytes([
                guest_memory[offset + 8], guest_memory[offset + 9],
                guest_memory[offset + 10], guest_memory[offset + 11],
            ]),
            flags: u16::from_le_bytes([
                guest_memory[offset + 12], guest_memory[offset + 13],
            ]),
            next: u16::from_le_bytes([
                guest_memory[offset + 14], guest_memory[offset + 15],
            ]),
        }
    }
}
















#[derive(Debug, Clone)]
pub struct VirtioConsoleState {
    
    pub device_features: u32,
    
    pub guest_features: u32,
    
    pub queue_select: u16,
    
    pub queue_pfn_0: u32,
    
    pub queue_pfn_1: u32,
    
    pub queue_size: u16,
    
    pub device_status: u8,
    
    pub isr_status: u8,
    
    pub cols: u16,
    
    pub rows: u16,
    
    pub max_nr_ports: u32,
    
    pub tx_last_avail_idx: u16,
}

impl Default for VirtioConsoleState {
    fn default() -> Self {
        Self {
            device_features: 0, 
            guest_features: 0,
            queue_select: 0,
            queue_pfn_0: 0,
            queue_pfn_1: 0,
            queue_size: 64,
            device_status: 0,
            isr_status: 0,
            cols: 80,
            rows: 25,
            max_nr_ports: 1,
            tx_last_avail_idx: 0,
        }
    }
}

impl VirtioConsoleState {
    
    
    pub fn io_read(&mut self, offset: u16) -> u32 {
        match offset {
            0x00 => self.device_features,
            0x04 => self.guest_features,
            0x08 => {
                match self.queue_select {
                    0 => self.queue_pfn_0,
                    1 => self.queue_pfn_1,
                    _ => 0,
                }
            }
            0x0C => self.queue_size as u32,
            0x0E => self.queue_select as u32,
            0x12 => self.device_status as u32,
            0x13 => {
                let val = self.isr_status as u32;
                self.isr_status = 0;
                val
            }
            
            0x14 => self.cols as u32,
            0x16 => self.rows as u32,
            0x18 => self.max_nr_ports,
            _ => 0,
        }
    }
    
    
    
    pub fn io_write(&mut self, offset: u16, value: u32) -> bool {
        match offset {
            0x04 => { self.guest_features = value; }
            0x08 => {
                match self.queue_select {
                    0 => self.queue_pfn_0 = value,
                    1 => self.queue_pfn_1 = value,
                    _ => {}
                }
            }
            0x0E => { self.queue_select = value as u16; }
            0x10 => {
                
                let oaq = value as u16;
                if oaq == 1 {
                    
                    return true;
                }
            }
            0x12 => {
                self.device_status = value as u8;
                if value == 0 {
                    
                    self.guest_features = 0;
                    self.queue_pfn_0 = 0;
                    self.queue_pfn_1 = 0;
                    self.queue_select = 0;
                    self.isr_status = 0;
                    self.tx_last_avail_idx = 0;
                }
            }
            
            0x1C => {
                let ch = (value & 0xFF) as u8;
                crate::serial_print!("{}", ch as char);
            }
            _ => {}
        }
        false
    }
    
    
    
    pub fn process_transmitq(&mut self, guest_memory: &[u8]) -> usize {
        if self.queue_pfn_1 == 0 || self.device_status & device_status::IQ_ == 0 {
            return 0;
        }
        
        let gpg = (self.queue_pfn_1 as u64) * 4096;
        let queue_size = self.queue_size as usize;
        
        let bls = gpg as usize;
        let fhr = bls + queue_size * 16;
        let bkt = (fhr + 1) & !1;
        
        if bkt + 4 > guest_memory.len() {
            return 0;
        }
        
        let fht = u16::from_le_bytes([
            guest_memory[bkt + 2],
            guest_memory[bkt + 3],
        ]);
        
        let mut total_bytes = 0usize;
        
        while self.tx_last_avail_idx != fht {
            let eys = (self.tx_last_avail_idx as usize) % queue_size;
            let cpl = bkt + 4 + eys * 2;
            
            if cpl + 2 > guest_memory.len() {
                break;
            }
            
            let tx = u16::from_le_bytes([
                guest_memory[cpl],
                guest_memory[cpl + 1],
            ]) as usize;
            
            
            let offset = bls + tx * 16;
            if offset + 16 <= guest_memory.len() {
                let addr = u64::from_le_bytes([
                    guest_memory[offset], guest_memory[offset + 1],
                    guest_memory[offset + 2], guest_memory[offset + 3],
                    guest_memory[offset + 4], guest_memory[offset + 5],
                    guest_memory[offset + 6], guest_memory[offset + 7],
                ]) as usize;
                let len = u32::from_le_bytes([
                    guest_memory[offset + 8], guest_memory[offset + 9],
                    guest_memory[offset + 10], guest_memory[offset + 11],
                ]) as usize;
                
                
                if addr + len <= guest_memory.len() {
                    for i in 0..len {
                        crate::serial_print!("{}", guest_memory[addr + i] as char);
                    }
                    total_bytes += len;
                }
            }
            
            self.tx_last_avail_idx = self.tx_last_avail_idx.wrapping_add(1);
            self.isr_status |= 1;
        }
        
        total_bytes
    }
}
