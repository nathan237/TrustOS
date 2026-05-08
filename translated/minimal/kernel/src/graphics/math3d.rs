




use micromath::F32Ext;
use core::ops::{Add, Sub, Mul, Neg};






#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Vec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Vec3 {
    pub const Bk: Vec3 = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    pub const Amy: Vec3 = Vec3 { x: 1.0, y: 1.0, z: 1.0 };
    pub const Bfo: Vec3 = Vec3 { x: 1.0, y: 0.0, z: 0.0 };
    pub const Bfr: Vec3 = Vec3 { x: 0.0, y: 1.0, z: 0.0 };
    pub const Ash: Vec3 = Vec3 { x: 0.0, y: 0.0, z: 1.0 };

    #[inline]
    pub const fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }

    #[inline]
    pub fn length(&self) -> f32 {
        (self.x * self.x + self.y * self.y + self.z * self.z).sqrt()
    }

#[inline]
    pub fn normalize(&self) -> Self {
        let len = self.length();
        if len > 0.0001 {
            Self {
                x: self.x / len,
                y: self.y / len,
                z: self.z / len,
            }
        } else {
            *self
        }
    }

    #[inline]
    pub fn dot(&self, other: Vec3) -> f32 {
        self.x * other.x + self.y * other.y + self.z * other.z
    }

    #[inline]
    pub fn cross(&self, other: Vec3) -> Self {
        Self {
            x: self.y * other.z - self.z * other.y,
            y: self.z * other.x - self.x * other.z,
            z: self.x * other.y - self.y * other.x,
        }
    }

    #[inline]
    pub fn lerp(&self, other: Vec3, t: f32) -> Self {
        Self {
            x: self.x + (other.x - self.x) * t,
            y: self.y + (other.y - self.y) * t,
            z: self.z + (other.z - self.z) * t,
        }
    }

    #[inline]
    pub fn scale(&self, j: f32) -> Self {
        Self {
            x: self.x * j,
            y: self.y * j,
            z: self.z * j,
        }
    }
}

impl Add for Vec3 {
    type Output = Self;
    fn add(self, other: Self) -> Self {
        Self {
            x: self.x + other.x,
            y: self.y + other.y,
            z: self.z + other.z,
        }
    }
}

impl Sub for Vec3 {
    type Output = Self;
    fn sub(self, other: Self) -> Self {
        Self {
            x: self.x - other.x,
            y: self.y - other.y,
            z: self.z - other.z,
        }
    }
}

impl Neg for Vec3 {
    type Output = Self;
    fn neg(self) -> Self {
        Self {
            x: -self.x,
            y: -self.y,
            z: -self.z,
        }
    }
}

impl Mul<f32> for Vec3 {
    type Output = Self;
    fn mul(self, j: f32) -> Self {
        Self {
            x: self.x * j,
            y: self.y * j,
            z: self.z * j,
        }
    }
}





#[derive(Clone, Copy, Debug, Default)]
pub struct Vec4 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}

impl Vec4 {
    pub const fn new(x: f32, y: f32, z: f32, w: f32) -> Self {
        Self { x, y, z, w }
    }

    pub const fn iae(v: Vec3, w: f32) -> Self {
        Self { x: v.x, y: v.y, z: v.z, w }
    }

    pub fn to_vec3(&self) -> Vec3 {
        if self.w.abs() > 0.0001 {
            Vec3::new(self.x / self.w, self.y / self.w, self.z / self.w)
        } else {
            Vec3::new(self.x, self.y, self.z)
        }
    }
}






#[derive(Clone, Copy, Debug)]
pub struct Mat4 {
    pub m: [[f32; 4]; 4],
}

impl Mat4 {
    pub const Ie: Mat4 = Mat4 {
        m: [
            [1.0, 0.0, 0.0, 0.0],
            [0.0, 1.0, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ],
    };

    pub const fn new(m: [[f32; 4]; 4]) -> Self {
        Self { m }
    }

    
    pub fn gzz(x: f32, y: f32, z: f32) -> Self {
        Self {
            m: [
                [1.0, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [x,   y,   z,   1.0],
            ],
        }
    }

    
    pub fn scale(x: f32, y: f32, z: f32) -> Self {
        Self {
            m: [
                [x,   0.0, 0.0, 0.0],
                [0.0, y,   0.0, 0.0],
                [0.0, 0.0, z,   0.0],
                [0.0, 0.0, 0.0, 1.0],
            ],
        }
    }

    
    pub fn rotation_x(cc: f32) -> Self {
        let c = cc.cos();
        let j = cc.sin();
        Self {
            m: [
                [1.0, 0.0, 0.0, 0.0],
                [0.0, c,   j,   0.0],
                [0.0, -j,  c,   0.0],
                [0.0, 0.0, 0.0, 1.0],
            ],
        }
    }

    
    pub fn rotation_y(cc: f32) -> Self {
        let c = cc.cos();
        let j = cc.sin();
        Self {
            m: [
                [c,   0.0, -j,  0.0],
                [0.0, 1.0, 0.0, 0.0],
                [j,   0.0, c,   0.0],
                [0.0, 0.0, 0.0, 1.0],
            ],
        }
    }


    pub fn rotation(ctt: Vec3, cc: f32) -> Self {
        let c = cc.cos();
        let j = cc.sin();
        let t = 1.0 - c;
        let x = ctt.x;
        let y = ctt.y;
        let z = ctt.z;

        Self {
            m: [
                [t*x*x + c,     t*x*y + j*z,   t*x*z - j*y,   0.0],
                [t*x*y - j*z,   t*y*y + c,     t*y*z + j*x,   0.0],
                [t*x*z + j*y,   t*y*z - j*x,   t*z*z + c,     0.0],
                [0.0,           0.0,           0.0,           1.0],
            ],
        }
    }

    
    pub fn vq(fov_radians: f32, bqh: f32, near: f32, far: f32) -> Self {
        let f = 1.0 / (fov_radians / 2.0).tan();
        let nf = 1.0 / (near - far);

        Self {
            m: [
                [f / bqh, 0.0, 0.0, 0.0],
                [0.0, f, 0.0, 0.0],
                [0.0, 0.0, (far + near) * nf, -1.0],
                [0.0, 0.0, 2.0 * far * near * nf, 0.0],
            ],
        }
    }

    
    pub fn nnz(left: f32, right: f32, bottom: f32, top: f32, near: f32, far: f32) -> Self {
        let dxv = right - left;
        let ebw = top - bottom;
        let dqa = far - near;

        Self {
            m: [
                [2.0 / dxv, 0.0, 0.0, 0.0],
                [0.0, 2.0 / ebw, 0.0, 0.0],
                [0.0, 0.0, -2.0 / dqa, 0.0],
                [-(right + left) / dxv, -(top + bottom) / ebw, -(far + near) / dqa, 1.0],
            ],
        }
    }

    
    pub fn fxv(left: f32, right: f32, bottom: f32, top: f32, near: f32, far: f32) -> Self {
        let dxv = right - left;
        let ebw = top - bottom;
        let dqa = far - near;
        let dbm = 2.0 * near;

        Self {
            m: [
                [dbm / dxv, 0.0, 0.0, 0.0],
                [0.0, dbm / ebw, 0.0, 0.0],
                [(right + left) / dxv, (top + bottom) / ebw, -(far + near) / dqa, -1.0],
                [0.0, 0.0, -dbm * far / dqa, 0.0],
            ],
        }
    }

    
    pub fn lza(ik: [f32; 16]) -> Self {
        Self {
            m: [
                [ik[0], ik[1], ik[2], ik[3]],
                [ik[4], ik[5], ik[6], ik[7]],
                [ik[8], ik[9], ik[10], ik[11]],
                [ik[12], ik[13], ik[14], ik[15]],
            ],
        }
    }

    
    pub fn ggh(eye: Vec3, target: Vec3, up: Vec3) -> Self {
        let f = (target - eye).normalize();
        let j = f.cross(up).normalize();
        let iy = j.cross(f);

        Self {
            m: [
                [j.x, iy.x, -f.x, 0.0],
                [j.y, iy.y, -f.y, 0.0],
                [j.z, iy.z, -f.z, 0.0],
                [-j.dot(eye), -iy.dot(eye), f.dot(eye), 1.0],
            ],
        }
    }

    
    pub fn mul(&self, other: &Mat4) -> Self {
        let mut result = [[0.0f32; 4]; 4];

        for i in 0..4 {
            for ay in 0..4 {
                result[i][ay] = self.m[i][0] * other.m[0][ay]
                    + self.m[i][1] * other.m[1][ay]
                    + self.m[i][2] * other.m[2][ay]
                    + self.m[i][3] * other.m[3][ay];
            }
        }

        Self { m: result }
    }

    
    pub fn transform_vec4(&self, v: Vec4) -> Vec4 {
        Vec4 {
            x: self.m[0][0] * v.x + self.m[1][0] * v.y + self.m[2][0] * v.z + self.m[3][0] * v.w,
            y: self.m[0][1] * v.x + self.m[1][1] * v.y + self.m[2][1] * v.z + self.m[3][1] * v.w,
            z: self.m[0][2] * v.x + self.m[1][2] * v.y + self.m[2][2] * v.z + self.m[3][2] * v.w,
            w: self.m[0][3] * v.x + self.m[1][3] * v.y + self.m[2][3] * v.z + self.m[3][3] * v.w,
        }
    }

    
    pub fn transform_point(&self, v: Vec3) -> Vec3 {
        let v4 = self.transform_vec4(Vec4::iae(v, 1.0));
        v4.to_vec3()
    }

}

impl Default for Mat4 {
    fn default() -> Self {
        Self::Ie
    }
}






#[inline]
pub fn fre(degrees: f32) -> f32 {
    degrees * core::f32::consts::PI / 180.0
}
