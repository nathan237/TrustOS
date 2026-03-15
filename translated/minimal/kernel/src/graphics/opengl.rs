

























use alloc::vec::Vec;
use spin::Mutex;
use micromath::Wo;

use super::math3d::{Vec3, Vec4, Mat4, kor};
use super::render2d::Color2D;
use super::texture;
use crate::framebuffer;






pub const BXY_: u32 = 0x0000;
pub const TO_: u32 = 0x0001;
pub const BXU_: u32 = 0x0003;
pub const NE_: u32 = 0x0002;
pub const ADA_: u32 = 0x0004;
pub const DOA_: u32 = 0x0005;
pub const ATO_: u32 = 0x0006;
pub const KG_: u32 = 0x0007;


pub const ACW_: u32 = 0x4000;
pub const ACX_: u32 = 0x0100;


pub const ACY_: u32 = 0x1700;
pub const NF_: u32 = 0x1701;


pub const TL_: u32 = 0x0B71;
pub const ATG_: u32 = 0x0B50;
pub const TN_: u32 = 0x4000;
pub const DNT_: u32 = 0x4001;
pub const ATE_: u32 = 0x0B44;
pub const ACV_: u32 = 0x0BE2;


pub const DNO_: u32 = 0x1200;
pub const DNQ_: u32 = 0x1201;
pub const DNY_: u32 = 0x1203;


pub const DNR_: u32 = 0x1D00;
pub const TP_: u32 = 0x1D01;


pub const BXS_: u32 = 0x0404;
pub const DNP_: u32 = 0x0405;
pub const BXT_: u32 = 0x0408;
pub const ATH_: u32 = 0x1B01;
pub const BXR_: u32 = 0x1B02;


pub const ATK_: u32 = 0;
pub const ATF_: u32 = 0x0500;
pub const DNS_: u32 = 0x0501;
pub const TM_: u32 = 0x0502;






#[derive(Clone, Copy, Default)]
struct Auc {
    qf: Vec3,
    s: Color2D,
    adg: Vec3,
    gui: (f32, f32),
}


struct GlState {
    
    jvp: i32,
    fyh: i32,
    faz: u32,
    fyg: u32,

    
    fon: Vec<Mat4>,
    frk: Vec<Mat4>,
    hex: u32,

    
    dpk: Color2D,
    iqc: Vec3,
    iqe: (f32, f32),
    hcv: Color2D,

    
    lvl: u32,
    lm: Vec<Auc>,
    fli: bool,

    
    iqy: bool,
    jdl: bool,
    kmm: bool,
    kdr: bool,
    lip: [bool; 8],

    
    ueo: [Vec4; 8],
    uel: [Color2D; 8],
    uem: [Color2D; 8],

    
    bty: Vec<f32>,

    
    fum: u32,

    
    gpn: u32,

    
    eek: u32,

    
    jr: bool,
}

impl GlState {
    const fn new() -> Self {
        Self {
            jvp: 0,
            fyh: 0,
            faz: 800,
            fyg: 600,
            fon: Vec::new(),
            frk: Vec::new(),
            hex: ACY_,
            dpk: Color2D::Zm,
            iqc: Vec3::Cqu,
            iqe: (0.0, 0.0),
            hcv: Color2D::Ox,
            lvl: ADA_,
            lm: Vec::new(),
            fli: false,
            iqy: false,
            jdl: false,
            kmm: false,
            kdr: false,
            lip: [false; 8],
            ueo: [Vec4::new(0.0, 0.0, 1.0, 0.0); 8],
            uel: [Color2D::Ox; 8],
            uem: [Color2D::Zm; 8],
            bty: Vec::new(),
            fum: TP_,
            gpn: BXR_,
            eek: ATK_,
            jr: false,
        }
    }

    fn bqc(&mut self) -> &mut Mat4 {
        let jo = match self.hex {
            NF_ => &mut self.frk,
            _ => &mut self.fon,
        };
        if jo.is_empty() {
            jo.push(Mat4::Sx);
        }
        jo.dsq().unwrap()
    }

    fn ted(&self) -> Mat4 {
        let euz = self.fon.qv().abn().unwrap_or(Mat4::Sx);
        let aci = self.frk.qv().abn().unwrap_or(Mat4::Sx);
        aci.mul(&euz)
    }
}

static AP_: Mutex<GlState> = Mutex::new(GlState::new());






pub fn nyy(z: u32, ac: u32) {
    let mut g = AP_.lock();
    g.faz = z;
    g.fyg = ac;
    
    let aw = (z as usize) * (ac as usize);
    let mut eo = alloc::vec::Vec::new();
    if eo.jug(aw).is_ok() {
        eo.cmg(aw, 1.0f32);
        g.bty = eo;
    } else {
        crate::serial_println!("[GL] WARNING: Failed to allocate depth buffer {} KB — OOM",
            aw * 4 / 1024);
    }
    g.fon.push(Mat4::Sx);
    g.frk.push(Mat4::Sx);
    g.jr = true;
}






pub fn tfw(b: i32, c: i32, z: u32, ac: u32) {
    let mut g = AP_.lock();
    g.jvp = b;
    g.fyh = c;
    g.faz = z;
    g.fyg = ac;
    
    let aw = (z as usize) * (ac as usize);
    let mut eo = alloc::vec::Vec::new();
    if eo.jug(aw).is_ok() {
        eo.cmg(aw, 1.0f32);
        g.bty = eo;
    }
}


pub fn ixa(ev: u32) {
    let mut g = AP_.lock();
    g.hex = ev;
}


pub fn hlp() {
    let mut g = AP_.lock();
    *g.bqc() = Mat4::Sx;
}


pub fn nza() {
    let mut g = AP_.lock();
    let cv = *g.bqc();
    match g.hex {
        NF_ => g.frk.push(cv),
        _ => g.fon.push(cv),
    }
}


pub fn nyz() {
    let mut g = AP_.lock();
    match g.hex {
        NF_ => { 
            if g.frk.len() > 1 {
                g.frk.pop();
            }
        },
        _ => {
            if g.fon.len() > 1 {
                g.fon.pop();
            }
        },
    }
}


pub fn tfv(b: f32, c: f32, av: f32) {
    let mut g = AP_.lock();
    let mmx = Mat4::mmx(b, c, av);
    let cv = *g.bqc();
    *g.bqc() = cv.mul(&mmx);
}


pub fn ixb(hg: f32, b: f32, c: f32, av: f32) {
    let mut g = AP_.lock();
    let gao = Vec3::new(b, c, av).all();
    let bak = kor(hg);
    let chh = Mat4::chh(gao, bak);
    let cv = *g.bqc();
    *g.bqc() = cv.mul(&chh);
}


pub fn yuu(b: f32, c: f32, av: f32) {
    let mut g = AP_.lock();
    let bv = Mat4::bv(b, c, av);
    let cv = *g.bqc();
    *g.bqc() = cv.mul(&bv);
}


pub fn yus(ef: &[f32; 16]) {
    let mut g = AP_.lock();
    let lkl = Mat4::sxq(*ef);
    let cv = *g.bqc();
    *g.bqc() = cv.mul(&lkl);
}


pub fn tft(fd: f32, hw: f32, abm: f32, qc: f32, bhl: f32, adt: f32) {
    let mut g = AP_.lock();
    let uzj = Mat4::uzk(fd, hw, abm, qc, bhl, adt);
    let cv = *g.bqc();
    *g.bqc() = cv.mul(&uzj);
}


pub fn yup(fd: f32, hw: f32, abm: f32, qc: f32, bhl: f32, adt: f32) {
    let mut g = AP_.lock();
    let kxg = Mat4::kxg(fd, hw, abm, qc, bhl, adt);
    let cv = *g.bqc();
    *g.bqc() = cv.mul(&kxg);
}






pub fn tgf(swl: f32, dyk: f32, bhl: f32, adt: f32) {
    let mut g = AP_.lock();
    let aqf = Mat4::aqf(kor(swl), dyk, bhl, adt);
    let cv = *g.bqc();
    *g.bqc() = cv.mul(&aqf);
}


pub fn nzf(
    fif: f32, ckh: f32, dqr: f32,
    yv: f32, uq: f32, qxs: f32,
    xom: f32, xon: f32, xoo: f32
) {
    let mut g = AP_.lock();
    let ljv = Mat4::ljv(
        Vec3::new(fif, ckh, dqr),
        Vec3::new(yv, uq, qxs),
        Vec3::new(xom, xon, xoo),
    );
    let cv = *g.bqc();
    *g.bqc() = cv.mul(&ljv);
}






pub fn nyx(m: f32, at: f32, o: f32, xxq: f32) {
    let mut g = AP_.lock();
    g.hcv = Color2D::xt(
        (m * 255.0) as u8,
        (at * 255.0) as u8,
        (o * 255.0) as u8,
    );
}


pub fn nyw(hs: u32) {
    let mut g = AP_.lock();
    
    if (hs & ACW_) != 0 {
        let s = g.hcv.lv();
        let d = g.faz;
        let i = g.fyg;
        framebuffer::ah(
            g.jvp as u32, 
            g.fyh as u32, 
            d, i, s
        );
    }
    
    if (hs & ACX_) != 0 {
        g.bty.vi(1.0);
    }
}


pub fn kzd(mh: u32) {
    let mut g = AP_.lock();
    match mh {
        TL_ => g.iqy = true,
        ATG_ => g.jdl = true,
        ATE_ => g.kmm = true,
        ACV_ => g.kdr = true,
        TN_..=0x4007 => {
            let w = (mh - TN_) as usize;
            if w < 8 {
                g.lip[w] = true;
            }
        }
        _ => g.eek = ATF_,
    }
}


pub fn yuo(mh: u32) {
    let mut g = AP_.lock();
    match mh {
        TL_ => g.iqy = false,
        ATG_ => g.jdl = false,
        ATE_ => g.kmm = false,
        ACV_ => g.kdr = false,
        TN_..=0x4007 => {
            let w = (mh - TN_) as usize;
            if w < 8 {
                g.lip[w] = false;
            }
        }
        _ => g.eek = ATF_,
    }
}


pub fn yuv(ev: u32) {
    let mut g = AP_.lock();
    g.fum = ev;
}


pub fn yut(dhc: u32, ev: u32) {
    let mut g = AP_.lock();
    if dhc == BXT_ || dhc == BXS_ {
        g.gpn = ev;
    }
}






pub fn cfa(ev: u32) {
    let mut g = AP_.lock();
    if g.fli {
        g.eek = TM_;
        return;
    }
    g.lvl = ev;
    g.lm.clear();
    g.fli = true;
}


pub fn cfb() {
    let mut g = AP_.lock();
    if !g.fli {
        g.eek = TM_;
        return;
    }
    g.fli = false;

    
    let uqw = g.ted();
    let dxp = g.jvp;
    let ddi = g.fyh;
    let att = g.faz as f32;
    let azc = g.fyg as f32;
    let eor = g.iqy;
    let ues = g.jdl;
    let gpn = g.gpn;
    let fum = g.fum;
    
    
    let lm = core::mem::take(&mut g.lm);
    let vle = g.lvl;
    
    
    
    let igw = g.faz;
    let bty = &mut g.bty as *mut Vec<f32>;
    
    
    let jtd = texture::tze();
    
    
    let mut ayk: Vec<Option<(i32, i32, f32, Color2D, f32, f32)>> = Vec::fc(lm.len());
    for p in &lm {
        let few = uqw.pvy(Vec4::nwk(p.qf, 1.0));
        if few.d > 0.0 {
            let urn = few.b / few.d;
            let uro = few.c / few.d;
            let urp = few.av / few.d;
            
            let xu = dxp + ((urn + 1.0) * 0.5 * att) as i32;
            let abi = ddi + ((1.0 - uro) * 0.5 * azc) as i32;
            
            let mut s = p.s;
            
            
            if ues {
                let csc = Vec3::new(0.5, -1.0, -0.5).all();
                let hsl = p.adg.amb(-csc).am(0.0);
                let cvo = 0.2;
                let hj = cvo + (1.0 - cvo) * hsl;
                s = Color2D::xt(
                    (s.m as f32 * hj) as u8,
                    (s.at as f32 * hj) as u8,
                    (s.o as f32 * hj) as u8,
                );
            }
            
            ayk.push(Some((xu, abi, urp, s, p.gui.0, p.gui.1)));
        } else {
            ayk.push(None);
        }
    }

    
    match vle {
        BXY_ => {
            for ai in &ayk {
                if let Some((b, c, av, s, tm, p)) = ai {
                    
                    let hjk = if jtd {
                        if let Some(guh) = texture::pfg(*tm, *p) {
                            guh
                        } else {
                            s.lv()
                        }
                    } else {
                        s.lv()
                    };
                    
                    if *b >= 0 && *c >= 0 && (*b as u32) < igw && (*c as u32) < (azc as u32) {
                        let w = (*c as usize) * (igw as usize) + (*b as usize);
                        unsafe {
                            let ng = &mut *bty;
                            if !eor || *av < ng[w] {
                                ng[w] = *av;
                                framebuffer::sf(*b as u32, *c as u32, hjk);
                            }
                        }
                    }
                }
            }
        }
        
        TO_ => {
            for a in (0..ayk.len()).akt(2) {
                if a + 1 < ayk.len() {
                    if let (Some((fy, fo, _, acw, _, _)), Some((dn, dp, _, _, _, _))) = 
                        (&ayk[a], &ayk[a + 1]) 
                    {
                        ahj(*fy, *fo, *dn, *dp, acw.lv());
                    }
                }
            }
        }
        
        BXU_ => {
            for a in 0..ayk.len().ao(1) {
                if let (Some((fy, fo, _, acw, _, _)), Some((dn, dp, _, _, _, _))) = 
                    (&ayk[a], &ayk[a + 1]) 
                {
                    ahj(*fy, *fo, *dn, *dp, acw.lv());
                }
            }
        }
        
        NE_ => {
            for a in 0..ayk.len() {
                let next = (a + 1) % ayk.len();
                if let (Some((fy, fo, _, acw, _, _)), Some((dn, dp, _, _, _, _))) = 
                    (&ayk[a], &ayk[next]) 
                {
                    ahj(*fy, *fo, *dn, *dp, acw.lv());
                }
            }
        }
        
        ADA_ => {
            for a in (0..ayk.len()).akt(3) {
                if a + 2 < ayk.len() {
                    if let (Some(ags), Some(pr), Some(pf)) = 
                        (&ayk[a], &ayk[a + 1], &ayk[a + 2]) 
                    {
                        if gpn == ATH_ {
                            
                            ahj(ags.0, ags.1, pr.0, pr.1, ags.3.lv());
                            ahj(pr.0, pr.1, pf.0, pf.1, pr.3.lv());
                            ahj(pf.0, pf.1, ags.0, ags.1, pf.3.lv());
                        } else {
                            
                            unsafe {
                                let ng = &mut *bty;
                                kqy(
                                    ags.0, ags.1, ags.2, ags.3, ags.4, ags.5,
                                    pr.0, pr.1, pr.2, pr.3, pr.4, pr.5,
                                    pf.0, pf.1, pf.2, pf.3, pf.4, pf.5,
                                    igw,
                                    eor,
                                    fum == TP_,
                                    jtd,
                                    ng,
                                );
                            }
                        }
                    }
                }
            }
        }
        
        KG_ => {
            
            for a in (0..ayk.len()).akt(4) {
                if a + 3 < ayk.len() {
                    if let (Some(ags), Some(pr), Some(pf), Some(bnt)) = 
                        (&ayk[a], &ayk[a + 1], &ayk[a + 2], &ayk[a + 3]) 
                    {
                        if gpn == ATH_ {
                            ahj(ags.0, ags.1, pr.0, pr.1, ags.3.lv());
                            ahj(pr.0, pr.1, pf.0, pf.1, pr.3.lv());
                            ahj(pf.0, pf.1, bnt.0, bnt.1, pf.3.lv());
                            ahj(bnt.0, bnt.1, ags.0, ags.1, bnt.3.lv());
                        } else {
                            unsafe {
                                let ng = &mut *bty;
                                
                                kqy(
                                    ags.0, ags.1, ags.2, ags.3, ags.4, ags.5,
                                    pr.0, pr.1, pr.2, pr.3, pr.4, pr.5,
                                    pf.0, pf.1, pf.2, pf.3, pf.4, pf.5,
                                    igw,
                                    eor,
                                    fum == TP_,
                                    jtd,
                                    ng,
                                );
                                
                                kqy(
                                    ags.0, ags.1, ags.2, ags.3, ags.4, ags.5,
                                    pf.0, pf.1, pf.2, pf.3, pf.4, pf.5,
                                    bnt.0, bnt.1, bnt.2, bnt.3, bnt.4, bnt.5,
                                    igw,
                                    eor,
                                    fum == TP_,
                                    jtd,
                                    ng,
                                );
                            }
                        }
                    }
                }
            }
        }
        
        _ => {}
    }
}


pub fn drf(m: f32, at: f32, o: f32) {
    let mut g = AP_.lock();
    g.dpk = Color2D::xt(
        (m.qp(0.0, 1.0) * 255.0) as u8,
        (at.qp(0.0, 1.0) * 255.0) as u8,
        (o.qp(0.0, 1.0) * 255.0) as u8,
    );
}


pub fn yum(m: u8, at: u8, o: u8) {
    let mut g = AP_.lock();
    g.dpk = Color2D::xt(m, at, o);
}


pub fn erd(m: f32, at: f32, o: f32, q: f32) {
    let mut g = AP_.lock();
    g.dpk = Color2D::dbi(
        (m.qp(0.0, 1.0) * 255.0) as u8,
        (at.qp(0.0, 1.0) * 255.0) as u8,
        (o.qp(0.0, 1.0) * 255.0) as u8,
        (q.qp(0.0, 1.0) * 255.0) as u8,
    );
}


pub fn bnc(b: f32, c: f32, av: f32) {
    let mut g = AP_.lock();
    g.iqc = Vec3::new(b, c, av).all();
}


pub fn azz(e: f32, ab: f32) {
    let mut g = AP_.lock();
    g.iqe = (e, ab);
}


pub fn jx(b: f32, c: f32, av: f32) {
    let mut g = AP_.lock();
    if !g.fli {
        g.eek = TM_;
        return;
    }
    
    
    let s = g.dpk;
    let adg = g.iqc;
    let gui = g.iqe;
    g.lm.push(Auc {
        qf: Vec3::new(b, c, av),
        s,
        adg,
        gui,
    });
}


#[inline]
pub fn yux(b: f32, c: f32) {
    jx(b, c, 0.0);
}



pub fn yuy(by: &[(f32, f32, f32)]) {
    let mut g = AP_.lock();
    if !g.fli {
        g.eek = TM_;
        return;
    }
    let s = g.dpk;
    let adg = g.iqc;
    let gui = g.iqe;
    g.lm.pcn(by.len());
    for &(b, c, av) in by {
        g.lm.push(Auc {
            qf: Vec3::new(b, c, av),
            s,
            adg,
            gui,
        });
    }
}


pub fn tfr() {
    
}


pub fn yuw() {
    framebuffer::sv();
}


pub fn yur() -> u32 {
    let mut g = AP_.lock();
    let rq = g.eek;
    g.eek = ATK_;
    rq
}





fn ahj(fy: i32, fo: i32, dn: i32, dp: i32, s: u32) {
    let dx = (dn - fy).gp();
    let bg = -(dp - fo).gp();
    let cr = if fy < dn { 1 } else { -1 };
    let cq = if fo < dp { 1 } else { -1 };
    let mut rq = dx + bg;
    let mut b = fy;
    let mut c = fo;

    loop {
        if b >= 0 && c >= 0 {
            framebuffer::sf(b as u32, c as u32, s);
        }
        if b == dn && c == dp { break; }
        let agl = 2 * rq;
        if agl >= bg { rq += bg; b += cr; }
        if agl <= dx { rq += dx; c += cq; }
    }
}

fn yna(
    fy: i32, fo: i32, alw: f32, acw: Color2D,
    dn: i32, dp: i32, aeu: f32, rw: Color2D,
    hy: i32, jz: i32, ahc: f32, tx: Color2D,
    z: u32,
    eor: bool,
    iax: bool,
    bty: &mut Vec<f32>,
) {
    
    let mut by = [(fy, fo, alw, acw), (dn, dp, aeu, rw), (hy, jz, ahc, tx)];
    by.bxf(|p| p.1);
    
    let (fy, fo, alw, acw) = by[0];
    let (dn, dp, aeu, rw) = by[1];
    let (hy, jz, ahc, tx) = by[2];
    
    if fo == jz { return; } 
    
    
    let iuz = acw;
    
    
    for c in fo.am(0)..=jz {
        if c < 0 { continue; }
        
        let (bpj, dnt, cdp, dnk, ihy, aiv) = if c < dp {
            
            if dp == fo {
                let ab = if jz != fo { (c - fo) as f32 / (jz - fo) as f32 } else { 0.0 };
                let bpj = fy + ((hy - fy) as f32 * ab) as i32;
                let dnt = alw + (ahc - alw) * ab;
                (bpj, dnt, cal(acw, tx, ab), bpj, dnt, cal(acw, tx, ab))
            } else {
                let aax = (c - fo) as f32 / (dp - fo) as f32;
                let aco = (c - fo) as f32 / (jz - fo) as f32;
                (
                    fy + ((dn - fy) as f32 * aax) as i32,
                    alw + (aeu - alw) * aax,
                    cal(acw, rw, aax),
                    fy + ((hy - fy) as f32 * aco) as i32,
                    alw + (ahc - alw) * aco,
                    cal(acw, tx, aco),
                )
            }
        } else {
            
            if jz == dp {
                (dn, aeu, rw, hy, ahc, tx)
            } else {
                let aax = (c - dp) as f32 / (jz - dp) as f32;
                let aco = (c - fo) as f32 / (jz - fo) as f32;
                (
                    dn + ((hy - dn) as f32 * aax) as i32,
                    aeu + (ahc - aeu) * aax,
                    cal(rw, tx, aax),
                    fy + ((hy - fy) as f32 * aco) as i32,
                    alw + (ahc - alw) * aco,
                    cal(acw, tx, aco),
                )
            }
        };
        
        let (ql, cqe, ibs, kto, ibr, hib) = if bpj < dnk {
            (bpj, dnk, dnt, ihy, cdp, aiv)
        } else {
            (dnk, bpj, ihy, dnt, aiv, cdp)
        };
        
        for b in ql.am(0)..=cqe {
            if b < 0 || b as u32 >= z { continue; }
            
            let ab = if cqe != ql {
                (b - ql) as f32 / (cqe - ql) as f32
            } else {
                0.0
            };
            
            let av = ibs + (kto - ibs) * ab;
            let s = if iax {
                cal(ibr, hib, ab)
            } else {
                iuz
            };
            
            let w = (c as usize) * (z as usize) + (b as usize);
            if w < bty.len() {
                if !eor || av < bty[w] {
                    bty[w] = av;
                    framebuffer::sf(b as u32, c as u32, s.lv());
                }
            }
        }
    }
}

fn cal(acw: Color2D, rw: Color2D, ab: f32) -> Color2D {
    Color2D::xt(
        (acw.m as f32 + (rw.m as f32 - acw.m as f32) * ab) as u8,
        (acw.at as f32 + (rw.at as f32 - acw.at as f32) * ab) as u8,
        (acw.o as f32 + (rw.o as f32 - acw.o as f32) * ab) as u8,
    )
}


fn kqy(
    fy: i32, fo: i32, alw: f32, acw: Color2D, dxd: f32, abk: f32,
    dn: i32, dp: i32, aeu: f32, rw: Color2D, gvi: f32, agy: f32,
    hy: i32, jz: i32, ahc: f32, tx: Color2D, fxr: f32, apg: f32,
    z: u32,
    eor: bool,
    iax: bool,
    xge: bool,
    bty: &mut Vec<f32>,
) {
    
    let mut by = [
        (fy, fo, alw, acw, dxd, abk), 
        (dn, dp, aeu, rw, gvi, agy), 
        (hy, jz, ahc, tx, fxr, apg)
    ];
    by.bxf(|p| p.1);
    
    let (fy, fo, alw, acw, dxd, abk) = by[0];
    let (dn, dp, aeu, rw, gvi, agy) = by[1];
    let (hy, jz, ahc, tx, fxr, apg) = by[2];
    
    if fo == jz { return; }
    
    let iuz = acw;
    
    for c in fo.am(0)..=jz {
        if c < 0 { continue; }
        
        
        let (bpj, dnt, cdp, ifp, asf, dnk, ihy, aiv, pwx, cci) = if c < dp {
            if dp == fo {
                let ab = if jz != fo { (c - fo) as f32 / (jz - fo) as f32 } else { 0.0 };
                let bpj = fy + ((hy - fy) as f32 * ab) as i32;
                let dnt = alw + (ahc - alw) * ab;
                let ifp = dxd + (fxr - dxd) * ab;
                let asf = abk + (apg - abk) * ab;
                (bpj, dnt, cal(acw, tx, ab), ifp, asf, bpj, dnt, cal(acw, tx, ab), ifp, asf)
            } else {
                let aax = (c - fo) as f32 / (dp - fo) as f32;
                let aco = (c - fo) as f32 / (jz - fo) as f32;
                (
                    fy + ((dn - fy) as f32 * aax) as i32,
                    alw + (aeu - alw) * aax,
                    cal(acw, rw, aax),
                    dxd + (gvi - dxd) * aax,
                    abk + (agy - abk) * aax,
                    fy + ((hy - fy) as f32 * aco) as i32,
                    alw + (ahc - alw) * aco,
                    cal(acw, tx, aco),
                    dxd + (fxr - dxd) * aco,
                    abk + (apg - abk) * aco,
                )
            }
        } else {
            if jz == dp {
                (dn, aeu, rw, gvi, agy, hy, ahc, tx, fxr, apg)
            } else {
                let aax = (c - dp) as f32 / (jz - dp) as f32;
                let aco = (c - fo) as f32 / (jz - fo) as f32;
                (
                    dn + ((hy - dn) as f32 * aax) as i32,
                    aeu + (ahc - aeu) * aax,
                    cal(rw, tx, aax),
                    gvi + (fxr - gvi) * aax,
                    agy + (apg - agy) * aax,
                    fy + ((hy - fy) as f32 * aco) as i32,
                    alw + (ahc - alw) * aco,
                    cal(acw, tx, aco),
                    dxd + (fxr - dxd) * aco,
                    abk + (apg - abk) * aco,
                )
            }
        };
        
        let (ql, cqe, ibs, kto, ibr, hib, pof, slo, poh, slp) = 
            if bpj < dnk {
                (bpj, dnk, dnt, ihy, cdp, aiv, ifp, pwx, asf, cci)
            } else {
                (dnk, bpj, ihy, dnt, aiv, cdp, pwx, ifp, cci, asf)
            };
        
        for b in ql.am(0)..=cqe {
            if b < 0 || b as u32 >= z { continue; }
            
            let ab = if cqe != ql {
                (b - ql) as f32 / (cqe - ql) as f32
            } else {
                0.0
            };
            
            let av = ibs + (kto - ibs) * ab;
            let tm = pof + (slo - pof) * ab;
            let p = poh + (slp - poh) * ab;
            
            
            let hjk = if xge {
                if let Some(guh) = texture::pfg(tm, p) {
                    
                    if iax {
                        let cvd = cal(ibr, hib, ab);
                        let agd = ((guh >> 16) & 0xFF) as u32;
                        let ejs = ((guh >> 8) & 0xFF) as u32;
                        let bov = (guh & 0xFF) as u32;
                        let m = (agd * cvd.m as u32 / 255).v(255);
                        let at = (ejs * cvd.at as u32 / 255).v(255);
                        let o = (bov * cvd.o as u32 / 255).v(255);
                        (0xFF << 24) | (m << 16) | (at << 8) | o
                    } else {
                        guh
                    }
                } else {
                    let s = if iax {
                        cal(ibr, hib, ab)
                    } else {
                        iuz
                    };
                    s.lv()
                }
            } else {
                let s = if iax {
                    cal(ibr, hib, ab)
                } else {
                    iuz
                };
                s.lv()
            };
            
            let w = (c as usize) * (z as usize) + (b as usize);
            if w < bty.len() {
                if !eor || av < bty[w] {
                    bty[w] = av;
                    framebuffer::sf(b as u32, c as u32, hjk);
                }
            }
        }
    }
}






pub fn yva(aw: f32) {
    let e = aw / 2.0;
    
    cfa(KG_);
    
    
    bnc(0.0, 0.0, 1.0);
    jx(-e, -e, e);
    jx(e, -e, e);
    jx(e, e, e);
    jx(-e, e, e);
    
    
    bnc(0.0, 0.0, -1.0);
    jx(e, -e, -e);
    jx(-e, -e, -e);
    jx(-e, e, -e);
    jx(e, e, -e);
    
    
    bnc(0.0, 1.0, 0.0);
    jx(-e, e, e);
    jx(e, e, e);
    jx(e, e, -e);
    jx(-e, e, -e);
    
    
    bnc(0.0, -1.0, 0.0);
    jx(-e, -e, -e);
    jx(e, -e, -e);
    jx(e, -e, e);
    jx(-e, -e, e);
    
    
    bnc(1.0, 0.0, 0.0);
    jx(e, -e, e);
    jx(e, -e, -e);
    jx(e, e, -e);
    jx(e, e, e);
    
    
    bnc(-1.0, 0.0, 0.0);
    jx(-e, -e, -e);
    jx(-e, -e, e);
    jx(-e, e, e);
    jx(-e, e, -e);
    
    cfb();
}


pub fn yvc(aw: f32) {
    let e = aw / 2.0;
    
    cfa(NE_);
    
    jx(-e, -e, e);
    jx(e, -e, e);
    jx(e, e, e);
    jx(-e, e, e);
    cfb();
    
    cfa(NE_);
    
    jx(-e, -e, -e);
    jx(e, -e, -e);
    jx(e, e, -e);
    jx(-e, e, -e);
    cfb();
    
    cfa(TO_);
    
    jx(-e, -e, e); jx(-e, -e, -e);
    jx(e, -e, e); jx(e, -e, -e);
    jx(e, e, e); jx(e, e, -e);
    jx(-e, e, e); jx(-e, e, -e);
    cfb();
}


pub fn yvb(aw: f32) {
    
    let zpr = 2;
    let ab = (1.0 + 5.0_f32.ibi()) / 2.0;
    
    
    let lm = [
        Vec3::new(-1.0, ab, 0.0).all().bv(aw),
        Vec3::new(1.0, ab, 0.0).all().bv(aw),
        Vec3::new(-1.0, -ab, 0.0).all().bv(aw),
        Vec3::new(1.0, -ab, 0.0).all().bv(aw),
        Vec3::new(0.0, -1.0, ab).all().bv(aw),
        Vec3::new(0.0, 1.0, ab).all().bv(aw),
        Vec3::new(0.0, -1.0, -ab).all().bv(aw),
        Vec3::new(0.0, 1.0, -ab).all().bv(aw),
        Vec3::new(ab, 0.0, -1.0).all().bv(aw),
        Vec3::new(ab, 0.0, 1.0).all().bv(aw),
        Vec3::new(-ab, 0.0, -1.0).all().bv(aw),
        Vec3::new(-ab, 0.0, 1.0).all().bv(aw),
    ];
    
    let ks = [
        (0, 11, 5), (0, 5, 1), (0, 1, 7), (0, 7, 10), (0, 10, 11),
        (1, 5, 9), (5, 11, 4), (11, 10, 2), (10, 7, 6), (7, 1, 8),
        (3, 9, 4), (3, 4, 2), (3, 2, 6), (3, 6, 8), (3, 8, 9),
        (4, 9, 5), (2, 4, 11), (6, 2, 10), (8, 6, 7), (9, 8, 1),
    ];
    
    cfa(ADA_);
    for (q, o, r) in &ks {
        let asf = lm[*q];
        let cci = lm[*o];
        let cvd = lm[*r];
        let adg = (cci - asf).bjr(cvd - asf).all();
        
        bnc(adg.b, adg.c, adg.av);
        jx(asf.b, asf.c, asf.av);
        jx(cci.b, cci.c, cci.av);
        jx(cvd.b, cvd.c, cvd.av);
    }
    cfb();
}


pub fn tgg(aw: f32) {
    let e = aw / 2.0;
    
    cfa(KG_);
    
    
    bnc(0.0, 0.0, 1.0);
    azz(0.0, 0.0); jx(-e, -e, e);
    azz(1.0, 0.0); jx(e, -e, e);
    azz(1.0, 1.0); jx(e, e, e);
    azz(0.0, 1.0); jx(-e, e, e);
    
    
    bnc(0.0, 0.0, -1.0);
    azz(0.0, 0.0); jx(e, -e, -e);
    azz(1.0, 0.0); jx(-e, -e, -e);
    azz(1.0, 1.0); jx(-e, e, -e);
    azz(0.0, 1.0); jx(e, e, -e);
    
    
    bnc(0.0, 1.0, 0.0);
    azz(0.0, 0.0); jx(-e, e, e);
    azz(1.0, 0.0); jx(e, e, e);
    azz(1.0, 1.0); jx(e, e, -e);
    azz(0.0, 1.0); jx(-e, e, -e);
    
    
    bnc(0.0, -1.0, 0.0);
    azz(0.0, 0.0); jx(-e, -e, -e);
    azz(1.0, 0.0); jx(e, -e, -e);
    azz(1.0, 1.0); jx(e, -e, e);
    azz(0.0, 1.0); jx(-e, -e, e);
    
    
    bnc(1.0, 0.0, 0.0);
    azz(0.0, 0.0); jx(e, -e, e);
    azz(1.0, 0.0); jx(e, -e, -e);
    azz(1.0, 1.0); jx(e, e, -e);
    azz(0.0, 1.0); jx(e, e, e);
    
    
    bnc(-1.0, 0.0, 0.0);
    azz(0.0, 0.0); jx(-e, -e, -e);
    azz(1.0, 0.0); jx(-e, -e, e);
    azz(1.0, 1.0); jx(-e, e, e);
    azz(0.0, 1.0); jx(-e, e, -e);
    
    cfb();
}


pub fn rvm(ezp: &mut u32) {
    texture::tfs(1, core::slice::yrq(ezp));
    texture::nyv(texture::CW_, *ezp);
    
    let xfm = texture::rqm(64, 0xFFFFFFFF, 0xFF404040);
    texture::tfu(
        texture::CW_, 0, texture::ACZ_,
        64, 64, 0, texture::ACZ_, 0, &xfm
    );
    texture::nzb(texture::CW_, texture::ATM_, texture::ATI_);
    texture::nzb(texture::CW_, texture::ATN_, texture::ATJ_);
}


pub fn rvn(hg: f32, ezp: u32) {
    
    texture::nyv(texture::CW_, ezp);
    texture::tfq(texture::CW_);
    
    
    drf(1.0, 1.0, 1.0);
    
    nza();
    ixb(hg, 0.5, 1.0, 0.3);
    tgg(1.5);
    nyz();
    
    texture::tfp(texture::CW_);
}
