

























use alloc::vec::Vec;
use spin::Mutex;
use micromath::F32Ext;

use super::math3d::{Vec3, Vec4, Mat4, fre};
use super::render2d::Color2D;
use super::texture;
use crate::framebuffer;






pub const CBE_: u32 = 0x0000;
pub const UU_: u32 = 0x0001;
pub const CBA_: u32 = 0x0003;
pub const OD_: u32 = 0x0002;
pub const AEQ_: u32 = 0x0004;
pub const DRU_: u32 = 0x0005;
pub const AVS_: u32 = 0x0006;
pub const KZ_: u32 = 0x0007;


pub const AEM_: u32 = 0x4000;
pub const AEN_: u32 = 0x0100;


pub const AEO_: u32 = 0x1700;
pub const OE_: u32 = 0x1701;


pub const UR_: u32 = 0x0B71;
pub const AVK_: u32 = 0x0B50;
pub const UT_: u32 = 0x4000;
pub const DRN_: u32 = 0x4001;
pub const AVI_: u32 = 0x0B44;
pub const AEL_: u32 = 0x0BE2;


pub const DRI_: u32 = 0x1200;
pub const DRK_: u32 = 0x1201;
pub const DRS_: u32 = 0x1203;


pub const DRL_: u32 = 0x1D00;
pub const UV_: u32 = 0x1D01;


pub const CAY_: u32 = 0x0404;
pub const DRJ_: u32 = 0x0405;
pub const CAZ_: u32 = 0x0408;
pub const AVL_: u32 = 0x1B01;
pub const CAX_: u32 = 0x1B02;


pub const AVO_: u32 = 0;
pub const AVJ_: u32 = 0x0500;
pub const DRM_: u32 = 0x0501;
pub const US_: u32 = 0x0502;






#[derive(Clone, Copy, Default)]
struct Td {
    position: Vec3,
    color: Color2D,
    normal: Vec3,
    texcoord: (f32, f32),
}


struct GlState {
    
    viewport_x: i32,
    viewport_y: i32,
    viewport_width: u32,
    viewport_height: u32,

    
    modelview_stack: Vec<Mat4>,
    projection_stack: Vec<Mat4>,
    current_matrix_mode: u32,

    
    current_color: Color2D,
    current_normal: Vec3,
    current_texcoord: (f32, f32),
    clear_color: Color2D,

    
    primitive_type: u32,
    vertices: Vec<Td>,
    in_begin_end: bool,

    
    depth_test_enabled: bool,
    lighting_enabled: bool,
    cull_face_enabled: bool,
    blend_enabled: bool,
    lights_enabled: [bool; 8],

    
    light_positions: [Vec4; 8],
    light_ambient: [Color2D; 8],
    light_diffuse: [Color2D; 8],

    
    depth_buffer: Vec<f32>,

    
    shade_model: u32,

    
    polygon_mode: u32,

    
    last_error: u32,

    
    initialized: bool,
}

impl GlState {
    const fn new() -> Self {
        Self {
            viewport_x: 0,
            viewport_y: 0,
            viewport_width: 800,
            viewport_height: 600,
            modelview_stack: Vec::new(),
            projection_stack: Vec::new(),
            current_matrix_mode: AEO_,
            current_color: Color2D::WHITE,
            current_normal: Vec3::Ash,
            current_texcoord: (0.0, 0.0),
            clear_color: Color2D::BLACK,
            primitive_type: AEQ_,
            vertices: Vec::new(),
            in_begin_end: false,
            depth_test_enabled: false,
            lighting_enabled: false,
            cull_face_enabled: false,
            blend_enabled: false,
            lights_enabled: [false; 8],
            light_positions: [Vec4::new(0.0, 0.0, 1.0, 0.0); 8],
            light_ambient: [Color2D::BLACK; 8],
            light_diffuse: [Color2D::WHITE; 8],
            depth_buffer: Vec::new(),
            shade_model: UV_,
            polygon_mode: CAX_,
            last_error: AVO_,
            initialized: false,
        }
    }

    fn current_matrix(&mut self) -> &mut Mat4 {
        let dn = match self.current_matrix_mode {
            OE_ => &mut self.projection_stack,
            _ => &mut self.modelview_stack,
        };
        if dn.is_empty() {
            dn.push(Mat4::Ie);
        }
        
        dn.last_mut().unwrap_or_else(|| unreachable!())
    }

    fn get_mvp(&self) -> Mat4 {
        let mv = self.modelview_stack.last().cloned().unwrap_or(Mat4::Ie);
        let oa = self.projection_stack.last().cloned().unwrap_or(Mat4::Ie);
        oa.mul(&mv)
    }
}

static AS_: Mutex<GlState> = Mutex::new(GlState::new());






pub fn ice(width: u32, height: u32) {
    let mut state = AS_.lock();
    state.viewport_width = width;
    state.viewport_height = height;
    
    let size = (width as usize) * (height as usize);
    let mut depth = alloc::vec::Vec::new();
    if depth.try_reserve_exact(size).is_ok() {
        depth.resize(size, 1.0f32);
        state.depth_buffer = depth;
    } else {
        crate::serial_println!("[GL] WARNING: Failed to allocate depth buffer {} KB — OOM",
            size * 4 / 1024);
    }
    state.modelview_stack.push(Mat4::Ie);
    state.projection_stack.push(Mat4::Ie);
    state.initialized = true;
}






pub fn mew(x: i32, y: i32, width: u32, height: u32) {
    let mut state = AS_.lock();
    state.viewport_x = x;
    state.viewport_y = y;
    state.viewport_width = width;
    state.viewport_height = height;
    
    let size = (width as usize) * (height as usize);
    let mut depth = alloc::vec::Vec::new();
    if depth.try_reserve_exact(size).is_ok() {
        depth.resize(size, 1.0f32);
        state.depth_buffer = depth;
    }
}


pub fn eoi(mode: u32) {
    let mut state = AS_.lock();
    state.current_matrix_mode = mode;
}


pub fn dqu() {
    let mut state = AS_.lock();
    *state.current_matrix() = Mat4::Ie;
}


pub fn icg() {
    let mut state = AS_.lock();
    let current = *state.current_matrix();
    match state.current_matrix_mode {
        OE_ => state.projection_stack.push(current),
        _ => state.modelview_stack.push(current),
    }
}


pub fn icf() {
    let mut state = AS_.lock();
    match state.current_matrix_mode {
        OE_ => { 
            if state.projection_stack.len() > 1 {
                state.projection_stack.pop();
            }
        },
        _ => {
            if state.modelview_stack.len() > 1 {
                state.modelview_stack.pop();
            }
        },
    }
}


pub fn mev(x: f32, y: f32, z: f32) {
    let mut state = AS_.lock();
    let gzz = Mat4::gzz(x, y, z);
    let current = *state.current_matrix();
    *state.current_matrix() = current.mul(&gzz);
}


pub fn eoj(cc: f32, x: f32, y: f32, z: f32) {
    let mut state = AS_.lock();
    let ctt = Vec3::new(x, y, z).normalize();
    let abf = fre(cc);
    let rotation = Mat4::rotation(ctt, abf);
    let current = *state.current_matrix();
    *state.current_matrix() = current.mul(&rotation);
}


pub fn qjk(x: f32, y: f32, z: f32) {
    let mut state = AS_.lock();
    let scale = Mat4::scale(x, y, z);
    let current = *state.current_matrix();
    *state.current_matrix() = current.mul(&scale);
}


pub fn qji(m: &[f32; 16]) {
    let mut state = AS_.lock();
    let ggo = Mat4::lza(*m);
    let current = *state.current_matrix();
    *state.current_matrix() = current.mul(&ggo);
}


pub fn met(left: f32, right: f32, bottom: f32, top: f32, near: f32, far: f32) {
    let mut state = AS_.lock();
    let nny = Mat4::nnz(left, right, bottom, top, near, far);
    let current = *state.current_matrix();
    *state.current_matrix() = current.mul(&nny);
}


pub fn qjf(left: f32, right: f32, bottom: f32, top: f32, near: f32, far: f32) {
    let mut state = AS_.lock();
    let fxv = Mat4::fxv(left, right, bottom, top, near, far);
    let current = *state.current_matrix();
    *state.current_matrix() = current.mul(&fxv);
}






pub fn mfc(fovy: f32, bqh: f32, near: f32, far: f32) {
    let mut state = AS_.lock();
    let vq = Mat4::vq(fre(fovy), bqh, near, far);
    let current = *state.current_matrix();
    *state.current_matrix() = current.mul(&vq);
}


pub fn icj(
    cje: f32, atr: f32, bmc: f32,
    center_x: f32, center_y: f32, center_z: f32,
    up_x: f32, up_y: f32, up_z: f32
) {
    let mut state = AS_.lock();
    let ggh = Mat4::ggh(
        Vec3::new(cje, atr, bmc),
        Vec3::new(center_x, center_y, center_z),
        Vec3::new(up_x, up_y, up_z),
    );
    let current = *state.current_matrix();
    *state.current_matrix() = current.mul(&ggh);
}






pub fn icd(r: f32, g: f32, b: f32, _a: f32) {
    let mut state = AS_.lock();
    state.clear_color = Color2D::rgb(
        (r * 255.0) as u8,
        (g * 255.0) as u8,
        (b * 255.0) as u8,
    );
}


pub fn icc(mask: u32) {
    let mut state = AS_.lock();
    
    if (mask & AEM_) != 0 {
        let color = state.clear_color.to_u32();
        let w = state.viewport_width;
        let h = state.viewport_height;
        framebuffer::fill_rect(
            state.viewport_x as u32, 
            state.viewport_y as u32, 
            w, h, color
        );
    }
    
    if (mask & AEN_) != 0 {
        state.depth_buffer.fill(1.0);
    }
}


pub fn fzc(cap: u32) {
    let mut state = AS_.lock();
    match cap {
        UR_ => state.depth_test_enabled = true,
        AVK_ => state.lighting_enabled = true,
        AVI_ => state.cull_face_enabled = true,
        AEL_ => state.blend_enabled = true,
        UT_..=0x4007 => {
            let idx = (cap - UT_) as usize;
            if idx < 8 {
                state.lights_enabled[idx] = true;
            }
        }
        _ => state.last_error = AVJ_,
    }
}


pub fn qje(cap: u32) {
    let mut state = AS_.lock();
    match cap {
        UR_ => state.depth_test_enabled = false,
        AVK_ => state.lighting_enabled = false,
        AVI_ => state.cull_face_enabled = false,
        AEL_ => state.blend_enabled = false,
        UT_..=0x4007 => {
            let idx = (cap - UT_) as usize;
            if idx < 8 {
                state.lights_enabled[idx] = false;
            }
        }
        _ => state.last_error = AVJ_,
    }
}


pub fn qjl(mode: u32) {
    let mut state = AS_.lock();
    state.shade_model = mode;
}


pub fn qjj(face: u32, mode: u32) {
    let mut state = AS_.lock();
    if face == CAZ_ || face == CAY_ {
        state.polygon_mode = mode;
    }
}






pub fn aqw(mode: u32) {
    let mut state = AS_.lock();
    if state.in_begin_end {
        state.last_error = US_;
        return;
    }
    state.primitive_type = mode;
    state.vertices.clear();
    state.in_begin_end = true;
}


pub fn aqx() {
    let mut state = AS_.lock();
    if !state.in_begin_end {
        state.last_error = US_;
        return;
    }
    state.in_begin_end = false;

    
    let nhi = state.get_mvp();
    let vp_x = state.viewport_x;
    let vp_y = state.viewport_y;
    let vp_w = state.viewport_width as f32;
    let vp_h = state.viewport_height as f32;
    let bys = state.depth_test_enabled;
    let myj = state.lighting_enabled;
    let polygon_mode = state.polygon_mode;
    let shade_model = state.shade_model;
    
    
    let vertices = core::mem::take(&mut state.vertices);
    let nxb = state.primitive_type;
    
    
    
    let eea = state.viewport_width;
    let depth_buffer = &mut state.depth_buffer as *mut Vec<f32>;
    
    
    let fcx = texture::mtw();
    
    
    let mut aaf: Vec<Option<(i32, i32, f32, Color2D, f32, f32)>> = Vec::with_capacity(vertices.len());
    for v in &vertices {
        let chb = nhi.transform_vec4(Vec4::iae(v.position, 1.0));
        if chb.w > 0.0 {
            let nht = chb.x / chb.w;
            let nhu = chb.y / chb.w;
            let nhv = chb.z / chb.w;
            
            let lw = vp_x + ((nht + 1.0) * 0.5 * vp_w) as i32;
            let nn = vp_y + ((1.0 - nhu) * 0.5 * vp_h) as i32;
            
            let mut color = v.color;
            
            
            if myj {
                let axv = Vec3::new(0.5, -1.0, -0.5).normalize();
                let dux = v.normal.dot(-axv).max(0.0);
                let ambient = 0.2;
                let intensity = ambient + (1.0 - ambient) * dux;
                color = Color2D::rgb(
                    (color.r as f32 * intensity) as u8,
                    (color.g as f32 * intensity) as u8,
                    (color.b as f32 * intensity) as u8,
                );
            }
            
            aaf.push(Some((lw, nn, nhv, color, v.texcoord.0, v.texcoord.1)));
        } else {
            aaf.push(None);
        }
    }

    
    match nxb {
        CBE_ => {
            for aa in &aaf {
                if let Some((x, y, z, color, iy, v)) = aa {
                    
                    let dpo = if fcx {
                        if let Some(tex_color) = texture::jcj(*iy, *v) {
                            tex_color
                        } else {
                            color.to_u32()
                        }
                    } else {
                        color.to_u32()
                    };
                    
                    if *x >= 0 && *y >= 0 && (*x as u32) < eea && (*y as u32) < (vp_h as u32) {
                        let idx = (*y as usize) * (eea as usize) + (*x as usize);
                        unsafe {
                            let fu = &mut *depth_buffer;
                            if !bys || *z < fu[idx] {
                                fu[idx] = *z;
                                framebuffer::put_pixel(*x as u32, *y as u32, dpo);
                            }
                        }
                    }
                }
            }
        }
        
        UU_ => {
            for i in (0..aaf.len()).step_by(2) {
                if i + 1 < aaf.len() {
                    if let (Some((bm, az, _, og, _, _)), Some((x1, y1, _, _, _, _))) = 
                        (&aaf[i], &aaf[i + 1]) 
                    {
                        draw_line(*bm, *az, *x1, *y1, og.to_u32());
                    }
                }
            }
        }
        
        CBA_ => {
            for i in 0..aaf.len().saturating_sub(1) {
                if let (Some((bm, az, _, og, _, _)), Some((x1, y1, _, _, _, _))) = 
                    (&aaf[i], &aaf[i + 1]) 
                {
                    draw_line(*bm, *az, *x1, *y1, og.to_u32());
                }
            }
        }
        
        OD_ => {
            for i in 0..aaf.len() {
                let next = (i + 1) % aaf.len();
                if let (Some((bm, az, _, og, _, _)), Some((x1, y1, _, _, _, _))) = 
                    (&aaf[i], &aaf[next]) 
                {
                    draw_line(*bm, *az, *x1, *y1, og.to_u32());
                }
            }
        }
        
        AEQ_ => {
            for i in (0..aaf.len()).step_by(3) {
                if i + 2 < aaf.len() {
                    if let (Some(qm), Some(gw), Some(gn)) = 
                        (&aaf[i], &aaf[i + 1], &aaf[i + 2]) 
                    {
                        if polygon_mode == AVL_ {
                            
                            draw_line(qm.0, qm.1, gw.0, gw.1, qm.3.to_u32());
                            draw_line(gw.0, gw.1, gn.0, gn.1, gw.3.to_u32());
                            draw_line(gn.0, gn.1, qm.0, qm.1, gn.3.to_u32());
                        } else {
                            
                            unsafe {
                                let fu = &mut *depth_buffer;
                                fsw(
                                    qm.0, qm.1, qm.2, qm.3, qm.4, qm.5,
                                    gw.0, gw.1, gw.2, gw.3, gw.4, gw.5,
                                    gn.0, gn.1, gn.2, gn.3, gn.4, gn.5,
                                    eea,
                                    bys,
                                    shade_model == UV_,
                                    fcx,
                                    fu,
                                );
                            }
                        }
                    }
                }
            }
        }
        
        KZ_ => {
            
            for i in (0..aaf.len()).step_by(4) {
                if i + 3 < aaf.len() {
                    if let (Some(qm), Some(gw), Some(gn), Some(aih)) = 
                        (&aaf[i], &aaf[i + 1], &aaf[i + 2], &aaf[i + 3]) 
                    {
                        if polygon_mode == AVL_ {
                            draw_line(qm.0, qm.1, gw.0, gw.1, qm.3.to_u32());
                            draw_line(gw.0, gw.1, gn.0, gn.1, gw.3.to_u32());
                            draw_line(gn.0, gn.1, aih.0, aih.1, gn.3.to_u32());
                            draw_line(aih.0, aih.1, qm.0, qm.1, aih.3.to_u32());
                        } else {
                            unsafe {
                                let fu = &mut *depth_buffer;
                                
                                fsw(
                                    qm.0, qm.1, qm.2, qm.3, qm.4, qm.5,
                                    gw.0, gw.1, gw.2, gw.3, gw.4, gw.5,
                                    gn.0, gn.1, gn.2, gn.3, gn.4, gn.5,
                                    eea,
                                    bys,
                                    shade_model == UV_,
                                    fcx,
                                    fu,
                                );
                                
                                fsw(
                                    qm.0, qm.1, qm.2, qm.3, qm.4, qm.5,
                                    gn.0, gn.1, gn.2, gn.3, gn.4, gn.5,
                                    aih.0, aih.1, aih.2, aih.3, aih.4, aih.5,
                                    eea,
                                    bys,
                                    shade_model == UV_,
                                    fcx,
                                    fu,
                                );
                            }
                        }
                    }
                }
            }
        }
        
        _ => {}
    }
}


pub fn bmm(r: f32, g: f32, b: f32) {
    let mut state = AS_.lock();
    state.current_color = Color2D::rgb(
        (r.clamp(0.0, 1.0) * 255.0) as u8,
        (g.clamp(0.0, 1.0) * 255.0) as u8,
        (b.clamp(0.0, 1.0) * 255.0) as u8,
    );
}


pub fn qjc(r: u8, g: u8, b: u8) {
    let mut state = AS_.lock();
    state.current_color = Color2D::rgb(r, g, b);
}


pub fn cae(r: f32, g: f32, b: f32, a: f32) {
    let mut state = AS_.lock();
    state.current_color = Color2D::bdl(
        (r.clamp(0.0, 1.0) * 255.0) as u8,
        (g.clamp(0.0, 1.0) * 255.0) as u8,
        (b.clamp(0.0, 1.0) * 255.0) as u8,
        (a.clamp(0.0, 1.0) * 255.0) as u8,
    );
}


pub fn ahz(x: f32, y: f32, z: f32) {
    let mut state = AS_.lock();
    state.current_normal = Vec3::new(x, y, z).normalize();
}


pub fn aay(j: f32, t: f32) {
    let mut state = AS_.lock();
    state.current_texcoord = (j, t);
}


pub fn dt(x: f32, y: f32, z: f32) {
    let mut state = AS_.lock();
    if !state.in_begin_end {
        state.last_error = US_;
        return;
    }
    
    
    let color = state.current_color;
    let normal = state.current_normal;
    let texcoord = state.current_texcoord;
    state.vertices.push(Td {
        position: Vec3::new(x, y, z),
        color,
        normal,
        texcoord,
    });
}


#[inline]
pub fn qjn(x: f32, y: f32) {
    dt(x, y, 0.0);
}



pub fn qjo(verts: &[(f32, f32, f32)]) {
    let mut state = AS_.lock();
    if !state.in_begin_end {
        state.last_error = US_;
        return;
    }
    let color = state.current_color;
    let normal = state.current_normal;
    let texcoord = state.current_texcoord;
    state.vertices.reserve(verts.len());
    for &(x, y, z) in verts {
        state.vertices.push(Td {
            position: Vec3::new(x, y, z),
            color,
            normal,
            texcoord,
        });
    }
}


pub fn mer() {
    
}


pub fn qjm() {
    framebuffer::ii();
}


pub fn qjh() -> u32 {
    let mut state = AS_.lock();
    let err = state.last_error;
    state.last_error = AVO_;
    err
}





fn draw_line(bm: i32, az: i32, x1: i32, y1: i32, color: u32) {
    let dx = (x1 - bm).abs();
    let ad = -(y1 - az).abs();
    let am = if bm < x1 { 1 } else { -1 };
    let ak = if az < y1 { 1 } else { -1 };
    let mut err = dx + ad;
    let mut x = bm;
    let mut y = az;

    loop {
        if x >= 0 && y >= 0 {
            framebuffer::put_pixel(x as u32, y as u32, color);
        }
        if x == x1 && y == y1 { break; }
        let pg = 2 * err;
        if pg >= ad { err += ad; x += am; }
        if pg <= dx { err += dx; y += ak; }
    }
}

fn qdp(
    bm: i32, az: i32, z0: f32, og: Color2D,
    x1: i32, y1: i32, po: f32, hw: Color2D,
    x2: i32, y2: i32, qt: f32, jf: Color2D,
    width: u32,
    bys: bool,
    smooth_shading: bool,
    depth_buffer: &mut Vec<f32>,
) {
    
    let mut verts = [(bm, az, z0, og), (x1, y1, po, hw), (x2, y2, qt, jf)];
    verts.sort_by_key(|v| v.1);
    
    let (bm, az, z0, og) = verts[0];
    let (x1, y1, po, hw) = verts[1];
    let (x2, y2, qt, jf) = verts[2];
    
    if az == y2 { return; } 
    
    
    let emx = og;
    
    
    for y in az.max(0)..=y2 {
        if y < 0 { continue; }
        
        let (aja, bkl, apz, bkd, zb, cb) = if y < y1 {
            
            if y1 == az {
                let t = if y2 != az { (y - az) as f32 / (y2 - az) as f32 } else { 0.0 };
                let aja = bm + ((x2 - bm) as f32 * t) as i32;
                let bkl = z0 + (qt - z0) * t;
                (aja, bkl, aom(og, jf, t), aja, bkl, aom(og, jf, t))
            } else {
                let ll = (y - az) as f32 / (y1 - az) as f32;
                let np = (y - az) as f32 / (y2 - az) as f32;
                (
                    bm + ((x1 - bm) as f32 * ll) as i32,
                    z0 + (po - z0) * ll,
                    aom(og, hw, ll),
                    bm + ((x2 - bm) as f32 * np) as i32,
                    z0 + (qt - z0) * np,
                    aom(og, jf, np),
                )
            }
        } else {
            
            if y2 == y1 {
                (x1, po, hw, x2, qt, jf)
            } else {
                let ll = (y - y1) as f32 / (y2 - y1) as f32;
                let np = (y - az) as f32 / (y2 - az) as f32;
                (
                    x1 + ((x2 - x1) as f32 * ll) as i32,
                    po + (qt - po) * ll,
                    aom(hw, jf, ll),
                    bm + ((x2 - bm) as f32 * np) as i32,
                    z0 + (qt - z0) * np,
                    aom(og, jf, np),
                )
            }
        };
        
        let (start_x, awy, start_z, end_z, start_c, end_c) = if aja < bkd {
            (aja, bkd, bkl, zb, apz, cb)
        } else {
            (bkd, aja, zb, bkl, cb, apz)
        };
        
        for x in start_x.max(0)..=awy {
            if x < 0 || x as u32 >= width { continue; }
            
            let t = if awy != start_x {
                (x - start_x) as f32 / (awy - start_x) as f32
            } else {
                0.0
            };
            
            let z = start_z + (end_z - start_z) * t;
            let color = if smooth_shading {
                aom(start_c, end_c, t)
            } else {
                emx
            };
            
            let idx = (y as usize) * (width as usize) + (x as usize);
            if idx < depth_buffer.len() {
                if !bys || z < depth_buffer[idx] {
                    depth_buffer[idx] = z;
                    framebuffer::put_pixel(x as u32, y as u32, color.to_u32());
                }
            }
        }
    }
}

fn aom(og: Color2D, hw: Color2D, t: f32) -> Color2D {
    Color2D::rgb(
        (og.r as f32 + (hw.r as f32 - og.r as f32) * t) as u8,
        (og.g as f32 + (hw.g as f32 - og.g as f32) * t) as u8,
        (og.b as f32 + (hw.b as f32 - og.b as f32) * t) as u8,
    )
}


fn fsw(
    bm: i32, az: i32, z0: f32, og: Color2D, u0: f32, v0: f32,
    x1: i32, y1: i32, po: f32, hw: Color2D, u1: f32, v1: f32,
    x2: i32, y2: i32, qt: f32, jf: Color2D, u2: f32, v2: f32,
    width: u32,
    bys: bool,
    smooth_shading: bool,
    texturing: bool,
    depth_buffer: &mut Vec<f32>,
) {
    
    let mut verts = [
        (bm, az, z0, og, u0, v0), 
        (x1, y1, po, hw, u1, v1), 
        (x2, y2, qt, jf, u2, v2)
    ];
    verts.sort_by_key(|v| v.1);
    
    let (bm, az, z0, og, u0, v0) = verts[0];
    let (x1, y1, po, hw, u1, v1) = verts[1];
    let (x2, y2, qt, jf, u2, v2) = verts[2];
    
    if az == y2 { return; }
    
    let emx = og;
    
    for y in az.max(0)..=y2 {
        if y < 0 { continue; }
        
        
        let (aja, bkl, apz, edc, va, bkd, zb, cb, ub, apk) = if y < y1 {
            if y1 == az {
                let t = if y2 != az { (y - az) as f32 / (y2 - az) as f32 } else { 0.0 };
                let aja = bm + ((x2 - bm) as f32 * t) as i32;
                let bkl = z0 + (qt - z0) * t;
                let edc = u0 + (u2 - u0) * t;
                let va = v0 + (v2 - v0) * t;
                (aja, bkl, aom(og, jf, t), edc, va, aja, bkl, aom(og, jf, t), edc, va)
            } else {
                let ll = (y - az) as f32 / (y1 - az) as f32;
                let np = (y - az) as f32 / (y2 - az) as f32;
                (
                    bm + ((x1 - bm) as f32 * ll) as i32,
                    z0 + (po - z0) * ll,
                    aom(og, hw, ll),
                    u0 + (u1 - u0) * ll,
                    v0 + (v1 - v0) * ll,
                    bm + ((x2 - bm) as f32 * np) as i32,
                    z0 + (qt - z0) * np,
                    aom(og, jf, np),
                    u0 + (u2 - u0) * np,
                    v0 + (v2 - v0) * np,
                )
            }
        } else {
            if y2 == y1 {
                (x1, po, hw, u1, v1, x2, qt, jf, u2, v2)
            } else {
                let ll = (y - y1) as f32 / (y2 - y1) as f32;
                let np = (y - az) as f32 / (y2 - az) as f32;
                (
                    x1 + ((x2 - x1) as f32 * ll) as i32,
                    po + (qt - po) * ll,
                    aom(hw, jf, ll),
                    u1 + (u2 - u1) * ll,
                    v1 + (v2 - v1) * ll,
                    bm + ((x2 - bm) as f32 * np) as i32,
                    z0 + (qt - z0) * np,
                    aom(og, jf, np),
                    u0 + (u2 - u0) * np,
                    v0 + (v2 - v0) * np,
                )
            }
        };
        
        let (start_x, awy, start_z, end_z, start_c, end_c, start_u, end_u, start_v, end_v) = 
            if aja < bkd {
                (aja, bkd, bkl, zb, apz, cb, edc, ub, va, apk)
            } else {
                (bkd, aja, zb, bkl, cb, apz, ub, edc, apk, va)
            };
        
        for x in start_x.max(0)..=awy {
            if x < 0 || x as u32 >= width { continue; }
            
            let t = if awy != start_x {
                (x - start_x) as f32 / (awy - start_x) as f32
            } else {
                0.0
            };
            
            let z = start_z + (end_z - start_z) * t;
            let iy = start_u + (end_u - start_u) * t;
            let v = start_v + (end_v - start_v) * t;
            
            
            let dpo = if texturing {
                if let Some(tex_color) = texture::jcj(iy, v) {
                    
                    if smooth_shading {
                        let bad = aom(start_c, end_c, t);
                        let tr = ((tex_color >> 16) & 0xFF) as u32;
                        let bwi = ((tex_color >> 8) & 0xFF) as u32;
                        let aiv = (tex_color & 0xFF) as u32;
                        let r = (tr * bad.r as u32 / 255).min(255);
                        let g = (bwi * bad.g as u32 / 255).min(255);
                        let b = (aiv * bad.b as u32 / 255).min(255);
                        (0xFF << 24) | (r << 16) | (g << 8) | b
                    } else {
                        tex_color
                    }
                } else {
                    let color = if smooth_shading {
                        aom(start_c, end_c, t)
                    } else {
                        emx
                    };
                    color.to_u32()
                }
            } else {
                let color = if smooth_shading {
                    aom(start_c, end_c, t)
                } else {
                    emx
                };
                color.to_u32()
            };
            
            let idx = (y as usize) * (width as usize) + (x as usize);
            if idx < depth_buffer.len() {
                if !bys || z < depth_buffer[idx] {
                    depth_buffer[idx] = z;
                    framebuffer::put_pixel(x as u32, y as u32, dpo);
                }
            }
        }
    }
}






pub fn qjp(size: f32) {
    let j = size / 2.0;
    
    aqw(KZ_);
    
    
    ahz(0.0, 0.0, 1.0);
    dt(-j, -j, j);
    dt(j, -j, j);
    dt(j, j, j);
    dt(-j, j, j);
    
    
    ahz(0.0, 0.0, -1.0);
    dt(j, -j, -j);
    dt(-j, -j, -j);
    dt(-j, j, -j);
    dt(j, j, -j);
    
    
    ahz(0.0, 1.0, 0.0);
    dt(-j, j, j);
    dt(j, j, j);
    dt(j, j, -j);
    dt(-j, j, -j);
    
    
    ahz(0.0, -1.0, 0.0);
    dt(-j, -j, -j);
    dt(j, -j, -j);
    dt(j, -j, j);
    dt(-j, -j, j);
    
    
    ahz(1.0, 0.0, 0.0);
    dt(j, -j, j);
    dt(j, -j, -j);
    dt(j, j, -j);
    dt(j, j, j);
    
    
    ahz(-1.0, 0.0, 0.0);
    dt(-j, -j, -j);
    dt(-j, -j, j);
    dt(-j, j, j);
    dt(-j, j, -j);
    
    aqx();
}


pub fn qjr(size: f32) {
    let j = size / 2.0;
    
    aqw(OD_);
    
    dt(-j, -j, j);
    dt(j, -j, j);
    dt(j, j, j);
    dt(-j, j, j);
    aqx();
    
    aqw(OD_);
    
    dt(-j, -j, -j);
    dt(j, -j, -j);
    dt(j, j, -j);
    dt(-j, j, -j);
    aqx();
    
    aqw(UU_);
    
    dt(-j, -j, j); dt(-j, -j, -j);
    dt(j, -j, j); dt(j, -j, -j);
    dt(j, j, j); dt(j, j, -j);
    dt(-j, j, j); dt(-j, j, -j);
    aqx();
}


pub fn qjq(size: f32) {
    
    let qxr = 2;
    let t = (1.0 + 5.0_f32.sqrt()) / 2.0;
    
    
    let vertices = [
        Vec3::new(-1.0, t, 0.0).normalize().scale(size),
        Vec3::new(1.0, t, 0.0).normalize().scale(size),
        Vec3::new(-1.0, -t, 0.0).normalize().scale(size),
        Vec3::new(1.0, -t, 0.0).normalize().scale(size),
        Vec3::new(0.0, -1.0, t).normalize().scale(size),
        Vec3::new(0.0, 1.0, t).normalize().scale(size),
        Vec3::new(0.0, -1.0, -t).normalize().scale(size),
        Vec3::new(0.0, 1.0, -t).normalize().scale(size),
        Vec3::new(t, 0.0, -1.0).normalize().scale(size),
        Vec3::new(t, 0.0, 1.0).normalize().scale(size),
        Vec3::new(-t, 0.0, -1.0).normalize().scale(size),
        Vec3::new(-t, 0.0, 1.0).normalize().scale(size),
    ];
    
    let faces = [
        (0, 11, 5), (0, 5, 1), (0, 1, 7), (0, 7, 10), (0, 10, 11),
        (1, 5, 9), (5, 11, 4), (11, 10, 2), (10, 7, 6), (7, 1, 8),
        (3, 9, 4), (3, 4, 2), (3, 2, 6), (3, 6, 8), (3, 8, 9),
        (4, 9, 5), (2, 4, 11), (6, 2, 10), (8, 6, 7), (9, 8, 1),
    ];
    
    aqw(AEQ_);
    for (a, b, c) in &faces {
        let va = vertices[*a];
        let apk = vertices[*b];
        let bad = vertices[*c];
        let normal = (apk - va).cross(bad - va).normalize();
        
        ahz(normal.x, normal.y, normal.z);
        dt(va.x, va.y, va.z);
        dt(apk.x, apk.y, apk.z);
        dt(bad.x, bad.y, bad.z);
    }
    aqx();
}


pub fn mfd(size: f32) {
    let j = size / 2.0;
    
    aqw(KZ_);
    
    
    ahz(0.0, 0.0, 1.0);
    aay(0.0, 0.0); dt(-j, -j, j);
    aay(1.0, 0.0); dt(j, -j, j);
    aay(1.0, 1.0); dt(j, j, j);
    aay(0.0, 1.0); dt(-j, j, j);
    
    
    ahz(0.0, 0.0, -1.0);
    aay(0.0, 0.0); dt(j, -j, -j);
    aay(1.0, 0.0); dt(-j, -j, -j);
    aay(1.0, 1.0); dt(-j, j, -j);
    aay(0.0, 1.0); dt(j, j, -j);
    
    
    ahz(0.0, 1.0, 0.0);
    aay(0.0, 0.0); dt(-j, j, j);
    aay(1.0, 0.0); dt(j, j, j);
    aay(1.0, 1.0); dt(j, j, -j);
    aay(0.0, 1.0); dt(-j, j, -j);
    
    
    ahz(0.0, -1.0, 0.0);
    aay(0.0, 0.0); dt(-j, -j, -j);
    aay(1.0, 0.0); dt(j, -j, -j);
    aay(1.0, 1.0); dt(j, -j, j);
    aay(0.0, 1.0); dt(-j, -j, j);
    
    
    ahz(1.0, 0.0, 0.0);
    aay(0.0, 0.0); dt(j, -j, j);
    aay(1.0, 0.0); dt(j, -j, -j);
    aay(1.0, 1.0); dt(j, j, -j);
    aay(0.0, 1.0); dt(j, j, j);
    
    
    ahz(-1.0, 0.0, 0.0);
    aay(0.0, 0.0); dt(-j, -j, -j);
    aay(1.0, 0.0); dt(-j, -j, j);
    aay(1.0, 1.0); dt(-j, j, j);
    aay(0.0, 1.0); dt(-j, j, -j);
    
    aqx();
}


pub fn ldd(ceg: &mut u32) {
    texture::mes(1, core::slice::from_mut(ceg));
    texture::icb(texture::DD_, *ceg);
    
    let phz = texture::kzh(64, 0xFFFFFFFF, 0xFF404040);
    texture::meu(
        texture::DD_, 0, texture::AEP_,
        64, 64, 0, texture::AEP_, 0, &phz
    );
    texture::ich(texture::DD_, texture::AVQ_, texture::AVM_);
    texture::ich(texture::DD_, texture::AVR_, texture::AVN_);
}


pub fn lde(cc: f32, ceg: u32) {
    
    texture::icb(texture::DD_, ceg);
    texture::meq(texture::DD_);
    
    
    bmm(1.0, 1.0, 1.0);
    
    icg();
    eoj(cc, 0.5, 1.0, 0.3);
    mfd(1.5);
    icf();
    
    texture::mep(texture::DD_);
}
