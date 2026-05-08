






























pub const BK_: usize = 128;
pub const HI_: usize = 256;
pub const AHV_: usize = 6;


const OB_: usize = 96;
const AEK_: usize = 54;
const UQ_: usize = 5184; 


const CCX_: f32 = 300.0;   
const AGF_: f32 = 0.04;     





pub struct DroneSwarmState {
    pub initialized: bool,
    screen_w: f32,
    screen_h: f32,

    
    drone_x: [f32; BK_],
    drone_y: [f32; BK_],
    drone_z: [f32; BK_],

    
    target_x: [f32; BK_],
    target_y: [f32; BK_],
    target_z: [f32; BK_],

    
    proj_x: [f32; BK_],
    proj_y: [f32; BK_],
    proj_depth: [f32; BK_],

    num_drones: usize,

    
    edge_a: [u8; HI_],
    edge_b: [u8; HI_],
    num_edges: usize,

    
    scene_rot_y: f32,
    scene_rot_x: f32,

    
    formation_idx: u8,
    state_timer: f32,

    
    total_time: f32,

    
    glow: [f32; UQ_],

    
    cell_w: f32,
    cell_h: f32,

    frame: u64,
}

impl DroneSwarmState {
    pub const fn new() -> Self {
        Self {
            initialized: false,
            screen_w: 0.0,
            screen_h: 0.0,
            drone_x: [0.0; BK_],
            drone_y: [0.0; BK_],
            drone_z: [0.0; BK_],
            target_x: [0.0; BK_],
            target_y: [0.0; BK_],
            target_z: [0.0; BK_],
            proj_x: [0.0; BK_],
            proj_y: [0.0; BK_],
            proj_depth: [0.0; BK_],
            num_drones: 0,
            edge_a: [0; HI_],
            edge_b: [0; HI_],
            num_edges: 0,
            scene_rot_y: 0.0,
            scene_rot_x: 0.0,
            formation_idx: 0,
            state_timer: 0.0,
            total_time: 0.0,
            glow: [0.0; UQ_],
            cell_w: 20.0,
            cell_h: 20.0,
            frame: 0,
        }
    }
}





pub fn init(j: &mut DroneSwarmState, w: u32, h: u32) {
    j.screen_w = w as f32;
    j.screen_h = h as f32;
    j.cell_w = j.screen_w / OB_ as f32;
    j.cell_h = j.screen_h / AEK_ as f32;

    
    j.formation_idx = 0;
    hja(j, 0);

    
    for i in 0..j.num_drones {
        j.drone_x[i] = j.target_x[i];
        j.drone_y[i] = j.target_y[i];
        j.drone_z[i] = j.target_z[i];
    }

    j.initialized = true;
}





fn hja(j: &mut DroneSwarmState, idx: u8) {
    
    for i in 0..BK_ {
        j.target_x[i] = 0.0;
        j.target_y[i] = 0.0;
        j.target_z[i] = 0.0;
    }
    for i in 0..HI_ {
        j.edge_a[i] = 0;
        j.edge_b[i] = 0;
    }
    match idx % AHV_ as u8 {
        0 => kfa(j),
        1 => kfv(j),
        2 => kes(j),
        3 => kez(j),
        4 => kev(j),
        _ => kfc(j),
    }
}


fn kfa(j: &mut DroneSwarmState) {
    const GJ_: usize = 32;
    let mut ne = 0usize;

    let bac = 2.5f32;
    let radius = 0.35f32;
    let height = 1.6f32;

    for i in 0..GJ_ {
        let t = i as f32 / (GJ_ - 1) as f32;
        let cc = t * bac * 6.2831853;
        let y = -0.8 + t * height;

        
        j.target_x[i] = radius * libm::cosf(cc);
        j.target_y[i] = y;
        j.target_z[i] = radius * libm::sinf(cc);

        
        j.target_x[GJ_ + i] = radius * libm::cosf(cc + 3.1415927);
        j.target_y[GJ_ + i] = y;
        j.target_z[GJ_ + i] = radius * libm::sinf(cc + 3.1415927);

        
        if i > 0 {
            j.edge_a[ne] = (i - 1) as u8;
            j.edge_b[ne] = i as u8;
            ne += 1;
            j.edge_a[ne] = (GJ_ + i - 1) as u8;
            j.edge_b[ne] = (GJ_ + i) as u8;
            ne += 1;
        }
        
        if i % 4 == 0 {
            j.edge_a[ne] = i as u8;
            j.edge_b[ne] = (GJ_ + i) as u8;
            ne += 1;
        }
    }

    j.num_drones = GJ_ * 2;
    j.num_edges = ne;
}


fn kfv(j: &mut DroneSwarmState) {
    let scale = 0.55f32;

    for bits in 0u8..16 {
        let bxc = if bits & 1 != 0 { 1.0f32 } else { -1.0 };
        let pvz = if bits & 2 != 0 { 1.0f32 } else { -1.0 };
        let pwf = if bits & 4 != 0 { 1.0f32 } else { -1.0 };
        let pth = if bits & 8 != 0 { 1.0f32 } else { -1.0 };

        let cwi = pth * 0.4 + 1.8;
        j.target_x[bits as usize] = (bxc / cwi) * scale;
        j.target_y[bits as usize] = (pvz / cwi) * scale;
        j.target_z[bits as usize] = (pwf / cwi) * scale;
    }
    j.num_drones = 16;

    
    let mut ne = 0usize;
    for i in 0u8..16 {
        for bf in 0u8..4 {
            let ay = i ^ (1 << bf);
            if ay > i {
                j.edge_a[ne] = i;
                j.edge_b[ne] = ay;
                ne += 1;
            }
        }
    }
    j.num_edges = ne;
}


fn kes(j: &mut DroneSwarmState) {
    const BGP_: usize = 32;
    let mut tl = 0usize;
    let mut ne = 0usize;

    let jav = 0.7f32;

    for dq in 0u8..3 {
        let jmq = match dq {
            0 => 0.0f32,
            1 => 1.047f32,  
            _ => -1.047f32,  
        };

        let base = tl;
        let chs = libm::cosf(jmq);
        let cqr = libm::sinf(jmq);

        for i in 0..BGP_ {
            let a = (i as f32 / BGP_ as f32) * 6.2831853;
            let x = jav * libm::cosf(a);
            let y = jav * libm::sinf(a);

            
            j.target_x[tl] = x;
            j.target_y[tl] = y * chs;
            j.target_z[tl] = y * cqr;

            
            if i > 0 {
                j.edge_a[ne] = (tl - 1) as u8;
                j.edge_b[ne] = tl as u8;
                ne += 1;
            }
            tl += 1;
        }
        
        j.edge_a[ne] = (tl - 1) as u8;
        j.edge_b[ne] = base as u8;
        ne += 1;
    }

    
    let evk = 0.08f32;
    let irg = tl;
    let nlq: [(f32, f32, f32); 4] = [
        (evk, 0.0, 0.0),
        (-evk, 0.0, 0.0),
        (0.0, evk, 0.0),
        (0.0, -evk, 0.0),
    ];
    for &(nx, re, wi) in nlq.iter() {
        j.target_x[tl] = nx;
        j.target_y[tl] = re;
        j.target_z[tl] = wi;
        tl += 1;
    }
    
    for i in 0u8..4 {
        for ay in (i + 1)..4 {
            j.edge_a[ne] = (irg as u8) + i;
            j.edge_b[ne] = (irg as u8) + ay;
            ne += 1;
        }
    }

    j.num_drones = tl;
    j.num_edges = ne;
}


fn kez(j: &mut DroneSwarmState) {
    const AMP_: usize = 24;
    const BDV_: usize = 4;
    let mut tl = 0usize;
    let mut ne = 0usize;

    let adi = 0.1f32;
    let aug = 0.8f32;

    for arm in 0..BDV_ {
        let offset = (arm as f32 / BDV_ as f32) * 6.2831853;

        for i in 0..AMP_ {
            let t = i as f32 / (AMP_ - 1) as f32;
            let r = adi + t * (aug - adi);
            let acz = offset + t * 3.0;
            let pwj = libm::sinf(t * 4.0) * 0.1;

            j.target_x[tl] = r * libm::cosf(acz);
            j.target_y[tl] = pwj;
            j.target_z[tl] = r * libm::sinf(acz);

            if i > 0 {
                j.edge_a[ne] = (tl - 1) as u8;
                j.edge_b[ne] = tl as u8;
                ne += 1;
            }
            tl += 1;
        }
    }

    
    let hnw = tl;
    for i in 0..8usize {
        let a = (i as f32 / 8.0) * 6.2831853;
        j.target_x[tl] = 0.08 * libm::cosf(a);
        j.target_y[tl] = 0.08 * libm::sinf(a * 1.5);
        j.target_z[tl] = 0.08 * libm::sinf(a);
        tl += 1;
    }
    for i in 0u8..8 {
        j.edge_a[ne] = (hnw as u8) + i;
        j.edge_b[ne] = (hnw as u8) + ((i + 1) % 8);
        ne += 1;
    }

    j.num_drones = tl;
    j.num_edges = ne;
}


fn kev(j: &mut DroneSwarmState) {
    const Gu: usize = 4;
    const Kg: usize = 4;
    const Po: usize = 3;
    let mut tl = 0usize;
    let mut ne = 0usize;

    let spacing = 0.45f32;
    let dbz = -((Gu - 1) as f32) * spacing * 0.5;
    let nmh = -((Kg - 1) as f32) * spacing * 0.5;
    let nmi = -((Po - 1) as f32) * spacing * 0.5;

    for iz in 0..Po {
        for gg in 0..Kg {
            for bi in 0..Gu {
                j.target_x[tl] = dbz + bi as f32 * spacing;
                j.target_y[tl] = nmh + gg as f32 * spacing;
                j.target_z[tl] = nmi + iz as f32 * spacing;
                tl += 1;
            }
        }
    }

    
    for iz in 0..Po {
        for gg in 0..Kg {
            for bi in 0..Gu {
                let idx = iz * Kg * Gu + gg * Gu + bi;
                if bi + 1 < Gu && ne < HI_ {
                    j.edge_a[ne] = idx as u8;
                    j.edge_b[ne] = (idx + 1) as u8;
                    ne += 1;
                }
                if gg + 1 < Kg && ne < HI_ {
                    j.edge_a[ne] = idx as u8;
                    j.edge_b[ne] = (idx + Gu) as u8;
                    ne += 1;
                }
                if iz + 1 < Po && ne < HI_ {
                    j.edge_a[ne] = idx as u8;
                    j.edge_b[ne] = (idx + Kg * Gu) as u8;
                    ne += 1;
                }
            }
        }
    }

    j.num_drones = tl;
    j.num_edges = ne;
}


fn kfc(j: &mut DroneSwarmState) {
    const Ms: usize = 64;
    let r = 0.7f32;

    for i in 0..Ms {
        let t = (i as f32 / Ms as f32) * 6.2831853;
        
        j.target_x[i] = r * libm::sinf(t);
        j.target_y[i] = r * libm::sinf(t) * libm::cosf(t);
        j.target_z[i] = r * libm::sinf(2.0 * t) * 0.3;
    }

    let mut ne = 0usize;
    for i in 0..Ms {
        j.edge_a[ne] = i as u8;
        j.edge_b[ne] = ((i + 1) % Ms) as u8;
        ne += 1;
    }

    j.num_drones = Ms;
    j.num_edges = ne;
}





pub fn update(j: &mut DroneSwarmState) {
    if !j.initialized {
        return;
    }
    j.frame += 1;
    j.total_time += 1.0;
    j.state_timer += 1.0;

    
    j.scene_rot_y += 0.006; 
    j.scene_rot_x = libm::sinf(j.total_time * 0.0015) * 0.35; 

    
    if j.state_timer >= CCX_ {
        let nms = j.num_drones;
        j.formation_idx = (j.formation_idx + 1) % AHV_ as u8;
        hja(j, j.formation_idx);
        
        for i in nms..j.num_drones {
            j.drone_x[i] = 0.0;
            j.drone_y[i] = 0.0;
            j.drone_z[i] = 0.0;
        }
        j.state_timer = 0.0;
    }

    
    for i in 0..BK_ {
        j.drone_x[i] += (j.target_x[i] - j.drone_x[i]) * AGF_;
        j.drone_y[i] += (j.target_y[i] - j.drone_y[i]) * AGF_;
        j.drone_z[i] += (j.target_z[i] - j.drone_z[i]) * AGF_;
    }

    
    nyt(j);

    
    ofk(j);
}





fn nyt(j: &mut DroneSwarmState) {
    let ahs = libm::cosf(j.scene_rot_y);
    let air = libm::sinf(j.scene_rot_y);
    let ahr = libm::cosf(j.scene_rot_x);
    let aiq = libm::sinf(j.scene_rot_x);

    let cx = j.screen_w * 0.5;
    let u = j.screen_h * 0.45;
    let scale = j.screen_w * 0.28; 
    let cam_dist = 3.5f32;

    for i in 0..BK_ {
        let x = j.drone_x[i];
        let y = j.drone_y[i];
        let z = j.drone_z[i];

        
        let x2 = x * ahs + z * air;
        let qt = -x * air + z * ahs;

        
        let y2 = y * ahr - qt * aiq;
        let bxf = y * aiq + qt * ahr;

        
        let w = (cam_dist + bxf).max(0.15);

        j.proj_x[i] = cx + (x2 / w) * scale;
        j.proj_y[i] = u + (y2 / w) * scale;
        j.proj_depth[i] = w;
    }
}






#[inline]
fn hdy(glow: &mut [f32; UQ_], x: f32, y: f32, na: f32, radius: i32) {
    let bi = x as i32;
    let gg = y as i32;
    let ixn = radius as f32 + 0.5;

    let mut ad = -radius;
    while ad <= radius {
        let mut dx = -radius;
        while dx <= radius {
            let cx = bi + dx;
            let u = gg + ad;
            if cx >= 0 && (cx as usize) < OB_ && u >= 0 && (u as usize) < AEK_ {
                let dg = x - cx as f32 - 0.5;
                let hj = y - u as f32 - 0.5;
                let em = libm::sqrtf(dg * dg + hj * hj);
                if em < ixn {
                    let att = 1.0 - em / ixn;
                    let att = att * att; 
                    let idx = u as usize * OB_ + cx as usize;
                    glow[idx] += na * att;
                    if glow[idx] > 5.0 {
                        glow[idx] = 5.0;
                    }
                }
            }
            dx += 1;
        }
        ad += 1;
    }
}

fn ofk(j: &mut DroneSwarmState) {
    
    for i in 0..UQ_ {
        j.glow[i] = 0.0;
    }

    let aq = j.cell_w;
    let ch = j.cell_h;

    
    for e in 0..j.num_edges {
        let a = j.edge_a[e] as usize;
        let b = j.edge_b[e] as usize;

        let bm = j.proj_x[a] / aq;
        let az = j.proj_y[a] / ch;
        let x1 = j.proj_x[b] / aq;
        let y1 = j.proj_y[b] / ch;

        
        let jyu = (j.proj_depth[a] + j.proj_depth[b]) * 0.5;
        let frn = (1.8 / jyu).min(1.5);
        let lnz = frn * 0.7;

        
        let dx = x1 - bm;
        let ad = y1 - az;
        let len = libm::sqrtf(dx * dx + ad * ad);
        let steps = ((len * 1.5) as usize).max(1).min(300);

        for step in 0..=steps {
            let t = step as f32 / steps as f32;
            let x = bm + dx * t;
            let y = az + ad * t;
            hdy(&mut j.glow, x, y, lnz, 1);
        }
    }

    
    for i in 0..j.num_drones {
        let hc = j.proj_x[i] / aq;
        let jh = j.proj_y[i] / ch;
        let frn = (2.5 / j.proj_depth[i]).min(3.0);
        hdy(&mut j.glow, hc, jh, frn, 2);
    }
}






pub struct DroneInteraction {
    pub brightness: f32,
    pub color_r: i16,
    pub color_g: i16,
    pub color_b: i16,
}

impl DroneInteraction {
    pub const Bc: Self = Self {
        brightness: 1.0,
        color_r: 0,
        color_g: 0,
        color_b: 0,
    };
}



pub fn query(j: &DroneSwarmState, p: f32, o: f32) -> DroneInteraction {
    if !j.initialized {
        return DroneInteraction::Bc;
    }

    let hc = (p / j.cell_w) as usize;
    let jh = (o / j.cell_h) as usize;
    if hc >= OB_ || jh >= AEK_ {
        return DroneInteraction::Bc;
    }

    let val = j.glow[jh * OB_ + hc];
    if val < 0.03 {
        return DroneInteraction::Bc;
    }

    
    let (alg, ahp, cb) = kkg(j.total_time, j.formation_idx);

    
    let brightness = if val > 2.0 {
        
        1.4 + (val - 2.0).min(2.0) * 0.4
    } else if val > 0.5 {
        
        1.1 + (val - 0.5) * 0.25
    } else {
        
        1.0 + val * 0.2
    };

    
    let fnv = (val / 2.5).min(1.0);

    DroneInteraction {
        brightness,
        color_r: (alg as f32 * fnv) as i16,
        color_g: (ahp as f32 * fnv) as i16,
        color_b: (cb as f32 * fnv) as i16,
    }
}









fn kkg(time: f32, formation: u8) -> (i16, i16, i16) {
    
    let (yi, bg, mq) = match formation % AHV_ as u8 {
        0 => (0i16, 60, 80),      
        1 => (20i16, 30, 90),      
        2 => (70i16, 50, -10),     
        3 => (50i16, -5, 70),      
        4 => (-10i16, 40, 70),     
        _ => (60i16, 20, 50),      
    };

    
    let fav = libm::sinf(time * 0.018) * 0.2 + 1.0;
    
    let oru = libm::sinf(time * 0.011 + 1.5) * 0.1 + 1.0;

    let r = (yi as f32 * fav) as i16;
    let g = (bg as f32 * oru) as i16;
    let b = (mq as f32 * fav) as i16;

    (r, g, b)
}
