






























pub const BI_: usize = 128;
pub const GR_: usize = 256;
pub const AGB_: usize = 6;


const NC_: usize = 96;
const ACU_: usize = 54;
const TK_: usize = 5184; 


const BZM_: f32 = 300.0;   
const AEL_: f32 = 0.04;     





pub struct DroneSwarmState {
    pub jr: bool,
    wf: f32,
    aav: f32,

    
    gfl: [f32; BI_],
    gfm: [f32; BI_],
    gfn: [f32; BI_],

    
    ayw: [f32; BI_],
    ayx: [f32; BI_],
    cuo: [f32; BI_],

    
    frh: [f32; BI_],
    fri: [f32; BI_],
    hwb: [f32; BI_],

    dka: usize,

    
    bzi: [u8; GR_],
    bzj: [u8; GR_],
    evm: usize,

    
    jnr: f32,
    jnq: f32,

    
    ghl: u8,
    jrr: f32,

    
    jtz: f32,

    
    tq: [f32; TK_],

    
    acc: f32,
    aqw: f32,

    frame: u64,
}

impl DroneSwarmState {
    pub const fn new() -> Self {
        Self {
            jr: false,
            wf: 0.0,
            aav: 0.0,
            gfl: [0.0; BI_],
            gfm: [0.0; BI_],
            gfn: [0.0; BI_],
            ayw: [0.0; BI_],
            ayx: [0.0; BI_],
            cuo: [0.0; BI_],
            frh: [0.0; BI_],
            fri: [0.0; BI_],
            hwb: [0.0; BI_],
            dka: 0,
            bzi: [0; GR_],
            bzj: [0; GR_],
            evm: 0,
            jnr: 0.0,
            jnq: 0.0,
            ghl: 0,
            jrr: 0.0,
            jtz: 0.0,
            tq: [0.0; TK_],
            acc: 20.0,
            aqw: 20.0,
            frame: 0,
        }
    }
}





pub fn init(e: &mut DroneSwarmState, d: u32, i: u32) {
    e.wf = d as f32;
    e.aav = i as f32;
    e.acc = e.wf / NC_ as f32;
    e.aqw = e.aav / ACU_ as f32;

    
    e.ghl = 0;
    nan(e, 0);

    
    for a in 0..e.dka {
        e.gfl[a] = e.ayw[a];
        e.gfm[a] = e.ayx[a];
        e.gfn[a] = e.cuo[a];
    }

    e.jr = true;
}





fn nan(e: &mut DroneSwarmState, w: u8) {
    
    for a in 0..BI_ {
        e.ayw[a] = 0.0;
        e.ayx[a] = 0.0;
        e.cuo[a] = 0.0;
    }
    for a in 0..GR_ {
        e.bzi[a] = 0;
        e.bzj[a] = 0;
    }
    match w % AGB_ as u8 {
        0 => qth(e),
        1 => quc(e),
        2 => qsz(e),
        3 => qtg(e),
        4 => qtc(e),
        _ => qtj(e),
    }
}


fn qth(e: &mut DroneSwarmState) {
    const FU_: usize = 32;
    let mut ne = 0usize;

    let cuy = 2.5f32;
    let dy = 0.35f32;
    let ac = 1.6f32;

    for a in 0..FU_ {
        let ab = a as f32 / (FU_ - 1) as f32;
        let hg = ab * cuy * 6.2831853;
        let c = -0.8 + ab * ac;

        
        e.ayw[a] = dy * libm::zq(hg);
        e.ayx[a] = c;
        e.cuo[a] = dy * libm::st(hg);

        
        e.ayw[FU_ + a] = dy * libm::zq(hg + 3.1415927);
        e.ayx[FU_ + a] = c;
        e.cuo[FU_ + a] = dy * libm::st(hg + 3.1415927);

        
        if a > 0 {
            e.bzi[ne] = (a - 1) as u8;
            e.bzj[ne] = a as u8;
            ne += 1;
            e.bzi[ne] = (FU_ + a - 1) as u8;
            e.bzj[ne] = (FU_ + a) as u8;
            ne += 1;
        }
        
        if a % 4 == 0 {
            e.bzi[ne] = a as u8;
            e.bzj[ne] = (FU_ + a) as u8;
            ne += 1;
        }
    }

    e.dka = FU_ * 2;
    e.evm = ne;
}


fn quc(e: &mut DroneSwarmState) {
    let bv = 0.55f32;

    for fs in 0u8..16 {
        let elf = if fs & 1 != 0 { 1.0f32 } else { -1.0 };
        let xwv = if fs & 2 != 0 { 1.0f32 } else { -1.0 };
        let xxb = if fs & 4 != 0 { 1.0f32 } else { -1.0 };
        let xth = if fs & 8 != 0 { 1.0f32 } else { -1.0 };

        let gel = xth * 0.4 + 1.8;
        e.ayw[fs as usize] = (elf / gel) * bv;
        e.ayx[fs as usize] = (xwv / gel) * bv;
        e.cuo[fs as usize] = (xxb / gel) * bv;
    }
    e.dka = 16;

    
    let mut ne = 0usize;
    for a in 0u8..16 {
        for ga in 0u8..4 {
            let fb = a ^ (1 << ga);
            if fb > a {
                e.bzi[ne] = a;
                e.bzj[ne] = fb;
                ne += 1;
            }
        }
    }
    e.evm = ne;
}


fn qsz(e: &mut DroneSwarmState) {
    const BEN_: usize = 32;
    let mut amk = 0usize;
    let mut ne = 0usize;

    let pdj = 0.7f32;

    for mz in 0u8..3 {
        let ptf = match mz {
            0 => 0.0f32,
            1 => 1.047f32,  
            _ => -1.047f32,  
        };

        let ar = amk;
        let ffx = libm::zq(ptf);
        let fuu = libm::st(ptf);

        for a in 0..BEN_ {
            let q = (a as f32 / BEN_ as f32) * 6.2831853;
            let b = pdj * libm::zq(q);
            let c = pdj * libm::st(q);

            
            e.ayw[amk] = b;
            e.ayx[amk] = c * ffx;
            e.cuo[amk] = c * fuu;

            
            if a > 0 {
                e.bzi[ne] = (amk - 1) as u8;
                e.bzj[ne] = amk as u8;
                ne += 1;
            }
            amk += 1;
        }
        
        e.bzi[ne] = (amk - 1) as u8;
        e.bzj[ne] = ar as u8;
        ne += 1;
    }

    
    let jhm = 0.08f32;
    let ork = amk;
    let uwc: [(f32, f32, f32); 4] = [
        (jhm, 0.0, 0.0),
        (-jhm, 0.0, 0.0),
        (0.0, jhm, 0.0),
        (0.0, -jhm, 0.0),
    ];
    for &(vt, ahr, arn) in uwc.iter() {
        e.ayw[amk] = vt;
        e.ayx[amk] = ahr;
        e.cuo[amk] = arn;
        amk += 1;
    }
    
    for a in 0u8..4 {
        for fb in (a + 1)..4 {
            e.bzi[ne] = (ork as u8) + a;
            e.bzj[ne] = (ork as u8) + fb;
            ne += 1;
        }
    }

    e.dka = amk;
    e.evm = ne;
}


fn qtg(e: &mut DroneSwarmState) {
    const AKU_: usize = 24;
    const BBS_: usize = 4;
    let mut amk = 0usize;
    let mut ne = 0usize;

    let bdm = 0.1f32;
    let djl = 0.8f32;

    for ccx in 0..BBS_ {
        let l = (ccx as f32 / BBS_ as f32) * 6.2831853;

        for a in 0..AKU_ {
            let ab = a as f32 / (AKU_ - 1) as f32;
            let m = bdm + ab * (djl - bdm);
            let bdb = l + ab * 3.0;
            let xxi = libm::st(ab * 4.0) * 0.1;

            e.ayw[amk] = m * libm::zq(bdb);
            e.ayx[amk] = xxi;
            e.cuo[amk] = m * libm::st(bdb);

            if a > 0 {
                e.bzi[ne] = (amk - 1) as u8;
                e.bzj[ne] = amk as u8;
                ne += 1;
            }
            amk += 1;
        }
    }

    
    let ngd = amk;
    for a in 0..8usize {
        let q = (a as f32 / 8.0) * 6.2831853;
        e.ayw[amk] = 0.08 * libm::zq(q);
        e.ayx[amk] = 0.08 * libm::st(q * 1.5);
        e.cuo[amk] = 0.08 * libm::st(q);
        amk += 1;
    }
    for a in 0u8..8 {
        e.bzi[ne] = (ngd as u8) + a;
        e.bzj[ne] = (ngd as u8) + ((a + 1) % 8);
        ne += 1;
    }

    e.dka = amk;
    e.evm = ne;
}


fn qtc(e: &mut DroneSwarmState) {
    const Qc: usize = 4;
    const Xs: usize = 4;
    const Akk: usize = 3;
    let mut amk = 0usize;
    let mut ne = 0usize;

    let aoa = 0.45f32;
    let lpw = -((Qc - 1) as f32) * aoa * 0.5;
    let uxg = -((Xs - 1) as f32) * aoa * 0.5;
    let uxh = -((Akk - 1) as f32) * aoa * 0.5;

    for jbx in 0..Akk {
        for og in 0..Xs {
            for fg in 0..Qc {
                e.ayw[amk] = lpw + fg as f32 * aoa;
                e.ayx[amk] = uxg + og as f32 * aoa;
                e.cuo[amk] = uxh + jbx as f32 * aoa;
                amk += 1;
            }
        }
    }

    
    for jbx in 0..Akk {
        for og in 0..Xs {
            for fg in 0..Qc {
                let w = jbx * Xs * Qc + og * Qc + fg;
                if fg + 1 < Qc && ne < GR_ {
                    e.bzi[ne] = w as u8;
                    e.bzj[ne] = (w + 1) as u8;
                    ne += 1;
                }
                if og + 1 < Xs && ne < GR_ {
                    e.bzi[ne] = w as u8;
                    e.bzj[ne] = (w + Qc) as u8;
                    ne += 1;
                }
                if jbx + 1 < Akk && ne < GR_ {
                    e.bzi[ne] = w as u8;
                    e.bzj[ne] = (w + Xs * Qc) as u8;
                    ne += 1;
                }
            }
        }
    }

    e.dka = amk;
    e.evm = ne;
}


fn qtj(e: &mut DroneSwarmState) {
    const Adj: usize = 64;
    let m = 0.7f32;

    for a in 0..Adj {
        let ab = (a as f32 / Adj as f32) * 6.2831853;
        
        e.ayw[a] = m * libm::st(ab);
        e.ayx[a] = m * libm::st(ab) * libm::zq(ab);
        e.cuo[a] = m * libm::st(2.0 * ab) * 0.3;
    }

    let mut ne = 0usize;
    for a in 0..Adj {
        e.bzi[ne] = a as u8;
        e.bzj[ne] = ((a + 1) % Adj) as u8;
        ne += 1;
    }

    e.dka = Adj;
    e.evm = ne;
}





pub fn qs(e: &mut DroneSwarmState) {
    if !e.jr {
        return;
    }
    e.frame += 1;
    e.jtz += 1.0;
    e.jrr += 1.0;

    
    e.jnr += 0.006; 
    e.jnq = libm::st(e.jtz * 0.0015) * 0.35; 

    
    if e.jrr >= BZM_ {
        let uxr = e.dka;
        e.ghl = (e.ghl + 1) % AGB_ as u8;
        nan(e, e.ghl);
        
        for a in uxr..e.dka {
            e.gfl[a] = 0.0;
            e.gfm[a] = 0.0;
            e.gfn[a] = 0.0;
        }
        e.jrr = 0.0;
    }

    
    for a in 0..BI_ {
        e.gfl[a] += (e.ayw[a] - e.gfl[a]) * AEL_;
        e.gfm[a] += (e.ayx[a] - e.gfm[a]) * AEL_;
        e.gfn[a] += (e.cuo[a] - e.gfn[a]) * AEL_;
    }

    
    vnd(e);

    
    vvw(e);
}





fn vnd(e: &mut DroneSwarmState) {
    let bmo = libm::zq(e.jnr);
    let bol = libm::st(e.jnr);
    let bmn = libm::zq(e.jnq);
    let bok = libm::st(e.jnq);

    let cx = e.wf * 0.5;
    let ae = e.aav * 0.45;
    let bv = e.wf * 0.28; 
    let aab = 3.5f32;

    for a in 0..BI_ {
        let b = e.gfl[a];
        let c = e.gfm[a];
        let av = e.gfn[a];

        
        let hy = b * bmo + av * bol;
        let ahc = -b * bol + av * bmo;

        
        let jz = c * bmn - ahc * bok;
        let eli = c * bok + ahc * bmn;

        
        let d = (aab + eli).am(0.15);

        e.frh[a] = cx + (hy / d) * bv;
        e.fri[a] = ae + (jz / d) * bv;
        e.hwb[a] = d;
    }
}






#[inline]
fn mtw(tq: &mut [f32; TK_], b: f32, c: f32, aaj: f32, dy: i32) {
    let fg = b as i32;
    let og = c as i32;
    let oyz = dy as f32 + 0.5;

    let mut bg = -dy;
    while bg <= dy {
        let mut dx = -dy;
        while dx <= dy {
            let cx = fg + dx;
            let ae = og + bg;
            if cx >= 0 && (cx as usize) < NC_ && ae >= 0 && (ae as usize) < ACU_ {
                let jf = b - cx as f32 - 0.5;
                let sc = c - ae as f32 - 0.5;
                let la = libm::bon(jf * jf + sc * sc);
                if la < oyz {
                    let ckj = 1.0 - la / oyz;
                    let ckj = ckj * ckj; 
                    let w = ae as usize * NC_ + cx as usize;
                    tq[w] += aaj * ckj;
                    if tq[w] > 5.0 {
                        tq[w] = 5.0;
                    }
                }
            }
            dx += 1;
        }
        bg += 1;
    }
}

fn vvw(e: &mut DroneSwarmState) {
    
    for a in 0..TK_ {
        e.tq[a] = 0.0;
    }

    let dt = e.acc;
    let bm = e.aqw;

    
    for aa in 0..e.evm {
        let q = e.bzi[aa] as usize;
        let o = e.bzj[aa] as usize;

        let fy = e.frh[q] / dt;
        let fo = e.fri[q] / bm;
        let dn = e.frh[o] / dt;
        let dp = e.fri[o] / bm;

        
        let qlx = (e.hwb[q] + e.hwb[o]) * 0.5;
        let kpe = (1.8 / qlx).v(1.5);
        let sis = kpe * 0.7;

        
        let dx = dn - fy;
        let bg = dp - fo;
        let len = libm::bon(dx * dx + bg * bg);
        let au = ((len * 1.5) as usize).am(1).v(300);

        for gu in 0..=au {
            let ab = gu as f32 / au as f32;
            let b = fy + dx * ab;
            let c = fo + bg * ab;
            mtw(&mut e.tq, b, c, sis, 1);
        }
    }

    
    for a in 0..e.dka {
        let qz = e.frh[a] / dt;
        let ub = e.fri[a] / bm;
        let kpe = (2.5 / e.hwb[a]).v(3.0);
        mtw(&mut e.tq, qz, ub, kpe, 2);
    }
}






pub struct DroneInteraction {
    pub kt: f32,
    pub cpl: i16,
    pub cwj: i16,
    pub cwi: i16,
}

impl DroneInteraction {
    pub const Cq: Self = Self {
        kt: 1.0,
        cpl: 0,
        cwj: 0,
        cwi: 0,
    };
}



pub fn query(e: &DroneSwarmState, y: f32, x: f32) -> DroneInteraction {
    if !e.jr {
        return DroneInteraction::Cq;
    }

    let qz = (y / e.acc) as usize;
    let ub = (x / e.aqw) as usize;
    if qz >= NC_ || ub >= ACU_ {
        return DroneInteraction::Cq;
    }

    let ap = e.tq[ub * NC_ + qz];
    if ap < 0.03 {
        return DroneInteraction::Cq;
    }

    
    let (btu, bmh, aiv) = ram(e.jtz, e.ghl);

    
    let kt = if ap > 2.0 {
        
        1.4 + (ap - 2.0).v(2.0) * 0.4
    } else if ap > 0.5 {
        
        1.1 + (ap - 0.5) * 0.25
    } else {
        
        1.0 + ap * 0.2
    };

    
    let kjy = (ap / 2.5).v(1.0);

    DroneInteraction {
        kt,
        cpl: (btu as f32 * kjy) as i16,
        cwj: (bmh as f32 * kjy) as i16,
        cwi: (aiv as f32 * kjy) as i16,
    }
}









fn ram(time: f32, swe: u8) -> (i16, i16, i16) {
    
    let (avi, ei, aaa) = match swe % AGB_ as u8 {
        0 => (0i16, 60, 80),      
        1 => (20i16, 30, 90),      
        2 => (70i16, 50, -10),     
        3 => (50i16, -5, 70),      
        4 => (-10i16, 40, 70),     
        _ => (60i16, 20, 50),      
    };

    
    let jpx = libm::st(time * 0.018) * 0.2 + 1.0;
    
    let wmx = libm::st(time * 0.011 + 1.5) * 0.1 + 1.0;

    let m = (avi as f32 * jpx) as i16;
    let at = (ei as f32 * wmx) as i16;
    let o = (aaa as f32 * jpx) as i16;

    (m, at, o)
}
