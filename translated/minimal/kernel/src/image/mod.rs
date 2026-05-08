








use alloc::string::String;
use alloc::vec::Vec;

pub mod bmp;
pub mod ppm;
pub mod png;

pub use bmp::*;
pub use ppm::*;
pub use png::*;


#[derive(Clone)]
pub struct Image {
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<u32>,  
}

impl Image {
    
    pub fn new(width: u32, height: u32) -> Self {
        let size = (width * height) as usize;
        Self {
            width,
            height,
            pixels: alloc::vec![0xFF000000; size],
        }
    }
    
    
    pub fn cjv(width: u32, height: u32, pixels: Vec<u32>) -> Self {
        Self { width, height, pixels }
    }
    
    
    pub fn get_pixel(&self, x: u32, y: u32) -> u32 {
        if x < self.width && y < self.height {
            self.pixels[(y * self.width + x) as usize]
        } else {
            0
        }
    }
    
    
    pub fn set_pixel(&mut self, x: u32, y: u32, color: u32) {
        if x < self.width && y < self.height {
            self.pixels[(y * self.width + x) as usize] = color;
        }
    }
    
    
    pub fn fill(&mut self, color: u32) {
        for ct in &mut self.pixels {
            *ct = color;
        }
    }
    
    
    pub fn draw(&self, x: i32, y: i32) {
        self.draw_scaled(x, y, self.width, self.height);
    }
    
    
    pub fn draw_scaled(&self, x: i32, y: i32, cif: u32, cie: u32) {
        let (fb_width, fb_height) = crate::framebuffer::kv();
        
        for ad in 0..cie {
            let nn = y + ad as i32;
            if nn < 0 || nn >= fb_height as i32 { continue; }
            
            
            let aft = (ad as u64 * self.height as u64 / cie as u64) as u32;
            
            for dx in 0..cif {
                let lw = x + dx as i32;
                if lw < 0 || lw >= fb_width as i32 { continue; }
                
                
                let ahc = (dx as u64 * self.width as u64 / cif as u64) as u32;
                
                let ct = self.get_pixel(ahc, aft);
                let alpha = (ct >> 24) & 0xFF;
                
                if alpha == 255 {
                    
                    crate::framebuffer::put_pixel(lw as u32, nn as u32, ct);
                } else if alpha > 0 {
                    
                    let bg = crate::framebuffer::get_pixel(lw as u32, nn as u32);
                    let bex = hic(ct, bg);
                    crate::framebuffer::put_pixel(lw as u32, nn as u32, bex);
                }
                
            }
        }
    }
    
    
    pub fn lic(&self, x: i32, y: i32, global_alpha: u8) {
        let (fb_width, fb_height) = crate::framebuffer::kv();
        
        for ak in 0..self.height {
            let nn = y + ak as i32;
            if nn < 0 || nn >= fb_height as i32 { continue; }
            
            for am in 0..self.width {
                let lw = x + am as i32;
                if lw < 0 || lw >= fb_width as i32 { continue; }
                
                let ct = self.get_pixel(am, ak);
                let nux = ((ct >> 24) & 0xFF) as u32;
                let hna = (nux * global_alpha as u32) / 255;
                
                if hna > 0 {
                    let modified = (hna << 24) | (ct & 0x00FFFFFF);
                    let bg = crate::framebuffer::get_pixel(lw as u32, nn as u32);
                    let bex = hic(modified, bg);
                    crate::framebuffer::put_pixel(lw as u32, nn as u32, bex);
                }
            }
        }
    }
    
    
    pub fn scale(&self, aym: u32, ayk: u32) -> Image {
        let mut dyk = Image::new(aym, ayk);
        
        for ad in 0..ayk {
            let aft = (ad as u64 * self.height as u64 / ayk as u64) as u32;
            for dx in 0..aym {
                let ahc = (dx as u64 * self.width as u64 / aym as u64) as u32;
                dyk.set_pixel(dx, ad, self.get_pixel(ahc, aft));
            }
        }
        
        dyk
    }
    
    
    pub fn qcb(&self, x: u32, y: u32, w: u32, h: u32) -> Image {
        let mut hoy = Image::new(w, h);
        
        for ad in 0..h {
            for dx in 0..w {
                hoy.set_pixel(dx, ad, self.get_pixel(x + dx, y + ad));
            }
        }
        
        hoy
    }
}


fn hic(fg: u32, bg: u32) -> u32 {
    let alpha = ((fg >> 24) & 0xFF) as u32;
    if alpha == 0 { return bg; }
    if alpha == 255 { return fg; }
    
    let sg = 255 - alpha;
    
    let bsn = ((fg >> 16) & 0xFF) as u32;
    let bsm = ((fg >> 8) & 0xFF) as u32;
    let bsl = (fg & 0xFF) as u32;
    
    let awg = ((bg >> 16) & 0xFF) as u32;
    let awf = ((bg >> 8) & 0xFF) as u32;
    let awe = (bg & 0xFF) as u32;
    
    let r = (bsn * alpha + awg * sg) / 255;
    let g = (bsm * alpha + awf * sg) / 255;
    let b = (bsl * alpha + awe * sg) / 255;
    
    0xFF000000 | (r << 16) | (g << 8) | b
}






pub fn load(path: &str) -> Option<Image> {
    
    let dtv = path.to_lowercase_simple();
    
    if dtv.ends_with(".bmp") {
        bmp::ete(path)
    } else if dtv.ends_with(".ppm") || dtv.ends_with(".pnm") {
        ppm::ikw(path)
    } else if dtv.ends_with(".raw") || dtv.ends_with(".rgba") {
        
        if let Ok(data) = crate::vfs::read_file(path) {
            
            if data.len() >= 8 {
                let width = u32::from_le_bytes([data[0], data[1], data[2], data[3]]);
                let height = u32::from_le_bytes([data[4], data[5], data[6], data[7]]);
                let tm = &data[8..];
                
                if tm.len() >= (width * height * 4) as usize {
                    let mut pixels = Vec::with_capacity((width * height) as usize);
                    for i in 0..(width * height) as usize {
                        let offset = i * 4;
                        let ct = u32::from_le_bytes([
                            tm[offset],
                            tm[offset + 1],
                            tm[offset + 2],
                            tm[offset + 3],
                        ]);
                        pixels.push(ct);
                    }
                    return Some(Image::cjv(width, height, pixels));
                }
            }
        }
        None
    } else {
        
        bmp::ete(path).or_else(|| ppm::ikw(path))
    }
}


pub fn gfw(data: &[u8], format: ImageFormat) -> Option<Image> {
    match format {
        ImageFormat::Bmp => bmp::dtq(data),
        ImageFormat::Ppm => ppm::gfz(data),
        ImageFormat::Raw { width, height } => {
            if data.len() >= (width * height * 4) as usize {
                let mut pixels = Vec::with_capacity((width * height) as usize);
                for i in 0..(width * height) as usize {
                    let offset = i * 4;
                    let ct = u32::from_le_bytes([
                        data[offset],
                        data[offset + 1],
                        data[offset + 2],
                        data[offset + 3],
                    ]);
                    pixels.push(ct);
                }
                Some(Image::cjv(width, height, pixels))
            } else {
                None
            }
        }
    }
}

#[derive(Clone, Copy)]
pub enum ImageFormat {
    Bmp,
    Ppm,
    Raw { width: u32, height: u32 },
}


trait Aqq {
    fn to_lowercase_simple(&self) -> String;
}

impl Aqq for str {
    fn to_lowercase_simple(&self) -> String {
        self.chars().map(|c| {
            if c >= 'A' && c <= 'Z' {
                (c as u8 + 32) as char
            } else {
                c
            }
        }).collect()
    }
}






pub fn kzk(width: u32, height: u32, color: u32) -> Image {
    let mut iv = Image::new(width, height);
    iv.fill(color);
    iv
}


pub fn hop(width: u32, height: u32, top_color: u32, bottom_color: u32) -> Image {
    let mut iv = Image::new(width, height);
    
    let tr = ((top_color >> 16) & 0xFF) as i32;
    let bwi = ((top_color >> 8) & 0xFF) as i32;
    let aiv = (top_color & 0xFF) as i32;
    
    let yi = ((bottom_color >> 16) & 0xFF) as i32;
    let bg = ((bottom_color >> 8) & 0xFF) as i32;
    let mq = (bottom_color & 0xFF) as i32;
    
    for y in 0..height {
        let t = y as f32 / height as f32;
        let r = (tr as f32 * (1.0 - t) + yi as f32 * t) as u32;
        let g = (bwi as f32 * (1.0 - t) + bg as f32 * t) as u32;
        let b = (aiv as f32 * (1.0 - t) + mq as f32 * t) as u32;
        let color = 0xFF000000 | (r << 16) | (g << 8) | b;
        
        for x in 0..width {
            iv.set_pixel(x, y, color);
        }
    }
    
    iv
}


pub fn kzg(width: u32, height: u32, size: u32, agh: u32, ale: u32) -> Image {
    let mut iv = Image::new(width, height);
    
    for y in 0..height {
        for x in 0..width {
            let kjz = ((x / size) + (y / size)) % 2 == 0;
            iv.set_pixel(x, y, if kjz { agh } else { ale });
        }
    }
    
    iv
}
