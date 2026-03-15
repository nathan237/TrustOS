//! xHCI (eXtensible Host Controller Interface) Driver
//!
//! USB 3.0+ host controller driver supporting HID devices (keyboard, mouse).
//!
//! Implementation covers:
//! - HC initialization (halt → reset → DCBAA/cmd ring/event ring → start)
//! - Command ring submission + doorbell ring
//! - Event ring polling/dequeue
//! - Scratchpad buffer allocation
//! - Enable Slot / Address Device / Configure Endpoint
//! - USB control transfers (GET_DESCRIPTOR, SET_CONFIGURATION, SET_PROTOCOL)
//! - Per-endpoint transfer rings
//! - HID Boot Protocol keyboard + mouse support
//! - Bridge to kernel keyboard/mouse subsystems

use alloc::vec::Vec;
use alloc::vec;
use alloc::boxed::Box;
use alloc::string::String;
use core::ptr;
use core::sync::atomic::{AtomicBool, Ordering};
use spin::Mutex;

// ============================================================================
// xHCI Register Structures
// ============================================================================

/// xHCI Capability Registers (read-only)
#[repr(C)]
// Public structure — visible outside this module.
pub struct XhciCapabilityRegs {
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

// Implementation block — defines methods for the type above.
impl XhciCapabilityRegs {
        // Public function — callable from other modules.
pub fn maximum_slots(&self) -> u8 {
        (self.hcsparams1 & 0xFF) as u8
    }
    
        // Public function — callable from other modules.
pub fn maximum_intrs(&self) -> u16 {
        ((self.hcsparams1 >> 8) & 0x7FF) as u16
    }
    
        // Public function — callable from other modules.
pub fn maximum_ports(&self) -> u8 {
        ((self.hcsparams1 >> 24) & 0xFF) as u8
    }
    
        // Public function — callable from other modules.
pub fn context_size(&self) -> usize {
        if (self.hccparams1 & (1 << 2)) != 0 { 64 } else { 32 }
    }
}

/// xHCI Operational Registers
#[repr(C)]
// Public structure — visible outside this module.
pub struct XhciOperationRegs {
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
// Compile-time constant — evaluated at compilation, zero runtime cost.
const USBCMD_HCRST: u32 = 1 << 1;
// Compile-time constant — evaluated at compilation, zero runtime cost.
const USBCMD_INTE: u32 = 1 << 2;

// USBSTS bits
const USBSTS_HCH: u32 = 1 << 0;  // Host Controller Halted
const USBSTS_CNR: u32 = 1 << 11; // Controller Not Ready

/// Port Status and Control Register
#[repr(C)]
// #[derive] — auto-generates trait implementations at compile time.
#[derive(Clone, Copy)]
// Public structure — visible outside this module.
pub struct XhciPortRegs {
    pub portsc: u32,    // Port status and control
    pub portpmsc: u32,  // Port power management
    pub portli: u32,    // Port link info
    pub porthlpmc: u32, // Port hardware LPM control
}

// PORTSC bits
pub(crate) // Compile-time constant — evaluated at compilation, zero runtime cost.
const PORTSC_CCS: u32 = 1 << 0;     // Current Connect Status
pub(crate) // Compile-time constant — evaluated at compilation, zero runtime cost.
const PORTSC_PED: u32 = 1 << 1;     // Port Enabled
pub(crate) // Compile-time constant — evaluated at compilation, zero runtime cost.
const PORTSC_PR: u32 = 1 << 4;      // Port Reset
pub(crate) // Compile-time constant — evaluated at compilation, zero runtime cost.
const PORTSC_PLS_MASK: u32 = 0xF << 5; // Port Link State
pub(crate) // Compile-time constant — evaluated at compilation, zero runtime cost.
const PORTSC_PP: u32 = 1 << 9;      // Port Power
pub(crate) // Compile-time constant — evaluated at compilation, zero runtime cost.
const PORTSC_SPEED_MASK: u32 = 0xF << 10; // Port Speed
pub(crate) // Compile-time constant — evaluated at compilation, zero runtime cost.
const PORTSC_CSC: u32 = 1 << 17;    // Connect Status Change
pub(crate) // Compile-time constant — evaluated at compilation, zero runtime cost.
const PORTSC_PRC: u32 = 1 << 21;    // Port Reset Change

// Port speeds
const SPEED_FULL: u32 = 1;   // USB 1.1 Full Speed (12 Mbps)
const SPEED_LOW: u32 = 2;    // USB 1.1 Low Speed (1.5 Mbps)
const SPEED_HIGH: u32 = 3;   // USB 2.0 High Speed (480 Mbps)
const SPEED_SUPER: u32 = 4;  // USB 3.0 SuperSpeed (5 Gbps)

/// Transfer Request Block (TRB)
#[repr(C, align(16))]
// #[derive] — auto-generates trait implementations at compile time.
#[derive(Clone, Copy, Default)]
// Public structure — visible outside this module.
pub struct Trb {
    pub parameter: u64,
    pub status: u32,
    pub control: u32,
}

// Implementation block — defines methods for the type above.
impl Trb {
        // Public function — callable from other modules.
pub fn new() -> Self {
        Self { parameter: 0, status: 0, control: 0 }
    }
    
        // Public function — callable from other modules.
pub fn link(next_ring_physical: u64) -> Self {
        Self {
            parameter: next_ring_physical,
            status: 0,
            control: (TRB_TYPE_LINK << 10) | TRB_CYCLE,
        }
    }
    
        // Public function — callable from other modules.
pub fn trb_type(&self) -> u8 {
        ((self.control >> 10) & 0x3F) as u8
    }
    
        // Public function — callable from other modules.
pub fn cycle_bit(&self) -> bool {
        (self.control & TRB_CYCLE) != 0
    }
}

// TRB types
pub(crate) // Compile-time constant — evaluated at compilation, zero runtime cost.
const TRB_TYPE_NORMAL: u32 = 1;
pub(crate) // Compile-time constant — evaluated at compilation, zero runtime cost.
const TRB_TYPE_SETUP: u32 = 2;
pub(crate) // Compile-time constant — evaluated at compilation, zero runtime cost.
const TRB_TYPE_DATA: u32 = 3;
pub(crate) // Compile-time constant — evaluated at compilation, zero runtime cost.
const TRB_TYPE_STATUS: u32 = 4;
pub(crate) // Compile-time constant — evaluated at compilation, zero runtime cost.
const TRB_TYPE_LINK: u32 = 6;
pub(crate) // Compile-time constant — evaluated at compilation, zero runtime cost.
const TRB_TYPE_EVENT_DATA: u32 = 7;
pub(crate) // Compile-time constant — evaluated at compilation, zero runtime cost.
const TRB_TYPE_NO_OPERATION: u32 = 8;
pub(crate) // Compile-time constant — evaluated at compilation, zero runtime cost.
const TRB_TYPE_ENABLE_SLOT: u32 = 9;
pub(crate) // Compile-time constant — evaluated at compilation, zero runtime cost.
const TRB_TYPE_DISABLE_SLOT: u32 = 10;
pub(crate) // Compile-time constant — evaluated at compilation, zero runtime cost.
const TRB_TYPE_ADDRESS_DEVICE: u32 = 11;
pub(crate) // Compile-time constant — evaluated at compilation, zero runtime cost.
const TRB_TYPE_CONFIGURE_EP: u32 = 12;
pub(crate) // Compile-time constant — evaluated at compilation, zero runtime cost.
const TRB_TYPE_EVALUATE_CONTEXT: u32 = 13;
pub(crate) // Compile-time constant — evaluated at compilation, zero runtime cost.
const TRB_TYPE_RESET_EP: u32 = 14;
pub(crate) // Compile-time constant — evaluated at compilation, zero runtime cost.
const TRB_TYPE_NO_OPERATION_COMMAND: u32 = 23;

// Event TRB types
pub(crate) // Compile-time constant — evaluated at compilation, zero runtime cost.
const TRB_TYPE_TRANSFER_EVENT: u32 = 32;
pub(crate) // Compile-time constant — evaluated at compilation, zero runtime cost.
const TRB_TYPE_COMMAND_COMPLETION: u32 = 33;
pub(crate) // Compile-time constant — evaluated at compilation, zero runtime cost.
const TRB_TYPE_PORT_STATUS_CHANGE: u32 = 34;

// TRB control bits
pub(crate) // Compile-time constant — evaluated at compilation, zero runtime cost.
const TRB_CYCLE: u32 = 1 << 0;
pub(crate) // Compile-time constant — evaluated at compilation, zero runtime cost.
const TRB_IOC: u32 = 1 << 5;   // Interrupt on Completion

/// Command Ring (256 TRBs)
#[repr(C, align(64))]
// Public structure — visible outside this module.
pub struct CommandRing {
    pub trbs: [Trb; 256],
}

/// Event Ring Segment Table Entry
#[repr(C, align(64))]
// #[derive] — auto-generates trait implementations at compile time.
#[derive(Clone, Copy)]
// Public structure — visible outside this module.
pub struct ErstEntry {
    pub ring_base: u64,
    pub ring_size: u16,
    pub _reserved: [u16; 3],
}

/// Runtime Registers (per-interrupter)
#[repr(C)]
// Public structure — visible outside this module.
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
// #[derive] — auto-generates trait implementations at compile time.
#[derive(Clone, Copy, Default)]
// Public structure — visible outside this module.
pub struct SlotContext {
    pub data: [u32; 8],
}

/// Endpoint Context  
#[repr(C, align(32))]
// #[derive] — auto-generates trait implementations at compile time.
#[derive(Clone, Copy, Default)]
// Public structure — visible outside this module.
pub struct EndpointContext {
    pub data: [u32; 8],
}

/// Device Context (Slot + 31 Endpoints)
#[repr(C, align(64))]
// Public structure — visible outside this module.
pub struct DeviceContext {
    pub slot: SlotContext,
    pub endpoints: [EndpointContext; 31],
}

/// Input Context (for Address Device)
#[repr(C, align(64))]
// Public structure — visible outside this module.
pub struct InputContext {
    pub input_control: InputControlContext,
    pub slot: SlotContext,
    pub endpoints: [EndpointContext; 31],
}

#[repr(C, align(32))]
// #[derive] — auto-generates trait implementations at compile time.
#[derive(Clone, Copy, Default)]
// Public structure — visible outside this module.
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
// Public structure — visible outside this module.
pub struct XhciDevice {
    pub slot_id: u8,
    pub port: u8,
    pub speed: u8,
    pub vendor_id: u16,
    pub product_id: u16,
    pub class: u8,
    pub subclass: u8,
    pub protocol: u8,
    pub number_configs: u8,
    pub maximum_packet_size: u16,
    pub manufacturer: String,
    pub product: String,
}

/// xHCI Controller
pub struct XhciController {
    pub base_physical: u64,
    pub base_virt: u64,
    pub capability_regs: *mut XhciCapabilityRegs,
    pub operation_regs: *mut XhciOperationRegs,
    pub doorbell_base: u64,
    pub runtime_base: u64,
    
    // Device Context Base Address Array
    pub dcbaa: Box<[u64; 256]>,
    pub dcbaa_physical: u64,
    
    // Command Ring
    pub command_ring: Box<CommandRing>,
    pub command_ring_physical: u64,
    pub command_enqueue: usize,
    pub command_cycle: bool,
    
    // Event Ring (for interrupter 0)
    pub event_ring: Box<[Trb; 256]>,
    pub event_ring_physical: u64,
    pub erst: Box<[ErstEntry; 1]>,
    pub erst_physical: u64,
    pub event_dequeue: usize,
    pub event_cycle: bool,
    
    // Device contexts
    pub device_contexts: [Option<Box<DeviceContext>>; 256],
    
    // Detected devices
    pub devices: Vec<XhciDevice>,
    
    pub maximum_slots: u8,
    pub maximum_ports: u8,
    pub context_size: usize,
    pub initialized: bool,
}

// SAFETY: XhciController is only accessed through a Mutex, ensuring exclusive access.
// The raw pointers point to memory-mapped I/O regions that are valid for the controller's lifetime.
unsafe // Trait implementation — fulfills a behavioral contract.
impl Send for XhciController {}

pub(crate) // Global shared state guarded by a Mutex (mutual exclusion lock).
static CONTROLLER: Mutex<Option<XhciController>> = Mutex::new(None);
pub(crate) // Atomic variable — provides lock-free thread-safe access.
static INITIALIZED: AtomicBool = AtomicBool::new(false);

// ============================================================================
// Helper functions
// ============================================================================

fn virt_to_physical(virt: u64) -> u64 {
    let hhdm = crate::memory::hhdm_offset();
    virt.wrapping_sub(hhdm)
}

// Public function — callable from other modules.
pub fn physical_to_virt(physical: u64) -> u64 {
    let hhdm = crate::memory::hhdm_offset();
    physical.wrapping_add(hhdm)
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
    let base_virt = // Pattern matching — Rust's exhaustive branching construct.
match crate::memory::map_mmio(bar0, 0x4000) {
        Ok(v) => v,
        Err(e) => {
            crate::serial_println!("[xHCI] Failed to map MMIO: {}", e);
            return false;
        }
    };
    
    crate::serial_println!("[xHCI] Mapped to virt {:#x}", base_virt);
    
    // Read capability registers
    let capability_regs = base_virt as *mut XhciCapabilityRegs;
    let capability = // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe { &*capability_regs };
    
    let caplength = capability.caplength as u64;
    let version = capability.hciversion;
    let maximum_slots = capability.maximum_slots();
    let maximum_ports = capability.maximum_ports();
    let context_size = capability.context_size();
    
    crate::serial_println!("[xHCI] Version: {}.{}", version >> 8, version & 0xFF);
    crate::serial_println!("[xHCI] Max slots: {}, Max ports: {}, Context size: {}", 
        maximum_slots, maximum_ports, context_size);
    
    // Get operational registers base
    let operation_base = base_virt + caplength;
    let operation_regs = operation_base as *mut XhciOperationRegs;
    
    // Get doorbell and runtime register bases
    let doorbell_base = base_virt + (capability.dboff as u64);
    let runtime_base = base_virt + (capability.rtsoff as u64);
    
    // ---- xHCI BIOS/OS Handoff (USBLEGSUP) ----
    // Walk Extended Capabilities to find USB Legacy Support and claim ownership
    // before we halt/reset.  Without this, BIOS SMI handlers may interfere.
    let xecp = ((capability.hccparams1 >> 16) & 0xFFFF) as u64;
    if xecp != 0 {
        let mut ecap_pointer = base_virt + (xecp << 2);
        for _ in 0..32 {
            let ecap_value = // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe { core::ptr::read_volatile(ecap_pointer as *// Compile-time constant — evaluated at compilation, zero runtime cost.
const u32) };
            let ecap_id = ecap_value & 0xFF;
            let next_off = (ecap_value >> 8) & 0xFF;

            if ecap_id == 1 {
                // USB Legacy Support capability (USBLEGSUP)
                crate::serial_println!("[xHCI] Found USBLEGSUP at offset {:#x}", ecap_pointer - base_virt);

                let bios_owns = (ecap_value >> 16) & 1;
                if bios_owns != 0 {
                    crate::serial_println!("[xHCI] BIOS owns controller, requesting handoff...");

                    // Set OS Owned Semaphore (bit 24)
                    unsafe { core::ptr::write_volatile(ecap_pointer as *mut u32, ecap_value | (1 << 24)); }

                    // Wait up to ~1 s for BIOS to release (bit 16 clears)
                    let mut ok = false;
                    for i in 0..1000u32 {
                        let v = // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe { core::ptr::read_volatile(ecap_pointer as *// Compile-time constant — evaluated at compilation, zero runtime cost.
const u32) };
                        if (v >> 16) & 1 == 0 {
                            ok = true;
                            crate::serial_println!("[xHCI] BIOS handoff complete ({}ms)", i);
                            break;
                        }
                        for _ in 0..10000 { core::hint::spin_loop(); }
                    }
                    if !ok {
                        crate::serial_println!("[xHCI] WARNING: BIOS handoff timed out, forcing");
                        let v = // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe { core::ptr::read_volatile(ecap_pointer as *// Compile-time constant — evaluated at compilation, zero runtime cost.
const u32) };
                                                // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe { core::ptr::write_volatile(ecap_pointer as *mut u32, (v & !(1u32 << 16)) | (1 << 24)); }
                    }

                    // Disable all USB SMI in USBLEGCTLSTS (offset +4):
                    // zero all enable bits (0, 4, 13-15), status bits are W1C so 0 = no change
                    let ctlsts_pointer = (ecap_pointer + 4) as *mut u32;
                                        // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe { core::ptr::write_volatile(ctlsts_pointer, 0); }
                    crate::serial_println!("[xHCI] USB SMI disabled");
                } else {
                    crate::serial_println!("[xHCI] No BIOS ownership, handoff not needed");
                }
                break;
            }

            if next_off == 0 { break; }
            ecap_pointer += (next_off as u64) << 2;
        }
    }

    // Halt controller if running
    let op = // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe { &mut *operation_regs };
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
    let dcbaa_physical = virt_to_physical(dcbaa.as_pointer() as u64);
    
    // Allocate Command Ring
    let mut command_ring = Box::new(CommandRing { trbs: [Trb::new(); 256] });
    let command_ring_physical = virt_to_physical(command_ring.trbs.as_pointer() as u64);
    
    // Set up link TRB at end of command ring
    command_ring.trbs[255] = Trb::link(command_ring_physical);
    
    // Allocate Event Ring
    let event_ring = Box::new([Trb::new(); 256]);
    let event_ring_physical = virt_to_physical(event_ring.as_pointer() as u64);
    
    // Allocate Event Ring Segment Table
    let mut erst = Box::new([ErstEntry {
        ring_base: event_ring_physical,
        ring_size: 256,
        _reserved: [0; 3],
    }]);
    let erst_physical = virt_to_physical(erst.as_pointer() as u64);
    
    // Configure controller
    op.config = maximum_slots as u32;
    
    // Set DCBAA pointer
    op.dcbaap = dcbaa_physical;
    
    // Set Command Ring Control Register (with cycle bit = 1)
    op.crcr = command_ring_physical | 1;
    
    // Set up Interrupter 0
    let intr_regs = (runtime_base + 0x20) as *mut XhciIntrRegs;
    let intr = // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe { &mut *intr_regs };
    
    intr.erstsz = 1;  // One segment
    intr.erstba = erst_physical;
    intr.erdp = event_ring_physical;
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
    const NONE_CONTEXT: Option<Box<DeviceContext>> = None;
    let device_contexts: [Option<Box<DeviceContext>>; 256] = [NONE_CONTEXT; 256];
    
    // Create controller state
    let controller = XhciController {
        base_physical: bar0,
        base_virt,
        capability_regs,
        operation_regs,
        doorbell_base,
        runtime_base,
        dcbaa,
        dcbaa_physical,
        command_ring,
        command_ring_physical,
        command_enqueue: 0,
        command_cycle: true,
        event_ring,
        event_ring_physical,
        erst,
        erst_physical,
        event_dequeue: 0,
        event_cycle: true,
        device_contexts,
        devices: Vec::new(),
        maximum_slots,
        maximum_ports,
        context_size,
        initialized: true,
    };
    
    *CONTROLLER.lock() = Some(controller);
    INITIALIZED.store(true, Ordering::SeqCst);
    
    // Allocate scratchpad buffers (required by spec)
    {
        let mut controller = CONTROLLER.lock();
        if let Some(c) = controller.as_mut() {
            allocator_scratchpad_buffers(c);
        }
    }
    
    // Initialize per-slot ring storage
    initialize_slot_rings(maximum_slots);
    
    // Enumerate root hub ports
    enumerate_ports();
    
    // Set up all detected devices (Enable Slot → Address → Descriptors → HID)
    setup_devices();
    
    true
}

/// Enumerate root hub ports and detect connected devices
fn enumerate_ports() {
    let mut controller = CONTROLLER.lock();
    let controller = // Pattern matching — Rust's exhaustive branching construct.
match controller.as_mut() {
        Some(c) => c,
        None => return,
    };
    
    let port_base = controller.base_virt + 
        (        // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe { &*controller.capability_regs }.caplength as u64) + 0x400;
    
    crate::serial_println!("[xHCI] Enumerating {} ports...", controller.maximum_ports);
    
    for port_number in 0..controller.maximum_ports {
        let port_regs = (port_base + (port_number as u64 * 16)) as *mut XhciPortRegs;
        let port = // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe { &mut *port_regs };
        
        let portsc = port.portsc;
        
        // Check if device connected
        if (portsc & PORTSC_CCS) != 0 {
            let speed = (portsc & PORTSC_SPEED_MASK) >> 10;
            let speed_str = // Pattern matching — Rust's exhaustive branching construct.
match speed {
                SPEED_LOW => "Low (1.5 Mbps)",
                SPEED_FULL => "Full (12 Mbps)",
                SPEED_HIGH => "High (480 Mbps)",
                SPEED_SUPER => "Super (5 Gbps)",
                _ => "Unknown",
            };
            
            crate::serial_println!("[xHCI] Port {}: Device connected, speed: {}", 
                port_number + 1, speed_str);
            
            // Clear status change bits
            port.portsc = portsc | PORTSC_CSC | PORTSC_PRC;
            
            // If not enabled, try to reset and enable
            if (portsc & PORTSC_PED) == 0 {
                crate::serial_println!("[xHCI] Port {}: Resetting...", port_number + 1);
                
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
                    crate::serial_println!("[xHCI] Port {}: Enabled after reset", port_number + 1);
                    
                    // Record device
                    controller.devices.push(XhciDevice {
                        slot_id: 0,
                        port: port_number + 1,
                        speed: speed as u8,
                        vendor_id: 0,
                        product_id: 0,
                        class: 0,
                        subclass: 0,
                        protocol: 0,
                        number_configs: 0,
                        maximum_packet_size: 0,
                        manufacturer: String::new(),
                        product: String::new(),
                    });
                }
            } else {
                crate::serial_println!("[xHCI] Port {}: Already enabled", port_number + 1);
                
                controller.devices.push(XhciDevice {
                    slot_id: 0,
                    port: port_number + 1,
                    speed: speed as u8,
                    vendor_id: 0,
                    product_id: 0,
                    class: 0,
                    subclass: 0,
                    protocol: 0,
                    number_configs: 0,
                    maximum_packet_size: 0,
                    manufacturer: String::new(),
                    product: String::new(),
                });
            }
        }
    }
    
    crate::serial_println!("[xHCI] Found {} connected devices", controller.devices.len());
}

// ============================================================================
// Command Ring Submission
// ============================================================================

/// Enqueue a TRB on the command ring and ring doorbell 0
pub(crate) fn submit_command(controller: &mut XhciController, trb: Trb) {
    let index = controller.command_enqueue;
    
    // Write TRB with correct cycle bit
    let mut cmd = trb;
    if controller.command_cycle {
        cmd.control |= TRB_CYCLE;
    } else {
        cmd.control &= !TRB_CYCLE;
    }
    
    controller.command_ring.trbs[index] = cmd;
    
    // Advance enqueue pointer
    controller.command_enqueue += 1;
    if controller.command_enqueue >= 255 {
        // Wrap: update link TRB cycle bit and reset
        let link_controller = (TRB_TYPE_LINK << 10) | if controller.command_cycle { TRB_CYCLE } else { 0 } | (1 << 1); // Toggle Cycle
        controller.command_ring.trbs[255].control = link_controller;
        controller.command_ring.trbs[255].parameter = controller.command_ring_physical;
        controller.command_cycle = !controller.command_cycle;
        controller.command_enqueue = 0;
    }
    
    // Ring doorbell 0 (Host Controller Command)
    unsafe {
        let db = controller.doorbell_base as *mut u32;
        ptr::write_volatile(db, 0);
    }
}

/// Poll event ring for a Command Completion Event. Returns (completion_code, slot_id, parameter).
fn wait_command_completion(controller: &mut XhciController) -> Option<(u8, u8, u64)> {
    for _ in 0..2_000_000u32 {
        let index = controller.event_dequeue;
        let trb = controller.event_ring[index];
        
        // Check phase bit
        let phase = (trb.control & TRB_CYCLE) != 0;
        if phase == controller.event_cycle {
            // Advance dequeue
            controller.event_dequeue += 1;
            if controller.event_dequeue >= 256 {
                controller.event_dequeue = 0;
                controller.event_cycle = !controller.event_cycle;
            }
            
            // Update ERDP
            let erdp_physical = controller.event_ring_physical + (controller.event_dequeue as u64 * 16);
            let intr_regs = (controller.runtime_base + 0x20) as *mut XhciIntrRegs;
                        // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe {
                (*intr_regs).erdp = erdp_physical | (1 << 3); // EHB bit to clear busy
            }
            
            let trb_type = (trb.control >> 10) & 0x3F;
            
            if trb_type == TRB_TYPE_COMMAND_COMPLETION {
                let completion_code = ((trb.status >> 24) & 0xFF) as u8;
                let slot_id = ((trb.control >> 24) & 0xFF) as u8;
                return Some((completion_code, slot_id, trb.parameter));
            }
            // Port Status Change or Transfer Event — skip for now
            continue;
        }
        core::hint::spin_loop();
    }
    None
}

/// Poll event ring for a Transfer Event. Returns (completion_code, transfer_length, endpoint_id).
fn wait_transfer_event(controller: &mut XhciController) -> Option<(u8, u32, u8)> {
    for _ in 0..5_000_000u32 {
        let index = controller.event_dequeue;
        let trb = controller.event_ring[index];
        
        let phase = (trb.control & TRB_CYCLE) != 0;
        if phase == controller.event_cycle {
            controller.event_dequeue += 1;
            if controller.event_dequeue >= 256 {
                controller.event_dequeue = 0;
                controller.event_cycle = !controller.event_cycle;
            }
            
            let erdp_physical = controller.event_ring_physical + (controller.event_dequeue as u64 * 16);
            let intr_regs = (controller.runtime_base + 0x20) as *mut XhciIntrRegs;
                        // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe {
                (*intr_regs).erdp = erdp_physical | (1 << 3);
            }
            
            let trb_type = (trb.control >> 10) & 0x3F;
            
            if trb_type == TRB_TYPE_TRANSFER_EVENT {
                let completion_code = ((trb.status >> 24) & 0xFF) as u8;
                let transfer_length = trb.status & 0xFFFFFF;
                let endpoint_id = ((trb.control >> 16) & 0x1F) as u8;
                return Some((completion_code, transfer_length, endpoint_id));
            }
            if trb_type == TRB_TYPE_COMMAND_COMPLETION {
                let completion_code = ((trb.status >> 24) & 0xFF) as u8;
                let slot_id = ((trb.control >> 24) & 0xFF) as u8;
                return Some((completion_code, 0, slot_id));
            }
            continue;
        }
        core::hint::spin_loop();
    }
    None
}

// ============================================================================
// Transfer Ring Management
// ============================================================================

/// A per-endpoint transfer ring (256 TRBs)
pub(crate) struct TransferRing {
    trbs: Box<[Trb; 256]>,
    pub(crate) physical: u64,
    pub(crate) enqueue: usize,
    pub(crate) cycle: bool,
}

// Implementation block — defines methods for the type above.
impl TransferRing {
    pub(crate) fn new() -> Option<Self> {
        let trbs = Box::new([Trb::new(); 256]);
        let physical = virt_to_physical(trbs.as_pointer() as u64);
        Some(Self { trbs, physical, enqueue: 0, cycle: true })
    }
    
    pub(crate) fn enqueue_trb(&mut self, mut trb: Trb) {
        if self.cycle {
            trb.control |= TRB_CYCLE;
        } else {
            trb.control &= !TRB_CYCLE;
        }
        self.trbs[self.enqueue] = trb;
        self.enqueue += 1;
        if self.enqueue >= 255 {
            // Link TRB
            let link_controller = (TRB_TYPE_LINK << 10) | if self.cycle { TRB_CYCLE } else { 0 } | (1 << 1);
            self.trbs[255].control = link_controller;
            self.trbs[255].parameter = self.physical;
            self.cycle = !self.cycle;
            self.enqueue = 0;
        }
    }
}

/// Per-slot transfer rings (slot → endpoint_id → TransferRing)
/// We store only the control endpoint (EP0, DCI=1) and one interrupt IN endpoint per slot
pub(crate) struct SlotRings {
    pub(crate) ep0: TransferRing,         // Control endpoint (DCI 1)
    pub(crate) interrupt_in: Option<TransferRing>, // HID interrupt IN endpoint
    pub(crate) interrupt_dci: u8,         // DCI of the interrupt IN endpoint
    pub(crate) bulk_in: Option<TransferRing>,    // Bulk IN endpoint (mass storage)
    pub(crate) bulk_in_dci: u8,
    pub(crate) bulk_out: Option<TransferRing>,   // Bulk OUT endpoint (mass storage)
    pub(crate) bulk_out_dci: u8,
}

// ============================================================================
// Scratchpad Buffer Allocation
// ============================================================================

fn allocator_scratchpad_buffers(controller: &mut XhciController) {
    let capability = // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe { &*controller.capability_regs };
    let hcsparams2 = capability.hcsparams2;
    
    let hi = ((hcsparams2 >> 21) & 0x1F) as u32;
    let lo = ((hcsparams2 >> 27) & 0x1F) as u32;
    let number_bufs = (hi << 5) | lo;
    
    if number_bufs == 0 {
        return;
    }
    
    crate::serial_println!("[xHCI] Allocating {} scratchpad buffers", number_bufs);
    
    // Allocate scratchpad buffer array (array of physical page addresses)
    let array_physical = // Pattern matching — Rust's exhaustive branching construct.
match crate::memory::frame::allocator_frame_zeroed() {
        Some(p) => p,
        None => { crate::serial_println!("[xHCI] OOM for scratchpad array"); return; }
    };
    let array_virt = physical_to_virt(array_physical) as *mut u64;
    
    // Allocate each scratchpad page
    for i in 0..number_bufs.minimum(512) as usize {
        if let Some(page_physical) = crate::memory::frame::allocator_frame_zeroed() {
                        // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe { ptr::write_volatile(array_virt.add(i), page_physical); }
        }
    }
    
    // DCBAA[0] = scratchpad buffer array physical address
    controller.dcbaa[0] = array_physical;
}

// ============================================================================
// USB Device Addressing (Enable Slot + Address Device)
// ============================================================================

/// Per-slot state tracked outside the main controller to avoid borrow issues
pub(crate) // Global shared state guarded by a Mutex (mutual exclusion lock).
static SLOT_RINGS: Mutex<Vec<Option<SlotRings>>> = Mutex::new(Vec::new());

fn initialize_slot_rings(maximum_slots: u8) {
    let mut rings = SLOT_RINGS.lock();
    rings.clear();
    for _ in 0..=maximum_slots {
        rings.push(None);
    }
}

/// Enable Slot command → returns slot_id
fn enable_slot(controller: &mut XhciController) -> Option<u8> {
    let trb = Trb {
        parameter: 0,
        status: 0,
        control: (TRB_TYPE_ENABLE_SLOT << 10),
    };
    
    submit_command(controller, trb);
    
    if let Some((cc, slot_id, _parameter)) = wait_command_completion(controller) {
        if cc == 1 { // Success
            crate::serial_println!("[xHCI] Enable Slot → slot_id={}", slot_id);
            return Some(slot_id);
        }
        crate::serial_println!("[xHCI] Enable Slot failed: cc={}", cc);
    }
    None
}

/// Address Device command — sets up input context and issues the command
fn address_device(controller: &mut XhciController, slot_id: u8, port_number: u8, speed: u8) -> bool {
    // Allocate device context
    let device_context = Box::new(DeviceContext {
        slot: SlotContext::default(),
        endpoints: [EndpointContext::default(); 31],
    });
    let device_context_physical = virt_to_physical(&*device_context as *// Compile-time constant — evaluated at compilation, zero runtime cost.
const _ as u64);
    controller.dcbaa[slot_id as usize] = device_context_physical;
    controller.device_contexts[slot_id as usize] = Some(device_context);
    
    // Allocate transfer ring for EP0 (control endpoint, DCI=1)
    let ep0_ring = // Pattern matching — Rust's exhaustive branching construct.
match TransferRing::new() {
        Some(r) => r,
        None => { crate::serial_println!("[xHCI] OOM for EP0 ring"); return false; }
    };
    let ep0_ring_physical = ep0_ring.physical;
    
    // Allocate input context (on heap, page-aligned conceptually)
    let input_context_physical = // Pattern matching — Rust's exhaustive branching construct.
match crate::memory::frame::allocator_frame_zeroed() {
        Some(p) => p,
        None => { crate::serial_println!("[xHCI] OOM for input context"); return false; }
    };
    let input_context_virt = physical_to_virt(input_context_physical);
    let context_size = controller.context_size;
    
    // Input Control Context: Add Slot (bit 0) + Add EP0 (bit 1)
    unsafe {
        let icc = input_context_virt as *mut u32;
        ptr::write_volatile(icc.add(1), 0x3); // add_flags: Slot (0) + EP0 (1)
    }
    
    // Slot Context (at offset context_size from input context base)
    let slot_context_virt = input_context_virt + context_size as u64;
    let maximum_packet = // Pattern matching — Rust's exhaustive branching construct.
match speed as u32 {
        SPEED_LOW => 8u16,
        SPEED_FULL => 8,
        SPEED_HIGH => 64,
        SPEED_SUPER => 512,
        _ => 64,
    };
    
        // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe {
        let slot = slot_context_virt as *mut u32;
        // DW0: Route String (0) | Speed (20:23) | Context Entries = 1 (27:31)
        let speed_field = (speed as u32) << 20;
        let context_entries = 1u32 << 27; // Only EP0
        ptr::write_volatile(slot, speed_field | context_entries);
        // DW1: Root Hub Port Number (16:23)
        ptr::write_volatile(slot.add(1), (port_number as u32) << 16);
    }
    
    // Endpoint 0 Context (at offset 2 * context_size) — DCI=1
    let ep0_context_virt = input_context_virt + (2 * context_size) as u64;
        // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe {
        let ep = ep0_context_virt as *mut u32;
        // DW1: CErr=3 (1:2) | EP Type=4 (Control Bidirectional, 3:5) | Max Packet Size (16:31)
        let ep_type_control = 4u32 << 3;
        let cerr = 3u32 << 1;
        let mps = (maximum_packet as u32) << 16;
        ptr::write_volatile(ep.add(1), cerr | ep_type_control | mps);
        // DW2-3: TR Dequeue Pointer (64-bit, with DCS=1 in bit 0)
        let tr_pointer = ep0_ring_physical | 1; // DCS = 1
        ptr::write_volatile(ep.add(2) as *mut u64, tr_pointer);
        // DW4: Average TRB Length = 8 (for control)
        ptr::write_volatile(ep.add(4), 8);
    }
    
    // Store the EP0 ring
    {
        let mut rings = SLOT_RINGS.lock();
        if (slot_id as usize) < rings.len() {
            rings[slot_id as usize] = Some(SlotRings {
                ep0: ep0_ring,
                interrupt_in: None,
                interrupt_dci: 0,
                bulk_in: None,
                bulk_in_dci: 0,
                bulk_out: None,
                bulk_out_dci: 0,
            });
        }
    }
    
    // Issue Address Device command
    let trb = Trb {
        parameter: input_context_physical,
        status: 0,
        control: (TRB_TYPE_ADDRESS_DEVICE << 10) | ((slot_id as u32) << 24),
    };
    
    submit_command(controller, trb);
    
    if let Some((cc, _sid, _parameter)) = wait_command_completion(controller) {
        if cc == 1 {
            crate::serial_println!("[xHCI] Address Device slot {} → success", slot_id);
            crate::memory::frame::free_frame(input_context_physical);
            return true;
        }
        crate::serial_println!("[xHCI] Address Device failed: cc={}", cc);
    }
    crate::memory::frame::free_frame(input_context_physical);
    false
}

// ============================================================================
// USB Control Transfers
// ============================================================================

/// USB Setup packet fields
const USB_DIRECTORY_IN: u8 = 0x80;
// Compile-time constant — evaluated at compilation, zero runtime cost.
const USB_DIRECTORY_OUT: u8 = 0x00;
// Compile-time constant — evaluated at compilation, zero runtime cost.
const USB_TYPE_STANDARD: u8 = 0x00;
// Compile-time constant — evaluated at compilation, zero runtime cost.
const USB_RECIP_DEVICE: u8 = 0x00;
// Compile-time constant — evaluated at compilation, zero runtime cost.
const USB_RECIP_INTERFACE: u8 = 0x01;

// Compile-time constant — evaluated at compilation, zero runtime cost.
const USB_REQUEST_GET_DESCRIPTOR: u8 = 0x06;
// Compile-time constant — evaluated at compilation, zero runtime cost.
const USB_REQUEST_SET_CONFIGURATION: u8 = 0x09;
// Compile-time constant — evaluated at compilation, zero runtime cost.
const USB_REQUEST_SET_PROTOCOL: u8 = 0x0B;
// Compile-time constant — evaluated at compilation, zero runtime cost.
const USB_REQUEST_SET_IDLE: u8 = 0x0A;

// Compile-time constant — evaluated at compilation, zero runtime cost.
const USB_DT_DEVICE: u8 = 0x01;
// Compile-time constant — evaluated at compilation, zero runtime cost.
const USB_DT_CONFIGURATION: u8 = 0x02;
// Compile-time constant — evaluated at compilation, zero runtime cost.
const USB_DT_STRING: u8 = 0x03;
// Compile-time constant — evaluated at compilation, zero runtime cost.
const USB_DT_INTERFACE: u8 = 0x04;
// Compile-time constant — evaluated at compilation, zero runtime cost.
const USB_DT_ENDPOINT: u8 = 0x05;
// Compile-time constant — evaluated at compilation, zero runtime cost.
const USB_DT_HID: u8 = 0x21;
// Compile-time constant — evaluated at compilation, zero runtime cost.
const USB_DT_HID_REPORT: u8 = 0x22;

/// Send a USB control transfer (Setup → Data IN → Status OUT)
fn control_transfer_in(
    controller: &mut XhciController,
    slot_id: u8,
    bm_request_type: u8,
    b_request: u8,
    w_value: u16,
    w_index: u16,
    w_length: u16,
    buffer_physical: u64,
) -> Option<u32> {
    let mut rings = SLOT_RINGS.lock();
    let slot_rings = rings.get_mut(slot_id as usize)?.as_mut()?;
    
    // Setup TRB
    let setup_data = (bm_request_type as u64)
        | ((b_request as u64) << 8)
        | ((w_value as u64) << 16)
        | ((w_index as u64) << 32)
        | ((w_length as u64) << 48);
    
    let setup_trb = Trb {
        parameter: setup_data,
        status: 8, // Transfer length = 8 (Setup packet is always 8 bytes)
        control: (TRB_TYPE_SETUP << 10) | (1 << 6) | (3 << 16), // IDT=1, TRT=3 (IN)
    };
    slot_rings.ep0.enqueue_trb(setup_trb);
    
    // Data IN TRB (if w_length > 0)
    if w_length > 0 {
        let data_trb = Trb {
            parameter: buffer_physical,
            status: w_length as u32,
            control: (TRB_TYPE_DATA << 10) | (1 << 16), // DIR=1 (IN)
        };
        slot_rings.ep0.enqueue_trb(data_trb);
    }
    
    // Status OUT TRB
    let status_trb = Trb {
        parameter: 0,
        status: 0,
        control: (TRB_TYPE_STATUS << 10) | TRB_IOC, // DIR=0 (OUT), IOC
    };
    slot_rings.ep0.enqueue_trb(status_trb);
    
    // Ring doorbell for slot (DCI=1 for EP0)
    drop(rings);
        // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe {
        let db = (controller.doorbell_base + (slot_id as u64) * 4) as *mut u32;
        ptr::write_volatile(db, 1); // Target = DCI 1
    }
    
    // Wait for transfer completion
    if let Some((cc, transfer_length, _ep)) = wait_transfer_event(controller) {
        if cc == 1 || cc == 13 { // Success or Short Packet
            return Some(transfer_length);
        }
        crate::serial_println!("[xHCI] Control IN failed: cc={}", cc);
    }
    None
}

/// Send a USB control transfer with no data (Setup → Status IN)
fn control_transfer_nodata(
    controller: &mut XhciController,
    slot_id: u8,
    bm_request_type: u8,
    b_request: u8,
    w_value: u16,
    w_index: u16,
) -> bool {
    let mut rings = SLOT_RINGS.lock();
    let slot_rings = // Pattern matching — Rust's exhaustive branching construct.
match rings.get_mut(slot_id as usize).and_then(|r| r.as_mut()) {
        Some(r) => r,
        None => return false,
    };
    
    let setup_data = (bm_request_type as u64)
        | ((b_request as u64) << 8)
        | ((w_value as u64) << 16)
        | ((w_index as u64) << 32);
    
    let setup_trb = Trb {
        parameter: setup_data,
        status: 8,
        control: (TRB_TYPE_SETUP << 10) | (1 << 6), // IDT=1, TRT=0 (No Data)
    };
    slot_rings.ep0.enqueue_trb(setup_trb);
    
    let status_trb = Trb {
        parameter: 0,
        status: 0,
        control: (TRB_TYPE_STATUS << 10) | TRB_IOC | (1 << 16), // DIR=1 (IN), IOC
    };
    slot_rings.ep0.enqueue_trb(status_trb);
    
    drop(rings);
        // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe {
        let db = (controller.doorbell_base + (slot_id as u64) * 4) as *mut u32;
        ptr::write_volatile(db, 1);
    }
    
    if let Some((cc, _, _)) = wait_transfer_event(controller) {
        return cc == 1;
    }
    false
}

// ============================================================================
// USB Device Enumeration (GET_DESCRIPTOR + SET_CONFIGURATION)
// ============================================================================

/// Get USB Device Descriptor and populate XhciDevice fields
fn get_device_descriptor(controller: &mut XhciController, slot_id: u8, device: &mut XhciDevice) -> bool {
    let buffer_physical = // Pattern matching — Rust's exhaustive branching construct.
match crate::memory::frame::allocator_frame_zeroed() {
        Some(p) => p,
        None => return false,
    };
    let buffer_virt = physical_to_virt(buffer_physical) as *// Compile-time constant — evaluated at compilation, zero runtime cost.
const u8;
    
    // GET_DESCRIPTOR (Device, 18 bytes)
    let result = control_transfer_in(
        controller, slot_id,
        USB_DIRECTORY_IN | USB_TYPE_STANDARD | USB_RECIP_DEVICE,
        USB_REQUEST_GET_DESCRIPTOR,
        (USB_DT_DEVICE as u16) << 8,
        0, 18, buffer_physical,
    );
    
    if result.is_some() {
                // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe {
            let _bcd_usb = ptr::read_unaligned(buffer_virt.add(2) as *// Compile-time constant — evaluated at compilation, zero runtime cost.
const u16);
            device.class = *buffer_virt.add(4);
            device.subclass = *buffer_virt.add(5);
            device.protocol = *buffer_virt.add(6);
            device.maximum_packet_size = *buffer_virt.add(7) as u16;
            device.vendor_id = ptr::read_unaligned(buffer_virt.add(8) as *// Compile-time constant — evaluated at compilation, zero runtime cost.
const u16);
            device.product_id = ptr::read_unaligned(buffer_virt.add(10) as *// Compile-time constant — evaluated at compilation, zero runtime cost.
const u16);
            device.number_configs = *buffer_virt.add(17);
        }
        crate::serial_println!("[xHCI] Device: VID={:04X} PID={:04X} class={:02X}:{:02X}:{:02X}",
            device.vendor_id, device.product_id, device.class, device.subclass, device.protocol);
    }
    
    crate::memory::frame::free_frame(buffer_physical);
    result.is_some()
}

/// Get Configuration Descriptor and find HID interfaces
/// Returns: Vec of (interface_num, subclass, protocol, endpoint_addr, max_packet, interval)
fn get_config_descriptor(controller: &mut XhciController, slot_id: u8) 
    -> Option<Vec<(u8, u8, u8, u8, u8, u16, u8)>> 
{
    let buffer_physical = // Pattern matching — Rust's exhaustive branching construct.
match crate::memory::frame::allocator_frame_zeroed() {
        Some(p) => p,
        None => return None,
    };
    let buffer_virt = physical_to_virt(buffer_physical) as *// Compile-time constant — evaluated at compilation, zero runtime cost.
const u8;
    
    // First read: get total length (just first 9 bytes)
    control_transfer_in(
        controller, slot_id,
        USB_DIRECTORY_IN | USB_TYPE_STANDARD | USB_RECIP_DEVICE,
        USB_REQUEST_GET_DESCRIPTOR,
        (USB_DT_CONFIGURATION as u16) << 8,
        0, 9, buffer_physical,
    )?;
    
    let total_length = // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe { ptr::read_unaligned(buffer_virt.add(2) as *// Compile-time constant — evaluated at compilation, zero runtime cost.
const u16) };
    let read_length = total_length.minimum(4096);
    
    // Second read: get full config descriptor
    control_transfer_in(
        controller, slot_id,
        USB_DIRECTORY_IN | USB_TYPE_STANDARD | USB_RECIP_DEVICE,
        USB_REQUEST_GET_DESCRIPTOR,
        (USB_DT_CONFIGURATION as u16) << 8,
        0, read_length, buffer_physical,
    )?;
    
    let config_value = // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe { *buffer_virt.add(5) };
    let mut interfaces = Vec::new();
    
    // Parse descriptors
    let mut offset = 0usize;
    let mut current_iface = (0u8, 0u8, 0u8, 0u8); // (class, interface_num, subclass, protocol)
    
    while offset + 1 < read_length as usize {
        let len = // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe { *buffer_virt.add(offset) } as usize;
        let descriptor_type = // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe { *buffer_virt.add(offset + 1) };
        
        if len == 0 { break; }
        
                // Pattern matching — Rust's exhaustive branching construct.
match descriptor_type {
            USB_DT_INTERFACE if len >= 9 => {
                let iface_number = // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe { *buffer_virt.add(offset + 2) };
                let iface_class = // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe { *buffer_virt.add(offset + 5) };
                let iface_subclass = // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe { *buffer_virt.add(offset + 6) };
                let iface_protocol = // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe { *buffer_virt.add(offset + 7) };
                
                current_iface = (iface_class, iface_number, iface_subclass, iface_protocol);
                
                if iface_class == 0x03 {
                    crate::serial_println!("[xHCI]   HID interface {}: subclass={} protocol={} ({})",
                        iface_number, iface_subclass, iface_protocol,
                                                // Pattern matching — Rust's exhaustive branching construct.
match iface_protocol { 1 => "keyboard", 2 => "mouse", _ => "other" });
                } else if iface_class == 0x08 {
                    crate::serial_println!("[xHCI]   Mass Storage interface {}: subclass={:#x} protocol={:#x}",
                        iface_number, iface_subclass, iface_protocol);
                }
            }
            USB_DT_ENDPOINT if len >= 7 => {
                let ep_address = // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe { *buffer_virt.add(offset + 2) };
                let ep_attrs = // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe { *buffer_virt.add(offset + 3) };
                let ep_maximum_packet = // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe { ptr::read_unaligned(buffer_virt.add(offset + 4) as *// Compile-time constant — evaluated at compilation, zero runtime cost.
const u16) };
                let ep_interval = // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe { *buffer_virt.add(offset + 6) };
                let ep_type_bits = ep_attrs & 0x03;
                
                let iface_class = current_iface.0;
                if iface_class == 0x03 && ep_type_bits == 3 && (ep_address & 0x80 != 0) {
                    // HID: interrupt IN endpoints only
                    interfaces.push((
                        iface_class, current_iface.1, current_iface.2, current_iface.3,
                        ep_address, ep_maximum_packet & 0x7FF, ep_interval,
                    ));
                } else if iface_class == 0x08 && ep_type_bits == 2 {
                    // Mass storage: bulk IN and OUT endpoints
                    interfaces.push((
                        iface_class, current_iface.1, current_iface.2, current_iface.3,
                        ep_address, ep_maximum_packet & 0x7FF, ep_interval,
                    ));
                }
            }
            _ => {}
        }
        
        offset += len;
    }
    
    // SET_CONFIGURATION
    if !interfaces.is_empty() {
        control_transfer_nodata(
            controller, slot_id,
            USB_DIRECTORY_OUT | USB_TYPE_STANDARD | USB_RECIP_DEVICE,
            USB_REQUEST_SET_CONFIGURATION,
            config_value as u16,
            0,
        );
    }
    
    crate::memory::frame::free_frame(buffer_physical);
    
    if interfaces.is_empty() { None } else { Some(interfaces) }
}

/// Configure an interrupt IN endpoint for HID polling
fn configure_hid_endpoint(
    controller: &mut XhciController,
    slot_id: u8,
    port_number: u8,
    speed: u8,
    ep_address: u8,
    maximum_packet: u16,
    interval: u8,
) -> bool {
    let ep_number = ep_address & 0x0F;
    let dci = (ep_number * 2 + 1) as u8; // IN endpoint DCI
    
    // Allocate interrupt IN transfer ring
    let int_ring = // Pattern matching — Rust's exhaustive branching construct.
match TransferRing::new() {
        Some(r) => r,
        None => return false,
    };
    let int_ring_physical = int_ring.physical;
    
    // Build input context for Configure Endpoint
    let input_physical = // Pattern matching — Rust's exhaustive branching construct.
match crate::memory::frame::allocator_frame_zeroed() {
        Some(p) => p,
        None => return false,
    };
    let input_virt = physical_to_virt(input_physical);
    let context_size = controller.context_size;
    
        // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe {
        let icc = input_virt as *mut u32;
        // Add flags: Slot (bit 0) + the endpoint (bit dci)
        ptr::write_volatile(icc.add(1), 1 | (1u32 << dci));
        
        // Slot Context (must re-specify): update Context Entries to include new DCI
        let slot = (input_virt + context_size as u64) as *mut u32;
        let speed_field = (speed as u32) << 20;
        let context_entries = (dci as u32) << 27;
        ptr::write_volatile(slot, speed_field | context_entries);
        ptr::write_volatile(slot.add(1), (port_number as u32) << 16);
        
        // Endpoint Context at offset (1 + dci) * context_size
        let ep_context = (input_virt + ((1 + dci as usize) * context_size) as u64) as *mut u32;
        
        // DW0: Interval (16:23) — for xHCI, interval = 2^(interval-1) * 125µs
        let xhci_interval = // Pattern matching — Rust's exhaustive branching construct.
match speed as u32 {
            SPEED_HIGH | SPEED_SUPER => interval.maximum(1) as u32,
            _ => {
                // Convert ms to 125µs frames: interval * 8, then log2 + 1
                let frames = (interval as u32).maximum(1) * 8;
                let mut log2 = 0u32;
                let mut v = frames;
                while v > 1 { v >>= 1; log2 += 1; }
                log2 + 1
            }
        };
        ptr::write_volatile(ep_context, (xhci_interval << 16));
        
        // DW1: CErr=3 | EP Type=7 (Interrupt IN) | Max Packet Size
        let ep_type_int_in = 7u32 << 3;
        let cerr = 3u32 << 1;
        let mps = (maximum_packet as u32) << 16;
        ptr::write_volatile(ep_context.add(1), cerr | ep_type_int_in | mps);
        
        // DW2-3: TR Dequeue Pointer
        ptr::write_volatile(ep_context.add(2) as *mut u64, int_ring_physical | 1);
        
        // DW4: Average TRB Length = max_packet, Max ESIT Payload = max_packet
        ptr::write_volatile(ep_context.add(4), (maximum_packet as u32) | ((maximum_packet as u32) << 16));
    }
    
    // Store interrupt ring
    {
        let mut rings = SLOT_RINGS.lock();
        if let Some(Some(slot_rings)) = rings.get_mut(slot_id as usize) {
            slot_rings.interrupt_in = Some(int_ring);
            slot_rings.interrupt_dci = dci;
        }
    }
    
    // Issue Configure Endpoint command
    let trb = Trb {
        parameter: input_physical,
        status: 0,
        control: (TRB_TYPE_CONFIGURE_EP << 10) | ((slot_id as u32) << 24),
    };
    
    submit_command(controller, trb);
    
    let success = if let Some((cc, _, _)) = wait_command_completion(controller) {
        if cc == 1 {
            crate::serial_println!("[xHCI] Configure Endpoint slot {} DCI {} → success", slot_id, dci);
            true
        } else {
            crate::serial_println!("[xHCI] Configure Endpoint failed: cc={}", cc);
            false
        }
    } else {
        false
    };
    
    crate::memory::frame::free_frame(input_physical);
    success
}

/// Configure bulk IN and OUT endpoints for a mass storage device
fn configure_bulk_endpoints(
    controller: &mut XhciController,
    slot_id: u8,
    port_number: u8,
    speed: u8,
    ep_in_address: u8,
    ep_out_address: u8,
    maximum_packet_in: u16,
    maximum_packet_out: u16,
) -> bool {
    let ep_in_number = ep_in_address & 0x0F;
    let dci_in = (ep_in_number * 2 + 1) as u8;  // IN endpoint DCI
    let ep_out_number = ep_out_address & 0x0F;
    let dci_out = (ep_out_number * 2) as u8;     // OUT endpoint DCI
    let maximum_dci = dci_in.maximum(dci_out);
    
    // Allocate transfer rings
    let bulk_in_ring = // Pattern matching — Rust's exhaustive branching construct.
match TransferRing::new() {
        Some(r) => r,
        None => return false,
    };
    let bulk_out_ring = // Pattern matching — Rust's exhaustive branching construct.
match TransferRing::new() {
        Some(r) => r,
        None => return false,
    };
    let in_ring_physical = bulk_in_ring.physical;
    let out_ring_physical = bulk_out_ring.physical;
    
    // Build input context for Configure Endpoint
    let input_physical = // Pattern matching — Rust's exhaustive branching construct.
match crate::memory::frame::allocator_frame_zeroed() {
        Some(p) => p,
        None => return false,
    };
    let input_virt = physical_to_virt(input_physical);
    let context_size = controller.context_size;
    
        // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe {
        let icc = input_virt as *mut u32;
        // Add Slot + both bulk endpoints
        ptr::write_volatile(icc.add(1), 1 | (1u32 << dci_in) | (1u32 << dci_out));
        
        // Slot Context: update Context Entries to include highest DCI
        let slot = (input_virt + context_size as u64) as *mut u32;
        let speed_field = (speed as u32) << 20;
        let context_entries = (maximum_dci as u32) << 27;
        ptr::write_volatile(slot, speed_field | context_entries);
        ptr::write_volatile(slot.add(1), (port_number as u32) << 16);
        
        // Bulk IN endpoint context (EP Type = 6 = Bulk IN)
        let ep_in_context = (input_virt + ((1 + dci_in as usize) * context_size) as u64) as *mut u32;
        let cerr = 3u32 << 1;
        let ep_type_bulk_in = 6u32 << 3;
        let mps_in = (maximum_packet_in as u32) << 16;
        ptr::write_volatile(ep_in_context.add(1), cerr | ep_type_bulk_in | mps_in);
        ptr::write_volatile(ep_in_context.add(2) as *mut u64, in_ring_physical | 1);
        ptr::write_volatile(ep_in_context.add(4), maximum_packet_in as u32);
        
        // Bulk OUT endpoint context (EP Type = 2 = Bulk OUT)
        let ep_out_context = (input_virt + ((1 + dci_out as usize) * context_size) as u64) as *mut u32;
        let ep_type_bulk_out = 2u32 << 3;
        let mps_out = (maximum_packet_out as u32) << 16;
        ptr::write_volatile(ep_out_context.add(1), cerr | ep_type_bulk_out | mps_out);
        ptr::write_volatile(ep_out_context.add(2) as *mut u64, out_ring_physical | 1);
        ptr::write_volatile(ep_out_context.add(4), maximum_packet_out as u32);
    }
    
    // Store bulk rings
    {
        let mut rings = SLOT_RINGS.lock();
        if let Some(Some(slot_rings)) = rings.get_mut(slot_id as usize) {
            slot_rings.bulk_in = Some(bulk_in_ring);
            slot_rings.bulk_in_dci = dci_in;
            slot_rings.bulk_out = Some(bulk_out_ring);
            slot_rings.bulk_out_dci = dci_out;
        }
    }
    
    // Issue Configure Endpoint command
    let trb = Trb {
        parameter: input_physical,
        status: 0,
        control: (TRB_TYPE_CONFIGURE_EP << 10) | ((slot_id as u32) << 24),
    };
    
    submit_command(controller, trb);
    
    let success = if let Some((cc, _, _)) = wait_command_completion(controller) {
        if cc == 1 {
            crate::serial_println!("[xHCI] Bulk endpoints configured: slot {} IN_DCI={} OUT_DCI={}",
                slot_id, dci_in, dci_out);
            true
        } else {
            crate::serial_println!("[xHCI] Configure bulk EPs failed: cc={}", cc);
            false
        }
    } else {
        false
    };
    
    crate::memory::frame::free_frame(input_physical);
    success
}

// ============================================================================
// HID Boot Protocol Support
// ============================================================================

/// Set HID Boot Protocol (SET_PROTOCOL request, protocol=0 for boot)
fn set_boot_protocol(controller: &mut XhciController, slot_id: u8, interface: u8) -> bool {
    control_transfer_nodata(
        controller, slot_id,
        USB_DIRECTORY_OUT | (1 << 5) | USB_RECIP_INTERFACE, // Class request to interface
        USB_REQUEST_SET_PROTOCOL,
        0, // Boot protocol
        interface as u16,
    )
}

/// Set HID Idle rate to 0 (only report on change)
fn set_idle(controller: &mut XhciController, slot_id: u8, interface: u8) -> bool {
    control_transfer_nodata(
        controller, slot_id,
        USB_DIRECTORY_OUT | (1 << 5) | USB_RECIP_INTERFACE,
        USB_REQUEST_SET_IDLE,
        0, // Idle rate = 0 (infinite)
        interface as u16,
    )
}

/// Queue a single Normal TRB on the interrupt IN endpoint for HID polling
fn queue_interrupt_in(controller: &XhciController, slot_id: u8, buffer_physical: u64, maximum_packet: u16) -> bool {
    let mut rings = SLOT_RINGS.lock();
    let slot_rings = // Pattern matching — Rust's exhaustive branching construct.
match rings.get_mut(slot_id as usize).and_then(|r| r.as_mut()) {
        Some(r) => r,
        None => return false,
    };
    
    let int_ring = // Pattern matching — Rust's exhaustive branching construct.
match slot_rings.interrupt_in.as_mut() {
        Some(r) => r,
        None => return false,
    };
    let dci = slot_rings.interrupt_dci;
    
    let trb = Trb {
        parameter: buffer_physical,
        status: maximum_packet as u32,
        control: (TRB_TYPE_NORMAL << 10) | TRB_IOC,
    };
    int_ring.enqueue_trb(trb);
    
    // Ring doorbell for this endpoint
    drop(rings);
        // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe {
        let db = (controller.doorbell_base + (slot_id as u64) * 4) as *mut u32;
        ptr::write_volatile(db, dci as u32);
    }
    
    true
}

// ============================================================================
// HID Report Processing — Keyboard & Mouse Bridge
// ============================================================================

/// HID Boot Protocol keyboard report (8 bytes):
///   [0] = modifier keys (Ctrl/Shift/Alt/GUI bitmask)
///   [1] = reserved
///   [2..7] = keycodes (up to 6 simultaneous)
fn process_keyboard_report(report: &[u8]) {
    if report.len() < 8 { return; }
    
    let _modifiers = report[0];
    // report[2..8] = pressed keycodes (HID usage IDs)
    
    for &keycode in &report[2..8] {
        if keycode == 0 { continue; }
        
        // Convert HID usage ID to ASCII
        let ascii = hid_keycode_to_ascii(keycode, report[0]);
        if ascii != 0 {
            crate::keyboard::push_key(ascii);
        }
    }
}

/// HID Boot Protocol mouse report (3-4 bytes):
///   [0] = buttons (bit0=left, bit1=right, bit2=middle)
///   [1] = X displacement (signed i8)
///   [2] = Y displacement (signed i8)
///   [3] = scroll wheel (signed i8, optional)
fn process_mouse_report(report: &[u8]) {
    if report.len() < 3 { return; }
    
    let buttons = report[0];
    let dx = report[1] as i8 as i32;
    let dy = report[2] as i8 as i32;
    let scroll = if report.len() >= 4 { report[3] as i8 } else { 0 };
    
    crate::mouse::inject_usb_mouse(
        dx, dy,
        buttons & 1 != 0,
        buttons & 2 != 0,
        buttons & 4 != 0,
        scroll,
    );
}

/// Convert HID keyboard usage ID to ASCII (with modifier support)
fn hid_keycode_to_ascii(keycode: u8, modifiers: u8) -> u8 {
    let shift = (modifiers & 0x22) != 0; // Left or Right Shift
    let _controller = (modifiers & 0x11) != 0;
    
        // Pattern matching — Rust's exhaustive branching construct.
match keycode {
        // Letters a-z (HID 0x04-0x1D)
        0x04..=0x1D => {
            let base = b'a' + (keycode - 0x04);
            if shift { base - 32 } else { base }
        }
        // Numbers 1-9, 0 (HID 0x1E-0x27)
        0x1E..=0x26 => {
            if shift {
                                // Pattern matching — Rust's exhaustive branching construct.
match keycode {
                    0x1E => b'!', 0x1F => b'@', 0x20 => b'#', 0x21 => b'$',
                    0x22 => b'%', 0x23 => b'^', 0x24 => b'&', 0x25 => b'*',
                    0x26 => b'(',
                    _ => 0,
                }
            } else {
                b'1' + (keycode - 0x1E)
            }
        }
        0x27 => if shift { b')' } else { b'0' },
        0x28 => b'\r',   // Enter
        0x29 => 0x1B,    // Escape
        0x2A => 0x08,    // Backspace
        0x2B => b'\t',   // Tab
        0x2C => b' ',    // Space
        0x2D => if shift { b'_' } else { b'-' },
        0x2E => if shift { b'+' } else { b'=' },
        0x2F => if shift { b'{' } else { b'[' },
        0x30 => if shift { b'}' } else { b']' },
        0x31 => if shift { b'|' } else { b'\\' },
        0x33 => if shift { b':' } else { b';' },
        0x34 => if shift { b'"' } else { b'\'' },
        0x35 => if shift { b'~' } else { b'`' },
        0x36 => if shift { b'<' } else { b',' },
        0x37 => if shift { b'>' } else { b'.' },
        0x38 => if shift { b'?' } else { b'/' },
        _ => 0,
    }
}

// ============================================================================
// Full Device Setup (called after enumerate_ports)
// ============================================================================

/// Set up all detected devices: Enable Slot → Address → Descriptors → HID/Mass Storage config
fn setup_devices() {
    let mut mouse_devices: Vec<(u8, u8, u8, u16, u16)> = Vec::new();
    
    {
        let mut controller = CONTROLLER.lock();
        let controller = // Pattern matching — Rust's exhaustive branching construct.
match controller.as_mut() {
            Some(c) => c,
            None => return,
        };
        
        let number_devices = controller.devices.len();
        if number_devices == 0 {
            return;
        }
        
        crate::serial_println!("[xHCI] Setting up {} devices...", number_devices);
        
        // Process each device: Enable Slot → Address → Get descriptors
        for i in 0..number_devices {
            let port = controller.devices[i].port;
            let speed = controller.devices[i].speed;
            
            // Enable Slot
            let slot_id = // Pattern matching — Rust's exhaustive branching construct.
match enable_slot(controller) {
                Some(id) => id,
                None => {
                    crate::serial_println!("[xHCI] Failed to enable slot for port {}", port);
                    continue;
                }
            };
            
            controller.devices[i].slot_id = slot_id;
            
            // Address Device
            if !address_device(controller, slot_id, port, speed) {
                crate::serial_println!("[xHCI] Failed to address device on port {}", port);
                continue;
            }
            
            // Get Device Descriptor
            let mut device = controller.devices[i].clone();
            if !get_device_descriptor(controller, slot_id, &mut device) {
                crate::serial_println!("[xHCI] Failed to get device descriptor for slot {}", slot_id);
                continue;
            }
            controller.devices[i] = device;
            
            // Get Configuration Descriptor + find HID and Mass Storage endpoints
            if let Some(all_interfaces) = get_config_descriptor(controller, slot_id) {
                let mut mouse_in: Option<(u8, u16)> = None;
                let mut mouse_out: Option<(u8, u16)> = None;
                
                for &(iface_class, iface_number, subclass, protocol, ep_address, maximum_packet, interval) in &all_interfaces {
                                        // Pattern matching — Rust's exhaustive branching construct.
match iface_class {
                        0x03 => {
                            // HID device
                            let _ = set_boot_protocol(controller, slot_id, iface_number);
                            let _ = set_idle(controller, slot_id, iface_number);
                            
                            configure_hid_endpoint(
                                controller, slot_id, port, speed,
                                ep_address, maximum_packet, interval,
                            );
                            
                            if controller.devices[i].class == 0 {
                                controller.devices[i].class = 0x03;
                                controller.devices[i].subclass = subclass;
                                controller.devices[i].protocol = protocol;
                            }
                            
                            crate::serial_println!("[xHCI] HID endpoint configured: slot {} EP {:#x} max_pkt {} interval {}",
                                slot_id, ep_address, maximum_packet, interval);
                        }
                        0x08 => {
                            // Mass storage: collect bulk endpoints
                            if ep_address & 0x80 != 0 {
                                mouse_in = Some((ep_address, maximum_packet));
                            } else {
                                mouse_out = Some((ep_address, maximum_packet));
                            }
                        }
                        _ => {}
                    }
                }
                
                // Configure mass storage bulk endpoints
                if let (Some((in_address, in_mps)), Some((out_address, out_mps))) = (mouse_in, mouse_out) {
                    if configure_bulk_endpoints(controller, slot_id, port, speed, in_address, out_address, in_mps, out_mps) {
                        mouse_devices.push((slot_id, in_address, out_address, in_mps, out_mps));
                        
                        if controller.devices[i].class == 0 {
                            controller.devices[i].class = 0x08;
                            controller.devices[i].subclass = 0x06;
                            controller.devices[i].protocol = 0x50;
                        }
                    }
                }
            }
        }
        
        crate::serial_println!("[xHCI] Device setup complete");
    }
    
    // Initialize mass storage devices outside CONTROLLER lock
    // (usb_storage bulk transfers need to lock CONTROLLER internally)
    for (slot_id, in_address, out_address, in_mps, out_mps) in mouse_devices {
        super::usb_storage::initialize_device(slot_id, in_address, out_address, in_mps, out_mps);
    }
}

// ============================================================================
// Bulk Transfer Support (for USB Mass Storage)
// ============================================================================

/// Send data on a bulk OUT endpoint. Returns true on success.
pub fn bulk_transfer_out(slot_id: u8, dci: u8, buffer_physical: u64, length: u32) -> bool {
    // Enqueue Normal TRB on the bulk OUT ring
    {
        let mut rings = SLOT_RINGS.lock();
        let slot_rings = // Pattern matching — Rust's exhaustive branching construct.
match rings.get_mut(slot_id as usize).and_then(|r| r.as_mut()) {
            Some(r) => r,
            None => return false,
        };
        
        let ring = // Pattern matching — Rust's exhaustive branching construct.
match slot_rings.bulk_out.as_mut() {
            Some(r) => r,
            None => return false,
        };
        
        let trb = Trb {
            parameter: buffer_physical,
            status: length,
            control: (TRB_TYPE_NORMAL << 10) | TRB_IOC,
        };
        ring.enqueue_trb(trb);
    }
    
    // Ring doorbell and wait for completion
    let mut controller = CONTROLLER.lock();
    let controller = // Pattern matching — Rust's exhaustive branching construct.
match controller.as_mut() {
        Some(c) => c,
        None => return false,
    };
    
        // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe {
        let db = (controller.doorbell_base + (slot_id as u64) * 4) as *mut u32;
        ptr::write_volatile(db, dci as u32);
    }
    
    if let Some((cc, _, _)) = wait_transfer_event(controller) {
        return cc == 1 || cc == 13; // Success or Short Packet
    }
    false
}

/// Receive data on a bulk IN endpoint. Returns number of bytes actually transferred.
pub fn bulk_transfer_in(slot_id: u8, dci: u8, buffer_physical: u64, length: u32) -> Option<u32> {
    // Enqueue Normal TRB on the bulk IN ring
    {
        let mut rings = SLOT_RINGS.lock();
        let slot_rings = // Pattern matching — Rust's exhaustive branching construct.
match rings.get_mut(slot_id as usize).and_then(|r| r.as_mut()) {
            Some(r) => r,
            None => return None,
        };
        
        let ring = // Pattern matching — Rust's exhaustive branching construct.
match slot_rings.bulk_in.as_mut() {
            Some(r) => r,
            None => return None,
        };
        
        let trb = Trb {
            parameter: buffer_physical,
            status: length,
            control: (TRB_TYPE_NORMAL << 10) | TRB_IOC,
        };
        ring.enqueue_trb(trb);
    }
    
    // Ring doorbell and wait for completion
    let mut controller = CONTROLLER.lock();
    let controller = // Pattern matching — Rust's exhaustive branching construct.
match controller.as_mut() {
        Some(c) => c,
        None => return None,
    };
    
        // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe {
        let db = (controller.doorbell_base + (slot_id as u64) * 4) as *mut u32;
        ptr::write_volatile(db, dci as u32);
    }
    
    if let Some((cc, residue, _)) = wait_transfer_event(controller) {
        if cc == 1 || cc == 13 {
            // Transfer event reports bytes NOT transferred (residue)
            return Some(length.saturating_sub(residue));
        }
    }
    None
}

/// Poll all HID devices once for input reports
pub fn poll_hid_devices() {
    // Collect HID device info under lock, then release
    let hid_devices: Vec<(u8, u16, u8)> = {
        let controller = CONTROLLER.lock();
                // Pattern matching — Rust's exhaustive branching construct.
match controller.as_ref() {
            Some(c) => c.devices.iter()
                .filter(|d| d.slot_id != 0 && d.class == 0x03)
                .map(|d| (d.slot_id, d.maximum_packet_size, d.protocol))
                .collect(),
            None => return,
        }
    };
    
    for &(slot_id, maximum_packet_size, protocol) in &hid_devices {
        let buffer_physical = // Pattern matching — Rust's exhaustive branching construct.
match crate::memory::frame::allocator_frame_zeroed() {
            Some(p) => p,
            None => continue,
        };
        let buffer_virt = physical_to_virt(buffer_physical);
        let maximum_packet = maximum_packet_size.maximum(8);
        
        // Queue interrupt IN transfer (requires lock)
        {
            let controller = CONTROLLER.lock();
            if let Some(controller) = controller.as_ref() {
                if !queue_interrupt_in(controller, slot_id, buffer_physical, maximum_packet) {
                    crate::memory::frame::free_frame(buffer_physical);
                    continue;
                }
            } else {
                crate::memory::frame::free_frame(buffer_physical);
                continue;
            }
        }
        
        // Poll for completion (with short timeout for non-blocking behavior)
        {
            let mut ctrl2 = CONTROLLER.lock();
            if let Some(controller) = ctrl2.as_mut() {
                for _ in 0..50_000u32 {
                    let index = controller.event_dequeue;
                    let trb = controller.event_ring[index];
                    let phase = (trb.control & TRB_CYCLE) != 0;
                    if phase == controller.event_cycle {
                        controller.event_dequeue += 1;
                        if controller.event_dequeue >= 256 {
                            controller.event_dequeue = 0;
                            controller.event_cycle = !controller.event_cycle;
                        }
                        let erdp = controller.event_ring_physical + (controller.event_dequeue as u64 * 16);
                        let intr = (controller.runtime_base + 0x20) as *mut XhciIntrRegs;
                                                // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe { (*intr).erdp = erdp | (1 << 3); }
                        
                        let cc = ((trb.status >> 24) & 0xFF) as u8;
                        if cc == 1 || cc == 13 { // Success or Short Packet
                            let report = // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe {
                                core::slice::from_raw_parts(buffer_virt as *// Compile-time constant — evaluated at compilation, zero runtime cost.
const u8, maximum_packet as usize)
                            };
                            
                                                        // Pattern matching — Rust's exhaustive branching construct.
match protocol {
                                1 => process_keyboard_report(report),
                                2 => process_mouse_report(report),
                                _ => {}
                            }
                        }
                        break;
                    }
                    core::hint::spin_loop();
                }
            }
        }
        
        crate::memory::frame::free_frame(buffer_physical);
        return; // Only poll one device per call to avoid blocking
    }
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
