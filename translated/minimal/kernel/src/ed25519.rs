










use alloc::vec::Vec;
use crate::tls13::crypto::{
    Ab, IR_, EA_,
    bzp, ahy, fwh,
    ajr, aql, md, py,
    dpk, gnt,
};





const BHR_: [u64; 80] = [
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

pub fn jfz(data: &[u8]) -> [u8; 64] {
    let mut h: [u64; 8] = [
        0x6a09e667f3bcc908, 0xbb67ae8584caa73b,
        0x3c6ef372fe94f82b, 0xa54ff53a5f1d36f1,
        0x510e527fade682d1, 0x9b05688c2b3e6c1f,
        0x1f83d9abfb41bd6b, 0x5be0cd19137e2179,
    ];

    let djk = (data.len() as u128) * 8;

    
    let mut bk = Vec::from(data);
    bk.push(0x80);
    while bk.len() % 128 != 112 {
        bk.push(0);
    }
    
    for i in (0..16).rev() {
        bk.push((djk >> (i * 8)) as u8);
    }

    
    for block in bk.chunks_exact(128) {
        let mut w = [0u64; 80];
        for i in 0..16 {
            w[i] = u64::from_be_bytes([
                block[i*8], block[i*8+1], block[i*8+2], block[i*8+3],
                block[i*8+4], block[i*8+5], block[i*8+6], block[i*8+7],
            ]);
        }
        for i in 16..80 {
            let auz = w[i-15].rotate_right(1) ^ w[i-15].rotate_right(8) ^ (w[i-15] >> 7);
            let afq = w[i-2].rotate_right(19) ^ w[i-2].rotate_right(61) ^ (w[i-2] >> 6);
            w[i] = w[i-16].wrapping_add(auz).wrapping_add(w[i-7]).wrapping_add(afq);
        }

        let (mut a, mut b, mut c, mut d, mut e, mut f, mut g, mut agm) =
            (h[0], h[1], h[2], h[3], h[4], h[5], h[6], h[7]);

        for i in 0..80 {
            let afq = e.rotate_right(14) ^ e.rotate_right(18) ^ e.rotate_right(41);
            let ch = (e & f) ^ ((!e) & g);
            let cee = agm.wrapping_add(afq).wrapping_add(ch).wrapping_add(BHR_[i]).wrapping_add(w[i]);
            let auz = a.rotate_right(28) ^ a.rotate_right(34) ^ a.rotate_right(39);
            let dtx = (a & b) ^ (a & c) ^ (b & c);
            let ebj = auz.wrapping_add(dtx);

            agm = g; g = f; f = e;
            e = d.wrapping_add(cee);
            d = c; c = b; b = a;
            a = cee.wrapping_add(ebj);
        }

        h[0] = h[0].wrapping_add(a); h[1] = h[1].wrapping_add(b);
        h[2] = h[2].wrapping_add(c); h[3] = h[3].wrapping_add(d);
        h[4] = h[4].wrapping_add(e); h[5] = h[5].wrapping_add(f);
        h[6] = h[6].wrapping_add(g); h[7] = h[7].wrapping_add(agm);
    }

    let mut out = [0u8; 64];
    for i in 0..8 {
        out[i*8..i*8+8].copy_from_slice(&h[i].to_be_bytes());
    }
    out
}


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
            let od = data.len().min(space);
            self.buffer[self.buffer_len..self.buffer_len + od]
                .copy_from_slice(&data[..od]);
            self.buffer_len += od;
            offset = od;

            if self.buffer_len == 128 {
                let block = self.buffer;
                fan(&mut self.state, &block);
                self.buffer_len = 0;
            }
        }

        while offset + 128 <= data.len() {
            let mut block = [0u8; 128];
            block.copy_from_slice(&data[offset..offset + 128]);
            fan(&mut self.state, &block);
            offset += 128;
        }

        if offset < data.len() {
            let ck = data.len() - offset;
            self.buffer[..ck].copy_from_slice(&data[offset..]);
            self.buffer_len = ck;
        }
    }

    pub fn finalize(&mut self) -> [u8; 64] {
        let djk = self.total_len * 8;

        
        self.buffer[self.buffer_len] = 0x80;
        self.buffer_len += 1;

        if self.buffer_len > 112 {
            
            for i in self.buffer_len..128 {
                self.buffer[i] = 0;
            }
            let block = self.buffer;
            fan(&mut self.state, &block);
            self.buffer_len = 0;
        }

        for i in self.buffer_len..112 {
            self.buffer[i] = 0;
        }

        for i in (0..16).rev() {
            self.buffer[112 + (15 - i)] = (djk >> (i * 8)) as u8;
        }

        let block = self.buffer;
        fan(&mut self.state, &block);

        let mut out = [0u8; 64];
        for i in 0..8 {
            out[i*8..i*8+8].copy_from_slice(&self.state[i].to_be_bytes());
        }
        out
    }
}

fn fan(state: &mut [u64; 8], block: &[u8; 128]) {
    let mut w = [0u64; 80];
    for i in 0..16 {
        w[i] = u64::from_be_bytes([
            block[i*8], block[i*8+1], block[i*8+2], block[i*8+3],
            block[i*8+4], block[i*8+5], block[i*8+6], block[i*8+7],
        ]);
    }
    for i in 16..80 {
        let auz = w[i-15].rotate_right(1) ^ w[i-15].rotate_right(8) ^ (w[i-15] >> 7);
        let afq = w[i-2].rotate_right(19) ^ w[i-2].rotate_right(61) ^ (w[i-2] >> 6);
        w[i] = w[i-16].wrapping_add(auz).wrapping_add(w[i-7]).wrapping_add(afq);
    }

    let (mut a, mut b, mut c, mut d, mut e, mut f, mut g, mut agm) =
        (state[0], state[1], state[2], state[3], state[4], state[5], state[6], state[7]);

    for i in 0..80 {
        let afq = e.rotate_right(14) ^ e.rotate_right(18) ^ e.rotate_right(41);
        let ch = (e & f) ^ ((!e) & g);
        let cee = agm.wrapping_add(afq).wrapping_add(ch).wrapping_add(BHR_[i]).wrapping_add(w[i]);
        let auz = a.rotate_right(28) ^ a.rotate_right(34) ^ a.rotate_right(39);
        let dtx = (a & b) ^ (a & c) ^ (b & c);
        let ebj = auz.wrapping_add(dtx);

        agm = g; g = f; f = e;
        e = d.wrapping_add(cee);
        d = c; c = b; b = a;
        a = cee.wrapping_add(ebj);
    }

    state[0] = state[0].wrapping_add(a); state[1] = state[1].wrapping_add(b);
    state[2] = state[2].wrapping_add(c); state[3] = state[3].wrapping_add(d);
    state[4] = state[4].wrapping_add(e); state[5] = state[5].wrapping_add(f);
    state[6] = state[6].wrapping_add(g); state[7] = state[7].wrapping_add(agm);
}






fn lur(a: &Ab) -> Ab {
    
    aql(&IR_, a)
}



fn hux() -> Ab {
    bzp(&[
        0xa3, 0x78, 0x59, 0x13, 0xca, 0x4d, 0xeb, 0x75,
        0xab, 0xd8, 0x41, 0x41, 0x4d, 0x0a, 0x70, 0x00,
        0x98, 0xe8, 0x79, 0x77, 0x79, 0x40, 0xc7, 0x8c,
        0x73, 0xfe, 0x6f, 0x2b, 0xee, 0x6c, 0x03, 0x52,
    ])
}


fn lnt() -> Ab {
    let d = hux();
    ajr(&d, &d)
}


fn lus() -> Ab {
    bzp(&[
        0xb0, 0xa0, 0x0e, 0x4a, 0x27, 0x1b, 0xee, 0xc4,
        0x78, 0xe4, 0x2f, 0xad, 0x06, 0x18, 0x43, 0x2f,
        0xa7, 0xd7, 0xfb, 0x3d, 0x99, 0x00, 0x4d, 0x2b,
        0x0b, 0xdf, 0xc1, 0x4f, 0x80, 0x24, 0x83, 0x2b,
    ])
}





#[derive(Clone)]
pub struct ExtPoint {
    x: Ab,
    y: Ab,
    z: Ab,
    t: Ab,
}

impl ExtPoint {
    
    pub fn identity() -> Self {
        Self {
            x: IR_,
            y: EA_,
            z: EA_,
            t: IR_,
        }
    }

    
    pub fn fij() -> Self {
        
        let y = bzp(&[
            0x58, 0x66, 0x66, 0x66, 0x66, 0x66, 0x66, 0x66,
            0x66, 0x66, 0x66, 0x66, 0x66, 0x66, 0x66, 0x66,
            0x66, 0x66, 0x66, 0x66, 0x66, 0x66, 0x66, 0x66,
            0x66, 0x66, 0x66, 0x66, 0x66, 0x66, 0x66, 0x66,
        ]);
        
        let x = bzp(&[
            0x1a, 0xd5, 0x25, 0x8f, 0x60, 0x2d, 0x56, 0xc9,
            0xb2, 0xa7, 0x25, 0x95, 0x60, 0xc7, 0x2c, 0x69,
            0x5c, 0xdc, 0xd6, 0xfd, 0x31, 0xe2, 0xa4, 0xc0,
            0xfe, 0x53, 0x6e, 0xcd, 0xd3, 0x36, 0x69, 0x21,
        ]);
        Self {
            t: md(&x, &y),
            x,
            y,
            z: EA_,
        }
    }

    
    pub fn add(&self, other: &Self) -> Self {
        let jq = lnt();
        let a = md(&aql(&self.y, &self.x), &aql(&other.y, &other.x));
        let b = md(&ajr(&self.y, &self.x), &ajr(&other.y, &other.x));
        let c = md(&md(&self.t, &other.t), &jq);
        let d = ajr(&self.z, &self.z);
        let d = md(&d, &other.z);
        let e = aql(&b, &a);
        let f = aql(&d, &c);
        let g = ajr(&d, &c);
        let h = ajr(&b, &a);
        Self {
            x: md(&e, &f),
            y: md(&g, &h),
            t: md(&e, &h),
            z: md(&f, &g),
        }
    }

    
    pub fn double(&self) -> Self {
        let a = py(&self.x);
        let b = py(&self.y);
        let c = ajr(&py(&self.z), &py(&self.z));
        let h = ajr(&a, &b);
        let pvy = ajr(&self.x, &self.y);
        let e = aql(&h, &py(&pvy));
        let g = aql(&a, &b);
        let f = ajr(&c, &g);
        Self {
            x: md(&e, &f),
            y: md(&g, &h),
            t: md(&e, &h),
            z: md(&f, &g),
        }
    }

    
    pub fn scalar_mul(&self, aeh: &[u8; 32]) -> Self {
        let mut result = ExtPoint::identity();
        let mut ts = self.clone();

        for i in 0..256 {
            let yk = i / 8;
            let bew = i % 8;
            if (aeh[yk] >> bew) & 1 == 1 {
                result = result.add(&ts);
            }
            ts = ts.double();
        }
        result
    }

    
    pub fn compress(&self) -> [u8; 32] {
        let ctd = dpk(&self.z);
        let x = md(&self.x, &ctd);
        let y = md(&self.y, &ctd);
        let mut j = ahy(&y);
        
        let hcv = ahy(&x);
        j[31] |= (hcv[0] & 1) << 7;
        j
    }

    
    pub fn hrc(j: &[u8; 32]) -> Option<Self> {
        
        let pvr = (j[31] >> 7) & 1;
        let mut jsc = *j;
        jsc[31] &= 0x7f;

        let y = bzp(&jsc);
        let d = hux();

        
        let y2 = py(&y);
        let iy = aql(&y2, &EA_);          
        let v = ajr(&md(&d, &y2), &EA_); 

        
        
        let v2 = py(&v);
        let v3 = md(&v2, &v);
        let v4 = py(&v2);
        let v7 = md(&v4, &v3);
        let hba = md(&iy, &v7);

        
        
        let (t250_1, _) = gnt(&hba);
        let pcf = py(&t250_1);
        let jlb = py(&pcf);       
        let bqo = md(&jlb, &hba); 

        
        
        let nwk = md(&jlb, &hba);

        let mut x = md(&md(&iy, &v3), &nwk);

        
        let fes = md(&v, &py(&x));
        let cgv = aql(&fes, &iy);
        let kit = ahy(&cgv);
        let kjf = kit.iter().all(|&b| b == 0);

        if !kjf {
            
            let nib = ajr(&fes, &iy);
            let nia = ahy(&nib);
            let nie = nia.iter().all(|&b| b == 0);
            if !nie {
                return None; 
            }
            x = md(&x, &lus());
        }

        
        let hcv = ahy(&x);
        if (hcv[0] & 1) != pvr {
            x = lur(&x);
        }

        let t = md(&x, &y);
        Some(Self { x, y, z: EA_, t })
    }
}








const OY_: [u8; 32] = [
    0xed, 0xd3, 0xf5, 0x5c, 0x1a, 0x63, 0x12, 0x58,
    0xd6, 0x9c, 0xf7, 0xa2, 0xde, 0xf9, 0xde, 0x14,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x10,
];



fn iob(x: &mut [i64; 64], r: &mut [u8; 32]) {
    for i in (32usize..=63).rev() {
        let mut ahn: i64 = 0;
        let start = if i >= 32 { i - 32 } else { 0 };
        let end = i - 12;
        for ay in start..end {
            x[ay] += ahn - 16 * x[i] * OY_[ay - start] as i64;
            ahn = (x[ay] + 128) >> 8;
            x[ay] -= ahn << 8;
        }
        x[end] += ahn;
        x[i] = 0;
    }

    let mut ahn: i64 = 0;
    for ay in 0..32 {
        x[ay] += ahn - (x[31] >> 4) * OY_[ay] as i64;
        ahn = x[ay] >> 8;
        x[ay] &= 255;
    }
    for ay in 0..32 {
        x[ay] -= ahn * OY_[ay] as i64;
    }
    for i in 0..32 {
        x[i + 1] += x[i] >> 8;
        r[i] = (x[i] & 255) as u8;
    }
}


fn gso(hash: &[u8; 64]) -> [u8; 32] {
    let mut x = [0i64; 64];
    for i in 0..64 {
        x[i] = hash[i] as i64;
    }
    let mut r = [0u8; 32];
    iob(&mut x, &mut r);
    r
}


fn okp(a: &[u8; 32], b: &[u8; 32], c: &[u8; 32]) -> [u8; 32] {
    let mut x = [0i64; 64];
    
    for i in 0..32 {
        x[i] = c[i] as i64;
    }
    
    for i in 0..32 {
        for ay in 0..32 {
            x[i + ay] += a[i] as i64 * b[ay] as i64;
        }
    }
    let mut r = [0u8; 32];
    iob(&mut x, &mut r);
    r
}








pub struct Yh {
    pub con: [u8; 32],
    pub secret_key: [u8; 32],   
}


pub fn ftz(seed: &[u8; 32]) -> [u8; 32] {
    let h = jfz(seed);
    let mut a = [0u8; 32];
    a.copy_from_slice(&h[..32]);

    
    a[0] &= 248;
    a[31] &= 127;
    a[31] |= 64;

    let bp = ExtPoint::fij();
    let gnn = bp.scalar_mul(&a);
    gnn.compress()
}


pub fn qej(seed: &[u8; 32]) -> Yh {
    let con = ftz(seed);
    Yh {
        con,
        secret_key: *seed,
    }
}










pub fn huz(message: &[u8], seed: &[u8; 32], con: &[u8; 32]) -> [u8; 64] {
    let h = jfz(seed);

    
    let mut a = [0u8; 32];
    a.copy_from_slice(&h[..32]);
    a[0] &= 248;
    a[31] &= 127;
    a[31] |= 64;

    
    
    let mut bgv = Sha512::new();
    bgv.update(&h[32..64]);
    bgv.update(message);
    let nkt = bgv.finalize();
    let r = gso(&nkt);

    
    let bp = ExtPoint::fij();
    let gnp = bp.scalar_mul(&r);
    let ixm = gnp.compress();

    
    let mut eoy = Sha512::new();
    eoy.update(&ixm);
    eoy.update(con);
    eoy.update(message);
    let mhb = eoy.finalize();
    let mha = gso(&mhb);

    
    let j = okp(&mha, &a, &r);

    let mut sig = [0u8; 64];
    sig[..32].copy_from_slice(&ixm);
    sig[32..].copy_from_slice(&j);
    sig
}










pub fn hva(message: &[u8], signature: &[u8; 64], con: &[u8; 32]) -> bool {
    
    let mut gpj = [0u8; 32];
    gpj.copy_from_slice(&signature[..32]);
    let mut gsg = [0u8; 32];
    gsg.copy_from_slice(&signature[32..]);

    
    if !okq(&gsg) {
        return false;
    }

    
    let gnn = match ExtPoint::hrc(con) {
        Some(aa) => aa,
        None => return false,
    };

    
    let gnp = match ExtPoint::hrc(&gpj) {
        Some(aa) => aa,
        None => return false,
    };

    
    let mut bgv = Sha512::new();
    bgv.update(&gpj);
    bgv.update(con);
    bgv.update(message);
    let mgz = bgv.finalize();
    let h = gso(&mgz);

    
    let bp = ExtPoint::fij();
    let cv = bp.scalar_mul(&gsg);

    
    let mhd = gnn.scalar_mul(&h);
    let amp = gnp.add(&mhd);

    
    
    let myf = cv.compress();
    let ogy = amp.compress();

    
    let mut jr = 0u8;
    for i in 0..32 {
        jr |= myf[i] ^ ogy[i];
    }
    jr == 0
}


fn okq(j: &[u8; 32]) -> bool {
    
    for i in (0..32).rev() {
        if j[i] < OY_[i] {
            return true;
        }
        if j[i] > OY_[i] {
            return false;
        }
    }
    false 
}





pub fn fkk(bytes: &[u8]) -> alloc::string::String {
    let mut j = alloc::string::String::with_capacity(bytes.len() * 2);
    for &b in bytes {
        use core::fmt::Write;
        let _ = write!(j, "{:02x}", b);
    }
    j
}

pub fn qkw(ga: &str) -> Option<[u8; 32]> {
    if ga.len() != 64 {
        return None;
    }
    let mut out = [0u8; 32];
    for i in 0..32 {
        let kgp = &ga[i*2..i*2+2];
        out[i] = u8::from_str_radix(kgp, 16).ok()?;
    }
    Some(out)
}
