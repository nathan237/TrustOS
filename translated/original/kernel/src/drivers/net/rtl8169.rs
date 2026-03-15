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
use crate::drivers::{Driver, DriverInfo, DriverStatus, DriverCategory};
use crate::pci::PciDevice;

// ============================================================================
// RTL8169 Register Offsets
// ============================================================================

const REG_MAC0: u32       = 0x00;  // MAC address bytes 0-3
const REG_MAC4: u32       = 0x04;  // MAC address bytes 4-5
const REG_TNPDS: u32      = 0x20;  // TX Normal Priority Descriptors (lo)
const REG_TNPDS_HI: u32   = 0x24;  // TX Normal Priority Descriptors (hi)
const REG_CMD: u32         = 0x37;  // Command register (8-bit)
const REG_TPPOLL: u32      = 0x38;  // TX Priority Polling (8-bit)
const REG_IMR: u32         = 0x3C;  // Interrupt Mask Register (16-bit)
const REG_ISR: u32         = 0x3E;  // Interrupt Status Register (16-bit)
const REG_TX_CONFIG: u32   = 0x40;  // TX Configuration
const REG_RX_CONFIG: u32   = 0x44;  // RX Configuration
const REG_MPC: u32         = 0x4C;  // Missed Packet Counter
const REG_9346CR: u32      = 0x50;  // 93C46 Command Register (8-bit)
const REG_CONFIG1: u32     = 0x52;  // Configuration 1
const REG_PHY_STATUS: u32  = 0x6C;  // PHY Status
const REG_RX_MAX_SIZE: u32 = 0xDA;  // RX Max Packet Size (16-bit)
const REG_CPCR: u32        = 0xE0;  // C+ Command Register (16-bit)
const REG_RDSAR: u32       = 0xE4;  // RX Descriptor Start Address (lo)
const REG_RDSAR_HI: u32    = 0xE8;  // RX Descriptor Start Address (hi)
const REG_ETH_TX_EARLY: u32 = 0xEC; // Early TX threshold (8-bit)

// ============================================================================
// Register Bit Definitions
// ============================================================================

// CMD register (0x37)
const CMD_RESET: u8 = 0x10;
const CMD_RX_ENABLE: u8 = 0x08;
const CMD_TX_ENABLE: u8 = 0x04;

// TPPOLL register (0x38)
const TPPOLL_NPQ: u8 = 0x40;  // Normal Priority Queue polling

// Interrupt bits (IMR/ISR)
const INT_ROK: u16 = 0x0001;    // RX OK
const INT_TOK: u16 = 0x0004;    // TX OK
const INT_LINK_CHG: u16 = 0x0020; // Link change
const INT_RX_OVERFLOW: u16 = 0x0010;
const INT_ALL: u16 = INT_ROK | INT_TOK | INT_LINK_CHG | INT_RX_OVERFLOW;

// TX Config
const TX_CFG_IFG: u32 = 0x03 << 24;  // Inter-frame gap (standard)
const TX_CFG_DMA_BURST: u32 = 0x07 << 8; // max DMA burst (unlimited)

// RX Config
const RX_CFG_AAP: u32 = 1 << 0;   // Accept All Packets
const RX_CFG_APM: u32 = 1 << 1;   // Accept Physical Match
const RX_CFG_AM: u32  = 1 << 2;   // Accept Multicast
const RX_CFG_AB: u32  = 1 << 3;   // Accept Broadcast
const RX_CFG_WRAP: u32 = 1 << 7;  // No wrap (not used in C+ mode)
const RX_CFG_DMA_BURST: u32 = 0x07 << 8; // Max DMA burst
const RX_CFG_NO_THRESHOLD: u32 = 0x07 << 13; // No FIFO threshold

// C+ Command Register
const CPCR_RX_VLAN: u16 = 1 << 6;
const CPCR_RX_CHKSUM: u16 = 1 << 5;
const CPCR_PCI_MUL_RW: u16 = 1 << 3;

// 93C46 Command Register (unlock/lock config)
const CFG_9346_UNLOCK: u8 = 0xC0;
const CFG_9346_LOCK: u8   = 0x00;

// PHY Status register (0x6C)
const PHY_STATUS_LINK: u32 = 0x02;
const PHY_STATUS_1000M: u32 = 0x10;
const PHY_STATUS_100M: u32 = 0x08;
const PHY_STATUS_10M: u32 = 0x04;

// ============================================================================
// Descriptor Format (C+ mode, 16 bytes each)
// ============================================================================

const NUM_RX_DESC: usize = 64;
const NUM_TX_DESC: usize = 64;
const RX_BUFFER_SIZE: usize = 2048;

// Descriptor flags (first u32: opts1)
const DESC_OWN: u32  = 1 << 31;  // Owned by NIC
const DESC_EOR: u32  = 1 << 30;  // End of Ring
const DESC_FS: u32   = 1 << 29;  // First Segment
const DESC_LS: u32   = 1 << 28;  // Last Segment

/// RTL8169 C+ mode descriptor (16 bytes, 256-byte aligned ring recommended)
#[repr(C, align(16))]
#[derive(Clone, Copy)]
struct Descriptor {
    opts1: u32,  // OWN | EOR | FS | LS | length
    opts2: u32,  // VLAN tag, offload flags
    buf_lo: u32, // buffer physical address low
    buf_hi: u32, // buffer physical address high
}

impl Default for Descriptor {
    fn default() -> Self {
        Self {
            opts1: 0,
            opts2: 0,
            buf_lo: 0,
            buf_hi: 0,
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
    rx_descs: Vec<Descriptor>,
    tx_descs: Vec<Descriptor>,
    rx_buffers: Vec<Vec<u8>>,
    tx_buffers: Vec<Vec<u8>>,

    // Ring indices
    rx_cur: usize,
    tx_cur: usize,

    // Statistics
    tx_packets: AtomicU64,
    rx_packets: AtomicU64,
    tx_bytes: AtomicU64,
    rx_bytes: AtomicU64,
    tx_errors: AtomicU64,
    rx_errors: AtomicU64,

    // State
    link_up: AtomicBool,
    initialized: AtomicBool,
}

impl Rtl8169Driver {
    pub fn new() -> Self {
        Self {
            status: DriverStatus::Unloaded,
            mmio_base: 0,
            mac: [0x52, 0x54, 0x00, 0x81, 0x69, 0x00],
            rx_descs: Vec::new(),
            tx_descs: Vec::new(),
            rx_buffers: Vec::new(),
            tx_buffers: Vec::new(),
            rx_cur: 0,
            tx_cur: 0,
            tx_packets: AtomicU64::new(0),
            rx_packets: AtomicU64::new(0),
            tx_bytes: AtomicU64::new(0),
            rx_bytes: AtomicU64::new(0),
            tx_errors: AtomicU64::new(0),
            rx_errors: AtomicU64::new(0),
            link_up: AtomicBool::new(false),
            initialized: AtomicBool::new(false),
        }
    }

    // ---- MMIO register helpers ----

    fn read8(&self, offset: u32) -> u8 {
        if self.mmio_base == 0 { return 0; }
        let addr = (self.mmio_base + offset as u64) as *const u8;
        unsafe { read_volatile(addr) }
    }

    fn write8(&self, offset: u32, val: u8) {
        if self.mmio_base == 0 { return; }
        let addr = (self.mmio_base + offset as u64) as *mut u8;
        unsafe { write_volatile(addr, val); }
    }

    fn read16(&self, offset: u32) -> u16 {
        if self.mmio_base == 0 { return 0; }
        let addr = (self.mmio_base + offset as u64) as *const u16;
        unsafe { read_volatile(addr) }
    }

    fn write16(&self, offset: u32, val: u16) {
        if self.mmio_base == 0 { return; }
        let addr = (self.mmio_base + offset as u64) as *mut u16;
        unsafe { write_volatile(addr, val); }
    }

    fn read32(&self, offset: u32) -> u32 {
        if self.mmio_base == 0 { return 0; }
        let addr = (self.mmio_base + offset as u64) as *const u32;
        unsafe { read_volatile(addr) }
    }

    fn write32(&self, offset: u32, val: u32) {
        if self.mmio_base == 0 { return; }
        let addr = (self.mmio_base + offset as u64) as *mut u32;
        unsafe { write_volatile(addr, val); }
    }

    /// Convert virtual address to physical (HHDM)
    fn virt_to_phys(virt: u64) -> u64 {
        const HHDM_OFFSET: u64 = 0xFFFF_8000_0000_0000;
        if virt >= HHDM_OFFSET { virt - HHDM_OFFSET } else { virt }
    }

    /// Software reset — set CMD.Reset, wait for it to clear
    fn reset(&self) {
        crate::log_debug!("[RTL8169] Resetting controller...");

        self.write8(REG_CMD, CMD_RESET);

        // Wait up to 100ms for reset to complete
        for _ in 0..10_000 {
            if self.read8(REG_CMD) & CMD_RESET == 0 {
                crate::log_debug!("[RTL8169] Reset complete");
                return;
            }
            for _ in 0..1000 { core::hint::spin_loop(); }
        }

        crate::log_warn!("[RTL8169] Reset timeout — continuing anyway");
    }

    /// Read MAC address from registers 0x00-0x05
    fn read_mac(&mut self) {
        let lo = self.read32(REG_MAC0);
        let hi = self.read32(REG_MAC4);

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
        self.write8(REG_9346CR, CFG_9346_UNLOCK);
    }

    /// Lock 93C46 config registers
    fn lock_config(&self) {
        self.write8(REG_9346CR, CFG_9346_LOCK);
    }

    /// Initialize RX descriptor ring and buffers
    fn init_rx(&mut self) {
        crate::log_debug!("[RTL8169] Initializing RX ring ({} descriptors)", NUM_RX_DESC);

        self.rx_descs = vec![Descriptor::default(); NUM_RX_DESC];
        self.rx_buffers = Vec::with_capacity(NUM_RX_DESC);

        for i in 0..NUM_RX_DESC {
            let buffer = vec![0u8; RX_BUFFER_SIZE];
            let phys = Self::virt_to_phys(buffer.as_ptr() as u64);

            let mut flags = DESC_OWN | (RX_BUFFER_SIZE as u32 & 0x3FFF);
            if i == NUM_RX_DESC - 1 {
                flags |= DESC_EOR; // Mark end of ring
            }

            self.rx_descs[i].opts1 = flags;
            self.rx_descs[i].opts2 = 0;
            self.rx_descs[i].buf_lo = phys as u32;
            self.rx_descs[i].buf_hi = (phys >> 32) as u32;

            self.rx_buffers.push(buffer);
        }

        // Write RX descriptor ring address
        let ring_phys = Self::virt_to_phys(self.rx_descs.as_ptr() as u64);
        self.write32(REG_RDSAR, ring_phys as u32);
        self.write32(REG_RDSAR_HI, (ring_phys >> 32) as u32);

        self.rx_cur = 0;
    }

    /// Initialize TX descriptor ring and buffers
    fn init_tx(&mut self) {
        crate::log_debug!("[RTL8169] Initializing TX ring ({} descriptors)", NUM_TX_DESC);

        self.tx_descs = vec![Descriptor::default(); NUM_TX_DESC];
        self.tx_buffers = Vec::with_capacity(NUM_TX_DESC);

        for i in 0..NUM_TX_DESC {
            let buffer = vec![0u8; RX_BUFFER_SIZE];

            let mut flags = 0u32;
            if i == NUM_TX_DESC - 1 {
                flags |= DESC_EOR; // Mark end of ring
            }

            self.tx_descs[i].opts1 = flags;
            self.tx_descs[i].opts2 = 0;

            let phys = Self::virt_to_phys(buffer.as_ptr() as u64);
            self.tx_descs[i].buf_lo = phys as u32;
            self.tx_descs[i].buf_hi = (phys >> 32) as u32;

            self.tx_buffers.push(buffer);
        }

        // Write TX descriptor ring address
        let ring_phys = Self::virt_to_phys(self.tx_descs.as_ptr() as u64);
        self.write32(REG_TNPDS, ring_phys as u32);
        self.write32(REG_TNPDS_HI, (ring_phys >> 32) as u32);

        self.tx_cur = 0;
    }

    /// Configure and enable the NIC
    fn enable(&mut self) {
        // Unlock config
        self.unlock_config();

        // C+ mode: enable PCI multiple read/write, checksum offload
        let cpcr = CPCR_PCI_MUL_RW | CPCR_RX_CHKSUM;
        self.write16(REG_CPCR, cpcr);

        // Set RX max packet size
        self.write16(REG_RX_MAX_SIZE, RX_BUFFER_SIZE as u16);

        // TX config: standard IFG, max DMA burst
        self.write32(REG_TX_CONFIG, TX_CFG_IFG | TX_CFG_DMA_BURST);

        // RX config: accept broadcast + physical match + multicast, max DMA burst
        let rxcfg = RX_CFG_APM | RX_CFG_AB | RX_CFG_AM | RX_CFG_DMA_BURST | RX_CFG_NO_THRESHOLD;
        self.write32(REG_RX_CONFIG, rxcfg);

        // Set early TX threshold
        self.write8(REG_ETH_TX_EARLY, 0x3F);

        // Lock config
        self.lock_config();

        // Enable RX and TX
        self.write8(REG_CMD, CMD_RX_ENABLE | CMD_TX_ENABLE);

        // Enable interrupts (all relevant)
        self.write16(REG_IMR, INT_ALL);

        crate::log_debug!("[RTL8169] Controller enabled (RX+TX)");
    }

    /// Check and update link status
    fn check_link(&mut self) {
        let phy = self.read32(REG_PHY_STATUS);
        let up = phy & PHY_STATUS_LINK != 0;
        self.link_up.store(up, Ordering::SeqCst);

        if up {
            let speed = if phy & PHY_STATUS_1000M != 0 {
                1000
            } else if phy & PHY_STATUS_100M != 0 {
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
    fn info(&self) -> &DriverInfo {
        &DRIVER_INFO
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
            .map_err(|e| {
                crate::serial_println!("[RTL8169] map_mmio failed: {}", e);
                "Failed to map RTL8169 MMIO"
            })?;

        crate::log_debug!("[RTL8169] MMIO: phys={:#x} virt={:#x}", bar0, self.mmio_base);

        // Reset
        self.reset();

        // Read MAC address
        self.read_mac();

        // Initialize descriptor rings
        self.init_rx();
        self.init_tx();

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
        self.write8(REG_CMD, 0);
        // Mask all interrupts
        self.write16(REG_IMR, 0);
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
            self.read32(REG_PHY_STATUS) & PHY_STATUS_LINK != 0
        } else {
            self.link_up.load(Ordering::Relaxed)
        }
    }

    fn link_speed(&self) -> u32 {
        if self.mmio_base == 0 { return 0; }
        let phy = self.read32(REG_PHY_STATUS);
        if phy & PHY_STATUS_1000M != 0 { 1000 }
        else if phy & PHY_STATUS_100M != 0 { 100 }
        else if phy & PHY_STATUS_10M != 0 { 10 }
        else { 0 }
    }

    fn send(&mut self, data: &[u8]) -> Result<(), &'static str> {
        if !self.initialized.load(Ordering::Relaxed) {
            return Err("Driver not initialized");
        }
        if data.len() > RX_BUFFER_SIZE { return Err("Packet too large"); }
        if data.len() < 14 { return Err("Packet too small"); }

        let idx = self.tx_cur;

        // Wait for descriptor to become available (OWN bit cleared by NIC)
        let mut timeout = 10_000;
        while self.tx_descs[idx].opts1 & DESC_OWN != 0 {
            timeout -= 1;
            if timeout == 0 {
                self.tx_errors.fetch_add(1, Ordering::Relaxed);
                return Err("TX timeout — descriptor still owned by NIC");
            }
            core::hint::spin_loop();
        }

        // Copy packet data to TX buffer
        let buffer = &mut self.tx_buffers[idx];
        buffer[..data.len()].copy_from_slice(data);

        // Update descriptor physical address (buffer may have moved)
        let phys = Self::virt_to_phys(buffer.as_ptr() as u64);
        self.tx_descs[idx].buf_lo = phys as u32;
        self.tx_descs[idx].buf_hi = (phys >> 32) as u32;

        // Set descriptor flags: OWN + FS + LS + length (+ EOR if last)
        let mut flags = DESC_OWN | DESC_FS | DESC_LS | (data.len() as u32 & 0x3FFF);
        if idx == NUM_TX_DESC - 1 {
            flags |= DESC_EOR;
        }
        self.tx_descs[idx].opts1 = flags;
        self.tx_descs[idx].opts2 = 0;

        // Notify NIC: poll TX normal priority queue
        self.write8(REG_TPPOLL, TPPOLL_NPQ);

        // Advance ring index
        self.tx_cur = (self.tx_cur + 1) % NUM_TX_DESC;

        self.tx_packets.fetch_add(1, Ordering::Relaxed);
        self.tx_bytes.fetch_add(data.len() as u64, Ordering::Relaxed);

        Ok(())
    }

    fn receive(&mut self) -> Option<Vec<u8>> {
        if !self.initialized.load(Ordering::Relaxed) { return None; }

        let idx = self.rx_cur;
        let opts1 = self.rx_descs[idx].opts1;

        // Check if NIC has released this descriptor (OWN bit cleared)
        if opts1 & DESC_OWN != 0 {
            return None;
        }

        // Check for first+last segment (we only support single-segment packets)
        if opts1 & (DESC_FS | DESC_LS) != (DESC_FS | DESC_LS) {
            // Multi-descriptor packet — reclaim and skip
            self.rx_errors.fetch_add(1, Ordering::Relaxed);
            self.reclaim_rx(idx);
            return None;
        }

        // Extract packet length (bits 0..13, minus 4 for CRC)
        let length = (opts1 & 0x3FFF) as usize;
        if length < 4 || length > RX_BUFFER_SIZE {
            self.rx_errors.fetch_add(1, Ordering::Relaxed);
            self.reclaim_rx(idx);
            return None;
        }

        let pkt_len = length - 4; // Strip CRC
        if pkt_len == 0 {
            self.reclaim_rx(idx);
            return None;
        }

        // Copy packet data
        let packet = self.rx_buffers[idx][..pkt_len].to_vec();

        // Reclaim descriptor
        self.reclaim_rx(idx);

        self.rx_packets.fetch_add(1, Ordering::Relaxed);
        self.rx_bytes.fetch_add(pkt_len as u64, Ordering::Relaxed);

        Some(packet)
    }

    fn poll(&mut self) {
        if !self.initialized.load(Ordering::Relaxed) { return; }

        // Read and acknowledge interrupt status
        let isr = self.read16(REG_ISR);
        if isr != 0 {
            self.write16(REG_ISR, isr); // Clear by writing back
        }

        // Update link status on link change
        if isr & INT_LINK_CHG != 0 {
            let phy = self.read32(REG_PHY_STATUS);
            self.link_up.store(phy & PHY_STATUS_LINK != 0, Ordering::SeqCst);
        }
    }

    fn stats(&self) -> NetStats {
        NetStats {
            tx_packets: self.tx_packets.load(Ordering::Relaxed),
            rx_packets: self.rx_packets.load(Ordering::Relaxed),
            tx_bytes: self.tx_bytes.load(Ordering::Relaxed),
            rx_bytes: self.rx_bytes.load(Ordering::Relaxed),
            tx_errors: self.tx_errors.load(Ordering::Relaxed),
            rx_errors: self.rx_errors.load(Ordering::Relaxed),
            tx_dropped: 0,
            rx_dropped: 0,
        }
    }

    fn set_promiscuous(&mut self, enabled: bool) -> Result<(), &'static str> {
        if !self.initialized.load(Ordering::Relaxed) { return Err("Not initialized"); }
        let mut rxcfg = self.read32(REG_RX_CONFIG);
        if enabled {
            rxcfg |= RX_CFG_AAP; // Accept all packets
        } else {
            rxcfg &= !RX_CFG_AAP;
        }
        self.write32(REG_RX_CONFIG, rxcfg);
        Ok(())
    }
}

impl Rtl8169Driver {
    /// Reclaim an RX descriptor back to the NIC
    fn reclaim_rx(&mut self, idx: usize) {
        let mut flags = DESC_OWN | (RX_BUFFER_SIZE as u32 & 0x3FFF);
        if idx == NUM_RX_DESC - 1 {
            flags |= DESC_EOR;
        }
        self.rx_descs[idx].opts1 = flags;
        self.rx_descs[idx].opts2 = 0;
        self.rx_cur = (self.rx_cur + 1) % NUM_RX_DESC;
    }
}

// ============================================================================
// Driver Info & Registration
// ============================================================================

const DRIVER_INFO: DriverInfo = DriverInfo {
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

pub fn register() {
    crate::drivers::register(DRIVER_INFO, || {
        Box::new(Rtl8169Driver::new())
    });
    crate::drivers::net::register_net_driver(DRIVER_INFO, || {
        Box::new(Rtl8169Driver::new())
    });
}

/// Check if the RTL8169 driver is initialized and active
pub fn is_initialized() -> bool {
    // Check if the active network driver is an RTL8169
    crate::drivers::net::has_driver()
}
