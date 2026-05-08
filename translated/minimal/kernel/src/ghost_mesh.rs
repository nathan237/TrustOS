


















use alloc::vec::Vec;





#[derive(Clone, Copy)]
pub struct V3 { pub x: f32, pub y: f32, pub z: f32 }

impl V3 {
    pub const fn new(x: f32, y: f32, z: f32) -> Self { Self { x, y, z } }
}

#[derive(Clone, Copy)]
pub struct H(pub u16, pub u16);







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
}

impl RainEffect {
    pub const Bc: Self = Self {
        glow: 0, depth: 128, trail_boost: 0, ripple: 0, dim: 0,
        fresnel: 0, specular: 0, ao: 0, bloom: 0, scanline: 0,
        inner_glow: 0, shadow: 0,
    };
}





const JI_: usize = 8;   
const EJ_: usize = 12;  





pub struct Pa {
    pub base_verts: Vec<V3>,
    pub deformed_verts: Vec<V3>,
    pub edges: Vec<H>,
    pub vert_bands: Vec<u8>,

    pub rot_x: i32,
    pub rot_y: i32,
    pub rot_z: i32,
    pub rot_speed_x: i32,
    pub rot_speed_y: i32,
    pub rot_speed_z: i32,
    pub scale: i32,
    pub scale_target: i32,
    pub center_x: i32,
    pub center_y: i32,

    
    pub column_hits: Vec<Vec<(i32, i32, u16, u8, i16)>>,
    
    
    pub column_bounds: Vec<(i32, i32)>,

    
    
    pub projected_z: Vec<i16>,
    
    pub ripple_radius: f32,
    
    pub z_min: i16,
    pub z_max: i16,
    
    pub col_w: i32,

    pub frame: u64,
    pub smooth_sub_bass: f32,
    pub smooth_bass: f32,
    pub smooth_mid: f32,
    pub smooth_treble: f32,
    pub beat_pulse: f32,
    pub initialized: bool,
    
    pub spec_x: i32,
    pub spec_y: i32,
    
    pub shadow_y_start: i32,
    pub shadow_y_end: i32,
    
    pub shape_center_y: i32,
}

impl Pa {
    pub const fn new() -> Self {
        Self {
            base_verts: Vec::new(),
            deformed_verts: Vec::new(),
            edges: Vec::new(),
            vert_bands: Vec::new(),
            rot_x: 0, rot_y: 0, rot_z: 0,
            rot_speed_x: 6,
            rot_speed_y: 10,
            rot_speed_z: 2,
            scale: 200,
            scale_target: 200,
            center_x: 0, center_y: 0,
            column_hits: Vec::new(),
            column_bounds: Vec::new(),
            projected_z: Vec::new(),
            ripple_radius: 999.0,
            z_min: 0,
            z_max: 0,
            col_w: 8,
            frame: 0,
            smooth_sub_bass: 0.0,
            smooth_bass: 0.0,
            smooth_mid: 0.0,
            smooth_treble: 0.0,
            beat_pulse: 0.0,
            initialized: false,
            spec_x: 0,
            spec_y: 0,
            shadow_y_start: 0,
            shadow_y_end: 0,
            shape_center_y: 0,
        }
    }

    fn ensure_init(&mut self) {
        if self.initialized { return; }
        let (lm, edges, apt) = mcj();
        self.deformed_verts = lm.clone();
        self.base_verts = lm;
        self.edges = edges;
        self.vert_bands = apt;
        self.initialized = true;
    }
}





fn mcj() -> (Vec<V3>, Vec<H>, Vec<u8>) {
    let mut verts = Vec::with_capacity(JI_ * EJ_ + 2);
    let mut apt = Vec::with_capacity(JI_ * EJ_ + 2);
    let mut edges = Vec::new();
    let pi: f32 = 3.14159265;

    
    verts.push(V3::new(0.0, -1.0, 0.0));
    apt.push(0);

    for ady in 0..JI_ {
        let yt = (ady as f32 + 1.0) / (JI_ as f32 + 1.0);
        let cc = -pi / 2.0 + pi * yt;
        let y = sinf(cc);
        let r = cosf(cc);
        let akx: u8 = match ady {
            0     => 0,
            1     => 1,
            2 | 3 => 2,
            4 | 5 => 2,
            _     => 3,
        };
        for ud in 0..EJ_ {
            let xu = 2.0 * pi * (ud as f32) / (EJ_ as f32);
            verts.push(V3::new(r * cosf(xu), y, r * sinf(xu)));
            apt.push(akx);
        }
    }

    
    verts.push(V3::new(0.0, 1.0, 0.0));
    apt.push(3);
    let gjv = verts.len() - 1;

    
    for ady in 0..JI_ {
        let base = 1 + ady * EJ_;
        for ud in 0..EJ_ {
            let next = if ud + 1 < EJ_ { ud + 1 } else { 0 };
            edges.push(H((base + ud) as u16, (base + next) as u16));
        }
    }

    
    for ud in 0..EJ_ {
        edges.push(H(0, (1 + ud) as u16));
        for ady in 0..(JI_ - 1) {
            let a = 1 + ady * EJ_ + ud;
            let b = 1 + (ady + 1) * EJ_ + ud;
            edges.push(H(a as u16, b as u16));
        }
        edges.push(H((1 + (JI_ - 1) * EJ_ + ud) as u16, gjv as u16));
    }

    (verts, edges, apt)
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


fn dsw(val: i32) -> i32 {
    if val <= 0 { return 0; }
    let mut x = val;
    let mut y = (x + 1) / 2;
    while y < x { x = y; y = (x + val / x) / 2; }
    x
}

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





pub fn update(
    state: &mut Pa,
    screen_w: u32, screen_h: u32,
    matrix_cols: usize,
    beat: f32, energy: f32,
    sub_bass: f32, bass: f32, mid: f32, treble: f32,
    playing: bool,
) {
    state.ensure_init();
    state.frame = state.frame.wrapping_add(1);
    state.center_x = screen_w as i32 / 2;
    state.center_y = screen_h as i32 / 3;

    
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

    
    if state.beat_pulse > 0.9 {
        state.ripple_radius = 0.0;
    }
    if state.ripple_radius < 600.0 {
        state.ripple_radius += 6.0;
    }

    
    let dir = if playing {
        ((state.smooth_sub_bass + state.smooth_bass) * 15.0 + beat * 8.0) as i32
    } else { 0 };
    state.rot_x += state.rot_speed_x + dir / 4;
    state.rot_y += state.rot_speed_y + dir;
    state.rot_z += state.rot_speed_z;
    state.rot_x %= 6283;
    state.rot_y %= 6283;
    state.rot_z %= 6283;

    
    state.scale_target = if playing {
        140 + ((state.smooth_sub_bass + state.smooth_bass) * 30.0) as i32
            + (state.beat_pulse * 30.0) as i32
    } else { 140 };
    state.scale += (state.scale_target - state.scale) / 3;
    if state.scale < 80 { state.scale = 80; }
    if state.scale > 220 { state.scale = 220; }

    
    
    let dhq = [
        state.smooth_sub_bass * 1.2,   
        state.smooth_bass     * 1.0,   
        state.smooth_mid      * 0.25,  
        state.smooth_treble   * 0.15,  
    ];
    let ann = state.beat_pulse * (0.3 + state.smooth_sub_bass * 0.3 + state.smooth_bass * 0.2);
    for i in 0..state.base_verts.len() {
        let lm = state.base_verts[i];
        let akx = state.vert_bands[i] as usize;
        let ank = if akx < 4 { dhq[akx] } else { energy * 0.2 };
        let r = 1.0 + 0.35 * ank + ann * 0.25;
        state.deformed_verts[i] = V3::new(lm.x * r, lm.y * r, lm.z * r);
    }

    
    let (scale, da, cm, qp) = (state.scale, state.rot_x, state.rot_y, state.rot_z);
    let (cx, u) = (state.center_x, state.center_y);
    let jpu = state.deformed_verts.len();
    let mut pverts: Vec<(i32, i32)> = Vec::with_capacity(jpu);
    state.projected_z.clear();
    state.projected_z.reserve(jpu);
    let mut ctf: i16 = i16::MAX;
    let mut cte: i16 = i16::MIN;
    for v in &state.deformed_verts {
        let (x3, bkf, bxf) = avl(*v, da, cm, qp, scale);
        pverts.push(project(x3, bkf, bxf, cx, u));
        let bkk = (bxf as i16).max(-500).min(500);
        state.projected_z.push(bkk);
        if bkk < ctf { ctf = bkk; }
        if bkk > cte { cte = bkk; }
    }
    state.z_min = ctf;
    state.z_max = cte;

    
    
    
    
    {
        let mut diy: i32 = i32::MIN;
        let mut djc: i32 = cx;
        let mut djd: i32 = u;
        
        let myg: i32 = 500;  
        let myh: i32 = 700;
        let myi: i32 = 500;
        for (pt, v) in state.deformed_verts.iter().enumerate() {
            
            let (nx, re, wi) = avl(*v, da, cm, qp, 1000);
            
            let dot = (nx * myg + re * myh + wi * myi) / 1000;
            if dot > diy {
                diy = dot;
                if pt < pverts.len() {
                    djc = pverts[pt].0;
                    djd = pverts[pt].1;
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
            state.shadow_y_start = 0;
            state.shadow_y_end = 0;
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
        let (bm, az) = pverts[th.0 as usize];
        let (x1, y1) = pverts[th.1 as usize];
        
        let ctc = if (th.0 as usize) < state.projected_z.len()
                     && (th.1 as usize) < state.projected_z.len() {
            ((state.projected_z[th.0 as usize] as i32
            + state.projected_z[th.1 as usize] as i32) / 2) as i16
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
                hits[ci].push((bki, bkh, aqk, 255, ctc));
            }
            
            let (ref mut bmin, ref mut bmax) = bounds[ci];
            if *bmin < 0 || ak < *bmin { *bmin = ak; }
            if *bmax < 0 || ak > *bmax { *bmax = ak; }
        }
        p += avf * stride;
        o += bwa * stride;
        j += stride;
    }
}







#[inline]
pub fn fli(
    state: &Pa,
    col: usize,
    pixel_y: i32,
    beat_pulse: f32,
    energy: f32,
) -> RainEffect {
    if col >= state.column_hits.len() { return RainEffect::Bc; }

    
    let ccs = col as i32 * state.col_w + state.col_w / 2;
    let dx = ccs - state.center_x;
    let ad = pixel_y - state.center_y;
    let em = dsw(dx * dx + ad * ad);
    let cdh = (em - state.ripple_radius as i32).abs();
    let grt = 35i32;
    let cdi = if state.ripple_radius > 0.0 && state.ripple_radius < 550.0
                      && cdh < grt {
        let ln = ((grt - cdh) * 255 / grt) as u32;
        let life = ((550.0 - state.ripple_radius) / 550.0).max(0.0);
        (ln as f32 * life * 0.45).min(120.0) as u8
    } else { 0u8 };

    
    let hits = &state.column_hits[col];
    let mut diz: u8 = 0;
    let mut hhn: i16 = 0;
    for &(bki, bkh, _eidx, intensity, z_val) in hits {
        if pixel_y >= bki && pixel_y <= bkh {
            let center = (bki + bkh) / 2;
            let em = (pixel_y - center).abs();
            let cw = ((bkh - bki) / 2).max(1);
            let ln = ((cw - em) * intensity as i32 / cw).max(0) as u8;
            if ln > diz {
                diz = ln;
                hhn = z_val;
            }
        }
    }

    if diz > 0 {
        
        let cgk = (diz as u32 + (beat_pulse * 60.0) as u32).min(255) as u8;

        
        let ffs = (state.z_max - state.z_min).max(1) as i32;
        let jsg = ((state.z_max as i32 - hhn as i32) * 255 / ffs).max(0).min(255) as u8;

        
        
        let lyw = {
            let (bmin, bmax) = state.column_bounds[col];
            if bmin >= 0 && bmax > bmin {
                let cw = ((bmax - bmin) / 2).max(1);
                let center_y = (bmin + bmax) / 2;
                let bsw = (pixel_y - center_y).abs();
                
                let dxt = (bsw as f32 / cw as f32).min(1.0);
                (dxt * dxt * 255.0) as u8
            } else { 0u8 }
        };

        
        let oup = {
            let cqb = ccs - state.spec_x;
            let cqc = pixel_y - state.spec_y;
            let dyt = dsw(cqb * cqb + cqc * cqc);
            let cqv = 45i32;
            if dyt < cqv {
                let t = (cqv - dyt) as f32 / cqv as f32;
                (t * t * 255.0) as u8
            } else { 0u8 }
        };

        
        let pmz = (diz as u32 * 80 / 255).min(80) as u8;

        
        let jwm = {
            let (bmin, bmax) = state.column_bounds[col];
            if bmin >= 0 && bmax > bmin {
                let cw = ((bmax - bmin) / 2).max(1);
                let center_y = (bmin + bmax) / 2;
                let bsw = (pixel_y - center_y).abs();
                
                let fub = (bsw as f32 / cw as f32).min(1.0);
                
                let jzc = 1.0 - (jsg as f32 / 255.0); 
                ((fub * 0.4 + jzc * 0.3) * 120.0).min(120.0) as u8
            } else { 0u8 }
        };

        
        let kcu = if cgk > 100 {
            ((cgk as u32 - 100) * 200 / 155).min(200) as u8
        } else { 0u8 };

        
        let olh = 0u8; 

        
        let mqd = {
            let doa = ccs - state.center_x;
            let doe = pixel_y - state.shape_center_y;
            let byn = doa * doa + doe * doe;
            let radius = 80i32;
            let amn = radius * radius;
            if byn < amn {
                let t = 1.0 - (byn as f32 / amn as f32);
                (t * 160.0) as u8
            } else { 0u8 }
        };

        return RainEffect {
            glow: cgk,
            depth: jsg,
            trail_boost: pmz,
            ripple: cdi,
            dim: 0,
            fresnel: lyw,
            specular: oup,
            ao: jwm,
            bloom: kcu,
            scanline: olh,
            inner_glow: mqd,
            shadow: 0,
        };
    }

    
    let (bmin, bmax) = state.column_bounds[col];
    if bmin >= 0 && bmax > bmin && pixel_y >= bmin && pixel_y <= bmax {
        let pko = pixel_y - bmin;
        let pkk = bmax - pixel_y;
        let pkl = pko.min(pkk);
        let cw = (bmax - bmin) / 2;
        if cw <= 0 {
            return RainEffect { glow: 0, depth: 128, trail_boost: 0, ripple: cdi, dim: 0,
                fresnel: 0, specular: 0, ao: 0, bloom: 0, scanline: 0, inner_glow: 0, shadow: 0 };
        }
        
        let center_y = (bmin + bmax) / 2;
        let bsw = (pixel_y - center_y).abs();
        let dxt = (bsw as f32 / cw as f32).min(1.0);
        let lyx = (dxt * dxt * 180.0) as u8;

        
        let ouq = {
            let cqb = ccs - state.spec_x;
            let cqc = pixel_y - state.spec_y;
            let dyt = dsw(cqb * cqb + cqc * cqc);
            let cqv = 55i32; 
            if dyt < cqv {
                let t = (cqv - dyt) as f32 / cqv as f32;
                (t * t * 200.0) as u8
            } else { 0u8 }
        };

        let kaa = 15u32 + 25 * (cw - pkl).max(0) as u32 / cw as u32;
        let cgk = (kaa + (energy * 15.0) as u32 + (beat_pulse * 25.0) as u32).min(75) as u8;

        
        let jwn = {
            let fub = (bsw as f32 / cw as f32).min(1.0);
            (fub * 0.5 * 100.0).min(100.0) as u8
        };

        
        let kcv = if cgk > 50 {
            ((cgk as u32 - 50) * 80 / 25).min(80) as u8
        } else { 0u8 };

        
        let mqe = {
            let doa = ccs - state.center_x;
            let doe = pixel_y - state.shape_center_y;
            let byn = doa * doa + doe * doe;
            let radius = 100i32;
            let amn = radius * radius;
            if byn < amn {
                let t = 1.0 - (byn as f32 / amn as f32);
                (t * 200.0) as u8
            } else { 0u8 }
        };

        return RainEffect {
            glow: cgk,
            depth: 128,
            trail_boost: 20,
            ripple: cdi,
            dim: 0,
            fresnel: lyx,
            specular: ouq,
            ao: jwn,
            bloom: kcv,
            scanline: 0,
            inner_glow: mqe,
            shadow: 0,
        };
    }

    
    
    if bmin >= 0 && bmax > bmin {
        let oq = 60i32; 
        let fsl = if pixel_y < bmin {
            bmin - pixel_y
        } else if pixel_y > bmax {
            pixel_y - bmax
        } else { 0 };
        if fsl > 0 && fsl < oq {
            
            let leq = ((oq - fsl) * 160 / oq) as u8;
            return RainEffect {
                glow: 0, depth: 128, trail_boost: 0, ripple: cdi,
                dim: leq, fresnel: 0, specular: 0,
                ao: 0, bloom: 0, scanline: 0, inner_glow: 0, shadow: 0,
            };
        }
    }

    
    if state.shadow_y_start > 0 && pixel_y > state.shadow_y_start && pixel_y < state.shadow_y_end {
        
        let (bmin, bmax) = state.column_bounds[col];
        
        
        let mkd = bmin >= 0 || {
            let left = if col > 0 { state.column_bounds[col - 1].0 >= 0 } else { false };
            let right = if col + 1 < state.column_bounds.len() { state.column_bounds[col + 1].0 >= 0 } else { false };
            left || right
        };
        if mkd {
            let orh = state.shadow_y_end - state.shadow_y_start;
            let lzr = pixel_y - state.shadow_y_start;
            let jgc = 1.0 - (lzr as f32 / orh as f32);
            let jgd = (jgc * jgc * 140.0) as u8; 
            if jgd > 5 {
                return RainEffect {
                    glow: 0, depth: 128, trail_boost: 0, ripple: cdi,
                    dim: 0, fresnel: 0, specular: 0,
                    ao: 0, bloom: 0, scanline: 0, inner_glow: 0, shadow: jgd,
                };
            }
        }
    }

    
    if state.shadow_y_start > 0 && pixel_y > state.shadow_y_start && pixel_y < state.shadow_y_start + 200 {
        
        let kvc = state.center_x / state.col_w.max(1);
        let kve = (col as i32 - kvc).abs();
        if kve < 15 && (col % 4 == 0 || col % 4 == 1) {
            let orj = pixel_y - state.shadow_y_start;
            let ln = 1.0 - (orj as f32 / 200.0);
            let iyd = (ln * ln * 80.0 * energy) as u8;
            if iyd > 3 {
                return RainEffect {
                    glow: 0, depth: 128, trail_boost: 0, ripple: cdi,
                    dim: 0, fresnel: 0, specular: 0,
                    ao: 0, bloom: 0, scanline: iyd, inner_glow: 0, shadow: 0,
                };
            }
        }
    }

    
    if cdi > 0 {
        return RainEffect { glow: 0, depth: 128, trail_boost: 0, ripple: cdi, dim: 0,
            fresnel: 0, specular: 0, ao: 0, bloom: 0, scanline: 0, inner_glow: 0, shadow: 0 };
    }

    RainEffect::Bc
}








#[inline]
pub fn gia(
    adi: u8, agd: u8, apu: u8,
    glow: u8, depth: u8, ripple: u8,
    fresnel: u8, specular: u8,
    ao: u8, bloom: u8, scanline: u8, inner_glow: u8, shadow: u8,
    beat: f32, energy: f32,
) -> (u8, u8, u8) {
    let mut r = adi as f32;
    let mut g = agd as f32;
    let mut b = apu as f32;

    
    if shadow > 5 {
        let j = 1.0 - (shadow as f32 / 255.0) * 0.6;
        r *= j; g *= j; b *= j;
    }

    
    if scanline > 3 {
        let ob = scanline as f32 / 255.0;
        r = (r + ob * 15.0).min(255.0);
        g = (g + ob * 50.0).min(255.0);
        b = (b + ob * 25.0).min(255.0);
    }

    
    if ao > 0 && glow > 0 {
        let bep = 1.0 - (ao as f32 / 255.0) * 0.45;
        r *= bep;
        g *= bep;
        b *= bep;
    }

    if glow > 0 {
        let atx = glow as f32 / 255.0;

        if glow > 80 {
            
            let ldj = 0.4 + 0.6 * (depth as f32 / 255.0);
            let atx = atx * ldj;
            let no = beat * 0.2;
            let tr = (140.0 + no * 80.0 + energy * 60.0).min(255.0);
            let bwi = 255.0f32;
            let aiv = (190.0 + no * 40.0 + energy * 30.0).min(255.0);
            r = (r * (1.0 - atx) + tr * atx).min(255.0);
            g = (g * (1.0 - atx) + bwi * atx).min(255.0);
            b = (b * (1.0 - atx) + aiv * atx).min(255.0);
        } else {
            
            let ahj = atx * 2.5;
            r = (r * (1.0 + ahj)).min(255.0);
            g = (g * (1.0 + ahj)).min(255.0);
            b = (b * (1.0 + ahj)).min(255.0);
        }
    }

    
    if inner_glow > 20 {
        let axo = (inner_glow as f32 - 20.0) / 235.0;
        let axo = axo * axo; 
        
        r = (r + axo * 30.0).min(255.0);
        g = (g + axo * 70.0).min(255.0);
        b = (b + axo * 45.0).min(255.0);
    }

    
    if fresnel > 120 {
        let cxm = (fresnel as f32 - 120.0) / 135.0;
        r = (r + cxm * 80.0).min(255.0);
        g = (g + cxm * 120.0).min(255.0);
        b = (b + cxm * 100.0).min(255.0);
    }

    
    if specular > 30 {
        let azo = (specular as f32 - 30.0) / 225.0;
        let azo = azo * azo;
        r = (r + azo * 180.0).min(255.0);
        g = (g + azo * 200.0).min(255.0);
        b = (b + azo * 190.0).min(255.0);
    }

    
    if bloom > 10 {
        let bl = bloom as f32 / 255.0;
        
        r = (r + bl * 40.0).min(255.0);
        g = (g + bl * 55.0).min(255.0);
        b = (b + bl * 45.0).min(255.0);
    }

    
    if ripple > 0 {
        let rip = ripple as f32 / 255.0;
        r = (r + 20.0 * rip).min(255.0);
        g = (g + 35.0 * rip).min(255.0);
        b = (b + 30.0 * rip).min(255.0);
    }

    (r as u8, g as u8, b as u8)
}







#[inline]
pub fn hmz(state: &Pa, col: usize) -> u8 {
    if col >= state.column_bounds.len() { return 100; }
    let (bmin, bmax) = state.column_bounds[col];
    if bmin < 0 || bmax <= bmin { return 100; }
    
    let kyh = ((bmax - bmin) as u32).min(400);
    let aed = 100u32.saturating_sub(kyh * 55 / 400);
    aed.max(45) as u8
}
