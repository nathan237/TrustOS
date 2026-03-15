










use alloc::vec::Vec;
use alloc::vec;
use spin::Mutex;
use core::sync::atomic::{AtomicU8, AtomicBool, Ordering};






static TX_: AtomicBool = AtomicBool::new(true);


static AVV_: AtomicU8 = AtomicU8::new(5);


pub fn zu() -> bool {
    TX_.load(Ordering::Relaxed)
}


pub fn cuf(iq: bool) {
    TX_.store(iq, Ordering::Relaxed);
    crate::serial_println!("[HOLO] HoloMatrix: {}", if iq { "ENABLED" } else { "DISABLED" });
}


pub fn xiq() -> bool {
    let cv = TX_.load(Ordering::Relaxed);
    TX_.store(!cv, Ordering::Relaxed);
    crate::serial_println!("[HOLO] HoloMatrix: {}", if !cv { "ENABLED" } else { "DISABLED" });
    !cv
}


pub fn hlk() -> HoloScene {
    HoloScene::ivy(AVV_.load(Ordering::Relaxed))
}


pub fn bid(amt: HoloScene) {
    AVV_.store(amt.xik(), Ordering::Relaxed);
    crate::serial_println!("[HOLO] Scene: {}", amt.j());
}


pub fn uum() -> HoloScene {
    let cv = hlk();
    let next = cv.next();
    bid(next);
    next
}



pub struct HoloMatrix {
    
    pub z: usize,
    
    pub ac: usize,
    
    pub bkq: usize,
    
    pub my: Vec<Vec<u8>>,
    
    pub hpu: Vec<f32>,
    
    pub dlk: f32,
    pub chi: f32,
    pub peb: f32,
    
    pub time: f32,
}


#[derive(Clone, Copy)]
pub struct Point3D {
    pub b: f32,
    pub c: f32,
    pub av: f32,
}

impl Point3D {
    pub fn new(b: f32, c: f32, av: f32) -> Self {
        Self { b, c, av }
    }
    
    
    pub fn dvg(self, hg: f32) -> Self {
        let apn = byz(hg);
        let aql = boj(hg);
        Self {
            b: self.b,
            c: self.c * apn - self.av * aql,
            av: self.c * aql + self.av * apn,
        }
    }
    
    
    pub fn cmj(self, hg: f32) -> Self {
        let apn = byz(hg);
        let aql = boj(hg);
        Self {
            b: self.b * apn + self.av * aql,
            c: self.c,
            av: -self.b * aql + self.av * apn,
        }
    }
    
    
    pub fn wad(self, hg: f32) -> Self {
        let apn = byz(hg);
        let aql = boj(hg);
        Self {
            b: self.b * apn - self.c * aql,
            c: self.b * aql + self.c * apn,
            av: self.av,
        }
    }
}


fn boj(b: f32) -> f32 { crate::math::lz(b) }


fn byz(b: f32) -> f32 { crate::math::rk(b) }


#[inline]
pub fn cuh(b: f32) -> f32 { crate::math::lz(b) }


#[inline]
pub fn eob(b: f32) -> f32 { crate::math::rk(b) }


fn bfj(b: f32) -> f32 { crate::math::ahn(b) }

impl HoloMatrix {
    
    pub fn new(z: usize, ac: usize, bkq: usize) -> Self {
        let mut my = Vec::fc(bkq);
        let mut hpu = Vec::fc(bkq);
        
        for a in 0..bkq {
            my.push(vec![0u8; z * ac]);
            
            hpu.push(a as f32 / (bkq - 1) as f32);
        }
        
        Self {
            z,
            ac,
            bkq,
            my,
            hpu,
            dlk: 0.0,
            chi: 0.0,
            peb: 0.0,
            time: 0.0,
        }
    }
    
    
    pub fn clear(&mut self) {
        for fl in &mut self.my {
            fl.vi(0);
        }
    }
    
    
    #[inline]
    pub fn mes(&mut self, fl: usize, b: i32, c: i32, hj: u8) {
        if fl < self.bkq 
            && b >= 0 && (b as usize) < self.z 
            && c >= 0 && (c as usize) < self.ac 
        {
            let w = c as usize * self.z + b as usize;
            
            let cv = self.my[fl][w] as u16;
            self.my[fl][w] = (cv + hj as u16).v(255) as u8;
        }
    }
    
    
    pub fn hgv(&mut self, cx: f32, ae: f32, zr: f32, dy: f32, hj: u8) {
        let mcm = self.z as f32 / 2.0;
        let mcn = self.ac as f32 / 2.0;
        
        for aup in 0..self.bkq {
            let gky = self.hpu[aup];
            
            
            let pt = gky - zr;
            
            
            if pt.gp() < dy {
                
                let raw = bfj(dy * dy - pt * pt);
                
                
                let rav = (raw * self.z as f32 / 2.0) as i32;
                let vfz = (cx * self.z as f32 / 2.0 + mcm) as i32;
                let vga = (ae * self.ac as f32 / 2.0 + mcn) as i32;
                
                
                let btz = 1.0 - pt.gp() / dy;
                let udg = (hj as f32 * btz) as u8;
                
                self.sch(aup, vfz, vga, rav, udg);
            }
        }
    }
    
    
    fn sch(&mut self, fl: usize, cx: i32, ae: i32, dy: i32, hj: u8) {
        let bwl = dy * dy;
        
        
        for bg in -dy..=dy {
            let epm = bfj((bwl - bg * bg) as f32) as i32;
            for dx in -epm..=epm {
                let ass = dx * dx + bg * bg;
                let kqh = bfj(ass as f32) / dy as f32;
                
                
                let siu = if kqh > 0.7 {
                    1.0 + (kqh - 0.7) * 2.0
                } else {
                    0.5 + kqh * 0.5
                };
                
                let vor = (hj as f32 * siu).v(255.0) as u8;
                self.mes(fl, cx + dx, ae + bg, vor);
            }
        }
    }
    
    
    pub fn gfg(&mut self, cx: f32, ae: f32, zr: f32, aw: f32, hj: u8) {
        let iv = aw / 2.0;
        
        
        let lm = [
            Point3D::new(-iv, -iv, -iv),
            Point3D::new( iv, -iv, -iv),
            Point3D::new( iv,  iv, -iv),
            Point3D::new(-iv,  iv, -iv),
            Point3D::new(-iv, -iv,  iv),
            Point3D::new( iv, -iv,  iv),
            Point3D::new( iv,  iv,  iv),
            Point3D::new(-iv,  iv,  iv),
        ];
        
        
        let cmk: Vec<Point3D> = lm.iter().map(|p| {
            p.dvg(self.dlk)
             .cmj(self.chi)
             .wad(self.peb)
        }).collect();
        
        
        let fag: Vec<Point3D> = cmk.iter().map(|p| {
            Point3D::new(p.b + cx, p.c + ae, p.av + zr)
        }).collect();
        
        
        let bu = [
            (0, 1), (1, 2), (2, 3), (3, 0), 
            (4, 5), (5, 6), (6, 7), (7, 4), 
            (0, 4), (1, 5), (2, 6), (3, 7), 
        ];
        
        
        for (hnh, hni) in &bu {
            self.bzg(&fag[*hnh], &fag[*hni], hj);
        }
    }
    
    
    pub fn bzg(&mut self, pr: &Point3D, pf: &Point3D, hj: u8) {
        let mcm = self.z as f32 / 2.0;
        let mcn = self.ac as f32 / 2.0;
        
        
        let dx = pf.b - pr.b;
        let bg = pf.c - pr.c;
        let pt = pf.av - pr.av;
        let go = bfj(dx * dx + bg * bg + pt * pt);
        let au = (go * 50.0) as usize + 1;
        
        for gu in 0..=au {
            let ab = gu as f32 / au as f32;
            let y = pr.b + dx * ab;
            let x = pr.c + bg * ab;
            let cbe = pr.av + pt * ab;
            
            
            let gky = (cbe + 1.0) / 2.0; 
            if gky >= 0.0 && gky <= 1.0 {
                let aup = ((gky * (self.bkq - 1) as f32) as usize).v(self.bkq - 1);
                
                
                let cr = (y * self.z as f32 / 2.5 + mcm) as i32;
                let cq = (x * self.ac as f32 / 2.5 + mcn) as i32;
                
                
                let nkp = ((1.0 - gky * 0.5) * hj as f32) as u8;
                
                
                for bc in -1..=1 {
                    self.mes(aup, cr + bc, cq, nkp);
                    self.mes(aup, cr, cq + bc, nkp);
                }
            }
        }
    }
    
    
    pub fn nnt(&mut self, cx: f32, ae: f32, zr: f32, czl: f32, cge: f32, hj: u8) {
        let mdh = 24;
        let mdi = 12;
        
        for a in 0..mdh {
            let psw = (a as f32 / mdh as f32) * 2.0 * 3.14159;
            let xgh = ((a + 1) as f32 / mdh as f32) * 2.0 * 3.14159;
            
            for fb in 0..mdi {
                let ovj = (fb as f32 / mdi as f32) * 2.0 * 3.14159;
                let vhk = ((fb + 1) as f32 / mdi as f32) * 2.0 * 3.14159;
                
                
                let pr = self.mly(cx, ae, zr, czl, cge, psw, ovj);
                let pf = self.mly(cx, ae, zr, czl, cge, psw, vhk);
                let bnt = self.mly(cx, ae, zr, czl, cge, xgh, ovj);
                
                
                self.bzg(&pr, &pf, hj / 2);
                self.bzg(&pr, &bnt, hj / 2);
            }
        }
    }
    
    fn mly(&self, cx: f32, ae: f32, zr: f32, czl: f32, cge: f32, bdb: f32, bnv: f32) -> Point3D {
        let b = (czl + cge * byz(bnv)) * byz(bdb);
        let c = (czl + cge * byz(bnv)) * boj(bdb);
        let av = cge * boj(bnv);
        
        
        let ai = Point3D::new(b, c, av)
            .dvg(self.dlk)
            .cmj(self.chi);
            
        Point3D::new(ai.b + cx, ai.c + ae, ai.av + zr)
    }
    
    
    pub fn fgx(&mut self, fby: f32, ixh: f32, kgz: i32, hj: u8) {
        let iv = ixh / 2.0;
        let ny = ixh / kgz as f32;
        
        
        for a in 0..=kgz {
            let av = -iv + a as f32 * ny;
            let pr = Point3D::new(-iv, fby, av)
                .cmj(self.chi);
            let pf = Point3D::new(iv, fby, av)
                .cmj(self.chi);
            self.bzg(&pr, &pf, hj);
        }
        
        
        for a in 0..=kgz {
            let b = -iv + a as f32 * ny;
            let pr = Point3D::new(b, fby, -iv)
                .cmj(self.chi);
            let pf = Point3D::new(b, fby, iv)
                .cmj(self.chi);
            self.bzg(&pr, &pf, hj);
        }
    }
    
    
    
    pub fn iov(&self, agg: u32, bzv: u32) -> Vec<u32> {
        let mut an = vec![agg; self.z * self.ac];
        
        
        let erm = ((bzv >> 16) & 0xFF) as u32;
        let dho = ((bzv >> 8) & 0xFF) as u32;
        let eqx = (bzv & 0xFF) as u32;
        
        
        for (aup, fl) in self.my.iter().cf().vv() {
            let eo = self.hpu[aup];
            
            
            let udi = 0.3 + 0.7 * (1.0 - eo);
            
            for c in 0..self.ac {
                for b in 0..self.z {
                    let hj = fl[c * self.z + b];
                    
                    if hj > 0 {
                        let ctj = c * self.z + b;
                        let cv = an[ctj];
                        
                        
                        let btu = ((cv >> 16) & 0xFF) as u32;
                        let bmh = ((cv >> 8) & 0xFF) as u32;
                        let aiv = (cv & 0xFF) as u32;
                        
                        
                        let dw = (hj as f32 / 255.0) * udi;
                        let nr = (btu as f32 * (1.0 - dw) + erm as f32 * dw) as u32;
                        let csu = (bmh as f32 * (1.0 - dw) + dho as f32 * dw) as u32;
                        let csq = (aiv as f32 * (1.0 - dw) + eqx as f32 * dw) as u32;
                        
                        an[ctj] = 0xFF000000 | (nr.v(255) << 16) | (csu.v(255) << 8) | csq.v(255);
                    }
                }
            }
        }
        
        an
    }
    
    
    pub fn qs(&mut self, iqv: f32) {
        self.time += iqv;
        self.chi += iqv * 0.5;  
        self.dlk = boj(self.time * 0.3) * 0.3;  
    }
    
    
    pub fn dbd(&mut self, amt: HoloScene) {
        self.clear();
        
        match amt {
            HoloScene::Jb => {
                self.gfg(0.0, 0.0, 0.5, 0.6, 200);
            },
            HoloScene::Un => {
                let xg = (boj(self.time * 2.0) + 1.0) / 2.0;
                let dy = 0.2 + xg * 0.15;
                self.hgv(0.0, 0.0, 0.5, dy, 180);
            },
            HoloScene::Dr => {
                self.nnt(0.0, 0.0, 0.5, 0.35, 0.12, 150);
            },
            HoloScene::St => {
                self.fgx(-0.4, 1.5, 8, 60);
                self.gfg(0.0, 0.0, 0.5, 0.4, 200);
            },
            HoloScene::Tn => {
                
                self.gfg(-0.4, 0.0, 0.5, 0.25, 150);
                self.hgv(0.4, 0.0, 0.5, 0.2, 180);
                self.nnt(0.0, 0.3, 0.5, 0.2, 0.08, 120);
            },
            HoloScene::Ij => {
                
                self.scq(0.0, 0.0, 0.5, 100);
            },
            HoloScene::Nv | HoloScene::Nu => {
                
                
                self.hgv(0.0, 0.0, 0.5, 0.3, 150);
            },
        }
    }
    
    
    fn scq(&mut self, cx: f32, ae: f32, zr: f32, hj: u8) {
        let fkm = 1.4;  
        let dy = 0.32;       
        let cuy = 3.5;         
        let jq = 100;      
        
        
        for a in 0..jq {
            let ab = a as f32 / jq as f32;
            let c = -fkm / 2.0 + ab * fkm;
            let hg = ab * cuy * 2.0 * 3.14159 + self.time;
            
            
            let dn = dy * byz(hg);
            let aeu = dy * boj(hg);
            
            
            let hy = dy * byz(hg + 3.14159);
            let ahc = dy * boj(hg + 3.14159);
            
            
            let lri = Point3D::new(dn, c, aeu * 0.5)
                .dvg(self.dlk * 0.5)
                .cmj(self.chi * 0.3);
            let lrk = Point3D::new(hy, c, ahc * 0.5)
                .dvg(self.dlk * 0.5)
                .cmj(self.chi * 0.3);
            
            let pr = Point3D::new(lri.b + cx, lri.c + ae, lri.av + zr);
            let pf = Point3D::new(lrk.b + cx, lrk.c + ae, lrk.av + zr);
            
            
            if a < jq - 1 {
                let aco = (a + 1) as f32 / jq as f32;
                let jz = -fkm / 2.0 + aco * fkm;
                let ijs = aco * cuy * 2.0 * 3.14159 + self.time;
                
                let lrh = Point3D::new(dy * byz(ijs), jz, dy * boj(ijs) * 0.5)
                    .dvg(self.dlk * 0.5)
                    .cmj(self.chi * 0.3);
                let lrj = Point3D::new(dy * byz(ijs + 3.14159), jz, dy * boj(ijs + 3.14159) * 0.5)
                    .dvg(self.dlk * 0.5)
                    .cmj(self.chi * 0.3);
                
                let vai = Point3D::new(lrh.b + cx, lrh.c + ae, lrh.av + zr);
                let vaj = Point3D::new(lrj.b + cx, lrj.c + ae, lrj.av + zr);
                
                
                self.bzg(&pr, &vai, hj);
                self.bzg(&pf, &vaj, hj);
            }
            
            
            if a % 10 == 0 {
                self.nnm(pr.b, pr.c, pr.av, hj);
                self.nnm(pf.b, pf.c, pf.av, hj);
            }
            
            
            if a % 4 == 0 {
                
                self.sbu(&pr, &pf, hj, a % 8 == 0);
            }
        }
        
        
        self.scz(cx, ae, zr, hj / 2);
    }
    
    
    fn nnm(&mut self, b: f32, c: f32, av: f32, hj: u8) {
        
        self.hgv(b, c, av, 0.03, hj);
    }
    
    
    fn sbu(&mut self, pr: &Point3D, pf: &Point3D, hj: u8, txo: bool) {
        
        let cx = (pr.b + pf.b) / 2.0;
        let ae = (pr.c + pf.c) / 2.0;
        let zr = (pr.av + pf.av) / 2.0;
        
        
        if txo {
            
            let guj = (pf.b - pr.b) / 3.0;
            let guk = (pf.c - pr.c) / 3.0;
            let gul = (pf.av - pr.av) / 3.0;
            
            
            self.bzg(
                &Point3D::new(pr.b + guj * 0.3, pr.c + guk * 0.3, pr.av + gul * 0.3),
                &Point3D::new(pr.b + guj * 0.7, pr.c + guk * 0.7, pr.av + gul * 0.7),
                hj / 2
            );
            
            self.bzg(
                &Point3D::new(cx - guj * 0.2, ae - guk * 0.2, zr - gul * 0.2),
                &Point3D::new(cx + guj * 0.2, ae + guk * 0.2, zr + gul * 0.2),
                hj / 2
            );
            
            self.bzg(
                &Point3D::new(pf.b - guj * 0.7, pf.c - guk * 0.7, pf.av - gul * 0.7),
                &Point3D::new(pf.b - guj * 0.3, pf.c - guk * 0.3, pf.av - gul * 0.3),
                hj / 2
            );
        } else {
            
            let jks = (pf.b - pr.b) / 4.0;
            let jkt = (pf.c - pr.c) / 4.0;
            let jku = (pf.av - pr.av) / 4.0;
            
            self.bzg(
                &Point3D::new(pr.b + jks * 1.2, pr.c + jkt * 1.2, pr.av + jku * 1.2),
                &Point3D::new(pr.b + jks * 1.8, pr.c + jkt * 1.8, pr.av + jku * 1.8),
                hj / 2
            );
            self.bzg(
                &Point3D::new(pf.b - jks * 1.8, pf.c - jkt * 1.8, pf.av - jku * 1.8),
                &Point3D::new(pf.b - jks * 1.2, pf.c - jkt * 1.2, pf.av - jku * 1.2),
                hj / 2
            );
        }
        
        
        self.bzg(pr, &Point3D::new(cx, ae, zr), hj / 3);
        self.bzg(&Point3D::new(cx, ae, zr), pf, hj / 3);
    }
    
    
    fn scz(&mut self, cx: f32, ae: f32, zr: f32, hj: u8) {
        
        for a in 0..8 {
            let hg = (a as f32 / 8.0) * 2.0 * 3.14159 + self.time * 0.7;
            let mrw = boj(self.time * 1.5 + a as f32 * 0.789) * 0.5;
            let htu = 0.45 + boj(self.time + a as f32) * 0.1;
            
            let y = cx + htu * byz(hg);
            let x = ae + mrw;
            let cbe = zr + htu * boj(hg) * 0.4;
            
            
            let xg = ((boj(self.time * 2.0 + a as f32 * 1.1) + 1.0) / 2.0 * 0.5 + 0.5) as f32;
            let veq = (hj as f32 * xg) as u8;
            
            self.hgv(y, x, cbe, 0.02, veq);
        }
    }
}


#[derive(Clone, Copy, PartialEq)]
pub enum HoloScene {
    Jb,
    Un,
    Dr,
    St,
    Tn,
    Ij,
    Nv,  
    Nu,      
}

impl HoloScene {
    
    pub fn next(self) -> Self {
        match self {
            Self::Jb => Self::Un,
            Self::Un => Self::Dr,
            Self::Dr => Self::St,
            Self::St => Self::Tn,
            Self::Tn => Self::Ij,
            Self::Ij => Self::Nv,
            Self::Nv => Self::Nu,
            Self::Nu => Self::Jb,
        }
    }
    
    
    pub fn j(&self) -> &'static str {
        match self {
            Self::Jb => "Cube",
            Self::Un => "Sphere",
            Self::Dr => "Torus",
            Self::St => "Grid+Cube",
            Self::Tn => "Multi",
            Self::Ij => "DNA",
            Self::Nv => "RT-Spheres",
            Self::Nu => "RT-DNA",
        }
    }
    
    
    pub fn xik(&self) -> u8 {
        match self {
            Self::Jb => 0,
            Self::Un => 1,
            Self::Dr => 2,
            Self::St => 3,
            Self::Tn => 4,
            Self::Ij => 5,
            Self::Nv => 6,
            Self::Nu => 7,
        }
    }
    
    
    pub fn ivy(w: u8) -> Self {
        match w {
            0 => Self::Jb,
            1 => Self::Un,
            2 => Self::Dr,
            3 => Self::St,
            4 => Self::Tn,
            5 => Self::Ij,
            6 => Self::Nv,
            7 => Self::Nu,
            _ => Self::Jb,
        }
    }
    
    
    pub fn nwf(j: &str) -> Option<Self> {
        match j.aqn().as_str() {
            "cube" | "box" => Some(Self::Jb),
            "sphere" | "ball" => Some(Self::Un),
            "torus" | "donut" | "ring" => Some(Self::Dr),
            "grid" | "grid+cube" | "gridcube" => Some(Self::St),
            "multi" | "multiple" | "shapes" => Some(Self::Tn),
            "dna" | "helix" => Some(Self::Ij),
            "rt-spheres" | "rtspheres" | "raytraced" => Some(Self::Nv),
            "rt-dna" | "rtdna" | "raytraced-dna" => Some(Self::Nu),
            _ => None,
        }
    }
    
    
    pub fn qgk() -> &'static [&'static str] {
        &["cube", "sphere", "torus", "grid", "multi", "dna", "rt-spheres", "rt-dna"]
    }
    
    
    pub fn tyr(&self) -> bool {
        oh!(self, Self::Nv | Self::Nu)
    }
}
