//! TrustLang — Integrated Programming Language for TrustOS
//!
//! A Rust-inspired language with:
//! - Familiar syntax (fn, let, if/else, while, for, return, struct)
//! - Stack-based bytecode VM
//! - No borrow checker (GC-free, manual memory via stack)
//! - Builtin I/O: print(), read_line(), file_read(), file_write()
//!
//! Usage:
//!   trustlang run file.tl    — compile and execute
//!   trustlang repl           — interactive REPL
//!   trustlang check file.tl  — syntax check only

pub mod lexer;
pub mod parser;
pub mod vm;
pub mod compiler;

use alloc::string::String;
use alloc::vec::Vec;

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
