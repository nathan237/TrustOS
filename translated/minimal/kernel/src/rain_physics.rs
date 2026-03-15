

































#[inline]
fn mcs(y: f32, x: f32, cx: f32, ae: f32, m: f32) -> f32 {
    let dx = y - cx;
    let bg = x - ae;
    libm::bon(dx * dx + bg * bg) - m
}



#[inline]
fn hzc(y: f32, x: f32, ax: f32, bga: f32, bx: f32, je: f32, m: f32) -> f32 {
    let ouv = y - ax;
    let ouw = x - bga;
    let ilc = bx - ax;
    let ild = je - bga;
    let mxh = ilc * ilc + ild * ild;
    let i = if mxh > 0.001 {
        let ab = (ouv * ilc + ouw * ild) / mxh;
        if ab < 0.0 { 0.0 } else if ab > 1.0 { 1.0 } else { ab }
    } else {
        0.0
    };
    let dx = ouv - ilc * i;
    let bg = ouw - ild * i;
    libm::bon(dx * dx + bg * bg) - m
}



#[inline]
fn fpr(q: f32, o: f32, eh: f32) -> f32 {
    let ohl = eh * 4.0;
    let bc = ohl - libm::dhb(q - o);
    let i = if bc > 0.0 { bc } else { 0.0 };
    let ef = if q < o { q } else { o };
    ef - i * i * 0.25 / ohl.am(0.001)
}





pub struct Yl {
    pub jr: bool,
    pub d: f32,
    pub i: f32,

    
    pub cux: f32,
    pub gvb: f32,
    pub fxl: f32,
    pub gva: f32,

    
    pub kgh: f32, pub gca: f32, pub enh: f32,  
    pub hcf: f32, pub imu: f32, pub gcb: f32,  
    pub hcg: f32, pub imv: f32, pub gcc: f32,  

    
    pub kei: f32, pub kej: f32, pub kek: f32, pub kel: f32, pub kem: f32,
    pub ken: f32, pub keo: f32, pub kep: f32, pub keq: f32, pub ker: f32,

    
    pub maw: f32, pub max: f32, pub may: f32, pub jna: f32, pub maz: f32,
    pub mba: f32, pub mbb: f32, pub mbc: f32, pub jnb: f32, pub mbd: f32,

    
    pub jcg: f32,
    pub lhr: f32,
    pub lhs: f32,

    
    pub mnb: f32,
    pub mnc: f32,
    pub mnd: f32,
    pub mna: f32,

    
    pub ihh: f32,
    pub frame: u64,
}

impl Yl {
    pub const fn new() -> Self {
        Self {
            jr: false,
            d: 0.0, i: 0.0,
            cux: 0.0, gvb: 0.0, fxl: 0.0, gva: 0.0,
            kgh: 0.0, gca: 0.0, enh: 0.0,
            hcf: 0.0, imu: 0.0, gcb: 0.0,
            hcg: 0.0, imv: 0.0, gcc: 0.0,
            kei: 0.0, kej: 0.0, kek: 0.0, kel: 0.0, kem: 0.0,
            ken: 0.0, keo: 0.0, kep: 0.0, keq: 0.0, ker: 0.0,
            maw: 0.0, max: 0.0, may: 0.0, jna: 0.0, maz: 0.0,
            mba: 0.0, mbb: 0.0, mbc: 0.0, jnb: 0.0, mbd: 0.0,
            jcg: 0.0, lhr: 0.0, lhs: 0.0,
            mnb: 0.0, mnc: 0.0, mnd: 0.0, mna: 0.0,
            ihh: 0.0, frame: 0,
        }
    }
}







pub fn yxt(e: &mut Yl, d: u32, i: u32) {
    let d = d as f32;
    let i = i as f32;
    e.d = d;
    e.i = i;

    
    
    e.cux   = d * 0.30;
    e.gvb = i * 0.48;      
    e.fxl = i * 0.78;      
    e.gva     = d * 0.010;     

    
    
    e.kgh = e.cux;
    e.gca = i * 0.32;
    e.enh  = d * 0.080;         

    
    e.hcf = e.cux - d * 0.063;   
    e.imu = i * 0.38;
    e.gcb  = d * 0.050;                

    
    e.hcg = e.cux + d * 0.063;
    e.imv = i * 0.38;
    e.gcc  = d * 0.050;

    
    let mzs = e.gva * 0.6;
    
    e.kei = e.cux - mzs;
    e.kej = e.gvb - i * 0.02;
    e.kek = e.hcf + e.gcb * 0.3;
    e.kel = e.imu + e.gcb * 0.4;
    e.kem  = d * 0.005;                    
    
    e.ken = e.cux + mzs;
    e.keo = e.gvb - i * 0.02;
    e.kep = e.hcg - e.gcc * 0.3;
    e.keq = e.imv + e.gcc * 0.4;
    e.ker  = d * 0.005;

    
    e.maw = e.cux - e.gva * 0.3;
    e.max = e.fxl;
    e.may = e.cux - d * 0.025;
    e.jna = e.fxl + i * 0.025;
    e.maz  = d * 0.004;

    e.mba = e.cux + e.gva * 0.3;
    e.mbb = e.fxl;
    e.mbc = e.cux + d * 0.025;
    e.jnb = e.fxl + i * 0.025;
    e.mbd  = d * 0.004;

    
    e.jcg     = i * 0.82;
    e.lhr  = d * 0.08;
    e.lhs = d * 0.92;

    
    let adf = 60.0;
    e.mnb  = (e.hcf - e.gcb - adf).am(0.0);
    e.mnc = (e.hcg + e.gcc + adf).v(d);
    e.mnd   = (e.gca - e.enh - adf).am(0.0);
    e.mna   = (e.jna.am(e.jnb) + adf).v(i);

    e.jr = true;
}







fn iey(y: f32, x: f32, e: &Yl) -> f32 {
    
    let fbk = libm::st(e.ihh) * e.enh * 0.04;
    let fbl = libm::st(e.ihh * 0.7 + 1.0) * e.enh * 0.015;

    
    let xmj = hzc(
        y, x,
        e.cux, e.fxl,
        e.cux + fbk * 0.1, e.gvb + fbl * 0.2,
        e.gva,
    );

    
    let rw = mcs(y, x, e.kgh + fbk, e.gca + fbl, e.enh);
    let tx = mcs(y, x, e.hcf + fbk * 0.8, e.imu + fbl * 0.8, e.gcb);
    let der = mcs(y, x, e.hcg + fbk * 0.8, e.imv + fbl * 0.8, e.gcc);
    let qvz = fpr(rw, fpr(tx, der, 25.0), 25.0);

    
    let bl = hzc(
        y, x,
        e.kei + fbk * 0.1, e.kej + fbl * 0.2,
        e.kek + fbk * 0.8, e.kel + fbl * 0.8,
        e.kem,
    );
    let avi = hzc(
        y, x,
        e.ken + fbk * 0.1, e.keo + fbl * 0.2,
        e.kep + fbk * 0.8, e.keq + fbl * 0.8,
        e.ker,
    );
    let ket = fpr(bl, avi, 15.0);

    
    let vzl = hzc(y, x, e.maw, e.max, e.may, e.jna, e.maz);
    let ftg = hzc(y, x, e.mba, e.mbb, e.mbc, e.jnb, e.mbd);
    let wab = fpr(vzl, ftg, 10.0);

    
    let iex = fpr(xmj, qvz, 18.0);
    let iex = fpr(iex, ket, 12.0);
    fpr(iex, wab, 10.0)
}



fn xmf(y: f32, x: f32, e: &Yl) -> (f32, f32) {
    let cel = 3.0;
    let dx = iey(y + cel, x, e) - iey(y - cel, x, e);
    let bg = iey(y, x + cel, e) - iey(y, x - cel, e);
    let len = libm::bon(dx * dx + bg * bg);
    if len > 0.001 {
        (dx / len, bg / len)
    } else {
        (0.0, -1.0)
    }
}







pub struct RainInteraction {
    
    
    pub fzd: i32,
    
    pub kt: f32,
    
    pub cpl: i16,
    pub cwj: i16,
    pub cwi: i16,
    
    pub clp: bool,
    
    pub flk: bool,
}

impl RainInteraction {
    pub const Cq: Self = Self {
        fzd: 0,
        kt: 1.0,
        cpl: 0, cwj: 0, cwi: 0,
        clp: false,
        flk: false,
    };
}


pub fn qs(g: &mut Yl) {
    if !g.jr { return; }
    g.frame = g.frame.cn(1);
    
    g.ihh = (g.frame as f32) * 0.008;
}





pub fn zhb(e: &Yl, y: f32, x: f32) -> RainInteraction {
    if !e.jr {
        return RainInteraction::Cq;
    }

    
    let opc = y >= e.mnb && y <= e.mnc
                 && x >= e.mnd  && x <= e.mna;

    
    let odr = x >= e.jcg - 8.0
                    && y >= e.lhr && y <= e.lhs;

    if !opc && !odr {
        return RainInteraction::Cq;
    }

    
    
    
    if opc {
        let fai = iey(y, x, e);

        const FV_: f32  = 15.0;   
        const AWZ_: f32    = 8.0;    
        const BVH_: f32 = 8.0;    
        const Bjc: f32     = 45.0;   

        if fai < Bjc {
            
            if fai < -AWZ_ {
                let eo = (-fai - AWZ_).v(60.0);
                let tp = (eo / 60.0).v(0.80);  
                return RainInteraction {
                    fzd: 0,
                    kt: 0.20 + (1.0 - tp) * 0.3,  
                    cpl: -10,
                    cwj: 5,
                    cwi: -15,
                    clp: false,
                    flk: false,
                };
            }

            
            if libm::dhb(fai) <= FV_ {
                let (vt, ahr) = xmf(y, x, e);

                
                
                
                let eqm = vt * BVH_;

                
                let gtq = 1.0 - libm::dhb(fai) / FV_;
                let aaj = 1.3 + gtq * 0.7;  

                
                let btu = (20.0 + gtq * 30.0) as i16;
                let bmh = (30.0 + gtq * 40.0) as i16;
                let aiv = (25.0 + gtq * 30.0) as i16;

                return RainInteraction {
                    fzd: eqm as i32,
                    kt: aaj,
                    cpl: btu,
                    cwj: bmh,
                    cwi: aiv,
                    clp: true,
                    flk: false,
                };
            }

            
            let itw = (fai - FV_) / (Bjc - FV_);
            let itw = itw.am(0.0).v(1.0);

            
            
            let qwa = e.gca + e.enh * 0.6;
            let qem = x < e.gvb + 30.0;
            if x > qwa && qem && fai < 30.0 {
                let iry = 1.0 - itw;
                return RainInteraction {
                    fzd: 0,
                    kt: 1.1 + iry * 0.3,
                    cpl: (8.0 * iry) as i16,
                    cwj: (15.0 * iry) as i16,
                    cwi: (10.0 * iry) as i16,
                    clp: false,
                    flk: false,
                };
            }

            
            let qou = x > e.gca + e.enh * 0.3;
            if qou && fai < 35.0 {
                let zc = 0.12 * (1.0 - itw);
                return RainInteraction {
                    fzd: 0,
                    kt: 1.0 - zc,
                    cpl: 0,
                    cwj: 0,
                    cwi: 0,
                    clp: false,
                    flk: false,
                };
            }
        }
    }

    
    
    
    if odr {
        let jcf = x - e.jcg;  

        const FV_: f32 = 6.0;  

        
        if libm::dhb(jcf) < FV_ {
            let gtq = 1.0 - libm::dhb(jcf) / FV_;
            
            let jpx = libm::st(y * 0.05 + e.ihh * 3.0) * 0.3 + 0.7;
            let aaj = 1.4 + gtq * jpx * 0.6;
            return RainInteraction {
                fzd: 0,
                kt: aaj,
                cpl: -10,
                cwj: 10,
                cwi: 50,   
                clp: false,
                flk: true,
            };
        }

        
        if jcf > FV_ {
            let eo = (jcf - FV_).v(120.0) / 120.0;
            let tp = 0.12 + eo * 0.55;  
            return RainInteraction {
                fzd: 0,
                kt: 1.0 - tp,
                cpl: -25,
                cwj: -8,
                cwi: 25,   
                clp: false,
                flk: true,
            };
        }
    }

    RainInteraction::Cq
}
