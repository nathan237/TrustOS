















use alloc::vec::Vec;
use super::Image;


pub fn ikw(path: &str) -> Option<Image> {
    let data = match crate::vfs::read_file(path) {
        Ok(d) => d,
        Err(_) => {
            crate::serial_println!("[PPM] Cannot read file: {}", path);
            return None;
        }
    };
    
    gfz(&data)
}


pub fn gfz(data: &[u8]) -> Option<Image> {
    if data.len() < 7 {
        return None;
    }
    
    
    if data[0] != b'P' {
        return None;
    }
    
    match data[1] {
        b'3' => nqu(&data[2..]),  
        b'6' => nqv(&data[2..]),  
        _ => None,
    }
}


fn nqu(data: &[u8]) -> Option<Image> {
    let text = core::str::from_utf8(data).ok()?;
    let mut tokens = text.split_whitespace()
        .filter(|j| !j.starts_with('#')); 
    
    let width: u32 = tokens.next()?.parse().ok()?;
    let height: u32 = tokens.next()?.parse().ok()?;
    let sh: u32 = tokens.next()?.parse().ok()?;
    
    if width == 0 || height == 0 || width > 8192 || height > 8192 {
        return None;
    }
    
    let mut pixels = Vec::with_capacity((width * height) as usize);
    
    for _ in 0..(width * height) {
        let r: u32 = tokens.next()?.parse().ok()?;
        let g: u32 = tokens.next()?.parse().ok()?;
        let b: u32 = tokens.next()?.parse().ok()?;
        
        
        let r = if sh == 255 { r } else { r * 255 / sh };
        let g = if sh == 255 { g } else { g * 255 / sh };
        let b = if sh == 255 { b } else { b * 255 / sh };
        
        let ct = 0xFF000000 | (r << 16) | (g << 8) | b;
        pixels.push(ct);
    }
    
    crate::serial_println!("[PPM] Loaded P3 {}x{} image", width, height);
    Some(Image::cjv(width, height, pixels))
}


fn nqv(data: &[u8]) -> Option<Image> {
    
    let mut pos = 0;
    let mut width = 0u32;
    let mut height = 0u32;
    let mut sh = 0u32;
    let mut gal = 0; 
    let mut gcf = false;
    let mut iri = [0u8; 16];
    let mut dbx = 0;
    
    while pos < data.len() && gal < 3 {
        let c = data[pos];
        pos += 1;
        
        if gcf {
            if c == b'\n' { gcf = false; }
            continue;
        }
        
        if c == b'#' {
            gcf = true;
            continue;
        }
        
        if c.is_ascii_whitespace() {
            if dbx > 0 {
                let rw = core::str::from_utf8(&iri[..dbx]).ok()?;
                let num: u32 = rw.parse().ok()?;
                dbx = 0;
                
                match gal {
                    0 => width = num,
                    1 => height = num,
                    2 => sh = num,
                    _ => {}
                }
                gal += 1;
            }
            continue;
        }
        
        if c.is_ascii_digit() && dbx < 16 {
            iri[dbx] = c;
            dbx += 1;
        }
    }
    
    if width == 0 || height == 0 || width > 8192 || height > 8192 {
        return None;
    }
    
    
    let tm = &data[pos..];
    let awm = if sh > 255 { 6 } else { 3 };
    
    if tm.len() < (width * height) as usize * awm {
        return None;
    }
    
    let mut pixels = Vec::with_capacity((width * height) as usize);
    let mut i = 0;
    
    for _ in 0..(width * height) {
        let (r, g, b) = if sh > 255 {
            
            let r = ((tm[i] as u32) << 8 | tm[i+1] as u32) * 255 / sh;
            let g = ((tm[i+2] as u32) << 8 | tm[i+3] as u32) * 255 / sh;
            let b = ((tm[i+4] as u32) << 8 | tm[i+5] as u32) * 255 / sh;
            i += 6;
            (r, g, b)
        } else {
            
            let r = if sh == 255 { tm[i] as u32 } else { tm[i] as u32 * 255 / sh };
            let g = if sh == 255 { tm[i+1] as u32 } else { tm[i+1] as u32 * 255 / sh };
            let b = if sh == 255 { tm[i+2] as u32 } else { tm[i+2] as u32 * 255 / sh };
            i += 3;
            (r, g, b)
        };
        
        let ct = 0xFF000000 | (r << 16) | (g << 8) | b;
        pixels.push(ct);
    }
    
    crate::serial_println!("[PPM] Loaded P6 {}x{} image", width, height);
    Some(Image::cjv(width, height, pixels))
}






pub fn qus(iv: &Image, path: &str) -> Result<(), &'static str> {
    let data = kzj(iv);
    crate::vfs::write_file(path, &data).map_err(|_| "Failed to write file")
}


pub fn kzj(iv: &Image) -> Vec<u8> {
    use alloc::format;
    
    let header = format!("P6\n{} {}\n255\n", iv.width, iv.height);
    let mut data = Vec::with_capacity(header.len() + (iv.width * iv.height * 3) as usize);
    
    data.extend_from_slice(header.as_bytes());
    
    for ct in &iv.pixels {
        let r = ((ct >> 16) & 0xFF) as u8;
        let g = ((ct >> 8) & 0xFF) as u8;
        let b = (ct & 0xFF) as u8;
        data.push(r);
        data.push(g);
        data.push(b);
    }
    
    data
}
