//! ATA/IDE Driver
//! 
//! Provides legacy IDE storage access for older systems.
//! Uses PIO mode for simplicity (DMA would be faster but more complex).

use alloc::string::String;
use alloc::vec::Vec;
use spin::Mutex;
use x86_64::instructions::port::Port;

/// IDE Channels
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum IdeChannel {
    Primary,
    Secondary,
}

/// Drive position on channel
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum DrivePosition {
    Master,
    Slave,
}

/// ATA Commands
mod cmd {
    pub const IDENTIFY: u8 = 0xEC;
    pub const IDENTIFY_PACKET: u8 = 0xA1;
    pub const READ_SECTORS: u8 = 0x20;
    pub const WRITE_SECTORS: u8 = 0x30;
    pub const CACHE_FLUSH: u8 = 0xE7;
}

/// Status register bits
mod status {
    pub const ERR: u8 = 1 << 0;
    pub const DRQ: u8 = 1 << 3;
    pub const DF: u8 = 1 << 5;
    pub const BSY: u8 = 1 << 7;
}

/// IDE ports
const PRIMARY_DATA: u16 = 0x1F0;
const PRIMARY_CONTROL: u16 = 0x3F6;
const SECONDARY_DATA: u16 = 0x170;
const SECONDARY_CONTROL: u16 = 0x376;

/// IDE Drive info
#[derive(Clone, Debug)]
pub struct IdeDriveInfo {
    pub channel: IdeChannel,
    pub position: DrivePosition,
    pub present: bool,
    pub atapi: bool,
    pub lba48: bool,
    pub sector_count: u64,
    pub model: String,
    pub serial: String,
}

/// IDE Controller state
pub struct IdeController {
    pub drives: Vec<IdeDriveInfo>,
    pub initialized: bool,
}

static CONTROLLER: Mutex<Option<IdeController>> = Mutex::new(None);

/// Wait for drive ready
fn wait_ready(base: u16) -> Result<(), &'static str> {
    let mut status_port = Port::<u8>::new(base + 7);
    
    for _ in 0..100_000 {
        let status = unsafe { status_port.read() };
        
        if status == 0xFF {
            return Err("No drive");
        }
        
        if status & status::BSY == 0 {
            if status & status::ERR != 0 {
                return Err("Drive error");
            }
            if status & status::DF != 0 {
                return Err("Drive fault");
            }
            return Ok(());
        }
        
        core::hint::spin_loop();
    }
    
    Err("Timeout")
}

/// Wait for DRQ
fn wait_drq(base: u16) -> Result<(), &'static str> {
    let mut status_port = Port::<u8>::new(base + 7);
    
    for _ in 0..100_000 {
        let status = unsafe { status_port.read() };
        
        if status & status::ERR != 0 {
            return Err("Drive error");
        }
        
        if status & status::BSY == 0 && status & status::DRQ != 0 {
            return Ok(());
        }
        
        core::hint::spin_loop();
    }
    
    Err("Timeout waiting for DRQ")
}

/// Soft reset a channel
fn soft_reset(control_base: u16) {
    let mut control = Port::<u8>::new(control_base);
    unsafe {
        control.write(0x04);
        for _ in 0..1000 { core::hint::spin_loop(); }
        control.write(0x00);
        for _ in 0..10000 { core::hint::spin_loop(); }
    }
}

/// Select a drive
fn select_drive(base: u16, slave: bool) {
    let mut drive_port = Port::<u8>::new(base + 6);
    unsafe {
        drive_port.write(if slave { 0xB0 } else { 0xA0 });
        for _ in 0..4 {
            let _ = Port::<u8>::new(base + 7).read();
        }
    }
}

/// Identify a drive
fn identify_drive(base: u16, _control: u16, slave: bool) -> Option<IdeDriveInfo> {
    select_drive(base, slave);
    
    // Clear sector count and LBA registers
    unsafe {
        Port::<u8>::new(base + 2).write(0);
        Port::<u8>::new(base + 3).write(0);
        Port::<u8>::new(base + 4).write(0);
        Port::<u8>::new(base + 5).write(0);
    }
    
    // Send IDENTIFY command
    unsafe {
        Port::<u8>::new(base + 7).write(cmd::IDENTIFY);
    }
    
    // Check for drive presence
    let status = unsafe { Port::<u8>::new(base + 7).read() };
    if status == 0 {
        return None;
    }
    
    // Wait for not busy
    let mut atapi = false;
    if wait_ready(base).is_err() {
        let lba_mid = unsafe { Port::<u8>::new(base + 4).read() };
        let lba_high = unsafe { Port::<u8>::new(base + 5).read() };
        
        if lba_mid == 0x14 && lba_high == 0xEB {
            atapi = true;
            unsafe {
                Port::<u8>::new(base + 7).write(cmd::IDENTIFY_PACKET);
            }
            if wait_ready(base).is_err() {
                return None;
            }
        } else {
            return None;
        }
    }
    
    // Wait for data
    if wait_drq(base).is_err() {
        return None;
    }
    
    // Read 256 words of identify data
    let mut data = [0u16; 256];
    let mut data_port = Port::<u16>::new(base);
    for i in 0..256 {
        data[i] = unsafe { data_port.read() };
    }
    
    // Parse identify data
    let lba48 = (data[83] & (1 << 10)) != 0;
    
    let sector_count = if lba48 {
        (data[100] as u64) |
        ((data[101] as u64) << 16) |
        ((data[102] as u64) << 32) |
        ((data[103] as u64) << 48)
    } else {
        (data[60] as u64) | ((data[61] as u64) << 16)
    };
    
    // Extract model string (words 27-46)
    let mut model = String::new();
    for i in 27..47 {
        let word = data[i];
        let c1 = ((word >> 8) & 0xFF) as u8;
        let c2 = (word & 0xFF) as u8;
        if c1 > 0x20 && c1 < 0x7F { model.push(c1 as char); }
        if c2 > 0x20 && c2 < 0x7F { model.push(c2 as char); }
    }
    
    // Extract serial (words 10-19)
    let mut serial = String::new();
    for i in 10..20 {
        let word = data[i];
        let c1 = ((word >> 8) & 0xFF) as u8;
        let c2 = (word & 0xFF) as u8;
        if c1 > 0x20 && c1 < 0x7F { serial.push(c1 as char); }
        if c2 > 0x20 && c2 < 0x7F { serial.push(c2 as char); }
    }
    
    let channel = if base == PRIMARY_DATA { IdeChannel::Primary } else { IdeChannel::Secondary };
    let position = if slave { DrivePosition::Slave } else { DrivePosition::Master };
    
    Some(IdeDriveInfo {
        channel,
        position,
        present: true,
        atapi,
        lba48,
        sector_count,
        model: model.trim_end().into(),
        serial: serial.trim_end().into(),
    })
}

/// Initialize IDE controller
pub fn init_ide() -> bool {
    crate::serial_println!("[IDE] Probing IDE channels...");
    
    let mut drives = Vec::new();
    
    // Reset and probe primary channel
    soft_reset(PRIMARY_CONTROL);
    
    if let Some(drive) = identify_drive(PRIMARY_DATA, PRIMARY_CONTROL, false) {
        crate::serial_println!("[IDE] Primary Master: {} ({} sectors)", 
            drive.model, drive.sector_count);
        drives.push(drive);
    }
    
    if let Some(drive) = identify_drive(PRIMARY_DATA, PRIMARY_CONTROL, true) {
        crate::serial_println!("[IDE] Primary Slave: {} ({} sectors)", 
            drive.model, drive.sector_count);
        drives.push(drive);
    }
    
    // Reset and probe secondary channel
    soft_reset(SECONDARY_CONTROL);
    
    if let Some(drive) = identify_drive(SECONDARY_DATA, SECONDARY_CONTROL, false) {
        crate::serial_println!("[IDE] Secondary Master: {} ({} sectors)", 
            drive.model, drive.sector_count);
        drives.push(drive);
    }
    
    if let Some(drive) = identify_drive(SECONDARY_DATA, SECONDARY_CONTROL, true) {
        crate::serial_println!("[IDE] Secondary Slave: {} ({} sectors)", 
            drive.model, drive.sector_count);
        drives.push(drive);
    }
    
    let has_drives = !drives.is_empty();
    
    *CONTROLLER.lock() = Some(IdeController {
        drives,
        initialized: has_drives,
    });
    
    crate::serial_println!("[IDE] Found {} drives", 
        CONTROLLER.lock().as_ref().map(|c| c.drives.len()).unwrap_or(0));
    
    has_drives
}

/// Read sectors using PIO
pub fn read_sectors(channel: IdeChannel, slave: bool, lba: u64, count: u8, buffer: &mut [u8]) -> Result<(), &'static str> {
    let base = match channel {
        IdeChannel::Primary => PRIMARY_DATA,
        IdeChannel::Secondary => SECONDARY_DATA,
    };
    
    if buffer.len() < (count as usize) * 512 {
        return Err("Buffer too small");
    }
    
    select_drive(base, slave);
    wait_ready(base)?;
    
    unsafe {
        Port::<u8>::new(base + 2).write(count);
        Port::<u8>::new(base + 3).write(lba as u8);
        Port::<u8>::new(base + 4).write((lba >> 8) as u8);
        Port::<u8>::new(base + 5).write((lba >> 16) as u8);
        Port::<u8>::new(base + 6).write(0xE0 | (if slave { 0x10 } else { 0 }) | ((lba >> 24) as u8 & 0x0F));
        Port::<u8>::new(base + 7).write(cmd::READ_SECTORS);
    }
    
    let mut data_port = Port::<u16>::new(base);
    let mut offset = 0;
    
    for _ in 0..count {
        wait_drq(base)?;
        
        for _ in 0..256 {
            let word = unsafe { data_port.read() };
            buffer[offset] = (word & 0xFF) as u8;
            buffer[offset + 1] = ((word >> 8) & 0xFF) as u8;
            offset += 2;
        }
    }
    
    Ok(())
}

/// Write sectors using PIO
pub fn write_sectors(channel: IdeChannel, slave: bool, lba: u64, count: u8, buffer: &[u8]) -> Result<(), &'static str> {
    let base = match channel {
        IdeChannel::Primary => PRIMARY_DATA,
        IdeChannel::Secondary => SECONDARY_DATA,
    };
    
    if buffer.len() < (count as usize) * 512 {
        return Err("Buffer too small");
    }
    
    select_drive(base, slave);
    wait_ready(base)?;
    
    unsafe {
        Port::<u8>::new(base + 2).write(count);
        Port::<u8>::new(base + 3).write(lba as u8);
        Port::<u8>::new(base + 4).write((lba >> 8) as u8);
        Port::<u8>::new(base + 5).write((lba >> 16) as u8);
        Port::<u8>::new(base + 6).write(0xE0 | (if slave { 0x10 } else { 0 }) | ((lba >> 24) as u8 & 0x0F));
        Port::<u8>::new(base + 7).write(cmd::WRITE_SECTORS);
    }
    
    let mut data_port = Port::<u16>::new(base);
    let mut offset = 0;
    
    for _ in 0..count {
        wait_drq(base)?;
        
        for _ in 0..256 {
            let word = (buffer[offset] as u16) | ((buffer[offset + 1] as u16) << 8);
            unsafe { data_port.write(word); }
            offset += 2;
        }
    }
    
    // Flush cache
    unsafe {
        Port::<u8>::new(base + 7).write(cmd::CACHE_FLUSH);
    }
    wait_ready(base)?;
    
    Ok(())
}

/// Get list of detected drives
pub fn list_drives() -> Vec<IdeDriveInfo> {
    CONTROLLER.lock().as_ref().map(|c| c.drives.clone()).unwrap_or_default()
}

/// Check if initialized
pub fn is_initialized() -> bool {
    CONTROLLER.lock().as_ref().map(|c| c.initialized).unwrap_or(false)
}
