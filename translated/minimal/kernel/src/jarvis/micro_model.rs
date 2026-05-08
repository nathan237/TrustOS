

































use alloc::vec::Vec;
use alloc::vec;





pub const EI_: usize = 256;    
pub const N_: usize = 64;
pub const AHG_: usize = 2;
pub const WN_: usize = N_ / AHG_; 
pub const CT_: usize = 128;     
pub const GC_: usize = 1;
pub const DS_: usize = 64;
pub const CJV_: f32 = 1e-5;





pub struct Tt {
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

pub struct MicroWeights {
    pub token_embed: Vec<f32>,   
    pub pos_embed: Vec<f32>,     
    pub layers: Vec<Tt>,
    pub rms_final: Vec<f32>,     
    pub w_output: Vec<f32>,      
}

impl MicroWeights {
    pub fn param_count(&self) -> usize {
        EI_ * N_              
        + DS_ * N_          
        + GC_ * (
            N_                        
            + N_ * N_ * 4  
            + N_                      
            + N_ * CT_ * 2     
            + CT_ * N_         
        )
        + N_                          
        + N_ * EI_            
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

    
    pub fn byt(data: &[f32]) -> Option<Self> {
        let mut pos = 0;
        let token_embed = yd(data, &mut pos, EI_ * N_)?;
        let pos_embed = yd(data, &mut pos, DS_ * N_)?;

        let mut layers = Vec::with_capacity(GC_);
        for _ in 0..GC_ {
            layers.push(Tt {
                rms_attn: yd(data, &mut pos, N_)?,
                w_q: yd(data, &mut pos, N_ * N_)?,
                w_k: yd(data, &mut pos, N_ * N_)?,
                w_v: yd(data, &mut pos, N_ * N_)?,
                w_o: yd(data, &mut pos, N_ * N_)?,
                rms_ffn: yd(data, &mut pos, N_)?,
                w_gate: yd(data, &mut pos, N_ * CT_)?,
                w_up: yd(data, &mut pos, N_ * CT_)?,
                w_down: yd(data, &mut pos, CT_ * N_)?,
            });
        }

        let rms_final = yd(data, &mut pos, N_)?;
        let w_output = yd(data, &mut pos, N_ * EI_)?;

        Some(MicroWeights {
            token_embed, pos_embed, layers, rms_final, w_output,
        })
    }

    
    pub fn bns() -> Self {
        let mut seed = 77u64;
        let cxc = 1.0 / (N_ as f32).sqrt_approx();
        let cji = 1.0 / (CT_ as f32).sqrt_approx();

        let mut layers = Vec::with_capacity(GC_);
        for _ in 0..GC_ {
            layers.push(Tt {
                rms_attn: vec![1.0; N_],
                w_q: afm(N_ * N_, cxc, &mut seed),
                w_k: afm(N_ * N_, cxc, &mut seed),
                w_v: afm(N_ * N_, cxc, &mut seed),
                w_o: afm(N_ * N_, cxc, &mut seed),
                rms_ffn: vec![1.0; N_],
                w_gate: afm(N_ * CT_, cji, &mut seed),
                w_up: afm(N_ * CT_, cji, &mut seed),
                w_down: afm(CT_ * N_, cji, &mut seed),
            });
        }

        MicroWeights {
            token_embed: afm(EI_ * N_, cxc, &mut seed),
            pos_embed: afm(DS_ * N_, 0.02, &mut seed),
            layers,
            rms_final: vec![1.0; N_],
            w_output: afm(N_ * EI_, cxc, &mut seed),
        }
    }
}





pub struct MicroEngine {
    
    cache_k: Vec<Vec<f32>>,
    cache_v: Vec<Vec<f32>>,
    cache_len: usize,
    
    buf_x: Vec<f32>,
    buf_xn: Vec<f32>,
    buf_q: Vec<f32>,
    buf_k: Vec<f32>,
    buf_v: Vec<f32>,
    buf_attn: Vec<f32>,
    buf_gate: Vec<f32>,
    buf_up: Vec<f32>,
    buf_logits: Vec<f32>,
    rng: u64,
}

impl MicroEngine {
    pub fn new() -> Self {
        MicroEngine {
            cache_k: (0..GC_).map(|_| Vec::with_capacity(DS_ * N_)).collect(),
            cache_v: (0..GC_).map(|_| Vec::with_capacity(DS_ * N_)).collect(),
            cache_len: 0,
            buf_x: vec![0.0; N_],
            buf_xn: vec![0.0; N_],
            buf_q: vec![0.0; N_],
            buf_k: vec![0.0; N_],
            buf_v: vec![0.0; N_],
            buf_attn: vec![0.0; DS_],
            buf_gate: vec![0.0; CT_],
            buf_up: vec![0.0; CT_],
            buf_logits: vec![0.0; EI_],
            rng: crate::time::yf().wrapping_add(0xBEEF_CAFE),
        }
    }

    
    pub fn generate(&mut self, model: &MicroWeights, nh: &[u8], alx: usize) -> Vec<u8> {
        self.clear_cache();
        let max = alx.min(DS_);
        let mut output = Vec::with_capacity(max);

        for &abm in nh.iter().take(DS_ - 1) {
            self.forward_one(model, abm);
        }

        let mut next = self.sample(0.7, 20, &output);
        for _ in 0..max {
            if next == 0 || next == 3 { break; }
            output.push(next);
            self.forward_one(model, next);
            next = self.sample(0.7, 20, &output);
        }
        output
    }

    
    pub fn hld(&mut self, model: &MicroWeights, input: &[u8]) -> u8 {
        self.clear_cache();
        for &abm in input.iter().take(DS_) {
            self.forward_one(model, abm);
        }
        dhw(&self.buf_logits)
    }

    
    pub fn atj(&mut self, model: &MicroWeights, tokens: &[u8]) -> f32 {
        self.clear_cache();
        let uj = tokens.len().min(DS_);
        if uj < 2 { return f32::MAX; }

        let mut aah = 0.0f32;
        let dbn = uj - 1;

        for t in 0..uj {
            self.forward_one(model, tokens[t]);
            if t < dbn {
                let target = tokens[t + 1] as usize;
                let mut dcx = self.buf_logits.clone();
                deq(&mut dcx);
                let aa = dcx[target].max(1e-10);
                aah += -aa.ln_approx();
            }
        }
        aah / dbn as f32
    }

    fn clear_cache(&mut self) {
        for k in &mut self.cache_k { k.clear(); }
        for v in &mut self.cache_v { v.clear(); }
        self.cache_len = 0;
    }

    fn forward_one(&mut self, model: &MicroWeights, abm: u8) {
        let pos = self.cache_len;
        if pos >= DS_ { return; }

        let asl = abm as usize;
        for i in 0..N_ {
            self.buf_x[i] = model.token_embed[asl * N_ + i]
                           + model.pos_embed[pos * N_ + i];
        }

        for l in 0..GC_ {
            let bj = &model.layers[l];

            
            aox(&mut self.buf_xn, &self.buf_x, &bj.rms_attn);

            
            tk(&mut self.buf_q, &bj.w_q, &self.buf_xn, N_, N_);
            tk(&mut self.buf_k, &bj.w_k, &self.buf_xn, N_, N_);
            tk(&mut self.buf_v, &bj.w_v, &self.buf_xn, N_, N_);

            self.cache_k[l].extend_from_slice(&self.buf_k);
            self.cache_v[l].extend_from_slice(&self.buf_v);

            let ake = pos + 1;
            let mut attn_out = vec![0.0f32; N_];
            let lge = (WN_ as f32).sqrt_approx();

            for h in 0..AHG_ {
                let ajw = h * WN_;
                for t in 0..ake {
                    let mut score = 0.0f32;
                    for d in 0..WN_ {
                        score += self.buf_q[ajw + d] * self.cache_k[l][t * N_ + ajw + d];
                    }
                    self.buf_attn[t] = score / lge;
                }
                deq(&mut self.buf_attn[..ake]);
                for t in 0..ake {
                    let w = self.buf_attn[t];
                    for d in 0..WN_ {
                        attn_out[ajw + d] += w * self.cache_v[l][t * N_ + ajw + d];
                    }
                }
            }

            
            let mut oa = vec![0.0f32; N_];
            tk(&mut oa, &bj.w_o, &attn_out, N_, N_);
            for i in 0..N_ { self.buf_x[i] += oa[i]; }

            
            aox(&mut self.buf_xn, &self.buf_x, &bj.rms_ffn);

            
            tk(&mut self.buf_gate, &bj.w_gate, &self.buf_xn, N_, CT_);
            tk(&mut self.buf_up, &bj.w_up, &self.buf_xn, N_, CT_);
            for i in 0..CT_ {
                let g = self.buf_gate[i];
                let sig = 1.0 / (1.0 + (-g).exp_approx());
                self.buf_gate[i] = g * sig * self.buf_up[i];
            }
            let mut bbr = vec![0.0f32; N_];
            tk(&mut bbr, &bj.w_down, &self.buf_gate, CT_, N_);
            for i in 0..N_ { self.buf_x[i] += bbr[i]; }
        }

        
        aox(&mut self.buf_xn, &self.buf_x, &model.rms_final);
        tk(&mut self.buf_logits, &model.w_output, &self.buf_xn, N_, EI_);
        self.cache_len = pos + 1;
    }

    fn sample(&mut self, temperature: f32, top_k: usize, cpd: &[u8]) -> u8 {
        if temperature <= 0.01 { return dhw(&self.buf_logits); }

        let mut logits = self.buf_logits.clone();
        for l in logits.iter_mut() { *l /= temperature; }

        
        let window = cpd.len().min(16);
        if window > 0 {
            let start = cpd.len() - window;
            for &asl in &cpd[start..] {
                let idx = asl as usize;
                if idx < EI_ {
                    if logits[idx] > 0.0 { logits[idx] /= 1.3; }
                    else { logits[idx] *= 1.3; }
                }
            }
        }

        
        if top_k > 0 && top_k < EI_ {
            let mut gch: Vec<(f32, usize)> = logits.iter().copied()
                .enumerate().map(|(i, v)| (v, i)).collect();
            gch.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(core::cmp::Ordering::Equal));
            let amz = gch[top_k.min(gch.len() - 1)].0;
            for l in logits.iter_mut() { if *l < amz { *l = f32::NEG_INFINITY; } }
        }

        deq(&mut logits);
        let r = self.rand_f32();
        let mut dlv = 0.0f32;
        for (i, &aa) in logits.iter().enumerate() {
            dlv += aa;
            if dlv >= r { return i as u8; }
        }
        (EI_ - 1) as u8
    }

    fn rand_f32(&mut self) -> f32 {
        let mut x = self.rng;
        x ^= x << 13; x ^= x >> 7; x ^= x << 17;
        self.rng = x;
        ((x >> 40) as u32 as f32) / ((1u32 << 24) as f32)
    }
}






pub struct Aal {
    pub heap_ok: bool,
    pub interrupts_ok: bool,
    pub fs_ok: bool,
    pub serial_ok: bool,
    pub full_brain_available: bool,
    pub full_brain_loaded: bool,
}


pub fn mvn() -> Aal {
    Aal {
        heap_ok: kja(),
        interrupts_ok: kje(),
        fs_ok: kix(),
        serial_ok: kjq(),
        full_brain_available: kiz(),
        full_brain_loaded: false, 
    }
}

fn kja() -> bool {
    
    let test: Vec<u8> = vec![42u8; 64];
    test.len() == 64
}

fn kje() -> bool {
    
    let flags: u64;
    unsafe { core::arch::asm!("pushfq; pop {}", out(reg) flags); }
    (flags & (1 << 9)) != 0 
}

fn kix() -> bool {
    crate::ramfs::bh(|fs| fs.exists("/"))
}

fn kjq() -> bool {
    
    let status: u8;
    unsafe { core::arch::asm!("in al, dx", out("al") status, in("dx") 0x3FDu16); }
    status != 0xFF
}


pub fn kiz() -> bool {
    crate::ramfs::bh(|fs| fs.exists("/jarvis/weights.bin"))
}





pub static DXT_: &[&str] = &[
    
    "help: show commands",
    "ls: list files",
    "ps: show processes",
    "free: memory usage",
    "uptime: system uptime",
    "reboot: restart system",
    "shutdown: power off",
    "clear: clear screen",
    
    "heap OK",
    "interrupts enabled",
    "filesystem ready",
    "serial port active",
    "kernel healthy",
    "all checks passed",
    
    "brain loading from fs",
    "brain loaded OK",
    "brain not found",
    "brain save to disk",
    "brain init random",
    "micro sentinel active",
    "full brain connected",
    "full brain offline",
    
    "I am micro-Jarvis",
    "kernel sentinel ready",
    "checking kernel state",
    "validating memory",
    "validating interrupts",
    "filesystem check OK",
    "loading full brain",
    "full brain available",
    "micro mode active",
    "sentinel watching",
    
    "error: heap corrupt",
    "error: interrupt fault",
    "error: fs unavailable",
    "warning: brain large",
    "status: all nominal",
    "status: degraded",
];





fn tk(out: &mut [f32], w: &[f32], x: &[f32], cols: usize, rows: usize) {
    for r in 0..rows {
        let mut sum = 0.0f32;
        let base = r * cols;
        for c in 0..cols { sum += w[base + c] * x[c]; }
        out[r] = sum;
    }
}

fn aox(out: &mut [f32], x: &[f32], tv: &[f32]) {
    let ae = x.len();
    let ss: f32 = x.iter().map(|v| v * v).sum();
    let ki = 1.0 / (ss / ae as f32 + CJV_).sqrt_approx();
    for i in 0..ae { out[i] = x[i] * ki * tv[i]; }
}

fn deq(data: &mut [f32]) {
    if data.is_empty() { return; }
    let max = data.iter().copied().fold(f32::NEG_INFINITY, f32::max);
    for v in data.iter_mut() { *v = (*v - max).exp_approx(); }
    let sum: f32 = data.iter().sum();
    if sum > 0.0 { let ki = 1.0 / sum; for v in data.iter_mut() { *v *= ki; } }
}

fn dhw(data: &[f32]) -> u8 {
    let mut adj = 0;
    for i in 1..data.len() { if data[i] > data[adj] { adj = i; } }
    adj as u8
}

fn afm(len: usize, scale: f32, seed: &mut u64) -> Vec<f32> {
    (0..len).map(|_| hcz(seed) * scale).collect()
}

fn hcz(state: &mut u64) -> f32 {
    let mut x = *state;
    x ^= x << 13; x ^= x >> 7; x ^= x << 17;
    *state = x;
    let bits = (x >> 40) as u32;
    (bits as f32 / (1u32 << 24) as f32) * 2.0 - 1.0
}

fn yd(data: &[f32], pos: &mut usize, count: usize) -> Option<Vec<f32>> {
    if *pos + count > data.len() { return None; }
    let v = data[*pos..*pos + count].to_vec();
    *pos += count;
    Some(v)
}





trait F32Ext {
    fn exp_approx(self) -> f32;
    fn sqrt_approx(self) -> f32;
    fn ln_approx(self) -> f32;
}

impl F32Ext for f32 {
    fn exp_approx(self) -> f32 {
        let x = self.clamp(-88.0, 88.0);
        let a = 12102203.0f32;
        let b = 1065353216.0f32;
        let bits = ((a * x + b) as i32).max(0) as u32;
        f32::from_bits(bits)
    }

    fn sqrt_approx(self) -> f32 {
        if self <= 0.0 { return 0.0; }
        let bits = self.to_bits();
        let uc = f32::from_bits((bits >> 1) + 0x1FBD_1DF5);
        (uc + self / uc) * 0.5
    }

    fn ln_approx(self) -> f32 {
        if self <= 0.0 { return -88.0; }
        let bits = self.to_bits();
        let e = ((bits >> 23) & 0xFF) as f32 - 127.0;
        let m = f32::from_bits((bits & 0x007FFFFF) | 0x3F800000);
        (e + (m - 1.0) * 1.4427) * core::f32::consts::LN_2
    }
}
