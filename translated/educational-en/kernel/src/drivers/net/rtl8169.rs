//! Realtek RTL8169/8168/8111 Gigabit Ethernet Driver
//!
//! Full driver for Realtek RTL8169-family NICs.
//! Supports QEMU `-device rtl8139` (8139C+/8169 compatible mode).
//!
//! Features:
//! - MMIO register access (volatile)
//! - TX/RX descriptor rings (C+ mode)
//! - Link detection
//! - MAC address read
//! - Polled packet send/receive

use alloc::boxed::Box;
use alloc::vec::Vec;
use alloc::vec;
use core::ptr::{read_volatile, write_volatile};
use core::sync::atomic::{AtomicBool, AtomicU64, Ordering};

use super::{NetworkDriver, NetStats};
use crate::drivers::{Driver, DriverInformation, DriverStatus, DriverCategory};
use crate::pci::PciDevice;

// ============================================================================
// RTL8169 Register Offsets
// ============================================================================

const REGISTER_MAC0: u32       = 0x00;  // MAC address bytes 0-3
const REGISTER_MAC4: u32       = 0x04;  // MAC address bytes 4-5
const REGISTER_TNPDS: u32      = 0x20;  // TX Normal Priority Descriptors (lo)
const REGISTER_TNPDS_HI: u32   = 0x24;  // TX Normal Priority Descriptors (hi)
const REGISTER_COMMAND: u32         = 0x37;  // Command register (8-bit)
const REGISTER_TPPOLL: u32      = 0x38;  // TX Priority Polling (8-bit)
const REGISTER_IMR: u32         = 0x3C;  // Interrupt Mask Register (16-bit)
const REGISTER_INTERRUPT_HANDLER: u32         = 0x3E;  // Interrupt Status Register (16-bit)
const REGISTER_TRANSMIT_CONFIG: u32   = 0x40;  // TX Configuration
const REGISTER_RECEIVE_CONFIG: u32   = 0x44;  // RX Configuration
const REGISTER_MPC: u32         = 0x4C;  // Missed Packet Counter
const REGISTER_9346CR: u32      = 0x50;  // 93C46 Command Register (8-bit)
const REGISTER_CONFIG1: u32     = 0x52;  // Configuration 1
const REGISTER_PHYSICAL_STATUS: u32  = 0x6C;  // PHY Status
const REGISTER_RECEIVE_MAXIMUM_SIZE: u32 = 0xDA;  // RX Max Packet Size (16-bit)
const REGISTER_CPCR: u32        = 0xE0;  // C+ Command Register (16-bit)
const REGISTER_RDSAR: u32       = 0xE4;  // RX Descriptor Start Address (lo)
const REGISTER_RDSAR_HI: u32    = 0xE8;  // RX Descriptor Start Address (hi)
const REGISTER_ETH_TRANSMIT_EARLY: u32 = 0xEC; // Early TX threshold (8-bit)

// ============================================================================
// Register Bit Definitions
// ============================================================================

// CMD register (0x37)
const COMMAND_RESET: u8 = 0x10;
// Compile-time constant — evaluated at compilation, zero runtime cost.
const COMMAND_RECEIVE_ENABLE: u8 = 0x08;
// Compile-time constant — evaluated at compilation, zero runtime cost.
const COMMAND_TRANSMIT_ENABLE: u8 = 0x04;

// TPPOLL register (0x38)
const TPPOLL_NPQ: u8 = 0x40;  // Normal Priority Queue polling

// Interrupt bits (IMR/ISR)
const INT_ROK: u16 = 0x0001;    // RX OK
const INT_TOKEN: u16 = 0x0004;    // TX OK
const INT_LINK_CHG: u16 = 0x0020; // Link change
const INT_RECEIVE_OVERFLOW: u16 = 0x0010;
// Compile-time constant — evaluated at compilation, zero runtime cost.
const INT_ALL: u16 = INT_ROK | INT_TOKEN | INT_LINK_CHG | INT_RECEIVE_OVERFLOW;

// TX Config
const TRANSMIT_CONFIGURATION_IFG: u32 = 0x03 << 24;  // Inter-frame gap (standard)
const TRANSMIT_CONFIGURATION_DMA_BURST: u32 = 0x07 << 8; // max DMA burst (unlimited)

// RX Config
const RECEIVE_CONFIGURATION_AAP: u32 = 1 << 0;   // Accept All Packets
const RECEIVE_CONFIGURATION_APM: u32 = 1 << 1;   // Accept Physical Match
const RECEIVE_CONFIGURATION_AM: u32  = 1 << 2;   // Accept Multicast
const RECEIVE_CONFIGURATION_AB: u32  = 1 << 3;   // Accept Broadcast
const RECEIVE_CONFIGURATION_WRAP: u32 = 1 << 7;  // No wrap (not used in C+ mode)
const RECEIVE_CONFIGURATION_DMA_BURST: u32 = 0x07 << 8; // Max DMA burst
const RECEIVE_CONFIGURATION_NO_THRESHOLD: u32 = 0x07 << 13; // No FIFO threshold

// C+ Command Register
const CPCR_RECEIVE_VLAN: u16 = 1 << 6;
// Compile-time constant — evaluated at compilation, zero runtime cost.
const CPCR_RECEIVE_CHKSUM: u16 = 1 << 5;
// Compile-time constant — evaluated at compilation, zero runtime cost.
const CPCR_PCI_MUL_RW: u16 = 1 << 3;

// 93C46 Command Register (unlock/lock config)
const CONFIGURATION_9346_UNLOCK: u8 = 0xC0;
// Compile-time constant — evaluated at compilation, zero runtime cost.
const CONFIGURATION_9346_LOCK: u8   = 0x00;

// PHY Status register (0x6C)
const PHYSICAL_STATUS_LINK: u32 = 0x02;
// Compile-time constant — evaluated at compilation, zero runtime cost.
const PHYSICAL_STATUS_1000M: u32 = 0x10;
// Compile-time constant — evaluated at compilation, zero runtime cost.
const PHYSICAL_STATUS_100M: u32 = 0x08;
// Compile-time constant — evaluated at compilation, zero runtime cost.
const PHYSICAL_STATUS_10M: u32 = 0x04;

// ============================================================================
// Descriptor Format (C+ mode, 16 bytes each)
// ============================================================================

const NUMBER_RECEIVE_DESCRIPTOR: usize = 64;
// Compile-time constant — evaluated at compilation, zero runtime cost.
const NUMBER_TRANSMIT_DESCRIPTOR: usize = 64;
// Compile-time constant — evaluated at compilation, zero runtime cost.
const RECEIVE_BUFFER_SIZE: usize = 2048;

// Descriptor flags (first u32: opts1)
const DESCRIPTOR_OWN: u32  = 1 << 31;  // Owned by NIC
const DESCRIPTOR_EOR: u32  = 1 << 30;  // End of Ring
const DESCRIPTOR_FILESYSTEM: u32   = 1 << 29;  // First Segment
const DESCRIPTOR_LS: u32   = 1 << 28;  // Last Segment

/// RTL8169 C+ mode descriptor (16 bytes, 256-byte aligned ring recommended)
#[repr(C, align(16))]
// #[derive] — auto-generates trait implementations at compile time.
#[derive(Clone, Copy)]
struct Descriptor {
    opts1: u32,  // OWN | EOR | FS | LS | length
    opts2: u32,  // VLAN tag, offload flags
    buffer_lo: u32, // buffer physical address low
    buffer_hi: u32, // buffer physical address high
}

// Trait implementation — fulfills a behavioral contract.
impl Default for Descriptor {
    fn default() -> Self {
        Self {
            opts1: 0,
            opts2: 0,
            buffer_lo: 0,
            buffer_hi: 0,
        }
    }
}

// ============================================================================
// RTL8169 Driver
// ============================================================================

pub struct Rtl8169Driver {
    status: DriverStatus,
    mmio_base: u64,
    mac: [u8; 6],

    // Descriptor rings
    receive_descs: Vec<Descriptor>,
    transmit_descs: Vec<Descriptor>,
    receive_buffers: Vec<Vec<u8>>,
    transmit_buffers: Vec<Vec<u8>>,

    // Ring indices
    receive_cur: usize,
    transmit_cur: usize,

    // Statistics
    transmit_packets: AtomicU64,
    receive_packets: AtomicU64,
    transmit_bytes: AtomicU64,
    receive_bytes: AtomicU64,
    transmit_errors: AtomicU64,
    receive_errors: AtomicU64,

    // State
    link_up: AtomicBool,
    initialized: AtomicBool,
}

// Implementation block — defines methods for the type above.
impl Rtl8169Driver {
        // Public function — callable from other modules.
pub fn new() -> Self {
        Self {
            status: DriverStatus::Unloaded,
            mmio_base: 0,
            mac: [0x52, 0x54, 0x00, 0x81, 0x69, 0x00],
            receive_descs: Vec::new(),
            transmit_descs: Vec::new(),
            receive_buffers: Vec::new(),
            transmit_buffers: Vec::new(),
            receive_cur: 0,
            transmit_cur: 0,
            transmit_packets: AtomicU64::new(0),
            receive_packets: AtomicU64::new(0),
            transmit_bytes: AtomicU64::new(0),
            receive_bytes: AtomicU64::new(0),
            transmit_errors: AtomicU64::new(0),
            receive_errors: AtomicU64::new(0),
            link_up: AtomicBool::new(false),
            initialized: AtomicBool::new(false),
        }
    }

    // ---- MMIO register helpers ----

    fn read8(&self, offset: u32) -> u8 {
        if self.mmio_base == 0 { return 0; }
        let address = (self.mmio_base + offset as u64) as *// Compile-time constant — evaluated at compilation, zero runtime cost.
const u8;
                // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe { read_volatile(address) }
    }

    fn write8(&self, offset: u32, value: u8) {
        if self.mmio_base == 0 { return; }
        let address = (self.mmio_base + offset as u64) as *mut u8;
                // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe { write_volatile(address, value); }
    }

    fn read16(&self, offset: u32) -> u16 {
        if self.mmio_base == 0 { return 0; }
        let address = (self.mmio_base + offset as u64) as *// Compile-time constant — evaluated at compilation, zero runtime cost.
const u16;
                // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe { read_volatile(address) }
    }

    fn write16(&self, offset: u32, value: u16) {
        if self.mmio_base == 0 { return; }
        let address = (self.mmio_base + offset as u64) as *mut u16;
                // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe { write_volatile(address, value); }
    }

    fn read32(&self, offset: u32) -> u32 {
        if self.mmio_base == 0 { return 0; }
        let address = (self.mmio_base + offset as u64) as *// Compile-time constant — evaluated at compilation, zero runtime cost.
const u32;
                // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe { read_volatile(address) }
    }

    fn write32(&self, offset: u32, value: u32) {
        if self.mmio_base == 0 { return; }
        let address = (self.mmio_base + offset as u64) as *mut u32;
                // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe { write_volatile(address, value); }
    }

    /// Convert virtual address to physical (HHDM)
    fn virt_to_physical(virt: u64) -> u64 {
                // Compile-time constant — evaluated at compilation, zero runtime cost.
const HHDM_OFFSET: u64 = 0xFFFF_8000_0000_0000;
        if virt >= HHDM_OFFSET { virt - HHDM_OFFSET } else { virt }
    }

    /// Software reset — set CMD.Reset, wait for it to clear
    fn reset(&self) {
        crate::log_debug!("[RTL8169] Resetting controller...");

        self.write8(REGISTER_COMMAND, COMMAND_RESET);

        // Wait up to 100ms for reset to complete
        for _ in 0..10_000 {
            if self.read8(REGISTER_COMMAND) & COMMAND_RESET == 0 {
                crate::log_debug!("[RTL8169] Reset complete");
                return;
            }
            for _ in 0..1000 { core::hint::spin_loop(); }
        }

        crate::log_warn!("[RTL8169] Reset timeout — continuing anyway");
    }

    /// Read MAC address from registers 0x00-0x05
    fn read_mac(&mut self) {
        let lo = self.read32(REGISTER_MAC0);
        let hi = self.read32(REGISTER_MAC4);

        self.mac[0] = (lo >> 0) as u8;
        self.mac[1] = (lo >> 8) as u8;
        self.mac[2] = (lo >> 16) as u8;
        self.mac[3] = (lo >> 24) as u8;
        self.mac[4] = (hi >> 0) as u8;
        self.mac[5] = (hi >> 8) as u8;

        // Fallback if MAC is all zeros (QEMU default)
        if self.mac == [0; 6] {
            self.mac = [0x52, 0x54, 0x00, 0x12, 0x81, 0x69];
        }
    }

    /// Unlock 93C46 config registers
    fn unlock_config(&self) {
        self.write8(REGISTER_9346CR, CONFIGURATION_9346_UNLOCK);
    }

    /// Lock 93C46 config registers
    fn lock_config(&self) {
        self.write8(REGISTER_9346CR, CONFIGURATION_9346_LOCK);
    }

    /// Initialize RX descriptor ring and buffers
    fn initialize_receive(&mut self) {
        crate::log_debug!("[RTL8169] Initializing RX ring ({} descriptors)", NUMBER_RECEIVE_DESCRIPTOR);

        self.receive_descs = vec![Descriptor::default(); NUMBER_RECEIVE_DESCRIPTOR];
        self.receive_buffers = Vec::with_capacity(NUMBER_RECEIVE_DESCRIPTOR);

        for i in 0..NUMBER_RECEIVE_DESCRIPTOR {
            let buffer = vec![0u8; RECEIVE_BUFFER_SIZE];
            let physical = Self::virt_to_physical(buffer.as_pointer() as u64);

            let mut flags = DESCRIPTOR_OWN | (RECEIVE_BUFFER_SIZE as u32 & 0x3FFF);
            if i == NUMBER_RECEIVE_DESCRIPTOR - 1 {
                flags |= DESCRIPTOR_EOR; // Mark end of ring
            }

            self.receive_descs[i].opts1 = flags;
            self.receive_descs[i].opts2 = 0;
            self.receive_descs[i].buffer_lo = physical as u32;
            self.receive_descs[i].buffer_hi = (physical >> 32) as u32;

            self.receive_buffers.push(buffer);
        }

        // Write RX descriptor ring address
        let ring_physical = Self::virt_to_physical(self.receive_descs.as_pointer() as u64);
        self.write32(REGISTER_RDSAR, ring_physical as u32);
        self.write32(REGISTER_RDSAR_HI, (ring_physical >> 32) as u32);

        self.receive_cur = 0;
    }

    /// Initialize TX descriptor ring and buffers
    fn initialize_transmit(&mut self) {
        crate::log_debug!("[RTL8169] Initializing TX ring ({} descriptors)", NUMBER_TRANSMIT_DESCRIPTOR);

        self.transmit_descs = vec![Descriptor::default(); NUMBER_TRANSMIT_DESCRIPTOR];
        self.transmit_buffers = Vec::with_capacity(NUMBER_TRANSMIT_DESCRIPTOR);

        for i in 0..NUMBER_TRANSMIT_DESCRIPTOR {
            let buffer = vec![0u8; RECEIVE_BUFFER_SIZE];

            let mut flags = 0u32;
            if i == NUMBER_TRANSMIT_DESCRIPTOR - 1 {
                flags |= DESCRIPTOR_EOR; // Mark end of ring
            }

            self.transmit_descs[i].opts1 = flags;
            self.transmit_descs[i].opts2 = 0;

            let physical = Self::virt_to_physical(buffer.as_pointer() as u64);
            self.transmit_descs[i].buffer_lo = physical as u32;
            self.transmit_descs[i].buffer_hi = (physical >> 32) as u32;

            self.transmit_buffers.push(buffer);
        }

        // Write TX descriptor ring address
        let ring_physical = Self::virt_to_physical(self.transmit_descs.as_pointer() as u64);
        self.write32(REGISTER_TNPDS, ring_physical as u32);
        self.write32(REGISTER_TNPDS_HI, (ring_physical >> 32) as u32);

        self.transmit_cur = 0;
    }

    /// Configure and enable the NIC
    fn enable(&mut self) {
        // Unlock config
        self.unlock_config();

        // C+ mode: enable PCI multiple read/write, checksum offload
        let cpcr = CPCR_PCI_MUL_RW | CPCR_RECEIVE_CHKSUM;
        self.write16(REGISTER_CPCR, cpcr);

        // Set RX max packet size
        self.write16(REGISTER_RECEIVE_MAXIMUM_SIZE, RECEIVE_BUFFER_SIZE as u16);

        // TX config: standard IFG, max DMA burst
        self.write32(REGISTER_TRANSMIT_CONFIG, TRANSMIT_CONFIGURATION_IFG | TRANSMIT_CONFIGURATION_DMA_BURST);

        // RX config: accept broadcast + physical match + multicast, max DMA burst
        let rxcfg = RECEIVE_CONFIGURATION_APM | RECEIVE_CONFIGURATION_AB | RECEIVE_CONFIGURATION_AM | RECEIVE_CONFIGURATION_DMA_BURST | RECEIVE_CONFIGURATION_NO_THRESHOLD;
        self.write32(REGISTER_RECEIVE_CONFIG, rxcfg);

        // Set early TX threshold
        self.write8(REGISTER_ETH_TRANSMIT_EARLY, 0x3F);

        // Lock config
        self.lock_config();

        // Enable RX and TX
        self.write8(REGISTER_COMMAND, COMMAND_RECEIVE_ENABLE | COMMAND_TRANSMIT_ENABLE);

        // Enable interrupts (all relevant)
        self.write16(REGISTER_IMR, INT_ALL);

        crate::log_debug!("[RTL8169] Controller enabled (RX+TX)");
    }

    /// Check and update link status
    fn check_link(&mut self) {
        let physical = self.read32(REGISTER_PHYSICAL_STATUS);
        let up = physical & PHYSICAL_STATUS_LINK != 0;
        self.link_up.store(up, Ordering::SeqCst);

        if up {
            let speed = if physical & PHYSICAL_STATUS_1000M != 0 {
                1000
            } else if physical & PHYSICAL_STATUS_100M != 0 {
                100
            } else {
                10
            };
            crate::log!("[RTL8169] Link up at {} Mbps", speed);
        }
    }
}

// ============================================================================
// Driver trait implementation
// ============================================================================

impl Driver for Rtl8169Driver {
    fn information(&self) -> &DriverInformation {
        &DRIVER_INFORMATION
    }

    fn probe(&mut self, pci_device: &PciDevice) -> Result<(), &'static str> {
        self.status = DriverStatus::Loading;

        crate::log!("[RTL8169] Probing {:04X}:{:04X}", pci_device.vendor_id, pci_device.device_id);

        // Get BAR0 (MMIO)
        let bar0 = pci_device.bar_address(0).ok_or("No BAR0")?;
        if bar0 == 0 { return Err("BAR0 is zero"); }

        // Map MMIO (256 bytes is the standard register space)
        const RTL8169_MMIO_SIZE: usize = 4096;
        self.mmio_base = crate::memory::map_mmio(bar0, RTL8169_MMIO_SIZE)
            .map_error(|e| {
                crate::serial_println!("[RTL8169] map_mmio failed: {}", e);
                "Failed to map RTL8169 MMIO"
            })?;

        crate::log_debug!("[RTL8169] MMIO: phys={:#x} virt={:#x}", bar0, self.mmio_base);

        // Reset
        self.reset();

        // Read MAC address
        self.read_mac();

        // Initialize descriptor rings
        self.initialize_receive();
        self.initialize_transmit();

        // Configure and enable
        self.enable();

        // Check link
        self.check_link();

        // If PHY doesn't report link immediately, assume it's up (QEMU quirk)
        if !self.link_up.load(Ordering::Relaxed) {
            crate::log_warn!("[RTL8169] Link not detected — assuming up (QEMU mode)");
            self.link_up.store(true, Ordering::SeqCst);
        }

        self.initialized.store(true, Ordering::SeqCst);
        self.status = DriverStatus::Running;

        crate::log!("[RTL8169] MAC: {:02X}:{:02X}:{:02X}:{:02X}:{:02X}:{:02X}",
            self.mac[0], self.mac[1], self.mac[2], self.mac[3], self.mac[4], self.mac[5]);

        Ok(())
    }

    fn start(&mut self) -> Result<(), &'static str> {
        self.status = DriverStatus::Running;
        Ok(())
    }

    fn stop(&mut self) -> Result<(), &'static str> {
        // Disable RX + TX
        self.write8(REGISTER_COMMAND, 0);
        // Mask all interrupts
        self.write16(REGISTER_IMR, 0);
        self.status = DriverStatus::Suspended;
        Ok(())
    }

    fn status(&self) -> DriverStatus {
        self.status
    }
}

// ============================================================================
// NetworkDriver trait implementation
// ============================================================================

impl NetworkDriver for Rtl8169Driver {
    fn mac_address(&self) -> [u8; 6] {
        self.mac
    }

    fn link_up(&self) -> bool {
        if self.mmio_base != 0 {
            self.read32(REGISTER_PHYSICAL_STATUS) & PHYSICAL_STATUS_LINK != 0
        } else {
            self.link_up.load(Ordering::Relaxed)
        }
    }

    fn link_speed(&self) -> u32 {
        if self.mmio_base == 0 { return 0; }
        let physical = self.read32(REGISTER_PHYSICAL_STATUS);
        if physical & PHYSICAL_STATUS_1000M != 0 { 1000 }
        else if physical & PHYSICAL_STATUS_100M != 0 { 100 }
        else if physical & PHYSICAL_STATUS_10M != 0 { 10 }
        else { 0 }
    }

    fn send(&mut self, data: &[u8]) -> Result<(), &'static str> {
        if !self.initialized.load(Ordering::Relaxed) {
            return Err("Driver not initialized");
        }
        if data.len() > RECEIVE_BUFFER_SIZE { return Err("Packet too large"); }
        if data.len() < 14 { return Err("Packet too small"); }

        let index = self.transmit_cur;

        // Wait for descriptor to become available (OWN bit cleared by NIC)
        let mut timeout = 10_000;
        while self.transmit_descs[index].opts1 & DESCRIPTOR_OWN != 0 {
            timeout -= 1;
            if timeout == 0 {
                self.transmit_errors.fetch_add(1, Ordering::Relaxed);
                return Err("TX timeout — descriptor still owned by NIC");
            }
            core::hint::spin_loop();
        }

        // Copy packet data to TX buffer
        let buffer = &mut self.transmit_buffers[index];
        buffer[..data.len()].copy_from_slice(data);

        // Update descriptor physical address (buffer may have moved)
        let physical = Self::virt_to_physical(buffer.as_pointer() as u64);
        self.transmit_descs[index].buffer_lo = physical as u32;
        self.transmit_descs[index].buffer_hi = (physical >> 32) as u32;

        // Set descriptor flags: OWN + FS + LS + length (+ EOR if last)
        let mut flags = DESCRIPTOR_OWN | DESCRIPTOR_FILESYSTEM | DESCRIPTOR_LS | (data.len() as u32 & 0x3FFF);
        if index == NUMBER_TRANSMIT_DESCRIPTOR - 1 {
            flags |= DESCRIPTOR_EOR;
        }
        self.transmit_descs[index].opts1 = flags;
        self.transmit_descs[index].opts2 = 0;

        // Notify NIC: poll TX normal priority queue
        self.write8(REGISTER_TPPOLL, TPPOLL_NPQ);

        // Advance ring index
        self.transmit_cur = (self.transmit_cur + 1) % NUMBER_TRANSMIT_DESCRIPTOR;

        self.transmit_packets.fetch_add(1, Ordering::Relaxed);
        self.transmit_bytes.fetch_add(data.len() as u64, Ordering::Relaxed);

        Ok(())
    }

    fn receive(&mut self) -> Option<Vec<u8>> {
        if !self.initialized.load(Ordering::Relaxed) { return None; }

        let index = self.receive_cur;
        let opts1 = self.receive_descs[index].opts1;

        // Check if NIC has released this descriptor (OWN bit cleared)
        if opts1 & DESCRIPTOR_OWN != 0 {
            return None;
        }

        // Check for first+last segment (we only support single-segment packets)
        if opts1 & (DESCRIPTOR_FILESYSTEM | DESCRIPTOR_LS) != (DESCRIPTOR_FILESYSTEM | DESCRIPTOR_LS) {
            // Multi-descriptor packet — reclaim and skip
            self.receive_errors.fetch_add(1, Ordering::Relaxed);
            self.reclaim_receive(index);
            return None;
        }

        // Extract packet length (bits 0..13, minus 4 for CRC)
        let length = (opts1 & 0x3FFF) as usize;
        if length < 4 || length > RECEIVE_BUFFER_SIZE {
            self.receive_errors.fetch_add(1, Ordering::Relaxed);
            self.reclaim_receive(index);
            return None;
        }

        let packet_length = length - 4; // Strip CRC
        if packet_length == 0 {
            self.reclaim_receive(index);
            return None;
        }

        // Copy packet data
        let packet = self.receive_buffers[index][..packet_length].to_vec();

        // Reclaim descriptor
        self.reclaim_receive(index);

        self.receive_packets.fetch_add(1, Ordering::Relaxed);
        self.receive_bytes.fetch_add(packet_length as u64, Ordering::Relaxed);

        Some(packet)
    }

    fn poll(&mut self) {
        if !self.initialized.load(Ordering::Relaxed) { return; }

        // Read and acknowledge interrupt status
        let interrupt_handler = self.read16(REGISTER_INTERRUPT_HANDLER);
        if interrupt_handler != 0 {
            self.write16(REGISTER_INTERRUPT_HANDLER, interrupt_handler); // Clear by writing back
        }

        // Update link status on link change
        if interrupt_handler & INT_LINK_CHG != 0 {
            let physical = self.read32(REGISTER_PHYSICAL_STATUS);
            self.link_up.store(physical & PHYSICAL_STATUS_LINK != 0, Ordering::SeqCst);
        }
    }

    fn stats(&self) -> NetStats {
        NetStats {
            transmit_packets: self.transmit_packets.load(Ordering::Relaxed),
            receive_packets: self.receive_packets.load(Ordering::Relaxed),
            transmit_bytes: self.transmit_bytes.load(Ordering::Relaxed),
            receive_bytes: self.receive_bytes.load(Ordering::Relaxed),
            transmit_errors: self.transmit_errors.load(Ordering::Relaxed),
            receive_errors: self.receive_errors.load(Ordering::Relaxed),
            transmit_dropped: 0,
            receive_dropped: 0,
        }
    }

    fn set_promiscuous(&mut self, enabled: bool) -> Result<(), &'static str> {
        if !self.initialized.load(Ordering::Relaxed) { return Err("Not initialized"); }
        let mut rxcfg = self.read32(REGISTER_RECEIVE_CONFIG);
        if enabled {
            rxcfg |= RECEIVE_CONFIGURATION_AAP; // Accept all packets
        } else {
            rxcfg &= !RECEIVE_CONFIGURATION_AAP;
        }
        self.write32(REGISTER_RECEIVE_CONFIG, rxcfg);
        Ok(())
    }
}

// Implementation block — defines methods for the type above.
impl Rtl8169Driver {
    /// Reclaim an RX descriptor back to the NIC
    fn reclaim_receive(&mut self, index: usize) {
        let mut flags = DESCRIPTOR_OWN | (RECEIVE_BUFFER_SIZE as u32 & 0x3FFF);
        if index == NUMBER_RECEIVE_DESCRIPTOR - 1 {
            flags |= DESCRIPTOR_EOR;
        }
        self.receive_descs[index].opts1 = flags;
        self.receive_descs[index].opts2 = 0;
        self.receive_cur = (self.receive_cur + 1) % NUMBER_RECEIVE_DESCRIPTOR;
    }
}

// ============================================================================
// Driver Info & Registration
// ============================================================================

const DRIVER_INFORMATION: DriverInformation = DriverInformation {
    name: "rtl8169",
    version: "1.0.0",
    author: "TrustOS Team",
    category: DriverCategory::Network,
    vendor_ids: &[
        (0x10EC, 0x8169),  // RTL8169
        (0x10EC, 0x8168),  // RTL8168/8111
        (0x10EC, 0x8161),  // RTL8169SC
        (0x10EC, 0x8136),  // RTL8101E/8102E
    ],
};

// Public function — callable from other modules.
pub fn register() {
    crate::drivers::register(DRIVER_INFORMATION, || {
        Box::new(Rtl8169Driver::new())
    });
    crate::drivers::net::register_net_driver(DRIVER_INFORMATION, || {
        Box::new(Rtl8169Driver::new())
    });
}

/// Check if the RTL8169 driver is initialized and active
pub fn is_initialized() -> bool {
    // Check if the active network driver is an RTL8169
    crate::drivers::net::has_driver()
}
