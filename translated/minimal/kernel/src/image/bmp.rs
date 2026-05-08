







use alloc::vec::Vec;
use super::Image;


pub fn ete(path: &str) -> Option<Image> {
    let data = match crate::vfs::read_file(path) {
        Ok(d) => d,
        Err(_) => {
            crate::serial_println!("[BMP] Cannot read file: {}", path);
            return None;
        }
    };
    
    dtq(&data)
}


pub fn dtq(data: &[u8]) -> Option<Image> {
    
    if data.len() < 54 {
        crate::serial_println!("[BMP] File too small");
        return None;
    }
    
    
    if data[0] != b'B' || data[1] != b'M' {
        crate::serial_println!("[BMP] Invalid signature");
        return None;
    }
    
    
    let data_offset = read_u32(&data[10..14]) as usize;
    let pwz = read_u32(&data[14..18]);
    let width = read_i32(&data[18..22]);
    let height = read_i32(&data[22..26]);
    let atc = read_u16(&data[28..30]);
    let compression = read_u32(&data[30..34]);
    
    
    if width <= 0 || width > 8192 {
        crate::serial_println!("[BMP] Invalid width: {}", width);
        return None;
    }
    
    let ckl = height.abs();
    if ckl <= 0 || ckl > 8192 {
        crate::serial_println!("[BMP] Invalid height: {}", height);
        return None;
    }
    
    
    if compression != 0 && compression != 3 {
        crate::serial_println!("[BMP] Unsupported compression: {}", compression);
        return None;
    }
    
    
    if atc != 24 && atc != 32 {
        crate::serial_println!("[BMP] Unsupported bit depth: {}", atc);
        return None;
    }
    
    let width = width as u32;
    let bgy = ckl as u32;
    let ged = height < 0;
    
    
    let awm = atc as usize / 8;
    let bvm = ((width as usize * awm + 3) / 4) * 4;
    
    
    let cod = (width * bgy) as usize;
    let mut pixels = Vec::with_capacity(cod);
    
    
    let tm = &data[data_offset..];
    
    for row in 0..bgy {
        
        let amv = if ged { row } else { bgy - 1 - row };
        let fk = amv as usize * bvm;
        
        for col in 0..width {
            let aee = fk + col as usize * awm;
            
            if aee + awm <= tm.len() {
                
                let b = tm[aee] as u32;
                let g = tm[aee + 1] as u32;
                let r = tm[aee + 2] as u32;
                let a = if atc == 32 {
                    tm[aee + 3] as u32
                } else {
                    255
                };
                
                
                let ct = (a << 24) | (r << 16) | (g << 8) | b;
                pixels.push(ct);
            } else {
                pixels.push(0xFF000000); 
            }
        }
    }
    
    crate::serial_println!("[BMP] Loaded {}x{} image ({}-bit)", width, bgy, atc);
    
    Some(Image::cjv(width, bgy, pixels))
}


fn read_u32(data: &[u8]) -> u32 {
    u32::from_le_bytes([data[0], data[1], data[2], data[3]])
}


fn read_i32(data: &[u8]) -> i32 {
    i32::from_le_bytes([data[0], data[1], data[2], data[3]])
}


fn read_u16(data: &[u8]) -> u16 {
    u16::from_le_bytes([data[0], data[1]])
}






pub fn qur(iv: &Image, path: &str) -> Result<(), &'static str> {
    let data = hom(iv);
    crate::vfs::write_file(path, &data).map_err(|_| "Failed to write file")
}


pub fn hom(iv: &Image) -> Vec<u8> {
    kzd(iv.width, iv.height, &iv.pixels)
}


pub fn kzd(width: u32, height: u32, pixels: &[u32]) -> Vec<u8> {
    let awm = 3; 
    let bvm = ((width as usize * awm + 3) / 4) * 4;
    let dwo = bvm * height as usize;
    let file_size = 54 + dwo;
    
    let mut bmp = Vec::with_capacity(file_size);
    
    
    bmp.extend_from_slice(&[b'B', b'M']);
    bmp.extend_from_slice(&(file_size as u32).to_le_bytes());
    bmp.extend_from_slice(&[0u8; 4]); 
    bmp.extend_from_slice(&54u32.to_le_bytes()); 
    
    
    bmp.extend_from_slice(&40u32.to_le_bytes()); 
    bmp.extend_from_slice(&(width as i32).to_le_bytes());
    bmp.extend_from_slice(&(height as i32).to_le_bytes()); 
    bmp.extend_from_slice(&1u16.to_le_bytes()); 
    bmp.extend_from_slice(&24u16.to_le_bytes()); 
    bmp.extend_from_slice(&0u32.to_le_bytes()); 
    bmp.extend_from_slice(&(dwo as u32).to_le_bytes());
    bmp.extend_from_slice(&[0u8; 16]); 
    
    
    for row in (0..height).rev() {
        for col in 0..width {
            let ct = pixels[(row * width + col) as usize];
            let r = ((ct >> 16) & 0xFF) as u8;
            let g = ((ct >> 8) & 0xFF) as u8;
            let b = (ct & 0xFF) as u8;
            bmp.push(b);
            bmp.push(g);
            bmp.push(r);
        }
        
        let padding = bvm - (width as usize * 3);
        for _ in 0..padding {
            bmp.push(0);
        }
    }
    
    bmp
}
