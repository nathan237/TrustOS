



















use alloc::string::{String, Gd};
use alloc::vec::Vec;
use alloc::vec;
use alloc::sync::Arc;
use alloc::format;
use spin::Mutex;

use super::{Et, Ep, Cc, FileType, Stat, Br, B, VfsError, I};
use super::fat32::Bj;






const BAN_: u32 = 0x454C4946; 


const BBR_: &[u8; 8] = b"NTFS    ";


const DTZ_: u64 = 0;          
const BAO_: u64 = 5;         


const BKQ_: u32 = 0x10;
const BKN_: u32 = 0x30;
const AKW_: u32 = 0x80;
const BKP_: u32 = 0x90;
const BKO_: u32 = 0xA0;
const DCS_: u32 = 0xB0;
const AKX_: u32 = 0xFFFFFFFF;


const ARZ_: u8 = 0;
const ASA_: u8 = 1;
const ACC_: u8 = 2;
const ASB_: u8 = 3;


const DTY_: u16 = 0x0001;
const CGI_: u16 = 0x0002;


const DQY_: u32 = 0x01;
const CBN_: u32 = 0x02;


const H_: usize = 512;






#[repr(C, packed)]
#[derive(Clone, Copy)]
struct Bnm {
    uak: [u8; 3],          
    clo: [u8; 8],            
    aid: u16,      
    anx: u8,    
    fzp: [u8; 7],        
    zco: u8,       
    jyk: [u8; 2],        
    wge: u16,     
    uwj: u16,             
    tor: u32,        
    yce: u32,            
    ycf: u32,            
    axf: u64,         
    uny: u64,               
    zcr: u64,        
    bhk: i8,        
    ycg: [u8; 3],        
    bbt: i8,       
    ych: [u8; 3],        
    zvu: u64,         
}

impl Bnm {
    fn cld(&self) -> bool {
        self.clo == *BBR_
    }

    fn qt(&self) -> u32 {
        let hbc = unsafe { core::ptr::md(core::ptr::vf!(self.aid)) };
        hbc as u32 * self.anx as u32
    }

    fn uoa(&self) -> u32 {
        if self.bhk > 0 {
            self.bhk as u32 * self.qt()
        } else {
            
            1u32 << (-(self.bhk as i32) as u32)
        }
    }

    fn tst(&self) -> u32 {
        if self.bbt > 0 {
            self.bbt as u32 * self.qt()
        } else {
            1u32 << (-(self.bbt as i32) as u32)
        }
    }

    fn cav(&self) -> u64 {
        let bve = unsafe { core::ptr::md(core::ptr::vf!(self.uny)) };
        bve * self.qt() as u64
    }
}


#[repr(C, packed)]
#[derive(Clone, Copy, Debug)]
struct Chj {
    sj: u32,                 
    gvq: u16,     
    ifx: u16,       
    uhw: u64,        
    zmk: u16,       
    yvy: u16,       
    stx: u16,     
    flags: u16,                 
    fxx: u32,             
    kae: u32,        
    yfo: u64,           
    zdp: u16,          
}


#[repr(C, packed)]
#[derive(Clone, Copy, Debug)]
struct Crm {
    gze: u32,             
    go: u32,                
    gnw: u8,           
    hsj: u8,            
    zdf: u16,           
    flags: u16,                 
    yfe: u16,               
}


#[repr(C, packed)]
#[derive(Clone, Copy, Debug)]
struct Dfw {
    zve: u32,          
    zvf: u16,          
    yxm: u16,          
}


#[repr(C, packed)]
#[derive(Clone, Copy, Debug)]
struct Awd {
    zbt: u64,            
    yxc: u64,           
    koc: u16,      
    yjs: u16,      
    mss: u32,              
    kae: u64,        
    lyc: u64,             
    yxx: u64,      
}


#[repr(C, packed)]
#[derive(Clone, Copy)]
struct Bgy {
    huh: u64,            
    eaa: u64,         
    efn: u64,     
    unz: u64, 
    dya: u64,           
    kae: u64,        
    lyc: u64,             
    flags: u32,                 
    zjq: u32,        
    hsj: u8,            
    oox: u8,              
    
}


#[repr(C, packed)]
#[derive(Clone, Copy)]
struct Cnk {
    eaa: u64,
    efn: u64,
    unz: u64,
    dya: u64,
    hjf: u32,       
    mss: [u8; 4],
}


#[repr(C, packed)]
#[derive(Clone, Copy)]
struct Czq {
    gze: u32,             
    yjl: u32,        
    bbt: u32,      
    yiz: u8,     
    mss: [u8; 3],
}


#[repr(C, packed)]
#[derive(Clone, Copy)]
struct Czp {
    nqj: u32,        
    aay: u32,            
    kae: u32,        
    flags: u32,                 
}


#[repr(C, packed)]
#[derive(Clone, Copy)]
struct Cft {
    uob: u64,         
    smf: u16,          
    roi: u16,        
    flags: u32,                 
}


const CBO_: u32 = 0x58444E49; 

#[repr(C, packed)]
#[derive(Clone, Copy)]
struct Czo {
    sj: u32,                 
    gvq: u16,
    ifx: u16,
    uhw: u64,
    cnr: u64,                   
    
}






#[derive(Clone, Debug)]
struct Pe {
    
    dxj: u64,
    
    go: u64,
    
    bve: u64,
}


fn kom(f: &[u8]) -> Vec<Pe> {
    let mut jng = Vec::new();
    let mut l = 0usize;
    let mut fgi: u64 = 0;
    let mut nih: i64 = 0;

    while l < f.len() {
        let dh = f[l];
        if dh == 0 {
            break; 
        }

        let jdh = (dh & 0x0F) as usize;
        let fpp = ((dh >> 4) & 0x0F) as usize;
        l += 1;

        if jdh == 0 || l + jdh + fpp > f.len() {
            break;
        }

        
        let mut mbf: u64 = 0;
        for a in 0..jdh {
            mbf |= (f[l + a] as u64) << (a * 8);
        }
        l += jdh;

        
        let mut jnf: i64 = 0;
        if fpp > 0 {
            for a in 0..fpp {
                jnf |= (f[l + a] as i64) << (a * 8);
            }
            
            let woa = 1i64 << (fpp * 8 - 1);
            if jnf & woa != 0 {
                jnf |= !((1i64 << (fpp * 8)) - 1);
            }
            l += fpp;

            nih += jnf;
        }

        let bve = if fpp == 0 {
            0 
        } else {
            nih as u64
        };

        jng.push(Pe {
            dxj: fgi,
            go: mbf,
            bve,
        });

        fgi += mbf;
    }

    jng
}






#[derive(Clone)]
struct Xo {
    
    vtg: u64,
    
    flags: u16,
    
    kvq: String,
    
    huh: u64,
    
    yy: u64,
    
    cfr: bool,
    
    eaa: u64,
    efn: u64,
    dya: u64,
    
    hjf: u32,
    
    iqp: Vec<Pe>,
    
    hfe: bool,
    
    fss: Vec<u8>,
    
    hnv: Vec<u8>,
    
    gjw: Vec<Pe>,
}






pub struct Akn {
    ff: Mutex<Ts>,
}

struct Ts {
    de: Arc<dyn Bj>,
    qt: u32,
    bhk: u32,
    bbt: u32,
    cav: u64,
    anx: u8,
    aid: u16,
    
    bvk: Vec<Pe>,
}

impl Ts {
    
    fn day(&self, aok: u64, k: &mut [u8]) -> Result<(), ()> {
        let zn = self.de.zn() as u64;
        let awy = aok / zn;
        let bho = (aok % zn) as usize;

        let xv = bho + k.len();
        let lpn = (xv + zn as usize - 1) / zn as usize;

        let mut ia = k.len();
        let mut avj = 0usize;
        let mut aae = vec![0u8; zn as usize];

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

    
    fn zhk(&self, bve: u64, az: u64, k: &mut [u8]) -> Result<(), ()> {
        let aok = bve * self.qt as u64;
        let nay = az as usize * self.qt as usize;
        if k.len() < nay {
            return Err(());
        }
        self.day(aok, &mut k[..nay])
    }

    
    fn mwb(&self, k: &mut [u8], gqp: usize) -> Result<(), ()> {
        if k.len() < 6 {
            return Err(());
        }
        let gvq = u16::dj([k[4], k[5]]) as usize;
        let ifx = u16::dj([k[6], k[7]]) as usize;

        if ifx < 2 || gvq + ifx * 2 > k.len() {
            return Err(());
        }

        
        let signature = u16::dj([
            k[gvq],
            k[gvq + 1],
        ]);

        
        let zn = self.aid as usize;
        for a in 1..ifx {
            let mda = a * zn;
            if mda > gqp || mda < 2 {
                break;
            }
            let iuw = mda - 2;

            
            let mhs = u16::dj([k[iuw], k[iuw + 1]]);
            if mhs != signature {
                return Err(()); 
            }

            
            let mbu = gvq + a * 2;
            if mbu + 1 < k.len() {
                k[iuw] = k[mbu];
                k[iuw + 1] = k[mbu + 1];
            }
        }

        Ok(())
    }

    
    fn vsb(&self, bwn: u64) -> Result<Vec<u8>, ()> {
        let gqp = self.bhk as usize;
        let mut k = vec![0u8; gqp];

        
        let naz = bwn * gqp as u64;
        let cnr = naz / self.qt as u64;
        let dke = (naz % self.qt as u64) as usize;

        
        let mut kft = gqp;
        let mut kfh = 0;
        let mut fgi = cnr;
        let mut kmz = dke;

        while kft > 0 {
            
            let vw = self.bvk.iter().du(|m| {
                fgi >= m.dxj && fgi < m.dxj + m.go
            });

            match vw {
                Some(vw) => {
                    let xqv = fgi - vw.dxj;
                    let bve = vw.bve + xqv;
                    let aok = bve * self.qt as u64 + kmz as u64;

                    let qlt = self.qt as usize - kmz;
                    let ajp = kft.v(qlt);

                    self.day(aok, &mut k[kfh..kfh + ajp])?;

                    kfh += ajp;
                    kft -= ajp;
                    kmz = 0;
                    fgi += 1;
                }
                None => return Err(()),
            }
        }

        
        self.mwb(&mut k, gqp)?;

        
        let sj = u32::dj([k[0], k[1], k[2], k[3]]);
        if sj != BAN_ {
            return Err(());
        }

        Ok(k)
    }

    
    fn vcw(&self, bwn: u64, k: &[u8]) -> Result<Xo, ()> {
        let dh = unsafe {
            core::ptr::md(k.fq() as *const Chj)
        };

        let flags = dh.flags;
        let cfr = (flags & CGI_) != 0;
        let kwi = dh.stx as usize;
        let fxx = dh.fxx as usize;

        let mut kvq = String::new();
        let mut mys: Option<u8> = None;
        let mut huh: u64 = 0;
        let mut yy: u64 = 0;
        let mut eaa: u64 = 0;
        let mut efn: u64 = 0;
        let mut dya: u64 = 0;
        let mut hjf: u32 = 0;
        let mut iqp = Vec::new();
        let mut hfe = false;
        let mut fss = Vec::new();
        let mut hnv = Vec::new();
        let mut gjw = Vec::new();

        let mut l = kwi;
        let ul = fxx.v(k.len());

        while l + 4 <= ul {
            let gze = u32::dj([
                k[l], k[l + 1], k[l + 2], k[l + 3],
            ]);

            if gze == AKX_ || gze == 0 {
                break;
            }

            if l + 8 > ul {
                break;
            }

            let fcu = u32::dj([
                k[l + 4], k[l + 5], k[l + 6], k[l + 7],
            ]) as usize;

            if fcu < 16 || fcu > ul - l {
                break;
            }

            let gnw = k[l + 8];
            let hsj = k[l + 9] as usize;

            
            let tzh = hsj == 0;

            match gze {
                BKQ_ if gnw == 0 => {
                    
                    if l + 24 <= ul {
                        let dxh = u32::dj([
                            k[l + 16], k[l + 17],
                            k[l + 18], k[l + 19],
                        ]) as usize;
                        let fya = u16::dj([
                            k[l + 20], k[l + 21],
                        ]) as usize;
                        let bjt = l + fya;
                        if dxh >= 48 && bjt + 48 <= k.len() {
                            let si = unsafe {
                                core::ptr::md(
                                    k[bjt..].fq() as *const Cnk
                                )
                            };
                            eaa = si.eaa;
                            efn = si.efn;
                            dya = si.dya;
                            hjf = si.hjf;
                        }
                    }
                }

                BKN_ if gnw == 0 => {
                    
                    if l + 24 <= ul {
                        let dxh = u32::dj([
                            k[l + 16], k[l + 17],
                            k[l + 18], k[l + 19],
                        ]) as usize;
                        let fya = u16::dj([
                            k[l + 20], k[l + 21],
                        ]) as usize;
                        let bjt = l + fya;
                        if dxh >= 66 && bjt + 66 <= k.len() {
                            let eqo = unsafe {
                                core::ptr::md(
                                    k[bjt..].fq() as *const Bgy
                                )
                            };
                            let csw = eqo.oox;
                            let hsh = eqo.hsj as usize;
                            let akj = bjt + 66;

                            
                            
                            let abv = match csw {
                                ASA_ => 4,
                                ASB_ => 3,
                                ARZ_ => 2,
                                ACC_ => 1,
                                _ => 0,
                            };
                            let kna = mys.map(|bo| match bo {
                                ASA_ => 4,
                                ASB_ => 3,
                                ARZ_ => 2,
                                ACC_ => 1,
                                _ => 0,
                            }).unwrap_or(0);

                            if abv > kna {
                                if akj + hsh * 2 <= k.len() {
                                    kvq = nkc(
                                        &k[akj..akj + hsh * 2]
                                    );
                                    mys = Some(csw);
                                    huh = eqo.huh & 0x0000FFFFFFFFFFFF;

                                    
                                    if yy == 0 {
                                        yy = unsafe {
                                            core::ptr::md(
                                                core::ptr::vf!(eqo.lyc)
                                            )
                                        };
                                    }
                                }
                            }
                        }
                    }
                }

                AKW_ if tzh => {
                    if gnw == 0 {
                        
                        hfe = true;
                        if l + 24 <= ul {
                            let dxh = u32::dj([
                                k[l + 16], k[l + 17],
                                k[l + 18], k[l + 19],
                            ]) as usize;
                            let fya = u16::dj([
                                k[l + 20], k[l + 21],
                            ]) as usize;
                            let bjt = l + fya;
                            if bjt + dxh <= k.len() {
                                fss = k[bjt..bjt + dxh].ip();
                                yy = dxh as u64;
                            }
                        }
                    } else {
                        
                        hfe = false;
                        if l + 64 <= ul {
                            let jhl = unsafe {
                                core::ptr::md(
                                    k[l + 16..].fq() as *const Awd
                                )
                            };
                            yy = jhl.lyc;
                            let mbh = unsafe {
                                core::ptr::md(
                                    core::ptr::vf!(jhl.koc)
                                )
                            } as usize;
                            let exx = l + mbh;
                            if exx < l + fcu {
                                iqp = kom(
                                    &k[exx..l + fcu]
                                );
                            }
                        }
                    }
                }

                BKP_ if gnw == 0 => {
                    
                    if l + 24 <= ul {
                        let dxh = u32::dj([
                            k[l + 16], k[l + 17],
                            k[l + 18], k[l + 19],
                        ]) as usize;
                        let fya = u16::dj([
                            k[l + 20], k[l + 21],
                        ]) as usize;
                        let bjt = l + fya;
                        if bjt + dxh <= k.len() {
                            hnv = k[bjt..bjt + dxh].ip();
                        }
                    }
                }

                BKO_ if gnw != 0 => {
                    
                    if l + 64 <= ul {
                        let jhl = unsafe {
                            core::ptr::md(
                                k[l + 16..].fq() as *const Awd
                            )
                        };
                        let mbh = unsafe {
                            core::ptr::md(
                                core::ptr::vf!(jhl.koc)
                            )
                        } as usize;
                        let exx = l + mbh;
                        if exx < l + fcu {
                            gjw = kom(
                                &k[exx..l + fcu]
                            );
                        }
                    }
                }

                _ => {}
            }

            l += fcu;
        }

        Ok(Xo {
            vtg: bwn,
            flags,
            kvq,
            huh,
            yy,
            cfr,
            eaa,
            efn,
            dya,
            hjf,
            iqp,
            hfe,
            fss,
            hnv,
            gjw,
        })
    }

    
    fn ehc(&self, bwn: u64) -> Result<Xo, ()> {
        let js = self.vsb(bwn)?;
        self.vcw(bwn, &js)
    }

    
    fn hwx(
        &self,
        record: &Xo,
        azv: u64,
        k: &mut [u8],
    ) -> Result<usize, ()> {
        if azv >= record.yy {
            return Ok(0);
        }

        let cgy = ((record.yy - azv) as usize).v(k.len());
        if cgy == 0 {
            return Ok(0);
        }

        if record.hfe {
            
            let ay = azv as usize;
            let ci = ay + cgy;
            if ci <= record.fss.len() {
                k[..cgy].dg(&record.fss[ay..ci]);
            } else {
                let apk = record.fss.len().ao(ay);
                k[..apk].dg(&record.fss[ay..ay + apk]);
            }
            return Ok(cgy);
        }

        
        let qt = self.qt as u64;
        let mut ia = cgy;
        let mut avj = 0usize;
        let mut l = azv;

        while ia > 0 {
            let cnr = l / qt;
            let dke = (l % qt) as usize;

            
            let vw = record.iqp.iter().du(|m| {
                cnr >= m.dxj && cnr < m.dxj + m.go
            });

            match vw {
                Some(vw) if vw.bve > 0 => {
                    let mox = cnr - vw.dxj;
                    let bve = vw.bve + mox;
                    let aok = bve * qt + dke as u64;

                    let bfz = qt as usize - dke;
                    let ajp = ia.v(bfz);

                    self.day(aok, &mut k[avj..avj + ajp])?;

                    avj += ajp;
                    l += ajp as u64;
                    ia -= ajp;
                }
                Some(_) => {
                    
                    let bfz = qt as usize - dke;
                    let jti = ia.v(bfz);
                    for o in &mut k[avj..avj + jti] {
                        *o = 0;
                    }
                    avj += jti;
                    l += jti as u64;
                    ia -= jti;
                }
                None => {
                    
                    for o in &mut k[avj..avj + ia] {
                        *o = 0;
                    }
                    ia = 0;
                }
            }
        }

        Ok(cgy)
    }

    
    fn vrs(&self, jng: &[Pe], cnr: u64, k: &mut [u8]) -> Result<(), ()> {
        let vw = jng.iter().du(|m| {
            cnr >= m.dxj && cnr < m.dxj + m.go
        });

        match vw {
            Some(vw) if vw.bve > 0 => {
                let mox = cnr - vw.dxj;
                let bve = vw.bve + mox;
                let kie = (k.len() + self.qt as usize - 1)
                    / self.qt as usize;
                
                for a in 0..kie {
                    let qux = (bve + a as u64) * self.qt as u64;
                    let nal = a * self.qt as usize;
                    let qsq = (nal + self.qt as usize).v(k.len());
                    self.day(qux, &mut k[nal..qsq])?;
                }
                Ok(())
            }
            _ => Err(()),
        }
    }

    
    fn exg(&self, record: &Xo) -> Result<Vec<(u64, String, bool)>, ()> {
        let mut ch = Vec::new();

        
        if record.hnv.len() >= 32 {
            let dim = &record.hnv;

            
            let djv = 16; 
            if djv + 16 <= dim.len() {
                let nqj = u32::dj([
                    dim[djv], dim[djv + 1],
                    dim[djv + 2], dim[djv + 3],
                ]) as usize;
                let aay = u32::dj([
                    dim[djv + 4], dim[djv + 5],
                    dim[djv + 6], dim[djv + 7],
                ]) as usize;

                let ay = djv + nqj;
                let ci = (djv + aay).v(dim.len());

                self.oui(&dim[ay..ci], &mut ch);
            }
        }

        
        if !record.gjw.is_empty() {
            let bbt = self.bbt as usize;
            let rby = (bbt + self.qt as usize - 1)
                / self.qt as usize;

            
            let xku: u64 = record.gjw.iter()
                .map(|m| m.go)
                .sum();

            let mut cnr: u64 = 0;
            while cnr < xku {
                let mut ajy = vec![0u8; bbt];
                if self.vrs(&record.gjw, cnr, &mut ajy).is_ok() {
                    
                    let _ = self.mwb(&mut ajy, bbt);

                    let sj = u32::dj([
                        ajy[0], ajy[1], ajy[2], ajy[3],
                    ]);
                    if sj == CBO_ {
                        
                        let dju = 0x18;
                        if dju + 16 <= ajy.len() {
                            let smn = u32::dj([
                                ajy[dju], ajy[dju + 1],
                                ajy[dju + 2], ajy[dju + 3],
                            ]) as usize;
                            let wi = u32::dj([
                                ajy[dju + 4], ajy[dju + 5],
                                ajy[dju + 6], ajy[dju + 7],
                            ]) as usize;

                            let ay = dju + smn;
                            let ci = (dju + wi).v(ajy.len());
                            if ay < ci {
                                self.oui(&ajy[ay..ci], &mut ch);
                            }
                        }
                    }
                }
                cnr += rby as u64;
            }
        }

        Ok(ch)
    }

    
    fn oui(
        &self,
        f: &[u8],
        ch: &mut Vec<(u64, String, bool)>,
    ) {
        let mut u = 0;
        while u + 16 <= f.len() {
            let bzn = unsafe {
                core::ptr::md(f[u..].fq() as *const Cft)
            };

            let bue = bzn.smf as usize;
            let byy = bzn.roi as usize;
            let flags = bzn.flags;

            if bue < 16 || bue > f.len() - u {
                break;
            }

            if (flags & CBN_) != 0 {
                break; 
            }

            if byy >= 66 {
                
                let dzt = u + 16; 
                if dzt + byy <= f.len() {
                    let ive = &f[dzt..dzt + byy];
                    if ive.len() >= 66 {
                        let eqo = unsafe {
                            core::ptr::md(
                                ive.fq() as *const Bgy
                            )
                        };

                        let csw = eqo.oox;
                        
                        if csw != ACC_ {
                            let hsh = eqo.hsj as usize;
                            let akj = 66;
                            if akj + hsh * 2 <= ive.len() {
                                let j = nkc(
                                    &ive[akj..akj + hsh * 2]
                                );

                                let hrl = bzn.uob & 0x0000FFFFFFFFFFFF;
                                let sun = unsafe {
                                    core::ptr::md(
                                        core::ptr::vf!(eqo.flags)
                                    )
                                };
                                let ta = (sun & 0x10000000) != 0;

                                
                                if !j.cj('$') && !j.is_empty() {
                                    ch.push((hrl, j, ta));
                                }
                            }
                        }
                    }
                }
            }

            u += bue;
        }
    }

    
    fn hgd(&self, rxr: u64, j: &str) -> Result<u64, ()> {
        let record = self.ehc(rxr)?;
        let ch = self.exg(&record)?;
        for (hrl, cxm, yae) in &ch {
            if cxm.dha(j) {
                return Ok(*hrl);
            }
        }
        Err(())
    }

    
    fn par(&self, record: &Xo) -> FileType {
        if record.cfr {
            FileType::K
        } else {
            FileType::Ea
        }
    }
}






fn nkc(f: &[u8]) -> String {
    let mut bw = Vec::fc(f.len() / 2);
    for jj in f.ras(2) {
        let rll = u16::dj([jj[0], jj[1]]);
        bw.push(rll);
    }

    
    let mut result = String::fc(bw.len());
    let mut a = 0;
    while a < bw.len() {
        let r = bw[a];
        if r >= 0xD800 && r <= 0xDBFF && a + 1 < bw.len() {
            
            let gd = r;
            let hh = bw[a + 1];
            if hh >= 0xDC00 && hh <= 0xDFFF {
                let bza = 0x10000 + ((gd as u32 - 0xD800) << 10) + (hh as u32 - 0xDC00);
                if let Some(bm) = char::zi(bza) {
                    result.push(bm);
                }
                a += 2;
                continue;
            }
        }
        if let Some(bm) = char::zi(r as u32) {
            result.push(bm);
        }
        a += 1;
    }

    result
}






fn efv(ori: u64) -> u64 {
    if ori == 0 {
        return 0;
    }
    
    
    const CHZ_: u64 = 11644473600;
    let dvm = ori / 10_000_000; 
    dvm.ao(CHZ_)
}






struct Awh {
    bwn: u64,
    de: Arc<dyn Bj>,
    qt: u32,
    bhk: u32,
    bbt: u32,
    cav: u64,
    anx: u8,
    aid: u16,
    bvk: Vec<Pe>,
}

impl Awh {
    fn csh(&self) -> Ts {
        Ts {
            de: self.de.clone(),
            qt: self.qt,
            bhk: self.bhk,
            bbt: self.bbt,
            cav: self.cav,
            anx: self.anx,
            aid: self.aid,
            bvk: self.bvk.clone(),
        }
    }
}

impl Et for Awh {
    fn read(&self, l: u64, k: &mut [u8]) -> B<usize> {
        let ff = self.csh();
        let record = ff.ehc(self.bwn)
            .jd(|_| VfsError::Av)?;
        ff.hwx(&record, l, k)
            .jd(|_| VfsError::Av)
    }

    fn write(&self, dnv: u64, ihz: &[u8]) -> B<usize> {
        Err(VfsError::Bz)
    }

    fn hm(&self) -> B<Stat> {
        let ff = self.csh();
        let record = ff.ehc(self.bwn)
            .jd(|_| VfsError::Av)?;
        Ok(Stat {
            dd: self.bwn,
            kd: ff.par(&record),
            aw: record.yy,
            xk: (record.yy + 511) / 512,
            py: ff.qt,
            ev: 0o444, 
            pi: 0,
            pw: 0,
            byi: efv(record.dya),
            bnp: efv(record.efn),
            cpq: efv(record.eaa),
        })
    }
}


struct Awg {
    bwn: u64,
    de: Arc<dyn Bj>,
    qt: u32,
    bhk: u32,
    bbt: u32,
    cav: u64,
    anx: u8,
    aid: u16,
    bvk: Vec<Pe>,
}

impl Awg {
    fn csh(&self) -> Ts {
        Ts {
            de: self.de.clone(),
            qt: self.qt,
            bhk: self.bhk,
            bbt: self.bbt,
            cav: self.cav,
            anx: self.anx,
            aid: self.aid,
            bvk: self.bvk.clone(),
        }
    }
}

impl Ep for Awg {
    fn cga(&self, j: &str) -> B<I> {
        let ff = self.csh();
        ff.hgd(self.bwn, j)
            .jd(|_| VfsError::N)
    }

    fn brx(&self) -> B<Vec<Br>> {
        let ff = self.csh();
        let record = ff.ehc(self.bwn)
            .jd(|_| VfsError::Av)?;
        let ch = ff.exg(&record)
            .jd(|_| VfsError::Av)?;

        Ok(ch.dse()
            .map(|(hrl, j, ta)| Br {
                j,
                dd: hrl,
                kd: if ta { FileType::K } else { FileType::Ea },
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
        let record = ff.ehc(self.bwn)
            .jd(|_| VfsError::Av)?;
        Ok(Stat {
            dd: self.bwn,
            kd: FileType::K,
            aw: record.yy,
            xk: 0,
            py: ff.qt,
            ev: 0o555, 
            pi: 0,
            pw: 0,
            byi: efv(record.dya),
            bnp: efv(record.efn),
            cpq: efv(record.eaa),
        })
    }
}


impl Cc for Akn {
    fn j(&self) -> &str { "ntfs" }

    fn cbm(&self) -> I { BAO_ }

    fn era(&self, dd: I) -> B<Arc<dyn Et>> {
        let ff = self.ff.lock();
        let record = ff.ehc(dd).jd(|_| VfsError::N)?;
        if record.cfr {
            return Err(VfsError::Tc);
        }
        Ok(Arc::new(Awh {
            bwn: dd,
            de: ff.de.clone(),
            qt: ff.qt,
            bhk: ff.bhk,
            bbt: ff.bbt,
            cav: ff.cav,
            anx: ff.anx,
            aid: ff.aid,
            bvk: ff.bvk.clone(),
        }))
    }

    fn dhl(&self, dd: I) -> B<Arc<dyn Ep>> {
        let ff = self.ff.lock();
        let record = ff.ehc(dd).jd(|_| VfsError::N)?;
        if !record.cfr {
            return Err(VfsError::Lz);
        }
        Ok(Arc::new(Awg {
            bwn: dd,
            de: ff.de.clone(),
            qt: ff.qt,
            bhk: ff.bhk,
            bbt: ff.bbt,
            cav: ff.cav,
            anx: ff.anx,
            aid: ff.aid,
            bvk: ff.bvk.clone(),
        }))
    }

    fn hm(&self, dd: I) -> B<Stat> {
        let ff = self.ff.lock();
        let record = ff.ehc(dd).jd(|_| VfsError::N)?;
        let agm = ff.par(&record);
        Ok(Stat {
            dd,
            kd: agm,
            aw: record.yy,
            xk: (record.yy + 511) / 512,
            py: ff.qt,
            ev: if record.cfr { 0o555 } else { 0o444 },
            pi: 0,
            pw: 0,
            byi: efv(record.dya),
            bnp: efv(record.efn),
            cpq: efv(record.eaa),
        })
    }
}






pub fn beu(de: Arc<dyn Bj>) -> Result<Arc<Akn>, &'static str> {
    
    let mut cvz = [0u8; H_];
    de.xr(0, &mut cvz).jd(|_| "Failed to read NTFS boot sector")?;

    
    let agh = unsafe { core::ptr::md(cvz.fq() as *const Bnm) };
    if !agh.cld() {
        return Err("Not an NTFS filesystem (bad OEM ID)");
    }

    
    if cvz[510] != 0x55 || cvz[511] != 0xAA {
        return Err("Not an NTFS filesystem (bad boot signature)");
    }

    let qt = agh.qt();
    let bhk = agh.uoa();
    let bbt = agh.tst();
    let cav = agh.cav();
    let aid = unsafe {
        core::ptr::md(core::ptr::vf!(agh.aid))
    };
    let anx = agh.anx;

    crate::serial_println!("[NTFS] Detected: cluster_size={} mft_record={}B index_block={}B",
        qt, bhk, bbt);
    crate::serial_println!("[NTFS] MFT at byte offset 0x{:X}", cav);

    
    
    let mut afs = vec![0u8; bhk as usize];
    let zn = de.zn() as u64;
    let uoc = cav / zn;
    let dbu = (bhk as u64 + zn - 1) / zn;

    let mut ozk = vec![0u8; (dbu * zn) as usize];
    for a in 0..dbu {
        de.xr(uoc + a, 
            &mut ozk[(a as usize * zn as usize)..((a + 1) as usize * zn as usize)])
            .jd(|_| "Failed to read MFT record 0")?;
    }
    afs.dg(&ozk[..bhk as usize]);

    
    {
        if afs.len() < 8 {
            return Err("MFT record too small");
        }
        let jux = u16::dj([afs[4], afs[5]]) as usize;
        let moi = u16::dj([afs[6], afs[7]]) as usize;
        if moi >= 2 && jux + moi * 2 <= afs.len() {
            let sig = u16::dj([afs[jux], afs[jux + 1]]);
            let wgd = aid as usize;
            for a in 1..moi {
                let mcz = a * wgd;
                if mcz <= afs.len() && mcz >= 2 {
                    let u = mcz - 2;
                    let mhs = u16::dj([afs[u], afs[u + 1]]);
                    if mhs == sig {
                        let mbt = jux + a * 2;
                        if mbt + 1 < afs.len() {
                            afs[u] = afs[mbt];
                            afs[u + 1] = afs[mbt + 1];
                        }
                    }
                }
            }
        }
    }

    
    let sj = u32::dj([afs[0], afs[1], afs[2], afs[3]]);
    if sj != BAN_ {
        return Err("MFT record 0 has bad magic");
    }

    
    let kwi = u16::dj([afs[20], afs[21]]) as usize;
    let fxx = u32::dj([afs[24], afs[25], afs[26], afs[27]]) as usize;
    let mut bvk = Vec::new();

    let mut dz = kwi;
    let ul = fxx.v(afs.len());
    while dz + 8 <= ul {
        let gzf = u32::dj([
            afs[dz], afs[dz + 1], afs[dz + 2], afs[dz + 3],
        ]);
        let gyd = u32::dj([
            afs[dz + 4], afs[dz + 5], afs[dz + 6], afs[dz + 7],
        ]) as usize;

        if gzf == AKX_ || gzf == 0 || gyd < 16 || gyd > ul - dz {
            break;
        }

        if gzf == AKW_ && dz + 9 < ul && afs[dz + 8] == 1 {
            
            let baf = afs[dz + 9] as usize;
            if baf == 0 && dz + 64 <= ul {
                let nr = unsafe {
                    core::ptr::md(
                        afs[dz + 16..].fq() as *const Awd
                    )
                };
                let wbo = unsafe {
                    core::ptr::md(core::ptr::vf!(nr.koc))
                } as usize;
                let exx = dz + wbo;
                if exx < dz + gyd {
                    bvk = kom(&afs[exx..dz + gyd]);
                }
            }
        }

        dz += gyd;
    }

    if bvk.is_empty() {
        return Err("Failed to parse $MFT data runs");
    }

    let xkk: u64 = bvk.iter().map(|m| m.go).sum();
    crate::serial_println!("[NTFS] $MFT: {} data runs, {} clusters total",
        bvk.len(), xkk);

    let fs = Arc::new(Akn {
        ff: Mutex::new(Ts {
            de,
            qt,
            bhk,
            bbt,
            cav,
            anx,
            aid,
            bvk,
        }),
    });

    
    {
        let ff = fs.ff.lock();
        match ff.ehc(BAO_) {
            Ok(exv) => {
                if !exv.cfr {
                    return Err("MFT record 5 is not a directory");
                }
                crate::serial_println!("[NTFS] Root directory OK, reading entries...");
                match ff.exg(&exv) {
                    Ok(ch) => {
                        crate::serial_println!("[NTFS] Root has {} entries", ch.len());
                    }
                    Err(_) => {
                        crate::serial_println!("[NTFS] Warning: could not read root dir entries");
                    }
                }
            }
            Err(_) => return Err("Failed to read root directory"),
        }
    }

    crate::serial_println!("[NTFS] Filesystem mounted successfully (read-only)");
    Ok(fs)
}


pub fn probe(de: &dyn Bj) -> bool {
    let mut cvz = [0u8; H_];
    if de.xr(0, &mut cvz).is_err() {
        return false;
    }
    
    &cvz[3..11] == BBR_
}


pub fn xmr() -> Option<Arc<Akn>> {
    use crate::drivers::partition::{hul, PartitionType};
    use crate::drivers::ahci;
    use super::fat32::AhciBlockReader;

    let ik = ahci::bhh();
    crate::serial_println!("[NTFS] Scanning {} AHCI devices for NTFS partitions", ik.len());

    for de in ik {
        let port = de.kg;
        let axf = de.agw;

        let dld = |jk: u64, k: &mut [u8]| -> Result<(), &'static str> {
            ahci::ain(port, jk, 1, k).map(|_| ())
        };

        if let Ok(gg) = hul(dld, axf) {
            for partition in &gg.aqd {
                match partition.duf {
                    PartitionType::Awf | PartitionType::Akg => {
                        crate::serial_println!("[NTFS] Found candidate partition at LBA {} ({})",
                            partition.aag, partition.ple());

                        let cha = Arc::new(AhciBlockReader::new(
                            port as usize,
                            partition.aag,
                        ));

                        
                        if probe(&*cha) {
                            match beu(cha) {
                                Ok(fs) => {
                                    crate::serial_println!("[NTFS] Mounted partition from port {} at LBA {}",
                                        port, partition.aag);
                                    return Some(fs);
                                }
                                Err(aa) => {
                                    crate::serial_println!("[NTFS] Mount failed: {}", aa);
                                }
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
