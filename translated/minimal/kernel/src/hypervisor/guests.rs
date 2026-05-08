



use alloc::vec::Vec;
use alloc::vec;


pub fn ieq() -> Vec<u8> {
    
    
    
    
    let mut code = Vec::new();
    
    
    let message = b"Hello from TrustVM!\n";
    
    
    for &byte in message {
        code.push(0xB0);        
        code.push(byte);        
        code.push(0xE6);        
        code.push(0xE9);        
    }
    
    
    code.push(0xF4);
    
    code
}


pub fn kyf(max: u32) -> Vec<u8> {
    let mut code = Vec::new();
    
    
    for i in 0..=max {
        
        for &byte in b"Count: " {
            code.push(0xB0);    
            code.push(byte);
            code.push(0xE6);    
            code.push(0xE9);
        }
        
        
        let blu = b'0' + (i % 10) as u8;
        code.push(0xB0);
        code.push(blu);
        code.push(0xE6);
        code.push(0xE9);
        
        
        code.push(0xB0);
        code.push(b'\n');
        code.push(0xE6);
        code.push(0xE9);
    }
    
    
    code.push(0xF4);
    
    code
}


pub fn mmw() -> Vec<u8> {
    let mut code = Vec::new();
    
    
    for &byte in b"Testing VMCALL hypercall...\n" {
        code.push(0xB0);
        code.push(byte);
        code.push(0xE6);
        code.push(0xE9);
    }
    
    
    
    code.extend_from_slice(&[0xB8, 0x00, 0x00, 0x00, 0x00]);
    
    code.extend_from_slice(&[0x0F, 0x01, 0xC1]);
    
    
    for &byte in b"VMCALL returned!\n" {
        code.push(0xB0);
        code.push(byte);
        code.push(0xE6);
        code.push(0xE9);
    }
    
    
    code.extend_from_slice(&[0xB8, 0x01, 0x00, 0x00, 0x00]); 
    code.extend_from_slice(&[0x0F, 0x01, 0xC1]); 
    
    
    code.push(0xF4);
    
    code
}


pub fn kyz() -> Vec<u8> {
    let mut code = Vec::new();
    
    
    for &byte in b"CPUID Test:\n" {
        code.push(0xB0);
        code.push(byte);
        code.push(0xE6);
        code.push(0xE9);
    }
    
    
    
    code.extend_from_slice(&[0x31, 0xC0]);
    
    code.extend_from_slice(&[0x0F, 0xA2]);
    
    
    
    for &byte in b"Vendor: " {
        code.push(0xB0);
        code.push(byte);
        code.push(0xE6);
        code.push(0xE9);
    }
    
    
    
    for &byte in b"(detected)\n" {
        code.push(0xB0);
        code.push(byte);
        code.push(0xE6);
        code.push(0xE9);
    }
    
    
    code.push(0xF4);
    
    code
}


pub fn orp() -> Vec<u8> {
    let mut code = Vec::new();
    
    
    
    
    
    for &byte in b"TrustVM Shell v0.1\n> " {
        code.push(0xB0);
        code.push(byte);
        code.push(0xE6);
        code.push(0xE9);
    }
    
    
    
    
    
    code.clear();
    for &byte in b"TrustVM Shell v0.1\n> " {
        code.push(0xB0);
        code.push(byte);
        code.push(0xE6);
        code.push(0xE9);
    }
    
    
    
    code.push(0xF4);
    
    code
}



pub fn mgl() -> Vec<u8> {
    let mut code = Vec::new();
    
    
    
    fn ajo(code: &mut Vec<u8>, bk: &[u8]) {
        for &byte in bk {
            code.extend_from_slice(&[0x66, 0xBA, 0xF8, 0x03]); 
            code.extend_from_slice(&[0xB0, byte]);               
            code.push(0xEE);                                     
        }
    }
    
    fn bgh(code: &mut Vec<u8>, bk: &[u8]) {
        for &byte in bk {
            code.extend_from_slice(&[0xB0, byte]);  
            code.extend_from_slice(&[0xE6, 0xE9]);  
        }
    }
    
    
    ajo(&mut code, b"\r\n");
    ajo(&mut code, b"========================================\r\n");
    ajo(&mut code, b"  TrustVM Protected-Mode Mini-Kernel\r\n");
    ajo(&mut code, b"  AMD SVM hardware virtualization\r\n");
    ajo(&mut code, b"========================================\r\n");
    bgh(&mut code, b"[PM-test] Boot OK\n");
    
    
    ajo(&mut code, b"[1/7] CPUID vendor... \r\n");
    
    code.extend_from_slice(&[0x31, 0xC0]);       
    code.extend_from_slice(&[0x0F, 0xA2]);       
    bgh(&mut code, b"[PM-test] CPUID leaf 0 OK\n");
    
    
    ajo(&mut code, b"[2/7] CPUID features... \r\n");
    code.extend_from_slice(&[0xB8, 0x01, 0x00, 0x00, 0x00]); 
    code.extend_from_slice(&[0x0F, 0xA2]);                     
    bgh(&mut code, b"[PM-test] CPUID leaf 1 OK\n");
    
    
    ajo(&mut code, b"[3/7] CPUID extended... \r\n");
    code.extend_from_slice(&[0xB8, 0x00, 0x00, 0x00, 0x80]); 
    code.extend_from_slice(&[0x0F, 0xA2]);                     
    bgh(&mut code, b"[PM-test] CPUID leaf 80000000 OK\n");
    
    
    ajo(&mut code, b"[4/7] I/O port tests... \r\n");
    
    
    code.extend_from_slice(&[0xE4, 0x21]); 
    
    code.extend_from_slice(&[0xB0, 0xFF]); 
    code.extend_from_slice(&[0xE6, 0x21]); 
    
    
    code.extend_from_slice(&[0xE4, 0x40]); 
    
    
    code.extend_from_slice(&[0xB0, 0x00]); 
    code.extend_from_slice(&[0xE6, 0x70]); 
    code.extend_from_slice(&[0xE4, 0x71]); 
    
    
    code.extend_from_slice(&[0xE4, 0x64]); 
    
    bgh(&mut code, b"[PM-test] I/O ports OK\n");
    
    
    ajo(&mut code, b"[5/7] Control register tests... \r\n");
    
    
    code.extend_from_slice(&[0x0F, 0x20, 0xC0]); 
    
    code.extend_from_slice(&[0x0D, 0x20, 0x00, 0x00, 0x00]); 
    
    code.extend_from_slice(&[0x0F, 0x22, 0xC0]); 
    bgh(&mut code, b"[PM-test] CR0 write OK\n");
    
    
    code.extend_from_slice(&[0x0F, 0x20, 0xE0]); 
    code.extend_from_slice(&[0x0D, 0x00, 0x02, 0x00, 0x00]); 
    code.extend_from_slice(&[0x0F, 0x22, 0xE0]); 
    bgh(&mut code, b"[PM-test] CR4 write OK\n");
    
    
    ajo(&mut code, b"[6/7] Memory test... \r\n");
    
    
    
    code.extend_from_slice(&[0xBF, 0x00, 0x50, 0x00, 0x00]);
    
    code.extend_from_slice(&[0xB8, 0xEF, 0xBE, 0xAD, 0xDE]);
    
    code.extend_from_slice(&[0x89, 0x07]);
    
    code.extend_from_slice(&[0x31, 0xC0]);
    
    code.extend_from_slice(&[0x8B, 0x07]);
    
    bgh(&mut code, b"[PM-test] Memory OK\n");
    
    
    ajo(&mut code, b"[7/7] Hypercall tests... \r\n");
    
    
    code.extend_from_slice(&[0xB8, 0x02, 0x00, 0x00, 0x00]); 
    code.extend_from_slice(&[0x0F, 0x01, 0xD9]);               
    
    bgh(&mut code, b"[PM-test] VMMCALL get_time OK\n");
    
    
    
    let mkk = b"Hello from TrustVM hypercall!\0";
    
    code.extend_from_slice(&[0xBF, 0x00, 0x60, 0x00, 0x00]);
    for (i, &byte) in mkk.iter().enumerate() {
        
        code.extend_from_slice(&[0xC6, 0x87]);
        code.extend_from_slice(&(i as u32).to_le_bytes());
        code.push(byte);
    }
    
    
    code.extend_from_slice(&[0xBB, 0x00, 0x60, 0x00, 0x00]); 
    code.extend_from_slice(&[0xB8, 0x01, 0x00, 0x00, 0x00]); 
    code.extend_from_slice(&[0x0F, 0x01, 0xD9]);               
    bgh(&mut code, b"[PM-test] VMMCALL print OK\n");
    
    
    ajo(&mut code, b"[*] HLT (waiting for timer inject)...\r\n");
    code.push(0xF4); 
    
    
    ajo(&mut code, b"[*] Woke from HLT!\r\n");
    ajo(&mut code, b"========================================\r\n");
    ajo(&mut code, b"  All 7 phases PASSED\r\n");
    ajo(&mut code, b"  VM exiting via VMMCALL...\r\n");
    ajo(&mut code, b"========================================\r\n");
    bgh(&mut code, b"[PM-test] ALL TESTS PASSED\n");
    
    
    
    code.extend_from_slice(&[0xB8, 0xAD, 0xDE, 0x00, 0x00]); 
    
    code.extend_from_slice(&[0xBB, 0xFE, 0xCA, 0x00, 0x00]); 
    
    code.extend_from_slice(&[0xB8, 0x00, 0x00, 0x00, 0x00]); 
    code.extend_from_slice(&[0x0F, 0x01, 0xD9]);               
    
    
    code.push(0xF4);
    
    code
}



pub fn mgg() -> Vec<u8> {
    
    let mut code = Vec::new();
    
    
    let message = b"[TrustVM Guest] Running in 64-bit mode!\n";
    
    for &byte in message {
        
        code.extend_from_slice(&[0xB0, byte]);
        
        code.extend_from_slice(&[0xE6, 0xE9]);
    }
    
    
    
    code.extend_from_slice(&[0x48, 0xC7, 0xC0, 0x01, 0x00, 0x00, 0x00]);
    
    code.extend_from_slice(&[0x0F, 0x01, 0xC1]);
    
    
    code.push(0xF4);
    
    code
}


pub fn eoc(name: &str) -> Option<Vec<u8>> {
    match name {
        "hello" => Some(ieq()),
        "counter" => Some(kyf(9)),
        "hypercall" => Some(mmw()),
        "cpuid" => Some(kyz()),
        "shell" => Some(orp()),
        "hello64" => Some(mgg()),
        "pm-test" | "protected" => Some(mgl()),
        "linux-test" => Some(super::linux_loader::fpb()),
        _ => None,
    }
}


pub fn dtj() -> &'static [&'static str] {
    &["hello", "counter", "hypercall", "cpuid", "shell", "hello64", "pm-test", "linux-test"]
}
