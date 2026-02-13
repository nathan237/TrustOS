//! Driver Framework
//!
//! Universal driver architecture like Linux/Windows.
//! Provides traits and registration for hardware drivers.

pub mod net;
pub mod ahci;
pub mod ata;
pub mod usb;
pub mod xhci;
pub mod input;
pub mod pci_ids;
pub mod partition;
pub mod virtio_gpu;
pub mod hda;

use alloc::boxed::Box;
use alloc::string::String;
use alloc::vec::Vec;
use spin::Mutex;

/// Driver categories
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DriverCategory {
    Network,
    Storage,
    Display,
    Input,
    Audio,
    USB,
    Other,
}

/// Driver information
#[derive(Debug, Clone)]
pub struct DriverInfo {
    pub name: &'static str,
    pub version: &'static str,
    pub author: &'static str,
    pub category: DriverCategory,
    pub vendor_ids: &'static [(u16, u16)],  // (vendor, device) pairs
}

/// Driver status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DriverStatus {
    Unloaded,
    Loading,
    Running,
    Error,
    Suspended,
}

/// Base trait for all drivers
pub trait Driver: Send + Sync {
    /// Get driver information
    fn info(&self) -> &DriverInfo;
    
    /// Initialize the driver with a PCI device
    fn probe(&mut self, pci_device: &crate::pci::PciDevice) -> Result<(), &'static str>;
    
    /// Start the driver
    fn start(&mut self) -> Result<(), &'static str>;
    
    /// Stop the driver
    fn stop(&mut self) -> Result<(), &'static str>;
    
    /// Get driver status
    fn status(&self) -> DriverStatus;
    
    /// Handle interrupt
    fn handle_interrupt(&mut self) {}
}

/// Registered driver entry
struct RegisteredDriver {
    info: DriverInfo,
    factory: fn() -> Box<dyn Driver>,
}

/// Driver registry
static REGISTRY: Mutex<Vec<RegisteredDriver>> = Mutex::new(Vec::new());

/// Loaded drivers
static LOADED: Mutex<Vec<Box<dyn Driver>>> = Mutex::new(Vec::new());

/// Register a driver factory
pub fn register(info: DriverInfo, factory: fn() -> Box<dyn Driver>) {
    let mut registry = REGISTRY.lock();
    crate::log_debug!("[DRIVERS] Registered: {} v{}", info.name, info.version);
    registry.push(RegisteredDriver { info, factory });
}

/// Find and load driver for a PCI device
pub fn probe_device(pci_dev: &crate::pci::PciDevice) -> Option<usize> {
    let registry = REGISTRY.lock();
    
    for entry in registry.iter() {
        // Network drivers are handled by drivers::net::probe_device
        if entry.info.category == DriverCategory::Network {
            continue;
        }
        for &(vendor, device) in entry.info.vendor_ids {
            // Match vendor:device or vendor:any (0xFFFF)
            if pci_dev.vendor_id == vendor && 
               (device == 0xFFFF || pci_dev.device_id == device) {
                
                // Create driver instance
                let mut driver = (entry.factory)();
                
                // Try to probe
                match driver.probe(pci_dev) {
                    Ok(()) => {
                        crate::log!("[DRIVERS] Loaded {} for {:04X}:{:04X}",
                            entry.info.name, pci_dev.vendor_id, pci_dev.device_id);
                        
                        // Start driver
                        if let Err(e) = driver.start() {
                            crate::log_warn!("[DRIVERS] Failed to start {}: {}", entry.info.name, e);
                            return None;
                        }
                        
                        let mut loaded = LOADED.lock();
                        let idx = loaded.len();
                        loaded.push(driver);
                        return Some(idx);
                    }
                    Err(e) => {
                        crate::log_debug!("[DRIVERS] {} probe failed: {}", entry.info.name, e);
                    }
                }
            }
        }
    }
    
    None
}

/// Get loaded driver count
pub fn loaded_count() -> usize {
    LOADED.lock().len()
}

/// List all registered drivers
pub fn list_registered() -> Vec<DriverInfo> {
    REGISTRY.lock().iter().map(|e| e.info.clone()).collect()
}

/// Initialize driver subsystem and register built-in drivers
pub fn init() {
    crate::log!("[DRIVERS] Initializing driver framework...");
    
    // Register network drivers
    net::register_drivers();
    
    // Initialize input subsystem
    input::init();
    
    let count = REGISTRY.lock().len();
    crate::log!("[DRIVERS] {} drivers registered", count);
}

/// Probe storage controllers (AHCI, IDE)
pub fn probe_storage() {
    crate::serial_println!("[DRIVERS] Probing storage controllers...");
    
    let devices = crate::pci::get_devices();
    
    for dev in &devices {
        // AHCI Controller (class 0x01, subclass 0x06, prog_if 0x01)
        if dev.class_code == 0x01 && dev.subclass == 0x06 && dev.prog_if == 0x01 {
            crate::serial_println!("[AHCI] Controller detected at {:02X}:{:02X}.{} (BAR5={:#x})",
                dev.bus, dev.device, dev.function, dev.bar[5]);
            let bar5 = dev.bar[5] as u64;
            if ahci::init(bar5) {
                crate::serial_println!("[DRIVERS] AHCI controller initialized");
                // Identify all devices to get sector counts
                ahci::identify_all_devices();
            }
        }
        
        // IDE Controller (class 0x01, subclass 0x01)
        if dev.class_code == 0x01 && dev.subclass == 0x01 {
            // Try IDE in compatibility mode
            if ata::init_ide() {
                crate::serial_println!("[DRIVERS] IDE controller initialized");
            }
        }
        
        // USB Controller (class 0x0C, subclass 0x03)
        if dev.class_code == 0x0C && dev.subclass == 0x03 {
            let usb_type = match dev.prog_if {
                0x00 => "UHCI (USB 1.0)",
                0x10 => "OHCI (USB 1.1)",
                0x20 => "EHCI (USB 2.0)",
                0x30 => "xHCI (USB 3.0)",
                _ => "Unknown USB",
            };
            crate::serial_println!("[USB] Controller detected: {} at {:02X}:{:02X}.{} (BAR0={:#x})", 
                usb_type, dev.bus, dev.device, dev.function, dev.bar[0]);
            
            // Initialize xHCI controller (USB 3.0)
            if dev.prog_if == 0x30 {
                let bar0 = dev.bar[0] as u64;
                if xhci::init(bar0) {
                    crate::serial_println!("[DRIVERS] xHCI controller initialized with {} devices", 
                        xhci::device_count());
                }
            }
        }
    }
    
    // Fallback: try IDE ports directly (legacy mode)
    if !ahci::is_initialized() && !ata::is_initialized() {
        crate::serial_println!("[DRIVERS] Trying legacy IDE ports...");
        let _ = ata::init_ide();
    }
    
    // Print storage summary
    if ahci::is_initialized() {
        crate::serial_println!("[DRIVERS] Storage: AHCI with {} ports", ahci::get_port_count());
    } else if ata::is_initialized() {
        crate::serial_println!("[DRIVERS] Storage: IDE with {} drives", ata::list_drives().len());
    } else {
        crate::serial_println!("[DRIVERS] Storage: No persistent storage (using RAM disk)");
    }
}

/// Check if any storage is available
pub fn has_storage() -> bool {
    ahci::is_initialized() || ata::is_initialized()
}

/// Auto-detect and load drivers for all PCI devices
pub fn auto_probe() {
    let devices = crate::pci::scan();
    let mut loaded = 0;
    
    for dev in &devices {
        if probe_device(dev).is_some() {
            loaded += 1;
        }
    }
    
    crate::log!("[DRIVERS] Auto-probe complete: {} drivers loaded", loaded);
}
