






use alloc::string::{String, ToString};
use alloc::vec::Vec;
use alloc::vec;
use alloc::boxed::Box;
use alloc::collections::BTreeMap;
use alloc::format;
use libm::{floor, ceil, round, sqrt, trunc};


#[derive(Debug, Clone)]
pub enum JsValue {
    Undefined,
    Null,
    Boolean(bool),
    Number(f64),
    String(String),
    Object(BTreeMap<String, JsValue>),
    Array(Vec<JsValue>),
    Aq(String, Vec<String>, String), 
    NativeFunction(String),
}

impl JsValue {
    
    pub fn to_bool(&self) -> bool {
        match self {
            JsValue::Undefined | JsValue::Null => false,
            JsValue::Boolean(b) => *b,
            JsValue::Number(ae) => *ae != 0.0 && !ae.is_nan(),
            JsValue::String(j) => !j.is_empty(),
            JsValue::Object(_) | JsValue::Array(_) => true,
            JsValue::Aq(..) | JsValue::NativeFunction(_) => true,
        }
    }
    
    
    pub fn to_number(&self) -> f64 {
        match self {
            JsValue::Undefined => f64::NAN,
            JsValue::Null => 0.0,
            JsValue::Boolean(true) => 1.0,
            JsValue::Boolean(false) => 0.0,
            JsValue::Number(ae) => *ae,
            JsValue::String(j) => j.parse().unwrap_or(f64::NAN),
            _ => f64::NAN,
        }
    }
    
    
    pub fn to_string(&self) -> String {
        match self {
            JsValue::Undefined => "undefined".to_string(),
            JsValue::Null => "null".to_string(),
            JsValue::Boolean(true) => "true".to_string(),
            JsValue::Boolean(false) => "false".to_string(),
            JsValue::Number(ae) => format!("{}", ae),
            JsValue::String(j) => j.clone(),
            JsValue::Object(_) => "[object Object]".to_string(),
            JsValue::Array(ik) => {
                let au: Vec<String> = ik.iter().map(|v| v.to_string()).collect();
                au.join(",")
            }
            JsValue::Aq(name, ..) => format!("function {}() {{ [native code] }}", name),
            JsValue::NativeFunction(name) => format!("function {}() {{ [native code] }}", name),
        }
    }
}


pub struct JsContext {
    pub global: BTreeMap<String, JsValue>,
    pub console_output: Vec<String>,
}

impl JsContext {
    pub fn new() -> Self {
        let mut ab = Self {
            global: BTreeMap::new(),
            console_output: Vec::new(),
        };
        
        
        ab.setup_builtins();
        ab
    }
    
    fn setup_builtins(&mut self) {
        
        let mut console = BTreeMap::new();
        console.insert("log".to_string(), JsValue::NativeFunction("console.log".to_string()));
        console.insert("warn".to_string(), JsValue::NativeFunction("console.warn".to_string()));
        console.insert("error".to_string(), JsValue::NativeFunction("console.error".to_string()));
        console.insert("info".to_string(), JsValue::NativeFunction("console.log".to_string()));
        console.insert("debug".to_string(), JsValue::NativeFunction("console.log".to_string()));
        self.global.insert("console".to_string(), JsValue::Object(console));
        
        
        let mut math = BTreeMap::new();
        math.insert("PI".to_string(), JsValue::Number(core::f64::consts::PI));
        math.insert("E".to_string(), JsValue::Number(core::f64::consts::E));
        math.insert("LN2".to_string(), JsValue::Number(core::f64::consts::LN_2));
        math.insert("LN10".to_string(), JsValue::Number(core::f64::consts::LN_10));
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
        
        
        let mut gei = BTreeMap::new();
        gei.insert("parse".to_string(), JsValue::NativeFunction("JSON.parse".to_string()));
        gei.insert("stringify".to_string(), JsValue::NativeFunction("JSON.stringify".to_string()));
        self.global.insert("JSON".to_string(), JsValue::Object(gei));
        
        
        let mut document = BTreeMap::new();
        document.insert("getElementById".to_string(), JsValue::NativeFunction("document.getElementById".to_string()));
        document.insert("querySelector".to_string(), JsValue::NativeFunction("document.querySelector".to_string()));
        document.insert("querySelectorAll".to_string(), JsValue::NativeFunction("document.querySelectorAll".to_string()));
        document.insert("createElement".to_string(), JsValue::NativeFunction("document.createElement".to_string()));
        document.insert("createTextNode".to_string(), JsValue::NativeFunction("document.createTextNode".to_string()));
        document.insert("write".to_string(), JsValue::NativeFunction("document.write".to_string()));
        document.insert("title".to_string(), JsValue::String("TrustOS Browser".to_string()));
        document.insert("readyState".to_string(), JsValue::String("complete".to_string()));
        
        
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
        
        
        let mut window = BTreeMap::new();
        window.insert("innerWidth".to_string(), JsValue::Number(1024.0));
        window.insert("innerHeight".to_string(), JsValue::Number(768.0));
        window.insert("location".to_string(), JsValue::Object(BTreeMap::new()));
        window.insert("navigator".to_string(), JsValue::Object({
            let mut eut = BTreeMap::new();
            eut.insert("userAgent".to_string(), JsValue::String("TrustOS/1.0".to_string()));
            eut.insert("platform".to_string(), JsValue::String("TrustOS".to_string()));
            eut.insert("language".to_string(), JsValue::String("en-US".to_string()));
            eut
        }));
        self.global.insert("window".to_string(), JsValue::Object(window));
        
        
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
    
    
    pub fn execute(&mut self, code: &str) -> Result<JsValue, String> {
        let tokens = crv(code)?;
        let dhy = parse(&tokens)?;
        self.eval_statements(&dhy)
    }
    
    
    fn eval_statements(&mut self, stmts: &[Statement]) -> Result<JsValue, String> {
        let mut result = JsValue::Undefined;
        for stmt in stmts {
            result = self.eval_statement(stmt)?;
        }
        Ok(result)
    }
    
    
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
            Statement::If(fc, avj, atp) => {
                let kwr = self.eval_expr(fc)?;
                if kwr.to_bool() {
                    self.eval_statements(avj)
                } else if let Some(else_stmts) = atp {
                    self.eval_statements(else_stmts)
                } else {
                    Ok(JsValue::Undefined)
                }
            }
            Statement::While(fc, body) => {
                while self.eval_expr(fc)?.to_bool() {
                    self.eval_statements(body)?;
                }
                Ok(JsValue::Undefined)
            }
            Statement::For(init, fc, update, body) => {
                if let Some(init_stmt) = init {
                    self.eval_statement(init_stmt)?;
                }
                while fc.as_ref().map(|c| self.eval_expr(c).map(|v| v.to_bool()).unwrap_or(false)).unwrap_or(true) {
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
            Statement::Aq(name, params, body) => {
                self.global.insert(
                    name.clone(),
                    JsValue::Aq(name.clone(), params.clone(), body.clone()),
                );
                Ok(JsValue::Undefined)
            }
            Statement::Bl(stmts) => self.eval_statements(stmts),
        }
    }
    
    
    fn eval_expr(&mut self, expr: &Expr) -> Result<JsValue, String> {
        match expr {
            Expr::Literal(lit) => Ok(lit.clone()),
            Expr::Identifier(name) => {
                self.global.get(name).cloned().ok_or_else(|| format!("ReferenceError: {} is not defined", name))
            }
            Expr::Binary(op, left, right) => {
                let nbg = self.eval_expr(left)?;
                let ojj = self.eval_expr(right)?;
                self.eval_binary_op(op, nbg, ojj)
            }
            Expr::Unary(op, dvw) => {
                let val = self.eval_expr(dvw)?;
                self.eval_unary_op(op, val)
            }
            Expr::Call(callee, args) => {
                
                let (func, receiver) = if let Expr::Member(obj_expr, _prop) = callee.as_ref() {
                    let recv = self.eval_expr(obj_expr)?;
                    let f = self.eval_expr(callee)?;
                    (f, Some(recv))
                } else {
                    (self.eval_expr(callee)?, None)
                };
                let mut hfq = Vec::new();
                for db in args {
                    hfq.push(self.eval_expr(db)?);
                }
                self.call_function_with_receiver(func, hfq, receiver)
            }
            Expr::Member(obj, prop) => {
                let gkg = self.eval_expr(obj)?;
                match &gkg {
                    JsValue::Object(map) => {
                        Ok(map.get(prop).cloned().unwrap_or(JsValue::Undefined))
                    }
                    JsValue::Array(ik) => {
                        match prop.as_str() {
                            "length" => Ok(JsValue::Number(ik.len() as f64)),
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
                                if let Ok(idx) = prop.parse::<usize>() {
                                    Ok(ik.get(idx).cloned().unwrap_or(JsValue::Undefined))
                                } else {
                                    Ok(JsValue::Undefined)
                                }
                            }
                        }
                    }
                    JsValue::String(j) => {
                        match prop.as_str() {
                            "length" => Ok(JsValue::Number(j.len() as f64)),
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
                        match prop.as_str() {
                            "toFixed" => Ok(JsValue::NativeFunction("Number.toFixed".to_string())),
                            "toString" => Ok(JsValue::NativeFunction("Number.toString".to_string())),
                            _ => Ok(JsValue::Undefined),
                        }
                    }
                    _ => Ok(JsValue::Undefined),
                }
            }
            Expr::Index(obj, idx) => {
                let gkg = self.eval_expr(obj)?;
                let ifq = self.eval_expr(idx)?;
                match gkg {
                    JsValue::Array(ik) => {
                        let i = ifq.to_number() as usize;
                        Ok(ik.get(i).cloned().unwrap_or(JsValue::Undefined))
                    }
                    JsValue::Object(map) => {
                        let key = ifq.to_string();
                        Ok(map.get(&key).cloned().unwrap_or(JsValue::Undefined))
                    }
                    _ => Ok(JsValue::Undefined),
                }
            }
            Expr::Array(elements) => {
                let mut ik = Vec::new();
                for el in elements {
                    ik.push(self.eval_expr(el)?);
                }
                Ok(JsValue::Array(ik))
            }
            Expr::Object(dcz) => {
                let mut map = BTreeMap::new();
                for (key, val) in dcz {
                    map.insert(key.clone(), self.eval_expr(val)?);
                }
                Ok(JsValue::Object(map))
            }
            Expr::Assign(name, value) => {
                let val = self.eval_expr(value)?;
                self.global.insert(name.clone(), val.clone());
                Ok(val)
            }
            Expr::Ternary(fc, gyn, fuh) => {
                if self.eval_expr(fc)?.to_bool() {
                    self.eval_expr(gyn)
                } else {
                    self.eval_expr(fuh)
                }
            }
        }
    }
    
    fn eval_binary_op(&self, op: &str, left: JsValue, right: JsValue) -> Result<JsValue, String> {
        match op {
            "+" => {
                
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
                    JsValue::Null => "object", 
                    JsValue::Boolean(_) => "boolean",
                    JsValue::Number(_) => "number",
                    JsValue::String(_) => "string",
                    JsValue::Object(_) | JsValue::Array(_) => "object",
                    JsValue::Aq(..) | JsValue::NativeFunction(_) => "function",
                };
                Ok(JsValue::String(t.to_string()))
            }
            _ => Err(format!("Unknown unary operator: {}", op)),
        }
    }
    
    fn call_function_with_receiver(&mut self, func: JsValue, args: Vec<JsValue>, receiver: Option<JsValue>) -> Result<JsValue, String> {
        match func {
            JsValue::NativeFunction(name) => self.call_native_with_receiver(&name, args, receiver),
            JsValue::Aq(_name, params, body) => {
                for (i, param) in params.iter().enumerate() {
                    let val = args.get(i).cloned().unwrap_or(JsValue::Undefined);
                    self.global.insert(param.clone(), val);
                }
                self.execute(&body)
            }
            _ => Err("TypeError: not a function".to_string()),
        }
    }

    fn pze(&mut self, func: JsValue, args: Vec<JsValue>) -> Result<JsValue, String> {
        self.call_function_with_receiver(func, args, None)
    }
    
    fn call_native_with_receiver(&mut self, name: &str, args: Vec<JsValue>, receiver: Option<JsValue>) -> Result<JsValue, String> {
        match name {
            
            "console.log" | "console.warn" | "console.error" => {
                let output: Vec<String> = args.iter().map(|v| v.to_string()).collect();
                let line = output.join(" ");
                self.console_output.push(line);
                Ok(JsValue::Undefined)
            }
            "alert" => {
                if let Some(bk) = args.first() {
                    self.console_output.push(format!("[ALERT] {}", bk.to_string()));
                }
                Ok(JsValue::Undefined)
            }

            
            "setTimeout" | "setInterval" => Ok(JsValue::Number(0.0)),
            "clearTimeout" | "clearInterval" => Ok(JsValue::Undefined),

            
            "Math.random" => {
                let seed = crate::cpu::tsc::ey();
                let obe = ((seed >> 16) as f64) / 65536.0;
                Ok(JsValue::Number(obe % 1.0))
            }
            "Math.floor" => {
                let ae = args.first().map(|v| v.to_number()).unwrap_or(0.0);
                Ok(JsValue::Number(floor(ae)))
            }
            "Math.ceil" => {
                let ae = args.first().map(|v| v.to_number()).unwrap_or(0.0);
                Ok(JsValue::Number(ceil(ae)))
            }
            "Math.round" => {
                let ae = args.first().map(|v| v.to_number()).unwrap_or(0.0);
                Ok(JsValue::Number(round(ae)))
            }
            "Math.abs" => {
                let ae = args.first().map(|v| v.to_number()).unwrap_or(0.0);
                Ok(JsValue::Number(libm::fabs(ae)))
            }
            "Math.sqrt" => {
                let ae = args.first().map(|v| v.to_number()).unwrap_or(0.0);
                Ok(JsValue::Number(sqrt(ae)))
            }
            "Math.min" => {
                if args.is_empty() { return Ok(JsValue::Number(f64::INFINITY)); }
                let mut m = args[0].to_number();
                for a in &args[1..] { let v = a.to_number(); if v < m { m = v; } }
                Ok(JsValue::Number(m))
            }
            "Math.max" => {
                if args.is_empty() { return Ok(JsValue::Number(f64::NEG_INFINITY)); }
                let mut m = args[0].to_number();
                for a in &args[1..] { let v = a.to_number(); if v > m { m = v; } }
                Ok(JsValue::Number(m))
            }
            "Math.pow" => {
                let base = args.first().map(|v| v.to_number()).unwrap_or(0.0);
                let afe = args.get(1).map(|v| v.to_number()).unwrap_or(0.0);
                Ok(JsValue::Number(libm::pow(base, afe)))
            }
            "Math.sin" => {
                let ae = args.first().map(|v| v.to_number()).unwrap_or(0.0);
                Ok(JsValue::Number(libm::sin(ae)))
            }
            "Math.cos" => {
                let ae = args.first().map(|v| v.to_number()).unwrap_or(0.0);
                Ok(JsValue::Number(libm::cos(ae)))
            }
            "Math.tan" => {
                let ae = args.first().map(|v| v.to_number()).unwrap_or(0.0);
                Ok(JsValue::Number(libm::tan(ae)))
            }
            "Math.log" => {
                let ae = args.first().map(|v| v.to_number()).unwrap_or(0.0);
                Ok(JsValue::Number(libm::log(ae)))
            }
            "Math.sign" => {
                let ae = args.first().map(|v| v.to_number()).unwrap_or(0.0);
                Ok(JsValue::Number(if ae > 0.0 { 1.0 } else if ae < 0.0 { -1.0 } else { 0.0 }))
            }
            "Math.trunc" => {
                let ae = args.first().map(|v| v.to_number()).unwrap_or(0.0);
                Ok(JsValue::Number(trunc(ae)))
            }

            
            "parseInt" => {
                let j = args.first().map(|v| v.to_string()).unwrap_or_default();
                let ae: f64 = j.trim().parse().unwrap_or(f64::NAN);
                Ok(JsValue::Number(trunc(ae)))
            }
            "parseFloat" => {
                let j = args.first().map(|v| v.to_string()).unwrap_or_default();
                let ae: f64 = j.trim().parse().unwrap_or(f64::NAN);
                Ok(JsValue::Number(ae))
            }
            "isNaN" => {
                let ae = args.first().map(|v| v.to_number()).unwrap_or(f64::NAN);
                Ok(JsValue::Boolean(ae.is_nan()))
            }
            "isFinite" => {
                let ae = args.first().map(|v| v.to_number()).unwrap_or(f64::NAN);
                Ok(JsValue::Boolean(ae.is_finite()))
            }

            
            "String" => Ok(JsValue::String(args.first().map(|v| v.to_string()).unwrap_or_default())),
            "Number" => Ok(JsValue::Number(args.first().map(|v| v.to_number()).unwrap_or(0.0))),
            "Boolean" => Ok(JsValue::Boolean(args.first().map(|v| v.to_bool()).unwrap_or(false))),
            "Array" => Ok(JsValue::Array(args)),
            "Object" => Ok(JsValue::Object(BTreeMap::new())),

            
            "encodeURIComponent" => {
                let j = args.first().map(|v| v.to_string()).unwrap_or_default();
                let mut atq = String::new();
                for b in j.bytes() {
                    if b.is_ascii_alphanumeric() || b"-_.!~*'()".contains(&b) {
                        atq.push(b as char);
                    } else {
                        atq.push_str(&format!("%{:02X}", b));
                    }
                }
                Ok(JsValue::String(atq))
            }
            "decodeURIComponent" => {
                let j = args.first().map(|v| v.to_string()).unwrap_or_default();
                let mut uu = Vec::new();
                let bytes = j.as_bytes();
                let mut i = 0;
                while i < bytes.len() {
                    if bytes[i] == b'%' && i + 2 < bytes.len() {
                        if let Ok(b) = u8::from_str_radix(core::str::from_utf8(&bytes[i+1..i+3]).unwrap_or("00"), 16) {
                            uu.push(b);
                            i += 3;
                            continue;
                        }
                    }
                    uu.push(bytes[i]);
                    i += 1;
                }
                Ok(JsValue::String(String::from_utf8(uu).unwrap_or_default()))
            }

            
            "JSON.parse" => {
                let j = args.first().map(|v| v.to_string()).unwrap_or_default();
                Ok(self.parse_json(&j))
            }
            "JSON.stringify" => {
                let val = args.first().cloned().unwrap_or(JsValue::Undefined);
                Ok(JsValue::String(self.stringify_json(&val)))
            }

            
            "document.getElementById" | "document.querySelector" | "document.querySelectorAll" => {
                
                let pxh = args.first().map(|v| v.to_string()).unwrap_or_default();
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

            
            "element.appendChild" | "element.removeChild" | "element.setAttribute" |
            "element.getAttribute" | "element.addEventListener" |
            "classList.add" | "classList.remove" | "classList.toggle" => Ok(JsValue::Undefined),
            "classList.contains" => Ok(JsValue::Boolean(false)),

            
            "String.toUpperCase" => {
                if let Some(JsValue::String(j)) = receiver.as_ref().or(args.first()) {
                    Ok(JsValue::String(j.to_uppercase()))
                } else { Ok(JsValue::Undefined) }
            }
            "String.toLowerCase" => {
                if let Some(JsValue::String(j)) = receiver.as_ref().or(args.first()) {
                    Ok(JsValue::String(j.to_lowercase()))
                } else { Ok(JsValue::Undefined) }
            }
            "String.trim" => {
                if let Some(JsValue::String(j)) = receiver.as_ref().or(args.first()) {
                    Ok(JsValue::String(j.trim().to_string()))
                } else { Ok(JsValue::Undefined) }
            }
            "String.trimStart" => {
                if let Some(JsValue::String(j)) = receiver.as_ref().or(args.first()) {
                    Ok(JsValue::String(j.trim_start().to_string()))
                } else { Ok(JsValue::Undefined) }
            }
            "String.trimEnd" => {
                if let Some(JsValue::String(j)) = receiver.as_ref().or(args.first()) {
                    Ok(JsValue::String(j.trim_end().to_string()))
                } else { Ok(JsValue::Undefined) }
            }
            "String.includes" => {
                if let Some(JsValue::String(j)) = receiver.as_ref() {
                    let search = args.first().map(|v| v.to_string()).unwrap_or_default();
                    Ok(JsValue::Boolean(j.contains(&search)))
                } else { Ok(JsValue::Boolean(false)) }
            }
            "String.indexOf" => {
                if let Some(JsValue::String(j)) = receiver.as_ref() {
                    let search = args.first().map(|v| v.to_string()).unwrap_or_default();
                    Ok(JsValue::Number(j.find(&search).map(|i| i as f64).unwrap_or(-1.0)))
                } else { Ok(JsValue::Number(-1.0)) }
            }
            "String.lastIndexOf" => {
                if let Some(JsValue::String(j)) = receiver.as_ref() {
                    let search = args.first().map(|v| v.to_string()).unwrap_or_default();
                    Ok(JsValue::Number(j.rfind(&search).map(|i| i as f64).unwrap_or(-1.0)))
                } else { Ok(JsValue::Number(-1.0)) }
            }
            "String.startsWith" => {
                if let Some(JsValue::String(j)) = receiver.as_ref() {
                    let nm = args.first().map(|v| v.to_string()).unwrap_or_default();
                    Ok(JsValue::Boolean(j.starts_with(&nm)))
                } else { Ok(JsValue::Boolean(false)) }
            }
            "String.endsWith" => {
                if let Some(JsValue::String(j)) = receiver.as_ref() {
                    let asi = args.first().map(|v| v.to_string()).unwrap_or_default();
                    Ok(JsValue::Boolean(j.ends_with(&asi)))
                } else { Ok(JsValue::Boolean(false)) }
            }
            "String.slice" | "String.substring" => {
                if let Some(JsValue::String(j)) = receiver.as_ref() {
                    let start = args.first().map(|v| v.to_number() as i64).unwrap_or(0);
                    let end = args.get(1).map(|v| v.to_number() as i64);
                    let len = j.len() as i64;
                    let start = if start < 0 { (len + start).max(0) as usize } else { (start as usize).min(j.len()) };
                    let end = match end {
                        Some(e) if e < 0 => (len + e).max(0) as usize,
                        Some(e) => (e as usize).min(j.len()),
                        None => j.len(),
                    };
                    if start <= end {
                        Ok(JsValue::String(j[start..end].to_string()))
                    } else {
                        Ok(JsValue::String(String::new()))
                    }
                } else { Ok(JsValue::String(String::new())) }
            }
            "String.replace" => {
                if let Some(JsValue::String(j)) = receiver.as_ref() {
                    let from = args.first().map(|v| v.to_string()).unwrap_or_default();
                    let to = args.get(1).map(|v| v.to_string()).unwrap_or_default();
                    Ok(JsValue::String(j.replacen(&from, &to, 1)))
                } else { Ok(JsValue::String(String::new())) }
            }
            "String.split" => {
                if let Some(JsValue::String(j)) = receiver.as_ref() {
                    let fad = args.first().map(|v| v.to_string()).unwrap_or_default();
                    let au: Vec<JsValue> = if fad.is_empty() {
                        j.chars().map(|c| JsValue::String(c.to_string())).collect()
                    } else {
                        j.split(&fad).map(|aa| JsValue::String(aa.to_string())).collect()
                    };
                    Ok(JsValue::Array(au))
                } else { Ok(JsValue::Array(Vec::new())) }
            }
            "String.charAt" => {
                if let Some(JsValue::String(j)) = receiver.as_ref() {
                    let idx = args.first().map(|v| v.to_number() as usize).unwrap_or(0);
                    Ok(JsValue::String(j.chars().nth(idx).map(|c| c.to_string()).unwrap_or_default()))
                } else { Ok(JsValue::String(String::new())) }
            }
            "String.charCodeAt" => {
                if let Some(JsValue::String(j)) = receiver.as_ref() {
                    let idx = args.first().map(|v| v.to_number() as usize).unwrap_or(0);
                    Ok(JsValue::Number(j.chars().nth(idx).map(|c| c as u32 as f64).unwrap_or(f64::NAN)))
                } else { Ok(JsValue::Number(f64::NAN)) }
            }
            "String.repeat" => {
                if let Some(JsValue::String(j)) = receiver.as_ref() {
                    let count = args.first().map(|v| v.to_number() as usize).unwrap_or(0).min(10000);
                    Ok(JsValue::String(j.repeat(count)))
                } else { Ok(JsValue::String(String::new())) }
            }
            "String.padStart" => {
                if let Some(JsValue::String(j)) = receiver.as_ref() {
                    let bpi = args.first().map(|v| v.to_number() as usize).unwrap_or(0);
                    let pad = args.get(1).map(|v| v.to_string()).unwrap_or(" ".to_string());
                    let mut result = j.clone();
                    while result.len() < bpi { result = format!("{}{}", pad, result); }
                    Ok(JsValue::String(result[result.len().saturating_sub(bpi)..].to_string()))
                } else { Ok(JsValue::String(String::new())) }
            }
            "String.padEnd" => {
                if let Some(JsValue::String(j)) = receiver.as_ref() {
                    let bpi = args.first().map(|v| v.to_number() as usize).unwrap_or(0);
                    let pad = args.get(1).map(|v| v.to_string()).unwrap_or(" ".to_string());
                    let mut result = j.clone();
                    while result.len() < bpi { result.push_str(&pad); }
                    result.truncate(bpi);
                    Ok(JsValue::String(result))
                } else { Ok(JsValue::String(String::new())) }
            }
            "String.concat" => {
                if let Some(JsValue::String(j)) = receiver.as_ref() {
                    let mut result = j.clone();
                    for a in &args { result.push_str(&a.to_string()); }
                    Ok(JsValue::String(result))
                } else { Ok(JsValue::String(String::new())) }
            }
            "String.match" | "String.search" => Ok(JsValue::Null), 

            
            "Number.toFixed" => {
                if let Some(JsValue::Number(ae)) = receiver.as_ref() {
                    let eke = args.first().map(|v| v.to_number() as usize).unwrap_or(0).min(20);
                    
                    let ha = libm::pow(10.0, eke as f64);
                    let oic = round(*ae * ha) / ha;
                    Ok(JsValue::String(format!("{:.prec$}", oic, prec = eke)))
                } else { Ok(JsValue::String("NaN".to_string())) }
            }
            "Number.toString" => {
                if let Some(JsValue::Number(ae)) = receiver.as_ref() {
                    Ok(JsValue::String(format!("{}", ae)))
                } else { Ok(JsValue::String(String::new())) }
            }

            
            "Array.push" => {
                if let Some(JsValue::Array(ik)) = receiver.as_ref() {
                    let mut cmz = ik.clone();
                    for a in args { cmz.push(a); }
                    let len = cmz.len() as f64;
                    Ok(JsValue::Number(len))
                } else { Ok(JsValue::Number(0.0)) }
            }
            "Array.pop" => {
                if let Some(JsValue::Array(ik)) = receiver.as_ref() {
                    let mut cmz = ik.clone();
                    Ok(cmz.pop().unwrap_or(JsValue::Undefined))
                } else { Ok(JsValue::Undefined) }
            }
            "Array.shift" => {
                if let Some(JsValue::Array(ik)) = receiver.as_ref() {
                    if ik.is_empty() { return Ok(JsValue::Undefined); }
                    Ok(ik[0].clone())
                } else { Ok(JsValue::Undefined) }
            }
            "Array.unshift" => {
                if let Some(JsValue::Array(ik)) = receiver.as_ref() {
                    Ok(JsValue::Number((ik.len() + args.len()) as f64))
                } else { Ok(JsValue::Number(0.0)) }
            }
            "Array.join" => {
                if let Some(JsValue::Array(ik)) = receiver.as_ref() {
                    let fad = args.first().map(|v| v.to_string()).unwrap_or(",".to_string());
                    let au: Vec<String> = ik.iter().map(|v| v.to_string()).collect();
                    Ok(JsValue::String(au.join(&fad)))
                } else { Ok(JsValue::String(String::new())) }
            }
            "Array.reverse" => {
                if let Some(JsValue::Array(ik)) = receiver.as_ref() {
                    let mut cmz = ik.clone();
                    cmz.reverse();
                    Ok(JsValue::Array(cmz))
                } else { Ok(JsValue::Array(Vec::new())) }
            }
            "Array.indexOf" => {
                if let Some(JsValue::Array(ik)) = receiver.as_ref() {
                    let search = args.first().cloned().unwrap_or(JsValue::Undefined);
                    let gte = search.to_string();
                    for (i, item) in ik.iter().enumerate() {
                        if item.to_string() == gte { return Ok(JsValue::Number(i as f64)); }
                    }
                    Ok(JsValue::Number(-1.0))
                } else { Ok(JsValue::Number(-1.0)) }
            }
            "Array.includes" => {
                if let Some(JsValue::Array(ik)) = receiver.as_ref() {
                    let search = args.first().cloned().unwrap_or(JsValue::Undefined);
                    let gte = search.to_string();
                    Ok(JsValue::Boolean(ik.iter().any(|v| v.to_string() == gte)))
                } else { Ok(JsValue::Boolean(false)) }
            }
            "Array.slice" => {
                if let Some(JsValue::Array(ik)) = receiver.as_ref() {
                    let start = args.first().map(|v| v.to_number() as i64).unwrap_or(0);
                    let end = args.get(1).map(|v| v.to_number() as i64);
                    let len = ik.len() as i64;
                    let start = if start < 0 { (len + start).max(0) as usize } else { (start as usize).min(ik.len()) };
                    let end = match end {
                        Some(e) if e < 0 => (len + e).max(0) as usize,
                        Some(e) => (e as usize).min(ik.len()),
                        None => ik.len(),
                    };
                    Ok(JsValue::Array(ik[start..end].to_vec()))
                } else { Ok(JsValue::Array(Vec::new())) }
            }
            "Array.concat" => {
                if let Some(JsValue::Array(ik)) = receiver.as_ref() {
                    let mut result = ik.clone();
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
                
                if let Some(JsValue::Array(ik)) = receiver.as_ref() {
                    Ok(JsValue::Array(ik.clone()))
                } else { Ok(JsValue::Array(Vec::new())) }
            }

            _ => Err(format!("Unknown native function: {}", name)),
        }
    }

    
    fn parse_json(&self, j: &str) -> JsValue {
        let j = j.trim();
        if j.is_empty() { return JsValue::Null; }
        match j.as_bytes()[0] {
            b'"' => {
                
                if j.len() >= 2 && j.ends_with('"') {
                    JsValue::String(j[1..j.len()-1].to_string())
                } else { JsValue::Null }
            }
            b'{' => {
                
                let mut map = BTreeMap::new();
                let inner = &j[1..j.len().saturating_sub(1)];
                let mut depth = 0i32;
                let mut start = 0;
                let mut entries = Vec::new();
                for (i, c) in inner.chars().enumerate() {
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
                    if let Some(ald) = entry.find(':') {
                        let key = entry[..ald].trim().trim_matches('"');
                        let val = entry[ald+1..].trim();
                        map.insert(key.to_string(), self.parse_json(val));
                    }
                }
                JsValue::Object(map)
            }
            b'[' => {
                
                let inner = &j[1..j.len().saturating_sub(1)];
                let mut depth = 0i32;
                let mut start = 0;
                let mut items = Vec::new();
                for (i, c) in inner.chars().enumerate() {
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
                
                if let Ok(ae) = j.parse::<f64>() {
                    JsValue::Number(ae)
                } else { JsValue::Null }
            }
        }
    }

    
    fn stringify_json(&self, val: &JsValue) -> String {
        match val {
            JsValue::Undefined | JsValue::Null => "null".to_string(),
            JsValue::Boolean(b) => if *b { "true" } else { "false" }.to_string(),
            JsValue::Number(ae) => format!("{}", ae),
            JsValue::String(j) => format!("\"{}\"", j.replace('\\', "\\\\").replace('"', "\\\"")),
            JsValue::Array(ik) => {
                let items: Vec<String> = ik.iter().map(|v| self.stringify_json(v)).collect();
                format!("[{}]", items.join(","))
            }
            JsValue::Object(map) => {
                let entries: Vec<String> = map.iter()
                    .map(|(k, v)| format!("\"{}\":{}", k, self.stringify_json(v)))
                    .collect();
                format!("{{{}}}", entries.join(","))
            }
            JsValue::Aq(..) | JsValue::NativeFunction(_) => "null".to_string(),
        }
    }
}





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

fn crv(code: &str) -> Result<Vec<Token>, String> {
    let mut tokens = Vec::new();
    let chars: Vec<char> = code.chars().collect();
    let mut i = 0;
    
    while i < chars.len() {
        let c = chars[i];
        
        
        if c.is_whitespace() {
            i += 1;
            continue;
        }
        
        
        if c == '/' && i + 1 < chars.len() {
            if chars[i + 1] == '/' {
                
                while i < chars.len() && chars[i] != '\n' {
                    i += 1;
                }
                continue;
            }
            if chars[i + 1] == '*' {
                
                i += 2;
                while i + 1 < chars.len() && !(chars[i] == '*' && chars[i + 1] == '/') {
                    i += 1;
                }
                i += 2;
                continue;
            }
        }
        
        
        if c.is_ascii_digit() || (c == '.' && i + 1 < chars.len() && chars[i + 1].is_ascii_digit()) {
            let start = i;
            while i < chars.len() && (chars[i].is_ascii_digit() || chars[i] == '.') {
                i += 1;
            }
            let rw: String = chars[start..i].iter().collect();
            let num: f64 = rw.parse().unwrap_or(0.0);
            tokens.push(Token::Number(num));
            continue;
        }
        
        
        if c == '"' || c == '\'' {
            let arw = c;
            i += 1;
            let start = i;
            while i < chars.len() && chars[i] != arw {
                if chars[i] == '\\' && i + 1 < chars.len() {
                    i += 2;
                } else {
                    i += 1;
                }
            }
            let j: String = chars[start..i].iter().collect();
            tokens.push(Token::String(j));
            i += 1; 
            continue;
        }
        
        
        if c.is_alphabetic() || c == '_' || c == '$' {
            let start = i;
            while i < chars.len() && (chars[i].is_alphanumeric() || chars[i] == '_' || chars[i] == '$') {
                i += 1;
            }
            let fx: String = chars[start..i].iter().collect();
            let clr = ["var", "let", "const", "function", "if", "else", "for", "while", 
                          "return", "true", "false", "null", "undefined", "typeof", "new"];
            if clr.contains(&fx.as_str()) {
                tokens.push(Token::Keyword(fx));
            } else {
                tokens.push(Token::Identifier(fx));
            }
            continue;
        }
        
        
        let jov: String = chars[i..].iter().take(2).collect();
        let jmi: String = chars[i..].iter().take(3).collect();
        
        if ["===", "!=="].contains(&jmi.as_str()) {
            tokens.push(Token::Operator(jmi));
            i += 3;
            continue;
        }
        
        if ["==", "!=", "<=", ">=", "&&", "||", "++", "--", "+=", "-=", "*=", "/=", "=>"]
            .contains(&jov.as_str()) {
            tokens.push(Token::Operator(jov));
            i += 2;
            continue;
        }
        
        
        if "+-*/%<>=!&|?:".contains(c) {
            tokens.push(Token::Operator(c.to_string()));
            i += 1;
            continue;
        }
        
        
        if "{}[]();,.".contains(c) {
            tokens.push(Token::Punctuation(c));
            i += 1;
            continue;
        }
        
        
        i += 1;
    }
    
    tokens.push(Token::Eof);
    Ok(tokens)
}





#[derive(Debug, Clone)]
pub enum Statement {
    Var(String, Option<Expr>),
    Expr(Expr),
    If(Expr, Vec<Statement>, Option<Vec<Statement>>),
    While(Expr, Vec<Statement>),
    For(Option<Box<Statement>>, Option<Expr>, Option<Expr>, Vec<Statement>),
    Return(Option<Expr>),
    Aq(String, Vec<String>, String),
    Bl(Vec<Statement>),
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
        if let Token::Punctuation(aa) = self.current() {
            if *aa == c {
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
            Token::Keyword(li) if li == "var" || li == "let" || li == "const" => {
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
            Token::Keyword(li) if li == "if" => {
                self.advance();
                self.expect_punct('(')?;
                let fc = self.parse_expression()?;
                self.expect_punct(')')?;
                let avj = self.parse_block_or_statement()?;
                let atp = if let Token::Keyword(li) = self.current() {
                    if li == "else" {
                        self.advance();
                        Some(self.parse_block_or_statement()?)
                    } else {
                        None
                    }
                } else {
                    None
                };
                Ok(Statement::If(fc, avj, atp))
            }
            Token::Keyword(li) if li == "while" => {
                self.advance();
                self.expect_punct('(')?;
                let fc = self.parse_expression()?;
                self.expect_punct(')')?;
                let body = self.parse_block_or_statement()?;
                Ok(Statement::While(fc, body))
            }
            Token::Keyword(li) if li == "return" => {
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
            Token::Keyword(li) if li == "function" => {
                self.advance();
                if let Token::Identifier(name) = self.current().clone() {
                    self.advance();
                    self.expect_punct('(')?;
                    let params = self.parse_params()?;
                    self.expect_punct(')')?;
                    self.expect_punct('{')?;
                    
                    let body = self.parse_block_body()?;
                    Ok(Statement::Aq(name, params, body))
                } else {
                    Err("Expected function name".to_string())
                }
            }
            Token::Punctuation('{') => {
                self.advance();
                let stmts = self.parse_statements()?;
                self.expect_punct('}')?;
                Ok(Statement::Bl(stmts))
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
        self.advance(); 
        Ok(String::new()) 
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
        let fc = self.parse_or()?;
        if let Token::Operator(op) = self.current() {
            if op == "?" {
                self.advance();
                let gyn = self.parse_expression()?;
                if let Token::Operator(op) = self.current() {
                    if op == ":" {
                        self.advance();
                        let fuh = self.parse_expression()?;
                        return Ok(Expr::Ternary(Box::new(fc), Box::new(gyn), Box::new(fuh)));
                    }
                }
            }
        }
        Ok(fc)
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
                let dvw = self.parse_unary()?;
                return Ok(Expr::Unary(op, Box::new(dvw)));
            }
        }
        if let Token::Keyword(li) = self.current() {
            if li == "typeof" {
                self.advance();
                let dvw = self.parse_unary()?;
                return Ok(Expr::Unary("typeof".to_string(), Box::new(dvw)));
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
            Token::Number(ae) => {
                self.advance();
                Ok(Expr::Literal(JsValue::Number(ae)))
            }
            Token::String(j) => {
                self.advance();
                Ok(Expr::Literal(JsValue::String(j)))
            }
            Token::Keyword(li) if li == "true" => {
                self.advance();
                Ok(Expr::Literal(JsValue::Boolean(true)))
            }
            Token::Keyword(li) if li == "false" => {
                self.advance();
                Ok(Expr::Literal(JsValue::Boolean(false)))
            }
            Token::Keyword(li) if li == "null" => {
                self.advance();
                Ok(Expr::Literal(JsValue::Null))
            }
            Token::Keyword(li) if li == "undefined" => {
                self.advance();
                Ok(Expr::Literal(JsValue::Undefined))
            }
            Token::Identifier(name) => {
                self.advance();
                
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
                let dcz = self.parse_object_props()?;
                self.expect_punct('}')?;
                Ok(Expr::Object(dcz))
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
        let mut dcz = Vec::new();
        while !matches!(self.current(), Token::Punctuation('}')) {
            let key = match self.current().clone() {
                Token::Identifier(j) | Token::String(j) => {
                    self.advance();
                    j
                }
                _ => return Err("Expected property name".to_string()),
            };
            self.expect_punct(':')?;
            let value = self.parse_expression()?;
            dcz.push((key, value));
            if let Token::Punctuation(',') = self.current() {
                self.advance();
            } else {
                break;
            }
        }
        Ok(dcz)
    }
}
