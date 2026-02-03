//! Cryptographic Primitives for TLS 1.3 (Pure Rust, no_std)
//!
//! Implements:
//! - SHA-256 (for HKDF and transcript hash)
//! - HKDF (for key derivation)
//! - AES-128-GCM (for record encryption)
//! - X25519 (for key exchange)

use alloc::vec::Vec;

// ============================================================================
// SHA-256
// ============================================================================

/// SHA-256 initial hash values
const SHA256_H: [u32; 8] = [
    0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a,
    0x510e527f, 0x9b05688c, 0x1f83d9ab, 0x5be0cd19,
];

/// SHA-256 round constants
const SHA256_K: [u32; 64] = [
    0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5,
    0x3956c25b, 0x59f111f1, 0x923f82a4, 0xab1c5ed5,
    0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3,
    0x72be5d74, 0x80deb1fe, 0x9bdc06a7, 0xc19bf174,
    0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc,
    0x2de92c6f, 0x4a7484aa, 0x5cb0a9dc, 0x76f988da,
    0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7,
    0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967,
    0x27b70a85, 0x2e1b2138, 0x4d2c6dfc, 0x53380d13,
    0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85,
    0xa2bfe8a1, 0xa81a664b, 0xc24b8b70, 0xc76c51a3,
    0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070,
    0x19a4c116, 0x1e376c08, 0x2748774c, 0x34b0bcb5,
    0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
    0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208,
    0x90befffa, 0xa4506ceb, 0xbef9a3f7, 0xc67178f2,
];

/// SHA-256 hasher
#[derive(Clone)]
pub struct Sha256 {
    state: [u32; 8],
    buffer: [u8; 64],
    buffer_len: usize,
    total_len: u64,
}

impl Sha256 {
    pub fn new() -> Self {
        Self {
            state: SHA256_H,
            buffer: [0u8; 64],
            buffer_len: 0,
            total_len: 0,
        }
    }
    
    pub fn update(&mut self, data: &[u8]) {
        for &byte in data {
            self.buffer[self.buffer_len] = byte;
            self.buffer_len += 1;
            self.total_len += 1;
            
            if self.buffer_len == 64 {
                self.process_block();
                self.buffer_len = 0;
            }
        }
    }
    
    pub fn finalize(&mut self) -> [u8; 32] {
        // Padding
        let bit_len = self.total_len * 8;
        self.buffer[self.buffer_len] = 0x80;
        self.buffer_len += 1;
        
        if self.buffer_len > 56 {
            while self.buffer_len < 64 {
                self.buffer[self.buffer_len] = 0;
                self.buffer_len += 1;
            }
            self.process_block();
            self.buffer_len = 0;
        }
        
        while self.buffer_len < 56 {
            self.buffer[self.buffer_len] = 0;
            self.buffer_len += 1;
        }
        
        // Append length in bits
        self.buffer[56..64].copy_from_slice(&bit_len.to_be_bytes());
        self.process_block();
        
        // Output
        let mut result = [0u8; 32];
        for (i, &word) in self.state.iter().enumerate() {
            result[i * 4..i * 4 + 4].copy_from_slice(&word.to_be_bytes());
        }
        result
    }
    
    fn process_block(&mut self) {
        let mut w = [0u32; 64];
        
        // Prepare message schedule
        for i in 0..16 {
            w[i] = u32::from_be_bytes([
                self.buffer[i * 4],
                self.buffer[i * 4 + 1],
                self.buffer[i * 4 + 2],
                self.buffer[i * 4 + 3],
            ]);
        }
        
        for i in 16..64 {
            let s0 = w[i - 15].rotate_right(7) ^ w[i - 15].rotate_right(18) ^ (w[i - 15] >> 3);
            let s1 = w[i - 2].rotate_right(17) ^ w[i - 2].rotate_right(19) ^ (w[i - 2] >> 10);
            w[i] = w[i - 16].wrapping_add(s0).wrapping_add(w[i - 7]).wrapping_add(s1);
        }
        
        // Working variables
        let [mut a, mut b, mut c, mut d, mut e, mut f, mut g, mut h] = self.state;
        
        // Compression
        for i in 0..64 {
            let s1 = e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25);
            let ch = (e & f) ^ ((!e) & g);
            let temp1 = h.wrapping_add(s1).wrapping_add(ch).wrapping_add(SHA256_K[i]).wrapping_add(w[i]);
            let s0 = a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22);
            let maj = (a & b) ^ (a & c) ^ (b & c);
            let temp2 = s0.wrapping_add(maj);
            
            h = g;
            g = f;
            f = e;
            e = d.wrapping_add(temp1);
            d = c;
            c = b;
            b = a;
            a = temp1.wrapping_add(temp2);
        }
        
        // Add to state
        self.state[0] = self.state[0].wrapping_add(a);
        self.state[1] = self.state[1].wrapping_add(b);
        self.state[2] = self.state[2].wrapping_add(c);
        self.state[3] = self.state[3].wrapping_add(d);
        self.state[4] = self.state[4].wrapping_add(e);
        self.state[5] = self.state[5].wrapping_add(f);
        self.state[6] = self.state[6].wrapping_add(g);
        self.state[7] = self.state[7].wrapping_add(h);
    }
}

/// Compute SHA-256 hash
pub fn sha256(data: &[u8]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(data);
    hasher.finalize()
}

/// Compute HMAC-SHA-256
pub fn hmac_sha256(key: &[u8], data: &[u8]) -> [u8; 32] {
    let mut key_block = [0u8; 64];
    
    if key.len() > 64 {
        let key_hash = sha256(key);
        key_block[..32].copy_from_slice(&key_hash);
    } else {
        key_block[..key.len()].copy_from_slice(key);
    }
    
    // Inner hash
    let mut ipad = [0x36u8; 64];
    for (i, &k) in key_block.iter().enumerate() {
        ipad[i] ^= k;
    }
    
    let mut inner = Sha256::new();
    inner.update(&ipad);
    inner.update(data);
    let inner_hash = inner.finalize();
    
    // Outer hash
    let mut opad = [0x5cu8; 64];
    for (i, &k) in key_block.iter().enumerate() {
        opad[i] ^= k;
    }
    
    let mut outer = Sha256::new();
    outer.update(&opad);
    outer.update(&inner_hash);
    outer.finalize()
}

// ============================================================================
// HKDF (RFC 5869)
// ============================================================================

/// HKDF-Extract
pub fn hkdf_extract(salt: &[u8], ikm: &[u8]) -> [u8; 32] {
    let salt = if salt.is_empty() { &[0u8; 32] } else { salt };
    hmac_sha256(salt, ikm)
}

/// HKDF-Expand
pub fn hkdf_expand(prk: &[u8; 32], info: &[u8], length: usize) -> Vec<u8> {
    let mut output = Vec::with_capacity(length);
    let mut t = Vec::new();
    let mut counter = 1u8;
    
    while output.len() < length {
        let mut input = Vec::new();
        input.extend_from_slice(&t);
        input.extend_from_slice(info);
        input.push(counter);
        
        t = hmac_sha256(prk, &input).to_vec();
        output.extend_from_slice(&t);
        counter += 1;
    }
    
    output.truncate(length);
    output
}

/// HKDF-Expand-Label for TLS 1.3
pub fn hkdf_expand_label(secret: &[u8; 32], label: &str, context: &[u8], length: usize) -> Vec<u8> {
    let full_label = alloc::format!("tls13 {}", label);
    let label_bytes = full_label.as_bytes();
    
    let mut info = Vec::new();
    info.extend_from_slice(&(length as u16).to_be_bytes());
    info.push(label_bytes.len() as u8);
    info.extend_from_slice(label_bytes);
    info.push(context.len() as u8);
    info.extend_from_slice(context);
    
    hkdf_expand(secret, &info, length)
}

/// Derive secret using HKDF-Expand-Label
pub fn derive_secret(secret: &[u8; 32], label: &str, transcript_hash: &[u8; 32]) -> [u8; 32] {
    let expanded = hkdf_expand_label(secret, label, transcript_hash, 32);
    let mut result = [0u8; 32];
    result.copy_from_slice(&expanded);
    result
}

// ============================================================================
// AES-128-GCM
// ============================================================================

/// AES S-box
const SBOX: [u8; 256] = [
    0x63, 0x7c, 0x77, 0x7b, 0xf2, 0x6b, 0x6f, 0xc5, 0x30, 0x01, 0x67, 0x2b, 0xfe, 0xd7, 0xab, 0x76,
    0xca, 0x82, 0xc9, 0x7d, 0xfa, 0x59, 0x47, 0xf0, 0xad, 0xd4, 0xa2, 0xaf, 0x9c, 0xa4, 0x72, 0xc0,
    0xb7, 0xfd, 0x93, 0x26, 0x36, 0x3f, 0xf7, 0xcc, 0x34, 0xa5, 0xe5, 0xf1, 0x71, 0xd8, 0x31, 0x15,
    0x04, 0xc7, 0x23, 0xc3, 0x18, 0x96, 0x05, 0x9a, 0x07, 0x12, 0x80, 0xe2, 0xeb, 0x27, 0xb2, 0x75,
    0x09, 0x83, 0x2c, 0x1a, 0x1b, 0x6e, 0x5a, 0xa0, 0x52, 0x3b, 0xd6, 0xb3, 0x29, 0xe3, 0x2f, 0x84,
    0x53, 0xd1, 0x00, 0xed, 0x20, 0xfc, 0xb1, 0x5b, 0x6a, 0xcb, 0xbe, 0x39, 0x4a, 0x4c, 0x58, 0xcf,
    0xd0, 0xef, 0xaa, 0xfb, 0x43, 0x4d, 0x33, 0x85, 0x45, 0xf9, 0x02, 0x7f, 0x50, 0x3c, 0x9f, 0xa8,
    0x51, 0xa3, 0x40, 0x8f, 0x92, 0x9d, 0x38, 0xf5, 0xbc, 0xb6, 0xda, 0x21, 0x10, 0xff, 0xf3, 0xd2,
    0xcd, 0x0c, 0x13, 0xec, 0x5f, 0x97, 0x44, 0x17, 0xc4, 0xa7, 0x7e, 0x3d, 0x64, 0x5d, 0x19, 0x73,
    0x60, 0x81, 0x4f, 0xdc, 0x22, 0x2a, 0x90, 0x88, 0x46, 0xee, 0xb8, 0x14, 0xde, 0x5e, 0x0b, 0xdb,
    0xe0, 0x32, 0x3a, 0x0a, 0x49, 0x06, 0x24, 0x5c, 0xc2, 0xd3, 0xac, 0x62, 0x91, 0x95, 0xe4, 0x79,
    0xe7, 0xc8, 0x37, 0x6d, 0x8d, 0xd5, 0x4e, 0xa9, 0x6c, 0x56, 0xf4, 0xea, 0x65, 0x7a, 0xae, 0x08,
    0xba, 0x78, 0x25, 0x2e, 0x1c, 0xa6, 0xb4, 0xc6, 0xe8, 0xdd, 0x74, 0x1f, 0x4b, 0xbd, 0x8b, 0x8a,
    0x70, 0x3e, 0xb5, 0x66, 0x48, 0x03, 0xf6, 0x0e, 0x61, 0x35, 0x57, 0xb9, 0x86, 0xc1, 0x1d, 0x9e,
    0xe1, 0xf8, 0x98, 0x11, 0x69, 0xd9, 0x8e, 0x94, 0x9b, 0x1e, 0x87, 0xe9, 0xce, 0x55, 0x28, 0xdf,
    0x8c, 0xa1, 0x89, 0x0d, 0xbf, 0xe6, 0x42, 0x68, 0x41, 0x99, 0x2d, 0x0f, 0xb0, 0x54, 0xbb, 0x16,
];

/// AES round constants
const RCON: [u8; 10] = [0x01, 0x02, 0x04, 0x08, 0x10, 0x20, 0x40, 0x80, 0x1b, 0x36];

/// AES-128 cipher - software implementation
/// Note: AES-NI hardware acceleration disabled due to x86_64-unknown-none target limitations
pub struct Aes128 {
    round_keys: [[u8; 16]; 11],
}

impl Aes128 {
    pub fn new(key: &[u8; 16]) -> Self {
        let mut round_keys = [[0u8; 16]; 11];
        round_keys[0].copy_from_slice(key);
        
        // Key expansion
        for i in 1..11 {
            let prev = round_keys[i - 1];
            let mut word = [prev[12], prev[13], prev[14], prev[15]];
            
            // RotWord + SubWord + Rcon
            let temp = word[0];
            word[0] = SBOX[word[1] as usize] ^ RCON[i - 1];
            word[1] = SBOX[word[2] as usize];
            word[2] = SBOX[word[3] as usize];
            word[3] = SBOX[temp as usize];
            
            for j in 0..4 {
                round_keys[i][j] = prev[j] ^ word[j];
            }
            for j in 4..16 {
                round_keys[i][j] = prev[j] ^ round_keys[i][j - 4];
            }
        }
        
        Self { round_keys }
    }
    
    /// Encrypt a single block using software AES
    pub fn encrypt_block(&self, block: &mut [u8; 16]) {
        self.encrypt_block_software(block);
    }
    
    /// Software AES encryption
    fn encrypt_block_software(&self, block: &mut [u8; 16]) {
        // Initial round key addition
        for i in 0..16 {
            block[i] ^= self.round_keys[0][i];
        }
        
        // Main rounds
        for round in 1..10 {
            self.sub_bytes(block);
            self.shift_rows(block);
            self.mix_columns(block);
            for i in 0..16 {
                block[i] ^= self.round_keys[round][i];
            }
        }
        
        // Final round
        self.sub_bytes(block);
        self.shift_rows(block);
        for i in 0..16 {
            block[i] ^= self.round_keys[10][i];
        }
    }
    
    fn sub_bytes(&self, block: &mut [u8; 16]) {
        for byte in block.iter_mut() {
            *byte = SBOX[*byte as usize];
        }
    }
    
    fn shift_rows(&self, block: &mut [u8; 16]) {
        // Row 1: shift left by 1
        let temp = block[1];
        block[1] = block[5];
        block[5] = block[9];
        block[9] = block[13];
        block[13] = temp;
        
        // Row 2: shift left by 2
        let (t0, t1) = (block[2], block[6]);
        block[2] = block[10];
        block[6] = block[14];
        block[10] = t0;
        block[14] = t1;
        
        // Row 3: shift left by 3 (= right by 1)
        let temp = block[15];
        block[15] = block[11];
        block[11] = block[7];
        block[7] = block[3];
        block[3] = temp;
    }
    
    fn mix_columns(&self, block: &mut [u8; 16]) {
        for i in 0..4 {
            let col = i * 4;
            let (a, b, c, d) = (block[col], block[col + 1], block[col + 2], block[col + 3]);
            
            block[col] = gf_mul(a, 2) ^ gf_mul(b, 3) ^ c ^ d;
            block[col + 1] = a ^ gf_mul(b, 2) ^ gf_mul(c, 3) ^ d;
            block[col + 2] = a ^ b ^ gf_mul(c, 2) ^ gf_mul(d, 3);
            block[col + 3] = gf_mul(a, 3) ^ b ^ c ^ gf_mul(d, 2);
        }
    }
}

/// Galois field multiplication
fn gf_mul(a: u8, b: u8) -> u8 {
    let mut result = 0u8;
    let mut a = a;
    let mut b = b;
    
    while b != 0 {
        if b & 1 != 0 {
            result ^= a;
        }
        let hi_bit = a & 0x80;
        a <<= 1;
        if hi_bit != 0 {
            a ^= 0x1b;
        }
        b >>= 1;
    }
    result
}

/// GCM multiplication in GF(2^128)
fn gcm_mul(x: &[u8; 16], h: &[u8; 16]) -> [u8; 16] {
    let mut z = [0u8; 16];
    let mut v = *h;
    
    for i in 0..16 {
        for bit in 0..8 {
            if (x[i] >> (7 - bit)) & 1 == 1 {
                for j in 0..16 {
                    z[j] ^= v[j];
                }
            }
            
            // Multiply v by x in GF(2^128)
            let lsb = v[15] & 1;
            for j in (1..16).rev() {
                v[j] = (v[j] >> 1) | (v[j - 1] << 7);
            }
            v[0] >>= 1;
            
            if lsb != 0 {
                v[0] ^= 0xe1; // R = x^128 + x^7 + x^2 + x + 1
            }
        }
    }
    
    z
}

/// GHASH function
fn ghash(h: &[u8; 16], aad: &[u8], ciphertext: &[u8]) -> [u8; 16] {
    let mut y = [0u8; 16];
    
    // Process AAD
    let aad_blocks = (aad.len() + 15) / 16;
    for i in 0..aad_blocks {
        let start = i * 16;
        let end = (start + 16).min(aad.len());
        for j in start..end {
            y[j - start] ^= aad[j];
        }
        y = gcm_mul(&y, h);
    }
    
    // Process ciphertext
    let ct_blocks = (ciphertext.len() + 15) / 16;
    for i in 0..ct_blocks {
        let start = i * 16;
        let end = (start + 16).min(ciphertext.len());
        for j in start..end {
            y[j - start] ^= ciphertext[j];
        }
        y = gcm_mul(&y, h);
    }
    
    // Final block: lengths
    let aad_bits = (aad.len() as u64) * 8;
    let ct_bits = (ciphertext.len() as u64) * 8;
    let mut len_block = [0u8; 16];
    len_block[..8].copy_from_slice(&aad_bits.to_be_bytes());
    len_block[8..].copy_from_slice(&ct_bits.to_be_bytes());
    
    for i in 0..16 {
        y[i] ^= len_block[i];
    }
    gcm_mul(&y, h)
}

/// AES-128-GCM encryption
pub fn aes_gcm_encrypt(
    key: &[u8; 16],
    nonce: &[u8; 12],
    aad: &[u8],
    plaintext: &[u8],
) -> Vec<u8> {
    let aes = Aes128::new(key);
    
    // Compute H = AES(K, 0^128)
    let mut h = [0u8; 16];
    aes.encrypt_block(&mut h);
    
    // Initial counter block
    let mut j0 = [0u8; 16];
    j0[..12].copy_from_slice(nonce);
    j0[15] = 1;
    
    // Encrypt J0 for tag computation
    let mut e_j0 = j0;
    aes.encrypt_block(&mut e_j0);
    
    // Encrypt plaintext with counter mode
    let mut ciphertext = Vec::with_capacity(plaintext.len() + 16);
    let blocks = (plaintext.len() + 15) / 16;
    
    for i in 0..blocks {
        // Increment counter
        let counter = 2u32 + i as u32;
        let mut block = [0u8; 16];
        block[..12].copy_from_slice(nonce);
        block[12..16].copy_from_slice(&counter.to_be_bytes());
        aes.encrypt_block(&mut block);
        
        let start = i * 16;
        let end = (start + 16).min(plaintext.len());
        for j in start..end {
            ciphertext.push(plaintext[j] ^ block[j - start]);
        }
    }
    
    // Compute tag
    let ghash_result = ghash(&h, aad, &ciphertext);
    let mut tag = [0u8; 16];
    for i in 0..16 {
        tag[i] = ghash_result[i] ^ e_j0[i];
    }
    
    ciphertext.extend_from_slice(&tag);
    ciphertext
}

/// AES-128-GCM decryption
pub fn aes_gcm_decrypt(
    key: &[u8; 16],
    nonce: &[u8; 12],
    aad: &[u8],
    ciphertext_with_tag: &[u8],
) -> Result<Vec<u8>, ()> {
    if ciphertext_with_tag.len() < 16 {
        return Err(());
    }
    
    let tag_offset = ciphertext_with_tag.len() - 16;
    let ciphertext = &ciphertext_with_tag[..tag_offset];
    let tag = &ciphertext_with_tag[tag_offset..];
    
    let aes = Aes128::new(key);
    
    // Compute H
    let mut h = [0u8; 16];
    aes.encrypt_block(&mut h);
    
    // Initial counter
    let mut j0 = [0u8; 16];
    j0[..12].copy_from_slice(nonce);
    j0[15] = 1;
    
    let mut e_j0 = j0;
    aes.encrypt_block(&mut e_j0);
    
    // Verify tag
    let ghash_result = ghash(&h, aad, ciphertext);
    let mut computed_tag = [0u8; 16];
    for i in 0..16 {
        computed_tag[i] = ghash_result[i] ^ e_j0[i];
    }
    
    // Constant-time comparison
    let mut diff = 0u8;
    for i in 0..16 {
        diff |= computed_tag[i] ^ tag[i];
    }
    if diff != 0 {
        return Err(());
    }
    
    // Decrypt
    let mut plaintext = Vec::with_capacity(ciphertext.len());
    let blocks = (ciphertext.len() + 15) / 16;
    
    for i in 0..blocks {
        let counter = 2u32 + i as u32;
        let mut block = [0u8; 16];
        block[..12].copy_from_slice(nonce);
        block[12..16].copy_from_slice(&counter.to_be_bytes());
        aes.encrypt_block(&mut block);
        
        let start = i * 16;
        let end = (start + 16).min(ciphertext.len());
        for j in start..end {
            plaintext.push(ciphertext[j] ^ block[j - start]);
        }
    }
    
    Ok(plaintext)
}

// ============================================================================
// X25519 (Curve25519 Diffie-Hellman)
// ============================================================================

/// X25519 base point multiplication (compute public key from private)
pub fn x25519_base(private_key: &[u8; 32]) -> [u8; 32] {
    // Base point = 9
    let mut base = [0u8; 32];
    base[0] = 9;
    x25519(private_key, &base)
}

/// X25519 scalar multiplication
pub fn x25519(k: &[u8; 32], u: &[u8; 32]) -> [u8; 32] {
    // Clamp the scalar
    let mut scalar = *k;
    scalar[0] &= 248;
    scalar[31] &= 127;
    scalar[31] |= 64;
    
    // Montgomery ladder on Curve25519
    // Using field element representation as [u64; 5] for GF(2^255 - 19)
    let u_fe = fe_from_bytes(u);
    
    let mut x_1 = u_fe;
    let mut x_2 = FE_ONE;
    let mut z_2 = FE_ZERO;
    let mut x_3 = u_fe;
    let mut z_3 = FE_ONE;
    
    let mut swap: u64 = 0;
    
    for i in (0..255).rev() {
        let bit = ((scalar[i >> 3] >> (i & 7)) & 1) as u64;
        swap ^= bit;
        fe_cswap(&mut x_2, &mut x_3, swap);
        fe_cswap(&mut z_2, &mut z_3, swap);
        swap = bit;
        
        let a = fe_add(&x_2, &z_2);
        let aa = fe_sq(&a);
        let b = fe_sub(&x_2, &z_2);
        let bb = fe_sq(&b);
        let e = fe_sub(&aa, &bb);
        let c = fe_add(&x_3, &z_3);
        let d = fe_sub(&x_3, &z_3);
        let da = fe_mul(&d, &a);
        let cb = fe_mul(&c, &b);
        
        x_3 = fe_sq(&fe_add(&da, &cb));
        z_3 = fe_mul(&x_1, &fe_sq(&fe_sub(&da, &cb)));
        x_2 = fe_mul(&aa, &bb);
        z_2 = fe_mul(&e, &fe_add(&aa, &fe_mul121666(&e)));
    }
    
    fe_cswap(&mut x_2, &mut x_3, swap);
    fe_cswap(&mut z_2, &mut z_3, swap);
    
    let result = fe_mul(&x_2, &fe_invert(&z_2));
    fe_to_bytes(&result)
}

// Field element operations for GF(2^255 - 19)
type Fe = [u64; 5];

const FE_ZERO: Fe = [0, 0, 0, 0, 0];
const FE_ONE: Fe = [1, 0, 0, 0, 0];

fn fe_from_bytes(b: &[u8; 32]) -> Fe {
    let mut h = [0u64; 5];
    
    fn load64(b: &[u8]) -> u64 {
        let mut result = 0u64;
        for i in 0..8.min(b.len()) {
            result |= (b[i] as u64) << (i * 8);
        }
        result
    }
    
    h[0] = load64(&b[0..8]) & 0x7ffffffffffff;
    h[1] = (load64(&b[6..14]) >> 3) & 0x7ffffffffffff;
    h[2] = (load64(&b[12..20]) >> 6) & 0x7ffffffffffff;
    h[3] = (load64(&b[19..27]) >> 1) & 0x7ffffffffffff;
    h[4] = (load64(&b[24..32]) >> 12) & 0x7ffffffffffff;
    
    h
}

fn fe_to_bytes(h: &Fe) -> [u8; 32] {
    let mut t = *h;
    fe_reduce(&mut t);
    
    let mut s = [0u8; 32];
    s[0] = t[0] as u8;
    s[1] = (t[0] >> 8) as u8;
    s[2] = (t[0] >> 16) as u8;
    s[3] = (t[0] >> 24) as u8;
    s[4] = (t[0] >> 32) as u8;
    s[5] = (t[0] >> 40) as u8;
    s[6] = ((t[0] >> 48) | (t[1] << 3)) as u8;
    s[7] = (t[1] >> 5) as u8;
    s[8] = (t[1] >> 13) as u8;
    s[9] = (t[1] >> 21) as u8;
    s[10] = (t[1] >> 29) as u8;
    s[11] = (t[1] >> 37) as u8;
    s[12] = ((t[1] >> 45) | (t[2] << 6)) as u8;
    s[13] = (t[2] >> 2) as u8;
    s[14] = (t[2] >> 10) as u8;
    s[15] = (t[2] >> 18) as u8;
    s[16] = (t[2] >> 26) as u8;
    s[17] = (t[2] >> 34) as u8;
    s[18] = (t[2] >> 42) as u8;
    s[19] = ((t[2] >> 50) | (t[3] << 1)) as u8;
    s[20] = (t[3] >> 7) as u8;
    s[21] = (t[3] >> 15) as u8;
    s[22] = (t[3] >> 23) as u8;
    s[23] = (t[3] >> 31) as u8;
    s[24] = (t[3] >> 39) as u8;
    s[25] = ((t[3] >> 47) | (t[4] << 4)) as u8;
    s[26] = (t[4] >> 4) as u8;
    s[27] = (t[4] >> 12) as u8;
    s[28] = (t[4] >> 20) as u8;
    s[29] = (t[4] >> 28) as u8;
    s[30] = (t[4] >> 36) as u8;
    s[31] = (t[4] >> 44) as u8;
    
    s
}

fn fe_reduce(h: &mut Fe) {
    let mut carry;
    
    for _ in 0..2 {
        carry = h[0] >> 51;
        h[0] &= 0x7ffffffffffff;
        h[1] += carry;
        
        carry = h[1] >> 51;
        h[1] &= 0x7ffffffffffff;
        h[2] += carry;
        
        carry = h[2] >> 51;
        h[2] &= 0x7ffffffffffff;
        h[3] += carry;
        
        carry = h[3] >> 51;
        h[3] &= 0x7ffffffffffff;
        h[4] += carry;
        
        carry = h[4] >> 51;
        h[4] &= 0x7ffffffffffff;
        h[0] += carry * 19;
    }
}

fn fe_add(a: &Fe, b: &Fe) -> Fe {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2], a[3] + b[3], a[4] + b[4]]
}

fn fe_sub(a: &Fe, b: &Fe) -> Fe {
    // Add 2*p to avoid underflow
    let p = 0x7ffffffffffedu64;
    [
        a[0] + 2 * p - b[0],
        a[1] + 2 * 0x7ffffffffffff - b[1],
        a[2] + 2 * 0x7ffffffffffff - b[2],
        a[3] + 2 * 0x7ffffffffffff - b[3],
        a[4] + 2 * 0x7ffffffffffff - b[4],
    ]
}

fn fe_mul(a: &Fe, b: &Fe) -> Fe {
    let mut r = [0u128; 5];
    
    for i in 0..5 {
        for j in 0..5 {
            let idx = (i + j) % 5;
            let factor = if i + j >= 5 { 19u128 } else { 1u128 };
            r[idx] += (a[i] as u128) * (b[j] as u128) * factor;
        }
    }
    
    let mut h = [0u64; 5];
    for i in 0..5 {
        h[i] = (r[i] & 0x7ffffffffffff) as u64;
        if i < 4 {
            r[i + 1] += r[i] >> 51;
        } else {
            h[0] += ((r[4] >> 51) * 19) as u64;
        }
    }
    
    fe_reduce(&mut h);
    h
}

fn fe_sq(a: &Fe) -> Fe {
    fe_mul(a, a)
}

fn fe_mul121666(a: &Fe) -> Fe {
    let mut r = [0u128; 5];
    for i in 0..5 {
        r[i] = (a[i] as u128) * 121666;
    }
    
    let mut h = [0u64; 5];
    for i in 0..5 {
        h[i] = (r[i] & 0x7ffffffffffff) as u64;
        if i < 4 {
            r[i + 1] += r[i] >> 51;
        } else {
            h[0] += ((r[4] >> 51) * 19) as u64;
        }
    }
    
    fe_reduce(&mut h);
    h
}

fn fe_invert(z: &Fe) -> Fe {
    // z^(p-2) = z^(2^255-21)
    let z2 = fe_sq(z);
    let z4 = fe_sq(&z2);
    let z8 = fe_sq(&z4);
    let z9 = fe_mul(&z8, z);
    let z11 = fe_mul(&z9, &z2);
    let z22 = fe_sq(&z11);
    let z_5_0 = fe_mul(&z22, &z9);
    
    let mut t0 = fe_sq(&z_5_0);
    for _ in 1..5 { t0 = fe_sq(&t0); }
    let z_10_5 = fe_mul(&t0, &z_5_0);
    
    t0 = fe_sq(&z_10_5);
    for _ in 1..10 { t0 = fe_sq(&t0); }
    let z_20_10 = fe_mul(&t0, &z_10_5);
    
    t0 = fe_sq(&z_20_10);
    for _ in 1..20 { t0 = fe_sq(&t0); }
    let z_40_20 = fe_mul(&t0, &z_20_10);
    
    t0 = fe_sq(&z_40_20);
    for _ in 1..10 { t0 = fe_sq(&t0); }
    let z_50_10 = fe_mul(&t0, &z_10_5);
    
    t0 = fe_sq(&z_50_10);
    for _ in 1..50 { t0 = fe_sq(&t0); }
    let z_100_50 = fe_mul(&t0, &z_50_10);
    
    t0 = fe_sq(&z_100_50);
    for _ in 1..100 { t0 = fe_sq(&t0); }
    let z_200_100 = fe_mul(&t0, &z_100_50);
    
    t0 = fe_sq(&z_200_100);
    for _ in 1..50 { t0 = fe_sq(&t0); }
    let z_250_50 = fe_mul(&t0, &z_50_10);
    
    t0 = fe_sq(&z_250_50);
    for _ in 1..5 { t0 = fe_sq(&t0); }
    
    fe_mul(&t0, &z11)
}

fn fe_cswap(a: &mut Fe, b: &mut Fe, swap: u64) {
    let mask = (0u64).wrapping_sub(swap);
    for i in 0..5 {
        let t = mask & (a[i] ^ b[i]);
        a[i] ^= t;
        b[i] ^= t;
    }
}
