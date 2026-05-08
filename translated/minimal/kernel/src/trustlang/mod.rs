














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


pub fn run(source: &str) -> Result<String, String> {
    let tokens = lexer::crv(source)?;
    let dhy = parser::parse(&tokens)?;
    let dkd = compiler::kwd(&dhy)?;
    vm::execute(&dkd)
}


pub fn cgv(source: &str) -> Result<(), String> {
    let tokens = lexer::crv(source)?;
    let pwr = parser::parse(&tokens)?;
    Ok(())
}



pub fn hwq(line: &str) -> Result<String, String> {
    
    if line.contains("fn main") || line.contains("fn ") {
        return run(line);
    }
    
    let wrapped = format!("fn main() {{ {} }}", line);
    run(&wrapped)
}



pub fn kwe(source: &str, builtin_cb: native::Ws) -> Result<i64, String> {
    let program = native::dle(source)?;
    unsafe { native::doz(&program, builtin_cb) }
}


pub fn ojd() -> (usize, usize, String) {
    tests::ezf()
}
