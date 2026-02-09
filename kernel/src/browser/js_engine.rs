//! JavaScript Engine (Minimal)
//!
//! A basic JavaScript interpreter for simple scripts.
//! Based on ECMAScript 5 with some ES6 features.
//! 
//! This is a simplified implementation for basic functionality.

use alloc::string::{String, ToString};
use alloc::vec::Vec;
use alloc::vec;
use alloc::boxed::Box;
use alloc::collections::BTreeMap;
use alloc::format;
use libm::{floor, ceil, round, sqrt, trunc};

/// JavaScript value types
#[derive(Debug, Clone)]
pub enum JsValue {
    Undefined,
    Null,
    Boolean(bool),
    Number(f64),
    String(String),
    Object(BTreeMap<String, JsValue>),
    Array(Vec<JsValue>),
    Function(String, Vec<String>, String), // name, params, body
    NativeFunction(String),
}

impl JsValue {
    /// Convert to boolean (truthy/falsy)
    pub fn to_bool(&self) -> bool {
        match self {
            JsValue::Undefined | JsValue::Null => false,
            JsValue::Boolean(b) => *b,
            JsValue::Number(n) => *n != 0.0 && !n.is_nan(),
            JsValue::String(s) => !s.is_empty(),
            JsValue::Object(_) | JsValue::Array(_) => true,
            JsValue::Function(..) | JsValue::NativeFunction(_) => true,
        }
    }
    
    /// Convert to number
    pub fn to_number(&self) -> f64 {
        match self {
            JsValue::Undefined => f64::NAN,
            JsValue::Null => 0.0,
            JsValue::Boolean(true) => 1.0,
            JsValue::Boolean(false) => 0.0,
            JsValue::Number(n) => *n,
            JsValue::String(s) => s.parse().unwrap_or(f64::NAN),
            _ => f64::NAN,
        }
    }
    
    /// Convert to string
    pub fn to_string(&self) -> String {
        match self {
            JsValue::Undefined => "undefined".to_string(),
            JsValue::Null => "null".to_string(),
            JsValue::Boolean(true) => "true".to_string(),
            JsValue::Boolean(false) => "false".to_string(),
            JsValue::Number(n) => format!("{}", n),
            JsValue::String(s) => s.clone(),
            JsValue::Object(_) => "[object Object]".to_string(),
            JsValue::Array(arr) => {
                let parts: Vec<String> = arr.iter().map(|v| v.to_string()).collect();
                parts.join(",")
            }
            JsValue::Function(name, ..) => format!("function {}() {{ [native code] }}", name),
            JsValue::NativeFunction(name) => format!("function {}() {{ [native code] }}", name),
        }
    }
}

/// JavaScript execution context
pub struct JsContext {
    pub global: BTreeMap<String, JsValue>,
    pub console_output: Vec<String>,
}

impl JsContext {
    pub fn new() -> Self {
        let mut ctx = Self {
            global: BTreeMap::new(),
            console_output: Vec::new(),
        };
        
        // Add built-in objects
        ctx.setup_builtins();
        ctx
    }
    
    fn setup_builtins(&mut self) {
        // console object
        let mut console = BTreeMap::new();
        console.insert("log".to_string(), JsValue::NativeFunction("console.log".to_string()));
        console.insert("warn".to_string(), JsValue::NativeFunction("console.warn".to_string()));
        console.insert("error".to_string(), JsValue::NativeFunction("console.error".to_string()));
        self.global.insert("console".to_string(), JsValue::Object(console));
        
        // Math object
        let mut math = BTreeMap::new();
        math.insert("PI".to_string(), JsValue::Number(core::f64::consts::PI));
        math.insert("E".to_string(), JsValue::Number(core::f64::consts::E));
        math.insert("random".to_string(), JsValue::NativeFunction("Math.random".to_string()));
        math.insert("floor".to_string(), JsValue::NativeFunction("Math.floor".to_string()));
        math.insert("ceil".to_string(), JsValue::NativeFunction("Math.ceil".to_string()));
        math.insert("round".to_string(), JsValue::NativeFunction("Math.round".to_string()));
        math.insert("abs".to_string(), JsValue::NativeFunction("Math.abs".to_string()));
        math.insert("sqrt".to_string(), JsValue::NativeFunction("Math.sqrt".to_string()));
        self.global.insert("Math".to_string(), JsValue::Object(math));
        
        // Global functions
        self.global.insert("parseInt".to_string(), JsValue::NativeFunction("parseInt".to_string()));
        self.global.insert("parseFloat".to_string(), JsValue::NativeFunction("parseFloat".to_string()));
        self.global.insert("isNaN".to_string(), JsValue::NativeFunction("isNaN".to_string()));
        self.global.insert("isFinite".to_string(), JsValue::NativeFunction("isFinite".to_string()));
        self.global.insert("alert".to_string(), JsValue::NativeFunction("alert".to_string()));
    }
    
    /// Execute JavaScript code
    pub fn execute(&mut self, code: &str) -> Result<JsValue, String> {
        let tokens = tokenize(code)?;
        let ast = parse(&tokens)?;
        self.eval_statements(&ast)
    }
    
    /// Evaluate statements
    fn eval_statements(&mut self, stmts: &[Statement]) -> Result<JsValue, String> {
        let mut result = JsValue::Undefined;
        for stmt in stmts {
            result = self.eval_statement(stmt)?;
        }
        Ok(result)
    }
    
    /// Evaluate a single statement
    fn eval_statement(&mut self, stmt: &Statement) -> Result<JsValue, String> {
        match stmt {
            Statement::Var(name, expr) => {
                let value = if let Some(e) = expr {
                    self.eval_expr(e)?
                } else {
                    JsValue::Undefined
                };
                self.global.insert(name.clone(), value);
                Ok(JsValue::Undefined)
            }
            Statement::Expr(expr) => self.eval_expr(expr),
            Statement::If(cond, then_block, else_block) => {
                let cond_val = self.eval_expr(cond)?;
                if cond_val.to_bool() {
                    self.eval_statements(then_block)
                } else if let Some(else_stmts) = else_block {
                    self.eval_statements(else_stmts)
                } else {
                    Ok(JsValue::Undefined)
                }
            }
            Statement::While(cond, body) => {
                while self.eval_expr(cond)?.to_bool() {
                    self.eval_statements(body)?;
                }
                Ok(JsValue::Undefined)
            }
            Statement::For(init, cond, update, body) => {
                if let Some(init_stmt) = init {
                    self.eval_statement(init_stmt)?;
                }
                while cond.as_ref().map(|c| self.eval_expr(c).map(|v| v.to_bool()).unwrap_or(false)).unwrap_or(true) {
                    self.eval_statements(body)?;
                    if let Some(upd) = update {
                        self.eval_expr(upd)?;
                    }
                }
                Ok(JsValue::Undefined)
            }
            Statement::Return(expr) => {
                if let Some(e) = expr {
                    self.eval_expr(e)
                } else {
                    Ok(JsValue::Undefined)
                }
            }
            Statement::Function(name, params, body) => {
                self.global.insert(
                    name.clone(),
                    JsValue::Function(name.clone(), params.clone(), body.clone()),
                );
                Ok(JsValue::Undefined)
            }
            Statement::Block(stmts) => self.eval_statements(stmts),
        }
    }
    
    /// Evaluate an expression
    fn eval_expr(&mut self, expr: &Expr) -> Result<JsValue, String> {
        match expr {
            Expr::Literal(lit) => Ok(lit.clone()),
            Expr::Identifier(name) => {
                self.global.get(name).cloned().ok_or_else(|| format!("ReferenceError: {} is not defined", name))
            }
            Expr::Binary(op, left, right) => {
                let lval = self.eval_expr(left)?;
                let rval = self.eval_expr(right)?;
                self.eval_binary_op(op, lval, rval)
            }
            Expr::Unary(op, operand) => {
                let val = self.eval_expr(operand)?;
                self.eval_unary_op(op, val)
            }
            Expr::Call(callee, args) => {
                let func = self.eval_expr(callee)?;
                let mut arg_vals = Vec::new();
                for arg in args {
                    arg_vals.push(self.eval_expr(arg)?);
                }
                self.call_function(func, arg_vals)
            }
            Expr::Member(obj, prop) => {
                let obj_val = self.eval_expr(obj)?;
                match obj_val {
                    JsValue::Object(map) => {
                        map.get(prop).cloned().ok_or_else(|| "Property not found".to_string())
                    }
                    JsValue::Array(arr) => {
                        if prop == "length" {
                            Ok(JsValue::Number(arr.len() as f64))
                        } else if let Ok(idx) = prop.parse::<usize>() {
                            Ok(arr.get(idx).cloned().unwrap_or(JsValue::Undefined))
                        } else {
                            Ok(JsValue::Undefined)
                        }
                    }
                    JsValue::String(s) => {
                        if prop == "length" {
                            Ok(JsValue::Number(s.len() as f64))
                        } else {
                            Ok(JsValue::Undefined)
                        }
                    }
                    _ => Ok(JsValue::Undefined),
                }
            }
            Expr::Index(obj, idx) => {
                let obj_val = self.eval_expr(obj)?;
                let idx_val = self.eval_expr(idx)?;
                match obj_val {
                    JsValue::Array(arr) => {
                        let i = idx_val.to_number() as usize;
                        Ok(arr.get(i).cloned().unwrap_or(JsValue::Undefined))
                    }
                    JsValue::Object(map) => {
                        let key = idx_val.to_string();
                        Ok(map.get(&key).cloned().unwrap_or(JsValue::Undefined))
                    }
                    _ => Ok(JsValue::Undefined),
                }
            }
            Expr::Array(elements) => {
                let mut arr = Vec::new();
                for el in elements {
                    arr.push(self.eval_expr(el)?);
                }
                Ok(JsValue::Array(arr))
            }
            Expr::Object(props) => {
                let mut map = BTreeMap::new();
                for (key, val) in props {
                    map.insert(key.clone(), self.eval_expr(val)?);
                }
                Ok(JsValue::Object(map))
            }
            Expr::Assign(name, value) => {
                let val = self.eval_expr(value)?;
                self.global.insert(name.clone(), val.clone());
                Ok(val)
            }
            Expr::Ternary(cond, then_expr, else_expr) => {
                if self.eval_expr(cond)?.to_bool() {
                    self.eval_expr(then_expr)
                } else {
                    self.eval_expr(else_expr)
                }
            }
        }
    }
    
    fn eval_binary_op(&self, op: &str, left: JsValue, right: JsValue) -> Result<JsValue, String> {
        match op {
            "+" => {
                // String concatenation or addition
                match (&left, &right) {
                    (JsValue::String(a), _) => Ok(JsValue::String(format!("{}{}", a, right.to_string()))),
                    (_, JsValue::String(b)) => Ok(JsValue::String(format!("{}{}", left.to_string(), b))),
                    _ => Ok(JsValue::Number(left.to_number() + right.to_number())),
                }
            }
            "-" => Ok(JsValue::Number(left.to_number() - right.to_number())),
            "*" => Ok(JsValue::Number(left.to_number() * right.to_number())),
            "/" => Ok(JsValue::Number(left.to_number() / right.to_number())),
            "%" => Ok(JsValue::Number(left.to_number() % right.to_number())),
            "<" => Ok(JsValue::Boolean(left.to_number() < right.to_number())),
            ">" => Ok(JsValue::Boolean(left.to_number() > right.to_number())),
            "<=" => Ok(JsValue::Boolean(left.to_number() <= right.to_number())),
            ">=" => Ok(JsValue::Boolean(left.to_number() >= right.to_number())),
            "==" | "===" => {
                // Simplified equality
                Ok(JsValue::Boolean(left.to_string() == right.to_string()))
            }
            "!=" | "!==" => {
                Ok(JsValue::Boolean(left.to_string() != right.to_string()))
            }
            "&&" => {
                if !left.to_bool() {
                    Ok(left)
                } else {
                    Ok(right)
                }
            }
            "||" => {
                if left.to_bool() {
                    Ok(left)
                } else {
                    Ok(right)
                }
            }
            _ => Err(format!("Unknown operator: {}", op)),
        }
    }
    
    fn eval_unary_op(&self, op: &str, val: JsValue) -> Result<JsValue, String> {
        match op {
            "!" => Ok(JsValue::Boolean(!val.to_bool())),
            "-" => Ok(JsValue::Number(-val.to_number())),
            "+" => Ok(JsValue::Number(val.to_number())),
            "typeof" => {
                let t = match val {
                    JsValue::Undefined => "undefined",
                    JsValue::Null => "object", // Historical quirk
                    JsValue::Boolean(_) => "boolean",
                    JsValue::Number(_) => "number",
                    JsValue::String(_) => "string",
                    JsValue::Object(_) | JsValue::Array(_) => "object",
                    JsValue::Function(..) | JsValue::NativeFunction(_) => "function",
                };
                Ok(JsValue::String(t.to_string()))
            }
            _ => Err(format!("Unknown unary operator: {}", op)),
        }
    }
    
    fn call_function(&mut self, func: JsValue, args: Vec<JsValue>) -> Result<JsValue, String> {
        match func {
            JsValue::NativeFunction(name) => self.call_native(&name, args),
            JsValue::Function(_name, params, body) => {
                // Simple function call - set parameters
                for (i, param) in params.iter().enumerate() {
                    let val = args.get(i).cloned().unwrap_or(JsValue::Undefined);
                    self.global.insert(param.clone(), val);
                }
                // Parse and execute body
                // This is simplified - a full implementation would handle scope properly
                self.execute(&body)
            }
            _ => Err("TypeError: not a function".to_string()),
        }
    }
    
    fn call_native(&mut self, name: &str, args: Vec<JsValue>) -> Result<JsValue, String> {
        match name {
            "console.log" | "console.warn" | "console.error" => {
                let output: Vec<String> = args.iter().map(|v| v.to_string()).collect();
                let line = output.join(" ");
                self.console_output.push(line);
                Ok(JsValue::Undefined)
            }
            "alert" => {
                if let Some(msg) = args.first() {
                    self.console_output.push(format!("[ALERT] {}", msg.to_string()));
                }
                Ok(JsValue::Undefined)
            }
            "Math.random" => {
                // Simple PRNG
                let seed = crate::cpu::tsc::read_tsc();
                let rand = ((seed >> 16) as f64) / 65536.0;
                Ok(JsValue::Number(rand % 1.0))
            }
            "Math.floor" => {
                let n = args.first().map(|v| v.to_number()).unwrap_or(0.0);
                Ok(JsValue::Number(floor(n)))
            }
            "Math.ceil" => {
                let n = args.first().map(|v| v.to_number()).unwrap_or(0.0);
                Ok(JsValue::Number(ceil(n)))
            }
            "Math.round" => {
                let n = args.first().map(|v| v.to_number()).unwrap_or(0.0);
                Ok(JsValue::Number(round(n)))
            }
            "Math.abs" => {
                let n = args.first().map(|v| v.to_number()).unwrap_or(0.0);
                Ok(JsValue::Number(libm::fabs(n)))
            }
            "Math.sqrt" => {
                let n = args.first().map(|v| v.to_number()).unwrap_or(0.0);
                Ok(JsValue::Number(sqrt(n)))
            }
            "parseInt" => {
                let s = args.first().map(|v| v.to_string()).unwrap_or_default();
                let n: f64 = s.trim().parse().unwrap_or(f64::NAN);
                Ok(JsValue::Number(trunc(n)))
            }
            "parseFloat" => {
                let s = args.first().map(|v| v.to_string()).unwrap_or_default();
                let n: f64 = s.trim().parse().unwrap_or(f64::NAN);
                Ok(JsValue::Number(n))
            }
            "isNaN" => {
                let n = args.first().map(|v| v.to_number()).unwrap_or(f64::NAN);
                Ok(JsValue::Boolean(n.is_nan()))
            }
            "isFinite" => {
                let n = args.first().map(|v| v.to_number()).unwrap_or(f64::NAN);
                Ok(JsValue::Boolean(n.is_finite()))
            }
            _ => Err(format!("Unknown native function: {}", name)),
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// TOKENIZER
// ═══════════════════════════════════════════════════════════════════════════════

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Number(f64),
    String(String),
    Identifier(String),
    Keyword(String),
    Operator(String),
    Punctuation(char),
    Eof,
}

fn tokenize(code: &str) -> Result<Vec<Token>, String> {
    let mut tokens = Vec::new();
    let chars: Vec<char> = code.chars().collect();
    let mut i = 0;
    
    while i < chars.len() {
        let c = chars[i];
        
        // Skip whitespace
        if c.is_whitespace() {
            i += 1;
            continue;
        }
        
        // Skip comments
        if c == '/' && i + 1 < chars.len() {
            if chars[i + 1] == '/' {
                // Line comment
                while i < chars.len() && chars[i] != '\n' {
                    i += 1;
                }
                continue;
            }
            if chars[i + 1] == '*' {
                // Block comment
                i += 2;
                while i + 1 < chars.len() && !(chars[i] == '*' && chars[i + 1] == '/') {
                    i += 1;
                }
                i += 2;
                continue;
            }
        }
        
        // Number
        if c.is_ascii_digit() || (c == '.' && i + 1 < chars.len() && chars[i + 1].is_ascii_digit()) {
            let start = i;
            while i < chars.len() && (chars[i].is_ascii_digit() || chars[i] == '.') {
                i += 1;
            }
            let num_str: String = chars[start..i].iter().collect();
            let num: f64 = num_str.parse().unwrap_or(0.0);
            tokens.push(Token::Number(num));
            continue;
        }
        
        // String
        if c == '"' || c == '\'' {
            let quote = c;
            i += 1;
            let start = i;
            while i < chars.len() && chars[i] != quote {
                if chars[i] == '\\' && i + 1 < chars.len() {
                    i += 2;
                } else {
                    i += 1;
                }
            }
            let s: String = chars[start..i].iter().collect();
            tokens.push(Token::String(s));
            i += 1; // Skip closing quote
            continue;
        }
        
        // Identifier or keyword
        if c.is_alphabetic() || c == '_' || c == '$' {
            let start = i;
            while i < chars.len() && (chars[i].is_alphanumeric() || chars[i] == '_' || chars[i] == '$') {
                i += 1;
            }
            let word: String = chars[start..i].iter().collect();
            let keywords = ["var", "let", "const", "function", "if", "else", "for", "while", 
                          "return", "true", "false", "null", "undefined", "typeof", "new"];
            if keywords.contains(&word.as_str()) {
                tokens.push(Token::Keyword(word));
            } else {
                tokens.push(Token::Identifier(word));
            }
            continue;
        }
        
        // Multi-char operators
        let two_char: String = chars[i..].iter().take(2).collect();
        let three_char: String = chars[i..].iter().take(3).collect();
        
        if ["===", "!=="].contains(&three_char.as_str()) {
            tokens.push(Token::Operator(three_char));
            i += 3;
            continue;
        }
        
        if ["==", "!=", "<=", ">=", "&&", "||", "++", "--", "+=", "-=", "*=", "/=", "=>"]
            .contains(&two_char.as_str()) {
            tokens.push(Token::Operator(two_char));
            i += 2;
            continue;
        }
        
        // Single-char operators
        if "+-*/%<>=!&|?:".contains(c) {
            tokens.push(Token::Operator(c.to_string()));
            i += 1;
            continue;
        }
        
        // Punctuation
        if "{}[]();,.".contains(c) {
            tokens.push(Token::Punctuation(c));
            i += 1;
            continue;
        }
        
        // Unknown character - skip
        i += 1;
    }
    
    tokens.push(Token::Eof);
    Ok(tokens)
}

// ═══════════════════════════════════════════════════════════════════════════════
// PARSER
// ═══════════════════════════════════════════════════════════════════════════════

#[derive(Debug, Clone)]
pub enum Statement {
    Var(String, Option<Expr>),
    Expr(Expr),
    If(Expr, Vec<Statement>, Option<Vec<Statement>>),
    While(Expr, Vec<Statement>),
    For(Option<Box<Statement>>, Option<Expr>, Option<Expr>, Vec<Statement>),
    Return(Option<Expr>),
    Function(String, Vec<String>, String),
    Block(Vec<Statement>),
}

#[derive(Debug, Clone)]
pub enum Expr {
    Literal(JsValue),
    Identifier(String),
    Binary(String, Box<Expr>, Box<Expr>),
    Unary(String, Box<Expr>),
    Call(Box<Expr>, Vec<Expr>),
    Member(Box<Expr>, String),
    Index(Box<Expr>, Box<Expr>),
    Array(Vec<Expr>),
    Object(Vec<(String, Expr)>),
    Assign(String, Box<Expr>),
    Ternary(Box<Expr>, Box<Expr>, Box<Expr>),
}

fn parse(tokens: &[Token]) -> Result<Vec<Statement>, String> {
    let mut parser = Parser { tokens, pos: 0 };
    parser.parse_statements()
}

struct Parser<'a> {
    tokens: &'a [Token],
    pos: usize,
}

impl<'a> Parser<'a> {
    fn current(&self) -> &Token {
        self.tokens.get(self.pos).unwrap_or(&Token::Eof)
    }
    
    fn advance(&mut self) {
        if self.pos < self.tokens.len() {
            self.pos += 1;
        }
    }
    
    fn expect_punct(&mut self, c: char) -> Result<(), String> {
        if let Token::Punctuation(p) = self.current() {
            if *p == c {
                self.advance();
                return Ok(());
            }
        }
        Err(format!("Expected '{}'", c))
    }
    
    fn parse_statements(&mut self) -> Result<Vec<Statement>, String> {
        let mut stmts = Vec::new();
        
        while !matches!(self.current(), Token::Eof | Token::Punctuation('}')) {
            stmts.push(self.parse_statement()?);
        }
        
        Ok(stmts)
    }
    
    fn parse_statement(&mut self) -> Result<Statement, String> {
        match self.current() {
            Token::Keyword(kw) if kw == "var" || kw == "let" || kw == "const" => {
                self.advance();
                if let Token::Identifier(name) = self.current().clone() {
                    self.advance();
                    let value = if let Token::Operator(op) = self.current() {
                        if op == "=" {
                            self.advance();
                            Some(self.parse_expression()?)
                        } else {
                            None
                        }
                    } else {
                        None
                    };
                    if let Token::Punctuation(';') = self.current() {
                        self.advance();
                    }
                    Ok(Statement::Var(name, value))
                } else {
                    Err("Expected identifier after var".to_string())
                }
            }
            Token::Keyword(kw) if kw == "if" => {
                self.advance();
                self.expect_punct('(')?;
                let cond = self.parse_expression()?;
                self.expect_punct(')')?;
                let then_block = self.parse_block_or_statement()?;
                let else_block = if let Token::Keyword(kw) = self.current() {
                    if kw == "else" {
                        self.advance();
                        Some(self.parse_block_or_statement()?)
                    } else {
                        None
                    }
                } else {
                    None
                };
                Ok(Statement::If(cond, then_block, else_block))
            }
            Token::Keyword(kw) if kw == "while" => {
                self.advance();
                self.expect_punct('(')?;
                let cond = self.parse_expression()?;
                self.expect_punct(')')?;
                let body = self.parse_block_or_statement()?;
                Ok(Statement::While(cond, body))
            }
            Token::Keyword(kw) if kw == "return" => {
                self.advance();
                let value = if let Token::Punctuation(';') | Token::Punctuation('}') = self.current() {
                    None
                } else {
                    Some(self.parse_expression()?)
                };
                if let Token::Punctuation(';') = self.current() {
                    self.advance();
                }
                Ok(Statement::Return(value))
            }
            Token::Keyword(kw) if kw == "function" => {
                self.advance();
                if let Token::Identifier(name) = self.current().clone() {
                    self.advance();
                    self.expect_punct('(')?;
                    let params = self.parse_params()?;
                    self.expect_punct(')')?;
                    self.expect_punct('{')?;
                    // For simplicity, store body as string (would need proper parsing)
                    let body = self.parse_block_body()?;
                    Ok(Statement::Function(name, params, body))
                } else {
                    Err("Expected function name".to_string())
                }
            }
            Token::Punctuation('{') => {
                self.advance();
                let stmts = self.parse_statements()?;
                self.expect_punct('}')?;
                Ok(Statement::Block(stmts))
            }
            _ => {
                let expr = self.parse_expression()?;
                if let Token::Punctuation(';') = self.current() {
                    self.advance();
                }
                Ok(Statement::Expr(expr))
            }
        }
    }
    
    fn parse_block_or_statement(&mut self) -> Result<Vec<Statement>, String> {
        if let Token::Punctuation('{') = self.current() {
            self.advance();
            let stmts = self.parse_statements()?;
            self.expect_punct('}')?;
            Ok(stmts)
        } else {
            Ok(vec![self.parse_statement()?])
        }
    }
    
    fn parse_block_body(&mut self) -> Result<String, String> {
        // Simplified: just find matching brace
        let mut depth = 1;
        let start = self.pos;
        while depth > 0 && !matches!(self.current(), Token::Eof) {
            match self.current() {
                Token::Punctuation('{') => depth += 1,
                Token::Punctuation('}') => depth -= 1,
                _ => {}
            }
            if depth > 0 {
                self.advance();
            }
        }
        self.advance(); // consume closing }
        Ok(String::new()) // Return empty - would need to return actual code
    }
    
    fn parse_params(&mut self) -> Result<Vec<String>, String> {
        let mut params = Vec::new();
        while let Token::Identifier(name) = self.current().clone() {
            params.push(name);
            self.advance();
            if let Token::Punctuation(',') = self.current() {
                self.advance();
            } else {
                break;
            }
        }
        Ok(params)
    }
    
    fn parse_expression(&mut self) -> Result<Expr, String> {
        self.parse_ternary()
    }
    
    fn parse_ternary(&mut self) -> Result<Expr, String> {
        let cond = self.parse_or()?;
        if let Token::Operator(op) = self.current() {
            if op == "?" {
                self.advance();
                let then_expr = self.parse_expression()?;
                if let Token::Operator(op) = self.current() {
                    if op == ":" {
                        self.advance();
                        let else_expr = self.parse_expression()?;
                        return Ok(Expr::Ternary(Box::new(cond), Box::new(then_expr), Box::new(else_expr)));
                    }
                }
            }
        }
        Ok(cond)
    }
    
    fn parse_or(&mut self) -> Result<Expr, String> {
        let mut left = self.parse_and()?;
        while let Token::Operator(op) = self.current() {
            if op == "||" {
                let op = op.clone();
                self.advance();
                let right = self.parse_and()?;
                left = Expr::Binary(op, Box::new(left), Box::new(right));
            } else {
                break;
            }
        }
        Ok(left)
    }
    
    fn parse_and(&mut self) -> Result<Expr, String> {
        let mut left = self.parse_equality()?;
        while let Token::Operator(op) = self.current() {
            if op == "&&" {
                let op = op.clone();
                self.advance();
                let right = self.parse_equality()?;
                left = Expr::Binary(op, Box::new(left), Box::new(right));
            } else {
                break;
            }
        }
        Ok(left)
    }
    
    fn parse_equality(&mut self) -> Result<Expr, String> {
        let mut left = self.parse_comparison()?;
        while let Token::Operator(op) = self.current() {
            if op == "==" || op == "!=" || op == "===" || op == "!==" {
                let op = op.clone();
                self.advance();
                let right = self.parse_comparison()?;
                left = Expr::Binary(op, Box::new(left), Box::new(right));
            } else {
                break;
            }
        }
        Ok(left)
    }
    
    fn parse_comparison(&mut self) -> Result<Expr, String> {
        let mut left = self.parse_additive()?;
        while let Token::Operator(op) = self.current() {
            if op == "<" || op == ">" || op == "<=" || op == ">=" {
                let op = op.clone();
                self.advance();
                let right = self.parse_additive()?;
                left = Expr::Binary(op, Box::new(left), Box::new(right));
            } else {
                break;
            }
        }
        Ok(left)
    }
    
    fn parse_additive(&mut self) -> Result<Expr, String> {
        let mut left = self.parse_multiplicative()?;
        while let Token::Operator(op) = self.current() {
            if op == "+" || op == "-" {
                let op = op.clone();
                self.advance();
                let right = self.parse_multiplicative()?;
                left = Expr::Binary(op, Box::new(left), Box::new(right));
            } else {
                break;
            }
        }
        Ok(left)
    }
    
    fn parse_multiplicative(&mut self) -> Result<Expr, String> {
        let mut left = self.parse_unary()?;
        while let Token::Operator(op) = self.current() {
            if op == "*" || op == "/" || op == "%" {
                let op = op.clone();
                self.advance();
                let right = self.parse_unary()?;
                left = Expr::Binary(op, Box::new(left), Box::new(right));
            } else {
                break;
            }
        }
        Ok(left)
    }
    
    fn parse_unary(&mut self) -> Result<Expr, String> {
        if let Token::Operator(op) = self.current() {
            if op == "!" || op == "-" || op == "+" {
                let op = op.clone();
                self.advance();
                let operand = self.parse_unary()?;
                return Ok(Expr::Unary(op, Box::new(operand)));
            }
        }
        if let Token::Keyword(kw) = self.current() {
            if kw == "typeof" {
                self.advance();
                let operand = self.parse_unary()?;
                return Ok(Expr::Unary("typeof".to_string(), Box::new(operand)));
            }
        }
        self.parse_postfix()
    }
    
    fn parse_postfix(&mut self) -> Result<Expr, String> {
        let mut expr = self.parse_primary()?;
        
        loop {
            match self.current() {
                Token::Punctuation('.') => {
                    self.advance();
                    if let Token::Identifier(prop) = self.current().clone() {
                        self.advance();
                        expr = Expr::Member(Box::new(expr), prop);
                    } else {
                        break;
                    }
                }
                Token::Punctuation('[') => {
                    self.advance();
                    let index = self.parse_expression()?;
                    self.expect_punct(']')?;
                    expr = Expr::Index(Box::new(expr), Box::new(index));
                }
                Token::Punctuation('(') => {
                    self.advance();
                    let args = self.parse_arguments()?;
                    self.expect_punct(')')?;
                    expr = Expr::Call(Box::new(expr), args);
                }
                _ => break,
            }
        }
        
        Ok(expr)
    }
    
    fn parse_primary(&mut self) -> Result<Expr, String> {
        match self.current().clone() {
            Token::Number(n) => {
                self.advance();
                Ok(Expr::Literal(JsValue::Number(n)))
            }
            Token::String(s) => {
                self.advance();
                Ok(Expr::Literal(JsValue::String(s)))
            }
            Token::Keyword(kw) if kw == "true" => {
                self.advance();
                Ok(Expr::Literal(JsValue::Boolean(true)))
            }
            Token::Keyword(kw) if kw == "false" => {
                self.advance();
                Ok(Expr::Literal(JsValue::Boolean(false)))
            }
            Token::Keyword(kw) if kw == "null" => {
                self.advance();
                Ok(Expr::Literal(JsValue::Null))
            }
            Token::Keyword(kw) if kw == "undefined" => {
                self.advance();
                Ok(Expr::Literal(JsValue::Undefined))
            }
            Token::Identifier(name) => {
                self.advance();
                // Check for assignment
                if let Token::Operator(op) = self.current() {
                    if op == "=" {
                        self.advance();
                        let value = self.parse_expression()?;
                        return Ok(Expr::Assign(name, Box::new(value)));
                    }
                }
                Ok(Expr::Identifier(name))
            }
            Token::Punctuation('(') => {
                self.advance();
                let expr = self.parse_expression()?;
                self.expect_punct(')')?;
                Ok(expr)
            }
            Token::Punctuation('[') => {
                self.advance();
                let elements = self.parse_array_elements()?;
                self.expect_punct(']')?;
                Ok(Expr::Array(elements))
            }
            Token::Punctuation('{') => {
                self.advance();
                let props = self.parse_object_props()?;
                self.expect_punct('}')?;
                Ok(Expr::Object(props))
            }
            _ => Err(format!("Unexpected token: {:?}", self.current())),
        }
    }
    
    fn parse_arguments(&mut self) -> Result<Vec<Expr>, String> {
        let mut args = Vec::new();
        if !matches!(self.current(), Token::Punctuation(')')) {
            args.push(self.parse_expression()?);
            while let Token::Punctuation(',') = self.current() {
                self.advance();
                args.push(self.parse_expression()?);
            }
        }
        Ok(args)
    }
    
    fn parse_array_elements(&mut self) -> Result<Vec<Expr>, String> {
        let mut elements = Vec::new();
        if !matches!(self.current(), Token::Punctuation(']')) {
            elements.push(self.parse_expression()?);
            while let Token::Punctuation(',') = self.current() {
                self.advance();
                if matches!(self.current(), Token::Punctuation(']')) {
                    break;
                }
                elements.push(self.parse_expression()?);
            }
        }
        Ok(elements)
    }
    
    fn parse_object_props(&mut self) -> Result<Vec<(String, Expr)>, String> {
        let mut props = Vec::new();
        while !matches!(self.current(), Token::Punctuation('}')) {
            let key = match self.current().clone() {
                Token::Identifier(s) | Token::String(s) => {
                    self.advance();
                    s
                }
                _ => return Err("Expected property name".to_string()),
            };
            self.expect_punct(':')?;
            let value = self.parse_expression()?;
            props.push((key, value));
            if let Token::Punctuation(',') = self.current() {
                self.advance();
            } else {
                break;
            }
        }
        Ok(props)
    }
}
