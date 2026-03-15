











use alloc::string::String;
use alloc::vec::Vec;
use core::sync::atomic::{AtomicBool, Ordering};
use spin::Mutex;





const BEB_: u64 = 0x00;     
const CON_: u64 = 0x08;      
const CNT_: u64 = 0x0C;   
const ECF_: u64 = 0x10;   
const AHB_: u64 = 0x14;      
const BEC_: u64 = 0x1C;    
const CNP_: u64 = 0x24;     
const BEA_: u64 = 0x28;     
const BDZ_: u64 = 0x30;     


const AQO_: u64 = 0x1000;


const ZW_: u32 = 1 << 0;         
const BLY_: u32 = 0 << 4;    
const BMC_: u32 = 0 << 7;     
const BLX_: u32 = 0 << 11;    
const BMB_: u32 = 6 << 16;    
const BMA_: u32 = 4 << 20;    


const APN_: u32 = 1 << 0;      
const BQG_: u32 = 1 << 1;      






const DCG_: u8 = 0x00;
const BJK_: u8 = 0x01;
const DCF_: u8 = 0x04;
const BJJ_: u8 = 0x05;
const YW_: u8 = 0x06;
const DCH_: u8 = 0x09;


const CCH_: u8 = 0x00;
const CCK_: u8 = 0x01;
const CCJ_: u8 = 0x02;


const CBJ_: u32 = 0x00;
const CBI_: u32 = 0x01;
const CBH_: u32 = 0x02;






#[derive(Clone, Copy, Default)]
#[repr(C)]
struct Ia {
    
    dot: u32,
    
    bvp: u32,
    
    yhr: u32,
    yhs: u32,
    
    zdd: u64,
    
    frm: u64,
    
    jkj: u64,
    
    fen: u32,
    ina: u32,
    nbw: u32,
    yho: u32,
    yhp: u32,
    yhq: u32,
}

const _: () = assert!(core::mem::size_of::<Ia>() == 64);


#[derive(Clone, Copy, Default)]
#[repr(C)]
struct Rx {
    
    sho: u32,
    
    epl: u32,
    
    zph: u32,
    
    khr: u32,
}

const _: () = assert!(core::mem::size_of::<Rx>() == 16);

impl Rx {
    fn ib(&self) -> bool {
        self.khr & (1 << 16) != 0
    }

    fn wt(&self) -> u16 {
        ((self.khr >> 17) & 0x7FF) as u16
    }

    fn yjo(&self) -> u16 {
        (self.khr & 0xFFFF) as u16
    }
}


struct QueuePair {
    
    mgz: u64,
    
    eiu: u64,
    
    kls: u64,
    
    dzy: u64,
    
    eo: u16,
    
    eyx: u16,
    
    dzx: u16,
    
    ipn: bool,
    
    jgw: u16,
    
    dus: u16,
}

impl QueuePair {
    
    fn new(dus: u16, eo: u16) -> Option<Self> {
        
        let eiu = crate::memory::frame::azg()?;
        let mgz = crate::memory::auv(eiu);
        
        
        let dzy = crate::memory::frame::azg()?;
        let kls = crate::memory::auv(dzy);
        
        Some(Self {
            mgz,
            eiu,
            kls,
            dzy,
            eo,
            eyx: 0,
            dzx: 0,
            ipn: true,     
            jgw: 0,
            dus,
        })
    }
    
    
    fn dmd(&mut self, mut cmd: Ia) -> u16 {
        let ncz = self.jgw;
        self.jgw = self.jgw.cn(1);
        
        
        cmd.dot = (cmd.dot & 0x0000FFFF) | ((ncz as u32) << 16);
        
        
        let l = self.eyx as usize * core::mem::size_of::<Ia>();
        unsafe {
            let ptr = (self.mgz + l as u64) as *mut Ia;
            core::ptr::write_volatile(ptr, cmd);
        }
        
        
        self.eyx = (self.eyx + 1) % self.eo;
        
        ncz
    }
    
    
    fn owi(&mut self) -> Option<Rx> {
        let l = self.dzx as usize * core::mem::size_of::<Rx>();
        let bt = unsafe {
            let ptr = (self.kls + l as u64) as *const Rx;
            core::ptr::read_volatile(ptr)
        };
        
        
        if bt.ib() == self.ipn {
            
            self.dzx += 1;
            if self.dzx >= self.eo {
                self.dzx = 0;
                self.ipn = !self.ipn;
            }
            Some(bt)
        } else {
            None
        }
    }
}


#[derive(Clone)]
pub struct Awj {
    
    pub bvp: u32,
    
    pub mfy: u64,
    
    pub bni: u32,
}


struct Awi {
    
    bly: u64,
    
    irl: u32,
    
    cvm: QueuePair,
    
    io: Option<QueuePair>,
    
    serial: String,
    
    model: String,
    
    fpg: u64,
    
    fpf: u32,
    
    evd: Vec<Awj>,
    
    omg: u32,
}

impl Awi {
    
    
    #[inline]
    fn amp(&self, l: u64) -> u32 {
        unsafe { core::ptr::read_volatile((self.bly + l) as *const u32) }
    }
    
    #[inline]
    fn aiu(&self, l: u64, bn: u32) {
        unsafe { core::ptr::write_volatile((self.bly + l) as *mut u32, bn) }
    }
    
    #[inline]
    fn zhi(&self, l: u64) -> u64 {
        let hh = self.amp(l) as u64;
        let gd = self.amp(l + 4) as u64;
        hh | (gd << 32)
    }
    
    #[inline]
    fn jxe(&self, l: u64, bn: u64) {
        self.aiu(l, bn as u32);
        self.aiu(l + 4, (bn >> 32) as u32);
    }
    
    
    
    
    fn pdk(&self, dus: u16, utu: u16) {
        let l = AQO_ + (2 * dus as u64) * self.irl as u64;
        self.aiu(l, utu as u32);
    }
    
    
    fn pdh(&self, dus: u16, efs: u16) {
        let l = AQO_ + (2 * dus as u64 + 1) * self.irl as u64;
        self.aiu(l, efs as u32);
    }
    
    
    
    
    fn gyc(&mut self, cmd: Ia) -> Result<Rx, &'static str> {
        let qbn = self.cvm.dmd(cmd);
        self.pdk(0, self.cvm.eyx);
        
        
        for _ in 0..1_000_000u32 {
            if let Some(ffz) = self.cvm.owi() {
                
                self.pdh(0, self.cvm.dzx);
                
                if ffz.wt() != 0 {
                    crate::serial_println!("[NVMe] Admin cmd failed: status={:#x}", ffz.wt());
                    return Err("NVMe admin command failed");
                }
                return Ok(ffz);
            }
            core::hint::hc();
        }
        Err("NVMe admin command timeout")
    }
    
    
    fn lfl(&mut self, cmd: Ia) -> Result<Rx, &'static str> {
        
        let eyx = {
            let io = self.io.as_mut().ok_or("NVMe I/O queue not initialized")?;
            let qbn = io.dmd(cmd);
            io.eyx
        };
        self.pdk(1, eyx);
        
        
        for _ in 0..10_000_000u32 {
            let io = self.io.as_mut().unwrap();
            if let Some(ffz) = io.owi() {
                let dzx = io.dzx;
                self.pdh(1, dzx);
                
                if ffz.wt() != 0 {
                    crate::serial_println!("[NVMe] I/O cmd failed: status={:#x}", ffz.wt());
                    return Err("NVMe I/O command failed");
                }
                return Ok(ffz);
            }
            core::hint::hc();
        }
        Err("NVMe I/O command timeout")
    }
    
    
    
    
    fn trj(&mut self) -> Result<(), &'static str> {
        let rg = crate::memory::frame::azg()
            .ok_or("NVMe: OOM for identify buffer")?;
        let aak = crate::memory::auv(rg);
        
        let cmd = Ia {
            dot: YW_ as u32,
            frm: rg,
            fen: CBI_,
            ..Default::default()
        };
        
        self.gyc(cmd)?;
        
        
        unsafe {
            let f = aak as *const u8;
            
            
            let mut plr = [0u8; 20];
            core::ptr::copy_nonoverlapping(f.add(4), plr.mw(), 20);
            self.serial = core::str::jg(&plr)
                .unwrap_or("?")
                .em()
                .into();
            
            
            let mut brk = [0u8; 40];
            core::ptr::copy_nonoverlapping(f.add(24), brk.mw(), 40);
            self.model = core::str::jg(&brk)
                .unwrap_or("?")
                .em()
                .into();
            
            
            
            let omn = *f.add(77);
            self.omg = if omn == 0 { 256 } else { 1u32 << omn };
        }
        
        crate::memory::frame::apt(rg);
        Ok(())
    }
    
    
    fn odc(&mut self, bvp: u32) -> Result<(u64, u32), &'static str> {
        let rg = crate::memory::frame::azg()
            .ok_or("NVMe: OOM for identify namespace buffer")?;
        let aak = crate::memory::auv(rg);
        
        let cmd = Ia {
            dot: YW_ as u32,
            bvp,
            frm: rg,
            fen: CBJ_,
            ..Default::default()
        };
        
        self.gyc(cmd)?;
        
        let (djz, bni) = unsafe {
            let f = aak as *const u8;
            
            
            let djz = core::ptr::md(f as *const u64);
            
            
            let sus = *f.add(26) & 0x0F;
            
            
            
            let uds = 128 + (sus as usize) * 4;
            let udr = core::ptr::md(f.add(uds) as *const u32);
            let udq = (udr >> 16) & 0xFF;
            let cak = 1u32 << udq;
            
            (djz, cak)
        };
        
        crate::memory::frame::apt(rg);
        Ok((djz, bni))
    }
    
    
    fn yxk(&mut self) -> Result<(), &'static str> {
        let (djz, bni) = self.odc(1)?;
        self.fpg = djz;
        self.fpf = bni;
        Ok(())
    }
    
    
    
    fn smk(&mut self) -> Result<(), &'static str> {
        let rg = crate::memory::frame::azg()
            .ok_or("NVMe: OOM for NS list buffer")?;
        let aak = crate::memory::auv(rg);
        
        let cmd = Ia {
            dot: YW_ as u32,
            bvp: 0, 
            frm: rg,
            fen: CBH_,
            ..Default::default()
        };
        
        let lpe: Vec<u32>;
        
        match self.gyc(cmd) {
            Ok(_) => {
                
                
                let mut esg = Vec::new();
                unsafe {
                    let aoy = aak as *const u32;
                    for a in 0..1024 {
                        let bvp = core::ptr::read_volatile(aoy.add(a));
                        if bvp == 0 { break; }
                        esg.push(bvp);
                    }
                }
                lpe = esg;
            }
            Err(_) => {
                
                crate::serial_println!("[NVMe] Active NSID list not supported, using NS1 only");
                lpe = alloc::vec![1];
            }
        }
        
        crate::memory::frame::apt(rg);
        
        self.evd.clear();
        
        for &bvp in &lpe {
            match self.odc(bvp) {
                Ok((djz, bni)) => {
                    if djz > 0 {
                        let aga = (djz * bni as u64) / (1024 * 1024);
                        crate::serial_println!("[NVMe] NS{}: {} LBAs x {} B = {} MB",
                            bvp, djz, bni, aga);
                        self.evd.push(Awj {
                            bvp,
                            mfy: djz,
                            bni,
                        });
                        
                        if bvp == 1 {
                            self.fpg = djz;
                            self.fpf = bni;
                        }
                    }
                }
                Err(aa) => {
                    crate::serial_println!("[NVMe] Failed to identify NS{}: {}", bvp, aa);
                }
            }
        }
        
        if self.evd.is_empty() {
            return Err("NVMe: no usable namespaces found");
        }
        
        Ok(())
    }
    
    
    
    
    fn rqo(&mut self, dus: u16, dzy: u64, eo: u16) -> Result<(), &'static str> {
        let cmd = Ia {
            dot: BJJ_ as u32,
            frm: dzy,
            
            fen: (dus as u32) | (((eo - 1) as u32) << 16),
            
            ina: 1,   
            ..Default::default()
        };
        
        self.gyc(cmd)?;
        Ok(())
    }
    
    
    fn rqp(&mut self, dus: u16, eiu: u64, rqe: u16, eo: u16) -> Result<(), &'static str> {
        let cmd = Ia {
            dot: BJK_ as u32,
            frm: eiu,
            
            fen: (dus as u32) | (((eo - 1) as u32) << 16),
            
            ina: 1 | ((rqe as u32) << 16),
            ..Default::default()
        };
        
        self.gyc(cmd)?;
        Ok(())
    }
    
    
    
    
    
    
    fn nap(&self, bcd: &[u64]) -> Result<(u64, Option<u64>), &'static str> {
        if bcd.len() <= 1 {
            
            Ok((0, None))
        } else if bcd.len() == 2 {
            
            Ok((bcd[1], None))
        } else {
            
            let jds = crate::memory::frame::azg()
                .ok_or("NVMe: OOM for PRP list")?;
            let ufy = crate::memory::auv(jds);
            
            let ia = bcd.len() - 1; 
            if ia > 512 {
                crate::memory::frame::apt(jds);
                return Err("NVMe: transfer too large for single PRP list");
            }
            
            unsafe {
                let ch = ufy as *mut u64;
                for a in 0..ia {
                    core::ptr::write_volatile(ch.add(a), bcd[a + 1]);
                }
            }
            
            Ok((jds, Some(jds)))
        }
    }
    
    
    fn vrx(&mut self, aag: u64, az: u16, bcd: &[u64]) -> Result<(), &'static str> {
        let (jkj, lvz) = self.nap(bcd)?;
        
        let cmd = Ia {
            dot: CCJ_ as u32,
            bvp: 1,
            frm: bcd[0],
            jkj,
            fen: aag as u32,
            ina: (aag >> 32) as u32,
            nbw: (az - 1) as u32,
            ..Default::default()
        };
        
        let result = self.lfl(cmd);
        if let Some(ht) = lvz {
            crate::memory::frame::apt(ht);
        }
        result?;
        Ok(())
    }
    
    
    fn xvm(&mut self, aag: u64, az: u16, bcd: &[u64]) -> Result<(), &'static str> {
        let (jkj, lvz) = self.nap(bcd)?;
        
        let cmd = Ia {
            dot: CCK_ as u32,
            bvp: 1,
            frm: bcd[0],
            jkj,
            fen: aag as u32,
            ina: (aag >> 32) as u32,
            nbw: (az - 1) as u32,
            ..Default::default()
        };
        
        let result = self.lfl(cmd);
        if let Some(ht) = lvz {
            crate::memory::frame::apt(ht);
        }
        result?;
        Ok(())
    }
    
    
    fn hjx(&mut self) -> Result<(), &'static str> {
        let cmd = Ia {
            dot: CCH_ as u32,
            bvp: 1,
            ..Default::default()
        };
        self.lfl(cmd)?;
        Ok(())
    }
}





static Qb: Mutex<Option<Awi>> = Mutex::new(None);
static Be: AtomicBool = AtomicBool::new(false);


pub fn ky() -> bool {
    Be.load(Ordering::Relaxed)
}


pub fn aty() -> u64 {
    Qb.lock().as_ref().map(|r| r.fpg).unwrap_or(0)
}


pub fn bni() -> u32 {
    Qb.lock().as_ref().map(|r| r.fpf).unwrap_or(512)
}


pub fn ani() -> Option<(String, String, u64, u32)> {
    let db = Qb.lock();
    let r = db.as_ref()?;
    Some((r.model.clone(), r.serial.clone(), r.fpg, r.fpf))
}


pub fn ufs() -> Vec<Awj> {
    Qb.lock().as_ref().map(|r| r.evd.clone()).age()
}















pub fn init(sq: &crate::pci::S) -> Result<(), &'static str> {
    crate::serial_println!("[NVMe] Initializing {:02X}:{:02X}.{} ({:04X}:{:04X})",
        sq.aq, sq.de, sq.gw,
        sq.ml, sq.mx);
    
    
    crate::pci::fhp(sq);
    crate::pci::fhq(sq);
    
    
    let cmd = crate::pci::byw(sq.aq, sq.de, sq.gw, 0x04);
    crate::pci::aso(sq.aq, sq.de, sq.gw, 0x04,
        (cmd | (1 << 10)) as u32); 
    
    
    let fcz = sq.cje(0).ok_or("NVMe: no BAR0")?;
    if fcz == 0 {
        return Err("NVMe: BAR0 is zero");
    }
    
    
    let bly = crate::memory::bki(fcz, 0x10000)?;
    
    crate::serial_println!("[NVMe] BAR0: phys={:#x}, virt={:#x}", fcz, bly);
    
    
    let mh = {
        let hh = unsafe { core::ptr::read_volatile((bly + BEB_) as *const u32) } as u64;
        let gd = unsafe { core::ptr::read_volatile((bly + BEB_ + 4) as *const u32) } as u64;
        hh | (gd << 32)
    };
    
    let ood = (mh & 0xFFFF) as u16 + 1;  
    let noe = ((mh >> 32) & 0xF) as u32; 
    let irl = 4u32 << noe;
    let uqc = ((mh >> 48) & 0xF) as u32; 
    let xhd = ((mh >> 24) & 0xFF) as u32; 
    
    let gwl = unsafe { core::ptr::read_volatile((bly + CON_) as *const u32) };
    let efb = (gwl >> 16) & 0xFFFF;
    let efm = (gwl >> 8) & 0xFF;
    
    crate::serial_println!("[NVMe] Version: {}.{}, MQES={}, DSTRD={}, MPS_MIN={}KB, Timeout={}ms",
        efb, efm, ood, noe, 4 << uqc, xhd * 500);
    
    
    let hwi = ood.v(64) as u16;
    
    
    let nn = unsafe { core::ptr::read_volatile((bly + AHB_) as *const u32) };
    if nn & ZW_ != 0 {
        
        unsafe { core::ptr::write_volatile((bly + AHB_) as *mut u32, nn & !ZW_) };
        
        
        for _ in 0..1_000_000u32 {
            let ipt = unsafe { core::ptr::read_volatile((bly + BEC_) as *const u32) };
            if ipt & APN_ == 0 {
                break;
            }
            core::hint::hc();
        }
    }
    
    
    let cvm = QueuePair::new(0, hwi)
        .ok_or("NVMe: OOM for admin queues")?;
    
    crate::serial_println!("[NVMe] Admin SQ phys={:#x}, CQ phys={:#x}, depth={}",
        cvm.eiu, cvm.dzy, hwi);
    
    
    
    let qkd = ((hwi - 1) as u32) | (((hwi - 1) as u32) << 16);
    unsafe {
        core::ptr::write_volatile((bly + CNP_) as *mut u32, qkd);
        
        core::ptr::write_volatile((bly + BEA_) as *mut u32, cvm.eiu as u32);
        core::ptr::write_volatile((bly + BEA_ + 4) as *mut u32, (cvm.eiu >> 32) as u32);
        
        core::ptr::write_volatile((bly + BDZ_) as *mut u32, cvm.dzy as u32);
        core::ptr::write_volatile((bly + BDZ_ + 4) as *mut u32, (cvm.dzy >> 32) as u32);
    }
    
    
    unsafe {
        core::ptr::write_volatile((bly + CNT_) as *mut u32, 0xFFFFFFFF);
    }
    
    
    let qxb = ZW_ | BLY_ | BMC_ | BLX_ | BMB_ | BMA_;
    unsafe {
        core::ptr::write_volatile((bly + AHB_) as *mut u32, qxb);
    }
    
    
    let mut ack = false;
    for _ in 0..5_000_000u32 {
        let ipt = unsafe { core::ptr::read_volatile((bly + BEC_) as *const u32) };
        if ipt & BQG_ != 0 {
            return Err("NVMe: Controller Fatal Status during enable");
        }
        if ipt & APN_ != 0 {
            ack = true;
            break;
        }
        core::hint::hc();
    }
    
    if !ack {
        return Err("NVMe: Controller did not become ready");
    }
    
    crate::serial_println!("[NVMe] Controller enabled and ready");
    
    
    let mut db = Awi {
        bly,
        irl,
        cvm,
        io: None,
        serial: String::new(),
        model: String::new(),
        fpg: 0,
        fpf: 512,
        evd: Vec::new(),
        omg: 256,
    };
    
    
    db.trj()?;
    crate::serial_println!("[NVMe] Model: '{}', Serial: '{}'", db.model, db.serial);
    
    
    db.smk()?;
    let jtt: u64 = db.evd.iter()
        .map(|csw| (csw.mfy * csw.bni as u64) / (1024 * 1024))
        .sum();
    crate::serial_println!("[NVMe] {} namespace(s), total {} MB", db.evd.len(), jtt);
    
    
    let jas = hwi;
    let lfm = QueuePair::new(1, jas)
        .ok_or("NVMe: OOM for I/O queues")?;
    
    db.rqo(1, lfm.dzy, jas)?;
    db.rqp(1, lfm.eiu, 1, jas)?;
    db.io = Some(lfm);
    
    crate::serial_println!("[NVMe] I/O queue pair created (depth={})", jas);
    
    
    let uvw = db.evd.len();
    *Qb.lock() = Some(db);
    Be.store(true, Ordering::Release);
    
    crate::serial_println!("[NVMe] ✓ Driver initialized — {} namespace(s), {} MB NVMe storage available",
        uvw, jtt);
    
    Ok(())
}










pub fn ain(aag: u64, az: usize, bi: &mut [u8]) -> Result<(), &'static str> {
    let mut db = Qb.lock();
    let db = db.as_mut().ok_or("NVMe: not initialized")?;
    
    let cak = db.fpf as usize;
    let xv = az * cak;
    
    if bi.len() < xv {
        return Err("NVMe: buffer too small");
    }
    
    if aag + az as u64 > db.fpg {
        return Err("NVMe: read past end of namespace");
    }
    
    
    
    let llc = 128usize;
    let lkt = llc * 4096;
    let lkz = (lkt / cak).am(1);
    
    let mut qa = aag;
    let mut l = 0usize;
    let mut ia = az;
    
    while ia > 0 {
        let jj = ia.v(lkz);
        let dov = jj * cak;
        let duc = (dov + 4095) / 4096;
        
        
        let mut dgl: Vec<u64> = Vec::fc(duc);
        for _ in 0..duc {
            match crate::memory::frame::azg() {
                Some(ht) => dgl.push(ht),
                None => {
                    
                    for ai in &dgl { crate::memory::frame::apt(*ai); }
                    return Err("NVMe: OOM for DMA read buffer");
                }
            }
        }
        
        
        db.vrx(qa, jj as u16, &dgl)?;
        
        
        let mut hbz = dov;
        for (a, &dai) in dgl.iter().cf() {
            let hdz = hbz.v(4096);
            let ju = crate::memory::auv(dai);
            unsafe {
                core::ptr::copy_nonoverlapping(
                    ju as *const u8,
                    bi[l + a * 4096..].mw(),
                    hdz,
                );
            }
            hbz -= hdz;
        }
        
        
        for ai in &dgl { crate::memory::frame::apt(*ai); }
        
        qa += jj as u64;
        l += dov;
        ia -= jj;
    }
    
    Ok(())
}


pub fn bpi(aag: u64, az: usize, bi: &[u8]) -> Result<(), &'static str> {
    let mut db = Qb.lock();
    let db = db.as_mut().ok_or("NVMe: not initialized")?;
    
    let cak = db.fpf as usize;
    let xv = az * cak;
    
    if bi.len() < xv {
        return Err("NVMe: buffer too small");
    }
    
    if aag + az as u64 > db.fpg {
        return Err("NVMe: write past end of namespace");
    }
    
    let llc = 128usize;
    let lkt = llc * 4096;
    let lkz = (lkt / cak).am(1);
    
    let mut qa = aag;
    let mut l = 0usize;
    let mut ia = az;
    
    while ia > 0 {
        let jj = ia.v(lkz);
        let dov = jj * cak;
        let duc = (dov + 4095) / 4096;
        
        
        let mut dgl: Vec<u64> = Vec::fc(duc);
        for _ in 0..duc {
            match crate::memory::frame::azg() {
                Some(ht) => dgl.push(ht),
                None => {
                    for ai in &dgl { crate::memory::frame::apt(*ai); }
                    return Err("NVMe: OOM for DMA write buffer");
                }
            }
        }
        
        
        let mut hbz = dov;
        for (a, &dai) in dgl.iter().cf() {
            let hdz = hbz.v(4096);
            let ju = crate::memory::auv(dai);
            unsafe {
                core::ptr::copy_nonoverlapping(
                    bi[l + a * 4096..].fq(),
                    ju as *mut u8,
                    hdz,
                );
            }
            hbz -= hdz;
        }
        
        
        db.xvm(qa, jj as u16, &dgl)?;
        
        
        for ai in &dgl { crate::memory::frame::apt(*ai); }
        
        qa += jj as u64;
        l += dov;
        ia -= jj;
    }
    
    Ok(())
}


pub fn hjx() -> Result<(), &'static str> {
    let mut db = Qb.lock();
    let db = db.as_mut().ok_or("NVMe: not initialized")?;
    db.hjx()
}
