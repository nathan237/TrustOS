














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


pub fn vw(iy: &str) -> Result<String, String> {
    let eb = lexer::fwz(iy)?;
    let gzb = parser::parse(&eb)?;
    let hby = compiler::rmx(&gzb)?;
    vm::bna(&hby)
}


pub fn feq(iy: &str) -> Result<(), String> {
    let eb = lexer::fwz(iy)?;
    let xxx = parser::parse(&eb)?;
    Ok(())
}



pub fn nrc(line: &str) -> Result<String, String> {
    
    if line.contains("fn main") || line.contains("fn ") {
        return vw(line);
    }
    
    let fyx = format!("fn main() {{ {} }}", line);
    vw(&fyx)
}



pub fn rmy(iy: &str, qug: native::Bcv) -> Result<i64, String> {
    let alo = native::hdq(iy)?;
    unsafe { native::him(&alo, qug) }
}


pub fn wbi() -> (usize, usize, String) {
    tests::jne()
}
