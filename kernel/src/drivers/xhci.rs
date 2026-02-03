//! xHCI (eXtensible Host Controller Interface) Driver
//!
//! USB 3.0+ host controller driver supporting HID devices (keyboard, mouse).

use alloc::vec::Vec;
use alloc::vec;
use alloc::boxed::Box;
use alloc::string::String;
use core::ptr;
use core::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use spin::Mutex;

// ============================================================================
// xHCI Register Structures
// ============================================================================

/// xHCI Capability Registers (read-only)
#[repr(C)]
pub struct XhciCapRegs {
    pub caplength: u8,          // 0x00: Capability register length
    pub _reserved: u8,          // 0x01
    pub hciversion: u16,        // 0x02: Interface version
    pub hcsparams1: u32,        // 0x04: Structural params 1
    pub hcsparams2: u32,        // 0x08: Structural params 2
    pub hcsparams3: u32,        // 0x0C: Structural params 3
    pub hccparams1: u32,        // 0x10: Capability params 1
    pub dboff: u32,             // 0x14: Doorbell offset
    pub rtsoff: u32,            // 0x18: Runtime registers offset
    pub hccparams2: u32,        // 0x1C: Capability params 2
}

impl XhciCapRegs {
    pub fn max_slots(&self) -> u8 {
        (self.hcsparams1 & 0xFF) as u8
    }
    
    pub fn max_intrs(&self) -> u16 {
        ((self.hcsparams1 >> 8) & 0x7FF) as u16
    }
    
    pub fn max_ports(&self) -> u8 {
        ((self.hcsparams1 >> 24) & 0xFF) as u8
    }
    
    pub fn context_size(&self) -> usize {
        if (self.hccparams1 & (1 << 2)) != 0 { 64 } else { 32 }
    }
}

/// xHCI Operational Registers
#[repr(C)]
pub struct XhciOpRegs {
    pub usbcmd: u32,            // 0x00: USB Command
    pub usbsts: u32,            // 0x04: USB Status
    pub pagesize: u32,          // 0x08: Page size
    pub _reserved1: [u32; 2],   // 0x0C-0x13
    pub dnctrl: u32,            // 0x14: Device notification control
    pub crcr: u64,              // 0x18: Command ring control
    pub _reserved2: [u32; 4],   // 0x20-0x2F
    pub dcbaap: u64,            // 0x30: Device context base address array pointer
    pub config: u32,            // 0x38: Configure
    // Port registers follow at offset 0x400 from operational base
}

// USBCMD bits
const USBCMD_RUN: u32 = 1 << 0;
const USBCMD_HCRST: u32 = 1 << 1;
const USBCMD_INTE: u32 = 1 << 2;

// USBSTS bits
const USBSTS_HCH: u32 = 1 << 0;  // Host Controller Halted
const USBSTS_CNR: u32 = 1 << 11; // Controller Not Ready

/// Port Status and Control Register
#[repr(C)]
#[derive(Clone, Copy)]
pub struct XhciPortRegs {
    pub portsc: u32,    // Port status and control
    pub portpmsc: u32,  // Port power management
    pub portli: u32,    // Port link info
    pub porthlpmc: u32, // Port hardware LPM control
}

// PORTSC bits
const PORTSC_CCS: u32 = 1 << 0;     // Current Connect Status
const PORTSC_PED: u32 = 1 << 1;     // Port Enabled
const PORTSC_PR: u32 = 1 << 4;      // Port Reset
const PORTSC_PLS_MASK: u32 = 0xF << 5; // Port Link State
const PORTSC_PP: u32 = 1 << 9;      // Port Power
const PORTSC_SPEED_MASK: u32 = 0xF << 10; // Port Speed
const PORTSC_CSC: u32 = 1 << 17;    // Connect Status Change
const PORTSC_PRC: u32 = 1 << 21;    // Port Reset Change

// Port speeds
const SPEED_FULL: u32 = 1;   // USB 1.1 Full Speed (12 Mbps)
const SPEED_LOW: u32 = 2;    // USB 1.1 Low Speed (1.5 Mbps)
const SPEED_HIGH: u32 = 3;   // USB 2.0 High Speed (480 Mbps)
const SPEED_SUPER: u32 = 4;  // USB 3.0 SuperSpeed (5 Gbps)

/// Transfer Request Block (TRB)
#[repr(C, align(16))]
#[derive(Clone, Copy, Default)]
pub struct Trb {
    pub parameter: u64,
    pub status: u32,
    pub control: u32,
}

impl Trb {
    pub fn new() -> Self {
        Self { parameter: 0, status: 0, control: 0 }
    }
    
    pub fn link(next_ring_phys: u64) -> Self {
        Self {
            parameter: next_ring_phys,
            status: 0,
            control: (TRB_TYPE_LINK << 10) | TRB_CYCLE,
        }
    }
    
    pub fn trb_type(&self) -> u8 {
        ((self.control >> 10) & 0x3F) as u8
    }
    
    pub fn cycle_bit(&self) -> bool {
        (self.control & TRB_CYCLE) != 0
    }
}

// TRB types
const TRB_TYPE_NORMAL: u32 = 1;
const TRB_TYPE_SETUP: u32 = 2;
const TRB_TYPE_DATA: u32 = 3;
const TRB_TYPE_STATUS: u32 = 4;
const TRB_TYPE_LINK: u32 = 6;
const TRB_TYPE_EVENT_DATA: u32 = 7;
const TRB_TYPE_NO_OP: u32 = 8;
const TRB_TYPE_ENABLE_SLOT: u32 = 9;
const TRB_TYPE_DISABLE_SLOT: u32 = 10;
const TRB_TYPE_ADDRESS_DEVICE: u32 = 11;
const TRB_TYPE_CONFIGURE_EP: u32 = 12;
const TRB_TYPE_EVALUATE_CTX: u32 = 13;
const TRB_TYPE_RESET_EP: u32 = 14;
const TRB_TYPE_NO_OP_CMD: u32 = 23;

// Event TRB types
const TRB_TYPE_TRANSFER_EVENT: u32 = 32;
const TRB_TYPE_CMD_COMPLETION: u32 = 33;
const TRB_TYPE_PORT_STATUS_CHANGE: u32 = 34;

// TRB control bits
const TRB_CYCLE: u32 = 1 << 0;
const TRB_IOC: u32 = 1 << 5;   // Interrupt on Completion

/// Command Ring (256 TRBs)
#[repr(C, align(64))]
pub struct CommandRing {
    pub trbs: [Trb; 256],
}

/// Event Ring Segment Table Entry
#[repr(C, align(64))]
#[derive(Clone, Copy)]
pub struct ErstEntry {
    pub ring_base: u64,
    pub ring_size: u16,
    pub _reserved: [u16; 3],
}

/// Runtime Registers (per-interrupter)
#[repr(C)]
pub struct XhciIntrRegs {
    pub iman: u32,      // Interrupter Management
    pub imod: u32,      // Interrupter Moderation
    pub erstsz: u32,    // Event Ring Segment Table Size
    pub _reserved: u32,
    pub erstba: u64,    // Event Ring Segment Table Base Address
    pub erdp: u64,      // Event Ring Dequeue Pointer
}

/// Slot Context
#[repr(C, align(32))]
#[derive(Clone, Copy, Default)]
pub struct SlotContext {
    pub data: [u32; 8],
}

/// Endpoint Context  
#[repr(C, align(32))]
#[derive(Clone, Copy, Default)]
pub struct EndpointContext {
    pub data: [u32; 8],
}

/// Device Context (Slot + 31 Endpoints)
#[repr(C, align(64))]
pub struct DeviceContext {
    pub slot: SlotContext,
    pub endpoints: [EndpointContext; 31],
}

/// Input Context (for Address Device)
#[repr(C, align(64))]
pub struct InputContext {
    pub input_control: InputControlContext,
    pub slot: SlotContext,
    pub endpoints: [EndpointContext; 31],
}

#[repr(C, align(32))]
#[derive(Clone, Copy, Default)]
pub struct InputControlContext {
    pub drop_flags: u32,
    pub add_flags: u32,
    pub _reserved: [u32; 6],
}

// ============================================================================
// xHCI Controller State
// ============================================================================

/// USB Device detected by xHCI
#[derive(Clone, Debug)]
pub struct XhciDevice {
    pub slot_id: u8,
    pub port: u8,
    pub speed: u8,
    pub vendor_id: u16,
    pub product_id: u16,
    pub class: u8,
    pub subclass: u8,
    pub protocol: u8,
}

/// xHCI Controller
pub struct XhciController {
    pub base_phys: u64,
    pub base_virt: u64,
    pub cap_regs: *mut XhciCapRegs,
    pub op_regs: *mut XhciOpRegs,
    pub doorbell_base: u64,
    pub runtime_base: u64,
    
    // Device Context Base Address Array
    pub dcbaa: Box<[u64; 256]>,
    pub dcbaa_phys: u64,
    
    // Command Ring
    pub cmd_ring: Box<CommandRing>,
    pub cmd_ring_phys: u64,
    pub cmd_enqueue: usize,
    pub cmd_cycle: bool,
    
    // Event Ring (for interrupter 0)
    pub event_ring: Box<[Trb; 256]>,
    pub event_ring_phys: u64,
    pub erst: Box<[ErstEntry; 1]>,
    pub erst_phys: u64,
    pub event_dequeue: usize,
    pub event_cycle: bool,
    
    // Device contexts
    pub device_contexts: [Option<Box<DeviceContext>>; 256],
    
    // Detected devices
    pub devices: Vec<XhciDevice>,
    
    pub max_slots: u8,
    pub max_ports: u8,
    pub context_size: usize,
    pub initialized: bool,
}

// SAFETY: XhciController is only accessed through a Mutex, ensuring exclusive access.
// The raw pointers point to memory-mapped I/O regions that are valid for the controller's lifetime.
unsafe impl Send for XhciController {}

static CONTROLLER: Mutex<Option<XhciController>> = Mutex::new(None);
static INITIALIZED: AtomicBool = AtomicBool::new(false);

// ============================================================================
// Helper functions
// ============================================================================

fn virt_to_phys(virt: u64) -> u64 {
    let hhdm = crate::memory::hhdm_offset();
    virt.wrapping_sub(hhdm)
}

fn phys_to_virt(phys: u64) -> u64 {
    let hhdm = crate::memory::hhdm_offset();
    phys.wrapping_add(hhdm)
}

// ============================================================================
// xHCI Initialization
// ============================================================================

/// Initialize xHCI controller
pub fn init(bar0: u64) -> bool {
    if bar0 == 0 || bar0 == 0xFFFFFFFF {
        crate::serial_println!("[xHCI] Invalid BAR0");
        return false;
    }
    
    crate::serial_println!("[xHCI] Initializing controller at phys {:#x}", bar0);
    
    // Map MMIO region
    let base_virt = match crate::memory::map_mmio(bar0, 0x4000) {
        Ok(v) => v,
        Err(e) => {
            crate::serial_println!("[xHCI] Failed to map MMIO: {}", e);
            return false;
        }
    };
    
    crate::serial_println!("[xHCI] Mapped to virt {:#x}", base_virt);
    
    // Read capability registers
    let cap_regs = base_virt as *mut XhciCapRegs;
    let cap = unsafe { &*cap_regs };
    
    let caplength = cap.caplength as u64;
    let version = cap.hciversion;
    let max_slots = cap.max_slots();
    let max_ports = cap.max_ports();
    let context_size = cap.context_size();
    
    crate::serial_println!("[xHCI] Version: {}.{}", version >> 8, version & 0xFF);
    crate::serial_println!("[xHCI] Max slots: {}, Max ports: {}, Context size: {}", 
        max_slots, max_ports, context_size);
    
    // Get operational registers base
    let op_base = base_virt + caplength;
    let op_regs = op_base as *mut XhciOpRegs;
    
    // Get doorbell and runtime register bases
    let doorbell_base = base_virt + (cap.dboff as u64);
    let runtime_base = base_virt + (cap.rtsoff as u64);
    
    // Halt controller if running
    let op = unsafe { &mut *op_regs };
    if (op.usbsts & USBSTS_HCH) == 0 {
        crate::serial_println!("[xHCI] Halting controller...");
        op.usbcmd &= !USBCMD_RUN;
        
        // Wait for halt
        for _ in 0..1000 {
            if (op.usbsts & USBSTS_HCH) != 0 {
                break;
            }
            for _ in 0..10000 { core::hint::spin_loop(); }
        }
    }
    
    // Reset controller
    crate::serial_println!("[xHCI] Resetting controller...");
    op.usbcmd |= USBCMD_HCRST;
    
    // Wait for reset to complete
    for _ in 0..1000 {
        if (op.usbcmd & USBCMD_HCRST) == 0 && (op.usbsts & USBSTS_CNR) == 0 {
            break;
        }
        for _ in 0..10000 { core::hint::spin_loop(); }
    }
    
    if (op.usbcmd & USBCMD_HCRST) != 0 || (op.usbsts & USBSTS_CNR) != 0 {
        crate::serial_println!("[xHCI] Reset failed!");
        return false;
    }
    
    crate::serial_println!("[xHCI] Reset complete");
    
    // Allocate DCBAA (Device Context Base Address Array)
    let mut dcbaa = Box::new([0u64; 256]);
    let dcbaa_phys = virt_to_phys(dcbaa.as_ptr() as u64);
    
    // Allocate Command Ring
    let mut cmd_ring = Box::new(CommandRing { trbs: [Trb::new(); 256] });
    let cmd_ring_phys = virt_to_phys(cmd_ring.trbs.as_ptr() as u64);
    
    // Set up link TRB at end of command ring
    cmd_ring.trbs[255] = Trb::link(cmd_ring_phys);
    
    // Allocate Event Ring
    let event_ring = Box::new([Trb::new(); 256]);
    let event_ring_phys = virt_to_phys(event_ring.as_ptr() as u64);
    
    // Allocate Event Ring Segment Table
    let mut erst = Box::new([ErstEntry {
        ring_base: event_ring_phys,
        ring_size: 256,
        _reserved: [0; 3],
    }]);
    let erst_phys = virt_to_phys(erst.as_ptr() as u64);
    
    // Configure controller
    op.config = max_slots as u32;
    
    // Set DCBAA pointer
    op.dcbaap = dcbaa_phys;
    
    // Set Command Ring Control Register (with cycle bit = 1)
    op.crcr = cmd_ring_phys | 1;
    
    // Set up Interrupter 0
    let intr_regs = (runtime_base + 0x20) as *mut XhciIntrRegs;
    let intr = unsafe { &mut *intr_regs };
    
    intr.erstsz = 1;  // One segment
    intr.erstba = erst_phys;
    intr.erdp = event_ring_phys;
    intr.iman = 0;    // Disable interrupts (we'll poll)
    intr.imod = 0;
    
    // Enable interrupts and run
    op.usbcmd = USBCMD_RUN | USBCMD_INTE;
    
    // Wait for controller to start
    for _ in 0..1000 {
        if (op.usbsts & USBSTS_HCH) == 0 {
            break;
        }
        for _ in 0..10000 { core::hint::spin_loop(); }
    }
    
    if (op.usbsts & USBSTS_HCH) != 0 {
        crate::serial_println!("[xHCI] Failed to start controller");
        return false;
    }
    
    crate::serial_println!("[xHCI] Controller running");
    
    // Create device contexts array (all None initially)
    const NONE_CTX: Option<Box<DeviceContext>> = None;
    let device_contexts: [Option<Box<DeviceContext>>; 256] = [NONE_CTX; 256];
    
    // Create controller state
    let controller = XhciController {
        base_phys: bar0,
        base_virt,
        cap_regs,
        op_regs,
        doorbell_base,
        runtime_base,
        dcbaa,
        dcbaa_phys,
        cmd_ring,
        cmd_ring_phys,
        cmd_enqueue: 0,
        cmd_cycle: true,
        event_ring,
        event_ring_phys,
        erst,
        erst_phys,
        event_dequeue: 0,
        event_cycle: true,
        device_contexts,
        devices: Vec::new(),
        max_slots,
        max_ports,
        context_size,
        initialized: true,
    };
    
    *CONTROLLER.lock() = Some(controller);
    INITIALIZED.store(true, Ordering::SeqCst);
    
    // Enumerate root hub ports
    enumerate_ports();
    
    true
}

/// Enumerate root hub ports and detect connected devices
fn enumerate_ports() {
    let mut ctrl = CONTROLLER.lock();
    let controller = match ctrl.as_mut() {
        Some(c) => c,
        None => return,
    };
    
    let port_base = controller.base_virt + 
        (unsafe { &*controller.cap_regs }.caplength as u64) + 0x400;
    
    crate::serial_println!("[xHCI] Enumerating {} ports...", controller.max_ports);
    
    for port_num in 0..controller.max_ports {
        let port_regs = (port_base + (port_num as u64 * 16)) as *mut XhciPortRegs;
        let port = unsafe { &mut *port_regs };
        
        let portsc = port.portsc;
        
        // Check if device connected
        if (portsc & PORTSC_CCS) != 0 {
            let speed = (portsc & PORTSC_SPEED_MASK) >> 10;
            let speed_str = match speed {
                SPEED_LOW => "Low (1.5 Mbps)",
                SPEED_FULL => "Full (12 Mbps)",
                SPEED_HIGH => "High (480 Mbps)",
                SPEED_SUPER => "Super (5 Gbps)",
                _ => "Unknown",
            };
            
            crate::serial_println!("[xHCI] Port {}: Device connected, speed: {}", 
                port_num + 1, speed_str);
            
            // Clear status change bits
            port.portsc = portsc | PORTSC_CSC | PORTSC_PRC;
            
            // If not enabled, try to reset and enable
            if (portsc & PORTSC_PED) == 0 {
                crate::serial_println!("[xHCI] Port {}: Resetting...", port_num + 1);
                
                // Trigger port reset
                port.portsc = (portsc & !PORTSC_PED) | PORTSC_PR;
                
                // Wait for reset to complete
                for _ in 0..100 {
                    for _ in 0..100000 { core::hint::spin_loop(); }
                    let new_portsc = port.portsc;
                    if (new_portsc & PORTSC_PR) == 0 && (new_portsc & PORTSC_PRC) != 0 {
                        // Clear PRC
                        port.portsc = new_portsc | PORTSC_PRC;
                        break;
                    }
                }
                
                let final_portsc = port.portsc;
                if (final_portsc & PORTSC_PED) != 0 {
                    crate::serial_println!("[xHCI] Port {}: Enabled after reset", port_num + 1);
                    
                    // Record device
                    controller.devices.push(XhciDevice {
                        slot_id: 0,  // Will be assigned when we address the device
                        port: port_num + 1,
                        speed: speed as u8,
                        vendor_id: 0,
                        product_id: 0,
                        class: 0,
                        subclass: 0,
                        protocol: 0,
                    });
                }
            } else {
                crate::serial_println!("[xHCI] Port {}: Already enabled", port_num + 1);
                
                controller.devices.push(XhciDevice {
                    slot_id: 0,
                    port: port_num + 1,
                    speed: speed as u8,
                    vendor_id: 0,
                    product_id: 0,
                    class: 0,
                    subclass: 0,
                    protocol: 0,
                });
            }
        }
    }
    
    crate::serial_println!("[xHCI] Found {} connected devices", controller.devices.len());
}

// ============================================================================
// Public API
// ============================================================================

/// Check if xHCI is initialized
pub fn is_initialized() -> bool {
    INITIALIZED.load(Ordering::SeqCst)
}

/// Get number of connected USB devices
pub fn device_count() -> usize {
    CONTROLLER.lock().as_ref().map(|c| c.devices.len()).unwrap_or(0)
}

/// List connected devices
pub fn list_devices() -> Vec<XhciDevice> {
    CONTROLLER.lock().as_ref()
        .map(|c| c.devices.clone())
        .unwrap_or_default()
}

/// Get device info by slot
pub fn get_device(slot_id: u8) -> Option<XhciDevice> {
    CONTROLLER.lock().as_ref()
        .and_then(|c| c.devices.iter().find(|d| d.slot_id == slot_id).cloned())
}
