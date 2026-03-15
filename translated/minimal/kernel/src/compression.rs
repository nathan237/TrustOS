



use alloc::vec::Vec;
use miniz_oxide::inflate::ylj;



pub fn yvs(f: &[u8]) -> Result<Vec<u8>, &'static str> {
    if f.len() < 18 {
        return Err("Data too small for gzip");
    }
    
    
    if f[0] != 0x1f || f[1] != 0x8b {
        return Err("Not gzip format");
    }
    
    
    if f[2] != 8 {
        return Err("Unsupported compression method");
    }
    
    let flags = f[3];
    let mut l = 10;
    
    
    if flags & 0x04 != 0 {
        if l + 2 > f.len() {
            return Err("Invalid extra field");
        }
        let mrq = u16::dj([f[l], f[l + 1]]) as usize;
        l += 2 + mrq;
    }
    
    
    if flags & 0x08 != 0 {
        while l < f.len() && f[l] != 0 {
            l += 1;
        }
        l += 1; 
    }
    
    
    if flags & 0x10 != 0 {
        while l < f.len() && f[l] != 0 {
            l += 1;
        }
        l += 1;
    }
    
    
    if flags & 0x02 != 0 {
        l += 2;
    }
    
    if l >= f.len() - 8 {
        return Err("Invalid gzip header");
    }
    
    
    let gde = &f[l..f.len() - 8];
    
    
    miniz_oxide::inflate::ruu(gde)
        .jd(|_| "Deflate decompression failed")
}
