//! PCI Bus Enumeration
//!
//! Real PCI hardware detection and device enumeration.
//! This is the foundation for all hardware drivers.

use alloc::vec::Vec;
use alloc::string::String;
use alloc::format;
use crate::arch::Port;
use spin::Mutex;

/// PCI configuration ports
const PCI_CONFIG_ADDRESS: u16 = 0xCF8;
const PCI_CONFIG_DATA: u16 = 0xCFC;

/// PCI device classes
pub mod class {
    pub const UNCLASSIFIED: u8 = 0x00;
    pub const MASS_STORAGE: u8 = 0x01;
    pub const NETWORK: u8 = 0x02;
    pub const DISPLAY: u8 = 0x03;
    pub const MULTIMEDIA: u8 = 0x04;
    pub const MEMORY: u8 = 0x05;
    pub const BRIDGE: u8 = 0x06;
    pub const SIMPLE_COMM: u8 = 0x07;
    pub const BASE_PERIPHERAL: u8 = 0x08;
    pub const INPUT: u8 = 0x09;
    pub const DOCKING: u8 = 0x0A;
    pub const PROCESSOR: u8 = 0x0B;
    pub const SERIAL_BUS: u8 = 0x0C;
    pub const WIRELESS: u8 = 0x0D;
    pub const INTELLIGENT: u8 = 0x0E;
    pub const SATELLITE: u8 = 0x0F;
    pub const ENCRYPTION: u8 = 0x10;
    pub const SIGNAL_PROC: u8 = 0x11;
}

/// Storage subclasses
pub mod storage {
    pub const SCSI: u8 = 0x00;
    pub const IDE: u8 = 0x01;
    pub const FLOPPY: u8 = 0x02;
    pub const IPI: u8 = 0x03;
    pub const RAID: u8 = 0x04;
    pub const ATA: u8 = 0x05;
    pub const SATA: u8 = 0x06;
    pub const SAS: u8 = 0x07;
    pub const NVM: u8 = 0x08;  // NVMe
}

/// Network subclasses
pub mod network {
    pub const ETHERNET: u8 = 0x00;
    pub const TOKEN_RING: u8 = 0x01;
    pub const FDDI: u8 = 0x02;
    pub const ATM: u8 = 0x03;
    pub const ISDN: u8 = 0x04;
    pub const PICMG: u8 = 0x06;
    pub const INFINIBAND: u8 = 0x07;
}

/// Bridge subclasses
pub mod bridge {
    pub const HOST: u8 = 0x00;
    pub const ISA: u8 = 0x01;
    pub const EISA: u8 = 0x02;
    pub const MCA: u8 = 0x03;
    pub const PCI_TO_PCI: u8 = 0x04;
    pub const PCMCIA: u8 = 0x05;
    pub const NUBUS: u8 = 0x06;
    pub const CARDBUS: u8 = 0x07;
    pub const RACEWAY: u8 = 0x08;
    pub const PCI_SEMI: u8 = 0x09;
    pub const INFINIBAND_PCI: u8 = 0x0A;
}

/// Serial bus subclasses
pub mod serial {
    pub const FIREWIRE: u8 = 0x00;
    pub const ACCESS: u8 = 0x01;
    pub const SSA: u8 = 0x02;
    pub const USB: u8 = 0x03;
    pub const FIBRE: u8 = 0x04;
    pub const SMBUS: u8 = 0x05;
    pub const INFINIBAND: u8 = 0x06;
    pub const IPMI: u8 = 0x07;
}

/// USB programming interfaces
pub mod usb {
    pub const UHCI: u8 = 0x00;
    pub const OHCI: u8 = 0x10;
    pub const EHCI: u8 = 0x20;
    pub const XHCI: u8 = 0x30;
}

/// PCI device information
#[derive(Debug, Clone)]
pub struct PciDevice {
    pub bus: u8,
    pub device: u8,
    pub function: u8,
    pub vendor_id: u16,
    pub device_id: u16,
    pub class_code: u8,
    pub subclass: u8,
    pub prog_if: u8,
    pub revision: u8,
    pub header_type: u8,
    pub interrupt_line: u8,
    pub interrupt_pin: u8,
    pub bar: [u32; 6],
}

impl PciDevice {
    /// Get class name
    pub fn class_name(&self) -> &'static str {
        match self.class_code {
            class::UNCLASSIFIED => "Unclassified",
            class::MASS_STORAGE => "Mass Storage",
            class::NETWORK => "Network Controller",
            class::DISPLAY => "Display Controller",
            class::MULTIMEDIA => "Multimedia",
            class::MEMORY => "Memory Controller",
            class::BRIDGE => "Bridge",
            class::SIMPLE_COMM => "Communication",
            class::BASE_PERIPHERAL => "Peripheral",
            class::INPUT => "Input Device",
            class::DOCKING => "Docking Station",
            class::PROCESSOR => "Processor",
            class::SERIAL_BUS => "Serial Bus",
            class::WIRELESS => "Wireless",
            class::INTELLIGENT => "Intelligent I/O",
            class::SATELLITE => "Satellite",
            class::ENCRYPTION => "Encryption",
            class::SIGNAL_PROC => "Signal Processing",
            _ => "Unknown",
        }
    }
    
    /// Get subclass name
    pub fn subclass_name(&self) -> &'static str {
        match (self.class_code, self.subclass) {
            // Storage
            (class::MASS_STORAGE, storage::IDE) => "IDE Controller",
            (class::MASS_STORAGE, storage::SATA) => "SATA Controller",
            (class::MASS_STORAGE, storage::NVM) => "NVMe Controller",
            (class::MASS_STORAGE, storage::RAID) => "RAID Controller",
            (class::MASS_STORAGE, storage::SCSI) => "SCSI Controller",
            (class::MASS_STORAGE, storage::ATA) => "ATA Controller",
            
            // Network
            (class::NETWORK, network::ETHERNET) => "Ethernet",
            (class::NETWORK, network::INFINIBAND) => "InfiniBand",
            
            // Display
            (class::DISPLAY, 0x00) => "VGA Compatible",
            (class::DISPLAY, 0x01) => "XGA Controller",
            (class::DISPLAY, 0x02) => "3D Controller",
            
            // Bridge
            (class::BRIDGE, bridge::HOST) => "Host Bridge",
            (class::BRIDGE, bridge::ISA) => "ISA Bridge",
            (class::BRIDGE, bridge::PCI_TO_PCI) => "PCI-to-PCI Bridge",
            
            // Serial Bus
            (class::SERIAL_BUS, serial::USB) => match self.prog_if {
                usb::UHCI => "USB UHCI",
                usb::OHCI => "USB OHCI",
                usb::EHCI => "USB 2.0 EHCI",
                usb::XHCI => "USB 3.0 xHCI",
                0xFE => "USB Device",
                _ => "USB Controller",
            },
            (class::SERIAL_BUS, serial::SMBUS) => "SMBus",
            
            _ => "",
        }
    }
    
    /// Get vendor name (common vendors)
    pub fn vendor_name(&self) -> &'static str {
        match self.vendor_id {
            0x8086 => "Intel",
            0x1022 => "AMD",
            0x10DE => "NVIDIA",
            0x1002 => "AMD/ATI",
            0x14E4 => "Broadcom",
            0x10EC => "Realtek",
            0x8087 => "Intel (Wireless)",
            0x1B4B => "Marvell",
            0x1969 => "Qualcomm Atheros",
            0x168C => "Qualcomm Atheros",
            0x1AF4 => "Red Hat (virtio)",
            0x1234 => "QEMU",
            0x15AD => "VMware",
            0x80EE => "VirtualBox",
            0x1AB8 => "Parallels",
            _ => "Unknown",
        }
    }
    
    /// Check if this is a multifunction device
    pub fn is_multifunction(&self) -> bool {
        self.header_type & 0x80 != 0
    }
    
    /// Get BAR address (masked)
    pub fn bar_address(&self, index: usize) -> Option<u64> {
        if index >= 6 {
            return None;
        }
        
        let bar = self.bar[index];
        if bar == 0 {
            return None;
        }
        
        // Check if memory or I/O
        if bar & 1 == 0 {
            // Memory BAR
            let bar_type = (bar >> 1) & 0x3;
            match bar_type {
                0 => Some((bar & 0xFFFFFFF0) as u64), // 32-bit
                2 if index < 5 => {
                    // 64-bit BAR
                    let high = self.bar[index + 1] as u64;
                    Some(((high << 32) | (bar & 0xFFFFFFF0) as u64))
                }
                _ => None,
            }
        } else {
            // I/O BAR
            Some((bar & 0xFFFFFFFC) as u64)
        }
    }
    
    /// Check if BAR is memory-mapped
    pub fn bar_is_memory(&self, index: usize) -> bool {
        if index >= 6 {
            return false;
        }
        self.bar[index] & 1 == 0
    }
    
    /// Check if BAR is I/O port
    pub fn bar_is_io(&self, index: usize) -> bool {
        if index >= 6 {
            return false;
        }
        self.bar[index] & 1 != 0
    }
}

/// Global device list
static DEVICES: Mutex<Vec<PciDevice>> = Mutex::new(Vec::new());

/// Cached PCIe ECAM base address (0 = not available)
static ECAM_BASE: core::sync::atomic::AtomicU64 = core::sync::atomic::AtomicU64::new(0);
/// Cached ECAM virtual base (mapped MMIO)
static ECAM_VIRT: core::sync::atomic::AtomicU64 = core::sync::atomic::AtomicU64::new(0);
/// ECAM start bus
static ECAM_START_BUS: core::sync::atomic::AtomicU8 = core::sync::atomic::AtomicU8::new(0);
/// ECAM end bus
static ECAM_END_BUS: core::sync::atomic::AtomicU8 = core::sync::atomic::AtomicU8::new(0);

/// Initialize PCIe ECAM if MCFG table is available
fn init_ecam() {
    if let Some(info) = crate::acpi::get_info() {
        if let Some(first) = info.mcfg_regions.first() {
            let base = first.base_address;
            let size = first.size() as usize;
            let start_bus = first.start_bus;
            let end_bus = first.end_bus;
            
            crate::serial_println!("[PCI] PCIe ECAM detected: base={:#x} size={:#x} buses={}-{}",
                base, size, start_bus, end_bus);
            
            match crate::memory::map_mmio(base, size) {
                Ok(virt) => {
                    ECAM_BASE.store(base, core::sync::atomic::Ordering::SeqCst);
                    ECAM_VIRT.store(virt, core::sync::atomic::Ordering::SeqCst);
                    ECAM_START_BUS.store(start_bus, core::sync::atomic::Ordering::SeqCst);
                    ECAM_END_BUS.store(end_bus, core::sync::atomic::Ordering::SeqCst);
                    crate::serial_println!("[PCI] PCIe ECAM mapped at virt={:#x}", virt);
                }
                Err(e) => {
                    crate::serial_println!("[PCI] Failed to map ECAM: {} — using legacy PIO only", e);
                }
            }
        }
    }
}

/// Read 32-bit value from PCIe ECAM config space (supports full 4K space, offset 0..4095)
pub fn ecam_config_read32(bus: u8, device: u8, function: u8, offset: u16) -> Option<u32> {
    let virt = ECAM_VIRT.load(core::sync::atomic::Ordering::Relaxed);
    if virt == 0 { return None; }
    let start = ECAM_START_BUS.load(core::sync::atomic::Ordering::Relaxed);
    let end = ECAM_END_BUS.load(core::sync::atomic::Ordering::Relaxed);
    if bus < start || bus > end || device > 31 || function > 7 || offset > 4092 {
        return None;
    }
    let addr = virt
        + ((bus - start) as u64) * (32 * 8 * 4096)
        + (device as u64) * (8 * 4096)
        + (function as u64) * 4096
        + (offset & 0xFFC) as u64;
    Some(unsafe { core::ptr::read_volatile(addr as *const u32) })
}

/// Write 32-bit value to PCIe ECAM config space
pub fn ecam_config_write32(bus: u8, device: u8, function: u8, offset: u16, value: u32) -> bool {
    let virt = ECAM_VIRT.load(core::sync::atomic::Ordering::Relaxed);
    if virt == 0 { return false; }
    let start = ECAM_START_BUS.load(core::sync::atomic::Ordering::Relaxed);
    let end = ECAM_END_BUS.load(core::sync::atomic::Ordering::Relaxed);
    if bus < start || bus > end || device > 31 || function > 7 || offset > 4092 {
        return false;
    }
    let addr = virt
        + ((bus - start) as u64) * (32 * 8 * 4096)
        + (device as u64) * (8 * 4096)
        + (function as u64) * 4096
        + (offset & 0xFFC) as u64;
    unsafe { core::ptr::write_volatile(addr as *mut u32, value); }
    true
}

/// Read from PCIe extended config space (offset 0-4095).
/// Falls back to legacy PIO for offsets < 256 if ECAM is unavailable.
pub fn pcie_config_read(dev: &PciDevice, offset: u16) -> u32 {
    if let Some(val) = ecam_config_read32(dev.bus, dev.device, dev.function, offset) {
        return val;
    }
    // Fallback to legacy PIO for standard config space
    if offset < 256 {
        return config_read(dev.bus, dev.device, dev.function, offset as u8);
    }
    0xFFFFFFFF // Inaccessible
}

/// Write to PCIe extended config space
pub fn pcie_config_write(dev: &PciDevice, offset: u16, value: u32) {
    if ecam_config_write32(dev.bus, dev.device, dev.function, offset, value) {
        return;
    }
    if offset < 256 {
        config_write(dev.bus, dev.device, dev.function, offset as u8, value);
    }
}

/// Check if PCIe ECAM is available
pub fn ecam_available() -> bool {
    ECAM_VIRT.load(core::sync::atomic::Ordering::Relaxed) != 0
}

/// Read PCI configuration register
pub fn config_read(bus: u8, device: u8, function: u8, offset: u8) -> u32 {
    let address: u32 = 
        (1 << 31) |                       // Enable bit
        ((bus as u32) << 16) |            // Bus number
        ((device as u32) << 11) |         // Device number
        ((function as u32) << 8) |        // Function number
        ((offset as u32) & 0xFC);         // Register offset (aligned)
    
    let mut addr_port: Port<u32> = Port::new(PCI_CONFIG_ADDRESS);
    let mut data_port: Port<u32> = Port::new(PCI_CONFIG_DATA);
    
    unsafe {
        addr_port.write(address);
        data_port.read()
    }
}

/// Write PCI configuration register
pub fn config_write(bus: u8, device: u8, function: u8, offset: u8, value: u32) {
    let address: u32 = 
        (1 << 31) |
        ((bus as u32) << 16) |
        ((device as u32) << 11) |
        ((function as u32) << 8) |
        ((offset as u32) & 0xFC);
    
    let mut addr_port: Port<u32> = Port::new(PCI_CONFIG_ADDRESS);
    let mut data_port: Port<u32> = Port::new(PCI_CONFIG_DATA);
    
    unsafe {
        addr_port.write(address);
        data_port.write(value);
    }
}

/// Read 16-bit value from PCI config
pub fn config_read16(bus: u8, device: u8, function: u8, offset: u8) -> u16 {
    let value = config_read(bus, device, function, offset & 0xFC);
    ((value >> ((offset & 2) * 8)) & 0xFFFF) as u16
}

/// Read 8-bit value from PCI config
pub fn config_read8(bus: u8, device: u8, function: u8, offset: u8) -> u8 {
    let value = config_read(bus, device, function, offset & 0xFC);
    ((value >> ((offset & 3) * 8)) & 0xFF) as u8
}

/// Scan a single function
fn scan_function(bus: u8, device: u8, function: u8) -> Option<PciDevice> {
    let vendor_device = config_read(bus, device, function, 0x00);
    let vendor_id = (vendor_device & 0xFFFF) as u16;
    
    if vendor_id == 0xFFFF || vendor_id == 0x0000 {
        return None;
    }
    
    let device_id = ((vendor_device >> 16) & 0xFFFF) as u16;
    
    let class_reg = config_read(bus, device, function, 0x08);
    let revision = (class_reg & 0xFF) as u8;
    let prog_if = ((class_reg >> 8) & 0xFF) as u8;
    let subclass = ((class_reg >> 16) & 0xFF) as u8;
    let class_code = ((class_reg >> 24) & 0xFF) as u8;
    
    let header_reg = config_read(bus, device, function, 0x0C);
    let header_type = ((header_reg >> 16) & 0xFF) as u8;
    
    let int_reg = config_read(bus, device, function, 0x3C);
    let interrupt_line = (int_reg & 0xFF) as u8;
    let interrupt_pin = ((int_reg >> 8) & 0xFF) as u8;
    
    // Read BARs
    let mut bar = [0u32; 6];
    for i in 0..6 {
        bar[i] = config_read(bus, device, function, 0x10 + (i as u8 * 4));
    }
    
    Some(PciDevice {
        bus,
        device,
        function,
        vendor_id,
        device_id,
        class_code,
        subclass,
        prog_if,
        revision,
        header_type,
        interrupt_line,
        interrupt_pin,
        bar,
    })
}

/// Scan a device (all functions)
fn scan_device(bus: u8, device: u8, devices: &mut Vec<PciDevice>) {
    if let Some(dev) = scan_function(bus, device, 0) {
        let multifunction = dev.is_multifunction();
        devices.push(dev);
        
        if multifunction {
            for function in 1..8 {
                if let Some(dev) = scan_function(bus, device, function) {
                    devices.push(dev);
                }
            }
        }
    }
}

/// Scan entire PCI bus
pub fn scan() -> Vec<PciDevice> {
    let mut devices = Vec::new();
    
    // Try to detect PCI - check multiple locations since slot 0:0.0 may be empty
    let mut pci_found = false;
    for device in 0..32 {
        let test = config_read(0, device, 0, 0);
        if test != 0xFFFFFFFF && test != 0x00000000 {
            pci_found = true;
            break;
        }
    }
    
    if !pci_found {
        crate::log_warn!("[PCI] No PCI bus detected - scanning anyway...");
    }
    
    // Scan buses 0-255 (scan even if detection failed - some VMs need this)
    // We must scan all buses to find devices behind PCI-to-PCI bridges
    // on real hardware with complex topologies.
    let mut max_bus: u8 = 0;
    
    for bus in 0..=255u8 {
        let mut found_on_bus = false;
        for device in 0..32 {
            let before = devices.len();
            scan_device(bus, device, &mut devices);
            if devices.len() > before {
                found_on_bus = true;
                // Check if any new device is a PCI bridge — if so, extend scan range
                for dev in &devices[before..] {
                    if dev.class_code == class::BRIDGE && dev.subclass == 0x04 {
                        // PCI-to-PCI bridge: read secondary bus number (offset 0x19)
                        let sec_bus = (crate::pci::config_read(dev.bus, dev.device, dev.function, 0x18) >> 8) as u8;
                        let sub_bus = (crate::pci::config_read(dev.bus, dev.device, dev.function, 0x18) >> 16) as u8;
                        if sub_bus > max_bus {
                            max_bus = sub_bus;
                        }
                        if sec_bus > max_bus {
                            max_bus = sec_bus;
                        }
                    }
                }
            }
        }
        
        // Adaptive stop: only scan beyond bus 0 if bridges were found
        // or if we have discovered devices on higher buses
        if bus >= max_bus && bus > 0 && !found_on_bus {
            // Allow scanning a few buses past the last known bridge
            if bus > max_bus + 2 {
                break;
            }
        }
    }
    
    devices
}

/// Initialize PCI subsystem
pub fn init() {
    // Try to enable PCIe ECAM first (for extended config space access)
    init_ecam();
    
    let devices = scan();
    let count = devices.len();
    
    crate::log!("[PCI] Found {} devices:", count);
    
    for dev in &devices {
        let subclass_name = dev.subclass_name();
        if subclass_name.is_empty() {
            crate::log!("[PCI]   {:02X}:{:02X}.{} {:04X}:{:04X} {} ({})",
                dev.bus, dev.device, dev.function,
                dev.vendor_id, dev.device_id,
                dev.class_name(),
                dev.vendor_name());
        } else {
            crate::log!("[PCI]   {:02X}:{:02X}.{} {:04X}:{:04X} {} - {} ({})",
                dev.bus, dev.device, dev.function,
                dev.vendor_id, dev.device_id,
                dev.class_name(),
                subclass_name,
                dev.vendor_name());
        }
    }
    
    *DEVICES.lock() = devices;
}

/// Get all PCI devices
pub fn get_devices() -> Vec<PciDevice> {
    DEVICES.lock().clone()
}

/// Find devices by class
pub fn find_by_class(class_code: u8) -> Vec<PciDevice> {
    DEVICES.lock().iter()
        .filter(|d| d.class_code == class_code)
        .cloned()
        .collect()
}

/// Find devices by class and subclass
pub fn find_by_class_subclass(class_code: u8, subclass: u8) -> Vec<PciDevice> {
    DEVICES.lock().iter()
        .filter(|d| d.class_code == class_code && d.subclass == subclass)
        .cloned()
        .collect()
}

/// Find device by vendor and device ID
pub fn find_by_id(vendor_id: u16, device_id: u16) -> Option<PciDevice> {
    DEVICES.lock().iter()
        .find(|d| d.vendor_id == vendor_id && d.device_id == device_id)
        .cloned()
}

/// Find first device of a class
pub fn find_first(class_code: u8) -> Option<PciDevice> {
    DEVICES.lock().iter()
        .find(|d| d.class_code == class_code)
        .cloned()
}

/// Enable bus mastering for a device
pub fn enable_bus_master(dev: &PciDevice) {
    let command = config_read16(dev.bus, dev.device, dev.function, 0x04);
    let new_command = command | 0x04; // Bus Master Enable
    config_write(dev.bus, dev.device, dev.function, 0x04, new_command as u32);
    crate::log_debug!("[PCI] Bus mastering enabled for {:02X}:{:02X}.{}", 
        dev.bus, dev.device, dev.function);
}

/// Enable memory space access for a device
pub fn enable_memory_space(dev: &PciDevice) {
    let command = config_read16(dev.bus, dev.device, dev.function, 0x04);
    let new_command = command | 0x02; // Memory Space Enable
    config_write(dev.bus, dev.device, dev.function, 0x04, new_command as u32);
}

/// Enable I/O space access for a device
pub fn enable_io_space(dev: &PciDevice) {
    let command = config_read16(dev.bus, dev.device, dev.function, 0x04);
    let new_command = command | 0x01; // I/O Space Enable
    config_write(dev.bus, dev.device, dev.function, 0x04, new_command as u32);
}

/// Walk PCI capabilities list and find a specific capability type
/// Returns the offset of the capability in PCI config space, or None
pub fn find_capability(dev: &PciDevice, cap_id: u8) -> Option<u8> {
    // Check if capabilities list is supported (status register bit 4)
    let status = config_read16(dev.bus, dev.device, dev.function, 0x06);
    if status & (1 << 4) == 0 {
        return None; // No capabilities
    }
    
    // Capabilities pointer is at offset 0x34
    let mut cap_ptr = config_read8(dev.bus, dev.device, dev.function, 0x34);
    let mut visited = 0u32;
    
    while cap_ptr != 0 && visited < 48 {
        let cap_type = config_read8(dev.bus, dev.device, dev.function, cap_ptr);
        if cap_type == cap_id {
            return Some(cap_ptr);
        }
        cap_ptr = config_read8(dev.bus, dev.device, dev.function, cap_ptr + 1);
        visited += 1;
    }
    
    None
}

/// Find all VirtIO vendor-specific capabilities (cap_id = 0x09)
/// Returns Vec of (cap_offset, cfg_type, bar, offset_within_bar, length)
pub fn find_virtio_capabilities(dev: &PciDevice) -> Vec<(u8, u8, u8, u32, u32)> {
    let mut caps = Vec::new();
    
    // Check if capabilities list is supported
    let status = config_read16(dev.bus, dev.device, dev.function, 0x06);
    if status & (1 << 4) == 0 {
        return caps;
    }
    
    let mut cap_ptr = config_read8(dev.bus, dev.device, dev.function, 0x34);
    let mut visited = 0u32;
    
    while cap_ptr != 0 && visited < 48 {
        let cap_type = config_read8(dev.bus, dev.device, dev.function, cap_ptr);
        
        if cap_type == 0x09 { // Vendor-specific (VirtIO uses this)
            // VirtIO PCI capability structure:
            // +0: cap_vndr (0x09)
            // +1: cap_next
            // +2: cap_len
            // +3: cfg_type (1=common, 2=notify, 3=isr, 4=device, 5=pci_cfg)
            // +4: bar
            // +5..+7: padding
            // +8: offset (u32)
            // +12: length (u32)
            let cfg_type = config_read8(dev.bus, dev.device, dev.function, cap_ptr + 3);
            let bar = config_read8(dev.bus, dev.device, dev.function, cap_ptr + 4);
            let offset = config_read(dev.bus, dev.device, dev.function, cap_ptr + 8);
            let length = config_read(dev.bus, dev.device, dev.function, cap_ptr + 12);
            
            caps.push((cap_ptr, cfg_type, bar, offset, length));
        }
        
        cap_ptr = config_read8(dev.bus, dev.device, dev.function, cap_ptr + 1);
        visited += 1;
    }
    
    caps
}

/// Read notify_off_multiplier from a VirtIO notify capability
pub fn read_notify_off_multiplier(dev: &PciDevice, cap_offset: u8) -> u32 {
    // The notify_off_multiplier is at cap_offset + 16
    config_read(dev.bus, dev.device, dev.function, cap_offset + 16)
}

/// Get summary of detected hardware
pub fn hardware_summary() -> String {
    let devices = DEVICES.lock();
    
    let (mut storage, mut network, mut display, mut usb, mut bridges) = (0, 0, 0, 0, 0);
    for d in devices.iter() {
        match d.class_code {
            class::MASS_STORAGE => storage += 1,
            class::NETWORK => network += 1,
            class::DISPLAY => display += 1,
            class::BRIDGE => bridges += 1,
            class::SERIAL_BUS if d.subclass == serial::USB => usb += 1,
            _ => {}
        }
    }
    
    format!(
        "PCI: {} devices (Storage:{}, Network:{}, Display:{}, USB:{}, Bridges:{})",
        devices.len(), storage, network, display, usb, bridges
    )
}

// ============================================================================
// MSI / MSI-X Support
// ============================================================================

/// PCI capability IDs
pub mod cap_id {
    pub const MSI: u8 = 0x05;
    pub const MSIX: u8 = 0x11;
    pub const PCIE: u8 = 0x10;
}

/// MSI Message Address (for x86_64 LAPIC)
/// Format: 0xFEE0_0000 | (destination_apic_id << 12)
pub fn msi_address(dest_apic_id: u8) -> u32 {
    0xFEE0_0000 | ((dest_apic_id as u32) << 12)
}

/// MSI Message Data
/// For fixed delivery: vector number in bits [7:0], edge trigger
pub fn msi_data(vector: u8) -> u32 {
    vector as u32 // edge-triggered, fixed delivery
}

/// Enable MSI for a device (single-vector, targets BSP LAPIC ID 0)
/// Returns the capability offset used, or None if MSI not supported
pub fn enable_msi(dev: &PciDevice, vector: u8) -> Option<u8> {
    let cap_off = find_capability(dev, cap_id::MSI)?;
    
    // Read MSI Message Control (cap_off + 2)
    let msg_ctrl = config_read16(dev.bus, dev.device, dev.function, cap_off + 2);
    let is_64bit = (msg_ctrl & (1 << 7)) != 0;
    
    // Disable MSI first
    let ctrl_masked = msg_ctrl & !(1u16 << 0); // clear MSI Enable
    config_write(dev.bus, dev.device, dev.function, (cap_off + 2) & 0xFC, 
        (config_read(dev.bus, dev.device, dev.function, (cap_off + 2) & 0xFC) 
            & !(0xFFFF << (((cap_off + 2) & 2) * 8)))
            | ((ctrl_masked as u32) << (((cap_off + 2) & 2) * 8)));
    
    // Write Message Address (cap_off + 4)
    let addr = msi_address(0); // Target BSP (APIC ID 0)
    config_write(dev.bus, dev.device, dev.function, cap_off + 4, addr);
    
    // Write Message Data
    let data_offset = if is_64bit {
        // 64-bit: address upper at +8, data at +12
        config_write(dev.bus, dev.device, dev.function, cap_off + 8, 0);
        cap_off + 12
    } else {
        cap_off + 8
    };
    
    let data = msi_data(vector);
    // Data is 16-bit so we need to handle the aligned write carefully
    let existing = config_read(dev.bus, dev.device, dev.function, data_offset & 0xFC);
    let shift = ((data_offset & 2) * 8) as u32;
    let mask = !(0xFFFF << shift);
    let new_val = (existing & mask) | ((data as u32) << shift);
    config_write(dev.bus, dev.device, dev.function, data_offset & 0xFC, new_val);
    
    // Request single vector (MME = 0)
    let new_ctrl = (msg_ctrl & !(0x7 << 4)) | (1 << 0); // MSI Enable, MME=000 (1 vector)
    let ctrl_existing = config_read(dev.bus, dev.device, dev.function, (cap_off + 2) & 0xFC);
    let ctrl_shift = ((cap_off + 2) & 2) * 8;
    let ctrl_mask = !(0xFFFF << ctrl_shift);
    let ctrl_new = (ctrl_existing & ctrl_mask as u32) | ((new_ctrl as u32) << ctrl_shift);
    config_write(dev.bus, dev.device, dev.function, (cap_off + 2) & 0xFC, ctrl_new);
    
    // Disable legacy INTx (set bit 10 in Command register)
    let cmd = config_read16(dev.bus, dev.device, dev.function, 0x04);
    config_write(dev.bus, dev.device, dev.function, 0x04, (cmd | (1 << 10)) as u32);
    
    crate::serial_println!("[PCI] MSI enabled for {:02X}:{:02X}.{} vector={} {}",
        dev.bus, dev.device, dev.function, vector,
        if is_64bit { "64-bit" } else { "32-bit" });
    
    Some(cap_off)
}

/// Enable MSI-X for a device (single entry, table entry 0)
pub fn enable_msix(dev: &PciDevice, vector: u8) -> Option<u8> {
    let cap_off = find_capability(dev, cap_id::MSIX)?;
    
    // Read MSI-X Message Control (cap_off + 2)
    let msg_ctrl = config_read16(dev.bus, dev.device, dev.function, cap_off + 2);
    let table_size = (msg_ctrl & 0x7FF) + 1;
    
    // Read Table BIR and offset (cap_off + 4)
    let table_info = config_read(dev.bus, dev.device, dev.function, cap_off + 4);
    let table_bir = (table_info & 0x7) as usize;
    let table_offset = (table_info & !0x7) as u64;
    
    // Get BAR address for the MSI-X table
    let bar_addr = match dev.bar_address(table_bir) {
        Some(a) => a,
        None => {
            crate::serial_println!("[PCI] MSI-X: BAR{} not configured", table_bir);
            return None;
        }
    };
    
    // Map the MSI-X table (need at least 16 bytes per entry)
    let table_phys = bar_addr + table_offset;
    let table_size_bytes = (table_size as usize) * 16;
    let table_virt = match crate::memory::map_mmio(table_phys, table_size_bytes.max(4096)) {
        Ok(v) => v,
        Err(e) => {
            crate::serial_println!("[PCI] MSI-X: Failed to map table: {}", e);
            return None;
        }
    };
    
    // Enable MSI-X + mask all vectors first
    let ctrl_val = config_read(dev.bus, dev.device, dev.function, (cap_off + 2) & 0xFC);
    let ctrl_shift = ((cap_off + 2) & 2) * 8;
    // Set bit 15 (Enable) and bit 14 (Function Mask)
    let new_ctrl_bits = (msg_ctrl | (1 << 15) | (1 << 14)) as u32;
    let masked = (ctrl_val & !(0xFFFF << ctrl_shift)) | (new_ctrl_bits << ctrl_shift);
    config_write(dev.bus, dev.device, dev.function, (cap_off + 2) & 0xFC, masked);
    
    // Write entry 0: address low, address high, data, vector control
    let entry_addr = table_virt;
    unsafe {
        // Message Address Low
        core::ptr::write_volatile(entry_addr as *mut u32, msi_address(0));
        // Message Address High
        core::ptr::write_volatile((entry_addr + 4) as *mut u32, 0);
        // Message Data
        core::ptr::write_volatile((entry_addr + 8) as *mut u32, msi_data(vector));
        // Vector Control — unmask (clear bit 0)
        core::ptr::write_volatile((entry_addr + 12) as *mut u32, 0);
    }
    
    // Clear Function Mask (keep Enable set)
    let unmask_ctrl = (msg_ctrl | (1 << 15)) & !(1 << 14);
    let final_val = (ctrl_val & !(0xFFFF << ctrl_shift)) | ((unmask_ctrl as u32) << ctrl_shift);
    config_write(dev.bus, dev.device, dev.function, (cap_off + 2) & 0xFC, final_val);
    
    // Disable legacy INTx
    let cmd = config_read16(dev.bus, dev.device, dev.function, 0x04);
    config_write(dev.bus, dev.device, dev.function, 0x04, (cmd | (1 << 10)) as u32);
    
    crate::serial_println!("[PCI] MSI-X enabled for {:02X}:{:02X}.{} vector={} table_size={}",
        dev.bus, dev.device, dev.function, vector, table_size);
    
    Some(cap_off)
}

/// Try MSI-X first, then MSI. Returns true if either succeeded.
pub fn enable_msi_any(dev: &PciDevice, vector: u8) -> bool {
    if enable_msix(dev, vector).is_some() {
        return true;
    }
    if enable_msi(dev, vector).is_some() {
        return true;
    }
    false
}

/// Disable MSI for a device
pub fn disable_msi(dev: &PciDevice) {
    if let Some(cap_off) = find_capability(dev, cap_id::MSI) {
        let msg_ctrl = config_read16(dev.bus, dev.device, dev.function, cap_off + 2);
        let new_ctrl = msg_ctrl & !(1u16 << 0);
        let existing = config_read(dev.bus, dev.device, dev.function, (cap_off + 2) & 0xFC);
        let shift = ((cap_off + 2) & 2) * 8;
        let mask = !(0xFFFF << shift);
        config_write(dev.bus, dev.device, dev.function, (cap_off + 2) & 0xFC,
            (existing & mask as u32) | ((new_ctrl as u32) << shift));
    }
}

/// Check if device supports MSI or MSI-X
pub fn has_msi_support(dev: &PciDevice) -> (bool, bool) {
    let msi = find_capability(dev, cap_id::MSI).is_some();
    let msix = find_capability(dev, cap_id::MSIX).is_some();
    (msi, msix)
}

// ============================================================================
// BAR Size Detection
// ============================================================================

/// Determine the size of a BAR by writing all 1s and reading back.
/// Returns the size in bytes, or 0 if the BAR is empty/unreadable.
/// WARNING: Temporarily disables the device's memory/IO decoding.
pub fn bar_size(dev: &PciDevice, bar_index: usize) -> u64 {
    if bar_index >= 6 {
        return 0;
    }
    let bar_offset = (0x10 + bar_index * 4) as u8;
    let original = config_read(dev.bus, dev.device, dev.function, bar_offset);
    
    if original == 0 {
        return 0; // BAR not configured
    }
    
    let is_io = original & 1 != 0;
    let is_64bit = !is_io && ((original >> 1) & 0x3) == 2;
    
    // Disable memory/IO decoding while probing
    let cmd = config_read16(dev.bus, dev.device, dev.function, 0x04);
    config_write(dev.bus, dev.device, dev.function, 0x04, (cmd & !0x03) as u32);
    
    // Write all 1s
    config_write(dev.bus, dev.device, dev.function, bar_offset, 0xFFFFFFFF);
    let readback = config_read(dev.bus, dev.device, dev.function, bar_offset);
    // Restore original
    config_write(dev.bus, dev.device, dev.function, bar_offset, original);
    
    let size = if is_io {
        let mask = readback & 0xFFFFFFFC;
        if mask == 0 { 0 } else { ((!mask) + 1) as u64 & 0xFFFF }
    } else if is_64bit && bar_index < 5 {
        let bar_offset_hi = (0x10 + (bar_index + 1) * 4) as u8;
        let original_hi = config_read(dev.bus, dev.device, dev.function, bar_offset_hi);
        config_write(dev.bus, dev.device, dev.function, bar_offset_hi, 0xFFFFFFFF);
        let readback_hi = config_read(dev.bus, dev.device, dev.function, bar_offset_hi);
        config_write(dev.bus, dev.device, dev.function, bar_offset_hi, original_hi);
        
        let full = ((readback_hi as u64) << 32) | (readback & 0xFFFFFFF0) as u64;
        if full == 0 { 0 } else { (!full).wrapping_add(1) }
    } else {
        let mask = readback & 0xFFFFFFF0;
        if mask == 0 { 0 } else { ((!mask) + 1) as u64 }
    };
    
    // Restore command register
    config_write(dev.bus, dev.device, dev.function, 0x04, cmd as u32);
    
    size
}
