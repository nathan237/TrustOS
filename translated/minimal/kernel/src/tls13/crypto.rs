







use alloc::vec::Vec;






const CWF_: [u32; 8] = [
    0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a,
    0x510e527f, 0x9b05688c, 0x1f83d9ab, 0x5be0cd19,
];


const CWG_: [u32; 64] = [
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
            state: CWF_,
            buffer: [0u8; 64],
            buffer_len: 0,
            total_len: 0,
        }
    }
    
    pub fn update(&mut self, data: &[u8]) {
        let mut offset = 0usize;
        let len = data.len();

        
        if self.buffer_len > 0 {
            let duy = 64 - self.buffer_len;
            let take = if len < duy { len } else { duy };
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

        
        while offset + 64 <= len {
            self.buffer.copy_from_slice(&data[offset..offset + 64]);
            self.total_len += 64;
            self.process_block();
            offset += 64;
        }

        
        let ck = len - offset;
        if ck > 0 {
            self.buffer[..ck].copy_from_slice(&data[offset..]);
            self.buffer_len = ck;
            self.total_len += ck as u64;
        }
    }
    
    pub fn finalize(&mut self) -> [u8; 32] {
        
        let djk = self.total_len * 8;
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
        
        
        self.buffer[56..64].copy_from_slice(&djk.to_be_bytes());
        self.process_block();
        
        
        let mut result = [0u8; 32];
        for (i, &fx) in self.state.iter().enumerate() {
            result[i * 4..i * 4 + 4].copy_from_slice(&fx.to_be_bytes());
        }
        result
    }
    
    fn process_block(&mut self) {
        let mut w = [0u32; 64];
        
        
        for i in 0..16 {
            w[i] = u32::from_be_bytes([
                self.buffer[i * 4],
                self.buffer[i * 4 + 1],
                self.buffer[i * 4 + 2],
                self.buffer[i * 4 + 3],
            ]);
        }
        
        for i in 16..64 {
            let auz = w[i - 15].rotate_right(7) ^ w[i - 15].rotate_right(18) ^ (w[i - 15] >> 3);
            let afq = w[i - 2].rotate_right(17) ^ w[i - 2].rotate_right(19) ^ (w[i - 2] >> 10);
            w[i] = w[i - 16].wrapping_add(auz).wrapping_add(w[i - 7]).wrapping_add(afq);
        }
        
        
        let [mut a, mut b, mut c, mut d, mut e, mut f, mut g, mut h] = self.state;
        
        
        for i in 0..64 {
            let afq = e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25);
            let ch = (e & f) ^ ((!e) & g);
            let cee = h.wrapping_add(afq).wrapping_add(ch).wrapping_add(CWG_[i]).wrapping_add(w[i]);
            let auz = a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22);
            let dtx = (a & b) ^ (a & c) ^ (b & c);
            let ebj = auz.wrapping_add(dtx);
            
            h = g;
            g = f;
            f = e;
            e = d.wrapping_add(cee);
            d = c;
            c = b;
            b = a;
            a = cee.wrapping_add(ebj);
        }
        
        
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


pub fn asg(data: &[u8]) -> [u8; 32] {
    let mut bgv = Sha256::new();
    bgv.update(data);
    bgv.finalize()
}


pub fn bmu(key: &[u8], data: &[u8]) -> [u8; 32] {
    let mut esb = [0u8; 64];
    
    if key.len() > 64 {
        let mvq = asg(key);
        esb[..32].copy_from_slice(&mvq);
    } else {
        esb[..key.len()].copy_from_slice(key);
    }
    
    
    let mut iho = [0x36u8; 64];
    for (i, &k) in esb.iter().enumerate() {
        iho[i] ^= k;
    }
    
    let mut inner = Sha256::new();
    inner.update(&iho);
    inner.update(data);
    let mqf = inner.finalize();
    
    
    let mut isj = [0x5cu8; 64];
    for (i, &k) in esb.iter().enumerate() {
        isj[i] ^= k;
    }
    
    let mut glh = Sha256::new();
    glh.update(&isj);
    glh.update(&mqf);
    glh.finalize()
}






pub fn epg(salt: &[u8], ikm: &[u8]) -> [u8; 32] {
    let salt = if salt.is_empty() { &[0u8; 32] } else { salt };
    bmu(salt, ikm)
}


pub fn mlu(prk: &[u8; 32], info: &[u8], length: usize) -> Vec<u8> {
    let mut output = Vec::with_capacity(length);
    let mut t = Vec::new();
    let mut counter = 1u8;
    
    while output.len() < length {
        let mut input = Vec::new();
        input.extend_from_slice(&t);
        input.extend_from_slice(info);
        input.push(counter);
        
        t = bmu(prk, &input).to_vec();
        output.extend_from_slice(&t);
        counter += 1;
    }
    
    output.truncate(length);
    output
}


pub fn czf(bvr: &[u8; 32], label: &str, context: &[u8], length: usize) -> Vec<u8> {
    let maf = alloc::format!("tls13 {}", label);
    let ijf = maf.as_bytes();
    
    let mut info = Vec::new();
    info.extend_from_slice(&(length as u16).to_be_bytes());
    info.push(ijf.len() as u8);
    info.extend_from_slice(ijf);
    info.push(context.len() as u8);
    info.extend_from_slice(context);
    
    mlu(bvr, &info, length)
}


pub fn cid(bvr: &[u8; 32], label: &str, transcript_hash: &[u8; 32]) -> [u8; 32] {
    let expanded = czf(bvr, label, transcript_hash, 32);
    let mut result = [0u8; 32];
    result.copy_from_slice(&expanded);
    result
}






const Nf: [u8; 256] = [
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


const Anm: [u8; 10] = [0x01, 0x02, 0x04, 0x08, 0x10, 0x20, 0x40, 0x80, 0x1b, 0x36];



pub struct Aes128 {
    round_keys: [[u8; 16]; 11],
}

impl Aes128 {
    pub fn new(key: &[u8; 16]) -> Self {
        let mut round_keys = [[0u8; 16]; 11];
        round_keys[0].copy_from_slice(key);
        
        
        for i in 1..11 {
            let prev = round_keys[i - 1];
            let mut fx = [prev[12], prev[13], prev[14], prev[15]];
            
            
            let ts = fx[0];
            fx[0] = Nf[fx[1] as usize] ^ Anm[i - 1];
            fx[1] = Nf[fx[2] as usize];
            fx[2] = Nf[fx[3] as usize];
            fx[3] = Nf[ts as usize];
            
            for ay in 0..4 {
                round_keys[i][ay] = prev[ay] ^ fx[ay];
            }
            for ay in 4..16 {
                round_keys[i][ay] = prev[ay] ^ round_keys[i][ay - 4];
            }
        }
        
        Self { round_keys }
    }
    
    
    pub fn encrypt_block(&self, block: &mut [u8; 16]) {
        self.encrypt_block_software(block);
    }
    
    
    fn encrypt_block_software(&self, block: &mut [u8; 16]) {
        
        for i in 0..16 {
            block[i] ^= self.round_keys[0][i];
        }
        
        
        for round in 1..10 {
            self.sub_bytes(block);
            self.shift_rows(block);
            self.mix_columns(block);
            for i in 0..16 {
                block[i] ^= self.round_keys[round][i];
            }
        }
        
        
        self.sub_bytes(block);
        self.shift_rows(block);
        for i in 0..16 {
            block[i] ^= self.round_keys[10][i];
        }
    }
    
    fn sub_bytes(&self, block: &mut [u8; 16]) {
        for byte in block.iter_mut() {
            *byte = Nf[*byte as usize];
        }
    }
    
    fn shift_rows(&self, block: &mut [u8; 16]) {
        
        let ts = block[1];
        block[1] = block[5];
        block[5] = block[9];
        block[9] = block[13];
        block[13] = ts;
        
        
        let (abl, ll) = (block[2], block[6]);
        block[2] = block[10];
        block[6] = block[14];
        block[10] = abl;
        block[14] = ll;
        
        
        let ts = block[15];
        block[15] = block[11];
        block[11] = block[7];
        block[7] = block[3];
        block[3] = ts;
    }
    
    fn mix_columns(&self, block: &mut [u8; 16]) {
        for i in 0..4 {
            let col = i * 4;
            let (a, b, c, d) = (block[col], block[col + 1], block[col + 2], block[col + 3]);
            
            block[col] = cad(a, 2) ^ cad(b, 3) ^ c ^ d;
            block[col + 1] = a ^ cad(b, 2) ^ cad(c, 3) ^ d;
            block[col + 2] = a ^ b ^ cad(c, 2) ^ cad(d, 3);
            block[col + 3] = cad(a, 3) ^ b ^ c ^ cad(d, 2);
        }
    }
}


fn cad(a: u8, b: u8) -> u8 {
    let mut result = 0u8;
    let mut a = a;
    let mut b = b;
    
    while b != 0 {
        if b & 1 != 0 {
            result ^= a;
        }
        let mlf = a & 0x80;
        a <<= 1;
        if mlf != 0 {
            a ^= 0x1b;
        }
        b >>= 1;
    }
    result
}


fn fyc(x: &[u8; 16], h: &[u8; 16]) -> [u8; 16] {
    let mut z = [0u8; 16];
    let mut v = *h;
    
    for i in 0..16 {
        for bf in 0..8 {
            if (x[i] >> (7 - bf)) & 1 == 1 {
                for ay in 0..16 {
                    z[ay] ^= v[ay];
                }
            }
            
            
            let nbc = v[15] & 1;
            for ay in (1..16).rev() {
                v[ay] = (v[ay] >> 1) | (v[ay - 1] << 7);
            }
            v[0] >>= 1;
            
            if nbc != 0 {
                v[0] ^= 0xe1; 
            }
        }
    }
    
    z
}


fn ajt(h: &[u8; 16], ahh: &[u8], pw: &[u8]) -> [u8; 16] {
    let mut y = [0u8; 16];
    
    
    let jta = (ahh.len() + 15) / 16;
    for i in 0..jta {
        let start = i * 16;
        let end = (start + 16).min(ahh.len());
        for ay in start..end {
            y[ay - start] ^= ahh[ay];
        }
        y = fyc(&y, h);
    }
    
    
    let kzt = (pw.len() + 15) / 16;
    for i in 0..kzt {
        let start = i * 16;
        let end = (start + 16).min(pw.len());
        for ay in start..end {
            y[ay - start] ^= pw[ay];
        }
        y = fyc(&y, h);
    }
    
    
    let ffu = (ahh.len() as u64) * 8;
    let fpf = (pw.len() as u64) * 8;
    let mut cme = [0u8; 16];
    cme[..8].copy_from_slice(&ffu.to_be_bytes());
    cme[8..].copy_from_slice(&fpf.to_be_bytes());
    
    for i in 0..16 {
        y[i] ^= cme[i];
    }
    fyc(&y, h)
}


pub fn efc(
    key: &[u8; 16],
    akh: &[u8; 12],
    ahh: &[u8],
    ry: &[u8],
) -> Vec<u8> {
    let ben = Aes128::new(key);
    
    
    let mut h = [0u8; 16];
    ben.encrypt_block(&mut h);
    
    
    let mut btr = [0u8; 16];
    btr[..12].copy_from_slice(akh);
    btr[15] = 1;
    
    
    let mut doh = btr;
    ben.encrypt_block(&mut doh);
    
    
    let mut pw = Vec::with_capacity(ry.len() + 16);
    let blocks = (ry.len() + 15) / 16;
    
    for i in 0..blocks {
        
        let counter = 2u32 + i as u32;
        let mut block = [0u8; 16];
        block[..12].copy_from_slice(akh);
        block[12..16].copy_from_slice(&counter.to_be_bytes());
        ben.encrypt_block(&mut block);
        
        let start = i * 16;
        let end = (start + 16).min(ry.len());
        for ay in start..end {
            pw.push(ry[ay] ^ block[ay - start]);
        }
    }
    
    
    let fza = ajt(&h, ahh, &pw);
    let mut tag = [0u8; 16];
    for i in 0..16 {
        tag[i] = fza[i] ^ doh[i];
    }
    
    pw.extend_from_slice(&tag);
    pw
}


pub fn hee(
    key: &[u8; 16],
    akh: &[u8; 12],
    ahh: &[u8],
    ciphertext_with_tag: &[u8],
) -> Result<Vec<u8>, ()> {
    if ciphertext_with_tag.len() < 16 {
        return Err(());
    }
    
    let jli = ciphertext_with_tag.len() - 16;
    let pw = &ciphertext_with_tag[..jli];
    let tag = &ciphertext_with_tag[jli..];
    
    let ben = Aes128::new(key);
    
    
    let mut h = [0u8; 16];
    ben.encrypt_block(&mut h);
    
    
    let mut btr = [0u8; 16];
    btr[..12].copy_from_slice(akh);
    btr[15] = 1;
    
    let mut doh = btr;
    ben.encrypt_block(&mut doh);
    
    
    let fza = ajt(&h, ahh, pw);
    let mut fny = [0u8; 16];
    for i in 0..16 {
        fny[i] = fza[i] ^ doh[i];
    }
    
    
    let mut jr = 0u8;
    for i in 0..16 {
        jr |= fny[i] ^ tag[i];
    }
    if jr != 0 {
        crate::serial_println!("[AES-GCM] Tag mismatch! computed={:02x?} received={:02x?}", 
            &fny[..8], &tag[..8]);
        return Err(());
    }
    
    
    let mut ry = Vec::with_capacity(pw.len());
    let blocks = (pw.len() + 15) / 16;
    
    for i in 0..blocks {
        let counter = 2u32 + i as u32;
        let mut block = [0u8; 16];
        block[..12].copy_from_slice(akh);
        block[12..16].copy_from_slice(&counter.to_be_bytes());
        ben.encrypt_block(&mut block);
        
        let start = i * 16;
        let end = (start + 16).min(pw.len());
        for ay in start..end {
            ry.push(pw[ay] ^ block[ay - start]);
        }
    }
    
    Ok(ry)
}






pub fn pvn(private_key: &[u8; 32]) -> [u8; 32] {
    
    let mut base = [0u8; 32];
    base[0] = 9;
    ffl(private_key, &base)
}



pub fn ffl(k: &[u8; 32], iy: &[u8; 32]) -> [u8; 32] {
    
    let mut aeh = *k;
    aeh[0] &= 248;
    aeh[31] &= 127;
    aeh[31] |= 64;
    
    
    let mut jpa = *iy;
    jpa[31] &= 127;
    
    
    let hef = bzp(&jpa);
    
    
    let mut ffj = EA_;
    let mut ffk = IR_;
    
    
    let mut hct = hef;
    let mut hcu = EA_;
    
    
    
    
    let mut gny = false;
    
    for i in (0..255).rev() {
        let hpj = ((aeh[i >> 3] >> (i & 7)) & 1) == 1;
        let hkw = (gny ^ hpj) as u64;
        
        
        emj(&mut ffj, &mut hct, hkw);
        emj(&mut ffk, &mut hcu, hkw);
        
        
        lem(&mut ffj, &mut ffk, &mut hct, &mut hcu, &hef);
        
        gny = hpj;
    }
    
    
    let hys = gny as u64;
    emj(&mut ffj, &mut ffk, hys);
    emj(&mut hct, &mut hcu, hys);
    
    
    let result = md(&ffj, &dpk(&ffk));
    ahy(&result)
}








fn lem(
    p_u: &mut Ab, p_w: &mut Ab,
    q_u: &mut Ab, q_w: &mut Ab,
    affine_pmq: &Ab,
) {
    let abl = ajr(p_u, p_w);
    let ll = aql(p_u, p_w);
    let np = ajr(q_u, q_w);
    let acw = aql(q_u, q_w);
    
    let bdx = py(&abl);    
    let bwd = py(&ll);    
    
    let bwe = aql(&bdx, &bwd);  
    
    let dff = md(&abl, &acw);  
    let dfg = md(&ll, &np);  
    
    let fcg = ajr(&dff, &dfg);  
    let eaw = aql(&dff, &dfg); 
    
    let gxq = py(&fcg);       
    let eay = py(&eaw);      
    
    let eaz = luq(&bwe); 
    
    let eba = md(&bdx, &bwd); 
    let fcf = ajr(&eaz, &bwd); 
    let ebb = md(&bwe, &fcf); 
    
    let gxr = md(affine_pmq, &eay); 
    let ebc = gxq;                       
    
    
    *p_u = eba;  
    *p_w = ebb;  
    
    
    *q_u = ebc;  
    *q_w = gxr;  
}


pub(crate) type Ab = [u64; 5];

pub(crate) const IR_: Ab = [0, 0, 0, 0, 0];
pub(crate) const EA_: Ab = [1, 0, 0, 0, 0];

pub(crate) fn bzp(b: &[u8; 32]) -> Ab {
    let mut h = [0u64; 5];
    
    fn dtp(b: &[u8]) -> u64 {
        let mut result = 0u64;
        for i in 0..8.min(b.len()) {
            result |= (b[i] as u64) << (i * 8);
        }
        result
    }
    
    h[0] = dtp(&b[0..8]) & 0x7ffffffffffff;
    h[1] = (dtp(&b[6..14]) >> 3) & 0x7ffffffffffff;
    h[2] = (dtp(&b[12..20]) >> 6) & 0x7ffffffffffff;
    h[3] = (dtp(&b[19..27]) >> 1) & 0x7ffffffffffff;
    h[4] = (dtp(&b[24..32]) >> 12) & 0x7ffffffffffff;
    
    h
}

pub(crate) fn ahy(h: &Ab) -> [u8; 32] {
    
    let mut hp = *h;
    fwh(&mut hp);
    
    
    
    
    
    let mut q = (hp[0] + 19) >> 51;
    q = (hp[1] + q) >> 51;
    q = (hp[2] + q) >> 51;
    q = (hp[3] + q) >> 51;
    q = (hp[4] + q) >> 51;
    
    
    hp[0] += 19 * q;
    
    
    const EF_: u64 = (1u64 << 51) - 1;
    hp[1] += hp[0] >> 51;
    hp[0] &= EF_;
    hp[2] += hp[1] >> 51;
    hp[1] &= EF_;
    hp[3] += hp[2] >> 51;
    hp[2] &= EF_;
    hp[4] += hp[3] >> 51;
    hp[3] &= EF_;
    
    hp[4] &= EF_;
    
    
    let mut j = [0u8; 32];
    j[0] = hp[0] as u8;
    j[1] = (hp[0] >> 8) as u8;
    j[2] = (hp[0] >> 16) as u8;
    j[3] = (hp[0] >> 24) as u8;
    j[4] = (hp[0] >> 32) as u8;
    j[5] = (hp[0] >> 40) as u8;
    j[6] = ((hp[0] >> 48) | (hp[1] << 3)) as u8;
    j[7] = (hp[1] >> 5) as u8;
    j[8] = (hp[1] >> 13) as u8;
    j[9] = (hp[1] >> 21) as u8;
    j[10] = (hp[1] >> 29) as u8;
    j[11] = (hp[1] >> 37) as u8;
    j[12] = ((hp[1] >> 45) | (hp[2] << 6)) as u8;
    j[13] = (hp[2] >> 2) as u8;
    j[14] = (hp[2] >> 10) as u8;
    j[15] = (hp[2] >> 18) as u8;
    j[16] = (hp[2] >> 26) as u8;
    j[17] = (hp[2] >> 34) as u8;
    j[18] = (hp[2] >> 42) as u8;
    j[19] = ((hp[2] >> 50) | (hp[3] << 1)) as u8;
    j[20] = (hp[3] >> 7) as u8;
    j[21] = (hp[3] >> 15) as u8;
    j[22] = (hp[3] >> 23) as u8;
    j[23] = (hp[3] >> 31) as u8;
    j[24] = (hp[3] >> 39) as u8;
    j[25] = ((hp[3] >> 47) | (hp[4] << 4)) as u8;
    j[26] = (hp[4] >> 4) as u8;
    j[27] = (hp[4] >> 12) as u8;
    j[28] = (hp[4] >> 20) as u8;
    j[29] = (hp[4] >> 28) as u8;
    j[30] = (hp[4] >> 36) as u8;
    j[31] = (hp[4] >> 44) as u8;
    
    j
}

pub(crate) fn fwh(h: &mut Ab) {
    let mut ahn;
    
    for _ in 0..2 {
        ahn = h[0] >> 51;
        h[0] &= 0x7ffffffffffff;
        h[1] += ahn;
        
        ahn = h[1] >> 51;
        h[1] &= 0x7ffffffffffff;
        h[2] += ahn;
        
        ahn = h[2] >> 51;
        h[2] &= 0x7ffffffffffff;
        h[3] += ahn;
        
        ahn = h[3] >> 51;
        h[3] &= 0x7ffffffffffff;
        h[4] += ahn;
        
        ahn = h[4] >> 51;
        h[4] &= 0x7ffffffffffff;
        h[0] += ahn * 19;
    }
}

pub(crate) fn ajr(a: &Ab, b: &Ab) -> Ab {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2], a[3] + b[3], a[4] + b[4]]
}

pub(crate) fn aql(a: &Ab, b: &Ab) -> Ab {
    
    
    
    
    FieldElement51::reduce([
        (a[0] + 36028797018963664u64) - b[0],  
        (a[1] + 36028797018963952u64) - b[1],  
        (a[2] + 36028797018963952u64) - b[2],
        (a[3] + 36028797018963952u64) - b[3],
        (a[4] + 36028797018963952u64) - b[4],
    ])
}


struct FieldElement51;
impl FieldElement51 {
    fn reduce(mut hp: [u64; 5]) -> Ab {
        const EF_: u64 = (1u64 << 51) - 1;
        
        let og = hp[0] >> 51;
        let hw = hp[1] >> 51;
        let jf = hp[2] >> 51;
        let bfc = hp[3] >> 51;
        let fkm = hp[4] >> 51;
        
        hp[0] &= EF_;
        hp[1] &= EF_;
        hp[2] &= EF_;
        hp[3] &= EF_;
        hp[4] &= EF_;
        
        hp[0] += fkm * 19;
        hp[1] += og;
        hp[2] += hw;
        hp[3] += jf;
        hp[4] += bfc;
        
        hp
    }
}

pub(crate) fn md(a: &Ab, b: &Ab) -> Ab {
    
    
    
    let abn = a[0] as u128;
    let eb = a[1] as u128;
    let fy = a[2] as u128;
    let kb = a[3] as u128;
    let aeq = a[4] as u128;
    
    let kl = b[0] as u128;
    let gf = b[1] as u128;
    let iq = b[2] as u128;
    let sc = b[3] as u128;
    let abr = b[4] as u128;
    
    
    
    
    
    
    
    
    let abl = abn*kl + 19*(eb*abr + fy*sc + kb*iq + aeq*gf);
    let ll = abn*gf + eb*kl + 19*(fy*abr + kb*sc + aeq*iq);
    let np = abn*iq + eb*gf + fy*kl + 19*(kb*abr + aeq*sc);
    let acw = abn*sc + eb*iq + fy*gf + kb*kl + 19*(aeq*abr);
    let bdx = abn*abr + eb*sc + fy*iq + kb*gf + aeq*kl;
    
    
    let mut h = [0u64; 5];
    
    let c = abl >> 51;
    h[0] = (abl & 0x7ffffffffffff) as u64;
    let ll = ll + c;
    
    let c = ll >> 51;
    h[1] = (ll & 0x7ffffffffffff) as u64;
    let np = np + c;
    
    let c = np >> 51;
    h[2] = (np & 0x7ffffffffffff) as u64;
    let acw = acw + c;
    
    let c = acw >> 51;
    h[3] = (acw & 0x7ffffffffffff) as u64;
    let bdx = bdx + c;
    
    let c = bdx >> 51;
    h[4] = (bdx & 0x7ffffffffffff) as u64;
    
    
    h[0] += (c as u64) * 19;
    
    
    let c = h[0] >> 51;
    h[0] &= 0x7ffffffffffff;
    h[1] += c;
    
    h
}

pub(crate) fn py(a: &Ab) -> Ab {
    md(a, a)
}

fn luq(a: &Ab) -> Ab {
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
    
    fwh(&mut h);
    h
}



pub(crate) fn gnt(z: &Ab) -> (Ab, Ab) {
    
    let abl = py(z);
    
    let ll = py(&py(&abl));
    
    let np = md(z, &ll);
    
    let acw = md(&abl, &np);
    
    let bdx = py(&acw);
    
    let bwd = md(&np, &bdx);
    
    
    let mut bwe = py(&bwd);
    for _ in 1..5 { bwe = py(&bwe); }
    let dff = md(&bwe, &bwd);
    
    
    let mut dfg = py(&dff);
    for _ in 1..10 { dfg = py(&dfg); }
    let fcg = md(&dfg, &dff);
    
    
    let mut eaw = py(&fcg);
    for _ in 1..20 { eaw = py(&eaw); }
    let gxq = md(&eaw, &fcg);
    
    
    let mut eay = py(&gxq);
    for _ in 1..10 { eay = py(&eay); }
    let eaz = md(&eay, &dff);
    
    
    let mut eba = py(&eaz);
    for _ in 1..50 { eba = py(&eba); }
    let fcf = md(&eba, &eaz);
    
    
    let mut ebb = py(&fcf);
    for _ in 1..100 { ebb = py(&ebb); }
    let gxr = md(&ebb, &fcf);
    
    
    let mut ebc = py(&gxr);
    for _ in 1..50 { ebc = py(&ebc); }
    let gxs = md(&ebc, &eaz);
    
    (gxs, acw)
}

pub(crate) fn dpk(z: &Ab) -> Ab {
    
    
    
    
    let (gxs, acw) = gnt(z);   
    
    
    let mut gxt = py(&gxs);
    for _ in 1..5 { gxt = py(&gxt); }
    
    
    md(&gxt, &acw)
}

pub(crate) fn emj(a: &mut Ab, b: &mut Ab, swap: u64) {
    let mask = (0u64).wrapping_sub(swap);
    for i in 0..5 {
        let t = mask & (a[i] ^ b[i]);
        a[i] ^= t;
        b[i] ^= t;
    }
}


pub fn jbv() {
    crate::serial_println!("[CRYPTO] Running self-tests...");
    
    
    let ctk: [u8; 16] = [0x2b, 0x7e, 0x15, 0x16, 0x28, 0xae, 0xd2, 0xa6,
                            0xab, 0xf7, 0x15, 0x88, 0x09, 0xcf, 0x4f, 0x3c];
    let mut block: [u8; 16] = [0x32, 0x43, 0xf6, 0xa8, 0x88, 0x5a, 0x30, 0x8d,
                               0x31, 0x31, 0x98, 0xa2, 0xe0, 0x37, 0x07, 0x34];
    let expected: [u8; 16] = [0x39, 0x25, 0x84, 0x1d, 0x02, 0xdc, 0x09, 0xfb,
                              0xdc, 0x11, 0x85, 0x97, 0x19, 0x6a, 0x0b, 0x32];
    
    let ben = Aes128::new(&ctk);
    ben.encrypt_block(&mut block);
    
    if block == expected {
        crate::serial_println!("[CRYPTO] AES-128: PASS");
    } else {
        crate::serial_println!("[CRYPTO] AES-128: FAIL");
        crate::serial_println!("[CRYPTO] Expected: {:02x?}", &expected);
        crate::serial_println!("[CRYPTO] Got:      {:02x?}", &block);
    }
    
    
    let fao = asg(&[]);
    let jga: [u8; 32] = [
        0xe3, 0xb0, 0xc4, 0x42, 0x98, 0xfc, 0x1c, 0x14,
        0x9a, 0xfb, 0xf4, 0xc8, 0x99, 0x6f, 0xb9, 0x24,
        0x27, 0xae, 0x41, 0xe4, 0x64, 0x9b, 0x93, 0x4c,
        0xa4, 0x95, 0x99, 0x1b, 0x78, 0x52, 0xb8, 0x55,
    ];
    
    if fao == jga {
        crate::serial_println!("[CRYPTO] SHA-256: PASS");
    } else {
        crate::serial_println!("[CRYPTO] SHA-256: FAIL");
        crate::serial_println!("[CRYPTO] Expected: {:02x?}", &jga[..16]);
        crate::serial_println!("[CRYPTO] Got:      {:02x?}", &fao[..16]);
    }
    
    
    let aeh: [u8; 32] = [
        0xa5, 0x46, 0xe3, 0x6b, 0xf0, 0x52, 0x7c, 0x9d,
        0x3b, 0x16, 0x15, 0x4b, 0x82, 0x46, 0x5e, 0xdd,
        0x62, 0x14, 0x4c, 0x0a, 0xc1, 0xfc, 0x5a, 0x18,
        0x50, 0x6a, 0x22, 0x44, 0xba, 0x44, 0x9a, 0xc4,
    ];
    let fdu: [u8; 32] = [
        0xe6, 0xdb, 0x68, 0x67, 0x58, 0x30, 0x30, 0xdb,
        0x35, 0x94, 0xc1, 0xa4, 0x24, 0xb1, 0x5f, 0x7c,
        0x72, 0x66, 0x24, 0xec, 0x26, 0xb3, 0x35, 0x3b,
        0x10, 0xa9, 0x03, 0xa6, 0xd0, 0xab, 0x1c, 0x4c,
    ];
    
    
    let mut bwp = fdu;
    bwp[31] &= 0x7f;  
    let bzo = bzp(&bwp);
    let bdm = ahy(&bzo);
    if bdm == bwp {
        crate::serial_println!("[CRYPTO] FE round-trip: PASS");
    } else {
        crate::serial_println!("[CRYPTO] FE round-trip: FAIL");
        crate::serial_println!("[CRYPTO] Input:  {:02x?}", &bwp[..16]);
        crate::serial_println!("[CRYPTO] Output: {:02x?}", &bdm[..16]);
    }
    
    
    let nnd = EA_;
    let ngz = md(&nnd, &bzo);
    let ioz = ahy(&ngz);
    if ioz == bwp {
        crate::serial_println!("[CRYPTO] FE mul identity: PASS");
    } else {
        crate::serial_println!("[CRYPTO] FE mul identity: FAIL");
        crate::serial_println!("[CRYPTO] Expected: {:02x?}", &bwp[..16]);
        crate::serial_println!("[CRYPTO] Got:      {:02x?}", &ioz[..16]);
    }
    
    
    let qxm = py(&bzo);
    
    crate::serial_println!("[CRYPTO] FE sq: OK (no crash)");
    
    
    let jtw = ajr(&bzo, &IR_);
    let jtv = ahy(&jtw);
    if jtv == bwp {
        crate::serial_println!("[CRYPTO] FE add identity: PASS");
    } else {
        crate::serial_println!("[CRYPTO] FE add identity: FAIL");
    }
    
    let oyk = aql(&bzo, &IR_);
    let sub_bytes = ahy(&oyk);
    if sub_bytes == bwp {
        crate::serial_println!("[CRYPTO] FE sub identity: PASS");
    } else {
        crate::serial_println!("[CRYPTO] FE sub identity: FAIL");
        crate::serial_println!("[CRYPTO] Expected: {:02x?}", &bwp[..16]);
        crate::serial_println!("[CRYPTO] Got:      {:02x?}", &sub_bytes[..16]);
    }
    
    
    let qt = py(&bzo);
    let pwg = md(&qt, &qt);      
    let pwh = py(&qt);              
    let jse = ahy(&pwg);
    let jsf = ahy(&pwh);
    if jse == jsf {
        crate::serial_println!("[CRYPTO] FE mul vs sq: PASS");
    } else {
        crate::serial_println!("[CRYPTO] FE mul vs sq: FAIL (z^2*z^2 != sq(z^2))");
        crate::serial_println!("[CRYPTO] mul: {:02x?}", &jse[..16]);
        crate::serial_println!("[CRYPTO] sq:  {:02x?}", &jsf[..16]);
    }
    
    let evs = ahy(&EA_);
    
    
    let csc: Ab = [2, 0, 0, 0, 0];
    let lxx: Ab = [4, 0, 0, 0, 0];
    let poo = md(&csc, &csc);
    let eni = ahy(&lxx);
    let jos = ahy(&poo);
    if jos == eni {
        crate::serial_println!("[CRYPTO] FE 2*2=4: PASS");
    } else {
        crate::serial_println!("[CRYPTO] FE 2*2=4: FAIL");
        crate::serial_println!("[CRYPTO] Expected: {:02x?}", &eni[..16]);
        crate::serial_println!("[CRYPTO] Got:      {:02x?}", &jos[..16]);
    }
    
    
    let ovg = py(&csc);
    let jhi = ahy(&ovg);
    if jhi == eni {
        crate::serial_println!("[CRYPTO] FE sq(2)=4: PASS");
    } else {
        crate::serial_println!("[CRYPTO] FE sq(2)=4: FAIL");
        crate::serial_println!("[CRYPTO] Expected: {:02x?}", &eni[..16]);
        crate::serial_println!("[CRYPTO] Got:      {:02x?}", &jhi[..16]);
    }
    
    
    let pom = dpk(&csc);
    let pon = md(&csc, &pom);
    let jow = ahy(&pon);
    if jow == evs {
        crate::serial_println!("[CRYPTO] FE 2*2^-1: PASS");
    } else {
        crate::serial_println!("[CRYPTO] FE 2*2^-1: FAIL");
        crate::serial_println!("[CRYPTO] Expected: {:02x?}", &evs[..16]);
        crate::serial_println!("[CRYPTO] Got:      {:02x?}", &jow[..16]);
    }
    
    
    let ctd = dpk(&bzo);
    let ngy = md(&bzo, &ctd);
    let ihg = ahy(&ngy);
    if ihg == evs {
        crate::serial_println!("[CRYPTO] FE invert: PASS");
    } else {
        crate::serial_println!("[CRYPTO] FE invert: FAIL");
        crate::serial_println!("[CRYPTO] Expected: {:02x?}", &evs[..16]);
        crate::serial_println!("[CRYPTO] Got:      {:02x?}", &ihg[..16]);
    }
    
    let jrs: [u8; 32] = [
        0xc3, 0xda, 0x55, 0x37, 0x9d, 0xe9, 0xc6, 0x90,
        0x8e, 0x94, 0xea, 0x4d, 0xf2, 0x8d, 0x08, 0x4f,
        0x32, 0xec, 0xcf, 0x03, 0x49, 0x1c, 0x71, 0xf7,
        0x54, 0xb4, 0x07, 0x55, 0x77, 0xa2, 0x85, 0x52,
    ];
    
    let ffm = ffl(&aeh, &fdu);
    
    if ffm == jrs {
        crate::serial_println!("[CRYPTO] X25519: PASS");
    } else {
        crate::serial_println!("[CRYPTO] X25519: FAIL");
        crate::serial_println!("[CRYPTO] Expected: {:02x?}", &jrs[..16]);
        crate::serial_println!("[CRYPTO] Got:      {:02x?}", &ffm[..16]);
    }
    
    
    
    let fyb: [u8; 16] = [0x00; 16];
    let fyd: [u8; 12] = [0x00; 12];
    let fye: [u8; 16] = [0x00; 16];
    let iaw: [u8; 16] = [
        0x03, 0x88, 0xda, 0xce, 0x60, 0xb6, 0xa3, 0x92,
        0xf3, 0x28, 0xc2, 0xb9, 0x71, 0xb2, 0xfe, 0x78,
    ];
    let iax: [u8; 16] = [
        0xab, 0x6e, 0x47, 0xd4, 0x2c, 0xec, 0x13, 0xbd,
        0xf5, 0x3a, 0x67, 0xb2, 0x12, 0x57, 0xbd, 0xdf,
    ];
    
    let cjz = efc(&fyb, &fyd, &[], &fye);
    
    if &cjz[..16] == &iaw && &cjz[16..] == &iax {
        crate::serial_println!("[CRYPTO] AES-GCM: PASS");
    } else {
        crate::serial_println!("[CRYPTO] AES-GCM: FAIL");
        crate::serial_println!("[CRYPTO] Expected CT:  {:02x?}", &iaw);
        crate::serial_println!("[CRYPTO] Got CT:       {:02x?}", &cjz[..16]);
        crate::serial_println!("[CRYPTO] Expected TAG: {:02x?}", &iax);
        crate::serial_println!("[CRYPTO] Got TAG:      {:02x?}", &cjz[16..]);
    }
    
    crate::serial_println!("[CRYPTO] Self-tests complete");
}



pub fn prr() -> (usize, usize) {
    let mut passed = 0usize;
    let mut bv = 0usize;

    
    let ctk: [u8; 16] = [0x2b,0x7e,0x15,0x16,0x28,0xae,0xd2,0xa6,
                              0xab,0xf7,0x15,0x88,0x09,0xcf,0x4f,0x3c];
    let mut block: [u8; 16] = [0x32,0x43,0xf6,0xa8,0x88,0x5a,0x30,0x8d,
                                0x31,0x31,0x98,0xa2,0xe0,0x37,0x07,0x34];
    let lsp: [u8; 16] = [0x39,0x25,0x84,0x1d,0x02,0xdc,0x09,0xfb,
                                   0xdc,0x11,0x85,0x97,0x19,0x6a,0x0b,0x32];
    Aes128::new(&ctk).encrypt_block(&mut block);
    if block == lsp { passed += 1; } else { bv += 1; }

    
    let fao = asg(&[]);
    let oqv: [u8; 32] = [
        0xe3,0xb0,0xc4,0x42,0x98,0xfc,0x1c,0x14,
        0x9a,0xfb,0xf4,0xc8,0x99,0x6f,0xb9,0x24,
        0x27,0xae,0x41,0xe4,0x64,0x9b,0x93,0x4c,
        0xa4,0x95,0x99,0x1b,0x78,0x52,0xb8,0x55,
    ];
    if fao == oqv { passed += 1; } else { bv += 1; }

    
    let oqt = asg(b"abc");
    let oqu: [u8; 32] = [
        0xba,0x78,0x16,0xbf,0x8f,0x01,0xcf,0xea,
        0x41,0x41,0x40,0xde,0x5d,0xae,0x22,0x23,
        0xb0,0x03,0x61,0xa3,0x96,0x17,0x7a,0x9c,
        0xb4,0x10,0xff,0x61,0xf2,0x00,0x15,0xad,
    ];
    if oqt == oqu { passed += 1; } else { bv += 1; }

    
    let aeh: [u8; 32] = [
        0xa5,0x46,0xe3,0x6b,0xf0,0x52,0x7c,0x9d,
        0x3b,0x16,0x15,0x4b,0x82,0x46,0x5e,0xdd,
        0x62,0x14,0x4c,0x0a,0xc1,0xfc,0x5a,0x18,
        0x50,0x6a,0x22,0x44,0xba,0x44,0x9a,0xc4,
    ];
    let fdu: [u8; 32] = [
        0xe6,0xdb,0x68,0x67,0x58,0x30,0x30,0xdb,
        0x35,0x94,0xc1,0xa4,0x24,0xb1,0x5f,0x7c,
        0x72,0x66,0x24,0xec,0x26,0xb3,0x35,0x3b,
        0x10,0xa9,0x03,0xa6,0xd0,0xab,0x1c,0x4c,
    ];
    let pvo: [u8; 32] = [
        0xc3,0xda,0x55,0x37,0x9d,0xe9,0xc6,0x90,
        0x8e,0x94,0xea,0x4d,0xf2,0x8d,0x08,0x4f,
        0x32,0xec,0xcf,0x03,0x49,0x1c,0x71,0xf7,
        0x54,0xb4,0x07,0x55,0x77,0xa2,0x85,0x52,
    ];
    let ffm = ffl(&aeh, &fdu);
    if ffm == pvo { passed += 1; } else { bv += 1; }

    
    let fyb: [u8; 16] = [0x00; 16];
    let fyd: [u8; 12] = [0x00; 12];
    let fye: [u8; 16] = [0x00; 16];
    let mbi: [u8; 16] = [
        0x03,0x88,0xda,0xce,0x60,0xb6,0xa3,0x92,
        0xf3,0x28,0xc2,0xb9,0x71,0xb2,0xfe,0x78,
    ];
    let mbj: [u8; 16] = [
        0xab,0x6e,0x47,0xd4,0x2c,0xec,0x13,0xbd,
        0xf5,0x3a,0x67,0xb2,0x12,0x57,0xbd,0xdf,
    ];
    let cjz = efc(&fyb, &fyd, &[], &fye);
    if &cjz[..16] == &mbi && &cjz[16..] == &mbj {
        passed += 1;
    } else {
        bv += 1;
    }

    
    let mly = b"Jefe";
    let mlw = b"what do ya want for nothing?";
    let mlz = bmu(mly, mlw);
    let mlx: [u8; 32] = [
        0x5b,0xdc,0xc1,0x46,0xbf,0x60,0x75,0x4e,
        0x6a,0x04,0x24,0x26,0x08,0x95,0x75,0xc7,
        0x5a,0x00,0x3f,0x08,0x9d,0x27,0x39,0x83,
        0x9d,0xec,0x58,0xb9,0x64,0xec,0x38,0x43,
    ];
    if mlz == mlx { passed += 1; } else { bv += 1; }

    (passed, bv)
}
