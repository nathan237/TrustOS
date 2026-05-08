





pub mod font;
pub mod logo;

use core::fmt;
use spin::Mutex;
use alloc::vec::Vec;
use alloc::boxed::Box;
use alloc::format;
use core::sync::atomic::{AtomicPtr, AtomicU64, AtomicBool, Ordering};
use crate::math::ra;


struct Awf {
    addr: *mut u8,
    width: u64,
    height: u64,
    pitch: u64,
    bpp: u16,
}


struct Hk {
    cursor_x: usize,
    cursor_y: usize,
    fg_color: u32,
    bg_color: u32,
}


pub static BL_: AtomicPtr<u8> = AtomicPtr::new(core::ptr::null_mut());
pub static X_: AtomicU64 = AtomicU64::new(0);
pub static W_: AtomicU64 = AtomicU64::new(0);
pub static CB_: AtomicU64 = AtomicU64::new(0);
pub static BXY_: AtomicU64 = AtomicU64::new(32); 


static Bi: Mutex<Option<Box<[u32]>>> = Mutex::new(None);
static BFP_: Mutex<Option<Box<[u32]>>> = Mutex::new(None);
static GM_: AtomicBool = AtomicBool::new(false);




static CR_: AtomicPtr<u32> = AtomicPtr::new(core::ptr::null_mut());
static EV_: AtomicU64 = AtomicU64::new(0);
static IT_: AtomicU64 = AtomicU64::new(0);





use core::sync::atomic::AtomicU32;

static TC_: AtomicBool = AtomicBool::new(false);
static ABO_: AtomicU32 = AtomicU32::new(0);
static ABQ_: AtomicU32 = AtomicU32::new(0);
static ABP_: AtomicU32 = AtomicU32::new(u32::MAX);
static ABR_: AtomicU32 = AtomicU32::new(u32::MAX);


pub fn jey(x: u32, y: u32, w: u32, h: u32) {
    ABO_.store(x, Ordering::Relaxed);
    ABQ_.store(y, Ordering::Relaxed);
    ABP_.store(x.saturating_add(w), Ordering::Relaxed);
    ABR_.store(y.saturating_add(h), Ordering::Relaxed);
    TC_.store(true, Ordering::Release);
}


pub fn hlf() {
    TC_.store(false, Ordering::Release);
}


#[inline(always)]
pub fn eic(x: u32, y: u32) -> bool {
    if !TC_.load(Ordering::Relaxed) {
        return true;
    }
    x >= ABO_.load(Ordering::Relaxed) && x < ABP_.load(Ordering::Relaxed) &&
    y >= ABQ_.load(Ordering::Relaxed) && y < ABR_.load(Ordering::Relaxed)
}




pub fn dix() {
    if let Some(ref mut buf) = *Bi.lock() {
        CR_.store(buf.as_mut_ptr(), Ordering::Release);
        EV_.store(X_.load(Ordering::Relaxed), Ordering::Release);
        IT_.store(W_.load(Ordering::Relaxed), Ordering::Release);
    }
}


pub fn civ() {
    CR_.store(core::ptr::null_mut(), Ordering::Release);
}




pub fn odv(ptr: *mut u32) {
    CR_.store(ptr, Ordering::Release);
    EV_.store(X_.load(Ordering::Relaxed), Ordering::Release);
    IT_.store(W_.load(Ordering::Relaxed), Ordering::Release);
}


pub fn ogf() {
    CR_.store(core::ptr::null_mut(), Ordering::Release);
}



pub fn lyj() -> *mut u32 {
    CR_.load(Ordering::Relaxed)
}




#[inline(always)]
pub fn lyk() -> (*mut u32, u32, u32) {
    let ptr = CR_.load(Ordering::Relaxed);
    let stride = EV_.load(Ordering::Relaxed) as u32;
    let height = IT_.load(Ordering::Relaxed) as u32;
    (ptr, stride, height)
}


pub struct FrameCtx {
    pub ptr: *mut u32,
    pub stride: u32,
    pub height: u32,
    pub clip_active: bool,
    pub clip_x1: u32,
    pub clip_y1: u32,
    pub clip_x2: u32,
    pub clip_y2: u32,
}

impl FrameCtx {
    
    #[inline(always)]
    pub fn snapshot() -> Self {
        Self {
            ptr: CR_.load(Ordering::Relaxed),
            stride: EV_.load(Ordering::Relaxed) as u32,
            height: IT_.load(Ordering::Relaxed) as u32,
            clip_active: TC_.load(Ordering::Relaxed),
            clip_x1: ABO_.load(Ordering::Relaxed),
            clip_y1: ABQ_.load(Ordering::Relaxed),
            clip_x2: ABP_.load(Ordering::Relaxed),
            clip_y2: ABR_.load(Ordering::Relaxed),
        }
    }

    
    #[inline(always)]
    pub fn put_pixel(&self, x: u32, y: u32, color: u32) {
        if self.ptr.is_null() { put_pixel(x, y, color); return; }
        if x >= self.stride || y >= self.height { return; }
        if self.clip_active && !(x >= self.clip_x1 && x < self.clip_x2 && y >= self.clip_y1 && y < self.clip_y2) {
            return;
        }
        unsafe { *self.ptr.add(y as usize * self.stride as usize + x as usize) = color; }
    }

    
    #[inline(always)]
    pub fn get_pixel(&self, x: u32, y: u32) -> u32 {
        if self.ptr.is_null() { return get_pixel(x, y); }
        if x >= self.stride || y >= self.height { return 0; }
        unsafe { *self.ptr.add(y as usize * self.stride as usize + x as usize) }
    }
}



#[inline(always)]
pub fn cz(x: u32, y: u32, color: u32) {
    let ptr = CR_.load(Ordering::Relaxed);
    if ptr.is_null() { put_pixel(x, y, color); return; }
    let stride = EV_.load(Ordering::Relaxed) as u32;
    let height = IT_.load(Ordering::Relaxed) as u32;
    if x >= stride || y >= height { return; }
    if !eic(x, y) { return; }
    unsafe { *ptr.add(y as usize * stride as usize + x as usize) = color; }
}


#[inline(always)]
pub fn fyu(x: u32, y: u32) -> u32 {
    let ptr = CR_.load(Ordering::Relaxed);
    if ptr.is_null() { return get_pixel(x, y); }
    let stride = EV_.load(Ordering::Relaxed) as u32;
    let height = IT_.load(Ordering::Relaxed) as u32;
    if x >= stride || y >= height { return 0; }
    unsafe { *ptr.add(y as usize * stride as usize + x as usize) }
}



const AJK_: usize = 1000;  
const QE_: usize = 256; 


#[derive(Clone)]
struct ScrollbackLine {
    chars: [char; QE_],
    colors: [(u32, u32); QE_], 
    len: usize,
}

impl ScrollbackLine {
    const fn new() -> Self {
        ScrollbackLine {
            chars: [' '; QE_],
            colors: [(0xFFFFFFFF, 0xFF000000); QE_],
            len: 0,
        }
    }
}


struct ScrollbackBuffer {
    lines: Vec<ScrollbackLine>,
    current_line: ScrollbackLine,
    scroll_offset: usize,  
    is_scrolled: bool,     
}

impl ScrollbackBuffer {
    fn new() -> Self {
        ScrollbackBuffer {
            lines: Vec::with_capacity(AJK_),
            current_line: ScrollbackLine::new(),
            scroll_offset: 0,
            is_scrolled: false,
        }
    }
    
    
    fn push_char(&mut self, c: char, fg: u32, bg: u32) {
        if c == '\n' {
            
            self.commit_line();
        } else if c == '\r' {
            
            self.current_line.len = 0;
        } else if c == '\x08' {
            
            if self.current_line.len > 0 {
                self.current_line.len -= 1;
            }
        } else if c.is_ascii_graphic() || c == ' ' {
            if self.current_line.len < QE_ {
                self.current_line.chars[self.current_line.len] = c;
                self.current_line.colors[self.current_line.len] = (fg, bg);
                self.current_line.len += 1;
            }
        }
    }
    
    
    fn commit_line(&mut self) {
        if self.lines.len() >= AJK_ {
            self.lines.remove(0); 
        }
        self.lines.push(self.current_line.clone());
        self.current_line = ScrollbackLine::new();
    }
    
    
    fn total_lines(&self) -> usize {
        self.lines.len()
    }
}

static Fy: Mutex<Option<ScrollbackBuffer>> = Mutex::new(None);
static YB_: AtomicBool = AtomicBool::new(false);



static SD_: Mutex<Option<Box<[u32]>>> = Mutex::new(None);
static MS_: AtomicBool = AtomicBool::new(false);






static ZS_: Mutex<Option<Box<[u32]>>> = Mutex::new(None);
static ZT_: AtomicBool = AtomicBool::new(false);






const ANN_: usize = 8;

struct Wr {
    frames: alloc::vec::Vec<Box<[u32]>>,
    write_idx: usize,
    read_idx: usize,
    count: usize,
    frame_size: usize, 
}

static IF_: Mutex<Option<Wr>> = Mutex::new(None);


pub fn moy() {
    let width = X_.load(Ordering::SeqCst) as usize;
    let height = W_.load(Ordering::SeqCst) as usize;
    if width == 0 || height == 0 { return; }
    let frame_size = width * height;
    let mut frames = alloc::vec::Vec::new();
    for i in 0..ANN_ {
        let mut buf = alloc::vec::Vec::new();
        if buf.try_reserve_exact(frame_size).is_err() {
            crate::serial_println!("[FB] BG ring: OOM at slot {} — allocated {}/{}", i, i, ANN_);
            break;
        }
        buf.resize(frame_size, 0u32);
        frames.push(buf.into_boxed_slice());
    }
    let bkr = frames.len();
    if bkr == 0 {
        crate::serial_println!("[FB] BG ring: no slots allocated — disabled");
        return;
    }
    *IF_.lock() = Some(Wr {
        frames,
        write_idx: 0,
        read_idx: 0,
        count: 0,
        frame_size,
    });
    crate::serial_println!("[FB] BG ring: {} slots × {} KB = {} KB total",
        bkr, frame_size * 4 / 1024, bkr * frame_size * 4 / 1024);
}


pub fn pyp() -> usize {
    if let Some(ref dq) = *IF_.lock() {
        dq.count
    } else {
        0
    }
}


pub fn pyq() -> bool {
    if let Some(ref dq) = *IF_.lock() {
        dq.count >= dq.frames.len()
    } else {
        true
    }
}




pub fn pyo() -> bool {
    let mut jg = IF_.lock();
    let dq = match jg.as_mut() {
        Some(r) => r,
        None => return false,
    };
    if dq.count >= dq.frames.len() { return false; }
    let slot = &mut dq.frames[dq.write_idx];
    CR_.store(slot.as_mut_ptr(), Ordering::Release);
    let width = X_.load(Ordering::Relaxed);
    let height = W_.load(Ordering::Relaxed);
    EV_.store(width, Ordering::Release);
    IT_.store(height, Ordering::Release);
    true
}



pub fn qaw() {
    CR_.store(core::ptr::null_mut(), Ordering::Release);
    let mut jg = IF_.lock();
    if let Some(ref mut dq) = *jg {
        dq.write_idx = (dq.write_idx + 1) % dq.frames.len();
        dq.count += 1;
    }
}



pub fn qbe() -> bool {
    let mut jg = IF_.lock();
    let dq = match jg.as_mut() {
        Some(r) if r.count > 0 => r,
        _ => return false,
    };
    let slot = &dq.frames[dq.read_idx];
    
    let ptr = CR_.load(Ordering::Relaxed);
    if !ptr.is_null() {
        let len = dq.frame_size;
        unsafe {
            core::ptr::copy_nonoverlapping(slot.as_ptr(), ptr, len);
        }
    } else {
        
        if let Some(ref mut back_buf) = *Bi.lock() {
            let len = dq.frame_size.min(back_buf.len());
            unsafe {
                core::ptr::copy_nonoverlapping(slot.as_ptr(), back_buf.as_mut_ptr(), len);
            }
        }
    }
    dq.read_idx = (dq.read_idx + 1) % dq.frames.len();
    dq.count -= 1;
    true
}



pub fn qfw() {
    if let Some(ref mut dq) = *IF_.lock() {
        dq.read_idx = 0;
        dq.write_idx = 0;
        dq.count = 0;
    }
}



pub const HH_: usize = 32;

#[derive(Clone, Copy, Default)]
pub struct DirtyRect {
    pub x: u32,
    pub y: u32,
    pub w: u32,
    pub h: u32,
}

impl DirtyRect {
    pub const fn new(x: u32, y: u32, w: u32, h: u32) -> Self {
        DirtyRect { x, y, w, h }
    }
    
    
    pub fn is_valid(&self) -> bool {
        self.w > 0 && self.h > 0
    }
    
    
    pub fn merge(&self, other: &DirtyRect) -> DirtyRect {
        if !self.is_valid() { return *other; }
        if !other.is_valid() { return *self; }
        
        let x1 = self.x.min(other.x);
        let y1 = self.y.min(other.y);
        let x2 = (self.x + self.w).max(other.x + other.w);
        let y2 = (self.y + self.h).max(other.y + other.h);
        
        DirtyRect { x: x1, y: y1, w: x2 - x1, h: y2 - y1 }
    }
    
    
    pub fn overlaps(&self, other: &DirtyRect) -> bool {
        !(self.x + self.w <= other.x || other.x + other.w <= self.x ||
          self.y + self.h <= other.y || other.y + other.h <= self.y)
    }
}

struct DirtyRectList {
    rects: [DirtyRect; HH_],
    count: usize,
    full_redraw: bool,
}

impl DirtyRectList {
    const fn new() -> Self {
        DirtyRectList {
            rects: [DirtyRect { x: 0, y: 0, w: 0, h: 0 }; HH_],
            count: 0,
            full_redraw: true, 
        }
    }
    
    fn add(&mut self, rect: DirtyRect) {
        if self.full_redraw { return; } 
        if !rect.is_valid() { return; }
        
        
        for i in 0..self.count {
            if self.rects[i].overlaps(&rect) {
                self.rects[i] = self.rects[i].merge(&rect);
                return;
            }
        }
        
        
        if self.count < HH_ {
            self.rects[self.count] = rect;
            self.count += 1;
        } else {
            
            self.full_redraw = true;
        }
    }
    
    fn clear(&mut self) {
        self.count = 0;
        self.full_redraw = false;
    }
    
    fn mark_full_redraw(&mut self) {
        self.full_redraw = true;
    }
}

static NR_: Mutex<DirtyRectList> = Mutex::new(DirtyRectList::new());

static Ck: Mutex<Hk> = Mutex::new(Hk {
    cursor_x: 0,
    cursor_y: 0,
    fg_color: 0xFFFFFFFF, 
    bg_color: 0xFF000000, 
});


const CL_: usize = 8;
const BP_: usize = 16;


const BBM_: usize = 32 * 1024 * 1024;



pub fn init(addr: *mut u8, width: u64, height: u64, pitch: u64, bpp: u16) {
    
    if bpp != 32 {
        crate::serial_println!("[FB] WARNING: unsupported bpp={}, forcing 32bpp interpretation (may have color artifacts)", bpp);
    }

    BL_.store(addr, Ordering::SeqCst);
    X_.store(width, Ordering::SeqCst);
    W_.store(height, Ordering::SeqCst);
    CB_.store(pitch, Ordering::SeqCst);
    BXY_.store(bpp as u64, Ordering::SeqCst);
    
    
    
    
    
    clear();
}


pub fn gcq() {
    *Fy.lock() = Some(ScrollbackBuffer::new());
    YB_.store(true, Ordering::SeqCst);
    crate::serial_println!("[FB] Scrollback buffer initialized ({} lines max)", AJK_);
}


pub fn is_initialized() -> bool {
    !BL_.load(Ordering::SeqCst).is_null()
}


pub fn width() -> u32 {
    X_.load(Ordering::SeqCst) as u32
}


pub fn height() -> u32 {
    W_.load(Ordering::SeqCst) as u32
}


pub fn fyq() -> *mut u32 {
    BL_.load(Ordering::SeqCst) as *mut u32
}


pub fn clear() {
    let addr = BL_.load(Ordering::SeqCst);
    if addr.is_null() {
        return;
    }
    
    let height = W_.load(Ordering::SeqCst) as usize;
    let pitch = CB_.load(Ordering::SeqCst) as usize;
    let bg_color = Ck.lock().bg_color;
    
    for y in 0..height {
        let row = unsafe { addr.add(y * pitch) };
        for x in 0..(pitch / 4) {
            unsafe {
                row.add(x * 4).cast::<u32>().write_volatile(bg_color);
            }
        }
    }
    
    let mut console = Ck.lock();
    console.cursor_x = 0;
    console.cursor_y = 0;
}


pub fn bdr(color: u32) {
    Ck.lock().fg_color = color;
}


pub fn qvk(color: u32) {
    Ck.lock().bg_color = color;
}


pub fn put_pixel(x: u32, y: u32, color: u32) {
    let addr = BL_.load(Ordering::SeqCst);
    if addr.is_null() {
        return;
    }
    
    let width = X_.load(Ordering::SeqCst) as u32;
    let height = W_.load(Ordering::SeqCst) as u32;
    let pitch = CB_.load(Ordering::SeqCst) as usize;
    
    if x >= width || y >= height {
        return;
    }
    if !eic(x, y) { return; }
    
    
    if GM_.load(Ordering::SeqCst) {
        if let Some(ref mut buf) = *Bi.lock() {
            let idx = y as usize * width as usize + x as usize;
            if idx < buf.len() {
                buf[idx] = color;
            }
        }
    } else {
        let offset = y as usize * pitch + x as usize * 4;
        unsafe {
            addr.add(offset).cast::<u32>().write_volatile(color);
        }
    }
}


pub fn get_pixel(x: u32, y: u32) -> u32 {
    let addr = BL_.load(Ordering::SeqCst);
    if addr.is_null() {
        return 0;
    }
    
    let width = X_.load(Ordering::SeqCst) as u32;
    let height = W_.load(Ordering::SeqCst) as u32;
    let pitch = CB_.load(Ordering::SeqCst) as usize;
    
    if x >= width || y >= height {
        return 0;
    }
    
    
    if GM_.load(Ordering::SeqCst) {
        if let Some(ref buf) = *Bi.lock() {
            let idx = y as usize * width as usize + x as usize;
            if idx < buf.len() {
                return buf[idx];
            }
        }
        0
    } else {
        let offset = y as usize * pitch + x as usize * 4;
        unsafe {
            addr.add(offset).cast::<u32>().read_volatile()
        }
    }
}







pub struct FastPixelContext {
    pub addr: *mut u8,
    pub width: usize,
    pub height: usize,
    pub pitch: usize,
    pub backbuffer: bool,
}

impl FastPixelContext {
    
    #[inline]
    pub fn new() -> Self {
        FastPixelContext {
            addr: BL_.load(Ordering::SeqCst),
            width: X_.load(Ordering::SeqCst) as usize,
            height: W_.load(Ordering::SeqCst) as usize,
            pitch: CB_.load(Ordering::SeqCst) as usize,
            backbuffer: GM_.load(Ordering::SeqCst),
        }
    }
    
    
    
    #[inline(always)]
    pub unsafe fn put_pixel_unchecked(&self, x: usize, y: usize, color: u32) {
        if self.backbuffer {
            
            if let Some(ref mut buf) = *Bi.lock() {
                let idx = y * self.width + x;
                *buf.get_unchecked_mut(idx) = color;
            }
        } else {
            let offset = y * self.pitch + x * 4;
            (self.addr.add(offset) as *mut u32).write_volatile(color);
        }
    }
    
    
    #[inline(always)]
    pub fn put_pixel(&self, x: usize, y: usize, color: u32) {
        if x >= self.width || y >= self.height { return; }
        unsafe { self.put_pixel_unchecked(x, y, color); }
    }
    
    
    #[inline]
    pub fn qfo(&self, x: usize, y: usize, len: usize, color: u32) {
        if y >= self.height || x >= self.width { return; }
        let cfr = len.min(self.width - x);
        
        if self.backbuffer {
            if let Some(ref mut buf) = *Bi.lock() {
                let start = y * self.width + x;
                #[cfg(target_arch = "x86_64")]
                unsafe {
                    crate::graphics::simd::adq(
                        buf.as_mut_ptr().add(start),
                        cfr,
                        color
                    );
                }
                #[cfg(not(target_arch = "x86_64"))]
                {
                    buf[start..start + cfr].fill(color);
                }
            }
        } else {
            unsafe {
                let ptr = (self.addr.add(y * self.pitch + x * 4)) as *mut u32;
                #[cfg(target_arch = "x86_64")]
                {
                    crate::graphics::simd::adq(ptr, cfr, color);
                }
                #[cfg(not(target_arch = "x86_64"))]
                {
                    for i in 0..cfr {
                        ptr.add(i).write_volatile(color);
                    }
                }
            }
        }
    }
    
    
    
    pub fn qhd(&self) -> Option<alloc::boxed::Box<[u32]>> {
        if self.backbuffer {
            if let Some(ref buf) = *Bi.lock() {
                
                return Some(buf.clone());
            }
        }
        None
    }
}


pub fn kv() -> (u32, u32) {
    (X_.load(Ordering::SeqCst) as u32, W_.load(Ordering::SeqCst) as u32)
}







#[inline]
pub fn rcl<F: FnOnce(*mut u32, usize, usize, usize)>(f: F) -> bool {
    let width = X_.load(Ordering::SeqCst) as usize;
    let height = W_.load(Ordering::SeqCst) as usize;
    if width == 0 || height == 0 { return false; }
    if let Some(ref mut buf) = *Bi.lock() {
        f(buf.as_mut_ptr(), width, height, width);
        true
    } else {
        false
    }
}


pub fn eob() -> *mut u8 {
    BL_.load(Ordering::SeqCst)
}


pub fn fyo() -> usize {
    CB_.load(Ordering::SeqCst) as usize
}




pub fn adw() {
    let width = X_.load(Ordering::SeqCst) as usize;
    let height = W_.load(Ordering::SeqCst) as usize;
    
    if width == 0 || height == 0 {
        return;
    }
    
    let size = width * height;
    let size_bytes = size * 4; 
    
    if size_bytes > BBM_ {
        crate::serial_println!("[FB] WARNING: Framebuffer {}x{} = {} KB too large for backbuffer (max {} KB), disabling double buffer",
            width, height, size_bytes / 1024, BBM_ / 1024);
        return;
    }
    
    
    let mut buffer = alloc::vec::Vec::new();
    if buffer.try_reserve_exact(size).is_err() {
        crate::serial_println!("[FB] WARNING: Failed to allocate backbuffer {} KB — OOM, desktop will use direct mode",
            size_bytes / 1024);
        return;
    }
    buffer.resize(size, 0u32);
    let buffer = buffer.into_boxed_slice();
    
    *Bi.lock() = Some(buffer);
    
    
    let mut goa = alloc::vec::Vec::new();
    if goa.try_reserve_exact(size).is_ok() {
        goa.resize(size, 0u32);
        *BFP_.lock() = Some(goa.into_boxed_slice());
        crate::serial_println!("[FB] Row-diff shadow buffer allocated: {} KB", size_bytes / 1024);
    }
    
    crate::serial_println!("[FB] Double buffer allocated: {}x{} ({} KB)", width, height, size_bytes / 1024);
}


pub fn pr(enabled: bool) {
    GM_.store(enabled, Ordering::SeqCst);
}


pub fn ajy() -> bool {
    GM_.load(Ordering::SeqCst)
}



pub fn aqr() -> Option<(*mut u8, u32, u32, u32)> {
    let width = X_.load(Ordering::SeqCst) as u32;
    let height = W_.load(Ordering::SeqCst) as u32;
    
    if width == 0 || height == 0 {
        return None;
    }
    
    let backbuffer = Bi.lock();
    if let Some(ref buf) = *backbuffer {
        
        let ptr = buf.as_ptr() as *mut u8;
        Some((ptr, width, height, width)) 
    } else {
        None
    }
}







pub fn ii() {
    let addr = BL_.load(Ordering::Relaxed);
    let width = X_.load(Ordering::Relaxed) as usize;
    let height = W_.load(Ordering::Relaxed) as usize;
    let pitch = CB_.load(Ordering::Relaxed) as usize;
    
    if width == 0 || height == 0 { return; }
    
    
    
    
    if crate::drivers::virtio_gpu::sw() {
        
        let mfs = crate::drivers::virtio_gpu::get_back_buffer()
            .or_else(|| crate::drivers::virtio_gpu::eod());
        if let Some((gpu_ptr, gpu_w, gpu_h)) = mfs {
            if let Some(ref buf) = *Bi.lock() {
                let ut = width.min(gpu_w as usize);
                let abw = height.min(gpu_h as usize);
                unsafe {
                    for y in 0..abw {
                        let src = buf.as_ptr().add(y * width);
                        let dst = gpu_ptr.add(y * gpu_w as usize);
                        #[cfg(target_arch = "x86_64")]
                        crate::graphics::simd::blg(dst, src, ut);
                        #[cfg(not(target_arch = "x86_64"))]
                        core::ptr::copy_nonoverlapping(src, dst, ut);
                    }
                }
            }
            
            let _ = crate::drivers::virtio_gpu::nww();
            
            
            return;
        }
    }
    
    
    if addr.is_null() { return; }
    
    oyz(addr, width, height, pitch);
}




fn oyz(addr: *mut u8, width: usize, height: usize, pitch: usize) {
    let kap = Bi.lock();
    let mut nts = BFP_.lock();
    
    let (mq, ccq) = match (kap.as_ref(), nts.as_mut()) {
        (Some(b), Some(aa)) => (b, aa),
        
        (Some(b), None) => {
            for y in 0..height {
                let azu = y * width;
                let afd = y * pitch;
                unsafe {
                    let src = b.as_ptr().add(azu);
                    let dst = addr.add(afd) as *mut u32;
                    #[cfg(target_arch = "x86_64")]
                    crate::graphics::simd::eiy(dst, src, width);
                    #[cfg(not(target_arch = "x86_64"))]
                    core::ptr::copy_nonoverlapping(src, dst, width);
                }
            }
            return;
        }
        _ => return,
    };
    
    for y in 0..height {
        let offset = y * width;
        let egp = &mq[offset..offset + width];
        let gmu = &mut ccq[offset..offset + width];
        
        
        let mut changed = false;
        let bqm = egp.as_ptr() as *const u64;
        let buq = gmu.as_ptr() as *const u64;
        let ccd = width / 2;
        
        
        let chunks = ccd / 8;
        let mut i = 0usize;
        unsafe {
            for _ in 0..chunks {
                if *bqm.add(i) != *buq.add(i)
                    || *bqm.add(i+1) != *buq.add(i+1)
                    || *bqm.add(i+2) != *buq.add(i+2)
                    || *bqm.add(i+3) != *buq.add(i+3)
                    || *bqm.add(i+4) != *buq.add(i+4)
                    || *bqm.add(i+5) != *buq.add(i+5)
                    || *bqm.add(i+6) != *buq.add(i+6)
                    || *bqm.add(i+7) != *buq.add(i+7)
                {
                    changed = true;
                    break;
                }
                i += 8;
            }
            
            if !changed {
                while i < ccd {
                    if *bqm.add(i) != *buq.add(i) {
                        changed = true;
                        break;
                    }
                    i += 1;
                }
            }
            
            if !changed && (width & 1) != 0 {
                if egp[width - 1] != gmu[width - 1] {
                    changed = true;
                }
            }
        }
        
        if changed {
            
            let afd = y * pitch;
            unsafe {
                let src = egp.as_ptr();
                let dst = addr.add(afd) as *mut u32;
                #[cfg(target_arch = "x86_64")]
                crate::graphics::simd::eiy(dst, src, width);
                #[cfg(not(target_arch = "x86_64"))]
                core::ptr::copy_nonoverlapping(src, dst, width);
            }
            
            gmu.copy_from_slice(egp);
        }
    }
}




fn oyy(addr: *mut u8, width: usize, height: usize, pitch: usize) {
    if let Some(ref buf) = *Bi.lock() {
        for y in 0..height {
            let azu = y * width;
            let afd = y * pitch;
            unsafe {
                let src = buf.as_ptr().add(azu);
                let dst = addr.add(afd) as *mut u32;
                #[cfg(target_arch = "x86_64")]
                crate::graphics::simd::eiy(dst, src, width);
                #[cfg(not(target_arch = "x86_64"))]
                core::ptr::copy_nonoverlapping(src, dst, width);
            }
        }
    }
}



pub fn ibg() -> Option<*const u32> {
    let mq = Bi.lock();
    mq.as_ref().map(|buf| buf.as_ptr())
}



pub fn oza() {
    let addr = BL_.load(Ordering::Relaxed);
    if addr.is_null() { return; }
    let width = X_.load(Ordering::Relaxed) as usize;
    let height = W_.load(Ordering::Relaxed) as usize;
    let pitch = CB_.load(Ordering::Relaxed) as usize;
    if width == 0 || height == 0 { return; }
    oyy(addr, width, height, pitch);
}


pub fn awo(color: u32) {
    if let Some(ref mut buf) = *Bi.lock() {
        #[cfg(target_arch = "x86_64")]
        unsafe {
            crate::graphics::simd::adq(buf.as_mut_ptr(), buf.len(), color);
        }
        #[cfg(not(target_arch = "x86_64"))]
        {
            buf.fill(color);
        }
    }
}






#[repr(C)]
struct Ki {
    src: *const u32,
    dst: *mut u8,
    src_stride: usize,   
    dst_pitch: usize,    
    width: usize,        
}

unsafe impl Send for Ki {}
unsafe impl Sync for Ki {}


fn hid(start: usize, end: usize, data: *mut u8) {
    let ab = unsafe { &*(data as *const Ki) };
    for y in start..end {
        unsafe {
            let src = ab.src.add(y * ab.src_stride);
            let dst = ab.dst.add(y * ab.dst_pitch) as *mut u32;
            #[cfg(target_arch = "x86_64")]
            {
                crate::graphics::simd::eiy(dst, src, ab.width);
            }
            #[cfg(not(target_arch = "x86_64"))]
            {
                core::ptr::copy_nonoverlapping(src, dst, ab.width);
            }
        }
    }
}









pub fn fjl(src: *const u32, w: usize, h: usize) {
    let addr = BL_.load(Ordering::SeqCst);
    if addr.is_null() { return; }

    let fb_w = X_.load(Ordering::SeqCst) as usize;
    let pitch = CB_.load(Ordering::SeqCst) as usize;
    let fb_h = W_.load(Ordering::SeqCst) as usize;

    let ut = w.min(fb_w);
    let abw = h.min(fb_h);

    let ab = Ki {
        src,
        dst: addr,
        src_stride: w,
        dst_pitch: pitch,
        width: ut,
    };

    
    crate::cpu::smp::bcz(
        abw,
        hid,
        &ab as *const Ki as *mut u8,
    );

    
    
    
    hid(0, abw, &ab as *const Ki as *mut u8);
}


pub fn fill_rect(x: u32, y: u32, w: u32, h: u32, color: u32) {
    let width = X_.load(Ordering::SeqCst) as u32;
    let height = W_.load(Ordering::SeqCst) as u32;
    
    let x1 = x.min(width);
    let y1 = y.min(height);
    let x2 = (x + w).min(width);
    let y2 = (y + h).min(height);
    
    if x2 <= x1 || y2 <= y1 { return; }
    
    
    let ptr = CR_.load(Ordering::Relaxed);
    if !ptr.is_null() {
        let stride = EV_.load(Ordering::Relaxed) as usize;
        let ddi = (x2 - x1) as usize;
        for o in y1..y2 {
            let fk = o as usize * stride + x1 as usize;
            #[cfg(target_arch = "x86_64")]
            unsafe {
                crate::graphics::simd::adq(
                    ptr.add(fk),
                    ddi,
                    color
                );
            }
            #[cfg(not(target_arch = "x86_64"))]
            unsafe {
                for i in 0..ddi {
                    *ptr.add(fk + i) = color;
                }
            }
        }
        return;
    }
    
    if GM_.load(Ordering::SeqCst) {
        if let Some(ref mut buf) = *Bi.lock() {
            let ddi = (x2 - x1) as usize;
            for o in y1..y2 {
                let fk = o as usize * width as usize + x1 as usize;
                if fk + ddi <= buf.len() {
                    
                    #[cfg(target_arch = "x86_64")]
                    unsafe {
                        crate::graphics::simd::adq(
                            buf.as_mut_ptr().add(fk),
                            ddi,
                            color
                        );
                    }
                    #[cfg(not(target_arch = "x86_64"))]
                    {
                        buf[fk..fk + ddi].fill(color);
                    }
                }
            }
        }
    } else {
        for o in y1..y2 {
            for p in x1..x2 {
                put_pixel(p, o, color);
            }
        }
    }
}


pub fn draw_pixel(x: u32, y: u32, color: u32) {
    let width = X_.load(Ordering::SeqCst) as u32;
    let height = W_.load(Ordering::SeqCst) as u32;
    
    if x >= width || y >= height { return; }
    
    
    if GM_.load(Ordering::SeqCst) {
        if let Some(ref mut buf) = *Bi.lock() {
            let offset = y as usize * width as usize + x as usize;
            if offset < buf.len() {
                buf[offset] = color;
            }
        }
    } else {
        let addr = BL_.load(Ordering::SeqCst);
        if addr.is_null() { return; }
        let pitch = CB_.load(Ordering::SeqCst) as usize;
        let offset = y as usize * pitch + x as usize * 4;
        unsafe {
            let ptr = addr.add(offset) as *mut u32;
            *ptr = color;
        }
    }
}


pub fn mn(x: u32, y: u32, len: u32, color: u32) {
    fill_rect(x, y, len, 1, color);
}



pub fn co(x: u32, y: u32, w: u32, h: u32, color: u32, alpha: u32) {
    let width = X_.load(Ordering::SeqCst) as u32;
    let height = W_.load(Ordering::SeqCst) as u32;
    
    let x1 = x.min(width);
    let y1 = y.min(height);
    let x2 = (x + w).min(width);
    let y2 = (y + h).min(height);
    if x2 <= x1 || y2 <= y1 { return; }
    
    let alpha = alpha.min(255);
    let ki = 255 - alpha;
    let pb = (color >> 16) & 0xFF;
    let akl = (color >> 8) & 0xFF;
    let cv = color & 0xFF;
    
    
    let ptr = CR_.load(Ordering::Relaxed);
    if !ptr.is_null() {
        let stride = EV_.load(Ordering::Relaxed) as usize;
        for o in y1..y2 {
            let grz = unsafe { ptr.add(o as usize * stride + x1 as usize) };
            let ezb = (x2 - x1) as usize;
            #[cfg(target_arch = "x86_64")]
            unsafe { crate::graphics::simd::hib(grz, ezb, color, alpha); }
            #[cfg(not(target_arch = "x86_64"))]
            for p in x1..x2 {
                let idx = o as usize * stride + p as usize;
                unsafe {
                    let ku = *ptr.add(idx);
                    let qw = (ku >> 16) & 0xFF;
                    let afb = (ku >> 8) & 0xFF;
                    let fu = ku & 0xFF;
                    let r = ((pb * alpha + qw * ki + 128) >> 8).min(255);
                    let g = ((akl * alpha + afb * ki + 128) >> 8).min(255);
                    let b = ((cv * alpha + fu * ki + 128) >> 8).min(255);
                    *ptr.add(idx) = 0xFF000000 | (r << 16) | (g << 8) | b;
                }
            }
        }
        return;
    }
    
    
    if GM_.load(Ordering::SeqCst) {
        if let Some(ref mut buf) = *Bi.lock() {
            let buf_len = buf.len();
            for o in y1..y2 {
                let row = o as usize * width as usize;
                let fk = row + x1 as usize;
                let ezb = (x2 - x1) as usize;
                if fk + ezb <= buf_len {
                    #[cfg(target_arch = "x86_64")]
                    unsafe { crate::graphics::simd::hib(buf.as_mut_ptr().add(fk), ezb, color, alpha); }
                    #[cfg(not(target_arch = "x86_64"))]
                    for p in x1..x2 {
                        let idx = row + p as usize;
                        let ku = buf[idx];
                        let qw = (ku >> 16) & 0xFF;
                        let afb = (ku >> 8) & 0xFF;
                        let fu = ku & 0xFF;
                        let r = ((pb * alpha + qw * ki + 128) >> 8).min(255);
                        let g = ((akl * alpha + afb * ki + 128) >> 8).min(255);
                        let b = ((cv * alpha + fu * ki + 128) >> 8).min(255);
                        buf[idx] = 0xFF000000 | (r << 16) | (g << 8) | b;
                    }
                }
            }
        }
    } else {
        for o in y1..y2 {
            for p in x1..x2 {
                let ku = get_pixel(p, o);
                let qw = (ku >> 16) & 0xFF;
                let afb = (ku >> 8) & 0xFF;
                let fu = ku & 0xFF;
                let r = ((pb * alpha + qw * ki + 128) >> 8).min(255);
                let g = ((akl * alpha + afb * ki + 128) >> 8).min(255);
                let b = ((cv * alpha + fu * ki + 128) >> 8).min(255);
                put_pixel(p, o, 0xFF000000 | (r << 16) | (g << 8) | b);
            }
        }
    }
}


pub fn zv(x: u32, y: u32, len: u32, color: u32) {
    fill_rect(x, y, 1, len, color);
}


pub fn draw_rect(x: u32, y: u32, w: u32, h: u32, color: u32) {
    mn(x, y, w, color);
    mn(x, y + h - 1, w, color);
    zv(x, y, h, color);
    zv(x + w - 1, y, h, color);
}


pub fn fill_circle(cx: u32, u: u32, radius: u32, color: u32) {
    if radius == 0 { return; }
    
    
    let ju = (radius * radius) as i32;
    for ad in 0..=radius {
        let dx = ra((ju - (ad * ad) as i32) as f32) as u32;
        if dx > 0 {
            
            if u >= ad {
                fill_rect(cx.saturating_sub(dx), u - ad, dx * 2 + 1, 1, color);
            }
            
            fill_rect(cx.saturating_sub(dx), u + ad, dx * 2 + 1, 1, color);
        }
    }
}


pub fn fill_rounded_rect(x: u32, y: u32, w: u32, h: u32, radius: u32, color: u32) {
    if w == 0 || h == 0 { return; }
    
    let r = radius.min(w / 2).min(h / 2);
    
    if r == 0 {
        fill_rect(x, y, w, h, color);
        return;
    }
    
    
    fill_rect(x, y + r, w, h - r * 2, color);
    
    fill_rect(x + r, y, w - r * 2, r, color);
    
    fill_rect(x + r, y + h - r, w - r * 2, r, color);
    
    
    let ju = (r * r) as i32;
    for ad in 0..r {
        let dx = ra((ju - (ad * ad) as i32) as f32) as u32;
        if dx > 0 {
            
            fill_rect(x + r - dx, y + r - ad - 1, dx, 1, color);
            
            fill_rect(x + w - r, y + r - ad - 1, dx, 1, color);
            
            fill_rect(x + r - dx, y + h - r + ad, dx, 1, color);
            
            fill_rect(x + w - r, y + h - r + ad, dx, 1, color);
        }
    }
}


pub fn stroke_rounded_rect(x: u32, y: u32, w: u32, h: u32, radius: u32, color: u32) {
    if w == 0 || h == 0 { return; }
    
    let r = radius.min(w / 2).min(h / 2);
    
    if r == 0 {
        draw_rect(x, y, w, h, color);
        return;
    }
    
    
    mn(x + r, y, w - r * 2, color);           
    mn(x + r, y + h - 1, w - r * 2, color);   
    
    zv(x, y + r, h - r * 2, color);           
    zv(x + w - 1, y + r, h - r * 2, color);   
    
    
    let mut p = r as i32;
    let mut o = 0i32;
    let mut err = 0i32;
    
    while p >= o {
        
        draw_pixel(x + r - p as u32, y + r - o as u32, color);
        draw_pixel(x + r - o as u32, y + r - p as u32, color);
        
        draw_pixel(x + w - 1 - r + p as u32, y + r - o as u32, color);
        draw_pixel(x + w - 1 - r + o as u32, y + r - p as u32, color);
        
        draw_pixel(x + r - p as u32, y + h - 1 - r + o as u32, color);
        draw_pixel(x + r - o as u32, y + h - 1 - r + p as u32, color);
        
        draw_pixel(x + w - 1 - r + p as u32, y + h - 1 - r + o as u32, color);
        draw_pixel(x + w - 1 - r + o as u32, y + h - 1 - r + p as u32, color);
        
        o += 1;
        err += 1 + 2 * o;
        if 2 * (err - p) + 1 > 0 {
            p -= 1;
            err += 1 - 2 * p;
        }
    }
}


pub fn draw_text(text: &str, x: u32, y: u32, color: u32) {
    let mut cx = x;
    for c in text.chars() {
        px(cx, y, c, color);
        cx += CL_ as u32;
    }
}


fn draw_char(c: char, x: usize, y: usize, fg: u32, bg: u32) {
    let du = font::ol(c);
    
    for row in 0..BP_ {
        let bits = du[row];
        for col in 0..CL_ {
            let color = if (bits >> (7 - col)) & 1 == 1 { fg } else { bg };
            put_pixel((x + col) as u32, (y + row) as u32, color);
        }
    }
}



pub fn px(x: u32, y: u32, c: char, color: u32) {
    let du = font::ol(c);
    let width = X_.load(Ordering::SeqCst) as u32;
    let height = W_.load(Ordering::SeqCst) as u32;
    
    if x >= width || y >= height { return; }
    
    
    if GM_.load(Ordering::SeqCst) {
        if let Some(ref mut buf) = *Bi.lock() {
            let stride = width as usize;
            for row in 0..BP_ {
                let o = y as usize + row;
                if o >= height as usize { break; }
                let bits = du[row];
                let pq = o * stride;
                for col in 0..CL_ {
                    if (bits >> (7 - col)) & 1 == 1 {
                        let p = x as usize + col;
                        if p < width as usize && eic(p as u32, o as u32) {
                            let offset = pq + p;
                            if offset < buf.len() {
                                buf[offset] = color;
                            }
                        }
                    }
                }
            }
        }
    } else {
        
        let addr = BL_.load(Ordering::SeqCst);
        if addr.is_null() { return; }
        let pitch = CB_.load(Ordering::SeqCst) as usize;
        for row in 0..BP_ {
            let o = y as usize + row;
            if o >= height as usize { break; }
            let bits = du[row];
            for col in 0..CL_ {
                if (bits >> (7 - col)) & 1 == 1 {
                    let p = x as usize + col;
                    if p < width as usize && eic(p as u32, o as u32) {
                        let offset = o * pitch + p * 4;
                        unsafe {
                            let ptr = addr.add(offset) as *mut u32;
                            *ptr = color;
                        }
                    }
                }
            }
        }
    }
}


fn write_char(c: char) {
    let width = X_.load(Ordering::SeqCst) as usize;
    let height = W_.load(Ordering::SeqCst) as usize;
    
    if width == 0 || height == 0 {
        return;
    }
    
    let cols = width / CL_;
    let rows = height / BP_;
    
    let mut console = Ck.lock();
    let fg = console.fg_color;
    let bg = console.bg_color;
    
    
    if YB_.load(Ordering::SeqCst) {
        if let Some(ref mut scrollback) = *Fy.lock() {
            if !scrollback.is_scrolled {
                scrollback.push_char(c, fg, bg);
            }
        }
    }
    
    match c {
        '\x08' => {
            if console.cursor_x > 0 {
                console.cursor_x -= 1;
            } else if console.cursor_y > 0 {
                console.cursor_y -= 1;
                console.cursor_x = cols.saturating_sub(1);
            } else {
                return;
            }

            let p = console.cursor_x * CL_;
            let o = console.cursor_y * BP_;
            let fg = console.fg_color;
            let bg = console.bg_color;
            drop(console);
            draw_char(' ', p, o, fg, bg);
            return;
        }
        '\n' => {
            console.cursor_x = 0;
            console.cursor_y += 1;
        }
        '\r' => {
            console.cursor_x = 0;
        }
        '\t' => {
            console.cursor_x = (console.cursor_x + 4) & !3;
        }
        _ => {
            if console.cursor_x >= cols {
                console.cursor_x = 0;
                console.cursor_y += 1;
            }
            
            let p = console.cursor_x * CL_;
            let o = console.cursor_y * BP_;
            
            
            let x = console.cursor_x;
            console.cursor_x += 1;
            drop(console);
            
            draw_char(c, p, o, fg, bg);
            return;
        }
    }
    
    
    if console.cursor_y >= rows {
        drop(console);
        scroll_up();
        Ck.lock().cursor_y = rows - 1;
    }
}


pub fn scroll_up() {
    let addr = BL_.load(Ordering::SeqCst);
    if addr.is_null() {
        return;
    }
    
    let height = W_.load(Ordering::SeqCst) as usize;
    let pitch = CB_.load(Ordering::SeqCst) as usize;
    let bg_color = Ck.lock().bg_color;
    
    
    for y in BP_..height {
        unsafe {
            let src = addr.add(y * pitch);
            let dst = addr.add((y - BP_) * pitch);
            core::ptr::copy(src, dst, pitch);
        }
    }
    
    
    for y in (height - BP_)..height {
        let row = unsafe { addr.add(y * pitch) };
        for x in 0..(pitch / 4) {
            unsafe {
                row.add(x * 4).cast::<u32>().write_volatile(bg_color);
            }
        }
    }
}


pub struct Vz;

impl fmt::Write for Vz {
    fn write_str(&mut self, j: &str) -> fmt::Result {
        for c in j.chars() {
            write_char(c);
        }
        Ok(())
    }
}



static BHN_: core::sync::atomic::AtomicBool =
    core::sync::atomic::AtomicBool::new(false);


pub fn jfk(on: bool) {
    BHN_.store(on, core::sync::atomic::Ordering::Relaxed);
}


#[doc(hidden)]
pub fn bxg(args: fmt::Arguments) {
    use core::fmt::Write;
    
    if crate::shell::btp() {
        let mut j = alloc::string::String::new();
        let _ = core::fmt::write(&mut j, args);
        crate::shell::khm(&j);
        return;
    }
    if !BHN_.load(core::sync::atomic::Ordering::Relaxed) {
        Vz.write_fmt(args).unwrap();
    }
    crate::serial::bxg(args);
}



#[doc(hidden)]
pub fn jsr(args: fmt::Arguments) {
    use core::fmt::Write;
    Vz.write_fmt(args).unwrap();
}


#[macro_export]
macro_rules! print {
    ($($db:tt)*) => {
        $crate::framebuffer::bxg(format_args!($($db)*))
    };
}


#[macro_export]
macro_rules! aru {
    ($($db:tt)*) => {
        $crate::framebuffer::jsr(format_args!($($db)*))
    };
}


#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($fmt:expr) => ($crate::print!(concat!($fmt, "\n")));
    ($fmt:expr, $($db:tt)*) => ($crate::print!(concat!($fmt, "\n"), $($db)*));
}


pub const NE_: u32 = 0xFF000000;
pub const R_: u32 = 0xFFFFFFFF;
pub const B_: u32 = 0xFF00FF00;
pub const G_: u32 = 0xFF00FF66;
pub const AX_: u32 = 0xFF00AA00;
pub const A_: u32 = 0xFFFF0000;
pub const CF_: u32 = 0xFF0000FF;
pub const D_: u32 = 0xFFFFFF00;
pub const C_: u32 = 0xFF00FFFF;
pub const DM_: u32 = 0xFFFF00FF;
pub const K_: u32 = 0xFF888888;


#[macro_export]
macro_rules! bq {
    ($color:expr, $($db:tt)*) => {{
        let qb = $crate::framebuffer::dqp();
        $crate::framebuffer::bdr($color);
        $crate::print!($($db)*);
        $crate::framebuffer::bdr(qb);
    }};
}


#[macro_export]
macro_rules! n {
    ($color:expr, $($db:tt)*) => {{
        let qb = $crate::framebuffer::dqp();
        $crate::framebuffer::bdr($color);
        $crate::println!($($db)*);
        $crate::framebuffer::bdr(qb);
    }};
}


pub fn dqp() -> u32 {
    Ck.lock().fg_color
}


pub fn guh() {
    let mut console = Ck.lock();
    console.fg_color = B_;
    console.bg_color = NE_;
}


pub fn qvn() {
    let mut console = Ck.lock();
    console.fg_color = R_;
    console.bg_color = NE_;
}




pub fn draw_text_at(text: &str, x: u32, y: u32, fg: u32, bg: u32) {
    for (i, c) in text.chars().enumerate() {
        let p = x + (i as u32) * CL_ as u32;
        draw_char(c, p as usize, y as usize, fg, bg);
    }
}


pub fn draw_text_centered(text: &str, y: u32, fg: u32) {
    let (width, _) = kv();
    let ebn = text.len() as u32 * CL_ as u32;
    let x = (width.saturating_sub(ebn)) / 2;
    draw_text_at(text, x, y, fg, NE_);
}


pub fn ftb(y: u32, color: u32) {
    let (width, _) = kv();
    let oq = width / 10;
    mn(oq, y, width - 2 * oq, color);
}



pub fn hle(row: usize) {
    let width = X_.load(Ordering::SeqCst) as usize;
    let height = W_.load(Ordering::SeqCst) as usize;
    if width == 0 || height == 0 { return; }
    let o = row * BP_;
    if o + BP_ > height { return; }
    let bg = Ck.lock().bg_color;
    fill_rect(0, o as u32, width as u32, BP_ as u32, bg);
}



pub fn ftd(col: usize, row: usize, text: &str, fg: u32, bg: u32) -> usize {
    let width = X_.load(Ordering::SeqCst) as usize;
    let height = W_.load(Ordering::SeqCst) as usize;
    if width == 0 || height == 0 { return 0; }
    let cols = width / CL_;
    let o = row * BP_;
    if o + BP_ > height { return 0; }
    let mut count = 0;
    for (i, ch) in text.chars().enumerate() {
        let c = col + i;
        if c >= cols { break; }
        draw_char(ch, c * CL_, o, fg, bg);
        count += 1;
    }
    count
}


pub fn afr(col: usize, row: usize) {
    let mut console = Ck.lock();
    console.cursor_x = col;
    console.cursor_y = row;
}


pub fn cyk() -> (usize, usize) {
    let console = Ck.lock();
    (console.cursor_x, console.cursor_y)
}


pub fn bly(x: u32, y: u32, width: u32, progress: u32, fg: u32, bg: u32) {
    let oz = (width * progress.min(100)) / 100;
    
    
    fill_rect(x, y, width, 16, bg);
    
    
    if oz > 0 {
        fill_rect(x, y, oz, 16, fg);
    }
    
    
    draw_rect(x, y, width, 16, fg);
}


pub fn hm(bk: &str, status: BootStatus) {
    let (bvz, color) = match status {
        BootStatus::Ok => ("[OK]", B_),
        BootStatus::Skip => ("[--]", K_),
        BootStatus::Fail => ("[!!]", A_),
        BootStatus::Info => ("[..]", C_),
    };
    
    
    let gko = dqp();
    bdr(color);
    crate::print!("{} ", bvz);
    bdr(gko);
    crate::println!("{}", bk);
}


#[derive(Clone, Copy)]
pub enum BootStatus {
    Ok,
    Skip,
    Fail,
    Info,
}


pub fn hti() {
    logo::hti();
}


pub fn gcn() {
    logo::gcn();
}


pub fn afw(phase: u32, message: &str) {
    logo::afw(phase, message);
}


pub fn fvz() {
    logo::fvz();
}


pub fn qww() {
    clear();
    guh();
    
    let (width, _height) = kv();
    
    
    ftb(0, B_);
    
    
    let asz = 16u32;
    draw_text_centered("╔════════════════════════════════════════════════════════════╗", asz, G_);
    draw_text_centered("║                                                            ║", asz + 16, G_);
    draw_text_centered("║   ████████╗██████╗ ██╗   ██╗███████╗████████╗ ██████╗ ███████╗  ║", asz + 32, B_);
    draw_text_centered("║   ╚══██╔══╝██╔══██╗██║   ██║██╔════╝╚══██╔══╝██╔═══██╗██╔════╝  ║", asz + 48, B_);
    draw_text_centered("║      ██║   ██████╔╝██║   ██║███████╗   ██║   ██║   ██║███████╗  ║", asz + 64, G_);
    draw_text_centered("║      ██║   ██╔══██╗██║   ██║╚════██║   ██║   ██║   ██║╚════██║  ║", asz + 80, B_);
    draw_text_centered("║      ██║   ██║  ██║╚██████╔╝███████║   ██║   ╚██████╔╝███████║  ║", asz + 96, G_);
    draw_text_centered("║      ╚═╝   ╚═╝  ╚═╝ ╚═════╝ ╚══════╝   ╚═╝    ╚═════╝ ╚══════╝  ║", asz + 112, B_);
    draw_text_centered("║                                                            ║", asz + 128, G_);
    draw_text_centered("║            FAST  •  SECURE  •  RELIABLE                    ║", asz + 144, AX_);
    draw_text_centered("║                                                            ║", asz + 160, G_);
    draw_text_centered("╚════════════════════════════════════════════════════════════╝", asz + 176, G_);
    
    
    ftb(asz + 200, B_);
    
    
    let bpd = ((asz + 220) / 16) as usize;
    afr(0, bpd);
}


pub fn qwy() {
    clear();
    guh();
    
    crate::println!("╔══════════════════════════════════════════════════════════════╗");
    crate::println!("║                      T R U S T - O S                         ║");
    crate::println!("║                 FAST • SECURE • RELIABLE                     ║");
    crate::println!("╚══════════════════════════════════════════════════════════════╝");
    crate::println!();
}




pub fn mox() {
    let width = X_.load(Ordering::SeqCst) as usize;
    let height = W_.load(Ordering::SeqCst) as usize;
    
    if width == 0 || height == 0 {
        return;
    }
    
    let size = width * height;
    
    let mut buffer = alloc::vec::Vec::new();
    if buffer.try_reserve_exact(size).is_err() {
        crate::serial_println!("[FB] WARNING: Failed to allocate background cache {} KB — OOM",
            size * 4 / 1024);
        return;
    }
    buffer.resize(size, 0u32);
    let buffer = buffer.into_boxed_slice();
    
    *SD_.lock() = Some(buffer);
    MS_.store(false, Ordering::SeqCst);
    crate::serial_println!("[FB] Background cache allocated: {} KB", size * 4 / 1024);
}


pub fn rbv() {
    MS_.store(true, Ordering::SeqCst);
}


pub fn ihi() {
    MS_.store(false, Ordering::SeqCst);
}


pub fn qmb() -> bool {
    MS_.load(Ordering::SeqCst)
}


pub fn quh() {
    let width = X_.load(Ordering::SeqCst) as usize;
    
    let cua = SD_.lock();
    if let Some(ref bg_buf) = *cua {
        if let Some(ref mut back_buf) = *Bi.lock() {
            
            let len = bg_buf.len().min(back_buf.len());
            unsafe {
                core::ptr::copy_nonoverlapping(bg_buf.as_ptr(), back_buf.as_mut_ptr(), len);
            }
        }
    }
}


pub fn qug(x: u32, y: u32, w: u32, h: u32) {
    let width = X_.load(Ordering::SeqCst) as u32;
    let height = W_.load(Ordering::SeqCst) as u32;
    
    let x1 = x.min(width);
    let y1 = y.min(height);
    let x2 = (x + w).min(width);
    let y2 = (y + h).min(height);
    
    let cua = SD_.lock();
    if let Some(ref bg_buf) = *cua {
        if let Some(ref mut back_buf) = *Bi.lock() {
            for o in y1..y2 {
                let zl = o as usize * width as usize + x1 as usize;
                let jhm = o as usize * width as usize + x2 as usize;
                let alj = zl;
                
                if jhm <= bg_buf.len() && jhm <= back_buf.len() {
                    unsafe {
                        let src = bg_buf.as_ptr().add(zl);
                        let dst = back_buf.as_mut_ptr().add(alj);
                        core::ptr::copy_nonoverlapping(src, dst, (x2 - x1) as usize);
                    }
                }
            }
        }
    }
}


pub fn qec<F: FnOnce()>(draw_fn: F) {
    
    draw_fn();
    
    
    kgt();
}


pub fn kgt() {
    
    let width = X_.load(Ordering::SeqCst) as usize;
    let height = W_.load(Ordering::SeqCst) as usize;
    let size = width * height;
    
    if size == 0 { return; }
    
    
    
    let mut cua = SD_.lock();
    let cga = Bi.lock();
    
    if let (Some(ref mut bg_buf), Some(ref back_buf)) = (&mut *cua, &*cga) {
        let len = back_buf.len().min(bg_buf.len());
        unsafe {
            core::ptr::copy_nonoverlapping(back_buf.as_ptr(), bg_buf.as_mut_ptr(), len);
        }
    }
    
    drop(cga);
    drop(cua);
    
    MS_.store(true, Ordering::SeqCst);
}




pub fn mpr() {
    let width = X_.load(Ordering::SeqCst) as usize;
    let height = W_.load(Ordering::SeqCst) as usize;
    if width == 0 || height == 0 { return; }
    let size = width * height;
    let mut buffer = alloc::vec::Vec::new();
    if buffer.try_reserve_exact(size).is_err() {
        crate::serial_println!("[FB] WARNING: Failed to allocate window overlay {} KB — OOM",
            size * 4 / 1024);
        return;
    }
    buffer.resize(size, 0u32);
    *ZS_.lock() = Some(buffer.into_boxed_slice());
    ZT_.store(false, Ordering::SeqCst);
    crate::serial_println!("[FB] Window overlay cache allocated: {} KB", size * 4 / 1024);
}


pub fn kgu() {
    let width = X_.load(Ordering::SeqCst) as usize;
    if width == 0 { return; }
    let mut dcd = ZS_.lock();
    let cga = Bi.lock();
    if let (Some(ref mut ov_buf), Some(ref back_buf)) = (&mut *dcd, &*cga) {
        let len = back_buf.len().min(ov_buf.len());
        unsafe {
            core::ptr::copy_nonoverlapping(back_buf.as_ptr(), ov_buf.as_mut_ptr(), len);
        }
    }
    drop(cga);
    drop(dcd);
    ZT_.store(true, Ordering::SeqCst);
}


pub fn mud() -> bool {
    ZT_.load(Ordering::SeqCst)
}


pub fn eqy() {
    ZT_.store(false, Ordering::SeqCst);
}




pub fn kco(x: i32, y: i32, w: u32, h: u32, shadow_margin: u32) {
    let ddw = X_.load(Ordering::SeqCst) as u32;
    let ezo = W_.load(Ordering::SeqCst) as u32;
    if ddw == 0 || ezo == 0 { return; }

    
    let da = (x - shadow_margin as i32).max(0) as u32;
    let cm = (y - shadow_margin as i32).max(0) as u32;
    let bja = ((x + w as i32 + shadow_margin as i32) as u32).min(ddw);
    let apa = ((y + h as i32 + shadow_margin as i32) as u32).min(ezo);
    if bja <= da || apa <= cm { return; }

    
    let ptr = CR_.load(Ordering::Relaxed);
    if !ptr.is_null() {
        let stride = EV_.load(Ordering::Relaxed) as usize;
        let dcd = ZS_.lock();
        if let Some(ref ov_buf) = *dcd {
            let aoy = (bja - da) as usize;
            for o in cm..apa {
                let off = o as usize * stride + da as usize;
                if off + aoy <= ov_buf.len() {
                    unsafe {
                        core::ptr::copy_nonoverlapping(
                            ov_buf.as_ptr().add(off),
                            ptr.add(off),
                            aoy,
                        );
                    }
                }
            }
        }
        return;
    }

    
    let dcd = ZS_.lock();
    let mut cga = Bi.lock();
    if let (Some(ref ov_buf), Some(ref mut back_buf)) = (&*dcd, &mut *cga) {
        let stride = ddw as usize;
        let aoy = (bja - da) as usize;
        for o in cm..apa {
            let off = o as usize * stride + da as usize;
            if off + aoy <= ov_buf.len() && off + aoy <= back_buf.len() {
                unsafe {
                    core::ptr::copy_nonoverlapping(
                        ov_buf.as_ptr().add(off),
                        back_buf.as_mut_ptr().add(off),
                        aoy,
                    );
                }
            }
        }
    }
}




pub fn add_dirty_rect(x: u32, y: u32, w: u32, h: u32) {
    NR_.lock().add(DirtyRect::new(x, y, w, h));
}


pub fn mark_full_redraw() {
    NR_.lock().mark_full_redraw();
}


pub fn hlg() {
    NR_.lock().clear();
}


pub fn needs_full_redraw() -> bool {
    NR_.lock().full_redraw
}


pub fn mcx() -> ([DirtyRect; HH_], usize, bool) {
    let jg = NR_.lock();
    (jg.rects, jg.count, jg.full_redraw)
}


pub fn qye() {
    let addr = BL_.load(Ordering::SeqCst);
    if addr.is_null() {
        return;
    }
    
    let width = X_.load(Ordering::SeqCst) as usize;
    let height = W_.load(Ordering::SeqCst) as usize;
    let pitch = CB_.load(Ordering::SeqCst) as usize;
    
    let (rects, count, full_redraw) = mcx();
    
    if full_redraw {
        
        ii();
        hlg();
        return;
    }
    
    if let Some(ref buf) = *Bi.lock() {
        for i in 0..count {
            let rect = &rects[i];
            if !rect.is_valid() { continue; }
            
            let x1 = (rect.x as usize).min(width);
            let y1 = (rect.y as usize).min(height);
            let x2 = ((rect.x + rect.w) as usize).min(width);
            let y2 = ((rect.y + rect.h) as usize).min(height);
            
            for y in y1..y2 {
                let azu = y * width + x1;
                let afd = y * pitch / 4 + x1; 
                let len = x2 - x1;
                
                unsafe {
                    let src = buf.as_ptr().add(azu);
                    let dst = (addr as *mut u32).add(y * pitch / 4 + x1);
                    core::ptr::copy_nonoverlapping(src, dst, len);
                }
            }
        }
    }
    
    hlg();
}


pub fn qho() -> usize {
    Ck.lock().cursor_y
}


pub fn qhn() -> usize {
    Ck.lock().cursor_x
}


pub fn olx(lines: usize) {
    if !YB_.load(Ordering::SeqCst) {
        return;
    }
    
    let width = X_.load(Ordering::SeqCst) as usize;
    let height = W_.load(Ordering::SeqCst) as usize;
    if width == 0 || height == 0 {
        return;
    }
    
    let yh = height / BP_;
    
    let mut scrollback = Fy.lock();
    if let Some(ref mut cv) = *scrollback {
        let aab = cv.total_lines().saturating_sub(yh);
        cv.scroll_offset = (cv.scroll_offset + lines).min(aab);
        cv.is_scrolled = cv.scroll_offset > 0;
        
        if cv.is_scrolled {
            
            let offset = cv.scroll_offset;
            let av = cv.total_lines();
            drop(scrollback);
            gqt(offset, av, yh);
        }
    }
}


pub fn scroll_down(lines: usize) {
    if !YB_.load(Ordering::SeqCst) {
        return;
    }
    
    let width = X_.load(Ordering::SeqCst) as usize;
    let height = W_.load(Ordering::SeqCst) as usize;
    if width == 0 || height == 0 {
        return;
    }
    
    let yh = height / BP_;
    
    let mut scrollback = Fy.lock();
    if let Some(ref mut cv) = *scrollback {
        cv.scroll_offset = cv.scroll_offset.saturating_sub(lines);
        cv.is_scrolled = cv.scroll_offset > 0;
        
        let offset = cv.scroll_offset;
        let av = cv.total_lines();
        drop(scrollback);
        
        gqt(offset, av, yh);
    }
}


pub fn olw() {
    if let Some(ref mut cv) = *Fy.lock() {
        cv.scroll_offset = 0;
        cv.is_scrolled = false;
    }
}



pub fn jam() -> (usize, usize) {
    let av = {
        let mut scrollback = Fy.lock();
        if let Some(ref mut cv) = *scrollback {
            cv.scroll_offset = 0;
            cv.is_scrolled = false;
            cv.total_lines()
        } else {
            return (0, 0);
        }
    };

    let height = W_.load(Ordering::SeqCst) as usize;
    if height == 0 {
        return (0, 0);
    }
    let yh = height / BP_;
    gqt(0, av, yh);

    let console = Ck.lock();
    (console.cursor_x, console.cursor_y)
}


pub fn geb() -> bool {
    if let Some(ref cv) = *Fy.lock() {
        cv.is_scrolled
    } else {
        false
    }
}


pub fn qil() -> (usize, usize) {
    if let Some(ref cv) = *Fy.lock() {
        (cv.scroll_offset, cv.total_lines())
    } else {
        (0, 0)
    }
}


fn gqt(scroll_offset: usize, total_lines: usize, yh: usize) {
    let width = X_.load(Ordering::SeqCst) as usize;
    let cols = width / CL_;
    
    
    let bg = Ck.lock().bg_color;
    kkw(bg);
    
    
    
    
    
    
    
    
    let mlq = if scroll_offset == 0 {
        yh.saturating_sub(1)
    } else {
        yh
    };
    let jii = total_lines.saturating_sub(mlq + scroll_offset);
    let hvt = total_lines.saturating_sub(scroll_offset);
    
    let mut gfs = 0usize;
    let mut gft = 0usize;
    let mut fzu = false;
    
    let scrollback = Fy.lock();
    if let Some(ref cv) = *scrollback {
        for (bor, xf) in (jii..hvt).enumerate() {
            if xf >= cv.lines.len() {
                continue;
            }
            let line = &cv.lines[xf];
            for (col, i) in (0..line.len.min(cols)).enumerate() {
                let c = line.chars[i];
                let (fg, bg) = line.colors[i];
                let p = col * CL_;
                let o = bor * BP_;
                draw_char(c, p, o, fg, bg);
            }
        }
        
        
        
        if scroll_offset == 0 {
            let bor = hvt.saturating_sub(jii);
            if bor < yh && cv.current_line.len > 0 {
                for col in 0..cv.current_line.len.min(cols) {
                    let c = cv.current_line.chars[col];
                    let (fg, bg) = cv.current_line.colors[col];
                    let p = col * CL_;
                    let o = bor * BP_;
                    draw_char(c, p, o, fg, bg);
                }
                gft = bor;
                gfs = cv.current_line.len.min(cols);
                fzu = true;
            } else if bor < yh {
                
                gft = bor;
                gfs = 0;
                fzu = true;
            }
        }
    }
    drop(scrollback);
    
    
    if scroll_offset == 0 && fzu {
        let mut console = Ck.lock();
        console.cursor_y = gft;
        console.cursor_x = gfs;
    }
    
    
    if scroll_offset > 0 {
        let cat = format!("-- SCROLL: +{} lines --", scroll_offset);
        let afu = cols.saturating_sub(cat.len() + 2);
        for (i, ch) in cat.chars().enumerate() {
            let p = (afu + i) * CL_;
            draw_char(ch, p, 0, 0xFFFFFF00, 0xFF000000); 
        }
    }
}


fn kkw(color: u32) {
    let addr = BL_.load(Ordering::SeqCst);
    if addr.is_null() {
        return;
    }
    
    let height = W_.load(Ordering::SeqCst) as usize;
    let pitch = CB_.load(Ordering::SeqCst) as usize;
    
    for y in 0..height {
        let row = unsafe { addr.add(y * pitch) };
        for x in 0..(pitch / 4) {
            unsafe {
                row.add(x * 4).cast::<u32>().write_volatile(color);
            }
        }
    }
}
