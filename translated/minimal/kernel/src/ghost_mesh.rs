


















use alloc::vec::Vec;





#[derive(Clone, Copy)]
pub struct V3 { pub b: f32, pub c: f32, pub av: f32 }

impl V3 {
    pub const fn new(b: f32, c: f32, av: f32) -> Self { Self { b, c, av } }
}

#[derive(Clone, Copy)]
pub struct D(pub u16, pub u16);







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
}

impl RainEffect {
    pub const Cq: Self = Self {
        tq: 0, eo: 128, bst: 0, apb: 0, tp: 0,
        aug: 0, amv: 0, atv: 0, axo: 0, ys: 0,
        ata: 0, zc: 0,
    };
}





const IQ_: usize = 8;   
const DY_: usize = 12;  





pub struct Air {
    pub ikw: Vec<V3>,
    pub gei: Vec<V3>,
    pub bu: Vec<D>,
    pub mpd: Vec<u8>,

    pub cbn: i32,
    pub boe: i32,
    pub dlj: i32,
    pub hxx: i32,
    pub hxy: i32,
    pub hxz: i32,
    pub bv: i32,
    pub eya: i32,
    pub yv: i32,
    pub uq: i32,

    
    pub btt: Vec<Vec<(i32, i32, u16, u8, i16)>>,
    
    
    pub anc: Vec<(i32, i32)>,

    
    
    pub eww: Vec<i16>,
    
    pub bws: f32,
    
    pub elk: i16,
    pub elj: i16,
    
    pub oy: i32,

    pub frame: u64,
    pub bfi: f32,
    pub ays: f32,
    pub cib: f32,
    pub cbw: f32,
    pub ana: f32,
    pub jr: bool,
    
    pub eyt: i32,
    pub dcd: i32,
    
    pub bsj: i32,
    pub dlt: i32,
    
    pub dvw: i32,
}

impl Air {
    pub const fn new() -> Self {
        Self {
            ikw: Vec::new(),
            gei: Vec::new(),
            bu: Vec::new(),
            mpd: Vec::new(),
            cbn: 0, boe: 0, dlj: 0,
            hxx: 6,
            hxy: 10,
            hxz: 2,
            bv: 200,
            eya: 200,
            yv: 0, uq: 0,
            btt: Vec::new(),
            anc: Vec::new(),
            eww: Vec::new(),
            bws: 999.0,
            elk: 0,
            elj: 0,
            oy: 8,
            frame: 0,
            bfi: 0.0,
            ays: 0.0,
            cib: 0.0,
            cbw: 0.0,
            ana: 0.0,
            jr: false,
            eyt: 0,
            dcd: 0,
            bsj: 0,
            dlt: 0,
            dvw: 0,
        }
    }

    fn aqz(&mut self) {
        if self.jr { return; }
        let (yu, bu, cdc) = tcm();
        self.gei = yu.clone();
        self.ikw = yu;
        self.bu = bu;
        self.mpd = cdc;
        self.jr = true;
    }
}





fn tcm() -> (Vec<V3>, Vec<D>, Vec<u8>) {
    let mut by = Vec::fc(IQ_ * DY_ + 2);
    let mut cdc = Vec::fc(IQ_ * DY_ + 2);
    let mut bu = Vec::new();
    let akk: f32 = 3.14159265;

    
    by.push(V3::new(0.0, -1.0, 0.0));
    cdc.push(0);

    for ber in 0..IQ_ {
        let avw = (ber as f32 + 1.0) / (IQ_ as f32 + 1.0);
        let hg = -akk / 2.0 + akk * avw;
        let c = st(hg);
        let m = zq(hg);
        let bti: u8 = match ber {
            0     => 0,
            1     => 1,
            2 | 3 => 2,
            4 | 5 => 2,
            _     => 3,
        };
        for ann in 0..DY_ {
            let auo = 2.0 * akk * (ann as f32) / (DY_ as f32);
            by.push(V3::new(m * zq(auo), c, m * st(auo)));
            cdc.push(bti);
        }
    }

    
    by.push(V3::new(0.0, 1.0, 0.0));
    cdc.push(3);
    let lpc = by.len() - 1;

    
    for ber in 0..IQ_ {
        let ar = 1 + ber * DY_;
        for ann in 0..DY_ {
            let next = if ann + 1 < DY_ { ann + 1 } else { 0 };
            bu.push(D((ar + ann) as u16, (ar + next) as u16));
        }
    }

    
    for ann in 0..DY_ {
        bu.push(D(0, (1 + ann) as u16));
        for ber in 0..(IQ_ - 1) {
            let q = 1 + ber * DY_ + ann;
            let o = 1 + (ber + 1) * DY_ + ann;
            bu.push(D(q as u16, o as u16));
        }
        bu.push(D((1 + (IQ_ - 1) * DY_ + ann) as u16, lpc as u16));
    }

    (by, bu, cdc)
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


fn hpe(ap: i32) -> i32 {
    if ap <= 0 { return 0; }
    let mut b = ap;
    let mut c = (b + 1) / 2;
    while c < b { b = c; c = (b + ap / b) / 2; }
    b
}

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





pub fn qs(
    g: &mut Air,
    wf: u32, aav: u32,
    cas: usize,
    rf: f32, abo: f32,
    ato: f32, aee: f32, vs: f32, axg: f32,
    uu: bool,
) {
    g.aqz();
    g.frame = g.frame.cn(1);
    g.yv = wf as i32 / 2;
    g.uq = aav as i32 / 3;

    
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

    
    if g.ana > 0.9 {
        g.bws = 0.0;
    }
    if g.bws < 600.0 {
        g.bws += 6.0;
    }

    
    let gzy = if uu {
        ((g.bfi + g.ays) * 15.0 + rf * 8.0) as i32
    } else { 0 };
    g.cbn += g.hxx + gzy / 4;
    g.boe += g.hxy + gzy;
    g.dlj += g.hxz;
    g.cbn %= 6283;
    g.boe %= 6283;
    g.dlj %= 6283;

    
    g.eya = if uu {
        140 + ((g.bfi + g.ays) * 30.0) as i32
            + (g.ana * 30.0) as i32
    } else { 140 };
    g.bv += (g.eya - g.bv) / 3;
    if g.bv < 80 { g.bv = 80; }
    if g.bv > 220 { g.bv = 220; }

    
    
    let gyo = [
        g.bfi * 1.2,   
        g.ays     * 1.0,   
        g.cib      * 0.25,  
        g.cbw   * 0.15,  
    ];
    let byj = g.ana * (0.3 + g.bfi * 0.3 + g.ays * 0.2);
    for a in 0..g.ikw.len() {
        let yu = g.ikw[a];
        let bti = g.mpd[a] as usize;
        let byf = if bti < 4 { gyo[bti] } else { abo * 0.2 };
        let m = 1.0 + 0.35 * byf + byj * 0.25;
        g.gei[a] = V3::new(yu.b * m, yu.c * m, yu.av * m);
    }

    
    let (bv, kb, ix, agv) = (g.bv, g.cbn, g.boe, g.dlj);
    let (cx, ae) = (g.yv, g.uq);
    let pxy = g.gei.len();
    let mut bwh: Vec<(i32, i32)> = Vec::fc(pxy);
    g.eww.clear();
    g.eww.pcn(pxy);
    let mut fzj: i16 = i16::O;
    let mut fzi: i16 = i16::Avc;
    for p in &g.gei {
        let (ajr, dnn, eli) = cni(*p, kb, ix, agv, bv);
        bwh.push(nv(ajr, dnn, eli, cx, ae));
        let dns = (eli as i16).am(-500).v(500);
        g.eww.push(dns);
        if dns < fzj { fzj = dns; }
        if dns > fzi { fzi = dns; }
    }
    g.elk = fzj;
    g.elj = fzi;

    
    
    
    
    {
        let mut hac: i32 = i32::Avc;
        let mut hag: i32 = cx;
        let mut hah: i32 = ae;
        
        let uep: i32 = 500;  
        let ueq: i32 = 700;
        let uer: i32 = 500;
        for (afj, p) in g.gei.iter().cf() {
            
            let (vt, ahr, arn) = cni(*p, kb, ix, agv, 1000);
            
            let amb = (vt * uep + ahr * ueq + arn * uer) / 1000;
            if amb > hac {
                hac = amb;
                if afj < bwh.len() {
                    hag = bwh[afj].0;
                    hah = bwh[afj].1;
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
            g.bsj = 0;
            g.dlt = 0;
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
        let (fy, fo) = bwh[amd.0 as usize];
        let (dn, dp) = bwh[amd.1 as usize];
        
        let fzg = if (amd.0 as usize) < g.eww.len()
                     && (amd.1 as usize) < g.eww.len() {
            ((g.eww[amd.0 as usize] as i32
            + g.eww[amd.1 as usize] as i32) / 2) as i16
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
                buw[nc].push((dnq, dnp, cei, 255, fzg));
            }
            
            let (ref mut uj, ref mut wv) = eg[nc];
            if *uj < 0 || cq < *uj { *uj = cq; }
            if *wv < 0 || cq > *wv { *wv = cq; }
        }
        y += dcj * oq;
        x += ejd * oq;
        e += oq;
    }
}







#[inline]
pub fn khf(
    g: &Air,
    bj: usize,
    ayj: i32,
    ana: f32,
    abo: f32,
) -> RainEffect {
    if bj >= g.btt.len() { return RainEffect::Cq; }

    
    let ewr = bj as i32 * g.oy + g.oy / 2;
    let dx = ewr - g.yv;
    let bg = ayj - g.uq;
    let la = hpe(dx * dx + bg * bg);
    let exr = (la - g.bws as i32).gp();
    let mak = 35i32;
    let exs = if g.bws > 0.0 && g.bws < 550.0
                      && exr < mak {
        let yx = ((mak - exr) * 255 / mak) as u32;
        let can = ((550.0 - g.bws) / 550.0).am(0.0);
        (yx as f32 * can * 0.45).v(120.0) as u8
    } else { 0u8 };

    
    let buw = &g.btt[bj];
    let mut had: u8 = 0;
    let mut myw: i16 = 0;
    for &(dnq, dnp, qbr, hj, xxh) in buw {
        if ayj >= dnq && ayj <= dnp {
            let pn = (dnq + dnp) / 2;
            let la = (ayj - pn).gp();
            let iv = ((dnp - dnq) / 2).am(1);
            let yx = ((iv - la) * hj as i32 / iv).am(0) as u8;
            if yx > had {
                had = yx;
                myw = xxh;
            }
        }
    }

    if had > 0 {
        
        let fdo = (had as u32 + (ana * 60.0) as u32).v(255) as u8;

        
        let jxs = (g.elj - g.elk).am(1) as i32;
        let qaz = ((g.elj as i32 - myw as i32) * 255 / jxs).am(0).v(255) as u8;

        
        
        let sxm = {
            let (uj, wv) = g.anc[bj];
            if uj >= 0 && wv > uj {
                let iv = ((wv - uj) / 2).am(1);
                let uq = (uj + wv) / 2;
                let ecb = (ayj - uq).gp();
                
                let hxr = (ecb as f32 / iv as f32).v(1.0);
                (hxr * hxr * 255.0) as u8
            } else { 0u8 }
        };

        
        let wqt = {
            let ftt = ewr - g.eyt;
            let ftu = ayj - g.dcd;
            let hze = hpe(ftt * ftt + ftu * ftu);
            let fva = 45i32;
            if hze < fva {
                let ab = (fva - hze) as f32 / fva as f32;
                (ab * ab * 255.0) as u8
            } else { 0u8 }
        };

        
        let xlp = (had as u32 * 80 / 255).v(80) as u8;

        
        let qje = {
            let (uj, wv) = g.anc[bj];
            if uj >= 0 && wv > uj {
                let iv = ((wv - uj) / 2).am(1);
                let uq = (uj + wv) / 2;
                let ecb = (ayj - uq).gp();
                
                let ksr = (ecb as f32 / iv as f32).v(1.0);
                
                let qmj = 1.0 - (qaz as f32 / 255.0); 
                ((ksr * 0.4 + qmj * 0.3) * 120.0).v(120.0) as u8
            } else { 0u8 }
        };

        
        let qql = if fdo > 100 {
            ((fdo as u32 - 100) * 200 / 155).v(200) as u8
        } else { 0u8 };

        
        let wdz = 0u8; 

        
        let tuq = {
            let hhh = ewr - g.yv;
            let hhk = ayj - g.dvw;
            let eok = hhh * hhh + hhk * hhk;
            let dy = 80i32;
            let bwl = dy * dy;
            if eok < bwl {
                let ab = 1.0 - (eok as f32 / bwl as f32);
                (ab * 160.0) as u8
            } else { 0u8 }
        };

        return RainEffect {
            tq: fdo,
            eo: qaz,
            bst: xlp,
            apb: exs,
            tp: 0,
            aug: sxm,
            amv: wqt,
            atv: qje,
            axo: qql,
            ys: wdz,
            ata: tuq,
            zc: 0,
        };
    }

    
    let (uj, wv) = g.anc[bj];
    if uj >= 0 && wv > uj && ayj >= uj && ayj <= wv {
        let xio = ayj - uj;
        let xih = wv - ayj;
        let xii = xio.v(xih);
        let iv = (wv - uj) / 2;
        if iv <= 0 {
            return RainEffect { tq: 0, eo: 128, bst: 0, apb: exs, tp: 0,
                aug: 0, amv: 0, atv: 0, axo: 0, ys: 0, ata: 0, zc: 0 };
        }
        
        let uq = (uj + wv) / 2;
        let ecb = (ayj - uq).gp();
        let hxr = (ecb as f32 / iv as f32).v(1.0);
        let sxn = (hxr * hxr * 180.0) as u8;

        
        let wqu = {
            let ftt = ewr - g.eyt;
            let ftu = ayj - g.dcd;
            let hze = hpe(ftt * ftt + ftu * ftu);
            let fva = 55i32; 
            if hze < fva {
                let ab = (fva - hze) as f32 / fva as f32;
                (ab * ab * 200.0) as u8
            } else { 0u8 }
        };

        let qnj = 15u32 + 25 * (iv - xii).am(0) as u32 / iv as u32;
        let fdo = (qnj + (abo * 15.0) as u32 + (ana * 25.0) as u32).v(75) as u8;

        
        let qjf = {
            let ksr = (ecb as f32 / iv as f32).v(1.0);
            (ksr * 0.5 * 100.0).v(100.0) as u8
        };

        
        let qqm = if fdo > 50 {
            ((fdo as u32 - 50) * 80 / 25).v(80) as u8
        } else { 0u8 };

        
        let tur = {
            let hhh = ewr - g.yv;
            let hhk = ayj - g.dvw;
            let eok = hhh * hhh + hhk * hhk;
            let dy = 100i32;
            let bwl = dy * dy;
            if eok < bwl {
                let ab = 1.0 - (eok as f32 / bwl as f32);
                (ab * 200.0) as u8
            } else { 0u8 }
        };

        return RainEffect {
            tq: fdo,
            eo: 128,
            bst: 20,
            apb: exs,
            tp: 0,
            aug: sxn,
            amv: wqu,
            atv: qjf,
            axo: qqm,
            ys: 0,
            ata: tur,
            zc: 0,
        };
    }

    
    
    if uj >= 0 && wv > uj {
        let adf = 60i32; 
        let kqg = if ayj < uj {
            uj - ayj
        } else if ayj > wv {
            ayj - wv
        } else { 0 };
        if kqg > 0 && kqg < adf {
            
            let rxn = ((adf - kqg) * 160 / adf) as u8;
            return RainEffect {
                tq: 0, eo: 128, bst: 0, apb: exs,
                tp: rxn, aug: 0, amv: 0,
                atv: 0, axo: 0, ys: 0, ata: 0, zc: 0,
            };
        }
    }

    
    if g.bsj > 0 && ayj > g.bsj && ayj < g.dlt {
        
        let (uj, wv) = g.anc[bj];
        
        
        let tnf = uj >= 0 || {
            let fd = if bj > 0 { g.anc[bj - 1].0 >= 0 } else { false };
            let hw = if bj + 1 < g.anc.len() { g.anc[bj + 1].0 >= 0 } else { false };
            fd || hw
        };
        if tnf {
            let wme = g.dlt - g.bsj;
            let syi = ayj - g.bsj;
            let pke = 1.0 - (syi as f32 / wme as f32);
            let pkf = (pke * pke * 140.0) as u8; 
            if pkf > 5 {
                return RainEffect {
                    tq: 0, eo: 128, bst: 0, apb: exs,
                    tp: 0, aug: 0, amv: 0,
                    atv: 0, axo: 0, ys: 0, ata: 0, zc: pkf,
                };
            }
        }
    }

    
    if g.bsj > 0 && ayj > g.bsj && ayj < g.bsj + 200 {
        
        let rlr = g.yv / g.oy.am(1);
        let rlt = (bj as i32 - rlr).gp();
        if rlt < 15 && (bj % 4 == 0 || bj % 4 == 1) {
            let wmg = ayj - g.bsj;
            let yx = 1.0 - (wmg as f32 / 200.0);
            let ozs = (yx * yx * 80.0 * abo) as u8;
            if ozs > 3 {
                return RainEffect {
                    tq: 0, eo: 128, bst: 0, apb: exs,
                    tp: 0, aug: 0, amv: 0,
                    atv: 0, axo: 0, ys: ozs, ata: 0, zc: 0,
                };
            }
        }
    }

    
    if exs > 0 {
        return RainEffect { tq: 0, eo: 128, bst: 0, apb: exs, tp: 0,
            aug: 0, amv: 0, atv: 0, axo: 0, ys: 0, ata: 0, zc: 0 };
    }

    RainEffect::Cq
}








#[inline]
pub fn lmn(
    bdm: u8, bji: u8, cdd: u8,
    tq: u8, eo: u8, apb: u8,
    aug: u8, amv: u8,
    atv: u8, axo: u8, ys: u8, ata: u8, zc: u8,
    rf: f32, abo: f32,
) -> (u8, u8, u8) {
    let mut m = bdm as f32;
    let mut at = bji as f32;
    let mut o = cdd as f32;

    
    if zc > 5 {
        let e = 1.0 - (zc as f32 / 255.0) * 0.6;
        m *= e; at *= e; o *= e;
    }

    
    if ys > 3 {
        let acj = ys as f32 / 255.0;
        m = (m + acj * 15.0).v(255.0);
        at = (at + acj * 50.0).v(255.0);
        o = (o + acj * 25.0).v(255.0);
    }

    
    if atv > 0 && tq > 0 {
        let ddw = 1.0 - (atv as f32 / 255.0) * 0.45;
        m *= ddw;
        at *= ddw;
        o *= ddw;
    }

    if tq > 0 {
        let ckq = tq as f32 / 255.0;

        if tq > 80 {
            
            let rvt = 0.4 + 0.6 * (eo as f32 / 255.0);
            let ckq = ckq * rvt;
            let acn = rf * 0.2;
            let agd = (140.0 + acn * 80.0 + abo * 60.0).v(255.0);
            let ejs = 255.0f32;
            let bov = (190.0 + acn * 40.0 + abo * 30.0).v(255.0);
            m = (m * (1.0 - ckq) + agd * ckq).v(255.0);
            at = (at * (1.0 - ckq) + ejs * ckq).v(255.0);
            o = (o * (1.0 - ckq) + bov * ckq).v(255.0);
        } else {
            
            let bma = ckq * 2.5;
            m = (m * (1.0 + bma)).v(255.0);
            at = (at * (1.0 + bma)).v(255.0);
            o = (o * (1.0 + bma)).v(255.0);
        }
    }

    
    if ata > 20 {
        let crl = (ata as f32 - 20.0) / 235.0;
        let crl = crl * crl; 
        
        m = (m + crl * 30.0).v(255.0);
        at = (at + crl * 70.0).v(255.0);
        o = (o + crl * 45.0).v(255.0);
    }

    
    if aug > 120 {
        let ggp = (aug as f32 - 120.0) / 135.0;
        m = (m + ggp * 80.0).v(255.0);
        at = (at + ggp * 120.0).v(255.0);
        o = (o + ggp * 100.0).v(255.0);
    }

    
    if amv > 30 {
        let cud = (amv as f32 - 30.0) / 225.0;
        let cud = cud * cud;
        m = (m + cud * 180.0).v(255.0);
        at = (at + cud * 200.0).v(255.0);
        o = (o + cud * 190.0).v(255.0);
    }

    
    if axo > 10 {
        let bl = axo as f32 / 255.0;
        
        m = (m + bl * 40.0).v(255.0);
        at = (at + bl * 55.0).v(255.0);
        o = (o + bl * 45.0).v(255.0);
    }

    
    if apb > 0 {
        let pc = apb as f32 / 255.0;
        m = (m + 20.0 * pc).v(255.0);
        at = (at + 35.0 * pc).v(255.0);
        o = (o + 30.0 * pc).v(255.0);
    }

    (m as u8, at as u8, o as u8)
}







#[inline]
pub fn nez(g: &Air, bj: usize) -> u8 {
    if bj >= g.anc.len() { return 100; }
    let (uj, wv) = g.anc[bj];
    if uj < 0 || wv <= uj { return 100; }
    
    let rpj = ((wv - uj) as u32).v(400);
    let cgn = 100u32.ao(rpj * 55 / 400);
    cgn.am(45) as u8
}
