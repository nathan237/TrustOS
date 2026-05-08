








#[cfg(target_arch = "x86_64")]
use core::arch::x86_64::*;







#[cfg(target_arch = "x86_64")]
#[inline]
pub unsafe fn adq(dst: *mut u32, count: usize, color: u32) {
    if count == 0 { return; }
    
    
    let bfl = _mm_set1_epi32(color as i32);
    
    let mut ptr = dst;
    let mut ck = count;
    
    
    let hel = (ptr as usize) & 15; 
    if hel != 0 {
        let nva = ((16 - hel) / 4).min(ck);
        for _ in 0..nva {
            *ptr = color;
            ptr = ptr.add(1);
            ck -= 1;
        }
    }
    
    
    while ck >= 16 {
        _mm_store_si128(ptr as *mut __m128i, bfl);
        _mm_store_si128(ptr.add(4) as *mut __m128i, bfl);
        _mm_store_si128(ptr.add(8) as *mut __m128i, bfl);
        _mm_store_si128(ptr.add(12) as *mut __m128i, bfl);
        ptr = ptr.add(16);
        ck -= 16;
    }
    
    
    while ck >= 4 {
        _mm_store_si128(ptr as *mut __m128i, bfl);
        ptr = ptr.add(4);
        ck -= 4;
    }
    
    
    for _ in 0..ck {
        *ptr = color;
        ptr = ptr.add(1);
    }
}


#[cfg(target_arch = "x86_64")]
#[inline]
pub unsafe fn blg(dst: *mut u32, src: *const u32, count: usize) {
    if count == 0 { return; }
    
    let mut nt = dst;
    let mut ps = src;
    let mut ck = count;
    
    
    while ck >= 16 {
        let v0 = _mm_loadu_si128(ps as *const __m128i);
        let v1 = _mm_loadu_si128(ps.add(4) as *const __m128i);
        let v2 = _mm_loadu_si128(ps.add(8) as *const __m128i);
        let v3 = _mm_loadu_si128(ps.add(12) as *const __m128i);
        
        _mm_storeu_si128(nt as *mut __m128i, v0);
        _mm_storeu_si128(nt.add(4) as *mut __m128i, v1);
        _mm_storeu_si128(nt.add(8) as *mut __m128i, v2);
        _mm_storeu_si128(nt.add(12) as *mut __m128i, v3);
        
        ps = ps.add(16);
        nt = nt.add(16);
        ck -= 16;
    }
    
    
    while ck >= 4 {
        let v = _mm_loadu_si128(ps as *const __m128i);
        _mm_storeu_si128(nt as *mut __m128i, v);
        ps = ps.add(4);
        nt = nt.add(4);
        ck -= 4;
    }
    
    
    for _ in 0..ck {
        *nt = *ps;
        ps = ps.add(1);
        nt = nt.add(1);
    }
}






















#[cfg(target_arch = "x86_64")]
#[inline]
pub unsafe fn eiy(dst: *mut u32, src: *const u32, count: usize) {
    if count == 0 { return; }

    let dst8 = dst as *mut u64;
    let jhl = src as *const u64;
    let ccd = count / 2;  
    let mut i = 0usize;

    
    while i + 8 <= ccd {
        let j = jhl.add(i);
        let d = dst8.add(i);
        let v0 = core::ptr::read_unaligned(j);
        let v1 = core::ptr::read_unaligned(j.add(1));
        let v2 = core::ptr::read_unaligned(j.add(2));
        let v3 = core::ptr::read_unaligned(j.add(3));
        let v4 = core::ptr::read_unaligned(j.add(4));
        let v5 = core::ptr::read_unaligned(j.add(5));
        let v6 = core::ptr::read_unaligned(j.add(6));
        let v7 = core::ptr::read_unaligned(j.add(7));
        core::arch::asm!(
            "movnti [{d}], {v0}",
            "movnti [{d} + 8], {v1}",
            "movnti [{d} + 16], {v2}",
            "movnti [{d} + 24], {v3}",
            "movnti [{d} + 32], {v4}",
            "movnti [{d} + 40], {v5}",
            "movnti [{d} + 48], {v6}",
            "movnti [{d} + 56], {v7}",
            d = in(reg) d,
            v0 = in(reg) v0,
            v1 = in(reg) v1,
            v2 = in(reg) v2,
            v3 = in(reg) v3,
            v4 = in(reg) v4,
            v5 = in(reg) v5,
            v6 = in(reg) v6,
            v7 = in(reg) v7,
            options(nostack),
        );
        i += 8;
    }

    
    while i < ccd {
        let v = core::ptr::read_unaligned(jhl.add(i));
        core::arch::asm!(
            "movnti [{d}], {v}",
            d = in(reg) dst8.add(i),
            v = in(reg) v,
            options(nostack),
        );
        i += 1;
    }

    
    if count & 1 != 0 {
        *dst.add(count - 1) = *src.add(count - 1);
    }

    
    core::arch::asm!("sfence", options(nostack));
}


#[cfg(target_arch = "x86_64")]
#[inline]
pub unsafe fn qfq(dst: *mut u32, count: usize, color: u32) {
    if count == 0 { return; }

    let color64 = (color as u64) | ((color as u64) << 32);
    let dst8 = dst as *mut u64;
    let ccd = count / 2;
    let mut i = 0usize;

    while i + 8 <= ccd {
        let d = dst8.add(i);
        core::arch::asm!(
            "movnti [{d}], {v}",
            "movnti [{d} + 8], {v}",
            "movnti [{d} + 16], {v}",
            "movnti [{d} + 24], {v}",
            "movnti [{d} + 32], {v}",
            "movnti [{d} + 40], {v}",
            "movnti [{d} + 48], {v}",
            "movnti [{d} + 56], {v}",
            d = in(reg) d,
            v = in(reg) color64,
            options(nostack),
        );
        i += 8;
    }

    while i < ccd {
        core::arch::asm!(
            "movnti [{d}], {v}",
            d = in(reg) dst8.add(i),
            v = in(reg) color64,
            options(nostack),
        );
        i += 1;
    }

    if count & 1 != 0 {
        *dst.add(count - 1) = color;
    }

    core::arch::asm!("sfence", options(nostack));
}








#[cfg(target_arch = "x86_64")]
#[inline]
pub unsafe fn egy(dst: *mut u32, src: *const u32, count: usize) {
    let mut nt = dst;
    let mut ps = src;
    let mut ck = count;

    let zero = _mm_setzero_si128();

    
    while ck >= 4 {
        let j = _mm_loadu_si128(ps as *const __m128i);
        let d = _mm_loadu_si128(nt as *const __m128i);

        
        let hey = _mm_srli_epi32(j, 24);
        let dhl = _mm_cmpeq_epi32(hey, zero);
        if _mm_movemask_epi8(dhl) == 0xFFFF {
            
            ps = ps.add(4);
            nt = nt.add(4);
            ck -= 4;
            continue;
        }
        let fgs = _mm_cmpeq_epi32(hey, _mm_set1_epi32(255));
        if _mm_movemask_epi8(fgs) == 0xFFFF {
            
            _mm_storeu_si128(nt as *mut __m128i, j);
            ps = ps.add(4);
            nt = nt.add(4);
            ck -= 4;
            continue;
        }

        
        
        let cpw = _mm_unpacklo_epi8(j, zero); 
        let dmf = _mm_unpacklo_epi8(d, zero); 

        
        
        let abn = _mm_shufflelo_epi16(cpw, 0xFF); 
        let eeq = _mm_shufflehi_epi16(abn, 0xFF);  
        let mrd = _mm_sub_epi16(_mm_set1_epi16(255), eeq);

        
        let gvw = _mm_mullo_epi16(cpw, eeq);
        let ftj = _mm_mullo_epi16(dmf, mrd);
        let eat = _mm_add_epi16(_mm_add_epi16(gvw, ftj), _mm_set1_epi16(128));
        let grg = _mm_srli_epi16(eat, 8);

        
        let cpv = _mm_unpackhi_epi8(j, zero);
        let dme = _mm_unpackhi_epi8(d, zero);

        let fy = _mm_shufflelo_epi16(cpv, 0xFF);
        let eep = _mm_shufflehi_epi16(fy, 0xFF);
        let mrc = _mm_sub_epi16(_mm_set1_epi16(255), eep);

        let gvx = _mm_mullo_epi16(cpv, eep);
        let dnu = _mm_mullo_epi16(dme, mrc);
        let eas = _mm_add_epi16(_mm_add_epi16(gvx, dnu), _mm_set1_epi16(128));
        let grf = _mm_srli_epi16(eas, 8);

        
        let result = _mm_packus_epi16(grg, grf);
        
        let result = _mm_or_si128(result, _mm_set1_epi32(0xFF000000u32 as i32));
        _mm_storeu_si128(nt as *mut __m128i, result);

        ps = ps.add(4);
        nt = nt.add(4);
        ck -= 4;
    }

    
    for _ in 0..ck {
        let alpha = (*ps >> 24) as u32;
        if alpha == 255 {
            *nt = *ps;
        } else if alpha > 0 {
            *nt = fjj(*ps, *nt);
        }
        ps = ps.add(1);
        nt = nt.add(1);
    }
}


#[inline(always)]
pub fn fjj(src: u32, dst: u32) -> u32 {
    let alpha = (src >> 24) as u32;
    if alpha == 0 { return dst; }
    if alpha == 255 { return src; }
    
    let sg = 255 - alpha;
    
    let pb = (src >> 16) & 0xFF;
    let akl = (src >> 8) & 0xFF;
    let cv = src & 0xFF;
    
    let qw = (dst >> 16) & 0xFF;
    let afb = (dst >> 8) & 0xFF;
    let fu = dst & 0xFF;
    
    
    
    let r = ((pb * alpha + qw * sg + 128) >> 8).min(255);
    let g = ((akl * alpha + afb * sg + 128) >> 8).min(255);
    let b = ((cv * alpha + fu * sg + 128) >> 8).min(255);
    
    0xFF000000 | (r << 16) | (g << 8) | b
}








#[cfg(target_arch = "x86_64")]
#[inline]
pub unsafe fn hib(dst: *mut u32, count: usize, color: u32, alpha: u32) {
    if count == 0 { return; }
    if alpha == 0 { return; }
    if alpha >= 255 {
        adq(dst, count, color | 0xFF000000);
        return;
    }

    let caz = 255 - alpha;

    
    let crb = _mm_set1_epi32(color as i32);
    let hez = _mm_set1_epi16(alpha as i16);
    let ihf = _mm_set1_epi16(caz as i16);
    let cpo = _mm_set1_epi16(128);
    let zero = _mm_setzero_si128();
    let ctn = _mm_set1_epi32(0xFF000000u32 as i32);

    
    let cpw = _mm_unpacklo_epi8(crb, zero);
    let cpv = _mm_unpackhi_epi8(crb, zero);
    let ovi = _mm_mullo_epi16(cpw, hez);
    let ovh = _mm_mullo_epi16(cpv, hez);

    let mut ptr = dst;
    let mut ck = count;

    
    while ck >= 4 {
        let d = _mm_loadu_si128(ptr as *const __m128i);

        
        let dmf = _mm_unpacklo_epi8(d, zero);
        let llr = _mm_mullo_epi16(dmf, ihf);
        let eat = _mm_add_epi16(_mm_add_epi16(ovi, llr), cpo);
        let grg = _mm_srli_epi16(eat, 8);

        
        let dme = _mm_unpackhi_epi8(d, zero);
        let dnu = _mm_mullo_epi16(dme, ihf);
        let eas = _mm_add_epi16(_mm_add_epi16(ovh, dnu), cpo);
        let grf = _mm_srli_epi16(eas, 8);

        
        let result = _mm_packus_epi16(grg, grf);
        let result = _mm_or_si128(result, ctn);
        _mm_storeu_si128(ptr as *mut __m128i, result);

        ptr = ptr.add(4);
        ck -= 4;
    }

    
    let pb = ((color >> 16) & 0xFF) as u32;
    let akl = ((color >> 8) & 0xFF) as u32;
    let cv = (color & 0xFF) as u32;
    for _ in 0..ck {
        let ku = *ptr;
        let qw = ((ku >> 16) & 0xFF) as u32;
        let afb = ((ku >> 8) & 0xFF) as u32;
        let fu = (ku & 0xFF) as u32;
        let r = ((pb * alpha + qw * caz + 128) >> 8).min(255);
        let g = ((akl * alpha + afb * caz + 128) >> 8).min(255);
        let b = ((cv * alpha + fu * caz + 128) >> 8).min(255);
        *ptr = 0xFF000000 | (r << 16) | (g << 8) | b;
        ptr = ptr.add(1);
    }
}






#[cfg(target_arch = "x86_64")]
pub unsafe fn pzy(fb: *mut u32, width: usize, height: usize, gne: usize, color: u32) {
    for y in 0..height {
        let row = fb.add(y * gne);
        adq(row, width, color);
    }
}


#[cfg(target_arch = "x86_64")]
pub unsafe fn pyw(
    fb: *mut u32,
    cjh: usize,
    src: *const u32,
    src_width: usize,
    fbj: usize,
    dst_x: usize,
    dst_y: usize,
) {
    for y in 0..fbj {
        let amv = src.add(y * src_width);
        let dnw = fb.add((dst_y + y) * cjh + dst_x);
        blg(dnw, amv, src_width);
    }
}






pub struct Oh {
    
    pub pixels: [u32; 128], 
    
    pub width: u8,
    
    pub height: u8,
    
    pub fg_color: u32,
}


pub struct GlyphCache {
    
    glyphs: [Option<Oh>; 128],
    
    current_fg: u32,
}

impl GlyphCache {
    pub const fn new() -> Self {
        const Bc: Option<Oh> = None;
        Self {
            glyphs: [Bc; 128],
            current_fg: 0xFF00FF66, 
        }
    }
    
    
    pub fn bdr(&mut self, color: u32) {
        if self.current_fg != color {
            self.current_fg = color;
            
            for g in &mut self.glyphs {
                *g = None;
            }
        }
    }
    
    
    pub fn ol(&mut self, c: char) -> &Oh {
        let idx = (c as usize) & 127;
        
        if self.glyphs[idx].is_none() || 
           self.glyphs[idx].as_ref().map(|g| g.fg_color) != Some(self.current_fg) {
            
            let dqw = crate::framebuffer::font::ol(c);
            let mut pixels = [0u32; 128];
            
            for (amq, &row) in dqw.iter().enumerate() {
                for bf in 0..8 {
                    if (row >> (7 - bf)) & 1 == 1 {
                        pixels[amq * 8 + bf] = self.current_fg;
                    }
                }
            }
            
            self.glyphs[idx] = Some(Oh {
                pixels,
                width: 8,
                height: 16,
                fg_color: self.current_fg,
            });
        }
        
        
        self.glyphs[idx].as_ref().unwrap_or_else(|| unreachable!())
    }
    
    
    #[inline]
    pub fn draw_glyph_to_buffer(
        &mut self,
        buffer: &mut [u32],
        stride: usize,
        x: usize,
        y: usize,
        c: char,
        fg: u32,
        bg: u32,
    ) {
        let dqw = crate::framebuffer::font::ol(c);
        
        for (amq, &row) in dqw.iter().enumerate() {
            let o = y + amq;
            let fk = o * stride + x;
            
            if fk + 8 > buffer.len() { continue; }
            
            
            for bf in 0..8u8 {
                let color = if (row >> (7 - bf)) & 1 == 1 { fg } else { bg };
                buffer[fk + bf as usize] = color;
            }
        }
    }
}


use spin::Mutex;
pub static CAU_: Mutex<GlyphCache> = Mutex::new(GlyphCache::new());






pub fn ofs(
    buffer: &mut [u32],
    stride: usize,
    x: usize,
    y: usize,
    text: &str,
    fg: u32,
    bg: u32,
) {
    let mut adk = CAU_.lock();
    let mut cx = x;
    
    for c in text.chars() {
        if cx + 8 > stride { break; }
        adk.draw_glyph_to_buffer(buffer, stride, cx, y, c, fg, bg);
        cx += 8;
    }
}


pub fn qtx(
    buffer: &mut [u32],
    stride: usize,
    x: usize,
    y: usize,
    lines: &[&str],
    fg: u32,
    bg: u32,
) {
    let mut u = y;
    for line in lines {
        ofs(buffer, stride, x, u, line, fg, bg);
        u += 16;
    }
}






pub fn hyi(buffer: &mut [u32], color: u32) {
    #[cfg(target_arch = "x86_64")]
    unsafe {
        if buffer.len() >= 4 {
            adq(buffer.as_mut_ptr(), buffer.len(), color);
            return;
        }
    }
    
    buffer.fill(color);
}


pub fn kxq(dst: &mut [u32], src: &[u32]) {
    let count = dst.len().min(src.len());
    #[cfg(target_arch = "x86_64")]
    unsafe {
        if count >= 4 {
            blg(dst.as_mut_ptr(), src.as_ptr(), count);
            return;
        }
    }
    
    dst[..count].copy_from_slice(&src[..count]);
}


pub fn pys(dst: &mut [u32], src: &[u32]) {
    let count = dst.len().min(src.len());
    #[cfg(target_arch = "x86_64")]
    unsafe {
        if count >= 2 {
            egy(dst.as_mut_ptr(), src.as_ptr(), count);
            return;
        }
    }
    
    for i in 0..count {
        dst[i] = fjj(src[i], dst[i]);
    }
}





use core::sync::atomic::{AtomicU8, Ordering};


static ANC_: AtomicU8 = AtomicU8::new(0);


#[cfg(target_arch = "x86_64")]
fn has_avx2() -> bool {
    let bfd = ANC_.load(Ordering::Relaxed);
    if bfd != 0 {
        return bfd == 2;
    }
    let result = unsafe {
        
        let ebx: u32;
        core::arch::asm!(
            "mov {tmp_rbx}, rbx",
            "cpuid",
            "mov {out}, ebx",
            "mov rbx, {tmp_rbx}",
            tmp_rbx = out(reg) _,
            out = out(reg) ebx,
            inout("eax") 7u32 => _,
            inout("ecx") 0u32 => _,
            out("edx") _,
        );
        (ebx & (1 << 5)) != 0
    };
    ANC_.store(if result { 2 } else { 1 }, Ordering::Relaxed);
    result
}



#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2")]
pub unsafe fn lvh(dst: *mut u32, count: usize, color: u32) {
    if count == 0 { return; }

    let bfl = _mm256_set1_epi32(color as i32);
    let mut ptr = dst;
    let mut ck = count;

    
    while ck >= 32 {
        _mm256_storeu_si256(ptr as *mut __m256i, bfl);
        _mm256_storeu_si256(ptr.add(8) as *mut __m256i, bfl);
        _mm256_storeu_si256(ptr.add(16) as *mut __m256i, bfl);
        _mm256_storeu_si256(ptr.add(24) as *mut __m256i, bfl);
        ptr = ptr.add(32);
        ck -= 32;
    }
    
    while ck >= 8 {
        _mm256_storeu_si256(ptr as *mut __m256i, bfl);
        ptr = ptr.add(8);
        ck -= 8;
    }
    
    for i in 0..ck {
        *ptr.add(i) = color;
    }
}



#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2")]
pub unsafe fn kxt(dst: *mut u32, src: *const u32, count: usize) {
    if count == 0 { return; }

    let mut d = dst;
    let mut j = src;
    let mut ck = count;

    
    while ck >= 32 {
        let a = _mm256_loadu_si256(j as *const __m256i);
        let b = _mm256_loadu_si256(j.add(8) as *const __m256i);
        let c = _mm256_loadu_si256(j.add(16) as *const __m256i);
        let e = _mm256_loadu_si256(j.add(24) as *const __m256i);
        _mm256_storeu_si256(d as *mut __m256i, a);
        _mm256_storeu_si256(d.add(8) as *mut __m256i, b);
        _mm256_storeu_si256(d.add(16) as *mut __m256i, c);
        _mm256_storeu_si256(d.add(24) as *mut __m256i, e);
        d = d.add(32);
        j = j.add(32);
        ck -= 32;
    }
    
    while ck >= 8 {
        _mm256_storeu_si256(d as *mut __m256i, _mm256_loadu_si256(j as *const __m256i));
        d = d.add(8);
        j = j.add(8);
        ck -= 8;
    }
    
    for i in 0..ck {
        *d.add(i) = *j.add(i);
    }
}



#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2")]
pub unsafe fn kce(dst: *mut u32, src: *const u32, count: usize) {
    if count == 0 { return; }

    let zero = _mm256_setzero_si256();
    let cw = _mm256_set1_epi16(128);

    let mut d = dst;
    let mut j = src;
    let mut ck = count;

    
    while ck >= 8 {
        let crb = _mm256_loadu_si256(j as *const __m256i);
        let huc = _mm256_loadu_si256(d as *const __m256i);

        
        let ctn = _mm256_srli_epi32(crb, 24);
        let fgs = _mm256_cmpeq_epi32(ctn, _mm256_set1_epi32(0xFF));
        let dhl = _mm256_cmpeq_epi32(ctn, zero);

        if _mm256_movemask_epi8(fgs) == -1i32 {
            _mm256_storeu_si256(d as *mut __m256i, crb);
        } else if _mm256_movemask_epi8(dhl) != -1i32 {
            
            let jhp = _mm256_unpacklo_epi8(crb, zero);
            let llq = _mm256_unpacklo_epi8(huc, zero);

            
            let hex = _mm256_shufflehi_epi16(
                _mm256_shufflelo_epi16(jhp, 0xFF), 0xFF
            );
            let mrf = _mm256_sub_epi16(_mm256_set1_epi16(255), hex);

            let kci = _mm256_add_epi16(
                _mm256_add_epi16(_mm256_mullo_epi16(jhp, hex), cw),
                _mm256_mullo_epi16(llq, mrf),
            );
            let ogo = _mm256_srli_epi16(kci, 8);

            
            let jho = _mm256_unpackhi_epi8(crb, zero);
            let llo = _mm256_unpackhi_epi8(huc, zero);

            let hew = _mm256_shufflehi_epi16(
                _mm256_shufflelo_epi16(jho, 0xFF), 0xFF
            );
            let mre = _mm256_sub_epi16(_mm256_set1_epi16(255), hew);

            let kch = _mm256_add_epi16(
                _mm256_add_epi16(_mm256_mullo_epi16(jho, hew), cw),
                _mm256_mullo_epi16(llo, mre),
            );
            let ogn = _mm256_srli_epi16(kch, 8);

            _mm256_storeu_si256(d as *mut __m256i, _mm256_packus_epi16(ogo, ogn));
        }
        

        d = d.add(8);
        j = j.add(8);
        ck -= 8;
    }
    
    for i in 0..ck {
        *d.add(i) = fjj(*j.add(i), *d.add(i));
    }
}






#[cfg(target_arch = "x86_64")]
#[inline]
pub unsafe fn qfp(dst: *mut u32, count: usize, color: u32) {
    if has_avx2() {
        lvh(dst, count, color);
    } else {
        adq(dst, count, color);
    }
}


#[cfg(target_arch = "x86_64")]
#[inline]
pub unsafe fn qbi(dst: *mut u32, src: *const u32, count: usize) {
    if has_avx2() {
        kxt(dst, src, count);
    } else {
        blg(dst, src, count);
    }
}


#[cfg(target_arch = "x86_64")]
#[inline]
pub unsafe fn pyt(dst: *mut u32, src: *const u32, count: usize) {
    if has_avx2() {
        kce(dst, src, count);
    } else {
        egy(dst, src, count);
    }
}
