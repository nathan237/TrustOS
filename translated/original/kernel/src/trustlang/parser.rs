//! TrustLang Parser — Recursive descent parser producing an AST
//!
//! Supports: functions, let bindings, if/else, while, for, return,
//! arithmetic, comparisons, function calls, arrays, structs.

use alloc::string::String;
use alloc::vec::Vec;
use alloc::boxed::Box;
use alloc::format;

use super::lexer::{Token, TokenKind};

// ─── AST Types ──────────────────────────────────────────────────────────

/// A complete program
#[derive(Debug, Clone)]
pub struct Program {
    pub items: Vec<Item>,
}

/// Top-level items
#[derive(Debug, Clone)]
pub enum Item {
    Function(FnDecl),
    Struct(StructDecl),
}

/// Function declaration
#[derive(Debug, Clone)]
pub struct FnDecl {
    pub name: String,
    pub params: Vec<(String, Type)>,
    pub ret_type: Type,
    pub body: Block,
}

/// Struct declaration
#[derive(Debug, Clone)]
pub struct StructDecl {
    pub name: String,
    pub fields: Vec<(String, Type)>,
}

/// Type annotation
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

/// A block of statements
#[derive(Debug, Clone)]
pub struct Block {
    pub stmts: Vec<Stmt>,
}

/// Statements
#[derive(Debug, Clone)]
pub enum Stmt {
    Let { name: String, mutable: bool, ty: Option<Type>, init: Option<Expr> },
    Assign { target: Expr, value: Expr },
    OpAssign { op: BinOp, target: Expr, value: Expr },
    Expr(Expr),
    Return(Option<Expr>),
    If { cond: Expr, then_block: Block, else_block: Option<Block> },
    While { cond: Expr, body: Block },
    For { var: String, iter: Expr, body: Block },
    Loop(Block),
    Break,
    Continue,
}

/// Expressions
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
    Block(Block),
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

// ─── Parser ─────────────────────────────────────────────────────────────

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
        let tok = &self.tokens[self.pos];
        if self.pos < self.tokens.len() - 1 { self.pos += 1; }
        tok
    }

    fn expect(&mut self, expected: &TokenKind) -> Result<&Token, String> {
        if core::mem::discriminant(self.peek()) == core::mem::discriminant(expected) {
            Ok(self.advance())
        } else {
            let tok = &self.tokens[self.pos];
            Err(format!("L{}:{}: expected {:?}, got {:?}", tok.line, tok.col, expected, tok.kind))
        }
    }

    fn at(&self, kind: &TokenKind) -> bool {
        core::mem::discriminant(self.peek()) == core::mem::discriminant(kind)
    }

    fn line_col(&self) -> (usize, usize) {
        let t = &self.tokens[self.pos.min(self.tokens.len() - 1)];
        (t.line, t.col)
    }

    // ─── Top-level ──────────────────────────────────────────────────

    fn parse_program(&mut self) -> Result<Program, String> {
        let mut items = Vec::new();
        while !self.at(&TokenKind::Eof) {
            items.push(self.parse_item()?);
        }
        Ok(Program { items })
    }

    fn parse_item(&mut self) -> Result<Item, String> {
        match self.peek() {
            TokenKind::Fn => Ok(Item::Function(self.parse_fn()?)),
            TokenKind::Struct => Ok(Item::Struct(self.parse_struct()?)),
            _ => {
                let (l, c) = self.line_col();
                Err(format!("L{}:{}: expected 'fn' or 'struct', got {:?}", l, c, self.peek()))
            }
        }
    }

    fn parse_fn(&mut self) -> Result<FnDecl, String> {
        self.expect(&TokenKind::Fn)?;
        let name = self.parse_ident()?;
        self.expect(&TokenKind::LParen)?;
        let params = self.parse_params()?;
        self.expect(&TokenKind::RParen)?;

        let ret_type = if self.at(&TokenKind::Arrow) {
            self.advance();
            self.parse_type()?
        } else {
            Type::Void
        };

        let body = self.parse_block()?;
        Ok(FnDecl { name, params, ret_type, body })
    }

    fn parse_struct(&mut self) -> Result<StructDecl, String> {
        self.expect(&TokenKind::Struct)?;
        let name = self.parse_ident()?;
        self.expect(&TokenKind::LBrace)?;
        let mut fields = Vec::new();
        while !self.at(&TokenKind::RBrace) && !self.at(&TokenKind::Eof) {
            let fname = self.parse_ident()?;
            self.expect(&TokenKind::Colon)?;
            let fty = self.parse_type()?;
            fields.push((fname, fty));
            if self.at(&TokenKind::Comma) { self.advance(); }
        }
        self.expect(&TokenKind::RBrace)?;
        Ok(StructDecl { name, fields })
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
                let n = name.clone();
                self.advance();
                Ok(Type::Named(n))
            }
            _ => {
                let (l, c) = self.line_col();
                Err(format!("L{}:{}: expected type, got {:?}", l, c, self.peek()))
            }
        }
    }

    fn parse_ident(&mut self) -> Result<String, String> {
        if let TokenKind::Ident(name) = self.peek().clone() {
            let n = name.clone();
            self.advance();
            Ok(n)
        } else {
            let (l, c) = self.line_col();
            Err(format!("L{}:{}: expected identifier, got {:?}", l, c, self.peek()))
        }
    }

    // ─── Blocks & Statements ────────────────────────────────────────

    fn parse_block(&mut self) -> Result<Block, String> {
        self.expect(&TokenKind::LBrace)?;
        let mut stmts = Vec::new();
        while !self.at(&TokenKind::RBrace) && !self.at(&TokenKind::Eof) {
            stmts.push(self.parse_stmt()?);
        }
        self.expect(&TokenKind::RBrace)?;
        Ok(Block { stmts })
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
                // Check for assignment
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
        let mutable = if self.at(&TokenKind::Mut) { self.advance(); true } else { false };
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
        Ok(Stmt::Let { name, mutable, ty, init })
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
        let cond = self.parse_expr()?;
        let then_block = self.parse_block()?;
        let else_block = if self.at(&TokenKind::Else) {
            self.advance();
            if self.at(&TokenKind::If) {
                // else if → wrap in block
                let elif = self.parse_if()?;
                Some(Block { stmts: alloc::vec![elif] })
            } else {
                Some(self.parse_block()?)
            }
        } else {
            None
        };
        Ok(Stmt::If { cond, then_block, else_block })
    }

    fn parse_while(&mut self) -> Result<Stmt, String> {
        self.expect(&TokenKind::While)?;
        let cond = self.parse_expr()?;
        let body = self.parse_block()?;
        Ok(Stmt::While { cond, body })
    }

    fn parse_for(&mut self) -> Result<Stmt, String> {
        self.expect(&TokenKind::For)?;
        let var = self.parse_ident()?;
        self.expect(&TokenKind::In)?;
        let iter = self.parse_expr()?;
        let body = self.parse_block()?;
        Ok(Stmt::For { var, iter, body })
    }

    fn parse_loop(&mut self) -> Result<Stmt, String> {
        self.expect(&TokenKind::Loop)?;
        let body = self.parse_block()?;
        Ok(Stmt::Loop(body))
    }

    fn eat_semi(&mut self) {
        if self.at(&TokenKind::Semicolon) { self.advance(); }
    }

    // ─── Expressions (Pratt parser) ─────────────────────────────────

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
            TokenKind::StringLit(s) => { let val = s.clone(); self.advance(); Ok(Expr::StringLit(val)) }
            TokenKind::BoolLit(b) => { let val = b; self.advance(); Ok(Expr::BoolLit(val)) }
            TokenKind::Ident(name) => {
                let n = name.clone();
                self.advance();
                // Function call?
                if self.at(&TokenKind::LParen) {
                    self.advance();
                    let mut args = Vec::new();
                    while !self.at(&TokenKind::RParen) && !self.at(&TokenKind::Eof) {
                        args.push(self.parse_expr()?);
                        if self.at(&TokenKind::Comma) { self.advance(); }
                    }
                    self.expect(&TokenKind::RParen)?;
                    Ok(Expr::Call { func: n, args })
                } else {
                    Ok(Expr::Ident(n))
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
                let mut elems = Vec::new();
                while !self.at(&TokenKind::RBracket) && !self.at(&TokenKind::Eof) {
                    elems.push(self.parse_expr()?);
                    if self.at(&TokenKind::Comma) { self.advance(); }
                }
                self.expect(&TokenKind::RBracket)?;
                Ok(Expr::Array(elems))
            }
            TokenKind::LBrace => {
                let block = self.parse_block()?;
                // Return last expr from block
                Ok(Expr::Block(block))
            }
            TokenKind::If => {
                // If as expression
                let stmt = self.parse_if()?;
                match stmt {
                    Stmt::If { cond, then_block, else_block } => {
                        Ok(Expr::Block(Block { stmts: alloc::vec![
                            Stmt::If { cond, then_block, else_block }
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

/// Parse a token stream into an AST
pub fn parse(tokens: &[Token]) -> Result<Program, String> {
    let mut parser = Parser::new(tokens.to_vec());
    parser.parse_program()
}
