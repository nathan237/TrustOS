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
        _ => None,
    }
}

/// Execute bytecode and return output
pub fn execute(bytecode: &Bytecode) -> Result<String, String> {
    let mut output = String::new();
    let mut stack: Vec<Value> = Vec::with_capacity(1024);
    let mut frames: Vec<CallFrame> = Vec::with_capacity(64);

    frames.push(CallFrame::new(bytecode.entry, 0));

    let max_steps = 10_000_000; // prevent infinite loops
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
            // Integer arithmetic
            x if x == Op::AddI as u8 => { bin_op_i64(&mut stack, |a, b| a.wrapping_add(b))?; }
            x if x == Op::SubI as u8 => { bin_op_i64(&mut stack, |a, b| a.wrapping_sub(b))?; }
            x if x == Op::MulI as u8 => { bin_op_i64(&mut stack, |a, b| a.wrapping_mul(b))?; }
            x if x == Op::DivI as u8 => {
                let b = stack.pop().unwrap_or(Value::I64(0)).as_i64()?;
                let a = stack.pop().unwrap_or(Value::I64(0)).as_i64()?;
                if b == 0 { return Err(String::from("division by zero")); }
                stack.push(Value::I64(a / b));
            }
            x if x == Op::ModI as u8 => {
                let b = stack.pop().unwrap_or(Value::I64(0)).as_i64()?;
                let a = stack.pop().unwrap_or(Value::I64(0)).as_i64()?;
                if b == 0 { return Err(String::from("modulo by zero")); }
                stack.push(Value::I64(a % b));
            }
            x if x == Op::NegI as u8 => {
                let v = stack.pop().unwrap_or(Value::I64(0)).as_i64()?;
                stack.push(Value::I64(-v));
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
            // Integer comparisons
            x if x == Op::EqI as u8 => { cmp_i64(&mut stack, |a, b| a == b)?; }
            x if x == Op::NeI as u8 => { cmp_i64(&mut stack, |a, b| a != b)?; }
            x if x == Op::LtI as u8 => { cmp_i64(&mut stack, |a, b| a < b)?; }
            x if x == Op::GtI as u8 => { cmp_i64(&mut stack, |a, b| a > b)?; }
            x if x == Op::LeI as u8 => { cmp_i64(&mut stack, |a, b| a <= b)?; }
            x if x == Op::GeI as u8 => { cmp_i64(&mut stack, |a, b| a >= b)?; }
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
            // Bitwise
            x if x == Op::BitAnd as u8 => { bin_op_i64(&mut stack, |a, b| a & b)?; }
            x if x == Op::BitOr as u8 => { bin_op_i64(&mut stack, |a, b| a | b)?; }
            x if x == Op::BitXor as u8 => { bin_op_i64(&mut stack, |a, b| a ^ b)?; }
            x if x == Op::Shl as u8 => { bin_op_i64(&mut stack, |a, b| a << (b & 63))?; }
            x if x == Op::Shr as u8 => { bin_op_i64(&mut stack, |a, b| a >> (b & 63))?; }
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

fn bin_op_i64(stack: &mut Vec<Value>, f: fn(i64, i64) -> i64) -> Result<(), String> {
    let b = stack.pop().unwrap_or(Value::I64(0)).as_i64()?;
    let a = stack.pop().unwrap_or(Value::I64(0)).as_i64()?;
    stack.push(Value::I64(f(a, b)));
    Ok(())
}

fn bin_op_f64(stack: &mut Vec<Value>, f: fn(f64, f64) -> f64) -> Result<(), String> {
    let b = stack.pop().unwrap_or(Value::F64(0.0)).as_f64()?;
    let a = stack.pop().unwrap_or(Value::F64(0.0)).as_f64()?;
    stack.push(Value::F64(f(a, b)));
    Ok(())
}

fn cmp_i64(stack: &mut Vec<Value>, f: fn(i64, i64) -> bool) -> Result<(), String> {
    let b = stack.pop().unwrap_or(Value::I64(0)).as_i64()?;
    let a = stack.pop().unwrap_or(Value::I64(0)).as_i64()?;
    stack.push(Value::Bool(f(a, b)));
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
