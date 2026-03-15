// ═══════════════════════════════════════════════════════════════════════════════
// BMP Image Loader
// ═══════════════════════════════════════════════════════════════════════════════
//
// Simple BMP loader for wallpapers and icons
// Supports: 24-bit and 32-bit uncompressed BMP
//
// ═══════════════════════════════════════════════════════════════════════════════

use alloc::vec::Vec;
use super::WallpaperData;

/// BMP file header (14 bytes)
#[repr(C, packed)]
struct BmpFileHeader {
    signature: [u8; 2],    // 'BM'
    file_size: u32,
    reserved: u32,
    data_offset: u32,
}

/// BMP info header (40 bytes for BITMAPINFOHEADER)
#[repr(C, packed)]
struct BmpInfoHeader {
    header_size: u32,
    width: i32,
    height: i32,
    planes: u16,
    bits_per_pixel: u16,
    compression: u32,
    image_size: u32,
    x_pixels_per_meter: u32,
    y_pixels_per_meter: u32,
    colors_used: u32,
    colors_important: u32,
}

/// Load a BMP file from the filesystem
pub fn load_bmp(path: &str) -> Option<WallpaperData> {
    let data = match crate::vfs::read_file(path) {
        Ok(d) => d,
        Err(_) => {
            crate::serial_println!("[BMP] Cannot read file: {}", path);
            return None;
        }
    };
    
    parse_bmp(&data)
}

/// Load BMP from raw bytes
pub fn load_bmp_from_bytes(data: &[u8]) -> Option<WallpaperData> {
    parse_bmp(data)
}

/// Parse BMP data
fn parse_bmp(data: &[u8]) -> Option<WallpaperData> {
    // Minimum size check
    if data.len() < 54 {
        crate::serial_println!("[BMP] File too small");
        return None;
    }
    
    // Check BMP signature
    if data[0] != b'B' || data[1] != b'M' {
        crate::serial_println!("[BMP] Invalid signature");
        return None;
    }
    
    // Parse headers manually (to avoid alignment issues)
    let data_offset = read_u32(&data[10..14]) as usize;
    let header_size = read_u32(&data[14..18]);
    let width = read_i32(&data[18..22]);
    let height = read_i32(&data[22..26]);
    let bits_per_pixel = read_u16(&data[28..30]);
    let compression = read_u32(&data[30..34]);
    
    // Validate
    if width <= 0 || width > 8192 {
        crate::serial_println!("[BMP] Invalid width: {}", width);
        return None;
    }
    
    let height_abs = height.abs();
    if height_abs <= 0 || height_abs > 8192 {
        crate::serial_println!("[BMP] Invalid height: {}", height);
        return None;
    }
    
    // Only support uncompressed
    if compression != 0 && compression != 3 {
        crate::serial_println!("[BMP] Unsupported compression: {}", compression);
        return None;
    }
    
    // Only support 24-bit or 32-bit
    if bits_per_pixel != 24 && bits_per_pixel != 32 {
        crate::serial_println!("[BMP] Unsupported bit depth: {}", bits_per_pixel);
        return None;
    }
    
    let width = width as u32;
    let height_u = height_abs as u32;
    let is_top_down = height < 0;
    
    // Calculate row stride (BMP rows are 4-byte aligned)
    let bytes_per_pixel = bits_per_pixel as usize / 8;
    let row_size = ((width as usize * bytes_per_pixel + 3) / 4) * 4;
    
    // Allocate pixels
    let pixel_count = (width * height_u) as usize;
    let mut pixels = Vec::with_capacity(pixel_count);
    
    // Parse pixel data
    let pixel_data = &data[data_offset..];
    
    for row in 0..height_u {
        // BMP stores bottom-to-top by default
        let src_row = if is_top_down { row } else { height_u - 1 - row };
        let row_start = src_row as usize * row_size;
        
        for col in 0..width {
            let pixel_start = row_start + col as usize * bytes_per_pixel;
            
            if pixel_start + bytes_per_pixel <= pixel_data.len() {
                // BMP stores BGR(A)
                let b = pixel_data[pixel_start] as u32;
                let g = pixel_data[pixel_start + 1] as u32;
                let r = pixel_data[pixel_start + 2] as u32;
                let a = if bits_per_pixel == 32 {
                    pixel_data[pixel_start + 3] as u32
                } else {
                    255
                };
                
                // Convert to ARGB
                let pixel = (a << 24) | (r << 16) | (g << 8) | b;
                pixels.push(pixel);
            } else {
                pixels.push(0xFF000000); // Black fallback
            }
        }
    }
    
    crate::serial_println!("[BMP] Loaded {}x{} image ({}-bit)", width, height_u, bits_per_pixel);
    
    Some(WallpaperData {
        width,
        height: height_u,
        pixels,
    })
}

/// Read little-endian u32
fn read_u32(data: &[u8]) -> u32 {
    u32::from_le_bytes([data[0], data[1], data[2], data[3]])
}

/// Read little-endian i32
fn read_i32(data: &[u8]) -> i32 {
    i32::from_le_bytes([data[0], data[1], data[2], data[3]])
}

/// Read little-endian u16
fn read_u16(data: &[u8]) -> u16 {
    u16::from_le_bytes([data[0], data[1]])
}

// ═══════════════════════════════════════════════════════════════════════════════
// BMP CREATION (for saving screenshots)
// ═══════════════════════════════════════════════════════════════════════════════

/// Create a BMP file from pixel data
pub fn create_bmp(width: u32, height: u32, pixels: &[u32]) -> Vec<u8> {
    let bytes_per_pixel = 3; // 24-bit
    let row_size = ((width as usize * bytes_per_pixel + 3) / 4) * 4;
    let pixel_data_size = row_size * height as usize;
    let file_size = 54 + pixel_data_size;
    
    let mut bmp = Vec::with_capacity(file_size);
    
    // File header (14 bytes)
    bmp.extend_from_slice(&[b'B', b'M']);
    bmp.extend_from_slice(&(file_size as u32).to_le_bytes());
    bmp.extend_from_slice(&[0u8; 4]); // Reserved
    bmp.extend_from_slice(&54u32.to_le_bytes()); // Data offset
    
    // Info header (40 bytes)
    bmp.extend_from_slice(&40u32.to_le_bytes()); // Header size
    bmp.extend_from_slice(&(width as i32).to_le_bytes());
    bmp.extend_from_slice(&(height as i32).to_le_bytes()); // Positive = bottom-up
    bmp.extend_from_slice(&1u16.to_le_bytes()); // Planes
    bmp.extend_from_slice(&24u16.to_le_bytes()); // Bits per pixel
    bmp.extend_from_slice(&0u32.to_le_bytes()); // Compression (none)
    bmp.extend_from_slice(&(pixel_data_size as u32).to_le_bytes());
    bmp.extend_from_slice(&[0u8; 16]); // Resolution and colors
    
    // Pixel data (bottom-to-top, BGR)
    for row in (0..height).rev() {
        for col in 0..width {
            let pixel = pixels[(row * width + col) as usize];
            let r = ((pixel >> 16) & 0xFF) as u8;
            let g = ((pixel >> 8) & 0xFF) as u8;
            let b = (pixel & 0xFF) as u8;
            bmp.push(b);
            bmp.push(g);
            bmp.push(r);
        }
        // Padding to 4-byte boundary
        let padding = row_size - (width as usize * 3);
        for _ in 0..padding {
            bmp.push(0);
        }
    }
    
    bmp
}
