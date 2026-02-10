// TrustVideo Player — renders decoded frames to framebuffer
// Supports: playback, pause, frame-by-frame step, loop
// Uses double-buffered framebuffer for smooth playback

use alloc::vec::Vec;
use alloc::vec;
use alloc::string::String;
use alloc::format;
use super::codec::{TvDecoder, TvEncoder};

/// Playback state
#[derive(Clone, Copy, PartialEq)]
pub enum PlayState {
    Playing,
    Paused,
    Stopped,
}

pub struct VideoPlayer {
    state: PlayState,
    pub loop_playback: bool,
}

impl VideoPlayer {
    pub fn new() -> Self {
        Self {
            state: PlayState::Stopped,
            loop_playback: false,
        }
    }

    /// Play a .tv file from raw bytes — renders to framebuffer
    pub fn play_data(&mut self, data: Vec<u8>) -> Result<String, String> {
        let mut decoder = TvDecoder::new(data)
            .ok_or_else(|| String::from("Invalid TrustVideo file"))?;

        let vw = decoder.header.width as u32;
        let vh = decoder.header.height as u32;
        let fps = decoder.header.fps as u64;
        let total = decoder.header.frame_count;
        let frame_ms = if fps > 0 { 1000 / fps } else { 33 };

        let sw = crate::framebuffer::width();
        let sh = crate::framebuffer::height();

        // Center video on screen
        let ox = if sw > vw { (sw - vw) / 2 } else { 0 };
        let oy = if sh > vh { (sh - vh) / 2 } else { 0 };

        self.state = PlayState::Playing;
        let mut frames_played: u32 = 0;

        // Clear screen
        crate::framebuffer::clear_backbuffer(0xFF000000);
        crate::framebuffer::swap_buffers();

        crate::serial_println!("[video] Playing {}x{} @ {}fps, {} frames", vw, vh, fps, total);

        loop {
            // Check for ESC key to stop
            if let Some(key) = crate::keyboard::try_read_key() {
                if key == 0x1B || key == b'q' {
                    // ESC or Q
                    self.state = PlayState::Stopped;
                    break;
                } else if key == b' ' {
                    // Space = pause/resume
                    self.state = if self.state == PlayState::Playing {
                        PlayState::Paused
                    } else {
                        PlayState::Playing
                    };
                }
            }

            if self.state == PlayState::Paused {
                // Spin while paused
                let pause_start = crate::time::uptime_ms();
                while crate::time::uptime_ms() < pause_start + 50 {
                    core::hint::spin_loop();
                }
                continue;
            }

            let frame_start = crate::time::uptime_ms();

            if let Some(pixels) = decoder.next_frame() {
                // Blit frame to backbuffer
                Self::blit_frame(pixels, vw, vh, ox, oy, sw);

                // Draw simple HUD
                Self::draw_hud(frames_played, total, fps as u32, sw, sh);

                crate::framebuffer::swap_buffers();
                frames_played += 1;
            } else if self.loop_playback {
                decoder.rewind();
                continue;
            } else {
                break;
            }

            // Frame timing
            let elapsed = crate::time::uptime_ms() - frame_start;
            if elapsed < frame_ms {
                let wait = frame_ms - elapsed;
                let end = crate::time::uptime_ms() + wait;
                while crate::time::uptime_ms() < end {
                    core::hint::spin_loop();
                }
            }
        }

        self.state = PlayState::Stopped;
        Ok(format!("Played {} frames", frames_played))
    }

    /// Blit decoded pixel buffer to framebuffer backbuffer
    fn blit_frame(pixels: &[u32], vw: u32, vh: u32, ox: u32, oy: u32, sw: u32) {
        let ctx = crate::framebuffer::FastPixelContext::new();
        let sh = crate::framebuffer::height();
        for y in 0..vh {
            let dy = oy + y;
            if dy >= sh { break; }
            let row_start = (y * vw) as usize;
            for x in 0..vw {
                let dx = ox + x;
                if dx >= sw { break; }
                let px = pixels[row_start + x as usize];
                ctx.put_pixel(dx as usize, dy as usize, px);
            }
        }
    }

    /// Draw minimal playback HUD (frame counter)
    fn draw_hud(current: u32, total: u32, fps: u32, sw: u32, sh: u32) {
        // Dark bar at bottom
        crate::framebuffer::fill_rect(0, sh - 20, sw, 20, 0xCC000000);

        // Progress bar
        let bar_w = sw - 20;
        let progress = if total > 0 {
            ((current as u64 * bar_w as u64) / total as u64) as u32
        } else { 0 };
        crate::framebuffer::fill_rect(10, sh - 14, bar_w, 8, 0xFF333333);
        crate::framebuffer::fill_rect(10, sh - 14, progress, 8, 0xFF00AAFF);
    }
}

// ── Procedural demo animations ──

/// Generate a plasma animation as TrustVideo data
pub fn generate_plasma_demo(width: u16, height: u16, frames: u32, fps: u16) -> Vec<u8> {
    let mut encoder = TvEncoder::new(width, height, fps);
    let npix = width as usize * height as usize;
    let mut buf = vec![0u32; npix];
    let w = width as usize;
    let h = height as usize;

    for f in 0..frames {
        let t = f as i32;
        for y in 0..h {
            for x in 0..w {
                let fx = (x * 256 / w) as i32;
                let fy = (y * 256 / h) as i32;
                let v1 = isin((fx.wrapping_mul(3).wrapping_add(t * 4)) as u8);
                let v2 = isin((fy.wrapping_mul(2).wrapping_add(t * 5)) as u8);
                let v3 = isin((fx.wrapping_add(fy).wrapping_mul(2).wrapping_add(t * 3)) as u8);
                let v4 = isin((fx.wrapping_sub(fy).wrapping_add(t * 7)) as u8);
                let avg = (v1 as i32 + v2 as i32 + v3 as i32 + v4 as i32) / 4;
                let hue = ((avg + 128) & 0xFF) as u8;
                let (r, g, b) = hue_to_rgb_int(hue);
                buf[y * w + x] = 0xFF000000 | ((r as u32) << 16) | ((g as u32) << 8) | b as u32;
            }
        }
        encoder.add_frame(&buf);
        if f % 10 == 0 {
            crate::serial_println!("[video] Encoding frame {}/{}", f + 1, frames);
        }
    }

    encoder.finalize()
}

/// Generate a fire effect animation
pub fn generate_fire_demo(width: u16, height: u16, frames: u32, fps: u16) -> Vec<u8> {
    let mut encoder = TvEncoder::new(width, height, fps);
    let w = width as usize;
    let h = height as usize;
    let npix = w * h;
    let mut heat = vec![0u8; npix]; // heat map
    let mut buf = vec![0u32; npix];
    let mut seed: u32 = 42;

    for f in 0..frames {
        // Random heat at bottom row
        for x in 0..w {
            seed = xorshift(seed);
            heat[(h - 1) * w + x] = (seed & 0xFF) as u8;
            // Extra intensity at bottom
            seed = xorshift(seed);
            heat[(h - 2) * w + x] = ((seed & 0xFF) as u16).min(255) as u8;
        }

        // Propagate heat upward with cooling
        for y in 0..h - 2 {
            for x in 0..w {
                let below = heat[(y + 1) * w + x] as u16;
                let bl = if x > 0 { heat[(y + 1) * w + x - 1] as u16 } else { below };
                let br = if x + 1 < w { heat[(y + 1) * w + x + 1] as u16 } else { below };
                let bb = heat[((y + 2).min(h - 1)) * w + x] as u16;
                let avg = (below + bl + br + bb) / 4;
                let cool = if avg > 2 { avg - 2 } else { 0 };
                heat[y * w + x] = cool.min(255) as u8;
            }
        }

        // Heat map → color (black → red → yellow → white)
        for i in 0..npix {
            let t = heat[i];
            let (r, g, b) = if t < 64 {
                (t * 4, 0u8, 0u8) // black → red
            } else if t < 128 {
                (255, (t - 64) * 4, 0u8) // red → yellow
            } else if t < 192 {
                (255, 255, (t - 128) * 4) // yellow → white
            } else {
                (255, 255, 255) // white
            };
            buf[i] = 0xFF000000 | ((r as u32) << 16) | ((g as u32) << 8) | b as u32;
        }

        encoder.add_frame(&buf);
        if f % 10 == 0 {
            crate::serial_println!("[video] Fire frame {}/{}", f + 1, frames);
        }
    }

    encoder.finalize()
}

/// Generate a matrix rain demo
pub fn generate_matrix_demo(width: u16, height: u16, frames: u32, fps: u16) -> Vec<u8> {
    let mut encoder = TvEncoder::new(width, height, fps);
    let w = width as usize;
    let h = height as usize;
    let npix = w * h;
    let mut buf = vec![0u32; npix];

    let col_w = 8; // pixel width of each "column"
    let ncols = w / col_w + 1;
    let mut drops = vec![0i32; ncols];
    let mut speeds = vec![0u8; ncols];
    let mut seed: u32 = 1337;

    // Init random drop positions and speeds
    for i in 0..ncols {
        seed = xorshift(seed);
        drops[i] = -((seed % h as u32) as i32);
        seed = xorshift(seed);
        speeds[i] = 1 + (seed % 4) as u8;
    }

    for f in 0..frames {
        // Fade existing pixels (darken by ~10%)
        for i in 0..npix {
            let px = buf[i];
            let r = ((px >> 16) & 0xFF) * 92 / 100;
            let g = ((px >> 8) & 0xFF) * 92 / 100;
            let b = (px & 0xFF) * 92 / 100;
            buf[i] = 0xFF000000 | (r << 16) | (g << 8) | b;
        }

        // Draw drops
        for c in 0..ncols {
            let x_base = c * col_w;
            let dy = drops[c];

            if dy >= 0 && (dy as usize) < h {
                let y = dy as usize;
                // Head of drop: bright white-green
                for px in 0..col_w.min(w - x_base) {
                    buf[y * w + x_base + px] = 0xFFCCFFCC;
                }
                // Trail: green
                for trail in 1..8u32 {
                    let ty = dy - trail as i32;
                    if ty >= 0 && (ty as usize) < h {
                        let intensity = 200 - trail * 20;
                        let g = intensity.min(255);
                        for px in 0..col_w.min(w - x_base) {
                            buf[ty as usize * w + x_base + px] =
                                0xFF000000 | ((g / 4) << 16) | (g << 8) | (g / 4);
                        }
                    }
                }
            }

            drops[c] += speeds[c] as i32;

            // Reset when off screen
            if drops[c] > (h as i32 + 20) {
                seed = xorshift(seed);
                drops[c] = -((seed % (h as u32 / 2)) as i32);
                seed = xorshift(seed);
                speeds[c] = 1 + (seed % 4) as u8;
            }
        }

        encoder.add_frame(&buf);
        if f % 10 == 0 {
            crate::serial_println!("[video] Matrix frame {}/{}", f + 1, frames);
        }
    }

    encoder.finalize()
}

// ── Math helpers (no libm) ──

/// Fast sin approximation using Taylor series (good enough for visuals)
/// xorshift32 PRNG
fn xorshift(mut x: u32) -> u32 {
    if x == 0 { x = 1; }
    x ^= x << 13;
    x ^= x >> 17;
    x ^= x << 5;
    x
}

// ── Real-time streaming renderer ──
// Generates and displays each frame directly — no file accumulation

/// Render an effect in real-time to the framebuffer (infinite loop until Q/ESC)
pub fn render_realtime(effect: &str, width: u16, height: u16, fps: u16) {
    let frame_ms = if fps > 0 { 1000u64 / fps as u64 } else { 33 };

    let sw = crate::framebuffer::width();
    let sh = crate::framebuffer::height();
    let rw = (width as u32).min(sw) as usize;
    let rh = (height as u32).min(sh) as usize;
    let ox = if sw > rw as u32 { (sw - rw as u32) / 2 } else { 0 } as usize;
    let oy = if sh > rh as u32 { (sh - rh as u32) / 2 } else { 0 } as usize;

    let rnpix = rw * rh;
    let mut buf = vec![0u32; rnpix];
    let mut frame: u32 = 0;
    let mut seed: u32 = 42;

    // State for fire effect
    let mut heat = if effect == "fire" { vec![0u8; rnpix] } else { Vec::new() };

    // State for matrix effect
    let col_w: usize = 8;
    let ncols = rw / col_w + 1;
    let mut drops = vec![0i32; ncols];
    let mut speeds = vec![0u8; ncols];
    if effect == "matrix" {
        for i in 0..ncols {
            seed = xorshift(seed);
            drops[i] = -((seed % rh as u32) as i32);
            seed = xorshift(seed);
            speeds[i] = 1 + (seed % 4) as u8;
        }
    }

    // Enable double buffering: write to RAM backbuffer, then swap to MMIO once
    let was_db = crate::framebuffer::is_double_buffer_enabled();
    if !was_db {
        crate::framebuffer::init_double_buffer();
        crate::framebuffer::set_double_buffer_mode(true);
    }

    // Clear screen via backbuffer (SSE2 in RAM — instant)
    crate::framebuffer::clear_backbuffer(0xFF000000);
    crate::framebuffer::swap_buffers();

    crate::serial_println!("[video] Starting {} render loop ({}x{} centered on {}x{}, backbuffer={})", 
        effect, rw, rh, sw, sh, crate::framebuffer::is_double_buffer_enabled());

    loop {
        let frame_start = crate::time::uptime_ms();

        // Check for Q/ESC
        if let Some(key) = crate::keyboard::try_read_key() {
            if key == 0x1B || key == b'q' { break; }
        }

        // Generate frame data in local RAM buffer (pure computation, very fast)
        match effect {
            "plasma" => render_plasma_frame(&mut buf, rw, rh, frame),
            "fire" => render_fire_frame(&mut buf, &mut heat, rw, rh, &mut seed),
            "matrix" => render_matrix_frame(&mut buf, rw, rh, &mut drops, &mut speeds, &mut seed, col_w, ncols),
            _ => break,
        }

        // Blit to backbuffer (RAM→RAM copy, very fast) then swap once to MMIO
        if let Some((bb_ptr, _bb_w, bb_h, bb_stride)) = crate::framebuffer::get_backbuffer_info() {
            let bb = bb_ptr as *mut u32;
            let bb_s = bb_stride as usize;
            for y in 0..rh {
                let dy = oy + y;
                if dy >= bb_h as usize { break; }
                let src_row = &buf[y * rw..y * rw + rw];
                unsafe {
                    let dst = bb.add(dy * bb_s + ox);
                    core::ptr::copy_nonoverlapping(src_row.as_ptr(), dst, rw);
                }
            }
        }
        crate::framebuffer::swap_buffers();

        frame = frame.wrapping_add(1);

        // Log frame timing (no busy-wait — interrupts may not fire during spin_loop)
        if frame <= 3 || frame % 60 == 0 {
            crate::serial_println!("[video] frame {} rendered", frame);
        }
    }

    // Restore double buffer state
    if !was_db {
        crate::framebuffer::set_double_buffer_mode(false);
    }

    crate::serial_println!("[video] Stopped after {} frames", frame);
}

/// Timed version of render_realtime — auto-stops after `duration_ms` milliseconds.
/// Used by the showcase command for automated demo sequences.
pub fn render_realtime_timed(effect: &str, width: u16, height: u16, fps: u16, duration_ms: u64) {
    let sw = crate::framebuffer::width();
    let sh = crate::framebuffer::height();
    let rw = (width as u32).min(sw) as usize;
    let rh = (height as u32).min(sh) as usize;
    let ox = if sw > rw as u32 { (sw - rw as u32) / 2 } else { 0 } as usize;
    let oy = if sh > rh as u32 { (sh - rh as u32) / 2 } else { 0 } as usize;

    let rnpix = rw * rh;
    let mut buf = vec![0u32; rnpix];
    let mut frame: u32 = 0;
    let mut seed: u32 = 42;

    let mut heat = if effect == "fire" { vec![0u8; rnpix] } else { Vec::new() };

    let col_w: usize = 8;
    let ncols = rw / col_w + 1;
    let mut drops = vec![0i32; ncols];
    let mut speeds = vec![0u8; ncols];
    if effect == "matrix" {
        for i in 0..ncols {
            seed = xorshift(seed);
            drops[i] = -((seed % rh as u32) as i32);
            seed = xorshift(seed);
            speeds[i] = 1 + (seed % 4) as u8;
        }
    }

    let was_db = crate::framebuffer::is_double_buffer_enabled();
    if !was_db {
        crate::framebuffer::init_double_buffer();
        crate::framebuffer::set_double_buffer_mode(true);
    }

    crate::framebuffer::clear_backbuffer(0xFF000000);
    crate::framebuffer::swap_buffers();

    // Use TSC for timing — uptime_ms() doesn't advance during spin_loop()
    let start_tsc = crate::cpu::tsc::read_tsc();
    let freq = crate::cpu::tsc::frequency_hz();
    let target_cycles = if freq > 0 { freq / 1000 * duration_ms } else { u64::MAX };

    loop {
        // Auto-stop after duration (TSC-based)
        let elapsed = crate::cpu::tsc::read_tsc().saturating_sub(start_tsc);
        if elapsed >= target_cycles { break; }

        // Also allow manual exit with Q/ESC
        if let Some(key) = crate::keyboard::try_read_key() {
            if key == 0x1B || key == b'q' { break; }
        }

        match effect {
            "plasma" => render_plasma_frame(&mut buf, rw, rh, frame),
            "fire" => render_fire_frame(&mut buf, &mut heat, rw, rh, &mut seed),
            "matrix" => render_matrix_frame(&mut buf, rw, rh, &mut drops, &mut speeds, &mut seed, col_w, ncols),
            _ => break,
        }

        if let Some((bb_ptr, _bb_w, bb_h, bb_stride)) = crate::framebuffer::get_backbuffer_info() {
            let bb = bb_ptr as *mut u32;
            let bb_s = bb_stride as usize;
            for y in 0..rh {
                let dy = oy + y;
                if dy >= bb_h as usize { break; }
                let src_row = &buf[y * rw..y * rw + rw];
                unsafe {
                    let dst = bb.add(dy * bb_s + ox);
                    core::ptr::copy_nonoverlapping(src_row.as_ptr(), dst, rw);
                }
            }
        }
        crate::framebuffer::swap_buffers();
        frame = frame.wrapping_add(1);
    }

    if !was_db {
        crate::framebuffer::set_double_buffer_mode(false);
    }

    crate::serial_println!("[video] Timed demo '{}' stopped after {} frames ({} ms)", effect, frame, duration_ms);
}

fn render_plasma_frame(buf: &mut [u32], w: usize, h: usize, frame: u32) {
    // Fixed-point integer plasma (no floating point)
    // Uses 256-entry sine LUT, 8-bit fractional precision
    let t = frame as i32;
    for y in 0..h {
        for x in 0..w {
            // Scale coordinates to 0-255 range
            let fx = (x * 256 / w) as i32;
            let fy = (y * 256 / h) as i32;
            // Four sine waves with different frequencies and phases
            let v1 = isin((fx.wrapping_mul(3).wrapping_add(t * 4)) as u8);
            let v2 = isin((fy.wrapping_mul(2).wrapping_add(t * 5)) as u8);
            let v3 = isin((fx.wrapping_add(fy).wrapping_mul(2).wrapping_add(t * 3)) as u8);
            let v4 = isin((fx.wrapping_sub(fy).wrapping_add(t * 7)) as u8);
            // Average: range -128..127 → map to 0..255 for hue
            let avg = (v1 as i32 + v2 as i32 + v3 as i32 + v4 as i32) / 4;
            let hue = ((avg + 128) & 0xFF) as u8;
            let (r, g, b) = hue_to_rgb_int(hue);
            buf[y * w + x] = 0xFF000000 | ((r as u32) << 16) | ((g as u32) << 8) | b as u32;
        }
    }
}

/// Integer sine table: 256 entries, output -127..127
#[inline(always)]
fn isin(idx: u8) -> i8 {
    // Pre-computed sine table (one full cycle in 256 steps)
    static SINTAB: [i8; 256] = {
        let mut t = [0i8; 256];
        let mut i = 0u16;
        while i < 256 {
            // sin(2*pi*i/256) * 127, computed at compile time
            // Using integer approximation of sine
            let phase = i as i32; // 0..255 maps to 0..2pi
            // Quadrant-based sine approximation
            let q = (phase & 0xFF) as i32;
            let val = if q < 64 {
                // 0..pi/2: rising 0..127
                (q * 127 / 64) as i8
            } else if q < 128 {
                // pi/2..pi: falling 127..0
                ((128 - q) * 127 / 64) as i8
            } else if q < 192 {
                // pi..3pi/2: falling 0..-127
                -((q - 128) * 127 / 64) as i8
            } else {
                // 3pi/2..2pi: rising -127..0
                -((256 - q) * 127 / 64) as i8
            };
            t[i as usize] = val;
            i += 1;
        }
        t
    };
    SINTAB[idx as usize]
}

/// Integer hue (0-255) to RGB
#[inline(always)]
fn hue_to_rgb_int(hue: u8) -> (u8, u8, u8) {
    let h = hue as u16;
    let sector = h * 6 / 256; // 0..5
    let frac = ((h * 6) % 256) as u8; // 0..255 within sector
    let q = 255 - frac;
    match sector {
        0 => (255, frac, 0),
        1 => (q, 255, 0),
        2 => (0, 255, frac),
        3 => (0, q, 255),
        4 => (frac, 0, 255),
        _ => (255, 0, q),
    }
}

fn render_fire_frame(buf: &mut [u32], heat: &mut [u8], w: usize, h: usize, seed: &mut u32) {
    // Random heat at bottom
    for x in 0..w {
        *seed = xorshift(*seed);
        heat[(h - 1) * w + x] = (*seed & 0xFF) as u8;
        *seed = xorshift(*seed);
        heat[(h - 2) * w + x] = ((*seed & 0xFF) as u16).min(255) as u8;
    }
    // Propagate upward
    for y in 0..h.saturating_sub(2) {
        for x in 0..w {
            let below = heat[(y + 1) * w + x] as u16;
            let bl = if x > 0 { heat[(y + 1) * w + x - 1] as u16 } else { below };
            let br = if x + 1 < w { heat[(y + 1) * w + x + 1] as u16 } else { below };
            let bb = heat[((y + 2).min(h - 1)) * w + x] as u16;
            let avg = (below + bl + br + bb) / 4;
            heat[y * w + x] = if avg > 2 { (avg - 2).min(255) as u8 } else { 0 };
        }
    }
    // Color map
    for i in 0..w * h {
        let t = heat[i];
        let (r, g, b) = if t < 64 {
            (t * 4, 0u8, 0u8)
        } else if t < 128 {
            (255, (t - 64) * 4, 0u8)
        } else if t < 192 {
            (255, 255, (t - 128) * 4)
        } else {
            (255, 255, 255)
        };
        buf[i] = 0xFF000000 | ((r as u32) << 16) | ((g as u32) << 8) | b as u32;
    }
}

fn render_matrix_frame(buf: &mut [u32], w: usize, h: usize,
    drops: &mut [i32], speeds: &mut [u8], seed: &mut u32,
    col_w: usize, ncols: usize)
{
    // Fade existing
    for i in 0..w * h {
        let px = buf[i];
        let r = ((px >> 16) & 0xFF) * 90 / 100;
        let g = ((px >> 8) & 0xFF) * 90 / 100;
        let b = (px & 0xFF) * 90 / 100;
        buf[i] = 0xFF000000 | (r << 16) | (g << 8) | b;
    }
    // Draw drops
    for c in 0..ncols {
        let x_base = c * col_w;
        let dy = drops[c];
        if dy >= 0 && (dy as usize) < h {
            let y = dy as usize;
            for px in 0..col_w.min(w.saturating_sub(x_base)) {
                buf[y * w + x_base + px] = 0xFFCCFFCC;
            }
            for trail in 1..8u32 {
                let ty = dy - trail as i32;
                if ty >= 0 && (ty as usize) < h {
                    let intensity = (200u32).saturating_sub(trail * 20);
                    let g = intensity.min(255);
                    for px in 0..col_w.min(w.saturating_sub(x_base)) {
                        buf[ty as usize * w + x_base + px] =
                            0xFF000000 | ((g / 4) << 16) | (g << 8) | (g / 4);
                    }
                }
            }
        }
        drops[c] += speeds[c] as i32;
        if drops[c] > (h as i32 + 20) {
            *seed = xorshift(*seed);
            drops[c] = -((*seed % (h as u32 / 2)) as i32);
            *seed = xorshift(*seed);
            speeds[c] = 1 + (*seed % 4) as u8;
        }
    }
}
