





















use alloc::vec::Vec;
use alloc::vec;






pub const BG_: usize = 256;


pub const E_: usize = 256;


pub const FP_: usize = 4;


pub const CU_: usize = E_ / FP_; 


pub const Y_: usize = 1024;


pub const AZ_: usize = 4;


pub const CY_: usize = 256;


pub const HC_: f32 = 1e-5;



fn ccw(b: f32) -> f32 {
    if b <= 0.0 { return 0.0; }
    let fs = b.bsr();
    let anj = f32::bhb((fs >> 1) + 0x1FBD_1DF5);
    (anj + b / anj) * 0.5
}






pub struct LayerWeights {
    
    pub cmh: Vec<f32>,
    
    pub biw: Vec<f32>,
    
    pub biu: Vec<f32>,
    
    pub bpg: Vec<f32>,
    
    pub biv: Vec<f32>,
    
    pub cmi: Vec<f32>,
    
    pub bit: Vec<f32>,
    
    pub bpf: Vec<f32>,
    
    pub bpe: Vec<f32>,
}

impl LayerWeights {
    
    fn dtm(dv: &mut u64) -> Self {
        let ike = 1.0 / ccw(E_ as f32);
        let fil = 1.0 / ccw(Y_ as f32);

        LayerWeights {
            cmh: vec![1.0f32; E_],
            biw: bhu(E_ * E_, ike, dv),
            biu: bhu(E_ * E_, ike, dv),
            bpg: bhu(E_ * E_, ike, dv),
            biv: bhu(E_ * E_, ike, dv),
            cmi: vec![1.0f32; E_],
            bit: bhu(E_ * Y_, fil, dv),
            bpf: bhu(E_ * Y_, fil, dv),
            bpe: bhu(Y_ * E_, fil, dv),
        }
    }

    
    pub fn vm(&self) -> usize {
        E_  
        + E_ * E_ * 4  
        + E_  
        + E_ * Y_ * 2  
        + Y_ * E_  
    }
}


pub struct TransformerWeights {
    
    pub bpa: Vec<f32>,
    
    pub cgq: Vec<f32>,
    
    pub my: Vec<LayerWeights>,
    
    pub chg: Vec<f32>,
    
    pub bft: Vec<f32>,
}

impl TransformerWeights {
    
    pub fn dtm() -> Self {
        let mut dv = 42_u64; 

        let npo = 1.0 / ccw(E_ as f32);

        let mut my = Vec::fc(AZ_);
        for _ in 0..AZ_ {
            my.push(LayerWeights::dtm(&mut dv));
        }

        TransformerWeights {
            bpa: bhu(BG_ * E_, npo, &mut dv),
            cgq: bhu(CY_ * E_, 0.02, &mut dv),
            my,
            chg: vec![1.0f32; E_],
            bft: bhu(E_ * BG_, npo, &mut dv),
        }
    }

    
    pub fn vm(&self) -> usize {
        BG_ * E_           
        + CY_ * E_            
        + self.my.iter().map(|dm| dm.vm()).sum::<usize>()
        + E_                       
        + E_ * BG_         
    }

    
    pub fn omv(&self) -> usize {
        self.vm() * 4
    }

    
    pub fn gsd(&self) -> Vec<f32> {
        let mut f = Vec::fc(self.vm());
        f.bk(&self.bpa);
        f.bk(&self.cgq);
        for fl in &self.my {
            f.bk(&fl.cmh);
            f.bk(&fl.biw);
            f.bk(&fl.biu);
            f.bk(&fl.bpg);
            f.bk(&fl.biv);
            f.bk(&fl.cmi);
            f.bk(&fl.bit);
            f.bk(&fl.bpf);
            f.bk(&fl.bpe);
        }
        f.bk(&self.chg);
        f.bk(&self.bft);
        f
    }

    
    
    pub fn pih(&self) -> Vec<u8> {
        let aal = self.vm() * 4;
        let mut bf = Vec::fc(aal);

        fn ctn(bf: &mut Vec<u8>, aue: &[f32]) {
            for bb in aue {
                bf.bk(&bb.ft());
            }
        }

        ctn(&mut bf, &self.bpa);
        ctn(&mut bf, &self.cgq);
        for fl in &self.my {
            ctn(&mut bf, &fl.cmh);
            ctn(&mut bf, &fl.biw);
            ctn(&mut bf, &fl.biu);
            ctn(&mut bf, &fl.bpg);
            ctn(&mut bf, &fl.biv);
            ctn(&mut bf, &fl.cmi);
            ctn(&mut bf, &fl.bit);
            ctn(&mut bf, &fl.bpf);
            ctn(&mut bf, &fl.bpe);
        }
        ctn(&mut bf, &self.chg);
        ctn(&mut bf, &self.bft);
        bf
    }

    
    pub fn eos(f: &[f32]) -> Option<Self> {
        let mut u = 0;

        let bpa = avb(f, &mut u, BG_ * E_)?;
        let cgq = avb(f, &mut u, CY_ * E_)?;

        let mut my = Vec::fc(AZ_);
        for _ in 0..AZ_ {
            let cmh = avb(f, &mut u, E_)?;
            let biw = avb(f, &mut u, E_ * E_)?;
            let biu = avb(f, &mut u, E_ * E_)?;
            let bpg = avb(f, &mut u, E_ * E_)?;
            let biv = avb(f, &mut u, E_ * E_)?;
            let cmi = avb(f, &mut u, E_)?;
            let bit = avb(f, &mut u, E_ * Y_)?;
            let bpf = avb(f, &mut u, E_ * Y_)?;
            let bpe = avb(f, &mut u, Y_ * E_)?;

            my.push(LayerWeights {
                cmh, biw, biu, bpg, biv, cmi, bit, bpf, bpe,
            });
        }

        let chg = avb(f, &mut u, E_)?;
        let bft = avb(f, &mut u, E_ * BG_)?;

        Some(TransformerWeights {
            bpa, cgq, my, chg, bft,
        })
    }

    
    pub fn apa(&mut self) {
        let sxl = Self::dtm();
        *self = sxl;
    }
}






fn avb(f: &[f32], u: &mut usize, len: usize) -> Option<Vec<f32>> {
    if *u + len > f.len() { return None; }
    let p = f[*u..*u + len].ip();
    *u += len;
    Some(p)
}


fn bhu(len: usize, bv: f32, dv: &mut u64) -> Vec<f32> {
    let mut p = Vec::fc(len);
    for _ in 0..len {
        p.push(mrs(dv) * bv);
    }
    p
}


fn mrs(g: &mut u64) -> f32 {
    let mut b = *g;
    b ^= b << 13;
    b ^= b >> 7;
    b ^= b << 17;
    *g = b;
    
    let fs = (b >> 40) as u32; 
    (fs as f32 / (1u32 << 24) as f32) * 2.0 - 1.0
}
