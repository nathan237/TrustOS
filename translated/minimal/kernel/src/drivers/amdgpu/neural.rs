










































use alloc::string::String;
use alloc::format;
use alloc::vec::Vec;
use alloc::vec;
use core::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use spin::Mutex;

use super::compute;






const DBQ_: usize = 16;
const DBR_: usize = 16;

const ELF_: usize = 16;


const DFK_: usize = DBQ_ * DBR_;




const CIP_: usize = 128;



const DMO_: f32 = 1.0 / 128.0;




























































































pub static CGG_: &[u32] = &[
    
    
    
    0x02020084 | (0x10 << 25),
    
    
    0x0204008F | (0x1C << 25),

    
    
    
    0x7E060280,

    
    
    0x7E0E0280,

    
    
    
    
    
    0xD3690006,
    0x00001D01,  

    
    
    0x020C0F06 | (0x25 << 25),  

    
    0xE0502000,
    0x80040600 | (6 << 8),  

    
    
    0xD3690006,
    0x00001D02,  

    
    0x020C0F06 | (0x25 << 25),

    
    0xE0502000,
    0x80050600 | (6 << 8) | (1 << 16),  

    
    0xBF8C0070,

    
    
    
    
    
    0xCC650003,
    0x040D0504,  

    
    
    0x020E0884 | (0x25 << 25),  

    
    
    
    0xD4C20000,
    0x00001D07,  

    
    
    
    
    
    
    
    0xBF860000u32.wrapping_sub(14),  

    
    
    0xD3690006,
    0x00001B01,  

    
    0x020C0506 | (0x25 << 25),

    
    0x020C0082 | (0x12 << 25),  

    
    0xE0702000,
    0x80030600 | (6 << 8) | (2 << 16),  

    
    0xBF8C0070,

    
    0xBF810000,
];










pub static CGF_: &[u32] = &[
    
    0x02020084 | (0x10 << 25),  
    0x0204008F | (0x1C << 25),  

    
    0x7E060280,  

    
    0x7E0E0280,  

    
    
    0xD3690006,      
    0x00001D01,
    0x020C0F06 | (0x25 << 25),  
    0x020C0082 | (0x12 << 25),  

    0xE0502000,      
    0x80040600 | (6 << 8),

    
    
    0xD3690006,      
    0x00001B07,      
    0x020C0506 | (0x25 << 25),  
    0x020C0082 | (0x12 << 25),  

    0xE0502000,      
    0x80050600 | (6 << 8) | (1 << 16),

    0xBF8C0070,      

    
    
    
    
    
    0xD4260003,      
    0x040D0504,      

    
    0x020E0881 | (0x25 << 25),  

    
    0xD4C20000,      
    0x00001D07,
    0xBF860000u32.wrapping_sub(16),  

    
    0xD3690006,      
    0x00001B01,
    0x020C0506 | (0x25 << 25),  
    0x020C0082 | (0x12 << 25),  

    0xE0702000,      
    0x80030600 | (6 << 8) | (2 << 16),

    0xBF8C0070,      
    0xBF810000,      
];







pub static CGH_: &[u32] = &[
    
    0x02020082 | (0x12 << 25),
    
    0xE0502000,
    0x80020100 | (1 << 8),
    
    0xBF8C0070,
    
    
    
    0x02040080 | (0x10 << 25),  
    
    0xE0702000,
    0x80020100 | (1 << 8),
    
    0xBF8C0070,
    
    0xBF810000,
];







pub static CGI_: &[u32] = &[
    
    0x02020082 | (0x12 << 25),
    
    0xE0502000,
    0x80020100 | (1 << 8),
    
    0xBF8C0070,
    
    
    0x02040802 | (0x08 << 25),  
    
    0xE0702000,
    0x80020100 | (1 << 8),
    0xBF8C0070,  
    0xBF810000,  
];








pub static CGC_: &[u32] = &[
    
    0x02020082 | (0x12 << 25),
    
    0xE0502000,
    0x80020100 | (1 << 8),
    
    0xE0502000,
    0x80030100 | (1 << 8) | (1 << 16),
    
    0xBF8C0070,
    
    
    0x02040702 | (0x03 << 25),
    
    0xE0702000,
    0x80020100 | (1 << 8) | (2 << 16),
    0xBF8C0070,
    0xBF810000,
];






#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NeuralKernel {
    
    GemmInt8,
    
    GemmFp32,
    
    ReLU,
    
    Scale,
    
    Add,
}

impl NeuralKernel {
    pub fn name(&self) -> &'static str {
        match self {
            NeuralKernel::GemmInt8 => "gemm_int8",
            NeuralKernel::GemmFp32 => "gemm_fp32",
            NeuralKernel::ReLU => "relu",
            NeuralKernel::Scale => "scale",
            NeuralKernel::Add => "add",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            NeuralKernel::GemmInt8 => "INT8 MatMul (V_DOT4_I32_I8, ~17 TOPS)",
            NeuralKernel::GemmFp32 => "FP32 MatMul (V_FMA_F32, ~9.75 TFLOPS)",
            NeuralKernel::ReLU => "ReLU activation max(x, 0)",
            NeuralKernel::Scale => "Scalar multiply (x * alpha)",
            NeuralKernel::Add => "Element-wise add (C = A + B)",
        }
    }

    pub fn shader_code(&self) -> &'static [u32] {
        match self {
            NeuralKernel::GemmInt8 => CGG_,
            NeuralKernel::GemmFp32 => CGF_,
            NeuralKernel::ReLU => CGH_,
            NeuralKernel::Scale => CGI_,
            NeuralKernel::Add => CGC_,
        }
    }

    pub fn sgpr_count(&self) -> u32 {
        match self {
            NeuralKernel::GemmInt8 | NeuralKernel::GemmFp32 => 15, 
            NeuralKernel::ReLU => 4,     
            NeuralKernel::Scale => 5,    
            NeuralKernel::Add => 12,     
        }
    }

    pub fn vgpr_count(&self) -> u32 {
        match self {
            NeuralKernel::GemmInt8 | NeuralKernel::GemmFp32 => 8, 
            NeuralKernel::ReLU | NeuralKernel::Scale => 3,
            NeuralKernel::Add => 4,
        }
    }
}


pub const ML_: &[NeuralKernel] = &[
    NeuralKernel::GemmInt8,
    NeuralKernel::GemmFp32,
    NeuralKernel::ReLU,
    NeuralKernel::Scale,
    NeuralKernel::Add,
];







pub fn kyn(a: &[i8], b: &[i8], c: &mut [i32], m: usize, ae: usize, k: usize) {
    for i in 0..m {
        for ay in 0..ae {
            let mut aku = 0i32;
            for aa in 0..k {
                aku += a[i * k + aa] as i32 * b[ay * k + aa] as i32;
            }
            c[i * ae + ay] = aku;
        }
    }
}


pub fn kym(a: &[f32], b: &[f32], c: &mut [f32], m: usize, ae: usize, k: usize) {
    for i in 0..m {
        for ay in 0..ae {
            let mut aku = 0.0f32;
            for aa in 0..k {
                aku += a[i * k + aa] * b[aa * ae + ay]; 
            }
            c[i * ae + ay] = aku;
        }
    }
}


pub fn hoj(data: &mut [f32]) {
    for x in data.iter_mut() {
        if *x < 0.0 { *x = 0.0; }
    }
}


pub fn hok(data: &mut [f32]) {
    for x in data.iter_mut() {
        let sig = 1.0 / (1.0 + (-*x).exp_approx());
        *x = *x * sig;
    }
}


pub fn kyl(data: &mut [f32]) {
    for x in data.iter_mut() {
        
        let x3 = *x * *x * *x;
        let inner = 0.7978845608 * (*x + 0.044715 * x3); 
        let t = inner.tanh_approx();
        *x = 0.5 * *x * (1.0 + t);
    }
}



pub fn fow(out: &mut [f32], x: &[f32], tv: &[f32], eps: f32) {
    let ae = x.len();
    let mut ss = 0.0f32;
    for &v in x {
        ss += v * v;
    }
    ss = 1.0 / (ss / ae as f32 + eps).sqrt_approx();
    for i in 0..ae {
        out[i] = x[i] * ss * tv[i];
    }
}


pub fn fox(data: &mut [f32]) {
    if data.is_empty() { return; }
    
    let mut sh = data[0];
    for &v in data.iter() {
        if v > sh { sh = v; }
    }
    
    let mut sum = 0.0f32;
    for x in data.iter_mut() {
        *x = (*x - sh).exp_approx();
        sum += *x;
    }
    
    if sum > 0.0 {
        let mri = 1.0 / sum;
        for x in data.iter_mut() {
            *x *= mri;
        }
    }
}


pub fn ixf(data: &[f32]) -> (Vec<i8>, f32) {
    let mut yw = 0.0f32;
    for &v in data {
        let hdq = if v < 0.0 { -v } else { v };
        if hdq > yw { yw = hdq; }
    }
    let scale = if yw > 0.0 { yw / 127.0 } else { 1.0 };
    let dsh = 1.0 / scale;
    let q: Vec<i8> = data.iter().map(|&v| {
        let q = (v * dsh) as i32;
        q.max(-128).min(127) as i8
    }).collect();
    (q, scale)
}


pub fn ldl(data: &[i32], scale_a: f32, scale_b: f32) -> Vec<f32> {
    let kvz = scale_a * scale_b;
    data.iter().map(|&v| v as f32 * kvz).collect()
}





trait Ra {
    fn exp_approx(self) -> f32;
    fn tanh_approx(self) -> f32;
    fn sqrt_approx(self) -> f32;
}

impl Ra for f32 {
    
    fn exp_approx(self) -> f32 {
        if self > 88.0 { return f32::MAX; }
        if self < -88.0 { return 0.0; }
        
        let x = self;
        let a = (1 << 23) as f32 / core::f32::consts::LN_2;
        let b = (1 << 23) as f32 * (127.0 - 0.04368); 
        let bits = ((a * x + b) as i32).max(0) as u32;
        f32::from_bits(bits)
    }

    
    fn tanh_approx(self) -> f32 {
        let x = self;
        if x > 5.0 { return 1.0; }
        if x < -5.0 { return -1.0; }
        let x2 = x * x;
        x * (27.0 + x2) / (27.0 + 9.0 * x2)
    }

    
    fn sqrt_approx(self) -> f32 {
        if self <= 0.0 { return 0.0; }
        let bits = self.to_bits();
        
        let uc = f32::from_bits((bits >> 1) + 0x1FBD_1DF5);
        
        let g = uc;
        (g + self / g) * 0.5
    }
}






struct Abs {
    gemm_count: u64,
    activation_count: u64,
    total_macs: u64, 
}

static XA_: Mutex<Abs> = Mutex::new(Abs {
    gemm_count: 0,
    activation_count: 0,
    total_macs: 0,
});

















pub fn fyh(a: &[i8], b: &[i8], m: usize, ae: usize, k: usize) -> Vec<i32> {
    
    let mut c = vec![0i32; m * ae];
    kyn(a, b, &mut c, m, ae, k);

    let mut state = XA_.lock();
    state.gemm_count += 1;
    state.total_macs += (m * ae * k) as u64;

    c
}


pub fn bgn(a: &[f32], b: &[f32], m: usize, ae: usize, k: usize) -> Vec<f32> {
    let mut c = vec![0.0f32; m * ae];
    kym(a, b, &mut c, m, ae, k);

    let mut state = XA_.lock();
    state.gemm_count += 1;
    state.total_macs += (m * ae * k) as u64;

    c
}


















pub fn pnd(
    input: &[f32],      
    w_q: &[f32],        
    w_k: &[f32],
    w_v: &[f32],
    w_o: &[f32],
    w_gate: &[f32],     
    w_up: &[f32],       
    w_down: &[f32],     
    rms_weight_attn: &[f32],
    rms_weight_ffn: &[f32],
    uj: usize,
    d_model: usize,
    bym: usize,
    n_heads: usize,
) -> Vec<f32> {
    let lbd = d_model / n_heads;

    
    let mut bnv = vec![0.0f32; uj * d_model];
    for j in 0..uj {
        let offset = j * d_model;
        fow(
            &mut bnv[offset..offset + d_model],
            &input[offset..offset + d_model],
            rms_weight_attn,
            1e-5,
        );
    }

    
    let q = bgn(&bnv, w_q, uj, d_model, d_model);
    let k = bgn(&bnv, w_k, uj, d_model, d_model);
    let v = bgn(&bnv, w_v, uj, d_model, d_model);

    
    
    let mut iiv = vec![0.0f32; d_model * uj];
    for i in 0..uj {
        for ay in 0..d_model {
            iiv[ay * uj + i] = k[i * d_model + ay];
        }
    }
    let mut cdo = bgn(&q, &iiv, uj, uj, d_model);
    
    let scale = 1.0 / (lbd as f32).sqrt_approx();
    for j in cdo.iter_mut() { *j *= scale; }
    
    for row in 0..uj {
        let offset = row * uj;
        fox(&mut cdo[offset..offset + uj]);
    }
    
    let jxz = bgn(&cdo, &v, uj, d_model, uj);

    
    let attn_out = bgn(&jxz, w_o, uj, d_model, d_model);
    let mut hidden = vec![0.0f32; uj * d_model];
    for i in 0..hidden.len() {
        hidden[i] = input[i] + attn_out[i];
    }

    
    let mut gjp = vec![0.0f32; uj * d_model];
    for j in 0..uj {
        let offset = j * d_model;
        fow(
            &mut gjp[offset..offset + d_model],
            &hidden[offset..offset + d_model],
            rms_weight_ffn,
            1e-5,
        );
    }

    
    let mut enu = bgn(&gjp, w_gate, uj, bym, d_model);
    let up = bgn(&gjp, w_up, uj, bym, d_model);
    hok(&mut enu);
    
    for i in 0..enu.len() {
        enu[i] *= up[i];
    }

    
    let bbr = bgn(&enu, w_down, uj, d_model, bym);
    let mut output = vec![0.0f32; uj * d_model];
    for i in 0..output.len() {
        output[i] = hidden[i] + bbr[i];
    }

    output
}







pub fn cdp() -> (u32, u32) {
    let mut gd = 0u32;
    let mut gv = 0u32;

    
    crate::serial_println!("[NEURAL] Test 1: INT8 GEMM 4×4 × 4×4");
    {
        let m = 4; let ae = 4; let k = 4;
        
        let a: Vec<i8> = vec![
            1, 0, 0, 0,
            0, 1, 0, 0,
            0, 0, 1, 0,
            0, 0, 0, 1,
        ];
        
        let b: Vec<i8> = vec![
            2, 0, 0, 0,   
            0, 2, 0, 0,   
            0, 0, 2, 0,   
            0, 0, 0, 2,   
        ];
        let c = fyh(&a, &b, m, ae, k);
        
        let expected = [2, 0, 0, 0,  0, 2, 0, 0,  0, 0, 2, 0,  0, 0, 0, 2];
        if c == expected { gd += 1; } else {
            crate::serial_println!("[NEURAL]   FAIL: got {:?}", &c[..16]);
            gv += 1;
        }
    }

    
    crate::serial_println!("[NEURAL] Test 2: FP32 GEMM 2×3 × 3×2");
    {
        let a: Vec<f32> = vec![1.0, 2.0, 3.0,  4.0, 5.0, 6.0];
        let b: Vec<f32> = vec![7.0, 8.0,  9.0, 10.0,  11.0, 12.0];
        let c = bgn(&a, &b, 2, 2, 3);
        
        
        
        
        let ok = (c[0] - 58.0).abs() < 0.01
              && (c[1] - 64.0).abs() < 0.01
              && (c[2] - 139.0).abs() < 0.01
              && (c[3] - 154.0).abs() < 0.01;
        if ok { gd += 1; } else {
            crate::serial_println!("[NEURAL]   FAIL: got {:?}", &c);
            gv += 1;
        }
    }

    
    crate::serial_println!("[NEURAL] Test 3: ReLU");
    {
        let mut data = vec![-3.0f32, -1.0, 0.0, 1.5, 4.0, -0.001];
        hoj(&mut data);
        let ok = data[0] == 0.0 && data[1] == 0.0 && data[2] == 0.0
              && data[3] == 1.5 && data[4] == 4.0 && data[5] == 0.0;
        if ok { gd += 1; } else {
            crate::serial_println!("[NEURAL]   FAIL: got {:?}", &data);
            gv += 1;
        }
    }

    
    crate::serial_println!("[NEURAL] Test 4: Softmax");
    {
        let mut data = vec![1.0f32, 2.0, 3.0, 4.0];
        fox(&mut data);
        let sum: f32 = data.iter().sum();
        let ok = (sum - 1.0).abs() < 0.01
              && data[3] > data[2]
              && data[2] > data[1]
              && data[1] > data[0];
        if ok { gd += 1; } else {
            crate::serial_println!("[NEURAL]   FAIL: sum={}, data={:?}", sum, &data);
            gv += 1;
        }
    }

    
    crate::serial_println!("[NEURAL] Test 5: RMSNorm");
    {
        let x = vec![1.0f32, 2.0, 3.0, 4.0];
        let w = vec![1.0f32; 4];
        let mut out = vec![0.0f32; 4];
        fow(&mut out, &x, &w, 1e-5);
        
        
        let aeg = (30.0f32 / 4.0).sqrt_approx();
        let ok = (out[0] - 1.0 / aeg).abs() < 0.05
              && (out[3] - 4.0 / aeg).abs() < 0.05;
        if ok { gd += 1; } else {
            crate::serial_println!("[NEURAL]   FAIL: out={:?}", &out);
            gv += 1;
        }
    }

    
    crate::serial_println!("[NEURAL] Test 6: Quant/Dequant round-trip");
    {
        let jsy = vec![1.0f32, 0.0, 0.0, 1.0]; 
        let jyx = vec![3.0f32, 0.0, 0.0, 3.0]; 
        let (a_q, a_scale) = ixf(&jsy);
        let (b_q, b_scale) = ixf(&jyx);
        let kgr = fyh(&a_q, &b_q, 2, 2, 2);
        let dkg = ldl(&kgr, a_scale, b_scale);
        
        let ok = (dkg[0] - 3.0).abs() < 0.5
              && (dkg[3] - 3.0).abs() < 0.5
              && dkg[1].abs() < 0.5
              && dkg[2].abs() < 0.5;
        if ok { gd += 1; } else {
            crate::serial_println!("[NEURAL]   FAIL: c_fp={:?} (scales: a={}, b={})", &dkg, a_scale, b_scale);
            gv += 1;
        }
    }

    
    crate::serial_println!("[NEURAL] Test 7: SiLU activation");
    {
        let mut data = vec![0.0f32, 1.0, -1.0, 5.0];
        hok(&mut data);
        
        let ok = data[0].abs() < 0.01
              && (data[1] - 0.731).abs() < 0.05
              && (data[2] + 0.269).abs() < 0.05
              && (data[3] - 4.966).abs() < 0.1;
        if ok { gd += 1; } else {
            crate::serial_println!("[NEURAL]   FAIL: {:?}", &data);
            gv += 1;
        }
    }

    
    crate::serial_println!("[NEURAL] Test 8: GELU activation");
    {
        let mut data = vec![0.0f32, 1.0, -1.0, 2.0];
        kyl(&mut data);
        
        let ok = data[0].abs() < 0.01
              && (data[1] - 0.841).abs() < 0.05
              && (data[2] + 0.159).abs() < 0.05
              && (data[3] - 1.955).abs() < 0.1;
        if ok { gd += 1; } else {
            crate::serial_println!("[NEURAL]   FAIL: {:?}", &data);
            gv += 1;
        }
    }

    
    crate::serial_println!("[NEURAL] Test 9: GPU kernel enumeration");
    {
        let ok = ML_.len() == 5
              && ML_[0].shader_code().len() > 5
              && ML_[1].shader_code().len() > 5;
        if ok { gd += 1; } else {
            gv += 1;
        }
    }

    
    crate::serial_println!("[NEURAL] Test 10: Math approximations");
    {
        let bsc = 1.0f32.exp_approx();       
        let ll = 1.0f32.tanh_approx();      
        let jcc = 4.0f32.sqrt_approx();      
        let ok = (bsc - 2.718).abs() < 0.2
              && (ll - 0.762).abs() < 0.05
              && (jcc - 2.0).abs() < 0.05;
        if ok { gd += 1; } else {
            crate::serial_println!("[NEURAL]   FAIL: exp(1)={}, tanh(1)={}, sqrt(4)={}", bsc, ll, jcc);
            gv += 1;
        }
    }

    (gd, gv)
}






pub fn summary() -> String {
    let state = XA_.lock();
    format!("Neural: {} GEMM, {} activations, {} MACs total",
        state.gemm_count, state.activation_count, state.total_macs)
}


pub fn info_lines() -> Vec<String> {
    let mut lines = Vec::new();
    let state = XA_.lock();

    lines.push(String::from("╔══════════════════════════════════════════════════╗"));
    lines.push(String::from("║  Neural Compute — GEMM + Ops for LLM Inference  ║"));
    lines.push(String::from("╠══════════════════════════════════════════════════╣"));
    lines.push(format!("║ GEMM ops:       {}                              ║", state.gemm_count));
    lines.push(format!("║ Activation ops: {}                              ║", state.activation_count));
    lines.push(format!("║ Total MACs:     {}                          ║", state.total_macs));
    lines.push(format!("║ GPU ready:      {}                          ║", compute::is_ready()));
    lines.push(String::from("╠══════════════════════════════════════════════════╣"));
    lines.push(String::from("║ GPU Kernels:                                     ║"));
    for k in ML_ {
        lines.push(format!("║  {:12} {} ({} insns)            ║",
            k.name(), k.description(), k.shader_code().len()));
    }
    lines.push(String::from("╠══════════════════════════════════════════════════╣"));
    lines.push(String::from("║ CPU Ops: gemm_int8, gemm_fp32, relu, silu, gelu ║"));
    lines.push(String::from("║          softmax, rmsnorm, quantize, dequantize  ║"));
    lines.push(String::from("║ Transformer: full LLaMA-style layer (CPU)        ║"));
    lines.push(String::from("╚══════════════════════════════════════════════════╝"));

    lines
}



pub fn kbj(dim: usize) -> f64 {
    let dim = dim.min(CIP_);
    let a: Vec<i8> = vec![1i8; dim * dim];
    let b: Vec<i8> = vec![1i8; dim * dim];

    let start = crate::time::yf();

    let acd = 4u32;
    for _ in 0..acd {
        let _ = fyh(&a, &b, dim, dim, dim);
    }

    let end = crate::time::yf();
    let elapsed_ms = end.saturating_sub(start).max(1);

    let pmd = 2 * dim * dim * dim * acd as usize; 
    let fzg = pmd as f64 / (elapsed_ms as f64 * 1_000.0); 
    fzg
}


trait Aha {
    fn abs(self) -> f32;
}
impl Aha for f32 {
    fn abs(self) -> f32 {
        if self < 0.0 { -self } else { self }
    }
}
