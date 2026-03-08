//! Audio Visualizer — 3D Holographic Desktop Overlay
//!
//! A transparent music visualizer overlay for TrustOS:
//! - 3D holographic fleur-de-lys logo (cyan/green with scanlines)
//! - Radial shockwave distortion on beat drops
//! - Expanding energy rings from logo center
//! - Desktop stays visible underneath (snapshot during playback)
//! - FFT-based spectral analysis with adaptive beat detection
//!
//! Shell: `play <file.wav>` or `daw viz <file.wav>`

use alloc::string::String;
use alloc::vec::Vec;
use alloc::format;
use core::sync::atomic::Ordering;

// ═══════════════════════════════════════════════════════════════════════════════
// Audio Decoding
// ═══════════════════════════════════════════════════════════════════════════════

/// Decode a WAV file (16-bit PCM) to 48 kHz stereo i16 interleaved samples.
pub fn decode_wav_to_pcm(data: &[u8]) -> Result<Vec<i16>, &'static str> {
    let info = crate::drivers::hda::parse_wav(data)?;
    if info.bits_per_sample != 16 {
        return Err("Only 16-bit PCM WAV supported");
    }

    let pcm = &data[info.data_offset..info.data_offset + info.data_size];
    let num_src_frames = info.data_size / (2 * info.channels as usize);
    let target_rate = 48000u32;
    let num_dst_frames = (num_src_frames as u64 * target_rate as u64
        / info.sample_rate as u64) as usize;

    let mut output = Vec::with_capacity(num_dst_frames * 2);

    for dst_frame in 0..num_dst_frames {
        let src_frame = (dst_frame as u64 * info.sample_rate as u64
            / target_rate as u64) as usize;
        if src_frame >= num_src_frames { break; }

        let idx = src_frame * info.channels as usize;
        let byte_idx = idx * 2;

        let left = if byte_idx + 1 < pcm.len() {
            i16::from_le_bytes([pcm[byte_idx], pcm[byte_idx + 1]])
        } else { 0 };

        let right = if info.channels >= 2 {
            let ri = (idx + 1) * 2;
            if ri + 1 < pcm.len() {
                i16::from_le_bytes([pcm[ri], pcm[ri + 1]])
            } else { left }
        } else { left };

        output.push(left);
        output.push(right);
    }

    Ok(output)
}

/// Detect file type from header magic bytes.
/// Returns "wav", "mp3", or "unknown".
pub fn detect_format(data: &[u8]) -> &'static str {
    if data.len() >= 12 && &data[0..4] == b"RIFF" && &data[8..12] == b"WAVE" {
        return "wav";
    }
    // MP3: starts with 0xFF 0xFB/0xF3/0xF2 (sync word) or ID3 tag
    if data.len() >= 3 {
        if data[0] == 0xFF && (data[1] & 0xE0) == 0xE0 {
            return "mp3";
        }
        if &data[0..3] == b"ID3" {
            return "mp3";
        }
    }
    "unknown"
}

// ═══════════════════════════════════════════════════════════════════════════════
// Audio Analysis — FFT-based Spectral Beat Detection & Band Energy
// ═══════════════════════════════════════════════════════════════════════════════

/// In-place Radix-2 FFT (Cooley–Tukey, decimation-in-time).
/// `re` and `im` must both be length N where N is a power of 2.
fn fft_in_place(re: &mut [f32], im: &mut [f32]) {
    let n = re.len();
    debug_assert!(n.is_power_of_two());
    debug_assert_eq!(n, im.len());

    // Bit-reversal permutation
    let mut j = 0usize;
    for i in 0..n {
        if i < j {
            re.swap(i, j);
            im.swap(i, j);
        }
        let mut m = n >> 1;
        while m >= 1 && j >= m {
            j -= m;
            m >>= 1;
        }
        j += m;
    }

    // Butterfly stages
    let mut step = 2;
    while step <= n {
        let half = step / 2;
        let angle_step = -core::f32::consts::PI * 2.0 / step as f32;
        for k in 0..half {
            let angle = angle_step * k as f32;
            let wr = libm::cosf(angle);
            let wi = libm::sinf(angle);
            let mut i = k;
            while i < n {
                let j = i + half;
                let tr = wr * re[j] - wi * im[j];
                let ti = wr * im[j] + wi * re[j];
                re[j] = re[i] - tr;
                im[j] = im[i] - ti;
                re[i] += tr;
                im[i] += ti;
                i += step;
            }
        }
        step <<= 1;
    }
}

/// FFT size — 1024 samples → ~21ms at 48kHz, gives 512 frequency bins.
/// Bin resolution = 48000/1024 ≈ 46.9 Hz per bin.
const FFT_N: usize = 1024;
const FFT_HALF: usize = FFT_N / 2;

/// Frequency band definitions (bin indices for 48kHz/1024-point FFT).
/// bin_freq = bin_index * 48000 / 1024 ≈ bin_index * 46.9 Hz
const BAND_SUB_BASS: (usize, usize) = (1, 2);      // 47–94 Hz (kick drum fundamental)
const BAND_BASS: (usize, usize)     = (2, 6);       // 94–281 Hz (bass guitar, kick)
const BAND_LOW_MID: (usize, usize)  = (6, 12);      // 281–563 Hz (snare, low vocal)
const BAND_MID: (usize, usize)      = (12, 45);     // 563–2109 Hz (vocal, guitar)
const BAND_HIGH_MID: (usize, usize) = (45, 90);     // 2109–4219 Hz (presence)
const BAND_TREBLE: (usize, usize)   = (90, 220);    // 4219–10313 Hz (hihat, cymbal)

/// Compute average magnitude in a frequency band from FFT output.
#[inline]
fn band_energy(re: &[f32], im: &[f32], lo: usize, hi: usize) -> f32 {
    if hi <= lo { return 0.0; }
    let mut sum = 0.0f32;
    for i in lo..hi.min(re.len()) {
        // Magnitude = sqrt(re² + im²), but we use magnitude² for speed and take sqrt at end
        sum += libm::sqrtf(re[i] * re[i] + im[i] * im[i]);
    }
    sum / (hi - lo) as f32
}

/// Beat detection state — FFT-based spectral analysis with adaptive thresholding.
struct BeatState {
    /// FFT scratch buffers (reused each frame)
    fft_re: [f32; FFT_N],
    fft_im: [f32; FFT_N],
    /// History of sub-bass energy for beat detection (43 frames ≈ 1.4s at 30fps)
    energy_hist: [f32; 43],
    hist_idx: usize,
    hist_count: usize,
    /// Current beat pulse (snaps to 1.0 on beat, decays)
    beat: f32,
    /// Smoothed continuous overall energy
    energy: f32,
    /// Smoothed band energies (6 bands)
    sub_bass: f32,
    bass: f32,
    low_mid: f32,
    mid: f32,
    high_mid: f32,
    treble: f32,
    /// Previous frame energy for onset slope
    prev_energy: f32,
    /// Auto-gain: tracked peak RMS for normalization
    peak_rms: f32,
    /// Debug frame counter
    dbg_frame: u32,
}

impl BeatState {
    fn new() -> Self {
        Self {
            fft_re: [0.0; FFT_N],
            fft_im: [0.0; FFT_N],
            energy_hist: [0.0; 43],
            hist_idx: 0,
            hist_count: 0,
            beat: 0.0,
            energy: 0.0,
            sub_bass: 0.0,
            bass: 0.0,
            low_mid: 0.0,
            mid: 0.0,
            high_mid: 0.0,
            treble: 0.0,
            prev_energy: 0.0,
            peak_rms: 1.0,
            dbg_frame: 0,
        }
    }

    /// Update beat detection from the current audio position using FFT.
    fn update(&mut self, audio: &[i16], center: usize) {
        self.dbg_frame += 1;

        // ── Extract FFT_N mono samples centered at `center` ──
        // We take every 2nd sample (left channel only from stereo interleaved)
        let mono_start = center.saturating_sub(FFT_N); // go back FFT_N stereo pairs = FFT_N*2 samples
        let mono_start = mono_start & !1; // align to even (left channel)

        let mut max_abs: f32 = 0.0;
        for i in 0..FFT_N {
            let idx = mono_start + i * 2; // step by 2 for left channel
            let sample = if idx < audio.len() { audio[idx] as f32 } else { 0.0 };
            self.fft_re[i] = sample;
            self.fft_im[i] = 0.0;
            let abs = if sample >= 0.0 { sample } else { -sample };
            if abs > max_abs { max_abs = abs; }
        }

        // ── Auto-gain normalization ──
        // Track the peak amplitude and normalize so quiet tracks are boosted
        if max_abs > self.peak_rms {
            self.peak_rms = self.peak_rms + (max_abs - self.peak_rms) * 0.3;
        } else {
            self.peak_rms = self.peak_rms * 0.9995; // very slow decay
        }
        let gain = if self.peak_rms > 100.0 { 16000.0 / self.peak_rms } else { 1.0 };

        // Apply Hann window + gain normalization
        for i in 0..FFT_N {
            // Hann window: 0.5 * (1 - cos(2π·i/N))
            let t = i as f32 / FFT_N as f32;
            let hann = 0.5 * (1.0 - libm::cosf(2.0 * core::f32::consts::PI * t));
            self.fft_re[i] *= hann * gain / 32768.0; // normalize to [-1, 1] range
        }

        // ── Compute FFT ──
        fft_in_place(&mut self.fft_re, &mut self.fft_im);

        // ── Extract band energies from frequency spectrum ──
        let raw_sub_bass = band_energy(&self.fft_re, &self.fft_im, BAND_SUB_BASS.0, BAND_SUB_BASS.1);
        let raw_bass = band_energy(&self.fft_re, &self.fft_im, BAND_BASS.0, BAND_BASS.1);
        let raw_low_mid = band_energy(&self.fft_re, &self.fft_im, BAND_LOW_MID.0, BAND_LOW_MID.1);
        let raw_mid = band_energy(&self.fft_re, &self.fft_im, BAND_MID.0, BAND_MID.1);
        let raw_high_mid = band_energy(&self.fft_re, &self.fft_im, BAND_HIGH_MID.0, BAND_HIGH_MID.1);
        let raw_treble = band_energy(&self.fft_re, &self.fft_im, BAND_TREBLE.0, BAND_TREBLE.1);

        // Overall energy = weighted sum of all bands
        let raw_energy = raw_sub_bass * 1.5 + raw_bass * 1.2 + raw_low_mid * 0.8
            + raw_mid * 0.5 + raw_high_mid * 0.3 + raw_treble * 0.2;

        // ── Smooth band energies (fast attack, slow release) ──
        self.sub_bass = Self::smooth(self.sub_bass, raw_sub_bass.min(1.0), 0.75, 0.10);
        self.bass     = Self::smooth(self.bass,     raw_bass.min(1.0),     0.70, 0.10);
        self.low_mid  = Self::smooth(self.low_mid,  raw_low_mid.min(1.0),  0.65, 0.12);
        self.mid      = Self::smooth(self.mid,      raw_mid.min(1.0),      0.60, 0.12);
        self.high_mid = Self::smooth(self.high_mid, raw_high_mid.min(1.0), 0.65, 0.14);
        self.treble   = Self::smooth(self.treble,   raw_treble.min(1.0),   0.70, 0.16);
        self.energy   = Self::smooth(self.energy,   raw_energy.min(1.5),   0.65, 0.10);

        // ── Beat detection with adaptive threshold (variance-based) ──
        let beat_energy = raw_sub_bass + raw_bass * 0.8; // focus on kick drum range

        // Store in history ring buffer
        self.energy_hist[self.hist_idx] = beat_energy;
        self.hist_idx = (self.hist_idx + 1) % self.energy_hist.len();
        if self.hist_count < self.energy_hist.len() {
            self.hist_count += 1;
        }

        // Compute average and variance of history
        let filled = self.hist_count.max(1) as f32;
        let avg: f32 = self.energy_hist.iter().take(self.hist_count).sum::<f32>() / filled;

        let mut var_sum = 0.0f32;
        for i in 0..self.hist_count {
            let diff = self.energy_hist[i] - avg;
            var_sum += diff * diff;
        }
        let variance = var_sum / filled;

        // Adaptive threshold: high variance (energetic music) → lower threshold
        // Low variance (quiet/lofi) → higher threshold to avoid false positives
        // But overall much more sensitive than before
        let threshold = (-15.0 * variance + 1.45).max(1.05).min(1.5);

        // Onset detection: energy spike above adaptive threshold + positive slope
        let onset_slope = beat_energy - self.prev_energy;
        if beat_energy > avg * threshold && onset_slope > 0.002 && self.hist_count > 5 {
            let strength = ((beat_energy - avg * threshold) / avg.max(0.001)).min(1.0);
            self.beat = (0.6 + strength * 0.4).min(1.0);
        } else {
            self.beat *= 0.88;
            if self.beat < 0.02 { self.beat = 0.0; }
        }
        self.prev_energy = beat_energy;

        // ── Periodic debug logging ──
        if self.dbg_frame == 1 || self.dbg_frame % 60 == 0 {
            crate::serial_println!(
                "[BEAT] f={} pos={} gain={:.1} E={:.3} beat={:.2} sub={:.2} bass={:.2} lm={:.2} mid={:.2} hm={:.2} tre={:.2} thr={:.2}",
                self.dbg_frame, center, gain, self.energy, self.beat,
                self.sub_bass, self.bass, self.low_mid, self.mid, self.high_mid, self.treble, threshold
            );
        }
    }

    #[inline]
    fn smooth(prev: f32, new: f32, attack: f32, release: f32) -> f32 {
        if new > prev { prev + (new - prev) * attack }
        else { prev + (new - prev) * release }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// 3D Holographic Desktop Overlay
// ═══════════════════════════════════════════════════════════════════════════════

/// Alpha-blend foreground (fr,fg,fb) over background pixel at given alpha (0–255).
#[inline(always)]
fn blend_alpha(bg: u32, fr: u32, fg: u32, fb: u32, alpha: u32) -> u32 {
    let inv = 255 - alpha;
    let br = (bg >> 16) & 0xFF;
    let bgr = (bg >> 8) & 0xFF;
    let bb = bg & 0xFF;
    let r = (fr * alpha + br * inv) / 255;
    let g = (fg * alpha + bgr * inv) / 255;
    let b = (fb * alpha + bb * inv) / 255;
    0xFF000000 | (r << 16) | (g << 8) | b
}

/// An expanding shockwave ring radiating from logo center on beat.
#[derive(Clone, Copy)]
struct ShockRing {
    radius: f32,
    strength: f32,
    alpha: f32,
}

const MAX_RINGS: usize = 8;
const RING_SPEED: f32 = 5.0;       // expansion pixels/frame
const RING_WIDTH: f32 = 35.0;      // distortion band half-width
const RING_DECAY: f32 = 0.96;      // per-frame strength decay
const RING_MAX_R: f32 = 650.0;     // rings die past this radius

/// The holographic overlay engine — manages desktop snapshot, shockwave rings,
/// and all overlay rendering.
struct HoloOverlay {
    /// Desktop snapshot (frozen screen pixels captured before playback)
    snapshot: Vec<u32>,
    snap_w: usize,
    snap_h: usize,
    /// Active expanding shockwave rings
    rings: [ShockRing; MAX_RINGS],
    ring_count: usize,
    /// Holographic sweep line phase (0.0–1.0)
    sweep: f32,
    /// Animation frame counter
    frame: u32,
    /// Previous beat value for onset edge detection
    prev_beat: f32,
}

impl HoloOverlay {
    fn new(w: u32, h: u32) -> Self {
        Self {
            snapshot: Vec::new(),
            snap_w: w as usize,
            snap_h: h as usize,
            rings: [ShockRing { radius: 0.0, strength: 0.0, alpha: 0.0 }; MAX_RINGS],
            ring_count: 0,
            sweep: 0.0,
            frame: 0,
            prev_beat: 0.0,
        }
    }

    /// Capture the current screen from MMIO framebuffer (the actual displayed pixels).
    /// This works regardless of backbuffer state since MMIO always has the last swapped frame.
    fn capture_snapshot(&mut self) {
        let addr = crate::framebuffer::FB_ADDR.load(Ordering::Relaxed);
        if addr.is_null() { return; }
        let pitch = crate::framebuffer::FB_PITCH.load(Ordering::Relaxed) as usize;
        let w = self.snap_w;
        let h = self.snap_h;

        self.snapshot = Vec::with_capacity(w * h);
        unsafe {
            for y in 0..h {
                let row = addr.add(y * pitch) as *const u32;
                for x in 0..w {
                    self.snapshot.push(core::ptr::read(row.add(x)));
                }
            }
        }
        crate::serial_println!("[VIZ] Snapshot captured: {}x{} ({} px)", w, h, self.snapshot.len());
    }

    /// Restore snapshot pixels to the backbuffer (fast bulk copy).
    fn restore_snapshot(&self) {
        if self.snapshot.is_empty() { return; }
        if let Some((ptr, _w, _h, _stride)) = crate::framebuffer::get_backbuffer_info() {
            let n = self.snapshot.len();
            unsafe {
                core::ptr::copy_nonoverlapping(
                    self.snapshot.as_ptr(),
                    ptr as *mut u32,
                    n,
                );
            }
        }
    }

    /// Read a pixel from the snapshot with bounds checking.
    #[inline]
    fn snap_px(&self, x: i32, y: i32) -> u32 {
        if x < 0 || y < 0 || x as usize >= self.snap_w || y as usize >= self.snap_h {
            return 0xFF000000;
        }
        self.snapshot[y as usize * self.snap_w + x as usize]
    }

    /// Spawn a new shockwave ring from the logo center.
    fn spawn_ring(&mut self, strength: f32) {
        let ring = ShockRing {
            radius: 25.0,
            strength: strength * 14.0,
            alpha: 0.95,
        };
        if self.ring_count < MAX_RINGS {
            self.rings[self.ring_count] = ring;
            self.ring_count += 1;
        } else {
            // Replace the weakest existing ring
            let mut wi = 0;
            for i in 1..MAX_RINGS {
                if self.rings[i].strength < self.rings[wi].strength { wi = i; }
            }
            self.rings[wi] = ring;
        }
    }

    /// Update overlay state for this frame (ring physics, sweep animation).
    fn tick(&mut self, beat: &BeatState) {
        self.frame += 1;

        // Sweep line animation (holographic refresh beam)
        self.sweep += 0.008 + beat.energy * 0.015;
        if self.sweep > 1.0 { self.sweep -= 1.0; }

        // Detect beat onset (rising edge) → spawn shockwave ring
        if beat.beat > 0.35 && self.prev_beat < 0.25 {
            self.spawn_ring(beat.beat);
        }
        self.prev_beat = beat.beat;

        // Update ring physics: expand, decay, cull dead rings
        let mut w = 0usize;
        for r in 0..self.ring_count {
            self.rings[r].radius += RING_SPEED;
            self.rings[r].strength *= RING_DECAY;
            self.rings[r].alpha *= 0.97;
            if self.rings[r].radius < RING_MAX_R && self.rings[r].alpha > 0.01 {
                if w != r { self.rings[w] = self.rings[r]; }
                w += 1;
            }
        }
        self.ring_count = w;
    }

    /// Render the complete overlay frame to the backbuffer.
    fn draw_frame(&self, beat: &BeatState) {
        let (fb, w, h) = match crate::framebuffer::get_backbuffer_info() {
            Some((p, w, h, _)) => (p as *mut u32, w as usize, h as usize),
            None => return,
        };
        let cx = w / 2;
        let cy = h * 42 / 100; // logo center: slightly above screen center

        // 1. Restore desktop snapshot as base layer
        self.restore_snapshot();

        // 2. Apply radial shockwave distortion from active rings
        if self.ring_count > 0 {
            self.apply_distortion(fb, w, h, cx, cy);
        }

        // 3. Ambient glow halo behind logo
        self.draw_glow(w as u32, h as u32, cx as u32, cy as u32, beat);

        // 4. Holographic logo (semi-transparent, scanlines, chromatic aberration)
        self.draw_logo(fb, w, h, cx, cy, beat);

        // 5. Shockwave ring outlines
        for i in 0..self.ring_count {
            self.draw_ring(fb, w, h, cx, cy, &self.rings[i]);
        }
    }

    /// Apply radial distortion: pixels near active rings get displaced outward from center.
    fn apply_distortion(&self, fb: *mut u32, w: usize, h: usize, cx: usize, cy: usize) {
        let cxf = cx as f32;
        let cyf = cy as f32;

        // Compute bounding box of all active rings
        let mut max_r: f32 = 0.0;
        for i in 0..self.ring_count {
            let r = self.rings[i].radius + RING_WIDTH + 5.0;
            if r > max_r { max_r = r; }
        }
        let ri = max_r as i32;
        let x0 = (cx as i32 - ri).max(0) as usize;
        let x1 = ((cx as i32 + ri) as usize).min(w.saturating_sub(1));
        let y0 = (cy as i32 - ri).max(0) as usize;
        let y1 = ((cy as i32 + ri) as usize).min(h.saturating_sub(1));

        for y in y0..=y1 {
            let dy = y as f32 - cyf;
            for x in x0..=x1 {
                let dx = x as f32 - cxf;
                let dist = libm::sqrtf(dx * dx + dy * dy);
                if dist < 1.0 { continue; }

                // Accumulate displacement from all active rings
                let mut disp: f32 = 0.0;
                for i in 0..self.ring_count {
                    let d = (dist - self.rings[i].radius) / RING_WIDTH;
                    if d > 1.0 || d < -1.0 { continue; }
                    // Smooth bell curve: (1 - d²)²
                    let t = 1.0 - d * d;
                    disp += self.rings[i].strength * t * t;
                }
                if disp < 0.5 { continue; }

                // Displace source pixel radially inward → visual "push outward" effect
                let inv = 1.0 / dist;
                let sx = (x as f32 - dx * inv * disp) as i32;
                let sy = (y as f32 - dy * inv * disp) as i32;
                let color = self.snap_px(sx, sy);
                unsafe {
                    core::ptr::write(fb.add(y * w + x), color);
                }
            }
        }
    }

    /// Draw ambient glow halo behind the logo (concentric soft rectangles).
    fn draw_glow(&self, fw: u32, fh: u32, cx: u32, cy: u32, beat: &BeatState) {
        let intensity = beat.energy * 0.3 + beat.beat * 0.5;
        if intensity < 0.02 { return; }
        let max_r = fw.min(fh) / 4;
        for ring in 0..10u32 {
            let r = (ring + 1) * max_r / 10;
            let t = ring as f32 / 10.0;
            let a = ((1.0 - t) * intensity * 18.0) as u32;
            if a < 1 { continue; }
            let left = cx.saturating_sub(r);
            let top = cy.saturating_sub(r);
            let rw = (r * 2).min(fw.saturating_sub(left));
            let rh = (r * 2).min(fh.saturating_sub(top));
            if rw > 0 && rh > 0 {
                crate::framebuffer::fill_rect_alpha(left, top, rw, rh, 0x00FFCC, a.min(12));
            }
        }
    }

    /// Draw the holographic 3D logo: semi-transparent cyan tint, scanlines,
    /// sweep beam, chromatic aberration on edges, beat-reactive brightness.
    fn draw_logo(&self, fb: *mut u32, w: usize, h: usize, cx: usize, cy: usize, beat: &BeatState) {
        let lw = crate::logo_bitmap::LOGO_W;
        let lh = crate::logo_bitmap::LOGO_H;
        let sc = ((h as u32) * 45 / lh as u32).max(100);
        let rw = lw as u32 * sc / 100;
        let rh = lh as u32 * sc / 100;
        let rx = (cx as u32).saturating_sub(rw / 2);
        let ry = (cy as u32).saturating_sub(rh / 2);

        let pulse = beat.beat;
        let energy = beat.energy;

        // Base hologram alpha: semi-transparent, pulses brighter on beat
        let base_a = 0.50 + energy * 0.15 + pulse * 0.25;
        // Sweep beam Y position within the logo
        let sweep_y = (self.sweep * rh as f32) as u32;
        // Per-frame flicker noise
        let flicker = ((self.frame.wrapping_mul(2654435761) % 100) as f32 / 100.0 - 0.5) * 0.04;
        // Chromatic aberration offset scales with beat
        let ca_off = 2 + (pulse * 3.0) as i32;

        // ── Main logo pass: per-pixel holographic rendering ──
        for py in 0..rh {
            let sy = (py * 100 / sc) as usize;
            if sy >= lh { continue; }
            let screen_y = ry + py;
            if screen_y >= h as u32 { continue; }

            // Scanline dimming: every 3rd row at 35% brightness (holographic projection lines)
            let scan = if py % 3 == 0 { 0.35f32 } else { 1.0f32 };
            // Sweep beam highlight
            let sd = py as f32 - sweep_y as f32;
            let sw_b = if libm::fabsf(sd) < 10.0 { 0.5 * (1.0 - libm::fabsf(sd) / 10.0) } else { 0.0f32 };

            for px in 0..rw {
                let sx = (px * 100 / sc) as usize;
                if sx >= lw { continue; }
                let screen_x = rx + px;
                if screen_x >= w as u32 { continue; }

                let argb = crate::logo_bitmap::logo_pixel(sx, sy);
                if (argb >> 24) & 0xFF < 20 { continue; }
                let (r, g, b) = ((argb >> 16) & 0xFF, (argb >> 8) & 0xFF, argb & 0xFF);
                let luma = (r * 77 + g * 150 + b * 29) >> 8;
                if luma < 28 { continue; } // skip dark/transparent background

                let edge = crate::logo_bitmap::logo_edge_pixel(sx, sy);
                let lf = luma as f32 / 255.0;

                // Holographic color mapping: convert to cyan/green tones
                let (mut hr, mut hg, mut hb) = if edge {
                    // Edges: bright neon cyan outline
                    (lf * 0.25 + pulse * 0.15,
                     lf * 1.1 + pulse * 0.4,
                     lf * 0.95 + pulse * 0.25)
                } else {
                    // Interior: subtle cyan/green holographic tint
                    (lf * 0.12,
                     lf * 0.7 + energy * 0.1,
                     lf * 0.55)
                };

                // Apply scanline dimming + sweep highlight + flicker
                hr = (hr * scan + sw_b * 0.2 + flicker).max(0.0);
                hg = (hg * scan + sw_b * 0.9 + flicker).max(0.0);
                hb = (hb * scan + sw_b * 0.7 + flicker).max(0.0);

                // Beat flash: brief bright flash on strong beats
                if pulse > 0.7 {
                    let fl = (pulse - 0.7) * 3.0;
                    hr += fl * 0.4;
                    hg += fl * 0.7;
                    hb += fl * 0.5;
                }

                let cr = (hr * 255.0).min(255.0) as u32;
                let cg = (hg * 255.0).min(255.0) as u32;
                let cb = (hb * 255.0).min(255.0) as u32;
                let a_mult = if edge { 1.3f32 } else { 1.0f32 };
                let alpha = (base_a * scan * a_mult * 255.0).min(255.0) as u32;

                let idx = screen_y as usize * w + screen_x as usize;
                unsafe {
                    let bg = core::ptr::read(fb.add(idx));
                    core::ptr::write(fb.add(idx), blend_alpha(bg, cr, cg, cb, alpha));
                }

                // Chromatic aberration on edges: red ghost left, blue ghost right
                if edge && alpha > 80 {
                    let ca_a = alpha / 4;
                    let lx = screen_x as i32 - ca_off;
                    if lx >= 0 && (lx as usize) < w {
                        let ci = screen_y as usize * w + lx as usize;
                        unsafe {
                            let bg = core::ptr::read(fb.add(ci));
                            core::ptr::write(fb.add(ci), blend_alpha(bg, 180, 10, 10, ca_a));
                        }
                    }
                    let rx2 = screen_x as usize + ca_off as usize;
                    if rx2 < w {
                        let ci = screen_y as usize * w + rx2;
                        unsafe {
                            let bg = core::ptr::read(fb.add(ci));
                            core::ptr::write(fb.add(ci), blend_alpha(bg, 10, 10, 200, ca_a));
                        }
                    }
                }
            }
        }

        // ── Outer edge glow layer (slightly enlarged, soft) ──
        let gsc = sc * 106 / 100;
        let gw = lw as u32 * gsc / 100;
        let gh = lh as u32 * gsc / 100;
        let gx = (cx as u32).saturating_sub(gw / 2);
        let gy = (cy as u32).saturating_sub(gh / 2);
        let ga = (12.0 + energy * 30.0 + pulse * 50.0).min(100.0) as u32;

        for py in (0..gh).step_by(2) {
            let sy = (py * 100 / gsc) as usize;
            if sy >= lh { continue; }
            let scr_y = gy + py;
            if scr_y >= h as u32 { continue; }
            for px in (0..gw).step_by(2) {
                let sx = (px * 100 / gsc) as usize;
                if sx >= lw { continue; }
                if !crate::logo_bitmap::logo_edge_pixel(sx, sy) { continue; }
                let scr_x = gx + px;
                if scr_x >= w as u32 { continue; }
                // 2×2 glow block
                for dy in 0..2u32 {
                    for dx in 0..2u32 {
                        let fx = scr_x + dx;
                        let fy = scr_y + dy;
                        if fx >= w as u32 || fy >= h as u32 { continue; }
                        let idx = fy as usize * w + fx as usize;
                        unsafe {
                            let bg = core::ptr::read(fb.add(idx));
                            core::ptr::write(fb.add(idx), blend_alpha(bg, 0, 255, 204, ga));
                        }
                    }
                }
            }
        }
    }

    /// Draw a single shockwave ring outline (thin cyan circle).
    fn draw_ring(&self, fb: *mut u32, w: usize, h: usize, cx: usize, cy: usize, ring: &ShockRing) {
        if ring.alpha < 0.02 { return; }
        let r = ring.radius;
        let segs = ((r * 6.28) as u32).max(60).min(720);
        let a = (ring.alpha * 180.0) as u32;

        for s in 0..segs {
            let angle = s as f32 * 6.2831853 / segs as f32;
            let px = cx as f32 + r * libm::cosf(angle);
            let py = cy as f32 + r * libm::sinf(angle);
            let ix = px as i32;
            let iy = py as i32;
            if ix < 0 || iy < 0 || ix >= w as i32 - 1 || iy >= h as i32 - 1 { continue; }

            // 2×2 dot for ring visibility
            for dy in 0..2i32 {
                for dx in 0..2i32 {
                    let fx = (ix + dx) as usize;
                    let fy = (iy + dy) as usize;
                    if fx >= w || fy >= h { continue; }
                    let idx = fy * w + fx;
                    unsafe {
                        let bg = core::ptr::read(fb.add(idx));
                        core::ptr::write(fb.add(idx), blend_alpha(bg, 0, 255, 204, a.min(180)));
                    }
                }
            }
        }
    }
}

/// Draw minimal overlay HUD: title, elapsed/total time, progress bar, Esc hint.
fn draw_overlay_hud(fb_w: u32, fb_h: u32, title: &str, elapsed_s: u32, total_s: u32, pct: u32) {
    let cw = 16u32; // char width at scale 2

    // ── Title (top center) ──
    let tw = title.len() as u32 * cw;
    let tx = (fb_w.saturating_sub(tw)) / 2;
    crate::framebuffer::fill_rect_alpha(tx.saturating_sub(12), 10, tw + 24, 36, 0x000000, 120);
    crate::graphics::scaling::draw_text_at_scale(tx as i32, 14, title, 0x00FFCC, 2);

    // ── Time (bottom-left) ──
    let em = elapsed_s / 60;
    let es = elapsed_s % 60;
    let tm = total_s / 60;
    let ts = total_s % 60;
    let time_s = format!("{}:{:02} / {}:{:02}", em, es, tm, ts);
    let tw2 = time_s.len() as u32 * 8 + 16;
    crate::framebuffer::fill_rect_alpha(8, fb_h.saturating_sub(50), tw2, 20, 0x000000, 100);
    crate::framebuffer::draw_text(&time_s, 16, fb_h.saturating_sub(48), 0x00AA88);

    // ── Progress bar (bottom) ──
    let py = fb_h.saturating_sub(24);
    let pw = fb_w.saturating_sub(60);
    let px_bar = 30u32;
    crate::framebuffer::fill_rect(px_bar, py, pw, 3, 0x001111);
    let filled = pw * pct.min(100) / 100;
    if filled > 0 {
        crate::framebuffer::fill_rect(px_bar, py, filled, 3, 0x00FFCC);
        crate::framebuffer::fill_rect_alpha(px_bar, py.saturating_sub(1), filled, 5, 0x00FFCC, 20);
    }

    // ── Esc hint (bottom-right) ──
    let hint = "[Esc] Exit";
    let hw = hint.len() as u32 * 8 + 8;
    crate::framebuffer::fill_rect_alpha(
        fb_w.saturating_sub(hw + 8), fb_h.saturating_sub(50), hw, 20, 0x000000, 80,
    );
    crate::framebuffer::draw_text(hint, fb_w.saturating_sub(hw + 4), fb_h.saturating_sub(48), 0x446666);
}

// ═══════════════════════════════════════════════════════════════════════════════
// Main Visualizer Entry Point
// ═══════════════════════════════════════════════════════════════════════════════

/// Launch the 3D holographic desktop overlay visualizer.
///
/// `audio`: stereo interleaved i16 @ 48 kHz
/// `title`: display title
///
/// The overlay captures a snapshot of the current desktop and renders on top:
/// holographic logo, shockwave distortion rings, all synced to the audio.
pub fn launch_visualizer(audio: &[i16], title: &str) -> Result<(), &'static str> {
    crate::audio::init().ok();
    let dur_s = audio.len() as f64 / (48000.0 * 2.0);
    crate::serial_println!(
        "[VIZ] Starting holographic overlay: {} ({} samples, {:.1}s)",
        title, audio.len(), dur_s
    );

    let fb_w = crate::framebuffer::FB_WIDTH.load(Ordering::Relaxed) as u32;
    let fb_h = crate::framebuffer::FB_HEIGHT.load(Ordering::Relaxed) as u32;

    // Ensure double buffer exists (don't reinit if desktop already has one)
    if !crate::framebuffer::is_double_buffer_enabled() {
        crate::framebuffer::init_double_buffer();
        crate::framebuffer::set_double_buffer_mode(true);
    }

    // Capture the current screen as our desktop backdrop
    let mut overlay = HoloOverlay::new(fb_w, fb_h);
    overlay.capture_snapshot();

    crate::serial_println!(
        "[VIZ] Using logo_bitmap: {}x{}",
        crate::logo_bitmap::LOGO_W, crate::logo_bitmap::LOGO_H
    );

    let mut beat_state = BeatState::new();
    let total_frames_audio = audio.len() / 2;
    let total_ms = (total_frames_audio as u64 * 1000) / 48000;
    let total_s = (total_ms / 1000) as u32;

    // ── DMA Streaming Setup ──
    let (dma_ptr, dma_cap) = crate::drivers::hda::get_dma_buffer_info()
        .ok_or("HDA not initialized")?;
    let half_i16 = dma_cap / 2;
    let half_bytes = (half_i16 * 2) as u32;
    let full_bytes = (dma_cap * 2) as u32;

    let initial = audio.len().min(dma_cap);
    crate::drivers::hda::start_looped_playback(&audio[0..initial])?;

    let mut write_cursor: usize = initial;
    let mut last_half: u32 = 0;
    let mut audio_exhausted = false;
    let mut silence_countdown: u32 = 0;
    let dma_play_ms = (dma_cap as u64 * 1000) / (48000 * 2);
    let frame_ms: u64 = 33;
    let exhaust_frames = ((dma_play_ms / frame_ms) + 10) as u32;

    crate::serial_println!(
        "[VIZ] DMA: buf={} i16, half={}, exhaust_frames={}",
        dma_cap, half_i16, exhaust_frames
    );

    // ── Intro: brief "NOW PLAYING" flash over desktop snapshot ──
    for f in 0..15u32 {
        overlay.restore_snapshot();
        if f < 8 {
            let a = (f * 30).min(230);
            let msg = "NOW PLAYING";
            let mw = msg.len() as u32 * 16 + 32;
            let mx = (fb_w.saturating_sub(mw)) / 2;
            let my = fb_h / 2 - 16;
            crate::framebuffer::fill_rect_alpha(
                mx.saturating_sub(8), my.saturating_sub(8), mw + 16, 48, 0x000000, a,
            );
            crate::graphics::scaling::draw_text_at_scale(mx as i32, my as i32, msg, 0x00FFCC, 2);
        }
        crate::framebuffer::swap_buffers();
        crate::cpu::tsc::pit_delay_ms(50);
    }

    // ── Main playback loop ──
    let mut vis_frame: u32 = 0;

    loop {
        // ── DMA streaming (refill half-buffers) ──
        crate::drivers::hda::clear_stream_status();
        crate::drivers::hda::ensure_running();

        let lpib = crate::drivers::hda::get_playback_position();
        let lpib_clamped = if lpib >= full_bytes { 0 } else { lpib };
        let current_half = if lpib_clamped < half_bytes { 0u32 } else { 1u32 };

        if current_half != last_half {
            if write_cursor < audio.len() {
                let dest_offset = last_half as usize * half_i16;
                let remaining = audio.len() - write_cursor;
                let to_copy = remaining.min(half_i16);
                unsafe {
                    core::ptr::copy_nonoverlapping(
                        audio.as_ptr().add(write_cursor),
                        dma_ptr.add(dest_offset),
                        to_copy,
                    );
                    if to_copy < half_i16 {
                        core::ptr::write_bytes(
                            dma_ptr.add(dest_offset + to_copy), 0, half_i16 - to_copy,
                        );
                    }
                }
                write_cursor += to_copy;
                if write_cursor >= audio.len() {
                    audio_exhausted = true;
                    crate::serial_println!("[VIZ] Audio data exhausted at frame {}", vis_frame);
                }
            } else {
                let dest_offset = last_half as usize * half_i16;
                unsafe {
                    core::ptr::write_bytes(dma_ptr.add(dest_offset), 0, half_i16);
                }
            }
            last_half = current_half;
        }

        // ── Audio position from frame counter ──
        let elapsed_ms = vis_frame as u64 * frame_ms;
        if elapsed_ms >= total_ms { break; }
        let audio_pos = ((elapsed_ms * 48000 * 2) / 1000) as usize;
        let audio_pos = audio_pos.min(audio.len().saturating_sub(2));

        if audio_exhausted {
            silence_countdown += 1;
            if silence_countdown >= exhaust_frames { break; }
        }

        let elapsed_s = (elapsed_ms / 1000) as u32;
        let progress_pct = (elapsed_ms * 100 / total_ms.max(1)) as u32;

        // ── Audio analysis ──
        beat_state.update(audio, audio_pos);

        // ── Overlay update + render ──
        overlay.tick(&beat_state);
        overlay.draw_frame(&beat_state);
        draw_overlay_hud(fb_w, fb_h, title, elapsed_s, total_s, progress_pct);
        crate::framebuffer::swap_buffers();

        vis_frame += 1;

        // ── Frame pacing + Esc detection ──
        let mut remaining_wait = frame_ms;
        let mut escaped = false;
        while remaining_wait > 0 {
            let d = remaining_wait.min(5);
            crate::cpu::tsc::pit_delay_ms(d);
            remaining_wait -= d;
            while let Some(sc) = crate::keyboard::try_read_key() {
                if sc & 0x80 != 0 { continue; }
                if sc == 0x01 { escaped = true; break; }
            }
            if escaped { break; }
        }
        if escaped { break; }
    }

    // Stop DMA immediately
    let _ = crate::drivers::hda::stop();

    // ── Outro: restore desktop with brief "PLAYBACK COMPLETE" message ──
    for f in 0..20u32 {
        overlay.restore_snapshot();
        if f < 10 {
            let a = ((20 - f) * 12).min(200);
            let msg = "PLAYBACK COMPLETE";
            let mw = msg.len() as u32 * 16 + 32;
            let mx = (fb_w.saturating_sub(mw)) / 2;
            let my = fb_h / 2 - 16;
            crate::framebuffer::fill_rect_alpha(
                mx.saturating_sub(8), my.saturating_sub(8), mw + 16, 48, 0x000000, a,
            );
            crate::graphics::scaling::draw_text_at_scale(mx as i32, my as i32, msg, 0x00FFCC, 2);
        }
        crate::framebuffer::swap_buffers();
        crate::cpu::tsc::pit_delay_ms(50);
    }

    // Restore desktop fully
    overlay.restore_snapshot();
    crate::framebuffer::swap_buffers();

    crate::serial_println!("[VIZ] Holographic visualizer closed");
    Ok(())
}

/// Load an audio file from the filesystem and launch the visualizer.
///
/// Supports: WAV (16-bit PCM, any sample rate/channels — resampled to 48kHz stereo)
///
/// Paths:
///   - `/home/song.wav` → ramfs
///   - `/mnt/...` → VFS (FAT32/ext4 disk)
pub fn play_file(path: &str) -> Result<(), &'static str> {
    crate::serial_println!("[VIZ] Loading file: {}", path);

    // Read file bytes — try VFS first, then ramfs
    let data: Vec<u8> = if crate::vfs::stat(path).is_ok() {
        // File exists on VFS (TrustFS, FAT32, ext4, devfs, procfs…)
        crate::vfs::read_file(path).map_err(|_| "Failed to read file from VFS")?
    } else {
        // ramfs fallback
        crate::ramfs::with_fs(|fs| {
            fs.read_file(path).map(|c| c.to_vec())
        }).map_err(|_| "Failed to read file from VFS or ramfs")?
    };

    if data.is_empty() {
        return Err("File is empty");
    }

    let format = detect_format(&data);
    crate::serial_println!("[VIZ] Detected format: {}, size: {} bytes", format, data.len());

    let audio = match format {
        "wav" => decode_wav_to_pcm(&data)?,
        "mp3" => return Err("MP3 not yet supported — convert to WAV first (ffmpeg -i song.mp3 song.wav)"),
        _ => return Err("Unknown audio format — only WAV (16-bit PCM) supported"),
    };

    if audio.is_empty() {
        return Err("Decoded audio is empty");
    }

    let duration_s = audio.len() as f64 / (48000.0 * 2.0);
    crate::serial_println!("[VIZ] Decoded: {} samples, {:.1}s", audio.len(), duration_s);

    // Extract filename for title
    let title = path.rsplit('/').next().unwrap_or(path);
    launch_visualizer(&audio, title)
}

// ═══════════════════════════════════════════════════════════════════════════════
// Built-in Songs — Embedded audio files
// ═══════════════════════════════════════════════════════════════════════════════

/// The real "Untitled (2)" WAV embedded at compile time (16-bit PCM, 48 kHz stereo)
#[cfg(feature = "daw")]
pub static UNTITLED2_WAV: &[u8] = include_bytes!("untitled2.wav");
#[cfg(not(feature = "daw"))]
pub static UNTITLED2_WAV: &[u8] = &[];

/// Standard VFS paths where the built-in track may live.
pub const UNTITLED2_VFS_PATHS: &[&str] = &[
    "/music/untitled2.wav",
    "/mnt/fat32/music/untitled2.wav",
    "/mnt/sda1/music/untitled2.wav",
    "/home/music/untitled2.wav",
];

/// Play "Untitled (2)" — tries VFS → disk → embedded (in that order).
pub fn play_untitled2() -> Result<(), &'static str> {
    // 1) Try loading from VFS (external storage / filesystem)
    for path in UNTITLED2_VFS_PATHS {
        if crate::vfs::stat(path).is_ok() {
            crate::serial_println!("[VIZ] Found '{}' on VFS, loading...", path);
            if let Ok(data) = crate::vfs::read_file(path) {
                if !data.is_empty() {
                    crate::serial_println!("[VIZ] Loaded {} bytes from VFS", data.len());
                    let audio = decode_wav_to_pcm(&data)?;
                    let duration_s = audio.len() as f64 / (48000.0 * 2.0);
                    crate::serial_println!("[VIZ] Decoded: {} samples, {:.1}s", audio.len(), duration_s);
                    return launch_visualizer(&audio, "Untitled (2)");
                }
            }
        }
    }

    // 2) Try loading from raw disk (AHCI sector approach)
    if let Ok(data) = crate::trustdaw::disk_audio::load_wav_from_disk() {
        crate::serial_println!("[VIZ] Loaded {} bytes from data disk", data.len());
        let audio = decode_wav_to_pcm(&data)?;
        let duration_s = audio.len() as f64 / (48000.0 * 2.0);
        crate::serial_println!("[VIZ] Decoded: {} samples, {:.1}s", audio.len(), duration_s);
        return launch_visualizer(&audio, "Untitled (2)");
    }

    // 3) Fall back to embedded WAV (only available with --features daw)
    if !UNTITLED2_WAV.is_empty() {
        crate::serial_println!("[VIZ] Using embedded WAV ({} bytes)", UNTITLED2_WAV.len());
        let audio = decode_wav_to_pcm(UNTITLED2_WAV)?;
        let duration_s = audio.len() as f64 / (48000.0 * 2.0);
        crate::serial_println!("[VIZ] Decoded: {} samples, {:.1}s", audio.len(), duration_s);
        return launch_visualizer(&audio, "Untitled (2)");
    }

    Err("Audio not found — place untitled2.wav in /music/ or build with --features daw")
}
