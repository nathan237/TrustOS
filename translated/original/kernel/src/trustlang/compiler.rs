//! TrustLang Compiler — AST to bytecode
//!
//! Compiles parsed AST into bytecode for the VM.
//! Handles: variable resolution, function calls, control flow.

use alloc::string::String;
use alloc::vec::Vec;
use alloc::collections::BTreeMap;
use alloc::format;

use super::parser::*;
use super::vm::{Op, Function, Bytecode, builtin_id};

/// Compiler state
struct Compiler {
    functions: Vec<Function>,
    func_map: BTreeMap<String, usize>, // name → function index
    strings: Vec<String>,
    string_map: BTreeMap<String, usize>,
}

/// Per-function compilation state
struct FnCompiler {
    code: Vec<u8>,
    locals: BTreeMap<String, u8>,
    next_local: u8,
    loop_starts: Vec<usize>,      // for continue
    loop_breaks: Vec<Vec<usize>>, // for break (patch targets)
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

    fn intern_string(&mut self, s: &str) -> u16 {
        if let Some(&idx) = self.string_map.get(s) {
            return idx as u16;
        }
        let idx = self.strings.len();
        self.strings.push(String::from(s));
        self.string_map.insert(String::from(s), idx);
        idx as u16
    }

    fn register_functions(&mut self, program: &Program) {
        for (i, item) in program.items.iter().enumerate() {
            if let Item::Function(f) = item {
                self.func_map.insert(f.name.clone(), i);
            }
        }
    }

    fn compile_program(&mut self, program: &Program) -> Result<Bytecode, String> {
        self.register_functions(program);

        for item in &program.items {
            match item {
                Item::Function(f) => self.compile_fn(f)?,
                Item::Struct(_) => {} // Structs don't produce bytecode (yet)
            }
        }

        let entry = self.func_map.get("main")
            .ok_or_else(|| String::from("no main() function found"))?;
        let entry = *entry;

        Ok(Bytecode {
            functions: self.functions.clone(),
            strings: self.strings.clone(),
            entry,
        })
    }

    fn compile_fn(&mut self, decl: &FnDecl) -> Result<(), String> {
        let mut fc = FnCompiler::new();

        // Register parameters as locals
        for (name, _ty) in &decl.params {
            fc.add_local(name);
        }

        // Compile body
        self.compile_block(&mut fc, &decl.body)?;

        // Ensure return at end
        if fc.code.last().copied() != Some(Op::Return as u8) {
            fc.emit_op(&mut self.strings, Op::PushI64);
            fc.emit_i64(0); // default return 0
            fc.emit_op(&mut self.strings, Op::Return);
        }

        let func = Function {
            name: decl.name.clone(),
            arity: decl.params.len() as u8,
            locals: fc.next_local,
            code: fc.code,
        };
        self.functions.push(func);
        Ok(())
    }

    fn compile_block(&mut self, fc: &mut FnCompiler, block: &Block) -> Result<(), String> {
        for stmt in &block.stmts {
            self.compile_stmt(fc, stmt)?;
        }
        Ok(())
    }

    fn compile_stmt(&mut self, fc: &mut FnCompiler, stmt: &Stmt) -> Result<(), String> {
        match stmt {
            Stmt::Let { name, init, .. } => {
                let slot = fc.add_local(name);
                if let Some(expr) = init {
                    self.compile_expr(fc, expr)?;
                    fc.emit_op(&mut self.strings, Op::StoreLocal);
                    fc.code.push(slot);
                }
            }
            Stmt::Assign { target, value } => {
                self.compile_expr(fc, value)?;
                self.compile_store(fc, target)?;
            }
            Stmt::OpAssign { op, target, value } => {
                self.compile_expr(fc, target)?;
                self.compile_expr(fc, value)?;
                // Emit the binary op (assume i64 for now)
                let vm_op = match op {
                    BinOp::Add => Op::AddI,
                    BinOp::Sub => Op::SubI,
                    BinOp::Mul => Op::MulI,
                    BinOp::Div => Op::DivI,
                    _ => Op::AddI,
                };
                fc.emit_op(&mut self.strings, vm_op);
                self.compile_store(fc, target)?;
            }
            Stmt::Expr(expr) => {
                self.compile_expr(fc, expr)?;
                fc.emit_op(&mut self.strings, Op::Pop); // discard result
            }
            Stmt::Return(val) => {
                if let Some(expr) = val {
                    self.compile_expr(fc, expr)?;
                } else {
                    fc.emit_op(&mut self.strings, Op::PushI64);
                    fc.emit_i64(0);
                }
                fc.emit_op(&mut self.strings, Op::Return);
            }
            Stmt::If { cond, then_block, else_block } => {
                self.compile_expr(fc, cond)?;
                let jump_else = fc.emit_jump(Op::JumpIfFalse);

                self.compile_block(fc, then_block)?;

                if let Some(else_blk) = else_block {
                    let jump_end = fc.emit_jump(Op::Jump);
                    fc.patch_jump(jump_else);
                    self.compile_block(fc, else_blk)?;
                    fc.patch_jump(jump_end);
                } else {
                    fc.patch_jump(jump_else);
                }
            }
            Stmt::While { cond, body } => {
                let loop_start = fc.code.len();
                fc.loop_starts.push(loop_start);
                fc.loop_breaks.push(Vec::new());

                self.compile_expr(fc, cond)?;
                let exit_jump = fc.emit_jump(Op::JumpIfFalse);

                self.compile_block(fc, body)?;
                fc.emit_jump_to(loop_start);

                fc.patch_jump(exit_jump);

                // Patch breaks
                let breaks = fc.loop_breaks.pop().unwrap_or_default();
                for b in breaks { fc.patch_jump(b); }
                fc.loop_starts.pop();
            }
            Stmt::For { var, iter, body } => {
                // Compile iterator (assume range for now)
                let slot = fc.add_local(var);
                // For Range { start, end }: init var=start, while var < end, var += 1
                if let Expr::Range { start, end } = iter {
                    self.compile_expr(fc, start)?;
                    fc.emit_op(&mut self.strings, Op::StoreLocal);
                    fc.code.push(slot);

                    // Compile end to a temporary
                    let end_slot = fc.add_local(&format!("__for_end_{}", slot));
                    self.compile_expr(fc, end)?;
                    fc.emit_op(&mut self.strings, Op::StoreLocal);
                    fc.code.push(end_slot);

                    let loop_start = fc.code.len();
                    fc.loop_starts.push(loop_start);
                    fc.loop_breaks.push(Vec::new());

                    // Condition: var < end
                    fc.emit_op(&mut self.strings, Op::LoadLocal);
                    fc.code.push(slot);
                    fc.emit_op(&mut self.strings, Op::LoadLocal);
                    fc.code.push(end_slot);
                    fc.emit_op(&mut self.strings, Op::LtI);

                    let exit_jump = fc.emit_jump(Op::JumpIfFalse);
                    self.compile_block(fc, body)?;

                    // Increment: var += 1
                    fc.emit_op(&mut self.strings, Op::LoadLocal);
                    fc.code.push(slot);
                    fc.emit_op(&mut self.strings, Op::PushI64);
                    fc.emit_i64(1);
                    fc.emit_op(&mut self.strings, Op::AddI);
                    fc.emit_op(&mut self.strings, Op::StoreLocal);
                    fc.code.push(slot);

                    fc.emit_jump_to(loop_start);
                    fc.patch_jump(exit_jump);

                    let breaks = fc.loop_breaks.pop().unwrap_or_default();
                    for b in breaks { fc.patch_jump(b); }
                    fc.loop_starts.pop();
                } else {
                    return Err(String::from("for loop requires a range expression"));
                }
            }
            Stmt::Loop(body) => {
                let loop_start = fc.code.len();
                fc.loop_starts.push(loop_start);
                fc.loop_breaks.push(Vec::new());

                self.compile_block(fc, body)?;
                fc.emit_jump_to(loop_start);

                let breaks = fc.loop_breaks.pop().unwrap_or_default();
                for b in breaks { fc.patch_jump(b); }
                fc.loop_starts.pop();
            }
            Stmt::Break => {
                let j = fc.emit_jump(Op::Jump);
                if let Some(breaks) = fc.loop_breaks.last_mut() {
                    breaks.push(j);
                }
            }
            Stmt::Continue => {
                if let Some(&start) = fc.loop_starts.last() {
                    fc.emit_jump_to(start);
                }
            }
        }
        Ok(())
    }

    fn compile_expr(&mut self, fc: &mut FnCompiler, expr: &Expr) -> Result<(), String> {
        match expr {
            Expr::IntLit(v) => {
                fc.emit_op(&mut self.strings, Op::PushI64);
                fc.emit_i64(*v);
            }
            Expr::FloatLit(v) => {
                fc.emit_op(&mut self.strings, Op::PushF64);
                fc.emit_f64(*v);
            }
            Expr::StringLit(s) => {
                let idx = self.intern_string(s);
                fc.emit_op(&mut self.strings, Op::PushStr);
                fc.emit_u16(idx);
            }
            Expr::BoolLit(b) => {
                fc.emit_op(&mut self.strings, Op::PushBool);
                fc.code.push(if *b { 1 } else { 0 });
            }
            Expr::Ident(name) => {
                if let Some(&slot) = fc.locals.get(name) {
                    fc.emit_op(&mut self.strings, Op::LoadLocal);
                    fc.code.push(slot);
                } else {
                    return Err(format!("undefined variable: {}", name));
                }
            }
            Expr::BinOp { op, left, right } => {
                self.compile_expr(fc, left)?;
                self.compile_expr(fc, right)?;
                let vm_op = match op {
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
                fc.emit_op(&mut self.strings, vm_op);
            }
            Expr::UnaryOp { op, expr } => {
                self.compile_expr(fc, expr)?;
                match op {
                    UnaryOp::Neg => fc.emit_op(&mut self.strings, Op::NegI),
                    UnaryOp::Not => fc.emit_op(&mut self.strings, Op::Not),
                }
            }
            Expr::Call { func, args } => {
                // Check if builtin
                if let Some(bid) = builtin_id(func) {
                    for arg in args {
                        self.compile_expr(fc, arg)?;
                    }
                    fc.emit_op(&mut self.strings, Op::CallBuiltin);
                    fc.code.push(bid);
                    fc.code.push(args.len() as u8);
                } else if let Some(&fidx) = self.func_map.get(func) {
                    // User function call
                    for arg in args {
                        self.compile_expr(fc, arg)?;
                    }
                    fc.emit_op(&mut self.strings, Op::Call);
                    fc.emit_u16(fidx as u16);
                    fc.code.push(args.len() as u8);
                } else {
                    return Err(format!("undefined function: {}", func));
                }
            }
            Expr::Index { array, index } => {
                self.compile_expr(fc, array)?;
                self.compile_expr(fc, index)?;
                fc.emit_op(&mut self.strings, Op::ArrayGet);
            }
            Expr::Array(elems) => {
                for elem in elems {
                    self.compile_expr(fc, elem)?;
                }
                fc.emit_op(&mut self.strings, Op::NewArray);
                fc.emit_u16(elems.len() as u16);
            }
            Expr::Range { start, end } => {
                // Ranges are handled specially in for loops
                // As a standalone expression, create an array
                self.compile_expr(fc, start)?;
                self.compile_expr(fc, end)?;
                // Store as two values (start, end) — handled at VM level
                fc.emit_op(&mut self.strings, Op::NewArray);
                fc.emit_u16(2);
            }
            Expr::Cast { expr, ty } => {
                self.compile_expr(fc, expr)?;
                match ty {
                    Type::F64 => fc.emit_op(&mut self.strings, Op::I64toF64),
                    Type::I64 => fc.emit_op(&mut self.strings, Op::F64toI64),
                    _ => {} // No-op for other casts
                }
            }
            Expr::Block(block) => {
                self.compile_block(fc, block)?;
            }
            Expr::Field { .. } => {
                return Err(String::from("struct field access not yet supported"));
            }
        }
        Ok(())
    }

    fn compile_store(&mut self, fc: &mut FnCompiler, target: &Expr) -> Result<(), String> {
        match target {
            Expr::Ident(name) => {
                if let Some(&slot) = fc.locals.get(name) {
                    fc.emit_op(&mut self.strings, Op::StoreLocal);
                    fc.code.push(slot);
                } else {
                    return Err(format!("undefined variable: {}", name));
                }
            }
            Expr::Index { array, index } => {
                // arr[i] = val  →  stack already has value on top
                // We need: array, index, value → ArraySet → store back
                // Value is already on stack from the caller.
                // We need to: load array, then index, then bring value to top
                // Reorder: push array, push index, then the value is already below
                // Actually: the value is on stack top. We need to get arr and idx under it.
                
                // Strategy: store value in a temp, load array, load index, load value, ArraySet, store array back
                if let Expr::Ident(arr_name) = array.as_ref() {
                    if let Some(&arr_slot) = fc.locals.get(arr_name) {
                        // temp slot for the value
                        let temp_slot = fc.next_local;
                        fc.next_local += 1;
                        // Store value to temp
                        fc.emit_op(&mut self.strings, Op::StoreLocal);
                        fc.code.push(temp_slot);
                        // Load array
                        fc.emit_op(&mut self.strings, Op::LoadLocal);
                        fc.code.push(arr_slot);
                        // Compile index expression
                        self.compile_expr(fc, index)?;
                        // Load value back from temp
                        fc.emit_op(&mut self.strings, Op::LoadLocal);
                        fc.code.push(temp_slot);
                        // ArraySet: pops array, index, value → pushes modified array
                        fc.emit_op(&mut self.strings, Op::ArraySet);
                        // Store modified array back
                        fc.emit_op(&mut self.strings, Op::StoreLocal);
                        fc.code.push(arr_slot);
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

    /// Emit a jump instruction, return the index to patch later
    fn emit_jump(&mut self, op: Op) -> usize {
        self.code.push(op as u8);
        let idx = self.code.len();
        self.code.push(0); // placeholder
        self.code.push(0);
        idx
    }

    /// Patch a jump to point to the current position
    fn patch_jump(&mut self, idx: usize) {
        let target = self.code.len() as u16;
        self.code[idx] = (target & 0xFF) as u8;
        self.code[idx + 1] = ((target >> 8) & 0xFF) as u8;
    }

    /// Emit a jump to a known target
    fn emit_jump_to(&mut self, target: usize) {
        self.code.push(Op::Jump as u8);
        self.code.push((target & 0xFF) as u8);
        self.code.push(((target >> 8) & 0xFF) as u8);
    }
}

/// Compile a program AST into bytecode
pub fn compile(program: &Program) -> Result<Bytecode, String> {
    let mut compiler = Compiler::new();
    compiler.compile_program(program)
}
