




use alloc::boxed::Box;
use core::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use spin::Mutex;

use crate::virtio::{VirtioDevice, Virtqueue, desc_flags, status};
use crate::pci::S;


pub const AR_: usize = 512;


pub mod features {
    pub const EGL_: u32 = 1 << 1;      
    pub const EFJ_: u32 = 1 << 2;       
    pub const Cxk: u32 = 1 << 4;      
    pub const Bqf: u32 = 1 << 5;            
    pub const DDQ_: u32 = 1 << 6;      
    pub const Cdb: u32 = 1 << 9;         
    pub const Djm: u32 = 1 << 10;     
    pub const DGM_: u32 = 1 << 11;   
}


pub mod req_type {
    pub const Cfk: u32 = 0;       
    pub const Cif: u32 = 1;      
    pub const Cdb: u32 = 4;    
    pub const Cua: u32 = 11; 
    pub const EKT_: u32 = 13;
}


pub mod blk_status {
    pub const Bnq: u8 = 0;
    pub const Czb: u8 = 1;
    pub const Dke: u8 = 2;
}


#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct VirtioBlkReqHdr {
    pub req_type: u32,
    pub awt: u32,
    pub jk: u64,
}

impl VirtioBlkReqHdr {
    pub const Am: usize = 16;
}


pub struct VirtioBlk {
    
    de: VirtioDevice,
    
    queue: Option<Box<Virtqueue>>,
    
    aty: u64,
    
    zn: u32,
    
    hwy: bool,
}


static Fl: Mutex<Option<VirtioBlk>> = Mutex::new(None);
static Be: AtomicBool = AtomicBool::new(false);


static BIK_: core::sync::atomic::AtomicU16 = core::sync::atomic::AtomicU16::new(0);

static QO_: AtomicBool = AtomicBool::new(false);


static Bpz: AtomicU64 = AtomicU64::new(0);
static Bwj: AtomicU64 = AtomicU64::new(0);
static JO_: AtomicU64 = AtomicU64::new(0);
static JP_: AtomicU64 = AtomicU64::new(0);

impl VirtioBlk {
    
    pub fn new(sq: &S) -> Result<Self, &'static str> {
        
        let aew = sq.bar[0];
        if aew == 0 {
            return Err("BAR0 not configured");
        }
        
        
        let agq = if aew & 1 == 1 {
            (aew & 0xFFFC) as u16
        } else {
            return Err("MMIO not supported yet, need I/O port BAR");
        };
        
        crate::log_debug!("[virtio-blk] I/O base: {:#X}", agq);
        
        let mut de = VirtioDevice::new(agq);
        
        
        de.apa();
        
        
        de.fzu(status::Or);
        de.fzu(status::Fl);
        
        
        let bju = de.pab();
        crate::log_debug!("[virtio-blk] Device features: {:#X}", bju);
        
        
        let hwy = (bju & features::Bqf) != 0;
        
        
        let ckb = 0u32; 
        de.pzx(ckb);
        
        
        
        let kgk = de.ozw(0) as u64;
        let kgj = de.ozw(4) as u64;
        let aty = kgk | (kgj << 32);
        
        crate::log!("[virtio-blk] Capacity: {} sectors ({} MB)", 
            aty, (aty * 512) / (1024 * 1024));
        
        if hwy {
            crate::log!("[virtio-blk] Device is read-only");
        }
        
        Ok(Self {
            de,
            queue: None,
            aty,
            zn: 512,
            hwy,
        })
    }
    
    
    pub fn wli(&mut self) -> Result<(), &'static str> {
        self.de.mdl(0);
        let aw = self.de.kyw();
        crate::log_debug!("[virtio-blk] Queue size: {}", aw);
        
        if aw == 0 {
            return Err("Queue not available");
        }
        
        let queue = self.ijn(aw)?;
        
        
        let duh = (queue.ki / 4096) as u32;
        self.de.meu(duh);
        
        self.queue = Some(queue);
        Ok(())
    }
    
    
    fn ijn(&self, aw: u16) -> Result<Box<Virtqueue>, &'static str> {
        Virtqueue::new(aw)
    }
    
    
    pub fn ay(&mut self) -> Result<(), &'static str> {
        self.de.fzu(status::HW_);
        crate::log!("[virtio-blk] Device started");
        Ok(())
    }
    
    
    pub fn ain(&mut self, awy: u64, az: usize, bi: &mut [u8]) -> Result<(), &'static str> {
        if bi.len() < az * AR_ {
            return Err("Buffer too small");
        }
        
        if awy + az as u64 > self.aty {
            return Err("Read beyond device capacity");
        }
        
        
        for a in 0..az {
            self.vsf(awy + a as u64, 
                &mut bi[a * AR_..(a + 1) * AR_])?;
        }
        
        Bpz.fetch_add(az as u64, Ordering::Relaxed);
        JO_.fetch_add((az * AR_) as u64, Ordering::Relaxed);
        
        Ok(())
    }
    
    
    fn vsf(&mut self, jk: u64, bi: &mut [u8]) -> Result<(), &'static str> {
        
        use alloc::boxed::Box;
        use alloc::vec;
        
        
        
        let aay = VirtioBlkReqHdr::Am + AR_ + 1;
        let mut alb = vec![0u8; aay].dsd();
        
        
        let dh = VirtioBlkReqHdr {
            req_type: req_type::Cfk,
            awt: 0,
            jk,
        };
        unsafe {
            let lbu = alb.mw() as *mut VirtioBlkReqHdr;
            core::ptr::write(lbu, dh);
        }
        
        
        alb[VirtioBlkReqHdr::Am + AR_] = 0xFF;
        
        
        let hp = crate::memory::lr();
        let kqi = alb.fq() as u64;
        let bua = kqi - hp;
        
        let lbt = bua;
        let cpu = bua + VirtioBlkReqHdr::Am as u64;
        let bik = bua + (VirtioBlkReqHdr::Am + AR_) as u64;
        
        let queue = self.queue.as_mut().ok_or("Queue not initialized")?;
        
        
        let ale = queue.blx().ok_or("No free descriptor")?;
        let eal = queue.blx().ok_or("No free descriptor")?;
        let cmy = queue.blx().ok_or("No free descriptor")?;
        
        
        queue.bwz(ale, lbt, VirtioBlkReqHdr::Am as u32, 
            desc_flags::Akj, eal);
        
        
        queue.bwz(eal, cpu, AR_ as u32,
            desc_flags::Db | desc_flags::Akj, cmy);
        
        
        queue.bwz(cmy, bik, 1, desc_flags::Db, 0);
        
        
        queue.dmd(ale);
        
        
        let agq = self.de.agq;
        
        
        unsafe {
            let mut port = crate::arch::Port::<u16>::new(agq + 0x10);
            port.write(0);
        }
        
        
        QO_.store(false, Ordering::Release);
        
        
        let queue = self.queue.as_mut().ok_or("Queue not initialized")?;
        
        
        let mut aah = 1_000_000u32;
        while !queue.ixy() && aah > 0 {
            if QO_.load(Ordering::Acquire) {
                break;
            }
            core::hint::hc();
            aah -= 1;
        }
        
        if aah == 0 {
            queue.ald(ale);
            queue.ald(eal);
            queue.ald(cmy);
            return Err("Request timeout");
        }
        
        
        let qdw = queue.jjp().ok_or("No completion")?;
        
        
        queue.ald(ale);
        queue.ald(eal);
        queue.ald(cmy);
        
        if alb[VirtioBlkReqHdr::Am + AR_] != blk_status::Bnq {
            return Err("Device error");
        }
        
        
        bi.dg(&alb[VirtioBlkReqHdr::Am..VirtioBlkReqHdr::Am + AR_]);
        
        Ok(())
    }
    
    
    pub fn bpi(&mut self, awy: u64, az: usize, bi: &[u8]) -> Result<(), &'static str> {
        if self.hwy {
            return Err("Device is read-only");
        }
        
        if bi.len() < az * AR_ {
            return Err("Buffer too small");
        }
        
        if awy + az as u64 > self.aty {
            return Err("Write beyond device capacity");
        }
        
        
        for a in 0..az {
            self.xvp(awy + a as u64,
                &bi[a * AR_..(a + 1) * AR_])?;
        }
        
        Bwj.fetch_add(az as u64, Ordering::Relaxed);
        JP_.fetch_add((az * AR_) as u64, Ordering::Relaxed);
        
        Ok(())
    }
    
    
    fn xvp(&mut self, jk: u64, bi: &[u8]) -> Result<(), &'static str> {
        
        use alloc::vec;
        
        
        
        let aay = VirtioBlkReqHdr::Am + AR_ + 1;
        let mut alb = vec![0u8; aay].dsd();
        
        
        let dh = VirtioBlkReqHdr {
            req_type: req_type::Cif,
            awt: 0,
            jk,
        };
        unsafe {
            let lbu = alb.mw() as *mut VirtioBlkReqHdr;
            core::ptr::write(lbu, dh);
        }
        
        
        alb[VirtioBlkReqHdr::Am..VirtioBlkReqHdr::Am + AR_]
            .dg(&bi[..AR_]);
        
        
        alb[VirtioBlkReqHdr::Am + AR_] = 0xFF;
        
        
        let hp = crate::memory::lr();
        let kqi = alb.fq() as u64;
        let bua = kqi - hp;
        
        let lbt = bua;
        let cpu = bua + VirtioBlkReqHdr::Am as u64;
        let bik = bua + (VirtioBlkReqHdr::Am + AR_) as u64;
        
        let queue = self.queue.as_mut().ok_or("Queue not initialized")?;
        
        let ale = queue.blx().ok_or("No free descriptor")?;
        let eal = queue.blx().ok_or("No free descriptor")?;
        let cmy = queue.blx().ok_or("No free descriptor")?;
        
        
        queue.bwz(ale, lbt, VirtioBlkReqHdr::Am as u32,
            desc_flags::Akj, eal);
        
        
        queue.bwz(eal, cpu, AR_ as u32,
            desc_flags::Akj, cmy);
        
        
        queue.bwz(cmy, bik, 1, desc_flags::Db, 0);
        
        queue.dmd(ale);
        
        
        let agq = self.de.agq;
        
        
        unsafe {
            let mut port = crate::arch::Port::<u16>::new(agq + 0x10);
            port.write(0);
        }
        
        
        QO_.store(false, Ordering::Release);
        
        
        let queue = self.queue.as_mut().ok_or("Queue not initialized")?;
        
        let mut aah = 1_000_000u32;
        while !queue.ixy() && aah > 0 {
            if QO_.load(Ordering::Acquire) {
                break;
            }
            core::hint::hc();
            aah -= 1;
        }
        
        if aah == 0 {
            queue.ald(ale);
            queue.ald(eal);
            queue.ald(cmy);
            return Err("Request timeout");
        }
        
        let qdw = queue.jjp().ok_or("No completion")?;
        
        queue.ald(ale);
        queue.ald(eal);
        queue.ald(cmy);
        
        if alb[VirtioBlkReqHdr::Am + AR_] != blk_status::Bnq {
            return Err("Device error");
        }
        
        Ok(())
    }
    
    
    pub fn aty(&self) -> u64 {
        self.aty
    }
    
    
    pub fn jbr(&self) -> bool {
        self.hwy
    }
}




pub fn init(sq: &S) -> Result<(), &'static str> {
    crate::log!("[virtio-blk] Initializing...");
    
    let mut rj = VirtioBlk::new(sq)?;
    rj.wli()?;
    rj.ay()?;
    
    
    BIK_.store(rj.de.agq, Ordering::SeqCst);
    
    Be.store(true, Ordering::SeqCst);
    *Fl.lock() = Some(rj);
    
    
    let irq = sq.esw;
    if irq > 0 && irq < 255 {
        crate::apic::jmw(irq, crate::apic::HH_);
        crate::serial_println!("[virtio-blk] IRQ {} routed to vector {}", irq, crate::apic::HH_);
    }
    
    Ok(())
}


pub fn ky() -> bool {
    Be.load(Ordering::Relaxed)
}


pub fn aty() -> u64 {
    Fl.lock().as_ref().map(|bc| bc.aty()).unwrap_or(0)
}


pub fn jbr() -> bool {
    Fl.lock().as_ref().map(|bc| bc.jbr()).unwrap_or(true)
}


pub fn ain(ay: u64, az: usize, bi: &mut [u8]) -> Result<(), &'static str> {
    let mut rj = Fl.lock();
    let ane = rj.as_mut().ok_or("Driver not initialized")?;
    ane.ain(ay, az, bi)
}


pub fn bpi(ay: u64, az: usize, bi: &[u8]) -> Result<(), &'static str> {
    let mut rj = Fl.lock();
    let ane = rj.as_mut().ok_or("Driver not initialized")?;
    ane.bpi(ay, az, bi)
}


pub fn xr(jk: u64, bi: &mut [u8; 512]) -> Result<(), &'static str> {
    ain(jk, 1, bi)
}


pub fn aby(jk: u64, bi: &[u8; 512]) -> Result<(), &'static str> {
    bpi(jk, 1, bi)
}


pub fn asx() -> (u64, u64, u64, u64) {
    (
        Bpz.load(Ordering::Relaxed),
        Bwj.load(Ordering::Relaxed),
        JO_.load(Ordering::Relaxed),
        JP_.load(Ordering::Relaxed),
    )
}



pub fn eck() {
    let agq = BIK_.load(Ordering::Relaxed);
    if agq == 0 { return; }
    
    
    let cru: u8 = unsafe {
        let mut port = crate::arch::Port::<u8>::new(agq + 0x13);
        port.read()
    };
    
    if cru & 1 != 0 {
        
        QO_.store(true, Ordering::Release);
    }
}
