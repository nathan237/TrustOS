//! TrustLang Virtual Machine — Stack-based bytecode interpreter
//!
//! Executes compiled TrustLang bytecode. Features:
//! - Stack-based evaluation (no register allocation needed)
//! - 256 local variables per call frame
//! - Call stack with return addresses
//! - Builtin functions: print, println, len, push, read_line, to_string

use alloc::string::String;
use alloc::vec::Vec;
use alloc::format;

/// Bytecode opcodes (u8)
#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(u8)]
pub enum Op {
    // Stack
    PushI64,    // push i64 constant (next 8 bytes)
    PushF64,    // push f64 constant (next 8 bytes)
    PushBool,   // push bool (next 1 byte)
    PushStr,    // push string constant (index in next 2 bytes)
    Pop,        // discard top
    Dup,        // duplicate top

    // Locals
    LoadLocal,  // load local variable (next 1 byte = slot)
    StoreLocal, // store to local variable (next 1 byte = slot)

    // Globals
    LoadGlobal,  // load global (next 2 bytes = index)
    StoreGlobal, // store global (next 2 bytes = index)

    // Arithmetic (i64)
    AddI, SubI, MulI, DivI, ModI, NegI,
    // Arithmetic (f64)
    AddF, SubF, MulF, DivF, NegF,
    // Comparison
    EqI, NeI, LtI, GtI, LeI, GeI,
    EqF, NeF, LtF, GtF, LeF, GeF,
    // Logical
    And, Or, Not,
    // Bitwise
    BitAnd, BitOr, BitXor, Shl, Shr,

    // Conversion
    I64toF64, F64toI64,

    // Control flow
    Jump,       // unconditional jump (next 2 bytes = offset)
    JumpIfFalse, // conditional jump
    Call,       // call function (next 2 bytes = func index, next 1 byte = arg count)
    CallBuiltin, // call builtin (next 1 byte = builtin id)
    Return,     // return from function

    // Array
    NewArray,   // create array (next 2 bytes = count)
    ArrayGet,   // array[index]
    ArraySet,   // array[index] = value
    ArrayLen,   // len(array)
    ArrayPush,  // push to array

    // String
    StrConcat,  // string concatenation

    // Special
    Halt,       // stop execution
}

/// A runtime value on the stack
#[derive(Debug, Clone)]
pub enum Value {
    I64(i64),
    F64(f64),
    Bool(bool),
    Str(String),
    Array(Vec<Value>),
    Void,
}

impl Value {
    fn as_i64(&self) -> Result<i64, String> {
        match self { Value::I64(v) => Ok(*v), _ => Err(format!("expected i64, got {:?}", self)) }
    }
    fn as_f64(&self) -> Result<f64, String> {
        match self { Value::F64(v) => Ok(*v), _ => Err(format!("expected f64, got {:?}", self)) }
    }
    fn as_bool(&self) -> Result<bool, String> {
        match self { Value::Bool(v) => Ok(*v), _ => Err(format!("expected bool, got {:?}", self)) }
    }
    fn to_display(&self) -> String {
        match self {
            Value::I64(v) => format!("{}", v),
            Value::F64(v) => format!("{:.6}", v),
            Value::Bool(v) => format!("{}", v),
            Value::Str(s) => s.clone(),
            Value::Array(a) => {
                let items: Vec<String> = a.iter().map(|v| v.to_display()).collect();
                format!("[{}]", items.join(", "))
            }
            Value::Void => String::from("()"),
        }
    }
}

/// Compiled bytecode for a function
#[derive(Debug, Clone)]
pub struct Function {
    pub name: String,
    pub arity: u8,         // number of parameters
    pub locals: u8,        // total local slots needed
    pub code: Vec<u8>,     // bytecode
}

/// A compiled program
#[derive(Debug, Clone)]
pub struct Bytecode {
    pub functions: Vec<Function>,
    pub strings: Vec<String>,     // string constant pool
    pub entry: usize,             // index of main()
}

/// Call frame
struct CallFrame {
    func_idx: usize,
    ip: usize,
    base: usize, // stack base for locals
    locals: [Value; 256],
}

impl CallFrame {
    fn new(func_idx: usize, base: usize) -> Self {
        const VOID: Value = Value::Void;
        Self {
            func_idx,
            ip: 0,
            base,
            locals: [VOID; 256],
        }
    }
}

/// Builtin function IDs
const BUILTIN_PRINT: u8 = 0;
const BUILTIN_PRINTLN: u8 = 1;
const BUILTIN_LEN: u8 = 2;
const BUILTIN_PUSH: u8 = 3;
const BUILTIN_TO_STRING: u8 = 4;
const BUILTIN_TO_INT: u8 = 5;
const BUILTIN_SQRT: u8 = 6;
const BUILTIN_ABS: u8 = 7;
const BUILTIN_PIXEL: u8 = 8;
const BUILTIN_CLEAR_SCREEN: u8 = 9;
const BUILTIN_FILL_RECT: u8 = 10;
const BUILTIN_DRAW_LINE: u8 = 11;
const BUILTIN_DRAW_CIRCLE: u8 = 12;
const BUILTIN_SCREEN_W: u8 = 13;
const BUILTIN_SCREEN_H: u8 = 14;
const BUILTIN_FLUSH: u8 = 15;
const BUILTIN_DRAW_TEXT: u8 = 16;
const BUILTIN_SLEEP: u8 = 17;

/// Resolve builtin name → ID
pub fn builtin_id(name: &str) -> Option<u8> {
    match name {
        "print" => Some(BUILTIN_PRINT),
        "println" => Some(BUILTIN_PRINTLN),
        "len" => Some(BUILTIN_LEN),
        "push" => Some(BUILTIN_PUSH),
        "to_string" => Some(BUILTIN_TO_STRING),
        "to_int" => Some(BUILTIN_TO_INT),
        "sqrt" => Some(BUILTIN_SQRT),
        "abs" => Some(BUILTIN_ABS),
        "pixel" => Some(BUILTIN_PIXEL),
        "clear_screen" => Some(BUILTIN_CLEAR_SCREEN),
        "fill_rect" => Some(BUILTIN_FILL_RECT),
        "draw_line" => Some(BUILTIN_DRAW_LINE),
        "draw_circle" => Some(BUILTIN_DRAW_CIRCLE),
        "screen_w" => Some(BUILTIN_SCREEN_W),
        "screen_h" => Some(BUILTIN_SCREEN_H),
        "flush" => Some(BUILTIN_FLUSH),
        "draw_text" => Some(BUILTIN_DRAW_TEXT),
        "sleep" => Some(BUILTIN_SLEEP),
        _ => None,
    }
}

/// Execute bytecode and return output
pub fn execute(bytecode: &Bytecode) -> Result<String, String> {
    let mut output = String::new();
    let mut stack: Vec<Value> = Vec::with_capacity(1024);
    let mut frames: Vec<CallFrame> = Vec::with_capacity(64);

    frames.push(CallFrame::new(bytecode.entry, 0));

    let max_steps = 500_000_000; // generous limit for graphics programs
    let mut steps = 0;

    loop {
        steps += 1;
        if steps > max_steps {
            return Err(String::from("execution limit exceeded (10M steps)"));
        }

        let frame = frames.last_mut().ok_or("no call frame")?;
        let func = &bytecode.functions[frame.func_idx];

        if frame.ip >= func.code.len() {
            // Implicit return
            if frames.len() <= 1 { break; }
            frames.pop();
            stack.push(Value::Void);
            continue;
        }

        let op = func.code[frame.ip];
        frame.ip += 1;

        match op {
            x if x == Op::PushI64 as u8 => {
                let bytes = read_bytes(&func.code, &mut frame.ip, 8);
                let v = i64::from_le_bytes(bytes.try_into().unwrap());
                stack.push(Value::I64(v));
            }
            x if x == Op::PushF64 as u8 => {
                let bytes = read_bytes(&func.code, &mut frame.ip, 8);
                let v = f64::from_le_bytes(bytes.try_into().unwrap());
                stack.push(Value::F64(v));
            }
            x if x == Op::PushBool as u8 => {
                let v = func.code[frame.ip] != 0;
                frame.ip += 1;
                stack.push(Value::Bool(v));
            }
            x if x == Op::PushStr as u8 => {
                let idx = read_u16(&func.code, &mut frame.ip) as usize;
                let s = bytecode.strings.get(idx).cloned().unwrap_or_default();
                stack.push(Value::Str(s));
            }
            x if x == Op::Pop as u8 => { stack.pop(); }
            x if x == Op::Dup as u8 => {
                let v = stack.last().cloned().unwrap_or(Value::Void);
                stack.push(v);
            }
            x if x == Op::LoadLocal as u8 => {
                let slot = func.code[frame.ip] as usize;
                frame.ip += 1;
                stack.push(frame.locals[slot].clone());
            }
            x if x == Op::StoreLocal as u8 => {
                let slot = func.code[frame.ip] as usize;
                frame.ip += 1;
                let val = stack.pop().unwrap_or(Value::Void);
                frame.locals[slot] = val;
            }
            // Integer arithmetic (auto-promotes to f64 if operands are float)
            x if x == Op::AddI as u8 => { bin_op_i64(&mut stack, |a, b| a.wrapping_add(b), |a, b| a + b)?; }
            x if x == Op::SubI as u8 => { bin_op_i64(&mut stack, |a, b| a.wrapping_sub(b), |a, b| a - b)?; }
            x if x == Op::MulI as u8 => { bin_op_i64(&mut stack, |a, b| a.wrapping_mul(b), |a, b| a * b)?; }
            x if x == Op::DivI as u8 => {
                let b_val = stack.pop().unwrap_or(Value::I64(0));
                let a_val = stack.pop().unwrap_or(Value::I64(0));
                match (&a_val, &b_val) {
                    (Value::F64(a), Value::F64(b)) => stack.push(Value::F64(a / b)),
                    (Value::I64(a), Value::F64(b)) => stack.push(Value::F64(*a as f64 / b)),
                    (Value::F64(a), Value::I64(b)) => stack.push(Value::F64(a / *b as f64)),
                    _ => {
                        let b = b_val.as_i64()?;
                        let a = a_val.as_i64()?;
                        if b == 0 { return Err(String::from("division by zero")); }
                        stack.push(Value::I64(a / b));
                    }
                }
            }
            x if x == Op::ModI as u8 => {
                let b_val = stack.pop().unwrap_or(Value::I64(0));
                let a_val = stack.pop().unwrap_or(Value::I64(0));
                match (&a_val, &b_val) {
                    (Value::F64(a), Value::F64(b)) => stack.push(Value::F64(a % b)),
                    (Value::I64(a), Value::F64(b)) => stack.push(Value::F64(*a as f64 % b)),
                    (Value::F64(a), Value::I64(b)) => stack.push(Value::F64(a % *b as f64)),
                    _ => {
                        let b = b_val.as_i64()?;
                        let a = a_val.as_i64()?;
                        if b == 0 { return Err(String::from("modulo by zero")); }
                        stack.push(Value::I64(a % b));
                    }
                }
            }
            x if x == Op::NegI as u8 => {
                let v = stack.pop().unwrap_or(Value::I64(0));
                match v {
                    Value::F64(f) => stack.push(Value::F64(-f)),
                    _ => stack.push(Value::I64(-v.as_i64()?)),
                }
            }
            // Float arithmetic
            x if x == Op::AddF as u8 => { bin_op_f64(&mut stack, |a, b| a + b)?; }
            x if x == Op::SubF as u8 => { bin_op_f64(&mut stack, |a, b| a - b)?; }
            x if x == Op::MulF as u8 => { bin_op_f64(&mut stack, |a, b| a * b)?; }
            x if x == Op::DivF as u8 => { bin_op_f64(&mut stack, |a, b| a / b)?; }
            x if x == Op::NegF as u8 => {
                let v = stack.pop().unwrap_or(Value::F64(0.0)).as_f64()?;
                stack.push(Value::F64(-v));
            }
            // Integer comparisons (auto-promotes to f64 if operands are float)
            x if x == Op::EqI as u8 => { cmp_i64(&mut stack, |a, b| a == b, |a, b| a == b)?; }
            x if x == Op::NeI as u8 => { cmp_i64(&mut stack, |a, b| a != b, |a, b| a != b)?; }
            x if x == Op::LtI as u8 => { cmp_i64(&mut stack, |a, b| a < b, |a, b| a < b)?; }
            x if x == Op::GtI as u8 => { cmp_i64(&mut stack, |a, b| a > b, |a, b| a > b)?; }
            x if x == Op::LeI as u8 => { cmp_i64(&mut stack, |a, b| a <= b, |a, b| a <= b)?; }
            x if x == Op::GeI as u8 => { cmp_i64(&mut stack, |a, b| a >= b, |a, b| a >= b)?; }
            // Float comparisons
            x if x == Op::EqF as u8 => { cmp_f64(&mut stack, |a, b| a == b)?; }
            x if x == Op::NeF as u8 => { cmp_f64(&mut stack, |a, b| a != b)?; }
            x if x == Op::LtF as u8 => { cmp_f64(&mut stack, |a, b| a < b)?; }
            x if x == Op::GtF as u8 => { cmp_f64(&mut stack, |a, b| a > b)?; }
            x if x == Op::LeF as u8 => { cmp_f64(&mut stack, |a, b| a <= b)?; }
            x if x == Op::GeF as u8 => { cmp_f64(&mut stack, |a, b| a >= b)?; }
            // Logical
            x if x == Op::And as u8 => {
                let b = stack.pop().unwrap_or(Value::Bool(false)).as_bool()?;
                let a = stack.pop().unwrap_or(Value::Bool(false)).as_bool()?;
                stack.push(Value::Bool(a && b));
            }
            x if x == Op::Or as u8 => {
                let b = stack.pop().unwrap_or(Value::Bool(false)).as_bool()?;
                let a = stack.pop().unwrap_or(Value::Bool(false)).as_bool()?;
                stack.push(Value::Bool(a || b));
            }
            x if x == Op::Not as u8 => {
                let v = stack.pop().unwrap_or(Value::Bool(false)).as_bool()?;
                stack.push(Value::Bool(!v));
            }
            // Bitwise (convert floats to i64 if needed)
            x if x == Op::BitAnd as u8 => { bin_op_i64(&mut stack, |a, b| a & b, |a, b| (a as i64 & b as i64) as f64)?; }
            x if x == Op::BitOr as u8 => { bin_op_i64(&mut stack, |a, b| a | b, |a, b| (a as i64 | b as i64) as f64)?; }
            x if x == Op::BitXor as u8 => { bin_op_i64(&mut stack, |a, b| a ^ b, |a, b| (a as i64 ^ b as i64) as f64)?; }
            x if x == Op::Shl as u8 => { bin_op_i64(&mut stack, |a, b| a << (b & 63), |a, b| ((a as i64) << (b as i64 & 63)) as f64)?; }
            x if x == Op::Shr as u8 => { bin_op_i64(&mut stack, |a, b| a >> (b & 63), |a, b| ((a as i64) >> (b as i64 & 63)) as f64)?; }
            // Conversion
            x if x == Op::I64toF64 as u8 => {
                let v = stack.pop().unwrap_or(Value::I64(0)).as_i64()?;
                stack.push(Value::F64(v as f64));
            }
            x if x == Op::F64toI64 as u8 => {
                let v = stack.pop().unwrap_or(Value::F64(0.0)).as_f64()?;
                stack.push(Value::I64(v as i64));
            }
            // Control flow
            x if x == Op::Jump as u8 => {
                let off = read_u16(&func.code, &mut frame.ip) as usize;
                frame.ip = off;
            }
            x if x == Op::JumpIfFalse as u8 => {
                let off = read_u16(&func.code, &mut frame.ip) as usize;
                let cond = stack.pop().unwrap_or(Value::Bool(false)).as_bool()?;
                if !cond { frame.ip = off; }
            }
            x if x == Op::Call as u8 => {
                let func_idx = read_u16(&func.code, &mut frame.ip) as usize;
                let argc = func.code[frame.ip] as usize;
                frame.ip += 1;
                // Pop args
                let mut args = Vec::with_capacity(argc);
                for _ in 0..argc {
                    args.push(stack.pop().unwrap_or(Value::Void));
                }
                args.reverse();
                // Create new frame
                let mut new_frame = CallFrame::new(func_idx, stack.len());
                for (i, arg) in args.into_iter().enumerate() {
                    new_frame.locals[i] = arg;
                }
                frames.push(new_frame);
            }
            x if x == Op::CallBuiltin as u8 => {
                let builtin = func.code[frame.ip];
                frame.ip += 1;
                let argc = func.code[frame.ip] as usize;
                frame.ip += 1;
                let mut args = Vec::with_capacity(argc);
                for _ in 0..argc {
                    args.push(stack.pop().unwrap_or(Value::Void));
                }
                args.reverse();
                let result = exec_builtin(builtin, &args, &mut output)?;
                stack.push(result);
            }
            x if x == Op::Return as u8 => {
                let ret = stack.pop().unwrap_or(Value::Void);
                if frames.len() <= 1 {
                    stack.push(ret);
                    break;
                }
                frames.pop();
                stack.push(ret);
            }
            // Array
            x if x == Op::NewArray as u8 => {
                let count = read_u16(&func.code, &mut frame.ip) as usize;
                let mut arr = Vec::with_capacity(count);
                for _ in 0..count {
                    arr.push(stack.pop().unwrap_or(Value::Void));
                }
                arr.reverse();
                stack.push(Value::Array(arr));
            }
            x if x == Op::ArrayGet as u8 => {
                let idx = stack.pop().unwrap_or(Value::I64(0)).as_i64()? as usize;
                let arr = stack.pop().unwrap_or(Value::Array(Vec::new()));
                match arr {
                    Value::Array(a) => {
                        let v = a.get(idx).cloned().unwrap_or(Value::Void);
                        stack.push(v);
                    }
                    Value::Str(s) => {
                        let c = s.as_bytes().get(idx).copied().unwrap_or(0);
                        stack.push(Value::I64(c as i64));
                    }
                    _ => return Err(String::from("index on non-array")),
                }
            }
            x if x == Op::ArraySet as u8 => {
                let val = stack.pop().unwrap_or(Value::Void);
                let idx = stack.pop().unwrap_or(Value::I64(0)).as_i64()? as usize;
                let arr = stack.pop().unwrap_or(Value::Array(Vec::new()));
                match arr {
                    Value::Array(mut a) => {
                        if idx < a.len() { a[idx] = val; }
                        stack.push(Value::Array(a));
                    }
                    _ => return Err(String::from("index-set on non-array")),
                }
            }
            x if x == Op::ArrayLen as u8 => {
                let v = stack.pop().unwrap_or(Value::Void);
                let len = match &v {
                    Value::Array(a) => a.len() as i64,
                    Value::Str(s) => s.len() as i64,
                    _ => 0,
                };
                stack.push(Value::I64(len));
            }
            x if x == Op::StrConcat as u8 => {
                let b = stack.pop().unwrap_or(Value::Str(String::new())).to_display();
                let a = stack.pop().unwrap_or(Value::Str(String::new())).to_display();
                stack.push(Value::Str(format!("{}{}", a, b)));
            }
            x if x == Op::Halt as u8 => break,
            _ => return Err(format!("unknown opcode: {}", op)),
        }
    }

    Ok(output)
}

// ─── Helpers ────────────────────────────────────────────────────────────

fn read_bytes(code: &[u8], ip: &mut usize, n: usize) -> Vec<u8> {
    let bytes = code[*ip..*ip + n].to_vec();
    *ip += n;
    bytes
}

fn read_u16(code: &[u8], ip: &mut usize) -> u16 {
    let v = u16::from_le_bytes([code[*ip], code[*ip + 1]]);
    *ip += 2;
    v
}

fn bin_op_i64(stack: &mut Vec<Value>, fi: fn(i64, i64) -> i64, ff: fn(f64, f64) -> f64) -> Result<(), String> {
    let b_val = stack.pop().unwrap_or(Value::I64(0));
    let a_val = stack.pop().unwrap_or(Value::I64(0));
    // Auto-promote to f64 if either operand is float
    match (&a_val, &b_val) {
        (Value::F64(a), Value::F64(b)) => stack.push(Value::F64(ff(*a, *b))),
        (Value::I64(a), Value::F64(b)) => stack.push(Value::F64(ff(*a as f64, *b))),
        (Value::F64(a), Value::I64(b)) => stack.push(Value::F64(ff(*a, *b as f64))),
        _ => {
            let a = a_val.as_i64()?;
            let b = b_val.as_i64()?;
            stack.push(Value::I64(fi(a, b)));
        }
    }
    Ok(())
}

fn bin_op_f64(stack: &mut Vec<Value>, f: fn(f64, f64) -> f64) -> Result<(), String> {
    let b = stack.pop().unwrap_or(Value::F64(0.0)).as_f64()?;
    let a = stack.pop().unwrap_or(Value::F64(0.0)).as_f64()?;
    stack.push(Value::F64(f(a, b)));
    Ok(())
}

fn cmp_i64(stack: &mut Vec<Value>, fi: fn(i64, i64) -> bool, ff: fn(f64, f64) -> bool) -> Result<(), String> {
    let b_val = stack.pop().unwrap_or(Value::I64(0));
    let a_val = stack.pop().unwrap_or(Value::I64(0));
    // Auto-promote to f64 comparison if either operand is float
    match (&a_val, &b_val) {
        (Value::F64(a), Value::F64(b)) => stack.push(Value::Bool(ff(*a, *b))),
        (Value::I64(a), Value::F64(b)) => stack.push(Value::Bool(ff(*a as f64, *b))),
        (Value::F64(a), Value::I64(b)) => stack.push(Value::Bool(ff(*a, *b as f64))),
        _ => {
            let a = a_val.as_i64()?;
            let b = b_val.as_i64()?;
            stack.push(Value::Bool(fi(a, b)));
        }
    }
    Ok(())
}

fn cmp_f64(stack: &mut Vec<Value>, f: fn(f64, f64) -> bool) -> Result<(), String> {
    let b = stack.pop().unwrap_or(Value::F64(0.0)).as_f64()?;
    let a = stack.pop().unwrap_or(Value::F64(0.0)).as_f64()?;
    stack.push(Value::Bool(f(a, b)));
    Ok(())
}

/// Execute a builtin function
fn exec_builtin(id: u8, args: &[Value], output: &mut String) -> Result<Value, String> {
    match id {
        BUILTIN_PRINT => {
            for arg in args { output.push_str(&arg.to_display()); }
            Ok(Value::Void)
        }
        BUILTIN_PRINTLN => {
            for arg in args { output.push_str(&arg.to_display()); }
            output.push('\n');
            Ok(Value::Void)
        }
        BUILTIN_LEN => {
            let v = args.first().unwrap_or(&Value::Void);
            match v {
                Value::Array(a) => Ok(Value::I64(a.len() as i64)),
                Value::Str(s) => Ok(Value::I64(s.len() as i64)),
                _ => Ok(Value::I64(0)),
            }
        }
        BUILTIN_PUSH => {
            if args.len() >= 2 {
                if let Value::Array(mut a) = args[0].clone() {
                    a.push(args[1].clone());
                    return Ok(Value::Array(a));
                }
            }
            Err(String::from("push expects (array, value)"))
        }
        BUILTIN_TO_STRING => {
            let v = args.first().unwrap_or(&Value::Void);
            Ok(Value::Str(v.to_display()))
        }
        BUILTIN_TO_INT => {
            let v = args.first().unwrap_or(&Value::Void);
            match v {
                Value::I64(n) => Ok(Value::I64(*n)),
                Value::F64(f) => Ok(Value::I64(*f as i64)),
                Value::Bool(b) => Ok(Value::I64(if *b { 1 } else { 0 })),
                Value::Str(s) => {
                    let n: i64 = parse_i64_simple(s.trim());
                    Ok(Value::I64(n))
                }
                _ => Ok(Value::I64(0)),
            }
        }
        BUILTIN_SQRT => {
            let v = args.first().unwrap_or(&Value::F64(0.0)).as_f64().unwrap_or(0.0);
            Ok(Value::F64(libm::sqrt(v)))
        }
        BUILTIN_ABS => {
            match args.first().unwrap_or(&Value::I64(0)) {
                Value::I64(n) => Ok(Value::I64(n.abs())),
                Value::F64(f) => Ok(Value::F64(libm::fabs(*f))),
                _ => Ok(Value::I64(0)),
            }
        }
        // ══════════════════════════════════════════════════
        // Graphics builtins — direct framebuffer access
        // ══════════════════════════════════════════════════
        BUILTIN_PIXEL => {
            // pixel(x, y, r, g, b)
            let x = args.get(0).and_then(|v| v.as_i64().ok()).unwrap_or(0) as u32;
            let y = args.get(1).and_then(|v| v.as_i64().ok()).unwrap_or(0) as u32;
            let r = args.get(2).and_then(|v| v.as_i64().ok()).unwrap_or(255) as u32 & 0xFF;
            let g = args.get(3).and_then(|v| v.as_i64().ok()).unwrap_or(255) as u32 & 0xFF;
            let b = args.get(4).and_then(|v| v.as_i64().ok()).unwrap_or(255) as u32 & 0xFF;
            let color = 0xFF000000 | (r << 16) | (g << 8) | b;
            crate::framebuffer::put_pixel(x, y, color);
            Ok(Value::Void)
        }
        BUILTIN_CLEAR_SCREEN => {
            // clear_screen(r, g, b)
            let r = args.get(0).and_then(|v| v.as_i64().ok()).unwrap_or(0) as u32 & 0xFF;
            let g = args.get(1).and_then(|v| v.as_i64().ok()).unwrap_or(0) as u32 & 0xFF;
            let b = args.get(2).and_then(|v| v.as_i64().ok()).unwrap_or(0) as u32 & 0xFF;
            let color = 0xFF000000 | (r << 16) | (g << 8) | b;
            let (sw, sh) = crate::framebuffer::get_dimensions();
            crate::framebuffer::fill_rect(0, 0, sw, sh, color);
            Ok(Value::Void)
        }
        BUILTIN_FILL_RECT => {
            // fill_rect(x, y, w, h, r, g, b)
            let x = args.get(0).and_then(|v| v.as_i64().ok()).unwrap_or(0) as u32;
            let y = args.get(1).and_then(|v| v.as_i64().ok()).unwrap_or(0) as u32;
            let w = args.get(2).and_then(|v| v.as_i64().ok()).unwrap_or(0) as u32;
            let h = args.get(3).and_then(|v| v.as_i64().ok()).unwrap_or(0) as u32;
            let r = args.get(4).and_then(|v| v.as_i64().ok()).unwrap_or(255) as u32 & 0xFF;
            let g = args.get(5).and_then(|v| v.as_i64().ok()).unwrap_or(255) as u32 & 0xFF;
            let b = args.get(6).and_then(|v| v.as_i64().ok()).unwrap_or(255) as u32 & 0xFF;
            let color = 0xFF000000 | (r << 16) | (g << 8) | b;
            crate::framebuffer::fill_rect(x, y, w, h, color);
            Ok(Value::Void)
        }
        BUILTIN_DRAW_LINE => {
            // draw_line(x1, y1, x2, y2, r, g, b)
            let x0 = args.get(0).and_then(|v| v.as_i64().ok()).unwrap_or(0);
            let y0 = args.get(1).and_then(|v| v.as_i64().ok()).unwrap_or(0);
            let x1 = args.get(2).and_then(|v| v.as_i64().ok()).unwrap_or(0);
            let y1 = args.get(3).and_then(|v| v.as_i64().ok()).unwrap_or(0);
            let r = args.get(4).and_then(|v| v.as_i64().ok()).unwrap_or(255) as u32 & 0xFF;
            let g = args.get(5).and_then(|v| v.as_i64().ok()).unwrap_or(255) as u32 & 0xFF;
            let b = args.get(6).and_then(|v| v.as_i64().ok()).unwrap_or(255) as u32 & 0xFF;
            let color = 0xFF000000 | (r << 16) | (g << 8) | b;
            // Bresenham line
            let mut cx = x0;
            let mut cy = y0;
            let dx = (x1 - x0).abs();
            let dy = -(y1 - y0).abs();
            let sx: i64 = if x0 < x1 { 1 } else { -1 };
            let sy: i64 = if y0 < y1 { 1 } else { -1 };
            let mut err = dx + dy;
            loop {
                if cx >= 0 && cy >= 0 {
                    crate::framebuffer::put_pixel(cx as u32, cy as u32, color);
                }
                if cx == x1 && cy == y1 { break; }
                let e2 = 2 * err;
                if e2 >= dy { err += dy; cx += sx; }
                if e2 <= dx { err += dx; cy += sy; }
            }
            Ok(Value::Void)
        }
        BUILTIN_DRAW_CIRCLE => {
            // draw_circle(cx, cy, radius, r, g, b)
            let cx = args.get(0).and_then(|v| v.as_i64().ok()).unwrap_or(0);
            let cy = args.get(1).and_then(|v| v.as_i64().ok()).unwrap_or(0);
            let radius = args.get(2).and_then(|v| v.as_i64().ok()).unwrap_or(0);
            let r = args.get(3).and_then(|v| v.as_i64().ok()).unwrap_or(255) as u32 & 0xFF;
            let g = args.get(4).and_then(|v| v.as_i64().ok()).unwrap_or(255) as u32 & 0xFF;
            let b = args.get(5).and_then(|v| v.as_i64().ok()).unwrap_or(255) as u32 & 0xFF;
            let color = 0xFF000000 | (r << 16) | (g << 8) | b;
            // Midpoint circle
            let mut x = radius;
            let mut y: i64 = 0;
            let mut d = 1 - radius;
            while x >= y {
                let pts = [
                    (cx + x, cy + y), (cx - x, cy + y),
                    (cx + x, cy - y), (cx - x, cy - y),
                    (cx + y, cy + x), (cx - y, cy + x),
                    (cx + y, cy - x), (cx - y, cy - x),
                ];
                for (px, py) in pts {
                    if px >= 0 && py >= 0 {
                        crate::framebuffer::put_pixel(px as u32, py as u32, color);
                    }
                }
                y += 1;
                if d <= 0 {
                    d += 2 * y + 1;
                } else {
                    x -= 1;
                    d += 2 * (y - x) + 1;
                }
            }
            Ok(Value::Void)
        }
        BUILTIN_SCREEN_W => {
            let (w, _) = crate::framebuffer::get_dimensions();
            Ok(Value::I64(w as i64))
        }
        BUILTIN_SCREEN_H => {
            let (_, h) = crate::framebuffer::get_dimensions();
            Ok(Value::I64(h as i64))
        }
        BUILTIN_FLUSH => {
            // flush() — swap buffers if double-buffered
            crate::framebuffer::swap_buffers();
            Ok(Value::Void)
        }
        BUILTIN_DRAW_TEXT => {
            // draw_text(text, x, y, r, g, b, scale)
            if let Some(Value::Str(text)) = args.get(0) {
                let x = args.get(1).and_then(|v| v.as_i64().ok()).unwrap_or(0) as u32;
                let y = args.get(2).and_then(|v| v.as_i64().ok()).unwrap_or(0) as u32;
                let r = args.get(3).and_then(|v| v.as_i64().ok()).unwrap_or(255) as u32 & 0xFF;
                let g = args.get(4).and_then(|v| v.as_i64().ok()).unwrap_or(255) as u32 & 0xFF;
                let b = args.get(5).and_then(|v| v.as_i64().ok()).unwrap_or(255) as u32 & 0xFF;
                let scale = args.get(6).and_then(|v| v.as_i64().ok()).unwrap_or(1) as u32;
                let color = 0xFF000000 | (r << 16) | (g << 8) | b;
                // Draw text with scaling
                let mut cx = x;
                for c in text.chars() {
                    let glyph = crate::framebuffer::font::get_glyph(c);
                    for (row, &bits) in glyph.iter().enumerate() {
                        for bit in 0..8u32 {
                            if bits & (0x80 >> bit) != 0 {
                                for sy in 0..scale {
                                    for sx in 0..scale {
                                        crate::framebuffer::put_pixel(
                                            cx + bit * scale + sx,
                                            y + row as u32 * scale + sy,
                                            color,
                                        );
                                    }
                                }
                            }
                        }
                    }
                    cx += 8 * scale;
                }
            }
            Ok(Value::Void)
        }
        BUILTIN_SLEEP => {
            // sleep(ms) — use PIT for reliable wall-clock timing
            let ms = args.get(0).and_then(|v| v.as_i64().ok()).unwrap_or(0) as u64;
            crate::cpu::tsc::pit_delay_ms(ms);
            Ok(Value::Void)
        }
        _ => Err(format!("unknown builtin id: {}", id)),
    }
}

/// Parse i64 from string without stdlib
fn parse_i64_simple(s: &str) -> i64 {
    let mut val: i64 = 0;
    let mut neg = false;
    for (i, ch) in s.chars().enumerate() {
        if i == 0 && ch == '-' { neg = true; continue; }
        if !ch.is_ascii_digit() { break; }
        val = val.wrapping_mul(10).wrapping_add((ch as i64) - 48);
    }
    if neg { -val } else { val }
}
