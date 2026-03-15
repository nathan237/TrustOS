//! VirtIO-Net Driver (Universal Architecture)
//!
//! Implements NetworkDriver trait for virtio-net devices.
//! This driver is auto-loaded when a virtio NIC is detected.

use alloc::boxed::Box;
use alloc::vec::Vec;
use core::sync::atomic::{AtomicU64, Ordering};

use super::{NetworkDriver, NetStats};
use crate::drivers::{Driver, DriverInformation, DriverStatus, DriverCategory};
use crate::pci::PciDevice;

/// Re-export the real virtio-net implementation
use crate::virtio_net::VirtioNet as VirtioNetImpl;

/// Wrapper that implements NetworkDriver trait
pub struct VirtioNetDriver {
    inner: Option<VirtioNetImpl>,
    status: DriverStatus,
    transmit_pkts: AtomicU64,
    receive_pkts: AtomicU64,
    transmit_bytes: AtomicU64,
    receive_bytes: AtomicU64,
}

// Implementation block — defines methods for the type above.
impl VirtioNetDriver {
        // Public function — callable from other modules.
pub fn new() -> Self {
        Self {
            inner: None,
            status: DriverStatus::Unloaded,
            transmit_pkts: AtomicU64::new(0),
            receive_pkts: AtomicU64::new(0),
            transmit_bytes: AtomicU64::new(0),
            receive_bytes: AtomicU64::new(0),
        }
    }
}

// Trait implementation — fulfills a behavioral contract.
impl Driver for VirtioNetDriver {
    fn information(&self) -> &DriverInformation {
        &DRIVER_INFORMATION
    }
    
    fn probe(&mut self, pci_device: &PciDevice) -> Result<(), &'static str> {
        self.status = DriverStatus::Loading;
        
        // Create virtio-net instance
        let mut driver = VirtioNetImpl::new(pci_device)?;
        driver.setup_queues()?;
        driver.setup_receive_buffers()?;
        
        // Route PCI interrupt through IOAPIC so the VirtIO ISR fires
        let irq = pci_device.interrupt_line;
        if irq > 0 && irq < 255 {
            crate::apic::route_pci_interrupt_request(irq, crate::apic::VIRTIO_VECTOR);
            crate::serial_println!("[virtio-net-drv] IRQ {} routed to vector {}", irq, crate::apic::VIRTIO_VECTOR);
        }
        
        // Store I/O base so the interrupt handler can ACK the ISR
        crate::virtio_net::set_iobase_for_interrupt_request(driver.iobase());
        
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

// Trait implementation — fulfills a behavioral contract.
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
        self.transmit_pkts.fetch_add(1, Ordering::Relaxed);
        self.transmit_bytes.fetch_add(data.len() as u64, Ordering::Relaxed);
        Ok(())
    }
    
    fn receive(&mut self) -> Option<Vec<u8>> {
        let driver = self.inner.as_mut()?;
        if let Some(packet) = driver.poll_receive() {
            self.receive_pkts.fetch_add(1, Ordering::Relaxed);
            self.receive_bytes.fetch_add(packet.len() as u64, Ordering::Relaxed);
            Some(packet)
        } else {
            None
        }
    }
    
    fn poll(&mut self) {
        if let Some(driver) = self.inner.as_mut() {
            driver.poll_transmit();
            // RX is polled in receive()
        }
    }
    
    fn stats(&self) -> NetStats {
        NetStats {
            transmit_packets: self.transmit_pkts.load(Ordering::Relaxed),
            receive_packets: self.receive_pkts.load(Ordering::Relaxed),
            transmit_bytes: self.transmit_bytes.load(Ordering::Relaxed),
            receive_bytes: self.receive_bytes.load(Ordering::Relaxed),
            transmit_errors: 0,
            receive_errors: 0,
            transmit_dropped: 0,
            receive_dropped: 0,
        }
    }
}

/// Driver information
const DRIVER_INFORMATION: DriverInformation = DriverInformation {
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
    crate::drivers::register(DRIVER_INFORMATION, || {
        Box::new(VirtioNetDriver::new())
    });
    crate::drivers::net::register_net_driver(DRIVER_INFORMATION, || {
        Box::new(VirtioNetDriver::new())
    });
}
