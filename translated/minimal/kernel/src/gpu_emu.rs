









use core::sync::atomic::{AtomicU32, AtomicBool, Ordering};
use alloc::vec::Vec;



pub const ZN_: usize = 32;


pub const DFK_: usize = 64;


pub const DXF_: usize = 256;






#[derive(Clone, Copy)]
#[repr(C)]
pub struct Cr {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
    pub time: f32,
    pub frame: u32,
}


#[derive(Clone, Copy, Default)]
#[repr(C)]
pub struct PixelOutput {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl PixelOutput {
    #[inline]
    pub fn to_u32(&self) -> u32 {
        ((self.a as u32) << 24) | ((self.r as u32) << 16) | ((self.g as u32) << 8) | (self.b as u32)
    }
    
    #[inline]
    pub fn aln(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b, a: 255 }
    }
    
    #[inline]
    pub fn from_rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }
}


pub type Ip = fn(input: Cr) -> PixelOutput;


pub type Atq = fn(global_id: u32, local_id: u32, uniforms: *const u8) -> u32;






pub struct VirtualGpu {
    
    frame: AtomicU32,
    
    time_ms: AtomicU32,
    
    active_shader: Option<Ip>,
    
    framebuffer: *mut u32,
    
    width: u32,
    height: u32,
    
    stride: u32,
    
    core_busy: [AtomicBool; ZN_],
    
    work_completed: AtomicU32,
}

unsafe impl Send for VirtualGpu {}
unsafe impl Sync for VirtualGpu {}

impl VirtualGpu {
    
    pub const fn new() -> Self {
        const CFC_: AtomicBool = AtomicBool::new(false);
        Self {
            frame: AtomicU32::new(0),
            time_ms: AtomicU32::new(0),
            active_shader: None,
            framebuffer: core::ptr::null_mut(),
            width: 0,
            height: 0,
            stride: 0,
            core_busy: [CFC_; ZN_],
            work_completed: AtomicU32::new(0),
        }
    }
    
    
    
    pub fn init(&mut self, framebuffer: *mut u32, width: u32, height: u32, stride: u32) {
        self.framebuffer = framebuffer;
        self.width = width;
        self.height = height;
        self.stride = stride;
    }
    
    
    pub fn set_shader(&mut self, shader: Ip) {
        self.active_shader = Some(shader);
    }
    
    
    
    pub fn dispatch_fullscreen(&self) {
        let Some(shader) = self.active_shader else { return };
        
        let width = self.width;
        let height = self.height;
        let fdi = (width * height) as usize;
        let time = self.time_ms.load(Ordering::Relaxed) as f32 / 1000.0;
        let frame = self.frame.load(Ordering::Relaxed);
        
        
        let cpu_count = crate::cpu::smp::ail() as usize;
        let cpu_count = cpu_count.max(1);
        
        
        let qqn = (fdi + cpu_count - 1) / cpu_count;
        
        
        self.work_completed.store(0, Ordering::Release);
        
        
        let ab = Eo {
            shader,
            framebuffer: self.framebuffer,
            width,
            height,
            stride: self.stride,
            time,
            frame,
        };
        
        
        
        crate::cpu::smp::bcz(
            height as usize,
            hsn,
            &ab as *const Eo as *mut u8,
        );
        
        
        self.frame.fetch_add(1, Ordering::Relaxed);
    }
    
    
    #[cfg(target_arch = "x86_64")]
    pub fn dispatch_fullscreen_simd(&self) {
        let Some(shader) = self.active_shader else { return };
        
        let width = self.width;
        let height = self.height;
        let time = self.time_ms.load(Ordering::Relaxed) as f32 / 1000.0;
        let frame = self.frame.load(Ordering::Relaxed);
        
        let ab = Eo {
            shader,
            framebuffer: self.framebuffer,
            width,
            height,
            stride: self.stride,
            time,
            frame,
        };
        
        
        crate::cpu::smp::bcz(
            height as usize,
            hso,
            &ab as *const Eo as *mut u8,
        );
        
        self.frame.fetch_add(1, Ordering::Relaxed);
    }
    
    
    pub fn tick(&self, brv: u32) {
        self.time_ms.fetch_add(brv, Ordering::Relaxed);
    }
    
    
    pub fn frame(&self) -> u32 {
        self.frame.load(Ordering::Relaxed)
    }
    
    
    pub fn time(&self) -> f32 {
        self.time_ms.load(Ordering::Relaxed) as f32 / 1000.0
    }
}














#[inline]
pub fn dispatch_fullscreen(
    framebuffer: *mut u32,
    width: u32,
    height: u32,
    time: f32,
    frame: u32,
    shader: Ip,
) {
    lff(framebuffer, width, height, width, time, frame, shader);
}


pub fn lff(
    framebuffer: *mut u32,
    width: u32,
    height: u32,
    stride: u32,
    time: f32,
    frame: u32,
    shader: Ip,
) {
    let ab = Eo {
        shader,
        framebuffer,
        width,
        height,
        stride,
        time,
        frame,
    };

    
    #[cfg(target_arch = "x86_64")]
    {
        crate::cpu::smp::bcz(
            height as usize,
            hso,
            &ab as *const Eo as *mut u8,
        );
    }

    #[cfg(not(target_arch = "x86_64"))]
    {
        crate::cpu::smp::bcz(
            height as usize,
            hsn,
            &ab as *const Eo as *mut u8,
        );
    }
}






#[repr(C)]
struct Eo {
    shader: Ip,
    framebuffer: *mut u32,
    width: u32,
    height: u32,
    stride: u32,  
    time: f32,
    frame: u32,
}

unsafe impl Send for Eo {}
unsafe impl Sync for Eo {}


fn hsn(start: usize, end: usize, data: *mut u8) {
    let ab = unsafe { &*(data as *const Eo) };
    let shader = ab.shader;
    let fb = ab.framebuffer;
    let width = ab.width;
    let height = ab.height;
    let stride = ab.stride as usize;

    for y in start..end {
        let pq = y * stride;
        for x in 0..width as usize {
            let input = Cr {
                x: x as u32,
                y: y as u32,
                width,
                height,
                time: ab.time,
                frame: ab.frame,
            };

            let output = shader(input);
            unsafe {
                *fb.add(pq + x) = output.to_u32();
            }
        }
    }
}


#[cfg(target_arch = "x86_64")]
fn hso(start: usize, end: usize, data: *mut u8) {
    use core::arch::x86_64::*;
    
    let ab = unsafe { &*(data as *const Eo) };
    let shader = ab.shader;
    let fb = ab.framebuffer;
    let width = ab.width as usize;
    let height = ab.height;
    let stride = ab.stride as usize;
    
    for y in start..end {
        let pq = y * stride;
        
        
        let mut x = 0;
        while x + 4 <= width {
            
            let mut colors = [0u32; 4];
            
            for i in 0..4 {
                let input = Cr {
                    x: (x + i) as u32,
                    y: y as u32,
                    width: width as u32,
                    height,
                    time: ab.time,
                    frame: ab.frame,
                };
                colors[i] = shader(input).to_u32();
            }
            
            
            unsafe {
                let pixels = _mm_loadu_si128(colors.as_ptr() as *const __m128i);
                _mm_storeu_si128(fb.add(pq + x) as *mut __m128i, pixels);
            }
            
            x += 4;
        }
        
        
        while x < width {
            let input = Cr {
                x: x as u32,
                y: y as u32,
                width: width as u32,
                height,
                time: ab.time,
                frame: ab.frame,
            };
            unsafe {
                *fb.add(pq + x) = shader(input).to_u32();
            }
            x += 1;
        }
    }
}






pub fn org(input: Cr) -> PixelOutput {
    let x = input.x as f32 / input.width as f32;
    let y = input.y as f32 / input.height as f32;
    let t = input.time;
    
    
    let v1 = eu(x * 10.0 + t);
    let v2 = eu(y * 10.0 + t * 1.5);
    let v3 = eu((x + y) * 5.0 + t * 0.7);
    let v4 = eu(ra((x - 0.5) * (x - 0.5) + (y - 0.5) * (y - 0.5)) * 10.0 - t * 2.0);
    
    let v = (v1 + v2 + v3 + v4) / 4.0;
    
    
    let r = ((v + 1.0) * 0.5 * 255.0) as u8;
    let g = ((eu(v * 3.14159 + t) + 1.0) * 0.5 * 255.0) as u8;
    let b = ((eu(v * 3.14159 * 2.0 + t * 1.3) + 1.0) * 0.5 * 255.0) as u8;
    
    PixelOutput::aln(r, g, b)
}


pub fn orc(input: Cr) -> PixelOutput {
    let x = input.x;
    let y = input.y;
    let t = input.time;
    let w = input.width;
    let h = input.height;
    
    
    let cell_w = 8u32;
    let cell_h = 16u32;
    let col = x / cell_w;
    let row = y / cell_h;
    let afh = x % cell_w;
    let ta = y % cell_h;
    
    
    let blf = col.wrapping_mul(2654435761);
    let chl = (blf & 0xFFFF) as f32 / 65535.0;
    let kvg = ((blf >> 8) & 0xFFFF) as f32 / 65535.0;
    
    
    let speed = 3.0 + chl * 8.0;           
    let offset = kvg * 50.0;               
    let trail_len = 8.0 + chl * 12.0;       
    
    
    let gah = ((t * speed + offset) % ((h / cell_h + 30) as f32)) as i32 - 15;
    let gry = row as i32;
    
    
    let em = gah - gry;
    
    if em < 0 || em > trail_len as i32 {
        
        return PixelOutput::aln(0, 0, 0);
    }
    
    
    let bab = em as f32 / trail_len;
    let brightness = if em == 0 {
        1.0  
    } else {
        (1.0 - bab) * 0.7
    };
    
    
    let cgt = (col.wrapping_mul(31337) ^ row.wrapping_mul(7919) ^ (input.frame / 3)) as f32;
    let mfe = nzg(afh, ta, cgt as u32);
    
    if !mfe {
        
        let glow = brightness * 0.15;
        let g = (glow * 255.0) as u8;
        return PixelOutput::aln(0, g / 2, g / 4);
    }
    
    
    let agd = brightness * 255.0;
    let r = if em == 0 { (agd * 0.8) as u8 } else { 0 };  
    let g = agd as u8;
    let b = if em == 0 { (agd * 0.8) as u8 } else { (agd * 0.2) as u8 };
    
    PixelOutput::aln(r, g, b)
}


#[inline]
fn nzg(fe: u32, ly: u32, seed: u32) -> bool {
    
    let hash = seed
        .wrapping_mul(2654435761)
        .wrapping_add(fe.wrapping_mul(7919))
        .wrapping_add(ly.wrapping_mul(31337));
    
    
    let nsk = (seed / 7) % 8;
    
    match nsk {
        0 => {
            
            ly % 4 < 2 && fe > 1 && fe < 6
        },
        1 => {
            
            fe == 3 || fe == 4 || (ly % 5 == 0 && fe > 1)
        },
        2 => {
            
            (ly == 2 || ly == 13) && fe > 1 && fe < 6 ||
            (fe == 2 || fe == 5) && ly > 2 && ly < 13
        },
        3 => {
            
            let jr = if fe > ly / 2 { fe - ly / 2 } else { ly / 2 - fe };
            jr < 2
        },
        4 => {
            
            (fe == 3 || fe == 4) && ly > 2 && ly < 14 ||
            (ly == 7 || ly == 8) && fe > 0 && fe < 7
        },
        5 => {
            
            (hash % 3) == 0 && fe > 0 && fe < 7 && ly > 1 && ly < 14
        },
        6 => {
            
            let mid = 4i32;
            let auv = (ly as i32 - 2).max(0).min(6);
            let lfz = (fe as i32 - mid).abs();
            ly > 2 && ly < 14 && lfz <= auv / 2
        },
        _ => {
            
            fe > 0 && fe < 7 && ly > 1 && ly < 14 && (fe + ly) % 3 != 0
        }
    }
}


pub fn orb(input: Cr) -> PixelOutput {
    let zoom = 2.5 + eu(input.time * 0.3) * 0.5;
    let cx = (input.x as f32 / input.width as f32 - 0.7) * zoom;
    let u = (input.y as f32 / input.height as f32 - 0.5) * zoom;
    
    let mut dgx = 0.0f32;
    let mut dgy = 0.0f32;
    let mut iter = 0u32;
    const AGV_: u32 = 64;
    
    while dgx * dgx + dgy * dgy < 4.0 && iter < AGV_ {
        let tmp = dgx * dgx - dgy * dgy + cx;
        dgy = 2.0 * dgx * dgy + u;
        dgx = tmp;
        iter += 1;
    }
    
    if iter == AGV_ {
        PixelOutput::aln(0, 0, 0)
    } else {
        let t = iter as f32 / AGV_ as f32;
        let r = (t * 255.0) as u8;
        let g = (lui(t * 2.0) * 255.0) as u8;
        let b = ((1.0 - t) * 255.0) as u8;
        PixelOutput::aln(r, g, b)
    }
}


pub fn oqy(input: Cr) -> PixelOutput {
    let r = (input.x * 255 / input.width) as u8;
    let g = (input.y * 255 / input.height) as u8;
    let b = ((input.time * 50.0) as u32 % 256) as u8;
    PixelOutput::aln(r, g, b)
}


pub fn oqx(input: Cr) -> PixelOutput {
    let x = input.x as f32;
    let y = input.y as f32;
    let h = input.height as f32;
    let t = input.time;
    
    
    let nkq = eu(x * 0.1 + t * 3.0) * 0.5 + 0.5;
    let nkr = eu(x * 0.17 + t * 2.3) * 0.5 + 0.5;
    let nks = eu(x * 0.23 + y * 0.1 + t * 1.7) * 0.5 + 0.5;
    
    let kab = 1.0 - (y / h);
    let heat = kab * (0.5 + nkq * 0.2 + nkr * 0.2 + nks * 0.1);
    let heat = heat.max(0.0).min(1.0);
    
    
    let (r, g, b) = if heat < 0.2 {
        let t = heat / 0.2;
        ((t * 128.0) as u8, 0, 0)
    } else if heat < 0.5 {
        let t = (heat - 0.2) / 0.3;
        (128 + (t * 127.0) as u8, (t * 100.0) as u8, 0)
    } else if heat < 0.8 {
        let t = (heat - 0.5) / 0.3;
        (255, 100 + (t * 155.0) as u8, (t * 50.0) as u8)
    } else {
        let t = (heat - 0.8) / 0.2;
        (255, 255, 50 + (t * 205.0) as u8)
    };
    
    PixelOutput::aln(r, g, b)
}







pub fn ora(input: Cr) -> PixelOutput {
    let w = input.width as f32;
    let h = input.height as f32;
    let t = input.time;
    
    
    let cx = (input.x as f32 - w * 0.5) / (h * 0.5);  
    let u = (input.y as f32 - h * 0.5) / (h * 0.5);
    
    
    let radius = ra(cx * cx + u * u).max(0.001);
    let cc = emf(u, cx);
    
    
    
    let depth = 1.0 / radius;
    
    
    let z = depth + t * 2.5;
    
    
    
    let avu = (cc / 6.28318 + 0.5) % 1.0;
    let avu = if avu < 0.0 { avu + 1.0 } else { avu };
    
    
    let ptq = z % 1.0;
    
    
    let flc = 0.08;
    let hka = 0.12;  
    
    let bqx = (avu / flc) as u32;
    let aho = (z / hka) as u32;
    let afh = (avu % flc) / flc;
    let ta = (ptq / hka) % 1.0;
    
    
    let cgt = bqx.wrapping_mul(31337) ^ aho.wrapping_mul(7919);
    let icm = jot(afh, ta, cgt);
    
    
    
    let aqm = (radius * 1.5).min(1.0);
    let ldi = (eu(z * 3.0 + t * 4.0) * 0.15 + 0.85);
    let brightness = aqm * ldi;
    
    
    let scanline = if (input.y % 3) == 0 { 0.85 } else { 1.0 };
    
    
    let icw = poj(avu, z, t);
    
    
    let din = if icm {
        brightness * scanline
    } else {
        brightness * 0.08 * scanline  
    };
    
    
    let gzq = (din + icw * 0.4).min(1.0);
    
    
    
    let hrl = (1.0 - aqm) * 0.3;  
    
    let r = (gzq * (80.0 + hrl * 100.0) * brightness) as u8;
    let g = (gzq * 255.0) as u8;
    let b = (gzq * (60.0 + hrl * 150.0 + icw * 80.0)) as u8;
    
    
    let hzd = (z * 8.0 + t * 10.0) % 1.0;
    if icm && hzd < 0.05 && radius < 0.8 {
        let flash = (1.0 - hzd / 0.05) * brightness;
        let ko = (r as f32 + flash * 200.0).min(255.0) as u8;
        let fg = (g as f32 + flash * 50.0).min(255.0) as u8;
        let fb = (b as f32 + flash * 200.0).min(255.0) as u8;
        return PixelOutput::aln(ko, fg, fb);
    }
    
    PixelOutput::aln(r, g, b)
}


#[inline]
fn jot(fe: f32, ly: f32, seed: u32) -> bool {
    let pattern = seed % 12;
    let p = (fe * 8.0) as u32;
    let o = (ly * 12.0) as u32;
    
    match pattern {
        0 => o > 2 && o < 10 && (p == 2 || p == 5),  
        1 => o == 3 || o == 8 || (p == 4 && o > 2 && o < 10),  
        2 => (p + o) % 3 == 0,  
        3 => p > 1 && p < 6 && (o == 2 || o == 9),  
        4 => (p == 3 || p == 4) && o > 1 && o < 11,  
        5 => o > 2 && o < 10 && p > 1 && p < 6 && (o - 2) % 2 == 0,  
        6 => {  
            (o == 2 || o == 9) && p > 1 && p < 6 ||
            (p == 2 || p == 5) && o > 2 && o < 9
        },
        7 => p == 3 && o > 1 && o < 11 || o == 6 && p > 0 && p < 7,  
        8 => (p + o / 2) % 4 == 0,  
        9 => o > 3 && o < 9 && ((p > 1 && p < 4) || (p > 4 && p < 7)),  
        10 => {  
            let center = 3.5;
            let em = if p as f32 > center { p as f32 - center } else { center - p as f32 };
            o > 2 && o < 10 && em < (o - 2) as f32 * 0.4
        },
        _ => (seed.wrapping_mul(p) ^ o) % 3 == 0,  
    }
}


#[inline]
fn poj(avu: f32, z: f32, t: f32) -> f32 {
    
    let oaw = 16.0;
    let ixq = avu * oaw;
    let ixp = zx(ixq - (ixq as i32) as f32 - 0.5);
    let oax = if ixp < 0.08 { (0.08 - ixp) / 0.08 } else { 0.0 };
    
    
    let ohh = 0.3;
    let jaw = z / ohh;
    let ohc = jaw - (jaw as i32) as f32;
    let jau = zx(ohc - 0.5);
    let ohe = if jau < 0.05 { (0.05 - jau) / 0.05 * 0.5 } else { 0.0 };
    
    
    let kq = eu(z * 2.0 - t * 8.0) * 0.3 + 0.7;
    
    (oax * 0.6 + ohe * kq) * 0.8
}


#[inline(always)]
fn emf(y: f32, x: f32) -> f32 { crate::math::emf(y, x) }






pub fn oqz(input: Cr) -> PixelOutput {
    let x = input.x;
    let y = input.y;
    let t = input.time;
    let h = input.height;
    
    
    let mut ech = 0.0f32;
    let mut ecd = 0.0f32;
    let mut ecb = 0.0f32;
    
    
    let (aml, g0, kl) = ewc(x, y, t, h, 0.4, 0.15, 6, 12, 0);
    ech += aml * 0.3;
    ecd += g0 * 0.3;
    ecb += kl * 0.5;  
    
    
    let (uh, bbu, gf) = ewc(x, y, t, h, 0.7, 0.35, 7, 14, 100);
    ech += uh * 0.5;
    ecd += bbu * 0.5;
    ecb += gf * 0.35;
    
    
    let (ju, axe, iq) = ewc(x, y, t, h, 1.0, 0.65, 8, 16, 200);
    ech += ju * 0.7;
    ecd += axe * 0.7;
    ecb += iq * 0.25;
    
    
    let (azf, g3, sc) = ewc(x, y, t, h, 1.5, 1.0, 10, 20, 300);
    ech += azf;
    ecd += g3;
    ecb += sc * 0.2;
    
    
    let scanline = if (y % 2) == 0 { 0.92 } else { 1.0 };
    
    let r = (ech * scanline).min(255.0) as u8;
    let g = (ecd * scanline).min(255.0) as u8;
    let b = (ecb * scanline).min(60.0) as u8;  
    
    PixelOutput::aln(r, g, b)
}


fn ewc(x: u32, y: u32, t: f32, h: u32, speed: f32, brightness: f32, 
                  cell_w: u32, cell_h: u32, seed_offset: u32) -> (f32, f32, f32) {
    let col = x / cell_w;
    let row = y / cell_h;
    let afh = x % cell_w;
    let ta = y % cell_h;
    
    
    let blf = col.wrapping_add(seed_offset).wrapping_mul(2654435761);
    let chl = (blf & 0xFFFF) as f32 / 65535.0;
    
    let fns = speed * (0.7 + chl * 0.6);
    let fnr = ((blf >> 16) & 0xFFFF) as f32 / 65535.0 * 50.0;
    let trail_len = 12.0 + chl * 20.0;
    
    
    let gah = ((t * fns * 8.0 + fnr) % ((h / cell_h + 40) as f32)) as i32 - 20;
    let gry = row as i32;
    let em = gah - gry;
    
    if em < 0 || em > trail_len as i32 {
        return (0.0, 0.0, 0.0);
    }
    
    
    let bab = em as f32 / trail_len;
    let intensity = if em == 0 {
        brightness * 255.0
    } else {
        brightness * (1.0 - bab * bab) * 180.0
    };
    
    
    let cgt = col.wrapping_mul(31337) ^ row.wrapping_mul(7919) ^ seed_offset;
    if !jot(afh as f32 / cell_w as f32, 
                              ta as f32 / cell_h as f32, cgt) {
        return (intensity * 0.05, intensity * 0.1, intensity * 0.03);
    }
    
    
    let (r, g, b) = if em == 0 {
        (intensity * 0.9, intensity, intensity * 0.9)  
    } else if em < 3 {
        (intensity * 0.3, intensity, intensity * 0.1)  
    } else {
        (0.0, intensity, intensity * 0.05)  
    };
    
    (r, g, b)
}





use crate::math::{ra, zx, eu, hr, esw};


#[inline(always)]
fn lui(x: f32) -> f32 {
    x - (x as i32) as f32
}







pub fn orf(input: Cr) -> PixelOutput {
    let w = input.width as f32;
    let h = input.height as f32;
    let t = input.time;
    
    
    let iy = (input.x as f32 / w - 0.5) * 2.0 * (w / h);  
    let v = (input.y as f32 / h - 0.5) * 2.0;
    
    
    let cgp = -3.0;
    let gpw = iy;
    let gpx = v;
    let gpy = 1.5;  
    
    
    let ddd = ra(gpw * gpw + gpx * gpx + gpy * gpy);
    let obz = gpw / ddd;
    let oca = gpx / ddd;
    let ocb = gpy / ddd;
    
    
    let ohs = 0.0;
    let oht = 0.0;
    let ohu = cgp;
    
    
    let mut em = 0.0f32;
    let mut gax = 0u8;  
    let mut gaw = (0.0f32, 0.0f32, 0.0f32);
    
    for _step in 0..24 {  
        let p = ohs + obz * em;
        let o = oht + oca * em;
        let aos = ohu + ocb * em;
        
        
        let (shape_dist, shape_id) = dys(p, o, aos, t);
        
        if shape_dist < 0.02 {  
            gax = shape_id;
            gaw = (p, o, aos);
            break;
        }
        
        em += shape_dist;
        if em > 20.0 { break; }
    }
    
    
    let (awg, awf, awe) = ncm(input.x, input.y, w, h, t);
    
    if gax == 0 {
        
        return PixelOutput::aln(awg, awf, awe);
    }
    
    
    
    let alh = (1.0 - (em - 1.0) / 10.0).clamp(0.2, 1.0);
    let diffuse = alh;
    
    
    let fuc = zx(gaw.0) > 0.4;
    let lof = zx(gaw.1) > 0.4;
    let eld = if fuc || lof { 0.5 } else { 0.0 };
    
    
    let aqm = ((em - 1.0) / 6.0).clamp(0.0, 0.6);
    
    
    let (shape_r, shape_g, shape_b) = match gax {
        1 => {  
            let r = (80.0 * diffuse + eld * 200.0) as u8;
            let g = (255.0 * diffuse + eld * 100.0) as u8;
            let b = (180.0 * diffuse + eld * 255.0) as u8;
            (r, g, b)
        },
        2 => {  
            let kq = eu(t * 3.0) * 0.2 + 0.8;
            let r = (60.0 * diffuse * kq) as u8;
            let g = (255.0 * diffuse * kq) as u8;
            let b = (100.0 * diffuse * kq) as u8;
            (r, g, b)
        },
        _ => (awg, awf, awe)
    };
    
    
    let opacity = 1.0 - aqm;
    let r = esw(awg, shape_r, opacity);
    let g = esw(awf, shape_g, opacity);
    let b = esw(awe, shape_b, opacity);
    
    
    let scan = if input.y % 3 == 0 { 0.9 } else { 1.0 };
    let r = ((r as f32) * scan) as u8;
    let g = ((g as f32) * scan) as u8;
    let b = ((b as f32) * scan) as u8;
    
    PixelOutput::aln(r, g, b)
}


#[inline(always)]
fn dys(x: f32, y: f32, z: f32, t: f32) -> (f32, u8) {
    
    let lag = 2.0 + eu(t * 0.5) * 0.3;
    
    
    let angle_y = t * 0.5;
    let (da, cm, qp) = oib(x, y, z - lag, 0.0, angle_y);
    let hph = omg(da, cm, qp, 0.6);
    
    
    let ouy = hr(t * 0.7) * 1.0;
    let ouz = eu(t * 0.5) * 0.6;
    let ova = 2.5;
    let jhg = omh(x - ouy, y - ouz, z - ova, 0.4);
    
    
    if hph < jhg {
        (hph, 1)
    } else {
        (jhg, 2)
    }
}


#[inline(always)]
fn omh(x: f32, y: f32, z: f32, r: f32) -> f32 {
    ra(x * x + y * y + z * z) - r
}


#[inline(always)]
fn omg(x: f32, y: f32, z: f32, j: f32) -> f32 {
    let dx = zx(x) - j;
    let ad = zx(y) - j;
    let dz = zx(z) - j;
    
    let glk = ra(
        dx.max(0.0) * dx.max(0.0) + 
        ad.max(0.0) * ad.max(0.0) + 
        dz.max(0.0) * dz.max(0.0)
    );
    let bmz = dx.max(ad).max(dz).min(0.0);
    glk + bmz
}


#[inline(always)]
fn quw(x: f32, y: f32, z: f32, gpl: f32, exo: f32) -> f32 {
    let q = ra(x * x + z * z) - gpl;
    ra(q * q + y * y) - exo
}


#[inline(always)]
fn oib(x: f32, y: f32, z: f32, ax: f32, aet: f32) -> (f32, f32, f32) {
    
    let ahs = hr(aet);
    let air = eu(aet);
    let x2 = x * ahs - z * air;
    let qt = x * air + z * ahs;
    
    
    let ahr = hr(ax);
    let aiq = eu(ax);
    let y2 = y * ahr - qt * aiq;
    let bxf = y * aiq + qt * ahr;
    
    (x2, y2, bxf)
}


fn quv(x: f32, y: f32, z: f32, t: f32) -> (f32, f32, f32) {
    let eps = 0.001;
    let (d, _) = dys(x, y, z, t);
    let (dx, _) = dys(x + eps, y, z, t);
    let (ad, _) = dys(x, y + eps, z, t);
    let (dz, _) = dys(x, y, z + eps, t);
    
    let nx = dx - d;
    let re = ad - d;
    let wi = dz - d;
    let len = ra(nx * nx + re * re + wi * wi).max(0.0001);
    (nx / len, re / len, wi / len)
}


fn eld(x: f32, y: f32, z: f32, _t: f32) -> f32 {
    let cxb = 0.02;
    let ax = zx(x);
    let aet = zx(y);
    let did = zx(z);
    
    
    let nmz = (zx(ax - 0.5) < cxb) && (zx(aet - 0.5) < cxb);
    let nna = (zx(ax - 0.5) < cxb) && (zx(did - 0.5) < cxb);
    let nnb = (zx(aet - 0.5) < cxb) && (zx(did - 0.5) < cxb);
    
    if nmz || nna || nnb {
        1.0
    } else {
        0.0
    }
}


fn ncm(x: u32, y: u32, w: f32, h: f32, t: f32) -> (u8, u8, u8) {
    let col = (x as f32 / 16.0) as u32;
    let row = (y as f32 / 18.0) as u32;
    
    
    let seed = col.wrapping_mul(31337) ^ 0xDEAD;
    let lug = 0.3 + ((seed % 100) as f32 / 100.0) * 0.7;
    let nuf = (seed % 1000) as f32 / 100.0;
    
    
    let emc = (t * lug + nuf) % 1.5;
    let eza = y as f32 / h;
    
    
    let drh = (emc - eza).abs();
    let erl = drh < 0.03;
    
    
    let ecn = 0.3;
    let igf = eza < emc && (emc - eza) < ecn;
    let cep = if igf { 1.0 - (emc - eza) / ecn } else { 0.0 };
    
    
    let cgt = col.wrapping_mul(7919) ^ row.wrapping_mul(31337);
    let ick = (cgt % 3) != 0;
    
    if erl && ick {
        (220, 255, 220)  
    } else if igf && ick {
        let g = (200.0 * cep) as u8;
        let r = (50.0 * cep) as u8;
        let b = (80.0 * cep) as u8;
        (r, g, b)
    } else {
        (5, 15, 8)  
    }
}





use spin::Mutex;

static Hd: Mutex<VirtualGpu> = Mutex::new(VirtualGpu::new());



pub fn init(framebuffer: *mut u32, width: u32, height: u32) {
    Hd.lock().init(framebuffer, width, height, width);
    crate::serial_println!("[VGPU] Initialized {}x{} virtual GPU ({} virtual cores)", 
        width, height, ZN_);
}


pub fn mpm(framebuffer: *mut u32, width: u32, height: u32, stride: u32) {
    Hd.lock().init(framebuffer, width, height, stride);
    crate::serial_println!("[VGPU] Initialized {}x{} stride={} virtual GPU ({} virtual cores)", 
        width, height, stride, ZN_);
}


pub fn set_shader(shader: Ip) {
    Hd.lock().set_shader(shader);
}







pub fn ord(input: Cr) -> PixelOutput {
    let w = input.width as f32;
    let h = input.height as f32;
    let t = input.time;
    
    
    let cx = (input.x as f32 - w * 0.5) / (h * 0.5);
    let u = (input.y as f32 - h * 0.5) / (h * 0.5);
    
    
    let radius = ra(cx * cx + u * u).max(0.001);
    let cc = emf(u, cx);
    
    
    let depth = 1.0 / radius;
    
    
    let z = depth + t * 3.0;
    
    
    
    let irk = 32.0;
    let hmk = (cc + 3.14159) / 6.28318;  
    let column = (hmk * irk) as u32;
    let ifx = (hmk * irk) % 1.0;  
    
    
    let blf = column.wrapping_mul(48271);
    let fnr = (blf % 1000) as f32 / 1000.0 * 10.0;
    
    
    let icl = 0.15;
    let fgi = z + fnr;
    let mfg = (fgi / icl) as u32;
    let dqx = (fgi / icl) % 1.0;
    
    
    
    let kvd = zx(ifx - 0.5);
    let mfh = dqx > 0.2 && dqx < 0.8;
    
    
    let ati = 0.3 + radius * 0.2;
    let nmx = kvd < ati;
    
    
    let glyph_seed = mfg.wrapping_mul(31337) ^ column.wrapping_mul(48271);
    let mff = nck(ifx, dqx, glyph_seed);
    
    
    
    let aqm = (radius * 1.8).min(1.0);
    
    
    let erl = dqx < 0.25;
    let cep = if erl { 1.0 } else { 1.0 - (dqx - 0.25) / 0.6 };
    
    
    let scanline = if input.y % 2 == 0 { 0.9 } else { 1.0 };
    
    
    if nmx && mfh && mff {
        
        let intensity = aqm * cep * scanline;
        
        if erl {
            
            let kq = eu(t * 8.0 + fgi * 4.0) * 0.2 + 0.8;
            let r = (200.0 * intensity * kq) as u8;
            let g = (255.0 * intensity) as u8;
            let b = (220.0 * intensity * kq) as u8;
            PixelOutput::aln(r, g, b)
        } else {
            
            let r = (40.0 * intensity) as u8;
            let g = (255.0 * intensity * cep) as u8;
            let b = (80.0 * intensity) as u8;
            PixelOutput::aln(r, g, b)
        }
    } else {
        
        let kbr = (aqm * 0.05 * scanline) as f32;
        let bg = (kbr * 40.0) as u8;
        PixelOutput::aln(0, bg, bg / 2)
    }
}


#[inline]
fn nck(fe: f32, ly: f32, seed: u32) -> bool {
    let p = (fe * 6.0) as u32;
    let o = (ly * 8.0) as u32;
    let pattern = seed % 10;
    
    match pattern {
        0 => p > 1 && p < 5,                              
        1 => o == 2 || o == 5,                            
        2 => (p + o) % 2 == 0,                            
        3 => p == 3 || o == 4,                            
        4 => o > 1 && o < 7 && p > 1 && p < 5,          
        5 => (p == 2 || p == 4) && o > 1 && o < 7,      
        6 => o == 3 || (p == 3 && o > 1 && o < 7),      
        7 => (p + o / 2) % 3 == 0,                        
        8 => o < 4 && p > 1 && p < 5,                    
        _ => (seed.wrapping_mul(p + 1) ^ (o + 1)) % 3 == 0, 
    }
}


pub fn draw() {
    Hd.lock().dispatch_fullscreen();
}


#[cfg(target_arch = "x86_64")]
pub fn ftc() {
    Hd.lock().dispatch_fullscreen_simd();
}


pub fn tick(brv: u32) {
    Hd.lock().tick(brv);
}


pub fn frame() -> u32 {
    Hd.lock().frame()
}


pub fn time() -> f32 {
    Hd.lock().time()
}








pub fn oqw(input: Cr) -> PixelOutput {
    let w = input.width as f32;
    let h = input.height as f32;
    let t = input.time;
    
    
    let cnq = (input.x as f32 * 2.0 - w) / h;
    let ayy = (input.y as f32 * 2.0 - h) / h;
    
    let fss = cnq * cnq + ayy * ayy;
    let l = zx(0.7 - fss);
    
    let j = (1.0 - l) * 5.0;
    let mut vx = cnq * j;
    let mut vy = ayy * j;
    
    let mut dvt: f32 = 0.0;
    let mut dvs: f32 = 0.0;
    let mut dvr: f32 = 0.0;
    
    
    let mut i: f32 = 1.0;
    while i <= 6.0 {
        let dsf = 1.0 / i;
        vx += hr(vy * i + t) * dsf + 0.7;
        vy += hr(vx * i + i + t) * dsf + 0.7;
        
        let jr = zx(vx - vy) * 0.2;
        dvt += (eu(vx) + 1.0) * jr;
        dvs += (eu(vy) + 1.0) * jr;
        dvr += (eu(vy) + 1.0) * jr;
        i += 1.0;
    }
    
    
    let boj = bbo(-4.0 * l);
    let fts = bbo(ayy);
    let ftt = bbo(-ayy);
    let ftu = bbo(ayy * -2.0);
    
    let ko = cjg(fts * boj / (dvt + 0.001));
    let fg = cjg(ftt * boj / (dvs + 0.001));
    let fb = cjg(ftu * boj / (dvr + 0.001));
    
    let r = (zx(ko) * 255.0).min(255.0) as u8;
    let g = (zx(fg) * 255.0).min(255.0) as u8;
    let b = (zx(fb) * 255.0).min(255.0) as u8;
    PixelOutput::aln(r, g, b)
}


#[inline(always)]
fn cjg(x: f32) -> f32 {
    let x2 = x * x;
    x / (1.0 + x.abs() + x2 * 0.28)
}


#[inline(always)]
fn bbo(x: f32) -> f32 {
    let x = x.clamp(-10.0, 10.0);
    let t = 1.0 + x / 256.0;
    let mut r = t;
    
    r = r * r; r = r * r; r = r * r; r = r * r;
    r = r * r; r = r * r; r = r * r; r = r * r;
    r
}


pub fn fyx(name: &str) -> Option<Ip> {
    match name.to_lowercase().as_str() {
        "plasma" => Some(org),
        "matrix" | "rain" => Some(orc),
        "mandelbrot" | "fractal" => Some(orb),
        "gradient" | "test" => Some(oqy),
        "fire" => Some(oqx),
        "tunnel" | "holotunnel" | "3d" => Some(ora),
        "parallax" | "holoparallax" | "depth" => Some(oqz),
        "shapes" | "objects" | "cubes" | "matrix3dshapes" => Some(orf),
        "rain3d" | "matrix3d" | "matrixrain3d" | "fly" => Some(ord),
        "cosmic" | "deform" | "vortex" | "complex" => Some(oqw),
        _ => None,
    }
}


pub fn qnq() -> &'static [&'static str] {
    &["plasma", "matrix", "mandelbrot", "gradient", "fire", "tunnel", "parallax", "shapes", "rain3d", "cosmic"]
}






















const Kd: usize = 8;

const PC_: usize = 240;

const CJU_: usize = 150;

const Mo: usize = 4;

const BDF_: usize = 10;
const PH_: usize = 45;

const CKA_: usize = 64;


static CJW_: [u8; 64] = [
    255, 250, 244, 238, 232, 225, 218, 211,
    204, 196, 189, 181, 174, 166, 158, 150,
    143, 135, 128, 121, 114, 107, 100,  94,
     88,  82,  76,  71,  66,  61,  56,  52,
     48,  44,  40,  37,  34,  31,  28,  26,
     24,  22,  20,  18,  16,  15,  14,  13,
     12,  11,  10,   9,   8,   7,   6,   5,
      5,   4,   4,   3,   3,   3,   2,   2,
];



const fn mbv() -> [u32; 256] {
    let mut bhu = [0xFF010201u32; 256]; 
    let mut i = 1u32;
    while i < 256 {
        let c = if i > 250 {
            
            let w = 200 + ((i - 250) * 10) as u32;
            let w = if w > 255 { 255 } else { w };
            (0xFF << 24) | (w << 16) | (255 << 8) | w
        } else if i > 200 {
            
            let f = i - 200;
            let r = f * 3 / 2;
            let g = 200 + f;
            let b = f / 2;
            (0xFF << 24) | (r << 16) | (g << 8) | b
        } else if i > 140 {
            
            let g = 130 + (i - 140) * 7 / 6;
            let r = (i - 140) / 6;
            let b = (i - 140) / 8;
            (0xFF << 24) | (r << 16) | (g << 8) | b
        } else if i > 80 {
            
            let g = 60 + (i - 80) * 7 / 6;
            let b = (i - 80) / 4;
            (0xFF << 24) | (g << 8) | b
        } else if i > 30 {
            
            let g = 20 + (i - 30) * 4 / 5;
            (0xFF << 24) | (g << 8)
        } else if i > 10 {
            
            let g = 6 + (i - 10) * 7 / 10;
            (0xFF << 24) | (g << 8)
        } else {
            
            let g = 2 + i / 2;
            (0xFF << 24) | (g << 8)
        };
        bhu[i as usize] = c;
        i += 1;
    }
    bhu
}

static CJJ_: [u32; 256] = mbv();


#[inline(always)]
fn imy(intensity: u8) -> u32 {
    CJJ_[intensity as usize]
}



const fn mbw() -> [u32; 256] {
    let mut bhu = [0xFF010201u32; 256];
    let mut i = 1u32;
    while i < 256 {
        
        let c = if i > 240 {
            
            let w = 220 + ((i - 240) * 2);
            let w = if w > 255 { 255 } else { w };
            (0xFF << 24) | (w << 16) | (255 << 8) | w
        } else if i > 180 {
            
            let f = i - 180;
            let r = 100 + f;
            let g = 200 + f / 2;
            let b = 140 + f;
            let r = if r > 255 { 255 } else { r };
            let g = if g > 255 { 255 } else { g };
            let b = if b > 255 { 255 } else { b };
            (0xFF << 24) | (r << 16) | (g << 8) | b
        } else if i > 120 {
            
            let f = i - 120;
            let r = 30 + f / 2;
            let g = 130 + f;
            let b = 60 + f;
            (0xFF << 24) | (r << 16) | (g << 8) | b
        } else if i > 60 {
            
            let f = i - 60;
            let g = 60 + f;
            let b = 30 + f / 2;
            let r = f / 4;
            (0xFF << 24) | (r << 16) | (g << 8) | b
        } else if i > 20 {
            
            let g = 20 + (i - 20);
            let b = 10 + (i - 20) / 2;
            (0xFF << 24) | (g << 8) | b
        } else {
            
            let g = 4 + i / 2;
            let b = 2 + i / 3;
            (0xFF << 24) | (g << 8) | b
        };
        bhu[i as usize] = c;
        i += 1;
    }
    bhu
}

static CJT_: [u32; 256] = mbw();


#[inline(always)]
fn nff(intensity: u8) -> u32 {
    CJT_[intensity as usize]
}


#[derive(Clone, Copy)]
struct MDrop {
    y: i16,              
    speed: u8,           
    counter: u8,         
    trail_len: u8,       
    glyph_seed: u32,     
    active: bool,
    
    
    locked_mask: u64,          
    locked_glyphs: [u8; 48],   
}

impl MDrop {
    const fn new() -> Self {
        Self {
            y: -100, speed: 2, counter: 0, trail_len: 20,
            glyph_seed: 0, active: false,
            locked_mask: 0, locked_glyphs: [0u8; 48],
        }
    }

    
    #[inline(always)]
    fn glyph_at(&self, tp: usize) -> usize {
        if tp < 48 && (self.locked_mask >> tp) & 1 != 0 {
            self.locked_glyphs[tp] as usize
        } else {
            let gs = self.glyph_seed.wrapping_add(tp as u32 * 2654435761);
            (gs % CKA_ as u32) as usize
        }
    }

    
    #[inline(always)]
    fn is_locked(&self, tp: usize) -> bool {
        tp < 48 && (self.locked_mask >> tp) & 1 != 0
    }
}


pub struct ShaderMatrixState {
    drops: [[MDrop; Mo]; PC_],
    col_depth: [u8; PC_],   
    num_cols: usize,
    num_rows: usize,
    frame: u32,
    rng: u32,
    initialized: bool,
}

impl ShaderMatrixState {
    pub const fn new() -> Self {
        Self {
            drops: [[MDrop::new(); Mo]; PC_],
            col_depth: [128u8; PC_],
            num_cols: 0,
            num_rows: 0,
            frame: 0,
            rng: 0xDEADBEEF,
            initialized: false,
        }
    }

    
    pub fn init(&mut self, screen_w: usize, screen_h: usize) {
        self.num_cols = (screen_w / Kd).min(PC_);
        self.num_rows = (screen_h / Kd).min(CJU_);
        self.frame = 0;
        self.rng = 0xDEADBEEF;

        
        for col in 0..self.num_cols {
            self.rng = self.rng.wrapping_mul(1103515245).wrapping_add(12345);
            
            let pattern = ((col * 17 + 53) % 97) as i32 - 48; 
            let gpp = (self.rng % 100) as i32 - 50;          
            let depth = (145i32 + pattern + gpp).clamp(20, 255) as u8;
            self.col_depth[col] = depth;
        }

        
        for col in 0..self.num_cols {
            let depth = self.col_depth[col];
            let brx = depth as u32; 

            let mut next_offset: i32 = 0;
            for di in 0..Mo {
                self.rng = self.rng.wrapping_mul(1103515245).wrapping_add(12345);

                
                let cmw = BDF_ as u32 + brx / 8;           
                let gha = (PH_ as u32).min(cmw + 20);  
                let wr = cmw + (self.rng % (gha - cmw + 1));

                self.rng = self.rng.wrapping_mul(1103515245).wrapping_add(12345);

                
                let mbd = 3u32.saturating_sub(brx / 128);    
                let dql = 2 + (255 - brx) / 50;             
                let gap = mbd + (self.rng % dql);

                let start_y = next_offset - (self.rng % 6) as i32;
                next_offset = start_y - wr as i32 - gap as i32;

                self.rng = self.rng.wrapping_mul(1103515245).wrapping_add(12345);

                
                let cqx = 1 + (255u32.saturating_sub(brx)) / 128; 
                let cqy = 1 + (255u32.saturating_sub(brx)) / 80; 
                let speed = cqx + (self.rng % cqy);

                self.rng = self.rng.wrapping_mul(1103515245).wrapping_add(12345);

                self.drops[col][di] = MDrop {
                    y: start_y as i16,
                    speed: speed.min(8) as u8,
                    counter: (self.rng % speed) as u8,
                    trail_len: wr.min(PH_ as u32) as u8,
                    glyph_seed: self.rng,
                    active: true,
                    locked_mask: 0,
                    locked_glyphs: [0u8; 48],
                };
            }
        }

        self.initialized = true;
    }

    
    pub fn update(&mut self) {
        self.frame = self.frame.wrapping_add(1);
        let aye = self.num_rows as i32 + PH_ as i32 + 10;

        for col in 0..self.num_cols {
            let depth = self.col_depth[col];
            let brx = depth as u32;

            for di in 0..Mo {
                let drop = &mut self.drops[col][di];
                if !drop.active { continue; }

                
                drop.counter = drop.counter.wrapping_add(1);
                if drop.counter >= drop.speed {
                    drop.counter = 0;
                    drop.y += 1;
                    
                    drop.glyph_seed = drop.glyph_seed.wrapping_mul(1103515245).wrapping_add(12345);

                    
                    
                    
                    
                    let gyz = drop.trail_len as usize;
                    if gyz >= 2 && gyz <= 48 {
                        let mut iwa = drop.glyph_at(0) as u8;
                        for tp in 1..gyz {
                            let eji = drop.glyph_at(tp) as u8;
                            if eji == iwa {
                                
                                drop.locked_mask |= (1u64 << (tp - 1)) | (1u64 << tp);
                                drop.locked_glyphs[tp - 1] = eji;
                                drop.locked_glyphs[tp] = eji;
                            }
                            iwa = eji;
                        }
                    }
                }

                
                if drop.y as i32 > aye {
                    self.rng = self.rng.wrapping_mul(1103515245).wrapping_add(12345);

                    let cmw = BDF_ as u32 + brx / 8;
                    let gha = (PH_ as u32).min(cmw + 20);
                    let wr = cmw + (self.rng % (gha - cmw + 1));

                    self.rng = self.rng.wrapping_mul(1103515245).wrapping_add(12345);
                    let gap = 2 + (self.rng % 6);
                    let afk = -(wr as i32) - gap as i32 - (self.rng % 8) as i32;

                    self.rng = self.rng.wrapping_mul(1103515245).wrapping_add(12345);
                    let cqx = 1 + (255u32.saturating_sub(brx)) / 128;
                    let cqy = 1 + (255u32.saturating_sub(brx)) / 80;
                    let speed = cqx + (self.rng % cqy);

                    self.rng = self.rng.wrapping_mul(1103515245).wrapping_add(12345);
                    drop.y = afk as i16;
                    drop.speed = speed.min(8) as u8;
                    drop.counter = 0;
                    drop.trail_len = wr.min(PH_ as u32) as u8;
                    drop.glyph_seed = self.rng;
                    
                    drop.locked_mask = 0;
                    drop.locked_glyphs = [0u8; 48];
                }
            }
        }
    }
}



#[repr(C)]
struct Mq {
    state: *const ShaderMatrixState,
    fb: *mut u32,
    fb_width: usize,
    fb_height: usize,
}

unsafe impl Send for Mq {}
unsafe impl Sync for Mq {}



fn ncl(start: usize, end: usize, data: *mut u8) {
    let ab = unsafe { &*(data as *const Mq) };
    let state = unsafe { &*ab.state };
    let fb = ab.fb;
    let fo = ab.fb_width;
    let cxt = ab.fb_height;
    let num_rows = state.num_rows;

    
    let glyphs = &crate::matrix_fast::CIL_;

    for col in start..end {
        let depth = state.col_depth[col] as u32;
        
        let dms = 100 + (depth * 155 / 255);

        for di in 0..Mo {
            let drop = &state.drops[col][di];
            if !drop.active { continue; }

            let head_y = drop.y as i32;
            let trail_len = drop.trail_len as usize;

            for tp in 0..trail_len {
                let aho = head_y - tp as i32;
                if aho < 0 || aho >= num_rows as i32 { continue; }

                
                let nbf = (tp * 63) / trail_len.max(1);
                let kad = CJW_[nbf.min(63)] as u32;
                let mut intensity = ((kad * dms) / 255).min(255) as u8;
                if intensity < 2 { continue; }

                
                let locked = drop.is_locked(tp);
                let axi = drop.glyph_at(tp);
                let du = &glyphs[axi];

                
                
                
                
                let color = if tp == 0 {
                    imy(intensity.max(250))
                } else if locked {
                    
                    intensity = intensity.saturating_add(60).min(255);
                    nff(intensity)
                } else {
                    imy(intensity)
                };

                
                let p = col * Kd + 1;
                let o = aho as usize * Kd + 1;

                
                if o + 6 <= cxt && p + 6 <= fo {
                    
                    for row in 0..6 {
                        let bits = du[row];
                        if bits == 0 { continue; }
                        let bop = (o + row) * fo + p;
                        unsafe {
                            if bits & 0b000001 != 0 { *fb.add(bop)     = color; }
                            if bits & 0b000010 != 0 { *fb.add(bop + 1) = color; }
                            if bits & 0b000100 != 0 { *fb.add(bop + 2) = color; }
                            if bits & 0b001000 != 0 { *fb.add(bop + 3) = color; }
                            if bits & 0b010000 != 0 { *fb.add(bop + 4) = color; }
                            if bits & 0b100000 != 0 { *fb.add(bop + 5) = color; }
                        }
                    }
                }
            }
        }
    }
}



static CWH_: spin::Mutex<ShaderMatrixState> =
    spin::Mutex::new(ShaderMatrixState::new());










pub fn ore(fb: *mut u32, width: usize, height: usize) {
    let mut state = CWH_.lock();

    
    if !state.initialized || state.num_cols != width / Kd || state.num_rows != height / Kd {
        state.init(width, height);
    }

    
    state.update();

    
    let fdi = width * height;
    unsafe {
        #[cfg(target_arch = "x86_64")]
        crate::graphics::simd::adq(fb, fdi, 0xFF010201);
        #[cfg(not(target_arch = "x86_64"))]
        {
            for i in 0..fdi {
                *fb.add(i) = 0xFF010201u32;
            }
        }
    }

    
    let num_cols = state.num_cols;
    let ab = Mq {
        state: &*state as *const ShaderMatrixState,
        fb,
        fb_width: width,
        fb_height: height,
    };

    crate::cpu::smp::bcz(
        num_cols,
        ncl,
        &ab as *const Mq as *mut u8,
    );
}