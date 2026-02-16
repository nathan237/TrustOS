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
const CBW_DIR_OUT: u8 = 0x00;
const CBW_DIR_IN: u8 = 0x80;

/// CSW status codes
const CSW_STATUS_PASSED: u8 = 0x00;
const CSW_STATUS_FAILED: u8 = 0x01;
const CSW_STATUS_PHASE_ERROR: u8 = 0x02;

/// SCSI commands
const SCSI_TEST_UNIT_READY: u8 = 0x00;
const SCSI_REQUEST_SENSE: u8 = 0x03;
const SCSI_INQUIRY: u8 = 0x12;
const SCSI_READ_CAPACITY_10: u8 = 0x25;
const SCSI_READ_10: u8 = 0x28;
const SCSI_WRITE_10: u8 = 0x2A;

/// Sector size (standard)
const SECTOR_SIZE: usize = 512;

/// Maximum bulk transfer size (64KB per transfer)
const MAX_TRANSFER_SECTORS: usize = 128; // 128 * 512 = 64KB

// ============================================================================
// Command Block Wrapper (CBW) — 31 bytes
// ============================================================================

#[repr(C, packed)]
#[derive(Clone, Copy)]
struct Cbw {
    signature: u32,        // CBW_SIGNATURE
    tag: u32,              // Unique tag for matching with CSW
    data_transfer_length: u32, // Expected data bytes
    flags: u8,             // Bit 7: direction (0=OUT, 1=IN)
    lun: u8,               // Logical Unit Number (usually 0)
    cb_length: u8,         // Length of SCSI command block (1-16)
    cb: [u8; 16],          // SCSI Command Block
}

impl Cbw {
    fn new(tag: u32, transfer_len: u32, dir: u8, lun: u8, cmd: &[u8]) -> Self {
        let mut cb = [0u8; 16];
        let len = cmd.len().min(16);
        cb[..len].copy_from_slice(&cmd[..len]);
        Self {
            signature: CBW_SIGNATURE,
            tag,
            data_transfer_length: transfer_len,
            flags: dir,
            lun,
            cb_length: len as u8,
            cb,
        }
    }

    fn as_bytes(&self) -> &[u8] {
        unsafe {
            core::slice::from_raw_parts(self as *const Self as *const u8, 31)
        }
    }
}

// ============================================================================
// Command Status Wrapper (CSW) — 13 bytes
// ============================================================================

#[repr(C, packed)]
#[derive(Clone, Copy, Default)]
struct Csw {
    signature: u32,      // CSW_SIGNATURE
    tag: u32,            // Matches CBW tag
    data_residue: u32,   // Difference between expected and actual transfer
    status: u8,          // 0=Passed, 1=Failed, 2=Phase Error
}

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
pub struct UsbStorageDevice {
    pub slot_id: u8,
    pub lun: u8,
    pub bulk_in_dci: u8,     // Device Context Index for Bulk IN
    pub bulk_out_dci: u8,    // Device Context Index for Bulk OUT
    pub max_packet_in: u16,
    pub max_packet_out: u16,
    pub block_count: u64,
    pub block_size: u32,
    pub vendor: String,
    pub product: String,
    pub ready: bool,
}

/// Global USB storage device list
static STORAGE_DEVICES: Mutex<Vec<UsbStorageDevice>> = Mutex::new(Vec::new());
static TAG_COUNTER: AtomicU32 = AtomicU32::new(1);
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
    let buf_phys = match crate::memory::frame::alloc_frame_zeroed() {
        Some(p) => p,
        None => return false,
    };
    let buf_virt = super::xhci::phys_to_virt(buf_phys) as *mut u8;
    
    let len = data.len().min(4096);
    unsafe {
        core::ptr::copy_nonoverlapping(data.as_ptr(), buf_virt, len);
    }

    let success = super::xhci::bulk_transfer_out(slot_id, dci, buf_phys, len as u32);
    crate::memory::frame::free_frame(buf_phys);
    success
}

/// Receive data on a bulk IN endpoint
fn bulk_in(slot_id: u8, dci: u8, buffer: &mut [u8], length: u32) -> Option<u32> {
    let buf_phys = match crate::memory::frame::alloc_frame_zeroed() {
        Some(p) => p,
        None => return None,
    };
    let buf_virt = super::xhci::phys_to_virt(buf_phys) as *mut u8;

    let result = super::xhci::bulk_transfer_in(slot_id, dci, buf_phys, length);
    if let Some(transferred) = result {
        let copy_len = (transferred as usize).min(buffer.len());
        unsafe {
            core::ptr::copy_nonoverlapping(buf_virt, buffer.as_mut_ptr(), copy_len);
        }
    }
    
    crate::memory::frame::free_frame(buf_phys);
    result
}

/// Receive large data (multi-page) on bulk IN endpoint  
fn bulk_in_large(slot_id: u8, dci: u8, buffer: &mut [u8], length: u32) -> Option<u32> {
    let mut offset = 0usize;
    let mut remaining = length;
    
    while remaining > 0 {
        let chunk = remaining.min(4096);
        let end = offset + chunk as usize;
        if end > buffer.len() { break; }

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
        let end = (offset + 4096).min(data.len());
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
    dev: &UsbStorageDevice,
    cmd: &[u8],
    data_in: Option<&mut [u8]>,
    data_out: Option<&[u8]>,
) -> Result<u8, &'static str> {
    let tag = next_tag();
    let direction;
    let transfer_len;
    
    if let Some(ref buf) = data_in {
        direction = CBW_DIR_IN;
        transfer_len = buf.len() as u32;
    } else if let Some(ref buf) = data_out {
        direction = CBW_DIR_OUT;
        transfer_len = buf.len() as u32;
    } else {
        direction = CBW_DIR_OUT;
        transfer_len = 0;
    }
    
    // 1. Send CBW on Bulk OUT
    let cbw = Cbw::new(tag, transfer_len, direction, dev.lun, cmd);
    if !bulk_out(dev.slot_id, dev.bulk_out_dci, cbw.as_bytes()) {
        return Err("CBW send failed");
    }
    
    // 2. Data phase (if any)
    if let Some(buf) = data_in {
        if transfer_len > 4096 {
            bulk_in_large(dev.slot_id, dev.bulk_in_dci, buf, transfer_len)
                .ok_or("Data IN failed")?;
        } else {
            bulk_in(dev.slot_id, dev.bulk_in_dci, buf, transfer_len)
                .ok_or("Data IN failed")?;
        }
    } else if let Some(buf) = data_out {
        if !bulk_out_large(dev.slot_id, dev.bulk_out_dci, buf) {
            return Err("Data OUT failed");
        }
    }
    
    // 3. Receive CSW on Bulk IN
    let mut csw_buf = [0u8; 13];
    bulk_in(dev.slot_id, dev.bulk_in_dci, &mut csw_buf, 13)
        .ok_or("CSW receive failed")?;
    
    let csw = Csw::from_bytes(&csw_buf).ok_or("Invalid CSW")?;
    if csw.tag != tag {
        return Err("CSW tag mismatch");
    }
    
    Ok(csw.status)
}

// ============================================================================
// SCSI High-Level Commands
// ============================================================================

/// SCSI INQUIRY — identify device vendor/product
fn scsi_inquiry(dev: &mut UsbStorageDevice) -> bool {
    let cmd = [SCSI_INQUIRY, 0, 0, 0, 36, 0]; // 36 bytes response
    let mut buf = [0u8; 36];
    
    match scsi_command(dev, &cmd, Some(&mut buf), None) {
        Ok(CSW_STATUS_PASSED) => {
            // Parse vendor (bytes 8-15) and product (bytes 16-31)
            let vendor = core::str::from_utf8(&buf[8..16])
                .unwrap_or("Unknown")
                .trim()
                .into();
            let product = core::str::from_utf8(&buf[16..32])
                .unwrap_or("Unknown")
                .trim()
                .into();
            dev.vendor = vendor;
            dev.product = product;
            crate::serial_println!("[USB-MS] INQUIRY: {} {}", dev.vendor, dev.product);
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
fn scsi_test_unit_ready(dev: &UsbStorageDevice) -> bool {
    let cmd = [SCSI_TEST_UNIT_READY, 0, 0, 0, 0, 0];
    matches!(scsi_command(dev, &cmd, None, None), Ok(CSW_STATUS_PASSED))
}

/// SCSI READ CAPACITY (10) — get block count and block size
fn scsi_read_capacity(dev: &mut UsbStorageDevice) -> bool {
    let cmd = [SCSI_READ_CAPACITY_10, 0, 0, 0, 0, 0, 0, 0, 0, 0];
    let mut buf = [0u8; 8];
    
    match scsi_command(dev, &cmd, Some(&mut buf), None) {
        Ok(CSW_STATUS_PASSED) => {
            let last_lba = u32::from_be_bytes([buf[0], buf[1], buf[2], buf[3]]);
            let block_size = u32::from_be_bytes([buf[4], buf[5], buf[6], buf[7]]);
            dev.block_count = last_lba as u64 + 1;
            dev.block_size = block_size;
            crate::serial_println!("[USB-MS] Capacity: {} blocks × {} bytes = {} MB",
                dev.block_count, dev.block_size,
                (dev.block_count * dev.block_size as u64) / (1024 * 1024));
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
fn scsi_read(dev: &UsbStorageDevice, lba: u32, count: u16, buffer: &mut [u8]) -> bool {
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
    
    matches!(scsi_command(dev, &cmd, Some(buffer), None), Ok(CSW_STATUS_PASSED))
}

/// SCSI WRITE (10) — write sectors to device
fn scsi_write(dev: &UsbStorageDevice, lba: u32, count: u16, buffer: &[u8]) -> bool {
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
    
    matches!(scsi_command(dev, &cmd, None, Some(buffer)), Ok(CSW_STATUS_PASSED))
}

/// SCSI REQUEST SENSE — get error details after a failed command
fn scsi_request_sense(dev: &UsbStorageDevice) -> Option<(u8, u8, u8)> {
    let cmd = [SCSI_REQUEST_SENSE, 0, 0, 0, 18, 0];
    let mut buf = [0u8; 18];
    
    match scsi_command(dev, &cmd, Some(&mut buf), None) {
        Ok(CSW_STATUS_PASSED) => {
            let sense_key = buf[2] & 0x0F;
            let asc = buf[12];
            let ascq = buf[13];
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
pub fn init_device(
    slot_id: u8,
    bulk_in_ep: u8,    // Endpoint address (e.g., 0x81)
    bulk_out_ep: u8,   // Endpoint address (e.g., 0x02)
    max_packet_in: u16,
    max_packet_out: u16,
) {
    let bulk_in_num = bulk_in_ep & 0x0F;
    let bulk_out_num = bulk_out_ep & 0x0F;
    let bulk_in_dci = bulk_in_num * 2 + 1;   // IN endpoint DCI
    let bulk_out_dci = bulk_out_num * 2;      // OUT endpoint DCI
    
    crate::serial_println!("[USB-MS] Initializing mass storage: slot {} IN_DCI={} OUT_DCI={}",
        slot_id, bulk_in_dci, bulk_out_dci);
    
    let mut dev = UsbStorageDevice {
        slot_id,
        lun: 0,
        bulk_in_dci,
        bulk_out_dci,
        max_packet_in,
        max_packet_out,
        block_count: 0,
        block_size: SECTOR_SIZE as u32,
        vendor: String::new(),
        product: String::new(),
        ready: false,
    };
    
    // INQUIRY
    scsi_inquiry(&mut dev);
    
    // Wait for unit ready (retry a few times for slow devices)
    for attempt in 0..5 {
        if scsi_test_unit_ready(&dev) {
            break;
        }
        // Request sense to clear any pending conditions
        if let Some((sk, asc, ascq)) = scsi_request_sense(&dev) {
            crate::serial_println!("[USB-MS] Sense: key={:#x} ASC={:#x} ASCQ={:#x}", sk, asc, ascq);
        }
        if attempt < 4 {
            // Small delay
            for _ in 0..100_000 { core::hint::spin_loop(); }
        }
    }
    
    // READ CAPACITY
    if scsi_read_capacity(&mut dev) {
        dev.ready = true;
    }
    
    STORAGE_DEVICES.lock().push(dev);
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

impl UsbBlockDevice {
    pub fn new(device_index: usize) -> Self {
        Self { device_index }
    }
}

impl BlockDevice for UsbBlockDevice {
    fn read_sector(&self, sector: u64, buffer: &mut [u8]) -> Result<(), ()> {
        let devices = STORAGE_DEVICES.lock();
        let dev = devices.get(self.device_index).ok_or(())?;
        if !dev.ready { return Err(()); }
        
        let required = dev.block_size as usize;
        if buffer.len() < required { return Err(()); }
        
        if scsi_read(dev, sector as u32, 1, &mut buffer[..required]) {
            Ok(())
        } else {
            Err(())
        }
    }
    
    fn write_sector(&self, sector: u64, buffer: &[u8]) -> Result<(), ()> {
        let devices = STORAGE_DEVICES.lock();
        let dev = devices.get(self.device_index).ok_or(())?;
        if !dev.ready { return Err(()); }
        
        let required = dev.block_size as usize;
        if buffer.len() < required { return Err(()); }
        
        if scsi_write(dev, sector as u32, 1, &buffer[..required]) {
            Ok(())
        } else {
            Err(())
        }
    }
    
    fn sector_size(&self) -> usize {
        let devices = STORAGE_DEVICES.lock();
        devices.get(self.device_index)
            .map(|dev| dev.block_size as usize)
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
    STORAGE_DEVICES.lock().iter().map(|dev| {
        let name = if dev.vendor.is_empty() && dev.product.is_empty() {
            alloc::format!("USB Storage (slot {})", dev.slot_id)
        } else {
            alloc::format!("{} {}", dev.vendor, dev.product)
        };
        (name, dev.block_count, dev.block_size)
    }).collect()
}

/// Read sectors from USB storage device
pub fn read_sectors(device_index: usize, start_lba: u64, count: usize, buffer: &mut [u8]) -> Result<(), &'static str> {
    let devices = STORAGE_DEVICES.lock();
    let dev = devices.get(device_index).ok_or("Invalid device index")?;
    if !dev.ready { return Err("Device not ready"); }
    
    let block_size = dev.block_size as usize;
    if buffer.len() < count * block_size {
        return Err("Buffer too small");
    }
    
    // Read in chunks of MAX_TRANSFER_SECTORS
    let mut lba = start_lba as u32;
    let mut offset = 0;
    let mut remaining = count;
    
    while remaining > 0 {
        let chunk = remaining.min(MAX_TRANSFER_SECTORS);
        let byte_count = chunk * block_size;
        
        if !scsi_read(dev, lba, chunk as u16, &mut buffer[offset..offset + byte_count]) {
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
    let dev = devices.get(device_index).ok_or("Invalid device index")?;
    if !dev.ready { return Err("Device not ready"); }
    
    let block_size = dev.block_size as usize;
    if buffer.len() < count * block_size {
        return Err("Buffer too small");
    }
    
    let mut lba = start_lba as u32;
    let mut offset = 0;
    let mut remaining = count;
    
    while remaining > 0 {
        let chunk = remaining.min(MAX_TRANSFER_SECTORS);
        let byte_count = chunk * block_size;
        
        if !scsi_write(dev, lba, chunk as u16, &buffer[offset..offset + byte_count]) {
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
