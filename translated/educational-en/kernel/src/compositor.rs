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
    pub src: *// Compile-time constant — evaluated at compilation, zero runtime cost.
const u32,
    pub dst: *mut u32,
    pub src_stride: usize,
    pub dst_stride: usize,
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
        
        let source_offset = y * params.src_stride;
        let destination_offset = y * params.dst_stride;
        
                // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe {
            let src = params.src.add(source_offset);
            let dst = params.dst.add(destination_offset);
            
            #[cfg(target_arch = "x86_64")]
            crate::graphics::simd::copy_row_sse2(dst, src, params.width);
            #[cfg(not(target_arch = "x86_64"))]
            core::ptr::copy_nonoverlapping(src, dst, params.width);
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
        
        let source_offset = y * params.src_stride;
        let destination_offset = y * params.dst_stride;
        
                // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe {
            let src = params.src.add(source_offset);
            let dst = params.dst.add(destination_offset);
            
            #[cfg(target_arch = "x86_64")]
            crate::graphics::simd::copy_row_sse2(dst, src, params.width);
            #[cfg(not(target_arch = "x86_64"))]
            core::ptr::copy_nonoverlapping(src, dst, params.width);
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
        let x1 = x.min(self.width);
        let y1 = y.min(self.height);
        let x2 = (x + w).min(self.width);
        let y2 = (y + h).min(self.height);
        
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
            let idx = (y * self.width + x) as usize;
            if idx < self.buffer.len() {
                self.buffer[idx] = color;
            }
        }
    }
    
    /// Get pixel from layer (for reading)
    #[inline]
        // Public function — callable from other modules.
pub fn get_pixel(&self, x: u32, y: u32) -> u32 {
        if x < self.width && y < self.height {
            let idx = (y * self.width + x) as usize;
            if idx < self.buffer.len() {
                return self.buffer[idx];
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
                    let px = cx + dx;
                    let py = cy + dy;
                    if px >= 0 && py >= 0 {
                        self.set_pixel(px as u32, py as u32, color);
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
                for col in 0..8 {
                    if bits & (0x80 >> col) != 0 {
                        self.set_pixel(cx + col, y + row as u32, color);
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
    gpu_target_ptr: usize,  // 0 = use composite_buffer, nonzero = GPU buffer
    gpu_target_len: usize,
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
            gpu_target_ptr: 0,
            gpu_target_len: 0,
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
                self.gpu_target_ptr = ptr as usize;
                self.gpu_target_len = (w * h) as usize;
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
        let (target_ptr, target_length) = if self.gpu_target_ptr != 0 {
            (self.gpu_target_ptr as *mut u32, self.gpu_target_len)
        } else {
            (self.composite_buffer.as_mut_ptr(), self.composite_buffer.len())
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
                    target_ptr,
                    target_length,
                    0xFF000000
                );
            }
            #[cfg(not(target_arch = "x86_64"))]
                        // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe {
                for i in 0..target_length {
                    *target_ptr.add(i) = 0xFF000000;
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
                let source_height = layer.height.min(self.screen_height.saturating_sub(layer.y));
                
                // Use parallel_for for full-screen background copy (4MB → split across cores)
                let params = PresentParams {
                    src: layer.buffer.as_ptr(),
                    dst: target_ptr,
                    src_stride: layer.width as usize,
                    dst_stride: self.screen_width as usize,
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
            
            // MEDIUM PATH: Opaque layer - SSE2 row-level alpha blend
            if opacity == 255 {
                for ly in 0..layer.height {
                    let screen_y = layer.y + ly;
                    if screen_y >= self.screen_height {
                        continue;
                    }
                    
                    let source_start = (ly * layer.width) as usize;
                    let destination_start = (screen_y * self.screen_width + layer.x) as usize;
                    let row_length = layer.width.min(self.screen_width - layer.x) as usize;
                    
                    if layer.x < self.screen_width 
                       && source_start + row_length <= layer.buffer.len()
                       && destination_start + row_length <= target_length {
                        // SSE2 alpha blend: 4 pixels at a time with fast-skip for transparent/opaque
                        #[cfg(target_arch = "x86_64")]
                                                // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe {
                            crate::graphics::simd::blend_row_sse2(
                                target_ptr.add(destination_start),
                                layer.buffer.as_ptr().add(source_start),
                                row_length,
                            );
                        }
                        #[cfg(not(target_arch = "x86_64"))]
                        {
                            for i in 0..row_length {
                                let source_color = layer.buffer[source_start + i];
                                let source_alpha = (source_color >> 24) & 0xFF;
                                if source_alpha > 200 {
                                                                        // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe { *target_ptr.add(destination_start + i) = source_color; }
                                } else if source_alpha > 0 {
                                    let destination_color = // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe { *target_ptr.add(destination_start + i) };
                                                                        // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe { *target_ptr.add(destination_start + i) = alpha_blend(source_color, destination_color, source_alpha); }
                                }
                            }
                        }
                    }
                }
                continue;
            }
            
            // SLOW PATH: Alpha blending with layer opacity — row-level SIMD
            for ly in 0..layer.height {
                let screen_y = layer.y + ly;
                if screen_y >= self.screen_height {
                    continue;
                }
                
                let source_start = (ly * layer.width) as usize;
                let row_length = layer.width.min(self.screen_width.saturating_sub(layer.x)) as usize;
                let destination_start = (screen_y * self.screen_width + layer.x) as usize;
                
                if layer.x >= self.screen_width 
                   || source_start + row_length > layer.buffer.len()
                   || destination_start + row_length > target_length {
                    continue;
                }
                
                // Pre-multiply source alpha by layer opacity, then SSE2 blend entire row
                #[cfg(target_arch = "x86_64")]
                                // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe {
                    blend_row_with_opacity(
                        target_ptr.add(destination_start),
                        layer.buffer.as_ptr().add(source_start),
                        row_length,
                        opacity,
                    );
                }
                #[cfg(not(target_arch = "x86_64"))]
                {
                    for i in 0..row_length {
                        let source_color = layer.buffer[source_start + i];
                        let source_alpha = ((source_color >> 24) & 0xFF) as u32;
                        if source_alpha == 0 { continue; }
                        let final_alpha = (source_alpha * opacity) / 255;
                        if final_alpha >= 255 {
                                                        // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe { *target_ptr.add(destination_start + i) = source_color; }
                        } else if final_alpha > 0 {
                            let destination_color = // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe { *target_ptr.add(destination_start + i) };
                                                        // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe { *target_ptr.add(destination_start + i) = alpha_blend(source_color, destination_color, final_alpha); }
                        }
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
        if self.gpu_target_ptr != 0 {
            // Trigger DMA transfer + flush (GPU display)
            let _ = crate::drivers::virtio_gpu::present_frame();
            // Skip MMIO writeback — GPU DMA handles display.
            // The old writeback_mmio_nt() here was a redundant 4MB NT copy every frame.
            return;
        }
        
        // ── VirtIO GPU path (non-direct): copy composite_buffer → GPU buffer ──
        if crate::drivers::virtio_gpu::is_available() {
            if let Some((gpu_ptr, gpu_w, gpu_h)) = crate::drivers::virtio_gpu::get_raw_buffer() {
                let copy_w = (self.screen_width as usize).min(gpu_w as usize);
                let copy_h = (self.screen_height as usize).min(gpu_h as usize);
                
                                // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe {
                    let source_base = self.composite_buffer.as_ptr();
                    let destination_base = gpu_ptr;
                    
                    for y in 0..copy_h {
                        let src = source_base.add(y * self.screen_width as usize);
                        let dst = destination_base.add(y * gpu_w as usize);
                        
                        #[cfg(target_arch = "x86_64")]
                        crate::graphics::simd::copy_row_sse2(dst, src, copy_w);
                        #[cfg(not(target_arch = "x86_64"))]
                        core::ptr::copy_nonoverlapping(src, dst, copy_w);
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
        
        let addr = FRAMEBUFFER_ADDRESS.load(Ordering::SeqCst);
        if addr.is_null() { return; }
        
        let fb_width = FRAMEBUFFER_WIDTH.load(Ordering::SeqCst) as usize;
        let fb_height = FRAMEBUFFER_HEIGHT.load(Ordering::SeqCst) as usize;
        let pitch = FRAMEBUFFER_PITCH.load(Ordering::SeqCst) as usize;
        let pitch_pixels = pitch / 4;
        
        let copy_width = fb_width.min(self.screen_width as usize);
        let copy_height = fb_height.min(self.screen_height as usize);
        
        // Choose source: GPU buffer if direct mode, else composite_buffer
        let source_base = if self.gpu_target_ptr != 0 {
            self.gpu_target_ptr as *// Compile-time constant — evaluated at compilation, zero runtime cost.
const u32
        } else {
            self.composite_buffer.as_ptr()
        };
        
        // Parallel NT writeback across all cores
        let params = PresentParams {
            src: source_base,
            dst: addr as *mut u32,
            src_stride: self.screen_width as usize,
            dst_stride: pitch_pixels,
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

/// Alpha blend two colors — uses >>8 approximation (matches SIMD path)
#[inline(always)]
fn alpha_blend(src: u32, dst: u32, alpha: u32) -> u32 {
    let inv_alpha = 255 - alpha;
    
    let sr = (src >> 16) & 0xFF;
    let sg = (src >> 8) & 0xFF;
    let sb = src & 0xFF;
    
    let dr = (dst >> 16) & 0xFF;
    let dg = (dst >> 8) & 0xFF;
    let db = dst & 0xFF;
    
    let r = (sr * alpha + dr * inv_alpha + 128) >> 8;
    let g = (sg * alpha + dg * inv_alpha + 128) >> 8;
    let b = (sb * alpha + db * inv_alpha + 128) >> 8;
    
    0xFF000000 | (r << 16) | (g << 8) | b
}

/// Blend a row of src pixels onto dst, pre-multiplying each pixel's alpha by a uniform layer opacity.
/// Uses SSE2 for 4-pixel-at-a-time processing.
#[cfg(target_arch = "x86_64")]
#[inline]
// SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe fn blend_row_with_opacity(dst: *mut u32, src: *// Compile-time constant — evaluated at compilation, zero runtime cost.
const u32, count: usize, opacity: u32) {
    use core::arch::x86_64::*;
    
    let mut destination_pointer = dst;
    let mut source_pointer = src;
    let mut remaining = count;
    
    let zero = _mm_setzero_si128();
    let opacity_vec = _mm_set1_epi16(opacity as i16);
    let round_vec = _mm_set1_epi16(128);
    let alpha_mask = _mm_set1_epi32(0xFF000000u32 as i32);
    let max255 = _mm_set1_epi16(255);
    
    while remaining >= 4 {
        let s = _mm_loadu_si128(source_pointer as *// Compile-time constant — evaluated at compilation, zero runtime cost.
const __m128i);
        
        // Quick check: all transparent → skip
        let s_alpha = _mm_srli_epi32(s, 24);
        let all_zero = _mm_cmpeq_epi32(s_alpha, zero);
        if _mm_movemask_epi8(all_zero) == 0xFFFF {
            source_pointer = source_pointer.add(4);
            destination_pointer = destination_pointer.add(4);
            remaining -= 4;
            continue;
        }
        
        let d = _mm_loadu_si128(destination_pointer as *// Compile-time constant — evaluated at compilation, zero runtime cost.
const __m128i);
        
        // --- Pixels 0-1 ---
        let s_lo = _mm_unpacklo_epi8(s, zero);
        let d_lo = _mm_unpacklo_epi8(d, zero);
        
        // Extract per-pixel alpha, multiply by layer opacity: final_alpha = (src_alpha * opacity) >> 8
        let a0 = _mm_shufflelo_epi16(s_lo, 0xFF);
        let a_lo = _mm_shufflehi_epi16(a0, 0xFF);
        let fa_lo = _mm_srli_epi16(_mm_add_epi16(_mm_mullo_epi16(a_lo, opacity_vec), round_vec), 8);
        let inv_fa_lo = _mm_sub_epi16(max255, fa_lo);
        
        let source_mul = _mm_mullo_epi16(s_lo, fa_lo);
        let destination_mul = _mm_mullo_epi16(d_lo, inv_fa_lo);
        let sum_lo = _mm_srli_epi16(_mm_add_epi16(_mm_add_epi16(source_mul, destination_mul), round_vec), 8);
        
        // --- Pixels 2-3 ---
        let s_hi = _mm_unpackhi_epi8(s, zero);
        let d_hi = _mm_unpackhi_epi8(d, zero);
        
        let a2 = _mm_shufflelo_epi16(s_hi, 0xFF);
        let a_hi = _mm_shufflehi_epi16(a2, 0xFF);
        let fa_hi = _mm_srli_epi16(_mm_add_epi16(_mm_mullo_epi16(a_hi, opacity_vec), round_vec), 8);
        let inv_fa_hi = _mm_sub_epi16(max255, fa_hi);
        
        let source_mul_hi = _mm_mullo_epi16(s_hi, fa_hi);
        let destination_mul_hi = _mm_mullo_epi16(d_hi, inv_fa_hi);
        let sum_hi = _mm_srli_epi16(_mm_add_epi16(_mm_add_epi16(source_mul_hi, destination_mul_hi), round_vec), 8);
        
        // Pack and force opaque output
        let result = _mm_packus_epi16(sum_lo, sum_hi);
        let result = _mm_or_si128(result, alpha_mask);
        _mm_storeu_si128(destination_pointer as *mut __m128i, result);
        
        source_pointer = source_pointer.add(4);
        destination_pointer = destination_pointer.add(4);
        remaining -= 4;
    }
    
    // Scalar tail
    for _ in 0..remaining {
        let source_color = *source_pointer;
        let source_alpha = ((source_color >> 24) & 0xFF) as u32;
        if source_alpha > 0 {
            let final_alpha = (source_alpha * opacity + 128) >> 8;
            if final_alpha >= 255 {
                *destination_pointer = source_color;
            } else if final_alpha > 0 {
                *destination_pointer = alpha_blend(source_color, *destination_pointer, final_alpha);
            }
        }
        source_pointer = source_pointer.add(1);
        destination_pointer = destination_pointer.add(1);
    }
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
