




use alloc::string::String;
use alloc::vec::Vec;
use alloc::collections::BTreeMap;
use alloc::format;

use super::parser::*;
use super::vm::{Op, Aq, Lf, ehg};


struct Compiler {
    functions: Vec<Aq>,
    func_map: BTreeMap<String, usize>, 
    strings: Vec<String>,
    string_map: BTreeMap<String, usize>,
}


struct FnCompiler {
    code: Vec<u8>,
    locals: BTreeMap<String, u8>,
    next_local: u8,
    loop_starts: Vec<usize>,      
    loop_breaks: Vec<Vec<usize>>, 
}

impl Compiler {
    fn new() -> Self {
        Self {
            functions: Vec::new(),
            func_map: BTreeMap::new(),
            strings: Vec::new(),
            string_map: BTreeMap::new(),
        }
    }

    fn intern_string(&mut self, j: &str) -> u16 {
        if let Some(&idx) = self.string_map.get(j) {
            return idx as u16;
        }
        let idx = self.strings.len();
        self.strings.push(String::from(j));
        self.string_map.insert(String::from(j), idx);
        idx as u16
    }

    fn register_functions(&mut self, program: &Program) {
        for (i, item) in program.items.iter().enumerate() {
            if let Item::Aq(f) = item {
                self.func_map.insert(f.name.clone(), i);
            }
        }
    }

    fn compile_program(&mut self, program: &Program) -> Result<Lf, String> {
        self.register_functions(program);

        for item in &program.items {
            match item {
                Item::Aq(f) => self.compile_fn(f)?,
                Item::Struct(_) => {} 
            }
        }

        let entry = self.func_map.get("main")
            .ok_or_else(|| String::from("no main() function found"))?;
        let entry = *entry;

        Ok(Lf {
            functions: self.functions.clone(),
            strings: self.strings.clone(),
            entry,
        })
    }

    fn compile_fn(&mut self, decl: &Md) -> Result<(), String> {
        let mut br = FnCompiler::new();

        
        for (name, _ty) in &decl.params {
            br.add_local(name);
        }

        
        self.compile_block(&mut br, &decl.body)?;

        
        if br.code.last().copied() != Some(Op::Return as u8) {
            br.emit_op(&mut self.strings, Op::PushI64);
            br.emit_i64(0); 
            br.emit_op(&mut self.strings, Op::Return);
        }

        let func = Aq {
            name: decl.name.clone(),
            arity: decl.params.len() as u8,
            locals: br.next_local,
            code: br.code,
        };
        self.functions.push(func);
        Ok(())
    }

    fn compile_block(&mut self, br: &mut FnCompiler, block: &Bl) -> Result<(), String> {
        for stmt in &block.stmts {
            self.compile_stmt(br, stmt)?;
        }
        Ok(())
    }

    fn compile_stmt(&mut self, br: &mut FnCompiler, stmt: &Stmt) -> Result<(), String> {
        match stmt {
            Stmt::Let { name, init, .. } => {
                let slot = br.add_local(name);
                if let Some(expr) = init {
                    self.compile_expr(br, expr)?;
                    br.emit_op(&mut self.strings, Op::StoreLocal);
                    br.code.push(slot);
                }
            }
            Stmt::Assign { target, value } => {
                self.compile_expr(br, value)?;
                self.compile_store(br, target)?;
            }
            Stmt::OpAssign { op, target, value } => {
                self.compile_expr(br, target)?;
                self.compile_expr(br, value)?;
                
                let hbt = match op {
                    BinOp::Add => Op::AddI,
                    BinOp::Sub => Op::SubI,
                    BinOp::Mul => Op::MulI,
                    BinOp::Div => Op::DivI,
                    _ => Op::AddI,
                };
                br.emit_op(&mut self.strings, hbt);
                self.compile_store(br, target)?;
            }
            Stmt::Expr(expr) => {
                self.compile_expr(br, expr)?;
                br.emit_op(&mut self.strings, Op::Pop); 
            }
            Stmt::Return(val) => {
                if let Some(expr) = val {
                    self.compile_expr(br, expr)?;
                } else {
                    br.emit_op(&mut self.strings, Op::PushI64);
                    br.emit_i64(0);
                }
                br.emit_op(&mut self.strings, Op::Return);
            }
            Stmt::If { fc, avj, atp } => {
                self.compile_expr(br, fc)?;
                let iit = br.emit_jump(Op::JumpIfFalse);

                self.compile_block(br, avj)?;

                if let Some(else_blk) = atp {
                    let mvg = br.emit_jump(Op::Jump);
                    br.patch_jump(iit);
                    self.compile_block(br, else_blk)?;
                    br.patch_jump(mvg);
                } else {
                    br.patch_jump(iit);
                }
            }
            Stmt::While { fc, body } => {
                let cbp = br.code.len();
                br.loop_starts.push(cbp);
                br.loop_breaks.push(Vec::new());

                self.compile_expr(br, fc)?;
                let fvq = br.emit_jump(Op::JumpIfFalse);

                self.compile_block(br, body)?;
                br.emit_jump_to(cbp);

                br.patch_jump(fvq);

                
                let cgl = br.loop_breaks.pop().unwrap_or_default();
                for b in cgl { br.patch_jump(b); }
                br.loop_starts.pop();
            }
            Stmt::For { ael, iter, body } => {
                
                let slot = br.add_local(ael);
                
                if let Expr::Range { start, end } = iter {
                    self.compile_expr(br, start)?;
                    br.emit_op(&mut self.strings, Op::StoreLocal);
                    br.code.push(slot);

                    
                    let hvv = br.add_local(&format!("__for_end_{}", slot));
                    self.compile_expr(br, end)?;
                    br.emit_op(&mut self.strings, Op::StoreLocal);
                    br.code.push(hvv);

                    let cbp = br.code.len();
                    br.loop_starts.push(cbp);
                    br.loop_breaks.push(Vec::new());

                    
                    br.emit_op(&mut self.strings, Op::LoadLocal);
                    br.code.push(slot);
                    br.emit_op(&mut self.strings, Op::LoadLocal);
                    br.code.push(hvv);
                    br.emit_op(&mut self.strings, Op::LtI);

                    let fvq = br.emit_jump(Op::JumpIfFalse);
                    self.compile_block(br, body)?;

                    
                    br.emit_op(&mut self.strings, Op::LoadLocal);
                    br.code.push(slot);
                    br.emit_op(&mut self.strings, Op::PushI64);
                    br.emit_i64(1);
                    br.emit_op(&mut self.strings, Op::AddI);
                    br.emit_op(&mut self.strings, Op::StoreLocal);
                    br.code.push(slot);

                    br.emit_jump_to(cbp);
                    br.patch_jump(fvq);

                    let cgl = br.loop_breaks.pop().unwrap_or_default();
                    for b in cgl { br.patch_jump(b); }
                    br.loop_starts.pop();
                } else {
                    return Err(String::from("for loop requires a range expression"));
                }
            }
            Stmt::Loop(body) => {
                let cbp = br.code.len();
                br.loop_starts.push(cbp);
                br.loop_breaks.push(Vec::new());

                self.compile_block(br, body)?;
                br.emit_jump_to(cbp);

                let cgl = br.loop_breaks.pop().unwrap_or_default();
                for b in cgl { br.patch_jump(b); }
                br.loop_starts.pop();
            }
            Stmt::Break => {
                let ay = br.emit_jump(Op::Jump);
                if let Some(cgl) = br.loop_breaks.last_mut() {
                    cgl.push(ay);
                }
            }
            Stmt::Continue => {
                if let Some(&start) = br.loop_starts.last() {
                    br.emit_jump_to(start);
                }
            }
        }
        Ok(())
    }

    fn compile_expr(&mut self, br: &mut FnCompiler, expr: &Expr) -> Result<(), String> {
        match expr {
            Expr::IntLit(v) => {
                br.emit_op(&mut self.strings, Op::PushI64);
                br.emit_i64(*v);
            }
            Expr::FloatLit(v) => {
                br.emit_op(&mut self.strings, Op::PushF64);
                br.emit_f64(*v);
            }
            Expr::StringLit(j) => {
                let idx = self.intern_string(j);
                br.emit_op(&mut self.strings, Op::PushStr);
                br.emit_u16(idx);
            }
            Expr::BoolLit(b) => {
                br.emit_op(&mut self.strings, Op::PushBool);
                br.code.push(if *b { 1 } else { 0 });
            }
            Expr::Ident(name) => {
                if let Some(&slot) = br.locals.get(name) {
                    br.emit_op(&mut self.strings, Op::LoadLocal);
                    br.code.push(slot);
                } else {
                    return Err(format!("undefined variable: {}", name));
                }
            }
            Expr::BinOp { op, left, right } => {
                self.compile_expr(br, left)?;
                self.compile_expr(br, right)?;
                let hbt = match op {
                    BinOp::Add => Op::AddI,
                    BinOp::Sub => Op::SubI,
                    BinOp::Mul => Op::MulI,
                    BinOp::Div => Op::DivI,
                    BinOp::Mod => Op::ModI,
                    BinOp::Eq => Op::EqI,
                    BinOp::NotEq => Op::NeI,
                    BinOp::Lt => Op::LtI,
                    BinOp::Gt => Op::GtI,
                    BinOp::LtEq => Op::LeI,
                    BinOp::GtEq => Op::GeI,
                    BinOp::And => Op::And,
                    BinOp::Or => Op::Or,
                    BinOp::BitAnd => Op::BitAnd,
                    BinOp::BitOr => Op::BitOr,
                    BinOp::BitXor => Op::BitXor,
                    BinOp::Shl => Op::Shl,
                    BinOp::Shr => Op::Shr,
                };
                br.emit_op(&mut self.strings, hbt);
            }
            Expr::UnaryOp { op, expr } => {
                self.compile_expr(br, expr)?;
                match op {
                    UnaryOp::Neg => br.emit_op(&mut self.strings, Op::NegI),
                    UnaryOp::Not => br.emit_op(&mut self.strings, Op::Not),
                }
            }
            Expr::Call { func, args } => {
                
                if let Some(bid) = ehg(func) {
                    for db in args {
                        self.compile_expr(br, db)?;
                    }
                    br.emit_op(&mut self.strings, Op::CallBuiltin);
                    br.code.push(bid);
                    br.code.push(args.len() as u8);
                } else if let Some(&fidx) = self.func_map.get(func) {
                    
                    for db in args {
                        self.compile_expr(br, db)?;
                    }
                    br.emit_op(&mut self.strings, Op::Call);
                    br.emit_u16(fidx as u16);
                    br.code.push(args.len() as u8);
                } else {
                    return Err(format!("undefined function: {}", func));
                }
            }
            Expr::Index { array, index } => {
                self.compile_expr(br, array)?;
                self.compile_expr(br, index)?;
                br.emit_op(&mut self.strings, Op::ArrayGet);
            }
            Expr::Array(doo) => {
                for cit in doo {
                    self.compile_expr(br, cit)?;
                }
                br.emit_op(&mut self.strings, Op::NewArray);
                br.emit_u16(doo.len() as u16);
            }
            Expr::Range { start, end } => {
                
                
                self.compile_expr(br, start)?;
                self.compile_expr(br, end)?;
                
                br.emit_op(&mut self.strings, Op::NewArray);
                br.emit_u16(2);
            }
            Expr::Cast { expr, ty } => {
                self.compile_expr(br, expr)?;
                match ty {
                    Type::F64 => br.emit_op(&mut self.strings, Op::I64toF64),
                    Type::I64 => br.emit_op(&mut self.strings, Op::F64toI64),
                    _ => {} 
                }
            }
            Expr::Bl(block) => {
                self.compile_block(br, block)?;
            }
            Expr::Field { .. } => {
                return Err(String::from("struct field access not yet supported"));
            }
        }
        Ok(())
    }

    fn compile_store(&mut self, br: &mut FnCompiler, target: &Expr) -> Result<(), String> {
        match target {
            Expr::Ident(name) => {
                if let Some(&slot) = br.locals.get(name) {
                    br.emit_op(&mut self.strings, Op::StoreLocal);
                    br.code.push(slot);
                } else {
                    return Err(format!("undefined variable: {}", name));
                }
            }
            Expr::Index { array, index } => {
                
                
                
                
                
                
                
                
                if let Expr::Ident(arr_name) = array.as_ref() {
                    if let Some(&arr_slot) = br.locals.get(arr_name) {
                        
                        let jls = br.next_local;
                        br.next_local += 1;
                        
                        br.emit_op(&mut self.strings, Op::StoreLocal);
                        br.code.push(jls);
                        
                        br.emit_op(&mut self.strings, Op::LoadLocal);
                        br.code.push(arr_slot);
                        
                        self.compile_expr(br, index)?;
                        
                        br.emit_op(&mut self.strings, Op::LoadLocal);
                        br.code.push(jls);
                        
                        br.emit_op(&mut self.strings, Op::ArraySet);
                        
                        br.emit_op(&mut self.strings, Op::StoreLocal);
                        br.code.push(arr_slot);
                    } else {
                        return Err(format!("undefined variable: {}", arr_name));
                    }
                } else {
                    return Err(String::from("array index assignment requires a variable"));
                }
            }
            _ => return Err(String::from("invalid assignment target")),
        }
        Ok(())
    }
}

impl FnCompiler {
    fn new() -> Self {
        Self {
            code: Vec::new(),
            locals: BTreeMap::new(),
            next_local: 0,
            loop_starts: Vec::new(),
            loop_breaks: Vec::new(),
        }
    }

    fn add_local(&mut self, name: &str) -> u8 {
        if let Some(&slot) = self.locals.get(name) {
            return slot;
        }
        let slot = self.next_local;
        self.locals.insert(String::from(name), slot);
        self.next_local += 1;
        slot
    }

    fn emit_op(&mut self, _strings: &mut Vec<String>, op: Op) {
        self.code.push(op as u8);
    }

    fn emit_i64(&mut self, v: i64) {
        self.code.extend_from_slice(&v.to_le_bytes());
    }

    fn emit_f64(&mut self, v: f64) {
        self.code.extend_from_slice(&v.to_le_bytes());
    }

    fn emit_u16(&mut self, v: u16) {
        self.code.extend_from_slice(&v.to_le_bytes());
    }

    
    fn emit_jump(&mut self, op: Op) -> usize {
        self.code.push(op as u8);
        let idx = self.code.len();
        self.code.push(0); 
        self.code.push(0);
        idx
    }

    
    fn patch_jump(&mut self, idx: usize) {
        let target = self.code.len() as u16;
        self.code[idx] = (target & 0xFF) as u8;
        self.code[idx + 1] = ((target >> 8) & 0xFF) as u8;
    }

    
    fn emit_jump_to(&mut self, target: usize) {
        self.code.push(Op::Jump as u8);
        self.code.push((target & 0xFF) as u8);
        self.code.push(((target >> 8) & 0xFF) as u8);
    }
}


pub fn kwd(program: &Program) -> Result<Lf, String> {
    let mut compiler = Compiler::new();
    compiler.compile_program(program)
}
