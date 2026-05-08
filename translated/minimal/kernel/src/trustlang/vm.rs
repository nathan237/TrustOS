







use alloc::string::String;
use alloc::vec::Vec;
use alloc::format;


#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(u8)]
pub enum Op {
    
    PushI64,    
    PushF64,    
    PushBool,   
    PushStr,    
    Pop,        
    Dup,        

    
    LoadLocal,  
    StoreLocal, 

    
    LoadGlobal,  
    StoreGlobal, 

    
    AddI, SubI, MulI, DivI, ModI, NegI,
    
    AddF, SubF, MulF, DivF, NegF,
    
    EqI, NeI, LtI, GtI, LeI, GeI,
    EqF, NeF, LtF, GtF, LeF, GeF,
    
    And, Or, Not,
    
    BitAnd, BitOr, BitXor, Shl, Shr,

    
    I64toF64, F64toI64,

    
    Jump,       
    JumpIfFalse, 
    Call,       
    CallBuiltin, 
    Return,     

    
    NewArray,   
    ArrayGet,   
    ArraySet,   
    ArrayLen,   
    ArrayPush,  

    
    StrConcat,  

    
    Halt,       
}

impl Op {
    
    
    #[inline(always)]
    fn atw(v: u8) -> Option<Op> {
        if v <= Op::Halt as u8 {
            
            Some(unsafe { core::mem::transmute(v) })
        } else {
            None
        }
    }
}


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
            Value::Str(j) => j.clone(),
            Value::Array(a) => {
                let items: Vec<String> = a.iter().map(|v| v.to_display()).collect();
                format!("[{}]", items.join(", "))
            }
            Value::Void => String::from("()"),
        }
    }
}


#[derive(Debug, Clone)]
pub struct Aq {
    pub name: String,
    pub arity: u8,         
    pub locals: u8,        
    pub code: Vec<u8>,     
}


#[derive(Debug, Clone)]
pub struct Lf {
    pub functions: Vec<Aq>,
    pub strings: Vec<String>,     
    pub entry: usize,             
}


struct CallFrame {
    func_idx: usize,
    ip: usize,
    base: usize, 
    locals: [Value; 256],
}

impl CallFrame {
    fn new(func_idx: usize, base: usize) -> Self {
        const Arr: Value = Value::Void;
        Self {
            func_idx,
            ip: 0,
            base,
            locals: [Arr; 256],
        }
    }
}


const AOZ_: u8 = 0;
const APA_: u8 = 1;
const AOU_: u8 = 2;
const APB_: u8 = 3;
const APL_: u8 = 4;
const APK_: u8 = 5;
const APH_: u8 = 6;
const AOH_: u8 = 7;
const AOY_: u8 = 8;
const AOK_: u8 = 9;
const AOQ_: u8 = 10;
const AON_: u8 = 11;
const AOM_: u8 = 12;
const APE_: u8 = 13;
const APD_: u8 = 14;
const AOR_: u8 = 15;
const AOO_: u8 = 16;
const APG_: u8 = 17;
const APJ_: u8 = 18;
const APC_: u8 = 19;

const AOJ_: u8 = 20;
const AOI_: u8 = 21;
const API_: u8 = 22;
const AOV_: u8 = 23;
const AOT_: u8 = 24;
const APM_: u8 = 25;
const AOP_: u8 = 26;
const AOS_: u8 = 27;
const APF_: u8 = 28;
const AOL_: u8 = 29;
const AOW_: u8 = 30;
const AOX_: u8 = 31;


pub fn ehg(name: &str) -> Option<u8> {
    match name {
        "print" => Some(AOZ_),
        "println" => Some(APA_),
        "len" => Some(AOU_),
        "push" => Some(APB_),
        "to_string" => Some(APL_),
        "to_int" => Some(APK_),
        "to_float" => Some(APJ_),
        "sqrt" => Some(APH_),
        "abs" => Some(AOH_),
        "pixel" => Some(AOY_),
        "clear_screen" => Some(AOK_),
        "fill_rect" => Some(AOQ_),
        "draw_line" => Some(AON_),
        "draw_circle" => Some(AOM_),
        "screen_w" => Some(APE_),
        "screen_h" => Some(APD_),
        "flush" => Some(AOR_),
        "draw_text" => Some(AOO_),
        "sleep" => Some(APG_),
        "read_line" => Some(APC_),
        
        "beat" => Some(AOJ_),
        "bass" => Some(AOI_),
        "sub_bass" => Some(API_),
        "mid" => Some(AOV_),
        "high_mid" => Some(AOT_),
        "treble" => Some(APM_),
        "energy" => Some(AOP_),
        "frame_num" => Some(AOS_),
        "sin_f" => Some(APF_),
        "cos_f" => Some(AOL_),
        "mouse_x" => Some(AOW_),
        "mouse_y" => Some(AOX_),
        _ => None,
    }
}


pub fn execute(dkd: &Lf) -> Result<String, String> {
    let mut output = String::new();
    let mut dn: Vec<Value> = Vec::with_capacity(1024);
    let mut frames: Vec<CallFrame> = Vec::with_capacity(64);

    frames.push(CallFrame::new(dkd.entry, 0));

    let ayd = 500_000_000; 
    let mut steps = 0;

    loop {
        steps += 1;
        if steps > ayd {
            return Err(String::from("execution limit exceeded (10M steps)"));
        }

        let frame = frames.last_mut().ok_or("no call frame")?;
        let func = &dkd.functions[frame.func_idx];

        if frame.ip >= func.code.len() {
            
            if frames.len() <= 1 { break; }
            frames.pop();
            dn.push(Value::Void);
            continue;
        }

        let ish = func.code[frame.ip];
        frame.ip += 1;

        let op = match Op::atw(ish) {
            Some(ays) => ays,
            None => return Err(format!("unknown opcode: {}", ish)),
        };

        match op {
            Op::PushI64 => {
                let bytes = read_bytes(&func.code, &mut frame.ip, 8);
                let v = i64::from_le_bytes(bytes.try_into().unwrap());
                dn.push(Value::I64(v));
            }
            Op::PushF64 => {
                let bytes = read_bytes(&func.code, &mut frame.ip, 8);
                let v = f64::from_le_bytes(bytes.try_into().unwrap());
                dn.push(Value::F64(v));
            }
            Op::PushBool => {
                let v = func.code[frame.ip] != 0;
                frame.ip += 1;
                dn.push(Value::Bool(v));
            }
            Op::PushStr => {
                let idx = read_u16(&func.code, &mut frame.ip) as usize;
                let j = dkd.strings.get(idx).cloned().unwrap_or_default();
                dn.push(Value::Str(j));
            }
            Op::Pop => { dn.pop(); }
            Op::Dup => {
                let v = dn.last().cloned().unwrap_or(Value::Void);
                dn.push(v);
            }
            Op::LoadLocal => {
                let slot = func.code[frame.ip] as usize;
                frame.ip += 1;
                dn.push(frame.locals[slot].clone());
            }
            Op::StoreLocal => {
                let slot = func.code[frame.ip] as usize;
                frame.ip += 1;
                let val = dn.pop().unwrap_or(Value::Void);
                frame.locals[slot] = val;
            }
            Op::LoadGlobal | Op::StoreGlobal => {
                
                frame.ip += 2;
            }
            
            Op::AddI => { bxt(&mut dn, |a, b| a.wrapping_add(b), |a, b| a + b)?; }
            Op::SubI => { bxt(&mut dn, |a, b| a.wrapping_sub(b), |a, b| a - b)?; }
            Op::MulI => { bxt(&mut dn, |a, b| a.wrapping_mul(b), |a, b| a * b)?; }
            Op::DivI => {
                let awd = dn.pop().unwrap_or(Value::I64(0));
                let bel = dn.pop().unwrap_or(Value::I64(0));
                match (&bel, &awd) {
                    (Value::F64(a), Value::F64(b)) => dn.push(Value::F64(a / b)),
                    (Value::I64(a), Value::F64(b)) => dn.push(Value::F64(*a as f64 / b)),
                    (Value::F64(a), Value::I64(b)) => dn.push(Value::F64(a / *b as f64)),
                    _ => {
                        let b = awd.as_i64()?;
                        let a = bel.as_i64()?;
                        if b == 0 { return Err(String::from("division by zero")); }
                        dn.push(Value::I64(a / b));
                    }
                }
            }
            Op::ModI => {
                let awd = dn.pop().unwrap_or(Value::I64(0));
                let bel = dn.pop().unwrap_or(Value::I64(0));
                match (&bel, &awd) {
                    (Value::F64(a), Value::F64(b)) => dn.push(Value::F64(a % b)),
                    (Value::I64(a), Value::F64(b)) => dn.push(Value::F64(*a as f64 % b)),
                    (Value::F64(a), Value::I64(b)) => dn.push(Value::F64(a % *b as f64)),
                    _ => {
                        let b = awd.as_i64()?;
                        let a = bel.as_i64()?;
                        if b == 0 { return Err(String::from("modulo by zero")); }
                        dn.push(Value::I64(a % b));
                    }
                }
            }
            Op::NegI => {
                let v = dn.pop().unwrap_or(Value::I64(0));
                match v {
                    Value::F64(f) => dn.push(Value::F64(-f)),
                    _ => dn.push(Value::I64(-v.as_i64()?)),
                }
            }
            
            Op::AddF => { egv(&mut dn, |a, b| a + b)?; }
            Op::SubF => { egv(&mut dn, |a, b| a - b)?; }
            Op::MulF => { egv(&mut dn, |a, b| a * b)?; }
            Op::DivF => { egv(&mut dn, |a, b| a / b)?; }
            Op::NegF => {
                let v = dn.pop().unwrap_or(Value::F64(0.0)).as_f64()?;
                dn.push(Value::F64(-v));
            }
            
            Op::EqI => { cvf(&mut dn, |a, b| a == b, |a, b| a == b)?; }
            Op::NeI => { cvf(&mut dn, |a, b| a != b, |a, b| a != b)?; }
            Op::LtI => { cvf(&mut dn, |a, b| a < b, |a, b| a < b)?; }
            Op::GtI => { cvf(&mut dn, |a, b| a > b, |a, b| a > b)?; }
            Op::LeI => { cvf(&mut dn, |a, b| a <= b, |a, b| a <= b)?; }
            Op::GeI => { cvf(&mut dn, |a, b| a >= b, |a, b| a >= b)?; }
            
            Op::EqF => { cve(&mut dn, |a, b| a == b)?; }
            Op::NeF => { cve(&mut dn, |a, b| a != b)?; }
            Op::LtF => { cve(&mut dn, |a, b| a < b)?; }
            Op::GtF => { cve(&mut dn, |a, b| a > b)?; }
            Op::LeF => { cve(&mut dn, |a, b| a <= b)?; }
            Op::GeF => { cve(&mut dn, |a, b| a >= b)?; }
            
            Op::And => {
                let b = dn.pop().unwrap_or(Value::Bool(false)).as_bool()?;
                let a = dn.pop().unwrap_or(Value::Bool(false)).as_bool()?;
                dn.push(Value::Bool(a && b));
            }
            Op::Or => {
                let b = dn.pop().unwrap_or(Value::Bool(false)).as_bool()?;
                let a = dn.pop().unwrap_or(Value::Bool(false)).as_bool()?;
                dn.push(Value::Bool(a || b));
            }
            Op::Not => {
                let v = dn.pop().unwrap_or(Value::Bool(false)).as_bool()?;
                dn.push(Value::Bool(!v));
            }
            
            Op::BitAnd => { bxt(&mut dn, |a, b| a & b, |a, b| (a as i64 & b as i64) as f64)?; }
            Op::BitOr => { bxt(&mut dn, |a, b| a | b, |a, b| (a as i64 | b as i64) as f64)?; }
            Op::BitXor => { bxt(&mut dn, |a, b| a ^ b, |a, b| (a as i64 ^ b as i64) as f64)?; }
            Op::Shl => { bxt(&mut dn, |a, b| a << (b & 63), |a, b| ((a as i64) << (b as i64 & 63)) as f64)?; }
            Op::Shr => { bxt(&mut dn, |a, b| a >> (b & 63), |a, b| ((a as i64) >> (b as i64 & 63)) as f64)?; }
            
            Op::I64toF64 => {
                let v = dn.pop().unwrap_or(Value::I64(0)).as_i64()?;
                dn.push(Value::F64(v as f64));
            }
            Op::F64toI64 => {
                let v = dn.pop().unwrap_or(Value::F64(0.0)).as_f64()?;
                dn.push(Value::I64(v as i64));
            }
            
            Op::Jump => {
                let off = read_u16(&func.code, &mut frame.ip) as usize;
                frame.ip = off;
            }
            Op::JumpIfFalse => {
                let off = read_u16(&func.code, &mut frame.ip) as usize;
                let fc = dn.pop().unwrap_or(Value::Bool(false)).as_bool()?;
                if !fc { frame.ip = off; }
            }
            Op::Call => {
                let func_idx = read_u16(&func.code, &mut frame.ip) as usize;
                let anl = func.code[frame.ip] as usize;
                frame.ip += 1;
                
                let mut args = Vec::with_capacity(anl);
                for _ in 0..anl {
                    args.push(dn.pop().unwrap_or(Value::Void));
                }
                args.reverse();
                
                let mut ipy = CallFrame::new(func_idx, dn.len());
                for (i, db) in args.into_iter().enumerate() {
                    ipy.locals[i] = db;
                }
                frames.push(ipy);
            }
            Op::CallBuiltin => {
                let kfx = func.code[frame.ip];
                frame.ip += 1;
                let anl = func.code[frame.ip] as usize;
                frame.ip += 1;
                let mut args = Vec::with_capacity(anl);
                for _ in 0..anl {
                    args.push(dn.pop().unwrap_or(Value::Void));
                }
                args.reverse();
                let result = lrt(kfx, &args, &mut output)?;
                dn.push(result);
            }
            Op::Return => {
                let ret = dn.pop().unwrap_or(Value::Void);
                if frames.len() <= 1 {
                    dn.push(ret);
                    break;
                }
                frames.pop();
                dn.push(ret);
            }
            
            Op::NewArray => {
                let count = read_u16(&func.code, &mut frame.ip) as usize;
                let mut ik = Vec::with_capacity(count);
                for _ in 0..count {
                    ik.push(dn.pop().unwrap_or(Value::Void));
                }
                ik.reverse();
                dn.push(Value::Array(ik));
            }
            Op::ArrayGet => {
                let idx = dn.pop().unwrap_or(Value::I64(0)).as_i64()? as usize;
                let ik = dn.pop().unwrap_or(Value::Array(Vec::new()));
                match ik {
                    Value::Array(a) => {
                        let v = a.get(idx).cloned().unwrap_or(Value::Void);
                        dn.push(v);
                    }
                    Value::Str(j) => {
                        let c = j.as_bytes().get(idx).copied().unwrap_or(0);
                        dn.push(Value::I64(c as i64));
                    }
                    _ => return Err(String::from("index on non-array")),
                }
            }
            Op::ArraySet => {
                let val = dn.pop().unwrap_or(Value::Void);
                let idx = dn.pop().unwrap_or(Value::I64(0)).as_i64()? as usize;
                let ik = dn.pop().unwrap_or(Value::Array(Vec::new()));
                match ik {
                    Value::Array(mut a) => {
                        if idx < a.len() { a[idx] = val; }
                        dn.push(Value::Array(a));
                    }
                    _ => return Err(String::from("index-set on non-array")),
                }
            }
            Op::ArrayLen => {
                let v = dn.pop().unwrap_or(Value::Void);
                let len = match &v {
                    Value::Array(a) => a.len() as i64,
                    Value::Str(j) => j.len() as i64,
                    _ => 0,
                };
                dn.push(Value::I64(len));
            }
            Op::ArrayPush => {
                let val = dn.pop().unwrap_or(Value::Void);
                let ik = dn.pop().unwrap_or(Value::Array(Vec::new()));
                match ik {
                    Value::Array(mut a) => {
                        a.push(val);
                        dn.push(Value::Array(a));
                    }
                    _ => return Err(String::from("push on non-array")),
                }
            }
            Op::StrConcat => {
                let b = dn.pop().unwrap_or(Value::Str(String::new())).to_display();
                let a = dn.pop().unwrap_or(Value::Str(String::new())).to_display();
                dn.push(Value::Str(format!("{}{}", a, b)));
            }
            Op::Halt => break,
        }
    }

    Ok(output)
}



fn read_bytes(code: &[u8], ip: &mut usize, ae: usize) -> Vec<u8> {
    let bytes = code[*ip..*ip + ae].to_vec();
    *ip += ae;
    bytes
}

fn read_u16(code: &[u8], ip: &mut usize) -> u16 {
    let v = u16::from_le_bytes([code[*ip], code[*ip + 1]]);
    *ip += 2;
    v
}

fn bxt(dn: &mut Vec<Value>, axb: fn(i64, i64) -> i64, ff: fn(f64, f64) -> f64) -> Result<(), String> {
    let awd = dn.pop().unwrap_or(Value::I64(0));
    let bel = dn.pop().unwrap_or(Value::I64(0));
    
    match (&bel, &awd) {
        (Value::F64(a), Value::F64(b)) => dn.push(Value::F64(ff(*a, *b))),
        (Value::I64(a), Value::F64(b)) => dn.push(Value::F64(ff(*a as f64, *b))),
        (Value::F64(a), Value::I64(b)) => dn.push(Value::F64(ff(*a, *b as f64))),
        _ => {
            let a = bel.as_i64()?;
            let b = awd.as_i64()?;
            dn.push(Value::I64(axb(a, b)));
        }
    }
    Ok(())
}

fn egv(dn: &mut Vec<Value>, f: fn(f64, f64) -> f64) -> Result<(), String> {
    let b = dn.pop().unwrap_or(Value::F64(0.0)).as_f64()?;
    let a = dn.pop().unwrap_or(Value::F64(0.0)).as_f64()?;
    dn.push(Value::F64(f(a, b)));
    Ok(())
}

fn cvf(dn: &mut Vec<Value>, axb: fn(i64, i64) -> bool, ff: fn(f64, f64) -> bool) -> Result<(), String> {
    let awd = dn.pop().unwrap_or(Value::I64(0));
    let bel = dn.pop().unwrap_or(Value::I64(0));
    
    match (&bel, &awd) {
        (Value::F64(a), Value::F64(b)) => dn.push(Value::Bool(ff(*a, *b))),
        (Value::I64(a), Value::F64(b)) => dn.push(Value::Bool(ff(*a as f64, *b))),
        (Value::F64(a), Value::I64(b)) => dn.push(Value::Bool(ff(*a, *b as f64))),
        _ => {
            let a = bel.as_i64()?;
            let b = awd.as_i64()?;
            dn.push(Value::Bool(axb(a, b)));
        }
    }
    Ok(())
}

fn cve(dn: &mut Vec<Value>, f: fn(f64, f64) -> bool) -> Result<(), String> {
    let b = dn.pop().unwrap_or(Value::F64(0.0)).as_f64()?;
    let a = dn.pop().unwrap_or(Value::F64(0.0)).as_f64()?;
    dn.push(Value::Bool(f(a, b)));
    Ok(())
}


fn lrt(id: u8, args: &[Value], output: &mut String) -> Result<Value, String> {
    match id {
        AOZ_ => {
            for db in args { output.push_str(&db.to_display()); }
            Ok(Value::Void)
        }
        APA_ => {
            for db in args { output.push_str(&db.to_display()); }
            output.push('\n');
            Ok(Value::Void)
        }
        AOU_ => {
            let v = args.first().unwrap_or(&Value::Void);
            match v {
                Value::Array(a) => Ok(Value::I64(a.len() as i64)),
                Value::Str(j) => Ok(Value::I64(j.len() as i64)),
                _ => Ok(Value::I64(0)),
            }
        }
        APB_ => {
            if args.len() >= 2 {
                if let Value::Array(mut a) = args[0].clone() {
                    a.push(args[1].clone());
                    return Ok(Value::Array(a));
                }
            }
            Err(String::from("push expects (array, value)"))
        }
        APL_ => {
            let v = args.first().unwrap_or(&Value::Void);
            Ok(Value::Str(v.to_display()))
        }
        APK_ => {
            let v = args.first().unwrap_or(&Value::Void);
            match v {
                Value::I64(ae) => Ok(Value::I64(*ae)),
                Value::F64(f) => Ok(Value::I64(*f as i64)),
                Value::Bool(b) => Ok(Value::I64(if *b { 1 } else { 0 })),
                Value::Str(j) => {
                    let ae: i64 = nqo(j.trim());
                    Ok(Value::I64(ae))
                }
                _ => Ok(Value::I64(0)),
            }
        }
        APH_ => {
            let v = args.first().unwrap_or(&Value::F64(0.0)).as_f64().unwrap_or(0.0);
            Ok(Value::F64(libm::sqrt(v)))
        }
        AOH_ => {
            match args.first().unwrap_or(&Value::I64(0)) {
                Value::I64(ae) => Ok(Value::I64(ae.abs())),
                Value::F64(f) => Ok(Value::F64(libm::fabs(*f))),
                _ => Ok(Value::I64(0)),
            }
        }
        
        
        
        AOY_ => {
            
            let x = args.get(0).and_then(|v| v.as_i64().ok()).unwrap_or(0) as u32;
            let y = args.get(1).and_then(|v| v.as_i64().ok()).unwrap_or(0) as u32;
            let r = args.get(2).and_then(|v| v.as_i64().ok()).unwrap_or(255) as u32 & 0xFF;
            let g = args.get(3).and_then(|v| v.as_i64().ok()).unwrap_or(255) as u32 & 0xFF;
            let b = args.get(4).and_then(|v| v.as_i64().ok()).unwrap_or(255) as u32 & 0xFF;
            let color = 0xFF000000 | (r << 16) | (g << 8) | b;
            crate::framebuffer::put_pixel(x, y, color);
            Ok(Value::Void)
        }
        AOK_ => {
            
            let r = args.get(0).and_then(|v| v.as_i64().ok()).unwrap_or(0) as u32 & 0xFF;
            let g = args.get(1).and_then(|v| v.as_i64().ok()).unwrap_or(0) as u32 & 0xFF;
            let b = args.get(2).and_then(|v| v.as_i64().ok()).unwrap_or(0) as u32 & 0xFF;
            let color = 0xFF000000 | (r << 16) | (g << 8) | b;
            let (dy, dw) = crate::framebuffer::kv();
            crate::framebuffer::fill_rect(0, 0, dy, dw, color);
            Ok(Value::Void)
        }
        AOQ_ => {
            
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
        AON_ => {
            
            let bm = args.get(0).and_then(|v| v.as_i64().ok()).unwrap_or(0);
            let az = args.get(1).and_then(|v| v.as_i64().ok()).unwrap_or(0);
            let x1 = args.get(2).and_then(|v| v.as_i64().ok()).unwrap_or(0);
            let y1 = args.get(3).and_then(|v| v.as_i64().ok()).unwrap_or(0);
            let r = args.get(4).and_then(|v| v.as_i64().ok()).unwrap_or(255) as u32 & 0xFF;
            let g = args.get(5).and_then(|v| v.as_i64().ok()).unwrap_or(255) as u32 & 0xFF;
            let b = args.get(6).and_then(|v| v.as_i64().ok()).unwrap_or(255) as u32 & 0xFF;
            let color = 0xFF000000 | (r << 16) | (g << 8) | b;
            
            let mut cx = bm;
            let mut u = az;
            let dx = (x1 - bm).abs();
            let ad = -(y1 - az).abs();
            let am: i64 = if bm < x1 { 1 } else { -1 };
            let ak: i64 = if az < y1 { 1 } else { -1 };
            let mut err = dx + ad;
            loop {
                if cx >= 0 && u >= 0 {
                    crate::framebuffer::put_pixel(cx as u32, u as u32, color);
                }
                if cx == x1 && u == y1 { break; }
                let pg = 2 * err;
                if pg >= ad { err += ad; cx += am; }
                if pg <= dx { err += dx; u += ak; }
            }
            Ok(Value::Void)
        }
        AOM_ => {
            
            let cx = args.get(0).and_then(|v| v.as_i64().ok()).unwrap_or(0);
            let u = args.get(1).and_then(|v| v.as_i64().ok()).unwrap_or(0);
            let radius = args.get(2).and_then(|v| v.as_i64().ok()).unwrap_or(0);
            let r = args.get(3).and_then(|v| v.as_i64().ok()).unwrap_or(255) as u32 & 0xFF;
            let g = args.get(4).and_then(|v| v.as_i64().ok()).unwrap_or(255) as u32 & 0xFF;
            let b = args.get(5).and_then(|v| v.as_i64().ok()).unwrap_or(255) as u32 & 0xFF;
            let color = 0xFF000000 | (r << 16) | (g << 8) | b;
            
            let mut x = radius;
            let mut y: i64 = 0;
            let mut d = 1 - radius;
            while x >= y {
                let pts = [
                    (cx + x, u + y), (cx - x, u + y),
                    (cx + x, u - y), (cx - x, u - y),
                    (cx + y, u + x), (cx - y, u + x),
                    (cx + y, u - x), (cx - y, u - x),
                ];
                for (p, o) in pts {
                    if p >= 0 && o >= 0 {
                        crate::framebuffer::put_pixel(p as u32, o as u32, color);
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
        APE_ => {
            let (w, _) = crate::framebuffer::kv();
            Ok(Value::I64(w as i64))
        }
        APD_ => {
            let (_, h) = crate::framebuffer::kv();
            Ok(Value::I64(h as i64))
        }
        AOR_ => {
            
            crate::framebuffer::ii();
            Ok(Value::Void)
        }
        AOO_ => {
            
            if let Some(Value::Str(text)) = args.get(0) {
                let x = args.get(1).and_then(|v| v.as_i64().ok()).unwrap_or(0) as u32;
                let y = args.get(2).and_then(|v| v.as_i64().ok()).unwrap_or(0) as u32;
                let r = args.get(3).and_then(|v| v.as_i64().ok()).unwrap_or(255) as u32 & 0xFF;
                let g = args.get(4).and_then(|v| v.as_i64().ok()).unwrap_or(255) as u32 & 0xFF;
                let b = args.get(5).and_then(|v| v.as_i64().ok()).unwrap_or(255) as u32 & 0xFF;
                let scale = args.get(6).and_then(|v| v.as_i64().ok()).unwrap_or(1) as u32;
                let color = 0xFF000000 | (r << 16) | (g << 8) | b;
                
                let mut cx = x;
                for c in text.chars() {
                    let du = crate::framebuffer::font::ol(c);
                    for (row, &bits) in du.iter().enumerate() {
                        for bf in 0..8u32 {
                            if bits & (0x80 >> bf) != 0 {
                                for ak in 0..scale {
                                    for am in 0..scale {
                                        crate::framebuffer::put_pixel(
                                            cx + bf * scale + am,
                                            y + row as u32 * scale + ak,
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
        APG_ => {
            
            let dh = args.get(0).and_then(|v| v.as_i64().ok()).unwrap_or(0) as u64;
            crate::cpu::tsc::hq(dh);
            Ok(Value::Void)
        }
        APJ_ => {
            let v = args.first().unwrap_or(&Value::Void);
            match v {
                Value::F64(f) => Ok(Value::F64(*f)),
                Value::I64(ae) => Ok(Value::F64(*ae as f64)),
                Value::Bool(b) => Ok(Value::F64(if *b { 1.0 } else { 0.0 })),
                Value::Str(j) => {
                    let f = nqf(j.trim());
                    Ok(Value::F64(f))
                }
                _ => Ok(Value::F64(0.0)),
            }
        }
        APC_ => {
            
            let line = crate::shell::read_line();
            Ok(Value::Str(line))
        }
        
        
        
        AOJ_ => Ok(Value::F64(crate::trustdaw::live_viz::mcr() as f64)),
        AOI_ => Ok(Value::F64(crate::trustdaw::live_viz::mcq() as f64)),
        API_ => Ok(Value::F64(crate::trustdaw::live_viz::mdw() as f64)),
        AOV_ => Ok(Value::F64(crate::trustdaw::live_viz::mdm() as f64)),
        AOT_ => Ok(Value::F64(crate::trustdaw::live_viz::mde() as f64)),
        APM_ => Ok(Value::F64(crate::trustdaw::live_viz::mea() as f64)),
        AOP_ => Ok(Value::F64(crate::trustdaw::live_viz::mdb() as f64)),
        AOS_ => Ok(Value::I64(crate::trustdaw::live_viz::mdd() as i64)),
        APF_ => {
            let x = args.first().unwrap_or(&Value::F64(0.0)).as_f64().unwrap_or(0.0);
            Ok(Value::F64(libm::sin(x)))
        }
        AOL_ => {
            let x = args.first().unwrap_or(&Value::F64(0.0)).as_f64().unwrap_or(0.0);
            Ok(Value::F64(libm::cos(x)))
        }
        AOW_ => {
            let d = crate::desktop::S.lock();
            let cg = d.cursor_x;
            drop(d);
            Ok(Value::I64(cg as i64))
        }
        AOX_ => {
            let d = crate::desktop::S.lock();
            let cr = d.cursor_y;
            drop(d);
            Ok(Value::I64(cr as i64))
        }
        _ => Err(format!("unknown builtin id: {}", id)),
    }
}


fn nqo(j: &str) -> i64 {
    let mut val: i64 = 0;
    let mut neg = false;
    for (i, ch) in j.chars().enumerate() {
        if i == 0 && ch == '-' { neg = true; continue; }
        if !ch.is_ascii_digit() { break; }
        val = val.wrapping_mul(10).wrapping_add((ch as i64) - 48);
    }
    if neg { -val } else { val }
}


fn nqf(j: &str) -> f64 {
    let mut val: f64 = 0.0;
    let mut neg = false;
    let mut yt = false;
    let mut dqe: f64 = 1.0;
    for (i, ch) in j.chars().enumerate() {
        if i == 0 && ch == '-' { neg = true; continue; }
        if ch == '.' && !yt { yt = true; continue; }
        if !ch.is_ascii_digit() { break; }
        let d = (ch as u8 - b'0') as f64;
        if yt {
            dqe *= 10.0;
            val += d / dqe;
        } else {
            val = val * 10.0 + d;
        }
    }
    if neg { -val } else { val }
}
