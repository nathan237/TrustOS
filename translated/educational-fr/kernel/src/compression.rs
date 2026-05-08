//! Compression utilities for tar.gz files
//!
//! This module provides gzip decompression using miniz_oxide

use alloc::vec::Vec;
use miniz_oxide::inflate::decompress_to_vec_zlib;

/// Decompress gzip data
/// Gzip format: 10 byte header, deflate data, 8 byte trailer
pub fn gzip_decompress(data: &[u8]) -> Result<Vec<u8>, &'static str> {
    if data.len() < 18 {
        return Err("Data too small for gzip");
    }
    
    // Check gzip magic
    if data[0] != 0x1f || data[1] != 0x8b {
        return Err("Not gzip format");
    }
    
    // Check compression method (8 = deflate)
    if data[2] != 8 {
        return Err("Unsupported compression method");
    }
    
    let flags = data[3];
    let mut offset = 10;
    
    // Skip optional extra field (FEXTRA)
    if flags & 0x04 != 0 {
        if offset + 2 > data.len() {
            return Err("Invalid extra field");
        }
        let xlen = u16::from_le_bytes([data[offset], data[offset + 1]]) as usize;
        offset += 2 + xlen;
    }
    
    // Skip original filename (FNAME)
    if flags & 0x08 != 0 {
        while offset < data.len() && data[offset] != 0 {
            offset += 1;
        }
        offset += 1; // skip null terminator
    }
    
    // Skip comment (FCOMMENT)
    if flags & 0x10 != 0 {
        while offset < data.len() && data[offset] != 0 {
            offset += 1;
        }
        offset += 1;
    }
    
    // Skip header CRC (FHCRC)
    if flags & 0x02 != 0 {
        offset += 2;
    }
    
    if offset >= data.len() - 8 {
        return Err("Invalid gzip header");
    }
    
    // The compressed deflate data is between offset and len-8 (8 byte trailer: CRC32 + ISIZE)
    let compressed_data = &data[offset..data.len() - 8];
    
    // Use miniz_oxide to decompress the raw deflate stream
    miniz_oxide::inflate::decompress_to_vec(compressed_data)
        .map_err(|_| "Deflate decompression failed")
}
