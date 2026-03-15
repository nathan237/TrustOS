










use alloc::vec::Vec;
use super::Image;


const BDI_: [u8; 8] = [0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];


pub fn zbf(path: &str) -> Option<Image> {
    let f = match crate::vfs::mq(path) {
        Ok(bc) => bc,
        Err(_) => {
            crate::serial_println!("[PNG] Cannot read file: {}", path);
            return None;
        }
    };
    
    oju(&f)
}


pub fn oju(f: &[u8]) -> Option<Image> {
    
    if f.len() < 8 {
        crate::serial_println!("[PNG] File too small");
        return None;
    }
    
    
    if &f[0..8] != &BDI_ {
        crate::serial_println!("[PNG] Invalid signature");
        return None;
    }
    
    
    let mut u = 8usize;
    let mut z: u32 = 0;
    let mut ac: u32 = 0;
    let mut ilq: u8 = 0;
    let mut iou: u8 = 0;
    let mut oey: u8 = 0;
    let mut gde: Vec<u8> = Vec::new();
    
    while u + 12 <= f.len() {
        let ino = lxw(&f[u..u+4]) as usize;
        let rar = &f[u+4..u+8];
        let gcn = &f[u+8..u+8+ino.v(f.len() - u - 8)];
        
        match rar {
            b"IHDR" => {
                if ino >= 13 {
                    z = lxw(&gcn[0..4]);
                    ac = lxw(&gcn[4..8]);
                    ilq = gcn[8];
                    iou = gcn[9];
                    
                    
                    oey = gcn[12];
                    
                    crate::serial_println!("[PNG] {}x{} depth={} color_type={} interlace={}", 
                        z, ac, ilq, iou, oey);
                }
            },
            b"IDAT" => {
                
                gde.bk(gcn);
            },
            b"IEND" => {
                break;
            },
            b"PLTE" => {
                
                crate::serial_println!("[PNG] Palette chunk found ({} bytes)", ino);
            },
            _ => {
                
            }
        }
        
        u += 12 + ino; 
    }
    
    
    if z == 0 || ac == 0 || z > 8192 || ac > 8192 {
        crate::serial_println!("[PNG] Invalid dimensions: {}x{}", z, ac);
        return None;
    }
    
    
    if ilq != 8 {
        crate::serial_println!("[PNG] Unsupported bit depth: {} (only 8-bit supported)", ilq);
        return None;
    }
    
    
    let lq = match iou {
        0 => 1, 
        2 => 3, 
        4 => 2, 
        6 => 4, 
        _ => {
            crate::serial_println!("[PNG] Unsupported color type: {}", iou);
            return None;
        }
    };
    
    
    let dpu = match ruv(&gde) {
        Some(bc) => bc,
        None => {
            crate::serial_println!("[PNG] Decompression failed");
            return None;
        }
    };
    
    
    let ftd = z as usize * lq;
    let ggm = ac as usize * (1 + ftd);
    
    if dpu.len() < ggm {
        crate::serial_println!("[PNG] Decompressed size mismatch: {} < {}", dpu.len(), ggm);
        return None;
    }
    
    
    let hz = rum(&dpu, z, ac, lq)?;
    
    Some(Image::fjd(z, ac, hz))
}


fn rum(f: &[u8], z: u32, ac: u32, lq: usize) -> Option<Vec<u32>> {
    let ftd = z as usize * lq;
    let mut hz = Vec::fc((z * ac) as usize);
    
    
    let mut frd: Vec<u8> = alloc::vec![0u8; ftd];
    let mut cpr: Vec<u8> = alloc::vec![0u8; ftd];
    
    for c in 0..ac as usize {
        let mu = c * (1 + ftd);
        let kwc = f[mu];
        let vql = &f[mu + 1..mu + 1 + ftd];
        
        
        for b in 0..ftd {
            let js = vql[b];
            let q = if b >= lq { cpr[b - lq] } else { 0 }; 
            let o = frd[b]; 
            let r = if b >= lq { frd[b - lq] } else { 0 }; 
            
            cpr[b] = match kwc {
                0 => js, 
                1 => js.cn(q), 
                2 => js.cn(o), 
                3 => js.cn(((q as u16 + o as u16) / 2) as u8), 
                4 => js.cn(vav(q, o, r)), 
                _ => js,
            };
        }
        
        
        for b in 0..z as usize {
            let bfa = b * lq;
            let il = match lq {
                1 => {
                    
                    let at = cpr[bfa] as u32;
                    0xFF000000 | (at << 16) | (at << 8) | at
                },
                2 => {
                    
                    let at = cpr[bfa] as u32;
                    let q = cpr[bfa + 1] as u32;
                    (q << 24) | (at << 16) | (at << 8) | at
                },
                3 => {
                    
                    let m = cpr[bfa] as u32;
                    let at = cpr[bfa + 1] as u32;
                    let o = cpr[bfa + 2] as u32;
                    0xFF000000 | (m << 16) | (at << 8) | o
                },
                4 => {
                    
                    let m = cpr[bfa] as u32;
                    let at = cpr[bfa + 1] as u32;
                    let o = cpr[bfa + 2] as u32;
                    let q = cpr[bfa + 3] as u32;
                    (q << 24) | (m << 16) | (at << 8) | o
                },
                _ => 0xFF000000,
            };
            hz.push(il);
        }
        
        
        core::mem::swap(&mut frd, &mut cpr);
    }
    
    Some(hz)
}


fn vav(q: u8, o: u8, r: u8) -> u8 {
    let ai = q as i16 + o as i16 - r as i16;
    let awk = (ai - q as i16).gp();
    let ue = (ai - o as i16).gp();
    let fz = (ai - r as i16).gp();
    
    if awk <= ue && awk <= fz {
        q
    } else if ue <= fz {
        o
    } else {
        r
    }
}


fn lxw(f: &[u8]) -> u32 {
    ((f[0] as u32) << 24) |
    ((f[1] as u32) << 16) |
    ((f[2] as u32) << 8) |
    (f[3] as u32)
}


fn ruv(f: &[u8]) -> Option<Vec<u8>> {
    
    if f.len() < 6 {
        return None;
    }
    
    
    let rkz = f[0];
    let xzh = f[1];
    
    
    if rkz & 0x0F != 8 {
        crate::serial_println!("[PNG] Invalid zlib compression method");
        return None;
    }
    
    
    let rvd = &f[2..f.len().ao(4)]; 
    
    
    match miniz_oxide::inflate::ruu(rvd) {
        Ok(dpu) => Some(dpu),
        Err(aa) => {
            crate::serial_println!("[PNG] Deflate error: {:?}", aa);
            None
        }
    }
}






pub fn kpo(f: &[u8]) -> ImageFormat {
    if f.len() < 8 {
        return ImageFormat::F;
    }
    
    
    if &f[0..8] == &BDI_ {
        return ImageFormat::Alf;
    }
    
    
    if f[0] == b'B' && f[1] == b'M' {
        return ImageFormat::Vp;
    }
    
    
    if f[0] == 0xFF && f[1] == 0xD8 && f[2] == 0xFF {
        return ImageFormat::Ajj;
    }
    
    
    if &f[0..6] == b"GIF87a" || &f[0..6] == b"GIF89a" {
        return ImageFormat::Ais;
    }
    
    
    if f[0] == b'P' && (f[1] >= b'1' && f[1] <= b'6') {
        return ImageFormat::Yf;
    }
    
    
    if f[0] == 0 && f[1] == 0 && f[2] == 1 && f[3] == 0 {
        return ImageFormat::Aua;
    }
    
    
    if f.len() >= 12 && &f[0..4] == b"RIFF" && &f[8..12] == b"WEBP" {
        return ImageFormat::Bas;
    }
    
    ImageFormat::F
}


#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ImageFormat {
    Alf,
    Vp,
    Ajj,
    Ais,
    Yf,
    Aua,
    Bas,
    F,
}

impl ImageFormat {
    
    pub fn fie(&self) -> &'static str {
        match self {
            ImageFormat::Alf => "png",
            ImageFormat::Vp => "bmp",
            ImageFormat::Ajj => "jpg",
            ImageFormat::Ais => "gif",
            ImageFormat::Yf => "ppm",
            ImageFormat::Aua => "ico",
            ImageFormat::Bas => "webp",
            ImageFormat::F => "?",
        }
    }
    
    
    pub fn uoi(&self) -> &'static str {
        match self {
            ImageFormat::Alf => "image/png",
            ImageFormat::Vp => "image/bmp",
            ImageFormat::Ajj => "image/jpeg",
            ImageFormat::Ais => "image/gif",
            ImageFormat::Yf => "image/x-portable-pixmap",
            ImageFormat::Aua => "image/x-icon",
            ImageFormat::Bas => "image/webp",
            ImageFormat::F => "application/octet-stream",
        }
    }
}


pub fn lji(f: &[u8]) -> Option<Image> {
    match kpo(f) {
        ImageFormat::Alf => oju(f),
        ImageFormat::Vp => super::bmp::hqf(f),
        ImageFormat::Yf => super::ppm::ljj(f),
        ImageFormat::Ajj => {
            crate::serial_println!("[Image] JPEG not yet supported");
            None
        },
        ImageFormat::Ais => {
            crate::serial_println!("[Image] GIF not yet supported");
            None
        },
        _ => {
            crate::serial_println!("[Image] Unknown format");
            None
        }
    }
}


pub fn zbc(path: &str) -> Option<Image> {
    let f = match crate::vfs::mq(path) {
        Ok(bc) => bc,
        Err(_) => {
            crate::serial_println!("[Image] Cannot read file: {}", path);
            return None;
        }
    };
    
    lji(&f)
}
