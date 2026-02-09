//! TrustLang Lexer — Tokenizer for Rust-like syntax
//!
//! Converts source text into a stream of tokens.
//! Handles: identifiers, keywords, numbers, strings, operators, delimiters.

use alloc::string::String;
use alloc::vec::Vec;
use alloc::format;

/// Token types
#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    // Literals
    IntLit(i64),
    FloatLit(f64),
    StringLit(String),
    BoolLit(bool),

    // Identifiers & keywords
    Ident(String),
    Fn,
    Let,
    Mut,
    If,
    Else,
    While,
    For,
    In,
    Return,
    Struct,
    Impl,
    Break,
    Continue,
    Loop,
    As,

    // Types
    TypeI64,
    TypeF64,
    TypeBool,
    TypeStr,
    TypeVoid,

    // Operators
    Plus,       // +
    Minus,      // -
    Star,       // *
    Slash,      // /
    Percent,    // %
    Eq,         // =
    EqEq,       // ==
    NotEq,      // !=
    Lt,         // <
    Gt,         // >
    LtEq,       // <=
    GtEq,       // >=
    And,        // &&
    Or,         // ||
    Not,        // !
    Ampersand,  // &
    Pipe,       // |
    Caret,      // ^
    Shl,        // <<
    Shr,        // >>
    PlusEq,     // +=
    MinusEq,    // -=
    StarEq,     // *=
    SlashEq,    // /=

    // Delimiters
    LParen,     // (
    RParen,     // )
    LBrace,     // {
    RBrace,     // }
    LBracket,   // [
    RBracket,   // ]

    // Punctuation
    Comma,      // ,
    Semicolon,  // ;
    Colon,      // :
    Arrow,      // ->
    FatArrow,   // =>
    Dot,        // .
    DotDot,     // ..

    // Special
    Eof,
}

/// A token with its position info
#[derive(Debug, Clone)]
pub struct Token {
    pub kind: TokenKind,
    pub line: usize,
    pub col: usize,
}

/// Tokenize TrustLang source into a token stream
pub fn tokenize(source: &str) -> Result<Vec<Token>, String> {
    let mut tokens = Vec::new();
    let chars: Vec<char> = source.chars().collect();
    let mut pos = 0;
    let mut line = 1;
    let mut col = 1;

    while pos < chars.len() {
        let ch = chars[pos];

        // Skip whitespace
        if ch == ' ' || ch == '\t' || ch == '\r' {
            pos += 1;
            col += 1;
            continue;
        }
        if ch == '\n' {
            pos += 1;
            line += 1;
            col = 1;
            continue;
        }

        // Skip comments
        if ch == '/' && pos + 1 < chars.len() {
            if chars[pos + 1] == '/' {
                // Line comment
                while pos < chars.len() && chars[pos] != '\n' { pos += 1; }
                continue;
            }
            if chars[pos + 1] == '*' {
                // Block comment
                pos += 2; col += 2;
                let mut depth = 1;
                while pos < chars.len() && depth > 0 {
                    if chars[pos] == '/' && pos + 1 < chars.len() && chars[pos + 1] == '*' {
                        depth += 1; pos += 1;
                    } else if chars[pos] == '*' && pos + 1 < chars.len() && chars[pos + 1] == '/' {
                        depth -= 1; pos += 1;
                    }
                    if chars[pos] == '\n' { line += 1; col = 0; }
                    pos += 1; col += 1;
                }
                continue;
            }
        }

        let start_col = col;

        // Numbers
        if ch.is_ascii_digit() {
            let start = pos;
            let mut is_float = false;
            while pos < chars.len() && (chars[pos].is_ascii_digit() || chars[pos] == '.' || chars[pos] == '_') {
                if chars[pos] == '.' {
                    if is_float { break; }
                    // Check if it's .. (range operator)
                    if pos + 1 < chars.len() && chars[pos + 1] == '.' { break; }
                    is_float = true;
                }
                pos += 1;
                col += 1;
            }
            let num_str: String = chars[start..pos].iter().filter(|&&c| c != '_').collect();
            if is_float {
                let val = parse_float(&num_str).map_err(|_| format!("L{}:{}: invalid float '{}'", line, start_col, num_str))?;
                tokens.push(Token { kind: TokenKind::FloatLit(val), line, col: start_col });
            } else {
                let val = parse_int(&num_str).map_err(|_| format!("L{}:{}: invalid integer '{}'", line, start_col, num_str))?;
                tokens.push(Token { kind: TokenKind::IntLit(val), line, col: start_col });
            }
            continue;
        }

        // Strings
        if ch == '"' {
            pos += 1; col += 1;
            let mut s = String::new();
            while pos < chars.len() && chars[pos] != '"' {
                if chars[pos] == '\\' && pos + 1 < chars.len() {
                    pos += 1; col += 1;
                    match chars[pos] {
                        'n' => s.push('\n'),
                        't' => s.push('\t'),
                        'r' => s.push('\r'),
                        '\\' => s.push('\\'),
                        '"' => s.push('"'),
                        '0' => s.push('\0'),
                        _ => { s.push('\\'); s.push(chars[pos]); }
                    }
                } else {
                    if chars[pos] == '\n' { line += 1; col = 0; }
                    s.push(chars[pos]);
                }
                pos += 1; col += 1;
            }
            if pos >= chars.len() {
                return Err(format!("L{}:{}: unterminated string", line, start_col));
            }
            pos += 1; col += 1; // skip closing "
            tokens.push(Token { kind: TokenKind::StringLit(s), line, col: start_col });
            continue;
        }

        // Char literals (treated as i64)
        if ch == '\'' {
            pos += 1; col += 1;
            let c = if pos < chars.len() && chars[pos] == '\\' {
                pos += 1; col += 1;
                match chars.get(pos) {
                    Some('n') => '\n',
                    Some('t') => '\t',
                    Some('r') => '\r',
                    Some('0') => '\0',
                    Some('\\') => '\\',
                    Some('\'') => '\'',
                    _ => return Err(format!("L{}:{}: invalid escape in char", line, start_col)),
                }
            } else if pos < chars.len() {
                chars[pos]
            } else {
                return Err(format!("L{}:{}: unterminated char", line, start_col));
            };
            pos += 1; col += 1;
            if pos >= chars.len() || chars[pos] != '\'' {
                return Err(format!("L{}:{}: unterminated char literal", line, start_col));
            }
            pos += 1; col += 1;
            tokens.push(Token { kind: TokenKind::IntLit(c as i64), line, col: start_col });
            continue;
        }

        // Identifiers & keywords
        if ch.is_ascii_alphabetic() || ch == '_' {
            let start = pos;
            while pos < chars.len() && (chars[pos].is_ascii_alphanumeric() || chars[pos] == '_') {
                pos += 1; col += 1;
            }
            let word: String = chars[start..pos].iter().collect();
            let kind = match word.as_str() {
                "fn" => TokenKind::Fn,
                "let" => TokenKind::Let,
                "mut" => TokenKind::Mut,
                "if" => TokenKind::If,
                "else" => TokenKind::Else,
                "while" => TokenKind::While,
                "for" => TokenKind::For,
                "in" => TokenKind::In,
                "return" => TokenKind::Return,
                "struct" => TokenKind::Struct,
                "impl" => TokenKind::Impl,
                "break" => TokenKind::Break,
                "continue" => TokenKind::Continue,
                "loop" => TokenKind::Loop,
                "as" => TokenKind::As,
                "true" => TokenKind::BoolLit(true),
                "false" => TokenKind::BoolLit(false),
                "i64" => TokenKind::TypeI64,
                "f64" => TokenKind::TypeF64,
                "bool" => TokenKind::TypeBool,
                "str" => TokenKind::TypeStr,
                _ => TokenKind::Ident(word),
            };
            tokens.push(Token { kind, line, col: start_col });
            continue;
        }

        // Two-char operators
        if pos + 1 < chars.len() {
            let two: String = chars[pos..pos + 2].iter().collect();
            let kind = match two.as_str() {
                "==" => Some(TokenKind::EqEq),
                "!=" => Some(TokenKind::NotEq),
                "<=" => Some(TokenKind::LtEq),
                ">=" => Some(TokenKind::GtEq),
                "&&" => Some(TokenKind::And),
                "||" => Some(TokenKind::Or),
                "+=" => Some(TokenKind::PlusEq),
                "-=" => Some(TokenKind::MinusEq),
                "*=" => Some(TokenKind::StarEq),
                "/=" => Some(TokenKind::SlashEq),
                "->" => Some(TokenKind::Arrow),
                "=>" => Some(TokenKind::FatArrow),
                "<<" => Some(TokenKind::Shl),
                ">>" => Some(TokenKind::Shr),
                ".." => Some(TokenKind::DotDot),
                _ => None,
            };
            if let Some(k) = kind {
                tokens.push(Token { kind: k, line, col: start_col });
                pos += 2; col += 2;
                continue;
            }
        }

        // Single-char operators & delimiters
        let kind = match ch {
            '+' => TokenKind::Plus,
            '-' => TokenKind::Minus,
            '*' => TokenKind::Star,
            '/' => TokenKind::Slash,
            '%' => TokenKind::Percent,
            '=' => TokenKind::Eq,
            '<' => TokenKind::Lt,
            '>' => TokenKind::Gt,
            '!' => TokenKind::Not,
            '&' => TokenKind::Ampersand,
            '|' => TokenKind::Pipe,
            '^' => TokenKind::Caret,
            '(' => TokenKind::LParen,
            ')' => TokenKind::RParen,
            '{' => TokenKind::LBrace,
            '}' => TokenKind::RBrace,
            '[' => TokenKind::LBracket,
            ']' => TokenKind::RBracket,
            ',' => TokenKind::Comma,
            ';' => TokenKind::Semicolon,
            ':' => TokenKind::Colon,
            '.' => TokenKind::Dot,
            _ => return Err(format!("L{}:{}: unexpected character '{}'", line, col, ch)),
        };
        tokens.push(Token { kind, line, col: start_col });
        pos += 1; col += 1;
    }

    tokens.push(Token { kind: TokenKind::Eof, line, col });
    Ok(tokens)
}

/// Parse integer (no stdlib)
fn parse_int(s: &str) -> Result<i64, ()> {
    let mut val: i64 = 0;
    let mut neg = false;
    for (i, ch) in s.chars().enumerate() {
        if i == 0 && ch == '-' { neg = true; continue; }
        if !ch.is_ascii_digit() { return Err(()); }
        val = val.checked_mul(10).ok_or(())?;
        val = val.checked_add((ch as i64) - 48).ok_or(())?;
    }
    Ok(if neg { -val } else { val })
}

/// Parse float (simple, no stdlib)
fn parse_float(s: &str) -> Result<f64, ()> {
    let mut int_part: f64 = 0.0;
    let mut frac_part: f64 = 0.0;
    let mut frac_div: f64 = 1.0;
    let mut in_frac = false;
    let mut neg = false;

    for (i, ch) in s.chars().enumerate() {
        if i == 0 && ch == '-' { neg = true; continue; }
        if ch == '.' { in_frac = true; continue; }
        if !ch.is_ascii_digit() { return Err(()); }
        let d = (ch as u8 - b'0') as f64;
        if in_frac {
            frac_div *= 10.0;
            frac_part += d / frac_div;
        } else {
            int_part = int_part * 10.0 + d;
        }
    }

    let val = int_part + frac_part;
    Ok(if neg { -val } else { val })
}
