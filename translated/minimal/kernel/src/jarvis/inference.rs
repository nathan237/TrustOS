











use alloc::vec::Vec;
use alloc::vec;
use super::model::*;
use super::tokenizer;







pub struct KVCache {
    
    k: Vec<Vec<f32>>,
    
    v: Vec<Vec<f32>>,
    
    pub len: usize,
}

impl KVCache {
    pub fn new() -> Self {
        KVCache {
            k: (0..BB_).map(|_| Vec::with_capacity(DF_ * E_)).collect(),
            v: (0..BB_).map(|_| Vec::with_capacity(DF_ * E_)).collect(),
            len: 0,
        }
    }

    pub fn clear(&mut self) {
        for layer_k in &mut self.k { layer_k.clear(); }
        for layer_v in &mut self.v { layer_v.clear(); }
        self.len = 0;
    }
}


pub struct InferenceConfig {
    
    pub temperature: f32,
    
    pub top_k: usize,
    
    pub alx: usize,
}

impl Default for InferenceConfig {
    fn default() -> Self {
        InferenceConfig {
            temperature: 0.8,
            top_k: 40,
            alx: 128,
        }
    }
}


pub struct InferenceEngine {
    
    kv_cache: KVCache,
    
    pub config: InferenceConfig,
    
    buf_x: Vec<f32>,      
    buf_xn: Vec<f32>,     
    buf_q: Vec<f32>,      
    buf_k: Vec<f32>,      
    buf_v: Vec<f32>,      
    buf_attn: Vec<f32>,   
    buf_gate: Vec<f32>,   
    buf_up: Vec<f32>,     
    buf_logits: Vec<f32>, 
    
    rng_state: u64,
}

impl InferenceEngine {
    pub fn new() -> Self {
        InferenceEngine {
            kv_cache: KVCache::new(),
            config: InferenceConfig::default(),
            buf_x: vec![0.0; E_],
            buf_xn: vec![0.0; E_],
            buf_q: vec![0.0; E_],
            buf_k: vec![0.0; E_],
            buf_v: vec![0.0; E_],
            buf_attn: vec![0.0; DF_],
            buf_gate: vec![0.0; Z_],
            buf_up: vec![0.0; Z_],
            buf_logits: vec![0.0; BI_],
            rng_state: crate::time::yf().wrapping_add(0xDEAD_BEEF),
        }
    }

    
    pub fn generate(&mut self, model: &TransformerWeights, nh: &[u8], alx: usize) -> Vec<u8> {
        self.kv_cache.clear();
        let max = alx.min(DF_);
        let mut output = Vec::with_capacity(max);

        
        for &abm in nh.iter().take(DF_ - 1) {
            self.forward_one(model, abm);
        }

        
        let mut evb = if !nh.is_empty() {
            self.sample_token_with_penalty(&output)
        } else {
            tokenizer::ABA_
        };

        for _ in 0..max {
            if evb == tokenizer::ADL_ { break; }
            output.push(evb);

            self.forward_one(model, evb);
            evb = self.sample_token_with_penalty(&output);
        }

        output
    }

    
    pub fn predict_next_token(&mut self, model: &TransformerWeights, context: &[u8]) -> u8 {
        self.kv_cache.clear();
        for &abm in context.iter().take(DF_) {
            self.forward_one(model, abm);
        }
        
        dhw(&self.buf_logits)
    }

    
    fn forward_one(&mut self, model: &TransformerWeights, abm: u8) {
        let pos = self.kv_cache.len;
        if pos >= DF_ { return; }

        
        let pku = abm as usize;
        for i in 0..E_ {
            self.buf_x[i] = model.token_embed[pku * E_ + i]
                           + model.pos_embed[pos * E_ + i];
        }

        
        for xv in 0..BB_ {
            let bj = &model.layers[xv];

            
            super::simd::aox(&mut self.buf_xn, &self.buf_x, &bj.rms_attn);

            
            super::simd::tk(&mut self.buf_q, &bj.w_q, &self.buf_xn, E_, E_);
            super::simd::tk(&mut self.buf_k, &bj.w_k, &self.buf_xn, E_, E_);
            super::simd::tk(&mut self.buf_v, &bj.w_v, &self.buf_xn, E_, E_);

            
            self.kv_cache.k[xv].extend_from_slice(&self.buf_k);
            self.kv_cache.v[xv].extend_from_slice(&self.buf_v);

            
            
            let ake = pos + 1; 
            let mut attn_out = vec![0.0f32; E_];

            for h in 0..GE_ {
                let epa = h * DA_;

                
                for t in 0..ake {
                    let mut score = 0.0f32;
                    for d in 0..DA_ {
                        score += self.buf_q[epa + d]
                               * self.kv_cache.k[xv][t * E_ + epa + d];
                    }
                    self.buf_attn[t] = score / (DA_ as f32).sqrt_approx();
                }

                

                
                jhb(&mut self.buf_attn[..ake]);

                
                for t in 0..ake {
                    let w = self.buf_attn[t];
                    for d in 0..DA_ {
                        attn_out[epa + d] +=
                            w * self.kv_cache.v[xv][t * E_ + epa + d];
                    }
                }
            }

            
            let mut exc = vec![0.0f32; E_];
            super::simd::tk(&mut exc, &bj.w_o, &attn_out, E_, E_);

            
            for i in 0..E_ {
                self.buf_x[i] += exc[i];
            }

            
            super::simd::aox(&mut self.buf_xn, &self.buf_x, &bj.rms_ffn);

            
            
            super::simd::tk(&mut self.buf_gate, &bj.w_gate, &self.buf_xn, E_, Z_);
            
            super::simd::tk(&mut self.buf_up, &bj.w_up, &self.buf_xn, E_, Z_);
            
            for i in 0..Z_ {
                let g = self.buf_gate[i];
                let sig = 1.0 / (1.0 + (-g).exp_approx());
                self.buf_gate[i] = g * sig * self.buf_up[i];
            }
            
            let mut bbr = vec![0.0f32; E_];
            super::simd::tk(&mut bbr, &bj.w_down, &self.buf_gate, Z_, E_);

            
            for i in 0..E_ {
                self.buf_x[i] += bbr[i];
            }
        }

        
        super::simd::aox(&mut self.buf_xn, &self.buf_x, &model.rms_final);

        
        super::simd::tk(&mut self.buf_logits, &model.w_output, &self.buf_xn, E_, BI_);

        self.kv_cache.len = pos + 1;
    }

    
    fn quo(&mut self) -> u8 {
        self.sample_token_with_penalty(&[])
    }

    
    fn sample_token_with_penalty(&mut self, recent_tokens: &[u8]) -> u8 {
        let ts = self.config.temperature;

        if ts <= 0.01 {
            
            return dhw(&self.buf_logits);
        }

        
        let mut logits = self.buf_logits.clone();
        for l in logits.iter_mut() {
            *l /= ts;
        }

        
        let iuj = recent_tokens.len().min(32);
        if iuj > 0 {
            let start = recent_tokens.len() - iuj;
            for &asl in &recent_tokens[start..] {
                let idx = asl as usize;
                if idx < BI_ {
                    
                    if logits[idx] > 0.0 {
                        logits[idx] /= 1.3;
                    } else {
                        logits[idx] *= 1.3;
                    }
                }
            }
        }

        
        if self.config.top_k > 0 && self.config.top_k < BI_ {
            let mut gcj: Vec<(f32, usize)> = logits.iter().copied()
                .enumerate().map(|(i, v)| (v, i)).collect();
            gcj.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(core::cmp::Ordering::Equal));
            let amz = gcj[self.config.top_k.min(gcj.len() - 1)].0;
            for l in logits.iter_mut() {
                if *l < amz { *l = f32::NEG_INFINITY; }
            }
        }

        
        jhb(&mut logits);

        
        let r = self.rand_f32();
        let mut dlv = 0.0f32;
        for (i, &aa) in logits.iter().enumerate() {
            dlv += aa;
            if dlv >= r {
                return i as u8;
            }
        }
        (BI_ - 1) as u8
    }

    
    fn rand_f32(&mut self) -> f32 {
        let mut x = self.rng_state;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.rng_state = x;
        ((x >> 40) as u32 as f32) / ((1u32 << 24) as f32)
    }
}








#[allow(dead_code)]
fn tk(out: &mut [f32], w: &[f32], x: &[f32], cols: usize, rows: usize) {
    for r in 0..rows {
        let mut sum = 0.0f32;
        let fk = r * cols;
        for c in 0..cols {
            sum += w[fk + c] * x[c];
        }
        out[r] = sum;
    }
}



#[allow(dead_code)]
fn aox(out: &mut [f32], x: &[f32], tv: &[f32]) {
    let ae = x.len();
    let mut ss = 0.0f32;
    for &v in x { ss += v * v; }
    let aeg = (ss / ae as f32 + HT_).sqrt_approx();
    let alu = 1.0 / aeg;
    for i in 0..ae {
        out[i] = x[i] * alu * tv[i];
    }
}


fn jhb(data: &mut [f32]) {
    if data.is_empty() { return; }
    let mut max = data[0];
    for &v in data.iter() { if v > max { max = v; } }
    let mut sum = 0.0f32;
    for v in data.iter_mut() {
        *v = (*v - max).exp_approx();
        sum += *v;
    }
    if sum > 0.0 {
        let ki = 1.0 / sum;
        for v in data.iter_mut() { *v *= ki; }
    }
}


fn dhw(data: &[f32]) -> u8 {
    let mut dja = 0u8;
    let mut hhm = f32::NEG_INFINITY;
    for (i, &v) in data.iter().enumerate() {
        if v > hhm {
            hhm = v;
            dja = i as u8;
        }
    }
    dja
}





trait Ra {
    fn exp_approx(self) -> f32;
    fn sqrt_approx(self) -> f32;
}

impl Ra for f32 {
    fn exp_approx(self) -> f32 {
        if self > 88.0 { return f32::MAX; }
        if self < -88.0 { return 0.0; }
        let a = (1 << 23) as f32 / core::f32::consts::LN_2;
        let b = (1 << 23) as f32 * (127.0 - 0.04368);
        let bits = ((a * self + b) as i32).max(0) as u32;
        f32::from_bits(bits)
    }

    fn sqrt_approx(self) -> f32 {
        if self <= 0.0 { return 0.0; }
        let bits = self.to_bits();
        let uc = f32::from_bits((bits >> 1) + 0x1FBD_1DF5);
        (uc + self / uc) * 0.5
    }
}







pub fn atj(model: &TransformerWeights, tokens: &[u8]) -> (f32, Vec<Vec<f32>>) {
    let mut engine = InferenceEngine::new();
    engine.config.temperature = 0.0; 

    let mut aah = 0.0f32;
    let mut heq = Vec::new();

    for t in 0..tokens.len().saturating_sub(1) {
        engine.forward_one(model, tokens[t]);

        
        let target = tokens[t + 1] as usize;
        let mut logits = engine.buf_logits.clone();

        
        let max = logits.iter().copied().fold(f32::NEG_INFINITY, f32::max);
        let mut jjr = 0.0f32;
        for l in logits.iter_mut() {
            *l = (*l - max).exp_approx();
            jjr += *l;
        }
        let nam = max + jjr.ln_approx();

        let ka = -(engine.buf_logits[target] - nam);
        aah += ka;

        heq.push(engine.buf_logits.clone());
    }

    let ae = (tokens.len() - 1).max(1);
    (aah / ae as f32, heq)
}

trait Ami {
    fn ln_approx(self) -> f32;
}
impl Ami for f32 {
    fn ln_approx(self) -> f32 {
        if self <= 0.0 { return -88.0; }
        let bits = self.to_bits();
        let e = ((bits >> 23) & 0xFF) as f32 - 127.0;
        let m = f32::from_bits((bits & 0x007FFFFF) | 0x3F800000);
        
        
        (e + (m - 1.0) * 1.4427) * core::f32::consts::LN_2
    }
}
