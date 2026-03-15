










use alloc::string::{String, Gd};
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

const H_: usize = 512;


#[repr(C, packed)]
#[derive(Clone, Copy)]
struct Asb {
    uak: [u8; 3],
    zec: [u8; 8],
    aid: u16,
    anx: u8,
    lzo: u16,
    dtr: u8,
    zki: u16,  
    zti: u16,  
    zcp: u8,
    yqg: u16,       
    wge: u16,
    uwj: u16,
    tor: u32,
    mmm: u32,
    
    kvf: u32,
    ypw: u16,
    yrx: u16,
    jms: u32,
    yrv: u16,
    yfh: u16,
    awt: [u8; 12],
    ynq: u8,
    pco: u8,
    ygt: u8,
    zvs: u32,
    zvt: [u8; 11],
    yrw: [u8; 8],
}

impl Asb {
    fn cld(&self) -> bool {
        
        let aid = unsafe { core::ptr::md(core::ptr::vf!(self.aid)) };
        let anx = self.anx;
        let awt = unsafe { core::ptr::md(core::ptr::vf!(self.lzo)) };
        let dtr = self.dtr;
        
        aid >= 512 && 
        aid <= 4096 &&
        anx >= 1 &&
        anx <= 128 &&
        awt >= 1 &&
        dtr >= 1
    }
    
    fn qt(&self) -> usize {
        let hbc = unsafe { core::ptr::md(core::ptr::vf!(self.aid)) } as usize;
        let ibb = self.anx as usize;
        hbc * ibb
    }
    
    fn nut(&self) -> u64 {
        let awt = unsafe { core::ptr::md(core::ptr::vf!(self.lzo)) } as u64;
        let dtr = self.dtr as u64;
        let hiy = unsafe { core::ptr::md(core::ptr::vf!(self.kvf)) } as u64;
        awt + (dtr * hiy)
    }
    
    fn kwl(&self) -> u64 {
        (unsafe { core::ptr::md(core::ptr::vf!(self.lzo)) }) as u64
    }
    
    fn nds(&self, ry: u32) -> u64 {
        let ibb = self.anx as u64;
        self.nut() + ((ry - 2) as u64 * ibb)
    }
}


#[repr(C, packed)]
#[derive(Clone, Copy)]
struct Fat32DirEntry {
    j: [u8; 8],
    wm: [u8; 3],
    qn: u8,
    lpf: u8,
    kly: u8,
    klx: u16,
    klv: u16,
    jyx: u16,
    hda: u16,
    lmm: u16,
    lml: u16,
    hdb: u16,
    yy: u32,
}

impl Fat32DirEntry {
    const DCU_: u8 = 0x01;
    const DCT_: u8 = 0x02;
    const DCV_: u8 = 0x04;
    const BKR_: u8 = 0x08;
    const LU_: u8 = 0x10;
    const BKM_: u8 = 0x20;
    const AKY_: u8 = 0x0F;
    
    fn ofz(&self) -> bool {
        self.j[0] == 0x00 || self.j[0] == 0xE5
    }
    
    fn ofw(&self) -> bool {
        self.j[0] == 0x00
    }
    
    fn lgg(&self) -> bool {
        (self.qn & Self::AKY_) == Self::AKY_
    }
    
    fn cfr(&self) -> bool {
        (self.qn & Self::LU_) != 0
    }
    
    fn ogy(&self) -> bool {
        (self.qn & Self::BKR_) != 0 && !self.lgg()
    }
    
    fn ry(&self) -> u32 {
        let gd = unsafe { core::ptr::md(core::ptr::vf!(self.hda)) } as u32;
        let hh = unsafe { core::ptr::md(core::ptr::vf!(self.hdb)) } as u32;
        (gd << 16) | hh
    }
    
    fn aw(&self) -> u32 {
        unsafe { core::ptr::md(core::ptr::vf!(self.yy)) }
    }
    
    fn nyn(&self) -> String {
        let bko = self.j;
        let spp = self.wm;
        
        
        let j: String = bko.iter()
            .fwc(|&&r| r != b' ' && r != 0)
            .map(|&r| {
                if r == 0x05 { 0xE5 as char } 
                else { r as char }
            })
            .collect();
        
        let wm: String = spp.iter()
            .fwc(|&&r| r != b' ' && r != 0)
            .map(|&r| r as char)
            .collect();
        
        if wm.is_empty() {
            j
        } else {
            alloc::format!("{}.{}", j, wm)
        }
    }
}


#[repr(C, packed)]
#[derive(Clone, Copy)]
struct Bgu {
    gon: u8,
    lng: [u16; 5],  
    qn: u8,         
    zar: u8,     
    bmj: u8,
    lnh: [u16; 6],  
    ry: u16,     
    lni: [u16; 2],  
}

impl Bgu {
    fn tcz(&self) -> Vec<char> {
        let mut bw = Vec::fc(13);
        
        
        let lng = unsafe { core::ptr::md(core::ptr::vf!(self.lng)) };
        let lnh = unsafe { core::ptr::md(core::ptr::vf!(self.lnh)) };
        let lni = unsafe { core::ptr::md(core::ptr::vf!(self.lni)) };
        
        for &r in &lng {
            if r == 0 || r == 0xFFFF { return bw; }
            bw.push(char::zi(r as u32).unwrap_or('?'));
        }
        for &r in &lnh {
            if r == 0 || r == 0xFFFF { return bw; }
            bw.push(char::zi(r as u32).unwrap_or('?'));
        }
        for &r in &lni {
            if r == 0 || r == 0xFFFF { return bw; }
            bw.push(char::zi(r as u32).unwrap_or('?'));
        }
        
        bw
    }
    
    fn gon(&self) -> u8 {
        self.gon & 0x3F
    }
    
    fn fmd(&self) -> bool {
        (self.gon & 0x40) != 0
    }
}


const ARQ_: u32 = 0x00000000;
const BVB_: u32 = 0x0FFFFFF8;  
const BVA_: u32 = 0x0FFFFFF7;


struct Sm {
    ry: u32,
    aw: u64,
    ta: bool,
    j: String,
    
    bmw: u32,
    
    eou: usize,
}


pub trait Bj: Send + Sync {
    fn xr(&self, jk: u64, bi: &mut [u8]) -> Result<(), ()>;
    fn aby(&self, jk: u64, bi: &[u8]) -> Result<(), ()>;
    fn zn(&self) -> usize { H_ }
}


pub trait Byn: Bj {}
impl<T: Bj> Byn for T {}


pub struct Bvw;

impl Bj for Bvw {
    fn xr(&self, jk: u64, bi: &mut [u8]) -> Result<(), ()> {
        
        crate::virtio_blk::ain(jk, 1, bi)
            .jd(|_| ())
    }
    
    fn aby(&self, jk: u64, bi: &[u8]) -> Result<(), ()> {
        crate::virtio_blk::bpi(jk, 1, bi)
            .jd(|_| ())
    }
}


pub struct AhciBlockReader {
    port: usize,
    jiv: u64,  
}

impl AhciBlockReader {
    pub fn new(port: usize, jiv: u64) -> Self {
        Self { port, jiv }
    }
}

impl Bj for AhciBlockReader {
    fn xr(&self, jk: u64, bi: &mut [u8]) -> Result<(), ()> {
        let jyu = self.jiv + jk;
        crate::drivers::ahci::ain(self.port as u8, jyu, 1, bi)
            .map(|_| ())
            .jd(|_| ())
    }
    
    fn aby(&self, jk: u64, bi: &[u8]) -> Result<(), ()> {
        let jyu = self.jiv + jk;
        crate::drivers::ahci::bpi(self.port as u8, jyu, 1, bi)
            .map(|_| ())
            .jd(|_| ())
    }
}


pub struct Fat32Fs {
    cha: Arc<dyn Bj>,
    agh: Asb,
    arj: RwLock<BTreeMap<I, Sm>>,
    hsv: AtomicU64,
    pea: I,
}

impl Fat32Fs {
    
    pub fn beu(cha: Arc<dyn Bj>) -> B<Self> {
        
        let mut cvz = [0u8; H_];
        cha.xr(0, &mut cvz)
            .jd(|_| VfsError::Av)?;
        
        
        if cvz[510] != 0x55 || cvz[511] != 0xAA {
            crate::log_warn!("[FAT32] Invalid boot signature");
            return Err(VfsError::Pr);
        }
        
        
        let agh = unsafe { 
            core::ptr::md(cvz.fq() as *const Asb)
        };
        
        if !agh.cld() {
            crate::log_warn!("[FAT32] Invalid BPB");
            return Err(VfsError::Pr);
        }
        
        let jms = { agh.jms };
        let aid = { agh.aid };
        let anx = { agh.anx };
        
        crate::log!("[FAT32] Mounted: {} bytes/sector, {} sectors/cluster, root cluster {}",
            aid, anx, jms);
        
        let fs = Self {
            cha,
            agh,
            arj: RwLock::new(BTreeMap::new()),
            hsv: AtomicU64::new(2),
            pea: 1,
        };
        
        
        {
            let mut arj = fs.arj.write();
            arj.insert(1, Sm {
                ry: jms,
                aw: 0,
                ta: true,
                j: String::from("/"),
                bmw: 0,
                eou: 0,
            });
        }
        
        Ok(fs)
    }
    
    
    fn gqj(&self, ry: u32) -> B<Vec<u8>> {
        let jk = self.agh.nds(ry);
        let anx = { self.agh.anx } as u64;
        let qt = self.agh.qt();
        
        let mut f = vec![0u8; qt];
        
        for a in 0..anx {
            let l = (a as usize) * H_;
            self.cha.xr(jk + a, &mut f[l..l + H_])
                .jd(|_| VfsError::Av)?;
        }
        
        Ok(f)
    }
    
    
    fn vrr(&self, ry: u32) -> B<u32> {
        let hiw = ry * 4;
        let aid = { self.agh.aid } as u32;
        let hix = self.agh.kwl() + (hiw / aid) as u64;
        let bho = (hiw % aid) as usize;
        
        let mut aae = [0u8; H_];
        self.cha.xr(hix, &mut aae)
            .jd(|_| VfsError::Av)?;
        
        let bt = u32::dj([
            aae[bho],
            aae[bho + 1],
            aae[bho + 2],
            aae[bho + 3],
        ]) & 0x0FFFFFFF;  
        
        Ok(bt)
    }
    
    
    fn ghx(&self, dly: u32) -> B<Vec<u32>> {
        let mut rh = Vec::new();
        let mut cv = dly;
        
        while cv >= 2 && cv < BVA_ {
            rh.push(cv);
            cv = self.vrr(cv)?;
            
            
            if rh.len() > 1_000_000 {
                return Err(VfsError::Av);
            }
        }
        
        Ok(rh)
    }
    
    
    fn ozv(&self, dly: u32, aw: Option<u64>) -> B<Vec<u8>> {
        let rh = self.ghx(dly)?;
        let qt = self.agh.qt();
        let aay = aw.unwrap_or((rh.len() * qt) as u64) as usize;
        
        let mut f = Vec::fc(aay);
        
        for ry in rh {
            let fez = self.gqj(ry)?;
            f.bk(&fez);
            if f.len() >= aay {
                break;
            }
        }
        
        f.dmu(aay);
        Ok(f)
    }
    
    
    fn gwx(&self, ry: u32, f: &[u8]) -> B<()> {
        let jk = self.agh.nds(ry);
        let anx = { self.agh.anx } as u64;
        let qt = self.agh.qt();
        
        if f.len() < qt {
            return Err(VfsError::Av);
        }
        
        for a in 0..anx {
            let l = (a as usize) * H_;
            self.cha.aby(jk + a, &f[l..l + H_])
                .jd(|_| VfsError::Av)?;
        }
        
        Ok(())
    }
    
    
    fn mrb(&self, ry: u32, bn: u32) -> B<()> {
        let hiw = ry * 4;
        let aid = { self.agh.aid } as u32;
        let hix = self.agh.kwl() + (hiw / aid) as u64;
        let bho = (hiw % aid) as usize;
        
        
        let mut aae = [0u8; H_];
        self.cha.xr(hix, &mut aae)
            .jd(|_| VfsError::Av)?;
        
        
        let loe = (bn & 0x0FFFFFFF) | 
                        (u32::dj([aae[bho], 
                                            aae[bho + 1],
                                            aae[bho + 2], 
                                            aae[bho + 3]]) & 0xF0000000);
        
        aae[bho..bho + 4]
            .dg(&loe.ho());
        
        
        self.cha.aby(hix, &aae)
            .jd(|_| VfsError::Av)?;
        
        
        let dtr = { self.agh.dtr } as u64;
        if dtr > 1 {
            let hiy = unsafe { core::ptr::md(core::ptr::vf!(self.agh.kvf)) } as u64;
            for srd in 1..dtr {
                let qml = hix + srd * hiy;
                let _ = self.cha.aby(qml, &aae);
            }
        }
        
        Ok(())
    }
    
    
    fn ijp(&self) -> B<u32> {
        let aid = { self.agh.aid } as u32;
        let sre = self.agh.kwl();
        let hiy = unsafe { core::ptr::md(core::ptr::vf!(self.agh.kvf)) };
        let mmm = unsafe { core::ptr::md(core::ptr::vf!(self.agh.mmm)) };
        let rtp = mmm as u64 - self.agh.nut();
        let ibb = self.agh.anx as u64;
        let xjx = (rtp / ibb) as u32 + 2;
        
        
        let ggh = aid / 4;
        let mut aae = [0u8; H_];
        
        for cmu in 0..hiy {
            self.cha.xr(sre + cmu as u64, &mut aae)
                .jd(|_| VfsError::Av)?;
            
            for bea in 0..ggh {
                let ry = cmu * ggh + bea;
                if ry < 2 || ry >= xjx {
                    continue;
                }
                
                let l = (bea * 4) as usize;
                let bn = u32::dj([
                    aae[l],
                    aae[l + 1],
                    aae[l + 2],
                    aae[l + 3],
                ]) & 0x0FFFFFFF;
                
                if bn == ARQ_ {
                    
                    self.mrb(ry, BVB_)?;
                    return Ok(ry);
                }
            }
        }
        
        Err(VfsError::Tq)
    }
    
    
    fn nsd(&self, ucc: u32) -> B<u32> {
        let bhm = self.ijp()?;
        self.mrb(ucc, bhm)?;
        Ok(bhm)
    }
    
    
    fn xvk(&self, dly: u32, l: u64, f: &[u8], knc: u64) -> B<(u32, u64)> {
        let qt = self.agh.qt();
        let mut rh = self.ghx(dly)?;
        
        
        let epw = l + f.len() as u64;
        let kie = ((epw + qt as u64 - 1) / qt as u64) as usize;
        
        
        while rh.len() < kie {
            let qv = *rh.qv().unwrap_or(&dly);
            let bhm = if rh.is_empty() {
                self.ijp()?
            } else {
                self.nsd(qv)?
            };
            rh.push(bhm);
        }
        
        
        let mut ia = f;
        let mut fbt = l as usize;
        
        for &ry in &rh {
            let ioe = (rh.iter().qf(|&r| r == ry).unwrap()) * qt;
            let rbw = ioe + qt;
            
            if fbt >= rbw || ia.is_empty() {
                continue;
            }
            
            if fbt < ioe {
                fbt = ioe;
            }
            
            
            let mut fez = self.gqj(ry)?;
            
            
            let dke = fbt - ioe;
            let wqm = qt - dke;
            let dwy = core::cmp::v(wqm, ia.len());
            
            
            fez[dke..dke + dwy]
                .dg(&ia[..dwy]);
            
            
            self.gwx(ry, &fez)?;
            
            ia = &ia[dwy..];
            fbt += dwy;
        }
        
        let brm = core::cmp::am(knc, epw);
        let lod = *rh.fv().unwrap_or(&dly);
        
        Ok((lod, brm))
    }
    
    
    fn tcl(uig: &str) -> [u8; 11] {
        let mut iag = [b' '; 11];
        let juw = uig.idx();
        
        
        let (urh, spt) = if let Some(fgw) = juw.bhx('.') {
            (&juw[..fgw], &juw[fgw + 1..])
        } else {
            (juw.as_str(), "")
        };
        
        
        for (a, bm) in urh.bw().hi(|r| r.bvb() || *r == '_').take(8).cf() {
            iag[a] = bm as u8;
        }
        
        
        for (a, bm) in spt.bw().hi(|r| r.bvb()).take(3).cf() {
            iag[8 + a] = bm as u8;
        }
        
        iag
    }
    
    
    fn stf(&self, bmw: u32) -> B<(u32, usize)> {
        let qt = self.agh.qt();
        let acy = core::mem::size_of::<Fat32DirEntry>();
        let isu = qt / acy;
        
        let rh = self.ghx(bmw)?;
        
        for &ry in &rh {
            let f = self.gqj(ry)?;
            
            for a in 0..isu {
                let l = a * acy;
                let iuu = f[l];
                
                if iuu == 0x00 || iuu == 0xE5 {
                    return Ok((ry, l));
                }
            }
        }
        
        
        let qv = *rh.qv().unwrap_or(&bmw);
        let bhm = self.nsd(qv)?;
        
        
        let xxk = vec![0u8; qt];
        self.gwx(bhm, &xxk)?;
        
        Ok((bhm, 0))
    }
    
    
    fn rqn(&self, bmw: u32, j: &str, ta: bool) -> B<(u32, u64)> {
        let (nqo, bql) = self.stf(bmw)?;
        
        
        let bhm = if ta {
            let ry = self.ijp()?;
            
            let qt = self.agh.qt();
            let mut kpz = vec![0u8; qt];
            
            
            let sai = Fat32DirEntry {
                j: [b'.', b' ', b' ', b' ', b' ', b' ', b' ', b' '],
                wm: [b' ', b' ', b' '],
                qn: Fat32DirEntry::LU_,
                lpf: 0,
                kly: 0,
                klx: 0,
                klv: 0,
                jyx: 0,
                hda: ((ry >> 16) & 0xFFFF) as u16,
                lmm: 0,
                lml: 0,
                hdb: (ry & 0xFFFF) as u16,
                yy: 0,
            };
            
            
            let sal = Fat32DirEntry {
                j: [b'.', b'.', b' ', b' ', b' ', b' ', b' ', b' '],
                wm: [b' ', b' ', b' '],
                qn: Fat32DirEntry::LU_,
                lpf: 0,
                kly: 0,
                klx: 0,
                klv: 0,
                jyx: 0,
                hda: ((bmw >> 16) & 0xFFFF) as u16,
                lmm: 0,
                lml: 0,
                hdb: (bmw & 0xFFFF) as u16,
                yy: 0,
            };
            
            let acy = core::mem::size_of::<Fat32DirEntry>();
            unsafe {
                core::ptr::write(kpz.mw() as *mut Fat32DirEntry, sai);
                core::ptr::write((kpz.mw().add(acy)) as *mut Fat32DirEntry, sal);
            }
            
            self.gwx(ry, &kpz)?;
            ry
        } else {
            0 
        };
        
        
        let dbz = Self::tcl(j);
        let daf = Fat32DirEntry {
            j: dbz[..8].try_into().unwrap_or([b' '; 8]),
            wm: dbz[8..11].try_into().unwrap_or([b' '; 3]),
            qn: if ta { Fat32DirEntry::LU_ } else { Fat32DirEntry::BKM_ },
            lpf: 0,
            kly: 0,
            klx: 0,
            klv: 0,
            jyx: 0,
            hda: ((bhm >> 16) & 0xFFFF) as u16,
            lmm: 0,
            lml: 0,
            hdb: (bhm & 0xFFFF) as u16,
            yy: 0,
        };
        
        
        let mut fez = self.gqj(nqo)?;
        unsafe {
            core::ptr::write(
                fez.mw().add(bql) as *mut Fat32DirEntry,
                daf
            );
        }
        self.gwx(nqo, &fez)?;
        
        Ok((bhm, 0))
    }
    
    
    fn xot(&self, dd: I, bhm: u32, brm: u64) -> B<()> {
        let arj = self.arj.read();
        let fa = arj.get(&dd).ok_or(VfsError::N)?;
        let bmw = fa.bmw;
        let nlp = fa.eou;
        drop(arj);

        if bmw == 0 {
            
            return Ok(());
        }

        let acy = core::mem::size_of::<Fat32DirEntry>();
        let qt = self.agh.qt();
        let isu = qt / acy;

        
        let rh = self.ghx(bmw)?;
        let ndr = nlp / qt;
        let dke = nlp % qt;

        if ndr >= rh.len() {
            return Err(VfsError::Av);
        }

        let pry = rh[ndr];
        let mut f = self.gqj(pry)?;

        
        let bt = unsafe {
            &mut *(f.mw().add(dke) as *mut Fat32DirEntry)
        };

        
        bt.hdb = (bhm & 0xFFFF) as u16;
        bt.hda = ((bhm >> 16) & 0xFFFF) as u16;

        
        if bt.qn & Fat32DirEntry::LU_ == 0 {
            bt.yy = brm as u32;
        }

        
        self.gwx(pry, &f)?;

        Ok(())
    }
    
    
    fn rvi(&self, bmw: u32, j: &str) -> B<()> {
        let qt = self.agh.qt();
        let acy = core::mem::size_of::<Fat32DirEntry>();
        let isu = qt / acy;
        
        let rh = self.ghx(bmw)?;
        
        for &ry in &rh {
            let mut f = self.gqj(ry)?;
            
            for a in 0..isu {
                let l = a * acy;
                let bt = unsafe {
                    core::ptr::md(f[l..].fq() as *const Fat32DirEntry)
                };
                
                if bt.ofw() {
                    return Err(VfsError::N);
                }
                
                if !bt.ofz() && !bt.lgg() && !bt.ogy() {
                    let cxm = bt.nyn();
                    if cxm.dha(j) {
                        
                        f[l] = 0xE5;
                        self.gwx(ry, &f)?;
                        
                        
                        let ntk = bt.ry();
                        if ntk >= 2 {
                            self.sxc(ntk)?;
                        }
                        
                        return Ok(());
                    }
                }
            }
        }
        
        Err(VfsError::N)
    }
    
    
    fn sxc(&self, dly: u32) -> B<()> {
        let rh = self.ghx(dly)?;
        
        for ry in rh {
            self.mrb(ry, ARQ_)?;
        }
        
        Ok(())
    }
    
    
    fn lsh(&self, ry: u32) -> B<Vec<(String, Fat32DirEntry, usize)>> {
        let f = self.ozv(ry, None)?;
        let acy = core::mem::size_of::<Fat32DirEntry>();
        let mut ch = Vec::new();
        let mut glk: Vec<(u8, Vec<char>)> = Vec::new();
        
        let mut a = 0;
        while a + acy <= f.len() {
            let bt = unsafe {
                core::ptr::md(f[a..].fq() as *const Fat32DirEntry)
            };
            
            if bt.ofw() {
                break;
            }
            
            if bt.ofz() {
                glk.clear();
                a += acy;
                continue;
            }
            
            if bt.lgg() {
                
                let oiz = unsafe {
                    core::ptr::md(f[a..].fq() as *const Bgu)
                };
                glk.push((oiz.gon(), oiz.tcz()));
            } else if !bt.ogy() {
                
                let j = if !glk.is_empty() {
                    
                    glk.bxf(|(gon, _)| *gon);
                    let ghr: String = glk.iter()
                        .iva(|(_, bw)| bw.iter())
                        .collect();
                    glk.clear();
                    ghr
                } else {
                    bt.nyn()
                };
                
                
                if j != "." && j != ".." {
                    ch.push((j, bt, a));
                }
            }
            
            a += acy;
        }
        
        Ok(ch)
    }
    
    fn kyt(&self, dd: I) -> B<Sm> {
        let arj = self.arj.read();
        arj.get(&dd).abn().ok_or(VfsError::N)
    }
    
    fn kaa(&self) -> I {
        self.hsv.fetch_add(1, Ordering::SeqCst)
    }
}


impl Clone for Sm {
    fn clone(&self) -> Self {
        Self {
            ry: self.ry,
            aw: self.aw,
            ta: self.ta,
            bmw: self.bmw,
            eou: self.eou,
            j: self.j.clone(),
        }
    }
}

impl Cc for Fat32Fs {
    fn j(&self) -> &str {
        "fat32"
    }
    
    fn cbm(&self) -> I {
        self.pea
    }
    
    fn era(&self, dd: I) -> B<Arc<dyn Et>> {
        let fa = self.kyt(dd)?;
        if fa.ta {
            return Err(VfsError::Tc);
        }
        
        Ok(Arc::new(Aid {
            fs: self as *const Fat32Fs,
            dd,
            ry: RwLock::new(fa.ry),
            aw: RwLock::new(fa.aw),
        }))
    }
    
    fn dhl(&self, dd: I) -> B<Arc<dyn Ep>> {
        let fa = self.kyt(dd)?;
        if !fa.ta {
            return Err(VfsError::Lz);
        }
        
        Ok(Arc::new(Aic {
            fs: self as *const Fat32Fs,
            dd,
            ry: fa.ry,
        }))
    }
    
    fn hm(&self, dd: I) -> B<Stat> {
        let fa = self.kyt(dd)?;
        
        Ok(Stat {
            dd,
            kd: if fa.ta { FileType::K } else { FileType::Ea },
            aw: fa.aw,
            xk: (fa.aw + 511) / 512,
            py: self.agh.qt() as u32,
            ev: if fa.ta { 0o755 } else { 0o644 },
            pi: 0,
            pw: 0,
            byi: 0,
            bnp: 0,
            cpq: 0,
        })
    }
}


struct Aid {
    fs: *const Fat32Fs,
    dd: I,
    ry: RwLock<u32>,
    aw: RwLock<u64>,
}

unsafe impl Send for Aid {}
unsafe impl Sync for Aid {}

impl Et for Aid {
    fn read(&self, l: u64, k: &mut [u8]) -> B<usize> {
        let aw = *self.aw.read();
        let ry = *self.ry.read();
        
        if l >= aw {
            return Ok(0);
        }
        
        let fs = unsafe { &*self.fs };
        let ajp = core::cmp::v(k.len() as u64, aw - l) as usize;
        
        if ry < 2 {
            
            return Ok(0);
        }
        
        
        let f = fs.ozv(ry, Some(aw))?;
        
        let ay = l as usize;
        let ci = ay + ajp;
        
        if ci <= f.len() {
            k[..ajp].dg(&f[ay..ci]);
            Ok(ajp)
        } else {
            Err(VfsError::Av)
        }
    }
    
    fn write(&self, l: u64, k: &[u8]) -> B<usize> {
        if k.is_empty() {
            return Ok(0);
        }
        
        let fs = unsafe { &*self.fs };
        let nif = *self.ry.read();
        let knc = *self.aw.read();
        
        
        let dly = if nif < 2 {
            let bhm = fs.ijp()?;
            *self.ry.write() = bhm;
            bhm
        } else {
            nif
        };
        
        
        let (bhm, brm) = fs.xvk(dly, l, k, knc)?;
        
        
        *self.ry.write() = bhm;
        *self.aw.write() = brm;
        
        
        if let Some(mut arj) = fs.arj.ifb() {
            if let Some(fa) = arj.ds(&self.dd) {
                fa.ry = bhm;
                fa.aw = brm;
            }
        }
        
        
        let _ = fs.xot(self.dd, bhm, brm);
        
        Ok(k.len())
    }
    
    fn hm(&self) -> B<Stat> {
        let aw = *self.aw.read();
        Ok(Stat {
            dd: self.dd,
            kd: FileType::Ea,
            aw,
            xk: (aw + 511) / 512,
            py: 512,
            ev: 0o644,
            pi: 0,
            pw: 0,
            byi: 0,
            bnp: 0,
            cpq: 0,
        })
    }
}


struct Aic {
    fs: *const Fat32Fs,
    dd: I,
    ry: u32,
}

unsafe impl Send for Aic {}
unsafe impl Sync for Aic {}

impl Ep for Aic {
    fn cga(&self, j: &str) -> B<I> {
        let fs = unsafe { &*self.fs };
        
        
        {
            let arj = fs.arj.read();
            for (dd, fa) in arj.iter() {
                if fa.j.dha(j) {
                    return Ok(*dd);
                }
            }
        }
        
        
        let ch = fs.lsh(self.ry)?;
        
        for (cxm, bt, aok) in ch {
            if cxm.dha(j) {
                let dd = fs.kaa();
                let fa = Sm {
                    ry: bt.ry(),
                    aw: bt.aw() as u64,
                    ta: bt.cfr(),
                    j: cxm,
                    bmw: self.ry,
                    eou: aok,
                };
                fs.arj.write().insert(dd, fa);
                return Ok(dd);
            }
        }
        
        Err(VfsError::N)
    }
    
    fn brx(&self) -> B<Vec<Br>> {
        let fs = unsafe { &*self.fs };
        let ch = fs.lsh(self.ry)?;
        
        let mut result = Vec::new();
        
        for (j, bt, aok) in ch {
            let dd = fs.kaa();
            let ta = bt.cfr();
            
            
            fs.arj.write().insert(dd, Sm {
                ry: bt.ry(),
                aw: bt.aw() as u64,
                ta,
                j: j.clone(),
                bmw: self.ry,
                eou: aok,
            });
            
            result.push(Br {
                j,
                dd,
                kd: if ta { FileType::K } else { FileType::Ea },
            });
        }
        
        Ok(result)
    }
    
    fn avp(&self, j: &str, kd: FileType) -> B<I> {
        let fs = unsafe { &*self.fs };
        
        
        if self.cga(j).is_ok() {
            return Err(VfsError::Ri);
        }
        
        let ta = oh!(kd, FileType::K);
        
        
        let (ry, dds) = fs.rqn(self.ry, j, ta)?;
        
        
        let eou = if let Ok(ch) = fs.lsh(self.ry) {
            ch.iter()
                .du(|(bo, _, _)| bo.dha(j))
                .map(|(_, _, dz)| *dz)
                .unwrap_or(0)
        } else {
            0
        };
        
        
        let dd = fs.kaa();
        fs.arj.write().insert(dd, Sm {
            ry,
            aw: 0,
            ta,
            j: j.to_string(),
            bmw: self.ry,
            eou,
        });
        
        Ok(dd)
    }
    
    fn cnm(&self, j: &str) -> B<()> {
        let fs = unsafe { &*self.fs };
        
        
        if j == "." || j == ".." {
            return Err(VfsError::Pr);
        }
        
        
        fs.rvi(self.ry, j)?;
        
        
        let mut arj = fs.arj.write();
        let cik: Vec<I> = arj.iter()
            .hi(|(_, fa)| fa.j.dha(j))
            .map(|(dd, _)| *dd)
            .collect();
        
        for dd in cik {
            arj.remove(&dd);
        }
        
        Ok(())
    }
    
    fn hm(&self) -> B<Stat> {
        let fs = unsafe { &*self.fs };
        fs.hm(self.dd)
    }
}


pub fn xmq() -> Option<Arc<Fat32Fs>> {
    use crate::drivers::partition::{hul, PartitionType};
    use crate::drivers::ahci;
    
    
    let ik = ahci::bhh();
    crate::log_debug!("[FAT32] Checking {} AHCI devices", ik.len());
    
    for de in ik {
        let port = de.kg;
        let axf = de.agw;
        crate::log_debug!("[FAT32] Port {}: {} sectors", port, axf);
        
        
        let dld = |jk: u64, k: &mut [u8]| -> Result<(), &'static str> {
            ahci::ain(port, jk, 1, k)
                .map(|_| ())
        };
        
        
        let vsu = Arc::new(AhciBlockReader::new(port as usize, 0));
        if let Ok(fs) = Fat32Fs::beu(vsu) {
            crate::log!("[FAT32] Mounted superfloppy FAT32 from port {}", port);
            return Some(Arc::new(fs));
        }
        
        
        if let Ok(gg) = hul(dld, axf) {
            for partition in &gg.aqd {
                
                match partition.duf {
                    PartitionType::Asa | 
                    PartitionType::Asc |
                    PartitionType::Akg => {
                        crate::log!("[FAT32] Found partition at LBA {}", partition.aag);
                        
                        let cha = Arc::new(AhciBlockReader::new(port as usize, partition.aag));
                        
                        match Fat32Fs::beu(cha) {
                            Ok(fs) => {
                                crate::log!("[FAT32] Successfully mounted partition");
                                return Some(Arc::new(fs));
                            }
                            Err(aa) => {
                                crate::log_warn!("[FAT32] Mount failed: {:?}", aa);
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
    }
    
    None
}
