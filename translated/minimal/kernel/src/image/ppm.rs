















use alloc::vec::Vec;
use super::Image;


pub fn ojv(path: &str) -> Option<Image> {
    let f = match crate::vfs::mq(path) {
        Ok(bc) => bc,
        Err(_) => {
            crate::serial_println!("[PPM] Cannot read file: {}", path);
            return None;
        }
    };
    
    ljj(&f)
}


pub fn ljj(f: &[u8]) -> Option<Image> {
    if f.len() < 7 {
        return None;
    }
    
    
    if f[0] != b'P' {
        return None;
    }
    
    match f[1] {
        b'3' => vdb(&f[2..]),  
        b'6' => vdc(&f[2..]),  
        _ => None,
    }
}


fn vdb(f: &[u8]) -> Option<Image> {
    let text = core::str::jg(f).bq()?;
    let mut eb = text.ayt()
        .hi(|e| !e.cj('#')); 
    
    let z: u32 = eb.next()?.parse().bq()?;
    let ac: u32 = eb.next()?.parse().bq()?;
    let aki: u32 = eb.next()?.parse().bq()?;
    
    if z == 0 || ac == 0 || z > 8192 || ac > 8192 {
        return None;
    }
    
    let mut hz = Vec::fc((z * ac) as usize);
    
    for _ in 0..(z * ac) {
        let m: u32 = eb.next()?.parse().bq()?;
        let at: u32 = eb.next()?.parse().bq()?;
        let o: u32 = eb.next()?.parse().bq()?;
        
        
        let m = if aki == 255 { m } else { m * 255 / aki };
        let at = if aki == 255 { at } else { at * 255 / aki };
        let o = if aki == 255 { o } else { o * 255 / aki };
        
        let il = 0xFF000000 | (m << 16) | (at << 8) | o;
        hz.push(il);
    }
    
    crate::serial_println!("[PPM] Loaded P3 {}x{} image", z, ac);
    Some(Image::fjd(z, ac, hz))
}


fn vdc(f: &[u8]) -> Option<Image> {
    
    let mut u = 0;
    let mut z = 0u32;
    let mut ac = 0u32;
    let mut aki = 0u32;
    let mut lbv = 0; 
    let mut ldt = false;
    let mut orm = [0u8; 16];
    let mut gof = 0;
    
    while u < f.len() && lbv < 3 {
        let r = f[u];
        u += 1;
        
        if ldt {
            if r == b'\n' { ldt = false; }
            continue;
        }
        
        if r == b'#' {
            ldt = true;
            continue;
        }
        
        if r.yyy() {
            if gof > 0 {
                let ajh = core::str::jg(&orm[..gof]).bq()?;
                let num: u32 = ajh.parse().bq()?;
                gof = 0;
                
                match lbv {
                    0 => z = num,
                    1 => ac = num,
                    2 => aki = num,
                    _ => {}
                }
                lbv += 1;
            }
            continue;
        }
        
        if r.atb() && gof < 16 {
            orm[gof] = r;
            gof += 1;
        }
    }
    
    if z == 0 || ac == 0 || z > 8192 || ac > 8192 {
        return None;
    }
    
    
    let amn = &f[u..];
    let coy = if aki > 255 { 6 } else { 3 };
    
    if amn.len() < (z * ac) as usize * coy {
        return None;
    }
    
    let mut hz = Vec::fc((z * ac) as usize);
    let mut a = 0;
    
    for _ in 0..(z * ac) {
        let (m, at, o) = if aki > 255 {
            
            let m = ((amn[a] as u32) << 8 | amn[a+1] as u32) * 255 / aki;
            let at = ((amn[a+2] as u32) << 8 | amn[a+3] as u32) * 255 / aki;
            let o = ((amn[a+4] as u32) << 8 | amn[a+5] as u32) * 255 / aki;
            a += 6;
            (m, at, o)
        } else {
            
            let m = if aki == 255 { amn[a] as u32 } else { amn[a] as u32 * 255 / aki };
            let at = if aki == 255 { amn[a+1] as u32 } else { amn[a+1] as u32 * 255 / aki };
            let o = if aki == 255 { amn[a+2] as u32 } else { amn[a+2] as u32 * 255 / aki };
            a += 3;
            (m, at, o)
        };
        
        let il = 0xFF000000 | (m << 16) | (at << 8) | o;
        hz.push(il);
    }
    
    crate::serial_println!("[PPM] Loaded P6 {}x{} image", z, ac);
    Some(Image::fjd(z, ac, hz))
}






pub fn zlq(th: &Image, path: &str) -> Result<(), &'static str> {
    let f = rqr(th);
    crate::vfs::ns(path, &f).jd(|_| "Failed to write file")
}


pub fn rqr(th: &Image) -> Vec<u8> {
    use alloc::format;
    
    let dh = format!("P6\n{} {}\n255\n", th.z, th.ac);
    let mut f = Vec::fc(dh.len() + (th.z * th.ac * 3) as usize);
    
    f.bk(dh.as_bytes());
    
    for il in &th.hz {
        let m = ((il >> 16) & 0xFF) as u8;
        let at = ((il >> 8) & 0xFF) as u8;
        let o = (il & 0xFF) as u8;
        f.push(m);
        f.push(at);
        f.push(o);
    }
    
    f
}
