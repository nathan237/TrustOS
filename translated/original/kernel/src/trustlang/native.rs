//! TrustLang Native Compiler — AST to x86_64 machine code
//!
//! Compiles TrustLang AST directly to executable x86_64 machine code.
//! Uses a stack-based evaluation model (push/pop on the hardware stack).
//! Locals are stored in the stack frame via [rbp - offset].
//!
//! Builtins are dispatched via a callback table passed at execution time.
//! The native code calls builtins through an indirect call: the callback
//! table pointer lives in R15 (callee-saved, set once at entry).

use alloc::string::String;
use alloc::vec::Vec;
use alloc::collections::BTreeMap;
use alloc::format;

use super::parser::*;
use super::x86asm::{X86Asm, Reg, Cc, Label};

/// A compiled native program ready to execute
pub struct NativeProgram {
    /// The raw executable machine code
    pub code: Vec<u8>,
    /// Offset of main() entry point within code
    pub entry_offset: usize,
    /// String constant pool (referenced by index)
    pub strings: Vec<String>,
}

/// Builtin function callback type.
/// Args: builtin_id (u8), arg_count, pointer to i64 args array → returns i64.
/// This is the bridge between native TrustLang code and kernel builtins.
pub type BuiltinFn = fn(u8, usize, *const i64) -> i64;

/// Per-function compilation context
struct FnCtx {
    label: Label,
    locals: BTreeMap<String, i32>,  // name → rbp offset (negative)
    next_offset: i32,               // next available local slot (-8, -16, ...)
    loop_starts: Vec<Label>,
    loop_ends: Vec<Label>,
}

impl FnCtx {
    fn new(label: Label) -> Self {
        Self {
            label,
            locals: BTreeMap::new(),
            next_offset: -8,
            loop_starts: Vec::new(),
            loop_ends: Vec::new(),
        }
    }

    fn alloc_local(&mut self, name: &str) -> i32 {
        if let Some(&off) = self.locals.get(name) {
            return off;
        }
        let off = self.next_offset;
        self.locals.insert(String::from(name), off);
        self.next_offset -= 8;
        off
    }

    fn frame_size(&self) -> i32 {
        let raw = (-self.next_offset) + 8; // total bytes used
        // Align to 16 bytes (ABI requirement)
        (raw + 15) & !15
    }
}

/// Native compiler state
struct NativeCompiler {
    asm: X86Asm,
    func_labels: BTreeMap<String, Label>,
    strings: Vec<String>,
    string_map: BTreeMap<String, usize>,
    builtin_trampoline: Label,
}

impl NativeCompiler {
    fn new() -> Self {
        let mut asm = X86Asm::new();
        let trampoline = asm.new_label();
        Self {
            asm,
            func_labels: BTreeMap::new(),
            strings: Vec::new(),
            string_map: BTreeMap::new(),
            builtin_trampoline: trampoline,
        }
    }

    fn intern_string(&mut self, s: &str) -> usize {
        if let Some(&idx) = self.string_map.get(s) {
            return idx;
        }
        let idx = self.strings.len();
        self.strings.push(String::from(s));
        self.string_map.insert(String::from(s), idx);
        idx
    }

    /// Map TrustLang builtin names to IDs matching vm.rs constants
    fn builtin_id(name: &str) -> Option<u8> {
        match name {
            "print"        => Some(0),
            "println"      => Some(1),
            "len"          => Some(2),
            "push"         => Some(3),
            "to_string"    => Some(4),
            "to_int"       => Some(5),
            "sqrt"         => Some(6),
            "abs"          => Some(7),
            "pixel"        => Some(8),
            "clear_screen" => Some(9),
            "fill_rect"    => Some(10),
            "draw_line"    => Some(11),
            "draw_circle"  => Some(12),
            "screen_w"     => Some(13),
            "screen_h"     => Some(14),
            "flush"        => Some(15),
            "draw_text"    => Some(16),
            "sleep"        => Some(17),
            "to_float"     => Some(18),
            "read_line"    => Some(19),
            _ => None,
        }
    }

    fn compile_program(&mut self, program: &Program) -> Result<NativeProgram, String> {
        // Phase 1: register all function labels
        for item in &program.items {
            if let Item::Function(f) = item {
                let label = self.asm.new_label();
                self.func_labels.insert(f.name.clone(), label);
            }
        }

        // Phase 2: emit builtin trampoline
        // The trampoline calls the BuiltinFn via R15.
        // Convention: rdi = builtin_id, rsi = argc, rdx = argv pointer
        self.asm.bind_label(self.builtin_trampoline);
        // R15 holds the callback function pointer
        self.asm.call_r(Reg::R15);
        self.asm.ret();

        // Phase 3: compile each function
        for item in &program.items {
            if let Item::Function(f) = item {
                self.compile_fn(f)?;
            }
        }

        // Resolve all jump/call patches
        self.asm.resolve_patches().map_err(|e| String::from(e))?;

        let entry_label = self.func_labels.get("main")
            .ok_or_else(|| String::from("no main() function found"))?;
        let entry_offset = self.asm.labels[entry_label.0]
            .ok_or_else(|| String::from("main() label unresolved"))?;

        Ok(NativeProgram {
            code: self.asm.code.clone(),
            strings: self.strings.clone(),
            entry_offset,
        })
    }

    fn compile_fn(&mut self, decl: &FnDecl) -> Result<(), String> {
        let label = *self.func_labels.get(&decl.name)
            .ok_or_else(|| format!("function '{}' not registered", decl.name))?;

        let mut ctx = FnCtx::new(label);

        // Register parameters as locals (passed via stack, pushed by caller)
        // Caller pushes args right-to-left, so first arg is at [rbp+16], etc.
        // We copy them to local slots in prologue.
        for (i, (name, _ty)) in decl.params.iter().enumerate() {
            let off = ctx.alloc_local(name);
            // Will be filled during prologue copy below
            let _ = (i, off);
        }

        // Pre-scan body for additional locals to compute frame size
        self.prescan_block_locals(&mut ctx, &decl.body);
        let frame_size = ctx.frame_size();

        // Emit label and prologue
        self.asm.bind_label(label);
        self.asm.prologue(frame_size);

        // Copy parameters from caller's stack into local slots
        // Caller pushes args right-to-left, so at [rbp+16] = arg0, [rbp+24] = arg1, etc.
        for (i, (name, _ty)) in decl.params.iter().enumerate() {
            let src_offset = 16 + (i as i32) * 8;
            let dst_offset = *ctx.locals.get(name).unwrap();
            self.asm.mov_r_rbp_offset(Reg::Rax, src_offset);
            self.asm.mov_rbp_offset_r(dst_offset, Reg::Rax);
        }

        // Compile body
        self.compile_block(&mut ctx, &decl.body)?;

        // Default return 0 if no explicit return
        self.asm.mov_r_imm32(Reg::Rax, 0);
        self.asm.epilogue();

        Ok(())
    }

    /// Pre-scan a block for let bindings to allocate local slots
    fn prescan_block_locals(&self, ctx: &mut FnCtx, block: &Block) {
        for stmt in &block.stmts {
            match stmt {
                Stmt::Let { name, .. } => { ctx.alloc_local(name); }
                Stmt::If { then_block, else_block, .. } => {
                    self.prescan_block_locals(ctx, then_block);
                    if let Some(eb) = else_block { self.prescan_block_locals(ctx, eb); }
                }
                Stmt::While { body, .. } | Stmt::Loop(body) => {
                    self.prescan_block_locals(ctx, body);
                }
                Stmt::For { var, body, .. } => {
                    ctx.alloc_local(var);
                    ctx.alloc_local(&format!("__for_end_{}", var));
                    self.prescan_block_locals(ctx, body);
                }
                _ => {}
            }
        }
    }

    fn compile_block(&mut self, ctx: &mut FnCtx, block: &Block) -> Result<(), String> {
        for stmt in &block.stmts {
            self.compile_stmt(ctx, stmt)?;
        }
        Ok(())
    }

    fn compile_stmt(&mut self, ctx: &mut FnCtx, stmt: &Stmt) -> Result<(), String> {
        match stmt {
            Stmt::Let { name, init, .. } => {
                let off = *ctx.locals.get(name).unwrap();
                if let Some(expr) = init {
                    self.compile_expr(ctx, expr)?;
                    self.asm.pop_r(Reg::Rax);
                    self.asm.mov_rbp_offset_r(off, Reg::Rax);
                }
            }
            Stmt::Assign { target, value } => {
                self.compile_expr(ctx, value)?;
                self.compile_store(ctx, target)?;
            }
            Stmt::OpAssign { op, target, value } => {
                self.compile_expr(ctx, target)?;
                self.compile_expr(ctx, value)?;
                self.emit_binop(*op)?;
                self.compile_store(ctx, &target.clone())?;
            }
            Stmt::Expr(expr) => {
                self.compile_expr(ctx, expr)?;
                self.asm.pop_r(Reg::Rax); // discard result
            }
            Stmt::Return(val) => {
                if let Some(expr) = val {
                    self.compile_expr(ctx, expr)?;
                    self.asm.pop_r(Reg::Rax);
                } else {
                    self.asm.mov_r_imm32(Reg::Rax, 0);
                }
                self.asm.epilogue();
            }
            Stmt::If { cond, then_block, else_block } => {
                self.compile_expr(ctx, cond)?;
                self.asm.pop_r(Reg::Rax);
                self.asm.test_r_r(Reg::Rax, Reg::Rax);
                let else_label = self.asm.new_label();
                self.asm.jcc_label(Cc::E, else_label); // jump if zero (false)
                self.compile_block(ctx, then_block)?;
                if let Some(eb) = else_block {
                    let end_label = self.asm.new_label();
                    self.asm.jmp_label(end_label);
                    self.asm.bind_label(else_label);
                    self.compile_block(ctx, eb)?;
                    self.asm.bind_label(end_label);
                } else {
                    self.asm.bind_label(else_label);
                }
            }
            Stmt::While { cond, body } => {
                let top = self.asm.new_label();
                let end = self.asm.new_label();
                ctx.loop_starts.push(top);
                ctx.loop_ends.push(end);

                self.asm.bind_label(top);
                self.compile_expr(ctx, cond)?;
                self.asm.pop_r(Reg::Rax);
                self.asm.test_r_r(Reg::Rax, Reg::Rax);
                self.asm.jcc_label(Cc::E, end);
                self.compile_block(ctx, body)?;
                self.asm.jmp_label(top);
                self.asm.bind_label(end);

                ctx.loop_starts.pop();
                ctx.loop_ends.pop();
            }
            Stmt::For { var, iter, body } => {
                if let Expr::Range { start, end } = iter {
                    let var_off = *ctx.locals.get(var).unwrap();
                    let end_name = format!("__for_end_{}", var);
                    let end_off = *ctx.locals.get(&end_name).unwrap();

                    self.compile_expr(ctx, start)?;
                    self.asm.pop_r(Reg::Rax);
                    self.asm.mov_rbp_offset_r(var_off, Reg::Rax);

                    self.compile_expr(ctx, end)?;
                    self.asm.pop_r(Reg::Rax);
                    self.asm.mov_rbp_offset_r(end_off, Reg::Rax);

                    let top = self.asm.new_label();
                    let end_lbl = self.asm.new_label();
                    ctx.loop_starts.push(top);
                    ctx.loop_ends.push(end_lbl);

                    self.asm.bind_label(top);
                    // var < end?
                    self.asm.mov_r_rbp_offset(Reg::Rax, var_off);
                    self.asm.mov_r_rbp_offset(Reg::Rcx, end_off);
                    self.asm.cmp_r_r(Reg::Rax, Reg::Rcx);
                    self.asm.jcc_label(Cc::Ge, end_lbl);

                    self.compile_block(ctx, body)?;

                    // var += 1
                    self.asm.mov_r_rbp_offset(Reg::Rax, var_off);
                    self.asm.add_r_imm32(Reg::Rax, 1);
                    self.asm.mov_rbp_offset_r(var_off, Reg::Rax);
                    self.asm.jmp_label(top);
                    self.asm.bind_label(end_lbl);

                    ctx.loop_starts.pop();
                    ctx.loop_ends.pop();
                } else {
                    return Err(String::from("for loop requires a range expression"));
                }
            }
            Stmt::Loop(body) => {
                let top = self.asm.new_label();
                let end = self.asm.new_label();
                ctx.loop_starts.push(top);
                ctx.loop_ends.push(end);
                self.asm.bind_label(top);
                self.compile_block(ctx, body)?;
                self.asm.jmp_label(top);
                self.asm.bind_label(end);
                ctx.loop_starts.pop();
                ctx.loop_ends.pop();
            }
            Stmt::Break => {
                if let Some(&end) = ctx.loop_ends.last() {
                    self.asm.jmp_label(end);
                }
            }
            Stmt::Continue => {
                if let Some(&top) = ctx.loop_starts.last() {
                    self.asm.jmp_label(top);
                }
            }
        }
        Ok(())
    }

    fn compile_expr(&mut self, ctx: &mut FnCtx, expr: &Expr) -> Result<(), String> {
        match expr {
            Expr::IntLit(v) => {
                self.asm.mov_r_imm64(Reg::Rax, *v);
                self.asm.push_r(Reg::Rax);
            }
            Expr::FloatLit(v) => {
                // Store f64 as raw i64 bits on the native stack
                let bits = v.to_bits() as i64;
                self.asm.mov_r_imm64(Reg::Rax, bits);
                self.asm.push_r(Reg::Rax);
            }
            Expr::BoolLit(b) => {
                self.asm.mov_r_imm32(Reg::Rax, if *b { 1 } else { 0 });
                self.asm.push_r(Reg::Rax);
            }
            Expr::StringLit(s) => {
                // Push the string pool index as an i64
                let idx = self.intern_string(s);
                self.asm.mov_r_imm64(Reg::Rax, idx as i64);
                self.asm.push_r(Reg::Rax);
            }
            Expr::Ident(name) => {
                if let Some(&off) = ctx.locals.get(name) {
                    self.asm.mov_r_rbp_offset(Reg::Rax, off);
                    self.asm.push_r(Reg::Rax);
                } else {
                    return Err(format!("undefined variable: {}", name));
                }
            }
            Expr::BinOp { op, left, right } => {
                self.compile_expr(ctx, left)?;
                self.compile_expr(ctx, right)?;
                self.emit_binop(*op)?;
            }
            Expr::UnaryOp { op, expr } => {
                self.compile_expr(ctx, expr)?;
                self.asm.pop_r(Reg::Rax);
                match op {
                    UnaryOp::Neg => self.asm.neg_r(Reg::Rax),
                    UnaryOp::Not => {
                        self.asm.test_r_r(Reg::Rax, Reg::Rax);
                        self.asm.setcc(Cc::E, Reg::Rax);
                        self.asm.movzx_r_r8(Reg::Rax, Reg::Rax);
                    }
                }
                self.asm.push_r(Reg::Rax);
            }
            Expr::Call { func, args } => {
                if let Some(bid) = Self::builtin_id(func) {
                    // Push args onto native stack for the builtin to read
                    // Convention: we build a small array on the stack,
                    // then call the trampoline with (id, argc, argv)
                    let argc = args.len();
                    // Allocate space on stack for args array
                    if argc > 0 {
                        self.asm.sub_r_imm32(Reg::Rsp, (argc as i32) * 8);
                    }
                    // Evaluate and store each arg
                    for (i, arg) in args.iter().enumerate() {
                        self.compile_expr(ctx, arg)?;
                        self.asm.pop_r(Reg::Rax);
                        // Store at [rsp + i*8]
                        // mov [rsp + i*8], rax
                        let off = (i as i32) * 8;
                        // Use a simple encoding: mov [rsp+off], rax
                        self.emit_mov_rsp_offset_r(off, Reg::Rax);
                    }
                    // Set up call: rdi=builtin_id, rsi=argc, rdx=argv (rsp)
                    self.asm.mov_r_imm32(Reg::Rdi, bid as i32);
                    self.asm.mov_r_imm32(Reg::Rsi, argc as i32);
                    self.asm.mov_r_r(Reg::Rdx, Reg::Rsp);
                    // Call trampoline (which calls R15)
                    self.asm.call_label(self.builtin_trampoline);
                    // Clean up args from stack
                    if argc > 0 {
                        self.asm.add_r_imm32(Reg::Rsp, (argc as i32) * 8);
                    }
                    // Push result
                    self.asm.push_r(Reg::Rax);
                } else if let Some(&func_label) = self.func_labels.get(func) {
                    // User function call: push args right-to-left, call, clean stack
                    for arg in args.iter().rev() {
                        self.compile_expr(ctx, arg)?;
                        // value already on stack from compile_expr
                    }
                    self.asm.call_label(func_label);
                    // Clean args from stack
                    let argc = args.len();
                    if argc > 0 {
                        self.asm.add_r_imm32(Reg::Rsp, (argc as i32) * 8);
                    }
                    // Push return value
                    self.asm.push_r(Reg::Rax);
                } else {
                    return Err(format!("undefined function: {}", func));
                }
            }
            Expr::Cast { expr, ty } => {
                self.compile_expr(ctx, expr)?;
                match ty {
                    Type::F64 => {
                        // i64 → f64: convert using hardware
                        // pop i64, cvtsi2sd, push f64 bits
                        // Simplified: just pass through (native mode treats as i64)
                    }
                    Type::I64 => {
                        // f64 → i64: just pass through for now
                    }
                    _ => {}
                }
            }
            Expr::Range { start, end } => {
                // As a standalone expression, just eval start (for compatibility)
                self.compile_expr(ctx, start)?;
                // Pop and discard end
                self.compile_expr(ctx, end)?;
                self.asm.pop_r(Reg::Rax); // discard end
            }
            Expr::Index { .. } | Expr::Array(_) | Expr::Field { .. } | Expr::Block(_) => {
                // Arrays and field access: fallback to 0 in native mode
                // These require heap allocation which the native backend doesn't support yet
                self.asm.mov_r_imm32(Reg::Rax, 0);
                self.asm.push_r(Reg::Rax);
            }
        }
        Ok(())
    }

    /// Emit a binary operation: pops two values, pushes result
    fn emit_binop(&mut self, op: BinOp) -> Result<(), String> {
        self.asm.pop_r(Reg::Rcx); // right
        self.asm.pop_r(Reg::Rax); // left
        match op {
            BinOp::Add => self.asm.add_r_r(Reg::Rax, Reg::Rcx),
            BinOp::Sub => self.asm.sub_r_r(Reg::Rax, Reg::Rcx),
            BinOp::Mul => self.asm.imul_r_r(Reg::Rax, Reg::Rcx),
            BinOp::Div => {
                self.asm.cqo();
                self.asm.idiv_r(Reg::Rcx);
                // quotient already in rax
            }
            BinOp::Mod => {
                self.asm.cqo();
                self.asm.idiv_r(Reg::Rcx);
                self.asm.mov_r_r(Reg::Rax, Reg::Rdx); // remainder in rdx
            }
            BinOp::Eq => {
                self.asm.cmp_r_r(Reg::Rax, Reg::Rcx);
                self.asm.setcc(Cc::E, Reg::Rax);
                self.asm.movzx_r_r8(Reg::Rax, Reg::Rax);
            }
            BinOp::NotEq => {
                self.asm.cmp_r_r(Reg::Rax, Reg::Rcx);
                self.asm.setcc(Cc::Ne, Reg::Rax);
                self.asm.movzx_r_r8(Reg::Rax, Reg::Rax);
            }
            BinOp::Lt => {
                self.asm.cmp_r_r(Reg::Rax, Reg::Rcx);
                self.asm.setcc(Cc::L, Reg::Rax);
                self.asm.movzx_r_r8(Reg::Rax, Reg::Rax);
            }
            BinOp::Gt => {
                self.asm.cmp_r_r(Reg::Rax, Reg::Rcx);
                self.asm.setcc(Cc::G, Reg::Rax);
                self.asm.movzx_r_r8(Reg::Rax, Reg::Rax);
            }
            BinOp::LtEq => {
                self.asm.cmp_r_r(Reg::Rax, Reg::Rcx);
                self.asm.setcc(Cc::Le, Reg::Rax);
                self.asm.movzx_r_r8(Reg::Rax, Reg::Rax);
            }
            BinOp::GtEq => {
                self.asm.cmp_r_r(Reg::Rax, Reg::Rcx);
                self.asm.setcc(Cc::Ge, Reg::Rax);
                self.asm.movzx_r_r8(Reg::Rax, Reg::Rax);
            }
            BinOp::And => self.asm.and_r_r(Reg::Rax, Reg::Rcx),
            BinOp::Or => self.asm.or_r_r(Reg::Rax, Reg::Rcx),
            BinOp::BitAnd => self.asm.and_r_r(Reg::Rax, Reg::Rcx),
            BinOp::BitOr => self.asm.or_r_r(Reg::Rax, Reg::Rcx),
            BinOp::BitXor => self.asm.xor_r_r(Reg::Rax, Reg::Rcx),
            BinOp::Shl => {
                // cl already has shift count (rcx low byte)
                self.asm.shl_r_cl(Reg::Rax);
            }
            BinOp::Shr => {
                self.asm.sar_r_cl(Reg::Rax);
            }
        }
        self.asm.push_r(Reg::Rax);
        Ok(())
    }

    /// Store the value on top of the eval stack to a target (variable)
    fn compile_store(&mut self, ctx: &mut FnCtx, target: &Expr) -> Result<(), String> {
        match target {
            Expr::Ident(name) => {
                if let Some(&off) = ctx.locals.get(name) {
                    self.asm.pop_r(Reg::Rax);
                    self.asm.mov_rbp_offset_r(off, Reg::Rax);
                } else {
                    return Err(format!("undefined variable: {}", name));
                }
            }
            _ => return Err(String::from("native: unsupported store target")),
        }
        Ok(())
    }

    /// Emit mov [rsp + offset], reg
    fn emit_mov_rsp_offset_r(&mut self, offset: i32, src: Reg) {
        // REX.W + mov [rsp + disp], src
        // Need SIB byte when base is rsp
        let mut rex: u8 = 0x48;
        if src.needs_rex() { rex |= 0x04; }
        self.asm.code.push(rex);
        self.asm.code.push(0x89);
        if offset == 0 {
            self.asm.code.push(X86Asm::modrm(0b00, src.lo3(), 0x04)); // SIB follows
            self.asm.code.push(0x24); // SIB: base=rsp, index=none
        } else if offset >= -128 && offset <= 127 {
            self.asm.code.push(X86Asm::modrm(0b01, src.lo3(), 0x04));
            self.asm.code.push(0x24);
            self.asm.code.push(offset as u8);
        } else {
            self.asm.code.push(X86Asm::modrm(0b10, src.lo3(), 0x04));
            self.asm.code.push(0x24);
            self.asm.code.extend_from_slice(&offset.to_le_bytes());
        }
    }
}

/// Compile TrustLang source to native x86_64 machine code
pub fn compile_native(source: &str) -> Result<NativeProgram, String> {
    let tokens = super::lexer::tokenize(source)?;
    let ast = super::parser::parse(&tokens)?;
    let mut compiler = NativeCompiler::new();
    compiler.compile_program(&ast)
}

/// Execute a NativeProgram.
/// SAFETY: This allocates executable memory and jumps to it.
/// The builtin_callback bridges native code back to kernel builtins.
pub unsafe fn execute_native(
    program: &NativeProgram,
    builtin_callback: BuiltinFn,
) -> Result<i64, String> {
    let code = &program.code;
    if code.is_empty() {
        return Err(String::from("empty native program"));
    }

    // Allocate executable page(s) from kernel heap
    let exec_mem = alloc_executable_pages(code.len())
        .ok_or_else(|| String::from("failed to allocate executable memory"))?;

    // Copy machine code
    core::ptr::copy_nonoverlapping(code.as_ptr(), exec_mem, code.len());

    // Entry point
    let entry: *const u8 = exec_mem.add(program.entry_offset);

    // Call convention:
    // R15 = builtin callback pointer (callee-saved)
    // We use inline asm to set R15 and call the entry point
    let result: i64;
    let cb_ptr = builtin_callback as usize;
    let entry_ptr = entry as usize;

    core::arch::asm!(
        "mov r15, {cb}",
        "call {entry}",
        cb = in(reg) cb_ptr,
        entry = in(reg) entry_ptr,
        out("rax") result,
        // Clobbers (everything the called code might touch)
        out("rcx") _, out("rdx") _, out("rsi") _, out("rdi") _,
        out("r8") _, out("r9") _, out("r10") _, out("r11") _,
        clobber_abi("C"),
    );

    // Free executable pages
    free_executable_pages(exec_mem, code.len());

    Ok(result)
}

// ─── Executable memory allocation ───────────────────────────────────────

/// Allocate page-aligned executable memory from the kernel heap.
/// In kernel space (Ring 0), all memory is already executable, so we just
/// need page-aligned allocation.
fn alloc_executable_pages(size: usize) -> Option<*mut u8> {
    use alloc::alloc::{alloc_zeroed, Layout};
    let alloc_size = (size + 4095) & !4095; // round up to page
    let layout = Layout::from_size_align(alloc_size, 4096).ok()?;
    let ptr = unsafe { alloc_zeroed(layout) };
    if ptr.is_null() { None } else { Some(ptr) }
}

/// Free executable pages back to the kernel heap.
fn free_executable_pages(ptr: *mut u8, size: usize) {
    use alloc::alloc::{dealloc, Layout};
    let alloc_size = (size + 4095) & !4095;
    if let Ok(layout) = Layout::from_size_align(alloc_size, 4096) {
        unsafe { dealloc(ptr, layout); }
    }
}
