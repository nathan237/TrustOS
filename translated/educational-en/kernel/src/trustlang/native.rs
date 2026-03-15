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
pub // Type alias — gives an existing type a new name for clarity.
type BuiltinFn = fn(u8, usize, *// Compile-time constant — evaluated at compilation, zero runtime cost.
const i64) -> i64;

/// Per-function compilation context
struct FnCtx {
    label: Label,
    locals: BTreeMap<String, i32>,  // name → rbp offset (negative)
    next_offset: i32,               // next available local slot (-8, -16, ...)
    loop_starts: Vec<Label>,
    loop_ends: Vec<Label>,
}

// Implementation block — defines methods for the type above.
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

    fn allocator_local(&mut self, name: &str) -> i32 {
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

// Implementation block — defines methods for the type above.
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
        if let Some(&index) = self.string_map.get(s) {
            return index;
        }
        let index = self.strings.len();
        self.strings.push(String::from(s));
        self.string_map.insert(String::from(s), index);
        index
    }

    /// Map TrustLang builtin names to IDs matching vm.rs constants
    fn builtin_id(name: &str) -> Option<u8> {
                // Pattern matching — Rust's exhaustive branching construct.
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
        self.asm.return_value();

        // Phase 3: compile each function
        for item in &program.items {
            if let Item::Function(f) = item {
                self.compile_fn(f)?;
            }
        }

        // Resolve all jump/call patches
        self.asm.resolve_patches().map_error(|e| String::from(e))?;

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

        let mut context = FnCtx::new(label);

        // Register parameters as locals (passed via stack, pushed by caller)
        // Caller pushes args right-to-left, so first arg is at [rbp+16], etc.
        // We copy them to local slots in prologue.
        for (i, (name, _ty)) in decl.params.iter().enumerate() {
            let off = context.allocator_local(name);
            // Will be filled during prologue copy below
            let _ = (i, off);
        }

        // Pre-scan body for additional locals to compute frame size
        self.prescan_block_locals(&mut context, &decl.body);
        let frame_size = context.frame_size();

        // Emit label and prologue
        self.asm.bind_label(label);
        self.asm.prologue(frame_size);

        // Copy parameters from caller's stack into local slots
        // Caller pushes args right-to-left, so at [rbp+16] = arg0, [rbp+24] = arg1, etc.
        for (i, (name, _ty)) in decl.params.iter().enumerate() {
            let source_offset = 16 + (i as i32) * 8;
            let destination_offset = *context.locals.get(name).unwrap();
            self.asm.mov_r_rbp_offset(Reg::Rax, source_offset);
            self.asm.mov_rbp_offset_r(destination_offset, Reg::Rax);
        }

        // Compile body
        self.compile_block(&mut context, &decl.body)?;

        // Default return 0 if no explicit return
        self.asm.mov_r_imm32(Reg::Rax, 0);
        self.asm.epilogue();

        Ok(())
    }

    /// Pre-scan a block for let bindings to allocate local slots
    fn prescan_block_locals(&self, context: &mut FnCtx, block: &Block) {
        for stmt in &block.stmts {
                        // Pattern matching — Rust's exhaustive branching construct.
match stmt {
                Stmt::Let { name, .. } => { context.allocator_local(name); }
                Stmt::If { then_block, else_block, .. } => {
                    self.prescan_block_locals(context, then_block);
                    if let Some(eb) = else_block { self.prescan_block_locals(context, eb); }
                }
                Stmt::While { body, .. } | Stmt::Loop(body) => {
                    self.prescan_block_locals(context, body);
                }
                Stmt::For { var, body, .. } => {
                    context.allocator_local(var);
                    context.allocator_local(&format!("__for_end_{}", var));
                    self.prescan_block_locals(context, body);
                }
                _ => {}
            }
        }
    }

    fn compile_block(&mut self, context: &mut FnCtx, block: &Block) -> Result<(), String> {
        for stmt in &block.stmts {
            self.compile_stmt(context, stmt)?;
        }
        Ok(())
    }

    fn compile_stmt(&mut self, context: &mut FnCtx, stmt: &Stmt) -> Result<(), String> {
                // Pattern matching — Rust's exhaustive branching construct.
match stmt {
            Stmt::Let { name, init, .. } => {
                let off = *context.locals.get(name).unwrap();
                if let Some(expr) = init {
                    self.compile_expr(context, expr)?;
                    self.asm.pop_r(Reg::Rax);
                    self.asm.mov_rbp_offset_r(off, Reg::Rax);
                }
            }
            Stmt::Assign { target, value } => {
                self.compile_expr(context, value)?;
                self.compile_store(context, target)?;
            }
            Stmt::OperationAssign { op, target, value } => {
                self.compile_expr(context, target)?;
                self.compile_expr(context, value)?;
                self.emit_binop(*op)?;
                self.compile_store(context, &target.clone())?;
            }
            Stmt::Expr(expr) => {
                self.compile_expr(context, expr)?;
                self.asm.pop_r(Reg::Rax); // discard result
            }
            Stmt::Return(value) => {
                if let Some(expr) = value {
                    self.compile_expr(context, expr)?;
                    self.asm.pop_r(Reg::Rax);
                } else {
                    self.asm.mov_r_imm32(Reg::Rax, 0);
                }
                self.asm.epilogue();
            }
            Stmt::If { condition, then_block, else_block } => {
                self.compile_expr(context, condition)?;
                self.asm.pop_r(Reg::Rax);
                self.asm.test_r_r(Reg::Rax, Reg::Rax);
                let else_label = self.asm.new_label();
                self.asm.jcc_label(Cc::E, else_label); // jump if zero (false)
                self.compile_block(context, then_block)?;
                if let Some(eb) = else_block {
                    let end_label = self.asm.new_label();
                    self.asm.jmp_label(end_label);
                    self.asm.bind_label(else_label);
                    self.compile_block(context, eb)?;
                    self.asm.bind_label(end_label);
                } else {
                    self.asm.bind_label(else_label);
                }
            }
            Stmt::While { condition, body } => {
                let top = self.asm.new_label();
                let end = self.asm.new_label();
                context.loop_starts.push(top);
                context.loop_ends.push(end);

                self.asm.bind_label(top);
                self.compile_expr(context, condition)?;
                self.asm.pop_r(Reg::Rax);
                self.asm.test_r_r(Reg::Rax, Reg::Rax);
                self.asm.jcc_label(Cc::E, end);
                self.compile_block(context, body)?;
                self.asm.jmp_label(top);
                self.asm.bind_label(end);

                context.loop_starts.pop();
                context.loop_ends.pop();
            }
            Stmt::For { var, iter, body } => {
                if let Expr::Range { start, end } = iter {
                    let var_off = *context.locals.get(var).unwrap();
                    let end_name = format!("__for_end_{}", var);
                    let end_off = *context.locals.get(&end_name).unwrap();

                    self.compile_expr(context, start)?;
                    self.asm.pop_r(Reg::Rax);
                    self.asm.mov_rbp_offset_r(var_off, Reg::Rax);

                    self.compile_expr(context, end)?;
                    self.asm.pop_r(Reg::Rax);
                    self.asm.mov_rbp_offset_r(end_off, Reg::Rax);

                    let top = self.asm.new_label();
                    let end_label = self.asm.new_label();
                    context.loop_starts.push(top);
                    context.loop_ends.push(end_label);

                    self.asm.bind_label(top);
                    // var < end?
                    self.asm.mov_r_rbp_offset(Reg::Rax, var_off);
                    self.asm.mov_r_rbp_offset(Reg::Rcx, end_off);
                    self.asm.cmp_r_r(Reg::Rax, Reg::Rcx);
                    self.asm.jcc_label(Cc::Ge, end_label);

                    self.compile_block(context, body)?;

                    // var += 1
                    self.asm.mov_r_rbp_offset(Reg::Rax, var_off);
                    self.asm.add_r_imm32(Reg::Rax, 1);
                    self.asm.mov_rbp_offset_r(var_off, Reg::Rax);
                    self.asm.jmp_label(top);
                    self.asm.bind_label(end_label);

                    context.loop_starts.pop();
                    context.loop_ends.pop();
                } else {
                    return Err(String::from("for loop requires a range expression"));
                }
            }
            Stmt::Loop(body) => {
                let top = self.asm.new_label();
                let end = self.asm.new_label();
                context.loop_starts.push(top);
                context.loop_ends.push(end);
                self.asm.bind_label(top);
                self.compile_block(context, body)?;
                self.asm.jmp_label(top);
                self.asm.bind_label(end);
                context.loop_starts.pop();
                context.loop_ends.pop();
            }
            Stmt::Break => {
                if let Some(&end) = context.loop_ends.last() {
                    self.asm.jmp_label(end);
                }
            }
            Stmt::Continue => {
                if let Some(&top) = context.loop_starts.last() {
                    self.asm.jmp_label(top);
                }
            }
        }
        Ok(())
    }

    fn compile_expr(&mut self, context: &mut FnCtx, expr: &Expr) -> Result<(), String> {
                // Pattern matching — Rust's exhaustive branching construct.
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
                let index = self.intern_string(s);
                self.asm.mov_r_imm64(Reg::Rax, index as i64);
                self.asm.push_r(Reg::Rax);
            }
            Expr::Ident(name) => {
                if let Some(&off) = context.locals.get(name) {
                    self.asm.mov_r_rbp_offset(Reg::Rax, off);
                    self.asm.push_r(Reg::Rax);
                } else {
                    return Err(format!("undefined variable: {}", name));
                }
            }
            Expr::BinOp { op, left, right } => {
                self.compile_expr(context, left)?;
                self.compile_expr(context, right)?;
                self.emit_binop(*op)?;
            }
            Expr::UnaryOp { op, expr } => {
                self.compile_expr(context, expr)?;
                self.asm.pop_r(Reg::Rax);
                                // Pattern matching — Rust's exhaustive branching construct.
match op {
                    UnaryOp::Neg => self.asm.negative_r(Reg::Rax),
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
                    for (i, argument) in args.iter().enumerate() {
                        self.compile_expr(context, argument)?;
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
                    for argument in args.iter().rev() {
                        self.compile_expr(context, argument)?;
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
                self.compile_expr(context, expr)?;
                                // Pattern matching — Rust's exhaustive branching construct.
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
                self.compile_expr(context, start)?;
                // Pop and discard end
                self.compile_expr(context, end)?;
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
    fn compile_store(&mut self, context: &mut FnCtx, target: &Expr) -> Result<(), String> {
                // Pattern matching — Rust's exhaustive branching construct.
match target {
            Expr::Ident(name) => {
                if let Some(&off) = context.locals.get(name) {
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
    fn emit_mov_rsp_offset_r(&mut self, offset: i32, source: Reg) {
        // REX.W + mov [rsp + disp], src
        // Need SIB byte when base is rsp
        let mut rex: u8 = 0x48;
        if source.needs_rex() { rex |= 0x04; }
        self.asm.code.push(rex);
        self.asm.code.push(0x89);
        if offset == 0 {
            self.asm.code.push(X86Asm::modrm(0b00, source.lo3(), 0x04)); // SIB follows
            self.asm.code.push(0x24); // SIB: base=rsp, index=none
        } else if offset >= -128 && offset <= 127 {
            self.asm.code.push(X86Asm::modrm(0b01, source.lo3(), 0x04));
            self.asm.code.push(0x24);
            self.asm.code.push(offset as u8);
        } else {
            self.asm.code.push(X86Asm::modrm(0b10, source.lo3(), 0x04));
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
pub // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe fn execute_native(
    program: &NativeProgram,
    builtin_callback: BuiltinFn,
) -> Result<i64, String> {
    let code = &program.code;
    if code.is_empty() {
        return Err(String::from("empty native program"));
    }

    // Allocate executable page(s) from kernel heap
    let execute_memory = allocator_executable_pages(code.len())
        .ok_or_else(|| String::from("failed to allocate executable memory"))?;

    // Copy machine code
    core::ptr::copy_nonoverlapping(code.as_pointer(), execute_memory, code.len());

    // Entry point
    let entry: *// Compile-time constant — evaluated at compilation, zero runtime cost.
const u8 = execute_memory.add(program.entry_offset);

    // Call convention:
    // R15 = builtin callback pointer (callee-saved)
    // We use inline asm to set R15 and call the entry point
    let result: i64;
    let callback_pointer = builtin_callback as usize;
    let entry_pointer = entry as usize;

    core::arch::asm!(
        "mov r15, {cb}",
        "call {entry}",
        callback = in(reg) callback_pointer,
        entry = in(reg) entry_pointer,
        out("rax") result,
        // Clobbers (everything the called code might touch)
        out("rcx") _, out("rdx") _, out("rsi") _, out("rdi") _,
        out("r8") _, out("r9") _, out("r10") _, out("r11") _,
        clobber_abi("C"),
    );

    // Free executable pages
    free_executable_pages(execute_memory, code.len());

    Ok(result)
}

// ─── Executable memory allocation ───────────────────────────────────────

/// Allocate page-aligned executable memory from the kernel heap.
/// In kernel space (Ring 0), all memory is already executable, so we just
/// need page-aligned allocation.
fn allocator_executable_pages(size: usize) -> Option<*mut u8> {
    use alloc::alloc::{alloc_zeroed, Layout};
    let allocator_size = (size + 4095) & !4095; // round up to page
    let layout = Layout::from_size_align(allocator_size, 4096).ok()?;
    let ptr = // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe { alloc_zeroed(layout) };
    if ptr.is_null() { None } else { Some(ptr) }
}

/// Free executable pages back to the kernel heap.
fn free_executable_pages(ptr: *mut u8, size: usize) {
    use alloc::alloc::{dealloc, Layout};
    let allocator_size = (size + 4095) & !4095;
    if let Ok(layout) = Layout::from_size_align(allocator_size, 4096) {
                // SAFETY: Unsafe block — bypasses Rust memory-safety guarantees. Ensure invariants manually.
unsafe { dealloc(ptr, layout); }
    }
}
