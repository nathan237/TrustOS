










use alloc::boxed::Box;
use alloc::vec::Vec;
use alloc::vec;
use core::ptr::{read_volatile, write_volatile};
use core::sync::atomic::{AtomicBool, AtomicU64, Ordering};

use super::{Ha, NetStats};
use crate::drivers::{Gi, Co, DriverStatus, DriverCategory};
use crate::pci::S;






const PC_: u32 = 0x0000;        
const WR_: u32 = 0x0008;      
const BED_: u32 = 0x0014;        


const CNS_: u32 = 0x00C0;         
const ECE_: u32 = 0x00D0;         
const AHC_: u32 = 0x00D8;         


const WQ_: u32 = 0x0100;        
const CNY_: u32 = 0x2800;       
const CNX_: u32 = 0x2804;       
const COA_: u32 = 0x2808;       
const CNZ_: u32 = 0x2810;         
const BEI_: u32 = 0x2818;         


const BEJ_: u32 = 0x0400;        
const COI_: u32 = 0x0410;        
const COF_: u32 = 0x3800;       
const COE_: u32 = 0x3804;       
const COH_: u32 = 0x3808;       
const COG_: u32 = 0x3810;         
const BEK_: u32 = 0x3818;         


const BEH_: u32 = 0x5400;        
const BEG_: u32 = 0x5404;        


const CNW_: u32 = 0x5200;         





const BQJ_: u32 = 1 << 0;         
const BQI_: u32 = 1 << 5;       
const BQK_: u32 = 1 << 6;        
const APO_: u32 = 1 << 26;       


const AIG_: u32 = 1 << 1;       
const BGH_: u32 = 0xC0; 


const CNF_: u32 = 1 << 1;         
const CNI_: u32 = 1 << 2;        
const AGW_: u32 = 1 << 3;        
const AGV_: u32 = 1 << 4;        
const CNG_: u32 = 0 << 6;   
const CNH_: u32 = 0 << 8; 
const CND_: u32 = 1 << 15;       
const CNE_: u32 = 0 << 16; 
const CNJ_: u32 = 1 << 26;     


const CXP_: u32 = 1 << 1;         
const CXQ_: u32 = 1 << 3;        
const CXO_: u32 = 4;        
const CXN_: u32 = 12;     
const CXR_: u32 = 1 << 24;      


const CXS_: u8 = 1 << 0;    
const CXT_: u8 = 1 << 1;   
const CXU_: u8 = 1 << 3;     


const AIR_: u8 = 1 << 0;     


const CNK_: u8 = 1 << 0;     


const CBG_: u32 = 1 << 2;         





const CB_: usize = 32;
const CR_: usize = 8;
const DN_: usize = 2048;


#[repr(C, align(16))]
#[derive(Clone, Copy)]
struct RxDesc {
    hbs: u64,    
    go: u16,         
    bmj: u16,       
    status: u8,          
    bqn: u8,          
    fvb: u16,        
}


#[repr(C, align(16))]
#[derive(Clone, Copy)]
struct TxDesc {
    hbs: u64,    
    go: u16,         
    rrb: u8,             
    cmd: u8,             
    status: u8,          
    eoe: u8,             
    fvb: u16,        
}

impl Default for RxDesc {
    fn default() -> Self {
        Self {
            hbs: 0,
            go: 0,
            bmj: 0,
            status: 0,
            bqn: 0,
            fvb: 0,
        }
    }
}

impl Default for TxDesc {
    fn default() -> Self {
        Self {
            hbs: 0,
            go: 0,
            rrb: 0,
            cmd: 0,
            status: AIR_, 
            eoe: 0,
            fvb: 0,
        }
    }
}





pub struct E1000Driver {
    status: DriverStatus,
    hv: u64,
    ed: [u8; 6],
    
    
    bcl: Vec<RxDesc>,
    bfn: Vec<TxDesc>,
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

impl E1000Driver {
    pub fn new() -> Self {
        Self {
            status: DriverStatus::Aff,
            hv: 0,
            ed: [0x52, 0x54, 0x00, 0xE1, 0x00, 0x00],
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
    
    
    fn bam(&self, l: u32) -> u32 {
        if self.hv == 0 {
            return 0;
        }
        let ag = (self.hv + l as u64) as *const u32;
        unsafe { read_volatile(ag) }
    }
    
    
    fn afl(&self, l: u32, bn: u32) {
        if self.hv == 0 {
            return;
        }
        let ag = (self.hv + l as u64) as *mut u32;
        unsafe { write_volatile(ag, bn) }
    }
    
    
    fn auv(ht: u64) -> u64 {
        const DS_: u64 = 0xFFFF_8000_0000_0000;
        ht + DS_
    }
    
    
    fn abw(ju: u64) -> u64 {
        const DS_: u64 = 0xFFFF_8000_0000_0000;
        if ju >= DS_ {
            ju - DS_
        } else {
            ju
        }
    }
    
    
    fn apa(&mut self) {
        crate::log_debug!("[E1000] Resetting device...");
        
        
        self.afl(AHC_, 0xFFFFFFFF);
        
        
        let db = self.bam(PC_);
        self.afl(PC_, db | APO_);
        
        
        for _ in 0..1000 {
            if self.bam(PC_) & APO_ == 0 {
                break;
            }
            for _ in 0..1000 {
                core::hint::hc();
            }
        }
        
        
        self.afl(AHC_, 0xFFFFFFFF);
        
        crate::log_debug!("[E1000] Reset complete");
    }
    
    
    fn lxr(&mut self) {
        
        let frx = self.bam(BEH_);
        let hwm = self.bam(BEG_);
        
        if frx != 0 || hwm != 0 {
            self.ed[0] = (frx >> 0) as u8;
            self.ed[1] = (frx >> 8) as u8;
            self.ed[2] = (frx >> 16) as u8;
            self.ed[3] = (frx >> 24) as u8;
            self.ed[4] = (hwm >> 0) as u8;
            self.ed[5] = (hwm >> 8) as u8;
            return;
        }
        
        
        for a in 0..3 {
            self.afl(BED_, 1 | ((a as u32) << 8));
            for _ in 0..1000 {
                let npg = self.bam(BED_);
                if npg & (1 << 4) != 0 {
                    let f = (npg >> 16) as u16;
                    self.ed[a * 2] = (f & 0xFF) as u8;
                    self.ed[a * 2 + 1] = (f >> 8) as u8;
                    break;
                }
                core::hint::hc();
            }
        }
        
        
        if self.ed == [0, 0, 0, 0, 0, 0] {
            self.ed = [0x52, 0x54, 0x00, 0x12, 0x34, 0x56];
        }
    }
    
    
    fn leg(&mut self) {
        crate::log_debug!("[E1000] Initializing RX ring ({} descriptors)", CB_);
        
        self.bcl = vec![RxDesc::default(); CB_];
        self.dbm = Vec::fc(CB_);
        
        for a in 0..CB_ {
            let bi = vec![0u8; DN_];
            let ki = Self::abw(bi.fq() as u64);
            self.bcl[a].hbs = ki;
            self.bcl[a].status = 0;
            self.dbm.push(bi);
        }
        
        let hfx = Self::abw(self.bcl.fq() as u64);
        
        self.afl(CNY_, hfx as u32);
        self.afl(CNX_, (hfx >> 32) as u32);
        
        let hxs = (CB_ * core::mem::size_of::<RxDesc>()) as u32;
        self.afl(COA_, hxs);
        
        self.afl(CNZ_, 0);
        self.afl(BEI_, (CB_ - 1) as u32);
        
        self.chm = 0;
    }
    
    
    fn lei(&mut self) {
        crate::log_debug!("[E1000] Initializing TX ring ({} descriptors)", CR_);
        
        self.bfn = vec![TxDesc::default(); CR_];
        self.cnk = Vec::fc(CR_);
        
        for a in 0..CR_ {
            self.cnk.push(vec![0u8; DN_]);
            
            self.bfn[a].status = AIR_;
        }
        
        let hfx = Self::abw(self.bfn.fq() as u64);
        
        self.afl(COF_, hfx as u32);
        self.afl(COE_, (hfx >> 32) as u32);
        
        let hxs = (CR_ * core::mem::size_of::<TxDesc>()) as u32;
        self.afl(COH_, hxs);
        
        self.afl(COG_, 0);
        self.afl(BEK_, 0);
        
        self.ddb = 0;
    }
    
    
    fn slc(&mut self) {
        let hwu = CNF_ | CNI_ | AGW_ | AGV_ 
                 | CNG_ | CNH_ | CND_ 
                 | CNJ_ | CNE_;
        self.afl(WQ_, hwu);
    }
    
    
    fn slf(&mut self) {
        self.afl(COI_, 10 | (8 << 10) | (6 << 20));
        
        let xbm = CXP_ | CXQ_ 
                 | (15 << CXO_) 
                 | (64 << CXN_) 
                 | CXR_;
        self.afl(BEJ_, xbm);
    }
    
    
    fn wlc(&mut self) {
        let db = self.bam(PC_);
        let hsq = db | BQK_ | BQI_ | BQJ_;
        self.afl(PC_, hsq);
        
        
        for a in 0..128 {
            self.afl(CNW_ + a * 4, 0);
        }
        
        
        for a in 0..500 {
            let status = self.bam(WR_);
            if status & AIG_ != 0 {
                self.aik.store(true, Ordering::SeqCst);
                let ig = match (status & BGH_) >> 6 {
                    0 => 10, 1 => 100, _ => 1000,
                };
                crate::log!("[E1000] Link up at {} Mbps (after {} iterations)", ig, a + 1);
                return;
            }
            for _ in 0..10000 { core::hint::hc(); }
        }
        
        crate::log_warn!("[E1000] Link not detected - continuing anyway (VirtualBox NAT mode)");
        self.aik.store(true, Ordering::SeqCst);
    }
}

impl Gi for E1000Driver {
    fn co(&self) -> &Co {
        &BZ_
    }
    
    fn probe(&mut self, cgm: &S) -> Result<(), &'static str> {
        self.status = DriverStatus::Py;
        
        crate::log!("[E1000] Probing {:04X}:{:04X}", cgm.ml, cgm.mx);
        
        let aew = cgm.cje(0).ok_or("No BAR0")?;
        if aew == 0 { return Err("BAR0 is zero"); }
        
        crate::serial_println!("[E1000] BAR0={:#x}, calling map_mmio...", aew);
        
        
        const BSV_: usize = 128 * 1024;
        self.hv = crate::memory::bki(aew, BSV_)
            .jd(|aa| { crate::serial_println!("[E1000] map_mmio failed: {}", aa); "Failed to map E1000 MMIO" })?;
        crate::serial_println!("[E1000] map_mmio returned {:#x}", self.hv);
        crate::log_debug!("[E1000] MMIO: phys={:#x} virt={:#x}", aew, self.hv);
        
        self.apa();
        self.lxr();
        
        
        let frx = (self.ed[0] as u32) | ((self.ed[1] as u32) << 8)
                | ((self.ed[2] as u32) << 16) | ((self.ed[3] as u32) << 24);
        let hwm = (self.ed[4] as u32) | ((self.ed[5] as u32) << 8) | (1 << 31);
        self.afl(BEH_, frx);
        self.afl(BEG_, hwm);
        
        self.leg();
        self.lei();
        self.wlc();
        self.slc();
        self.slf();
        
        self.jr.store(true, Ordering::SeqCst);
        self.status = DriverStatus::Ai;
        
        crate::log!("[E1000] MAC: {:02X}:{:02X}:{:02X}:{:02X}:{:02X}:{:02X}",
            self.ed[0], self.ed[1], self.ed[2], self.ed[3], self.ed[4], self.ed[5]);
        
        Ok(())
    }
    
    fn ay(&mut self) -> Result<(), &'static str> {
        self.status = DriverStatus::Ai;
        Ok(())
    }
    
    fn qg(&mut self) -> Result<(), &'static str> {
        self.afl(WQ_, 0);
        self.afl(BEJ_, 0);
        self.afl(AHC_, 0xFFFFFFFF);
        self.status = DriverStatus::Ky;
        Ok(())
    }
    
    fn status(&self) -> DriverStatus {
        self.status
    }
}

impl Ha for E1000Driver {
    fn csg(&self) -> [u8; 6] {
        self.ed
    }
    
    fn aik(&self) -> bool {
        if self.hv != 0 {
            let status = self.bam(WR_);
            status & AIG_ != 0
        } else {
            self.aik.load(Ordering::Relaxed)
        }
    }
    
    fn gll(&self) -> u32 {
        if self.hv == 0 { return 0; }
        let status = self.bam(WR_);
        match (status & BGH_) >> 6 {
            0 => 10, 1 => 100, _ => 1000,
        }
    }
    
    fn baq(&mut self, f: &[u8]) -> Result<(), &'static str> {
        if !self.jr.load(Ordering::Relaxed) {
            return Err("Driver not initialized");
        }
        if f.len() > DN_ { return Err("Packet too large"); }
        if f.len() < 14 { return Err("Packet too small"); }
        
        let and = self.ddb;
        
        
        let mut aah = 10000;
        while self.bfn[and].status & AIR_ == 0 {
            aah -= 1;
            if aah == 0 {
                self.dmv.fetch_add(1, Ordering::Relaxed);
                return Err("TX timeout");
            }
            core::hint::hc();
        }
        
        
        let bi = &mut self.cnk[and];
        bi[..f.len()].dg(f);
        
        
        let ki = Self::abw(bi.fq() as u64);
        self.bfn[and].hbs = ki;
        self.bfn[and].go = f.len() as u16;
        self.bfn[and].cmd = CXS_ | CXT_ | CXU_;
        self.bfn[and].status = 0;
        
        
        self.ddb = (self.ddb + 1) % CR_;
        self.afl(BEK_, self.ddb as u32);
        
        self.cuz.fetch_add(1, Ordering::Relaxed);
        self.bpc.fetch_add(f.len() as u64, Ordering::Relaxed);
        
        Ok(())
    }
    
    fn chb(&mut self) -> Option<Vec<u8>> {
        if !self.jr.load(Ordering::Relaxed) { return None; }
        
        let and = self.chm;
        let status = self.bcl[and].status;
        
        if status & CNK_ == 0 { return None; }
        
        
        
        if self.bcl[and].bqn != 0 {
            self.dbn.fetch_add(1, Ordering::Relaxed);
            self.bcl[and].status = 0;
            self.chm = (self.chm + 1) % CB_;
            return None;
        }
        
        let go = self.bcl[and].go as usize;
        if go == 0 || go > DN_ {
            self.bcl[and].status = 0;
            self.chm = (self.chm + 1) % CB_;
            return None;
        }
        
        let ex = self.dbm[and][..go].ip();
        
        self.bcl[and].status = 0;
        self.bcl[and].go = 0;
        self.afl(BEI_, and as u32);
        self.chm = (self.chm + 1) % CB_;
        
        self.dbo.fetch_add(1, Ordering::Relaxed);
        self.bsc.fetch_add(go as u64, Ordering::Relaxed);
        
        Some(ex)
    }
    
    fn poll(&mut self) {
        if !self.jr.load(Ordering::Relaxed) { return; }
        let bnh = self.bam(CNS_);
        if bnh & CBG_ != 0 {
            let status = self.bam(WR_);
            self.aik.store(status & AIG_ != 0, Ordering::SeqCst);
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
        let mut hwu = self.bam(WQ_);
        if iq { hwu |= AGW_ | AGV_; } 
        else { hwu &= !(AGW_ | AGV_); }
        self.afl(WQ_, hwu);
        Ok(())
    }
}

const BZ_: Co = Co {
    j: "e1000",
    dk: "1.0.0",
    gzh: "TrustOS Team",
    gb: DriverCategory::As,
    fye: &[
        (0x8086, 0x100E),  
        (0x8086, 0x100F),  
        (0x8086, 0x10D3),  
        (0x8086, 0x153A),  
        (0x8086, 0x1533),  
    ],
};

pub fn nw() {
    crate::drivers::nw(BZ_, || {
        Box::new(E1000Driver::new())
    });
    crate::drivers::net::jly(BZ_, || {
        Box::new(E1000Driver::new())
    });
}
