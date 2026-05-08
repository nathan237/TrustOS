







#[derive(Clone, Copy)]
pub struct V3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

#[derive(Clone, Copy)]
pub struct Qt {
    pub x: f32,
    pub y: f32,
}



pub use crate::math::{eu, hr, ra};


#[inline(always)]
fn jrl(mut a: f32) -> f32 {
    const By: f32 = 3.14159265;
    const Kr: f32 = 6.2831853;
    while a > By { a -= Kr; }
    while a < -By { a += Kr; }
    a
}



#[inline(always)]
pub fn bbo(x: f32) -> f32 {
    if x < -6.0 { return 0.0; }
    if x > 0.0 { return 1.0; }
    
    let t = 1.0 + x * 0.125; 
    let t = if t < 0.0 { 0.0 } else { t };
    let t = t * t; 
    let t = t * t; 
    t * t           
}



#[inline(always)]
fn rotate_y(v: V3, a: f32) -> V3 {
    let c = hr(a);
    let j = eu(a);
    V3 { x: v.x * c + v.z * j, y: v.y, z: -v.x * j + v.z * c }
}

#[inline(always)]
fn rotate_x(v: V3, a: f32) -> V3 {
    let c = hr(a);
    let j = eu(a);
    V3 { x: v.x, y: v.y * c - v.z * j, z: v.y * j + v.z * c }
}

#[inline(always)]
fn joo(v: V3, dz: f32) -> V3 {
    V3 { x: v.x, y: v.y, z: v.z + dz }
}


#[inline(always)]
fn project(v: V3) -> Qt {
    if v.z.abs() < 0.001 {
        Qt { x: 0.0, y: 0.0 }
    } else {
        Qt { x: v.x / v.z, y: v.y / v.z }
    }
}


#[inline(always)]
fn gzd(aa: Qt, w: usize, h: usize) -> (i32, i32) {
    let scale = w.min(h) as f32 * 0.45;
    let am = (aa.x * scale) + (w as f32 * 0.5);
    let ak = (-aa.y * scale) + (h as f32 * 0.5);
    (am as i32, ak as i32)
}


#[inline(always)]
fn avl(v: V3, angle_y: f32, angle_x: f32, dz: f32, w: usize, h: usize) -> (i32, i32, f32) {
    let auu = rotate_x(rotate_y(v, angle_y), angle_x);
    let ceq = joo(auu, dz);
    let jdl = gzd(project(ceq), w, h);
    (jdl.0, jdl.1, ceq.z)
}



pub struct Ai {
    pub vertices: alloc::vec::Vec<V3>,
    pub edges: alloc::vec::Vec<(usize, usize)>,
    
    pub edge_colors: Option<alloc::vec::Vec<u32>>,
    
    pub faces: Option<alloc::vec::Vec<(usize, usize, usize)>>,
    
    pub face_colors: Option<alloc::vec::Vec<u32>>,
}

pub fn mesh_cube() -> Ai {
    let v = alloc::vec![
        V3 { x: -0.5, y: -0.5, z: -0.5 }, V3 { x:  0.5, y: -0.5, z: -0.5 },
        V3 { x:  0.5, y:  0.5, z: -0.5 }, V3 { x: -0.5, y:  0.5, z: -0.5 },
        V3 { x: -0.5, y: -0.5, z:  0.5 }, V3 { x:  0.5, y: -0.5, z:  0.5 },
        V3 { x:  0.5, y:  0.5, z:  0.5 }, V3 { x: -0.5, y:  0.5, z:  0.5 },
    ];
    let e = alloc::vec![
        (0,1),(1,2),(2,3),(3,0), 
        (4,5),(5,6),(6,7),(7,4), 
        (0,4),(1,5),(2,6),(3,7), 
    ];
    let f = alloc::vec![
        
        (0, 1, 2), (0, 2, 3),
        
        (5, 4, 7), (5, 7, 6),
        
        (3, 2, 6), (3, 6, 7),
        
        (4, 5, 1), (4, 1, 0),
        
        (1, 5, 6), (1, 6, 2),
        
        (4, 0, 3), (4, 3, 7),
    ];
    Ai { vertices: v, edges: e, edge_colors: None, faces: Some(f), face_colors: None }
}

pub fn mesh_pyramid() -> Ai {
    let v = alloc::vec![
        V3 { x: -0.5, y: -0.5, z: -0.5 }, V3 { x:  0.5, y: -0.5, z: -0.5 },
        V3 { x:  0.5, y: -0.5, z:  0.5 }, V3 { x: -0.5, y: -0.5, z:  0.5 },
        V3 { x:  0.0, y:  0.7, z:  0.0 },
    ];
    let e = alloc::vec![
        (0,1),(1,2),(2,3),(3,0), 
        (0,4),(1,4),(2,4),(3,4), 
    ];
    let f = alloc::vec![
        
        (0, 1, 2), (0, 2, 3),
        
        (0, 1, 4), (1, 2, 4), (2, 3, 4), (3, 0, 4),
    ];
    Ai { vertices: v, edges: e, edge_colors: None, faces: Some(f), face_colors: None }
}

pub fn mesh_diamond() -> Ai {
    let v = alloc::vec![
        V3 { x:  0.0, y:  0.7, z:  0.0 }, 
        V3 { x: -0.5, y:  0.0, z: -0.5 }, V3 { x:  0.5, y:  0.0, z: -0.5 },
        V3 { x:  0.5, y:  0.0, z:  0.5 }, V3 { x: -0.5, y:  0.0, z:  0.5 },
        V3 { x:  0.0, y: -0.7, z:  0.0 }, 
    ];
    let e = alloc::vec![
        (1,2),(2,3),(3,4),(4,1), 
        (0,1),(0,2),(0,3),(0,4), 
        (5,1),(5,2),(5,3),(5,4), 
    ];
    let f = alloc::vec![
        
        (0, 1, 2), (0, 2, 3), (0, 3, 4), (0, 4, 1),
        
        (5, 2, 1), (5, 3, 2), (5, 4, 3), (5, 1, 4),
    ];
    Ai { vertices: v, edges: e, edge_colors: None, faces: Some(f), face_colors: None }
}

pub fn mesh_torus(bcm: f32, aro: f32, major_seg: usize, minor_seg: usize) -> Ai {
    let mut verts = alloc::vec::Vec::with_capacity(major_seg * minor_seg);
    let mut edges = alloc::vec::Vec::new();
    
    for i in 0..major_seg {
        let acz = (i as f32 / major_seg as f32) * 6.2831853;
        let wb = hr(acz);
        let uz = eu(acz);
        for ay in 0..minor_seg {
            let aij = (ay as f32 / minor_seg as f32) * 6.2831853;
            let cp = hr(aij);
            let sp = eu(aij);
            let x = (bcm + aro * cp) * wb;
            let z = (bcm + aro * cp) * uz;
            let y = aro * sp;
            verts.push(V3 { x, y, z });
            
            let idx = i * minor_seg + ay;
            let nkd = i * minor_seg + (ay + 1) % minor_seg;
            edges.push((idx, nkd));
            let nkb = ((i + 1) % major_seg) * minor_seg + ay;
            edges.push((idx, nkb));
        }
    }
    Ai { vertices: verts, edges, edge_colors: None, faces: None, face_colors: None }
}

pub fn mesh_icosphere(radius: f32) -> Ai {
    let t = (1.0 + ra(5.0)) / 2.0;
    let j = radius / ra(1.0 + t * t);
    let a = j;
    let b = j * t;
    
    let v = alloc::vec![
        V3 { x: -a, y:  b, z: 0.0 }, V3 { x:  a, y:  b, z: 0.0 },
        V3 { x: -a, y: -b, z: 0.0 }, V3 { x:  a, y: -b, z: 0.0 },
        V3 { x: 0.0, y: -a, z:  b }, V3 { x: 0.0, y:  a, z:  b },
        V3 { x: 0.0, y: -a, z: -b }, V3 { x: 0.0, y:  a, z: -b },
        V3 { x:  b, y: 0.0, z: -a }, V3 { x:  b, y: 0.0, z:  a },
        V3 { x: -b, y: 0.0, z: -a }, V3 { x: -b, y: 0.0, z:  a },
    ];
    let e = alloc::vec![
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
    Ai { vertices: v, edges: e, edge_colors: None, faces: None, face_colors: None }
}

pub fn mesh_grid(cw: f32, divisions: usize) -> Ai {
    let mut verts = alloc::vec::Vec::new();
    let mut edges = alloc::vec::Vec::new();
    let step = (cw * 2.0) / divisions as f32;
    let ae = divisions + 1;
    
    for i in 0..ae {
        for ay in 0..ae {
            let x = -cw + i as f32 * step;
            let z = -cw + ay as f32 * step;
            verts.push(V3 { x, y: -0.3, z });
            let idx = i * ae + ay;
            if ay + 1 < ae { edges.push((idx, idx + 1)); }
            if i + 1 < ae { edges.push((idx, idx + ae)); }
        }
    }
    Ai { vertices: verts, edges, edge_colors: None, faces: None, face_colors: None }
}



pub fn mesh_penger() -> Ai {
    let mut verts = alloc::vec::Vec::with_capacity(80);
    let mut edges = alloc::vec::Vec::with_capacity(120);

    
    
    let bqs = verts.len(); 
    let fv = 0.28; let bd = 0.22; let dc = -0.40;
    verts.push(V3 { x: -fv, y: dc, z: -bd }); 
    verts.push(V3 { x:  fv, y: dc, z: -bd }); 
    verts.push(V3 { x:  fv, y: dc, z:  bd }); 
    verts.push(V3 { x: -fv, y: dc, z:  bd }); 
    
    let bkw = verts.len(); 
    verts.push(V3 { x: -fv, y: 0.0, z: -bd }); 
    verts.push(V3 { x:  fv, y: 0.0, z: -bd }); 
    verts.push(V3 { x:  fv, y: 0.0, z:  bd }); 
    verts.push(V3 { x: -fv, y: 0.0, z:  bd }); 
    
    let bqt = verts.len(); 
    let gr = 0.24; let fcn = 0.20;
    verts.push(V3 { x: -gr, y: 0.30, z: -fcn }); 
    verts.push(V3 { x:  gr, y: 0.30, z: -fcn }); 
    verts.push(V3 { x:  gr, y: 0.30, z:  fcn }); 
    verts.push(V3 { x: -gr, y: 0.30, z:  fcn }); 

    
    edges.push((bqs, bqs+1)); edges.push((bqs+1, bqs+2));
    edges.push((bqs+2, bqs+3)); edges.push((bqs+3, bqs));
    
    edges.push((bkw, bkw+1)); edges.push((bkw+1, bkw+2));
    edges.push((bkw+2, bkw+3)); edges.push((bkw+3, bkw));
    
    edges.push((bqt, bqt+1)); edges.push((bqt+1, bqt+2));
    edges.push((bqt+2, bqt+3)); edges.push((bqt+3, bqt));
    
    for i in 0..4 { edges.push((bqs + i, bkw + i)); }
    for i in 0..4 { edges.push((bkw + i, bqt + i)); }

    
    let btd = verts.len(); 
    let xc = 0.26; let aqz = 0.22; let eoz = 0.32;
    verts.push(V3 { x: -xc, y: eoz, z: -aqz }); 
    verts.push(V3 { x:  xc, y: eoz, z: -aqz }); 
    verts.push(V3 { x:  xc, y: eoz, z:  aqz }); 
    verts.push(V3 { x: -xc, y: eoz, z:  aqz }); 
    
    let ara = verts.len(); 
    let epu = 0.24; let epq = 0.20;
    verts.push(V3 { x: -epu, y: 0.65, z: -epq }); 
    verts.push(V3 { x:  epu, y: 0.65, z: -epq }); 
    verts.push(V3 { x:  epu, y: 0.65, z:  epq }); 
    verts.push(V3 { x: -epu, y: 0.65, z:  epq }); 

    
    edges.push((btd, btd+1)); edges.push((btd+1, btd+2));
    edges.push((btd+2, btd+3)); edges.push((btd+3, btd));
    edges.push((ara, ara+1)); edges.push((ara+1, ara+2));
    edges.push((ara+2, ara+3)); edges.push((ara+3, ara));
    for i in 0..4 { edges.push((btd + i, ara + i)); }

    
    
    let hhg = verts.len(); 
    let kbc = 0.0; let kbd = -0.05; let kbg = -bd - 0.01;
    let kbe = 0.16; let kbf = 0.28;
    for i in 0..8u32 {
        let cc = i as f32 * 0.7853982; 
        let x = kbc + kbe * hr(cc);
        let y = kbd + kbf * eu(cc);
        verts.push(V3 { x, y, z: kbg });
    }
    for i in 0..8 { edges.push((hhg + i, hhg + (i + 1) % 8)); }

    
    let atr = 0.50; let bmc = -aqz - 0.01; let aoc = 0.04;
    
    let el = verts.len(); 
    verts.push(V3 { x: -0.10 - aoc, y: atr - aoc, z: bmc });
    verts.push(V3 { x: -0.10 + aoc, y: atr - aoc, z: bmc });
    verts.push(V3 { x: -0.10 + aoc, y: atr + aoc, z: bmc });
    verts.push(V3 { x: -0.10 - aoc, y: atr + aoc, z: bmc });
    edges.push((el, el+1)); edges.push((el+1, el+2)); edges.push((el+2, el+3)); edges.push((el+3, el));
    
    let ajp = verts.len(); 
    verts.push(V3 { x: 0.10 - aoc, y: atr - aoc, z: bmc });
    verts.push(V3 { x: 0.10 + aoc, y: atr - aoc, z: bmc });
    verts.push(V3 { x: 0.10 + aoc, y: atr + aoc, z: bmc });
    verts.push(V3 { x: 0.10 - aoc, y: atr + aoc, z: bmc });
    edges.push((ajp, ajp+1)); edges.push((ajp+1, ajp+2)); edges.push((ajp+2, ajp+3)); edges.push((ajp+3, ajp));
    
    edges.push((el, el+2)); edges.push((el+1, el+3));
    edges.push((ajp, ajp+2)); edges.push((ajp+1, ajp+3));

    
    let egq = 0.42; let egr = -aqz - 0.03;
    let bam = verts.len(); 
    verts.push(V3 { x:  0.00, y: egq + 0.03, z: egr }); 
    verts.push(V3 { x: -0.04, y: egq,        z: egr }); 
    verts.push(V3 { x:  0.04, y: egq,        z: egr }); 
    verts.push(V3 { x:  0.00, y: egq - 0.02, z: egr - 0.02 }); 
    edges.push((bam, bam+1)); edges.push((bam, bam+2)); edges.push((bam+1, bam+2));
    edges.push((bam+1, bam+3)); edges.push((bam+2, bam+3)); edges.push((bam, bam+3));

    
    let aof = -0.42; let bzw = 0.04; let aoe = 0.10; let aod = 0.14;
    
    let adr = verts.len(); 
    verts.push(V3 { x: -0.16 - aoe, y: aof,          z: -aod });
    verts.push(V3 { x: -0.16 + aoe, y: aof,          z: -aod });
    verts.push(V3 { x: -0.16 + aoe, y: aof,          z:  aod });
    verts.push(V3 { x: -0.16 - aoe, y: aof,          z:  aod });
    verts.push(V3 { x: -0.16 - aoe, y: aof - bzw, z: -aod });
    verts.push(V3 { x: -0.16 + aoe, y: aof - bzw, z: -aod });
    verts.push(V3 { x: -0.16 + aoe, y: aof - bzw, z:  aod });
    verts.push(V3 { x: -0.16 - aoe, y: aof - bzw, z:  aod });
    edges.push((adr, adr+1)); edges.push((adr+1, adr+2)); edges.push((adr+2, adr+3)); edges.push((adr+3, adr));
    edges.push((adr+4, adr+5)); edges.push((adr+5, adr+6)); edges.push((adr+6, adr+7)); edges.push((adr+7, adr+4));
    for i in 0..4 { edges.push((adr + i, adr + 4 + i)); }
    
    let ko = verts.len(); 
    verts.push(V3 { x: 0.16 - aoe, y: aof,          z: -aod });
    verts.push(V3 { x: 0.16 + aoe, y: aof,          z: -aod });
    verts.push(V3 { x: 0.16 + aoe, y: aof,          z:  aod });
    verts.push(V3 { x: 0.16 - aoe, y: aof,          z:  aod });
    verts.push(V3 { x: 0.16 - aoe, y: aof - bzw, z: -aod });
    verts.push(V3 { x: 0.16 + aoe, y: aof - bzw, z: -aod });
    verts.push(V3 { x: 0.16 + aoe, y: aof - bzw, z:  aod });
    verts.push(V3 { x: 0.16 - aoe, y: aof - bzw, z:  aod });
    edges.push((ko, ko+1)); edges.push((ko+1, ko+2)); edges.push((ko+2, ko+3)); edges.push((ko+3, ko));
    edges.push((ko+4, ko+5)); edges.push((ko+5, ko+6)); edges.push((ko+6, ko+7)); edges.push((ko+7, ko+4));
    for i in 0..4 { edges.push((ko + i, ko + 4 + i)); }

    
    
    let cfl = verts.len(); 
    verts.push(V3 { x: -fv - 0.01, y:  0.20, z:  0.00 }); 
    verts.push(V3 { x: -fv - 0.14, y: -0.05, z: -0.04 }); 
    verts.push(V3 { x: -fv - 0.10, y: -0.25, z: -0.02 }); 
    verts.push(V3 { x: -fv - 0.01, y: -0.15, z:  0.00 }); 
    edges.push((cfl, cfl+1)); edges.push((cfl+1, cfl+2)); edges.push((cfl+2, cfl+3)); edges.push((cfl+3, cfl));
    
    let aep = verts.len(); 
    verts.push(V3 { x: fv + 0.01, y:  0.20, z:  0.00 });
    verts.push(V3 { x: fv + 0.14, y: -0.05, z: -0.04 });
    verts.push(V3 { x: fv + 0.10, y: -0.25, z: -0.02 });
    verts.push(V3 { x: fv + 0.01, y: -0.15, z:  0.00 });
    edges.push((aep, aep+1)); edges.push((aep+1, aep+2)); edges.push((aep+2, aep+3)); edges.push((aep+3, aep));

    
    let cvw = verts.len(); 
    let cvx = 0.65; let hoz = 0.22;
    for i in 0..8u32 {
        let cc = i as f32 * 0.7853982; 
        let x = hoz * hr(cc);
        let z = hoz * 0.9 * eu(cc);
        let pwc = 0.08 * hr(cc * 2.0); 
        verts.push(V3 { x, y: cvx + pwc, z });
    }
    for i in 0..8 { edges.push((cvw + i, cvw + (i + 1) % 8)); }
    
    edges.push((cvw + 0, ara + 1)); 
    edges.push((cvw + 2, ara + 2)); 
    edges.push((cvw + 4, ara + 3)); 
    edges.push((cvw + 6, ara));     

    Ai { vertices: verts, edges, edge_colors: None, faces: None, face_colors: None }
}



pub fn inm() -> Ai {
    let mut verts = alloc::vec::Vec::with_capacity(220);
    let mut edges = alloc::vec::Vec::with_capacity(350);

    let mo: f32 = 0.18;     
    let depth: f32 = 0.06;  
    let pitch: f32 = 0.25;  
    let start_x: f32 = -pitch * 3.0; 
    let top: f32 = 0.20;    
    let age: f32 = -0.20;   
    let mid: f32 = 0.0;     

    
    fn bko(verts: &mut alloc::vec::Vec<V3>, edges: &mut alloc::vec::Vec<(usize, usize)>,
                    points: &[(f32, f32)], fh: f32, depth: f32) {
        let base = verts.len();
        for &(p, o) in points {
            verts.push(V3 { x: fh + p, y: o, z: -depth }); 
            verts.push(V3 { x: fh + p, y: o, z: depth });  
        }
        for i in 0..points.len() {
            
            edges.push((base + i * 2, base + i * 2 + 1));
            
            if i + 1 < points.len() {
                edges.push((base + i * 2, base + (i + 1) * 2));
            }
            
            if i + 1 < points.len() {
                edges.push((base + i * 2 + 1, base + (i + 1) * 2 + 1));
            }
        }
    }

    
    let fh = start_x;
    bko(&mut verts, &mut edges, &[(0.0, top), (mo, top)], fh, depth);
    bko(&mut verts, &mut edges, &[(mo * 0.5, top), (mo * 0.5, age)], fh, depth);

    
    let fh = start_x + pitch;
    bko(&mut verts, &mut edges,
        &[(0.0, age), (0.0, top), (mo, top), (mo, mid), (0.0, mid)], fh, depth);
    bko(&mut verts, &mut edges,
        &[(0.04, mid), (mo, age)], fh, depth); 

    
    let fh = start_x + pitch * 2.0;
    bko(&mut verts, &mut edges,
        &[(0.0, top), (0.0, age), (mo, age), (mo, top)], fh, depth);

    
    let fh = start_x + pitch * 3.0;
    bko(&mut verts, &mut edges,
        &[(mo, top), (0.0, top), (0.0, mid), (mo, mid), (mo, age), (0.0, age)], fh, depth);

    
    let fh = start_x + pitch * 4.0;
    bko(&mut verts, &mut edges, &[(0.0, top), (mo, top)], fh, depth);
    bko(&mut verts, &mut edges, &[(mo * 0.5, top), (mo * 0.5, age)], fh, depth);

    
    let fh = start_x + pitch * 5.0;
    bko(&mut verts, &mut edges,
        &[(0.0, top), (mo, top), (mo, age), (0.0, age), (0.0, top)], fh, depth);

    
    let fh = start_x + pitch * 6.0;
    bko(&mut verts, &mut edges,
        &[(mo, top), (0.0, top), (0.0, mid), (mo, mid), (mo, age), (0.0, age)], fh, depth);

    Ai { vertices: verts, edges, edge_colors: None, faces: None, face_colors: None }
}




pub fn mesh_character() -> Ai {
    let mut verts = alloc::vec::Vec::with_capacity(120);
    let mut edges = alloc::vec::Vec::with_capacity(200);
    let mut colors = alloc::vec::Vec::with_capacity(200);

    
    let byf   = 0xFF88BBDD; 
    let eio   = 0xFFAADDFF; 
    let anv   = 0xFF7799BB; 
    let aao = 0xFFFFCC44; 
    let hmm   = 0xFF6688AA; 

    
    let mut avz = |verts: &mut alloc::vec::Vec<V3>, edges: &mut alloc::vec::Vec<(usize, usize)>,
                       colors: &mut alloc::vec::Vec<u32>,
                       cx: f32, u: f32, mj: f32, xc: f32, agm: f32, aqz: f32, color: u32| -> usize {
        let b = verts.len();
        
        verts.push(V3 { x: cx - xc, y: u - agm, z: mj - aqz }); 
        verts.push(V3 { x: cx + xc, y: u - agm, z: mj - aqz }); 
        verts.push(V3 { x: cx + xc, y: u - agm, z: mj + aqz }); 
        verts.push(V3 { x: cx - xc, y: u - agm, z: mj + aqz }); 
        verts.push(V3 { x: cx - xc, y: u + agm, z: mj - aqz }); 
        verts.push(V3 { x: cx + xc, y: u + agm, z: mj - aqz }); 
        verts.push(V3 { x: cx + xc, y: u + agm, z: mj + aqz }); 
        verts.push(V3 { x: cx - xc, y: u + agm, z: mj + aqz }); 
        
        edges.push((b, b+1)); edges.push((b+1, b+2)); edges.push((b+2, b+3)); edges.push((b+3, b));
        
        edges.push((b+4, b+5)); edges.push((b+5, b+6)); edges.push((b+6, b+7)); edges.push((b+7, b+4));
        
        edges.push((b, b+4)); edges.push((b+1, b+5)); edges.push((b+2, b+6)); edges.push((b+3, b+7));
        
        for _ in 0..12 { colors.push(color); }
        b
    };

    
    
    let bgw = verts.len(); 
    let ery = 0.10; let erx = 0.09; let erz = 0.58;
    verts.push(V3 { x: -ery, y: erz, z: -erx });
    verts.push(V3 { x:  ery, y: erz, z: -erx });
    verts.push(V3 { x:  ery, y: erz, z:  erx });
    verts.push(V3 { x: -ery, y: erz, z:  erx });
    
    let bmd = verts.len(); 
    let elz = 0.14; let ely = 0.12; let ema = 0.68;
    verts.push(V3 { x: -elz, y: ema, z: -ely });
    verts.push(V3 { x:  elz, y: ema, z: -ely });
    verts.push(V3 { x:  elz, y: ema, z:  ely });
    verts.push(V3 { x: -elz, y: ema, z:  ely });
    
    let bli = verts.len(); 
    let eje = 0.12; let ejd = 0.10; let cvx = 0.78;
    verts.push(V3 { x: -eje, y: cvx, z: -ejd });
    verts.push(V3 { x:  eje, y: cvx, z: -ejd });
    verts.push(V3 { x:  eje, y: cvx, z:  ejd });
    verts.push(V3 { x: -eje, y: cvx, z:  ejd });
    
    let ara = verts.len(); 
    verts.push(V3 { x: 0.0, y: 0.84, z: 0.0 });
    
    edges.push((bgw, bgw+1)); edges.push((bgw+1, bgw+2));
    edges.push((bgw+2, bgw+3)); edges.push((bgw+3, bgw));
    for _ in 0..4 { colors.push(eio); }
    
    edges.push((bmd, bmd+1)); edges.push((bmd+1, bmd+2));
    edges.push((bmd+2, bmd+3)); edges.push((bmd+3, bmd));
    for _ in 0..4 { colors.push(eio); }
    
    edges.push((bli, bli+1)); edges.push((bli+1, bli+2));
    edges.push((bli+2, bli+3)); edges.push((bli+3, bli));
    for _ in 0..4 { colors.push(aao); } 
    
    for i in 0..4 { edges.push((bgw + i, bmd + i)); colors.push(eio); }
    
    for i in 0..4 { edges.push((bmd + i, bli + i)); colors.push(eio); }
    
    for i in 0..4 { edges.push((bli + i, ara)); colors.push(aao); }

    
    let dbo = verts.len(); 
    verts.push(V3 { x: -0.06, y: 0.52, z: 0.0 }); 
    verts.push(V3 { x:  0.06, y: 0.52, z: 0.0 }); 
    edges.push((bgw, dbo)); colors.push(aao);
    edges.push((bgw+1, dbo+1)); colors.push(aao);
    edges.push((dbo, dbo+1)); colors.push(aao);

    
    
    let dw = verts.len(); 
    let fax = 0.22; let fas = 0.11; let fay = 0.48;
    verts.push(V3 { x: -fax, y: fay, z: -fas });
    verts.push(V3 { x:  fax, y: fay, z: -fas });
    verts.push(V3 { x:  fax, y: fay, z:  fas });
    verts.push(V3 { x: -fax, y: fay, z:  fas });
    
    let ch = verts.len(); 
    let ehz = 0.20; let ehu = 0.10; let eia = 0.30;
    verts.push(V3 { x: -ehz, y: eia, z: -ehu });
    verts.push(V3 { x:  ehz, y: eia, z: -ehu });
    verts.push(V3 { x:  ehz, y: eia, z:  ehu });
    verts.push(V3 { x: -ehz, y: eia, z:  ehu });
    
    let apn = verts.len(); 
    let ffa = 0.16; let feu = 0.09; let ffb = 0.08;
    verts.push(V3 { x: -ffa, y: ffb, z: -feu });
    verts.push(V3 { x:  ffa, y: ffb, z: -feu });
    verts.push(V3 { x:  ffa, y: ffb, z:  feu });
    verts.push(V3 { x: -ffa, y: ffb, z:  feu });
    
    let axl = verts.len(); 
    let epo = 0.17; let czg = 0.09; let epp = -0.04;
    verts.push(V3 { x: -epo, y: epp, z: -czg });
    verts.push(V3 { x:  epo, y: epp, z: -czg });
    verts.push(V3 { x:  epo, y: epp, z:  czg });
    verts.push(V3 { x: -epo, y: epp, z:  czg });
    
    edges.push((dbo, dw)); colors.push(byf);
    edges.push((dbo+1, dw+1)); colors.push(byf);
    
    edges.push((dw, dw+1)); edges.push((dw+1, dw+2)); edges.push((dw+2, dw+3)); edges.push((dw+3, dw));
    for _ in 0..4 { colors.push(byf); }
    
    edges.push((ch, ch+1)); edges.push((ch+1, ch+2)); edges.push((ch+2, ch+3)); edges.push((ch+3, ch));
    for _ in 0..4 { colors.push(byf); }
    
    edges.push((apn, apn+1)); edges.push((apn+1, apn+2)); edges.push((apn+2, apn+3)); edges.push((apn+3, apn));
    for _ in 0..4 { colors.push(aao); } 
    
    edges.push((axl, axl+1)); edges.push((axl+1, axl+2)); edges.push((axl+2, axl+3)); edges.push((axl+3, axl));
    for _ in 0..4 { colors.push(byf); }
    
    for i in 0..4 { edges.push((dw + i, ch + i)); colors.push(byf); }
    for i in 0..4 { edges.push((ch + i, apn + i)); colors.push(byf); }
    for i in 0..4 { edges.push((apn + i, axl + i)); colors.push(byf); }

    
    let etk = avz(&mut verts, &mut edges, &mut colors,
        -0.30, 0.38, 0.0,   0.06, 0.10, 0.06,  anv); 
    let esy = avz(&mut verts, &mut edges, &mut colors,
        -0.32, 0.16, 0.0,   0.05, 0.10, 0.05,  anv); 
    
    edges.push((dw, etk + 4)); colors.push(aao); 
    edges.push((dw, etk + 7)); colors.push(aao);
    
    edges.push((etk, esy + 4)); colors.push(aao);
    edges.push((etk + 3, esy + 7)); colors.push(aao);
    
    let ikc = avz(&mut verts, &mut edges, &mut colors,
        -0.33, 0.02, 0.0,   0.04, 0.04, 0.04,  aao);
    edges.push((esy, ikc + 4)); colors.push(anv);
    edges.push((esy + 1, ikc + 5)); colors.push(anv);

    
    let eze = avz(&mut verts, &mut edges, &mut colors,
        0.30, 0.38, 0.0,   0.06, 0.10, 0.06,  anv);
    let eyq = avz(&mut verts, &mut edges, &mut colors,
        0.32, 0.16, 0.0,   0.05, 0.10, 0.05,  anv);
    edges.push((dw + 1, eze + 5)); colors.push(aao);
    edges.push((dw + 1, eze + 6)); colors.push(aao);
    edges.push((eze + 1, eyq + 5)); colors.push(aao);
    edges.push((eze + 2, eyq + 6)); colors.push(aao);
    let jaq = avz(&mut verts, &mut edges, &mut colors,
        0.33, 0.02, 0.0,   0.04, 0.04, 0.04,  aao);
    edges.push((eyq, jaq + 4)); colors.push(anv);
    edges.push((eyq + 1, jaq + 5)); colors.push(anv);

    
    let etj = avz(&mut verts, &mut edges, &mut colors,
        -0.09, -0.18, 0.0,   0.06, 0.12, 0.06,  anv);
    let eti = avz(&mut verts, &mut edges, &mut colors,
        -0.09, -0.40, 0.0,   0.05, 0.10, 0.05,  anv);
    
    edges.push((axl, etj + 4)); colors.push(aao);
    edges.push((axl + 3, etj + 7)); colors.push(aao);
    
    edges.push((etj, eti + 4)); colors.push(aao);
    edges.push((etj + 3, eti + 7)); colors.push(aao);
    
    let ikb = avz(&mut verts, &mut edges, &mut colors,
        -0.09, -0.53, -0.02,   0.06, 0.03, 0.08,  hmm);
    edges.push((eti, ikb + 4)); colors.push(anv);
    edges.push((eti + 1, ikb + 5)); colors.push(anv);

    
    let ezd = avz(&mut verts, &mut edges, &mut colors,
        0.09, -0.18, 0.0,   0.06, 0.12, 0.06,  anv);
    let ezc = avz(&mut verts, &mut edges, &mut colors,
        0.09, -0.40, 0.0,   0.05, 0.10, 0.05,  anv);
    edges.push((axl + 1, ezd + 5)); colors.push(aao);
    edges.push((axl + 2, ezd + 6)); colors.push(aao);
    edges.push((ezd + 1, ezc + 5)); colors.push(aao);
    edges.push((ezd + 2, ezc + 6)); colors.push(aao);
    let jap = avz(&mut verts, &mut edges, &mut colors,
        0.09, -0.53, -0.02,   0.06, 0.03, 0.08,  hmm);
    edges.push((ezc, jap + 4)); colors.push(anv);
    edges.push((ezc + 1, jap + 5)); colors.push(anv);

    Ai { vertices: verts, edges, edge_colors: Some(colors), faces: None, face_colors: None }
}

pub fn mesh_helix(radius: f32, height: f32, bac: f32, gq: usize) -> Ai {
    let mut verts = alloc::vec::Vec::new();
    let mut edges = alloc::vec::Vec::new();
    let plk = bac * 6.2831853;
    
    for strand in 0..2u32 {
        let offset = strand as f32 * 3.14159265;
        let base = verts.len();
        for i in 0..gq {
            let t = i as f32 / (gq - 1) as f32;
            let cc = t * plk + offset;
            let x = radius * hr(cc);
            let z = radius * eu(cc);
            let y = -height * 0.5 + t * height;
            verts.push(V3 { x, y, z });
            if i > 0 { edges.push((base + i - 1, base + i)); }
        }
    }
    
    let cw = gq;
    for i in (0..gq).step_by(gq / 8) {
        edges.push((i, i + cw));
    }
    Ai { vertices: verts, edges, edge_colors: None, faces: None, face_colors: None }
}




#[inline(always)]
fn cross(a: V3, b: V3) -> V3 {
    V3 {
        x: a.y * b.z - a.z * b.y,
        y: a.z * b.x - a.x * b.z,
        z: a.x * b.y - a.y * b.x,
    }
}


#[inline(always)]
fn dot(a: V3, b: V3) -> f32 {
    a.x * b.x + a.y * b.y + a.z * b.z
}


#[inline(always)]
fn sub(a: V3, b: V3) -> V3 {
    V3 { x: a.x - b.x, y: a.y - b.y, z: a.z - b.z }
}


#[inline(always)]
fn normalize(v: V3) -> V3 {
    let len = ra(v.x * v.x + v.y * v.y + v.z * v.z);
    if len < 0.0001 { return V3 { x: 0.0, y: 0.0, z: 1.0 }; }
    let ki = 1.0 / len;
    V3 { x: v.x * ki, y: v.y * ki, z: v.z * ki }
}




#[inline(never)]
fn fwt(buf: &mut [u32], w: usize, h: usize,
                 mut bm: i32, mut az: i32,
                 mut x1: i32, mut y1: i32,
                 mut x2: i32, mut y2: i32,
                 color: u32) {
    
    if az > y1 { core::mem::swap(&mut bm, &mut x1); core::mem::swap(&mut az, &mut y1); }
    if y1 > y2 { core::mem::swap(&mut x1, &mut x2); core::mem::swap(&mut y1, &mut y2); }
    if az > y1 { core::mem::swap(&mut bm, &mut x1); core::mem::swap(&mut az, &mut y1); }

    let sn = y2 - az;
    if sn == 0 { return; }

    
    let ajb = az.max(0);
    let bkg = y2.min(h as i32 - 1);

    for y in ajb..=bkg {
        let dyv = y >= y1;
        let cqg = if dyv { y2 - y1 } else { y1 - az };

        
        let gxu = (y - az) as f32 / sn as f32;
        let aja = bm as f32 + (x2 - bm) as f32 * gxu; 

        let bkd = if cqg == 0 {
            aja 
        } else if dyv {
            let crg = (y - y1) as f32 / cqg as f32;
            x1 as f32 + (x2 - x1) as f32 * crg
        } else {
            let crg = (y - az) as f32 / cqg as f32;
            bm as f32 + (x1 - bm) as f32 * crg
        };

        let mut left = aja as i32;
        let mut right = bkd as i32;
        if left > right { core::mem::swap(&mut left, &mut right); }

        
        left = left.max(0);
        right = right.min(w as i32 - 1);

        let row = y as usize * w;
        for x in left..=right {
            let idx = row + x as usize;
            if idx < buf.len() {
                buf[idx] = color;
            }
        }
    }
}



pub fn ofv(buf: &mut [u32], w: usize, h: usize,
                             mesh: &Ai, angle_y: f32, angle_x: f32, dz: f32,
                             color: u32) {
    for (idx, &(a, b)) in mesh.edges.iter().enumerate() {
        if a >= mesh.vertices.len() || b >= mesh.vertices.len() { continue; }
        let (bm, az, z0) = avl(mesh.vertices[a], angle_y, angle_x, dz, w, h);
        let (x1, y1, po) = avl(mesh.vertices[b], angle_y, angle_x, dz, w, h);
        let avg_z = (z0 + po) * 0.5;
        let base = match &mesh.edge_colors {
            Some(ec) if idx < ec.len() => ec[idx],
            _ => color,
        };
        let c = frp(avg_z, base);
        draw_line_thick(buf, w, h, bm, az, x1, y1, c);
    }
    
    for v in &mesh.vertices {
        let (am, ak, _z) = avl(*v, angle_y, angle_x, dz, w, h);
        for ad in -1..=1i32 {
            for dx in -1..=1i32 {
                efa(buf, w, h, am + dx, ak + ad, 0x00FFFFFF);
            }
        }
    }
}




pub fn grb(buf: &mut [u32], w: usize, h: usize,
                          mesh: &Ai, angle_y: f32, angle_x: f32, dz: f32,
                          qf: u32, axv: V3, ambient: f32) {
    let faces = match &mesh.faces {
        Some(f) => f,
        None => return,
    };

    
    let mut bpr = alloc::vec::Vec::with_capacity(mesh.vertices.len());
    let mut agy = alloc::vec::Vec::with_capacity(mesh.vertices.len());
    for v in &mesh.vertices {
        let auu = rotate_x(rotate_y(*v, angle_y), angle_x);
        let ceq = joo(auu, dz);
        let olr = gzd(project(ceq), w, h);
        bpr.push(ceq);
        agy.push(olr);
    }

    
    
    struct Aka {
        idx: usize,
        avg_z: f32,
        brightness: f32,
    }
    let mut dgh = alloc::vec::Vec::with_capacity(faces.len());

    for (i, &(a, b, c)) in faces.iter().enumerate() {
        if a >= bpr.len() || b >= bpr.len() || c >= bpr.len() { continue; }

        let va = bpr[a];
        let apk = bpr[b];
        let bad = bpr[c];

        
        let lnx = sub(apk, va);
        let lny = sub(bad, va);
        let normal = normalize(cross(lnx, lny));

        
        
        
        if normal.z > 0.0 { continue; }

        
        let dux = dot(normal, axv);
        let brightness = ambient + (1.0 - ambient) * dux.max(0.0);

        let avg_z = (va.z + apk.z + bad.z) / 3.0;
        dgh.push(Aka { idx: i, avg_z, brightness });
    }

    
    
    for i in 1..dgh.len() {
        let mut ay = i;
        while ay > 0 && dgh[ay].avg_z > dgh[ay - 1].avg_z {
            dgh.swap(ay, ay - 1);
            ay -= 1;
        }
    }

    
    let adi = (qf >> 16) & 0xFF;
    let agd = (qf >> 8) & 0xFF;
    let apu = qf & 0xFF;

    for face in &dgh {
        let (a, b, c) = faces[face.idx];
        let (sx0, sy0) = agy[a];
        let (wn, aiu) = agy[b];
        let (tq, acv) = agy[c];

        
        let (ko, fg, fb) = match &mesh.face_colors {
            Some(br) if face.idx < br.len() => {
                ((br[face.idx] >> 16) & 0xFF,
                 (br[face.idx] >> 8) & 0xFF,
                 br[face.idx] & 0xFF)
            }
            _ => (adi, agd, apu),
        };

        let r = ((ko as f32 * face.brightness) as u32).min(255);
        let g = ((fg as f32 * face.brightness) as u32).min(255);
        let b = ((fb as f32 * face.brightness) as u32).min(255);
        let gur = 0xFF000000 | (r << 16) | (g << 8) | b;

        fwt(buf, w, h, sx0, sy0, wn, aiu, tq, acv, gur);
    }

    
    for &(a, b) in &mesh.edges {
        if a >= agy.len() || b >= agy.len() { continue; }
        let (bm, az) = agy[a];
        let (x1, y1) = agy[b];
        
        let awx = 0xFF000000 | ((adi / 3) << 16) | ((agd / 3) << 8) | (apu / 3);
        draw_line(buf, w, h, bm, az, x1, y1, awx);
    }
}




#[inline(always)]
fn efa(buf: &mut [u32], w: usize, h: usize, x: i32, y: i32, color: u32) {
    if x < 0 || y < 0 || x >= w as i32 || y >= h as i32 { return; }
    let idx = y as usize * w + x as usize;
    if idx >= buf.len() { return; }
    let dst = buf[idx];
    let r = ((dst >> 16) & 0xFF) + ((color >> 16) & 0xFF);
    let g = ((dst >> 8) & 0xFF) + ((color >> 8) & 0xFF);
    let b = (dst & 0xFF) + (color & 0xFF);
    buf[idx] = 0xFF000000 | (r.min(255) << 16) | (g.min(255) << 8) | b.min(255);
}


fn draw_line(buf: &mut [u32], w: usize, h: usize, bm: i32, az: i32, x1: i32, y1: i32, color: u32) {
    let mut bm = bm; let mut az = az;
    let dx = (x1 - bm).abs();
    let ad = -(y1 - az).abs();
    let am: i32 = if bm < x1 { 1 } else { -1 };
    let ak: i32 = if az < y1 { 1 } else { -1 };
    let mut err = dx + ad;
    
    loop {
        efa(buf, w, h, bm, az, color);
        if bm == x1 && az == y1 { break; }
        let pg = 2 * err;
        if pg >= ad { err += ad; bm += am; }
        if pg <= dx { err += dx; az += ak; }
    }
}


fn draw_line_thick(buf: &mut [u32], w: usize, h: usize, bm: i32, az: i32, x1: i32, y1: i32, color: u32) {
    draw_line(buf, w, h, bm, az, x1, y1, color);
    draw_line(buf, w, h, bm + 1, az, x1 + 1, y1, color);
    draw_line(buf, w, h, bm, az + 1, x1, y1 + 1, color);
}


fn dim(buf: &mut [u32], ha: u8) {
    for p in buf.iter_mut() {
        let r = ((*p >> 16) & 0xFF) * ha as u32 / 256;
        let g = ((*p >> 8) & 0xFF) * ha as u32 / 256;
        let b = (*p & 0xFF) * ha as u32 / 256;
        *p = 0xFF000000 | (r << 16) | (g << 8) | b;
    }
}


fn frp(z: f32, qf: u32) -> u32 {
    let intensity = ((1.0 / (z * 0.5 + 1.0)) * 255.0) as u32;
    let intensity = intensity.min(255);
    let r = ((qf >> 16) & 0xFF) * intensity / 255;
    let g = ((qf >> 8) & 0xFF) * intensity / 255;
    let b = (qf & 0xFF) * intensity / 255;
    0xFF000000 | (r << 16) | (g << 8) | b
}



struct Aol {
    y: f32,
    speed: f32,
    len: u32,
    du: u8,
}


struct Alc {
    x: f32,
    z: f32,
    speed: f32,
    trail_len: u8,
    phase: f32,
}



#[derive(Clone, Copy, PartialEq)]
pub enum FormulaScene {
    Cube,
    Pyramid,
    Diamond,
    Torus,
    Icosphere,
    Grid,
    Helix,
    Multi,
    Penger,
    TrustOs,
    HoloMatrix,
    Character,
}



pub struct FormulaRenderer {
    pub scene: FormulaScene,
    pub angle_y: f32,
    pub angle_x: f32,
    pub dz: f32,
    pub wire_color: u32,
    frame: u32,
    rain_columns: alloc::vec::Vec<Aol>,
    rain_inited: bool,
    
    holo_rain_inited: bool,
    holo_columns: alloc::vec::Vec<Alc>,
    
    mesh_cube: Option<Ai>,
    mesh_pyramid: Option<Ai>,
    mesh_diamond: Option<Ai>,
    mesh_torus: Option<Ai>,
    mesh_icosphere: Option<Ai>,
    mesh_grid: Option<Ai>,
    mesh_helix: Option<Ai>,
    mesh_penger: Option<Ai>,
    mesh_trustos: Option<Ai>,
    mesh_character: Option<Ai>,
}

impl FormulaRenderer {
    pub fn new() -> Self {
        Self {
            scene: FormulaScene::HoloMatrix,
            angle_y: 0.0,
            angle_x: 0.3,
            dz: 2.0,
            wire_color: 0xFF00FF66,
            frame: 0,
            rain_columns: alloc::vec::Vec::new(),
            rain_inited: false,
            holo_rain_inited: false,
            holo_columns: alloc::vec::Vec::new(),
            mesh_cube: None,
            mesh_pyramid: None,
            mesh_diamond: None,
            mesh_torus: None,
            mesh_icosphere: None,
            mesh_grid: None,
            mesh_helix: None,
            mesh_penger: None,
            mesh_trustos: None,
            mesh_character: None,
        }
    }

    pub fn set_scene(&mut self, scene: FormulaScene) {
        self.scene = scene;
        
        self.ensure_mesh();
    }

    fn ensure_mesh(&mut self) {
        match self.scene {
            FormulaScene::Cube => { if self.mesh_cube.is_none() { self.mesh_cube = Some(mesh_cube()); } }
            FormulaScene::Pyramid => { if self.mesh_pyramid.is_none() { self.mesh_pyramid = Some(mesh_pyramid()); } }
            FormulaScene::Diamond => { if self.mesh_diamond.is_none() { self.mesh_diamond = Some(mesh_diamond()); } }
            FormulaScene::Torus => { if self.mesh_torus.is_none() { self.mesh_torus = Some(mesh_torus(0.5, 0.2, 16, 12)); } }
            FormulaScene::Icosphere => { if self.mesh_icosphere.is_none() { self.mesh_icosphere = Some(mesh_icosphere(0.6)); } }
            FormulaScene::Grid => { if self.mesh_grid.is_none() { self.mesh_grid = Some(mesh_grid(1.5, 10)); } }
            FormulaScene::Helix => { if self.mesh_helix.is_none() { self.mesh_helix = Some(mesh_helix(0.4, 1.2, 3.0, 60)); } }
            FormulaScene::Penger => { if self.mesh_penger.is_none() { self.mesh_penger = Some(mesh_penger()); } }
            FormulaScene::TrustOs => { if self.mesh_trustos.is_none() { self.mesh_trustos = Some(inm()); } }
            FormulaScene::Character => { if self.mesh_character.is_none() { self.mesh_character = Some(mesh_character()); } }
            FormulaScene::HoloMatrix => {  }
            FormulaScene::Multi => {
                if self.mesh_cube.is_none() { self.mesh_cube = Some(mesh_cube()); }
                if self.mesh_pyramid.is_none() { self.mesh_pyramid = Some(mesh_pyramid()); }
                if self.mesh_diamond.is_none() { self.mesh_diamond = Some(mesh_diamond()); }
                if self.mesh_torus.is_none() { self.mesh_torus = Some(mesh_torus(0.5, 0.2, 16, 12)); }
            }
        }
    }

    fn get_mesh(&self) -> Option<&Ai> {
        match self.scene {
            FormulaScene::Cube => self.mesh_cube.as_ref(),
            FormulaScene::Pyramid => self.mesh_pyramid.as_ref(),
            FormulaScene::Diamond => self.mesh_diamond.as_ref(),
            FormulaScene::Torus => self.mesh_torus.as_ref(),
            FormulaScene::Icosphere => self.mesh_icosphere.as_ref(),
            FormulaScene::Grid => self.mesh_grid.as_ref(),
            FormulaScene::Helix => self.mesh_helix.as_ref(),
            FormulaScene::Penger => self.mesh_penger.as_ref(),
            FormulaScene::TrustOs => self.mesh_trustos.as_ref(),
            FormulaScene::Character => self.mesh_character.as_ref(),
            FormulaScene::HoloMatrix => None, 
            FormulaScene::Multi => None, 
        }
    }

    
    pub fn update(&mut self) {
        self.frame = self.frame.wrapping_add(1);
        match self.scene {
            FormulaScene::TrustOs => {
                self.angle_y += 0.012;
                self.angle_x = 0.15 + eu(self.frame as f32 * 0.006) * 0.1;
                self.dz = 3.2 + eu(self.frame as f32 * 0.005) * 0.2;
            }
            FormulaScene::Character => {
                
                self.angle_y += 0.018;
                self.angle_x = 0.20 + eu(self.frame as f32 * 0.007) * 0.12;
                self.dz = 2.2 + eu(self.frame as f32 * 0.004) * 0.15;
            }
            FormulaScene::HoloMatrix => {
                
                
                self.angle_y = eu(self.frame as f32 * 0.0008) * 0.015;
                self.angle_x = eu(self.frame as f32 * 0.0006) * 0.01;
                self.dz = 0.0;
            }
            _ => {
                self.angle_y += 0.025;
                self.angle_x = 0.3 + eu(self.frame as f32 * 0.008) * 0.2;
                self.dz = 2.0 + eu(self.frame as f32 * 0.005) * 0.3;
            }
        }
    }

    
    pub fn render(&self, buf: &mut [u32], w: usize, h: usize) {
        
        #[cfg(target_arch = "x86_64")]
        unsafe {
            use core::arch::x86_64::*;
            let awh = _mm_set1_epi32(0xFF000000u32 as i32);
            let ptr = buf.as_mut_ptr() as *mut __m128i;
            let count = buf.len() / 4;
            for i in 0..count {
                _mm_storeu_si128(ptr.add(i), awh);
            }
            
            for i in (count * 4)..buf.len() {
                buf[i] = 0xFF000000;
            }
        }
        #[cfg(not(target_arch = "x86_64"))]
        buf.fill(0xFF000000);

        if self.scene == FormulaScene::HoloMatrix {
            
            self.render_holo_matrix(buf, w, h);
            return;
        }

        if self.scene == FormulaScene::Character {
            
            self.render_character_bg(buf, w, h);
        } else {
            
            self.render_rain(buf, w, h);
        }
        
        
        if self.scene == FormulaScene::Multi {
            self.render_multi(buf, w, h);
        } else if let Some(mesh) = self.get_mesh() {
            self.render_wireframe(buf, w, h, mesh, self.angle_y, self.angle_x, self.dz, self.wire_color);
            self.render_vertices(buf, w, h, mesh, self.angle_y, self.angle_x, self.dz);
        }

        
        if self.scene == FormulaScene::TrustOs || self.scene == FormulaScene::Character {
            self.render_scanlines(buf, w, h);
        }
    }

    fn render_wireframe(&self, buf: &mut [u32], w: usize, h: usize, mesh: &Ai,
                        aet: f32, ax: f32, dz: f32, color: u32) {
        for (idx, &(a, b)) in mesh.edges.iter().enumerate() {
            if a >= mesh.vertices.len() || b >= mesh.vertices.len() { continue; }
            let (bm, az, z0) = avl(mesh.vertices[a], aet, ax, dz, w, h);
            let (x1, y1, po) = avl(mesh.vertices[b], aet, ax, dz, w, h);
            let avg_z = (z0 + po) * 0.5;
            
            let qf = match &mesh.edge_colors {
                Some(ec) if idx < ec.len() => ec[idx],
                _ => color,
            };
            let c = frp(avg_z, qf);
            draw_line_thick(buf, w, h, bm, az, x1, y1, c);
        }
    }

    fn render_vertices(&self, buf: &mut [u32], w: usize, h: usize, mesh: &Ai,
                       aet: f32, ax: f32, dz: f32) {
        for v in &mesh.vertices {
            let (am, ak, _z) = avl(*v, aet, ax, dz, w, h);
            
            for ad in -1..=1i32 {
                for dx in -1..=1i32 {
                    efa(buf, w, h, am + dx, ak + ad, 0x00FFFFFF);
                }
            }
        }
    }

    fn render_multi(&self, buf: &mut [u32], w: usize, h: usize) {
        let t = self.frame as f32 * 0.012;
        let isp = 1.2;
        
        
        let shapes: [(f32, u32, FormulaScene); 4] = [
            (0.0, 0xFF00FF66, FormulaScene::Cube),
            (1.5707963, 0xFF6666FF, FormulaScene::Pyramid),
            (3.14159265, 0xFFFF6600, FormulaScene::Diamond),
            (4.7123889, 0xFFFF00FF, FormulaScene::Torus),
        ];
        
        for (angle_offset, color, scene) in &shapes {
            let isn = t + angle_offset;
            let fh = isp * hr(isn);
            let evy = isp * eu(isn);
            
            let mesh = match scene {
                FormulaScene::Cube => self.mesh_cube.as_ref(),
                FormulaScene::Pyramid => self.mesh_pyramid.as_ref(),
                FormulaScene::Diamond => self.mesh_diamond.as_ref(),
                FormulaScene::Torus => self.mesh_torus.as_ref(),
                _ => None,
            };
            
            if let Some(mesh) = mesh {
                
                for &(a, b) in &mesh.edges {
                    if a >= mesh.vertices.len() || b >= mesh.vertices.len() { continue; }
                    let va = V3 { x: mesh.vertices[a].x * 0.35 + fh, y: mesh.vertices[a].y * 0.35, z: mesh.vertices[a].z * 0.35 + evy };
                    let apk = V3 { x: mesh.vertices[b].x * 0.35 + fh, y: mesh.vertices[b].y * 0.35, z: mesh.vertices[b].z * 0.35 + evy };
                    let (bm, az, z0) = avl(va, self.angle_y * 0.5, self.angle_x, self.dz + 1.0, w, h);
                    let (x1, y1, po) = avl(apk, self.angle_y * 0.5, self.angle_x, self.dz + 1.0, w, h);
                    let avg_z = (z0 + po) * 0.5;
                    let c = frp(avg_z, *color);
                    draw_line_thick(buf, w, h, bm, az, x1, y1, c);
                }
            }
        }
    }

    
    
    
    
    
    
    
    
    
    
    
    
    fn render_holo_matrix(&self, buf: &mut [u32], w: usize, h: usize) {
        
        const Gm: [u64; 16] = [
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

        
        
        
        const BDW_: usize = 10;        
        const TE_: usize = 18;    
        const CLK_: usize = BDW_ * TE_; 
        const RP_: f32 = 16.0;         
        const BLL_: f32 = 0.5;           
        const DCR_: f32 = 5.5;      

        let frame = self.frame;
        
        let khe = frame as f32 * 0.018; 

        let khb = self.angle_y; 
        let kha = self.angle_x;

        let cx = (w as f32) * 0.5;
        let u = (h as f32) * 0.5;

        
        {
            let dbc = 40i32;
            let kq = 0.85 + 0.15 * eu(frame as f32 * 0.03);
            let ion = dbc * dbc;
            for cm in -dbc..=dbc {
                let o = u as i32 + cm;
                if o < 0 || o >= h as i32 { continue; }
                let apa = cm * cm;
                for da in -dbc..=dbc {
                    let jq = da * da + apa;
                    if jq > ion { continue; }
                    let amk = cx as i32 + da;
                    if amk < 0 || amk >= w as i32 { continue; }
                    let t = 1.0 - (jq as f32 / ion as f32);
                    let t = t * t * kq;
                    let g = (t * 45.0) as u32;
                    let b = (t * 15.0) as u32;
                    if g > 0 {
                        let glow = 0xFF000000 | (g.min(255) << 8) | b.min(255);
                        let idx = o as usize * w + amk as usize;
                        if idx < buf.len() {
                            let dst = buf[idx];
                            let cpr = ((dst >> 16) & 0xFF) + ((glow >> 16) & 0xFF);
                            let bgq = ((dst >> 8) & 0xFF) + ((glow >> 8) & 0xFF);
                            let mq = (dst & 0xFF) + (glow & 0xFF);
                            buf[idx] = 0xFF000000 | (cpr.min(255) << 16) | (bgq.min(255) << 8) | mq.min(255);
                        }
                    }
                }
            }
        }

        
        for ow in 0..CLK_ {
            let dq = ow / TE_;
            let slot = ow % TE_;

            
            let seed = (ow as u32).wrapping_mul(2654435761).wrapping_add(374761393);
            let dyw = seed.wrapping_mul(1664525).wrapping_add(1013904223);
            let ona = dyw.wrapping_mul(1664525).wrapping_add(1013904223);

            
            let egi = (slot as f32 / TE_ as f32) * 6.2831853;
            let jwc = (seed % 1000) as f32 / 1000.0 * 0.3 - 0.15;
            let acz = egi + jwc;

            
            let ohi = (dq as f32 + 0.3) / BDW_ as f32;
            let radius = 0.3 + ohi * DCR_;

            
            let chm = radius * hr(acz);
            let kvl = radius * eu(acz);

            
            let kvo = (dyw % 10000) as f32 / 10000.0 * RP_;
            
            let kvp = kvo - (khe % RP_);
            let kvn = ((kvp % RP_) + RP_) % RP_ + BLL_;

            
            let speed = 0.012 + (ona % 1000) as f32 / 1000.0 * 0.02;
            let trail_len = 6 + (seed >> 12) as usize % 12;
            let hkn: f32 = 0.30;
            let phase = (dyw >> 8) as f32 / 256.0;
            let ctb: f32 = 3.0 + radius * 0.5;

            
            let eck = ctb + trail_len as f32 * hkn + 2.0;
            let mkn = ctb * 0.5 - ((frame as f32 * speed + phase * ctb) % eck);

            for ci in 0..trail_len {
                let ta = mkn + ci as f32 * hkn;
                if ta > ctb || ta < -ctb { continue; }

                
                let vy = kvl + ta;
                let v = V3 { x: chm, y: vy, z: kvn };

                
                let auu = rotate_x(rotate_y(v, khb), kha);
                if auu.z < BLL_ { continue; }

                let aa = project(auu);
                let (am, ak) = gzd(aa, w, h);

                
                let ctd = 1.0 / auu.z;
                let scale = (ctd * ctd * 30.0).max(1.0).min(4.0) as i32;

                let bbx = scale * 5;
                let bbw = scale * 7;
                if am + bbx < -5 || am - bbx > w as i32 + 5 { continue; }
                if ak + bbw < -5 || ak - bbw > h as i32 + 5 { continue; }

                
                let jol = ci as f32 / trail_len as f32;
                let cep = (1.0 - jol) * (1.0 - jol);
                
                let dmt = bbo(-auu.z * 0.12);
                let brightness = cep * dmt;

                if brightness < 0.015 { continue; }

                
                let axi = (seed.wrapping_mul(ci as u32 + 1).wrapping_add(frame / 5)) as usize % 16;
                let du = Gm[axi];

                
                let (alg, ahp, cb) = if ci == 0 {
                    
                    ((brightness * 200.0) as u32, (brightness * 255.0) as u32, (brightness * 210.0) as u32)
                } else if ci <= 2 {
                    ((brightness * 50.0) as u32, (brightness * 255.0) as u32, (brightness * 70.0) as u32)
                } else {
                    ((brightness * 8.0) as u32, (brightness * 230.0) as u32, (brightness * 20.0) as u32)
                };
                let alg = alg.min(255);
                let ahp = ahp.min(255);
                let cb = cb.min(255);
                let color = 0xFF000000 | (alg << 16) | (ahp << 8) | cb;

                
                let fh = am - bbx / 2;
                let hk = ak - bbw / 2;
                let pqj = ci <= 1; 
                for row in 0..7i32 {
                    for col in 0..5i32 {
                        let bew = row * 5 + col;
                        if (du >> bew) & 1 == 0 { continue; }
                        for o in 0..scale {
                            let hj = hk + row * scale + o;
                            if hj < 0 || hj >= h as i32 { continue; }
                            let cpq = hj as usize * w;
                            for p in 0..scale {
                                let dg = fh + col * scale + p;
                                if dg < 0 || dg >= w as i32 { continue; }
                                let idx = cpq + dg as usize;
                                if pqj {
                                    let dst = buf[idx];
                                    let cpr = ((dst >> 16) & 0xFF) + ((color >> 16) & 0xFF);
                                    let bgq = ((dst >> 8) & 0xFF) + ((color >> 8) & 0xFF);
                                    let mq = (dst & 0xFF) + (color & 0xFF);
                                    buf[idx] = 0xFF000000 | (cpr.min(255) << 16) | (bgq.min(255) << 8) | mq.min(255);
                                } else {
                                    buf[idx] = color;
                                }
                            }
                        }
                    }
                }

                
                if ci == 0 && ahp > 80 && scale >= 2 {
                    let fze = ahp / 4;
                    let aog = 0xFF000000 | (fze << 8);
                    let kya: [(i32,i32); 4] = [
                        (fh - 1, hk - 1), (fh + bbx, hk - 1),
                        (fh - 1, hk + bbw), (fh + bbx, hk + bbw),
                    ];
                    for &(hc, jh) in &kya {
                        if hc >= 0 && hc < w as i32 && jh >= 0 && jh < h as i32 {
                            efa(buf, w, h, hc, jh, aog);
                        }
                    }
                }
            }
        }

        
        
        {
            let icv: u32 = 0x14; 
            let epl = u as i32 + 30; 
            let gkb = 12; 
            let irw = 16; 
            let mgc = (frame as f32 * 0.35) % 48.0; 

            
            for i in 0..gkb {
                let t = (i as f32 + mgc / 48.0 * (gkb as f32 / 4.0)) / gkb as f32;
                let t = t.min(0.99);
                let nn = epl + ((1.0 - t) * (1.0 - t) * (h as f32 - epl as f32)) as i32;
                if nn < 0 || nn >= h as i32 { continue; }
                let dmt = (1.0 - t) * (1.0 - t);
                let g = (icv as f32 * dmt * 1.5) as u32;
                if g < 3 { continue; }
                let btx = 0xFF000000 | (g.min(255) << 8) | (g.min(255) / 4);
                let cpq = nn as usize * w;
                
                let mut p = 0;
                while p < w {
                    let idx = cpq + p;
                    if idx < buf.len() {
                        let dst = buf[idx];
                        let bgq = ((dst >> 8) & 0xFF) + ((btx >> 8) & 0xFF);
                        let mq = (dst & 0xFF) + (btx & 0xFF);
                        buf[idx] = 0xFF000000 | (bgq.min(255) << 8) | mq.min(255);
                    }
                    p += 2; 
                }
            }

            
            for i in 0..irw {
                let vx = (i as f32 / irw as f32) * w as f32;
                let vx = vx as i32;
                let bottom_y = h as i32 - 1;
                let steps = (bottom_y - epl).max(1);
                let mut j = 0;
                while j < steps {
                    let t = j as f32 / steps as f32;
                    let ak = epl + j;
                    if ak >= h as i32 { break; }
                    if ak < 0 { j += 4; continue; }
                    let am = cx as i32 + ((vx - cx as i32) as f32 * t * t) as i32;
                    if am >= 0 && am < w as i32 {
                        let dmt = t * t;
                        let g = (icv as f32 * dmt * 1.2) as u32;
                        if g >= 2 {
                            let idx = ak as usize * w + am as usize;
                            if idx < buf.len() {
                                let btx = 0xFF000000 | (g.min(255) << 8) | (g.min(255) / 4);
                                let dst = buf[idx];
                                let bgq = ((dst >> 8) & 0xFF) + ((btx >> 8) & 0xFF);
                                let mq = (dst & 0xFF) + (btx & 0xFF);
                                buf[idx] = 0xFF000000 | (bgq.min(255) << 8) | mq.min(255);
                            }
                        }
                    }
                    j += 3; 
                }
            }
        }

        
        
        {
            let iru = 30;
            for i in 0..iru {
                let seed = (i as u32).wrapping_mul(2654435761).wrapping_add(frame.wrapping_mul(7));
                let dyw = seed.wrapping_mul(1664525).wrapping_add(1013904223);
                let cc = (i as f32 / iru as f32) * 6.2831853 + (seed % 1000) as f32 * 0.001;
                
                let jzz = 80.0 + (dyw % 400) as f32;
                let life = ((frame.wrapping_mul(3).wrapping_add(seed)) % 120) as f32 / 120.0;
                let em = jzz + life * 400.0;
                let len = 3.0 + life * 15.0;

                let vg = hr(cc);
                let vt = eu(cc);
                let bm = cx as i32 + (em * vg) as i32;
                let az = u as i32 + (em * vt) as i32;
                let x1 = cx as i32 + ((em + len) * vg) as i32;
                let y1 = u as i32 + ((em + len) * vt) as i32;

                let ln = ((1.0 - life) * 40.0) as u32;
                if ln < 2 { continue; }
                let oxx = 0xFF000000 | (ln.min(255) << 8) | (ln.min(255) / 3);
                draw_line(buf, w, h, bm, az, x1, y1, oxx);
            }
        }

    }

    
    fn render_character_bg(&self, buf: &mut [u32], w: usize, h: usize) {
        let cx = w as f32 * 0.5;
        let u = h as f32 * 0.5;
        let ggv = ra(cx * cx + u * u);

        for y in 0..h {
            for x in 0..w {
                let idx = y * w + x;
                if idx >= buf.len() { break; }
                
                let t = y as f32 / h as f32;
                let adi = (8.0 + t * 12.0) as u32;
                let agd = (12.0 + t * 20.0) as u32;
                let apu = (30.0 + t * 35.0) as u32;
                
                let dx = x as f32 - cx;
                let ad = y as f32 - u;
                let em = ra(dx * dx + ad * ad) / ggv;
                let csk = 1.0 - em * em * 0.6;
                let r = ((adi as f32 * csk) as u32).min(255);
                let g = ((agd as f32 * csk) as u32).min(255);
                let b = ((apu as f32 * csk) as u32).min(255);
                buf[idx] = 0xFF000000 | (r << 16) | (g << 8) | b;
            }
        }

        
        let iez = h * 45 / 100; 
        let fzl: u32 = 0x18;
        let irm = 10;
        for i in 0..irm {
            let t = i as f32 / irm as f32;
            let nn = iez + ((1.0 - t) * (1.0 - t) * (h - iez) as f32) as usize;
            if nn >= h { continue; }
            let ln = (1.0 - t) * (1.0 - t);
            let g = (fzl as f32 * ln * 1.5) as u32;
            if g < 2 { continue; }
            let btx = 0xFF000000 | (g.min(60) / 3 << 16) | (g.min(60) / 2 << 8) | g.min(60);
            let cpq = nn * w;
            let mut p = 0;
            while p < w {
                let idx = cpq + p;
                if idx < buf.len() {
                    let dst = buf[idx];
                    let cpr = ((dst >> 16) & 0xFF) + ((btx >> 16) & 0xFF);
                    let bgq = ((dst >> 8) & 0xFF) + ((btx >> 8) & 0xFF);
                    let mq = (dst & 0xFF) + (btx & 0xFF);
                    buf[idx] = 0xFF000000 | (cpr.min(255) << 16) | (bgq.min(255) << 8) | mq.min(255);
                }
                p += 2;
            }
        }
    }

    
    fn render_scanlines(&self, buf: &mut [u32], w: usize, h: usize) {
        
        let cycle = (h as f32) * 2.0;
        let cox = (self.frame as f32 * 1.8) % cycle;
        let eau = if cox > h as f32 { cycle - cox } else { cox };
        let eau = eau as i32;

        for y in 0..h {
            let fk = y * w;
            let azm = fk + w;
            if azm > buf.len() { break; }

            
            if y % 3 == 0 {
                for p in buf[fk..azm].iter_mut() {
                    let r = ((*p >> 16) & 0xFF) * 200 / 256;
                    let g = ((*p >> 8) & 0xFF) * 200 / 256;
                    let b = (*p & 0xFF) * 200 / 256;
                    *p = 0xFF000000 | (r << 16) | (g << 8) | b;
                }
            }

            
            let em = (y as i32 - eau).unsigned_abs();
            if em < 6 {
                let ahj = (6 - em) * 5;
                for p in buf[fk..azm].iter_mut() {
                    let r = (((*p >> 16) & 0xFF) + ahj).min(255);
                    let g = (((*p >> 8) & 0xFF) + ahj).min(255);
                    let b = ((*p & 0xFF) + ahj).min(255);
                    *p = 0xFF000000 | (r << 16) | (g << 8) | b;
                }
            }

            
            let mux = (y as u32).wrapping_mul(2654435761).wrapping_add(self.frame);
            if mux % 97 < 3 {
                
                let no = 2;
                let row = &mut buf[fk..azm];
                if w > no {
                    for x in (no..w).rev() {
                        row[x] = row[x - no];
                    }
                    for x in 0..no {
                        row[x] = 0xFF000000;
                    }
                }
            }
        }
    }

    fn render_rain(&self, buf: &mut [u32], w: usize, h: usize) {
        
        let nls = w / 12;
        let frame = self.frame;
        
        for i in 0..nls {
            
            let seed = (i as u32).wrapping_mul(2654435761);
            let x = (seed % w as u32) as i32;
            let speed = 2 + (seed >> 16) % 5;
            let len = 4 + (seed >> 8) % 12;
            let pwa = ((frame.wrapping_mul(speed)) % (h as u32 + len)) as i32;
            
            for ay in 0..len as i32 {
                let y = pwa - ay;
                if y >= 0 && y < h as i32 {
                    let ln = (len as i32 - ay) as u32 * 255 / len as u32;
                    let g = (ln * 180 / 255).min(255);
                    let color = 0xFF000000 | (g << 8);
                    let idx = y as usize * w + x as usize;
                    if idx < buf.len() {
                        buf[idx] = color;
                    }
                }
            }
        }
    }
}
