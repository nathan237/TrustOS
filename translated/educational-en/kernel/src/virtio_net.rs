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
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const CSUM: u32 = 1 << 0;           // Checksum offload
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const GUEST_CSUM: u32 = 1 << 1;     // Guest can handle checksums
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const MAC: u32 = 1 << 5;            // Device has MAC address
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const GSO: u32 = 1 << 6;            // GSO support
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const GUEST_TSO4: u32 = 1 << 7;     // Guest can handle TSO v4
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const GUEST_TSO6: u32 = 1 << 8;     // Guest can handle TSO v6
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const GUEST_ECN: u32 = 1 << 9;      // Guest can handle ECN
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const GUEST_UFO: u32 = 1 << 10;     // Guest can handle UFO
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const HOST_TSO4: u32 = 1 << 11;     // Host can handle TSO v4
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const HOST_TSO6: u32 = 1 << 12;     // Host can handle TSO v6
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const HOST_ECN: u32 = 1 << 13;      // Host can handle ECN
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const HOST_UFO: u32 = 1 << 14;      // Host can handle UFO
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const MRG_RXBUF: u32 = 1 << 15;     // Mergeable RX buffers
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const STATUS: u32 = 1 << 16;        // Link status available
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const CONTROLLER_VQ: u32 = 1 << 17;       // Control virtqueue
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const CONTROLLER_RECEIVE: u32 = 1 << 18;       // Control RX mode
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const CONTROLLER_VLAN: u32 = 1 << 19;     // Control VLAN filtering
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const GUEST_ANNOUNCE: u32 = 1 << 21; // Guest can announce
}

/// virtio-net packet header (legacy mode)
#[repr(C, packed)]
// #[derive] — auto-generates trait implementations at compile time.
#[derive(Debug, Clone, Copy, Default)]
// Public structure — visible outside this module.
pub struct VirtioNetHdr {
    pub flags: u8,
    pub gso_type: u8,
    pub header_length: u16,
    pub gso_size: u16,
    pub csum_start: u16,
    pub csum_offset: u16,
}

// Implementation block — defines methods for the type above.
impl VirtioNetHdr {
    pub     // Compile-time constant — evaluated at compilation, zero runtime cost.
const SIZE: usize = 10;
    
        // Public function — callable from other modules.
pub fn new() -> Self {
        Self::default()
    }
}

/// RX buffer with header space
#[repr(C)]
struct ReceiveBuffer {
    header: VirtioNetHdr,
    data: [u8; 1514],  // Max ethernet frame
}

/// TX buffer with header space
#[repr(C)]
struct TransmitBuffer {
    header: VirtioNetHdr,
    data: [u8; 1514],
}

/// VirtIO network device driver
pub struct VirtioNet {
    /// Base VirtIO device
    device: VirtioDevice,
    /// Receive queue (queue 0)
    receive_queue: Option<Box<Virtqueue>>,
    /// Transmit queue (queue 1)
    transmit_queue: Option<Box<Virtqueue>>,
    /// MAC address
    mac: [u8; 6],
    /// Link status (up/down)
    link_up: bool,
    /// RX buffers (kept alive for DMA)
    receive_buffers: Vec<Box<ReceiveBuffer>>,
    /// TX buffers
    transmit_buffers: Vec<Box<TransmitBuffer>>,
    /// Pending TX descriptor indices
    transmit_pending: VecDeque<u16>,
}

/// Global driver instance
static DRIVER: Mutex<Option<VirtioNet>> = Mutex::new(None);
// Atomic variable — provides lock-free thread-safe access.
static INITIALIZED: AtomicBool = AtomicBool::new(false);

/// Received packets queue
static RECEIVE_PACKETS: Mutex<VecDeque<Vec<u8>>> = Mutex::new(VecDeque::new());

/// VirtIO net I/O base (for ISR access without locking DRIVER)
static VIRTIO_NET_IOBASE: core::sync::atomic::AtomicU16 = core::sync::atomic::AtomicU16::new(0);
/// Interrupt pending flag (set by ISR, cleared by poll)
static VIRTIO_NET_INTERRUPT_REQUEST_PENDING: AtomicBool = AtomicBool::new(false);

/// Statistics
static PACKETS_TRANSMIT: core::sync::atomic::AtomicU64 = core::sync::atomic::AtomicU64::new(0);
// Atomic variable — provides lock-free thread-safe access.
static PACKETS_RECEIVE: core::sync::atomic::AtomicU64 = core::sync::atomic::AtomicU64::new(0);
// Atomic variable — provides lock-free thread-safe access.
static BYTES_TRANSMIT: core::sync::atomic::AtomicU64 = core::sync::atomic::AtomicU64::new(0);
// Atomic variable — provides lock-free thread-safe access.
static BYTES_RECEIVE: core::sync::atomic::AtomicU64 = core::sync::atomic::AtomicU64::new(0);

// Implementation block — defines methods for the type above.
impl VirtioNet {
    /// Initialize the driver with a PCI device
    pub fn new(pci_device: &PciDevice) -> Result<Self, &'static str> {
        // Get I/O base from BAR0 (legacy VirtIO uses I/O ports)
        let bar0 = pci_device.bar[0];
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
            receive_queue: None,
            transmit_queue: None,
            mac,
            link_up,
            receive_buffers: Vec::new(),
            transmit_buffers: Vec::new(),
            transmit_pending: VecDeque::new(),
        })
    }
    
    /// Setup virtqueues
    pub fn setup_queues(&mut self) -> Result<(), &'static str> {
        // Setup RX queue (queue 0)
        self.device.select_queue(0);
        let receive_size = self.device.get_queue_size();
        crate::log_debug!("[virtio-net] RX queue size: {}", receive_size);
        
        if receive_size == 0 {
            return Err("RX queue not available");
        }
        
        let receive_queue = self.allocator_queue(receive_size)?;
        
        // Tell device where the queue is
        let pfn = (receive_queue.physical_address / 4096) as u32;
        self.device.set_queue_address(pfn);
        
        self.receive_queue = Some(receive_queue);
        
        // Setup TX queue (queue 1)
        self.device.select_queue(1);
        let transmit_size = self.device.get_queue_size();
        crate::log_debug!("[virtio-net] TX queue size: {}", transmit_size);
        
        if transmit_size == 0 {
            return Err("TX queue not available");
        }
        
        let transmit_queue = self.allocator_queue(transmit_size)?;
        
        let pfn = (transmit_queue.physical_address / 4096) as u32;
        self.device.set_queue_address(pfn);
        
        self.transmit_queue = Some(transmit_queue);
        
        Ok(())
    }
    
    /// Allocate a virtqueue
    fn allocator_queue(&mut self, size: u16) -> Result<Box<Virtqueue>, &'static str> {
        let total_size = Virtqueue::calc_size(size);
        
        // Allocate page-aligned memory
        // For simplicity, use heap (in production, use DMA-safe allocator)
        let layout = core::alloc::Layout::from_size_align(total_size, 4096)
            .map_error(|_| "Layout error")?;
        
        let ptr = // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe { alloc::alloc::alloc_zeroed(layout) };
        if ptr.is_null() {
            return Err("Failed to allocate queue memory");
        }
        
        // Calculate virtual addresses of each section
        let descriptor_pointer = ptr as *mut VirtqDesc;
        let avail_offset = core::mem::size_of::<VirtqDesc>() * size as usize;
        let avail_pointer = // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe { ptr.add(avail_offset) as *mut virtio::VirtqAvail };
        let used_offset = ((avail_offset + 6 + 2 * size as usize) + 4095) & !4095;
        let used_pointer = // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe { ptr.add(used_offset) as *mut virtio::VirtqUsed };
        
        // Physical address (for DMA) - in higher half, subtract HHDM offset
        let virt_address = ptr as u64;
        let hhdm_offset = crate::memory::hhdm_offset();
        let physical_address = if virt_address >= hhdm_offset {
            virt_address - hhdm_offset
        } else {
            // Heap is below HHDM, use identity mapping assumption
            virt_address
        };
        
        // Initialize free list
        let mut free_list = vec![0u16; size as usize];
        for i in 0..(size - 1) {
            free_list[i as usize] = i + 1;
        }
        free_list[(size - 1) as usize] = 0xFFFF; // End marker
        
        Ok(Box::new(Virtqueue {
            size,
            physical_address,
            desc: descriptor_pointer,
            avail: avail_pointer,
            used: used_pointer,
            last_used_index: 0,
            free_head: 0,
            number_free: size,
            free_list,
        }))
    }
    
    /// Setup RX buffers
    pub fn setup_receive_buffers(&mut self) -> Result<(), &'static str> {
        let queue = self.receive_queue.as_mut().ok_or("RX queue not initialized")?;
        
        // Allocate buffers and add them to the available ring
        let number_buffers = (queue.size / 2).minimum(128) as usize; // Use half the queue, max 128
        
        for _ in 0..number_buffers {
            let buffer = Box::new(ReceiveBuffer {
                header: VirtioNetHdr::new(),
                data: [0u8; 1514],
            });
            
            let descriptor_index = queue.allocator_descriptor().ok_or("No free descriptors")?;
            
            // Get physical address of buffer
            let buffer_pointer = &*buffer as *// Compile-time constant — evaluated at compilation, zero runtime cost.
const ReceiveBuffer;
            let virt_address = buffer_pointer as u64;
            let hhdm = crate::memory::hhdm_offset();
            let physical_address = if virt_address >= hhdm { virt_address - hhdm } else { virt_address };
            
            // Setup descriptor (device writes to this buffer)
            unsafe {
                let desc = &mut *queue.desc.add(descriptor_index as usize);
                desc.address = physical_address;
                desc.len = core::mem::size_of::<ReceiveBuffer>() as u32;
                desc.flags = desc_flags::WRITE; // Device writes to buffer
                desc.next = 0;
                
                // Add to available ring
                queue.add_available(descriptor_index);
            }
            
            self.receive_buffers.push(buffer);
        }
        
        crate::log_debug!("[virtio-net] {} RX buffers ready", number_buffers);
        
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
        self.poll_transmit_internal();
        
        let queue = self.transmit_queue.as_mut().ok_or("TX queue not initialized")?;
        
        // Allocate TX buffer
        let mut buffer = Box::new(TransmitBuffer {
            header: VirtioNetHdr::new(),
            data: [0u8; 1514],
        });
        
        // Copy packet data
        buffer.data[..data.len()].copy_from_slice(data);
        
        // Allocate descriptor
        let descriptor_index = queue.allocator_descriptor().ok_or("TX queue full")?;
        
        // Get physical address
        let buffer_pointer = &*buffer as *// Compile-time constant — evaluated at compilation, zero runtime cost.
const TransmitBuffer;
        let virt_address = buffer_pointer as u64;
        let hhdm = crate::memory::hhdm_offset();
        let physical_address = if virt_address >= hhdm { virt_address - hhdm } else { virt_address };
        
        // Setup descriptor
        unsafe {
            let desc = &mut *queue.desc.add(descriptor_index as usize);
            desc.address = physical_address;
            desc.len = (VirtioNetHdr::SIZE + data.len()) as u32;
            desc.flags = 0; // Device reads from buffer
            desc.next = 0;
            
            queue.add_available(descriptor_index);
        }
        
        // Keep buffer alive until transmission complete
        self.transmit_buffers.push(buffer);
        self.transmit_pending.push_back(descriptor_index);
        
        // Notify device
        self.device.notify_queue(1);
        
        // Update stats
        PACKETS_TRANSMIT.fetch_add(1, Ordering::Relaxed);
        BYTES_TRANSMIT.fetch_add(data.len() as u64, Ordering::Relaxed);
        
        Ok(())
    }
    
    /// Poll for received packets
    pub fn poll_receive(&mut self) -> Option<Vec<u8>> {
        let queue = self.receive_queue.as_mut()?;
        
                // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe {
            if !queue.has_used() {
                return None;
            }
            
            let used = queue.pop_used()?;
            
            // used is (id, len) tuple
            let descriptor_index = used.0 as u16;
            let desc = &*queue.desc.add(descriptor_index as usize);
            
            // Get the data (skip virtio header)
            let buffer_pointer = (crate::memory::hhdm_offset() + desc.address) as *// Compile-time constant — evaluated at compilation, zero runtime cost.
const ReceiveBuffer;
            let buffer = &*buffer_pointer;
            
            let data_length = (used.1 as usize).saturating_sub(VirtioNetHdr::SIZE);
            if data_length > 0 && data_length <= 1514 {
                let packet = buffer.data[..data_length].to_vec();
                
                // Re-add buffer to available ring
                queue.add_available(descriptor_index);
                self.device.notify_queue(0);
                
                // Update stats
                PACKETS_RECEIVE.fetch_add(1, Ordering::Relaxed);
                BYTES_RECEIVE.fetch_add(data_length as u64, Ordering::Relaxed);
                
                return Some(packet);
            }
            
            // Re-add buffer even on error
            queue.add_available(descriptor_index);
            self.device.notify_queue(0);
        }
        
        None
    }
    
    /// Handle completed transmissions (internal)
    fn poll_transmit_internal(&mut self) {
        let queue = // Pattern matching — Rust's exhaustive branching construct.
match self.transmit_queue.as_mut() {
            Some(q) => q,
            None => return,
        };
        
                // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe {
            while queue.has_used() {
                if let Some(used) = queue.pop_used() {
                    // Free the descriptor - used is (id, len) tuple
                    queue.free_descriptor(used.0 as u16);
                    
                    // Remove from pending
                    if let Some(position) = self.transmit_pending.iter().position(|&x| x == used.0 as u16) {
                        self.transmit_pending.remove(position);
                    }
                }
            }
        }
        
        // Clean up TX buffers (simple approach: keep last N)
        while self.transmit_buffers.len() > 16 && !self.transmit_pending.is_empty() {
            self.transmit_buffers.remove(0);
        }
    }
    
    /// Handle completed transmissions (public API)
    pub fn poll_transmit(&mut self) {
        self.poll_transmit_internal();
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
pub fn init(pci_device: &PciDevice) -> Result<(), &'static str> {
    crate::log!("[virtio-net] Initializing...");
    
    let mut driver = VirtioNet::new(pci_device)?;
    driver.setup_queues()?;
    driver.setup_receive_buffers()?;
    driver.start()?;
    
    // Store iobase for ISR access
    VIRTIO_NET_IOBASE.store(driver.device.iobase, Ordering::SeqCst);
    
    INITIALIZED.store(true, Ordering::SeqCst);
    *DRIVER.lock() = Some(driver);
    
    // Route PCI interrupt through IOAPIC
    let irq = pci_device.interrupt_line;
    if irq > 0 && irq < 255 {
        crate::apic::route_pci_interrupt_request(irq, crate::apic::VIRTIO_VECTOR);
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
    let driver = driver.as_mut().ok_or("Driver not initialized")?;
    driver.send(data)
}

/// Poll for incoming packets (non-blocking)
pub fn poll() {
    // Clear IRQ pending flag if set
    VIRTIO_NET_INTERRUPT_REQUEST_PENDING.store(false, Ordering::Relaxed);
    
    let mut driver = DRIVER.lock();
    if let Some(driver) = driver.as_mut() {
        // Check for completed TX
        driver.poll_transmit();
        
        // Check for received packets
        while let Some(packet) = driver.poll_receive() {
            RECEIVE_PACKETS.lock().push_back(packet);
        }
    }
}

/// Receive a packet (non-blocking)
pub fn receive_packet() -> Option<Vec<u8>> {
    poll(); // Update RX queue first
    RECEIVE_PACKETS.lock().pop_front()
}

/// Get driver statistics
pub fn get_stats() -> (u64, u64, u64, u64) {
    (
        PACKETS_TRANSMIT.load(Ordering::Relaxed),
        PACKETS_RECEIVE.load(Ordering::Relaxed),
        BYTES_TRANSMIT.load(Ordering::Relaxed),
        BYTES_RECEIVE.load(Ordering::Relaxed),
    )
}

/// Called from the VirtIO ISR — reads ISR status and sets pending flag.
/// Safe to call from interrupt context (no mutex locks).
pub fn handle_interrupt() {
    let iobase = VIRTIO_NET_IOBASE.load(Ordering::Relaxed);
    if iobase == 0 { return; }
    
    // Read ISR status register (iobase+0x13) — this also acknowledges the interrupt
    let interrupt_handler: u8 = // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe {
        let mut port = Port::<u8>::new(iobase + 0x13);
        port.read()
    };
    
    if interrupt_handler & 1 != 0 {
        // Bit 0: used ring update (packets ready)
        VIRTIO_NET_INTERRUPT_REQUEST_PENDING.store(true, Ordering::Release);
    }
}

/// Check if there's a pending VirtIO net interrupt
pub fn interrupt_request_pending() -> bool {
    VIRTIO_NET_INTERRUPT_REQUEST_PENDING.load(Ordering::Relaxed)
}

/// Set the I/O base for interrupt handling (used by the universal driver wrapper
/// when the legacy init path is not taken).
pub fn set_iobase_for_interrupt_request(iobase: u16) {
    VIRTIO_NET_IOBASE.store(iobase, Ordering::SeqCst);
}

/// Handle interrupt using just the stored IOBASE (no DRIVER lock).
/// Used by the interrupt handler when the new driver framework is active
/// but the legacy INITIALIZED flag is false.
pub fn handle_interrupt_from_iobase() {
    handle_interrupt();
}
