




use micromath::Wo;
use core::ops::{Add, Sub, Mul, Neg};






#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Vec3 {
    pub b: f32,
    pub c: f32,
    pub av: f32,
}

impl Vec3 {
    pub const Dh: Vec3 = Vec3 { b: 0.0, c: 0.0, av: 0.0 };
    pub const Cie: Vec3 = Vec3 { b: 1.0, c: 1.0, av: 1.0 };
    pub const Dma: Vec3 = Vec3 { b: 1.0, c: 0.0, av: 0.0 };
    pub const Dmf: Vec3 = Vec3 { b: 0.0, c: 1.0, av: 0.0 };
    pub const Cqu: Vec3 = Vec3 { b: 0.0, c: 0.0, av: 1.0 };

    #[inline]
    pub const fn new(b: f32, c: f32, av: f32) -> Self {
        Self { b, c, av }
    }

    #[inline]
    pub fn go(&self) -> f32 {
        (self.b * self.b + self.c * self.c + self.av * self.av).ibi()
    }

#[inline]
    pub fn all(&self) -> Self {
        let len = self.go();
        if len > 0.0001 {
            Self {
                b: self.b / len,
                c: self.c / len,
                av: self.av / len,
            }
        } else {
            *self
        }
    }

    #[inline]
    pub fn amb(&self, gq: Vec3) -> f32 {
        self.b * gq.b + self.c * gq.c + self.av * gq.av
    }

    #[inline]
    pub fn bjr(&self, gq: Vec3) -> Self {
        Self {
            b: self.c * gq.av - self.av * gq.c,
            c: self.av * gq.b - self.b * gq.av,
            av: self.b * gq.c - self.c * gq.b,
        }
    }

    #[inline]
    pub fn csb(&self, gq: Vec3, ab: f32) -> Self {
        Self {
            b: self.b + (gq.b - self.b) * ab,
            c: self.c + (gq.c - self.c) * ab,
            av: self.av + (gq.av - self.av) * ab,
        }
    }

    #[inline]
    pub fn bv(&self, e: f32) -> Self {
        Self {
            b: self.b * e,
            c: self.c * e,
            av: self.av * e,
        }
    }
}

impl Add for Vec3 {
    type Dd = Self;
    fn add(self, gq: Self) -> Self {
        Self {
            b: self.b + gq.b,
            c: self.c + gq.c,
            av: self.av + gq.av,
        }
    }
}

impl Sub for Vec3 {
    type Dd = Self;
    fn sub(self, gq: Self) -> Self {
        Self {
            b: self.b - gq.b,
            c: self.c - gq.c,
            av: self.av - gq.av,
        }
    }
}

impl Neg for Vec3 {
    type Dd = Self;
    fn neg(self) -> Self {
        Self {
            b: -self.b,
            c: -self.c,
            av: -self.av,
        }
    }
}

impl Mul<f32> for Vec3 {
    type Dd = Self;
    fn mul(self, e: f32) -> Self {
        Self {
            b: self.b * e,
            c: self.c * e,
            av: self.av * e,
        }
    }
}





#[derive(Clone, Copy, Debug, Default)]
pub struct Vec4 {
    pub b: f32,
    pub c: f32,
    pub av: f32,
    pub d: f32,
}

impl Vec4 {
    pub const fn new(b: f32, c: f32, av: f32, d: f32) -> Self {
        Self { b, c, av, d }
    }

    pub const fn nwk(p: Vec3, d: f32) -> Self {
        Self { b: p.b, c: p.c, av: p.av, d }
    }

    pub fn xip(&self) -> Vec3 {
        if self.d.gp() > 0.0001 {
            Vec3::new(self.b / self.d, self.c / self.d, self.av / self.d)
        } else {
            Vec3::new(self.b, self.c, self.av)
        }
    }
}






#[derive(Clone, Copy, Debug)]
pub struct Mat4 {
    pub ef: [[f32; 4]; 4],
}

impl Mat4 {
    pub const Sx: Mat4 = Mat4 {
        ef: [
            [1.0, 0.0, 0.0, 0.0],
            [0.0, 1.0, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ],
    };

    pub const fn new(ef: [[f32; 4]; 4]) -> Self {
        Self { ef }
    }

    
    pub fn mmx(b: f32, c: f32, av: f32) -> Self {
        Self {
            ef: [
                [1.0, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [b,   c,   av,   1.0],
            ],
        }
    }

    
    pub fn bv(b: f32, c: f32, av: f32) -> Self {
        Self {
            ef: [
                [b,   0.0, 0.0, 0.0],
                [0.0, c,   0.0, 0.0],
                [0.0, 0.0, av,   0.0],
                [0.0, 0.0, 0.0, 1.0],
            ],
        }
    }

    
    pub fn dlk(hg: f32) -> Self {
        let r = hg.cjt();
        let e = hg.ayq();
        Self {
            ef: [
                [1.0, 0.0, 0.0, 0.0],
                [0.0, r,   e,   0.0],
                [0.0, -e,  r,   0.0],
                [0.0, 0.0, 0.0, 1.0],
            ],
        }
    }

    
    pub fn chi(hg: f32) -> Self {
        let r = hg.cjt();
        let e = hg.ayq();
        Self {
            ef: [
                [r,   0.0, -e,  0.0],
                [0.0, 1.0, 0.0, 0.0],
                [e,   0.0, r,   0.0],
                [0.0, 0.0, 0.0, 1.0],
            ],
        }
    }


    pub fn chh(gao: Vec3, hg: f32) -> Self {
        let r = hg.cjt();
        let e = hg.ayq();
        let ab = 1.0 - r;
        let b = gao.b;
        let c = gao.c;
        let av = gao.av;

        Self {
            ef: [
                [ab*b*b + r,     ab*b*c + e*av,   ab*b*av - e*c,   0.0],
                [ab*b*c - e*av,   ab*c*c + r,     ab*c*av + e*b,   0.0],
                [ab*b*av + e*c,   ab*c*av - e*b,   ab*av*av + r,     0.0],
                [0.0,           0.0,           0.0,           1.0],
            ],
        }
    }

    
    pub fn aqf(swj: f32, dyk: f32, bhl: f32, adt: f32) -> Self {
        let bb = 1.0 / (swj / 2.0).mjs();
        let gns = 1.0 / (bhl - adt);

        Self {
            ef: [
                [bb / dyk, 0.0, 0.0, 0.0],
                [0.0, bb, 0.0, 0.0],
                [0.0, 0.0, (adt + bhl) * gns, -1.0],
                [0.0, 0.0, 2.0 * adt * bhl * gns, 0.0],
            ],
        }
    }

    
    pub fn uzk(fd: f32, hw: f32, abm: f32, qc: f32, bhl: f32, adt: f32) -> Self {
        let hxt = hw - fd;
        let idu = qc - abm;
        let hkb = adt - bhl;

        Self {
            ef: [
                [2.0 / hxt, 0.0, 0.0, 0.0],
                [0.0, 2.0 / idu, 0.0, 0.0],
                [0.0, 0.0, -2.0 / hkb, 0.0],
                [-(hw + fd) / hxt, -(qc + abm) / idu, -(adt + bhl) / hkb, 1.0],
            ],
        }
    }

    
    pub fn kxg(fd: f32, hw: f32, abm: f32, qc: f32, bhl: f32, adt: f32) -> Self {
        let hxt = hw - fd;
        let idu = qc - abm;
        let hkb = adt - bhl;
        let gni = 2.0 * bhl;

        Self {
            ef: [
                [gni / hxt, 0.0, 0.0, 0.0],
                [0.0, gni / idu, 0.0, 0.0],
                [(hw + fd) / hxt, (qc + abm) / idu, -(adt + bhl) / hkb, -1.0],
                [0.0, 0.0, -gni * adt / hkb, 0.0],
            ],
        }
    }

    
    pub fn sxq(sy: [f32; 16]) -> Self {
        Self {
            ef: [
                [sy[0], sy[1], sy[2], sy[3]],
                [sy[4], sy[5], sy[6], sy[7]],
                [sy[8], sy[9], sy[10], sy[11]],
                [sy[12], sy[13], sy[14], sy[15]],
            ],
        }
    }

    
    pub fn ljv(ito: Vec3, cd: Vec3, bln: Vec3) -> Self {
        let bb = (cd - ito).all();
        let e = bb.bjr(bln).all();
        let tm = e.bjr(bb);

        Self {
            ef: [
                [e.b, tm.b, -bb.b, 0.0],
                [e.c, tm.c, -bb.c, 0.0],
                [e.av, tm.av, -bb.av, 0.0],
                [-e.amb(ito), -tm.amb(ito), bb.amb(ito), 1.0],
            ],
        }
    }

    
    pub fn mul(&self, gq: &Mat4) -> Self {
        let mut result = [[0.0f32; 4]; 4];

        for a in 0..4 {
            for fb in 0..4 {
                result[a][fb] = self.ef[a][0] * gq.ef[0][fb]
                    + self.ef[a][1] * gq.ef[1][fb]
                    + self.ef[a][2] * gq.ef[2][fb]
                    + self.ef[a][3] * gq.ef[3][fb];
            }
        }

        Self { ef: result }
    }

    
    pub fn pvy(&self, p: Vec4) -> Vec4 {
        Vec4 {
            b: self.ef[0][0] * p.b + self.ef[1][0] * p.c + self.ef[2][0] * p.av + self.ef[3][0] * p.d,
            c: self.ef[0][1] * p.b + self.ef[1][1] * p.c + self.ef[2][1] * p.av + self.ef[3][1] * p.d,
            av: self.ef[0][2] * p.b + self.ef[1][2] * p.c + self.ef[2][2] * p.av + self.ef[3][2] * p.d,
            d: self.ef[0][3] * p.b + self.ef[1][3] * p.c + self.ef[2][3] * p.av + self.ef[3][3] * p.d,
        }
    }

    
    pub fn pvx(&self, p: Vec3) -> Vec3 {
        let cnq = self.pvy(Vec4::nwk(p, 1.0));
        cnq.xip()
    }

}

impl Default for Mat4 {
    fn default() -> Self {
        Self::Sx
    }
}






#[inline]
pub fn kor(rve: f32) -> f32 {
    rve * core::f32::consts::Eu / 180.0
}
