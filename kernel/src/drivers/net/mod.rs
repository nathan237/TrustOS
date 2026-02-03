//! Network Driver Interface
//!
//! Universal interface for all network drivers.
//! Drivers implement NetworkDriver trait and register themselves.

use alloc::boxed::Box;
use alloc::vec::Vec;
use spin::Mutex;
use core::sync::atomic::{AtomicBool, Ordering};

use super::{Driver, DriverInfo, DriverStatus, DriverCategory, register};
use crate::pci::PciDevice;

/// Network driver trait - extends base Driver
pub trait NetworkDriver: Driver {
    /// Get MAC address
    fn mac_address(&self) -> [u8; 6];
    
    /// Check if link is up
    fn link_up(&self) -> bool;
    
    /// Get link speed in Mbps (0 if unknown)
    fn link_speed(&self) -> u32 { 0 }
    
    /// Send a raw ethernet frame
    fn send(&mut self, data: &[u8]) -> Result<(), &'static str>;
    
    /// Receive a packet (non-blocking)
    fn receive(&mut self) -> Option<Vec<u8>>;
    
    /// Poll for events (TX completion, RX packets)
    fn poll(&mut self);
    
    /// Get statistics
    fn stats(&self) -> NetStats;
    
    /// Enable promiscuous mode
    fn set_promiscuous(&mut self, _enabled: bool) -> Result<(), &'static str> {
        Err("Not supported")
    }
    
    /// Add multicast address
    fn add_multicast(&mut self, _mac: [u8; 6]) -> Result<(), &'static str> {
        Err("Not supported")
    }
}

/// Network statistics
#[derive(Debug, Clone, Copy, Default)]
pub struct NetStats {
    pub tx_packets: u64,
    pub rx_packets: u64,
    pub tx_bytes: u64,
    pub rx_bytes: u64,
    pub tx_errors: u64,
    pub rx_errors: u64,
    pub tx_dropped: u64,
    pub rx_dropped: u64,
}

/// Active network driver
static ACTIVE_DRIVER: Mutex<Option<Box<dyn NetworkDriver>>> = Mutex::new(None);
static DRIVER_ACTIVE: AtomicBool = AtomicBool::new(false);

/// Registered network driver entry
struct RegisteredNetDriver {
    info: DriverInfo,
    factory: fn() -> Box<dyn NetworkDriver>,
}

/// Network driver registry
static NET_REGISTRY: Mutex<Vec<RegisteredNetDriver>> = Mutex::new(Vec::new());

// ============================================================================
// VirtIO-Net Driver Implementation
// ============================================================================

mod virtio;

// ============================================================================
// E1000 Driver Implementation (Intel NICs)
// ============================================================================

mod e1000;

// ============================================================================
// RTL8139 Driver Implementation (Realtek)
// ============================================================================

mod rtl8139;

// ============================================================================
// Public API
// ============================================================================

/// Register all network drivers
pub fn register_drivers() {
    virtio::register();
    e1000::register();
    rtl8139::register();
}

/// Register a network driver factory
pub fn register_net_driver(info: DriverInfo, factory: fn() -> Box<dyn NetworkDriver>) {
    let mut registry = NET_REGISTRY.lock();
    registry.push(RegisteredNetDriver { info, factory });
}

/// Probe and load a network driver for a PCI device
pub fn probe_device(pci_dev: &PciDevice) -> bool {
    let registry = NET_REGISTRY.lock();
    for entry in registry.iter() {
        for &(vendor, device) in entry.info.vendor_ids {
            if pci_dev.vendor_id == vendor && (device == 0xFFFF || pci_dev.device_id == device) {
                let mut driver = (entry.factory)();
                match driver.probe(pci_dev) {
                    Ok(()) => {
                        if let Err(e) = driver.start() {
                            crate::log_warn!("[DRIVERS] Failed to start {}: {}", entry.info.name, e);
                            return false;
                        }
                        *ACTIVE_DRIVER.lock() = Some(driver);
                        DRIVER_ACTIVE.store(true, Ordering::SeqCst);
                        return true;
                    }
                    Err(e) => {
                        crate::log_debug!("[DRIVERS] {} probe failed: {}", entry.info.name, e);
                    }
                }
            }
        }
    }
    false
}

/// Check if we have an active driver
pub fn has_driver() -> bool {
    DRIVER_ACTIVE.load(Ordering::Relaxed)
}

/// Get MAC address from active driver
pub fn get_mac() -> Option<[u8; 6]> {
    ACTIVE_DRIVER.lock().as_ref().map(|d| d.mac_address())
}

/// Check link status
pub fn link_up() -> bool {
    ACTIVE_DRIVER.lock().as_ref().map(|d| d.link_up()).unwrap_or(false)
}

/// Send packet via active driver
pub fn send(data: &[u8]) -> Result<(), &'static str> {
    let mut guard = ACTIVE_DRIVER.lock();
    let driver = guard.as_mut().ok_or("No network driver")?;
    driver.send(data)
}

/// Receive packet from active driver
pub fn receive() -> Option<Vec<u8>> {
    let mut guard = ACTIVE_DRIVER.lock();
    guard.as_mut().and_then(|d| d.receive())
}

/// Poll active driver
pub fn poll() {
    if let Some(driver) = ACTIVE_DRIVER.lock().as_mut() {
        driver.poll();
    }
}

/// Get statistics from active driver
pub fn stats() -> NetStats {
    ACTIVE_DRIVER.lock().as_ref()
        .map(|d| d.stats())
        .unwrap_or_default()
}
