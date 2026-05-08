//! Realtek RTL8139 Network Driver
//!
//! Fully functional driver for RTL8139/RTL8139C/RTL8100 NICs.
//! Uses PIO (port I/O) for register access, ring buffer for RX, 4 TX descriptors.

use alloc::boxed::Box;
use alloc::vec::Vec;
use alloc::vec;
use core::sync::atomic::{AtomicU64, Ordering};

use super::{NetworkDriver, NetStats};
use crate::drivers::{Driver, DriverInfo, DriverStatus, DriverCategory};
use crate::pci::PciDevice;

// ═══════════════════════════════════════════════════════════════════
// RTL8139 Register offsets (from data sheet)
// ═══════════════════════════════════════════════════════════════════
const REG_MAC0:     u16 = 0x00;  // MAC address bytes 0-3
const REG_MAC4:     u16 = 0x04;  // MAC address bytes 4-5
const REG_TX_STATUS0: u16 = 0x10; // TX status descriptor 0 (4 descriptors, +4 each)
const REG_TX_ADDR0: u16 = 0x20;  // TX buffer address descriptor 0 (4 descriptors, +4 each)
const REG_RX_BUF:   u16 = 0x30;  // RX buffer physical address
const REG_CMD:      u16 = 0x37;  // Command register
const REG_RX_BUF_PTR: u16 = 0x38; // Current RX buffer read pointer (CAPR)
const REG_IMR:      u16 = 0x3C;  // Interrupt Mask Register
const REG_ISR:      u16 = 0x3E;  // Interrupt Status Register
const REG_TX_CONFIG: u16 = 0x40; // TX configuration
const REG_RX_CONFIG: u16 = 0x44; // RX configuration
const REG_CONFIG1:  u16 = 0x52;  // Configuration register 1

// Command register bits
const CMD_RX_ENABLE: u8 = 0x08;
const CMD_TX_ENABLE: u8 = 0x04;
const CMD_RESET:     u8 = 0x10;

// Interrupt bits
const INT_RX_OK:      u16 = 0x0001;
const INT_RX_ERR:     u16 = 0x0002;
const INT_TX_OK:      u16 = 0x0004;
const INT_TX_ERR:     u16 = 0x0008;
const INT_RX_OVERFLOW: u16 = 0x0010;

// RX config: Accept broadcast + multicast + physical match + all physical, WRAP, 32K+16 buffer
const RX_CONFIG: u32 = 0x0000_0F0F; // AAM | APM | AM | AB | WRAP | 32K+16 buffer (1=8K+16, use 32K+16 bits)

// TX status bits
const TX_OWN: u32 = 1 << 13;  // DMA transfer completed (OWN cleared = we can write)

// RX header flags
const RX_ROK: u16 = 0x0001;

// Buffer sizes
const RX_BUF_SIZE: usize = 8192 + 16 + 1500; // 8K + 16 wrap + 1500 extra
const TX_BUF_SIZE: usize = 1536;              // Max Ethernet frame
const NUM_TX_DESC: usize = 4;

pub struct Rtl8139Driver {
    status: DriverStatus,
    mac: [u8; 6],
    io_base: u16,
    rx_buffer: Option<Box<[u8]>>,
    tx_buffers: Option<[Box<[u8]>; NUM_TX_DESC]>,
    rx_offset: usize,
    tx_cur: usize,
    stats: NetStats,
}

// Port I/O helpers
#[inline]
fn inb(port: u16) -> u8 {
    let val: u8;
    unsafe { core::arch::asm!("in al, dx", out("al") val, in("dx") port, options(nostack, preserves_flags)); }
    val
}

#[inline]
fn inw(port: u16) -> u16 {
    let val: u16;
    unsafe { core::arch::asm!("in ax, dx", out("ax") val, in("dx") port, options(nostack, preserves_flags)); }
    val
}

#[inline]
fn inl(port: u16) -> u32 {
    let val: u32;
    unsafe { core::arch::asm!("in eax, dx", out("eax") val, in("dx") port, options(nostack, preserves_flags)); }
    val
}

#[inline]
fn outb(port: u16, val: u8) {
    unsafe { core::arch::asm!("out dx, al", in("dx") port, in("al") val, options(nostack, preserves_flags)); }
}

#[inline]
fn outw(port: u16, val: u16) {
    unsafe { core::arch::asm!("out dx, ax", in("dx") port, in("ax") val, options(nostack, preserves_flags)); }
}

#[inline]
fn outl(port: u16, val: u32) {
    unsafe { core::arch::asm!("out dx, eax", in("dx") port, in("eax") val, options(nostack, preserves_flags)); }
}

/// Convert a kernel virtual address to physical address for DMA
fn virt_to_phys(virt: u64) -> u32 {
    let hhdm = crate::memory::hhdm_offset();
    // All heap allocations (Box, Vec) are in HHDM range
    if virt >= hhdm {
        (virt - hhdm) as u32
    } else {
        // Identity-mapped or low address — use as-is
        virt as u32
    }
}

impl Rtl8139Driver {
    pub fn new() -> Self {
        Self {
            status: DriverStatus::Unloaded,
            mac: [0; 6],
            io_base: 0,
            rx_buffer: None,
            tx_buffers: None,
            rx_offset: 0,
            tx_cur: 0,
            stats: NetStats::default(),
        }
    }

    fn read8(&self, reg: u16) -> u8 { inb(self.io_base + reg) }
    fn read16(&self, reg: u16) -> u16 { inw(self.io_base + reg) }
    fn read32(&self, reg: u16) -> u32 { inl(self.io_base + reg) }
    fn write8(&self, reg: u16, val: u8) { outb(self.io_base + reg, val) }
    fn write16(&self, reg: u16, val: u16) { outw(self.io_base + reg, val) }
    fn write32(&self, reg: u16, val: u32) { outl(self.io_base + reg, val) }
}

impl Driver for Rtl8139Driver {
    fn info(&self) -> &DriverInfo {
        &DRIVER_INFO
    }

    fn probe(&mut self, pci_device: &PciDevice) -> Result<(), &'static str> {
        self.status = DriverStatus::Loading;

        // Get I/O base from BAR0 (RTL8139 uses PIO)
        let bar0 = pci_device.bar[0];
        if bar0 & 1 == 0 {
            return Err("RTL8139 BAR0 is not I/O space");
        }
        self.io_base = (bar0 & 0xFFFC) as u16;

        crate::serial_println!("[rtl8139] I/O base: {:#x}", self.io_base);

        // Enable PCI bus mastering + I/O space
        crate::pci::enable_bus_master(pci_device);
        let cmd = crate::pci::config_read16(pci_device.bus, pci_device.device, pci_device.function, 0x04);
        crate::pci::config_write(pci_device.bus, pci_device.device, pci_device.function, 0x04,
            (cmd | 0x05) as u32); // Bus Master + I/O Space

        // Power on (write 0x00 to CONFIG1)
        self.write8(REG_CONFIG1, 0x00);

        // Software reset
        self.write8(REG_CMD, CMD_RESET);
        // Wait for reset to complete (bit clears)
        for _ in 0..10000u32 {
            if self.read8(REG_CMD) & CMD_RESET == 0 { break; }
            core::hint::spin_loop();
        }

        // Read MAC address
        let mac_lo = self.read32(REG_MAC0);
        let mac_hi = self.read16(REG_MAC4);
        self.mac[0] = (mac_lo & 0xFF) as u8;
        self.mac[1] = ((mac_lo >> 8) & 0xFF) as u8;
        self.mac[2] = ((mac_lo >> 16) & 0xFF) as u8;
        self.mac[3] = ((mac_lo >> 24) & 0xFF) as u8;
        self.mac[4] = (mac_hi & 0xFF) as u8;
        self.mac[5] = ((mac_hi >> 8) & 0xFF) as u8;

        crate::serial_println!("[rtl8139] MAC: {:02X}:{:02X}:{:02X}:{:02X}:{:02X}:{:02X}",
            self.mac[0], self.mac[1], self.mac[2], self.mac[3], self.mac[4], self.mac[5]);

        // Allocate RX buffer (must be physically contiguous for DMA)
        let rx_buf = vec![0u8; RX_BUF_SIZE].into_boxed_slice();
        let rx_phys = virt_to_phys(rx_buf.as_ptr() as u64);
        crate::serial_println!("[rtl8139] RX buffer: virt={:#x} phys={:#x}", rx_buf.as_ptr() as u64, rx_phys);

        // Set RX buffer address
        self.write32(REG_RX_BUF, rx_phys);
        self.rx_buffer = Some(rx_buf);

        // Allocate TX buffers (4 descriptors)
        let tx0 = vec![0u8; TX_BUF_SIZE].into_boxed_slice();
        let tx1 = vec![0u8; TX_BUF_SIZE].into_boxed_slice();
        let tx2 = vec![0u8; TX_BUF_SIZE].into_boxed_slice();
        let tx3 = vec![0u8; TX_BUF_SIZE].into_boxed_slice();

        // Set TX buffer addresses
        self.write32(REG_TX_ADDR0, virt_to_phys(tx0.as_ptr() as u64));
        self.write32(REG_TX_ADDR0 + 4, virt_to_phys(tx1.as_ptr() as u64));
        self.write32(REG_TX_ADDR0 + 8, virt_to_phys(tx2.as_ptr() as u64));
        self.write32(REG_TX_ADDR0 + 12, virt_to_phys(tx3.as_ptr() as u64));

        self.tx_buffers = Some([tx0, tx1, tx2, tx3]);
        self.tx_cur = 0;

        // Enable interrupt mask (RX OK, TX OK, errors)
        self.write16(REG_IMR, INT_RX_OK | INT_RX_ERR | INT_TX_OK | INT_TX_ERR | INT_RX_OVERFLOW);

        // Configure RX: accept broadcast + multicast + physical match, WRAP bit, 8K buffer
        // Bits: AB=0x08, AM=0x04, APM=0x02, AAP=0x01, WRAP=bit7, buffer size=00 (8K+16)
        self.write32(REG_RX_CONFIG, 0x0000_8F0F); // WRAP | AB | AM | APM | AAP | 8K+16

        // Configure TX: default DMA burst, interframe gap
        self.write32(REG_TX_CONFIG, 0x0300_0000); // IFG=normal, DMA burst=2048 bytes

        // Enable RX and TX
        self.write8(REG_CMD, CMD_RX_ENABLE | CMD_TX_ENABLE);

        // Reset RX buffer read pointer
        self.rx_offset = 0;
        self.write16(REG_RX_BUF_PTR, 0xFFF0); // Initial CAPR (CAPR = read ptr - 0x10)

        crate::serial_println!("[rtl8139] Initialized successfully");
        self.status = DriverStatus::Running;
        Ok(())
    }

    fn start(&mut self) -> Result<(), &'static str> {
        self.status = DriverStatus::Running;
        Ok(())
    }

    fn stop(&mut self) -> Result<(), &'static str> {
        // Disable RX/TX
        self.write8(REG_CMD, 0);
        self.write16(REG_IMR, 0);
        self.status = DriverStatus::Suspended;
        Ok(())
    }

    fn status(&self) -> DriverStatus {
        self.status
    }
}

impl NetworkDriver for Rtl8139Driver {
    fn mac_address(&self) -> [u8; 6] {
        self.mac
    }

    fn link_up(&self) -> bool {
        if self.io_base == 0 { return false; }
        // Check basic mode status register (MII register at offset 0x58)
        // Bit 2 = link status, but on RTL8139 the media status register is simpler:
        // MSR at offset 0x58, bit 2 = link ok (inverted: 0 = link up)
        let msr = self.read8(0x58);
        msr & 0x04 == 0 // Link is up when bit 2 is clear
    }

    fn send(&mut self, data: &[u8]) -> Result<(), &'static str> {
        if data.len() > TX_BUF_SIZE {
            return Err("Packet too large");
        }

        let desc = self.tx_cur;

        // Check if descriptor is available (OWN bit should be set = DMA complete)
        let status_reg = REG_TX_STATUS0 + (desc as u16) * 4;
        let status = self.read32(status_reg);
        // On first use status is 0, after that OWN (bit 13) is set when DMA is done
        if desc > 0 || self.stats.tx_packets > 0 {
            // Wait briefly for DMA to complete
            for _ in 0..10000u32 {
                let s = self.read32(status_reg);
                if s & TX_OWN != 0 { break; }
                core::hint::spin_loop();
            }
        }

        // Copy data to TX buffer
        let tx_bufs = self.tx_buffers.as_mut().ok_or("TX buffers not allocated")?;
        tx_bufs[desc][..data.len()].copy_from_slice(data);

        // Write TX status: size in bits [12:0], clear OWN
        // Threshold bits [21:16] = 0 (start TX immediately)
        self.write32(status_reg, data.len() as u32 & 0x1FFF);

        self.tx_cur = (self.tx_cur + 1) % NUM_TX_DESC;
        self.stats.tx_packets += 1;
        self.stats.tx_bytes += data.len() as u64;

        Ok(())
    }

    fn receive(&mut self) -> Option<Vec<u8>> {
        // Check if RX buffer is empty (CMD register bit 0 = buffer empty)
        let cmd = self.read8(REG_CMD);
        if cmd & 0x01 != 0 {
            return None; // BUFE = 1 → buffer empty
        }

        let rx_buf = self.rx_buffer.as_ref()?;
        let buf_size = 8192 + 16; // Actual ring buffer size (8K + 16 wrap)

        // Read packet header at current offset
        let offset = self.rx_offset % buf_size;

        // Header: [u16 status] [u16 length]
        let header_status = u16::from_le_bytes([rx_buf[offset], rx_buf[(offset + 1) % buf_size]]);
        let header_length = u16::from_le_bytes([rx_buf[(offset + 2) % buf_size], rx_buf[(offset + 3) % buf_size]]);

        // Validate
        if header_status & RX_ROK == 0 {
            // Error packet — skip it
            self.stats.rx_errors += 1;
            // Advance past this packet (length includes 4-byte CRC)
            let total = ((header_length as usize) + 4 + 3) & !3; // DWORD-align
            self.rx_offset = (self.rx_offset + total) % buf_size;
            self.write16(REG_RX_BUF_PTR, (self.rx_offset as u16).wrapping_sub(0x10));
            return None;
        }

        let pkt_len = header_length as usize;
        if pkt_len < 8 || pkt_len > 1522 {
            // Invalid length
            self.stats.rx_errors += 1;
            let total = ((pkt_len + 4 + 3) & !3).max(4);
            self.rx_offset = (self.rx_offset + total) % buf_size;
            self.write16(REG_RX_BUF_PTR, (self.rx_offset as u16).wrapping_sub(0x10));
            return None;
        }

        // Copy packet data (skip 4-byte header, exclude 4-byte CRC at end)
        let data_len = pkt_len - 4; // Strip CRC
        let data_start = (offset + 4) % buf_size;
        let mut packet = vec![0u8; data_len];

        for i in 0..data_len {
            packet[i] = rx_buf[(data_start + i) % buf_size];
        }

        // Advance read pointer (header + data + CRC, DWORD aligned)
        let total = (pkt_len + 4 + 3) & !3; // 4-byte header + packet length, aligned
        self.rx_offset = (self.rx_offset + total) % buf_size;
        self.write16(REG_RX_BUF_PTR, (self.rx_offset as u16).wrapping_sub(0x10));

        self.stats.rx_packets += 1;
        self.stats.rx_bytes += data_len as u64;

        Some(packet)
    }

    fn poll(&mut self) {
        if self.io_base == 0 { return; }
        // Acknowledge all pending interrupts
        let isr = self.read16(REG_ISR);
        if isr != 0 {
            self.write16(REG_ISR, isr); // W1C — clear handled interrupts
        }
    }

    fn stats(&self) -> NetStats {
        self.stats
    }
}

const DRIVER_INFO: DriverInfo = DriverInfo {
    name: "rtl8139",
    version: "1.0.0",
    author: "T-RustOs Team",
    category: DriverCategory::Network,
    vendor_ids: &[
        (0x10EC, 0x8139),  // RTL8139
        (0x10EC, 0x8138),  // RTL8139 variant
    ],
};

pub fn register() {
    crate::drivers::register(DRIVER_INFO, || {
        Box::new(Rtl8139Driver::new())
    });
    crate::drivers::net::register_net_driver(DRIVER_INFO, || {
        Box::new(Rtl8139Driver::new())
    });
}
