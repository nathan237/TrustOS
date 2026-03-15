//! AES-NI Hardware Acceleration
//!
//! Uses Intel AES-NI instructions for ~10x faster AES encryption
//! than software implementation.

use core::arch::x86_64::*;

/// Check if AES-NI is available
pub fn is_available() -> bool {
    super::has_aesni()
}

/// AES-128 key schedule (11 round keys from 128-bit key)
#[repr(C, align(16))]
pub struct Aes128Key {
    round_keys: [__m128i; 11],
}

impl Aes128Key {
    /// Create key schedule from 16-byte key
    pub fn new(key: &[u8; 16]) -> Self {
        if !is_available() {
            panic!("AES-NI not available");
        }
        
        unsafe {
            let mut round_keys: [__m128i; 11] = core::mem::zeroed();
            
            // Load initial key
            round_keys[0] = _mm_loadu_si128(key.as_ptr() as *const __m128i);
            
            // Key expansion
            round_keys[1] = aes128_key_expand(round_keys[0], 0x01);
            round_keys[2] = aes128_key_expand(round_keys[1], 0x02);
            round_keys[3] = aes128_key_expand(round_keys[2], 0x04);
            round_keys[4] = aes128_key_expand(round_keys[3], 0x08);
            round_keys[5] = aes128_key_expand(round_keys[4], 0x10);
            round_keys[6] = aes128_key_expand(round_keys[5], 0x20);
            round_keys[7] = aes128_key_expand(round_keys[6], 0x40);
            round_keys[8] = aes128_key_expand(round_keys[7], 0x80);
            round_keys[9] = aes128_key_expand(round_keys[8], 0x1B);
            round_keys[10] = aes128_key_expand(round_keys[9], 0x36);
            
            Self { round_keys }
        }
    }
    
    /// Encrypt single 16-byte block
    pub fn encrypt_block(&self, plaintext: &[u8; 16]) -> [u8; 16] {
        unsafe {
            let mut block = _mm_loadu_si128(plaintext.as_ptr() as *const __m128i);
            
            // Initial round
            block = _mm_xor_si128(block, self.round_keys[0]);
            
            // 9 main rounds
            block = _mm_aesenc_si128(block, self.round_keys[1]);
            block = _mm_aesenc_si128(block, self.round_keys[2]);
            block = _mm_aesenc_si128(block, self.round_keys[3]);
            block = _mm_aesenc_si128(block, self.round_keys[4]);
            block = _mm_aesenc_si128(block, self.round_keys[5]);
            block = _mm_aesenc_si128(block, self.round_keys[6]);
            block = _mm_aesenc_si128(block, self.round_keys[7]);
            block = _mm_aesenc_si128(block, self.round_keys[8]);
            block = _mm_aesenc_si128(block, self.round_keys[9]);
            
            // Final round
            block = _mm_aesenclast_si128(block, self.round_keys[10]);
            
            let mut output = [0u8; 16];
            _mm_storeu_si128(output.as_mut_ptr() as *mut __m128i, block);
            output
        }
    }
    
    /// Decrypt single 16-byte block
    pub fn decrypt_block(&self, ciphertext: &[u8; 16]) -> [u8; 16] {
        unsafe {
            let mut block = _mm_loadu_si128(ciphertext.as_ptr() as *const __m128i);
            
            // Initial round with last key
            block = _mm_xor_si128(block, self.round_keys[10]);
            
            // 9 main rounds (in reverse with inverse mix columns)
            block = _mm_aesdec_si128(block, _mm_aesimc_si128(self.round_keys[9]));
            block = _mm_aesdec_si128(block, _mm_aesimc_si128(self.round_keys[8]));
            block = _mm_aesdec_si128(block, _mm_aesimc_si128(self.round_keys[7]));
            block = _mm_aesdec_si128(block, _mm_aesimc_si128(self.round_keys[6]));
            block = _mm_aesdec_si128(block, _mm_aesimc_si128(self.round_keys[5]));
            block = _mm_aesdec_si128(block, _mm_aesimc_si128(self.round_keys[4]));
            block = _mm_aesdec_si128(block, _mm_aesimc_si128(self.round_keys[3]));
            block = _mm_aesdec_si128(block, _mm_aesimc_si128(self.round_keys[2]));
            block = _mm_aesdec_si128(block, _mm_aesimc_si128(self.round_keys[1]));
            
            // Final round
            block = _mm_aesdeclast_si128(block, self.round_keys[0]);
            
            let mut output = [0u8; 16];
            _mm_storeu_si128(output.as_mut_ptr() as *mut __m128i, block);
            output
        }
    }
}

/// AES-128 key expansion helper
#[inline]
unsafe fn aes128_key_expand(key: __m128i, rcon: i32) -> __m128i {
    let mut key = key;
    let mut keygen = _mm_aeskeygenassist_si128(key, rcon);
    keygen = _mm_shuffle_epi32(keygen, 0xFF);
    key = _mm_xor_si128(key, _mm_slli_si128(key, 4));
    key = _mm_xor_si128(key, _mm_slli_si128(key, 4));
    key = _mm_xor_si128(key, _mm_slli_si128(key, 4));
    _mm_xor_si128(key, keygen)
}

/// AES-GCM context using hardware acceleration
pub struct AesGcm {
    key: Aes128Key,
    h: __m128i,  // Hash subkey (encrypted zero block)
}

impl AesGcm {
    /// Create new AES-GCM context
    pub fn new(key: &[u8; 16]) -> Self {
        let aes_key = Aes128Key::new(key);
        
        // Compute H = AES(K, 0)
        let zero = [0u8; 16];
        let h_bytes = aes_key.encrypt_block(&zero);
        
        let h = unsafe { _mm_loadu_si128(h_bytes.as_ptr() as *const __m128i) };
        
        Self { key: aes_key, h }
    }
    
    /// Encrypt with AES-GCM
    /// Returns (ciphertext, tag)
    pub fn encrypt(&self, nonce: &[u8; 12], plaintext: &[u8], aad: &[u8]) -> (alloc::vec::Vec<u8>, [u8; 16]) {
        use alloc::vec::Vec;
        
        // Counter block: nonce || 0x00000001
        let mut counter = [0u8; 16];
        counter[..12].copy_from_slice(nonce);
        counter[15] = 1;
        
        // Encrypt J0 for final tag XOR
        let j0_enc = self.key.encrypt_block(&counter);
        
        // Start with counter = 2
        counter[15] = 2;
        
        // Encrypt plaintext (CTR mode)
        let mut ciphertext = Vec::with_capacity(plaintext.len());
        
        for chunk in plaintext.chunks(16) {
            let keystream = self.key.encrypt_block(&counter);
            
            for (i, &byte) in chunk.iter().enumerate() {
                ciphertext.push(byte ^ keystream[i]);
            }
            
            // Increment counter
            increment_counter(&mut counter);
        }
        
        // Compute authentication tag using GHASH
        let tag = self.compute_tag(&j0_enc, aad, &ciphertext);
        
        (ciphertext, tag)
    }
    
    /// Decrypt with AES-GCM
    /// Returns None if authentication fails
    pub fn decrypt(&self, nonce: &[u8; 12], ciphertext: &[u8], aad: &[u8], tag: &[u8; 16]) -> Option<alloc::vec::Vec<u8>> {
        use alloc::vec::Vec;
        
        // Counter block
        let mut counter = [0u8; 16];
        counter[..12].copy_from_slice(nonce);
        counter[15] = 1;
        
        // Encrypt J0 for final tag XOR
        let j0_enc = self.key.encrypt_block(&counter);
        
        // Verify tag first
        let expected_tag = self.compute_tag(&j0_enc, aad, ciphertext);
        
        // Constant-time compare
        let mut diff = 0u8;
        for (a, b) in tag.iter().zip(expected_tag.iter()) {
            diff |= a ^ b;
        }
        
        if diff != 0 {
            return None; // Authentication failed
        }
        
        // Decrypt (same as encrypt in CTR mode)
        counter[15] = 2;
        
        let mut plaintext = Vec::with_capacity(ciphertext.len());
        
        for chunk in ciphertext.chunks(16) {
            let keystream = self.key.encrypt_block(&counter);
            
            for (i, &byte) in chunk.iter().enumerate() {
                plaintext.push(byte ^ keystream[i]);
            }
            
            increment_counter(&mut counter);
        }
        
        Some(plaintext)
    }
    
    /// Compute GHASH authentication tag
    fn compute_tag(&self, j0_enc: &[u8; 16], aad: &[u8], ciphertext: &[u8]) -> [u8; 16] {
        // GHASH(H, A, C) where:
        // A = additional authenticated data
        // C = ciphertext
        
        let mut ghash = unsafe { _mm_setzero_si128() };
        
        // Process AAD
        for chunk in aad.chunks(16) {
            let mut block = [0u8; 16];
            block[..chunk.len()].copy_from_slice(chunk);
            
            let data = unsafe { _mm_loadu_si128(block.as_ptr() as *const __m128i) };
            ghash = unsafe { _mm_xor_si128(ghash, data) };
            ghash = gf_multiply(ghash, self.h);
        }
        
        // Process ciphertext
        for chunk in ciphertext.chunks(16) {
            let mut block = [0u8; 16];
            block[..chunk.len()].copy_from_slice(chunk);
            
            let data = unsafe { _mm_loadu_si128(block.as_ptr() as *const __m128i) };
            ghash = unsafe { _mm_xor_si128(ghash, data) };
            ghash = gf_multiply(ghash, self.h);
        }
        
        // Length block: [len(A) in bits || len(C) in bits]
        let mut len_block = [0u8; 16];
        let aad_bits = (aad.len() as u64) * 8;
        let ct_bits = (ciphertext.len() as u64) * 8;
        len_block[..8].copy_from_slice(&aad_bits.to_be_bytes());
        len_block[8..16].copy_from_slice(&ct_bits.to_be_bytes());
        
        let len_data = unsafe { _mm_loadu_si128(len_block.as_ptr() as *const __m128i) };
        ghash = unsafe { _mm_xor_si128(ghash, len_data) };
        ghash = gf_multiply(ghash, self.h);
        
        // XOR with encrypted J0
        let j0 = unsafe { _mm_loadu_si128(j0_enc.as_ptr() as *const __m128i) };
        ghash = unsafe { _mm_xor_si128(ghash, j0) };
        
        let mut tag = [0u8; 16];
        unsafe { _mm_storeu_si128(tag.as_mut_ptr() as *mut __m128i, ghash) };
        tag
    }
}

/// Increment 128-bit counter (big-endian, last 4 bytes)
fn increment_counter(counter: &mut [u8; 16]) {
    for i in (12..16).rev() {
        counter[i] = counter[i].wrapping_add(1);
        if counter[i] != 0 {
            break;
        }
    }
}

/// GF(2^128) multiplication using PCLMULQDQ
#[inline]
fn gf_multiply(a: __m128i, b: __m128i) -> __m128i {
    unsafe {
        // Carryless multiplication
        let tmp3 = _mm_clmulepi64_si128(a, b, 0x00);
        let tmp4 = _mm_clmulepi64_si128(a, b, 0x10);
        let tmp5 = _mm_clmulepi64_si128(a, b, 0x01);
        let tmp6 = _mm_clmulepi64_si128(a, b, 0x11);
        
        let tmp4 = _mm_xor_si128(tmp4, tmp5);
        let tmp5 = _mm_slli_si128(tmp4, 8);
        let tmp4 = _mm_srli_si128(tmp4, 8);
        let tmp3 = _mm_xor_si128(tmp3, tmp5);
        let tmp6 = _mm_xor_si128(tmp6, tmp4);
        
        // Reduction modulo x^128 + x^7 + x^2 + x + 1
        let tmp7 = _mm_srli_epi32(tmp3, 31);
        let tmp8 = _mm_srli_epi32(tmp6, 31);
        let tmp3 = _mm_slli_epi32(tmp3, 1);
        let tmp6 = _mm_slli_epi32(tmp6, 1);
        
        let tmp9 = _mm_srli_si128(tmp7, 12);
        let tmp8 = _mm_slli_si128(tmp8, 4);
        let tmp7 = _mm_slli_si128(tmp7, 4);
        let tmp3 = _mm_or_si128(tmp3, tmp7);
        let tmp6 = _mm_or_si128(tmp6, tmp8);
        let tmp6 = _mm_or_si128(tmp6, tmp9);
        
        let tmp7 = _mm_slli_epi32(tmp3, 31);
        let tmp8 = _mm_slli_epi32(tmp3, 30);
        let tmp9 = _mm_slli_epi32(tmp3, 25);
        
        let tmp7 = _mm_xor_si128(tmp7, tmp8);
        let tmp7 = _mm_xor_si128(tmp7, tmp9);
        let tmp8 = _mm_srli_si128(tmp7, 4);
        let tmp7 = _mm_slli_si128(tmp7, 12);
        let tmp3 = _mm_xor_si128(tmp3, tmp7);
        
        let tmp2 = _mm_srli_epi32(tmp3, 1);
        let tmp4 = _mm_srli_epi32(tmp3, 2);
        let tmp5 = _mm_srli_epi32(tmp3, 7);
        let tmp2 = _mm_xor_si128(tmp2, tmp4);
        let tmp2 = _mm_xor_si128(tmp2, tmp5);
        let tmp2 = _mm_xor_si128(tmp2, tmp8);
        let tmp3 = _mm_xor_si128(tmp3, tmp2);
        let tmp6 = _mm_xor_si128(tmp6, tmp3);
        
        tmp6
    }
}

extern crate alloc;
