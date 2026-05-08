



use alloc::vec::Vec;
use alloc::vec;
use alloc::string::String;
use alloc::format;
use super::codec::{TvDecoder, TvEncoder};


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

    
    pub fn play_data(&mut self, data: Vec<u8>) -> Result<String, String> {
        let mut aaq = TvDecoder::new(data)
            .ok_or_else(|| String::from("Invalid TrustVideo file"))?;

        let bt = aaq.header.width as u32;
        let ex = aaq.header.height as u32;
        let fps = aaq.header.fps as u64;
        let av = aaq.header.frame_count;
        let vj = if fps > 0 { 1000 / fps } else { 33 };

        let dy = crate::framebuffer::width();
        let dw = crate::framebuffer::height();

        
        let fh = if dy > bt { (dy - bt) / 2 } else { 0 };
        let hk = if dw > ex { (dw - ex) / 2 } else { 0 };

        self.state = PlayState::Playing;
        let mut fxq: u32 = 0;

        
        crate::framebuffer::awo(0xFF000000);
        crate::framebuffer::ii();

        crate::serial_println!("[video] Playing {}x{} @ {}fps, {} frames", bt, ex, fps, av);

        loop {
            
            if let Some(key) = crate::keyboard::kr() {
                if key == 0x1B || key == b'q' {
                    
                    self.state = PlayState::Stopped;
                    break;
                } else if key == b' ' {
                    
                    self.state = if self.state == PlayState::Playing {
                        PlayState::Paused
                    } else {
                        PlayState::Playing
                    };
                }
            }

            if self.state == PlayState::Paused {
                
                let nsn = crate::time::uptime_ms();
                while crate::time::uptime_ms() < nsn + 50 {
                    core::hint::spin_loop();
                }
                continue;
            }

            let cyd = crate::time::uptime_ms();

            if let Some(pixels) = aaq.next_frame() {
                
                Self::kcm(pixels, bt, ex, fh, hk, dy);

                
                Self::lje(fxq, av, fps as u32, dy, dw);

                crate::framebuffer::ii();
                fxq += 1;
            } else if self.loop_playback {
                aaq.rewind();
                continue;
            } else {
                break;
            }

            
            let bb = crate::time::uptime_ms() - cyd;
            if bb < vj {
                let bqb = vj - bb;
                let end = crate::time::uptime_ms() + bqb;
                while crate::time::uptime_ms() < end {
                    core::hint::spin_loop();
                }
            }
        }

        self.state = PlayState::Stopped;
        Ok(format!("Played {} frames", fxq))
    }

    
    fn kcm(pixels: &[u32], bt: u32, ex: u32, fh: u32, hk: u32, dy: u32) {
        let ab = crate::framebuffer::FastPixelContext::new();
        let dw = crate::framebuffer::height();
        for y in 0..ex {
            let ad = hk + y;
            if ad >= dw { break; }
            let fk = (y * bt) as usize;
            for x in 0..bt {
                let dx = fh + x;
                if dx >= dy { break; }
                let p = pixels[fk + x as usize];
                ab.put_pixel(dx as usize, ad as usize, p);
            }
        }
    }

    
    fn lje(current: u32, av: u32, fps: u32, dy: u32, dw: u32) {
        
        crate::framebuffer::fill_rect(0, dw - 20, dy, 20, 0xCC000000);

        
        let ek = dy - 20;
        let progress = if av > 0 {
            ((current as u64 * ek as u64) / av as u64) as u32
        } else { 0 };
        crate::framebuffer::fill_rect(10, dw - 14, ek, 8, 0xFF333333);
        crate::framebuffer::fill_rect(10, dw - 14, progress, 8, 0xFF00AAFF);
    }
}




pub fn qhb(width: u16, height: u16, frames: u32, fps: u16) -> Vec<u8> {
    let mut bzk = TvEncoder::new(width, height, fps);
    let yz = width as usize * height as usize;
    let mut buf = vec![0u32; yz];
    let w = width as usize;
    let h = height as usize;

    for f in 0..frames {
        let t = f as i32;
        for y in 0..h {
            for x in 0..w {
                let dg = (x * 256 / w) as i32;
                let hj = (y * 256 / h) as i32;
                let v1 = cbe((dg.wrapping_mul(3).wrapping_add(t * 4)) as u8);
                let v2 = cbe((hj.wrapping_mul(2).wrapping_add(t * 5)) as u8);
                let v3 = cbe((dg.wrapping_add(hj).wrapping_mul(2).wrapping_add(t * 3)) as u8);
                let v4 = cbe((dg.wrapping_sub(hj).wrapping_add(t * 7)) as u8);
                let ns = (v1 as i32 + v2 as i32 + v3 as i32 + v4 as i32) / 4;
                let zz = ((ns + 128) & 0xFF) as u8;
                let (r, g, b) = iff(zz);
                buf[y * w + x] = 0xFF000000 | ((r as u32) << 16) | ((g as u32) << 8) | b as u32;
            }
        }
        bzk.add_frame(&buf);
        if f % 10 == 0 {
            crate::serial_println!("[video] Encoding frame {}/{}", f + 1, frames);
        }
    }

    bzk.finalize()
}


pub fn qgz(width: u16, height: u16, frames: u32, fps: u16) -> Vec<u8> {
    let mut bzk = TvEncoder::new(width, height, fps);
    let w = width as usize;
    let h = height as usize;
    let yz = w * h;
    let mut heat = vec![0u8; yz]; 
    let mut buf = vec![0u32; yz];
    let mut seed: u32 = 42;

    for f in 0..frames {
        
        for x in 0..w {
            seed = xorshift(seed);
            heat[(h - 1) * w + x] = (seed & 0xFF) as u8;
            
            seed = xorshift(seed);
            heat[(h - 2) * w + x] = ((seed & 0xFF) as u16).min(255) as u8;
        }

        
        for y in 0..h - 2 {
            for x in 0..w {
                let bev = heat[(y + 1) * w + x] as u16;
                let bl = if x > 0 { heat[(y + 1) * w + x - 1] as u16 } else { bev };
                let yi = if x + 1 < w { heat[(y + 1) * w + x + 1] as u16 } else { bev };
                let mq = heat[((y + 2).min(h - 1)) * w + x] as u16;
                let ns = (bev + bl + yi + mq) / 4;
                let kxo = if ns > 2 { ns - 2 } else { 0 };
                heat[y * w + x] = kxo.min(255) as u8;
            }
        }

        
        for i in 0..yz {
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

        bzk.add_frame(&buf);
        if f % 10 == 0 {
            crate::serial_println!("[video] Fire frame {}/{}", f + 1, frames);
        }
    }

    bzk.finalize()
}


pub fn qha(width: u16, height: u16, frames: u32, fps: u16) -> Vec<u8> {
    let mut bzk = TvEncoder::new(width, height, fps);
    let w = width as usize;
    let h = height as usize;
    let yz = w * h;
    let mut buf = vec![0u32; yz];

    let col_w = 8; 
    let xx = w / col_w + 1;
    let mut drops = vec![0i32; xx];
    let mut speeds = vec![0u8; xx];
    let mut seed: u32 = 1337;

    
    for i in 0..xx {
        seed = xorshift(seed);
        drops[i] = -((seed % h as u32) as i32);
        seed = xorshift(seed);
        speeds[i] = 1 + (seed % 4) as u8;
    }

    for f in 0..frames {
        
        for i in 0..yz {
            let p = buf[i];
            let r = ((p >> 16) & 0xFF) * 92 / 100;
            let g = ((p >> 8) & 0xFF) * 92 / 100;
            let b = (p & 0xFF) * 92 / 100;
            buf[i] = 0xFF000000 | (r << 16) | (g << 8) | b;
        }

        
        for c in 0..xx {
            let bxd = c * col_w;
            let ad = drops[c];

            if ad >= 0 && (ad as usize) < h {
                let y = ad as usize;
                
                for p in 0..col_w.min(w - bxd) {
                    buf[y * w + bxd + p] = 0xFFCCFFCC;
                }
                
                for wr in 1..8u32 {
                    let ty = ad - wr as i32;
                    if ty >= 0 && (ty as usize) < h {
                        let intensity = 200 - wr * 20;
                        let g = intensity.min(255);
                        for p in 0..col_w.min(w - bxd) {
                            buf[ty as usize * w + bxd + p] =
                                0xFF000000 | ((g / 4) << 16) | (g << 8) | (g / 4);
                        }
                    }
                }
            }

            drops[c] += speeds[c] as i32;

            
            if drops[c] > (h as i32 + 20) {
                seed = xorshift(seed);
                drops[c] = -((seed % (h as u32 / 2)) as i32);
                seed = xorshift(seed);
                speeds[c] = 1 + (seed % 4) as u8;
            }
        }

        bzk.add_frame(&buf);
        if f % 10 == 0 {
            crate::serial_println!("[video] Matrix frame {}/{}", f + 1, frames);
        }
    }

    bzk.finalize()
}



use crate::draw_utils::jsa as xorshift;





pub fn ofp(aoa: &str, width: u16, height: u16, fps: u16) {
    let vj = if fps > 0 { 1000u64 / fps as u64 } else { 33 };

    let dy = crate::framebuffer::width();
    let dw = crate::framebuffer::height();
    let lk = (width as u32).min(dy) as usize;
    let pp = (height as u32).min(dw) as usize;
    let fh = if dy > lk as u32 { (dy - lk as u32) / 2 } else { 0 } as usize;
    let hk = if dw > pp as u32 { (dw - pp as u32) / 2 } else { 0 } as usize;

    let dxw = lk * pp;
    let mut buf = vec![0u32; dxw];
    let mut frame: u32 = 0;
    let mut seed: u32 = 42;

    
    let mut heat = if aoa == "fire" { vec![0u8; dxw] } else { Vec::new() };

    
    let col_w: usize = 8;
    let xx = lk / col_w + 1;
    let mut drops = vec![0i32; xx];
    let mut speeds = vec![0u8; xx];
    if aoa == "matrix" {
        for i in 0..xx {
            seed = xorshift(seed);
            drops[i] = -((seed % pp as u32) as i32);
            seed = xorshift(seed);
            speeds[i] = 1 + (seed % 4) as u8;
        }
    }

    
    let pu = crate::framebuffer::ajy();
    if !pu {
        crate::framebuffer::adw();
        crate::framebuffer::pr(true);
    }

    
    crate::framebuffer::awo(0xFF000000);
    crate::framebuffer::ii();

    crate::serial_println!("[video] Starting {} render loop ({}x{} centered on {}x{}, backbuffer={})", 
        aoa, lk, pp, dy, dw, crate::framebuffer::ajy());

    loop {
        let cyd = crate::time::uptime_ms();

        
        if let Some(key) = crate::keyboard::kr() {
            if key == 0x1B || key == b'q' { break; }
        }

        
        match aoa {
            "plasma" => izv(&mut buf, lk, pp, frame),
            "fire" => izp(&mut buf, &mut heat, lk, pp, &mut seed),
            "matrix" => izs(&mut buf, lk, pp, &mut drops, &mut speeds, &mut seed, col_w, xx),
            "shader" => izx(&mut buf, lk, pp, frame),
            _ => break,
        }

        
        if let Some((bb_ptr, _bb_w, bb_h, bb_stride)) = crate::framebuffer::aqr() {
            let mq = bb_ptr as *mut u32;
            let aeu = bb_stride as usize;
            for y in 0..pp {
                let ad = hk + y;
                if ad >= bb_h as usize { break; }
                let amv = &buf[y * lk..y * lk + lk];
                unsafe {
                    let dst = mq.add(ad * aeu + fh);
                    core::ptr::copy_nonoverlapping(amv.as_ptr(), dst, lk);
                }
            }
        }
        crate::framebuffer::ii();

        frame = frame.wrapping_add(1);

        
        if frame <= 3 || frame % 60 == 0 {
            crate::serial_println!("[video] frame {} rendered", frame);
        }
    }

    
    if !pu {
        crate::framebuffer::pr(false);
    }

    crate::serial_println!("[video] Stopped after {} frames", frame);
}



pub fn ddk(aoa: &str, width: u16, height: u16, fps: u16, duration_ms: u64) {
    let dy = crate::framebuffer::width();
    let dw = crate::framebuffer::height();
    let lk = (width as u32).min(dy) as usize;
    let pp = (height as u32).min(dw) as usize;
    let fh = if dy > lk as u32 { (dy - lk as u32) / 2 } else { 0 } as usize;
    let hk = if dw > pp as u32 { (dw - pp as u32) / 2 } else { 0 } as usize;

    let dxw = lk * pp;
    let mut buf = vec![0u32; dxw];
    let mut frame: u32 = 0;
    let mut seed: u32 = 42;

    let mut heat = if aoa == "fire" { vec![0u8; dxw] } else { Vec::new() };

    let col_w: usize = 8;
    let xx = lk / col_w + 1;
    let mut drops = vec![0i32; xx];
    let mut speeds = vec![0u8; xx];
    if aoa == "matrix" {
        for i in 0..xx {
            seed = xorshift(seed);
            drops[i] = -((seed % pp as u32) as i32);
            seed = xorshift(seed);
            speeds[i] = 1 + (seed % 4) as u8;
        }
    }

    let pu = crate::framebuffer::ajy();
    if !pu {
        crate::framebuffer::adw();
        crate::framebuffer::pr(true);
    }

    crate::framebuffer::awo(0xFF000000);
    crate::framebuffer::ii();

    
    let rr = crate::cpu::tsc::ey();
    let freq = crate::cpu::tsc::we();
    let acx = if freq > 0 { freq / 1000 * duration_ms } else { u64::MAX };

    loop {
        
        let bb = crate::cpu::tsc::ey().saturating_sub(rr);
        if bb >= acx { break; }

        
        if let Some(key) = crate::keyboard::kr() {
            if key == 0x1B || key == b'q' { break; }
        }

        match aoa {
            "plasma" => izv(&mut buf, lk, pp, frame),
            "fire" => izp(&mut buf, &mut heat, lk, pp, &mut seed),
            "matrix" => izs(&mut buf, lk, pp, &mut drops, &mut speeds, &mut seed, col_w, xx),
            "shader" => izx(&mut buf, lk, pp, frame),
            _ => break,
        }

        if let Some((bb_ptr, _bb_w, bb_h, bb_stride)) = crate::framebuffer::aqr() {
            let mq = bb_ptr as *mut u32;
            let aeu = bb_stride as usize;
            for y in 0..pp {
                let ad = hk + y;
                if ad >= bb_h as usize { break; }
                let amv = &buf[y * lk..y * lk + lk];
                unsafe {
                    let dst = mq.add(ad * aeu + fh);
                    core::ptr::copy_nonoverlapping(amv.as_ptr(), dst, lk);
                }
            }
        }
        crate::framebuffer::ii();
        frame = frame.wrapping_add(1);
    }

    if !pu {
        crate::framebuffer::pr(false);
    }

    crate::serial_println!("[video] Timed demo '{}' stopped after {} frames ({} ms)", aoa, frame, duration_ms);
}

fn izv(buf: &mut [u32], w: usize, h: usize, frame: u32) {
    
    
    let t = frame as i32;
    for y in 0..h {
        for x in 0..w {
            
            let dg = (x * 256 / w) as i32;
            let hj = (y * 256 / h) as i32;
            
            let v1 = cbe((dg.wrapping_mul(3).wrapping_add(t * 4)) as u8);
            let v2 = cbe((hj.wrapping_mul(2).wrapping_add(t * 5)) as u8);
            let v3 = cbe((dg.wrapping_add(hj).wrapping_mul(2).wrapping_add(t * 3)) as u8);
            let v4 = cbe((dg.wrapping_sub(hj).wrapping_add(t * 7)) as u8);
            
            let ns = (v1 as i32 + v2 as i32 + v3 as i32 + v4 as i32) / 4;
            let zz = ((ns + 128) & 0xFF) as u8;
            let (r, g, b) = iff(zz);
            buf[y * w + x] = 0xFF000000 | ((r as u32) << 16) | ((g as u32) << 8) | b as u32;
        }
    }
}


#[inline(always)]
fn cbe(idx: u8) -> i8 {
    
    static Apn: [i8; 256] = {
        let mut t = [0i8; 256];
        let mut i = 0u16;
        while i < 256 {
            
            
            let phase = i as i32; 
            
            let q = (phase & 0xFF) as i32;
            let val = if q < 64 {
                
                (q * 127 / 64) as i8
            } else if q < 128 {
                
                ((128 - q) * 127 / 64) as i8
            } else if q < 192 {
                
                -((q - 128) * 127 / 64) as i8
            } else {
                
                -((256 - q) * 127 / 64) as i8
            };
            t[i as usize] = val;
            i += 1;
        }
        t
    };
    Apn[idx as usize]
}


#[inline(always)]
fn iff(zz: u8) -> (u8, u8, u8) {
    let h = zz as u16;
    let dj = h * 6 / 256; 
    let yt = ((h * 6) % 256) as u8; 
    let q = 255 - yt;
    match dj {
        0 => (255, yt, 0),
        1 => (q, 255, 0),
        2 => (0, 255, yt),
        3 => (0, q, 255),
        4 => (yt, 0, 255),
        _ => (255, 0, q),
    }
}

fn izp(buf: &mut [u32], heat: &mut [u8], w: usize, h: usize, seed: &mut u32) {
    
    for x in 0..w {
        *seed = xorshift(*seed);
        heat[(h - 1) * w + x] = (*seed & 0xFF) as u8;
        *seed = xorshift(*seed);
        heat[(h - 2) * w + x] = ((*seed & 0xFF) as u16).min(255) as u8;
    }
    
    for y in 0..h.saturating_sub(2) {
        for x in 0..w {
            let bev = heat[(y + 1) * w + x] as u16;
            let bl = if x > 0 { heat[(y + 1) * w + x - 1] as u16 } else { bev };
            let yi = if x + 1 < w { heat[(y + 1) * w + x + 1] as u16 } else { bev };
            let mq = heat[((y + 2).min(h - 1)) * w + x] as u16;
            let ns = (bev + bl + yi + mq) / 4;
            heat[y * w + x] = if ns > 2 { (ns - 2).min(255) as u8 } else { 0 };
        }
    }
    
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

fn izs(buf: &mut [u32], w: usize, h: usize,
    drops: &mut [i32], speeds: &mut [u8], seed: &mut u32,
    col_w: usize, xx: usize)
{
    
    for i in 0..w * h {
        let p = buf[i];
        let r = ((p >> 16) & 0xFF) * 90 / 100;
        let g = ((p >> 8) & 0xFF) * 90 / 100;
        let b = (p & 0xFF) * 90 / 100;
        buf[i] = 0xFF000000 | (r << 16) | (g << 8) | b;
    }
    
    for c in 0..xx {
        let bxd = c * col_w;
        let ad = drops[c];
        if ad >= 0 && (ad as usize) < h {
            let y = ad as usize;
            for p in 0..col_w.min(w.saturating_sub(bxd)) {
                buf[y * w + bxd + p] = 0xFFCCFFCC;
            }
            for wr in 1..8u32 {
                let ty = ad - wr as i32;
                if ty >= 0 && (ty as usize) < h {
                    let intensity = (200u32).saturating_sub(wr * 20);
                    let g = intensity.min(255);
                    for p in 0..col_w.min(w.saturating_sub(bxd)) {
                        buf[ty as usize * w + bxd + p] =
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







#[inline(always)]
fn bbo(x: f32) -> f32 {
    
    let x = if x > 10.0 { 10.0 } else if x < -10.0 { -10.0 } else { x };
    
    let a = (1 << 23) as f32 / core::f32::consts::LN_2;
    let b = (1 << 23) as f32 * (127.0 - 0.04367744890362246);
    let v = (a * x + b) as i32;
    f32::from_bits(if v > 0 { v as u32 } else { 0 })
}


#[inline(always)]
fn cjg(x: f32) -> f32 {
    if x > 5.0 { return 1.0; }
    if x < -5.0 { return -1.0; }
    let x2 = x * x;
    x / (1.0 + x.abs() + x2 * 0.28)
}

fn izx(buf: &mut [u32], w: usize, h: usize, frame: u32) {
    
    let scale = 4usize;
    let dy = w / scale;
    let dw = h / scale;

    let t = frame as f32 * 0.03;
    let cm = dw as f32;
    let da = dy as f32;

    
    static Aeh: [f32; 256] = {
        let mut bhu = [0.0f32; 256];
        let mut i = 0;
        while i < 256 {
            
            let cc = (i as f64) * 6.283185307179586 / 256.0;
            
            let a = cc % 6.283185307179586;
            let x = if a > 3.14159265358979 { a - 6.283185307179586 } else { a };
            let x2 = x * x;
            let x3 = x2 * x;
            let cfo = x3 * x2;
            let csy = cfo * x2;
            let ffn = csy * x2;
            let j = x - x3 / 6.0 + cfo / 120.0 - csy / 5040.0 + ffn / 362880.0;
            bhu[i] = j as f32;
            i += 1;
        }
        bhu
    };
    
    #[inline(always)]
    fn fxx(x: f32) -> f32 {
        
        let idx = ((x * (256.0 / 6.2831853)) as i32 & 255) as u8;
        Aeh[idx as usize]
    }
    
    #[inline(always)]
    fn hxw(x: f32) -> f32 {
        let idx = (((x + 1.5707963) * (256.0 / 6.2831853)) as i32 & 255) as u8;
        Aeh[idx as usize]
    }

    for ak in 0..dw {
        let ayy = (ak as f32 * 2.0 - cm) / cm;
        
        let fts = bbo(ayy);
        let ftt = bbo(-ayy);
        let ftu = bbo(ayy * -2.0);

        for am in 0..dy {
            let cnq = (am as f32 * 2.0 - da) / cm;

            let fss = cnq * cnq + ayy * ayy;
            let l = (0.7 - fss).abs();

            let j = (1.0 - l) * 5.0; 
            let mut vx = cnq * j;
            let mut vy = ayy * j;

            let mut dvt: f32 = 0.0;
            let mut dvs: f32 = 0.0;
            let mut dvr: f32 = 0.0;

            
            let mut i: f32 = 1.0;
            while i <= 6.0 {
                let dsf = 1.0 / i;
                vx += hxw(vy * i + t) * dsf + 0.7;
                vy += hxw(vx * i + i + t) * dsf + 0.7;

                let jr = (vx - vy).abs() * 0.2;
                dvt += (fxx(vx) + 1.0) * jr;
                dvs += (fxx(vy) + 1.0) * jr;
                dvr += (fxx(vy) + 1.0) * jr;
                i += 1.0;
            }

            let boj = bbo(-4.0 * l);
            let ko = cjg(fts * boj / (dvt + 0.001));
            let fg = cjg(ftt * boj / (dvs + 0.001));
            let fb = cjg(ftu * boj / (dvr + 0.001));

            let r = (ko.abs() * 255.0).min(255.0) as u32;
            let g = (fg.abs() * 255.0).min(255.0) as u32;
            let b = (fb.abs() * 255.0).min(255.0) as u32;
            let color = 0xFF000000 | (r << 16) | (g << 8) | b;

            
            for ad in 0..scale {
                let o = ak * scale + ad;
                if o >= h { break; }
                let cpq = o * w + am * scale;
                for dx in 0..scale {
                    let p = am * scale + dx;
                    if p < w {
                        buf[cpq + dx] = color;
                    }
                }
            }
        }
    }
}
