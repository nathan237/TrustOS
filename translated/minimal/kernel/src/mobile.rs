



























use alloc::string::String;
use alloc::vec::Vec;

use crate::framebuffer;





const RK_: u32 = 0xFF050606;
const JJ_: u32 = 0xFF070B09;
const ELB_: u32 = 0xFF0A0F0C;
const ELA_: u32 = 0xFF0D1310;

const I_: u32 = 0xFF00FF66;
const AG_: u32 = 0xFF00CC55;
const BK_: u32 = 0xFF00AA44;
const X_: u32 = 0xFF008844;
const BH_: u32 = 0xFF006633;
const P_: u32 = 0xFF003B1A;

const EC_: u32 = 0xFFB0B2B0;
const GC_: u32 = 0xFF8C8E8C;
const AT_: u32 = 0xFF606260;
const AJ_: u32 = 0xFF3A3C3A;

const EKY_: u32 = 0xFFFFD166;
const DC_: u32 = 0xFFFF5555;
const EKZ_: u32 = 0xFF4ECDC4;

const AC_: u32 = 0xFFE0E8E4;
const N_: u32 = 0xFF8A9890;
const ELN_: u32 = 0xFF00CC55;






pub const DRX_: u32 = 1179;
pub const DRU_: u32 = 2556;

pub const DRW_: u32 = 393;
pub const DRV_: u32 = 852;

pub const CCM_: u32 = 195;
pub const CCL_: u32 = 90;




pub const DAH_: u32 = 498;
pub const DAG_: u32 = 1080;





const AIF_: u32 = 44;
const GH_: u32 = 90;
const IC_: u32 = 5;
const BWR_: u32 = 134;


const EJ_: u32 = 3;
const BV_: u32 = 72;
const NO_: u32 = 18;
const CBF_: u32 = 18;
const UB_: u32 = BV_ + CBF_ + 24; 
const TS_: u32 = 20; 


const HV_: usize = 5;
const BU_: u32 = 52;
const ABD_: u32 = 20;
const SS_: u32 = 12;


pub const HL_: u32 = 36;


const ZX_: u32 = 20;


const BGR_: u32 = 16;
const BGS_: u32 = 14;





#[derive(Clone, Copy)]
pub struct Ix {
    pub j: &'static str,
    pub caf: u8,
    pub mm: u32,
}


const IM_: &[Ix] = &[
    Ix { j: "Terminal",  caf: 0,  mm: 0xFF20CC60 },
    Ix { j: "Files",     caf: 1,  mm: 0xFFDDAA30 },
    Ix { j: "Editor",    caf: 2,  mm: 0xFF5090E0 },
    Ix { j: "Calc",      caf: 3,  mm: 0xFFCC6633 },
    Ix { j: "Network",   caf: 4,  mm: 0xFF40AADD },
    Ix { j: "Games",     caf: 5,  mm: 0xFFCC4444 },
    Ix { j: "Browser",   caf: 6,  mm: 0xFF4488DD },
    Ix { j: "TrustEd",   caf: 7,  mm: 0xFF9060D0 },
    Ix { j: "Settings",  caf: 8,  mm: 0xFF9988BB },
    Ix { j: "About",     caf: 9,  mm: 0xFF40CC80 },
    Ix { j: "Music",     caf: 10, mm: 0xFFFF6090 },
    Ix { j: "Chess",     caf: 11, mm: 0xFFD4A854 },
];


const AQM_: [usize; HV_] = [0, 1, 6, 10, 8];





#[derive(Clone, Copy, PartialEq)]
pub enum MobileView {
    Lo,
    Zv,
    Apa,
    Wc,
}

#[derive(Clone)]
pub struct MobileState {
    pub gh: bool,
    pub bls: MobileView,
    pub fcj: Option<u32>,
    pub tps: i32,
    pub tpr: i32,
    pub wws: i32,
    pub dzl: u8,
    pub uvs: u8,
    pub kyf: i32,
    pub kyg: i32,
    pub kyb: bool,
    pub kyc: bool,
    pub kyd: bool,
    pub dod: u64,
    pub hmx: i32,
    pub job: String,
    pub kid: Vec<(u32, u8)>,
    pub bso: String,
    
    pub dxp: i32,
    pub ddi: i32,
    pub att: u32,
    pub azc: u32,
    
    pub gnh: bool,
    
    pub lnc: u8,
    
    pub dwq: Vec<String>,
    
    pub pse: String,
    
    pub aqu: String,
    
    pub fec: u8,
    
    pub coz: i64,
    
    pub dzk: bool,
    
    pub gha: i32,
    
    pub dqu: u8,
    
    pub mey: i32,
    
    pub gsk: [bool; 6],
    
    pub kxp: i32,
    
    pub hbi: u8,
    
    pub isk: u32,
    
    pub gfy: u8,
    
    pub cpd: i32,
    
    pub hct: u8,
}

impl MobileState {
    pub const fn new() -> Self {
        Self {
            gh: false,
            bls: MobileView::Lo,
            fcj: None,
            tps: 0,
            tpr: 0,
            wws: 0,
            dzl: 0,
            uvs: 0,
            kyf: 0,
            kyg: 0,
            kyb: false,
            kyc: false,
            kyd: false,
            dod: 0,
            hmx: -1,
            job: String::new(),
            kid: Vec::new(),
            bso: String::new(),
            dxp: 0,
            ddi: 0,
            att: DAH_,
            azc: DAG_,
            gnh: false,
            lnc: 0,
            dwq: Vec::new(),
            pse: String::new(),
            aqu: String::new(),
            fec: 0,
            coz: 0,
            dzk: false,
            gha: -1,
            dqu: 0,
            mey: -1,
            gsk: [true, false, false, false, true, true],
            kxp: -1,
            hbi: 0,
            isk: 0,
            gfy: 0,
            cpd: -1,
            hct: 0,
        }
    }
}



pub fn nbi(wf: u32, aav: u32) -> (i32, i32, u32, u32) {
    
    let azc = aav;
    let att = (aav * CCL_ / CCM_).v(wf);
    let fp = ((wf.ao(att)) / 2) as i32;
    let iz = 0i32;
    (fp, iz, att, azc)
}





fn cb(b: i32, c: i32, text: &str, s: u32) {
    crate::graphics::scaling::kri(b, c, text, s);
}

fn np(cx: i32, c: i32, text: &str, s: u32) {
    let d = crate::graphics::scaling::clj(text) as i32;
    cb(cx - d / 2, c, text, s);
}

fn bmi() -> u32 {
    crate::graphics::scaling::bmi()
}


fn mf(b: i32, c: i32, d: u32, i: u32, dy: u32, s: u32) {
    if d == 0 || i == 0 { return; }
    let m = dy.v(d / 2).v(i / 2);
    if m == 0 {
        if b >= 0 && c >= 0 { framebuffer::ah(b as u32, c as u32, d, i, s); }
        return;
    }
    let yi = d as i32;
    let gd = i as i32;
    let jl = m as i32;
    
    fip(b, c + jl, yi, gd - jl * 2, s);
    fip(b + jl, c, yi - jl * 2, jl, s);
    fip(b + jl, c + gd - jl, yi - jl * 2, jl, s);
    
    let uv = jl * jl;
    for bg in 0..jl {
        let dx = iua(uv - bg * bg);
        fip(b + jl - dx, c + jl - bg - 1, dx, 1, s);
        fip(b + yi - jl, c + jl - bg - 1, dx, 1, s);
        fip(b + jl - dx, c + gd - jl + bg, dx, 1, s);
        fip(b + yi - jl, c + gd - jl + bg, dx, 1, s);
    }
}


fn tf(b: i32, c: i32, d: u32, i: u32, dy: u32, s: u32) {
    if d == 0 || i == 0 { return; }
    let m = dy.v(d / 2).v(i / 2);
    let yi = d as i32;
    let gd = i as i32;
    let jl = m as i32;
    if m == 0 {
        if b >= 0 && c >= 0 { framebuffer::lx(b as u32, c as u32, d, i, s); }
        return;
    }
    
    for y in (b + jl)..(b + yi - jl) {
        ayl(y, c, s);
        ayl(y, c + gd - 1, s);
    }
    for x in (c + jl)..(c + gd - jl) {
        ayl(b, x, s);
        ayl(b + yi - 1, x, s);
    }
    
    let uv = jl * jl;
    let mut jcv = jl;
    for bg in 0..=jl {
        let dx = iua(uv - bg * bg);
        
        for ax in dx..=jcv {
            
            ayl(b + jl - ax, c + jl - bg, s);
            
            ayl(b + yi - 1 - jl + ax, c + jl - bg, s);
            
            ayl(b + jl - ax, c + gd - 1 - jl + bg, s);
            
            ayl(b + yi - 1 - jl + ax, c + gd - 1 - jl + bg, s);
        }
        jcv = dx;
    }
}


fn fip(b: i32, c: i32, d: i32, i: i32, s: u32) {
    if d <= 0 || i <= 0 || b + d <= 0 || c + i <= 0 { return; }
    let fy = b.am(0) as u32;
    let fo = c.am(0) as u32;
    let dn = (b + d).am(0) as u32;
    let dp = (c + i).am(0) as u32;
    if dn > fy && dp > fo {
        framebuffer::ah(fy, fo, dn - fy, dp - fo, s);
    }
}

fn ayl(b: i32, c: i32, s: u32) {
    if b >= 0 && c >= 0 {
        framebuffer::sf(b as u32, c as u32, s);
    }
}

fn iua(p: i32) -> i32 {
    if p <= 0 { return 0; }
    let mut b = p;
    let mut c = (b + 1) / 2;
    while c < b { b = c; c = (b + p / b) / 2; }
    b
}


fn qku(c: i32, b: i32) -> u32 {
    let ax = if b < 0 { -b } else { b };
    let bga = if c < 0 { -c } else { c };
    
    let bvq = if b >= 0 {
        if c >= 0 {
            if ax >= bga { 0 } else { 1 }
        } else {
            if ax >= bga { 7 } else { 6 }
        }
    } else {
        if c >= 0 {
            if ax >= bga { 3 } else { 2 }
        } else {
            if ax >= bga { 4 } else { 5 }
        }
    };
    bvq
}






fn iru(b: i32, c: i32, d: u32, i: u32, jyj: u32, adh: u32) {
    if d == 0 || i == 0 { return; }
    let mru = b.am(0) as u32;
    let mry = c.am(0) as u32;
    
    framebuffer::ih(mru, mry, d, i, 0x000000, (adh * 7 / 10).v(255));
    
    framebuffer::ih(mru, mry, d, i, 0x001A0A, (adh * 2 / 10).v(255));
    
    let mfm = i / 4;
    for br in 0..mfm {
        let dw = ((mfm - br) * 18 / mfm).v(255);
        if dw > 0 {
            framebuffer::ih(mru, mry + br, d, 1, 0xFFFFFF, dw);
        }
    }
    
    tf(b, c, d, i, jyj, AJ_);
}





pub fn hgw(fp: i32, iz: i32, gm: u32, qdz: u32, bso: &str, eln: u64) {
    let b = fp;
    let c = iz;
    let d = gm;
    let i = AIF_;

    
    framebuffer::ih(b.am(0) as u32, c.am(0) as u32, d, i, 0x040A06, 180);
    framebuffer::ih(b.am(0) as u32, c.am(0) as u32, d, i, 0x00AA44, 8);

    
    framebuffer::ah(b.am(0) as u32, (c + i as i32 - 1).am(0) as u32, d, 1, AJ_);

    
    let dt = bmi();
    np(b + d as i32 / 2, c + 14, bso, EC_);

    
    cb(b + 12, c + 14, "TrustOS", AG_);

    
    let kb = b + d as i32 - 12;
    
    let bx = (kb - 24).am(0) as u32;
    let je = (c + 16).am(0) as u32;
    framebuffer::lx(bx, je, 18, 10, AT_);
    framebuffer::ah(bx + 18, je + 3, 2, 4, AT_);
    framebuffer::ah(bx + 2, je + 2, 14, 6, AG_); 

    
    let fx = (kb - 50) as i32;
    let lw = c + 24;
    for mz in 0..3u32 {
        let m = 2 + mz * 3;
        let uv = (m * m) as i32;
        let jl = m.ao(1);
        let lwt = (jl * jl) as i32;
        for bg in 0..=m as i32 {
            for dx in -(m as i32)..=(m as i32) {
                let us = dx * dx + bg * bg;
                if us <= uv && us >= lwt && bg <= 0 {
                    let bj = if mz == 0 { AG_ } else { X_ };
                    ayl(fx + dx, lw + bg, bj);
                }
            }
        }
    }
    ayl(fx, lw + 1, AG_); 
}





pub fn nnc(
    fp: i32, iz: i32, gm: u32, me: u32,
    tou: i32, eln: u64,
) {
    let bo = IM_.len() as u32;
    let lk = (bo + EJ_ - 1) / EJ_;

    
    
    const AFT_: u32 = 138;
    let ipd = iz + AIF_ as i32;
    let ipc = iz + me as i32 - GH_ as i32 - IC_ as i32 - AFT_ as i32;
    let nd = (ipc - ipd).am(0) as u32;

    
    let ftv = 36u32;
    let pgy = 14u32;
    let bco = gm.ao(pgy * 2);
    let cmt = fp + pgy as i32;
    let blb = ipd + 10;
    
    mf(cmt, blb, bco, ftv, 12, 0xFF0A120E);
    tf(cmt, blb, bco, ftv, 12, AJ_);
    
    let fnl = cmt + 12;
    let fnm = blb + 8;
    
    for bg in -4i32..=4 {
        for dx in -4i32..=4 {
            let us = dx * dx + bg * bg;
            if us >= 9 && us <= 16 {
                ayl(fnl + dx, fnm + bg, AT_);
            }
        }
    }
    
    ayl(fnl + 4, fnm + 4, AT_);
    ayl(fnl + 5, fnm + 5, AT_);
    ayl(fnl + 6, fnm + 6, AT_);
    
    cb(cmt + 26, blb + 10, "Search", N_);

    
    let hly = blb + ftv as i32 + 10;
    let ero = (ipc - hly).am(0) as u32;

    
    let lal = gm.ao(TS_ * 2);
    
    let acc = lal / EJ_; 
    let lao = fp + TS_ as i32;

    let mmf = lk * UB_;
    let drk = hly + (ero as i32 - mmf as i32) / 2;

    for a in 0..bo {
        let bj = a % EJ_;
        let br = a / EJ_;
        let bjf = &IM_[a as usize];

        
        let fg = lao + (bj * acc) as i32 + (acc as i32 - BV_ as i32) / 2;
        let og = drk + (br * UB_) as i32;
        let jbj = tou == a as i32;

        
        
        mf(fg, og, BV_, BV_, NO_, 0xFF060A06);
        if jbj {
            tf(fg, og, BV_, BV_, NO_, bjf.mm);
            
            tf(fg - 1, og - 1, BV_ + 2, BV_ + 2, NO_ + 1, P_);
        } else {
            tf(fg, og, BV_, BV_, NO_, AJ_);
        }

        
        let ads = if jbj { bjf.mm } else { BH_ };
        let cx = fg + BV_ as i32 / 2;
        let ae = og + BV_ as i32 / 2;
        nnd(cx, ae, bjf.caf, ads, jbj);

        
        let bbw = if jbj { I_ } else { N_ };
        let mj = fg + BV_ as i32 / 2;
        let ct = og + BV_ as i32 + 2;
        np(mj, ct, bjf.j, bbw);
    }
}





pub fn irs(fp: i32, iz: i32, gm: u32, me: u32, tov: i32, eln: u64) {
    let eoz = iz + me as i32 - GH_ as i32 - IC_ as i32;
    let eay = fp + SS_ as i32;
    let hgn = gm - SS_ * 2;
    let cjz = GH_ - 10; 

    
    iru(eay, eoz, hgn, cjz, ABD_, 200);
    
    framebuffer::ih(
        (eay + ABD_ as i32).am(0) as u32,
        eoz.am(0) as u32,
        hgn.ao(ABD_ * 2), 1,
        AT_, 120,
    );

    
    let aza = HV_ as u32 * BU_ + (HV_ as u32 - 1) * 12;
    let ql = eay + (hgn as i32 - aza as i32) / 2;
    let bem = eoz + (cjz as i32 - BU_ as i32) / 2;

    for (di, &com) in AQM_.iter().cf() {
        let bjf = &IM_[com];
        let fg = ql + (di as u32 * (BU_ + 12)) as i32;
        let fmb = tov == di as i32;

        
        mf(fg, bem, BU_, BU_, 12, 0xFF060A06);
        if fmb {
            tf(fg, bem, BU_, BU_, 12, bjf.mm);
        } else {
            tf(fg, bem, BU_, BU_, 12, AJ_);
        }

        
        let sci = if fmb { bjf.mm } else { BH_ };
        let cx = fg + BU_ as i32 / 2;
        let ae = bem + BU_ as i32 / 2;
        nnd(cx, ae, bjf.caf, sci, fmb);

        
        let bbw = if fmb { I_ } else { N_ };
        let mj = fg + BU_ as i32 / 2;
        let ct = bem + BU_ as i32 + 2;
        np(mj, ct, bjf.j, bbw);
    }
}





pub fn irt(fp: i32, iz: i32, gm: u32, me: u32) {
    let lo = BWR_;
    let tn = IC_;
    let bx = fp + (gm as i32 - lo as i32) / 2;
    let je = iz + me as i32 - tn as i32 - 4;
    mf(bx, je, lo, tn, 3, EC_);
}





pub fn sbe(fp: i32, iz: i32, gm: u32, dq: &str, eln: u64) {
    let i = HL_;
    
    framebuffer::ih(fp.am(0) as u32, iz.am(0) as u32, gm, i, 0x0A1A0A, 220);
    
    framebuffer::ah((fp).am(0) as u32, (iz + i as i32 - 1).am(0) as u32, gm, 1, AT_);
    
    framebuffer::ih(fp.am(0) as u32, iz.am(0) as u32, gm, 1, 0x00FF66, 15);
    
    np(fp + gm as i32 / 2, iz + 10, dq, AC_);
    
    cb(fp + 10, iz + 10, "<", AG_);
}





pub fn sbo(
    fp: i32, iz: i32, gm: u32, me: u32,
    ee: &[(u32, &str)], 
    cms: i32, eln: u64,
) {
    
    framebuffer::ih(fp.am(0) as u32, iz.am(0) as u32, gm, me, 0x000000, 180);

    if ee.is_empty() {
        np(fp + gm as i32 / 2, iz + me as i32 / 2, "No apps open", N_);
        return;
    }

    
    let bpw = (gm * 7 / 10).v(400);
    let bgg = (me * 5 / 10).v(600);
    let gcg = iz + (me as i32 - bgg as i32) / 2;

    let xju = ee.len() as u32 * bpw + (ee.len() as u32).ao(1) * BGR_;
    let ql = fp + (gm as i32 - xju as i32) / 2 - cms;

    for (a, &(ajq, dq)) in ee.iter().cf() {
        let cx = ql + (a as u32 * (bpw + BGR_)) as i32;
        
        mf(cx, gcg, bpw, bgg, BGS_, 0xFF0A0A0A);
        tf(cx, gcg, bpw, bgg, BGS_, AT_);
        
        framebuffer::ih(cx.am(0) as u32, gcg.am(0) as u32, bpw, 28, 0x0A1A0A, 220);
        cb(cx + 10, gcg + 6, dq, AC_);
        
        let bdr = cx + bpw as i32 - 20;
        cb(bdr, gcg + 6, "X", DC_);
        
        np(cx + bpw as i32 / 2, gcg + bgg as i32 / 2, dq, N_);
    }
}





pub fn sck(fp: i32, iz: i32, gm: u32, qdz: u32, li: u8, eln: u64) {
    if li == 0 { return; }
    let i = (340u32 * li as u32 / 100).am(1);
    let d = gm.ao(24);
    let b = fp + 12;
    let c = iz;

    
    iru(b, c, d, i, ZX_, 230);
    
    framebuffer::ih(
        (b + ZX_ as i32).am(0) as u32, c.am(0) as u32,
        d.ao(ZX_ * 2), 1, EC_, 60,
    );

    if i < 100 { return; } 

    let cx = b + d as i32 / 2;
    let mut ty = c + 20;

    
    cb(b + 16, ty, "Brightness", AC_);
    ty += 20;
    let lo = d - 40;
    framebuffer::ah((b + 20).am(0) as u32, ty.am(0) as u32, lo, 6, AJ_);
    framebuffer::ah((b + 20).am(0) as u32, ty.am(0) as u32, lo * 7 / 10, 6, AG_);
    ty += 24;

    
    cb(b + 16, ty, "Volume", AC_);
    ty += 20;
    framebuffer::ah((b + 20).am(0) as u32, ty.am(0) as u32, lo, 6, AJ_);
    framebuffer::ah((b + 20).am(0) as u32, ty.am(0) as u32, lo * 5 / 10, 6, AG_);
    ty += 24;

    
    let bll = 50u32;
    let ezt = 10u32;
    let mkt = ["WiFi", "BT", "Air", "DND"];
    let ptd = [true, false, false, false];
    let xkr = mkt.len() as u32 * bll + (mkt.len() as u32 - 1) * ezt;
    let xgz = b + (d as i32 - xkr as i32) / 2;

    for (a, &cu) in mkt.iter().cf() {
        let gx = xgz + (a as u32 * (bll + ezt)) as i32;
        let ei = if ptd[a] { P_ } else { RK_ };
        let acu = if ptd[a] { AG_ } else { AJ_ };
        mf(gx, ty, bll, bll, 10, ei);
        tf(gx, ty, bll, bll, 10, acu);
        np(gx + bll as i32 / 2, ty + bll as i32 / 2 - 7, cu, AC_);
    }
}





pub fn sex(fp: i32, iz: i32, gm: u32, me: u32) {
    
    tf(fp - 3, iz - 3, gm + 6, me + 6, 18, P_);
    
    tf(fp - 2, iz - 2, gm + 4, me + 4, 16, EC_);
    
    tf(fp - 1, iz - 1, gm + 2, me + 2, 14, AT_);
}





#[derive(Clone, Copy, PartialEq)]
pub enum MobileAction {
    None,
    Atc,
    Bof,
    Bnx,
    Bdp,
    Bkr(u8),      
    Bks(u8),  
    Bcm,
    Bzn(u32),
    Bmx,
    Chp,
    Bmw,
    Bmv(u8),
    
    CalcButton(u8),
    
    Bhb(u8),
    
    Bha,
    
    Bso(u8),
    
    Bhu(u8),
    
    Agv(u8),
    
    Bfu(u8),
    
    Arm(u8),
    
    Bdl(u8),
    
    Bmu,
    
    Buc,
}



pub fn tjt(
    g: &mut MobileState,
    id: GestureEvent,
) -> MobileAction {
    match id {
        GestureEvent::Btz(b, c) => {
            g.kyf = b;
            g.kyg = c;
            g.kyb = true;
            g.kyc = c > g.azc as i32 - 60;
            g.kyd = c < 44;
            
            if g.bls == MobileView::Lo {
                g.hmx = obw(b, c, g.att, g.azc);
            }
            MobileAction::None
        }
        GestureEvent::Bua(b, c) => {
            g.kyb = false;
            let dx = b - g.kyf;
            let bg = c - g.kyg;
            let la = iua(dx * dx + bg * bg);

            if la < 15 {
                
                return tlf(g, b, c);
            }

            
            if bg.gp() > dx.gp() && bg.gp() > 30 {
                if bg < 0 {
                    
                    if g.kyc {
                        
                        if g.bls == MobileView::Zv {
                            return MobileAction::Atc;
                        } else if g.bls == MobileView::Lo {
                            return MobileAction::Bof;
                        }
                    }
                    if g.bls == MobileView::Wc {
                        return MobileAction::Bdp;
                    }
                } else {
                    
                    if g.kyd && g.bls == MobileView::Lo {
                        return MobileAction::Bnx;
                    }
                }
            }

            g.hmx = -1;
            MobileAction::None
        }
        GestureEvent::Fw(jyn, iij) => {
            MobileAction::None
        }
    }
}

fn tlf(g: &mut MobileState, b: i32, c: i32) -> MobileAction {
    match g.bls {
        MobileView::Lo => {
            
            let oor = tpd(b, c, g.att, g.azc, g.gnh);
            if oor != MobileAction::None {
                return oor;
            }
            
            let w = obw(b, c, g.att, g.azc);
            if w >= 0 && (w as usize) < IM_.len() {
                g.hmx = -1;
                return MobileAction::Bkr(w as u8);
            }
            
            let nmk = tpc(b, c, g.att, g.azc);
            if nmk >= 0 {
                return MobileAction::Bks(nmk as u8);
            }
            MobileAction::None
        }
        MobileView::Zv => {
            
            if c < HL_ as i32 && b < 40 {
                return MobileAction::Bcm;
            }
            
            if let Some(com) = g.fcj {
                let alk = c - HL_ as i32;
                return tiy(g, com, b, alk);
            }
            MobileAction::None
        }
        MobileView::Apa => {
            
            MobileAction::Atc
        }
        MobileView::Wc => {
            MobileAction::None
        }
    }
}



fn tiy(g: &MobileState, com: u32, b: i32, c: i32) -> MobileAction {
    let gm = g.att;
    let me = g.azc;
    let ov = 14i32;

    match com {
        
        3 => {
            let fg = ov;
            let adc = (gm as i32 - ov * 2) as u32;
            let cjy = 60i32;
            let qx = 44i32;
            let aib = 6i32;
            let pm = ((adc - 6 * 3) / 4) as i32;
            let nzt = 10 + cjy + 14;
            
            if c >= nzt {
                let br = (c - nzt) / (qx + aib);
                let bj = (b - fg) / (pm + aib);
                if br >= 0 && br < 5 && bj >= 0 && bj < 4 {
                    
                    let qsk: [[u8; 4]; 5] = [
                        [16, 17, 18, 14], 
                        [7,  8,  9,  13], 
                        [4,  5,  6,  12], 
                        [1,  2,  3,  11], 
                        [0,  10, 15, 255],
                    ];
                    let aj = qsk[br as usize][bj as usize];
                    if aj != 255 {
                        return MobileAction::CalcButton(aj);
                    }
                }
            }
            MobileAction::None
        }
        
        1 => {
            let ph = 40i32;
            let hjt = 32;
            if c >= hjt {
                let w = (c - hjt) / ph;
                if w >= 0 && w < 8 {
                    return MobileAction::Bhb(w as u8);
                }
            }
            
            if c < 28 && b < 80 && g.dqu > 0 {
                return MobileAction::Bha;
            }
            MobileAction::None
        }
        
        8 => {
            let ph = 52i32;
            let hjt = 10;
            if c >= hjt {
                let w = (c - hjt) / ph;
                if w >= 0 && w < 6 {
                    return MobileAction::Bso(w as u8);
                }
            }
            MobileAction::None
        }
        
        5 => {
            let bgg = 56i32;
            let kgo = 8i32;
            let nus = 34;
            if c >= nus {
                let w = (c - nus) / (bgg + kgo);
                if w >= 0 && w < 5 {
                    return MobileAction::Bhu(w as u8);
                }
            }
            MobileAction::None
        }
        
        6 => {
            let cgi = 40;
            
            if c >= 4 && c < 34 {
                return MobileAction::Agv(0);
            }
            if g.hbi == 0 {
                
                let eue = cgi + 84;
                let oji = 20;
                if c >= eue && c < eue + oji * 3 {
                    let w = (c - eue) / oji;
                    return MobileAction::Agv(w as u8 + 1);
                }
            } else {
                
                if c >= cgi {
                    return MobileAction::Agv(0);
                }
            }
            MobileAction::None
        }
        
        2 => {
            
            if c < 26 {
                if b < 80 {
                    return MobileAction::Arm(0);
                } else {
                    return MobileAction::Arm(1);
                }
            }
            
            let gy = 16;
            let dez = 30;
            if c >= dez {
                let line = (c - dez) / gy;
                if line >= 0 && line < 12 {
                    return MobileAction::Bfu(line as u8);
                }
            }
            MobileAction::None
        }
        
        11 => {
            let aly = (gm as i32 - 16).v((me.ao(HL_ + 80)) as i32).v(400);
            let cell = aly / 8;
            let aoj = (gm as i32 - aly) / 2;
            let apl = 10;
            if b >= aoj && b < aoj + aly && c >= apl && c < apl + aly {
                let bj = (b - aoj) / cell;
                let br = (c - apl) / cell;
                let im = (br * 8 + bj) as u8;
                return MobileAction::Bdl(im);
            }
            MobileAction::None
        }
        
        10 => {
            let adc = (gm as i32 - ov * 2) as u32;
            let byh = adc.v(200);
            
            let cdw = byh as i32 + 16 + 20 + 28 + 20 + 14;
            let pm = 48i32;
            let qx = 36i32;
            let aib = 10i32;
            let aza = 5 * pm + 4 * aib;
            let kfe = ov + ((adc as i32 - aza) / 2);
            if c >= cdw && c < cdw + qx {
                for a in 0..5 {
                    let bx = kfe + a * (pm + aib);
                    if b >= bx && b < bx + pm {
                        return MobileAction::Bmu;
                    }
                }
            }
            MobileAction::None
        }
        
        0 => {
            
            let nd = me.ao(HL_ + 20);
            if c > nd as i32 - 40 {
                return MobileAction::Buc;
            }
            MobileAction::None
        }
        
        _ => MobileAction::None,
    }
}

#[derive(Clone, Copy)]
pub enum GestureEvent {
    Btz(i32, i32),
    Bua(i32, i32),
    Fw(i32, i32),
}





fn obw(b: i32, c: i32, gm: u32, me: u32) -> i32 {
    let bo = IM_.len() as u32;
    let lk = (bo + EJ_ - 1) / EJ_;
    const AFT_: u32 = 138;
    let ipd = AIF_ as i32;
    let ipc = me as i32 - GH_ as i32 - IC_ as i32 - AFT_ as i32;
    
    let hly = ipd + 56;
    let ero = (ipc - hly).am(0) as u32;

    let lal = gm.ao(TS_ * 2);
    let acc = lal / EJ_;
    let lao = TS_ as i32;

    let mmf = lk * UB_;
    let drk = hly + (ero as i32 - mmf as i32) / 2;

    for a in 0..bo {
        let bj = a % EJ_;
        let br = a / EJ_;
        let fg = lao + (bj * acc) as i32 + (acc as i32 - BV_ as i32) / 2;
        let og = drk + (br * UB_) as i32;
        if b >= fg && b < fg + BV_ as i32 && c >= og && c < og + BV_ as i32 {
            return a as i32;
        }
    }
    -1
}



fn tpd(b: i32, c: i32, gm: u32, me: u32, gfo: bool) -> MobileAction {
    const LO_: u32 = 130;
    const QR_: u32 = 14;
    const GI_: u32 = 26;
    let fys = gm.ao(QR_ * 2);
    let fbh = QR_ as i32;
    let hte = crate::visualizer::IR_ as u32;
    let itn = if gfo { hte * GI_ + 8 } else { 0 };
    let fbi = me as i32 - GH_ as i32 - IC_ as i32 - LO_ as i32 - 8 - itn as i32;

    let aku = LO_ + itn;

    
    if b < fbh || b > fbh + fys as i32 || c < fbi || c > fbi + aku as i32 {
        return MobileAction::None;
    }

    
    let cce = fbi + 12;
    if c >= cce && c < cce + 18 && b > fbh + fys as i32 / 2 {
        return MobileAction::Bmw;
    }

    
    let ov = 14u32;
    let fg = fbh + ov as i32;
    let adc = fys.ao(ov * 2);
    let cdw = fbi + 92;
    let pm = 40i32;
    let qx = 20i32;
    let aib = 6i32;
    let hsg = 5i32;
    let mmb = hsg * pm + (hsg - 1) * aib;
    let kff = fg + (adc as i32 - mmb) / 2;

    if c >= cdw && c <= cdw + qx {
        for cvv in 0..5 {
            let bx = kff + cvv * (pm + aib);
            if b >= bx && b < bx + pm {
                return MobileAction::Bmx;
            }
        }
    }

    
    if gfo {
        let rua = fbi + LO_ as i32 + 4;
        for hrm in 0..hte {
            let ajd = rua + (hrm * GI_) as i32;
            if c >= ajd && c < ajd + GI_ as i32 {
                return MobileAction::Bmv(hrm as u8);
            }
        }
    }

    MobileAction::None
}

fn tpc(b: i32, c: i32, gm: u32, me: u32) -> i32 {
    let eoz = me as i32 - GH_ as i32 - IC_ as i32;
    let eay = SS_ as i32;
    let hgn = gm - SS_ * 2;
    let cjz = GH_ - 10;

    if c < eoz || c > eoz + cjz as i32 { return -1; }

    let aza = HV_ as u32 * BU_ + (HV_ as u32 - 1) * 12;
    let ql = eay + (hgn as i32 - aza as i32) / 2;

    for di in 0..HV_ {
        let fg = ql + (di as u32 * (BU_ + 12)) as i32;
        if b >= fg && b < fg + BU_ as i32 {
            return di as i32;
        }
    }
    -1
}





pub fn xgr(g: &mut MobileState) {
    g.dod = g.dod.cn(1);

    
    if g.bls == MobileView::Wc && g.dzl < 100 {
        g.dzl = (g.dzl + 8).v(100);
    }
    if g.bls != MobileView::Wc && g.dzl > 0 {
        g.dzl = g.dzl.ao(8);
    }

    
    g.kid.zkb(|(ddq, yx)| {
        *yx = yx.ao(4);
        *yx > 0
    });
}





fn nnd(cx: i32, ae: i32, caf: u8, s: u32, xzs: bool) {
    match caf {
        0 => { 
            tf(cx - 14, ae - 10, 28, 20, 3, s);
            framebuffer::ah((cx - 13).am(0) as u32, (ae - 9).am(0) as u32, 26, 3, s);
            
            framebuffer::ah((cx - 11).am(0) as u32, (ae - 8).am(0) as u32, 2, 1, 0xFF0A0A0A);
            framebuffer::ah((cx - 8).am(0) as u32, (ae - 8).am(0) as u32, 2, 1, 0xFF0A0A0A);
            framebuffer::ah((cx - 5).am(0) as u32, (ae - 8).am(0) as u32, 2, 1, 0xFF0A0A0A);
            
            cb(cx - 8, ae - 2, ">", s);
            framebuffer::ah((cx - 2).am(0) as u32, ae.am(0) as u32, 8, 2, s);
        }
        1 => { 
            framebuffer::ah((cx - 14).am(0) as u32, (ae - 8).am(0) as u32, 12, 5, s);
            framebuffer::ah((cx - 14).am(0) as u32, (ae - 3).am(0) as u32, 28, 15, s);
            framebuffer::ah((cx - 12).am(0) as u32, (ae - 1).am(0) as u32, 24, 11, 0xFF0A0A0A);
            framebuffer::ah((cx - 8).am(0) as u32, (ae + 2).am(0) as u32, 16, 1, 0xFF303020);
            framebuffer::ah((cx - 8).am(0) as u32, (ae + 5).am(0) as u32, 12, 1, 0xFF303020);
        }
        2 => { 
            framebuffer::ah((cx - 10).am(0) as u32, (ae - 12).am(0) as u32, 20, 24, s);
            framebuffer::ah((cx + 4).am(0) as u32, (ae - 12).am(0) as u32, 6, 6, 0xFF0A0A0A);
            framebuffer::ah((cx + 4).am(0) as u32, (ae - 12).am(0) as u32, 1, 6, s);
            framebuffer::ah((cx + 4).am(0) as u32, (ae - 7).am(0) as u32, 6, 1, s);
            framebuffer::ah((cx - 8).am(0) as u32, (ae - 6).am(0) as u32, 16, 16, 0xFF0A0A0A);
            framebuffer::ah((cx - 6).am(0) as u32, (ae - 4).am(0) as u32, 6, 1, 0xFF6688CC);
            framebuffer::ah((cx - 6).am(0) as u32, (ae - 2).am(0) as u32, 10, 1, s);
            framebuffer::ah((cx - 6).am(0) as u32, ae.am(0) as u32, 8, 1, 0xFFCC8844);
            framebuffer::ah((cx - 6).am(0) as u32, (ae + 2).am(0) as u32, 12, 1, s);
            framebuffer::ah((cx - 6).am(0) as u32, (ae + 4).am(0) as u32, 4, 1, 0xFF88BB44);
            framebuffer::ah((cx - 6).am(0) as u32, (ae + 6).am(0) as u32, 9, 1, s);
        }
        3 => { 
            tf(cx - 10, ae - 12, 20, 24, 2, s);
            framebuffer::ah((cx - 8).am(0) as u32, (ae - 10).am(0) as u32, 16, 6, 0xFF1A3320);
            cb(cx - 4, ae - 10, "42", 0xFF40FF40);
            for br in 0..3u32 {
                for bj in 0..3u32 {
                    let bx = (cx as u32).nj(8) + bj * 6;
                    let je = (ae as u32).nj(1) + br * 5;
                    let bmc = if br == 2 && bj == 2 { 0xFFCC6633 } else { s };
                    framebuffer::ah(bx.am(0), je.am(0), 4, 3, bmc);
                }
            }
        }
        4 => { 
            let qff = cx;
            let qfg = ae + 4;
            for mz in 0..3u32 {
                let m = 4 + mz * 4;
                let uv = (m * m) as i32;
                let jl = m.ao(2);
                let lwt = (jl * jl) as i32;
                for bg in 0..=m as i32 {
                    for dx in -(m as i32)..=(m as i32) {
                        let us = dx * dx + bg * bg;
                        if us <= uv && us >= lwt && bg <= 0 {
                            let yx = if mz == 0 { s } else { P_ };
                            ayl(qff + dx, qfg + bg, yx);
                        }
                    }
                }
            }
            framebuffer::ah((cx - 1).am(0) as u32, (ae + 3).am(0) as u32, 3, 3, s);
        }
        5 => { 
            framebuffer::ah((cx - 12).am(0) as u32, (ae - 4).am(0) as u32, 24, 12, s);
            framebuffer::ah((cx - 14).am(0) as u32, (ae - 2).am(0) as u32, 4, 8, s);
            framebuffer::ah((cx + 10).am(0) as u32, (ae - 2).am(0) as u32, 4, 8, s);
            framebuffer::ah((cx - 11).am(0) as u32, (ae - 3).am(0) as u32, 22, 10, 0xFF0A0A0A);
            framebuffer::ah((cx - 9).am(0) as u32, (ae - 1).am(0) as u32, 5, 1, s);
            framebuffer::ah((cx - 7).am(0) as u32, (ae - 3).am(0) as u32, 1, 5, s);
            framebuffer::ah((cx + 5).am(0) as u32, (ae - 2).am(0) as u32, 2, 2, 0xFF4488DD);
            framebuffer::ah((cx + 8).am(0) as u32, (ae - 1).am(0) as u32, 2, 2, DC_);
            framebuffer::ah((cx + 5).am(0) as u32, (ae + 1).am(0) as u32, 2, 2, 0xFF44DD44);
            framebuffer::ah((cx + 8).am(0) as u32, (ae + 2).am(0) as u32, 2, 2, 0xFFDDDD44);
        }
        6 => { 
            for bg in -8i32..=8 {
                for dx in -8i32..=8 {
                    let us = dx * dx + bg * bg;
                    if us <= 64 && us >= 49 {
                        ayl(cx + dx, ae + bg, s);
                    }
                }
            }
            
            framebuffer::ah((cx - 1).am(0) as u32, (ae - 7).am(0) as u32, 2, 14, s);
            framebuffer::ah((cx - 7).am(0) as u32, (ae - 1).am(0) as u32, 14, 2, s);
        }
        7 => { 
            
            tf(cx - 8, ae - 6, 16, 12, 1, s);
            
            tf(cx - 4, ae - 10, 16, 12, 1, P_);
            
            ayl(cx - 8, ae - 6, s); ayl(cx - 4, ae - 10, s);
            ayl(cx + 7, ae - 6, s); ayl(cx + 11, ae - 10, s);
        }
        8 => { 
            for bg in 0..18u32 {
                for dx in 0..18u32 {
                    let ym = dx as i32 - 9;
                    let wl = bg as i32 - 9;
                    let dgk = ym * ym + wl * wl;
                    
                    if dgk >= 36 && dgk <= 81 {
                        let hg = qku(wl, ym);
                        if dgk > 56 {
                            
                            if hg % 2 == 0 { ayl(cx - 9 + dx as i32, ae - 9 + bg as i32, s); }
                        } else {
                            ayl(cx - 9 + dx as i32, ae - 9 + bg as i32, s);
                        }
                    }
                    
                    if dgk <= 9 {
                        ayl(cx - 9 + dx as i32, ae - 9 + bg as i32, 0xFF0A0A0A);
                    }
                }
            }
        }
        9 => { 
            for bg in -8i32..=8 {
                for dx in -8i32..=8 {
                    let us = dx * dx + bg * bg;
                    if us <= 64 && us >= 49 { ayl(cx + dx, ae + bg, s); }
                }
            }
            cb(cx - 2, ae - 6, "i", s);
        }
        10 => { 
            
            framebuffer::ah((cx - 3).am(0) as u32, (ae + 2).am(0) as u32, 6, 4, s);
            
            framebuffer::ah((cx + 2).am(0) as u32, (ae - 8).am(0) as u32, 2, 12, s);
            
            framebuffer::ah((cx + 3).am(0) as u32, (ae - 8).am(0) as u32, 4, 2, s);
            framebuffer::ah((cx + 5).am(0) as u32, (ae - 6).am(0) as u32, 2, 2, s);
        }
        11 => { 
            
            framebuffer::ah((cx - 1).am(0) as u32, (ae - 10).am(0) as u32, 2, 6, s);
            framebuffer::ah((cx - 3).am(0) as u32, (ae - 8).am(0) as u32, 6, 2, s);
            
            framebuffer::ah((cx - 4).am(0) as u32, (ae - 4).am(0) as u32, 8, 10, s);
            framebuffer::ah((cx - 3).am(0) as u32, (ae - 3).am(0) as u32, 6, 8, 0xFF0A0A0A);
            
            framebuffer::ah((cx - 6).am(0) as u32, (ae + 5).am(0) as u32, 12, 3, s);
        }
        _ => {
            
            mf(cx - 12, ae - 12, 24, 24, 6, s);
            cb(cx - 3, ae - 6, "?", 0xFF0A0A0A);
        }
    }
}





pub fn qjm() -> usize { IM_.len() }
pub fn kas(w: usize) -> &'static str { IM_[w].j }
pub fn rzy(gk: usize) -> usize { AQM_[gk] }
pub fn ymp() -> usize { HV_ }






pub struct Adc {
    pub uu: bool,
    pub rf: f32,
    pub abo: f32,
    pub ato: f32,
    pub aee: f32,
    pub vs: f32,
    pub axg: f32,
    pub frame: u64,
}




pub fn sdz(
    fp: i32, iz: i32, gm: u32, me: u32,
    audio: &Adc,
    gfo: bool,
    igo: u8,
) {
    const LO_: u32 = 130;
    const QR_: u32 = 14;
    const DBM_: u32 = 16;
    const GI_: u32 = 26;

    let fys = gm.ao(QR_ * 2);
    let fbh = fp + QR_ as i32;
    
    let itn = if gfo { crate::visualizer::IR_ as u32 * GI_ + 8 } else { 0 };
    let fbi = iz + me as i32 - GH_ as i32 - IC_ as i32 - LO_ as i32 - 8 - itn as i32;

    
    iru(fbh, fbi, fys, LO_, DBM_, 210);

    let ov = 14u32;
    let fg = fbh + ov as i32; 
    let adc = fys.ao(ov * 2); 
    let mut ae = fbi + 12;

    
    let dq = if audio.uu { "Now Playing" } else { "Music" };
    let ejy = if audio.uu { I_ } else { GC_ };
    cb(fg, ae, dq, ejy);

    
    let czz = if (igo as usize) < crate::visualizer::OG_.len() {
        crate::visualizer::OG_[igo as usize]
    } else { "Sphere" };
    let onw = crate::graphics::scaling::clj(czz) as i32;
    let qkr = if gfo { "^" } else { "v" };
    let fct = fg + adc as i32 - onw - 14;
    cb(fct, ae, czz, BK_);
    cb(fct + onw + 4, ae, qkr, I_);

    ae += 20;

    
    let tn = 8u32;
    let mxv = 4u32;
    let lo = (adc - mxv * 3) / 4;
    let cdc: [(f32, u32, &str); 4] = [
        (audio.ato, 0xFF00FF44, "SB"),
        (audio.aee,     0xFF00CC88, "BA"),
        (audio.vs,      0xFF00AACC, "MD"),
        (audio.axg,   0xFF8866FF, "TR"),
    ];
    for (cvv, &(jy, s, cu)) in cdc.iter().cf() {
        let bx = fg + (cvv as u32 * (lo + mxv)) as i32;
        
        framebuffer::ih(bx.am(0) as u32, ae.am(0) as u32, lo, tn, 0x112211, 150);
        
        let vi = if audio.uu {
            (jy.v(1.0) * lo as f32) as u32
        } else { 0 };
        if vi > 0 {
            framebuffer::ah(bx.am(0) as u32, ae.am(0) as u32, vi, tn, s);
            framebuffer::ih(bx.am(0) as u32, ae.am(0) as u32, vi, tn, 0xFFFFFF, 15);
        }
        
        let zv = crate::graphics::scaling::clj(cu) as i32;
        cb(bx + (lo as i32 - zv) / 2, ae - 1, cu, 0xFFAABBAA);
    }
    ae += tn as i32 + 8;

    
    let ddk = 36u32;
    framebuffer::ih(fg.am(0) as u32, ae.am(0) as u32, adc, ddk, 0x030908, 160);
    
    framebuffer::ih(fg.am(0) as u32, ae.am(0) as u32, adc, 1, 0x00FF66, 25);
    framebuffer::ih(fg.am(0) as u32, (ae + ddk as i32 - 1).am(0) as u32, adc, 1, 0x00FF66, 15);

    let bkl = ae + ddk as i32 / 2;
    let wp = (ddk / 2 - 2) as f32;

    if audio.uu {
        
        let jgk = adc.v(256) as usize;
        for a in 0..jgk {
            let ab = a as f32 / jgk as f32;
            let ib = audio.frame as f32 * 0.06;
            
            let bic = libm::st(ab * 12.0 + ib) * audio.abo;
            let cuc = libm::st(ab * 28.0 + ib * 1.4) * audio.axg * 0.5;
            let hym = libm::st(ab * 5.0 + ib * 0.7) * audio.aee * 0.7;
            let qol = 1.0 + audio.rf * 0.6;
            let byf = ((bic + cuc + hym) * qol).am(-1.0).v(1.0);
            let mrv = (byf * wp) as i32;

            let y = (fg + a as i32).am(0) as u32;
            let x = ((bkl + mrv).am(ae + 2).v(ae + ddk as i32 - 3)) as u32;

            
            let ght = 0xCC;
            let coo = (audio.rf * 160.0).v(255.0) as u32;
            let vpn = (audio.abo * 50.0).v(50.0) as u32;
            let s = 0xFF000000 | (vpn << 16) | (ght << 8) | coo;
            framebuffer::sf(y, x, s);
            
            framebuffer::sf(y, x, 0xFF00FFCC);
        }
        
        if audio.rf > 0.4 {
            let ceq = ((audio.rf - 0.4) * 40.0).v(30.0) as u32;
            framebuffer::ih(fg.am(0) as u32, ae.am(0) as u32, adc, ddk, 0x00FF88, ceq);
        }
    } else {
        
        framebuffer::ah((fg + 4).am(0) as u32, bkl.am(0) as u32, adc.ao(8), 1, 0xFF334433);
        np(fg + adc as i32 / 2, bkl - 6, "---", 0xFF445544);
    }
    ae += ddk as i32 + 8;

    
    let nhu: &[&str] = &["|<", "<<", if audio.uu { "||" } else { ">" }, ">>", ">|"];
    let hsg = nhu.len() as u32;
    let pm = 40u32;
    let qx = 20u32;
    let aib = 6u32;
    let mmb = hsg * pm + (hsg - 1) * aib;
    let kff = fg + (adc as i32 - mmb as i32) / 2;

    for (cvv, &cu) in nhu.iter().cf() {
        let bx = kff + (cvv as u32 * (pm + aib)) as i32;
        let hpb = cvv == 2;
        let ei = if hpb {
            if audio.uu { 0x00AA55u32 } else { 0x005533u32 }
        } else {
            0x1A2A1Au32
        };
        framebuffer::ih(bx.am(0) as u32, ae.am(0) as u32, pm, qx, ei, 190);
        
        framebuffer::ih(bx.am(0) as u32, ae.am(0) as u32, pm, 1, 0x00FF88, 30);
        let zv = crate::graphics::scaling::clj(cu) as i32;
        let hpv = if hpb { 0xFF00FFAA } else { EC_ };
        cb(bx + (pm as i32 - zv) / 2, ae + 4, cu, hpv);
    }
    ae += qx as i32 + 4;

    
    if gfo {
        let koj = fbh + 8;
        let njt = fys - 16;
        let hte = crate::visualizer::IR_ as u32;
        let rtz = hte * GI_ + 8;
        
        iru(koj, ae, njt, rtz, 12, 230);
        let mut bg = ae + 4;
        for hrm in 0..hte {
            let j = crate::visualizer::OG_[hrm as usize];
            let qe = hrm as u8 == igo;
            if qe {
                framebuffer::ih((koj + 4).am(0) as u32, bg.am(0) as u32, njt - 8, GI_, 0x00FF66, 30);
            }
            let tzy = if qe { I_ } else { N_ };
            let feq = if qe { "> " } else { "  " };
            use alloc::format;
            let cu = format!("{}{}", feq, j);
            cb(koj + 12, bg + 6, &cu, tzy);
            bg += GI_ as i32;
        }
    }
}







pub fn sdy(
    fp: i32, iz: i32, gm: u32, me: u32,
    com: u32, frame: u64, audio: &Adc,
    g: &MobileState,
) {
    let gl = iz + HL_ as i32;
    let nd = me.ao(HL_ + 20);
    
    framebuffer::ih(fp.am(0) as u32, gl.am(0) as u32, gm, nd, 0x050A06, 220);

    match com {
        0 => sbp(fp, gl, gm, nd, frame, g),
        1 => sbj(fp, gl, gm, nd, g),
        2 => sbi(fp, gl, gm, nd, frame, g),
        3 => sbg(fp, gl, gm, nd, frame, g),
        4 => sbm(fp, gl, gm, nd, frame),
        5 => sbk(fp, gl, gm, nd, g),
        6 => sbf(fp, gl, gm, nd, g),
        7 => sbq(fp, gl, gm, nd, frame),
        8 => sbn(fp, gl, gm, nd, g),
        9 => sbd(fp, gl, gm, nd, frame),
        10 => sbl(fp, gl, gm, nd, frame, audio),
        11 => sbh(fp, gl, gm, nd, g),
        _ => {
            np(fp + gm as i32 / 2, gl + nd as i32 / 2, "Unknown App", N_);
        }
    }
}


fn sbp(fp: i32, ae: i32, gm: u32, bm: u32, frame: u64, g: &MobileState) {
    let ov = 10i32;
    let fg = fp + ov;

    
    framebuffer::ih(fp.am(0) as u32, ae.am(0) as u32, gm, 24, 0x0A1A0A, 200);
    cb(fg, ae + 4, "trustos@mobile:~$", I_);

    let gy = 16i32;
    let mut ct = ae + 28;

    
    if g.dwq.is_empty() {
        let xue = [
            "TrustOS v2.0 — Mobile Shell",
            "Type 'help' for available commands.",
            "",
        ];
        for line in &xue {
            if ct + gy > ae + bm as i32 - 40 { break; }
            let s = if line.cj("TrustOS") { AG_ } else { N_ };
            cb(fg, ct, line, s);
            ct += gy;
        }
    }

    
    let ayf = ((bm as i32 - 68) / gy).am(1) as usize;
    let ay = if g.dwq.len() > ayf { g.dwq.len() - ayf } else { 0 };
    for line in &g.dwq[ay..] {
        if ct + gy > ae + bm as i32 - 40 { break; }
        let s = if line.cj("$") { I_ }
                    else if line.cj("TrustOS") { AG_ }
                    else { N_ };
        cb(fg, ct, line, s);
        ct += gy;
    }

    
    let alf = ae + bm as i32 - 36;
    framebuffer::ih(fp.am(0) as u32, alf.am(0) as u32, gm, 32, 0x0A1A0A, 200);
    let aau = alloc::format!("$ {}", g.pse);
    cb(fg, alf + 8, &aau, I_);
    
    if (frame / 30) % 2 == 0 {
        let lf = fg + crate::graphics::scaling::clj(&aau) as i32 + 2;
        framebuffer::ah(lf.am(0) as u32, (alf + 8).am(0) as u32, 8, 14, I_);
    }
    
    np(fp + gm as i32 / 2, alf - 14, "Tap here to run a command", AT_);
}


fn sbj(fp: i32, ae: i32, gm: u32, bm: u32, g: &MobileState) {
    let ov = 10i32;
    let fg = fp + ov;

    
    framebuffer::ih(fp.am(0) as u32, ae.am(0) as u32, gm, 28, 0x0A120E, 200);
    let ltb = if g.dqu == 0 { "/home/user/" } else { "/home/user/Documents/" };
    if g.dqu > 0 {
        cb(fg, ae + 6, "< Back", I_);
        let ars = crate::graphics::scaling::clj(ltb) as i32;
        cb(fp + gm as i32 - ov - ars, ae + 6, ltb, AG_);
    } else {
        cb(fg, ae + 6, ltb, AG_);
    }

    
    let sly: [(&str, &str, u32); 8] = [
        ("Documents", "DIR", 0xFFDDAA30),
        ("Downloads", "DIR", 0xFFDDAA30),
        ("Pictures",  "DIR", 0xFFDDAA30),
        ("Music",     "DIR", 0xFF4488DD),
        ("readme.md", "4KB", N_),
        ("config.toml","2KB", N_),
        ("notes.txt", "1KB", N_),
        ("photo.png", "3MB", 0xFF9060D0),
    ];
    let slz: [(&str, &str, u32); 6] = [
        ("project.rs", "12KB", 0xFF6688CC),
        ("report.pdf", "2MB", 0xFFCC4444),
        ("budget.csv", "8KB", 0xFF40CC80),
        ("slides.md",  "6KB", N_),
        ("backup.zip", "45MB", 0xFF9060D0),
        ("todo.txt",   "1KB", N_),
    ];

    let ph = 40u32;
    let mut ahm = ae + 32;

    if g.dqu == 0 {
        for (a, &(j, aw, s)) in sly.iter().cf() {
            if ahm + ph as i32 > ae + bm as i32 { break; }
            let qe = g.gha == a as i32;
            let ei = if qe { 0x0A2A15 } else { 0x060A08 };
            framebuffer::ih(fp.am(0) as u32, ahm.am(0) as u32, gm, ph, ei, 180);
            framebuffer::ah((fp + 8).am(0) as u32, (ahm + ph as i32 - 1).am(0) as u32, gm.ao(16), 1, AJ_);
            let trd = if aw == "DIR" { ">" } else { "-" };
            let csp = if qe { I_ } else { s };
            cb(fg, ahm + 12, trd, csp);
            cb(fg + 16, ahm + 12, j, csp);
            let kp = crate::graphics::scaling::clj(aw) as i32;
            cb(fp + gm as i32 - ov - kp, ahm + 12, aw, AT_);
            ahm += ph as i32;
        }
    } else {
        for (a, &(j, aw, s)) in slz.iter().cf() {
            if ahm + ph as i32 > ae + bm as i32 { break; }
            let qe = g.gha == a as i32;
            let ei = if qe { 0x0A2A15 } else { 0x060A08 };
            framebuffer::ih(fp.am(0) as u32, ahm.am(0) as u32, gm, ph, ei, 180);
            framebuffer::ah((fp + 8).am(0) as u32, (ahm + ph as i32 - 1).am(0) as u32, gm.ao(16), 1, AJ_);
            let csp = if qe { I_ } else { s };
            cb(fg, ahm + 12, "-", csp);
            cb(fg + 16, ahm + 12, j, csp);
            let kp = crate::graphics::scaling::clj(aw) as i32;
            cb(fp + gm as i32 - ov - kp, ahm + 12, aw, AT_);
            ahm += ph as i32;
        }
    }
}


fn sbi(fp: i32, ae: i32, gm: u32, bm: u32, frame: u64, g: &MobileState) {
    let ov = 10i32;
    let fg = fp + ov;

    
    framebuffer::ih(fp.am(0) as u32, ae.am(0) as u32, gm, 26, 0x0A1A10, 200);
    let xab = if g.gfy == 0 { I_ } else { AT_ };
    let xac = if g.gfy == 1 { I_ } else { AT_ };
    cb(fg, ae + 6, "main.rs", xab);
    cb(fg + 80, ae + 6, "lib.rs", xac);
    
    let xoa = if g.gfy == 0 { fg } else { fg + 80 };
    framebuffer::ah(xoa.am(0) as u32, (ae + 24).am(0) as u32, 50, 2, I_);

    
    let rlj = [
        (1, "fn main() {"),
        (2, "    let os = TrustOS::new();"),
        (3, "    os.init_hardware();"),
        (4, "    os.start_desktop();"),
        (5, ""),
        (6, "    // Mobile mode"),
        (7, "    if os.is_mobile() {"),
        (8, "        os.launch_mobile();"),
        (9, "    }"),
        (10, ""),
        (11, "    os.run_forever();"),
        (12, "}"),
    ];
    let rlk = [
        (1, "pub mod kernel;"),
        (2, "pub mod desktop;"),
        (3, "pub mod mobile;"),
        (4, "pub mod audio;"),
        (5, "pub mod visualizer;"),
        (6, "pub mod network;"),
        (7, ""),
        (8, "pub fn init() {"),
        (9, "    kernel::start();"),
        (10, "}"),
        (11, ""),
        (12, ""),
    ];

    let rlf = if g.gfy == 0 { &rlj } else { &rlk };

    let gy = 16i32;
    let mut ct = ae + 30;
    for &(num, line) in rlf.iter() {
        if ct + gy > ae + bm as i32 { break; }
        use alloc::format;
        let ajh = format!("{:3}", num);
        
        let afb = (num - 1) as u32 == g.isk;
        if afb {
            framebuffer::ih(fp.am(0) as u32, ct.am(0) as u32, gm, gy as u32, 0x1A2A1A, 120);
        }
        cb(fg, ct, &ajh, if afb { I_ } else { AT_ });
        
        let s = if line.contains("fn ") { 0xFF6688CC }
            else if line.contains("let ") || line.contains("pub ") { 0xFF8866FF }
            else if line.contains("//") { 0xFF556655 }
            else if line.contains("TrustOS") || line.contains("mod ") { I_ }
            else { N_ };
        cb(fg + 30, ct, line, s);
        ct += gy;
    }
    
    if (frame / 30) % 2 == 0 {
        let ot = ae + 30 + g.isk as i32 * gy;
        if ot >= ae + 30 && ot < ae + bm as i32 {
            framebuffer::ah((fg + 30).am(0) as u32, ot.am(0) as u32, 2, 14, I_);
        }
    }
}


fn sbg(fp: i32, ae: i32, gm: u32, bm: u32, eln: u64, g: &MobileState) {
    let ov = 14i32;
    let fg = fp + ov;
    let adc = (gm as i32 - ov * 2) as u32;

    
    let cjy = 60u32;
    framebuffer::ih(fg.am(0) as u32, (ae + 10).am(0) as u32, adc, cjy, 0x0A1A10, 220);
    tf(fg, ae + 10, adc, cjy, 8, AJ_);
    let nlt = if g.aqu.is_empty() { "0" } else { &g.aqu };
    let qd = crate::graphics::scaling::clj(nlt) as i32;
    cb(fg + adc as i32 - qd - 10, ae + 30, nlt, I_);

    
    let qsj = [
        ["C", "+/-", "%", "/"],
        ["7", "8", "9", "x"],
        ["4", "5", "6", "-"],
        ["1", "2", "3", "+"],
        ["0", ".", "=", ""],
    ];
    let qx = 44u32;
    let aib = 6u32;
    let pm = (adc - aib * 3) / 4;
    let mut je = ae + 10 + cjy as i32 + 14;

    for br in &qsj {
        let mut bx = fg;
        for &cu in br {
            if cu.is_empty() { bx += (pm + aib) as i32; continue; }
            let ogl = oh!(cu, "/" | "x" | "-" | "+" | "=");
            let txl = oh!(cu, "C" | "+/-" | "%");
            let ei = if ogl { 0xFF008844u32 }
                     else if txl { 0xFF333833 }
                     else { 0xFF1A221A };
            mf(bx, je, pm, qx, 8, ei);
            tf(bx, je, pm, qx, 8, AJ_);
            let hpv = if ogl { I_ } else { AC_ };
            np(bx + pm as i32 / 2, je + 14, cu, hpv);
            bx += (pm + aib) as i32;
        }
        je += (qx + aib) as i32;
    }
}


fn sbm(fp: i32, ae: i32, gm: u32, bm: u32, frame: u64) {
    let ov = 12i32;
    let fg = fp + ov;
    let adc = gm as i32 - ov * 2;

    let mut ct = ae + 10;
    let wga = 28i32;

    
    cb(fg, ct, "WiFi", I_);
    cb(fg + adc - 24, ct, "ON", AG_);
    ct += wga;
    framebuffer::ih(fg.am(0) as u32, ct.am(0) as u32, adc as u32, 40, 0x0A120E, 180);
    cb(fg + 8, ct + 12, "TrustNet-5G", AC_);
    cb(fg + adc - 80, ct + 12, "Connected", AG_);
    ct += 48;

    
    cb(fg, ct, "Network Info", I_);
    ct += 22;
    let co = [
        ("IP Address:", "192.168.1.42"),
        ("Subnet:", "255.255.255.0"),
        ("Gateway:", "192.168.1.1"),
        ("DNS:", "8.8.8.8"),
        ("MAC:", "AA:BB:CC:DD:EE:FF"),
    ];
    for &(cu, bn) in &co {
        if ct + 18 > ae + bm as i32 { break; }
        cb(fg + 8, ct, cu, N_);
        let mqc = crate::graphics::scaling::clj(bn) as i32;
        cb(fg + adc - mqc - 8, ct, bn, AC_);
        ct += 20;
    }
    ct += 10;

    
    cb(fg, ct, "Signal", I_);
    ct += 20;
    let lo = (adc - 16) as u32;
    framebuffer::ah((fg + 8).am(0) as u32, ct.am(0) as u32, lo, 8, AJ_);
    let cug = ((frame % 100) as u32 * lo / 100).am(lo * 7 / 10);
    framebuffer::ah((fg + 8).am(0) as u32, ct.am(0) as u32, cug, 8, AG_);
}


fn sbk(fp: i32, ae: i32, gm: u32, bm: u32, g: &MobileState) {
    let ov = 12i32;
    let fg = fp + ov;
    let adc = (gm as i32 - ov * 2) as u32;

    cb(fg, ae + 10, "Games Library", I_);

    let tad = [
        ("Snake", "Classic arcade", 0xFF44DD44),
        ("Chess", "Strategy board game", 0xFFD4A854),
        ("3D FPS", "Raycasting demo", 0xFF4488DD),
        ("GameBoy", "GB emulator", 0xFF9060D0),
        ("NES", "NES emulator", 0xFFCC4444),
    ];

    let bgg = 56u32;
    let kgo = 8u32;
    let mut ub = ae + 34;

    for (a, &(j, desc, mm)) in tad.iter().cf() {
        if ub + bgg as i32 > ae + bm as i32 { break; }
        let qe = g.kxp == a as i32;
        let ei = if qe { 0xFF0C1610 } else { 0xFF080C0A };
        let acu = if qe { I_ } else { AJ_ };
        mf(fg, ub, adc, bgg, 10, ei);
        tf(fg, ub, adc, bgg, 10, acu);
        
        framebuffer::ah(fg.am(0) as u32, (ub + 8).am(0) as u32, 3, bgg - 16, mm);
        
        cb(fg + 14, ub + 10, j, mm);
        
        cb(fg + 14, ub + 28, desc, N_);
        
        let qsl = if qe { ">>>" } else { ">" };
        let hbk = if qe { I_ } else { AT_ };
        cb(fp + gm as i32 - ov - 30, ub + 16, qsl, hbk);
        ub += (bgg + kgo) as i32;
    }
}


fn sbf(fp: i32, ae: i32, gm: u32, bm: u32, g: &MobileState) {
    let ov = 8i32;
    let fg = fp + ov;
    let adc = (gm as i32 - ov * 2) as u32;

    
    let pxl = 30u32;
    mf(fg, ae + 4, adc, pxl, 10, 0xFF0A120E);
    tf(fg, ae + 4, adc, pxl, 10, AJ_);
    let url = match g.hbi {
        0 => "https://trustos.local",
        1 => "https://trustos.local/docs",
        2 => "https://trustos.local/source",
        3 => "https://trustos.local/downloads",
        _ => "https://trustos.local",
    };
    cb(fg + 10, ae + 12, url, N_);

    
    let cgi = ae + 40;
    framebuffer::ih(fp.am(0) as u32, cgi.am(0) as u32, gm, bm - 40, 0x0C140E, 200);

    let mut ct = cgi + 10;
    match g.hbi {
        0 => {
            cb(fg + 4, ct, "Welcome to TrustOS", I_); ct += 24;
            cb(fg + 4, ct, "A secure, minimal operating system", N_); ct += 20;
            cb(fg + 4, ct, "built with Rust.", N_); ct += 30;
            cb(fg + 4, ct, "> Documentation", 0xFF4488DD); ct += 20;
            cb(fg + 4, ct, "> Source Code", 0xFF4488DD); ct += 20;
            cb(fg + 4, ct, "> Downloads", 0xFF4488DD);
        }
        1 => {
            cb(fg + 4, ct, "Documentation", I_); ct += 24;
            cb(fg + 4, ct, "1. Getting Started", 0xFF4488DD); ct += 20;
            cb(fg + 4, ct, "   Install TrustOS on bare metal", N_); ct += 16;
            cb(fg + 4, ct, "   or run in QEMU/VirtualBox.", N_); ct += 24;
            cb(fg + 4, ct, "2. Mobile Mode", 0xFF4488DD); ct += 20;
            cb(fg + 4, ct, "   Portrait UI for small screens.", N_); ct += 24;
            cb(fg + 4, ct, "3. Desktop Mode", 0xFF4488DD); ct += 20;
            cb(fg + 4, ct, "   Full windowed environment.", N_); ct += 24;
            cb(fg + 4, ct, "4. Audio System", 0xFF4488DD); ct += 20;
            cb(fg + 4, ct, "   HD Audio with DMA.", N_); ct += 30;
            cb(fg + 4, ct, "< Back to Home", AG_);
        }
        2 => {
            cb(fg + 4, ct, "Source Code", I_); ct += 24;
            cb(fg + 4, ct, "Repository:", N_); ct += 20;
            cb(fg + 4, ct, "  github.com/trustos/kernel", 0xFF4488DD); ct += 24;
            cb(fg + 4, ct, "Language: Rust (no_std)", N_); ct += 20;
            cb(fg + 4, ct, "LOC: ~25,000", N_); ct += 20;
            cb(fg + 4, ct, "License: MIT", N_); ct += 30;
            cb(fg + 4, ct, "< Back to Home", AG_);
        }
        3 => {
            cb(fg + 4, ct, "Downloads", I_); ct += 24;
            cb(fg + 4, ct, "TrustOS v2.0 ISO", 0xFF4488DD); ct += 20;
            cb(fg + 4, ct, "  Size: 12 MB | x86_64", N_); ct += 24;
            cb(fg + 4, ct, "TrustOS v2.0 aarch64", 0xFF4488DD); ct += 20;
            cb(fg + 4, ct, "  Size: 14 MB | ARM64", N_); ct += 24;
            cb(fg + 4, ct, "VBox Appliance (.ova)", 0xFF4488DD); ct += 20;
            cb(fg + 4, ct, "  Size: 50 MB | Pre-configured", N_); ct += 30;
            cb(fg + 4, ct, "< Back to Home", AG_);
        }
        _ => {
            cb(fg + 4, ct, "Page not found", N_);
        }
    }

    
    ct = ae + bm as i32 - 18;
    framebuffer::ah((fg + 4).am(0) as u32, (ct - 4).am(0) as u32, adc - 8, 1, AJ_);
    cb(fg + 4, ct, "TrustOS Browser v1.0", AT_);
}


fn sbq(fp: i32, ae: i32, gm: u32, bm: u32, frame: u64) {
    let ov = 10i32;
    let fg = fp + ov;
    let adc = (gm as i32 - ov * 2) as u32;

    
    framebuffer::ih(fp.am(0) as u32, ae.am(0) as u32, gm, 28, 0x0A120E, 200);
    let mlu = ["Move", "Rot", "Scale", "Add"];
    let mut gx = fg;
    for bxo in &mlu {
        cb(gx, ae + 7, bxo, GC_);
        gx += 50;
    }

    
    let ekn = ae + 32;
    let azb = bm.ao(60);
    framebuffer::ih(fp.am(0) as u32, ekn.am(0) as u32, gm, azb, 0x030806, 220);

    
    let nis = fp + gm as i32 / 2;
    let niu = ekn + azb as i32 / 2;
    for a in 0..8u32 {
        let l = (a as i32 - 4) * 20;
        framebuffer::ih(fp.am(0) as u32, (niu + l).am(0) as u32, gm, 1, 0x002A15, 40);
        framebuffer::ih((nis + l).am(0) as u32, ekn.am(0) as u32, 1, azb, 0x002A15, 40);
    }

    
    let ab = frame as f32 * 0.03;
    let fuu = libm::st(ab);
    let ffx = libm::zq(ab);
    let e = 40.0f32;
    let nhx: [(f32, f32, f32); 8] = [
        (-e, -e, -e), (e, -e, -e), (e, e, -e), (-e, e, -e),
        (-e, -e,  e), (e, -e,  e), (e, e,  e), (-e, e,  e),
    ];
    let bu: [(usize, usize); 12] = [
        (0,1),(1,2),(2,3),(3,0), (4,5),(5,6),(6,7),(7,4),
        (0,4),(1,5),(2,6),(3,7),
    ];
    let nv = |ai: (f32, f32, f32)| -> (i32, i32) {
        let kb = ai.0 * ffx - ai.2 * fuu;
        let agv = ai.0 * fuu + ai.2 * ffx;
        let ix = ai.1 * libm::zq(ab * 0.7) - agv * libm::st(ab * 0.7);
        (nis + kb as i32, niu + ix as i32)
    };
    for &(q, o) in &bu {
        let (dn, dp) = nv(nhx[q]);
        let (hy, jz) = nv(nhx[o]);
        ahj(dn, dp, hy, jz, AG_);
    }

    
    let edf = ekn + azb as i32 - 20;
    cb(fg, edf, "Vertices: 8  Faces: 6  Edges: 12", AT_);
}


fn sbn(fp: i32, ae: i32, gm: u32, bm: u32, g: &MobileState) {
    let ov = 12i32;
    let fg = fp + ov;
    let adc = (gm as i32 - ov * 2) as u32;

    let bar = [
        ("WiFi", "Wireless connection", 0xFF40CC80),
        ("Bluetooth", "Paired devices", 0xFF4488DD),
        ("Airplane", "Radio off", 0xFFCC8844),
        ("Do Not Disturb", "Silence alerts", 0xFFFF6090),
        ("Dark Mode", "Display theme", 0xFF9988BB),
        ("Notifications", "Push alerts", 0xFF40AADD),
    ];

    let ph = 52u32;
    let mut ix = ae + 10;
    for (a, &(dq, desc, mm)) in bar.iter().cf() {
        if ix + ph as i32 > ae + bm as i32 { break; }
        let qe = g.mey == a as i32;
        let ei = if qe { 0x0A1A12 } else { 0x080C0A };
        framebuffer::ih(fg.am(0) as u32, ix.am(0) as u32, adc, ph, ei, 180);
        framebuffer::ah((fg + 4).am(0) as u32, (ix + ph as i32 - 1).am(0) as u32, adc - 8, 1, AJ_);
        
        framebuffer::ah((fg + 8).am(0) as u32, (ix + 20).am(0) as u32, 4, 4, mm);
        cb(fg + 20, ix + 10, dq, AC_);
        cb(fg + 20, ix + 28, desc, N_);
        
        let mlp = a < g.gsk.len() && g.gsk[a];
        let mls = fp + gm as i32 - ov - 44;
        let xis = if mlp { 0xFF008844 } else { 0xFF333833 };
        mf(mls, ix + 14, 40, 22, 11, xis);
        
        let etq = if mlp { mls + 20 } else { mls + 2 };
        mf(etq, ix + 16, 18, 18, 9, if mlp { I_ } else { AT_ });
        ix += ph as i32;
    }
}


fn sbd(fp: i32, ae: i32, gm: u32, bm: u32, eln: u64) {
    let ov = 14i32;
    let fg = fp + ov;

    let yv = fp + gm as i32 / 2;
    let mut ct = ae + 20;

    
    np(yv, ct, "TrustOS", I_);
    ct += 24;
    np(yv, ct, "v2.0.0", AG_);
    ct += 30;

    
    framebuffer::ah((fg + 20).am(0) as u32, ct.am(0) as u32, gm.ao(68), 1, AJ_);
    ct += 16;

    let co = [
        ("Kernel:", "TrustOS Microkernel"),
        ("Arch:", "x86_64 / aarch64"),
        ("License:", "MIT"),
        ("Desktop:", "TrustOS Desktop v2"),
        ("Browser:", "TrustBrowser v1.0"),
        ("Audio:", "HD Audio + DMA"),
        ("Graphics:", "COSMIC Renderer"),
        ("Uptime:", "4h 23m"),
    ];

    for &(cu, bn) in &co {
        if ct + 20 > ae + bm as i32 { break; }
        cb(fg, ct, cu, N_);
        let mqc = crate::graphics::scaling::clj(bn) as i32;
        cb(fp + gm as i32 - ov - mqc, ct, bn, AC_);
        ct += 22;
    }
}


fn sbl(fp: i32, ae: i32, gm: u32, bm: u32, frame: u64, audio: &Adc) {
    let ov = 14i32;
    let fg = fp + ov;
    let adc = (gm as i32 - ov * 2) as u32;

    
    let byh = adc.v(200);
    let gyz = fp + (gm as i32 - byh as i32) / 2;
    let mut ct = ae + 14;
    mf(gyz, ct, byh, byh, 14, 0xFF0A1A10);
    tf(gyz, ct, byh, byh, 14, AJ_);

    
    if audio.uu {
        let cgd = gyz + byh as i32 / 2;
        let bkl = ct + byh as i32 / 2;
        let lne = 16u32;
        let lo = byh / (lne * 2);
        for a in 0..lne {
            let ab = a as f32 / lne as f32;
            let ib = frame as f32 * 0.08 + ab * 6.28;
            let byf = (libm::st(ib) * audio.abo + audio.aee * 0.5).am(0.1).v(1.0);
            let i = (byf * (byh as f32 * 0.4)) as u32;
            let bx = gyz + 10 + (a * (lo * 2)) as i32;
            let je = ct + byh as i32 / 2 + (byh as i32 / 4 - i as i32).am(0);
            let at = (128.0 + byf * 127.0).v(255.0) as u32;
            framebuffer::ah(bx.am(0) as u32, je.am(0) as u32, lo, i, 0xFF000000 | (at << 8) | 0x40);
        }
    } else {
        np(gyz + byh as i32 / 2, ct + byh as i32 / 2 - 6, "No Track", N_);
    }
    ct += byh as i32 + 16;

    
    let dq = if audio.uu { "Untitled (2) - Lo-Fi" } else { "No Track Playing" };
    np(fp + gm as i32 / 2, ct, dq, AC_);
    ct += 20;
    np(fp + gm as i32 / 2, ct, "TrustOS Audio", N_);
    ct += 28;

    
    let lo = adc - 20;
    framebuffer::ah((fg + 10).am(0) as u32, ct.am(0) as u32, lo, 4, AJ_);
    if audio.uu {
        let li = (frame % 300) as u32 * lo / 300;
        framebuffer::ah((fg + 10).am(0) as u32, ct.am(0) as u32, li, 4, I_);
    }
    ct += 20;

    
    let rrh = ["|<", "<<", if audio.uu { "||" } else { ">" }, ">>", ">|"];
    let pm = 48u32;
    let qx = 36u32;
    let aib = 10u32;
    let aza = 5 * pm + 4 * aib;
    let kfe = fg + (adc as i32 - aza as i32) / 2;
    for (a, &cu) in rrh.iter().cf() {
        let bx = kfe + (a as u32 * (pm + aib)) as i32;
        let hpb = a == 2;
        let ei = if hpb { if audio.uu { 0xFF005533 } else { 0xFF003322 } } else { 0xFF1A2A1A };
        mf(bx, ct, pm, qx, 10, ei);
        let hpv = if hpb { I_ } else { EC_ };
        np(bx + pm as i32 / 2, ct + 10, cu, hpv);
    }
    ct += qx as i32 + 16;

    
    let qmn = ["Sub", "Bass", "Mid", "Treble"];
    let qmq = [audio.ato, audio.aee, audio.vs, audio.axg];
    let qmm: [u32; 4] = [0xFF00FF44, 0xFF00CC88, 0xFF00AACC, 0xFF8866FF];
    let ikl = (adc - 12) / 4;
    for (a, (&j, &ap)) in qmn.iter().fca(qmq.iter()).cf() {
        let bx = fg + (a as u32 * (ikl + 4)) as i32;
        framebuffer::ih(bx.am(0) as u32, ct.am(0) as u32, ikl, 10, 0x112211, 150);
        let vi = if audio.uu { (ap.v(1.0) * ikl as f32) as u32 } else { 0 };
        if vi > 0 {
            framebuffer::ah(bx.am(0) as u32, ct.am(0) as u32, vi, 10, qmm[a]);
        }
        np(bx + ikl as i32 / 2, ct + 12, j, AT_);
    }
}


fn sbh(fp: i32, ae: i32, gm: u32, bm: u32, g: &MobileState) {
    let ov = 8i32;

    
    let aly = (gm as i32 - ov * 2).v(bm as i32 - 60).v(400);
    let cell = aly / 8;
    let aoj = fp + (gm as i32 - aly) / 2;
    let apl = ae + 10;

    let wgv = if g.cpd >= 0 { g.cpd / 8 } else { -1 };
    let wgu = if g.cpd >= 0 { g.cpd % 8 } else { -1 };

    
    for br in 0..8u32 {
        for bj in 0..8u32 {
            let dio = (br + bj) % 2 == 0;
            let qe = br as i32 == wgv && bj as i32 == wgu;
            let s = if qe { 0xFF2A5A2A }
                       else if dio { 0xFF2A3A2A }
                       else { 0xFF0A140A };
            let cx = aoj + (bj * cell as u32) as i32;
            let ix = apl + (br * cell as u32) as i32;
            framebuffer::ah(cx.am(0) as u32, ix.am(0) as u32, cell as u32, cell as u32, s);
            
            if qe {
                
                framebuffer::ah(cx.am(0) as u32, ix.am(0) as u32, cell as u32, 2, I_);
                framebuffer::ah(cx.am(0) as u32, (ix + cell - 2).am(0) as u32, cell as u32, 2, I_);
                
                framebuffer::ah(cx.am(0) as u32, ix.am(0) as u32, 2, cell as u32, I_);
                framebuffer::ah((cx + cell - 2).am(0) as u32, ix.am(0) as u32, 2, cell as u32, I_);
            }
        }
    }

    
    tf(aoj - 1, apl - 1, aly as u32 + 2, aly as u32 + 2, 2, AT_);

    
    let ovq = ["R", "N", "B", "Q", "K", "B", "N", "R"];
    for bj in 0..8 {
        let cx = aoj + bj * cell + cell / 2;
        
        np(cx, apl + 7 * cell + cell / 2 - 6, ovq[bj as usize], 0xFFDDDDDD);
        np(cx, apl + 6 * cell + cell / 2 - 6, "P", 0xFFDDDDDD);
        
        np(cx, apl + cell / 2 - 6, ovq[bj as usize], 0xFFD4A854);
        np(cx, apl + cell + cell / 2 - 6, "P", 0xFFD4A854);
    }

    
    let uo = apl + aly + 8;
    let gvd = if g.hct == 0 { "White to move" } else { "Black to move" };
    let ifg = if g.hct == 0 { 0xFFDDDDDD } else { 0xFFD4A854 };
    np(fp + gm as i32 / 2, uo, gvd, ifg);

    if g.cpd >= 0 {
        let wrn = alloc::format!("Selected: {}{}", 
            (b'a' + (g.cpd % 8) as u8) as char,
            8 - g.cpd / 8);
        np(fp + gm as i32 / 2, uo + 18, &wrn, AG_);
    }
}





fn ahj(fy: i32, fo: i32, dn: i32, dp: i32, s: u32) {
    let dx = (dn - fy).gp();
    let bg = -(dp - fo).gp();
    let cr = if fy < dn { 1 } else { -1 };
    let cq = if fo < dp { 1 } else { -1 };
    let mut rq = dx + bg;
    let mut cx = fy;
    let mut ae = fo;
    loop {
        ayl(cx, ae, s);
        if cx == dn && ae == dp { break; }
        let agl = 2 * rq;
        if agl >= bg {
            if cx == dn { break; }
            rq += bg;
            cx += cr;
        }
        if agl <= dx {
            if ae == dp { break; }
            rq += dx;
            ae += cq;
        }
    }
}
