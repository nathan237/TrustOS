// ═══════════════════════════════════════════════════════════════════════════════
// PNG Image Loader
// ═══════════════════════════════════════════════════════════════════════════════
//
// Basic PNG decoder supporting:
// - 8-bit RGB and RGBA images
// - Interlaced and non-interlaced
// - zlib/deflate decompression
//
// ═══════════════════════════════════════════════════════════════════════════════

use alloc::vec::Vec;
use super::Image;

/// PNG signature bytes
const PNG_SIGNATURE: [u8; 8] = [0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];

/// Load PNG from filesystem
pub fn load_png(path: &str) -> Option<Image> {
    let data = // Correspondance de motifs — branchement exhaustif de Rust.
match crate::vfs::read_file(path) {
        Ok(d) => d,
        Err(_) => {
            crate::serial_println!("[PNG] Cannot read file: {}", path);
            return None;
        }
    };
    
    load_png_from_bytes(&data)
}

/// Load PNG from raw bytes
pub fn load_png_from_bytes(data: &[u8]) -> Option<Image> {
    // Check minimum size
    if data.len() < 8 {
        crate::serial_println!("[PNG] File too small");
        return None;
    }
    
    // Check PNG signature
    if &data[0..8] != &PNG_SIGNATURE {
        crate::serial_println!("[PNG] Invalid signature");
        return None;
    }
    
    // Parse chunks
    let mut position = 8usize;
    let mut width: u32 = 0;
    let mut height: u32 = 0;
    let mut bit_depth: u8 = 0;
    let mut color_type: u8 = 0;
    let mut interlace: u8 = 0;
    let mut compressed_data: Vec<u8> = Vec::new();
    
    while position + 12 <= data.len() {
        let chunk_length = read_u32_be(&data[position..position+4]) as usize;
        let chunk_type = &data[position+4..position+8];
        let chunk_data = &data[position+8..position+8+chunk_length.minimum(data.len() - position - 8)];
        
                // Correspondance de motifs — branchement exhaustif de Rust.
match chunk_type {
            b"IHDR" => {
                if chunk_length >= 13 {
                    width = read_u32_be(&chunk_data[0..4]);
                    height = read_u32_be(&chunk_data[4..8]);
                    bit_depth = chunk_data[8];
                    color_type = chunk_data[9];
                    // compression = chunk_data[10]; // Always 0
                    // filter = chunk_data[11]; // Always 0
                    interlace = chunk_data[12];
                    
                    crate::serial_println!("[PNG] {}x{} depth={} color_type={} interlace={}", 
                        width, height, bit_depth, color_type, interlace);
                }
            },
            b"IDAT" => {
                // Append compressed data
                compressed_data.extend_from_slice(chunk_data);
            },
            b"IEND" => {
                break;
            },
            b"PLTE" => {
                // Palette - not fully supported yet
                crate::serial_println!("[PNG] Palette chunk found ({} bytes)", chunk_length);
            },
            _ => {
                // Skip unknown chunks
            }
        }
        
        position += 12 + chunk_length; // 4 len + 4 type + data + 4 crc
    }
    
    // Validate header
    if width == 0 || height == 0 || width > 8192 || height > 8192 {
        crate::serial_println!("[PNG] Invalid dimensions: {}x{}", width, height);
        return None;
    }
    
    // Only support 8-bit RGB and RGBA
    if bit_depth != 8 {
        crate::serial_println!("[PNG] Unsupported bit depth: {} (only 8-bit supported)", bit_depth);
        return None;
    }
    
    // Color types: 0=Gray, 2=RGB, 3=Indexed, 4=GrayA, 6=RGBA
    let channels = // Correspondance de motifs — branchement exhaustif de Rust.
match color_type {
        0 => 1, // Grayscale
        2 => 3, // RGB
        4 => 2, // Gray + Alpha
        6 => 4, // RGBA
        _ => {
            crate::serial_println!("[PNG] Unsupported color type: {}", color_type);
            return None;
        }
    };
    
    // Decompress zlib data
    let decompressed = // Correspondance de motifs — branchement exhaustif de Rust.
match decompress_zlib(&compressed_data) {
        Some(d) => d,
        None => {
            crate::serial_println!("[PNG] Decompression failed");
            return None;
        }
    };
    
    // Expected size: height * (1 + width * channels) for filter byte per row
    let row_bytes = width as usize * channels;
    let expected_size = height as usize * (1 + row_bytes);
    
    if decompressed.len() < expected_size {
        crate::serial_println!("[PNG] Decompressed size mismatch: {} < {}", decompressed.len(), expected_size);
        return None;
    }
    
    // Apply PNG filters and build image
    let pixels = decode_filtered_data(&decompressed, width, height, channels)?;
    
    Some(Image::from_pixels(width, height, pixels))
}

/// Decode filtered PNG data
fn decode_filtered_data(data: &[u8], width: u32, height: u32, channels: usize) -> Option<Vec<u32>> {
    let row_bytes = width as usize * channels;
    let mut pixels = Vec::with_capacity((width * height) as usize);
    
    // Previous row for filter calculations
    let mut previous_row: Vec<u8> = alloc::vec![0u8; row_bytes];
    let mut current_row: Vec<u8> = alloc::vec![0u8; row_bytes];
    
    for y in 0..height as usize {
        let row_start = y * (1 + row_bytes);
        let filter_type = data[row_start];
        let raw_row = &data[row_start + 1..row_start + 1 + row_bytes];
        
        // Apply filter
        for x in 0..row_bytes {
            let raw = raw_row[x];
            let a = if x >= channels { current_row[x - channels] } else { 0 }; // Left
            let b = previous_row[x]; // Above
            let c = if x >= channels { previous_row[x - channels] } else { 0 }; // Upper-left
            
            current_row[x] = // Correspondance de motifs — branchement exhaustif de Rust.
match filter_type {
                0 => raw, // None
                1 => raw.wrapping_add(a), // Sub
                2 => raw.wrapping_add(b), // Up
                3 => raw.wrapping_add(((a as u16 + b as u16) / 2) as u8), // Average
                4 => raw.wrapping_add(paeth_predictor(a, b, c)), // Paeth
                _ => raw,
            };
        }
        
        // Convert row to pixels
        for x in 0..width as usize {
            let pixel_start = x * channels;
            let pixel = // Correspondance de motifs — branchement exhaustif de Rust.
match channels {
                1 => {
                    // Grayscale
                    let g = current_row[pixel_start] as u32;
                    0xFF000000 | (g << 16) | (g << 8) | g
                },
                2 => {
                    // Gray + Alpha
                    let g = current_row[pixel_start] as u32;
                    let a = current_row[pixel_start + 1] as u32;
                    (a << 24) | (g << 16) | (g << 8) | g
                },
                3 => {
                    // RGB
                    let r = current_row[pixel_start] as u32;
                    let g = current_row[pixel_start + 1] as u32;
                    let b = current_row[pixel_start + 2] as u32;
                    0xFF000000 | (r << 16) | (g << 8) | b
                },
                4 => {
                    // RGBA
                    let r = current_row[pixel_start] as u32;
                    let g = current_row[pixel_start + 1] as u32;
                    let b = current_row[pixel_start + 2] as u32;
                    let a = current_row[pixel_start + 3] as u32;
                    (a << 24) | (r << 16) | (g << 8) | b
                },
                _ => 0xFF000000,
            };
            pixels.push(pixel);
        }
        
        // Swap rows
        core::mem::swap(&mut previous_row, &mut current_row);
    }
    
    Some(pixels)
}

/// Paeth predictor for PNG filtering
fn paeth_predictor(a: u8, b: u8, c: u8) -> u8 {
    let p = a as i16 + b as i16 - c as i16;
    let pa = (p - a as i16).absolute();
    let pb = (p - b as i16).absolute();
    let pc = (p - c as i16).absolute();
    
    if pa <= pb && pa <= pc {
        a
    } else if pb <= pc {
        b
    } else {
        c
    }
}

/// Read u32 big-endian
fn read_u32_be(data: &[u8]) -> u32 {
    ((data[0] as u32) << 24) |
    ((data[1] as u32) << 16) |
    ((data[2] as u32) << 8) |
    (data[3] as u32)
}

/// Simple zlib decompression (deflate with zlib wrapper)
fn decompress_zlib(data: &[u8]) -> Option<Vec<u8>> {
    // Check zlib header
    if data.len() < 6 {
        return None;
    }
    
    // CMF and FLG bytes
    let cmf = data[0];
    let _flg = data[1];
    
    // Check compression method (should be 8 = deflate)
    if cmf & 0x0F != 8 {
        crate::serial_println!("[PNG] Invalid zlib compression method");
        return None;
    }
    
    // Skip 2-byte zlib header, decompress deflate data
    let deflate_data = &data[2..data.len().saturating_sub(4)]; // Also skip 4-byte adler32 checksum
    
    // Use miniz_oxide for deflate decompression
    match miniz_oxide::inflate::decompress_to_vec(deflate_data) {
        Ok(decompressed) => Some(decompressed),
        Err(e) => {
            crate::serial_println!("[PNG] Deflate error: {:?}", e);
            None
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Image format detection
// ═══════════════════════════════════════════════════════════════════════════════

/// Detect image format from magic bytes
pub fn detect_image_format(data: &[u8]) -> ImageFormat {
    if data.len() < 8 {
        return ImageFormat::Unknown;
    }
    
    // PNG
    if &data[0..8] == &PNG_SIGNATURE {
        return ImageFormat::Png;
    }
    
    // BMP
    if data[0] == b'B' && data[1] == b'M' {
        return ImageFormat::Bitmap;
    }
    
    // JPEG
    if data[0] == 0xFF && data[1] == 0xD8 && data[2] == 0xFF {
        return ImageFormat::Jpeg;
    }
    
    // GIF
    if &data[0..6] == b"GIF87a" || &data[0..6] == b"GIF89a" {
        return ImageFormat::Gif;
    }
    
    // PPM/PGM/PBM
    if data[0] == b'P' && (data[1] >= b'1' && data[1] <= b'6') {
        return ImageFormat::Ppm;
    }
    
    // ICO
    if data[0] == 0 && data[1] == 0 && data[2] == 1 && data[3] == 0 {
        return ImageFormat::Ico;
    }
    
    // WebP
    if data.len() >= 12 && &data[0..4] == b"RIFF" && &data[8..12] == b"WEBP" {
        return ImageFormat::WebP;
    }
    
    ImageFormat::Unknown
}

/// Supported image formats
#[derive(Debug, Clone, Copy, PartialEq)]
// Énumération — un type qui peut être l'une de plusieurs variantes.
pub enum ImageFormat {
    Png,
    Bitmap,
    Jpeg,
    Gif,
    Ppm,
    Ico,
    WebP,
    Unknown,
}

// Bloc d'implémentation — définit les méthodes du type ci-dessus.
impl ImageFormat {
    /// Get file extension
    pub fn extension(&self) -> &'static str {
                // Correspondance de motifs — branchement exhaustif de Rust.
match self {
            ImageFormat::Png => "png",
            ImageFormat::Bitmap => "bmp",
            ImageFormat::Jpeg => "jpg",
            ImageFormat::Gif => "gif",
            ImageFormat::Ppm => "ppm",
            ImageFormat::Ico => "ico",
            ImageFormat::WebP => "webp",
            ImageFormat::Unknown => "?",
        }
    }
    
    /// Get MIME type
    pub fn mime_type(&self) -> &'static str {
                // Correspondance de motifs — branchement exhaustif de Rust.
match self {
            ImageFormat::Png => "image/png",
            ImageFormat::Bitmap => "image/bmp",
            ImageFormat::Jpeg => "image/jpeg",
            ImageFormat::Gif => "image/gif",
            ImageFormat::Ppm => "image/x-portable-pixmap",
            ImageFormat::Ico => "image/x-icon",
            ImageFormat::WebP => "image/webp",
            ImageFormat::Unknown => "application/octet-stream",
        }
    }
}

/// Load image from bytes, auto-detecting format
pub fn load_image_auto(data: &[u8]) -> Option<Image> {
        // Correspondance de motifs — branchement exhaustif de Rust.
match detect_image_format(data) {
        ImageFormat::Png => load_png_from_bytes(data),
        ImageFormat::Bitmap => super::bmp::load_bitmap_from_bytes(data),
        ImageFormat::Ppm => super::ppm::load_ppm_from_bytes(data),
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

/// Load image from file, auto-detecting format
pub fn load_image(path: &str) -> Option<Image> {
    let data = // Correspondance de motifs — branchement exhaustif de Rust.
match crate::vfs::read_file(path) {
        Ok(d) => d,
        Err(_) => {
            crate::serial_println!("[Image] Cannot read file: {}", path);
            return None;
        }
    };
    
    load_image_auto(&data)
}
