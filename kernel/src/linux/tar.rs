//! TAR archive extraction
//!
//! Simple tar.gz extractor for Alpine rootfs.

use alloc::string::String;
use alloc::vec::Vec;
use alloc::vec;
use alloc::format;

/// TAR header (POSIX ustar format)
#[repr(C)]
struct TarHeader {
    name: [u8; 100],
    mode: [u8; 8],
    uid: [u8; 8],
    gid: [u8; 8],
    size: [u8; 12],
    mtime: [u8; 12],
    chksum: [u8; 8],
    typeflag: u8,
    linkname: [u8; 100],
    magic: [u8; 6],
    version: [u8; 2],
    uname: [u8; 32],
    gname: [u8; 32],
    devmajor: [u8; 8],
    devminor: [u8; 8],
    prefix: [u8; 155],
    _pad: [u8; 12],
}

const TAR_BLOCK_SIZE: usize = 512;

/// File types in tar
const REGTYPE: u8 = b'0';
const AREGTYPE: u8 = 0;
const DIRTYPE: u8 = b'5';
const SYMTYPE: u8 = b'2';

/// Extract a .tar.gz file to destination
pub fn extract_tarball(tarball_path: &str, dest: &str) -> Result<usize, &'static str> {
    // Read the tarball
    let compressed = crate::ramfs::with_fs(|fs| {
        fs.read_file(tarball_path)
            .map(|data| data.to_vec())
            .map_err(|_| "tarball not found")
    })?;
    
    // Decompress gzip
    let decompressed = decompress_gzip(&compressed)?;
    
    // Extract tar
    extract_tar(&decompressed, dest)
}

/// Simple gzip decompression
fn decompress_gzip(data: &[u8]) -> Result<Vec<u8>, &'static str> {
    // Check gzip magic
    if data.len() < 10 || data[0] != 0x1f || data[1] != 0x8b {
        return Err("not a gzip file");
    }
    
    // Check compression method (must be deflate = 8)
    if data[2] != 8 {
        return Err("unsupported compression method");
    }
    
    let flags = data[3];
    let mut pos = 10;
    
    // Skip extra field
    if flags & 0x04 != 0 {
        if pos + 2 > data.len() {
            return Err("truncated gzip");
        }
        let xlen = (data[pos] as usize) | ((data[pos + 1] as usize) << 8);
        pos += 2 + xlen;
    }
    
    // Skip filename
    if flags & 0x08 != 0 {
        while pos < data.len() && data[pos] != 0 {
            pos += 1;
        }
        pos += 1;
    }
    
    // Skip comment
    if flags & 0x10 != 0 {
        while pos < data.len() && data[pos] != 0 {
            pos += 1;
        }
        pos += 1;
    }
    
    // Skip header CRC
    if flags & 0x02 != 0 {
        pos += 2;
    }
    
    if pos >= data.len() {
        return Err("truncated gzip header");
    }
    
    // Get original size from last 4 bytes
    let len = data.len();
    if len < 8 {
        return Err("truncated gzip");
    }
    let orig_size = (data[len - 4] as usize)
        | ((data[len - 3] as usize) << 8)
        | ((data[len - 2] as usize) << 16)
        | ((data[len - 1] as usize) << 24);
    
    // Use miniz_oxide for deflate decompression
    let compressed_data = &data[pos..len - 8];
    
    // Allocate output buffer
    let mut output = Vec::with_capacity(orig_size);
    
    // Simple inflate implementation
    match inflate_deflate(compressed_data, &mut output, orig_size) {
        Ok(()) => Ok(output),
        Err(e) => Err(e),
    }
}

/// Simple DEFLATE decompression (RFC 1951)
fn inflate_deflate(input: &[u8], output: &mut Vec<u8>, expected_size: usize) -> Result<(), &'static str> {
    // Use a simple state machine for deflate
    let mut decoder = DeflateDecoder::new(input);
    
    loop {
        match decoder.decode_block(output)? {
            BlockResult::Continue => continue,
            BlockResult::Done => break,
        }
        
        // Safety limit
        if output.len() > expected_size + 1024 * 1024 {
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
    
    fn read_bits(&mut self, n: u8) -> Result<u32, &'static str> {
        while self.bit_count < n {
            if self.pos >= self.input.len() {
                return Err("unexpected end of input");
            }
            self.bit_buf |= (self.input[self.pos] as u32) << self.bit_count;
            self.pos += 1;
            self.bit_count += 8;
        }
        
        let result = self.bit_buf & ((1 << n) - 1);
        self.bit_buf >>= n;
        self.bit_count -= n;
        Ok(result)
    }
    
    fn decode_block(&mut self, output: &mut Vec<u8>) -> Result<BlockResult, &'static str> {
        let bfinal = self.read_bits(1)?;
        let btype = self.read_bits(2)?;
        
        match btype {
            0 => {
                // Stored block
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
                // Fixed Huffman
                self.decode_huffman_block(output, true)?;
            }
            2 => {
                // Dynamic Huffman
                self.decode_huffman_block(output, false)?;
            }
            _ => return Err("invalid block type"),
        }
        
        if bfinal != 0 {
            Ok(BlockResult::Done)
        } else {
            Ok(BlockResult::Continue)
        }
    }
    
    fn decode_huffman_block(&mut self, output: &mut Vec<u8>, fixed: bool) -> Result<(), &'static str> {
        // Build Huffman tables
        let (lit_lens, dist_lens) = if fixed {
            Self::fixed_huffman_tables()
        } else {
            self.read_dynamic_tables()?
        };
        
        loop {
            let sym = self.decode_symbol(&lit_lens)?;
            
            if sym < 256 {
                output.push(sym as u8);
            } else if sym == 256 {
                break;
            } else {
                // Length/distance pair
                let length = self.decode_length(sym)?;
                let dist_sym = self.decode_symbol(&dist_lens)?;
                let distance = self.decode_distance(dist_sym)?;
                
                if distance > output.len() {
                    return Err("invalid back reference");
                }
                
                let start = output.len() - distance;
                for i in 0..length {
                    let byte = output[start + (i % distance)];
                    output.push(byte);
                }
            }
        }
        
        Ok(())
    }
    
    fn fixed_huffman_tables() -> (Vec<u8>, Vec<u8>) {
        let mut lit_lens = vec![0u8; 288];
        for i in 0..144 { lit_lens[i] = 8; }
        for i in 144..256 { lit_lens[i] = 9; }
        for i in 256..280 { lit_lens[i] = 7; }
        for i in 280..288 { lit_lens[i] = 8; }
        
        let dist_lens = vec![5u8; 32];
        
        (lit_lens, dist_lens)
    }
    
    fn read_dynamic_tables(&mut self) -> Result<(Vec<u8>, Vec<u8>), &'static str> {
        let hlit = self.read_bits(5)? as usize + 257;
        let hdist = self.read_bits(5)? as usize + 1;
        let hclen = self.read_bits(4)? as usize + 4;
        
        const CODE_ORDER: [usize; 19] = [16, 17, 18, 0, 8, 7, 9, 6, 10, 5, 11, 4, 12, 3, 13, 2, 14, 1, 15];
        
        let mut code_lens = [0u8; 19];
        for i in 0..hclen {
            code_lens[CODE_ORDER[i]] = self.read_bits(3)? as u8;
        }
        
        let code_lens_vec = code_lens.to_vec();
        
        // Decode literal/length and distance code lengths
        let mut all_lens = Vec::with_capacity(hlit + hdist);
        while all_lens.len() < hlit + hdist {
            let sym = self.decode_symbol(&code_lens_vec)?;
            match sym {
                0..=15 => all_lens.push(sym as u8),
                16 => {
                    let count = self.read_bits(2)? as usize + 3;
                    let last = *all_lens.last().ok_or("invalid code")?;
                    for _ in 0..count { all_lens.push(last); }
                }
                17 => {
                    let count = self.read_bits(3)? as usize + 3;
                    for _ in 0..count { all_lens.push(0); }
                }
                18 => {
                    let count = self.read_bits(7)? as usize + 11;
                    for _ in 0..count { all_lens.push(0); }
                }
                _ => return Err("invalid code length symbol"),
            }
        }
        
        let lit_lens = all_lens[..hlit].to_vec();
        let dist_lens = all_lens[hlit..].to_vec();
        
        Ok((lit_lens, dist_lens))
    }
    
    fn decode_symbol(&mut self, code_lens: &[u8]) -> Result<u16, &'static str> {
        // Build canonical Huffman table on the fly
        let mut code = 0u32;
        let mut first = [0u32; 16];
        let mut index = [0u16; 16];
        
        // Count codes of each length
        let mut count = [0u16; 16];
        for &len in code_lens {
            if len > 0 && (len as usize) < 16 {
                count[len as usize] += 1;
            }
        }
        
        // Build first code for each length
        let mut idx = 0u16;
        for i in 1..16 {
            first[i] = code;
            index[i] = idx;
            code = (code + count[i] as u32) << 1;
            idx += count[i];
        }
        
        // Build symbol table
        let mut symbols = vec![0u16; code_lens.len()];
        for (sym, &len) in code_lens.iter().enumerate() {
            if len > 0 {
                let len = len as usize;
                let sym_idx = index[len] as usize;
                if sym_idx < symbols.len() {
                    symbols[sym_idx] = sym as u16;
                    index[len] += 1;
                }
            }
        }
        
        // Decode symbol
        let mut code = 0u32;
        let mut first_idx = [0u16; 16];
        let mut first_code = [0u32; 16];
        
        idx = 0;
        let mut c = 0u32;
        for i in 1..16 {
            first_code[i] = c;
            first_idx[i] = idx;
            c = (c + count[i] as u32) << 1;
            idx += count[i];
        }
        
        for len in 1..16 {
            code = (code << 1) | self.read_bits(1)?;
            let cnt = count[len];
            if code < first_code[len] + cnt as u32 {
                let sym_idx = first_idx[len] + (code - first_code[len]) as u16;
                return Ok(symbols[sym_idx as usize]);
            }
        }
        
        Err("invalid huffman code")
    }
    
    fn decode_length(&mut self, sym: u16) -> Result<usize, &'static str> {
        const LEN_BASE: [u16; 29] = [3, 4, 5, 6, 7, 8, 9, 10, 11, 13, 15, 17, 19, 23, 27, 31, 35, 43, 51, 59, 67, 83, 99, 115, 131, 163, 195, 227, 258];
        const LEN_EXTRA: [u8; 29] = [0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 2, 2, 2, 2, 3, 3, 3, 3, 4, 4, 4, 4, 5, 5, 5, 5, 0];
        
        let idx = (sym - 257) as usize;
        if idx >= LEN_BASE.len() {
            return Err("invalid length code");
        }
        
        let base = LEN_BASE[idx] as usize;
        let extra = LEN_EXTRA[idx];
        let extra_bits = if extra > 0 { self.read_bits(extra)? as usize } else { 0 };
        
        Ok(base + extra_bits)
    }
    
    fn decode_distance(&mut self, sym: u16) -> Result<usize, &'static str> {
        const DIST_BASE: [u16; 30] = [1, 2, 3, 4, 5, 7, 9, 13, 17, 25, 33, 49, 65, 97, 129, 193, 257, 385, 513, 769, 1025, 1537, 2049, 3073, 4097, 6145, 8193, 12289, 16385, 24577];
        const DIST_EXTRA: [u8; 30] = [0, 0, 0, 0, 1, 1, 2, 2, 3, 3, 4, 4, 5, 5, 6, 6, 7, 7, 8, 8, 9, 9, 10, 10, 11, 11, 12, 12, 13, 13];
        
        let idx = sym as usize;
        if idx >= DIST_BASE.len() {
            return Err("invalid distance code");
        }
        
        let base = DIST_BASE[idx] as usize;
        let extra = DIST_EXTRA[idx];
        let extra_bits = if extra > 0 { self.read_bits(extra)? as usize } else { 0 };
        
        Ok(base + extra_bits)
    }
}

/// Extract a tar archive to destination
fn extract_tar(data: &[u8], dest: &str) -> Result<usize, &'static str> {
    let mut pos = 0;
    let mut file_count = 0;
    
    while pos + TAR_BLOCK_SIZE <= data.len() {
        let header = &data[pos..pos + TAR_BLOCK_SIZE];
        
        // Check for end of archive (two zero blocks)
        if header.iter().all(|&b| b == 0) {
            break;
        }
        
        // Parse header
        let name = parse_tar_name(header)?;
        let size = parse_octal(&header[124..136])?;
        let typeflag = header[156];
        
        // Get prefix for long names
        let prefix = parse_string(&header[345..500]);
        let full_name = if prefix.is_empty() {
            name
        } else {
            format!("{}/{}", prefix, name)
        };
        
        // Skip . and empty names
        if full_name.is_empty() || full_name == "." || full_name == "./" {
            pos += TAR_BLOCK_SIZE;
            pos += align_to_block(size);
            continue;
        }
        
        // Clean the path (remove leading ./)
        let clean_name = full_name.trim_start_matches("./").trim_start_matches('/');
        if clean_name.is_empty() {
            pos += TAR_BLOCK_SIZE;
            pos += align_to_block(size);
            continue;
        }
        
        let dest_path = format!("{}/{}", dest, clean_name);
        
        pos += TAR_BLOCK_SIZE;
        
        match typeflag {
            DIRTYPE | b'/' => {
                // Directory
                crate::ramfs::with_fs(|fs| {
                    let _ = fs.mkdir(&dest_path);
                });
            }
            REGTYPE | AREGTYPE | b'\0' => {
                // Regular file
                if size > 0 && pos + size <= data.len() {
                    let content = &data[pos..pos + size];
                    
                    // Ensure parent directory exists
                    let parent = parent_path(&dest_path);
                    ensure_dir_exists(&parent);
                    
                    crate::ramfs::with_fs(|fs| {
                        let _ = fs.touch(&dest_path);
                        let _ = fs.write_file(&dest_path, content);
                    });
                    
                    file_count += 1;
                }
            }
            SYMTYPE => {
                // Symbolic link - create as regular file with link target as content
                let link_target = parse_string(&header[157..257]);
                crate::ramfs::with_fs(|fs| {
                    let _ = fs.touch(&dest_path);
                    let _ = fs.write_file(&dest_path, link_target.as_bytes());
                });
            }
            _ => {
                // Unknown type, skip
            }
        }
        
        pos += align_to_block(size);
    }
    
    Ok(file_count)
}

fn parse_tar_name(header: &[u8]) -> Result<String, &'static str> {
    Ok(parse_string(&header[0..100]))
}

fn parse_string(bytes: &[u8]) -> String {
    let end = bytes.iter().position(|&b| b == 0).unwrap_or(bytes.len());
    let cow = String::from_utf8_lossy(&bytes[..end]);
    String::from(cow.trim())
}

fn parse_octal(bytes: &[u8]) -> Result<usize, &'static str> {
    let s = parse_string(bytes);
    if s.is_empty() {
        return Ok(0);
    }
    usize::from_str_radix(&s, 8).map_err(|_| "invalid octal")
}

fn align_to_block(size: usize) -> usize {
    if size == 0 {
        0
    } else {
        ((size + TAR_BLOCK_SIZE - 1) / TAR_BLOCK_SIZE) * TAR_BLOCK_SIZE
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

fn ensure_dir_exists(path: &str) {
    if path == "/" || path.is_empty() {
        return;
    }
    
    crate::ramfs::with_fs(|fs| {
        if !fs.exists(path) {
            let parent = parent_path(path);
            ensure_dir_exists(&parent);
            let _ = fs.mkdir(path);
        }
    });
}
