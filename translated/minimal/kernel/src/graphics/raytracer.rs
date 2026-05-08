










use alloc::vec::Vec;
use alloc::vec;






#[derive(Clone, Copy)]
pub struct Vec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Vec3 {
    pub const Bk: Vec3 = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    pub const Amy: Vec3 = Vec3 { x: 1.0, y: 1.0, z: 1.0 };
    pub const Np: Vec3 = Vec3 { x: 0.0, y: 1.0, z: 0.0 };
    
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
        jhj(self.length_sq())
    }
    
    #[inline]
    pub fn normalize(self) -> Vec3 {
        let len = self.length();
        if len > 0.0001 {
            self * (1.0 / len)
        } else {
            Vec3::Bk
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
    fn add(self, amp: Vec3) -> Vec3 {
        Vec3::new(self.x + amp.x, self.y + amp.y, self.z + amp.z)
    }
}

impl core::ops::Sub for Vec3 {
    type Output = Vec3;
    fn sub(self, amp: Vec3) -> Vec3 {
        Vec3::new(self.x - amp.x, self.y - amp.y, self.z - amp.z)
    }
}

impl core::ops::Mul<f32> for Vec3 {
    type Output = Vec3;
    fn mul(self, amp: f32) -> Vec3 {
        Vec3::new(self.x * amp, self.y * amp, self.z * amp)
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
    fn mul(self, amp: Vec3) -> Vec3 {
        Vec3::new(self.x * amp.x, self.y * amp.y, self.z * amp.z)
    }
}


fn jhj(x: f32) -> f32 { crate::math::ra(x) }


fn dzp(x: f32) -> f32 { crate::math::eu(x) }

fn fon(x: f32) -> f32 { crate::math::hr(x) }


fn pdb(x: f32) -> f32 { crate::math::hxv(x) }






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


#[derive(Clone, Copy)]
pub struct HitInfo {
    pub t: f32,           
    pub point: Vec3,      
    pub normal: Vec3,     
    pub material_id: u8,  
}

impl HitInfo {
    pub fn cng() -> Self {
        Self {
            t: f32::MAX,
            point: Vec3::Bk,
            normal: Vec3::Np,
            material_id: 0,
        }
    }
    
    pub fn hit(&self) -> bool {
        self.t < f32::MAX - 1.0
    }
}






pub struct Sphere {
    pub center: Vec3,
    pub radius: f32,
    pub material_id: u8,
}

impl Sphere {
    pub fn new(center: Vec3, radius: f32, material_id: u8) -> Self {
        Self { center, radius, material_id }
    }
    
    
    pub fn intersect(&self, ob: &Ray) -> HitInfo {
        let bnx = ob.origin - self.center;
        let a = ob.direction.length_sq();
        let eoq = bnx.dot(ob.direction);
        let c = bnx.length_sq() - self.radius * self.radius;
        let cwp = eoq * eoq - a * c;
        
        if cwp < 0.0 {
            return HitInfo::cng();
        }
        
        let jhk = jhj(cwp);
        let mut t = (-eoq - jhk) / a;
        
        if t < 0.001 {
            t = (-eoq + jhk) / a;
            if t < 0.001 {
                return HitInfo::cng();
            }
        }
        
        let point = ob.at(t);
        let normal = (point - self.center).normalize();
        
        HitInfo {
            t,
            point,
            normal,
            material_id: self.material_id,
        }
    }
}


pub struct Plane {
    pub point: Vec3,
    pub normal: Vec3,
    pub material_id: u8,
}

impl Plane {
    pub fn new(point: Vec3, normal: Vec3, material_id: u8) -> Self {
        Self { point, normal: normal.normalize(), material_id }
    }
    
    pub fn intersect(&self, ob: &Ray) -> HitInfo {
        let cwi = self.normal.dot(ob.direction);
        
        if cwi.abs() < 0.0001 {
            return HitInfo::cng();
        }
        
        let t = (self.point - ob.origin).dot(self.normal) / cwi;
        
        if t < 0.001 {
            return HitInfo::cng();
        }
        
        HitInfo {
            t,
            point: ob.at(t),
            normal: self.normal,
            material_id: self.material_id,
        }
    }
}


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
    
    pub fn intersect(&self, ob: &Ray) -> HitInfo {
        let aub = Vec3::new(
            1.0 / ob.direction.x,
            1.0 / ob.direction.y,
            1.0 / ob.direction.z,
        );
        
        let ll = (self.min.x - ob.origin.x) * aub.x;
        let np = (self.max.x - ob.origin.x) * aub.x;
        let acw = (self.min.y - ob.origin.y) * aub.y;
        let bdx = (self.max.y - ob.origin.y) * aub.y;
        let bwd = (self.min.z - ob.origin.z) * aub.z;
        let bwe = (self.max.z - ob.origin.z) * aub.z;
        
        let gzb = ll.min(np).max(acw.min(bdx)).max(bwd.min(bwe));
        let gza = ll.max(np).min(acw.max(bdx)).min(bwd.max(bwe));
        
        if gza < 0.0 || gzb > gza {
            return HitInfo::cng();
        }
        
        let t = if gzb < 0.001 { gza } else { gzb };
        if t < 0.001 {
            return HitInfo::cng();
        }
        
        let point = ob.at(t);
        
        
        let center = (self.min + self.max) * 0.5;
        let ggb = point - center;
        let cw = (self.max - self.min) * 0.5;
        
        let fjb = 1.0001;
        let normal = Vec3::new(
            (ggb.x / cw.x * fjb) as i32 as f32,
            (ggb.y / cw.y * fjb) as i32 as f32,
            (ggb.z / cw.z * fjb) as i32 as f32,
        ).normalize();
        
        HitInfo {
            t,
            point,
            normal,
            material_id: self.material_id,
        }
    }
}






#[derive(Clone, Copy)]
pub struct Material {
    pub color: Vec3,       
    pub ambient: f32,      
    pub diffuse: f32,      
    pub specular: f32,     
    pub shininess: f32,    
    pub reflectivity: f32, 
    pub emission: f32,     
}

impl Material {
    pub const fn frd() -> Self {
        Self {
            color: Vec3 { x: 0.0, y: 1.0, z: 0.4 }, 
            ambient: 0.1,
            diffuse: 0.6,
            specular: 0.8,
            shininess: 32.0,
            reflectivity: 0.3,
            emission: 0.2,
        }
    }
    
    pub const fn qep(r: f32, g: f32, b: f32) -> Self {
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
    
    pub const fn nco(r: f32, g: f32, b: f32) -> Self {
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
    
    pub const fn nfc(r: f32, g: f32, b: f32) -> Self {
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






pub struct Acq {
    pub position: Vec3,
    pub color: Vec3,
    pub intensity: f32,
}


pub struct RayTracer {
    pub width: usize,
    pub height: usize,
    pub spheres: Vec<Sphere>,
    pub planes: Vec<Plane>,
    pub boxes: Vec<Box3D>,
    pub materials: Vec<Material>,
    pub lights: Vec<Acq>,
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
            materials: vec![Material::frd()],
            lights: Vec::new(),
            camera_pos: Vec3::new(0.0, 0.0, -3.0),
            camera_target: Vec3::Bk,
            fov: 60.0,
            time: 0.0,
            max_bounces: 2,
        }
    }
    
    
    pub fn clear_scene(&mut self) {
        self.spheres.clear();
        self.planes.clear();
        self.boxes.clear();
        self.lights.clear();
    }
    
    
    pub fn add_sphere(&mut self, center: Vec3, radius: f32, material_id: u8) {
        self.spheres.push(Sphere::new(center, radius, material_id));
    }
    
    
    pub fn add_plane(&mut self, point: Vec3, normal: Vec3, material_id: u8) {
        self.planes.push(Plane::new(point, normal, material_id));
    }
    
    
    pub fn avz(&mut self, center: Vec3, half_size: Vec3, material_id: u8) {
        self.boxes.push(Box3D::new(center, half_size, material_id));
    }
    
    
    pub fn add_light(&mut self, position: Vec3, color: Vec3, intensity: f32) {
        self.lights.push(Acq { position, color, intensity });
    }
    
    
    pub fn add_material(&mut self, ayb: Material) -> u8 {
        let id = self.materials.len() as u8;
        self.materials.push(ayb);
        id
    }
    
    
    fn get_ray(&self, x: usize, y: usize) -> Ray {
        let bqh = self.width as f32 / self.height as f32;
        let lya = self.fov * 3.14159 / 180.0;
        let jll = pdb(lya / 2.0);
        
        
        let p = (2.0 * (x as f32 + 0.5) / self.width as f32 - 1.0) * jll * bqh;
        let o = (1.0 - 2.0 * (y as f32 + 0.5) / self.height as f32) * jll;
        
        
        let forward = (self.camera_target - self.camera_pos).normalize();
        let right = forward.cross(Vec3::Np).normalize();
        let up = right.cross(forward);
        
        let direction = (forward + right * p + up * o).normalize();
        
        Ray::new(self.camera_pos, direction)
    }
    
    
    fn trace(&self, ob: &Ray) -> HitInfo {
        let mut chc = HitInfo::cng();
        
        for sphere in &self.spheres {
            let hit = sphere.intersect(ob);
            if hit.hit() && hit.t < chc.t {
                chc = hit;
            }
        }
        
        for plane in &self.planes {
            let hit = plane.intersect(ob);
            if hit.hit() && hit.t < chc.t {
                chc = hit;
            }
        }
        
        for box3d in &self.boxes {
            let hit = box3d.intersect(ob);
            if hit.hit() && hit.t < chc.t {
                chc = hit;
            }
        }
        
        chc
    }
    
    
    fn in_shadow(&self, point: Vec3, light_pos: Vec3) -> f32 {
        let dfp = light_pos - point;
        let byu = dfp.length();
        let ob = Ray::new(point + dfp.normalize() * 0.01, dfp.normalize());
        
        let hit = self.trace(&ob);
        if hit.hit() && hit.t < byu {
            0.3 
        } else {
            1.0 
        }
    }
    
    
    fn shade(&self, hit: &HitInfo, ob: &Ray, depth: u8) -> Vec3 {
        if !hit.hit() {
            
            let t = (ob.direction.y + 1.0) * 0.5;
            return Vec3::new(0.0, 0.02, 0.05).lerp(Vec3::new(0.0, 0.1, 0.15), t);
        }
        
        let ayb = &self.materials[hit.material_id as usize % self.materials.len()];
        let mut color = ayb.color * ayb.ambient;
        
        
        if ayb.emission > 0.0 {
            color = color + ayb.color * ayb.emission;
        }
        
        
        for light in &self.lights {
            let dfp = (light.position - hit.point).normalize();
            let shadow = self.in_shadow(hit.point, light.position);
            
            
            let jr = hit.normal.dot(dfp).max(0.0);
            let leo = ayb.color * light.color * (jr * ayb.diffuse * shadow * light.intensity);
            color = color + leo;
            
            
            let eyg = (-dfp).reflect(hit.normal);
            let ye = (-ob.direction).dot(eyg).max(0.0);
            let ouo = nwj(ye, ayb.shininess) * ayb.specular * shadow;
            color = color + light.color * (ouo * light.intensity);
        }
        
        
        if ayb.reflectivity > 0.0 && depth < self.max_bounces {
            let eyg = ob.direction.reflect(hit.normal);
            let izc = Ray::new(hit.point + eyg * 0.01, eyg);
            let oec = self.trace(&izc);
            let oea = self.shade(&oec, &izc, depth + 1);
            color = color.lerp(oea, ayb.reflectivity);
        }
        
        
        Vec3::new(
            color.x.min(1.0),
            color.y.min(1.0),
            color.z.min(1.0),
        )
    }
    
    
    pub fn render(&self) -> Vec<u32> {
        let mut buffer = vec![0u32; self.width * self.height];
        
        for y in 0..self.height {
            for x in 0..self.width {
                let ob = self.get_ray(x, y);
                let hit = self.trace(&ob);
                let color = self.shade(&hit, &ob, 0);
                
                
                let r = (color.x * 255.0) as u32;
                let g = (color.y * 255.0) as u32;
                let b = (color.z * 255.0) as u32;
                buffer[y * self.width + x] = 0xFF000000 | (r << 16) | (g << 8) | b;
            }
        }
        
        buffer
    }
    
    
    pub fn qtz(&self, num_layers: usize) -> Vec<Vec<u8>> {
        let mut layers = Vec::with_capacity(num_layers);
        
        for _ in 0..num_layers {
            layers.push(vec![0u8; self.width * self.height]);
        }
        
        
        for y in 0..self.height {
            for x in 0..self.width {
                let ob = self.get_ray(x, y);
                let hit = self.trace(&ob);
                
                if hit.hit() {
                    
                    let pwi = ((hit.t - 1.0) / 10.0).max(0.0).min(1.0);
                    let xv = (pwi * (num_layers - 1) as f32) as usize;
                    
                    
                    let color = self.shade(&hit, &ob, 0);
                    let intensity = ((color.x + color.y + color.z) / 3.0 * 255.0) as u8;
                    
                    let idx = y * self.width + x;
                    if xv < num_layers {
                        layers[xv][idx] = layers[xv][idx].saturating_add(intensity);
                    }
                }
            }
        }
        
        layers
    }
    
    
    pub fn update(&mut self, delta_time: f32) {
        self.time += delta_time;
    }
    
    
    pub fn setup_dna_scene(&mut self) {
        self.clear_scene();
        
        
        let gaz = self.add_material(Material {
            color: Vec3::new(0.0, 1.0, 0.5),
            ambient: 0.1,
            diffuse: 0.5,
            specular: 0.9,
            shininess: 64.0,
            reflectivity: 0.2,
            emission: 0.3,
        });
        
        
        let ckm = 4.0;
        let radius = 1.0;
        let bac = 3.0;
        let segments = 30;
        
        for i in 0..segments {
            let t = i as f32 / segments as f32;
            let y = -ckm / 2.0 + t * ckm;
            let cc = t * bac * 2.0 * 3.14159 + self.time;
            
            
            let x1 = radius * fon(cc);
            let po = radius * dzp(cc);
            self.add_sphere(Vec3::new(x1, y, po), 0.1, gaz);
            
            
            let x2 = radius * fon(cc + 3.14159);
            let qt = radius * dzp(cc + 3.14159);
            self.add_sphere(Vec3::new(x2, y, qt), 0.1, gaz);
            
            
            if i % 3 == 0 {
                self.add_sphere(Vec3::new((x1 + x2) / 2.0, y, (po + qt) / 2.0), 0.08, gaz);
            }
        }
        
        
        self.add_light(Vec3::new(5.0, 5.0, -5.0), Vec3::new(0.0, 1.0, 0.5), 1.2);
        self.add_light(Vec3::new(-5.0, 3.0, -3.0), Vec3::new(0.5, 0.0, 1.0), 0.8);
        
        
        self.camera_pos = Vec3::new(0.0, 0.0, -6.0);
        self.camera_target = Vec3::Bk;
    }
    
    
    pub fn setup_spheres_scene(&mut self) {
        self.clear_scene();
        
        
        let ncc = self.add_material(Material::frd());
        let ncf = self.add_material(Material {
            color: Vec3::new(1.0, 0.0, 0.8),
            ..Material::frd()
        });
        let nce = self.add_material(Material::nfc(1.0, 0.8, 0.2));
        let ncd = self.add_material(Material::nco(0.1, 0.1, 0.15));
        
        
        let pwb = dzp(self.time) * 0.3;
        self.add_sphere(Vec3::new(0.0, pwb, 0.0), 0.8, ncc);
        
        
        for i in 0..6 {
            let cc = (i as f32 / 6.0) * 2.0 * 3.14159 + self.time * 0.5;
            let dvy = 2.0;
            let x = dvy * fon(cc);
            let z = dvy * dzp(cc);
            let y = dzp(self.time + i as f32) * 0.4;
            
            let ggo = if i % 2 == 0 { ncf } else { nce };
            self.add_sphere(Vec3::new(x, y, z), 0.3, ggo);
        }
        
        
        self.add_plane(Vec3::new(0.0, -1.5, 0.0), Vec3::Np, ncd);
        
        
        self.add_light(Vec3::new(4.0, 6.0, -4.0), Vec3::new(1.0, 1.0, 0.9), 1.0);
        self.add_light(Vec3::new(-3.0, 4.0, 2.0), Vec3::new(0.2, 0.4, 1.0), 0.6);
        
        
        self.camera_pos = Vec3::new(0.0, 2.0, -5.0);
        self.camera_target = Vec3::Bk;
    }
}


fn nwj(base: f32, afe: f32) -> f32 {
    
    if afe <= 1.0 { return base; }
    if afe <= 2.0 { return base * base; }
    if afe <= 4.0 { return base * base * base * base; }
    if afe <= 8.0 { 
        let bxc = base * base * base * base;
        return bxc * bxc;
    }
    if afe <= 16.0 {
        let bxc = base * base * base * base;
        return bxc * bxc * bxc * bxc;
    }
    
    let jrt = base * base * base * base * base * base * base * base;
    jrt * jrt
}
