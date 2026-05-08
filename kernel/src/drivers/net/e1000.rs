//! Intel E1000 Network Driver
//!
//! Complete driver for Intel 82540EM/82545EM NICs (e1000).
//! Supports QEMU, VirtualBox, VMware, and real Intel hardware.
//!
//! Features:
//! - MMIO register access
//! - TX/RX descriptor rings
//! - Link detection
//! - Interrupt handling (polled mode)

use alloc::boxed::Box;
use alloc::vec::Vec;
use alloc::vec;
use core::ptr::{read_volatile, write_volatile};
use core::sync::atomic::{AtomicBool, AtomicU64, Ordering};

use super::{NetworkDriver, NetStats};
use crate::drivers::{Driver, DriverInfo, DriverStatus, DriverCategory};
use crate::pci::PciDevice;

// ============================================================================
// E1000 Register Offsets
// ============================================================================

// Device Control
const REG_CTRL: u32 = 0x0000;        // Device Control
const REG_STATUS: u32 = 0x0008;      // Device Status
const REG_EERD: u32 = 0x0014;        // EEPROM Read

// Interrupt Control
const REG_ICR: u32 = 0x00C0;         // Interrupt Cause Read
const REG_IMS: u32 = 0x00D0;         // Interrupt Mask Set
const REG_IMC: u32 = 0x00D8;         // Interrupt Mask Clear

// Receive Control
const REG_RCTL: u32 = 0x0100;        // Receive Control
const REG_RDBAL: u32 = 0x2800;       // RX Descriptor Base Low
const REG_RDBAH: u32 = 0x2804;       // RX Descriptor Base High
const REG_RDLEN: u32 = 0x2808;       // RX Descriptor Length
const REG_RDH: u32 = 0x2810;         // RX Descriptor Head
const REG_RDT: u32 = 0x2818;         // RX Descriptor Tail

// Transmit Control
const REG_TCTL: u32 = 0x0400;        // Transmit Control
const REG_TIPG: u32 = 0x0410;        // TX Inter-Packet Gap
const REG_TDBAL: u32 = 0x3800;       // TX Descriptor Base Low
const REG_TDBAH: u32 = 0x3804;       // TX Descriptor Base High
const REG_TDLEN: u32 = 0x3808;       // TX Descriptor Length
const REG_TDH: u32 = 0x3810;         // TX Descriptor Head
const REG_TDT: u32 = 0x3818;         // TX Descriptor Tail

// MAC Address
const REG_RAL0: u32 = 0x5400;        // Receive Address Low
const REG_RAH0: u32 = 0x5404;        // Receive Address High

// Multicast Table Array
const REG_MTA: u32 = 0x5200;         // Multicast Table Array (128 entries)

// ============================================================================
// Control Register Bits
// ============================================================================

const CTRL_FD: u32 = 1 << 0;         // Full Duplex
const CTRL_ASDE: u32 = 1 << 5;       // Auto-Speed Detection Enable
const CTRL_SLU: u32 = 1 << 6;        // Set Link Up
const CTRL_RST: u32 = 1 << 26;       // Device Reset

// Status Register Bits
const STATUS_LU: u32 = 1 << 1;       // Link Up
const STATUS_SPEED_MASK: u32 = 0xC0; // Speed indication

// Receive Control Bits
const RCTL_EN: u32 = 1 << 1;         // Receiver Enable
const RCTL_SBP: u32 = 1 << 2;        // Store Bad Packets
const RCTL_UPE: u32 = 1 << 3;        // Unicast Promiscuous Enable
const RCTL_MPE: u32 = 1 << 4;        // Multicast Promiscuous Enable
const RCTL_LBM_NONE: u32 = 0 << 6;   // No Loopback
const RCTL_RDMTS_HALF: u32 = 0 << 8; // RX Desc Min Threshold 1/2
const RCTL_BAM: u32 = 1 << 15;       // Broadcast Accept Mode
const RCTL_BSIZE_2048: u32 = 0 << 16; // Buffer Size 2048
const RCTL_SECRC: u32 = 1 << 26;     // Strip Ethernet CRC

// Transmit Control Bits
const TCTL_EN: u32 = 1 << 1;         // Transmit Enable
const TCTL_PSP: u32 = 1 << 3;        // Pad Short Packets
const TCTL_CT_SHIFT: u32 = 4;        // Collision Threshold
const TCTL_COLD_SHIFT: u32 = 12;     // Collision Distance
const TCTL_RTLC: u32 = 1 << 24;      // Re-transmit on Late Collision

// TX Descriptor Command Bits
const TDESC_CMD_EOP: u8 = 1 << 0;    // End of Packet
const TDESC_CMD_IFCS: u8 = 1 << 1;   // Insert FCS/CRC
const TDESC_CMD_RS: u8 = 1 << 3;     // Report Status

// TX Descriptor Status Bits
const TDESC_STA_DD: u8 = 1 << 0;     // Descriptor Done

// RX Descriptor Status Bits
const RDESC_STA_DD: u8 = 1 << 0;     // Descriptor Done

// Interrupt Bits
const ICR_LSC: u32 = 1 << 2;         // Link Status Change

// SPT (Skylake PCH) specific registers
const REG_CTRL_EXT: u32 = 0x0018;    // Extended Device Control
const REG_FWSM: u32 = 0x5B54;        // Firmware Semaphore
const REG_EXTCNF_CTRL: u32 = 0x0F00; // Extended Config Control
const E1000_FLASH_BASE_ADDR: u32 = 0xE000; // Flash registers base (SPT)
const CTRL_FRCSPD: u32 = 1 << 11;    // Force Speed
const CTRL_FRCDPX: u32 = 1 << 12;    // Force Duplex
const CTRL_PHY_RST: u32 = 1 << 31;   // PHY Reset
const CTRL_EXT_PHYPDEN: u32 = 1 << 20; // PHY Power Down Enable
const CTRL_EXT_SPD_BYPS: u32 = 1 << 15; // Speed Select Bypass

// ============================================================================
// Descriptor Structures
// ============================================================================

const NUM_RX_DESC: usize = 32;
const NUM_TX_DESC: usize = 8;
const RX_BUFFER_SIZE: usize = 2048;

/// Receive Descriptor (Legacy Format)
#[repr(C, align(16))]
#[derive(Clone, Copy)]
struct RxDesc {
    buffer_addr: u64,    // Physical address of buffer
    length: u16,         // Length of received packet
    checksum: u16,       // Packet checksum
    status: u8,          // Descriptor status
    errors: u8,          // Descriptor errors
    special: u16,        // VLAN tag if VP set
}

/// Transmit Descriptor (Legacy Format)
#[repr(C, align(16))]
#[derive(Clone, Copy)]
struct TxDesc {
    buffer_addr: u64,    // Physical address of buffer
    length: u16,         // Length of packet
    cso: u8,             // Checksum offset
    cmd: u8,             // Command field
    status: u8,          // Descriptor status
    css: u8,             // Checksum start
    special: u16,        // VLAN tag
}

impl Default for RxDesc {
    fn default() -> Self {
        Self {
            buffer_addr: 0,
            length: 0,
            checksum: 0,
            status: 0,
            errors: 0,
            special: 0,
        }
    }
}

impl Default for TxDesc {
    fn default() -> Self {
        Self {
            buffer_addr: 0,
            length: 0,
            cso: 0,
            cmd: 0,
            status: TDESC_STA_DD, // Mark as done initially
            css: 0,
            special: 0,
        }
    }
}

// ============================================================================
// E1000 Driver
// ============================================================================

pub struct E1000Driver {
    status: DriverStatus,
    mmio_base: u64,
    mac: [u8; 6],
    is_ich: bool,       // ICH8/ICH9 variant (e1000e)
    is_spt: bool,       // SPT variant (I219 Skylake PCH and newer)
    
    // Descriptor rings (must be 16-byte aligned)
    rx_descs: Vec<RxDesc>,
    tx_descs: Vec<TxDesc>,
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

impl E1000Driver {
    pub fn new() -> Self {
        Self {
            status: DriverStatus::Unloaded,
            mmio_base: 0,
            mac: [0x52, 0x54, 0x00, 0xE1, 0x00, 0x00],
            is_ich: false,
            is_spt: false,
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
    
    /// Read a 32-bit register
    fn read_reg(&self, offset: u32) -> u32 {
        if self.mmio_base == 0 {
            return 0;
        }
        let addr = (self.mmio_base + offset as u64) as *const u32;
        unsafe { read_volatile(addr) }
    }
    
    /// Write a 32-bit register
    fn write_reg(&self, offset: u32, value: u32) {
        if self.mmio_base == 0 {
            return;
        }
        let addr = (self.mmio_base + offset as u64) as *mut u32;
        unsafe { write_volatile(addr, value) }
    }
    
    /// Get virtual address for HHDM
    fn phys_to_virt(phys: u64) -> u64 {
        phys + crate::memory::hhdm_offset()
    }
    
    /// Get physical address from virtual (for HHDM region)
    fn virt_to_phys(virt: u64) -> u64 {
        let hhdm = crate::memory::hhdm_offset();
        if virt >= hhdm { virt - hhdm } else { virt }
    }
    
    /// Disable ULP (Ultra Low Power) mode for SPT variants
    /// Without this, the PHY may be inaccessible on I219
    fn disable_ulp(&mut self) {
        if !self.is_spt { return; }
        crate::serial_println!("[E1000] Disabling ULP mode for SPT...");
        
        // Clear CTRL_EXT force SMBus mode (if set)
        let ctrl_ext = self.read_reg(REG_CTRL_EXT);
        self.write_reg(REG_CTRL_EXT, ctrl_ext & !0x00000800); // clear FORCE_SMBUS
        
        // Small delay for PHY to wake
        for _ in 0..5000 {
            unsafe { core::arch::asm!("out dx, al", in("dx") 0x80u16, in("al") 0u8, options(nomem, nostack)); }
        }
    }
    
    /// Reset the device
    fn reset(&mut self) {
        crate::serial_println!("[E1000] Resetting device (is_ich={}, is_spt={})...", self.is_ich, self.is_spt);
        
        // Disable interrupts
        self.write_reg(REG_IMC, 0xFFFFFFFF);
        
        // Disable TX and RX before reset
        self.write_reg(REG_RCTL, 0);
        self.write_reg(0x0400, 0x00000008); // TCTL = PSP only
        
        // Flush
        let _ = self.read_reg(REG_STATUS);
        
        // Wait 10ms for pending transactions
        for _ in 0..10000 {
            unsafe { core::arch::asm!("out dx, al", in("dx") 0x80u16, in("al") 0u8, options(nomem, nostack)); }
        }
        
        // For SPT, disable ULP before reset
        if self.is_spt {
            self.disable_ulp();
        }
        
        // MAC-only reset — do NOT reset PHY, BIOS has it configured
        // The Linux e1000e driver re-initializes PHY with Kumeran/MDIO workarounds
        // after PHY reset, but we don't have that capability yet.
        let ctrl = self.read_reg(REG_CTRL);
        self.write_reg(REG_CTRL, (ctrl & !CTRL_PHY_RST) | CTRL_RST);
        
        // ICH/SPT needs ~25ms after RST before MMIO is accessible.
        for _ in 0..25000 {
            unsafe { core::arch::asm!("out dx, al", in("dx") 0x80u16, in("al") 0u8, options(nomem, nostack)); }
        }
        
        // Wait for reset to complete (bounded: ~50ms max)
        for i in 0..500u32 {
            let val = self.read_reg(REG_CTRL);
            if val & CTRL_RST == 0 {
                crate::serial_println!("[E1000] Reset cleared after {} polls", i);
                break;
            }
            // ~100µs per iteration
            for _ in 0..100 {
                unsafe { core::arch::asm!("out dx, al", in("dx") 0x80u16, in("al") 0u8, options(nomem, nostack)); }
            }
        }
        
        // Disable interrupts again after reset
        self.write_reg(REG_IMC, 0xFFFFFFFF);
        // Clear pending interrupt causes
        let _ = self.read_reg(REG_ICR);
        
        crate::serial_println!("[E1000] Reset complete");
    }
    
    /// Read MAC address from EEPROM or RAL/RAH
    fn read_mac(&mut self) {
        // Try to read from RAL/RAH (already set by firmware)
        let ral = self.read_reg(REG_RAL0);
        let rah = self.read_reg(REG_RAH0);
        
        if ral != 0 || rah != 0 {
            self.mac[0] = (ral >> 0) as u8;
            self.mac[1] = (ral >> 8) as u8;
            self.mac[2] = (ral >> 16) as u8;
            self.mac[3] = (ral >> 24) as u8;
            self.mac[4] = (rah >> 0) as u8;
            self.mac[5] = (rah >> 8) as u8;
            return;
        }
        
        // Try EEPROM read
        // ICH8/ICH9 (e1000e): done bit is bit 1, address shift is 2
        // Classic e1000: done bit is bit 4, address shift is 8
        let (done_bit, addr_shift) = if self.is_ich { (1 << 1, 2) } else { (1 << 4, 8) };
        
        for i in 0..3u32 {
            self.write_reg(REG_EERD, 1 | (i << addr_shift));
            for _ in 0..1000 {
                let eerd = self.read_reg(REG_EERD);
                if eerd & done_bit != 0 {
                    let data = (eerd >> 16) as u16;
                    self.mac[i as usize * 2] = (data & 0xFF) as u8;
                    self.mac[i as usize * 2 + 1] = (data >> 8) as u8;
                    break;
                }
                core::hint::spin_loop();
            }
        }
        
        // If EEPROM failed, use default QEMU MAC
        if self.mac == [0, 0, 0, 0, 0, 0] {
            self.mac = [0x52, 0x54, 0x00, 0x12, 0x34, 0x56];
        }
    }
    
    /// Initialize receive ring
    fn init_rx(&mut self) {
        crate::log_debug!("[E1000] Initializing RX ring ({} descriptors)", NUM_RX_DESC);
        
        self.rx_descs = vec![RxDesc::default(); NUM_RX_DESC];
        self.rx_buffers = Vec::with_capacity(NUM_RX_DESC);
        
        for i in 0..NUM_RX_DESC {
            let buffer = vec![0u8; RX_BUFFER_SIZE];
            let phys_addr = Self::virt_to_phys(buffer.as_ptr() as u64);
            self.rx_descs[i].buffer_addr = phys_addr;
            self.rx_descs[i].status = 0;
            self.rx_buffers.push(buffer);
        }
        
        let descs_phys = Self::virt_to_phys(self.rx_descs.as_ptr() as u64);
        
        self.write_reg(REG_RDBAL, descs_phys as u32);
        self.write_reg(REG_RDBAH, (descs_phys >> 32) as u32);
        
        let ring_size = (NUM_RX_DESC * core::mem::size_of::<RxDesc>()) as u32;
        self.write_reg(REG_RDLEN, ring_size);
        
        self.write_reg(REG_RDH, 0);
        self.write_reg(REG_RDT, (NUM_RX_DESC - 1) as u32);
        
        self.rx_cur = 0;
    }
    
    /// Initialize transmit ring
    fn init_tx(&mut self) {
        crate::log_debug!("[E1000] Initializing TX ring ({} descriptors)", NUM_TX_DESC);
        
        self.tx_descs = vec![TxDesc::default(); NUM_TX_DESC];
        self.tx_buffers = Vec::with_capacity(NUM_TX_DESC);
        
        for i in 0..NUM_TX_DESC {
            self.tx_buffers.push(vec![0u8; RX_BUFFER_SIZE]);
            // Mark all TX descriptors as done so first send doesn't wait
            self.tx_descs[i].status = TDESC_STA_DD;
        }
        
        let descs_phys = Self::virt_to_phys(self.tx_descs.as_ptr() as u64);
        
        self.write_reg(REG_TDBAL, descs_phys as u32);
        self.write_reg(REG_TDBAH, (descs_phys >> 32) as u32);
        
        let ring_size = (NUM_TX_DESC * core::mem::size_of::<TxDesc>()) as u32;
        self.write_reg(REG_TDLEN, ring_size);
        
        self.write_reg(REG_TDH, 0);
        self.write_reg(REG_TDT, 0);
        
        self.tx_cur = 0;
    }
    
    /// Enable receive
    fn enable_rx(&mut self) {
        let rctl = RCTL_EN | RCTL_SBP | RCTL_UPE | RCTL_MPE 
                 | RCTL_LBM_NONE | RCTL_RDMTS_HALF | RCTL_BAM 
                 | RCTL_SECRC | RCTL_BSIZE_2048;
        self.write_reg(REG_RCTL, rctl);
    }
    
    /// Enable transmit
    fn enable_tx(&mut self) {
        self.write_reg(REG_TIPG, 10 | (8 << 10) | (6 << 20));
        
        let tctl = TCTL_EN | TCTL_PSP 
                 | (15 << TCTL_CT_SHIFT) 
                 | (64 << TCTL_COLD_SHIFT) 
                 | TCTL_RTLC;
        self.write_reg(REG_TCTL, tctl);
    }
    
    /// Setup link
    fn setup_link(&mut self) {
        let mut ctrl = self.read_reg(REG_CTRL);
        ctrl |= CTRL_SLU;  // Set Link Up
        if self.is_spt {
            // SPT/PCH: do NOT force speed/duplex, let auto-neg
            ctrl &= !(CTRL_FRCSPD | CTRL_FRCDPX);
            ctrl |= CTRL_ASDE; // Auto-Speed Detection
        } else {
            ctrl |= CTRL_ASDE | CTRL_FD;
        }
        self.write_reg(REG_CTRL, ctrl);
        crate::serial_println!("[E1000] CTRL={:#010X}", self.read_reg(REG_CTRL));
        
        // Clear multicast table
        for i in 0..128 {
            self.write_reg(REG_MTA + i * 4, 0);
        }
        
        // Wait for link — ~200ms max (200 * ~1ms) — fast enough for boot
        for i in 0..200u32 {
            let status = self.read_reg(REG_STATUS);
            if status & STATUS_LU != 0 {
                self.link_up.store(true, Ordering::SeqCst);
                let speed = match (status & STATUS_SPEED_MASK) >> 6 {
                    0 => 10, 1 => 100, _ => 1000,
                };
                crate::log!("[E1000] Link up at {} Mbps (after {} iterations)", speed, i + 1);
                return;
            }
            // ~1ms per iteration via port 0x80
            for _ in 0..1000 {
                unsafe { core::arch::asm!("out dx, al", in("dx") 0x80u16, in("al") 0u8, options(nomem, nostack)); }
            }
        }
        // Continue anyway — no cable or VirtualBox NAT mode
        crate::log_warn!("[E1000] Link not detected - continuing anyway");
        self.link_up.store(true, Ordering::SeqCst);
    }
}

impl Driver for E1000Driver {
    fn info(&self) -> &DriverInfo {
        &DRIVER_INFO
    }
    
    fn probe(&mut self, pci_device: &PciDevice) -> Result<(), &'static str> {
        self.status = DriverStatus::Loading;
        
        // Enable PCI Bus Mastering and Memory Space (CRITICAL for DMA)
        crate::pci::enable_bus_master(pci_device);
        crate::pci::enable_memory_space(pci_device);
        crate::serial_println!("[E1000] PCI bus mastering + memory space enabled");
        
        // Detect ICH8/ICH9 variant (e1000e)
        self.is_ich = matches!(pci_device.device_id,
            0x1049 | 0x104A | 0x104B | 0x104C | 0x104D |  // ICH8
            0x10BD | 0x10BF | 0x10C0 | 0x10C2 | 0x10C3 |  // ICH9
            0x10CB | 0x10CC | 0x10CD | 0x10CE |             // ICH9
            0x10DE | 0x10DF | 0x10E5 |                       // ICH10
            0x10EA | 0x10EB | 0x10EF | 0x10F0 | 0x10F5 |    // PCH
            0x153A | 0x153B |                                 // I217 (Haswell)
            0x15A0 | 0x15A1 | 0x15A2 | 0x15A3 |             // I218 (Wildcat Point)
            0x15B7 | 0x15B8 | 0x15B9 |                       // I219 (Skylake)
            0x15D6 | 0x15D7 | 0x15D8 |                       // I219 (Kaby Lake)
            0x15E3 |                                          // I219 (Cannon Lake)
            0x0D4C | 0x0D4D | 0x0D4E | 0x0D4F               // I219 (Comet/Ice Lake)
        );
        
        // Detect SPT (Skylake PCH and newer) — different NVM/Flash access
        self.is_spt = matches!(pci_device.device_id,
            0x15B7 | 0x15B8 | 0x15B9 |                       // I219 (Skylake/SPT)
            0x15D6 | 0x15D7 | 0x15D8 |                       // I219 (Kaby Lake/KBP)
            0x15E3 |                                          // I219 (Cannon Lake/CNP)
            0x0D4C | 0x0D4D | 0x0D4E | 0x0D4F               // I219 (Comet/Ice Lake)
        );
        
        let variant = if self.is_spt { "e1000e (SPT)" } else if self.is_ich { "e1000e (ICH)" } else { "e1000" };
        crate::log!("[E1000] Probing {:04X}:{:04X} ({})", pci_device.vendor_id, pci_device.device_id, variant);
        
        let bar0 = pci_device.bar_address(0).ok_or("No BAR0")?;
        if bar0 == 0 { return Err("BAR0 is zero"); }
        
        crate::serial_println!("[E1000] BAR0={:#x}, calling map_mmio...", bar0);
        crate::println!("    [e1000] map_mmio BAR0={:#X}...", bar0);
        
        // Map MMIO region (128KB for E1000)
        const E1000_MMIO_SIZE: usize = 128 * 1024;
        self.mmio_base = crate::memory::map_mmio(bar0, E1000_MMIO_SIZE)
            .map_err(|e| { crate::serial_println!("[E1000] map_mmio failed: {}", e); "Failed to map E1000 MMIO" })?;
        crate::serial_println!("[E1000] map_mmio returned {:#x}", self.mmio_base);
        crate::println!("    [e1000] map_mmio OK -> {:#X}", self.mmio_base);
        crate::log_debug!("[E1000] MMIO: phys={:#x} virt={:#x}", bar0, self.mmio_base);
        
        crate::println!("    [e1000] reset...");
        self.reset();
        crate::println!("    [e1000] read_mac...");
        self.read_mac();
        
        // Set MAC in receive address registers
        let ral = (self.mac[0] as u32) | ((self.mac[1] as u32) << 8)
                | ((self.mac[2] as u32) << 16) | ((self.mac[3] as u32) << 24);
        let rah = (self.mac[4] as u32) | ((self.mac[5] as u32) << 8) | (1 << 31);
        self.write_reg(REG_RAL0, ral);
        self.write_reg(REG_RAH0, rah);
        
        crate::println!("    [e1000] init_rx...");
        self.init_rx();
        crate::println!("    [e1000] init_tx...");
        self.init_tx();
        crate::println!("    [e1000] setup_link...");
        self.setup_link();
        crate::println!("    [e1000] enable_rx/tx...");
        self.enable_rx();
        self.enable_tx();
        
        self.initialized.store(true, Ordering::SeqCst);
        self.status = DriverStatus::Running;
        
        crate::log!("[E1000] MAC: {:02X}:{:02X}:{:02X}:{:02X}:{:02X}:{:02X}",
            self.mac[0], self.mac[1], self.mac[2], self.mac[3], self.mac[4], self.mac[5]);
        
        // Dump key registers to serial for hardware debug
        crate::serial_println!("[E1000] STATUS={:#010X} CTRL={:#010X} RCTL={:#010X} TCTL={:#010X}",
            self.read_reg(REG_STATUS), self.read_reg(REG_CTRL),
            self.read_reg(REG_RCTL), self.read_reg(0x0400));
        if self.is_spt {
            crate::serial_println!("[E1000] FWSM={:#010X} CTRL_EXT={:#010X} EXTCNF_CTRL={:#010X}",
                self.read_reg(REG_FWSM), self.read_reg(REG_CTRL_EXT),
                self.read_reg(REG_EXTCNF_CTRL));
        }
        
        Ok(())
    }
    
    fn start(&mut self) -> Result<(), &'static str> {
        self.status = DriverStatus::Running;
        Ok(())
    }
    
    fn stop(&mut self) -> Result<(), &'static str> {
        self.write_reg(REG_RCTL, 0);
        self.write_reg(REG_TCTL, 0);
        self.write_reg(REG_IMC, 0xFFFFFFFF);
        self.status = DriverStatus::Suspended;
        Ok(())
    }
    
    fn status(&self) -> DriverStatus {
        self.status
    }
}

impl NetworkDriver for E1000Driver {
    fn mac_address(&self) -> [u8; 6] {
        self.mac
    }
    
    fn link_up(&self) -> bool {
        if self.mmio_base != 0 {
            let status = self.read_reg(REG_STATUS);
            status & STATUS_LU != 0
        } else {
            self.link_up.load(Ordering::Relaxed)
        }
    }
    
    fn link_speed(&self) -> u32 {
        if self.mmio_base == 0 { return 0; }
        let status = self.read_reg(REG_STATUS);
        match (status & STATUS_SPEED_MASK) >> 6 {
            0 => 10, 1 => 100, _ => 1000,
        }
    }
    
    fn send(&mut self, data: &[u8]) -> Result<(), &'static str> {
        if !self.initialized.load(Ordering::Relaxed) {
            return Err("Driver not initialized");
        }
        if data.len() > RX_BUFFER_SIZE { return Err("Packet too large"); }
        if data.len() < 14 { return Err("Packet too small"); }
        
        let desc_idx = self.tx_cur;
        
        // Wait for descriptor to be available
        let mut timeout = 10000;
        while self.tx_descs[desc_idx].status & TDESC_STA_DD == 0 {
            timeout -= 1;
            if timeout == 0 {
                self.tx_errors.fetch_add(1, Ordering::Relaxed);
                return Err("TX timeout");
            }
            core::hint::spin_loop();
        }
        
        // Copy data to TX buffer
        let buffer = &mut self.tx_buffers[desc_idx];
        buffer[..data.len()].copy_from_slice(data);
        
        // Setup descriptor
        let phys_addr = Self::virt_to_phys(buffer.as_ptr() as u64);
        self.tx_descs[desc_idx].buffer_addr = phys_addr;
        self.tx_descs[desc_idx].length = data.len() as u16;
        self.tx_descs[desc_idx].cmd = TDESC_CMD_EOP | TDESC_CMD_IFCS | TDESC_CMD_RS;
        self.tx_descs[desc_idx].status = 0;
        
        // Advance tail
        self.tx_cur = (self.tx_cur + 1) % NUM_TX_DESC;
        self.write_reg(REG_TDT, self.tx_cur as u32);
        
        self.tx_packets.fetch_add(1, Ordering::Relaxed);
        self.tx_bytes.fetch_add(data.len() as u64, Ordering::Relaxed);
        
        Ok(())
    }
    
    fn receive(&mut self) -> Option<Vec<u8>> {
        if !self.initialized.load(Ordering::Relaxed) { return None; }
        
        let desc_idx = self.rx_cur;
        let status = self.rx_descs[desc_idx].status;
        
        if status & RDESC_STA_DD == 0 { return None; }
        
        // Packet received (only log errors, not every packet)
        
        if self.rx_descs[desc_idx].errors != 0 {
            self.rx_errors.fetch_add(1, Ordering::Relaxed);
            self.rx_descs[desc_idx].status = 0;
            self.rx_cur = (self.rx_cur + 1) % NUM_RX_DESC;
            return None;
        }
        
        let length = self.rx_descs[desc_idx].length as usize;
        if length == 0 || length > RX_BUFFER_SIZE {
            self.rx_descs[desc_idx].status = 0;
            self.rx_cur = (self.rx_cur + 1) % NUM_RX_DESC;
            return None;
        }
        
        let packet = self.rx_buffers[desc_idx][..length].to_vec();
        
        self.rx_descs[desc_idx].status = 0;
        self.rx_descs[desc_idx].length = 0;
        self.write_reg(REG_RDT, desc_idx as u32);
        self.rx_cur = (self.rx_cur + 1) % NUM_RX_DESC;
        
        self.rx_packets.fetch_add(1, Ordering::Relaxed);
        self.rx_bytes.fetch_add(length as u64, Ordering::Relaxed);
        
        Some(packet)
    }
    
    fn poll(&mut self) {
        if !self.initialized.load(Ordering::Relaxed) { return; }
        let icr = self.read_reg(REG_ICR);
        if icr & ICR_LSC != 0 {
            let status = self.read_reg(REG_STATUS);
            self.link_up.store(status & STATUS_LU != 0, Ordering::SeqCst);
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
        let mut rctl = self.read_reg(REG_RCTL);
        if enabled { rctl |= RCTL_UPE | RCTL_MPE; } 
        else { rctl &= !(RCTL_UPE | RCTL_MPE); }
        self.write_reg(REG_RCTL, rctl);
        Ok(())
    }
}

const DRIVER_INFO: DriverInfo = DriverInfo {
    name: "e1000",
    version: "1.0.0",
    author: "TrustOS Team",
    category: DriverCategory::Network,
    vendor_ids: &[
        (0x8086, 0x100E),  // 82540EM (QEMU default)
        (0x8086, 0x100F),  // 82545EM (VMware)
        (0x8086, 0x10D3),  // 82574L
        (0x8086, 0x153A),  // I217-LM
        (0x8086, 0x153B),  // I217-V
        (0x8086, 0x1533),  // I210
        // e1000e PCH — Skylake/Kaby Lake (H170, Z170, B150, etc.)
        (0x8086, 0x15A0),  // I218-LM (Wildcat Point)
        (0x8086, 0x15A1),  // I218-V (Wildcat Point)
        (0x8086, 0x15A2),  // I218-LM-3
        (0x8086, 0x15A3),  // I218-V-3
        (0x8086, 0x15B7),  // I219-LM (Skylake)
        (0x8086, 0x15B8),  // I219-V (Skylake) — ASUS H170-PRO
        (0x8086, 0x15B9),  // I219-LM-2
        (0x8086, 0x15D6),  // I219-V-2 (Kaby Lake)
        (0x8086, 0x15D7),  // I219-LM-3 (Kaby Lake)
        (0x8086, 0x15D8),  // I219-V-3
        (0x8086, 0x15E3),  // I219-LM (Cannon Lake)
        (0x8086, 0x0D4E),  // I219-LM (Comet Lake)
        (0x8086, 0x0D4F),  // I219-V (Comet Lake)
        (0x8086, 0x0D4C),  // I219-LM (Ice Lake)
        (0x8086, 0x0D4D),  // I219-V (Ice Lake)
        // e1000e (ICH8/ICH9) — ThinkPad T61, T400, X200, various laptops/desktops
        (0x8086, 0x1049),  // 82566MM (T61 onboard)
        (0x8086, 0x104A),  // 82566DM
        (0x8086, 0x104B),  // 82566DC
        (0x8086, 0x104C),  // 82562V
        (0x8086, 0x104D),  // 82566MC
        (0x8086, 0x10BD),  // 82566DM-2
        (0x8086, 0x10BF),  // 82567LF
        (0x8086, 0x10C0),  // 82562V-2
        (0x8086, 0x10C2),  // 82562G-2
        (0x8086, 0x10C3),  // 82562GT-2
        (0x8086, 0x10CB),  // 82567V
        (0x8086, 0x10CC),  // 82567LM-2
        (0x8086, 0x10CD),  // 82567LF-2
        (0x8086, 0x10CE),  // 82567V-2
        (0x8086, 0x10DE),  // 82567LM-3 (T500/W500)
        (0x8086, 0x10DF),  // 82567LF-3
        (0x8086, 0x10E5),  // 82567LM-4
        (0x8086, 0x10EA),  // 82577LM
        (0x8086, 0x10EB),  // 82577LC
        (0x8086, 0x10EF),  // 82578DM
        (0x8086, 0x10F0),  // 82578DC
        (0x8086, 0x10F5),  // 82567LM (ICH9)
    ],
};

pub fn register() {
    crate::drivers::register(DRIVER_INFO, || {
        Box::new(E1000Driver::new())
    });
    crate::drivers::net::register_net_driver(DRIVER_INFO, || {
        Box::new(E1000Driver::new())
    });
}
