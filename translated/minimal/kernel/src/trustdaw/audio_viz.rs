










use alloc::string::String;
use alloc::vec::Vec;
use alloc::format;
use core::sync::atomic::Ordering;






pub fn byq(data: &[u8]) -> Result<Vec<i16>, &'static str> {
    let info = crate::drivers::hda::ewj(data)?;
    if info.bits_per_sample != 16 {
        return Err("Only 16-bit PCM WAV supported");
    }

    let dck = &data[info.data_offset..info.data_offset + info.data_size];
    let dvq = info.data_size / (2 * info.channels as usize);
    let dfi = 48000u32;
    let dbw = (dvq as u64 * dfi as u64
        / info.sample_rate as u64) as usize;

    let mut output = Vec::with_capacity(dbw * 2);

    for dst_frame in 0..dbw {
        let eaf = (dst_frame as u64 * info.sample_rate as u64
            / dfi as u64) as usize;
        if eaf >= dvq { break; }

        let idx = eaf * info.channels as usize;
        let yk = idx * 2;

        let left = if yk + 1 < dck.len() {
            i16::from_le_bytes([dck[yk], dck[yk + 1]])
        } else { 0 };

        let right = if info.channels >= 2 {
            let dk = (idx + 1) * 2;
            if dk + 1 < dck.len() {
                i16::from_le_bytes([dck[dk], dck[dk + 1]])
            } else { left }
        } else { left };

        output.push(left);
        output.push(right);
    }

    Ok(output)
}



pub fn dmx(data: &[u8]) -> &'static str {
    if data.len() >= 12 && &data[0..4] == b"RIFF" && &data[8..12] == b"WAVE" {
        return "wav";
    }
    
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







fn luw(xh: &mut [f32], xq: &mut [f32]) {
    let ae = xh.len();
    debug_assert!(ae.is_power_of_two());
    debug_assert_eq!(ae, xq.len());

    
    let mut ay = 0usize;
    for i in 0..ae {
        if i < ay {
            xh.swap(i, ay);
            xq.swap(i, ay);
        }
        let mut m = ae >> 1;
        while m >= 1 && ay >= m {
            ay -= m;
            m >>= 1;
        }
        ay += m;
    }

    
    let mut step = 2;
    while step <= ae {
        let cw = step / 2;
        let dhr = -core::f32::consts::PI * 2.0 / step as f32;
        for k in 0..cw {
            let cc = dhr * k as f32;
            let aep = libm::cosf(cc);
            let ld = libm::sinf(cc);
            let mut i = k;
            while i < ae {
                let ay = i + cw;
                let tr = aep * xh[ay] - ld * xq[ay];
                let cej = aep * xq[ay] + ld * xh[ay];
                xh[ay] = xh[i] - tr;
                xq[ay] = xq[i] - cej;
                xh[i] += tr;
                xq[i] += cej;
                i += step;
            }
        }
        step <<= 1;
    }
}



const FS_: usize = 1024;
const DPH_: usize = FS_ / 2;



const ANH_: (usize, usize) = (1, 2);      
const AND_: (usize, usize)     = (2, 6);       
const ANF_: (usize, usize)  = (6, 12);      
const ANG_: (usize, usize)      = (12, 45);     
const ANE_: (usize, usize) = (45, 90);     
const ANI_: (usize, usize)   = (90, 220);    


#[inline]
fn ctu(xh: &[f32], xq: &[f32], lo: usize, hi: usize) -> f32 {
    if hi <= lo { return 0.0; }
    let mut sum = 0.0f32;
    for i in lo..hi.min(xh.len()) {
        
        sum += libm::sqrtf(xh[i] * xh[i] + xq[i] * xq[i]);
    }
    sum / (hi - lo) as f32
}


struct BeatState {
    
    fft_re: [f32; FS_],
    fft_im: [f32; FS_],
    
    energy_hist: [f32; 43],
    hist_idx: usize,
    hist_count: usize,
    
    beat: f32,
    
    energy: f32,
    
    sub_bass: f32,
    bass: f32,
    low_mid: f32,
    mid: f32,
    high_mid: f32,
    treble: f32,
    
    prev_energy: f32,
    
    peak_rms: f32,
    
    dbg_frame: u32,
}

impl BeatState {
    fn new() -> Self {
        Self {
            fft_re: [0.0; FS_],
            fft_im: [0.0; FS_],
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

    
    fn update(&mut self, audio: &[i16], center: usize) {
        self.dbg_frame += 1;

        
        
        let dun = center.saturating_sub(FS_); 
        let dun = dun & !1; 

        let mut yw: f32 = 0.0;
        for i in 0..FS_ {
            let idx = dun + i * 2; 
            let sample = if idx < audio.len() { audio[idx] as f32 } else { 0.0 };
            self.fft_re[i] = sample;
            self.fft_im[i] = 0.0;
            let abs = if sample >= 0.0 { sample } else { -sample };
            if abs > yw { yw = abs; }
        }

        
        
        if yw > self.peak_rms {
            self.peak_rms = self.peak_rms + (yw - self.peak_rms) * 0.3;
        } else {
            self.peak_rms = self.peak_rms * 0.9995; 
        }
        let bmi = if self.peak_rms > 100.0 { 16000.0 / self.peak_rms } else { 1.0 };

        
        for i in 0..FS_ {
            
            let t = i as f32 / FS_ as f32;
            let drf = 0.5 * (1.0 - libm::cosf(2.0 * core::f32::consts::PI * t));
            self.fft_re[i] *= drf * bmi / 32768.0; 
        }

        
        luw(&mut self.fft_re, &mut self.fft_im);

        
        let gpt = ctu(&self.fft_re, &self.fft_im, ANH_.0, ANH_.1);
        let biu = ctu(&self.fft_re, &self.fft_im, AND_.0, AND_.1);
        let ixz = ctu(&self.fft_re, &self.fft_im, ANF_.0, ANF_.1);
        let cda = ctu(&self.fft_re, &self.fft_im, ANG_.0, ANG_.1);
        let ixy = ctu(&self.fft_re, &self.fft_im, ANE_.0, ANE_.1);
        let iya = ctu(&self.fft_re, &self.fft_im, ANI_.0, ANI_.1);

        
        let obo = gpt * 1.5 + biu * 1.2 + ixz * 0.8
            + cda * 0.5 + ixy * 0.3 + iya * 0.2;

        
        self.sub_bass = Self::cqu(self.sub_bass, gpt.min(1.0), 0.75, 0.10);
        self.bass     = Self::cqu(self.bass,     biu.min(1.0),     0.70, 0.10);
        self.low_mid  = Self::cqu(self.low_mid,  ixz.min(1.0),  0.65, 0.12);
        self.mid      = Self::cqu(self.mid,      cda.min(1.0),      0.60, 0.12);
        self.high_mid = Self::cqu(self.high_mid, ixy.min(1.0), 0.65, 0.14);
        self.treble   = Self::cqu(self.treble,   iya.min(1.0),   0.70, 0.16);
        self.energy   = Self::cqu(self.energy,   obo.min(1.5),   0.65, 0.10);

        
        let diu = gpt + biu * 0.8; 

        
        self.energy_hist[self.hist_idx] = diu;
        self.hist_idx = (self.hist_idx + 1) % self.energy_hist.len();
        if self.hist_count < self.energy_hist.len() {
            self.hist_count += 1;
        }

        
        let oz = self.hist_count.max(1) as f32;
        let ns: f32 = self.energy_hist.iter().take(self.hist_count).sum::<f32>() / oz;

        let mut cex = 0.0f32;
        for i in 0..self.hist_count {
            let jr = self.energy_hist[i] - ns;
            cex += jr * jr;
        }
        let edo = cex / oz;

        
        
        
        let amz = (-15.0 * edo + 1.45).max(1.05).min(1.5);

        
        let nng = diu - self.prev_energy;
        if diu > ns * amz && nng > 0.002 && self.hist_count > 5 {
            let strength = ((diu - ns * amz) / ns.max(0.001)).min(1.0);
            self.beat = (0.6 + strength * 0.4).min(1.0);
        } else {
            self.beat *= 0.88;
            if self.beat < 0.02 { self.beat = 0.0; }
        }
        self.prev_energy = diu;

        
        if self.dbg_frame == 1 || self.dbg_frame % 60 == 0 {
            crate::serial_println!(
                "[BEAT] f={} pos={} gain={:.1} E={:.3} beat={:.2} sub={:.2} bass={:.2} lm={:.2} mid={:.2} hm={:.2} tre={:.2} thr={:.2}",
                self.dbg_frame, center, bmi, self.energy, self.beat,
                self.sub_bass, self.bass, self.low_mid, self.mid, self.high_mid, self.treble, amz
            );
        }
    }

    #[inline]
    fn cqu(prev: f32, new: f32, attack: f32, release: f32) -> f32 {
        if new > prev { prev + (new - prev) * attack }
        else { prev + (new - prev) * release }
    }
}






#[inline(always)]
fn cgi(bg: u32, ko: u32, fg: u32, fb: u32, alpha: u32) -> u32 {
    let ki = 255 - alpha;
    let yi = (bg >> 16) & 0xFF;
    let kby = (bg >> 8) & 0xFF;
    let mq = bg & 0xFF;
    let r = (ko * alpha + yi * ki) / 255;
    let g = (fg * alpha + kby * ki) / 255;
    let b = (fb * alpha + mq * ki) / 255;
    0xFF000000 | (r << 16) | (g << 8) | b
}


#[derive(Clone, Copy)]
struct Qi {
    radius: f32,
    strength: f32,
    alpha: f32,
}

const WI_: usize = 8;
const CSH_: f32 = 5.0;       
const BGQ_: f32 = 35.0;      
const CSE_: f32 = 0.96;      
const CSF_: f32 = 650.0;     



struct HoloOverlay {
    
    snapshot: Vec<u32>,
    snap_w: usize,
    snap_h: usize,
    
    rings: [Qi; WI_],
    ring_count: usize,
    
    sweep: f32,
    
    frame: u32,
    
    prev_beat: f32,
}

impl HoloOverlay {
    fn new(w: u32, h: u32) -> Self {
        Self {
            snapshot: Vec::new(),
            snap_w: w as usize,
            snap_h: h as usize,
            rings: [Qi { radius: 0.0, strength: 0.0, alpha: 0.0 }; WI_],
            ring_count: 0,
            sweep: 0.0,
            frame: 0,
            prev_beat: 0.0,
        }
    }

    
    
    fn capture_snapshot(&mut self) {
        let addr = crate::framebuffer::BL_.load(Ordering::Relaxed);
        if addr.is_null() { return; }
        let pitch = crate::framebuffer::CB_.load(Ordering::Relaxed) as usize;
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

    
    fn restore_snapshot(&self) {
        if self.snapshot.is_empty() { return; }
        if let Some((ptr, _w, _h, _stride)) = crate::framebuffer::aqr() {
            let ae = self.snapshot.len();
            unsafe {
                core::ptr::copy_nonoverlapping(
                    self.snapshot.as_ptr(),
                    ptr as *mut u32,
                    ae,
                );
            }
        }
    }

    
    #[inline]
    fn snap_px(&self, x: i32, y: i32) -> u32 {
        if x < 0 || y < 0 || x as usize >= self.snap_w || y as usize >= self.snap_h {
            return 0xFF000000;
        }
        self.snapshot[y as usize * self.snap_w + x as usize]
    }

    
    fn spawn_ring(&mut self, strength: f32) {
        let dq = Qi {
            radius: 25.0,
            strength: strength * 14.0,
            alpha: 0.95,
        };
        if self.ring_count < WI_ {
            self.rings[self.ring_count] = dq;
            self.ring_count += 1;
        } else {
            
            let mut ld = 0;
            for i in 1..WI_ {
                if self.rings[i].strength < self.rings[ld].strength { ld = i; }
            }
            self.rings[ld] = dq;
        }
    }

    
    fn tick(&mut self, beat: &BeatState) {
        self.frame += 1;

        
        self.sweep += 0.008 + beat.energy * 0.015;
        if self.sweep > 1.0 { self.sweep -= 1.0; }

        
        if beat.beat > 0.35 && self.prev_beat < 0.25 {
            self.spawn_ring(beat.beat);
        }
        self.prev_beat = beat.beat;

        
        let mut w = 0usize;
        for r in 0..self.ring_count {
            self.rings[r].radius += CSH_;
            self.rings[r].strength *= CSE_;
            self.rings[r].alpha *= 0.97;
            if self.rings[r].radius < CSF_ && self.rings[r].alpha > 0.01 {
                if w != r { self.rings[w] = self.rings[r]; }
                w += 1;
            }
        }
        self.ring_count = w;
    }

    
    fn draw_frame(&self, beat: &BeatState) {
        let (fb, w, h) = match crate::framebuffer::aqr() {
            Some((aa, w, h, _)) => (aa as *mut u32, w as usize, h as usize),
            None => return,
        };
        let cx = w / 2;
        let u = h * 42 / 100; 

        
        self.restore_snapshot();

        
        if self.ring_count > 0 {
            self.apply_distortion(fb, w, h, cx, u);
        }

        
        self.draw_glow(w as u32, h as u32, cx as u32, u as u32, beat);

        
        self.draw_logo(fb, w, h, cx, u, beat);

        
        for i in 0..self.ring_count {
            self.draw_ring(fb, w, h, cx, u, &self.rings[i]);
        }
    }

    
    fn apply_distortion(&self, fb: *mut u32, w: usize, h: usize, cx: usize, u: usize) {
        let law = cx as f32;
        let lbb = u as f32;

        
        let mut aug: f32 = 0.0;
        for i in 0..self.ring_count {
            let r = self.rings[i].radius + BGQ_ + 5.0;
            if r > aug { aug = r; }
        }
        let dk = aug as i32;
        let bm = (cx as i32 - dk).max(0) as usize;
        let x1 = ((cx as i32 + dk) as usize).min(w.saturating_sub(1));
        let az = (u as i32 - dk).max(0) as usize;
        let y1 = ((u as i32 + dk) as usize).min(h.saturating_sub(1));

        for y in az..=y1 {
            let ad = y as f32 - lbb;
            for x in bm..=x1 {
                let dx = x as f32 - law;
                let em = libm::sqrtf(dx * dx + ad * ad);
                if em < 1.0 { continue; }

                
                let mut uv: f32 = 0.0;
                for i in 0..self.ring_count {
                    let d = (em - self.rings[i].radius) / BGQ_;
                    if d > 1.0 || d < -1.0 { continue; }
                    
                    let t = 1.0 - d * d;
                    uv += self.rings[i].strength * t * t;
                }
                if uv < 0.5 { continue; }

                
                let ki = 1.0 / em;
                let am = (x as f32 - dx * ki * uv) as i32;
                let ak = (y as f32 - ad * ki * uv) as i32;
                let color = self.snap_px(am, ak);
                unsafe {
                    core::ptr::write(fb.add(y * w + x), color);
                }
            }
        }
    }

    
    fn draw_glow(&self, fo: u32, cxt: u32, cx: u32, u: u32, beat: &BeatState) {
        let intensity = beat.energy * 0.3 + beat.beat * 0.5;
        if intensity < 0.02 { return; }
        let aug = fo.min(cxt) / 4;
        for dq in 0..10u32 {
            let r = (dq + 1) * aug / 10;
            let t = dq as f32 / 10.0;
            let a = ((1.0 - t) * intensity * 18.0) as u32;
            if a < 1 { continue; }
            let left = cx.saturating_sub(r);
            let top = u.saturating_sub(r);
            let lk = (r * 2).min(fo.saturating_sub(left));
            let pp = (r * 2).min(cxt.saturating_sub(top));
            if lk > 0 && pp > 0 {
                crate::framebuffer::co(left, top, lk, pp, 0x00FFCC, a.min(12));
            }
        }
    }

    
    
    fn draw_logo(&self, fb: *mut u32, w: usize, h: usize, cx: usize, u: usize, beat: &BeatState) {
        let mo = crate::logo_bitmap::BA_;
        let ee = crate::logo_bitmap::BN_;
        let dr = ((h as u32) * 45 / ee as u32).max(100);
        let lk = mo as u32 * dr / 100;
        let pp = ee as u32 * dr / 100;
        let da = (cx as u32).saturating_sub(lk / 2);
        let cm = (u as u32).saturating_sub(pp / 2);

        let kq = beat.beat;
        let energy = beat.energy;

        
        let jzw = 0.50 + energy * 0.15 + kq * 0.25;
        
        let eau = (self.sweep * pp as f32) as u32;
        
        let fxf = ((self.frame.wrapping_mul(2654435761) % 100) as f32 / 100.0 - 0.5) * 0.04;
        
        let hjp = 2 + (kq * 3.0) as i32;

        
        for o in 0..pp {
            let ak = (o * 100 / dr) as usize;
            if ak >= ee { continue; }
            let nn = cm + o;
            if nn >= h as u32 { continue; }

            
            let scan = if o % 3 == 0 { 0.35f32 } else { 1.0f32 };
            
            let sd = o as f32 - eau as f32;
            let gwt = if libm::fabsf(sd) < 10.0 { 0.5 * (1.0 - libm::fabsf(sd) / 10.0) } else { 0.0f32 };

            for p in 0..lk {
                let am = (p * 100 / dr) as usize;
                if am >= mo { continue; }
                let lw = da + p;
                if lw >= w as u32 { continue; }

                let abq = crate::logo_bitmap::bhr(am, ak);
                if (abq >> 24) & 0xFF < 20 { continue; }
                let (r, g, b) = ((abq >> 16) & 0xFF, (abq >> 8) & 0xFF, abq & 0xFF);
                let ilj = (r * 77 + g * 150 + b * 29) >> 8;
                if ilj < 28 { continue; } 

                let th = crate::logo_bitmap::dtu(am, ak);
                let dam = ilj as f32 / 255.0;

                
                let (mut czh, mut cze, mut czc) = if th {
                    
                    (dam * 0.25 + kq * 0.15,
                     dam * 1.1 + kq * 0.4,
                     dam * 0.95 + kq * 0.25)
                } else {
                    
                    (dam * 0.12,
                     dam * 0.7 + energy * 0.1,
                     dam * 0.55)
                };

                
                czh = (czh * scan + gwt * 0.2 + fxf).max(0.0);
                cze = (cze * scan + gwt * 0.9 + fxf).max(0.0);
                czc = (czc * scan + gwt * 0.7 + fxf).max(0.0);

                
                if kq > 0.7 {
                    let adr = (kq - 0.7) * 3.0;
                    czh += adr * 0.4;
                    cze += adr * 0.7;
                    czc += adr * 0.5;
                }

                let alg = (czh * 255.0).min(255.0) as u32;
                let ahp = (cze * 255.0).min(255.0) as u32;
                let cb = (czc * 255.0).min(255.0) as u32;
                let jsz = if th { 1.3f32 } else { 1.0f32 };
                let alpha = (jzw * scan * jsz * 255.0).min(255.0) as u32;

                let idx = nn as usize * w + lw as usize;
                unsafe {
                    let bg = core::ptr::read(fb.add(idx));
                    core::ptr::write(fb.add(idx), cgi(bg, alg, ahp, cb, alpha));
                }

                
                if th && alpha > 80 {
                    let hjo = alpha / 4;
                    let fe = lw as i32 - hjp;
                    if fe >= 0 && (fe as usize) < w {
                        let ci = nn as usize * w + fe as usize;
                        unsafe {
                            let bg = core::ptr::read(fb.add(ci));
                            core::ptr::write(fb.add(ci), cgi(bg, 180, 10, 10, hjo));
                        }
                    }
                    let bja = lw as usize + hjp as usize;
                    if bja < w {
                        let ci = nn as usize * w + bja;
                        unsafe {
                            let bg = core::ptr::read(fb.add(ci));
                            core::ptr::write(fb.add(ci), cgi(bg, 10, 10, 200, hjo));
                        }
                    }
                }
            }
        }

        
        let eoo = dr * 106 / 100;
        let fz = mo as u32 * eoo / 100;
        let agl = ee as u32 * eoo / 100;
        let hc = (cx as u32).saturating_sub(fz / 2);
        let jh = (u as u32).saturating_sub(agl / 2);
        let mba = (12.0 + energy * 30.0 + kq * 50.0).min(100.0) as u32;

        for o in (0..agl).step_by(2) {
            let ak = (o * 100 / eoo) as usize;
            if ak >= ee { continue; }
            let jdk = jh + o;
            if jdk >= h as u32 { continue; }
            for p in (0..fz).step_by(2) {
                let am = (p * 100 / eoo) as usize;
                if am >= mo { continue; }
                if !crate::logo_bitmap::dtu(am, ak) { continue; }
                let jdj = hc + p;
                if jdj >= w as u32 { continue; }
                
                for ad in 0..2u32 {
                    for dx in 0..2u32 {
                        let dg = jdj + dx;
                        let hj = jdk + ad;
                        if dg >= w as u32 || hj >= h as u32 { continue; }
                        let idx = hj as usize * w + dg as usize;
                        unsafe {
                            let bg = core::ptr::read(fb.add(idx));
                            core::ptr::write(fb.add(idx), cgi(bg, 0, 255, 204, mba));
                        }
                    }
                }
            }
        }
    }

    
    fn draw_ring(&self, fb: *mut u32, w: usize, h: usize, cx: usize, u: usize, dq: &Qi) {
        if dq.alpha < 0.02 { return; }
        let r = dq.radius;
        let dyz = ((r * 6.28) as u32).max(60).min(720);
        let a = (dq.alpha * 180.0) as u32;

        for j in 0..dyz {
            let cc = j as f32 * 6.2831853 / dyz as f32;
            let p = cx as f32 + r * libm::cosf(cc);
            let o = u as f32 + r * libm::sinf(cc);
            let bi = p as i32;
            let gg = o as i32;
            if bi < 0 || gg < 0 || bi >= w as i32 - 1 || gg >= h as i32 - 1 { continue; }

            
            for ad in 0..2i32 {
                for dx in 0..2i32 {
                    let dg = (bi + dx) as usize;
                    let hj = (gg + ad) as usize;
                    if dg >= w || hj >= h { continue; }
                    let idx = hj * w + dg;
                    unsafe {
                        let bg = core::ptr::read(fb.add(idx));
                        core::ptr::write(fb.add(idx), cgi(bg, 0, 255, 204, a.min(180)));
                    }
                }
            }
        }
    }
}


fn lka(fb_w: u32, fb_h: u32, title: &str, bbi: u32, bee: u32, aed: u32) {
    let aq = 16u32; 

    
    let gr = title.len() as u32 * aq;
    let bu = (fb_w.saturating_sub(gr)) / 2;
    crate::framebuffer::co(bu.saturating_sub(12), 10, gr + 24, 36, 0x000000, 120);
    crate::graphics::scaling::aat(bu as i32, 14, title, 0x00FFCC, 2);

    
    let lpa = bbi / 60;
    let es = bbi % 60;
    let pkh = bee / 60;
    let jy = bee % 60;
    let jmr = format!("{}:{:02} / {}:{:02}", lpa, es, pkh, jy);
    let apj = jmr.len() as u32 * 8 + 16;
    crate::framebuffer::co(8, fb_h.saturating_sub(50), apj, 20, 0x000000, 100);
    crate::framebuffer::draw_text(&jmr, 16, fb_h.saturating_sub(48), 0x00AA88);

    
    let o = fb_h.saturating_sub(24);
    let wl = fb_w.saturating_sub(60);
    let gpd = 30u32;
    crate::framebuffer::fill_rect(gpd, o, wl, 3, 0x001111);
    let oz = wl * aed.min(100) / 100;
    if oz > 0 {
        crate::framebuffer::fill_rect(gpd, o, oz, 3, 0x00FFCC);
        crate::framebuffer::co(gpd, o.saturating_sub(1), oz, 5, 0x00FFCC, 20);
    }

    
    let hint = "[Esc] Exit";
    let xc = hint.len() as u32 * 8 + 8;
    crate::framebuffer::co(
        fb_w.saturating_sub(xc + 8), fb_h.saturating_sub(50), xc, 20, 0x000000, 80,
    );
    crate::framebuffer::draw_text(hint, fb_w.saturating_sub(xc + 4), fb_h.saturating_sub(48), 0x446666);
}












pub fn cma(audio: &[i16], title: &str) -> Result<(), &'static str> {
    crate::audio::init().ok();
    let lmq = audio.len() as f64 / (48000.0 * 2.0);
    crate::serial_println!(
        "[VIZ] Starting holographic overlay: {} ({} samples, {:.1}s)",
        title, audio.len(), lmq
    );

    let fb_w = crate::framebuffer::X_.load(Ordering::Relaxed) as u32;
    let fb_h = crate::framebuffer::W_.load(Ordering::Relaxed) as u32;

    
    if !crate::framebuffer::ajy() {
        crate::framebuffer::adw();
        crate::framebuffer::pr(true);
    }

    
    let mut ayx = HoloOverlay::new(fb_w, fb_h);
    ayx.capture_snapshot();

    crate::serial_println!(
        "[VIZ] Using logo_bitmap: {}x{}",
        crate::logo_bitmap::BA_, crate::logo_bitmap::BN_
    );

    let mut bkv = BeatState::new();
    let plw = audio.len() / 2;
    let total_ms = (plw as u64 * 1000) / 48000;
    let bee = (total_ms / 1000) as u32;

    
    let (dma_ptr, dma_cap) = crate::drivers::hda::cym()
        .ok_or("HDA not initialized")?;
    let aaz = dma_cap / 2;
    let drd = (aaz * 2) as u32;
    let cye = (dma_cap * 2) as u32;

    let are = audio.len().min(dma_cap);
    crate::drivers::hda::bdu(&audio[0..are])?;

    let mut write_cursor: usize = are;
    let mut last_half: u32 = 0;
    let mut audio_exhausted = false;
    let mut jgi: u32 = 0;
    let lgg = (dma_cap as u64 * 1000) / (48000 * 2);
    let vj: u64 = 33;
    let hwz = ((lgg / vj) + 10) as u32;

    crate::serial_println!(
        "[VIZ] DMA: buf={} i16, half={}, exhaust_frames={}",
        dma_cap, aaz, hwz
    );

    
    for f in 0..15u32 {
        ayx.restore_snapshot();
        if f < 8 {
            let a = (f * 30).min(230);
            let bk = "NOW PLAYING";
            let buk = bk.len() as u32 * 16 + 32;
            let cg = (fb_w.saturating_sub(buk)) / 2;
            let cr = fb_h / 2 - 16;
            crate::framebuffer::co(
                cg.saturating_sub(8), cr.saturating_sub(8), buk + 16, 48, 0x000000, a,
            );
            crate::graphics::scaling::aat(cg as i32, cr as i32, bk, 0x00FFCC, 2);
        }
        crate::framebuffer::ii();
        crate::cpu::tsc::hq(50);
    }

    
    let mut vis_frame: u32 = 0;

    loop {
        
        crate::drivers::hda::hli();
        crate::drivers::hda::hwa();

        let alw = crate::drivers::hda::dqq();
        let dav = if alw >= cye { 0 } else { alw };
        let dlx = if dav < drd { 0u32 } else { 1u32 };

        if dlx != last_half {
            if write_cursor < audio.len() {
                let awv = last_half as usize * aaz;
                let ck = audio.len() - write_cursor;
                let od = ck.min(aaz);
                unsafe {
                    core::ptr::copy_nonoverlapping(
                        audio.as_ptr().add(write_cursor),
                        dma_ptr.add(awv),
                        od,
                    );
                    if od < aaz {
                        core::ptr::write_bytes(
                            dma_ptr.add(awv + od), 0, aaz - od,
                        );
                    }
                }
                write_cursor += od;
                if write_cursor >= audio.len() {
                    audio_exhausted = true;
                    crate::serial_println!("[VIZ] Audio data exhausted at frame {}", vis_frame);
                }
            } else {
                let awv = last_half as usize * aaz;
                unsafe {
                    core::ptr::write_bytes(dma_ptr.add(awv), 0, aaz);
                }
            }
            last_half = dlx;
        }

        
        let elapsed_ms = vis_frame as u64 * vj;
        if elapsed_ms >= total_ms { break; }
        let cts = ((elapsed_ms * 48000 * 2) / 1000) as usize;
        let cts = cts.min(audio.len().saturating_sub(2));

        if audio_exhausted {
            jgi += 1;
            if jgi >= hwz { break; }
        }

        let bbi = (elapsed_ms / 1000) as u32;
        let nyr = (elapsed_ms * 100 / total_ms.max(1)) as u32;

        
        bkv.update(audio, cts);

        
        crate::trustdaw::live_viz::oon(
            bkv.beat, bkv.bass, bkv.sub_bass,
            bkv.mid, bkv.high_mid, bkv.treble,
            bkv.energy, vis_frame,
        );

        
        ayx.tick(&bkv);
        ayx.draw_frame(&bkv);

        
        crate::trustdaw::live_viz::ojc();

        lka(fb_w, fb_h, title, bbi, bee, nyr);
        crate::framebuffer::ii();

        vis_frame += 1;

        
        let mut gqz = vj;
        let mut wc = false;
        while gqz > 0 {
            let d = gqz.min(5);
            crate::cpu::tsc::hq(d);
            gqz -= d;
            while let Some(dr) = crate::keyboard::kr() {
                if dr & 0x80 != 0 { continue; }
                if dr == 0x01 { wc = true; break; }
            }
            if wc { break; }
        }
        if wc { break; }
    }

    
    let _ = crate::drivers::hda::stop();

    
    for f in 0..20u32 {
        ayx.restore_snapshot();
        if f < 10 {
            let a = ((20 - f) * 12).min(200);
            let bk = "PLAYBACK COMPLETE";
            let buk = bk.len() as u32 * 16 + 32;
            let cg = (fb_w.saturating_sub(buk)) / 2;
            let cr = fb_h / 2 - 16;
            crate::framebuffer::co(
                cg.saturating_sub(8), cr.saturating_sub(8), buk + 16, 48, 0x000000, a,
            );
            crate::graphics::scaling::aat(cg as i32, cr as i32, bk, 0x00FFCC, 2);
        }
        crate::framebuffer::ii();
        crate::cpu::tsc::hq(50);
    }

    
    ayx.restore_snapshot();
    crate::framebuffer::ii();

    crate::serial_println!("[VIZ] Holographic visualizer closed");
    Ok(())
}








pub fn ivd(path: &str) -> Result<(), &'static str> {
    crate::serial_println!("[VIZ] Loading file: {}", path);

    
    let data: Vec<u8> = if crate::vfs::stat(path).is_ok() {
        
        crate::vfs::read_file(path).map_err(|_| "Failed to read file from VFS")?
    } else {
        
        crate::ramfs::bh(|fs| {
            fs.read_file(path).map(|c| c.to_vec())
        }).map_err(|_| "Failed to read file from VFS or ramfs")?
    };

    if data.is_empty() {
        return Err("File is empty");
    }

    let format = dmx(&data);
    crate::serial_println!("[VIZ] Detected format: {}, size: {} bytes", format, data.len());

    let audio = match format {
        "wav" => byq(&data)?,
        "mp3" => return Err("MP3 not yet supported — convert to WAV first (ffmpeg -i song.mp3 song.wav)"),
        _ => return Err("Unknown audio format — only WAV (16-bit PCM) supported"),
    };

    if audio.is_empty() {
        return Err("Decoded audio is empty");
    }

    let cip = audio.len() as f64 / (48000.0 * 2.0);
    crate::serial_println!("[VIZ] Decoded: {} samples, {:.1}s", audio.len(), cip);

    
    let title = path.rsplit('/').next().unwrap_or(path);
    cma(&audio, title)
}






#[cfg(feature = "daw")]
pub static ZF_: &[u8] = include_bytes!("untitled2.wav");
#[cfg(not(feature = "daw"))]
pub static ZF_: &[u8] = &[];


pub const DDH_: &[&str] = &[
    "/music/untitled2.wav",
    "/mnt/fat32/music/untitled2.wav",
    "/mnt/sda1/music/untitled2.wav",
    "/home/music/untitled2.wav",
];


pub fn gni() -> Result<(), &'static str> {
    
    for path in DDH_ {
        if crate::vfs::stat(path).is_ok() {
            crate::serial_println!("[VIZ] Found '{}' on VFS, loading...", path);
            if let Ok(data) = crate::vfs::read_file(path) {
                if !data.is_empty() {
                    crate::serial_println!("[VIZ] Loaded {} bytes from VFS", data.len());
                    let audio = byq(&data)?;
                    let cip = audio.len() as f64 / (48000.0 * 2.0);
                    crate::serial_println!("[VIZ] Decoded: {} samples, {:.1}s", audio.len(), cip);
                    return cma(&audio, "Untitled (2)");
                }
            }
        }
    }

    
    if let Ok(data) = crate::trustdaw::disk_audio::nai() {
        crate::serial_println!("[VIZ] Loaded {} bytes from data disk", data.len());
        let audio = byq(&data)?;
        let cip = audio.len() as f64 / (48000.0 * 2.0);
        crate::serial_println!("[VIZ] Decoded: {} samples, {:.1}s", audio.len(), cip);
        return cma(&audio, "Untitled (2)");
    }

    
    if !ZF_.is_empty() {
        crate::serial_println!("[VIZ] Using embedded WAV ({} bytes)", ZF_.len());
        let audio = byq(ZF_)?;
        let cip = audio.len() as f64 / (48000.0 * 2.0);
        crate::serial_println!("[VIZ] Decoded: {} samples, {:.1}s", audio.len(), cip);
        return cma(&audio, "Untitled (2)");
    }

    Err("Audio not found — place untitled2.wav in /music/ or build with --features daw")
}


pub const BMS_: &[&str] = &[
    "/music/TrustAnthem.wav",
    "/music/trustanthem.wav",
    "/mnt/fat32/music/TrustAnthem.wav",
    "/mnt/sda1/music/TrustAnthem.wav",
    "/home/music/TrustAnthem.wav",
];


pub fn nvf() -> Result<(), &'static str> {
    
    for path in BMS_ {
        if crate::vfs::stat(path).is_ok() {
            crate::serial_println!("[VIZ] Found '{}' on VFS, loading...", path);
            if let Ok(data) = crate::vfs::read_file(path) {
                if !data.is_empty() {
                    let audio = byq(&data)?;
                    return cma(&audio, "TrustAnthem");
                }
            }
        }
    }

    
    if let Ok(bs) = crate::trustdaw::disk_audio::exz() {
        
        for (i, track) in bs.tracks.iter().enumerate() {
            let gj = track.name.as_str();
            if gj.eq_ignore_ascii_case("TrustAnthem") || gj.eq_ignore_ascii_case("trustanthem") {
                crate::serial_println!("[VIZ] Found TrustAnthem on disk (track {})", i);
                if let Ok((data, _name)) = crate::trustdaw::disk_audio::etf(i) {
                    let audio = byq(&data)?;
                    return cma(&audio, "TrustAnthem");
                }
            }
        }
        
        if let Ok((data, name)) = crate::trustdaw::disk_audio::etf(0) {
            crate::serial_println!("[VIZ] Loading first disk track: '{}'", name);
            let audio = byq(&data)?;
            return cma(&audio, &name);
        }
    }

    Err("TrustAnthem not found — place TrustAnthem.wav on the data disk or in /music/")
}
