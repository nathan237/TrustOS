










use alloc::vec::Vec;
use super::Image;


const BFL_: [u8; 8] = [0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];


pub fn qnu(path: &str) -> Option<Image> {
    let data = match crate::vfs::read_file(path) {
        Ok(d) => d,
        Err(_) => {
            crate::serial_println!("[PNG] Cannot read file: {}", path);
            return None;
        }
    };
    
    ikv(&data)
}


pub fn ikv(data: &[u8]) -> Option<Image> {
    
    if data.len() < 8 {
        crate::serial_println!("[PNG] File too small");
        return None;
    }
    
    
    if &data[0..8] != &BFL_ {
        crate::serial_println!("[PNG] Invalid signature");
        return None;
    }
    
    
    let mut pos = 8usize;
    let mut width: u32 = 0;
    let mut height: u32 = 0;
    let mut egw: u8 = 0;
    let mut eiq: u8 = 0;
    let mut ihc: u8 = 0;
    let mut cvm: Vec<u8> = Vec::new();
    
    while pos + 12 <= data.len() {
        let ehx = gqi(&data[pos..pos+4]) as usize;
        let kkl = &data[pos+4..pos+8];
        let cvb = &data[pos+8..pos+8+ehx.min(data.len() - pos - 8)];
        
        match kkl {
            b"IHDR" => {
                if ehx >= 13 {
                    width = gqi(&cvb[0..4]);
                    height = gqi(&cvb[4..8]);
                    egw = cvb[8];
                    eiq = cvb[9];
                    
                    
                    ihc = cvb[12];
                    
                    crate::serial_println!("[PNG] {}x{} depth={} color_type={} interlace={}", 
                        width, height, egw, eiq, ihc);
                }
            },
            b"IDAT" => {
                
                cvm.extend_from_slice(cvb);
            },
            b"IEND" => {
                break;
            },
            b"PLTE" => {
                
                crate::serial_println!("[PNG] Palette chunk found ({} bytes)", ehx);
            },
            _ => {
                
            }
        }
        
        pos += 12 + ehx; 
    }
    
    
    if width == 0 || height == 0 || width > 8192 || height > 8192 {
        crate::serial_println!("[PNG] Invalid dimensions: {}x{}", width, height);
        return None;
    }
    
    
    if egw != 8 {
        crate::serial_println!("[PNG] Unsupported bit depth: {} (only 8-bit supported)", egw);
        return None;
    }
    
    
    let channels = match eiq {
        0 => 1, 
        2 => 3, 
        4 => 2, 
        6 => 4, 
        _ => {
            crate::serial_println!("[PNG] Unsupported color type: {}", eiq);
            return None;
        }
    };
    
    
    let blr = match lcv(&cvm) {
        Some(d) => d,
        None => {
            crate::serial_println!("[PNG] Decompression failed");
            return None;
        }
    };
    
    
    let cpp = width as usize * channels;
    let cxj = height as usize * (1 + cpp);
    
    if blr.len() < cxj {
        crate::serial_println!("[PNG] Decompressed size mismatch: {} < {}", blr.len(), cxj);
        return None;
    }
    
    
    let pixels = lck(&blr, width, height, channels)?;
    
    Some(Image::cjv(width, height, pixels))
}


fn lck(data: &[u8], width: u32, height: u32, channels: usize) -> Option<Vec<u32>> {
    let cpp = width as usize * channels;
    let mut pixels = Vec::with_capacity((width * height) as usize);
    
    
    let mut prev_row: Vec<u8> = alloc::vec![0u8; cpp];
    let mut aws: Vec<u8> = alloc::vec![0u8; cpp];
    
    for y in 0..height as usize {
        let fk = y * (1 + cpp);
        let fwx = data[fk];
        let obq = &data[fk + 1..fk + 1 + cpp];
        
        
        for x in 0..cpp {
            let dm = obq[x];
            let a = if x >= channels { aws[x - channels] } else { 0 }; 
            let b = prev_row[x]; 
            let c = if x >= channels { prev_row[x - channels] } else { 0 }; 
            
            aws[x] = match fwx {
                0 => dm, 
                1 => dm.wrapping_add(a), 
                2 => dm.wrapping_add(b), 
                3 => dm.wrapping_add(((a as u16 + b as u16) / 2) as u8), 
                4 => dm.wrapping_add(npi(a, b, c)), 
                _ => dm,
            };
        }
        
        
        for x in 0..width as usize {
            let aee = x * channels;
            let ct = match channels {
                1 => {
                    
                    let g = aws[aee] as u32;
                    0xFF000000 | (g << 16) | (g << 8) | g
                },
                2 => {
                    
                    let g = aws[aee] as u32;
                    let a = aws[aee + 1] as u32;
                    (a << 24) | (g << 16) | (g << 8) | g
                },
                3 => {
                    
                    let r = aws[aee] as u32;
                    let g = aws[aee + 1] as u32;
                    let b = aws[aee + 2] as u32;
                    0xFF000000 | (r << 16) | (g << 8) | b
                },
                4 => {
                    
                    let r = aws[aee] as u32;
                    let g = aws[aee + 1] as u32;
                    let b = aws[aee + 2] as u32;
                    let a = aws[aee + 3] as u32;
                    (a << 24) | (r << 16) | (g << 8) | b
                },
                _ => 0xFF000000,
            };
            pixels.push(ct);
        }
        
        
        core::mem::swap(&mut prev_row, &mut aws);
    }
    
    Some(pixels)
}


fn npi(a: u8, b: u8, c: u8) -> u8 {
    let aa = a as i16 + b as i16 - c as i16;
    let pa = (aa - a as i16).abs();
    let ji = (aa - b as i16).abs();
    let pc = (aa - c as i16).abs();
    
    if pa <= ji && pa <= pc {
        a
    } else if ji <= pc {
        b
    } else {
        c
    }
}


fn gqi(data: &[u8]) -> u32 {
    ((data[0] as u32) << 24) |
    ((data[1] as u32) << 16) |
    ((data[2] as u32) << 8) |
    (data[3] as u32)
}


fn lcv(data: &[u8]) -> Option<Vec<u8>> {
    
    if data.len() < 6 {
        return None;
    }
    
    
    let kul = data[0];
    let pwx = data[1];
    
    
    if kul & 0x0F != 8 {
        crate::serial_println!("[PNG] Invalid zlib compression method");
        return None;
    }
    
    
    let lda = &data[2..data.len().saturating_sub(4)]; 
    
    
    match miniz_oxide::inflate::decompress_to_vec(lda) {
        Ok(blr) => Some(blr),
        Err(e) => {
            crate::serial_println!("[PNG] Deflate error: {:?}", e);
            None
        }
    }
}






pub fn frw(data: &[u8]) -> ImageFormat {
    if data.len() < 8 {
        return ImageFormat::Unknown;
    }
    
    
    if &data[0..8] == &BFL_ {
        return ImageFormat::Png;
    }
    
    
    if data[0] == b'B' && data[1] == b'M' {
        return ImageFormat::Bmp;
    }
    
    
    if data[0] == 0xFF && data[1] == 0xD8 && data[2] == 0xFF {
        return ImageFormat::Jpeg;
    }
    
    
    if &data[0..6] == b"GIF87a" || &data[0..6] == b"GIF89a" {
        return ImageFormat::Gif;
    }
    
    
    if data[0] == b'P' && (data[1] >= b'1' && data[1] <= b'6') {
        return ImageFormat::Ppm;
    }
    
    
    if data[0] == 0 && data[1] == 0 && data[2] == 1 && data[3] == 0 {
        return ImageFormat::Ico;
    }
    
    
    if data.len() >= 12 && &data[0..4] == b"RIFF" && &data[8..12] == b"WEBP" {
        return ImageFormat::WebP;
    }
    
    ImageFormat::Unknown
}


#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ImageFormat {
    Png,
    Bmp,
    Jpeg,
    Gif,
    Ppm,
    Ico,
    WebP,
    Unknown,
}

impl ImageFormat {
    
    pub fn extension(&self) -> &'static str {
        match self {
            ImageFormat::Png => "png",
            ImageFormat::Bmp => "bmp",
            ImageFormat::Jpeg => "jpg",
            ImageFormat::Gif => "gif",
            ImageFormat::Ppm => "ppm",
            ImageFormat::Ico => "ico",
            ImageFormat::WebP => "webp",
            ImageFormat::Unknown => "?",
        }
    }
    
    
    pub fn mime_type(&self) -> &'static str {
        match self {
            ImageFormat::Png => "image/png",
            ImageFormat::Bmp => "image/bmp",
            ImageFormat::Jpeg => "image/jpeg",
            ImageFormat::Gif => "image/gif",
            ImageFormat::Ppm => "image/x-portable-pixmap",
            ImageFormat::Ico => "image/x-icon",
            ImageFormat::WebP => "image/webp",
            ImageFormat::Unknown => "application/octet-stream",
        }
    }
}


pub fn gfy(data: &[u8]) -> Option<Image> {
    match frw(data) {
        ImageFormat::Png => ikv(data),
        ImageFormat::Bmp => super::bmp::dtq(data),
        ImageFormat::Ppm => super::ppm::gfz(data),
        ImageFormat::Jpeg => {
            crate::serial_println!("[Image] JPEG not yet supported");
            None
        },
        ImageFormat::Gif => {
            crate::serial_println!("[Image] GIF not yet supported");
            None
        },
        _ => {
            crate::serial_println!("[Image] Unknown format");
            None
        }
    }
}


pub fn qnr(path: &str) -> Option<Image> {
    let data = match crate::vfs::read_file(path) {
        Ok(d) => d,
        Err(_) => {
            crate::serial_println!("[Image] Cannot read file: {}", path);
            return None;
        }
    };
    
    gfy(&data)
}
