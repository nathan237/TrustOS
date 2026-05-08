




use alloc::string::String;
use alloc::vec::Vec;
use alloc::format;


#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    
    IntLit(i64),
    FloatLit(f64),
    StringLit(String),
    BoolLit(bool),

    
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

    
    TypeI64,
    TypeF64,
    TypeBool,
    TypeStr,
    TypeVoid,

    
    Plus,       
    Minus,      
    Star,       
    Slash,      
    Percent,    
    Eq,         
    EqEq,       
    NotEq,      
    Lt,         
    Gt,         
    LtEq,       
    GtEq,       
    And,        
    Or,         
    Not,        
    Ampersand,  
    Pipe,       
    Caret,      
    Shl,        
    Shr,        
    PlusEq,     
    MinusEq,    
    StarEq,     
    SlashEq,    

    
    LParen,     
    RParen,     
    LBrace,     
    RBrace,     
    LBracket,   
    RBracket,   

    
    Comma,      
    Semicolon,  
    Colon,      
    Arrow,      
    FatArrow,   
    Dot,        
    DotDot,     

    
    Eof,
}


#[derive(Debug, Clone)]
pub struct Token {
    pub kind: TokenKind,
    pub line: usize,
    pub col: usize,
}


pub fn crv(source: &str) -> Result<Vec<Token>, String> {
    let mut tokens = Vec::new();
    let chars: Vec<char> = source.chars().collect();
    let mut pos = 0;
    let mut line = 1;
    let mut col = 1;

    while pos < chars.len() {
        let ch = chars[pos];

        
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

        
        if ch == '/' && pos + 1 < chars.len() {
            if chars[pos + 1] == '/' {
                
                while pos < chars.len() && chars[pos] != '\n' { pos += 1; }
                continue;
            }
            if chars[pos + 1] == '*' {
                
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

        let afu = col;

        
        if ch.is_ascii_digit() {
            let start = pos;
            let mut gdu = false;
            while pos < chars.len() && (chars[pos].is_ascii_digit() || chars[pos] == '.' || chars[pos] == '_') {
                if chars[pos] == '.' {
                    if gdu { break; }
                    
                    if pos + 1 < chars.len() && chars[pos + 1] == '.' { break; }
                    gdu = true;
                }
                pos += 1;
                col += 1;
            }
            let rw: String = chars[start..pos].iter().filter(|&&c| c != '_').collect();
            if gdu {
                let val = gmi(&rw).map_err(|_| format!("L{}:{}: invalid float '{}'", line, afu, rw))?;
                tokens.push(Token { kind: TokenKind::FloatLit(val), line, col: afu });
            } else {
                let val = nqr(&rw).map_err(|_| format!("L{}:{}: invalid integer '{}'", line, afu, rw))?;
                tokens.push(Token { kind: TokenKind::IntLit(val), line, col: afu });
            }
            continue;
        }

        
        if ch == '"' {
            pos += 1; col += 1;
            let mut j = String::new();
            while pos < chars.len() && chars[pos] != '"' {
                if chars[pos] == '\\' && pos + 1 < chars.len() {
                    pos += 1; col += 1;
                    match chars[pos] {
                        'n' => j.push('\n'),
                        't' => j.push('\t'),
                        'r' => j.push('\r'),
                        '\\' => j.push('\\'),
                        '"' => j.push('"'),
                        '0' => j.push('\0'),
                        _ => { j.push('\\'); j.push(chars[pos]); }
                    }
                } else {
                    if chars[pos] == '\n' { line += 1; col = 0; }
                    j.push(chars[pos]);
                }
                pos += 1; col += 1;
            }
            if pos >= chars.len() {
                return Err(format!("L{}:{}: unterminated string", line, afu));
            }
            pos += 1; col += 1; 
            tokens.push(Token { kind: TokenKind::StringLit(j), line, col: afu });
            continue;
        }

        
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
                    _ => return Err(format!("L{}:{}: invalid escape in char", line, afu)),
                }
            } else if pos < chars.len() {
                chars[pos]
            } else {
                return Err(format!("L{}:{}: unterminated char", line, afu));
            };
            pos += 1; col += 1;
            if pos >= chars.len() || chars[pos] != '\'' {
                return Err(format!("L{}:{}: unterminated char literal", line, afu));
            }
            pos += 1; col += 1;
            tokens.push(Token { kind: TokenKind::IntLit(c as i64), line, col: afu });
            continue;
        }

        
        if ch.is_ascii_alphabetic() || ch == '_' {
            let start = pos;
            while pos < chars.len() && (chars[pos].is_ascii_alphanumeric() || chars[pos] == '_') {
                pos += 1; col += 1;
            }
            let fx: String = chars[start..pos].iter().collect();
            let kind = match fx.as_str() {
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
                _ => TokenKind::Ident(fx),
            };
            tokens.push(Token { kind, line, col: afu });
            continue;
        }

        
        if pos + 1 < chars.len() {
            let csc: String = chars[pos..pos + 2].iter().collect();
            let kind = match csc.as_str() {
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
                tokens.push(Token { kind: k, line, col: afu });
                pos += 2; col += 2;
                continue;
            }
        }

        
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
        tokens.push(Token { kind, line, col: afu });
        pos += 1; col += 1;
    }

    tokens.push(Token { kind: TokenKind::Eof, line, col });
    Ok(tokens)
}


fn nqr(j: &str) -> Result<i64, ()> {
    let mut val: i64 = 0;
    let mut neg = false;
    for (i, ch) in j.chars().enumerate() {
        if i == 0 && ch == '-' { neg = true; continue; }
        if !ch.is_ascii_digit() { return Err(()); }
        val = val.checked_mul(10).ok_or(())?;
        val = val.checked_add((ch as i64) - 48).ok_or(())?;
    }
    Ok(if neg { -val } else { val })
}


fn gmi(j: &str) -> Result<f64, ()> {
    let mut gda: f64 = 0.0;
    let mut hzu: f64 = 0.0;
    let mut dqe: f64 = 1.0;
    let mut ifz = false;
    let mut neg = false;

    for (i, ch) in j.chars().enumerate() {
        if i == 0 && ch == '-' { neg = true; continue; }
        if ch == '.' { ifz = true; continue; }
        if !ch.is_ascii_digit() { return Err(()); }
        let d = (ch as u8 - b'0') as f64;
        if ifz {
            dqe *= 10.0;
            hzu += d / dqe;
        } else {
            gda = gda * 10.0 + d;
        }
    }

    let val = gda + hzu;
    Ok(if neg { -val } else { val })
}
