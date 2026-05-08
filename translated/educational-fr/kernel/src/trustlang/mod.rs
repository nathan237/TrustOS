//! TrustLang — Integrated Programming Language for TrustOS
//!
//! A Rust-inspired language with:
//! - Familiar syntax (fn, let, if/else, while, for, return, struct)
//! - Stack-based bytecode VM + native x86_64 compiler
//! - No borrow checker (GC-free, manual memory via stack)
//! - Builtin I/O: print(), read_line(), file_read(), file_write()
//!
//! Usage:
//!   trustlang run file.tl       — compile and execute (bytecode VM)
//!   trustlang compile file.tl   — compile to native x86_64 and execute
//!   trustlang repl              — interactive REPL
//!   trustlang check file.tl     — syntax check only
//!   trustlang test              — run native backend test suite

pub mod lexer;
pub mod parser;
pub mod vm;
pub mod compiler;
pub mod x86asm;
pub mod native;
pub mod tests;

use alloc::string::String;
use alloc::vec::Vec;
use alloc::format;

/// Compile and run TrustLang source code
pub fn run(source: &str) -> Result<String, String> {
    let tokens = lexer::tokenize(source)?;
    let ast = parser::parse(&tokens)?;
    let bytecode = compiler::compile(&ast)?;
    vm::execute(&bytecode)
}

/// Syntax-check only (returns errors or "OK")
pub fn check(source: &str) -> Result<(), String> {
    let tokens = lexer::tokenize(source)?;
    let _ast = parser::parse(&tokens)?;
    Ok(())
}

/// Evaluate a single expression or statement in REPL mode.
/// Wraps the input in `fn main() { ... }` for convenience.
pub fn eval_line(line: &str) -> Result<String, String> {
    // Try as-is first (might be a full program)
    if line.contains("fn main") || line.contains("fn ") {
        return run(line);
    }
    // Wrap in main()
    let wrapped = format!("fn main() {{ {} }}", line);
    run(&wrapped)
}

/// Compile TrustLang source to native x86_64 and execute it.
/// Returns the i64 return value of main().
pub fn compile_and_run_native(source: &str, builtin_cb: native::BuiltinFn) -> Result<i64, String> {
    let program = native::compile_native(source)?;
        // SÉCURITÉ : Bloc unsafe — contourne les garanties mémoire de Rust. Vérifier les invariants manuellement.
unsafe { native::execute_native(&program, builtin_cb) }
}

/// Run the native backend test suite.
pub fn run_native_tests() -> (usize, usize, String) {
    tests::run_all_tests()
}
