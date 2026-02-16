//! NVMe Driver — NVM Express 1.4 over PCIe
//!
//! Implements the NVMe specification for SSD storage:
//! - Controller reset and initialization
//! - Admin Submission/Completion Queue pair
//! - I/O Submission/Completion Queue pair
//! - Identify Controller / Identify Namespace
//! - Read / Write commands (LBA-based)
//! - Polling-based completion (no MSI-X)
//!
//! Reference: NVM Express Base Specification 1.4

use alloc::string::String;
use core::sync::atomic::{AtomicBool, Ordering};
use spin::Mutex;

// ═══════════════════════════════════════════════════════════════════════
// NVMe Register Offsets (MMIO BAR0)
// ═══════════════════════════════════════════════════════════════════════

const REG_CAP: u64 = 0x00;     // Controller Capabilities (64-bit)
const REG_VS: u64 = 0x08;      // Version
const REG_INTMS: u64 = 0x0C;   // Interrupt Mask Set
const REG_INTMC: u64 = 0x10;   // Interrupt Mask Clear
const REG_CC: u64 = 0x14;      // Controller Configuration
const REG_CSTS: u64 = 0x1C;    // Controller Status
const REG_AQA: u64 = 0x24;     // Admin Queue Attributes
const REG_ASQ: u64 = 0x28;     // Admin Submission Queue Base (64-bit)
const REG_ACQ: u64 = 0x30;     // Admin Completion Queue Base (64-bit)

// Doorbell stride is variable — base at 0x1000, stride from CAP.DSTRD
const DOORBELL_BASE: u64 = 0x1000;

// Controller Configuration bits
const CC_EN: u32 = 1 << 0;         // Enable
const CC_CSS_NVM: u32 = 0 << 4;    // I/O Command Set: NVM
const CC_MPS_4K: u32 = 0 << 7;     // Memory Page Size: 4KB (2^(12+0))
const CC_AMS_RR: u32 = 0 << 11;    // Arbitration: Round Robin
const CC_IOSQES: u32 = 6 << 16;    // I/O SQ Entry Size: 2^6 = 64 bytes
const CC_IOCQES: u32 = 4 << 20;    // I/O CQ Entry Size: 2^4 = 16 bytes

// Controller Status bits
const CSTS_RDY: u32 = 1 << 0;      // Ready
const CSTS_CFS: u32 = 1 << 1;      // Controller Fatal Status

// ═══════════════════════════════════════════════════════════════════════
// NVMe Command Opcodes
// ═══════════════════════════════════════════════════════════════════════

// Admin commands
const ADMIN_DELETE_IO_SQ: u8 = 0x00;
const ADMIN_CREATE_IO_SQ: u8 = 0x01;
const ADMIN_DELETE_IO_CQ: u8 = 0x04;
const ADMIN_CREATE_IO_CQ: u8 = 0x05;
const ADMIN_IDENTIFY: u8 = 0x06;
const ADMIN_SET_FEATURES: u8 = 0x09;

// NVM I/O commands
const IO_FLUSH: u8 = 0x00;
const IO_WRITE: u8 = 0x01;
const IO_READ: u8 = 0x02;

// Identify CNS values
const IDENTIFY_NAMESPACE: u32 = 0x00;
const IDENTIFY_CONTROLLER: u32 = 0x01;
const IDENTIFY_ACTIVE_NSID_LIST: u32 = 0x02;

// ═══════════════════════════════════════════════════════════════════════
// NVMe Data Structures
// ═══════════════════════════════════════════════════════════════════════

/// NVMe Submission Queue Entry (64 bytes)
#[derive(Clone, Copy, Default)]
#[repr(C)]
struct SqEntry {
    /// Opcode + flags + command ID
    cdw0: u32,
    /// Namespace ID
    nsid: u32,
    /// Reserved
    cdw2: u32,
    cdw3: u32,
    /// Metadata pointer
    mptr: u64,
    /// PRP Entry 1 (data pointer)
    prp1: u64,
    /// PRP Entry 2 (data pointer or PRP list pointer)
    prp2: u64,
    /// Command-specific DWORDs 10-15
    cdw10: u32,
    cdw11: u32,
    cdw12: u32,
    cdw13: u32,
    cdw14: u32,
    cdw15: u32,
}

const _: () = assert!(core::mem::size_of::<SqEntry>() == 64);

/// NVMe Completion Queue Entry (16 bytes)
#[derive(Clone, Copy, Default)]
#[repr(C)]
struct CqEntry {
    /// Command-specific result
    dw0: u32,
    /// Reserved
    dw1: u32,
    /// SQ Head Pointer (15:0) + SQ Identifier (31:16)
    sq_head_sqid: u32,
    /// Command ID (15:0) + Phase Tag (bit 16) + Status Field (31:17)
    cid_status: u32,
}

const _: () = assert!(core::mem::size_of::<CqEntry>() == 16);

impl CqEntry {
    fn phase(&self) -> bool {
        self.cid_status & (1 << 16) != 0
    }

    fn status_code(&self) -> u16 {
        ((self.cid_status >> 17) & 0x7FF) as u16
    }

    fn command_id(&self) -> u16 {
        (self.cid_status & 0xFFFF) as u16
    }
}

/// Queue pair (submission + completion) with physical addresses
struct QueuePair {
    /// Virtual address of SQ (array of SqEntry)
    sq_virt: u64,
    /// Physical address of SQ
    sq_phys: u64,
    /// Virtual address of CQ (array of CqEntry)
    cq_virt: u64,
    /// Physical address of CQ
    cq_phys: u64,
    /// Queue depth (number of entries)
    depth: u16,
    /// SQ tail (next entry to write)
    sq_tail: u16,
    /// CQ head (next entry to read)
    cq_head: u16,
    /// Expected phase bit (toggles each wrap-around)
    cq_phase: bool,
    /// Next command ID
    next_cid: u16,
    /// Queue ID (0 = admin, 1+ = I/O)
    qid: u16,
}

impl QueuePair {
    /// Allocate a new queue pair using physical frame allocator
    fn new(qid: u16, depth: u16) -> Option<Self> {
        // SQ: depth × 64 bytes. One 4KB page fits 64 entries.
        let sq_phys = crate::memory::frame::alloc_frame_zeroed()?;
        let sq_virt = crate::memory::phys_to_virt(sq_phys);
        
        // CQ: depth × 16 bytes. One 4KB page fits 256 entries.
        let cq_phys = crate::memory::frame::alloc_frame_zeroed()?;
        let cq_virt = crate::memory::phys_to_virt(cq_phys);
        
        Some(Self {
            sq_virt,
            sq_phys,
            cq_virt,
            cq_phys,
            depth,
            sq_tail: 0,
            cq_head: 0,
            cq_phase: true,     // Phase starts at 1
            next_cid: 0,
            qid,
        })
    }
    
    /// Submit a command to the SQ. Returns the command ID.
    fn submit(&mut self, mut cmd: SqEntry) -> u16 {
        let cid = self.next_cid;
        self.next_cid = self.next_cid.wrapping_add(1);
        
        // Set command ID in CDW0 bits [31:16]
        cmd.cdw0 = (cmd.cdw0 & 0x0000FFFF) | ((cid as u32) << 16);
        
        // Write to SQ[tail]
        let offset = self.sq_tail as usize * core::mem::size_of::<SqEntry>();
        unsafe {
            let ptr = (self.sq_virt + offset as u64) as *mut SqEntry;
            core::ptr::write_volatile(ptr, cmd);
        }
        
        // Advance tail (wrap around)
        self.sq_tail = (self.sq_tail + 1) % self.depth;
        
        cid
    }
    
    /// Poll CQ for a completion. Returns (CqEntry, true) if found.
    fn poll_completion(&mut self) -> Option<CqEntry> {
        let offset = self.cq_head as usize * core::mem::size_of::<CqEntry>();
        let entry = unsafe {
            let ptr = (self.cq_virt + offset as u64) as *const CqEntry;
            core::ptr::read_volatile(ptr)
        };
        
        // Check phase bit matches expected
        if entry.phase() == self.cq_phase {
            // Advance CQ head
            self.cq_head += 1;
            if self.cq_head >= self.depth {
                self.cq_head = 0;
                self.cq_phase = !self.cq_phase;
            }
            Some(entry)
        } else {
            None
        }
    }
}

/// NVMe Controller
struct NvmeController {
    /// MMIO base virtual address
    bar_virt: u64,
    /// Doorbell stride (in bytes) = 4 << CAP.DSTRD
    doorbell_stride: u32,
    /// Admin queue pair
    admin: QueuePair,
    /// I/O queue pair (queue ID 1)
    io: Option<QueuePair>,
    /// Controller serial number
    serial: String,
    /// Controller model
    model: String,
    /// Namespace 1 size in LBAs
    ns1_size: u64,
    /// Namespace 1 LBA data size (bytes, typically 512)
    ns1_lba_size: u32,
    /// Maximum transfer size (in pages)
    max_transfer_pages: u32,
}

impl NvmeController {
    // ─── MMIO register access ────────────────────────────────────
    
    #[inline]
    fn read32(&self, offset: u64) -> u32 {
        unsafe { core::ptr::read_volatile((self.bar_virt + offset) as *const u32) }
    }
    
    #[inline]
    fn write32(&self, offset: u64, value: u32) {
        unsafe { core::ptr::write_volatile((self.bar_virt + offset) as *mut u32, value) }
    }
    
    #[inline]
    fn read64(&self, offset: u64) -> u64 {
        let lo = self.read32(offset) as u64;
        let hi = self.read32(offset + 4) as u64;
        lo | (hi << 32)
    }
    
    #[inline]
    fn write64(&self, offset: u64, value: u64) {
        self.write32(offset, value as u32);
        self.write32(offset + 4, (value >> 32) as u32);
    }
    
    // ─── Doorbell access ─────────────────────────────────────────
    
    /// Ring the Submission Queue tail doorbell
    fn ring_sq_doorbell(&self, qid: u16, new_tail: u16) {
        let offset = DOORBELL_BASE + (2 * qid as u64) * self.doorbell_stride as u64;
        self.write32(offset, new_tail as u32);
    }
    
    /// Ring the Completion Queue head doorbell
    fn ring_cq_doorbell(&self, qid: u16, new_head: u16) {
        let offset = DOORBELL_BASE + (2 * qid as u64 + 1) * self.doorbell_stride as u64;
        self.write32(offset, new_head as u32);
    }
    
    // ─── Command submission + polling ────────────────────────────
    
    /// Submit an admin command and wait for completion (polling).
    fn admin_cmd(&mut self, cmd: SqEntry) -> Result<CqEntry, &'static str> {
        let _cid = self.admin.submit(cmd);
        self.ring_sq_doorbell(0, self.admin.sq_tail);
        
        // Poll for completion with timeout
        for _ in 0..1_000_000u32 {
            if let Some(cqe) = self.admin.poll_completion() {
                // Update CQ doorbell
                self.ring_cq_doorbell(0, self.admin.cq_head);
                
                if cqe.status_code() != 0 {
                    crate::serial_println!("[NVMe] Admin cmd failed: status={:#x}", cqe.status_code());
                    return Err("NVMe admin command failed");
                }
                return Ok(cqe);
            }
            core::hint::spin_loop();
        }
        Err("NVMe admin command timeout")
    }
    
    /// Submit an I/O command and wait for completion (polling).
    fn io_cmd(&mut self, cmd: SqEntry) -> Result<CqEntry, &'static str> {
        // Submit and extract sq_tail before releasing borrow
        let sq_tail = {
            let io = self.io.as_mut().ok_or("NVMe I/O queue not initialized")?;
            let _cid = io.submit(cmd);
            io.sq_tail
        };
        self.ring_sq_doorbell(1, sq_tail);
        
        // Poll for completion
        for _ in 0..10_000_000u32 {
            let io = self.io.as_mut().unwrap();
            if let Some(cqe) = io.poll_completion() {
                let cq_head = io.cq_head;
                self.ring_cq_doorbell(1, cq_head);
                
                if cqe.status_code() != 0 {
                    crate::serial_println!("[NVMe] I/O cmd failed: status={:#x}", cqe.status_code());
                    return Err("NVMe I/O command failed");
                }
                return Ok(cqe);
            }
            core::hint::spin_loop();
        }
        Err("NVMe I/O command timeout")
    }
    
    // ─── Identify commands ───────────────────────────────────────
    
    /// Identify Controller (CNS=1). Writes 4KB data to `buf_phys`.
    fn identify_controller(&mut self) -> Result<(), &'static str> {
        let buf_phys = crate::memory::frame::alloc_frame_zeroed()
            .ok_or("NVMe: OOM for identify buffer")?;
        let buf_virt = crate::memory::phys_to_virt(buf_phys);
        
        let cmd = SqEntry {
            cdw0: ADMIN_IDENTIFY as u32,
            prp1: buf_phys,
            cdw10: IDENTIFY_CONTROLLER,
            ..Default::default()
        };
        
        self.admin_cmd(cmd)?;
        
        // Parse Identify Controller data
        unsafe {
            let data = buf_virt as *const u8;
            
            // Serial Number: bytes 4-23 (20 chars, ASCII)
            let mut sn = [0u8; 20];
            core::ptr::copy_nonoverlapping(data.add(4), sn.as_mut_ptr(), 20);
            self.serial = core::str::from_utf8(&sn)
                .unwrap_or("?")
                .trim()
                .into();
            
            // Model Number: bytes 24-63 (40 chars, ASCII)
            let mut mn = [0u8; 40];
            core::ptr::copy_nonoverlapping(data.add(24), mn.as_mut_ptr(), 40);
            self.model = core::str::from_utf8(&mn)
                .unwrap_or("?")
                .trim()
                .into();
            
            // MDTS (Maximum Data Transfer Size): byte 77
            // 0 = no limit, else 2^(MDTS) pages
            let mdts = *data.add(77);
            self.max_transfer_pages = if mdts == 0 { 256 } else { 1u32 << mdts };
        }
        
        crate::memory::frame::free_frame(buf_phys);
        Ok(())
    }
    
    /// Identify Namespace (CNS=0, NSID=1). Reads size and LBA format.
    fn identify_namespace(&mut self) -> Result<(), &'static str> {
        let buf_phys = crate::memory::frame::alloc_frame_zeroed()
            .ok_or("NVMe: OOM for identify namespace buffer")?;
        let buf_virt = crate::memory::phys_to_virt(buf_phys);
        
        let cmd = SqEntry {
            cdw0: ADMIN_IDENTIFY as u32,
            nsid: 1,
            prp1: buf_phys,
            cdw10: IDENTIFY_NAMESPACE,
            ..Default::default()
        };
        
        self.admin_cmd(cmd)?;
        
        unsafe {
            let data = buf_virt as *const u8;
            
            // NSZE (Namespace Size): bytes 0-7, in LBAs
            self.ns1_size = core::ptr::read_unaligned(data as *const u64);
            
            // FLBAS (Formatted LBA Size): byte 26, bits [3:0] = LBA format index
            let flbas = *data.add(26) & 0x0F;
            
            // LBAF[flbas]: starts at offset 128, each is 4 bytes
            // bits [23:16] = LBADS (LBA Data Size as power of 2)
            let lbaf_offset = 128 + (flbas as usize) * 4;
            let lbaf = core::ptr::read_unaligned(data.add(lbaf_offset) as *const u32);
            let lbads = (lbaf >> 16) & 0xFF;
            self.ns1_lba_size = 1u32 << lbads;
        }
        
        crate::memory::frame::free_frame(buf_phys);
        Ok(())
    }
    
    // ─── I/O Queue creation ──────────────────────────────────────
    
    /// Create I/O Completion Queue (admin command)
    fn create_io_cq(&mut self, qid: u16, cq_phys: u64, depth: u16) -> Result<(), &'static str> {
        let cmd = SqEntry {
            cdw0: ADMIN_CREATE_IO_CQ as u32,
            prp1: cq_phys,
            // CDW10: QID (15:0) + Queue Size (31:16) — size is 0-based
            cdw10: (qid as u32) | (((depth - 1) as u32) << 16),
            // CDW11: PC=1 (Physically Contiguous), IEN=0, IV=0 (polling, no interrupts)
            cdw11: 1,   // PC=1
            ..Default::default()
        };
        
        self.admin_cmd(cmd)?;
        Ok(())
    }
    
    /// Create I/O Submission Queue (admin command)
    fn create_io_sq(&mut self, qid: u16, sq_phys: u64, cqid: u16, depth: u16) -> Result<(), &'static str> {
        let cmd = SqEntry {
            cdw0: ADMIN_CREATE_IO_SQ as u32,
            prp1: sq_phys,
            // CDW10: QID (15:0) + Queue Size (31:16) — size is 0-based
            cdw10: (qid as u32) | (((depth - 1) as u32) << 16),
            // CDW11: PC=1 (Physically Contiguous) + CQID (31:16)
            cdw11: 1 | ((cqid as u32) << 16),
            ..Default::default()
        };
        
        self.admin_cmd(cmd)?;
        Ok(())
    }
    
    // ─── Read / Write ────────────────────────────────────────────
    
    /// Read LBAs from namespace 1.
    /// `start_lba` = starting LBA
    /// `count` = number of LBAs to read (1-based, max limited by MDTS)
    /// `buf_phys` = physical address of destination buffer
    fn read_lbas(&mut self, start_lba: u64, count: u16, buf_phys: u64) -> Result<(), &'static str> {
        let total_bytes = count as u64 * self.ns1_lba_size as u64;
        
        // Build PRP2: if transfer > 4KB, we need a PRP list or second PRP
        let prp2 = if total_bytes <= 4096 {
            0
        } else if total_bytes <= 8192 {
            // Two pages: PRP1 = first page, PRP2 = second page
            buf_phys + 4096
        } else {
            // Need PRP list for > 8KB transfers
            // For now, limit to 8KB (2 pages). Multi-page PRP list can be added later.
            return Err("NVMe: transfer > 8KB not supported yet");
        };
        
        let cmd = SqEntry {
            cdw0: IO_READ as u32,
            nsid: 1,
            prp1: buf_phys,
            prp2,
            cdw10: start_lba as u32,               // Starting LBA low
            cdw11: (start_lba >> 32) as u32,        // Starting LBA high
            cdw12: (count - 1) as u32,              // NLB (0-based)
            ..Default::default()
        };
        
        self.io_cmd(cmd)?;
        Ok(())
    }
    
    /// Write LBAs to namespace 1.
    fn write_lbas(&mut self, start_lba: u64, count: u16, buf_phys: u64) -> Result<(), &'static str> {
        let total_bytes = count as u64 * self.ns1_lba_size as u64;
        
        let prp2 = if total_bytes <= 4096 {
            0
        } else if total_bytes <= 8192 {
            buf_phys + 4096
        } else {
            return Err("NVMe: transfer > 8KB not supported yet");
        };
        
        let cmd = SqEntry {
            cdw0: IO_WRITE as u32,
            nsid: 1,
            prp1: buf_phys,
            prp2,
            cdw10: start_lba as u32,
            cdw11: (start_lba >> 32) as u32,
            cdw12: (count - 1) as u32,
            ..Default::default()
        };
        
        self.io_cmd(cmd)?;
        Ok(())
    }
    
    /// Flush (sync) namespace 1.
    fn flush(&mut self) -> Result<(), &'static str> {
        let cmd = SqEntry {
            cdw0: IO_FLUSH as u32,
            nsid: 1,
            ..Default::default()
        };
        self.io_cmd(cmd)?;
        Ok(())
    }
}

// ═══════════════════════════════════════════════════════════════════════
// Global driver state
// ═══════════════════════════════════════════════════════════════════════

static NVME: Mutex<Option<NvmeController>> = Mutex::new(None);
static INITIALIZED: AtomicBool = AtomicBool::new(false);

/// Check if NVMe driver is initialized
pub fn is_initialized() -> bool {
    INITIALIZED.load(Ordering::Relaxed)
}

/// Get NVMe namespace capacity in LBAs
pub fn capacity() -> u64 {
    NVME.lock().as_ref().map(|c| c.ns1_size).unwrap_or(0)
}

/// Get LBA size in bytes
pub fn lba_size() -> u32 {
    NVME.lock().as_ref().map(|c| c.ns1_lba_size).unwrap_or(512)
}

/// Get NVMe drive info
pub fn get_info() -> Option<(String, String, u64, u32)> {
    let ctrl = NVME.lock();
    let c = ctrl.as_ref()?;
    Some((c.model.clone(), c.serial.clone(), c.ns1_size, c.ns1_lba_size))
}

// ═══════════════════════════════════════════════════════════════════════
// Initialization
// ═══════════════════════════════════════════════════════════════════════

/// Initialize the NVMe driver for a given PCI device.
///
/// Performs the full NVMe init sequence:
/// 1. Map BAR0 MMIO
/// 2. Disable controller (CC.EN=0)
/// 3. Allocate admin queues
/// 4. Configure AQA, ASQ, ACQ registers
/// 5. Enable controller (CC.EN=1), wait for CSTS.RDY
/// 6. Identify Controller + Identify Namespace
/// 7. Create I/O queue pair
pub fn init(pci_dev: &crate::pci::PciDevice) -> Result<(), &'static str> {
    crate::serial_println!("[NVMe] Initializing {:02X}:{:02X}.{} ({:04X}:{:04X})",
        pci_dev.bus, pci_dev.device, pci_dev.function,
        pci_dev.vendor_id, pci_dev.device_id);
    
    // ── Step 0: Enable PCI bus mastering + memory space ──
    crate::pci::enable_bus_master(pci_dev);
    crate::pci::enable_memory_space(pci_dev);
    
    // Disable interrupts via PCI command register (we use polling)
    let cmd = crate::pci::config_read16(pci_dev.bus, pci_dev.device, pci_dev.function, 0x04);
    crate::pci::config_write(pci_dev.bus, pci_dev.device, pci_dev.function, 0x04,
        (cmd | (1 << 10)) as u32); // Interrupt Disable bit
    
    // ── Step 1: Map BAR0 (MMIO) ──
    let bar0_phys = pci_dev.bar_address(0).ok_or("NVMe: no BAR0")?;
    if bar0_phys == 0 {
        return Err("NVMe: BAR0 is zero");
    }
    
    // NVMe BAR0 is typically 16KB-64KB. Map 64KB to be safe.
    let bar_virt = crate::memory::map_mmio(bar0_phys, 0x10000)?;
    
    crate::serial_println!("[NVMe] BAR0: phys={:#x}, virt={:#x}", bar0_phys, bar_virt);
    
    // ── Step 2: Read capabilities ──
    let cap = {
        let lo = unsafe { core::ptr::read_volatile((bar_virt + REG_CAP) as *const u32) } as u64;
        let hi = unsafe { core::ptr::read_volatile((bar_virt + REG_CAP + 4) as *const u32) } as u64;
        lo | (hi << 32)
    };
    
    let mqes = (cap & 0xFFFF) as u16 + 1;  // Maximum Queue Entries Supported (0-based)
    let dstrd = ((cap >> 32) & 0xF) as u32; // Doorbell Stride (4 << DSTRD)
    let doorbell_stride = 4u32 << dstrd;
    let mpsmin = ((cap >> 48) & 0xF) as u32; // Min Memory Page Size (2^(12+MPSMIN))
    let timeout_500ms = ((cap >> 24) & 0xFF) as u32; // Timeout in 500ms units
    
    let vs = unsafe { core::ptr::read_volatile((bar_virt + REG_VS) as *const u32) };
    let major = (vs >> 16) & 0xFFFF;
    let minor = (vs >> 8) & 0xFF;
    
    crate::serial_println!("[NVMe] Version: {}.{}, MQES={}, DSTRD={}, MPS_MIN={}KB, Timeout={}ms",
        major, minor, mqes, dstrd, 4 << mpsmin, timeout_500ms * 500);
    
    // Use smaller of MQES and 64 entries (keeps within 1 page)
    let queue_depth = mqes.min(64) as u16;
    
    // ── Step 3: Disable controller ──
    let cc = unsafe { core::ptr::read_volatile((bar_virt + REG_CC) as *const u32) };
    if cc & CC_EN != 0 {
        // Controller is enabled, disable it
        unsafe { core::ptr::write_volatile((bar_virt + REG_CC) as *mut u32, cc & !CC_EN) };
        
        // Wait for CSTS.RDY = 0
        for _ in 0..1_000_000u32 {
            let csts = unsafe { core::ptr::read_volatile((bar_virt + REG_CSTS) as *const u32) };
            if csts & CSTS_RDY == 0 {
                break;
            }
            core::hint::spin_loop();
        }
    }
    
    // ── Step 4: Allocate admin queues ──
    let admin = QueuePair::new(0, queue_depth)
        .ok_or("NVMe: OOM for admin queues")?;
    
    crate::serial_println!("[NVMe] Admin SQ phys={:#x}, CQ phys={:#x}, depth={}",
        admin.sq_phys, admin.cq_phys, queue_depth);
    
    // ── Step 5: Configure admin queue registers ──
    // AQA: Admin Queue Attributes — ACQS (27:16) + ASQS (11:0), both 0-based
    let aqa = ((queue_depth - 1) as u32) | (((queue_depth - 1) as u32) << 16);
    unsafe {
        core::ptr::write_volatile((bar_virt + REG_AQA) as *mut u32, aqa);
        // ASQ: Admin Submission Queue Base Address (64-bit, page-aligned)
        core::ptr::write_volatile((bar_virt + REG_ASQ) as *mut u32, admin.sq_phys as u32);
        core::ptr::write_volatile((bar_virt + REG_ASQ + 4) as *mut u32, (admin.sq_phys >> 32) as u32);
        // ACQ: Admin Completion Queue Base Address (64-bit, page-aligned)
        core::ptr::write_volatile((bar_virt + REG_ACQ) as *mut u32, admin.cq_phys as u32);
        core::ptr::write_volatile((bar_virt + REG_ACQ + 4) as *mut u32, (admin.cq_phys >> 32) as u32);
    }
    
    // Mask all interrupts (we use polling)
    unsafe {
        core::ptr::write_volatile((bar_virt + REG_INTMS) as *mut u32, 0xFFFFFFFF);
    }
    
    // ── Step 6: Enable controller ──
    let cc_val = CC_EN | CC_CSS_NVM | CC_MPS_4K | CC_AMS_RR | CC_IOSQES | CC_IOCQES;
    unsafe {
        core::ptr::write_volatile((bar_virt + REG_CC) as *mut u32, cc_val);
    }
    
    // Wait for CSTS.RDY = 1 (or CFS = 1)
    let mut ready = false;
    for _ in 0..5_000_000u32 {
        let csts = unsafe { core::ptr::read_volatile((bar_virt + REG_CSTS) as *const u32) };
        if csts & CSTS_CFS != 0 {
            return Err("NVMe: Controller Fatal Status during enable");
        }
        if csts & CSTS_RDY != 0 {
            ready = true;
            break;
        }
        core::hint::spin_loop();
    }
    
    if !ready {
        return Err("NVMe: Controller did not become ready");
    }
    
    crate::serial_println!("[NVMe] Controller enabled and ready");
    
    // ── Step 7: Build controller struct ──
    let mut ctrl = NvmeController {
        bar_virt,
        doorbell_stride,
        admin,
        io: None,
        serial: String::new(),
        model: String::new(),
        ns1_size: 0,
        ns1_lba_size: 512,
        max_transfer_pages: 256,
    };
    
    // ── Step 8: Identify Controller ──
    ctrl.identify_controller()?;
    crate::serial_println!("[NVMe] Model: '{}', Serial: '{}'", ctrl.model, ctrl.serial);
    
    // ── Step 9: Identify Namespace 1 ──
    ctrl.identify_namespace()?;
    let size_mb = (ctrl.ns1_size * ctrl.ns1_lba_size as u64) / (1024 * 1024);
    crate::serial_println!("[NVMe] NS1: {} LBAs × {} bytes = {} MB",
        ctrl.ns1_size, ctrl.ns1_lba_size, size_mb);
    
    // ── Step 10: Create I/O Queue Pair (QID=1) ──
    let io_depth = queue_depth;
    let io_queue = QueuePair::new(1, io_depth)
        .ok_or("NVMe: OOM for I/O queues")?;
    
    ctrl.create_io_cq(1, io_queue.cq_phys, io_depth)?;
    ctrl.create_io_sq(1, io_queue.sq_phys, 1, io_depth)?;
    ctrl.io = Some(io_queue);
    
    crate::serial_println!("[NVMe] I/O queue pair created (depth={})", io_depth);
    
    // ── Done! ──
    *NVME.lock() = Some(ctrl);
    INITIALIZED.store(true, Ordering::Release);
    
    crate::serial_println!("[NVMe] ✓ Driver initialized — {} MB NVMe storage available", size_mb);
    
    Ok(())
}

// ═══════════════════════════════════════════════════════════════════════
// Public Read / Write API
// ═══════════════════════════════════════════════════════════════════════

/// Read sectors from NVMe namespace 1.
///
/// `start_lba` — starting LBA
/// `count` — number of sectors (LBAs) to read
/// `buffer` — destination buffer (must be at least count × lba_size bytes)
pub fn read_sectors(start_lba: u64, count: usize, buffer: &mut [u8]) -> Result<(), &'static str> {
    let mut ctrl = NVME.lock();
    let ctrl = ctrl.as_mut().ok_or("NVMe: not initialized")?;
    
    let lba_sz = ctrl.ns1_lba_size as usize;
    let total_bytes = count * lba_sz;
    
    if buffer.len() < total_bytes {
        return Err("NVMe: buffer too small");
    }
    
    if start_lba + count as u64 > ctrl.ns1_size {
        return Err("NVMe: read past end of namespace");
    }
    
    // Process in chunks that fit in 2 pages (8KB, PRP1+PRP2)
    let max_lbas_per_chunk = (8192 / lba_sz).max(1);
    let mut lba = start_lba;
    let mut offset = 0usize;
    let mut remaining = count;
    
    while remaining > 0 {
        let chunk = remaining.min(max_lbas_per_chunk);
        let chunk_bytes = chunk * lba_sz;
        
        // Allocate DMA buffer (use frame allocator for guaranteed physical pages)
        let pages_needed = (chunk_bytes + 4095) / 4096;
        let dma_phys = crate::memory::frame::alloc_frame_zeroed()
            .ok_or("NVMe: OOM for DMA read buffer")?;
        let dma_phys2 = if pages_needed > 1 {
            Some(crate::memory::frame::alloc_frame_zeroed()
                .ok_or("NVMe: OOM for DMA read buffer page 2")?)
        } else {
            None
        };
        
        // For 2-page transfer, set phys of second page correctly
        // NVMe PRP needs physically contiguous or PRP list
        // We use individual frames: PRP1=page1, PRP2=page2
        let actual_prp1 = dma_phys;
        
        ctrl.read_lbas(lba, chunk as u16, actual_prp1)?;
        
        // Copy first page
        let first_page_bytes = chunk_bytes.min(4096);
        let dma_virt = crate::memory::phys_to_virt(dma_phys);
        unsafe {
            core::ptr::copy_nonoverlapping(
                dma_virt as *const u8,
                buffer[offset..].as_mut_ptr(),
                first_page_bytes,
            );
        }
        
        // Copy second page if needed
        if let Some(phys2) = dma_phys2 {
            let remaining_bytes = chunk_bytes - first_page_bytes;
            if remaining_bytes > 0 {
                let virt2 = crate::memory::phys_to_virt(phys2);
                unsafe {
                    core::ptr::copy_nonoverlapping(
                        virt2 as *const u8,
                        buffer[offset + first_page_bytes..].as_mut_ptr(),
                        remaining_bytes,
                    );
                }
            }
            crate::memory::frame::free_frame(phys2);
        }
        
        crate::memory::frame::free_frame(dma_phys);
        
        lba += chunk as u64;
        offset += chunk_bytes;
        remaining -= chunk;
    }
    
    Ok(())
}

/// Write sectors to NVMe namespace 1.
pub fn write_sectors(start_lba: u64, count: usize, buffer: &[u8]) -> Result<(), &'static str> {
    let mut ctrl = NVME.lock();
    let ctrl = ctrl.as_mut().ok_or("NVMe: not initialized")?;
    
    let lba_sz = ctrl.ns1_lba_size as usize;
    let total_bytes = count * lba_sz;
    
    if buffer.len() < total_bytes {
        return Err("NVMe: buffer too small");
    }
    
    if start_lba + count as u64 > ctrl.ns1_size {
        return Err("NVMe: write past end of namespace");
    }
    
    let max_lbas_per_chunk = (8192 / lba_sz).max(1);
    let mut lba = start_lba;
    let mut offset = 0usize;
    let mut remaining = count;
    
    while remaining > 0 {
        let chunk = remaining.min(max_lbas_per_chunk);
        let chunk_bytes = chunk * lba_sz;
        
        let pages_needed = (chunk_bytes + 4095) / 4096;
        let dma_phys = crate::memory::frame::alloc_frame_zeroed()
            .ok_or("NVMe: OOM for DMA write buffer")?;
        let dma_phys2 = if pages_needed > 1 {
            Some(crate::memory::frame::alloc_frame_zeroed()
                .ok_or("NVMe: OOM for DMA write buffer page 2")?)
        } else {
            None
        };
        
        // Copy data to DMA buffer (first page)
        let first_page_bytes = chunk_bytes.min(4096);
        let dma_virt = crate::memory::phys_to_virt(dma_phys);
        unsafe {
            core::ptr::copy_nonoverlapping(
                buffer[offset..].as_ptr(),
                dma_virt as *mut u8,
                first_page_bytes,
            );
        }
        
        // Copy second page if needed
        if let Some(phys2) = dma_phys2 {
            let remaining_bytes = chunk_bytes - first_page_bytes;
            if remaining_bytes > 0 {
                let virt2 = crate::memory::phys_to_virt(phys2);
                unsafe {
                    core::ptr::copy_nonoverlapping(
                        buffer[offset + first_page_bytes..].as_ptr(),
                        virt2 as *mut u8,
                        remaining_bytes,
                    );
                }
            }
        }
        
        ctrl.write_lbas(lba, chunk as u16, dma_phys)?;
        
        if let Some(phys2) = dma_phys2 {
            crate::memory::frame::free_frame(phys2);
        }
        crate::memory::frame::free_frame(dma_phys);
        
        lba += chunk as u64;
        offset += chunk_bytes;
        remaining -= chunk;
    }
    
    Ok(())
}

/// Flush pending writes
pub fn flush() -> Result<(), &'static str> {
    let mut ctrl = NVME.lock();
    let ctrl = ctrl.as_mut().ok_or("NVMe: not initialized")?;
    ctrl.flush()
}
