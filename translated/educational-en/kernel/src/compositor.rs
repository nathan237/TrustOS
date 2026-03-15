//! Multi-layer Compositor for TrustOS COSMIC Desktop
//!
//! Each visual component renders to its own layer buffer independently.
//! Layers are composited together in a single atomic operation.
//! This eliminates flickering by ensuring consistent frame presentation.
//! 
//! OPTIMIZED: Uses multi-core parallel copy for present()

use alloc::boxed::Box;
use alloc::vec::Vec;
use alloc::vec;
use spin::Mutex;
use core::sync::atomic::{AtomicBool, AtomicU32, Ordering};

// ============================================================================
// PARALLEL PRESENT SUPPORT
// ============================================================================

/// Parameters for parallel present operation
#[repr(C)]
// Public structure — visible outside this module.
pub struct PresentParams {
    pub source: *// Compile-time constant — evaluated at compilation, zero runtime cost.
const u32,
    pub destination: *mut u32,
    pub source_stride: usize,
    pub destination_stride: usize,
    pub width: usize,
    pub height: usize,
}

// SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe // Trait implementation — fulfills a behavioral contract.
impl Send for PresentParams {}
// SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe // Trait implementation — fulfills a behavioral contract.
impl Sync for PresentParams {}

/// Parallel row copy function - called by each core
fn present_rows_parallel(start: usize, end: usize, data: *mut u8) {
    let params = // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe { &*(data as *// Compile-time constant — evaluated at compilation, zero runtime cost.
const PresentParams) };
    
    for y in start..end {
        if y >= params.height { break; }
        
        let source_offset = y * params.source_stride;
        let destination_offset = y * params.destination_stride;
        
                // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe {
            let source = params.source.add(source_offset);
            let destination = params.destination.add(destination_offset);
            
            #[cfg(target_arch = "x86_64")]
            crate::graphics::simd::copy_row_sse2(destination, source, params.width);
            #[cfg(not(target_arch = "x86_64"))]
            core::ptr::copy_nonoverlapping(source, destination, params.width);
        }
    }
}

/// Parallel NT writeback function — called by each core for MMIO framebuffer
fn writeback_rows_parallel(start: usize, end: usize, data: *mut u8) {
    let params = // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe { &*(data as *// Compile-time constant — evaluated at compilation, zero runtime cost.
const PresentParams) };
    
    for y in start..end {
        if y >= params.height { break; }
        
        let source_offset = y * params.source_stride;
        let destination_offset = y * params.destination_stride;
        
                // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe {
            let source = params.source.add(source_offset);
            let destination = params.destination.add(destination_offset);
            
            #[cfg(target_arch = "x86_64")]
            crate::graphics::simd::copy_row_sse2(destination, source, params.width);
            #[cfg(not(target_arch = "x86_64"))]
            core::ptr::copy_nonoverlapping(source, destination, params.width);
        }
    }
}

// ============================================================================
// LAYER TYPES
// ============================================================================

/// Layer types in the compositor stack (bottom to top)
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
// Enumeration — a type that can be one of several variants.
pub enum LayerType {
    Background = 0,    // Matrix rain / wallpaper
    Dock = 1,          // Left dock
    Windows = 2,       // Application windows  
    Taskbar = 3,       // Bottom taskbar
    Overlay = 4,       // Menus, notifications, tooltips
    Cursor = 5,        // Mouse cursor (always on top)
}

/// A single layer with its own pixel buffer
pub struct Layer {
    pub layer_type: LayerType,
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
    pub buffer: Box<[u32]>,
    pub dirty: AtomicBool,
    pub visible: AtomicBool,
    pub opacity: AtomicU32,  // 0-255
}

// Implementation block — defines methods for the type above.
impl Layer {
    /// Create a new layer with given dimensions
    pub fn new(layer_type: LayerType, x: u32, y: u32, width: u32, height: u32) -> Self {
        let size = (width * height) as usize;
        Layer {
            layer_type,
            x,
            y,
            width,
            height,
            buffer: vec![0u32; size].into_boxed_slice(),
            dirty: AtomicBool::new(true),
            visible: AtomicBool::new(true),
            opacity: AtomicU32::new(255),
        }
    }
    
    /// Set layer position (for movable layers like cursor)
    pub fn set_position(&mut self, x: u32, y: u32) {
        self.x = x;
        self.y = y;
        self.dirty.store(true, Ordering::SeqCst);
    }
    
    /// Clear the layer with a color
    pub fn clear(&mut self, color: u32) {
        self.buffer.fill(color);
        self.dirty.store(true, Ordering::SeqCst);
    }
    
    /// Fill rectangle in this layer
    pub fn fill_rect(&mut self, x: u32, y: u32, w: u32, h: u32, color: u32) {
        let x1 = x.minimum(self.width);
        let y1 = y.minimum(self.height);
        let x2 = (x + w).minimum(self.width);
        let y2 = (y + h).minimum(self.height);
        
        for py in y1..y2 {
            let row_start = (py * self.width + x1) as usize;
            let row_end = (py * self.width + x2) as usize;
            if row_end <= self.buffer.len() {
                self.buffer[row_start..row_end].fill(color);
            }
        }
        self.dirty.store(true, Ordering::SeqCst);
    }
    
    /// Draw outlined rectangle
    pub fn draw_rect(&mut self, x: u32, y: u32, w: u32, h: u32, color: u32) {
        // Top & bottom
        self.fill_rect(x, y, w, 1, color);
        self.fill_rect(x, y + h.saturating_sub(1), w, 1, color);
        // Left & right
        self.fill_rect(x, y, 1, h, color);
        self.fill_rect(x + w.saturating_sub(1), y, 1, h, color);
    }
    
    /// Set pixel in layer
    #[inline]
        // Public function — callable from other modules.
pub fn set_pixel(&mut self, x: u32, y: u32, color: u32) {
        if x < self.width && y < self.height {
            let index = (y * self.width + x) as usize;
            if index < self.buffer.len() {
                self.buffer[index] = color;
            }
        }
    }
    
    /// Get pixel from layer (for reading)
    #[inline]
        // Public function — callable from other modules.
pub fn get_pixel(&self, x: u32, y: u32) -> u32 {
        if x < self.width && y < self.height {
            let index = (y * self.width + x) as usize;
            if index < self.buffer.len() {
                return self.buffer[index];
            }
        }
        0
    }
    
    /// Fill circle in layer
    pub fn fill_circle(&mut self, cx: u32, cy: u32, radius: u32, color: u32) {
        let r = radius as i32;
        let cx = cx as i32;
        let cy = cy as i32;
        
        for dy in -r..=r {
            for dx in -r..=r {
                if dx * dx + dy * dy <= r * r {
                    let pixel = cx + dx;
                    let py = cy + dy;
                    if pixel >= 0 && py >= 0 {
                        self.set_pixel(pixel as u32, py as u32, color);
                    }
                }
            }
        }
        self.dirty.store(true, Ordering::SeqCst);
    }
    
    /// Draw text in layer using framebuffer's font
    pub fn draw_text(&mut self, text: &str, x: u32, y: u32, color: u32) {
        let mut cx = x;
        for c in text.chars() {
            if c == ' ' {
                cx += 8;
                continue;
            }
            // Get glyph from font - returns [u8; 16] directly
            let glyph = crate::framebuffer::font::get_glyph(c);
            for (row, &bits) in glyph.iter().enumerate() {
                for column in 0..8 {
                    if bits & (0x80 >> column) != 0 {
                        self.set_pixel(cx + column, y + row as u32, color);
                    }
                }
            }
            cx += 8;
        }
        self.dirty.store(true, Ordering::SeqCst);
    }
}

/// The Compositor manages all layers and composites them
pub struct Compositor {
    layers: Vec<Layer>,
    screen_width: u32,
    screen_height: u32,
    composite_buffer: Box<[u32]>,
    /// GPU direct rendering: composite directly into GPU backing buffer
    /// Stored as usize to keep Compositor: Send (raw pointers are !Send)
    gpu_target_pointer: usize,  // 0 = use composite_buffer, nonzero = GPU buffer
    gpu_target_length: usize,
}

// Implementation block — defines methods for the type above.
impl Compositor {
    /// Create new compositor for given screen dimensions
    pub fn new(width: u32, height: u32) -> Self {
        let size = (width * height) as usize;
        Compositor {
            layers: Vec::new(),
            screen_width: width,
            screen_height: height,
            composite_buffer: vec![0u32; size].into_boxed_slice(),
            gpu_target_pointer: 0,
            gpu_target_length: 0,
        }
    }
    
    /// Add a layer to the compositor
    pub fn add_layer(&mut self, layer_type: LayerType, x: u32, y: u32, w: u32, h: u32) -> usize {
        let layer = Layer::new(layer_type, x, y, w, h);
        self.layers.push(layer);
        self.layers.len() - 1
    }
    
    /// Add a fullscreen layer
    pub fn add_fullscreen_layer(&mut self, layer_type: LayerType) -> usize {
        self.add_layer(layer_type, 0, 0, self.screen_width, self.screen_height)
    }
    
    /// Get mutable reference to a layer
    pub fn get_layer_mut(&mut self, index: usize) -> Option<&mut Layer> {
        self.layers.get_mut(index)
    }
    
    /// Get reference to a layer
    pub fn get_layer(&self, index: usize) -> Option<&Layer> {
        self.layers.get(index)
    }
    
    /// Enable GPU direct mode: composite directly into GPU backing buffer
    /// Eliminates the 4 MB present copy (composite_buffer → GPU buffer)
    pub fn enable_gpu_direct(&mut self) {
        if crate::drivers::virtio_gpu::is_available() {
            if let Some((ptr, w, h)) = crate::drivers::virtio_gpu::get_raw_buffer() {
                self.gpu_target_pointer = ptr as usize;
                self.gpu_target_length = (w * h) as usize;
                crate::serial_println!("[COMPOSITOR] GPU direct mode: composite → GPU buffer (skip 4MB copy!)");
            }
        }
    }
    
    /// Composite all visible layers into the target buffer
    /// When GPU direct mode is enabled, writes directly to GPU backing buffer
    /// Layers are drawn in order (first = bottom, last = top)
    /// OPTIMIZED: Uses fast path for opaque full-width layers
    pub fn composite(&mut self) {
        // Determine target buffer: GPU backing or own composite_buffer
        let (target_pointer, target_length) = if self.gpu_target_pointer != 0 {
            (self.gpu_target_pointer as *mut u32, self.gpu_target_length)
        } else {
            (self.composite_buffer.as_mut_pointer(), self.composite_buffer.len())
        };

        // Create sorted indices for rendering (don't modify the layers vector!)
        let mut render_order: Vec<usize> = (0..self.layers.len()).collect();
        render_order.sort_by_key(|&i| self.layers[i].layer_type as u8);
        
        // Check if the first (background) layer is full-screen opaque
        // If so, we can skip the initial fill since it will be overwritten
        let skip_initial_fill = if let Some(&first_index) = render_order.first() {
            let layer = &self.layers[first_index];
            layer.visible.load(Ordering::SeqCst) 
                && layer.layer_type == LayerType::Background
                && layer.x == 0 && layer.y == 0
                && layer.width >= self.screen_width
                && layer.height >= self.screen_height
                && layer.opacity.load(Ordering::SeqCst) == 255
        } else {
            false
        };
        
        // Only fill if necessary
        if !skip_initial_fill {
            // Use SSE2 for fast fill if available
            #[cfg(target_arch = "x86_64")]
                        // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe {
                crate::graphics::simd::fill_row_sse2(
                    target_pointer,
                    target_length,
                    0xFF000000
                );
            }
            #[cfg(not(target_arch = "x86_64"))]
                        // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe {
                for i in 0..target_length {
                    *target_pointer.add(i) = 0xFF000000;
                }
            }
        }
        
        // Composite each visible layer in z-order
        for &layer_index in &render_order {
            let layer = &self.layers[layer_index];
            if !layer.visible.load(Ordering::SeqCst) {
                continue;
            }
            
            let opacity = layer.opacity.load(Ordering::SeqCst);
            
            // FAST PATH: Full-width opaque layer - use parallel memcpy
            // Skip FAST PATH for taskbar to debug rendering issues
            if opacity == 255 && layer.x == 0 && layer.width == self.screen_width 
               && layer.layer_type == LayerType::Background {
                let source_height = layer.height.minimum(self.screen_height.saturating_sub(layer.y));
                
                // Use parallel_for for full-screen background copy (4MB → split across cores)
                let params = PresentParams {
                    source: layer.buffer.as_pointer(),
                    destination: target_pointer,
                    source_stride: layer.width as usize,
                    destination_stride: self.screen_width as usize,
                    width: layer.width as usize,
                    height: source_height as usize,
                };
                
                crate::cpu::smp::parallel_for(
                    source_height as usize,
                    present_rows_parallel,
                    &params as *// Compile-time constant — evaluated at compilation, zero runtime cost.
const PresentParams as *mut u8,
                );
                continue;
            }
            
            // MEDIUM PATH: Opaque layer - skip alpha blending per pixel
            if opacity == 255 {
                for ly in 0..layer.height {
                    let screen_y = layer.y + ly;
                    if screen_y >= self.screen_height {
                        continue;
                    }
                    
                    // Copy row at once if possible
                    let source_start = (ly * layer.width) as usize;
                    let destination_start = (screen_y * self.screen_width + layer.x) as usize;
                    let row_length = layer.width.minimum(self.screen_width - layer.x) as usize;
                    
                    if layer.x < self.screen_width 
                       && source_start + row_length <= layer.buffer.len()
                       && destination_start + row_length <= target_length {
                        // Fast row copy for opaque content
                        for i in 0..row_length {
                            let source_color = layer.buffer[source_start + i];
                            let source_alpha = (source_color >> 24) & 0xFF;
                            if source_alpha > 200 { // Mostly opaque
                                unsafe { *target_pointer.add(destination_start + i) = source_color; }
                            } else if source_alpha > 0 {
                                // Quick alpha blend
                                let destination_color = // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe { *target_pointer.add(destination_start + i) };
                                                                // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe { *target_pointer.add(destination_start + i) = alpha_blend(source_color, destination_color, source_alpha); }
                            }
                        }
                    }
                }
                continue;
            }
            
            // SLOW PATH: Alpha blending with layer opacity
            for ly in 0..layer.height {
                let screen_y = layer.y + ly;
                if screen_y >= self.screen_height {
                    continue;
                }
                
                for lx in 0..layer.width {
                    let screen_x = layer.x + lx;
                    if screen_x >= self.screen_width {
                        continue;
                    }
                    
                    let source_index = (ly * layer.width + lx) as usize;
                    let destination_index = (screen_y * self.screen_width + screen_x) as usize;
                    
                    if source_index >= layer.buffer.len() || destination_index >= target_length {
                        continue;
                    }
                    
                    let source_color = layer.buffer[source_index];
                    let source_alpha = ((source_color >> 24) & 0xFF) as u32;
                    
                    // Skip fully transparent pixels
                    if source_alpha == 0 {
                        continue;
                    }
                    
                    // Apply layer opacity
                    let final_alpha = (source_alpha * opacity) / 255;
                    
                    if final_alpha >= 255 {
                        // Fully opaque, just copy
                        unsafe { *target_pointer.add(destination_index) = source_color; }
                    } else if final_alpha > 0 {
                        // Alpha blend
                        let destination_color = // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe { *target_pointer.add(destination_index) };
                                                // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe { *target_pointer.add(destination_index) = alpha_blend(source_color, destination_color, final_alpha); }
                    }
                }
            }
        }
        
    }
    
    /// Copy composite buffer to the display
    /// Uses VirtIO GPU DMA path when available, falls back to MMIO framebuffer.
    /// When GPU direct mode is active, ALSO writes to MMIO framebuffer with
    /// non-temporal (streaming) stores for visibility on the primary VGA display.
    /// 
    /// Inspired by game engine render pipelines (id Tech, Godot, Bevy):
    /// - Non-temporal stores bypass cache → no pollution of render working set
    /// - Write-Combining memory type batches stores into 64-byte bursts
    /// - GPU DMA transfer runs in parallel with CPU stores
    pub fn present(&self) {
        // ── GPU direct mode: composite() already wrote to GPU buffer ──
        if self.gpu_target_pointer != 0 {
            // Trigger DMA transfer + flush (GPU display)
            let _ = crate::drivers::virtio_gpu::present_frame();
            // Also write to MMIO framebuffer for VGA display visibility
            self.writeback_mmio_nt();
            return;
        }
        
        // ── VirtIO GPU path (non-direct): copy composite_buffer → GPU buffer ──
        if crate::drivers::virtio_gpu::is_available() {
            if let Some((gpu_pointer, gpu_w, gpu_h)) = crate::drivers::virtio_gpu::get_raw_buffer() {
                let copy_w = (self.screen_width as usize).minimum(gpu_w as usize);
                let copy_h = (self.screen_height as usize).minimum(gpu_h as usize);
                
                                // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe {
                    let source_base = self.composite_buffer.as_pointer();
                    let destination_base = gpu_pointer;
                    
                    for y in 0..copy_h {
                        let source = source_base.add(y * self.screen_width as usize);
                        let destination = destination_base.add(y * gpu_w as usize);
                        
                        #[cfg(target_arch = "x86_64")]
                        crate::graphics::simd::copy_row_sse2(destination, source, copy_w);
                        #[cfg(not(target_arch = "x86_64"))]
                        core::ptr::copy_nonoverlapping(source, destination, copy_w);
                    }
                }
                
                let _ = crate::drivers::virtio_gpu::present_frame();
                return;
            }
        }
        
        // ── MMIO framebuffer fallback (only when no GPU) ──
        // Uses non-temporal stores + WC memory type for maximum throughput
        self.writeback_mmio_nt();
    }
    
    /// Present-only: NO-OP for frame-rate decoupling.
    /// Skip frames don't re-transfer the unchanged buffer through VirtIO DMA.
    /// The display keeps showing the last presented frame.
    /// This is the key optimization: skip the 4MB transfer_to_host_2d + resource_flush
    /// that was bottlenecking every frame at ~33ms.
    pub fn present_only(&self) {
        // Intentionally empty — no redundant DMA transfer
        // The last present_frame() result is still displayed on screen
    }
    
    /// Write frame to MMIO framebuffer using non-temporal (streaming) SSE2 stores.
    /// Source is either the GPU buffer (gpu_target_ptr) or the composite_buffer.
    /// 
    /// Non-temporal stores (`movntdq`) are standard in GPU drivers and game engines:
    /// - Skip cache hierarchy entirely (writes go directly to write-combine buffer)
    /// - Combined with WC memory type: individual stores merge into 64-byte bursts
    /// - Prevents cache pollution: rendering working set stays hot in L1/L2
    fn writeback_mmio_nt(&self) {
        use crate::framebuffer::{FRAMEBUFFER_ADDRESS, FRAMEBUFFER_WIDTH, FRAMEBUFFER_HEIGHT, FRAMEBUFFER_PITCH};
        
        let address = FRAMEBUFFER_ADDRESS.load(Ordering::SeqCst);
        if address.is_null() { return; }
        
        let framebuffer_width = FRAMEBUFFER_WIDTH.load(Ordering::SeqCst) as usize;
        let framebuffer_height = FRAMEBUFFER_HEIGHT.load(Ordering::SeqCst) as usize;
        let pitch = FRAMEBUFFER_PITCH.load(Ordering::SeqCst) as usize;
        let pitch_pixels = pitch / 4;
        
        let copy_width = framebuffer_width.minimum(self.screen_width as usize);
        let copy_height = framebuffer_height.minimum(self.screen_height as usize);
        
        // Choose source: GPU buffer if direct mode, else composite_buffer
        let source_base = if self.gpu_target_pointer != 0 {
            self.gpu_target_pointer as *// Compile-time constant — evaluated at compilation, zero runtime cost.
const u32
        } else {
            self.composite_buffer.as_pointer()
        };
        
        // Parallel NT writeback across all cores
        let params = PresentParams {
            source: source_base,
            destination: address as *mut u32,
            source_stride: self.screen_width as usize,
            destination_stride: pitch_pixels,
            width: copy_width,
            height: copy_height,
        };
        
        crate::cpu::smp::parallel_for(
            copy_height,
            writeback_rows_parallel,
            &params as *// Compile-time constant — evaluated at compilation, zero runtime cost.
const PresentParams as *mut u8,
        );
    }
    
    /// Get layer count
    pub fn layer_count(&self) -> usize {
        self.layers.len()
    }
}

/// Alpha blend two colors
#[inline]
fn alpha_blend(source: u32, destination: u32, alpha: u32) -> u32 {
    let inv_alpha = 255 - alpha;
    
    let sr = (source >> 16) & 0xFF;
    let sg = (source >> 8) & 0xFF;
    let sb = source & 0xFF;
    
    let dr = (destination >> 16) & 0xFF;
    let dg = (destination >> 8) & 0xFF;
    let db = destination & 0xFF;
    
    let r = (sr * alpha + dr * inv_alpha) / 255;
    let g = (sg * alpha + dg * inv_alpha) / 255;
    let b = (sb * alpha + db * inv_alpha) / 255;
    
    0xFF000000 | (r << 16) | (g << 8) | b
}

// ============================================================
// Global compositor instance
// ============================================================
static COMPOSITOR: Mutex<Option<Compositor>> = Mutex::new(None);

/// Initialize the global compositor
pub fn init(width: u32, height: u32) {
    let compositor = Compositor::new(width, height);
    *COMPOSITOR.lock() = Some(compositor);
    crate::serial_println!("[COMPOSITOR] Initialized {}x{}", width, height);
}

/// Get mutable access to global compositor
pub fn with_compositor<F, R>(f: F) -> Option<R>
where
    F: FnOnce(&mut Compositor) -> R,
{
    COMPOSITOR.lock().as_mut().map(f)
}

/// Get read-only access to compositor
pub fn with_compositor_ref<F, R>(f: F) -> Option<R>
where
    F: FnOnce(&Compositor) -> R,
{
    COMPOSITOR.lock().as_ref().map(f)
}
