//! VirtIO Block Device Driver
//!
//! Driver for virtio-blk devices (QEMU, KVM, etc.)
//! Provides persistent storage through virtual block devices.

use alloc::boxed::Box;
use core::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use spin::Mutex;

use crate::virtio::{VirtioDevice, Virtqueue, desc_flags, status};
use crate::pci::PciDevice;

/// Block size (standard sector size)
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const BLOCK_SIZE: usize = 512;

/// virtio-blk feature bits
pub mod features {
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const SIZE_MAXIMUM: u32 = 1 << 1;      // Max segment size
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const SEG_MAXIMUM: u32 = 1 << 2;       // Max segments per request
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const GEOMETRY: u32 = 1 << 4;      // Disk geometry available
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const RO: u32 = 1 << 5;            // Read-only device
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const BLK_SIZE: u32 = 1 << 6;      // Block size available
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const FLUSH: u32 = 1 << 9;         // Flush command supported
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const TOPOLOGY: u32 = 1 << 10;     // Topology info available
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const CONFIG_WCE: u32 = 1 << 11;   // Write cache enable
}

/// virtio-blk request types
pub mod req_type {
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const IN: u32 = 0;       // Read
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const OUT: u32 = 1;      // Write
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const FLUSH: u32 = 4;    // Flush
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const DISCARD: u32 = 11; // Discard
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const WRITE_ZEROES: u32 = 13;
}

/// virtio-blk status codes
pub mod blk_status {
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const OK: u8 = 0;
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const IOERR: u8 = 1;
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const UNSUPP: u8 = 2;
}

/// virtio-blk request header
#[repr(C)]
// #[derive] — auto-generates trait implementations at compile time.
#[derive(Debug, Clone, Copy, Default)]
// Public structure — visible outside this module.
pub struct VirtioBlkReqHdr {
    pub req_type: u32,
    pub reserved: u32,
    pub sector: u64,
}

// Implementation block — defines methods for the type above.
impl VirtioBlkReqHdr {
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const SIZE: usize = 16;
}

/// VirtIO block device driver
pub struct VirtioBlk {
    /// Base VirtIO device
    device: VirtioDevice,
    /// Request queue (queue 0)
    queue: Option<Box<Virtqueue>>,
    /// Device capacity in sectors
    capacity: u64,
    /// Sector size
    sector_size: u32,
    /// Read-only flag
    read_only: bool,
}

/// Global driver instance
static DRIVER: Mutex<Option<VirtioBlk>> = Mutex::new(None);
// Atomic variable — provides lock-free thread-safe access.
static INITIALIZED: AtomicBool = AtomicBool::new(false);

/// VirtIO blk I/O base (for ISR access without locking DRIVER)
static VIRTIO_BLOCK_IOBASE: core::sync::atomic::AtomicU16 = core::sync::atomic::AtomicU16::new(0);
/// Request completion flag (set by ISR, polled by read/write)
static VIRTIO_BLOCK_COMPLETE: AtomicBool = AtomicBool::new(false);

/// Statistics
static READS: AtomicU64 = AtomicU64::new(0);
// Atomic variable — provides lock-free thread-safe access.
static WRITES: AtomicU64 = AtomicU64::new(0);
// Atomic variable — provides lock-free thread-safe access.
static BYTES_READ: AtomicU64 = AtomicU64::new(0);
// Atomic variable — provides lock-free thread-safe access.
static BYTES_WRITTEN: AtomicU64 = AtomicU64::new(0);

// Implementation block — defines methods for the type above.
impl VirtioBlk {
    /// Initialize the driver with a PCI device
    pub fn new(pci_dev: &PciDevice) -> Result<Self, &'static str> {
        // Get I/O base from BAR0
        let bar0 = pci_dev.bar[0];
        if bar0 == 0 {
            return Err("BAR0 not configured");
        }
        
        // Check if it's I/O space (bit 0 = 1)
        let iobase = if bar0 & 1 == 1 {
            (bar0 & 0xFFFC) as u16
        } else {
            return Err("MMIO not supported yet, need I/O port BAR");
        };
        
        crate::log_debug!("[virtio-blk] I/O base: {:#X}", iobase);
        
        let mut device = VirtioDevice::new(iobase);
        
        // Reset device
        device.reset();
        
        // Acknowledge device
        device.add_status(status::ACKNOWLEDGE);
        device.add_status(status::DRIVER);
        
        // Read features
        let device_features = device.read_device_features();
        crate::log_debug!("[virtio-blk] Device features: {:#X}", device_features);
        
        // Check if read-only
        let read_only = (device_features & features::RO) != 0;
        
        // Negotiate features
        let driver_features = 0u32; // We don't need any special features for basic I/O
        device.write_driver_features(driver_features);
        
        // Read capacity from config space (offset 0 for block devices)
        // Config space for virtio-blk starts at offset 0x14 (after legacy regs)
        let capability_lo = device.read_config32(0) as u64;
        let capability_hi = device.read_config32(4) as u64;
        let capacity = capability_lo | (capability_hi << 32);
        
        crate::log!("[virtio-blk] Capacity: {} sectors ({} MB)", 
            capacity, (capacity * 512) / (1024 * 1024));
        
        if read_only {
            crate::log!("[virtio-blk] Device is read-only");
        }
        
        Ok(Self {
            device,
            queue: None,
            capacity,
            sector_size: 512,
            read_only,
        })
    }
    
    /// Setup virtqueue
    pub fn setup_queue(&mut self) -> Result<(), &'static str> {
        self.device.select_queue(0);
        let size = self.device.get_queue_size();
        crate::log_debug!("[virtio-blk] Queue size: {}", size);
        
        if size == 0 {
            return Err("Queue not available");
        }
        
        let queue = self.alloc_queue(size)?;
        
        // Tell device where the queue is
        let pfn = (queue.phys_addr / 4096) as u32;
        self.device.set_queue_address(pfn);
        
        self.queue = Some(queue);
        Ok(())
    }
    
    /// Allocate a virtqueue
    fn alloc_queue(&self, size: u16) -> Result<Box<Virtqueue>, &'static str> {
        Virtqueue::new(size)
    }
    
    /// Start the device
    pub fn start(&mut self) -> Result<(), &'static str> {
        self.device.add_status(status::DRIVER_OK);
        crate::log!("[virtio-blk] Device started");
        Ok(())
    }
    
    /// Read sectors from device
    pub fn read_sectors(&mut self, start_sector: u64, count: usize, buffer: &mut [u8]) -> Result<(), &'static str> {
        if buffer.len() < count * BLOCK_SIZE {
            return Err("Buffer too small");
        }
        
        if start_sector + count as u64 > self.capacity {
            return Err("Read beyond device capacity");
        }
        
        // For simplicity, read one sector at a time
        for i in 0..count {
            self.read_one_sector(start_sector + i as u64, 
                &mut buffer[i * BLOCK_SIZE..(i + 1) * BLOCK_SIZE])?;
        }
        
        READS.fetch_add(count as u64, Ordering::Relaxed);
        BYTES_READ.fetch_add((count * BLOCK_SIZE) as u64, Ordering::Relaxed);
        
        Ok(())
    }
    
    /// Read a single sector
    fn read_one_sector(&mut self, sector: u64, buffer: &mut [u8]) -> Result<(), &'static str> {
        // Use heap-allocated DMA buffers to ensure proper physical address translation
        use alloc::boxed::Box;
        use alloc::vec;
        
        // Allocate DMA buffer on heap (which is in HHDM)
        // Layout: [header: 16 bytes][data: 512 bytes][status: 1 byte]
        let total_size = VirtioBlkReqHdr::SIZE + BLOCK_SIZE + 1;
        let mut dma_buf = vec![0u8; total_size].into_boxed_slice();
        
        // Write header at offset 0
        let header = VirtioBlkReqHdr {
            req_type: req_type::IN,
            reserved: 0,
            sector,
        };
                // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe {
            let header_pointer = dma_buf.as_mut_ptr() as *mut VirtioBlkReqHdr;
            core::ptr::write(header_pointer, header);
        }
        
        // Set status byte to 0xFF at the end
        dma_buf[VirtioBlkReqHdr::SIZE + BLOCK_SIZE] = 0xFF;
        
        // Get physical addresses from heap (which is in HHDM)
        let hhdm = crate::memory::hhdm_offset();
        let dma_virt = dma_buf.as_ptr() as u64;
        let dma_physical = dma_virt - hhdm;
        
        let header_physical = dma_physical;
        let data_phys = dma_physical + VirtioBlkReqHdr::SIZE as u64;
        let status_phys = dma_physical + (VirtioBlkReqHdr::SIZE + BLOCK_SIZE) as u64;
        
        let queue = self.queue.as_mut().ok_or("Queue not initialized")?;
        
        // Setup descriptor chain: header -> data -> status
        let head = queue.alloc_desc().ok_or("No free descriptor")?;
        let data_descriptor = queue.alloc_desc().ok_or("No free descriptor")?;
        let status_descriptor = queue.alloc_desc().ok_or("No free descriptor")?;
        
        // Header descriptor (device reads)
        queue.set_desc(head, header_physical, VirtioBlkReqHdr::SIZE as u32, 
            desc_flags::NEXT, data_descriptor);
        
        // Data descriptor (device writes)
        queue.set_desc(data_descriptor, data_phys, BLOCK_SIZE as u32,
            desc_flags::WRITE | desc_flags::NEXT, status_descriptor);
        
        // Status descriptor (device writes)
        queue.set_desc(status_descriptor, status_phys, 1, desc_flags::WRITE, 0);
        
        // Submit to available ring
        queue.submit(head);
        
        // Get iobase before dropping queue borrow
        let iobase = self.device.iobase;
        
        // Notify device directly
        unsafe {
            let mut port = crate::arch::Port::<u16>::new(iobase + 0x10);
            port.write(0);
        }
        
        // Wait for completion (interrupt-assisted polling)
        VIRTIO_BLOCK_COMPLETE.store(false, Ordering::Release);
        
        // Get queue reference again for polling
        let queue = self.queue.as_mut().ok_or("Queue not initialized")?;
        
        // Wait for completion — check interrupt flag or used ring
        let mut timeout = 1_000_000u32;
        while !queue.has_used() && timeout > 0 {
            if VIRTIO_BLOCK_COMPLETE.load(Ordering::Acquire) {
                break;
            }
            core::hint::spin_loop();
            timeout -= 1;
        }
        
        if timeout == 0 {
            queue.free_desc(head);
            queue.free_desc(data_descriptor);
            queue.free_desc(status_descriptor);
            return Err("Request timeout");
        }
        
        // Process completion
        let _used = queue.pop_used().ok_or("No completion")?;
        
        // Free descriptors
        queue.free_desc(head);
        queue.free_desc(data_descriptor);
        queue.free_desc(status_descriptor);
        
        if dma_buf[VirtioBlkReqHdr::SIZE + BLOCK_SIZE] != blk_status::OK {
            return Err("Device error");
        }
        
        // Copy data to output buffer
        buffer.copy_from_slice(&dma_buf[VirtioBlkReqHdr::SIZE..VirtioBlkReqHdr::SIZE + BLOCK_SIZE]);
        
        Ok(())
    }
    
    /// Write sectors to device
    pub fn write_sectors(&mut self, start_sector: u64, count: usize, buffer: &[u8]) -> Result<(), &'static str> {
        if self.read_only {
            return Err("Device is read-only");
        }
        
        if buffer.len() < count * BLOCK_SIZE {
            return Err("Buffer too small");
        }
        
        if start_sector + count as u64 > self.capacity {
            return Err("Write beyond device capacity");
        }
        
        // Write one sector at a time
        for i in 0..count {
            self.write_one_sector(start_sector + i as u64,
                &buffer[i * BLOCK_SIZE..(i + 1) * BLOCK_SIZE])?;
        }
        
        WRITES.fetch_add(count as u64, Ordering::Relaxed);
        BYTES_WRITTEN.fetch_add((count * BLOCK_SIZE) as u64, Ordering::Relaxed);
        
        Ok(())
    }
    
    /// Write a single sector
    fn write_one_sector(&mut self, sector: u64, buffer: &[u8]) -> Result<(), &'static str> {
        // Use heap-allocated DMA buffers to ensure proper physical address translation
        use alloc::vec;
        
        // Allocate DMA buffer on heap (which is in HHDM)
        // Layout: [header: 16 bytes][data: 512 bytes][status: 1 byte]
        let total_size = VirtioBlkReqHdr::SIZE + BLOCK_SIZE + 1;
        let mut dma_buf = vec![0u8; total_size].into_boxed_slice();
        
        // Write header at offset 0
        let header = VirtioBlkReqHdr {
            req_type: req_type::OUT,
            reserved: 0,
            sector,
        };
                // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe {
            let header_pointer = dma_buf.as_mut_ptr() as *mut VirtioBlkReqHdr;
            core::ptr::write(header_pointer, header);
        }
        
        // Copy input data to DMA buffer
        dma_buf[VirtioBlkReqHdr::SIZE..VirtioBlkReqHdr::SIZE + BLOCK_SIZE]
            .copy_from_slice(&buffer[..BLOCK_SIZE]);
        
        // Set status byte to 0xFF at the end
        dma_buf[VirtioBlkReqHdr::SIZE + BLOCK_SIZE] = 0xFF;
        
        // Get physical addresses from heap (which is in HHDM)
        let hhdm = crate::memory::hhdm_offset();
        let dma_virt = dma_buf.as_ptr() as u64;
        let dma_physical = dma_virt - hhdm;
        
        let header_physical = dma_physical;
        let data_phys = dma_physical + VirtioBlkReqHdr::SIZE as u64;
        let status_phys = dma_physical + (VirtioBlkReqHdr::SIZE + BLOCK_SIZE) as u64;
        
        let queue = self.queue.as_mut().ok_or("Queue not initialized")?;
        
        let head = queue.alloc_desc().ok_or("No free descriptor")?;
        let data_descriptor = queue.alloc_desc().ok_or("No free descriptor")?;
        let status_descriptor = queue.alloc_desc().ok_or("No free descriptor")?;
        
        // Header (device reads)
        queue.set_desc(head, header_physical, VirtioBlkReqHdr::SIZE as u32,
            desc_flags::NEXT, data_descriptor);
        
        // Data (device reads for write operation)
        queue.set_desc(data_descriptor, data_phys, BLOCK_SIZE as u32,
            desc_flags::NEXT, status_descriptor);
        
        // Status (device writes)
        queue.set_desc(status_descriptor, status_phys, 1, desc_flags::WRITE, 0);
        
        queue.submit(head);
        
        // Get iobase before dropping queue borrow
        let iobase = self.device.iobase;
        
        // Notify device directly
        unsafe {
            let mut port = crate::arch::Port::<u16>::new(iobase + 0x10);
            port.write(0);
        }
        
        // Wait for completion (interrupt-assisted polling)
        VIRTIO_BLOCK_COMPLETE.store(false, Ordering::Release);
        
        // Get queue reference again for polling
        let queue = self.queue.as_mut().ok_or("Queue not initialized")?;
        
        let mut timeout = 1_000_000u32;
        while !queue.has_used() && timeout > 0 {
            if VIRTIO_BLOCK_COMPLETE.load(Ordering::Acquire) {
                break;
            }
            core::hint::spin_loop();
            timeout -= 1;
        }
        
        if timeout == 0 {
            queue.free_desc(head);
            queue.free_desc(data_descriptor);
            queue.free_desc(status_descriptor);
            return Err("Request timeout");
        }
        
        let _used = queue.pop_used().ok_or("No completion")?;
        
        queue.free_desc(head);
        queue.free_desc(data_descriptor);
        queue.free_desc(status_descriptor);
        
        if dma_buf[VirtioBlkReqHdr::SIZE + BLOCK_SIZE] != blk_status::OK {
            return Err("Device error");
        }
        
        Ok(())
    }
    
    /// Get device capacity in sectors
    pub fn capacity(&self) -> u64 {
        self.capacity
    }
    
    /// Check if device is read-only
    pub fn is_read_only(&self) -> bool {
        self.read_only
    }
}

// ============ Public API ============

/// Initialize virtio-blk driver
pub fn init(pci_dev: &PciDevice) -> Result<(), &'static str> {
    crate::log!("[virtio-blk] Initializing...");
    
    let mut driver = VirtioBlk::new(pci_dev)?;
    driver.setup_queue()?;
    driver.start()?;
    
    // Store iobase for ISR access
    VIRTIO_BLOCK_IOBASE.store(driver.device.iobase, Ordering::SeqCst);
    
    INITIALIZED.store(true, Ordering::SeqCst);
    *DRIVER.lock() = Some(driver);
    
    // Route PCI interrupt through IOAPIC
    let irq = pci_dev.interrupt_line;
    if irq > 0 && irq < 255 {
        crate::apic::route_pci_interrupt_request(irq, crate::apic::VIRTIO_VECTOR);
        crate::serial_println!("[virtio-blk] IRQ {} routed to vector {}", irq, crate::apic::VIRTIO_VECTOR);
    }
    
    Ok(())
}

/// Check if driver is initialized
pub fn is_initialized() -> bool {
    INITIALIZED.load(Ordering::Relaxed)
}

/// Get device capacity in sectors
pub fn capacity() -> u64 {
    DRIVER.lock().as_ref().map(|d| d.capacity()).unwrap_or(0)
}

/// Check if device is read-only
pub fn is_read_only() -> bool {
    DRIVER.lock().as_ref().map(|d| d.is_read_only()).unwrap_or(true)
}

/// Read sectors
pub fn read_sectors(start: u64, count: usize, buffer: &mut [u8]) -> Result<(), &'static str> {
    let mut driver = DRIVER.lock();
    let drv = driver.as_mut().ok_or("Driver not initialized")?;
    drv.read_sectors(start, count, buffer)
}

/// Write sectors
pub fn write_sectors(start: u64, count: usize, buffer: &[u8]) -> Result<(), &'static str> {
    let mut driver = DRIVER.lock();
    let drv = driver.as_mut().ok_or("Driver not initialized")?;
    drv.write_sectors(start, count, buffer)
}

/// Read a single sector (convenience wrapper for VFS)
pub fn read_sector(sector: u64, buffer: &mut [u8; 512]) -> Result<(), &'static str> {
    read_sectors(sector, 1, buffer)
}

/// Write a single sector (convenience wrapper for VFS)
pub fn write_sector(sector: u64, buffer: &[u8; 512]) -> Result<(), &'static str> {
    write_sectors(sector, 1, buffer)
}

/// Get statistics
pub fn get_stats() -> (u64, u64, u64, u64) {
    (
        READS.load(Ordering::Relaxed),
        WRITES.load(Ordering::Relaxed),
        BYTES_READ.load(Ordering::Relaxed),
        BYTES_WRITTEN.load(Ordering::Relaxed),
    )
}

/// Called from the VirtIO ISR — reads ISR status and sets completion flag.
/// Safe to call from interrupt context (no mutex locks).
pub fn handle_interrupt() {
    let iobase = VIRTIO_BLOCK_IOBASE.load(Ordering::Relaxed);
    if iobase == 0 { return; }
    
    // Read ISR status register (iobase+0x13) — this also acknowledges the interrupt
    let isr: u8 = // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe {
        let mut port = crate::arch::Port::<u8>::new(iobase + 0x13);
        port.read()
    };
    
    if isr & 1 != 0 {
        // Bit 0: used ring update (request completed)
        VIRTIO_BLOCK_COMPLETE.store(true, Ordering::Release);
    }
}
