











use alloc::vec::Vec;
use alloc::vec;
use super::model::*;
use super::backprop::ModelGrads;


pub struct AdamState {
    
    pub ef: Vec<f32>,
    
    pub p: Vec<f32>,
    
    pub ab: u64,
    
    pub aad: f32,
    
    pub ilk: f32,
    
    pub ill: f32,
    
    pub cel: f32,
    
    pub thh: f32,
    
    pub pzf: f32,
}

impl AdamState {
    
    pub fn new(vm: usize) -> Self {
        AdamState {
            ef: vec![0.0; vm],
            p: vec![0.0; vm],
            ab: 0,
            aad: 0.001,
            ilk: 0.9,
            ill: 0.999,
            cel: 1e-8,
            thh: 1.0,
            pzf: 0.01,
        }
    }

    
    pub fn mqt(vm: usize, aad: f32) -> Self {
        let mut e = Self::new(vm);
        e.aad = aad;
        e
    }

    
    pub fn gu(&mut self, model: &mut TransformerWeights, arg: &ModelGrads) {
        self.ab += 1;

        
        let qof = 1.0 - owu(self.ilk, self.ab);
        let cde = 1.0 - owu(self.ill, self.ab);
        let cgb = self.aad / qof; 

        let mut w = 0;

        
        self.cvb(&mut model.bpa, &arg.dfs, &mut w, cgb, cde);

        
        self.cvb(&mut model.cgq, &arg.dfo, &mut w, cgb, cde);

        
        for dm in 0..AZ_ {
            let bnk = &arg.my[dm];
            let zv = &mut model.my[dm];

            self.cvb(&mut zv.cmh, &bnk.dfp, &mut w, cgb, cde);
            self.cvb(&mut zv.biw, &bnk.dfx, &mut w, cgb, cde);
            self.cvb(&mut zv.biu, &bnk.dfv, &mut w, cgb, cde);
            self.cvb(&mut zv.bpg, &bnk.dfz, &mut w, cgb, cde);
            self.cvb(&mut zv.biv, &bnk.dfw, &mut w, cgb, cde);
            self.cvb(&mut zv.cmi, &bnk.dfq, &mut w, cgb, cde);
            self.cvb(&mut zv.bit, &bnk.dfu, &mut w, cgb, cde);
            self.cvb(&mut zv.bpf, &bnk.dfy, &mut w, cgb, cde);
            self.cvb(&mut zv.bpe, &bnk.dft, &mut w, cgb, cde);
        }

        
        self.cvb(&mut model.chg, &arg.dfr, &mut w, cgb, cde);

        
        self.cvb(&mut model.bft, &arg.dfn, &mut w, cgb, cde);
    }

    
    fn cvb(&mut self, bix: &mut [f32], arg: &[f32], w: &mut usize, cgb: f32, cde: f32) {
        let pze = self.pzf;
        let uio = self.aad; 
        for a in 0..bix.len() {
            let fb = *w + a;
            if fb >= self.ef.len() { break; }

            let at = arg[a];

            
            self.ef[fb] = self.ilk * self.ef[fb] + (1.0 - self.ilk) * at;
            self.p[fb] = self.ill * self.p[fb] + (1.0 - self.ill) * at * at;

            
            let xqc = self.p[fb] / cde;

            
            if pze > 0.0 {
                bix[a] *= 1.0 - uio * pze;
            }

            
            bix[a] -= cgb * self.ef[fb] / (ccw(xqc) + self.cel);
        }
        *w += bix.len();
    }

    
    pub fn apa(&mut self) {
        for p in self.ef.el() { *p = 0.0; }
        for p in self.p.el() { *p = 0.0; }
        self.ab = 0;
    }

    
    pub fn omv(&self) -> usize {
        (self.ef.len() + self.p.len()) * 4
    }
}












pub fn kkw(gu: u64, tk: u64, fbf: u64, lka: f32, eex: f32) -> f32 {
    if tk == 0 { return lka; }
    if gu < fbf {
        
        eex + (lka - eex) * (gu as f32 / fbf.am(1) as f32)
    } else {
        
        let ruc = tk.ao(fbf).am(1);
        let li = (gu - fbf) as f32 / ruc as f32;
        let li = if li > 1.0 { 1.0 } else { li };
        eex + 0.5 * (lka - eex) * (1.0 + byz(li * 3.14159265))
    }
}


fn byz(b: f32) -> f32 {
    
    let akk = 3.14159265f32;
    let mut q = b;
    
    if q < 0.0 { q = -q; }
    while q > 2.0 * akk { q -= 2.0 * akk; }
    
    let ope = q > akk;
    if ope { q -= akk; }
    
    
    let bvy = akk * akk;
    let ap = (bvy - 4.0 * q * q) / (bvy + q * q);
    if ope { -ap } else { ap }
}





fn ccw(b: f32) -> f32 {
    if b <= 0.0 { return 0.0; }
    let fs = b.bsr();
    let anj = f32::bhb((fs >> 1) + 0x1FBD_1DF5);
    (anj + b / anj) * 0.5
}


fn owu(ar: f32, bgz: u64) -> f32 {
    let mut result = 1.0f32;
    let mut o = ar;
    let mut aa = bgz;
    while aa > 0 {
        if aa & 1 == 1 { result *= o; }
        o *= o;
        aa >>= 1;
    }
    result
}
