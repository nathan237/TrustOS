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
// Énumération — un type qui peut être l'une de plusieurs variantes.
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

// Bloc d'implémentation — définit les méthodes du type ci-dessus.
impl JsValue {
    /// Convert to boolean (truthy/falsy)
    pub fn to_bool(&self) -> bool {
                // Correspondance de motifs — branchement exhaustif de Rust.
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
                // Correspondance de motifs — branchement exhaustif de Rust.
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
                // Correspondance de motifs — branchement exhaustif de Rust.
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

// Bloc d'implémentation — définit les méthodes du type ci-dessus.
impl JsContext {
        // Fonction publique — appelable depuis d'autres modules.
pub fn new() -> Self {
        let mut context = Self {
            global: BTreeMap::new(),
            console_output: Vec::new(),
        };
        
        // Add built-in objects
        context.setup_builtins();
        context
    }
    
    fn setup_builtins(&mut self) {
        // console object
        let mut console = BTreeMap::new();
        console.insert("log".to_string(), JsValue::NativeFunction("console.log".to_string()));
        console.insert("warn".to_string(), JsValue::NativeFunction("console.warn".to_string()));
        console.insert("error".to_string(), JsValue::NativeFunction("console.error".to_string()));
        console.insert("info".to_string(), JsValue::NativeFunction("console.log".to_string()));
        console.insert("debug".to_string(), JsValue::NativeFunction("console.log".to_string()));
        self.global.insert("console".to_string(), JsValue::Object(console));
        
        // Math object
        let mut math = BTreeMap::new();
        math.insert("PI".to_string(), JsValue::Number(core::f64::consts::PI));
        math.insert("E".to_string(), JsValue::Number(core::f64::consts::E));
        math.insert("LN2".to_string(), JsValue::Number(core::f64::consts::LINE_2));
        math.insert("LN10".to_string(), JsValue::Number(core::f64::consts::LINE_10));
        math.insert("SQRT2".to_string(), JsValue::Number(core::f64::consts::SQRT_2));
        math.insert("random".to_string(), JsValue::NativeFunction("Math.random".to_string()));
        math.insert("floor".to_string(), JsValue::NativeFunction("Math.floor".to_string()));
        math.insert("ceil".to_string(), JsValue::NativeFunction("Math.ceil".to_string()));
        math.insert("round".to_string(), JsValue::NativeFunction("Math.round".to_string()));
        math.insert("abs".to_string(), JsValue::NativeFunction("Math.abs".to_string()));
        math.insert("sqrt".to_string(), JsValue::NativeFunction("Math.sqrt".to_string()));
        math.insert("min".to_string(), JsValue::NativeFunction("Math.min".to_string()));
        math.insert("max".to_string(), JsValue::NativeFunction("Math.max".to_string()));
        math.insert("pow".to_string(), JsValue::NativeFunction("Math.pow".to_string()));
        math.insert("sin".to_string(), JsValue::NativeFunction("Math.sin".to_string()));
        math.insert("cos".to_string(), JsValue::NativeFunction("Math.cos".to_string()));
        math.insert("tan".to_string(), JsValue::NativeFunction("Math.tan".to_string()));
        math.insert("log".to_string(), JsValue::NativeFunction("Math.log".to_string()));
        math.insert("sign".to_string(), JsValue::NativeFunction("Math.sign".to_string()));
        math.insert("trunc".to_string(), JsValue::NativeFunction("Math.trunc".to_string()));
        self.global.insert("Math".to_string(), JsValue::Object(math));
        
        // JSON object
        let mut json = BTreeMap::new();
        json.insert("parse".to_string(), JsValue::NativeFunction("JSON.parse".to_string()));
        json.insert("stringify".to_string(), JsValue::NativeFunction("JSON.stringify".to_string()));
        self.global.insert("JSON".to_string(), JsValue::Object(json));
        
        // document object (stub DOM)
        let mut document = BTreeMap::new();
        document.insert("getElementById".to_string(), JsValue::NativeFunction("document.getElementById".to_string()));
        document.insert("querySelector".to_string(), JsValue::NativeFunction("document.querySelector".to_string()));
        document.insert("querySelectorAll".to_string(), JsValue::NativeFunction("document.querySelectorAll".to_string()));
        document.insert("createElement".to_string(), JsValue::NativeFunction("document.createElement".to_string()));
        document.insert("createTextNode".to_string(), JsValue::NativeFunction("document.createTextNode".to_string()));
        document.insert("write".to_string(), JsValue::NativeFunction("document.write".to_string()));
        document.insert("title".to_string(), JsValue::String("TrustOS Browser".to_string()));
        document.insert("readyState".to_string(), JsValue::String("complete".to_string()));
        
        // document.body stub
        let mut body = BTreeMap::new();
        body.insert("innerHTML".to_string(), JsValue::String(String::new()));
        body.insert("textContent".to_string(), JsValue::String(String::new()));
        body.insert("className".to_string(), JsValue::String(String::new()));
        body.insert("style".to_string(), JsValue::Object(BTreeMap::new()));
        body.insert("appendChild".to_string(), JsValue::NativeFunction("element.appendChild".to_string()));
        body.insert("children".to_string(), JsValue::Array(Vec::new()));
        body.insert("tagName".to_string(), JsValue::String("BODY".to_string()));
        document.insert("body".to_string(), JsValue::Object(body));
        self.global.insert("document".to_string(), JsValue::Object(document));
        
        // window object
        let mut window = BTreeMap::new();
        window.insert("innerWidth".to_string(), JsValue::Number(1024.0));
        window.insert("innerHeight".to_string(), JsValue::Number(768.0));
        window.insert("location".to_string(), JsValue::Object(BTreeMap::new()));
        window.insert("navigator".to_string(), JsValue::Object({
            let mut nav = BTreeMap::new();
            nav.insert("userAgent".to_string(), JsValue::String("TrustOS/1.0".to_string()));
            nav.insert("platform".to_string(), JsValue::String("TrustOS".to_string()));
            nav.insert("language".to_string(), JsValue::String("en-US".to_string()));
            nav
        }));
        self.global.insert("window".to_string(), JsValue::Object(window));
        
        // Global functions
        self.global.insert("parseInt".to_string(), JsValue::NativeFunction("parseInt".to_string()));
        self.global.insert("parseFloat".to_string(), JsValue::NativeFunction("parseFloat".to_string()));
        self.global.insert("isNaN".to_string(), JsValue::NativeFunction("isNaN".to_string()));
        self.global.insert("isFinite".to_string(), JsValue::NativeFunction("isFinite".to_string()));
        self.global.insert("alert".to_string(), JsValue::NativeFunction("alert".to_string()));
        self.global.insert("setTimeout".to_string(), JsValue::NativeFunction("setTimeout".to_string()));
        self.global.insert("setInterval".to_string(), JsValue::NativeFunction("setInterval".to_string()));
        self.global.insert("clearTimeout".to_string(), JsValue::NativeFunction("clearTimeout".to_string()));
        self.global.insert("clearInterval".to_string(), JsValue::NativeFunction("clearInterval".to_string()));
        self.global.insert("encodeURIComponent".to_string(), JsValue::NativeFunction("encodeURIComponent".to_string()));
        self.global.insert("decodeURIComponent".to_string(), JsValue::NativeFunction("decodeURIComponent".to_string()));
        self.global.insert("String".to_string(), JsValue::NativeFunction("String".to_string()));
        self.global.insert("Number".to_string(), JsValue::NativeFunction("Number".to_string()));
        self.global.insert("Boolean".to_string(), JsValue::NativeFunction("Boolean".to_string()));
        self.global.insert("Array".to_string(), JsValue::NativeFunction("Array".to_string()));
        self.global.insert("Object".to_string(), JsValue::NativeFunction("Object".to_string()));
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
                // Correspondance de motifs — branchement exhaustif de Rust.
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
            Statement::If(condition, then_block, else_block) => {
                let condition_value = self.eval_expr(condition)?;
                if condition_value.to_bool() {
                    self.eval_statements(then_block)
                } else if let Some(else_stmts) = else_block {
                    self.eval_statements(else_stmts)
                } else {
                    Ok(JsValue::Undefined)
                }
            }
            Statement::While(condition, body) => {
                while self.eval_expr(condition)?.to_bool() {
                    self.eval_statements(body)?;
                }
                Ok(JsValue::Undefined)
            }
            Statement::For(init, condition, update, body) => {
                if let Some(initialize_stmt) = init {
                    self.eval_statement(initialize_stmt)?;
                }
                while condition.as_ref().map(|c| self.eval_expr(c).map(|v| v.to_bool()).unwrap_or(false)).unwrap_or(true) {
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
                // Correspondance de motifs — branchement exhaustif de Rust.
match expr {
            Expr::Literal(lit) => Ok(lit.clone()),
            Expr::Identifier(name) => {
                self.global.get(name).cloned().ok_or_else(|| format!("ReferenceError: {} is not defined", name))
            }
            Expr::Binary(op, left, right) => {
                let lval = self.eval_expr(left)?;
                let rval = self.eval_expr(right)?;
                self.eval_binary_operation(op, lval, rval)
            }
            Expr::Unary(op, operand) => {
                let value = self.eval_expr(operand)?;
                self.eval_unary_operation(op, value)
            }
            Expr::Call(callee, args) => {
                // Handle method calls (obj.method(args)) by passing obj as receiver
                let (func, receiver) = if let Expr::Member(object_expr, _prop) = callee.as_ref() {
                    let recv = self.eval_expr(object_expr)?;
                    let f = self.eval_expr(callee)?;
                    (f, Some(recv))
                } else {
                    (self.eval_expr(callee)?, None)
                };
                let mut argument_vals = Vec::new();
                for argument in args {
                    argument_vals.push(self.eval_expr(argument)?);
                }
                self.call_function_with_receiver(func, argument_vals, receiver)
            }
            Expr::Member(object, prop) => {
                let object_value = self.eval_expr(object)?;
                                // Correspondance de motifs — branchement exhaustif de Rust.
match &object_value {
                    JsValue::Object(map) => {
                        Ok(map.get(prop).cloned().unwrap_or(JsValue::Undefined))
                    }
                    JsValue::Array(arr) => {
                                                // Correspondance de motifs — branchement exhaustif de Rust.
match prop.as_str() {
                            "length" => Ok(JsValue::Number(arr.len() as f64)),
                            "push" => Ok(JsValue::NativeFunction("Array.push".to_string())),
                            "pop" => Ok(JsValue::NativeFunction("Array.pop".to_string())),
                            "shift" => Ok(JsValue::NativeFunction("Array.shift".to_string())),
                            "unshift" => Ok(JsValue::NativeFunction("Array.unshift".to_string())),
                            "join" => Ok(JsValue::NativeFunction("Array.join".to_string())),
                            "reverse" => Ok(JsValue::NativeFunction("Array.reverse".to_string())),
                            "indexOf" => Ok(JsValue::NativeFunction("Array.indexOf".to_string())),
                            "includes" => Ok(JsValue::NativeFunction("Array.includes".to_string())),
                            "slice" => Ok(JsValue::NativeFunction("Array.slice".to_string())),
                            "concat" => Ok(JsValue::NativeFunction("Array.concat".to_string())),
                            "map" => Ok(JsValue::NativeFunction("Array.map".to_string())),
                            "filter" => Ok(JsValue::NativeFunction("Array.filter".to_string())),
                            "forEach" => Ok(JsValue::NativeFunction("Array.forEach".to_string())),
                            "find" => Ok(JsValue::NativeFunction("Array.find".to_string())),
                            "some" => Ok(JsValue::NativeFunction("Array.some".to_string())),
                            "every" => Ok(JsValue::NativeFunction("Array.every".to_string())),
                            "sort" => Ok(JsValue::NativeFunction("Array.sort".to_string())),
                            "fill" => Ok(JsValue::NativeFunction("Array.fill".to_string())),
                            "flat" => Ok(JsValue::NativeFunction("Array.flat".to_string())),
                            "reduce" => Ok(JsValue::NativeFunction("Array.reduce".to_string())),
                            _ => {
                                if let Ok(index) = prop.parse::<usize>() {
                                    Ok(arr.get(index).cloned().unwrap_or(JsValue::Undefined))
                                } else {
                                    Ok(JsValue::Undefined)
                                }
                            }
                        }
                    }
                    JsValue::String(s) => {
                                                // Correspondance de motifs — branchement exhaustif de Rust.
match prop.as_str() {
                            "length" => Ok(JsValue::Number(s.len() as f64)),
                            "toUpperCase" => Ok(JsValue::NativeFunction("String.toUpperCase".to_string())),
                            "toLowerCase" => Ok(JsValue::NativeFunction("String.toLowerCase".to_string())),
                            "trim" => Ok(JsValue::NativeFunction("String.trim".to_string())),
                            "trimStart" | "trimLeft" => Ok(JsValue::NativeFunction("String.trimStart".to_string())),
                            "trimEnd" | "trimRight" => Ok(JsValue::NativeFunction("String.trimEnd".to_string())),
                            "includes" => Ok(JsValue::NativeFunction("String.includes".to_string())),
                            "indexOf" => Ok(JsValue::NativeFunction("String.indexOf".to_string())),
                            "lastIndexOf" => Ok(JsValue::NativeFunction("String.lastIndexOf".to_string())),
                            "startsWith" => Ok(JsValue::NativeFunction("String.startsWith".to_string())),
                            "endsWith" => Ok(JsValue::NativeFunction("String.endsWith".to_string())),
                            "slice" => Ok(JsValue::NativeFunction("String.slice".to_string())),
                            "substring" => Ok(JsValue::NativeFunction("String.substring".to_string())),
                            "replace" => Ok(JsValue::NativeFunction("String.replace".to_string())),
                            "split" => Ok(JsValue::NativeFunction("String.split".to_string())),
                            "charAt" => Ok(JsValue::NativeFunction("String.charAt".to_string())),
                            "charCodeAt" => Ok(JsValue::NativeFunction("String.charCodeAt".to_string())),
                            "repeat" => Ok(JsValue::NativeFunction("String.repeat".to_string())),
                            "padStart" => Ok(JsValue::NativeFunction("String.padStart".to_string())),
                            "padEnd" => Ok(JsValue::NativeFunction("String.padEnd".to_string())),
                            "concat" => Ok(JsValue::NativeFunction("String.concat".to_string())),
                            "match" => Ok(JsValue::NativeFunction("String.match".to_string())),
                            "search" => Ok(JsValue::NativeFunction("String.search".to_string())),
                            _ => Ok(JsValue::Undefined),
                        }
                    }
                    JsValue::Number(_n) => {
                                                // Correspondance de motifs — branchement exhaustif de Rust.
match prop.as_str() {
                            "toFixed" => Ok(JsValue::NativeFunction("Number.toFixed".to_string())),
                            "toString" => Ok(JsValue::NativeFunction("Number.toString".to_string())),
                            _ => Ok(JsValue::Undefined),
                        }
                    }
                    _ => Ok(JsValue::Undefined),
                }
            }
            Expr::Index(object, index) => {
                let object_value = self.eval_expr(object)?;
                let index_value = self.eval_expr(index)?;
                                // Correspondance de motifs — branchement exhaustif de Rust.
match object_value {
                    JsValue::Array(arr) => {
                        let i = index_value.to_number() as usize;
                        Ok(arr.get(i).cloned().unwrap_or(JsValue::Undefined))
                    }
                    JsValue::Object(map) => {
                        let key = index_value.to_string();
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
                for (key, value) in props {
                    map.insert(key.clone(), self.eval_expr(value)?);
                }
                Ok(JsValue::Object(map))
            }
            Expr::Assign(name, value) => {
                let value = self.eval_expr(value)?;
                self.global.insert(name.clone(), value.clone());
                Ok(value)
            }
            Expr::Ternary(condition, then_expr, else_expr) => {
                if self.eval_expr(condition)?.to_bool() {
                    self.eval_expr(then_expr)
                } else {
                    self.eval_expr(else_expr)
                }
            }
        }
    }
    
    fn eval_binary_operation(&self, op: &str, left: JsValue, right: JsValue) -> Result<JsValue, String> {
                // Correspondance de motifs — branchement exhaustif de Rust.
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
    
    fn eval_unary_operation(&self, op: &str, value: JsValue) -> Result<JsValue, String> {
                // Correspondance de motifs — branchement exhaustif de Rust.
match op {
            "!" => Ok(JsValue::Boolean(!value.to_bool())),
            "-" => Ok(JsValue::Number(-value.to_number())),
            "+" => Ok(JsValue::Number(value.to_number())),
            "typeof" => {
                let t = // Correspondance de motifs — branchement exhaustif de Rust.
match value {
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
    
    fn call_function_with_receiver(&mut self, func: JsValue, args: Vec<JsValue>, receiver: Option<JsValue>) -> Result<JsValue, String> {
                // Correspondance de motifs — branchement exhaustif de Rust.
match func {
            JsValue::NativeFunction(name) => self.call_native_with_receiver(&name, args, receiver),
            JsValue::Function(_name, params, body) => {
                for (i, parameter) in params.iter().enumerate() {
                    let value = args.get(i).cloned().unwrap_or(JsValue::Undefined);
                    self.global.insert(parameter.clone(), value);
                }
                self.execute(&body)
            }
            _ => Err("TypeError: not a function".to_string()),
        }
    }

    fn call_function(&mut self, func: JsValue, args: Vec<JsValue>) -> Result<JsValue, String> {
        self.call_function_with_receiver(func, args, None)
    }
    
    fn call_native_with_receiver(&mut self, name: &str, args: Vec<JsValue>, receiver: Option<JsValue>) -> Result<JsValue, String> {
                // Correspondance de motifs — branchement exhaustif de Rust.
match name {
            // ── Console ────────────────────────────────────────────────
            "console.log" | "console.warn" | "console.error" => {
                let output: Vec<String> = args.iter().map(|v| v.to_string()).collect();
                let line = output.join(" ");
                self.console_output.push(line);
                Ok(JsValue::Undefined)
            }
            "alert" => {
                if let Some(message) = args.first() {
                    self.console_output.push(format!("[ALERT] {}", message.to_string()));
                }
                Ok(JsValue::Undefined)
            }

            // ── Timers (stubs — no real async in bare-metal) ───────────
            "setTimeout" | "setInterval" => Ok(JsValue::Number(0.0)),
            "clearTimeout" | "clearInterval" => Ok(JsValue::Undefined),

            // ── Math ───────────────────────────────────────────────────
            "Math.random" => {
                let seed = crate::cpu::tsc::read_tsc();
                let random = ((seed >> 16) as f64) / 65536.0;
                Ok(JsValue::Number(random % 1.0))
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
            "Math.min" => {
                if args.is_empty() { return Ok(JsValue::Number(f64::INFINITY)); }
                let mut m = args[0].to_number();
                for a in &args[1..] { let v = a.to_number(); if v < m { m = v; } }
                Ok(JsValue::Number(m))
            }
            "Math.max" => {
                if args.is_empty() { return Ok(JsValue::Number(f64::NEGATIVE_INFINITY)); }
                let mut m = args[0].to_number();
                for a in &args[1..] { let v = a.to_number(); if v > m { m = v; } }
                Ok(JsValue::Number(m))
            }
            "Math.pow" => {
                let base = args.first().map(|v| v.to_number()).unwrap_or(0.0);
                let exp = args.get(1).map(|v| v.to_number()).unwrap_or(0.0);
                Ok(JsValue::Number(libm::pow(base, exp)))
            }
            "Math.sin" => {
                let n = args.first().map(|v| v.to_number()).unwrap_or(0.0);
                Ok(JsValue::Number(libm::sin(n)))
            }
            "Math.cos" => {
                let n = args.first().map(|v| v.to_number()).unwrap_or(0.0);
                Ok(JsValue::Number(libm::cos(n)))
            }
            "Math.tan" => {
                let n = args.first().map(|v| v.to_number()).unwrap_or(0.0);
                Ok(JsValue::Number(libm::tan(n)))
            }
            "Math.log" => {
                let n = args.first().map(|v| v.to_number()).unwrap_or(0.0);
                Ok(JsValue::Number(libm::log(n)))
            }
            "Math.sign" => {
                let n = args.first().map(|v| v.to_number()).unwrap_or(0.0);
                Ok(JsValue::Number(if n > 0.0 { 1.0 } else if n < 0.0 { -1.0 } else { 0.0 }))
            }
            "Math.trunc" => {
                let n = args.first().map(|v| v.to_number()).unwrap_or(0.0);
                Ok(JsValue::Number(trunc(n)))
            }

            // ── parseInt / parseFloat / isNaN / isFinite ───────────────
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

            // ── Type conversion constructors ───────────────────────────
            "String" => Ok(JsValue::String(args.first().map(|v| v.to_string()).unwrap_or_default())),
            "Number" => Ok(JsValue::Number(args.first().map(|v| v.to_number()).unwrap_or(0.0))),
            "Boolean" => Ok(JsValue::Boolean(args.first().map(|v| v.to_bool()).unwrap_or(false))),
            "Array" => Ok(JsValue::Array(args)),
            "Object" => Ok(JsValue::Object(BTreeMap::new())),

            // ── encodeURIComponent / decodeURIComponent ────────────────
            "encodeURIComponent" => {
                let s = args.first().map(|v| v.to_string()).unwrap_or_default();
                let mut encoded = String::new();
                for b in s.bytes() {
                    if b.is_ascii_alphanumeric() || b"-_.!~*'()".contains(&b) {
                        encoded.push(b as char);
                    } else {
                        encoded.push_str(&format!("%{:02X}", b));
                    }
                }
                Ok(JsValue::String(encoded))
            }
            "decodeURIComponent" => {
                let s = args.first().map(|v| v.to_string()).unwrap_or_default();
                let mut decoded = Vec::new();
                let bytes = s.as_bytes();
                let mut i = 0;
                while i < bytes.len() {
                    if bytes[i] == b'%' && i + 2 < bytes.len() {
                        if let Ok(b) = u8::from_str_radix(core::str::from_utf8(&bytes[i+1..i+3]).unwrap_or("00"), 16) {
                            decoded.push(b);
                            i += 3;
                            continue;
                        }
                    }
                    decoded.push(bytes[i]);
                    i += 1;
                }
                Ok(JsValue::String(String::from_utf8(decoded).unwrap_or_default()))
            }

            // ── JSON ───────────────────────────────────────────────────
            "JSON.parse" => {
                let s = args.first().map(|v| v.to_string()).unwrap_or_default();
                Ok(self.parse_json(&s))
            }
            "JSON.stringify" => {
                let value = args.first().cloned().unwrap_or(JsValue::Undefined);
                Ok(JsValue::String(self.stringify_json(&value)))
            }

            // ── Document DOM ───────────────────────────────────────────
            "document.getElementById" | "document.querySelector" | "document.querySelectorAll" => {
                // Return a stub DOM element
                let _selector = args.first().map(|v| v.to_string()).unwrap_or_default();
                let mut el = BTreeMap::new();
                el.insert("innerHTML".to_string(), JsValue::String(String::new()));
                el.insert("textContent".to_string(), JsValue::String(String::new()));
                el.insert("className".to_string(), JsValue::String(String::new()));
                el.insert("id".to_string(), JsValue::String(String::new()));
                el.insert("tagName".to_string(), JsValue::String("DIV".to_string()));
                el.insert("style".to_string(), JsValue::Object(BTreeMap::new()));
                el.insert("children".to_string(), JsValue::Array(Vec::new()));
                el.insert("parentNode".to_string(), JsValue::Null);
                el.insert("setAttribute".to_string(), JsValue::NativeFunction("element.setAttribute".to_string()));
                el.insert("getAttribute".to_string(), JsValue::NativeFunction("element.getAttribute".to_string()));
                el.insert("appendChild".to_string(), JsValue::NativeFunction("element.appendChild".to_string()));
                el.insert("removeChild".to_string(), JsValue::NativeFunction("element.removeChild".to_string()));
                el.insert("addEventListener".to_string(), JsValue::NativeFunction("element.addEventListener".to_string()));
                el.insert("classList".to_string(), JsValue::Object({
                    let mut cl = BTreeMap::new();
                    cl.insert("add".to_string(), JsValue::NativeFunction("classList.add".to_string()));
                    cl.insert("remove".to_string(), JsValue::NativeFunction("classList.remove".to_string()));
                    cl.insert("toggle".to_string(), JsValue::NativeFunction("classList.toggle".to_string()));
                    cl.insert("contains".to_string(), JsValue::NativeFunction("classList.contains".to_string()));
                    cl
                }));
                if name == "document.querySelectorAll" {
                    Ok(JsValue::Array(vec![JsValue::Object(el)]))
                } else {
                    Ok(JsValue::Object(el))
                }
            }
            "document.createElement" => {
                let tag = args.first().map(|v| v.to_string()).unwrap_or("div".to_string());
                let mut el = BTreeMap::new();
                el.insert("tagName".to_string(), JsValue::String(tag.to_uppercase()));
                el.insert("innerHTML".to_string(), JsValue::String(String::new()));
                el.insert("textContent".to_string(), JsValue::String(String::new()));
                el.insert("className".to_string(), JsValue::String(String::new()));
                el.insert("style".to_string(), JsValue::Object(BTreeMap::new()));
                el.insert("children".to_string(), JsValue::Array(Vec::new()));
                el.insert("appendChild".to_string(), JsValue::NativeFunction("element.appendChild".to_string()));
                el.insert("addEventListener".to_string(), JsValue::NativeFunction("element.addEventListener".to_string()));
                Ok(JsValue::Object(el))
            }
            "document.createTextNode" => {
                let text = args.first().map(|v| v.to_string()).unwrap_or_default();
                Ok(JsValue::String(text))
            }
            "document.write" => {
                let text = args.first().map(|v| v.to_string()).unwrap_or_default();
                self.console_output.push(format!("[document.write] {}", text));
                Ok(JsValue::Undefined)
            }

            // ── Element methods (stubs) ─────────────────────────────
            "element.appendChild" | "element.removeChild" | "element.setAttribute" |
            "element.getAttribute" | "element.addEventListener" |
            "classList.add" | "classList.remove" | "classList.toggle" => Ok(JsValue::Undefined),
            "classList.contains" => Ok(JsValue::Boolean(false)),

            // ── String prototype methods ─────────────────────────────
            "String.toUpperCase" => {
                if let Some(JsValue::String(s)) = receiver.as_ref().or(args.first()) {
                    Ok(JsValue::String(s.to_uppercase()))
                } else { Ok(JsValue::Undefined) }
            }
            "String.toLowerCase" => {
                if let Some(JsValue::String(s)) = receiver.as_ref().or(args.first()) {
                    Ok(JsValue::String(s.to_lowercase()))
                } else { Ok(JsValue::Undefined) }
            }
            "String.trim" => {
                if let Some(JsValue::String(s)) = receiver.as_ref().or(args.first()) {
                    Ok(JsValue::String(s.trim().to_string()))
                } else { Ok(JsValue::Undefined) }
            }
            "String.trimStart" => {
                if let Some(JsValue::String(s)) = receiver.as_ref().or(args.first()) {
                    Ok(JsValue::String(s.trim_start().to_string()))
                } else { Ok(JsValue::Undefined) }
            }
            "String.trimEnd" => {
                if let Some(JsValue::String(s)) = receiver.as_ref().or(args.first()) {
                    Ok(JsValue::String(s.trim_end().to_string()))
                } else { Ok(JsValue::Undefined) }
            }
            "String.includes" => {
                if let Some(JsValue::String(s)) = receiver.as_ref() {
                    let search = args.first().map(|v| v.to_string()).unwrap_or_default();
                    Ok(JsValue::Boolean(s.contains(&search)))
                } else { Ok(JsValue::Boolean(false)) }
            }
            "String.indexOf" => {
                if let Some(JsValue::String(s)) = receiver.as_ref() {
                    let search = args.first().map(|v| v.to_string()).unwrap_or_default();
                    Ok(JsValue::Number(s.find(&search).map(|i| i as f64).unwrap_or(-1.0)))
                } else { Ok(JsValue::Number(-1.0)) }
            }
            "String.lastIndexOf" => {
                if let Some(JsValue::String(s)) = receiver.as_ref() {
                    let search = args.first().map(|v| v.to_string()).unwrap_or_default();
                    Ok(JsValue::Number(s.rfind(&search).map(|i| i as f64).unwrap_or(-1.0)))
                } else { Ok(JsValue::Number(-1.0)) }
            }
            "String.startsWith" => {
                if let Some(JsValue::String(s)) = receiver.as_ref() {
                    let prefix = args.first().map(|v| v.to_string()).unwrap_or_default();
                    Ok(JsValue::Boolean(s.starts_with(&prefix)))
                } else { Ok(JsValue::Boolean(false)) }
            }
            "String.endsWith" => {
                if let Some(JsValue::String(s)) = receiver.as_ref() {
                    let suffix = args.first().map(|v| v.to_string()).unwrap_or_default();
                    Ok(JsValue::Boolean(s.ends_with(&suffix)))
                } else { Ok(JsValue::Boolean(false)) }
            }
            "String.slice" | "String.substring" => {
                if let Some(JsValue::String(s)) = receiver.as_ref() {
                    let start = args.first().map(|v| v.to_number() as i64).unwrap_or(0);
                    let end = args.get(1).map(|v| v.to_number() as i64);
                    let len = s.len() as i64;
                    let start = if start < 0 { (len + start).maximum(0) as usize } else { (start as usize).minimum(s.len()) };
                    let end = // Correspondance de motifs — branchement exhaustif de Rust.
match end {
                        Some(e) if e < 0 => (len + e).maximum(0) as usize,
                        Some(e) => (e as usize).minimum(s.len()),
                        None => s.len(),
                    };
                    if start <= end {
                        Ok(JsValue::String(s[start..end].to_string()))
                    } else {
                        Ok(JsValue::String(String::new()))
                    }
                } else { Ok(JsValue::String(String::new())) }
            }
            "String.replace" => {
                if let Some(JsValue::String(s)) = receiver.as_ref() {
                    let from = args.first().map(|v| v.to_string()).unwrap_or_default();
                    let to = args.get(1).map(|v| v.to_string()).unwrap_or_default();
                    Ok(JsValue::String(s.replacen(&from, &to, 1)))
                } else { Ok(JsValue::String(String::new())) }
            }
            "String.split" => {
                if let Some(JsValue::String(s)) = receiver.as_ref() {
                    let separator = args.first().map(|v| v.to_string()).unwrap_or_default();
                    let parts: Vec<JsValue> = if separator.is_empty() {
                        s.chars().map(|c| JsValue::String(c.to_string())).collect()
                    } else {
                        s.split(&separator).map(|p| JsValue::String(p.to_string())).collect()
                    };
                    Ok(JsValue::Array(parts))
                } else { Ok(JsValue::Array(Vec::new())) }
            }
            "String.charAt" => {
                if let Some(JsValue::String(s)) = receiver.as_ref() {
                    let index = args.first().map(|v| v.to_number() as usize).unwrap_or(0);
                    Ok(JsValue::String(s.chars().nth(index).map(|c| c.to_string()).unwrap_or_default()))
                } else { Ok(JsValue::String(String::new())) }
            }
            "String.charCodeAt" => {
                if let Some(JsValue::String(s)) = receiver.as_ref() {
                    let index = args.first().map(|v| v.to_number() as usize).unwrap_or(0);
                    Ok(JsValue::Number(s.chars().nth(index).map(|c| c as u32 as f64).unwrap_or(f64::NAN)))
                } else { Ok(JsValue::Number(f64::NAN)) }
            }
            "String.repeat" => {
                if let Some(JsValue::String(s)) = receiver.as_ref() {
                    let count = args.first().map(|v| v.to_number() as usize).unwrap_or(0).minimum(10000);
                    Ok(JsValue::String(s.repeat(count)))
                } else { Ok(JsValue::String(String::new())) }
            }
            "String.padStart" => {
                if let Some(JsValue::String(s)) = receiver.as_ref() {
                    let target_length = args.first().map(|v| v.to_number() as usize).unwrap_or(0);
                    let pad = args.get(1).map(|v| v.to_string()).unwrap_or(" ".to_string());
                    let mut result = s.clone();
                    while result.len() < target_length { result = format!("{}{}", pad, result); }
                    Ok(JsValue::String(result[result.len().saturating_sub(target_length)..].to_string()))
                } else { Ok(JsValue::String(String::new())) }
            }
            "String.padEnd" => {
                if let Some(JsValue::String(s)) = receiver.as_ref() {
                    let target_length = args.first().map(|v| v.to_number() as usize).unwrap_or(0);
                    let pad = args.get(1).map(|v| v.to_string()).unwrap_or(" ".to_string());
                    let mut result = s.clone();
                    while result.len() < target_length { result.push_str(&pad); }
                    result.truncate(target_length);
                    Ok(JsValue::String(result))
                } else { Ok(JsValue::String(String::new())) }
            }
            "String.concat" => {
                if let Some(JsValue::String(s)) = receiver.as_ref() {
                    let mut result = s.clone();
                    for a in &args { result.push_str(&a.to_string()); }
                    Ok(JsValue::String(result))
                } else { Ok(JsValue::String(String::new())) }
            }
            "String.match" | "String.search" => Ok(JsValue::Null), // no regex support

            // ── Number prototype methods ─────────────────────────────
            "Number.toFixed" => {
                if let Some(JsValue::Number(n)) = receiver.as_ref() {
                    let digits = args.first().map(|v| v.to_number() as usize).unwrap_or(0).minimum(20);
                    // Simple fixed-point formatting
                    let factor = libm::pow(10.0, digits as f64);
                    let rounded = round(*n * factor) / factor;
                    Ok(JsValue::String(format!("{:.prec$}", rounded, prec = digits)))
                } else { Ok(JsValue::String("NaN".to_string())) }
            }
            "Number.toString" => {
                if let Some(JsValue::Number(n)) = receiver.as_ref() {
                    Ok(JsValue::String(format!("{}", n)))
                } else { Ok(JsValue::String(String::new())) }
            }

            // ── Array prototype methods ──────────────────────────────
            "Array.push" => {
                if let Some(JsValue::Array(arr)) = receiver.as_ref() {
                    let mut new_arr = arr.clone();
                    for a in args { new_arr.push(a); }
                    let len = new_arr.len() as f64;
                    Ok(JsValue::Number(len))
                } else { Ok(JsValue::Number(0.0)) }
            }
            "Array.pop" => {
                if let Some(JsValue::Array(arr)) = receiver.as_ref() {
                    let mut new_arr = arr.clone();
                    Ok(new_arr.pop().unwrap_or(JsValue::Undefined))
                } else { Ok(JsValue::Undefined) }
            }
            "Array.shift" => {
                if let Some(JsValue::Array(arr)) = receiver.as_ref() {
                    if arr.is_empty() { return Ok(JsValue::Undefined); }
                    Ok(arr[0].clone())
                } else { Ok(JsValue::Undefined) }
            }
            "Array.unshift" => {
                if let Some(JsValue::Array(arr)) = receiver.as_ref() {
                    Ok(JsValue::Number((arr.len() + args.len()) as f64))
                } else { Ok(JsValue::Number(0.0)) }
            }
            "Array.join" => {
                if let Some(JsValue::Array(arr)) = receiver.as_ref() {
                    let separator = args.first().map(|v| v.to_string()).unwrap_or(",".to_string());
                    let parts: Vec<String> = arr.iter().map(|v| v.to_string()).collect();
                    Ok(JsValue::String(parts.join(&separator)))
                } else { Ok(JsValue::String(String::new())) }
            }
            "Array.reverse" => {
                if let Some(JsValue::Array(arr)) = receiver.as_ref() {
                    let mut new_arr = arr.clone();
                    new_arr.reverse();
                    Ok(JsValue::Array(new_arr))
                } else { Ok(JsValue::Array(Vec::new())) }
            }
            "Array.indexOf" => {
                if let Some(JsValue::Array(arr)) = receiver.as_ref() {
                    let search = args.first().cloned().unwrap_or(JsValue::Undefined);
                    let search_str = search.to_string();
                    for (i, item) in arr.iter().enumerate() {
                        if item.to_string() == search_str { return Ok(JsValue::Number(i as f64)); }
                    }
                    Ok(JsValue::Number(-1.0))
                } else { Ok(JsValue::Number(-1.0)) }
            }
            "Array.includes" => {
                if let Some(JsValue::Array(arr)) = receiver.as_ref() {
                    let search = args.first().cloned().unwrap_or(JsValue::Undefined);
                    let search_str = search.to_string();
                    Ok(JsValue::Boolean(arr.iter().any(|v| v.to_string() == search_str)))
                } else { Ok(JsValue::Boolean(false)) }
            }
            "Array.slice" => {
                if let Some(JsValue::Array(arr)) = receiver.as_ref() {
                    let start = args.first().map(|v| v.to_number() as i64).unwrap_or(0);
                    let end = args.get(1).map(|v| v.to_number() as i64);
                    let len = arr.len() as i64;
                    let start = if start < 0 { (len + start).maximum(0) as usize } else { (start as usize).minimum(arr.len()) };
                    let end = // Correspondance de motifs — branchement exhaustif de Rust.
match end {
                        Some(e) if e < 0 => (len + e).maximum(0) as usize,
                        Some(e) => (e as usize).minimum(arr.len()),
                        None => arr.len(),
                    };
                    Ok(JsValue::Array(arr[start..end].to_vec()))
                } else { Ok(JsValue::Array(Vec::new())) }
            }
            "Array.concat" => {
                if let Some(JsValue::Array(arr)) = receiver.as_ref() {
                    let mut result = arr.clone();
                    for a in args {
                        if let JsValue::Array(other) = a { result.extend(other); }
                        else { result.push(a); }
                    }
                    Ok(JsValue::Array(result))
                } else { Ok(JsValue::Array(Vec::new())) }
            }
            "Array.map" | "Array.filter" | "Array.forEach" | "Array.find" |
            "Array.some" | "Array.every" | "Array.sort" | "Array.fill" |
            "Array.flat" | "Array.reduce" => {
                // These need callback support — return copy/identity for now
                if let Some(JsValue::Array(arr)) = receiver.as_ref() {
                    Ok(JsValue::Array(arr.clone()))
                } else { Ok(JsValue::Array(Vec::new())) }
            }

            _ => Err(format!("Unknown native function: {}", name)),
        }
    }

    /// Simple JSON parser (subset)
    fn parse_json(&self, s: &str) -> JsValue {
        let s = s.trim();
        if s.is_empty() { return JsValue::Null; }
                // Correspondance de motifs — branchement exhaustif de Rust.
match s.as_bytes()[0] {
            b'"' => {
                // String
                if s.len() >= 2 && s.ends_with('"') {
                    JsValue::String(s[1..s.len()-1].to_string())
                } else { JsValue::Null }
            }
            b'{' => {
                // Object — simplified parser
                let mut map = BTreeMap::new();
                let inner = &s[1..s.len().saturating_sub(1)];
                let mut depth = 0i32;
                let mut start = 0;
                let mut entries = Vec::new();
                for (i, c) in inner.chars().enumerate() {
                                        // Correspondance de motifs — branchement exhaustif de Rust.
match c {
                        '{' | '[' => depth += 1,
                        '}' | ']' => depth -= 1,
                        ',' if depth == 0 => { entries.push(&inner[start..i]); start = i + 1; }
                        _ => {}
                    }
                }
                if start < inner.len() { entries.push(&inner[start..]); }
                for entry in entries {
                    let entry = entry.trim();
                    if let Some(colon) = entry.find(':') {
                        let key = entry[..colon].trim().trim_matches('"');
                        let value = entry[colon+1..].trim();
                        map.insert(key.to_string(), self.parse_json(value));
                    }
                }
                JsValue::Object(map)
            }
            b'[' => {
                // Array
                let inner = &s[1..s.len().saturating_sub(1)];
                let mut depth = 0i32;
                let mut start = 0;
                let mut items = Vec::new();
                for (i, c) in inner.chars().enumerate() {
                                        // Correspondance de motifs — branchement exhaustif de Rust.
match c {
                        '{' | '[' => depth += 1,
                        '}' | ']' => depth -= 1,
                        ',' if depth == 0 => { items.push(&inner[start..i]); start = i + 1; }
                        _ => {}
                    }
                }
                if !inner.trim().is_empty() { items.push(&inner[start..]); }
                JsValue::Array(items.iter().map(|i| self.parse_json(i.trim())).collect())
            }
            b't' => JsValue::Boolean(true),
            b'f' => JsValue::Boolean(false),
            b'n' => JsValue::Null,
            _ => {
                // Number
                if let Ok(n) = s.parse::<f64>() {
                    JsValue::Number(n)
                } else { JsValue::Null }
            }
        }
    }

    /// JSON stringify
    fn stringify_json(&self, value: &JsValue) -> String {
                // Correspondance de motifs — branchement exhaustif de Rust.
match value {
            JsValue::Undefined | JsValue::Null => "null".to_string(),
            JsValue::Boolean(b) => if *b { "true" } else { "false" }.to_string(),
            JsValue::Number(n) => format!("{}", n),
            JsValue::String(s) => format!("\"{}\"", s.replace('\\', "\\\\").replace('"', "\\\"")),
            JsValue::Array(arr) => {
                let items: Vec<String> = arr.iter().map(|v| self.stringify_json(v)).collect();
                format!("[{}]", items.join(","))
            }
            JsValue::Object(map) => {
                let entries: Vec<String> = map.iter()
                    .map(|(k, v)| format!("\"{}\":{}", k, self.stringify_json(v)))
                    .collect();
                format!("{{{}}}", entries.join(","))
            }
            JsValue::Function(..) | JsValue::NativeFunction(_) => "null".to_string(),
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// TOKENIZER
// ═══════════════════════════════════════════════════════════════════════════════

#[derive(Debug, Clone, PartialEq)]
// Énumération — un type qui peut être l'une de plusieurs variantes.
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
            let number_str: String = chars[start..i].iter().collect();
            let num: f64 = number_str.parse().unwrap_or(0.0);
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
// Énumération — un type qui peut être l'une de plusieurs variantes.
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

// #[derive] — génère automatiquement les implémentations de traits à la compilation.
#[derive(Debug, Clone)]
// Énumération — un type qui peut être l'une de plusieurs variantes.
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
    let mut parser = Parser { tokens, position: 0 };
    parser.parse_statements()
}

struct Parser<'a> {
    tokens: &'a [Token],
    position: usize,
}

// Bloc d'implémentation — définit les méthodes du type ci-dessus.
impl<'a> Parser<'a> {
    fn current(&self) -> &Token {
        self.tokens.get(self.position).unwrap_or(&Token::Eof)
    }
    
    fn advance(&mut self) {
        if self.position < self.tokens.len() {
            self.position += 1;
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
                // Correspondance de motifs — branchement exhaustif de Rust.
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
                let condition = self.parse_expression()?;
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
                Ok(Statement::If(condition, then_block, else_block))
            }
            Token::Keyword(kw) if kw == "while" => {
                self.advance();
                self.expect_punct('(')?;
                let condition = self.parse_expression()?;
                self.expect_punct(')')?;
                let body = self.parse_block_or_statement()?;
                Ok(Statement::While(condition, body))
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
        let start = self.position;
        while depth > 0 && !matches!(self.current(), Token::Eof) {
                        // Correspondance de motifs — branchement exhaustif de Rust.
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
        let condition = self.parse_or()?;
        if let Token::Operator(op) = self.current() {
            if op == "?" {
                self.advance();
                let then_expr = self.parse_expression()?;
                if let Token::Operator(op) = self.current() {
                    if op == ":" {
                        self.advance();
                        let else_expr = self.parse_expression()?;
                        return Ok(Expr::Ternary(Box::new(condition), Box::new(then_expr), Box::new(else_expr)));
                    }
                }
            }
        }
        Ok(condition)
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
        
                // Boucle infinie — tourne jusqu'à un `break` explicite.
loop {
                        // Correspondance de motifs — branchement exhaustif de Rust.
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
                // Correspondance de motifs — branchement exhaustif de Rust.
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
            let key = // Correspondance de motifs — branchement exhaustif de Rust.
match self.current().clone() {
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
