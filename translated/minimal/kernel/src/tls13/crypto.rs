







use alloc::vec::Vec;






const CSO_: [u32; 8] = [
    0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a,
    0x510e527f, 0x9b05688c, 0x1f83d9ab, 0x5be0cd19,
];


const CSP_: [u32; 64] = [
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
    g: [u32; 8],
    bi: [u8; 64],
    aic: usize,
    aeb: u64,
}

impl Sha256 {
    pub fn new() -> Self {
        Self {
            g: CSO_,
            bi: [0u8; 64],
            aic: 0,
            aeb: 0,
        }
    }
    
    pub fn qs(&mut self, f: &[u8]) {
        let mut l = 0usize;
        let len = f.len();

        
        if self.aic > 0 {
            let hsm = 64 - self.aic;
            let take = if len < hsm { len } else { hsm };
            self.bi[self.aic..self.aic + take]
                .dg(&f[..take]);
            self.aic += take;
            self.aeb += take as u64;
            l += take;

            if self.aic == 64 {
                self.jkb();
                self.aic = 0;
            }
        }

        
        while l + 64 <= len {
            self.bi.dg(&f[l..l + 64]);
            self.aeb += 64;
            self.jkb();
            l += 64;
        }

        
        let ia = len - l;
        if ia > 0 {
            self.bi[..ia].dg(&f[l..]);
            self.aic = ia;
            self.aeb += ia as u64;
        }
    }
    
    pub fn bqs(&mut self) -> [u8; 32] {
        
        let has = self.aeb * 8;
        self.bi[self.aic] = 0x80;
        self.aic += 1;
        
        if self.aic > 56 {
            while self.aic < 64 {
                self.bi[self.aic] = 0;
                self.aic += 1;
            }
            self.jkb();
            self.aic = 0;
        }
        
        while self.aic < 56 {
            self.bi[self.aic] = 0;
            self.aic += 1;
        }
        
        
        self.bi[56..64].dg(&has.ft());
        self.jkb();
        
        
        let mut result = [0u8; 32];
        for (a, &od) in self.g.iter().cf() {
            result[a * 4..a * 4 + 4].dg(&od.ft());
        }
        result
    }
    
    fn jkb(&mut self) {
        let mut d = [0u32; 64];
        
        
        for a in 0..16 {
            d[a] = u32::oa([
                self.bi[a * 4],
                self.bi[a * 4 + 1],
                self.bi[a * 4 + 2],
                self.bi[a * 4 + 3],
            ]);
        }
        
        for a in 16..64 {
            let cmq = d[a - 15].arw(7) ^ d[a - 15].arw(18) ^ (d[a - 15] >> 3);
            let bic = d[a - 2].arw(17) ^ d[a - 2].arw(19) ^ (d[a - 2] >> 10);
            d[a] = d[a - 16].cn(cmq).cn(d[a - 7]).cn(bic);
        }
        
        
        let [mut q, mut o, mut r, mut bc, mut aa, mut bb, mut at, mut i] = self.g;
        
        
        for a in 0..64 {
            let bic = aa.arw(6) ^ aa.arw(11) ^ aa.arw(25);
            let bm = (aa & bb) ^ ((!aa) & at);
            let ezn = i.cn(bic).cn(bm).cn(CSP_[a]).cn(d[a]);
            let cmq = q.arw(2) ^ q.arw(13) ^ q.arw(22);
            let hqp = (q & o) ^ (q & r) ^ (o & r);
            let idc = cmq.cn(hqp);
            
            i = at;
            at = bb;
            bb = aa;
            aa = bc.cn(ezn);
            bc = r;
            r = o;
            o = q;
            q = ezn.cn(idc);
        }
        
        
        self.g[0] = self.g[0].cn(q);
        self.g[1] = self.g[1].cn(o);
        self.g[2] = self.g[2].cn(r);
        self.g[3] = self.g[3].cn(bc);
        self.g[4] = self.g[4].cn(aa);
        self.g[5] = self.g[5].cn(bb);
        self.g[6] = self.g[6].cn(at);
        self.g[7] = self.g[7].cn(i);
    }
}


pub fn chw(f: &[u8]) -> [u8; 32] {
    let mut dhx = Sha256::new();
    dhx.qs(f);
    dhx.bqs()
}


pub fn drt(bs: &[u8], f: &[u8]) -> [u8; 32] {
    let mut jcc = [0u8; 64];
    
    if bs.len() > 64 {
        let ubg = chw(bs);
        jcc[..32].dg(&ubg);
    } else {
        jcc[..bs.len()].dg(bs);
    }
    
    
    let mut ofm = [0x36u8; 64];
    for (a, &eh) in jcc.iter().cf() {
        ofm[a] ^= eh;
    }
    
    let mut ff = Sha256::new();
    ff.qs(&ofm);
    ff.qs(f);
    let tus = ff.bqs();
    
    
    let mut osp = [0x5cu8; 64];
    for (a, &eh) in jcc.iter().cf() {
        osp[a] ^= eh;
    }
    
    let mut lrb = Sha256::new();
    lrb.qs(&osp);
    lrb.qs(&tus);
    lrb.bqs()
}






pub fn iyk(bsd: &[u8], trx: &[u8]) -> [u8; 32] {
    let bsd = if bsd.is_empty() { &[0u8; 32] } else { bsd };
    drt(bsd, trx)
}


pub fn tpe(vlo: &[u8; 32], co: &[u8], go: usize) -> Vec<u8> {
    let mut an = Vec::fc(go);
    let mut ab = Vec::new();
    let mut va = 1u8;
    
    while an.len() < go {
        let mut input = Vec::new();
        input.bk(&ab);
        input.bk(co);
        input.push(va);
        
        ab = drt(vlo, &input).ip();
        an.bk(&ab);
        va += 1;
    }
    
    an.dmu(go);
    an
}


pub fn gjc(eig: &[u8; 32], cu: &str, context: &[u8], go: usize) -> Vec<u8> {
    let syz = alloc::format!("tls13 {}", cu);
    let ohz = syz.as_bytes();
    
    let mut co = Vec::new();
    co.bk(&(go as u16).ft());
    co.push(ohz.len() as u8);
    co.bk(ohz);
    co.push(context.len() as u8);
    co.bk(context);
    
    tpe(eig, &co, go)
}


pub fn fgo(eig: &[u8; 32], cu: &str, ape: &[u8; 32]) -> [u8; 32] {
    let tg = gjc(eig, cu, ape, 32);
    let mut result = [0u8; 32];
    result.dg(&tg);
    result
}






const Aen: [u8; 256] = [
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


const Cjg: [u8; 10] = [0x01, 0x02, 0x04, 0x08, 0x10, 0x20, 0x40, 0x80, 0x1b, 0x36];



pub struct Aes128 {
    xh: [[u8; 16]; 11],
}

impl Aes128 {
    pub fn new(bs: &[u8; 16]) -> Self {
        let mut xh = [[0u8; 16]; 11];
        xh[0].dg(bs);
        
        
        for a in 1..11 {
            let vo = xh[a - 1];
            let mut od = [vo[12], vo[13], vo[14], vo[15]];
            
            
            let bcz = od[0];
            od[0] = Aen[od[1] as usize] ^ Cjg[a - 1];
            od[1] = Aen[od[2] as usize];
            od[2] = Aen[od[3] as usize];
            od[3] = Aen[bcz as usize];
            
            for fb in 0..4 {
                xh[a][fb] = vo[fb] ^ od[fb];
            }
            for fb in 4..16 {
                xh[a][fb] = vo[fb] ^ xh[a][fb - 4];
            }
        }
        
        Self { xh }
    }
    
    
    pub fn cke(&self, block: &mut [u8; 16]) {
        self.slj(block);
    }
    
    
    fn slj(&self, block: &mut [u8; 16]) {
        
        for a in 0..16 {
            block[a] ^= self.xh[0][a];
        }
        
        
        for jmv in 1..10 {
            self.icb(block);
            self.pki(block);
            self.uoq(block);
            for a in 0..16 {
                block[a] ^= self.xh[jmv][a];
            }
        }
        
        
        self.icb(block);
        self.pki(block);
        for a in 0..16 {
            block[a] ^= self.xh[10][a];
        }
    }
    
    fn icb(&self, block: &mut [u8; 16]) {
        for hf in block.el() {
            *hf = Aen[*hf as usize];
        }
    }
    
    fn pki(&self, block: &mut [u8; 16]) {
        
        let bcz = block[1];
        block[1] = block[5];
        block[5] = block[9];
        block[9] = block[13];
        block[13] = bcz;
        
        
        let (bin, aax) = (block[2], block[6]);
        block[2] = block[10];
        block[6] = block[14];
        block[10] = bin;
        block[14] = aax;
        
        
        let bcz = block[15];
        block[15] = block[11];
        block[11] = block[7];
        block[7] = block[3];
        block[3] = bcz;
    }
    
    fn uoq(&self, block: &mut [u8; 16]) {
        for a in 0..4 {
            let bj = a * 4;
            let (q, o, r, bc) = (block[bj], block[bj + 1], block[bj + 2], block[bj + 3]);
            
            block[bj] = erc(q, 2) ^ erc(o, 3) ^ r ^ bc;
            block[bj + 1] = q ^ erc(o, 2) ^ erc(r, 3) ^ bc;
            block[bj + 2] = q ^ o ^ erc(r, 2) ^ erc(bc, 3);
            block[bj + 3] = erc(q, 3) ^ o ^ r ^ erc(bc, 2);
        }
    }
}


fn erc(q: u8, o: u8) -> u8 {
    let mut result = 0u8;
    let mut q = q;
    let mut o = o;
    
    while o != 0 {
        if o & 1 != 0 {
            result ^= q;
        }
        let ton = q & 0x80;
        q <<= 1;
        if ton != 0 {
            q ^= 0x1b;
        }
        o >>= 1;
    }
    result
}


fn kxt(b: &[u8; 16], i: &[u8; 16]) -> [u8; 16] {
    let mut av = [0u8; 16];
    let mut p = *i;
    
    for a in 0..16 {
        for ga in 0..8 {
            if (b[a] >> (7 - ga)) & 1 == 1 {
                for fb in 0..16 {
                    av[fb] ^= p[fb];
                }
            }
            
            
            let uir = p[15] & 1;
            for fb in (1..16).vv() {
                p[fb] = (p[fb] >> 1) | (p[fb - 1] << 7);
            }
            p[0] >>= 1;
            
            if uir != 0 {
                p[0] ^= 0xe1; 
            }
        }
    }
    
    av
}


fn bqv(i: &[u8; 16], blv: &[u8], afm: &[u8]) -> [u8; 16] {
    let mut c = [0u8; 16];
    
    
    let qej = (blv.len() + 15) / 16;
    for a in 0..qej {
        let ay = a * 16;
        let ci = (ay + 16).v(blv.len());
        for fb in ay..ci {
            c[fb - ay] ^= blv[fb];
        }
        c = kxt(&c, i);
    }
    
    
    let rrc = (afm.len() + 15) / 16;
    for a in 0..rrc {
        let ay = a * 16;
        let ci = (ay + 16).v(afm.len());
        for fb in ay..ci {
            c[fb - ay] ^= afm[fb];
        }
        c = kxt(&c, i);
    }
    
    
    let jyq = (blv.len() as u64) * 8;
    let kmc = (afm.len() as u64) * 8;
    let mut fmx = [0u8; 16];
    fmx[..8].dg(&jyq.ft());
    fmx[8..].dg(&kmc.ft());
    
    for a in 0..16 {
        c[a] ^= fmx[a];
    }
    kxt(&c, i)
}


pub fn ijd(
    bs: &[u8; 16],
    brn: &[u8; 12],
    blv: &[u8],
    ajk: &[u8],
) -> Vec<u8> {
    let dye = Aes128::new(bs);
    
    
    let mut i = [0u8; 16];
    dye.cke(&mut i);
    
    
    let mut eea = [0u8; 16];
    eea[..12].dg(brn);
    eea[15] = 1;
    
    
    let mut hhp = eea;
    dye.cke(&mut hhp);
    
    
    let mut afm = Vec::fc(ajk.len() + 16);
    let xk = (ajk.len() + 15) / 16;
    
    for a in 0..xk {
        
        let va = 2u32 + a as u32;
        let mut block = [0u8; 16];
        block[..12].dg(brn);
        block[12..16].dg(&va.ft());
        dye.cke(&mut block);
        
        let ay = a * 16;
        let ci = (ay + 16).v(ajk.len());
        for fb in ay..ci {
            afm.push(ajk[fb] ^ block[fb - ay]);
        }
    }
    
    
    let kzb = bqv(&i, blv, &afm);
    let mut ll = [0u8; 16];
    for a in 0..16 {
        ll[a] = kzb[a] ^ hhp[a];
    }
    
    afm.bk(&ll);
    afm
}


pub fn muf(
    bs: &[u8; 16],
    brn: &[u8; 12],
    blv: &[u8],
    int: &[u8],
) -> Result<Vec<u8>, ()> {
    if int.len() < 16 {
        return Err(());
    }
    
    let prq = int.len() - 16;
    let afm = &int[..prq];
    let ll = &int[prq..];
    
    let dye = Aes128::new(bs);
    
    
    let mut i = [0u8; 16];
    dye.cke(&mut i);
    
    
    let mut eea = [0u8; 16];
    eea[..12].dg(brn);
    eea[15] = 1;
    
    let mut hhp = eea;
    dye.cke(&mut hhp);
    
    
    let kzb = bqv(&i, blv, afm);
    let mut kkh = [0u8; 16];
    for a in 0..16 {
        kkh[a] = kzb[a] ^ hhp[a];
    }
    
    
    let mut wz = 0u8;
    for a in 0..16 {
        wz |= kkh[a] ^ ll[a];
    }
    if wz != 0 {
        crate::serial_println!("[AES-GCM] Tag mismatch! computed={:02x?} received={:02x?}", 
            &kkh[..8], &ll[..8]);
        return Err(());
    }
    
    
    let mut ajk = Vec::fc(afm.len());
    let xk = (afm.len() + 15) / 16;
    
    for a in 0..xk {
        let va = 2u32 + a as u32;
        let mut block = [0u8; 16];
        block[..12].dg(brn);
        block[12..16].dg(&va.ft());
        dye.cke(&mut block);
        
        let ay = a * 16;
        let ci = (ay + 16).v(afm.len());
        for fb in ay..ci {
            ajk.push(afm[fb] ^ block[fb - ay]);
        }
    }
    
    Ok(ajk)
}






pub fn xwd(vlm: &[u8; 32]) -> [u8; 32] {
    
    let mut ar = [0u8; 32];
    ar[0] = 9;
    jxk(vlm, &ar)
}



pub fn jxk(eh: &[u8; 32], tm: &[u8; 32]) -> [u8; 32] {
    
    let mut bfe = *eh;
    bfe[0] &= 248;
    bfe[31] &= 127;
    bfe[31] |= 64;
    
    
    let mut pww = *tm;
    pww[31] &= 127;
    
    
    let mug = eqe(&pww);
    
    
    let mut jxi = DQ_;
    let mut jxj = HX_;
    
    
    let mut mrk = mug;
    let mut mrl = DQ_;
    
    
    
    
    let mut lvf = false;
    
    for a in (0..255).vv() {
        let nhz = ((bfe[a >> 3] >> (a & 7)) & 1) == 1;
        let ncw = (lvf ^ nhz) as u64;
        
        
        iud(&mut jxi, &mut mrk, ncw);
        iud(&mut jxj, &mut mrl, ncw);
        
        
        rxk(&mut jxi, &mut jxj, &mut mrk, &mut mrl, &mug);
        
        lvf = nhz;
    }
    
    
    let nub = lvf as u64;
    iud(&mut jxi, &mut jxj, nub);
    iud(&mut mrk, &mut mrl, nub);
    
    
    let result = zh(&jxi, &hjb(&jxj));
    bnb(&result)
}








fn rxk(
    lrm: &mut Aq, lrn: &mut Aq,
    lwm: &mut Aq, lwn: &mut Aq,
    qfw: &Aq,
) {
    let bin = bqq(lrm, lrn);
    let aax = cen(lrm, lrn);
    let aco = bqq(lwm, lwn);
    let bcx = cen(lwm, lwn);
    
    let dcl = afo(&bin);    
    let eji = afo(&aax);    
    
    let ejj = cen(&dcl, &eji);  
    
    let gua = zh(&bin, &bcx);  
    let gub = zh(&aax, &aco);  
    
    let jsi = bqq(&gua, &gub);  
    let ico = cen(&gua, &gub); 
    
    let mjj = afo(&jsi);       
    let icp = afo(&ico);      
    
    let icq = srk(&ejj); 
    
    let icr = zh(&dcl, &eji); 
    let jsh = bqq(&icq, &eji); 
    let ics = zh(&ejj, &jsh); 
    
    let mjk = zh(qfw, &icp); 
    let ict = mjj;                       
    
    
    *lrm = icr;  
    *lrn = ics;  
    
    
    *lwm = ict;  
    *lwn = mjk;  
}


pub(crate) type Aq = [u64; 5];

pub(crate) const HX_: Aq = [0, 0, 0, 0, 0];
pub(crate) const DQ_: Aq = [1, 0, 0, 0, 0];

pub(crate) fn eqe(o: &[u8; 32]) -> Aq {
    let mut i = [0u64; 5];
    
    fn hqe(o: &[u8]) -> u64 {
        let mut result = 0u64;
        for a in 0..8.v(o.len()) {
            result |= (o[a] as u64) << (a * 8);
        }
        result
    }
    
    i[0] = hqe(&o[0..8]) & 0x7ffffffffffff;
    i[1] = (hqe(&o[6..14]) >> 3) & 0x7ffffffffffff;
    i[2] = (hqe(&o[12..20]) >> 6) & 0x7ffffffffffff;
    i[3] = (hqe(&o[19..27]) >> 1) & 0x7ffffffffffff;
    i[4] = (hqe(&o[24..32]) >> 12) & 0x7ffffffffffff;
    
    i
}

pub(crate) fn bnb(i: &Aq) -> [u8; 32] {
    
    let mut ra = *i;
    kvj(&mut ra);
    
    
    
    
    
    let mut fm = (ra[0] + 19) >> 51;
    fm = (ra[1] + fm) >> 51;
    fm = (ra[2] + fm) >> 51;
    fm = (ra[3] + fm) >> 51;
    fm = (ra[4] + fm) >> 51;
    
    
    ra[0] += 19 * fm;
    
    
    const DU_: u64 = (1u64 << 51) - 1;
    ra[1] += ra[0] >> 51;
    ra[0] &= DU_;
    ra[2] += ra[1] >> 51;
    ra[1] &= DU_;
    ra[3] += ra[2] >> 51;
    ra[2] &= DU_;
    ra[4] += ra[3] >> 51;
    ra[3] &= DU_;
    
    ra[4] &= DU_;
    
    
    let mut e = [0u8; 32];
    e[0] = ra[0] as u8;
    e[1] = (ra[0] >> 8) as u8;
    e[2] = (ra[0] >> 16) as u8;
    e[3] = (ra[0] >> 24) as u8;
    e[4] = (ra[0] >> 32) as u8;
    e[5] = (ra[0] >> 40) as u8;
    e[6] = ((ra[0] >> 48) | (ra[1] << 3)) as u8;
    e[7] = (ra[1] >> 5) as u8;
    e[8] = (ra[1] >> 13) as u8;
    e[9] = (ra[1] >> 21) as u8;
    e[10] = (ra[1] >> 29) as u8;
    e[11] = (ra[1] >> 37) as u8;
    e[12] = ((ra[1] >> 45) | (ra[2] << 6)) as u8;
    e[13] = (ra[2] >> 2) as u8;
    e[14] = (ra[2] >> 10) as u8;
    e[15] = (ra[2] >> 18) as u8;
    e[16] = (ra[2] >> 26) as u8;
    e[17] = (ra[2] >> 34) as u8;
    e[18] = (ra[2] >> 42) as u8;
    e[19] = ((ra[2] >> 50) | (ra[3] << 1)) as u8;
    e[20] = (ra[3] >> 7) as u8;
    e[21] = (ra[3] >> 15) as u8;
    e[22] = (ra[3] >> 23) as u8;
    e[23] = (ra[3] >> 31) as u8;
    e[24] = (ra[3] >> 39) as u8;
    e[25] = ((ra[3] >> 47) | (ra[4] << 4)) as u8;
    e[26] = (ra[4] >> 4) as u8;
    e[27] = (ra[4] >> 12) as u8;
    e[28] = (ra[4] >> 20) as u8;
    e[29] = (ra[4] >> 28) as u8;
    e[30] = (ra[4] >> 36) as u8;
    e[31] = (ra[4] >> 44) as u8;
    
    e
}

pub(crate) fn kvj(i: &mut Aq) {
    let mut bmf;
    
    for _ in 0..2 {
        bmf = i[0] >> 51;
        i[0] &= 0x7ffffffffffff;
        i[1] += bmf;
        
        bmf = i[1] >> 51;
        i[1] &= 0x7ffffffffffff;
        i[2] += bmf;
        
        bmf = i[2] >> 51;
        i[2] &= 0x7ffffffffffff;
        i[3] += bmf;
        
        bmf = i[3] >> 51;
        i[3] &= 0x7ffffffffffff;
        i[4] += bmf;
        
        bmf = i[4] >> 51;
        i[4] &= 0x7ffffffffffff;
        i[0] += bmf * 19;
    }
}

pub(crate) fn bqq(q: &Aq, o: &Aq) -> Aq {
    [q[0] + o[0], q[1] + o[1], q[2] + o[2], q[3] + o[3], q[4] + o[4]]
}

pub(crate) fn cen(q: &Aq, o: &Aq) -> Aq {
    
    
    
    
    FieldElement51::vtn([
        (q[0] + 36028797018963664u64) - o[0],  
        (q[1] + 36028797018963952u64) - o[1],  
        (q[2] + 36028797018963952u64) - o[2],
        (q[3] + 36028797018963952u64) - o[3],
        (q[4] + 36028797018963952u64) - o[4],
    ])
}


struct FieldElement51;
impl FieldElement51 {
    fn vtn(mut ra: [u64; 5]) -> Aq {
        const DU_: u64 = (1u64 << 51) - 1;
        
        let acw = ra[0] >> 51;
        let rw = ra[1] >> 51;
        let tx = ra[2] >> 51;
        let der = ra[3] >> 51;
        let kfx = ra[4] >> 51;
        
        ra[0] &= DU_;
        ra[1] &= DU_;
        ra[2] &= DU_;
        ra[3] &= DU_;
        ra[4] &= DU_;
        
        ra[0] += kfx * 19;
        ra[1] += acw;
        ra[2] += rw;
        ra[3] += tx;
        ra[4] += der;
        
        ra
    }
}

pub(crate) fn zh(q: &Aq, o: &Aq) -> Aq {
    
    
    
    let bfv = q[0] as u128;
    let km = q[1] as u128;
    let oe = q[2] as u128;
    let vy = q[3] as u128;
    let bfw = q[4] as u128;
    
    let wu = o[0] as u128;
    let of = o[1] as u128;
    let tb = o[2] as u128;
    let ajw = o[3] as u128;
    let bay = o[4] as u128;
    
    
    
    
    
    
    
    
    let bin = bfv*wu + 19*(km*bay + oe*ajw + vy*tb + bfw*of);
    let aax = bfv*of + km*wu + 19*(oe*bay + vy*ajw + bfw*tb);
    let aco = bfv*tb + km*of + oe*wu + 19*(vy*bay + bfw*ajw);
    let bcx = bfv*ajw + km*tb + oe*of + vy*wu + 19*(bfw*bay);
    let dcl = bfv*bay + km*ajw + oe*tb + vy*of + bfw*wu;
    
    
    let mut i = [0u64; 5];
    
    let r = bin >> 51;
    i[0] = (bin & 0x7ffffffffffff) as u64;
    let aax = aax + r;
    
    let r = aax >> 51;
    i[1] = (aax & 0x7ffffffffffff) as u64;
    let aco = aco + r;
    
    let r = aco >> 51;
    i[2] = (aco & 0x7ffffffffffff) as u64;
    let bcx = bcx + r;
    
    let r = bcx >> 51;
    i[3] = (bcx & 0x7ffffffffffff) as u64;
    let dcl = dcl + r;
    
    let r = dcl >> 51;
    i[4] = (dcl & 0x7ffffffffffff) as u64;
    
    
    i[0] += (r as u64) * 19;
    
    
    let r = i[0] >> 51;
    i[0] &= 0x7ffffffffffff;
    i[1] += r;
    
    i
}

pub(crate) fn afo(q: &Aq) -> Aq {
    zh(q, q)
}

fn srk(q: &Aq) -> Aq {
    let mut m = [0u128; 5];
    for a in 0..5 {
        m[a] = (q[a] as u128) * 121666;
    }
    
    let mut i = [0u64; 5];
    for a in 0..5 {
        i[a] = (m[a] & 0x7ffffffffffff) as u64;
        if a < 4 {
            m[a + 1] += m[a] >> 51;
        } else {
            i[0] += ((m[4] >> 51) * 19) as u64;
        }
    }
    
    kvj(&mut i);
    i
}



pub(crate) fn luy(av: &Aq) -> (Aq, Aq) {
    
    let bin = afo(av);
    
    let aax = afo(&afo(&bin));
    
    let aco = zh(av, &aax);
    
    let bcx = zh(&bin, &aco);
    
    let dcl = afo(&bcx);
    
    let eji = zh(&aco, &dcl);
    
    
    let mut ejj = afo(&eji);
    for _ in 1..5 { ejj = afo(&ejj); }
    let gua = zh(&ejj, &eji);
    
    
    let mut gub = afo(&gua);
    for _ in 1..10 { gub = afo(&gub); }
    let jsi = zh(&gub, &gua);
    
    
    let mut ico = afo(&jsi);
    for _ in 1..20 { ico = afo(&ico); }
    let mjj = zh(&ico, &jsi);
    
    
    let mut icp = afo(&mjj);
    for _ in 1..10 { icp = afo(&icp); }
    let icq = zh(&icp, &gua);
    
    
    let mut icr = afo(&icq);
    for _ in 1..50 { icr = afo(&icr); }
    let jsh = zh(&icr, &icq);
    
    
    let mut ics = afo(&jsh);
    for _ in 1..100 { ics = afo(&ics); }
    let mjk = zh(&ics, &jsh);
    
    
    let mut ict = afo(&mjk);
    for _ in 1..50 { ict = afo(&ict); }
    let mjl = zh(&ict, &icq);
    
    (mjl, bcx)
}

pub(crate) fn hjb(av: &Aq) -> Aq {
    
    
    
    
    let (mjl, bcx) = luy(av);   
    
    
    let mut mjm = afo(&mjl);
    for _ in 1..5 { mjm = afo(&mjm); }
    
    
    zh(&mjm, &bcx)
}

pub(crate) fn iud(q: &mut Aq, o: &mut Aq, swap: u64) {
    let hs = (0u64).nj(swap);
    for a in 0..5 {
        let ab = hs & (q[a] ^ o[a]);
        q[a] ^= ab;
        o[a] ^= ab;
    }
}


pub fn peq() {
    crate::serial_println!("[CRYPTO] Running self-tests...");
    
    
    let fzw: [u8; 16] = [0x2b, 0x7e, 0x15, 0x16, 0x28, 0xae, 0xd2, 0xa6,
                            0xab, 0xf7, 0x15, 0x88, 0x09, 0xcf, 0x4f, 0x3c];
    let mut block: [u8; 16] = [0x32, 0x43, 0xf6, 0xa8, 0x88, 0x5a, 0x30, 0x8d,
                               0x31, 0x31, 0x98, 0xa2, 0xe0, 0x37, 0x07, 0x34];
    let qy: [u8; 16] = [0x39, 0x25, 0x84, 0x1d, 0x02, 0xdc, 0x09, 0xfb,
                              0xdc, 0x11, 0x85, 0x97, 0x19, 0x6a, 0x0b, 0x32];
    
    let dye = Aes128::new(&fzw);
    dye.cke(&mut block);
    
    if block == qy {
        crate::serial_println!("[CRYPTO] AES-128: PASS");
    } else {
        crate::serial_println!("[CRYPTO] AES-128: FAIL");
        crate::serial_println!("[CRYPTO] Expected: {:02x?}", &qy);
        crate::serial_println!("[CRYPTO] Got:      {:02x?}", &block);
    }
    
    
    let jpr = chw(&[]);
    let pkb: [u8; 32] = [
        0xe3, 0xb0, 0xc4, 0x42, 0x98, 0xfc, 0x1c, 0x14,
        0x9a, 0xfb, 0xf4, 0xc8, 0x99, 0x6f, 0xb9, 0x24,
        0x27, 0xae, 0x41, 0xe4, 0x64, 0x9b, 0x93, 0x4c,
        0xa4, 0x95, 0x99, 0x1b, 0x78, 0x52, 0xb8, 0x55,
    ];
    
    if jpr == pkb {
        crate::serial_println!("[CRYPTO] SHA-256: PASS");
    } else {
        crate::serial_println!("[CRYPTO] SHA-256: FAIL");
        crate::serial_println!("[CRYPTO] Expected: {:02x?}", &pkb[..16]);
        crate::serial_println!("[CRYPTO] Got:      {:02x?}", &jpr[..16]);
    }
    
    
    let bfe: [u8; 32] = [
        0xa5, 0x46, 0xe3, 0x6b, 0xf0, 0x52, 0x7c, 0x9d,
        0x3b, 0x16, 0x15, 0x4b, 0x82, 0x46, 0x5e, 0xdd,
        0x62, 0x14, 0x4c, 0x0a, 0xc1, 0xfc, 0x5a, 0x18,
        0x50, 0x6a, 0x22, 0x44, 0xba, 0x44, 0x9a, 0xc4,
    ];
    let jun: [u8; 32] = [
        0xe6, 0xdb, 0x68, 0x67, 0x58, 0x30, 0x30, 0xdb,
        0x35, 0x94, 0xc1, 0xa4, 0x24, 0xb1, 0x5f, 0x7c,
        0x72, 0x66, 0x24, 0xec, 0x26, 0xb3, 0x35, 0x3b,
        0x10, 0xa9, 0x03, 0xa6, 0xd0, 0xab, 0x1c, 0x4c,
    ];
    
    
    let mut ekh = jun;
    ekh[31] &= 0x7f;  
    let eqd = eqe(&ekh);
    let dbl = bnb(&eqd);
    if dbl == ekh {
        crate::serial_println!("[CRYPTO] FE round-trip: PASS");
    } else {
        crate::serial_println!("[CRYPTO] FE round-trip: FAIL");
        crate::serial_println!("[CRYPTO] Input:  {:02x?}", &ekh[..16]);
        crate::serial_println!("[CRYPTO] Output: {:02x?}", &dbl[..16]);
    }
    
    
    let uyi = DQ_;
    let uqn = zh(&uyi, &eqd);
    let oop = bnb(&uqn);
    if oop == ekh {
        crate::serial_println!("[CRYPTO] FE mul identity: PASS");
    } else {
        crate::serial_println!("[CRYPTO] FE mul identity: FAIL");
        crate::serial_println!("[CRYPTO] Expected: {:02x?}", &ekh[..16]);
        crate::serial_println!("[CRYPTO] Got:      {:02x?}", &oop[..16]);
    }
    
    
    let zpi = afo(&eqd);
    
    crate::serial_println!("[CRYPTO] FE sq: OK (no crash)");
    
    
    let qfn = bqq(&eqd, &HX_);
    let qfi = bnb(&qfn);
    if qfi == ekh {
        crate::serial_println!("[CRYPTO] FE add identity: PASS");
    } else {
        crate::serial_println!("[CRYPTO] FE add identity: FAIL");
    }
    
    let wvp = cen(&eqd, &HX_);
    let icb = bnb(&wvp);
    if icb == ekh {
        crate::serial_println!("[CRYPTO] FE sub identity: PASS");
    } else {
        crate::serial_println!("[CRYPTO] FE sub identity: FAIL");
        crate::serial_println!("[CRYPTO] Expected: {:02x?}", &ekh[..16]);
        crate::serial_println!("[CRYPTO] Got:      {:02x?}", &icb[..16]);
    }
    
    
    let ahc = afo(&eqd);
    let xxc = zh(&ahc, &ahc);      
    let xxd = afo(&ahc);              
    let qax = bnb(&xxc);
    let qay = bnb(&xxd);
    if qax == qay {
        crate::serial_println!("[CRYPTO] FE mul vs sq: PASS");
    } else {
        crate::serial_println!("[CRYPTO] FE mul vs sq: FAIL (z^2*z^2 != sq(z^2))");
        crate::serial_println!("[CRYPTO] mul: {:02x?}", &qax[..16]);
        crate::serial_println!("[CRYPTO] sq:  {:02x?}", &qay[..16]);
    }
    
    let jhv = bnb(&DQ_);
    
    
    let fxo: Aq = [2, 0, 0, 0, 0];
    let swf: Aq = [4, 0, 0, 0, 0];
    let xnm = zh(&fxo, &fxo);
    let ivn = bnb(&swf);
    let pwi = bnb(&xnm);
    if pwi == ivn {
        crate::serial_println!("[CRYPTO] FE 2*2=4: PASS");
    } else {
        crate::serial_println!("[CRYPTO] FE 2*2=4: FAIL");
        crate::serial_println!("[CRYPTO] Expected: {:02x?}", &ivn[..16]);
        crate::serial_println!("[CRYPTO] Got:      {:02x?}", &pwi[..16]);
    }
    
    
    let wro = afo(&fxo);
    let pmm = bnb(&wro);
    if pmm == ivn {
        crate::serial_println!("[CRYPTO] FE sq(2)=4: PASS");
    } else {
        crate::serial_println!("[CRYPTO] FE sq(2)=4: FAIL");
        crate::serial_println!("[CRYPTO] Expected: {:02x?}", &ivn[..16]);
        crate::serial_println!("[CRYPTO] Got:      {:02x?}", &pmm[..16]);
    }
    
    
    let xnk = hjb(&fxo);
    let xnl = zh(&fxo, &xnk);
    let pwp = bnb(&xnl);
    if pwp == jhv {
        crate::serial_println!("[CRYPTO] FE 2*2^-1: PASS");
    } else {
        crate::serial_println!("[CRYPTO] FE 2*2^-1: FAIL");
        crate::serial_println!("[CRYPTO] Expected: {:02x?}", &jhv[..16]);
        crate::serial_println!("[CRYPTO] Got:      {:02x?}", &pwp[..16]);
    }
    
    
    let fzh = hjb(&eqd);
    let uqm = zh(&eqd, &fzh);
    let ofe = bnb(&uqm);
    if ofe == jhv {
        crate::serial_println!("[CRYPTO] FE invert: PASS");
    } else {
        crate::serial_println!("[CRYPTO] FE invert: FAIL");
        crate::serial_println!("[CRYPTO] Expected: {:02x?}", &jhv[..16]);
        crate::serial_println!("[CRYPTO] Got:      {:02x?}", &ofe[..16]);
    }
    
    let qaj: [u8; 32] = [
        0xc3, 0xda, 0x55, 0x37, 0x9d, 0xe9, 0xc6, 0x90,
        0x8e, 0x94, 0xea, 0x4d, 0xf2, 0x8d, 0x08, 0x4f,
        0x32, 0xec, 0xcf, 0x03, 0x49, 0x1c, 0x71, 0xf7,
        0x54, 0xb4, 0x07, 0x55, 0x77, 0xa2, 0x85, 0x52,
    ];
    
    let jxl = jxk(&bfe, &jun);
    
    if jxl == qaj {
        crate::serial_println!("[CRYPTO] X25519: PASS");
    } else {
        crate::serial_println!("[CRYPTO] X25519: FAIL");
        crate::serial_println!("[CRYPTO] Expected: {:02x?}", &qaj[..16]);
        crate::serial_println!("[CRYPTO] Got:      {:02x?}", &jxl[..16]);
    }
    
    
    
    let kxs: [u8; 16] = [0x00; 16];
    let kxu: [u8; 12] = [0x00; 12];
    let kxv: [u8; 16] = [0x00; 16];
    let nxg: [u8; 16] = [
        0x03, 0x88, 0xda, 0xce, 0x60, 0xb6, 0xa3, 0x92,
        0xf3, 0x28, 0xc2, 0xb9, 0x71, 0xb2, 0xfe, 0x78,
    ];
    let nxh: [u8; 16] = [
        0xab, 0x6e, 0x47, 0xd4, 0x2c, 0xec, 0x13, 0xbd,
        0xf5, 0x3a, 0x67, 0xb2, 0x12, 0x57, 0xbd, 0xdf,
    ];
    
    let fjh = ijd(&kxs, &kxu, &[], &kxv);
    
    if &fjh[..16] == &nxg && &fjh[16..] == &nxh {
        crate::serial_println!("[CRYPTO] AES-GCM: PASS");
    } else {
        crate::serial_println!("[CRYPTO] AES-GCM: FAIL");
        crate::serial_println!("[CRYPTO] Expected CT:  {:02x?}", &nxg);
        crate::serial_println!("[CRYPTO] Got CT:       {:02x?}", &fjh[..16]);
        crate::serial_println!("[CRYPTO] Expected TAG: {:02x?}", &nxh);
        crate::serial_println!("[CRYPTO] Got TAG:      {:02x?}", &fjh[16..]);
    }
    
    crate::serial_println!("[CRYPTO] Self-tests complete");
}



pub fn xrh() -> (usize, usize) {
    let mut cg = 0usize;
    let mut gv = 0usize;

    
    let fzw: [u8; 16] = [0x2b,0x7e,0x15,0x16,0x28,0xae,0xd2,0xa6,
                              0xab,0xf7,0x15,0x88,0x09,0xcf,0x4f,0x3c];
    let mut block: [u8; 16] = [0x32,0x43,0xf6,0xa8,0x88,0x5a,0x30,0x8d,
                                0x31,0x31,0x98,0xa2,0xe0,0x37,0x07,0x34];
    let spf: [u8; 16] = [0x39,0x25,0x84,0x1d,0x02,0xdc,0x09,0xfb,
                                   0xdc,0x11,0x85,0x97,0x19,0x6a,0x0b,0x32];
    Aes128::new(&fzw).cke(&mut block);
    if block == spf { cg += 1; } else { gv += 1; }

    
    let jpr = chw(&[]);
    let wls: [u8; 32] = [
        0xe3,0xb0,0xc4,0x42,0x98,0xfc,0x1c,0x14,
        0x9a,0xfb,0xf4,0xc8,0x99,0x6f,0xb9,0x24,
        0x27,0xae,0x41,0xe4,0x64,0x9b,0x93,0x4c,
        0xa4,0x95,0x99,0x1b,0x78,0x52,0xb8,0x55,
    ];
    if jpr == wls { cg += 1; } else { gv += 1; }

    
    let wlq = chw(b"abc");
    let wlr: [u8; 32] = [
        0xba,0x78,0x16,0xbf,0x8f,0x01,0xcf,0xea,
        0x41,0x41,0x40,0xde,0x5d,0xae,0x22,0x23,
        0xb0,0x03,0x61,0xa3,0x96,0x17,0x7a,0x9c,
        0xb4,0x10,0xff,0x61,0xf2,0x00,0x15,0xad,
    ];
    if wlq == wlr { cg += 1; } else { gv += 1; }

    
    let bfe: [u8; 32] = [
        0xa5,0x46,0xe3,0x6b,0xf0,0x52,0x7c,0x9d,
        0x3b,0x16,0x15,0x4b,0x82,0x46,0x5e,0xdd,
        0x62,0x14,0x4c,0x0a,0xc1,0xfc,0x5a,0x18,
        0x50,0x6a,0x22,0x44,0xba,0x44,0x9a,0xc4,
    ];
    let jun: [u8; 32] = [
        0xe6,0xdb,0x68,0x67,0x58,0x30,0x30,0xdb,
        0x35,0x94,0xc1,0xa4,0x24,0xb1,0x5f,0x7c,
        0x72,0x66,0x24,0xec,0x26,0xb3,0x35,0x3b,
        0x10,0xa9,0x03,0xa6,0xd0,0xab,0x1c,0x4c,
    ];
    let xwe: [u8; 32] = [
        0xc3,0xda,0x55,0x37,0x9d,0xe9,0xc6,0x90,
        0x8e,0x94,0xea,0x4d,0xf2,0x8d,0x08,0x4f,
        0x32,0xec,0xcf,0x03,0x49,0x1c,0x71,0xf7,
        0x54,0xb4,0x07,0x55,0x77,0xa2,0x85,0x52,
    ];
    let jxl = jxk(&bfe, &jun);
    if jxl == xwe { cg += 1; } else { gv += 1; }

    
    let kxs: [u8; 16] = [0x00; 16];
    let kxu: [u8; 12] = [0x00; 12];
    let kxv: [u8; 16] = [0x00; 16];
    let tak: [u8; 16] = [
        0x03,0x88,0xda,0xce,0x60,0xb6,0xa3,0x92,
        0xf3,0x28,0xc2,0xb9,0x71,0xb2,0xfe,0x78,
    ];
    let tal: [u8; 16] = [
        0xab,0x6e,0x47,0xd4,0x2c,0xec,0x13,0xbd,
        0xf5,0x3a,0x67,0xb2,0x12,0x57,0xbd,0xdf,
    ];
    let fjh = ijd(&kxs, &kxu, &[], &kxv);
    if &fjh[..16] == &tak && &fjh[16..] == &tal {
        cg += 1;
    } else {
        gv += 1;
    }

    
    let tpj = b"Jefe";
    let tph = b"what do ya want for nothing?";
    let tpk = drt(tpj, tph);
    let tpi: [u8; 32] = [
        0x5b,0xdc,0xc1,0x46,0xbf,0x60,0x75,0x4e,
        0x6a,0x04,0x24,0x26,0x08,0x95,0x75,0xc7,
        0x5a,0x00,0x3f,0x08,0x9d,0x27,0x39,0x83,
        0x9d,0xec,0x58,0xb9,0x64,0xec,0x38,0x43,
    ];
    if tpk == tpi { cg += 1; } else { gv += 1; }

    (cg, gv)
}
