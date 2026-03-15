










































use alloc::string::String;
use alloc::format;
use alloc::vec::Vec;
use alloc::vec;
use core::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use spin::Mutex;

use super::compute;






const CXY_: usize = 16;
const CXZ_: usize = 16;

const EHO_: usize = 16;


const DBP_: usize = CXY_ * CXZ_;




const CFG_: usize = 128;



const DIZ_: f32 = 1.0 / 128.0;




























































































pub static CCX_: &[u32] = &[
    
    
    
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

    
    
    
    
    
    
    
    0xBF860000u32.nj(14),  

    
    
    0xD3690006,
    0x00001B01,  

    
    0x020C0506 | (0x25 << 25),

    
    0x020C0082 | (0x12 << 25),  

    
    0xE0702000,
    0x80030600 | (6 << 8) | (2 << 16),  

    
    0xBF8C0070,

    
    0xBF810000,
];










pub static CCW_: &[u32] = &[
    
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
    0xBF860000u32.nj(16),  

    
    0xD3690006,      
    0x00001B01,
    0x020C0506 | (0x25 << 25),  
    0x020C0082 | (0x12 << 25),  

    0xE0702000,      
    0x80030600 | (6 << 8) | (2 << 16),

    0xBF8C0070,      
    0xBF810000,      
];







pub static CCY_: &[u32] = &[
    
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







pub static CCZ_: &[u32] = &[
    
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








pub static CCT_: &[u32] = &[
    
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
    
    Wt,
    
    Ws,
    
    Ym,
    
    Yp,
    
    Add,
}

impl NeuralKernel {
    pub fn j(&self) -> &'static str {
        match self {
            NeuralKernel::Wt => "gemm_int8",
            NeuralKernel::Ws => "gemm_fp32",
            NeuralKernel::Ym => "relu",
            NeuralKernel::Yp => "scale",
            NeuralKernel::Add => "add",
        }
    }

    pub fn dc(&self) -> &'static str {
        match self {
            NeuralKernel::Wt => "INT8 MatMul (V_DOT4_I32_I8, ~17 TOPS)",
            NeuralKernel::Ws => "FP32 MatMul (V_FMA_F32, ~9.75 TFLOPS)",
            NeuralKernel::Ym => "ReLU activation max(x, 0)",
            NeuralKernel::Yp => "Scalar multiply (x * alpha)",
            NeuralKernel::Add => "Element-wise add (C = A + B)",
        }
    }

    pub fn fun(&self) -> &'static [u32] {
        match self {
            NeuralKernel::Wt => CCX_,
            NeuralKernel::Ws => CCW_,
            NeuralKernel::Ym => CCY_,
            NeuralKernel::Yp => CCZ_,
            NeuralKernel::Add => CCT_,
        }
    }

    pub fn jpo(&self) -> u32 {
        match self {
            NeuralKernel::Wt | NeuralKernel::Ws => 15, 
            NeuralKernel::Ym => 4,     
            NeuralKernel::Yp => 5,    
            NeuralKernel::Add => 12,     
        }
    }

    pub fn jvl(&self) -> u32 {
        match self {
            NeuralKernel::Wt | NeuralKernel::Ws => 8, 
            NeuralKernel::Ym | NeuralKernel::Yp => 3,
            NeuralKernel::Add => 4,
        }
    }
}


pub const LP_: &[NeuralKernel] = &[
    NeuralKernel::Wt,
    NeuralKernel::Ws,
    NeuralKernel::Ym,
    NeuralKernel::Yp,
    NeuralKernel::Add,
];







pub fn rpq(q: &[i8], o: &[i8], r: &mut [i32], ef: usize, bo: usize, eh: usize) {
    for a in 0..ef {
        for fb in 0..bo {
            let mut btc = 0i32;
            for ai in 0..eh {
                btc += q[a * eh + ai] as i32 * o[fb * eh + ai] as i32;
            }
            r[a * bo + fb] = btc;
        }
    }
}


pub fn rpp(q: &[f32], o: &[f32], r: &mut [f32], ef: usize, bo: usize, eh: usize) {
    for a in 0..ef {
        for fb in 0..bo {
            let mut btc = 0.0f32;
            for ai in 0..eh {
                btc += q[a * eh + ai] * o[ai * bo + fb]; 
            }
            r[a * bo + fb] = btc;
        }
    }
}


pub fn ngr(f: &mut [f32]) {
    for b in f.el() {
        if *b < 0.0 { *b = 0.0; }
    }
}


pub fn ngs(f: &mut [f32]) {
    for b in f.el() {
        let sig = 1.0 / (1.0 + (-*b).cqh());
        *b = *b * sig;
    }
}


pub fn rpo(f: &mut [f32]) {
    for b in f.el() {
        
        let ajr = *b * *b * *b;
        let ff = 0.7978845608 * (*b + 0.044715 * ajr); 
        let ab = ff.mjt();
        *b = 0.5 * *b * (1.0 + ab);
    }
}



pub fn klm(bd: &mut [f32], b: &[f32], amz: &[f32], cel: f32) {
    let bo = b.len();
    let mut rv = 0.0f32;
    for &p in b {
        rv += p * p;
    }
    rv = 1.0 / (rv / bo as f32 + cel).bfj();
    for a in 0..bo {
        bd[a] = b[a] * rv * amz[a];
    }
}


pub fn kln(f: &mut [f32]) {
    if f.is_empty() { return; }
    
    let mut aki = f[0];
    for &p in f.iter() {
        if p > aki { aki = p; }
    }
    
    let mut sum = 0.0f32;
    for b in f.el() {
        *b = (*b - aki).cqh();
        sum += *b;
    }
    
    if sum > 0.0 {
        let tvw = 1.0 / sum;
        for b in f.el() {
            *b *= tvw;
        }
    }
}


pub fn oyr(f: &[f32]) -> (Vec<i8>, f32) {
    let mut awd = 0.0f32;
    for &p in f {
        let mtg = if p < 0.0 { -p } else { p };
        if mtg > awd { awd = mtg; }
    }
    let bv = if awd > 0.0 { awd / 127.0 } else { 1.0 };
    let hom = 1.0 / bv;
    let fm: Vec<i8> = f.iter().map(|&p| {
        let fm = (p * hom) as i32;
        fm.am(-128).v(127) as i8
    }).collect();
    (fm, bv)
}


pub fn rvv(f: &[i32], wdj: f32, wdk: f32) -> Vec<f32> {
    let rmp = wdj * wdk;
    f.iter().map(|&p| p as f32 * rmp).collect()
}





trait Apb {
    fn cqh(self) -> f32;
    fn mjt(self) -> f32;
    fn bfj(self) -> f32;
}

impl Apb for f32 {
    
    fn cqh(self) -> f32 {
        if self > 88.0 { return f32::O; }
        if self < -88.0 { return 0.0; }
        
        let b = self;
        let q = (1 << 23) as f32 / core::f32::consts::IG_;
        let o = (1 << 23) as f32 * (127.0 - 0.04368); 
        let fs = ((q * b + o) as i32).am(0) as u32;
        f32::bhb(fs)
    }

    
    fn mjt(self) -> f32 {
        let b = self;
        if b > 5.0 { return 1.0; }
        if b < -5.0 { return -1.0; }
        let hy = b * b;
        b * (27.0 + hy) / (27.0 + 9.0 * hy)
    }

    
    fn bfj(self) -> f32 {
        if self <= 0.0 { return 0.0; }
        let fs = self.bsr();
        
        let anj = f32::bhb((fs >> 1) + 0x1FBD_1DF5);
        
        let at = anj;
        (at + self / at) * 0.5
    }
}






struct Bng {
    hlc: u64,
    jzb: u64,
    iej: u64, 
}

static VR_: Mutex<Bng> = Mutex::new(Bng {
    hlc: 0,
    jzb: 0,
    iej: 0,
});

















pub fn kxy(q: &[i8], o: &[i8], ef: usize, bo: usize, eh: usize) -> Vec<i32> {
    
    let mut r = vec![0i32; ef * bo];
    rpq(q, o, &mut r, ef, bo, eh);

    let mut g = VR_.lock();
    g.hlc += 1;
    g.iej += (ef * bo * eh) as u64;

    r
}


pub fn dhk(q: &[f32], o: &[f32], ef: usize, bo: usize, eh: usize) -> Vec<f32> {
    let mut r = vec![0.0f32; ef * bo];
    rpp(q, o, &mut r, ef, bo, eh);

    let mut g = VR_.lock();
    g.hlc += 1;
    g.iej += (ef * bo * eh) as u64;

    r
}


















pub fn xlu(
    input: &[f32],      
    biw: &[f32],        
    biu: &[f32],
    bpg: &[f32],
    biv: &[f32],
    bit: &[f32],     
    bpf: &[f32],       
    bpe: &[f32],     
    vzp: &[f32],
    vzq: &[f32],
    anz: usize,
    aub: usize,
    eoj: usize,
    urc: usize,
) -> Vec<f32> {
    let rsy = aub / urc;

    
    let mut dtp = vec![0.0f32; anz * aub];
    for e in 0..anz {
        let l = e * aub;
        klm(
            &mut dtp[l..l + aub],
            &input[l..l + aub],
            vzp,
            1e-5,
        );
    }

    
    let fm = dhk(&dtp, biw, anz, aub, aub);
    let eh = dhk(&dtp, biu, anz, aub, aub);
    let p = dhk(&dtp, bpg, anz, aub, aub);

    
    
    let mut ohm = vec![0.0f32; aub * anz];
    for a in 0..anz {
        for fb in 0..aub {
            ohm[fb * anz + a] = eh[a * aub + fb];
        }
    }
    let mut eyd = dhk(&fm, &ohm, anz, anz, aub);
    
    let bv = 1.0 / (rsy as f32).bfj();
    for e in eyd.el() { *e *= bv; }
    
    for br in 0..anz {
        let l = br * anz;
        kln(&mut eyd[l..l + anz]);
    }
    
    let qky = dhk(&eyd, &p, anz, aub, anz);

    
    let con = dhk(&qky, biv, anz, aub, aub);
    let mut hidden = vec![0.0f32; anz * aub];
    for a in 0..hidden.len() {
        hidden[a] = input[a] + con[a];
    }

    
    let mut lou = vec![0.0f32; anz * aub];
    for e in 0..anz {
        let l = e * aub;
        klm(
            &mut lou[l..l + aub],
            &hidden[l..l + aub],
            vzq,
            1e-5,
        );
    }

    
    let mut iwh = dhk(&lou, bit, anz, eoj, aub);
    let bln = dhk(&lou, bpf, anz, eoj, aub);
    ngs(&mut iwh);
    
    for a in 0..iwh.len() {
        iwh[a] *= bln[a];
    }

    
    let cxv = dhk(&iwh, bpe, anz, aub, eoj);
    let mut an = vec![0.0f32; anz * aub];
    for a in 0..an.len() {
        an[a] = hidden[a] + cxv[a];
    }

    an
}







pub fn eyj() -> (u32, u32) {
    let mut afu = 0u32;
    let mut ace = 0u32;

    
    crate::serial_println!("[NEURAL] Test 1: INT8 GEMM 4×4 × 4×4");
    {
        let ef = 4; let bo = 4; let eh = 4;
        
        let q: Vec<i8> = vec![
            1, 0, 0, 0,
            0, 1, 0, 0,
            0, 0, 1, 0,
            0, 0, 0, 1,
        ];
        
        let o: Vec<i8> = vec![
            2, 0, 0, 0,   
            0, 2, 0, 0,   
            0, 0, 2, 0,   
            0, 0, 0, 2,   
        ];
        let r = kxy(&q, &o, ef, bo, eh);
        
        let qy = [2, 0, 0, 0,  0, 2, 0, 0,  0, 0, 2, 0,  0, 0, 0, 2];
        if r == qy { afu += 1; } else {
            crate::serial_println!("[NEURAL]   FAIL: got {:?}", &r[..16]);
            ace += 1;
        }
    }

    
    crate::serial_println!("[NEURAL] Test 2: FP32 GEMM 2×3 × 3×2");
    {
        let q: Vec<f32> = vec![1.0, 2.0, 3.0,  4.0, 5.0, 6.0];
        let o: Vec<f32> = vec![7.0, 8.0,  9.0, 10.0,  11.0, 12.0];
        let r = dhk(&q, &o, 2, 2, 3);
        
        
        
        
        let bq = (r[0] - 58.0).gp() < 0.01
              && (r[1] - 64.0).gp() < 0.01
              && (r[2] - 139.0).gp() < 0.01
              && (r[3] - 154.0).gp() < 0.01;
        if bq { afu += 1; } else {
            crate::serial_println!("[NEURAL]   FAIL: got {:?}", &r);
            ace += 1;
        }
    }

    
    crate::serial_println!("[NEURAL] Test 3: ReLU");
    {
        let mut f = vec![-3.0f32, -1.0, 0.0, 1.5, 4.0, -0.001];
        ngr(&mut f);
        let bq = f[0] == 0.0 && f[1] == 0.0 && f[2] == 0.0
              && f[3] == 1.5 && f[4] == 4.0 && f[5] == 0.0;
        if bq { afu += 1; } else {
            crate::serial_println!("[NEURAL]   FAIL: got {:?}", &f);
            ace += 1;
        }
    }

    
    crate::serial_println!("[NEURAL] Test 4: Softmax");
    {
        let mut f = vec![1.0f32, 2.0, 3.0, 4.0];
        kln(&mut f);
        let sum: f32 = f.iter().sum();
        let bq = (sum - 1.0).gp() < 0.01
              && f[3] > f[2]
              && f[2] > f[1]
              && f[1] > f[0];
        if bq { afu += 1; } else {
            crate::serial_println!("[NEURAL]   FAIL: sum={}, data={:?}", sum, &f);
            ace += 1;
        }
    }

    
    crate::serial_println!("[NEURAL] Test 5: RMSNorm");
    {
        let b = vec![1.0f32, 2.0, 3.0, 4.0];
        let d = vec![1.0f32; 4];
        let mut bd = vec![0.0f32; 4];
        klm(&mut bd, &b, &d, 1e-5);
        
        
        let bfd = (30.0f32 / 4.0).bfj();
        let bq = (bd[0] - 1.0 / bfd).gp() < 0.05
              && (bd[3] - 4.0 / bfd).gp() < 0.05;
        if bq { afu += 1; } else {
            crate::serial_println!("[NEURAL]   FAIL: out={:?}", &bd);
            ace += 1;
        }
    }

    
    crate::serial_println!("[NEURAL] Test 6: Quant/Dequant round-trip");
    {
        let qeg = vec![1.0f32, 0.0, 0.0, 1.0]; 
        let qmc = vec![3.0f32, 0.0, 0.0, 3.0]; 
        let (qei, mte) = oyr(&qeg);
        let (qme, mxf) = oyr(&qmc);
        let qve = kxy(&qei, &qme, 2, 2, 2);
        let hcc = rvv(&qve, mte, mxf);
        
        let bq = (hcc[0] - 3.0).gp() < 0.5
              && (hcc[3] - 3.0).gp() < 0.5
              && hcc[1].gp() < 0.5
              && hcc[2].gp() < 0.5;
        if bq { afu += 1; } else {
            crate::serial_println!("[NEURAL]   FAIL: c_fp={:?} (scales: a={}, b={})", &hcc, mte, mxf);
            ace += 1;
        }
    }

    
    crate::serial_println!("[NEURAL] Test 7: SiLU activation");
    {
        let mut f = vec![0.0f32, 1.0, -1.0, 5.0];
        ngs(&mut f);
        
        let bq = f[0].gp() < 0.01
              && (f[1] - 0.731).gp() < 0.05
              && (f[2] + 0.269).gp() < 0.05
              && (f[3] - 4.966).gp() < 0.1;
        if bq { afu += 1; } else {
            crate::serial_println!("[NEURAL]   FAIL: {:?}", &f);
            ace += 1;
        }
    }

    
    crate::serial_println!("[NEURAL] Test 8: GELU activation");
    {
        let mut f = vec![0.0f32, 1.0, -1.0, 2.0];
        rpo(&mut f);
        
        let bq = f[0].gp() < 0.01
              && (f[1] - 0.841).gp() < 0.05
              && (f[2] + 0.159).gp() < 0.05
              && (f[3] - 1.955).gp() < 0.1;
        if bq { afu += 1; } else {
            crate::serial_println!("[NEURAL]   FAIL: {:?}", &f);
            ace += 1;
        }
    }

    
    crate::serial_println!("[NEURAL] Test 9: GPU kernel enumeration");
    {
        let bq = LP_.len() == 5
              && LP_[0].fun().len() > 5
              && LP_[1].fun().len() > 5;
        if bq { afu += 1; } else {
            ace += 1;
        }
    }

    
    crate::serial_println!("[NEURAL] Test 10: Math approximations");
    {
        let ebb = 1.0f32.cqh();       
        let aax = 1.0f32.mjt();      
        let pew = 4.0f32.bfj();      
        let bq = (ebb - 2.718).gp() < 0.2
              && (aax - 0.762).gp() < 0.05
              && (pew - 2.0).gp() < 0.05;
        if bq { afu += 1; } else {
            crate::serial_println!("[NEURAL]   FAIL: exp(1)={}, tanh(1)={}, sqrt(4)={}", ebb, aax, pew);
            ace += 1;
        }
    }

    (afu, ace)
}






pub fn awz() -> String {
    let g = VR_.lock();
    format!("Neural: {} GEMM, {} activations, {} MACs total",
        g.hlc, g.jzb, g.iej)
}


pub fn zl() -> Vec<String> {
    let mut ak = Vec::new();
    let g = VR_.lock();

    ak.push(String::from("╔══════════════════════════════════════════════════╗"));
    ak.push(String::from("║  Neural Compute — GEMM + Ops for LLM Inference  ║"));
    ak.push(String::from("╠══════════════════════════════════════════════════╣"));
    ak.push(format!("║ GEMM ops:       {}                              ║", g.hlc));
    ak.push(format!("║ Activation ops: {}                              ║", g.jzb));
    ak.push(format!("║ Total MACs:     {}                          ║", g.iej));
    ak.push(format!("║ GPU ready:      {}                          ║", compute::uc()));
    ak.push(String::from("╠══════════════════════════════════════════════════╣"));
    ak.push(String::from("║ GPU Kernels:                                     ║"));
    for eh in LP_ {
        ak.push(format!("║  {:12} {} ({} insns)            ║",
            eh.j(), eh.dc(), eh.fun().len()));
    }
    ak.push(String::from("╠══════════════════════════════════════════════════╣"));
    ak.push(String::from("║ CPU Ops: gemm_int8, gemm_fp32, relu, silu, gelu ║"));
    ak.push(String::from("║          softmax, rmsnorm, quantize, dequantize  ║"));
    ak.push(String::from("║ Transformer: full LLaMA-style layer (CPU)        ║"));
    ak.push(String::from("╚══════════════════════════════════════════════════╝"));

    ak
}



pub fn qow(tp: usize) -> f64 {
    let tp = tp.v(CFG_);
    let q: Vec<i8> = vec![1i8; tp * tp];
    let o: Vec<i8> = vec![1i8; tp * tp];

    let ay = crate::time::ave();

    let bbu = 4u32;
    for _ in 0..bbu {
        let _ = kxy(&q, &o, tp, tp, tp);
    }

    let ci = crate::time::ave();
    let oz = ci.ao(ay).am(1);

    let xkm = 2 * tp * tp * tp * bbu as usize; 
    let kzi = xkm as f64 / (oz as f64 * 1_000.0); 
    kzi
}


trait Bxo {
    fn gp(self) -> f32;
}
impl Bxo for f32 {
    fn gp(self) -> f32 {
        if self < 0.0 { -self } else { self }
    }
}
