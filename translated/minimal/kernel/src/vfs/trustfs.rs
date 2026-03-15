










use alloc::string::String;
use alloc::vec::Vec;
use alloc::vec;
use alloc::sync::Arc;
use alloc::collections::BTreeMap;
use spin::RwLock;
use core::sync::atomic::{AtomicU64, Ordering};

use super::{
    Cc, Et, Ep, Stat, Br, FileType,
    I, B, VfsError
};
use super::fat32::Bj;

const H_: usize = 512;
const Iv: u32 = 0x54525553; 
const Afl: u32 = 1;

const AIL_: u64 = 0;
const UF_: u64 = 1;
const CBS_: u64 = 16;
const LW_: u64 = 17;
const ZL_: u64 = 16;

const BC_: u64 = 97;

const AFA_: usize = 256;
const UE_: usize = H_ / core::mem::size_of::<DiskInode>();

const KP_: usize = 28;
const BX_: usize = 12;
const BF_: usize = H_ / 4; 

const AZO_: usize = BX_ + BF_ + BF_ * BF_;


#[repr(C)]
#[derive(Clone, Copy, Debug)]
struct Superblock {
    sj: u32,
    dk: u32,
    fxa: u32,
    ivt: u32,
    pva: u32,
    ivv: u32,
    py: u32,
    cbm: u32,
    awt: [u32; 8],
}

impl Superblock {
    fn new(axf: u64) -> Self {
        Self {
            sj: Iv,
            dk: Afl,
            fxa: (axf - BC_) as u32,
            ivt: (axf - BC_ - 1) as u32, 
            pva: AFA_ as u32,
            ivv: (AFA_ - 1) as u32, 
            py: H_ as u32,
            cbm: 1,
            awt: [0; 8],
        }
    }
    
    fn cld(&self) -> bool {
        self.sj == Iv && self.dk == Afl
    }
}


#[repr(C)]
#[derive(Clone, Copy, Debug)]
struct DiskInode {
    ev: u16,          
    fpa: u16,         
    aw: u32,          
    xk: u32,        
    byi: u32,         
    bnp: u32,         
    eav: [u32; BX_], 
    brd: u32,      
    dgn: u32, 
}

impl Default for DiskInode {
    fn default() -> Self {
        Self {
            ev: 0,
            fpa: 0,
            aw: 0,
            xk: 0,
            byi: 0,
            bnp: 0,
            eav: [0; BX_],
            brd: 0,
            dgn: 0,
        }
    }
}

impl DiskInode {
    fn kd(&self) -> FileType {
        match (self.ev >> 12) & 0xF {
            0x4 => FileType::K,
            0x8 => FileType::Ea,
            0x2 => FileType::Mv,
            0x6 => FileType::Bj,
            _ => FileType::Ea,
        }
    }
    
    fn ta(&self) -> bool {
        self.kd() == FileType::K
    }
    
    fn pjh(&mut self, agm: FileType) {
        let xnq = match agm {
            FileType::K => 0x4,
            FileType::Ea => 0x8,
            FileType::Mv => 0x2,
            FileType::Bj => 0x6,
            _ => 0x8,
        };
        self.ev = (self.ev & 0x0FFF) | (xnq << 12);
    }
}


#[repr(C)]
#[derive(Clone, Copy)]
struct My {
    fa: u32,
    j: [u8; KP_],
}

impl My {
    fn amj(&self) -> &str {
        let ci = self.j.iter().qf(|&o| o == 0).unwrap_or(KP_);
        core::str::jg(&self.j[..ci]).unwrap_or("")
    }
}


struct InodeCache {
    arj: BTreeMap<I, DiskInode>,
}

impl InodeCache {
    fn new() -> Self {
        Self {
            arj: BTreeMap::new(),
        }
    }
}


struct Bus {
    fs: Arc<Afa>,
    dd: I,
}

impl Et for Bus {
    fn read(&self, l: u64, k: &mut [u8]) -> B<usize> {
        self.fs.mq(self.dd, l, k)
    }
    
    fn write(&self, l: u64, k: &[u8]) -> B<usize> {
        self.fs.ns(self.dd, l, k)
    }
    
    fn hm(&self) -> B<Stat> {
        self.fs.hm(self.dd)
    }
    
    fn dmu(&self, aw: u64) -> B<()> {
        self.fs.dmu(self.dd, aw)
    }
    
    fn sync(&self) -> B<()> {
        self.fs.sync()
    }
}


struct Bur {
    fs: Arc<Afa>,
    dd: I,
}

impl Ep for Bur {
    fn cga(&self, j: &str) -> B<I> {
        self.fs.cga(self.dd, j)
    }
    
    fn brx(&self) -> B<Vec<Br>> {
        self.fs.brx(self.dd)
    }
    
    fn avp(&self, j: &str, kd: FileType) -> B<I> {
        self.fs.avp(self.dd, j, kd)
    }
    
    fn cnm(&self, j: &str) -> B<()> {
        self.fs.cnm(self.dd, j)
    }
    
    fn hm(&self) -> B<Stat> {
        self.fs.hm(self.dd)
    }
}


struct Afa {
    ezg: RwLock<Superblock>,
    jaf: RwLock<InodeCache>,
    no: RwLock<bool>,
    backend: Arc<dyn Bj>,
}

impl Afa {
    
    fn xr(&self, jk: u64, k: &mut [u8; H_]) -> B<()> {
        if super::block_cache::qvi(jk, k).is_ok() {
            return Ok(());
        }
        self.backend.xr(jk, k)
            .jd(|_| VfsError::Av)
    }
    
    
    fn aby(&self, jk: u64, k: &[u8; H_]) -> B<()> {
        if super::block_cache::qvj(jk, k).is_ok() {
            return Ok(());
        }
        self.backend.aby(jk, k)
            .jd(|_| VfsError::Av)
    }

    
    fn zwu(&self, jk: u64, k: &[u8; H_]) -> B<()> {
        
        let _ = super::wal::ljr(jk, k);
        self.aby(jk, k)
    }

    
    fn svd(&self) -> B<()> {
        let backend = &self.backend;
        let eld = |jk: u64, f: &[u8; H_]| -> Result<(), ()> {
            backend.aby(jk, f)
        };
        super::wal::dfc(&eld).jd(|_| VfsError::Av)
    }
    
    
    fn ayn(&self, dd: I) -> B<DiskInode> {
        
        {
            let bdq = self.jaf.read();
            if let Some(fa) = bdq.arj.get(&dd) {
                return Ok(*fa);
            }
        }
        
        
        let jk = UF_ + (dd as u64 / UE_ as u64);
        let l = (dd as usize % UE_) * core::mem::size_of::<DiskInode>();
        
        let mut k = [0u8; H_];
        self.xr(jk, &mut k)?;
        
        let flm = k[l..].fq() as *const DiskInode;
        let fa = unsafe { *flm };
        
        
        {
            let mut bdq = self.jaf.write();
            bdq.arj.insert(dd, fa);
        }
        
        Ok(fa)
    }
    
    
    fn jxh(&self, dd: I, fa: &DiskInode) -> B<()> {
        let jk = UF_ + (dd as u64 / UE_ as u64);
        let l = (dd as usize % UE_) * core::mem::size_of::<DiskInode>();
        
        
        let mut k = [0u8; H_];
        self.xr(jk, &mut k)?;
        
        let flm = k[l..].mw() as *mut DiskInode;
        unsafe { *flm = *fa; }
        
        self.aby(jk, &k)?;
        
        
        {
            let mut bdq = self.jaf.write();
            bdq.arj.insert(dd, *fa);
        }
        
        *self.no.write() = true;
        Ok(())
    }
    
    
    fn qgu(&self) -> B<I> {
        let mut is = self.ezg.write();
        if is.ivv == 0 {
            return Err(VfsError::Tq);
        }
        
        
        for dd in 1..AFA_ as u64 {
            let fa = self.ayn(dd)?;
            if fa.fpa == 0 && fa.ev == 0 {
                is.ivv -= 1;
                return Ok(dd);
            }
        }
        
        Err(VfsError::Tq)
    }
    
    
    fn ijk(&self) -> B<u32> {
        let mut is = self.ezg.write();
        if is.ivt == 0 {
            return Err(VfsError::Tq);
        }
        
        
        for fdj in 0..ZL_ {
            let mut k = [0u8; H_];
            self.xr(LW_ + fdj, &mut k)?;
            
            for avk in 0..H_ {
                if k[avk] != 0xFF {
                    
                    for ga in 0..8 {
                        if (k[avk] & (1 << ga)) == 0 {
                            
                            k[avk] |= 1 << ga;
                            self.aby(LW_ + fdj, &k)?;
                            
                            is.ivt -= 1;
                            let block = (fdj as u32 * H_ as u32 * 8)
                                + (avk as u32 * 8)
                                + ga as u32;
                            return Ok(block);
                        }
                    }
                }
            }
        }
        
        Err(VfsError::Tq)
    }
    
    
    fn ebz(&self, block: u32) -> B<()> {
        let mut is = self.ezg.write();
        
        let fdj = block as u64 / (H_ as u64 * 8);
        let avk = (block as usize / 8) % H_;
        let ga = block as usize % 8;
        
        if fdj >= ZL_ {
            return Err(VfsError::Bjs);
        }
        
        let mut k = [0u8; H_];
        self.xr(LW_ + fdj, &mut k)?;
        
        
        k[avk] &= !(1 << ga);
        self.aby(LW_ + fdj, &k)?;
        
        is.ivt += 1;
        Ok(())
    }
    
    
    fn sxe(&self, fa: &DiskInode) -> B<()> {
        
        for a in 0..BX_ {
            if fa.eav[a] != 0 {
                self.ebz(fa.eav[a])?;
            }
        }
        
        
        if fa.brd != 0 {
            let mut crm = [0u8; H_];
            self.xr(BC_ + fa.brd as u64, &mut crm)?;
            let egy = unsafe { &*(crm.fq() as *const [u32; BF_]) };
            
            for &ptr in egy.iter() {
                if ptr != 0 {
                    self.ebz(ptr)?;
                }
            }
            
            
            self.ebz(fa.brd)?;
        }
        
        
        if fa.dgn != 0 {
            let mut eef = [0u8; H_];
            self.xr(BC_ + fa.dgn as u64, &mut eef)?;
            let etr = unsafe { &*(eef.fq() as *const [u32; BF_]) };
            
            for &eeg in etr.iter() {
                if eeg != 0 {
                    let mut eeh = [0u8; H_];
                    self.xr(BC_ + eeg as u64, &mut eeh)?;
                    let hpm = unsafe { &*(eeh.fq() as *const [u32; BF_]) };
                    
                    for &njj in hpm.iter() {
                        if njj != 0 {
                            self.ebz(njj)?;
                        }
                    }
                    
                    
                    self.ebz(eeg)?;
                }
            }
            
            
            self.ebz(fa.dgn)?;
        }
        
        Ok(())
    }
    
    
    fn pcs(&self, fa: &DiskInode, bbb: usize) -> B<u32> {
        if bbb < BX_ {
            Ok(fa.eav[bbb])
        } else if bbb < BX_ + BF_ {
            if fa.brd == 0 { return Ok(0); }
            let mut crm = [0u8; H_];
            self.xr(BC_ + fa.brd as u64, &mut crm)?;
            let egy = unsafe { &*(crm.fq() as *const [u32; BF_]) };
            Ok(egy[bbb - BX_])
        } else if bbb < AZO_ {
            
            if fa.dgn == 0 { return Ok(0); }
            let hgb = bbb - BX_ - BF_;
            let crx = hgb / BF_;
            let dsn = hgb % BF_;
            
            let mut eef = [0u8; H_];
            self.xr(BC_ + fa.dgn as u64, &mut eef)?;
            let etr = unsafe { &*(eef.fq() as *const [u32; BF_]) };
            let eeg = etr[crx];
            if eeg == 0 { return Ok(0); }
            
            let mut eeh = [0u8; H_];
            self.xr(BC_ + eeg as u64, &mut eeh)?;
            let hpm = unsafe { &*(eeh.fq() as *const [u32; BF_]) };
            Ok(hpm[dsn])
        } else {
            Err(VfsError::Tq) 
        }
    }

    
    fn xvl(&self, fa: &mut DiskInode, w: usize, hau: u32) -> B<()> {
        if fa.brd == 0 {
            fa.brd = self.ijk()?;
            let ajs = [0u8; H_];
            self.aby(BC_ + fa.brd as u64, &ajs)?;
        }
        let mut crm = [0u8; H_];
        self.xr(BC_ + fa.brd as u64, &mut crm)?;
        let egy = unsafe { &mut *(crm.mw() as *mut [u32; BF_]) };
        egy[w] = hau;
        self.aby(BC_ + fa.brd as u64, &crm)
    }

    
    fn xvj(&self, fa: &mut DiskInode, hgb: usize, hau: u32) -> B<()> {
        let ajs = [0u8; H_];
        
        if fa.dgn == 0 {
            fa.dgn = self.ijk()?;
            self.aby(BC_ + fa.dgn as u64, &ajs)?;
        }
        let crx = hgb / BF_;
        let dsn = hgb % BF_;
        
        let mut eef = [0u8; H_];
        self.xr(BC_ + fa.dgn as u64, &mut eef)?;
        let etr = unsafe { &mut *(eef.mw() as *mut [u32; BF_]) };
        
        if etr[crx] == 0 {
            etr[crx] = self.ijk()?;
            self.aby(BC_ + fa.dgn as u64, &eef)?;
            self.aby(BC_ + etr[crx] as u64, &ajs)?;
        }
        let eeg = etr[crx];
        
        let mut eeh = [0u8; H_];
        self.xr(BC_ + eeg as u64, &mut eeh)?;
        let hpm = unsafe { &mut *(eeh.mw() as *mut [u32; BF_]) };
        hpm[dsn] = hau;
        self.aby(BC_ + eeg as u64, &eeh)
    }

    
    fn mq(&self, dd: I, l: u64, k: &mut [u8]) -> B<usize> {
        let fa = self.ayn(dd)?;
        
        if l >= fa.aw as u64 {
            return Ok(0); 
        }
        
        let ajp = core::cmp::v(k.len(), (fa.aw as u64 - l) as usize);
        let mut cjl = 0;
        let mut azv = l as usize;
        
        while cjl < ajp {
            let bbb = azv / H_;
            let emw = azv % H_;
            
            let clw = self.pcs(&fa, bbb)?;
            if clw == 0 { break; }
            
            let mut aae = [0u8; H_];
            self.xr(BC_ + clw as u64, &mut aae)?;
            
            let aiw = core::cmp::v(H_ - emw, ajp - cjl);
            k[cjl..cjl + aiw]
                .dg(&aae[emw..emw + aiw]);
            
            cjl += aiw;
            azv += aiw;
        }
        
        Ok(cjl)
    }
    
    
    fn ns(&self, dd: I, l: u64, k: &[u8]) -> B<usize> {
        let mut fa = self.ayn(dd)?;
        
        let mut cjm = 0;
        let mut azv = l as usize;
        let ukw = AZO_;
        
        while cjm < k.len() {
            let bbb = azv / H_;
            let emw = azv % H_;
            
            if bbb >= ukw { break; }
            
            
            let clw = self.pcs(&fa, bbb)?;
            let clw = if clw == 0 {
                let jgr = self.ijk()?;
                fa.xk += 1;
                if bbb < BX_ {
                    fa.eav[bbb] = jgr;
                } else if bbb < BX_ + BF_ {
                    self.xvl(&mut fa, bbb - BX_, jgr)?;
                } else {
                    self.xvj(&mut fa, bbb - BX_ - BF_, jgr)?;
                }
                jgr
            } else {
                clw
            };
            
            let jk = BC_ + clw as u64;
            let aiw = core::cmp::v(H_ - emw, k.len() - cjm);
            
            
            let mut aae = [0u8; H_];
            if emw > 0 || aiw < H_ {
                self.xr(jk, &mut aae)?;
            }
            
            aae[emw..emw + aiw]
                .dg(&k[cjm..cjm + aiw]);
            
            self.aby(jk, &aae)?;
            
            cjm += aiw;
            azv += aiw;
        }
        
        
        let brm = core::cmp::am(fa.aw, (l + cjm as u64) as u32);
        if brm != fa.aw {
            fa.aw = brm;
            fa.bnp = (crate::logger::lh() / 100) as u32;
            self.jxh(dd, &fa)?;
        }
        
        Ok(cjm)
    }
    
    
    fn cga(&self, ger: I, j: &str) -> B<I> {
        let ch = self.brx(ger)?;
        for bt in ch {
            if bt.j == j {
                return Ok(bt.dd);
            }
        }
        Err(VfsError::N)
    }
    
    
    fn brx(&self, ger: I) -> B<Vec<Br>> {
        let fa = self.ayn(ger)?;
        if !fa.ta() {
            return Err(VfsError::Lz);
        }
        
        let mut ch = Vec::new();
        let acy = core::mem::size_of::<My>();
        let htd = fa.aw as usize / acy;
        
        for a in 0..htd {
            let l = (a * acy) as u64;
            let mut k = [0u8; 32]; 
            self.mq(ger, l, &mut k)?;
            
            let kts = k.fq() as *const My;
            let irh = unsafe { &*kts };
            
            if irh.fa != 0 {
                let rag = self.ayn(irh.fa as I)?;
                ch.push(Br {
                    j: String::from(irh.amj()),
                    dd: irh.fa as I,
                    kd: rag.kd(),
                });
            }
        }
        
        Ok(ch)
    }
    
    
    fn avp(&self, dak: I, j: &str, kd: FileType) -> B<I> {
        if j.len() > KP_ {
            return Err(VfsError::Pr);
        }
        
        
        if self.cga(dak, j).is_ok() {
            return Err(VfsError::Ri);
        }
        
        
        let lnu = self.qgu()?;
        let mut hsr = DiskInode::default();
        hsr.pjh(kd);
        hsr.fpa = 1;
        hsr.ev |= 0o644; 
        
        if kd == FileType::K {
            hsr.ev |= 0o111; 
        }
        
        self.jxh(lnu, &hsr)?;
        
        
        let mut bt = My {
            fa: lnu as u32,
            j: [0; KP_],
        };
        let bko = j.as_bytes();
        let zg = core::cmp::v(bko.len(), KP_);
        bt.j[..zg].dg(&bko[..zg]);
        
        let vbo = self.ayn(dak)?;
        let l = vbo.aw as u64;
        
        let hif = unsafe {
            core::slice::anh(
                &bt as *const My as *const u8,
                core::mem::size_of::<My>()
            )
        };
        
        self.ns(dak, l, hif)?;
        
        Ok(lnu)
    }
    
    
    fn cnm(&self, dak: I, j: &str) -> B<()> {
        let ch = self.brx(dak)?;
        let acy = core::mem::size_of::<My>();
        
        for (a, bt) in ch.iter().cf() {
            if bt.j == j {
                
                let mut fa = self.ayn(bt.dd)?;
                
                
                if fa.ta() && fa.aw > 0 {
                    let zf = self.brx(bt.dd)?;
                    if !zf.is_empty() {
                        return Err(VfsError::Bnj);
                    }
                }
                
                
                fa.fpa = fa.fpa.ao(1);
                
                if fa.fpa == 0 {
                    
                    if let Err(aa) = self.sxe(&fa) {
                        crate::log_warn!("[TRUSTFS] Warning: failed to free blocks for inode {}: {:?}", bt.dd, aa);
                    }
                    
                    fa.ev = 0;
                    fa.aw = 0;
                    fa.xk = 0;
                    fa.eav = [0; BX_];
                    fa.brd = 0;
                    
                    
                    {
                        let mut is = self.ezg.write();
                        is.ivv += 1;
                    }
                }
                
                self.jxh(bt.dd, &fa)?;
                
                
                let mut xxl = My {
                    fa: 0,
                    j: [0; KP_],
                };
                let l = (a * acy) as u64;
                let hif = unsafe {
                    core::slice::anh(
                        &xxl as *const My as *const u8,
                        acy
                    )
                };
                self.ns(dak, l, hif)?;
                
                return Ok(());
            }
        }
        
        Err(VfsError::N)
    }
    
    
    fn dmu(&self, dd: I, aw: u64) -> B<()> {
        let mut fa = self.ayn(dd)?;
        let jhs = fa.aw as u64;
        let brm = aw;
        
        if brm < jhs {
            
            let uxn = ((jhs + H_ as u64 - 1) / H_ as u64) as usize;
            let opk = ((brm + H_ as u64 - 1) / H_ as u64) as usize;
            
            for bbb in opk..uxn {
                if bbb < BX_ {
                    if fa.eav[bbb] != 0 {
                        let _ = self.ebz(fa.eav[bbb]);
                        fa.eav[bbb] = 0;
                        fa.xk = fa.xk.ao(1);
                    }
                } else if bbb < BX_ + BF_ {
                    if fa.brd != 0 {
                        let mut crm = [0u8; H_];
                        self.xr(BC_ + fa.brd as u64, &mut crm)?;
                        let egy = unsafe { &mut *(crm.mw() as *mut [u32; BF_]) };
                        let w = bbb - BX_;
                        if egy[w] != 0 {
                            let _ = self.ebz(egy[w]);
                            egy[w] = 0;
                            fa.xk = fa.xk.ao(1);
                            self.aby(BC_ + fa.brd as u64, &crm)?;
                        }
                    }
                }
            }
            
            
            if opk <= BX_ && fa.brd != 0 {
                let _ = self.ebz(fa.brd);
                fa.brd = 0;
            }
        }
        
        fa.aw = brm as u32;
        fa.bnp = (crate::logger::lh() / 100) as u32;
        self.jxh(dd, &fa)
    }
    
    
    fn hm(&self, dd: I) -> B<Stat> {
        let fa = self.ayn(dd)?;
        Ok(Stat {
            dd,
            kd: fa.kd(),
            aw: fa.aw as u64,
            xk: fa.xk as u64,
            py: H_ as u32,
            ev: fa.ev as u32,
            pi: 0,
            pw: 0,
            byi: fa.byi as u64,
            bnp: fa.bnp as u64,
            cpq: 0,
        })
    }
    
    
    fn sync(&self) -> B<()> {
        
        self.svd()?;
        
        let _ = super::block_cache::sync();
        
        let is = self.ezg.read();
        let mut k = [0u8; H_];
        let hys = k.mw() as *mut Superblock;
        unsafe { *hys = *is; }
        self.aby(AIL_, &k)?;
        
        *self.no.write() = false;
        crate::log_debug!("[TrustFS] sync complete");
        Ok(())
    }
}


pub struct TrustFs {
    ff: Arc<Afa>,
}

impl TrustFs {
    
    pub fn new(backend: Arc<dyn Bj>, aty: u64) -> B<Self> {
        
        let mut k = [0u8; H_];
        backend.xr(AIL_, &mut k)
            .jd(|_| VfsError::Av)?;
        
        let hys = k.fq() as *const Superblock;
        let nrs = unsafe { *hys };
        
        let ezg = if nrs.cld() {
            crate::log_debug!("[TrustFS] Found existing filesystem");
            nrs
        } else {
            crate::log!("[TrustFS] Formatting new filesystem...");
            Self::swd(&*backend, aty)?
        };
        
        
        let mxm = &*backend;
        let vxi = |jk: u64, k: &mut [u8; H_]| -> Result<(), ()> {
            mxm.xr(jk, k).jd(|_| ())
        };
        let vxj = |jk: u64, f: &[u8; H_]| -> Result<(), ()> {
            mxm.aby(jk, f).jd(|_| ())
        };
        match super::wal::vxh(&vxi, &vxj) {
            Ok(0) => {},
            Ok(bo) => crate::log!("[TrustFS] WAL replay: {} writes recovered", bo),
            Err(_) => crate::log_warn!("[TrustFS] WAL replay failed"),
        }

        let ff = Arc::new(Afa {
            ezg: RwLock::new(ezg),
            jaf: RwLock::new(InodeCache::new()),
            no: RwLock::new(false),
            backend,
        });
        
        Ok(Self { ff })
    }
    
    
    fn swd(backend: &dyn Bj, aty: u64) -> B<Superblock> {
        let is = Superblock::new(aty);
        
        
        let mut k = [0u8; H_];
        let hys = k.mw() as *mut Superblock;
        unsafe { *hys = is; }
        backend.aby(AIL_, &k)
            .jd(|_| VfsError::Av)?;
        
        
        let qbb = [0u8; H_];
        for a in 0..CBS_ {
            backend.aby(UF_ + a, &qbb)
                .jd(|_| VfsError::Av)?;
        }
        
        
        for a in 0..ZL_ {
            backend.aby(LW_ + a, &qbb)
                .jd(|_| VfsError::Av)?;
        }
        
        
        let mut cbm = DiskInode::default();
        cbm.pjh(FileType::K);
        cbm.fpa = 1;
        cbm.ev |= 0o755;
        
        let tuu = UF_;
        let mut ler = [0u8; H_];
        let flm = ler[core::mem::size_of::<DiskInode>()..].mw() as *mut DiskInode;
        unsafe { *flm = cbm; }
        
        let vzy = core::mem::size_of::<DiskInode>(); 
        let flm = ler[vzy..].mw() as *mut DiskInode;
        unsafe { *flm = cbm; }
        backend.aby(tuu, &ler)
            .jd(|_| VfsError::Av)?;
        
        crate::log!("[TrustFS] Formatted: {} blocks, {} inodes", is.fxa, is.pva);
        
        Ok(is)
    }
}

impl Cc for TrustFs {
    fn j(&self) -> &str {
        "trustfs"
    }
    
    fn cbm(&self) -> I {
        1
    }
    
    fn era(&self, dd: I) -> B<Arc<dyn Et>> {
        let fa = self.ff.ayn(dd)?;
        if fa.ta() {
            return Err(VfsError::Tc);
        }
        Ok(Arc::new(Bus {
            fs: Arc::clone(&self.ff),
            dd,
        }))
    }
    
    fn dhl(&self, dd: I) -> B<Arc<dyn Ep>> {
        let fa = self.ff.ayn(dd)?;
        if !fa.ta() {
            return Err(VfsError::Lz);
        }
        Ok(Arc::new(Bur {
            fs: Arc::clone(&self.ff),
            dd,
        }))
    }
    
    fn hm(&self, dd: I) -> B<Stat> {
        self.ff.hm(dd)
    }
    
    fn sync(&self) -> B<()> {
        self.ff.sync()
    }
}
