




use core::arch::x86_64::*;


pub fn anl() -> bool {
    super::cfe()
}


#[repr(C, align(16))]
pub struct Aes128Key {
    xh: [acb; 11],
}

impl Aes128Key {
    
    pub fn new(bs: &[u8; 16]) -> Self {
        if !anl() {
            panic!("AES-NI not available");
        }
        
        unsafe {
            let mut xh: [acb; 11] = core::mem::zeroed();
            
            
            xh[0] = byb(bs.fq() as *const acb);
            
            
            xh[1] = dnz(xh[0], 0x01);
            xh[2] = dnz(xh[1], 0x02);
            xh[3] = dnz(xh[2], 0x04);
            xh[4] = dnz(xh[3], 0x08);
            xh[5] = dnz(xh[4], 0x10);
            xh[6] = dnz(xh[5], 0x20);
            xh[7] = dnz(xh[6], 0x40);
            xh[8] = dnz(xh[7], 0x80);
            xh[9] = dnz(xh[8], 0x1B);
            xh[10] = dnz(xh[9], 0x36);
            
            Self { xh }
        }
    }
    
    
    pub fn cke(&self, ajk: &[u8; 16]) -> [u8; 16] {
        unsafe {
            let mut block = byb(ajk.fq() as *const acb);
            
            
            block = bjc(block, self.xh[0]);
            
            
            block = elq(block, self.xh[1]);
            block = elq(block, self.xh[2]);
            block = elq(block, self.xh[3]);
            block = elq(block, self.xh[4]);
            block = elq(block, self.xh[5]);
            block = elq(block, self.xh[6]);
            block = elq(block, self.xh[7]);
            block = elq(block, self.xh[8]);
            block = elq(block, self.xh[9]);
            
            
            block = ybd(block, self.xh[10]);
            
            let mut an = [0u8; 16];
            ccs(an.mw() as *mut acb, block);
            an
        }
    }
    
    
    pub fn ylk(&self, afm: &[u8; 16]) -> [u8; 16] {
        unsafe {
            let mut block = byb(afm.fq() as *const acb);
            
            
            block = bjc(block, self.xh[10]);
            
            
            block = elp(block, elr(self.xh[9]));
            block = elp(block, elr(self.xh[8]));
            block = elp(block, elr(self.xh[7]));
            block = elp(block, elr(self.xh[6]));
            block = elp(block, elr(self.xh[5]));
            block = elp(block, elr(self.xh[4]));
            block = elp(block, elr(self.xh[3]));
            block = elp(block, elr(self.xh[2]));
            block = elp(block, elr(self.xh[1]));
            
            
            block = ybc(block, self.xh[0]);
            
            let mut an = [0u8; 16];
            ccs(an.mw() as *mut acb, block);
            an
        }
    }
}


#[inline]
unsafe fn dnz(bs: acb, vqu: i32) -> acb {
    let mut bs = bs;
    let mut lhg = ybe(bs, vqu);
    lhg = ybg(lhg, 0xFF);
    bs = bjc(bs, fzn(bs, 4));
    bs = bjc(bs, fzn(bs, 4));
    bs = bjc(bs, fzn(bs, 4));
    bjc(bs, lhg)
}


pub struct Bxt {
    bs: Aes128Key,
    i: acb,  
}

impl Bxt {
    
    pub fn new(bs: &[u8; 16]) -> Self {
        let fzw = Aes128Key::new(bs);
        
        
        let ajs = [0u8; 16];
        let tio = fzw.cke(&ajs);
        
        let i = unsafe { byb(tio.fq() as *const acb) };
        
        Self { bs: fzw, i }
    }
    
    
    
    pub fn npy(&self, brn: &[u8; 12], ajk: &[u8], blv: &[u8]) -> (alloc::vec::Vec<u8>, [u8; 16]) {
        use alloc::vec::Vec;
        
        
        let mut va = [0u8; 16];
        va[..12].dg(brn);
        va[15] = 1;
        
        
        let hpi = self.bs.cke(&va);
        
        
        va[15] = 2;
        
        
        let mut afm = Vec::fc(ajk.len());
        
        for jj in ajk.btq(16) {
            let lhh = self.bs.cke(&va);
            
            for (a, &hf) in jj.iter().cf() {
                afm.push(hf ^ lhh[a]);
            }
            
            
            ody(&mut va);
        }
        
        
        let ll = self.nfj(&hpi, blv, &afm);
        
        (afm, ll)
    }
    
    
    
    pub fn ruw(&self, brn: &[u8; 12], afm: &[u8], blv: &[u8], ll: &[u8; 16]) -> Option<alloc::vec::Vec<u8>> {
        use alloc::vec::Vec;
        
        
        let mut va = [0u8; 16];
        va[..12].dg(brn);
        va[15] = 1;
        
        
        let hpi = self.bs.cke(&va);
        
        
        let sph = self.nfj(&hpi, blv, afm);
        
        
        let mut wz = 0u8;
        for (q, o) in ll.iter().fca(sph.iter()) {
            wz |= q ^ o;
        }
        
        if wz != 0 {
            return None; 
        }
        
        
        va[15] = 2;
        
        let mut ajk = Vec::fc(afm.len());
        
        for jj in afm.btq(16) {
            let lhh = self.bs.cke(&va);
            
            for (a, &hf) in jj.iter().cf() {
                ajk.push(hf ^ lhh[a]);
            }
            
            ody(&mut va);
        }
        
        Some(ajk)
    }
    
    
    fn nfj(&self, hpi: &[u8; 16], blv: &[u8], afm: &[u8]) -> [u8; 16] {
        
        
        
        
        let mut bqv = unsafe { mso() };
        
        
        for jj in blv.btq(16) {
            let mut block = [0u8; 16];
            block[..jj.len()].dg(jj);
            
            let f = unsafe { byb(block.fq() as *const acb) };
            bqv = unsafe { bjc(bqv, f) };
            bqv = kza(bqv, self.i);
        }
        
        
        for jj in afm.btq(16) {
            let mut block = [0u8; 16];
            block[..jj.len()].dg(jj);
            
            let f = unsafe { byb(block.fq() as *const acb) };
            bqv = unsafe { bjc(bqv, f) };
            bqv = kza(bqv, self.i);
        }
        
        
        let mut fmx = [0u8; 16];
        let jyq = (blv.len() as u64) * 8;
        let kmc = (afm.len() as u64) * 8;
        fmx[..8].dg(&jyq.ft());
        fmx[8..16].dg(&kmc.ft());
        
        let udy = unsafe { byb(fmx.fq() as *const acb) };
        bqv = unsafe { bjc(bqv, udy) };
        bqv = kza(bqv, self.i);
        
        
        let eea = unsafe { byb(hpi.fq() as *const acb) };
        bqv = unsafe { bjc(bqv, eea) };
        
        let mut ll = [0u8; 16];
        unsafe { ccs(ll.mw() as *mut acb, bqv) };
        ll
    }
}


fn ody(va: &mut [u8; 16]) {
    for a in (12..16).vv() {
        va[a] = va[a].cn(1);
        if va[a] != 0 {
            break;
        }
    }
}


#[inline]
fn kza(q: acb, o: acb) -> acb {
    unsafe {
        
        let bsq = jyb(q, o, 0x00);
        let ezx = jyb(q, o, 0x10);
        let idv = jyb(q, o, 0x01);
        let dct = jyb(q, o, 0x11);
        
        let ezx = bjc(ezx, idv);
        let idv = fzn(ezx, 8);
        let ezx = msp(ezx, 8);
        let bsq = bjc(bsq, idv);
        let dct = bjc(dct, ezx);
        
        
        let cur = gxj(bsq, 31);
        let fwy = gxj(dct, 31);
        let bsq = iie(bsq, 1);
        let dct = iie(dct, 1);
        
        let mli = msp(cur, 12);
        let fwy = fzn(fwy, 4);
        let cur = fzn(cur, 4);
        let bsq = iic(bsq, cur);
        let dct = iic(dct, fwy);
        let dct = iic(dct, mli);
        
        let cur = iie(bsq, 31);
        let fwy = iie(bsq, 30);
        let mli = iie(bsq, 25);
        
        let cur = bjc(cur, fwy);
        let cur = bjc(cur, mli);
        let fwy = msp(cur, 4);
        let cur = fzn(cur, 12);
        let bsq = bjc(bsq, cur);
        
        let fwx = gxj(bsq, 1);
        let ezx = gxj(bsq, 2);
        let idv = gxj(bsq, 7);
        let fwx = bjc(fwx, ezx);
        let fwx = bjc(fwx, idv);
        let fwx = bjc(fwx, fwy);
        let bsq = bjc(bsq, fwx);
        let dct = bjc(dct, bsq);
        
        dct
    }
}

extern crate alloc;
