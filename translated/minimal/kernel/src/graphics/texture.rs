




use alloc::vec::Vec;
use alloc::boxed::Box;
use spin::Mutex;
use micromath::F32Ext;

use super::render2d::Color2D;






pub const DRT_: u32 = 0x0DE0;
pub const DD_: u32 = 0x0DE1;


pub const AVQ_: u32 = 0x2800;
pub const AVR_: u32 = 0x2801;
pub const CBG_: u32 = 0x2802;
pub const CBH_: u32 = 0x2803;


pub const AVN_: u32 = 0x2600;
pub const AVM_: u32 = 0x2601;
pub const DRR_: u32 = 0x2700;
pub const DRP_: u32 = 0x2701;
pub const CBD_: u32 = 0x2702;
pub const DRO_: u32 = 0x2703;


pub const AVP_: u32 = 0x2901;
pub const CAV_: u32 = 0x2900;
pub const CAW_: u32 = 0x812F;
pub const CBC_: u32 = 0x8370;


pub const CBF_: u32 = 0x1907;
pub const AEP_: u32 = 0x1908;
pub const CBB_: u32 = 0x1909;
pub const DRQ_: u32 = 0x190A;






#[derive(Clone)]
pub struct TextureLevel {
    pub width: u32,
    pub height: u32,
    pub data: Vec<u32>, 
}

impl TextureLevel {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            data: alloc::vec![0u32; (width * height) as usize],
        }
    }
    
    
    #[inline]
    pub fn sample_pixel(&self, x: u32, y: u32) -> u32 {
        let x = x.min(self.width.saturating_sub(1));
        let y = y.min(self.height.saturating_sub(1));
        self.data[(y * self.width + x) as usize]
    }
    
    
    pub fn sample_bilinear(&self, iy: f32, v: f32) -> u32 {
        let x = iy * (self.width as f32 - 1.0);
        let y = v * (self.height as f32 - 1.0);
        
        let bm = x.floor() as u32;
        let az = y.floor() as u32;
        let x1 = (bm + 1).min(self.width - 1);
        let y1 = (az + 1).min(self.height - 1);
        
        let dg = x.fract();
        let hj = y.fract();
        
        let anq = self.sample_pixel(bm, az);
        let apx = self.sample_pixel(x1, az);
        let anr = self.sample_pixel(bm, y1);
        let apy = self.sample_pixel(x1, y1);
        
        
        let r = Self::egz(
            ((anq >> 16) & 0xFF) as f32,
            ((apx >> 16) & 0xFF) as f32,
            ((anr >> 16) & 0xFF) as f32,
            ((apy >> 16) & 0xFF) as f32,
            dg, hj
        ) as u32;
        let g = Self::egz(
            ((anq >> 8) & 0xFF) as f32,
            ((apx >> 8) & 0xFF) as f32,
            ((anr >> 8) & 0xFF) as f32,
            ((apy >> 8) & 0xFF) as f32,
            dg, hj
        ) as u32;
        let b = Self::egz(
            (anq & 0xFF) as f32,
            (apx & 0xFF) as f32,
            (anr & 0xFF) as f32,
            (apy & 0xFF) as f32,
            dg, hj
        ) as u32;
        let a = Self::egz(
            ((anq >> 24) & 0xFF) as f32,
            ((apx >> 24) & 0xFF) as f32,
            ((anr >> 24) & 0xFF) as f32,
            ((apy >> 24) & 0xFF) as f32,
            dg, hj
        ) as u32;
        
        (a << 24) | (r << 16) | (g << 8) | b
    }
    
    #[inline]
    fn egz(anq: f32, apx: f32, anr: f32, apy: f32, dg: f32, hj: f32) -> f32 {
        let og = anq + (apx - anq) * dg;
        let hw = anr + (apy - anr) * dg;
        og + (hw - og) * hj
    }
}


pub struct Texture {
    pub id: u32,
    pub target: u32,
    pub levels: Vec<TextureLevel>,
    pub mag_filter: u32,
    pub min_filter: u32,
    pub wrap_s: u32,
    pub wrap_t: u32,
}

impl Texture {
    pub fn new(id: u32) -> Self {
        Self {
            id,
            target: DD_,
            levels: Vec::new(),
            mag_filter: AVM_,
            min_filter: CBD_,
            wrap_s: AVP_,
            wrap_t: AVP_,
        }
    }
    
    
    pub fn upload(&mut self, width: u32, height: u32, format: u32, data: &[u8]) {
        let mut level = TextureLevel::new(width, height);
        
        match format {
            AEP_ => {
                for (i, df) in data.chunks(4).enumerate() {
                    if df.len() == 4 && i < level.data.len() {
                        level.data[i] = ((df[3] as u32) << 24) 
                                      | ((df[0] as u32) << 16) 
                                      | ((df[1] as u32) << 8) 
                                      | (df[2] as u32);
                    }
                }
            }
            CBF_ => {
                for (i, df) in data.chunks(3).enumerate() {
                    if df.len() == 3 && i < level.data.len() {
                        level.data[i] = 0xFF000000 
                                      | ((df[0] as u32) << 16) 
                                      | ((df[1] as u32) << 8) 
                                      | (df[2] as u32);
                    }
                }
            }
            CBB_ => {
                for (i, &cml) in data.iter().enumerate() {
                    if i < level.data.len() {
                        level.data[i] = 0xFF000000 
                                      | ((cml as u32) << 16) 
                                      | ((cml as u32) << 8) 
                                      | (cml as u32);
                    }
                }
            }
            _ => {}
        }
        
        self.levels.clear();
        self.levels.push(level);
    }
    
    
    pub fn generate_mipmaps(&mut self) {
        if self.levels.is_empty() {
            return;
        }
        
        let mut w = self.levels[0].width / 2;
        let mut h = self.levels[0].height / 2;
        
        while w >= 1 && h >= 1 {
            let prev = &self.levels[self.levels.len() - 1];
            let mut level = TextureLevel::new(w, h);
            
            
            for y in 0..h {
                for x in 0..w {
                    let am = x * 2;
                    let ak = y * 2;
                    
                    let anq = prev.sample_pixel(am, ak);
                    let apx = prev.sample_pixel(am + 1, ak);
                    let anr = prev.sample_pixel(am, ak + 1);
                    let apy = prev.sample_pixel(am + 1, ak + 1);
                    
                    let r = (((anq >> 16) & 0xFF) + ((apx >> 16) & 0xFF) 
                           + ((anr >> 16) & 0xFF) + ((apy >> 16) & 0xFF)) / 4;
                    let g = (((anq >> 8) & 0xFF) + ((apx >> 8) & 0xFF) 
                           + ((anr >> 8) & 0xFF) + ((apy >> 8) & 0xFF)) / 4;
                    let b = ((anq & 0xFF) + (apx & 0xFF) 
                           + (anr & 0xFF) + (apy & 0xFF)) / 4;
                    let a = (((anq >> 24) & 0xFF) + ((apx >> 24) & 0xFF) 
                           + ((anr >> 24) & 0xFF) + ((apy >> 24) & 0xFF)) / 4;
                    
                    level.data[(y * w + x) as usize] = (a << 24) | (r << 16) | (g << 8) | b;
                }
            }
            
            self.levels.push(level);
            
            if w == 1 && h == 1 { break; }
            w = w.max(1) / 2;
            h = h.max(1) / 2;
            if w == 0 { w = 1; }
            if h == 0 { h = 1; }
        }
    }
    
    
    pub fn sample(&self, mut iy: f32, mut v: f32) -> u32 {
        if self.levels.is_empty() {
            return 0xFFFFFFFF;
        }
        
        
        iy = self.apply_wrap(iy, self.wrap_s);
        v = self.apply_wrap(v, self.wrap_t);
        
        let level = &self.levels[0]; 
        
        match self.mag_filter {
            AVN_ => {
                let x = (iy * level.width as f32) as u32;
                let y = (v * level.height as f32) as u32;
                level.sample_pixel(x, y)
            }
            _ => level.sample_bilinear(iy, v),
        }
    }
    
    fn apply_wrap(&self, coord: f32, mode: u32) -> f32 {
        match mode {
            CAV_ | CAW_ => coord.clamp(0.0, 1.0),
            CBC_ => {
                let i = coord.floor() as i32;
                let f = coord.fract();
                if i % 2 == 0 { f } else { 1.0 - f }
            }
            _ => { 
                let f = coord.fract();
                if f < 0.0 { 1.0 + f } else { f }
            }
        }
    }
}






pub struct TextureState {
    textures: Vec<Option<Texture>>,
    next_id: u32,
    bound_2d: Option<u32>,
    texture_2d_enabled: bool,
}

impl TextureState {
    pub const fn new() -> Self {
        Self {
            textures: Vec::new(),
            next_id: 1,
            bound_2d: None,
            texture_2d_enabled: false,
        }
    }
}

static FH_: Mutex<TextureState> = Mutex::new(TextureState::new());






pub fn mes(ae: i32, textures: &mut [u32]) {
    let mut state = FH_.lock();
    for i in 0..(ae as usize) {
        if i < textures.len() {
            let id = state.next_id;
            textures[i] = id;
            state.textures.push(Some(Texture::new(id)));
            state.next_id += 1;
        }
    }
}


pub fn icb(target: u32, texture: u32) {
    let mut state = FH_.lock();
    if target == DD_ {
        state.bound_2d = if texture == 0 { None } else { Some(texture) };
    }
}


pub fn ich(target: u32, biq: u32, param: u32) {
    let mut state = FH_.lock();
    
    let ceg = match target {
        DD_ => state.bound_2d,
        _ => None,
    };
    
    if let Some(id) = ceg {
        if let Some(Some(bdz)) = state.textures.iter_mut().find(|t| t.as_ref().map(|x| x.id) == Some(id)) {
            match biq {
                AVQ_ => bdz.mag_filter = param,
                AVR_ => bdz.min_filter = param,
                CBG_ => bdz.wrap_s = param,
                CBH_ => bdz.wrap_t = param,
                _ => {}
            }
        }
    }
}


pub fn meu(
    _target: u32,
    _level: i32,
    _internal_format: u32,
    width: u32,
    height: u32,
    _border: i32,
    format: u32,
    _type: u32,
    data: &[u8],
) {
    let mut state = FH_.lock();
    
    if let Some(id) = state.bound_2d {
        if let Some(Some(bdz)) = state.textures.iter_mut().find(|t| t.as_ref().map(|x| x.id) == Some(id)) {
            bdz.upload(width, height, format, data);
        }
    }
}


pub fn qjg(target: u32) {
    let mut state = FH_.lock();
    
    let ceg = match target {
        DD_ => state.bound_2d,
        _ => None,
    };
    
    if let Some(id) = ceg {
        if let Some(Some(bdz)) = state.textures.iter_mut().find(|t| t.as_ref().map(|x| x.id) == Some(id)) {
            bdz.generate_mipmaps();
        }
    }
}


pub fn meq(target: u32) {
    let mut state = FH_.lock();
    if target == DD_ {
        state.texture_2d_enabled = true;
    }
}


pub fn mep(target: u32) {
    let mut state = FH_.lock();
    if target == DD_ {
        state.texture_2d_enabled = false;
    }
}


pub fn jcj(iy: f32, v: f32) -> Option<u32> {
    let state = FH_.lock();
    
    if !state.texture_2d_enabled {
        return None;
    }
    
    if let Some(id) = state.bound_2d {
        if let Some(Some(bdz)) = state.textures.iter().find(|t| t.as_ref().map(|x| x.id) == Some(id)) {
            return Some(bdz.sample(iy, v));
        }
    }
    
    None
}


pub fn mtw() -> bool {
    FH_.lock().texture_2d_enabled
}


pub fn qjd(ae: i32, textures: &[u32]) {
    let mut state = FH_.lock();
    for i in 0..(ae as usize) {
        if i < textures.len() {
            let id = textures[i];
            if let Some(pos) = state.textures.iter().position(|t| t.as_ref().map(|x| x.id) == Some(id)) {
                state.textures[pos] = None;
            }
            if state.bound_2d == Some(id) {
                state.bound_2d = None;
            }
        }
    }
}






pub fn kzh(size: u32, agh: u32, ale: u32) -> Vec<u8> {
    let mut data = Vec::with_capacity((size * size * 4) as usize);
    let hks = size / 8;
    
    for y in 0..size {
        for x in 0..size {
            let cx = x / hks.max(1);
            let u = y / hks.max(1);
            let color = if (cx + u) % 2 == 0 { agh } else { ale };
            
            data.push(((color >> 16) & 0xFF) as u8); 
            data.push(((color >> 8) & 0xFF) as u8);  
            data.push((color & 0xFF) as u8);         
            data.push(((color >> 24) & 0xFF) as u8); 
        }
    }
    
    data
}


pub fn qbp(size: u32) -> Vec<u8> {
    let mut data = Vec::with_capacity((size * size * 4) as usize);
    let djo = size / 4;
    let djp = size / 2;
    let duo = 2u32;
    
    let kdy: u32 = 0xFF8B4513; 
    let ngg: u32 = 0xFFC0C0C0; 
    
    for y in 0..size {
        for x in 0..size {
            let row = y / djo;
            let offset = if row % 2 == 0 { 0 } else { djp / 2 };
            let bx = (x + offset) % djp;
            let dc = y % djo;
            
            let mtd = dc < duo || bx < duo;
            let color = if mtd { ngg } else { kdy };
            
            data.push(((color >> 16) & 0xFF) as u8);
            data.push(((color >> 8) & 0xFF) as u8);
            data.push((color & 0xFF) as u8);
            data.push(0xFF);
        }
    }
    
    data
}


pub fn qbr(size: u32, agh: Color2D, ale: Color2D, horizontal: bool) -> Vec<u8> {
    let mut data = Vec::with_capacity((size * size * 4) as usize);
    
    for y in 0..size {
        for x in 0..size {
            let t = if horizontal {
                x as f32 / size as f32
            } else {
                y as f32 / size as f32
            };
            
            let r = (agh.r as f32 + (ale.r as f32 - agh.r as f32) * t) as u8;
            let g = (agh.g as f32 + (ale.g as f32 - agh.g as f32) * t) as u8;
            let b = (agh.b as f32 + (ale.b as f32 - agh.b as f32) * t) as u8;
            
            data.push(r);
            data.push(g);
            data.push(b);
            data.push(0xFF);
        }
    }
    
    data
}
