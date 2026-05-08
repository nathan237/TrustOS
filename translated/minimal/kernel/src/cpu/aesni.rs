




use core::arch::x86_64::*;


pub fn sw() -> bool {
    super::has_aesni()
}


#[repr(C, align(16))]
pub struct Aes128Key {
    round_keys: [__m128i; 11],
}

impl Aes128Key {
    
    pub fn new(key: &[u8; 16]) -> Self {
        if !sw() {
            panic!("AES-NI not available");
        }
        
        unsafe {
            let mut round_keys: [__m128i; 11] = core::mem::zeroed();
            
            
            round_keys[0] = _mm_loadu_si128(key.as_ptr() as *const __m128i);
            
            
            round_keys[1] = bkq(round_keys[0], 0x01);
            round_keys[2] = bkq(round_keys[1], 0x02);
            round_keys[3] = bkq(round_keys[2], 0x04);
            round_keys[4] = bkq(round_keys[3], 0x08);
            round_keys[5] = bkq(round_keys[4], 0x10);
            round_keys[6] = bkq(round_keys[5], 0x20);
            round_keys[7] = bkq(round_keys[6], 0x40);
            round_keys[8] = bkq(round_keys[7], 0x80);
            round_keys[9] = bkq(round_keys[8], 0x1B);
            round_keys[10] = bkq(round_keys[9], 0x36);
            
            Self { round_keys }
        }
    }
    
    
    pub fn encrypt_block(&self, ry: &[u8; 16]) -> [u8; 16] {
        unsafe {
            let mut block = _mm_loadu_si128(ry.as_ptr() as *const __m128i);
            
            
            block = _mm_xor_si128(block, self.round_keys[0]);
            
            
            block = _mm_aesenc_si128(block, self.round_keys[1]);
            block = _mm_aesenc_si128(block, self.round_keys[2]);
            block = _mm_aesenc_si128(block, self.round_keys[3]);
            block = _mm_aesenc_si128(block, self.round_keys[4]);
            block = _mm_aesenc_si128(block, self.round_keys[5]);
            block = _mm_aesenc_si128(block, self.round_keys[6]);
            block = _mm_aesenc_si128(block, self.round_keys[7]);
            block = _mm_aesenc_si128(block, self.round_keys[8]);
            block = _mm_aesenc_si128(block, self.round_keys[9]);
            
            
            block = _mm_aesenclast_si128(block, self.round_keys[10]);
            
            let mut output = [0u8; 16];
            _mm_storeu_si128(output.as_mut_ptr() as *mut __m128i, block);
            output
        }
    }
    
    
    pub fn qci(&self, pw: &[u8; 16]) -> [u8; 16] {
        unsafe {
            let mut block = _mm_loadu_si128(pw.as_ptr() as *const __m128i);
            
            
            block = _mm_xor_si128(block, self.round_keys[10]);
            
            
            block = _mm_aesdec_si128(block, _mm_aesimc_si128(self.round_keys[9]));
            block = _mm_aesdec_si128(block, _mm_aesimc_si128(self.round_keys[8]));
            block = _mm_aesdec_si128(block, _mm_aesimc_si128(self.round_keys[7]));
            block = _mm_aesdec_si128(block, _mm_aesimc_si128(self.round_keys[6]));
            block = _mm_aesdec_si128(block, _mm_aesimc_si128(self.round_keys[5]));
            block = _mm_aesdec_si128(block, _mm_aesimc_si128(self.round_keys[4]));
            block = _mm_aesdec_si128(block, _mm_aesimc_si128(self.round_keys[3]));
            block = _mm_aesdec_si128(block, _mm_aesimc_si128(self.round_keys[2]));
            block = _mm_aesdec_si128(block, _mm_aesimc_si128(self.round_keys[1]));
            
            
            block = _mm_aesdeclast_si128(block, self.round_keys[0]);
            
            let mut output = [0u8; 16];
            _mm_storeu_si128(output.as_mut_ptr() as *mut __m128i, block);
            output
        }
    }
}


#[inline]
unsafe fn bkq(key: __m128i, rcon: i32) -> __m128i {
    let mut key = key;
    let mut gep = _mm_aeskeygenassist_si128(key, rcon);
    gep = _mm_shuffle_epi32(gep, 0xFF);
    key = _mm_xor_si128(key, _mm_slli_si128(key, 4));
    key = _mm_xor_si128(key, _mm_slli_si128(key, 4));
    key = _mm_xor_si128(key, _mm_slli_si128(key, 4));
    _mm_xor_si128(key, gep)
}


pub struct Ahc {
    key: Aes128Key,
    h: __m128i,  
}

impl Ahc {
    
    pub fn new(key: &[u8; 16]) -> Self {
        let ctk = Aes128Key::new(key);
        
        
        let zero = [0u8; 16];
        let mgx = ctk.encrypt_block(&zero);
        
        let h = unsafe { _mm_loadu_si128(mgx.as_ptr() as *const __m128i) };
        
        Self { key: ctk, h }
    }
    
    
    
    pub fn encrypt(&self, akh: &[u8; 12], ry: &[u8], ahh: &[u8]) -> (alloc::vec::Vec<u8>, [u8; 16]) {
        use alloc::vec::Vec;
        
        
        let mut counter = [0u8; 16];
        counter[..12].copy_from_slice(akh);
        counter[15] = 1;
        
        
        let dta = self.key.encrypt_block(&counter);
        
        
        counter[15] = 2;
        
        
        let mut pw = Vec::with_capacity(ry.len());
        
        for df in ry.chunks(16) {
            let geq = self.key.encrypt_block(&counter);
            
            for (i, &byte) in df.iter().enumerate() {
                pw.push(byte ^ geq[i]);
            }
            
            
            igg(&mut counter);
        }
        
        
        let tag = self.compute_tag(&dta, ahh, &pw);
        
        (pw, tag)
    }
    
    
    
    pub fn lcw(&self, akh: &[u8; 12], pw: &[u8], ahh: &[u8], tag: &[u8; 16]) -> Option<alloc::vec::Vec<u8>> {
        use alloc::vec::Vec;
        
        
        let mut counter = [0u8; 16];
        counter[..12].copy_from_slice(akh);
        counter[15] = 1;
        
        
        let dta = self.key.encrypt_block(&counter);
        
        
        let lsr = self.compute_tag(&dta, ahh, pw);
        
        
        let mut jr = 0u8;
        for (a, b) in tag.iter().zip(lsr.iter()) {
            jr |= a ^ b;
        }
        
        if jr != 0 {
            return None; 
        }
        
        
        counter[15] = 2;
        
        let mut ry = Vec::with_capacity(pw.len());
        
        for df in pw.chunks(16) {
            let geq = self.key.encrypt_block(&counter);
            
            for (i, &byte) in df.iter().enumerate() {
                ry.push(byte ^ geq[i]);
            }
            
            igg(&mut counter);
        }
        
        Some(ry)
    }
    
    
    fn compute_tag(&self, dta: &[u8; 16], ahh: &[u8], pw: &[u8]) -> [u8; 16] {
        
        
        
        
        let mut ajt = unsafe { _mm_setzero_si128() };
        
        
        for df in ahh.chunks(16) {
            let mut block = [0u8; 16];
            block[..df.len()].copy_from_slice(df);
            
            let data = unsafe { _mm_loadu_si128(block.as_ptr() as *const __m128i) };
            ajt = unsafe { _mm_xor_si128(ajt, data) };
            ajt = fyz(ajt, self.h);
        }
        
        
        for df in pw.chunks(16) {
            let mut block = [0u8; 16];
            block[..df.len()].copy_from_slice(df);
            
            let data = unsafe { _mm_loadu_si128(block.as_ptr() as *const __m128i) };
            ajt = unsafe { _mm_xor_si128(ajt, data) };
            ajt = fyz(ajt, self.h);
        }
        
        
        let mut cme = [0u8; 16];
        let ffu = (ahh.len() as u64) * 8;
        let fpf = (pw.len() as u64) * 8;
        cme[..8].copy_from_slice(&ffu.to_be_bytes());
        cme[8..16].copy_from_slice(&fpf.to_be_bytes());
        
        let mxw = unsafe { _mm_loadu_si128(cme.as_ptr() as *const __m128i) };
        ajt = unsafe { _mm_xor_si128(ajt, mxw) };
        ajt = fyz(ajt, self.h);
        
        
        let btr = unsafe { _mm_loadu_si128(dta.as_ptr() as *const __m128i) };
        ajt = unsafe { _mm_xor_si128(ajt, btr) };
        
        let mut tag = [0u8; 16];
        unsafe { _mm_storeu_si128(tag.as_mut_ptr() as *mut __m128i, ajt) };
        tag
    }
}


fn igg(counter: &mut [u8; 16]) {
    for i in (12..16).rev() {
        counter[i] = counter[i].wrapping_add(1);
        if counter[i] != 0 {
            break;
        }
    }
}


#[inline]
fn fyz(a: __m128i, b: __m128i) -> __m128i {
    unsafe {
        
        let akp = _mm_clmulepi64_si128(a, b, 0x00);
        let cem = _mm_clmulepi64_si128(a, b, 0x10);
        let eby = _mm_clmulepi64_si128(a, b, 0x01);
        let beb = _mm_clmulepi64_si128(a, b, 0x11);
        
        let cem = _mm_xor_si128(cem, eby);
        let eby = _mm_slli_si128(cem, 8);
        let cem = _mm_srli_si128(cem, 8);
        let akp = _mm_xor_si128(akp, eby);
        let beb = _mm_xor_si128(beb, cem);
        
        
        let azy = _mm_srli_epi32(akp, 31);
        let cru = _mm_srli_epi32(beb, 31);
        let akp = _mm_slli_epi32(akp, 1);
        let beb = _mm_slli_epi32(beb, 1);
        
        let gzc = _mm_srli_si128(azy, 12);
        let cru = _mm_slli_si128(cru, 4);
        let azy = _mm_slli_si128(azy, 4);
        let akp = _mm_or_si128(akp, azy);
        let beb = _mm_or_si128(beb, cru);
        let beb = _mm_or_si128(beb, gzc);
        
        let azy = _mm_slli_epi32(akp, 31);
        let cru = _mm_slli_epi32(akp, 30);
        let gzc = _mm_slli_epi32(akp, 25);
        
        let azy = _mm_xor_si128(azy, cru);
        let azy = _mm_xor_si128(azy, gzc);
        let cru = _mm_srli_si128(azy, 4);
        let azy = _mm_slli_si128(azy, 12);
        let akp = _mm_xor_si128(akp, azy);
        
        let crt = _mm_srli_epi32(akp, 1);
        let cem = _mm_srli_epi32(akp, 2);
        let eby = _mm_srli_epi32(akp, 7);
        let crt = _mm_xor_si128(crt, cem);
        let crt = _mm_xor_si128(crt, eby);
        let crt = _mm_xor_si128(crt, cru);
        let akp = _mm_xor_si128(akp, crt);
        let beb = _mm_xor_si128(beb, akp);
        
        beb
    }
}

extern crate alloc;
