//! Recursive-descent parser for the chained DSL.
//!
//! Grammar:
//!   chain    := primary ( `.` ident `(` arglist? `)` )*
//!   primary  := ident `(` arglist? `)` | number | string | list
//!   list     := `[` expr ( `,` expr )* `]`
//!   arglist  := expr ( `,` expr )*
//!
//! Each token is produced by `lexer::tokenize`.

use alloc::boxed::Box;
use alloc::string::String;
use alloc::vec::Vec;

use super::lexer::{tokenize, Token, LexError};

#[derive(Debug, Clone)]
pub enum Expr {
    /// String literal (mini-notation source typically).
    Str(String),
    /// Integer literal.
    Int(i64),
    /// Q16.16 fixed-point literal.
    Fixed(i64),
    /// Bare identifier (e.g. `sawtooth` used as a synth selector).
    Ident(String),
    /// `[ a, b, c ]` list.
    List(Vec<Expr>),
    /// `name(args...)`.
    Call { name: String, args: Vec<Expr> },
    /// `receiver.method(args...)`.
    Method { receiver: Box<Expr>, name: String, args: Vec<Expr> },
}

#[derive(Debug)]
pub enum ParseError {
    Lex(LexError),
    Unexpected(&'static str),
    EndOfInput,
}

impl From<LexError> for ParseError {
    fn from(e: LexError) -> Self { ParseError::Lex(e) }
}

struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, pos: 0 }
    }

    fn peek(&self) -> &Token {
        self.tokens.get(self.pos).unwrap_or(&Token::Eof)
    }

    fn bump(&mut self) -> Token {
        let t = self.tokens.get(self.pos).cloned().unwrap_or(Token::Eof);
        self.pos += 1;
        t
    }

    fn eat(&mut self, t: &Token) -> bool {
        if self.peek() == t {
            self.pos += 1;
            true
        } else {
            false
        }
    }

    fn expect(&mut self, t: &Token, msg: &'static str) -> Result<(), ParseError> {
        if self.eat(t) { Ok(()) } else { Err(ParseError::Unexpected(msg)) }
    }

    fn parse_expr(&mut self) -> Result<Expr, ParseError> {
        let mut e = self.parse_primary()?;
        loop {
            // Method call: `.name(args)` or just `.name`
            if matches!(self.peek(), Token::Dot) {
                self.bump();
                let name = match self.bump() {
                    Token::Ident(s) => s,
                    _ => return Err(ParseError::Unexpected("expected method name after '.'")),
                };
                let args = if matches!(self.peek(), Token::LParen) {
                    self.bump();
                    let a = self.parse_arglist()?;
                    self.expect(&Token::RParen, "expected ')' after method args")?;
                    a
                } else {
                    Vec::new()
                };
                e = Expr::Method { receiver: Box::new(e), name, args };
            } else {
                break;
            }
        }
        Ok(e)
    }

    fn parse_primary(&mut self) -> Result<Expr, ParseError> {
        match self.bump() {
            Token::Str(s) => Ok(Expr::Str(s)),
            Token::Int(n) => Ok(Expr::Int(n)),
            Token::Fixed(q) => Ok(Expr::Fixed(q)),
            Token::LBracket => {
                let mut items = Vec::new();
                if !matches!(self.peek(), Token::RBracket) {
                    items.push(self.parse_expr()?);
                    while self.eat(&Token::Comma) {
                        items.push(self.parse_expr()?);
                    }
                }
                self.expect(&Token::RBracket, "expected ']' to close list")?;
                Ok(Expr::List(items))
            }
            Token::Ident(name) => {
                if matches!(self.peek(), Token::LParen) {
                    self.bump();
                    let args = self.parse_arglist()?;
                    self.expect(&Token::RParen, "expected ')' after call args")?;
                    Ok(Expr::Call { name, args })
                } else {
                    Ok(Expr::Ident(name))
                }
            }
            Token::Eof => Err(ParseError::EndOfInput),
            _ => Err(ParseError::Unexpected("unexpected token in primary")),
        }
    }

    fn parse_arglist(&mut self) -> Result<Vec<Expr>, ParseError> {
        let mut out = Vec::new();
        if matches!(self.peek(), Token::RParen) {
            return Ok(out);
        }
        out.push(self.parse_expr()?);
        while self.eat(&Token::Comma) {
            out.push(self.parse_expr()?);
        }
        Ok(out)
    }
}

/// Parse a full DSL source line into an `Expr` chain.
pub fn parse(src: &str) -> Result<Expr, ParseError> {
    let tokens = tokenize(src)?;
    let mut p = Parser::new(tokens);
    let e = p.parse_expr()?;
    if !matches!(p.peek(), Token::Eof) {
        return Err(ParseError::Unexpected("trailing tokens"));
    }
    Ok(e)
}
