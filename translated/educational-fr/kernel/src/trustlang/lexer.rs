//! TrustLang Lexer — Tokenizer for Rust-like syntax
//!
//! Converts source text into a stream of tokens.
//! Handles: identifiers, keywords, numbers, strings, operators, delimiters.

use alloc::string::String;
use alloc::vec::Vec;
use alloc::format;

/// Token types
#[derive(Debug, Clone, PartialEq)]
// Énumération — un type qui peut être l'une de plusieurs variantes.
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
// Structure publique — visible à l'extérieur de ce module.
pub struct Token {
    pub kind: TokenKind,
    pub line: usize,
    pub column: usize,
}

/// Tokenize TrustLang source into a token stream
pub fn tokenize(source: &str) -> Result<Vec<Token>, String> {
    let mut tokens = Vec::new();
    let chars: Vec<char> = source.chars().collect();
    let mut position = 0;
    let mut line = 1;
    let mut column = 1;

    while position < chars.len() {
        let character = chars[position];

        // Skip whitespace
        if character == ' ' || character == '\t' || character == '\r' {
            position += 1;
            column += 1;
            continue;
        }
        if character == '\n' {
            position += 1;
            line += 1;
            column = 1;
            continue;
        }

        // Skip comments
        if character == '/' && position + 1 < chars.len() {
            if chars[position + 1] == '/' {
                // Line comment
                while position < chars.len() && chars[position] != '\n' { position += 1; }
                continue;
            }
            if chars[position + 1] == '*' {
                // Block comment
                position += 2; column += 2;
                let mut depth = 1;
                while position < chars.len() && depth > 0 {
                    if chars[position] == '/' && position + 1 < chars.len() && chars[position + 1] == '*' {
                        depth += 1; position += 1;
                    } else if chars[position] == '*' && position + 1 < chars.len() && chars[position + 1] == '/' {
                        depth -= 1; position += 1;
                    }
                    if chars[position] == '\n' { line += 1; column = 0; }
                    position += 1; column += 1;
                }
                continue;
            }
        }

        let start_column = column;

        // Numbers
        if character.is_ascii_digit() {
            let start = position;
            let mut is_float = false;
            while position < chars.len() && (chars[position].is_ascii_digit() || chars[position] == '.' || chars[position] == '_') {
                if chars[position] == '.' {
                    if is_float { break; }
                    // Check if it's .. (range operator)
                    if position + 1 < chars.len() && chars[position + 1] == '.' { break; }
                    is_float = true;
                }
                position += 1;
                column += 1;
            }
            let number_str: String = chars[start..position].iter().filter(|&&c| c != '_').collect();
            if is_float {
                let value = parse_float(&number_str).map_error(|_| format!("L{}:{}: invalid float '{}'", line, start_column, number_str))?;
                tokens.push(Token { kind: TokenKind::FloatLit(value), line, column: start_column });
            } else {
                let value = parse_int(&number_str).map_error(|_| format!("L{}:{}: invalid integer '{}'", line, start_column, number_str))?;
                tokens.push(Token { kind: TokenKind::IntLit(value), line, column: start_column });
            }
            continue;
        }

        // Strings
        if character == '"' {
            position += 1; column += 1;
            let mut s = String::new();
            while position < chars.len() && chars[position] != '"' {
                if chars[position] == '\\' && position + 1 < chars.len() {
                    position += 1; column += 1;
                                        // Correspondance de motifs — branchement exhaustif de Rust.
match chars[position] {
                        'n' => s.push('\n'),
                        't' => s.push('\t'),
                        'r' => s.push('\r'),
                        '\\' => s.push('\\'),
                        '"' => s.push('"'),
                        '0' => s.push('\0'),
                        _ => { s.push('\\'); s.push(chars[position]); }
                    }
                } else {
                    if chars[position] == '\n' { line += 1; column = 0; }
                    s.push(chars[position]);
                }
                position += 1; column += 1;
            }
            if position >= chars.len() {
                return Err(format!("L{}:{}: unterminated string", line, start_column));
            }
            position += 1; column += 1; // skip closing "
            tokens.push(Token { kind: TokenKind::StringLit(s), line, column: start_column });
            continue;
        }

        // Char literals (treated as i64)
        if character == '\'' {
            position += 1; column += 1;
            let c = if position < chars.len() && chars[position] == '\\' {
                position += 1; column += 1;
                                // Correspondance de motifs — branchement exhaustif de Rust.
match chars.get(position) {
                    Some('n') => '\n',
                    Some('t') => '\t',
                    Some('r') => '\r',
                    Some('0') => '\0',
                    Some('\\') => '\\',
                    Some('\'') => '\'',
                    _ => return Err(format!("L{}:{}: invalid escape in char", line, start_column)),
                }
            } else if position < chars.len() {
                chars[position]
            } else {
                return Err(format!("L{}:{}: unterminated char", line, start_column));
            };
            position += 1; column += 1;
            if position >= chars.len() || chars[position] != '\'' {
                return Err(format!("L{}:{}: unterminated char literal", line, start_column));
            }
            position += 1; column += 1;
            tokens.push(Token { kind: TokenKind::IntLit(c as i64), line, column: start_column });
            continue;
        }

        // Identifiers & keywords
        if character.is_ascii_alphabetic() || character == '_' {
            let start = position;
            while position < chars.len() && (chars[position].is_ascii_alphanumeric() || chars[position] == '_') {
                position += 1; column += 1;
            }
            let word: String = chars[start..position].iter().collect();
            let kind = // Correspondance de motifs — branchement exhaustif de Rust.
match word.as_str() {
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
            tokens.push(Token { kind, line, column: start_column });
            continue;
        }

        // Two-char operators
        if position + 1 < chars.len() {
            let two: String = chars[position..position + 2].iter().collect();
            let kind = // Correspondance de motifs — branchement exhaustif de Rust.
match two.as_str() {
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
                tokens.push(Token { kind: k, line, column: start_column });
                position += 2; column += 2;
                continue;
            }
        }

        // Single-char operators & delimiters
        let kind = // Correspondance de motifs — branchement exhaustif de Rust.
match character {
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
            _ => return Err(format!("L{}:{}: unexpected character '{}'", line, column, character)),
        };
        tokens.push(Token { kind, line, column: start_column });
        position += 1; column += 1;
    }

    tokens.push(Token { kind: TokenKind::Eof, line, column });
    Ok(tokens)
}

/// Parse integer (no stdlib)
fn parse_int(s: &str) -> Result<i64, ()> {
    let mut value: i64 = 0;
    let mut neg = false;
    for (i, character) in s.chars().enumerate() {
        if i == 0 && character == '-' { neg = true; continue; }
        if !character.is_ascii_digit() { return Err(()); }
        value = value.checked_mul(10).ok_or(())?;
        value = value.checked_add((character as i64) - 48).ok_or(())?;
    }
    Ok(if neg { -value } else { value })
}

/// Parse float (simple, no stdlib)
fn parse_float(s: &str) -> Result<f64, ()> {
    let mut int_part: f64 = 0.0;
    let mut frac_part: f64 = 0.0;
    let mut frac_div: f64 = 1.0;
    let mut in_frac = false;
    let mut neg = false;

    for (i, character) in s.chars().enumerate() {
        if i == 0 && character == '-' { neg = true; continue; }
        if character == '.' { in_frac = true; continue; }
        if !character.is_ascii_digit() { return Err(()); }
        let d = (character as u8 - b'0') as f64;
        if in_frac {
            frac_div *= 10.0;
            frac_part += d / frac_div;
        } else {
            int_part = int_part * 10.0 + d;
        }
    }

    let value = int_part + frac_part;
    Ok(if neg { -value } else { value })
}
