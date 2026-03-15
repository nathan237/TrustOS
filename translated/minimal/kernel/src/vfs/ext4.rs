












use alloc::string::{String, Gd};
use alloc::vec::Vec;
use alloc::vec;
use alloc::sync::Arc;
use alloc::format;
use spin::Mutex;

use super::{Et, Ep, Cc, FileType, Stat, Br, B, VfsError, I};
use super::fat32::Bj;






const ARN_: u16 = 0xEF53;


const XO_: u64 = 1024;


const ARM_: u32 = 0x00080000;


const ABY_: u32 = 2;


const DLL_: u8 = 0;
const BUV_: u8 = 1;
const BUU_: u8 = 2;
const BUT_: u8 = 3;
const BUS_: u8 = 4;
const DLJ_: u8 = 5;
const DLK_: u8 = 6;
const BUW_: u8 = 7;


const BGW_: u16 = 0xF000;
const PV_: u16 = 0x8000;
const AIN_: u16 = 0x4000;
const AIO_: u16 = 0xA000;
const XQ_: u16 = 0x2000;
const CXE_: u16 = 0x6000;


#[repr(C)]
#[derive(Clone, Copy)]
struct Ccq {
    zlb: u32,         
    wbx: u32,      
    zli: u32,    
    zkz: u32, 
    zla: u32,    
    wbz: u32,     
    wcb: u32,       
    zld: u32,     
    jnh: u32,     
    zkr: u32,   
    pez: u32,     
    zlh: u32,                
    zll: u32,                
    zlg: u16,            
    zle: u16,        
    wcc: u16,                
    zlj: u16,                
    zkv: u16,               
    zlf: u16,      
    zlc: u32,            
    zkq: u32,        
    zks: u32,           
    wcd: u32,            
    zku: u16,           
    zkt: u16,           
    
    zky: u32,            
    wca: u16,           
    zkp: u16,       
    zkw: u32,       
    wby: u32,     
    zkx: u32,    
    zlk: [u8; 16],            
    wce: [u8; 16],     
    msr: [u8; 168],            
    
    
}


#[repr(C)]
#[derive(Clone, Copy)]
struct Cwj {
    yfw: u32,     
    yge: u32,     
    ygg: u32,      
    yga: u16,
    ygb: u16,
    ygj: u16,  
    yfz: u16,               
    yfy: u32,   
    yfu: u16,
    ygc: u16,
    ygh: u16,    
    yfx: u16,            
    
    yfv: u32,     
    ygd: u32,     
    ygf: u32,      
}


#[repr(C)]
#[derive(Clone, Copy)]
struct Sl {
    fkv: u16,                 
    hno: u16,                  
    tqy: u32,              
    hnj: u32,                
    hnl: u32,                
    hnn: u32,                
    yxe: u32,                
    hnm: u16,                  
    yxh: u16,          
    hnk: u32,            
    ocv: u32,                
    yxi: u32,                 
    izd: [u32; 15],          
    yxg: u32,           
    yxf: u32,          
    tqx: u32,            
    fzo: [u8; 16],              
}


#[repr(C)]
#[derive(Clone, Copy)]
struct Ccn {
    sjl: u16,      
    sjk: u16,     
    yop: u16,         
    sjj: u16,       
    yoo: u32,  
}


#[repr(C)]
#[derive(Clone, Copy)]
struct Ccm {
    sjc: u32,       
    sjd: u16,         
    sje: u16,    
    sjf: u32,    
}


#[repr(C)]
#[derive(Clone, Copy)]
struct Cco {
    sjo: u32,       
    sjq: u32,     
    sjp: u16,     
    fzo: u16,
}


#[repr(C)]
struct Cwi {
    fa: u32,          
    jlr: u16,        
    baf: u8,        
    kd: u8,       
    
}






pub struct Sj {
    ff: Mutex<Sk>,
}

struct Sk {
    py: u32,
    bac: u16,
    cro: u32,
    dej: u32,
    dhf: u32,
    cfc: u32,
    iei: u32,
    de: Arc<dyn Bj>,
}

impl Sk {
    
    fn day(&self, aok: u64, k: &mut [u8]) -> Result<(), ()> {
        let zn = self.de.zn() as u64;
        let awy = aok / zn;
        let bho = (aok % zn) as usize;
        
        
        let xv = bho + k.len();
        let lpn = (xv + zn as usize - 1) / zn as usize;
        
        let mut aae = vec![0u8; zn as usize];
        let mut ia = k.len();
        let mut avj = 0usize;
        
        for a in 0..lpn {
            self.de.xr(awy + a as u64, &mut aae)?;
            
            let big = if a == 0 { bho } else { 0 };
            let zg = (zn as usize - big).v(ia);
            
            k[avj..avj + zg]
                .dg(&aae[big..big + zg]);
            
            avj += zg;
            ia -= zg;
        }
        
        Ok(())
    }
    
    
    fn jlh(&self, hau: u64, k: &mut [u8]) -> Result<(), ()> {
        let aok = hau * self.py as u64;
        self.day(aok, k)
    }
    
    
    fn tds(&self, cyi: u32) -> Result<u64, ()> {
        
        let tao = if self.py == 1024 { 2 } else { 1 };
        let tap = tao as u64 * self.py as u64
            + cyi as u64 * self.cfc as u64;
        
        let mut eas = [0u8; 64];
        let cgy = (self.cfc as usize).v(64);
        self.day(tap, &mut eas[..cgy])?;
        
        let hh = u32::dj([eas[8], eas[9], eas[10], eas[11]]);
        let gd = if self.cfc >= 64 {
            u32::dj([eas[0x28], eas[0x29], eas[0x2A], eas[0x2B]])
        } else {
            0
        };
        
        Ok(hh as u64 | ((gd as u64) << 32))
    }
    
    
    fn ayn(&self, dd: u32) -> Result<Sl, ()> {
        if dd == 0 { return Err(()); }
        
        let cyi = (dd - 1) / self.cro;
        let index = (dd - 1) % self.cro;
        
        let tuv = self.tds(cyi)?;
        let tut = tuv * self.py as u64
            + index as u64 * self.bac as u64;
        
        let mut k = [0u8; 128];
        self.day(tut, &mut k)?;
        
        Ok(unsafe { core::ptr::md(k.fq() as *const Sl) })
    }
    
    
    fn bac(&self, fa: &Sl) -> u64 {
        let hh = fa.tqy as u64;
        let gd = if fa.fkv & PV_ == PV_ {
            (fa.tqx as u64) << 32
        } else {
            0
        };
        hh | gd
    }
    
    
    fn nse(&self, fa: &Sl, bkg: u32) -> Result<u64, ()> {
        
        let js = unsafe {
            core::slice::anh(
                fa.izd.fq() as *const u8,
                60, 
            )
        };
        
        self.nsf(js, bkg, 4) 
    }
    
    
    fn nsf(&self, f: &[u8], bkg: u32, nkq: u32) -> Result<u64, ()> {
        if f.len() < 12 || nkq == 0 { return Err(()); }
        
        let dh = unsafe { core::ptr::md(f.fq() as *const Ccn) };
        
        if dh.sjl != 0xF30A {
            return Err(()); 
        }
        
        let ch = dh.sjk as usize;
        
        if dh.sjj == 0 {
            
            for a in 0..ch {
                let l = 12 + a * 12;
                if l + 12 > f.len() { break; }
                
                let itm = unsafe {
                    core::ptr::md(f[l..].fq() as *const Ccm)
                };
                
                let ay = itm.sjc;
                let len = itm.sjd as u32;
                
                if bkg >= ay && bkg < ay + len {
                    let ltv = (itm.sje as u64) << 32
                        | itm.sjf as u64;
                    let qqk = (bkg - ay) as u64;
                    return Ok(ltv + qqk);
                }
            }
            Err(()) 
        } else {
            
            let mut prz: Option<(usize, u64)> = None;
            
            for a in 0..ch {
                let l = 12 + a * 12;
                if l + 12 > f.len() { break; }
                
                let w = unsafe {
                    core::ptr::md(f[l..].fq() as *const Cco)
                };
                
                if bkg >= w.sjo {
                    let khk = (w.sjp as u64) << 32
                        | w.sjq as u64;
                    prz = Some((a, khk));
                }
            }
            
            if let Some((_, khk)) = prz {
                let mut ncv = vec![0u8; self.py as usize];
                self.jlh(khk, &mut ncv)?;
                self.nsf(&ncv, bkg, nkq - 1)
            } else {
                Err(())
            }
        }
    }
    
    
    fn hwx(&self, fa: &Sl, azv: u64, k: &mut [u8]) -> Result<usize, ()> {
        let yy = self.bac(fa);
        if azv >= yy {
            return Ok(0);
        }
        
        let cgy = ((yy - azv) as usize).v(k.len());
        if cgy == 0 { return Ok(0); }
        
        let py = self.py as u64;
        let mut ia = cgy;
        let mut avj = 0usize;
        let mut l = azv;
        
        while ia > 0 {
            let bkg = (l / py) as u32;
            let fpo = (l % py) as usize;
            let zg = (py as usize - fpo).v(ia);
            
            if (fa.ocv & ARM_) != 0 {
                match self.nse(fa, bkg) {
                    Ok(clw) => {
                        let mut ajy = vec![0u8; py as usize];
                        self.jlh(clw, &mut ajy)?;
                        k[avj..avj + zg]
                            .dg(&ajy[fpo..fpo + zg]);
                    }
                    Err(()) => {
                        
                        for o in &mut k[avj..avj + zg] {
                            *o = 0;
                        }
                    }
                }
            } else {
                
                match self.ois(fa, bkg) {
                    Some(clw) if clw != 0 => {
                        let mut ajy = vec![0u8; py as usize];
                        self.jlh(clw as u64, &mut ajy)?;
                        k[avj..avj + zg]
                            .dg(&ajy[fpo..fpo + zg]);
                    }
                    _ => {
                        
                        for o in &mut k[avj..avj + zg] {
                            *o = 0;
                        }
                    }
                }
            }
            
            avj += zg;
            l += zg as u64;
            ia -= zg;
        }
        
        Ok(cgy)
    }
    
    
    fn ois(&self, fa: &Sl, bkg: u32) -> Option<u32> {
        let fro = self.py / 4; 
        
        if bkg < 12 {
            
            Some(fa.izd[bkg as usize])
        } else if bkg < 12 + fro {
            
            let oee = fa.izd[12];
            if oee == 0 { return Some(0); }
            self.lxq(oee as u64, (bkg - 12) as usize)
        } else if bkg < 12 + fro + fro * fro {
            
            let nln = fa.izd[13];
            if nln == 0 { return Some(0); }
            let w = bkg - 12 - fro;
            let fmk = w / fro;
            let bvd = w % fro;
            let odz = self.lxq(nln as u64, fmk as usize)?;
            if odz == 0 { return Some(0); }
            self.lxq(odz as u64, bvd as usize)
        } else {
            
            None
        }
    }
    
    
    fn lxq(&self, block: u64, index: usize) -> Option<u32> {
        let aok = block * self.py as u64 + (index * 4) as u64;
        let mut k = [0u8; 4];
        self.day(aok, &mut k).bq()?;
        Some(u32::dj(k))
    }
    
    
    fn exg(&self, dd: u32) -> Result<Vec<(u32, String, u8)>, ()> {
        let fa = self.ayn(dd)?;
        let yy = self.bac(&fa);
        
        let mut ch = Vec::new();
        let mut l = 0u64;
        
        while l < yy {
            let py = self.py as u64;
            let bkg = (l / py) as u32;
            let fpo = (l % py) as usize;
            
            
            let clw = if (fa.ocv & ARM_) != 0 {
                self.nse(&fa, bkg)?
            } else {
                self.ois(&fa, bkg)
                    .ok_or(())? as u64
            };
            
            let mut ajy = vec![0u8; py as usize];
            self.jlh(clw, &mut ajy)?;
            
            let mut u = fpo;
            while u + 8 <= py as usize {
                let isw = u32::dj([
                    ajy[u], ajy[u+1], ajy[u+2], ajy[u+3]
                ]);
                let jlr = u16::dj([ajy[u+4], ajy[u+5]]) as usize;
                let baf = ajy[u+6] as usize;
                let kd = ajy[u+7];
                
                if jlr == 0 { break; } 
                
                if isw != 0 && baf > 0 && u + 8 + baf <= ajy.len() {
                    let j = core::str::jg(&ajy[u+8..u+8+baf])
                        .unwrap_or("")
                        .to_string();
                    if !j.is_empty() {
                        ch.push((isw, j, kd));
                    }
                }
                
                u += jlr;
                l += jlr as u64;
            }
            
            
            let oqj = ((l / py) + 1) * py;
            if l < oqj && u >= py as usize {
                l = oqj;
            }
        }
        
        Ok(ch)
    }
    
    
    fn hgd(&self, ger: u32, j: &str) -> Result<u32, ()> {
        let ch = self.exg(ger)?;
        for (dd, cxm, xzk) in &ch {
            if cxm == j {
                return Ok(*dd);
            }
        }
        Err(())
    }
    
    
    fn aqj(&self, path: &str) -> Result<u32, ()> {
        let path = path.tl('/');
        if path.is_empty() {
            return Ok(ABY_);
        }
        
        let mut cv = ABY_;
        for ffm in path.adk('/') {
            if ffm.is_empty() || ffm == "." { continue; }
            cv = self.hgd(cv, ffm)?;
        }
        Ok(cv)
    }
    
    
    fn fll(&self, fa: &Sl) -> FileType {
        match fa.fkv & BGW_ {
            PV_ => FileType::Ea,
            AIN_ => FileType::K,
            AIO_ => FileType::Anh,
            XQ_ => FileType::Mv,
            CXE_ => FileType::Bj,
            _ => FileType::Ea,
        }
    }
}

fn kqa(agm: u8) -> FileType {
    match agm {
        BUV_ => FileType::Ea,
        BUU_ => FileType::K,
        BUW_ => FileType::Anh,
        BUT_ => FileType::Mv,
        BUS_ => FileType::Bj,
        _ => FileType::Ea,
    }
}






struct Ccp {
    fs: Arc<Sj>,
    dd: u32,
}

impl Et for Ccp {
    fn read(&self, l: u64, k: &mut [u8]) -> B<usize> {
        let ff = self.fs.ff.lock();
        let fa = ff.ayn(self.dd).jd(|_| VfsError::Av)?;
        ff.hwx(&fa, l, k).jd(|_| VfsError::Av)
    }
    
    fn write(&self, dnv: u64, ihz: &[u8]) -> B<usize> {
        Err(VfsError::Bz) 
    }
    
    fn hm(&self) -> B<Stat> {
        let ff = self.fs.ff.lock();
        let fa = ff.ayn(self.dd).jd(|_| VfsError::Av)?;
        let yy = ff.bac(&fa);
        let agm = ff.fll(&fa);
        
        Ok(Stat {
            dd: self.dd as u64,
            kd: agm,
            aw: yy,
            xk: fa.hnk as u64,
            py: ff.py,
            ev: fa.fkv as u32,
            pi: fa.hno as u32,
            pw: fa.hnm as u32,
            byi: fa.hnj as u64,
            bnp: fa.hnn as u64,
            cpq: fa.hnl as u64,
        })
    }
}


struct Ccl {
    fs: Arc<Sj>,
    dd: u32,
}

impl Ep for Ccl {
    fn cga(&self, j: &str) -> B<I> {
        let ff = self.fs.ff.lock();
        ff.hgd(self.dd, j)
            .map(|dd| dd as u64)
            .jd(|_| VfsError::N)
    }
    
    fn brx(&self) -> B<Vec<Br>> {
        let ff = self.fs.ff.lock();
        let ch = ff.exg(self.dd)
            .jd(|_| VfsError::Av)?;
        
        Ok(ch.dse()
            .hi(|(_, j, _)| j != "." && j != "..")
            .map(|(dd, j, agm)| Br {
                j,
                dd: dd as u64,
                kd: kqa(agm),
            })
            .collect())
    }
    
    fn avp(&self, blu: &str, gxf: FileType) -> B<I> {
        Err(VfsError::Bz)
    }
    
    fn cnm(&self, blu: &str) -> B<()> {
        Err(VfsError::Bz)
    }
    
    fn hm(&self) -> B<Stat> {
        let ff = self.fs.ff.lock();
        let fa = ff.ayn(self.dd).jd(|_| VfsError::Av)?;
        
        Ok(Stat {
            dd: self.dd as u64,
            kd: FileType::K,
            aw: ff.bac(&fa),
            xk: fa.hnk as u64,
            py: ff.py,
            ev: fa.fkv as u32,
            pi: fa.hno as u32,
            pw: fa.hnm as u32,
            byi: fa.hnj as u64,
            bnp: fa.hnn as u64,
            cpq: fa.hnl as u64,
        })
    }
}


impl Cc for Sj {
    fn j(&self) -> &str { "ext4" }
    
    fn cbm(&self) -> I { ABY_ as u64 }
    
    fn era(&self, dd: I) -> B<Arc<dyn Et>> {
        let ff = self.ff.lock();
        let fa = ff.ayn(dd as u32).jd(|_| VfsError::N)?;
        let agm = ff.fll(&fa);
        if agm == FileType::K {
            return Err(VfsError::Tc);
        }
        drop(ff);
        
        
        
        
        Ok(Arc::new(Aru {
            dd: dd as u32,
            de: self.ff.lock().de.clone(),
            py: self.ff.lock().py,
            bac: self.ff.lock().bac,
            cro: self.ff.lock().cro,
            dej: self.ff.lock().dej,
            dhf: self.ff.lock().dhf,
            cfc: self.ff.lock().cfc,
        }))
    }
    
    fn dhl(&self, dd: I) -> B<Arc<dyn Ep>> {
        let ff = self.ff.lock();
        let fa = ff.ayn(dd as u32).jd(|_| VfsError::N)?;
        let agm = ff.fll(&fa);
        if agm != FileType::K {
            return Err(VfsError::Lz);
        }
        drop(ff);
        
        Ok(Arc::new(Art {
            dd: dd as u32,
            de: self.ff.lock().de.clone(),
            py: self.ff.lock().py,
            bac: self.ff.lock().bac,
            cro: self.ff.lock().cro,
            dej: self.ff.lock().dej,
            dhf: self.ff.lock().dhf,
            cfc: self.ff.lock().cfc,
        }))
    }
    
    fn hm(&self, dd: I) -> B<Stat> {
        let ff = self.ff.lock();
        let fa = ff.ayn(dd as u32).jd(|_| VfsError::N)?;
        let yy = ff.bac(&fa);
        let agm = ff.fll(&fa);
        
        Ok(Stat {
            dd,
            kd: agm,
            aw: yy,
            xk: fa.hnk as u64,
            py: ff.py,
            ev: fa.fkv as u32,
            pi: fa.hno as u32,
            pw: fa.hnm as u32,
            byi: fa.hnj as u64,
            bnp: fa.hnn as u64,
            cpq: fa.hnl as u64,
        })
    }
}


struct Aru {
    dd: u32,
    de: Arc<dyn Bj>,
    py: u32,
    bac: u16,
    cro: u32,
    dej: u32,
    dhf: u32,
    cfc: u32,
}

impl Aru {
    fn csh(&self) -> Sk {
        Sk {
            py: self.py,
            bac: self.bac,
            cro: self.cro,
            dej: self.dej,
            dhf: self.dhf,
            cfc: self.cfc,
            iei: 0,
            de: self.de.clone(),
        }
    }
}

impl Et for Aru {
    fn read(&self, l: u64, k: &mut [u8]) -> B<usize> {
        let ff = self.csh();
        let fa = ff.ayn(self.dd).jd(|_| VfsError::Av)?;
        ff.hwx(&fa, l, k).jd(|_| VfsError::Av)
    }
    
    fn write(&self, dnv: u64, ihz: &[u8]) -> B<usize> {
        Err(VfsError::Bz)
    }
    
    fn hm(&self) -> B<Stat> {
        let ff = self.csh();
        let fa = ff.ayn(self.dd).jd(|_| VfsError::Av)?;
        let yy = ff.bac(&fa);
        let agm = ff.fll(&fa);
        Ok(Stat {
            dd: self.dd as u64,
            kd: agm,
            aw: yy,
            xk: fa.hnk as u64,
            py: ff.py,
            ev: fa.fkv as u32,
            pi: fa.hno as u32,
            pw: fa.hnm as u32,
            byi: fa.hnj as u64,
            bnp: fa.hnn as u64,
            cpq: fa.hnl as u64,
        })
    }
}


struct Art {
    dd: u32,
    de: Arc<dyn Bj>,
    py: u32,
    bac: u16,
    cro: u32,
    dej: u32,
    dhf: u32,
    cfc: u32,
}

impl Art {
    fn csh(&self) -> Sk {
        Sk {
            py: self.py,
            bac: self.bac,
            cro: self.cro,
            dej: self.dej,
            dhf: self.dhf,
            cfc: self.cfc,
            iei: 0,
            de: self.de.clone(),
        }
    }
}

impl Ep for Art {
    fn cga(&self, j: &str) -> B<I> {
        let ff = self.csh();
        ff.hgd(self.dd, j)
            .map(|dd| dd as u64)
            .jd(|_| VfsError::N)
    }
    
    fn brx(&self) -> B<Vec<Br>> {
        let ff = self.csh();
        let ch = ff.exg(self.dd)
            .jd(|_| VfsError::Av)?;
        
        Ok(ch.dse()
            .hi(|(_, j, _)| j != "." && j != "..")
            .map(|(dd, j, agm)| Br {
                j,
                dd: dd as u64,
                kd: kqa(agm),
            })
            .collect())
    }
    
    fn avp(&self, blu: &str, gxf: FileType) -> B<I> {
        Err(VfsError::Bz)
    }
    
    fn cnm(&self, blu: &str) -> B<()> {
        Err(VfsError::Bz)
    }
    
    fn hm(&self) -> B<Stat> {
        let ff = self.csh();
        let fa = ff.ayn(self.dd).jd(|_| VfsError::Av)?;
        Ok(Stat {
            dd: self.dd as u64,
            kd: FileType::K,
            aw: ff.bac(&fa),
            xk: fa.hnk as u64,
            py: ff.py,
            ev: fa.fkv as u32,
            pi: fa.hno as u32,
            pw: fa.hnm as u32,
            byi: fa.hnj as u64,
            bnp: fa.hnn as u64,
            cpq: fa.hnl as u64,
        })
    }
}






pub fn beu(de: Arc<dyn Bj>) -> Result<Arc<Sj>, &'static str> {
    
    let mut pfz = [0u8; 256]; 
    let zn = de.zn();
    
    
    let awy = XO_ / zn as u64;
    let mut js = vec![0u8; zn * 4]; 
    for a in 0..4u64 {
        let _ = de.xr(awy + a, &mut js[a as usize * zn..(a as usize + 1) * zn]);
    }
    
    let ftp = (XO_ - awy * zn as u64) as usize;
    if ftp + 256 > js.len() {
        return Err("Superblock read overflow");
    }
    pfz.dg(&js[ftp..ftp + 256]);
    
    let is = unsafe { core::ptr::md(pfz.fq() as *const Ccq) };
    
    
    if is.wcc != ARN_ {
        return Err("Not an ext4 filesystem (bad magic)");
    }
    
    let py = 1024u32 << is.wcb;
    let bac = if is.wcd >= 1 { is.wca } else { 128 };
    
    
    let edt = (is.wby & 0x80) != 0; 
    let cfc = if edt { 64u32 } else { 32 };
    
    let iei = (is.wbx + is.jnh - 1) / is.jnh;
    
    let xsv = core::str::jg(&is.wce)
        .unwrap_or("")
        .bdd('\0');
    
    crate::serial_println!("[ext4] Mounted: \"{}\" block_size={} inode_size={} groups={} 64bit={}",
        xsv, py, bac, iei, edt);
    crate::serial_println!("[ext4] {} inodes, {} blocks per group",
        is.pez, is.jnh);
    
    let fs = Arc::new(Sj {
        ff: Mutex::new(Sk {
            py,
            bac,
            cro: is.pez,
            dej: is.jnh,
            dhf: is.wbz,
            cfc,
            iei,
            de,
        }),
    });
    
    Ok(fs)
}


pub fn probe(de: &dyn Bj) -> bool {
    let zn = de.zn();
    let awy = XO_ / zn as u64;
    
    let mut js = vec![0u8; zn * 4];
    for a in 0..4u64 {
        if de.xr(awy + a, &mut js[a as usize * zn..(a as usize + 1) * zn]).is_err() {
            return false;
        }
    }
    
    let ftp = (XO_ - awy * zn as u64) as usize;
    if ftp + 2 + 0x38 > js.len() { return false; }
    
    let sj = u16::dj([js[ftp + 0x38], js[ftp + 0x39]]);
    sj == ARN_
}


pub fn mq(fs: &Sj, path: &str) -> Result<Vec<u8>, &'static str> {
    let ff = fs.ff.lock();
    let dd = ff.aqj(path).jd(|_| "File not found")?;
    let fa = ff.ayn(dd).jd(|_| "Failed to read inode")?;
    
    let agm = ff.fll(&fa);
    if agm == FileType::K {
        return Err("Is a directory");
    }
    
    let aw = ff.bac(&fa);
    if aw > 64 * 1024 * 1024 {
        return Err("File too large (>64MB)");
    }
    
    let mut k = vec![0u8; aw as usize];
    ff.hwx(&fa, 0, &mut k).jd(|_| "Read error")?;
    Ok(k)
}


pub fn ojo(fs: &Sj, path: &str) -> Result<Vec<(String, FileType, u64)>, &'static str> {
    let ff = fs.ff.lock();
    let dd = ff.aqj(path).jd(|_| "Directory not found")?;
    let fa = ff.ayn(dd).jd(|_| "Failed to read inode")?;
    
    let agm = ff.fll(&fa);
    if agm != FileType::K {
        return Err("Not a directory");
    }
    
    let ch = ff.exg(dd).jd(|_| "Read error")?;
    
    let mut result = Vec::new();
    for (smd, j, syp) in ch {
        if j == "." || j == ".." { continue; }
        
        let kd = kqa(syp);
        let aw = if let Ok(isw) = ff.ayn(smd) {
            ff.bac(&isw)
        } else {
            0
        };
        
        result.push((j, kd, aw));
    }
    
    Ok(result)
}
