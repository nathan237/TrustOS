//! VirtIO Block Device Emulation
//!
//! Implements a VirtIO legacy block device (device type 2) with RAM-backed storage.
//! The guest communicates through VirtIO I/O port registers (BAR0 at 0xC040).
//!
//! VirtIO Legacy I/O Layout (BAR0 base + offset):
//!   +0x00: Device Features (RO, 32-bit) — features offered by device
//!   +0x04: Guest Features (RW, 32-bit) — features accepted by guest
//!   +0x08: Queue Address (RW, 32-bit) — PFN of virtqueue
//!   +0x0C: Queue Size (RO, 16-bit) — max entries in virtqueue
//!   +0x0E: Queue Select (RW, 16-bit) — which queue to configure
//!   +0x10: Queue Notify (WO, 16-bit) — notify device about new buffers
//!   +0x12: Device Status (RW, 8-bit) — driver status flags
//!   +0x13: ISR Status (RO, 8-bit) — interrupt status
//!   +0x14+: Device-specific config (block device: capacity, etc.)
//!
//! Block Device Config (at BAR0 + 0x14):
//!   +0x00: capacity (u64) — total sectors (512 bytes each)
//!   +0x08: size_max (u32) — max segment size
//!   +0x0C: seg_max (u32) — max segments per request
//!
//! VirtIO Block Request Format:
//!   struct virtio_blk_req {
//!       type: u32,    // 0=IN(read), 1=OUT(write), 4=GET_ID
//!       reserved: u32,
//!       sector: u64,
//!       data: [u8],   // length varies
//!       status: u8,   // 0=OK, 1=IOERR, 2=UNSUPPORTED
//!   }
//!
//! Reference: VirtIO Specification v1.1, Section 5.2

use alloc::vec::Vec;

/// VirtIO block request types
pub mod req_type {
    pub const VIRTIO_BLK_T_IN: u32 = 0;     // Read
    pub const VIRTIO_BLK_T_OUT: u32 = 1;    // Write
    pub const VIRTIO_BLK_T_FLUSH: u32 = 4;  // Flush
    pub const VIRTIO_BLK_T_GET_ID: u32 = 8; // Get device ID
}

/// VirtIO block status codes
pub mod status {
    pub const VIRTIO_BLK_S_OK: u8 = 0;
    pub const VIRTIO_BLK_S_IOERR: u8 = 1;
    pub const VIRTIO_BLK_S_UNSUPP: u8 = 2;
}

/// VirtIO device status bits
pub mod device_status {
    pub const ACKNOWLEDGE: u8 = 1;
    pub const DRIVER: u8 = 2;
    pub const DRIVER_OK: u8 = 4;
    pub const FEATURES_OK: u8 = 8;
    pub const DEVICE_NEEDS_RESET: u8 = 64;
    pub const FAILED: u8 = 128;
}

/// VirtIO feature bits for block device
pub mod features {
    pub const VIRTIO_BLK_F_SIZE_MAX: u32 = 1 << 1;
    pub const VIRTIO_BLK_F_SEG_MAX: u32 = 1 << 2;
    pub const VIRTIO_BLK_F_GEOMETRY: u32 = 1 << 4;
    pub const VIRTIO_BLK_F_RO: u32 = 1 << 5;
    pub const VIRTIO_BLK_F_BLK_SIZE: u32 = 1 << 6;
    pub const VIRTIO_BLK_F_FLUSH: u32 = 1 << 9;
}

/// Sector size in bytes
pub const SECTOR_SIZE: usize = 512;

/// Virtqueue descriptor
#[derive(Debug, Clone, Copy, Default)]
#[repr(C)]
pub struct VirtqDesc {
    /// Guest physical address of the buffer
    pub addr: u64,
    /// Length of the buffer
    pub len: u32,
    /// Descriptor flags (NEXT=1, WRITE=2, INDIRECT=4)
    pub flags: u16,
    /// Next descriptor index (if NEXT flag set)
    pub next: u16,
}

/// VirtIO block device state
#[derive(Debug, Clone)]
pub struct VirtioBlkState {
    /// Device features offered
    pub device_features: u32,
    /// Guest-accepted features
    pub guest_features: u32,
    /// Currently selected queue (0 = requestq)
    pub queue_select: u16,
    /// Queue 0 (requestq) PFN address
    pub queue_pfn: u32,
    /// Queue size (max descriptors)
    pub queue_size: u16,
    /// Device status register
    pub device_status: u8,
    /// ISR status (bit 0 = used buffer notification, bit 1 = config change)
    pub isr_status: u8,
    /// Block device capacity in sectors
    pub capacity_sectors: u64,
    /// Queue last seen available index
    pub last_avail_idx: u16,
}

impl Default for VirtioBlkState {
    fn default() -> Self {
        Self {
            device_features: features::VIRTIO_BLK_F_SIZE_MAX
                           | features::VIRTIO_BLK_F_SEG_MAX
                           | features::VIRTIO_BLK_F_FLUSH,
            guest_features: 0,
            queue_select: 0,
            queue_pfn: 0,
            queue_size: 128, // Max 128 descriptors
            device_status: 0,
            isr_status: 0,
            capacity_sectors: 64, // 64 sectors = 32KB default (overridden by VM)
            last_avail_idx: 0,
        }
    }
}

impl VirtioBlkState {
    /// Create with specific capacity
    pub fn with_capacity(storage_bytes: usize) -> Self {
        let mut state = Self::default();
        state.capacity_sectors = (storage_bytes / SECTOR_SIZE) as u64;
        state
    }
    
    /// Handle I/O read from VirtIO block device port space
    /// `offset` is relative to BAR0 base (0xC040)
    pub fn io_read(&mut self, offset: u16) -> u32 {
        match offset {
            // Device features
            0x00 => self.device_features,
            // Guest features
            0x04 => self.guest_features,
            // Queue PFN
            0x08 => self.queue_pfn,
            // Queue size
            0x0C => self.queue_size as u32,
            // Queue select
            0x0E => self.queue_select as u32,
            // Device status
            0x12 => self.device_status as u32,
            // ISR status (reading clears it)
            0x13 => {
                let val = self.isr_status as u32;
                self.isr_status = 0;
                val
            }
            // Device-specific: capacity low 32 bits
            0x14 => (self.capacity_sectors & 0xFFFF_FFFF) as u32,
            // Device-specific: capacity high 32 bits
            0x18 => ((self.capacity_sectors >> 32) & 0xFFFF_FFFF) as u32,
            // Device-specific: size_max
            0x1C => 0x1000, // 4KB max segment
            // Device-specific: seg_max
            0x20 => 128,   // Max 128 segments
            _ => 0,
        }
    }
    
    /// Handle I/O write to VirtIO block device port space
    /// `offset` is relative to BAR0 base (0xC040)
    /// Returns true if a queue notification was received (needs processing)
    pub fn io_write(&mut self, offset: u16, value: u32) -> bool {
        match offset {
            // Guest features
            0x04 => {
                self.guest_features = value;
            }
            // Queue PFN
            0x08 => {
                self.queue_pfn = value;
            }
            // Queue select
            0x0E => {
                self.queue_select = value as u16;
            }
            // Queue notify
            0x10 => {
                // Guest notified us about new buffers
                return true;
            }
            // Device status
            0x12 => {
                self.device_status = value as u8;
                if value == 0 {
                    // Reset
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
    
    /// Process pending virtqueue requests.
    /// Reads descriptors from guest memory, performs block I/O, writes back status.
    /// Returns the number of requests processed.
    pub fn process_queue(&mut self, guest_memory: &mut [u8], storage: &mut [u8]) -> usize {
        if self.queue_pfn == 0 || self.device_status & device_status::DRIVER_OK == 0 {
            return 0;
        }
        
        let queue_addr = (self.queue_pfn as u64) * 4096; // PFN to byte address
        let queue_size = self.queue_size as usize;
        
        // Virtqueue layout:
        // Descriptors: queue_addr + 0, each 16 bytes
        // Available ring: after descriptors, aligned to 2
        // Used ring: after available ring, aligned to 4096
        let desc_base = queue_addr as usize;
        let avail_base = desc_base + queue_size * 16;
        let avail_base_aligned = (avail_base + 1) & !1; // Align to 2
        let used_base = avail_base_aligned + 4 + queue_size * 2 + 2; // flags + ring + used_event
        let used_base_aligned = (used_base + 4095) & !4095; // Align to 4096
        
        // Read available ring index
        if avail_base_aligned + 2 >= guest_memory.len() {
            return 0;
        }
        let avail_idx = u16::from_le_bytes([
            guest_memory[avail_base_aligned + 2],
            guest_memory[avail_base_aligned + 3],
        ]);
        
        let mut processed = 0usize;
        
        while self.last_avail_idx != avail_idx {
            let ring_idx = (self.last_avail_idx as usize) % queue_size;
            let ring_offset = avail_base_aligned + 4 + ring_idx * 2;
            
            if ring_offset + 2 > guest_memory.len() {
                break;
            }
            
            let desc_idx = u16::from_le_bytes([
                guest_memory[ring_offset],
                guest_memory[ring_offset + 1],
            ]) as usize;
            
            // Process the descriptor chain
            let used_len = self.process_request(guest_memory, storage, desc_base, desc_idx, queue_size);
            
            // Write to used ring
            if used_base_aligned + 4 >= guest_memory.len() {
                break;
            }
            let used_idx = u16::from_le_bytes([
                guest_memory[used_base_aligned + 2],
                guest_memory[used_base_aligned + 3],
            ]);
            let used_ring_entry = used_base_aligned + 4 + (used_idx as usize % queue_size) * 8;
            
            if used_ring_entry + 8 <= guest_memory.len() {
                // Write used ring entry: { id: u32, len: u32 }
                let id_bytes = (desc_idx as u32).to_le_bytes();
                let len_bytes = (used_len as u32).to_le_bytes();
                guest_memory[used_ring_entry..used_ring_entry + 4].copy_from_slice(&id_bytes);
                guest_memory[used_ring_entry + 4..used_ring_entry + 8].copy_from_slice(&len_bytes);
                
                // Increment used index
                let new_used_idx = used_idx.wrapping_add(1);
                let bytes = new_used_idx.to_le_bytes();
                guest_memory[used_base_aligned + 2] = bytes[0];
                guest_memory[used_base_aligned + 3] = bytes[1];
            }
            
            self.last_avail_idx = self.last_avail_idx.wrapping_add(1);
            processed += 1;
            
            // Set ISR bit for used buffer notification
            self.isr_status |= 1;
        }
        
        processed
    }
    
    /// Process a single request from the descriptor chain
    fn process_request(
        &self,
        guest_memory: &mut [u8],
        storage: &mut [u8],
        desc_base: usize,
        first_desc: usize,
        queue_size: usize,
    ) -> usize {
        // Read the first descriptor (request header: type + sector)
        let header = self.read_desc(guest_memory, desc_base, first_desc);
        if header.addr as usize + 16 > guest_memory.len() {
            return 0;
        }
        
        // Parse request header
        let req_type = u32::from_le_bytes([
            guest_memory[header.addr as usize],
            guest_memory[header.addr as usize + 1],
            guest_memory[header.addr as usize + 2],
            guest_memory[header.addr as usize + 3],
        ]);
        let sector = u64::from_le_bytes([
            guest_memory[header.addr as usize + 8],
            guest_memory[header.addr as usize + 9],
            guest_memory[header.addr as usize + 10],
            guest_memory[header.addr as usize + 11],
            guest_memory[header.addr as usize + 12],
            guest_memory[header.addr as usize + 13],
            guest_memory[header.addr as usize + 14],
            guest_memory[header.addr as usize + 15],
        ]);
        
        // Follow chain: header → data descriptor(s) → status descriptor
        let mut total_len = 0usize;
        let mut current = first_desc;
        let mut data_descs: [(u64, u32, u16); 16] = [(0, 0, 0); 16];
        let mut data_count = 0usize;
        let mut status_desc: Option<(u64, u32)> = None;
        let mut is_first = true;
        
        loop {
            let desc = self.read_desc(guest_memory, desc_base, current);
            
            if is_first {
                is_first = false;
            } else if desc.flags & 2 != 0 {
                // WRITE flag = device writes to this buffer
                // Could be data (for read requests) or status
                if desc.len == 1 {
                    status_desc = Some((desc.addr, desc.len));
                } else {
                    if data_count < 16 {
                        data_descs[data_count] = (desc.addr, desc.len, desc.flags);
                        data_count += 1;
                    }
                }
            } else {
                // No WRITE flag = device reads from this buffer (data for write requests)
                if data_count < 16 {
                    data_descs[data_count] = (desc.addr, desc.len, desc.flags);
                    data_count += 1;
                }
            }
            
            if desc.flags & 1 == 0 {
                // No NEXT flag — end of chain
                // If we didn't find a status desc yet, the last one is status
                if status_desc.is_none() && desc.len == 1 {
                    status_desc = Some((desc.addr, desc.len));
                    if data_count > 0 {
                        data_count -= 1; // Remove it from data list
                    }
                }
                break;
            }
            
            current = desc.next as usize;
            if current >= queue_size {
                break;
            }
        }
        
        // Execute the request
        let result_status = match req_type {
            req_type::VIRTIO_BLK_T_IN => {
                // Read from storage into guest memory
                let mut offset = sector as usize * SECTOR_SIZE;
                for i in 0..data_count {
                    let (addr, len, _) = data_descs[i];
                    let addr = addr as usize;
                    let len = len as usize;
                    if offset + len <= storage.len() && addr + len <= guest_memory.len() {
                        guest_memory[addr..addr + len].copy_from_slice(&storage[offset..offset + len]);
                        total_len += len;
                    }
                    offset += len;
                }
                status::VIRTIO_BLK_S_OK
            }
            req_type::VIRTIO_BLK_T_OUT => {
                // Write from guest memory into storage
                let mut offset = sector as usize * SECTOR_SIZE;
                for i in 0..data_count {
                    let (addr, len, _) = data_descs[i];
                    let addr = addr as usize;
                    let len = len as usize;
                    if offset + len <= storage.len() && addr + len <= guest_memory.len() {
                        storage[offset..offset + len].copy_from_slice(&guest_memory[addr..addr + len]);
                        total_len += len;
                    }
                    offset += len;
                }
                status::VIRTIO_BLK_S_OK
            }
            req_type::VIRTIO_BLK_T_FLUSH => {
                // RAM-backed, flush is a no-op
                status::VIRTIO_BLK_S_OK
            }
            req_type::VIRTIO_BLK_T_GET_ID => {
                // Write device ID string
                let id_str = b"trustos-virtio-blk\0";
                if let Some((addr, _, _)) = data_descs.first() {
                    let addr = *addr as usize;
                    let copy_len = id_str.len().min(20);
                    if addr + copy_len <= guest_memory.len() {
                        guest_memory[addr..addr + copy_len].copy_from_slice(&id_str[..copy_len]);
                        total_len += copy_len;
                    }
                }
                status::VIRTIO_BLK_S_OK
            }
            _ => status::VIRTIO_BLK_S_UNSUPP,
        };
        
        // Write status byte
        if let Some((addr, _)) = status_desc {
            let addr = addr as usize;
            if addr < guest_memory.len() {
                guest_memory[addr] = result_status;
                total_len += 1;
            }
        }
        
        total_len
    }
    
    /// Read a virtqueue descriptor from guest memory
    fn read_desc(&self, guest_memory: &[u8], desc_base: usize, index: usize) -> VirtqDesc {
        let offset = desc_base + index * 16;
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

/// VirtIO Console Device State (VirtIO legacy I/O transport)
///
/// Port space layout at BAR0 (0xC000):
///   +0x00: Device Features (RO) — offered features
///   +0x04: Guest Features (RW) — accepted features
///   +0x08: Queue PFN (RW) — virtqueue physical address
///   +0x0C: Queue Size (RO) — max entries
///   +0x0E: Queue Select (RW)
///   +0x10: Queue Notify (WO)
///   +0x12: Device Status (RW)
///   +0x13: ISR Status (RO, clears on read)
///   +0x14: cols (u16) — console columns
///   +0x16: rows (u16) — console rows
///   +0x18: max_nr_ports (u32)
///   +0x1C: emerg_wr (u32) — emergency write port
#[derive(Debug, Clone)]
pub struct VirtioConsoleState {
    /// Device features
    pub device_features: u32,
    /// Guest features
    pub guest_features: u32,
    /// Queue select
    pub queue_select: u16,
    /// Queue 0 (receiveq) PFN
    pub queue_pfn_0: u32,
    /// Queue 1 (transmitq) PFN
    pub queue_pfn_1: u32,
    /// Queue size
    pub queue_size: u16,
    /// Device status
    pub device_status: u8,
    /// ISR status
    pub isr_status: u8,
    /// Console columns
    pub cols: u16,
    /// Console rows
    pub rows: u16,
    /// Maximum number of ports
    pub max_nr_ports: u32,
    /// Transmitq last available index
    pub tx_last_avail_idx: u16,
}

impl Default for VirtioConsoleState {
    fn default() -> Self {
        Self {
            device_features: 0, // Simple console, no multiport
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
    /// Handle I/O read from VirtIO console port space
    /// `offset` is relative to BAR0 base (0xC000)
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
            // Console config
            0x14 => self.cols as u32,
            0x16 => self.rows as u32,
            0x18 => self.max_nr_ports,
            _ => 0,
        }
    }
    
    /// Handle I/O write to VirtIO console port space
    /// Returns true if transmitq was notified (data to extract)
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
                // Queue notify — check which queue
                let queue_idx = value as u16;
                if queue_idx == 1 {
                    // Transmitq notification — guest sent data
                    return true;
                }
            }
            0x12 => {
                self.device_status = value as u8;
                if value == 0 {
                    // Reset
                    self.guest_features = 0;
                    self.queue_pfn_0 = 0;
                    self.queue_pfn_1 = 0;
                    self.queue_select = 0;
                    self.isr_status = 0;
                    self.tx_last_avail_idx = 0;
                }
            }
            // Emergency write — guest writes a character directly
            0x1C => {
                let ch = (value & 0xFF) as u8;
                crate::serial_print!("{}", ch as char);
            }
            _ => {}
        }
        false
    }
    
    /// Process transmitq: read data from guest and output to serial.
    /// Returns bytes read from guest transmit queue.
    pub fn process_transmitq(&mut self, guest_memory: &[u8]) -> usize {
        if self.queue_pfn_1 == 0 || self.device_status & device_status::DRIVER_OK == 0 {
            return 0;
        }
        
        let queue_addr = (self.queue_pfn_1 as u64) * 4096;
        let queue_size = self.queue_size as usize;
        
        let desc_base = queue_addr as usize;
        let avail_base = desc_base + queue_size * 16;
        let avail_base_aligned = (avail_base + 1) & !1;
        
        if avail_base_aligned + 4 > guest_memory.len() {
            return 0;
        }
        
        let avail_idx = u16::from_le_bytes([
            guest_memory[avail_base_aligned + 2],
            guest_memory[avail_base_aligned + 3],
        ]);
        
        let mut total_bytes = 0usize;
        
        while self.tx_last_avail_idx != avail_idx {
            let ring_idx = (self.tx_last_avail_idx as usize) % queue_size;
            let ring_offset = avail_base_aligned + 4 + ring_idx * 2;
            
            if ring_offset + 2 > guest_memory.len() {
                break;
            }
            
            let desc_idx = u16::from_le_bytes([
                guest_memory[ring_offset],
                guest_memory[ring_offset + 1],
            ]) as usize;
            
            // Read descriptor and output data
            let offset = desc_base + desc_idx * 16;
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
                
                // Output each byte to serial
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
