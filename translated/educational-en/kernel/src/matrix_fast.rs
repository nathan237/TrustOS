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
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const BRAILLE_DOTS_W: usize = 2;   // 2 dots wide per character
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const BRAILLE_DOTS_H: usize = 4;   // 4 dots tall per character
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const CELL_PIXEL_W: usize = 16;    // 16 pixels wide per cell (2 × 8px spacing)
pub // Compile-time constant — evaluated at compilation, zero runtime cost.
const CELL_PIXEL_H: usize = 32;    // 32 pixels tall per cell (4 × 8px spacing)

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
pub(crate) // Compile-time constant — evaluated at compilation, zero runtime cost.
const MATRIX_GLYPHS_6X6: [[u8; 6]; 64] = [
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
pub(crate) // Compile-time constant — evaluated at compilation, zero runtime cost.
const MATRIX_CHARSET: &[u8] = b"0123456789ABCDEF@#$%&*<>[]{}|/\\";

// ═══════════════════════════════════════════════════════════════════════════════
// GLYPH MATRIX RAIN - 6x6 Matrix characters with multiple drops per column
// ═══════════════════════════════════════════════════════════════════════════════

/// High-resolution cell size — 4px cells = 4× density vs 8px, same perf
/// Each cell is CELL_SIZE×CELL_SIZE pixels with a 3×3 glyph inside
const CELL_SIZE: usize = 4;
/// Glyph pixel dimensions inside each cell
const GLYPH_SIZE: usize = 3;
/// Center offset for cube hit-testing
const CELL_CENTER: usize = CELL_SIZE / 2;

/// Trail length bounds - LONGER for denser rain
const MINIMUM_TRAIL_LENGTH: usize = 15;
// Compile-time constant — evaluated at compilation, zero runtime cost.
const MAXIMUM_TRAIL_LENGTH: usize = 50;

/// Maximum drops per column - MORE for denser rain
const DROPS_PER_COLUMN: usize = 6;

/// Total columns — supports up to 2048px width at 4px cells
const MAXIMUM_COLUMNS: usize = 512;

/// Ultra-compact 3×3 pixel micro-glyphs — 64 patterns for high-res matrix rain
/// Each glyph is [row0, row1, row2], each row uses lower 3 bits (bit0=left)
/// At 3×3 pixels the brain reads patterns, not characters — cinematic density
const MATRIX_GLYPHS_3X3: [[u8; 3]; 64] = [
    [0b111, 0b101, 0b111], // 0  - Zero (box)
    [0b010, 0b010, 0b010], // 1  - One (vertical)
    [0b111, 0b010, 0b111], // 2  - Two (H)
    [0b110, 0b011, 0b110], // 3  - Three (zigzag)
    [0b101, 0b111, 0b001], // 4  - Four
    [0b011, 0b110, 0b011], // 5  - Five (reverse zigzag)
    [0b011, 0b111, 0b111], // 6  - Six
    [0b111, 0b001, 0b010], // 7  - Seven (angle)
    [0b111, 0b111, 0b111], // 8  - Eight (full block)
    [0b111, 0b111, 0b001], // 9  - Nine
    [0b111, 0b010, 0b100], // 10 - ア
    [0b001, 0b111, 0b010], // 11 - イ
    [0b111, 0b101, 0b010], // 12 - ウ
    [0b111, 0b100, 0b111], // 13 - エ
    [0b010, 0b111, 0b100], // 14 - オ
    [0b100, 0b111, 0b010], // 15 - カ
    [0b010, 0b111, 0b001], // 16 - キ (L mirror)
    [0b110, 0b001, 0b100], // 17 - ク
    [0b100, 0b110, 0b001], // 18 - ケ
    [0b111, 0b001, 0b111], // 19 - コ (U rotated)
    [0b101, 0b111, 0b100], // 20 - サ
    [0b100, 0b010, 0b001], // 21 - シ (diagonal)
    [0b111, 0b010, 0b001], // 22 - ス
    [0b100, 0b111, 0b001], // 23 - セ
    [0b101, 0b001, 0b010], // 24 - ソ
    [0b011, 0b111, 0b100], // 25 - タ
    [0b111, 0b011, 0b010], // 26 - チ
    [0b101, 0b010, 0b010], // 27 - ツ
    [0b111, 0b010, 0b010], // 28 - テ (T)
    [0b100, 0b110, 0b100], // 29 - ト
    [0b010, 0b111, 0b110], // 30 - ナ
    [0b111, 0b000, 0b111], // 31 - ニ (double line)
    [0b010, 0b101, 0b010], // 32 - Diamond
    [0b010, 0b101, 0b111], // 33 - Triangle up
    [0b111, 0b101, 0b010], // 34 - Triangle down
    [0b010, 0b111, 0b010], // 35 - Cross (+)
    [0b101, 0b010, 0b101], // 36 - X mark
    [0b111, 0b000, 0b111], // 37 - Horizontal lines
    [0b101, 0b101, 0b101], // 38 - Vertical lines
    [0b110, 0b110, 0b000], // 39 - Corner TL
    [0b011, 0b011, 0b000], // 40 - Corner TR
    [0b000, 0b110, 0b110], // 41 - Corner BL
    [0b000, 0b011, 0b011], // 42 - Corner BR
    [0b010, 0b000, 0b010], // 43 - Colon
    [0b000, 0b010, 0b000], // 44 - Dot
    [0b010, 0b101, 0b000], // 45 - Caret
    [0b000, 0b101, 0b010], // 46 - V shape
    [0b100, 0b010, 0b100], // 47 - Wave
    [0b001, 0b010, 0b001], // 48 - Wave reverse
    [0b110, 0b010, 0b011], // 49 - S curve
    [0b011, 0b010, 0b110], // 50 - S reverse
    [0b010, 0b100, 0b010], // 51 - Shift left
    [0b010, 0b001, 0b010], // 52 - Shift right
    [0b101, 0b000, 0b101], // 53 - Sparse dots
    [0b001, 0b011, 0b111], // 54 - Staircase
    [0b100, 0b110, 0b111], // 55 - Staircase mirror
    [0b111, 0b011, 0b001], // 56 - Stair down
    [0b111, 0b110, 0b100], // 57 - Stair down mirror
    [0b011, 0b100, 0b011], // 58 - Bracket
    [0b110, 0b001, 0b110], // 59 - Bracket reverse
    [0b101, 0b010, 0b010], // 60 - Y shape
    [0b010, 0b010, 0b101], // 61 - Y inverted
    [0b110, 0b011, 0b001], // 62 - Slash
    [0b011, 0b110, 0b100], // 63 - Backslash
];

/// Number of 3×3 micro-glyphs
const GLYPH_3X3_COUNT: usize = 64;

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
    trail_length: u8,
    /// Starting glyph index (randomized per drop)
    glyph_seed: u32,
    /// Is this drop active?
    active: bool,
}

// Implementation block — defines methods for the type above.
impl RainDrop {
    fn new_inactive() -> Self {
        Self {
            y: -100,
            speed: 1,
            delay: 0,
            trail_length: MINIMUM_TRAIL_LENGTH as u8,
            glyph_seed: 0,
            active: false,
        }
    }
    
    /// Get the bottom Y position of this drop's trail
    fn tail_y(&self) -> i32 {
        self.y - self.trail_length as i32
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// SHAPE OVERLAY SYSTEM - 3D forms traced by rain drops
// ═══════════════════════════════════════════════════════════════════════════════

/// Shape overlay modes - forms traced by rain
#[derive(Clone, Copy, PartialEq)]
// Enumeration — a type that can be one of several variants.
pub enum ShapeOverlay {
    None,
    Cube,
    Sphere,
    Torus,
    DNA,
}

/// Maximum number of shape drops (points that trace the form)
const MAXIMUM_SHAPE_DROPS: usize = 144;

/// Maximum cube flow drops (rain that flows on cube edges)
const MAXIMUM_CUBE_FLOW_DROPS: usize = 500;  // Dense grid/hatching pattern on cube top face

/// A drop that traces a 3D shape edge
#[derive(Clone, Copy)]
struct ShapeDrop {
    /// Screen X position (pixel)
    column: i32,
    /// Screen Y position (pixel)
    row: i32,
    /// Previous X position (for trail direction)
    previous_column: i32,
    /// Previous Y position (for trail direction)
    previous_row: i32,
    /// Depth (Z) for brightness - 0.0=far/dim, 1.0=near/bright
    depth: f32,
    /// Progress along the edge (0.0 to 1.0)
    progress: f32,
    /// Edge index this drop follows
    edge_index: u8,
    /// Trail length (in pixels)
    trail_length: u8,
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
    trail_length: u8,
    /// Is this drop active?
    active: bool,
    /// Glyph seed for variation
    glyph_seed: u32,
    /// Life remaining (1.0 = full, 0.0 = dead)
    life: f32,
}

// Implementation block — defines methods for the type above.
impl ShapeDrop {
    fn new() -> Self {
        Self {
            column: 0,
            row: 0,
            previous_column: 0,
            previous_row: 0,
            depth: 0.5,
            progress: 0.0,
            edge_index: 0,
            trail_length: 6,
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
    buffer_pointer: *mut u32,
    buffer_length: usize,
    framebuffer_width: usize,
    framebuffer_height: usize,
    cell_size: usize,
    cell_rows: usize,
    // BrailleMatrix data pointers (immutable during render)
    drops_pointer: *// Compile-time constant — evaluated at compilation, zero runtime cost.
const [RainDrop; DROPS_PER_COLUMN],
    column_depth_pointer: *// Compile-time constant — evaluated at compilation, zero runtime cost.
const u8,
    number_cols: usize,
    // Pre-computed cube geometry
    cube_active: bool,
    cube_minimum_x: f32,
    cube_maximum_x: f32,
    cube_minimum_y: f32,
    cube_maximum_y: f32,
    top_edges: [(f32, f32, f32, f32); 4],
    left_edges: [(f32, f32, f32, f32); 4],
    right_edges: [(f32, f32, f32, f32); 4],
    pad_edges: [(f32, f32, f32, f32); 4],
}

// SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe // Trait implementation — fulfills a behavioral contract.
impl Send for BrailleParallelParams {}
// SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe // Trait implementation — fulfills a behavioral contract.
impl Sync for BrailleParallelParams {}

/// Draw 3×3 micro-glyph at pixel position (raw pointer, parallel-safe)
/// Ultra-fast: only 9 pixel positions, fully unrolled, no loop overhead
#[inline(always)]
// SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe fn draw_glyph_3x3_raw(buffer: *mut u32, framebuffer_width: usize, framebuffer_height: usize,
                              pixel: usize, py: usize, glyph: &[u8; 3], color: u32) {
    if py + 2 >= framebuffer_height || pixel + 2 >= framebuffer_width { return; }
    // Row 0
    let b0 = glyph[0];
    let base0 = py * framebuffer_width + pixel;
    if b0 & 0b001 != 0 { *buffer.add(base0) = color; }
    if b0 & 0b010 != 0 { *buffer.add(base0 + 1) = color; }
    if b0 & 0b100 != 0 { *buffer.add(base0 + 2) = color; }
    // Row 1
    let b1 = glyph[1];
    let base1 = base0 + framebuffer_width;
    if b1 & 0b001 != 0 { *buffer.add(base1) = color; }
    if b1 & 0b010 != 0 { *buffer.add(base1 + 1) = color; }
    if b1 & 0b100 != 0 { *buffer.add(base1 + 2) = color; }
    // Row 2
    let b2 = glyph[2];
    let base2 = base1 + framebuffer_width;
    if b2 & 0b001 != 0 { *buffer.add(base2) = color; }
    if b2 & 0b010 != 0 { *buffer.add(base2 + 1) = color; }
    if b2 & 0b100 != 0 { *buffer.add(base2 + 2) = color; }
}

/// Draw 6x6 glyph at pixel position (raw pointer version for shape overlays)
#[inline]
// SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe fn draw_glyph_6x6_raw(buffer: *mut u32, framebuffer_width: usize, framebuffer_height: usize,
                              pixel: usize, py: usize, glyph: &[u8; 6], color: u32) {
    if py >= framebuffer_height || pixel >= framebuffer_width { return; }
    let maximum_row = (framebuffer_height - py).minimum(6);
    let maximum_column = (framebuffer_width - pixel).minimum(6);
    for row in 0..maximum_row {
        let bits = glyph[row];
        if bits == 0 { continue; }
        let base = (py + row) * framebuffer_width + pixel;
        if bits & 0b000001 != 0 && 0 < maximum_column { *buffer.add(base) = color; }
        if bits & 0b000010 != 0 && 1 < maximum_column { *buffer.add(base + 1) = color; }
        if bits & 0b000100 != 0 && 2 < maximum_column { *buffer.add(base + 2) = color; }
        if bits & 0b001000 != 0 && 3 < maximum_column { *buffer.add(base + 3) = color; }
        if bits & 0b010000 != 0 && 4 < maximum_column { *buffer.add(base + 4) = color; }
        if bits & 0b100000 != 0 && 5 < maximum_column { *buffer.add(base + 5) = color; }
    }
}

/// Point-in-quad winding test (free function for parallel use)
#[inline(always)]
fn point_in_quad_par(pixel: f32, py: f32, edges: &[(f32, f32, f32, f32); 4]) -> bool {
    let mut position = 0u8;
    let mut neg = 0u8;
    for &(ex, ey, ox, oy) in edges.iter() {
        let cross = ex * (py - oy) - ey * (pixel - ox);
        if cross > 0.0 { position += 1; }
        else if cross < 0.0 { neg += 1; }
    }
    position == 0 || neg == 0
}

/// Parallel column renderer — called by each core
fn render_columns_parallel(start: usize, end: usize, data: *mut u8) {
    let p = // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe { &*(data as *// Compile-time constant — evaluated at compilation, zero runtime cost.
const BrailleParallelParams) };
    let cell_size = p.cell_size;
    
    for column in start..end {
        if column >= p.number_cols { break; }
        
        let depth = // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe { *p.column_depth_pointer.add(column) };
        let depth_brightness = 100 + (depth as u32 * 155 / 255);
        
        let column_pixel = (column * cell_size + CELL_CENTER) as f32;
        let column_in_cube_x = p.cube_active && column_pixel >= p.cube_minimum_x && column_pixel <= p.cube_maximum_x;
        
        let drops = // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe { &*p.drops_pointer.add(column) };
        
        for drop_index in 0..DROPS_PER_COLUMN {
            let drop = &drops[drop_index];
            if !drop.active { continue; }
            
            let head_y = drop.y;
            let trail_length = drop.trail_length as usize;
            
            for trail_position in 0..trail_length {
                let cell_y = head_y - trail_position as i32;
                if cell_y < 0 || cell_y >= p.cell_rows as i32 { continue; }
                
                let intensity_index = (trail_position * 63) / trail_length.maximum(1);
                let base_intensity = INTENSITY_LUT[intensity_index.minimum(63)] as u32;
                let intensity = ((base_intensity * depth_brightness) / 255) as u8;
                
                if column_in_cube_x {
                    let py = (cell_y as usize * cell_size + CELL_CENTER) as f32;
                    if py >= p.cube_minimum_y && py <= p.cube_maximum_y {
                        if point_in_quad_par(column_pixel, py, &p.pad_edges) { continue; }
                        if point_in_quad_par(column_pixel, py, &p.top_edges)
                            || point_in_quad_par(column_pixel, py, &p.left_edges)
                            || point_in_quad_par(column_pixel, py, &p.right_edges) {
                            continue;
                        }
                    }
                }
                
                if intensity < 2 { continue; }
                
                let glyph_seed = drop.glyph_seed.wrapping_add(trail_position as u32 * 2654435761);
                let glyph_index = (glyph_seed % GLYPH_3X3_COUNT as u32) as usize;
                let glyph = &MATRIX_GLYPHS_3X3[glyph_index];
                let color = intensity_to_color(intensity);
                
                let pixel = column * cell_size + 1;
                let py = cell_y as usize * cell_size + 1;
                                // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe {
                    draw_glyph_3x3_raw(p.buffer_pointer, p.framebuffer_width, p.framebuffer_height,
                                       pixel, py, glyph, color);
                }
                
                // HEAD GLOW — compact for 3×3 cells
                if trail_position == 0 && intensity > 200 {
                    let gx = pixel + 1; // center of 3×3
                    let gy = py + 1;
                    let offsets: [(i32, i32); 4] = [(-1,0),(1,0),(0,-1),(0,1)];
                    for &(ox, oy) in &offsets {
                        let transmit = gx as i32 + ox;
                        let ty = gy as i32 + oy;
                        if transmit >= 0 && transmit < p.framebuffer_width as i32 && ty >= 0 && ty < p.framebuffer_height as i32 {
                            let index = ty as usize * p.framebuffer_width + transmit as usize;
                                                        // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe {
                                let e = *p.buffer_pointer.add(index);
                                let nr = (((e >> 16) & 0xFF) + 10).minimum(255);
                                let ng = (((e >> 8) & 0xFF) + 48).minimum(255);
                                let nb = ((e & 0xFF) + 32).minimum(255);
                                *p.buffer_pointer.add(index) = 0xFF000000 | (nr << 16) | (ng << 8) | nb;
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
    column_depth: Vec<u8>,
    /// Global RNG seed
    rng: u32,
    /// Frame counter
    frame: u32,
    /// Number of active columns
    number_cols: usize,
    /// Number of rows on screen
    number_rows: usize,
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

// Implementation block — defines methods for the type above.
impl BrailleMatrix {
        // Public function — callable from other modules.
pub fn new() -> Self {
        use alloc::vec::Vec;
        
        let cols = 1280 / CELL_SIZE; // 320 cols at 4px cells (was 160 at 8px)
        let rows = 800 / CELL_SIZE;  // 200 rows at 4px cells (was 100 at 8px)
        
        // Allocate on heap using Vec to avoid stack overflow
        let mut drops: Vec<[RainDrop; DROPS_PER_COLUMN]> = Vec::with_capacity(MAXIMUM_COLUMNS);
        let mut column_depth: Vec<u8> = vec![128u8; MAXIMUM_COLUMNS];
        let mut rng = 0xDEADBEEFu32;
        
        // Initialize drops vec with default values
        for _ in 0..MAXIMUM_COLUMNS {
            drops.push([RainDrop::new_inactive(); DROPS_PER_COLUMN]);
        }
        
        // Assign random depth per column (creates parallax lanes)
        // Use pseudo-noise pattern for natural look (no sin() needed in no_std)
        for column in 0..cols {
            rng = rng.wrapping_mul(1103515245).wrapping_add(12345);
            // Create depth bands: mix of random and position-based pattern
            // Prime-based modulo creates pseudo-wave without floating-point sin
            let pattern = ((column * 17 + 53) % 97) as i32 - 48; // -48 to +48
            let random = (rng % 100) as i32 - 50; // -50 to +50
            let base_depth = 145i32 + pattern + random;
            column_depth[column] = base_depth.clamp(30, 255) as u8;
        }
        
        // Initialize drops with staggered starting positions
        for column in 0..cols {
            let depth = column_depth[column];
            // Track cumulative offset to prevent overlap
            let mut next_start_offset: i32 = 0;
            
            for drop_index in 0..DROPS_PER_COLUMN {
                rng = rng.wrapping_mul(1103515245).wrapping_add(12345);
                
                // Trail length based on depth: closer = longer trails
                let depth_factor = depth as f32 / 255.0;
                let _trail_range = MAXIMUM_TRAIL_LENGTH - MINIMUM_TRAIL_LENGTH;
                let minimum_trail = (MINIMUM_TRAIL_LENGTH as f32 * (0.5 + depth_factor * 0.5)) as usize;
                let maximum_trail = (MAXIMUM_TRAIL_LENGTH as f32 * (0.6 + depth_factor * 0.4)) as usize;
                let trail_length = minimum_trail + (rng % (maximum_trail - minimum_trail + 1) as u32) as usize;
                
                rng = rng.wrapping_mul(1103515245).wrapping_add(12345);
                
                // Gap based on depth: closer = MINIMAL gaps (ultra dense rain!)
                // Far (depth=30): gap 2-8, Close (depth=255): gap 0-2
                let gap_minimum = ((1.0 - depth_factor) * 2.0) as i32; // 0-2
                let gap_range = (2.0 + (1.0 - depth_factor) * 6.0).maximum(1.0) as i32; // 2-8
                let gap = gap_minimum + (rng % gap_range as u32) as i32;
                let start_y = next_start_offset - (rng % 8) as i32;  // Tighter spawn
                
                // Update offset for next drop: must wait until this trail passes
                next_start_offset = start_y - trail_length as i32 - gap;
                
                // Speed based on depth: closer = faster (1-2), far = slower (2-5)
                rng = rng.wrapping_mul(1103515245).wrapping_add(12345);
                let speed_minimum = (1.0 + (1.0 - depth_factor) * 1.5) as u8; // 1-2.5
                let speed_range = (2.0 + (1.0 - depth_factor) * 3.0) as u8; // 2-5
                let speed = speed_minimum + (rng % speed_range as u32) as u8;
                
                rng = rng.wrapping_mul(1103515245).wrapping_add(12345);
                
                drops[column][drop_index] = RainDrop {
                    y: start_y,
                    speed,
                    delay: (rng % speed as u32) as u8,
                    trail_length: trail_length as u8,
                    glyph_seed: rng,
                    active: true,
                };
            }
        }
        
        // Initialize shape_drops on heap
        let mut shape_drops: Vec<ShapeDrop> = Vec::with_capacity(MAXIMUM_SHAPE_DROPS);
        for _ in 0..MAXIMUM_SHAPE_DROPS {
            shape_drops.push(ShapeDrop::new());
        }
        
        // Initialize cube_flow_drops on heap - all INACTIVE, spawned by rain contact
        let mut cube_flow_drops: Vec<CubeFlowDrop> = Vec::with_capacity(MAXIMUM_CUBE_FLOW_DROPS);
        for _ in 0..MAXIMUM_CUBE_FLOW_DROPS {
            cube_flow_drops.push(CubeFlowDrop {
                screen_x: 0.0,
                screen_y: 0.0,
                vel_x: 0.0,
                vel_y: 0.0,
                trail_length: 5,
                active: false,  // Inactive until triggered by rain
                glyph_seed: 0,
                life: 0.0,
            });
        }
        
        Self {
            drops,
            column_depth,
            rng,
            frame: 0,
            number_cols: cols,
            number_rows: rows,
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
            for i in 0..MAXIMUM_CUBE_FLOW_DROPS {
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
        let drop_count = // Pattern matching — Rust's exhaustive branching construct.
match mode {
            ShapeOverlay::Cube => 90,      // 9 visible edges × 10 drops per edge
            ShapeOverlay::Sphere => 64,    // Points on surface
            ShapeOverlay::Torus => 80,     // Double loop
            ShapeOverlay::DNA => 64,       // Helix points
            ShapeOverlay::None => 0,
        };
        
        self.shape_drop_count = drop_count.minimum(MAXIMUM_SHAPE_DROPS);
        
        for i in 0..self.shape_drop_count {
            rng = rng.wrapping_mul(1103515245).wrapping_add(12345);
            self.shape_drops[i] = ShapeDrop {
                column: 0,
                row: 0,
                previous_column: 0,
                previous_row: 0,
                depth: 0.5,
                progress: (i as f32 / self.shape_drop_count as f32),
                edge_index: (i % 12) as u8,
                trail_length: 5 + (rng % 4) as u8,
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
    fn fast_sin(x: f32) -> f32 { crate::math::fast_sin(x) }
    
    /// Fast cos approximation
    #[inline(always)]
    fn fast_cos(x: f32) -> f32 { crate::math::fast_cos(x) }
    
    /// Fast square root approximation
    #[inline(always)]
    fn fast_sqrt(x: f32) -> f32 { crate::math::fast_sqrt(x) }
    
    /// Fast floor - works for both positive and negative values
    #[inline(always)]
    fn fast_floor(x: f32) -> f32 {
        let i = x as i32;
        if (i as f32) > x { (i - 1) as f32 } else { i as f32 }
    }

    /// Update all rain drops
    pub fn update(&mut self) {
        self.frame = self.frame.wrapping_add(1);
        
        let maximum_y = (self.number_rows as i32) + MAXIMUM_TRAIL_LENGTH as i32 + 10;
        
        for column in 0..self.number_cols {
            let depth = self.column_depth[column];
            let depth_factor = depth as f32 / 255.0;
            
            // First pass: collect info about all drops and determine if any need reset
            let mut needs_reset: [bool; DROPS_PER_COLUMN] = [false; DROPS_PER_COLUMN];
            let mut minimum_tail_y: i32 = 0;
            
            for drop_index in 0..DROPS_PER_COLUMN {
                let drop = &self.drops[column][drop_index];
                if drop.active {
                    let tail = drop.tail_y();
                    if tail < minimum_tail_y {
                        minimum_tail_y = tail;
                    }
                }
            }
            
            // Second pass: update positions
            for drop_index in 0..DROPS_PER_COLUMN {
                let drop = &mut self.drops[column][drop_index];
                
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
                if drop.y > maximum_y {
                    needs_reset[drop_index] = true;
                }
            }
            
            // Third pass: reset drops that went off screen
            for drop_index in 0..DROPS_PER_COLUMN {
                if !needs_reset[drop_index] {
                    continue;
                }
                
                // Recalculate min_tail_y excluding drops being reset
                let mut current_minimum_tail: i32 = 0;
                for other_index in 0..DROPS_PER_COLUMN {
                    if other_index != drop_index && !needs_reset[other_index] {
                        let drop = &self.drops[column][other_index];
                        if drop.active {
                            let tail = drop.tail_y();
                            if tail < current_minimum_tail {
                                current_minimum_tail = tail;
                            }
                        }
                    }
                }
                
                self.rng = self.rng.wrapping_mul(1103515245).wrapping_add(12345);
                    
                // Trail length based on depth: closer = longer trails
                let minimum_trail = (MINIMUM_TRAIL_LENGTH as f32 * (0.5 + depth_factor * 0.5)) as usize;
                let maximum_trail = (MAXIMUM_TRAIL_LENGTH as f32 * (0.6 + depth_factor * 0.4)) as usize;
                let new_trail = minimum_trail + (self.rng % (maximum_trail - minimum_trail + 1).maximum(1) as u32) as usize;
                
                self.rng = self.rng.wrapping_mul(1103515245).wrapping_add(12345);
                    
                // Gap based on depth: closer = MINIMAL gaps (ultra dense!)
                let gap_minimum = ((1.0 - depth_factor) * 2.0) as i32; // 0-2
                let gap_range = (2.0 + (1.0 - depth_factor) * 6.0).maximum(1.0) as i32; // 2-8
                let gap = gap_minimum + (self.rng % gap_range as u32) as i32;
                    
                // Start above the lowest other drop's tail - tighter spawn
                let new_y = current_minimum_tail - new_trail as i32 - gap - (self.rng % 5) as i32;
                
                self.rng = self.rng.wrapping_mul(1103515245).wrapping_add(12345);
                    
                // Speed based on depth: closer = faster (1-2), far = slower (2-5)
                let speed_minimum = (1.0 + (1.0 - depth_factor) * 1.5) as u8;
                let speed_range = (2.0 + (1.0 - depth_factor) * 3.0).maximum(1.0) as u8;
                let new_speed = speed_minimum + (self.rng % speed_range as u32) as u8;
                
                self.rng = self.rng.wrapping_mul(1103515245).wrapping_add(12345);
                
                let drop = &mut self.drops[column][drop_index];
                drop.trail_length = new_trail as u8;
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
            let center_x = (self.number_cols * CELL_SIZE / 2) as f32;  // Pixel center X
            let center_y = (self.number_rows * CELL_SIZE / 2) as f32;  // Pixel center Y
            let scale = ((self.number_rows * CELL_SIZE) as f32).minimum((self.number_cols * CELL_SIZE) as f32) * 0.18;  // MUST match entity layer
            
            let count = self.shape_drop_count.minimum(MAXIMUM_SHAPE_DROPS);
            for i in 0..count {
                let drop = &mut self.shape_drops[i];
                
                // Advance progress along edge
                drop.progress += drop.speed;
                if drop.progress >= 1.0 {
                    drop.progress -= 1.0;
                    drop.edge_index = (drop.edge_index + 1) % 9;  // 9 visible edges
                    drop.glyph_seed = drop.glyph_seed.wrapping_mul(1103515245).wrapping_add(12345);
                }
                
                // Calculate 3D position based on shape mode
                let (x, y, z) = // Pattern matching — Rust's exhaustive branching construct.
match self.shape_mode {
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
                        
                        let edge = edges[drop.edge_index as usize % 9];
                        let v1 = verts[edge.0];
                        let v2 = verts[edge.1];
                        
                        // Interpolate along edge
                        let t = drop.progress;
                        let pixel = v1.0 + (v2.0 - v1.0) * t;
                        let py = v1.1 + (v2.1 - v1.1) * t;
                        let pz = v1.2 + (v2.2 - v1.2) * t;
                        
                        // Rotate around Y axis first
                        let rx1 = pixel * cos_y - pz * sin_y;
                        let rz1 = pixel * sin_y + pz * cos_y;
                        
                        // Then rotate around X axis
                        let ry2 = py * cos_x - rz1 * sin_x;
                        let rz2 = py * sin_x + rz1 * cos_x;
                        
                        // Perspective projection (camera at z=5, same as entity layer)
                        let cam_dist = 5.0;
                        let proj_z = rz2 + cam_dist;
                        let perspective = cam_dist / proj_z.maximum(0.5);
                        
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
                drop.previous_column = drop.column;
                drop.previous_row = drop.row;
                drop.column = x;
                drop.row = y;
                drop.depth = z.clamp(0.0, 1.0);
            }
        }
        
        // Note: cube flow drops are now procedural (rendered in render_cube_flow_layer)
        // No particle-based update needed — just advance shape_time
    }
    
    /// Render to framebuffer - OPTIMIZED for maximum FPS
    /// Strategy: Only render active drops, skip empty cells entirely
    pub fn render(&self, buffer: &mut [u32], framebuffer_width: usize, framebuffer_height: usize) {
        // Fast SSE2 fill with dark blue-tinted base (cinematic feel)
        let bg_color = 0xFF010203u32;
        #[cfg(target_arch = "x86_64")]
                // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe {
            crate::graphics::simd::fill_row_sse2(buffer.as_mut_pointer(), buffer.len(), bg_color);
        }
        #[cfg(not(target_arch = "x86_64"))]
        buffer.fill(bg_color);
        
        let cell_size = CELL_SIZE;
        let cell_cols = framebuffer_width / cell_size;
        let cell_rows = framebuffer_height / cell_size;
        
        // Pre-compute cube TOP FACE diamond shape for masking and glow
        let cube_active = self.shape_mode == ShapeOverlay::Cube;
        let center_x = (framebuffer_width / 2) as f32;
        let center_y = (framebuffer_height / 2) as f32;
        let scale = (framebuffer_height as f32).minimum(framebuffer_width as f32) * 0.18;
        
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
            let perspective = cam_dist / proj_z.maximum(1.0);
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
            let persp = cam_dist / proj_z.maximum(1.0);
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
        let cube_minimum_x = if cube_active { cv0.0.minimum(cv1.0).minimum(cv2.0).minimum(cv3.0).minimum(cv4.0).minimum(cv7.0).minimum(cv_t2.0) - 2.0 } else { 0.0 };
        let cube_maximum_x = if cube_active { cv0.0.maximum(cv1.0).maximum(cv2.0).maximum(cv3.0).maximum(cv4.0).maximum(cv7.0).maximum(cv_t2.0) + 2.0 } else { 0.0 };
        let cube_minimum_y = if cube_active { cv0.1.minimum(cv1.1).minimum(cv2.1).minimum(cv3.1).minimum(cv4.1).minimum(cv7.1).minimum(cv_t2.1) - 20.0 } else { 0.0 };
        let cube_maximum_y = if cube_active { cv0.1.maximum(cv1.1).maximum(cv2.1).maximum(cv3.1).maximum(cv4.1).maximum(cv7.1).maximum(cv_t2.1) + 2.0 } else { 0.0 };
        
        // Pre-compute glow metrics ONCE (not per-cell)
        let diamond_half_w = if cube_active { (top_right_x - top_left_x) / 2.0 } else { 1.0 };
        let diamond_half_h = if cube_active { (top_front_y - top_back_y) / 2.0 } else { 1.0 };
        let diamond_center_x_g = (top_left_x + top_right_x) / 2.0;
        let diamond_center_y_g = (top_back_y + top_front_y) / 2.0;
        let inv_diamond_half_w = 1.0 / diamond_half_w.maximum(1.0);
        let inv_diamond_half_h = 1.0 / diamond_half_h.maximum(1.0);
        
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
        fn point_in_quad_fast(pixel: f32, py: f32, edges: &[(f32, f32, f32, f32); 4]) -> bool {
            let mut position = 0u8;
            let mut neg = 0u8;
            for &(ex, ey, ox, oy) in edges.iter() {
                let cross = ex * (py - oy) - ey * (pixel - ox);
                if cross > 0.0 { position += 1; } 
                else if cross < 0.0 { neg += 1; }
            }
            position == 0 || neg == 0
        }
        
        // PARALLEL column rendering via SMP — split columns across cores
        let total_cols = cell_cols.minimum(self.number_cols);
        let params = BrailleParallelParams {
            buffer_pointer: buffer.as_mut_pointer(),
            buffer_length: buffer.len(),
            framebuffer_width,
            framebuffer_height,
            cell_size,
            cell_rows,
            drops_pointer: self.drops.as_pointer(),
            column_depth_pointer: self.column_depth.as_pointer(),
            number_cols: self.number_cols,
            cube_active,
            cube_minimum_x,
            cube_maximum_x,
            cube_minimum_y,
            cube_maximum_y,
            top_edges,
            left_edges,
            right_edges,
            pad_edges,
        };
        
        crate::cpu::smp::parallel_for(
            total_cols,
            render_columns_parallel,
            &params as *// Compile-time constant — evaluated at compilation, zero runtime cost.
const BrailleParallelParams as *mut u8,
        );
        
        // Shape overlay is now handled by render_entity_layer()
        // This allows rain to be rendered first, then entity on top
    }
    
    /// Render entity overlay - white pixel layer on top of rain
    /// Uses small 2x2 pixels for clean geometric shapes
    pub fn render_entity_layer(&self, buffer: &mut [u32], framebuffer_width: usize, framebuffer_height: usize) {
        if self.shape_mode == ShapeOverlay::None {
            return;
        }
        
        let center_x = (framebuffer_width / 2) as f32;
        let center_y = (framebuffer_height / 2) as f32;
        let scale = (framebuffer_height as f32).minimum(framebuffer_width as f32) * 0.18;  // Smaller cube
        
        // Time for animation
        let time = self.shape_time;
        
                // Pattern matching — Rust's exhaustive branching construct.
match self.shape_mode {
            ShapeOverlay::Cube => {
                // No wireframe — the rain interaction defines the cube shape
            },
            ShapeOverlay::Sphere => {
                // Wireframe sphere - latitude/longitude lines
                let number_segments = 24;
                
                // Draw longitude lines (vertical circles)
                for i in 0..6 {
                    let phi = (i as f32 / 6.0) * 3.14159;
                    let previous_color = 0xFF00AA44;  // Matrix green
                    
                    let mut previous_x = 0i32;
                    let mut previous_y = 0i32;
                    let mut first = true;
                    
                    for j in 0..=number_segments {
                        let theta = (j as f32 / number_segments as f32) * 3.14159 * 2.0;
                        
                        let x = Self::fast_sin(theta) * Self::fast_cos(phi);
                        let z = Self::fast_sin(theta) * Self::fast_sin(phi);
                        let y = Self::fast_cos(theta);
                        
                        // Rotate by time for subtle motion
                        let rot_angle = time * 0.1;
                        let receive = x * Self::fast_cos(rot_angle) - z * Self::fast_sin(rot_angle);
                        let rz = x * Self::fast_sin(rot_angle) + z * Self::fast_cos(rot_angle);
                        
                        let perspective = 3.0 / (4.0 + rz);
                        let pixel = (center_x + receive * scale * perspective) as i32;
                        let py = (center_y + y * scale * perspective * 0.8) as i32;
                        
                        if !first {
                            let depth = (rz + 1.0) / 2.0;
                            let color = if depth > 0.5 { 0xFFCCFFCC } else { previous_color };  // Matrix green
                            self.draw_line_thick(buffer, framebuffer_width, framebuffer_height, 
                                               previous_x, previous_y, pixel, py, color, 2);
                        }
                        previous_x = pixel;
                        previous_y = py;
                        first = false;
                    }
                }
                
                // Draw latitude lines (horizontal circles)
                for i in 1..4 {
                    let y_level = -0.75 + (i as f32 * 0.5);
                    let radius = (1.0 - y_level * y_level).maximum(0.0);
                    let radius = {
                        let mut x = radius;
                        let mut y = radius * 0.5;
                        for _ in 0..4 { y = (y + x / y) * 0.5; }
                        y
                    };
                    
                    let mut previous_x = 0i32;
                    let mut previous_y = 0i32;
                    let mut first = true;
                    
                    for j in 0..=number_segments {
                        let angle = (j as f32 / number_segments as f32) * 3.14159 * 2.0 + time * 0.1;
                        let x = Self::fast_cos(angle) * radius;
                        let z = Self::fast_sin(angle) * radius;
                        
                        let perspective = 3.0 / (4.0 + z);
                        let pixel = (center_x + x * scale * perspective) as i32;
                        let py = (center_y + y_level * scale * perspective * 0.8) as i32;
                        
                        if !first {
                            let depth = (z + 1.0) / 2.0;
                            let color = if depth > 0.5 { 0xFFCCFFCC } else { 0xFF00AA44 };  // Matrix green
                            self.draw_line_thick(buffer, framebuffer_width, framebuffer_height, 
                                               previous_x, previous_y, pixel, py, color, 2);
                        }
                        previous_x = pixel;
                        previous_y = py;
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
                    
                    let mut previous_x = 0i32;
                    let mut previous_y = 0i32;
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
                        let receive = x * Self::fast_cos(rot) - z * Self::fast_sin(rot);
                        let rz = x * Self::fast_sin(rot) + z * Self::fast_cos(rot);
                        
                        let perspective = 3.0 / (4.0 + rz);
                        let pixel = (center_x + receive * scale * perspective) as i32;
                        let py = (center_y + y * scale * perspective) as i32;
                        
                        if !first {
                            let depth = (rz + 1.0) / 2.0;
                            let color = if depth > 0.5 { 0xFFCCFFCC } else { 0xFF00AA44 };  // Matrix green
                            self.draw_line_thick(buffer, framebuffer_width, framebuffer_height, 
                                               previous_x, previous_y, pixel, py, color, 2);
                        }
                        previous_x = pixel;
                        previous_y = py;
                        first = false;
                    }
                }
            },
            ShapeOverlay::DNA => {
                // Double helix
                let helix_height = scale * 1.5;
                let helix_radius = scale * 0.3;
                let segments = 40;
                
                let mut previous_x1 = 0i32;
                let mut previous_y1 = 0i32;
                let mut previous_x2 = 0i32;
                let mut previous_y2 = 0i32;
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
                        self.draw_line_thick(buffer, framebuffer_width, framebuffer_height, 
                                           previous_x1, previous_y1, px1, py1, color1, 2);
                        self.draw_line_thick(buffer, framebuffer_width, framebuffer_height, 
                                           previous_x2, previous_y2, px2, py2, color2, 2);
                        
                        // Draw connecting rungs every 4 segments
                        if i % 4 == 0 {
                            self.draw_line_thick(buffer, framebuffer_width, framebuffer_height, 
                                               px1, py1, px2, py2, 0xFF44FF44, 1);
                        }
                    }
                    previous_x1 = px1;
                    previous_y1 = py1;
                    previous_x2 = px2;
                    previous_y2 = py2;
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
    pub fn render_cube_flow_layer(&self, buffer: &mut [u32], framebuffer_width: usize, framebuffer_height: usize) {
        if self.shape_mode != ShapeOverlay::Cube {
            return;
        }
        
        let cell_size = CELL_SIZE;
        let screen_width = (self.number_cols * cell_size) as f32;
        let screen_height = (self.number_rows * cell_size) as f32;
        let center_x = screen_width / 2.0;
        let center_y = screen_height / 2.0;
        let scale = screen_height.minimum(screen_width) * 0.18;
        
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
            let perspective = cam_dist / proj_z.maximum(1.0);
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
        let mut bb_minimum_x = all_pts[0].0;
        let mut bb_maximum_x = all_pts[0].0;
        let mut bb_minimum_y = all_pts[0].1;
        let mut bb_maximum_y = all_pts[0].1;
        for p in &all_pts[1..] {
            if p.0 < bb_minimum_x { bb_minimum_x = p.0; }
            if p.0 > bb_maximum_x { bb_maximum_x = p.0; }
            if p.1 < bb_minimum_y { bb_minimum_y = p.1; }
            if p.1 > bb_maximum_y { bb_maximum_y = p.1; }
        }
        
        let cell_x0 = ((bb_minimum_x / cell_size as f32) as i32).maximum(0) as usize;
        let cell_x1 = ((bb_maximum_x / cell_size as f32) as i32 + 1).minimum(self.number_cols as i32) as usize;
        let cell_y0 = ((bb_minimum_y / cell_size as f32) as i32).maximum(0) as usize;
        let cell_y1 = ((bb_maximum_y / cell_size as f32) as i32 + 1).minimum(self.number_rows as i32) as usize;
        
        let point_in_quad = |pixel: f32, py: f32, q: &[(f32, f32); 4]| -> bool {
            let mut position = 0i32;
            let mut neg = 0i32;
            for i in 0..4 {
                let j = (i + 1) % 4;
                let cross = (q[j].0 - q[i].0) * (py - q[i].1) - (q[j].1 - q[i].1) * (pixel - q[i].0);
                if cross > 0.0 { position += 1; } 
                else if cross < 0.0 { neg += 1; }
            }
            position == 0 || neg == 0
        };
        
        let time = self.shape_time;
        let number_lanes = 8.0_f32;   // number of rain streams per axis
        let lane_width = 0.35_f32; // fraction of lane that is "on"
        
        for cy in cell_y0..cell_y1 {
            for cx in cell_x0..cell_x1 {
                let pixel = (cx * cell_size + CELL_CENTER) as f32;
                let py = (cy * cell_size + CELL_CENTER) as f32;
                
                // === TOP FACE: diagonal rain following isometric edges ===
                if top_det.absolute() > 0.01 && point_in_quad(pixel, py, &top_quad) {
                    // Compute UV on the top face (0..1 range)
                    let dpx = pixel - t0.0;
                    let dpy = py - t0.1;
                    let inv_det = 1.0 / top_det;
                    let u = (dpx * top_v_dy - dpy * top_v_dx) * inv_det;
                    let v = (top_u_dx * dpy - top_u_dy * dpx) * inv_det;
                    
                    // Lane coordinates: u selects which u-lane, v selects which v-lane
                    let u_lane = u * number_lanes;
                    let v_lane = v * number_lanes;
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
                                brightness = (1.0 - dm / trail).maximum(0.0);
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
                                let b2 = (1.0 - dm / trail).maximum(0.0);
                                if b2 > brightness { brightness = b2; }
                            }
                        }
                        
                        if brightness < 0.08 { brightness = 0.08; }
                        
                        // Glyph selection
                        let cell_hash = (cx as u32).wrapping_mul(2654435761)
                            .wrapping_add((cy as u32).wrapping_mul(340573321));
                        let anim_frame = (time * 8.0) as u32;
                        let glyph_index = ((cell_hash.wrapping_add(anim_frame)) % GLYPH_3X3_COUNT as u32) as usize;
                        let glyph = &MATRIX_GLYPHS_3X3[glyph_index];
                        
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
                        
                        self.draw_glyph_3x3(buffer, framebuffer_width, cx * cell_size + 1, cy * cell_size + 1, glyph, color);
                    }
                    continue;
                }
                
                // === LEFT FACE: vertical rain columns ===
                if point_in_quad(pixel, py, &left_quad) {
                    let column_pixel = 10.0_f32;
                    let column_f = pixel / column_pixel;
                    let column_frac = column_f - Self::fast_floor(column_f);
                    
                    if column_frac < 0.4 {
                        let column_id = Self::fast_floor(column_f) as i32;
                        let seed = (column_id as u32).wrapping_mul(2654435761);
                        let scroll_speed = 2.5 + (seed % 6) as f32 * 0.3;
                        let phase = (seed % 100) as f32 * 0.05;
                        let head = time * scroll_speed + phase;
                        let position = py / column_pixel;
                        let trail = 3.0;
                        let period = 4.0 + (seed % 3) as f32 * 0.5;
                        let d = position - head;
                        let dm = d - Self::fast_floor(d / period) * period;
                        
                        let mut brightness: f32 = 0.06;
                        if dm >= 0.0 && dm < trail {
                            brightness = (1.0 - dm / trail).maximum(0.0);
                            if brightness < 0.06 { brightness = 0.06; }
                        }
                        
                        let cell_hash = (cx as u32).wrapping_mul(2654435761)
                            .wrapping_add((cy as u32).wrapping_mul(340573321));
                        let anim_frame = (time * 6.0) as u32;
                        let glyph_index = ((cell_hash.wrapping_add(anim_frame)) % GLYPH_3X3_COUNT as u32) as usize;
                        let glyph = &MATRIX_GLYPHS_3X3[glyph_index];
                        
                        let color = if brightness > 0.88 {
                            let g = (brightness * 200.0) as u8;
                            0xFF000000 | 0x00200000 | ((g as u32) << 8) | 0x18
                        } else {
                            let g = (12.0 + brightness * 140.0) as u8;
                            0xFF000000 | ((g as u32) << 8) | 0x04
                        };
                        
                        self.draw_glyph_3x3(buffer, framebuffer_width, cx * cell_size + 1, cy * cell_size + 1, glyph, color);
                    }
                    continue;
                }
                
                // === RIGHT FACE (z=-1): vertical rain columns ===
                if point_in_quad(pixel, py, &right_quad) {
                    let column_pixel = 10.0_f32;
                    let column_f = pixel / column_pixel;
                    let column_frac = column_f - Self::fast_floor(column_f);
                    
                    if column_frac < 0.4 {
                        let column_id = Self::fast_floor(column_f) as i32;
                        let seed = (column_id as u32).wrapping_mul(340573321);
                        let scroll_speed = 2.8 + (seed % 5) as f32 * 0.25;
                        let phase = (seed % 100) as f32 * 0.05;
                        let head = time * scroll_speed + phase;
                        let position = py / column_pixel;
                        let trail = 3.5;
                        let period = 4.5 + (seed % 3) as f32 * 0.5;
                        let d = position - head;
                        let dm = d - Self::fast_floor(d / period) * period;
                        
                        let mut brightness: f32 = 0.06;
                        if dm >= 0.0 && dm < trail {
                            brightness = (1.0 - dm / trail).maximum(0.0);
                            if brightness < 0.06 { brightness = 0.06; }
                        }
                        
                        let cell_hash = (cx as u32).wrapping_mul(2654435761)
                            .wrapping_add((cy as u32).wrapping_mul(340573321));
                        let anim_frame = (time * 7.0) as u32;
                        let glyph_index = ((cell_hash.wrapping_add(anim_frame)) % GLYPH_3X3_COUNT as u32) as usize;
                        let glyph = &MATRIX_GLYPHS_3X3[glyph_index];
                        
                        let color = if brightness > 0.88 {
                            let g = (brightness * 230.0) as u8;
                            0xFF000000 | 0x00280000 | ((g as u32) << 8) | 0x20
                        } else {
                            let g = (15.0 + brightness * 170.0) as u8;
                            0xFF000000 | ((g as u32) << 8) | 0x06
                        };
                        
                        self.draw_glyph_3x3(buffer, framebuffer_width, cx * cell_size + 1, cy * cell_size + 1, glyph, color);
                    }
                }
            }
        }
    }
    
    /// Draw a thick line using Bresenham algorithm with width
    fn draw_line_thick(&self, buffer: &mut [u32], framebuffer_width: usize, framebuffer_height: usize,
                       x0: i32, y0: i32, x1: i32, y1: i32, color: u32, thickness: i32) {
        // Safety: skip if both endpoints are way off-screen
        let margin = 100i32;
        let w = framebuffer_width as i32;
        let h = framebuffer_height as i32;
        if (x0 < -margin && x1 < -margin) || (x0 > w + margin && x1 > w + margin) {
            return;
        }
        if (y0 < -margin && y1 < -margin) || (y0 > h + margin && y1 > h + margin) {
            return;
        }
        
        let dx = (x1 - x0).absolute();
        let dy = -(y1 - y0).absolute();
        let sx = if x0 < x1 { 1 } else { -1 };
        let sy = if y0 < y1 { 1 } else { -1 };
        let mut error = dx + dy;
        
        let mut x = x0;
        let mut y = y0;
        
        // Safety: limit iterations to prevent infinite loops
        let maximum_steps = (dx.absolute() + (-dy).absolute() + 10) as usize;
        let mut steps = 0usize;
        
                // Infinite loop — runs until an explicit `break`.
loop {
            steps += 1;
            if steps > maximum_steps { break; }
            
            // Draw thick pixel (cross pattern for thickness)
            for ty in -thickness/2..=thickness/2 {
                for transmit in -thickness/2..=thickness/2 {
                    let pixel = x + transmit;
                    let py = y + ty;
                    if pixel >= 0 && py >= 0 {
                        let pxu = pixel as usize;
                        let pyu = py as usize;
                        if pxu < framebuffer_width && pyu < framebuffer_height {
                            buffer[pyu * framebuffer_width + pxu] = color;
                        }
                    }
                }
            }
            
            if x == x1 && y == y1 { break; }
            
            let e2 = 2 * error;
            if e2 >= dy {
                error += dy;
                x += sx;
            }
            if e2 <= dx {
                error += dx;
                y += sy;
            }
        }
    }
    
    /// Draw a line with Matrix flow effect - pulses travel along the edge
    fn draw_line_matrix_flow(&self, buffer: &mut [u32], framebuffer_width: usize, framebuffer_height: usize,
                              x0: i32, y0: i32, x1: i32, y1: i32, 
                              base_color: u32, depth: f32, edge_index: usize) {
        // Safety: skip if both endpoints are way off-screen
        let margin = 100i32;
        let w = framebuffer_width as i32;
        let h = framebuffer_height as i32;
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
        let line_length = (dx_f * dx_f + dy_f * dy_f).maximum(1.0);
        let line_length = {
            let mut x = line_length;
            let mut y = line_length * 0.5;
            for _ in 0..4 { y = (y + x / y) * 0.5; }
            y
        };
        
        let dx = (x1 - x0).absolute();
        let dy = -(y1 - y0).absolute();
        let sx = if x0 < x1 { 1 } else { -1 };
        let sy = if y0 < y1 { 1 } else { -1 };
        let mut error = dx + dy;
        
        let mut x = x0;
        let mut y = y0;
        let mut pixel_index = 0usize;
        
        // Safety: limit iterations
        let maximum_steps = (dx.absolute() + (-dy).absolute() + 10) as usize;
        let mut steps = 0usize;
        
        // Number of "data pulses" traveling along this edge
        let number_pulses = 3;
        // Speed of pulses (cycles per second)
        let pulse_speed = 2.0 + (edge_index as f32 * 0.3);
        // Width of each pulse (in pixels)
        let pulse_width = 12.0;
        
                // Infinite loop — runs until an explicit `break`.
loop {
            steps += 1;
            if steps > maximum_steps { break; }
            
            // Calculate position along line (0.0 to 1.0)
            let t = pixel_index as f32 / line_length.maximum(1.0);
            
            // Calculate pulse intensity at this position
            let mut pulse_intensity = 0.0f32;
            for p in 0..number_pulses {
                // Each pulse offset in time and position
                let pulse_phase = (time * pulse_speed + p as f32 / number_pulses as f32) % 1.0;
                let pulse_center = pulse_phase;
                
                // Distance from pulse center (wrapping)
                let dist1 = (t - pulse_center).absolute();
                let dist2 = (t - pulse_center - 1.0).absolute();
                let dist3 = (t - pulse_center + 1.0).absolute();
                let dist = dist1.minimum(dist2).minimum(dist3);
                
                // Gaussian-like falloff for pulse
                let pulse_t = dist * line_length / pulse_width;
                if pulse_t < 1.0 {
                    pulse_intensity += (1.0 - pulse_t * pulse_t).maximum(0.0);
                }
            }
            pulse_intensity = pulse_intensity.minimum(1.0);
            
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
                for transmit in -thickness/2..=thickness/2 {
                    let pixel = x + transmit;
                    let py_coord = y + ty;
                    if pixel >= 0 && py_coord >= 0 {
                        let pxu = pixel as usize;
                        let pyu = py_coord as usize;
                        if pxu < framebuffer_width && pyu < framebuffer_height {
                            buffer[pyu * framebuffer_width + pxu] = color;
                        }
                    }
                }
            }
            
            pixel_index += 1;
            
            if x == x1 && y == y1 { break; }
            
            let e2 = 2 * error;
            if e2 >= dy {
                error += dy;
                x += sx;
            }
            if e2 <= dx {
                error += dx;
                y += sy;
            }
        }
    }
    
    /// Draw a 3×3 micro-glyph at pixel position — high-res version
    #[inline(always)]
    fn draw_glyph_3x3(&self, buffer: &mut [u32], framebuffer_width: usize,
                       pixel: usize, py: usize, glyph: &[u8; 3], color: u32) {
        let framebuffer_height = buffer.len() / framebuffer_width;
        if py + 2 >= framebuffer_height || pixel + 2 >= framebuffer_width { return; }
        // Row 0
        let b0 = glyph[0];
        let base0 = py * framebuffer_width + pixel;
        if b0 & 0b001 != 0 { buffer[base0] = color; }
        if b0 & 0b010 != 0 { buffer[base0 + 1] = color; }
        if b0 & 0b100 != 0 { buffer[base0 + 2] = color; }
        // Row 1
        let b1 = glyph[1];
        let base1 = base0 + framebuffer_width;
        if b1 & 0b001 != 0 { buffer[base1] = color; }
        if b1 & 0b010 != 0 { buffer[base1 + 1] = color; }
        if b1 & 0b100 != 0 { buffer[base1 + 2] = color; }
        // Row 2
        let b2 = glyph[2];
        let base2 = base1 + framebuffer_width;
        if b2 & 0b001 != 0 { buffer[base2] = color; }
        if b2 & 0b010 != 0 { buffer[base2 + 1] = color; }
        if b2 & 0b100 != 0 { buffer[base2 + 2] = color; }
    }

    /// Draw a 6x6 glyph at pixel position (kept for entity overlay)
    #[inline(always)]
    fn draw_glyph_6x6(&self, buffer: &mut [u32], framebuffer_width: usize, 
                       pixel: usize, py: usize, glyph: &[u8; 6], color: u32) {
        let framebuffer_height = buffer.len() / framebuffer_width;
        if py >= framebuffer_height || pixel >= framebuffer_width { return; }
        let maximum_row = (framebuffer_height - py).minimum(6);
        let maximum_column = (framebuffer_width - pixel).minimum(6);
        for row in 0..maximum_row {
            let row_bits = glyph[row];
            if row_bits == 0 { continue; }
            let row_start = (py + row) * framebuffer_width + pixel;
            if row_bits & 0b000001 != 0 && 0 < maximum_column { buffer[row_start] = color; }
            if row_bits & 0b000010 != 0 && 1 < maximum_column { buffer[row_start + 1] = color; }
            if row_bits & 0b000100 != 0 && 2 < maximum_column { buffer[row_start + 2] = color; }
            if row_bits & 0b001000 != 0 && 3 < maximum_column { buffer[row_start + 3] = color; }
            if row_bits & 0b010000 != 0 && 4 < maximum_column { buffer[row_start + 4] = color; }
            if row_bits & 0b100000 != 0 && 5 < maximum_column { buffer[row_start + 5] = color; }
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

// Implementation block — defines methods for the type above.
impl FastMatrixRenderer {
        // Public function — callable from other modules.
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
    pub fn render(&self, buffer: &mut [u32], framebuffer_width: usize, framebuffer_height: usize) {
        let cols = (framebuffer_width / CELL_PIXEL_W).minimum(self.cols);
        let rows = (framebuffer_height / CELL_PIXEL_H).minimum(self.rows);
        
        for column in 0..cols {
            let head_y = self.heads[column];
            
            for row in 0..rows {
                let dist = head_y - (row as i32);
                
                if dist >= 0 && dist < 16 {
                    // Calculate intensity
                    let intensity = INTENSITY_LUT[(dist as usize).minimum(31)];
                    if intensity > 10 {
                        // Get character for this cell
                        let c = self.chars[column * self.rows + row];
                        
                        // Color: bright green fading
                        let color = (0xFF << 24) | ((intensity as u32) << 8);
                        
                        // Draw character
                        self.draw_char(buffer, framebuffer_width, column, row, c, color);
                    }
                }
            }
        }
    }
    
    /// Draw character using font
    fn draw_char(&self, buffer: &mut [u32], framebuffer_width: usize, 
                 column: usize, row: usize, c: u8, color: u32) {
        let pixel = column * CELL_PIXEL_W;
        let py = row * CELL_PIXEL_H;
        
        let glyph = crate::framebuffer::font::get_glyph(c as char);
        
        for (gy, &bits) in glyph.iter().enumerate() {
            let y = py + gy;
            if y >= buffer.len() / framebuffer_width { continue; }
            
            for gx in 0..8 {
                if (bits >> (7 - gx)) & 1 != 0 {
                    let x = pixel + gx;
                    let index = y * framebuffer_width + x;
                    if index < buffer.len() {
                        buffer[index] = color;
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
const MAXIMUM_DROPS_3D: usize = 200;

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
    trail_length: u8,
    /// Glyph randomizer
    glyph_seed: u32,
    /// Is drop on a surface?
    on_surface: bool,
    /// Surface flow timer (resets drop after flowing)
    flow_time: u8,
}

// Implementation block — defines methods for the type above.
impl Drop3D {
    fn new() -> Self {
        Self {
            x: 0.0, y: -10.0, z: 0.5,
            vx: 0.0, vy: 0.5, vz: 0.0,
            trail_length: 20,
            glyph_seed: 0,
            on_surface: false,
            flow_time: 0,
        }
    }
}

/// 3D shape types for collision
#[derive(Clone, Copy)]
// Enumeration — a type that can be one of several variants.
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
    drops: [Drop3D; MAXIMUM_DROPS_3D],
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

// Implementation block — defines methods for the type above.
impl Matrix3D {
        // Public function — callable from other modules.
pub fn new() -> Self {
        let mut drops = [Drop3D::new(); MAXIMUM_DROPS_3D];
        let mut rng = 0xDEADBEEFu32;
        
        // Initialize drops with random positions
        for drop in drops.iterator_mut() {
            rng = rng.wrapping_mul(1103515245).wrapping_add(12345);
            drop.x = (rng % 160) as f32;
            rng = rng.wrapping_mul(1103515245).wrapping_add(12345);
            drop.y = -((rng % 120) as f32);
            rng = rng.wrapping_mul(1103515245).wrapping_add(12345);
            drop.z = 0.2 + (rng % 80) as f32 / 100.0; // 0.2 to 1.0
            
            // Speed based on depth (closer = faster)
            drop.vy = 0.3 + drop.z * 0.7; // 0.3 to 1.0
            
            rng = rng.wrapping_mul(1103515245).wrapping_add(12345);
            drop.trail_length = 10 + (rng % 30) as u8;
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
        for slot in self.shapes.iterator_mut() {
            if slot.is_none() {
                *slot = Some(shape);
                return;
            }
        }
    }
    
    /// Clear all shapes
    pub fn clear_shapes(&mut self) {
        for slot in self.shapes.iterator_mut() {
            *slot = None;
        }
    }
    
    /// Fast approximate sin (delegates to shared math)
    #[inline(always)]
    fn fast_sin(x: f32) -> f32 { crate::math::fast_sin(x) }
    
    /// Fast approximate cos
    #[inline(always)]
    fn fast_cos(x: f32) -> f32 { crate::math::fast_cos(x) }
    
    /// Check if point is inside a shape, return (is_inside, surface_normal)
    fn check_collision_static(shapes: &[Option<Shape3D>; 4], x: f32, y: f32, z: f32) -> Option<(f32, f32, f32)> {
        for shape in shapes.iter().filter_map(|s| *s) {
                        // Pattern matching — Rust's exhaustive branching construct.
match shape {
                Shape3D::Sphere { cx, cy, cz, r } => {
                    let dx = x - cx;
                    let dy = y - cy;
                    let dz = z - cz;
                    let dist_sq = dx * dx + dy * dy + dz * dz;
                    let r_sq = r * r;
                    
                    if dist_sq < r_sq {
                        // Inside sphere - calculate outward normal
                        let dist = Self::fast_sqrt(dist_sq).maximum(0.01);
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
                    let receive = dx * cos_r - dz * sin_r;
                    let ry = dy;
                    let rz = dx * sin_r + dz * cos_r;
                    
                    // Check if inside axis-aligned box
                    if receive.absolute() < half && ry.absolute() < half && rz.absolute() < half {
                        // Find closest face for normal
                        let ax = half - receive.absolute();
                        let ay = half - ry.absolute();
                        let az = half - rz.absolute();
                        
                        // Rotate normal back
                        if ax < ay && ax < az {
                            let nx = if receive > 0.0 { 1.0 } else { -1.0 };
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
                        let directory_x = if dist_xz > 0.01 { dx / dist_xz } else { 1.0 };
                        let directory_z = if dist_xz > 0.01 { dz / dist_xz } else { 0.0 };
                        
                        let nx = tube_dx * directory_x / tube_dist.maximum(0.01);
                        let ny = dy / tube_dist.maximum(0.01);
                        let nz = tube_dx * directory_z / tube_dist.maximum(0.01);
                        
                        return Some((nx, ny, nz));
                    }
                }
            }
        }
        None
    }
    
    /// Fast square root approximation
    #[inline(always)]
    fn fast_sqrt(x: f32) -> f32 { crate::math::fast_sqrt(x) }
    
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
        
        for i in 0..MAXIMUM_DROPS_3D {
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
                    drop.vy += gravity * (1.0 - grav_dot * grav_dot).maximum(0.0);
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
    pub fn render(&self, buffer: &mut [u32], framebuffer_width: usize, framebuffer_height: usize) {
        // Fill background
        let bg_color = 0xFF010201u32;
        buffer.fill(bg_color);
        
        let cell_size = CELL_SIZE;
        
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
            let trail_length = drop.trail_length as usize;
            for trail_position in 0..trail_length {
                // Trail position in world space
                let ty = drop.y - trail_position as f32 * 0.8;
                if ty < 0.0 { continue; }
                
                let (transmit, ty_screen, _) = self.project(drop.x, ty, drop.z);
                
                if transmit < 0 || transmit >= (framebuffer_width / cell_size) as i32 { continue; }
                if ty_screen < 0 || ty_screen >= (framebuffer_height / cell_size) as i32 { continue; }
                
                // Intensity fades along trail
                let intensity_index = (trail_position * 63) / trail_length.maximum(1);
                let base_intensity = INTENSITY_LUT[intensity_index.minimum(63)] as u32;
                let intensity = (((base_intensity * depth_brightness) / 255) + surface_boost).minimum(255) as u8;
                
                if intensity < 2 { continue; }
                
                // Get glyph
                let glyph_seed = drop.glyph_seed.wrapping_add(trail_position as u32 * 2654435761);
                let glyph_index = (glyph_seed % GLYPH_3X3_COUNT as u32) as usize;
                let glyph = &MATRIX_GLYPHS_3X3[glyph_index];
                let color = intensity_to_color(intensity);
                
                let pixel = transmit as usize * cell_size + 1;
                let py = ty_screen as usize * cell_size + 1;
                self.draw_glyph_3x3(buffer, framebuffer_width, pixel, py, glyph, color);
            }
        }
        
        // Optional: Draw shape outlines for debugging
        // self.draw_shape_outline(buffer, fb_width, fb_height);
    }
    
    /// Draw a 3×3 micro-glyph at pixel position — high-res version
    #[inline(always)]
    fn draw_glyph_3x3(&self, buffer: &mut [u32], framebuffer_width: usize,
                       pixel: usize, py: usize, glyph: &[u8; 3], color: u32) {
        let framebuffer_height = buffer.len() / framebuffer_width;
        if py + 2 >= framebuffer_height || pixel + 2 >= framebuffer_width { return; }
        let b0 = glyph[0]; let base0 = py * framebuffer_width + pixel;
        if b0 & 0b001 != 0 { buffer[base0] = color; }
        if b0 & 0b010 != 0 { buffer[base0 + 1] = color; }
        if b0 & 0b100 != 0 { buffer[base0 + 2] = color; }
        let b1 = glyph[1]; let base1 = base0 + framebuffer_width;
        if b1 & 0b001 != 0 { buffer[base1] = color; }
        if b1 & 0b010 != 0 { buffer[base1 + 1] = color; }
        if b1 & 0b100 != 0 { buffer[base1 + 2] = color; }
        let b2 = glyph[2]; let base2 = base1 + framebuffer_width;
        if b2 & 0b001 != 0 { buffer[base2] = color; }
        if b2 & 0b010 != 0 { buffer[base2 + 1] = color; }
        if b2 & 0b100 != 0 { buffer[base2 + 2] = color; }
    }
    
    /// Draw a 6x6 glyph (kept for compatibility)
    #[inline(always)]
    fn draw_glyph_6x6(&self, buffer: &mut [u32], framebuffer_width: usize, 
                       pixel: usize, py: usize, glyph: &[u8; 6], color: u32) {
        let framebuffer_height = buffer.len() / framebuffer_width;
        if py >= framebuffer_height || pixel >= framebuffer_width { return; }
        let maximum_row = (framebuffer_height - py).minimum(6);
        let maximum_column = (framebuffer_width - pixel).minimum(6);
        for row in 0..maximum_row {
            let row_bits = glyph[row];
            if row_bits == 0 { continue; }
            let row_start = (py + row) * framebuffer_width + pixel;
            if row_bits & 0b000001 != 0 && 0 < maximum_column { buffer[row_start] = color; }
            if row_bits & 0b000010 != 0 && 1 < maximum_column { buffer[row_start + 1] = color; }
            if row_bits & 0b000100 != 0 && 2 < maximum_column { buffer[row_start + 2] = color; }
            if row_bits & 0b001000 != 0 && 3 < maximum_column { buffer[row_start + 3] = color; }
            if row_bits & 0b010000 != 0 && 4 < maximum_column { buffer[row_start + 4] = color; }
            if row_bits & 0b100000 != 0 && 5 < maximum_column { buffer[row_start + 5] = color; }
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
