







use alloc::boxed::Box;
use alloc::vec::Vec;
use alloc::vec;
use spin::Mutex;
use core::sync::atomic::{AtomicBool, AtomicU32, Ordering};






#[repr(C)]
pub struct Gy {
    pub src: *const u32,
    pub dst: *mut u32,
    pub src_stride: usize,
    pub dst_stride: usize,
    pub width: usize,
    pub height: usize,
}

unsafe impl Send for Gy {}
unsafe impl Sync for Gy {}


fn nwx(start: usize, end: usize, data: *mut u8) {
    let params = unsafe { &*(data as *const Gy) };
    
    for y in start..end {
        if y >= params.height { break; }
        
        let azu = y * params.src_stride;
        let afd = y * params.dst_stride;
        
        unsafe {
            let src = params.src.add(azu);
            let dst = params.dst.add(afd);
            
            #[cfg(target_arch = "x86_64")]
            crate::graphics::simd::blg(dst, src, params.width);
            #[cfg(not(target_arch = "x86_64"))]
            core::ptr::copy_nonoverlapping(src, dst, params.width);
        }
    }
}


fn pvh(start: usize, end: usize, data: *mut u8) {
    let params = unsafe { &*(data as *const Gy) };
    
    for y in start..end {
        if y >= params.height { break; }
        
        let azu = y * params.src_stride;
        let afd = y * params.dst_stride;
        
        unsafe {
            let src = params.src.add(azu);
            let dst = params.dst.add(afd);
            
            #[cfg(target_arch = "x86_64")]
            crate::graphics::simd::blg(dst, src, params.width);
            #[cfg(not(target_arch = "x86_64"))]
            core::ptr::copy_nonoverlapping(src, dst, params.width);
        }
    }
}






#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum LayerType {
    Background = 0,    
    Dock = 1,          
    Windows = 2,       
    Taskbar = 3,       
    Overlay = 4,       
    Cursor = 5,        
}


pub struct Layer {
    pub layer_type: LayerType,
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
    pub buffer: Box<[u32]>,
    pub dirty: AtomicBool,
    pub visible: AtomicBool,
    pub opacity: AtomicU32,  
}

impl Layer {
    
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
    
    
    pub fn set_position(&mut self, x: u32, y: u32) {
        self.x = x;
        self.y = y;
        self.dirty.store(true, Ordering::SeqCst);
    }
    
    
    pub fn clear(&mut self, color: u32) {
        self.buffer.fill(color);
        self.dirty.store(true, Ordering::SeqCst);
    }
    
    
    pub fn fill_rect(&mut self, x: u32, y: u32, w: u32, h: u32, color: u32) {
        let x1 = x.min(self.width);
        let y1 = y.min(self.height);
        let x2 = (x + w).min(self.width);
        let y2 = (y + h).min(self.height);
        
        for o in y1..y2 {
            let fk = (o * self.width + x1) as usize;
            let azm = (o * self.width + x2) as usize;
            if azm <= self.buffer.len() {
                self.buffer[fk..azm].fill(color);
            }
        }
        self.dirty.store(true, Ordering::SeqCst);
    }
    
    
    pub fn draw_rect(&mut self, x: u32, y: u32, w: u32, h: u32, color: u32) {
        
        self.fill_rect(x, y, w, 1, color);
        self.fill_rect(x, y + h.saturating_sub(1), w, 1, color);
        
        self.fill_rect(x, y, 1, h, color);
        self.fill_rect(x + w.saturating_sub(1), y, 1, h, color);
    }
    
    
    #[inline]
    pub fn set_pixel(&mut self, x: u32, y: u32, color: u32) {
        if x < self.width && y < self.height {
            let idx = (y * self.width + x) as usize;
            if idx < self.buffer.len() {
                self.buffer[idx] = color;
            }
        }
    }
    
    
    #[inline]
    pub fn get_pixel(&self, x: u32, y: u32) -> u32 {
        if x < self.width && y < self.height {
            let idx = (y * self.width + x) as usize;
            if idx < self.buffer.len() {
                return self.buffer[idx];
            }
        }
        0
    }
    
    
    pub fn fill_circle(&mut self, cx: u32, u: u32, radius: u32, color: u32) {
        let r = radius as i32;
        let cx = cx as i32;
        let u = u as i32;
        
        for ad in -r..=r {
            for dx in -r..=r {
                if dx * dx + ad * ad <= r * r {
                    let p = cx + dx;
                    let o = u + ad;
                    if p >= 0 && o >= 0 {
                        self.set_pixel(p as u32, o as u32, color);
                    }
                }
            }
        }
        self.dirty.store(true, Ordering::SeqCst);
    }
    
    
    pub fn draw_text(&mut self, text: &str, x: u32, y: u32, color: u32) {
        let mut cx = x;
        for c in text.chars() {
            if c == ' ' {
                cx += 8;
                continue;
            }
            
            let du = crate::framebuffer::font::ol(c);
            for (row, &bits) in du.iter().enumerate() {
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


pub struct Compositor {
    layers: Vec<Layer>,
    screen_width: u32,
    screen_height: u32,
    composite_buffer: Box<[u32]>,
    
    
    gpu_target_ptr: usize,  
    gpu_target_len: usize,
}

impl Compositor {
    
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
    
    
    pub fn add_layer(&mut self, layer_type: LayerType, x: u32, y: u32, w: u32, h: u32) -> usize {
        let bj = Layer::new(layer_type, x, y, w, h);
        self.layers.push(bj);
        self.layers.len() - 1
    }
    
    
    pub fn add_fullscreen_layer(&mut self, layer_type: LayerType) -> usize {
        self.add_layer(layer_type, 0, 0, self.screen_width, self.screen_height)
    }
    
    
    pub fn get_layer_mut(&mut self, index: usize) -> Option<&mut Layer> {
        self.layers.get_mut(index)
    }
    
    
    pub fn get_layer(&self, index: usize) -> Option<&Layer> {
        self.layers.get(index)
    }
    
    
    
    pub fn enable_gpu_direct(&mut self) {
        if crate::drivers::virtio_gpu::sw() {
            if let Some((ptr, w, h)) = crate::drivers::virtio_gpu::eod() {
                self.gpu_target_ptr = ptr as usize;
                self.gpu_target_len = (w * h) as usize;
                crate::serial_println!("[COMPOSITOR] GPU direct mode: composite → GPU buffer (skip 4MB copy!)");
            }
        }
    }
    
    
    
    
    
    pub fn composite(&mut self) {
        
        let (target_ptr, bpi) = if self.gpu_target_ptr != 0 {
            (self.gpu_target_ptr as *mut u32, self.gpu_target_len)
        } else {
            (self.composite_buffer.as_mut_ptr(), self.composite_buffer.len())
        };

        
        let mut grd: Vec<usize> = (0..self.layers.len()).collect();
        grd.sort_by_key(|&i| self.layers[i].layer_type as u8);
        
        
        
        let oto = if let Some(&emt) = grd.first() {
            let bj = &self.layers[emt];
            bj.visible.load(Ordering::SeqCst) 
                && bj.layer_type == LayerType::Background
                && bj.x == 0 && bj.y == 0
                && bj.width >= self.screen_width
                && bj.height >= self.screen_height
                && bj.opacity.load(Ordering::SeqCst) == 255
        } else {
            false
        };
        
        
        if !oto {
            
            #[cfg(target_arch = "x86_64")]
            unsafe {
                crate::graphics::simd::adq(
                    target_ptr,
                    bpi,
                    0xFF000000
                );
            }
            #[cfg(not(target_arch = "x86_64"))]
            unsafe {
                for i in 0..bpi {
                    *target_ptr.add(i) = 0xFF000000;
                }
            }
        }
        
        
        for &xv in &grd {
            let bj = &self.layers[xv];
            if !bj.visible.load(Ordering::SeqCst) {
                continue;
            }
            
            let opacity = bj.opacity.load(Ordering::SeqCst);
            
            
            
            if opacity == 255 && bj.x == 0 && bj.width == self.screen_width 
               && bj.layer_type == LayerType::Background {
                let fbj = bj.height.min(self.screen_height.saturating_sub(bj.y));
                
                
                let params = Gy {
                    src: bj.buffer.as_ptr(),
                    dst: target_ptr,
                    src_stride: bj.width as usize,
                    dst_stride: self.screen_width as usize,
                    width: bj.width as usize,
                    height: fbj as usize,
                };
                
                crate::cpu::smp::bcz(
                    fbj as usize,
                    nwx,
                    &params as *const Gy as *mut u8,
                );
                continue;
            }
            
            
            if opacity == 255 {
                for ly in 0..bj.height {
                    let nn = bj.y + ly;
                    if nn >= self.screen_height {
                        continue;
                    }
                    
                    let zl = (ly * bj.width) as usize;
                    let alj = (nn * self.screen_width + bj.x) as usize;
                    let aoy = bj.width.min(self.screen_width - bj.x) as usize;
                    
                    if bj.x < self.screen_width 
                       && zl + aoy <= bj.buffer.len()
                       && alj + aoy <= bpi {
                        
                        #[cfg(target_arch = "x86_64")]
                        unsafe {
                            crate::graphics::simd::egy(
                                target_ptr.add(alj),
                                bj.buffer.as_ptr().add(zl),
                                aoy,
                            );
                        }
                        #[cfg(not(target_arch = "x86_64"))]
                        {
                            for i in 0..aoy {
                                let bjh = bj.buffer[zl + i];
                                let bvv = (bjh >> 24) & 0xFF;
                                if bvv > 200 {
                                    unsafe { *target_ptr.add(alj + i) = bjh; }
                                } else if bvv > 0 {
                                    let fti = unsafe { *target_ptr.add(alj + i) };
                                    unsafe { *target_ptr.add(alj + i) = ctm(bjh, fti, bvv); }
                                }
                            }
                        }
                    }
                }
                continue;
            }
            
            
            for ly in 0..bj.height {
                let nn = bj.y + ly;
                if nn >= self.screen_height {
                    continue;
                }
                
                let zl = (ly * bj.width) as usize;
                let aoy = bj.width.min(self.screen_width.saturating_sub(bj.x)) as usize;
                let alj = (nn * self.screen_width + bj.x) as usize;
                
                if bj.x >= self.screen_width 
                   || zl + aoy > bj.buffer.len()
                   || alj + aoy > bpi {
                    continue;
                }
                
                
                #[cfg(target_arch = "x86_64")]
                unsafe {
                    kcf(
                        target_ptr.add(alj),
                        bj.buffer.as_ptr().add(zl),
                        aoy,
                        opacity,
                    );
                }
                #[cfg(not(target_arch = "x86_64"))]
                {
                    for i in 0..aoy {
                        let bjh = bj.buffer[zl + i];
                        let bvv = ((bjh >> 24) & 0xFF) as u32;
                        if bvv == 0 { continue; }
                        let cjo = (bvv * opacity) / 255;
                        if cjo >= 255 {
                            unsafe { *target_ptr.add(alj + i) = bjh; }
                        } else if cjo > 0 {
                            let fti = unsafe { *target_ptr.add(alj + i) };
                            unsafe { *target_ptr.add(alj + i) = ctm(bjh, fti, cjo); }
                        }
                    }
                }
            }
        }
        
    }
    
    
    
    
    
    
    
    
    
    
    pub fn present(&self) {
        
        if self.gpu_target_ptr != 0 {
            
            let _ = crate::drivers::virtio_gpu::ivv();
            
            
            return;
        }
        
        
        if crate::drivers::virtio_gpu::sw() {
            if let Some((gpu_ptr, gpu_w, gpu_h)) = crate::drivers::virtio_gpu::eod() {
                let ut = (self.screen_width as usize).min(gpu_w as usize);
                let abw = (self.screen_height as usize).min(gpu_h as usize);
                
                unsafe {
                    let gvt = self.composite_buffer.as_ptr();
                    let lln = gpu_ptr;
                    
                    for y in 0..abw {
                        let src = gvt.add(y * self.screen_width as usize);
                        let dst = lln.add(y * gpu_w as usize);
                        
                        #[cfg(target_arch = "x86_64")]
                        crate::graphics::simd::blg(dst, src, ut);
                        #[cfg(not(target_arch = "x86_64"))]
                        core::ptr::copy_nonoverlapping(src, dst, ut);
                    }
                }
                
                let _ = crate::drivers::virtio_gpu::ivv();
                return;
            }
        }
        
        
        
        self.writeback_mmio_nt();
    }
    
    
    
    
    
    
    pub fn present_only(&self) {
        
        
    }
    
    
    
    
    
    
    
    
    fn writeback_mmio_nt(&self) {
        use crate::framebuffer::{BL_, X_, W_, CB_};
        
        let addr = BL_.load(Ordering::SeqCst);
        if addr.is_null() { return; }
        
        let fb_width = X_.load(Ordering::SeqCst) as usize;
        let fb_height = W_.load(Ordering::SeqCst) as usize;
        let pitch = CB_.load(Ordering::SeqCst) as usize;
        let gne = pitch / 4;
        
        let kxv = fb_width.min(self.screen_width as usize);
        let hnp = fb_height.min(self.screen_height as usize);
        
        
        let gvt = if self.gpu_target_ptr != 0 {
            self.gpu_target_ptr as *const u32
        } else {
            self.composite_buffer.as_ptr()
        };
        
        
        let params = Gy {
            src: gvt,
            dst: addr as *mut u32,
            src_stride: self.screen_width as usize,
            dst_stride: gne,
            width: kxv,
            height: hnp,
        };
        
        crate::cpu::smp::bcz(
            hnp,
            pvh,
            &params as *const Gy as *mut u8,
        );
    }
    
    
    pub fn layer_count(&self) -> usize {
        self.layers.len()
    }
}


#[inline(always)]
fn ctm(src: u32, dst: u32, alpha: u32) -> u32 {
    let sg = 255 - alpha;
    
    let pb = (src >> 16) & 0xFF;
    let akl = (src >> 8) & 0xFF;
    let cv = src & 0xFF;
    
    let qw = (dst >> 16) & 0xFF;
    let afb = (dst >> 8) & 0xFF;
    let fu = dst & 0xFF;
    
    let r = (pb * alpha + qw * sg + 128) >> 8;
    let g = (akl * alpha + afb * sg + 128) >> 8;
    let b = (cv * alpha + fu * sg + 128) >> 8;
    
    0xFF000000 | (r << 16) | (g << 8) | b
}



#[cfg(target_arch = "x86_64")]
#[inline]
unsafe fn kcf(dst: *mut u32, src: *const u32, count: usize, opacity: u32) {
    use core::arch::x86_64::*;
    
    let mut nt = dst;
    let mut ps = src;
    let mut ck = count;
    
    let zero = _mm_setzero_si128();
    let isi = _mm_set1_epi16(opacity as i16);
    let cpo = _mm_set1_epi16(128);
    let ctn = _mm_set1_epi32(0xFF000000u32 as i32);
    let imd = _mm_set1_epi16(255);
    
    while ck >= 4 {
        let j = _mm_loadu_si128(ps as *const __m128i);
        
        
        let ojt = _mm_srli_epi32(j, 24);
        let dhl = _mm_cmpeq_epi32(ojt, zero);
        if _mm_movemask_epi8(dhl) == 0xFFFF {
            ps = ps.add(4);
            nt = nt.add(4);
            ck -= 4;
            continue;
        }
        
        let d = _mm_loadu_si128(nt as *const __m128i);
        
        
        let cpw = _mm_unpacklo_epi8(j, zero);
        let dmf = _mm_unpacklo_epi8(d, zero);
        
        
        let abn = _mm_shufflelo_epi16(cpw, 0xFF);
        let eeq = _mm_shufflehi_epi16(abn, 0xFF);
        let hxs = _mm_srli_epi16(_mm_add_epi16(_mm_mullo_epi16(eeq, isi), cpo), 8);
        let mrh = _mm_sub_epi16(imd, hxs);
        
        let gvw = _mm_mullo_epi16(cpw, hxs);
        let ftj = _mm_mullo_epi16(dmf, mrh);
        let eat = _mm_srli_epi16(_mm_add_epi16(_mm_add_epi16(gvw, ftj), cpo), 8);
        
        
        let cpv = _mm_unpackhi_epi8(j, zero);
        let dme = _mm_unpackhi_epi8(d, zero);
        
        let fy = _mm_shufflelo_epi16(cpv, 0xFF);
        let eep = _mm_shufflehi_epi16(fy, 0xFF);
        let hxr = _mm_srli_epi16(_mm_add_epi16(_mm_mullo_epi16(eep, isi), cpo), 8);
        let mrg = _mm_sub_epi16(imd, hxr);
        
        let gvx = _mm_mullo_epi16(cpv, hxr);
        let dnu = _mm_mullo_epi16(dme, mrg);
        let eas = _mm_srli_epi16(_mm_add_epi16(_mm_add_epi16(gvx, dnu), cpo), 8);
        
        
        let result = _mm_packus_epi16(eat, eas);
        let result = _mm_or_si128(result, ctn);
        _mm_storeu_si128(nt as *mut __m128i, result);
        
        ps = ps.add(4);
        nt = nt.add(4);
        ck -= 4;
    }
    
    
    for _ in 0..ck {
        let bjh = *ps;
        let bvv = ((bjh >> 24) & 0xFF) as u32;
        if bvv > 0 {
            let cjo = (bvv * opacity + 128) >> 8;
            if cjo >= 255 {
                *nt = bjh;
            } else if cjo > 0 {
                *nt = ctm(bjh, *nt, cjo);
            }
        }
        ps = ps.add(1);
        nt = nt.add(1);
    }
}




static Gg: Mutex<Option<Compositor>> = Mutex::new(None);


pub fn init(width: u32, height: u32) {
    let compositor = Compositor::new(width, height);
    *Gg.lock() = Some(compositor);
    crate::serial_println!("[COMPOSITOR] Initialized {}x{}", width, height);
}


pub fn bjz<F, U>(f: F) -> Option<U>
where
    F: FnOnce(&mut Compositor) -> U,
{
    Gg.lock().as_mut().map(f)
}


pub fn rcn<F, U>(f: F) -> Option<U>
where
    F: FnOnce(&Compositor) -> U,
{
    Gg.lock().as_ref().map(f)
}
