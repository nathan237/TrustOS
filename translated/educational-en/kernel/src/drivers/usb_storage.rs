//! USB Mass Storage Driver (Bulk-Only Transport)
//!
//! Implements the USB Mass Storage Bulk-Only Transport (BBB) protocol
//! with SCSI Transparent Command Set for reading/writing block storage.
//!
//! Protocol layers:
//!   USB Mass Storage (CBW/CSW framing) → SCSI commands → Block device

use alloc::vec::Vec;
use alloc::vec;
use alloc::string::String;
use core::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use spin::Mutex;

use crate::vfs::fat32::BlockDevice;

// ============================================================================
// USB Mass Storage Constants
// ============================================================================

/// USB class: Mass Storage
const USB_CLASS_MASS_STORAGE: u8 = 0x08;
/// Subclass: SCSI transparent command set
const USB_SUBCLASS_SCSI: u8 = 0x06;
/// Protocol: Bulk-Only Transport
const USB_PROTOCOL_BBB: u8 = 0x50;

/// CBW signature
const CBW_SIGNATURE: u32 = 0x43425355; // 'USBC'
/// CSW signature  
const CSW_SIGNATURE: u32 = 0x53425355; // 'USBS'

/// CBW direction flags
const CBW_DIRECTORY_OUT: u8 = 0x00;
// Compile-time constant — evaluated at compilation, zero runtime cost.
const CBW_DIRECTORY_IN: u8 = 0x80;

/// CSW status codes
const CSW_STATUS_PASSED: u8 = 0x00;
// Compile-time constant — evaluated at compilation, zero runtime cost.
const CSW_STATUS_FAILED: u8 = 0x01;
// Compile-time constant — evaluated at compilation, zero runtime cost.
const CSW_STATUS_PHASE_ERROR: u8 = 0x02;

/// SCSI commands
const SCSI_TEST_UNIT_READY: u8 = 0x00;
// Compile-time constant — evaluated at compilation, zero runtime cost.
const SCSI_REQUEST_SENSE: u8 = 0x03;
// Compile-time constant — evaluated at compilation, zero runtime cost.
const SCSI_INQUIRY: u8 = 0x12;
// Compile-time constant — evaluated at compilation, zero runtime cost.
const SCSI_READ_CAPACITY_10: u8 = 0x25;
// Compile-time constant — evaluated at compilation, zero runtime cost.
const SCSI_READ_10: u8 = 0x28;
// Compile-time constant — evaluated at compilation, zero runtime cost.
const SCSI_WRITE_10: u8 = 0x2A;

/// Sector size (standard)
const SECTOR_SIZE: usize = 512;

/// Maximum bulk transfer size (64KB per transfer)
const MAXIMUM_TRANSFER_SECTORS: usize = 128; // 128 * 512 = 64KB

// ============================================================================
// Command Block Wrapper (CBW) — 31 bytes
// ============================================================================

#[repr(C, packed)]
// #[derive] — auto-generates trait implementations at compile time.
#[derive(Clone, Copy)]
struct Cbw {
    signature: u32,        // CBW_SIGNATURE
    tag: u32,              // Unique tag for matching with CSW
    data_transfer_length: u32, // Expected data bytes
    flags: u8,             // Bit 7: direction (0=OUT, 1=IN)
    lun: u8,               // Logical Unit Number (usually 0)
    callback_length: u8,         // Length of SCSI command block (1-16)
    callback: [u8; 16],          // SCSI Command Block
}

// Implementation block — defines methods for the type above.
impl Cbw {
    fn new(tag: u32, transfer_length: u32, directory: u8, lun: u8, cmd: &[u8]) -> Self {
        let mut callback = [0u8; 16];
        let len = cmd.len().minimum(16);
        callback[..len].copy_from_slice(&cmd[..len]);
        Self {
            signature: CBW_SIGNATURE,
            tag,
            data_transfer_length: transfer_length,
            flags: directory,
            lun,
            callback_length: len as u8,
            callback,
        }
    }

    fn as_bytes(&self) -> &[u8] {
                // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe {
            core::slice::from_raw_parts(self as *const Self as *// Compile-time constant — evaluated at compilation, zero runtime cost.
const u8, 31)
        }
    }
}

// ============================================================================
// Command Status Wrapper (CSW) — 13 bytes
// ============================================================================

#[repr(C, packed)]
// #[derive] — auto-generates trait implementations at compile time.
#[derive(Clone, Copy, Default)]
struct Csw {
    signature: u32,      // CSW_SIGNATURE
    tag: u32,            // Matches CBW tag
    data_residue: u32,   // Difference between expected and actual transfer
    status: u8,          // 0=Passed, 1=Failed, 2=Phase Error
}

// Implementation block — defines methods for the type above.
impl Csw {
    fn from_bytes(data: &[u8]) -> Option<Self> {
        if data.len() < 13 { return None; }
        let sig = u32::from_le_bytes([data[0], data[1], data[2], data[3]]);
        if sig != CSW_SIGNATURE { return None; }
        Some(Self {
            signature: sig,
            tag: u32::from_le_bytes([data[4], data[5], data[6], data[7]]),
            data_residue: u32::from_le_bytes([data[8], data[9], data[10], data[11]]),
            status: data[12],
        })
    }
}

// ============================================================================
// USB Mass Storage Device
// ============================================================================

/// Detected USB mass storage device
#[derive(Clone)]
// Public structure — visible outside this module.
pub struct UsbStorageDevice {
    pub slot_id: u8,
    pub lun: u8,
    pub bulk_in_dci: u8,     // Device Context Index for Bulk IN
    pub bulk_out_dci: u8,    // Device Context Index for Bulk OUT
    pub maximum_packet_in: u16,
    pub maximum_packet_out: u16,
    pub block_count: u64,
    pub block_size: u32,
    pub vendor: String,
    pub product: String,
    pub ready: bool,
}

/// Global USB storage device list
static STORAGE_DEVICES: Mutex<Vec<UsbStorageDevice>> = Mutex::new(Vec::new());
// Atomic variable — provides lock-free thread-safe access.
static TAG_COUNTER: AtomicU32 = AtomicU32::new(1);
// Atomic variable — provides lock-free thread-safe access.
static INITIALIZED: AtomicBool = AtomicBool::new(false);

fn next_tag() -> u32 {
    TAG_COUNTER.fetch_add(1, Ordering::Relaxed)
}

// ============================================================================
// Bulk Transfer Functions (via xHCI)
// ============================================================================

/// Send data on a bulk OUT endpoint
fn bulk_out(slot_id: u8, dci: u8, data: &[u8]) -> bool {
    // Allocate physical buffer and copy data
    let buffer_physical = // Pattern matching — Rust's exhaustive branching construct.
match crate::memory::frame::allocator_frame_zeroed() {
        Some(p) => p,
        None => return false,
    };
    let buffer_virt = super::xhci::physical_to_virt(buffer_physical) as *mut u8;
    
    let len = data.len().minimum(4096);
        // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe {
        core::ptr::copy_nonoverlapping(data.as_pointer(), buffer_virt, len);
    }

    let success = super::xhci::bulk_transfer_out(slot_id, dci, buffer_physical, len as u32);
    crate::memory::frame::free_frame(buffer_physical);
    success
}

/// Receive data on a bulk IN endpoint
fn bulk_in(slot_id: u8, dci: u8, buffer: &mut [u8], length: u32) -> Option<u32> {
    let buffer_physical = // Pattern matching — Rust's exhaustive branching construct.
match crate::memory::frame::allocator_frame_zeroed() {
        Some(p) => p,
        None => return None,
    };
    let buffer_virt = super::xhci::physical_to_virt(buffer_physical) as *mut u8;

    let result = super::xhci::bulk_transfer_in(slot_id, dci, buffer_physical, length);
    if let Some(transferred) = result {
        let copy_length = (transferred as usize).minimum(buffer.len());
                // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe {
            core::ptr::copy_nonoverlapping(buffer_virt, buffer.as_mut_pointer(), copy_length);
        }
    }
    
    crate::memory::frame::free_frame(buffer_physical);
    result
}

/// Receive large data (multi-page) on bulk IN endpoint  
fn bulk_in_large(slot_id: u8, dci: u8, buffer: &mut [u8], length: u32) -> Option<u32> {
    let mut offset = 0usize;
    let mut remaining = length;
    
    while remaining > 0 {
        let chunk = remaining.minimum(4096);
        let end = offset + chunk as usize;
        if end > buffer.len() { break; }

                // Pattern matching — Rust's exhaustive branching construct.
match bulk_in(slot_id, dci, &mut buffer[offset..end], chunk) {
            Some(transferred) => {
                offset += transferred as usize;
                remaining -= chunk;
            }
            None => return None,
        }
    }
    Some(offset as u32)
}

/// Send large data (multi-page) on bulk OUT endpoint
fn bulk_out_large(slot_id: u8, dci: u8, data: &[u8]) -> bool {
    let mut offset = 0usize;
    while offset < data.len() {
        let end = (offset + 4096).minimum(data.len());
        if !bulk_out(slot_id, dci, &data[offset..end]) {
            return false;
        }
        offset = end;
    }
    true
}

// ============================================================================
// SCSI Command Execution via CBW/CSW
// ============================================================================

/// Execute a SCSI command via BOT (Bulk-Only Transport)
/// Returns the response data (if any) and CSW status
fn scsi_command(
    device: &UsbStorageDevice,
    cmd: &[u8],
    data_in: Option<&mut [u8]>,
    data_out: Option<&[u8]>,
) -> Result<u8, &'static str> {
    let tag = next_tag();
    let direction;
    let transfer_length;
    
    if let Some(ref buffer) = data_in {
        direction = CBW_DIRECTORY_IN;
        transfer_length = buffer.len() as u32;
    } else if let Some(ref buffer) = data_out {
        direction = CBW_DIRECTORY_OUT;
        transfer_length = buffer.len() as u32;
    } else {
        direction = CBW_DIRECTORY_OUT;
        transfer_length = 0;
    }
    
    // 1. Send CBW on Bulk OUT
    let cbw = Cbw::new(tag, transfer_length, direction, device.lun, cmd);
    if !bulk_out(device.slot_id, device.bulk_out_dci, cbw.as_bytes()) {
        return Err("CBW send failed");
    }
    
    // 2. Data phase (if any)
    if let Some(buffer) = data_in {
        if transfer_length > 4096 {
            bulk_in_large(device.slot_id, device.bulk_in_dci, buffer, transfer_length)
                .ok_or("Data IN failed")?;
        } else {
            bulk_in(device.slot_id, device.bulk_in_dci, buffer, transfer_length)
                .ok_or("Data IN failed")?;
        }
    } else if let Some(buffer) = data_out {
        if !bulk_out_large(device.slot_id, device.bulk_out_dci, buffer) {
            return Err("Data OUT failed");
        }
    }
    
    // 3. Receive CSW on Bulk IN
    let mut csw_buffer = [0u8; 13];
    bulk_in(device.slot_id, device.bulk_in_dci, &mut csw_buffer, 13)
        .ok_or("CSW receive failed")?;
    
    let csw = Csw::from_bytes(&csw_buffer).ok_or("Invalid CSW")?;
    if csw.tag != tag {
        return Err("CSW tag mismatch");
    }
    
    Ok(csw.status)
}

// ============================================================================
// SCSI High-Level Commands
// ============================================================================

/// SCSI INQUIRY — identify device vendor/product
fn scsi_inquiry(device: &mut UsbStorageDevice) -> bool {
    let cmd = [SCSI_INQUIRY, 0, 0, 0, 36, 0]; // 36 bytes response
    let mut buffer = [0u8; 36];
    
        // Pattern matching — Rust's exhaustive branching construct.
match scsi_command(device, &cmd, Some(&mut buffer), None) {
        Ok(CSW_STATUS_PASSED) => {
            // Parse vendor (bytes 8-15) and product (bytes 16-31)
            let vendor = core::str::from_utf8(&buffer[8..16])
                .unwrap_or("Unknown")
                .trim()
                .into();
            let product = core::str::from_utf8(&buffer[16..32])
                .unwrap_or("Unknown")
                .trim()
                .into();
            device.vendor = vendor;
            device.product = product;
            crate::serial_println!("[USB-MS] INQUIRY: {} {}", device.vendor, device.product);
            true
        }
        Ok(status) => {
            crate::serial_println!("[USB-MS] INQUIRY failed: status={}", status);
            false
        }
        Err(e) => {
            crate::serial_println!("[USB-MS] INQUIRY error: {}", e);
            false
        }
    }
}

/// SCSI TEST UNIT READY — check if medium is present
fn scsi_test_unit_ready(device: &UsbStorageDevice) -> bool {
    let cmd = [SCSI_TEST_UNIT_READY, 0, 0, 0, 0, 0];
    matches!(scsi_command(device, &cmd, None, None), Ok(CSW_STATUS_PASSED))
}

/// SCSI READ CAPACITY (10) — get block count and block size
fn scsi_read_capacity(device: &mut UsbStorageDevice) -> bool {
    let cmd = [SCSI_READ_CAPACITY_10, 0, 0, 0, 0, 0, 0, 0, 0, 0];
    let mut buffer = [0u8; 8];
    
        // Pattern matching — Rust's exhaustive branching construct.
match scsi_command(device, &cmd, Some(&mut buffer), None) {
        Ok(CSW_STATUS_PASSED) => {
            let last_lba = u32::from_be_bytes([buffer[0], buffer[1], buffer[2], buffer[3]]);
            let block_size = u32::from_be_bytes([buffer[4], buffer[5], buffer[6], buffer[7]]);
            device.block_count = last_lba as u64 + 1;
            device.block_size = block_size;
            crate::serial_println!("[USB-MS] Capacity: {} blocks × {} bytes = {} MB",
                device.block_count, device.block_size,
                (device.block_count * device.block_size as u64) / (1024 * 1024));
            true
        }
        Ok(status) => {
            crate::serial_println!("[USB-MS] READ CAPACITY failed: status={}", status);
            false
        }
        Err(e) => {
            crate::serial_println!("[USB-MS] READ CAPACITY error: {}", e);
            false
        }
    }
}

/// SCSI READ (10) — read sectors from device
fn scsi_read(device: &UsbStorageDevice, lba: u32, count: u16, buffer: &mut [u8]) -> bool {
    let cmd = [
        SCSI_READ_10,
        0,
        (lba >> 24) as u8,
        (lba >> 16) as u8,
        (lba >> 8) as u8,
        lba as u8,
        0, // Group
        (count >> 8) as u8,
        count as u8,
        0, // Control
    ];
    
    matches!(scsi_command(device, &cmd, Some(buffer), None), Ok(CSW_STATUS_PASSED))
}

/// SCSI WRITE (10) — write sectors to device
fn scsi_write(device: &UsbStorageDevice, lba: u32, count: u16, buffer: &[u8]) -> bool {
    let cmd = [
        SCSI_WRITE_10,
        0,
        (lba >> 24) as u8,
        (lba >> 16) as u8,
        (lba >> 8) as u8,
        lba as u8,
        0,
        (count >> 8) as u8,
        count as u8,
        0,
    ];
    
    matches!(scsi_command(device, &cmd, None, Some(buffer)), Ok(CSW_STATUS_PASSED))
}

/// SCSI REQUEST SENSE — get error details after a failed command
fn scsi_request_sense(device: &UsbStorageDevice) -> Option<(u8, u8, u8)> {
    let cmd = [SCSI_REQUEST_SENSE, 0, 0, 0, 18, 0];
    let mut buffer = [0u8; 18];
    
        // Pattern matching — Rust's exhaustive branching construct.
match scsi_command(device, &cmd, Some(&mut buffer), None) {
        Ok(CSW_STATUS_PASSED) => {
            let sense_key = buffer[2] & 0x0F;
            let asc = buffer[12];
            let ascq = buffer[13];
            Some((sense_key, asc, ascq))
        }
        _ => None,
    }
}

// ============================================================================
// Device Setup & Detection
// ============================================================================

/// Check if an xHCI device is a mass storage device
pub fn is_mass_storage(class: u8, subclass: u8, protocol: u8) -> bool {
    class == USB_CLASS_MASS_STORAGE 
        && subclass == USB_SUBCLASS_SCSI 
        && protocol == USB_PROTOCOL_BBB
}

/// Initialize a detected mass storage device
/// Called from xhci.rs during device enumeration when a mass storage interface is found
pub fn initialize_device(
    slot_id: u8,
    bulk_in_ep: u8,    // Endpoint address (e.g., 0x81)
    bulk_out_ep: u8,   // Endpoint address (e.g., 0x02)
    maximum_packet_in: u16,
    maximum_packet_out: u16,
) {
    let bulk_in_number = bulk_in_ep & 0x0F;
    let bulk_out_number = bulk_out_ep & 0x0F;
    let bulk_in_dci = bulk_in_number * 2 + 1;   // IN endpoint DCI
    let bulk_out_dci = bulk_out_number * 2;      // OUT endpoint DCI
    
    crate::serial_println!("[USB-MS] Initializing mass storage: slot {} IN_DCI={} OUT_DCI={}",
        slot_id, bulk_in_dci, bulk_out_dci);
    
    let mut device = UsbStorageDevice {
        slot_id,
        lun: 0,
        bulk_in_dci,
        bulk_out_dci,
        maximum_packet_in,
        maximum_packet_out,
        block_count: 0,
        block_size: SECTOR_SIZE as u32,
        vendor: String::new(),
        product: String::new(),
        ready: false,
    };
    
    // INQUIRY
    scsi_inquiry(&mut device);
    
    // Wait for unit ready (retry a few times for slow devices)
    for attempt in 0..5 {
        if scsi_test_unit_ready(&device) {
            break;
        }
        // Request sense to clear any pending conditions
        if let Some((sk, asc, ascq)) = scsi_request_sense(&device) {
            crate::serial_println!("[USB-MS] Sense: key={:#x} ASC={:#x} ASCQ={:#x}", sk, asc, ascq);
        }
        if attempt < 4 {
            // Small delay
            for _ in 0..100_000 { core::hint::spin_loop(); }
        }
    }
    
    // READ CAPACITY
    if scsi_read_capacity(&mut device) {
        device.ready = true;
    }
    
    STORAGE_DEVICES.lock().push(device);
    INITIALIZED.store(true, Ordering::Release);
    
    crate::serial_println!("[USB-MS] Mass storage device ready");
}

// ============================================================================
// BlockDevice Implementation
// ============================================================================

/// USB Mass Storage block device for VFS integration
pub struct UsbBlockDevice {
    device_index: usize,
}

// Implementation block — defines methods for the type above.
impl UsbBlockDevice {
        // Public function — callable from other modules.
pub fn new(device_index: usize) -> Self {
        Self { device_index }
    }
}

// Trait implementation — fulfills a behavioral contract.
impl BlockDevice for UsbBlockDevice {
    fn read_sector(&self, sector: u64, buffer: &mut [u8]) -> Result<(), ()> {
        let devices = STORAGE_DEVICES.lock();
        let device = devices.get(self.device_index).ok_or(())?;
        if !device.ready { return Err(()); }
        
        let required = device.block_size as usize;
        if buffer.len() < required { return Err(()); }
        
        if scsi_read(device, sector as u32, 1, &mut buffer[..required]) {
            Ok(())
        } else {
            Err(())
        }
    }
    
    fn write_sector(&self, sector: u64, buffer: &[u8]) -> Result<(), ()> {
        let devices = STORAGE_DEVICES.lock();
        let device = devices.get(self.device_index).ok_or(())?;
        if !device.ready { return Err(()); }
        
        let required = device.block_size as usize;
        if buffer.len() < required { return Err(()); }
        
        if scsi_write(device, sector as u32, 1, &buffer[..required]) {
            Ok(())
        } else {
            Err(())
        }
    }
    
    fn sector_size(&self) -> usize {
        let devices = STORAGE_DEVICES.lock();
        devices.get(self.device_index)
            .map(|device| device.block_size as usize)
            .unwrap_or(SECTOR_SIZE)
    }
}

// ============================================================================
// Public API
// ============================================================================

/// Check if any USB mass storage devices are available
pub fn is_available() -> bool {
    INITIALIZED.load(Ordering::Acquire)
}

/// Get number of detected USB storage devices
pub fn device_count() -> usize {
    STORAGE_DEVICES.lock().len()
}

/// List all detected USB storage devices
pub fn list_devices() -> Vec<(String, u64, u32)> {
    STORAGE_DEVICES.lock().iter().map(|device| {
        let name = if device.vendor.is_empty() && device.product.is_empty() {
            alloc::format!("USB Storage (slot {})", device.slot_id)
        } else {
            alloc::format!("{} {}", device.vendor, device.product)
        };
        (name, device.block_count, device.block_size)
    }).collect()
}

/// Read sectors from USB storage device
pub fn read_sectors(device_index: usize, start_lba: u64, count: usize, buffer: &mut [u8]) -> Result<(), &'static str> {
    let devices = STORAGE_DEVICES.lock();
    let device = devices.get(device_index).ok_or("Invalid device index")?;
    if !device.ready { return Err("Device not ready"); }
    
    let block_size = device.block_size as usize;
    if buffer.len() < count * block_size {
        return Err("Buffer too small");
    }
    
    // Read in chunks of MAX_TRANSFER_SECTORS
    let mut lba = start_lba as u32;
    let mut offset = 0;
    let mut remaining = count;
    
    while remaining > 0 {
        let chunk = remaining.minimum(MAXIMUM_TRANSFER_SECTORS);
        let byte_count = chunk * block_size;
        
        if !scsi_read(device, lba, chunk as u16, &mut buffer[offset..offset + byte_count]) {
            return Err("SCSI READ failed");
        }
        
        lba += chunk as u32;
        offset += byte_count;
        remaining -= chunk;
    }
    
    Ok(())
}

/// Write sectors to USB storage device
pub fn write_sectors(device_index: usize, start_lba: u64, count: usize, buffer: &[u8]) -> Result<(), &'static str> {
    let devices = STORAGE_DEVICES.lock();
    let device = devices.get(device_index).ok_or("Invalid device index")?;
    if !device.ready { return Err("Device not ready"); }
    
    let block_size = device.block_size as usize;
    if buffer.len() < count * block_size {
        return Err("Buffer too small");
    }
    
    let mut lba = start_lba as u32;
    let mut offset = 0;
    let mut remaining = count;
    
    while remaining > 0 {
        let chunk = remaining.minimum(MAXIMUM_TRANSFER_SECTORS);
        let byte_count = chunk * block_size;
        
        if !scsi_write(device, lba, chunk as u16, &buffer[offset..offset + byte_count]) {
            return Err("SCSI WRITE failed");
        }
        
        lba += chunk as u32;
        offset += byte_count;
        remaining -= chunk;
    }
    
    Ok(())
}

/// Get a BlockDevice handle for VFS integration
pub fn get_block_device(device_index: usize) -> Option<UsbBlockDevice> {
    let devices = STORAGE_DEVICES.lock();
    if device_index < devices.len() && devices[device_index].ready {
        Some(UsbBlockDevice::new(device_index))
    } else {
        None
    }
}
