











use alloc::vec::Vec;
use alloc::vec;
use alloc::boxed::Box;
use core::sync::atomic::{AtomicBool, Ordering};
use spin::Mutex;

use crate::pci::{self, L};
use crate::memory;






pub const DEN_: u16 = 0x1050;

pub const DEO_: u16 = 0x1AF4;


pub mod virtio_cap {
    pub const AQU_: u8 = 1;
    pub const BDR_: u8 = 2;
    pub const AZL_: u8 = 3;
    pub const ASD_: u8 = 4;
    pub const EOW_: u8 = 5;
}


pub mod dev_status {
    pub const Gf: u8 = 1;
    pub const Cl: u8 = 2;
    pub const IQ_: u8 = 4;
    pub const NY_: u8 = 8;
    pub const EOP_: u8 = 64;
    pub const Sa: u8 = 128;
}


pub mod features {
    pub const DFR_: u64 = 1 << 0;
    pub const BKS_: u64 = 1 << 1;
    pub const EPA_: u64 = 1 << 2;
    pub const EOZ_: u64 = 1 << 3;
    pub const DEM_: u64 = 1 << 32;
}


#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq)]
#[allow(dead_code)]
pub enum GpuCtrlType {
    
    CmdGetDisplayInfo = 0x0100,
    CmdResourceCreate2d = 0x0101,
    CmdResourceUnref = 0x0102,
    CmdSetScanout = 0x0103,
    CmdResourceFlush = 0x0104,
    CmdTransferToHost2d = 0x0105,
    CmdResourceAttachBacking = 0x0106,
    CmdResourceDetachBacking = 0x0107,
    CmdGetCapsetInfo = 0x0108,
    CmdGetCapset = 0x0109,
    CmdGetEdid = 0x010a,

    
    CmdCtxCreate = 0x0200,
    CmdCtxDestroy = 0x0201,
    CmdCtxAttachResource = 0x0202,
    CmdCtxDetachResource = 0x0203,
    CmdResourceCreate3d = 0x0204,
    CmdTransferToHost3d = 0x0205,
    CmdTransferFromHost3d = 0x0206,
    CmdSubmit3d = 0x0207,

    
    CmdUpdateCursor = 0x0300,
    CmdMoveCursor = 0x0301,

    
    RespOkNodata = 0x1100,
    RespOkDisplayInfo = 0x1101,
    RespOkCapsetInfo = 0x1102,
    RespOkCapset = 0x1103,
    RespOkEdid = 0x1104,

    
    RespErrUnspec = 0x1200,
    RespErrOutOfMemory = 0x1201,
    RespErrInvalidScanoutId = 0x1202,
    RespErrInvalidResourceId = 0x1203,
    RespErrInvalidContextId = 0x1204,
    RespErrInvalidParameter = 0x1205,
}


#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq)]
#[allow(dead_code)]
pub enum GpuFormat {
    B8G8R8A8Unorm = 1,
    B8G8R8X8Unorm = 2,
    A8R8G8B8Unorm = 3,
    X8R8G8B8Unorm = 4,
    R8G8B8A8Unorm = 67,
    X8B8G8R8Unorm = 68,
    A8B8G8R8Unorm = 121,
    R8G8B8X8Unorm = 134,
}






#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct Ac {
    pub ctrl_type: u32,
    pub flags: u32,
    pub fence_id: u64,
    pub ctx_id: u32,
    pub eys: u8,
    pub padding: [u8; 3],
}


#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct Eh {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}


#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct Akw {
    pub r: Eh,
    pub enabled: u32,
    pub flags: u32,
}


#[repr(C)]
#[derive(Clone, Copy)]
pub struct Zl {
    pub kp: Ac,
    pub pmodes: [Akw; 16],
}


#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Zk {
    pub kp: Ac,
    pub zj: u32,
    pub format: u32,
    pub width: u32,
    pub height: u32,
}


#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Zm {
    pub kp: Ac,
    pub r: Eh,
    pub scanout_id: u32,
    pub zj: u32,
}


#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Jv {
    pub kp: Ac,
    pub r: Eh,
    pub zj: u32,
    pub padding: u32,
}


#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Jw {
    pub kp: Ac,
    pub r: Eh,
    pub offset: u64,
    pub zj: u32,
    pub padding: u32,
}


#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Zj {
    pub addr: u64,
    pub length: u32,
    pub padding: u32,
}


#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Sr {
    pub kp: Ac,
    pub zj: u32,
    pub nr_entries: u32,
}





#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
struct VirtqDesc {
    addr: u64,
    len: u32,
    flags: u16,
    next: u16,
}

const RJ_: u16 = 1;
const RK_: u16 = 2;

#[repr(C)]
struct Kx {
    flags: u16,
    idx: u16,
}

#[repr(C)]
#[derive(Clone, Copy, Default)]
struct Qv {
    id: u32,
    len: u32,
}

#[repr(C)]
struct Ky {
    flags: u16,
    idx: u16,
}


struct GpuVirtqueue {
    size: u16,
    _phys_base: u64,
    _virt_base: *mut u8,
    desc: *mut VirtqDesc,
    avail: *mut Kx,
    used: *mut Ky,
    free_head: u16,
    num_free: u16,
    free_list: Vec<u16>,
    last_used_idx: u16,
}

unsafe impl Send for GpuVirtqueue {}
unsafe impl Sync for GpuVirtqueue {}

impl GpuVirtqueue {
    fn new(size: u16) -> Result<Self, &'static str> {
        use alloc::alloc::{alloc_zeroed, Layout};
        
        let cwj = core::mem::size_of::<VirtqDesc>() * size as usize;
        let fhs = 6 + 2 * size as usize;
        let bps = ((cwj + fhs) + 4095) & !4095;
        let pqn = 6 + core::mem::size_of::<Qv>() * size as usize;
        let total_size = bps + pqn + 4096;
        
        let layout = Layout::from_size_align(total_size, 4096)
            .map_err(|_| "Invalid virtqueue layout")?;
        let ptr = unsafe { alloc_zeroed(layout) };
        if ptr.is_null() { return Err("Failed to allocate virtqueue"); }
        
        let virt_addr = ptr as u64;
        let bz = memory::hhdm_offset();
        let phys_addr = if virt_addr >= bz { virt_addr - bz } else { virt_addr };
        
        let desc = ptr as *mut VirtqDesc;
        let avail = unsafe { ptr.add(cwj) as *mut Kx };
        let used = unsafe { ptr.add(bps) as *mut Ky };
        
        let mut free_list = vec![0u16; size as usize];
        for i in 0..(size as usize).saturating_sub(1) {
            free_list[i] = (i + 1) as u16;
        }
        if size > 0 { free_list[size as usize - 1] = 0xFFFF; }
        
        Ok(Self {
            size,
            _phys_base: phys_addr,
            _virt_base: ptr,
            desc,
            avail,
            used,
            free_head: 0,
            num_free: size,
            free_list,
            last_used_idx: 0,
        })
    }
    
    fn alloc_desc(&mut self) -> Option<u16> {
        if self.num_free == 0 { return None; }
        let idx = self.free_head;
        self.free_head = self.free_list[idx as usize];
        self.num_free -= 1;
        Some(idx)
    }
    
    fn free_desc(&mut self, idx: u16) {
        self.free_list[idx as usize] = self.free_head;
        self.free_head = idx;
        self.num_free += 1;
    }
    
    fn set_desc(&mut self, idx: u16, addr: u64, len: u32, flags: u16, next: u16) {
        unsafe {
            let d = &mut *self.desc.add(idx as usize);
            d.addr = addr;
            d.len = len;
            d.flags = flags;
            d.next = next;
        }
    }
    
    fn submit(&mut self, su: u16) {
        unsafe {
            let avail = &mut *self.avail;
            let cpm = (self.avail as *mut u8).add(4) as *mut u16;
            let idx = avail.idx;
            *cpm.add((idx % self.size) as usize) = su;
            core::sync::atomic::fence(Ordering::Release);
            avail.idx = idx.wrapping_add(1);
        }
    }
    
    fn poll_used(&mut self) -> Option<(u32, u32)> {
        unsafe {
            core::sync::atomic::fence(Ordering::Acquire);
            let used = &*self.used;
            if used.idx == self.last_used_idx { return None; }
            let cpm = (self.used as *mut u8).add(4) as *mut Qv;
            let cit = *cpm.add((self.last_used_idx % self.size) as usize);
            self.last_used_idx = self.last_used_idx.wrapping_add(1);
            Some((cit.id, cit.len))
        }
    }
    
    fn desc_phys(&self) -> u64 { self._phys_base }
    fn avail_phys(&self) -> u64 {
        let cwj = core::mem::size_of::<VirtqDesc>() * self.size as usize;
        self._phys_base + cwj as u64
    }
    fn used_phys(&self) -> u64 {
        let cwj = core::mem::size_of::<VirtqDesc>() * self.size as usize;
        let fhs = 6 + 2 * self.size as usize;
        let bps = ((cwj + fhs) + 4095) & !4095;
        self._phys_base + bps as u64
    }
}





struct Fv {
    base: *mut u8,
    _len: u32,
}

unsafe impl Send for Fv {}
unsafe impl Sync for Fv {}

impl Fv {
    fn read8(&self, offset: u32) -> u8 {
        unsafe { core::ptr::read_volatile(self.base.add(offset as usize)) }
    }
    fn read16(&self, offset: u32) -> u16 {
        unsafe { core::ptr::read_volatile(self.base.add(offset as usize) as *const u16) }
    }
    fn read32(&self, offset: u32) -> u32 {
        unsafe { core::ptr::read_volatile(self.base.add(offset as usize) as *const u32) }
    }
    fn write8(&self, offset: u32, val: u8) {
        unsafe { core::ptr::write_volatile(self.base.add(offset as usize), val) }
    }
    fn write16(&self, offset: u32, val: u16) {
        unsafe { core::ptr::write_volatile(self.base.add(offset as usize) as *mut u16, val) }
    }
    fn write32(&self, offset: u32, val: u32) {
        unsafe { core::ptr::write_volatile(self.base.add(offset as usize) as *mut u32, val) }
    }
    fn write64(&self, offset: u32, val: u64) {
        self.write32(offset, val as u32);
        self.write32(offset + 4, (val >> 32) as u32);
    }
}


mod common_cfg {
    pub const ASF_: u32 = 0x00;
    pub const ASE_: u32 = 0x04;
    pub const ASU_: u32 = 0x08;
    pub const AST_: u32 = 0x0C;
    pub const EOU_: u32 = 0x10;
    pub const EOV_: u32 = 0x12;
    pub const ES_: u32 = 0x14;
    pub const EON_: u32 = 0x15;
    pub const AIO_: u32 = 0x16;
    pub const XV_: u32 = 0x18;
    pub const CQJ_: u32 = 0x1A;
    pub const CQI_: u32 = 0x1C;
    pub const CQL_: u32 = 0x1E;
    pub const CQF_: u32 = 0x20;
    pub const CQH_: u32 = 0x28;
    pub const CQG_: u32 = 0x30;
}


mod gpu_cfg {
    pub const EOT_: u32 = 0x00;
    pub const EOS_: u32 = 0x04;
    pub const CLM_: u32 = 0x08;
    pub const CLJ_: u32 = 0x0C;
}





struct DmaCommandBuffer {
    phys: u64,
    virt: *mut u8,
    bek: usize,
}

unsafe impl Send for DmaCommandBuffer {}
unsafe impl Sync for DmaCommandBuffer {}

impl DmaCommandBuffer {
    fn new(size: usize) -> Result<Self, &'static str> {
        use alloc::alloc::{alloc_zeroed, Layout};
        let layout = Layout::from_size_align(size, 4096)
            .map_err(|_| "DMA buffer layout error")?;
        let ptr = unsafe { alloc_zeroed(layout) };
        if ptr.is_null() { return Err("DMA buffer allocation failed"); }
        let virt = ptr as u64;
        let bz = memory::hhdm_offset();
        let phys = if virt >= bz { virt - bz } else { virt };
        Ok(Self { phys, virt: ptr, bek: size })
    }
    
    unsafe fn write_at<T: Copy>(&self, offset: usize, val: &T) {
        core::ptr::write_volatile(self.virt.add(offset) as *mut T, *val);
    }
    
    unsafe fn read_at<T: Copy>(&self, offset: usize) -> T {
        core::ptr::read_volatile(self.virt.add(offset) as *const T)
    }
    
    fn qqh(&self, offset: usize) -> u64 { self.phys + offset as u64 }
}






pub struct GpuSurface {
    pub zj: u32,
    pub width: u32,
    pub height: u32,
    pub data: Box<[u32]>,
}

impl GpuSurface {
    pub fn new(width: u32, height: u32) -> Self {
        let size = (width * height) as usize;
        Self {
            zj: 0,
            width,
            height,
            data: alloc::vec![0u32; size].into_boxed_slice(),
        }
    }
    
    pub fn clear(&mut self, color: u32) { self.data.fill(color); }
    
    #[inline]
    pub fn set_pixel(&mut self, x: u32, y: u32, color: u32) {
        if x < self.width && y < self.height {
            self.data[(y * self.width + x) as usize] = color;
        }
    }
    
    #[inline]
    pub fn get_pixel(&self, x: u32, y: u32) -> u32 {
        if x < self.width && y < self.height {
            self.data[(y * self.width + x) as usize]
        } else { 0 }
    }
    
    pub fn fill_rect(&mut self, x: u32, y: u32, w: u32, h: u32, color: u32) {
        let x1 = x.min(self.width);
        let y1 = y.min(self.height);
        let x2 = (x + w).min(self.width);
        let y2 = (y + h).min(self.height);
        for o in y1..y2 {
            let start = (o * self.width + x1) as usize;
            let end = (o * self.width + x2) as usize;
            self.data[start..end].fill(color);
        }
    }
    
    pub fn blit(&mut self, src: &GpuSurface, dst_x: i32, dst_y: i32) {
        for ak in 0..src.height {
            for am in 0..src.width {
                let dx = dst_x + am as i32;
                let ad = dst_y + ak as i32;
                if dx >= 0 && ad >= 0 && dx < self.width as i32 && ad < self.height as i32 {
                    let ct = src.get_pixel(am, ak);
                    let alpha = (ct >> 24) & 0xFF;
                    if alpha >= 128 {
                        self.set_pixel(dx as u32, ad as u32, ct);
                    }
                }
            }
        }
    }
    
    fn set_pixel_safe(&mut self, x: i32, y: i32, color: u32) {
        if x >= 0 && y >= 0 && x < self.width as i32 && y < self.height as i32 {
            self.set_pixel(x as u32, y as u32, color);
        }
    }

    pub fn draw_line(&mut self, bm: i32, az: i32, x1: i32, y1: i32, color: u32) {
        let dx = (x1 - bm).abs();
        let ad = -(y1 - az).abs();
        let am = if bm < x1 { 1 } else { -1 };
        let ak = if az < y1 { 1 } else { -1 };
        let mut err = dx + ad;
        let (mut x, mut y) = (bm, az);
        loop {
            self.set_pixel_safe(x, y, color);
            if x == x1 && y == y1 { break; }
            let pg = 2 * err;
            if pg >= ad { err += ad; x += am; }
            if pg <= dx { err += dx; y += ak; }
        }
    }

    pub fn draw_rect(&mut self, x: u32, y: u32, w: u32, h: u32, color: u32) {
        let (x, y, w, h) = (x as i32, y as i32, w as i32, h as i32);
        self.draw_line(x, y, x+w-1, y, color);
        self.draw_line(x, y+h-1, x+w-1, y+h-1, color);
        self.draw_line(x, y, x, y+h-1, color);
        self.draw_line(x+w-1, y, x+w-1, y+h-1, color);
    }

    pub fn draw_circle(&mut self, cx: i32, u: i32, radius: i32, color: u32) {
        let (mut x, mut y, mut err) = (radius, 0i32, 0i32);
        while x >= y {
            self.set_pixel_safe(cx+x, u+y, color);
            self.set_pixel_safe(cx+y, u+x, color);
            self.set_pixel_safe(cx-y, u+x, color);
            self.set_pixel_safe(cx-x, u+y, color);
            self.set_pixel_safe(cx-x, u-y, color);
            self.set_pixel_safe(cx-y, u-x, color);
            self.set_pixel_safe(cx+y, u-x, color);
            self.set_pixel_safe(cx+x, u-y, color);
            y += 1;
            err += 1 + 2*y;
            if 2*(err-x)+1 > 0 { x -= 1; err += 1 - 2*x; }
        }
    }

    pub fn fill_circle(&mut self, cx: i32, u: i32, radius: i32, color: u32) {
        for ad in -radius..=radius {
            for dx in -radius..=radius {
                if dx*dx + ad*ad <= radius*radius {
                    self.set_pixel_safe(cx+dx, u+ad, color);
                }
            }
        }
    }

    pub fn draw_rounded_rect(&mut self, x: u32, y: u32, w: u32, h: u32, _radius: u32, color: u32) {
        self.draw_rect(x, y, w, h, color);
    }

    pub fn fill_rounded_rect(&mut self, x: u32, y: u32, w: u32, h: u32, _radius: u32, color: u32) {
        self.fill_rect(x, y, w, h, color);
    }

    pub fn pyv(&mut self, src: &GpuSurface, dst_x: i32, dst_y: i32, dst_w: u32, dst_h: u32) {
        if dst_w == 0 || dst_h == 0 || src.width == 0 || src.height == 0 { return; }
        for ad in 0..dst_h {
            for dx in 0..dst_w {
                let am = (dx * src.width) / dst_w;
                let ak = (ad * src.height) / dst_h;
                let p = dst_x + dx as i32;
                let o = dst_y + ad as i32;
                if p >= 0 && o >= 0 && p < self.width as i32 && o < self.height as i32 {
                    self.set_pixel(p as u32, o as u32, src.get_pixel(am, ak));
                }
            }
        }
    }
}





pub struct VirtioGpu {
    _pci_dev: Option<L>,
    common_cfg: Option<Fv>,
    notify_cfg: Option<Fv>,
    _isr_cfg: Option<Fv>,
    device_cfg: Option<Fv>,
    _notify_off_multiplier: u32,
    controlq: Option<GpuVirtqueue>,
    dma_buf: Option<DmaCommandBuffer>,
    display_width: u32,
    display_height: u32,
    num_scanouts: u32,
    next_resource_id: u32,
    scanout_resource_id: u32,
    backing_buffer: Option<Box<[u32]>>,
    backing_phys: u64,
    initialized: bool,
    has_3d: bool,
    
    back_resource_id: u32,
    back_buffer: Option<Box<[u32]>>,
    back_phys: u64,
    double_buffer_enabled: bool,
    front_is_a: bool, 
}

impl VirtioGpu {
    pub const fn new() -> Self {
        Self {
            _pci_dev: None,
            common_cfg: None,
            notify_cfg: None,
            _isr_cfg: None,
            device_cfg: None,
            _notify_off_multiplier: 0,
            controlq: None,
            dma_buf: None,
            display_width: 0,
            display_height: 0,
            num_scanouts: 0,
            next_resource_id: 1,
            scanout_resource_id: 0,
            backing_buffer: None,
            backing_phys: 0,
            initialized: false,
            has_3d: false,
            back_resource_id: 0,
            back_buffer: None,
            back_phys: 0,
            double_buffer_enabled: false,
            front_is_a: true,
        }
    }
    
    fn etu(s: &L, agc: u8, offset: u32, length: u32) -> Result<Fv, &'static str> {
        let fib = s.bar_address(agc as usize)
            .ok_or("BAR not configured")?;
        if !s.bar_is_memory(agc as usize) {
            return Err("Expected memory BAR, got I/O");
        }
        let phys = fib + offset as u64;
        let virt = memory::yv(phys, length.max(4096) as usize)?;
        crate::serial_println!("[VIRTIO-GPU] Mapped BAR{}: phys={:#X} virt={:#X} len={}", 
            agc, phys, virt, length);
        Ok(Fv { base: virt as *mut u8, _len: length })
    }
    
    
    pub fn init(&mut self, s: L) -> Result<(), &'static str> {
        crate::serial_println!("[VIRTIO-GPU] === Initializing VirtIO GPU ===");
        crate::serial_println!("[VIRTIO-GPU] PCI {:02X}:{:02X}.{} vid={:#06X} did={:#06X}",
            s.bus, s.device, s.function, s.vendor_id, s.device_id);
        
        pci::bzi(&s);
        pci::bzj(&s);
        
        
        let caps = pci::lwc(&s);
        if caps.is_empty() {
            return Err("No VirtIO capabilities found");
        }
        
        crate::serial_println!("[VIRTIO-GPU] Found {} VirtIO capabilities", caps.len());
        
        let mut gjs: u8 = 0;
        
        for &(wa, ehn, bar, offset, length) in &caps {
            let name = match ehn {
                1 => "COMMON", 2 => "NOTIFY", 3 => "ISR", 4 => "DEVICE", 5 => "PCI", _ => "?",
            };
            crate::serial_println!("[VIRTIO-GPU]   cap@{:#X}: {} BAR{} off={:#X} len={}", 
                wa, name, bar, offset, length);
            
            match ehn {
                virtio_cap::AQU_ => {
                    self.common_cfg = Some(Self::etu(&s, bar, offset, length)?);
                }
                virtio_cap::BDR_ => {
                    self.notify_cfg = Some(Self::etu(&s, bar, offset, length)?);
                    gjs = wa;
                }
                virtio_cap::AZL_ => {
                    self._isr_cfg = Some(Self::etu(&s, bar, offset, length)?);
                }
                virtio_cap::ASD_ => {
                    self.device_cfg = Some(Self::etu(&s, bar, offset, length)?);
                }
                _ => {}
            }
        }
        
        
        if self.common_cfg.is_none() { return Err("Missing COMMON_CFG"); }
        if self.notify_cfg.is_none() { return Err("Missing NOTIFY_CFG"); }
        if self.device_cfg.is_none() { return Err("Missing DEVICE_CFG"); }
        
        if gjs > 0 {
            self._notify_off_multiplier = pci::ocv(&s, gjs);
        }
        
        
        
        
        
        self.common_write8(common_cfg::ES_, 0);
        for _ in 0..10000 { core::hint::spin_loop(); }
        
        
        self.common_write8(common_cfg::ES_, dev_status::Gf);
        
        
        self.common_write8(common_cfg::ES_, dev_status::Gf | dev_status::Cl);
        
        
        self.common_write32(common_cfg::ASF_, 0);
        let luu = self.common_read32(common_cfg::ASE_);
        self.common_write32(common_cfg::ASF_, 1);
        let lut = self.common_read32(common_cfg::ASE_);
        let device_features = (luu as u64) | ((lut as u64) << 32);
        
        crate::serial_println!("[VIRTIO-GPU] Device features: {:#018X}", device_features);
        self.has_3d = device_features & features::DFR_ != 0;
        
        let mut driver_features = features::DEM_;
        if device_features & features::BKS_ != 0 {
            driver_features |= features::BKS_;
        }
        
        self.common_write32(common_cfg::ASU_, 0);
        self.common_write32(common_cfg::AST_, driver_features as u32);
        self.common_write32(common_cfg::ASU_, 1);
        self.common_write32(common_cfg::AST_, (driver_features >> 32) as u32);
        
        
        self.common_write8(common_cfg::ES_,
            dev_status::Gf | dev_status::Cl | dev_status::NY_);
        
        let status = self.common_read8(common_cfg::ES_);
        if status & dev_status::NY_ == 0 {
            self.common_write8(common_cfg::ES_, dev_status::Sa);
            return Err("Device rejected features");
        }
        crate::serial_println!("[VIRTIO-GPU] Features OK (3D={})", self.has_3d);
        
        
        self.setup_controlq()?;
        
        
        self.common_write8(common_cfg::ES_,
            dev_status::Gf | dev_status::Cl | dev_status::NY_ | dev_status::IQ_);
        crate::serial_println!("[VIRTIO-GPU] DRIVER_OK set");
        
        
        self.dma_buf = Some(DmaCommandBuffer::new(8192)?);
        
        
        self.num_scanouts = self.device_read32(gpu_cfg::CLM_);
        let nlr = self.device_read32(gpu_cfg::CLJ_);
        crate::serial_println!("[VIRTIO-GPU] scanouts={} capsets={}", self.num_scanouts, nlr);
        
        
        self.get_display_info()?;
        
        self._pci_dev = Some(s);
        self.initialized = true;
        
        crate::serial_println!("[VIRTIO-GPU] === Init complete: {}x{} ===", 
            self.display_width, self.display_height);
        Ok(())
    }
    
    
    fn common_write8(&self, offset: u32, val: u8) {
        if let Some(c) = &self.common_cfg { c.write8(offset, val); }
    }
    fn qay(&self, offset: u32, val: u16) {
        if let Some(c) = &self.common_cfg { c.write16(offset, val); }
    }
    fn common_write32(&self, offset: u32, val: u32) {
        if let Some(c) = &self.common_cfg { c.write32(offset, val); }
    }
    fn common_read8(&self, offset: u32) -> u8 {
        self.common_cfg.as_ref().map(|c| c.read8(offset)).unwrap_or(0)
    }
    fn qax(&self, offset: u32) -> u16 {
        self.common_cfg.as_ref().map(|c| c.read16(offset)).unwrap_or(0)
    }
    fn common_read32(&self, offset: u32) -> u32 {
        self.common_cfg.as_ref().map(|c| c.read32(offset)).unwrap_or(0)
    }
    fn device_read32(&self, offset: u32) -> u32 {
        self.device_cfg.as_ref().map(|c| c.read32(offset)).unwrap_or(0)
    }
    
    fn setup_controlq(&mut self) -> Result<(), &'static str> {
        let brb = self.common_cfg.as_ref().ok_or("Missing COMMON_CFG")?;
        brb.write16(common_cfg::AIO_, 0);
        let max_size = brb.read16(common_cfg::XV_);
        crate::serial_println!("[VIRTIO-GPU] controlq max_size={}", max_size);
        if max_size == 0 { return Err("controlq not available"); }
        
        let queue_size = max_size.min(64);
        brb.write16(common_cfg::XV_, queue_size);
        
        let fer = GpuVirtqueue::new(queue_size)?;
        
        brb.write64(common_cfg::CQF_, fer.desc_phys());
        brb.write64(common_cfg::CQH_, fer.avail_phys());
        brb.write64(common_cfg::CQG_, fer.used_phys());
        brb.write16(common_cfg::CQJ_, 0xFFFF);
        brb.write16(common_cfg::CQI_, 1);
        
        let pxd = brb.read16(common_cfg::CQL_);
        
        self.controlq = Some(fer);
        crate::serial_println!("[VIRTIO-GPU] controlq ready (size={})", queue_size);
        Ok(())
    }
    
    fn qps(&self) {
        if let Some(bif) = &self.notify_cfg {
            bif.write16(0, 0);
        }
    }
    
    
    fn send_command(&mut self, cmd_len: u32, resp_offset: usize, resp_len: u32) -> Result<u32, &'static str> {
        
        let hsv = self.dma_buf.as_ref().ok_or("DMA not ready")?.phys;
        
        let controlq = self.controlq.as_mut().ok_or("controlq not ready")?;
        
        let ejs = controlq.alloc_desc().ok_or("No free desc (cmd)")?;
        let eju = controlq.alloc_desc().ok_or("No free desc (resp)")?;
        
        controlq.set_desc(ejs, hsv, cmd_len, RJ_, eju);
        controlq.set_desc(eju, hsv + resp_offset as u64, resp_len, RK_, 0);
        
        controlq.submit(ejs);
        
        if let Some(bif) = &self.notify_cfg {
            bif.write16(0, 0);
        }
        
        let mut mz = 5_000_000u32;
        loop {
            if let Some(_) = controlq.poll_used() { break; }
            mz -= 1;
            if mz == 0 {
                controlq.free_desc(eju);
                controlq.free_desc(ejs);
                return Err("Command timeout");
            }
            core::hint::spin_loop();
        }
        
        let dma = self.dma_buf.as_ref().ok_or("DMA buffer not initialized")?;
        let ddm = unsafe { dma.read_at::<Ac>(resp_offset) }.ctrl_type;
        controlq.free_desc(eju);
        controlq.free_desc(ejs);
        Ok(ddm)
    }
    
    fn get_display_info(&mut self) -> Result<(), &'static str> {
        let dma = self.dma_buf.as_ref().ok_or("DMA not ready")?;
        
        let cmd = Ac {
            ctrl_type: GpuCtrlType::CmdGetDisplayInfo as u32,
            ..Default::default()
        };
        unsafe { dma.write_at(0, &cmd); }
        
        let ddm = self.send_command(
            core::mem::size_of::<Ac>() as u32,
            512, 
            core::mem::size_of::<Zl>() as u32,
        )?;
        
        if ddm != GpuCtrlType::RespOkDisplayInfo as u32 {
            crate::serial_println!("[VIRTIO-GPU] GET_DISPLAY_INFO failed: {:#X}", ddm);
            return Err("GET_DISPLAY_INFO failed");
        }
        
        let dma = self.dma_buf.as_ref().ok_or("DMA buffer not initialized")?;
        let eo: Zl = unsafe { dma.read_at(512) };
        
        for (i, pm) in eo.pmodes.iter().enumerate() {
            if pm.enabled != 0 {
                self.display_width = pm.r.width;
                self.display_height = pm.r.height;
                crate::serial_println!("[VIRTIO-GPU] Display {}: {}x{}", i, pm.r.width, pm.r.height);
                break;
            }
        }
        
        if self.display_width == 0 {
            self.display_width = 1280;
            self.display_height = 800;
            crate::serial_println!("[VIRTIO-GPU] Defaulting to {}x{}", self.display_width, self.display_height);
        }
        Ok(())
    }
    
    pub fn create_resource_2d(&mut self, width: u32, height: u32) -> Result<u32, &'static str> {
        let id = self.next_resource_id;
        self.next_resource_id += 1;
        let dma = self.dma_buf.as_ref().ok_or("DMA not ready")?;
        
        let cmd = Zk {
            kp: Ac { ctrl_type: GpuCtrlType::CmdResourceCreate2d as u32, ..Default::default() },
            zj: id,
            format: GpuFormat::B8G8R8X8Unorm as u32,
            width,
            height,
        };
        unsafe { dma.write_at(0, &cmd); }
        
        let eo = self.send_command(
            core::mem::size_of::<Zk>() as u32,
            512, core::mem::size_of::<Ac>() as u32,
        )?;
        
        if eo != GpuCtrlType::RespOkNodata as u32 {
            return Err("RESOURCE_CREATE_2D failed");
        }
        crate::serial_println!("[VIRTIO-GPU] Resource {} created ({}x{})", id, width, height);
        Ok(id)
    }
    
    pub fn attach_backing(&mut self, zj: u32, hg: u64, buf_len: u32) -> Result<(), &'static str> {
        let dma = self.dma_buf.as_ref().ok_or("DMA not ready")?;
        
        let cmd = Sr {
            kp: Ac { ctrl_type: GpuCtrlType::CmdResourceAttachBacking as u32, ..Default::default() },
            zj,
            nr_entries: 1,
        };
        unsafe { dma.write_at(0, &cmd); }
        
        let entry = Zj { addr: hg, length: buf_len, padding: 0 };
        unsafe { dma.write_at(core::mem::size_of::<Sr>(), &entry); }
        
        let fnd = (core::mem::size_of::<Sr>() + core::mem::size_of::<Zj>()) as u32;
        let eo = self.send_command(fnd, 512, core::mem::size_of::<Ac>() as u32)?;
        
        if eo != GpuCtrlType::RespOkNodata as u32 {
            return Err("ATTACH_BACKING failed");
        }
        crate::serial_println!("[VIRTIO-GPU] Backing attached: phys={:#X} len={}", hg, buf_len);
        Ok(())
    }
    
    pub fn set_scanout(&mut self, scanout_id: u32, zj: u32, w: u32, h: u32) -> Result<(), &'static str> {
        let dma = self.dma_buf.as_ref().ok_or("DMA not ready")?;
        let cmd = Zm {
            kp: Ac { ctrl_type: GpuCtrlType::CmdSetScanout as u32, ..Default::default() },
            r: Eh { x: 0, y: 0, width: w, height: h },
            scanout_id,
            zj,
        };
        unsafe { dma.write_at(0, &cmd); }
        
        let eo = self.send_command(
            core::mem::size_of::<Zm>() as u32,
            512, core::mem::size_of::<Ac>() as u32,
        )?;
        
        if eo != GpuCtrlType::RespOkNodata as u32 { return Err("SET_SCANOUT failed"); }
        self.scanout_resource_id = zj;
        crate::serial_println!("[VIRTIO-GPU] Scanout {} -> resource {} ({}x{})", scanout_id, zj, w, h);
        Ok(())
    }
    
    pub fn raw(&mut self, zj: u32, w: u32, h: u32) -> Result<(), &'static str> {
        let dma = self.dma_buf.as_ref().ok_or("DMA not ready")?;
        let cmd = Jw {
            kp: Ac { ctrl_type: GpuCtrlType::CmdTransferToHost2d as u32, ..Default::default() },
            r: Eh { x: 0, y: 0, width: w, height: h },
            offset: 0,
            zj,
            padding: 0,
        };
        unsafe { dma.write_at(0, &cmd); }
        
        let eo = self.send_command(
            core::mem::size_of::<Jw>() as u32,
            512, core::mem::size_of::<Ac>() as u32,
        )?;
        if eo != GpuCtrlType::RespOkNodata as u32 { return Err("TRANSFER failed"); }
        Ok(())
    }
    
    pub fn qfz(&mut self, zj: u32, w: u32, h: u32) -> Result<(), &'static str> {
        let dma = self.dma_buf.as_ref().ok_or("DMA not ready")?;
        let cmd = Jv {
            kp: Ac { ctrl_type: GpuCtrlType::CmdResourceFlush as u32, ..Default::default() },
            r: Eh { x: 0, y: 0, width: w, height: h },
            zj,
            padding: 0,
        };
        unsafe { dma.write_at(0, &cmd); }
        
        let eo = self.send_command(
            core::mem::size_of::<Jv>() as u32,
            512, core::mem::size_of::<Ac>() as u32,
        )?;
        if eo != GpuCtrlType::RespOkNodata as u32 { return Err("FLUSH failed"); }
        Ok(())
    }
    
    
    
    pub fn setup_scanout(&mut self) -> Result<(), &'static str> {
        if !self.initialized { return Err("GPU not initialized"); }
        
        
        let (fb_w, fb_h) = crate::framebuffer::kv();
        if fb_w > 0 && fb_h > 0 {
            crate::serial_println!("[VIRTIO-GPU] Using framebuffer dimensions: {}x{} (display was {}x{})",
                fb_w, fb_h, self.display_width, self.display_height);
            self.display_width = fb_w;
            self.display_height = fb_h;
        }
        
        let w = self.display_width;
        let h = self.display_height;
        crate::serial_println!("[VIRTIO-GPU] Setting up scanout {}x{}", w, h);
        
        let zj = self.create_resource_2d(w, h)?;
        
        
        let ate = (w * h) as usize;
        let djy = ate * 4;
        
        use alloc::alloc::{alloc_zeroed, Layout};
        let layout = Layout::from_size_align(djy, 4096).map_err(|_| "Layout error")?;
        let ptr = unsafe { alloc_zeroed(layout) };
        if ptr.is_null() { return Err("Backing buffer allocation failed"); }
        
        let virt = ptr as u64;
        let bz = memory::hhdm_offset();
        let phys = if virt >= bz { virt - bz } else { virt };
        
        let buffer = unsafe {
            let slice = core::slice::from_raw_parts_mut(ptr as *mut u32, ate);
            Box::from_raw(slice as *mut [u32])
        };
        
        self.backing_buffer = Some(buffer);
        self.backing_phys = phys;
        
        self.attach_backing(zj, phys, djy as u32)?;
        self.set_scanout(0, zj, w, h)?;
        
        crate::serial_println!("[VIRTIO-GPU] Scanout ready! phys={:#X}", phys);
        Ok(())
    }
    
    
    
    
    pub fn setup_double_buffer(&mut self) -> Result<(), &'static str> {
        if !self.initialized { return Err("GPU not initialized"); }
        if self.scanout_resource_id == 0 { return Err("No primary scanout"); }
        
        let w = self.display_width;
        let h = self.display_height;
        
        
        let fhx = self.create_resource_2d(w, h)?;
        
        
        let ate = (w * h) as usize;
        let djy = ate * 4;
        
        use alloc::alloc::{alloc_zeroed, Layout};
        let layout = Layout::from_size_align(djy, 4096)
            .map_err(|_| "Layout error")?;
        let ptr = unsafe { alloc_zeroed(layout) };
        if ptr.is_null() { return Err("Back buffer allocation failed"); }
        
        let virt = ptr as u64;
        let bz = memory::hhdm_offset();
        let phys = if virt >= bz { virt - bz } else { virt };
        
        let buffer = unsafe {
            let slice = core::slice::from_raw_parts_mut(ptr as *mut u32, ate);
            Box::from_raw(slice as *mut [u32])
        };
        
        self.attach_backing(fhx, phys, djy as u32)?;
        
        self.back_resource_id = fhx;
        self.back_buffer = Some(buffer);
        self.back_phys = phys;
        self.double_buffer_enabled = true;
        self.front_is_a = true;
        
        crate::serial_println!("[VIRTIO-GPU] Double buffer enabled: resource A={}, B={}", 
            self.scanout_resource_id, fhx);
        Ok(())
    }
    
    
    pub fn swap_gpu_buffers(&mut self) -> Result<(), &'static str> {
        if !self.double_buffer_enabled { return Ok(()); }
        
        let (w, h) = (self.display_width, self.display_height);
        
        if self.front_is_a {
            
            self.set_scanout(0, self.back_resource_id, w, h)?;
        } else {
            
            self.set_scanout(0, self.scanout_resource_id, w, h)?;
        }
        
        self.front_is_a = !self.front_is_a;
        Ok(())
    }
    
    
    pub fn get_back_buffer(&mut self) -> Option<&mut [u32]> {
        if !self.double_buffer_enabled {
            return self.backing_buffer.as_deref_mut();
        }
        if self.front_is_a {
            
            self.back_buffer.as_deref_mut()
        } else {
            
            self.backing_buffer.as_deref_mut()
        }
    }
    
    pub fn qhe(&mut self) -> Option<&mut [u32]> {
        self.backing_buffer.as_deref_mut()
    }
    
    pub fn kv(&self) -> (u32, u32) {
        (self.display_width, self.display_height)
    }
    
    
    
    
    
    
    pub fn present(&mut self) -> Result<(), &'static str> {
        let bvl = self.scanout_resource_id;
        if bvl == 0 { return Err("No scanout"); }
        let (w, h) = (self.display_width, self.display_height);
        
        let dma = self.dma_buf.as_ref().ok_or("DMA not ready")?;
        let ali = dma.phys;
        
        
        let gzw = Jw {
            kp: Ac { ctrl_type: GpuCtrlType::CmdTransferToHost2d as u32, ..Default::default() },
            r: Eh { x: 0, y: 0, width: w, height: h },
            offset: 0,
            zj: bvl,
            padding: 0,
        };
        unsafe { dma.write_at(0, &gzw); }
        
        
        let dpz = Jv {
            kp: Ac { ctrl_type: GpuCtrlType::CmdResourceFlush as u32, ..Default::default() },
            r: Eh { x: 0, y: 0, width: w, height: h },
            zj: bvl,
            padding: 0,
        };
        unsafe { dma.write_at(256, &dpz); }
        
        let gzy = core::mem::size_of::<Jw>() as u32;
        let fxg = core::mem::size_of::<Jv>() as u32;
        let cph = core::mem::size_of::<Ac>() as u32;
        
        let controlq = self.controlq.as_mut().ok_or("controlq not ready")?;
        
        
        let aqd = controlq.alloc_desc().ok_or("No free desc")?;
        let vh = controlq.alloc_desc().ok_or("No free desc")?;
        let jq = controlq.alloc_desc().ok_or("No free desc")?;
        let aqe = controlq.alloc_desc().ok_or("No free desc")?;
        
        
        controlq.set_desc(aqd, ali, gzy, RJ_, vh);
        controlq.set_desc(vh, ali + 512, cph, RK_, 0);
        
        
        controlq.set_desc(jq, ali + 256, fxg, RJ_, aqe);
        controlq.set_desc(aqe, ali + 768, cph, RK_, 0);
        
        
        controlq.submit(aqd);
        controlq.submit(jq);
        
        
        if let Some(bif) = &self.notify_cfg {
            bif.write16(0, 0);
        }
        
        
        let mut completed = 0u8;
        let mut mz = 5_000_000u32;
        while completed < 2 {
            if let Some(_) = controlq.poll_used() {
                completed += 1;
            }
            if completed < 2 {
                mz -= 1;
                if mz == 0 {
                    controlq.free_desc(aqe);
                    controlq.free_desc(jq);
                    controlq.free_desc(vh);
                    controlq.free_desc(aqd);
                    return Err("Batched present timeout");
                }
                core::hint::spin_loop();
            }
        }
        
        
        let dma = self.dma_buf.as_ref().ok_or("DMA buffer not initialized")?;
        let pcj = unsafe { dma.read_at::<Ac>(512) }.ctrl_type;
        let ltx = unsafe { dma.read_at::<Ac>(768) }.ctrl_type;
        
        controlq.free_desc(aqe);
        controlq.free_desc(jq);
        controlq.free_desc(vh);
        controlq.free_desc(aqd);
        
        if pcj != GpuCtrlType::RespOkNodata as u32 { return Err("TRANSFER failed"); }
        if ltx != GpuCtrlType::RespOkNodata as u32 { return Err("FLUSH failed"); }
        
        Ok(())
    }
    
    
    
    
    pub fn present_rect(&mut self, x: u32, y: u32, w: u32, h: u32) -> Result<(), &'static str> {
        let bvl = self.scanout_resource_id;
        if bvl == 0 { return Err("No scanout"); }
        
        
        let x = x.min(self.display_width);
        let y = y.min(self.display_height);
        let w = w.min(self.display_width.saturating_sub(x));
        let h = h.min(self.display_height.saturating_sub(y));
        if w == 0 || h == 0 { return Ok(()); }
        
        let dma = self.dma_buf.as_ref().ok_or("DMA not ready")?;
        let ali = dma.phys;
        
        
        let offset = ((y * self.display_width + x) as u64) * 4;
        
        let gzw = Jw {
            kp: Ac { ctrl_type: GpuCtrlType::CmdTransferToHost2d as u32, ..Default::default() },
            r: Eh { x, y, width: w, height: h },
            offset,
            zj: bvl,
            padding: 0,
        };
        unsafe { dma.write_at(0, &gzw); }
        
        let dpz = Jv {
            kp: Ac { ctrl_type: GpuCtrlType::CmdResourceFlush as u32, ..Default::default() },
            r: Eh { x, y, width: w, height: h },
            zj: bvl,
            padding: 0,
        };
        unsafe { dma.write_at(256, &dpz); }
        
        let gzy = core::mem::size_of::<Jw>() as u32;
        let fxg = core::mem::size_of::<Jv>() as u32;
        let cph = core::mem::size_of::<Ac>() as u32;
        
        let controlq = self.controlq.as_mut().ok_or("controlq not ready")?;
        let aqd = controlq.alloc_desc().ok_or("No free desc")?;
        let vh = controlq.alloc_desc().ok_or("No free desc")?;
        let jq = controlq.alloc_desc().ok_or("No free desc")?;
        let aqe = controlq.alloc_desc().ok_or("No free desc")?;
        
        controlq.set_desc(aqd, ali, gzy, RJ_, vh);
        controlq.set_desc(vh, ali + 512, cph, RK_, 0);
        controlq.set_desc(jq, ali + 256, fxg, RJ_, aqe);
        controlq.set_desc(aqe, ali + 768, cph, RK_, 0);
        
        controlq.submit(aqd);
        controlq.submit(jq);
        
        if let Some(bif) = &self.notify_cfg {
            bif.write16(0, 0);
        }
        
        let mut completed = 0u8;
        let mut mz = 5_000_000u32;
        while completed < 2 {
            if let Some(_) = controlq.poll_used() { completed += 1; }
            if completed < 2 {
                mz -= 1;
                if mz == 0 {
                    controlq.free_desc(aqe);
                    controlq.free_desc(jq);
                    controlq.free_desc(vh);
                    controlq.free_desc(aqd);
                    return Err("Rect present timeout");
                }
                core::hint::spin_loop();
            }
        }
        
        controlq.free_desc(aqe);
        controlq.free_desc(jq);
        controlq.free_desc(vh);
        controlq.free_desc(aqd);
        Ok(())
    }
    
    pub fn is_initialized(&self) -> bool { self.initialized }
    pub fn qkj(&self) -> bool { self.has_3d }
}





static Dg: Mutex<VirtioGpu> = Mutex::new(VirtioGpu::new());
static LA_: AtomicBool = AtomicBool::new(false);

pub fn igs() -> Result<(), &'static str> {
    for device in crate::pci::scan() {
        if device.vendor_id == DEO_ && device.device_id == DEN_ {
            crate::serial_println!("[VIRTIO-GPU] Found device at {:02x}:{:02x}.{}",
                device.bus, device.device, device.function);
            
            let mut gpu = Dg.lock();
            match gpu.init(device) {
                Ok(()) => {
                    match gpu.setup_scanout() {
                        Ok(()) => {
                            LA_.store(true, Ordering::SeqCst);
                            crate::serial_println!("[VIRTIO-GPU] ✓ Ready for rendering!");
                            
                            match gpu.setup_double_buffer() {
                                Ok(()) => crate::serial_println!("[VIRTIO-GPU] ✓ Double buffer enabled (tear-free)"),
                                Err(e) => crate::serial_println!("[VIRTIO-GPU] Double buffer skipped: {}", e),
                            }
                        }
                        Err(e) => crate::serial_println!("[VIRTIO-GPU] Scanout failed: {}", e),
                    }
                }
                Err(e) => crate::serial_println!("[VIRTIO-GPU] Init failed: {}", e),
            }
            return Ok(());
        }
    }
    crate::serial_println!("[VIRTIO-GPU] No VirtIO GPU found");
    Ok(())
}

pub fn sw() -> bool {
    LA_.load(Ordering::SeqCst)
}

pub fn create_surface(width: u32, height: u32) -> GpuSurface {
    GpuSurface::new(width, height)
}


pub fn qtu<F: FnOnce(&mut [u32], u32, u32)>(render_fn: F) -> Result<(), &'static str> {
    let mut gpu = Dg.lock();
    if !gpu.initialized { return Err("GPU not initialized"); }
    let (w, h) = (gpu.display_width, gpu.display_height);
    if let Some(buf) = gpu.backing_buffer.as_deref_mut() {
        render_fn(buf, w, h);
    }
    gpu.present()
}


pub fn ivv() -> Result<(), &'static str> {
    Dg.lock().present()
}



pub fn nww() -> Result<(), &'static str> {
    let mut gpu = Dg.lock();
    if !gpu.double_buffer_enabled {
        return gpu.present();
    }
    
    gpu.present()?;
    
    gpu.swap_gpu_buffers()
}


pub fn get_back_buffer() -> Option<(*mut u32, u32, u32)> {
    let mut gpu = Dg.lock();
    if !gpu.initialized { return None; }
    let (w, h) = (gpu.display_width, gpu.display_height);
    gpu.get_back_buffer().map(|buf| (buf.as_mut_ptr(), w, h))
}




pub fn nwv(rects: &[(u32, u32, u32, u32)]) {
    if rects.is_empty() { return; }
    
    
    let (fb_w, _fb_h) = crate::framebuffer::kv();
    if let Some((gpu_ptr, gpu_w, gpu_h)) = eod() {
        if let Some(mq) = crate::framebuffer::ibg() {
            let ut = (fb_w as usize).min(gpu_w as usize);
            let abw = gpu_h as usize;
            unsafe {
                for y in 0..abw {
                    let src = mq.add(y * fb_w as usize);
                    let dst = gpu_ptr.add(y * gpu_w as usize);
                    #[cfg(target_arch = "x86_64")]
                    crate::graphics::simd::blg(dst, src, ut);
                    #[cfg(not(target_arch = "x86_64"))]
                    core::ptr::copy_nonoverlapping(src, dst, ut);
                }
            }
        }
    }
    
    
    let mut gpu = Dg.lock();
    if !gpu.initialized { return; }
    let bvl = gpu.scanout_resource_id;
    if bvl == 0 { return; }
    
    for &(x, y, w, h) in rects {
        let _ = gpu.present_rect(x, y, w, h);
    }
}


pub fn eod() -> Option<(*mut u32, u32, u32)> {
    let mut gpu = Dg.lock();
    if !gpu.initialized { return None; }
    let (w, h) = (gpu.display_width, gpu.display_height);
    gpu.backing_buffer.as_deref_mut().map(|buf| (buf.as_mut_ptr(), w, h))
}


pub fn gcl() -> alloc::string::String {
    let gpu = Dg.lock();
    if gpu.initialized {
        alloc::format!("VirtIO GPU: {}x{} 2D (3D={})", gpu.display_width, gpu.display_height,
            if gpu.has_3d { "virgl" } else { "no" })
    } else {
        alloc::string::String::from("VirtIO GPU: not available")
    }
}


pub fn kcn(surface: &GpuSurface, x: u32, y: u32) {
    let (fb_w, fb_h) = crate::framebuffer::kv();
    crate::framebuffer::pr(true);
    for ak in 0..surface.height {
        let acv = y + ak;
        if acv >= fb_h { break; }
        for am in 0..surface.width {
            let tq = x + am;
            if tq >= fb_w { break; }
            crate::framebuffer::put_pixel(tq, acv, surface.get_pixel(am, ak));
        }
    }
    crate::framebuffer::ii();
}

pub fn qga() {
    if crate::framebuffer::ajy() {
        crate::framebuffer::ii();
    }
}

pub fn init() {
    crate::serial_println!("[GPU] Initializing graphics subsystem...");
    if let Err(e) = igs() {
        crate::serial_println!("[GPU] PCI init error: {}", e);
    }
    crate::framebuffer::adw();
    crate::framebuffer::pr(true);
    crate::serial_println!("[GPU] Graphics ready (VirtIO: {})", 
        if sw() { "ACTIVE" } else { "fallback" });
}











#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct Vt {
    pub capset_id: u32,
    pub capset_max_version: u32,
    pub capset_max_size: u32,
    pub padding: u32,
}


#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Zh {
    pub kp: Ac,
    pub dbt: u32,
    pub context_init: u32,
    pub debug_name: [u8; 64],
}


#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Zi {
    pub kp: Ac,
}


#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Awy {
    pub kp: Ac,
    pub zj: u32,
    pub target: u32,     
    pub format: u32,     
    pub fjf: u32,       
    pub width: u32,
    pub height: u32,
    pub depth: u32,
    pub array_size: u32,
    pub last_level: u32,
    pub nr_samples: u32,
    pub flags: u32,
    pub padding: u32,
}


#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Zn {
    pub kp: Ac,
    pub size: u32,
    pub padding: u32,
    
}


#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Awx {
    pub kp: Ac,
    pub zj: u32,
    pub padding: u32,
}


#[allow(dead_code)]
pub mod virgl_bind {
    pub const DMQ_: u32 = 1 << 0;
    pub const EFY_: u32 = 1 << 1;
    pub const EHK_: u32 = 1 << 3;
    pub const ENI_: u32 = 1 << 4;
    pub const DUR_: u32 = 1 << 5;
    pub const DKI_: u32 = 1 << 6;
    pub const EJY_: u32 = 1 << 14;
}


#[allow(dead_code)]
pub mod virgl_target {
    pub const Ata: u32 = 0;
    pub const ELA_: u32 = 1;
    pub const ELB_: u32 = 2;
    pub const ELC_: u32 = 3;
    pub const ELD_: u32 = 4;
}


pub struct Agc {
    ctx_id: u32,
    active: bool,
    capset_version: u32,
}

static ZK_: Mutex<Agc> = Mutex::new(Agc {
    ctx_id: 0,
    active: false,
    capset_version: 0,
});


pub fn qkt() -> bool {
    let gpu = Dg.lock();
    gpu.has_3d
}


pub fn qro() -> Option<Vt> {
    let mut gpu = Dg.lock();
    if !gpu.has_3d || !gpu.initialized { return None; }
    
    let dma = gpu.dma_buf.as_ref()?;
    
    
    let cmd = Ac {
        ctrl_type: GpuCtrlType::CmdGetCapsetInfo as u32,
        ..Default::default()
    };
    unsafe { dma.write_at(0, &cmd); }
    
    unsafe { (dma.virt.add(core::mem::size_of::<Ac>()) as *mut u32).write_volatile(0); }
    
    let fnd = core::mem::size_of::<Ac>() as u32 + 4;
    let cph = core::mem::size_of::<Ac>() as u32 + core::mem::size_of::<Vt>() as u32;
    
    let ddm = gpu.send_command(fnd, 512, cph).ok()?;
    if ddm != GpuCtrlType::RespOkCapsetInfo as u32 { return None; }
    
    let dma = gpu.dma_buf.as_ref()?;
    let info: Vt = unsafe { dma.read_at(512 + core::mem::size_of::<Ac>()) };
    crate::serial_println!("[VIRGL] Capset: id={}, max_version={}, max_size={}", 
        info.capset_id, info.capset_max_version, info.capset_max_size);
    Some(info)
}


pub fn qca(name: &str) -> Result<u32, &'static str> {
    let mut gpu = Dg.lock();
    if !gpu.has_3d { return Err("No 3D support"); }
    if !gpu.initialized { return Err("GPU not initialized"); }
    
    let ctx_id = 1u32; 
    
    let mut cmd = Zh {
        kp: Ac {
            ctrl_type: GpuCtrlType::CmdCtxCreate as u32,
            ctx_id,
            ..Default::default()
        },
        dbt: name.len().min(63) as u32,
        context_init: 0,
        debug_name: [0u8; 64],
    };
    
    let agt = name.as_bytes();
    let mb = agt.len().min(63);
    cmd.debug_name[..mb].copy_from_slice(&agt[..mb]);
    
    let dma = gpu.dma_buf.as_ref().ok_or("DMA not ready")?;
    unsafe { dma.write_at(0, &cmd); }
    
    let eo = gpu.send_command(
        core::mem::size_of::<Zh>() as u32,
        512,
        core::mem::size_of::<Ac>() as u32,
    )?;
    
    if eo != GpuCtrlType::RespOkNodata as u32 {
        return Err("CTX_CREATE failed");
    }
    
    let mut bae = ZK_.lock();
    bae.ctx_id = ctx_id;
    bae.active = true;
    
    crate::serial_println!("[VIRGL] 3D context created: id={} name={}", ctx_id, name);
    Ok(ctx_id)
}


pub fn qcu() -> Result<(), &'static str> {
    let mut bae = ZK_.lock();
    if !bae.active { return Ok(()); }
    
    let mut gpu = Dg.lock();
    let cmd = Zi {
        kp: Ac {
            ctrl_type: GpuCtrlType::CmdCtxDestroy as u32,
            ctx_id: bae.ctx_id,
            ..Default::default()
        },
    };
    
    let dma = gpu.dma_buf.as_ref().ok_or("DMA not ready")?;
    unsafe { dma.write_at(0, &cmd); }
    
    let eo = gpu.send_command(
        core::mem::size_of::<Zi>() as u32,
        512,
        core::mem::size_of::<Ac>() as u32,
    )?;
    
    if eo != GpuCtrlType::RespOkNodata as u32 {
        return Err("CTX_DESTROY failed");
    }
    
    bae.active = false;
    crate::serial_println!("[VIRGL] 3D context destroyed");
    Ok(())
}


pub fn qxt(commands: &[u8]) -> Result<(), &'static str> {
    let bae = ZK_.lock();
    if !bae.active { return Err("No active 3D context"); }
    let ctx_id = bae.ctx_id;
    drop(bae);
    
    let mut gpu = Dg.lock();
    if !gpu.initialized { return Err("GPU not initialized"); }
    
    if commands.len() > 3800 { return Err("Command buffer too large"); }
    
    let dma = gpu.dma_buf.as_ref().ok_or("DMA not ready")?;
    
    let cmd = Zn {
        kp: Ac {
            ctrl_type: GpuCtrlType::CmdSubmit3d as u32,
            ctx_id,
            ..Default::default()
        },
        size: commands.len() as u32,
        padding: 0,
    };
    unsafe { dma.write_at(0, &cmd); }
    
    
    let hly = core::mem::size_of::<Zn>();
    unsafe {
        core::ptr::copy_nonoverlapping(
            commands.as_ptr(),
            dma.virt.add(hly),
            commands.len(),
        );
    }
    
    let plr = (hly + commands.len()) as u32;
    let eo = gpu.send_command(plr, 512, core::mem::size_of::<Ac>() as u32)?;
    
    if eo != GpuCtrlType::RespOkNodata as u32 {
        return Err("SUBMIT_3D failed");
    }
    Ok(())
}


pub fn rbz() -> alloc::string::String {
    let gpu = Dg.lock();
    let bae = ZK_.lock();
    if gpu.has_3d {
        alloc::format!("VIRGL: {} (ctx={})", 
            if bae.active { "active" } else { "ready" },
            bae.ctx_id)
    } else {
        alloc::string::String::from("VIRGL: not available")
    }
}





pub struct Layer {
    pub surface: GpuSurface,
    pub x: i32,
    pub y: i32,
    pub z_order: i32,
    pub visible: bool,
    pub opacity: u8,
}

pub struct Compositor {
    layers: Vec<Layer>,
    output: GpuSurface,
    background_color: u32,
}

impl Compositor {
    pub fn new(width: u32, height: u32) -> Self {
        Self { layers: Vec::new(), output: GpuSurface::new(width, height), background_color: 0xFF1A1A1A }
    }
    pub fn add_layer(&mut self, surface: GpuSurface, x: i32, y: i32, z_order: i32) -> usize {
        let idx = self.layers.len();
        self.layers.push(Layer { surface, x, y, z_order, visible: true, opacity: 255 });
        self.layers.sort_by_key(|l| l.z_order);
        idx
    }
    pub fn oez(&mut self, index: usize) {
        if index < self.layers.len() { self.layers.remove(index); }
    }
    pub fn compose(&mut self) {
        self.output.clear(self.background_color);
        for bj in &self.layers {
            if bj.visible { self.output.blit(&bj.surface, bj.x, bj.y); }
        }
    }
    pub fn render(&self) { kcn(&self.output, 0, 0); }
    pub fn get_layer(&self, index: usize) -> Option<&Layer> { self.layers.get(index) }
    pub fn get_layer_mut(&mut self, index: usize) -> Option<&mut Layer> { self.layers.get_mut(index) }
    pub fn ooo(&mut self, color: u32) { self.background_color = color; }
}
