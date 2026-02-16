//! 3D Math Library
//!
//! Provides vector, matrix and transform operations for 3D graphics.
//! Uses micromath for no_std compatible math functions.

use micromath::F32Ext;
use core::ops::{Add, Sub, Mul, Neg};

// ═══════════════════════════════════════════════════════════════════════════════
// VECTOR3
// ═══════════════════════════════════════════════════════════════════════════════

/// 3D Vector
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Vec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Vec3 {
    pub const ZERO: Vec3 = Vec3 { x: 0.0, y: 0.0, z: 0.0 };
    pub const ONE: Vec3 = Vec3 { x: 1.0, y: 1.0, z: 1.0 };
    pub const X: Vec3 = Vec3 { x: 1.0, y: 0.0, z: 0.0 };
    pub const Y: Vec3 = Vec3 { x: 0.0, y: 1.0, z: 0.0 };
    pub const Z: Vec3 = Vec3 { x: 0.0, y: 0.0, z: 1.0 };

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
    pub fn scale(&self, s: f32) -> Self {
        Self {
            x: self.x * s,
            y: self.y * s,
            z: self.z * s,
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
    fn mul(self, s: f32) -> Self {
        Self {
            x: self.x * s,
            y: self.y * s,
            z: self.z * s,
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// VECTOR4 (Homogeneous coordinates)
// ═══════════════════════════════════════════════════════════════════════════════

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

    pub const fn from_vec3(v: Vec3, w: f32) -> Self {
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

// ═══════════════════════════════════════════════════════════════════════════════
// MATRIX 4x4
// ═══════════════════════════════════════════════════════════════════════════════

/// 4x4 Matrix for 3D transformations (column-major order)
#[derive(Clone, Copy, Debug)]
pub struct Mat4 {
    pub m: [[f32; 4]; 4],
}

impl Mat4 {
    pub const IDENTITY: Mat4 = Mat4 {
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

    /// Create translation matrix
    pub fn translation(x: f32, y: f32, z: f32) -> Self {
        Self {
            m: [
                [1.0, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [x,   y,   z,   1.0],
            ],
        }
    }

    /// Create scale matrix
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

    /// Create rotation matrix around X axis
    pub fn rotation_x(angle: f32) -> Self {
        let c = angle.cos();
        let s = angle.sin();
        Self {
            m: [
                [1.0, 0.0, 0.0, 0.0],
                [0.0, c,   s,   0.0],
                [0.0, -s,  c,   0.0],
                [0.0, 0.0, 0.0, 1.0],
            ],
        }
    }

    /// Create rotation matrix around Y axis
    pub fn rotation_y(angle: f32) -> Self {
        let c = angle.cos();
        let s = angle.sin();
        Self {
            m: [
                [c,   0.0, -s,  0.0],
                [0.0, 1.0, 0.0, 0.0],
                [s,   0.0, c,   0.0],
                [0.0, 0.0, 0.0, 1.0],
            ],
        }
    }


    /// Create rotation matrix around an arbitrary axis (Rodrigues' rotation formula)
    pub fn rotation(axis: Vec3, angle: f32) -> Self {
        let c = angle.cos();
        let s = angle.sin();
        let t = 1.0 - c;
        let x = axis.x;
        let y = axis.y;
        let z = axis.z;

        Self {
            m: [
                [t*x*x + c,     t*x*y + s*z,   t*x*z - s*y,   0.0],
                [t*x*y - s*z,   t*y*y + c,     t*y*z + s*x,   0.0],
                [t*x*z + s*y,   t*y*z - s*x,   t*z*z + c,     0.0],
                [0.0,           0.0,           0.0,           1.0],
            ],
        }
    }

    /// Create perspective projection matrix
    pub fn perspective(fov_radians: f32, aspect: f32, near: f32, far: f32) -> Self {
        let f = 1.0 / (fov_radians / 2.0).tan();
        let nf = 1.0 / (near - far);

        Self {
            m: [
                [f / aspect, 0.0, 0.0, 0.0],
                [0.0, f, 0.0, 0.0],
                [0.0, 0.0, (far + near) * nf, -1.0],
                [0.0, 0.0, 2.0 * far * near * nf, 0.0],
            ],
        }
    }

    /// Create orthographic projection matrix
    pub fn orthographic(left: f32, right: f32, bottom: f32, top: f32, near: f32, far: f32) -> Self {
        let rml = right - left;
        let tmb = top - bottom;
        let fmn = far - near;

        Self {
            m: [
                [2.0 / rml, 0.0, 0.0, 0.0],
                [0.0, 2.0 / tmb, 0.0, 0.0],
                [0.0, 0.0, -2.0 / fmn, 0.0],
                [-(right + left) / rml, -(top + bottom) / tmb, -(far + near) / fmn, 1.0],
            ],
        }
    }

    /// Create frustum (perspective) projection matrix
    pub fn frustum(left: f32, right: f32, bottom: f32, top: f32, near: f32, far: f32) -> Self {
        let rml = right - left;
        let tmb = top - bottom;
        let fmn = far - near;
        let n2 = 2.0 * near;

        Self {
            m: [
                [n2 / rml, 0.0, 0.0, 0.0],
                [0.0, n2 / tmb, 0.0, 0.0],
                [(right + left) / rml, (top + bottom) / tmb, -(far + near) / fmn, -1.0],
                [0.0, 0.0, -n2 * far / fmn, 0.0],
            ],
        }
    }

    /// Create matrix from a flat array (column-major order like OpenGL)
    pub fn from_array(arr: [f32; 16]) -> Self {
        Self {
            m: [
                [arr[0], arr[1], arr[2], arr[3]],
                [arr[4], arr[5], arr[6], arr[7]],
                [arr[8], arr[9], arr[10], arr[11]],
                [arr[12], arr[13], arr[14], arr[15]],
            ],
        }
    }

    /// Create look-at view matrix
    pub fn look_at(eye: Vec3, target: Vec3, up: Vec3) -> Self {
        let f = (target - eye).normalize();
        let s = f.cross(up).normalize();
        let u = s.cross(f);

        Self {
            m: [
                [s.x, u.x, -f.x, 0.0],
                [s.y, u.y, -f.y, 0.0],
                [s.z, u.z, -f.z, 0.0],
                [-s.dot(eye), -u.dot(eye), f.dot(eye), 1.0],
            ],
        }
    }

    /// Multiply two matrices
    pub fn mul(&self, other: &Mat4) -> Self {
        let mut result = [[0.0f32; 4]; 4];

        for i in 0..4 {
            for j in 0..4 {
                result[i][j] = self.m[i][0] * other.m[0][j]
                    + self.m[i][1] * other.m[1][j]
                    + self.m[i][2] * other.m[2][j]
                    + self.m[i][3] * other.m[3][j];
            }
        }

        Self { m: result }
    }

    /// Transform a Vec4
    pub fn transform_vec4(&self, v: Vec4) -> Vec4 {
        Vec4 {
            x: self.m[0][0] * v.x + self.m[1][0] * v.y + self.m[2][0] * v.z + self.m[3][0] * v.w,
            y: self.m[0][1] * v.x + self.m[1][1] * v.y + self.m[2][1] * v.z + self.m[3][1] * v.w,
            z: self.m[0][2] * v.x + self.m[1][2] * v.y + self.m[2][2] * v.z + self.m[3][2] * v.w,
            w: self.m[0][3] * v.x + self.m[1][3] * v.y + self.m[2][3] * v.z + self.m[3][3] * v.w,
        }
    }

    /// Transform a Vec3 as a point (w=1)
    pub fn transform_point(&self, v: Vec3) -> Vec3 {
        let v4 = self.transform_vec4(Vec4::from_vec3(v, 1.0));
        v4.to_vec3()
    }

}

impl Default for Mat4 {
    fn default() -> Self {
        Self::IDENTITY
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// HELPER FUNCTIONS
// ═══════════════════════════════════════════════════════════════════════════════

/// Convert degrees to radians
#[inline]
pub fn deg_to_rad(degrees: f32) -> f32 {
    degrees * core::f32::consts::PI / 180.0
}
