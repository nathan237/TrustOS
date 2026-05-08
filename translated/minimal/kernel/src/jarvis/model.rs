





















use alloc::vec::Vec;
use alloc::vec;






pub const BI_: usize = 256;


pub const E_: usize = 256;


pub const GE_: usize = 4;


pub const DA_: usize = E_ / GE_; 


pub const Z_: usize = 1024;


pub const BB_: usize = 4;


pub const DF_: usize = 256;


pub const HT_: f32 = 1e-5;



fn apq(x: f32) -> f32 {
    if x <= 0.0 { return 0.0; }
    let bits = x.to_bits();
    let uc = f32::from_bits((bits >> 1) + 0x1FBD_1DF5);
    (uc + x / uc) * 0.5
}






pub struct LayerWeights {
    
    pub rms_attn: Vec<f32>,
    
    pub w_q: Vec<f32>,
    
    pub w_k: Vec<f32>,
    
    pub w_v: Vec<f32>,
    
    pub w_o: Vec<f32>,
    
    pub rms_ffn: Vec<f32>,
    
    pub w_gate: Vec<f32>,
    
    pub w_up: Vec<f32>,
    
    pub w_down: Vec<f32>,
}

impl LayerWeights {
    
    fn bns(seed: &mut u64) -> Self {
        let efs = 1.0 / apq(E_ as f32);
        let cji = 1.0 / apq(Z_ as f32);

        LayerWeights {
            rms_attn: vec![1.0f32; E_],
            w_q: afm(E_ * E_, efs, seed),
            w_k: afm(E_ * E_, efs, seed),
            w_v: afm(E_ * E_, efs, seed),
            w_o: afm(E_ * E_, efs, seed),
            rms_ffn: vec![1.0f32; E_],
            w_gate: afm(E_ * Z_, cji, seed),
            w_up: afm(E_ * Z_, cji, seed),
            w_down: afm(Z_ * E_, cji, seed),
        }
    }

    
    pub fn param_count(&self) -> usize {
        E_  
        + E_ * E_ * 4  
        + E_  
        + E_ * Z_ * 2  
        + Z_ * E_  
    }
}


pub struct TransformerWeights {
    
    pub token_embed: Vec<f32>,
    
    pub pos_embed: Vec<f32>,
    
    pub layers: Vec<LayerWeights>,
    
    pub rms_final: Vec<f32>,
    
    pub w_output: Vec<f32>,
}

impl TransformerWeights {
    
    pub fn bns() -> Self {
        let mut seed = 42_u64; 

        let hvj = 1.0 / apq(E_ as f32);

        let mut layers = Vec::with_capacity(BB_);
        for _ in 0..BB_ {
            layers.push(LayerWeights::bns(&mut seed));
        }

        TransformerWeights {
            token_embed: afm(BI_ * E_, hvj, &mut seed),
            pos_embed: afm(DF_ * E_, 0.02, &mut seed),
            layers,
            rms_final: vec![1.0f32; E_],
            w_output: afm(E_ * BI_, hvj, &mut seed),
        }
    }

    
    pub fn param_count(&self) -> usize {
        BI_ * E_           
        + DF_ * E_            
        + self.layers.iter().map(|l| l.param_count()).sum::<usize>()
        + E_                       
        + E_ * BI_         
    }

    
    pub fn memory_bytes(&self) -> usize {
        self.param_count() * 4
    }

    
    pub fn serialize(&self) -> Vec<f32> {
        let mut data = Vec::with_capacity(self.param_count());
        data.extend_from_slice(&self.token_embed);
        data.extend_from_slice(&self.pos_embed);
        for bj in &self.layers {
            data.extend_from_slice(&bj.rms_attn);
            data.extend_from_slice(&bj.w_q);
            data.extend_from_slice(&bj.w_k);
            data.extend_from_slice(&bj.w_v);
            data.extend_from_slice(&bj.w_o);
            data.extend_from_slice(&bj.rms_ffn);
            data.extend_from_slice(&bj.w_gate);
            data.extend_from_slice(&bj.w_up);
            data.extend_from_slice(&bj.w_down);
        }
        data.extend_from_slice(&self.rms_final);
        data.extend_from_slice(&self.w_output);
        data
    }

    
    
    pub fn serialize_to_bytes(&self) -> Vec<u8> {
        let nb = self.param_count() * 4;
        let mut bytes = Vec::with_capacity(nb);

        fn azd(bytes: &mut Vec<u8>, xn: &[f32]) {
            for f in xn {
                bytes.extend_from_slice(&f.to_be_bytes());
            }
        }

        azd(&mut bytes, &self.token_embed);
        azd(&mut bytes, &self.pos_embed);
        for bj in &self.layers {
            azd(&mut bytes, &bj.rms_attn);
            azd(&mut bytes, &bj.w_q);
            azd(&mut bytes, &bj.w_k);
            azd(&mut bytes, &bj.w_v);
            azd(&mut bytes, &bj.w_o);
            azd(&mut bytes, &bj.rms_ffn);
            azd(&mut bytes, &bj.w_gate);
            azd(&mut bytes, &bj.w_up);
            azd(&mut bytes, &bj.w_down);
        }
        azd(&mut bytes, &self.rms_final);
        azd(&mut bytes, &self.w_output);
        bytes
    }

    
    pub fn byt(data: &[f32]) -> Option<Self> {
        let mut pos = 0;

        let token_embed = yd(data, &mut pos, BI_ * E_)?;
        let pos_embed = yd(data, &mut pos, DF_ * E_)?;

        let mut layers = Vec::with_capacity(BB_);
        for _ in 0..BB_ {
            let rms_attn = yd(data, &mut pos, E_)?;
            let w_q = yd(data, &mut pos, E_ * E_)?;
            let w_k = yd(data, &mut pos, E_ * E_)?;
            let w_v = yd(data, &mut pos, E_ * E_)?;
            let w_o = yd(data, &mut pos, E_ * E_)?;
            let rms_ffn = yd(data, &mut pos, E_)?;
            let w_gate = yd(data, &mut pos, E_ * Z_)?;
            let w_up = yd(data, &mut pos, E_ * Z_)?;
            let w_down = yd(data, &mut pos, Z_ * E_)?;

            layers.push(LayerWeights {
                rms_attn, w_q, w_k, w_v, w_o, rms_ffn, w_gate, w_up, w_down,
            });
        }

        let rms_final = yd(data, &mut pos, E_)?;
        let w_output = yd(data, &mut pos, E_ * BI_)?;

        Some(TransformerWeights {
            token_embed, pos_embed, layers, rms_final, w_output,
        })
    }

    
    pub fn reset(&mut self) {
        let lyv = Self::bns();
        *self = lyv;
    }
}






fn yd(data: &[f32], pos: &mut usize, len: usize) -> Option<Vec<f32>> {
    if *pos + len > data.len() { return None; }
    let v = data[*pos..*pos + len].to_vec();
    *pos += len;
    Some(v)
}


fn afm(len: usize, scale: f32, seed: &mut u64) -> Vec<f32> {
    let mut v = Vec::with_capacity(len);
    for _ in 0..len {
        v.push(hcz(seed) * scale);
    }
    v
}


fn hcz(state: &mut u64) -> f32 {
    let mut x = *state;
    x ^= x << 13;
    x ^= x >> 7;
    x ^= x << 17;
    *state = x;
    
    let bits = (x >> 40) as u32; 
    (bits as f32 / (1u32 << 24) as f32) * 2.0 - 1.0
}
