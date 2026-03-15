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
        let mut offset = 0usize;
        let len = data.len();

        // If there's leftover in the buffer, fill it first
        if self.buffer_len > 0 {
            let need = 64 - self.buffer_len;
            let take = if len < need { len } else { need };
            self.buffer[self.buffer_len..self.buffer_len + take]
                .copy_from_slice(&data[..take]);
            self.buffer_len += take;
            self.total_len += take as u64;
            offset += take;

            if self.buffer_len == 64 {
                self.process_block();
                self.buffer_len = 0;
            }
        }

        // Process full 64-byte blocks directly from input
        while offset + 64 <= len {
            self.buffer.copy_from_slice(&data[offset..offset + 64]);
            self.total_len += 64;
            self.process_block();
            offset += 64;
        }

        // Stash remaining bytes
        let remaining = len - offset;
        if remaining > 0 {
            self.buffer[..remaining].copy_from_slice(&data[offset..]);
            self.buffer_len = remaining;
            self.total_len += remaining as u64;
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
        crate::serial_println!("[AES-GCM] Tag mismatch! computed={:02x?} received={:02x?}", 
            &computed_tag[..8], &tag[..8]);
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
/// Uses the Montgomery ladder algorithm exactly as in curve25519-dalek
pub fn x25519(k: &[u8; 32], u: &[u8; 32]) -> [u8; 32] {
    // Clamp the scalar per RFC 7748
    let mut scalar = *k;
    scalar[0] &= 248;
    scalar[31] &= 127;
    scalar[31] |= 64;
    
    // Clear the top bit of the u-coordinate per RFC 7748
    let mut u_clamped = *u;
    u_clamped[31] &= 127;
    
    // Montgomery ladder using Algorithm 8 from Costello-Smith 2017
    let affine_u = fe_from_bytes(&u_clamped);
    
    // x0 = ProjectivePoint::identity() = (U:W) = (1:0)
    let mut x0_u = FE_ONE;
    let mut x0_w = FE_ZERO;
    
    // x1 = (affine_u, 1)
    let mut x1_u = affine_u;
    let mut x1_w = FE_ONE;
    
    // Go through bits from most to least significant, using sliding window of 2
    // Per dalek: scalar invariant #1 says MSB (bit 255) is 0, so skip it
    // We iterate bits 254 down to 0
    let mut prev_bit = false;
    
    for i in (0..255).rev() {
        let cur_bit = ((scalar[i >> 3] >> (i & 7)) & 1) == 1;
        let choice = (prev_bit ^ cur_bit) as u64;
        
        // Conditional swap based on XOR of consecutive bits
        fe_cswap(&mut x0_u, &mut x1_u, choice);
        fe_cswap(&mut x0_w, &mut x1_w, choice);
        
        // differential_add_and_double: P = x0, Q = x1
        differential_add_and_double(&mut x0_u, &mut x0_w, &mut x1_u, &mut x1_w, &affine_u);
        
        prev_bit = cur_bit;
    }
    
    // Final swap based on the LSB (bit 0)
    let final_swap = prev_bit as u64;
    fe_cswap(&mut x0_u, &mut x0_w, final_swap);
    fe_cswap(&mut x1_u, &mut x1_w, final_swap);
    
    // Convert x0 to affine: u = U / W
    let result = fe_mul(&x0_u, &fe_invert(&x0_w));
    fe_to_bytes(&result)
}

/// Perform the double-and-add step of the Montgomery ladder.
/// This is Algorithm 8 from Costello-Smith 2017.
/// 
/// Given projective points (U_P : W_P) = u(P), (U_Q : W_Q) = u(Q),
/// and the affine difference u_{P-Q} = u(P-Q), set:
///     (U_P : W_P) <- u([2]P)
///     (U_Q : W_Q) <- u(P + Q)
fn differential_add_and_double(
    p_u: &mut Fe, p_w: &mut Fe,
    q_u: &mut Fe, q_w: &mut Fe,
    affine_pmq: &Fe,
) {
    let t0 = fe_add(p_u, p_w);
    let t1 = fe_sub(p_u, p_w);
    let t2 = fe_add(q_u, q_w);
    let t3 = fe_sub(q_u, q_w);
    
    let t4 = fe_sq(&t0);    // (U_P + W_P)^2
    let t5 = fe_sq(&t1);    // (U_P - W_P)^2
    
    let t6 = fe_sub(&t4, &t5);  // 4 U_P W_P
    
    let t7 = fe_mul(&t0, &t3);  // (U_P + W_P)(U_Q - W_Q)
    let t8 = fe_mul(&t1, &t2);  // (U_P - W_P)(U_Q + W_Q)
    
    let t9 = fe_add(&t7, &t8);  // 2(U_P U_Q - W_P W_Q)
    let t10 = fe_sub(&t7, &t8); // 2(W_P U_Q - U_P W_Q)
    
    let t11 = fe_sq(&t9);       // 4(U_P U_Q - W_P W_Q)^2
    let t12 = fe_sq(&t10);      // 4(W_P U_Q - U_P W_Q)^2
    
    let t13 = fe_mul121666(&t6); // ((A+2)/4) * 4 U_P W_P = (A+2) U_P W_P / 4 * 4 = (A+2) U_P W_P
    
    let t14 = fe_mul(&t4, &t5); // (U_P^2 - W_P^2)^2
    let t15 = fe_add(&t13, &t5); // (U_P - W_P)^2 + (A+2) U_P W_P
    let t16 = fe_mul(&t6, &t15); // 4 U_P W_P * ((U_P - W_P)^2 + (A+2)/4 * 4 U_P W_P)
    
    let t17 = fe_mul(affine_pmq, &t12); // U_D * 4(W_P U_Q - U_P W_Q)^2
    let t18 = t11;                       // W_D * 4(U_P U_Q - W_P W_Q)^2 (W_D = 1)
    
    // Update P = [2]P
    *p_u = t14;  // U_{P'} = (U_P + W_P)^2 (U_P - W_P)^2
    *p_w = t16;  // W_{P'} = 4 U_P W_P * ((U_P - W_P)^2 + ((A+2)/4) * 4 U_P W_P)
    
    // Update Q = P + Q
    *q_u = t18;  // U_{Q'} = W_D * 4(U_P U_Q - W_P W_Q)^2
    *q_w = t17;  // W_{Q'} = U_D * 4(W_P U_Q - U_P W_Q)^2
}

// Field element operations for GF(2^255 - 19)
pub(crate) type Fe = [u64; 5];

pub(crate) const FE_ZERO: Fe = [0, 0, 0, 0, 0];
pub(crate) const FE_ONE: Fe = [1, 0, 0, 0, 0];

pub(crate) fn fe_from_bytes(b: &[u8; 32]) -> Fe {
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

pub(crate) fn fe_to_bytes(h: &Fe) -> [u8; 32] {
    // First do weak reduction to ensure limbs are bounded
    let mut limbs = *h;
    fe_reduce(&mut limbs);
    
    // Canonical reduction: compute q = (h + 19) / 2^255
    // h >= p <==> h + 19 >= p + 19 <==> h + 19 >= 2^255
    // Therefore q = 1 if h >= p, else q = 0
    // Then r = h - q*p = h + 19*q - 2^255*q
    let mut q = (limbs[0] + 19) >> 51;
    q = (limbs[1] + q) >> 51;
    q = (limbs[2] + q) >> 51;
    q = (limbs[3] + q) >> 51;
    q = (limbs[4] + q) >> 51;
    
    // Now compute r = h + 19*q (and ignore the 2^255*q part by masking)
    limbs[0] += 19 * q;
    
    // Carry propagation
    const LOW_51_BIT_MASK: u64 = (1u64 << 51) - 1;
    limbs[1] += limbs[0] >> 51;
    limbs[0] &= LOW_51_BIT_MASK;
    limbs[2] += limbs[1] >> 51;
    limbs[1] &= LOW_51_BIT_MASK;
    limbs[3] += limbs[2] >> 51;
    limbs[2] &= LOW_51_BIT_MASK;
    limbs[4] += limbs[3] >> 51;
    limbs[3] &= LOW_51_BIT_MASK;
    // Discard the carry from limbs[4] - this subtracts 2^255*q
    limbs[4] &= LOW_51_BIT_MASK;
    
    // Now arrange the bits of the limbs into bytes
    let mut s = [0u8; 32];
    s[0] = limbs[0] as u8;
    s[1] = (limbs[0] >> 8) as u8;
    s[2] = (limbs[0] >> 16) as u8;
    s[3] = (limbs[0] >> 24) as u8;
    s[4] = (limbs[0] >> 32) as u8;
    s[5] = (limbs[0] >> 40) as u8;
    s[6] = ((limbs[0] >> 48) | (limbs[1] << 3)) as u8;
    s[7] = (limbs[1] >> 5) as u8;
    s[8] = (limbs[1] >> 13) as u8;
    s[9] = (limbs[1] >> 21) as u8;
    s[10] = (limbs[1] >> 29) as u8;
    s[11] = (limbs[1] >> 37) as u8;
    s[12] = ((limbs[1] >> 45) | (limbs[2] << 6)) as u8;
    s[13] = (limbs[2] >> 2) as u8;
    s[14] = (limbs[2] >> 10) as u8;
    s[15] = (limbs[2] >> 18) as u8;
    s[16] = (limbs[2] >> 26) as u8;
    s[17] = (limbs[2] >> 34) as u8;
    s[18] = (limbs[2] >> 42) as u8;
    s[19] = ((limbs[2] >> 50) | (limbs[3] << 1)) as u8;
    s[20] = (limbs[3] >> 7) as u8;
    s[21] = (limbs[3] >> 15) as u8;
    s[22] = (limbs[3] >> 23) as u8;
    s[23] = (limbs[3] >> 31) as u8;
    s[24] = (limbs[3] >> 39) as u8;
    s[25] = ((limbs[3] >> 47) | (limbs[4] << 4)) as u8;
    s[26] = (limbs[4] >> 4) as u8;
    s[27] = (limbs[4] >> 12) as u8;
    s[28] = (limbs[4] >> 20) as u8;
    s[29] = (limbs[4] >> 28) as u8;
    s[30] = (limbs[4] >> 36) as u8;
    s[31] = (limbs[4] >> 44) as u8;
    
    s
}

pub(crate) fn fe_reduce(h: &mut Fe) {
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

pub(crate) fn fe_add(a: &Fe, b: &Fe) -> Fe {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2], a[3] + b[3], a[4] + b[4]]
}

pub(crate) fn fe_sub(a: &Fe, b: &Fe) -> Fe {
    // Subtract in GF(2^255-19) by computing a - b + 16*p
    // 16p = 16 * (2^255 - 19) = 2^259 - 304
    // In radix-51: we add enough headroom to avoid underflow
    // Using the same values as curve25519-dalek
    FieldElement51::reduce([
        (a[0] + 36028797018963664u64) - b[0],  // 2^55 - 16*19
        (a[1] + 36028797018963952u64) - b[1],  // 2^55 - 16
        (a[2] + 36028797018963952u64) - b[2],
        (a[3] + 36028797018963952u64) - b[3],
        (a[4] + 36028797018963952u64) - b[4],
    ])
}

/// Reduce 64-bit limbs to fit in 51 bits (weak reduction)
struct FieldElement51;
impl FieldElement51 {
    fn reduce(mut limbs: [u64; 5]) -> Fe {
        const LOW_51_BIT_MASK: u64 = (1u64 << 51) - 1;
        
        let c0 = limbs[0] >> 51;
        let c1 = limbs[1] >> 51;
        let c2 = limbs[2] >> 51;
        let c3 = limbs[3] >> 51;
        let c4 = limbs[4] >> 51;
        
        limbs[0] &= LOW_51_BIT_MASK;
        limbs[1] &= LOW_51_BIT_MASK;
        limbs[2] &= LOW_51_BIT_MASK;
        limbs[3] &= LOW_51_BIT_MASK;
        limbs[4] &= LOW_51_BIT_MASK;
        
        limbs[0] += c4 * 19;
        limbs[1] += c0;
        limbs[2] += c1;
        limbs[3] += c2;
        limbs[4] += c3;
        
        limbs
    }
}

pub(crate) fn fe_mul(a: &Fe, b: &Fe) -> Fe {
    // Schoolbook multiplication with reduction mod 2^255-19
    // The result coefficients r[0..9] need to be computed then folded
    
    let a0 = a[0] as u128;
    let a1 = a[1] as u128;
    let a2 = a[2] as u128;
    let a3 = a[3] as u128;
    let a4 = a[4] as u128;
    
    let b0 = b[0] as u128;
    let b1 = b[1] as u128;
    let b2 = b[2] as u128;
    let b3 = b[3] as u128;
    let b4 = b[4] as u128;
    
    // Products that contribute to each coefficient (before reduction)
    // t0 = a0*b0 + 19*(a1*b4 + a2*b3 + a3*b2 + a4*b1)
    // t1 = a0*b1 + a1*b0 + 19*(a2*b4 + a3*b3 + a4*b2)
    // t2 = a0*b2 + a1*b1 + a2*b0 + 19*(a3*b4 + a4*b3)
    // t3 = a0*b3 + a1*b2 + a2*b1 + a3*b0 + 19*(a4*b4)
    // t4 = a0*b4 + a1*b3 + a2*b2 + a3*b1 + a4*b0
    
    let t0 = a0*b0 + 19*(a1*b4 + a2*b3 + a3*b2 + a4*b1);
    let t1 = a0*b1 + a1*b0 + 19*(a2*b4 + a3*b3 + a4*b2);
    let t2 = a0*b2 + a1*b1 + a2*b0 + 19*(a3*b4 + a4*b3);
    let t3 = a0*b3 + a1*b2 + a2*b1 + a3*b0 + 19*(a4*b4);
    let t4 = a0*b4 + a1*b3 + a2*b2 + a3*b1 + a4*b0;
    
    // Carry propagation
    let mut h = [0u64; 5];
    
    let c = t0 >> 51;
    h[0] = (t0 & 0x7ffffffffffff) as u64;
    let t1 = t1 + c;
    
    let c = t1 >> 51;
    h[1] = (t1 & 0x7ffffffffffff) as u64;
    let t2 = t2 + c;
    
    let c = t2 >> 51;
    h[2] = (t2 & 0x7ffffffffffff) as u64;
    let t3 = t3 + c;
    
    let c = t3 >> 51;
    h[3] = (t3 & 0x7ffffffffffff) as u64;
    let t4 = t4 + c;
    
    let c = t4 >> 51;
    h[4] = (t4 & 0x7ffffffffffff) as u64;
    
    // Reduce: 2^255 = 19 mod p
    h[0] += (c as u64) * 19;
    
    // One more carry if needed
    let c = h[0] >> 51;
    h[0] &= 0x7ffffffffffff;
    h[1] += c;
    
    h
}

pub(crate) fn fe_sq(a: &Fe) -> Fe {
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

/// Compute z^(2^250-1), returning (z^(2^250-1), z^11)
/// This follows the exact addition chain from curve25519-dalek
pub(crate) fn pow22501(z: &Fe) -> (Fe, Fe) {
    // t0 = z^2
    let t0 = fe_sq(z);
    // t1 = z^8 = (z^2)^2^2
    let t1 = fe_sq(&fe_sq(&t0));
    // t2 = z^9 = z^8 * z
    let t2 = fe_mul(z, &t1);
    // t3 = z^11 = z^9 * z^2
    let t3 = fe_mul(&t0, &t2);
    // t4 = z^22 = (z^11)^2
    let t4 = fe_sq(&t3);
    // t5 = z^31 = z^(2^5-1) = z^22 * z^9
    let t5 = fe_mul(&t2, &t4);
    
    // t6 = z^(2^10-1)
    let mut t6 = fe_sq(&t5);
    for _ in 1..5 { t6 = fe_sq(&t6); }
    let t7 = fe_mul(&t6, &t5);
    
    // t8 = z^(2^20-1)
    let mut t8 = fe_sq(&t7);
    for _ in 1..10 { t8 = fe_sq(&t8); }
    let t9 = fe_mul(&t8, &t7);
    
    // t10 = z^(2^40-1)
    let mut t10 = fe_sq(&t9);
    for _ in 1..20 { t10 = fe_sq(&t10); }
    let t11 = fe_mul(&t10, &t9);
    
    // t12 = z^(2^50-1)
    let mut t12 = fe_sq(&t11);
    for _ in 1..10 { t12 = fe_sq(&t12); }
    let t13 = fe_mul(&t12, &t7);
    
    // t14 = z^(2^100-1)
    let mut t14 = fe_sq(&t13);
    for _ in 1..50 { t14 = fe_sq(&t14); }
    let t15 = fe_mul(&t14, &t13);
    
    // t16 = z^(2^200-1)
    let mut t16 = fe_sq(&t15);
    for _ in 1..100 { t16 = fe_sq(&t16); }
    let t17 = fe_mul(&t16, &t15);
    
    // t18 = z^(2^250-1)
    let mut t18 = fe_sq(&t17);
    for _ in 1..50 { t18 = fe_sq(&t18); }
    let t19 = fe_mul(&t18, &t13);
    
    (t19, t3)
}

pub(crate) fn fe_invert(z: &Fe) -> Fe {
    // z^(p-2) = z^(2^255-21)
    // The bits of p-2 = 2^255 - 21 = 2^255 - 32 + 11
    // In binary: 11010111111...11 (253 ones with gaps at positions 2 and 4)
    
    let (t19, t3) = pow22501(z);   // t19: z^(2^250-1), t3: z^11
    
    // t20 = z^(2^255-32) = (z^(2^250-1))^32
    let mut t20 = fe_sq(&t19);
    for _ in 1..5 { t20 = fe_sq(&t20); }
    
    // t21 = z^(2^255-21) = z^(2^255-32) * z^11
    fe_mul(&t20, &t3)
}

pub(crate) fn fe_cswap(a: &mut Fe, b: &mut Fe, swap: u64) {
    let mask = (0u64).wrapping_sub(swap);
    for i in 0..5 {
        let t = mask & (a[i] ^ b[i]);
        a[i] ^= t;
        b[i] ^= t;
    }
}

/// Run crypto self-tests with known test vectors
pub fn run_self_tests() {
    crate::serial_println!("[CRYPTO] Running self-tests...");
    
    // Test AES-128 with NIST test vector
    let aes_key: [u8; 16] = [0x2b, 0x7e, 0x15, 0x16, 0x28, 0xae, 0xd2, 0xa6,
                            0xab, 0xf7, 0x15, 0x88, 0x09, 0xcf, 0x4f, 0x3c];
    let mut block: [u8; 16] = [0x32, 0x43, 0xf6, 0xa8, 0x88, 0x5a, 0x30, 0x8d,
                               0x31, 0x31, 0x98, 0xa2, 0xe0, 0x37, 0x07, 0x34];
    let expected: [u8; 16] = [0x39, 0x25, 0x84, 0x1d, 0x02, 0xdc, 0x09, 0xfb,
                              0xdc, 0x11, 0x85, 0x97, 0x19, 0x6a, 0x0b, 0x32];
    
    let aes = Aes128::new(&aes_key);
    aes.encrypt_block(&mut block);
    
    if block == expected {
        crate::serial_println!("[CRYPTO] AES-128: PASS");
    } else {
        crate::serial_println!("[CRYPTO] AES-128: FAIL");
        crate::serial_println!("[CRYPTO] Expected: {:02x?}", &expected);
        crate::serial_println!("[CRYPTO] Got:      {:02x?}", &block);
    }
    
    // Test SHA-256 with empty input
    let sha_empty = sha256(&[]);
    let sha_expected: [u8; 32] = [
        0xe3, 0xb0, 0xc4, 0x42, 0x98, 0xfc, 0x1c, 0x14,
        0x9a, 0xfb, 0xf4, 0xc8, 0x99, 0x6f, 0xb9, 0x24,
        0x27, 0xae, 0x41, 0xe4, 0x64, 0x9b, 0x93, 0x4c,
        0xa4, 0x95, 0x99, 0x1b, 0x78, 0x52, 0xb8, 0x55,
    ];
    
    if sha_empty == sha_expected {
        crate::serial_println!("[CRYPTO] SHA-256: PASS");
    } else {
        crate::serial_println!("[CRYPTO] SHA-256: FAIL");
        crate::serial_println!("[CRYPTO] Expected: {:02x?}", &sha_expected[..16]);
        crate::serial_println!("[CRYPTO] Got:      {:02x?}", &sha_empty[..16]);
    }
    
    // Test X25519 with RFC 7748 test vector
    let scalar: [u8; 32] = [
        0xa5, 0x46, 0xe3, 0x6b, 0xf0, 0x52, 0x7c, 0x9d,
        0x3b, 0x16, 0x15, 0x4b, 0x82, 0x46, 0x5e, 0xdd,
        0x62, 0x14, 0x4c, 0x0a, 0xc1, 0xfc, 0x5a, 0x18,
        0x50, 0x6a, 0x22, 0x44, 0xba, 0x44, 0x9a, 0xc4,
    ];
    let u_coord: [u8; 32] = [
        0xe6, 0xdb, 0x68, 0x67, 0x58, 0x30, 0x30, 0xdb,
        0x35, 0x94, 0xc1, 0xa4, 0x24, 0xb1, 0x5f, 0x7c,
        0x72, 0x66, 0x24, 0xec, 0x26, 0xb3, 0x35, 0x3b,
        0x10, 0xa9, 0x03, 0xa6, 0xd0, 0xab, 0x1c, 0x4c,
    ];
    
    // Test round-trip: fe_from_bytes -> fe_to_bytes
    let mut u_test = u_coord;
    u_test[31] &= 0x7f;  // Clear top bit
    let fe = fe_from_bytes(&u_test);
    let rt = fe_to_bytes(&fe);
    if rt == u_test {
        crate::serial_println!("[CRYPTO] FE round-trip: PASS");
    } else {
        crate::serial_println!("[CRYPTO] FE round-trip: FAIL");
        crate::serial_println!("[CRYPTO] Input:  {:02x?}", &u_test[..16]);
        crate::serial_println!("[CRYPTO] Output: {:02x?}", &rt[..16]);
    }
    
    // Test fe_mul: 1 * u = u
    let one = FE_ONE;
    let mul_result = fe_mul(&one, &fe);
    let mul_bytes = fe_to_bytes(&mul_result);
    if mul_bytes == u_test {
        crate::serial_println!("[CRYPTO] FE mul identity: PASS");
    } else {
        crate::serial_println!("[CRYPTO] FE mul identity: FAIL");
        crate::serial_println!("[CRYPTO] Expected: {:02x?}", &u_test[..16]);
        crate::serial_println!("[CRYPTO] Got:      {:02x?}", &mul_bytes[..16]);
    }
    
    // Test fe_sq: u^2 round-trip
    let sq_result = fe_sq(&fe);
    // No simple check, just ensure it doesn't crash
    crate::serial_println!("[CRYPTO] FE sq: OK (no crash)");
    
    // Test fe_add and fe_sub: u + 0 = u, u - 0 = u
    let add_result = fe_add(&fe, &FE_ZERO);
    let add_bytes = fe_to_bytes(&add_result);
    if add_bytes == u_test {
        crate::serial_println!("[CRYPTO] FE add identity: PASS");
    } else {
        crate::serial_println!("[CRYPTO] FE add identity: FAIL");
    }
    
    let sub_result = fe_sub(&fe, &FE_ZERO);
    let sub_bytes = fe_to_bytes(&sub_result);
    if sub_bytes == u_test {
        crate::serial_println!("[CRYPTO] FE sub identity: PASS");
    } else {
        crate::serial_println!("[CRYPTO] FE sub identity: FAIL");
        crate::serial_println!("[CRYPTO] Expected: {:02x?}", &u_test[..16]);
        crate::serial_println!("[CRYPTO] Got:      {:02x?}", &sub_bytes[..16]);
    }
    
    // Test consistency: z^2 * z^2 = z^4 = (z^2)^2
    let z2 = fe_sq(&fe);
    let z4_mul = fe_mul(&z2, &z2);      // z^2 * z^2
    let z4_sq = fe_sq(&z2);              // (z^2)^2
    let z4_mul_bytes = fe_to_bytes(&z4_mul);
    let z4_sq_bytes = fe_to_bytes(&z4_sq);
    if z4_mul_bytes == z4_sq_bytes {
        crate::serial_println!("[CRYPTO] FE mul vs sq: PASS");
    } else {
        crate::serial_println!("[CRYPTO] FE mul vs sq: FAIL (z^2*z^2 != sq(z^2))");
        crate::serial_println!("[CRYPTO] mul: {:02x?}", &z4_mul_bytes[..16]);
        crate::serial_println!("[CRYPTO] sq:  {:02x?}", &z4_sq_bytes[..16]);
    }
    
    let one_bytes = fe_to_bytes(&FE_ONE);
    
    // Test simple mul: 2 * 2 = 4
    let two: Fe = [2, 0, 0, 0, 0];
    let four: Fe = [4, 0, 0, 0, 0];
    let two_times_two = fe_mul(&two, &two);
    let four_bytes = fe_to_bytes(&four);
    let tt_bytes = fe_to_bytes(&two_times_two);
    if tt_bytes == four_bytes {
        crate::serial_println!("[CRYPTO] FE 2*2=4: PASS");
    } else {
        crate::serial_println!("[CRYPTO] FE 2*2=4: FAIL");
        crate::serial_println!("[CRYPTO] Expected: {:02x?}", &four_bytes[..16]);
        crate::serial_println!("[CRYPTO] Got:      {:02x?}", &tt_bytes[..16]);
    }
    
    // Test: sq(2) = 4
    let sq_two = fe_sq(&two);
    let sq_two_bytes = fe_to_bytes(&sq_two);
    if sq_two_bytes == four_bytes {
        crate::serial_println!("[CRYPTO] FE sq(2)=4: PASS");
    } else {
        crate::serial_println!("[CRYPTO] FE sq(2)=4: FAIL");
        crate::serial_println!("[CRYPTO] Expected: {:02x?}", &four_bytes[..16]);
        crate::serial_println!("[CRYPTO] Got:      {:02x?}", &sq_two_bytes[..16]);
    }
    
    // Test: 2 * 2^(-1) = 1
    let two_inv = fe_invert(&two);
    let two_mul_inv = fe_mul(&two, &two_inv);
    let two_inv_bytes = fe_to_bytes(&two_mul_inv);
    if two_inv_bytes == one_bytes {
        crate::serial_println!("[CRYPTO] FE 2*2^-1: PASS");
    } else {
        crate::serial_println!("[CRYPTO] FE 2*2^-1: FAIL");
        crate::serial_println!("[CRYPTO] Expected: {:02x?}", &one_bytes[..16]);
        crate::serial_println!("[CRYPTO] Got:      {:02x?}", &two_inv_bytes[..16]);
    }
    
    // Test z * z^(-1) = 1
    let z_inv = fe_invert(&fe);
    let mul_inv = fe_mul(&fe, &z_inv);
    let inv_bytes = fe_to_bytes(&mul_inv);
    if inv_bytes == one_bytes {
        crate::serial_println!("[CRYPTO] FE invert: PASS");
    } else {
        crate::serial_println!("[CRYPTO] FE invert: FAIL");
        crate::serial_println!("[CRYPTO] Expected: {:02x?}", &one_bytes[..16]);
        crate::serial_println!("[CRYPTO] Got:      {:02x?}", &inv_bytes[..16]);
    }
    
    let x25519_expected: [u8; 32] = [
        0xc3, 0xda, 0x55, 0x37, 0x9d, 0xe9, 0xc6, 0x90,
        0x8e, 0x94, 0xea, 0x4d, 0xf2, 0x8d, 0x08, 0x4f,
        0x32, 0xec, 0xcf, 0x03, 0x49, 0x1c, 0x71, 0xf7,
        0x54, 0xb4, 0x07, 0x55, 0x77, 0xa2, 0x85, 0x52,
    ];
    
    let x25519_result = x25519(&scalar, &u_coord);
    
    if x25519_result == x25519_expected {
        crate::serial_println!("[CRYPTO] X25519: PASS");
    } else {
        crate::serial_println!("[CRYPTO] X25519: FAIL");
        crate::serial_println!("[CRYPTO] Expected: {:02x?}", &x25519_expected[..16]);
        crate::serial_println!("[CRYPTO] Got:      {:02x?}", &x25519_result[..16]);
    }
    
    // Test AES-GCM with NIST test vector
    // From NIST GCM test vectors - Test Case 2
    let gcm_key: [u8; 16] = [0x00; 16];
    let gcm_nonce: [u8; 12] = [0x00; 12];
    let gcm_plaintext: [u8; 16] = [0x00; 16];
    let gcm_expected_ct: [u8; 16] = [
        0x03, 0x88, 0xda, 0xce, 0x60, 0xb6, 0xa3, 0x92,
        0xf3, 0x28, 0xc2, 0xb9, 0x71, 0xb2, 0xfe, 0x78,
    ];
    let gcm_expected_tag: [u8; 16] = [
        0xab, 0x6e, 0x47, 0xd4, 0x2c, 0xec, 0x13, 0xbd,
        0xf5, 0x3a, 0x67, 0xb2, 0x12, 0x57, 0xbd, 0xdf,
    ];
    
    let gcm_result = aes_gcm_encrypt(&gcm_key, &gcm_nonce, &[], &gcm_plaintext);
    
    if &gcm_result[..16] == &gcm_expected_ct && &gcm_result[16..] == &gcm_expected_tag {
        crate::serial_println!("[CRYPTO] AES-GCM: PASS");
    } else {
        crate::serial_println!("[CRYPTO] AES-GCM: FAIL");
        crate::serial_println!("[CRYPTO] Expected CT:  {:02x?}", &gcm_expected_ct);
        crate::serial_println!("[CRYPTO] Got CT:       {:02x?}", &gcm_result[..16]);
        crate::serial_println!("[CRYPTO] Expected TAG: {:02x?}", &gcm_expected_tag);
        crate::serial_println!("[CRYPTO] Got TAG:      {:02x?}", &gcm_result[16..]);
    }
    
    crate::serial_println!("[CRYPTO] Self-tests complete");
}

/// Run crypto self-tests and return (passed, failed) counts.
/// Used by the integration test suite (`inttest`).
pub fn verify_crypto() -> (usize, usize) {
    let mut passed = 0usize;
    let mut failed = 0usize;

    // AES-128 NIST test vector
    let aes_key: [u8; 16] = [0x2b,0x7e,0x15,0x16,0x28,0xae,0xd2,0xa6,
                              0xab,0xf7,0x15,0x88,0x09,0xcf,0x4f,0x3c];
    let mut block: [u8; 16] = [0x32,0x43,0xf6,0xa8,0x88,0x5a,0x30,0x8d,
                                0x31,0x31,0x98,0xa2,0xe0,0x37,0x07,0x34];
    let expected_aes: [u8; 16] = [0x39,0x25,0x84,0x1d,0x02,0xdc,0x09,0xfb,
                                   0xdc,0x11,0x85,0x97,0x19,0x6a,0x0b,0x32];
    Aes128::new(&aes_key).encrypt_block(&mut block);
    if block == expected_aes { passed += 1; } else { failed += 1; }

    // SHA-256 empty input
    let sha_empty = sha256(&[]);
    let sha_exp: [u8; 32] = [
        0xe3,0xb0,0xc4,0x42,0x98,0xfc,0x1c,0x14,
        0x9a,0xfb,0xf4,0xc8,0x99,0x6f,0xb9,0x24,
        0x27,0xae,0x41,0xe4,0x64,0x9b,0x93,0x4c,
        0xa4,0x95,0x99,0x1b,0x78,0x52,0xb8,0x55,
    ];
    if sha_empty == sha_exp { passed += 1; } else { failed += 1; }

    // SHA-256 "abc" (NIST)
    let sha_abc = sha256(b"abc");
    let sha_abc_exp: [u8; 32] = [
        0xba,0x78,0x16,0xbf,0x8f,0x01,0xcf,0xea,
        0x41,0x41,0x40,0xde,0x5d,0xae,0x22,0x23,
        0xb0,0x03,0x61,0xa3,0x96,0x17,0x7a,0x9c,
        0xb4,0x10,0xff,0x61,0xf2,0x00,0x15,0xad,
    ];
    if sha_abc == sha_abc_exp { passed += 1; } else { failed += 1; }

    // X25519 RFC 7748 test vector
    let scalar: [u8; 32] = [
        0xa5,0x46,0xe3,0x6b,0xf0,0x52,0x7c,0x9d,
        0x3b,0x16,0x15,0x4b,0x82,0x46,0x5e,0xdd,
        0x62,0x14,0x4c,0x0a,0xc1,0xfc,0x5a,0x18,
        0x50,0x6a,0x22,0x44,0xba,0x44,0x9a,0xc4,
    ];
    let u_coord: [u8; 32] = [
        0xe6,0xdb,0x68,0x67,0x58,0x30,0x30,0xdb,
        0x35,0x94,0xc1,0xa4,0x24,0xb1,0x5f,0x7c,
        0x72,0x66,0x24,0xec,0x26,0xb3,0x35,0x3b,
        0x10,0xa9,0x03,0xa6,0xd0,0xab,0x1c,0x4c,
    ];
    let x25519_exp: [u8; 32] = [
        0xc3,0xda,0x55,0x37,0x9d,0xe9,0xc6,0x90,
        0x8e,0x94,0xea,0x4d,0xf2,0x8d,0x08,0x4f,
        0x32,0xec,0xcf,0x03,0x49,0x1c,0x71,0xf7,
        0x54,0xb4,0x07,0x55,0x77,0xa2,0x85,0x52,
    ];
    let x25519_result = x25519(&scalar, &u_coord);
    if x25519_result == x25519_exp { passed += 1; } else { failed += 1; }

    // AES-GCM NIST Test Case 2 (zero key/nonce, zero plaintext)
    let gcm_key: [u8; 16] = [0x00; 16];
    let gcm_nonce: [u8; 12] = [0x00; 12];
    let gcm_plaintext: [u8; 16] = [0x00; 16];
    let gcm_exp_ct: [u8; 16] = [
        0x03,0x88,0xda,0xce,0x60,0xb6,0xa3,0x92,
        0xf3,0x28,0xc2,0xb9,0x71,0xb2,0xfe,0x78,
    ];
    let gcm_exp_tag: [u8; 16] = [
        0xab,0x6e,0x47,0xd4,0x2c,0xec,0x13,0xbd,
        0xf5,0x3a,0x67,0xb2,0x12,0x57,0xbd,0xdf,
    ];
    let gcm_result = aes_gcm_encrypt(&gcm_key, &gcm_nonce, &[], &gcm_plaintext);
    if &gcm_result[..16] == &gcm_exp_ct && &gcm_result[16..] == &gcm_exp_tag {
        passed += 1;
    } else {
        failed += 1;
    }

    // HMAC-SHA256 with known vector (RFC 4231 Test Case 2)
    let hmac_key = b"Jefe";
    let hmac_data = b"what do ya want for nothing?";
    let hmac_result = hmac_sha256(hmac_key, hmac_data);
    let hmac_expected: [u8; 32] = [
        0x5b,0xdc,0xc1,0x46,0xbf,0x60,0x75,0x4e,
        0x6a,0x04,0x24,0x26,0x08,0x95,0x75,0xc7,
        0x5a,0x00,0x3f,0x08,0x9d,0x27,0x39,0x83,
        0x9d,0xec,0x58,0xb9,0x64,0xec,0x38,0x43,
    ];
    if hmac_result == hmac_expected { passed += 1; } else { failed += 1; }

    (passed, failed)
}
