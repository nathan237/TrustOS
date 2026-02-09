//! HoloMatrix - Volumetric 3D Rendering via Matrix Layer Superposition
//!
//! Creates holographic 3D effects by rendering shapes across multiple
//! depth layers (Z-slices) and compositing them with depth-based intensity.
//!
//! Architecture:
//! - VolumetricMatrix: 3D grid of voxels organized as Z-ordered 2D slices
//! - Each slice is a 2D matrix with intensity values
//! - Slices composite with depth fog/glow effects
//! - Shapes are projected onto each slice at their Z-intersection

use alloc::vec::Vec;
use alloc::vec;
use spin::Mutex;
use core::sync::atomic::{AtomicU8, AtomicBool, Ordering};

// ═══════════════════════════════════════════════════════════════════════════════
// GLOBAL HOLOMATRIX SETTINGS (accessible from shell commands)
// ═══════════════════════════════════════════════════════════════════════════════

/// Global HoloMatrix enabled state (default: enabled for DNA helix)
static HOLO_ENABLED: AtomicBool = AtomicBool::new(true);

/// Global HoloMatrix scene index (0-5) - default to DNA helix (5)
static HOLO_SCENE: AtomicU8 = AtomicU8::new(5);

/// Check if HoloMatrix is enabled
pub fn is_enabled() -> bool {
    HOLO_ENABLED.load(Ordering::Relaxed)
}

/// Enable/disable HoloMatrix
pub fn set_enabled(enabled: bool) {
    HOLO_ENABLED.store(enabled, Ordering::Relaxed);
    crate::serial_println!("[HOLO] HoloMatrix: {}", if enabled { "ENABLED" } else { "DISABLED" });
}

/// Toggle HoloMatrix on/off
pub fn toggle() -> bool {
    let current = HOLO_ENABLED.load(Ordering::Relaxed);
    HOLO_ENABLED.store(!current, Ordering::Relaxed);
    crate::serial_println!("[HOLO] HoloMatrix: {}", if !current { "ENABLED" } else { "DISABLED" });
    !current
}

/// Get current scene
pub fn get_scene() -> HoloScene {
    HoloScene::from_index(HOLO_SCENE.load(Ordering::Relaxed))
}

/// Set current scene
pub fn set_scene(scene: HoloScene) {
    HOLO_SCENE.store(scene.to_index(), Ordering::Relaxed);
    crate::serial_println!("[HOLO] Scene: {}", scene.name());
}

/// Cycle to next scene
pub fn next_scene() -> HoloScene {
    let current = get_scene();
    let next = current.next();
    set_scene(next);
    next
}

/// Volumetric matrix for holographic rendering
/// Organized as NUM_LAYERS 2D slices at different Z depths
pub struct HoloMatrix {
    /// Width of each slice
    pub width: usize,
    /// Height of each slice
    pub height: usize,
    /// Number of Z layers
    pub num_layers: usize,
    /// Data: layers[z][y * width + x] = intensity (0-255)
    pub layers: Vec<Vec<u8>>,
    /// Z depth of each layer (0.0 = near, 1.0 = far)
    pub layer_depths: Vec<f32>,
    /// Rotation angles for animated objects
    pub rotation_x: f32,
    pub rotation_y: f32,
    pub rotation_z: f32,
    /// Animation time
    pub time: f32,
}

/// 3D point
#[derive(Clone, Copy)]
pub struct Point3D {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Point3D {
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }
    
    /// Rotate around X axis
    pub fn rotate_x(self, angle: f32) -> Self {
        let cos_a = cos_approx(angle);
        let sin_a = sin_approx(angle);
        Self {
            x: self.x,
            y: self.y * cos_a - self.z * sin_a,
            z: self.y * sin_a + self.z * cos_a,
        }
    }
    
    /// Rotate around Y axis
    pub fn rotate_y(self, angle: f32) -> Self {
        let cos_a = cos_approx(angle);
        let sin_a = sin_approx(angle);
        Self {
            x: self.x * cos_a + self.z * sin_a,
            y: self.y,
            z: -self.x * sin_a + self.z * cos_a,
        }
    }
    
    /// Rotate around Z axis
    pub fn rotate_z(self, angle: f32) -> Self {
        let cos_a = cos_approx(angle);
        let sin_a = sin_approx(angle);
        Self {
            x: self.x * cos_a - self.y * sin_a,
            y: self.x * sin_a + self.y * cos_a,
            z: self.z,
        }
    }
}

/// Fast approximate sine (internal)
fn sin_approx(x: f32) -> f32 {
    sin_approx_pub(x)
}

/// Fast approximate cosine (internal)
fn cos_approx(x: f32) -> f32 {
    cos_approx_pub(x)
}

/// Fast approximate sine (public for shell.rs)
#[inline]
pub fn sin_approx_pub(x: f32) -> f32 {
    // Normalize to -PI..PI
    let mut x = x % (2.0 * 3.14159);
    if x > 3.14159 { x -= 2.0 * 3.14159; }
    if x < -3.14159 { x += 2.0 * 3.14159; }
    
    // Parabolic approximation
    let abs_x = if x < 0.0 { -x } else { x };
    let sign = if x < 0.0 { -1.0 } else { 1.0 };
    
    // sin(x) ≈ (4/π)x - (4/π²)x²  for 0 <= x <= π
    let y = 1.27323954 * abs_x - 0.405284735 * abs_x * abs_x;
    
    sign * y
}

/// Fast approximate cosine (public for shell.rs)
#[inline]
pub fn cos_approx_pub(x: f32) -> f32 {
    sin_approx_pub(x + 1.5707963) // cos(x) = sin(x + π/2)
}

/// Fast approximate square root
fn sqrt_approx(x: f32) -> f32 {
    if x <= 0.0 { return 0.0; }
    let mut guess = x / 2.0;
    for _ in 0..4 {
        guess = (guess + x / guess) / 2.0;
    }
    guess
}

impl HoloMatrix {
    /// Create a new holographic matrix
    pub fn new(width: usize, height: usize, num_layers: usize) -> Self {
        let mut layers = Vec::with_capacity(num_layers);
        let mut layer_depths = Vec::with_capacity(num_layers);
        
        for i in 0..num_layers {
            layers.push(vec![0u8; width * height]);
            // Linear depth distribution from 0 (near) to 1 (far)
            layer_depths.push(i as f32 / (num_layers - 1) as f32);
        }
        
        Self {
            width,
            height,
            num_layers,
            layers,
            layer_depths,
            rotation_x: 0.0,
            rotation_y: 0.0,
            rotation_z: 0.0,
            time: 0.0,
        }
    }
    
    /// Clear all layers
    pub fn clear(&mut self) {
        for layer in &mut self.layers {
            layer.fill(0);
        }
    }
    
    /// Set a point on a specific layer
    #[inline]
    pub fn set_point(&mut self, layer: usize, x: i32, y: i32, intensity: u8) {
        if layer < self.num_layers 
            && x >= 0 && (x as usize) < self.width 
            && y >= 0 && (y as usize) < self.height 
        {
            let idx = y as usize * self.width + x as usize;
            // Additive blend
            let current = self.layers[layer][idx] as u16;
            self.layers[layer][idx] = (current + intensity as u16).min(255) as u8;
        }
    }
    
    /// Draw a 3D sphere distributed across layers
    pub fn draw_sphere(&mut self, cx: f32, cy: f32, cz: f32, radius: f32, intensity: u8) {
        let screen_cx = self.width as f32 / 2.0;
        let screen_cy = self.height as f32 / 2.0;
        
        for layer_idx in 0..self.num_layers {
            let layer_z = self.layer_depths[layer_idx];
            
            // Distance from sphere center to this Z plane
            let dz = layer_z - cz;
            
            // Check if this plane intersects the sphere
            if dz.abs() < radius {
                // Radius of intersection circle at this Z
                let circle_radius = sqrt_approx(radius * radius - dz * dz);
                
                // Draw circle on this layer
                let circle_px = (circle_radius * self.width as f32 / 2.0) as i32;
                let pcx = (cx * self.width as f32 / 2.0 + screen_cx) as i32;
                let pcy = (cy * self.height as f32 / 2.0 + screen_cy) as i32;
                
                // Intensity decreases with distance from center
                let depth_factor = 1.0 - dz.abs() / radius;
                let layer_intensity = (intensity as f32 * depth_factor) as u8;
                
                self.draw_circle_layer(layer_idx, pcx, pcy, circle_px, layer_intensity);
            }
        }
    }
    
    /// Draw a circle on a specific layer
    fn draw_circle_layer(&mut self, layer: usize, cx: i32, cy: i32, radius: i32, intensity: u8) {
        let r_sq = radius * radius;
        
        // Draw filled circle with edge glow
        for dy in -radius..=radius {
            let dx_max = sqrt_approx((r_sq - dy * dy) as f32) as i32;
            for dx in -dx_max..=dx_max {
                let dist_sq = dx * dx + dy * dy;
                let dist_ratio = sqrt_approx(dist_sq as f32) / radius as f32;
                
                // Edge glow effect - brighter at edges
                let edge_factor = if dist_ratio > 0.7 {
                    1.0 + (dist_ratio - 0.7) * 2.0
                } else {
                    0.5 + dist_ratio * 0.5
                };
                
                let px_intensity = (intensity as f32 * edge_factor).min(255.0) as u8;
                self.set_point(layer, cx + dx, cy + dy, px_intensity);
            }
        }
    }
    
    /// Draw a 3D wireframe cube
    pub fn draw_cube(&mut self, cx: f32, cy: f32, cz: f32, size: f32, intensity: u8) {
        let half = size / 2.0;
        
        // 8 vertices of the cube
        let vertices = [
            Point3D::new(-half, -half, -half),
            Point3D::new( half, -half, -half),
            Point3D::new( half,  half, -half),
            Point3D::new(-half,  half, -half),
            Point3D::new(-half, -half,  half),
            Point3D::new( half, -half,  half),
            Point3D::new( half,  half,  half),
            Point3D::new(-half,  half,  half),
        ];
        
        // Rotate vertices
        let rotated: Vec<Point3D> = vertices.iter().map(|v| {
            v.rotate_x(self.rotation_x)
             .rotate_y(self.rotation_y)
             .rotate_z(self.rotation_z)
        }).collect();
        
        // Translate to center
        let translated: Vec<Point3D> = rotated.iter().map(|v| {
            Point3D::new(v.x + cx, v.y + cy, v.z + cz)
        }).collect();
        
        // 12 edges of the cube
        let edges = [
            (0, 1), (1, 2), (2, 3), (3, 0), // Front face
            (4, 5), (5, 6), (6, 7), (7, 4), // Back face
            (0, 4), (1, 5), (2, 6), (3, 7), // Connecting edges
        ];
        
        // Draw each edge across layers
        for (i1, i2) in &edges {
            self.draw_line_3d(&translated[*i1], &translated[*i2], intensity);
        }
    }
    
    /// Draw a 3D line distributed across layers
    pub fn draw_line_3d(&mut self, p1: &Point3D, p2: &Point3D, intensity: u8) {
        let screen_cx = self.width as f32 / 2.0;
        let screen_cy = self.height as f32 / 2.0;
        
        // Number of points along the line
        let dx = p2.x - p1.x;
        let dy = p2.y - p1.y;
        let dz = p2.z - p1.z;
        let length = sqrt_approx(dx * dx + dy * dy + dz * dz);
        let steps = (length * 50.0) as usize + 1;
        
        for step in 0..=steps {
            let t = step as f32 / steps as f32;
            let px = p1.x + dx * t;
            let py = p1.y + dy * t;
            let pz = p1.z + dz * t;
            
            // Find closest layer
            let layer_z = (pz + 1.0) / 2.0; // Normalize -1..1 to 0..1
            if layer_z >= 0.0 && layer_z <= 1.0 {
                let layer_idx = ((layer_z * (self.num_layers - 1) as f32) as usize).min(self.num_layers - 1);
                
                // Project to screen
                let sx = (px * self.width as f32 / 2.5 + screen_cx) as i32;
                let sy = (py * self.height as f32 / 2.5 + screen_cy) as i32;
                
                // Intensity based on Z (closer = brighter)
                let depth_intensity = ((1.0 - layer_z * 0.5) * intensity as f32) as u8;
                
                // Draw a small cross for visibility
                for d in -1..=1 {
                    self.set_point(layer_idx, sx + d, sy, depth_intensity);
                    self.set_point(layer_idx, sx, sy + d, depth_intensity);
                }
            }
        }
    }
    
    /// Draw a wireframe torus (donut)
    pub fn draw_torus(&mut self, cx: f32, cy: f32, cz: f32, major_r: f32, minor_r: f32, intensity: u8) {
        let segments_major = 24;
        let segments_minor = 12;
        
        for i in 0..segments_major {
            let theta1 = (i as f32 / segments_major as f32) * 2.0 * 3.14159;
            let theta2 = ((i + 1) as f32 / segments_major as f32) * 2.0 * 3.14159;
            
            for j in 0..segments_minor {
                let phi1 = (j as f32 / segments_minor as f32) * 2.0 * 3.14159;
                let phi2 = ((j + 1) as f32 / segments_minor as f32) * 2.0 * 3.14159;
                
                // Calculate torus points
                let p1 = self.torus_point(cx, cy, cz, major_r, minor_r, theta1, phi1);
                let p2 = self.torus_point(cx, cy, cz, major_r, minor_r, theta1, phi2);
                let p3 = self.torus_point(cx, cy, cz, major_r, minor_r, theta2, phi1);
                
                // Draw lines
                self.draw_line_3d(&p1, &p2, intensity / 2);
                self.draw_line_3d(&p1, &p3, intensity / 2);
            }
        }
    }
    
    fn torus_point(&self, cx: f32, cy: f32, cz: f32, major_r: f32, minor_r: f32, theta: f32, phi: f32) -> Point3D {
        let x = (major_r + minor_r * cos_approx(phi)) * cos_approx(theta);
        let y = (major_r + minor_r * cos_approx(phi)) * sin_approx(theta);
        let z = minor_r * sin_approx(phi);
        
        // Apply rotation
        let p = Point3D::new(x, y, z)
            .rotate_x(self.rotation_x)
            .rotate_y(self.rotation_y);
            
        Point3D::new(p.x + cx, p.y + cy, p.z + cz)
    }
    
    /// Draw a grid plane (floor-like)
    pub fn draw_grid(&mut self, y_level: f32, grid_size: f32, cells: i32, intensity: u8) {
        let half = grid_size / 2.0;
        let cell_size = grid_size / cells as f32;
        
        // Horizontal lines
        for i in 0..=cells {
            let z = -half + i as f32 * cell_size;
            let p1 = Point3D::new(-half, y_level, z)
                .rotate_y(self.rotation_y);
            let p2 = Point3D::new(half, y_level, z)
                .rotate_y(self.rotation_y);
            self.draw_line_3d(&p1, &p2, intensity);
        }
        
        // Vertical lines
        for i in 0..=cells {
            let x = -half + i as f32 * cell_size;
            let p1 = Point3D::new(x, y_level, -half)
                .rotate_y(self.rotation_y);
            let p2 = Point3D::new(x, y_level, half)
                .rotate_y(self.rotation_y);
            self.draw_line_3d(&p1, &p2, intensity);
        }
    }
    
    /// Composite all layers into a single ARGB buffer
    /// Returns pixel buffer ready for framebuffer
    pub fn composite(&self, base_color: u32, glow_color: u32) -> Vec<u32> {
        let mut output = vec![base_color; self.width * self.height];
        
        // Extract glow color components
        let gr = ((glow_color >> 16) & 0xFF) as u32;
        let gg = ((glow_color >> 8) & 0xFF) as u32;
        let gb = (glow_color & 0xFF) as u32;
        
        // Composite layers from back to front with depth effects
        for (layer_idx, layer) in self.layers.iter().enumerate().rev() {
            let depth = self.layer_depths[layer_idx];
            
            // Depth fog: further layers are more transparent
            let layer_opacity = 0.3 + 0.7 * (1.0 - depth);
            
            for y in 0..self.height {
                for x in 0..self.width {
                    let intensity = layer[y * self.width + x];
                    
                    if intensity > 0 {
                        let pixel_idx = y * self.width + x;
                        let current = output[pixel_idx];
                        
                        // Get current RGB
                        let cr = ((current >> 16) & 0xFF) as u32;
                        let cg = ((current >> 8) & 0xFF) as u32;
                        let cb = (current & 0xFF) as u32;
                        
                        // Blend with glow color
                        let alpha = (intensity as f32 / 255.0) * layer_opacity;
                        let nr = (cr as f32 * (1.0 - alpha) + gr as f32 * alpha) as u32;
                        let ng = (cg as f32 * (1.0 - alpha) + gg as f32 * alpha) as u32;
                        let nb = (cb as f32 * (1.0 - alpha) + gb as f32 * alpha) as u32;
                        
                        output[pixel_idx] = 0xFF000000 | (nr.min(255) << 16) | (ng.min(255) << 8) | nb.min(255);
                    }
                }
            }
        }
        
        output
    }
    
    /// Update animation
    pub fn update(&mut self, delta_time: f32) {
        self.time += delta_time;
        self.rotation_y += delta_time * 0.5;  // Slow Y rotation
        self.rotation_x = sin_approx(self.time * 0.3) * 0.3;  // Subtle wobble
    }
    
    /// Render a complete holographic scene
    pub fn render_scene(&mut self, scene: HoloScene) {
        self.clear();
        
        match scene {
            HoloScene::RotatingCube => {
                self.draw_cube(0.0, 0.0, 0.5, 0.6, 200);
            },
            HoloScene::SpherePulse => {
                let pulse = (sin_approx(self.time * 2.0) + 1.0) / 2.0;
                let radius = 0.2 + pulse * 0.15;
                self.draw_sphere(0.0, 0.0, 0.5, radius, 180);
            },
            HoloScene::Torus => {
                self.draw_torus(0.0, 0.0, 0.5, 0.35, 0.12, 150);
            },
            HoloScene::GridWithCube => {
                self.draw_grid(-0.4, 1.5, 8, 60);
                self.draw_cube(0.0, 0.0, 0.5, 0.4, 200);
            },
            HoloScene::MultiShape => {
                // Multiple floating shapes
                self.draw_cube(-0.4, 0.0, 0.5, 0.25, 150);
                self.draw_sphere(0.4, 0.0, 0.5, 0.2, 180);
                self.draw_torus(0.0, 0.3, 0.5, 0.2, 0.08, 120);
            },
            HoloScene::DNA => {
                // DNA helix
                self.draw_dna_helix(0.0, 0.0, 0.5, 100);
            },
            HoloScene::RayTracedSpheres | HoloScene::RayTracedDNA => {
                // RayTraced scenes are rendered by the RayTracer directly
                // Show a simple placeholder if HoloMatrix is asked to render them
                self.draw_sphere(0.0, 0.0, 0.5, 0.3, 150);
            },
        }
    }
    
    /// Draw an enhanced DNA double helix with nucleotide base pairs
    fn draw_dna_helix(&mut self, cx: f32, cy: f32, cz: f32, intensity: u8) {
        let helix_length = 1.4;  // Taller helix
        let radius = 0.32;       // Wider radius
        let turns = 3.5;         // More turns
        let segments = 100;      // Smoother curves
        
        // Draw backbone phosphate groups on both strands
        for i in 0..segments {
            let t = i as f32 / segments as f32;
            let y = -helix_length / 2.0 + t * helix_length;
            let angle = t * turns * 2.0 * 3.14159 + self.time;
            
            // First strand phosphate backbone
            let x1 = radius * cos_approx(angle);
            let z1 = radius * sin_approx(angle);
            
            // Second strand phosphate backbone (180° offset)
            let x2 = radius * cos_approx(angle + 3.14159);
            let z2 = radius * sin_approx(angle + 3.14159);
            
            // Apply rotation for 3D effect
            let p1_raw = Point3D::new(x1, y, z1 * 0.5)
                .rotate_x(self.rotation_x * 0.5)
                .rotate_y(self.rotation_y * 0.3);
            let p2_raw = Point3D::new(x2, y, z2 * 0.5)
                .rotate_x(self.rotation_x * 0.5)
                .rotate_y(self.rotation_y * 0.3);
            
            let p1 = Point3D::new(p1_raw.x + cx, p1_raw.y + cy, p1_raw.z + cz);
            let p2 = Point3D::new(p2_raw.x + cx, p2_raw.y + cy, p2_raw.z + cz);
            
            // Draw backbone segments with thicker lines
            if i < segments - 1 {
                let t2 = (i + 1) as f32 / segments as f32;
                let y2 = -helix_length / 2.0 + t2 * helix_length;
                let angle2 = t2 * turns * 2.0 * 3.14159 + self.time;
                
                let p1_next_raw = Point3D::new(radius * cos_approx(angle2), y2, radius * sin_approx(angle2) * 0.5)
                    .rotate_x(self.rotation_x * 0.5)
                    .rotate_y(self.rotation_y * 0.3);
                let p2_next_raw = Point3D::new(radius * cos_approx(angle2 + 3.14159), y2, radius * sin_approx(angle2 + 3.14159) * 0.5)
                    .rotate_x(self.rotation_x * 0.5)
                    .rotate_y(self.rotation_y * 0.3);
                
                let p1_next = Point3D::new(p1_next_raw.x + cx, p1_next_raw.y + cy, p1_next_raw.z + cz);
                let p2_next = Point3D::new(p2_next_raw.x + cx, p2_next_raw.y + cy, p2_next_raw.z + cz);
                
                // Draw thick backbone (multiple parallel lines)
                self.draw_line_3d(&p1, &p1_next, intensity);
                self.draw_line_3d(&p2, &p2_next, intensity);
            }
            
            // Draw phosphate "bumps" on backbone every 10 segments
            if i % 10 == 0 {
                self.draw_phosphate_group(p1.x, p1.y, p1.z, intensity);
                self.draw_phosphate_group(p2.x, p2.y, p2.z, intensity);
            }
            
            // Draw base pairs connecting the two strands
            if i % 4 == 0 {
                // Base pair with hydrogen bonds (A-T or G-C)
                self.draw_base_pair(&p1, &p2, intensity, i % 8 == 0);
            }
        }
        
        // Add floating nucleotide particles for ambiance
        self.draw_floating_particles(cx, cy, cz, intensity / 2);
    }
    
    /// Draw a phosphate group (small sphere-like bump)
    fn draw_phosphate_group(&mut self, x: f32, y: f32, z: f32, intensity: u8) {
        // Draw a small bright spot for phosphate
        self.draw_sphere(x, y, z, 0.03, intensity);
    }
    
    /// Draw a base pair with hydrogen bonds
    fn draw_base_pair(&mut self, p1: &Point3D, p2: &Point3D, intensity: u8, is_gc: bool) {
        // Calculate center and intermediate points
        let cx = (p1.x + p2.x) / 2.0;
        let cy = (p1.y + p2.y) / 2.0;
        let cz = (p1.z + p2.z) / 2.0;
        
        // G-C has 3 hydrogen bonds, A-T has 2
        if is_gc {
            // Three hydrogen bonds for G-C pair
            let third_x = (p2.x - p1.x) / 3.0;
            let third_y = (p2.y - p1.y) / 3.0;
            let third_z = (p2.z - p1.z) / 3.0;
            
            // Bond 1 (near p1)
            self.draw_line_3d(
                &Point3D::new(p1.x + third_x * 0.3, p1.y + third_y * 0.3, p1.z + third_z * 0.3),
                &Point3D::new(p1.x + third_x * 0.7, p1.y + third_y * 0.7, p1.z + third_z * 0.7),
                intensity / 2
            );
            // Bond 2 (center)
            self.draw_line_3d(
                &Point3D::new(cx - third_x * 0.2, cy - third_y * 0.2, cz - third_z * 0.2),
                &Point3D::new(cx + third_x * 0.2, cy + third_y * 0.2, cz + third_z * 0.2),
                intensity / 2
            );
            // Bond 3 (near p2)
            self.draw_line_3d(
                &Point3D::new(p2.x - third_x * 0.7, p2.y - third_y * 0.7, p2.z - third_z * 0.7),
                &Point3D::new(p2.x - third_x * 0.3, p2.y - third_y * 0.3, p2.z - third_z * 0.3),
                intensity / 2
            );
        } else {
            // Two hydrogen bonds for A-T pair
            let quarter_x = (p2.x - p1.x) / 4.0;
            let quarter_y = (p2.y - p1.y) / 4.0;
            let quarter_z = (p2.z - p1.z) / 4.0;
            
            self.draw_line_3d(
                &Point3D::new(p1.x + quarter_x * 1.2, p1.y + quarter_y * 1.2, p1.z + quarter_z * 1.2),
                &Point3D::new(p1.x + quarter_x * 1.8, p1.y + quarter_y * 1.8, p1.z + quarter_z * 1.8),
                intensity / 2
            );
            self.draw_line_3d(
                &Point3D::new(p2.x - quarter_x * 1.8, p2.y - quarter_y * 1.8, p2.z - quarter_z * 1.8),
                &Point3D::new(p2.x - quarter_x * 1.2, p2.y - quarter_y * 1.2, p2.z - quarter_z * 1.2),
                intensity / 2
            );
        }
        
        // Draw the actual bases as small markers at ends
        self.draw_line_3d(p1, &Point3D::new(cx, cy, cz), intensity / 3);
        self.draw_line_3d(&Point3D::new(cx, cy, cz), p2, intensity / 3);
    }
    
    /// Draw floating particles for ambiance
    fn draw_floating_particles(&mut self, cx: f32, cy: f32, cz: f32, intensity: u8) {
        // 8 floating particles orbiting the helix
        for i in 0..8 {
            let angle = (i as f32 / 8.0) * 2.0 * 3.14159 + self.time * 0.7;
            let y_offset = sin_approx(self.time * 1.5 + i as f32 * 0.789) * 0.5;
            let orbit_radius = 0.45 + sin_approx(self.time + i as f32) * 0.1;
            
            let px = cx + orbit_radius * cos_approx(angle);
            let py = cy + y_offset;
            let pz = cz + orbit_radius * sin_approx(angle) * 0.4;
            
            // Pulsing intensity
            let pulse = ((sin_approx(self.time * 2.0 + i as f32 * 1.1) + 1.0) / 2.0 * 0.5 + 0.5) as f32;
            let particle_intensity = (intensity as f32 * pulse) as u8;
            
            self.draw_sphere(px, py, pz, 0.02, particle_intensity);
        }
    }
}

/// Available holographic scenes
#[derive(Clone, Copy, PartialEq)]
pub enum HoloScene {
    RotatingCube,
    SpherePulse,
    Torus,
    GridWithCube,
    MultiShape,
    DNA,
    RayTracedSpheres,  // Ray traced floating spheres
    RayTracedDNA,      // Ray traced DNA helix
}

impl HoloScene {
    /// Get next scene
    pub fn next(self) -> Self {
        match self {
            Self::RotatingCube => Self::SpherePulse,
            Self::SpherePulse => Self::Torus,
            Self::Torus => Self::GridWithCube,
            Self::GridWithCube => Self::MultiShape,
            Self::MultiShape => Self::DNA,
            Self::DNA => Self::RayTracedSpheres,
            Self::RayTracedSpheres => Self::RayTracedDNA,
            Self::RayTracedDNA => Self::RotatingCube,
        }
    }
    
    /// Get scene name
    pub fn name(&self) -> &'static str {
        match self {
            Self::RotatingCube => "Cube",
            Self::SpherePulse => "Sphere",
            Self::Torus => "Torus",
            Self::GridWithCube => "Grid+Cube",
            Self::MultiShape => "Multi",
            Self::DNA => "DNA",
            Self::RayTracedSpheres => "RT-Spheres",
            Self::RayTracedDNA => "RT-DNA",
        }
    }
    
    /// Convert to index
    pub fn to_index(&self) -> u8 {
        match self {
            Self::RotatingCube => 0,
            Self::SpherePulse => 1,
            Self::Torus => 2,
            Self::GridWithCube => 3,
            Self::MultiShape => 4,
            Self::DNA => 5,
            Self::RayTracedSpheres => 6,
            Self::RayTracedDNA => 7,
        }
    }
    
    /// Convert from index
    pub fn from_index(idx: u8) -> Self {
        match idx {
            0 => Self::RotatingCube,
            1 => Self::SpherePulse,
            2 => Self::Torus,
            3 => Self::GridWithCube,
            4 => Self::MultiShape,
            5 => Self::DNA,
            6 => Self::RayTracedSpheres,
            7 => Self::RayTracedDNA,
            _ => Self::RotatingCube,
        }
    }
    
    /// Get scene by name
    pub fn from_name(name: &str) -> Option<Self> {
        match name.to_lowercase().as_str() {
            "cube" | "box" => Some(Self::RotatingCube),
            "sphere" | "ball" => Some(Self::SpherePulse),
            "torus" | "donut" | "ring" => Some(Self::Torus),
            "grid" | "grid+cube" | "gridcube" => Some(Self::GridWithCube),
            "multi" | "multiple" | "shapes" => Some(Self::MultiShape),
            "dna" | "helix" => Some(Self::DNA),
            "rt-spheres" | "rtspheres" | "raytraced" => Some(Self::RayTracedSpheres),
            "rt-dna" | "rtdna" | "raytraced-dna" => Some(Self::RayTracedDNA),
            _ => None,
        }
    }
    
    /// List all scenes
    pub fn all_names() -> &'static [&'static str] {
        &["cube", "sphere", "torus", "grid", "multi", "dna", "rt-spheres", "rt-dna"]
    }
    
    /// Check if this scene uses ray tracing
    pub fn is_raytraced(&self) -> bool {
        matches!(self, Self::RayTracedSpheres | Self::RayTracedDNA)
    }
}
