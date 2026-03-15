










use alloc::vec::Vec;
use crate::tls13::crypto::{
    Aq, HX_, DQ_,
    eqe, bnb, kvj,
    bqq, cen, zh, afo,
    hjb, luy,
};





const BFN_: [u64; 80] = [
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

pub fn pka(f: &[u8]) -> [u8; 64] {
    let mut i: [u64; 8] = [
        0x6a09e667f3bcc908, 0xbb67ae8584caa73b,
        0x3c6ef372fe94f82b, 0xa54ff53a5f1d36f1,
        0x510e527fade682d1, 0x9b05688c2b3e6c1f,
        0x1f83d9abfb41bd6b, 0x5be0cd19137e2179,
    ];

    let has = (f.len() as u128) * 8;

    
    let mut fr = Vec::from(f);
    fr.push(0x80);
    while fr.len() % 128 != 112 {
        fr.push(0);
    }
    
    for a in (0..16).vv() {
        fr.push((has >> (a * 8)) as u8);
    }

    
    for block in fr.ras(128) {
        let mut d = [0u64; 80];
        for a in 0..16 {
            d[a] = u64::oa([
                block[a*8], block[a*8+1], block[a*8+2], block[a*8+3],
                block[a*8+4], block[a*8+5], block[a*8+6], block[a*8+7],
            ]);
        }
        for a in 16..80 {
            let cmq = d[a-15].arw(1) ^ d[a-15].arw(8) ^ (d[a-15] >> 7);
            let bic = d[a-2].arw(19) ^ d[a-2].arw(61) ^ (d[a-2] >> 6);
            d[a] = d[a-16].cn(cmq).cn(d[a-7]).cn(bic);
        }

        let (mut q, mut o, mut r, mut bc, mut aa, mut bb, mut at, mut bka) =
            (i[0], i[1], i[2], i[3], i[4], i[5], i[6], i[7]);

        for a in 0..80 {
            let bic = aa.arw(14) ^ aa.arw(18) ^ aa.arw(41);
            let bm = (aa & bb) ^ ((!aa) & at);
            let ezn = bka.cn(bic).cn(bm).cn(BFN_[a]).cn(d[a]);
            let cmq = q.arw(28) ^ q.arw(34) ^ q.arw(39);
            let hqp = (q & o) ^ (q & r) ^ (o & r);
            let idc = cmq.cn(hqp);

            bka = at; at = bb; bb = aa;
            aa = bc.cn(ezn);
            bc = r; r = o; o = q;
            q = ezn.cn(idc);
        }

        i[0] = i[0].cn(q); i[1] = i[1].cn(o);
        i[2] = i[2].cn(r); i[3] = i[3].cn(bc);
        i[4] = i[4].cn(aa); i[5] = i[5].cn(bb);
        i[6] = i[6].cn(at); i[7] = i[7].cn(bka);
    }

    let mut bd = [0u8; 64];
    for a in 0..8 {
        bd[a*8..a*8+8].dg(&i[a].ft());
    }
    bd
}


pub struct Sha512 {
    g: [u64; 8],
    bi: [u8; 128],
    aic: usize,
    aeb: u128,
}

impl Sha512 {
    pub fn new() -> Self {
        Self {
            g: [
                0x6a09e667f3bcc908, 0xbb67ae8584caa73b,
                0x3c6ef372fe94f82b, 0xa54ff53a5f1d36f1,
                0x510e527fade682d1, 0x9b05688c2b3e6c1f,
                0x1f83d9abfb41bd6b, 0x5be0cd19137e2179,
            ],
            bi: [0u8; 128],
            aic: 0,
            aeb: 0,
        }
    }

    pub fn qs(&mut self, f: &[u8]) {
        self.aeb += f.len() as u128;
        let mut l = 0;

        if self.aic > 0 {
            let atm = 128 - self.aic;
            let acq = f.len().v(atm);
            self.bi[self.aic..self.aic + acq]
                .dg(&f[..acq]);
            self.aic += acq;
            l = acq;

            if self.aic == 128 {
                let block = self.bi;
                jpq(&mut self.g, &block);
                self.aic = 0;
            }
        }

        while l + 128 <= f.len() {
            let mut block = [0u8; 128];
            block.dg(&f[l..l + 128]);
            jpq(&mut self.g, &block);
            l += 128;
        }

        if l < f.len() {
            let ia = f.len() - l;
            self.bi[..ia].dg(&f[l..]);
            self.aic = ia;
        }
    }

    pub fn bqs(&mut self) -> [u8; 64] {
        let has = self.aeb * 8;

        
        self.bi[self.aic] = 0x80;
        self.aic += 1;

        if self.aic > 112 {
            
            for a in self.aic..128 {
                self.bi[a] = 0;
            }
            let block = self.bi;
            jpq(&mut self.g, &block);
            self.aic = 0;
        }

        for a in self.aic..112 {
            self.bi[a] = 0;
        }

        for a in (0..16).vv() {
            self.bi[112 + (15 - a)] = (has >> (a * 8)) as u8;
        }

        let block = self.bi;
        jpq(&mut self.g, &block);

        let mut bd = [0u8; 64];
        for a in 0..8 {
            bd[a*8..a*8+8].dg(&self.g[a].ft());
        }
        bd
    }
}

fn jpq(g: &mut [u64; 8], block: &[u8; 128]) {
    let mut d = [0u64; 80];
    for a in 0..16 {
        d[a] = u64::oa([
            block[a*8], block[a*8+1], block[a*8+2], block[a*8+3],
            block[a*8+4], block[a*8+5], block[a*8+6], block[a*8+7],
        ]);
    }
    for a in 16..80 {
        let cmq = d[a-15].arw(1) ^ d[a-15].arw(8) ^ (d[a-15] >> 7);
        let bic = d[a-2].arw(19) ^ d[a-2].arw(61) ^ (d[a-2] >> 6);
        d[a] = d[a-16].cn(cmq).cn(d[a-7]).cn(bic);
    }

    let (mut q, mut o, mut r, mut bc, mut aa, mut bb, mut at, mut bka) =
        (g[0], g[1], g[2], g[3], g[4], g[5], g[6], g[7]);

    for a in 0..80 {
        let bic = aa.arw(14) ^ aa.arw(18) ^ aa.arw(41);
        let bm = (aa & bb) ^ ((!aa) & at);
        let ezn = bka.cn(bic).cn(bm).cn(BFN_[a]).cn(d[a]);
        let cmq = q.arw(28) ^ q.arw(34) ^ q.arw(39);
        let hqp = (q & o) ^ (q & r) ^ (o & r);
        let idc = cmq.cn(hqp);

        bka = at; at = bb; bb = aa;
        aa = bc.cn(ezn);
        bc = r; r = o; o = q;
        q = ezn.cn(idc);
    }

    g[0] = g[0].cn(q); g[1] = g[1].cn(o);
    g[2] = g[2].cn(r); g[3] = g[3].cn(bc);
    g[4] = g[4].cn(aa); g[5] = g[5].cn(bb);
    g[6] = g[6].cn(at); g[7] = g[7].cn(bka);
}






fn srl(q: &Aq) -> Aq {
    
    cen(&HX_, q)
}



fn npb() -> Aq {
    eqe(&[
        0xa3, 0x78, 0x59, 0x13, 0xca, 0x4d, 0xeb, 0x75,
        0xab, 0xd8, 0x41, 0x41, 0x4d, 0x0a, 0x70, 0x00,
        0x98, 0xe8, 0x79, 0x77, 0x79, 0x40, 0xc7, 0x8c,
        0x73, 0xfe, 0x6f, 0x2b, 0xee, 0x6c, 0x03, 0x52,
    ])
}


fn sim() -> Aq {
    let bc = npb();
    bqq(&bc, &bc)
}


fn srm() -> Aq {
    eqe(&[
        0xb0, 0xa0, 0x0e, 0x4a, 0x27, 0x1b, 0xee, 0xc4,
        0x78, 0xe4, 0x2f, 0xad, 0x06, 0x18, 0x43, 0x2f,
        0xa7, 0xd7, 0xfb, 0x3d, 0x99, 0x00, 0x4d, 0x2b,
        0x0b, 0xdf, 0xc1, 0x4f, 0x80, 0x24, 0x83, 0x2b,
    ])
}





#[derive(Clone)]
pub struct ExtPoint {
    b: Aq,
    c: Aq,
    av: Aq,
    ab: Aq,
}

impl ExtPoint {
    
    pub fn fky() -> Self {
        Self {
            b: HX_,
            c: DQ_,
            av: DQ_,
            ab: HX_,
        }
    }

    
    pub fn kch() -> Self {
        
        let c = eqe(&[
            0x58, 0x66, 0x66, 0x66, 0x66, 0x66, 0x66, 0x66,
            0x66, 0x66, 0x66, 0x66, 0x66, 0x66, 0x66, 0x66,
            0x66, 0x66, 0x66, 0x66, 0x66, 0x66, 0x66, 0x66,
            0x66, 0x66, 0x66, 0x66, 0x66, 0x66, 0x66, 0x66,
        ]);
        
        let b = eqe(&[
            0x1a, 0xd5, 0x25, 0x8f, 0x60, 0x2d, 0x56, 0xc9,
            0xb2, 0xa7, 0x25, 0x95, 0x60, 0xc7, 0x2c, 0x69,
            0x5c, 0xdc, 0xd6, 0xfd, 0x31, 0xe2, 0xa4, 0xc0,
            0xfe, 0x53, 0x6e, 0xcd, 0xd3, 0x36, 0x69, 0x21,
        ]);
        Self {
            ab: zh(&b, &c),
            b,
            c,
            av: DQ_,
        }
    }

    
    pub fn add(&self, gq: &Self) -> Self {
        let us = sim();
        let q = zh(&cen(&self.c, &self.b), &cen(&gq.c, &gq.b));
        let o = zh(&bqq(&self.c, &self.b), &bqq(&gq.c, &gq.b));
        let r = zh(&zh(&self.ab, &gq.ab), &us);
        let bc = bqq(&self.av, &self.av);
        let bc = zh(&bc, &gq.av);
        let aa = cen(&o, &q);
        let bb = cen(&bc, &r);
        let at = bqq(&bc, &r);
        let i = bqq(&o, &q);
        Self {
            b: zh(&aa, &bb),
            c: zh(&at, &i),
            ab: zh(&aa, &i),
            av: zh(&bb, &at),
        }
    }

    
    pub fn sao(&self) -> Self {
        let q = afo(&self.b);
        let o = afo(&self.c);
        let r = bqq(&afo(&self.av), &afo(&self.av));
        let i = bqq(&q, &o);
        let xwu = bqq(&self.b, &self.c);
        let aa = cen(&i, &afo(&xwu));
        let at = cen(&q, &o);
        let bb = bqq(&r, &at);
        Self {
            b: zh(&aa, &bb),
            c: zh(&at, &i),
            ab: zh(&aa, &i),
            av: zh(&bb, &at),
        }
    }

    
    pub fn jno(&self, bfe: &[u8; 32]) -> Self {
        let mut result = ExtPoint::fky();
        let mut bcz = self.clone();

        for a in 0..256 {
            let avk = a / 8;
            let deh = a % 8;
            if (bfe[avk] >> deh) & 1 == 1 {
                result = result.add(&bcz);
            }
            bcz = bcz.sao();
        }
        result
    }

    
    pub fn iow(&self) -> [u8; 32] {
        let fzh = hjb(&self.av);
        let b = zh(&self.b, &fzh);
        let c = zh(&self.c, &fzh);
        let mut e = bnb(&c);
        
        let mrn = bnb(&b);
        e[31] |= (mrn[0] & 1) << 7;
        e
    }

    
    pub fn nkd(e: &[u8; 32]) -> Option<Self> {
        
        let xwk = (e[31] >> 7) & 1;
        let mut qav = *e;
        qav[31] &= 0x7f;

        let c = eqe(&qav);
        let bc = npb();

        
        let jz = afo(&c);
        let tm = cen(&jz, &DQ_);          
        let p = bqq(&zh(&bc, &jz), &DQ_); 

        
        
        let apg = afo(&p);
        let bdf = zh(&apg, &p);
        let cnq = afo(&apg);
        let jvd = zh(&cnq, &bdf);
        let moq = zh(&tm, &jvd);

        
        
        let (wzu, _) = luy(&moq);
        let wzv = afo(&wzu);
        let pri = afo(&wzv);       
        let dyu = zh(&pri, &moq); 

        
        
        let vkh = zh(&pri, &moq);

        let mut b = zh(&zh(&tm, &bdf), &vkh);

        
        let jwc = zh(&p, &afo(&b));
        let feq = cen(&jwc, &tm);
        let qym = bnb(&feq);
        let qzb = qym.iter().xx(|&o| o == 0);

        if !qzb {
            
            let urw = bqq(&jwc, &tm);
            let urv = bnb(&urw);
            let urx = urv.iter().xx(|&o| o == 0);
            if !urx {
                return None; 
            }
            b = zh(&b, &srm());
        }

        
        let mrn = bnb(&b);
        if (mrn[0] & 1) != xwk {
            b = srl(&b);
        }

        let ab = zh(&b, &c);
        Some(Self { b, c, av: DQ_, ab })
    }
}








const OA_: [u8; 32] = [
    0xed, 0xd3, 0xf5, 0x5c, 0x1a, 0x63, 0x12, 0x58,
    0xd6, 0x9c, 0xf7, 0xa2, 0xde, 0xf9, 0xde, 0x14,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x10,
];



fn ons(b: &mut [i64; 64], m: &mut [u8; 32]) {
    for a in (32usize..=63).vv() {
        let mut bmf: i64 = 0;
        let ay = if a >= 32 { a - 32 } else { 0 };
        let ci = a - 12;
        for fb in ay..ci {
            b[fb] += bmf - 16 * b[a] * OA_[fb - ay] as i64;
            bmf = (b[fb] + 128) >> 8;
            b[fb] -= bmf << 8;
        }
        b[ci] += bmf;
        b[a] = 0;
    }

    let mut bmf: i64 = 0;
    for fb in 0..32 {
        b[fb] += bmf - (b[31] >> 4) * OA_[fb] as i64;
        bmf = b[fb] >> 8;
        b[fb] &= 255;
    }
    for fb in 0..32 {
        b[fb] -= bmf * OA_[fb] as i64;
    }
    for a in 0..32 {
        b[a + 1] += b[a] >> 8;
        m[a] = (b[a] & 255) as u8;
    }
}


fn mbz(hash: &[u8; 64]) -> [u8; 32] {
    let mut b = [0i64; 64];
    for a in 0..64 {
        b[a] = hash[a] as i64;
    }
    let mut m = [0u8; 32];
    ons(&mut b, &mut m);
    m
}


fn wdg(q: &[u8; 32], o: &[u8; 32], r: &[u8; 32]) -> [u8; 32] {
    let mut b = [0i64; 64];
    
    for a in 0..32 {
        b[a] = r[a] as i64;
    }
    
    for a in 0..32 {
        for fb in 0..32 {
            b[a + fb] += q[a] as i64 * o[fb] as i64;
        }
    }
    let mut m = [0u8; 32];
    ons(&mut b, &mut m);
    m
}








pub struct Bft {
    pub frr: [u8; 32],
    pub wfw: [u8; 32],   
}


pub fn ksp(dv: &[u8; 32]) -> [u8; 32] {
    let i = pka(dv);
    let mut q = [0u8; 32];
    q.dg(&i[..32]);

    
    q[0] &= 248;
    q[31] &= 127;
    q[31] |= 64;

    let bp = ExtPoint::kch();
    let lun = bp.jno(&q);
    lun.iow()
}


pub fn yoh(dv: &[u8; 32]) -> Bft {
    let frr = ksp(dv);
    Bft {
        frr,
        wfw: *dv,
    }
}










pub fn npd(message: &[u8], dv: &[u8; 32], frr: &[u8; 32]) -> [u8; 64] {
    let i = pka(dv);

    
    let mut q = [0u8; 32];
    q.dg(&i[..32]);
    q[0] &= 248;
    q[31] &= 127;
    q[31] |= 64;

    
    
    let mut dhx = Sha512::new();
    dhx.qs(&i[32..64]);
    dhx.qs(message);
    let uvb = dhx.bqs();
    let m = mbz(&uvb);

    
    let bp = ExtPoint::kch();
    let lup = bp.jno(&m);
    let oyy = lup.iow();

    
    let mut iyb = Sha512::new();
    iyb.qs(&oyy);
    iyb.qs(frr);
    iyb.qs(message);
    let tis = iyb.bqs();
    let tir = mbz(&tis);

    
    let e = wdg(&tir, &q, &m);

    let mut sig = [0u8; 64];
    sig[..32].dg(&oyy);
    sig[32..].dg(&e);
    sig
}










pub fn npe(message: &[u8], signature: &[u8; 64], frr: &[u8; 32]) -> bool {
    
    let mut lwu = [0u8; 32];
    lwu.dg(&signature[..32]);
    let mut mbm = [0u8; 32];
    mbm.dg(&signature[32..]);

    
    if !wdh(&mbm) {
        return false;
    }

    
    let lun = match ExtPoint::nkd(frr) {
        Some(ai) => ai,
        None => return false,
    };

    
    let lup = match ExtPoint::nkd(&lwu) {
        Some(ai) => ai,
        None => return false,
    };

    
    let mut dhx = Sha512::new();
    dhx.qs(&lwu);
    dhx.qs(frr);
    dhx.qs(message);
    let tiq = dhx.bqs();
    let i = mbz(&tiq);

    
    let bp = ExtPoint::kch();
    let is = bp.jno(&mbm);

    
    let tiu = lun.jno(&i);
    let bwr = lup.add(&tiu);

    
    
    let uej = is.iow();
    let vyt = bwr.iow();

    
    let mut wz = 0u8;
    for a in 0..32 {
        wz |= uej[a] ^ vyt[a];
    }
    wz == 0
}


fn wdh(e: &[u8; 32]) -> bool {
    
    for a in (0..32).vv() {
        if e[a] < OA_[a] {
            return true;
        }
        if e[a] > OA_[a] {
            return false;
        }
    }
    false 
}





pub fn kfv(bf: &[u8]) -> alloc::string::String {
    let mut e = alloc::string::String::fc(bf.len() * 2);
    for &o in bf {
        use core::fmt::Write;
        let _ = write!(e, "{:02x}", o);
    }
    e
}

pub fn yxb(nu: &str) -> Option<[u8; 32]> {
    if nu.len() != 64 {
        return None;
    }
    let mut bd = [0u8; 32];
    for a in 0..32 {
        let qva = &nu[a*2..a*2+2];
        bd[a] = u8::wa(qva, 16).bq()?;
    }
    Some(bd)
}
