//! Software 3D Renderer
//!
//! Provides wireframe and filled 3D rendering without GPU acceleration.
//! Supports meshes, lighting, and basic transformations.

use alloc::vec::Vec;
use alloc::boxed::Box;
use micromath::F32Ext;

use super::math3d::{Vec3, Vec4, Mat4, Transform, deg_to_rad};
use super::render2d::{FramebufferTarget, Color2D, Renderer2D};

// ═══════════════════════════════════════════════════════════════════════════════
// VERTEX & TRIANGLE
// ═══════════════════════════════════════════════════════════════════════════════

/// A vertex with position, normal, and UV
#[derive(Clone, Copy, Debug, Default)]
pub struct Vertex {
    pub position: Vec3,
    pub normal: Vec3,
    pub uv: (f32, f32),
    pub color: Color2D,
}

impl Vertex {
    pub fn new(position: Vec3) -> Self {
        Self {
            position,
            normal: Vec3::Y,
            uv: (0.0, 0.0),
            color: Color2D::WHITE,
        }
    }

    pub fn with_normal(mut self, normal: Vec3) -> Self {
        self.normal = normal;
        self
    }

    pub fn with_uv(mut self, u: f32, v: f32) -> Self {
        self.uv = (u, v);
        self
    }

    pub fn with_color(mut self, color: Color2D) -> Self {
        self.color = color;
        self
    }
}

/// Triangle with 3 vertex indices
#[derive(Clone, Copy, Debug)]
pub struct Triangle {
    pub v0: usize,
    pub v1: usize,
    pub v2: usize,
}

impl Triangle {
    pub const fn new(v0: usize, v1: usize, v2: usize) -> Self {
        Self { v0, v1, v2 }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// MESH
// ═══════════════════════════════════════════════════════════════════════════════

/// 3D Mesh containing vertices and triangles
pub struct Mesh {
    pub vertices: Vec<Vertex>,
    pub triangles: Vec<Triangle>,
    pub transform: Transform,
}

impl Mesh {
    pub fn new() -> Self {
        Self {
            vertices: Vec::new(),
            triangles: Vec::new(),
            transform: Transform::IDENTITY,
        }
    }

    /// Create a cube mesh
    pub fn cube(size: f32) -> Self {
        let s = size / 2.0;
        let mut mesh = Self::new();

        // Front face
        mesh.vertices.push(Vertex::new(Vec3::new(-s, -s,  s)).with_normal(Vec3::Z).with_color(Color2D::RED));
        mesh.vertices.push(Vertex::new(Vec3::new( s, -s,  s)).with_normal(Vec3::Z).with_color(Color2D::GREEN));
        mesh.vertices.push(Vertex::new(Vec3::new( s,  s,  s)).with_normal(Vec3::Z).with_color(Color2D::BLUE));
        mesh.vertices.push(Vertex::new(Vec3::new(-s,  s,  s)).with_normal(Vec3::Z).with_color(Color2D::YELLOW));

        // Back face
        mesh.vertices.push(Vertex::new(Vec3::new( s, -s, -s)).with_normal(-Vec3::Z).with_color(Color2D::CYAN));
        mesh.vertices.push(Vertex::new(Vec3::new(-s, -s, -s)).with_normal(-Vec3::Z).with_color(Color2D::MAGENTA));
        mesh.vertices.push(Vertex::new(Vec3::new(-s,  s, -s)).with_normal(-Vec3::Z).with_color(Color2D::ORANGE));
        mesh.vertices.push(Vertex::new(Vec3::new( s,  s, -s)).with_normal(-Vec3::Z).with_color(Color2D::PURPLE));

        // Triangles (2 per face, 6 faces = 12 triangles)
        // Front
        mesh.triangles.push(Triangle::new(0, 1, 2));
        mesh.triangles.push(Triangle::new(0, 2, 3));
        // Back
        mesh.triangles.push(Triangle::new(4, 5, 6));
        mesh.triangles.push(Triangle::new(4, 6, 7));
        // Top
        mesh.triangles.push(Triangle::new(3, 2, 7));
        mesh.triangles.push(Triangle::new(3, 7, 6));
        // Bottom
        mesh.triangles.push(Triangle::new(5, 4, 1));
        mesh.triangles.push(Triangle::new(5, 1, 0));
        // Right
        mesh.triangles.push(Triangle::new(1, 4, 7));
        mesh.triangles.push(Triangle::new(1, 7, 2));
        // Left
        mesh.triangles.push(Triangle::new(5, 0, 3));
        mesh.triangles.push(Triangle::new(5, 3, 6));

        mesh
    }

    /// Create a pyramid mesh
    pub fn pyramid(base: f32, height: f32) -> Self {
        let s = base / 2.0;
        let mut mesh = Self::new();

        // Base vertices
        mesh.vertices.push(Vertex::new(Vec3::new(-s, 0.0, -s)).with_color(Color2D::RED));
        mesh.vertices.push(Vertex::new(Vec3::new( s, 0.0, -s)).with_color(Color2D::GREEN));
        mesh.vertices.push(Vertex::new(Vec3::new( s, 0.0,  s)).with_color(Color2D::BLUE));
        mesh.vertices.push(Vertex::new(Vec3::new(-s, 0.0,  s)).with_color(Color2D::YELLOW));
        // Apex
        mesh.vertices.push(Vertex::new(Vec3::new(0.0, height, 0.0)).with_color(Color2D::WHITE));

        // Base
        mesh.triangles.push(Triangle::new(0, 2, 1));
        mesh.triangles.push(Triangle::new(0, 3, 2));
        // Sides
        mesh.triangles.push(Triangle::new(0, 1, 4));
        mesh.triangles.push(Triangle::new(1, 2, 4));
        mesh.triangles.push(Triangle::new(2, 3, 4));
        mesh.triangles.push(Triangle::new(3, 0, 4));

        mesh
    }

    /// Create a simple plane mesh
    pub fn plane(width: f32, depth: f32) -> Self {
        let w = width / 2.0;
        let d = depth / 2.0;
        let mut mesh = Self::new();

        mesh.vertices.push(Vertex::new(Vec3::new(-w, 0.0, -d)).with_normal(Vec3::Y).with_color(Color2D::GRAY));
        mesh.vertices.push(Vertex::new(Vec3::new( w, 0.0, -d)).with_normal(Vec3::Y).with_color(Color2D::GRAY));
        mesh.vertices.push(Vertex::new(Vec3::new( w, 0.0,  d)).with_normal(Vec3::Y).with_color(Color2D::GRAY));
        mesh.vertices.push(Vertex::new(Vec3::new(-w, 0.0,  d)).with_normal(Vec3::Y).with_color(Color2D::GRAY));

        mesh.triangles.push(Triangle::new(0, 1, 2));
        mesh.triangles.push(Triangle::new(0, 2, 3));

        mesh
    }

    /// Create an icosphere (approximated sphere)
    pub fn sphere(radius: f32, subdivisions: u32) -> Self {
        let mut mesh = Self::new();
        
        // Start with icosahedron
        let t = (1.0 + 5.0_f32.sqrt()) / 2.0;
        
        // Vertices of icosahedron
        let verts = [
            Vec3::new(-1.0,  t, 0.0).normalize().scale(radius),
            Vec3::new( 1.0,  t, 0.0).normalize().scale(radius),
            Vec3::new(-1.0, -t, 0.0).normalize().scale(radius),
            Vec3::new( 1.0, -t, 0.0).normalize().scale(radius),
            Vec3::new(0.0, -1.0,  t).normalize().scale(radius),
            Vec3::new(0.0,  1.0,  t).normalize().scale(radius),
            Vec3::new(0.0, -1.0, -t).normalize().scale(radius),
            Vec3::new(0.0,  1.0, -t).normalize().scale(radius),
            Vec3::new( t, 0.0, -1.0).normalize().scale(radius),
            Vec3::new( t, 0.0,  1.0).normalize().scale(radius),
            Vec3::new(-t, 0.0, -1.0).normalize().scale(radius),
            Vec3::new(-t, 0.0,  1.0).normalize().scale(radius),
        ];

        for v in &verts {
            mesh.vertices.push(Vertex::new(*v).with_normal(v.normalize()).with_color(Color2D::WHITE));
        }

        // Faces of icosahedron
        let faces = [
            (0, 11, 5), (0, 5, 1), (0, 1, 7), (0, 7, 10), (0, 10, 11),
            (1, 5, 9), (5, 11, 4), (11, 10, 2), (10, 7, 6), (7, 1, 8),
            (3, 9, 4), (3, 4, 2), (3, 2, 6), (3, 6, 8), (3, 8, 9),
            (4, 9, 5), (2, 4, 11), (6, 2, 10), (8, 6, 7), (9, 8, 1),
        ];

        for (a, b, c) in &faces {
            mesh.triangles.push(Triangle::new(*a, *b, *c));
        }

        // TODO: Subdivide for smoother sphere

        mesh
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// CAMERA
// ═══════════════════════════════════════════════════════════════════════════════

/// 3D Camera
pub struct Camera {
    pub position: Vec3,
    pub target: Vec3,
    pub up: Vec3,
    pub fov: f32, // in degrees
    pub near: f32,
    pub far: f32,
    pub aspect: f32,
}

impl Camera {
    pub fn new(aspect: f32) -> Self {
        Self {
            position: Vec3::new(0.0, 0.0, 5.0),
            target: Vec3::ZERO,
            up: Vec3::Y,
            fov: 60.0,
            near: 0.1,
            far: 100.0,
            aspect,
        }
    }

    pub fn look_at(&mut self, target: Vec3) {
        self.target = target;
    }

    pub fn view_matrix(&self) -> Mat4 {
        Mat4::look_at(self.position, self.target, self.up)
    }

    pub fn projection_matrix(&self) -> Mat4 {
        Mat4::perspective(deg_to_rad(self.fov), self.aspect, self.near, self.far)
    }

    /// Combined view-projection matrix
    pub fn matrix(&self) -> Mat4 {
        self.projection_matrix().mul(&self.view_matrix())
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// LIGHT
// ═══════════════════════════════════════════════════════════════════════════════

/// Light type
#[derive(Clone, Copy)]
pub enum LightType {
    Directional(Vec3), // direction
    Point(Vec3),       // position
    Ambient,
}

/// Light source
pub struct Light {
    pub light_type: LightType,
    pub color: Color2D,
    pub intensity: f32,
}

impl Light {
    pub fn directional(direction: Vec3, color: Color2D, intensity: f32) -> Self {
        Self {
            light_type: LightType::Directional(direction.normalize()),
            color,
            intensity,
        }
    }

    pub fn point(position: Vec3, color: Color2D, intensity: f32) -> Self {
        Self {
            light_type: LightType::Point(position),
            color,
            intensity,
        }
    }

    pub fn ambient(color: Color2D, intensity: f32) -> Self {
        Self {
            light_type: LightType::Ambient,
            color,
            intensity,
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// RENDER MODE
// ═══════════════════════════════════════════════════════════════════════════════

/// Rendering mode
#[derive(Clone, Copy, PartialEq)]
pub enum RenderMode {
    Wireframe,
    Solid,
    SolidWireframe,
}

// ═══════════════════════════════════════════════════════════════════════════════
// 3D RENDERER
// ═══════════════════════════════════════════════════════════════════════════════

/// Software 3D Renderer
pub struct Renderer3D {
    width: u32,
    height: u32,
    depth_buffer: Box<[f32]>,
    pub camera: Camera,
    pub lights: Vec<Light>,
    pub render_mode: RenderMode,
    pub wireframe_color: Color2D,
    pub background_color: Color2D,
}

impl Renderer3D {
    pub fn new(width: u32, height: u32) -> Self {
        let size = (width * height) as usize;
        let aspect = width as f32 / height as f32;

        let mut renderer = Self {
            width,
            height,
            depth_buffer: alloc::vec![f32::MAX; size].into_boxed_slice(),
            camera: Camera::new(aspect),
            lights: Vec::new(),
            render_mode: RenderMode::Wireframe,
            wireframe_color: Color2D::GREEN,
            background_color: Color2D::rgb(20, 20, 30),
        };

        // Add default light
        renderer.lights.push(Light::directional(
            Vec3::new(1.0, -1.0, -1.0),
            Color2D::WHITE,
            1.0,
        ));
        renderer.lights.push(Light::ambient(Color2D::WHITE, 0.2));

        renderer
    }

    /// Clear the depth buffer
    pub fn clear_depth(&mut self) {
        self.depth_buffer.fill(f32::MAX);
    }

    /// Project a 3D point to screen coordinates
    fn project(&self, point: Vec3, mvp: &Mat4) -> Option<(i32, i32, f32)> {
        let clip = mvp.transform_vec4(Vec4::from_vec3(point, 1.0));

        // Clip against near plane
        if clip.w <= 0.0 {
            return None;
        }

        // Perspective divide
        let ndc_x = clip.x / clip.w;
        let ndc_y = clip.y / clip.w;
        let ndc_z = clip.z / clip.w;

        // Check if in view
        if ndc_x < -1.0 || ndc_x > 1.0 || ndc_y < -1.0 || ndc_y > 1.0 {
            return None;
        }

        // Convert to screen coordinates
        let screen_x = ((ndc_x + 1.0) * 0.5 * self.width as f32) as i32;
        let screen_y = ((1.0 - ndc_y) * 0.5 * self.height as f32) as i32;

        Some((screen_x, screen_y, ndc_z))
    }

    /// Draw a line with depth testing
    fn draw_line_3d(&self, target: &mut FramebufferTarget, 
                    x0: i32, y0: i32, _z0: f32,
                    x1: i32, y1: i32, _z1: f32,
                    color: Color2D) {
        // Bresenham's line algorithm
        let dx = (x1 - x0).abs();
        let dy = -(y1 - y0).abs();
        let sx = if x0 < x1 { 1 } else { -1 };
        let sy = if y0 < y1 { 1 } else { -1 };
        let mut err = dx + dy;

        let mut x = x0;
        let mut y = y0;

        loop {
            if x >= 0 && y >= 0 && (x as u32) < self.width && (y as u32) < self.height {
                target.set_pixel(x as u32, y as u32, color.to_u32());
            }

            if x == x1 && y == y1 {
                break;
            }

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

    /// Calculate lighting for a surface
    fn calculate_lighting(&self, normal: Vec3, position: Vec3) -> Color2D {
        let mut final_r = 0.0f32;
        let mut final_g = 0.0f32;
        let mut final_b = 0.0f32;

        for light in &self.lights {
            let (intensity, color) = match light.light_type {
                LightType::Ambient => {
                    (light.intensity, light.color)
                }
                LightType::Directional(dir) => {
                    let ndotl = normal.dot(-dir).max(0.0);
                    (ndotl * light.intensity, light.color)
                }
                LightType::Point(pos) => {
                    let to_light = (pos - position).normalize();
                    let ndotl = normal.dot(to_light).max(0.0);
                    let dist = (pos - position).length();
                    let attenuation = 1.0 / (1.0 + 0.1 * dist + 0.01 * dist * dist);
                    (ndotl * light.intensity * attenuation, light.color)
                }
            };

            final_r += (color.r as f32 / 255.0) * intensity;
            final_g += (color.g as f32 / 255.0) * intensity;
            final_b += (color.b as f32 / 255.0) * intensity;
        }

        Color2D::rgb(
            (final_r * 255.0).min(255.0) as u8,
            (final_g * 255.0).min(255.0) as u8,
            (final_b * 255.0).min(255.0) as u8,
        )
    }

    /// Fill a triangle with flat shading
    fn fill_triangle(&mut self, target: &mut FramebufferTarget,
                     x0: i32, y0: i32, z0: f32,
                     x1: i32, y1: i32, z1: f32,
                     x2: i32, y2: i32, z2: f32,
                     color: Color2D) {
        // Sort vertices by y
        let mut pts = [(x0, y0, z0), (x1, y1, z1), (x2, y2, z2)];
        if pts[0].1 > pts[1].1 { pts.swap(0, 1); }
        if pts[1].1 > pts[2].1 { pts.swap(1, 2); }
        if pts[0].1 > pts[1].1 { pts.swap(0, 1); }

        let (x0, y0, z0) = pts[0];
        let (x1, y1, z1) = pts[1];
        let (x2, y2, z2) = pts[2];

        let total_height = y2 - y0;
        if total_height == 0 {
            return;
        }

        for y in y0..=y2 {
            if y < 0 || y as u32 >= self.height {
                continue;
            }

            let second_half = y > y1 || y1 == y0;
            let segment_height = if second_half { y2 - y1 } else { y1 - y0 };
            if segment_height == 0 {
                continue;
            }

            let alpha = (y - y0) as f32 / total_height as f32;
            let beta = if second_half {
                (y - y1) as f32 / segment_height as f32
            } else {
                (y - y0) as f32 / segment_height as f32
            };

            let mut xa = x0 as f32 + (x2 - x0) as f32 * alpha;
            let mut xb = if second_half {
                x1 as f32 + (x2 - x1) as f32 * beta
            } else {
                x0 as f32 + (x1 - x0) as f32 * beta
            };

            let mut za = z0 + (z2 - z0) * alpha;
            let mut zb = if second_half {
                z1 + (z2 - z1) * beta
            } else {
                z0 + (z1 - z0) * beta
            };

            if xa > xb {
                core::mem::swap(&mut xa, &mut xb);
                core::mem::swap(&mut za, &mut zb);
            }

            let x_start = xa as i32;
            let x_end = xb as i32;

            for x in x_start..=x_end {
                if x < 0 || x as u32 >= self.width {
                    continue;
                }

                let t = if x_end != x_start {
                    (x - x_start) as f32 / (x_end - x_start) as f32
                } else {
                    0.0
                };
                let z = za + (zb - za) * t;

                let idx = (y as u32 * self.width + x as u32) as usize;
                if z < self.depth_buffer[idx] {
                    self.depth_buffer[idx] = z;
                    target.set_pixel(x as u32, y as u32, color.to_u32());
                }
            }
        }
    }

    /// Render a mesh
    pub fn render_mesh(&mut self, target: &mut FramebufferTarget, mesh: &Mesh) {
        let model = mesh.transform.matrix();
        let view_proj = self.camera.matrix();
        let mvp = view_proj.mul(&model);

        for tri in &mesh.triangles {
            let v0 = &mesh.vertices[tri.v0];
            let v1 = &mesh.vertices[tri.v1];
            let v2 = &mesh.vertices[tri.v2];

            // Transform vertices
            let p0 = model.transform_point(v0.position);
            let p1 = model.transform_point(v1.position);
            let p2 = model.transform_point(v2.position);

            // Calculate face normal for backface culling and lighting
            let edge1 = p1 - p0;
            let edge2 = p2 - p0;
            let normal = edge1.cross(edge2).normalize();

            // Backface culling
            let view_dir = (self.camera.position - p0).normalize();
            if normal.dot(view_dir) < 0.0 {
                continue;
            }

            // Project to screen
            let proj0 = self.project(v0.position, &mvp);
            let proj1 = self.project(v1.position, &mvp);
            let proj2 = self.project(v2.position, &mvp);

            match (proj0, proj1, proj2) {
                (Some((x0, y0, z0)), Some((x1, y1, z1)), Some((x2, y2, z2))) => {
                    match self.render_mode {
                        RenderMode::Wireframe => {
                            self.draw_line_3d(target, x0, y0, z0, x1, y1, z1, self.wireframe_color);
                            self.draw_line_3d(target, x1, y1, z1, x2, y2, z2, self.wireframe_color);
                            self.draw_line_3d(target, x2, y2, z2, x0, y0, z0, self.wireframe_color);
                        }
                        RenderMode::Solid => {
                            let center = (p0 + p1 + p2).scale(1.0 / 3.0);
                            let light = self.calculate_lighting(normal, center);
                            let shaded = Color2D::rgb(
                                ((v0.color.r as u32 * light.r as u32) / 255) as u8,
                                ((v0.color.g as u32 * light.g as u32) / 255) as u8,
                                ((v0.color.b as u32 * light.b as u32) / 255) as u8,
                            );
                            self.fill_triangle(target, x0, y0, z0, x1, y1, z1, x2, y2, z2, shaded);
                        }
                        RenderMode::SolidWireframe => {
                            let center = (p0 + p1 + p2).scale(1.0 / 3.0);
                            let light = self.calculate_lighting(normal, center);
                            let shaded = Color2D::rgb(
                                ((v0.color.r as u32 * light.r as u32) / 255) as u8,
                                ((v0.color.g as u32 * light.g as u32) / 255) as u8,
                                ((v0.color.b as u32 * light.b as u32) / 255) as u8,
                            );
                            self.fill_triangle(target, x0, y0, z0, x1, y1, z1, x2, y2, z2, shaded);
                            self.draw_line_3d(target, x0, y0, z0, x1, y1, z1, Color2D::BLACK);
                            self.draw_line_3d(target, x1, y1, z1, x2, y2, z2, Color2D::BLACK);
                            self.draw_line_3d(target, x2, y2, z2, x0, y0, z0, Color2D::BLACK);
                        }
                    }
                }
                _ => {}
            }
        }
    }

    /// Clear and prepare for new frame
    pub fn begin_frame(&mut self, target: &mut FramebufferTarget) {
        target.clear_color(self.background_color.to_u32());
        self.clear_depth();
    }
}
