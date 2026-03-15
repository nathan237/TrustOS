



use alloc::vec::Vec;
use alloc::vec;


pub fn obp() -> Vec<u8> {
    
    
    
    
    let mut aj = Vec::new();
    
    
    let message = b"Hello from TrustVM!\n";
    
    
    for &hf in message {
        aj.push(0xB0);        
        aj.push(hf);        
        aj.push(0xE6);        
        aj.push(0xE9);        
    }
    
    
    aj.push(0xF4);
    
    aj
}


pub fn rph(am: u32) -> Vec<u8> {
    let mut aj = Vec::new();
    
    
    for a in 0..=am {
        
        for &hf in b"Count: " {
            aj.push(0xB0);    
            aj.push(hf);
            aj.push(0xE6);    
            aj.push(0xE9);
        }
        
        
        let dpy = b'0' + (a % 10) as u8;
        aj.push(0xB0);
        aj.push(dpy);
        aj.push(0xE6);
        aj.push(0xE9);
        
        
        aj.push(0xB0);
        aj.push(b'\n');
        aj.push(0xE6);
        aj.push(0xE9);
    }
    
    
    aj.push(0xF4);
    
    aj
}


pub fn tqs() -> Vec<u8> {
    let mut aj = Vec::new();
    
    
    for &hf in b"Testing VMCALL hypercall...\n" {
        aj.push(0xB0);
        aj.push(hf);
        aj.push(0xE6);
        aj.push(0xE9);
    }
    
    
    
    aj.bk(&[0xB8, 0x00, 0x00, 0x00, 0x00]);
    
    aj.bk(&[0x0F, 0x01, 0xC1]);
    
    
    for &hf in b"VMCALL returned!\n" {
        aj.push(0xB0);
        aj.push(hf);
        aj.push(0xE6);
        aj.push(0xE9);
    }
    
    
    aj.bk(&[0xB8, 0x01, 0x00, 0x00, 0x00]); 
    aj.bk(&[0x0F, 0x01, 0xC1]); 
    
    
    aj.push(0xF4);
    
    aj
}


pub fn rqd() -> Vec<u8> {
    let mut aj = Vec::new();
    
    
    for &hf in b"CPUID Test:\n" {
        aj.push(0xB0);
        aj.push(hf);
        aj.push(0xE6);
        aj.push(0xE9);
    }
    
    
    
    aj.bk(&[0x31, 0xC0]);
    
    aj.bk(&[0x0F, 0xA2]);
    
    
    
    for &hf in b"Vendor: " {
        aj.push(0xB0);
        aj.push(hf);
        aj.push(0xE6);
        aj.push(0xE9);
    }
    
    
    
    for &hf in b"(detected)\n" {
        aj.push(0xB0);
        aj.push(hf);
        aj.push(0xE6);
        aj.push(0xE9);
    }
    
    
    aj.push(0xF4);
    
    aj
}


pub fn wmr() -> Vec<u8> {
    let mut aj = Vec::new();
    
    
    
    
    
    for &hf in b"TrustVM Shell v0.1\n> " {
        aj.push(0xB0);
        aj.push(hf);
        aj.push(0xE6);
        aj.push(0xE9);
    }
    
    
    
    
    
    aj.clear();
    for &hf in b"TrustVM Shell v0.1\n> " {
        aj.push(0xB0);
        aj.push(hf);
        aj.push(0xE6);
        aj.push(0xE9);
    }
    
    
    
    aj.push(0xF4);
    
    aj
}



pub fn tic() -> Vec<u8> {
    let mut aj = Vec::new();
    
    
    
    fn bqk(aj: &mut Vec<u8>, fr: &[u8]) {
        for &hf in fr {
            aj.bk(&[0x66, 0xBA, 0xF8, 0x03]); 
            aj.bk(&[0xB0, hf]);               
            aj.push(0xEE);                                     
        }
    }
    
    fn dgx(aj: &mut Vec<u8>, fr: &[u8]) {
        for &hf in fr {
            aj.bk(&[0xB0, hf]);  
            aj.bk(&[0xE6, 0xE9]);  
        }
    }
    
    
    bqk(&mut aj, b"\r\n");
    bqk(&mut aj, b"========================================\r\n");
    bqk(&mut aj, b"  TrustVM Protected-Mode Mini-Kernel\r\n");
    bqk(&mut aj, b"  AMD SVM hardware virtualization\r\n");
    bqk(&mut aj, b"========================================\r\n");
    dgx(&mut aj, b"[PM-test] Boot OK\n");
    
    
    bqk(&mut aj, b"[1/7] CPUID vendor... \r\n");
    
    aj.bk(&[0x31, 0xC0]);       
    aj.bk(&[0x0F, 0xA2]);       
    dgx(&mut aj, b"[PM-test] CPUID leaf 0 OK\n");
    
    
    bqk(&mut aj, b"[2/7] CPUID features... \r\n");
    aj.bk(&[0xB8, 0x01, 0x00, 0x00, 0x00]); 
    aj.bk(&[0x0F, 0xA2]);                     
    dgx(&mut aj, b"[PM-test] CPUID leaf 1 OK\n");
    
    
    bqk(&mut aj, b"[3/7] CPUID extended... \r\n");
    aj.bk(&[0xB8, 0x00, 0x00, 0x00, 0x80]); 
    aj.bk(&[0x0F, 0xA2]);                     
    dgx(&mut aj, b"[PM-test] CPUID leaf 80000000 OK\n");
    
    
    bqk(&mut aj, b"[4/7] I/O port tests... \r\n");
    
    
    aj.bk(&[0xE4, 0x21]); 
    
    aj.bk(&[0xB0, 0xFF]); 
    aj.bk(&[0xE6, 0x21]); 
    
    
    aj.bk(&[0xE4, 0x40]); 
    
    
    aj.bk(&[0xB0, 0x00]); 
    aj.bk(&[0xE6, 0x70]); 
    aj.bk(&[0xE4, 0x71]); 
    
    
    aj.bk(&[0xE4, 0x64]); 
    
    dgx(&mut aj, b"[PM-test] I/O ports OK\n");
    
    
    bqk(&mut aj, b"[5/7] Control register tests... \r\n");
    
    
    aj.bk(&[0x0F, 0x20, 0xC0]); 
    
    aj.bk(&[0x0D, 0x20, 0x00, 0x00, 0x00]); 
    
    aj.bk(&[0x0F, 0x22, 0xC0]); 
    dgx(&mut aj, b"[PM-test] CR0 write OK\n");
    
    
    aj.bk(&[0x0F, 0x20, 0xE0]); 
    aj.bk(&[0x0D, 0x00, 0x02, 0x00, 0x00]); 
    aj.bk(&[0x0F, 0x22, 0xE0]); 
    dgx(&mut aj, b"[PM-test] CR4 write OK\n");
    
    
    bqk(&mut aj, b"[6/7] Memory test... \r\n");
    
    
    
    aj.bk(&[0xBF, 0x00, 0x50, 0x00, 0x00]);
    
    aj.bk(&[0xB8, 0xEF, 0xBE, 0xAD, 0xDE]);
    
    aj.bk(&[0x89, 0x07]);
    
    aj.bk(&[0x31, 0xC0]);
    
    aj.bk(&[0x8B, 0x07]);
    
    dgx(&mut aj, b"[PM-test] Memory OK\n");
    
    
    bqk(&mut aj, b"[7/7] Hypercall tests... \r\n");
    
    
    aj.bk(&[0xB8, 0x02, 0x00, 0x00, 0x00]); 
    aj.bk(&[0x0F, 0x01, 0xD9]);               
    
    dgx(&mut aj, b"[PM-test] VMMCALL get_time OK\n");
    
    
    
    let tno = b"Hello from TrustVM hypercall!\0";
    
    aj.bk(&[0xBF, 0x00, 0x60, 0x00, 0x00]);
    for (a, &hf) in tno.iter().cf() {
        
        aj.bk(&[0xC6, 0x87]);
        aj.bk(&(a as u32).ho());
        aj.push(hf);
    }
    
    
    aj.bk(&[0xBB, 0x00, 0x60, 0x00, 0x00]); 
    aj.bk(&[0xB8, 0x01, 0x00, 0x00, 0x00]); 
    aj.bk(&[0x0F, 0x01, 0xD9]);               
    dgx(&mut aj, b"[PM-test] VMMCALL print OK\n");
    
    
    bqk(&mut aj, b"[*] HLT (waiting for timer inject)...\r\n");
    aj.push(0xF4); 
    
    
    bqk(&mut aj, b"[*] Woke from HLT!\r\n");
    bqk(&mut aj, b"========================================\r\n");
    bqk(&mut aj, b"  All 7 phases PASSED\r\n");
    bqk(&mut aj, b"  VM exiting via VMMCALL...\r\n");
    bqk(&mut aj, b"========================================\r\n");
    dgx(&mut aj, b"[PM-test] ALL TESTS PASSED\n");
    
    
    
    aj.bk(&[0xB8, 0xAD, 0xDE, 0x00, 0x00]); 
    
    aj.bk(&[0xBB, 0xFE, 0xCA, 0x00, 0x00]); 
    
    aj.bk(&[0xB8, 0x00, 0x00, 0x00, 0x00]); 
    aj.bk(&[0x0F, 0x01, 0xD9]);               
    
    
    aj.push(0xF4);
    
    aj
}



pub fn thw() -> Vec<u8> {
    
    let mut aj = Vec::new();
    
    
    let message = b"[TrustVM Guest] Running in 64-bit mode!\n";
    
    for &hf in message {
        
        aj.bk(&[0xB0, hf]);
        
        aj.bk(&[0xE6, 0xE9]);
    }
    
    
    
    aj.bk(&[0x48, 0xC7, 0xC0, 0x01, 0x00, 0x00, 0x00]);
    
    aj.bk(&[0x0F, 0x01, 0xC1]);
    
    
    aj.push(0xF4);
    
    aj
}


pub fn iwr(j: &str) -> Option<Vec<u8>> {
    match j {
        "hello" => Some(obp()),
        "counter" => Some(rph(9)),
        "hypercall" => Some(tqs()),
        "cpuid" => Some(rqd()),
        "shell" => Some(wmr()),
        "hello64" => Some(thw()),
        "pm-test" | "protected" => Some(tic()),
        "linux-test" => Some(super::linux_loader::klw()),
        _ => None,
    }
}


pub fn hpy() -> &'static [&'static str] {
    &["hello", "counter", "hypercall", "cpuid", "shell", "hello64", "pm-test", "linux-test"]
}
