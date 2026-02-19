//! VirtIO-Net Driver (Universal Architecture)
//!
//! Implements NetworkDriver trait for virtio-net devices.
//! This driver is auto-loaded when a virtio NIC is detected.

use alloc::boxed::Box;
use alloc::vec::Vec;
use core::sync::atomic::{AtomicU64, Ordering};

use super::{NetworkDriver, NetStats};
use crate::drivers::{Driver, DriverInfo, DriverStatus, DriverCategory};
use crate::pci::PciDevice;

/// Re-export the real virtio-net implementation
use crate::virtio_net::VirtioNet as VirtioNetImpl;

/// Wrapper that implements NetworkDriver trait
pub struct VirtioNetDriver {
    inner: Option<VirtioNetImpl>,
    status: DriverStatus,
    tx_pkts: AtomicU64,
    rx_pkts: AtomicU64,
    tx_bytes: AtomicU64,
    rx_bytes: AtomicU64,
}

impl VirtioNetDriver {
    pub fn new() -> Self {
        Self {
            inner: None,
            status: DriverStatus::Unloaded,
            tx_pkts: AtomicU64::new(0),
            rx_pkts: AtomicU64::new(0),
            tx_bytes: AtomicU64::new(0),
            rx_bytes: AtomicU64::new(0),
        }
    }
}

impl Driver for VirtioNetDriver {
    fn info(&self) -> &DriverInfo {
        &DRIVER_INFO
    }
    
    fn probe(&mut self, pci_device: &PciDevice) -> Result<(), &'static str> {
        self.status = DriverStatus::Loading;
        
        // Create virtio-net instance
        let mut driver = VirtioNetImpl::new(pci_device)?;
        driver.setup_queues()?;
        driver.setup_rx_buffers()?;
        
        // Route PCI interrupt through IOAPIC so the VirtIO ISR fires
        let irq = pci_device.interrupt_line;
        if irq > 0 && irq < 255 {
            crate::apic::route_pci_irq(irq, crate::apic::VIRTIO_VECTOR);
            crate::serial_println!("[virtio-net-drv] IRQ {} routed to vector {}", irq, crate::apic::VIRTIO_VECTOR);
        }
        
        // Store I/O base so the interrupt handler can ACK the ISR
        crate::virtio_net::set_iobase_for_irq(driver.iobase());
        
        self.inner = Some(driver);
        Ok(())
    }
    
    fn start(&mut self) -> Result<(), &'static str> {
        let driver = self.inner.as_mut().ok_or("Driver not probed")?;
        driver.start()?;
        self.status = DriverStatus::Running;
        Ok(())
    }
    
    fn stop(&mut self) -> Result<(), &'static str> {
        self.status = DriverStatus::Suspended;
        Ok(())
    }
    
    fn status(&self) -> DriverStatus {
        self.status
    }
}

impl NetworkDriver for VirtioNetDriver {
    fn mac_address(&self) -> [u8; 6] {
        self.inner.as_ref()
            .map(|d| d.mac())
            .unwrap_or([0; 6])
    }
    
    fn link_up(&self) -> bool {
        self.inner.as_ref()
            .map(|d| d.is_link_up())
            .unwrap_or(false)
    }
    
    fn send(&mut self, data: &[u8]) -> Result<(), &'static str> {
        let driver = self.inner.as_mut().ok_or("Driver not initialized")?;
        driver.send(data)?;
        self.tx_pkts.fetch_add(1, Ordering::Relaxed);
        self.tx_bytes.fetch_add(data.len() as u64, Ordering::Relaxed);
        Ok(())
    }
    
    fn receive(&mut self) -> Option<Vec<u8>> {
        let driver = self.inner.as_mut()?;
        if let Some(packet) = driver.poll_rx() {
            self.rx_pkts.fetch_add(1, Ordering::Relaxed);
            self.rx_bytes.fetch_add(packet.len() as u64, Ordering::Relaxed);
            Some(packet)
        } else {
            None
        }
    }
    
    fn poll(&mut self) {
        if let Some(driver) = self.inner.as_mut() {
            driver.poll_tx();
            // RX is polled in receive()
        }
    }
    
    fn stats(&self) -> NetStats {
        NetStats {
            tx_packets: self.tx_pkts.load(Ordering::Relaxed),
            rx_packets: self.rx_pkts.load(Ordering::Relaxed),
            tx_bytes: self.tx_bytes.load(Ordering::Relaxed),
            rx_bytes: self.rx_bytes.load(Ordering::Relaxed),
            tx_errors: 0,
            rx_errors: 0,
            tx_dropped: 0,
            rx_dropped: 0,
        }
    }
}

/// Driver information
const DRIVER_INFO: DriverInfo = DriverInfo {
    name: "virtio-net",
    version: "1.0.0",
    author: "T-RustOs Team",
    category: DriverCategory::Network,
    vendor_ids: &[
        (0x1AF4, 0x1000),  // VirtIO Network (legacy)
        (0x1AF4, 0x1041),  // VirtIO Network (modern)
    ],
};

/// Register this driver
pub fn register() {
    crate::drivers::register(DRIVER_INFO, || {
        Box::new(VirtioNetDriver::new())
    });
    crate::drivers::net::register_net_driver(DRIVER_INFO, || {
        Box::new(VirtioNetDriver::new())
    });
}
