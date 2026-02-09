//! VirtIO GPU Driver — Real Hardware Implementation
//!
//! Implements VirtIO GPU 2D acceleration using the modern PCI transport.
//! Provides DMA-based framebuffer transfers for efficient display updates.
//!
//! Architecture:
//! - CPU renders to a backing buffer (same as before)
//! - VirtIO GPU transfers the buffer to the host GPU via DMA
//! - Host GPU displays the resource on a scanout
//!
//! Reference: https://docs.oasis-open.org/virtio/virtio/v1.2/virtio-v1.2.html

use alloc::vec::Vec;
use alloc::vec;
use alloc::boxed::Box;
use core::sync::atomic::{AtomicBool, Ordering};
use spin::Mutex;

use crate::pci::{self, PciDevice};
use crate::memory;

// ═══════════════════════════════════════════════════════════════════════════════
// VirtIO GPU Constants
// ═══════════════════════════════════════════════════════════════════════════════

/// VirtIO GPU PCI device ID (modern, VirtIO 1.0+)
pub const VIRTIO_GPU_PCI_DEVICE_ID: u16 = 0x1050;
/// VirtIO vendor ID
pub const VIRTIO_VENDOR_ID: u16 = 0x1AF4;

/// VirtIO PCI capability types
pub mod virtio_cap {
    pub const COMMON_CFG: u8 = 1;
    pub const NOTIFY_CFG: u8 = 2;
    pub const ISR_CFG: u8 = 3;
    pub const DEVICE_CFG: u8 = 4;
    pub const _PCI_CFG: u8 = 5;
}

/// VirtIO device status bits
pub mod dev_status {
    pub const ACKNOWLEDGE: u8 = 1;
    pub const DRIVER: u8 = 2;
    pub const DRIVER_OK: u8 = 4;
    pub const FEATURES_OK: u8 = 8;
    pub const _DEVICE_NEEDS_RESET: u8 = 64;
    pub const FAILED: u8 = 128;
}

/// VirtIO GPU feature bits
pub mod features {
    pub const _VIRTIO_GPU_F_VIRGL: u64 = 1 << 0;
    pub const VIRTIO_GPU_F_EDID: u64 = 1 << 1;
    pub const _VIRTIO_GPU_F_RESOURCE_UUID: u64 = 1 << 2;
    pub const _VIRTIO_GPU_F_RESOURCE_BLOB: u64 = 1 << 3;
    pub const VIRTIO_F_VERSION_1: u64 = 1 << 32;
}

/// VirtIO GPU control command types
#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq)]
#[allow(dead_code)]
pub enum GpuCtrlType {
    // 2D commands
    CmdGetDisplayInfo = 0x0100,
    CmdResourceCreate2d = 0x0101,
    CmdResourceUnref = 0x0102,
    CmdSetScanout = 0x0103,
    CmdResourceFlush = 0x0104,
    CmdTransferToHost2d = 0x0105,
    CmdResourceAttachBacking = 0x0106,
    CmdResourceDetachBacking = 0x0107,
    CmdGetCapsetInfo = 0x0108,
    CmdGetCapset = 0x0109,
    CmdGetEdid = 0x010a,

    // Cursor commands
    CmdUpdateCursor = 0x0300,
    CmdMoveCursor = 0x0301,

    // Success responses
    RespOkNodata = 0x1100,
    RespOkDisplayInfo = 0x1101,
    RespOkCapsetInfo = 0x1102,
    RespOkCapset = 0x1103,
    RespOkEdid = 0x1104,

    // Error responses
    RespErrUnspec = 0x1200,
    RespErrOutOfMemory = 0x1201,
    RespErrInvalidScanoutId = 0x1202,
    RespErrInvalidResourceId = 0x1203,
    RespErrInvalidContextId = 0x1204,
    RespErrInvalidParameter = 0x1205,
}

/// Pixel formats
#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq)]
#[allow(dead_code)]
pub enum GpuFormat {
    B8G8R8A8Unorm = 1,
    B8G8R8X8Unorm = 2,
    A8R8G8B8Unorm = 3,
    X8R8G8B8Unorm = 4,
    R8G8B8A8Unorm = 67,
    X8B8G8R8Unorm = 68,
    A8B8G8R8Unorm = 121,
    R8G8B8X8Unorm = 134,
}

// ═══════════════════════════════════════════════════════════════════════════════
// VirtIO GPU Protocol Structures (all repr(C) for DMA)
// ═══════════════════════════════════════════════════════════════════════════════

/// Control header — prefix for all commands and responses
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct GpuCtrlHdr {
    pub ctrl_type: u32,
    pub flags: u32,
    pub fence_id: u64,
    pub ctx_id: u32,
    pub ring_idx: u8,
    pub padding: [u8; 3],
}

/// Rectangle
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct GpuRect {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

/// Display info for one scanout
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct GpuDisplayOne {
    pub r: GpuRect,
    pub enabled: u32,
    pub flags: u32,
}

/// Response to CMD_GET_DISPLAY_INFO
#[repr(C)]
#[derive(Clone, Copy)]
pub struct GpuRespDisplayInfo {
    pub hdr: GpuCtrlHdr,
    pub pmodes: [GpuDisplayOne; 16],
}

/// CMD_RESOURCE_CREATE_2D
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct GpuResourceCreate2d {
    pub hdr: GpuCtrlHdr,
    pub resource_id: u32,
    pub format: u32,
    pub width: u32,
    pub height: u32,
}

/// CMD_SET_SCANOUT
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct GpuSetScanout {
    pub hdr: GpuCtrlHdr,
    pub r: GpuRect,
    pub scanout_id: u32,
    pub resource_id: u32,
}

/// CMD_RESOURCE_FLUSH
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct GpuResourceFlush {
    pub hdr: GpuCtrlHdr,
    pub r: GpuRect,
    pub resource_id: u32,
    pub padding: u32,
}

/// CMD_TRANSFER_TO_HOST_2D
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct GpuTransferToHost2d {
    pub hdr: GpuCtrlHdr,
    pub r: GpuRect,
    pub offset: u64,
    pub resource_id: u32,
    pub padding: u32,
}

/// Memory entry for resource backing
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct GpuMemEntry {
    pub addr: u64,
    pub length: u32,
    pub padding: u32,
}

/// CMD_RESOURCE_ATTACH_BACKING
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct GpuResourceAttachBacking {
    pub hdr: GpuCtrlHdr,
    pub resource_id: u32,
    pub nr_entries: u32,
}

// ═══════════════════════════════════════════════════════════════════════════════
// VirtIO Ring (local implementation for GPU)
// ═══════════════════════════════════════════════════════════════════════════════

#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
struct VirtqDesc {
    addr: u64,
    len: u32,
    flags: u16,
    next: u16,
}

const VIRTQ_DESC_F_NEXT: u16 = 1;
const VIRTQ_DESC_F_WRITE: u16 = 2;

#[repr(C)]
struct VirtqAvail {
    flags: u16,
    idx: u16,
}

#[repr(C)]
#[derive(Clone, Copy, Default)]
struct VirtqUsedElem {
    id: u32,
    len: u32,
}

#[repr(C)]
struct VirtqUsed {
    flags: u16,
    idx: u16,
}

/// GPU virtqueue
struct GpuVirtqueue {
    size: u16,
    _phys_base: u64,
    _virt_base: *mut u8,
    desc: *mut VirtqDesc,
    avail: *mut VirtqAvail,
    used: *mut VirtqUsed,
    free_head: u16,
    num_free: u16,
    free_list: Vec<u16>,
    last_used_idx: u16,
}

unsafe impl Send for GpuVirtqueue {}
unsafe impl Sync for GpuVirtqueue {}

impl GpuVirtqueue {
    fn new(size: u16) -> Result<Self, &'static str> {
        use alloc::alloc::{alloc_zeroed, Layout};
        
        let desc_bytes = core::mem::size_of::<VirtqDesc>() * size as usize;
        let avail_bytes = 6 + 2 * size as usize;
        let used_offset = ((desc_bytes + avail_bytes) + 4095) & !4095;
        let used_bytes = 6 + core::mem::size_of::<VirtqUsedElem>() * size as usize;
        let total_size = used_offset + used_bytes + 4096;
        
        let layout = Layout::from_size_align(total_size, 4096)
            .map_err(|_| "Invalid virtqueue layout")?;
        let ptr = unsafe { alloc_zeroed(layout) };
        if ptr.is_null() { return Err("Failed to allocate virtqueue"); }
        
        let virt_addr = ptr as u64;
        let hhdm = memory::hhdm_offset();
        let phys_addr = if virt_addr >= hhdm { virt_addr - hhdm } else { virt_addr };
        
        let desc = ptr as *mut VirtqDesc;
        let avail = unsafe { ptr.add(desc_bytes) as *mut VirtqAvail };
        let used = unsafe { ptr.add(used_offset) as *mut VirtqUsed };
        
        let mut free_list = vec![0u16; size as usize];
        for i in 0..(size as usize).saturating_sub(1) {
            free_list[i] = (i + 1) as u16;
        }
        if size > 0 { free_list[size as usize - 1] = 0xFFFF; }
        
        Ok(Self {
            size,
            _phys_base: phys_addr,
            _virt_base: ptr,
            desc,
            avail,
            used,
            free_head: 0,
            num_free: size,
            free_list,
            last_used_idx: 0,
        })
    }
    
    fn alloc_desc(&mut self) -> Option<u16> {
        if self.num_free == 0 { return None; }
        let idx = self.free_head;
        self.free_head = self.free_list[idx as usize];
        self.num_free -= 1;
        Some(idx)
    }
    
    fn free_desc(&mut self, idx: u16) {
        self.free_list[idx as usize] = self.free_head;
        self.free_head = idx;
        self.num_free += 1;
    }
    
    fn set_desc(&mut self, idx: u16, addr: u64, len: u32, flags: u16, next: u16) {
        unsafe {
            let d = &mut *self.desc.add(idx as usize);
            d.addr = addr;
            d.len = len;
            d.flags = flags;
            d.next = next;
        }
    }
    
    fn submit(&mut self, head: u16) {
        unsafe {
            let avail = &mut *self.avail;
            let ring_ptr = (self.avail as *mut u8).add(4) as *mut u16;
            let idx = avail.idx;
            *ring_ptr.add((idx % self.size) as usize) = head;
            core::sync::atomic::fence(Ordering::Release);
            avail.idx = idx.wrapping_add(1);
        }
    }
    
    fn poll_used(&mut self) -> Option<(u32, u32)> {
        unsafe {
            core::sync::atomic::fence(Ordering::Acquire);
            let used = &*self.used;
            if used.idx == self.last_used_idx { return None; }
            let ring_ptr = (self.used as *mut u8).add(4) as *mut VirtqUsedElem;
            let elem = *ring_ptr.add((self.last_used_idx % self.size) as usize);
            self.last_used_idx = self.last_used_idx.wrapping_add(1);
            Some((elem.id, elem.len))
        }
    }
    
    fn desc_phys(&self) -> u64 { self._phys_base }
    fn avail_phys(&self) -> u64 {
        let desc_bytes = core::mem::size_of::<VirtqDesc>() * self.size as usize;
        self._phys_base + desc_bytes as u64
    }
    fn used_phys(&self) -> u64 {
        let desc_bytes = core::mem::size_of::<VirtqDesc>() * self.size as usize;
        let avail_bytes = 6 + 2 * self.size as usize;
        let used_offset = ((desc_bytes + avail_bytes) + 4095) & !4095;
        self._phys_base + used_offset as u64
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// MMIO Region Accessor
// ═══════════════════════════════════════════════════════════════════════════════

struct MmioRegion {
    base: *mut u8,
    _len: u32,
}

unsafe impl Send for MmioRegion {}
unsafe impl Sync for MmioRegion {}

impl MmioRegion {
    fn read8(&self, offset: u32) -> u8 {
        unsafe { core::ptr::read_volatile(self.base.add(offset as usize)) }
    }
    fn read16(&self, offset: u32) -> u16 {
        unsafe { core::ptr::read_volatile(self.base.add(offset as usize) as *const u16) }
    }
    fn read32(&self, offset: u32) -> u32 {
        unsafe { core::ptr::read_volatile(self.base.add(offset as usize) as *const u32) }
    }
    fn write8(&self, offset: u32, val: u8) {
        unsafe { core::ptr::write_volatile(self.base.add(offset as usize), val) }
    }
    fn write16(&self, offset: u32, val: u16) {
        unsafe { core::ptr::write_volatile(self.base.add(offset as usize) as *mut u16, val) }
    }
    fn write32(&self, offset: u32, val: u32) {
        unsafe { core::ptr::write_volatile(self.base.add(offset as usize) as *mut u32, val) }
    }
    fn write64(&self, offset: u32, val: u64) {
        self.write32(offset, val as u32);
        self.write32(offset + 4, (val >> 32) as u32);
    }
}

/// Common config offsets (VirtIO PCI modern)
mod common_cfg {
    pub const DEVICE_FEATURE_SELECT: u32 = 0x00;
    pub const DEVICE_FEATURE: u32 = 0x04;
    pub const DRIVER_FEATURE_SELECT: u32 = 0x08;
    pub const DRIVER_FEATURE: u32 = 0x0C;
    pub const _MSIX_CONFIG: u32 = 0x10;
    pub const _NUM_QUEUES: u32 = 0x12;
    pub const DEVICE_STATUS: u32 = 0x14;
    pub const _CONFIG_GENERATION: u32 = 0x15;
    pub const QUEUE_SELECT: u32 = 0x16;
    pub const QUEUE_SIZE: u32 = 0x18;
    pub const QUEUE_MSIX_VECTOR: u32 = 0x1A;
    pub const QUEUE_ENABLE: u32 = 0x1C;
    pub const QUEUE_NOTIFY_OFF: u32 = 0x1E;
    pub const QUEUE_DESC: u32 = 0x20;
    pub const QUEUE_DRIVER: u32 = 0x28;
    pub const QUEUE_DEVICE: u32 = 0x30;
}

/// GPU device config offsets
mod gpu_cfg {
    pub const _EVENTS_READ: u32 = 0x00;
    pub const _EVENTS_CLEAR: u32 = 0x04;
    pub const NUM_SCANOUTS: u32 = 0x08;
    pub const NUM_CAPSETS: u32 = 0x0C;
}

// ═══════════════════════════════════════════════════════════════════════════════
// DMA Command Buffer
// ═══════════════════════════════════════════════════════════════════════════════

struct DmaCommandBuffer {
    phys: u64,
    virt: *mut u8,
    _size: usize,
}

unsafe impl Send for DmaCommandBuffer {}
unsafe impl Sync for DmaCommandBuffer {}

impl DmaCommandBuffer {
    fn new(size: usize) -> Result<Self, &'static str> {
        use alloc::alloc::{alloc_zeroed, Layout};
        let layout = Layout::from_size_align(size, 4096)
            .map_err(|_| "DMA buffer layout error")?;
        let ptr = unsafe { alloc_zeroed(layout) };
        if ptr.is_null() { return Err("DMA buffer allocation failed"); }
        let virt = ptr as u64;
        let hhdm = memory::hhdm_offset();
        let phys = if virt >= hhdm { virt - hhdm } else { virt };
        Ok(Self { phys, virt: ptr, _size: size })
    }
    
    unsafe fn write_at<T: Copy>(&self, offset: usize, val: &T) {
        core::ptr::write_volatile(self.virt.add(offset) as *mut T, *val);
    }
    
    unsafe fn read_at<T: Copy>(&self, offset: usize) -> T {
        core::ptr::read_volatile(self.virt.add(offset) as *const T)
    }
    
    fn phys_at(&self, offset: usize) -> u64 { self.phys + offset as u64 }
}

// ═══════════════════════════════════════════════════════════════════════════════
// GPU Surface (compatibility API)
// ═══════════════════════════════════════════════════════════════════════════════

/// GPU Surface for 2D operations
pub struct GpuSurface {
    pub resource_id: u32,
    pub width: u32,
    pub height: u32,
    pub data: Box<[u32]>,
}

impl GpuSurface {
    pub fn new(width: u32, height: u32) -> Self {
        let size = (width * height) as usize;
        Self {
            resource_id: 0,
            width,
            height,
            data: alloc::vec![0u32; size].into_boxed_slice(),
        }
    }
    
    pub fn clear(&mut self, color: u32) { self.data.fill(color); }
    
    #[inline]
    pub fn set_pixel(&mut self, x: u32, y: u32, color: u32) {
        if x < self.width && y < self.height {
            self.data[(y * self.width + x) as usize] = color;
        }
    }
    
    #[inline]
    pub fn get_pixel(&self, x: u32, y: u32) -> u32 {
        if x < self.width && y < self.height {
            self.data[(y * self.width + x) as usize]
        } else { 0 }
    }
    
    pub fn fill_rect(&mut self, x: u32, y: u32, w: u32, h: u32, color: u32) {
        let x1 = x.min(self.width);
        let y1 = y.min(self.height);
        let x2 = (x + w).min(self.width);
        let y2 = (y + h).min(self.height);
        for py in y1..y2 {
            let start = (py * self.width + x1) as usize;
            let end = (py * self.width + x2) as usize;
            self.data[start..end].fill(color);
        }
    }
    
    pub fn blit(&mut self, src: &GpuSurface, dst_x: i32, dst_y: i32) {
        for sy in 0..src.height {
            for sx in 0..src.width {
                let dx = dst_x + sx as i32;
                let dy = dst_y + sy as i32;
                if dx >= 0 && dy >= 0 && dx < self.width as i32 && dy < self.height as i32 {
                    let pixel = src.get_pixel(sx, sy);
                    let alpha = (pixel >> 24) & 0xFF;
                    if alpha >= 128 {
                        self.set_pixel(dx as u32, dy as u32, pixel);
                    }
                }
            }
        }
    }
    
    fn set_pixel_safe(&mut self, x: i32, y: i32, color: u32) {
        if x >= 0 && y >= 0 && x < self.width as i32 && y < self.height as i32 {
            self.set_pixel(x as u32, y as u32, color);
        }
    }

    pub fn draw_line(&mut self, x0: i32, y0: i32, x1: i32, y1: i32, color: u32) {
        let dx = (x1 - x0).abs();
        let dy = -(y1 - y0).abs();
        let sx = if x0 < x1 { 1 } else { -1 };
        let sy = if y0 < y1 { 1 } else { -1 };
        let mut err = dx + dy;
        let (mut x, mut y) = (x0, y0);
        loop {
            self.set_pixel_safe(x, y, color);
            if x == x1 && y == y1 { break; }
            let e2 = 2 * err;
            if e2 >= dy { err += dy; x += sx; }
            if e2 <= dx { err += dx; y += sy; }
        }
    }

    pub fn draw_rect(&mut self, x: u32, y: u32, w: u32, h: u32, color: u32) {
        let (x, y, w, h) = (x as i32, y as i32, w as i32, h as i32);
        self.draw_line(x, y, x+w-1, y, color);
        self.draw_line(x, y+h-1, x+w-1, y+h-1, color);
        self.draw_line(x, y, x, y+h-1, color);
        self.draw_line(x+w-1, y, x+w-1, y+h-1, color);
    }

    pub fn draw_circle(&mut self, cx: i32, cy: i32, radius: i32, color: u32) {
        let (mut x, mut y, mut err) = (radius, 0i32, 0i32);
        while x >= y {
            self.set_pixel_safe(cx+x, cy+y, color);
            self.set_pixel_safe(cx+y, cy+x, color);
            self.set_pixel_safe(cx-y, cy+x, color);
            self.set_pixel_safe(cx-x, cy+y, color);
            self.set_pixel_safe(cx-x, cy-y, color);
            self.set_pixel_safe(cx-y, cy-x, color);
            self.set_pixel_safe(cx+y, cy-x, color);
            self.set_pixel_safe(cx+x, cy-y, color);
            y += 1;
            err += 1 + 2*y;
            if 2*(err-x)+1 > 0 { x -= 1; err += 1 - 2*x; }
        }
    }

    pub fn fill_circle(&mut self, cx: i32, cy: i32, radius: i32, color: u32) {
        for dy in -radius..=radius {
            for dx in -radius..=radius {
                if dx*dx + dy*dy <= radius*radius {
                    self.set_pixel_safe(cx+dx, cy+dy, color);
                }
            }
        }
    }

    pub fn draw_rounded_rect(&mut self, x: u32, y: u32, w: u32, h: u32, _radius: u32, color: u32) {
        self.draw_rect(x, y, w, h, color);
    }

    pub fn fill_rounded_rect(&mut self, x: u32, y: u32, w: u32, h: u32, _radius: u32, color: u32) {
        self.fill_rect(x, y, w, h, color);
    }

    pub fn blit_scaled(&mut self, src: &GpuSurface, dst_x: i32, dst_y: i32, dst_w: u32, dst_h: u32) {
        if dst_w == 0 || dst_h == 0 || src.width == 0 || src.height == 0 { return; }
        for dy in 0..dst_h {
            for dx in 0..dst_w {
                let sx = (dx * src.width) / dst_w;
                let sy = (dy * src.height) / dst_h;
                let px = dst_x + dx as i32;
                let py = dst_y + dy as i32;
                if px >= 0 && py >= 0 && px < self.width as i32 && py < self.height as i32 {
                    self.set_pixel(px as u32, py as u32, src.get_pixel(sx, sy));
                }
            }
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// VirtIO GPU Driver
// ═══════════════════════════════════════════════════════════════════════════════

pub struct VirtioGpu {
    _pci_dev: Option<PciDevice>,
    common_cfg: Option<MmioRegion>,
    notify_cfg: Option<MmioRegion>,
    _isr_cfg: Option<MmioRegion>,
    device_cfg: Option<MmioRegion>,
    _notify_off_multiplier: u32,
    controlq: Option<GpuVirtqueue>,
    dma_buf: Option<DmaCommandBuffer>,
    display_width: u32,
    display_height: u32,
    num_scanouts: u32,
    next_resource_id: u32,
    scanout_resource_id: u32,
    backing_buffer: Option<Box<[u32]>>,
    backing_phys: u64,
    initialized: bool,
    has_3d: bool,
}

impl VirtioGpu {
    pub const fn new() -> Self {
        Self {
            _pci_dev: None,
            common_cfg: None,
            notify_cfg: None,
            _isr_cfg: None,
            device_cfg: None,
            _notify_off_multiplier: 0,
            controlq: None,
            dma_buf: None,
            display_width: 0,
            display_height: 0,
            num_scanouts: 0,
            next_resource_id: 1,
            scanout_resource_id: 0,
            backing_buffer: None,
            backing_phys: 0,
            initialized: false,
            has_3d: false,
        }
    }
    
    fn map_bar_region(dev: &PciDevice, bar_idx: u8, offset: u32, length: u32) -> Result<MmioRegion, &'static str> {
        let bar_addr = dev.bar_address(bar_idx as usize)
            .ok_or("BAR not configured")?;
        if !dev.bar_is_memory(bar_idx as usize) {
            return Err("Expected memory BAR, got I/O");
        }
        let phys = bar_addr + offset as u64;
        let virt = memory::map_mmio(phys, length.max(4096) as usize)?;
        crate::serial_println!("[VIRTIO-GPU] Mapped BAR{}: phys={:#X} virt={:#X} len={}", 
            bar_idx, phys, virt, length);
        Ok(MmioRegion { base: virt as *mut u8, _len: length })
    }
    
    /// Initialize from PCI device
    pub fn init(&mut self, dev: PciDevice) -> Result<(), &'static str> {
        crate::serial_println!("[VIRTIO-GPU] === Initializing VirtIO GPU ===");
        crate::serial_println!("[VIRTIO-GPU] PCI {:02X}:{:02X}.{} vid={:#06X} did={:#06X}",
            dev.bus, dev.device, dev.function, dev.vendor_id, dev.device_id);
        
        pci::enable_bus_master(&dev);
        pci::enable_memory_space(&dev);
        
        // Find VirtIO PCI capabilities (modern transport)
        let caps = pci::find_virtio_capabilities(&dev);
        if caps.is_empty() {
            return Err("No VirtIO capabilities found");
        }
        
        crate::serial_println!("[VIRTIO-GPU] Found {} VirtIO capabilities", caps.len());
        
        let mut notify_cap_offset: u8 = 0;
        
        for &(cap_off, cfg_type, bar, offset, length) in &caps {
            let name = match cfg_type {
                1 => "COMMON", 2 => "NOTIFY", 3 => "ISR", 4 => "DEVICE", 5 => "PCI", _ => "?",
            };
            crate::serial_println!("[VIRTIO-GPU]   cap@{:#X}: {} BAR{} off={:#X} len={}", 
                cap_off, name, bar, offset, length);
            
            match cfg_type {
                virtio_cap::COMMON_CFG => {
                    self.common_cfg = Some(Self::map_bar_region(&dev, bar, offset, length)?);
                }
                virtio_cap::NOTIFY_CFG => {
                    self.notify_cfg = Some(Self::map_bar_region(&dev, bar, offset, length)?);
                    notify_cap_offset = cap_off;
                }
                virtio_cap::ISR_CFG => {
                    self._isr_cfg = Some(Self::map_bar_region(&dev, bar, offset, length)?);
                }
                virtio_cap::DEVICE_CFG => {
                    self.device_cfg = Some(Self::map_bar_region(&dev, bar, offset, length)?);
                }
                _ => {}
            }
        }
        
        // Verify required capabilities are present
        if self.common_cfg.is_none() { return Err("Missing COMMON_CFG"); }
        if self.notify_cfg.is_none() { return Err("Missing NOTIFY_CFG"); }
        if self.device_cfg.is_none() { return Err("Missing DEVICE_CFG"); }
        
        if notify_cap_offset > 0 {
            self._notify_off_multiplier = pci::read_notify_off_multiplier(&dev, notify_cap_offset);
        }
        
        // === VirtIO 1.0 initialization handshake ===
        // Use inline closure to access common_cfg without borrowing self
        
        // 1. Reset
        self.common_write8(common_cfg::DEVICE_STATUS, 0);
        for _ in 0..10000 { core::hint::spin_loop(); }
        
        // 2. ACKNOWLEDGE
        self.common_write8(common_cfg::DEVICE_STATUS, dev_status::ACKNOWLEDGE);
        
        // 3. DRIVER
        self.common_write8(common_cfg::DEVICE_STATUS, dev_status::ACKNOWLEDGE | dev_status::DRIVER);
        
        // 4. Feature negotiation
        self.common_write32(common_cfg::DEVICE_FEATURE_SELECT, 0);
        let feat_lo = self.common_read32(common_cfg::DEVICE_FEATURE);
        self.common_write32(common_cfg::DEVICE_FEATURE_SELECT, 1);
        let feat_hi = self.common_read32(common_cfg::DEVICE_FEATURE);
        let device_features = (feat_lo as u64) | ((feat_hi as u64) << 32);
        
        crate::serial_println!("[VIRTIO-GPU] Device features: {:#018X}", device_features);
        self.has_3d = device_features & features::_VIRTIO_GPU_F_VIRGL != 0;
        
        let mut driver_features = features::VIRTIO_F_VERSION_1;
        if device_features & features::VIRTIO_GPU_F_EDID != 0 {
            driver_features |= features::VIRTIO_GPU_F_EDID;
        }
        
        self.common_write32(common_cfg::DRIVER_FEATURE_SELECT, 0);
        self.common_write32(common_cfg::DRIVER_FEATURE, driver_features as u32);
        self.common_write32(common_cfg::DRIVER_FEATURE_SELECT, 1);
        self.common_write32(common_cfg::DRIVER_FEATURE, (driver_features >> 32) as u32);
        
        // 5. FEATURES_OK
        self.common_write8(common_cfg::DEVICE_STATUS,
            dev_status::ACKNOWLEDGE | dev_status::DRIVER | dev_status::FEATURES_OK);
        
        let status = self.common_read8(common_cfg::DEVICE_STATUS);
        if status & dev_status::FEATURES_OK == 0 {
            self.common_write8(common_cfg::DEVICE_STATUS, dev_status::FAILED);
            return Err("Device rejected features");
        }
        crate::serial_println!("[VIRTIO-GPU] Features OK (3D={})", self.has_3d);
        
        // 6. Setup controlq (queue 0)
        self.setup_controlq()?;
        
        // 7. DRIVER_OK
        self.common_write8(common_cfg::DEVICE_STATUS,
            dev_status::ACKNOWLEDGE | dev_status::DRIVER | dev_status::FEATURES_OK | dev_status::DRIVER_OK);
        crate::serial_println!("[VIRTIO-GPU] DRIVER_OK set");
        
        // 8. DMA command buffer
        self.dma_buf = Some(DmaCommandBuffer::new(8192)?);
        
        // 9. Read GPU config
        self.num_scanouts = self.device_read32(gpu_cfg::NUM_SCANOUTS);
        let num_capsets = self.device_read32(gpu_cfg::NUM_CAPSETS);
        crate::serial_println!("[VIRTIO-GPU] scanouts={} capsets={}", self.num_scanouts, num_capsets);
        
        // 10. Get display info
        self.get_display_info()?;
        
        self._pci_dev = Some(dev);
        self.initialized = true;
        
        crate::serial_println!("[VIRTIO-GPU] === Init complete: {}x{} ===", 
            self.display_width, self.display_height);
        Ok(())
    }
    
    // Helper MMIO accessors that borrow only the specific field
    fn common_write8(&self, offset: u32, val: u8) {
        if let Some(c) = &self.common_cfg { c.write8(offset, val); }
    }
    fn common_write16(&self, offset: u32, val: u16) {
        if let Some(c) = &self.common_cfg { c.write16(offset, val); }
    }
    fn common_write32(&self, offset: u32, val: u32) {
        if let Some(c) = &self.common_cfg { c.write32(offset, val); }
    }
    fn common_read8(&self, offset: u32) -> u8 {
        self.common_cfg.as_ref().map(|c| c.read8(offset)).unwrap_or(0)
    }
    fn common_read16(&self, offset: u32) -> u16 {
        self.common_cfg.as_ref().map(|c| c.read16(offset)).unwrap_or(0)
    }
    fn common_read32(&self, offset: u32) -> u32 {
        self.common_cfg.as_ref().map(|c| c.read32(offset)).unwrap_or(0)
    }
    fn device_read32(&self, offset: u32) -> u32 {
        self.device_cfg.as_ref().map(|c| c.read32(offset)).unwrap_or(0)
    }
    
    fn setup_controlq(&mut self) -> Result<(), &'static str> {
        let common = self.common_cfg.as_ref().ok_or("Missing COMMON_CFG")?;
        common.write16(common_cfg::QUEUE_SELECT, 0);
        let max_size = common.read16(common_cfg::QUEUE_SIZE);
        crate::serial_println!("[VIRTIO-GPU] controlq max_size={}", max_size);
        if max_size == 0 { return Err("controlq not available"); }
        
        let queue_size = max_size.min(64);
        common.write16(common_cfg::QUEUE_SIZE, queue_size);
        
        let vq = GpuVirtqueue::new(queue_size)?;
        
        common.write64(common_cfg::QUEUE_DESC, vq.desc_phys());
        common.write64(common_cfg::QUEUE_DRIVER, vq.avail_phys());
        common.write64(common_cfg::QUEUE_DEVICE, vq.used_phys());
        common.write16(common_cfg::QUEUE_MSIX_VECTOR, 0xFFFF);
        common.write16(common_cfg::QUEUE_ENABLE, 1);
        
        let _notify_off = common.read16(common_cfg::QUEUE_NOTIFY_OFF);
        
        self.controlq = Some(vq);
        crate::serial_println!("[VIRTIO-GPU] controlq ready (size={})", queue_size);
        Ok(())
    }
    
    fn notify_controlq(&self) {
        if let Some(notify) = &self.notify_cfg {
            notify.write16(0, 0);
        }
    }
    
    /// Send command + wait for response (synchronous)
    fn send_command(&mut self, cmd_len: u32, resp_offset: usize, resp_len: u32) -> Result<u32, &'static str> {
        // Extract DMA phys base before mutable controlq borrow
        let dma_phys_base = self.dma_buf.as_ref().ok_or("DMA not ready")?.phys;
        
        let controlq = self.controlq.as_mut().ok_or("controlq not ready")?;
        
        let d_cmd = controlq.alloc_desc().ok_or("No free desc (cmd)")?;
        let d_resp = controlq.alloc_desc().ok_or("No free desc (resp)")?;
        
        controlq.set_desc(d_cmd, dma_phys_base, cmd_len, VIRTQ_DESC_F_NEXT, d_resp);
        controlq.set_desc(d_resp, dma_phys_base + resp_offset as u64, resp_len, VIRTQ_DESC_F_WRITE, 0);
        
        controlq.submit(d_cmd);
        // Notify inline (avoid re-borrowing self)
        if let Some(notify) = &self.notify_cfg {
            notify.write16(0, 0);
        }
        
        let mut timeout = 5_000_000u32;
        loop {
            if let Some(_) = controlq.poll_used() { break; }
            timeout -= 1;
            if timeout == 0 {
                controlq.free_desc(d_resp);
                controlq.free_desc(d_cmd);
                return Err("Command timeout");
            }
            core::hint::spin_loop();
        }
        
        let dma = self.dma_buf.as_ref().unwrap();
        let resp_type = unsafe { dma.read_at::<GpuCtrlHdr>(resp_offset) }.ctrl_type;
        controlq.free_desc(d_resp);
        controlq.free_desc(d_cmd);
        Ok(resp_type)
    }
    
    fn get_display_info(&mut self) -> Result<(), &'static str> {
        let dma = self.dma_buf.as_ref().ok_or("DMA not ready")?;
        
        let cmd = GpuCtrlHdr {
            ctrl_type: GpuCtrlType::CmdGetDisplayInfo as u32,
            ..Default::default()
        };
        unsafe { dma.write_at(0, &cmd); }
        
        let resp_type = self.send_command(
            core::mem::size_of::<GpuCtrlHdr>() as u32,
            512, // response offset
            core::mem::size_of::<GpuRespDisplayInfo>() as u32,
        )?;
        
        if resp_type != GpuCtrlType::RespOkDisplayInfo as u32 {
            crate::serial_println!("[VIRTIO-GPU] GET_DISPLAY_INFO failed: {:#X}", resp_type);
            return Err("GET_DISPLAY_INFO failed");
        }
        
        let dma = self.dma_buf.as_ref().unwrap();
        let resp: GpuRespDisplayInfo = unsafe { dma.read_at(512) };
        
        for (i, pm) in resp.pmodes.iter().enumerate() {
            if pm.enabled != 0 {
                self.display_width = pm.r.width;
                self.display_height = pm.r.height;
                crate::serial_println!("[VIRTIO-GPU] Display {}: {}x{}", i, pm.r.width, pm.r.height);
                break;
            }
        }
        
        if self.display_width == 0 {
            self.display_width = 1280;
            self.display_height = 800;
            crate::serial_println!("[VIRTIO-GPU] Defaulting to {}x{}", self.display_width, self.display_height);
        }
        Ok(())
    }
    
    pub fn create_resource_2d(&mut self, width: u32, height: u32) -> Result<u32, &'static str> {
        let id = self.next_resource_id;
        self.next_resource_id += 1;
        let dma = self.dma_buf.as_ref().ok_or("DMA not ready")?;
        
        let cmd = GpuResourceCreate2d {
            hdr: GpuCtrlHdr { ctrl_type: GpuCtrlType::CmdResourceCreate2d as u32, ..Default::default() },
            resource_id: id,
            format: GpuFormat::B8G8R8X8Unorm as u32,
            width,
            height,
        };
        unsafe { dma.write_at(0, &cmd); }
        
        let resp = self.send_command(
            core::mem::size_of::<GpuResourceCreate2d>() as u32,
            512, core::mem::size_of::<GpuCtrlHdr>() as u32,
        )?;
        
        if resp != GpuCtrlType::RespOkNodata as u32 {
            return Err("RESOURCE_CREATE_2D failed");
        }
        crate::serial_println!("[VIRTIO-GPU] Resource {} created ({}x{})", id, width, height);
        Ok(id)
    }
    
    pub fn attach_backing(&mut self, resource_id: u32, buf_phys: u64, buf_len: u32) -> Result<(), &'static str> {
        let dma = self.dma_buf.as_ref().ok_or("DMA not ready")?;
        
        let cmd = GpuResourceAttachBacking {
            hdr: GpuCtrlHdr { ctrl_type: GpuCtrlType::CmdResourceAttachBacking as u32, ..Default::default() },
            resource_id,
            nr_entries: 1,
        };
        unsafe { dma.write_at(0, &cmd); }
        
        let entry = GpuMemEntry { addr: buf_phys, length: buf_len, padding: 0 };
        unsafe { dma.write_at(core::mem::size_of::<GpuResourceAttachBacking>(), &entry); }
        
        let cmd_sz = (core::mem::size_of::<GpuResourceAttachBacking>() + core::mem::size_of::<GpuMemEntry>()) as u32;
        let resp = self.send_command(cmd_sz, 512, core::mem::size_of::<GpuCtrlHdr>() as u32)?;
        
        if resp != GpuCtrlType::RespOkNodata as u32 {
            return Err("ATTACH_BACKING failed");
        }
        crate::serial_println!("[VIRTIO-GPU] Backing attached: phys={:#X} len={}", buf_phys, buf_len);
        Ok(())
    }
    
    pub fn set_scanout(&mut self, scanout_id: u32, resource_id: u32, w: u32, h: u32) -> Result<(), &'static str> {
        let dma = self.dma_buf.as_ref().ok_or("DMA not ready")?;
        let cmd = GpuSetScanout {
            hdr: GpuCtrlHdr { ctrl_type: GpuCtrlType::CmdSetScanout as u32, ..Default::default() },
            r: GpuRect { x: 0, y: 0, width: w, height: h },
            scanout_id,
            resource_id,
        };
        unsafe { dma.write_at(0, &cmd); }
        
        let resp = self.send_command(
            core::mem::size_of::<GpuSetScanout>() as u32,
            512, core::mem::size_of::<GpuCtrlHdr>() as u32,
        )?;
        
        if resp != GpuCtrlType::RespOkNodata as u32 { return Err("SET_SCANOUT failed"); }
        self.scanout_resource_id = resource_id;
        crate::serial_println!("[VIRTIO-GPU] Scanout {} -> resource {} ({}x{})", scanout_id, resource_id, w, h);
        Ok(())
    }
    
    pub fn transfer_to_host(&mut self, resource_id: u32, w: u32, h: u32) -> Result<(), &'static str> {
        let dma = self.dma_buf.as_ref().ok_or("DMA not ready")?;
        let cmd = GpuTransferToHost2d {
            hdr: GpuCtrlHdr { ctrl_type: GpuCtrlType::CmdTransferToHost2d as u32, ..Default::default() },
            r: GpuRect { x: 0, y: 0, width: w, height: h },
            offset: 0,
            resource_id,
            padding: 0,
        };
        unsafe { dma.write_at(0, &cmd); }
        
        let resp = self.send_command(
            core::mem::size_of::<GpuTransferToHost2d>() as u32,
            512, core::mem::size_of::<GpuCtrlHdr>() as u32,
        )?;
        if resp != GpuCtrlType::RespOkNodata as u32 { return Err("TRANSFER failed"); }
        Ok(())
    }
    
    pub fn flush_resource(&mut self, resource_id: u32, w: u32, h: u32) -> Result<(), &'static str> {
        let dma = self.dma_buf.as_ref().ok_or("DMA not ready")?;
        let cmd = GpuResourceFlush {
            hdr: GpuCtrlHdr { ctrl_type: GpuCtrlType::CmdResourceFlush as u32, ..Default::default() },
            r: GpuRect { x: 0, y: 0, width: w, height: h },
            resource_id,
            padding: 0,
        };
        unsafe { dma.write_at(0, &cmd); }
        
        let resp = self.send_command(
            core::mem::size_of::<GpuResourceFlush>() as u32,
            512, core::mem::size_of::<GpuCtrlHdr>() as u32,
        )?;
        if resp != GpuCtrlType::RespOkNodata as u32 { return Err("FLUSH failed"); }
        Ok(())
    }
    
    /// Setup complete scanout pipeline
    /// Uses the Limine framebuffer dimensions for consistency with the compositor
    pub fn setup_scanout(&mut self) -> Result<(), &'static str> {
        if !self.initialized { return Err("GPU not initialized"); }
        
        // Use Limine framebuffer dimensions so GPU resource matches compositor
        let (fb_w, fb_h) = crate::framebuffer::get_dimensions();
        if fb_w > 0 && fb_h > 0 {
            crate::serial_println!("[VIRTIO-GPU] Using framebuffer dimensions: {}x{} (display was {}x{})",
                fb_w, fb_h, self.display_width, self.display_height);
            self.display_width = fb_w;
            self.display_height = fb_h;
        }
        
        let w = self.display_width;
        let h = self.display_height;
        crate::serial_println!("[VIRTIO-GPU] Setting up scanout {}x{}", w, h);
        
        let resource_id = self.create_resource_2d(w, h)?;
        
        // Allocate page-aligned backing buffer
        let buf_size = (w * h) as usize;
        let buf_bytes = buf_size * 4;
        
        use alloc::alloc::{alloc_zeroed, Layout};
        let layout = Layout::from_size_align(buf_bytes, 4096).map_err(|_| "Layout error")?;
        let ptr = unsafe { alloc_zeroed(layout) };
        if ptr.is_null() { return Err("Backing buffer allocation failed"); }
        
        let virt = ptr as u64;
        let hhdm = memory::hhdm_offset();
        let phys = if virt >= hhdm { virt - hhdm } else { virt };
        
        let buffer = unsafe {
            let slice = core::slice::from_raw_parts_mut(ptr as *mut u32, buf_size);
            Box::from_raw(slice as *mut [u32])
        };
        
        self.backing_buffer = Some(buffer);
        self.backing_phys = phys;
        
        self.attach_backing(resource_id, phys, buf_bytes as u32)?;
        self.set_scanout(0, resource_id, w, h)?;
        
        crate::serial_println!("[VIRTIO-GPU] Scanout ready! phys={:#X}", phys);
        Ok(())
    }
    
    pub fn get_buffer(&mut self) -> Option<&mut [u32]> {
        self.backing_buffer.as_deref_mut()
    }
    
    pub fn get_dimensions(&self) -> (u32, u32) {
        (self.display_width, self.display_height)
    }
    
    /// Present: transfer backing buffer to host + flush display
    /// OPTIMIZED: Batches both commands in a single VirtIO submission
    /// - 1 notify instead of 2
    /// - 1 poll cycle instead of 2
    /// - 4 descriptor alloc/free instead of 4 (but 1 round trip)
    pub fn present(&mut self) -> Result<(), &'static str> {
        let rid = self.scanout_resource_id;
        if rid == 0 { return Err("No scanout"); }
        let (w, h) = (self.display_width, self.display_height);
        
        let dma = self.dma_buf.as_ref().ok_or("DMA not ready")?;
        let dma_phys = dma.phys;
        
        // Write transfer_to_host command at DMA offset 0
        let transfer_cmd = GpuTransferToHost2d {
            hdr: GpuCtrlHdr { ctrl_type: GpuCtrlType::CmdTransferToHost2d as u32, ..Default::default() },
            r: GpuRect { x: 0, y: 0, width: w, height: h },
            offset: 0,
            resource_id: rid,
            padding: 0,
        };
        unsafe { dma.write_at(0, &transfer_cmd); }
        
        // Write flush command at DMA offset 256
        let flush_cmd = GpuResourceFlush {
            hdr: GpuCtrlHdr { ctrl_type: GpuCtrlType::CmdResourceFlush as u32, ..Default::default() },
            r: GpuRect { x: 0, y: 0, width: w, height: h },
            resource_id: rid,
            padding: 0,
        };
        unsafe { dma.write_at(256, &flush_cmd); }
        
        let transfer_sz = core::mem::size_of::<GpuTransferToHost2d>() as u32;
        let flush_sz = core::mem::size_of::<GpuResourceFlush>() as u32;
        let resp_sz = core::mem::size_of::<GpuCtrlHdr>() as u32;
        
        let controlq = self.controlq.as_mut().ok_or("controlq not ready")?;
        
        // Allocate 4 descriptors for both command chains
        let d0 = controlq.alloc_desc().ok_or("No free desc")?;
        let d1 = controlq.alloc_desc().ok_or("No free desc")?;
        let d2 = controlq.alloc_desc().ok_or("No free desc")?;
        let d3 = controlq.alloc_desc().ok_or("No free desc")?;
        
        // Chain 1: transfer cmd (offset 0) → response (offset 512)
        controlq.set_desc(d0, dma_phys, transfer_sz, VIRTQ_DESC_F_NEXT, d1);
        controlq.set_desc(d1, dma_phys + 512, resp_sz, VIRTQ_DESC_F_WRITE, 0);
        
        // Chain 2: flush cmd (offset 256) → response (offset 768)
        controlq.set_desc(d2, dma_phys + 256, flush_sz, VIRTQ_DESC_F_NEXT, d3);
        controlq.set_desc(d3, dma_phys + 768, resp_sz, VIRTQ_DESC_F_WRITE, 0);
        
        // Submit both chains to the available ring
        controlq.submit(d0);
        controlq.submit(d2);
        
        // Single notification for both commands
        if let Some(notify) = &self.notify_cfg {
            notify.write16(0, 0);
        }
        
        // Poll for both completions
        let mut completed = 0u8;
        let mut timeout = 5_000_000u32;
        while completed < 2 {
            if let Some(_) = controlq.poll_used() {
                completed += 1;
            }
            if completed < 2 {
                timeout -= 1;
                if timeout == 0 {
                    controlq.free_desc(d3);
                    controlq.free_desc(d2);
                    controlq.free_desc(d1);
                    controlq.free_desc(d0);
                    return Err("Batched present timeout");
                }
                core::hint::spin_loop();
            }
        }
        
        // Check responses
        let dma = self.dma_buf.as_ref().unwrap();
        let t_resp = unsafe { dma.read_at::<GpuCtrlHdr>(512) }.ctrl_type;
        let f_resp = unsafe { dma.read_at::<GpuCtrlHdr>(768) }.ctrl_type;
        
        controlq.free_desc(d3);
        controlq.free_desc(d2);
        controlq.free_desc(d1);
        controlq.free_desc(d0);
        
        if t_resp != GpuCtrlType::RespOkNodata as u32 { return Err("TRANSFER failed"); }
        if f_resp != GpuCtrlType::RespOkNodata as u32 { return Err("FLUSH failed"); }
        
        Ok(())
    }
    
    pub fn is_initialized(&self) -> bool { self.initialized }
    pub fn has_3d_support(&self) -> bool { self.has_3d }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Global Instance & Public API
// ═══════════════════════════════════════════════════════════════════════════════

static GPU: Mutex<VirtioGpu> = Mutex::new(VirtioGpu::new());
static GPU_AVAILABLE: AtomicBool = AtomicBool::new(false);

pub fn init_from_pci() -> Result<(), &'static str> {
    for device in crate::pci::scan() {
        if device.vendor_id == VIRTIO_VENDOR_ID && device.device_id == VIRTIO_GPU_PCI_DEVICE_ID {
            crate::serial_println!("[VIRTIO-GPU] Found device at {:02x}:{:02x}.{}",
                device.bus, device.device, device.function);
            
            let mut gpu = GPU.lock();
            match gpu.init(device) {
                Ok(()) => {
                    match gpu.setup_scanout() {
                        Ok(()) => {
                            GPU_AVAILABLE.store(true, Ordering::SeqCst);
                            crate::serial_println!("[VIRTIO-GPU] ✓ Ready for rendering!");
                        }
                        Err(e) => crate::serial_println!("[VIRTIO-GPU] Scanout failed: {}", e),
                    }
                }
                Err(e) => crate::serial_println!("[VIRTIO-GPU] Init failed: {}", e),
            }
            return Ok(());
        }
    }
    crate::serial_println!("[VIRTIO-GPU] No VirtIO GPU found");
    Ok(())
}

pub fn is_available() -> bool {
    GPU_AVAILABLE.load(Ordering::SeqCst)
}

pub fn create_surface(width: u32, height: u32) -> GpuSurface {
    GpuSurface::new(width, height)
}

/// Render frame using VirtIO GPU DMA path
pub fn render_frame<F: FnOnce(&mut [u32], u32, u32)>(render_fn: F) -> Result<(), &'static str> {
    let mut gpu = GPU.lock();
    if !gpu.initialized { return Err("GPU not initialized"); }
    let (w, h) = (gpu.display_width, gpu.display_height);
    if let Some(buf) = gpu.backing_buffer.as_deref_mut() {
        render_fn(buf, w, h);
    }
    gpu.present()
}

/// Present the current backing buffer (after external rendering)
pub fn present_frame() -> Result<(), &'static str> {
    GPU.lock().present()
}

/// Get raw buffer pointer for direct rendering
pub fn get_raw_buffer() -> Option<(*mut u32, u32, u32)> {
    let mut gpu = GPU.lock();
    if !gpu.initialized { return None; }
    let (w, h) = (gpu.display_width, gpu.display_height);
    gpu.backing_buffer.as_deref_mut().map(|buf| (buf.as_mut_ptr(), w, h))
}

/// Info string for shell
pub fn info_string() -> alloc::string::String {
    let gpu = GPU.lock();
    if gpu.initialized {
        alloc::format!("VirtIO GPU: {}x{} 2D (3D={})", gpu.display_width, gpu.display_height,
            if gpu.has_3d { "virgl" } else { "no" })
    } else {
        alloc::string::String::from("VirtIO GPU: not available")
    }
}

/// Fallback: blit surface to framebuffer (when no VirtIO GPU)
pub fn blit_to_screen(surface: &GpuSurface, x: u32, y: u32) {
    let (fb_w, fb_h) = crate::framebuffer::get_dimensions();
    crate::framebuffer::set_double_buffer_mode(true);
    for sy in 0..surface.height {
        let sy2 = y + sy;
        if sy2 >= fb_h { break; }
        for sx in 0..surface.width {
            let sx2 = x + sx;
            if sx2 >= fb_w { break; }
            crate::framebuffer::put_pixel(sx2, sy2, surface.get_pixel(sx, sy));
        }
    }
    crate::framebuffer::swap_buffers();
}

pub fn flush_screen() {
    if crate::framebuffer::is_double_buffer_enabled() {
        crate::framebuffer::swap_buffers();
    }
}

pub fn init() {
    crate::serial_println!("[GPU] Initializing graphics subsystem...");
    if let Err(e) = init_from_pci() {
        crate::serial_println!("[GPU] PCI init error: {}", e);
    }
    crate::framebuffer::init_double_buffer();
    crate::framebuffer::set_double_buffer_mode(true);
    crate::serial_println!("[GPU] Graphics ready (VirtIO: {})", 
        if is_available() { "ACTIVE" } else { "fallback" });
}

// ═══════════════════════════════════════════════════════════════════════════════
// Compositor (compatibility)
// ═══════════════════════════════════════════════════════════════════════════════

pub struct Layer {
    pub surface: GpuSurface,
    pub x: i32,
    pub y: i32,
    pub z_order: i32,
    pub visible: bool,
    pub opacity: u8,
}

pub struct Compositor {
    layers: Vec<Layer>,
    output: GpuSurface,
    background_color: u32,
}

impl Compositor {
    pub fn new(width: u32, height: u32) -> Self {
        Self { layers: Vec::new(), output: GpuSurface::new(width, height), background_color: 0xFF1A1A1A }
    }
    pub fn add_layer(&mut self, surface: GpuSurface, x: i32, y: i32, z_order: i32) -> usize {
        let idx = self.layers.len();
        self.layers.push(Layer { surface, x, y, z_order, visible: true, opacity: 255 });
        self.layers.sort_by_key(|l| l.z_order);
        idx
    }
    pub fn remove_layer(&mut self, index: usize) {
        if index < self.layers.len() { self.layers.remove(index); }
    }
    pub fn compose(&mut self) {
        self.output.clear(self.background_color);
        for layer in &self.layers {
            if layer.visible { self.output.blit(&layer.surface, layer.x, layer.y); }
        }
    }
    pub fn render(&self) { blit_to_screen(&self.output, 0, 0); }
    pub fn get_layer(&self, index: usize) -> Option<&Layer> { self.layers.get(index) }
    pub fn get_layer_mut(&mut self, index: usize) -> Option<&mut Layer> { self.layers.get_mut(index) }
    pub fn set_background(&mut self, color: u32) { self.background_color = color; }
}
