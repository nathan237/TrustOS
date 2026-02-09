//! HoloVolume V13 – Color Modifier for Existing Rain
//!
//! This module provides a SHAPE INTENSITY MAP that modifies the desktop's
//! existing Matrix rain colors. It does NOT render anything itself.
//!
//! The desktop rain continues normally - this just provides a 2D map of
//! color modifiers based on where a 3D shape intersects each cell.

use alloc::vec::Vec;
use alloc::vec;
use alloc::boxed::Box;

// ═══════════════════════════════════════════════════════════════════════════════
// CONSTANTS - Must match desktop
// ═══════════════════════════════════════════════════════════════════════════════

pub const MATRIX_COLS: usize = 240;
pub const MATRIX_ROWS: usize = 68;

/// Depth layers for 3D calculations
pub const VOL_DEPTH: usize = 16;

/// Edge thickness for outline detection
pub const EDGE_THRESHOLD: f32 = 0.12;

// ═══════════════════════════════════════════════════════════════════════════════
// COLOR MODIFIER - What to do with rain color at each cell
// ═══════════════════════════════════════════════════════════════════════════════

/// Color modification for a cell
#[derive(Clone, Copy, PartialEq)]
pub enum ColorMod {
    /// Normal rain - no modification
    Normal,
    /// On shape edge - use pure green (0xFF00FF00)  
    Edge,
    /// Inside shape - gradient value (0.0 = black center, 1.0 = near edge)
    Inside(f32),
}

// ═══════════════════════════════════════════════════════════════════════════════
// SHAPES & MODES
// ═══════════════════════════════════════════════════════════════════════════════

#[derive(Clone, Copy, PartialEq)]
pub enum Shape3D {
    None,
    Cube,
    Sphere,
    Torus,
    DnaHelix,
}

#[derive(Clone, Copy, PartialEq)]
pub enum RenderMode {
    Hologram,
    MatrixRain,
    DnaHelix,
    RotatingCube,
    Sphere,
}

// ═══════════════════════════════════════════════════════════════════════════════
// HOLOVOLUME - Provides intensity map, does NOT render
// ═══════════════════════════════════════════════════════════════════════════════

pub struct HoloVolume {
    pub width: usize,
    pub height: usize,
    pub depth: usize,
    
    /// Current shape
    pub shape: Shape3D,
    
    /// Render mode
    pub render_mode: RenderMode,
    
    /// Animation time
    time: f32,
    
    /// Shape rotation
    rotation: f32,
    
    /// Pre-computed intensity map for current frame
    /// This is what the desktop reads to modify rain colors
    /// Vec for heap allocation without stack overflow
    intensity_map: Vec<ColorMod>,
    
    /// Screen dimensions (for coordinate mapping)
    screen_width: usize,
    screen_height: usize,
}

impl HoloVolume {
    pub fn new(width: usize, height: usize, depth: usize) -> Self {
        // Allocate intensity map on heap using Vec (no stack overflow)
        // Flat array: index = col * MATRIX_ROWS + row
        let intensity_map = vec![ColorMod::Normal; MATRIX_COLS * MATRIX_ROWS];
        Self {
            width,
            height,
            depth,
            shape: Shape3D::Cube,
            render_mode: RenderMode::Hologram,
            time: 0.0,
            rotation: 0.0,
            intensity_map,
            screen_width: 1920,
            screen_height: 1080,
        }
    }
    
    /// Helper to convert 2D coords to flat index
    #[inline]
    fn idx(col: usize, row: usize) -> usize {
        col * MATRIX_ROWS + row
    }
    
    pub fn set_shape(&mut self, shape: Shape3D) {
        self.shape = shape;
    }
    
    /// Set screen dimensions for proper coordinate mapping
    pub fn set_screen_size(&mut self, width: usize, height: usize) {
        self.screen_width = width;
        self.screen_height = height;
    }
    
    /// Update animation and recompute intensity map
    pub fn update(&mut self, dt: f32) {
        self.time += dt;
        self.rotation += dt * 0.5;
        
        // Determine effective shape
        let effective_shape = match self.render_mode {
            RenderMode::DnaHelix => Shape3D::DnaHelix,
            RenderMode::RotatingCube => Shape3D::Cube,
            RenderMode::Sphere => Shape3D::Sphere,
            RenderMode::MatrixRain => Shape3D::None,
            RenderMode::Hologram => self.shape,
        };
        
        // Recompute intensity map
        self.compute_intensity_map(effective_shape);
    }
    
    /// Compute the intensity map for current frame
    fn compute_intensity_map(&mut self, shape: Shape3D) {
        if shape == Shape3D::None {
            // No shape - all normal
            for col in 0..MATRIX_COLS {
                for row in 0..MATRIX_ROWS {
                    self.intensity_map[Self::idx(col, row)] = ColorMod::Normal;
                }
            }
            return;
        }
        
        let cell_w = 8.0;  // CELL_WIDTH
        let cell_h = 16.0; // CELL_HEIGHT
        
        // Shape center and size
        let cx = self.screen_width as f32 / 2.0;
        let cy = self.screen_height as f32 / 2.0;
        let shape_radius = (self.screen_width.min(self.screen_height) as f32) * 0.25;
        
        for col in 0..MATRIX_COLS {
            let x = col as f32 * cell_w + cell_w / 2.0;
            
            for row in 0..MATRIX_ROWS {
                let y = row as f32 * cell_h + cell_h / 2.0;
                
                // Normalized coords around shape center
                let nx = (x - cx) / shape_radius;
                let ny = (y - cy) / shape_radius;
                
                // Find min SDF across all Z layers
                let mut min_sdf = 999.0f32;
                let mut any_inside = false;
                
                for z_layer in 0..VOL_DEPTH {
                    let nz = (z_layer as f32 / VOL_DEPTH as f32) * 2.0 - 1.0;
                    let sdf = self.shape_sdf(nx, ny, nz, shape);
                    
                    if sdf < min_sdf {
                        min_sdf = sdf;
                    }
                    if sdf < 0.0 {
                        any_inside = true;
                    }
                }
                
                // Determine color modifier
                self.intensity_map[Self::idx(col, row)] = if min_sdf.abs() < EDGE_THRESHOLD {
                    // ON EDGE
                    ColorMod::Edge
                } else if any_inside {
                    // INSIDE - gradient based on distance from edge
                    let edge_dist = -min_sdf;
                    let max_depth = 0.8;
                    let gradient = if edge_dist < max_depth {
                        1.0 - (edge_dist / max_depth)
                    } else {
                        0.0
                    };
                    ColorMod::Inside(gradient)
                } else {
                    // OUTSIDE
                    ColorMod::Normal
                };
            }
        }
    }
    
    /// Generate u8 intensity map for parallel renderer
    /// Format: 0=normal, 1=edge, 2-255=inside gradient (2=near edge, 255=center black)
    /// Returns Vec to avoid stack overflow
    pub fn get_u8_intensity_map(&self) -> Vec<u8> {
        let mut result = vec![0u8; MATRIX_COLS * MATRIX_ROWS];
        
        for col in 0..MATRIX_COLS {
            for row in 0..MATRIX_ROWS {
                let idx = Self::idx(col, row);
                result[idx] = match self.intensity_map[idx] {
                    ColorMod::Normal => 0,
                    ColorMod::Edge => 1,
                    ColorMod::Inside(gradient) => {
                        // gradient: 1.0 = near edge, 0.0 = center
                        // Convert to: 2 = near edge, 255 = center
                        let inverted = 1.0 - gradient;
                        (2.0 + inverted * 253.0) as u8
                    }
                };
            }
        }
        
        result
    }
    
    /// Get the color modifier for a specific cell
    /// The desktop rain uses this to modify its colors
    #[inline]
    pub fn get_color_mod(&self, col: usize, row: usize) -> ColorMod {
        if col < MATRIX_COLS && row < MATRIX_ROWS {
            self.intensity_map[Self::idx(col, row)]
        } else {
            ColorMod::Normal
        }
    }
    
    /// Check if shape is active (so desktop knows whether to apply modifiers)
    pub fn has_active_shape(&self) -> bool {
        let effective_shape = match self.render_mode {
            RenderMode::DnaHelix => Shape3D::DnaHelix,
            RenderMode::RotatingCube => Shape3D::Cube,
            RenderMode::Sphere => Shape3D::Sphere,
            RenderMode::MatrixRain => Shape3D::None,
            RenderMode::Hologram => self.shape,
        };
        effective_shape != Shape3D::None
    }
    
    /// Apply color modifier to a base green value
    /// Returns the final color (ARGB format)
    #[inline]
    pub fn apply_color_mod(&self, base_green: u32, col: usize, row: usize) -> u32 {
        match self.get_color_mod(col, row) {
            ColorMod::Normal => {
                // Normal rain - keep color as is (but never pure green)
                let g = base_green.min(238);
                0xFF000000 | (g << 8)
            }
            ColorMod::Edge => {
                // Edge - PURE GREEN (only place it's used!)
                0xFF00FF00
            }
            ColorMod::Inside(gradient) => {
                // Inside - gradient from edge (bright) to center (black)
                let g = ((base_green as f32) * gradient * 0.7) as u32;
                if g < 10 {
                    0xFF000000 // Too dark, just black
                } else {
                    0xFF000000 | (g << 8)
                }
            }
        }
    }
    
    /// Legacy render function - now just clears to black
    /// The desktop's normal rain will be rendered, then modified
    pub fn render_to_buffer(&self, buffer: &mut [u32], _screen_width: usize, _screen_height: usize) {
        // Don't render anything - just ensure buffer is black
        // The desktop will render its own rain
        buffer.fill(0xFF000000);
    }
    
    /// Signed distance function for shapes
    fn shape_sdf(&self, x: f32, y: f32, z: f32, shape: Shape3D) -> f32 {
        // Apply rotation around Y axis
        let cos_r = libm::cosf(self.rotation);
        let sin_r = libm::sinf(self.rotation);
        let rx = x * cos_r - z * sin_r;
        let rz = x * sin_r + z * cos_r;
        let ry = y;
        
        match shape {
            Shape3D::Cube => {
                let size = 0.7;
                let dx = libm::fabsf(rx) - size;
                let dy = libm::fabsf(ry) - size;
                let dz = libm::fabsf(rz) - size;
                let mx = libm::fmaxf(dx, 0.0);
                let my = libm::fmaxf(dy, 0.0);
                let mz = libm::fmaxf(dz, 0.0);
                let outside = libm::sqrtf(mx * mx + my * my + mz * mz);
                let inside = libm::fminf(libm::fmaxf(dx, libm::fmaxf(dy, dz)), 0.0);
                outside + inside
            }
            
            Shape3D::Sphere => {
                let radius = 0.8;
                libm::sqrtf(rx * rx + ry * ry + rz * rz) - radius
            }
            
            Shape3D::Torus => {
                let major = 0.6;
                let minor = 0.25;
                let q = libm::sqrtf(rx * rx + rz * rz) - major;
                libm::sqrtf(q * q + ry * ry) - minor
            }
            
            Shape3D::DnaHelix => {
                let helix_radius = 0.4;
                let helix_pitch = 2.0;
                let strand_radius = 0.15;
                
                let angle = ry * helix_pitch + self.time * 2.0;
                
                let s1x = helix_radius * libm::cosf(angle);
                let s1z = helix_radius * libm::sinf(angle);
                let dx1 = rx - s1x;
                let dz1 = rz - s1z;
                let d1 = libm::sqrtf(dx1 * dx1 + dz1 * dz1) - strand_radius;
                
                let s2x = helix_radius * libm::cosf(angle + core::f32::consts::PI);
                let s2z = helix_radius * libm::sinf(angle + core::f32::consts::PI);
                let dx2 = rx - s2x;
                let dz2 = rz - s2z;
                let d2 = libm::sqrtf(dx2 * dx2 + dz2 * dz2) - strand_radius;
                
                libm::fminf(d1, d2)
            }
            
            Shape3D::None => 999.0,
        }
    }
}
