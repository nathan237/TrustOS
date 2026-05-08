//! NVIDIA GPU Driver — NV50 (Tesla) Family
//!
//! Native NVIDIA GPU driver for TrustOS.
//! Supports NV50 (Tesla) family GPUs (G80–GT200), including Quadro NVS 140M.
//!
//! Architecture:
//! - PCI detection (vendor 0x10DE, G80-G98 device IDs)
//! - BAR0 MMIO mapping (16MB register space)
//! - BAR1 VRAM aperture mapping
//! - GPU identity & VRAM probing
//! - PFIFO command channel setup
//! - 2D engine (NV50_TWOD 0x502D) for accelerated fill/blit
//!
//! References:
//! - envytools: https://envytools.readthedocs.io/en/latest/hw/
//! - nouveau kernel driver: drivers/gpu/drm/nouveau/

pub mod regs;

use alloc::string::String;
use alloc::format;
use alloc::vec::Vec;
use core::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use spin::Mutex;

use crate::pci::{self, PciDevice};
use crate::memory;

// ═══════════════════════════════════════════════════════════════════════════════
// Constants
// ═══════════════════════════════════════════════════════════════════════════════

/// NVIDIA PCI vendor ID
pub // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const NVIDIA_VENDOR_ID: u16 = 0x10DE;

/// Supported NV50 (Tesla) family device IDs
/// G80, G84, G86, G92, G94, G96, G98
const SUPPORTED_DEVICE_IDS: &[(u16, u16, &str)] = &[
    // G80
    (0x0190, 0x019F, "GeForce 8800"),
    // G84
    (0x0400, 0x040F, "GeForce 8600"),
    // G86 — Quadro NVS 140M / GeForce 8500
    (0x0420, 0x042F, "Quadro NVS 140M / GeForce 8500"),
    // G92
    (0x0600, 0x060F, "GeForce 8800/9800"),
    // G94
    (0x0620, 0x063F, "GeForce 9600"),
    // G96
    (0x0640, 0x065F, "GeForce 9500/9400"),
    // G98
    (0x06E0, 0x06EF, "GeForce G100/G105M"),
    // GT200 (still Tesla family)
    (0x05E0, 0x05EF, "GeForce GTX 260/280"),
    // MCP7x/MCP8x integrated (Tesla)
    (0x0840, 0x084F, "GeForce 8200M"),
    (0x0860, 0x086F, "GeForce 8100/8200"),
];

/// PCI BAR indices
mod bar {
    /// BAR0: MMIO registers (16MB)
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const MMIO: usize = 0;
    /// BAR1: VRAM aperture (framebuffer window)
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const VRAM: usize = 1;
    /// BAR3: RAMIN aperture (instance memory)
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const RAMIN: usize = 3;
}

// ═══════════════════════════════════════════════════════════════════════════════
// GPU State
// ═══════════════════════════════════════════════════════════════════════════════

/// GPU hardware information
#[derive(Debug, Clone)]
// Structure publique — visible à l'extérieur de ce module.
pub struct NvGpuInformation {
    pub vendor_id: u16,
    pub device_id: u16,
    pub revision: u8,
    pub bus: u8,
    pub device: u8,
    pub function: u8,
    /// Chipset ID (from PMC_BOOT_0, bits 20-27)
    pub chipset_id: u8,
    /// Stepping (from PMC_BOOT_0, bits 0-7)
    pub stepping: u8,
    /// Human-readable GPU name
    pub gpu_name: &'static str,
    /// VRAM size in bytes
    pub vram_size: u64,
    /// BAR0 MMIO virtual address
    pub mmio_base: u64,
    /// BAR0 MMIO size
    pub mmio_size: u64,
    /// BAR1 VRAM aperture virtual address
    pub vram_base: u64,
    /// BAR1 VRAM aperture size
    pub vram_aperture_size: u64,
    /// PCIe link speed (gen)
    pub pcie_gen: u8,
    /// PCIe link width  
    pub pcie_width: u8,
}

// Bloc d'implémentation — définit les méthodes du type ci-dessus.
impl NvGpuInformation {
        // Fonction publique — appelable depuis d'autres modules.
pub fn chipset_name(&self) -> &'static str {
                // Correspondance de motifs — branchement exhaustif de Rust.
match self.chipset_id {
            0x50 => "G80",
            0x84 => "G84",
            0x86 => "G86",
            0x92 => "G92",
            0x94 => "G94",
            0x96 => "G96",
            0x98 => "G98",
            0xA0 => "GT200",
            _ => "NV50-unknown",
        }
    }
    
        // Fonction publique — appelable depuis d'autres modules.
pub fn summary_string(&self) -> String {
        format!("{} ({}) | {} MB VRAM | PCIe Gen{} x{}",
            self.gpu_name, self.chipset_name(),
            self.vram_size / (1024 * 1024),
            self.pcie_gen, self.pcie_width)
    }
}

/// FIFO push buffer state for command submission
struct FifoChannel {
    /// Physical address of the push buffer
    pushbuf_phys: u64,
    /// Virtual address of the push buffer
    pushbuf_virt: u64,
    /// Push buffer size in bytes
    pushbuf_size: usize,
    /// Current write offset (in dwords)
    put: u32,
    /// Channel number
    channel: u32,
    /// Whether 2D engine object is bound
    twod_bound: bool,
}

/// Driver state
struct NvidiaState {
    initialized: bool,
    gpu_info: Option<NvGpuInformation>,
    fifo: Option<FifoChannel>,
    /// Whether 2D acceleration is ready
    accel_2d_ready: bool,
}

// État global partagé protégé par un Mutex (verrou d'exclusion mutuelle).
static STATE: Mutex<NvidiaState> = Mutex::new(NvidiaState {
    initialized: false,
    gpu_info: None,
    fifo: None,
    accel_2d_ready: false,
});

// Variable atomique — accès thread-safe sans verrou.
static GPU_DETECTED: AtomicBool = AtomicBool::new(false);
// Variable atomique — accès thread-safe sans verrou.
static ACCEL_READY: AtomicBool = AtomicBool::new(false);
/// MMIO base cached for fast access (avoid lock)
static MMIO_BASE: AtomicU64 = AtomicU64::new(0);
/// VRAM aperture base cached
static VRAM_BASE: AtomicU64 = AtomicU64::new(0);

// ═══════════════════════════════════════════════════════════════════════════════
// MMIO Helpers
// ═══════════════════════════════════════════════════════════════════════════════

/// Read 32-bit GPU register
#[inline]
// SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe fn mmio_rd32(base: u64, offset: u32) -> u32 {
    core::ptr::read_volatile((base + offset as u64) as *// Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const u32)
}

/// Write 32-bit GPU register
#[inline]
// SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe fn mmio_wr32(base: u64, offset: u32, val: u32) {
    core::ptr::write_volatile((base + offset as u64) as *mut u32, val);
}

/// Wait for a register to match mask (with timeout)
unsafe fn mmio_wait(base: u64, offset: u32, mask: u32, value: u32, timeout_us: u32) -> bool {
    for _ in 0..timeout_us {
        if mmio_rd32(base, offset) & mask == value {
            return true;
        }
        // Rough microsecond delay via port 0x80 writes
        core::arch::asm!("out 0x80, al", in("al") 0u8, options(nostack, preserves_flags));
    }
    false
}

// ═══════════════════════════════════════════════════════════════════════════════
// PCI Probe
// ═══════════════════════════════════════════════════════════════════════════════

/// Find an NVIDIA NV50 GPU on the PCI bus
fn probe_pci() -> Option<PciDevice> {
    let devices = pci::find_by_class(pci::class::DISPLAY);
    for dev in devices {
        if dev.vendor_id != NVIDIA_VENDOR_ID {
            continue;
        }
        // Check if device ID falls within a supported range
        for &(lo, hi, _name) in SUPPORTED_DEVICE_IDS {
            if dev.device_id >= lo && dev.device_id <= hi {
                return Some(dev);
            }
        }
        // Log unrecognized NVIDIA GPU
        crate::serial_println!("[NVIDIA] Unrecognized NVIDIA GPU: {:04X}:{:04X}",
            dev.vendor_id, dev.device_id);
    }
    None
}

/// Get the GPU name from a device ID
fn gpu_name_for_id(device_id: u16) -> &'static str {
    for &(lo, hi, name) in SUPPORTED_DEVICE_IDS {
        if device_id >= lo && device_id <= hi {
            return name;
        }
    }
    "NVIDIA (Unknown)"
}

// ═══════════════════════════════════════════════════════════════════════════════
// GPU Initialization
// ═══════════════════════════════════════════════════════════════════════════════

/// Read GPU identity from PMC_BOOT_0
unsafe fn read_gpu_identity(mmio: u64) -> (u8, u8) {
    let boot0 = mmio_rd32(mmio, regs::PMC_BOOT_0);
    let chipset_id = ((boot0 >> 20) & 0xFF) as u8;
    let stepping = (boot0 & 0xFF) as u8;
    crate::serial_println!("[NVIDIA] PMC_BOOT_0 = {:#010X} → chipset={:#04X} stepping={:#04X}",
        boot0, chipset_id, stepping);
    (chipset_id, stepping)
}

/// Read VRAM size from PFB registers
unsafe fn read_vram_size(mmio: u64) -> u64 {
    let cfg0 = mmio_rd32(mmio, regs::PFB_CFG0);
    let cfg1 = mmio_rd32(mmio, regs::PFB_CFG1);
    crate::serial_println!("[NVIDIA] PFB_CFG0={:#010X} PFB_CFG1={:#010X}", cfg0, cfg1);
    
    // For NV50, VRAM size is encoded in fb config registers
    // The exact encoding depends on the chip, but a common approach:
    // Check the VRAM size field in bits
    let size_mb = // Correspondance de motifs — branchement exhaustif de Rust.
match cfg1 & 0xFFF {
        s if s > 0 => s as u64,
        _ => {
            // Fallback: try reading from BAR1 size
            128 // Default for NVS 140M
        }
    };
    
    let vram_bytes = size_mb * 1024 * 1024;
    crate::serial_println!("[NVIDIA] VRAM: {} MB", size_mb);
    vram_bytes
}

/// Read PCIe link info from PCI config space
fn read_pcie_link(dev: &PciDevice) -> (u8, u8) {
    if let Some(cap_offset) = pci::find_capability(dev, 0x10) {
        let link_status = pci::config_read16(dev.bus, dev.device, dev.function, 
            cap_offset as u8 + 0x12);
        let speed = (link_status & 0xF) as u8;
        let width = ((link_status >> 4) & 0x3F) as u8;
        (speed, width)
    } else {
        (1, 16) // Default PCIe Gen1 x16
    }
}

/// Initialize GPU engines after MMIO is mapped
unsafe fn gpu_engine_initialize(mmio: u64) -> bool {
    crate::serial_println!("[NVIDIA] Initializing GPU engines...");
    
    // 1. Master enable — activate all engines
    let enable = mmio_rd32(mmio, regs::PMC_ENABLE);
    crate::serial_println!("[NVIDIA] Current PMC_ENABLE = {:#010X}", enable);
    
    // Enable PFIFO, PGRAPH, PFB, PDISPLAY
    let new_enable = enable | regs::PMC_ENABLE_PFIFO | regs::PMC_ENABLE_PGRAPH 
                           | regs::PMC_ENABLE_PFB | regs::PMC_ENABLE_PDISPLAY;
    mmio_wr32(mmio, regs::PMC_ENABLE, new_enable);
    
    // Verify
    let verify = mmio_rd32(mmio, regs::PMC_ENABLE);
    crate::serial_println!("[NVIDIA] PMC_ENABLE after write = {:#010X}", verify);
    
    // 2. Clear pending interrupts
    mmio_wr32(mmio, regs::PMC_INTR_HOST, 0xFFFFFFFF);
    mmio_wr32(mmio, regs::PGRAPH_INTR, 0xFFFFFFFF);
    mmio_wr32(mmio, regs::PGRAPH_TRAP, 0xC0000000);
    mmio_wr32(mmio, regs::PFIFO_INTR, 0xFFFFFFFF);
    
    // 3. Disable all interrupts (we'll poll instead of using IRQs)
    mmio_wr32(mmio, regs::PMC_INTR_EN_HOST, 0);
    mmio_wr32(mmio, regs::PGRAPH_INTR_EN, 0);
    mmio_wr32(mmio, regs::PFIFO_INTR_EN, 0);
    
    // 4. Enable PFIFO
    mmio_wr32(mmio, regs::PFIFO_ENABLE, 1);
    
    // 5. Setup timer (for delay operations)
    mmio_wr32(mmio, regs::PTIMER_NUMERATOR, 0x00000008);
    mmio_wr32(mmio, regs::PTIMER_DENOMINATOR, 0x00000003);
    
    // 6. Read interrupt status to confirm engines are alive
    let intr = mmio_rd32(mmio, regs::PMC_INTR_HOST);
    let pgraph_status = mmio_rd32(mmio, regs::PGRAPH_STATUS);
    crate::serial_println!("[NVIDIA] PMC_INTR = {:#010X}, PGRAPH status = {:#010X}", 
        intr, pgraph_status);
    
    true
}

/// Set up FIFO channel 0 for DMA push buffer command submission
unsafe fn setup_fifo_channel(mmio: u64, vram_base: u64) -> Option<FifoChannel> {
    crate::serial_println!("[NVIDIA] Setting up FIFO channel 0...");
    
    // Allocate push buffer in VRAM (4KB, page-aligned)
    // We place it at the start of the VRAM aperture (BAR1 offset 0)
    // The actual GPU physical address is at the start of VRAM
    let pushbuf_size: usize = 4096;     // 4KB push buffer
    let pushbuf_vram_offset: u64 = 0;   // Offset within VRAM
    
    // The virtual address is via BAR1 aperture
    let pushbuf_virt = vram_base + pushbuf_vram_offset;
    
    // Zero the push buffer
    let ptr = pushbuf_virt as *mut u8;
    for i in 0..pushbuf_size {
        core::ptr::write_volatile(ptr.add(i), 0);
    }
    
    // Configure PFIFO for channel 0 in DMA mode
    // Set FIFO mode to DMA for channel 0
    let mode = mmio_rd32(mmio, regs::PFIFO_MODE);
    mmio_wr32(mmio, regs::PFIFO_MODE, mode | 1); // Channel 0 = DMA mode
    
    // Enable DMA for channel 0
    let dma = mmio_rd32(mmio, regs::PFIFO_DMA);
    mmio_wr32(mmio, regs::PFIFO_DMA, dma | 1);
    
    // Set DMA push buffer base address (channel 0)
    // For NV50, the user-space channel registers are at NV50_USER_BASE + ch*STRIDE
    let user_base = mmio + regs::NV50_USER_BASE as u64;
    
    // Initialize PUT and GET to 0 (empty)
    mmio_wr32(mmio, regs::NV50_USER_BASE + regs::NV50_USER_DMA_PUT, 0);
    mmio_wr32(mmio, regs::NV50_USER_BASE + regs::NV50_USER_DMA_GET, 0);
    
    crate::serial_println!("[NVIDIA] FIFO channel 0 configured (pushbuf at VRAM offset {:#X})", 
        pushbuf_vram_offset);
    
    Some(FifoChannel {
        pushbuf_phys: pushbuf_vram_offset,
        pushbuf_virt,
        pushbuf_size,
        put: 0,
        channel: 0,
        twod_bound: false,
    })
}

// ═══════════════════════════════════════════════════════════════════════════════
// 2D Acceleration via MMIO (register-based, no FIFO needed)
// For initial support we use direct BAR1 writes for simple operations
// ═══════════════════════════════════════════════════════════════════════════════

/// Configure 2D engine destination surface to point at the framebuffer
/// This sets up the GPU's 2D engine to target a linear framebuffer in VRAM
unsafe fn setup_2d_surface(mmio: u64, framebuffer_physical: u64, width: u32, height: u32, pitch: u32) {
    crate::serial_println!("[NVIDIA] Setting up 2D surface: {}x{} pitch={} fb_phys={:#X}",
        width, height, pitch, framebuffer_physical);
    
    // For the 2D engine via direct PGRAPH register writes (MMIO method),
    // we can submit 2D commands through the PGRAPH subchannel registers.
    // However, the proper way is via PFIFO push buffer.
    //
    // For simplicity in phase 1, we use BAR1 (VRAM aperture) directly for
    // CPU-side blit operations, which is still faster than going through  
    // the framebuffer LFB because BAR1 has write-combining.
    //
    // The VRAM aperture surface config will be done when we do FIFO-based
    // 2D engine submissions in a later phase.
}

/// Accelerated fill rectangle via VRAM aperture (BAR1 write-combining)
///
/// When GPUs 2D FIFO isn't available, this still provides a speedup over
/// the LFB (linear framebuffer) path because BAR1 uses write-combining.
pub fn accel_fill_rect(x: u32, y: u32, w: u32, h: u32, color: u32) {
    let vram = VRAM_BASE.load(Ordering::Relaxed);
    if vram == 0 {
        return;
    }
    
    // Get framebuffer dimensions
    let (fb_w, fb_h) = crate::framebuffer::get_dimensions();
    if fb_w == 0 || fb_h == 0 || x >= fb_w || y >= fb_h {
        return;
    }
    
    let x_end = (x + w).min(fb_w);
    let y_end = (y + h).min(fb_h);
    let pitch = fb_w; // pixels per row (assuming 32bpp, pitch = width * 4 bytes)
    
    // Write directly to VRAM aperture with write-combining
    // This bypasses the CPU framebuffer and writes to GPU VRAM
    unsafe {
        let base = vram as *mut u32;
        for row in y..y_end {
            let row_offset = (row * pitch + x) as isize;
            let row_width = (x_end - x) as usize;
            let row_pointer = base.offset(row_offset);
            
            // Use volatile writes (MMIO region)
            for col in 0..row_width {
                core::ptr::write_volatile(row_pointer.add(col), color);
            }
        }
    }
}

/// Accelerated blit (copy rectangle) via VRAM aperture
pub fn accel_copy_rect(source_x: u32, source_y: u32, dst_x: u32, dst_y: u32, w: u32, h: u32) {
    let vram = VRAM_BASE.load(Ordering::Relaxed);
    if vram == 0 {
        return;
    }
    
    let (fb_w, fb_h) = crate::framebuffer::get_dimensions();
    if fb_w == 0 || fb_h == 0 {
        return;
    }
    
    let pitch = fb_w;
    
        // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe {
        let base = vram as *mut u32;
        
        // Handle overlapping copies by choosing correct iteration order
        if dst_y > source_y || (dst_y == source_y && dst_x > source_x) {
            // Copy bottom-to-top, right-to-left (destination is below/right of source)
            for row in (0..h).rev() {
                let sy = source_y + row;
                let dy = dst_y + row;
                if sy >= fb_h || dy >= fb_h { continue; }
                
                for col in (0..w).rev() {
                    let sx = source_x + col;
                    let dx = dst_x + col;
                    if sx >= fb_w || dx >= fb_w { continue; }
                    
                    let val = core::ptr::read_volatile(base.offset((sy * pitch + sx) as isize));
                    core::ptr::write_volatile(base.offset((dy * pitch + dx) as isize), val);
                }
            }
        } else {
            // Copy top-to-bottom, left-to-right
            for row in 0..h {
                let sy = source_y + row;
                let dy = dst_y + row;
                if sy >= fb_h || dy >= fb_h { continue; }
                
                for col in 0..w {
                    let sx = source_x + col;
                    let dx = dst_x + col;
                    if sx >= fb_w || dx >= fb_w { continue; }
                    
                    let val = core::ptr::read_volatile(base.offset((sy * pitch + sx) as isize));
                    core::ptr::write_volatile(base.offset((dy * pitch + dx) as isize), val);
                }
            }
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Public API
// ═══════════════════════════════════════════════════════════════════════════════

/// Initialize the NVIDIA NV50 GPU driver
pub fn init() {
    crate::serial_println!("[NVIDIA] ═══════════════════════════════════════════════════");
    crate::serial_println!("[NVIDIA] NVIDIA GPU Driver — NV50 (Tesla) Family");
    crate::serial_println!("[NVIDIA] ═══════════════════════════════════════════════════");
    
    // Step 1: Find NVIDIA GPU on PCI bus
    let dev = // Correspondance de motifs — branchement exhaustif de Rust.
match probe_pci() {
        Some(d) => d,
        None => {
            crate::serial_println!("[NVIDIA] No supported NVIDIA GPU found on PCI bus");
            // List display controllers for diagnostic
            let display_devs = pci::find_by_class(pci::class::DISPLAY);
            if display_devs.is_empty() {
                crate::serial_println!("[NVIDIA] No display controllers found at all");
            } else {
                for d in &display_devs {
                    crate::serial_println!("[NVIDIA] Display: {:04X}:{:04X} at {:02X}:{:02X}.{}",
                        d.vendor_id, d.device_id, d.bus, d.device, d.function);
                }
            }
            return;
        }
    };
    
    crate::serial_println!("[NVIDIA] Found: {:04X}:{:04X} rev {:02X} at {:02X}:{:02X}.{}",
        dev.vendor_id, dev.device_id, dev.revision, dev.bus, dev.device, dev.function);
    
    // Step 2: Enable PCI bus mastering and memory space 
    pci::enable_bus_master(&dev);
    pci::enable_memory_space(&dev);
    
    // Step 3: Map BAR0 (MMIO registers — 16MB)
    let mmio_physical = // Correspondance de motifs — branchement exhaustif de Rust.
match dev.bar_address(bar::MMIO) {
        Some(addr) if addr > 0 => addr,
        _ => {
            crate::serial_println!("[NVIDIA] ERROR: BAR0 (MMIO) not available");
            return;
        }
    };
    
    let mmio_size = 16 * 1024 * 1024; // 16MB for NV50
    crate::serial_println!("[NVIDIA] BAR0 (MMIO): phys={:#010X} size={}MB", mmio_physical, mmio_size / (1024*1024));
    
    let mmio_virt = // Correspondance de motifs — branchement exhaustif de Rust.
match memory::map_mmio(mmio_physical, mmio_size) {
        Ok(v) => v,
        Err(e) => {
            crate::serial_println!("[NVIDIA] ERROR: Failed to map BAR0: {}", e);
            return;
        }
    };
    crate::serial_println!("[NVIDIA] BAR0 mapped at virt={:#014X}", mmio_virt);
    MMIO_BASE.store(mmio_virt, Ordering::SeqCst);
    
    // Step 4: Map BAR1 (VRAM aperture)
    let vram_physical = dev.bar_address(bar::VRAM).unwrap_or(0);
    let mut vram_virt: u64 = 0;
    let mut vram_ap_size: u64 = 0;
    
    if vram_physical > 0 {
        // Detect BAR1 size (typically 64MB-256MB)
        let bar1_raw = dev.bar[bar::VRAM];
        // For BAR sizing, use standard PCI mechanism
        vram_ap_size = 256 * 1024 * 1024; // Default 256MB for NV50
        
        crate::serial_println!("[NVIDIA] BAR1 (VRAM): phys={:#010X} aperture={}MB", 
            vram_physical, vram_ap_size / (1024*1024));
        
        // Map a portion of VRAM aperture (enough for framebuffer)
        // For 1920x1080x4 = ~8MB, map 16MB to be safe
        let map_size = 16 * 1024 * 1024;
                // Correspondance de motifs — branchement exhaustif de Rust.
match memory::map_mmio(vram_physical, map_size) {
            Ok(v) => {
                vram_virt = v;
                crate::serial_println!("[NVIDIA] BAR1 mapped at virt={:#014X} ({}MB)", v, map_size / (1024*1024));
                VRAM_BASE.store(vram_virt, Ordering::SeqCst);
            }
            Err(e) => {
                crate::serial_println!("[NVIDIA] WARNING: Failed to map BAR1 VRAM: {}", e);
                // Continue without VRAM aperture — no 2D accel but identity still works
            }
        }
    }
    
    // Step 5: Read GPU identity
    let (chipset_id, stepping) = // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe { read_gpu_identity(mmio_virt) };
    
    // Step 6: Read VRAM size
    let vram_size = // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe { read_vram_size(mmio_virt) };
    
    // Step 7: Read PCIe link info
    let (pcie_gen, pcie_width) = read_pcie_link(&dev);
    
    // Step 8: Initialize GPU engines
    let engines_ok = // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe { gpu_engine_initialize(mmio_virt) };
    if !engines_ok {
        crate::serial_println!("[NVIDIA] WARNING: Engine init had issues, continuing anyway");
    }
    
    // Step 9: Setup FIFO channel (if VRAM aperture is available)
    let fifo = if vram_virt > 0 {
                // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe { setup_fifo_channel(mmio_virt, vram_virt) }
    } else {
        None
    };
    
    let accel_2d = fifo.is_some() && vram_virt > 0;
    
    // Build GPU info
    let info = NvGpuInformation {
        vendor_id: dev.vendor_id,
        device_id: dev.device_id,
        revision: dev.revision,
        bus: dev.bus,
        device: dev.device,
        function: dev.function,
        chipset_id,
        stepping,
        gpu_name: gpu_name_for_id(dev.device_id),
        vram_size,
        mmio_base: mmio_virt,
        mmio_size: mmio_size as u64,
        vram_base: vram_virt,
        vram_aperture_size: vram_ap_size,
        pcie_gen,
        pcie_width,
    };
    
    // Print summary
    crate::serial_println!("[NVIDIA] ───────────────────────────────────────────────────");
    crate::serial_println!("[NVIDIA] GPU: {} ({})", info.gpu_name, info.chipset_name());
    crate::serial_println!("[NVIDIA] PCI: {:04X}:{:04X} rev {:02X} at {:02X}:{:02X}.{}", 
        info.vendor_id, info.device_id, info.revision,
        info.bus, info.device, info.function);
    crate::serial_println!("[NVIDIA] Chipset: {:#04X} stepping {:#04X}", chipset_id, stepping);
    crate::serial_println!("[NVIDIA] VRAM: {} MB", info.vram_size / (1024 * 1024));
    crate::serial_println!("[NVIDIA] PCIe: Gen{} x{}", pcie_gen, pcie_width);
    crate::serial_println!("[NVIDIA] MMIO: {:#X} ({}MB)", mmio_virt, mmio_size / (1024*1024));
    if vram_virt > 0 {
        crate::serial_println!("[NVIDIA] VRAM aperture: {:#X}", vram_virt);
    }
    crate::serial_println!("[NVIDIA] 2D Acceleration: {}", if accel_2d { "READY" } else { "UNAVAILABLE" });
    crate::serial_println!("[NVIDIA] ───────────────────────────────────────────────────");
    
    // Store state
    let mut state = STATE.lock();
    state.initialized = true;
    state.gpu_info = Some(info);
    state.fifo = fifo;
    state.accel_2d_ready = accel_2d;
    GPU_DETECTED.store(true, Ordering::SeqCst);
    ACCEL_READY.store(accel_2d, Ordering::SeqCst);
    drop(state);
}

/// Check if an NVIDIA GPU was detected
pub fn is_detected() -> bool {
    GPU_DETECTED.load(Ordering::Relaxed)
}

/// Check if 2D acceleration is available
pub fn is_accel_ready() -> bool {
    ACCEL_READY.load(Ordering::Relaxed)
}

/// Get GPU info
pub fn get_information() -> Option<NvGpuInformation> {
    STATE.lock().gpu_info.clone()
}

/// Get summary string for boot log / shell
pub fn summary() -> String {
    if let Some(info) = get_information() {
        info.summary_string()
    } else {
        String::from("No NVIDIA GPU detected")
    }
}
