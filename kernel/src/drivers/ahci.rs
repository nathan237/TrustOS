//! AHCI (Advanced Host Controller Interface) Driver
//! 
//! Provides SATA storage access for modern systems.
//! AHCI is the standard for SATA controllers since ~2004.

use alloc::vec::Vec;
use alloc::vec;
use alloc::string::String;
use alloc::format;
use alloc::boxed::Box;
use spin::Mutex;
use core::ptr;

// ============================================================================
// FIS (Frame Information Structure) Types
// ============================================================================

/// FIS Type identifiers
#[repr(u8)]
#[derive(Clone, Copy)]
pub enum FisType {
    RegH2D = 0x27,      // Register FIS - Host to Device
    RegD2H = 0x34,      // Register FIS - Device to Host
    DmaActivate = 0x39, // DMA Activate FIS
    DmaSetup = 0x41,    // DMA Setup FIS
    Data = 0x46,        // Data FIS
    Bist = 0x58,        // BIST Activate FIS
    PioSetup = 0x5F,    // PIO Setup FIS
    DevBits = 0xA1,     // Set Device Bits FIS
}

/// FIS Register Host to Device
#[repr(C, packed)]
#[derive(Clone, Copy)]
pub struct FisRegH2D {
    pub fis_type: u8,   // FisType::RegH2D
    pub pmport_c: u8,   // PM Port | C bit (command/control)
    pub command: u8,    // ATA command
    pub featurel: u8,   // Feature low byte
    
    pub lba0: u8,       // LBA 7:0
    pub lba1: u8,       // LBA 15:8
    pub lba2: u8,       // LBA 23:16
    pub device: u8,     // Device register
    
    pub lba3: u8,       // LBA 31:24
    pub lba4: u8,       // LBA 39:32
    pub lba5: u8,       // LBA 47:40
    pub featureh: u8,   // Feature high byte
    
    pub countl: u8,     // Sector count low
    pub counth: u8,     // Sector count high
    pub icc: u8,        // Isochronous command completion
    pub control: u8,    // Control register
    
    pub _reserved: [u8; 4],
}

impl FisRegH2D {
    pub const fn new() -> Self {
        Self {
            fis_type: FisType::RegH2D as u8,
            pmport_c: 0,
            command: 0,
            featurel: 0,
            lba0: 0, lba1: 0, lba2: 0,
            device: 0,
            lba3: 0, lba4: 0, lba5: 0,
            featureh: 0,
            countl: 0, counth: 0,
            icc: 0, control: 0,
            _reserved: [0; 4],
        }
    }
}

/// FIS Register Device to Host
#[repr(C, packed)]
pub struct FisRegD2H {
    pub fis_type: u8,
    pub pmport_i: u8,
    pub status: u8,
    pub error: u8,
    pub lba0: u8, pub lba1: u8, pub lba2: u8,
    pub device: u8,
    pub lba3: u8, pub lba4: u8, pub lba5: u8,
    pub _reserved0: u8,
    pub countl: u8, pub counth: u8,
    pub _reserved1: [u8; 6],
}

/// FIS PIO Setup
#[repr(C, packed)]
pub struct FisPioSetup {
    pub fis_type: u8,
    pub pmport_di: u8,
    pub status: u8,
    pub error: u8,
    pub lba0: u8, pub lba1: u8, pub lba2: u8,
    pub device: u8,
    pub lba3: u8, pub lba4: u8, pub lba5: u8,
    pub _reserved0: u8,
    pub countl: u8, pub counth: u8,
    pub _reserved1: u8,
    pub e_status: u8,
    pub tc: u16,
    pub _reserved2: [u8; 2],
}

/// FIS DMA Setup
#[repr(C, packed)]
pub struct FisDmaSetup {
    pub fis_type: u8,
    pub pmport_dai: u8,
    pub _reserved0: [u8; 2],
    pub dma_buffer_id: u64,
    pub _reserved1: u32,
    pub dma_buffer_offset: u32,
    pub transfer_count: u32,
    pub _reserved2: u32,
}

/// Received FIS structure (256 bytes, must be 256-byte aligned)
#[repr(C, align(256))]
pub struct HbaFis {
    pub dsfis: FisDmaSetup,       // 0x00-0x1B
    pub _pad0: [u8; 4],
    pub psfis: FisPioSetup,       // 0x20-0x33
    pub _pad1: [u8; 12],
    pub rfis: FisRegD2H,          // 0x40-0x53
    pub _pad2: [u8; 4],
    pub sdbfis: [u8; 8],          // 0x58-0x5F Set Device Bits
    pub ufis: [u8; 64],           // 0x60-0x9F Unknown FIS
    pub _reserved: [u8; 0x60],    // 0xA0-0xFF
}

// ============================================================================
// Command Structures
// ============================================================================

/// Command Header (32 bytes each, 32 headers = 1KB)
#[repr(C)]
#[derive(Clone, Copy)]
pub struct HbaCmdHeader {
    /// Command FIS length in DWORDs (2-16), ATAPI, Write, Prefetchable
    pub flags: u16,
    /// Physical Region Descriptor Table Length (entries)
    pub prdtl: u16,
    /// Physical Region Descriptor Byte Count (transferred)
    pub prdbc: u32,
    /// Command Table Base Address (128-byte aligned)
    pub ctba: u64,
    /// Reserved
    pub _reserved: [u32; 4],
}

impl HbaCmdHeader {
    pub const fn new() -> Self {
        Self {
            flags: 0,
            prdtl: 0,
            prdbc: 0,
            ctba: 0,
            _reserved: [0; 4],
        }
    }
}

/// Physical Region Descriptor Table Entry
#[repr(C)]
#[derive(Clone, Copy)]
pub struct HbaPrdtEntry {
    /// Data Base Address (2-byte aligned)
    pub dba: u64,
    /// Reserved
    pub _reserved: u32,
    /// Byte count (bit 31 = interrupt on completion)
    /// Actual count is value + 1, max 4MB
    pub dbc_i: u32,
}

/// Command Table (must be 128-byte aligned)
/// Contains the command FIS and PRDT entries
#[repr(C, align(128))]
pub struct HbaCmdTable {
    /// Command FIS (64 bytes)
    pub cfis: [u8; 64],
    /// ATAPI Command (16 bytes)
    pub acmd: [u8; 16],
    /// Reserved
    pub _reserved: [u8; 48],
    /// PRDT entries (up to 65535, we use 8 for simplicity = 128 bytes)
    pub prdt: [HbaPrdtEntry; 8],
}

/// Command List (1KB, 32 command headers)
#[repr(C, align(1024))]
pub struct HbaCmdList {
    pub headers: [HbaCmdHeader; 32],
}

// ============================================================================
// Port Memory Structures (per-port allocation)
// ============================================================================

/// Memory allocated for each active port
pub struct PortMemory {
    pub cmd_list: Box<HbaCmdList>,
    pub fis: Box<HbaFis>,
    pub cmd_tables: [Box<HbaCmdTable>; 8],  // 8 command tables
}

impl PortMemory {
    pub fn new() -> Self {
        Self {
            cmd_list: Box::new(HbaCmdList { headers: [HbaCmdHeader::new(); 32] }),
            fis: Box::new(unsafe { core::mem::zeroed() }),
            cmd_tables: core::array::from_fn(|_| Box::new(unsafe { core::mem::zeroed() })),
        }
    }
}

/// AHCI HBA Memory Registers
#[repr(C)]
pub struct HbaMemory {
    /// Host Capabilities
    pub cap: u32,
    /// Global Host Control
    pub ghc: u32,
    /// Interrupt Status
    pub is: u32,
    /// Ports Implemented
    pub pi: u32,
    /// Version
    pub vs: u32,
    /// Command Completion Coalescing Control
    pub ccc_ctl: u32,
    /// Command Completion Coalescing Ports
    pub ccc_ports: u32,
    /// Enclosure Management Location
    pub em_loc: u32,
    /// Enclosure Management Control
    pub em_ctl: u32,
    /// Host Capabilities Extended
    pub cap2: u32,
    /// BIOS/OS Handoff Control and Status
    pub bohc: u32,
    /// Reserved
    _reserved: [u8; 0x74],
    /// Vendor Specific
    _vendor: [u8; 0x60],
    /// Port registers (up to 32 ports)
    pub ports: [HbaPort; 32],
}

/// AHCI Port Registers
#[repr(C)]
#[derive(Clone, Copy)]
pub struct HbaPort {
    /// Command List Base Address
    pub clb: u64,
    /// FIS Base Address
    pub fb: u64,
    /// Interrupt Status
    pub is: u32,
    /// Interrupt Enable
    pub ie: u32,
    /// Command and Status
    pub cmd: u32,
    /// Reserved
    _reserved0: u32,
    /// Task File Data
    pub tfd: u32,
    /// Signature
    pub sig: u32,
    /// SATA Status
    pub ssts: u32,
    /// SATA Control
    pub sctl: u32,
    /// SATA Error
    pub serr: u32,
    /// SATA Active
    pub sact: u32,
    /// Command Issue
    pub ci: u32,
    /// SATA Notification
    pub sntf: u32,
    /// FIS-based Switching Control
    pub fbs: u32,
    /// Reserved
    _reserved1: [u32; 11],
    /// Vendor Specific
    _vendor: [u32; 4],
}

/// AHCI device type
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AhciDeviceType {
    None,
    Sata,
    Satapi,  // SATA ATAPI (CD/DVD)
    Semb,    // Enclosure management bridge
    Pm,      // Port multiplier
}

/// Port information
#[derive(Debug, Clone)]
pub struct AhciPort {
    pub port_num: u8,
    pub device_type: AhciDeviceType,
    pub sector_count: u64,
    pub model: String,
    pub serial: String,
}

/// AHCI Controller state
pub struct AhciController {
    pub base_addr: u64,
    pub virt_addr: u64,
    pub ports: Vec<AhciPort>,
    pub port_memory: Vec<Option<PortMemory>>,
    pub initialized: bool,
}

static CONTROLLER: Mutex<Option<AhciController>> = Mutex::new(None);

/// SATA signatures
const SATA_SIG_ATA: u32 = 0x00000101;
const SATA_SIG_ATAPI: u32 = 0xEB140101;
const SATA_SIG_SEMB: u32 = 0xC33C0101;
const SATA_SIG_PM: u32 = 0x96690101;

/// Port command bits
const HBA_PORT_CMD_ST: u32 = 1 << 0;   // Start
const HBA_PORT_CMD_FRE: u32 = 1 << 4;  // FIS Receive Enable
const HBA_PORT_CMD_FR: u32 = 1 << 14;  // FIS Receive Running
const HBA_PORT_CMD_CR: u32 = 1 << 15;  // Command List Running

/// ATA Commands
const ATA_CMD_READ_DMA_EXT: u8 = 0x25;
const ATA_CMD_WRITE_DMA_EXT: u8 = 0x35;
const ATA_CMD_IDENTIFY: u8 = 0xEC;

/// ATA Status bits
const ATA_DEV_BUSY: u8 = 0x80;
const ATA_DEV_DRQ: u8 = 0x08;

/// Sector size
const SECTOR_SIZE: usize = 512;

/// Convert virtual address to physical (for DMA)
fn virt_to_phys(virt: u64) -> u64 {
    let hhdm = crate::memory::hhdm_offset();
    virt.wrapping_sub(hhdm)
}

/// Stop a port's command engine
fn stop_cmd(port: &mut HbaPort) {
    // Clear ST (Stop command engine)
    port.cmd &= !HBA_PORT_CMD_ST;
    
    // Clear FRE
    port.cmd &= !HBA_PORT_CMD_FRE;
    
    // Wait until FR and CR are cleared
    for _ in 0..1000 {
        if (port.cmd & HBA_PORT_CMD_FR) == 0 && (port.cmd & HBA_PORT_CMD_CR) == 0 {
            break;
        }
        // Small delay
        for _ in 0..1000 { core::hint::spin_loop(); }
    }
}

/// Start a port's command engine
fn start_cmd(port: &mut HbaPort) {
    // Wait until CR is cleared
    while (port.cmd & HBA_PORT_CMD_CR) != 0 {
        core::hint::spin_loop();
    }
    
    // Set FRE and ST
    port.cmd |= HBA_PORT_CMD_FRE;
    port.cmd |= HBA_PORT_CMD_ST;
}

/// Find a free command slot
fn find_cmdslot(port: &HbaPort) -> Option<u32> {
    // Get slots in use
    let slots = port.sact | port.ci;
    
    for i in 0..32 {
        if (slots & (1 << i)) == 0 {
            return Some(i);
        }
    }
    None
}

/// Initialize AHCI controller
pub fn init(bar5: u64) -> bool {
    if bar5 == 0 || bar5 == 0xFFFFFFFF {
        crate::serial_println!("[AHCI] Invalid BAR5 address");
        return false;
    }
    
    // BAR5 contains ABAR (AHCI Base Address) - physical address
    let abar_phys = (bar5 & !0xF) as u64;
    
    const AHCI_MMIO_SIZE: usize = 0x2000;  // 8KB
    
    crate::serial_println!("[AHCI] Mapping MMIO at phys={:#x} size={:#x}", abar_phys, AHCI_MMIO_SIZE);
    
    let abar_virt = match crate::memory::map_mmio(abar_phys, AHCI_MMIO_SIZE) {
        Ok(virt) => virt,
        Err(e) => {
            crate::serial_println!("[AHCI] Failed to map MMIO: {}", e);
            return false;
        }
    };
    
    crate::serial_println!("[AHCI] Initializing at ABAR phys={:#x} virt={:#x}", abar_phys, abar_virt);
    
    let hba = unsafe { &mut *(abar_virt as *mut HbaMemory) };
    
    // Check AHCI version
    let version = hba.vs;
    let major = (version >> 16) & 0xFF;
    let minor = version & 0xFF;
    crate::serial_println!("[AHCI] Version {}.{}", major, minor);
    
    let pi = hba.pi;
    let cap = hba.cap;
    let num_cmd_slots = ((cap >> 8) & 0x1F) + 1;
    
    crate::serial_println!("[AHCI] {} ports implemented, {} command slots", 
        pi.count_ones(), num_cmd_slots);
    
    // Enable AHCI mode
    hba.ghc |= 1 << 31;
    
    let mut ports = Vec::new();
    let mut port_memory: Vec<Option<PortMemory>> = (0..32).map(|_| None).collect();
    
    // Probe each implemented port
    for i in 0..32 {
        if pi & (1 << i) != 0 {
            let port = unsafe { &mut *(hba.ports.as_mut_ptr().add(i)) };
            
            let ssts = port.ssts;
            let det = ssts & 0x0F;
            let ipm = (ssts >> 8) & 0x0F;
            
            if det == 3 && ipm == 1 {
                let sig = port.sig;
                let device_type = match sig {
                    SATA_SIG_ATA => AhciDeviceType::Sata,
                    SATA_SIG_ATAPI => AhciDeviceType::Satapi,
                    SATA_SIG_SEMB => AhciDeviceType::Semb,
                    SATA_SIG_PM => AhciDeviceType::Pm,
                    _ => AhciDeviceType::None,
                };
                
                if device_type != AhciDeviceType::None {
                    crate::serial_println!("[AHCI] Port {}: {:?} device detected", i, device_type);
                    
                    // Allocate port memory structures
                    let mem = PortMemory::new();
                    
                    // Stop command engine before reconfiguring
                    stop_cmd(port);
                    
                    // Set Command List Base Address
                    let clb_phys = virt_to_phys(&*mem.cmd_list as *const _ as u64);
                    port.clb = clb_phys;
                    
                    // Set FIS Base Address
                    let fb_phys = virt_to_phys(&*mem.fis as *const _ as u64);
                    port.fb = fb_phys;
                    
                    // Clear interrupt status
                    port.is = 0xFFFFFFFF;
                    
                    // Clear error register
                    port.serr = 0xFFFFFFFF;
                    
                    // Start command engine
                    start_cmd(port);
                    
                    port_memory[i] = Some(mem);
                    
                    ports.push(AhciPort {
                        port_num: i as u8,
                        device_type,
                        sector_count: 0,
                        model: String::from("Unknown"),
                        serial: String::from("Unknown"),
                    });
                }
            }
        }
    }
    
    let has_devices = !ports.is_empty();
    
    *CONTROLLER.lock() = Some(AhciController {
        base_addr: abar_phys,
        virt_addr: abar_virt,
        ports,
        port_memory,
        initialized: has_devices,
    });
    
    crate::serial_println!("[AHCI] Initialization {}", 
        if has_devices { "complete" } else { "no devices" });
    
    has_devices
}

/// Get number of detected ports with devices
pub fn get_port_count() -> u8 {
    CONTROLLER.lock().as_ref().map(|c| c.ports.len() as u8).unwrap_or(0)
}

/// List detected devices
pub fn list_devices() -> Vec<AhciPort> {
    CONTROLLER.lock().as_ref().map(|c| c.ports.clone()).unwrap_or_default()
}

/// Check if initialized
pub fn is_initialized() -> bool {
    CONTROLLER.lock().as_ref().map(|c| c.initialized).unwrap_or(false)
}

/// Identify a device and get its sector count
/// Must be called after init() to populate device info
pub fn identify_device(port_num: u8) -> Result<u64, &'static str> {
    let mut ctrl = CONTROLLER.lock();
    let controller = ctrl.as_mut().ok_or("AHCI not initialized")?;
    
    if !controller.initialized {
        return Err("AHCI not initialized");
    }
    
    let port_memory = controller.port_memory[port_num as usize].as_mut()
        .ok_or("Port memory not allocated")?;
    
    let hba = unsafe { &mut *(controller.virt_addr as *mut HbaMemory) };
    let port = unsafe { &mut *(hba.ports.as_mut_ptr().add(port_num as usize)) };
    
    // Clear interrupt status
    port.is = 0xFFFFFFFF;
    
    // Find free command slot
    let slot = find_cmdslot(port).ok_or("No free command slot")?;
    
    // Get command header
    let cmd_header = &mut port_memory.cmd_list.headers[slot as usize];
    
    // Setup command header for IDENTIFY
    cmd_header.flags = 5;  // CFL = 5 DWORDs
    cmd_header.prdtl = 1;  // One PRDT entry
    cmd_header.prdbc = 0;
    
    let cmd_table = &mut *port_memory.cmd_tables[slot as usize];
    let cmd_table_phys = virt_to_phys(cmd_table as *const _ as u64);
    cmd_header.ctba = cmd_table_phys;
    
    // Zero out the command table
    unsafe {
        ptr::write_bytes(cmd_table as *mut HbaCmdTable, 0, 1);
    }
    
    // Allocate a buffer for IDENTIFY data (512 bytes)
    let mut identify_buffer = vec![0u8; 512];
    let buffer_phys = virt_to_phys(identify_buffer.as_ptr() as u64);
    
    // Setup PRDT entry
    cmd_table.prdt[0].dba = buffer_phys;
    cmd_table.prdt[0].dbc_i = (512 - 1) | (1 << 31);  // 512 bytes, interrupt on completion
    
    // Setup command FIS (Register H2D) for IDENTIFY
    let cfis = unsafe { &mut *(cmd_table.cfis.as_mut_ptr() as *mut FisRegH2D) };
    cfis.fis_type = FisType::RegH2D as u8;
    cfis.pmport_c = 0x80;  // Command bit
    cfis.command = ATA_CMD_IDENTIFY;
    cfis.device = 0;  // Master device
    cfis.countl = 0;
    cfis.counth = 0;
    cfis.lba0 = 0;
    cfis.lba1 = 0;
    cfis.lba2 = 0;
    cfis.lba3 = 0;
    cfis.lba4 = 0;
    cfis.lba5 = 0;
    
    // Memory barrier
    core::sync::atomic::fence(core::sync::atomic::Ordering::SeqCst);
    
    // Wait for port to be not busy
    let mut spin = 0u32;
    while (port.tfd & ((ATA_DEV_BUSY | ATA_DEV_DRQ) as u32)) != 0 && spin < 1_000_000 {
        spin += 1;
        core::hint::spin_loop();
    }
    if spin >= 1_000_000 {
        return Err("Port busy timeout");
    }
    
    // Issue command
    port.ci = 1 << slot;
    
    // Wait for completion
    let mut timeout = 0u32;
    loop {
        if (port.ci & (1 << slot)) == 0 {
            break;
        }
        if (port.is & (1 << 30)) != 0 {
            return Err("Task file error during IDENTIFY");
        }
        timeout += 1;
        if timeout > 10_000_000 {
            return Err("IDENTIFY command timeout");
        }
        core::hint::spin_loop();
    }
    
    // Parse IDENTIFY data
    // Words 60-61: Total addressable sectors (28-bit LBA)
    // Words 100-103: Total addressable sectors (48-bit LBA)
    let words = unsafe { 
        core::slice::from_raw_parts(identify_buffer.as_ptr() as *const u16, 256) 
    };
    
    // Check if 48-bit LBA is supported (bit 10 of word 83)
    let lba48_supported = (words[83] & (1 << 10)) != 0;
    
    let sector_count = if lba48_supported {
        // Use 48-bit LBA sector count (words 100-103)
        (words[100] as u64) |
        ((words[101] as u64) << 16) |
        ((words[102] as u64) << 32) |
        ((words[103] as u64) << 48)
    } else {
        // Use 28-bit LBA sector count (words 60-61)
        (words[60] as u64) | ((words[61] as u64) << 16)
    };
    
    // Extract model string (words 27-46, 40 chars)
    let mut model = String::new();
    for i in 27..47 {
        let w = words[i];
        let c1 = ((w >> 8) & 0xFF) as u8;
        let c2 = (w & 0xFF) as u8;
        if c1 >= 0x20 && c1 < 0x7F { model.push(c1 as char); }
        if c2 >= 0x20 && c2 < 0x7F { model.push(c2 as char); }
    }
    let model = String::from(model.trim());
    
    crate::serial_println!("[AHCI] Port {}: {} sectors ({} MB), model: {}", 
        port_num, sector_count, sector_count / 2048, model);
    
    // Update port info in controller
    if let Some(port_info) = controller.ports.iter_mut().find(|p| p.port_num == port_num) {
        port_info.sector_count = sector_count;
        port_info.model = model;
    }
    
    Ok(sector_count)
}

/// Identify all devices after initialization
pub fn identify_all_devices() {
    let port_nums: Vec<u8> = {
        CONTROLLER.lock().as_ref()
            .map(|c| c.ports.iter().map(|p| p.port_num).collect())
            .unwrap_or_default()
    };
    
    for port_num in port_nums {
        if let Err(e) = identify_device(port_num) {
            crate::serial_println!("[AHCI] Failed to identify port {}: {}", port_num, e);
        }
    }
}

/// Read sectors from a port
/// port: port number (0-31)
/// lba: starting logical block address
/// count: number of sectors to read (max 128)
/// buffer: destination buffer (must be count * 512 bytes)
pub fn read_sectors(port_num: u8, lba: u64, count: u16, buffer: &mut [u8]) -> Result<usize, &'static str> {
    if count == 0 || count > 128 {
        return Err("Invalid sector count (1-128)");
    }
    
    let required_size = (count as usize) * SECTOR_SIZE;
    if buffer.len() < required_size {
        return Err("Buffer too small");
    }
    
    let mut ctrl = CONTROLLER.lock();
    let controller = ctrl.as_mut().ok_or("AHCI not initialized")?;
    
    if !controller.initialized {
        return Err("AHCI not initialized");
    }
    
    // Find port info
    let port_idx = controller.ports.iter().position(|p| p.port_num == port_num)
        .ok_or("Port not found")?;
    
    let port_memory = controller.port_memory[port_num as usize].as_mut()
        .ok_or("Port memory not allocated")?;
    
    // Access HBA port registers
    let hba = unsafe { &mut *(controller.virt_addr as *mut HbaMemory) };
    let port = unsafe { &mut *(hba.ports.as_mut_ptr().add(port_num as usize)) };
    
    // Clear interrupt status
    port.is = 0xFFFFFFFF;
    
    // Find free command slot
    let slot = find_cmdslot(port).ok_or("No free command slot")?;
    
    // Get command header
    let cmd_header = &mut port_memory.cmd_list.headers[slot as usize];
    
    // Setup command header
    // CFL (Command FIS Length) = 5 DWORDs = 20 bytes / 4 = 5
    // PRDTL = 1 (one PRDT entry)
    cmd_header.flags = 5;  // CFL = 5 DWORDs
    cmd_header.prdtl = 1;  // One PRDT entry
    cmd_header.prdbc = 0;  // Clear bytes count
    
    // Setup command table address
    let cmd_table = &mut *port_memory.cmd_tables[slot as usize];
    let cmd_table_phys = virt_to_phys(cmd_table as *const _ as u64);
    cmd_header.ctba = cmd_table_phys;
    
    // Zero out the command table
    unsafe {
        ptr::write_bytes(cmd_table as *mut HbaCmdTable, 0, 1);
    }
    
    // Setup PRDT entry - point to buffer
    // We need physical address of buffer
    let buffer_phys = virt_to_phys(buffer.as_ptr() as u64);
    cmd_table.prdt[0].dba = buffer_phys;
    cmd_table.prdt[0].dbc_i = ((required_size - 1) as u32) | (1 << 31);  // Byte count - 1, interrupt on completion
    
    // Setup command FIS (Register H2D)
    let cfis = unsafe { &mut *(cmd_table.cfis.as_mut_ptr() as *mut FisRegH2D) };
    cfis.fis_type = FisType::RegH2D as u8;
    cfis.pmport_c = 0x80;  // Command (bit 7 = 1)
    cfis.command = ATA_CMD_READ_DMA_EXT;
    
    // LBA addressing
    cfis.lba0 = (lba & 0xFF) as u8;
    cfis.lba1 = ((lba >> 8) & 0xFF) as u8;
    cfis.lba2 = ((lba >> 16) & 0xFF) as u8;
    cfis.device = 0x40;  // LBA mode
    cfis.lba3 = ((lba >> 24) & 0xFF) as u8;
    cfis.lba4 = ((lba >> 32) & 0xFF) as u8;
    cfis.lba5 = ((lba >> 40) & 0xFF) as u8;
    
    // Sector count
    cfis.countl = (count & 0xFF) as u8;
    cfis.counth = ((count >> 8) & 0xFF) as u8;
    
    // Memory barrier
    core::sync::atomic::fence(core::sync::atomic::Ordering::SeqCst);
    
    // Wait for port to be not busy
    let mut spin = 0u32;
    while (port.tfd & ((ATA_DEV_BUSY | ATA_DEV_DRQ) as u32)) != 0 && spin < 1_000_000 {
        spin += 1;
        core::hint::spin_loop();
    }
    if spin >= 1_000_000 {
        return Err("Port busy timeout");
    }
    
    // Issue command
    port.ci = 1 << slot;
    
    // Wait for completion
    let mut timeout = 0u32;
    loop {
        // Check if command completed
        if (port.ci & (1 << slot)) == 0 {
            break;
        }
        
        // Check for task file error
        if (port.is & (1 << 30)) != 0 {
            return Err("Task file error");
        }
        
        timeout += 1;
        if timeout > 10_000_000 {
            return Err("Command timeout");
        }
        
        core::hint::spin_loop();
    }
    
    // Check for errors
    if (port.is & (1 << 30)) != 0 {
        return Err("Task file error after completion");
    }
    
    Ok(required_size)
}

/// Write sectors to a port
pub fn write_sectors(port_num: u8, lba: u64, count: u16, buffer: &[u8]) -> Result<usize, &'static str> {
    if count == 0 || count > 128 {
        return Err("Invalid sector count (1-128)");
    }
    
    let required_size = (count as usize) * SECTOR_SIZE;
    if buffer.len() < required_size {
        return Err("Buffer too small");
    }
    
    let mut ctrl = CONTROLLER.lock();
    let controller = ctrl.as_mut().ok_or("AHCI not initialized")?;
    
    if !controller.initialized {
        return Err("AHCI not initialized");
    }
    
    let port_memory = controller.port_memory[port_num as usize].as_mut()
        .ok_or("Port memory not allocated")?;
    
    let hba = unsafe { &mut *(controller.virt_addr as *mut HbaMemory) };
    let port = unsafe { &mut *(hba.ports.as_mut_ptr().add(port_num as usize)) };
    
    port.is = 0xFFFFFFFF;
    
    let slot = find_cmdslot(port).ok_or("No free command slot")?;
    
    let cmd_header = &mut port_memory.cmd_list.headers[slot as usize];
    
    // Write bit is bit 6 in flags
    cmd_header.flags = 5 | (1 << 6);  // CFL = 5, Write = 1
    cmd_header.prdtl = 1;
    cmd_header.prdbc = 0;
    
    let cmd_table = &mut *port_memory.cmd_tables[slot as usize];
    let cmd_table_phys = virt_to_phys(cmd_table as *const _ as u64);
    cmd_header.ctba = cmd_table_phys;
    
    unsafe {
        ptr::write_bytes(cmd_table as *mut HbaCmdTable, 0, 1);
    }
    
    let buffer_phys = virt_to_phys(buffer.as_ptr() as u64);
    cmd_table.prdt[0].dba = buffer_phys;
    cmd_table.prdt[0].dbc_i = ((required_size - 1) as u32) | (1 << 31);
    
    let cfis = unsafe { &mut *(cmd_table.cfis.as_mut_ptr() as *mut FisRegH2D) };
    cfis.fis_type = FisType::RegH2D as u8;
    cfis.pmport_c = 0x80;
    cfis.command = ATA_CMD_WRITE_DMA_EXT;
    
    cfis.lba0 = (lba & 0xFF) as u8;
    cfis.lba1 = ((lba >> 8) & 0xFF) as u8;
    cfis.lba2 = ((lba >> 16) & 0xFF) as u8;
    cfis.device = 0x40;
    cfis.lba3 = ((lba >> 24) & 0xFF) as u8;
    cfis.lba4 = ((lba >> 32) & 0xFF) as u8;
    cfis.lba5 = ((lba >> 40) & 0xFF) as u8;
    
    cfis.countl = (count & 0xFF) as u8;
    cfis.counth = ((count >> 8) & 0xFF) as u8;
    
    core::sync::atomic::fence(core::sync::atomic::Ordering::SeqCst);
    
    let mut spin = 0u32;
    while (port.tfd & ((ATA_DEV_BUSY | ATA_DEV_DRQ) as u32)) != 0 && spin < 1_000_000 {
        spin += 1;
        core::hint::spin_loop();
    }
    if spin >= 1_000_000 {
        return Err("Port busy timeout");
    }
    
    port.ci = 1 << slot;
    
    let mut timeout = 0u32;
    loop {
        if (port.ci & (1 << slot)) == 0 {
            break;
        }
        
        if (port.is & (1 << 30)) != 0 {
            return Err("Task file error");
        }
        
        timeout += 1;
        if timeout > 10_000_000 {
            return Err("Command timeout");
        }
        
        core::hint::spin_loop();
    }
    
    if (port.is & (1 << 30)) != 0 {
        return Err("Task file error after completion");
    }
    
    Ok(required_size)
}

// ============================================================================
// Secure Storage API
// ============================================================================

use crate::security::{StorageOperation, StorageSecurityError, DiskId};
use crate::security::storage;

/// Secure read sectors - checks permissions before reading
/// 
/// This is the preferred API for userspace programs.
/// Kernel code can use read_sectors() directly.
pub fn secure_read_sectors(
    port_num: u8,
    lba: u64,
    count: u16,
    buffer: &mut [u8],
    task_id: u64,
) -> Result<usize, StorageError> {
    let disk = DiskId(port_num);
    let op = StorageOperation::ReadSectors;
    
    // Check security
    storage::check_operation(disk, op, task_id)
        .map_err(StorageError::Security)?;
    
    // Audit the operation
    storage::audit_operation(task_id, disk, op, true);
    
    // Perform the read
    read_sectors(port_num, lba, count, buffer)
        .map_err(StorageError::Io)
}

/// Secure write sectors - checks permissions before writing
/// 
/// Writing requires elevated privileges (danger level 2).
pub fn secure_write_sectors(
    port_num: u8,
    lba: u64,
    count: u16,
    buffer: &[u8],
    task_id: u64,
) -> Result<usize, StorageError> {
    let disk = DiskId(port_num);
    let op = StorageOperation::WriteSectors;
    
    // Check security
    match storage::check_operation(disk, op, task_id) {
        Ok(()) => {}
        Err(e) => {
            storage::audit_operation(task_id, disk, op, false);
            return Err(StorageError::Security(e));
        }
    }
    
    // Audit the operation
    storage::audit_operation(task_id, disk, op, true);
    
    // Perform the write
    write_sectors(port_num, lba, count, buffer)
        .map_err(StorageError::Io)
}

/// Secure format disk - VERY dangerous, requires explicit unlock
/// 
/// This zeros out the entire disk. Danger level 5.
pub fn secure_format_disk(
    port_num: u8,
    task_id: u64,
) -> Result<(), StorageError> {
    let disk = DiskId(port_num);
    let op = StorageOperation::LowLevelFormat;
    
    // Check security - requires disk to be unlocked
    match storage::check_operation(disk, op, task_id) {
        Ok(()) => {}
        Err(e) => {
            storage::audit_operation(task_id, disk, op, false);
            crate::log_warn!(
                "[AHCI] FORMAT DENIED: task {} tried to format disk {} without permission",
                task_id, port_num
            );
            return Err(StorageError::Security(e));
        }
    }
    
    crate::log_warn!("[AHCI] !!! FORMATTING DISK {} - ALL DATA WILL BE LOST !!!", port_num);
    
    // Get disk size
    let disk_info = get_port_info(port_num).ok_or(StorageError::Io("Port not found"))?;
    let total_sectors = disk_info.sector_count;
    
    if total_sectors == 0 {
        return Err(StorageError::Io("Unknown disk size"));
    }
    
    // Zero buffer (one sector at a time for safety)
    let zero_buffer = [0u8; SECTOR_SIZE];
    
    // Format in chunks
    let mut formatted = 0u64;
    while formatted < total_sectors {
        write_sectors(port_num, formatted, 1, &zero_buffer)
            .map_err(StorageError::Io)?;
        formatted += 1;
        
        // Progress every 1000 sectors
        if formatted % 1000 == 0 {
            crate::log!("[AHCI] Format progress: {}/{} sectors", formatted, total_sectors);
        }
    }
    
    storage::audit_operation(task_id, disk, op, true);
    crate::log!("[AHCI] Disk {} formatted successfully ({} sectors)", port_num, total_sectors);
    
    Ok(())
}

/// Storage error types
#[derive(Debug)]
pub enum StorageError {
    /// Security policy violation
    Security(StorageSecurityError),
    /// I/O error
    Io(&'static str),
}

impl core::fmt::Display for StorageError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Security(e) => write!(f, "Security: {}", e),
            Self::Io(e) => write!(f, "I/O: {}", e),
        }
    }
}

/// Get port info for a specific port
pub fn get_port_info(port_num: u8) -> Option<AhciPort> {
    let ctrl = CONTROLLER.lock();
    let controller = ctrl.as_ref()?;
    controller.ports.iter().find(|p| p.port_num == port_num).cloned()
}
