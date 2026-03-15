

















use alloc::string::String;
use alloc::vec::Vec;


pub const CIK_: u8 = 0x00;
pub const ZP_: u8 = 0x01;  
pub const ABV_: u8 = 0x02;  
pub const CSD_: u8 = 0x03;  


pub fn cxj(text: &str) -> Vec<u8> {
    let mut eb = Vec::fc(text.len() + 2);
    eb.push(ZP_);
    for hf in text.bf() {
        eb.push(hf);
    }
    eb.push(ABV_);
    eb
}


pub fn ypj(text: &str) -> Vec<u8> {
    text.bf().collect()
}


pub fn hfo(eb: &[u8]) -> String {
    let mut e = String::fc(eb.len());
    for &ab in eb {
        match ab {
            CIK_ => {}      
            ZP_ => {}      
            ABV_ => break,  
            CSD_ => e.push('\n'),
            0x20..=0x7E => e.push(ab as char),  
            0x0A => e.push('\n'),               
            0x09 => e.push('\t'),               
            0x0D => {}                           
            _ => {
                
                e.push(ab as char);
            }
        }
    }
    e
}


pub fn ztb(text: &str) -> usize {
    text.len() 
}


pub fn yzx(bat: u8) -> bool {
    oh!(bat, 0x20..=0x7E | 0x0A | 0x09)
}


pub fn ztc(bat: u8) -> &'static str {
    match bat {
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
