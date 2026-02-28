//! VirtIO Network Driver
//!
//! Real driver for virtio-net devices (QEMU, KVM, etc.)
//! Implements actual packet transmission and reception.

use alloc::vec::Vec;
use alloc::vec;
use alloc::boxed::Box;
use alloc::collections::VecDeque;
use core::sync::atomic::{AtomicBool, Ordering};
use spin::Mutex;
use crate::arch::Port;

use crate::virtio::{self, VirtioDevice, Virtqueue, VirtqDesc, desc_flags, status, legacy_reg};
use crate::pci::PciDevice;

/// virtio-net feature bits
pub mod features {
    pub const CSUM: u32 = 1 << 0;           // Checksum offload
    pub const GUEST_CSUM: u32 = 1 << 1;     // Guest can handle checksums
    pub const MAC: u32 = 1 << 5;            // Device has MAC address
    pub const GSO: u32 = 1 << 6;            // GSO support
    pub const GUEST_TSO4: u32 = 1 << 7;     // Guest can handle TSO v4
    pub const GUEST_TSO6: u32 = 1 << 8;     // Guest can handle TSO v6
    pub const GUEST_ECN: u32 = 1 << 9;      // Guest can handle ECN
    pub const GUEST_UFO: u32 = 1 << 10;     // Guest can handle UFO
    pub const HOST_TSO4: u32 = 1 << 11;     // Host can handle TSO v4
    pub const HOST_TSO6: u32 = 1 << 12;     // Host can handle TSO v6
    pub const HOST_ECN: u32 = 1 << 13;      // Host can handle ECN
    pub const HOST_UFO: u32 = 1 << 14;      // Host can handle UFO
    pub const MRG_RXBUF: u32 = 1 << 15;     // Mergeable RX buffers
    pub const STATUS: u32 = 1 << 16;        // Link status available
    pub const CTRL_VQ: u32 = 1 << 17;       // Control virtqueue
    pub const CTRL_RX: u32 = 1 << 18;       // Control RX mode
    pub const CTRL_VLAN: u32 = 1 << 19;     // Control VLAN filtering
    pub const GUEST_ANNOUNCE: u32 = 1 << 21; // Guest can announce
}

/// virtio-net packet header (legacy mode)
#[repr(C, packed)]
#[derive(Debug, Clone, Copy, Default)]
pub struct VirtioNetHdr {
    pub flags: u8,
    pub gso_type: u8,
    pub hdr_len: u16,
    pub gso_size: u16,
    pub csum_start: u16,
    pub csum_offset: u16,
}

impl VirtioNetHdr {
    pub const SIZE: usize = 10;
    
    pub fn new() -> Self {
        Self::default()
    }
}

/// RX buffer with header space
#[repr(C)]
struct RxBuffer {
    header: VirtioNetHdr,
    data: [u8; 1514],  // Max ethernet frame
}

/// TX buffer with header space
#[repr(C)]
struct TxBuffer {
    header: VirtioNetHdr,
    data: [u8; 1514],
}

/// VirtIO network device driver
pub struct VirtioNet {
    /// Base VirtIO device
    device: VirtioDevice,
    /// Receive queue (queue 0)
    rx_queue: Option<Box<Virtqueue>>,
    /// Transmit queue (queue 1)
    tx_queue: Option<Box<Virtqueue>>,
    /// MAC address
    mac: [u8; 6],
    /// Link status (up/down)
    link_up: bool,
    /// RX buffers (kept alive for DMA)
    rx_buffers: Vec<Box<RxBuffer>>,
    /// TX buffers
    tx_buffers: Vec<Box<TxBuffer>>,
    /// Pending TX descriptor indices
    tx_pending: VecDeque<u16>,
}

/// Global driver instance
static DRIVER: Mutex<Option<VirtioNet>> = Mutex::new(None);
static INITIALIZED: AtomicBool = AtomicBool::new(false);

/// Received packets queue
static RX_PACKETS: Mutex<VecDeque<Vec<u8>>> = Mutex::new(VecDeque::new());

/// VirtIO net I/O base (for ISR access without locking DRIVER)
static VIRTIO_NET_IOBASE: core::sync::atomic::AtomicU16 = core::sync::atomic::AtomicU16::new(0);
/// Interrupt pending flag (set by ISR, cleared by poll)
static VIRTIO_NET_IRQ_PENDING: AtomicBool = AtomicBool::new(false);

/// Statistics
static PACKETS_TX: core::sync::atomic::AtomicU64 = core::sync::atomic::AtomicU64::new(0);
static PACKETS_RX: core::sync::atomic::AtomicU64 = core::sync::atomic::AtomicU64::new(0);
static BYTES_TX: core::sync::atomic::AtomicU64 = core::sync::atomic::AtomicU64::new(0);
static BYTES_RX: core::sync::atomic::AtomicU64 = core::sync::atomic::AtomicU64::new(0);

impl VirtioNet {
    /// Initialize the driver with a PCI device
    pub fn new(pci_dev: &PciDevice) -> Result<Self, &'static str> {
        // Get I/O base from BAR0 (legacy VirtIO uses I/O ports)
        let bar0 = pci_dev.bar[0];
        if bar0 == 0 {
            return Err("BAR0 not configured");
        }
        
        // Check if it's I/O space (bit 0 = 1)
        let iobase = if bar0 & 1 == 1 {
            (bar0 & 0xFFFC) as u16  // I/O port address
        } else {
            return Err("MMIO not supported yet, need I/O port BAR");
        };
        
        crate::log_debug!("[virtio-net] I/O base: {:#X}", iobase);
        
        let mut device = VirtioDevice::new(iobase);
        
        // Reset device
        device.reset();
        
        // Acknowledge device
        device.add_status(status::ACKNOWLEDGE);
        device.add_status(status::DRIVER);
        
        // Read features
        let device_features = device.read_device_features();
        crate::log_debug!("[virtio-net] Device features: {:#X}", device_features);
        
        // Negotiate features (we only need basic ones for now)
        let mut driver_features = 0u32;
        if device_features & features::MAC != 0 {
            driver_features |= features::MAC;
        }
        if device_features & features::STATUS != 0 {
            driver_features |= features::STATUS;
        }
        
        device.write_driver_features(driver_features);
        crate::log_debug!("[virtio-net] Driver features: {:#X}", driver_features);
        
        // Read MAC address from config space
        let mut mac = [0u8; 6];
        for i in 0..6 {
            mac[i] = device.read_config8(i as u16);
        }
        crate::log!("[virtio-net] MAC: {:02X}:{:02X}:{:02X}:{:02X}:{:02X}:{:02X}",
            mac[0], mac[1], mac[2], mac[3], mac[4], mac[5]);
        
        // Check link status
        let link_up = if driver_features & features::STATUS != 0 {
            let net_status = device.read_config16(6);
            net_status & 1 != 0
        } else {
            true // Assume up if no status feature
        };
        
        crate::log!("[virtio-net] Link: {}", if link_up { "UP" } else { "DOWN" });
        
        Ok(Self {
            device,
            rx_queue: None,
            tx_queue: None,
            mac,
            link_up,
            rx_buffers: Vec::new(),
            tx_buffers: Vec::new(),
            tx_pending: VecDeque::new(),
        })
    }
    
    /// Setup virtqueues
    pub fn setup_queues(&mut self) -> Result<(), &'static str> {
        // Setup RX queue (queue 0)
        self.device.select_queue(0);
        let rx_size = self.device.get_queue_size();
        crate::log_debug!("[virtio-net] RX queue size: {}", rx_size);
        
        if rx_size == 0 {
            return Err("RX queue not available");
        }
        
        let rx_queue = self.alloc_queue(rx_size)?;
        
        // Tell device where the queue is
        let pfn = (rx_queue.phys_addr / 4096) as u32;
        self.device.set_queue_address(pfn);
        
        self.rx_queue = Some(rx_queue);
        
        // Setup TX queue (queue 1)
        self.device.select_queue(1);
        let tx_size = self.device.get_queue_size();
        crate::log_debug!("[virtio-net] TX queue size: {}", tx_size);
        
        if tx_size == 0 {
            return Err("TX queue not available");
        }
        
        let tx_queue = self.alloc_queue(tx_size)?;
        
        let pfn = (tx_queue.phys_addr / 4096) as u32;
        self.device.set_queue_address(pfn);
        
        self.tx_queue = Some(tx_queue);
        
        Ok(())
    }
    
    /// Allocate a virtqueue
    fn alloc_queue(&mut self, size: u16) -> Result<Box<Virtqueue>, &'static str> {
        let total_size = Virtqueue::calc_size(size);
        
        // Allocate page-aligned memory
        // For simplicity, use heap (in production, use DMA-safe allocator)
        let layout = core::alloc::Layout::from_size_align(total_size, 4096)
            .map_err(|_| "Layout error")?;
        
        let ptr = unsafe { alloc::alloc::alloc_zeroed(layout) };
        if ptr.is_null() {
            return Err("Failed to allocate queue memory");
        }
        
        // Calculate virtual addresses of each section
        let desc_ptr = ptr as *mut VirtqDesc;
        let avail_offset = core::mem::size_of::<VirtqDesc>() * size as usize;
        let avail_ptr = unsafe { ptr.add(avail_offset) as *mut virtio::VirtqAvail };
        let used_offset = ((avail_offset + 6 + 2 * size as usize) + 4095) & !4095;
        let used_ptr = unsafe { ptr.add(used_offset) as *mut virtio::VirtqUsed };
        
        // Physical address (for DMA) - in higher half, subtract HHDM offset
        let virt_addr = ptr as u64;
        let hhdm_offset = crate::memory::hhdm_offset();
        let phys_addr = if virt_addr >= hhdm_offset {
            virt_addr - hhdm_offset
        } else {
            // Heap is below HHDM, use identity mapping assumption
            virt_addr
        };
        
        // Initialize free list
        let mut free_list = vec![0u16; size as usize];
        for i in 0..(size - 1) {
            free_list[i as usize] = i + 1;
        }
        free_list[(size - 1) as usize] = 0xFFFF; // End marker
        
        Ok(Box::new(Virtqueue {
            size,
            phys_addr,
            desc: desc_ptr,
            avail: avail_ptr,
            used: used_ptr,
            last_used_idx: 0,
            free_head: 0,
            num_free: size,
            free_list,
        }))
    }
    
    /// Setup RX buffers
    pub fn setup_rx_buffers(&mut self) -> Result<(), &'static str> {
        let queue = self.rx_queue.as_mut().ok_or("RX queue not initialized")?;
        
        // Allocate buffers and add them to the available ring
        let num_buffers = (queue.size / 2).min(32) as usize; // Use half the queue, max 32
        
        for _ in 0..num_buffers {
            let buffer = Box::new(RxBuffer {
                header: VirtioNetHdr::new(),
                data: [0u8; 1514],
            });
            
            let desc_idx = queue.alloc_desc().ok_or("No free descriptors")?;
            
            // Get physical address of buffer
            let buffer_ptr = &*buffer as *const RxBuffer;
            let virt_addr = buffer_ptr as u64;
            let hhdm = crate::memory::hhdm_offset();
            let phys_addr = if virt_addr >= hhdm { virt_addr - hhdm } else { virt_addr };
            
            // Setup descriptor (device writes to this buffer)
            unsafe {
                let desc = &mut *queue.desc.add(desc_idx as usize);
                desc.addr = phys_addr;
                desc.len = core::mem::size_of::<RxBuffer>() as u32;
                desc.flags = desc_flags::WRITE; // Device writes to buffer
                desc.next = 0;
                
                // Add to available ring
                queue.add_available(desc_idx);
            }
            
            self.rx_buffers.push(buffer);
        }
        
        crate::log_debug!("[virtio-net] {} RX buffers ready", num_buffers);
        
        Ok(())
    }
    
    /// Start the device
    pub fn start(&mut self) -> Result<(), &'static str> {
        // Signal that driver is ready
        self.device.add_status(status::DRIVER_OK);
        
        // Notify device about RX buffers
        self.device.notify_queue(0);
        
        crate::log!("[virtio-net] Device started");
        Ok(())
    }
    
    /// Send a packet
    pub fn send(&mut self, data: &[u8]) -> Result<(), &'static str> {
        if data.len() > 1514 {
            return Err("Packet too large");
        }
        
        // First, recycle any completed TX descriptors
        self.poll_tx_internal();
        
        let queue = self.tx_queue.as_mut().ok_or("TX queue not initialized")?;
        
        // Allocate TX buffer
        let mut buffer = Box::new(TxBuffer {
            header: VirtioNetHdr::new(),
            data: [0u8; 1514],
        });
        
        // Copy packet data
        buffer.data[..data.len()].copy_from_slice(data);
        
        // Allocate descriptor
        let desc_idx = queue.alloc_desc().ok_or("TX queue full")?;
        
        // Get physical address
        let buffer_ptr = &*buffer as *const TxBuffer;
        let virt_addr = buffer_ptr as u64;
        let hhdm = crate::memory::hhdm_offset();
        let phys_addr = if virt_addr >= hhdm { virt_addr - hhdm } else { virt_addr };
        
        // Setup descriptor
        unsafe {
            let desc = &mut *queue.desc.add(desc_idx as usize);
            desc.addr = phys_addr;
            desc.len = (VirtioNetHdr::SIZE + data.len()) as u32;
            desc.flags = 0; // Device reads from buffer
            desc.next = 0;
            
            queue.add_available(desc_idx);
        }
        
        // Keep buffer alive until transmission complete
        self.tx_buffers.push(buffer);
        self.tx_pending.push_back(desc_idx);
        
        // Notify device
        self.device.notify_queue(1);
        
        // Update stats
        PACKETS_TX.fetch_add(1, Ordering::Relaxed);
        BYTES_TX.fetch_add(data.len() as u64, Ordering::Relaxed);
        
        Ok(())
    }
    
    /// Poll for received packets
    pub fn poll_rx(&mut self) -> Option<Vec<u8>> {
        let queue = self.rx_queue.as_mut()?;
        
        unsafe {
            if !queue.has_used() {
                return None;
            }
            
            let used = queue.pop_used()?;
            
            // used is (id, len) tuple
            let desc_idx = used.0 as u16;
            let desc = &*queue.desc.add(desc_idx as usize);
            
            // Get the data (skip virtio header)
            let buffer_ptr = (crate::memory::hhdm_offset() + desc.addr) as *const RxBuffer;
            let buffer = &*buffer_ptr;
            
            let data_len = (used.1 as usize).saturating_sub(VirtioNetHdr::SIZE);
            if data_len > 0 && data_len <= 1514 {
                let packet = buffer.data[..data_len].to_vec();
                
                // Re-add buffer to available ring
                queue.add_available(desc_idx);
                self.device.notify_queue(0);
                
                // Update stats
                PACKETS_RX.fetch_add(1, Ordering::Relaxed);
                BYTES_RX.fetch_add(data_len as u64, Ordering::Relaxed);
                
                return Some(packet);
            }
            
            // Re-add buffer even on error
            queue.add_available(desc_idx);
            self.device.notify_queue(0);
        }
        
        None
    }
    
    /// Handle completed transmissions (internal)
    fn poll_tx_internal(&mut self) {
        let queue = match self.tx_queue.as_mut() {
            Some(q) => q,
            None => return,
        };
        
        unsafe {
            while queue.has_used() {
                if let Some(used) = queue.pop_used() {
                    // Free the descriptor - used is (id, len) tuple
                    queue.free_desc(used.0 as u16);
                    
                    // Remove from pending
                    if let Some(pos) = self.tx_pending.iter().position(|&x| x == used.0 as u16) {
                        self.tx_pending.remove(pos);
                    }
                }
            }
        }
        
        // Clean up TX buffers (simple approach: keep last N)
        while self.tx_buffers.len() > 16 && !self.tx_pending.is_empty() {
            self.tx_buffers.remove(0);
        }
    }
    
    /// Handle completed transmissions (public API)
    pub fn poll_tx(&mut self) {
        self.poll_tx_internal();
    }
    
    /// Get MAC address
    pub fn mac(&self) -> [u8; 6] {
        self.mac
    }
    
    /// Check link status
    pub fn is_link_up(&self) -> bool {
        self.link_up
    }
    
    /// Get the I/O base address (for IRQ setup from the universal driver wrapper)
    pub fn iobase(&self) -> u16 {
        self.device.iobase
    }
}

// ============ Public API ============

/// Initialize virtio-net driver
pub fn init(pci_dev: &PciDevice) -> Result<(), &'static str> {
    crate::log!("[virtio-net] Initializing...");
    
    let mut driver = VirtioNet::new(pci_dev)?;
    driver.setup_queues()?;
    driver.setup_rx_buffers()?;
    driver.start()?;
    
    // Store iobase for ISR access
    VIRTIO_NET_IOBASE.store(driver.device.iobase, Ordering::SeqCst);
    
    INITIALIZED.store(true, Ordering::SeqCst);
    *DRIVER.lock() = Some(driver);
    
    // Route PCI interrupt through IOAPIC
    let irq = pci_dev.interrupt_line;
    if irq > 0 && irq < 255 {
        crate::apic::route_pci_irq(irq, crate::apic::VIRTIO_VECTOR);
        crate::serial_println!("[virtio-net] IRQ {} routed to vector {}", irq, crate::apic::VIRTIO_VECTOR);
    }
    
    Ok(())
}

/// Check if driver is initialized
pub fn is_initialized() -> bool {
    INITIALIZED.load(Ordering::Relaxed)
}

/// Get MAC address
pub fn get_mac() -> Option<[u8; 6]> {
    DRIVER.lock().as_ref().map(|d| d.mac())
}

/// Send a raw ethernet frame
pub fn send_packet(data: &[u8]) -> Result<(), &'static str> {
    let mut driver = DRIVER.lock();
    let drv = driver.as_mut().ok_or("Driver not initialized")?;
    drv.send(data)
}

/// Poll for incoming packets (non-blocking)
pub fn poll() {
    // Clear IRQ pending flag if set
    VIRTIO_NET_IRQ_PENDING.store(false, Ordering::Relaxed);
    
    let mut driver = DRIVER.lock();
    if let Some(drv) = driver.as_mut() {
        // Check for completed TX
        drv.poll_tx();
        
        // Check for received packets
        while let Some(packet) = drv.poll_rx() {
            RX_PACKETS.lock().push_back(packet);
        }
    }
}

/// Receive a packet (non-blocking)
pub fn receive_packet() -> Option<Vec<u8>> {
    poll(); // Update RX queue first
    RX_PACKETS.lock().pop_front()
}

/// Get driver statistics
pub fn get_stats() -> (u64, u64, u64, u64) {
    (
        PACKETS_TX.load(Ordering::Relaxed),
        PACKETS_RX.load(Ordering::Relaxed),
        BYTES_TX.load(Ordering::Relaxed),
        BYTES_RX.load(Ordering::Relaxed),
    )
}

/// Called from the VirtIO ISR — reads ISR status and sets pending flag.
/// Safe to call from interrupt context (no mutex locks).
pub fn handle_interrupt() {
    let iobase = VIRTIO_NET_IOBASE.load(Ordering::Relaxed);
    if iobase == 0 { return; }
    
    // Read ISR status register (iobase+0x13) — this also acknowledges the interrupt
    let isr: u8 = unsafe {
        let mut port = Port::<u8>::new(iobase + 0x13);
        port.read()
    };
    
    if isr & 1 != 0 {
        // Bit 0: used ring update (packets ready)
        VIRTIO_NET_IRQ_PENDING.store(true, Ordering::Release);
    }
}

/// Check if there's a pending VirtIO net interrupt
pub fn irq_pending() -> bool {
    VIRTIO_NET_IRQ_PENDING.load(Ordering::Relaxed)
}

/// Set the I/O base for interrupt handling (used by the universal driver wrapper
/// when the legacy init path is not taken).
pub fn set_iobase_for_irq(iobase: u16) {
    VIRTIO_NET_IOBASE.store(iobase, Ordering::SeqCst);
}

/// Handle interrupt using just the stored IOBASE (no DRIVER lock).
/// Used by the interrupt handler when the new driver framework is active
/// but the legacy INITIALIZED flag is false.
pub fn handle_interrupt_from_iobase() {
    handle_interrupt();
}
