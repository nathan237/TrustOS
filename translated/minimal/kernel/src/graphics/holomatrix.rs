










use alloc::vec::Vec;
use alloc::vec;
use spin::Mutex;
use core::sync::atomic::{AtomicU8, AtomicBool, Ordering};






static VF_: AtomicBool = AtomicBool::new(true);


static AXZ_: AtomicU8 = AtomicU8::new(5);


pub fn lq() -> bool {
    VF_.load(Ordering::Relaxed)
}


pub fn set_enabled(enabled: bool) {
    VF_.store(enabled, Ordering::Relaxed);
    crate::serial_println!("[HOLO] HoloMatrix: {}", if enabled { "ENABLED" } else { "DISABLED" });
}


pub fn pkp() -> bool {
    let current = VF_.load(Ordering::Relaxed);
    VF_.store(!current, Ordering::Relaxed);
    crate::serial_println!("[HOLO] HoloMatrix: {}", if !current { "ENABLED" } else { "DISABLED" });
    !current
}


pub fn dqr() -> HoloScene {
    HoloScene::enm(AXZ_.load(Ordering::Relaxed))
}


pub fn set_scene(scene: HoloScene) {
    AXZ_.store(scene.to_index(), Ordering::Relaxed);
    crate::serial_println!("[HOLO] Scene: {}", scene.name());
}


pub fn nkf() -> HoloScene {
    let current = dqr();
    let next = current.next();
    set_scene(next);
    next
}



pub struct HoloMatrix {
    
    pub width: usize,
    
    pub height: usize,
    
    pub num_layers: usize,
    
    pub layers: Vec<Vec<u8>>,
    
    pub layer_depths: Vec<f32>,
    
    pub rotation_x: f32,
    pub rotation_y: f32,
    pub rotation_z: f32,
    
    pub time: f32,
}


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
    
    
    pub fn rotate_x(self, cc: f32) -> Self {
        let vg = anx(cc);
        let vt = aip(cc);
        Self {
            x: self.x,
            y: self.y * vg - self.z * vt,
            z: self.y * vt + self.z * vg,
        }
    }
    
    
    pub fn rotate_y(self, cc: f32) -> Self {
        let vg = anx(cc);
        let vt = aip(cc);
        Self {
            x: self.x * vg + self.z * vt,
            y: self.y,
            z: -self.x * vt + self.z * vg,
        }
    }
    
    
    pub fn rotate_z(self, cc: f32) -> Self {
        let vg = anx(cc);
        let vt = aip(cc);
        Self {
            x: self.x * vg - self.y * vt,
            y: self.x * vt + self.y * vg,
            z: self.z,
        }
    }
}


fn aip(x: f32) -> f32 { crate::math::eu(x) }


fn anx(x: f32) -> f32 { crate::math::hr(x) }


#[inline]
pub fn azr(x: f32) -> f32 { crate::math::eu(x) }


#[inline]
pub fn byi(x: f32) -> f32 { crate::math::hr(x) }


fn sqrt_approx(x: f32) -> f32 { crate::math::ra(x) }

impl HoloMatrix {
    
    pub fn new(width: usize, height: usize, num_layers: usize) -> Self {
        let mut layers = Vec::with_capacity(num_layers);
        let mut layer_depths = Vec::with_capacity(num_layers);
        
        for i in 0..num_layers {
            layers.push(vec![0u8; width * height]);
            
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
    
    
    pub fn clear(&mut self) {
        for bj in &mut self.layers {
            bj.fill(0);
        }
    }
    
    
    #[inline]
    pub fn set_point(&mut self, bj: usize, x: i32, y: i32, intensity: u8) {
        if bj < self.num_layers 
            && x >= 0 && (x as usize) < self.width 
            && y >= 0 && (y as usize) < self.height 
        {
            let idx = y as usize * self.width + x as usize;
            
            let current = self.layers[bj][idx] as u16;
            self.layers[bj][idx] = (current + intensity as u16).min(255) as u8;
        }
    }
    
    
    pub fn draw_sphere(&mut self, cx: f32, u: f32, mj: f32, radius: f32, intensity: u8) {
        let gsx = self.width as f32 / 2.0;
        let gsy = self.height as f32 / 2.0;
        
        for xv in 0..self.num_layers {
            let dai = self.layer_depths[xv];
            
            
            let dz = dai - mj;
            
            
            if dz.abs() < radius {
                
                let kkp = sqrt_approx(radius * radius - dz * dz);
                
                
                let kko = (kkp * self.width as f32 / 2.0) as i32;
                let nsy = (cx * self.width as f32 / 2.0 + gsx) as i32;
                let nsz = (u * self.height as f32 / 2.0 + gsy) as i32;
                
                
                let alh = 1.0 - dz.abs() / radius;
                let mxg = (intensity as f32 * alh) as u8;
                
                self.draw_circle_layer(xv, nsy, nsz, kko, mxg);
            }
        }
    }
    
    
    fn draw_circle_layer(&mut self, bj: usize, cx: i32, u: i32, radius: i32, intensity: u8) {
        let amn = radius * radius;
        
        
        for ad in -radius..=radius {
            let bzb = sqrt_approx((amn - ad * ad) as f32) as i32;
            for dx in -bzb..=bzb {
                let wz = dx * dx + ad * ad;
                let fsm = sqrt_approx(wz as f32) / radius as f32;
                
                
                let lob = if fsm > 0.7 {
                    1.0 + (fsm - 0.7) * 2.0
                } else {
                    0.5 + fsm * 0.5
                };
                
                let nzy = (intensity as f32 * lob).min(255.0) as u8;
                self.set_point(bj, cx + dx, u + ad, nzy);
            }
        }
    }
    
    
    pub fn draw_cube(&mut self, cx: f32, u: f32, mj: f32, size: f32, intensity: u8) {
        let cw = size / 2.0;
        
        
        let vertices = [
            Point3D::new(-cw, -cw, -cw),
            Point3D::new( cw, -cw, -cw),
            Point3D::new( cw,  cw, -cw),
            Point3D::new(-cw,  cw, -cw),
            Point3D::new(-cw, -cw,  cw),
            Point3D::new( cw, -cw,  cw),
            Point3D::new( cw,  cw,  cw),
            Point3D::new(-cw,  cw,  cw),
        ];
        
        
        let auu: Vec<Point3D> = vertices.iter().map(|v| {
            v.rotate_x(self.rotation_x)
             .rotate_y(self.rotation_y)
             .rotate_z(self.rotation_z)
        }).collect();
        
        
        let ceq: Vec<Point3D> = auu.iter().map(|v| {
            Point3D::new(v.x + cx, v.y + u, v.z + mj)
        }).collect();
        
        
        let edges = [
            (0, 1), (1, 2), (2, 3), (3, 0), 
            (4, 5), (5, 6), (6, 7), (7, 4), 
            (0, 4), (1, 5), (2, 6), (3, 7), 
        ];
        
        
        for (i1, i2) in &edges {
            self.draw_line_3d(&ceq[*i1], &ceq[*i2], intensity);
        }
    }
    
    
    pub fn draw_line_3d(&mut self, gw: &Point3D, gn: &Point3D, intensity: u8) {
        let gsx = self.width as f32 / 2.0;
        let gsy = self.height as f32 / 2.0;
        
        
        let dx = gn.x - gw.x;
        let ad = gn.y - gw.y;
        let dz = gn.z - gw.z;
        let length = sqrt_approx(dx * dx + ad * ad + dz * dz);
        let steps = (length * 50.0) as usize + 1;
        
        for step in 0..=steps {
            let t = step as f32 / steps as f32;
            let p = gw.x + dx * t;
            let o = gw.y + ad * t;
            let aos = gw.z + dz * t;
            
            
            let dai = (aos + 1.0) / 2.0; 
            if dai >= 0.0 && dai <= 1.0 {
                let xv = ((dai * (self.num_layers - 1) as f32) as usize).min(self.num_layers - 1);
                
                
                let am = (p * self.width as f32 / 2.5 + gsx) as i32;
                let ak = (o * self.height as f32 / 2.5 + gsy) as i32;
                
                
                let hrm = ((1.0 - dai * 0.5) * intensity as f32) as u8;
                
                
                for d in -1..=1 {
                    self.set_point(xv, am + d, ak, hrm);
                    self.set_point(xv, am, ak + d, hrm);
                }
            }
        }
    }
    
    
    pub fn draw_torus(&mut self, cx: f32, u: f32, mj: f32, bcm: f32, aro: f32, intensity: u8) {
        let gtp = 24;
        let gtq = 12;
        
        for i in 0..gtp {
            let jmh = (i as f32 / gtp as f32) * 2.0 * 3.14159;
            let piu = ((i + 1) as f32 / gtp as f32) * 2.0 * 3.14159;
            
            for ay in 0..gtq {
                let iup = (ay as f32 / gtq as f32) * 2.0 * 3.14159;
                let nug = ((ay + 1) as f32 / gtq as f32) * 2.0 * 3.14159;
                
                
                let gw = self.torus_point(cx, u, mj, bcm, aro, jmh, iup);
                let gn = self.torus_point(cx, u, mj, bcm, aro, jmh, nug);
                let aih = self.torus_point(cx, u, mj, bcm, aro, piu, iup);
                
                
                self.draw_line_3d(&gw, &gn, intensity / 2);
                self.draw_line_3d(&gw, &aih, intensity / 2);
            }
        }
    }
    
    fn torus_point(&self, cx: f32, u: f32, mj: f32, bcm: f32, aro: f32, acz: f32, aij: f32) -> Point3D {
        let x = (bcm + aro * anx(aij)) * anx(acz);
        let y = (bcm + aro * anx(aij)) * aip(acz);
        let z = aro * aip(aij);
        
        
        let aa = Point3D::new(x, y, z)
            .rotate_x(self.rotation_x)
            .rotate_y(self.rotation_y);
            
        Point3D::new(aa.x + cx, aa.y + u, aa.z + mj)
    }
    
    
    pub fn draw_grid(&mut self, cfq: f32, eon: f32, cells: i32, intensity: u8) {
        let cw = eon / 2.0;
        let cell_size = eon / cells as f32;
        
        
        for i in 0..=cells {
            let z = -cw + i as f32 * cell_size;
            let gw = Point3D::new(-cw, cfq, z)
                .rotate_y(self.rotation_y);
            let gn = Point3D::new(cw, cfq, z)
                .rotate_y(self.rotation_y);
            self.draw_line_3d(&gw, &gn, intensity);
        }
        
        
        for i in 0..=cells {
            let x = -cw + i as f32 * cell_size;
            let gw = Point3D::new(x, cfq, -cw)
                .rotate_y(self.rotation_y);
            let gn = Point3D::new(x, cfq, cw)
                .rotate_y(self.rotation_y);
            self.draw_line_3d(&gw, &gn, intensity);
        }
    }
    
    
    
    pub fn composite(&self, qf: u32, aog: u32) -> Vec<u32> {
        let mut output = vec![qf; self.width * self.height];
        
        
        let cag = ((aog >> 16) & 0xFF) as u32;
        let bgq = ((aog >> 8) & 0xFF) as u32;
        let cab = (aog & 0xFF) as u32;
        
        
        for (xv, bj) in self.layers.iter().enumerate().rev() {
            let depth = self.layer_depths[xv];
            
            
            let mxh = 0.3 + 0.7 * (1.0 - depth);
            
            for y in 0..self.height {
                for x in 0..self.width {
                    let intensity = bj[y * self.width + x];
                    
                    if intensity > 0 {
                        let aza = y * self.width + x;
                        let current = output[aza];
                        
                        
                        let alg = ((current >> 16) & 0xFF) as u32;
                        let ahp = ((current >> 8) & 0xFF) as u32;
                        let cb = (current & 0xFF) as u32;
                        
                        
                        let alpha = (intensity as f32 / 255.0) * mxh;
                        let nr = (alg as f32 * (1.0 - alpha) + cag as f32 * alpha) as u32;
                        let ayn = (ahp as f32 * (1.0 - alpha) + bgq as f32 * alpha) as u32;
                        let ayj = (cb as f32 * (1.0 - alpha) + cab as f32 * alpha) as u32;
                        
                        output[aza] = 0xFF000000 | (nr.min(255) << 16) | (ayn.min(255) << 8) | ayj.min(255);
                    }
                }
            }
        }
        
        output
    }
    
    
    pub fn update(&mut self, delta_time: f32) {
        self.time += delta_time;
        self.rotation_y += delta_time * 0.5;  
        self.rotation_x = aip(self.time * 0.3) * 0.3;  
    }
    
    
    pub fn bdi(&mut self, scene: HoloScene) {
        self.clear();
        
        match scene {
            HoloScene::RotatingCube => {
                self.draw_cube(0.0, 0.0, 0.5, 0.6, 200);
            },
            HoloScene::SpherePulse => {
                let kq = (aip(self.time * 2.0) + 1.0) / 2.0;
                let radius = 0.2 + kq * 0.15;
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
                
                self.draw_cube(-0.4, 0.0, 0.5, 0.25, 150);
                self.draw_sphere(0.4, 0.0, 0.5, 0.2, 180);
                self.draw_torus(0.0, 0.3, 0.5, 0.2, 0.08, 120);
            },
            HoloScene::DNA => {
                
                self.draw_dna_helix(0.0, 0.0, 0.5, 100);
            },
            HoloScene::RayTracedSpheres | HoloScene::RayTracedDNA => {
                
                
                self.draw_sphere(0.0, 0.0, 0.5, 0.3, 150);
            },
        }
    }
    
    
    fn draw_dna_helix(&mut self, cx: f32, u: f32, mj: f32, intensity: u8) {
        let ckm = 1.4;  
        let radius = 0.32;       
        let bac = 3.5;         
        let segments = 100;      
        
        
        for i in 0..segments {
            let t = i as f32 / segments as f32;
            let y = -ckm / 2.0 + t * ckm;
            let cc = t * bac * 2.0 * 3.14159 + self.time;
            
            
            let x1 = radius * anx(cc);
            let po = radius * aip(cc);
            
            
            let x2 = radius * anx(cc + 3.14159);
            let qt = radius * aip(cc + 3.14159);
            
            
            let gln = Point3D::new(x1, y, po * 0.5)
                .rotate_x(self.rotation_x * 0.5)
                .rotate_y(self.rotation_y * 0.3);
            let glp = Point3D::new(x2, y, qt * 0.5)
                .rotate_x(self.rotation_x * 0.5)
                .rotate_y(self.rotation_y * 0.3);
            
            let gw = Point3D::new(gln.x + cx, gln.y + u, gln.z + mj);
            let gn = Point3D::new(glp.x + cx, glp.y + u, glp.z + mj);
            
            
            if i < segments - 1 {
                let np = (i + 1) as f32 / segments as f32;
                let y2 = -ckm / 2.0 + np * ckm;
                let efl = np * bac * 2.0 * 3.14159 + self.time;
                
                let glm = Point3D::new(radius * anx(efl), y2, radius * aip(efl) * 0.5)
                    .rotate_x(self.rotation_x * 0.5)
                    .rotate_y(self.rotation_y * 0.3);
                let glo = Point3D::new(radius * anx(efl + 3.14159), y2, radius * aip(efl + 3.14159) * 0.5)
                    .rotate_x(self.rotation_x * 0.5)
                    .rotate_y(self.rotation_y * 0.3);
                
                let now = Point3D::new(glm.x + cx, glm.y + u, glm.z + mj);
                let nox = Point3D::new(glo.x + cx, glo.y + u, glo.z + mj);
                
                
                self.draw_line_3d(&gw, &now, intensity);
                self.draw_line_3d(&gn, &nox, intensity);
            }
            
            
            if i % 10 == 0 {
                self.draw_phosphate_group(gw.x, gw.y, gw.z, intensity);
                self.draw_phosphate_group(gn.x, gn.y, gn.z, intensity);
            }
            
            
            if i % 4 == 0 {
                
                self.draw_base_pair(&gw, &gn, intensity, i % 8 == 0);
            }
        }
        
        
        self.draw_floating_particles(cx, u, mj, intensity / 2);
    }
    
    
    fn draw_phosphate_group(&mut self, x: f32, y: f32, z: f32, intensity: u8) {
        
        self.draw_sphere(x, y, z, 0.03, intensity);
    }
    
    
    fn draw_base_pair(&mut self, gw: &Point3D, gn: &Point3D, intensity: u8, is_gc: bool) {
        
        let cx = (gw.x + gn.x) / 2.0;
        let u = (gw.y + gn.y) / 2.0;
        let mj = (gw.z + gn.z) / 2.0;
        
        
        if is_gc {
            
            let dfl = (gn.x - gw.x) / 3.0;
            let dfm = (gn.y - gw.y) / 3.0;
            let dfn = (gn.z - gw.z) / 3.0;
            
            
            self.draw_line_3d(
                &Point3D::new(gw.x + dfl * 0.3, gw.y + dfm * 0.3, gw.z + dfn * 0.3),
                &Point3D::new(gw.x + dfl * 0.7, gw.y + dfm * 0.7, gw.z + dfn * 0.7),
                intensity / 2
            );
            
            self.draw_line_3d(
                &Point3D::new(cx - dfl * 0.2, u - dfm * 0.2, mj - dfn * 0.2),
                &Point3D::new(cx + dfl * 0.2, u + dfm * 0.2, mj + dfn * 0.2),
                intensity / 2
            );
            
            self.draw_line_3d(
                &Point3D::new(gn.x - dfl * 0.7, gn.y - dfm * 0.7, gn.z - dfn * 0.7),
                &Point3D::new(gn.x - dfl * 0.3, gn.y - dfm * 0.3, gn.z - dfn * 0.3),
                intensity / 2
            );
        } else {
            
            let exk = (gn.x - gw.x) / 4.0;
            let exl = (gn.y - gw.y) / 4.0;
            let exm = (gn.z - gw.z) / 4.0;
            
            self.draw_line_3d(
                &Point3D::new(gw.x + exk * 1.2, gw.y + exl * 1.2, gw.z + exm * 1.2),
                &Point3D::new(gw.x + exk * 1.8, gw.y + exl * 1.8, gw.z + exm * 1.8),
                intensity / 2
            );
            self.draw_line_3d(
                &Point3D::new(gn.x - exk * 1.8, gn.y - exl * 1.8, gn.z - exm * 1.8),
                &Point3D::new(gn.x - exk * 1.2, gn.y - exl * 1.2, gn.z - exm * 1.2),
                intensity / 2
            );
        }
        
        
        self.draw_line_3d(gw, &Point3D::new(cx, u, mj), intensity / 3);
        self.draw_line_3d(&Point3D::new(cx, u, mj), gn, intensity / 3);
    }
    
    
    fn draw_floating_particles(&mut self, cx: f32, u: f32, mj: f32, intensity: u8) {
        
        for i in 0..8 {
            let cc = (i as f32 / 8.0) * 2.0 * 3.14159 + self.time * 0.7;
            let hdd = aip(self.time * 1.5 + i as f32 * 0.789) * 0.5;
            let dvy = 0.45 + aip(self.time + i as f32) * 0.1;
            
            let p = cx + dvy * anx(cc);
            let o = u + hdd;
            let aos = mj + dvy * aip(cc) * 0.4;
            
            
            let kq = ((aip(self.time * 2.0 + i as f32 * 1.1) + 1.0) / 2.0 * 0.5 + 0.5) as f32;
            let nrt = (intensity as f32 * kq) as u8;
            
            self.draw_sphere(p, o, aos, 0.02, nrt);
        }
    }
}


#[derive(Clone, Copy, PartialEq)]
pub enum HoloScene {
    RotatingCube,
    SpherePulse,
    Torus,
    GridWithCube,
    MultiShape,
    DNA,
    RayTracedSpheres,  
    RayTracedDNA,      
}

impl HoloScene {
    
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
    
    
    pub fn enm(idx: u8) -> Self {
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
    
    
    pub fn iad(name: &str) -> Option<Self> {
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
    
    
    pub fn juo() -> &'static [&'static str] {
        &["cube", "sphere", "torus", "grid", "multi", "dna", "rt-spheres", "rt-dna"]
    }
    
    
    pub fn is_raytraced(&self) -> bool {
        matches!(self, Self::RayTracedSpheres | Self::RayTracedDNA)
    }
}
