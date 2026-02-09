//! Braille Matrix Rain - Sub-pixel resolution Matrix effect
//! 
//! Uses Unicode Braille patterns (U+2800-U+28FF) to achieve 8× resolution:
//! - Each Braille character is a 2×4 dot grid
//! - Screen becomes 320 "sub-columns" × 200 "sub-rows" (at 1280×800)
//! - Rain falls at sub-pixel precision for ultra-smooth effect

use alloc::vec::Vec;
use alloc::vec;

// ═══════════════════════════════════════════════════════════════════════════════
// CONSTANTS
// ═══════════════════════════════════════════════════════════════════════════════

/// Braille character dimensions - 6x6 dots with spacing
pub const BRAILLE_DOTS_W: usize = 2;   // 2 dots wide per character
pub const BRAILLE_DOTS_H: usize = 4;   // 4 dots tall per character
pub const CELL_PIXEL_W: usize = 16;    // 16 pixels wide per cell (2 × 8px spacing)
pub const CELL_PIXEL_H: usize = 32;    // 32 pixels tall per cell (4 × 8px spacing)

/// Trail length in dots - long for dramatic effect
const TRAIL_LENGTH: i32 = 64;

/// Intensity lookup table for trail fade (64 entries for long trails)
/// Extended with more gradual fade to gray instead of black
const INTENSITY_LUT: [u8; 64] = [
    255, 252, 248, 244, 240, 236, 231, 226,
    221, 216, 210, 204, 198, 192, 186, 179,
    172, 165, 158, 151, 144, 137, 130, 123,
    116, 109, 102, 96, 90, 84, 78, 72,
    67, 62, 57, 52, 48, 44, 40, 36,
    33, 30, 27, 24, 22, 20, 18, 16,
    14, 13, 12, 11, 10, 9, 8, 7,
    6, 5, 5, 4, 4, 3, 3, 2,  // Never goes to 0 - always some glow!
];

/// Pre-computed color lookup table for O(1) color mapping
/// 256 entries: intensity -> 0xAARRGGBB color
/// ENHANCED: Cyan-white heads, richer green gradient, blue-tinted shadows, ambient glow
const fn generate_color_lut() -> [u32; 256] {
    let mut lut = [0xFF000000u32; 256];
    let mut i = 1u32;
    while i < 256 {
        let color = if i > 250 {
            // Brilliant white-cyan head (phosphor overload)
            let t = i - 250; // 0-5
            let r = 180 + t * 15; // 180-255
            let g = 255;
            let b = 220 + t * 7; // 220-255
            (0xFF << 24) | (r << 16) | (g << 8) | b
        } else if i > 235 {
            // Cyan-white transition (electric feel)
            let t = i - 235; // 0-15
            let r = 60 + t * 8; // 60-180
            let g = 220 + t * 2; // 220-250
            let b = 120 + t * 6; // 120-210
            (0xFF << 24) | (r << 16) | (g << 8) | b
        } else if i > 200 {
            // Bright lime-cyan
            let t = i - 200; // 0-35
            let r = t; // 0-35
            let g = 180 + t * 2; // 180-250
            let b = 30 + t * 2; // 30-100
            (0xFF << 24) | (r << 16) | (g << 8) | b
        } else if i > 160 {
            // Matrix green (classic bright)
            let t = i - 160; // 0-40
            let g = 140 + t; // 140-180
            let r = t / 6; // subtle warmth
            let b = 10 + t / 3; // slight cyan
            (0xFF << 24) | (r << 16) | (g << 8) | b
        } else if i > 120 {
            // Green with cyan undertone
            let t = i - 120; // 0-40
            let g = 100 + t; // 100-140
            let b = 8 + t / 3; // 8-21 cyan tint
            let r = t / 12;
            (0xFF << 24) | (r << 16) | (g << 8) | b
        } else if i > 80 {
            // Forest green
            let t = i - 80; // 0-40
            let g = 60 + t; // 60-100
            let b = 4 + t / 5; // slight blue-green
            (0xFF << 24) | (g << 8) | b
        } else if i > 50 {
            // Dark teal-green
            let t = i - 50; // 0-30
            let g = 30 + t; // 30-60
            let b = 3 + t / 6; // subtle blue
            (0xFF << 24) | (g << 8) | b
        } else if i > 25 {
            // Dark green with blue-green ambient
            let t = i - 25; // 0-25
            let g = 12 + t; // 12-37
            let b = 2 + t / 5; // 2-7
            (0xFF << 24) | (g << 8) | b
        } else if i > 10 {
            // Dim gray-green
            let t = i - 10; // 0-15
            let g = 5 + t / 2; // 5-12
            let b = 1 + t / 8;
            (0xFF << 24) | (g << 8) | b
        } else {
            // Minimum glow - dark blue-green haze (never pure black)
            let g = 2 + i / 3;
            let b = 1 + i / 5;
            (0xFF << 24) | (g << 8) | b
        };
        lut[i as usize] = color;
        i += 1;
    }
    lut
}

/// Static color lookup - computed at compile time!
static COLOR_LUT: [u32; 256] = generate_color_lut();

/// Fast color lookup using pre-computed LUT - O(1) operation!
#[inline(always)]
pub(crate) fn intensity_to_color(intensity: u8) -> u32 {
    COLOR_LUT[intensity as usize]
}

/// Matrix-style 6x6 pixel glyphs - inspired by the iconic Matrix digital rain
/// Each glyph is 6 rows, each row is 6 bits (stored in lower 6 bits of u8)
/// Format: [row0, row1, row2, row3, row4, row5] - top to bottom
/// Bits: 0b00FEDCBA where A=leftmost pixel, F=rightmost pixel
pub(crate) const MATRIX_GLYPHS_6X6: [[u8; 6]; 64] = [
    // 0 - Zero
    [0b011110, 0b100001, 0b100101, 0b101001, 0b100001, 0b011110],
    // 1 - One  
    [0b001100, 0b010100, 0b000100, 0b000100, 0b000100, 0b011111],
    // 2 - Two (stylized S)
    [0b011110, 0b000001, 0b001110, 0b010000, 0b100000, 0b111111],
    // 3 - Three
    [0b111110, 0b000001, 0b001110, 0b000001, 0b000001, 0b111110],
    // 4 - Four
    [0b100010, 0b100010, 0b100010, 0b111111, 0b000010, 0b000010],
    // 5 - Five (stylized)
    [0b111111, 0b100000, 0b111110, 0b000001, 0b000001, 0b111110],
    // 6 - Six (curved)
    [0b011110, 0b100000, 0b111110, 0b100001, 0b100001, 0b011110],
    // 7 - Seven
    [0b111111, 0b000001, 0b000010, 0b000100, 0b001000, 0b001000],
    // 8 - Eight
    [0b011110, 0b100001, 0b011110, 0b100001, 0b100001, 0b011110],
    // 9 - Nine
    [0b011110, 0b100001, 0b100001, 0b011111, 0b000001, 0b011110],
    // 10 - Katakana ア (a)
    [0b111111, 0b000010, 0b000100, 0b001000, 0b010000, 0b100000],
    // 11 - Katakana イ (i)
    [0b000010, 0b111110, 0b000010, 0b000100, 0b001000, 0b010000],
    // 12 - Katakana ウ (u)
    [0b001100, 0b111111, 0b100001, 0b100001, 0b010010, 0b001100],
    // 13 - Katakana エ (e)
    [0b111111, 0b000100, 0b000100, 0b000100, 0b000100, 0b111111],
    // 14 - Katakana オ (o)
    [0b000100, 0b111111, 0b000100, 0b001010, 0b010001, 0b100000],
    // 15 - Katakana カ (ka)
    [0b001000, 0b111111, 0b001001, 0b001010, 0b011100, 0b001000],
    // 16 - Katakana キ (ki)
    [0b000100, 0b111111, 0b000100, 0b111111, 0b000100, 0b000100],
    // 17 - Katakana ク (ku)
    [0b011110, 0b000010, 0b000100, 0b001000, 0b010000, 0b100000],
    // 18 - Katakana ケ (ke)
    [0b001000, 0b111111, 0b000100, 0b000010, 0b000001, 0b000000],
    // 19 - Katakana コ (ko)
    [0b111111, 0b000001, 0b000001, 0b000001, 0b000001, 0b111111],
    // 20 - Katakana サ (sa)
    [0b010010, 0b111111, 0b010010, 0b000100, 0b001000, 0b110000],
    // 21 - Katakana シ (shi)
    [0b100000, 0b100100, 0b000010, 0b000001, 0b000010, 0b011100],
    // 22 - Katakana ス (su)
    [0b111111, 0b000010, 0b000100, 0b001010, 0b010001, 0b100000],
    // 23 - Katakana セ (se)
    [0b010000, 0b111111, 0b010010, 0b010100, 0b011000, 0b010000],
    // 24 - Katakana ソ (so)
    [0b100010, 0b010010, 0b000100, 0b001000, 0b010000, 0b100000],
    // 25 - Katakana タ (ta)
    [0b001100, 0b111111, 0b000100, 0b111111, 0b001000, 0b110000],
    // 26 - Katakana チ (chi)
    [0b111111, 0b000100, 0b111111, 0b000100, 0b001000, 0b110000],
    // 27 - Katakana ツ (tsu)
    [0b100010, 0b010010, 0b000100, 0b000100, 0b001000, 0b110000],
    // 28 - Katakana テ (te)
    [0b111111, 0b000100, 0b000100, 0b000100, 0b001000, 0b110000],  
    // 29 - Katakana ト (to)
    [0b100000, 0b111100, 0b100000, 0b100000, 0b010000, 0b001111],
    // 30 - Katakana ナ (na)
    [0b000100, 0b111111, 0b000100, 0b001000, 0b010000, 0b100000],
    // 31 - Katakana ニ (ni)
    [0b111111, 0b000000, 0b000000, 0b000000, 0b000000, 0b111111],
    // 32 - Diamond
    [0b000100, 0b001010, 0b010001, 0b010001, 0b001010, 0b000100],
    // 33 - Triangle up
    [0b000100, 0b001010, 0b010001, 0b100001, 0b111111, 0b000000],
    // 34 - Triangle down
    [0b111111, 0b100001, 0b010001, 0b001010, 0b000100, 0b000000],
    // 35 - Arrow up
    [0b000100, 0b001110, 0b010101, 0b000100, 0b000100, 0b000100],
    // 36 - Arrow down  
    [0b000100, 0b000100, 0b000100, 0b010101, 0b001110, 0b000100],
    // 37 - Horizontal lines
    [0b111111, 0b000000, 0b111111, 0b000000, 0b111111, 0b000000],
    // 38 - Vertical lines
    [0b010010, 0b010010, 0b010010, 0b010010, 0b010010, 0b010010],
    // 39 - Cross
    [0b000100, 0b000100, 0b111111, 0b000100, 0b000100, 0b000100],
    // 40 - X mark
    [0b100001, 0b010010, 0b001100, 0b001100, 0b010010, 0b100001],
    // 41 - Circle
    [0b011110, 0b100001, 0b100001, 0b100001, 0b100001, 0b011110],
    // 42 - Square
    [0b111111, 0b100001, 0b100001, 0b100001, 0b100001, 0b111111],
    // 43 - Half circle top
    [0b011110, 0b100001, 0b100001, 0b000000, 0b000000, 0b000000],
    // 44 - Half circle bottom
    [0b000000, 0b000000, 0b000000, 0b100001, 0b100001, 0b011110],
    // 45 - Curved left
    [0b000011, 0b001100, 0b010000, 0b010000, 0b001100, 0b000011],
    // 46 - Curved right
    [0b110000, 0b001100, 0b000010, 0b000010, 0b001100, 0b110000],
    // 47 - Wave
    [0b000000, 0b011000, 0b100100, 0b000010, 0b000001, 0b000000],
    // 48 - Double wave
    [0b011000, 0b100100, 0b011000, 0b000110, 0b001001, 0b000110],
    // 49 - Spiral (ロ style)
    [0b111111, 0b100001, 0b101101, 0b101101, 0b100001, 0b111111],
    // 50 - Omega-like
    [0b011110, 0b100001, 0b100001, 0b100001, 0b010010, 0b101101],
    // 51 - Lambda  
    [0b100001, 0b010010, 0b001100, 0b001100, 0b000100, 0b000100],
    // 52 - Sigma
    [0b111111, 0b100000, 0b010000, 0b010000, 0b100000, 0b111111],
    // 53 - Pi
    [0b111111, 0b010010, 0b010010, 0b010010, 0b010010, 0b010010],
    // 54 - Theta
    [0b011110, 0b100001, 0b111111, 0b100001, 0b100001, 0b011110],
    // 55 - Phi
    [0b000100, 0b011110, 0b100101, 0b100101, 0b011110, 0b000100],
    // 56 - Small dot
    [0b000000, 0b000000, 0b001100, 0b001100, 0b000000, 0b000000],
    // 57 - Colon
    [0b000000, 0b001100, 0b001100, 0b000000, 0b001100, 0b001100],
    // 58 - Asterisk
    [0b000100, 0b010101, 0b001110, 0b001110, 0b010101, 0b000100],
    // 59 - Hash
    [0b010010, 0b111111, 0b010010, 0b010010, 0b111111, 0b010010],
    // 60 - Percent
    [0b110001, 0b110010, 0b000100, 0b001000, 0b010011, 0b100011],
    // 61 - Bracket left
    [0b001110, 0b001000, 0b001000, 0b001000, 0b001000, 0b001110],
    // 62 - Bracket right
    [0b011100, 0b000100, 0b000100, 0b000100, 0b000100, 0b011100],
    // 63 - Full block
    [0b111111, 0b111111, 0b111111, 0b111111, 0b111111, 0b111111],
];

/// Number of available glyphs
const GLYPH_COUNT: usize = 64;

/// Matrix character set for classic look
pub(crate) const MATRIX_CHARSET: &[u8] = b"0123456789ABCDEF@#$%&*<>[]{}|/\\";

// ═══════════════════════════════════════════════════════════════════════════════
// GLYPH MATRIX RAIN - 6x6 Matrix characters with multiple drops per column
// ═══════════════════════════════════════════════════════════════════════════════

/// Trail length bounds - LONGER for denser rain
const MIN_TRAIL_LENGTH: usize = 15;
const MAX_TRAIL_LENGTH: usize = 50;

/// Maximum drops per column - MORE for denser rain
const DROPS_PER_COLUMN: usize = 6;

/// Total columns for 1280px width
const MAX_COLUMNS: usize = 160;

/// A single rain drop
#[derive(Clone, Copy)]
struct RainDrop {
    /// Y position (in cells, can be negative when above screen)
    y: i32,
    /// Speed: frames between each move (1=fastest, 12=slowest)
    speed: u8,
    /// Delay counter for speed
    delay: u8,
    /// Trail length for this drop (varies per drop)
    trail_len: u8,
    /// Starting glyph index (randomized per drop)
    glyph_seed: u32,
    /// Is this drop active?
    active: bool,
}

impl RainDrop {
    fn new_inactive() -> Self {
        Self {
            y: -100,
            speed: 1,
            delay: 0,
            trail_len: MIN_TRAIL_LENGTH as u8,
            glyph_seed: 0,
            active: false,
        }
    }
    
    /// Get the bottom Y position of this drop's trail
    fn tail_y(&self) -> i32 {
        self.y - self.trail_len as i32
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// SHAPE OVERLAY SYSTEM - 3D forms traced by rain drops
// ═══════════════════════════════════════════════════════════════════════════════

/// Shape overlay modes - forms traced by rain
#[derive(Clone, Copy, PartialEq)]
pub enum ShapeOverlay {
    None,
    Cube,
    Sphere,
    Torus,
    DNA,
}

/// Maximum number of shape drops (points that trace the form)
const MAX_SHAPE_DROPS: usize = 144;

/// Maximum cube flow drops (rain that flows on cube edges)
const MAX_CUBE_FLOW_DROPS: usize = 500;  // Dense grid/hatching pattern on cube top face

/// A drop that traces a 3D shape edge
#[derive(Clone, Copy)]
struct ShapeDrop {
    /// Screen X position (pixel)
    col: i32,
    /// Screen Y position (pixel)
    row: i32,
    /// Previous X position (for trail direction)
    prev_col: i32,
    /// Previous Y position (for trail direction)
    prev_row: i32,
    /// Depth (Z) for brightness - 0.0=far/dim, 1.0=near/bright
    depth: f32,
    /// Progress along the edge (0.0 to 1.0)
    progress: f32,
    /// Edge index this drop follows
    edge_idx: u8,
    /// Trail length (in pixels)
    trail_len: u8,
    /// Speed along edge
    speed: f32,
    /// Glyph seed
    glyph_seed: u32,
}

/// A drop that flows on cube TOP face (the "rain deflection" effect)
/// When rain hits the diamond-shaped top, it flows diagonally outward
#[derive(Clone, Copy)]
struct CubeFlowDrop {
    /// Screen X position (pixel)
    screen_x: f32,
    /// Screen Y position (pixel)  
    screen_y: f32,
    /// Velocity X (pixels per frame) - diagonal direction
    vel_x: f32,
    /// Velocity Y (pixels per frame) - diagonal direction
    vel_y: f32,
    /// Trail length
    trail_len: u8,
    /// Is this drop active?
    active: bool,
    /// Glyph seed for variation
    glyph_seed: u32,
    /// Life remaining (1.0 = full, 0.0 = dead)
    life: f32,
}

impl ShapeDrop {
    fn new() -> Self {
        Self {
            col: 0,
            row: 0,
            prev_col: 0,
            prev_row: 0,
            depth: 0.5,
            progress: 0.0,
            edge_idx: 0,
            trail_len: 6,
            speed: 0.012,
            glyph_seed: 0,
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// PARALLEL BRAILLE RENDERING - Split columns across cores
// ═══════════════════════════════════════════════════════════════════════════════

/// Parameters for parallel braille column rendering
#[repr(C)]
struct BrailleParallelParams {
    buffer_ptr: *mut u32,
    buffer_len: usize,
    fb_width: usize,
    fb_height: usize,
    cell_size: usize,
    cell_rows: usize,
    // BrailleMatrix data pointers (immutable during render)
    drops_ptr: *const [RainDrop; DROPS_PER_COLUMN],
    col_depth_ptr: *const u8,
    num_cols: usize,
    // Pre-computed cube geometry
    cube_active: bool,
    cube_min_x: f32,
    cube_max_x: f32,
    cube_min_y: f32,
    cube_max_y: f32,
    top_edges: [(f32, f32, f32, f32); 4],
    left_edges: [(f32, f32, f32, f32); 4],
    right_edges: [(f32, f32, f32, f32); 4],
    pad_edges: [(f32, f32, f32, f32); 4],
}

unsafe impl Send for BrailleParallelParams {}
unsafe impl Sync for BrailleParallelParams {}

/// Draw 6x6 glyph at pixel position (raw pointer version for parallel render)
#[inline]
unsafe fn draw_glyph_6x6_raw(buffer: *mut u32, fb_width: usize, fb_height: usize,
                              px: usize, py: usize, glyph: &[u8; 6], color: u32) {
    if py >= fb_height || px >= fb_width { return; }
    let max_row = (fb_height - py).min(6);
    let max_col = (fb_width - px).min(6);
    for row in 0..max_row {
        let bits = glyph[row];
        if bits == 0 { continue; }
        let base = (py + row) * fb_width + px;
        if bits & 0b000001 != 0 && 0 < max_col { *buffer.add(base) = color; }
        if bits & 0b000010 != 0 && 1 < max_col { *buffer.add(base + 1) = color; }
        if bits & 0b000100 != 0 && 2 < max_col { *buffer.add(base + 2) = color; }
        if bits & 0b001000 != 0 && 3 < max_col { *buffer.add(base + 3) = color; }
        if bits & 0b010000 != 0 && 4 < max_col { *buffer.add(base + 4) = color; }
        if bits & 0b100000 != 0 && 5 < max_col { *buffer.add(base + 5) = color; }
    }
}

/// Point-in-quad winding test (free function for parallel use)
#[inline(always)]
fn point_in_quad_par(px: f32, py: f32, edges: &[(f32, f32, f32, f32); 4]) -> bool {
    let mut pos = 0u8;
    let mut neg = 0u8;
    for &(ex, ey, ox, oy) in edges.iter() {
        let cross = ex * (py - oy) - ey * (px - ox);
        if cross > 0.0 { pos += 1; }
        else if cross < 0.0 { neg += 1; }
    }
    pos == 0 || neg == 0
}

/// Parallel column renderer — called by each core
fn render_columns_parallel(start: usize, end: usize, data: *mut u8) {
    let p = unsafe { &*(data as *const BrailleParallelParams) };
    let cell_size = p.cell_size;
    
    for col in start..end {
        if col >= p.num_cols { break; }
        
        let depth = unsafe { *p.col_depth_ptr.add(col) };
        let depth_brightness = 100 + (depth as u32 * 155 / 255);
        
        let col_px = (col * cell_size + 4) as f32;
        let col_in_cube_x = p.cube_active && col_px >= p.cube_min_x && col_px <= p.cube_max_x;
        
        let drops = unsafe { &*p.drops_ptr.add(col) };
        
        for drop_idx in 0..DROPS_PER_COLUMN {
            let drop = &drops[drop_idx];
            if !drop.active { continue; }
            
            let head_y = drop.y;
            let trail_length = drop.trail_len as usize;
            
            for trail_pos in 0..trail_length {
                let cell_y = head_y - trail_pos as i32;
                if cell_y < 0 || cell_y >= p.cell_rows as i32 { continue; }
                
                let intensity_idx = (trail_pos * 63) / trail_length.max(1);
                let base_intensity = INTENSITY_LUT[intensity_idx.min(63)] as u32;
                let intensity = ((base_intensity * depth_brightness) / 255) as u8;
                
                if col_in_cube_x {
                    let py = (cell_y as usize * cell_size + 4) as f32;
                    if py >= p.cube_min_y && py <= p.cube_max_y {
                        if point_in_quad_par(col_px, py, &p.pad_edges) { continue; }
                        if point_in_quad_par(col_px, py, &p.top_edges)
                            || point_in_quad_par(col_px, py, &p.left_edges)
                            || point_in_quad_par(col_px, py, &p.right_edges) {
                            continue;
                        }
                    }
                }
                
                if intensity < 2 { continue; }
                
                let glyph_seed = drop.glyph_seed.wrapping_add(trail_pos as u32 * 2654435761);
                let glyph_idx = (glyph_seed % GLYPH_COUNT as u32) as usize;
                let glyph = &MATRIX_GLYPHS_6X6[glyph_idx];
                let color = intensity_to_color(intensity);
                
                let px = col * cell_size + 1;
                let py = cell_y as usize * cell_size + 1;
                unsafe {
                    draw_glyph_6x6_raw(p.buffer_ptr, p.fb_width, p.fb_height,
                                       px, py, glyph, color);
                }
                
                // HEAD GLOW
                if trail_pos == 0 && intensity > 200 {
                    let gx = px + 3;
                    let gy = py + 3;
                    let offsets: [(i32, i32); 4] = [(-1,0),(1,0),(0,-1),(0,1)];
                    for &(ox, oy) in &offsets {
                        let tx = gx as i32 + ox;
                        let ty = gy as i32 + oy;
                        if tx >= 0 && tx < p.fb_width as i32 && ty >= 0 && ty < p.fb_height as i32 {
                            let idx = ty as usize * p.fb_width + tx as usize;
                            unsafe {
                                let e = *p.buffer_ptr.add(idx);
                                let nr = (((e >> 16) & 0xFF) + 10).min(255);
                                let ng = (((e >> 8) & 0xFF) + 48).min(255);
                                let nb = ((e & 0xFF) + 32).min(255);
                                *p.buffer_ptr.add(idx) = 0xFF000000 | (nr << 16) | (ng << 8) | nb;
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Glyph Matrix Rain Renderer - Multiple drops per column with DEPTH effect
/// Closer columns (high depth) = denser rain, brighter colors
/// Far columns (low depth) = sparse rain, dimmer colors
pub struct BrailleMatrix {
    /// All drops: drops[col][drop_idx] - Vec to avoid stack overflow
    drops: Vec<[RainDrop; DROPS_PER_COLUMN]>,
    /// Depth per column: 0=far (dim, sparse), 255=close (bright, dense)
    col_depth: Vec<u8>,
    /// Global RNG seed
    rng: u32,
    /// Frame counter
    frame: u32,
    /// Number of active columns
    num_cols: usize,
    /// Number of rows on screen
    num_rows: usize,
    /// Shape overlay mode
    shape_mode: ShapeOverlay,
    /// Shape animation time
    shape_time: f32,
    /// Shape drops (traces the 3D form) - Vec to avoid stack overflow
    shape_drops: Vec<ShapeDrop>,
    /// Number of active shape drops
    shape_drop_count: usize,
    /// Cube flow drops (rain dripping on cube edges) - Vec to avoid stack overflow
    cube_flow_drops: Vec<CubeFlowDrop>,
}

impl BrailleMatrix {
    pub fn new() -> Self {
        use alloc::vec::Vec;
        
        let cols = 160; // 1280 / 8
        let rows = 100; // 800 / 8
        
        // Allocate on heap using Vec to avoid stack overflow
        let mut drops: Vec<[RainDrop; DROPS_PER_COLUMN]> = Vec::with_capacity(MAX_COLUMNS);
        let mut col_depth: Vec<u8> = vec![128u8; MAX_COLUMNS];
        let mut rng = 0xDEADBEEFu32;
        
        // Initialize drops vec with default values
        for _ in 0..MAX_COLUMNS {
            drops.push([RainDrop::new_inactive(); DROPS_PER_COLUMN]);
        }
        
        // Assign random depth per column (creates parallax lanes)
        // Use pseudo-noise pattern for natural look (no sin() needed in no_std)
        for col in 0..cols {
            rng = rng.wrapping_mul(1103515245).wrapping_add(12345);
            // Create depth bands: mix of random and position-based pattern
            // Prime-based modulo creates pseudo-wave without floating-point sin
            let pattern = ((col * 17 + 53) % 97) as i32 - 48; // -48 to +48
            let random = (rng % 100) as i32 - 50; // -50 to +50
            let base_depth = 145i32 + pattern + random;
            col_depth[col] = base_depth.clamp(30, 255) as u8;
        }
        
        // Initialize drops with staggered starting positions
        for col in 0..cols {
            let depth = col_depth[col];
            // Track cumulative offset to prevent overlap
            let mut next_start_offset: i32 = 0;
            
            for drop_idx in 0..DROPS_PER_COLUMN {
                rng = rng.wrapping_mul(1103515245).wrapping_add(12345);
                
                // Trail length based on depth: closer = longer trails
                let depth_factor = depth as f32 / 255.0;
                let _trail_range = MAX_TRAIL_LENGTH - MIN_TRAIL_LENGTH;
                let min_trail = (MIN_TRAIL_LENGTH as f32 * (0.5 + depth_factor * 0.5)) as usize;
                let max_trail = (MAX_TRAIL_LENGTH as f32 * (0.6 + depth_factor * 0.4)) as usize;
                let trail_len = min_trail + (rng % (max_trail - min_trail + 1) as u32) as usize;
                
                rng = rng.wrapping_mul(1103515245).wrapping_add(12345);
                
                // Gap based on depth: closer = MINIMAL gaps (ultra dense rain!)
                // Far (depth=30): gap 2-8, Close (depth=255): gap 0-2
                let gap_min = ((1.0 - depth_factor) * 2.0) as i32; // 0-2
                let gap_range = (2.0 + (1.0 - depth_factor) * 6.0).max(1.0) as i32; // 2-8
                let gap = gap_min + (rng % gap_range as u32) as i32;
                let start_y = next_start_offset - (rng % 8) as i32;  // Tighter spawn
                
                // Update offset for next drop: must wait until this trail passes
                next_start_offset = start_y - trail_len as i32 - gap;
                
                // Speed based on depth: closer = faster (1-2), far = slower (2-5)
                rng = rng.wrapping_mul(1103515245).wrapping_add(12345);
                let speed_min = (1.0 + (1.0 - depth_factor) * 1.5) as u8; // 1-2.5
                let speed_range = (2.0 + (1.0 - depth_factor) * 3.0) as u8; // 2-5
                let speed = speed_min + (rng % speed_range as u32) as u8;
                
                rng = rng.wrapping_mul(1103515245).wrapping_add(12345);
                
                drops[col][drop_idx] = RainDrop {
                    y: start_y,
                    speed,
                    delay: (rng % speed as u32) as u8,
                    trail_len: trail_len as u8,
                    glyph_seed: rng,
                    active: true,
                };
            }
        }
        
        // Initialize shape_drops on heap
        let mut shape_drops: Vec<ShapeDrop> = Vec::with_capacity(MAX_SHAPE_DROPS);
        for _ in 0..MAX_SHAPE_DROPS {
            shape_drops.push(ShapeDrop::new());
        }
        
        // Initialize cube_flow_drops on heap - all INACTIVE, spawned by rain contact
        let mut cube_flow_drops: Vec<CubeFlowDrop> = Vec::with_capacity(MAX_CUBE_FLOW_DROPS);
        for _ in 0..MAX_CUBE_FLOW_DROPS {
            cube_flow_drops.push(CubeFlowDrop {
                screen_x: 0.0,
                screen_y: 0.0,
                vel_x: 0.0,
                vel_y: 0.0,
                trail_len: 5,
                active: false,  // Inactive until triggered by rain
                glyph_seed: 0,
                life: 0.0,
            });
        }
        
        Self {
            drops,
            col_depth,
            rng,
            frame: 0,
            num_cols: cols,
            num_rows: rows,
            shape_mode: ShapeOverlay::None,  // No shape by default
            shape_time: 0.0,
            shape_drops,
            shape_drop_count: 0,
            cube_flow_drops,
        }
    }
    
    /// Set shape overlay mode
    pub fn set_shape(&mut self, mode: ShapeOverlay) {
        self.shape_mode = mode;
        self.shape_time = 0.0;
        
        // Reset cube flow drops when cube mode is enabled
        if mode == ShapeOverlay::Cube {
            for i in 0..MAX_CUBE_FLOW_DROPS {
                self.cube_flow_drops[i].active = false;
                self.cube_flow_drops[i].life = 0.0;
            }
        }
        
        if mode == ShapeOverlay::None {
            self.shape_drop_count = 0;
            return;
        }
        
        // Initialize shape drops based on mode
        let mut rng = 0x12345678u32;
        let drop_count = match mode {
            ShapeOverlay::Cube => 90,      // 9 visible edges × 10 drops per edge
            ShapeOverlay::Sphere => 64,    // Points on surface
            ShapeOverlay::Torus => 80,     // Double loop
            ShapeOverlay::DNA => 64,       // Helix points
            ShapeOverlay::None => 0,
        };
        
        self.shape_drop_count = drop_count.min(MAX_SHAPE_DROPS);
        
        for i in 0..self.shape_drop_count {
            rng = rng.wrapping_mul(1103515245).wrapping_add(12345);
            self.shape_drops[i] = ShapeDrop {
                col: 0,
                row: 0,
                prev_col: 0,
                prev_row: 0,
                depth: 0.5,
                progress: (i as f32 / self.shape_drop_count as f32),
                edge_idx: (i % 12) as u8,
                trail_len: 5 + (rng % 4) as u8,
                speed: 0.006 + (rng % 50) as f32 / 5000.0,
                glyph_seed: rng,
            };
        }
    }
    
    /// Get current shape mode
    pub fn get_shape(&self) -> ShapeOverlay {
        self.shape_mode
    }
    
    /// Fast sin approximation
    #[inline(always)]
    fn fast_sin(x: f32) -> f32 {
        let x = x % 6.28318;
        let x = if x > 3.14159 { x - 6.28318 } else if x < -3.14159 { x + 6.28318 } else { x };
        let x2 = x * x;
        x * (1.0 - x2 / 6.0 + x2 * x2 / 120.0)
    }
    
    /// Fast cos approximation
    #[inline(always)]
    fn fast_cos(x: f32) -> f32 {
        Self::fast_sin(x + 1.5708)
    }
    
    /// Fast square root approximation (Newton-Raphson)
    #[inline(always)]
    fn fast_sqrt(x: f32) -> f32 {
        if x <= 0.0 { return 0.0; }
        let mut guess = x * 0.5;
        guess = 0.5 * (guess + x / guess);
        guess = 0.5 * (guess + x / guess);
        guess
    }
    
    /// Fast floor - works for both positive and negative values
    #[inline(always)]
    fn fast_floor(x: f32) -> f32 {
        let i = x as i32;
        if (i as f32) > x { (i - 1) as f32 } else { i as f32 }
    }

    /// Update all rain drops
    pub fn update(&mut self) {
        self.frame = self.frame.wrapping_add(1);
        
        let max_y = (self.num_rows as i32) + MAX_TRAIL_LENGTH as i32 + 10;
        
        for col in 0..self.num_cols {
            let depth = self.col_depth[col];
            let depth_factor = depth as f32 / 255.0;
            
            // First pass: collect info about all drops and determine if any need reset
            let mut needs_reset: [bool; DROPS_PER_COLUMN] = [false; DROPS_PER_COLUMN];
            let mut min_tail_y: i32 = 0;
            
            for drop_idx in 0..DROPS_PER_COLUMN {
                let drop = &self.drops[col][drop_idx];
                if drop.active {
                    let tail = drop.tail_y();
                    if tail < min_tail_y {
                        min_tail_y = tail;
                    }
                }
            }
            
            // Second pass: update positions
            for drop_idx in 0..DROPS_PER_COLUMN {
                let drop = &mut self.drops[col][drop_idx];
                
                if !drop.active {
                    continue;
                }
                
                // Speed-based movement
                drop.delay = drop.delay.wrapping_add(1);
                if drop.delay >= drop.speed {
                    drop.delay = 0;
                    drop.y += 1;
                    
                    // Update glyph seed for animation
                    drop.glyph_seed = drop.glyph_seed.wrapping_mul(1103515245).wrapping_add(12345);
                }
                
                // Mark for reset if off screen
                if drop.y > max_y {
                    needs_reset[drop_idx] = true;
                }
            }
            
            // Third pass: reset drops that went off screen
            for drop_idx in 0..DROPS_PER_COLUMN {
                if !needs_reset[drop_idx] {
                    continue;
                }
                
                // Recalculate min_tail_y excluding drops being reset
                let mut current_min_tail: i32 = 0;
                for other_idx in 0..DROPS_PER_COLUMN {
                    if other_idx != drop_idx && !needs_reset[other_idx] {
                        let drop = &self.drops[col][other_idx];
                        if drop.active {
                            let tail = drop.tail_y();
                            if tail < current_min_tail {
                                current_min_tail = tail;
                            }
                        }
                    }
                }
                
                self.rng = self.rng.wrapping_mul(1103515245).wrapping_add(12345);
                    
                // Trail length based on depth: closer = longer trails
                let min_trail = (MIN_TRAIL_LENGTH as f32 * (0.5 + depth_factor * 0.5)) as usize;
                let max_trail = (MAX_TRAIL_LENGTH as f32 * (0.6 + depth_factor * 0.4)) as usize;
                let new_trail = min_trail + (self.rng % (max_trail - min_trail + 1).max(1) as u32) as usize;
                
                self.rng = self.rng.wrapping_mul(1103515245).wrapping_add(12345);
                    
                // Gap based on depth: closer = MINIMAL gaps (ultra dense!)
                let gap_min = ((1.0 - depth_factor) * 2.0) as i32; // 0-2
                let gap_range = (2.0 + (1.0 - depth_factor) * 6.0).max(1.0) as i32; // 2-8
                let gap = gap_min + (self.rng % gap_range as u32) as i32;
                    
                // Start above the lowest other drop's tail - tighter spawn
                let new_y = current_min_tail - new_trail as i32 - gap - (self.rng % 5) as i32;
                
                self.rng = self.rng.wrapping_mul(1103515245).wrapping_add(12345);
                    
                // Speed based on depth: closer = faster (1-2), far = slower (2-5)
                let speed_min = (1.0 + (1.0 - depth_factor) * 1.5) as u8;
                let speed_range = (2.0 + (1.0 - depth_factor) * 3.0).max(1.0) as u8;
                let new_speed = speed_min + (self.rng % speed_range as u32) as u8;
                
                self.rng = self.rng.wrapping_mul(1103515245).wrapping_add(12345);
                
                let drop = &mut self.drops[col][drop_idx];
                drop.trail_len = new_trail as u8;
                drop.y = new_y;
                drop.speed = new_speed;
                drop.delay = 0;
                drop.glyph_seed = self.rng;
            }
        }
        
        // Update shape overlay drops
        if self.shape_mode != ShapeOverlay::None && self.shape_drop_count > 0 {
            self.shape_time += 0.016; // ~60 FPS step
            
            // Center in pixels (assuming 1280x800 or similar)
            let center_x = (self.num_cols * 8 / 2) as f32;  // Pixel center X
            let center_y = (self.num_rows * 8 / 2) as f32;  // Pixel center Y
            let scale = ((self.num_rows * 8) as f32).min((self.num_cols * 8) as f32) * 0.18;  // MUST match entity layer
            
            let count = self.shape_drop_count.min(MAX_SHAPE_DROPS);
            for i in 0..count {
                let drop = &mut self.shape_drops[i];
                
                // Advance progress along edge
                drop.progress += drop.speed;
                if drop.progress >= 1.0 {
                    drop.progress -= 1.0;
                    drop.edge_idx = (drop.edge_idx + 1) % 9;  // 9 visible edges
                    drop.glyph_seed = drop.glyph_seed.wrapping_mul(1103515245).wrapping_add(12345);
                }
                
                // Calculate 3D position based on shape mode
                let (x, y, z) = match self.shape_mode {
                    ShapeOverlay::Cube => {
                        // Static wireframe cube - 9 visible edges (no rotation)
                        let angle_y = 0.785398_f32;  // 45° - MUST match entity layer
                        let angle_x = 0.523599_f32;  // 30° - MUST match entity layer
                        let cos_y = Self::fast_cos(angle_y);
                        let sin_y = Self::fast_sin(angle_y);
                        let cos_x = Self::fast_cos(angle_x);
                        let sin_x = Self::fast_sin(angle_x);
                        
                        // Cube vertices (unit cube centered at origin)
                        let verts: [(f32, f32, f32); 8] = [
                            (-1.0, -1.0, -1.0), (1.0, -1.0, -1.0),
                            (1.0, 1.0, -1.0), (-1.0, 1.0, -1.0),
                            (-1.0, -1.0, 1.0), (1.0, -1.0, 1.0),
                            (1.0, 1.0, 1.0), (-1.0, 1.0, 1.0),
                        ];
                        
                        // 12 edges of the cube wireframe
                        // 9 visible edges only (back edges hidden in isometric view)
                        let edges: [(usize, usize); 9] = [
                            (0,1), (0,4), (0,3),   // top-back vertex
                            (1,5),                  // top-right
                            (4,5), (4,7),           // front-top
                            (5,6),                  // right-front vertical
                            (3,7),                  // left vertical
                            (6,7),                  // front-bottom
                        ];
                        
                        let edge = edges[drop.edge_idx as usize % 9];
                        let v1 = verts[edge.0];
                        let v2 = verts[edge.1];
                        
                        // Interpolate along edge
                        let t = drop.progress;
                        let px = v1.0 + (v2.0 - v1.0) * t;
                        let py = v1.1 + (v2.1 - v1.1) * t;
                        let pz = v1.2 + (v2.2 - v1.2) * t;
                        
                        // Rotate around Y axis first
                        let rx1 = px * cos_y - pz * sin_y;
                        let rz1 = px * sin_y + pz * cos_y;
                        
                        // Then rotate around X axis
                        let ry2 = py * cos_x - rz1 * sin_x;
                        let rz2 = py * sin_x + rz1 * cos_x;
                        
                        // Perspective projection (camera at z=5, same as entity layer)
                        let cam_dist = 5.0;
                        let proj_z = rz2 + cam_dist;
                        let perspective = cam_dist / proj_z.max(0.5);
                        
                        // Project to screen (centered)
                        let screen_x = center_x + rx1 * scale * perspective;
                        let screen_y = center_y + ry2 * scale * perspective;
                        
                        // Depth for brightness: normalize Z to 0-1 range
                        let depth_normalized = (rz2 + 2.0) / 4.0;  // -2 to +2 -> 0 to 1
                        
                        (screen_x as i32, screen_y as i32, depth_normalized)
                    },
                    ShapeOverlay::Sphere => {
                        // Points on rotating sphere
                        let phi = (i as f32 / self.shape_drop_count as f32) * 3.14159 * 2.0;
                        let theta = drop.progress * 3.14159;
                        let angle = self.shape_time * 0.5;
                        
                        let x = Self::fast_sin(theta) * Self::fast_cos(phi + angle);
                        let y = Self::fast_sin(theta) * Self::fast_sin(phi + angle);
                        let z = Self::fast_cos(theta);
                        
                        let perspective = 2.0 / (3.0 + z * 0.5);
                        let depth = (z + 1.0) / 2.0;
                        ((center_x + x * scale * perspective) as i32,
                         (center_y + y * scale * perspective * 0.7) as i32, depth)
                    },
                    ShapeOverlay::Torus => {
                        // Double helix / torus
                        let u = drop.progress * 6.28318;
                        let v = (i as f32 / self.shape_drop_count as f32) * 6.28318;
                        let angle = self.shape_time * 0.4;
                        
                        let r1 = 1.5;
                        let r2 = 0.5;
                        let x = (r1 + r2 * Self::fast_cos(v)) * Self::fast_cos(u + angle);
                        let y = (r1 + r2 * Self::fast_cos(v)) * Self::fast_sin(u + angle);
                        let z = r2 * Self::fast_sin(v);
                        
                        let perspective = 1.5 / (2.5 + z * 0.3);
                        let depth = (z + 0.5) / 1.0;
                        ((center_x + x * scale * 0.6 * perspective) as i32,
                         (center_y + y * scale * 0.4 * perspective) as i32, depth)
                    },
                    ShapeOverlay::DNA => {
                        // DNA double helix
                        let t = drop.progress * 10.0 + (i as f32 * 0.1);
                        let angle = self.shape_time * 0.6;
                        let helix = if i % 2 == 0 { 1.0 } else { -1.0 };
                        
                        let x = Self::fast_cos(t + angle) * helix;
                        let y = (t % 6.28318) / 3.14159 - 1.0;
                        let z = Self::fast_sin(t + angle) * helix;
                        
                        let perspective = 2.0 / (3.0 + z * 0.5);
                        let depth = (z + 1.0) / 2.0;
                        ((center_x + x * scale * 0.5 * perspective) as i32,
                         (center_y + y * scale * 0.8) as i32, depth)
                    },
                    ShapeOverlay::None => (0, 0, 0.5),
                };
                
                // Store previous position for trail direction
                drop.prev_col = drop.col;
                drop.prev_row = drop.row;
                drop.col = x;
                drop.row = y;
                drop.depth = z.clamp(0.0, 1.0);
            }
        }
        
        // Note: cube flow drops are now procedural (rendered in render_cube_flow_layer)
        // No particle-based update needed — just advance shape_time
    }
    
    /// Render to framebuffer - OPTIMIZED for maximum FPS
    /// Strategy: Only render active drops, skip empty cells entirely
    pub fn render(&self, buffer: &mut [u32], fb_width: usize, fb_height: usize) {
        // Fast SSE2 fill with dark blue-tinted base (cinematic feel)
        let bg_color = 0xFF010203u32;
        #[cfg(target_arch = "x86_64")]
        unsafe {
            crate::graphics::simd::fill_row_sse2(buffer.as_mut_ptr(), buffer.len(), bg_color);
        }
        #[cfg(not(target_arch = "x86_64"))]
        buffer.fill(bg_color);
        
        let cell_size = 8usize;
        let cell_cols = fb_width / cell_size;
        let cell_rows = fb_height / cell_size;
        
        // Pre-compute cube TOP FACE diamond shape for masking and glow
        let cube_active = self.shape_mode == ShapeOverlay::Cube;
        let center_x = (fb_width / 2) as f32;
        let center_y = (fb_height / 2) as f32;
        let scale = (fb_height as f32).min(fb_width as f32) * 0.18;
        
        // Isometric angles (same as entity layer)
        let angle_y = 0.785398_f32; // 45°
        let angle_x = 0.523599_f32; // 30°
        let cos_y = Self::fast_cos(angle_y);
        let sin_y = Self::fast_sin(angle_y);
        let cos_x = Self::fast_cos(angle_x);
        let sin_x = Self::fast_sin(angle_x);
        
        // Project top face vertices to screen
        let project_top = |x3d: f32, z3d: f32| -> (f32, f32) {
            let y3d = -1.0; // Top face
            let rx1 = x3d * cos_y - z3d * sin_y;
            let rz1 = x3d * sin_y + z3d * cos_y;
            let ry2 = y3d * cos_x - rz1 * sin_x;
            let rz2 = y3d * sin_x + rz1 * cos_x;
            let cam_dist = 5.0;
            let proj_z = rz2 + cam_dist;
            let perspective = cam_dist / proj_z.max(1.0);
            (center_x + rx1 * scale * perspective, center_y + ry2 * scale * perspective)
        };
        
        // Diamond vertices of top face
        let (top_back_x, top_back_y) = project_top(0.0, -1.0);
        let (top_left_x, top_left_y) = project_top(-1.0, 0.0);
        let (top_right_x, top_right_y) = project_top(1.0, 0.0);
        let (top_front_x, top_front_y) = project_top(0.0, 1.0);
        
        // PRE-COMPUTE cube silhouette for masking - project all 8 vertices ONCE
        // Cube vertices: (±1, ±1, ±1)
        let project_cv = |x3d: f32, y3d: f32, z3d: f32| -> (f32, f32) {
            let rx1 = x3d * cos_y - z3d * sin_y;
            let rz1 = x3d * sin_y + z3d * cos_y;
            let ry2 = y3d * cos_x - rz1 * sin_x;
            let rz2 = y3d * sin_x + rz1 * cos_x;
            let cam_dist = 5.0;
            let proj_z = rz2 + cam_dist;
            let persp = cam_dist / proj_z.max(1.0);
            (center_x + rx1 * scale * persp, center_y + ry2 * scale * persp)
        };
        
        // Pre-compute the 3 visible face quads (top, left side, right side)
        // With 45° Y rotation: visible faces are y=-1 (top), x=-1 (left), z=-1 (right)
        let cv0 = if cube_active { project_cv(-1.0, -1.0, -1.0) } else { (0.0, 0.0) };
        let cv1 = if cube_active { project_cv( 1.0, -1.0, -1.0) } else { (0.0, 0.0) };
        let cv2 = if cube_active { project_cv( 1.0,  1.0, -1.0) } else { (0.0, 0.0) };
        let cv3 = if cube_active { project_cv(-1.0,  1.0, -1.0) } else { (0.0, 0.0) };
        let cv4 = if cube_active { project_cv(-1.0, -1.0,  1.0) } else { (0.0, 0.0) };
        let cv7 = if cube_active { project_cv(-1.0,  1.0,  1.0) } else { (0.0, 0.0) };
        
        // Top face quad (y=-1): cv0 → cv1 → cv4 (via t2) → cv4 (via t3)
        // Diamond: back=cv0, right=cv1, front=project(1,-1,1), left=cv4
        let cv_t2 = if cube_active { project_cv( 1.0, -1.0,  1.0) } else { (0.0, 0.0) };
        let top_quad = [cv0, cv1, cv_t2, cv4];
        // Left face quad (x=-1): cv0 → cv4 → cv7 → cv3
        let left_quad = [cv0, cv4, cv7, cv3];
        // Right face quad (z=-1): cv0 → cv1 → cv2 → cv3
        let right_quad = [cv0, cv1, cv2, cv3];
        
        // Bounding box for quick rejection (expanded upward for padding)
        let cube_min_x = if cube_active { cv0.0.min(cv1.0).min(cv2.0).min(cv3.0).min(cv4.0).min(cv7.0).min(cv_t2.0) - 2.0 } else { 0.0 };
        let cube_max_x = if cube_active { cv0.0.max(cv1.0).max(cv2.0).max(cv3.0).max(cv4.0).max(cv7.0).max(cv_t2.0) + 2.0 } else { 0.0 };
        let cube_min_y = if cube_active { cv0.1.min(cv1.1).min(cv2.1).min(cv3.1).min(cv4.1).min(cv7.1).min(cv_t2.1) - 20.0 } else { 0.0 };
        let cube_max_y = if cube_active { cv0.1.max(cv1.1).max(cv2.1).max(cv3.1).max(cv4.1).max(cv7.1).max(cv_t2.1) + 2.0 } else { 0.0 };
        
        // Pre-compute glow metrics ONCE (not per-cell)
        let diamond_half_w = if cube_active { (top_right_x - top_left_x) / 2.0 } else { 1.0 };
        let diamond_half_h = if cube_active { (top_front_y - top_back_y) / 2.0 } else { 1.0 };
        let diamond_center_x_g = (top_left_x + top_right_x) / 2.0;
        let diamond_center_y_g = (top_back_y + top_front_y) / 2.0;
        let inv_diamond_half_w = 1.0 / diamond_half_w.max(1.0);
        let inv_diamond_half_h = 1.0 / diamond_half_h.max(1.0);
        
        // Point-in-quad winding test (no trig - just cross products)
        // Pre-compute edge vectors for each quad to avoid recomputing
        let quad_edges = |q: &[(f32, f32); 4]| -> [(f32, f32, f32, f32); 4] {
            let mut edges = [(0.0f32, 0.0f32, 0.0f32, 0.0f32); 4];
            for i in 0..4 {
                let j = (i + 1) % 4;
                edges[i] = (q[j].0 - q[i].0, q[j].1 - q[i].1, q[i].0, q[i].1);
            }
            edges
        };
        let top_edges = if cube_active { quad_edges(&top_quad) } else { [(0.0,0.0,0.0,0.0); 4] };
        let left_edges = if cube_active { quad_edges(&left_quad) } else { [(0.0,0.0,0.0,0.0); 4] };
        let right_edges = if cube_active { quad_edges(&right_quad) } else { [(0.0,0.0,0.0,0.0); 4] };
        
        // Padding zone: a slightly expanded top quad to create black separation
        // Expand the top face diamond outward and upward by a few pixels
        let pad = 16.0_f32;
        let top_center_x = (cv0.0 + cv1.0 + cv_t2.0 + cv4.0) / 4.0;
        let top_center_y = (cv0.1 + cv1.1 + cv_t2.1 + cv4.1) / 4.0;
        let expand = |p: (f32, f32)| -> (f32, f32) {
            let dx = p.0 - top_center_x;
            let dy = p.1 - top_center_y;
            (p.0 + dx * 0.12 + 0.0, p.1 + dy * 0.12 - pad * 0.5)
        };
        let pad_quad = if cube_active {
            [expand(cv0), expand(cv1), expand(cv_t2), expand(cv4)]
        } else {
            [(0.0,0.0); 4]
        };
        let pad_edges = if cube_active { quad_edges(&pad_quad) } else { [(0.0,0.0,0.0,0.0); 4] };
        
        #[inline(always)]
        fn point_in_quad_fast(px: f32, py: f32, edges: &[(f32, f32, f32, f32); 4]) -> bool {
            let mut pos = 0u8;
            let mut neg = 0u8;
            for &(ex, ey, ox, oy) in edges.iter() {
                let cross = ex * (py - oy) - ey * (px - ox);
                if cross > 0.0 { pos += 1; } 
                else if cross < 0.0 { neg += 1; }
            }
            pos == 0 || neg == 0
        }
        
        // PARALLEL column rendering via SMP — split columns across cores
        let total_cols = cell_cols.min(self.num_cols);
        let params = BrailleParallelParams {
            buffer_ptr: buffer.as_mut_ptr(),
            buffer_len: buffer.len(),
            fb_width,
            fb_height,
            cell_size,
            cell_rows,
            drops_ptr: self.drops.as_ptr(),
            col_depth_ptr: self.col_depth.as_ptr(),
            num_cols: self.num_cols,
            cube_active,
            cube_min_x,
            cube_max_x,
            cube_min_y,
            cube_max_y,
            top_edges,
            left_edges,
            right_edges,
            pad_edges,
        };
        
        crate::cpu::smp::parallel_for(
            total_cols,
            render_columns_parallel,
            &params as *const BrailleParallelParams as *mut u8,
        );
        
        // Shape overlay is now handled by render_entity_layer()
        // This allows rain to be rendered first, then entity on top
    }
    
    /// Render entity overlay - white pixel layer on top of rain
    /// Uses small 2x2 pixels for clean geometric shapes
    pub fn render_entity_layer(&self, buffer: &mut [u32], fb_width: usize, fb_height: usize) {
        if self.shape_mode == ShapeOverlay::None {
            return;
        }
        
        let center_x = (fb_width / 2) as f32;
        let center_y = (fb_height / 2) as f32;
        let scale = (fb_height as f32).min(fb_width as f32) * 0.18;  // Smaller cube
        
        // Time for animation
        let time = self.shape_time;
        
        match self.shape_mode {
            ShapeOverlay::Cube => {
                // No wireframe — the rain interaction defines the cube shape
            },
            ShapeOverlay::Sphere => {
                // Wireframe sphere - latitude/longitude lines
                let num_segments = 24;
                
                // Draw longitude lines (vertical circles)
                for i in 0..6 {
                    let phi = (i as f32 / 6.0) * 3.14159;
                    let prev_color = 0xFF00AA44;  // Matrix green
                    
                    let mut prev_x = 0i32;
                    let mut prev_y = 0i32;
                    let mut first = true;
                    
                    for j in 0..=num_segments {
                        let theta = (j as f32 / num_segments as f32) * 3.14159 * 2.0;
                        
                        let x = Self::fast_sin(theta) * Self::fast_cos(phi);
                        let z = Self::fast_sin(theta) * Self::fast_sin(phi);
                        let y = Self::fast_cos(theta);
                        
                        // Rotate by time for subtle motion
                        let rot_angle = time * 0.1;
                        let rx = x * Self::fast_cos(rot_angle) - z * Self::fast_sin(rot_angle);
                        let rz = x * Self::fast_sin(rot_angle) + z * Self::fast_cos(rot_angle);
                        
                        let perspective = 3.0 / (4.0 + rz);
                        let px = (center_x + rx * scale * perspective) as i32;
                        let py = (center_y + y * scale * perspective * 0.8) as i32;
                        
                        if !first {
                            let depth = (rz + 1.0) / 2.0;
                            let color = if depth > 0.5 { 0xFFCCFFCC } else { prev_color };  // Matrix green
                            self.draw_line_thick(buffer, fb_width, fb_height, 
                                               prev_x, prev_y, px, py, color, 2);
                        }
                        prev_x = px;
                        prev_y = py;
                        first = false;
                    }
                }
                
                // Draw latitude lines (horizontal circles)
                for i in 1..4 {
                    let y_level = -0.75 + (i as f32 * 0.5);
                    let radius = (1.0 - y_level * y_level).max(0.0);
                    let radius = {
                        let mut x = radius;
                        let mut y = radius * 0.5;
                        for _ in 0..4 { y = (y + x / y) * 0.5; }
                        y
                    };
                    
                    let mut prev_x = 0i32;
                    let mut prev_y = 0i32;
                    let mut first = true;
                    
                    for j in 0..=num_segments {
                        let angle = (j as f32 / num_segments as f32) * 3.14159 * 2.0 + time * 0.1;
                        let x = Self::fast_cos(angle) * radius;
                        let z = Self::fast_sin(angle) * radius;
                        
                        let perspective = 3.0 / (4.0 + z);
                        let px = (center_x + x * scale * perspective) as i32;
                        let py = (center_y + y_level * scale * perspective * 0.8) as i32;
                        
                        if !first {
                            let depth = (z + 1.0) / 2.0;
                            let color = if depth > 0.5 { 0xFFCCFFCC } else { 0xFF00AA44 };  // Matrix green
                            self.draw_line_thick(buffer, fb_width, fb_height, 
                                               prev_x, prev_y, px, py, color, 2);
                        }
                        prev_x = px;
                        prev_y = py;
                        first = false;
                    }
                }
            },
            ShapeOverlay::Torus => {
                // Wireframe torus
                let major_r = 0.7;
                let minor_r = 0.3;
                let segments = 16;
                
                // Draw rings around the torus
                for i in 0..8 {
                    let u = (i as f32 / 8.0) * 3.14159 * 2.0;
                    let cu = Self::fast_cos(u);
                    let su = Self::fast_sin(u);
                    
                    let mut prev_x = 0i32;
                    let mut prev_y = 0i32;
                    let mut first = true;
                    
                    for j in 0..=segments {
                        let v = (j as f32 / segments as f32) * 3.14159 * 2.0;
                        let cv = Self::fast_cos(v);
                        let sv = Self::fast_sin(v);
                        
                        let x = (major_r + minor_r * cv) * cu;
                        let y = minor_r * sv;
                        let z = (major_r + minor_r * cv) * su;
                        
                        // Rotate
                        let rot = time * 0.2;
                        let rx = x * Self::fast_cos(rot) - z * Self::fast_sin(rot);
                        let rz = x * Self::fast_sin(rot) + z * Self::fast_cos(rot);
                        
                        let perspective = 3.0 / (4.0 + rz);
                        let px = (center_x + rx * scale * perspective) as i32;
                        let py = (center_y + y * scale * perspective) as i32;
                        
                        if !first {
                            let depth = (rz + 1.0) / 2.0;
                            let color = if depth > 0.5 { 0xFFCCFFCC } else { 0xFF00AA44 };  // Matrix green
                            self.draw_line_thick(buffer, fb_width, fb_height, 
                                               prev_x, prev_y, px, py, color, 2);
                        }
                        prev_x = px;
                        prev_y = py;
                        first = false;
                    }
                }
            },
            ShapeOverlay::DNA => {
                // Double helix
                let helix_height = scale * 1.5;
                let helix_radius = scale * 0.3;
                let segments = 40;
                
                let mut prev_x1 = 0i32;
                let mut prev_y1 = 0i32;
                let mut prev_x2 = 0i32;
                let mut prev_y2 = 0i32;
                let mut first = true;
                
                for i in 0..=segments {
                    let t = i as f32 / segments as f32;
                    let y = center_y - helix_height / 2.0 + helix_height * t;
                    let angle = t * 3.14159 * 4.0 + time * 0.5;
                    
                    // Two strands offset by pi
                    let x1 = center_x + Self::fast_cos(angle) * helix_radius;
                    let x2 = center_x + Self::fast_cos(angle + 3.14159) * helix_radius;
                    let z1 = Self::fast_sin(angle);
                    let z2 = Self::fast_sin(angle + 3.14159);
                    
                    let px1 = x1 as i32;
                    let py1 = y as i32;
                    let px2 = x2 as i32;
                    let py2 = y as i32;
                    
                    if !first {
                        let color1 = if z1 > 0.0 { 0xFFCCFFCC } else { 0xFF00AA44 };  // Matrix green
                        let color2 = if z2 > 0.0 { 0xFFCCFFCC } else { 0xFF00AA44 };  // Matrix green
                        self.draw_line_thick(buffer, fb_width, fb_height, 
                                           prev_x1, prev_y1, px1, py1, color1, 2);
                        self.draw_line_thick(buffer, fb_width, fb_height, 
                                           prev_x2, prev_y2, px2, py2, color2, 2);
                        
                        // Draw connecting rungs every 4 segments
                        if i % 4 == 0 {
                            self.draw_line_thick(buffer, fb_width, fb_height, 
                                               px1, py1, px2, py2, 0xFF44FF44, 1);
                        }
                    }
                    prev_x1 = px1;
                    prev_y1 = py1;
                    prev_x2 = px2;
                    prev_y2 = py2;
                    first = false;
                }
            },
            ShapeOverlay::None => {},
        }
    }
    
    /// Render cube flow layer — rain deflected by cube surfaces
    /// TOP face: rain splits into 2 diagonal streams following isometric edges
    /// Where 2 diagonals cross → white pixel (brilliance effect)
    /// SIDE faces: rain flows vertically downward
    pub fn render_cube_flow_layer(&self, buffer: &mut [u32], fb_width: usize, fb_height: usize) {
        if self.shape_mode != ShapeOverlay::Cube {
            return;
        }
        
        let cell_size = 8usize;
        let screen_width = (self.num_cols * cell_size) as f32;
        let screen_height = (self.num_rows * cell_size) as f32;
        let center_x = screen_width / 2.0;
        let center_y = screen_height / 2.0;
        let scale = screen_height.min(screen_width) * 0.18;
        
        let angle_y = 0.785398_f32;
        let angle_x = 0.523599_f32;
        let cos_y = Self::fast_cos(angle_y);
        let sin_y = Self::fast_sin(angle_y);
        let cos_x = Self::fast_cos(angle_x);
        let sin_x = Self::fast_sin(angle_x);
        let cam_dist = 5.0_f32;
        
        let project = |x3d: f32, y3d: f32, z3d: f32| -> (f32, f32) {
            let rx1 = x3d * cos_y - z3d * sin_y;
            let rz1 = x3d * sin_y + z3d * cos_y;
            let ry2 = y3d * cos_x - rz1 * sin_x;
            let rz2 = y3d * sin_x + rz1 * cos_x;
            let proj_z = rz2 + cam_dist;
            let perspective = cam_dist / proj_z.max(1.0);
            (center_x + rx1 * scale * perspective, center_y + ry2 * scale * perspective)
        };
        
        // Top face diamond (y=-1 plane)
        let t0 = project(-1.0, -1.0, -1.0); // back vertex
        let t1 = project( 1.0, -1.0, -1.0); // right vertex
        let t2 = project( 1.0, -1.0,  1.0); // front vertex (bottom of diamond)
        let t3 = project(-1.0, -1.0,  1.0); // left vertex
        
        // Bottom vertices for side faces
        let l2 = project(-1.0,  1.0,  1.0);  // left-face bottom-front
        let l3 = project(-1.0,  1.0, -1.0);  // left-face bottom-back
        let r2 = project( 1.0,  1.0, -1.0);  // right-face bottom-right
        
        let top_quad = [t0, t1, t2, t3];
        let left_quad = [t0, t3, l2, l3];
        // Right face = z=-1 plane (visible on right side with 45° Y rotation)
        let right_quad = [t0, t1, r2, l3];
        
        // UV basis vectors for the top face
        // u-axis: t0 → t1 (back to right edge direction)
        // v-axis: t0 → t3 (back to left edge direction)
        let top_u_dx = t1.0 - t0.0;
        let top_u_dy = t1.1 - t0.1;
        let top_v_dx = t3.0 - t0.0;
        let top_v_dy = t3.1 - t0.1;
        let top_det = top_u_dx * top_v_dy - top_u_dy * top_v_dx;
        
        // Bounding box
        let all_pts = [t0, t1, t2, t3, l2, l3, r2];
        let mut bb_min_x = all_pts[0].0;
        let mut bb_max_x = all_pts[0].0;
        let mut bb_min_y = all_pts[0].1;
        let mut bb_max_y = all_pts[0].1;
        for p in &all_pts[1..] {
            if p.0 < bb_min_x { bb_min_x = p.0; }
            if p.0 > bb_max_x { bb_max_x = p.0; }
            if p.1 < bb_min_y { bb_min_y = p.1; }
            if p.1 > bb_max_y { bb_max_y = p.1; }
        }
        
        let cell_x0 = ((bb_min_x / cell_size as f32) as i32).max(0) as usize;
        let cell_x1 = ((bb_max_x / cell_size as f32) as i32 + 1).min(self.num_cols as i32) as usize;
        let cell_y0 = ((bb_min_y / cell_size as f32) as i32).max(0) as usize;
        let cell_y1 = ((bb_max_y / cell_size as f32) as i32 + 1).min(self.num_rows as i32) as usize;
        
        let point_in_quad = |px: f32, py: f32, q: &[(f32, f32); 4]| -> bool {
            let mut pos = 0i32;
            let mut neg = 0i32;
            for i in 0..4 {
                let j = (i + 1) % 4;
                let cross = (q[j].0 - q[i].0) * (py - q[i].1) - (q[j].1 - q[i].1) * (px - q[i].0);
                if cross > 0.0 { pos += 1; } 
                else if cross < 0.0 { neg += 1; }
            }
            pos == 0 || neg == 0
        };
        
        let time = self.shape_time;
        let num_lanes = 8.0_f32;   // number of rain streams per axis
        let lane_width = 0.35_f32; // fraction of lane that is "on"
        
        for cy in cell_y0..cell_y1 {
            for cx in cell_x0..cell_x1 {
                let px = (cx * cell_size + 4) as f32;
                let py = (cy * cell_size + 4) as f32;
                
                // === TOP FACE: diagonal rain following isometric edges ===
                if top_det.abs() > 0.01 && point_in_quad(px, py, &top_quad) {
                    // Compute UV on the top face (0..1 range)
                    let dpx = px - t0.0;
                    let dpy = py - t0.1;
                    let inv_det = 1.0 / top_det;
                    let u = (dpx * top_v_dy - dpy * top_v_dx) * inv_det;
                    let v = (top_u_dx * dpy - top_u_dy * dpx) * inv_det;
                    
                    // Lane coordinates: u selects which u-lane, v selects which v-lane
                    let u_lane = u * num_lanes;
                    let v_lane = v * num_lanes;
                    let u_frac = u_lane - Self::fast_floor(u_lane);
                    let v_frac = v_lane - Self::fast_floor(v_lane);
                    
                    let on_u = u_frac < lane_width; // stream parallel to u-axis
                    let on_v = v_frac < lane_width; // stream parallel to v-axis
                    
                    if on_u || on_v {
                        let u_lane_id = Self::fast_floor(u_lane) as i32;
                        let v_lane_id = Self::fast_floor(v_lane) as i32;
                        
                        // Scrolling animation: streams flow along their axis
                        let scroll_speed = 2.5;
                        let mut brightness: f32 = 0.0;
                        
                        if on_u {
                            // Stream flows along u direction
                            let seed = (v_lane_id as u32).wrapping_mul(2654435761);
                            let phase = (seed % 100) as f32 * 0.04;
                            let head = time * scroll_speed + phase;
                            let trail = 0.4_f32;  // trail in UV units
                            let period = 0.7 + (seed % 3) as f32 * 0.15;
                            let d = u - head;
                            let dm = d - Self::fast_floor(d / period) * period;
                            if dm >= 0.0 && dm < trail {
                                brightness = (1.0 - dm / trail).max(0.0);
                            }
                        }
                        
                        if on_v {
                            let seed = (u_lane_id as u32).wrapping_mul(340573321);
                            let phase = (seed % 100) as f32 * 0.04;
                            let head = time * scroll_speed * 0.9 + phase;
                            let trail = 0.4_f32;
                            let period = 0.7 + (seed % 3) as f32 * 0.15;
                            let d = v - head;
                            let dm = d - Self::fast_floor(d / period) * period;
                            if dm >= 0.0 && dm < trail {
                                let b2 = (1.0 - dm / trail).max(0.0);
                                if b2 > brightness { brightness = b2; }
                            }
                        }
                        
                        if brightness < 0.08 { brightness = 0.08; }
                        
                        // Glyph selection
                        let cell_hash = (cx as u32).wrapping_mul(2654435761)
                            .wrapping_add((cy as u32).wrapping_mul(340573321));
                        let anim_frame = (time * 8.0) as u32;
                        let glyph_idx = ((cell_hash.wrapping_add(anim_frame)) % GLYPH_COUNT as u32) as usize;
                        let glyph = &MATRIX_GLYPHS_6X6[glyph_idx];
                        
                        // Color: INTERSECTION of both lanes → WHITE
                        let color = if on_u && on_v {
                            // Intersection! Bright white
                            let w = (180.0 + brightness * 75.0) as u8;
                            0xFF000000 | ((w as u32) << 16) | ((w as u32) << 8) | (w as u32)
                        } else if brightness > 0.85 {
                            // Bright head of single stream
                            let g = (brightness * 255.0) as u8;
                            0xFF000000 | 0x00300000 | ((g as u32) << 8) | 0x30
                        } else {
                            // Regular green stream
                            let g = (30.0 + brightness * 220.0) as u8;
                            0xFF000000 | ((g as u32) << 8) | 0x06
                        };
                        
                        self.draw_glyph_6x6(buffer, fb_width, cx * cell_size + 1, cy * cell_size + 1, glyph, color);
                    }
                    continue;
                }
                
                // === LEFT FACE: vertical rain columns ===
                if point_in_quad(px, py, &left_quad) {
                    let col_px = 10.0_f32;
                    let col_f = px / col_px;
                    let col_frac = col_f - Self::fast_floor(col_f);
                    
                    if col_frac < 0.4 {
                        let col_id = Self::fast_floor(col_f) as i32;
                        let seed = (col_id as u32).wrapping_mul(2654435761);
                        let scroll_speed = 2.5 + (seed % 6) as f32 * 0.3;
                        let phase = (seed % 100) as f32 * 0.05;
                        let head = time * scroll_speed + phase;
                        let pos = py / col_px;
                        let trail = 3.0;
                        let period = 4.0 + (seed % 3) as f32 * 0.5;
                        let d = pos - head;
                        let dm = d - Self::fast_floor(d / period) * period;
                        
                        let mut brightness: f32 = 0.06;
                        if dm >= 0.0 && dm < trail {
                            brightness = (1.0 - dm / trail).max(0.0);
                            if brightness < 0.06 { brightness = 0.06; }
                        }
                        
                        let cell_hash = (cx as u32).wrapping_mul(2654435761)
                            .wrapping_add((cy as u32).wrapping_mul(340573321));
                        let anim_frame = (time * 6.0) as u32;
                        let glyph_idx = ((cell_hash.wrapping_add(anim_frame)) % GLYPH_COUNT as u32) as usize;
                        let glyph = &MATRIX_GLYPHS_6X6[glyph_idx];
                        
                        let color = if brightness > 0.88 {
                            let g = (brightness * 200.0) as u8;
                            0xFF000000 | 0x00200000 | ((g as u32) << 8) | 0x18
                        } else {
                            let g = (12.0 + brightness * 140.0) as u8;
                            0xFF000000 | ((g as u32) << 8) | 0x04
                        };
                        
                        self.draw_glyph_6x6(buffer, fb_width, cx * cell_size + 1, cy * cell_size + 1, glyph, color);
                    }
                    continue;
                }
                
                // === RIGHT FACE (z=-1): vertical rain columns ===
                if point_in_quad(px, py, &right_quad) {
                    let col_px = 10.0_f32;
                    let col_f = px / col_px;
                    let col_frac = col_f - Self::fast_floor(col_f);
                    
                    if col_frac < 0.4 {
                        let col_id = Self::fast_floor(col_f) as i32;
                        let seed = (col_id as u32).wrapping_mul(340573321);
                        let scroll_speed = 2.8 + (seed % 5) as f32 * 0.25;
                        let phase = (seed % 100) as f32 * 0.05;
                        let head = time * scroll_speed + phase;
                        let pos = py / col_px;
                        let trail = 3.5;
                        let period = 4.5 + (seed % 3) as f32 * 0.5;
                        let d = pos - head;
                        let dm = d - Self::fast_floor(d / period) * period;
                        
                        let mut brightness: f32 = 0.06;
                        if dm >= 0.0 && dm < trail {
                            brightness = (1.0 - dm / trail).max(0.0);
                            if brightness < 0.06 { brightness = 0.06; }
                        }
                        
                        let cell_hash = (cx as u32).wrapping_mul(2654435761)
                            .wrapping_add((cy as u32).wrapping_mul(340573321));
                        let anim_frame = (time * 7.0) as u32;
                        let glyph_idx = ((cell_hash.wrapping_add(anim_frame)) % GLYPH_COUNT as u32) as usize;
                        let glyph = &MATRIX_GLYPHS_6X6[glyph_idx];
                        
                        let color = if brightness > 0.88 {
                            let g = (brightness * 230.0) as u8;
                            0xFF000000 | 0x00280000 | ((g as u32) << 8) | 0x20
                        } else {
                            let g = (15.0 + brightness * 170.0) as u8;
                            0xFF000000 | ((g as u32) << 8) | 0x06
                        };
                        
                        self.draw_glyph_6x6(buffer, fb_width, cx * cell_size + 1, cy * cell_size + 1, glyph, color);
                    }
                }
            }
        }
    }
    
    /// Draw a thick line using Bresenham algorithm with width
    fn draw_line_thick(&self, buffer: &mut [u32], fb_width: usize, fb_height: usize,
                       x0: i32, y0: i32, x1: i32, y1: i32, color: u32, thickness: i32) {
        // Safety: skip if both endpoints are way off-screen
        let margin = 100i32;
        let w = fb_width as i32;
        let h = fb_height as i32;
        if (x0 < -margin && x1 < -margin) || (x0 > w + margin && x1 > w + margin) {
            return;
        }
        if (y0 < -margin && y1 < -margin) || (y0 > h + margin && y1 > h + margin) {
            return;
        }
        
        let dx = (x1 - x0).abs();
        let dy = -(y1 - y0).abs();
        let sx = if x0 < x1 { 1 } else { -1 };
        let sy = if y0 < y1 { 1 } else { -1 };
        let mut err = dx + dy;
        
        let mut x = x0;
        let mut y = y0;
        
        // Safety: limit iterations to prevent infinite loops
        let max_steps = (dx.abs() + (-dy).abs() + 10) as usize;
        let mut steps = 0usize;
        
        loop {
            steps += 1;
            if steps > max_steps { break; }
            
            // Draw thick pixel (cross pattern for thickness)
            for ty in -thickness/2..=thickness/2 {
                for tx in -thickness/2..=thickness/2 {
                    let px = x + tx;
                    let py = y + ty;
                    if px >= 0 && py >= 0 {
                        let pxu = px as usize;
                        let pyu = py as usize;
                        if pxu < fb_width && pyu < fb_height {
                            buffer[pyu * fb_width + pxu] = color;
                        }
                    }
                }
            }
            
            if x == x1 && y == y1 { break; }
            
            let e2 = 2 * err;
            if e2 >= dy {
                err += dy;
                x += sx;
            }
            if e2 <= dx {
                err += dx;
                y += sy;
            }
        }
    }
    
    /// Draw a line with Matrix flow effect - pulses travel along the edge
    fn draw_line_matrix_flow(&self, buffer: &mut [u32], fb_width: usize, fb_height: usize,
                              x0: i32, y0: i32, x1: i32, y1: i32, 
                              base_color: u32, depth: f32, edge_idx: usize) {
        // Safety: skip if both endpoints are way off-screen
        let margin = 100i32;
        let w = fb_width as i32;
        let h = fb_height as i32;
        if (x0 < -margin && x1 < -margin) || (x0 > w + margin && x1 > w + margin) {
            return;
        }
        if (y0 < -margin && y1 < -margin) || (y0 > h + margin && y1 > h + margin) {
            return;
        }
        
        let time = self.shape_time;
        
        // Calculate line length for flow effect
        let dx_f = (x1 - x0) as f32;
        let dy_f = (y1 - y0) as f32;
        let line_len = (dx_f * dx_f + dy_f * dy_f).max(1.0);
        let line_len = {
            let mut x = line_len;
            let mut y = line_len * 0.5;
            for _ in 0..4 { y = (y + x / y) * 0.5; }
            y
        };
        
        let dx = (x1 - x0).abs();
        let dy = -(y1 - y0).abs();
        let sx = if x0 < x1 { 1 } else { -1 };
        let sy = if y0 < y1 { 1 } else { -1 };
        let mut err = dx + dy;
        
        let mut x = x0;
        let mut y = y0;
        let mut pixel_idx = 0usize;
        
        // Safety: limit iterations
        let max_steps = (dx.abs() + (-dy).abs() + 10) as usize;
        let mut steps = 0usize;
        
        // Number of "data pulses" traveling along this edge
        let num_pulses = 3;
        // Speed of pulses (cycles per second)
        let pulse_speed = 2.0 + (edge_idx as f32 * 0.3);
        // Width of each pulse (in pixels)
        let pulse_width = 12.0;
        
        loop {
            steps += 1;
            if steps > max_steps { break; }
            
            // Calculate position along line (0.0 to 1.0)
            let t = pixel_idx as f32 / line_len.max(1.0);
            
            // Calculate pulse intensity at this position
            let mut pulse_intensity = 0.0f32;
            for p in 0..num_pulses {
                // Each pulse offset in time and position
                let pulse_phase = (time * pulse_speed + p as f32 / num_pulses as f32) % 1.0;
                let pulse_center = pulse_phase;
                
                // Distance from pulse center (wrapping)
                let dist1 = (t - pulse_center).abs();
                let dist2 = (t - pulse_center - 1.0).abs();
                let dist3 = (t - pulse_center + 1.0).abs();
                let dist = dist1.min(dist2).min(dist3);
                
                // Gaussian-like falloff for pulse
                let pulse_t = dist * line_len / pulse_width;
                if pulse_t < 1.0 {
                    pulse_intensity += (1.0 - pulse_t * pulse_t).max(0.0);
                }
            }
            pulse_intensity = pulse_intensity.min(1.0);
            
            // Blend between base color and bright white based on pulse
            let (base_r, base_g, base_b) = (
                ((base_color >> 16) & 0xFF) as f32,
                ((base_color >> 8) & 0xFF) as f32,
                (base_color & 0xFF) as f32,
            );
            
            // Pulse color: bright white-green
            let pulse_r = 220.0;
            let pulse_g = 255.0;
            let pulse_b = 220.0;
            
            let r = (base_r + (pulse_r - base_r) * pulse_intensity) as u32;
            let g = (base_g + (pulse_g - base_g) * pulse_intensity) as u32;
            let b = (base_b + (pulse_b - base_b) * pulse_intensity) as u32;
            let color = 0xFF000000 | (r << 16) | (g << 8) | b;
            
            // Draw pixel with thickness based on pulse
            let thickness = if pulse_intensity > 0.3 { 2 } else { 1 };
            for ty in -thickness/2..=thickness/2 {
                for tx in -thickness/2..=thickness/2 {
                    let px = x + tx;
                    let py_coord = y + ty;
                    if px >= 0 && py_coord >= 0 {
                        let pxu = px as usize;
                        let pyu = py_coord as usize;
                        if pxu < fb_width && pyu < fb_height {
                            buffer[pyu * fb_width + pxu] = color;
                        }
                    }
                }
            }
            
            pixel_idx += 1;
            
            if x == x1 && y == y1 { break; }
            
            let e2 = 2 * err;
            if e2 >= dy {
                err += dy;
                x += sx;
            }
            if e2 <= dx {
                err += dx;
                y += sy;
            }
        }
    }
    
    /// Draw a 6x6 glyph at pixel position - OPTIMIZED: minimal bounds checks
    #[inline(always)]
    fn draw_glyph_6x6(&self, buffer: &mut [u32], fb_width: usize, 
                       px: usize, py: usize, glyph: &[u8; 6], color: u32) {
        // Pre-compute fb_height once (avoid division in loop)
        let fb_height = buffer.len() / fb_width;
        
        // Early exit if completely off-screen
        if py >= fb_height || px >= fb_width {
            return;
        }
        
        // Determine safe row range
        let max_row = (fb_height - py).min(6);
        let max_col = (fb_width - px).min(6);
        
        // Unrolled inner loop for common case
        for row in 0..max_row {
            let row_bits = glyph[row];
            if row_bits == 0 { continue; } // Skip empty rows
            
            let row_start = (py + row) * fb_width + px;
            
            // Fast unrolled pixel writes (no bounds check needed now)
            if row_bits & 0b000001 != 0 && 0 < max_col { buffer[row_start] = color; }
            if row_bits & 0b000010 != 0 && 1 < max_col { buffer[row_start + 1] = color; }
            if row_bits & 0b000100 != 0 && 2 < max_col { buffer[row_start + 2] = color; }
            if row_bits & 0b001000 != 0 && 3 < max_col { buffer[row_start + 3] = color; }
            if row_bits & 0b010000 != 0 && 4 < max_col { buffer[row_start + 4] = color; }
            if row_bits & 0b100000 != 0 && 5 < max_col { buffer[row_start + 5] = color; }
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// FAST MATRIX RENDERER (Classic ASCII style)
// ═══════════════════════════════════════════════════════════════════════════════

/// Fast Matrix Rain using font glyphs
pub struct FastMatrixRenderer {
    /// Head position for each column
    heads: Vec<i32>,
    /// Speed per column
    speeds: Vec<u8>,
    /// Character for each cell (pre-generated)
    chars: Vec<u8>,
    /// Frame counter
    frame: u32,
    /// Dimensions
    cols: usize,
    rows: usize,
}

impl FastMatrixRenderer {
    pub fn new() -> Self {
        // Default for 1280x800 screen
        let cols = 1280 / CELL_PIXEL_W; // 160 columns
        let rows = 800 / CELL_PIXEL_H;  // 50 rows
        
        let mut heads = vec![0i32; cols];
        let mut speeds = vec![1u8; cols];
        let mut chars = vec![0u8; cols * rows];
        
        // Initialize
        for i in 0..cols {
            let seed = (i as u32).wrapping_mul(2654435761) ^ 0xDEADBEEF;
            heads[i] = -((seed % (rows as u32 * 2)) as i32);
            speeds[i] = 1 + (seed % 2) as u8;
            
            // Fill column with random characters
            for j in 0..rows {
                let char_seed = seed.wrapping_mul((j + 1) as u32);
                chars[i * rows + j] = MATRIX_CHARSET[(char_seed as usize) % MATRIX_CHARSET.len()];
            }
        }
        
        Self { heads, speeds, chars, frame: 0, cols, rows }
    }
    
    /// Update rain positions
    pub fn update(&mut self) {
        self.frame = self.frame.wrapping_add(1);
        
        for i in 0..self.cols {
            self.heads[i] += self.speeds[i] as i32;
            
            // Reset when off screen
            if self.heads[i] > (self.rows as i32) + 20 {
                let seed = (i as u32).wrapping_mul(self.frame).wrapping_add(0xBEEF);
                self.heads[i] = -((seed % 30) as i32);
            }
        }
    }
    
    /// Render to framebuffer
    pub fn render(&self, buffer: &mut [u32], fb_width: usize, fb_height: usize) {
        let cols = (fb_width / CELL_PIXEL_W).min(self.cols);
        let rows = (fb_height / CELL_PIXEL_H).min(self.rows);
        
        for col in 0..cols {
            let head_y = self.heads[col];
            
            for row in 0..rows {
                let dist = head_y - (row as i32);
                
                if dist >= 0 && dist < 16 {
                    // Calculate intensity
                    let intensity = INTENSITY_LUT[(dist as usize).min(31)];
                    if intensity > 10 {
                        // Get character for this cell
                        let c = self.chars[col * self.rows + row];
                        
                        // Color: bright green fading
                        let color = (0xFF << 24) | ((intensity as u32) << 8);
                        
                        // Draw character
                        self.draw_char(buffer, fb_width, col, row, c, color);
                    }
                }
            }
        }
    }
    
    /// Draw character using font
    fn draw_char(&self, buffer: &mut [u32], fb_width: usize, 
                 col: usize, row: usize, c: u8, color: u32) {
        let px = col * CELL_PIXEL_W;
        let py = row * CELL_PIXEL_H;
        
        let glyph = crate::framebuffer::font::get_glyph(c as char);
        
        for (gy, &bits) in glyph.iter().enumerate() {
            let y = py + gy;
            if y >= buffer.len() / fb_width { continue; }
            
            for gx in 0..8 {
                if (bits >> (7 - gx)) & 1 != 0 {
                    let x = px + gx;
                    let idx = y * fb_width + x;
                    if idx < buffer.len() {
                        buffer[idx] = color;
                    }
                }
            }
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// MATRIX 3D - Volumetric rain with shapes
// ═══════════════════════════════════════════════════════════════════════════════

/// Maximum number of 3D drops (reduced to avoid stack overflow)
const MAX_DROPS_3D: usize = 200;

/// A 3D rain drop with physics
#[derive(Clone, Copy)]
struct Drop3D {
    /// World X position (0.0 to 160.0 in cell units)
    x: f32,
    /// World Y position (falling from negative to positive)
    y: f32,
    /// World Z position (depth: 0.0=far, 1.0=close)
    z: f32,
    /// Velocity X (for surface flow)
    vx: f32,
    /// Velocity Y (fall + flow) 
    vy: f32,
    /// Velocity Z (depth movement)
    vz: f32,
    /// Trail length
    trail_len: u8,
    /// Glyph randomizer
    glyph_seed: u32,
    /// Is drop on a surface?
    on_surface: bool,
    /// Surface flow timer (resets drop after flowing)
    flow_time: u8,
}

impl Drop3D {
    fn new() -> Self {
        Self {
            x: 0.0, y: -10.0, z: 0.5,
            vx: 0.0, vy: 0.5, vz: 0.0,
            trail_len: 20,
            glyph_seed: 0,
            on_surface: false,
            flow_time: 0,
        }
    }
}

/// 3D shape types for collision
#[derive(Clone, Copy)]
pub enum Shape3D {
    /// Sphere: center (x, y, z) and radius
    Sphere { cx: f32, cy: f32, cz: f32, r: f32 },
    /// Cube: center (x, y, z), half-size, and rotation angle
    Cube { cx: f32, cy: f32, cz: f32, half: f32, rot: f32 },
    /// Torus: center (x, y, z), major radius, minor radius
    Torus { cx: f32, cy: f32, cz: f32, R: f32, r: f32 },
}

/// Matrix 3D Rain Renderer with volumetric effects
pub struct Matrix3D {
    /// All 3D drops
    drops: [Drop3D; MAX_DROPS_3D],
    /// Active shapes
    shapes: [Option<Shape3D>; 4],
    /// RNG seed
    rng: u32,
    /// Frame counter
    frame: u32,
    /// Animation time (for rotating shapes)
    time: f32,
    /// Screen dimensions
    width: usize,
    height: usize,
}

impl Matrix3D {
    pub fn new() -> Self {
        let mut drops = [Drop3D::new(); MAX_DROPS_3D];
        let mut rng = 0xDEADBEEFu32;
        
        // Initialize drops with random positions
        for drop in drops.iter_mut() {
            rng = rng.wrapping_mul(1103515245).wrapping_add(12345);
            drop.x = (rng % 160) as f32;
            rng = rng.wrapping_mul(1103515245).wrapping_add(12345);
            drop.y = -((rng % 120) as f32);
            rng = rng.wrapping_mul(1103515245).wrapping_add(12345);
            drop.z = 0.2 + (rng % 80) as f32 / 100.0; // 0.2 to 1.0
            
            // Speed based on depth (closer = faster)
            drop.vy = 0.3 + drop.z * 0.7; // 0.3 to 1.0
            
            rng = rng.wrapping_mul(1103515245).wrapping_add(12345);
            drop.trail_len = 10 + (rng % 30) as u8;
            drop.glyph_seed = rng;
        }
        
        // Default: one rotating sphere in center
        let shapes = [
            Some(Shape3D::Sphere { cx: 80.0, cy: 50.0, cz: 0.5, r: 15.0 }),
            None,
            None,
            None,
        ];
        
        Self {
            drops,
            shapes,
            rng,
            frame: 0,
            time: 0.0,
            width: 1280,
            height: 800,
        }
    }
    
    /// Add a shape to the scene
    pub fn add_shape(&mut self, shape: Shape3D) {
        for slot in self.shapes.iter_mut() {
            if slot.is_none() {
                *slot = Some(shape);
                return;
            }
        }
    }
    
    /// Clear all shapes
    pub fn clear_shapes(&mut self) {
        for slot in self.shapes.iter_mut() {
            *slot = None;
        }
    }
    
    /// Fast approximate sin (Taylor series, no libm)
    #[inline(always)]
    fn fast_sin(x: f32) -> f32 {
        // Normalize to -π to π
        let x = x % 6.28318;
        let x = if x > 3.14159 { x - 6.28318 } else if x < -3.14159 { x + 6.28318 } else { x };
        // Taylor series approximation
        let x2 = x * x;
        x * (1.0 - x2 / 6.0 + x2 * x2 / 120.0)
    }
    
    /// Fast approximate cos
    #[inline(always)]
    fn fast_cos(x: f32) -> f32 {
        Self::fast_sin(x + 1.5708)
    }
    
    /// Check if point is inside a shape, return (is_inside, surface_normal)
    fn check_collision_static(shapes: &[Option<Shape3D>; 4], x: f32, y: f32, z: f32) -> Option<(f32, f32, f32)> {
        for shape in shapes.iter().filter_map(|s| *s) {
            match shape {
                Shape3D::Sphere { cx, cy, cz, r } => {
                    let dx = x - cx;
                    let dy = y - cy;
                    let dz = z - cz;
                    let dist_sq = dx * dx + dy * dy + dz * dz;
                    let r_sq = r * r;
                    
                    if dist_sq < r_sq {
                        // Inside sphere - calculate outward normal
                        let dist = Self::fast_sqrt(dist_sq).max(0.01);
                        return Some((dx / dist, dy / dist, dz / dist));
                    }
                }
                Shape3D::Cube { cx, cy, cz, half, rot } => {
                    // Rotate point around cube center
                    let dx = x - cx;
                    let dy = y - cy;
                    let dz = z - cz;
                    
                    let cos_r = Self::fast_cos(rot);
                    let sin_r = Self::fast_sin(rot);
                    
                    // Rotate around Y axis
                    let rx = dx * cos_r - dz * sin_r;
                    let ry = dy;
                    let rz = dx * sin_r + dz * cos_r;
                    
                    // Check if inside axis-aligned box
                    if rx.abs() < half && ry.abs() < half && rz.abs() < half {
                        // Find closest face for normal
                        let ax = half - rx.abs();
                        let ay = half - ry.abs();
                        let az = half - rz.abs();
                        
                        // Rotate normal back
                        if ax < ay && ax < az {
                            let nx = if rx > 0.0 { 1.0 } else { -1.0 };
                            return Some((nx * cos_r, 0.0, nx * sin_r));
                        } else if ay < az {
                            return Some((0.0, if ry > 0.0 { 1.0 } else { -1.0 }, 0.0));
                        } else {
                            let nz = if rz > 0.0 { 1.0 } else { -1.0 };
                            return Some((-nz * sin_r, 0.0, nz * cos_r));
                        }
                    }
                }
                Shape3D::Torus { cx, cy, cz, R, r } => {
                    let dx = x - cx;
                    let dy = y - cy;
                    let dz = z - cz;
                    
                    // Distance from center to point projected onto XZ plane
                    let dist_xz = Self::fast_sqrt(dx * dx + dz * dz);
                    // Distance from torus tube center
                    let tube_dx = dist_xz - R;
                    let tube_dist = Self::fast_sqrt(tube_dx * tube_dx + dy * dy);
                    
                    if tube_dist < r {
                        // Inside torus
                        let dir_x = if dist_xz > 0.01 { dx / dist_xz } else { 1.0 };
                        let dir_z = if dist_xz > 0.01 { dz / dist_xz } else { 0.0 };
                        
                        let nx = tube_dx * dir_x / tube_dist.max(0.01);
                        let ny = dy / tube_dist.max(0.01);
                        let nz = tube_dx * dir_z / tube_dist.max(0.01);
                        
                        return Some((nx, ny, nz));
                    }
                }
            }
        }
        None
    }
    
    /// Fast square root approximation
    #[inline(always)]
    fn fast_sqrt(x: f32) -> f32 {
        if x <= 0.0 { return 0.0; }
        // Newton-Raphson iteration
        let mut guess = x * 0.5;
        guess = 0.5 * (guess + x / guess);
        guess = 0.5 * (guess + x / guess);
        guess
    }
    
    /// Update all drops
    pub fn update(&mut self) {
        self.frame = self.frame.wrapping_add(1);
        self.time += 0.02; // Rotation speed
        
        // Update rotating cube rotation with time
        if let Some(Shape3D::Cube { ref mut rot, .. }) = self.shapes.get_mut(0).and_then(|s| s.as_mut()) {
            *rot = self.time;
        }
        
        let gravity = 0.02f32;
        
        // Copy shapes for collision (avoid borrow conflict)
        let shapes = self.shapes;
        
        for i in 0..MAX_DROPS_3D {
            let drop = &mut self.drops[i];
            
            // Check collision BEFORE moving
            if let Some((nx, ny, nz)) = Self::check_collision_static(&shapes, drop.x, drop.y, drop.z) {
                if !drop.on_surface {
                    // Just hit surface - transfer to surface flow
                    drop.on_surface = true;
                    drop.flow_time = 0;
                    
                    // Project velocity onto surface tangent
                    // Remove normal component from velocity
                    let dot = drop.vx * nx + drop.vy * ny + drop.vz * nz;
                    drop.vx -= dot * nx;
                    drop.vy -= dot * ny;
                    drop.vz -= dot * nz;
                    
                    // Add gravity tangent component
                    let grav_dot = ny; // gravity is (0, 1, 0)
                    drop.vy += gravity * (1.0 - grav_dot * grav_dot).max(0.0);
                }
                
                // Push out of shape
                drop.x += nx * 0.2;
                drop.y += ny * 0.2;
                drop.z += nz * 0.2;
                
                // Slow down on surface
                drop.vx *= 0.95;
                drop.vy *= 0.95;
                drop.vz *= 0.95;
                
                // Add slight tangent flow (water running down)
                drop.vy += gravity * 0.5;
                
                drop.flow_time += 1;
            } else {
                // Free fall
                drop.on_surface = false;
                drop.vy += gravity;
            }
            
            // Apply velocity
            drop.x += drop.vx;
            drop.y += drop.vy;
            drop.z += drop.vz;
            
            // Clamp Z to valid range
            drop.z = drop.z.clamp(0.1, 1.0);
            
            // Reset if off screen or flowed too long
            let reset = drop.y > 110.0 || drop.x < -5.0 || drop.x > 165.0 || drop.flow_time > 100;
            
            if reset {
                self.rng = self.rng.wrapping_mul(1103515245).wrapping_add(12345);
                drop.x = (self.rng % 160) as f32;
                self.rng = self.rng.wrapping_mul(1103515245).wrapping_add(12345);
                drop.y = -((self.rng % 40) as f32);
                self.rng = self.rng.wrapping_mul(1103515245).wrapping_add(12345);
                drop.z = 0.2 + (self.rng % 80) as f32 / 100.0;
                
                drop.vx = 0.0;
                drop.vy = 0.3 + drop.z * 0.7;
                drop.vz = 0.0;
                drop.on_surface = false;
                drop.flow_time = 0;
                drop.glyph_seed = self.rng;
            }
            
            // Update glyph animation
            drop.glyph_seed = drop.glyph_seed.wrapping_mul(1103515245).wrapping_add(12345);
        }
    }
    
    /// Project 3D point to 2D screen with perspective
    #[inline(always)]
    fn project(&self, x: f32, y: f32, z: f32) -> (i32, i32, f32) {
        // Simple perspective: closer (z=1) at natural position, far (z=0) shifted up
        // This creates depth where far drops appear higher (more distant)
        let scale = 0.7 + z * 0.3; // 0.7 to 1.0 scale
        let center_x = 80.0;
        let center_y = 50.0;
        
        // Perspective offset (far drops drift toward center/top)
        let screen_x = center_x + (x - center_x) * scale;
        let screen_y = center_y + (y - center_y) * scale;
        
        (screen_x as i32, screen_y as i32, z)
    }
    
    /// Render to framebuffer
    pub fn render(&self, buffer: &mut [u32], fb_width: usize, fb_height: usize) {
        // Fill background
        let bg_color = 0xFF010201u32;
        buffer.fill(bg_color);
        
        let cell_size = 8usize;
        
        // Sort drops by Z for proper occlusion (far first)
        // For performance, we skip sorting and just render with alpha blend effect
        
        for drop in self.drops.iter() {
            if drop.y < -5.0 { continue; }
            
            let (screen_x, screen_y, depth) = self.project(drop.x, drop.y, drop.z);
            
            // Brightness based on depth
            let depth_brightness = (50.0 + depth * 205.0) as u32;
            
            // Surface drops glow brighter (flowing water effect)
            let surface_boost = if drop.on_surface { 30u32 } else { 0 };
            
            // Draw trail
            let trail_len = drop.trail_len as usize;
            for trail_pos in 0..trail_len {
                // Trail position in world space
                let ty = drop.y - trail_pos as f32 * 0.8;
                if ty < 0.0 { continue; }
                
                let (tx, ty_screen, _) = self.project(drop.x, ty, drop.z);
                
                if tx < 0 || tx >= (fb_width / cell_size) as i32 { continue; }
                if ty_screen < 0 || ty_screen >= (fb_height / cell_size) as i32 { continue; }
                
                // Intensity fades along trail
                let intensity_idx = (trail_pos * 63) / trail_len.max(1);
                let base_intensity = INTENSITY_LUT[intensity_idx.min(63)] as u32;
                let intensity = (((base_intensity * depth_brightness) / 255) + surface_boost).min(255) as u8;
                
                if intensity < 2 { continue; }
                
                // Get glyph
                let glyph_seed = drop.glyph_seed.wrapping_add(trail_pos as u32 * 2654435761);
                let glyph_idx = (glyph_seed % GLYPH_COUNT as u32) as usize;
                let glyph = &MATRIX_GLYPHS_6X6[glyph_idx];
                let color = intensity_to_color(intensity);
                
                let px = tx as usize * cell_size + 1;
                let py = ty_screen as usize * cell_size + 1;
                self.draw_glyph_6x6(buffer, fb_width, px, py, glyph, color);
            }
        }
        
        // Optional: Draw shape outlines for debugging
        // self.draw_shape_outline(buffer, fb_width, fb_height);
    }
    
    /// Draw a 6x6 glyph at pixel position
    #[inline(always)]
    fn draw_glyph_6x6(&self, buffer: &mut [u32], fb_width: usize, 
                       px: usize, py: usize, glyph: &[u8; 6], color: u32) {
        let fb_height = buffer.len() / fb_width;
        
        if py >= fb_height || px >= fb_width {
            return;
        }
        
        let max_row = (fb_height - py).min(6);
        let max_col = (fb_width - px).min(6);
        
        for row in 0..max_row {
            let row_bits = glyph[row];
            if row_bits == 0 { continue; }
            
            let row_start = (py + row) * fb_width + px;
            
            if row_bits & 0b000001 != 0 && 0 < max_col { buffer[row_start] = color; }
            if row_bits & 0b000010 != 0 && 1 < max_col { buffer[row_start + 1] = color; }
            if row_bits & 0b000100 != 0 && 2 < max_col { buffer[row_start + 2] = color; }
            if row_bits & 0b001000 != 0 && 3 < max_col { buffer[row_start + 3] = color; }
            if row_bits & 0b010000 != 0 && 4 < max_col { buffer[row_start + 4] = color; }
            if row_bits & 0b100000 != 0 && 5 < max_col { buffer[row_start + 5] = color; }
        }
    }
    
    /// Set shapes for demo
    pub fn set_demo_shapes(&mut self) {
        self.shapes = [
            // Rotating sphere in center
            Some(Shape3D::Sphere { cx: 80.0, cy: 50.0, cz: 0.5, r: 18.0 }),
            None,
            None,
            None,
        ];
    }
    
    /// Set rotating cube
    pub fn set_cube(&mut self) {
        self.shapes = [
            Some(Shape3D::Cube { cx: 80.0, cy: 50.0, cz: 0.5, half: 12.0, rot: self.time }),
            None,
            None,
            None,
        ];
    }
    
    /// Set torus (donut)
    pub fn set_torus(&mut self) {
        self.shapes = [
            Some(Shape3D::Torus { cx: 80.0, cy: 50.0, cz: 0.5, R: 15.0, r: 5.0 }),
            None,
            None,
            None,
        ];
    }
}
