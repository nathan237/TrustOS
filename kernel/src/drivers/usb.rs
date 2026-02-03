//! USB Driver Subsystem
//! 
//! Provides USB controller initialization and HID device support.
//! Supports EHCI (USB 2.0) and xHCI (USB 3.0) controllers.

use alloc::vec::Vec;
use alloc::string::String;
use spin::Mutex;

/// USB Controller type
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum UsbControllerType {
    UHCI,   // USB 1.0 (12 Mbps)
    OHCI,   // USB 1.1 (12 Mbps)
    EHCI,   // USB 2.0 (480 Mbps)
    XHCI,   // USB 3.0+ (5+ Gbps)
}

/// USB Device class
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum UsbDeviceClass {
    Hub,
    HID,         // Human Interface Device (keyboard, mouse)
    MassStorage,
    Audio,
    Video,
    Printer,
    Other(u8),
}

/// USB Device info
#[derive(Clone, Debug)]
pub struct UsbDevice {
    pub address: u8,
    pub class: UsbDeviceClass,
    pub vendor_id: u16,
    pub product_id: u16,
    pub manufacturer: String,
    pub product: String,
}

/// USB Controller state
pub struct UsbController {
    pub controller_type: UsbControllerType,
    pub base_addr: u64,
    pub devices: Vec<UsbDevice>,
    pub initialized: bool,
}

static CONTROLLERS: Mutex<Vec<UsbController>> = Mutex::new(Vec::new());

/// Check if any USB controller is initialized
pub fn is_initialized() -> bool {
    CONTROLLERS.lock().iter().any(|c| c.initialized)
}

/// Initialize EHCI controller
pub fn init_ehci(base: u64) -> bool {
    if base == 0 || base == 0xFFFFFFFF {
        return false;
    }
    
    crate::serial_println!("[USB] EHCI controller at {:#x}", base);
    
    // EHCI initialization would go here
    // For now, we just register the controller
    
    let controller = UsbController {
        controller_type: UsbControllerType::EHCI,
        base_addr: base,
        devices: Vec::new(),
        initialized: false,  // Not fully implemented yet
    };
    
    CONTROLLERS.lock().push(controller);
    
    // TODO: Full EHCI initialization:
    // 1. Reset controller (USBCMD)
    // 2. Set up periodic/async schedules
    // 3. Enable interrupts
    // 4. Start controller
    // 5. Enumerate devices on root ports
    
    false  // Not fully implemented
}

/// Initialize xHCI controller
pub fn init_xhci(base: u64) -> bool {
    if base == 0 || base == 0xFFFFFFFF {
        return false;
    }
    
    crate::serial_println!("[USB] xHCI controller at {:#x}", base);
    
    let controller = UsbController {
        controller_type: UsbControllerType::XHCI,
        base_addr: base,
        devices: Vec::new(),
        initialized: false,  // Not fully implemented yet
    };
    
    CONTROLLERS.lock().push(controller);
    
    // TODO: Full xHCI initialization:
    // 1. Read capability registers
    // 2. Reset controller
    // 3. Set up device context array
    // 4. Set up command ring
    // 5. Set up event ring
    // 6. Enable interrupts
    // 7. Run controller
    // 8. Enumerate root hub ports
    
    false  // Not fully implemented
}

/// Enumerate USB devices (stub)
pub fn enumerate_devices() -> Vec<UsbDevice> {
    CONTROLLERS.lock()
        .iter()
        .flat_map(|c| c.devices.clone())
        .collect()
}

/// Get controller count
pub fn controller_count() -> usize {
    CONTROLLERS.lock().len()
}

/// Check for HID devices
pub fn has_hid_devices() -> bool {
    CONTROLLERS.lock()
        .iter()
        .any(|c| c.devices.iter().any(|d| d.class == UsbDeviceClass::HID))
}
