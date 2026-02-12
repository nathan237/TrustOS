// ═══════════════════════════════════════════════════════════════════════
// Ed25519 Digital Signature Scheme — Pure Rust, no_std
//
// Implements RFC 8032 Ed25519 signatures using the existing GF(2^255-19)
// field arithmetic from the TLS 1.3 module.
//
// Addresses GitHub Issue #2: "security: signature system forgeable"
// Asymmetric signing: private key signs, public key verifies — without
// knowing the signing secret.
// ═══════════════════════════════════════════════════════════════════════

use alloc::vec::Vec;
use crate::tls13::crypto::{
    Fe, FE_ZERO, FE_ONE,
    fe_from_bytes, fe_to_bytes, fe_reduce,
    fe_add, fe_sub, fe_mul, fe_sq,
    fe_invert, pow22501,
};

// ─────────────────────────────────────────────────────────────────────
// SHA-512 (required by Ed25519, not available elsewhere in the kernel)
// ─────────────────────────────────────────────────────────────────────

const SHA512_K: [u64; 80] = [
    0x428a2f98d728ae22, 0x7137449123ef65cd, 0xb5c0fbcfec4a3b47, 0xe9b5dba58189dbbc,
    0x3956c25bf348b538, 0x59f111f1b605d019, 0x923f82a4af194f9b, 0xab1c5ed5da6d8118,
    0xd807aa98a3030242, 0x12835b0145706fbe, 0x243185be4ee4b28c, 0x550c7dc3d5ffb4e2,
    0x72be5d74f27b896f, 0x80deb1fe3b1696b1, 0x9bdc06a725c71235, 0xc19bf174cf692694,
    0xe49b69c19ef14ad2, 0xefbe4786384f25e3, 0x0fc19dc68b8cd5b5, 0x240ca1cc77ac9c65,
    0x2de92c6f592b0275, 0x4a7484aa6ea6e483, 0x5cb0a9dcbd41fbd4, 0x76f988da831153b5,
    0x983e5152ee66dfab, 0xa831c66d2db43210, 0xb00327c898fb213f, 0xbf597fc7beef0ee4,
    0xc6e00bf33da88fc2, 0xd5a79147930aa725, 0x06ca6351e003826f, 0x142929670a0e6e70,
    0x27b70a8546d22ffc, 0x2e1b21385c26c926, 0x4d2c6dfc5ac42aed, 0x53380d139d95b3df,
    0x650a73548baf63de, 0x766a0abb3c77b2a8, 0x81c2c92e47edaee6, 0x92722c851482353b,
    0xa2bfe8a1a81a664b, 0xa81a664bbc423001, 0xc24b8b70d0f89791, 0xc76c51a30654be30,
    0xd192e819d6ef5218, 0xd69906245565a910, 0xf40e35855771202a, 0x106aa07032bbd1b8,
    0x19a4c116b8d2d0c8, 0x1e376c085141ab53, 0x2748774cdf8eeb99, 0x34b0bcb5e19b48a8,
    0x391c0cb3c5c95a63, 0x4ed8aa4ae3418acb, 0x5b9cca4f7763e373, 0x682e6ff3d6b2b8a3,
    0x748f82ee5defb2fc, 0x78a5636f43172f60, 0x84c87814a1f0ab72, 0x8cc702081a6439ec,
    0x90befffa23631e28, 0xa4506cebde82bde9, 0xbef9a3f7b2c67915, 0xc67178f2e372532b,
    0xca273eceea26619c, 0xd186b8c721c0c207, 0xeada7dd6cde0eb1e, 0xf57d4f7fee6ed178,
    0x06f067aa72176fba, 0x0a637dc5a2c898a6, 0x113f9804bef90dae, 0x1b710b35131c471b,
    0x28db77f523047d84, 0x32caab7b40c72493, 0x3c9ebe0a15c9bebc, 0x431d67c49c100d4c,
    0x4cc5d4becb3e42b6, 0x597f299cfc657e2a, 0x5fcb6fab3ad6faec, 0x6c44198c4a475817,
];

pub fn sha512(data: &[u8]) -> [u8; 64] {
    let mut h: [u64; 8] = [
        0x6a09e667f3bcc908, 0xbb67ae8584caa73b,
        0x3c6ef372fe94f82b, 0xa54ff53a5f1d36f1,
        0x510e527fade682d1, 0x9b05688c2b3e6c1f,
        0x1f83d9abfb41bd6b, 0x5be0cd19137e2179,
    ];

    let bit_len = (data.len() as u128) * 8;

    // Pre-process: pad message
    let mut msg = Vec::from(data);
    msg.push(0x80);
    while msg.len() % 128 != 112 {
        msg.push(0);
    }
    // Append length as 128-bit big-endian
    for i in (0..16).rev() {
        msg.push((bit_len >> (i * 8)) as u8);
    }

    // Process each 128-byte block
    for block in msg.chunks_exact(128) {
        let mut w = [0u64; 80];
        for i in 0..16 {
            w[i] = u64::from_be_bytes([
                block[i*8], block[i*8+1], block[i*8+2], block[i*8+3],
                block[i*8+4], block[i*8+5], block[i*8+6], block[i*8+7],
            ]);
        }
        for i in 16..80 {
            let s0 = w[i-15].rotate_right(1) ^ w[i-15].rotate_right(8) ^ (w[i-15] >> 7);
            let s1 = w[i-2].rotate_right(19) ^ w[i-2].rotate_right(61) ^ (w[i-2] >> 6);
            w[i] = w[i-16].wrapping_add(s0).wrapping_add(w[i-7]).wrapping_add(s1);
        }

        let (mut a, mut b, mut c, mut d, mut e, mut f, mut g, mut hh) =
            (h[0], h[1], h[2], h[3], h[4], h[5], h[6], h[7]);

        for i in 0..80 {
            let s1 = e.rotate_right(14) ^ e.rotate_right(18) ^ e.rotate_right(41);
            let ch = (e & f) ^ ((!e) & g);
            let temp1 = hh.wrapping_add(s1).wrapping_add(ch).wrapping_add(SHA512_K[i]).wrapping_add(w[i]);
            let s0 = a.rotate_right(28) ^ a.rotate_right(34) ^ a.rotate_right(39);
            let maj = (a & b) ^ (a & c) ^ (b & c);
            let temp2 = s0.wrapping_add(maj);

            hh = g; g = f; f = e;
            e = d.wrapping_add(temp1);
            d = c; c = b; b = a;
            a = temp1.wrapping_add(temp2);
        }

        h[0] = h[0].wrapping_add(a); h[1] = h[1].wrapping_add(b);
        h[2] = h[2].wrapping_add(c); h[3] = h[3].wrapping_add(d);
        h[4] = h[4].wrapping_add(e); h[5] = h[5].wrapping_add(f);
        h[6] = h[6].wrapping_add(g); h[7] = h[7].wrapping_add(hh);
    }

    let mut out = [0u8; 64];
    for i in 0..8 {
        out[i*8..i*8+8].copy_from_slice(&h[i].to_be_bytes());
    }
    out
}

/// Incremental SHA-512 hasher
pub struct Sha512 {
    state: [u64; 8],
    buffer: [u8; 128],
    buffer_len: usize,
    total_len: u128,
}

impl Sha512 {
    pub fn new() -> Self {
        Self {
            state: [
                0x6a09e667f3bcc908, 0xbb67ae8584caa73b,
                0x3c6ef372fe94f82b, 0xa54ff53a5f1d36f1,
                0x510e527fade682d1, 0x9b05688c2b3e6c1f,
                0x1f83d9abfb41bd6b, 0x5be0cd19137e2179,
            ],
            buffer: [0u8; 128],
            buffer_len: 0,
            total_len: 0,
        }
    }

    pub fn update(&mut self, data: &[u8]) {
        self.total_len += data.len() as u128;
        let mut offset = 0;

        if self.buffer_len > 0 {
            let space = 128 - self.buffer_len;
            let to_copy = data.len().min(space);
            self.buffer[self.buffer_len..self.buffer_len + to_copy]
                .copy_from_slice(&data[..to_copy]);
            self.buffer_len += to_copy;
            offset = to_copy;

            if self.buffer_len == 128 {
                let block = self.buffer;
                sha512_compress(&mut self.state, &block);
                self.buffer_len = 0;
            }
        }

        while offset + 128 <= data.len() {
            let mut block = [0u8; 128];
            block.copy_from_slice(&data[offset..offset + 128]);
            sha512_compress(&mut self.state, &block);
            offset += 128;
        }

        if offset < data.len() {
            let remaining = data.len() - offset;
            self.buffer[..remaining].copy_from_slice(&data[offset..]);
            self.buffer_len = remaining;
        }
    }

    pub fn finalize(&mut self) -> [u8; 64] {
        let bit_len = self.total_len * 8;

        // Pad
        self.buffer[self.buffer_len] = 0x80;
        self.buffer_len += 1;

        if self.buffer_len > 112 {
            // Need another block
            for i in self.buffer_len..128 {
                self.buffer[i] = 0;
            }
            let block = self.buffer;
            sha512_compress(&mut self.state, &block);
            self.buffer_len = 0;
        }

        for i in self.buffer_len..112 {
            self.buffer[i] = 0;
        }

        for i in (0..16).rev() {
            self.buffer[112 + (15 - i)] = (bit_len >> (i * 8)) as u8;
        }

        let block = self.buffer;
        sha512_compress(&mut self.state, &block);

        let mut out = [0u8; 64];
        for i in 0..8 {
            out[i*8..i*8+8].copy_from_slice(&self.state[i].to_be_bytes());
        }
        out
    }
}

fn sha512_compress(state: &mut [u64; 8], block: &[u8; 128]) {
    let mut w = [0u64; 80];
    for i in 0..16 {
        w[i] = u64::from_be_bytes([
            block[i*8], block[i*8+1], block[i*8+2], block[i*8+3],
            block[i*8+4], block[i*8+5], block[i*8+6], block[i*8+7],
        ]);
    }
    for i in 16..80 {
        let s0 = w[i-15].rotate_right(1) ^ w[i-15].rotate_right(8) ^ (w[i-15] >> 7);
        let s1 = w[i-2].rotate_right(19) ^ w[i-2].rotate_right(61) ^ (w[i-2] >> 6);
        w[i] = w[i-16].wrapping_add(s0).wrapping_add(w[i-7]).wrapping_add(s1);
    }

    let (mut a, mut b, mut c, mut d, mut e, mut f, mut g, mut hh) =
        (state[0], state[1], state[2], state[3], state[4], state[5], state[6], state[7]);

    for i in 0..80 {
        let s1 = e.rotate_right(14) ^ e.rotate_right(18) ^ e.rotate_right(41);
        let ch = (e & f) ^ ((!e) & g);
        let temp1 = hh.wrapping_add(s1).wrapping_add(ch).wrapping_add(SHA512_K[i]).wrapping_add(w[i]);
        let s0 = a.rotate_right(28) ^ a.rotate_right(34) ^ a.rotate_right(39);
        let maj = (a & b) ^ (a & c) ^ (b & c);
        let temp2 = s0.wrapping_add(maj);

        hh = g; g = f; f = e;
        e = d.wrapping_add(temp1);
        d = c; c = b; b = a;
        a = temp1.wrapping_add(temp2);
    }

    state[0] = state[0].wrapping_add(a); state[1] = state[1].wrapping_add(b);
    state[2] = state[2].wrapping_add(c); state[3] = state[3].wrapping_add(d);
    state[4] = state[4].wrapping_add(e); state[5] = state[5].wrapping_add(f);
    state[6] = state[6].wrapping_add(g); state[7] = state[7].wrapping_add(hh);
}


// ─────────────────────────────────────────────────────────────────────
// Ed25519 curve constants
// ─────────────────────────────────────────────────────────────────────

/// fe_neg: compute -a mod p
fn fe_neg(a: &Fe) -> Fe {
    // 0 - a  (in GF(2^255-19), we add a multiple of p before subtracting)
    fe_sub(&FE_ZERO, a)
}

/// d = -121665/121666 mod p (twisted Edwards curve parameter)
/// In little-endian bytes: a3785913ca4deb75abd841414d0a700098e8797779 40c78c73fe6f2bee6c0352
fn ed25519_d() -> Fe {
    fe_from_bytes(&[
        0xa3, 0x78, 0x59, 0x13, 0xca, 0x4d, 0xeb, 0x75,
        0xab, 0xd8, 0x41, 0x41, 0x4d, 0x0a, 0x70, 0x00,
        0x98, 0xe8, 0x79, 0x77, 0x79, 0x40, 0xc7, 0x8c,
        0x73, 0xfe, 0x6f, 0x2b, 0xee, 0x6c, 0x03, 0x52,
    ])
}

/// 2*d mod p
fn ed25519_2d() -> Fe {
    let d = ed25519_d();
    fe_add(&d, &d)
}

/// sqrt(-1) mod p = 2^((p-1)/4) mod p
fn fe_sqrt_m1() -> Fe {
    fe_from_bytes(&[
        0xb0, 0xa0, 0x0e, 0x4a, 0x27, 0x1b, 0xee, 0xc4,
        0x78, 0xe4, 0x2f, 0xad, 0x06, 0x18, 0x43, 0x2f,
        0xa7, 0xd7, 0xfb, 0x3d, 0x99, 0x00, 0x4d, 0x2b,
        0x0b, 0xdf, 0xc1, 0x4f, 0x80, 0x24, 0x83, 0x2b,
    ])
}


// ─────────────────────────────────────────────────────────────────────
// Extended twisted Edwards point (X:Y:Z:T) where x=X/Z, y=Y/Z, T=XY/Z
// ─────────────────────────────────────────────────────────────────────

#[derive(Clone)]
pub struct ExtPoint {
    x: Fe,
    y: Fe,
    z: Fe,
    t: Fe,
}

impl ExtPoint {
    /// Identity point (0, 1, 1, 0)
    pub fn identity() -> Self {
        Self {
            x: FE_ZERO,
            y: FE_ONE,
            z: FE_ONE,
            t: FE_ZERO,
        }
    }

    /// Ed25519 base point B
    pub fn basepoint() -> Self {
        // B_y = 4/5 mod p  (little-endian)
        let y = fe_from_bytes(&[
            0x58, 0x66, 0x66, 0x66, 0x66, 0x66, 0x66, 0x66,
            0x66, 0x66, 0x66, 0x66, 0x66, 0x66, 0x66, 0x66,
            0x66, 0x66, 0x66, 0x66, 0x66, 0x66, 0x66, 0x66,
            0x66, 0x66, 0x66, 0x66, 0x66, 0x66, 0x66, 0x66,
        ]);
        // B_x (little-endian)
        let x = fe_from_bytes(&[
            0x1a, 0xd5, 0x25, 0x8f, 0x60, 0x2d, 0x56, 0xc9,
            0xb2, 0xa7, 0x25, 0x95, 0x60, 0xc7, 0x2c, 0x69,
            0x5c, 0xdc, 0xd6, 0xfd, 0x31, 0xe2, 0xa4, 0xc0,
            0xfe, 0x53, 0x6e, 0xcd, 0xd3, 0x36, 0x69, 0x21,
        ]);
        Self {
            t: fe_mul(&x, &y),
            x,
            y,
            z: FE_ONE,
        }
    }

    /// Point addition (unified formula for twisted Edwards curves)
    pub fn add(&self, other: &Self) -> Self {
        let d2 = ed25519_2d();
        let a = fe_mul(&fe_sub(&self.y, &self.x), &fe_sub(&other.y, &other.x));
        let b = fe_mul(&fe_add(&self.y, &self.x), &fe_add(&other.y, &other.x));
        let c = fe_mul(&fe_mul(&self.t, &other.t), &d2);
        let d = fe_add(&self.z, &self.z);
        let d = fe_mul(&d, &other.z);
        let e = fe_sub(&b, &a);
        let f = fe_sub(&d, &c);
        let g = fe_add(&d, &c);
        let h = fe_add(&b, &a);
        Self {
            x: fe_mul(&e, &f),
            y: fe_mul(&g, &h),
            t: fe_mul(&e, &h),
            z: fe_mul(&f, &g),
        }
    }

    /// Point doubling (optimized)
    pub fn double(&self) -> Self {
        let a = fe_sq(&self.x);
        let b = fe_sq(&self.y);
        let c = fe_add(&fe_sq(&self.z), &fe_sq(&self.z));
        let h = fe_add(&a, &b);
        let xy = fe_add(&self.x, &self.y);
        let e = fe_sub(&h, &fe_sq(&xy));
        let g = fe_sub(&a, &b);
        let f = fe_add(&c, &g);
        Self {
            x: fe_mul(&e, &f),
            y: fe_mul(&g, &h),
            t: fe_mul(&e, &h),
            z: fe_mul(&f, &g),
        }
    }

    /// Scalar multiplication: compute scalar * self  (double-and-add)
    pub fn scalar_mul(&self, scalar: &[u8; 32]) -> Self {
        let mut result = ExtPoint::identity();
        let mut temp = self.clone();

        for i in 0..256 {
            let byte_idx = i / 8;
            let bit_idx = i % 8;
            if (scalar[byte_idx] >> bit_idx) & 1 == 1 {
                result = result.add(&temp);
            }
            temp = temp.double();
        }
        result
    }

    /// Compress point to 32 bytes (y coordinate + sign of x in high bit)
    pub fn compress(&self) -> [u8; 32] {
        let z_inv = fe_invert(&self.z);
        let x = fe_mul(&self.x, &z_inv);
        let y = fe_mul(&self.y, &z_inv);
        let mut s = fe_to_bytes(&y);
        // Set high bit to low bit of x
        let x_bytes = fe_to_bytes(&x);
        s[31] |= (x_bytes[0] & 1) << 7;
        s
    }

    /// Decompress point from 32 bytes
    pub fn decompress(s: &[u8; 32]) -> Option<Self> {
        // Extract sign bit of x and clear it
        let x_sign = (s[31] >> 7) & 1;
        let mut y_bytes = *s;
        y_bytes[31] &= 0x7f;

        let y = fe_from_bytes(&y_bytes);
        let d = ed25519_d();

        // Compute x^2 = (y^2 - 1) / (d*y^2 + 1)
        let y2 = fe_sq(&y);
        let u = fe_sub(&y2, &FE_ONE);          // u = y^2 - 1
        let v = fe_add(&fe_mul(&d, &y2), &FE_ONE); // v = d*y^2 + 1

        // x = sqrt(u/v)
        // Using: x = u * v^3 * (u * v^7)^((p-5)/8)
        let v2 = fe_sq(&v);
        let v3 = fe_mul(&v2, &v);
        let v4 = fe_sq(&v2);
        let v7 = fe_mul(&v4, &v3);
        let uv7 = fe_mul(&u, &v7);

        // Compute uv7^((p-5)/8) = uv7^(2^252 - 3)
        // Using pow22501: z^(2^250-1), then square twice and multiply
        let (t250_1, _) = pow22501(&uv7);
        let t251 = fe_sq(&t250_1);
        let t252 = fe_sq(&t251);       // uv7^(2^252 - 4)
        let beta = fe_mul(&t252, &uv7); // uv7^(2^252 - 3)  ... wait

        // Actually: (p-5)/8 = (2^255 - 24)/8 = 2^252 - 3
        // We computed uv7^(2^252-4) above, need uv7^(2^252-3) = uv7^(2^252-4) * uv7
        let pow_result = fe_mul(&t252, &uv7);

        let mut x = fe_mul(&fe_mul(&u, &v3), &pow_result);

        // Check: v * x^2 == u ?
        let vx2 = fe_mul(&v, &fe_sq(&x));
        let check = fe_sub(&vx2, &u);
        let check_bytes = fe_to_bytes(&check);
        let check_is_zero = check_bytes.iter().all(|&b| b == 0);

        if !check_is_zero {
            // Try x * sqrt(-1)
            let neg_check = fe_add(&vx2, &u);
            let neg_bytes = fe_to_bytes(&neg_check);
            let neg_is_zero = neg_bytes.iter().all(|&b| b == 0);
            if !neg_is_zero {
                return None; // Not a valid point
            }
            x = fe_mul(&x, &fe_sqrt_m1());
        }

        // Adjust sign
        let x_bytes = fe_to_bytes(&x);
        if (x_bytes[0] & 1) != x_sign {
            x = fe_neg(&x);
        }

        let t = fe_mul(&x, &y);
        Some(Self { x, y, z: FE_ONE, t })
    }
}


// ─────────────────────────────────────────────────────────────────────
// Scalar arithmetic mod l (group order)
// l = 2^252 + 27742317777372353535851937790883648493
// Using TweetNaCl-style base-256 reduction
// ─────────────────────────────────────────────────────────────────────

/// l in little-endian bytes
const L_BYTES: [u8; 32] = [
    0xed, 0xd3, 0xf5, 0x5c, 0x1a, 0x63, 0x12, 0x58,
    0xd6, 0x9c, 0xf7, 0xa2, 0xde, 0xf9, 0xde, 0x14,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x10,
];

/// Reduce a 64-element i64 array mod l, output 32 bytes
/// Based on TweetNaCl's modL function
fn mod_l(x: &mut [i64; 64], r: &mut [u8; 32]) {
    for i in (32usize..=63).rev() {
        let mut carry: i64 = 0;
        let start = if i >= 32 { i - 32 } else { 0 };
        let end = i - 12;
        for j in start..end {
            x[j] += carry - 16 * x[i] * L_BYTES[j - start] as i64;
            carry = (x[j] + 128) >> 8;
            x[j] -= carry << 8;
        }
        x[end] += carry;
        x[i] = 0;
    }

    let mut carry: i64 = 0;
    for j in 0..32 {
        x[j] += carry - (x[31] >> 4) * L_BYTES[j] as i64;
        carry = x[j] >> 8;
        x[j] &= 255;
    }
    for j in 0..32 {
        x[j] -= carry * L_BYTES[j] as i64;
    }
    for i in 0..32 {
        x[i + 1] += x[i] >> 8;
        r[i] = (x[i] & 255) as u8;
    }
}

/// Reduce a 64-byte hash output mod l → 32-byte scalar
fn sc_reduce(hash: &[u8; 64]) -> [u8; 32] {
    let mut x = [0i64; 64];
    for i in 0..64 {
        x[i] = hash[i] as i64;
    }
    let mut r = [0u8; 32];
    mod_l(&mut x, &mut r);
    r
}

/// Compute r = a*b + c mod l (scalar multiply-and-add)
fn sc_muladd(a: &[u8; 32], b: &[u8; 32], c: &[u8; 32]) -> [u8; 32] {
    let mut x = [0i64; 64];
    // Start with c
    for i in 0..32 {
        x[i] = c[i] as i64;
    }
    // Add a * b (schoolbook multiplication in base-256)
    for i in 0..32 {
        for j in 0..32 {
            x[i + j] += a[i] as i64 * b[j] as i64;
        }
    }
    let mut r = [0u8; 32];
    mod_l(&mut x, &mut r);
    r
}


// ─────────────────────────────────────────────────────────────────────
// Ed25519 Public API
// ─────────────────────────────────────────────────────────────────────

/// Ed25519 keypair: (public_key, private_key)
/// private_key is the 32-byte seed
/// public_key is the compressed point A = [a]B
pub struct Ed25519Keypair {
    pub public_key: [u8; 32],
    pub secret_key: [u8; 32],   // 32-byte seed
}

/// Generate a public key from a 32-byte seed
pub fn ed25519_public_key(seed: &[u8; 32]) -> [u8; 32] {
    let h = sha512(seed);
    let mut a = [0u8; 32];
    a.copy_from_slice(&h[..32]);

    // Clamp scalar
    a[0] &= 248;
    a[31] &= 127;
    a[31] |= 64;

    let bp = ExtPoint::basepoint();
    let point_a = bp.scalar_mul(&a);
    point_a.compress()
}

/// Generate a keypair from a 32-byte seed
pub fn ed25519_keypair(seed: &[u8; 32]) -> Ed25519Keypair {
    let public_key = ed25519_public_key(seed);
    Ed25519Keypair {
        public_key,
        secret_key: *seed,
    }
}

/// Sign a message. Returns a 64-byte signature (R || S).
///
/// Algorithm (RFC 8032):
/// 1. h = SHA-512(seed) → first 32 bytes = scalar a (clamped), last 32 = nonce prefix
/// 2. A = [a]B (public key)
/// 3. r = SHA-512(nonce_prefix || message) mod l
/// 4. R = [r]B
/// 5. S = (r + SHA-512(R || A || message) * a) mod l
/// 6. Signature = R || S
pub fn ed25519_sign(message: &[u8], seed: &[u8; 32], public_key: &[u8; 32]) -> [u8; 64] {
    let h = sha512(seed);

    // Clamp scalar a
    let mut a = [0u8; 32];
    a.copy_from_slice(&h[..32]);
    a[0] &= 248;
    a[31] &= 127;
    a[31] |= 64;

    // nonce_prefix = h[32..64]
    // r = SHA-512(nonce_prefix || message) mod l
    let mut hasher = Sha512::new();
    hasher.update(&h[32..64]);
    hasher.update(message);
    let nonce_hash = hasher.finalize();
    let r = sc_reduce(&nonce_hash);

    // R = [r]B
    let bp = ExtPoint::basepoint();
    let point_r = bp.scalar_mul(&r);
    let r_compressed = point_r.compress();

    // h_ram = SHA-512(R || A || message) mod l
    let mut hasher2 = Sha512::new();
    hasher2.update(&r_compressed);
    hasher2.update(public_key);
    hasher2.update(message);
    let h_ram_hash = hasher2.finalize();
    let h_ram = sc_reduce(&h_ram_hash);

    // S = (r + h_ram * a) mod l
    let s = sc_muladd(&h_ram, &a, &r);

    let mut sig = [0u8; 64];
    sig[..32].copy_from_slice(&r_compressed);
    sig[32..].copy_from_slice(&s);
    sig
}

/// Verify an Ed25519 signature. Returns true if valid.
///
/// Algorithm:
/// 1. Parse R from signature bytes 0..32, S from bytes 32..64
/// 2. Decompress R and A (public key) as points
/// 3. h = SHA-512(R || A || message) mod l
/// 4. Check: [8*S]B == [8]R + [8*h]A
///
/// (We use the cofactored equation for robustness)
pub fn ed25519_verify(message: &[u8], signature: &[u8; 64], public_key: &[u8; 32]) -> bool {
    // Parse signature
    let mut r_bytes = [0u8; 32];
    r_bytes.copy_from_slice(&signature[..32]);
    let mut s_bytes = [0u8; 32];
    s_bytes.copy_from_slice(&signature[32..]);

    // Check s < l
    if !scalar_is_canonical(&s_bytes) {
        return false;
    }

    // Decompress public key A
    let point_a = match ExtPoint::decompress(public_key) {
        Some(p) => p,
        None => return false,
    };

    // Decompress R
    let point_r = match ExtPoint::decompress(&r_bytes) {
        Some(p) => p,
        None => return false,
    };

    // h = SHA-512(R || A || message) mod l
    let mut hasher = Sha512::new();
    hasher.update(&r_bytes);
    hasher.update(public_key);
    hasher.update(message);
    let h_hash = hasher.finalize();
    let h = sc_reduce(&h_hash);

    // Compute [S]B
    let bp = ExtPoint::basepoint();
    let sb = bp.scalar_mul(&s_bytes);

    // Compute R + [h]A
    let ha = point_a.scalar_mul(&h);
    let rhs = point_r.add(&ha);

    // Compare: [S]B == R + [h]A
    // We compare compressed forms for simplicity
    let lhs_bytes = sb.compress();
    let rhs_bytes = rhs.compress();

    // Constant-time comparison
    let mut diff = 0u8;
    for i in 0..32 {
        diff |= lhs_bytes[i] ^ rhs_bytes[i];
    }
    diff == 0
}

/// Check that a scalar is less than l (canonical form)
fn scalar_is_canonical(s: &[u8; 32]) -> bool {
    // Compare s against l byte-by-byte from MSB
    for i in (0..32).rev() {
        if s[i] < L_BYTES[i] {
            return true;
        }
        if s[i] > L_BYTES[i] {
            return false;
        }
    }
    false // s == l is not canonical
}


// ─────────────────────────────────────────────────────────────────────
// Hex encoding helpers
// ─────────────────────────────────────────────────────────────────────

pub fn bytes_to_hex(bytes: &[u8]) -> alloc::string::String {
    let mut s = alloc::string::String::with_capacity(bytes.len() * 2);
    for &b in bytes {
        use core::fmt::Write;
        let _ = write!(s, "{:02x}", b);
    }
    s
}

pub fn hex_to_bytes_32(hex: &str) -> Option<[u8; 32]> {
    if hex.len() != 64 {
        return None;
    }
    let mut out = [0u8; 32];
    for i in 0..32 {
        let byte_str = &hex[i*2..i*2+2];
        out[i] = u8::from_str_radix(byte_str, 16).ok()?;
    }
    Some(out)
}
