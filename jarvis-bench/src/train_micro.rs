// Micro-JARVIS Trainer
// Trains the tiny kernel sentinel model (~50K params)
// and exports jarvis_micro.bin for embedding in the kernel.

use std::time::Instant;

// Hyperparameters
const VOCAB: usize = 256;
const D: usize = 64;
const NH: usize = 2;
const DK: usize = D / NH;
const DFF: usize = 128;
const NL: usize = 1;
const MAXSEQ: usize = 64;
const EPS: f32 = 1e-5;

// ═══════════════════════════════════════════════════════════════
// Weight structures
// ═══════════════════════════════════════════════════════════════
struct LayerWeights {
    rms_attn: Vec<f32>, w_q: Vec<f32>, w_k: Vec<f32>, w_v: Vec<f32>, w_o: Vec<f32>,
    rms_ffn: Vec<f32>, w_gate: Vec<f32>, w_up: Vec<f32>, w_down: Vec<f32>,
}

struct Model {
    token_embed: Vec<f32>, pos_embed: Vec<f32>,
    layers: Vec<LayerWeights>,
    rms_final: Vec<f32>, w_output: Vec<f32>,
}

impl Model {
    fn new_random() -> Self {
        let mut seed = 77u64;
        let es = 1.0 / (D as f32).sqrt();
        let fs = 1.0 / (DFF as f32).sqrt();
        let mut layers = Vec::new();
        for _ in 0..NL {
            layers.push(LayerWeights {
                rms_attn: vec![1.0; D],
                w_q: rvec(D*D, es, &mut seed), w_k: rvec(D*D, es, &mut seed),
                w_v: rvec(D*D, es, &mut seed), w_o: rvec(D*D, es, &mut seed),
                rms_ffn: vec![1.0; D],
                w_gate: rvec(D*DFF, fs, &mut seed), w_up: rvec(D*DFF, fs, &mut seed),
                w_down: rvec(DFF*D, fs, &mut seed),
            });
        }
        Model {
            token_embed: rvec(VOCAB*D, es, &mut seed), pos_embed: rvec(MAXSEQ*D, 0.02, &mut seed),
            layers, rms_final: vec![1.0; D], w_output: rvec(D*VOCAB, es, &mut seed),
        }
    }

    fn param_count(&self) -> usize {
        VOCAB*D + MAXSEQ*D + NL*(D + D*D*4 + D + D*DFF*2 + DFF*D) + D + D*VOCAB
    }

    fn serialize(&self) -> Vec<f32> {
        let mut d = Vec::with_capacity(self.param_count());
        d.extend(&self.token_embed); d.extend(&self.pos_embed);
        for l in &self.layers {
            d.extend(&l.rms_attn); d.extend(&l.w_q); d.extend(&l.w_k);
            d.extend(&l.w_v); d.extend(&l.w_o); d.extend(&l.rms_ffn);
            d.extend(&l.w_gate); d.extend(&l.w_up); d.extend(&l.w_down);
        }
        d.extend(&self.rms_final); d.extend(&self.w_output);
        d
    }
}

// ═══════════════════════════════════════════════════════════════
// Gradient structures
// ═══════════════════════════════════════════════════════════════
struct LayerGrads {
    d_rms_attn: Vec<f32>, d_wq: Vec<f32>, d_wk: Vec<f32>, d_wv: Vec<f32>, d_wo: Vec<f32>,
    d_rms_ffn: Vec<f32>, d_wgate: Vec<f32>, d_wup: Vec<f32>, d_wdown: Vec<f32>,
}
impl LayerGrads {
    fn new() -> Self {
        LayerGrads {
            d_rms_attn: vec![0.0; D], d_wq: vec![0.0; D*D], d_wk: vec![0.0; D*D],
            d_wv: vec![0.0; D*D], d_wo: vec![0.0; D*D], d_rms_ffn: vec![0.0; D],
            d_wgate: vec![0.0; D*DFF], d_wup: vec![0.0; D*DFF], d_wdown: vec![0.0; DFF*D],
        }
    }
    fn zero(&mut self) {
        self.d_rms_attn.fill(0.0); self.d_wq.fill(0.0); self.d_wk.fill(0.0);
        self.d_wv.fill(0.0); self.d_wo.fill(0.0); self.d_rms_ffn.fill(0.0);
        self.d_wgate.fill(0.0); self.d_wup.fill(0.0); self.d_wdown.fill(0.0);
    }
}

struct ModelGrads {
    d_te: Vec<f32>, d_pe: Vec<f32>,
    layers: Vec<LayerGrads>,
    d_rf: Vec<f32>, d_out: Vec<f32>,
}
impl ModelGrads {
    fn new() -> Self {
        ModelGrads {
            d_te: vec![0.0; VOCAB*D], d_pe: vec![0.0; MAXSEQ*D],
            layers: (0..NL).map(|_| LayerGrads::new()).collect(),
            d_rf: vec![0.0; D], d_out: vec![0.0; D*VOCAB],
        }
    }
    fn zero(&mut self) {
        self.d_te.fill(0.0); self.d_pe.fill(0.0);
        for l in &mut self.layers { l.zero(); }
        self.d_rf.fill(0.0); self.d_out.fill(0.0);
    }
    fn accumulate(&mut self, o: &ModelGrads) {
        vadd(&mut self.d_te, &o.d_te); vadd(&mut self.d_pe, &o.d_pe);
        for (dl, sl) in self.layers.iter_mut().zip(&o.layers) {
            vadd(&mut dl.d_rms_attn, &sl.d_rms_attn); vadd(&mut dl.d_wq, &sl.d_wq);
            vadd(&mut dl.d_wk, &sl.d_wk); vadd(&mut dl.d_wv, &sl.d_wv);
            vadd(&mut dl.d_wo, &sl.d_wo); vadd(&mut dl.d_rms_ffn, &sl.d_rms_ffn);
            vadd(&mut dl.d_wgate, &sl.d_wgate); vadd(&mut dl.d_wup, &sl.d_wup);
            vadd(&mut dl.d_wdown, &sl.d_wdown);
        }
        vadd(&mut self.d_rf, &o.d_rf); vadd(&mut self.d_out, &o.d_out);
    }
    fn scale(&mut self, s: f32) {
        vscale(&mut self.d_te, s); vscale(&mut self.d_pe, s);
        for l in &mut self.layers {
            vscale(&mut l.d_rms_attn, s); vscale(&mut l.d_wq, s); vscale(&mut l.d_wk, s);
            vscale(&mut l.d_wv, s); vscale(&mut l.d_wo, s); vscale(&mut l.d_rms_ffn, s);
            vscale(&mut l.d_wgate, s); vscale(&mut l.d_wup, s); vscale(&mut l.d_wdown, s);
        }
        vscale(&mut self.d_rf, s); vscale(&mut self.d_out, s);
    }
    fn clip_norm(&mut self, max_norm: f32) {
        let ss = |s: &[f32]| -> f32 { s.iter().map(|g| g*g).sum() };
        let mut total: f32 = ss(&self.d_te) + ss(&self.d_pe) + ss(&self.d_rf) + ss(&self.d_out);
        for l in &self.layers {
            total += ss(&l.d_rms_attn)+ss(&l.d_wq)+ss(&l.d_wk)+ss(&l.d_wv)
                +ss(&l.d_wo)+ss(&l.d_rms_ffn)+ss(&l.d_wgate)+ss(&l.d_wup)+ss(&l.d_wdown);
        }
        let norm = total.sqrt();
        if norm > max_norm && norm > 0.0 {
            self.scale(max_norm / norm);
        }
    }
}

// ═══════════════════════════════════════════════════════════════
// AdamW optimizer
// ═══════════════════════════════════════════════════════════════
struct Adam { m: Vec<f32>, v: Vec<f32>, t: u64, lr: f32 }
impl Adam {
    fn new(n: usize) -> Self { Adam { m: vec![0.0; n], v: vec![0.0; n], t: 0, lr: 0.002 } }
    fn step(&mut self, model: &mut Model, grads: &ModelGrads) {
        self.t += 1;
        let bc1 = 1.0 - 0.9f32.powi(self.t as i32);
        let bc2 = 1.0 - 0.999f32.powi(self.t as i32);
        let lr_t = self.lr / bc1;
        let mut idx = 0;
        self.upd(&mut model.token_embed, &grads.d_te, &mut idx, lr_t, bc2);
        self.upd(&mut model.pos_embed, &grads.d_pe, &mut idx, lr_t, bc2);
        for i in 0..NL {
            let lg = &grads.layers[i]; let lw = &mut model.layers[i];
            self.upd(&mut lw.rms_attn, &lg.d_rms_attn, &mut idx, lr_t, bc2);
            self.upd(&mut lw.w_q, &lg.d_wq, &mut idx, lr_t, bc2);
            self.upd(&mut lw.w_k, &lg.d_wk, &mut idx, lr_t, bc2);
            self.upd(&mut lw.w_v, &lg.d_wv, &mut idx, lr_t, bc2);
            self.upd(&mut lw.w_o, &lg.d_wo, &mut idx, lr_t, bc2);
            self.upd(&mut lw.rms_ffn, &lg.d_rms_ffn, &mut idx, lr_t, bc2);
            self.upd(&mut lw.w_gate, &lg.d_wgate, &mut idx, lr_t, bc2);
            self.upd(&mut lw.w_up, &lg.d_wup, &mut idx, lr_t, bc2);
            self.upd(&mut lw.w_down, &lg.d_wdown, &mut idx, lr_t, bc2);
        }
        self.upd(&mut model.rms_final, &grads.d_rf, &mut idx, lr_t, bc2);
        self.upd(&mut model.w_output, &grads.d_out, &mut idx, lr_t, bc2);
    }
    fn upd(&mut self, w: &mut [f32], g: &[f32], idx: &mut usize, lr_t: f32, bc2: f32) {
        for i in 0..w.len() {
            let j = *idx + i;
            if j >= self.m.len() { break; }
            self.m[j] = 0.9 * self.m[j] + 0.1 * g[i];
            self.v[j] = 0.999 * self.v[j] + 0.001 * g[i] * g[i];
            let vh = self.v[j] / bc2;
            w[i] *= 1.0 - self.lr * 0.01; // weight decay
            w[i] -= lr_t * self.m[j] / (vh.sqrt() + 1e-8);
        }
        *idx += w.len();
    }
}

// ═══════════════════════════════════════════════════════════════
// Forward + Backward
// ═══════════════════════════════════════════════════════════════
struct LayerActs {
    x_in: Vec<f32>, xna: Vec<f32>, q: Vec<f32>, k: Vec<f32>, v: Vec<f32>,
    aw: Vec<Vec<f32>>, ao: Vec<f32>, x_mid: Vec<f32>, xnf: Vec<f32>,
    gp: Vec<f32>, ga: Vec<f32>, up: Vec<f32>, gated: Vec<f32>,
}

fn forward_backward(model: &Model, tokens: &[u8]) -> (f32, ModelGrads) {
    let seq = tokens.len().min(MAXSEQ);
    if seq < 2 { return (f32::MAX, ModelGrads::new()); }

    let mut all_acts = Vec::with_capacity(seq);
    let mut all_k: Vec<Vec<Vec<f32>>> = vec![Vec::new(); NL];
    let mut all_v: Vec<Vec<Vec<f32>>> = vec![Vec::new(); NL];

    for t in 0..seq {
        let tok = tokens[t] as usize;
        let mut x = vec![0.0f32; D];
        for i in 0..D { x[i] = model.token_embed[tok*D+i] + model.pos_embed[t*D+i]; }

        let mut lacts = Vec::with_capacity(NL);
        for l in 0..NL {
            let ly = &model.layers[l];
            let x_in = x.clone();
            let mut xn = vec![0.0; D]; rmsnorm(&mut xn, &x_in, &ly.rms_attn);
            let (mut q, mut kv, mut vv) = (vec![0.0; D], vec![0.0; D], vec![0.0; D]);
            mv(&mut q, &ly.w_q, &xn, D, D); mv(&mut kv, &ly.w_k, &xn, D, D); mv(&mut vv, &ly.w_v, &xn, D, D);
            all_k[l].push(kv.clone()); all_v[l].push(vv.clone());

            let np = t + 1;
            let mut ao = vec![0.0f32; D];
            let dks = (DK as f32).sqrt();
            let mut aws = Vec::with_capacity(NH);
            for h in 0..NH {
                let ho = h * DK;
                let mut sc = vec![0.0f32; np];
                for p in 0..np { let mut s=0.0; for d in 0..DK { s += q[ho+d]*all_k[l][p][ho+d]; } sc[p]=s/dks; }
                softmax(&mut sc);
                for p in 0..np { let w=sc[p]; for d in 0..DK { ao[ho+d]+=w*all_v[l][p][ho+d]; } }
                aws.push(sc);
            }
            let mut proj = vec![0.0; D]; mv(&mut proj, &ly.w_o, &ao, D, D);
            for i in 0..D { x[i] = x_in[i] + proj[i]; }
            let xm = x.clone();
            let mut xnf = vec![0.0; D]; rmsnorm(&mut xnf, &xm, &ly.rms_ffn);
            let (mut gp, mut up) = (vec![0.0; DFF], vec![0.0; DFF]);
            mv(&mut gp, &ly.w_gate, &xnf, D, DFF); mv(&mut up, &ly.w_up, &xnf, D, DFF);
            let mut ga = vec![0.0; DFF]; let mut gated = vec![0.0; DFF];
            for i in 0..DFF { ga[i] = silu(gp[i]); gated[i] = ga[i]*up[i]; }
            let mut fo = vec![0.0; D]; mv(&mut fo, &ly.w_down, &gated, DFF, D);
            for i in 0..D { x[i] = xm[i] + fo[i]; }

            lacts.push(LayerActs { x_in, xna: xn, q, k: all_k[l][t].clone(), v: all_v[l][t].clone(),
                aw: aws, ao, x_mid: xm, xnf, gp, ga, up, gated });
        }
        let mut xf = vec![0.0; D]; rmsnorm(&mut xf, &x, &model.rms_final);
        let mut logits = vec![0.0; VOCAB]; mv(&mut logits, &model.w_output, &xf, D, VOCAB);
        all_acts.push((x, lacts, xf, logits));
    }

    // Backward
    let mut loss = 0.0f32;
    let nt = seq - 1;
    let mut grads = ModelGrads::new();

    for t in 0..nt {
        let tgt = tokens[t+1] as usize;
        let (ref ax, ref la, ref xfn, ref logits) = all_acts[t];
        let mut probs = logits.clone(); softmax(&mut probs);
        loss += -probs[tgt].max(1e-10).ln();
        let mut dl = probs; dl[tgt] -= 1.0;
        let s = 1.0 / nt as f32;
        for v in dl.iter_mut() { *v *= s; }

        outer_acc(&mut grads.d_out, &dl, xfn, D, VOCAB);
        let mut dxfn = vec![0.0; D]; mvt(&mut dxfn, &model.w_output, &dl, D, VOCAB);
        let mut dx = bwd_rms(&dxfn, ax, &model.rms_final, &mut grads.d_rf);

        for l in (0..NL).rev() {
            let la = &la[l]; let ly = &model.layers[l]; let lg = &mut grads.layers[l];
            let dfo = dx.clone();
            let mut dg = vec![0.0; DFF];
            outer_acc(&mut lg.d_wdown, &dfo, &la.gated, DFF, D);
            mvt(&mut dg, &ly.w_down, &dfo, DFF, D);
            let (mut dgp, mut dup) = (vec![0.0; DFF], vec![0.0; DFF]);
            for i in 0..DFF { dup[i]=dg[i]*la.ga[i]; dgp[i]=dg[i]*la.up[i]*silu_grad(la.gp[i]); }
            let mut dxnf = vec![0.0; D];
            outer_acc(&mut lg.d_wgate, &dgp, &la.xnf, D, DFF);
            outer_acc(&mut lg.d_wup, &dup, &la.xnf, D, DFF);
            mvt(&mut dxnf, &ly.w_gate, &dgp, D, DFF);
            mvta(&mut dxnf, &ly.w_up, &dup, D, DFF);
            let dxm = bwd_rms(&dxnf, &la.x_mid, &ly.rms_ffn, &mut lg.d_rms_ffn);
            let mut dxp = vec![0.0; D]; for i in 0..D { dxp[i]=dx[i]+dxm[i]; }

            outer_acc(&mut lg.d_wo, &dxp, &la.ao, D, D);
            let mut dao = vec![0.0; D]; mvt(&mut dao, &ly.w_o, &dxp, D, D);

            let np = t+1;
            let (mut dq, mut dks, mut dvs) = (vec![0.0; D], vec![0.0; D], vec![0.0; D]);
            let dksqrt = (DK as f32).sqrt();
            for h in 0..NH {
                let ho = h*DK; let wts = &la.aw[h];
                let mut dw = vec![0.0; np];
                for p in 0..np {
                    let mut s=0.0;
                    for d in 0..DK { s+=dao[ho+d]*all_v[l][p][ho+d]; if p==t{dvs[ho+d]+=wts[p]*dao[ho+d];} }
                    dw[p]=s;
                }
                let dot: f32 = (0..np).map(|p| dw[p]*wts[p]).sum();
                let mut dsc = vec![0.0f32; np];
                for p in 0..np { dsc[p]=wts[p]*(dw[p]-dot); }
                for p in 0..np { let ds=dsc[p]/dksqrt;
                    for d in 0..DK { dq[ho+d]+=ds*all_k[l][p][ho+d]; if p==t{dks[ho+d]+=ds*la.q[ho+d];} } }
            }
            outer_acc(&mut lg.d_wq, &dq, &la.xna, D, D);
            outer_acc(&mut lg.d_wk, &dks, &la.xna, D, D);
            outer_acc(&mut lg.d_wv, &dvs, &la.xna, D, D);
            let mut dxna = vec![0.0; D];
            mvt(&mut dxna, &ly.w_q, &dq, D, D);
            mvta(&mut dxna, &ly.w_k, &dks, D, D);
            mvta(&mut dxna, &ly.w_v, &dvs, D, D);
            let dxi = bwd_rms(&dxna, &la.x_in, &ly.rms_attn, &mut lg.d_rms_attn);
            for i in 0..D { dx[i]=dxp[i]+dxi[i]; }
        }
        let tok = tokens[t] as usize;
        for i in 0..D { grads.d_te[tok*D+i]+=dx[i]; grads.d_pe[t*D+i]+=dx[i]; }
    }
    (loss / nt as f32, grads)
}

// ═══════════════════════════════════════════════════════════════
// Corpus — kernel sentinel data
// ═══════════════════════════════════════════════════════════════
static CORPUS: &[&str] = &[
    "help: show commands", "ls: list files", "ps: show processes",
    "free: memory usage", "uptime: system uptime", "reboot: restart",
    "shutdown: power off", "clear: clear screen", "date: show date",
    "whoami: show user", "hostname: show name", "uname: system info",
    "heap OK", "interrupts enabled", "filesystem ready",
    "serial port active", "kernel healthy", "all checks passed",
    "brain loading from fs", "brain loaded OK", "brain not found",
    "brain save to disk", "brain init random", "micro sentinel active",
    "full brain connected", "full brain offline", "full brain available",
    "I am micro-Jarvis", "kernel sentinel ready", "checking kernel",
    "validating memory", "validating interrupts", "filesystem check OK",
    "loading full brain", "micro mode active", "sentinel watching",
    "error: heap corrupt", "error: interrupt fault", "error: fs error",
    "warning: brain large", "status: all nominal", "status: degraded",
    "Q: status A: nominal", "Q: heap A: OK", "Q: fs A: ready",
    "Q: brain A: loading", "Q: help A: type help",
    "kernel boot complete", "init sequence done", "ready for commands",
    "memory allocated OK", "page tables set up", "GDT loaded",
    "IDT configured", "APIC initialized", "timer running",
    "serial COM1 active", "framebuffer ready", "shell started",
];

// ═══════════════════════════════════════════════════════════════
// Math
// ═══════════════════════════════════════════════════════════════
fn mv(o: &mut [f32], w: &[f32], x: &[f32], c: usize, r: usize) {
    for i in 0..r { let mut s=0.0; let b=i*c; for j in 0..c { s+=w[b+j]*x[j]; } o[i]=s; }
}
fn mvt(o: &mut [f32], w: &[f32], y: &[f32], c: usize, r: usize) {
    o.fill(0.0); for i in 0..r { let b=i*c; let yr=y[i]; for j in 0..c { o[j]+=w[b+j]*yr; } }
}
fn mvta(o: &mut [f32], w: &[f32], y: &[f32], c: usize, r: usize) {
    for i in 0..r { let b=i*c; let yr=y[i]; for j in 0..c { o[j]+=w[b+j]*yr; } }
}
fn outer_acc(dw: &mut [f32], dy: &[f32], x: &[f32], c: usize, r: usize) {
    for i in 0..r { let b=i*c; let dr=dy[i]; for j in 0..c { dw[b+j]+=dr*x[j]; } }
}
fn rmsnorm(o: &mut [f32], x: &[f32], w: &[f32]) {
    let n=x.len(); let ss: f32=x.iter().map(|v|v*v).sum();
    let inv=1.0/(ss/n as f32+EPS).sqrt(); for i in 0..n { o[i]=x[i]*inv*w[i]; }
}
fn bwd_rms(dout: &[f32], x: &[f32], w: &[f32], dw: &mut [f32]) -> Vec<f32> {
    let n=x.len(); let ss: f32=x.iter().map(|v|v*v).sum();
    let inv=1.0/(ss/n as f32+EPS).sqrt();
    for i in 0..n { dw[i]+=dout[i]*x[i]*inv; }
    let mut dn=vec![0.0;n]; for i in 0..n { dn[i]=dout[i]*w[i]; }
    let mut dot=0.0f32; for i in 0..n { dot+=x[i]*inv*dn[i]; } dot/=n as f32;
    let mut dx=vec![0.0;n]; for i in 0..n { dx[i]=inv*(dn[i]-x[i]*inv*dot); } dx
}
fn softmax(d: &mut [f32]) {
    if d.is_empty(){return;} let mx=d.iter().copied().fold(f32::NEG_INFINITY,f32::max);
    for v in d.iter_mut(){*v=(*v-mx).exp();} let s: f32=d.iter().sum();
    if s>0.0{let i=1.0/s; for v in d.iter_mut(){*v*=i;}}
}
fn silu(x: f32) -> f32 { x/(1.0+(-x).exp()) }
fn silu_grad(x: f32) -> f32 { let s=1.0/(1.0+(-x).exp()); s+x*s*(1.0-s) }
fn vadd(a: &mut [f32], b: &[f32]) { a.iter_mut().zip(b).for_each(|(a,b)| *a+=b); }
fn vscale(a: &mut [f32], s: f32) { a.iter_mut().for_each(|v| *v*=s); }
fn rvec(n: usize, s: f32, seed: &mut u64) -> Vec<f32> { (0..n).map(|_| xf32(seed)*s).collect() }
fn xf32(st: &mut u64) -> f32 {
    let mut x=*st; x^=x<<13; x^=x>>7; x^=x<<17; *st=x;
    ((x>>40)as u32 as f32/(1u32<<24)as f32)*2.0-1.0
}

fn compute_loss(model: &Model, tokens: &[u8]) -> f32 {
    let seq=tokens.len().min(MAXSEQ); if seq<2{return f32::MAX;}
    let mut loss=0.0f32; let nt=seq-1;
    let mut all_k: Vec<Vec<Vec<f32>>>=vec![Vec::new();NL];
    let mut all_v: Vec<Vec<Vec<f32>>>=vec![Vec::new();NL];
    for t in 0..seq {
        let tok=tokens[t]as usize;
        let mut x=vec![0.0;D]; for i in 0..D{x[i]=model.token_embed[tok*D+i]+model.pos_embed[t*D+i];}
        for l in 0..NL {
            let ly=&model.layers[l]; let xi=x.clone();
            let mut xn=vec![0.0;D]; rmsnorm(&mut xn,&xi,&ly.rms_attn);
            let(mut q,mut kv,mut vv)=(vec![0.0;D],vec![0.0;D],vec![0.0;D]);
            mv(&mut q,&ly.w_q,&xn,D,D); mv(&mut kv,&ly.w_k,&xn,D,D); mv(&mut vv,&ly.w_v,&xn,D,D);
            all_k[l].push(kv); all_v[l].push(vv);
            let np=t+1; let mut ao=vec![0.0;D]; let dks=(DK as f32).sqrt();
            for h in 0..NH { let ho=h*DK;
                let mut sc=vec![0.0;np];
                for p in 0..np{let mut s=0.0;for d in 0..DK{s+=q[ho+d]*all_k[l][p][ho+d];}sc[p]=s/dks;}
                softmax(&mut sc);
                for p in 0..np{let w=sc[p];for d in 0..DK{ao[ho+d]+=w*all_v[l][p][ho+d];}}
            }
            let mut proj=vec![0.0;D]; mv(&mut proj,&ly.w_o,&ao,D,D);
            for i in 0..D{x[i]=xi[i]+proj[i];}
            let xm=x.clone(); let mut xnf=vec![0.0;D]; rmsnorm(&mut xnf,&xm,&ly.rms_ffn);
            let(mut gp,mut up)=(vec![0.0;DFF],vec![0.0;DFF]);
            mv(&mut gp,&ly.w_gate,&xnf,D,DFF); mv(&mut up,&ly.w_up,&xnf,D,DFF);
            let mut gated=vec![0.0;DFF]; for i in 0..DFF{gated[i]=silu(gp[i])*up[i];}
            let mut fo=vec![0.0;D]; mv(&mut fo,&ly.w_down,&gated,DFF,D);
            for i in 0..D{x[i]=xm[i]+fo[i];}
        }
        let mut xf=vec![0.0;D]; rmsnorm(&mut xf,&x,&model.rms_final);
        let mut logits=vec![0.0;VOCAB]; mv(&mut logits,&model.w_output,&xf,D,VOCAB);
        if t<nt { let tgt=tokens[t+1]as usize; let mut p=logits; softmax(&mut p); loss+=-p[tgt].max(1e-10).ln(); }
    }
    loss/nt as f32
}

// ═══════════════════════════════════════════════════════════════
// MAIN
// ═══════════════════════════════════════════════════════════════
fn main() {
    println!("==========================================");
    println!("  MICRO-JARVIS SENTINEL TRAINER");
    println!("  ~50K param kernel watchdog model");
    println!("==========================================");

    let t0 = Instant::now();
    let mut model = Model::new_random();
    let params = model.param_count();
    println!("\n[INIT] {} params ({:.1} KB)", params, params as f64 * 4.0 / 1024.0);

    // Eval before
    let mut loss_sum = 0.0f32;
    let mut cnt = 0;
    for text in CORPUS { let tk: Vec<u8>=text.bytes().collect(); if tk.len()>=2{loss_sum+=compute_loss(&model,&tk);cnt+=1;} }
    let loss_before = loss_sum / cnt as f32;
    println!("[PRE] Loss: {:.4} ({} samples)", loss_before, cnt);

    // Train — 50 epochs (model is tiny, this is fast)
    let mut opt = Adam::new(params);
    let epochs = 50;
    let batch_sz = 4;
    println!("\n[TRAIN] {} epochs, batch={}", epochs, batch_sz);
    let t1 = Instant::now();

    for ep in 0..epochs {
        let mut eloss = 0.0f32; let mut ecnt = 0;
        let mut ag = ModelGrads::new(); let mut bc = 0;
        for text in CORPUS {
            let tk: Vec<u8> = text.bytes().collect();
            if tk.len() < 2 { continue; }
            let (l, g) = forward_backward(&model, &tk);
            eloss += l; ecnt += 1; ag.accumulate(&g); bc += 1;
            if bc >= batch_sz {
                ag.scale(1.0/bc as f32); ag.clip_norm(1.0);
                opt.step(&mut model, &ag); ag.zero(); bc = 0;
            }
        }
        if bc > 0 { ag.scale(1.0/bc as f32); ag.clip_norm(1.0); opt.step(&mut model, &ag); ag.zero(); }

        // Cosine LR
        let prog = ep as f32 / epochs as f32;
        opt.lr = 0.002 * 0.5 * (1.0 + (std::f32::consts::PI * prog).cos());

        if (ep+1) % 10 == 0 || ep == 0 {
            println!("  Epoch {}/{}: loss={:.4} lr={:.5} ({:.1}s)",
                ep+1, epochs, eloss/ecnt.max(1)as f32, opt.lr, t1.elapsed().as_secs_f64());
        }
    }

    // Eval after
    loss_sum = 0.0; cnt = 0;
    for text in CORPUS { let tk: Vec<u8>=text.bytes().collect(); if tk.len()>=2{loss_sum+=compute_loss(&model,&tk);cnt+=1;} }
    let loss_after = loss_sum / cnt as f32;
    println!("\n[POST] Loss: {:.4} (was {:.4}, improvement {:.1}%)",
        loss_after, loss_before, (1.0-loss_after/loss_before)*100.0);

    // Export
    let floats = model.serialize();
    let bytes: Vec<u8> = floats.iter().flat_map(|f| f.to_le_bytes()).collect();
    let out = "jarvis_micro.bin";
    std::fs::write(out, &bytes).expect("write failed");
    println!("\n[EXPORT] {} params, {} bytes ({:.1} KB) -> {}",
        floats.len(), bytes.len(), bytes.len() as f64 / 1024.0, out);

    println!("\n==========================================");
    println!("  Micro sentinel trained and exported!");
    println!("  Copy to kernel/src/jarvis/jarvis_micro.bin");
    println!("==========================================");
}
