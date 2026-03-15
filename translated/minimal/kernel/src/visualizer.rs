


















extern crate alloc;
use alloc::vec::Vec;





#[derive(Clone, Copy)]
struct V3 { b: f32, c: f32, av: f32 }

impl V3 {
    const fn new(b: f32, c: f32, av: f32) -> Self { Self { b, c, av } }
}

#[derive(Clone, Copy)]
struct D(u16, u16);


#[derive(Clone, Copy)]
pub struct RainEffect {
    pub tq: u8,
    pub eo: u8,
    pub bst: u8,
    pub apb: u8,
    pub tp: u8,
    pub aug: u8,
    pub amv: u8,
    pub atv: u8,
    pub axo: u8,
    pub ys: u8,
    pub ata: u8,
    pub zc: u8,
    
    pub ejq: u8,
    pub ejn: u8,
    pub ejl: u8,
    
    pub dcm: u8,
}

impl RainEffect {
    pub const Cq: Self = Self {
        tq: 0, eo: 128, bst: 0, apb: 0, tp: 0,
        aug: 0, amv: 0, atv: 0, axo: 0, ys: 0,
        ata: 0, zc: 0,
        ejq: 0, ejn: 0, ejl: 0, dcm: 0,
    };
}





pub const IR_: u8 = 14;

pub const OG_: [&str; 14] = [
    "Sphere",
    "Morphing",
    "Lorenz",
    "Spectrum",
    "Ribbon",
    "Starburst",
    "Image",
    "TorusKnot",
    "DNA Helix",
    "Tesseract",
    "Vortex",
    "PlasmaSphere",
    "Galaxy",
    "Subscribe",
];





pub const AGE_: u8 = 24;

pub const CIM_: [&str; 24] = [
    
    "Matrix",     
    "Cyber",      
    "Fire",       
    "Ocean",      
    "Aurora",     
    "Gold",       
    "Red",        
    "Blue",       
    "Purple",     
    "Pink",       
    "Yellow",     
    "Cyan",       
    "White",      
    
    "Rainbow",    
    "Neon Mix",   
    "Lava Sea",   
    "Prism",      
    "Sunset",     
    "Arctic",     
    "Toxic",      
    "Vampire",    
    "Nebula",     
    "Inferno",    
    
    "Random",     
];



pub fn clr(aim: u8, ab: f32) -> (f32, f32, f32) {
    let ab = ab.am(0.0).v(1.0);
    match aim {
        0 => { 
            let m = 20.0 + ab * 80.0;
            let at = 120.0 + ab * 135.0;
            let o = 20.0 + ab * 60.0;
            (m, at, o)
        }
        1 => { 
            if ab < 0.33 {
                let e = ab / 0.33;
                (20.0 + e * 60.0, 180.0 + e * 75.0, 255.0) 
            } else if ab < 0.66 {
                let e = (ab - 0.33) / 0.33;
                (80.0 + e * 175.0, 255.0 - e * 175.0, 255.0 - e * 55.0) 
            } else {
                let e = (ab - 0.66) / 0.34;
                (255.0, 80.0 - e * 40.0, 200.0 + e * 55.0) 
            }
        }
        2 => { 
            if ab < 0.4 {
                let e = ab / 0.4;
                (180.0 + e * 75.0, 20.0 + e * 60.0, 0.0) 
            } else if ab < 0.75 {
                let e = (ab - 0.4) / 0.35;
                (255.0, 80.0 + e * 120.0, e * 30.0) 
            } else {
                let e = (ab - 0.75) / 0.25;
                (255.0, 200.0 + e * 55.0, 30.0 + e * 200.0) 
            }
        }
        3 => { 
            if ab < 0.5 {
                let e = ab / 0.5;
                (0.0, 30.0 + e * 100.0, 120.0 + e * 135.0) 
            } else {
                let e = (ab - 0.5) / 0.5;
                (e * 80.0, 130.0 + e * 125.0, 255.0) 
            }
        }
        4 => { 
            if ab < 0.33 {
                let e = ab / 0.33;
                (30.0 + e * 40.0, 200.0 + e * 55.0, 80.0 + e * 40.0) 
            } else if ab < 0.66 {
                let e = (ab - 0.33) / 0.33;
                (70.0 + e * 100.0, 255.0 - e * 155.0, 120.0 + e * 80.0) 
            } else {
                let e = (ab - 0.66) / 0.34;
                (170.0 + e * 85.0, 100.0 + e * 50.0, 200.0 + e * 55.0) 
            }
        }
        5 => { 
            if ab < 0.5 {
                let e = ab / 0.5;
                (180.0 + e * 75.0, 130.0 + e * 50.0, 10.0 + e * 20.0)
            } else {
                let e = (ab - 0.5) / 0.5;
                (255.0, 180.0 + e * 75.0, 30.0 + e * 225.0)
            }
        }
        6 => { 
            if ab < 0.4 {
                let e = ab / 0.4;
                (100.0 + e * 100.0, 5.0 + e * 15.0, 5.0 + e * 10.0)
            } else if ab < 0.75 {
                let e = (ab - 0.4) / 0.35;
                (200.0 + e * 55.0, 20.0 + e * 40.0, 15.0 + e * 30.0)
            } else {
                let e = (ab - 0.75) / 0.25;
                (255.0, 60.0 + e * 160.0, 45.0 + e * 180.0)
            }
        }
        7 => { 
            if ab < 0.4 {
                let e = ab / 0.4;
                (5.0 + e * 15.0, 10.0 + e * 40.0, 100.0 + e * 100.0)
            } else if ab < 0.75 {
                let e = (ab - 0.4) / 0.35;
                (20.0 + e * 40.0, 50.0 + e * 80.0, 200.0 + e * 55.0)
            } else {
                let e = (ab - 0.75) / 0.25;
                (60.0 + e * 160.0, 130.0 + e * 125.0, 255.0)
            }
        }
        8 => { 
            if ab < 0.4 {
                let e = ab / 0.4;
                (60.0 + e * 50.0, 5.0 + e * 15.0, 100.0 + e * 60.0)
            } else if ab < 0.75 {
                let e = (ab - 0.4) / 0.35;
                (110.0 + e * 60.0, 20.0 + e * 40.0, 160.0 + e * 60.0)
            } else {
                let e = (ab - 0.75) / 0.25;
                (170.0 + e * 70.0, 60.0 + e * 140.0, 220.0 + e * 35.0)
            }
        }
        9 => { 
            if ab < 0.4 {
                let e = ab / 0.4;
                (140.0 + e * 60.0, 10.0 + e * 20.0, 80.0 + e * 40.0)
            } else if ab < 0.75 {
                let e = (ab - 0.4) / 0.35;
                (200.0 + e * 55.0, 30.0 + e * 50.0, 120.0 + e * 40.0)
            } else {
                let e = (ab - 0.75) / 0.25;
                (255.0, 80.0 + e * 130.0, 160.0 + e * 80.0)
            }
        }
        10 => { 
            if ab < 0.4 {
                let e = ab / 0.4;
                (140.0 + e * 60.0, 100.0 + e * 50.0, 5.0 + e * 10.0)
            } else if ab < 0.75 {
                let e = (ab - 0.4) / 0.35;
                (200.0 + e * 55.0, 150.0 + e * 75.0, 15.0 + e * 20.0)
            } else {
                let e = (ab - 0.75) / 0.25;
                (255.0, 225.0 + e * 30.0, 35.0 + e * 200.0)
            }
        }
        11 => { 
            if ab < 0.4 {
                let e = ab / 0.4;
                (5.0 + e * 10.0, 80.0 + e * 70.0, 100.0 + e * 60.0)
            } else if ab < 0.75 {
                let e = (ab - 0.4) / 0.35;
                (15.0 + e * 30.0, 150.0 + e * 80.0, 160.0 + e * 95.0)
            } else {
                let e = (ab - 0.75) / 0.25;
                (45.0 + e * 180.0, 230.0 + e * 25.0, 255.0)
            }
        }
        _ => { 
            if ab < 0.4 {
                let e = ab / 0.4;
                let p = 60.0 + e * 60.0;
                (p, p, p)
            } else if ab < 0.75 {
                let e = (ab - 0.4) / 0.35;
                let p = 120.0 + e * 80.0;
                (p, p + e * 10.0, p + e * 15.0) 
            } else {
                let e = (ab - 0.75) / 0.25;
                let p = 200.0 + e * 55.0;
                (p, p, p)
            }
        }
    }
}



fn dzp(pr: u8, pf: u8, bnt: u8, ab: f32) -> (f32, f32, f32) {
    let ab = ab.am(0.0).v(1.0);
    if ab < 0.3 {
        
        clr(pr, ab / 0.3)
    } else if ab < 0.45 {
        
        let e = (ab - 0.3) / 0.15;
        let (aqh, cyd, of) = clr(pr, 0.8 + e * 0.2);
        let (uv, cqu, tb) = clr(pf, e * 0.2);
        (aqh * (1.0 - e) + uv * e, cyd * (1.0 - e) + cqu * e, of * (1.0 - e) + tb * e)
    } else if ab < 0.65 {
        
        let e = (ab - 0.45) / 0.2;
        clr(pf, e)
    } else if ab < 0.8 {
        
        let e = (ab - 0.65) / 0.15;
        let (aqh, cyd, of) = clr(pf, 0.8 + e * 0.2);
        let (uv, cqu, tb) = clr(bnt, e * 0.2);
        (aqh * (1.0 - e) + uv * e, cyd * (1.0 - e) + cqu * e, of * (1.0 - e) + tb * e)
    } else {
        
        let e = (ab - 0.8) / 0.2;
        clr(bnt, e)
    }
}


fn lwy(ab: f32) -> (f32, f32, f32) {
    let ab = ab.am(0.0).v(1.0);
    
    let i = ab * 6.0; 
    let a = i as u8;
    let bb = i - a as f32;
    match a {
        0 => (255.0, bb * 255.0, 0.0),                    
        1 => (255.0 * (1.0 - bb), 255.0, 0.0),            
        2 => (0.0, 255.0, bb * 255.0),                     
        3 => (0.0, 255.0 * (1.0 - bb), 255.0),            
        4 => (bb * 255.0, 0.0, 255.0),                     
        _ => (255.0, 0.0, 255.0 * (1.0 - bb)),            
    }
}


fn vqc(ab: f32) -> (f32, f32, f32) {
    
    let fs = ab.bsr();
    let hash = fs.hx(2654435761); 
    let aya = (hash % 360) as f32 / 360.0;
    lwy(aya)
}


pub fn vpv(bj: usize, br: usize, frame: u32) -> (u8, u8, u8) {
    let dv = (bj as u32).hx(2654435761)
        ^ (br as u32).hx(1103515245)
        ^ frame.hx(214013).cn(2531011);
    let aya = (dv % 360) as f32 / 360.0;
    let (m, at, o) = lwy(aya);
    (m.v(255.0) as u8, at.v(255.0) as u8, o.v(255.0) as u8)
}


pub fn ecc(aim: u8, ab: f32) -> (f32, f32, f32) {
    match aim {
        0..=12 => clr(aim, ab),
        13 => lwy(ab),                     
        14 => dzp(1, 2, 4, ab),            
        15 => dzp(2, 3, 5, ab),            
        16 => dzp(1, 4, 3, ab),            
        17 => dzp(6, 5, 8, ab),            
        18 => dzp(11, 7, 12, ab),          
        19 => dzp(0, 10, 11, ab),          
        20 => dzp(6, 8, 9, ab),            
        21 => dzp(7, 8, 9, ab),            
        22 => dzp(6, 2, 10, ab),           
        _  => vqc(ab),                      
    }
}





pub struct VisualizerState {
    
    pub ev: u8,

    
    by: Vec<V3>,           
    bu: Vec<D>,         
    bwh: Vec<(i32, i32)>,  
    cgu: Vec<i16>,         

    
    cbn: i32, boe: i32, dlj: i32,
    hxx: i32, hxy: i32, hxz: i32,
    bv: i32, eya: i32,
    yv: i32, uq: i32,

    
    pub btt: Vec<Vec<(i32, i32, u16, u8, i16)>>,
    pub anc: Vec<(i32, i32)>,

    
    pub elk: i16, pub elj: i16,
    pub oy: i32,
    pub frame: u64,
    pub bfi: f32,
    pub ays: f32,
    pub cib: f32,
    pub cbw: f32,
    pub ana: f32,
    pub bws: f32,
    pub eyt: i32, pub dcd: i32,
    pub bsj: i32, pub dlt: i32,
    pub dvw: i32,

    
    jqy: Vec<V3>,
    mgt: Vec<D>,
    mgs: Vec<u8>,

    
    eux: [Vec<V3>; 4],     
    gmw: [Vec<D>; 4],
    onz: Vec<V3>,
    gmx: f32,               
    hrv: usize,               

    
    glx: Vec<V3>,          
    jdy: [f32; 3],         

    
    jqx: Vec<V3>,
    mgo: Vec<u8>,
    mgp: Vec<D>,

    
    dvb: Vec<V3>,

    
    ewd: Vec<Bop>,
    lsz: u32,

    
    fla: i32,
    flb: i32,
    edd: u32,
    edc: u32,

    
    jtn: Vec<V3>,
    mlx: Vec<D>,

    
    irj: Vec<V3>,
    kqj: Vec<D>,

    
    xbq: Vec<V3>,   
    xbr: Vec<D>,
    mkg: f32,    

    
    mpx: Vec<V3>,
    mpy: Vec<D>,

    
    jjk: Vec<V3>,
    lue: Vec<D>,
    viq: Vec<V3>,
    vip: Vec<D>,

    
    nxc: Vec<V3>,
    nxd: Vec<D>,

    
    fhf: f32,           
    fhg: f32,           
    gfr: f32,          
    gfs: f32,          
    gfq: f32,       
    mib: Vec<V3>,
    mic: Vec<D>,

    
    pub aim: u8,

    pub jr: bool,
}

struct Bop {
    b: f32, c: f32, av: f32,
    fp: f32, iz: f32, ciq: f32,
    can: f32,
}

impl VisualizerState {
    pub const fn new() -> Self {
        Self {
            ev: 9,
            by: Vec::new(), bu: Vec::new(),
            bwh: Vec::new(), cgu: Vec::new(),
            cbn: 0, boe: 0, dlj: 0,
            hxx: 6, hxy: 10, hxz: 2,
            bv: 180, eya: 180,
            yv: 0, uq: 0,
            btt: Vec::new(), anc: Vec::new(),
            elk: 0, elj: 0, oy: 8,
            frame: 0,
            bfi: 0.0, ays: 0.0,
            cib: 0.0, cbw: 0.0,
            ana: 0.0, bws: 999.0,
            eyt: 0, dcd: 0,
            bsj: 0, dlt: 0, dvw: 0,
            jqy: Vec::new(), mgt: Vec::new(), mgs: Vec::new(),
            eux: [Vec::new(), Vec::new(), Vec::new(), Vec::new()],
            gmw: [Vec::new(), Vec::new(), Vec::new(), Vec::new()],
            onz: Vec::new(),
            gmx: 0.0, hrv: 0,
            glx: Vec::new(), jdy: [1.0, 1.0, 1.0],
            jqx: Vec::new(), mgo: Vec::new(), mgp: Vec::new(),
            dvb: Vec::new(),
            ewd: Vec::new(), lsz: 0,
            fla: 0, flb: 0,
            edd: 0, edc: 0,
            jtn: Vec::new(), mlx: Vec::new(),
            irj: Vec::new(), kqj: Vec::new(),
            xbq: Vec::new(), xbr: Vec::new(), mkg: 0.0,
            mpx: Vec::new(), mpy: Vec::new(),
            jjk: Vec::new(), lue: Vec::new(),
            viq: Vec::new(), vip: Vec::new(),
            nxc: Vec::new(), nxd: Vec::new(),
            fhf: 200.0, fhg: 150.0, gfr: 1.8, gfs: 1.2,
            gfq: 0.0,
            mib: Vec::new(), mic: Vec::new(),
            aim: 0,
            jr: false,
        }
    }
}





fn dlw(foq: i32) -> i32 {
    let ifh: i32 = 6283;
    let mut q = foq % ifh;
    if q < 0 { q += ifh; }
    let fqt: i32 = 3141;
    let (jyp, iaq) = if q > fqt { (q - fqt, -1i32) } else { (q, 1) };
    let ai = fqt as i64;
    let b = jyp as i64;
    let zp = b * (ai - b);
    let bzd = 5 * ai * ai - 4 * zp;
    if bzd == 0 { return 0; }
    (16 * zp * 1000 / bzd) as i32 * iaq
}

fn dzw(foq: i32) -> i32 { dlw(foq + 1571) }
fn st(m: f32) -> f32 { dlw((m * 1000.0) as i32) as f32 / 1000.0 }
fn zq(m: f32) -> f32 { dzw((m * 1000.0) as i32) as f32 / 1000.0 }

fn cni(p: V3, kb: i32, ix: i32, agv: i32, e: i32) -> (i32, i32, i32) {
    let fp = (p.b * e as f32) as i32;
    let iz = (p.c * e as f32) as i32;
    let ciq = (p.av * e as f32) as i32;
    let (cr, cx) = (dlw(kb), dzw(kb));
    let dp = (iz * cx - ciq * cr) / 1000;
    let aeu = (iz * cr + ciq * cx) / 1000;
    let (cq, ae) = (dlw(ix), dzw(ix));
    let hy = (fp * ae + aeu * cq) / 1000;
    let ahc = (-fp * cq + aeu * ae) / 1000;
    let (nf, zr) = (dlw(agv), dzw(agv));
    let ajr = (hy * zr - dp * nf) / 1000;
    let dnn = (hy * nf + dp * zr) / 1000;
    (ajr, dnn, ahc)
}

fn nv(b: i32, c: i32, av: i32, cx: i32, ae: i32) -> (i32, i32) {
    let bc: i32 = 600;
    let bzd = bc + av;
    if bzd <= 10 { return (cx, ae); }
    (cx + b * bc / bzd, ae - c * bc / bzd)
}

fn csb(q: f32, o: f32, ab: f32) -> f32 { q + (o - q) * ab }





const IY_: usize = 8;
const DZ_: usize = 12;

fn tbk() -> (Vec<V3>, Vec<D>, Vec<u8>) {
    let mut by = Vec::fc(IY_ * DZ_ + 2);
    let mut cdc = Vec::fc(IY_ * DZ_ + 2);
    let mut bu = Vec::new();
    let akk: f32 = 3.14159265;
    by.push(V3::new(0.0, -1.0, 0.0)); cdc.push(0);
    for ber in 0..IY_ {
        let avw = (ber as f32 + 1.0) / (IY_ as f32 + 1.0);
        let hg = -akk / 2.0 + akk * avw;
        let c = st(hg);
        let m = zq(hg);
        let bti: u8 = match ber { 0 => 0, 1 => 1, 2..=5 => 2, _ => 3 };
        for ann in 0..DZ_ {
            let auo = 2.0 * akk * (ann as f32) / (DZ_ as f32);
            by.push(V3::new(m * zq(auo), c, m * st(auo)));
            cdc.push(bti);
        }
    }
    by.push(V3::new(0.0, 1.0, 0.0)); cdc.push(3);
    let lpc = by.len() - 1;
    for ber in 0..IY_ {
        let ar = 1 + ber * DZ_;
        for ann in 0..DZ_ {
            let next = if ann + 1 < DZ_ { ann + 1 } else { 0 };
            bu.push(D((ar + ann) as u16, (ar + next) as u16));
        }
    }
    for ann in 0..DZ_ {
        bu.push(D(0, (1 + ann) as u16));
        for ber in 0..(IY_ - 1) {
            let q = 1 + ber * DZ_ + ann;
            let o = 1 + (ber + 1) * DZ_ + ann;
            bu.push(D(q as u16, o as u16));
        }
        bu.push(D((1 + (IY_ - 1) * DZ_ + ann) as u16, lpc as u16));
    }
    (by, bu, cdc)
}


fn tba() -> (Vec<V3>, Vec<D>) {
    let bnv: f32 = 1.618034; 
    let e = 1.0 / libm::bon(1.0 + bnv * bnv);
    let dm = bnv * e;
    let by = alloc::vec![
        V3::new(-e,  dm, 0.0), V3::new( e,  dm, 0.0),
        V3::new(-e, -dm, 0.0), V3::new( e, -dm, 0.0),
        V3::new(0.0, -e,  dm), V3::new(0.0,  e,  dm),
        V3::new(0.0, -e, -dm), V3::new(0.0,  e, -dm),
        V3::new( dm, 0.0, -e), V3::new( dm, 0.0,  e),
        V3::new(-dm, 0.0, -e), V3::new(-dm, 0.0,  e),
    ];
    #[allow(clippy::yxl)]
    let bu = alloc::vec![
        D(0,1), D(0,5), D(0,7), D(0,10), D(0,11),
        D(1,5), D(1,7), D(1,8), D(1,9),
        D(2,3), D(2,4), D(2,6), D(2,10), D(2,11),
        D(3,4), D(3,6), D(3,8), D(3,9),
        D(4,5), D(4,9), D(4,11),
        D(5,9), D(5,11),
        D(6,7), D(6,8), D(6,10),
        D(7,8), D(7,10),
        D(8,9), D(10,11),
    ];
    (by, bu)
}


fn tau() -> (Vec<V3>, Vec<D>) {
    let e: f32 = 0.82;
    let by = alloc::vec![
        V3::new(-e, -e, -e), V3::new( e, -e, -e),
        V3::new( e,  e, -e), V3::new(-e,  e, -e),
        V3::new(-e, -e,  e), V3::new( e, -e,  e),
        V3::new( e,  e,  e), V3::new(-e,  e,  e),
    ];
    let bu = alloc::vec![
        D(0,1), D(1,2), D(2,3), D(3,0),
        D(4,5), D(5,6), D(6,7), D(7,4),
        D(0,4), D(1,5), D(2,6), D(3,7),
    ];
    (by, bu)
}


fn taw() -> (Vec<V3>, Vec<D>) {
    let by = alloc::vec![
        V3::new(0.0,  1.2, 0.0),  
        V3::new(0.0, -1.2, 0.0),  
        V3::new( 1.0, 0.0, 0.0),  
        V3::new(-1.0, 0.0, 0.0),  
        V3::new(0.0, 0.0,  1.0),  
        V3::new(0.0, 0.0, -1.0),  
    ];
    let bu = alloc::vec![
        D(0,2), D(0,3), D(0,4), D(0,5),
        D(1,2), D(1,3), D(1,4), D(1,5),
        D(2,4), D(4,3), D(3,5), D(5,2),
    ];
    (by, bu)
}


fn tbm() -> (Vec<V3>, Vec<D>) {
    
    let mut by = Vec::fc(14);
    
    let e: f32 = 0.55;
    by.push(V3::new(0.0,  e, 0.0));   
    by.push(V3::new(0.0, -e, 0.0));   
    by.push(V3::new( e, 0.0, 0.0));   
    by.push(V3::new(-e, 0.0, 0.0));   
    by.push(V3::new(0.0, 0.0,  e));   
    by.push(V3::new(0.0, 0.0, -e));   
    
    let ab: f32 = 1.1;
    by.push(V3::new( ab,  ab,  ab));     
    by.push(V3::new(-ab,  ab,  ab));     
    by.push(V3::new( ab, -ab,  ab));     
    by.push(V3::new(-ab, -ab,  ab));     
    by.push(V3::new( ab,  ab, -ab));     
    by.push(V3::new(-ab,  ab, -ab));     
    by.push(V3::new( ab, -ab, -ab));     
    by.push(V3::new(-ab, -ab, -ab));     
    let mut bu = Vec::new();
    
    bu.push(D(0,2)); bu.push(D(0,3)); bu.push(D(0,4)); bu.push(D(0,5));
    bu.push(D(1,2)); bu.push(D(1,3)); bu.push(D(1,4)); bu.push(D(1,5));
    bu.push(D(2,4)); bu.push(D(4,3)); bu.push(D(3,5)); bu.push(D(5,2));
    
    for pmj in 6..14u16 {
        
        let bxk = by[pmj as usize];
        let mut nma: Vec<(u16, f32)> = (0..6u16).map(|a| {
            let hph = by[a as usize];
            let dx = bxk.b - hph.b; let bg = bxk.c - hph.c; let pt = bxk.av - hph.av;
            (a, dx*dx + bg*bg + pt*pt)
        }).collect();
        nma.bxe(|q, o| {
            if q.1 < o.1 { core::cmp::Ordering::Tg }
            else if q.1 > o.1 { core::cmp::Ordering::Ss }
            else { core::cmp::Ordering::Arq }
        });
        for eh in 0..3 {
            bu.push(D(pmj, nma[eh].0));
        }
    }
    (by, bu)
}






fn tbr(otm: u32, oyq: u32, jq: usize) -> (Vec<V3>, Vec<D>) {
    let mut by = Vec::fc(jq);
    let mut bu = Vec::new();
    let bvy: f32 = 6.28318;
    let lww: f32 = 0.7; 
    let jkz: f32 = 0.3; 

    for a in 0..jq {
        let ab = bvy * a as f32 / jq as f32;
        let m = lww + jkz * zq(oyq as f32 * ab);
        let b = m * zq(otm as f32 * ab);
        let c = m * st(otm as f32 * ab);
        let av = jkz * st(oyq as f32 * ab);
        by.push(V3::new(b, c, av));
    }
    
    for a in 0..jq {
        let next = (a + 1) % jq;
        bu.push(D(a as u16, next as u16));
    }
    
    for a in (0..jq).akt(4) {
        let bjr = (a + jq / 3) % jq;
        bu.push(D(a as u16, bjr as u16));
    }
    (by, bu)
}


fn tax(jq: usize, cuy: f32) -> (Vec<V3>, Vec<D>) {
    
    let mut by = Vec::fc(jq * 2);
    let mut bu = Vec::new();
    let bvy: f32 = 6.28318;
    let iyg: f32 = 0.5;
    let ac: f32 = 2.0;

    
    for a in 0..jq {
        let ab = a as f32 / jq as f32;
        let hg = bvy * cuy * ab;
        let c = -ac / 2.0 + ac * ab;
        by.push(V3::new(iyg * zq(hg), c, iyg * st(hg)));
    }
    
    for a in 0..jq {
        let ab = a as f32 / jq as f32;
        let hg = bvy * cuy * ab + 3.14159;
        let c = -ac / 2.0 + ac * ab;
        by.push(V3::new(iyg * zq(hg), c, iyg * st(hg)));
    }
    
    for a in 0..(jq - 1) {
        bu.push(D(a as u16, (a + 1) as u16)); 
        bu.push(D((jq + a) as u16, (jq + a + 1) as u16)); 
    }
    
    for a in (0..jq).akt(3) {
        bu.push(D(a as u16, (jq + a) as u16));
    }
    (by, bu)
}


fn tbq() -> (Vec<[f32; 4]>, Vec<D>) {
    
    let mut jvk: Vec<[f32; 4]> = Vec::fc(16);
    let e: f32 = 0.6;
    for a in 0..16u8 {
        let b = if a & 1 != 0 { e } else { -e };
        let c = if a & 2 != 0 { e } else { -e };
        let av = if a & 4 != 0 { e } else { -e };
        let d = if a & 8 != 0 { e } else { -e };
        jvk.push([b, c, av, d]);
    }
    
    let mut bu = Vec::new();
    for a in 0..16u16 {
        for ga in 0..4u16 {
            let fb = a ^ (1 << ga);
            if fb > a {
                bu.push(D(a, fb));
            }
        }
    }
    (jvk, bu)
}


fn vnc(p: [f32; 4], jwd: f32) -> V3 {
    
    let dt = zq(jwd);
    let kp = st(jwd);
    let b = p[0] * dt - p[3] * kp;
    let d = p[0] * kp + p[3] * dt;
    
    let nir = zq(jwd * 0.7);
    let pqe = st(jwd * 0.7);
    let c = p[1] * nir - d * pqe;
    let bfs = p[1] * pqe + d * nir;
    
    let njc: f32 = 2.5;
    let bv = njc / (njc + bfs);
    V3::new(b * bv, c * bv, p[2] * bv)
}


fn tbv(um: usize, jq: usize) -> (Vec<V3>, Vec<D>) {
    let mut by = Vec::fc(um * jq);
    let mut bu = Vec::new();
    let bvy: f32 = 6.28318;

    for mz in 0..um {
        let ab = mz as f32 / um as f32;
        let av = -1.0 + 2.0 * ab;
        let m = 0.2 + 0.6 * (1.0 - ab); 
        let xnj = ab * 2.0; 
        for pk in 0..jq {
            let hg = bvy * pk as f32 / jq as f32 + xnj;
            let b = m * zq(hg);
            let c = m * st(hg);
            by.push(V3::new(b, c, av));
        }
    }
    
    for mz in 0..um {
        let ar = mz * jq;
        for pk in 0..jq {
            let next = (pk + 1) % jq;
            bu.push(D((ar + pk) as u16, (ar + next) as u16));
        }
    }
    
    for mz in 0..(um - 1) {
        let ar = mz * jq;
        let uub = (mz + 1) * jq;
        for pk in (0..jq).akt(2) {
            bu.push(D((ar + pk) as u16, (uub + pk) as u16));
        }
    }
    (by, bu)
}


fn tbh() -> (Vec<V3>, Vec<D>) {
    
    let ber: usize = 6;
    let ann: usize = 10;
    let mut by = Vec::fc(ber * ann + 2);
    let mut bu = Vec::new();
    let akk: f32 = 3.14159;

    by.push(V3::new(0.0, -0.5, 0.0));
    for auo in 0..ber {
        let avw = (auo as f32 + 1.0) / (ber as f32 + 1.0);
        let hg = -akk / 2.0 + akk * avw;
        let c = st(hg) * 0.5;
        let m = zq(hg) * 0.5;
        for hh in 0..ann {
            let q = 6.28318 * hh as f32 / ann as f32;
            by.push(V3::new(m * zq(q), c, m * st(q)));
        }
    }
    by.push(V3::new(0.0, 0.5, 0.0));
    
    for auo in 0..ber {
        let ar = 1 + auo * ann;
        for hh in 0..ann {
            let next = if hh + 1 < ann { hh + 1 } else { 0 };
            bu.push(D((ar + hh) as u16, (ar + next) as u16));
        }
    }
    
    for hh in 0..ann {
        bu.push(D(0, (1 + hh) as u16));
        for auo in 0..(ber - 1) {
            let q = 1 + auo * ann + hh;
            let o = 1 + (auo + 1) * ann + hh;
            bu.push(D(q as u16, o as u16));
        }
        let qv = 1 + (ber - 1) * ann + hh;
        bu.push(D(qv as u16, (by.len() - 1) as u16));
    }
    (by, bu)
}


fn taz(cvq: usize, cbc: usize) -> (Vec<V3>, Vec<D>) {
    let mut by = Vec::fc(cvq * cbc + 1);
    let mut bu = Vec::new();
    let bvy: f32 = 6.28318;

    
    by.push(V3::new(0.0, 0.0, 0.0));

    for ccx in 0..cvq {
        let kaz = bvy * ccx as f32 / cvq as f32;
        for a in 0..cbc {
            let ab = (a as f32 + 1.0) / cbc as f32;
            let m = 0.1 + ab * 0.9;
            let hg = kaz + ab * bvy * 1.2; 
            let b = m * zq(hg);
            let av = m * st(hg);
            
            let c = st(hg * 2.0) * 0.08 * m;
            by.push(V3::new(b, c, av));
        }
    }
    
    for ccx in 0..cvq {
        let ar = 1 + ccx * cbc;
        
        bu.push(D(0, ar as u16));
        for a in 0..(cbc - 1) {
            bu.push(D((ar + a) as u16, (ar + a + 1) as u16));
        }
    }
    
    for ccx in 0..cvq {
        let lof = (ccx + 1) % cvq;
        for a in (0..cbc).akt(4) {
            let q = 1 + ccx * cbc + a;
            let o = 1 + lof * cbc + a;
            if q < by.len() && o < by.len() {
                bu.push(D(q as u16, o as u16));
            }
        }
    }
    (by, bu)
}







fn tbp() -> (Vec<V3>, Vec<D>) {
    let mut by = Vec::fc(256);
    let mut bu = Vec::new();

    
    
    let aix = 0.9f32;   
    let hga = 0.55f32;  
    let pt = 0.08f32;  

    
    let abk = by.len();
    by.push(V3::new(0.0, hga, pt));           
    by.push(V3::new(aix, 0.0, pt));           
    by.push(V3::new(0.0, -hga, pt));          
    by.push(V3::new(-aix, 0.0, pt));          

    
    by.push(V3::new(0.0, hga, -pt));          
    by.push(V3::new(aix, 0.0, -pt));          
    by.push(V3::new(0.0, -hga, -pt));         
    by.push(V3::new(-aix, 0.0, -pt));         

    
    bu.push(D(abk as u16, (abk + 1) as u16));
    bu.push(D((abk + 1) as u16, (abk + 2) as u16));
    bu.push(D((abk + 2) as u16, (abk + 3) as u16));
    bu.push(D((abk + 3) as u16, abk as u16));
    
    bu.push(D((abk + 4) as u16, (abk + 5) as u16));
    bu.push(D((abk + 5) as u16, (abk + 6) as u16));
    bu.push(D((abk + 6) as u16, (abk + 7) as u16));
    bu.push(D((abk + 7) as u16, (abk + 4) as u16));
    
    for a in 0..4 {
        bu.push(D((abk + a) as u16, (abk + 4 + a) as u16));
    }

    
    let ars = 0.22f32;   
    let afv = 0.28f32;
    let gpm = 0.05f32; 
    let cbe = pt + 0.02;

    let se = by.len();
    
    by.push(V3::new(-ars + gpm, afv, cbe));       
    by.push(V3::new(ars * 1.2 + gpm, 0.0, cbe)); 
    by.push(V3::new(-ars + gpm, -afv, cbe));      
    
    by.push(V3::new(-ars + gpm, afv, -cbe));
    by.push(V3::new(ars * 1.2 + gpm, 0.0, -cbe));
    by.push(V3::new(-ars + gpm, -afv, -cbe));

    
    bu.push(D(se as u16, (se + 1) as u16));
    bu.push(D((se + 1) as u16, (se + 2) as u16));
    bu.push(D((se + 2) as u16, se as u16));
    
    bu.push(D((se + 3) as u16, (se + 4) as u16));
    bu.push(D((se + 4) as u16, (se + 5) as u16));
    bu.push(D((se + 5) as u16, (se + 3) as u16));
    
    for a in 0..3 {
        bu.push(D((se + a) as u16, (se + 3 + a) as u16));
    }

    
    
    let sl = -hga - 0.25;  
    let jdj = 0.14f32;    
    let oix = 0.15f32;    
    let jtc = 0.04f32;      
    let aza = 9.0 * jdj; 
    let ql = -aza / 2.0;

    
    
    let ued: [&[(f32, f32, f32, f32)]; 9] = [
        
        &[(0.0, 1.0, 0.8, 1.0), (0.0, 1.0, 0.0, 0.5), (0.0, 0.5, 0.8, 0.5),
          (0.8, 0.5, 0.8, 0.0), (0.0, 0.0, 0.8, 0.0)],
        
        &[(0.0, 1.0, 0.0, 0.0), (0.0, 0.0, 0.8, 0.0), (0.8, 0.0, 0.8, 1.0)],
        
        &[(0.0, 0.0, 0.0, 1.0), (0.0, 1.0, 0.6, 1.0), (0.6, 1.0, 0.7, 0.75),
          (0.7, 0.75, 0.6, 0.5), (0.0, 0.5, 0.6, 0.5), (0.6, 0.5, 0.7, 0.25),
          (0.7, 0.25, 0.6, 0.0), (0.0, 0.0, 0.6, 0.0)],
        
        &[(0.0, 1.0, 0.8, 1.0), (0.0, 1.0, 0.0, 0.5), (0.0, 0.5, 0.8, 0.5),
          (0.8, 0.5, 0.8, 0.0), (0.0, 0.0, 0.8, 0.0)],
        
        &[(0.8, 1.0, 0.0, 1.0), (0.0, 1.0, 0.0, 0.0), (0.0, 0.0, 0.8, 0.0)],
        
        &[(0.0, 0.0, 0.0, 1.0), (0.0, 1.0, 0.6, 1.0), (0.6, 1.0, 0.7, 0.75),
          (0.7, 0.75, 0.6, 0.5), (0.0, 0.5, 0.6, 0.5), (0.4, 0.5, 0.8, 0.0)],
        
        &[(0.2, 1.0, 0.6, 1.0), (0.4, 1.0, 0.4, 0.0), (0.2, 0.0, 0.6, 0.0)],
        
        &[(0.0, 0.0, 0.0, 1.0), (0.0, 1.0, 0.6, 1.0), (0.6, 1.0, 0.7, 0.75),
          (0.7, 0.75, 0.6, 0.5), (0.0, 0.5, 0.6, 0.5), (0.6, 0.5, 0.7, 0.25),
          (0.7, 0.25, 0.6, 0.0), (0.0, 0.0, 0.6, 0.0)],
        
        &[(0.0, 0.0, 0.0, 1.0), (0.0, 1.0, 0.8, 1.0), (0.0, 0.5, 0.6, 0.5),
          (0.0, 0.0, 0.8, 0.0)],
    ];

    for (alj, wvf) in ued.iter().cf() {
        let mp = ql + alj as f32 * jdj;
        for &(dn, dp, hy, jz) in *wvf {
            let asa = mp + dn * (jdj * 0.85);
            let bos = sl + dp * oix;
            let amy = mp + hy * (jdj * 0.85);
            let bcw = sl + jz * oix;

            let afj = by.len();
            
            by.push(V3::new(asa, bos, jtc));
            by.push(V3::new(amy, bcw, jtc));
            
            by.push(V3::new(asa, bos, -jtc));
            by.push(V3::new(amy, bcw, -jtc));
            
            bu.push(D(afj as u16, (afj + 1) as u16));
            
            bu.push(D((afj + 2) as u16, (afj + 3) as u16));
            
            bu.push(D(afj as u16, (afj + 2) as u16));
            bu.push(D((afj + 1) as u16, (afj + 3) as u16));
        }
    }

    (by, bu)
}

fn qtq(e: &mut VisualizerState) {
    
    e.by.clear();
    e.by.bk(&e.mib);
    e.bu.clear();
    e.bu.bk(&e.mic);
}





fn aqz(e: &mut VisualizerState) {
    if e.jr { return; }

    
    let (bxk, grr, is) = tbk();
    e.jqx = bxk.clone();
    e.mgo = is.clone();
    e.mgp = grr.clone();
    e.jqy = bxk;
    e.mgt = grr;
    e.mgs = is;

    
    let (ocx, tqz) = tba();
    let (nhy, rrn) = tau();
    let (nlf, rxf) = taw();
    let (pnw, wsk) = tbm();

    
    let jfm = ocx.len().am(nhy.len()).am(nlf.len()).am(pnw.len());

    
    fn jii(p: &[V3], cd: usize) -> Vec<V3> {
        let mut bd = p.ip();
        while bd.len() < cd {
            bd.push(*bd.qv().unwrap_or(&V3::new(0.0, 0.0, 0.0)));
        }
        bd
    }

    e.eux[0] = jii(&ocx, jfm);
    e.eux[1] = jii(&nhy, jfm);
    e.eux[2] = jii(&nlf, jfm);
    e.eux[3] = jii(&pnw, jfm);
    e.gmw[0] = tqz;
    e.gmw[1] = rrn;
    e.gmw[2] = rxf;
    e.gmw[3] = wsk;
    e.onz = e.eux[0].clone();

    
    e.glx = Vec::fc(400);
    e.jdy = [1.0, 1.0, 1.0];

    
    e.dvb = Vec::fc(128);

    
    e.ewd = Vec::fc(200);

    
    let (xib, xia) = tbr(2, 3, 120);
    e.jtn = xib;
    e.mlx = xia;

    
    let (rzr, rzq) = tax(60, 3.0);
    e.irj = rzr;
    e.kqj = rzq;

    
    

    
    let (xta, xsz) = tbv(10, 12);
    e.mpx = xta;
    e.mpy = xsz;

    
    let (vnq, vnp) = tbh();
    e.jjk = vnq;
    e.lue = vnp;

    
    let (szz, szy) = taz(4, 20);
    e.nxc = szz;
    e.nxd = szy;

    
    let (wvq, wvl) = tbp();
    e.mib = wvq;
    e.mic = wvl;

    e.jr = true;
}





fn nao(e: &mut VisualizerState) {
    
    let gyo = [
        e.bfi * 1.2,
        e.ays * 1.0,
        e.cib * 0.25,
        e.cbw * 0.15,
    ];
    let byj = e.ana * (0.3 + e.bfi * 0.3 + e.ays * 0.2);

    e.by.clear();
    for a in 0..e.jqy.len() {
        let yu = e.jqy[a];
        let bti = e.mgs[a] as usize;
        let byf = if bti < 4 { gyo[bti] } else { 0.0 };
        let m = 1.0 + 0.35 * byf + byj * 0.25;
        e.by.push(V3::new(yu.b * m, yu.c * m, yu.av * m));
    }
    e.bu.clear();
    e.bu.bk(&e.mgt);
}

fn qtm(e: &mut VisualizerState) {
    
    let sxx = e.hrv;
    let ptu = (e.hrv + 1) % 4;
    let ab = e.gmx - libm::yqz(e.gmx); 

    
    let mge = ab * ab * (3.0 - 2.0 * ab);

    let from = &e.eux[sxx];
    let wh = &e.eux[ptu];
    let az = from.len().v(wh.len());

    
    let gkn = e.ana * 0.15;

    e.by.clear();
    for a in 0..az {
        let b = csb(from[a].b, wh[a].b, mge) + gkn * st(a as f32 * 2.1);
        let c = csb(from[a].c, wh[a].c, mge) + gkn * zq(a as f32 * 1.7);
        let av = csb(from[a].av, wh[a].av, mge) + gkn * st(a as f32 * 3.3);
        e.by.push(V3::new(b, c, av));
    }

    
    e.bu.clear();
    e.bu.bk(&e.gmw[ptu]);
}

fn qtr(e: &mut VisualizerState) {
    
    let wny = 10.0 + e.ays * 5.0;
    let vys = 28.0 + e.cib * 10.0;
    let dyu = 2.667 + e.cbw * 1.0;
    let os: f32 = 0.006;

    let [b, c, av] = e.jdy;
    let dx = wny * (c - b);
    let bg = b * (vys - av) - c;
    let pt = b * c - dyu * av;
    let vt = b + dx * os;
    let ahr = c + bg * os;
    let arn = av + pt * os;
    e.jdy = [vt, ahr, arn];

    
    
    let bv = 0.04;
    let se = V3::new(vt * bv, (arn - 25.0) * bv, ahr * bv);
    e.glx.push(se);
    const EL_: usize = 350;
    while e.glx.len() > EL_ {
        e.glx.remove(0);
    }

    
    e.by.clear();
    e.bu.clear();
    for ai in &e.glx {
        e.by.push(*ai);
    }
    for a in 0..(e.by.len().ao(1)) {
        if a < 65535 {
            e.bu.push(D(a as u16, (a + 1) as u16));
        }
    }
}

fn qts(e: &mut VisualizerState) {
    
    let gyo = [
        e.bfi * 0.8,
        e.ays * 0.6,
        e.cib * 0.5,
        e.cbw * 0.35,
    ];
    let byj = e.ana * 0.4;

    e.by.clear();
    for a in 0..e.jqx.len() {
        let yu = e.jqx[a];
        let bti = e.mgo[a] as usize;
        let byf = if bti < 4 { gyo[bti] } else { 0.0 };
        
        let m = 1.0 + 0.6 * byf + byj * 0.3;
        e.by.push(V3::new(yu.b * m, yu.c * m, yu.av * m));
    }
    e.bu.clear();
    e.bu.bk(&e.mgp);
}

fn qtt(e: &mut VisualizerState) {
    
    const Aye: usize = 80;
    const Cqc: f32 = 0.6;

    
    if e.dvb.len() < Aye {
        
        e.dvb.clear();
        for a in 0..Aye {
            let ab = a as f32 / Aye as f32;
            e.dvb.push(V3::new(0.0, 0.0, -1.5 + 3.0 * ab));
        }
    }

    
    e.dvb.remove(0);
    let uct = e.dvb.qv().efd(1.5, |p| p.av);
    let bg = (e.cib * 0.6 + e.cbw * 0.3) * st(e.frame as f32 * 0.1);
    let dx = e.ays * 0.4 * zq(e.frame as f32 * 0.07);
    e.dvb.push(V3::new(dx, bg, uct));

    
    e.by.clear();
    e.bu.clear();
    for (a, sp) in e.dvb.iter().cf() {
        
        let d = Cqc + e.bfi * 0.3;
        let hg = e.frame as f32 * 0.02 + a as f32 * 0.08;
        let vt = zq(hg);
        let ahr = st(hg);
        e.by.push(V3::new(sp.b - vt * d, sp.c - ahr * d, sp.av));
        e.by.push(V3::new(sp.b + vt * d, sp.c + ahr * d, sp.av));
    }
    
    let bo = e.dvb.len();
    for a in 0..bo {
        let ar = (a * 2) as u16;
        
        if ar + 1 < e.by.len() as u16 {
            e.bu.push(D(ar, ar + 1));
        }
        
        if a + 1 < bo {
            let next = ((a + 1) * 2) as u16;
            e.bu.push(D(ar, next));
            e.bu.push(D(ar + 1, next + 1));
        }
    }
}

fn qtu(e: &mut VisualizerState) {
    
    e.lsz = e.lsz.cn(1);

    
    if e.ana > 0.8 {
        let az = 30 + (e.ays * 20.0) as usize;
        let dv = e.frame as u32;
        for a in 0..az.v(50) {
            let hash = dv.hx(2654435761).cn(a as u32);
            let jf = ((hash & 0xFF) as f32 / 128.0 - 1.0) * 0.8;
            let sc = (((hash >> 8) & 0xFF) as f32 / 128.0 - 1.0) * 0.8;
            let szu = (((hash >> 16) & 0xFF) as f32 / 128.0 - 1.0) * 0.8;
            let ig = 0.03 + (e.bfi + e.ays) * 0.02;
            e.ewd.push(Bop {
                b: 0.0, c: 0.0, av: 0.0,
                fp: jf * ig, iz: sc * ig, ciq: szu * ig,
                can: 1.0,
            });
        }
    }

    
    for ai in e.ewd.el() {
        ai.b += ai.fp;
        ai.c += ai.iz;
        ai.av += ai.ciq;
        ai.iz -= 0.0003; 
        ai.can -= 0.012;
    }
    e.ewd.ajm(|ai| ai.can > 0.0);
    while e.ewd.len() > 200 { e.ewd.remove(0); }

    
    e.by.clear();
    e.bu.clear();
    for ai in &e.ewd {
        e.by.push(V3::new(ai.b, ai.c, ai.av));
    }
    let bo = e.by.len();
    
    for a in (0..bo.ao(1)).akt(1) {
        if a + 1 < bo && a < 65535 {
            e.bu.push(D(a as u16, (a + 1) as u16));
        }
    }
    
    for a in (0..bo.ao(3)).akt(3) {
        if a + 3 < bo && a + 3 < 65535 {
            e.bu.push(D(a as u16, (a + 3) as u16));
        }
    }
}

fn qtv(e: &mut VisualizerState) {
    
    
    
    let cfz = crate::logo_bitmap::AY_ as u32;
    let cfy = crate::logo_bitmap::BL_ as u32;

    
    let dgi = cfz * 3 / 2;
    let eaw = cfy * 3 / 2;
    e.edd = dgi;
    e.edc = eaw;
    e.fla = e.yv - dgi as i32 / 2;
    e.flb = e.uq - eaw as i32 / 2;

    
    let xg = (e.ana * 40.0) as i32;
    e.fla -= xg / 2;
    e.flb -= xg / 2;
    e.edd = (dgi as i32 + xg) as u32;
    e.edc = (eaw as i32 + xg) as u32;

    
    let oy = e.oy;
    if oy > 0 {
        let aur = e.anc.len();
        for bj in 0..aur {
            let cds = bj as i32 * oy + oy / 2;
            if cds >= e.fla && cds < e.fla + e.edd as i32 {
                e.anc[bj] = (
                    e.flb.am(0),
                    (e.flb + e.edc as i32).am(0),
                );
            }
        }
    }

    
    e.by.clear();
    e.bu.clear();
}

fn qtw(e: &mut VisualizerState) {
    
    let byj = e.ana * 0.3 + e.ays * 0.2;
    let xme = e.cbw * 0.15;
    let time = e.frame as f32 * 0.02;

    e.by.clear();
    for (a, yu) in e.jtn.iter().cf() {
        let ab = a as f32 / e.jtn.len().am(1) as f32;
        
        let m = 1.0 + byj * 0.4 + st(ab * 6.28 * 3.0 + time) * xme;
        e.by.push(V3::new(yu.b * m, yu.c * m, yu.av * m));
    }
    e.bu.clear();
    e.bu.bk(&e.mlx);
}

fn qtx(e: &mut VisualizerState) {
    
    let byj = e.ana * 0.25;
    let uof = e.cib * 0.3;
    let time = e.frame as f32 * 0.015;
    let jq = e.irj.len() / 2;

    e.by.clear();
    for (a, yu) in e.irj.iter().cf() {
        let ab = (a % jq.am(1)) as f32 / jq.am(1) as f32;
        
        let m = 1.0 + byj * 0.3;
        
        let nsi = uof * st(ab * 6.28 + time);
        let cdp = zq(nsi);
        let bcm = st(nsi);
        let vt = yu.b * cdp - yu.av * bcm;
        let arn = yu.b * bcm + yu.av * cdp;
        e.by.push(V3::new(vt * m, yu.c * m, arn * m));
    }
    e.bu.clear();
    e.bu.bk(&e.kqj);
}

fn qty(e: &mut VisualizerState) {
    
    e.mkg += 0.02 + e.ays * 0.05 + e.ana * 0.1;

    let (jvk, siz) = tbq();

    e.by.clear();
    for cnq in &jvk {
        let bdf = vnc(*cnq, e.mkg);
        
        let xg = 1.0 + e.ana * 0.3;
        e.by.push(V3::new(bdf.b * xg, bdf.c * xg, bdf.av * xg));
    }
    e.bu.clear();
    e.bu.bk(&siz);
}

fn qtn(e: &mut VisualizerState) {
    
    let time = e.frame as f32 * 0.03;
    let byj = e.ana * 0.4;
    let um = 10usize;
    let jq = 12usize;

    e.by.clear();
    for (a, yu) in e.mpx.iter().cf() {
        let mz = a / jq;
        let ab = mz as f32 / um as f32;
        
        let pej = 1.0 + byj * (1.0 - ab) + e.bfi * 0.2;
        
        let spin = time * (1.0 + ab * 2.0);
        let cdp = zq(spin);
        let bcm = st(spin);
        let vt = yu.b * cdp - yu.c * bcm;
        let ahr = yu.b * bcm + yu.c * cdp;
        e.by.push(V3::new(vt * pej, ahr * pej, yu.av));
    }
    e.bu.clear();
    e.bu.bk(&e.mpy);
}

fn qto(e: &mut VisualizerState) {
    
    let time = e.frame as f32 * 0.02;
    let byj = e.ana * 0.5;
    let abo = e.bfi + e.ays;

    e.by.clear();
    
    for yu in e.jjk.iter() {
        let m = 1.0 + byj * 0.3;
        e.by.push(V3::new(yu.b * m, yu.c * m, yu.av * m));
    }
    let gdj = e.jjk.len();

    
    let lpo: usize = 6;
    let jsu: usize = 8;
    let bvy: f32 = 6.28318;
    for ab in 0..lpo {
        let iku = bvy * ab as f32 / lpo as f32 + time * 0.5;
        let ksy = st(time * 0.7 + ab as f32 * 1.5) * 0.6;
        let fde = zq(iku) * zq(ksy) * 0.5;
        let dec = st(ksy) * 0.5;
        let qns = st(iku) * zq(ksy) * 0.5;

        for pk in 0..jsu {
            let apc = (pk as f32 + 1.0) / jsu as f32;
            
            let go = 0.5 + abo * 0.4 + byj * 0.3;
            let uag = st(time * 3.0 + pk as f32 * 2.0 + ab as f32) * 0.08 * apc;
            let uah = zq(time * 2.5 + pk as f32 * 1.7 + ab as f32) * 0.08 * apc;
            let uai = st(time * 2.0 + pk as f32 * 2.3 + ab as f32) * 0.08 * apc;
            e.by.push(V3::new(
                fde * (1.0 + apc * go) + uag,
                dec * (1.0 + apc * go) + uah,
                qns * (1.0 + apc * go) + uai,
            ));
        }
    }

    e.bu.clear();
    e.bu.bk(&e.lue);
    
    for ab in 0..lpo {
        let gzt = gdj + ab * jsu;
        
        
        if gzt < e.by.len() && gdj > 0 {
            let fxm = e.by[gzt];
            let mut ilj = f32::O;
            let mut hae = 0u16;
            for nc in 0..gdj {
                let gec = e.by[nc];
                let dx = fxm.b - gec.b; let bg = fxm.c - gec.c; let pt = fxm.av - gec.av;
                let bc = dx*dx + bg*bg + pt*pt;
                if bc < ilj { ilj = bc; hae = nc as u16; }
            }
            e.bu.push(D(hae, gzt as u16));
        }
        
        for pk in 0..(jsu - 1) {
            let q = gzt + pk;
            let o = gzt + pk + 1;
            if q < e.by.len() && o < e.by.len() {
                e.bu.push(D(q as u16, o as u16));
            }
        }
    }
}

fn qtp(e: &mut VisualizerState) {
    
    let time = e.frame as f32 * 0.01;
    let byj = e.ana * 0.3;
    let bvy: f32 = 6.28318;
    let cvq: usize = 4;
    let cbc: usize = 20;

    e.by.clear();
    
    e.by.push(V3::new(0.0, st(time * 2.0) * 0.05, 0.0));

    for ccx in 0..cvq {
        let kaz = bvy * ccx as f32 / cvq as f32;
        for a in 0..cbc {
            let ab = (a as f32 + 1.0) / cbc as f32;
            let m = 0.1 + ab * (0.9 + byj * 0.3);
            let hg = kaz + ab * bvy * 1.2 + time;
            let b = m * zq(hg);
            let av = m * st(hg);
            
            let c = st(hg * 2.0 + time * 1.5) * 0.1 * m;
            
            let gkn = e.cbw * 0.05 * ab;
            let uaw = st(time * 4.0 + a as f32) * gkn;
            let uax = zq(time * 3.5 + a as f32) * gkn;
            e.by.push(V3::new(b + uaw, c, av + uax));
        }
    }

    e.bu.clear();
    
    for ccx in 0..cvq {
        let ar = 1 + ccx * cbc;
        e.bu.push(D(0, ar as u16));
        for a in 0..(cbc - 1) {
            e.bu.push(D((ar + a) as u16, (ar + a + 1) as u16));
        }
    }
    
    for ccx in 0..cvq {
        let lof = (ccx + 1) % cvq;
        for a in (0..cbc).akt(4) {
            let q = 1 + ccx * cbc + a;
            let o = 1 + lof * cbc + a;
            if q < e.by.len() && o < e.by.len() {
                e.bu.push(D(q as u16, o as u16));
            }
        }
    }
}





pub fn qs(
    g: &mut VisualizerState,
    wf: u32, aav: u32,
    cas: usize,
    rf: f32, abo: f32,
    ato: f32, aee: f32, vs: f32, axg: f32,
    uu: bool,
) {
    aqz(g);
    g.frame = g.frame.cn(1);
    g.yv = wf as i32 / 2;
    g.uq = aav as i32 / 2;

    
    let bie = 0.15f32;
    if uu {
        g.bfi += (ato - g.bfi) * bie;
        g.ays     += (aee     - g.ays)     * bie;
        g.cib      += (vs      - g.cib)      * bie;
        g.cbw   += (axg   - g.cbw)   * bie;
    } else {
        g.bfi *= 0.95;
        g.ays     *= 0.95;
        g.cib      *= 0.95;
        g.cbw   *= 0.95;
    }

    
    let iky = uu && rf > 0.5 && (ato + aee) > 0.4;
    if iky { g.ana = 1.0; }
    g.ana *= 0.90;

    
    if g.ana > 0.9 { g.bws = 0.0; }
    if g.bws < 600.0 { g.bws += 6.0; }

    
    if g.ev == 13 {
        
        g.cbn += 3;
        g.boe += 5;
        g.dlj += 1;
        g.cbn %= 6283; g.boe %= 6283; g.dlj %= 6283;
    } else {
        let gzy = if uu {
            ((g.bfi + g.ays) * 15.0 + rf * 8.0) as i32
        } else { 0 };
        g.cbn += g.hxx + gzy / 4;
        g.boe += g.hxy + gzy;
        g.dlj += g.hxz;
        g.cbn %= 6283; g.boe %= 6283; g.dlj %= 6283;
    }

    
    if g.ev == 13 {
        
        g.bv = 160;
        g.eya = 160;
    } else {
        g.eya = if uu {
            150 + ((g.bfi + g.ays) * 25.0) as i32
                + (g.ana * 25.0) as i32
        } else { 150 };
        g.bv += (g.eya - g.bv) / 3;
        g.bv = g.bv.am(80).v(220);
    }

    
    if g.ev == 13 {
        
        let wmn = (g.bv as f32 * 0.9).am(80.0);
        let wmo = (g.bv as f32 * 0.7).am(60.0);

        
        g.fhf += g.gfr;
        g.fhg += g.gfs;

        
        let czm = wmn;
        let jex = wmo;
        let kp = wf as f32;
        let kl = aav as f32;

        if g.fhf - czm < 0.0 {
            g.fhf = czm;
            g.gfr = g.gfr.gp();
        } else if g.fhf + czm > kp {
            g.fhf = kp - czm;
            g.gfr = -(g.gfr.gp());
        }
        if g.fhg - jex < 0.0 {
            g.fhg = jex;
            g.gfs = g.gfs.gp();
        } else if g.fhg + jex > kl {
            g.fhg = kl - jex;
            g.gfs = -(g.gfs.gp());
        }

        
        g.yv = g.fhf as i32;
        g.uq = g.fhg as i32;

        
        if iky {
            g.gfq = 1.0;
        }
        g.gfq *= 0.92;
        if g.gfq < 0.01 { g.gfq = 0.0; }
    }

    
    if g.ev == 1 {
        
        let ig = if g.ana > 0.5 { 0.08 } else { 0.004 };
        g.gmx += ig;
        if g.gmx >= 1.0 {
            g.gmx -= 1.0;
            g.hrv = (g.hrv + 1) % 4;
        }
    }

    
    match g.ev {
        0 => nao(g),
        1 => qtm(g),
        2 => qtr(g),
        3 => qts(g),
        4 => qtt(g),
        5 => qtu(g),
        6 => qtv(g),
        7 => qtw(g),
        8 => qtx(g),
        9 => qty(g),
        10 => qtn(g),
        11 => qto(g),
        12 => qtp(g),
        13 => qtq(g),
        _ => nao(g),
    }

    
    let (bv, kb, ix, agv) = (g.bv, g.cbn, g.boe, g.dlj);
    let (cx, ae) = (g.yv, g.uq);

    g.bwh.clear();
    g.cgu.clear();
    let mut fzj: i16 = i16::O;
    let mut fzi: i16 = i16::Avc;
    for p in &g.by {
        let (ajr, dnn, eli) = cni(*p, kb, ix, agv, bv);
        g.bwh.push(nv(ajr, dnn, eli, cx, ae));
        let dns = (eli as i16).am(-500).v(500);
        g.cgu.push(dns);
        if dns < fzj { fzj = dns; }
        if dns > fzi { fzi = dns; }
    }
    g.elk = fzj;
    g.elj = fzi;

    
    {
        let mut hac: i32 = i32::Avc;
        let mut hag: i32 = cx;
        let mut hah: i32 = ae;
        let (mj, ct, ujd) = (500i32, 700, 500);
        for (afj, p) in g.by.iter().cf() {
            let (vt, ahr, arn) = cni(*p, kb, ix, agv, 1000);
            let amb = (vt * mj + ahr * ct + arn * ujd) / 1000;
            if amb > hac {
                hac = amb;
                if afj < g.bwh.len() {
                    hag = g.bwh[afj].0;
                    hah = g.bwh[afj].1;
                }
            }
        }
        g.eyt = hag;
        g.dcd = hah;
    }

    
    {
        let mut dhp: i32 = -1;
        for &(_, wv) in g.anc.iter() {
            if wv > dhp { dhp = wv; }
        }
        if dhp > 0 {
            g.bsj = dhp;
            g.dlt = dhp + 120;
        } else {
            g.bsj = 0; g.dlt = 0;
        }
        g.dvw = ae;
    }

    
    let oy = if cas > 0 { wf as i32 / cas as i32 } else { 8 };
    g.oy = oy;

    if g.btt.len() != cas {
        g.btt.clear();
        g.anc.clear();
        for _ in 0..cas {
            g.btt.push(Vec::new());
            g.anc.push((-1, -1));
        }
    } else {
        for i in g.btt.el() { i.clear(); }
        for o in g.anc.el() { *o = (-1, -1); }
    }

    for (ksw, amd) in g.bu.iter().cf() {
        let q = amd.0 as usize;
        let o = amd.1 as usize;
        if q >= g.bwh.len() || o >= g.bwh.len() { continue; }
        let (fy, fo) = g.bwh[q];
        let (dn, dp) = g.bwh[o];
        let fzg = if q < g.cgu.len() && o < g.cgu.len() {
            ((g.cgu[q] as i32 + g.cgu[o] as i32) / 2) as i16
        } else { 0 };
        lxb(fy, fo, dn, dp, ksw as u16, fzg, oy, cas,
                       aav as i32, &mut g.btt, &mut g.anc);
    }
}

fn lxb(
    fy: i32, fo: i32, dn: i32, dp: i32,
    cei: u16, fzg: i16, oy: i32, aur: usize, kl: i32,
    buw: &mut [Vec<(i32, i32, u16, u8, i16)>],
    eg: &mut [(i32, i32)],
) {
    if oy <= 0 { return; }
    let dx = (dn - fy).gp();
    let bg = (dp - fo).gp();
    let au = dx.am(bg).am(1).v(2048);
    let dcj = ((dn - fy) * 1024) / au;
    let ejd = ((dp - fo) * 1024) / au;
    let mut y = fy * 1024;
    let mut x = fo * 1024;
    let gpz = 8i32;
    let oq = 2;
    let mut e = 0;
    while e <= au {
        let cr = y / 1024;
        let cq = x / 1024;
        let r = cr / oy;
        if r >= 0 && (r as usize) < aur && cq >= 0 && cq < kl {
            let nc = r as usize;
            if buw[nc].len() < 64 {
                let dnq = (cq - gpz).am(0);
                let dnp = (cq + gpz).v(kl - 1);
                let la = ((cr - r * oy - oy / 2).gp() as u32 * 255 / (gpz as u32 + 1)).v(255) as u8;
                let hj = 255u8.ao(la);
                buw[nc].push((dnq, dnp, cei, hj, fzg));
            }
            let (ref mut uj, ref mut wv) = eg[nc];
            let dxu = cq;
            if *uj < 0 || dxu < *uj { *uj = dxu; }
            if dxu > *wv { *wv = dxu; }
        }
        y += dcj * oq;
        x += ejd * oq;
        e += oq;
    }
}





pub fn khf(
    g: &VisualizerState, bj: usize, c: i32,
    ana: f32, abo: f32,
) -> RainEffect {
    if !g.jr { return RainEffect::Cq; }
    let aur = g.btt.len();
    if bj >= aur { return RainEffect::Cq; }

    let (cx, ae) = (g.yv, g.uq);

    
    
    
    if g.ev == 6 && g.edd > 0 && g.edc > 0 {
        let xu = bj as i32 * g.oy + g.oy / 2;

        
        let amr = xu - g.fla;
        let aio = c - g.flb;

        if amr >= 0 && amr < g.edd as i32
            && aio >= 0 && aio < g.edc as i32
        {
            let cfz = crate::logo_bitmap::AY_;
            let cfy = crate::logo_bitmap::BL_;
            
            let fld = (amr as u32 * cfz as u32 / g.edd) as usize;
            let fle = (aio as u32 * cfy as u32 / g.edc) as usize;

            if fld < cfz && fle < cfy {
                let il = crate::logo_bitmap::djc(fld, fle);
                let q = (il >> 24) & 0xFF;
                let oc = (il >> 16) & 0xFF;
                let bah = (il >> 8) & 0xFF;
                let ue = il & 0xFF;

                
                let kt = (oc.am(bah).am(ue)) as u8;
                if q > 30 && kt > 15 {
                    
                    
                    let qnf = 140u8 + (kt / 4);
                    let kco = (ana * 60.0).v(60.0) as u8;
                    let btk = qnf.akq(kco).v(230);

                    
                    let mut apb: u8 = 0;
                    {
                        let dx = (xu - cx) as f32;
                        let bg = (c - ae) as f32;
                        let la = libm::bon(dx * dx + bg * bg);
                        let exr = libm::dhb(la - g.bws);
                        if exr < 35.0 {
                            let can = (1.0 - g.bws / 500.0).am(0.0);
                            let ab = (1.0 - exr / 35.0) * can;
                            apb = (ab * 120.0).v(120.0) as u8;
                        }
                    }

                    
                    let tq = if kt > 100 { (kt - 60) / 2 } else { 0 };

                    return RainEffect {
                        tq,
                        eo: 128,
                        bst: 10,
                        apb,
                        tp: 0,
                        aug: 0,
                        amv: 0,
                        atv: 0,
                        axo: if kt > 180 { kt / 3 } else { 0 },
                        ys: 0,
                        ata: 0,
                        zc: 0,
                        ejq: oc as u8,
                        ejn: bah as u8,
                        ejl: ue as u8,
                        dcm: btk,
                    };
                }
            }
        }

        
        let amr = xu - g.fla;
        let aio = c - g.flb;
        let shq = if amr < 0 { -amr }
            else if amr >= g.edd as i32 { amr - g.edd as i32 + 1 }
            else { 0 };
        let shv = if aio < 0 { -aio }
            else if aio >= g.edc as i32 { aio - g.edc as i32 + 1 }
            else { 0 };
        let dqm = shq.am(shv);
        if dqm > 0 && dqm < 40 {
            let tp = ((1.0 - dqm as f32 / 40.0) * 50.0) as u8;
            return RainEffect { tp, ..RainEffect::Cq };
        }

        return RainEffect::Cq;
    }

    
    
    

    
    let jlu = if g.ev == 13 { g.gfq } else { 0.0 };

    
    let mut apb: u8 = 0;
    {
        let dx = (bj as i32 * g.oy + g.oy / 2 - cx) as f32;
        let bg = (c - ae) as f32;
        let la = libm::bon(dx * dx + bg * bg);
        let exr = libm::dhb(la - g.bws);
        if exr < 35.0 {
            let can = (1.0 - g.bws / 500.0).am(0.0);
            let ab = (1.0 - exr / 35.0) * can;
            apb = (ab * 120.0).v(120.0) as u8;
        }
    }

    
    let buw = &g.btt[bj];
    for &(dnq, dnp, qbr, hj, xxe) in buw.iter() {
        if c >= dnq && c <= dnp {
            let dqm = (c - (dnq + dnp) / 2).gp();
            let iv = (dnp - dnq) / 2;
            let tgc = if iv > 0 {
                1.0 - (dqm as f32 / iv as f32).v(1.0)
            } else { 1.0 };
            let tq = (80.0 + tgc * 175.0 * (hj as f32 / 255.0)) as u8;

            
            let jxs = (g.elj as i32 - g.elk as i32).am(1);
            let eo = ((xxe as i32 - g.elk as i32) * 255 / jxs).am(0).v(255) as u8;

            
            let (uj, wv) = g.anc[bj];
            let aug = if wv > uj {
                let kni = (uj + wv) / 2;
                let ksq = (c - kni).gp() as f32;
                let wp = ((wv - uj) / 2) as f32;
                if wp > 0.0 {
                    let bb = (ksq / wp).v(1.0);
                    (bb * bb * 255.0) as u8
                } else { 0 }
            } else { 0 };

            
            let ftt = (bj as i32 * g.oy - g.eyt).gp();
            let ftu = (c - g.dcd).gp();
            let pmf = ftt + ftu;
            let amv = if pmf < 60 {
                ((1.0 - pmf as f32 / 60.0) * 255.0) as u8
            } else { 0 };

            
            let atv = if wv > uj {
                let ab = (c - uj) as f32 / (wv - uj).am(1) as f32;
                let owh = (0.5 - ab).gp() * 2.0; 
                (owh * owh * 100.0) as u8
            } else { 0 };

            
            let axo = (tq as u16 * eo as u16 / 512).v(200) as u8;

            
            let ata = if wv > uj {
                let dx = bj as i32 * g.oy + g.oy / 2 - cx;
                let bg = c - ae;
                let m = libm::bon((dx * dx + bg * bg) as f32);
                let djl = ((wv - uj) as f32 / 2.0).am(1.0);
                let ab = (1.0 - m / djl).am(0.0);
                (ab * ab * 180.0) as u8
            } else { 0 };

            return RainEffect {
                tq, eo,
                bst: (tq / 4).v(80),
                apb,
                tp: 0,
                aug, amv, atv, axo,
                ys: 0,
                ata,
                zc: 0,
                ejq: if jlu > 0.01 { 255 } else { 0 },
                ejn: 0,
                ejl: 0,
                dcm: (jlu * 220.0).v(220.0) as u8,
            };
        }
    }

    
    let (uj, wv) = g.anc[bj];
    if uj >= 0 && c >= uj && c <= wv {
        let tq = 20u8;
        let jxs = (g.elj as i32 - g.elk as i32).am(1);
        let eo = 128u8;

        let aug = {
            let kni = (uj + wv) / 2;
            let ksq = (c - kni).gp() as f32;
            let wp = ((wv - uj) / 2) as f32;
            if wp > 0.0 {
                let bb = (ksq / wp).v(1.0);
                (bb * bb * 200.0) as u8
            } else { 0 }
        };

        let ata = {
            let dx = bj as i32 * g.oy + g.oy / 2 - cx;
            let bg = c - ae;
            let m = libm::bon((dx * dx + bg * bg) as f32);
            let djl = ((wv - uj) as f32 / 2.0).am(1.0);
            let ab = (1.0 - m / djl).am(0.0);
            (ab * ab * 120.0) as u8
        };

        return RainEffect {
            tq, eo, bst: 5,
            apb, tp: 0,
            aug, amv: 0, atv: 0, axo: 0,
            ys: 0, ata, zc: 0,
            ejq: if jlu > 0.01 { 255 } else { 0 },
            ejn: 0,
            ejl: 0,
            dcm: (jlu * 180.0).v(180.0) as u8,
        };
    }

    
    if uj >= 0 {
        let lrf = if c < uj { uj - c } else if c > wv { c - wv } else { 0 };
        if lrf > 0 && lrf < 60 {
            let tp = ((1.0 - lrf as f32 / 60.0) * 80.0) as u8;
            return RainEffect {
                tq: 0, eo: 128, bst: 0,
                apb, tp, aug: 0, amv: 0, atv: 0,
                axo: 0, ys: 0, ata: 0, zc: 0,
                ejq: 0, ejn: 0, ejl: 0, dcm: 0,
            };
        }
    }

    
    if c > g.bsj && c < g.dlt {
        let ab = (c - g.bsj) as f32 / (g.dlt - g.bsj) as f32;
        let zc = ((1.0 - ab) * (1.0 - ab) * 120.0) as u8;
        return RainEffect {
            tq: 0, eo: 128, bst: 0,
            apb, tp: 0, aug: 0, amv: 0, atv: 0,
            axo: 0, ys: 0, ata: 0, zc,
            ejq: 0, ejn: 0, ejl: 0, dcm: 0,
        };
    }

    
    if c > g.dvw && c < g.dvw + 200 {
        let y = bj as i32 * g.oy + g.oy / 2;
        let dx = (y - cx).gp();
        if dx < 80 && bj % 4 == 0 {
            let ozr = (1.0 - dx as f32 / 80.0) * (1.0 - (c - g.dvw) as f32 / 200.0);
            if ozr > 0.05 {
                let ys = (ozr * 150.0).v(200.0) as u8;
                return RainEffect {
                    tq: 0, eo: 128, bst: 0,
                    apb, tp: 0, aug: 0, amv: 0, atv: 0,
                    axo: 0, ys, ata: 0, zc: 0,
                    ejq: 0, ejn: 0, ejl: 0, dcm: 0,
                };
            }
        }
    }

    
    if apb > 0 {
        return RainEffect { apb, ..RainEffect::Cq };
    }

    RainEffect::Cq
}





pub fn lmn(
    bdm: u8, bji: u8, cdd: u8,
    tq: u8, eo: u8, apb: u8,
    aug: u8, amv: u8,
    atv: u8, axo: u8, ys: u8, ata: u8, zc: u8,
    rf: f32, abo: f32,
    ato: f32, aee: f32, vs: f32, axg: f32,
    aim: u8,
) -> (u8, u8, u8) {
    let mut m = bdm as f32;
    let mut at = bji as f32;
    let mut o = cdd as f32;

    
    let uxy = tq > 80;
    let dsa = ata > 20 || tq > 40;

    
    let eoq = eo as f32 / 255.0;

    
    
    
    if zc > 5 {
        let e = 1.0 - (zc as f32 / 255.0) * 0.6;
        m *= e; at *= e; o *= e;
    }

    
    
    
    if ys > 3 {
        let acj = ys as f32 / 255.0;
        
        let (oc, bah, ue) = ecc(aim, 0.3);
        m = (m + acj * oc * 0.3).v(255.0);
        at = (at + acj * bah * 0.3).v(255.0);
        o = (o + acj * ue * 0.3).v(255.0);
    }

    
    
    
    
    if uxy {
        let ckq = tq as f32 / 255.0;

        
        let sit = 0.55 + 0.45 * eoq; 

        
        let gwq = ckq * sit;
        m = (m * (1.0 - gwq) + 255.0 * gwq).v(255.0);
        at = (at * (1.0 - gwq) + 255.0 * gwq).v(255.0);
        o = (o * (1.0 - gwq) + 255.0 * gwq).v(255.0);

        
        let (oc, bah, ue) = ecc(aim, eoq);
        let cuq = 0.15 * ckq;
        m = (m * (1.0 - cuq) + oc * cuq).v(255.0);
        at = (at * (1.0 - cuq) + bah * cuq).v(255.0);
        o = (o * (1.0 - cuq) + ue * cuq).v(255.0);

        
        if atv > 0 {
            let ddw = 1.0 - (atv as f32 / 255.0) * 0.3;
            m *= ddw; at *= ddw; o *= ddw;
        }
    }
    
    
    
    
    
    else if ata > 20 {
        let crl = (ata as f32 - 20.0) / 235.0;
        let crl = crl * crl; 

        
        
        
        let mfk = 0.3 + 0.7 * eoq;

        
        
        let (oc, bah, ue) = ecc(aim, eoq);

        
        let ugg = oc * mfk;
        let ugf = bah * mfk;
        let uge = ue * mfk;

        
        let btk = (crl * 0.80).v(0.80);
        m = (m * (1.0 - btk) + ugg * btk).v(255.0);
        at = (at * (1.0 - btk) + ugf * btk).v(255.0);
        o = (o * (1.0 - btk) + uge * btk).v(255.0);

        
        if atv > 0 {
            let ddw = 1.0 - (atv as f32 / 255.0) * 0.5;
            m *= ddw; at *= ddw; o *= ddw;
        }

        
        if eoq > 0.8 {
            let lnq = (eoq - 0.8) / 0.2 * 30.0;
            m = (m + lnq).v(255.0);
            at = (at + lnq).v(255.0);
            o = (o + lnq).v(255.0);
        }
    }
    
    
    
    else if tq > 0 {
        let ckq = tq as f32 / 255.0;
        let (oc, bah, ue) = ecc(aim, eoq * 0.5 + 0.3);
        let bma = ckq * 0.5;
        m = (m + oc * bma).v(255.0);
        at = (at + bah * bma).v(255.0);
        o = (o + ue * bma).v(255.0);
    }

    
    
    
    if aug > 120 {
        let ggp = (aug as f32 - 120.0) / 135.0;
        let gwr = ggp * ggp;
        m = (m * (1.0 - gwr) + 252.0 * gwr).v(255.0);
        at = (at * (1.0 - gwr) + 255.0 * gwr).v(255.0);
        o = (o * (1.0 - gwr) + 252.0 * gwr).v(255.0);
    }

    
    
    
    if amv > 30 {
        let cud = (amv as f32 - 30.0) / 225.0;
        let cud = cud * cud;
        
        let (oc, bah, ue) = ecc(aim, 0.9);
        m = (m + cud * (200.0 + oc * 0.2)).v(255.0);
        at = (at + cud * (220.0 + bah * 0.1)).v(255.0);
        o = (o + cud * (200.0 + ue * 0.2)).v(255.0);
    }

    
    
    
    if axo > 10 {
        let bl = axo as f32 / 255.0;
        let (oc, bah, ue) = ecc(aim, 0.5);
        m = (m + bl * oc * 0.15).v(255.0);
        at = (at + bl * bah * 0.15).v(255.0);
        o = (o + bl * ue * 0.15).v(255.0);
    }

    
    
    
    if apb > 0 {
        let pc = apb as f32 / 255.0;
        let (oc, bah, ue) = ecc(aim, 0.6);
        m = (m + pc * oc * 0.2).v(255.0);
        at = (at + pc * bah * 0.2).v(255.0);
        o = (o + pc * ue * 0.2).v(255.0);
    }

    
    
    
    if dsa && abo > 0.03 {
        let jfe = ato.am(aee).am(vs).am(axg);
        if jfe > 0.10 {
            let hj = (abo * 0.8 + 0.1).v(0.55);

            
            let qmp = if ato >= jfe - 0.05 {
                0.1f32
            } else if aee >= jfe - 0.05 {
                0.35
            } else if vs >= jfe - 0.05 {
                0.65
            } else {
                0.9
            };

            
            let uor = qmp * 0.6 + eoq * 0.4;
            let (agd, ejs, bov) = ecc(aim, uor);

            let xg = 1.0 + rf * 0.5;
            let btk = (hj * xg).v(0.65);
            m = (m * (1.0 - btk) + agd * btk).v(255.0);
            at = (at * (1.0 - btk) + ejs * btk).v(255.0);
            o = (o * (1.0 - btk) + bov * btk).v(255.0);

            
            if rf > 0.5 {
                let lhj = (rf - 0.5) * 45.0;
                m = (m + lhj).v(255.0);
                at = (at + lhj).v(255.0);
                o = (o + lhj).v(255.0);
            }
        }
    }

    
    
    
    if dsa {
        (m.v(255.0) as u8, at.v(255.0) as u8, o.v(255.0) as u8)
    } else if aim == 0 {
        
        let hku = at as u8;
        let lwv = (m as u8).v(hku);
        let kbp = (o as u8).v(hku);
        (lwv, hku, kbp)
    } else {
        
        let (oc, bah, ue) = ecc(aim, 0.15);
        let cuq = 0.18f32;
        let lwv = (m * (1.0 - cuq) + oc * cuq).v(255.0) as u8;
        let hku = (at * (1.0 - cuq) + bah * cuq).v(255.0) as u8;
        let kbp = (o * (1.0 - cuq) + ue * cuq).v(255.0) as u8;
        (lwv, hku, kbp)
    }
}





#[inline]
pub fn nez(g: &VisualizerState, bj: usize) -> u8 {
    if bj < g.anc.len() {
        let (uj, wv) = g.anc[bj];
        if uj >= 0 && wv > uj {
            let dlx = (wv - uj) as u32;
            let wpt = (45 + (55 * dlx / 400).v(55)) as u8;
            return 100u8.ao(100 - wpt);
        }
    }
    100
}
