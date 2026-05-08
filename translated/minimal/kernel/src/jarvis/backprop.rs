











use alloc::vec::Vec;
use alloc::vec;
use super::model::*;






pub struct LayerGrads {
    pub d_rms_attn: Vec<f32>,   
    pub d_wq: Vec<f32>,         
    pub d_wk: Vec<f32>,         
    pub d_wv: Vec<f32>,         
    pub d_wo: Vec<f32>,         
    pub d_rms_ffn: Vec<f32>,    
    pub d_wgate: Vec<f32>,      
    pub d_wup: Vec<f32>,        
    pub d_wdown: Vec<f32>,      
}

impl LayerGrads {
    pub fn new() -> Self {
        LayerGrads {
            d_rms_attn: vec![0.0; E_],
            d_wq: vec![0.0; E_ * E_],
            d_wk: vec![0.0; E_ * E_],
            d_wv: vec![0.0; E_ * E_],
            d_wo: vec![0.0; E_ * E_],
            d_rms_ffn: vec![0.0; E_],
            d_wgate: vec![0.0; E_ * Z_],
            d_wup: vec![0.0; E_ * Z_],
            d_wdown: vec![0.0; Z_ * E_],
        }
    }

    pub fn zero(&mut self) {
        for v in self.d_rms_attn.iter_mut() { *v = 0.0; }
        for v in self.d_wq.iter_mut() { *v = 0.0; }
        for v in self.d_wk.iter_mut() { *v = 0.0; }
        for v in self.d_wv.iter_mut() { *v = 0.0; }
        for v in self.d_wo.iter_mut() { *v = 0.0; }
        for v in self.d_rms_ffn.iter_mut() { *v = 0.0; }
        for v in self.d_wgate.iter_mut() { *v = 0.0; }
        for v in self.d_wup.iter_mut() { *v = 0.0; }
        for v in self.d_wdown.iter_mut() { *v = 0.0; }
    }
}


pub struct ModelGrads {
    pub d_token_embed: Vec<f32>,  
    pub d_pos_embed: Vec<f32>,    
    pub layers: Vec<LayerGrads>,
    pub d_rms_final: Vec<f32>,    
    pub d_output: Vec<f32>,       
}

impl ModelGrads {
    pub fn new() -> Self {
        let mut layers = Vec::with_capacity(BB_);
        for _ in 0..BB_ {
            layers.push(LayerGrads::new());
        }
        ModelGrads {
            d_token_embed: vec![0.0; BI_ * E_],
            d_pos_embed: vec![0.0; DF_ * E_],
            layers,
            d_rms_final: vec![0.0; E_],
            d_output: vec![0.0; E_ * BI_],
        }
    }

    pub fn zero(&mut self) {
        for v in self.d_token_embed.iter_mut() { *v = 0.0; }
        for v in self.d_pos_embed.iter_mut() { *v = 0.0; }
        for l in &mut self.layers { l.zero(); }
        for v in self.d_rms_final.iter_mut() { *v = 0.0; }
        for v in self.d_output.iter_mut() { *v = 0.0; }
    }

    
    pub fn count(&self) -> usize {
        self.d_token_embed.len() + self.d_pos_embed.len()
        + self.layers.iter().map(|l| {
            l.d_rms_attn.len() + l.d_wq.len() + l.d_wk.len() + l.d_wv.len()
            + l.d_wo.len() + l.d_rms_ffn.len() + l.d_wgate.len() + l.d_wup.len()
            + l.d_wdown.len()
        }).sum::<usize>()
        + self.d_rms_final.len() + self.d_output.len()
    }

    
    pub fn grad_norm(&self) -> f32 {
        let mut ss = 0.0f32;
        let awa = |ss: &mut f32, j: &[f32]| { for &g in j { *ss += g * g; } };
        awa(&mut ss, &self.d_token_embed);
        awa(&mut ss, &self.d_pos_embed);
        for l in &self.layers {
            awa(&mut ss, &l.d_rms_attn);
            awa(&mut ss, &l.d_wq); awa(&mut ss, &l.d_wk);
            awa(&mut ss, &l.d_wv); awa(&mut ss, &l.d_wo);
            awa(&mut ss, &l.d_rms_ffn);
            awa(&mut ss, &l.d_wgate); awa(&mut ss, &l.d_wup);
            awa(&mut ss, &l.d_wdown);
        }
        awa(&mut ss, &self.d_rms_final);
        awa(&mut ss, &self.d_output);
        apq(ss)
    }

    
    pub fn clip_norm(&mut self, max_norm: f32) {
        let mu = self.grad_norm();
        if mu > max_norm && mu > 0.0 {
            let j = max_norm / mu;
            let dr = |v: &mut [f32], j: f32| { for g in v.iter_mut() { *g *= j; } };
            dr(&mut self.d_token_embed, j);
            dr(&mut self.d_pos_embed, j);
            for l in &mut self.layers {
                dr(&mut l.d_rms_attn, j); dr(&mut l.d_wq, j); dr(&mut l.d_wk, j);
                dr(&mut l.d_wv, j); dr(&mut l.d_wo, j); dr(&mut l.d_rms_ffn, j);
                dr(&mut l.d_wgate, j); dr(&mut l.d_wup, j); dr(&mut l.d_wdown, j);
            }
            dr(&mut self.d_rms_final, j);
            dr(&mut self.d_output, j);
        }
    }

    
    pub fn accumulate(&mut self, other: &ModelGrads) {
        let add = |dst: &mut [f32], src: &[f32]| {
            for (d, j) in dst.iter_mut().zip(src.iter()) { *d += *j; }
        };
        add(&mut self.d_token_embed, &other.d_token_embed);
        add(&mut self.d_pos_embed, &other.d_pos_embed);
        for (dl, sl) in self.layers.iter_mut().zip(other.layers.iter()) {
            add(&mut dl.d_rms_attn, &sl.d_rms_attn);
            add(&mut dl.d_wq, &sl.d_wq); add(&mut dl.d_wk, &sl.d_wk);
            add(&mut dl.d_wv, &sl.d_wv); add(&mut dl.d_wo, &sl.d_wo);
            add(&mut dl.d_rms_ffn, &sl.d_rms_ffn);
            add(&mut dl.d_wgate, &sl.d_wgate); add(&mut dl.d_wup, &sl.d_wup);
            add(&mut dl.d_wdown, &sl.d_wdown);
        }
        add(&mut self.d_rms_final, &other.d_rms_final);
        add(&mut self.d_output, &other.d_output);
    }

    
    pub fn scale(&mut self, j: f32) {
        let dr = |v: &mut [f32]| { for g in v.iter_mut() { *g *= j; } };
        dr(&mut self.d_token_embed);
        dr(&mut self.d_pos_embed);
        for l in &mut self.layers {
            dr(&mut l.d_rms_attn); dr(&mut l.d_wq); dr(&mut l.d_wk);
            dr(&mut l.d_wv); dr(&mut l.d_wo); dr(&mut l.d_rms_ffn);
            dr(&mut l.d_wgate); dr(&mut l.d_wup); dr(&mut l.d_wdown);
        }
        dr(&mut self.d_rms_final);
        dr(&mut self.d_output);
    }
}






struct Acs {
    
    x: Vec<f32>,
    
    layer_acts: Vec<Aat>,
    
    x_final_norm: Vec<f32>,
    
    logits: Vec<f32>,
}

struct Aat {
    
    x_in: Vec<f32>,       
    x_norm_attn: Vec<f32>,
    q: Vec<f32>,          
    k: Vec<f32>,          
    v: Vec<f32>,          
    attn_weights: Vec<Vec<f32>>,  
    attn_out: Vec<f32>,   
    exc: Vec<f32>,   
    
    x_mid: Vec<f32>,      
    x_norm_ffn: Vec<f32>, 
    gate_pre: Vec<f32>,   
    gate_act: Vec<f32>,   
    up: Vec<f32>,         
    gated: Vec<f32>,      
    bbr: Vec<f32>,    
}






#[allow(dead_code)]
fn tk(out: &mut [f32], w: &[f32], x: &[f32], cols: usize, rows: usize) {
    for r in 0..rows {
        let mut sum = 0.0f32;
        let base = r * cols;
        for c in 0..cols {
            sum += w[base + c] * x[c];
        }
        out[r] = sum;
    }
}


#[allow(dead_code)]
fn aox(out: &mut [f32], x: &[f32], tv: &[f32]) -> f32 {
    let ae = x.len();
    let mut ss = 0.0f32;
    for &v in x { ss += v * v; }
    let aeg = apq(ss / ae as f32 + HT_);
    let ki = 1.0 / aeg;
    for i in 0..ae {
        out[i] = x[i] * ki * tv[i];
    }
    ki 
}

fn deq(data: &mut [f32]) {
    if data.is_empty() { return; }
    let max = data.iter().copied().fold(f32::NEG_INFINITY, f32::max);
    let mut sum = 0.0f32;
    for v in data.iter_mut() {
        *v = fhf(*v - max);
        sum += *v;
    }
    if sum > 0.0 {
        let ki = 1.0 / sum;
        for v in data.iter_mut() { *v *= ki; }
    }
}

fn fhf(x: f32) -> f32 {
    if x > 88.0 { return f32::MAX; }
    if x < -88.0 { return 0.0; }
    let a = (1 << 23) as f32 / core::f32::consts::LN_2;
    let b = (1 << 23) as f32 * (127.0 - 0.04368);
    let bits = ((a * x + b) as i32).max(0) as u32;
    f32::from_bits(bits)
}

pub fn apq(x: f32) -> f32 {
    if x <= 0.0 { return 0.0; }
    let bits = x.to_bits();
    let uc = f32::from_bits((bits >> 1) + 0x1FBD_1DF5);
    (uc + x / uc) * 0.5
}

fn osu(x: f32) -> f32 {
    let sig = 1.0 / (1.0 + fhf(-x));
    x * sig
}

fn osv(x: f32) -> f32 {
    let sig = 1.0 / (1.0 + fhf(-x));
    sig + x * sig * (1.0 - sig)
}










pub fn eng(model: &TransformerWeights, tokens: &[u8]) -> (f32, ModelGrads) {
    let uj = tokens.len().min(super::model::DF_);
    if uj < 2 {
        return (f32::MAX, ModelGrads::new());
    }

    
    let mut hen: Vec<Acs> = Vec::with_capacity(uj);
    
    let mut efg: Vec<Vec<Vec<f32>>> = vec![Vec::new(); BB_]; 
    let mut efh: Vec<Vec<Vec<f32>>> = vec![Vec::new(); BB_];

    for t in 0..uj {
        let asl = tokens[t] as usize;
        let pos = t;

        
        let mut x = vec![0.0f32; E_];
        for i in 0..E_ {
            x[i] = model.token_embed[asl * E_ + i] + model.pos_embed[pos * E_ + i];
        }

        let mut ijn = Vec::with_capacity(BB_);

        for l in 0..BB_ {
            let bj = &model.layers[l];
            let x_in = x.clone();

            
            let mut eej = vec![0.0f32; E_];
            let _ = super::simd::aox(&mut eej, &x_in, &bj.rms_attn);

            
            let mut q = vec![0.0f32; E_];
            let mut k = vec![0.0f32; E_];
            let mut v = vec![0.0f32; E_];
            super::simd::tk(&mut q, &bj.w_q, &eej, E_, E_);
            super::simd::tk(&mut k, &bj.w_k, &eej, E_, E_);
            super::simd::tk(&mut v, &bj.w_v, &eej, E_, E_);

            
            efg[l].push(k.clone());
            efh[l].push(v.clone());

            
            let ake = t + 1;
            let mut attn_out = vec![0.0f32; E_];
            let fql = apq(DA_ as f32);
            let mut hfw = Vec::with_capacity(GE_);

            for h in 0..GE_ {
                let ajw = h * DA_;
                let mut cdo = vec![0.0f32; ake];
                for aa in 0..ake {
                    let mut j = 0.0f32;
                    for d in 0..DA_ {
                        j += q[ajw + d] * efg[l][aa][ajw + d];
                    }
                    cdo[aa] = j / fql;
                }
                deq(&mut cdo);

                for aa in 0..ake {
                    let w = cdo[aa];
                    for d in 0..DA_ {
                        attn_out[ajw + d] += w * efh[l][aa][ajw + d];
                    }
                }
                hfw.push(cdo);
            }

            
            let mut oa = vec![0.0f32; E_];
            super::simd::tk(&mut oa, &bj.w_o, &attn_out, E_, E_);

            
            for i in 0..E_ { x[i] = x_in[i] + oa[i]; }
            let x_mid = x.clone();

            
            let mut x_norm_ffn = vec![0.0f32; E_];
            let _ = super::simd::aox(&mut x_norm_ffn, &x_mid, &bj.rms_ffn);

            
            let mut gate_pre = vec![0.0f32; Z_];
            let mut up = vec![0.0f32; Z_];
            super::simd::tk(&mut gate_pre, &bj.w_gate, &x_norm_ffn, E_, Z_);
            super::simd::tk(&mut up, &bj.w_up, &x_norm_ffn, E_, Z_);

            let mut gate_act = vec![0.0f32; Z_];
            let mut gated = vec![0.0f32; Z_];
            for i in 0..Z_ {
                gate_act[i] = osu(gate_pre[i]);
                gated[i] = gate_act[i] * up[i];
            }

            let mut bbr = vec![0.0f32; E_];
            super::simd::tk(&mut bbr, &bj.w_down, &gated, Z_, E_);

            
            for i in 0..E_ { x[i] = x_mid[i] + bbr[i]; }

            ijn.push(Aat {
                x_in, x_norm_attn: eej, q, k: efg[l][t].clone(), v: efh[l][t].clone(),
                attn_weights: hfw, attn_out, exc: oa,
                x_mid, x_norm_ffn, gate_pre, gate_act, up, gated, bbr,
            });
        }

        
        let mut hcw = vec![0.0f32; E_];
        super::simd::aox(&mut hcw, &x, &model.rms_final);

        
        let mut logits = vec![0.0f32; BI_];
        super::simd::tk(&mut logits, &model.w_output, &hcw, E_, BI_);

        hen.push(Acs {
            x: x.clone(),
            layer_acts: ijn,
            x_final_norm: hcw,
            logits,
        });
    }

    
    let mut aah = 0.0f32;
    let dbn = uj - 1;
    let mut wg = ModelGrads::new();

    
    
    

    
    for t in 0..dbn {
        let target = tokens[t + 1] as usize;
        let eey = &hen[t];

        
        
        let mut dcx = eey.logits.clone();
        deq(&mut dcx);

        
        let noy = dcx[target].max(1e-10);
        aah += -ln_approx(noy);

        let mut ejt = dcx; 
        ejt[target] -= 1.0;  
        
        let scale = 1.0 / dbn as f32;
        for v in ejt.iter_mut() { *v *= scale; }

        
        
        super::simd::ayw(&mut wg.d_output, &ejt, &eey.x_final_norm, E_, BI_);

        
        let mut hqm = vec![0.0f32; E_];
        super::simd::bnm(&mut hqm, &model.w_output, &ejt, E_, BI_);

        
        let mut byo = fhz(&hqm, &eey.x, &model.rms_final, &mut wg.d_rms_final);

        
        for l in (0..BB_).rev() {
            let xu = &eey.layer_acts[l];
            let bj = &model.layers[l];
            let aie = &mut wg.layers[l];

            
            let hqk = byo.clone(); 
            

            
            
            let mut fqj = vec![0.0f32; Z_];
            super::simd::ayw(&mut aie.d_wdown, &hqk, &xu.gated, Z_, E_);
            super::simd::bnm(&mut fqj, &bj.w_down, &hqk, Z_, E_);

            
            let mut fqi = vec![0.0f32; Z_];
            let mut fqo = vec![0.0f32; Z_];
            for i in 0..Z_ {
                
                let lbc = fqj[i] * xu.up[i];
                
                fqo[i] = fqj[i] * xu.gate_act[i];
                
                fqi[i] = lbc * osv(xu.gate_pre[i]);
            }

            
            
            let mut fqr = vec![0.0f32; E_];
            
            super::simd::ayw(&mut aie.d_wgate, &fqi, &xu.x_norm_ffn, E_, Z_);
            super::simd::ayw(&mut aie.d_wup, &fqo, &xu.x_norm_ffn, E_, Z_);
            
            super::simd::bnm(&mut fqr, &bj.w_gate, &fqi, E_, Z_);
            super::simd::cbq(&mut fqr, &bj.w_up, &fqo, E_, Z_);

            
            let lbh = fhz(&fqr, &xu.x_mid, &bj.rms_ffn, &mut aie.d_rms_ffn);

            
            let mut ejv = vec![0.0f32; E_];
            for i in 0..E_ {
                ejv[i] = byo[i] + lbh[i]; 
            }

            
            super::simd::ayw(&mut aie.d_wo, &ejv, &xu.attn_out, E_, E_);
            let mut fqh = vec![0.0f32; E_];
            super::simd::bnm(&mut fqh, &bj.w_o, &ejv, E_, E_);

            
            let fql = apq(DA_ as f32);
            let ake = t + 1;
            let mut fqn = vec![0.0f32; E_];
            let mut fqk = vec![0.0f32; E_];
            let mut fqp = vec![0.0f32; E_];
            for h in 0..GE_ {
                let ajw = h * DA_;
                let hcs = &xu.attn_weights[h];

                let mut fqq = vec![0.0f32; ake];
                for aa in 0..ake {
                    let mut j = 0.0f32;
                    for d in 0..DA_ {
                        j += fqh[ajw + d] * efh[l][aa][ajw + d];
                        if aa == t { fqp[ajw + d] += hcs[aa] * fqh[ajw + d]; }
                    }
                    fqq[aa] = j;
                }

                let dot: f32 = (0..ake).map(|aa| fqq[aa] * hcs[aa]).sum();
                let mut hql = vec![0.0f32; ake];
                for aa in 0..ake {
                    hql[aa] = hcs[aa] * (fqq[aa] - dot);
                }

                for aa in 0..ake {
                    let ds = hql[aa] / fql;
                    for d in 0..DA_ {
                        fqn[ajw + d] += ds * efg[l][aa][ajw + d];
                        if aa == t { fqk[ajw + d] += ds * xu.q[ajw + d]; }
                    }
                }
            }

            
            
            super::simd::ayw(&mut aie.d_wq, &fqn, &xu.x_norm_attn, E_, E_);
            super::simd::ayw(&mut aie.d_wk, &fqk, &xu.x_norm_attn, E_, E_);
            super::simd::ayw(&mut aie.d_wv, &fqp, &xu.x_norm_attn, E_, E_);
            
            let mut ejw = vec![0.0f32; E_];
            super::simd::bnm(&mut ejw, &bj.w_q, &fqn, E_, E_);
            super::simd::cbq(&mut ejw, &bj.w_k, &fqk, E_, E_);
            super::simd::cbq(&mut ejw, &bj.w_v, &fqp, E_, E_);

            
            let lbg = fhz(&ejw, &xu.x_in, &bj.rms_attn, &mut aie.d_rms_attn);

            
            for i in 0..E_ {
                byo[i] = ejv[i] + lbg[i];
            }
        }

        
        let asl = tokens[t] as usize;
        for i in 0..E_ {
            wg.d_token_embed[asl * E_ + i] += byo[i];
            wg.d_pos_embed[t * E_ + i] += byo[i];
        }
    }

    let adh = aah / dbn as f32;
    (adh, wg)
}







fn fhz(d_out: &[f32], x: &[f32], tv: &[f32], d_weight: &mut [f32]) -> Vec<f32> {
    let ae = x.len();
    let mut ss = 0.0f32;
    for &v in x { ss += v * v; }
    let aeg = apq(ss / ae as f32 + HT_);
    let alu = 1.0 / aeg;

    
    for i in 0..ae {
        d_weight[i] += d_out[i] * x[i] * alu;
    }

    
    
    
    let mut fqm = vec![0.0f32; ae];
    for i in 0..ae {
        fqm[i] = d_out[i] * tv[i];
    }

    
    let mut dot = 0.0f32;
    for i in 0..ae {
        dot += x[i] * alu * fqm[i];
    }
    dot /= ae as f32;

    let mut byo = vec![0.0f32; ae];
    for i in 0..ae {
        byo[i] = alu * (fqm[i] - x[i] * alu * dot);
    }
    byo
}





fn ln_approx(x: f32) -> f32 {
    if x <= 0.0 { return -88.0; }
    let bits = x.to_bits();
    let e = ((bits >> 23) & 0xFF) as f32 - 127.0;
    let m = f32::from_bits((bits & 0x007FFFFF) | 0x3F800000);
    (e + (m - 1.0) * 1.4427) * core::f32::consts::LN_2
}
