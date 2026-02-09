//! Pixlens - Visual Cryptanalysis Tool (No Dependencies Version)
//! 
//! Reveal hidden patterns in binaries through custom bitmap mappings.
//! Different mappings act as "lenses" to see what normal analysis cannot.
//! Outputs PPM format (viewable in most image viewers, GIMP, IrfanView, etc.)
//!
//! Author: Nathan (nated0ge)

use std::env;
use std::fs::File;
use std::io::{Read, Write, BufWriter};
use std::path::Path;

fn main() {
    let args: Vec<String> = env::args().collect();
    
    if args.len() < 2 {
        print_usage(&args[0]);
        return;
    }

    let mut input_path = String::new();
    let mut output_path = String::from("output.ppm");
    let mut mapping = String::from("linear");
    let mut param: u8 = 0;
    let mut width: u32 = 512;
    let mut all_mappings = false;

    // Parse arguments
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "-i" | "--input" => {
                i += 1;
                if i < args.len() { input_path = args[i].clone(); }
            }
            "-o" | "--output" => {
                i += 1;
                if i < args.len() { output_path = args[i].clone(); }
            }
            "-m" | "--mapping" => {
                i += 1;
                if i < args.len() { mapping = args[i].clone(); }
            }
            "-p" | "--param" => {
                i += 1;
                if i < args.len() { param = args[i].parse().unwrap_or(0); }
            }
            "-w" | "--width" => {
                i += 1;
                if i < args.len() { width = args[i].parse().unwrap_or(512); }
            }
            "--all" => { all_mappings = true; }
            "-h" | "--help" => { print_usage(&args[0]); return; }
            _ => {
                if input_path.is_empty() && !args[i].starts_with('-') {
                    input_path = args[i].clone();
                }
            }
        }
        i += 1;
    }

    if input_path.is_empty() {
        eprintln!("Error: No input file specified");
        print_usage(&args[0]);
        return;
    }

    // Read input file
    let mut file = match File::open(&input_path) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("Error opening {}: {}", input_path, e);
            return;
        }
    };
    let mut data = Vec::new();
    file.read_to_end(&mut data).expect("Failed to read file");

    println!("[pixlens] Loaded {} bytes from {}", data.len(), input_path);

    if all_mappings {
        generate_all_mappings(&data, &output_path, width);
    } else {
        let (img, w, h) = apply_mapping(&data, &mapping, param, width);
        save_ppm(&output_path, &img, w, h);
        println!("[pixlens] Saved {} ({} mapping, {}x{})", output_path, mapping, w, h);
    }
}

fn print_usage(prog: &str) {
    println!("Pixlens - Visual Cryptanalysis Tool");
    println!("Author: Nathan (nated0ge)");
    println!();
    println!("Usage: {} -i <input> [options]", prog);
    println!();
    println!("Options:");
    println!("  -i, --input <file>    Input file to analyze");
    println!("  -o, --output <file>   Output image (default: output.ppm)");
    println!("  -m, --mapping <mode>  Mapping mode (default: linear)");
    println!("  -p, --param <n>       Parameter for mapping (0-255)");
    println!("  -w, --width <n>       Image width (default: 512)");
    println!("  --all                 Generate all mappings");
    println!();
    println!("Mappings:");
    println!("  linear    - Basic X=offset, Y=line, pixel=byte");
    println!("  xor       - XOR each byte with param before display");
    println!("  bitplane  - Show only bit N (param=0-7)");
    println!("  digraph   - 256x256 byte-pair correlation scatter");
    println!("  trigraph  - RGB = 3 consecutive bytes");
    println!("  entropy   - Local entropy (param=window size)");
    println!("  modulo    - Fold at period N (param=period)");
    println!("  diff      - Difference between consecutive bytes");
    println!("  frequency - 2D histogram heatmap");
    println!("  hilbert   - Hilbert curve spatial mapping");
    println!("  ascii     - Highlight printable ASCII");
    println!("  null      - Highlight null bytes");
    println!("  highent   - Highlight high-entropy regions");
    println!();
    println!("Examples:");
    println!("  {} -i binary.exe", prog);
    println!("  {} -i encrypted.bin -m digraph", prog);
    println!("  {} -i data.bin -m xor -p 255", prog);
    println!("  {} -i malware.bin --all", prog);
}

fn generate_all_mappings(data: &[u8], output_base: &str, width: u32) {
    let mappings = [
        ("linear", 0u8),
        ("xor", 0xFF),
        ("xor", 0xAA),
        ("bitplane", 0),
        ("bitplane", 7),
        ("digraph", 0),
        ("trigraph", 0),
        ("entropy", 16),
        ("entropy", 64),
        ("modulo", 16),
        ("modulo", 64),
        ("diff", 0),
        ("frequency", 0),
        ("ascii", 0),
        ("null", 0),
        ("highent", 32),
    ];

    let base = Path::new(output_base).file_stem().unwrap().to_str().unwrap();
    let parent = Path::new(output_base).parent().unwrap_or(Path::new("."));

    for (mode, param) in mappings {
        let suffix = if param > 0 && mode != "digraph" && mode != "trigraph" && mode != "frequency" {
            format!("{}_{}", mode, param)
        } else {
            mode.to_string()
        };
        let path = parent.join(format!("{}_{}.ppm", base, suffix));
        let (img, w, h) = apply_mapping(data, mode, param, width);
        save_ppm(path.to_str().unwrap(), &img, w, h);
        println!("[pixlens] Generated {:?}", path);
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// PPM OUTPUT
// ═══════════════════════════════════════════════════════════════════════════

fn save_ppm(path: &str, pixels: &[(u8, u8, u8)], width: u32, height: u32) {
    let file = File::create(path).expect("Failed to create output file");
    let mut writer = BufWriter::new(file);
    
    // PPM header
    writeln!(writer, "P6").unwrap();
    writeln!(writer, "{} {}", width, height).unwrap();
    writeln!(writer, "255").unwrap();
    
    // Pixel data
    for &(r, g, b) in pixels {
        writer.write_all(&[r, g, b]).unwrap();
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// MAPPING DISPATCH
// ═══════════════════════════════════════════════════════════════════════════

fn apply_mapping(data: &[u8], mode: &str, param: u8, width: u32) -> (Vec<(u8, u8, u8)>, u32, u32) {
    match mode {
        "linear" => map_linear(data, width),
        "xor" => map_xor(data, param, width),
        "bitplane" => map_bitplane(data, param, width),
        "digraph" => map_digraph(data),
        "trigraph" => map_trigraph(data, width),
        "entropy" => map_entropy(data, param.max(4) as usize, width),
        "modulo" => map_modulo(data, param.max(1) as u32),
        "diff" => map_diff(data, width),
        "frequency" => map_frequency(data),
        "hilbert" => map_hilbert(data),
        "ascii" => map_ascii(data, width),
        "null" => map_null(data, width),
        "highent" => map_high_entropy(data, param.max(8) as usize, width),
        _ => {
            eprintln!("Unknown mapping: {}, using linear", mode);
            map_linear(data, width)
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// MAPPING IMPLEMENTATIONS
// ═══════════════════════════════════════════════════════════════════════════

fn map_linear(data: &[u8], width: u32) -> (Vec<(u8, u8, u8)>, u32, u32) {
    let height = (data.len() as u32 + width - 1) / width;
    let mut pixels = vec![(0u8, 0u8, 0u8); (width * height) as usize];

    for (i, &byte) in data.iter().enumerate() {
        pixels[i] = (byte, byte, byte);
    }
    (pixels, width, height)
}

fn map_xor(data: &[u8], key: u8, width: u32) -> (Vec<(u8, u8, u8)>, u32, u32) {
    let height = (data.len() as u32 + width - 1) / width;
    let mut pixels = vec![(0u8, 0u8, 0u8); (width * height) as usize];

    for (i, &byte) in data.iter().enumerate() {
        let v = byte ^ key;
        pixels[i] = (v, v, v);
    }
    (pixels, width, height)
}

fn map_bitplane(data: &[u8], bit: u8, width: u32) -> (Vec<(u8, u8, u8)>, u32, u32) {
    let height = (data.len() as u32 + width - 1) / width;
    let mut pixels = vec![(0u8, 0u8, 0u8); (width * height) as usize];
    let mask = 1 << (bit.min(7));

    for (i, &byte) in data.iter().enumerate() {
        let v = if (byte & mask) != 0 { 255 } else { 0 };
        pixels[i] = (v, v, v);
    }
    (pixels, width, height)
}

/// Digraph: X=byte[i], Y=byte[i+1] - EXTREMELY powerful for crypto analysis
fn map_digraph(data: &[u8]) -> (Vec<(u8, u8, u8)>, u32, u32) {
    let mut pixels = vec![(0u8, 0u8, 0u8); 256 * 256];
    let mut counts = [[0u32; 256]; 256];

    for w in data.windows(2) {
        counts[w[0] as usize][w[1] as usize] += 1;
    }

    let max = counts.iter().flatten().max().copied().unwrap_or(1).max(1);

    for y in 0..256 {
        for x in 0..256 {
            let count = counts[y][x];
            if count > 0 {
                let norm = (((count as f32).ln() / (max as f32).ln()) * 255.0) as u8;
                pixels[y * 256 + x] = (0, norm, norm / 2);
            }
        }
    }
    (pixels, 256, 256)
}

fn map_trigraph(data: &[u8], width: u32) -> (Vec<(u8, u8, u8)>, u32, u32) {
    let pixel_count = data.len() / 3;
    let height = ((pixel_count as u32) + width - 1) / width;
    let mut pixels = vec![(0u8, 0u8, 0u8); (width * height) as usize];

    for i in 0..pixel_count {
        let r = data[i * 3];
        let g = data.get(i * 3 + 1).copied().unwrap_or(0);
        let b = data.get(i * 3 + 2).copied().unwrap_or(0);
        pixels[i] = (r, g, b);
    }
    (pixels, width, height)
}

fn map_entropy(data: &[u8], window: usize, width: u32) -> (Vec<(u8, u8, u8)>, u32, u32) {
    let height = (data.len() as u32 + width - 1) / width;
    let mut pixels = vec![(0u8, 0u8, 0u8); (width * height) as usize];

    for i in 0..data.len() {
        let start = i.saturating_sub(window / 2);
        let end = (i + window / 2).min(data.len());
        let entropy = calc_entropy(&data[start..end]);
        let norm = ((entropy / 8.0) * 255.0) as u8;
        // Low entropy = blue, High = red
        pixels[i] = (norm, 0, 255 - norm);
    }
    (pixels, width, height)
}

fn calc_entropy(data: &[u8]) -> f32 {
    if data.is_empty() { return 0.0; }
    let mut counts = [0u32; 256];
    for &b in data { counts[b as usize] += 1; }
    let len = data.len() as f32;
    let mut e = 0.0f32;
    for &c in &counts {
        if c > 0 {
            let p = c as f32 / len;
            e -= p * p.log2();
        }
    }
    e
}

fn map_modulo(data: &[u8], period: u32) -> (Vec<(u8, u8, u8)>, u32, u32) {
    let height = (data.len() as u32 + period - 1) / period;
    let mut pixels = vec![(0u8, 0u8, 0u8); (period * height) as usize];

    for (i, &byte) in data.iter().enumerate() {
        pixels[i] = (byte, byte, byte);
    }
    (pixels, period, height)
}

fn map_diff(data: &[u8], width: u32) -> (Vec<(u8, u8, u8)>, u32, u32) {
    let height = (data.len() as u32 + width - 1) / width;
    let mut pixels = vec![(0u8, 0u8, 0u8); (width * height) as usize];

    for i in 0..data.len() {
        let diff = if i > 0 {
            let a = data[i - 1] as i16;
            let b = data[i] as i16;
            ((b - a + 256) % 256) as u8
        } else {
            data[i]
        };
        // Positive = green, Negative = red
        if diff > 128 {
            pixels[i] = (255 - diff, 0, 0);
        } else {
            pixels[i] = (0, diff * 2, 0);
        }
    }
    (pixels, width, height)
}

fn map_frequency(data: &[u8]) -> (Vec<(u8, u8, u8)>, u32, u32) {
    let mut pixels = vec![(0u8, 0u8, 0u8); 256 * 256];
    let mut counts = [[0u32; 256]; 256];

    for w in data.windows(2) {
        counts[w[0] as usize][w[1] as usize] += 1;
    }

    let max = counts.iter().flatten().max().copied().unwrap_or(1).max(1);

    for y in 0..256 {
        for x in 0..256 {
            let v = ((counts[y][x] as f32 / max as f32).sqrt() * 255.0) as u8;
            pixels[y * 256 + x] = heatmap(v);
        }
    }
    (pixels, 256, 256)
}

fn heatmap(v: u8) -> (u8, u8, u8) {
    match v {
        0..=31 => (0, 0, v * 4),
        32..=95 => (0, (v - 32) * 4, 255),
        96..=159 => (0, 255, 255 - (v - 96) * 4),
        160..=223 => ((v - 160) * 4, 255, 0),
        224..=255 => (255, 255u8.saturating_sub((v - 224) * 8), 0),
    }
}

fn map_hilbert(data: &[u8]) -> (Vec<(u8, u8, u8)>, u32, u32) {
    let n = ((data.len() as f64).sqrt().ceil() as u32).next_power_of_two();
    let mut pixels = vec![(0u8, 0u8, 0u8); (n * n) as usize];

    for (i, &byte) in data.iter().enumerate() {
        let (x, y) = hilbert_d2xy(n, i as u32);
        if x < n && y < n {
            pixels[(y * n + x) as usize] = (byte, byte, byte);
        }
    }
    (pixels, n, n)
}

fn hilbert_d2xy(n: u32, d: u32) -> (u32, u32) {
    let mut x = 0u32;
    let mut y = 0u32;
    let mut s = 1u32;
    let mut d = d;

    while s < n {
        let rx = 1 & (d / 2);
        let ry = 1 & (d ^ rx);
        
        if ry == 0 {
            if rx == 1 {
                x = s - 1 - x;
                y = s - 1 - y;
            }
            std::mem::swap(&mut x, &mut y);
        }
        
        x += s * rx;
        y += s * ry;
        d /= 4;
        s *= 2;
    }
    (x, y)
}

fn map_ascii(data: &[u8], width: u32) -> (Vec<(u8, u8, u8)>, u32, u32) {
    let height = (data.len() as u32 + width - 1) / width;
    let mut pixels = vec![(0u8, 0u8, 0u8); (width * height) as usize];

    for (i, &byte) in data.iter().enumerate() {
        pixels[i] = match byte {
            0x00 => (20, 20, 40),              // Null = dark blue
            0x20..=0x7E => (0, 255, 0),        // Printable = green
            0x09 | 0x0A | 0x0D => (0, 200, 0), // Whitespace
            0x80..=0xFF => (255, 0, 0),        // High bytes = red
            _ => (100, 100, 100),              // Control chars
        };
    }
    (pixels, width, height)
}

fn map_null(data: &[u8], width: u32) -> (Vec<(u8, u8, u8)>, u32, u32) {
    let height = (data.len() as u32 + width - 1) / width;
    let mut pixels = vec![(0u8, 0u8, 0u8); (width * height) as usize];

    for (i, &byte) in data.iter().enumerate() {
        pixels[i] = match byte {
            0x00 => (255, 0, 0),   // Null = red
            0xFF => (0, 0, 255),   // 0xFF = blue
            _ => (byte / 2, byte / 2, byte / 2),
        };
    }
    (pixels, width, height)
}

fn map_high_entropy(data: &[u8], window: usize, width: u32) -> (Vec<(u8, u8, u8)>, u32, u32) {
    let height = (data.len() as u32 + width - 1) / width;
    let mut pixels = vec![(0u8, 0u8, 0u8); (width * height) as usize];

    for i in 0..data.len() {
        let start = i.saturating_sub(window / 2);
        let end = (i + window / 2).min(data.len());
        let entropy = calc_entropy(&data[start..end]);
        
        pixels[i] = if entropy > 7.5 {
            (255, 0, 0)     // Encrypted
        } else if entropy > 6.5 {
            (255, 165, 0)   // Compressed
        } else if entropy > 4.0 {
            (255, 255, 0)   // Code
        } else if entropy > 2.0 {
            (0, 255, 0)     // Text
        } else {
            (0, 0, 255)     // Padding
        };
    }
    (pixels, width, height)
}
