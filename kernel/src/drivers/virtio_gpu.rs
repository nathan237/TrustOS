//! VirtIO GPU Driver
//!
//! This driver implements the VirtIO GPU specification for 2D/3D acceleration.
//! It provides hardware-accelerated graphics for the TrustOS desktop.
//!
//! Features:
//! - 2D scanout (display buffer to screen)
//! - 2D resource management (textures, surfaces)
//! - 2D rendering commands (blit, fill, copy)
//! - Optional 3D support (OpenGL ES commands)
//!
//! Reference: https://docs.oasis-open.org/virtio/virtio/v1.2/virtio-v1.2.html

use alloc::vec::Vec;
use alloc::boxed::Box;
use alloc::collections::BTreeMap;
use core::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use spin::Mutex;

use crate::pci::PciDevice;

// ═══════════════════════════════════════════════════════════════════════════════
// VirtIO GPU Constants
// ═══════════════════════════════════════════════════════════════════════════════

/// VirtIO GPU device ID
pub const VIRTIO_GPU_DEVICE_ID: u16 = 0x1050;

/// VirtIO GPU feature bits
pub mod features {
    pub const VIRTIO_GPU_F_VIRGL: u32 = 0;      // 3D virgl support
    pub const VIRTIO_GPU_F_EDID: u32 = 1;       // EDID support
    pub const VIRTIO_GPU_F_RESOURCE_UUID: u32 = 2;
    pub const VIRTIO_GPU_F_RESOURCE_BLOB: u32 = 3;
    pub const VIRTIO_GPU_F_CONTEXT_INIT: u32 = 4;
}

/// VirtIO GPU control commands
#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum VirtioGpuCtrlType {
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
    CmdResourceAssignUuid = 0x010b,
    CmdResourceCreateBlob = 0x010c,
    CmdSetScanoutBlob = 0x010d,

    // Cursor commands
    CmdUpdateCursor = 0x0300,
    CmdMoveCursor = 0x0301,

    // Success responses
    RespOkNodata = 0x1100,
    RespOkDisplayInfo = 0x1101,
    RespOkCapsetInfo = 0x1102,
    RespOkCapset = 0x1103,
    RespOkEdid = 0x1104,
    RespOkResourceUuid = 0x1105,
    RespOkMapInfo = 0x1106,

    // Error responses
    RespErrUnspec = 0x1200,
    RespErrOutOfMemory = 0x1201,
    RespErrInvalidScanoutId = 0x1202,
    RespErrInvalidResourceId = 0x1203,
    RespErrInvalidContextId = 0x1204,
    RespErrInvalidParameter = 0x1205,
}

/// Pixel formats supported
#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum VirtioGpuFormats {
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
// VirtIO GPU Structures
// ═══════════════════════════════════════════════════════════════════════════════

/// Control header for all commands
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct VirtioGpuCtrlHdr {
    pub ctrl_type: u32,
    pub flags: u32,
    pub fence_id: u64,
    pub ctx_id: u32,
    pub ring_idx: u8,
    pub padding: [u8; 3],
}

/// Display info for one scanout
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct VirtioGpuDisplayOne {
    pub r: VirtioGpuRect,
    pub enabled: u32,
    pub flags: u32,
}

/// Rectangle structure
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct VirtioGpuRect {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

/// Resource create 2D command
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct VirtioGpuResourceCreate2d {
    pub hdr: VirtioGpuCtrlHdr,
    pub resource_id: u32,
    pub format: u32,
    pub width: u32,
    pub height: u32,
}

/// Set scanout command
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct VirtioGpuSetScanout {
    pub hdr: VirtioGpuCtrlHdr,
    pub r: VirtioGpuRect,
    pub scanout_id: u32,
    pub resource_id: u32,
}

/// Resource flush command
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct VirtioGpuResourceFlush {
    pub hdr: VirtioGpuCtrlHdr,
    pub r: VirtioGpuRect,
    pub resource_id: u32,
    pub padding: u32,
}

/// Transfer to host 2D command
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct VirtioGpuTransferToHost2d {
    pub hdr: VirtioGpuCtrlHdr,
    pub r: VirtioGpuRect,
    pub offset: u64,
    pub resource_id: u32,
    pub padding: u32,
}

/// Memory entry for resource backing
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct VirtioGpuMemEntry {
    pub addr: u64,
    pub length: u32,
    pub padding: u32,
}

/// Attach backing command
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct VirtioGpuResourceAttachBacking {
    pub hdr: VirtioGpuCtrlHdr,
    pub resource_id: u32,
    pub nr_entries: u32,
}

/// Cursor position
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct VirtioGpuCursorPos {
    pub scanout_id: u32,
    pub x: u32,
    pub y: u32,
    pub padding: u32,
}

/// Update cursor command
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct VirtioGpuUpdateCursor {
    pub hdr: VirtioGpuCtrlHdr,
    pub pos: VirtioGpuCursorPos,
    pub resource_id: u32,
    pub hot_x: u32,
    pub hot_y: u32,
    pub padding: u32,
}

// ═══════════════════════════════════════════════════════════════════════════════
// GPU Resource Management
// ═══════════════════════════════════════════════════════════════════════════════

/// A GPU resource (texture/surface)
pub struct GpuResource {
    pub id: u32,
    pub width: u32,
    pub height: u32,
    pub format: VirtioGpuFormats,
    pub data: Box<[u32]>,  // Pixel data (RGBA)
}

/// GPU Surface for 2D operations
pub struct GpuSurface {
    pub resource_id: u32,
    pub width: u32,
    pub height: u32,
    pub data: Box<[u32]>,
}

impl GpuSurface {
    /// Create a new surface
    pub fn new(width: u32, height: u32) -> Self {
        let size = (width * height) as usize;
        let data = alloc::vec![0u32; size].into_boxed_slice();
        
        Self {
            resource_id: 0,
            width,
            height,
            data,
        }
    }
    
    /// Clear with color
    pub fn clear(&mut self, color: u32) {
        self.data.fill(color);
    }
    
    /// Set pixel
    #[inline]
    pub fn set_pixel(&mut self, x: u32, y: u32, color: u32) {
        if x < self.width && y < self.height {
            let idx = (y * self.width + x) as usize;
            self.data[idx] = color;
        }
    }
    
    /// Get pixel
    #[inline]
    pub fn get_pixel(&self, x: u32, y: u32) -> u32 {
        if x < self.width && y < self.height {
            let idx = (y * self.width + x) as usize;
            self.data[idx]
        } else {
            0
        }
    }
    
    /// Fill rectangle
    pub fn fill_rect(&mut self, x: u32, y: u32, w: u32, h: u32, color: u32) {
        let x1 = x.min(self.width);
        let y1 = y.min(self.height);
        let x2 = (x + w).min(self.width);
        let y2 = (y + h).min(self.height);
        
        for py in y1..y2 {
            let row_start = (py * self.width + x1) as usize;
            let row_end = (py * self.width + x2) as usize;
            self.data[row_start..row_end].fill(color);
        }
    }
    
    /// Blit another surface (with alpha blending)
    pub fn blit(&mut self, src: &GpuSurface, dst_x: i32, dst_y: i32) {
        for sy in 0..src.height {
            for sx in 0..src.width {
                let dx = dst_x + sx as i32;
                let dy = dst_y + sy as i32;
                
                if dx >= 0 && dy >= 0 && dx < self.width as i32 && dy < self.height as i32 {
                    let src_pixel = src.get_pixel(sx, sy);
                    let alpha = (src_pixel >> 24) & 0xFF;
                    
                    if alpha == 255 {
                        self.set_pixel(dx as u32, dy as u32, src_pixel);
                    } else if alpha > 0 {
                        let dst_pixel = self.get_pixel(dx as u32, dy as u32);
                        let blended = blend_pixels(src_pixel, dst_pixel, alpha);
                        self.set_pixel(dx as u32, dy as u32, blended);
                    }
                }
            }
        }
    }
    
    /// Blit with scaling (simple nearest-neighbor)
    pub fn blit_scaled(&mut self, src: &GpuSurface, dst_x: i32, dst_y: i32, dst_w: u32, dst_h: u32) {
        if dst_w == 0 || dst_h == 0 || src.width == 0 || src.height == 0 {
            return;
        }
        
        for dy in 0..dst_h {
            for dx in 0..dst_w {
                let sx = (dx * src.width) / dst_w;
                let sy = (dy * src.height) / dst_h;
                
                let px = dst_x + dx as i32;
                let py = dst_y + dy as i32;
                
                if px >= 0 && py >= 0 && px < self.width as i32 && py < self.height as i32 {
                    let color = src.get_pixel(sx, sy);
                    self.set_pixel(px as u32, py as u32, color);
                }
            }
        }
    }
    
    /// Draw line (Bresenham)
    pub fn draw_line(&mut self, x0: i32, y0: i32, x1: i32, y1: i32, color: u32) {
        let dx = (x1 - x0).abs();
        let dy = -(y1 - y0).abs();
        let sx = if x0 < x1 { 1 } else { -1 };
        let sy = if y0 < y1 { 1 } else { -1 };
        let mut err = dx + dy;
        let mut x = x0;
        let mut y = y0;
        
        loop {
            if x >= 0 && y >= 0 && x < self.width as i32 && y < self.height as i32 {
                self.set_pixel(x as u32, y as u32, color);
            }
            
            if x == x1 && y == y1 { break; }
            
            let e2 = 2 * err;
            if e2 >= dy {
                err += dy;
                x += sx;
            }
            if e2 <= dx {
                err += dx;
                y += sy;
            }
        }
    }
    
    /// Draw rectangle outline
    pub fn draw_rect(&mut self, x: u32, y: u32, w: u32, h: u32, color: u32) {
        let x = x as i32;
        let y = y as i32;
        let w = w as i32;
        let h = h as i32;
        
        self.draw_line(x, y, x + w - 1, y, color);
        self.draw_line(x, y + h - 1, x + w - 1, y + h - 1, color);
        self.draw_line(x, y, x, y + h - 1, color);
        self.draw_line(x + w - 1, y, x + w - 1, y + h - 1, color);
    }
    
    /// Draw circle (Midpoint algorithm)
    pub fn draw_circle(&mut self, cx: i32, cy: i32, radius: i32, color: u32) {
        let mut x = radius;
        let mut y = 0;
        let mut err = 0;
        
        while x >= y {
            self.set_pixel_safe(cx + x, cy + y, color);
            self.set_pixel_safe(cx + y, cy + x, color);
            self.set_pixel_safe(cx - y, cy + x, color);
            self.set_pixel_safe(cx - x, cy + y, color);
            self.set_pixel_safe(cx - x, cy - y, color);
            self.set_pixel_safe(cx - y, cy - x, color);
            self.set_pixel_safe(cx + y, cy - x, color);
            self.set_pixel_safe(cx + x, cy - y, color);
            
            y += 1;
            err += 1 + 2 * y;
            if 2 * (err - x) + 1 > 0 {
                x -= 1;
                err += 1 - 2 * x;
            }
        }
    }
    
    /// Fill circle
    pub fn fill_circle(&mut self, cx: i32, cy: i32, radius: i32, color: u32) {
        for dy in -radius..=radius {
            for dx in -radius..=radius {
                if dx * dx + dy * dy <= radius * radius {
                    self.set_pixel_safe(cx + dx, cy + dy, color);
                }
            }
        }
    }
    
    /// Draw rounded rectangle
    pub fn draw_rounded_rect(&mut self, x: u32, y: u32, w: u32, h: u32, radius: u32, color: u32) {
        let r = radius.min(w / 2).min(h / 2) as i32;
        let x = x as i32;
        let y = y as i32;
        let w = w as i32;
        let h = h as i32;
        
        // Horizontal lines
        self.draw_line(x + r, y, x + w - r - 1, y, color);
        self.draw_line(x + r, y + h - 1, x + w - r - 1, y + h - 1, color);
        
        // Vertical lines
        self.draw_line(x, y + r, x, y + h - r - 1, color);
        self.draw_line(x + w - 1, y + r, x + w - 1, y + h - r - 1, color);
        
        // Corners (quarter circles)
        self.draw_quarter_circle(x + r, y + r, r, color, 2);
        self.draw_quarter_circle(x + w - r - 1, y + r, r, color, 1);
        self.draw_quarter_circle(x + r, y + h - r - 1, r, color, 3);
        self.draw_quarter_circle(x + w - r - 1, y + h - r - 1, r, color, 4);
    }
    
    /// Fill rounded rectangle
    pub fn fill_rounded_rect(&mut self, x: u32, y: u32, w: u32, h: u32, radius: u32, color: u32) {
        let r = radius.min(w / 2).min(h / 2);
        
        // Center rectangle
        self.fill_rect(x + r, y, w - 2 * r, h, color);
        
        // Left and right strips
        self.fill_rect(x, y + r, r, h - 2 * r, color);
        self.fill_rect(x + w - r, y + r, r, h - 2 * r, color);
        
        // Corner fills
        self.fill_quarter_circle((x + r) as i32, (y + r) as i32, r as i32, color, 2);
        self.fill_quarter_circle((x + w - r - 1) as i32, (y + r) as i32, r as i32, color, 1);
        self.fill_quarter_circle((x + r) as i32, (y + h - r - 1) as i32, r as i32, color, 3);
        self.fill_quarter_circle((x + w - r - 1) as i32, (y + h - r - 1) as i32, r as i32, color, 4);
    }
    
    fn set_pixel_safe(&mut self, x: i32, y: i32, color: u32) {
        if x >= 0 && y >= 0 && x < self.width as i32 && y < self.height as i32 {
            self.set_pixel(x as u32, y as u32, color);
        }
    }
    
    fn draw_quarter_circle(&mut self, cx: i32, cy: i32, r: i32, color: u32, quarter: u8) {
        let mut x = r;
        let mut y = 0;
        let mut err = 0;
        
        while x >= y {
            match quarter {
                1 => { // Top-right
                    self.set_pixel_safe(cx + x, cy - y, color);
                    self.set_pixel_safe(cx + y, cy - x, color);
                }
                2 => { // Top-left
                    self.set_pixel_safe(cx - x, cy - y, color);
                    self.set_pixel_safe(cx - y, cy - x, color);
                }
                3 => { // Bottom-left
                    self.set_pixel_safe(cx - x, cy + y, color);
                    self.set_pixel_safe(cx - y, cy + x, color);
                }
                4 => { // Bottom-right
                    self.set_pixel_safe(cx + x, cy + y, color);
                    self.set_pixel_safe(cx + y, cy + x, color);
                }
                _ => {}
            }
            
            y += 1;
            err += 1 + 2 * y;
            if 2 * (err - x) + 1 > 0 {
                x -= 1;
                err += 1 - 2 * x;
            }
        }
    }
    
    fn fill_quarter_circle(&mut self, cx: i32, cy: i32, r: i32, color: u32, quarter: u8) {
        for dy in 0..=r {
            for dx in 0..=r {
                if dx * dx + dy * dy <= r * r {
                    match quarter {
                        1 => self.set_pixel_safe(cx + dx, cy - dy, color),
                        2 => self.set_pixel_safe(cx - dx, cy - dy, color),
                        3 => self.set_pixel_safe(cx - dx, cy + dy, color),
                        4 => self.set_pixel_safe(cx + dx, cy + dy, color),
                        _ => {}
                    }
                }
            }
        }
    }
}

/// Alpha blend two pixels
#[inline]
fn blend_pixels(src: u32, dst: u32, alpha: u32) -> u32 {
    let inv_alpha = 255 - alpha;
    
    let sr = (src >> 16) & 0xFF;
    let sg = (src >> 8) & 0xFF;
    let sb = src & 0xFF;
    
    let dr = (dst >> 16) & 0xFF;
    let dg = (dst >> 8) & 0xFF;
    let db = dst & 0xFF;
    
    let r = (sr * alpha + dr * inv_alpha) / 255;
    let g = (sg * alpha + dg * inv_alpha) / 255;
    let b = (sb * alpha + db * inv_alpha) / 255;
    
    0xFF000000 | (r << 16) | (g << 8) | b
}

// ═══════════════════════════════════════════════════════════════════════════════
// VirtIO GPU Driver
// ═══════════════════════════════════════════════════════════════════════════════

/// VirtIO GPU driver state
pub struct VirtioGpu {
    /// PCI device
    pci_device: Option<PciDevice>,
    
    /// Device features
    features: u32,
    
    /// Display info
    displays: Vec<VirtioGpuDisplayOne>,
    
    /// Resources
    resources: BTreeMap<u32, GpuResource>,
    next_resource_id: u32,
    
    /// Primary scanout surface
    scanout_resource: u32,
    
    /// Initialized flag
    initialized: bool,
    
    /// 3D (virgl) support
    has_3d: bool,
}

impl VirtioGpu {
    pub const fn new() -> Self {
        Self {
            pci_device: None,
            features: 0,
            displays: Vec::new(),
            resources: BTreeMap::new(),
            next_resource_id: 1,
            scanout_resource: 0,
            initialized: false,
            has_3d: false,
        }
    }
    
    /// Initialize the GPU driver
    pub fn init(&mut self, device: PciDevice) -> Result<(), &'static str> {
        crate::serial_println!("[VIRTIO-GPU] Initializing GPU driver...");
        
        self.pci_device = Some(device);
        
        // TODO: Read features, negotiate, setup virtqueues
        // For now, we simulate a basic 2D GPU
        
        // Simulate display info
        let display = VirtioGpuDisplayOne {
            r: VirtioGpuRect { x: 0, y: 0, width: 1920, height: 1080 },
            enabled: 1,
            flags: 0,
        };
        self.displays.push(display);
        
        self.initialized = true;
        
        crate::serial_println!("[VIRTIO-GPU] Initialized with {} display(s)", self.displays.len());
        
        Ok(())
    }
    
    /// Check if initialized
    pub fn is_initialized(&self) -> bool {
        self.initialized
    }
    
    /// Check if 3D is supported
    pub fn has_3d_support(&self) -> bool {
        self.has_3d
    }
    
    /// Get display count
    pub fn display_count(&self) -> usize {
        self.displays.len()
    }
    
    /// Get display info
    pub fn get_display(&self, index: usize) -> Option<&VirtioGpuDisplayOne> {
        self.displays.get(index)
    }
    
    /// Create a 2D resource
    pub fn create_resource_2d(&mut self, width: u32, height: u32, format: VirtioGpuFormats) -> u32 {
        let id = self.next_resource_id;
        self.next_resource_id += 1;
        
        let size = (width * height) as usize;
        let data = alloc::vec![0u32; size].into_boxed_slice();
        
        let resource = GpuResource {
            id,
            width,
            height,
            format,
            data,
        };
        
        self.resources.insert(id, resource);
        
        crate::serial_println!("[VIRTIO-GPU] Created resource {} ({}x{})", id, width, height);
        
        id
    }
    
    /// Destroy a resource
    pub fn destroy_resource(&mut self, id: u32) {
        self.resources.remove(&id);
    }
    
    /// Get resource
    pub fn get_resource(&self, id: u32) -> Option<&GpuResource> {
        self.resources.get(&id)
    }
    
    /// Get resource mutable
    pub fn get_resource_mut(&mut self, id: u32) -> Option<&mut GpuResource> {
        self.resources.get_mut(&id)
    }
    
    /// Set scanout (bind resource to display)
    pub fn set_scanout(&mut self, scanout_id: u32, resource_id: u32) -> Result<(), &'static str> {
        if scanout_id as usize >= self.displays.len() {
            return Err("Invalid scanout ID");
        }
        
        if resource_id != 0 && !self.resources.contains_key(&resource_id) {
            return Err("Invalid resource ID");
        }
        
        self.scanout_resource = resource_id;
        
        Ok(())
    }
    
    /// Flush resource to display (copy to framebuffer)
    pub fn flush_resource(&self, resource_id: u32, rect: &VirtioGpuRect) -> Result<(), &'static str> {
        let resource = self.resources.get(&resource_id)
            .ok_or("Invalid resource ID")?;
        
        // Copy resource data to framebuffer
        let (fb_width, fb_height) = crate::framebuffer::get_dimensions();
        
        let x1 = rect.x.min(fb_width).min(resource.width);
        let y1 = rect.y.min(fb_height).min(resource.height);
        let x2 = (rect.x + rect.width).min(fb_width).min(resource.width);
        let y2 = (rect.y + rect.height).min(fb_height).min(resource.height);
        
        for y in y1..y2 {
            for x in x1..x2 {
                let idx = (y * resource.width + x) as usize;
                if idx < resource.data.len() {
                    crate::framebuffer::put_pixel(x, y, resource.data[idx]);
                }
            }
        }
        
        Ok(())
    }
    
    /// Update hardware cursor
    pub fn update_cursor(&mut self, resource_id: u32, hot_x: u32, hot_y: u32) -> Result<(), &'static str> {
        // TODO: Implement hardware cursor
        Ok(())
    }
    
    /// Move hardware cursor
    pub fn move_cursor(&mut self, x: u32, y: u32) -> Result<(), &'static str> {
        // TODO: Implement hardware cursor movement
        Ok(())
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Global GPU Instance
// ═══════════════════════════════════════════════════════════════════════════════

static GPU: Mutex<VirtioGpu> = Mutex::new(VirtioGpu::new());
static GPU_AVAILABLE: AtomicBool = AtomicBool::new(false);

/// Initialize VirtIO GPU from PCI scan
pub fn init_from_pci() -> Result<(), &'static str> {
    // Look for VirtIO GPU device
    for device in crate::pci::scan() {
        if device.vendor_id == 0x1AF4 && device.device_id == VIRTIO_GPU_DEVICE_ID {
            crate::serial_println!("[VIRTIO-GPU] Found VirtIO GPU at {:02x}:{:02x}.{}", 
                device.bus, device.device, device.function);
            
            let mut gpu = GPU.lock();
            gpu.init(device)?;
            
            GPU_AVAILABLE.store(true, Ordering::SeqCst);
            
            return Ok(());
        }
    }
    
    crate::serial_println!("[VIRTIO-GPU] No VirtIO GPU found, using software rendering");
    Ok(())
}

/// Check if GPU is available
pub fn is_available() -> bool {
    GPU_AVAILABLE.load(Ordering::SeqCst)
}

/// Create a surface
pub fn create_surface(width: u32, height: u32) -> GpuSurface {
    GpuSurface::new(width, height)
}

/// Blit surface to framebuffer
pub fn blit_to_screen(surface: &GpuSurface, x: u32, y: u32) {
    let (fb_width, fb_height) = crate::framebuffer::get_dimensions();
    
    // Use double buffering if available
    crate::framebuffer::set_double_buffer_mode(true);
    
    for sy in 0..surface.height {
        let screen_y = y + sy;
        if screen_y >= fb_height { break; }
        
        for sx in 0..surface.width {
            let screen_x = x + sx;
            if screen_x >= fb_width { break; }
            
            let color = surface.get_pixel(sx, sy);
            crate::framebuffer::put_pixel(screen_x, screen_y, color);
        }
    }
    
    crate::framebuffer::swap_buffers();
}

/// Flush entire screen
pub fn flush_screen() {
    if crate::framebuffer::is_double_buffer_enabled() {
        crate::framebuffer::swap_buffers();
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Compositor Layer
// ═══════════════════════════════════════════════════════════════════════════════

/// Layer for compositing
pub struct Layer {
    pub surface: GpuSurface,
    pub x: i32,
    pub y: i32,
    pub z_order: i32,
    pub visible: bool,
    pub opacity: u8,
}

/// Simple compositor
pub struct Compositor {
    layers: Vec<Layer>,
    output: GpuSurface,
    background_color: u32,
}

impl Compositor {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            layers: Vec::new(),
            output: GpuSurface::new(width, height),
            background_color: 0xFF1A1A1A,
        }
    }
    
    /// Add a layer
    pub fn add_layer(&mut self, surface: GpuSurface, x: i32, y: i32, z_order: i32) -> usize {
        let idx = self.layers.len();
        self.layers.push(Layer {
            surface,
            x,
            y,
            z_order,
            visible: true,
            opacity: 255,
        });
        
        // Sort by z-order
        self.layers.sort_by_key(|l| l.z_order);
        
        idx
    }
    
    /// Remove a layer
    pub fn remove_layer(&mut self, index: usize) {
        if index < self.layers.len() {
            self.layers.remove(index);
        }
    }
    
    /// Compose all layers and render to output
    pub fn compose(&mut self) {
        // Clear background
        self.output.clear(self.background_color);
        
        // Composite layers (back to front)
        for layer in &self.layers {
            if layer.visible {
                self.output.blit(&layer.surface, layer.x, layer.y);
            }
        }
    }
    
    /// Render to screen
    pub fn render(&self) {
        blit_to_screen(&self.output, 0, 0);
    }
    
    /// Get layer
    pub fn get_layer(&self, index: usize) -> Option<&Layer> {
        self.layers.get(index)
    }
    
    /// Get layer mutable
    pub fn get_layer_mut(&mut self, index: usize) -> Option<&mut Layer> {
        self.layers.get_mut(index)
    }
    
    /// Set background color
    pub fn set_background(&mut self, color: u32) {
        self.background_color = color;
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// High-Level Graphics API
// ═══════════════════════════════════════════════════════════════════════════════

/// Initialize the graphics subsystem
pub fn init() {
    crate::serial_println!("[GPU] Initializing graphics subsystem...");
    
    // Try to find VirtIO GPU
    if let Err(e) = init_from_pci() {
        crate::serial_println!("[GPU] PCI init error: {}", e);
    }
    
    // Enable double buffering in framebuffer
    crate::framebuffer::init_double_buffer();
    crate::framebuffer::set_double_buffer_mode(true);
    
    crate::serial_println!("[GPU] Graphics subsystem ready");
}
