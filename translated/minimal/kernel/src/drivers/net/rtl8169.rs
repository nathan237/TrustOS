











use alloc::boxed::Box;
use alloc::vec::Vec;
use alloc::vec;
use core::ptr::{read_volatile, write_volatile};
use core::sync::atomic::{AtomicBool, AtomicU64, Ordering};

use super::{Ha, NetStats};
use crate::drivers::{Gi, Co, DriverStatus, DriverCategory};
use crate::pci::S;





const CNU_: u32       = 0x00;  
const CNV_: u32       = 0x04;  
const COJ_: u32      = 0x20;  
const COK_: u32   = 0x24;  
const WO_: u32         = 0x37;  
const COL_: u32      = 0x38;  
const BEE_: u32         = 0x3C;  
const BEF_: u32         = 0x3E;  
const COM_: u32   = 0x40;  
const AHD_: u32   = 0x44;  
const ECG_: u32         = 0x4C;  
const BDY_: u32      = 0x50;  
const ECD_: u32     = 0x52;  
const WP_: u32  = 0x6C;  
const COD_: u32 = 0xDA;  
const CNQ_: u32        = 0xE0;  
const COB_: u32       = 0xE4;  
const COC_: u32    = 0xE8;  
const CNR_: u32 = 0xEC; 






const AOA_: u8 = 0x10;
const BMQ_: u8 = 0x08;
const BMR_: u8 = 0x04;


const CYH_: u8 = 0x40;  


const CBW_: u16 = 0x0001;    
const CBY_: u16 = 0x0004;    
const AXD_: u16 = 0x0020; 
const CBX_: u16 = 0x0010;
const CBV_: u16 = CBW_ | CBY_ | AXD_ | CBX_;


const CZB_: u32 = 0x03 << 24;  
const CZA_: u32 = 0x07 << 8; 


const BEV_: u32 = 1 << 0;   
const CPW_: u32 = 1 << 1;   
const CPV_: u32  = 1 << 2;   
const CPU_: u32  = 1 << 3;   
const EDJ_: u32 = 1 << 7;  
const CPX_: u32 = 0x07 << 8; 
const CPY_: u32 = 0x07 << 13; 


const DGX_: u16 = 1 << 6;
const BOW_: u16 = 1 << 5;
const BOV_: u16 = 1 << 3;


const BME_: u8 = 0xC0;
const BMD_: u8   = 0x00;


const AGL_: u32 = 0x02;
const BDB_: u32 = 0x10;
const BDC_: u32 = 0x08;
const CJL_: u32 = 0x04;





const CB_: usize = 64;
const CR_: usize = 64;
const DN_: usize = 2048;


const MS_: u32  = 1 << 31;  
const SO_: u32  = 1 << 30;  
const AAZ_: u32   = 1 << 29;  
const ABA_: u32   = 1 << 28;  


#[repr(C, align(16))]
#[derive(Clone, Copy)]
struct Descriptor {
    dkg: u32,  
    htt: u32,  
    imh: u32, 
    img: u32, 
}

impl Default for Descriptor {
    fn default() -> Self {
        Self {
            dkg: 0,
            htt: 0,
            imh: 0,
            img: 0,
        }
    }
}





pub struct Rtl8169Driver {
    status: DriverStatus,
    hv: u64,
    ed: [u8; 6],

    
    bcl: Vec<Descriptor>,
    bfn: Vec<Descriptor>,
    dbm: Vec<Vec<u8>>,
    cnk: Vec<Vec<u8>>,

    
    chm: usize,
    ddb: usize,

    
    cuz: AtomicU64,
    dbo: AtomicU64,
    bpc: AtomicU64,
    bsc: AtomicU64,
    dmv: AtomicU64,
    dbn: AtomicU64,

    
    aik: AtomicBool,
    jr: AtomicBool,
}

impl Rtl8169Driver {
    pub fn new() -> Self {
        Self {
            status: DriverStatus::Aff,
            hv: 0,
            ed: [0x52, 0x54, 0x00, 0x81, 0x69, 0x00],
            bcl: Vec::new(),
            bfn: Vec::new(),
            dbm: Vec::new(),
            cnk: Vec::new(),
            chm: 0,
            ddb: 0,
            cuz: AtomicU64::new(0),
            dbo: AtomicU64::new(0),
            bpc: AtomicU64::new(0),
            bsc: AtomicU64::new(0),
            dmv: AtomicU64::new(0),
            dbn: AtomicU64::new(0),
            aik: AtomicBool::new(false),
            jr: AtomicBool::new(false),
        }
    }

    

    fn akm(&self, l: u32) -> u8 {
        if self.hv == 0 { return 0; }
        let ag = (self.hv + l as u64) as *const u8;
        unsafe { read_volatile(ag) }
    }

    fn akw(&self, l: u32, ap: u8) {
        if self.hv == 0 { return; }
        let ag = (self.hv + l as u64) as *mut u8;
        unsafe { write_volatile(ag, ap); }
    }

    fn aym(&self, l: u32) -> u16 {
        if self.hv == 0 { return 0; }
        let ag = (self.hv + l as u64) as *const u16;
        unsafe { read_volatile(ag) }
    }

    fn asg(&self, l: u32, ap: u16) {
        if self.hv == 0 { return; }
        let ag = (self.hv + l as u64) as *mut u16;
        unsafe { write_volatile(ag, ap); }
    }

    fn amp(&self, l: u32) -> u32 {
        if self.hv == 0 { return 0; }
        let ag = (self.hv + l as u64) as *const u32;
        unsafe { read_volatile(ag) }
    }

    fn aiu(&self, l: u32, ap: u32) {
        if self.hv == 0 { return; }
        let ag = (self.hv + l as u64) as *mut u32;
        unsafe { write_volatile(ag, ap); }
    }

    
    fn abw(ju: u64) -> u64 {
        const DS_: u64 = 0xFFFF_8000_0000_0000;
        if ju >= DS_ { ju - DS_ } else { ju }
    }

    
    fn apa(&self) {
        crate::log_debug!("[RTL8169] Resetting controller...");

        self.akw(WO_, AOA_);

        
        for _ in 0..10_000 {
            if self.akm(WO_) & AOA_ == 0 {
                crate::log_debug!("[RTL8169] Reset complete");
                return;
            }
            for _ in 0..1000 { core::hint::hc(); }
        }

        crate::log_warn!("[RTL8169] Reset timeout — continuing anyway");
    }

    
    fn lxr(&mut self) {
        let hh = self.amp(CNU_);
        let gd = self.amp(CNV_);

        self.ed[0] = (hh >> 0) as u8;
        self.ed[1] = (hh >> 8) as u8;
        self.ed[2] = (hh >> 16) as u8;
        self.ed[3] = (hh >> 24) as u8;
        self.ed[4] = (gd >> 0) as u8;
        self.ed[5] = (gd >> 8) as u8;

        
        if self.ed == [0; 6] {
            self.ed = [0x52, 0x54, 0x00, 0x12, 0x81, 0x69];
        }
    }

    
    fn xoh(&self) {
        self.akw(BDY_, BME_);
    }

    
    fn uhu(&self) {
        self.akw(BDY_, BMD_);
    }

    
    fn leg(&mut self) {
        crate::log_debug!("[RTL8169] Initializing RX ring ({} descriptors)", CB_);

        self.bcl = vec![Descriptor::default(); CB_];
        self.dbm = Vec::fc(CB_);

        for a in 0..CB_ {
            let bi = vec![0u8; DN_];
            let ht = Self::abw(bi.fq() as u64);

            let mut flags = MS_ | (DN_ as u32 & 0x3FFF);
            if a == CB_ - 1 {
                flags |= SO_; 
            }

            self.bcl[a].dkg = flags;
            self.bcl[a].htt = 0;
            self.bcl[a].imh = ht as u32;
            self.bcl[a].img = (ht >> 32) as u32;

            self.dbm.push(bi);
        }

        
        let bhy = Self::abw(self.bcl.fq() as u64);
        self.aiu(COB_, bhy as u32);
        self.aiu(COC_, (bhy >> 32) as u32);

        self.chm = 0;
    }

    
    fn lei(&mut self) {
        crate::log_debug!("[RTL8169] Initializing TX ring ({} descriptors)", CR_);

        self.bfn = vec![Descriptor::default(); CR_];
        self.cnk = Vec::fc(CR_);

        for a in 0..CR_ {
            let bi = vec![0u8; DN_];

            let mut flags = 0u32;
            if a == CR_ - 1 {
                flags |= SO_; 
            }

            self.bfn[a].dkg = flags;
            self.bfn[a].htt = 0;

            let ht = Self::abw(bi.fq() as u64);
            self.bfn[a].imh = ht as u32;
            self.bfn[a].img = (ht >> 32) as u32;

            self.cnk.push(bi);
        }

        
        let bhy = Self::abw(self.bfn.fq() as u64);
        self.aiu(COJ_, bhy as u32);
        self.aiu(COK_, (bhy >> 32) as u32);

        self.ddb = 0;
    }

    
    fn aiy(&mut self) {
        
        self.xoh();

        
        let rpl = BOV_ | BOW_;
        self.asg(CNQ_, rpl);

        
        self.asg(COD_, DN_ as u16);

        
        self.aiu(COM_, CZB_ | CZA_);

        
        let hyl = CPW_ | CPU_ | CPV_ | CPX_ | CPY_;
        self.aiu(AHD_, hyl);

        
        self.akw(CNR_, 0x3F);

        
        self.uhu();

        
        self.akw(WO_, BMQ_ | BMR_);

        
        self.asg(BEE_, CBV_);

        crate::log_debug!("[RTL8169] Controller enabled (RX+TX)");
    }

    
    fn qzc(&mut self) {
        let egr = self.amp(WP_);
        let bln = egr & AGL_ != 0;
        self.aik.store(bln, Ordering::SeqCst);

        if bln {
            let ig = if egr & BDB_ != 0 {
                1000
            } else if egr & BDC_ != 0 {
                100
            } else {
                10
            };
            crate::log!("[RTL8169] Link up at {} Mbps", ig);
        }
    }
}





impl Gi for Rtl8169Driver {
    fn co(&self) -> &Co {
        &BZ_
    }

    fn probe(&mut self, cgm: &S) -> Result<(), &'static str> {
        self.status = DriverStatus::Py;

        crate::log!("[RTL8169] Probing {:04X}:{:04X}", cgm.ml, cgm.mx);

        
        let aew = cgm.cje(0).ok_or("No BAR0")?;
        if aew == 0 { return Err("BAR0 is zero"); }

        
        const CPP_: usize = 4096;
        self.hv = crate::memory::bki(aew, CPP_)
            .jd(|aa| {
                crate::serial_println!("[RTL8169] map_mmio failed: {}", aa);
                "Failed to map RTL8169 MMIO"
            })?;

        crate::log_debug!("[RTL8169] MMIO: phys={:#x} virt={:#x}", aew, self.hv);

        
        self.apa();

        
        self.lxr();

        
        self.leg();
        self.lei();

        
        self.aiy();

        
        self.qzc();

        
        if !self.aik.load(Ordering::Relaxed) {
            crate::log_warn!("[RTL8169] Link not detected — assuming up (QEMU mode)");
            self.aik.store(true, Ordering::SeqCst);
        }

        self.jr.store(true, Ordering::SeqCst);
        self.status = DriverStatus::Ai;

        crate::log!("[RTL8169] MAC: {:02X}:{:02X}:{:02X}:{:02X}:{:02X}:{:02X}",
            self.ed[0], self.ed[1], self.ed[2], self.ed[3], self.ed[4], self.ed[5]);

        Ok(())
    }

    fn ay(&mut self) -> Result<(), &'static str> {
        self.status = DriverStatus::Ai;
        Ok(())
    }

    fn qg(&mut self) -> Result<(), &'static str> {
        
        self.akw(WO_, 0);
        
        self.asg(BEE_, 0);
        self.status = DriverStatus::Ky;
        Ok(())
    }

    fn status(&self) -> DriverStatus {
        self.status
    }
}





impl Ha for Rtl8169Driver {
    fn csg(&self) -> [u8; 6] {
        self.ed
    }

    fn aik(&self) -> bool {
        if self.hv != 0 {
            self.amp(WP_) & AGL_ != 0
        } else {
            self.aik.load(Ordering::Relaxed)
        }
    }

    fn gll(&self) -> u32 {
        if self.hv == 0 { return 0; }
        let egr = self.amp(WP_);
        if egr & BDB_ != 0 { 1000 }
        else if egr & BDC_ != 0 { 100 }
        else if egr & CJL_ != 0 { 10 }
        else { 0 }
    }

    fn baq(&mut self, f: &[u8]) -> Result<(), &'static str> {
        if !self.jr.load(Ordering::Relaxed) {
            return Err("Driver not initialized");
        }
        if f.len() > DN_ { return Err("Packet too large"); }
        if f.len() < 14 { return Err("Packet too small"); }

        let w = self.ddb;

        
        let mut aah = 10_000;
        while self.bfn[w].dkg & MS_ != 0 {
            aah -= 1;
            if aah == 0 {
                self.dmv.fetch_add(1, Ordering::Relaxed);
                return Err("TX timeout — descriptor still owned by NIC");
            }
            core::hint::hc();
        }

        
        let bi = &mut self.cnk[w];
        bi[..f.len()].dg(f);

        
        let ht = Self::abw(bi.fq() as u64);
        self.bfn[w].imh = ht as u32;
        self.bfn[w].img = (ht >> 32) as u32;

        
        let mut flags = MS_ | AAZ_ | ABA_ | (f.len() as u32 & 0x3FFF);
        if w == CR_ - 1 {
            flags |= SO_;
        }
        self.bfn[w].dkg = flags;
        self.bfn[w].htt = 0;

        
        self.akw(COL_, CYH_);

        
        self.ddb = (self.ddb + 1) % CR_;

        self.cuz.fetch_add(1, Ordering::Relaxed);
        self.bpc.fetch_add(f.len() as u64, Ordering::Relaxed);

        Ok(())
    }

    fn chb(&mut self) -> Option<Vec<u8>> {
        if !self.jr.load(Ordering::Relaxed) { return None; }

        let w = self.chm;
        let dkg = self.bcl[w].dkg;

        
        if dkg & MS_ != 0 {
            return None;
        }

        
        if dkg & (AAZ_ | ABA_) != (AAZ_ | ABA_) {
            
            self.dbn.fetch_add(1, Ordering::Relaxed);
            self.jls(w);
            return None;
        }

        
        let go = (dkg & 0x3FFF) as usize;
        if go < 4 || go > DN_ {
            self.dbn.fetch_add(1, Ordering::Relaxed);
            self.jls(w);
            return None;
        }

        let duk = go - 4; 
        if duk == 0 {
            self.jls(w);
            return None;
        }

        
        let ex = self.dbm[w][..duk].ip();

        
        self.jls(w);

        self.dbo.fetch_add(1, Ordering::Relaxed);
        self.bsc.fetch_add(duk as u64, Ordering::Relaxed);

        Some(ex)
    }

    fn poll(&mut self) {
        if !self.jr.load(Ordering::Relaxed) { return; }

        
        let cru = self.aym(BEF_);
        if cru != 0 {
            self.asg(BEF_, cru); 
        }

        
        if cru & AXD_ != 0 {
            let egr = self.amp(WP_);
            self.aik.store(egr & AGL_ != 0, Ordering::SeqCst);
        }
    }

    fn cm(&self) -> NetStats {
        NetStats {
            cuz: self.cuz.load(Ordering::Relaxed),
            dbo: self.dbo.load(Ordering::Relaxed),
            bpc: self.bpc.load(Ordering::Relaxed),
            bsc: self.bsc.load(Ordering::Relaxed),
            dmv: self.dmv.load(Ordering::Relaxed),
            dbn: self.dbn.load(Ordering::Relaxed),
            mnn: 0,
            mbk: 0,
        }
    }

    fn pjd(&mut self, iq: bool) -> Result<(), &'static str> {
        if !self.jr.load(Ordering::Relaxed) { return Err("Not initialized"); }
        let mut hyl = self.amp(AHD_);
        if iq {
            hyl |= BEV_; 
        } else {
            hyl &= !BEV_;
        }
        self.aiu(AHD_, hyl);
        Ok(())
    }
}

impl Rtl8169Driver {
    
    fn jls(&mut self, w: usize) {
        let mut flags = MS_ | (DN_ as u32 & 0x3FFF);
        if w == CB_ - 1 {
            flags |= SO_;
        }
        self.bcl[w].dkg = flags;
        self.bcl[w].htt = 0;
        self.chm = (self.chm + 1) % CB_;
    }
}





const BZ_: Co = Co {
    j: "rtl8169",
    dk: "1.0.0",
    gzh: "TrustOS Team",
    gb: DriverCategory::As,
    fye: &[
        (0x10EC, 0x8169),  
        (0x10EC, 0x8168),  
        (0x10EC, 0x8161),  
        (0x10EC, 0x8136),  
    ],
};

pub fn nw() {
    crate::drivers::nw(BZ_, || {
        Box::new(Rtl8169Driver::new())
    });
    crate::drivers::net::jly(BZ_, || {
        Box::new(Rtl8169Driver::new())
    });
}


pub fn ky() -> bool {
    
    crate::drivers::net::bzy()
}
