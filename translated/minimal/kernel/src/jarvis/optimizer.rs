











use alloc::vec::Vec;
use alloc::vec;
use super::model::*;
use super::backprop::ModelGrads;


pub struct AdamState {
    
    pub m: Vec<f32>,
    
    pub v: Vec<f32>,
    
    pub t: u64,
    
    pub lr: f32,
    
    pub beta1: f32,
    
    pub beta2: f32,
    
    pub eps: f32,
    
    pub grad_clip: f32,
    
    pub weight_decay: f32,
}

impl AdamState {
    
    pub fn new(param_count: usize) -> Self {
        AdamState {
            m: vec![0.0; param_count],
            v: vec![0.0; param_count],
            t: 0,
            lr: 0.001,
            beta1: 0.9,
            beta2: 0.999,
            eps: 1e-8,
            grad_clip: 1.0,
            weight_decay: 0.01,
        }
    }

    
    pub fn hck(param_count: usize, lr: f32) -> Self {
        let mut j = Self::new(param_count);
        j.lr = lr;
        j
    }

    
    pub fn step(&mut self, model: &mut TransformerWeights, wg: &ModelGrads) {
        self.t += 1;

        
        let kar = 1.0 - ivr(self.beta1, self.t);
        let apv = 1.0 - ivr(self.beta2, self.t);
        let arl = self.lr / kar; 

        let mut idx = 0;

        
        self.update_slice(&mut model.token_embed, &wg.d_token_embed, &mut idx, arl, apv);

        
        self.update_slice(&mut model.pos_embed, &wg.d_pos_embed, &mut idx, arl, apv);

        
        for l in 0..BB_ {
            let aie = &wg.layers[l];
            let mo = &mut model.layers[l];

            self.update_slice(&mut mo.rms_attn, &aie.d_rms_attn, &mut idx, arl, apv);
            self.update_slice(&mut mo.w_q, &aie.d_wq, &mut idx, arl, apv);
            self.update_slice(&mut mo.w_k, &aie.d_wk, &mut idx, arl, apv);
            self.update_slice(&mut mo.w_v, &aie.d_wv, &mut idx, arl, apv);
            self.update_slice(&mut mo.w_o, &aie.d_wo, &mut idx, arl, apv);
            self.update_slice(&mut mo.rms_ffn, &aie.d_rms_ffn, &mut idx, arl, apv);
            self.update_slice(&mut mo.w_gate, &aie.d_wgate, &mut idx, arl, apv);
            self.update_slice(&mut mo.w_up, &aie.d_wup, &mut idx, arl, apv);
            self.update_slice(&mut mo.w_down, &aie.d_wdown, &mut idx, arl, apv);
        }

        
        self.update_slice(&mut model.rms_final, &wg.d_rms_final, &mut idx, arl, apv);

        
        self.update_slice(&mut model.w_output, &wg.d_output, &mut idx, arl, apv);
    }

    
    fn update_slice(&mut self, afx: &mut [f32], wg: &[f32], idx: &mut usize, arl: f32, apv: f32) {
        let jqy = self.weight_decay;
        let naz = self.lr; 
        for i in 0..afx.len() {
            let ay = *idx + i;
            if ay >= self.m.len() { break; }

            let g = wg[i];

            
            self.m[ay] = self.beta1 * self.m[ay] + (1.0 - self.beta1) * g;
            self.v[ay] = self.beta2 * self.v[ay] + (1.0 - self.beta2) * g * g;

            
            let pqv = self.v[ay] / apv;

            
            if jqy > 0.0 {
                afx[i] *= 1.0 - naz * jqy;
            }

            
            afx[i] -= arl * self.m[ay] / (apq(pqv) + self.eps);
        }
        *idx += afx.len();
    }

    
    pub fn reset(&mut self) {
        for v in self.m.iter_mut() { *v = 0.0; }
        for v in self.v.iter_mut() { *v = 0.0; }
        self.t = 0;
    }

    
    pub fn memory_bytes(&self) -> usize {
        (self.m.len() + self.v.len()) * 4
    }
}












pub fn foo(step: u64, ix: u64, cfb: u64, lr_max: f32, buc: f32) -> f32 {
    if ix == 0 { return lr_max; }
    if step < cfb {
        
        buc + (lr_max - buc) * (step as f32 / cfb.max(1) as f32)
    } else {
        
        let lcg = ix.saturating_sub(cfb).max(1);
        let progress = (step - cfb) as f32 / lcg as f32;
        let progress = if progress > 1.0 { 1.0 } else { progress };
        buc + 0.5 * (lr_max - buc) * (1.0 + anx(progress * 3.14159265))
    }
}


fn anx(x: f32) -> f32 {
    
    let pi = 3.14159265f32;
    let mut a = x;
    
    if a < 0.0 { a = -a; }
    while a > 2.0 * pi { a -= 2.0 * pi; }
    
    let ipk = a > pi;
    if ipk { a -= pi; }
    
    
    let ame = pi * pi;
    let val = (ame - 4.0 * a * a) / (ame + a * a);
    if ipk { -val } else { val }
}





fn apq(x: f32) -> f32 {
    if x <= 0.0 { return 0.0; }
    let bits = x.to_bits();
    let uc = f32::from_bits((bits >> 1) + 0x1FBD_1DF5);
    (uc + x / uc) * 0.5
}


fn ivr(base: f32, afe: u64) -> f32 {
    let mut result = 1.0f32;
    let mut b = base;
    let mut e = afe;
    while e > 0 {
        if e & 1 == 1 { result *= b; }
        b *= b;
        e >>= 1;
    }
    result
}
