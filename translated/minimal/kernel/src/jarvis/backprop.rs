











use alloc::vec::Vec;
use alloc::vec;
use super::model::*;






pub struct LayerGrads {
    pub dfp: Vec<f32>,   
    pub dfx: Vec<f32>,         
    pub dfv: Vec<f32>,         
    pub dfz: Vec<f32>,         
    pub dfw: Vec<f32>,         
    pub dfq: Vec<f32>,    
    pub dfu: Vec<f32>,      
    pub dfy: Vec<f32>,        
    pub dft: Vec<f32>,      
}

impl LayerGrads {
    pub fn new() -> Self {
        LayerGrads {
            dfp: vec![0.0; E_],
            dfx: vec![0.0; E_ * E_],
            dfv: vec![0.0; E_ * E_],
            dfz: vec![0.0; E_ * E_],
            dfw: vec![0.0; E_ * E_],
            dfq: vec![0.0; E_],
            dfu: vec![0.0; E_ * Y_],
            dfy: vec![0.0; E_ * Y_],
            dft: vec![0.0; Y_ * E_],
        }
    }

    pub fn ajs(&mut self) {
        for p in self.dfp.el() { *p = 0.0; }
        for p in self.dfx.el() { *p = 0.0; }
        for p in self.dfv.el() { *p = 0.0; }
        for p in self.dfz.el() { *p = 0.0; }
        for p in self.dfw.el() { *p = 0.0; }
        for p in self.dfq.el() { *p = 0.0; }
        for p in self.dfu.el() { *p = 0.0; }
        for p in self.dfy.el() { *p = 0.0; }
        for p in self.dft.el() { *p = 0.0; }
    }
}


pub struct ModelGrads {
    pub dfs: Vec<f32>,  
    pub dfo: Vec<f32>,    
    pub my: Vec<LayerGrads>,
    pub dfr: Vec<f32>,    
    pub dfn: Vec<f32>,       
}

impl ModelGrads {
    pub fn new() -> Self {
        let mut my = Vec::fc(AZ_);
        for _ in 0..AZ_ {
            my.push(LayerGrads::new());
        }
        ModelGrads {
            dfs: vec![0.0; BG_ * E_],
            dfo: vec![0.0; CY_ * E_],
            my,
            dfr: vec![0.0; E_],
            dfn: vec![0.0; E_ * BG_],
        }
    }

    pub fn ajs(&mut self) {
        for p in self.dfs.el() { *p = 0.0; }
        for p in self.dfo.el() { *p = 0.0; }
        for dm in &mut self.my { dm.ajs(); }
        for p in self.dfr.el() { *p = 0.0; }
        for p in self.dfn.el() { *p = 0.0; }
    }

    
    pub fn az(&self) -> usize {
        self.dfs.len() + self.dfo.len()
        + self.my.iter().map(|dm| {
            dm.dfp.len() + dm.dfx.len() + dm.dfv.len() + dm.dfz.len()
            + dm.dfw.len() + dm.dfq.len() + dm.dfu.len() + dm.dfy.len()
            + dm.dft.len()
        }).sum::<usize>()
        + self.dfr.len() + self.dfn.len()
    }

    
    pub fn thi(&self) -> f32 {
        let mut rv = 0.0f32;
        let coj = |rv: &mut f32, e: &[f32]| { for &at in e { *rv += at * at; } };
        coj(&mut rv, &self.dfs);
        coj(&mut rv, &self.dfo);
        for dm in &self.my {
            coj(&mut rv, &dm.dfp);
            coj(&mut rv, &dm.dfx); coj(&mut rv, &dm.dfv);
            coj(&mut rv, &dm.dfz); coj(&mut rv, &dm.dfw);
            coj(&mut rv, &dm.dfq);
            coj(&mut rv, &dm.dfu); coj(&mut rv, &dm.dfy);
            coj(&mut rv, &dm.dft);
        }
        coj(&mut rv, &self.dfr);
        coj(&mut rv, &self.dfn);
        ccw(rv)
    }

    
    pub fn hcy(&mut self, olz: f32) {
        let abh = self.thi();
        if abh > olz && abh > 0.0 {
            let e = olz / abh;
            let jt = |p: &mut [f32], e: f32| { for at in p.el() { *at *= e; } };
            jt(&mut self.dfs, e);
            jt(&mut self.dfo, e);
            for dm in &mut self.my {
                jt(&mut dm.dfp, e); jt(&mut dm.dfx, e); jt(&mut dm.dfv, e);
                jt(&mut dm.dfz, e); jt(&mut dm.dfw, e); jt(&mut dm.dfq, e);
                jt(&mut dm.dfu, e); jt(&mut dm.dfy, e); jt(&mut dm.dft, e);
            }
            jt(&mut self.dfr, e);
            jt(&mut self.dfn, e);
        }
    }

    
    pub fn mtk(&mut self, gq: &ModelGrads) {
        let add = |cs: &mut [f32], cy: &[f32]| {
            for (bc, e) in cs.el().fca(cy.iter()) { *bc += *e; }
        };
        add(&mut self.dfs, &gq.dfs);
        add(&mut self.dfo, &gq.dfo);
        for (dl, aks) in self.my.el().fca(gq.my.iter()) {
            add(&mut dl.dfp, &aks.dfp);
            add(&mut dl.dfx, &aks.dfx); add(&mut dl.dfv, &aks.dfv);
            add(&mut dl.dfz, &aks.dfz); add(&mut dl.dfw, &aks.dfw);
            add(&mut dl.dfq, &aks.dfq);
            add(&mut dl.dfu, &aks.dfu); add(&mut dl.dfy, &aks.dfy);
            add(&mut dl.dft, &aks.dft);
        }
        add(&mut self.dfr, &gq.dfr);
        add(&mut self.dfn, &gq.dfn);
    }

    
    pub fn bv(&mut self, e: f32) {
        let jt = |p: &mut [f32]| { for at in p.el() { *at *= e; } };
        jt(&mut self.dfs);
        jt(&mut self.dfo);
        for dm in &mut self.my {
            jt(&mut dm.dfp); jt(&mut dm.dfx); jt(&mut dm.dfv);
            jt(&mut dm.dfz); jt(&mut dm.dfw); jt(&mut dm.dfq);
            jt(&mut dm.dfu); jt(&mut dm.dfy); jt(&mut dm.dft);
        }
        jt(&mut self.dfr);
        jt(&mut self.dfn);
    }
}






struct Bph {
    
    b: Vec<f32>,
    
    oii: Vec<Bkt>,
    
    qam: Vec<f32>,
    
    auq: Vec<f32>,
}

struct Bkt {
    
    ihn: Vec<f32>,       
    jxo: Vec<f32>,
    fm: Vec<f32>,          
    eh: Vec<f32>,          
    p: Vec<f32>,          
    mwp: Vec<Vec<f32>>,  
    con: Vec<f32>,   
    jkf: Vec<f32>,   
    
    iho: Vec<f32>,      
    fzc: Vec<f32>, 
    hky: Vec<f32>,   
    hkx: Vec<f32>,   
    bln: Vec<f32>,         
    hkz: Vec<f32>,      
    cxv: Vec<f32>,    
}






#[allow(bgr)]
fn ami(bd: &mut [f32], d: &[f32], b: &[f32], ec: usize, lk: usize) {
    for m in 0..lk {
        let mut sum = 0.0f32;
        let ar = m * ec;
        for r in 0..ec {
            sum += d[ar + r] * b[r];
        }
        bd[m] = sum;
    }
}


#[allow(bgr)]
fn cbl(bd: &mut [f32], b: &[f32], amz: &[f32]) -> f32 {
    let bo = b.len();
    let mut rv = 0.0f32;
    for &p in b { rv += p * p; }
    let bfd = ccw(rv / bo as f32 + HC_);
    let wq = 1.0 / bfd;
    for a in 0..bo {
        bd[a] = b[a] * wq * amz[a];
    }
    wq 
}

fn gss(f: &mut [f32]) {
    if f.is_empty() { return; }
    let am = f.iter().hu().cqs(f32::IP_, f32::am);
    let mut sum = 0.0f32;
    for p in f.el() {
        *p = kat(*p - am);
        sum += *p;
    }
    if sum > 0.0 {
        let wq = 1.0 / sum;
        for p in f.el() { *p *= wq; }
    }
}

fn kat(b: f32) -> f32 {
    if b > 88.0 { return f32::O; }
    if b < -88.0 { return 0.0; }
    let q = (1 << 23) as f32 / core::f32::consts::IG_;
    let o = (1 << 23) as f32 * (127.0 - 0.04368);
    let fs = ((q * b + o) as i32).am(0) as u32;
    f32::bhb(fs)
}

pub fn ccw(b: f32) -> f32 {
    if b <= 0.0 { return 0.0; }
    let fs = b.bsr();
    let anj = f32::bhb((fs >> 1) + 0x1FBD_1DF5);
    (anj + b / anj) * 0.5
}

fn woi(b: f32) -> f32 {
    let sig = 1.0 / (1.0 + kat(-b));
    b * sig
}

fn woj(b: f32) -> f32 {
    let sig = 1.0 / (1.0 + kat(-b));
    sig + b * sig * (1.0 - sig)
}










pub fn ivk(model: &TransformerWeights, eb: &[u8]) -> (f32, ModelGrads) {
    let anz = eb.len().v(super::model::CY_);
    if anz < 2 {
        return (f32::O, ModelGrads::new());
    }

    
    let mut mup: Vec<Bph> = Vec::fc(anz);
    
    let mut ijh: Vec<Vec<Vec<f32>>> = vec![Vec::new(); AZ_]; 
    let mut iji: Vec<Vec<Vec<f32>>> = vec![Vec::new(); AZ_];

    for ab in 0..anz {
        let cil = eb[ab] as usize;
        let u = ab;

        
        let mut b = vec![0.0f32; E_];
        for a in 0..E_ {
            b[a] = model.bpa[cil * E_ + a] + model.cgq[u * E_ + a];
        }

        let mut oij = Vec::fc(AZ_);

        for dm in 0..AZ_ {
            let fl = &model.my[dm];
            let ihn = b.clone();

            
            let mut ihp = vec![0.0f32; E_];
            let _ = super::simd::cbl(&mut ihp, &ihn, &fl.cmh);

            
            let mut fm = vec![0.0f32; E_];
            let mut eh = vec![0.0f32; E_];
            let mut p = vec![0.0f32; E_];
            super::simd::ami(&mut fm, &fl.biw, &ihp, E_, E_);
            super::simd::ami(&mut eh, &fl.biu, &ihp, E_, E_);
            super::simd::ami(&mut p, &fl.bpg, &ihp, E_, E_);

            
            ijh[dm].push(eh.clone());
            iji[dm].push(p.clone());

            
            let brl = ab + 1;
            let mut con = vec![0.0f32; E_];
            let kns = ccw(CU_ as f32);
            let mut mwq = Vec::fc(FP_);

            for i in 0..FP_ {
                let bra = i * CU_;
                let mut eyd = vec![0.0f32; brl];
                for ai in 0..brl {
                    let mut e = 0.0f32;
                    for bc in 0..CU_ {
                        e += fm[bra + bc] * ijh[dm][ai][bra + bc];
                    }
                    eyd[ai] = e / kns;
                }
                gss(&mut eyd);

                for ai in 0..brl {
                    let d = eyd[ai];
                    for bc in 0..CU_ {
                        con[bra + bc] += d * iji[dm][ai][bra + bc];
                    }
                }
                mwq.push(eyd);
            }

            
            let mut aci = vec![0.0f32; E_];
            super::simd::ami(&mut aci, &fl.biv, &con, E_, E_);

            
            for a in 0..E_ { b[a] = ihn[a] + aci[a]; }
            let iho = b.clone();

            
            let mut fzc = vec![0.0f32; E_];
            let _ = super::simd::cbl(&mut fzc, &iho, &fl.cmi);

            
            let mut hky = vec![0.0f32; Y_];
            let mut bln = vec![0.0f32; Y_];
            super::simd::ami(&mut hky, &fl.bit, &fzc, E_, Y_);
            super::simd::ami(&mut bln, &fl.bpf, &fzc, E_, Y_);

            let mut hkx = vec![0.0f32; Y_];
            let mut hkz = vec![0.0f32; Y_];
            for a in 0..Y_ {
                hkx[a] = woi(hky[a]);
                hkz[a] = hkx[a] * bln[a];
            }

            let mut cxv = vec![0.0f32; E_];
            super::simd::ami(&mut cxv, &fl.bpe, &hkz, Y_, E_);

            
            for a in 0..E_ { b[a] = iho[a] + cxv[a]; }

            oij.push(Bkt {
                ihn, jxo: ihp, fm, eh: ijh[dm][ab].clone(), p: iji[dm][ab].clone(),
                mwp: mwq, con, jkf: aci,
                iho, fzc, hky, hkx, bln, hkz, cxv,
            });
        }

        
        let mut mro = vec![0.0f32; E_];
        super::simd::cbl(&mut mro, &b, &model.chg);

        
        let mut auq = vec![0.0f32; BG_];
        super::simd::ami(&mut auq, &model.bft, &mro, E_, BG_);

        mup.push(Bph {
            b: b.clone(),
            oii: oij,
            qam: mro,
            auq,
        });
    }

    
    let mut ayy = 0.0f32;
    let gnj = anz - 1;
    let mut arg = ModelGrads::new();

    
    
    

    
    for ab in 0..gnj {
        let cd = eb[ab + 1] as usize;
        let iiw = &mup[ab];

        
        
        let mut gpw = iiw.auq.clone();
        gss(&mut gpw);

        
        let vam = gpw[cd].am(1e-10);
        ayy += -ees(vam);

        let mut iqk = gpw; 
        iqk[cd] -= 1.0;  
        
        let bv = 1.0 / gnj as f32;
        for p in iqk.el() { *p *= bv; }

        
        
        super::simd::ctd(&mut arg.dfn, &iqk, &iiw.qam, E_, BG_);

        
        let mut njg = vec![0.0f32; E_];
        super::simd::dta(&mut njg, &model.bft, &iqk, E_, BG_);

        
        let mut eol = kbv(&njg, &iiw.b, &model.chg, &mut arg.dfr);

        
        for dm in (0..AZ_).vv() {
            let auo = &iiw.oii[dm];
            let fl = &model.my[dm];
            let bnk = &mut arg.my[dm];

            
            let njd = eol.clone(); 
            

            
            
            let mut knp = vec![0.0f32; Y_];
            super::simd::ctd(&mut bnk.dft, &njd, &auo.hkz, Y_, E_);
            super::simd::dta(&mut knp, &fl.bpe, &njd, Y_, E_);

            
            let mut kno = vec![0.0f32; Y_];
            let mut knw = vec![0.0f32; Y_];
            for a in 0..Y_ {
                
                let rsx = knp[a] * auo.bln[a];
                
                knw[a] = knp[a] * auo.hkx[a];
                
                kno[a] = rsx * woj(auo.hky[a]);
            }

            
            
            let mut knz = vec![0.0f32; E_];
            
            super::simd::ctd(&mut bnk.dfu, &kno, &auo.fzc, E_, Y_);
            super::simd::ctd(&mut bnk.dfy, &knw, &auo.fzc, E_, Y_);
            
            super::simd::dta(&mut knz, &fl.bit, &kno, E_, Y_);
            super::simd::euq(&mut knz, &fl.bpf, &knw, E_, Y_);

            
            let rtd = kbv(&knz, &auo.iho, &fl.cmi, &mut bnk.dfq);

            
            let mut iqm = vec![0.0f32; E_];
            for a in 0..E_ {
                iqm[a] = eol[a] + rtd[a]; 
            }

            
            super::simd::ctd(&mut bnk.dfw, &iqm, &auo.con, E_, E_);
            let mut knn = vec![0.0f32; E_];
            super::simd::dta(&mut knn, &fl.biv, &iqm, E_, E_);

            
            let kns = ccw(CU_ as f32);
            let brl = ab + 1;
            let mut knv = vec![0.0f32; E_];
            let mut knr = vec![0.0f32; E_];
            let mut knx = vec![0.0f32; E_];
            for i in 0..FP_ {
                let bra = i * CU_;
                let mrj = &auo.mwp[i];

                let mut kny = vec![0.0f32; brl];
                for ai in 0..brl {
                    let mut e = 0.0f32;
                    for bc in 0..CU_ {
                        e += knn[bra + bc] * iji[dm][ai][bra + bc];
                        if ai == ab { knx[bra + bc] += mrj[ai] * knn[bra + bc]; }
                    }
                    kny[ai] = e;
                }

                let amb: f32 = (0..brl).map(|ai| kny[ai] * mrj[ai]).sum();
                let mut njf = vec![0.0f32; brl];
                for ai in 0..brl {
                    njf[ai] = mrj[ai] * (kny[ai] - amb);
                }

                for ai in 0..brl {
                    let bjw = njf[ai] / kns;
                    for bc in 0..CU_ {
                        knv[bra + bc] += bjw * ijh[dm][ai][bra + bc];
                        if ai == ab { knr[bra + bc] += bjw * auo.fm[bra + bc]; }
                    }
                }
            }

            
            
            super::simd::ctd(&mut bnk.dfx, &knv, &auo.jxo, E_, E_);
            super::simd::ctd(&mut bnk.dfv, &knr, &auo.jxo, E_, E_);
            super::simd::ctd(&mut bnk.dfz, &knx, &auo.jxo, E_, E_);
            
            let mut iqn = vec![0.0f32; E_];
            super::simd::dta(&mut iqn, &fl.biw, &knv, E_, E_);
            super::simd::euq(&mut iqn, &fl.biu, &knr, E_, E_);
            super::simd::euq(&mut iqn, &fl.bpg, &knx, E_, E_);

            
            let rtc = kbv(&iqn, &auo.ihn, &fl.cmh, &mut bnk.dfp);

            
            for a in 0..E_ {
                eol[a] = iqm[a] + rtc[a];
            }
        }

        
        let cil = eb[ab] as usize;
        for a in 0..E_ {
            arg.dfs[cil * E_ + a] += eol[a];
            arg.dfo[ab * E_ + a] += eol[a];
        }
    }

    let bdl = ayy / gnj as f32;
    (bdl, arg)
}







fn kbv(nje: &[f32], b: &[f32], amz: &[f32], rtb: &mut [f32]) -> Vec<f32> {
    let bo = b.len();
    let mut rv = 0.0f32;
    for &p in b { rv += p * p; }
    let bfd = ccw(rv / bo as f32 + HC_);
    let bva = 1.0 / bfd;

    
    for a in 0..bo {
        rtb[a] += nje[a] * b[a] * bva;
    }

    
    
    
    let mut knu = vec![0.0f32; bo];
    for a in 0..bo {
        knu[a] = nje[a] * amz[a];
    }

    
    let mut amb = 0.0f32;
    for a in 0..bo {
        amb += b[a] * bva * knu[a];
    }
    amb /= bo as f32;

    let mut eol = vec![0.0f32; bo];
    for a in 0..bo {
        eol[a] = bva * (knu[a] - b[a] * bva * amb);
    }
    eol
}





fn ees(b: f32) -> f32 {
    if b <= 0.0 { return -88.0; }
    let fs = b.bsr();
    let aa = ((fs >> 23) & 0xFF) as f32 - 127.0;
    let ef = f32::bhb((fs & 0x007FFFFF) | 0x3F800000);
    (aa + (ef - 1.0) * 1.4427) * core::f32::consts::IG_
}
