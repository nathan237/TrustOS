







#[derive(Clone, Copy)]
pub struct V3 {
    pub b: f32,
    pub c: f32,
    pub av: f32,
}

#[derive(Clone, Copy)]
pub struct Anz {
    pub b: f32,
    pub c: f32,
}



pub use crate::math::{lz, rk, ahn};


#[inline(always)]
fn pzv(mut q: f32) -> f32 {
    const Eu: f32 = 3.14159265;
    const Yy: f32 = 6.2831853;
    while q > Eu { q -= Yy; }
    while q < -Eu { q += Yy; }
    q
}



#[inline(always)]
pub fn cxr(b: f32) -> f32 {
    if b < -6.0 { return 0.0; }
    if b > 0.0 { return 1.0; }
    
    let ab = 1.0 + b * 0.125; 
    let ab = if ab < 0.0 { 0.0 } else { ab };
    let ab = ab * ab; 
    let ab = ab * ab; 
    ab * ab           
}



#[inline(always)]
fn cmj(p: V3, q: f32) -> V3 {
    let r = rk(q);
    let e = lz(q);
    V3 { b: p.b * r + p.av * e, c: p.c, av: -p.b * e + p.av * r }
}

#[inline(always)]
fn dvg(p: V3, q: f32) -> V3 {
    let r = rk(q);
    let e = lz(q);
    V3 { b: p.b, c: p.c * r - p.av * e, av: p.c * e + p.av * r }
}

#[inline(always)]
fn pwa(p: V3, pt: f32) -> V3 {
    V3 { b: p.b, c: p.c, av: p.av + pt }
}


#[inline(always)]
fn nv(p: V3) -> Anz {
    if p.av.gp() < 0.001 {
        Anz { b: 0.0, c: 0.0 }
    } else {
        Anz { b: p.b / p.av, c: p.c / p.av }
    }
}


#[inline(always)]
fn mlk(ai: Anz, d: usize, i: usize) -> (i32, i32) {
    let bv = d.v(i) as f32 * 0.45;
    let cr = (ai.b * bv) + (d as f32 * 0.5);
    let cq = (-ai.c * bv) + (i as f32 * 0.5);
    (cr as i32, cq as i32)
}


#[inline(always)]
fn cni(p: V3, aev: f32, ajt: f32, pt: f32, d: usize, i: usize) -> (i32, i32, f32) {
    let cmk = dvg(cmj(p, aev), ajt);
    let fag = pwa(cmk, pt);
    let pgo = mlk(nv(fag), d, i);
    (pgo.0, pgo.1, fag.av)
}



pub struct Bh {
    pub lm: alloc::vec::Vec<V3>,
    pub bu: alloc::vec::Vec<(usize, usize)>,
    
    pub cqd: Option<alloc::vec::Vec<u32>>,
    
    pub ks: Option<alloc::vec::Vec<(usize, usize, usize)>>,
    
    pub cxq: Option<alloc::vec::Vec<u32>>,
}

pub fn czt() -> Bh {
    let p = alloc::vec![
        V3 { b: -0.5, c: -0.5, av: -0.5 }, V3 { b:  0.5, c: -0.5, av: -0.5 },
        V3 { b:  0.5, c:  0.5, av: -0.5 }, V3 { b: -0.5, c:  0.5, av: -0.5 },
        V3 { b: -0.5, c: -0.5, av:  0.5 }, V3 { b:  0.5, c: -0.5, av:  0.5 },
        V3 { b:  0.5, c:  0.5, av:  0.5 }, V3 { b: -0.5, c:  0.5, av:  0.5 },
    ];
    let aa = alloc::vec![
        (0,1),(1,2),(2,3),(3,0), 
        (4,5),(5,6),(6,7),(7,4), 
        (0,4),(1,5),(2,6),(3,7), 
    ];
    let bb = alloc::vec![
        
        (0, 1, 2), (0, 2, 3),
        
        (5, 4, 7), (5, 7, 6),
        
        (3, 2, 6), (3, 6, 7),
        
        (4, 5, 1), (4, 1, 0),
        
        (1, 5, 6), (1, 6, 2),
        
        (4, 0, 3), (4, 3, 7),
    ];
    Bh { lm: p, bu: aa, cqd: None, ks: Some(bb), cxq: None }
}

pub fn czv() -> Bh {
    let p = alloc::vec![
        V3 { b: -0.5, c: -0.5, av: -0.5 }, V3 { b:  0.5, c: -0.5, av: -0.5 },
        V3 { b:  0.5, c: -0.5, av:  0.5 }, V3 { b: -0.5, c: -0.5, av:  0.5 },
        V3 { b:  0.0, c:  0.7, av:  0.0 },
    ];
    let aa = alloc::vec![
        (0,1),(1,2),(2,3),(3,0), 
        (0,4),(1,4),(2,4),(3,4), 
    ];
    let bb = alloc::vec![
        
        (0, 1, 2), (0, 2, 3),
        
        (0, 1, 4), (1, 2, 4), (2, 3, 4), (3, 0, 4),
    ];
    Bh { lm: p, bu: aa, cqd: None, ks: Some(bb), cxq: None }
}

pub fn czu() -> Bh {
    let p = alloc::vec![
        V3 { b:  0.0, c:  0.7, av:  0.0 }, 
        V3 { b: -0.5, c:  0.0, av: -0.5 }, V3 { b:  0.5, c:  0.0, av: -0.5 },
        V3 { b:  0.5, c:  0.0, av:  0.5 }, V3 { b: -0.5, c:  0.0, av:  0.5 },
        V3 { b:  0.0, c: -0.7, av:  0.0 }, 
    ];
    let aa = alloc::vec![
        (1,2),(2,3),(3,4),(4,1), 
        (0,1),(0,2),(0,3),(0,4), 
        (5,1),(5,2),(5,3),(5,4), 
    ];
    let bb = alloc::vec![
        
        (0, 1, 2), (0, 2, 3), (0, 3, 4), (0, 4, 1),
        
        (5, 2, 1), (5, 3, 2), (5, 4, 3), (5, 1, 4),
    ];
    Bh { lm: p, bu: aa, cqd: None, ks: Some(bb), cxq: None }
}

pub fn czw(czl: f32, cge: f32, jeq: usize, fok: usize) -> Bh {
    let mut by = alloc::vec::Vec::fc(jeq * fok);
    let mut bu = alloc::vec::Vec::new();
    
    for a in 0..jeq {
        let bdb = (a as f32 / jeq as f32) * 6.2831853;
        let aqx = rk(bdb);
        let apc = lz(bdb);
        for fb in 0..fok {
            let bnv = (fb as f32 / fok as f32) * 6.2831853;
            let bza = rk(bnv);
            let sp = lz(bnv);
            let b = (czl + cge * bza) * aqx;
            let av = (czl + cge * bza) * apc;
            let c = cge * sp;
            by.push(V3 { b, c, av });
            
            let w = a * fok + fb;
            let uuj = a * fok + (fb + 1) % fok;
            bu.push((w, uuj));
            let uug = ((a + 1) % jeq) * fok + fb;
            bu.push((w, uug));
        }
    }
    Bh { lm: by, bu, cqd: None, ks: None, cxq: None }
}

pub fn fod(dy: f32) -> Bh {
    let ab = (1.0 + ahn(5.0)) / 2.0;
    let e = dy / ahn(1.0 + ab * ab);
    let q = e;
    let o = e * ab;
    
    let p = alloc::vec![
        V3 { b: -q, c:  o, av: 0.0 }, V3 { b:  q, c:  o, av: 0.0 },
        V3 { b: -q, c: -o, av: 0.0 }, V3 { b:  q, c: -o, av: 0.0 },
        V3 { b: 0.0, c: -q, av:  o }, V3 { b: 0.0, c:  q, av:  o },
        V3 { b: 0.0, c: -q, av: -o }, V3 { b: 0.0, c:  q, av: -o },
        V3 { b:  o, c: 0.0, av: -q }, V3 { b:  o, c: 0.0, av:  q },
        V3 { b: -o, c: 0.0, av: -q }, V3 { b: -o, c: 0.0, av:  q },
    ];
    let aa = alloc::vec![
        (0,1),(0,5),(0,7),(0,10),(0,11),
        (1,5),(1,7),(1,8),(1,9),
        (2,3),(2,4),(2,6),(2,10),(2,11),
        (3,4),(3,6),(3,8),(3,9),
        (4,5),(4,9),(4,11),
        (5,9),(5,11),
        (6,7),(6,8),(6,10),
        (7,8),(7,10),
        (8,9),
        (10,11),
    ];
    Bh { lm: p, bu: aa, cqd: None, ks: None, cxq: None }
}

pub fn foc(iv: f32, nmb: usize) -> Bh {
    let mut by = alloc::vec::Vec::new();
    let mut bu = alloc::vec::Vec::new();
    let gu = (iv * 2.0) / nmb as f32;
    let bo = nmb + 1;
    
    for a in 0..bo {
        for fb in 0..bo {
            let b = -iv + a as f32 * gu;
            let av = -iv + fb as f32 * gu;
            by.push(V3 { b, c: -0.3, av });
            let w = a * bo + fb;
            if fb + 1 < bo { bu.push((w, w + 1)); }
            if a + 1 < bo { bu.push((w, w + bo)); }
        }
    }
    Bh { lm: by, bu, cqd: None, ks: None, cxq: None }
}



pub fn foe() -> Bh {
    let mut by = alloc::vec::Vec::fc(80);
    let mut bu = alloc::vec::Vec::fc(120);

    
    
    let dyz = by.len(); 
    let nm = 0.28; let dys = 0.22; let je = -0.40;
    by.push(V3 { b: -nm, c: je, av: -dys }); 
    by.push(V3 { b:  nm, c: je, av: -dys }); 
    by.push(V3 { b:  nm, c: je, av:  dys }); 
    by.push(V3 { b: -nm, c: je, av:  dys }); 
    
    let dol = by.len(); 
    by.push(V3 { b: -nm, c: 0.0, av: -dys }); 
    by.push(V3 { b:  nm, c: 0.0, av: -dys }); 
    by.push(V3 { b:  nm, c: 0.0, av:  dys }); 
    by.push(V3 { b: -nm, c: 0.0, av:  dys }); 
    
    let dzb = by.len(); 
    let qd = 0.24; let jss = 0.20;
    by.push(V3 { b: -qd, c: 0.30, av: -jss }); 
    by.push(V3 { b:  qd, c: 0.30, av: -jss }); 
    by.push(V3 { b:  qd, c: 0.30, av:  jss }); 
    by.push(V3 { b: -qd, c: 0.30, av:  jss }); 

    
    bu.push((dyz, dyz+1)); bu.push((dyz+1, dyz+2));
    bu.push((dyz+2, dyz+3)); bu.push((dyz+3, dyz));
    
    bu.push((dol, dol+1)); bu.push((dol+1, dol+2));
    bu.push((dol+2, dol+3)); bu.push((dol+3, dol));
    
    bu.push((dzb, dzb+1)); bu.push((dzb+1, dzb+2));
    bu.push((dzb+2, dzb+3)); bu.push((dzb+3, dzb));
    
    for a in 0..4 { bu.push((dyz + a, dol + a)); }
    for a in 0..4 { bu.push((dol + a, dzb + a)); }

    
    let ecs = by.len(); 
    let avz = 0.26; let cfh = 0.22; let iyc = 0.32;
    by.push(V3 { b: -avz, c: iyc, av: -cfh }); 
    by.push(V3 { b:  avz, c: iyc, av: -cfh }); 
    by.push(V3 { b:  avz, c: iyc, av:  cfh }); 
    by.push(V3 { b: -avz, c: iyc, av:  cfh }); 
    
    let cfi = by.len(); 
    let iyy = 0.24; let iyw = 0.20;
    by.push(V3 { b: -iyy, c: 0.65, av: -iyw }); 
    by.push(V3 { b:  iyy, c: 0.65, av: -iyw }); 
    by.push(V3 { b:  iyy, c: 0.65, av:  iyw }); 
    by.push(V3 { b: -iyy, c: 0.65, av:  iyw }); 

    
    bu.push((ecs, ecs+1)); bu.push((ecs+1, ecs+2));
    bu.push((ecs+2, ecs+3)); bu.push((ecs+3, ecs));
    bu.push((cfi, cfi+1)); bu.push((cfi+1, cfi+2));
    bu.push((cfi+2, cfi+3)); bu.push((cfi+3, cfi));
    for a in 0..4 { bu.push((ecs + a, cfi + a)); }

    
    
    let myp = by.len(); 
    let qop = 0.0; let qoq = -0.05; let qot = -dys - 0.01;
    let qor = 0.16; let qos = 0.28;
    for a in 0..8u32 {
        let hg = a as f32 * 0.7853982; 
        let b = qop + qor * rk(hg);
        let c = qoq + qos * lz(hg);
        by.push(V3 { b, c, av: qot });
    }
    for a in 0..8 { bu.push((myp + a, myp + (a + 1) % 8)); }

    
    let ckh = 0.50; let dqr = -cfh - 0.01; let bzp = 0.04;
    
    let ij = by.len(); 
    by.push(V3 { b: -0.10 - bzp, c: ckh - bzp, av: dqr });
    by.push(V3 { b: -0.10 + bzp, c: ckh - bzp, av: dqr });
    by.push(V3 { b: -0.10 + bzp, c: ckh + bzp, av: dqr });
    by.push(V3 { b: -0.10 - bzp, c: ckh + bzp, av: dqr });
    bu.push((ij, ij+1)); bu.push((ij+1, ij+2)); bu.push((ij+2, ij+3)); bu.push((ij+3, ij));
    
    let bqm = by.len(); 
    by.push(V3 { b: 0.10 - bzp, c: ckh - bzp, av: dqr });
    by.push(V3 { b: 0.10 + bzp, c: ckh - bzp, av: dqr });
    by.push(V3 { b: 0.10 + bzp, c: ckh + bzp, av: dqr });
    by.push(V3 { b: 0.10 - bzp, c: ckh + bzp, av: dqr });
    bu.push((bqm, bqm+1)); bu.push((bqm+1, bqm+2)); bu.push((bqm+2, bqm+3)); bu.push((bqm+3, bqm));
    
    bu.push((ij, ij+2)); bu.push((ij+1, ij+3));
    bu.push((bqm, bqm+2)); bu.push((bqm+1, bqm+3));

    
    let ilf = 0.42; let ilg = -cfh - 0.03;
    let cvw = by.len(); 
    by.push(V3 { b:  0.00, c: ilf + 0.03, av: ilg }); 
    by.push(V3 { b: -0.04, c: ilf,        av: ilg }); 
    by.push(V3 { b:  0.04, c: ilf,        av: ilg }); 
    by.push(V3 { b:  0.00, c: ilf - 0.02, av: ilg - 0.02 }); 
    bu.push((cvw, cvw+1)); bu.push((cvw, cvw+2)); bu.push((cvw+1, cvw+2));
    bu.push((cvw+1, cvw+3)); bu.push((cvw+2, cvw+3)); bu.push((cvw, cvw+3));

    
    let bzs = -0.42; let eqr = 0.04; let bzr = 0.10; let bzq = 0.14;
    
    let bee = by.len(); 
    by.push(V3 { b: -0.16 - bzr, c: bzs,          av: -bzq });
    by.push(V3 { b: -0.16 + bzr, c: bzs,          av: -bzq });
    by.push(V3 { b: -0.16 + bzr, c: bzs,          av:  bzq });
    by.push(V3 { b: -0.16 - bzr, c: bzs,          av:  bzq });
    by.push(V3 { b: -0.16 - bzr, c: bzs - eqr, av: -bzq });
    by.push(V3 { b: -0.16 + bzr, c: bzs - eqr, av: -bzq });
    by.push(V3 { b: -0.16 + bzr, c: bzs - eqr, av:  bzq });
    by.push(V3 { b: -0.16 - bzr, c: bzs - eqr, av:  bzq });
    bu.push((bee, bee+1)); bu.push((bee+1, bee+2)); bu.push((bee+2, bee+3)); bu.push((bee+3, bee));
    bu.push((bee+4, bee+5)); bu.push((bee+5, bee+6)); bu.push((bee+6, bee+7)); bu.push((bee+7, bee+4));
    for a in 0..4 { bu.push((bee + a, bee + 4 + a)); }
    
    let xb = by.len(); 
    by.push(V3 { b: 0.16 - bzr, c: bzs,          av: -bzq });
    by.push(V3 { b: 0.16 + bzr, c: bzs,          av: -bzq });
    by.push(V3 { b: 0.16 + bzr, c: bzs,          av:  bzq });
    by.push(V3 { b: 0.16 - bzr, c: bzs,          av:  bzq });
    by.push(V3 { b: 0.16 - bzr, c: bzs - eqr, av: -bzq });
    by.push(V3 { b: 0.16 + bzr, c: bzs - eqr, av: -bzq });
    by.push(V3 { b: 0.16 + bzr, c: bzs - eqr, av:  bzq });
    by.push(V3 { b: 0.16 - bzr, c: bzs - eqr, av:  bzq });
    bu.push((xb, xb+1)); bu.push((xb+1, xb+2)); bu.push((xb+2, xb+3)); bu.push((xb+3, xb));
    bu.push((xb+4, xb+5)); bu.push((xb+5, xb+6)); bu.push((xb+6, xb+7)); bu.push((xb+7, xb+4));
    for a in 0..4 { bu.push((xb + a, xb + 4 + a)); }

    
    
    let fbr = by.len(); 
    by.push(V3 { b: -nm - 0.01, c:  0.20, av:  0.00 }); 
    by.push(V3 { b: -nm - 0.14, c: -0.05, av: -0.04 }); 
    by.push(V3 { b: -nm - 0.10, c: -0.25, av: -0.02 }); 
    by.push(V3 { b: -nm - 0.01, c: -0.15, av:  0.00 }); 
    bu.push((fbr, fbr+1)); bu.push((fbr+1, fbr+2)); bu.push((fbr+2, fbr+3)); bu.push((fbr+3, fbr));
    
    let bfu = by.len(); 
    by.push(V3 { b: nm + 0.01, c:  0.20, av:  0.00 });
    by.push(V3 { b: nm + 0.14, c: -0.05, av: -0.04 });
    by.push(V3 { b: nm + 0.10, c: -0.25, av: -0.02 });
    by.push(V3 { b: nm + 0.01, c: -0.15, av:  0.00 });
    bu.push((bfu, bfu+1)); bu.push((bfu+1, bfu+2)); bu.push((bfu+2, bfu+3)); bu.push((bfu+3, bfu));

    
    let gds = by.len(); 
    let gdt = 0.65; let nho = 0.22;
    for a in 0..8u32 {
        let hg = a as f32 * 0.7853982; 
        let b = nho * rk(hg);
        let av = nho * 0.9 * lz(hg);
        let xwy = 0.08 * rk(hg * 2.0); 
        by.push(V3 { b, c: gdt + xwy, av });
    }
    for a in 0..8 { bu.push((gds + a, gds + (a + 1) % 8)); }
    
    bu.push((gds + 0, cfi + 1)); 
    bu.push((gds + 2, cfi + 2)); 
    bu.push((gds + 4, cfi + 3)); 
    bu.push((gds + 6, cfi));     

    Bh { lm: by, bu, cqd: None, ks: None, cxq: None }
}



pub fn onb() -> Bh {
    let mut by = alloc::vec::Vec::fc(220);
    let mut bu = alloc::vec::Vec::fc(350);

    let zv: f32 = 0.18;     
    let eo: f32 = 0.06;  
    let jb: f32 = 0.25;  
    let ql: f32 = -jb * 3.0; 
    let qc: f32 = 0.20;    
    let bjj: f32 = -0.20;   
    let vs: f32 = 0.0;     

    
    fn dny(by: &mut alloc::vec::Vec<V3>, bu: &mut alloc::vec::Vec<(usize, usize)>,
                    egw: &[(f32, f32)], mp: f32, eo: f32) {
        let ar = by.len();
        for &(y, x) in egw {
            by.push(V3 { b: mp + y, c: x, av: -eo }); 
            by.push(V3 { b: mp + y, c: x, av: eo });  
        }
        for a in 0..egw.len() {
            
            bu.push((ar + a * 2, ar + a * 2 + 1));
            
            if a + 1 < egw.len() {
                bu.push((ar + a * 2, ar + (a + 1) * 2));
            }
            
            if a + 1 < egw.len() {
                bu.push((ar + a * 2 + 1, ar + (a + 1) * 2 + 1));
            }
        }
    }

    
    let mp = ql;
    dny(&mut by, &mut bu, &[(0.0, qc), (zv, qc)], mp, eo);
    dny(&mut by, &mut bu, &[(zv * 0.5, qc), (zv * 0.5, bjj)], mp, eo);

    
    let mp = ql + jb;
    dny(&mut by, &mut bu,
        &[(0.0, bjj), (0.0, qc), (zv, qc), (zv, vs), (0.0, vs)], mp, eo);
    dny(&mut by, &mut bu,
        &[(0.04, vs), (zv, bjj)], mp, eo); 

    
    let mp = ql + jb * 2.0;
    dny(&mut by, &mut bu,
        &[(0.0, qc), (0.0, bjj), (zv, bjj), (zv, qc)], mp, eo);

    
    let mp = ql + jb * 3.0;
    dny(&mut by, &mut bu,
        &[(zv, qc), (0.0, qc), (0.0, vs), (zv, vs), (zv, bjj), (0.0, bjj)], mp, eo);

    
    let mp = ql + jb * 4.0;
    dny(&mut by, &mut bu, &[(0.0, qc), (zv, qc)], mp, eo);
    dny(&mut by, &mut bu, &[(zv * 0.5, qc), (zv * 0.5, bjj)], mp, eo);

    
    let mp = ql + jb * 5.0;
    dny(&mut by, &mut bu,
        &[(0.0, qc), (zv, qc), (zv, bjj), (0.0, bjj), (0.0, qc)], mp, eo);

    
    let mp = ql + jb * 6.0;
    dny(&mut by, &mut bu,
        &[(zv, qc), (0.0, qc), (0.0, vs), (zv, vs), (zv, bjj), (0.0, bjj)], mp, eo);

    Bh { lm: by, bu, cqd: None, ks: None, cxq: None }
}




pub fn gmq() -> Bh {
    let mut by = alloc::vec::Vec::fc(120);
    let mut bu = alloc::vec::Vec::fc(200);
    let mut colors = alloc::vec::Vec::fc(200);

    
    let ens   = 0xFF88BBDD; 
    let ios   = 0xFFAADDFF; 
    let bys   = 0xFF7799BB; 
    let azh = 0xFFFFCC44; 
    let neq   = 0xFF6688AA; 

    
    let mut coi = |by: &mut alloc::vec::Vec<V3>, bu: &mut alloc::vec::Vec<(usize, usize)>,
                       colors: &mut alloc::vec::Vec<u32>,
                       cx: f32, ae: f32, zr: f32, avz: f32, bka: f32, cfh: f32, s: u32| -> usize {
        let o = by.len();
        
        by.push(V3 { b: cx - avz, c: ae - bka, av: zr - cfh }); 
        by.push(V3 { b: cx + avz, c: ae - bka, av: zr - cfh }); 
        by.push(V3 { b: cx + avz, c: ae - bka, av: zr + cfh }); 
        by.push(V3 { b: cx - avz, c: ae - bka, av: zr + cfh }); 
        by.push(V3 { b: cx - avz, c: ae + bka, av: zr - cfh }); 
        by.push(V3 { b: cx + avz, c: ae + bka, av: zr - cfh }); 
        by.push(V3 { b: cx + avz, c: ae + bka, av: zr + cfh }); 
        by.push(V3 { b: cx - avz, c: ae + bka, av: zr + cfh }); 
        
        bu.push((o, o+1)); bu.push((o+1, o+2)); bu.push((o+2, o+3)); bu.push((o+3, o));
        
        bu.push((o+4, o+5)); bu.push((o+5, o+6)); bu.push((o+6, o+7)); bu.push((o+7, o+4));
        
        bu.push((o, o+4)); bu.push((o+1, o+5)); bu.push((o+2, o+6)); bu.push((o+3, o+7));
        
        for _ in 0..12 { colors.push(s); }
        o
    };

    
    
    let dhy = by.len(); 
    let jbz = 0.10; let jby = 0.09; let jca = 0.58;
    by.push(V3 { b: -jbz, c: jca, av: -jby });
    by.push(V3 { b:  jbz, c: jca, av: -jby });
    by.push(V3 { b:  jbz, c: jca, av:  jby });
    by.push(V3 { b: -jbz, c: jca, av:  jby });
    
    let dqs = by.len(); 
    let itr = 0.14; let itq = 0.12; let its = 0.68;
    by.push(V3 { b: -itr, c: its, av: -itq });
    by.push(V3 { b:  itr, c: its, av: -itq });
    by.push(V3 { b:  itr, c: its, av:  itq });
    by.push(V3 { b: -itr, c: its, av:  itq });
    
    let dph = by.len(); 
    let ips = 0.12; let ipr = 0.10; let gdt = 0.78;
    by.push(V3 { b: -ips, c: gdt, av: -ipr });
    by.push(V3 { b:  ips, c: gdt, av: -ipr });
    by.push(V3 { b:  ips, c: gdt, av:  ipr });
    by.push(V3 { b: -ips, c: gdt, av:  ipr });
    
    let cfi = by.len(); 
    by.push(V3 { b: 0.0, c: 0.84, av: 0.0 });
    
    bu.push((dhy, dhy+1)); bu.push((dhy+1, dhy+2));
    bu.push((dhy+2, dhy+3)); bu.push((dhy+3, dhy));
    for _ in 0..4 { colors.push(ios); }
    
    bu.push((dqs, dqs+1)); bu.push((dqs+1, dqs+2));
    bu.push((dqs+2, dqs+3)); bu.push((dqs+3, dqs));
    for _ in 0..4 { colors.push(ios); }
    
    bu.push((dph, dph+1)); bu.push((dph+1, dph+2));
    bu.push((dph+2, dph+3)); bu.push((dph+3, dph));
    for _ in 0..4 { colors.push(azh); } 
    
    for a in 0..4 { bu.push((dhy + a, dqs + a)); colors.push(ios); }
    
    for a in 0..4 { bu.push((dqs + a, dph + a)); colors.push(ios); }
    
    for a in 0..4 { bu.push((dph + a, cfi)); colors.push(azh); }

    
    let gnl = by.len(); 
    by.push(V3 { b: -0.06, c: 0.52, av: 0.0 }); 
    by.push(V3 { b:  0.06, c: 0.52, av: 0.0 }); 
    bu.push((dhy, gnl)); colors.push(azh);
    bu.push((dhy+1, gnl+1)); colors.push(azh);
    bu.push((gnl, gnl+1)); colors.push(azh);

    
    
    let kl = by.len(); 
    let jqf = 0.22; let jpu = 0.11; let jqg = 0.48;
    by.push(V3 { b: -jqf, c: jqg, av: -jpu });
    by.push(V3 { b:  jqf, c: jqg, av: -jpu });
    by.push(V3 { b:  jqf, c: jqg, av:  jpu });
    by.push(V3 { b: -jqf, c: jqg, av:  jpu });
    
    let bm = by.len(); 
    let inq = 0.20; let ini = 0.10; let inr = 0.30;
    by.push(V3 { b: -inq, c: inr, av: -ini });
    by.push(V3 { b:  inq, c: inr, av: -ini });
    by.push(V3 { b:  inq, c: inr, av:  ini });
    by.push(V3 { b: -inq, c: inr, av:  ini });
    
    let dxq = by.len(); 
    let jwn = 0.16; let jwf = 0.09; let jwo = 0.08;
    by.push(V3 { b: -jwn, c: jwo, av: -jwf });
    by.push(V3 { b:  jwn, c: jwo, av: -jwf });
    by.push(V3 { b:  jwn, c: jwo, av:  jwf });
    by.push(V3 { b: -jwn, c: jwo, av:  jwf });
    
    let crh = by.len(); 
    let iyu = 0.17; let gje = 0.09; let iyv = -0.04;
    by.push(V3 { b: -iyu, c: iyv, av: -gje });
    by.push(V3 { b:  iyu, c: iyv, av: -gje });
    by.push(V3 { b:  iyu, c: iyv, av:  gje });
    by.push(V3 { b: -iyu, c: iyv, av:  gje });
    
    bu.push((gnl, kl)); colors.push(ens);
    bu.push((gnl+1, kl+1)); colors.push(ens);
    
    bu.push((kl, kl+1)); bu.push((kl+1, kl+2)); bu.push((kl+2, kl+3)); bu.push((kl+3, kl));
    for _ in 0..4 { colors.push(ens); }
    
    bu.push((bm, bm+1)); bu.push((bm+1, bm+2)); bu.push((bm+2, bm+3)); bu.push((bm+3, bm));
    for _ in 0..4 { colors.push(ens); }
    
    bu.push((dxq, dxq+1)); bu.push((dxq+1, dxq+2)); bu.push((dxq+2, dxq+3)); bu.push((dxq+3, dxq));
    for _ in 0..4 { colors.push(azh); } 
    
    bu.push((crh, crh+1)); bu.push((crh+1, crh+2)); bu.push((crh+2, crh+3)); bu.push((crh+3, crh));
    for _ in 0..4 { colors.push(ens); }
    
    for a in 0..4 { bu.push((kl + a, bm + a)); colors.push(ens); }
    for a in 0..4 { bu.push((bm + a, dxq + a)); colors.push(ens); }
    for a in 0..4 { bu.push((dxq + a, crh + a)); colors.push(ens); }

    
    let jed = coi(&mut by, &mut bu, &mut colors,
        -0.30, 0.38, 0.0,   0.06, 0.10, 0.06,  bys); 
    let jdk = coi(&mut by, &mut bu, &mut colors,
        -0.32, 0.16, 0.0,   0.05, 0.10, 0.05,  bys); 
    
    bu.push((kl, jed + 4)); colors.push(azh); 
    bu.push((kl, jed + 7)); colors.push(azh);
    
    bu.push((jed, jdk + 4)); colors.push(azh);
    bu.push((jed + 3, jdk + 7)); colors.push(azh);
    
    let ojb = coi(&mut by, &mut bu, &mut colors,
        -0.33, 0.02, 0.0,   0.04, 0.04, 0.04,  azh);
    bu.push((jdk, ojb + 4)); colors.push(bys);
    bu.push((jdk + 1, ojb + 5)); colors.push(bys);

    
    let jnd = coi(&mut by, &mut bu, &mut colors,
        0.30, 0.38, 0.0,   0.06, 0.10, 0.06,  bys);
    let jmk = coi(&mut by, &mut bu, &mut colors,
        0.32, 0.16, 0.0,   0.05, 0.10, 0.05,  bys);
    bu.push((kl + 1, jnd + 5)); colors.push(azh);
    bu.push((kl + 1, jnd + 6)); colors.push(azh);
    bu.push((jnd + 1, jmk + 5)); colors.push(azh);
    bu.push((jnd + 2, jmk + 6)); colors.push(azh);
    let pdc = coi(&mut by, &mut bu, &mut colors,
        0.33, 0.02, 0.0,   0.04, 0.04, 0.04,  azh);
    bu.push((jmk, pdc + 4)); colors.push(bys);
    bu.push((jmk + 1, pdc + 5)); colors.push(bys);

    
    let jec = coi(&mut by, &mut bu, &mut colors,
        -0.09, -0.18, 0.0,   0.06, 0.12, 0.06,  bys);
    let jeb = coi(&mut by, &mut bu, &mut colors,
        -0.09, -0.40, 0.0,   0.05, 0.10, 0.05,  bys);
    
    bu.push((crh, jec + 4)); colors.push(azh);
    bu.push((crh + 3, jec + 7)); colors.push(azh);
    
    bu.push((jec, jeb + 4)); colors.push(azh);
    bu.push((jec + 3, jeb + 7)); colors.push(azh);
    
    let oja = coi(&mut by, &mut bu, &mut colors,
        -0.09, -0.53, -0.02,   0.06, 0.03, 0.08,  neq);
    bu.push((jeb, oja + 4)); colors.push(bys);
    bu.push((jeb + 1, oja + 5)); colors.push(bys);

    
    let jnc = coi(&mut by, &mut bu, &mut colors,
        0.09, -0.18, 0.0,   0.06, 0.12, 0.06,  bys);
    let jmz = coi(&mut by, &mut bu, &mut colors,
        0.09, -0.40, 0.0,   0.05, 0.10, 0.05,  bys);
    bu.push((crh + 1, jnc + 5)); colors.push(azh);
    bu.push((crh + 2, jnc + 6)); colors.push(azh);
    bu.push((jnc + 1, jmz + 5)); colors.push(azh);
    bu.push((jnc + 2, jmz + 6)); colors.push(azh);
    let pda = coi(&mut by, &mut bu, &mut colors,
        0.09, -0.53, -0.02,   0.06, 0.03, 0.08,  neq);
    bu.push((jmz, pda + 4)); colors.push(bys);
    bu.push((jmz + 1, pda + 5)); colors.push(bys);

    Bh { lm: by, bu, cqd: Some(colors), ks: None, cxq: None }
}

pub fn gmr(dy: f32, ac: f32, cuy: f32, pk: usize) -> Bh {
    let mut by = alloc::vec::Vec::new();
    let mut bu = alloc::vec::Vec::new();
    let xjs = cuy * 6.2831853;
    
    for wuy in 0..2u32 {
        let l = wuy as f32 * 3.14159265;
        let ar = by.len();
        for a in 0..pk {
            let ab = a as f32 / (pk - 1) as f32;
            let hg = ab * xjs + l;
            let b = dy * rk(hg);
            let av = dy * lz(hg);
            let c = -ac * 0.5 + ab * ac;
            by.push(V3 { b, c, av });
            if a > 0 { bu.push((ar + a - 1, ar + a)); }
        }
    }
    
    let iv = pk;
    for a in (0..pk).akt(pk / 8) {
        bu.push((a, a + iv));
    }
    Bh { lm: by, bu, cqd: None, ks: None, cxq: None }
}




#[inline(always)]
fn bjr(q: V3, o: V3) -> V3 {
    V3 {
        b: q.c * o.av - q.av * o.c,
        c: q.av * o.b - q.b * o.av,
        av: q.b * o.c - q.c * o.b,
    }
}


#[inline(always)]
fn amb(q: V3, o: V3) -> f32 {
    q.b * o.b + q.c * o.c + q.av * o.av
}


#[inline(always)]
fn sub(q: V3, o: V3) -> V3 {
    V3 { b: q.b - o.b, c: q.c - o.c, av: q.av - o.av }
}


#[inline(always)]
fn all(p: V3) -> V3 {
    let len = ahn(p.b * p.b + p.c * p.c + p.av * p.av);
    if len < 0.0001 { return V3 { b: 0.0, c: 0.0, av: 1.0 }; }
    let wq = 1.0 / len;
    V3 { b: p.b * wq, c: p.c * wq, av: p.av * wq }
}




#[inline(never)]
fn kvx(k: &mut [u32], d: usize, i: usize,
                 mut fy: i32, mut fo: i32,
                 mut dn: i32, mut dp: i32,
                 mut hy: i32, mut jz: i32,
                 s: u32) {
    
    if fo > dp { core::mem::swap(&mut fy, &mut dn); core::mem::swap(&mut fo, &mut dp); }
    if dp > jz { core::mem::swap(&mut dn, &mut hy); core::mem::swap(&mut dp, &mut jz); }
    if fo > dp { core::mem::swap(&mut fy, &mut dn); core::mem::swap(&mut fo, &mut dp); }

    let aku = jz - fo;
    if aku == 0 { return; }

    
    let bpl = fo.am(0);
    let dno = jz.v(i as i32 - 1);

    for c in bpl..=dno {
        let hzh = c >= dp;
        let ftz = if hzh { jz - dp } else { dp - fo };

        
        let mjn = (c - fo) as f32 / aku as f32;
        let bpj = fy as f32 + (hy - fy) as f32 * mjn; 

        let dnk = if ftz == 0 {
            bpj 
        } else if hzh {
            let fwa = (c - dp) as f32 / ftz as f32;
            dn as f32 + (hy - dn) as f32 * fwa
        } else {
            let fwa = (c - fo) as f32 / ftz as f32;
            fy as f32 + (dn - fy) as f32 * fwa
        };

        let mut fd = bpj as i32;
        let mut hw = dnk as i32;
        if fd > hw { core::mem::swap(&mut fd, &mut hw); }

        
        fd = fd.am(0);
        hw = hw.v(d as i32 - 1);

        let br = c as usize * d;
        for b in fd..=hw {
            let w = br + b as usize;
            if w < k.len() {
                k[w] = s;
            }
        }
    }
}



pub fn vxb(k: &mut [u32], d: usize, i: usize,
                             mesh: &Bh, aev: f32, ajt: f32, pt: f32,
                             s: u32) {
    for (w, &(q, o)) in mesh.bu.iter().cf() {
        if q >= mesh.lm.len() || o >= mesh.lm.len() { continue; }
        let (fy, fo, alw) = cni(mesh.lm[q], aev, ajt, pt, d, i);
        let (dn, dp, aeu) = cni(mesh.lm[o], aev, ajt, pt, d, i);
        let bth = (alw + aeu) * 0.5;
        let ar = match &mesh.cqd {
            Some(ec) if w < ec.len() => ec[w],
            _ => s,
        };
        let r = kpg(bth, ar);
        dqg(k, d, i, fy, fo, dn, dp, r);
    }
    
    for p in &mesh.lm {
        let (cr, cq, qed) = cni(*p, aev, ajt, pt, d, i);
        for bg in -1..=1i32 {
            for dx in -1..=1i32 {
                ijb(k, d, i, cr + dx, cq + bg, 0x00FFFFFF);
            }
        }
    }
}




pub fn lzc(k: &mut [u32], d: usize, i: usize,
                          mesh: &Bh, aev: f32, ajt: f32, pt: f32,
                          agg: u32, csc: V3, cvo: f32) {
    let ks = match &mesh.ks {
        Some(bb) => bb,
        None => return,
    };

    
    let mut dxc = alloc::vec::Vec::fc(mesh.lm.len());
    let mut bkz = alloc::vec::Vec::fc(mesh.lm.len());
    for p in &mesh.lm {
        let cmk = dvg(cmj(*p, aev), ajt);
        let fag = pwa(cmk, pt);
        let wem = mlk(nv(fag), d, i);
        dxc.push(fag);
        bkz.push(wem);
    }

    
    
    struct Cdh {
        w: usize,
        bth: f32,
        kt: f32,
    }
    let mut gwd = alloc::vec::Vec::fc(ks.len());

    for (a, &(q, o, r)) in ks.iter().cf() {
        if q >= dxc.len() || o >= dxc.len() || r >= dxc.len() { continue; }

        let asf = dxc[q];
        let cci = dxc[o];
        let cvd = dxc[r];

        
        let siq = sub(cci, asf);
        let sir = sub(cvd, asf);
        let adg = all(bjr(siq, sir));

        
        
        
        if adg.av > 0.0 { continue; }

        
        let hsl = amb(adg, csc);
        let kt = cvo + (1.0 - cvo) * hsl.am(0.0);

        let bth = (asf.av + cci.av + cvd.av) / 3.0;
        gwd.push(Cdh { w: a, bth, kt });
    }

    
    
    for a in 1..gwd.len() {
        let mut fb = a;
        while fb > 0 && gwd[fb].bth > gwd[fb - 1].bth {
            gwd.swap(fb, fb - 1);
            fb -= 1;
        }
    }

    
    let bdm = (agg >> 16) & 0xFF;
    let bji = (agg >> 8) & 0xFF;
    let cdd = agg & 0xFF;

    for dhc in &gwd {
        let (q, o, r) = ks[dhc.w];
        let (dmf, dmg) = bkz[q];
        let (asa, bos) = bkz[o];
        let (amy, bcw) = bkz[r];

        
        let (xb, lp, pq) = match &mesh.cxq {
            Some(gc) if dhc.w < gc.len() => {
                ((gc[dhc.w] >> 16) & 0xFF,
                 (gc[dhc.w] >> 8) & 0xFF,
                 gc[dhc.w] & 0xFF)
            }
            _ => (bdm, bji, cdd),
        };

        let m = ((xb as f32 * dhc.kt) as u32).v(255);
        let at = ((lp as f32 * dhc.kt) as u32).v(255);
        let o = ((pq as f32 * dhc.kt) as u32).v(255);
        let mfi = 0xFF000000 | (m << 16) | (at << 8) | o;

        kvx(k, d, i, dmf, dmg, asa, bos, amy, bcw, mfi);
    }

    
    for &(q, o) in &mesh.bu {
        if q >= bkz.len() || o >= bkz.len() { continue; }
        let (fy, fo) = bkz[q];
        let (dn, dp) = bkz[o];
        
        let cqc = 0xFF000000 | ((bdm / 3) << 16) | ((bji / 3) << 8) | (cdd / 3);
        ahj(k, d, i, fy, fo, dn, dp, cqc);
    }
}




#[inline(always)]
fn ijb(k: &mut [u32], d: usize, i: usize, b: i32, c: i32, s: u32) {
    if b < 0 || c < 0 || b >= d as i32 || c >= i as i32 { return; }
    let w = c as usize * d + b as usize;
    if w >= k.len() { return; }
    let cs = k[w];
    let m = ((cs >> 16) & 0xFF) + ((s >> 16) & 0xFF);
    let at = ((cs >> 8) & 0xFF) + ((s >> 8) & 0xFF);
    let o = (cs & 0xFF) + (s & 0xFF);
    k[w] = 0xFF000000 | (m.v(255) << 16) | (at.v(255) << 8) | o.v(255);
}


fn ahj(k: &mut [u32], d: usize, i: usize, fy: i32, fo: i32, dn: i32, dp: i32, s: u32) {
    let mut fy = fy; let mut fo = fo;
    let dx = (dn - fy).gp();
    let bg = -(dp - fo).gp();
    let cr: i32 = if fy < dn { 1 } else { -1 };
    let cq: i32 = if fo < dp { 1 } else { -1 };
    let mut rq = dx + bg;
    
    loop {
        ijb(k, d, i, fy, fo, s);
        if fy == dn && fo == dp { break; }
        let agl = 2 * rq;
        if agl >= bg { rq += bg; fy += cr; }
        if agl <= dx { rq += dx; fo += cq; }
    }
}


fn dqg(k: &mut [u32], d: usize, i: usize, fy: i32, fo: i32, dn: i32, dp: i32, s: u32) {
    ahj(k, d, i, fy, fo, dn, dp, s);
    ahj(k, d, i, fy + 1, fo, dn + 1, dp, s);
    ahj(k, d, i, fy, fo + 1, dn, dp + 1, s);
}


fn tp(k: &mut [u32], pv: u8) {
    for y in k.el() {
        let m = ((*y >> 16) & 0xFF) * pv as u32 / 256;
        let at = ((*y >> 8) & 0xFF) * pv as u32 / 256;
        let o = (*y & 0xFF) * pv as u32 / 256;
        *y = 0xFF000000 | (m << 16) | (at << 8) | o;
    }
}


fn kpg(av: f32, agg: u32) -> u32 {
    let hj = ((1.0 / (av * 0.5 + 1.0)) * 255.0) as u32;
    let hj = hj.v(255);
    let m = ((agg >> 16) & 0xFF) * hj / 255;
    let at = ((agg >> 8) & 0xFF) * hj / 255;
    let o = (agg & 0xFF) * hj / 255;
    0xFF000000 | (m << 16) | (at << 8) | o
}



struct Ckd {
    c: f32,
    ig: f32,
    len: u32,
    ka: u8,
}


struct Cfa {
    b: f32,
    av: f32,
    ig: f32,
    acr: u8,
    ib: f32,
}



#[derive(Clone, Copy, PartialEq)]
pub enum FormulaScene {
    Dw,
    Yh,
    Wh,
    Dr,
    Ajb,
    Pn,
    Aix,
    Adg,
    Ald,
    Zg,
    HoloMatrix,
    Kh,
}



pub struct FormulaRenderer {
    pub amt: FormulaScene,
    pub aev: f32,
    pub ajt: f32,
    pub pt: f32,
    pub dxr: u32,
    frame: u32,
    vpr: alloc::vec::Vec<Ckd>,
    vps: bool,
    
    tpo: bool,
    tpn: alloc::vec::Vec<Cfa>,
    
    czt: Option<Bh>,
    czv: Option<Bh>,
    czu: Option<Bh>,
    czw: Option<Bh>,
    fod: Option<Bh>,
    foc: Option<Bh>,
    gmr: Option<Bh>,
    foe: Option<Bh>,
    jfz: Option<Bh>,
    gmq: Option<Bh>,
}

impl FormulaRenderer {
    pub fn new() -> Self {
        Self {
            amt: FormulaScene::HoloMatrix,
            aev: 0.0,
            ajt: 0.3,
            pt: 2.0,
            dxr: 0xFF00FF66,
            frame: 0,
            vpr: alloc::vec::Vec::new(),
            vps: false,
            tpo: false,
            tpn: alloc::vec::Vec::new(),
            czt: None,
            czv: None,
            czu: None,
            czw: None,
            fod: None,
            foc: None,
            gmr: None,
            foe: None,
            jfz: None,
            gmq: None,
        }
    }

    pub fn bid(&mut self, amt: FormulaScene) {
        self.amt = amt;
        
        self.slu();
    }

    fn slu(&mut self) {
        match self.amt {
            FormulaScene::Dw => { if self.czt.is_none() { self.czt = Some(czt()); } }
            FormulaScene::Yh => { if self.czv.is_none() { self.czv = Some(czv()); } }
            FormulaScene::Wh => { if self.czu.is_none() { self.czu = Some(czu()); } }
            FormulaScene::Dr => { if self.czw.is_none() { self.czw = Some(czw(0.5, 0.2, 16, 12)); } }
            FormulaScene::Ajb => { if self.fod.is_none() { self.fod = Some(fod(0.6)); } }
            FormulaScene::Pn => { if self.foc.is_none() { self.foc = Some(foc(1.5, 10)); } }
            FormulaScene::Aix => { if self.gmr.is_none() { self.gmr = Some(gmr(0.4, 1.2, 3.0, 60)); } }
            FormulaScene::Ald => { if self.foe.is_none() { self.foe = Some(foe()); } }
            FormulaScene::Zg => { if self.jfz.is_none() { self.jfz = Some(onb()); } }
            FormulaScene::Kh => { if self.gmq.is_none() { self.gmq = Some(gmq()); } }
            FormulaScene::HoloMatrix => {  }
            FormulaScene::Adg => {
                if self.czt.is_none() { self.czt = Some(czt()); }
                if self.czv.is_none() { self.czv = Some(czv()); }
                if self.czu.is_none() { self.czu = Some(czu()); }
                if self.czw.is_none() { self.czw = Some(czw(0.5, 0.2, 16, 12)); }
            }
        }
    }

    fn tea(&self) -> Option<&Bh> {
        match self.amt {
            FormulaScene::Dw => self.czt.as_ref(),
            FormulaScene::Yh => self.czv.as_ref(),
            FormulaScene::Wh => self.czu.as_ref(),
            FormulaScene::Dr => self.czw.as_ref(),
            FormulaScene::Ajb => self.fod.as_ref(),
            FormulaScene::Pn => self.foc.as_ref(),
            FormulaScene::Aix => self.gmr.as_ref(),
            FormulaScene::Ald => self.foe.as_ref(),
            FormulaScene::Zg => self.jfz.as_ref(),
            FormulaScene::Kh => self.gmq.as_ref(),
            FormulaScene::HoloMatrix => None, 
            FormulaScene::Adg => None, 
        }
    }

    
    pub fn qs(&mut self) {
        self.frame = self.frame.cn(1);
        match self.amt {
            FormulaScene::Zg => {
                self.aev += 0.012;
                self.ajt = 0.15 + lz(self.frame as f32 * 0.006) * 0.1;
                self.pt = 3.2 + lz(self.frame as f32 * 0.005) * 0.2;
            }
            FormulaScene::Kh => {
                
                self.aev += 0.018;
                self.ajt = 0.20 + lz(self.frame as f32 * 0.007) * 0.12;
                self.pt = 2.2 + lz(self.frame as f32 * 0.004) * 0.15;
            }
            FormulaScene::HoloMatrix => {
                
                
                self.aev = lz(self.frame as f32 * 0.0008) * 0.015;
                self.ajt = lz(self.frame as f32 * 0.0006) * 0.01;
                self.pt = 0.0;
            }
            _ => {
                self.aev += 0.025;
                self.ajt = 0.3 + lz(self.frame as f32 * 0.008) * 0.2;
                self.pt = 2.0 + lz(self.frame as f32 * 0.005) * 0.3;
            }
        }
    }

    
    pub fn tj(&self, k: &mut [u32], d: usize, i: usize) {
        
        #[cfg(target_arch = "x86_64")]
        unsafe {
            use core::arch::x86_64::*;
            let cot = els(0xFF000000u32 as i32);
            let ptr = k.mw() as *mut acb;
            let az = k.len() / 4;
            for a in 0..az {
                ccs(ptr.add(a), cot);
            }
            
            for a in (az * 4)..k.len() {
                k[a] = 0xFF000000;
            }
        }
        #[cfg(not(target_arch = "x86_64"))]
        k.vi(0xFF000000);

        if self.amt == FormulaScene::HoloMatrix {
            
            self.vvx(k, d, i);
            return;
        }

        if self.amt == FormulaScene::Kh {
            
            self.vvk(k, d, i);
        } else {
            
            self.vwk(k, d, i);
        }
        
        
        if self.amt == FormulaScene::Adg {
            self.vwd(k, d, i);
        } else if let Some(mesh) = self.tea() {
            self.vxa(k, d, i, mesh, self.aev, self.ajt, self.pt, self.dxr);
            self.vwv(k, d, i, mesh, self.aev, self.ajt, self.pt);
        }

        
        if self.amt == FormulaScene::Zg || self.amt == FormulaScene::Kh {
            self.vwm(k, d, i);
        }
    }

    fn vxa(&self, k: &mut [u32], d: usize, i: usize, mesh: &Bh,
                        bga: f32, ax: f32, pt: f32, s: u32) {
        for (w, &(q, o)) in mesh.bu.iter().cf() {
            if q >= mesh.lm.len() || o >= mesh.lm.len() { continue; }
            let (fy, fo, alw) = cni(mesh.lm[q], bga, ax, pt, d, i);
            let (dn, dp, aeu) = cni(mesh.lm[o], bga, ax, pt, d, i);
            let bth = (alw + aeu) * 0.5;
            
            let agg = match &mesh.cqd {
                Some(ec) if w < ec.len() => ec[w],
                _ => s,
            };
            let r = kpg(bth, agg);
            dqg(k, d, i, fy, fo, dn, dp, r);
        }
    }

    fn vwv(&self, k: &mut [u32], d: usize, i: usize, mesh: &Bh,
                       bga: f32, ax: f32, pt: f32) {
        for p in &mesh.lm {
            let (cr, cq, qed) = cni(*p, bga, ax, pt, d, i);
            
            for bg in -1..=1i32 {
                for dx in -1..=1i32 {
                    ijb(k, d, i, cr + dx, cq + bg, 0x00FFFFFF);
                }
            }
        }
    }

    fn vwd(&self, k: &mut [u32], d: usize, i: usize) {
        let ab = self.frame as f32 * 0.012;
        let osy = 1.2;
        
        
        let chy: [(f32, u32, FormulaScene); 4] = [
            (0.0, 0xFF00FF66, FormulaScene::Dw),
            (1.5707963, 0xFF6666FF, FormulaScene::Yh),
            (3.14159265, 0xFFFF6600, FormulaScene::Wh),
            (4.7123889, 0xFFFF00FF, FormulaScene::Dr),
        ];
        
        for (qil, s, amt) in &chy {
            let osx = ab + qil;
            let mp = osy * rk(osx);
            let jig = osy * lz(osx);
            
            let mesh = match amt {
                FormulaScene::Dw => self.czt.as_ref(),
                FormulaScene::Yh => self.czv.as_ref(),
                FormulaScene::Wh => self.czu.as_ref(),
                FormulaScene::Dr => self.czw.as_ref(),
                _ => None,
            };
            
            if let Some(mesh) = mesh {
                
                for &(q, o) in &mesh.bu {
                    if q >= mesh.lm.len() || o >= mesh.lm.len() { continue; }
                    let asf = V3 { b: mesh.lm[q].b * 0.35 + mp, c: mesh.lm[q].c * 0.35, av: mesh.lm[q].av * 0.35 + jig };
                    let cci = V3 { b: mesh.lm[o].b * 0.35 + mp, c: mesh.lm[o].c * 0.35, av: mesh.lm[o].av * 0.35 + jig };
                    let (fy, fo, alw) = cni(asf, self.aev * 0.5, self.ajt, self.pt + 1.0, d, i);
                    let (dn, dp, aeu) = cni(cci, self.aev * 0.5, self.ajt, self.pt + 1.0, d, i);
                    let bth = (alw + aeu) * 0.5;
                    let r = kpg(bth, *s);
                    dqg(k, d, i, fy, fo, dn, dp, r);
                }
            }
        }
    }

    
    
    
    
    
    
    
    
    
    
    
    
    fn vvx(&self, k: &mut [u32], d: usize, i: usize) {
        
        const Pm: [u64; 16] = [
            0b11111_10001_10001_11111_00001_00001_00001, 
            0b00100_00100_11111_00100_01010_10001_00000, 
            0b11111_00001_00010_00100_01000_10000_11111, 
            0b01110_00010_00010_00010_00010_00010_11111, 
            0b11111_10000_10000_11110_00001_00001_11110, 
            0b00100_01110_10101_00100_00100_00100_00100, 
            0b01010_01010_11111_01010_01010_00100_00100, 
            0b11111_00001_11111_10000_10000_10000_11111, 
            0b10001_10001_01010_00100_01010_10001_10001, 
            0b11111_10001_10001_10001_10001_10001_11111, 
            0b00100_01110_10101_10101_01110_00100_00100, 
            0b10001_01010_00100_11111_00100_00100_00100, 
            0b11111_00100_00100_01110_10001_10001_01110, 
            0b10100_10100_10100_10110_10001_10001_01110, 
            0b01110_10001_10001_01110_00100_01010_10001, 
            0b00100_01010_10001_11111_10001_01010_00100, 
        ];

        
        
        
        const BBT_: usize = 10;        
        const SB_: usize = 18;    
        const CIB_: usize = BBT_ * SB_; 
        const QU_: f32 = 16.0;         
        const BJG_: f32 = 0.5;           
        const CYZ_: f32 = 5.5;      

        let frame = self.frame;
        
        let qvu = frame as f32 * 0.018; 

        let qvr = self.aev; 
        let qvq = self.ajt;

        let cx = (d as f32) * 0.5;
        let ae = (i as f32) * 0.5;

        
        {
            let gmi = 40i32;
            let xg = 0.85 + 0.15 * lz(frame as f32 * 0.03);
            let ooe = gmi * gmi;
            for ix in -gmi..=gmi {
                let x = ae as i32 + ix;
                if x < 0 || x >= i as i32 { continue; }
                let dbp = ix * ix;
                for kb in -gmi..=gmi {
                    let us = kb * kb + dbp;
                    if us > ooe { continue; }
                    let bwi = cx as i32 + kb;
                    if bwi < 0 || bwi >= d as i32 { continue; }
                    let ab = 1.0 - (us as f32 / ooe as f32);
                    let ab = ab * ab * xg;
                    let at = (ab * 45.0) as u32;
                    let o = (ab * 15.0) as u32;
                    if at > 0 {
                        let tq = 0xFF000000 | (at.v(255) << 8) | o.v(255);
                        let w = x as usize * d + bwi as usize;
                        if w < k.len() {
                            let cs = k[w];
                            let ftg = ((cs >> 16) & 0xFF) + ((tq >> 16) & 0xFF);
                            let dho = ((cs >> 8) & 0xFF) + ((tq >> 8) & 0xFF);
                            let aaa = (cs & 0xFF) + (tq & 0xFF);
                            k[w] = 0xFF000000 | (ftg.v(255) << 16) | (dho.v(255) << 8) | aaa.v(255);
                        }
                    }
                }
            }
        }

        
        for adq in 0..CIB_ {
            let mz = adq / SB_;
            let gk = adq % SB_;

            
            let dv = (adq as u32).hx(2654435761).cn(374761393);
            let hzj = dv.hx(1664525).cn(1013904223);
            let wgj = hzj.hx(1664525).cn(1013904223);

            
            let iku = (gk as f32 / SB_ as f32) * 6.2831853;
            let qij = (dv % 1000) as f32 / 1000.0 * 0.3 - 0.15;
            let bdb = iku + qij;

            
            let vzf = (mz as f32 + 0.3) / BBT_ as f32;
            let dy = 0.3 + vzf * CYZ_;

            
            let ffl = dy * rk(bdb);
            let rmb = dy * lz(bdb);

            
            let rme = (hzj % 10000) as f32 / 10000.0 * QU_;
            
            let rmf = rme - (qvu % QU_);
            let rmd = ((rmf % QU_) + QU_) % QU_ + BJG_;

            
            let ig = 0.012 + (wgj % 1000) as f32 / 1000.0 * 0.02;
            let acr = 6 + (dv >> 12) as usize % 12;
            let nck: f32 = 0.30;
            let ib = (hzj >> 8) as f32 / 256.0;
            let fze: f32 = 3.0 + dy * 0.5;

            
            let iep = fze + acr as f32 * nck + 2.0;
            let tnv = fze * 0.5 - ((frame as f32 * ig + ib * fze) % iep);

            for nc in 0..acr {
                let alk = tnv + nc as f32 * nck;
                if alk > fze || alk < -fze { continue; }

                
                let iz = rmb + alk;
                let p = V3 { b: ffl, c: iz, av: rmd };

                
                let cmk = dvg(cmj(p, qvr), qvq);
                if cmk.av < BJG_ { continue; }

                let ai = nv(cmk);
                let (cr, cq) = mlk(ai, d, i);

                
                let fzh = 1.0 / cmk.av;
                let bv = (fzh * fzh * 30.0).am(1.0).v(4.0) as i32;

                let cyg = bv * 5;
                let cyf = bv * 7;
                if cr + cyg < -5 || cr - cyg > d as i32 + 5 { continue; }
                if cq + cyf < -5 || cq - cyf > i as i32 + 5 { continue; }

                
                let pvu = nc as f32 / acr as f32;
                let fad = (1.0 - pvu) * (1.0 - pvu);
                
                let hfv = cxr(-cmk.av * 0.12);
                let kt = fad * hfv;

                if kt < 0.015 { continue; }

                
                let cqy = (dv.hx(nc as u32 + 1).cn(frame / 5)) as usize % 16;
                let ka = Pm[cqy];

                
                let (btu, bmh, aiv) = if nc == 0 {
                    
                    ((kt * 200.0) as u32, (kt * 255.0) as u32, (kt * 210.0) as u32)
                } else if nc <= 2 {
                    ((kt * 50.0) as u32, (kt * 255.0) as u32, (kt * 70.0) as u32)
                } else {
                    ((kt * 8.0) as u32, (kt * 230.0) as u32, (kt * 20.0) as u32)
                };
                let btu = btu.v(255);
                let bmh = bmh.v(255);
                let aiv = aiv.v(255);
                let s = 0xFF000000 | (btu << 16) | (bmh << 8) | aiv;

                
                let mp = cr - cyg / 2;
                let qw = cq - cyf / 2;
                let xpj = nc <= 1; 
                for br in 0..7i32 {
                    for bj in 0..5i32 {
                        let deh = br * 5 + bj;
                        if (ka >> deh) & 1 == 0 { continue; }
                        for x in 0..bv {
                            let sc = qw + br * bv + x;
                            if sc < 0 || sc >= i as i32 { continue; }
                            let fte = sc as usize * d;
                            for y in 0..bv {
                                let jf = mp + bj * bv + y;
                                if jf < 0 || jf >= d as i32 { continue; }
                                let w = fte + jf as usize;
                                if xpj {
                                    let cs = k[w];
                                    let ftg = ((cs >> 16) & 0xFF) + ((s >> 16) & 0xFF);
                                    let dho = ((cs >> 8) & 0xFF) + ((s >> 8) & 0xFF);
                                    let aaa = (cs & 0xFF) + (s & 0xFF);
                                    k[w] = 0xFF000000 | (ftg.v(255) << 16) | (dho.v(255) << 8) | aaa.v(255);
                                } else {
                                    k[w] = s;
                                }
                            }
                        }
                    }
                }

                
                if nc == 0 && bmh > 80 && bv >= 2 {
                    let kzg = bmh / 4;
                    let bzv = 0xFF000000 | (kzg << 8);
                    let rpd: [(i32,i32); 4] = [
                        (mp - 1, qw - 1), (mp + cyg, qw - 1),
                        (mp - 1, qw + cyf), (mp + cyg, qw + cyf),
                    ];
                    for &(qz, ub) in &rpd {
                        if qz >= 0 && qz < d as i32 && ub >= 0 && ub < i as i32 {
                            ijb(k, d, i, qz, ub, bzv);
                        }
                    }
                }
            }
        }

        
        
        {
            let nzr: u32 = 0x14; 
            let iyp = ae as i32 + 30; 
            let lpk = 12; 
            let orz = 16; 
            let tho = (frame as f32 * 0.35) % 48.0; 

            
            for a in 0..lpk {
                let ab = (a as f32 + tho / 48.0 * (lpk as f32 / 4.0)) / lpk as f32;
                let ab = ab.v(0.99);
                let abi = iyp + ((1.0 - ab) * (1.0 - ab) * (i as f32 - iyp as f32)) as i32;
                if abi < 0 || abi >= i as i32 { continue; }
                let hfv = (1.0 - ab) * (1.0 - ab);
                let at = (nzr as f32 * hfv * 1.5) as u32;
                if at < 3 { continue; }
                let een = 0xFF000000 | (at.v(255) << 8) | (at.v(255) / 4);
                let fte = abi as usize * d;
                
                let mut y = 0;
                while y < d {
                    let w = fte + y;
                    if w < k.len() {
                        let cs = k[w];
                        let dho = ((cs >> 8) & 0xFF) + ((een >> 8) & 0xFF);
                        let aaa = (cs & 0xFF) + (een & 0xFF);
                        k[w] = 0xFF000000 | (dho.v(255) << 8) | aaa.v(255);
                    }
                    y += 2; 
                }
            }

            
            for a in 0..orz {
                let fp = (a as f32 / orz as f32) * d as f32;
                let fp = fp as i32;
                let imd = i as i32 - 1;
                let au = (imd - iyp).am(1);
                let mut e = 0;
                while e < au {
                    let ab = e as f32 / au as f32;
                    let cq = iyp + e;
                    if cq >= i as i32 { break; }
                    if cq < 0 { e += 4; continue; }
                    let cr = cx as i32 + ((fp - cx as i32) as f32 * ab * ab) as i32;
                    if cr >= 0 && cr < d as i32 {
                        let hfv = ab * ab;
                        let at = (nzr as f32 * hfv * 1.2) as u32;
                        if at >= 2 {
                            let w = cq as usize * d + cr as usize;
                            if w < k.len() {
                                let een = 0xFF000000 | (at.v(255) << 8) | (at.v(255) / 4);
                                let cs = k[w];
                                let dho = ((cs >> 8) & 0xFF) + ((een >> 8) & 0xFF);
                                let aaa = (cs & 0xFF) + (een & 0xFF);
                                k[w] = 0xFF000000 | (dho.v(255) << 8) | aaa.v(255);
                            }
                        }
                    }
                    e += 3; 
                }
            }
        }

        
        
        {
            let orx = 30;
            for a in 0..orx {
                let dv = (a as u32).hx(2654435761).cn(frame.hx(7));
                let hzj = dv.hx(1664525).cn(1013904223);
                let hg = (a as f32 / orx as f32) * 6.2831853 + (dv % 1000) as f32 * 0.001;
                
                let qnh = 80.0 + (hzj % 400) as f32;
                let can = ((frame.hx(3).cn(dv)) % 120) as f32 / 120.0;
                let la = qnh + can * 400.0;
                let len = 3.0 + can * 15.0;

                let apn = rk(hg);
                let aql = lz(hg);
                let fy = cx as i32 + (la * apn) as i32;
                let fo = ae as i32 + (la * aql) as i32;
                let dn = cx as i32 + ((la + len) * apn) as i32;
                let dp = ae as i32 + ((la + len) * aql) as i32;

                let yx = ((1.0 - can) * 40.0) as u32;
                if yx < 2 { continue; }
                let wuz = 0xFF000000 | (yx.v(255) << 8) | (yx.v(255) / 3);
                ahj(k, d, i, fy, fo, dn, dp, wuz);
            }
        }

    }

    
    fn vvk(&self, k: &mut [u32], d: usize, i: usize) {
        let cx = d as f32 * 0.5;
        let ae = i as f32 * 0.5;
        let lkv = ahn(cx * cx + ae * ae);

        for c in 0..i {
            for b in 0..d {
                let w = c * d + b;
                if w >= k.len() { break; }
                
                let ab = c as f32 / i as f32;
                let bdm = (8.0 + ab * 12.0) as u32;
                let bji = (12.0 + ab * 20.0) as u32;
                let cdd = (30.0 + ab * 35.0) as u32;
                
                let dx = b as f32 - cx;
                let bg = c as f32 - ae;
                let la = ahn(dx * dx + bg * bg) / lkv;
                let fyi = 1.0 - la * la * 0.6;
                let m = ((bdm as f32 * fyi) as u32).v(255);
                let at = ((bji as f32 * fyi) as u32).v(255);
                let o = ((cdd as f32 * fyi) as u32).v(255);
                k[w] = 0xFF000000 | (m << 16) | (at << 8) | o;
            }
        }

        
        let oca = i * 45 / 100; 
        let lak: u32 = 0x18;
        let orr = 10;
        for a in 0..orr {
            let ab = a as f32 / orr as f32;
            let abi = oca + ((1.0 - ab) * (1.0 - ab) * (i - oca) as f32) as usize;
            if abi >= i { continue; }
            let yx = (1.0 - ab) * (1.0 - ab);
            let at = (lak as f32 * yx * 1.5) as u32;
            if at < 2 { continue; }
            let een = 0xFF000000 | (at.v(60) / 3 << 16) | (at.v(60) / 2 << 8) | at.v(60);
            let fte = abi * d;
            let mut y = 0;
            while y < d {
                let w = fte + y;
                if w < k.len() {
                    let cs = k[w];
                    let ftg = ((cs >> 16) & 0xFF) + ((een >> 16) & 0xFF);
                    let dho = ((cs >> 8) & 0xFF) + ((een >> 8) & 0xFF);
                    let aaa = (cs & 0xFF) + (een & 0xFF);
                    k[w] = 0xFF000000 | (ftg.v(255) << 16) | (dho.v(255) << 8) | aaa.v(255);
                }
                y += 2;
            }
        }
    }

    
    fn vwm(&self, k: &mut [u32], d: usize, i: usize) {
        
        let bgq = (i as f32) * 2.0;
        let fsa = (self.frame as f32 * 1.8) % bgq;
        let icj = if fsa > i as f32 { bgq - fsa } else { fsa };
        let icj = icj as i32;

        for c in 0..i {
            let mu = c * d;
            let cub = mu + d;
            if cub > k.len() { break; }

            
            if c % 3 == 0 {
                for y in k[mu..cub].el() {
                    let m = ((*y >> 16) & 0xFF) * 200 / 256;
                    let at = ((*y >> 8) & 0xFF) * 200 / 256;
                    let o = (*y & 0xFF) * 200 / 256;
                    *y = 0xFF000000 | (m << 16) | (at << 8) | o;
                }
            }

            
            let la = (c as i32 - icj).eki();
            if la < 6 {
                let bma = (6 - la) * 5;
                for y in k[mu..cub].el() {
                    let m = (((*y >> 16) & 0xFF) + bma).v(255);
                    let at = (((*y >> 8) & 0xFF) + bma).v(255);
                    let o = ((*y & 0xFF) + bma).v(255);
                    *y = 0xFF000000 | (m << 16) | (at << 8) | o;
                }
            }

            
            let uaf = (c as u32).hx(2654435761).cn(self.frame);
            if uaf % 97 < 3 {
                
                let acn = 2;
                let br = &mut k[mu..cub];
                if d > acn {
                    for b in (acn..d).vv() {
                        br[b] = br[b - acn];
                    }
                    for b in 0..acn {
                        br[b] = 0xFF000000;
                    }
                }
            }
        }
    }

    fn vwk(&self, k: &mut [u32], d: usize, i: usize) {
        
        let uwh = d / 12;
        let frame = self.frame;
        
        for a in 0..uwh {
            
            let dv = (a as u32).hx(2654435761);
            let b = (dv % d as u32) as i32;
            let ig = 2 + (dv >> 16) % 5;
            let len = 4 + (dv >> 8) % 12;
            let xww = ((frame.hx(ig)) % (i as u32 + len)) as i32;
            
            for fb in 0..len as i32 {
                let c = xww - fb;
                if c >= 0 && c < i as i32 {
                    let yx = (len as i32 - fb) as u32 * 255 / len as u32;
                    let at = (yx * 180 / 255).v(255);
                    let s = 0xFF000000 | (at << 8);
                    let w = c as usize * d + b as usize;
                    if w < k.len() {
                        k[w] = s;
                    }
                }
            }
        }
    }
}
