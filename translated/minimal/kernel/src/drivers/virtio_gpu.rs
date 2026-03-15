











use alloc::vec::Vec;
use alloc::vec;
use alloc::boxed::Box;
use core::sync::atomic::{AtomicBool, Ordering};
use spin::Mutex;

use crate::pci::{self, S};
use crate::memory;






pub const DAV_: u16 = 0x1050;

pub const DAW_: u16 = 0x1AF4;


pub mod virtio_cap {
    pub const AOU_: u8 = 1;
    pub const BBO_: u8 = 2;
    pub const AXK_: u8 = 3;
    pub const AQA_: u8 = 4;
    pub const ELL_: u8 = 5;
}


pub mod dev_status {
    pub const Or: u8 = 1;
    pub const Fl: u8 = 2;
    pub const HW_: u8 = 4;
    pub const MZ_: u8 = 8;
    pub const ELE_: u8 = 64;
    pub const Arw: u8 = 128;
}


pub mod features {
    pub const DBW_: u64 = 1 << 0;
    pub const BIL_: u64 = 1 << 1;
    pub const ELP_: u64 = 1 << 2;
    pub const ELO_: u64 = 1 << 3;
    pub const DAU_: u64 = 1 << 32;
}


#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq)]
#[allow(bgr)]
pub enum GpuCtrlType {
    
    Bzs = 0x0100,
    Bzu = 0x0101,
    Ctg = 0x0102,
    Bzv = 0x0103,
    Aqa = 0x0104,
    Aqb = 0x0105,
    Bzt = 0x0106,
    Ctf = 0x0107,
    Bzr = 0x0108,
    Ctb = 0x0109,
    Ctc = 0x010a,

    
    Bzp = 0x0200,
    Bzq = 0x0201,
    Csz = 0x0202,
    Cta = 0x0203,
    Cte = 0x0204,
    Cti = 0x0205,
    Cth = 0x0206,
    Bzw = 0x0207,

    
    Ctj = 0x0300,
    Ctd = 0x0301,

    
    Mg = 0x1100,
    Ckq = 0x1101,
    Ckp = 0x1102,
    Dgd = 0x1103,
    Dge = 0x1104,

    
    Dgc = 0x1200,
    Dgb = 0x1201,
    Dga = 0x1202,
    Dfz = 0x1203,
    Dfx = 0x1204,
    Dfy = 0x1205,
}


#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq)]
#[allow(bgr)]
pub enum GpuFormat {
    Crp = 1,
    Byc = 2,
    Cqw = 3,
    Dmc = 4,
    Dfb = 67,
    Dmb = 68,
    Cqv = 121,
    Dfc = 134,
}






#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct Ar {
    pub btv: u32,
    pub flags: u32,
    pub yqj: u64,
    pub cjv: u32,
    pub jml: u8,
    pub ob: [u8; 3],
}


#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct Ko {
    pub b: u32,
    pub c: u32,
    pub z: u32,
    pub ac: u32,
}


#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct Ceo {
    pub m: Ko,
    pub iq: u32,
    pub flags: u32,
}


#[repr(C)]
#[derive(Clone, Copy)]
pub struct Bif {
    pub zj: Ar,
    pub vjp: [Ceo; 16],
}


#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Bie {
    pub zj: Ar,
    pub awu: u32,
    pub format: u32,
    pub z: u32,
    pub ac: u32,
}


#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Big {
    pub zj: Ar,
    pub m: Ko,
    pub mcj: u32,
    pub awu: u32,
}


#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Wv {
    pub zj: Ar,
    pub m: Ko,
    pub awu: u32,
    pub ob: u32,
}


#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Ww {
    pub zj: Ar,
    pub m: Ko,
    pub l: u64,
    pub awu: u32,
    pub ob: u32,
}


#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Bid {
    pub ag: u64,
    pub go: u32,
    pub ob: u32,
}


#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Atf {
    pub zj: Ar,
    pub awu: u32,
    pub uvv: u32,
}





#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
struct VirtqDesc {
    ag: u64,
    len: u32,
    flags: u16,
    next: u16,
}

const QP_: u16 = 1;
const QQ_: u16 = 2;

#[repr(C)]
struct Zk {
    flags: u16,
    w: u16,
}

#[repr(C)]
#[derive(Clone, Copy, Default)]
struct Aob {
    ad: u32,
    len: u32,
}

#[repr(C)]
struct Zl {
    flags: u16,
    w: u16,
}


struct GpuVirtqueue {
    aw: u16,
    iih: u64,
    qea: *mut u8,
    desc: *mut VirtqDesc,
    apk: *mut Zk,
    mr: *mut Zl,
    cyb: u16,
    dts: u16,
    buk: Vec<u16>,
    csa: u16,
}

unsafe impl Send for GpuVirtqueue {}
unsafe impl Sync for GpuVirtqueue {}

impl GpuVirtqueue {
    fn new(aw: u16) -> Result<Self, &'static str> {
        use alloc::alloc::{alloc_zeroed, Layout};
        
        let gen = core::mem::size_of::<VirtqDesc>() * aw as usize;
        let kbm = 6 + 2 * aw as usize;
        let dxe = ((gen + kbm) + 4095) & !4095;
        let xpq = 6 + core::mem::size_of::<Aob>() * aw as usize;
        let aay = dxe + xpq + 4096;
        
        let layout = Layout::bjy(aay, 4096)
            .jd(|_| "Invalid virtqueue layout")?;
        let ptr = unsafe { alloc_zeroed(layout) };
        if ptr.abq() { return Err("Failed to allocate virtqueue"); }
        
        let vd = ptr as u64;
        let hp = memory::lr();
        let ki = if vd >= hp { vd - hp } else { vd };
        
        let desc = ptr as *mut VirtqDesc;
        let apk = unsafe { ptr.add(gen) as *mut Zk };
        let mr = unsafe { ptr.add(dxe) as *mut Zl };
        
        let mut buk = vec![0u16; aw as usize];
        for a in 0..(aw as usize).ao(1) {
            buk[a] = (a + 1) as u16;
        }
        if aw > 0 { buk[aw as usize - 1] = 0xFFFF; }
        
        Ok(Self {
            aw,
            iih: ki,
            qea: ptr,
            desc,
            apk,
            mr,
            cyb: 0,
            dts: aw,
            buk,
            csa: 0,
        })
    }
    
    fn blx(&mut self) -> Option<u16> {
        if self.dts == 0 { return None; }
        let w = self.cyb;
        self.cyb = self.buk[w as usize];
        self.dts -= 1;
        Some(w)
    }
    
    fn ald(&mut self, w: u16) {
        self.buk[w as usize] = self.cyb;
        self.cyb = w;
        self.dts += 1;
    }
    
    fn bwz(&mut self, w: u16, ag: u64, len: u32, flags: u16, next: u16) {
        unsafe {
            let bc = &mut *self.desc.add(w as usize);
            bc.ag = ag;
            bc.len = len;
            bc.flags = flags;
            bc.next = next;
        }
    }
    
    fn dmd(&mut self, ale: u16) {
        unsafe {
            let apk = &mut *self.apk;
            let fsy = (self.apk as *mut u8).add(4) as *mut u16;
            let w = apk.w;
            *fsy.add((w % self.aw) as usize) = ale;
            core::sync::atomic::cxt(Ordering::Release);
            apk.w = w.cn(1);
        }
    }
    
    fn lur(&mut self) -> Option<(u32, u32)> {
        unsafe {
            core::sync::atomic::cxt(Ordering::Acquire);
            let mr = &*self.mr;
            if mr.w == self.csa { return None; }
            let fsy = (self.mr as *mut u8).add(4) as *mut Aob;
            let fhm = *fsy.add((self.csa % self.aw) as usize);
            self.csa = self.csa.cn(1);
            Some((fhm.ad, fhm.len))
        }
    }
    
    fn rwb(&self) -> u64 { self.iih }
    fn qlp(&self) -> u64 {
        let gen = core::mem::size_of::<VirtqDesc>() * self.aw as usize;
        self.iih + gen as u64
    }
    fn xps(&self) -> u64 {
        let gen = core::mem::size_of::<VirtqDesc>() * self.aw as usize;
        let kbm = 6 + 2 * self.aw as usize;
        let dxe = ((gen + kbm) + 4095) & !4095;
        self.iih + dxe as u64
    }
}





struct No {
    ar: *mut u8,
    jxx: u32,
}

unsafe impl Send for No {}
unsafe impl Sync for No {}

impl No {
    fn akm(&self, l: u32) -> u8 {
        unsafe { core::ptr::read_volatile(self.ar.add(l as usize)) }
    }
    fn aym(&self, l: u32) -> u16 {
        unsafe { core::ptr::read_volatile(self.ar.add(l as usize) as *const u16) }
    }
    fn amp(&self, l: u32) -> u32 {
        unsafe { core::ptr::read_volatile(self.ar.add(l as usize) as *const u32) }
    }
    fn akw(&self, l: u32, ap: u8) {
        unsafe { core::ptr::write_volatile(self.ar.add(l as usize), ap) }
    }
    fn asg(&self, l: u32, ap: u16) {
        unsafe { core::ptr::write_volatile(self.ar.add(l as usize) as *mut u16, ap) }
    }
    fn aiu(&self, l: u32, ap: u32) {
        unsafe { core::ptr::write_volatile(self.ar.add(l as usize) as *mut u32, ap) }
    }
    fn jxe(&self, l: u32, ap: u64) {
        self.aiu(l, ap as u32);
        self.aiu(l + 4, (ap >> 32) as u32);
    }
}


mod common_cfg {
    pub const AQC_: u32 = 0x00;
    pub const AQB_: u32 = 0x04;
    pub const AQR_: u32 = 0x08;
    pub const AQQ_: u32 = 0x0C;
    pub const ELJ_: u32 = 0x10;
    pub const ELK_: u32 = 0x12;
    pub const EF_: u32 = 0x14;
    pub const ELC_: u32 = 0x15;
    pub const AGU_: u32 = 0x16;
    pub const WM_: u32 = 0x18;
    pub const CNA_: u32 = 0x1A;
    pub const CMZ_: u32 = 0x1C;
    pub const CNC_: u32 = 0x1E;
    pub const CMW_: u32 = 0x20;
    pub const CMY_: u32 = 0x28;
    pub const CMX_: u32 = 0x30;
}


mod gpu_cfg {
    pub const ELI_: u32 = 0x00;
    pub const ELH_: u32 = 0x04;
    pub const CID_: u32 = 0x08;
    pub const CIA_: u32 = 0x0C;
}





struct DmaCommandBuffer {
    ht: u64,
    ju: *mut u8,
    dds: usize,
}

unsafe impl Send for DmaCommandBuffer {}
unsafe impl Sync for DmaCommandBuffer {}

impl DmaCommandBuffer {
    fn new(aw: usize) -> Result<Self, &'static str> {
        use alloc::alloc::{alloc_zeroed, Layout};
        let layout = Layout::bjy(aw, 4096)
            .jd(|_| "DMA buffer layout error")?;
        let ptr = unsafe { alloc_zeroed(layout) };
        if ptr.abq() { return Err("DMA buffer allocation failed"); }
        let ju = ptr as u64;
        let hp = memory::lr();
        let ht = if ju >= hp { ju - hp } else { ju };
        Ok(Self { ht, ju: ptr, dds: aw })
    }
    
    unsafe fn ciu<T: Copy>(&self, l: usize, ap: &T) {
        core::ptr::write_volatile(self.ju.add(l) as *mut T, *ap);
    }
    
    unsafe fn read_at<T: Copy>(&self, l: usize) -> T {
        core::ptr::read_volatile(self.ju.add(l) as *const T)
    }
    
    fn zfa(&self, l: usize) -> u64 { self.ht + l as u64 }
}






pub struct GpuSurface {
    pub awu: u32,
    pub z: u32,
    pub ac: u32,
    pub f: Box<[u32]>,
}

impl GpuSurface {
    pub fn new(z: u32, ac: u32) -> Self {
        let aw = (z * ac) as usize;
        Self {
            awu: 0,
            z,
            ac,
            f: alloc::vec![0u32; aw].dsd(),
        }
    }
    
    pub fn clear(&mut self, s: u32) { self.f.vi(s); }
    
    #[inline]
    pub fn aht(&mut self, b: u32, c: u32, s: u32) {
        if b < self.z && c < self.ac {
            self.f[(c * self.z + b) as usize] = s;
        }
    }
    
    #[inline]
    pub fn beg(&self, b: u32, c: u32) -> u32 {
        if b < self.z && c < self.ac {
            self.f[(c * self.z + b) as usize]
        } else { 0 }
    }
    
    pub fn ah(&mut self, b: u32, c: u32, d: u32, i: u32, s: u32) {
        let dn = b.v(self.z);
        let dp = c.v(self.ac);
        let hy = (b + d).v(self.z);
        let jz = (c + i).v(self.ac);
        for x in dp..jz {
            let ay = (x * self.z + dn) as usize;
            let ci = (x * self.z + hy) as usize;
            self.f[ay..ci].vi(s);
        }
    }
    
    pub fn bge(&mut self, cy: &GpuSurface, buc: i32, bqg: i32) {
        for cq in 0..cy.ac {
            for cr in 0..cy.z {
                let dx = buc + cr as i32;
                let bg = bqg + cq as i32;
                if dx >= 0 && bg >= 0 && dx < self.z as i32 && bg < self.ac as i32 {
                    let il = cy.beg(cr, cq);
                    let dw = (il >> 24) & 0xFF;
                    if dw >= 128 {
                        self.aht(dx as u32, bg as u32, il);
                    }
                }
            }
        }
    }
    
    fn dvq(&mut self, b: i32, c: i32, s: u32) {
        if b >= 0 && c >= 0 && b < self.z as i32 && c < self.ac as i32 {
            self.aht(b as u32, c as u32, s);
        }
    }

    pub fn ahj(&mut self, fy: i32, fo: i32, dn: i32, dp: i32, s: u32) {
        let dx = (dn - fy).gp();
        let bg = -(dp - fo).gp();
        let cr = if fy < dn { 1 } else { -1 };
        let cq = if fo < dp { 1 } else { -1 };
        let mut rq = dx + bg;
        let (mut b, mut c) = (fy, fo);
        loop {
            self.dvq(b, c, s);
            if b == dn && c == dp { break; }
            let agl = 2 * rq;
            if agl >= bg { rq += bg; b += cr; }
            if agl <= dx { rq += dx; c += cq; }
        }
    }

    pub fn lx(&mut self, b: u32, c: u32, d: u32, i: u32, s: u32) {
        let (b, c, d, i) = (b as i32, c as i32, d as i32, i as i32);
        self.ahj(b, c, b+d-1, c, s);
        self.ahj(b, c+i-1, b+d-1, c+i-1, s);
        self.ahj(b, c, b, c+i-1, s);
        self.ahj(b+d-1, c, b+d-1, c+i-1, s);
    }

    pub fn cxc(&mut self, cx: i32, ae: i32, dy: i32, s: u32) {
        let (mut b, mut c, mut rq) = (dy, 0i32, 0i32);
        while b >= c {
            self.dvq(cx+b, ae+c, s);
            self.dvq(cx+c, ae+b, s);
            self.dvq(cx-c, ae+b, s);
            self.dvq(cx-b, ae+c, s);
            self.dvq(cx-b, ae-c, s);
            self.dvq(cx-c, ae-b, s);
            self.dvq(cx+c, ae-b, s);
            self.dvq(cx+b, ae-c, s);
            c += 1;
            rq += 1 + 2*c;
            if 2*(rq-b)+1 > 0 { b -= 1; rq += 1 - 2*b; }
        }
    }

    pub fn abc(&mut self, cx: i32, ae: i32, dy: i32, s: u32) {
        for bg in -dy..=dy {
            for dx in -dy..=dy {
                if dx*dx + bg*bg <= dy*dy {
                    self.dvq(cx+dx, ae+bg, s);
                }
            }
        }
    }

    pub fn mf(&mut self, b: u32, c: u32, d: u32, i: u32, jyj: u32, s: u32) {
        self.lx(b, c, d, i, s);
    }

    pub fn afp(&mut self, b: u32, c: u32, d: u32, i: u32, jyj: u32, s: u32) {
        self.ah(b, c, d, i, s);
    }

    pub fn ygn(&mut self, cy: &GpuSurface, buc: i32, bqg: i32, krx: u32, kru: u32) {
        if krx == 0 || kru == 0 || cy.z == 0 || cy.ac == 0 { return; }
        for bg in 0..kru {
            for dx in 0..krx {
                let cr = (dx * cy.z) / krx;
                let cq = (bg * cy.ac) / kru;
                let y = buc + dx as i32;
                let x = bqg + bg as i32;
                if y >= 0 && x >= 0 && y < self.z as i32 && x < self.ac as i32 {
                    self.aht(y as u32, x as u32, cy.beg(cr, cq));
                }
            }
        }
    }
}





pub struct VirtioGpu {
    mst: Option<S>,
    common_cfg: Option<No>,
    fpe: Option<No>,
    msk: Option<No>,
    ira: Option<No>,
    msq: u32,
    wy: Option<GpuVirtqueue>,
    alb: Option<DmaCommandBuffer>,
    bjv: u32,
    bqf: u32,
    lpm: u32,
    loo: u32,
    eyb: u32,
    fcx: Option<Box<[u32]>>,
    mxo: u64,
    jr: bool,
    ecl: bool,
    
    kbs: u32,
    aqt: Option<Box<[u32]>>,
    mxk: u64,
    hgq: bool,
    ghp: bool, 
}

impl VirtioGpu {
    pub const fn new() -> Self {
        Self {
            mst: None,
            common_cfg: None,
            fpe: None,
            msk: None,
            ira: None,
            msq: 0,
            wy: None,
            alb: None,
            bjv: 0,
            bqf: 0,
            lpm: 0,
            loo: 1,
            eyb: 0,
            fcx: None,
            mxo: 0,
            jr: false,
            ecl: false,
            kbs: 0,
            aqt: None,
            mxk: 0,
            hgq: false,
            ghp: true,
        }
    }
    
    fn jev(ba: &S, kcc: u8, l: u32, go: u32) -> Result<No, &'static str> {
        let kbx = ba.cje(kcc as usize)
            .ok_or("BAR not configured")?;
        if !ba.mxx(kcc as usize) {
            return Err("Expected memory BAR, got I/O");
        }
        let ht = kbx + l as u64;
        let ju = memory::bki(ht, go.am(4096) as usize)?;
        crate::serial_println!("[VIRTIO-GPU] Mapped BAR{}: phys={:#X} virt={:#X} len={}", 
            kcc, ht, ju, go);
        Ok(No { ar: ju as *mut u8, jxx: go })
    }
    
    
    pub fn init(&mut self, ba: S) -> Result<(), &'static str> {
        crate::serial_println!("[VIRTIO-GPU] === Initializing VirtIO GPU ===");
        crate::serial_println!("[VIRTIO-GPU] PCI {:02X}:{:02X}.{} vid={:#06X} did={:#06X}",
            ba.aq, ba.de, ba.gw, ba.ml, ba.mx);
        
        pci::fhp(&ba);
        pci::fhq(&ba);
        
        
        let dr = pci::stp(&ba);
        if dr.is_empty() {
            return Err("No VirtIO capabilities found");
        }
        
        crate::serial_println!("[VIRTIO-GPU] Found {} VirtIO capabilities", dr.len());
        
        let mut loy: u8 = 0;
        
        for &(aqv, ind, bar, l, go) in &dr {
            let j = match ind {
                1 => "COMMON", 2 => "NOTIFY", 3 => "ISR", 4 => "DEVICE", 5 => "PCI", _ => "?",
            };
            crate::serial_println!("[VIRTIO-GPU]   cap@{:#X}: {} BAR{} off={:#X} len={}", 
                aqv, j, bar, l, go);
            
            match ind {
                virtio_cap::AOU_ => {
                    self.common_cfg = Some(Self::jev(&ba, bar, l, go)?);
                }
                virtio_cap::BBO_ => {
                    self.fpe = Some(Self::jev(&ba, bar, l, go)?);
                    loy = aqv;
                }
                virtio_cap::AXK_ => {
                    self.msk = Some(Self::jev(&ba, bar, l, go)?);
                }
                virtio_cap::AQA_ => {
                    self.ira = Some(Self::jev(&ba, bar, l, go)?);
                }
                _ => {}
            }
        }
        
        
        if self.common_cfg.is_none() { return Err("Missing COMMON_CFG"); }
        if self.fpe.is_none() { return Err("Missing NOTIFY_CFG"); }
        if self.ira.is_none() { return Err("Missing DEVICE_CFG"); }
        
        if loy > 0 {
            self.msq = pci::vsd(&ba, loy);
        }
        
        
        
        
        
        self.gdc(common_cfg::EF_, 0);
        for _ in 0..10000 { core::hint::hc(); }
        
        
        self.gdc(common_cfg::EF_, dev_status::Or);
        
        
        self.gdc(common_cfg::EF_, dev_status::Or | dev_status::Fl);
        
        
        self.gdb(common_cfg::AQC_, 0);
        let sro = self.nfc(common_cfg::AQB_);
        self.gdb(common_cfg::AQC_, 1);
        let srn = self.nfc(common_cfg::AQB_);
        let bju = (sro as u64) | ((srn as u64) << 32);
        
        crate::serial_println!("[VIRTIO-GPU] Device features: {:#018X}", bju);
        self.ecl = bju & features::DBW_ != 0;
        
        let mut ckb = features::DAU_;
        if bju & features::BIL_ != 0 {
            ckb |= features::BIL_;
        }
        
        self.gdb(common_cfg::AQR_, 0);
        self.gdb(common_cfg::AQQ_, ckb as u32);
        self.gdb(common_cfg::AQR_, 1);
        self.gdb(common_cfg::AQQ_, (ckb >> 32) as u32);
        
        
        self.gdc(common_cfg::EF_,
            dev_status::Or | dev_status::Fl | dev_status::MZ_);
        
        let status = self.rmu(common_cfg::EF_);
        if status & dev_status::MZ_ == 0 {
            self.gdc(common_cfg::EF_, dev_status::Arw);
            return Err("Device rejected features");
        }
        crate::serial_println!("[VIRTIO-GPU] Features OK (3D={})", self.ecl);
        
        
        self.wkk()?;
        
        
        self.gdc(common_cfg::EF_,
            dev_status::Or | dev_status::Fl | dev_status::MZ_ | dev_status::HW_);
        crate::serial_println!("[VIRTIO-GPU] DRIVER_OK set");
        
        
        self.alb = Some(DmaCommandBuffer::new(8192)?);
        
        
        self.lpm = self.nlc(gpu_cfg::CID_);
        let uwe = self.nlc(gpu_cfg::CIA_);
        crate::serial_println!("[VIRTIO-GPU] scanouts={} capsets={}", self.lpm, uwe);
        
        
        self.tdk()?;
        
        self.mst = Some(ba);
        self.jr = true;
        
        crate::serial_println!("[VIRTIO-GPU] === Init complete: {}x{} ===", 
            self.bjv, self.bqf);
        Ok(())
    }
    
    
    fn gdc(&self, l: u32, ap: u8) {
        if let Some(r) = &self.common_cfg { r.akw(l, ap); }
    }
    fn yjq(&self, l: u32, ap: u16) {
        if let Some(r) = &self.common_cfg { r.asg(l, ap); }
    }
    fn gdb(&self, l: u32, ap: u32) {
        if let Some(r) = &self.common_cfg { r.aiu(l, ap); }
    }
    fn rmu(&self, l: u32) -> u8 {
        self.common_cfg.as_ref().map(|r| r.akm(l)).unwrap_or(0)
    }
    fn yjp(&self, l: u32) -> u16 {
        self.common_cfg.as_ref().map(|r| r.aym(l)).unwrap_or(0)
    }
    fn nfc(&self, l: u32) -> u32 {
        self.common_cfg.as_ref().map(|r| r.amp(l)).unwrap_or(0)
    }
    fn nlc(&self, l: u32) -> u32 {
        self.ira.as_ref().map(|r| r.amp(l)).unwrap_or(0)
    }
    
    fn wkk(&mut self) -> Result<(), &'static str> {
        let dzq = self.common_cfg.as_ref().ok_or("Missing COMMON_CFG")?;
        dzq.asg(common_cfg::AGU_, 0);
        let ate = dzq.aym(common_cfg::WM_);
        crate::serial_println!("[VIRTIO-GPU] controlq max_size={}", ate);
        if ate == 0 { return Err("controlq not available"); }
        
        let art = ate.v(64);
        dzq.asg(common_cfg::WM_, art);
        
        let jwa = GpuVirtqueue::new(art)?;
        
        dzq.jxe(common_cfg::CMW_, jwa.rwb());
        dzq.jxe(common_cfg::CMY_, jwa.qlp());
        dzq.jxe(common_cfg::CMX_, jwa.xps());
        dzq.asg(common_cfg::CNA_, 0xFFFF);
        dzq.asg(common_cfg::CMZ_, 1);
        
        let ybm = dzq.aym(common_cfg::CNC_);
        
        self.wy = Some(jwa);
        crate::serial_println!("[VIRTIO-GPU] controlq ready (size={})", art);
        Ok(())
    }
    
    fn zdy(&self) {
        if let Some(djy) = &self.fpe {
            djy.asg(0, 0);
        }
    }
    
    
    fn dvn(&mut self, rfu: u32, pct: usize, vxz: u32) -> Result<u32, &'static str> {
        
        let nme = self.alb.as_ref().ok_or("DMA not ready")?.ht;
        
        let wy = self.wy.as_mut().ok_or("controlq not ready")?;
        
        let iqj = wy.blx().ok_or("No free desc (cmd)")?;
        let iql = wy.blx().ok_or("No free desc (resp)")?;
        
        wy.bwz(iqj, nme, rfu, QP_, iql);
        wy.bwz(iql, nme + pct as u64, vxz, QQ_, 0);
        
        wy.dmd(iqj);
        
        if let Some(djy) = &self.fpe {
            djy.asg(0, 0);
        }
        
        let mut aah = 5_000_000u32;
        loop {
            if let Some(_) = wy.lur() { break; }
            aah -= 1;
            if aah == 0 {
                wy.ald(iql);
                wy.ald(iqj);
                return Err("Command timeout");
            }
            core::hint::hc();
        }
        
        let dma = self.alb.as_ref().unwrap();
        let gqw = unsafe { dma.read_at::<Ar>(pct) }.btv;
        wy.ald(iql);
        wy.ald(iqj);
        Ok(gqw)
    }
    
    fn tdk(&mut self) -> Result<(), &'static str> {
        let dma = self.alb.as_ref().ok_or("DMA not ready")?;
        
        let cmd = Ar {
            btv: GpuCtrlType::Bzs as u32,
            ..Default::default()
        };
        unsafe { dma.ciu(0, &cmd); }
        
        let gqw = self.dvn(
            core::mem::size_of::<Ar>() as u32,
            512, 
            core::mem::size_of::<Bif>() as u32,
        )?;
        
        if gqw != GpuCtrlType::Ckq as u32 {
            crate::serial_println!("[VIRTIO-GPU] GET_DISPLAY_INFO failed: {:#X}", gqw);
            return Err("GET_DISPLAY_INFO failed");
        }
        
        let dma = self.alb.as_ref().unwrap();
        let lj: Bif = unsafe { dma.read_at(512) };
        
        for (a, hvj) in lj.vjp.iter().cf() {
            if hvj.iq != 0 {
                self.bjv = hvj.m.z;
                self.bqf = hvj.m.ac;
                crate::serial_println!("[VIRTIO-GPU] Display {}: {}x{}", a, hvj.m.z, hvj.m.ac);
                break;
            }
        }
        
        if self.bjv == 0 {
            self.bjv = 1280;
            self.bqf = 800;
            crate::serial_println!("[VIRTIO-GPU] Defaulting to {}x{}", self.bjv, self.bqf);
        }
        Ok(())
    }
    
    pub fn nhg(&mut self, z: u32, ac: u32) -> Result<u32, &'static str> {
        let ad = self.loo;
        self.loo += 1;
        let dma = self.alb.as_ref().ok_or("DMA not ready")?;
        
        let cmd = Bie {
            zj: Ar { btv: GpuCtrlType::Bzu as u32, ..Default::default() },
            awu: ad,
            format: GpuFormat::Byc as u32,
            z,
            ac,
        };
        unsafe { dma.ciu(0, &cmd); }
        
        let lj = self.dvn(
            core::mem::size_of::<Bie>() as u32,
            512, core::mem::size_of::<Ar>() as u32,
        )?;
        
        if lj != GpuCtrlType::Mg as u32 {
            return Err("RESOURCE_CREATE_2D failed");
        }
        crate::serial_println!("[VIRTIO-GPU] Resource {} created ({}x{})", ad, z, ac);
        Ok(ad)
    }
    
    pub fn mwm(&mut self, awu: u32, rg: u64, bjl: u32) -> Result<(), &'static str> {
        let dma = self.alb.as_ref().ok_or("DMA not ready")?;
        
        let cmd = Atf {
            zj: Ar { btv: GpuCtrlType::Bzt as u32, ..Default::default() },
            awu,
            uvv: 1,
        };
        unsafe { dma.ciu(0, &cmd); }
        
        let bt = Bid { ag: rg, go: bjl, ob: 0 };
        unsafe { dma.ciu(core::mem::size_of::<Atf>(), &bt); }
        
        let kjf = (core::mem::size_of::<Atf>() + core::mem::size_of::<Bid>()) as u32;
        let lj = self.dvn(kjf, 512, core::mem::size_of::<Ar>() as u32)?;
        
        if lj != GpuCtrlType::Mg as u32 {
            return Err("ATTACH_BACKING failed");
        }
        crate::serial_println!("[VIRTIO-GPU] Backing attached: phys={:#X} len={}", rg, bjl);
        Ok(())
    }
    
    pub fn mew(&mut self, mcj: u32, awu: u32, d: u32, i: u32) -> Result<(), &'static str> {
        let dma = self.alb.as_ref().ok_or("DMA not ready")?;
        let cmd = Big {
            zj: Ar { btv: GpuCtrlType::Bzv as u32, ..Default::default() },
            m: Ko { b: 0, c: 0, z: d, ac: i },
            mcj,
            awu,
        };
        unsafe { dma.ciu(0, &cmd); }
        
        let lj = self.dvn(
            core::mem::size_of::<Big>() as u32,
            512, core::mem::size_of::<Ar>() as u32,
        )?;
        
        if lj != GpuCtrlType::Mg as u32 { return Err("SET_SCANOUT failed"); }
        self.eyb = awu;
        crate::serial_println!("[VIRTIO-GPU] Scanout {} -> resource {} ({}x{})", mcj, awu, d, i);
        Ok(())
    }
    
    pub fn zto(&mut self, awu: u32, d: u32, i: u32) -> Result<(), &'static str> {
        let dma = self.alb.as_ref().ok_or("DMA not ready")?;
        let cmd = Ww {
            zj: Ar { btv: GpuCtrlType::Aqb as u32, ..Default::default() },
            m: Ko { b: 0, c: 0, z: d, ac: i },
            l: 0,
            awu,
            ob: 0,
        };
        unsafe { dma.ciu(0, &cmd); }
        
        let lj = self.dvn(
            core::mem::size_of::<Ww>() as u32,
            512, core::mem::size_of::<Ar>() as u32,
        )?;
        if lj != GpuCtrlType::Mg as u32 { return Err("TRANSFER failed"); }
        Ok(())
    }
    
    pub fn yrc(&mut self, awu: u32, d: u32, i: u32) -> Result<(), &'static str> {
        let dma = self.alb.as_ref().ok_or("DMA not ready")?;
        let cmd = Wv {
            zj: Ar { btv: GpuCtrlType::Aqa as u32, ..Default::default() },
            m: Ko { b: 0, c: 0, z: d, ac: i },
            awu,
            ob: 0,
        };
        unsafe { dma.ciu(0, &cmd); }
        
        let lj = self.dvn(
            core::mem::size_of::<Wv>() as u32,
            512, core::mem::size_of::<Ar>() as u32,
        )?;
        if lj != GpuCtrlType::Mg as u32 { return Err("FLUSH failed"); }
        Ok(())
    }
    
    
    
    pub fn wlj(&mut self) -> Result<(), &'static str> {
        if !self.jr { return Err("GPU not initialized"); }
        
        
        let (gz, kc) = crate::framebuffer::yn();
        if gz > 0 && kc > 0 {
            crate::serial_println!("[VIRTIO-GPU] Using framebuffer dimensions: {}x{} (display was {}x{})",
                gz, kc, self.bjv, self.bqf);
            self.bjv = gz;
            self.bqf = kc;
        }
        
        let d = self.bjv;
        let i = self.bqf;
        crate::serial_println!("[VIRTIO-GPU] Setting up scanout {}x{}", d, i);
        
        let awu = self.nhg(d, i)?;
        
        
        let dzh = (d * i) as usize;
        let hbr = dzh * 4;
        
        use alloc::alloc::{alloc_zeroed, Layout};
        let layout = Layout::bjy(hbr, 4096).jd(|_| "Layout error")?;
        let ptr = unsafe { alloc_zeroed(layout) };
        if ptr.abq() { return Err("Backing buffer allocation failed"); }
        
        let ju = ptr as u64;
        let hp = memory::lr();
        let ht = if ju >= hp { ju - hp } else { ju };
        
        let bi = unsafe {
            let slice = core::slice::bef(ptr as *mut u32, dzh);
            Box::nwh(slice as *mut [u32])
        };
        
        self.fcx = Some(bi);
        self.mxo = ht;
        
        self.mwm(awu, ht, hbr as u32)?;
        self.mew(0, awu, d, i)?;
        
        crate::serial_println!("[VIRTIO-GPU] Scanout ready! phys={:#X}", ht);
        Ok(())
    }
    
    
    
    
    pub fn wkp(&mut self) -> Result<(), &'static str> {
        if !self.jr { return Err("GPU not initialized"); }
        if self.eyb == 0 { return Err("No primary scanout"); }
        
        let d = self.bjv;
        let i = self.bqf;
        
        
        let kbr = self.nhg(d, i)?;
        
        
        let dzh = (d * i) as usize;
        let hbr = dzh * 4;
        
        use alloc::alloc::{alloc_zeroed, Layout};
        let layout = Layout::bjy(hbr, 4096)
            .jd(|_| "Layout error")?;
        let ptr = unsafe { alloc_zeroed(layout) };
        if ptr.abq() { return Err("Back buffer allocation failed"); }
        
        let ju = ptr as u64;
        let hp = memory::lr();
        let ht = if ju >= hp { ju - hp } else { ju };
        
        let bi = unsafe {
            let slice = core::slice::bef(ptr as *mut u32, dzh);
            Box::nwh(slice as *mut [u32])
        };
        
        self.mwm(kbr, ht, hbr as u32)?;
        
        self.kbs = kbr;
        self.aqt = Some(bi);
        self.mxk = ht;
        self.hgq = true;
        self.ghp = true;
        
        crate::serial_println!("[VIRTIO-GPU] Double buffer enabled: resource A={}, B={}", 
            self.eyb, kbr);
        Ok(())
    }
    
    
    pub fn wwk(&mut self) -> Result<(), &'static str> {
        if !self.hgq { return Ok(()); }
        
        let (d, i) = (self.bjv, self.bqf);
        
        if self.ghp {
            
            self.mew(0, self.kbs, d, i)?;
        } else {
            
            self.mew(0, self.eyb, d, i)?;
        }
        
        self.ghp = !self.ghp;
        Ok(())
    }
    
    
    pub fn kyi(&mut self) -> Option<&mut [u32]> {
        if !self.hgq {
            return self.fcx.gza();
        }
        if self.ghp {
            
            self.aqt.gza()
        } else {
            
            self.fcx.gza()
        }
    }
    
    pub fn ysp(&mut self) -> Option<&mut [u32]> {
        self.fcx.gza()
    }
    
    pub fn yn(&self) -> (u32, u32) {
        (self.bjv, self.bqf)
    }
    
    
    
    
    
    
    pub fn brs(&mut self) -> Result<(), &'static str> {
        let ehs = self.eyb;
        if ehs == 0 { return Err("No scanout"); }
        let (d, i) = (self.bjv, self.bqf);
        
        let dma = self.alb.as_ref().ok_or("DMA not ready")?;
        let bua = dma.ht;
        
        
        let mms = Ww {
            zj: Ar { btv: GpuCtrlType::Aqb as u32, ..Default::default() },
            m: Ko { b: 0, c: 0, z: d, ac: i },
            l: 0,
            awu: ehs,
            ob: 0,
        };
        unsafe { dma.ciu(0, &mms); }
        
        
        let hjz = Wv {
            zj: Ar { btv: GpuCtrlType::Aqa as u32, ..Default::default() },
            m: Ko { b: 0, c: 0, z: d, ac: i },
            awu: ehs,
            ob: 0,
        };
        unsafe { dma.ciu(256, &hjz); }
        
        let mmu = core::mem::size_of::<Ww>() as u32;
        let kwp = core::mem::size_of::<Wv>() as u32;
        let fst = core::mem::size_of::<Ar>() as u32;
        
        let wy = self.wy.as_mut().ok_or("controlq not ready")?;
        
        
        let cdx = wy.blx().ok_or("No free desc")?;
        let apo = wy.blx().ok_or("No free desc")?;
        let us = wy.blx().ok_or("No free desc")?;
        let cdy = wy.blx().ok_or("No free desc")?;
        
        
        wy.bwz(cdx, bua, mmu, QP_, apo);
        wy.bwz(apo, bua + 512, fst, QQ_, 0);
        
        
        wy.bwz(us, bua + 256, kwp, QP_, cdy);
        wy.bwz(cdy, bua + 768, fst, QQ_, 0);
        
        
        wy.dmd(cdx);
        wy.dmd(us);
        
        
        if let Some(djy) = &self.fpe {
            djy.asg(0, 0);
        }
        
        
        let mut cpn = 0u8;
        let mut aah = 5_000_000u32;
        while cpn < 2 {
            if let Some(_) = wy.lur() {
                cpn += 1;
            }
            if cpn < 2 {
                aah -= 1;
                if aah == 0 {
                    wy.ald(cdy);
                    wy.ald(us);
                    wy.ald(apo);
                    wy.ald(cdx);
                    return Err("Batched present timeout");
                }
                core::hint::hc();
            }
        }
        
        
        let dma = self.alb.as_ref().unwrap();
        let wzy = unsafe { dma.read_at::<Ar>(512) }.btv;
        let sqp = unsafe { dma.read_at::<Ar>(768) }.btv;
        
        wy.ald(cdy);
        wy.ald(us);
        wy.ald(apo);
        wy.ald(cdx);
        
        if wzy != GpuCtrlType::Mg as u32 { return Err("TRANSFER failed"); }
        if sqp != GpuCtrlType::Mg as u32 { return Err("FLUSH failed"); }
        
        Ok(())
    }
    
    
    
    
    pub fn vkw(&mut self, b: u32, c: u32, d: u32, i: u32) -> Result<(), &'static str> {
        let ehs = self.eyb;
        if ehs == 0 { return Err("No scanout"); }
        
        
        let b = b.v(self.bjv);
        let c = c.v(self.bqf);
        let d = d.v(self.bjv.ao(b));
        let i = i.v(self.bqf.ao(c));
        if d == 0 || i == 0 { return Ok(()); }
        
        let dma = self.alb.as_ref().ok_or("DMA not ready")?;
        let bua = dma.ht;
        
        
        let l = ((c * self.bjv + b) as u64) * 4;
        
        let mms = Ww {
            zj: Ar { btv: GpuCtrlType::Aqb as u32, ..Default::default() },
            m: Ko { b, c, z: d, ac: i },
            l,
            awu: ehs,
            ob: 0,
        };
        unsafe { dma.ciu(0, &mms); }
        
        let hjz = Wv {
            zj: Ar { btv: GpuCtrlType::Aqa as u32, ..Default::default() },
            m: Ko { b, c, z: d, ac: i },
            awu: ehs,
            ob: 0,
        };
        unsafe { dma.ciu(256, &hjz); }
        
        let mmu = core::mem::size_of::<Ww>() as u32;
        let kwp = core::mem::size_of::<Wv>() as u32;
        let fst = core::mem::size_of::<Ar>() as u32;
        
        let wy = self.wy.as_mut().ok_or("controlq not ready")?;
        let cdx = wy.blx().ok_or("No free desc")?;
        let apo = wy.blx().ok_or("No free desc")?;
        let us = wy.blx().ok_or("No free desc")?;
        let cdy = wy.blx().ok_or("No free desc")?;
        
        wy.bwz(cdx, bua, mmu, QP_, apo);
        wy.bwz(apo, bua + 512, fst, QQ_, 0);
        wy.bwz(us, bua + 256, kwp, QP_, cdy);
        wy.bwz(cdy, bua + 768, fst, QQ_, 0);
        
        wy.dmd(cdx);
        wy.dmd(us);
        
        if let Some(djy) = &self.fpe {
            djy.asg(0, 0);
        }
        
        let mut cpn = 0u8;
        let mut aah = 5_000_000u32;
        while cpn < 2 {
            if let Some(_) = wy.lur() { cpn += 1; }
            if cpn < 2 {
                aah -= 1;
                if aah == 0 {
                    wy.ald(cdy);
                    wy.ald(us);
                    wy.ald(apo);
                    wy.ald(cdx);
                    return Err("Rect present timeout");
                }
                core::hint::hc();
            }
        }
        
        wy.ald(cdy);
        wy.ald(us);
        wy.ald(apo);
        wy.ald(cdx);
        Ok(())
    }
    
    pub fn ky(&self) -> bool { self.jr }
    pub fn ywf(&self) -> bool { self.ecl }
}





static Hq: Mutex<VirtioGpu> = Mutex::new(VirtioGpu::new());
static KH_: AtomicBool = AtomicBool::new(false);

pub fn oel() -> Result<(), &'static str> {
    for de in crate::pci::arx() {
        if de.ml == DAW_ && de.mx == DAV_ {
            crate::serial_println!("[VIRTIO-GPU] Found device at {:02x}:{:02x}.{}",
                de.aq, de.de, de.gw);
            
            let mut gpu = Hq.lock();
            match gpu.init(de) {
                Ok(()) => {
                    match gpu.wlj() {
                        Ok(()) => {
                            KH_.store(true, Ordering::SeqCst);
                            crate::serial_println!("[VIRTIO-GPU] ✓ Ready for rendering!");
                            
                            match gpu.wkp() {
                                Ok(()) => crate::serial_println!("[VIRTIO-GPU] ✓ Double buffer enabled (tear-free)"),
                                Err(aa) => crate::serial_println!("[VIRTIO-GPU] Double buffer skipped: {}", aa),
                            }
                        }
                        Err(aa) => crate::serial_println!("[VIRTIO-GPU] Scanout failed: {}", aa),
                    }
                }
                Err(aa) => crate::serial_println!("[VIRTIO-GPU] Init failed: {}", aa),
            }
            return Ok(());
        }
    }
    crate::serial_println!("[VIRTIO-GPU] No VirtIO GPU found");
    Ok(())
}

pub fn anl() -> bool {
    KH_.load(Ordering::SeqCst)
}

pub fn fgc(z: u32, ac: u32) -> GpuSurface {
    GpuSurface::new(z, ac)
}


pub fn zjk<G: FnOnce(&mut [u32], u32, u32)>(vvu: G) -> Result<(), &'static str> {
    let mut gpu = Hq.lock();
    if !gpu.jr { return Err("GPU not initialized"); }
    let (d, i) = (gpu.bjv, gpu.bqf);
    if let Some(k) = gpu.fcx.gza() {
        vvu(k, d, i);
    }
    gpu.brs()
}


pub fn owx() -> Result<(), &'static str> {
    Hq.lock().brs()
}



pub fn vku() -> Result<(), &'static str> {
    let mut gpu = Hq.lock();
    if !gpu.hgq {
        return gpu.brs();
    }
    
    gpu.brs()?;
    
    gpu.wwk()
}


pub fn kyi() -> Option<(*mut u32, u32, u32)> {
    let mut gpu = Hq.lock();
    if !gpu.jr { return None; }
    let (d, i) = (gpu.bjv, gpu.bqf);
    gpu.kyi().map(|k| (k.mw(), d, i))
}




pub fn vkt(akn: &[(u32, u32, u32, u32)]) {
    if akn.is_empty() { return; }
    
    
    let (gz, xzd) = crate::framebuffer::yn();
    if let Some((hlu, erl, hlt)) = iwv() {
        if let Some(aaa) = crate::framebuffer::nxr() {
            let aoo = (gz as usize).v(erl as usize);
            let bbg = hlt as usize;
            unsafe {
                for c in 0..bbg {
                    let cy = aaa.add(c * gz as usize);
                    let cs = hlu.add(c * erl as usize);
                    #[cfg(target_arch = "x86_64")]
                    crate::graphics::simd::dpd(cs, cy, aoo);
                    #[cfg(not(target_arch = "x86_64"))]
                    core::ptr::copy_nonoverlapping(cy, cs, aoo);
                }
            }
        }
    }
    
    
    let mut gpu = Hq.lock();
    if !gpu.jr { return; }
    let ehs = gpu.eyb;
    if ehs == 0 { return; }
    
    for &(b, c, d, i) in akn {
        let _ = gpu.vkw(b, c, d, i);
    }
}


pub fn iwv() -> Option<(*mut u32, u32, u32)> {
    let mut gpu = Hq.lock();
    if !gpu.jr { return None; }
    let (d, i) = (gpu.bjv, gpu.bqf);
    gpu.fcx.gza().map(|k| (k.mw(), d, i))
}


pub fn lea() -> alloc::string::String {
    let gpu = Hq.lock();
    if gpu.jr {
        alloc::format!("VirtIO GPU: {}x{} 2D (3D={})", gpu.bjv, gpu.bqf,
            if gpu.ecl { "virgl" } else { "no" })
    } else {
        alloc::string::String::from("VirtIO GPU: not available")
    }
}


pub fn qqg(surface: &GpuSurface, b: u32, c: u32) {
    let (gz, kc) = crate::framebuffer::yn();
    crate::framebuffer::afi(true);
    for cq in 0..surface.ac {
        let bcw = c + cq;
        if bcw >= kc { break; }
        for cr in 0..surface.z {
            let amy = b + cr;
            if amy >= gz { break; }
            crate::framebuffer::sf(amy, bcw, surface.beg(cr, cq));
        }
    }
    crate::framebuffer::sv();
}

pub fn yrd() {
    if crate::framebuffer::bre() {
        crate::framebuffer::sv();
    }
}

pub fn init() {
    crate::serial_println!("[GPU] Initializing graphics subsystem...");
    if let Err(aa) = oel() {
        crate::serial_println!("[GPU] PCI init error: {}", aa);
    }
    crate::framebuffer::beo();
    crate::framebuffer::afi(true);
    crate::serial_println!("[GPU] Graphics ready (VirtIO: {})", 
        if anl() { "ACTIVE" } else { "fallback" });
}











#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct Bal {
    pub qwc: u32,
    pub qwe: u32,
    pub qwd: u32,
    pub ob: u32,
}


#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Bib {
    pub zj: Ar,
    pub gnt: u32,
    pub rom: u32,
    pub njv: [u8; 64],
}


#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Bic {
    pub zj: Ar,
}


#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Cyf {
    pub zj: Ar,
    pub awu: u32,
    pub cd: u32,     
    pub format: u32,     
    pub kdj: u32,       
    pub z: u32,
    pub ac: u32,
    pub eo: u32,
    pub yfc: u32,
    pub zak: u32,
    pub zea: u32,
    pub flags: u32,
    pub ob: u32,
}


#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Bih {
    pub zj: Ar,
    pub aw: u32,
    pub ob: u32,
    
}


#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Cye {
    pub zj: Ar,
    pub awu: u32,
    pub ob: u32,
}


#[allow(bgr)]
pub mod virgl_bind {
    pub const DJB_: u32 = 1 << 0;
    pub const ECI_: u32 = 1 << 1;
    pub const EDS_: u32 = 1 << 3;
    pub const EJU_: u32 = 1 << 4;
    pub const DQX_: u32 = 1 << 5;
    pub const DGP_: u32 = 1 << 6;
    pub const EGF_: u32 = 1 << 14;
}


#[allow(bgr)]
pub mod virgl_target {
    pub const Cru: u32 = 0;
    pub const EHJ_: u32 = 1;
    pub const EHK_: u32 = 2;
    pub const EHL_: u32 = 3;
    pub const EHM_: u32 = 4;
}


pub struct Bvv {
    cjv: u32,
    gh: bool,
    qwf: u32,
}

static YG_: Mutex<Bvv> = Mutex::new(Bvv {
    cjv: 0,
    gh: false,
    qwf: 0,
});


pub fn ywo() -> bool {
    let gpu = Hq.lock();
    gpu.ecl
}


pub fn zhc() -> Option<Bal> {
    let mut gpu = Hq.lock();
    if !gpu.ecl || !gpu.jr { return None; }
    
    let dma = gpu.alb.as_ref()?;
    
    
    let cmd = Ar {
        btv: GpuCtrlType::Bzr as u32,
        ..Default::default()
    };
    unsafe { dma.ciu(0, &cmd); }
    
    unsafe { (dma.ju.add(core::mem::size_of::<Ar>()) as *mut u32).write_volatile(0); }
    
    let kjf = core::mem::size_of::<Ar>() as u32 + 4;
    let fst = core::mem::size_of::<Ar>() as u32 + core::mem::size_of::<Bal>() as u32;
    
    let gqw = gpu.dvn(kjf, 512, fst).bq()?;
    if gqw != GpuCtrlType::Ckp as u32 { return None; }
    
    let dma = gpu.alb.as_ref()?;
    let co: Bal = unsafe { dma.read_at(512 + core::mem::size_of::<Ar>()) };
    crate::serial_println!("[VIRGL] Capset: id={}, max_version={}, max_size={}", 
        co.qwc, co.qwe, co.qwd);
    Some(co)
}


pub fn ykt(j: &str) -> Result<u32, &'static str> {
    let mut gpu = Hq.lock();
    if !gpu.ecl { return Err("No 3D support"); }
    if !gpu.jr { return Err("GPU not initialized"); }
    
    let cjv = 1u32; 
    
    let mut cmd = Bib {
        zj: Ar {
            btv: GpuCtrlType::Bzp as u32,
            cjv,
            ..Default::default()
        },
        gnt: j.len().v(63) as u32,
        rom: 0,
        njv: [0u8; 64],
    };
    
    let bko = j.as_bytes();
    let zg = bko.len().v(63);
    cmd.njv[..zg].dg(&bko[..zg]);
    
    let dma = gpu.alb.as_ref().ok_or("DMA not ready")?;
    unsafe { dma.ciu(0, &cmd); }
    
    let lj = gpu.dvn(
        core::mem::size_of::<Bib>() as u32,
        512,
        core::mem::size_of::<Ar>() as u32,
    )?;
    
    if lj != GpuCtrlType::Mg as u32 {
        return Err("CTX_CREATE failed");
    }
    
    let mut cvf = YG_.lock();
    cvf.cjv = cjv;
    cvf.gh = true;
    
    crate::serial_println!("[VIRGL] 3D context created: id={} name={}", cjv, j);
    Ok(cjv)
}


pub fn ylv() -> Result<(), &'static str> {
    let mut cvf = YG_.lock();
    if !cvf.gh { return Ok(()); }
    
    let mut gpu = Hq.lock();
    let cmd = Bic {
        zj: Ar {
            btv: GpuCtrlType::Bzq as u32,
            cjv: cvf.cjv,
            ..Default::default()
        },
    };
    
    let dma = gpu.alb.as_ref().ok_or("DMA not ready")?;
    unsafe { dma.ciu(0, &cmd); }
    
    let lj = gpu.dvn(
        core::mem::size_of::<Bic>() as u32,
        512,
        core::mem::size_of::<Ar>() as u32,
    )?;
    
    if lj != GpuCtrlType::Mg as u32 {
        return Err("CTX_DESTROY failed");
    }
    
    cvf.gh = false;
    crate::serial_println!("[VIRGL] 3D context destroyed");
    Ok(())
}


pub fn zpt(commands: &[u8]) -> Result<(), &'static str> {
    let cvf = YG_.lock();
    if !cvf.gh { return Err("No active 3D context"); }
    let cjv = cvf.cjv;
    drop(cvf);
    
    let mut gpu = Hq.lock();
    if !gpu.jr { return Err("GPU not initialized"); }
    
    if commands.len() > 3800 { return Err("Command buffer too large"); }
    
    let dma = gpu.alb.as_ref().ok_or("DMA not ready")?;
    
    let cmd = Bih {
        zj: Ar {
            btv: GpuCtrlType::Bzw as u32,
            cjv,
            ..Default::default()
        },
        aw: commands.len() as u32,
        ob: 0,
    };
    unsafe { dma.ciu(0, &cmd); }
    
    
    let nea = core::mem::size_of::<Bih>();
    unsafe {
        core::ptr::copy_nonoverlapping(
            commands.fq(),
            dma.ju.add(nea),
            commands.len(),
        );
    }
    
    let xjy = (nea + commands.len()) as u32;
    let lj = gpu.dvn(xjy, 512, core::mem::size_of::<Ar>() as u32)?;
    
    if lj != GpuCtrlType::Mg as u32 {
        return Err("SUBMIT_3D failed");
    }
    Ok(())
}


pub fn zvj() -> alloc::string::String {
    let gpu = Hq.lock();
    let cvf = YG_.lock();
    if gpu.ecl {
        alloc::format!("VIRGL: {} (ctx={})", 
            if cvf.gh { "active" } else { "ready" },
            cvf.cjv)
    } else {
        alloc::string::String::from("VIRGL: not available")
    }
}





pub struct Layer {
    pub surface: GpuSurface,
    pub b: i32,
    pub c: i32,
    pub ell: i32,
    pub iw: bool,
    pub adh: u8,
}

pub struct Compositor {
    my: Vec<Layer>,
    an: GpuSurface,
    cdb: u32,
}

impl Compositor {
    pub fn new(z: u32, ac: u32) -> Self {
        Self { my: Vec::new(), an: GpuSurface::new(z, ac), cdb: 0xFF1A1A1A }
    }
    pub fn dyc(&mut self, surface: GpuSurface, b: i32, c: i32, ell: i32) -> usize {
        let w = self.my.len();
        self.my.push(Layer { surface, b, c, ell, iw: true, adh: 255 });
        self.my.bxf(|dm| dm.ell);
        w
    }
    pub fn vux(&mut self, index: usize) {
        if index < self.my.len() { self.my.remove(index); }
    }
    pub fn nff(&mut self) {
        self.an.clear(self.cdb);
        for fl in &self.my {
            if fl.iw { self.an.bge(&fl.surface, fl.b, fl.c); }
        }
    }
    pub fn tj(&self) { qqg(&self.an, 0, 0); }
    pub fn iws(&self, index: usize) -> Option<&Layer> { self.my.get(index) }
    pub fn dhm(&mut self, index: usize) -> Option<&mut Layer> { self.my.ds(index) }
    pub fn wig(&mut self, s: u32) { self.cdb = s; }
}
