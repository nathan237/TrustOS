//! Mini Guest Programs for TrustVM
//!
//! Petits programmes guests prêts à l'emploi pour tester l'hyperviseur

use alloc::vec::Vec;
use alloc::vec;

/// Simple guest that prints "Hello from VM!" and halts
pub fn hello_guest() -> Vec<u8> {
    // This is x86_64 machine code that:
    // 1. Outputs "Hello from TrustVM!" to port 0xE9 (QEMU debug)
    // 2. Executes HLT to stop
    
    let mut code = Vec::new();
    
    // Message to print
    let message = b"Hello from TrustVM!\n";
    
    // MOV AL, char ; OUT 0xE9, AL for each character
    for &byte in message {
        code.push(0xB0);        // MOV AL, imm8
        code.push(byte);        // character
        code.push(0xE6);        // OUT imm8, AL
        code.push(0xE9);        // port 0xE9
    }
    
    // HLT
    code.push(0xF4);
    
    code
}

/// Guest that counts and prints numbers
pub fn counter_guest(max: u32) -> Vec<u8> {
    let mut code = Vec::new();
    
    // Print "Count: X" for X from 0 to max
    for i in 0..=max {
        // Print "Count: "
        for &byte in b"Count: " {
            code.push(0xB0);    // MOV AL, imm8
            code.push(byte);
            code.push(0xE6);    // OUT imm8, AL
            code.push(0xE9);
        }
        
        // Print number (simple ASCII)
        let digit = b'0' + (i % 10) as u8;
        code.push(0xB0);
        code.push(digit);
        code.push(0xE6);
        code.push(0xE9);
        
        // Newline
        code.push(0xB0);
        code.push(b'\n');
        code.push(0xE6);
        code.push(0xE9);
    }
    
    // HLT
    code.push(0xF4);
    
    code
}

/// Guest that uses VMCALL hypercall
pub fn hypercall_guest() -> Vec<u8> {
    let mut code = Vec::new();
    
    // Print message first
    for &byte in b"Testing VMCALL hypercall...\n" {
        code.push(0xB0);
        code.push(byte);
        code.push(0xE6);
        code.push(0xE9);
    }
    
    // VMCALL with function 0 (print)
    // MOV EAX, 0
    code.extend_from_slice(&[0xB8, 0x00, 0x00, 0x00, 0x00]);
    // VMCALL
    code.extend_from_slice(&[0x0F, 0x01, 0xC1]);
    
    // Print result
    for &byte in b"VMCALL returned!\n" {
        code.push(0xB0);
        code.push(byte);
        code.push(0xE6);
        code.push(0xE9);
    }
    
    // VMCALL with function 1 (exit)
    code.extend_from_slice(&[0xB8, 0x01, 0x00, 0x00, 0x00]); // MOV EAX, 1
    code.extend_from_slice(&[0x0F, 0x01, 0xC1]); // VMCALL
    
    // Should not reach here, but HLT just in case
    code.push(0xF4);
    
    code
}

/// Guest that uses CPUID
pub fn cpuid_guest() -> Vec<u8> {
    let mut code = Vec::new();
    
    // Print header
    for &byte in b"CPUID Test:\n" {
        code.push(0xB0);
        code.push(byte);
        code.push(0xE6);
        code.push(0xE9);
    }
    
    // CPUID with EAX=0 to get vendor string
    // XOR EAX, EAX (31 C0)
    code.extend_from_slice(&[0x31, 0xC0]);
    // CPUID (0F A2)
    code.extend_from_slice(&[0x0F, 0xA2]);
    
    // EBX, EDX, ECX contain vendor string
    // Print "Vendor: " 
    for &byte in b"Vendor: " {
        code.push(0xB0);
        code.push(byte);
        code.push(0xE6);
        code.push(0xE9);
    }
    
    // Print EBX (4 chars) - requires more complex code
    // For simplicity, just print "OK\n"
    for &byte in b"(detected)\n" {
        code.push(0xB0);
        code.push(byte);
        code.push(0xE6);
        code.push(0xE9);
    }
    
    // HLT
    code.push(0xF4);
    
    code
}

/// Interactive shell guest (simple)
pub fn shell_guest() -> Vec<u8> {
    let mut code = Vec::new();
    
    // Print prompt and wait for input
    // This is a very simple "shell" that just echoes
    
    // Print banner
    for &byte in b"TrustVM Shell v0.1\n> " {
        code.push(0xB0);
        code.push(byte);
        code.push(0xE6);
        code.push(0xE9);
    }
    
    // For ports > 255 like 0x3F8, we need MOV DX, port; IN AL, DX
    // But for simplicity, just print banner and halt
    
    // Print banner
    code.clear();
    for &byte in b"TrustVM Shell v0.1\n> " {
        code.push(0xB0);
        code.push(byte);
        code.push(0xE6);
        code.push(0xE9);
    }
    
    // Simple version: just HLT for now
    // A real shell would need more complex I/O handling
    code.push(0xF4);
    
    code
}

/// Protected-mode guest that tests I/O, CR writes, and HLT
/// Use with setup_protected_mode(0x1000, 0x8000)
pub fn guest_protected_mode_test() -> Vec<u8> {
    let mut code = Vec::new();
    
    // 32-bit protected mode code
    // Print banner via COM1 serial (port 0x3F8) — tests I/O emulation
    let banner = b"[TrustVM] Protected mode guest running!\r\n";
    for &byte in banner {
        // MOV DX, 0x3F8
        code.extend_from_slice(&[0x66, 0xBA, 0xF8, 0x03]);
        // MOV AL, byte
        code.extend_from_slice(&[0xB0, byte]);
        // OUT DX, AL
        code.push(0xEE);
    }
    
    // Also print via debug port 0xE9
    for &byte in b"[PM-test] I/O OK\n" {
        code.extend_from_slice(&[0xB0, byte]);  // MOV AL, imm8
        code.extend_from_slice(&[0xE6, 0xE9]);  // OUT 0xE9, AL
    }
    
    // Read PIC mask — IN AL, 0x21
    code.extend_from_slice(&[0xE4, 0x21]); // IN AL, 0x21 (PIC1 data)
    
    // Write PIC mask — OUT 0x21, AL (mask all IRQs)
    code.extend_from_slice(&[0xB0, 0xFF]); // MOV AL, 0xFF
    code.extend_from_slice(&[0xE6, 0x21]); // OUT 0x21, AL
    
    // Print CR test message
    for &byte in b"[PM-test] CR write test\n" {
        code.extend_from_slice(&[0xB0, byte]);
        code.extend_from_slice(&[0xE6, 0xE9]);
    }
    
    // Read CR0 into EAX
    code.extend_from_slice(&[0x0F, 0x20, 0xC0]); // MOV EAX, CR0
    // Set NE bit (bit 5) — numeric error
    code.extend_from_slice(&[0x0D, 0x20, 0x00, 0x00, 0x00]); // OR EAX, 0x20
    // Write back CR0
    code.extend_from_slice(&[0x0F, 0x22, 0xC0]); // MOV CR0, EAX
    
    // Print success
    for &byte in b"[PM-test] CR0 write OK\n" {
        code.extend_from_slice(&[0xB0, byte]);
        code.extend_from_slice(&[0xE6, 0xE9]);
    }
    
    // Test HLT (should get timer interrupt injected)
    for &byte in b"[PM-test] HLT test\n" {
        code.extend_from_slice(&[0xB0, byte]);
        code.extend_from_slice(&[0xE6, 0xE9]);
    }
    code.push(0xF4); // HLT
    
    // After waking from HLT, print and exit
    for &byte in b"[PM-test] Woke from HLT!\n" {
        code.extend_from_slice(&[0xB0, byte]);
        code.extend_from_slice(&[0xE6, 0xE9]);
    }
    
    // Exit via VMMCALL (AMD: 0F 01 D9)
    code.extend_from_slice(&[0xB8, 0x01, 0x00, 0x00, 0x00]); // MOV EAX, 1 (exit)
    code.extend_from_slice(&[0x0F, 0x01, 0xD9]); // VMMCALL
    
    // HLT fallback
    code.push(0xF4);
    
    code
}

/// 64-bit guest code template
/// Returns (code, entry_offset)
pub fn guest_64bit_hello() -> Vec<u8> {
    // 64-bit mode code
    let mut code = Vec::new();
    
    // Print message using debug port
    let message = b"[TrustVM Guest] Running in 64-bit mode!\n";
    
    for &byte in message {
        // MOV AL, byte
        code.extend_from_slice(&[0xB0, byte]);
        // OUT 0xE9, AL
        code.extend_from_slice(&[0xE6, 0xE9]);
    }
    
    // VMCALL to exit (hypercall 1)
    // MOV RAX, 1
    code.extend_from_slice(&[0x48, 0xC7, 0xC0, 0x01, 0x00, 0x00, 0x00]);
    // VMCALL
    code.extend_from_slice(&[0x0F, 0x01, 0xC1]);
    
    // HLT (fallback)
    code.push(0xF4);
    
    code
}

/// Get a guest by name
pub fn get_guest(name: &str) -> Option<Vec<u8>> {
    match name {
        "hello" => Some(hello_guest()),
        "counter" => Some(counter_guest(9)),
        "hypercall" => Some(hypercall_guest()),
        "cpuid" => Some(cpuid_guest()),
        "shell" => Some(shell_guest()),
        "hello64" => Some(guest_64bit_hello()),
        "pm-test" | "protected" => Some(guest_protected_mode_test()),
        _ => None,
    }
}

/// List available guests
pub fn list_guests() -> &'static [&'static str] {
    &["hello", "counter", "hypercall", "cpuid", "shell", "hello64", "pm-test"]
}
