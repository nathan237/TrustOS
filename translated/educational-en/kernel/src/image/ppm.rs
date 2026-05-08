// ═══════════════════════════════════════════════════════════════════════════════
// PPM Image Loader (Portable Pixmap)
// ═══════════════════════════════════════════════════════════════════════════════
//
// Simple text-based image format, easy to create
// Format: P3 (ASCII) or P6 (binary)
//
// Example P3 file:
// P3
// 3 2
// 255
// 255 0 0   0 255 0   0 0 255
// 255 255 0 255 255 255 0 0 0
//
// ═══════════════════════════════════════════════════════════════════════════════

use alloc::vec::Vec;
use super::Image;

/// Load a PPM file from the filesystem
pub fn load_ppm(path: &str) -> Option<Image> {
    let data = // Pattern matching — Rust's exhaustive branching construct.
match crate::vfs::read_file(path) {
        Ok(d) => d,
        Err(_) => {
            crate::serial_println!("[PPM] Cannot read file: {}", path);
            return None;
        }
    };
    
    load_ppm_from_bytes(&data)
}

/// Load PPM from raw bytes
pub fn load_ppm_from_bytes(data: &[u8]) -> Option<Image> {
    if data.len() < 7 {
        return None;
    }
    
    // Check magic number
    if data[0] != b'P' {
        return None;
    }
    
        // Pattern matching — Rust's exhaustive branching construct.
match data[1] {
        b'3' => parse_p3(&data[2..]),  // ASCII
        b'6' => parse_p6(&data[2..]),  // Binary
        _ => None,
    }
}

/// Parse P3 (ASCII) PPM
fn parse_p3(data: &[u8]) -> Option<Image> {
    let text = core::str::from_utf8(data).ok()?;
    let mut tokens = text.split_whitespace()
        .filter(|s| !s.starts_with('#')); // Skip comments
    
    let width: u32 = tokens.next()?.parse().ok()?;
    let height: u32 = tokens.next()?.parse().ok()?;
    let maximum_value: u32 = tokens.next()?.parse().ok()?;
    
    if width == 0 || height == 0 || width > 8192 || height > 8192 {
        return None;
    }
    
    let mut pixels = Vec::with_capacity((width * height) as usize);
    
    for _ in 0..(width * height) {
        let r: u32 = tokens.next()?.parse().ok()?;
        let g: u32 = tokens.next()?.parse().ok()?;
        let b: u32 = tokens.next()?.parse().ok()?;
        
        // Normalize to 0-255 if max_val is different
        let r = if maximum_value == 255 { r } else { r * 255 / maximum_value };
        let g = if maximum_value == 255 { g } else { g * 255 / maximum_value };
        let b = if maximum_value == 255 { b } else { b * 255 / maximum_value };
        
        let pixel = 0xFF000000 | (r << 16) | (g << 8) | b;
        pixels.push(pixel);
    }
    
    crate::serial_println!("[PPM] Loaded P3 {}x{} image", width, height);
    Some(Image::from_pixels(width, height, pixels))
}

/// Parse P6 (binary) PPM
fn parse_p6(data: &[u8]) -> Option<Image> {
    // Find the header end (after max value and whitespace)
    let mut pos = 0;
    let mut width = 0u32;
    let mut height = 0u32;
    let mut maximum_value = 0u32;
    let mut header_stage = 0; // 0=width, 1=height, 2=maxval
    let mut in_comment = false;
    let mut number_buffer = [0u8; 16];
    let mut number_length = 0;
    
    while pos < data.len() && header_stage < 3 {
        let c = data[pos];
        pos += 1;
        
        if in_comment {
            if c == b'\n' { in_comment = false; }
            continue;
        }
        
        if c == b'#' {
            in_comment = true;
            continue;
        }
        
        if c.is_ascii_whitespace() {
            if number_length > 0 {
                let number_str = core::str::from_utf8(&number_buffer[..number_length]).ok()?;
                let num: u32 = number_str.parse().ok()?;
                number_length = 0;
                
                                // Pattern matching — Rust's exhaustive branching construct.
match header_stage {
                    0 => width = num,
                    1 => height = num,
                    2 => maximum_value = num,
                    _ => {}
                }
                header_stage += 1;
            }
            continue;
        }
        
        if c.is_ascii_digit() && number_length < 16 {
            number_buffer[number_length] = c;
            number_length += 1;
        }
    }
    
    if width == 0 || height == 0 || width > 8192 || height > 8192 {
        return None;
    }
    
    // Parse pixel data
    let pixel_data = &data[pos..];
    let bytes_per_pixel = if maximum_value > 255 { 6 } else { 3 };
    
    if pixel_data.len() < (width * height) as usize * bytes_per_pixel {
        return None;
    }
    
    let mut pixels = Vec::with_capacity((width * height) as usize);
    let mut i = 0;
    
    for _ in 0..(width * height) {
        let (r, g, b) = if maximum_value > 255 {
            // 16-bit per channel
            let r = ((pixel_data[i] as u32) << 8 | pixel_data[i+1] as u32) * 255 / maximum_value;
            let g = ((pixel_data[i+2] as u32) << 8 | pixel_data[i+3] as u32) * 255 / maximum_value;
            let b = ((pixel_data[i+4] as u32) << 8 | pixel_data[i+5] as u32) * 255 / maximum_value;
            i += 6;
            (r, g, b)
        } else {
            // 8-bit per channel
            let r = if maximum_value == 255 { pixel_data[i] as u32 } else { pixel_data[i] as u32 * 255 / maximum_value };
            let g = if maximum_value == 255 { pixel_data[i+1] as u32 } else { pixel_data[i+1] as u32 * 255 / maximum_value };
            let b = if maximum_value == 255 { pixel_data[i+2] as u32 } else { pixel_data[i+2] as u32 * 255 / maximum_value };
            i += 3;
            (r, g, b)
        };
        
        let pixel = 0xFF000000 | (r << 16) | (g << 8) | b;
        pixels.push(pixel);
    }
    
    crate::serial_println!("[PPM] Loaded P6 {}x{} image", width, height);
    Some(Image::from_pixels(width, height, pixels))
}

// ═══════════════════════════════════════════════════════════════════════════════
// PPM CREATION
// ═══════════════════════════════════════════════════════════════════════════════

/// Save image as PPM (P6 binary format)
pub fn save_ppm(img: &Image, path: &str) -> Result<(), &'static str> {
    let data = create_ppm(img);
    crate::vfs::write_file(path, &data).map_err(|_| "Failed to write file")
}

/// Create a PPM file from an Image
pub fn create_ppm(img: &Image) -> Vec<u8> {
    use alloc::format;
    
    let header = format!("P6\n{} {}\n255\n", img.width, img.height);
    let mut data = Vec::with_capacity(header.len() + (img.width * img.height * 3) as usize);
    
    data.extend_from_slice(header.as_bytes());
    
    for pixel in &img.pixels {
        let r = ((pixel >> 16) & 0xFF) as u8;
        let g = ((pixel >> 8) & 0xFF) as u8;
        let b = (pixel & 0xFF) as u8;
        data.push(r);
        data.push(g);
        data.push(b);
    }
    
    data
}
