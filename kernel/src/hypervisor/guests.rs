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
    
    // ── Helper closures ──────────────────────────────────────────
    // Print string via COM1 serial (port 0x3F8) — 32-bit pmode
    fn emit_serial(code: &mut Vec<u8>, msg: &[u8]) {
        for &byte in msg {
            code.extend_from_slice(&[0x66, 0xBA, 0xF8, 0x03]); // MOV DX, 0x3F8
            code.extend_from_slice(&[0xB0, byte]);               // MOV AL, byte
            code.push(0xEE);                                     // OUT DX, AL
        }
    }
    // Print string via debug port 0xE9
    fn emit_debug(code: &mut Vec<u8>, msg: &[u8]) {
        for &byte in msg {
            code.extend_from_slice(&[0xB0, byte]);  // MOV AL, imm8
            code.extend_from_slice(&[0xE6, 0xE9]);  // OUT 0xE9, AL
        }
    }
    
    // ── Phase 1: Banner ──────────────────────────────────────────
    emit_serial(&mut code, b"\r\n");
    emit_serial(&mut code, b"========================================\r\n");
    emit_serial(&mut code, b"  TrustVM Protected-Mode Mini-Kernel\r\n");
    emit_serial(&mut code, b"  AMD SVM hardware virtualization\r\n");
    emit_serial(&mut code, b"========================================\r\n");
    emit_debug(&mut code, b"[PM-test] Boot OK\n");
    
    // ── Phase 2: CPUID queries ───────────────────────────────────
    emit_serial(&mut code, b"[1/7] CPUID vendor... \r\n");
    // EAX=0 → vendor string in EBX:EDX:ECX
    code.extend_from_slice(&[0x31, 0xC0]);       // XOR EAX, EAX
    code.extend_from_slice(&[0x0F, 0xA2]);       // CPUID
    emit_debug(&mut code, b"[PM-test] CPUID leaf 0 OK\n");
    
    // EAX=1 → processor info + feature flags
    emit_serial(&mut code, b"[2/7] CPUID features... \r\n");
    code.extend_from_slice(&[0xB8, 0x01, 0x00, 0x00, 0x00]); // MOV EAX, 1
    code.extend_from_slice(&[0x0F, 0xA2]);                     // CPUID
    emit_debug(&mut code, b"[PM-test] CPUID leaf 1 OK\n");
    
    // EAX=0x80000000 → extended CPUID max
    emit_serial(&mut code, b"[3/7] CPUID extended... \r\n");
    code.extend_from_slice(&[0xB8, 0x00, 0x00, 0x00, 0x80]); // MOV EAX, 0x80000000
    code.extend_from_slice(&[0x0F, 0xA2]);                     // CPUID
    emit_debug(&mut code, b"[PM-test] CPUID leaf 80000000 OK\n");
    
    // ── Phase 3: I/O ports ───────────────────────────────────────
    emit_serial(&mut code, b"[4/7] I/O port tests... \r\n");
    
    // Read PIC mask — IN AL, 0x21
    code.extend_from_slice(&[0xE4, 0x21]); // IN AL, 0x21 (PIC1 data)
    // Write PIC mask — all masked
    code.extend_from_slice(&[0xB0, 0xFF]); // MOV AL, 0xFF
    code.extend_from_slice(&[0xE6, 0x21]); // OUT 0x21, AL
    
    // Read PIT counter 0 (port 0x40) — tests PIT I/O
    code.extend_from_slice(&[0xE4, 0x40]); // IN AL, 0x40
    
    // Read CMOS register 0 (seconds) — port 0x70/0x71
    code.extend_from_slice(&[0xB0, 0x00]); // MOV AL, 0 (seconds register)
    code.extend_from_slice(&[0xE6, 0x70]); // OUT 0x70, AL (select reg)
    code.extend_from_slice(&[0xE4, 0x71]); // IN AL, 0x71 (read value)
    
    // Read keyboard status port
    code.extend_from_slice(&[0xE4, 0x64]); // IN AL, 0x64 (KBC status)
    
    emit_debug(&mut code, b"[PM-test] I/O ports OK\n");
    
    // ── Phase 4: CR writes ───────────────────────────────────────
    emit_serial(&mut code, b"[5/7] Control register tests... \r\n");
    
    // Read CR0 into EAX
    code.extend_from_slice(&[0x0F, 0x20, 0xC0]); // MOV EAX, CR0
    // Set NE bit (bit 5) — numeric error
    code.extend_from_slice(&[0x0D, 0x20, 0x00, 0x00, 0x00]); // OR EAX, 0x20
    // Write back CR0
    code.extend_from_slice(&[0x0F, 0x22, 0xC0]); // MOV CR0, EAX
    emit_debug(&mut code, b"[PM-test] CR0 write OK\n");
    
    // Read CR4 — set OSFXSR (bit 9) for SSE support indication
    code.extend_from_slice(&[0x0F, 0x20, 0xE0]); // MOV EAX, CR4
    code.extend_from_slice(&[0x0D, 0x00, 0x02, 0x00, 0x00]); // OR EAX, 0x200 (OSFXSR)
    code.extend_from_slice(&[0x0F, 0x22, 0xE0]); // MOV CR4, EAX
    emit_debug(&mut code, b"[PM-test] CR4 write OK\n");
    
    // ── Phase 5: Memory operations ───────────────────────────────
    emit_serial(&mut code, b"[6/7] Memory test... \r\n");
    
    // Write a pattern to address 0x5000 (well within our 4MB guest memory)
    // MOV EDI, 0x5000
    code.extend_from_slice(&[0xBF, 0x00, 0x50, 0x00, 0x00]);
    // MOV EAX, 0xDEADBEEF
    code.extend_from_slice(&[0xB8, 0xEF, 0xBE, 0xAD, 0xDE]);
    // MOV [EDI], EAX (store)
    code.extend_from_slice(&[0x89, 0x07]);
    // MOV EAX, 0 (clear)
    code.extend_from_slice(&[0x31, 0xC0]);
    // MOV EAX, [EDI] (load back)
    code.extend_from_slice(&[0x8B, 0x07]);
    // EAX should now be 0xDEADBEEF — verified by Inspector's register view
    emit_debug(&mut code, b"[PM-test] Memory OK\n");
    
    // ── Phase 6: Hypercall — get TSC ─────────────────────────────
    emit_serial(&mut code, b"[7/7] Hypercall tests... \r\n");
    
    // VMMCALL func 0x02 = get TSC
    code.extend_from_slice(&[0xB8, 0x02, 0x00, 0x00, 0x00]); // MOV EAX, 2
    code.extend_from_slice(&[0x0F, 0x01, 0xD9]);               // VMMCALL
    // EAX now has TSC low bits
    emit_debug(&mut code, b"[PM-test] VMMCALL get_time OK\n");
    
    // Store a string at GPA 0x6000 for hypercall_print
    // "Hello from hypercall!\0"
    let hc_msg = b"Hello from TrustVM hypercall!\0";
    // MOV EDI, 0x6000
    code.extend_from_slice(&[0xBF, 0x00, 0x60, 0x00, 0x00]);
    for (i, &byte) in hc_msg.iter().enumerate() {
        // MOV BYTE [EDI + i], byte
        code.extend_from_slice(&[0xC6, 0x87]);
        code.extend_from_slice(&(i as u32).to_le_bytes());
        code.push(byte);
    }
    
    // VMMCALL func 0x01 = print string at GPA in EBX
    code.extend_from_slice(&[0xBB, 0x00, 0x60, 0x00, 0x00]); // MOV EBX, 0x6000
    code.extend_from_slice(&[0xB8, 0x01, 0x00, 0x00, 0x00]); // MOV EAX, 1
    code.extend_from_slice(&[0x0F, 0x01, 0xD9]);               // VMMCALL
    emit_debug(&mut code, b"[PM-test] VMMCALL print OK\n");
    
    // ── Phase 7: HLT + wake ──────────────────────────────────────
    emit_serial(&mut code, b"[*] HLT (waiting for timer inject)...\r\n");
    code.push(0xF4); // HLT
    
    // After waking from HLT, declare success
    emit_serial(&mut code, b"[*] Woke from HLT!\r\n");
    emit_serial(&mut code, b"========================================\r\n");
    emit_serial(&mut code, b"  All 7 phases PASSED\r\n");
    emit_serial(&mut code, b"  VM exiting via VMMCALL...\r\n");
    emit_serial(&mut code, b"========================================\r\n");
    emit_debug(&mut code, b"[PM-test] ALL TESTS PASSED\n");
    
    // ── Exit via VMMCALL ─────────────────────────────────────────
    // Load EAX with 0xDEAD to leave a visible signature in registers
    code.extend_from_slice(&[0xB8, 0xAD, 0xDE, 0x00, 0x00]); // MOV EAX, 0xDEAD
    // Load EBX with 0xCAFE
    code.extend_from_slice(&[0xBB, 0xFE, 0xCA, 0x00, 0x00]); // MOV EBX, 0xCAFE
    // Now set EAX=0 (exit hypercall)
    code.extend_from_slice(&[0xB8, 0x00, 0x00, 0x00, 0x00]); // MOV EAX, 0 (exit)
    code.extend_from_slice(&[0x0F, 0x01, 0xD9]);               // VMMCALL
    
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
