









use alloc::string::String;
use alloc::vec::Vec;
use alloc::collections::BTreeMap;
use alloc::format;

use super::parser::*;
use super::x86asm::{X86Asm, Reg, Cc, Br};


pub struct Pp {
    
    pub code: Vec<u8>,
    
    pub entry_offset: usize,
    
    pub strings: Vec<String>,
}




pub type Ws = fn(u8, usize, *const i64) -> i64;


struct FnCtx {
    label: Br,
    locals: BTreeMap<String, i32>,  
    next_offset: i32,               
    loop_starts: Vec<Br>,
    loop_ends: Vec<Br>,
}

impl FnCtx {
    fn new(label: Br) -> Self {
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
        let dm = (-self.next_offset) + 8; 
        
        (dm + 15) & !15
    }
}


struct NativeCompiler {
    asm: X86Asm,
    func_labels: BTreeMap<String, Br>,
    strings: Vec<String>,
    string_map: BTreeMap<String, usize>,
    builtin_trampoline: Br,
}

impl NativeCompiler {
    fn new() -> Self {
        let mut asm = X86Asm::new();
        let pnc = asm.new_label();
        Self {
            asm,
            func_labels: BTreeMap::new(),
            strings: Vec::new(),
            string_map: BTreeMap::new(),
            builtin_trampoline: pnc,
        }
    }

    fn intern_string(&mut self, j: &str) -> usize {
        if let Some(&idx) = self.string_map.get(j) {
            return idx;
        }
        let idx = self.strings.len();
        self.strings.push(String::from(j));
        self.string_map.insert(String::from(j), idx);
        idx
    }

    
    fn ehg(name: &str) -> Option<u8> {
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

    fn compile_program(&mut self, program: &Program) -> Result<Pp, String> {
        
        for item in &program.items {
            if let Item::Aq(f) = item {
                let label = self.asm.new_label();
                self.func_labels.insert(f.name.clone(), label);
            }
        }

        
        
        
        self.asm.bind_label(self.builtin_trampoline);
        
        self.asm.call_r(Reg::R15);
        self.asm.ret();

        
        for item in &program.items {
            if let Item::Aq(f) = item {
                self.compile_fn(f)?;
            }
        }

        
        self.asm.resolve_patches().map_err(|e| String::from(e))?;

        let lqr = self.func_labels.get("main")
            .ok_or_else(|| String::from("no main() function found"))?;
        let entry_offset = self.asm.labels[lqr.0]
            .ok_or_else(|| String::from("main() label unresolved"))?;

        Ok(Pp {
            code: self.asm.code.clone(),
            strings: self.strings.clone(),
            entry_offset,
        })
    }

    fn compile_fn(&mut self, decl: &Md) -> Result<(), String> {
        let label = *self.func_labels.get(&decl.name)
            .ok_or_else(|| format!("function '{}' not registered", decl.name))?;

        let mut ab = FnCtx::new(label);

        
        
        
        for (i, (name, _ty)) in decl.params.iter().enumerate() {
            let off = ab.alloc_local(name);
            
            let _ = (i, off);
        }

        
        self.prescan_block_locals(&mut ab, &decl.body);
        let frame_size = ab.frame_size();

        
        self.asm.bind_label(label);
        self.asm.prologue(frame_size);

        
        
        for (i, (name, _ty)) in decl.params.iter().enumerate() {
            let azu = 16 + (i as i32) * 8;
            let afd = *ab.locals.get(name).unwrap();
            self.asm.mov_r_rbp_offset(Reg::Rax, azu);
            self.asm.mov_rbp_offset_r(afd, Reg::Rax);
        }

        
        self.compile_block(&mut ab, &decl.body)?;

        
        self.asm.mov_r_imm32(Reg::Rax, 0);
        self.asm.epilogue();

        Ok(())
    }

    
    fn prescan_block_locals(&self, ab: &mut FnCtx, block: &Bl) {
        for stmt in &block.stmts {
            match stmt {
                Stmt::Let { name, .. } => { ab.alloc_local(name); }
                Stmt::If { avj, atp, .. } => {
                    self.prescan_block_locals(ab, avj);
                    if let Some(bsd) = atp { self.prescan_block_locals(ab, bsd); }
                }
                Stmt::While { body, .. } | Stmt::Loop(body) => {
                    self.prescan_block_locals(ab, body);
                }
                Stmt::For { ael, body, .. } => {
                    ab.alloc_local(ael);
                    ab.alloc_local(&format!("__for_end_{}", ael));
                    self.prescan_block_locals(ab, body);
                }
                _ => {}
            }
        }
    }

    fn compile_block(&mut self, ab: &mut FnCtx, block: &Bl) -> Result<(), String> {
        for stmt in &block.stmts {
            self.compile_stmt(ab, stmt)?;
        }
        Ok(())
    }

    fn compile_stmt(&mut self, ab: &mut FnCtx, stmt: &Stmt) -> Result<(), String> {
        match stmt {
            Stmt::Let { name, init, .. } => {
                let off = *ab.locals.get(name).unwrap();
                if let Some(expr) = init {
                    self.compile_expr(ab, expr)?;
                    self.asm.pop_r(Reg::Rax);
                    self.asm.mov_rbp_offset_r(off, Reg::Rax);
                }
            }
            Stmt::Assign { target, value } => {
                self.compile_expr(ab, value)?;
                self.compile_store(ab, target)?;
            }
            Stmt::OpAssign { op, target, value } => {
                self.compile_expr(ab, target)?;
                self.compile_expr(ab, value)?;
                self.emit_binop(*op)?;
                self.compile_store(ab, &target.clone())?;
            }
            Stmt::Expr(expr) => {
                self.compile_expr(ab, expr)?;
                self.asm.pop_r(Reg::Rax); 
            }
            Stmt::Return(val) => {
                if let Some(expr) = val {
                    self.compile_expr(ab, expr)?;
                    self.asm.pop_r(Reg::Rax);
                } else {
                    self.asm.mov_r_imm32(Reg::Rax, 0);
                }
                self.asm.epilogue();
            }
            Stmt::If { fc, avj, atp } => {
                self.compile_expr(ab, fc)?;
                self.asm.pop_r(Reg::Rax);
                self.asm.test_r_r(Reg::Rax, Reg::Rax);
                let fui = self.asm.new_label();
                self.asm.jcc_label(Cc::Hq, fui); 
                self.compile_block(ab, avj)?;
                if let Some(bsd) = atp {
                    let hvs = self.asm.new_label();
                    self.asm.jmp_label(hvs);
                    self.asm.bind_label(fui);
                    self.compile_block(ab, bsd)?;
                    self.asm.bind_label(hvs);
                } else {
                    self.asm.bind_label(fui);
                }
            }
            Stmt::While { fc, body } => {
                let top = self.asm.new_label();
                let end = self.asm.new_label();
                ab.loop_starts.push(top);
                ab.loop_ends.push(end);

                self.asm.bind_label(top);
                self.compile_expr(ab, fc)?;
                self.asm.pop_r(Reg::Rax);
                self.asm.test_r_r(Reg::Rax, Reg::Rax);
                self.asm.jcc_label(Cc::Hq, end);
                self.compile_block(ab, body)?;
                self.asm.jmp_label(top);
                self.asm.bind_label(end);

                ab.loop_starts.pop();
                ab.loop_ends.pop();
            }
            Stmt::For { ael, iter, body } => {
                if let Expr::Range { start, end } = iter {
                    let feg = *ab.locals.get(ael).unwrap();
                    let lqb = format!("__for_end_{}", ael);
                    let hvu = *ab.locals.get(&lqb).unwrap();

                    self.compile_expr(ab, start)?;
                    self.asm.pop_r(Reg::Rax);
                    self.asm.mov_rbp_offset_r(feg, Reg::Rax);

                    self.compile_expr(ab, end)?;
                    self.asm.pop_r(Reg::Rax);
                    self.asm.mov_rbp_offset_r(hvu, Reg::Rax);

                    let top = self.asm.new_label();
                    let fut = self.asm.new_label();
                    ab.loop_starts.push(top);
                    ab.loop_ends.push(fut);

                    self.asm.bind_label(top);
                    
                    self.asm.mov_r_rbp_offset(Reg::Rax, feg);
                    self.asm.mov_r_rbp_offset(Reg::Rcx, hvu);
                    self.asm.cmp_r_r(Reg::Rax, Reg::Rcx);
                    self.asm.jcc_label(Cc::Ge, fut);

                    self.compile_block(ab, body)?;

                    
                    self.asm.mov_r_rbp_offset(Reg::Rax, feg);
                    self.asm.add_r_imm32(Reg::Rax, 1);
                    self.asm.mov_rbp_offset_r(feg, Reg::Rax);
                    self.asm.jmp_label(top);
                    self.asm.bind_label(fut);

                    ab.loop_starts.pop();
                    ab.loop_ends.pop();
                } else {
                    return Err(String::from("for loop requires a range expression"));
                }
            }
            Stmt::Loop(body) => {
                let top = self.asm.new_label();
                let end = self.asm.new_label();
                ab.loop_starts.push(top);
                ab.loop_ends.push(end);
                self.asm.bind_label(top);
                self.compile_block(ab, body)?;
                self.asm.jmp_label(top);
                self.asm.bind_label(end);
                ab.loop_starts.pop();
                ab.loop_ends.pop();
            }
            Stmt::Break => {
                if let Some(&end) = ab.loop_ends.last() {
                    self.asm.jmp_label(end);
                }
            }
            Stmt::Continue => {
                if let Some(&top) = ab.loop_starts.last() {
                    self.asm.jmp_label(top);
                }
            }
        }
        Ok(())
    }

    fn compile_expr(&mut self, ab: &mut FnCtx, expr: &Expr) -> Result<(), String> {
        match expr {
            Expr::IntLit(v) => {
                self.asm.mov_r_imm64(Reg::Rax, *v);
                self.asm.push_r(Reg::Rax);
            }
            Expr::FloatLit(v) => {
                
                let bits = v.to_bits() as i64;
                self.asm.mov_r_imm64(Reg::Rax, bits);
                self.asm.push_r(Reg::Rax);
            }
            Expr::BoolLit(b) => {
                self.asm.mov_r_imm32(Reg::Rax, if *b { 1 } else { 0 });
                self.asm.push_r(Reg::Rax);
            }
            Expr::StringLit(j) => {
                
                let idx = self.intern_string(j);
                self.asm.mov_r_imm64(Reg::Rax, idx as i64);
                self.asm.push_r(Reg::Rax);
            }
            Expr::Ident(name) => {
                if let Some(&off) = ab.locals.get(name) {
                    self.asm.mov_r_rbp_offset(Reg::Rax, off);
                    self.asm.push_r(Reg::Rax);
                } else {
                    return Err(format!("undefined variable: {}", name));
                }
            }
            Expr::BinOp { op, left, right } => {
                self.compile_expr(ab, left)?;
                self.compile_expr(ab, right)?;
                self.emit_binop(*op)?;
            }
            Expr::UnaryOp { op, expr } => {
                self.compile_expr(ab, expr)?;
                self.asm.pop_r(Reg::Rax);
                match op {
                    UnaryOp::Neg => self.asm.neg_r(Reg::Rax),
                    UnaryOp::Not => {
                        self.asm.test_r_r(Reg::Rax, Reg::Rax);
                        self.asm.setcc(Cc::Hq, Reg::Rax);
                        self.asm.movzx_r_r8(Reg::Rax, Reg::Rax);
                    }
                }
                self.asm.push_r(Reg::Rax);
            }
            Expr::Call { func, args } => {
                if let Some(bid) = Self::ehg(func) {
                    
                    
                    
                    let anl = args.len();
                    
                    if anl > 0 {
                        self.asm.sub_r_imm32(Reg::Rsp, (anl as i32) * 8);
                    }
                    
                    for (i, db) in args.iter().enumerate() {
                        self.compile_expr(ab, db)?;
                        self.asm.pop_r(Reg::Rax);
                        
                        
                        let off = (i as i32) * 8;
                        
                        self.emit_mov_rsp_offset_r(off, Reg::Rax);
                    }
                    
                    self.asm.mov_r_imm32(Reg::Rdi, bid as i32);
                    self.asm.mov_r_imm32(Reg::Rsi, anl as i32);
                    self.asm.mov_r_r(Reg::Rdx, Reg::Rsp);
                    
                    self.asm.call_label(self.builtin_trampoline);
                    
                    if anl > 0 {
                        self.asm.add_r_imm32(Reg::Rsp, (anl as i32) * 8);
                    }
                    
                    self.asm.push_r(Reg::Rax);
                } else if let Some(&func_label) = self.func_labels.get(func) {
                    
                    for db in args.iter().rev() {
                        self.compile_expr(ab, db)?;
                        
                    }
                    self.asm.call_label(func_label);
                    
                    let anl = args.len();
                    if anl > 0 {
                        self.asm.add_r_imm32(Reg::Rsp, (anl as i32) * 8);
                    }
                    
                    self.asm.push_r(Reg::Rax);
                } else {
                    return Err(format!("undefined function: {}", func));
                }
            }
            Expr::Cast { expr, ty } => {
                self.compile_expr(ab, expr)?;
                match ty {
                    Type::F64 => {
                        
                        
                        
                    }
                    Type::I64 => {
                        
                    }
                    _ => {}
                }
            }
            Expr::Range { start, end } => {
                
                self.compile_expr(ab, start)?;
                
                self.compile_expr(ab, end)?;
                self.asm.pop_r(Reg::Rax); 
            }
            Expr::Index { .. } | Expr::Array(_) | Expr::Field { .. } | Expr::Bl(_) => {
                
                
                self.asm.mov_r_imm32(Reg::Rax, 0);
                self.asm.push_r(Reg::Rax);
            }
        }
        Ok(())
    }

    
    fn emit_binop(&mut self, op: BinOp) -> Result<(), String> {
        self.asm.pop_r(Reg::Rcx); 
        self.asm.pop_r(Reg::Rax); 
        match op {
            BinOp::Add => self.asm.add_r_r(Reg::Rax, Reg::Rcx),
            BinOp::Sub => self.asm.sub_r_r(Reg::Rax, Reg::Rcx),
            BinOp::Mul => self.asm.imul_r_r(Reg::Rax, Reg::Rcx),
            BinOp::Div => {
                self.asm.cqo();
                self.asm.idiv_r(Reg::Rcx);
                
            }
            BinOp::Mod => {
                self.asm.cqo();
                self.asm.idiv_r(Reg::Rcx);
                self.asm.mov_r_r(Reg::Rax, Reg::Rdx); 
            }
            BinOp::Eq => {
                self.asm.cmp_r_r(Reg::Rax, Reg::Rcx);
                self.asm.setcc(Cc::Hq, Reg::Rax);
                self.asm.movzx_r_r8(Reg::Rax, Reg::Rax);
            }
            BinOp::NotEq => {
                self.asm.cmp_r_r(Reg::Rax, Reg::Rcx);
                self.asm.setcc(Cc::Ne, Reg::Rax);
                self.asm.movzx_r_r8(Reg::Rax, Reg::Rax);
            }
            BinOp::Lt => {
                self.asm.cmp_r_r(Reg::Rax, Reg::Rcx);
                self.asm.setcc(Cc::Th, Reg::Rax);
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
                
                self.asm.shl_r_cl(Reg::Rax);
            }
            BinOp::Shr => {
                self.asm.sar_r_cl(Reg::Rax);
            }
        }
        self.asm.push_r(Reg::Rax);
        Ok(())
    }

    
    fn compile_store(&mut self, ab: &mut FnCtx, target: &Expr) -> Result<(), String> {
        match target {
            Expr::Ident(name) => {
                if let Some(&off) = ab.locals.get(name) {
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

    
    fn emit_mov_rsp_offset_r(&mut self, offset: i32, src: Reg) {
        
        
        let mut rp: u8 = 0x48;
        if src.needs_rex() { rp |= 0x04; }
        self.asm.code.push(rp);
        self.asm.code.push(0x89);
        if offset == 0 {
            self.asm.code.push(X86Asm::fi(0b00, src.lo3(), 0x04)); 
            self.asm.code.push(0x24); 
        } else if offset >= -128 && offset <= 127 {
            self.asm.code.push(X86Asm::fi(0b01, src.lo3(), 0x04));
            self.asm.code.push(0x24);
            self.asm.code.push(offset as u8);
        } else {
            self.asm.code.push(X86Asm::fi(0b10, src.lo3(), 0x04));
            self.asm.code.push(0x24);
            self.asm.code.extend_from_slice(&offset.to_le_bytes());
        }
    }
}


pub fn dle(source: &str) -> Result<Pp, String> {
    let tokens = super::lexer::crv(source)?;
    let dhy = super::parser::parse(&tokens)?;
    let mut compiler = NativeCompiler::new();
    compiler.compile_program(&dhy)
}




pub unsafe fn doz(
    program: &Pp,
    builtin_callback: Ws,
) -> Result<i64, String> {
    let code = &program.code;
    if code.is_empty() {
        return Err(String::from("empty native program"));
    }

    
    let fvm = juu(code.len())
        .ok_or_else(|| String::from("failed to allocate executable memory"))?;

    
    core::ptr::copy_nonoverlapping(code.as_ptr(), fvm, code.len());

    
    let entry: *const u8 = fvm.add(program.entry_offset);

    
    
    
    let result: i64;
    let cb_ptr = builtin_callback as usize;
    let entry_ptr = entry as usize;

    core::arch::asm!(
        "mov r15, {cb}",
        "call {entry}",
        cb = in(reg) cb_ptr,
        entry = in(reg) entry_ptr,
        out("rax") result,
        
        out("rcx") _, out("rdx") _, out("rsi") _, out("rdi") _,
        out("r8") _, out("r9") _, out("r10") _, out("r11") _,
        clobber_abi("C"),
    );

    
    lyp(fvm, code.len());

    Ok(result)
}






fn juu(size: usize) -> Option<*mut u8> {
    use alloc::alloc::{alloc_zeroed, Layout};
    let fgt = (size + 4095) & !4095; 
    let layout = Layout::from_size_align(fgt, 4096).ok()?;
    let ptr = unsafe { alloc_zeroed(layout) };
    if ptr.is_null() { None } else { Some(ptr) }
}


fn lyp(ptr: *mut u8, size: usize) {
    use alloc::alloc::{dealloc, Layout};
    let fgt = (size + 4095) & !4095;
    if let Ok(layout) = Layout::from_size_align(fgt, 4096) {
        unsafe { dealloc(ptr, layout); }
    }
}
