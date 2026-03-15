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
use crate::drivers::{Driver, DriverInformation, DriverStatus, DriverCategory};
use crate::pci::PciDevice;

// ============================================================================
// E1000 Register Offsets
// ============================================================================

// Device Control
const REGISTER_CONTROLLER: u32 = 0x0000;        // Device Control
const REGISTER_STATUS: u32 = 0x0008;      // Device Status
const REGISTER_EERD: u32 = 0x0014;        // EEPROM Read

// Interrupt Control
const REGISTER_ICR: u32 = 0x00C0;         // Interrupt Cause Read
const REGISTER_IMS: u32 = 0x00D0;         // Interrupt Mask Set
const REGISTER_IMC: u32 = 0x00D8;         // Interrupt Mask Clear

// Receive Control
const REGISTER_RCTL: u32 = 0x0100;        // Receive Control
const REGISTER_RDBAL: u32 = 0x2800;       // RX Descriptor Base Low
const REGISTER_RDBAH: u32 = 0x2804;       // RX Descriptor Base High
const REGISTER_RDLEN: u32 = 0x2808;       // RX Descriptor Length
const REGISTER_RDH: u32 = 0x2810;         // RX Descriptor Head
const REGISTER_RDT: u32 = 0x2818;         // RX Descriptor Tail

// Transmit Control
const REGISTER_TCTL: u32 = 0x0400;        // Transmit Control
const REGISTER_TIPG: u32 = 0x0410;        // TX Inter-Packet Gap
const REGISTER_TDBAL: u32 = 0x3800;       // TX Descriptor Base Low
const REGISTER_TDBAH: u32 = 0x3804;       // TX Descriptor Base High
const REGISTER_TDLEN: u32 = 0x3808;       // TX Descriptor Length
const REGISTER_TDH: u32 = 0x3810;         // TX Descriptor Head
const REGISTER_TDT: u32 = 0x3818;         // TX Descriptor Tail

// MAC Address
const REGISTER_RAL0: u32 = 0x5400;        // Receive Address Low
const REGISTER_RAH0: u32 = 0x5404;        // Receive Address High

// Multicast Table Array
const REGISTER_MTA: u32 = 0x5200;         // Multicast Table Array (128 entries)

// ============================================================================
// Control Register Bits
// ============================================================================

const CONTROLLER_FD: u32 = 1 << 0;         // Full Duplex
const CONTROLLER_ASDE: u32 = 1 << 5;       // Auto-Speed Detection Enable
const CONTROLLER_SLU: u32 = 1 << 6;        // Set Link Up
const CONTROLLER_RST: u32 = 1 << 26;       // Device Reset

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
const TDESC_COMMAND_EOP: u8 = 1 << 0;    // End of Packet
const TDESC_COMMAND_IFCS: u8 = 1 << 1;   // Insert FCS/CRC
const TDESC_COMMAND_RS: u8 = 1 << 3;     // Report Status

// TX Descriptor Status Bits
const TDESC_STA_DD: u8 = 1 << 0;     // Descriptor Done

// RX Descriptor Status Bits
const RDESC_STA_DD: u8 = 1 << 0;     // Descriptor Done

// Interrupt Bits
const ICR_LSC: u32 = 1 << 2;         // Link Status Change

// ============================================================================
// Descriptor Structures
// ============================================================================

const NUMBER_RECEIVE_DESCRIPTOR: usize = 32;
// Compile-time constant — evaluated at compilation, zero runtime cost.
const NUMBER_TRANSMIT_DESCRIPTOR: usize = 8;
// Compile-time constant — evaluated at compilation, zero runtime cost.
const RECEIVE_BUFFER_SIZE: usize = 2048;

/// Receive Descriptor (Legacy Format)
#[repr(C, align(16))]
// #[derive] — auto-generates trait implementations at compile time.
#[derive(Clone, Copy)]
struct RxDesc {
    buffer_address: u64,    // Physical address of buffer
    length: u16,         // Length of received packet
    checksum: u16,       // Packet checksum
    status: u8,          // Descriptor status
    errors: u8,          // Descriptor errors
    special: u16,        // VLAN tag if VP set
}

/// Transmit Descriptor (Legacy Format)
#[repr(C, align(16))]
// #[derive] — auto-generates trait implementations at compile time.
#[derive(Clone, Copy)]
struct TxDesc {
    buffer_address: u64,    // Physical address of buffer
    length: u16,         // Length of packet
    cso: u8,             // Checksum offset
    cmd: u8,             // Command field
    status: u8,          // Descriptor status
    css: u8,             // Checksum start
    special: u16,        // VLAN tag
}

// Trait implementation — fulfills a behavioral contract.
impl Default for RxDesc {
    fn default() -> Self {
        Self {
            buffer_address: 0,
            length: 0,
            checksum: 0,
            status: 0,
            errors: 0,
            special: 0,
        }
    }
}

// Trait implementation — fulfills a behavioral contract.
impl Default for TxDesc {
    fn default() -> Self {
        Self {
            buffer_address: 0,
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
    
    // Descriptor rings (must be 16-byte aligned)
    receive_descs: Vec<RxDesc>,
    transmit_descs: Vec<TxDesc>,
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
impl E1000Driver {
        // Public function — callable from other modules.
pub fn new() -> Self {
        Self {
            status: DriverStatus::Unloaded,
            mmio_base: 0,
            mac: [0x52, 0x54, 0x00, 0xE1, 0x00, 0x00],
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
    
    /// Read a 32-bit register
    fn read_register(&self, offset: u32) -> u32 {
        if self.mmio_base == 0 {
            return 0;
        }
        let address = (self.mmio_base + offset as u64) as *// Compile-time constant — evaluated at compilation, zero runtime cost.
const u32;
                // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe { read_volatile(address) }
    }
    
    /// Write a 32-bit register
    fn write_register(&self, offset: u32, value: u32) {
        if self.mmio_base == 0 {
            return;
        }
        let address = (self.mmio_base + offset as u64) as *mut u32;
                // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe { write_volatile(address, value) }
    }
    
    /// Get virtual address for HHDM
    fn physical_to_virt(physical: u64) -> u64 {
                // Compile-time constant — evaluated at compilation, zero runtime cost.
const HHDM_OFFSET: u64 = 0xFFFF_8000_0000_0000;
        physical + HHDM_OFFSET
    }
    
    /// Get physical address from virtual (for HHDM region)
    fn virt_to_physical(virt: u64) -> u64 {
                // Compile-time constant — evaluated at compilation, zero runtime cost.
const HHDM_OFFSET: u64 = 0xFFFF_8000_0000_0000;
        if virt >= HHDM_OFFSET {
            virt - HHDM_OFFSET
        } else {
            virt
        }
    }
    
    /// Reset the device
    fn reset(&mut self) {
        crate::log_debug!("[E1000] Resetting device...");
        
        // Disable interrupts
        self.write_register(REGISTER_IMC, 0xFFFFFFFF);
        
        // Reset device
        let controller = self.read_register(REGISTER_CONTROLLER);
        self.write_register(REGISTER_CONTROLLER, controller | CONTROLLER_RST);
        
        // Wait for reset to complete
        for _ in 0..1000 {
            if self.read_register(REGISTER_CONTROLLER) & CONTROLLER_RST == 0 {
                break;
            }
            for _ in 0..1000 {
                core::hint::spin_loop();
            }
        }
        
        // Disable interrupts again after reset
        self.write_register(REGISTER_IMC, 0xFFFFFFFF);
        
        crate::log_debug!("[E1000] Reset complete");
    }
    
    /// Read MAC address from EEPROM or RAL/RAH
    fn read_mac(&mut self) {
        // Try to read from RAL/RAH (already set by firmware)
        let ral = self.read_register(REGISTER_RAL0);
        let rah = self.read_register(REGISTER_RAH0);
        
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
        for i in 0..3 {
            self.write_register(REGISTER_EERD, 1 | ((i as u32) << 8));
            for _ in 0..1000 {
                let eerd = self.read_register(REGISTER_EERD);
                if eerd & (1 << 4) != 0 {
                    let data = (eerd >> 16) as u16;
                    self.mac[i * 2] = (data & 0xFF) as u8;
                    self.mac[i * 2 + 1] = (data >> 8) as u8;
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
    fn initialize_receive(&mut self) {
        crate::log_debug!("[E1000] Initializing RX ring ({} descriptors)", NUMBER_RECEIVE_DESCRIPTOR);
        
        self.receive_descs = vec![RxDesc::default(); NUMBER_RECEIVE_DESCRIPTOR];
        self.receive_buffers = Vec::with_capacity(NUMBER_RECEIVE_DESCRIPTOR);
        
        for i in 0..NUMBER_RECEIVE_DESCRIPTOR {
            let buffer = vec![0u8; RECEIVE_BUFFER_SIZE];
            let physical_address = Self::virt_to_physical(buffer.as_pointer() as u64);
            self.receive_descs[i].buffer_address = physical_address;
            self.receive_descs[i].status = 0;
            self.receive_buffers.push(buffer);
        }
        
        let descs_physical = Self::virt_to_physical(self.receive_descs.as_pointer() as u64);
        
        self.write_register(REGISTER_RDBAL, descs_physical as u32);
        self.write_register(REGISTER_RDBAH, (descs_physical >> 32) as u32);
        
        let ring_size = (NUMBER_RECEIVE_DESCRIPTOR * core::mem::size_of::<RxDesc>()) as u32;
        self.write_register(REGISTER_RDLEN, ring_size);
        
        self.write_register(REGISTER_RDH, 0);
        self.write_register(REGISTER_RDT, (NUMBER_RECEIVE_DESCRIPTOR - 1) as u32);
        
        self.receive_cur = 0;
    }
    
    /// Initialize transmit ring
    fn initialize_transmit(&mut self) {
        crate::log_debug!("[E1000] Initializing TX ring ({} descriptors)", NUMBER_TRANSMIT_DESCRIPTOR);
        
        self.transmit_descs = vec![TxDesc::default(); NUMBER_TRANSMIT_DESCRIPTOR];
        self.transmit_buffers = Vec::with_capacity(NUMBER_TRANSMIT_DESCRIPTOR);
        
        for i in 0..NUMBER_TRANSMIT_DESCRIPTOR {
            self.transmit_buffers.push(vec![0u8; RECEIVE_BUFFER_SIZE]);
            // Mark all TX descriptors as done so first send doesn't wait
            self.transmit_descs[i].status = TDESC_STA_DD;
        }
        
        let descs_physical = Self::virt_to_physical(self.transmit_descs.as_pointer() as u64);
        
        self.write_register(REGISTER_TDBAL, descs_physical as u32);
        self.write_register(REGISTER_TDBAH, (descs_physical >> 32) as u32);
        
        let ring_size = (NUMBER_TRANSMIT_DESCRIPTOR * core::mem::size_of::<TxDesc>()) as u32;
        self.write_register(REGISTER_TDLEN, ring_size);
        
        self.write_register(REGISTER_TDH, 0);
        self.write_register(REGISTER_TDT, 0);
        
        self.transmit_cur = 0;
    }
    
    /// Enable receive
    fn enable_receive(&mut self) {
        let rctl = RCTL_EN | RCTL_SBP | RCTL_UPE | RCTL_MPE 
                 | RCTL_LBM_NONE | RCTL_RDMTS_HALF | RCTL_BAM 
                 | RCTL_SECRC | RCTL_BSIZE_2048;
        self.write_register(REGISTER_RCTL, rctl);
    }
    
    /// Enable transmit
    fn enable_transmit(&mut self) {
        self.write_register(REGISTER_TIPG, 10 | (8 << 10) | (6 << 20));
        
        let tctl = TCTL_EN | TCTL_PSP 
                 | (15 << TCTL_CT_SHIFT) 
                 | (64 << TCTL_COLD_SHIFT) 
                 | TCTL_RTLC;
        self.write_register(REGISTER_TCTL, tctl);
    }
    
    /// Setup link
    fn setup_link(&mut self) {
        let controller = self.read_register(REGISTER_CONTROLLER);
        let new_controller = controller | CONTROLLER_SLU | CONTROLLER_ASDE | CONTROLLER_FD;
        self.write_register(REGISTER_CONTROLLER, new_controller);
        
        // Clear multicast table
        for i in 0..128 {
            self.write_register(REGISTER_MTA + i * 4, 0);
        }
        
        // Wait for link - try for longer with VirtualBox
        for i in 0..500 {
            let status = self.read_register(REGISTER_STATUS);
            if status & STATUS_LU != 0 {
                self.link_up.store(true, Ordering::SeqCst);
                let speed = // Pattern matching — Rust's exhaustive branching construct.
match (status & STATUS_SPEED_MASK) >> 6 {
                    0 => 10, 1 => 100, _ => 1000,
                };
                crate::log!("[E1000] Link up at {} Mbps (after {} iterations)", speed, i + 1);
                return;
            }
            for _ in 0..10000 { core::hint::spin_loop(); }
        }
        // Continue anyway - VirtualBox may not report link but TX/RX still work
        crate::log_warn!("[E1000] Link not detected - continuing anyway (VirtualBox NAT mode)");
        self.link_up.store(true, Ordering::SeqCst);
    }
}

// Trait implementation — fulfills a behavioral contract.
impl Driver for E1000Driver {
    fn information(&self) -> &DriverInformation {
        &DRIVER_INFORMATION
    }
    
    fn probe(&mut self, pci_device: &PciDevice) -> Result<(), &'static str> {
        self.status = DriverStatus::Loading;
        
        crate::log!("[E1000] Probing {:04X}:{:04X}", pci_device.vendor_id, pci_device.device_id);
        
        let bar0 = pci_device.bar_address(0).ok_or("No BAR0")?;
        if bar0 == 0 { return Err("BAR0 is zero"); }
        
        crate::serial_println!("[E1000] BAR0={:#x}, calling map_mmio...", bar0);
        
        // Map MMIO region (128KB for E1000)
        const E1000_MMIO_SIZE: usize = 128 * 1024;
        self.mmio_base = crate::memory::map_mmio(bar0, E1000_MMIO_SIZE)
            .map_error(|e| { crate::serial_println!("[E1000] map_mmio failed: {}", e); "Failed to map E1000 MMIO" })?;
        crate::serial_println!("[E1000] map_mmio returned {:#x}", self.mmio_base);
        crate::log_debug!("[E1000] MMIO: phys={:#x} virt={:#x}", bar0, self.mmio_base);
        
        self.reset();
        self.read_mac();
        
        // Set MAC in receive address registers
        let ral = (self.mac[0] as u32) | ((self.mac[1] as u32) << 8)
                | ((self.mac[2] as u32) << 16) | ((self.mac[3] as u32) << 24);
        let rah = (self.mac[4] as u32) | ((self.mac[5] as u32) << 8) | (1 << 31);
        self.write_register(REGISTER_RAL0, ral);
        self.write_register(REGISTER_RAH0, rah);
        
        self.initialize_receive();
        self.initialize_transmit();
        self.setup_link();
        self.enable_receive();
        self.enable_transmit();
        
        self.initialized.store(true, Ordering::SeqCst);
        self.status = DriverStatus::Running;
        
        crate::log!("[E1000] MAC: {:02X}:{:02X}:{:02X}:{:02X}:{:02X}:{:02X}",
            self.mac[0], self.mac[1], self.mac[2], self.mac[3], self.mac[4], self.mac[5]);
        
        Ok(())
    }
    
    fn start(&mut self) -> Result<(), &'static str> {
        self.status = DriverStatus::Running;
        Ok(())
    }
    
    fn stop(&mut self) -> Result<(), &'static str> {
        self.write_register(REGISTER_RCTL, 0);
        self.write_register(REGISTER_TCTL, 0);
        self.write_register(REGISTER_IMC, 0xFFFFFFFF);
        self.status = DriverStatus::Suspended;
        Ok(())
    }
    
    fn status(&self) -> DriverStatus {
        self.status
    }
}

// Trait implementation — fulfills a behavioral contract.
impl NetworkDriver for E1000Driver {
    fn mac_address(&self) -> [u8; 6] {
        self.mac
    }
    
    fn link_up(&self) -> bool {
        if self.mmio_base != 0 {
            let status = self.read_register(REGISTER_STATUS);
            status & STATUS_LU != 0
        } else {
            self.link_up.load(Ordering::Relaxed)
        }
    }
    
    fn link_speed(&self) -> u32 {
        if self.mmio_base == 0 { return 0; }
        let status = self.read_register(REGISTER_STATUS);
                // Pattern matching — Rust's exhaustive branching construct.
match (status & STATUS_SPEED_MASK) >> 6 {
            0 => 10, 1 => 100, _ => 1000,
        }
    }
    
    fn send(&mut self, data: &[u8]) -> Result<(), &'static str> {
        if !self.initialized.load(Ordering::Relaxed) {
            return Err("Driver not initialized");
        }
        if data.len() > RECEIVE_BUFFER_SIZE { return Err("Packet too large"); }
        if data.len() < 14 { return Err("Packet too small"); }
        
        let descriptor_index = self.transmit_cur;
        
        // Wait for descriptor to be available
        let mut timeout = 10000;
        while self.transmit_descs[descriptor_index].status & TDESC_STA_DD == 0 {
            timeout -= 1;
            if timeout == 0 {
                self.transmit_errors.fetch_add(1, Ordering::Relaxed);
                return Err("TX timeout");
            }
            core::hint::spin_loop();
        }
        
        // Copy data to TX buffer
        let buffer = &mut self.transmit_buffers[descriptor_index];
        buffer[..data.len()].copy_from_slice(data);
        
        // Setup descriptor
        let physical_address = Self::virt_to_physical(buffer.as_pointer() as u64);
        self.transmit_descs[descriptor_index].buffer_address = physical_address;
        self.transmit_descs[descriptor_index].length = data.len() as u16;
        self.transmit_descs[descriptor_index].cmd = TDESC_COMMAND_EOP | TDESC_COMMAND_IFCS | TDESC_COMMAND_RS;
        self.transmit_descs[descriptor_index].status = 0;
        
        // Advance tail
        self.transmit_cur = (self.transmit_cur + 1) % NUMBER_TRANSMIT_DESCRIPTOR;
        self.write_register(REGISTER_TDT, self.transmit_cur as u32);
        
        self.transmit_packets.fetch_add(1, Ordering::Relaxed);
        self.transmit_bytes.fetch_add(data.len() as u64, Ordering::Relaxed);
        
        Ok(())
    }
    
    fn receive(&mut self) -> Option<Vec<u8>> {
        if !self.initialized.load(Ordering::Relaxed) { return None; }
        
        let descriptor_index = self.receive_cur;
        let status = self.receive_descs[descriptor_index].status;
        
        if status & RDESC_STA_DD == 0 { return None; }
        
        // Packet received (only log errors, not every packet)
        
        if self.receive_descs[descriptor_index].errors != 0 {
            self.receive_errors.fetch_add(1, Ordering::Relaxed);
            self.receive_descs[descriptor_index].status = 0;
            self.receive_cur = (self.receive_cur + 1) % NUMBER_RECEIVE_DESCRIPTOR;
            return None;
        }
        
        let length = self.receive_descs[descriptor_index].length as usize;
        if length == 0 || length > RECEIVE_BUFFER_SIZE {
            self.receive_descs[descriptor_index].status = 0;
            self.receive_cur = (self.receive_cur + 1) % NUMBER_RECEIVE_DESCRIPTOR;
            return None;
        }
        
        let packet = self.receive_buffers[descriptor_index][..length].to_vec();
        
        self.receive_descs[descriptor_index].status = 0;
        self.receive_descs[descriptor_index].length = 0;
        self.write_register(REGISTER_RDT, descriptor_index as u32);
        self.receive_cur = (self.receive_cur + 1) % NUMBER_RECEIVE_DESCRIPTOR;
        
        self.receive_packets.fetch_add(1, Ordering::Relaxed);
        self.receive_bytes.fetch_add(length as u64, Ordering::Relaxed);
        
        Some(packet)
    }
    
    fn poll(&mut self) {
        if !self.initialized.load(Ordering::Relaxed) { return; }
        let icr = self.read_register(REGISTER_ICR);
        if icr & ICR_LSC != 0 {
            let status = self.read_register(REGISTER_STATUS);
            self.link_up.store(status & STATUS_LU != 0, Ordering::SeqCst);
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
        let mut rctl = self.read_register(REGISTER_RCTL);
        if enabled { rctl |= RCTL_UPE | RCTL_MPE; } 
        else { rctl &= !(RCTL_UPE | RCTL_MPE); }
        self.write_register(REGISTER_RCTL, rctl);
        Ok(())
    }
}

// Compile-time constant — evaluated at compilation, zero runtime cost.
const DRIVER_INFORMATION: DriverInformation = DriverInformation {
    name: "e1000",
    version: "1.0.0",
    author: "TrustOS Team",
    category: DriverCategory::Network,
    vendor_ids: &[
        (0x8086, 0x100E),  // 82540EM (QEMU default)
        (0x8086, 0x100F),  // 82545EM (VMware)
        (0x8086, 0x10D3),  // 82574L
        (0x8086, 0x153A),  // I217-LM
        (0x8086, 0x1533),  // I210
    ],
};

// Public function — callable from other modules.
pub fn register() {
    crate::drivers::register(DRIVER_INFORMATION, || {
        Box::new(E1000Driver::new())
    });
    crate::drivers::net::register_net_driver(DRIVER_INFORMATION, || {
        Box::new(E1000Driver::new())
    });
}
