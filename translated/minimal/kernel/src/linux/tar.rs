



use alloc::string::String;
use alloc::vec::Vec;
use alloc::vec;
use alloc::format;


#[repr(C)]
struct Beb {
    name: [u8; 100],
    mode: [u8; 8],
    uid: [u8; 8],
    gid: [u8; 8],
    size: [u8; 12],
    mtime: [u8; 12],
    chksum: [u8; 8],
    joz: u8,
    linkname: [u8; 100],
    magic: [u8; 6],
    version: [u8; 2],
    asq: [u8; 32],
    gname: [u8; 32],
    devmajor: [u8; 8],
    devminor: [u8; 8],
    nm: [u8; 155],
    _pad: [u8; 12],
}

const HX_: usize = 512;


const Aoa: u8 = b'0';
const Agw: u8 = 0;
const Aih: u8 = b'5';
const Aqb: u8 = b'2';


pub fn ltv(tarball_path: &str, mt: &str) -> Result<usize, &'static str> {
    
    let qv = crate::ramfs::bh(|fs| {
        fs.read_file(tarball_path)
            .map(|data| data.to_vec())
            .map_err(|_| "tarball not found")
    })?;
    
    
    let blr = lcu(&qv)?;
    
    
    ltu(&blr, mt)
}


fn lcu(data: &[u8]) -> Result<Vec<u8>, &'static str> {
    
    if data.len() < 10 || data[0] != 0x1f || data[1] != 0x8b {
        return Err("not a gzip file");
    }
    
    
    if data[2] != 8 {
        return Err("unsupported compression method");
    }
    
    let flags = data[3];
    let mut pos = 10;
    
    
    if flags & 0x04 != 0 {
        if pos + 2 > data.len() {
            return Err("truncated gzip");
        }
        let hcy = (data[pos] as usize) | ((data[pos + 1] as usize) << 8);
        pos += 2 + hcy;
    }
    
    
    if flags & 0x08 != 0 {
        while pos < data.len() && data[pos] != 0 {
            pos += 1;
        }
        pos += 1;
    }
    
    
    if flags & 0x10 != 0 {
        while pos < data.len() && data[pos] != 0 {
            pos += 1;
        }
        pos += 1;
    }
    
    
    if flags & 0x02 != 0 {
        pos += 2;
    }
    
    if pos >= data.len() {
        return Err("truncated gzip header");
    }
    
    
    let len = data.len();
    if len < 8 {
        return Err("truncated gzip");
    }
    let iss = (data[len - 4] as usize)
        | ((data[len - 3] as usize) << 8)
        | ((data[len - 2] as usize) << 16)
        | ((data[len - 1] as usize) << 24);
    
    
    let cvm = &data[pos..len - 8];
    
    
    let mut output = Vec::with_capacity(iss);
    
    
    match mor(cvm, &mut output, iss) {
        Ok(()) => Ok(output),
        Err(e) => Err(e),
    }
}


fn mor(input: &[u8], output: &mut Vec<u8>, cxj: usize) -> Result<(), &'static str> {
    
    let mut aaq = DeflateDecoder::new(input);
    
    loop {
        match aaq.decode_block(output)? {
            BlockResult::Continue => continue,
            BlockResult::Done => break,
        }
        
        
        if output.len() > cxj + 1024 * 1024 {
            return Err("decompression overflow");
        }
    }
    
    Ok(())
}

enum BlockResult {
    Continue,
    Done,
}

struct DeflateDecoder<'a> {
    input: &'a [u8],
    pos: usize,
    bit_buf: u32,
    bit_count: u8,
}

impl<'a> DeflateDecoder<'a> {
    fn new(input: &'a [u8]) -> Self {
        Self {
            input,
            pos: 0,
            bit_buf: 0,
            bit_count: 0,
        }
    }
    
    fn read_bits(&mut self, ae: u8) -> Result<u32, &'static str> {
        while self.bit_count < ae {
            if self.pos >= self.input.len() {
                return Err("unexpected end of input");
            }
            self.bit_buf |= (self.input[self.pos] as u32) << self.bit_count;
            self.pos += 1;
            self.bit_count += 8;
        }
        
        let result = self.bit_buf & ((1 << ae) - 1);
        self.bit_buf >>= ae;
        self.bit_count -= ae;
        Ok(result)
    }
    
    fn decode_block(&mut self, output: &mut Vec<u8>) -> Result<BlockResult, &'static str> {
        let kbm = self.read_bits(1)?;
        let kel = self.read_bits(2)?;
        
        match kel {
            0 => {
                
                self.bit_buf = 0;
                self.bit_count = 0;
                
                if self.pos + 4 > self.input.len() {
                    return Err("truncated stored block");
                }
                
                let len = (self.input[self.pos] as u16) | ((self.input[self.pos + 1] as u16) << 8);
                self.pos += 4;
                
                let len = len as usize;
                if self.pos + len > self.input.len() {
                    return Err("truncated stored block data");
                }
                
                output.extend_from_slice(&self.input[self.pos..self.pos + len]);
                self.pos += len;
            }
            1 => {
                
                self.decode_huffman_block(output, true)?;
            }
            2 => {
                
                self.decode_huffman_block(output, false)?;
            }
            _ => return Err("invalid block type"),
        }
        
        if kbm != 0 {
            Ok(BlockResult::Done)
        } else {
            Ok(BlockResult::Continue)
        }
    }
    
    fn decode_huffman_block(&mut self, output: &mut Vec<u8>, fixed: bool) -> Result<(), &'static str> {
        
        let (bty, dni) = if fixed {
            Self::lwo()
        } else {
            self.read_dynamic_tables()?
        };
        
        loop {
            let sym = self.decode_symbol(&bty)?;
            
            if sym < 256 {
                output.push(sym as u8);
            } else if sym == 256 {
                break;
            } else {
                
                let length = self.decode_length(sym)?;
                let lga = self.decode_symbol(&dni)?;
                let byu = self.decode_distance(lga)?;
                
                if byu > output.len() {
                    return Err("invalid back reference");
                }
                
                let start = output.len() - byu;
                for i in 0..length {
                    let byte = output[start + (i % byu)];
                    output.push(byte);
                }
            }
        }
        
        Ok(())
    }
    
    fn lwo() -> (Vec<u8>, Vec<u8>) {
        let mut bty = vec![0u8; 288];
        for i in 0..144 { bty[i] = 8; }
        for i in 144..256 { bty[i] = 9; }
        for i in 256..280 { bty[i] = 7; }
        for i in 280..288 { bty[i] = 8; }
        
        let dni = vec![5u8; 32];
        
        (bty, dni)
    }
    
    fn read_dynamic_tables(&mut self) -> Result<(Vec<u8>, Vec<u8>), &'static str> {
        let eph = self.read_bits(5)? as usize + 257;
        let ieh = self.read_bits(5)? as usize + 1;
        let mkl = self.read_bits(4)? as usize + 4;
        
        const BPM_: [usize; 19] = [16, 17, 18, 0, 8, 7, 9, 6, 10, 5, 11, 4, 12, 3, 13, 2, 14, 1, 15];
        
        let mut cvg = [0u8; 19];
        for i in 0..mkl {
            cvg[BPM_[i]] = self.read_bits(3)? as u8;
        }
        
        let kup = cvg.to_vec();
        
        
        let mut bxk = Vec::with_capacity(eph + ieh);
        while bxk.len() < eph + ieh {
            let sym = self.decode_symbol(&kup)?;
            match sym {
                0..=15 => bxk.push(sym as u8),
                16 => {
                    let count = self.read_bits(2)? as usize + 3;
                    let last = *bxk.last().ok_or("invalid code")?;
                    for _ in 0..count { bxk.push(last); }
                }
                17 => {
                    let count = self.read_bits(3)? as usize + 3;
                    for _ in 0..count { bxk.push(0); }
                }
                18 => {
                    let count = self.read_bits(7)? as usize + 11;
                    for _ in 0..count { bxk.push(0); }
                }
                _ => return Err("invalid code length symbol"),
            }
        }
        
        let bty = bxk[..eph].to_vec();
        let dni = bxk[eph..].to_vec();
        
        Ok((bty, dni))
    }
    
    fn decode_symbol(&mut self, cvg: &[u8]) -> Result<u16, &'static str> {
        
        let mut code = 0u32;
        let mut first = [0u32; 16];
        let mut index = [0u16; 16];
        
        
        let mut count = [0u16; 16];
        for &len in cvg {
            if len > 0 && (len as usize) < 16 {
                count[len as usize] += 1;
            }
        }
        
        
        let mut idx = 0u16;
        for i in 1..16 {
            first[i] = code;
            index[i] = idx;
            code = (code + count[i] as u32) << 1;
            idx += count[i];
        }
        
        
        let mut symbols = vec![0u16; cvg.len()];
        for (sym, &len) in cvg.iter().enumerate() {
            if len > 0 {
                let len = len as usize;
                let sym_idx = index[len] as usize;
                if sym_idx < symbols.len() {
                    symbols[sym_idx] = sym as u16;
                    index[len] += 1;
                }
            }
        }
        
        
        let mut code = 0u32;
        let mut emt = [0u16; 16];
        let mut fxd = [0u32; 16];
        
        idx = 0;
        let mut c = 0u32;
        for i in 1..16 {
            fxd[i] = c;
            emt[i] = idx;
            c = (c + count[i] as u32) << 1;
            idx += count[i];
        }
        
        for len in 1..16 {
            code = (code << 1) | self.read_bits(1)?;
            let cnt = count[len];
            if code < fxd[len] + cnt as u32 {
                let sym_idx = emt[len] + (code - fxd[len]) as u16;
                return Ok(symbols[sym_idx as usize]);
            }
        }
        
        Err("invalid huffman code")
    }
    
    fn decode_length(&mut self, sym: u16) -> Result<usize, &'static str> {
        const BAZ_: [u16; 29] = [3, 4, 5, 6, 7, 8, 9, 10, 11, 13, 15, 17, 19, 23, 27, 31, 35, 43, 51, 59, 67, 83, 99, 115, 131, 163, 195, 227, 258];
        const CHU_: [u8; 29] = [0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 2, 2, 2, 2, 3, 3, 3, 3, 4, 4, 4, 4, 5, 5, 5, 5, 0];
        
        let idx = (sym - 257) as usize;
        if idx >= BAZ_.len() {
            return Err("invalid length code");
        }
        
        let base = BAZ_[idx] as usize;
        let ua = CHU_[idx];
        let fvv = if ua > 0 { self.read_bits(ua)? as usize } else { 0 };
        
        Ok(base + fvv)
    }
    
    fn decode_distance(&mut self, sym: u16) -> Result<usize, &'static str> {
        const ASL_: [u16; 30] = [1, 2, 3, 4, 5, 7, 9, 13, 17, 25, 33, 49, 65, 97, 129, 193, 257, 385, 513, 769, 1025, 1537, 2049, 3073, 4097, 6145, 8193, 12289, 16385, 24577];
        const BUR_: [u8; 30] = [0, 0, 0, 0, 1, 1, 2, 2, 3, 3, 4, 4, 5, 5, 6, 6, 7, 7, 8, 8, 9, 9, 10, 10, 11, 11, 12, 12, 13, 13];
        
        let idx = sym as usize;
        if idx >= ASL_.len() {
            return Err("invalid distance code");
        }
        
        let base = ASL_[idx] as usize;
        let ua = BUR_[idx];
        let fvv = if ua > 0 { self.read_bits(ua)? as usize } else { 0 };
        
        Ok(base + fvv)
    }
}


fn ltu(data: &[u8], mt: &str) -> Result<usize, &'static str> {
    let mut pos = 0;
    let mut adp = 0;
    
    while pos + HX_ <= data.len() {
        let header = &data[pos..pos + HX_];
        
        
        if header.iter().all(|&b| b == 0) {
            break;
        }
        
        
        let name = nrg(header)?;
        let size = nqt(&header[124..136])?;
        let joz = header[156];
        
        
        let nm = ewi(&header[345..500]);
        let cyf = if nm.is_empty() {
            name
        } else {
            format!("{}/{}", nm, name)
        };
        
        
        if cyf.is_empty() || cyf == "." || cyf == "./" {
            pos += HX_;
            pos += fgq(size);
            continue;
        }
        
        
        let bld = cyf.trim_start_matches("./").trim_start_matches('/');
        if bld.is_empty() {
            pos += HX_;
            pos += fgq(size);
            continue;
        }
        
        let bfw = format!("{}/{}", mt, bld);
        
        pos += HX_;
        
        match joz {
            Aih | b'/' => {
                
                crate::ramfs::bh(|fs| {
                    let _ = fs.mkdir(&bfw);
                });
            }
            Aoa | Agw | b'\0' => {
                
                if size > 0 && pos + size <= data.len() {
                    let content = &data[pos..pos + size];
                    
                    
                    let parent = parent_path(&bfw);
                    hvz(&parent);
                    
                    crate::ramfs::bh(|fs| {
                        let _ = fs.touch(&bfw);
                        let _ = fs.write_file(&bfw, content);
                    });
                    
                    adp += 1;
                }
            }
            Aqb => {
                
                let mys = ewi(&header[157..257]);
                crate::ramfs::bh(|fs| {
                    let _ = fs.touch(&bfw);
                    let _ = fs.write_file(&bfw, mys.as_bytes());
                });
            }
            _ => {
                
            }
        }
        
        pos += fgq(size);
    }
    
    Ok(adp)
}

fn nrg(header: &[u8]) -> Result<String, &'static str> {
    Ok(ewi(&header[0..100]))
}

fn ewi(bytes: &[u8]) -> String {
    let end = bytes.iter().position(|&b| b == 0).unwrap_or(bytes.len());
    let cow = String::from_utf8_lossy(&bytes[..end]);
    String::from(cow.trim())
}

fn nqt(bytes: &[u8]) -> Result<usize, &'static str> {
    let j = ewi(bytes);
    if j.is_empty() {
        return Ok(0);
    }
    usize::from_str_radix(&j, 8).map_err(|_| "invalid octal")
}

fn fgq(size: usize) -> usize {
    if size == 0 {
        0
    } else {
        ((size + HX_ - 1) / HX_) * HX_
    }
}

fn parent_path(path: &str) -> String {
    if let Some(pos) = path.rfind('/') {
        if pos == 0 {
            String::from("/")
        } else {
            String::from(&path[..pos])
        }
    } else {
        String::from("/")
    }
}

fn hvz(path: &str) {
    if path == "/" || path.is_empty() {
        return;
    }
    
    crate::ramfs::bh(|fs| {
        if !fs.exists(path) {
            let parent = parent_path(path);
            hvz(&parent);
            let _ = fs.mkdir(path);
        }
    });
}
