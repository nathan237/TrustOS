










use alloc::vec::Vec;
use alloc::vec;






#[derive(Clone, Copy)]
pub struct Vec3 {
    pub b: f32,
    pub c: f32,
    pub av: f32,
}

impl Vec3 {
    pub const Dh: Vec3 = Vec3 { b: 0.0, c: 0.0, av: 0.0 };
    pub const Cie: Vec3 = Vec3 { b: 1.0, c: 1.0, av: 1.0 };
    pub const Afc: Vec3 = Vec3 { b: 0.0, c: 1.0, av: 0.0 };
    
    #[inline]
    pub fn new(b: f32, c: f32, av: f32) -> Self {
        Self { b, c, av }
    }
    
    #[inline]
    pub fn amb(self, gq: Vec3) -> f32 {
        self.b * gq.b + self.c * gq.c + self.av * gq.av
    }
    
    #[inline]
    pub fn bjr(self, gq: Vec3) -> Vec3 {
        Vec3::new(
            self.c * gq.av - self.av * gq.c,
            self.av * gq.b - self.b * gq.av,
            self.b * gq.c - self.c * gq.b,
        )
    }
    
    #[inline]
    pub fn lil(self) -> f32 {
        self.amb(self)
    }
    
    #[inline]
    pub fn go(self) -> f32 {
        pmn(self.lil())
    }
    
    #[inline]
    pub fn all(self) -> Vec3 {
        let len = self.go();
        if len > 0.0001 {
            self * (1.0 / len)
        } else {
            Vec3::Dh
        }
    }
    
    #[inline]
    pub fn pbb(self, adg: Vec3) -> Vec3 {
        self - adg * (2.0 * self.amb(adg))
    }
    
    #[inline]
    pub fn csb(self, gq: Vec3, ab: f32) -> Vec3 {
        self * (1.0 - ab) + gq * ab
    }
}

impl core::ops::Add for Vec3 {
    type Dd = Vec3;
    fn add(self, bwr: Vec3) -> Vec3 {
        Vec3::new(self.b + bwr.b, self.c + bwr.c, self.av + bwr.av)
    }
}

impl core::ops::Sub for Vec3 {
    type Dd = Vec3;
    fn sub(self, bwr: Vec3) -> Vec3 {
        Vec3::new(self.b - bwr.b, self.c - bwr.c, self.av - bwr.av)
    }
}

impl core::ops::Mul<f32> for Vec3 {
    type Dd = Vec3;
    fn mul(self, bwr: f32) -> Vec3 {
        Vec3::new(self.b * bwr, self.c * bwr, self.av * bwr)
    }
}

impl core::ops::Neg for Vec3 {
    type Dd = Vec3;
    fn neg(self) -> Vec3 {
        Vec3::new(-self.b, -self.c, -self.av)
    }
}

impl core::ops::Mul<Vec3> for Vec3 {
    type Dd = Vec3;
    fn mul(self, bwr: Vec3) -> Vec3 {
        Vec3::new(self.b * bwr.b, self.c * bwr.c, self.av * bwr.av)
    }
}


fn pmn(b: f32) -> f32 { crate::math::ahn(b) }


fn iat(b: f32) -> f32 { crate::math::lz(b) }

fn kkv(b: f32) -> f32 { crate::math::rk(b) }


fn xar(b: f32) -> f32 { crate::math::nsw(b) }






#[derive(Clone, Copy)]
pub struct Ray {
    pub atf: Vec3,
    pub sz: Vec3,
}

impl Ray {
    pub fn new(atf: Vec3, sz: Vec3) -> Self {
        Self { atf, sz: sz.all() }
    }
    
    pub fn aoi(self, ab: f32) -> Vec3 {
        self.atf + self.sz * ab
    }
}


#[derive(Clone, Copy)]
pub struct HitInfo {
    pub ab: f32,           
    pub nl: Vec3,      
    pub adg: Vec3,     
    pub bby: u8,  
}

impl HitInfo {
    pub fn fpc() -> Self {
        Self {
            ab: f32::O,
            nl: Vec3::Dh,
            adg: Vec3::Afc,
            bby: 0,
        }
    }
    
    pub fn agp(&self) -> bool {
        self.ab < f32::O - 1.0
    }
}






pub struct Sphere {
    pub pn: Vec3,
    pub dy: f32,
    pub bby: u8,
}

impl Sphere {
    pub fn new(pn: Vec3, dy: f32, bby: u8) -> Self {
        Self { pn, dy, bby }
    }
    
    
    pub fn hoj(&self, acj: &Ray) -> HitInfo {
        let dtu = acj.atf - self.pn;
        let q = acj.sz.lil();
        let ixm = dtu.amb(acj.sz);
        let r = dtu.lil() - self.dy * self.dy;
        let gew = ixm * ixm - q * r;
        
        if gew < 0.0 {
            return HitInfo::fpc();
        }
        
        let pmo = pmn(gew);
        let mut ab = (-ixm - pmo) / q;
        
        if ab < 0.001 {
            ab = (-ixm + pmo) / q;
            if ab < 0.001 {
                return HitInfo::fpc();
            }
        }
        
        let nl = acj.aoi(ab);
        let adg = (nl - self.pn).all();
        
        HitInfo {
            ab,
            nl,
            adg,
            bby: self.bby,
        }
    }
}


pub struct Plane {
    pub nl: Vec3,
    pub adg: Vec3,
    pub bby: u8,
}

impl Plane {
    pub fn new(nl: Vec3, adg: Vec3, bby: u8) -> Self {
        Self { nl, adg: adg.all(), bby }
    }
    
    pub fn hoj(&self, acj: &Ray) -> HitInfo {
        let gel = self.adg.amb(acj.sz);
        
        if gel.gp() < 0.0001 {
            return HitInfo::fpc();
        }
        
        let ab = (self.nl - acj.atf).amb(self.adg) / gel;
        
        if ab < 0.001 {
            return HitInfo::fpc();
        }
        
        HitInfo {
            ab,
            nl: acj.aoi(ab),
            adg: self.adg,
            bby: self.bby,
        }
    }
}


pub struct Box3D {
    pub v: Vec3,
    pub am: Vec3,
    pub bby: u8,
}

impl Box3D {
    pub fn new(pn: Vec3, ixn: Vec3, bby: u8) -> Self {
        Self {
            v: pn - ixn,
            am: pn + ixn,
            bby,
        }
    }
    
    pub fn hoj(&self, acj: &Ray) -> HitInfo {
        let cla = Vec3::new(
            1.0 / acj.sz.b,
            1.0 / acj.sz.c,
            1.0 / acj.sz.av,
        );
        
        let aax = (self.v.b - acj.atf.b) * cla.b;
        let aco = (self.am.b - acj.atf.b) * cla.b;
        let bcx = (self.v.c - acj.atf.c) * cla.c;
        let dcl = (self.am.c - acj.atf.c) * cla.c;
        let eji = (self.v.av - acj.atf.av) * cla.av;
        let ejj = (self.am.av - acj.atf.av) * cla.av;
        
        let mlh = aax.v(aco).am(bcx.v(dcl)).am(eji.v(ejj));
        let mlg = aax.am(aco).v(bcx.am(dcl)).v(eji.am(ejj));
        
        if mlg < 0.0 || mlh > mlg {
            return HitInfo::fpc();
        }
        
        let ab = if mlh < 0.001 { mlg } else { mlh };
        if ab < 0.001 {
            return HitInfo::fpc();
        }
        
        let nl = acj.aoi(ab);
        
        
        let pn = (self.v + self.am) * 0.5;
        let ljm = nl - pn;
        let iv = (self.am - self.v) * 0.5;
        
        let kde = 1.0001;
        let adg = Vec3::new(
            (ljm.b / iv.b * kde) as i32 as f32,
            (ljm.c / iv.c * kde) as i32 as f32,
            (ljm.av / iv.av * kde) as i32 as f32,
        ).all();
        
        HitInfo {
            ab,
            nl,
            adg,
            bby: self.bby,
        }
    }
}






#[derive(Clone, Copy)]
pub struct Material {
    pub s: Vec3,       
    pub cvo: f32,      
    pub cpz: f32,      
    pub amv: f32,     
    pub gsm: f32,    
    pub fsm: f32, 
    pub fho: f32,     
}

impl Material {
    pub const fn koq() -> Self {
        Self {
            s: Vec3 { b: 0.0, c: 1.0, av: 0.4 }, 
            cvo: 0.1,
            cpz: 0.6,
            amv: 0.8,
            gsm: 32.0,
            fsm: 0.3,
            fho: 0.2,
        }
    }
    
    pub const fn yox(m: f32, at: f32, o: f32) -> Self {
        Self {
            s: Vec3 { b: m, c: at, av: o },
            cvo: 0.0,
            cpz: 0.0,
            amv: 0.0,
            gsm: 1.0,
            fsm: 0.0,
            fho: 1.0,
        }
    }
    
    pub const fn ukp(m: f32, at: f32, o: f32) -> Self {
        Self {
            s: Vec3 { b: m, c: at, av: o },
            cvo: 0.15,
            cpz: 0.85,
            amv: 0.1,
            gsm: 8.0,
            fsm: 0.0,
            fho: 0.0,
        }
    }
    
    pub const fn unv(m: f32, at: f32, o: f32) -> Self {
        Self {
            s: Vec3 { b: m, c: at, av: o },
            cvo: 0.05,
            cpz: 0.3,
            amv: 1.0,
            gsm: 128.0,
            fsm: 0.7,
            fho: 0.0,
        }
    }
}






pub struct Bpe {
    pub qf: Vec3,
    pub s: Vec3,
    pub hj: f32,
}


pub struct RayTracer {
    pub z: usize,
    pub ac: usize,
    pub jqz: Vec<Sphere>,
    pub hvf: Vec<Plane>,
    pub ime: Vec<Box3D>,
    pub hqu: Vec<Material>,
    pub jdm: Vec<Bpe>,
    pub hce: Vec3,
    pub ims: Vec3,
    pub ckm: f32,
    pub time: f32,
    pub oln: u8,
}

impl RayTracer {
    pub fn new(z: usize, ac: usize) -> Self {
        Self {
            z,
            ac,
            jqz: Vec::new(),
            hvf: Vec::new(),
            ime: Vec::new(),
            hqu: vec![Material::koq()],
            jdm: Vec::new(),
            hce: Vec3::new(0.0, 0.0, -3.0),
            ims: Vec3::Dh,
            ckm: 60.0,
            time: 0.0,
            oln: 2,
        }
    }
    
    
    pub fn ndh(&mut self) {
        self.jqz.clear();
        self.hvf.clear();
        self.ime.clear();
        self.jdm.clear();
    }
    
    
    pub fn gxy(&mut self, pn: Vec3, dy: f32, bby: u8) {
        self.jqz.push(Sphere::new(pn, dy, bby));
    }
    
    
    pub fn qfl(&mut self, nl: Vec3, adg: Vec3, bby: u8) {
        self.hvf.push(Plane::new(nl, adg, bby));
    }
    
    
    pub fn coi(&mut self, pn: Vec3, ixn: Vec3, bby: u8) {
        self.ime.push(Box3D::new(pn, ixn, bby));
    }
    
    
    pub fn iiz(&mut self, qf: Vec3, s: Vec3, hj: f32) {
        self.jdm.push(Bpe { qf, s, hj });
    }
    
    
    pub fn gxw(&mut self, csj: Material) -> u8 {
        let ad = self.hqu.len() as u8;
        self.hqu.push(csj);
        ad
    }
    
    
    fn nyl(&self, b: usize, c: usize) -> Ray {
        let dyk = self.z as f32 / self.ac as f32;
        let swi = self.ckm * 3.14159 / 180.0;
        let prv = xar(swi / 2.0);
        
        
        let y = (2.0 * (b as f32 + 0.5) / self.z as f32 - 1.0) * prv * dyk;
        let x = (1.0 - 2.0 * (c as f32 + 0.5) / self.ac as f32) * prv;
        
        
        let fiz = (self.ims - self.hce).all();
        let hw = fiz.bjr(Vec3::Afc).all();
        let bln = hw.bjr(fiz);
        
        let sz = (fiz + hw * y + bln * x).all();
        
        Ray::new(self.hce, sz)
    }
    
    
    fn trace(&self, acj: &Ray) -> HitInfo {
        let mut fey = HitInfo::fpc();
        
        for wrb in &self.jqz {
            let agp = wrb.hoj(acj);
            if agp.agp() && agp.ab < fey.ab {
                fey = agp;
            }
        }
        
        for vio in &self.hvf {
            let agp = vio.hoj(acj);
            if agp.agp() && agp.ab < fey.ab {
                fey = agp;
            }
        }
        
        for qro in &self.ime {
            let agp = qro.hoj(acj);
            if agp.agp() && agp.ab < fey.ab {
                fey = agp;
            }
        }
        
        fey
    }
    
    
    fn tso(&self, nl: Vec3, uen: Vec3) -> f32 {
        let gur = uen - nl;
        let eoy = gur.go();
        let acj = Ray::new(nl + gur.all() * 0.01, gur.all());
        
        let agp = self.trace(&acj);
        if agp.agp() && agp.ab < eoy {
            0.3 
        } else {
            1.0 
        }
    }
    
    
    fn dlr(&self, agp: &HitInfo, acj: &Ray, eo: u8) -> Vec3 {
        if !agp.agp() {
            
            let ab = (acj.sz.c + 1.0) * 0.5;
            return Vec3::new(0.0, 0.02, 0.05).csb(Vec3::new(0.0, 0.1, 0.15), ab);
        }
        
        let csj = &self.hqu[agp.bby as usize % self.hqu.len()];
        let mut s = csj.s * csj.cvo;
        
        
        if csj.fho > 0.0 {
            s = s + csj.s * csj.fho;
        }
        
        
        for light in &self.jdm {
            let gur = (light.qf - agp.nl).all();
            let zc = self.tso(agp.nl, light.qf);
            
            
            let wz = agp.adg.amb(gur).am(0.0);
            let rxl = csj.s * light.s * (wz * csj.cpz * zc * light.hj);
            s = s + rxl;
            
            
            let jlx = (-gur).pbb(agp.adg);
            let avc = (-acj.sz).amb(jlx).am(0.0);
            let wqs = vkg(avc, csj.gsm) * csj.amv * zc;
            s = s + light.s * (wqs * light.hj);
        }
        
        
        if csj.fsm > 0.0 && eo < self.oln {
            let jlx = acj.sz.pbb(agp.adg);
            let pbc = Ray::new(agp.nl + jlx * 0.01, jlx);
            let vtu = self.trace(&pbc);
            let vts = self.dlr(&vtu, &pbc, eo + 1);
            s = s.csb(vts, csj.fsm);
        }
        
        
        Vec3::new(
            s.b.v(1.0),
            s.c.v(1.0),
            s.av.v(1.0),
        )
    }
    
    
    pub fn tj(&self) -> Vec<u32> {
        let mut bi = vec![0u32; self.z * self.ac];
        
        for c in 0..self.ac {
            for b in 0..self.z {
                let acj = self.nyl(b, c);
                let agp = self.trace(&acj);
                let s = self.dlr(&agp, &acj, 0);
                
                
                let m = (s.b * 255.0) as u32;
                let at = (s.c * 255.0) as u32;
                let o = (s.av * 255.0) as u32;
                bi[c * self.z + b] = 0xFF000000 | (m << 16) | (at << 8) | o;
            }
        }
        
        bi
    }
    
    
    pub fn zjp(&self, bkq: usize) -> Vec<Vec<u8>> {
        let mut my = Vec::fc(bkq);
        
        for _ in 0..bkq {
            my.push(vec![0u8; self.z * self.ac]);
        }
        
        
        for c in 0..self.ac {
            for b in 0..self.z {
                let acj = self.nyl(b, c);
                let agp = self.trace(&acj);
                
                if agp.agp() {
                    
                    let xxg = ((agp.ab - 1.0) / 10.0).am(0.0).v(1.0);
                    let aup = (xxg * (bkq - 1) as f32) as usize;
                    
                    
                    let s = self.dlr(&agp, &acj, 0);
                    let hj = ((s.b + s.c + s.av) / 3.0 * 255.0) as u8;
                    
                    let w = c * self.z + b;
                    if aup < bkq {
                        my[aup][w] = my[aup][w].akq(hj);
                    }
                }
            }
        }
        
        my
    }
    
    
    pub fn qs(&mut self, iqv: f32) {
        self.time += iqv;
    }
    
    
    pub fn wko(&mut self) {
        self.ndh();
        
        
        let lcl = self.gxw(Material {
            s: Vec3::new(0.0, 1.0, 0.5),
            cvo: 0.1,
            cpz: 0.5,
            amv: 0.9,
            gsm: 64.0,
            fsm: 0.2,
            fho: 0.3,
        });
        
        
        let fkm = 4.0;
        let dy = 1.0;
        let cuy = 3.0;
        let jq = 30;
        
        for a in 0..jq {
            let ab = a as f32 / jq as f32;
            let c = -fkm / 2.0 + ab * fkm;
            let hg = ab * cuy * 2.0 * 3.14159 + self.time;
            
            
            let dn = dy * kkv(hg);
            let aeu = dy * iat(hg);
            self.gxy(Vec3::new(dn, c, aeu), 0.1, lcl);
            
            
            let hy = dy * kkv(hg + 3.14159);
            let ahc = dy * iat(hg + 3.14159);
            self.gxy(Vec3::new(hy, c, ahc), 0.1, lcl);
            
            
            if a % 3 == 0 {
                self.gxy(Vec3::new((dn + hy) / 2.0, c, (aeu + ahc) / 2.0), 0.08, lcl);
            }
        }
        
        
        self.iiz(Vec3::new(5.0, 5.0, -5.0), Vec3::new(0.0, 1.0, 0.5), 1.2);
        self.iiz(Vec3::new(-5.0, 3.0, -3.0), Vec3::new(0.5, 0.0, 1.0), 0.8);
        
        
        self.hce = Vec3::new(0.0, 0.0, -6.0);
        self.ims = Vec3::Dh;
    }
    
    
    pub fn wlk(&mut self) {
        self.ndh();
        
        
        let ukd = self.gxw(Material::koq());
        let ukg = self.gxw(Material {
            s: Vec3::new(1.0, 0.0, 0.8),
            ..Material::koq()
        });
        let ukf = self.gxw(Material::unv(1.0, 0.8, 0.2));
        let uke = self.gxw(Material::ukp(0.1, 0.1, 0.15));
        
        
        let xwx = iat(self.time) * 0.3;
        self.gxy(Vec3::new(0.0, xwx, 0.0), 0.8, ukd);
        
        
        for a in 0..6 {
            let hg = (a as f32 / 6.0) * 2.0 * 3.14159 + self.time * 0.5;
            let htu = 2.0;
            let b = htu * kkv(hg);
            let av = htu * iat(hg);
            let c = iat(self.time + a as f32) * 0.4;
            
            let lkl = if a % 2 == 0 { ukg } else { ukf };
            self.gxy(Vec3::new(b, c, av), 0.3, lkl);
        }
        
        
        self.qfl(Vec3::new(0.0, -1.5, 0.0), Vec3::Afc, uke);
        
        
        self.iiz(Vec3::new(4.0, 6.0, -4.0), Vec3::new(1.0, 1.0, 0.9), 1.0);
        self.iiz(Vec3::new(-3.0, 4.0, 2.0), Vec3::new(0.2, 0.4, 1.0), 0.6);
        
        
        self.hce = Vec3::new(0.0, 2.0, -5.0);
        self.ims = Vec3::Dh;
    }
}


fn vkg(ar: f32, bgz: f32) -> f32 {
    
    if bgz <= 1.0 { return ar; }
    if bgz <= 2.0 { return ar * ar; }
    if bgz <= 4.0 { return ar * ar * ar * ar; }
    if bgz <= 8.0 { 
        let elf = ar * ar * ar * ar;
        return elf * elf;
    }
    if bgz <= 16.0 {
        let elf = ar * ar * ar * ar;
        return elf * elf * elf * elf;
    }
    
    let qak = ar * ar * ar * ar * ar * ar * ar * ar;
    qak * qak
}
