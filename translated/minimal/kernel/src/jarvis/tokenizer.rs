

















use alloc::string::String;
use alloc::vec::Vec;


pub const CLT_: u8 = 0x00;
pub const ABA_: u8 = 0x01;  
pub const ADL_: u8 = 0x02;  
pub const CVU_: u8 = 0x03;  


pub fn bbj(text: &str) -> Vec<u8> {
    let mut tokens = Vec::with_capacity(text.len() + 2);
    tokens.push(ABA_);
    for byte in text.bytes() {
        tokens.push(byte);
    }
    tokens.push(ADL_);
    tokens
}


pub fn qfb(text: &str) -> Vec<u8> {
    text.bytes().collect()
}


pub fn dmo(tokens: &[u8]) -> String {
    let mut j = String::with_capacity(tokens.len());
    for &t in tokens {
        match t {
            CLT_ => {}      
            ABA_ => {}      
            ADL_ => break,  
            CVU_ => j.push('\n'),
            0x20..=0x7E => j.push(t as char),  
            0x0A => j.push('\n'),               
            0x09 => j.push('\t'),               
            0x0D => {}                           
            _ => {
                
                j.push(t as char);
            }
        }
    }
    j
}


pub fn rap(text: &str) -> usize {
    text.len() 
}


pub fn qmv(abm: u8) -> bool {
    matches!(abm, 0x20..=0x7E | 0x0A | 0x09)
}


pub fn raq(abm: u8) -> &'static str {
    match abm {
        0x00 => "<PAD>",
        0x01 => "<BOS>",
        0x02 => "<EOS>",
        0x03 => "<SEP>",
        0x0A => "<NL>",
        0x0D => "<CR>",
        0x09 => "<TAB>",
        0x20 => "<SP>",
        _ => "",  
    }
}
