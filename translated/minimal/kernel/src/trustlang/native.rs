









use alloc::string::String;
use alloc::vec::Vec;
use alloc::collections::BTreeMap;
use alloc::format;

use super::parser::*;
use super::x86asm::{X86Asm, Reg, Cc, Dy};


pub struct Akl {
    
    pub aj: Vec<u8>,
    
    pub bql: usize,
    
    pub pd: Vec<String>,
}




pub type Bcv = fn(u8, usize, *const i64) -> i64;


struct FnCtx {
    cu: Dy,
    bbx: BTreeMap<String, i32>,  
    foz: i32,               
    bvf: Vec<Dy>,
    euk: Vec<Dy>,
}

impl FnCtx {
    fn new(cu: Dy) -> Self {
        Self {
            cu,
            bbx: BTreeMap::new(),
            foz: -8,
            bvf: Vec::new(),
            euk: Vec::new(),
        }
    }

    fn ijm(&mut self, j: &str) -> i32 {
        if let Some(&dz) = self.bbx.get(j) {
            return dz;
        }
        let dz = self.foz;
        self.bbx.insert(String::from(j), dz);
        self.foz -= 8;
        dz
    }

    fn bzt(&self) -> i32 {
        let js = (-self.foz) + 8; 
        
        (js + 15) & !15
    }
}


struct NativeCompiler {
    asm: X86Asm,
    hkr: BTreeMap<String, Dy>,
    pd: Vec<String>,
    fvr: BTreeMap<String, usize>,
    kfm: Dy,
}

impl NativeCompiler {
    fn new() -> Self {
        let mut asm = X86Asm::new();
        let xls = asm.dtl();
        Self {
            asm,
            hkr: BTreeMap::new(),
            pd: Vec::new(),
            fvr: BTreeMap::new(),
            kfm: xls,
        }
    }

    fn lfb(&mut self, e: &str) -> usize {
        if let Some(&w) = self.fvr.get(e) {
            return w;
        }
        let w = self.pd.len();
        self.pd.push(String::from(e));
        self.fvr.insert(String::from(e), w);
        w
    }

    
    fn imj(j: &str) -> Option<u8> {
        match j {
            "print"        => Some(0),
            "println"      => Some(1),
            "len"          => Some(2),
            "push"         => Some(3),
            "to_string"    => Some(4),
            "to_int"       => Some(5),
            "sqrt"         => Some(6),
            "abs"          => Some(7),
            "pixel"        => Some(8),
            "clear_screen" => Some(9),
            "fill_rect"    => Some(10),
            "draw_line"    => Some(11),
            "draw_circle"  => Some(12),
            "screen_w"     => Some(13),
            "screen_h"     => Some(14),
            "flush"        => Some(15),
            "draw_text"    => Some(16),
            "sleep"        => Some(17),
            "to_float"     => Some(18),
            "read_line"    => Some(19),
            _ => None,
        }
    }

    fn kkc(&mut self, alo: &Program) -> Result<Akl, String> {
        
        for item in &alo.pj {
            if let Item::Bs(bb) = item {
                let cu = self.asm.dtl();
                self.hkr.insert(bb.j.clone(), cu);
            }
        }

        
        
        
        self.asm.deg(self.kfm);
        
        self.asm.qvo(Reg::Aec);
        self.asm.aux();

        
        for item in &alo.pj {
            if let Item::Bs(bb) = item {
                self.kkb(bb)?;
            }
        }

        
        self.asm.vxw().jd(|aa| String::from(aa))?;

        let sme = self.hkr.get("main")
            .ok_or_else(|| String::from("no main() function found"))?;
        let bql = self.asm.cze[sme.0]
            .ok_or_else(|| String::from("main() label unresolved"))?;

        Ok(Akl {
            aj: self.asm.aj.clone(),
            pd: self.pd.clone(),
            bql,
        })
    }

    fn kkb(&mut self, aqy: &Abu) -> Result<(), String> {
        let cu = *self.hkr.get(&aqy.j)
            .ok_or_else(|| format!("function '{}' not registered", aqy.j))?;

        let mut be = FnCtx::new(cu);

        
        
        
        for (a, (j, msw)) in aqy.oi.iter().cf() {
            let dz = be.ijm(j);
            
            let _ = (a, dz);
        }

        
        self.hvw(&mut be, &aqy.gj);
        let bzt = be.bzt();

        
        self.asm.deg(cu);
        self.asm.vne(bzt);

        
        
        for (a, (j, msw)) in aqy.oi.iter().cf() {
            let cum = 16 + (a as i32) * 8;
            let bgu = *be.bbx.get(j).unwrap();
            self.asm.hrz(Reg::J, cum);
            self.asm.gna(bgu, Reg::J);
        }

        
        self.cjp(&mut be, &aqy.gj)?;

        
        self.asm.gmz(Reg::J, 0);
        self.asm.nqw();

        Ok(())
    }

    
    fn hvw(&self, be: &mut FnCtx, block: &Dj) {
        for stmt in &block.boq {
            match stmt {
                Stmt::Pu { j, .. } => { be.ijm(j); }
                Stmt::Gx { cne, ckc, .. } => {
                    self.hvw(be, cne);
                    if let Some(ebc) = ckc { self.hvw(be, ebc); }
                }
                Stmt::La { gj, .. } | Stmt::Pz(gj) => {
                    self.hvw(be, gj);
                }
                Stmt::Ll { bfp, gj, .. } => {
                    be.ijm(bfp);
                    be.ijm(&format!("__for_end_{}", bfp));
                    self.hvw(be, gj);
                }
                _ => {}
            }
        }
    }

    fn cjp(&mut self, be: &mut FnCtx, block: &Dj) -> Result<(), String> {
        for stmt in &block.boq {
            self.kkd(be, stmt)?;
        }
        Ok(())
    }

    fn kkd(&mut self, be: &mut FnCtx, stmt: &Stmt) -> Result<(), String> {
        match stmt {
            Stmt::Pu { j, init, .. } => {
                let dz = *be.bbx.get(j).unwrap();
                if let Some(expr) = init {
                    self.aex(be, expr)?;
                    self.asm.clz(Reg::J);
                    self.asm.gna(dz, Reg::J);
                }
            }
            Stmt::Vk { cd, bn } => {
                self.aex(be, bn)?;
                self.hdr(be, cd)?;
            }
            Stmt::Xw { op, cd, bn } => {
                self.aex(be, cd)?;
                self.aex(be, bn)?;
                self.npp(*op)?;
                self.hdr(be, &cd.clone())?;
            }
            Stmt::Expr(expr) => {
                self.aex(be, expr)?;
                self.asm.clz(Reg::J); 
            }
            Stmt::Hd(ap) => {
                if let Some(expr) = ap {
                    self.aex(be, expr)?;
                    self.asm.clz(Reg::J);
                } else {
                    self.asm.gmz(Reg::J, 0);
                }
                self.asm.nqw();
            }
            Stmt::Gx { mo, cne, ckc } => {
                self.aex(be, mo)?;
                self.asm.clz(Reg::J);
                self.asm.mkj(Reg::J, Reg::J);
                let kta = self.asm.dtl();
                self.asm.lgu(Cc::Se, kta); 
                self.cjp(be, cne)?;
                if let Some(ebc) = ckc {
                    let npz = self.asm.dtl();
                    self.asm.gko(npz);
                    self.asm.deg(kta);
                    self.cjp(be, ebc)?;
                    self.asm.deg(npz);
                } else {
                    self.asm.deg(kta);
                }
            }
            Stmt::La { mo, gj } => {
                let qc = self.asm.dtl();
                let ci = self.asm.dtl();
                be.bvf.push(qc);
                be.euk.push(ci);

                self.asm.deg(qc);
                self.aex(be, mo)?;
                self.asm.clz(Reg::J);
                self.asm.mkj(Reg::J, Reg::J);
                self.asm.lgu(Cc::Se, ci);
                self.cjp(be, gj)?;
                self.asm.gko(qc);
                self.asm.deg(ci);

                be.bvf.pop();
                be.euk.pop();
            }
            Stmt::Ll { bfp, iter, gj } => {
                if let Expr::Nt { ay, ci } = iter {
                    let jvh = *be.bbx.get(bfp).unwrap();
                    let slm = format!("__for_end_{}", bfp);
                    let nqb = *be.bbx.get(&slm).unwrap();

                    self.aex(be, ay)?;
                    self.asm.clz(Reg::J);
                    self.asm.gna(jvh, Reg::J);

                    self.aex(be, ci)?;
                    self.asm.clz(Reg::J);
                    self.asm.gna(nqb, Reg::J);

                    let qc = self.asm.dtl();
                    let ktl = self.asm.dtl();
                    be.bvf.push(qc);
                    be.euk.push(ktl);

                    self.asm.deg(qc);
                    
                    self.asm.hrz(Reg::J, jvh);
                    self.asm.hrz(Reg::Fe, nqb);
                    self.asm.fff(Reg::J, Reg::Fe);
                    self.asm.lgu(Cc::Wr, ktl);

                    self.cjp(be, gj)?;

                    
                    self.asm.hrz(Reg::J, jvh);
                    self.asm.jzg(Reg::J, 1);
                    self.asm.gna(jvh, Reg::J);
                    self.asm.gko(qc);
                    self.asm.deg(ktl);

                    be.bvf.pop();
                    be.euk.pop();
                } else {
                    return Err(String::from("for loop requires a range expression"));
                }
            }
            Stmt::Pz(gj) => {
                let qc = self.asm.dtl();
                let ci = self.asm.dtl();
                be.bvf.push(qc);
                be.euk.push(ci);
                self.asm.deg(qc);
                self.cjp(be, gj)?;
                self.asm.gko(qc);
                self.asm.deg(ci);
                be.bvf.pop();
                be.euk.pop();
            }
            Stmt::Vr => {
                if let Some(&ci) = be.euk.qv() {
                    self.asm.gko(ci);
                }
            }
            Stmt::Cg => {
                if let Some(&qc) = be.bvf.qv() {
                    self.asm.gko(qc);
                }
            }
        }
        Ok(())
    }

    fn aex(&mut self, be: &mut FnCtx, expr: &Expr) -> Result<(), String> {
        match expr {
            Expr::Ta(p) => {
                self.asm.lmu(Reg::J, *p);
                self.asm.dkx(Reg::J);
            }
            Expr::Wq(p) => {
                
                let fs = p.bsr() as i64;
                self.asm.lmu(Reg::J, fs);
                self.asm.dkx(Reg::J);
            }
            Expr::Rp(o) => {
                self.asm.gmz(Reg::J, if *o { 1 } else { 0 });
                self.asm.dkx(Reg::J);
            }
            Expr::Yw(e) => {
                
                let w = self.lfb(e);
                self.asm.lmu(Reg::J, w as i64);
                self.asm.dkx(Reg::J);
            }
            Expr::Kq(j) => {
                if let Some(&dz) = be.bbx.get(j) {
                    self.asm.hrz(Reg::J, dz);
                    self.asm.dkx(Reg::J);
                } else {
                    return Err(format!("undefined variable: {}", j));
                }
            }
            Expr::BinOp { op, fd, hw } => {
                self.aex(be, fd)?;
                self.aex(be, hw)?;
                self.npp(*op)?;
            }
            Expr::UnaryOp { op, expr } => {
                self.aex(be, expr)?;
                self.asm.clz(Reg::J);
                match op {
                    UnaryOp::Neg => self.asm.ury(Reg::J),
                    UnaryOp::Np => {
                        self.asm.mkj(Reg::J, Reg::J);
                        self.asm.ful(Cc::Se, Reg::J);
                        self.asm.fop(Reg::J, Reg::J);
                    }
                }
                self.asm.dkx(Reg::J);
            }
            Expr::En { ke, n } => {
                if let Some(kdf) = Self::imj(ke) {
                    
                    
                    
                    let byg = n.len();
                    
                    if byg > 0 {
                        self.asm.ppl(Reg::Qo, (byg as i32) * 8);
                    }
                    
                    for (a, ji) in n.iter().cf() {
                        self.aex(be, ji)?;
                        self.asm.clz(Reg::J);
                        
                        
                        let dz = (a as i32) * 8;
                        
                        self.ski(dz, Reg::J);
                    }
                    
                    self.asm.gmz(Reg::Bql, kdf as i32);
                    self.asm.gmz(Reg::Brf, byg as i32);
                    self.asm.jgg(Reg::Axm, Reg::Qo);
                    
                    self.asm.nbl(self.kfm);
                    
                    if byg > 0 {
                        self.asm.jzg(Reg::Qo, (byg as i32) * 8);
                    }
                    
                    self.asm.dkx(Reg::J);
                } else if let Some(&szh) = self.hkr.get(ke) {
                    
                    for ji in n.iter().vv() {
                        self.aex(be, ji)?;
                        
                    }
                    self.asm.nbl(szh);
                    
                    let byg = n.len();
                    if byg > 0 {
                        self.asm.jzg(Reg::Qo, (byg as i32) * 8);
                    }
                    
                    self.asm.dkx(Reg::J);
                } else {
                    return Err(format!("undefined function: {}", ke));
                }
            }
            Expr::Apu { expr, ty } => {
                self.aex(be, expr)?;
                match ty {
                    Type::R => {
                        
                        
                        
                    }
                    Type::Ab => {
                        
                    }
                    _ => {}
                }
            }
            Expr::Nt { ay, ci } => {
                
                self.aex(be, ay)?;
                
                self.aex(be, ci)?;
                self.asm.clz(Reg::J); 
            }
            Expr::Index { .. } | Expr::U(_) | Expr::Asg { .. } | Expr::Dj(_) => {
                
                
                self.asm.gmz(Reg::J, 0);
                self.asm.dkx(Reg::J);
            }
        }
        Ok(())
    }

    
    fn npp(&mut self, op: BinOp) -> Result<(), String> {
        self.asm.clz(Reg::Fe); 
        self.asm.clz(Reg::J); 
        match op {
            BinOp::Add => self.asm.qfm(Reg::J, Reg::Fe),
            BinOp::Sub => self.asm.wvo(Reg::J, Reg::Fe),
            BinOp::Mul => self.asm.tsk(Reg::J, Reg::Fe),
            BinOp::Div => {
                self.asm.ngw();
                self.asm.odd(Reg::Fe);
                
            }
            BinOp::Xp => {
                self.asm.ngw();
                self.asm.odd(Reg::Fe);
                self.asm.jgg(Reg::J, Reg::Axm); 
            }
            BinOp::Eq => {
                self.asm.fff(Reg::J, Reg::Fe);
                self.asm.ful(Cc::Se, Reg::J);
                self.asm.fop(Reg::J, Reg::J);
            }
            BinOp::Xu => {
                self.asm.fff(Reg::J, Reg::Fe);
                self.asm.ful(Cc::Adl, Reg::J);
                self.asm.fop(Reg::J, Reg::J);
            }
            BinOp::Lt => {
                self.asm.fff(Reg::J, Reg::Fe);
                self.asm.ful(Cc::Aur, Reg::J);
                self.asm.fop(Reg::J, Reg::J);
            }
            BinOp::Jn => {
                self.asm.fff(Reg::J, Reg::Fe);
                self.asm.ful(Cc::Aii, Reg::J);
                self.asm.fop(Reg::J, Reg::J);
            }
            BinOp::Xm => {
                self.asm.fff(Reg::J, Reg::Fe);
                self.asm.ful(Cc::Te, Reg::J);
                self.asm.fop(Reg::J, Reg::J);
            }
            BinOp::Wx => {
                self.asm.fff(Reg::J, Reg::Fe);
                self.asm.ful(Cc::Wr, Reg::J);
                self.asm.fop(Reg::J, Reg::J);
            }
            BinOp::Ex => self.asm.mvs(Reg::J, Reg::Fe),
            BinOp::Fx => self.asm.osw(Reg::J, Reg::Fe),
            BinOp::Vm => self.asm.mvs(Reg::J, Reg::Fe),
            BinOp::Vn => self.asm.osw(Reg::J, Reg::Fe),
            BinOp::Vo => self.asm.xwp(Reg::J, Reg::Fe),
            BinOp::Ob => {
                
                self.asm.wmy(Reg::J);
            }
            BinOp::Oc => {
                self.asm.wcr(Reg::J);
            }
        }
        self.asm.dkx(Reg::J);
        Ok(())
    }

    
    fn hdr(&mut self, be: &mut FnCtx, cd: &Expr) -> Result<(), String> {
        match cd {
            Expr::Kq(j) => {
                if let Some(&dz) = be.bbx.get(j) {
                    self.asm.clz(Reg::J);
                    self.asm.gna(dz, Reg::J);
                } else {
                    return Err(format!("undefined variable: {}", j));
                }
            }
            _ => return Err(String::from("native: unsupported store target")),
        }
        Ok(())
    }

    
    fn ski(&mut self, l: i32, cy: Reg) {
        
        
        let mut aip: u8 = 0x48;
        if cy.evf() { aip |= 0x04; }
        self.asm.aj.push(aip);
        self.asm.aj.push(0x89);
        if l == 0 {
            self.asm.aj.push(X86Asm::ms(0b00, cy.ael(), 0x04)); 
            self.asm.aj.push(0x24); 
        } else if l >= -128 && l <= 127 {
            self.asm.aj.push(X86Asm::ms(0b01, cy.ael(), 0x04));
            self.asm.aj.push(0x24);
            self.asm.aj.push(l as u8);
        } else {
            self.asm.aj.push(X86Asm::ms(0b10, cy.ael(), 0x04));
            self.asm.aj.push(0x24);
            self.asm.aj.bk(&l.ho());
        }
    }
}


pub fn hdq(iy: &str) -> Result<Akl, String> {
    let eb = super::lexer::fwz(iy)?;
    let gzb = super::parser::parse(&eb)?;
    let mut compiler = NativeCompiler::new();
    compiler.kkc(&gzb)
}




pub unsafe fn him(
    alo: &Akl,
    quf: Bcv,
) -> Result<i64, String> {
    let aj = &alo.aj;
    if aj.is_empty() {
        return Err(String::from("empty native program"));
    }

    
    let kuj = qgs(aj.len())
        .ok_or_else(|| String::from("failed to allocate executable memory"))?;

    
    core::ptr::copy_nonoverlapping(aj.fq(), kuj, aj.len());

    
    let bt: *const u8 = kuj.add(alo.bql);

    
    
    
    let result: i64;
    let qwq = quf as usize;
    let kts = bt as usize;

    core::arch::asm!(
        "mov r15, {cb}",
        "call {entry}",
        aiv = in(reg) qwq,
        bt = in(reg) kts,
        bd("rax") result,
        
        bd("rcx") _, bd("rdx") _, bd("rsi") _, bd("rdi") _,
        bd("r8") _, bd("r9") _, bd("r10") _, bd("r11") _,
        yip("C"),
    );

    
    sxd(kuj, aj.len());

    Ok(result)
}






fn qgs(aw: usize) -> Option<*mut u8> {
    use alloc::alloc::{alloc_zeroed, Layout};
    let kab = (aw + 4095) & !4095; 
    let layout = Layout::bjy(kab, 4096).bq()?;
    let ptr = unsafe { alloc_zeroed(layout) };
    if ptr.abq() { None } else { Some(ptr) }
}


fn sxd(ptr: *mut u8, aw: usize) {
    use alloc::alloc::{dealloc, Layout};
    let kab = (aw + 4095) & !4095;
    if let Ok(layout) = Layout::bjy(kab, 4096) {
        unsafe { dealloc(ptr, layout); }
    }
}
