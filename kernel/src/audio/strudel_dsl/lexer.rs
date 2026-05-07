//! Tokeniser for the TrustStrudel method-chaining DSL.
//!
//! Recognises:
//!   - identifiers (`s`, `note`, `lpf`, `scale`, `sawtooth`, …)
//!   - string literals (`"bd sd hh cp"`)
//!   - integer + decimal numbers (`440`, `0.5`)
//!   - punctuation `.` `(` `)` `,` `[` `]`
//!   - line/eof terminators
//!
//! No allocations beyond the `Vec<Token>` and the owned `String` payloads.

use alloc::string::String;
use alloc::vec::Vec;

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    /// Identifier or keyword (e.g. `s`, `note`, `sawtooth`).
    Ident(String),
    /// Quoted string literal (mini-notation source typically).
    Str(String),
    /// Integer literal.
    Int(i64),
    /// Decimal literal stored in Q16.16 fixed-point.
    Fixed(i64),
    /// `.`
    Dot,
    /// `(`
    LParen,
    /// `)`
    RParen,
    /// `,`
    Comma,
    /// `[`
    LBracket,
    /// `]`
    RBracket,
    /// End of input.
    Eof,
}

#[derive(Debug)]
pub enum LexError {
    UnterminatedString,
    BadNumber,
    BadChar(char),
}

/// Convert a positive decimal "n.m" with up to 6 digits past the dot into Q16.16.
fn decimal_to_q16_16(int_part: i64, frac_str: &str) -> i64 {
    let mut frac_q = 0i64;
    let mut div = 1i64;
    for c in frac_str.chars().take(6) {
        if let Some(d) = c.to_digit(10) {
            frac_q = frac_q * 10 + d as i64;
            div *= 10;
        }
    }
    // frac_q / div is in [0,1). Convert to Q16.16:
    let frac_q16 = (frac_q << 16) / div.max(1);
    (int_part << 16) + frac_q16
}

/// Tokenise a DSL source line.
pub fn tokenize(src: &str) -> Result<Vec<Token>, LexError> {
    let mut out = Vec::with_capacity(16);
    let bytes = src.as_bytes();
    let mut i = 0;

    while i < bytes.len() {
        let c = bytes[i] as char;

        // Whitespace.
        if c == ' ' || c == '\t' || c == '\n' || c == '\r' {
            i += 1;
            continue;
        }

        // Line comment `//`
        if c == '/' && i + 1 < bytes.len() && bytes[i + 1] as char == '/' {
            while i < bytes.len() && bytes[i] as char != '\n' {
                i += 1;
            }
            continue;
        }

        match c {
            '.' => { out.push(Token::Dot); i += 1; continue; }
            '(' => { out.push(Token::LParen); i += 1; continue; }
            ')' => { out.push(Token::RParen); i += 1; continue; }
            ',' => { out.push(Token::Comma); i += 1; continue; }
            '[' => { out.push(Token::LBracket); i += 1; continue; }
            ']' => { out.push(Token::RBracket); i += 1; continue; }
            _ => {}
        }

        // String literal "..."
        if c == '"' {
            i += 1;
            let start = i;
            while i < bytes.len() && bytes[i] as char != '"' {
                i += 1;
            }
            if i >= bytes.len() {
                return Err(LexError::UnterminatedString);
            }
            let s = core::str::from_utf8(&bytes[start..i]).unwrap_or("");
            out.push(Token::Str(String::from(s)));
            i += 1; // skip closing quote
            continue;
        }

        // Number (optionally negative). We treat unary `-` as part of a number
        // when the previous token suggests an expression position.
        let is_neg_number = c == '-'
            && i + 1 < bytes.len()
            && (bytes[i + 1] as char).is_ascii_digit();

        if c.is_ascii_digit() || is_neg_number {
            let start = i;
            if is_neg_number { i += 1; }
            while i < bytes.len() && (bytes[i] as char).is_ascii_digit() {
                i += 1;
            }
            let mut is_fixed = false;
            if i < bytes.len() && bytes[i] as char == '.' {
                // Check this dot is decimal, not method-call. A decimal is followed by a digit.
                if i + 1 < bytes.len() && (bytes[i + 1] as char).is_ascii_digit() {
                    is_fixed = true;
                    i += 1;
                    while i < bytes.len() && (bytes[i] as char).is_ascii_digit() {
                        i += 1;
                    }
                }
            }
            let lit = core::str::from_utf8(&bytes[start..i]).unwrap_or("0");
            if is_fixed {
                if let Some(dot_at) = lit.find('.') {
                    let int_str = &lit[..dot_at];
                    let frac_str = &lit[dot_at + 1..];
                    let int_val: i64 = int_str.parse().map_err(|_| LexError::BadNumber)?;
                    let mag = decimal_to_q16_16(int_val.abs(), frac_str);
                    let signed = if int_val < 0 || int_str.starts_with('-') { -mag } else { mag };
                    out.push(Token::Fixed(signed));
                } else {
                    return Err(LexError::BadNumber);
                }
            } else {
                let v: i64 = lit.parse().map_err(|_| LexError::BadNumber)?;
                out.push(Token::Int(v));
            }
            continue;
        }

        // Identifier / keyword.
        if c == '_' || c.is_ascii_alphabetic() {
            let start = i;
            while i < bytes.len() {
                let ch = bytes[i] as char;
                if ch == '_' || ch.is_ascii_alphanumeric() || ch == '#' {
                    i += 1;
                } else {
                    break;
                }
            }
            let s = core::str::from_utf8(&bytes[start..i]).unwrap_or("");
            out.push(Token::Ident(String::from(s)));
            continue;
        }

        return Err(LexError::BadChar(c));
    }

    out.push(Token::Eof);
    Ok(out)
}
