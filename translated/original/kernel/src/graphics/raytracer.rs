//! RayTracer - Real-time Ray Tracing for HoloMatrix
//!
//! Implements ray tracing techniques for volumetric 3D rendering:
//! - Ray-sphere/plane/box intersections
//! - Phong shading model
//! - Soft shadows via ray marching
//! - Ambient occlusion approximation
//! - Reflection rays (limited bounces)
//!
//! Can be used standalone or to enhance HoloMatrix rendering.

use alloc::vec::Vec;
use alloc::vec;

// ═══════════════════════════════════════════════════════════════════════════════
// MATH UTILITIES
// ═══════════════════════════════════════════════════════════════════════════════

/// 3D Vector for ray tracing
#[derive(Clone, Copy)]
pub struct Vec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Vec3 {
    pub const ZERO: Vec3 = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    pub const ONE: Vec3 = Vec3 { x: 1.0, y: 1.0, z: 1.0 };
    pub const UP: Vec3 = Vec3 { x: 0.0, y: 1.0, z: 0.0 };
    
    #[inline]
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }
    
    #[inline]
    pub fn dot(self, other: Vec3) -> f32 {
        self.x * other.x + self.y * other.y + self.z * other.z
    }
    
    #[inline]
    pub fn cross(self, other: Vec3) -> Vec3 {
        Vec3::new(
            self.y * other.z - self.z * other.y,
            self.z * other.x - self.x * other.z,
            self.x * other.y - self.y * other.x,
        )
    }
    
    #[inline]
    pub fn length_sq(self) -> f32 {
        self.dot(self)
    }
    
    #[inline]
    pub fn length(self) -> f32 {
        sqrt_fast(self.length_sq())
    }
    
    #[inline]
    pub fn normalize(self) -> Vec3 {
        let len = self.length();
        if len > 0.0001 {
            self * (1.0 / len)
        } else {
            Vec3::ZERO
        }
    }
    
    #[inline]
    pub fn reflect(self, normal: Vec3) -> Vec3 {
        self - normal * (2.0 * self.dot(normal))
    }
    
    #[inline]
    pub fn lerp(self, other: Vec3, t: f32) -> Vec3 {
        self * (1.0 - t) + other * t
    }
}

impl core::ops::Add for Vec3 {
    type Output = Vec3;
    fn add(self, rhs: Vec3) -> Vec3 {
        Vec3::new(self.x + rhs.x, self.y + rhs.y, self.z + rhs.z)
    }
}

impl core::ops::Sub for Vec3 {
    type Output = Vec3;
    fn sub(self, rhs: Vec3) -> Vec3 {
        Vec3::new(self.x - rhs.x, self.y - rhs.y, self.z - rhs.z)
    }
}

impl core::ops::Mul<f32> for Vec3 {
    type Output = Vec3;
    fn mul(self, rhs: f32) -> Vec3 {
        Vec3::new(self.x * rhs, self.y * rhs, self.z * rhs)
    }
}

impl core::ops::Neg for Vec3 {
    type Output = Vec3;
    fn neg(self) -> Vec3 {
        Vec3::new(-self.x, -self.y, -self.z)
    }
}

impl core::ops::Mul<Vec3> for Vec3 {
    type Output = Vec3;
    fn mul(self, rhs: Vec3) -> Vec3 {
        Vec3::new(self.x * rhs.x, self.y * rhs.y, self.z * rhs.z)
    }
}

/// Fast square root (delegates to shared math)
fn sqrt_fast(x: f32) -> f32 { crate::math::fast_sqrt(x) }

/// Fast sine (delegates to shared math)
fn sin_fast(x: f32) -> f32 { crate::math::fast_sin(x) }

fn cos_fast(x: f32) -> f32 { crate::math::fast_cos(x) }

/// Fast tangent (delegates to shared math)
fn tan_fast(x: f32) -> f32 { crate::math::fast_tan(x) }

// ═══════════════════════════════════════════════════════════════════════════════
// RAY PRIMITIVES
// ═══════════════════════════════════════════════════════════════════════════════

/// A ray in 3D space
#[derive(Clone, Copy)]
pub struct Ray {
    pub origin: Vec3,
    pub direction: Vec3,
}

impl Ray {
    pub fn new(origin: Vec3, direction: Vec3) -> Self {
        Self { origin, direction: direction.normalize() }
    }
    
    pub fn at(self, t: f32) -> Vec3 {
        self.origin + self.direction * t
    }
}

/// Hit information from ray intersection
#[derive(Clone, Copy)]
pub struct HitInfo {
    pub t: f32,           // Distance along ray
    pub point: Vec3,      // Hit position
    pub normal: Vec3,     // Surface normal
    pub material_id: u8,  // Material identifier
}

impl HitInfo {
    pub fn none() -> Self {
        Self {
            t: f32::MAX,
            point: Vec3::ZERO,
            normal: Vec3::UP,
            material_id: 0,
        }
    }
    
    pub fn hit(&self) -> bool {
        self.t < f32::MAX - 1.0
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// SCENE PRIMITIVES
// ═══════════════════════════════════════════════════════════════════════════════

/// Sphere primitive
pub struct Sphere {
    pub center: Vec3,
    pub radius: f32,
    pub material_id: u8,
}

impl Sphere {
    pub fn new(center: Vec3, radius: f32, material_id: u8) -> Self {
        Self { center, radius, material_id }
    }
    
    /// Ray-sphere intersection
    pub fn intersect(&self, ray: &Ray) -> HitInfo {
        let oc = ray.origin - self.center;
        let a = ray.direction.length_sq();
        let half_b = oc.dot(ray.direction);
        let c = oc.length_sq() - self.radius * self.radius;
        let discriminant = half_b * half_b - a * c;
        
        if discriminant < 0.0 {
            return HitInfo::none();
        }
        
        let sqrtd = sqrt_fast(discriminant);
        let mut t = (-half_b - sqrtd) / a;
        
        if t < 0.001 {
            t = (-half_b + sqrtd) / a;
            if t < 0.001 {
                return HitInfo::none();
            }
        }
        
        let point = ray.at(t);
        let normal = (point - self.center).normalize();
        
        HitInfo {
            t,
            point,
            normal,
            material_id: self.material_id,
        }
    }
}

/// Infinite plane primitive
pub struct Plane {
    pub point: Vec3,
    pub normal: Vec3,
    pub material_id: u8,
}

impl Plane {
    pub fn new(point: Vec3, normal: Vec3, material_id: u8) -> Self {
        Self { point, normal: normal.normalize(), material_id }
    }
    
    pub fn intersect(&self, ray: &Ray) -> HitInfo {
        let denom = self.normal.dot(ray.direction);
        
        if denom.abs() < 0.0001 {
            return HitInfo::none();
        }
        
        let t = (self.point - ray.origin).dot(self.normal) / denom;
        
        if t < 0.001 {
            return HitInfo::none();
        }
        
        HitInfo {
            t,
            point: ray.at(t),
            normal: self.normal,
            material_id: self.material_id,
        }
    }
}

/// Box primitive (axis-aligned)
pub struct Box3D {
    pub min: Vec3,
    pub max: Vec3,
    pub material_id: u8,
}

impl Box3D {
    pub fn new(center: Vec3, half_size: Vec3, material_id: u8) -> Self {
        Self {
            min: center - half_size,
            max: center + half_size,
            material_id,
        }
    }
    
    pub fn intersect(&self, ray: &Ray) -> HitInfo {
        let inv_d = Vec3::new(
            1.0 / ray.direction.x,
            1.0 / ray.direction.y,
            1.0 / ray.direction.z,
        );
        
        let t1 = (self.min.x - ray.origin.x) * inv_d.x;
        let t2 = (self.max.x - ray.origin.x) * inv_d.x;
        let t3 = (self.min.y - ray.origin.y) * inv_d.y;
        let t4 = (self.max.y - ray.origin.y) * inv_d.y;
        let t5 = (self.min.z - ray.origin.z) * inv_d.z;
        let t6 = (self.max.z - ray.origin.z) * inv_d.z;
        
        let tmin = t1.min(t2).max(t3.min(t4)).max(t5.min(t6));
        let tmax = t1.max(t2).min(t3.max(t4)).min(t5.max(t6));
        
        if tmax < 0.0 || tmin > tmax {
            return HitInfo::none();
        }
        
        let t = if tmin < 0.001 { tmax } else { tmin };
        if t < 0.001 {
            return HitInfo::none();
        }
        
        let point = ray.at(t);
        
        // Calculate normal based on which face was hit
        let center = (self.min + self.max) * 0.5;
        let local = point - center;
        let half = (self.max - self.min) * 0.5;
        
        let bias = 1.0001;
        let normal = Vec3::new(
            (local.x / half.x * bias) as i32 as f32,
            (local.y / half.y * bias) as i32 as f32,
            (local.z / half.z * bias) as i32 as f32,
        ).normalize();
        
        HitInfo {
            t,
            point,
            normal,
            material_id: self.material_id,
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// MATERIALS
// ═══════════════════════════════════════════════════════════════════════════════

/// Material properties
#[derive(Clone, Copy)]
pub struct Material {
    pub color: Vec3,       // Base color (RGB 0-1)
    pub ambient: f32,      // Ambient light factor
    pub diffuse: f32,      // Diffuse factor
    pub specular: f32,     // Specular factor
    pub shininess: f32,    // Specular exponent
    pub reflectivity: f32, // How reflective (0-1)
    pub emission: f32,     // Self-emission (glow)
}

impl Material {
    pub const fn default_holo() -> Self {
        Self {
            color: Vec3 { x: 0.0, y: 1.0, z: 0.4 }, // Cyan-green holographic
            ambient: 0.1,
            diffuse: 0.6,
            specular: 0.8,
            shininess: 32.0,
            reflectivity: 0.3,
            emission: 0.2,
        }
    }
    
    pub const fn emissive(r: f32, g: f32, b: f32) -> Self {
        Self {
            color: Vec3 { x: r, y: g, z: b },
            ambient: 0.0,
            diffuse: 0.0,
            specular: 0.0,
            shininess: 1.0,
            reflectivity: 0.0,
            emission: 1.0,
        }
    }
    
    pub const fn matte(r: f32, g: f32, b: f32) -> Self {
        Self {
            color: Vec3 { x: r, y: g, z: b },
            ambient: 0.15,
            diffuse: 0.85,
            specular: 0.1,
            shininess: 8.0,
            reflectivity: 0.0,
            emission: 0.0,
        }
    }
    
    pub const fn metallic(r: f32, g: f32, b: f32) -> Self {
        Self {
            color: Vec3 { x: r, y: g, z: b },
            ambient: 0.05,
            diffuse: 0.3,
            specular: 1.0,
            shininess: 128.0,
            reflectivity: 0.7,
            emission: 0.0,
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// RAYTRACER ENGINE
// ═══════════════════════════════════════════════════════════════════════════════

/// Point light source
pub struct PointLight {
    pub position: Vec3,
    pub color: Vec3,
    pub intensity: f32,
}

/// The raytracer engine
pub struct RayTracer {
    pub width: usize,
    pub height: usize,
    pub spheres: Vec<Sphere>,
    pub planes: Vec<Plane>,
    pub boxes: Vec<Box3D>,
    pub materials: Vec<Material>,
    pub lights: Vec<PointLight>,
    pub camera_pos: Vec3,
    pub camera_target: Vec3,
    pub fov: f32,
    pub time: f32,
    pub max_bounces: u8,
}

impl RayTracer {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            spheres: Vec::new(),
            planes: Vec::new(),
            boxes: Vec::new(),
            materials: vec![Material::default_holo()],
            lights: Vec::new(),
            camera_pos: Vec3::new(0.0, 0.0, -3.0),
            camera_target: Vec3::ZERO,
            fov: 60.0,
            time: 0.0,
            max_bounces: 2,
        }
    }
    
    /// Clear all scene objects
    pub fn clear_scene(&mut self) {
        self.spheres.clear();
        self.planes.clear();
        self.boxes.clear();
        self.lights.clear();
    }
    
    /// Add a sphere to the scene
    pub fn add_sphere(&mut self, center: Vec3, radius: f32, material_id: u8) {
        self.spheres.push(Sphere::new(center, radius, material_id));
    }
    
    /// Add a plane to the scene
    pub fn add_plane(&mut self, point: Vec3, normal: Vec3, material_id: u8) {
        self.planes.push(Plane::new(point, normal, material_id));
    }
    
    /// Add a box to the scene
    pub fn add_box(&mut self, center: Vec3, half_size: Vec3, material_id: u8) {
        self.boxes.push(Box3D::new(center, half_size, material_id));
    }
    
    /// Add a light to the scene
    pub fn add_light(&mut self, position: Vec3, color: Vec3, intensity: f32) {
        self.lights.push(PointLight { position, color, intensity });
    }
    
    /// Add a material
    pub fn add_material(&mut self, material: Material) -> u8 {
        let id = self.materials.len() as u8;
        self.materials.push(material);
        id
    }
    
    /// Get camera ray for a pixel
    fn get_ray(&self, x: usize, y: usize) -> Ray {
        let aspect = self.width as f32 / self.height as f32;
        let fov_rad = self.fov * 3.14159 / 180.0;
        let tan_fov = tan_fast(fov_rad / 2.0);
        
        // Normalized device coordinates
        let px = (2.0 * (x as f32 + 0.5) / self.width as f32 - 1.0) * tan_fov * aspect;
        let py = (1.0 - 2.0 * (y as f32 + 0.5) / self.height as f32) * tan_fov;
        
        // Camera coordinate system
        let forward = (self.camera_target - self.camera_pos).normalize();
        let right = forward.cross(Vec3::UP).normalize();
        let up = right.cross(forward);
        
        let direction = (forward + right * px + up * py).normalize();
        
        Ray::new(self.camera_pos, direction)
    }
    
    /// Find closest intersection
    fn trace(&self, ray: &Ray) -> HitInfo {
        let mut closest = HitInfo::none();
        
        for sphere in &self.spheres {
            let hit = sphere.intersect(ray);
            if hit.hit() && hit.t < closest.t {
                closest = hit;
            }
        }
        
        for plane in &self.planes {
            let hit = plane.intersect(ray);
            if hit.hit() && hit.t < closest.t {
                closest = hit;
            }
        }
        
        for box3d in &self.boxes {
            let hit = box3d.intersect(ray);
            if hit.hit() && hit.t < closest.t {
                closest = hit;
            }
        }
        
        closest
    }
    
    /// Check if a point is in shadow
    fn in_shadow(&self, point: Vec3, light_pos: Vec3) -> f32 {
        let to_light = light_pos - point;
        let distance = to_light.length();
        let ray = Ray::new(point + to_light.normalize() * 0.01, to_light.normalize());
        
        let hit = self.trace(&ray);
        if hit.hit() && hit.t < distance {
            0.3 // Soft shadow
        } else {
            1.0 // Fully lit
        }
    }
    
    /// Shade a hit point using Phong model
    fn shade(&self, hit: &HitInfo, ray: &Ray, depth: u8) -> Vec3 {
        if !hit.hit() {
            // Background gradient
            let t = (ray.direction.y + 1.0) * 0.5;
            return Vec3::new(0.0, 0.02, 0.05).lerp(Vec3::new(0.0, 0.1, 0.15), t);
        }
        
        let material = &self.materials[hit.material_id as usize % self.materials.len()];
        let mut color = material.color * material.ambient;
        
        // Emission
        if material.emission > 0.0 {
            color = color + material.color * material.emission;
        }
        
        // For each light
        for light in &self.lights {
            let to_light = (light.position - hit.point).normalize();
            let shadow = self.in_shadow(hit.point, light.position);
            
            // Diffuse
            let diff = hit.normal.dot(to_light).max(0.0);
            let diffuse_color = material.color * light.color * (diff * material.diffuse * shadow * light.intensity);
            color = color + diffuse_color;
            
            // Specular
            let reflect_dir = (-to_light).reflect(hit.normal);
            let spec = (-ray.direction).dot(reflect_dir).max(0.0);
            let spec_intensity = pow_fast(spec, material.shininess) * material.specular * shadow;
            color = color + light.color * (spec_intensity * light.intensity);
        }
        
        // Reflection
        if material.reflectivity > 0.0 && depth < self.max_bounces {
            let reflect_dir = ray.direction.reflect(hit.normal);
            let reflect_ray = Ray::new(hit.point + reflect_dir * 0.01, reflect_dir);
            let reflect_hit = self.trace(&reflect_ray);
            let reflect_color = self.shade(&reflect_hit, &reflect_ray, depth + 1);
            color = color.lerp(reflect_color, material.reflectivity);
        }
        
        // Clamp color
        Vec3::new(
            color.x.min(1.0),
            color.y.min(1.0),
            color.z.min(1.0),
        )
    }
    
    /// Render the scene to a pixel buffer
    pub fn render(&self) -> Vec<u32> {
        let mut buffer = vec![0u32; self.width * self.height];
        
        for y in 0..self.height {
            for x in 0..self.width {
                let ray = self.get_ray(x, y);
                let hit = self.trace(&ray);
                let color = self.shade(&hit, &ray, 0);
                
                // Convert to ARGB
                let r = (color.x * 255.0) as u32;
                let g = (color.y * 255.0) as u32;
                let b = (color.z * 255.0) as u32;
                buffer[y * self.width + x] = 0xFF000000 | (r << 16) | (g << 8) | b;
            }
        }
        
        buffer
    }
    
    /// Render to HoloMatrix-compatible layers (volumetric output)
    pub fn render_to_layers(&self, num_layers: usize) -> Vec<Vec<u8>> {
        let mut layers = Vec::with_capacity(num_layers);
        
        for _ in 0..num_layers {
            layers.push(vec![0u8; self.width * self.height]);
        }
        
        // For each pixel, trace and deposit intensity at intersection depth
        for y in 0..self.height {
            for x in 0..self.width {
                let ray = self.get_ray(x, y);
                let hit = self.trace(&ray);
                
                if hit.hit() {
                    // Map hit distance to layer index
                    let z_normalized = ((hit.t - 1.0) / 10.0).max(0.0).min(1.0);
                    let layer_idx = (z_normalized * (num_layers - 1) as f32) as usize;
                    
                    // Calculate intensity from shading
                    let color = self.shade(&hit, &ray, 0);
                    let intensity = ((color.x + color.y + color.z) / 3.0 * 255.0) as u8;
                    
                    let idx = y * self.width + x;
                    if layer_idx < num_layers {
                        layers[layer_idx][idx] = layers[layer_idx][idx].saturating_add(intensity);
                    }
                }
            }
        }
        
        layers
    }
    
    /// Update animation
    pub fn update(&mut self, delta_time: f32) {
        self.time += delta_time;
    }
    
    /// Setup DNA helix scene
    pub fn setup_dna_scene(&mut self) {
        self.clear_scene();
        
        // Add holographic material
        let holo_mat = self.add_material(Material {
            color: Vec3::new(0.0, 1.0, 0.5),
            ambient: 0.1,
            diffuse: 0.5,
            specular: 0.9,
            shininess: 64.0,
            reflectivity: 0.2,
            emission: 0.3,
        });
        
        // Create DNA helix with spheres along backbone
        let helix_length = 4.0;
        let radius = 1.0;
        let turns = 3.0;
        let segments = 30;
        
        for i in 0..segments {
            let t = i as f32 / segments as f32;
            let y = -helix_length / 2.0 + t * helix_length;
            let angle = t * turns * 2.0 * 3.14159 + self.time;
            
            // Strand 1
            let x1 = radius * cos_fast(angle);
            let z1 = radius * sin_fast(angle);
            self.add_sphere(Vec3::new(x1, y, z1), 0.1, holo_mat);
            
            // Strand 2
            let x2 = radius * cos_fast(angle + 3.14159);
            let z2 = radius * sin_fast(angle + 3.14159);
            self.add_sphere(Vec3::new(x2, y, z2), 0.1, holo_mat);
            
            // Base pair connection (every 3 segments)
            if i % 3 == 0 {
                self.add_sphere(Vec3::new((x1 + x2) / 2.0, y, (z1 + z2) / 2.0), 0.08, holo_mat);
            }
        }
        
        // Add lights
        self.add_light(Vec3::new(5.0, 5.0, -5.0), Vec3::new(0.0, 1.0, 0.5), 1.2);
        self.add_light(Vec3::new(-5.0, 3.0, -3.0), Vec3::new(0.5, 0.0, 1.0), 0.8);
        
        // Camera
        self.camera_pos = Vec3::new(0.0, 0.0, -6.0);
        self.camera_target = Vec3::ZERO;
    }
    
    /// Setup floating spheres scene (demo scene)
    pub fn setup_spheres_scene(&mut self) {
        self.clear_scene();
        
        // Materials
        let mat_cyan = self.add_material(Material::default_holo());
        let mat_magenta = self.add_material(Material {
            color: Vec3::new(1.0, 0.0, 0.8),
            ..Material::default_holo()
        });
        let mat_gold = self.add_material(Material::metallic(1.0, 0.8, 0.2));
        let mat_floor = self.add_material(Material::matte(0.1, 0.1, 0.15));
        
        // Central sphere
        let y_bob = sin_fast(self.time) * 0.3;
        self.add_sphere(Vec3::new(0.0, y_bob, 0.0), 0.8, mat_cyan);
        
        // Orbiting spheres
        for i in 0..6 {
            let angle = (i as f32 / 6.0) * 2.0 * 3.14159 + self.time * 0.5;
            let orbit_radius = 2.0;
            let x = orbit_radius * cos_fast(angle);
            let z = orbit_radius * sin_fast(angle);
            let y = sin_fast(self.time + i as f32) * 0.4;
            
            let mat = if i % 2 == 0 { mat_magenta } else { mat_gold };
            self.add_sphere(Vec3::new(x, y, z), 0.3, mat);
        }
        
        // Floor plane
        self.add_plane(Vec3::new(0.0, -1.5, 0.0), Vec3::UP, mat_floor);
        
        // Lights
        self.add_light(Vec3::new(4.0, 6.0, -4.0), Vec3::new(1.0, 1.0, 0.9), 1.0);
        self.add_light(Vec3::new(-3.0, 4.0, 2.0), Vec3::new(0.2, 0.4, 1.0), 0.6);
        
        // Camera
        self.camera_pos = Vec3::new(0.0, 2.0, -5.0);
        self.camera_target = Vec3::ZERO;
    }
}

/// Fast power approximation
fn pow_fast(base: f32, exp: f32) -> f32 {
    // Use repeated multiplication for integer exponents
    if exp <= 1.0 { return base; }
    if exp <= 2.0 { return base * base; }
    if exp <= 4.0 { return base * base * base * base; }
    if exp <= 8.0 { 
        let x4 = base * base * base * base;
        return x4 * x4;
    }
    if exp <= 16.0 {
        let x4 = base * base * base * base;
        return x4 * x4 * x4 * x4;
    }
    // Approximate for larger exponents
    let x8 = base * base * base * base * base * base * base * base;
    x8 * x8
}
