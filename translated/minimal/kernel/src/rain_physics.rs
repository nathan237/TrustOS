

































#[inline]
fn gtb(p: f32, o: f32, cx: f32, u: f32, r: f32) -> f32 {
    let dx = p - cx;
    let ad = o - u;
    libm::sqrtf(dx * dx + ad * ad) - r
}



#[inline]
fn dyr(p: f32, o: f32, ax: f32, aet: f32, bx: f32, dc: f32, r: f32) -> f32 {
    let iuc = p - ax;
    let iud = o - aet;
    let egn = bx - ax;
    let ego = dc - aet;
    let hgi = egn * egn + ego * ego;
    let h = if hgi > 0.001 {
        let t = (iuc * egn + iud * ego) / hgi;
        if t < 0.0 { 0.0 } else if t > 1.0 { 1.0 } else { t }
    } else {
        0.0
    };
    let dx = iuc - egn * h;
    let ad = iud - ego * h;
    libm::sqrtf(dx * dx + ad * ad) - r
}



#[inline]
fn cnl(a: f32, b: f32, k: f32) -> f32 {
    let iiu = k * 4.0;
    let d = iiu - libm::fabsf(a - b);
    let h = if d > 0.0 { d } else { 0.0 };
    let m = if a < b { a } else { b };
    m - h * h * 0.25 / iiu.max(0.001)
}





pub struct Km {
    pub initialized: bool,
    pub w: f32,
    pub h: f32,

    
    pub trunk_cx: f32,
    pub trunk_top_y: f32,
    pub trunk_bot_y: f32,
    pub trunk_r: f32,

    
    pub can1_cx: f32, pub can1_cy: f32, pub can1_r: f32,  
    pub can2_cx: f32, pub can2_cy: f32, pub can2_r: f32,  
    pub can3_cx: f32, pub can3_cy: f32, pub can3_r: f32,  

    
    pub br_l_ax: f32, pub br_l_ay: f32, pub br_l_bx: f32, pub br_l_by: f32, pub br_l_r: f32,
    pub br_r_ax: f32, pub br_r_ay: f32, pub br_r_bx: f32, pub br_r_by: f32, pub br_r_r: f32,

    
    pub rt_l_ax: f32, pub rt_l_ay: f32, pub rt_l_bx: f32, pub rt_l_by: f32, pub rt_l_r: f32,
    pub rt_r_ax: f32, pub rt_r_ay: f32, pub rt_r_bx: f32, pub rt_r_by: f32, pub rt_r_r: f32,

    
    pub lake_y: f32,
    pub lake_left: f32,
    pub lake_right: f32,

    
    pub tree_bb_left: f32,
    pub tree_bb_right: f32,
    pub tree_bb_top: f32,
    pub tree_bb_bot: f32,

    
    pub wind_phase: f32,
    pub frame: u64,
}

impl Km {
    pub const fn new() -> Self {
        Self {
            initialized: false,
            w: 0.0, h: 0.0,
            trunk_cx: 0.0, trunk_top_y: 0.0, trunk_bot_y: 0.0, trunk_r: 0.0,
            can1_cx: 0.0, can1_cy: 0.0, can1_r: 0.0,
            can2_cx: 0.0, can2_cy: 0.0, can2_r: 0.0,
            can3_cx: 0.0, can3_cy: 0.0, can3_r: 0.0,
            br_l_ax: 0.0, br_l_ay: 0.0, br_l_bx: 0.0, br_l_by: 0.0, br_l_r: 0.0,
            br_r_ax: 0.0, br_r_ay: 0.0, br_r_bx: 0.0, br_r_by: 0.0, br_r_r: 0.0,
            rt_l_ax: 0.0, rt_l_ay: 0.0, rt_l_bx: 0.0, rt_l_by: 0.0, rt_l_r: 0.0,
            rt_r_ax: 0.0, rt_r_ay: 0.0, rt_r_bx: 0.0, rt_r_by: 0.0, rt_r_r: 0.0,
            lake_y: 0.0, lake_left: 0.0, lake_right: 0.0,
            tree_bb_left: 0.0, tree_bb_right: 0.0, tree_bb_top: 0.0, tree_bb_bot: 0.0,
            wind_phase: 0.0, frame: 0,
        }
    }
}







pub fn qle(j: &mut Km, w: u32, h: u32) {
    let w = w as f32;
    let h = h as f32;
    j.w = w;
    j.h = h;

    
    
    j.trunk_cx   = w * 0.30;
    j.trunk_top_y = h * 0.48;      
    j.trunk_bot_y = h * 0.78;      
    j.trunk_r     = w * 0.010;     

    
    
    j.can1_cx = j.trunk_cx;
    j.can1_cy = h * 0.32;
    j.can1_r  = w * 0.080;         

    
    j.can2_cx = j.trunk_cx - w * 0.063;   
    j.can2_cy = h * 0.38;
    j.can2_r  = w * 0.050;                

    
    j.can3_cx = j.trunk_cx + w * 0.063;
    j.can3_cy = h * 0.38;
    j.can3_r  = w * 0.050;

    
    let hii = j.trunk_r * 0.6;
    
    j.br_l_ax = j.trunk_cx - hii;
    j.br_l_ay = j.trunk_top_y - h * 0.02;
    j.br_l_bx = j.can2_cx + j.can2_r * 0.3;
    j.br_l_by = j.can2_cy + j.can2_r * 0.4;
    j.br_l_r  = w * 0.005;                    
    
    j.br_r_ax = j.trunk_cx + hii;
    j.br_r_ay = j.trunk_top_y - h * 0.02;
    j.br_r_bx = j.can3_cx - j.can3_r * 0.3;
    j.br_r_by = j.can3_cy + j.can3_r * 0.4;
    j.br_r_r  = w * 0.005;

    
    j.rt_l_ax = j.trunk_cx - j.trunk_r * 0.3;
    j.rt_l_ay = j.trunk_bot_y;
    j.rt_l_bx = j.trunk_cx - w * 0.025;
    j.rt_l_by = j.trunk_bot_y + h * 0.025;
    j.rt_l_r  = w * 0.004;

    j.rt_r_ax = j.trunk_cx + j.trunk_r * 0.3;
    j.rt_r_ay = j.trunk_bot_y;
    j.rt_r_bx = j.trunk_cx + w * 0.025;
    j.rt_r_by = j.trunk_bot_y + h * 0.025;
    j.rt_r_r  = w * 0.004;

    
    j.lake_y     = h * 0.82;
    j.lake_left  = w * 0.08;
    j.lake_right = w * 0.92;

    
    let oq = 60.0;
    j.tree_bb_left  = (j.can2_cx - j.can2_r - oq).max(0.0);
    j.tree_bb_right = (j.can3_cx + j.can3_r + oq).min(w);
    j.tree_bb_top   = (j.can1_cy - j.can1_r - oq).max(0.0);
    j.tree_bb_bot   = (j.rt_l_by.max(j.rt_r_by) + oq).min(h);

    j.initialized = true;
}







fn ecp(p: f32, o: f32, j: &Km) -> f32 {
    
    let cff = libm::sinf(j.wind_phase) * j.can1_r * 0.04;
    let cfh = libm::sinf(j.wind_phase * 0.7 + 1.0) * j.can1_r * 0.015;

    
    let pnr = dyr(
        p, o,
        j.trunk_cx, j.trunk_bot_y,
        j.trunk_cx + cff * 0.1, j.trunk_top_y + cfh * 0.2,
        j.trunk_r,
    );

    
    let hw = gtb(p, o, j.can1_cx + cff, j.can1_cy + cfh, j.can1_r);
    let jf = gtb(p, o, j.can2_cx + cff * 0.8, j.can2_cy + cfh * 0.8, j.can2_r);
    let bfc = gtb(p, o, j.can3_cx + cff * 0.8, j.can3_cy + cfh * 0.8, j.can3_r);
    let khh = cnl(hw, cnl(jf, bfc, 25.0), 25.0);

    
    let bl = dyr(
        p, o,
        j.br_l_ax + cff * 0.1, j.br_l_ay + cfh * 0.2,
        j.br_l_bx + cff * 0.8, j.br_l_by + cfh * 0.8,
        j.br_l_r,
    );
    let yi = dyr(
        p, o,
        j.br_r_ax + cff * 0.1, j.br_r_ay + cfh * 0.2,
        j.br_r_bx + cff * 0.8, j.br_r_by + cfh * 0.8,
        j.br_r_r,
    );
    let fjs = cnl(bl, yi, 15.0);

    
    let ohn = dyr(p, o, j.rt_l_ax, j.rt_l_ay, j.rt_l_bx, j.rt_l_by, j.rt_l_r);
    let cpr = dyr(p, o, j.rt_r_ax, j.rt_r_ay, j.rt_r_bx, j.rt_r_by, j.rt_r_r);
    let oia = cnl(ohn, cpr, 10.0);

    
    let tree = cnl(pnr, khh, 18.0);
    let tree = cnl(tree, fjs, 12.0);
    cnl(tree, oia, 10.0)
}



fn pno(p: f32, o: f32, j: &Km) -> (f32, f32) {
    let eps = 3.0;
    let dx = ecp(p + eps, o, j) - ecp(p - eps, o, j);
    let ad = ecp(p, o + eps, j) - ecp(p, o - eps, j);
    let len = libm::sqrtf(dx * dx + ad * ad);
    if len > 0.001 {
        (dx / len, ad / len)
    } else {
        (0.0, -1.0)
    }
}







pub struct RainInteraction {
    
    
    pub x_offset: i32,
    
    pub brightness: f32,
    
    pub color_r: i16,
    pub color_g: i16,
    pub color_b: i16,
    
    pub on_surface: bool,
    
    pub in_lake: bool,
}

impl RainInteraction {
    pub const Bc: Self = Self {
        x_offset: 0,
        brightness: 1.0,
        color_r: 0, color_g: 0, color_b: 0,
        on_surface: false,
        in_lake: false,
    };
}


pub fn update(state: &mut Km) {
    if !state.initialized { return; }
    state.frame = state.frame.wrapping_add(1);
    
    state.wind_phase = (state.frame as f32) * 0.008;
}





pub fn qrn(j: &Km, p: f32, o: f32) -> RainInteraction {
    if !j.initialized {
        return RainInteraction::Bc;
    }

    
    let ipi = p >= j.tree_bb_left && p <= j.tree_bb_right
                 && o >= j.tree_bb_top  && o <= j.tree_bb_bot;

    
    let igb = o >= j.lake_y - 8.0
                    && p >= j.lake_left && p <= j.lake_right;

    if !ipi && !igb {
        return RainInteraction::Bc;
    }

    
    
    
    if ipi {
        let ces = ecp(p, o, j);

        const GK_: f32  = 15.0;   
        const AZB_: f32    = 8.0;    
        const BYO_: f32 = 8.0;    
        const Zv: f32     = 45.0;   

        if ces < Zv {
            
            if ces < -AZB_ {
                let depth = (-ces - AZB_).min(60.0);
                let dim = (depth / 60.0).min(0.80);  
                return RainInteraction {
                    x_offset: 0,
                    brightness: 0.20 + (1.0 - dim) * 0.3,  
                    color_r: -10,
                    color_g: 5,
                    color_b: -15,
                    on_surface: false,
                    in_lake: false,
                };
            }

            
            if libm::fabsf(ces) <= GK_ {
                let (nx, re) = pno(p, o, j);

                
                
                
                let bzu = nx * BYO_;

                
                let dfa = 1.0 - libm::fabsf(ces) / GK_;
                let na = 1.3 + dfa * 0.7;  

                
                let alg = (20.0 + dfa * 30.0) as i16;
                let ahp = (30.0 + dfa * 40.0) as i16;
                let cb = (25.0 + dfa * 30.0) as i16;

                return RainInteraction {
                    x_offset: bzu as i32,
                    brightness: na,
                    color_r: alg,
                    color_g: ahp,
                    color_b: cb,
                    on_surface: true,
                    in_lake: false,
                };
            }

            
            let emd = (ces - GK_) / (Zv - GK_);
            let emd = emd.max(0.0).min(1.0);

            
            
            let khi = j.can1_cy + j.can1_r * 0.6;
            let jtc = o < j.trunk_top_y + 30.0;
            if o > khi && jtc && ces < 30.0 {
                let ekt = 1.0 - emd;
                return RainInteraction {
                    x_offset: 0,
                    brightness: 1.1 + ekt * 0.3,
                    color_r: (8.0 * ekt) as i16,
                    color_g: (15.0 * ekt) as i16,
                    color_b: (10.0 * ekt) as i16,
                    on_surface: false,
                    in_lake: false,
                };
            }

            
            let kbh = o > j.can1_cy + j.can1_r * 0.3;
            if kbh && ces < 35.0 {
                let shadow = 0.12 * (1.0 - emd);
                return RainInteraction {
                    x_offset: 0,
                    brightness: 1.0 - shadow,
                    color_r: 0,
                    color_g: 0,
                    color_b: 0,
                    on_surface: false,
                    in_lake: false,
                };
            }
        }
    }

    
    
    
    if igb {
        let esd = o - j.lake_y;  

        const GK_: f32 = 6.0;  

        
        if libm::fabsf(esd) < GK_ {
            let dfa = 1.0 - libm::fabsf(esd) / GK_;
            
            let fav = libm::sinf(p * 0.05 + j.wind_phase * 3.0) * 0.3 + 0.7;
            let na = 1.4 + dfa * fav * 0.6;
            return RainInteraction {
                x_offset: 0,
                brightness: na,
                color_r: -10,
                color_g: 10,
                color_b: 50,   
                on_surface: false,
                in_lake: true,
            };
        }

        
        if esd > GK_ {
            let depth = (esd - GK_).min(120.0) / 120.0;
            let dim = 0.12 + depth * 0.55;  
            return RainInteraction {
                x_offset: 0,
                brightness: 1.0 - dim,
                color_r: -25,
                color_g: -8,
                color_b: 25,   
                on_surface: false,
                in_lake: true,
            };
        }
    }

    RainInteraction::Bc
}
