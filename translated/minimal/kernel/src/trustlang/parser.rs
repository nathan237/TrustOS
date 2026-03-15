




use alloc::string::String;
use alloc::vec::Vec;
use alloc::boxed::Box;
use alloc::format;

use super::lexer::{Token, TokenKind};




#[derive(Debug, Clone)]
pub struct Program {
    pub pj: Vec<Item>,
}


#[derive(Debug, Clone)]
pub enum Item {
    Bs(Abu),
    Yx(Azj),
}


#[derive(Debug, Clone)]
pub struct Abu {
    pub j: String,
    pub oi: Vec<(String, Type)>,
    pub pcy: Type,
    pub gj: Dj,
}


#[derive(Debug, Clone)]
pub struct Azj {
    pub j: String,
    pub fields: Vec<(String, Type)>,
}


#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    Ab,
    R,
    Em,
    He,
    Cn,
    U(Box<Type>),
    Chu(String),
}


#[derive(Debug, Clone)]
pub struct Dj {
    pub boq: Vec<Stmt>,
}


#[derive(Debug, Clone)]
pub enum Stmt {
    Pu { j: String, oos: bool, ty: Option<Type>, init: Option<Expr> },
    Vk { cd: Expr, bn: Expr },
    Xw { op: BinOp, cd: Expr, bn: Expr },
    Expr(Expr),
    Hd(Option<Expr>),
    Gx { mo: Expr, cne: Dj, ckc: Option<Dj> },
    La { mo: Expr, gj: Dj },
    Ll { bfp: String, iter: Expr, gj: Dj },
    Pz(Dj),
    Vr,
    Cg,
}


#[derive(Debug, Clone)]
pub enum Expr {
    Ta(i64),
    Wq(f64),
    Yw(String),
    Rp(bool),
    Kq(String),
    BinOp { op: BinOp, fd: Box<Expr>, hw: Box<Expr> },
    UnaryOp { op: UnaryOp, expr: Box<Expr> },
    En { ke: String, n: Vec<Expr> },
    Index { array: Box<Expr>, index: Box<Expr> },
    Asg { uww: Box<Expr>, buj: String },
    U(Vec<Expr>),
    Nt { ay: Box<Expr>, ci: Box<Expr> },
    Apu { expr: Box<Expr>, ty: Type },
    Dj(Dj),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BinOp {
    Add, Sub, Mul, Div, Xp,
    Eq, Xu, Lt, Jn, Xm, Wx,
    Ex, Fx,
    Vm, Vn, Vo, Ob, Oc,
}

#[derive(Debug, Clone, Copy)]
pub enum UnaryOp { Neg, Np }



struct Parser {
    eb: Vec<Token>,
    u: usize,
}

impl Parser {
    fn new(eb: Vec<Token>) -> Self {
        Self { eb, u: 0 }
    }

    fn amm(&self) -> &TokenKind {
        &self.eb.get(self.u).map(|ab| &ab.kk).unwrap_or(&TokenKind::Im)
    }

    fn nb(&mut self) -> &Token {
        let cil = &self.eb[self.u];
        if self.u < self.eb.len() - 1 { self.u += 1; }
        cil
    }

    fn expect(&mut self, qy: &TokenKind) -> Result<&Token, String> {
        if core::mem::gew(self.amm()) == core::mem::gew(qy) {
            Ok(self.nb())
        } else {
            let cil = &self.eb[self.u];
            Err(format!("L{}:{}: expected {:?}, got {:?}", cil.line, cil.bj, qy, cil.kk))
        }
    }

    fn aoi(&self, kk: &TokenKind) -> bool {
        core::mem::gew(self.amm()) == core::mem::gew(kk)
    }

    fn jdn(&self) -> (usize, usize) {
        let ab = &self.eb[self.u.v(self.eb.len() - 1)];
        (ab.line, ab.bj)
    }

    

    fn vdf(&mut self) -> Result<Program, String> {
        let mut pj = Vec::new();
        while !self.aoi(&TokenKind::Im) {
            pj.push(self.vct()?);
        }
        Ok(Program { pj })
    }

    fn vct(&mut self) -> Result<Item, String> {
        match self.amm() {
            TokenKind::Fn => Ok(Item::Bs(self.vci()?)),
            TokenKind::Yx => Ok(Item::Yx(self.vdw()?)),
            _ => {
                let (dm, r) = self.jdn();
                Err(format!("L{}:{}: expected 'fn' or 'struct', got {:?}", dm, r, self.amm()))
            }
        }
    }

    fn vci(&mut self) -> Result<Abu, String> {
        self.expect(&TokenKind::Fn)?;
        let j = self.fqh()?;
        self.expect(&TokenKind::Kr)?;
        let oi = self.lss()?;
        self.expect(&TokenKind::Jv)?;

        let pcy = if self.aoi(&TokenKind::Ov) {
            self.nb();
            self.gov()?
        } else {
            Type::Cn
        };

        let gj = self.fqf()?;
        Ok(Abu { j, oi, pcy, gj })
    }

    fn vdw(&mut self) -> Result<Azj, String> {
        self.expect(&TokenKind::Yx)?;
        let j = self.fqh()?;
        self.expect(&TokenKind::Ajn)?;
        let mut fields = Vec::new();
        while !self.aoi(&TokenKind::Yj) && !self.aoi(&TokenKind::Im) {
            let ebt = self.fqh()?;
            self.expect(&TokenKind::Ahb)?;
            let syr = self.gov()?;
            fields.push((ebt, syr));
            if self.aoi(&TokenKind::Aar) { self.nb(); }
        }
        self.expect(&TokenKind::Yj)?;
        Ok(Azj { j, fields })
    }

    fn lss(&mut self) -> Result<Vec<(String, Type)>, String> {
        let mut oi = Vec::new();
        while !self.aoi(&TokenKind::Jv) && !self.aoi(&TokenKind::Im) {
            let j = self.fqh()?;
            self.expect(&TokenKind::Ahb)?;
            let ty = self.gov()?;
            oi.push((j, ty));
            if self.aoi(&TokenKind::Aar) { self.nb(); }
        }
        Ok(oi)
    }

    fn gov(&mut self) -> Result<Type, String> {
        match self.amm().clone() {
            TokenKind::Buv => { self.nb(); Ok(Type::Ab) }
            TokenKind::Buu => { self.nb(); Ok(Type::R) }
            TokenKind::But => { self.nb(); Ok(Type::Em) }
            TokenKind::Buw => { self.nb(); Ok(Type::He) }
            TokenKind::Ajo => {
                self.nb();
                let ff = self.gov()?;
                self.expect(&TokenKind::Aed)?;
                Ok(Type::U(Box::new(ff)))
            }
            TokenKind::Kq(j) => {
                let bo = j.clone();
                self.nb();
                Ok(Type::Chu(bo))
            }
            _ => {
                let (dm, r) = self.jdn();
                Err(format!("L{}:{}: expected type, got {:?}", dm, r, self.amm()))
            }
        }
    }

    fn fqh(&mut self) -> Result<String, String> {
        if let TokenKind::Kq(j) = self.amm().clone() {
            let bo = j.clone();
            self.nb();
            Ok(bo)
        } else {
            let (dm, r) = self.jdn();
            Err(format!("L{}:{}: expected identifier, got {:?}", dm, r, self.amm()))
        }
    }

    

    fn fqf(&mut self) -> Result<Dj, String> {
        self.expect(&TokenKind::Ajn)?;
        let mut boq = Vec::new();
        while !self.aoi(&TokenKind::Yj) && !self.aoi(&TokenKind::Im) {
            boq.push(self.vdu()?);
        }
        self.expect(&TokenKind::Yj)?;
        Ok(Dj { boq })
    }

    fn vdu(&mut self) -> Result<Stmt, String> {
        match self.amm() {
            TokenKind::Pu => self.vcu(),
            TokenKind::Hd => self.vdk(),
            TokenKind::Gx => self.lsn(),
            TokenKind::La => self.vem(),
            TokenKind::Ll => self.vcj(),
            TokenKind::Pz => self.vcv(),
            TokenKind::Vr => { self.nb(); self.dql(); Ok(Stmt::Vr) }
            TokenKind::Cg => { self.nb(); self.dql(); Ok(Stmt::Cg) }
            _ => {
                let expr = self.bey()?;
                
                match self.amm() {
                    TokenKind::Eq => {
                        self.nb();
                        let bn = self.bey()?;
                        self.dql();
                        Ok(Stmt::Vk { cd: expr, bn })
                    }
                    TokenKind::Bpd => { self.nb(); let p = self.bey()?; self.dql(); Ok(Stmt::Xw { op: BinOp::Add, cd: expr, bn: p }) }
                    TokenKind::Bmg => { self.nb(); let p = self.bey()?; self.dql(); Ok(Stmt::Xw { op: BinOp::Sub, cd: expr, bn: p }) }
                    TokenKind::Bti => { self.nb(); let p = self.bey()?; self.dql(); Ok(Stmt::Xw { op: BinOp::Mul, cd: expr, bn: p }) }
                    TokenKind::Bsy => { self.nb(); let p = self.bey()?; self.dql(); Ok(Stmt::Xw { op: BinOp::Div, cd: expr, bn: p }) }
                    _ => {
                        self.dql();
                        Ok(Stmt::Expr(expr))
                    }
                }
            }
        }
    }

    fn vcu(&mut self) -> Result<Stmt, String> {
        self.expect(&TokenKind::Pu)?;
        let oos = if self.aoi(&TokenKind::Bmy) { self.nb(); true } else { false };
        let j = self.fqh()?;
        let ty = if self.aoi(&TokenKind::Ahb) {
            self.nb();
            Some(self.gov()?)
        } else {
            None
        };
        let init = if self.aoi(&TokenKind::Eq) {
            self.nb();
            Some(self.bey()?)
        } else {
            None
        };
        self.dql();
        Ok(Stmt::Pu { j, oos, ty, init })
    }

    fn vdk(&mut self) -> Result<Stmt, String> {
        self.expect(&TokenKind::Hd)?;
        let ap = if self.aoi(&TokenKind::Ayo) || self.aoi(&TokenKind::Yj) {
            None
        } else {
            Some(self.bey()?)
        };
        self.dql();
        Ok(Stmt::Hd(ap))
    }

    fn lsn(&mut self) -> Result<Stmt, String> {
        self.expect(&TokenKind::Gx)?;
        let mo = self.bey()?;
        let cne = self.fqf()?;
        let ckc = if self.aoi(&TokenKind::Bfw) {
            self.nb();
            if self.aoi(&TokenKind::Gx) {
                
                let skd = self.lsn()?;
                Some(Dj { boq: alloc::vec![skd] })
            } else {
                Some(self.fqf()?)
            }
        } else {
            None
        };
        Ok(Stmt::Gx { mo, cne, ckc })
    }

    fn vem(&mut self) -> Result<Stmt, String> {
        self.expect(&TokenKind::La)?;
        let mo = self.bey()?;
        let gj = self.fqf()?;
        Ok(Stmt::La { mo, gj })
    }

    fn vcj(&mut self) -> Result<Stmt, String> {
        self.expect(&TokenKind::Ll)?;
        let bfp = self.fqh()?;
        self.expect(&TokenKind::Bjn)?;
        let iter = self.bey()?;
        let gj = self.fqf()?;
        Ok(Stmt::Ll { bfp, iter, gj })
    }

    fn vcv(&mut self) -> Result<Stmt, String> {
        self.expect(&TokenKind::Pz)?;
        let gj = self.fqf()?;
        Ok(Stmt::Pz(gj))
    }

    fn dql(&mut self) {
        if self.aoi(&TokenKind::Ayo) { self.nb(); }
    }

    

    fn bey(&mut self) -> Result<Expr, String> {
        self.lsr()
    }

    fn lsr(&mut self) -> Result<Expr, String> {
        let mut fd = self.hui()?;
        while self.aoi(&TokenKind::Fx) {
            self.nb();
            let hw = self.hui()?;
            fd = Expr::BinOp { op: BinOp::Fx, fd: Box::new(fd), hw: Box::new(hw) };
        }
        Ok(fd)
    }

    fn hui(&mut self) -> Result<Expr, String> {
        let mut fd = self.huj()?;
        while self.aoi(&TokenKind::Ex) {
            self.nb();
            let hw = self.huj()?;
            fd = Expr::BinOp { op: BinOp::Ex, fd: Box::new(fd), hw: Box::new(hw) };
        }
        Ok(fd)
    }

    fn huj(&mut self) -> Result<Expr, String> {
        let mut fd = self.oub()?;
        loop {
            let op = match self.amm() {
                TokenKind::Bfz => BinOp::Eq,
                TokenKind::Xu => BinOp::Xu,
                TokenKind::Lt => BinOp::Lt,
                TokenKind::Jn => BinOp::Jn,
                TokenKind::Xm => BinOp::Xm,
                TokenKind::Wx => BinOp::Wx,
                _ => break,
            };
            self.nb();
            let hw = self.oub()?;
            fd = Expr::BinOp { op, fd: Box::new(fd), hw: Box::new(hw) };
        }
        Ok(fd)
    }

    fn oub(&mut self) -> Result<Expr, String> {
        let mut fd = self.got()?;
        loop {
            let op = match self.amm() {
                TokenKind::Bbs => BinOp::Vm,
                TokenKind::Yc => BinOp::Vn,
                TokenKind::Bdj => BinOp::Vo,
                TokenKind::Ob => BinOp::Ob,
                TokenKind::Oc => BinOp::Oc,
                _ => break,
            };
            self.nb();
            let hw = self.got()?;
            fd = Expr::BinOp { op, fd: Box::new(fd), hw: Box::new(hw) };
        }
        Ok(fd)
    }

    fn got(&mut self) -> Result<Expr, String> {
        let mut fd = self.huk()?;
        loop {
            let op = match self.amm() {
                TokenKind::Yd => BinOp::Add,
                TokenKind::Tm => BinOp::Sub,
                _ => break,
            };
            self.nb();
            let hw = self.huk()?;
            fd = Expr::BinOp { op, fd: Box::new(fd), hw: Box::new(hw) };
        }
        Ok(fd)
    }

    fn huk(&mut self) -> Result<Expr, String> {
        let mut fd = self.fqk()?;
        loop {
            let op = match self.amm() {
                TokenKind::And => BinOp::Mul,
                TokenKind::Bsx => BinOp::Div,
                TokenKind::Qk => BinOp::Xp,
                _ => break,
            };
            self.nb();
            let hw = self.fqk()?;
            fd = Expr::BinOp { op, fd: Box::new(fd), hw: Box::new(hw) };
        }
        Ok(fd)
    }

    fn fqk(&mut self) -> Result<Expr, String> {
        match self.amm() {
            TokenKind::Tm => {
                self.nb();
                let expr = self.hum()?;
                Ok(Expr::UnaryOp { op: UnaryOp::Neg, expr: Box::new(expr) })
            }
            TokenKind::Np => {
                self.nb();
                let expr = self.hum()?;
                Ok(Expr::UnaryOp { op: UnaryOp::Np, expr: Box::new(expr) })
            }
            _ => self.hum(),
        }
    }

    fn hum(&mut self) -> Result<Expr, String> {
        let mut expr = self.lsu()?;
        loop {
            match self.amm() {
                TokenKind::Ajo => {
                    self.nb();
                    let index = self.bey()?;
                    self.expect(&TokenKind::Aed)?;
                    expr = Expr::Index { array: Box::new(expr), index: Box::new(index) };
                }
                TokenKind::Bew => {
                    self.nb();
                    let buj = self.fqh()?;
                    expr = Expr::Asg { uww: Box::new(expr), buj };
                }
                TokenKind::Bca => {
                    self.nb();
                    let ty = self.gov()?;
                    expr = Expr::Apu { expr: Box::new(expr), ty };
                }
                TokenKind::Bex => {
                    self.nb();
                    let ci = self.got()?;
                    expr = Expr::Nt { ay: Box::new(expr), ci: Box::new(ci) };
                }
                _ => break,
            }
        }
        Ok(expr)
    }

    fn lsu(&mut self) -> Result<Expr, String> {
        match self.amm().clone() {
            TokenKind::Ta(p) => { let ap = p; self.nb(); Ok(Expr::Ta(ap)) }
            TokenKind::Wq(p) => { let ap = p; self.nb(); Ok(Expr::Wq(ap)) }
            TokenKind::Yw(e) => { let ap = e.clone(); self.nb(); Ok(Expr::Yw(ap)) }
            TokenKind::Rp(o) => { let ap = o; self.nb(); Ok(Expr::Rp(ap)) }
            TokenKind::Kq(j) => {
                let bo = j.clone();
                self.nb();
                
                if self.aoi(&TokenKind::Kr) {
                    self.nb();
                    let mut n = Vec::new();
                    while !self.aoi(&TokenKind::Jv) && !self.aoi(&TokenKind::Im) {
                        n.push(self.bey()?);
                        if self.aoi(&TokenKind::Aar) { self.nb(); }
                    }
                    self.expect(&TokenKind::Jv)?;
                    Ok(Expr::En { ke: bo, n })
                } else {
                    Ok(Expr::Kq(bo))
                }
            }
            TokenKind::Kr => {
                self.nb();
                let expr = self.bey()?;
                self.expect(&TokenKind::Jv)?;
                Ok(expr)
            }
            TokenKind::Ajo => {
                self.nb();
                let mut hhx = Vec::new();
                while !self.aoi(&TokenKind::Aed) && !self.aoi(&TokenKind::Im) {
                    hhx.push(self.bey()?);
                    if self.aoi(&TokenKind::Aar) { self.nb(); }
                }
                self.expect(&TokenKind::Aed)?;
                Ok(Expr::U(hhx))
            }
            TokenKind::Ajn => {
                let block = self.fqf()?;
                
                Ok(Expr::Dj(block))
            }
            TokenKind::Gx => {
                
                let stmt = self.lsn()?;
                match stmt {
                    Stmt::Gx { mo, cne, ckc } => {
                        Ok(Expr::Dj(Dj { boq: alloc::vec![
                            Stmt::Gx { mo, cne, ckc }
                        ]}))
                    }
                    _ => unreachable!(),
                }
            }
            _ => {
                let (dm, r) = self.jdn();
                Err(format!("L{}:{}: unexpected token {:?}", dm, r, self.amm()))
            }
        }
    }
}


pub fn parse(eb: &[Token]) -> Result<Program, String> {
    let mut parser = Parser::new(eb.ip());
    parser.vdf()
}
