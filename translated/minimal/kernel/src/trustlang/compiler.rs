




use alloc::string::String;
use alloc::vec::Vec;
use alloc::collections::BTreeMap;
use alloc::format;

use super::parser::*;
use super::vm::{Op, Bs, Aaf, imj};


struct Compiler {
    ajb: Vec<Bs>,
    iwb: BTreeMap<String, usize>, 
    pd: Vec<String>,
    fvr: BTreeMap<String, usize>,
}


struct FnCompiler {
    aj: Vec<u8>,
    bbx: BTreeMap<String, u8>,
    gnr: u8,
    bvf: Vec<usize>,      
    euj: Vec<Vec<usize>>, 
}

impl Compiler {
    fn new() -> Self {
        Self {
            ajb: Vec::new(),
            iwb: BTreeMap::new(),
            pd: Vec::new(),
            fvr: BTreeMap::new(),
        }
    }

    fn lfb(&mut self, e: &str) -> u16 {
        if let Some(&w) = self.fvr.get(e) {
            return w as u16;
        }
        let w = self.pd.len();
        self.pd.push(String::from(e));
        self.fvr.insert(String::from(e), w);
        w as u16
    }

    fn vud(&mut self, alo: &Program) {
        for (a, item) in alo.pj.iter().cf() {
            if let Item::Bs(bb) = item {
                self.iwb.insert(bb.j.clone(), a);
            }
        }
    }

    fn kkc(&mut self, alo: &Program) -> Result<Aaf, String> {
        self.vud(alo);

        for item in &alo.pj {
            match item {
                Item::Bs(bb) => self.kkb(bb)?,
                Item::Yx(_) => {} 
            }
        }

        let bt = self.iwb.get("main")
            .ok_or_else(|| String::from("no main() function found"))?;
        let bt = *bt;

        Ok(Aaf {
            ajb: self.ajb.clone(),
            pd: self.pd.clone(),
            bt,
        })
    }

    fn kkb(&mut self, aqy: &Abu) -> Result<(), String> {
        let mut gc = FnCompiler::new();

        
        for (j, msw) in &aqy.oi {
            gc.ija(j);
        }

        
        self.cjp(&mut gc, &aqy.gj)?;

        
        if gc.aj.qv().hu() != Some(Op::Hd as u8) {
            gc.aif(&mut self.pd, Op::Adz);
            gc.ism(0); 
            gc.aif(&mut self.pd, Op::Hd);
        }

        let ke = Bs {
            j: aqy.j.clone(),
            qkm: aqy.oi.len() as u8,
            bbx: gc.gnr,
            aj: gc.aj,
        };
        self.ajb.push(ke);
        Ok(())
    }

    fn cjp(&mut self, gc: &mut FnCompiler, block: &Dj) -> Result<(), String> {
        for stmt in &block.boq {
            self.kkd(gc, stmt)?;
        }
        Ok(())
    }

    fn kkd(&mut self, gc: &mut FnCompiler, stmt: &Stmt) -> Result<(), String> {
        match stmt {
            Stmt::Pu { j, init, .. } => {
                let gk = gc.ija(j);
                if let Some(expr) = init {
                    self.aex(gc, expr)?;
                    gc.aif(&mut self.pd, Op::Qv);
                    gc.aj.push(gk);
                }
            }
            Stmt::Vk { cd, bn } => {
                self.aex(gc, bn)?;
                self.hdr(gc, cd)?;
            }
            Stmt::Xw { op, cd, bn } => {
                self.aex(gc, cd)?;
                self.aex(gc, bn)?;
                
                let mpp = match op {
                    BinOp::Add => Op::Zq,
                    BinOp::Sub => Op::Azl,
                    BinOp::Mul => Op::Avv,
                    BinOp::Div => Op::Ara,
                    _ => Op::Zq,
                };
                gc.aif(&mut self.pd, mpp);
                self.hdr(gc, cd)?;
            }
            Stmt::Expr(expr) => {
                self.aex(gc, expr)?;
                gc.aif(&mut self.pd, Op::Bpg); 
            }
            Stmt::Hd(ap) => {
                if let Some(expr) = ap {
                    self.aex(gc, expr)?;
                } else {
                    gc.aif(&mut self.pd, Op::Adz);
                    gc.ism(0);
                }
                gc.aif(&mut self.pd, Op::Hd);
            }
            Stmt::Gx { mo, cne, ckc } => {
                self.aex(gc, mo)?;
                let ohj = gc.hhy(Op::Ajk);

                self.cjp(gc, cne)?;

                if let Some(ske) = ckc {
                    let uat = gc.hhy(Op::Nh);
                    gc.ewf(ohj);
                    self.cjp(gc, ske)?;
                    gc.ewf(uat);
                } else {
                    gc.ewf(ohj);
                }
            }
            Stmt::La { mo, gj } => {
                let eul = gc.aj.len();
                gc.bvf.push(eul);
                gc.euj.push(Vec::new());

                self.aex(gc, mo)?;
                let kun = gc.hhy(Op::Ajk);

                self.cjp(gc, gj)?;
                gc.isn(eul);

                gc.ewf(kun);

                
                let fdq = gc.euj.pop().age();
                for o in fdq { gc.ewf(o); }
                gc.bvf.pop();
            }
            Stmt::Ll { bfp, iter, gj } => {
                
                let gk = gc.ija(bfp);
                
                if let Expr::Nt { ay, ci } = iter {
                    self.aex(gc, ay)?;
                    gc.aif(&mut self.pd, Op::Qv);
                    gc.aj.push(gk);

                    
                    let nqc = gc.ija(&format!("__for_end_{}", gk));
                    self.aex(gc, ci)?;
                    gc.aif(&mut self.pd, Op::Qv);
                    gc.aj.push(nqc);

                    let eul = gc.aj.len();
                    gc.bvf.push(eul);
                    gc.euj.push(Vec::new());

                    
                    gc.aif(&mut self.pd, Op::Ti);
                    gc.aj.push(gk);
                    gc.aif(&mut self.pd, Op::Ti);
                    gc.aj.push(nqc);
                    gc.aif(&mut self.pd, Op::Auy);

                    let kun = gc.hhy(Op::Ajk);
                    self.cjp(gc, gj)?;

                    
                    gc.aif(&mut self.pd, Op::Ti);
                    gc.aj.push(gk);
                    gc.aif(&mut self.pd, Op::Adz);
                    gc.ism(1);
                    gc.aif(&mut self.pd, Op::Zq);
                    gc.aif(&mut self.pd, Op::Qv);
                    gc.aj.push(gk);

                    gc.isn(eul);
                    gc.ewf(kun);

                    let fdq = gc.euj.pop().age();
                    for o in fdq { gc.ewf(o); }
                    gc.bvf.pop();
                } else {
                    return Err(String::from("for loop requires a range expression"));
                }
            }
            Stmt::Pz(gj) => {
                let eul = gc.aj.len();
                gc.bvf.push(eul);
                gc.euj.push(Vec::new());

                self.cjp(gc, gj)?;
                gc.isn(eul);

                let fdq = gc.euj.pop().age();
                for o in fdq { gc.ewf(o); }
                gc.bvf.pop();
            }
            Stmt::Vr => {
                let fb = gc.hhy(Op::Nh);
                if let Some(fdq) = gc.euj.dsq() {
                    fdq.push(fb);
                }
            }
            Stmt::Cg => {
                if let Some(&ay) = gc.bvf.qv() {
                    gc.isn(ay);
                }
            }
        }
        Ok(())
    }

    fn aex(&mut self, gc: &mut FnCompiler, expr: &Expr) -> Result<(), String> {
        match expr {
            Expr::Ta(p) => {
                gc.aif(&mut self.pd, Op::Adz);
                gc.ism(*p);
            }
            Expr::Wq(p) => {
                gc.aif(&mut self.pd, Op::Bpq);
                gc.skh(*p);
            }
            Expr::Yw(e) => {
                let w = self.lfb(e);
                gc.aif(&mut self.pd, Op::Bpr);
                gc.isp(w);
            }
            Expr::Rp(o) => {
                gc.aif(&mut self.pd, Op::Bpp);
                gc.aj.push(if *o { 1 } else { 0 });
            }
            Expr::Kq(j) => {
                if let Some(&gk) = gc.bbx.get(j) {
                    gc.aif(&mut self.pd, Op::Ti);
                    gc.aj.push(gk);
                } else {
                    return Err(format!("undefined variable: {}", j));
                }
            }
            Expr::BinOp { op, fd, hw } => {
                self.aex(gc, fd)?;
                self.aex(gc, hw)?;
                let mpp = match op {
                    BinOp::Add => Op::Zq,
                    BinOp::Sub => Op::Azl,
                    BinOp::Mul => Op::Avv,
                    BinOp::Div => Op::Ara,
                    BinOp::Xp => Op::Bmj,
                    BinOp::Eq => Op::Bga,
                    BinOp::Xu => Op::Bnc,
                    BinOp::Lt => Op::Auy,
                    BinOp::Jn => Op::Bik,
                    BinOp::Xm => Op::Bkz,
                    BinOp::Wx => Op::Bhw,
                    BinOp::Ex => Op::Ex,
                    BinOp::Fx => Op::Fx,
                    BinOp::Vm => Op::Vm,
                    BinOp::Vn => Op::Vn,
                    BinOp::Vo => Op::Vo,
                    BinOp::Ob => Op::Ob,
                    BinOp::Oc => Op::Oc,
                };
                gc.aif(&mut self.pd, mpp);
            }
            Expr::UnaryOp { op, expr } => {
                self.aex(gc, expr)?;
                match op {
                    UnaryOp::Neg => gc.aif(&mut self.pd, Op::Bnd),
                    UnaryOp::Np => gc.aif(&mut self.pd, Op::Np),
                }
            }
            Expr::En { ke, n } => {
                
                if let Some(kdf) = imj(ke) {
                    for ji in n {
                        self.aex(gc, ji)?;
                    }
                    gc.aif(&mut self.pd, Op::Bdi);
                    gc.aj.push(kdf);
                    gc.aj.push(n.len() as u8);
                } else if let Some(&ssa) = self.iwb.get(ke) {
                    
                    for ji in n {
                        self.aex(gc, ji)?;
                    }
                    gc.aif(&mut self.pd, Op::En);
                    gc.isp(ssa as u16);
                    gc.aj.push(n.len() as u8);
                } else {
                    return Err(format!("undefined function: {}", ke));
                }
            }
            Expr::Index { array, index } => {
                self.aex(gc, array)?;
                self.aex(gc, index)?;
                gc.aif(&mut self.pd, Op::Bby);
            }
            Expr::U(hhx) => {
                for fhm in hhx {
                    self.aex(gc, fhm)?;
                }
                gc.aif(&mut self.pd, Op::Avz);
                gc.isp(hhx.len() as u16);
            }
            Expr::Nt { ay, ci } => {
                
                
                self.aex(gc, ay)?;
                self.aex(gc, ci)?;
                
                gc.aif(&mut self.pd, Op::Avz);
                gc.isp(2);
            }
            Expr::Apu { expr, ty } => {
                self.aex(gc, expr)?;
                match ty {
                    Type::R => gc.aif(&mut self.pd, Op::Biy),
                    Type::Ab => gc.aif(&mut self.pd, Op::Bgk),
                    _ => {} 
                }
            }
            Expr::Dj(block) => {
                self.cjp(gc, block)?;
            }
            Expr::Asg { .. } => {
                return Err(String::from("struct field access not yet supported"));
            }
        }
        Ok(())
    }

    fn hdr(&mut self, gc: &mut FnCompiler, cd: &Expr) -> Result<(), String> {
        match cd {
            Expr::Kq(j) => {
                if let Some(&gk) = gc.bbx.get(j) {
                    gc.aif(&mut self.pd, Op::Qv);
                    gc.aj.push(gk);
                } else {
                    return Err(format!("undefined variable: {}", j));
                }
            }
            Expr::Index { array, index } => {
                
                
                
                
                
                
                
                
                if let Expr::Kq(mwh) = array.as_ref() {
                    if let Some(&mwi) = gc.bbx.get(mwh) {
                        
                        let psd = gc.gnr;
                        gc.gnr += 1;
                        
                        gc.aif(&mut self.pd, Op::Qv);
                        gc.aj.push(psd);
                        
                        gc.aif(&mut self.pd, Op::Ti);
                        gc.aj.push(mwi);
                        
                        self.aex(gc, index)?;
                        
                        gc.aif(&mut self.pd, Op::Ti);
                        gc.aj.push(psd);
                        
                        gc.aif(&mut self.pd, Op::Bbz);
                        
                        gc.aif(&mut self.pd, Op::Qv);
                        gc.aj.push(mwi);
                    } else {
                        return Err(format!("undefined variable: {}", mwh));
                    }
                } else {
                    return Err(String::from("array index assignment requires a variable"));
                }
            }
            _ => return Err(String::from("invalid assignment target")),
        }
        Ok(())
    }
}

impl FnCompiler {
    fn new() -> Self {
        Self {
            aj: Vec::new(),
            bbx: BTreeMap::new(),
            gnr: 0,
            bvf: Vec::new(),
            euj: Vec::new(),
        }
    }

    fn ija(&mut self, j: &str) -> u8 {
        if let Some(&gk) = self.bbx.get(j) {
            return gk;
        }
        let gk = self.gnr;
        self.bbx.insert(String::from(j), gk);
        self.gnr += 1;
        gk
    }

    fn aif(&mut self, ydd: &mut Vec<String>, op: Op) {
        self.aj.push(op as u8);
    }

    fn ism(&mut self, p: i64) {
        self.aj.bk(&p.ho());
    }

    fn skh(&mut self, p: f64) {
        self.aj.bk(&p.ho());
    }

    fn isp(&mut self, p: u16) {
        self.aj.bk(&p.ho());
    }

    
    fn hhy(&mut self, op: Op) -> usize {
        self.aj.push(op as u8);
        let w = self.aj.len();
        self.aj.push(0); 
        self.aj.push(0);
        w
    }

    
    fn ewf(&mut self, w: usize) {
        let cd = self.aj.len() as u16;
        self.aj[w] = (cd & 0xFF) as u8;
        self.aj[w + 1] = ((cd >> 8) & 0xFF) as u8;
    }

    
    fn isn(&mut self, cd: usize) {
        self.aj.push(Op::Nh as u8);
        self.aj.push((cd & 0xFF) as u8);
        self.aj.push(((cd >> 8) & 0xFF) as u8);
    }
}


pub fn rmx(alo: &Program) -> Result<Aaf, String> {
    let mut compiler = Compiler::new();
    compiler.kkc(alo)
}
