




use alloc::vec::Vec;
use alloc::vec;
use alloc::boxed::Box;
use alloc::collections::VecDeque;
use core::sync::atomic::{AtomicBool, Ordering};
use spin::Mutex;
use crate::arch::Port;

use crate::virtio::{self, VirtioDevice, Virtqueue, VirtqDesc, desc_flags, status, legacy_reg};
use crate::pci::S;


pub mod features {
    pub const Csq: u32 = 1 << 0;           
    pub const DON_: u32 = 1 << 1;     
    pub const Bls: u32 = 1 << 5;            
    pub const Cxy: u32 = 1 << 6;            
    pub const DOP_: u32 = 1 << 7;     
    pub const DOQ_: u32 = 1 << 8;     
    pub const DOO_: u32 = 1 << 9;      
    pub const DOR_: u32 = 1 << 10;     
    pub const DPG_: u32 = 1 << 11;     
    pub const DPH_: u32 = 1 << 12;     
    pub const DPF_: u32 = 1 << 13;      
    pub const DPI_: u32 = 1 << 14;      
    pub const DUX_: u32 = 1 << 15;     
    pub const Nz: u32 = 1 << 16;        
    pub const DIL_: u32 = 1 << 17;       
    pub const DIJ_: u32 = 1 << 18;       
    pub const DIK_: u32 = 1 << 19;     
    pub const DOM_: u32 = 1 << 21; 
}


#[repr(C, packed)]
#[derive(Debug, Clone, Copy, Default)]
pub struct VirtioNetHdr {
    pub flags: u8,
    pub yvo: u8,
    pub ywv: u16,
    pub yvn: u16,
    pub yla: u16,
    pub ykz: u16,
}

impl VirtioNetHdr {
    pub const Am: usize = 10;
    
    pub fn new() -> Self {
        Self::default()
    }
}


#[repr(C)]
struct Ael {
    dh: VirtioNetHdr,
    f: [u8; 1514],  
}


#[repr(C)]
struct Bab {
    dh: VirtioNetHdr,
    f: [u8; 1514],
}


pub struct VirtioNet {
    
    de: VirtioDevice,
    
    ftl: Option<Box<Virtqueue>>,
    
    fxp: Option<Box<Virtqueue>>,
    
    ed: [u8; 6],
    
    aik: bool,
    
    dbm: Vec<Box<Ael>>,
    
    cnk: Vec<Box<Bab>>,
    
    ifi: VecDeque<u16>,
}


static Fl: Mutex<Option<VirtioNet>> = Mutex::new(None);
static Be: AtomicBool = AtomicBool::new(false);


static BEW_: Mutex<VecDeque<Vec<u8>>> = Mutex::new(VecDeque::new());


static AJO_: core::sync::atomic::AtomicU16 = core::sync::atomic::AtomicU16::new(0);

static AJP_: AtomicBool = AtomicBool::new(false);


static BCI_: core::sync::atomic::AtomicU64 = core::sync::atomic::AtomicU64::new(0);
static BCH_: core::sync::atomic::AtomicU64 = core::sync::atomic::AtomicU64::new(0);
static ANL_: core::sync::atomic::AtomicU64 = core::sync::atomic::AtomicU64::new(0);
static ANK_: core::sync::atomic::AtomicU64 = core::sync::atomic::AtomicU64::new(0);

impl VirtioNet {
    
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
        
        crate::log_debug!("[virtio-net] I/O base: {:#X}", agq);
        
        let mut de = VirtioDevice::new(agq);
        
        
        de.apa();
        
        
        de.fzu(status::Or);
        de.fzu(status::Fl);
        
        
        let bju = de.pab();
        crate::log_debug!("[virtio-net] Device features: {:#X}", bju);
        
        
        let mut ckb = 0u32;
        if bju & features::Bls != 0 {
            ckb |= features::Bls;
        }
        if bju & features::Nz != 0 {
            ckb |= features::Nz;
        }
        
        de.pzx(ckb);
        crate::log_debug!("[virtio-net] Driver features: {:#X}", ckb);
        
        
        let mut ed = [0u8; 6];
        for a in 0..6 {
            ed[a] = de.vrj(a as u16);
        }
        crate::log!("[virtio-net] MAC: {:02X}:{:02X}:{:02X}:{:02X}:{:02X}:{:02X}",
            ed[0], ed[1], ed[2], ed[3], ed[4], ed[5]);
        
        
        let aik = if ckb & features::Nz != 0 {
            let usg = de.vri(6);
            usg & 1 != 0
        } else {
            true 
        };
        
        crate::log!("[virtio-net] Link: {}", if aik { "UP" } else { "DOWN" });
        
        Ok(Self {
            de,
            ftl: None,
            fxp: None,
            ed,
            aik,
            dbm: Vec::new(),
            cnk: Vec::new(),
            ifi: VecDeque::new(),
        })
    }
    
    
    pub fn pjr(&mut self) -> Result<(), &'static str> {
        
        self.de.mdl(0);
        let mbl = self.de.kyw();
        crate::log_debug!("[virtio-net] RX queue size: {}", mbl);
        
        if mbl == 0 {
            return Err("RX queue not available");
        }
        
        let ftl = self.ijn(mbl)?;
        
        
        let duh = (ftl.ki / 4096) as u32;
        self.de.meu(duh);
        
        self.ftl = Some(ftl);
        
        
        self.de.mdl(1);
        let mno = self.de.kyw();
        crate::log_debug!("[virtio-net] TX queue size: {}", mno);
        
        if mno == 0 {
            return Err("TX queue not available");
        }
        
        let fxp = self.ijn(mno)?;
        
        let duh = (fxp.ki / 4096) as u32;
        self.de.meu(duh);
        
        self.fxp = Some(fxp);
        
        Ok(())
    }
    
    
    fn ijn(&mut self, aw: u16) -> Result<Box<Virtqueue>, &'static str> {
        let aay = Virtqueue::nbh(aw);
        
        
        
        let layout = core::alloc::Layout::bjy(aay, 4096)
            .jd(|_| "Layout error")?;
        
        let ptr = unsafe { alloc::alloc::alloc_zeroed(layout) };
        if ptr.abq() {
            return Err("Failed to allocate queue memory");
        }
        
        
        let rwc = ptr as *mut VirtqDesc;
        let fcw = core::mem::size_of::<VirtqDesc>() * aw as usize;
        let qlq = unsafe { ptr.add(fcw) as *mut virtio::Zk };
        let dxe = ((fcw + 6 + 2 * aw as usize) + 4095) & !4095;
        let xpt = unsafe { ptr.add(dxe) as *mut virtio::Zl };
        
        
        let vd = ptr as u64;
        let lr = crate::memory::lr();
        let ki = if vd >= lr {
            vd - lr
        } else {
            
            vd
        };
        
        
        let mut buk = vec![0u16; aw as usize];
        for a in 0..(aw - 1) {
            buk[a as usize] = a + 1;
        }
        buk[(aw - 1) as usize] = 0xFFFF; 
        
        Ok(Box::new(Virtqueue {
            aw,
            ki,
            desc: rwc,
            apk: qlq,
            mr: xpt,
            csa: 0,
            cyb: 0,
            dts: aw,
            buk,
        }))
    }
    
    
    pub fn pjs(&mut self) -> Result<(), &'static str> {
        let queue = self.ftl.as_mut().ok_or("RX queue not initialized")?;
        
        
        let orn = (queue.aw / 2).v(128) as usize; 
        
        for _ in 0..orn {
            let bi = Box::new(Ael {
                dh: VirtioNetHdr::new(),
                f: [0u8; 1514],
            });
            
            let and = queue.blx().ok_or("No free descriptors")?;
            
            
            let doo = &*bi as *const Ael;
            let vd = doo as u64;
            let hp = crate::memory::lr();
            let ki = if vd >= hp { vd - hp } else { vd };
            
            
            unsafe {
                let desc = &mut *queue.desc.add(and as usize);
                desc.ag = ki;
                desc.len = core::mem::size_of::<Ael>() as u32;
                desc.flags = desc_flags::Db; 
                desc.next = 0;
                
                
                queue.gxv(and);
            }
            
            self.dbm.push(bi);
        }
        
        crate::log_debug!("[virtio-net] {} RX buffers ready", orn);
        
        Ok(())
    }
    
    
    pub fn ay(&mut self) -> Result<(), &'static str> {
        
        self.de.fzu(status::HW_);
        
        
        self.de.jhj(0);
        
        crate::log!("[virtio-net] Device started");
        Ok(())
    }
    
    
    pub fn baq(&mut self, f: &[u8]) -> Result<(), &'static str> {
        if f.len() > 1514 {
            return Err("Packet too large");
        }
        
        
        self.own();
        
        let queue = self.fxp.as_mut().ok_or("TX queue not initialized")?;
        
        
        let mut bi = Box::new(Bab {
            dh: VirtioNetHdr::new(),
            f: [0u8; 1514],
        });
        
        
        bi.f[..f.len()].dg(f);
        
        
        let and = queue.blx().ok_or("TX queue full")?;
        
        
        let doo = &*bi as *const Bab;
        let vd = doo as u64;
        let hp = crate::memory::lr();
        let ki = if vd >= hp { vd - hp } else { vd };
        
        
        unsafe {
            let desc = &mut *queue.desc.add(and as usize);
            desc.ag = ki;
            desc.len = (VirtioNetHdr::Am + f.len()) as u32;
            desc.flags = 0; 
            desc.next = 0;
            
            queue.gxv(and);
        }
        
        
        self.cnk.push(bi);
        self.ifi.agt(and);
        
        
        self.de.jhj(1);
        
        
        BCI_.fetch_add(1, Ordering::Relaxed);
        ANL_.fetch_add(f.len() as u64, Ordering::Relaxed);
        
        Ok(())
    }
    
    
    pub fn owk(&mut self) -> Option<Vec<u8>> {
        let queue = self.ftl.as_mut()?;
        
        unsafe {
            if !queue.ixy() {
                return None;
            }
            
            let mr = queue.jjp()?;
            
            
            let and = mr.0 as u16;
            let desc = &*queue.desc.add(and as usize);
            
            
            let doo = (crate::memory::lr() + desc.ag) as *const Ael;
            let bi = &*doo;
            
            let cwv = (mr.1 as usize).ao(VirtioNetHdr::Am);
            if cwv > 0 && cwv <= 1514 {
                let ex = bi.f[..cwv].ip();
                
                
                queue.gxv(and);
                self.de.jhj(0);
                
                
                BCH_.fetch_add(1, Ordering::Relaxed);
                ANK_.fetch_add(cwv as u64, Ordering::Relaxed);
                
                return Some(ex);
            }
            
            
            queue.gxv(and);
            self.de.jhj(0);
        }
        
        None
    }
    
    
    fn own(&mut self) {
        let queue = match self.fxp.as_mut() {
            Some(fm) => fm,
            None => return,
        };
        
        unsafe {
            while queue.ixy() {
                if let Some(mr) = queue.jjp() {
                    
                    queue.ald(mr.0 as u16);
                    
                    
                    if let Some(u) = self.ifi.iter().qf(|&b| b == mr.0 as u16) {
                        self.ifi.remove(u);
                    }
                }
            }
        }
        
        
        while self.cnk.len() > 16 && !self.ifi.is_empty() {
            self.cnk.remove(0);
        }
    }
    
    
    pub fn owm(&mut self) {
        self.own();
    }
    
    
    pub fn ed(&self) -> [u8; 6] {
        self.ed
    }
    
    
    pub fn txy(&self) -> bool {
        self.aik
    }
    
    
    pub fn agq(&self) -> u16 {
        self.de.agq
    }
}




pub fn init(sq: &S) -> Result<(), &'static str> {
    crate::log!("[virtio-net] Initializing...");
    
    let mut rj = VirtioNet::new(sq)?;
    rj.pjr()?;
    rj.pjs()?;
    rj.ay()?;
    
    
    AJO_.store(rj.de.agq, Ordering::SeqCst);
    
    Be.store(true, Ordering::SeqCst);
    *Fl.lock() = Some(rj);
    
    
    let irq = sq.esw;
    if irq > 0 && irq < 255 {
        crate::apic::jmw(irq, crate::apic::HH_);
        crate::serial_println!("[virtio-net] IRQ {} routed to vector {}", irq, crate::apic::HH_);
    }
    
    Ok(())
}


pub fn ky() -> bool {
    Be.load(Ordering::Relaxed)
}


pub fn cez() -> Option<[u8; 6]> {
    Fl.lock().as_ref().map(|bc| bc.ed())
}


pub fn blc(f: &[u8]) -> Result<(), &'static str> {
    let mut rj = Fl.lock();
    let ane = rj.as_mut().ok_or("Driver not initialized")?;
    ane.baq(f)
}


pub fn poll() {
    
    AJP_.store(false, Ordering::Relaxed);
    
    let mut rj = Fl.lock();
    if let Some(ane) = rj.as_mut() {
        
        ane.owm();
        
        
        while let Some(ex) = ane.owk() {
            BEW_.lock().agt(ex);
        }
    }
}


pub fn pao() -> Option<Vec<u8>> {
    poll(); 
    BEW_.lock().awp()
}


pub fn asx() -> (u64, u64, u64, u64) {
    (
        BCI_.load(Ordering::Relaxed),
        BCH_.load(Ordering::Relaxed),
        ANL_.load(Ordering::Relaxed),
        ANK_.load(Ordering::Relaxed),
    )
}



pub fn eck() {
    let agq = AJO_.load(Ordering::Relaxed);
    if agq == 0 { return; }
    
    
    let cru: u8 = unsafe {
        let mut port = Port::<u8>::new(agq + 0x13);
        port.read()
    };
    
    if cru & 1 != 0 {
        
        AJP_.store(true, Ordering::Release);
    }
}


pub fn jay() -> bool {
    AJP_.load(Ordering::Relaxed)
}



pub fn wjc(agq: u16) {
    AJO_.store(agq, Ordering::SeqCst);
}




pub fn tjx() {
    eck();
}
