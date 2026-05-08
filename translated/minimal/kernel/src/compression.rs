



use alloc::vec::Vec;
use miniz_oxide::inflate::decompress_to_vec_zlib;



pub fn qka(data: &[u8]) -> Result<Vec<u8>, &'static str> {
    if data.len() < 18 {
        return Err("Data too small for gzip");
    }
    
    
    if data[0] != 0x1f || data[1] != 0x8b {
        return Err("Not gzip format");
    }
    
    
    if data[2] != 8 {
        return Err("Unsupported compression method");
    }
    
    let flags = data[3];
    let mut offset = 10;
    
    
    if flags & 0x04 != 0 {
        if offset + 2 > data.len() {
            return Err("Invalid extra field");
        }
        let hcy = u16::from_le_bytes([data[offset], data[offset + 1]]) as usize;
        offset += 2 + hcy;
    }
    
    
    if flags & 0x08 != 0 {
        while offset < data.len() && data[offset] != 0 {
            offset += 1;
        }
        offset += 1; 
    }
    
    
    if flags & 0x10 != 0 {
        while offset < data.len() && data[offset] != 0 {
            offset += 1;
        }
        offset += 1;
    }
    
    
    if flags & 0x02 != 0 {
        offset += 2;
    }
    
    if offset >= data.len() - 8 {
        return Err("Invalid gzip header");
    }
    
    
    let cvm = &data[offset..data.len() - 8];
    
    
    miniz_oxide::inflate::decompress_to_vec(cvm)
        .map_err(|_| "Deflate decompression failed")
}
