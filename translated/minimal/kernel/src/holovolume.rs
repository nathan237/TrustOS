







use alloc::vec::Vec;
use alloc::vec;
use alloc::boxed::Box;





pub const AD_: usize = 240;
pub const AV_: usize = 68;


pub const BKX_: usize = 16;


pub const BWA_: f32 = 0.12;






#[derive(Clone, Copy, PartialEq)]
pub enum ColorMod {
    
    Normal,
    
    H,
    
    Inside(f32),
}





#[derive(Clone, Copy, PartialEq)]
pub enum Shape3D {
    None,
    Cube,
    Sphere,
    Torus,
    DnaHelix,
}

#[derive(Clone, Copy, PartialEq)]
pub enum RenderMode {
    Hologram,
    MatrixRain,
    DnaHelix,
    RotatingCube,
    Sphere,
}





pub struct HoloVolume {
    pub width: usize,
    pub height: usize,
    pub depth: usize,
    
    
    pub shape: Shape3D,
    
    
    pub render_mode: RenderMode,
    
    
    time: f32,
    
    
    rotation: f32,
    
    
    
    
    intensity_map: Vec<ColorMod>,
    
    
    screen_width: usize,
    screen_height: usize,
}

impl HoloVolume {
    pub fn new(width: usize, height: usize, depth: usize) -> Self {
        
        
        let intensity_map = vec![ColorMod::Normal; AD_ * AV_];
        Self {
            width,
            height,
            depth,
            shape: Shape3D::Cube,
            render_mode: RenderMode::Hologram,
            time: 0.0,
            rotation: 0.0,
            intensity_map,
            screen_width: 1920,
            screen_height: 1080,
        }
    }
    
    
    #[inline]
    fn idx(col: usize, row: usize) -> usize {
        col * AV_ + row
    }
    
    pub fn set_shape(&mut self, shape: Shape3D) {
        self.shape = shape;
    }
    
    
    pub fn set_screen_size(&mut self, width: usize, height: usize) {
        self.screen_width = width;
        self.screen_height = height;
    }
    
    
    pub fn update(&mut self, fm: f32) {
        self.time += fm;
        self.rotation += fm * 0.5;
        
        
        let fuf = match self.render_mode {
            RenderMode::DnaHelix => Shape3D::DnaHelix,
            RenderMode::RotatingCube => Shape3D::Cube,
            RenderMode::Sphere => Shape3D::Sphere,
            RenderMode::MatrixRain => Shape3D::None,
            RenderMode::Hologram => self.shape,
        };
        
        
        self.compute_intensity_map(fuf);
    }
    
    
    fn compute_intensity_map(&mut self, shape: Shape3D) {
        if shape == Shape3D::None {
            
            for col in 0..AD_ {
                for row in 0..AV_ {
                    self.intensity_map[Self::idx(col, row)] = ColorMod::Normal;
                }
            }
            return;
        }
        
        let cell_w = 8.0;  
        let cell_h = 16.0; 
        
        
        let cx = self.screen_width as f32 / 2.0;
        let u = self.screen_height as f32 / 2.0;
        let jge = (self.screen_width.min(self.screen_height) as f32) * 0.25;
        
        for col in 0..AD_ {
            let x = col as f32 * cell_w + cell_w / 2.0;
            
            for row in 0..AV_ {
                let y = row as f32 * cell_h + cell_h / 2.0;
                
                
                let nx = (x - cx) / jge;
                let re = (y - u) / jge;
                
                
                let mut euk = 999.0f32;
                let mut hfi = false;
                
                for z_layer in 0..BKX_ {
                    let wi = (z_layer as f32 / BKX_ as f32) * 2.0 - 1.0;
                    let gtd = self.shape_sdf(nx, re, wi, shape);
                    
                    if gtd < euk {
                        euk = gtd;
                    }
                    if gtd < 0.0 {
                        hfi = true;
                    }
                }
                
                
                self.intensity_map[Self::idx(col, row)] = if euk.abs() < BWA_ {
                    
                    ColorMod::H
                } else if hfi {
                    
                    let bma = -euk;
                    let imi = 0.8;
                    let bmo = if bma < imi {
                        1.0 - (bma / imi)
                    } else {
                        0.0
                    };
                    ColorMod::Inside(bmo)
                } else {
                    
                    ColorMod::Normal
                };
            }
        }
    }
    
    
    
    
    pub fn get_u8_intensity_map(&self) -> Vec<u8> {
        let mut result = vec![0u8; AD_ * AV_];
        
        for col in 0..AD_ {
            for row in 0..AV_ {
                let idx = Self::idx(col, row);
                result[idx] = match self.intensity_map[idx] {
                    ColorMod::Normal => 0,
                    ColorMod::H => 1,
                    ColorMod::Inside(bmo) => {
                        
                        
                        let mro = 1.0 - bmo;
                        (2.0 + mro * 253.0) as u8
                    }
                };
            }
        }
        
        result
    }
    
    
    
    #[inline]
    pub fn get_color_mod(&self, col: usize, row: usize) -> ColorMod {
        if col < AD_ && row < AV_ {
            self.intensity_map[Self::idx(col, row)]
        } else {
            ColorMod::Normal
        }
    }
    
    
    pub fn qkl(&self) -> bool {
        let fuf = match self.render_mode {
            RenderMode::DnaHelix => Shape3D::DnaHelix,
            RenderMode::RotatingCube => Shape3D::Cube,
            RenderMode::Sphere => Shape3D::Sphere,
            RenderMode::MatrixRain => Shape3D::None,
            RenderMode::Hologram => self.shape,
        };
        fuf != Shape3D::None
    }
    
    
    
    #[inline]
    pub fn pyf(&self, dik: u32, col: usize, row: usize) -> u32 {
        match self.get_color_mod(col, row) {
            ColorMod::Normal => {
                
                let g = dik.min(238);
                0xFF000000 | (g << 8)
            }
            ColorMod::H => {
                
                0xFF00FF00
            }
            ColorMod::Inside(bmo) => {
                
                let g = ((dik as f32) * bmo * 0.7) as u32;
                if g < 10 {
                    0xFF000000 
                } else {
                    0xFF000000 | (g << 8)
                }
            }
        }
    }
    
    
    
    pub fn qty(&self, buffer: &mut [u32], _screen_width: usize, _screen_height: usize) {
        
        
        buffer.fill(0xFF000000);
    }
    
    
    fn shape_sdf(&self, x: f32, y: f32, z: f32, shape: Shape3D) -> f32 {
        
        let bax = libm::cosf(self.rotation);
        let bds = libm::sinf(self.rotation);
        let da = x * bax - z * bds;
        let qp = x * bds + z * bax;
        let cm = y;
        
        match shape {
            Shape3D::Cube => {
                let size = 0.7;
                let dx = libm::fabsf(da) - size;
                let ad = libm::fabsf(cm) - size;
                let dz = libm::fabsf(qp) - size;
                let cg = libm::fmaxf(dx, 0.0);
                let cr = libm::fmaxf(ad, 0.0);
                let ipf = libm::fmaxf(dz, 0.0);
                let glk = libm::sqrtf(cg * cg + cr * cr + ipf * ipf);
                let bmz = libm::fminf(libm::fmaxf(dx, libm::fmaxf(ad, dz)), 0.0);
                glk + bmz
            }
            
            Shape3D::Sphere => {
                let radius = 0.8;
                libm::sqrtf(da * da + cm * cm + qp * qp) - radius
            }
            
            Shape3D::Torus => {
                let axz = 0.6;
                let ayh = 0.25;
                let q = libm::sqrtf(da * da + qp * qp) - axz;
                libm::sqrtf(q * q + cm * cm) - ayh
            }
            
            Shape3D::DnaHelix => {
                let ckn = 0.4;
                let mky = 2.0;
                let jiz = 0.15;
                
                let cc = cm * mky + self.time * 2.0;
                
                let ojp = ckn * libm::cosf(cc);
                let ojq = ckn * libm::sinf(cc);
                let huj = da - ojp;
                let hup = qp - ojq;
                let vh = libm::sqrtf(huj * huj + hup * hup) - jiz;
                
                let ojr = ckn * libm::cosf(cc + core::f32::consts::PI);
                let ojs = ckn * libm::sinf(cc + core::f32::consts::PI);
                let huk = da - ojr;
                let huq = qp - ojs;
                let jq = libm::sqrtf(huk * huk + huq * huq) - jiz;
                
                libm::fminf(vh, jq)
            }
            
            Shape3D::None => 999.0,
        }
    }
}
