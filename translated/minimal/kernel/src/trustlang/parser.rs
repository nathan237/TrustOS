




use alloc::string::String;
use alloc::vec::Vec;
use alloc::boxed::Box;
use alloc::format;

use super::lexer::{Token, TokenKind};




#[derive(Debug, Clone)]
pub struct Program {
    pub items: Vec<Item>,
}


#[derive(Debug, Clone)]
pub enum Item {
    Aq(Md),
    Struct(Vf),
}


#[derive(Debug, Clone)]
pub struct Md {
    pub name: String,
    pub params: Vec<(String, Type)>,
    pub jan: Type,
    pub body: Bl,
}


#[derive(Debug, Clone)]
pub struct Vf {
    pub name: String,
    pub fields: Vec<(String, Type)>,
}


#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    I64,
    F64,
    Bool,
    Str,
    Void,
    Array(Box<Type>),
    Named(String),
}


#[derive(Debug, Clone)]
pub struct Bl {
    pub stmts: Vec<Stmt>,
}


#[derive(Debug, Clone)]
pub enum Stmt {
    Let { name: String, ipd: bool, ty: Option<Type>, init: Option<Expr> },
    Assign { target: Expr, value: Expr },
    OpAssign { op: BinOp, target: Expr, value: Expr },
    Expr(Expr),
    Return(Option<Expr>),
    If { fc: Expr, avj: Bl, atp: Option<Bl> },
    While { fc: Expr, body: Bl },
    For { ael: String, iter: Expr, body: Bl },
    Loop(Bl),
    Break,
    Continue,
}


#[derive(Debug, Clone)]
pub enum Expr {
    IntLit(i64),
    FloatLit(f64),
    StringLit(String),
    BoolLit(bool),
    Ident(String),
    BinOp { op: BinOp, left: Box<Expr>, right: Box<Expr> },
    UnaryOp { op: UnaryOp, expr: Box<Expr> },
    Call { func: String, args: Vec<Expr> },
    Index { array: Box<Expr>, index: Box<Expr> },
    Field { object: Box<Expr>, field: String },
    Array(Vec<Expr>),
    Range { start: Box<Expr>, end: Box<Expr> },
    Cast { expr: Box<Expr>, ty: Type },
    Bl(Bl),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BinOp {
    Add, Sub, Mul, Div, Mod,
    Eq, NotEq, Lt, Gt, LtEq, GtEq,
    And, Or,
    BitAnd, BitOr, BitXor, Shl, Shr,
}

#[derive(Debug, Clone, Copy)]
pub enum UnaryOp { Neg, Not }



struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, pos: 0 }
    }

    fn peek(&self) -> &TokenKind {
        &self.tokens.get(self.pos).map(|t| &t.kind).unwrap_or(&TokenKind::Eof)
    }

    fn advance(&mut self) -> &Token {
        let asl = &self.tokens[self.pos];
        if self.pos < self.tokens.len() - 1 { self.pos += 1; }
        asl
    }

    fn expect(&mut self, expected: &TokenKind) -> Result<&Token, String> {
        if core::mem::discriminant(self.peek()) == core::mem::discriminant(expected) {
            Ok(self.advance())
        } else {
            let asl = &self.tokens[self.pos];
            Err(format!("L{}:{}: expected {:?}, got {:?}", asl.line, asl.col, expected, asl.kind))
        }
    }

    fn at(&self, kind: &TokenKind) -> bool {
        core::mem::discriminant(self.peek()) == core::mem::discriminant(kind)
    }

    fn line_col(&self) -> (usize, usize) {
        let t = &self.tokens[self.pos.min(self.tokens.len() - 1)];
        (t.line, t.col)
    }

    

    fn parse_program(&mut self) -> Result<Program, String> {
        let mut items = Vec::new();
        while !self.at(&TokenKind::Eof) {
            items.push(self.parse_item()?);
        }
        Ok(Program { items })
    }

    fn parse_item(&mut self) -> Result<Item, String> {
        match self.peek() {
            TokenKind::Fn => Ok(Item::Aq(self.parse_fn()?)),
            TokenKind::Struct => Ok(Item::Struct(self.parse_struct()?)),
            _ => {
                let (l, c) = self.line_col();
                Err(format!("L{}:{}: expected 'fn' or 'struct', got {:?}", l, c, self.peek()))
            }
        }
    }

    fn parse_fn(&mut self) -> Result<Md, String> {
        self.expect(&TokenKind::Fn)?;
        let name = self.parse_ident()?;
        self.expect(&TokenKind::LParen)?;
        let params = self.parse_params()?;
        self.expect(&TokenKind::RParen)?;

        let jan = if self.at(&TokenKind::Arrow) {
            self.advance();
            self.parse_type()?
        } else {
            Type::Void
        };

        let body = self.parse_block()?;
        Ok(Md { name, params, jan, body })
    }

    fn parse_struct(&mut self) -> Result<Vf, String> {
        self.expect(&TokenKind::Struct)?;
        let name = self.parse_ident()?;
        self.expect(&TokenKind::LBrace)?;
        let mut fields = Vec::new();
        while !self.at(&TokenKind::RBrace) && !self.at(&TokenKind::Eof) {
            let bsr = self.parse_ident()?;
            self.expect(&TokenKind::Colon)?;
            let lzx = self.parse_type()?;
            fields.push((bsr, lzx));
            if self.at(&TokenKind::Comma) { self.advance(); }
        }
        self.expect(&TokenKind::RBrace)?;
        Ok(Vf { name, fields })
    }

    fn parse_params(&mut self) -> Result<Vec<(String, Type)>, String> {
        let mut params = Vec::new();
        while !self.at(&TokenKind::RParen) && !self.at(&TokenKind::Eof) {
            let name = self.parse_ident()?;
            self.expect(&TokenKind::Colon)?;
            let ty = self.parse_type()?;
            params.push((name, ty));
            if self.at(&TokenKind::Comma) { self.advance(); }
        }
        Ok(params)
    }

    fn parse_type(&mut self) -> Result<Type, String> {
        match self.peek().clone() {
            TokenKind::TypeI64 => { self.advance(); Ok(Type::I64) }
            TokenKind::TypeF64 => { self.advance(); Ok(Type::F64) }
            TokenKind::TypeBool => { self.advance(); Ok(Type::Bool) }
            TokenKind::TypeStr => { self.advance(); Ok(Type::Str) }
            TokenKind::LBracket => {
                self.advance();
                let inner = self.parse_type()?;
                self.expect(&TokenKind::RBracket)?;
                Ok(Type::Array(Box::new(inner)))
            }
            TokenKind::Ident(name) => {
                let ae = name.clone();
                self.advance();
                Ok(Type::Named(ae))
            }
            _ => {
                let (l, c) = self.line_col();
                Err(format!("L{}:{}: expected type, got {:?}", l, c, self.peek()))
            }
        }
    }

    fn parse_ident(&mut self) -> Result<String, String> {
        if let TokenKind::Ident(name) = self.peek().clone() {
            let ae = name.clone();
            self.advance();
            Ok(ae)
        } else {
            let (l, c) = self.line_col();
            Err(format!("L{}:{}: expected identifier, got {:?}", l, c, self.peek()))
        }
    }

    

    fn parse_block(&mut self) -> Result<Bl, String> {
        self.expect(&TokenKind::LBrace)?;
        let mut stmts = Vec::new();
        while !self.at(&TokenKind::RBrace) && !self.at(&TokenKind::Eof) {
            stmts.push(self.parse_stmt()?);
        }
        self.expect(&TokenKind::RBrace)?;
        Ok(Bl { stmts })
    }

    fn parse_stmt(&mut self) -> Result<Stmt, String> {
        match self.peek() {
            TokenKind::Let => self.parse_let(),
            TokenKind::Return => self.parse_return(),
            TokenKind::If => self.parse_if(),
            TokenKind::While => self.parse_while(),
            TokenKind::For => self.parse_for(),
            TokenKind::Loop => self.parse_loop(),
            TokenKind::Break => { self.advance(); self.eat_semi(); Ok(Stmt::Break) }
            TokenKind::Continue => { self.advance(); self.eat_semi(); Ok(Stmt::Continue) }
            _ => {
                let expr = self.parse_expr()?;
                
                match self.peek() {
                    TokenKind::Eq => {
                        self.advance();
                        let value = self.parse_expr()?;
                        self.eat_semi();
                        Ok(Stmt::Assign { target: expr, value })
                    }
                    TokenKind::PlusEq => { self.advance(); let v = self.parse_expr()?; self.eat_semi(); Ok(Stmt::OpAssign { op: BinOp::Add, target: expr, value: v }) }
                    TokenKind::MinusEq => { self.advance(); let v = self.parse_expr()?; self.eat_semi(); Ok(Stmt::OpAssign { op: BinOp::Sub, target: expr, value: v }) }
                    TokenKind::StarEq => { self.advance(); let v = self.parse_expr()?; self.eat_semi(); Ok(Stmt::OpAssign { op: BinOp::Mul, target: expr, value: v }) }
                    TokenKind::SlashEq => { self.advance(); let v = self.parse_expr()?; self.eat_semi(); Ok(Stmt::OpAssign { op: BinOp::Div, target: expr, value: v }) }
                    _ => {
                        self.eat_semi();
                        Ok(Stmt::Expr(expr))
                    }
                }
            }
        }
    }

    fn parse_let(&mut self) -> Result<Stmt, String> {
        self.expect(&TokenKind::Let)?;
        let ipd = if self.at(&TokenKind::Mut) { self.advance(); true } else { false };
        let name = self.parse_ident()?;
        let ty = if self.at(&TokenKind::Colon) {
            self.advance();
            Some(self.parse_type()?)
        } else {
            None
        };
        let init = if self.at(&TokenKind::Eq) {
            self.advance();
            Some(self.parse_expr()?)
        } else {
            None
        };
        self.eat_semi();
        Ok(Stmt::Let { name, ipd, ty, init })
    }

    fn parse_return(&mut self) -> Result<Stmt, String> {
        self.expect(&TokenKind::Return)?;
        let val = if self.at(&TokenKind::Semicolon) || self.at(&TokenKind::RBrace) {
            None
        } else {
            Some(self.parse_expr()?)
        };
        self.eat_semi();
        Ok(Stmt::Return(val))
    }

    fn parse_if(&mut self) -> Result<Stmt, String> {
        self.expect(&TokenKind::If)?;
        let fc = self.parse_expr()?;
        let avj = self.parse_block()?;
        let atp = if self.at(&TokenKind::Else) {
            self.advance();
            if self.at(&TokenKind::If) {
                
                let loz = self.parse_if()?;
                Some(Bl { stmts: alloc::vec![loz] })
            } else {
                Some(self.parse_block()?)
            }
        } else {
            None
        };
        Ok(Stmt::If { fc, avj, atp })
    }

    fn parse_while(&mut self) -> Result<Stmt, String> {
        self.expect(&TokenKind::While)?;
        let fc = self.parse_expr()?;
        let body = self.parse_block()?;
        Ok(Stmt::While { fc, body })
    }

    fn parse_for(&mut self) -> Result<Stmt, String> {
        self.expect(&TokenKind::For)?;
        let ael = self.parse_ident()?;
        self.expect(&TokenKind::In)?;
        let iter = self.parse_expr()?;
        let body = self.parse_block()?;
        Ok(Stmt::For { ael, iter, body })
    }

    fn parse_loop(&mut self) -> Result<Stmt, String> {
        self.expect(&TokenKind::Loop)?;
        let body = self.parse_block()?;
        Ok(Stmt::Loop(body))
    }

    fn eat_semi(&mut self) {
        if self.at(&TokenKind::Semicolon) { self.advance(); }
    }

    

    fn parse_expr(&mut self) -> Result<Expr, String> {
        self.parse_or()
    }

    fn parse_or(&mut self) -> Result<Expr, String> {
        let mut left = self.parse_and()?;
        while self.at(&TokenKind::Or) {
            self.advance();
            let right = self.parse_and()?;
            left = Expr::BinOp { op: BinOp::Or, left: Box::new(left), right: Box::new(right) };
        }
        Ok(left)
    }

    fn parse_and(&mut self) -> Result<Expr, String> {
        let mut left = self.parse_comparison()?;
        while self.at(&TokenKind::And) {
            self.advance();
            let right = self.parse_comparison()?;
            left = Expr::BinOp { op: BinOp::And, left: Box::new(left), right: Box::new(right) };
        }
        Ok(left)
    }

    fn parse_comparison(&mut self) -> Result<Expr, String> {
        let mut left = self.parse_bitwise()?;
        loop {
            let op = match self.peek() {
                TokenKind::EqEq => BinOp::Eq,
                TokenKind::NotEq => BinOp::NotEq,
                TokenKind::Lt => BinOp::Lt,
                TokenKind::Gt => BinOp::Gt,
                TokenKind::LtEq => BinOp::LtEq,
                TokenKind::GtEq => BinOp::GtEq,
                _ => break,
            };
            self.advance();
            let right = self.parse_bitwise()?;
            left = Expr::BinOp { op, left: Box::new(left), right: Box::new(right) };
        }
        Ok(left)
    }

    fn parse_bitwise(&mut self) -> Result<Expr, String> {
        let mut left = self.parse_additive()?;
        loop {
            let op = match self.peek() {
                TokenKind::Ampersand => BinOp::BitAnd,
                TokenKind::Pipe => BinOp::BitOr,
                TokenKind::Caret => BinOp::BitXor,
                TokenKind::Shl => BinOp::Shl,
                TokenKind::Shr => BinOp::Shr,
                _ => break,
            };
            self.advance();
            let right = self.parse_additive()?;
            left = Expr::BinOp { op, left: Box::new(left), right: Box::new(right) };
        }
        Ok(left)
    }

    fn parse_additive(&mut self) -> Result<Expr, String> {
        let mut left = self.parse_multiplicative()?;
        loop {
            let op = match self.peek() {
                TokenKind::Plus => BinOp::Add,
                TokenKind::Minus => BinOp::Sub,
                _ => break,
            };
            self.advance();
            let right = self.parse_multiplicative()?;
            left = Expr::BinOp { op, left: Box::new(left), right: Box::new(right) };
        }
        Ok(left)
    }

    fn parse_multiplicative(&mut self) -> Result<Expr, String> {
        let mut left = self.parse_unary()?;
        loop {
            let op = match self.peek() {
                TokenKind::Star => BinOp::Mul,
                TokenKind::Slash => BinOp::Div,
                TokenKind::Percent => BinOp::Mod,
                _ => break,
            };
            self.advance();
            let right = self.parse_unary()?;
            left = Expr::BinOp { op, left: Box::new(left), right: Box::new(right) };
        }
        Ok(left)
    }

    fn parse_unary(&mut self) -> Result<Expr, String> {
        match self.peek() {
            TokenKind::Minus => {
                self.advance();
                let expr = self.parse_postfix()?;
                Ok(Expr::UnaryOp { op: UnaryOp::Neg, expr: Box::new(expr) })
            }
            TokenKind::Not => {
                self.advance();
                let expr = self.parse_postfix()?;
                Ok(Expr::UnaryOp { op: UnaryOp::Not, expr: Box::new(expr) })
            }
            _ => self.parse_postfix(),
        }
    }

    fn parse_postfix(&mut self) -> Result<Expr, String> {
        let mut expr = self.parse_primary()?;
        loop {
            match self.peek() {
                TokenKind::LBracket => {
                    self.advance();
                    let index = self.parse_expr()?;
                    self.expect(&TokenKind::RBracket)?;
                    expr = Expr::Index { array: Box::new(expr), index: Box::new(index) };
                }
                TokenKind::Dot => {
                    self.advance();
                    let field = self.parse_ident()?;
                    expr = Expr::Field { object: Box::new(expr), field };
                }
                TokenKind::As => {
                    self.advance();
                    let ty = self.parse_type()?;
                    expr = Expr::Cast { expr: Box::new(expr), ty };
                }
                TokenKind::DotDot => {
                    self.advance();
                    let end = self.parse_additive()?;
                    expr = Expr::Range { start: Box::new(expr), end: Box::new(end) };
                }
                _ => break,
            }
        }
        Ok(expr)
    }

    fn parse_primary(&mut self) -> Result<Expr, String> {
        match self.peek().clone() {
            TokenKind::IntLit(v) => { let val = v; self.advance(); Ok(Expr::IntLit(val)) }
            TokenKind::FloatLit(v) => { let val = v; self.advance(); Ok(Expr::FloatLit(val)) }
            TokenKind::StringLit(j) => { let val = j.clone(); self.advance(); Ok(Expr::StringLit(val)) }
            TokenKind::BoolLit(b) => { let val = b; self.advance(); Ok(Expr::BoolLit(val)) }
            TokenKind::Ident(name) => {
                let ae = name.clone();
                self.advance();
                
                if self.at(&TokenKind::LParen) {
                    self.advance();
                    let mut args = Vec::new();
                    while !self.at(&TokenKind::RParen) && !self.at(&TokenKind::Eof) {
                        args.push(self.parse_expr()?);
                        if self.at(&TokenKind::Comma) { self.advance(); }
                    }
                    self.expect(&TokenKind::RParen)?;
                    Ok(Expr::Call { func: ae, args })
                } else {
                    Ok(Expr::Ident(ae))
                }
            }
            TokenKind::LParen => {
                self.advance();
                let expr = self.parse_expr()?;
                self.expect(&TokenKind::RParen)?;
                Ok(expr)
            }
            TokenKind::LBracket => {
                self.advance();
                let mut doo = Vec::new();
                while !self.at(&TokenKind::RBracket) && !self.at(&TokenKind::Eof) {
                    doo.push(self.parse_expr()?);
                    if self.at(&TokenKind::Comma) { self.advance(); }
                }
                self.expect(&TokenKind::RBracket)?;
                Ok(Expr::Array(doo))
            }
            TokenKind::LBrace => {
                let block = self.parse_block()?;
                
                Ok(Expr::Bl(block))
            }
            TokenKind::If => {
                
                let stmt = self.parse_if()?;
                match stmt {
                    Stmt::If { fc, avj, atp } => {
                        Ok(Expr::Bl(Bl { stmts: alloc::vec![
                            Stmt::If { fc, avj, atp }
                        ]}))
                    }
                    _ => unreachable!(),
                }
            }
            _ => {
                let (l, c) = self.line_col();
                Err(format!("L{}:{}: unexpected token {:?}", l, c, self.peek()))
            }
        }
    }
}


pub fn parse(tokens: &[Token]) -> Result<Program, String> {
    let mut parser = Parser::new(tokens.to_vec());
    parser.parse_program()
}
