


















extern crate alloc;
use alloc::vec::Vec;





#[derive(Clone, Copy)]
struct V3 { x: f32, y: f32, z: f32 }

impl V3 {
    const fn new(x: f32, y: f32, z: f32) -> Self { Self { x, y, z } }
}

#[derive(Clone, Copy)]
struct H(u16, u16);


#[derive(Clone, Copy)]
pub struct RainEffect {
    pub glow: u8,
    pub depth: u8,
    pub trail_boost: u8,
    pub ripple: u8,
    pub dim: u8,
    pub fresnel: u8,
    pub specular: u8,
    pub ao: u8,
    pub bloom: u8,
    pub scanline: u8,
    pub inner_glow: u8,
    pub shadow: u8,
    
    pub target_r: u8,
    pub target_g: u8,
    pub target_b: u8,
    
    pub target_blend: u8,
}

impl RainEffect {
    pub const Bc: Self = Self {
        glow: 0, depth: 128, trail_boost: 0, ripple: 0, dim: 0,
        fresnel: 0, specular: 0, ao: 0, bloom: 0, scanline: 0,
        inner_glow: 0, shadow: 0,
        target_r: 0, target_g: 0, target_b: 0, target_blend: 0,
    };
}





pub const JJ_: u8 = 14;

pub const PE_: [&str; 14] = [
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





pub const AHY_: u8 = 24;

pub const CLV_: [&str; 24] = [
    
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



pub fn auk(palette: u8, t: f32) -> (f32, f32, f32) {
    let t = t.max(0.0).min(1.0);
    match palette {
        0 => { 
            let r = 20.0 + t * 80.0;
            let g = 120.0 + t * 135.0;
            let b = 20.0 + t * 60.0;
            (r, g, b)
        }
        1 => { 
            if t < 0.33 {
                let j = t / 0.33;
                (20.0 + j * 60.0, 180.0 + j * 75.0, 255.0) 
            } else if t < 0.66 {
                let j = (t - 0.33) / 0.33;
                (80.0 + j * 175.0, 255.0 - j * 175.0, 255.0 - j * 55.0) 
            } else {
                let j = (t - 0.66) / 0.34;
                (255.0, 80.0 - j * 40.0, 200.0 + j * 55.0) 
            }
        }
        2 => { 
            if t < 0.4 {
                let j = t / 0.4;
                (180.0 + j * 75.0, 20.0 + j * 60.0, 0.0) 
            } else if t < 0.75 {
                let j = (t - 0.4) / 0.35;
                (255.0, 80.0 + j * 120.0, j * 30.0) 
            } else {
                let j = (t - 0.75) / 0.25;
                (255.0, 200.0 + j * 55.0, 30.0 + j * 200.0) 
            }
        }
        3 => { 
            if t < 0.5 {
                let j = t / 0.5;
                (0.0, 30.0 + j * 100.0, 120.0 + j * 135.0) 
            } else {
                let j = (t - 0.5) / 0.5;
                (j * 80.0, 130.0 + j * 125.0, 255.0) 
            }
        }
        4 => { 
            if t < 0.33 {
                let j = t / 0.33;
                (30.0 + j * 40.0, 200.0 + j * 55.0, 80.0 + j * 40.0) 
            } else if t < 0.66 {
                let j = (t - 0.33) / 0.33;
                (70.0 + j * 100.0, 255.0 - j * 155.0, 120.0 + j * 80.0) 
            } else {
                let j = (t - 0.66) / 0.34;
                (170.0 + j * 85.0, 100.0 + j * 50.0, 200.0 + j * 55.0) 
            }
        }
        5 => { 
            if t < 0.5 {
                let j = t / 0.5;
                (180.0 + j * 75.0, 130.0 + j * 50.0, 10.0 + j * 20.0)
            } else {
                let j = (t - 0.5) / 0.5;
                (255.0, 180.0 + j * 75.0, 30.0 + j * 225.0)
            }
        }
        6 => { 
            if t < 0.4 {
                let j = t / 0.4;
                (100.0 + j * 100.0, 5.0 + j * 15.0, 5.0 + j * 10.0)
            } else if t < 0.75 {
                let j = (t - 0.4) / 0.35;
                (200.0 + j * 55.0, 20.0 + j * 40.0, 15.0 + j * 30.0)
            } else {
                let j = (t - 0.75) / 0.25;
                (255.0, 60.0 + j * 160.0, 45.0 + j * 180.0)
            }
        }
        7 => { 
            if t < 0.4 {
                let j = t / 0.4;
                (5.0 + j * 15.0, 10.0 + j * 40.0, 100.0 + j * 100.0)
            } else if t < 0.75 {
                let j = (t - 0.4) / 0.35;
                (20.0 + j * 40.0, 50.0 + j * 80.0, 200.0 + j * 55.0)
            } else {
                let j = (t - 0.75) / 0.25;
                (60.0 + j * 160.0, 130.0 + j * 125.0, 255.0)
            }
        }
        8 => { 
            if t < 0.4 {
                let j = t / 0.4;
                (60.0 + j * 50.0, 5.0 + j * 15.0, 100.0 + j * 60.0)
            } else if t < 0.75 {
                let j = (t - 0.4) / 0.35;
                (110.0 + j * 60.0, 20.0 + j * 40.0, 160.0 + j * 60.0)
            } else {
                let j = (t - 0.75) / 0.25;
                (170.0 + j * 70.0, 60.0 + j * 140.0, 220.0 + j * 35.0)
            }
        }
        9 => { 
            if t < 0.4 {
                let j = t / 0.4;
                (140.0 + j * 60.0, 10.0 + j * 20.0, 80.0 + j * 40.0)
            } else if t < 0.75 {
                let j = (t - 0.4) / 0.35;
                (200.0 + j * 55.0, 30.0 + j * 50.0, 120.0 + j * 40.0)
            } else {
                let j = (t - 0.75) / 0.25;
                (255.0, 80.0 + j * 130.0, 160.0 + j * 80.0)
            }
        }
        10 => { 
            if t < 0.4 {
                let j = t / 0.4;
                (140.0 + j * 60.0, 100.0 + j * 50.0, 5.0 + j * 10.0)
            } else if t < 0.75 {
                let j = (t - 0.4) / 0.35;
                (200.0 + j * 55.0, 150.0 + j * 75.0, 15.0 + j * 20.0)
            } else {
                let j = (t - 0.75) / 0.25;
                (255.0, 225.0 + j * 30.0, 35.0 + j * 200.0)
            }
        }
        11 => { 
            if t < 0.4 {
                let j = t / 0.4;
                (5.0 + j * 10.0, 80.0 + j * 70.0, 100.0 + j * 60.0)
            } else if t < 0.75 {
                let j = (t - 0.4) / 0.35;
                (15.0 + j * 30.0, 150.0 + j * 80.0, 160.0 + j * 95.0)
            } else {
                let j = (t - 0.75) / 0.25;
                (45.0 + j * 180.0, 230.0 + j * 25.0, 255.0)
            }
        }
        _ => { 
            if t < 0.4 {
                let j = t / 0.4;
                let v = 60.0 + j * 60.0;
                (v, v, v)
            } else if t < 0.75 {
                let j = (t - 0.4) / 0.35;
                let v = 120.0 + j * 80.0;
                (v, v + j * 10.0, v + j * 15.0) 
            } else {
                let j = (t - 0.75) / 0.25;
                let v = 200.0 + j * 55.0;
                (v, v, v)
            }
        }
    }
}



fn bra(gw: u8, gn: u8, aih: u8, t: f32) -> (f32, f32, f32) {
    let t = t.max(0.0).min(1.0);
    if t < 0.3 {
        
        auk(gw, t / 0.3)
    } else if t < 0.45 {
        
        let j = (t - 0.3) / 0.15;
        let (uh, bbu, gf) = auk(gw, 0.8 + j * 0.2);
        let (ju, axe, iq) = auk(gn, j * 0.2);
        (uh * (1.0 - j) + ju * j, bbu * (1.0 - j) + axe * j, gf * (1.0 - j) + iq * j)
    } else if t < 0.65 {
        
        let j = (t - 0.45) / 0.2;
        auk(gn, j)
    } else if t < 0.8 {
        
        let j = (t - 0.65) / 0.15;
        let (uh, bbu, gf) = auk(gn, 0.8 + j * 0.2);
        let (ju, axe, iq) = auk(aih, j * 0.2);
        (uh * (1.0 - j) + ju * j, bbu * (1.0 - j) + axe * j, gf * (1.0 - j) + iq * j)
    } else {
        
        let j = (t - 0.8) / 0.2;
        auk(aih, j)
    }
}


fn gpo(t: f32) -> (f32, f32, f32) {
    let t = t.max(0.0).min(1.0);
    
    let h = t * 6.0; 
    let i = h as u8;
    let f = h - i as f32;
    match i {
        0 => (255.0, f * 255.0, 0.0),                    
        1 => (255.0 * (1.0 - f), 255.0, 0.0),            
        2 => (0.0, 255.0, f * 255.0),                     
        3 => (0.0, 255.0 * (1.0 - f), 255.0),            
        4 => (f * 255.0, 0.0, 255.0),                     
        _ => (255.0, 0.0, 255.0 * (1.0 - f)),            
    }
}


fn obg(t: f32) -> (f32, f32, f32) {
    
    let bits = t.to_bits();
    let hash = bits.wrapping_mul(2654435761); 
    let zz = (hash % 360) as f32 / 360.0;
    gpo(zz)
}


pub fn oba(col: usize, row: usize, frame: u32) -> (u8, u8, u8) {
    let seed = (col as u32).wrapping_mul(2654435761)
        ^ (row as u32).wrapping_mul(1103515245)
        ^ frame.wrapping_mul(214013).wrapping_add(2531011);
    let zz = (seed % 360) as f32 / 360.0;
    let (r, g, b) = gpo(zz);
    (r.min(255.0) as u8, g.min(255.0) as u8, b.min(255.0) as u8)
}


pub fn bsy(palette: u8, t: f32) -> (f32, f32, f32) {
    match palette {
        0..=12 => auk(palette, t),
        13 => gpo(t),                     
        14 => bra(1, 2, 4, t),            
        15 => bra(2, 3, 5, t),            
        16 => bra(1, 4, 3, t),            
        17 => bra(6, 5, 8, t),            
        18 => bra(11, 7, 12, t),          
        19 => bra(0, 10, 11, t),          
        20 => bra(6, 8, 9, t),            
        21 => bra(7, 8, 9, t),            
        22 => bra(6, 2, 10, t),           
        _  => obg(t),                      
    }
}





pub struct VisualizerState {
    
    pub mode: u8,

    
    verts: Vec<V3>,           
    edges: Vec<H>,         
    pverts: Vec<(i32, i32)>,  
    proj_z: Vec<i16>,         

    
    rot_x: i32, rot_y: i32, rot_z: i32,
    rot_speed_x: i32, rot_speed_y: i32, rot_speed_z: i32,
    scale: i32, scale_target: i32,
    center_x: i32, center_y: i32,

    
    pub column_hits: Vec<Vec<(i32, i32, u16, u8, i16)>>,
    pub column_bounds: Vec<(i32, i32)>,

    
    pub z_min: i16, pub z_max: i16,
    pub col_w: i32,
    pub frame: u64,
    pub smooth_sub_bass: f32,
    pub smooth_bass: f32,
    pub smooth_mid: f32,
    pub smooth_treble: f32,
    pub beat_pulse: f32,
    pub ripple_radius: f32,
    pub spec_x: i32, pub spec_y: i32,
    pub shadow_y_start: i32, pub shadow_y_end: i32,
    pub shape_center_y: i32,

    
    sphere_base: Vec<V3>,
    sphere_edges: Vec<H>,
    sphere_bands: Vec<u8>,

    
    morph_shapes: [Vec<V3>; 4],     
    morph_edges_all: [Vec<H>; 4],
    morph_current: Vec<V3>,
    morph_phase: f32,               
    morph_idx: usize,               

    
    lorenz_trail: Vec<V3>,          
    lorenz_state: [f32; 3],         

    
    spec_base: Vec<V3>,
    spec_bands: Vec<u8>,
    spec_edges: Vec<H>,

    
    ribbon_spine: Vec<V3>,

    
    particles: Vec<Aci>,
    particle_timer: u32,

    
    image_offset_x: i32,
    image_offset_y: i32,
    image_display_w: u32,
    image_display_h: u32,

    
    torus_knot_base: Vec<V3>,
    torus_knot_edges: Vec<H>,

    
    dna_base: Vec<V3>,
    dna_edges: Vec<H>,

    
    tesseract_base: Vec<V3>,   
    tesseract_edges: Vec<H>,
    tesseract_w_angle: f32,    

    
    vortex_base: Vec<V3>,
    vortex_edges: Vec<H>,

    
    plasma_base: Vec<V3>,
    plasma_edges: Vec<H>,
    plasma_tendrils: Vec<V3>,
    plasma_tendril_edges: Vec<H>,

    
    galaxy_base: Vec<V3>,
    galaxy_edges: Vec<H>,

    
    dvd_x: f32,           
    dvd_y: f32,           
    dvd_vx: f32,          
    dvd_vy: f32,          
    dvd_flash: f32,       
    subscribe_base: Vec<V3>,
    subscribe_edges: Vec<H>,

    
    pub palette: u8,

    pub initialized: bool,
}

struct Aci {
    x: f32, y: f32, z: f32,
    vx: f32, vy: f32, vz: f32,
    life: f32,
}

impl VisualizerState {
    pub const fn new() -> Self {
        Self {
            mode: 9,
            verts: Vec::new(), edges: Vec::new(),
            pverts: Vec::new(), proj_z: Vec::new(),
            rot_x: 0, rot_y: 0, rot_z: 0,
            rot_speed_x: 6, rot_speed_y: 10, rot_speed_z: 2,
            scale: 180, scale_target: 180,
            center_x: 0, center_y: 0,
            column_hits: Vec::new(), column_bounds: Vec::new(),
            z_min: 0, z_max: 0, col_w: 8,
            frame: 0,
            smooth_sub_bass: 0.0, smooth_bass: 0.0,
            smooth_mid: 0.0, smooth_treble: 0.0,
            beat_pulse: 0.0, ripple_radius: 999.0,
            spec_x: 0, spec_y: 0,
            shadow_y_start: 0, shadow_y_end: 0, shape_center_y: 0,
            sphere_base: Vec::new(), sphere_edges: Vec::new(), sphere_bands: Vec::new(),
            morph_shapes: [Vec::new(), Vec::new(), Vec::new(), Vec::new()],
            morph_edges_all: [Vec::new(), Vec::new(), Vec::new(), Vec::new()],
            morph_current: Vec::new(),
            morph_phase: 0.0, morph_idx: 0,
            lorenz_trail: Vec::new(), lorenz_state: [1.0, 1.0, 1.0],
            spec_base: Vec::new(), spec_bands: Vec::new(), spec_edges: Vec::new(),
            ribbon_spine: Vec::new(),
            particles: Vec::new(), particle_timer: 0,
            image_offset_x: 0, image_offset_y: 0,
            image_display_w: 0, image_display_h: 0,
            torus_knot_base: Vec::new(), torus_knot_edges: Vec::new(),
            dna_base: Vec::new(), dna_edges: Vec::new(),
            tesseract_base: Vec::new(), tesseract_edges: Vec::new(), tesseract_w_angle: 0.0,
            vortex_base: Vec::new(), vortex_edges: Vec::new(),
            plasma_base: Vec::new(), plasma_edges: Vec::new(),
            plasma_tendrils: Vec::new(), plasma_tendril_edges: Vec::new(),
            galaxy_base: Vec::new(), galaxy_edges: Vec::new(),
            dvd_x: 200.0, dvd_y: 150.0, dvd_vx: 1.8, dvd_vy: 1.2,
            dvd_flash: 0.0,
            subscribe_base: Vec::new(), subscribe_edges: Vec::new(),
            palette: 0,
            initialized: false,
        }
    }
}





fn bjf(mrad: i32) -> i32 {
    let ecv: i32 = 6283;
    let mut a = mrad % ecv;
    if a < 0 { a += ecv; }
    let cob: i32 = 3141;
    let (a_n, dzm) = if a > cob { (a - cob, -1i32) } else { (a, 1) };
    let aa = cob as i64;
    let x = a_n as i64;
    let mh = x * (aa - x);
    let anz = 5 * aa * aa - 4 * mh;
    if anz == 0 { return 0; }
    (16 * mh * 1000 / anz) as i32 * dzm
}

fn bre(mrad: i32) -> i32 { bjf(mrad + 1571) }
fn sinf(r: f32) -> f32 { bjf((r * 1000.0) as i32) as f32 / 1000.0 }
fn cosf(r: f32) -> f32 { bre((r * 1000.0) as i32) as f32 / 1000.0 }

fn avl(v: V3, da: i32, cm: i32, qp: i32, j: i32) -> (i32, i32, i32) {
    let vx = (v.x * j as f32) as i32;
    let vy = (v.y * j as f32) as i32;
    let vz = (v.z * j as f32) as i32;
    let (am, cx) = (bjf(da), bre(da));
    let y1 = (vy * cx - vz * am) / 1000;
    let po = (vy * am + vz * cx) / 1000;
    let (ak, u) = (bjf(cm), bre(cm));
    let x2 = (vx * u + po * ak) / 1000;
    let qt = (-vx * ak + po * u) / 1000;
    let (fq, mj) = (bjf(qp), bre(qp));
    let x3 = (x2 * mj - y1 * fq) / 1000;
    let bkf = (x2 * fq + y1 * mj) / 1000;
    (x3, bkf, qt)
}

fn project(x: i32, y: i32, z: i32, cx: i32, u: i32) -> (i32, i32) {
    let d: i32 = 600;
    let anz = d + z;
    if anz <= 10 { return (cx, u); }
    (cx + x * d / anz, u - y * d / anz)
}

fn lerp(a: f32, b: f32, t: f32) -> f32 { a + (b - a) * t }





const JR_: usize = 8;
const EL_: usize = 12;

fn mby() -> (Vec<V3>, Vec<H>, Vec<u8>) {
    let mut verts = Vec::with_capacity(JR_ * EL_ + 2);
    let mut apt = Vec::with_capacity(JR_ * EL_ + 2);
    let mut edges = Vec::new();
    let pi: f32 = 3.14159265;
    verts.push(V3::new(0.0, -1.0, 0.0)); apt.push(0);
    for ady in 0..JR_ {
        let yt = (ady as f32 + 1.0) / (JR_ as f32 + 1.0);
        let cc = -pi / 2.0 + pi * yt;
        let y = sinf(cc);
        let r = cosf(cc);
        let akx: u8 = match ady { 0 => 0, 1 => 1, 2..=5 => 2, _ => 3 };
        for ud in 0..EL_ {
            let xu = 2.0 * pi * (ud as f32) / (EL_ as f32);
            verts.push(V3::new(r * cosf(xu), y, r * sinf(xu)));
            apt.push(akx);
        }
    }
    verts.push(V3::new(0.0, 1.0, 0.0)); apt.push(3);
    let gjv = verts.len() - 1;
    for ady in 0..JR_ {
        let base = 1 + ady * EL_;
        for ud in 0..EL_ {
            let next = if ud + 1 < EL_ { ud + 1 } else { 0 };
            edges.push(H((base + ud) as u16, (base + next) as u16));
        }
    }
    for ud in 0..EL_ {
        edges.push(H(0, (1 + ud) as u16));
        for ady in 0..(JR_ - 1) {
            let a = 1 + ady * EL_ + ud;
            let b = 1 + (ady + 1) * EL_ + ud;
            edges.push(H(a as u16, b as u16));
        }
        edges.push(H((1 + (JR_ - 1) * EL_ + ud) as u16, gjv as u16));
    }
    (verts, edges, apt)
}


fn mbu() -> (Vec<V3>, Vec<H>) {
    let aij: f32 = 1.618034; 
    let j = 1.0 / libm::sqrtf(1.0 + aij * aij);
    let l = aij * j;
    let verts = alloc::vec![
        V3::new(-j,  l, 0.0), V3::new( j,  l, 0.0),
        V3::new(-j, -l, 0.0), V3::new( j, -l, 0.0),
        V3::new(0.0, -j,  l), V3::new(0.0,  j,  l),
        V3::new(0.0, -j, -l), V3::new(0.0,  j, -l),
        V3::new( l, 0.0, -j), V3::new( l, 0.0,  j),
        V3::new(-l, 0.0, -j), V3::new(-l, 0.0,  j),
    ];
    #[allow(clippy::identity_op)]
    let edges = alloc::vec![
        H(0,1), H(0,5), H(0,7), H(0,10), H(0,11),
        H(1,5), H(1,7), H(1,8), H(1,9),
        H(2,3), H(2,4), H(2,6), H(2,10), H(2,11),
        H(3,4), H(3,6), H(3,8), H(3,9),
        H(4,5), H(4,9), H(4,11),
        H(5,9), H(5,11),
        H(6,7), H(6,8), H(6,10),
        H(7,8), H(7,10),
        H(8,9), H(10,11),
    ];
    (verts, edges)
}


fn mbq() -> (Vec<V3>, Vec<H>) {
    let j: f32 = 0.82;
    let verts = alloc::vec![
        V3::new(-j, -j, -j), V3::new( j, -j, -j),
        V3::new( j,  j, -j), V3::new(-j,  j, -j),
        V3::new(-j, -j,  j), V3::new( j, -j,  j),
        V3::new( j,  j,  j), V3::new(-j,  j,  j),
    ];
    let edges = alloc::vec![
        H(0,1), H(1,2), H(2,3), H(3,0),
        H(4,5), H(5,6), H(6,7), H(7,4),
        H(0,4), H(1,5), H(2,6), H(3,7),
    ];
    (verts, edges)
}


fn mbr() -> (Vec<V3>, Vec<H>) {
    let verts = alloc::vec![
        V3::new(0.0,  1.2, 0.0),  
        V3::new(0.0, -1.2, 0.0),  
        V3::new( 1.0, 0.0, 0.0),  
        V3::new(-1.0, 0.0, 0.0),  
        V3::new(0.0, 0.0,  1.0),  
        V3::new(0.0, 0.0, -1.0),  
    ];
    let edges = alloc::vec![
        H(0,2), H(0,3), H(0,4), H(0,5),
        H(1,2), H(1,3), H(1,4), H(1,5),
        H(2,4), H(4,3), H(3,5), H(5,2),
    ];
    (verts, edges)
}


fn mbz() -> (Vec<V3>, Vec<H>) {
    
    let mut verts = Vec::with_capacity(14);
    
    let j: f32 = 0.55;
    verts.push(V3::new(0.0,  j, 0.0));   
    verts.push(V3::new(0.0, -j, 0.0));   
    verts.push(V3::new( j, 0.0, 0.0));   
    verts.push(V3::new(-j, 0.0, 0.0));   
    verts.push(V3::new(0.0, 0.0,  j));   
    verts.push(V3::new(0.0, 0.0, -j));   
    
    let t: f32 = 1.1;
    verts.push(V3::new( t,  t,  t));     
    verts.push(V3::new(-t,  t,  t));     
    verts.push(V3::new( t, -t,  t));     
    verts.push(V3::new(-t, -t,  t));     
    verts.push(V3::new( t,  t, -t));     
    verts.push(V3::new(-t,  t, -t));     
    verts.push(V3::new( t, -t, -t));     
    verts.push(V3::new(-t, -t, -t));     
    let mut edges = Vec::new();
    
    edges.push(H(0,2)); edges.push(H(0,3)); edges.push(H(0,4)); edges.push(H(0,5));
    edges.push(H(1,2)); edges.push(H(1,3)); edges.push(H(1,4)); edges.push(H(1,5));
    edges.push(H(2,4)); edges.push(H(4,3)); edges.push(H(3,5)); edges.push(H(5,2));
    
    for spike in 6..14u16 {
        
        let amx = verts[spike as usize];
        let mut hss: Vec<(u16, f32)> = (0..6u16).map(|i| {
            let dsz = verts[i as usize];
            let dx = amx.x - dsz.x; let ad = amx.y - dsz.y; let dz = amx.z - dsz.z;
            (i, dx*dx + ad*ad + dz*dz)
        }).collect();
        hss.sort_by(|a, b| {
            if a.1 < b.1 { core::cmp::Ordering::Less }
            else if a.1 > b.1 { core::cmp::Ordering::Greater }
            else { core::cmp::Ordering::Equal }
        });
        for k in 0..3 {
            edges.push(H(spike, hss[k].0));
        }
    }
    (verts, edges)
}






fn mcc(p_k: u32, q_k: u32, segments: usize) -> (Vec<V3>, Vec<H>) {
    let mut verts = Vec::with_capacity(segments);
    let mut edges = Vec::new();
    let ame: f32 = 6.28318;
    let gpl: f32 = 0.7; 
    let exo: f32 = 0.3; 

    for i in 0..segments {
        let t = ame * i as f32 / segments as f32;
        let r = gpl + exo * cosf(q_k as f32 * t);
        let x = r * cosf(p_k as f32 * t);
        let y = r * sinf(p_k as f32 * t);
        let z = exo * sinf(q_k as f32 * t);
        verts.push(V3::new(x, y, z));
    }
    
    for i in 0..segments {
        let next = (i + 1) % segments;
        edges.push(H(i as u16, next as u16));
    }
    
    for i in (0..segments).step_by(4) {
        let cross = (i + segments / 3) % segments;
        edges.push(H(i as u16, cross as u16));
    }
    (verts, edges)
}


fn mbs(segments: usize, bac: f32) -> (Vec<V3>, Vec<H>) {
    
    let mut verts = Vec::with_capacity(segments * 2);
    let mut edges = Vec::new();
    let ame: f32 = 6.28318;
    let epc: f32 = 0.5;
    let height: f32 = 2.0;

    
    for i in 0..segments {
        let t = i as f32 / segments as f32;
        let cc = ame * bac * t;
        let y = -height / 2.0 + height * t;
        verts.push(V3::new(epc * cosf(cc), y, epc * sinf(cc)));
    }
    
    for i in 0..segments {
        let t = i as f32 / segments as f32;
        let cc = ame * bac * t + 3.14159;
        let y = -height / 2.0 + height * t;
        verts.push(V3::new(epc * cosf(cc), y, epc * sinf(cc)));
    }
    
    for i in 0..(segments - 1) {
        edges.push(H(i as u16, (i + 1) as u16)); 
        edges.push(H((segments + i) as u16, (segments + i + 1) as u16)); 
    }
    
    for i in (0..segments).step_by(3) {
        edges.push(H(i as u16, (segments + i) as u16));
    }
    (verts, edges)
}


fn mcb() -> (Vec<[f32; 4]>, Vec<H>) {
    
    let mut feh: Vec<[f32; 4]> = Vec::with_capacity(16);
    let j: f32 = 0.6;
    for i in 0..16u8 {
        let x = if i & 1 != 0 { j } else { -j };
        let y = if i & 2 != 0 { j } else { -j };
        let z = if i & 4 != 0 { j } else { -j };
        let w = if i & 8 != 0 { j } else { -j };
        feh.push([x, y, z, w]);
    }
    
    let mut edges = Vec::new();
    for i in 0..16u16 {
        for bf in 0..4u16 {
            let ay = i ^ (1 << bf);
            if ay > i {
                edges.push(H(i, ay));
            }
        }
    }
    (feh, edges)
}


fn nys(v: [f32; 4], w_angle: f32) -> V3 {
    
    let aq = cosf(w_angle);
    let dy = sinf(w_angle);
    let x = v[0] * aq - v[3] * dy;
    let w = v[0] * dy + v[3] * aq;
    
    let hpy = cosf(w_angle * 0.7);
    let jka = sinf(w_angle * 0.7);
    let y = v[1] * hpy - w * jka;
    let aeo = v[1] * jka + w * hpy;
    
    let hqj: f32 = 2.5;
    let scale = hqj / (hqj + aeo);
    V3::new(x * scale, y * scale, v[2] * scale)
}


fn mcd(rings: usize, segments: usize) -> (Vec<V3>, Vec<H>) {
    let mut verts = Vec::with_capacity(rings * segments);
    let mut edges = Vec::new();
    let ame: f32 = 6.28318;

    for dq in 0..rings {
        let t = dq as f32 / rings as f32;
        let z = -1.0 + 2.0 * t;
        let r = 0.2 + 0.6 * (1.0 - t); 
        let pol = t * 2.0; 
        for gq in 0..segments {
            let cc = ame * gq as f32 / segments as f32 + pol;
            let x = r * cosf(cc);
            let y = r * sinf(cc);
            verts.push(V3::new(x, y, z));
        }
    }
    
    for dq in 0..rings {
        let base = dq * segments;
        for gq in 0..segments {
            let next = (gq + 1) % segments;
            edges.push(H((base + gq) as u16, (base + next) as u16));
        }
    }
    
    for dq in 0..(rings - 1) {
        let base = dq * segments;
        let njy = (dq + 1) * segments;
        for gq in (0..segments).step_by(2) {
            edges.push(H((base + gq) as u16, (njy + gq) as u16));
        }
    }
    (verts, edges)
}


fn mbx() -> (Vec<V3>, Vec<H>) {
    
    let ady: usize = 6;
    let ud: usize = 10;
    let mut verts = Vec::with_capacity(ady * ud + 2);
    let mut edges = Vec::new();
    let pi: f32 = 3.14159;

    verts.push(V3::new(0.0, -0.5, 0.0));
    for xu in 0..ady {
        let yt = (xu as f32 + 1.0) / (ady as f32 + 1.0);
        let cc = -pi / 2.0 + pi * yt;
        let y = sinf(cc) * 0.5;
        let r = cosf(cc) * 0.5;
        for lo in 0..ud {
            let a = 6.28318 * lo as f32 / ud as f32;
            verts.push(V3::new(r * cosf(a), y, r * sinf(a)));
        }
    }
    verts.push(V3::new(0.0, 0.5, 0.0));
    
    for xu in 0..ady {
        let base = 1 + xu * ud;
        for lo in 0..ud {
            let next = if lo + 1 < ud { lo + 1 } else { 0 };
            edges.push(H((base + lo) as u16, (base + next) as u16));
        }
    }
    
    for lo in 0..ud {
        edges.push(H(0, (1 + lo) as u16));
        for xu in 0..(ady - 1) {
            let a = 1 + xu * ud + lo;
            let b = 1 + (xu + 1) * ud + lo;
            edges.push(H(a as u16, b as u16));
        }
        let last = 1 + (ady - 1) * ud + lo;
        edges.push(H(last as u16, (verts.len() - 1) as u16));
    }
    (verts, edges)
}


fn mbt(baj: usize, aor: usize) -> (Vec<V3>, Vec<H>) {
    let mut verts = Vec::with_capacity(baj * aor + 1);
    let mut edges = Vec::new();
    let ame: f32 = 6.28318;

    
    verts.push(V3::new(0.0, 0.0, 0.0));

    for arm in 0..baj {
        let fhj = ame * arm as f32 / baj as f32;
        for i in 0..aor {
            let t = (i as f32 + 1.0) / aor as f32;
            let r = 0.1 + t * 0.9;
            let cc = fhj + t * ame * 1.2; 
            let x = r * cosf(cc);
            let z = r * sinf(cc);
            
            let y = sinf(cc * 2.0) * 0.08 * r;
            verts.push(V3::new(x, y, z));
        }
    }
    
    for arm in 0..baj {
        let base = 1 + arm * aor;
        
        edges.push(H(0, base as u16));
        for i in 0..(aor - 1) {
            edges.push(H((base + i) as u16, (base + i + 1) as u16));
        }
    }
    
    for arm in 0..baj {
        let gjk = (arm + 1) % baj;
        for i in (0..aor).step_by(4) {
            let a = 1 + arm * aor + i;
            let b = 1 + gjk * aor + i;
            if a < verts.len() && b < verts.len() {
                edges.push(H(a as u16, b as u16));
            }
        }
    }
    (verts, edges)
}







fn mca() -> (Vec<V3>, Vec<H>) {
    let mut verts = Vec::with_capacity(256);
    let mut edges = Vec::new();

    
    
    let qx = 0.9f32;   
    let dna = 0.55f32;  
    let dz = 0.08f32;  

    
    let v0 = verts.len();
    verts.push(V3::new(0.0, dna, dz));           
    verts.push(V3::new(qx, 0.0, dz));           
    verts.push(V3::new(0.0, -dna, dz));          
    verts.push(V3::new(-qx, 0.0, dz));          

    
    verts.push(V3::new(0.0, dna, -dz));          
    verts.push(V3::new(qx, 0.0, -dz));          
    verts.push(V3::new(0.0, -dna, -dz));         
    verts.push(V3::new(-qx, 0.0, -dz));         

    
    edges.push(H(v0 as u16, (v0 + 1) as u16));
    edges.push(H((v0 + 1) as u16, (v0 + 2) as u16));
    edges.push(H((v0 + 2) as u16, (v0 + 3) as u16));
    edges.push(H((v0 + 3) as u16, v0 as u16));
    
    edges.push(H((v0 + 4) as u16, (v0 + 5) as u16));
    edges.push(H((v0 + 5) as u16, (v0 + 6) as u16));
    edges.push(H((v0 + 6) as u16, (v0 + 7) as u16));
    edges.push(H((v0 + 7) as u16, (v0 + 4) as u16));
    
    for i in 0..4 {
        edges.push(H((v0 + i) as u16, (v0 + 4 + i) as u16));
    }

    
    let wl = 0.22f32;   
    let qc = 0.28f32;
    let dcv = 0.05f32; 
    let aos = dz + 0.02;

    let jd = verts.len();
    
    verts.push(V3::new(-wl + dcv, qc, aos));       
    verts.push(V3::new(wl * 1.2 + dcv, 0.0, aos)); 
    verts.push(V3::new(-wl + dcv, -qc, aos));      
    
    verts.push(V3::new(-wl + dcv, qc, -aos));
    verts.push(V3::new(wl * 1.2 + dcv, 0.0, -aos));
    verts.push(V3::new(-wl + dcv, -qc, -aos));

    
    edges.push(H(jd as u16, (jd + 1) as u16));
    edges.push(H((jd + 1) as u16, (jd + 2) as u16));
    edges.push(H((jd + 2) as u16, jd as u16));
    
    edges.push(H((jd + 3) as u16, (jd + 4) as u16));
    edges.push(H((jd + 4) as u16, (jd + 5) as u16));
    edges.push(H((jd + 5) as u16, (jd + 3) as u16));
    
    for i in 0..3 {
        edges.push(H((jd + i) as u16, (jd + 3 + i) as u16));
    }

    
    
    let ie = -dna - 0.25;  
    let esx = 0.14f32;    
    let ijz = 0.15f32;    
    let fcw = 0.04f32;      
    let aaj = 9.0 * esx; 
    let start_x = -aaj / 2.0;

    
    
    let mxz: [&[(f32, f32, f32, f32)]; 9] = [
        
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

    for (sz, strokes) in mxz.iter().enumerate() {
        let fh = start_x + sz as f32 * esx;
        for &(x1, y1, x2, y2) in *strokes {
            let wn = fh + x1 * (esx * 0.85);
            let aiu = ie + y1 * ijz;
            let tq = fh + x2 * (esx * 0.85);
            let acv = ie + y2 * ijz;

            let pt = verts.len();
            
            verts.push(V3::new(wn, aiu, fcw));
            verts.push(V3::new(tq, acv, fcw));
            
            verts.push(V3::new(wn, aiu, -fcw));
            verts.push(V3::new(tq, acv, -fcw));
            
            edges.push(H(pt as u16, (pt + 1) as u16));
            
            edges.push(H((pt + 2) as u16, (pt + 3) as u16));
            
            edges.push(H(pt as u16, (pt + 2) as u16));
            edges.push(H((pt + 1) as u16, (pt + 3) as u16));
        }
    }

    (verts, edges)
}

fn kfj(j: &mut VisualizerState) {
    
    j.verts.clear();
    j.verts.extend_from_slice(&j.subscribe_base);
    j.edges.clear();
    j.edges.extend_from_slice(&j.subscribe_edges);
}





fn ensure_init(j: &mut VisualizerState) {
    if j.initialized { return; }

    
    let (amx, dea, cv) = mby();
    j.spec_base = amx.clone();
    j.spec_bands = cv.clone();
    j.spec_edges = dea.clone();
    j.sphere_base = amx;
    j.sphere_edges = dea;
    j.sphere_bands = cv;

    
    let (ico_v, ico_e) = mbu();
    let (cube_v, cube_e) = mbq();
    let (dia_v, dia_e) = mbr();
    let (star_v, star_e) = mbz();

    
    let eud = ico_v.len().max(cube_v.len()).max(dia_v.len()).max(star_v.len());

    
    fn evz(v: &[V3], target: usize) -> Vec<V3> {
        let mut out = v.to_vec();
        while out.len() < target {
            out.push(*out.last().unwrap_or(&V3::new(0.0, 0.0, 0.0)));
        }
        out
    }

    j.morph_shapes[0] = evz(&ico_v, eud);
    j.morph_shapes[1] = evz(&cube_v, eud);
    j.morph_shapes[2] = evz(&dia_v, eud);
    j.morph_shapes[3] = evz(&star_v, eud);
    j.morph_edges_all[0] = ico_e;
    j.morph_edges_all[1] = cube_e;
    j.morph_edges_all[2] = dia_e;
    j.morph_edges_all[3] = star_e;
    j.morph_current = j.morph_shapes[0].clone();

    
    j.lorenz_trail = Vec::with_capacity(400);
    j.lorenz_state = [1.0, 1.0, 1.0];

    
    j.ribbon_spine = Vec::with_capacity(128);

    
    j.particles = Vec::with_capacity(200);

    
    let (tk_v, tk_e) = mcc(2, 3, 120);
    j.torus_knot_base = tk_v;
    j.torus_knot_edges = tk_e;

    
    let (dna_v, dna_e) = mbs(60, 3.0);
    j.dna_base = dna_v;
    j.dna_edges = dna_e;

    
    

    
    let (vt_v, vt_e) = mcd(10, 12);
    j.vortex_base = vt_v;
    j.vortex_edges = vt_e;

    
    let (ps_v, ps_e) = mbx();
    j.plasma_base = ps_v;
    j.plasma_edges = ps_e;

    
    let (gal_v, gal_e) = mbt(4, 20);
    j.galaxy_base = gal_v;
    j.galaxy_edges = gal_e;

    
    let (sub_v, sub_e) = mca();
    j.subscribe_base = sub_v;
    j.subscribe_edges = sub_e;

    j.initialized = true;
}





fn hjb(j: &mut VisualizerState) {
    
    let dhq = [
        j.smooth_sub_bass * 1.2,
        j.smooth_bass * 1.0,
        j.smooth_mid * 0.25,
        j.smooth_treble * 0.15,
    ];
    let ann = j.beat_pulse * (0.3 + j.smooth_sub_bass * 0.3 + j.smooth_bass * 0.2);

    j.verts.clear();
    for i in 0..j.sphere_base.len() {
        let lm = j.sphere_base[i];
        let akx = j.sphere_bands[i] as usize;
        let ank = if akx < 4 { dhq[akx] } else { 0.0 };
        let r = 1.0 + 0.35 * ank + ann * 0.25;
        j.verts.push(V3::new(lm.x * r, lm.y * r, lm.z * r));
    }
    j.edges.clear();
    j.edges.extend_from_slice(&j.sphere_edges);
}

fn kff(j: &mut VisualizerState) {
    
    let lzh = j.morph_idx;
    let jmz = (j.morph_idx + 1) % 4;
    let t = j.morph_phase - libm::floorf(j.morph_phase); 

    
    let gvk = t * t * (3.0 - 2.0 * t);

    let from = &j.morph_shapes[lzh];
    let to = &j.morph_shapes[jmz];
    let count = from.len().min(to.len());

    
    let dae = j.beat_pulse * 0.15;

    j.verts.clear();
    for i in 0..count {
        let x = lerp(from[i].x, to[i].x, gvk) + dae * sinf(i as f32 * 2.1);
        let y = lerp(from[i].y, to[i].y, gvk) + dae * cosf(i as f32 * 1.7);
        let z = lerp(from[i].z, to[i].z, gvk) + dae * sinf(i as f32 * 3.3);
        j.verts.push(V3::new(x, y, z));
    }

    
    j.edges.clear();
    j.edges.extend_from_slice(&j.morph_edges_all[jmz]);
}

fn kfk(j: &mut VisualizerState) {
    
    let osn = 10.0 + j.smooth_bass * 5.0;
    let ogx = 28.0 + j.smooth_mid * 10.0;
    let bqo = 2.667 + j.smooth_treble * 1.0;
    let fm: f32 = 0.006;

    let [x, y, z] = j.lorenz_state;
    let dx = osn * (y - x);
    let ad = x * (ogx - z) - y;
    let dz = x * y - bqo * z;
    let nx = x + dx * fm;
    let re = y + ad * fm;
    let wi = z + dz * fm;
    j.lorenz_state = [nx, re, wi];

    
    
    let scale = 0.04;
    let jd = V3::new(nx * scale, (wi - 25.0) * scale, re * scale);
    j.lorenz_trail.push(jd);
    const EZ_: usize = 350;
    while j.lorenz_trail.len() > EZ_ {
        j.lorenz_trail.remove(0);
    }

    
    j.verts.clear();
    j.edges.clear();
    for aa in &j.lorenz_trail {
        j.verts.push(*aa);
    }
    for i in 0..(j.verts.len().saturating_sub(1)) {
        if i < 65535 {
            j.edges.push(H(i as u16, (i + 1) as u16));
        }
    }
}

fn kfl(j: &mut VisualizerState) {
    
    let dhq = [
        j.smooth_sub_bass * 0.8,
        j.smooth_bass * 0.6,
        j.smooth_mid * 0.5,
        j.smooth_treble * 0.35,
    ];
    let ann = j.beat_pulse * 0.4;

    j.verts.clear();
    for i in 0..j.spec_base.len() {
        let lm = j.spec_base[i];
        let akx = j.spec_bands[i] as usize;
        let ank = if akx < 4 { dhq[akx] } else { 0.0 };
        
        let r = 1.0 + 0.6 * ank + ann * 0.3;
        j.verts.push(V3::new(lm.x * r, lm.y * r, lm.z * r));
    }
    j.edges.clear();
    j.edges.extend_from_slice(&j.spec_edges);
}

fn kfm(j: &mut VisualizerState) {
    
    const Ut: usize = 80;
    const Arx: f32 = 0.6;

    
    if j.ribbon_spine.len() < Ut {
        
        j.ribbon_spine.clear();
        for i in 0..Ut {
            let t = i as f32 / Ut as f32;
            j.ribbon_spine.push(V3::new(0.0, 0.0, -1.5 + 3.0 * t));
        }
    }

    
    j.ribbon_spine.remove(0);
    let mwu = j.ribbon_spine.last().map_or(1.5, |v| v.z);
    let ad = (j.smooth_mid * 0.6 + j.smooth_treble * 0.3) * sinf(j.frame as f32 * 0.1);
    let dx = j.smooth_bass * 0.4 * cosf(j.frame as f32 * 0.07);
    j.ribbon_spine.push(V3::new(dx, ad, mwu));

    
    j.verts.clear();
    j.edges.clear();
    for (i, sp) in j.ribbon_spine.iter().enumerate() {
        
        let w = Arx + j.smooth_sub_bass * 0.3;
        let cc = j.frame as f32 * 0.02 + i as f32 * 0.08;
        let nx = cosf(cc);
        let re = sinf(cc);
        j.verts.push(V3::new(sp.x - nx * w, sp.y - re * w, sp.z));
        j.verts.push(V3::new(sp.x + nx * w, sp.y + re * w, sp.z));
    }
    
    let ae = j.ribbon_spine.len();
    for i in 0..ae {
        let base = (i * 2) as u16;
        
        if base + 1 < j.verts.len() as u16 {
            j.edges.push(H(base, base + 1));
        }
        
        if i + 1 < ae {
            let next = ((i + 1) * 2) as u16;
            j.edges.push(H(base, next));
            j.edges.push(H(base + 1, next + 1));
        }
    }
}

fn kfn(j: &mut VisualizerState) {
    
    j.particle_timer = j.particle_timer.wrapping_add(1);

    
    if j.beat_pulse > 0.8 {
        let count = 30 + (j.smooth_bass * 20.0) as usize;
        let seed = j.frame as u32;
        for i in 0..count.min(50) {
            let hash = seed.wrapping_mul(2654435761).wrapping_add(i as u32);
            let dg = ((hash & 0xFF) as f32 / 128.0 - 1.0) * 0.8;
            let hj = (((hash >> 8) & 0xFF) as f32 / 128.0 - 1.0) * 0.8;
            let maz = (((hash >> 16) & 0xFF) as f32 / 128.0 - 1.0) * 0.8;
            let speed = 0.03 + (j.smooth_sub_bass + j.smooth_bass) * 0.02;
            j.particles.push(Aci {
                x: 0.0, y: 0.0, z: 0.0,
                vx: dg * speed, vy: hj * speed, vz: maz * speed,
                life: 1.0,
            });
        }
    }

    
    for aa in j.particles.iter_mut() {
        aa.x += aa.vx;
        aa.y += aa.vy;
        aa.z += aa.vz;
        aa.vy -= 0.0003; 
        aa.life -= 0.012;
    }
    j.particles.retain(|aa| aa.life > 0.0);
    while j.particles.len() > 200 { j.particles.remove(0); }

    
    j.verts.clear();
    j.edges.clear();
    for aa in &j.particles {
        j.verts.push(V3::new(aa.x, aa.y, aa.z));
    }
    let ae = j.verts.len();
    
    for i in (0..ae.saturating_sub(1)).step_by(1) {
        if i + 1 < ae && i < 65535 {
            j.edges.push(H(i as u16, (i + 1) as u16));
        }
    }
    
    for i in (0..ae.saturating_sub(3)).step_by(3) {
        if i + 3 < ae && i + 3 < 65535 {
            j.edges.push(H(i as u16, (i + 3) as u16));
        }
    }
}

fn kfo(j: &mut VisualizerState) {
    
    
    
    let ark = crate::logo_bitmap::BA_ as u32;
    let arj = crate::logo_bitmap::BN_ as u32;

    
    let bfz = ark * 3 / 2;
    let bsa = arj * 3 / 2;
    j.image_display_w = bfz;
    j.image_display_h = bsa;
    j.image_offset_x = j.center_x - bfz as i32 / 2;
    j.image_offset_y = j.center_y - bsa as i32 / 2;

    
    let kq = (j.beat_pulse * 40.0) as i32;
    j.image_offset_x -= kq / 2;
    j.image_offset_y -= kq / 2;
    j.image_display_w = (bfz as i32 + kq) as u32;
    j.image_display_h = (bsa as i32 + kq) as u32;

    
    let col_w = j.col_w;
    if col_w > 0 {
        let xx = j.column_bounds.len();
        for col in 0..xx {
            let aqa = col as i32 * col_w + col_w / 2;
            if aqa >= j.image_offset_x && aqa < j.image_offset_x + j.image_display_w as i32 {
                j.column_bounds[col] = (
                    j.image_offset_y.max(0),
                    (j.image_offset_y + j.image_display_h as i32).max(0),
                );
            }
        }
    }

    
    j.verts.clear();
    j.edges.clear();
}

fn kfp(j: &mut VisualizerState) {
    
    let ann = j.beat_pulse * 0.3 + j.smooth_bass * 0.2;
    let pnn = j.smooth_treble * 0.15;
    let time = j.frame as f32 * 0.02;

    j.verts.clear();
    for (i, lm) in j.torus_knot_base.iter().enumerate() {
        let t = i as f32 / j.torus_knot_base.len().max(1) as f32;
        
        let r = 1.0 + ann * 0.4 + sinf(t * 6.28 * 3.0 + time) * pnn;
        j.verts.push(V3::new(lm.x * r, lm.y * r, lm.z * r));
    }
    j.edges.clear();
    j.edges.extend_from_slice(&j.torus_knot_edges);
}

fn kfq(j: &mut VisualizerState) {
    
    let ann = j.beat_pulse * 0.25;
    let nfh = j.smooth_mid * 0.3;
    let time = j.frame as f32 * 0.015;
    let segments = j.dna_base.len() / 2;

    j.verts.clear();
    for (i, lm) in j.dna_base.iter().enumerate() {
        let t = (i % segments.max(1)) as f32 / segments.max(1) as f32;
        
        let r = 1.0 + ann * 0.3;
        
        let hxk = nfh * sinf(t * 6.28 + time);
        let apz = cosf(hxk);
        let acl = sinf(hxk);
        let nx = lm.x * apz - lm.z * acl;
        let wi = lm.x * acl + lm.z * apz;
        j.verts.push(V3::new(nx * r, lm.y * r, wi * r));
    }
    j.edges.clear();
    j.edges.extend_from_slice(&j.dna_edges);
}

fn kfr(j: &mut VisualizerState) {
    
    j.tesseract_w_angle += 0.02 + j.smooth_bass * 0.05 + j.beat_pulse * 0.1;

    let (feh, edges4d) = mcb();

    j.verts.clear();
    for v4 in &feh {
        let v3 = nys(*v4, j.tesseract_w_angle);
        
        let kq = 1.0 + j.beat_pulse * 0.3;
        j.verts.push(V3::new(v3.x * kq, v3.y * kq, v3.z * kq));
    }
    j.edges.clear();
    j.edges.extend_from_slice(&edges4d);
}

fn kfg(j: &mut VisualizerState) {
    
    let time = j.frame as f32 * 0.03;
    let ann = j.beat_pulse * 0.4;
    let rings = 10usize;
    let segments = 12usize;

    j.verts.clear();
    for (i, lm) in j.vortex_base.iter().enumerate() {
        let dq = i / segments;
        let t = dq as f32 / rings as f32;
        
        let jbq = 1.0 + ann * (1.0 - t) + j.smooth_sub_bass * 0.2;
        
        let spin = time * (1.0 + t * 2.0);
        let apz = cosf(spin);
        let acl = sinf(spin);
        let nx = lm.x * apz - lm.y * acl;
        let re = lm.x * acl + lm.y * apz;
        j.verts.push(V3::new(nx * jbq, re * jbq, lm.z));
    }
    j.edges.clear();
    j.edges.extend_from_slice(&j.vortex_edges);
}

fn kfh(j: &mut VisualizerState) {
    
    let time = j.frame as f32 * 0.02;
    let ann = j.beat_pulse * 0.5;
    let energy = j.smooth_sub_bass + j.smooth_bass;

    j.verts.clear();
    
    for lm in j.plasma_base.iter() {
        let r = 1.0 + ann * 0.3;
        j.verts.push(V3::new(lm.x * r, lm.y * r, lm.z * r));
    }
    let cvr = j.plasma_base.len();

    
    let gke: usize = 6;
    let fcp: usize = 8;
    let ame: f32 = 6.28318;
    for t in 0..gke {
        let egi = ame * t as f32 / gke as f32 + time * 0.5;
        let fug = sinf(time * 0.7 + t as f32 * 1.5) * 0.6;
        let cgf = cosf(egi) * cosf(fug) * 0.5;
        let bet = sinf(fug) * 0.5;
        let kah = sinf(egi) * cosf(fug) * 0.5;

        for gq in 0..fcp {
            let uz = (gq as f32 + 1.0) / fcp as f32;
            
            let length = 0.5 + energy * 0.4 + ann * 0.3;
            let muy = sinf(time * 3.0 + gq as f32 * 2.0 + t as f32) * 0.08 * uz;
            let muz = cosf(time * 2.5 + gq as f32 * 1.7 + t as f32) * 0.08 * uz;
            let mva = sinf(time * 2.0 + gq as f32 * 2.3 + t as f32) * 0.08 * uz;
            j.verts.push(V3::new(
                cgf * (1.0 + uz * length) + muy,
                bet * (1.0 + uz * length) + muz,
                kah * (1.0 + uz * length) + mva,
            ));
        }
    }

    j.edges.clear();
    j.edges.extend_from_slice(&j.plasma_edges);
    
    for t in 0..gke {
        let dil = cvr + t * fcp;
        
        
        if dil < j.verts.len() && cvr > 0 {
            let csb = j.verts[dil];
            let mut egt = f32::MAX;
            let mut dja = 0u16;
            for ci in 0..cvr {
                let cwe = j.verts[ci];
                let dx = csb.x - cwe.x; let ad = csb.y - cwe.y; let dz = csb.z - cwe.z;
                let d = dx*dx + ad*ad + dz*dz;
                if d < egt { egt = d; dja = ci as u16; }
            }
            j.edges.push(H(dja, dil as u16));
        }
        
        for gq in 0..(fcp - 1) {
            let a = dil + gq;
            let b = dil + gq + 1;
            if a < j.verts.len() && b < j.verts.len() {
                j.edges.push(H(a as u16, b as u16));
            }
        }
    }
}

fn kfi(j: &mut VisualizerState) {
    
    let time = j.frame as f32 * 0.01;
    let ann = j.beat_pulse * 0.3;
    let ame: f32 = 6.28318;
    let baj: usize = 4;
    let aor: usize = 20;

    j.verts.clear();
    
    j.verts.push(V3::new(0.0, sinf(time * 2.0) * 0.05, 0.0));

    for arm in 0..baj {
        let fhj = ame * arm as f32 / baj as f32;
        for i in 0..aor {
            let t = (i as f32 + 1.0) / aor as f32;
            let r = 0.1 + t * (0.9 + ann * 0.3);
            let cc = fhj + t * ame * 1.2 + time;
            let x = r * cosf(cc);
            let z = r * sinf(cc);
            
            let y = sinf(cc * 2.0 + time * 1.5) * 0.1 * r;
            
            let dae = j.smooth_treble * 0.05 * t;
            let mvj = sinf(time * 4.0 + i as f32) * dae;
            let mvk = cosf(time * 3.5 + i as f32) * dae;
            j.verts.push(V3::new(x + mvj, y, z + mvk));
        }
    }

    j.edges.clear();
    
    for arm in 0..baj {
        let base = 1 + arm * aor;
        j.edges.push(H(0, base as u16));
        for i in 0..(aor - 1) {
            j.edges.push(H((base + i) as u16, (base + i + 1) as u16));
        }
    }
    
    for arm in 0..baj {
        let gjk = (arm + 1) % baj;
        for i in (0..aor).step_by(4) {
            let a = 1 + arm * aor + i;
            let b = 1 + gjk * aor + i;
            if a < j.verts.len() && b < j.verts.len() {
                j.edges.push(H(a as u16, b as u16));
            }
        }
    }
}





pub fn update(
    state: &mut VisualizerState,
    screen_w: u32, screen_h: u32,
    matrix_cols: usize,
    beat: f32, energy: f32,
    sub_bass: f32, bass: f32, mid: f32, treble: f32,
    playing: bool,
) {
    ensure_init(state);
    state.frame = state.frame.wrapping_add(1);
    state.center_x = screen_w as i32 / 2;
    state.center_y = screen_h as i32 / 2;

    
    let afs = 0.15f32;
    if playing {
        state.smooth_sub_bass += (sub_bass - state.smooth_sub_bass) * afs;
        state.smooth_bass     += (bass     - state.smooth_bass)     * afs;
        state.smooth_mid      += (mid      - state.smooth_mid)      * afs;
        state.smooth_treble   += (treble   - state.smooth_treble)   * afs;
    } else {
        state.smooth_sub_bass *= 0.95;
        state.smooth_bass     *= 0.95;
        state.smooth_mid      *= 0.95;
        state.smooth_treble   *= 0.95;
    }

    
    let egj = playing && beat > 0.5 && (sub_bass + bass) > 0.4;
    if egj { state.beat_pulse = 1.0; }
    state.beat_pulse *= 0.90;

    
    if state.beat_pulse > 0.9 { state.ripple_radius = 0.0; }
    if state.ripple_radius < 600.0 { state.ripple_radius += 6.0; }

    
    if state.mode == 13 {
        
        state.rot_x += 3;
        state.rot_y += 5;
        state.rot_z += 1;
        state.rot_x %= 6283; state.rot_y %= 6283; state.rot_z %= 6283;
    } else {
        let dir = if playing {
            ((state.smooth_sub_bass + state.smooth_bass) * 15.0 + beat * 8.0) as i32
        } else { 0 };
        state.rot_x += state.rot_speed_x + dir / 4;
        state.rot_y += state.rot_speed_y + dir;
        state.rot_z += state.rot_speed_z;
        state.rot_x %= 6283; state.rot_y %= 6283; state.rot_z %= 6283;
    }

    
    if state.mode == 13 {
        
        state.scale = 160;
        state.scale_target = 160;
    } else {
        state.scale_target = if playing {
            150 + ((state.smooth_sub_bass + state.smooth_bass) * 25.0) as i32
                + (state.beat_pulse * 25.0) as i32
        } else { 150 };
        state.scale += (state.scale_target - state.scale) / 3;
        state.scale = state.scale.max(80).min(220);
    }

    
    if state.mode == 13 {
        
        let orm = (state.scale as f32 * 0.9).max(80.0);
        let orn = (state.scale as f32 * 0.7).max(60.0);

        
        state.dvd_x += state.dvd_vx;
        state.dvd_y += state.dvd_vy;

        
        let bcn = orm;
        let etv = orn;
        let dy = screen_w as f32;
        let dw = screen_h as f32;

        if state.dvd_x - bcn < 0.0 {
            state.dvd_x = bcn;
            state.dvd_vx = state.dvd_vx.abs();
        } else if state.dvd_x + bcn > dy {
            state.dvd_x = dy - bcn;
            state.dvd_vx = -(state.dvd_vx.abs());
        }
        if state.dvd_y - etv < 0.0 {
            state.dvd_y = etv;
            state.dvd_vy = state.dvd_vy.abs();
        } else if state.dvd_y + etv > dw {
            state.dvd_y = dw - etv;
            state.dvd_vy = -(state.dvd_vy.abs());
        }

        
        state.center_x = state.dvd_x as i32;
        state.center_y = state.dvd_y as i32;

        
        if egj {
            state.dvd_flash = 1.0;
        }
        state.dvd_flash *= 0.92;
        if state.dvd_flash < 0.01 { state.dvd_flash = 0.0; }
    }

    
    if state.mode == 1 {
        
        let speed = if state.beat_pulse > 0.5 { 0.08 } else { 0.004 };
        state.morph_phase += speed;
        if state.morph_phase >= 1.0 {
            state.morph_phase -= 1.0;
            state.morph_idx = (state.morph_idx + 1) % 4;
        }
    }

    
    match state.mode {
        0 => hjb(state),
        1 => kff(state),
        2 => kfk(state),
        3 => kfl(state),
        4 => kfm(state),
        5 => kfn(state),
        6 => kfo(state),
        7 => kfp(state),
        8 => kfq(state),
        9 => kfr(state),
        10 => kfg(state),
        11 => kfh(state),
        12 => kfi(state),
        13 => kfj(state),
        _ => hjb(state),
    }

    
    let (scale, da, cm, qp) = (state.scale, state.rot_x, state.rot_y, state.rot_z);
    let (cx, u) = (state.center_x, state.center_y);

    state.pverts.clear();
    state.proj_z.clear();
    let mut ctf: i16 = i16::MAX;
    let mut cte: i16 = i16::MIN;
    for v in &state.verts {
        let (x3, bkf, bxf) = avl(*v, da, cm, qp, scale);
        state.pverts.push(project(x3, bkf, bxf, cx, u));
        let bkk = (bxf as i16).max(-500).min(500);
        state.proj_z.push(bkk);
        if bkk < ctf { ctf = bkk; }
        if bkk > cte { cte = bkk; }
    }
    state.z_min = ctf;
    state.z_max = cte;

    
    {
        let mut diy: i32 = i32::MIN;
        let mut djc: i32 = cx;
        let mut djd: i32 = u;
        let (fe, ly, lz) = (500i32, 700, 500);
        for (pt, v) in state.verts.iter().enumerate() {
            let (nx, re, wi) = avl(*v, da, cm, qp, 1000);
            let dot = (nx * fe + re * ly + wi * lz) / 1000;
            if dot > diy {
                diy = dot;
                if pt < state.pverts.len() {
                    djc = state.pverts[pt].0;
                    djd = state.pverts[pt].1;
                }
            }
        }
        state.spec_x = djc;
        state.spec_y = djd;
    }

    
    {
        let mut bgr: i32 = -1;
        for &(_, bmax) in state.column_bounds.iter() {
            if bmax > bgr { bgr = bmax; }
        }
        if bgr > 0 {
            state.shadow_y_start = bgr;
            state.shadow_y_end = bgr + 120;
        } else {
            state.shadow_y_start = 0; state.shadow_y_end = 0;
        }
        state.shape_center_y = u;
    }

    
    let col_w = if matrix_cols > 0 { screen_w as i32 / matrix_cols as i32 } else { 8 };
    state.col_w = col_w;

    if state.column_hits.len() != matrix_cols {
        state.column_hits.clear();
        state.column_bounds.clear();
        for _ in 0..matrix_cols {
            state.column_hits.push(Vec::new());
            state.column_bounds.push((-1, -1));
        }
    } else {
        for h in state.column_hits.iter_mut() { h.clear(); }
        for b in state.column_bounds.iter_mut() { *b = (-1, -1); }
    }

    for (ei, th) in state.edges.iter().enumerate() {
        let a = th.0 as usize;
        let b = th.1 as usize;
        if a >= state.pverts.len() || b >= state.pverts.len() { continue; }
        let (bm, az) = state.pverts[a];
        let (x1, y1) = state.pverts[b];
        let ctc = if a < state.proj_z.len() && b < state.proj_z.len() {
            ((state.proj_z[a] as i32 + state.proj_z[b] as i32) / 2) as i16
        } else { 0 };
        gpq(bm, az, x1, y1, ei as u16, ctc, col_w, matrix_cols,
                       screen_h as i32, &mut state.column_hits, &mut state.column_bounds);
    }
}

fn gpq(
    bm: i32, az: i32, x1: i32, y1: i32,
    aqk: u16, ctc: i16, col_w: i32, xx: usize, dw: i32,
    hits: &mut [Vec<(i32, i32, u16, u8, i16)>],
    bounds: &mut [(i32, i32)],
) {
    if col_w <= 0 { return; }
    let dx = (x1 - bm).abs();
    let ad = (y1 - az).abs();
    let steps = dx.max(ad).max(1).min(2048);
    let avf = ((x1 - bm) * 1024) / steps;
    let bwa = ((y1 - az) * 1024) / steps;
    let mut p = bm * 1024;
    let mut o = az * 1024;
    let dda = 8i32;
    let stride = 2;
    let mut j = 0;
    while j <= steps {
        let am = p / 1024;
        let ak = o / 1024;
        let c = am / col_w;
        if c >= 0 && (c as usize) < xx && ak >= 0 && ak < dw {
            let ci = c as usize;
            if hits[ci].len() < 64 {
                let bki = (ak - dda).max(0);
                let bkh = (ak + dda).min(dw - 1);
                let em = ((am - c * col_w - col_w / 2).abs() as u32 * 255 / (dda as u32 + 1)).min(255) as u8;
                let intensity = 255u8.saturating_sub(em);
                hits[ci].push((bki, bkh, aqk, intensity, ctc));
            }
            let (ref mut bmin, ref mut bmax) = bounds[ci];
            let bqc = ak;
            if *bmin < 0 || bqc < *bmin { *bmin = bqc; }
            if bqc > *bmax { *bmax = bqc; }
        }
        p += avf * stride;
        o += bwa * stride;
        j += stride;
    }
}





pub fn fli(
    state: &VisualizerState, col: usize, y: i32,
    beat_pulse: f32, energy: f32,
) -> RainEffect {
    if !state.initialized { return RainEffect::Bc; }
    let xx = state.column_hits.len();
    if col >= xx { return RainEffect::Bc; }

    let (cx, u) = (state.center_x, state.center_y);

    
    
    
    if state.mode == 6 && state.image_display_w > 0 && state.image_display_h > 0 {
        let lw = col as i32 * state.col_w + state.col_w / 2;

        
        let sk = lw - state.image_offset_x;
        let qn = y - state.image_offset_y;

        if sk >= 0 && sk < state.image_display_w as i32
            && qn >= 0 && qn < state.image_display_h as i32
        {
            let ark = crate::logo_bitmap::BA_;
            let arj = crate::logo_bitmap::BN_;
            
            let ckw = (sk as u32 * ark as u32 / state.image_display_w) as usize;
            let ckx = (qn as u32 * arj as u32 / state.image_display_h) as usize;

            if ckw < ark && ckx < arj {
                let ct = crate::logo_bitmap::bhr(ckw, ckx);
                let a = (ct >> 24) & 0xFF;
                let ej = (ct >> 16) & 0xFF;
                let abe = (ct >> 8) & 0xFF;
                let ji = ct & 0xFF;

                
                let brightness = (ej.max(abe).max(ji)) as u8;
                if a > 30 && brightness > 15 {
                    
                    
                    let jzx = 140u8 + (brightness / 4);
                    let fip = (beat_pulse * 60.0).min(60.0) as u8;
                    let blend = jzx.saturating_add(fip).min(230);

                    
                    let mut ripple: u8 = 0;
                    {
                        let dx = (lw - cx) as f32;
                        let ad = (y - u) as f32;
                        let em = libm::sqrtf(dx * dx + ad * ad);
                        let cdh = libm::fabsf(em - state.ripple_radius);
                        if cdh < 35.0 {
                            let life = (1.0 - state.ripple_radius / 500.0).max(0.0);
                            let t = (1.0 - cdh / 35.0) * life;
                            ripple = (t * 120.0).min(120.0) as u8;
                        }
                    }

                    
                    let glow = if brightness > 100 { (brightness - 60) / 2 } else { 0 };

                    return RainEffect {
                        glow,
                        depth: 128,
                        trail_boost: 10,
                        ripple,
                        dim: 0,
                        fresnel: 0,
                        specular: 0,
                        ao: 0,
                        bloom: if brightness > 180 { brightness / 3 } else { 0 },
                        scanline: 0,
                        inner_glow: 0,
                        shadow: 0,
                        target_r: ej as u8,
                        target_g: abe as u8,
                        target_b: ji as u8,
                        target_blend: blend,
                    };
                }
            }
        }

        
        let sk = lw - state.image_offset_x;
        let qn = y - state.image_offset_y;
        let lmv = if sk < 0 { -sk }
            else if sk >= state.image_display_w as i32 { sk - state.image_display_w as i32 + 1 }
            else { 0 };
        let lmy = if qn < 0 { -qn }
            else if qn >= state.image_display_h as i32 { qn - state.image_display_h as i32 + 1 }
            else { 0 };
        let bma = lmv.max(lmy);
        if bma > 0 && bma < 40 {
            let dim = ((1.0 - bma as f32 / 40.0) * 50.0) as u8;
            return RainEffect { dim, ..RainEffect::Bc };
        }

        return RainEffect::Bc;
    }

    
    
    

    
    let eyd = if state.mode == 13 { state.dvd_flash } else { 0.0 };

    
    let mut ripple: u8 = 0;
    {
        let dx = (col as i32 * state.col_w + state.col_w / 2 - cx) as f32;
        let ad = (y - u) as f32;
        let em = libm::sqrtf(dx * dx + ad * ad);
        let cdh = libm::fabsf(em - state.ripple_radius);
        if cdh < 35.0 {
            let life = (1.0 - state.ripple_radius / 500.0).max(0.0);
            let t = (1.0 - cdh / 35.0) * life;
            ripple = (t * 120.0).min(120.0) as u8;
        }
    }

    
    let hits = &state.column_hits[col];
    for &(bki, bkh, _eidx, intensity, z_depth) in hits.iter() {
        if y >= bki && y <= bkh {
            let bma = (y - (bki + bkh) / 2).abs();
            let cw = (bkh - bki) / 2;
            let mez = if cw > 0 {
                1.0 - (bma as f32 / cw as f32).min(1.0)
            } else { 1.0 };
            let glow = (80.0 + mez * 175.0 * (intensity as f32 / 255.0)) as u8;

            
            let ffs = (state.z_max as i32 - state.z_min as i32).max(1);
            let depth = ((z_depth as i32 - state.z_min as i32) * 255 / ffs).max(0).min(255) as u8;

            
            let (bmin, bmax) = state.column_bounds[col];
            let fresnel = if bmax > bmin {
                let fqd = (bmin + bmax) / 2;
                let fua = (y - fqd).abs() as f32;
                let kh = ((bmax - bmin) / 2) as f32;
                if kh > 0.0 {
                    let f = (fua / kh).min(1.0);
                    (f * f * 255.0) as u8
                } else { 0 }
            } else { 0 };

            
            let cqb = (col as i32 * state.col_w - state.spec_x).abs();
            let cqc = (y - state.spec_y).abs();
            let jhd = cqb + cqc;
            let specular = if jhd < 60 {
                ((1.0 - jhd as f32 / 60.0) * 255.0) as u8
            } else { 0 };

            
            let ao = if bmax > bmin {
                let t = (y - bmin) as f32 / (bmax - bmin).max(1) as f32;
                let ivl = (0.5 - t).abs() * 2.0; 
                (ivl * ivl * 100.0) as u8
            } else { 0 };

            
            let bloom = (glow as u16 * depth as u16 / 512).min(200) as u8;

            
            let inner_glow = if bmax > bmin {
                let dx = col as i32 * state.col_w + state.col_w / 2 - cx;
                let ad = y - u;
                let r = libm::sqrtf((dx * dx + ad * ad) as f32);
                let aug = ((bmax - bmin) as f32 / 2.0).max(1.0);
                let t = (1.0 - r / aug).max(0.0);
                (t * t * 180.0) as u8
            } else { 0 };

            return RainEffect {
                glow, depth,
                trail_boost: (glow / 4).min(80),
                ripple,
                dim: 0,
                fresnel, specular, ao, bloom,
                scanline: 0,
                inner_glow,
                shadow: 0,
                target_r: if eyd > 0.01 { 255 } else { 0 },
                target_g: 0,
                target_b: 0,
                target_blend: (eyd * 220.0).min(220.0) as u8,
            };
        }
    }

    
    let (bmin, bmax) = state.column_bounds[col];
    if bmin >= 0 && y >= bmin && y <= bmax {
        let glow = 20u8;
        let ffs = (state.z_max as i32 - state.z_min as i32).max(1);
        let depth = 128u8;

        let fresnel = {
            let fqd = (bmin + bmax) / 2;
            let fua = (y - fqd).abs() as f32;
            let kh = ((bmax - bmin) / 2) as f32;
            if kh > 0.0 {
                let f = (fua / kh).min(1.0);
                (f * f * 200.0) as u8
            } else { 0 }
        };

        let inner_glow = {
            let dx = col as i32 * state.col_w + state.col_w / 2 - cx;
            let ad = y - u;
            let r = libm::sqrtf((dx * dx + ad * ad) as f32);
            let aug = ((bmax - bmin) as f32 / 2.0).max(1.0);
            let t = (1.0 - r / aug).max(0.0);
            (t * t * 120.0) as u8
        };

        return RainEffect {
            glow, depth, trail_boost: 5,
            ripple, dim: 0,
            fresnel, specular: 0, ao: 0, bloom: 0,
            scanline: 0, inner_glow, shadow: 0,
            target_r: if eyd > 0.01 { 255 } else { 0 },
            target_g: 0,
            target_b: 0,
            target_blend: (eyd * 180.0).min(180.0) as u8,
        };
    }

    
    if bmin >= 0 {
        let gll = if y < bmin { bmin - y } else if y > bmax { y - bmax } else { 0 };
        if gll > 0 && gll < 60 {
            let dim = ((1.0 - gll as f32 / 60.0) * 80.0) as u8;
            return RainEffect {
                glow: 0, depth: 128, trail_boost: 0,
                ripple, dim, fresnel: 0, specular: 0, ao: 0,
                bloom: 0, scanline: 0, inner_glow: 0, shadow: 0,
                target_r: 0, target_g: 0, target_b: 0, target_blend: 0,
            };
        }
    }

    
    if y > state.shadow_y_start && y < state.shadow_y_end {
        let t = (y - state.shadow_y_start) as f32 / (state.shadow_y_end - state.shadow_y_start) as f32;
        let shadow = ((1.0 - t) * (1.0 - t) * 120.0) as u8;
        return RainEffect {
            glow: 0, depth: 128, trail_boost: 0,
            ripple, dim: 0, fresnel: 0, specular: 0, ao: 0,
            bloom: 0, scanline: 0, inner_glow: 0, shadow,
            target_r: 0, target_g: 0, target_b: 0, target_blend: 0,
        };
    }

    
    if y > state.shape_center_y && y < state.shape_center_y + 200 {
        let p = col as i32 * state.col_w + state.col_w / 2;
        let dx = (p - cx).abs();
        if dx < 80 && col % 4 == 0 {
            let iyc = (1.0 - dx as f32 / 80.0) * (1.0 - (y - state.shape_center_y) as f32 / 200.0);
            if iyc > 0.05 {
                let scanline = (iyc * 150.0).min(200.0) as u8;
                return RainEffect {
                    glow: 0, depth: 128, trail_boost: 0,
                    ripple, dim: 0, fresnel: 0, specular: 0, ao: 0,
                    bloom: 0, scanline, inner_glow: 0, shadow: 0,
                    target_r: 0, target_g: 0, target_b: 0, target_blend: 0,
                };
            }
        }
    }

    
    if ripple > 0 {
        return RainEffect { ripple, ..RainEffect::Bc };
    }

    RainEffect::Bc
}





pub fn gia(
    adi: u8, agd: u8, apu: u8,
    glow: u8, depth: u8, ripple: u8,
    fresnel: u8, specular: u8,
    ao: u8, bloom: u8, scanline: u8, inner_glow: u8, shadow: u8,
    beat: f32, energy: f32,
    sub_bass: f32, bass: f32, mid: f32, treble: f32,
    palette: u8,
) -> (u8, u8, u8) {
    let mut r = adi as f32;
    let mut g = agd as f32;
    let mut b = apu as f32;

    
    let nmy = glow > 80;
    let bmz = inner_glow > 20 || glow > 40;

    
    let byr = depth as f32 / 255.0;

    
    
    
    if shadow > 5 {
        let j = 1.0 - (shadow as f32 / 255.0) * 0.6;
        r *= j; g *= j; b *= j;
    }

    
    
    
    if scanline > 3 {
        let ob = scanline as f32 / 255.0;
        
        let (ej, abe, ji) = bsy(palette, 0.3);
        r = (r + ob * ej * 0.3).min(255.0);
        g = (g + ob * abe * 0.3).min(255.0);
        b = (b + ob * ji * 0.3).min(255.0);
    }

    
    
    
    
    if nmy {
        let atx = glow as f32 / 255.0;

        
        let loa = 0.55 + 0.45 * byr; 

        
        let dgn = atx * loa;
        r = (r * (1.0 - dgn) + 255.0 * dgn).min(255.0);
        g = (g * (1.0 - dgn) + 255.0 * dgn).min(255.0);
        b = (b * (1.0 - dgn) + 255.0 * dgn).min(255.0);

        
        let (ej, abe, ji) = bsy(palette, byr);
        let azx = 0.15 * atx;
        r = (r * (1.0 - azx) + ej * azx).min(255.0);
        g = (g * (1.0 - azx) + abe * azx).min(255.0);
        b = (b * (1.0 - azx) + ji * azx).min(255.0);

        
        if ao > 0 {
            let bep = 1.0 - (ao as f32 / 255.0) * 0.3;
            r *= bep; g *= bep; b *= bep;
        }
    }
    
    
    
    
    
    else if inner_glow > 20 {
        let axo = (inner_glow as f32 - 20.0) / 235.0;
        let axo = axo * axo; 

        
        
        
        let gut = 0.3 + 0.7 * byr;

        
        
        let (ej, abe, ji) = bsy(palette, byr);

        
        let mzq = ej * gut;
        let mzp = abe * gut;
        let mzo = ji * gut;

        
        let blend = (axo * 0.80).min(0.80);
        r = (r * (1.0 - blend) + mzq * blend).min(255.0);
        g = (g * (1.0 - blend) + mzp * blend).min(255.0);
        b = (b * (1.0 - blend) + mzo * blend).min(255.0);

        
        if ao > 0 {
            let bep = 1.0 - (ao as f32 / 255.0) * 0.5;
            r *= bep; g *= bep; b *= bep;
        }

        
        if byr > 0.8 {
            let git = (byr - 0.8) / 0.2 * 30.0;
            r = (r + git).min(255.0);
            g = (g + git).min(255.0);
            b = (b + git).min(255.0);
        }
    }
    
    
    
    else if glow > 0 {
        let atx = glow as f32 / 255.0;
        let (ej, abe, ji) = bsy(palette, byr * 0.5 + 0.3);
        let ahj = atx * 0.5;
        r = (r + ej * ahj).min(255.0);
        g = (g + abe * ahj).min(255.0);
        b = (b + ji * ahj).min(255.0);
    }

    
    
    
    if fresnel > 120 {
        let cxm = (fresnel as f32 - 120.0) / 135.0;
        let dgo = cxm * cxm;
        r = (r * (1.0 - dgo) + 252.0 * dgo).min(255.0);
        g = (g * (1.0 - dgo) + 255.0 * dgo).min(255.0);
        b = (b * (1.0 - dgo) + 252.0 * dgo).min(255.0);
    }

    
    
    
    if specular > 30 {
        let azo = (specular as f32 - 30.0) / 225.0;
        let azo = azo * azo;
        
        let (ej, abe, ji) = bsy(palette, 0.9);
        r = (r + azo * (200.0 + ej * 0.2)).min(255.0);
        g = (g + azo * (220.0 + abe * 0.1)).min(255.0);
        b = (b + azo * (200.0 + ji * 0.2)).min(255.0);
    }

    
    
    
    if bloom > 10 {
        let bl = bloom as f32 / 255.0;
        let (ej, abe, ji) = bsy(palette, 0.5);
        r = (r + bl * ej * 0.15).min(255.0);
        g = (g + bl * abe * 0.15).min(255.0);
        b = (b + bl * ji * 0.15).min(255.0);
    }

    
    
    
    if ripple > 0 {
        let rip = ripple as f32 / 255.0;
        let (ej, abe, ji) = bsy(palette, 0.6);
        r = (r + rip * ej * 0.2).min(255.0);
        g = (g + rip * abe * 0.2).min(255.0);
        b = (b + rip * ji * 0.2).min(255.0);
    }

    
    
    
    if bmz && energy > 0.03 {
        let ety = sub_bass.max(bass).max(mid).max(treble);
        if ety > 0.10 {
            let intensity = (energy * 0.8 + 0.1).min(0.55);

            
            let jzi = if sub_bass >= ety - 0.05 {
                0.1f32
            } else if bass >= ety - 0.05 {
                0.35
            } else if mid >= ety - 0.05 {
                0.65
            } else {
                0.9
            };

            
            let nfl = jzi * 0.6 + byr * 0.4;
            let (tr, bwi, aiv) = bsy(palette, nfl);

            let kq = 1.0 + beat * 0.5;
            let blend = (intensity * kq).min(0.65);
            r = (r * (1.0 - blend) + tr * blend).min(255.0);
            g = (g * (1.0 - blend) + bwi * blend).min(255.0);
            b = (b * (1.0 - blend) + aiv * blend).min(255.0);

            
            if beat > 0.5 {
                let ges = (beat - 0.5) * 45.0;
                r = (r + ges).min(255.0);
                g = (g + ges).min(255.0);
                b = (b + ges).min(255.0);
            }
        }
    }

    
    
    
    if bmz {
        (r.min(255.0) as u8, g.min(255.0) as u8, b.min(255.0) as u8)
    } else if palette == 0 {
        
        let dqk = g as u8;
        let gpk = (r as u8).min(dqk);
        let fhv = (b as u8).min(dqk);
        (gpk, dqk, fhv)
    } else {
        
        let (ej, abe, ji) = bsy(palette, 0.15);
        let azx = 0.18f32;
        let gpk = (r * (1.0 - azx) + ej * azx).min(255.0) as u8;
        let dqk = (g * (1.0 - azx) + abe * azx).min(255.0) as u8;
        let fhv = (b * (1.0 - azx) + ji * azx).min(255.0) as u8;
        (gpk, dqk, fhv)
    }
}





#[inline]
pub fn hmz(state: &VisualizerState, col: usize) -> u8 {
    if col < state.column_bounds.len() {
        let (bmin, bmax) = state.column_bounds[col];
        if bmin >= 0 && bmax > bmin {
            let bjg = (bmax - bmin) as u32;
            let otu = (45 + (55 * bjg / 400).min(55)) as u8;
            return 100u8.saturating_sub(100 - otu);
        }
    }
    100
}
