//! ELF Executable Runner
//!
//! Loads and executes ELF binaries in Ring 3 user space.
//! Provides the bridge between the shell and userland execution.

use alloc::vec::Vec;
use alloc::string::String;
use core::sync::atomic::{AtomicU64, AtomicPtr, Ordering};
use crate::elf::{LoadedElf, ElfError, ElfResult};
use crate::memory::paging::{AddressSpace, PageFlags, UserMemoryRegion};
use crate::memory::hhdm_offset;

/// Stack size for user processes (1 MB)
const USER_STACK_SIZE: usize = 1024 * 1024;

// ── Current process context (for syscall / page-fault access) ──

/// Raw pointer to the currently-executing user AddressSpace.
/// Set before entering Ring 3, cleared on return.
static CURRENT_USER_SPACE: AtomicPtr<AddressSpace> = AtomicPtr::new(core::ptr::null_mut());

/// Current user program break (heap top virtual address)
static CURRENT_USER_BRK: AtomicU64 = AtomicU64::new(0);

/// Current user stack bottom (lowest mapped stack page)
static CURRENT_USER_STACK_BOTTOM: AtomicU64 = AtomicU64::new(0);

/// Access the current user AddressSpace from within a syscall or page fault handler.
/// Returns `None` if no user process is running.
///
/// # Safety
/// The caller must not hold this reference across an address-space switch.
pub fn with_current_address_space<F, R>(f: F) -> Option<R>
where
    F: FnOnce(&mut AddressSpace) -> R,
{
    let ptr = CURRENT_USER_SPACE.load(Ordering::Acquire);
    if ptr.is_null() {
        return None;
    }
    // Safety: pointer is valid while a user process is executing, and we are in a
    // syscall / exception handler on the same (only) CPU.
    Some(f(unsafe { &mut *ptr }))
}

/// Get current user program break
pub fn current_brk() -> u64 {
    CURRENT_USER_BRK.load(Ordering::Relaxed)
}

/// Set current user program break
pub fn set_current_brk(brk: u64) {
    CURRENT_USER_BRK.store(brk, Ordering::SeqCst);
}

/// Get current user stack bottom (lowest valid stack address)
pub fn current_stack_bottom() -> u64 {
    CURRENT_USER_STACK_BOTTOM.load(Ordering::Relaxed)
}

/// Result of program execution
#[derive(Debug)]
pub enum ExecResult {
    /// Program exited normally with code
    Exited(i32),
    /// Program crashed/faulted
    Faulted(&'static str),
    /// Failed to load
    LoadError(ElfError),
    /// Memory allocation failed
    MemoryError,
}

/// Linux-compatible execve syscall
/// Replaces the current process image with a new program
pub fn execve(path: &str, argv: &[&str], _envp: &[&str]) -> Result<(), ExecResult> {
    match exec_path(path, argv) {
        ExecResult::Exited(0) => Ok(()),
        other => Err(other),
    }
}

/// Execute an ELF file from path
pub fn exec_path(path: &str, args: &[&str]) -> ExecResult {
    crate::log!("[EXEC] Loading: {}", path);
    
    // Load ELF file
    let elf = match crate::elf::load_from_path(path) {
        Ok(e) => e,
        Err(e) => {
            crate::log_error!("[EXEC] Failed to load ELF: {:?}", e);
            return ExecResult::LoadError(e);
        }
    };
    
    exec_elf(&elf, args)
}

/// Execute an ELF from bytes
pub fn exec_bytes(data: &[u8], args: &[&str]) -> ExecResult {
    // Load ELF from memory
    let elf = match crate::elf::load_from_bytes(data) {
        Ok(e) => e,
        Err(e) => {
            crate::log_error!("[EXEC] Failed to parse ELF: {:?}", e);
            return ExecResult::LoadError(e);
        }
    };
    
    exec_elf(&elf, args)
}

/// Execute a loaded ELF
fn exec_elf(elf: &LoadedElf, args: &[&str]) -> ExecResult {
    crate::log!("[EXEC] Entry point: {:#x}", elf.entry_point);
    crate::log!("[EXEC] Address range: {:#x} - {:#x}", elf.min_vaddr, elf.max_vaddr);
    
    // Create user address space
    let mut address_space = match AddressSpace::new_with_kernel() {
        Some(a) => a,
        None => {
            crate::log_error!("[EXEC] Failed to create address space");
            return ExecResult::MemoryError;
        }
    };
    
    let hhdm = hhdm_offset();
    
    // Map ELF segments into user space
    for segment in &elf.segments {
        let pages_needed = ((segment.size as usize + 4095) / 4096).max(1);
        
        crate::log_debug!("[EXEC] Mapping segment: vaddr={:#x}, size={}, pages={}", 
            segment.vaddr, segment.size, pages_needed);
        
        for page_idx in 0..pages_needed {
            let virt_addr = (segment.vaddr & !0xFFF) + (page_idx as u64 * 4096);
            
            // Allocate physical page
            let phys_page = match alloc_physical_page() {
                Some(p) => p,
                None => {
                    crate::log_error!("[EXEC] Out of memory");
                    return ExecResult::MemoryError;
                }
            };
            
            // Determine page flags
            let is_exec = (segment.flags & 1) != 0;  // PF_X
            let is_write = (segment.flags & 2) != 0; // PF_W
            
            let flags = if is_exec {
                PageFlags::USER_CODE
            } else if is_write {
                PageFlags::USER_DATA
            } else {
                PageFlags::USER_RODATA
            };
            
            // Map the page
            if address_space.map_page(virt_addr, phys_page, flags).is_none() {
                crate::log_error!("[EXEC] Failed to map page at {:#x}", virt_addr);
                return ExecResult::MemoryError;
            }
            
            // Copy segment data to physical page
            let page_virt = phys_page + hhdm;
            let seg_offset = page_idx * 4096;
            let copy_start = seg_offset;
            let copy_end = ((segment.data.len()).min(seg_offset + 4096)).max(seg_offset);
            
            if copy_start < segment.data.len() {
                let src = &segment.data[copy_start..copy_end];
                let dest = unsafe {
                    core::slice::from_raw_parts_mut(page_virt as *mut u8, 4096)
                };
                dest[..src.len()].copy_from_slice(src);
                // Zero the rest of the page
                for b in &mut dest[src.len()..] {
                    *b = 0;
                }
            } else {
                // Zero the entire page (BSS)
                let dest = unsafe {
                    core::slice::from_raw_parts_mut(page_virt as *mut u8, 4096)
                };
                dest.fill(0);
            }
        }
    }
    
    // Allocate and map user stack with guard page
    let stack_pages = USER_STACK_SIZE / 4096;
    let stack_base = UserMemoryRegion::STACK_TOP - (stack_pages as u64 * 4096);
    let guard_page = stack_base - 4096;
    
    crate::log_debug!("[EXEC] Guard page at {:#x}, stack: {:#x} - {:#x}", guard_page, stack_base, UserMemoryRegion::STACK_TOP);
    
    // Guard page: intentionally left unmapped — any access triggers a page fault
    // (no map_page call for guard_page address)
    
    for i in 0..stack_pages {
        let virt_addr = stack_base + (i as u64 * 4096);
        
        let phys_page = match alloc_physical_page() {
            Some(p) => p,
            None => {
                crate::log_error!("[EXEC] Out of memory for stack");
                return ExecResult::MemoryError;
            }
        };
        
        // Zero the stack page
        let page_virt = phys_page + hhdm;
        unsafe {
            core::ptr::write_bytes(page_virt as *mut u8, 0, 4096);
        }
        
        if address_space.map_page(virt_addr, phys_page, PageFlags::USER_DATA).is_none() {
            crate::log_error!("[EXEC] Failed to map stack at {:#x}", virt_addr);
            return ExecResult::MemoryError;
        }
    }
    
    // Set up initial stack with argc/argv (System V ABI: argc at RSP, then argv pointers, then strings)
    let mut sp = UserMemoryRegion::STACK_TOP;
    
    // Step 1: Copy argument strings onto the stack (at the top, above the pointers)
    let mut arg_addrs: Vec<u64> = Vec::new();
    for arg in args.iter().rev() {
        let bytes = arg.as_bytes();
        sp -= (bytes.len() as u64) + 1; // +1 for null terminator
        // Write string to user stack via HHDM
        let stack_page_base = sp & !0xFFF;
        let page_offset = (sp - stack_base) as usize;
        let page_idx = page_offset / 4096;
        if page_idx < stack_pages {
            // Find physical page backing this stack address
            // We write via the HHDM mapping — find the PTE
            if let Some(phys) = address_space.translate(sp) {
                let dest = (phys + hhdm) as *mut u8;
                unsafe {
                    core::ptr::copy_nonoverlapping(bytes.as_ptr(), dest, bytes.len());
                    *dest.add(bytes.len()) = 0; // null terminator
                }
            }
        }
        arg_addrs.push(sp);
    }
    arg_addrs.reverse(); // Now arg_addrs[0] = first arg address
    
    // Step 2: Align SP to 8 bytes
    sp &= !7;
    
    // Step 3: Push null terminator for argv array
    sp -= 8;
    if let Some(phys) = address_space.translate(sp) {
        unsafe { *((phys + hhdm) as *mut u64) = 0; }
    }
    
    // Step 4: Push argv pointers (in reverse)
    for addr in arg_addrs.iter().rev() {
        sp -= 8;
        if let Some(phys) = address_space.translate(sp) {
            unsafe { *((phys + hhdm) as *mut u64) = *addr; }
        }
    }
    let argv_ptr = sp; // argv points to first pointer
    
    // Step 5: Push argc
    sp -= 8;
    if let Some(phys) = address_space.translate(sp) {
        unsafe { *((phys + hhdm) as *mut u64) = args.len() as u64; }
    }
    
    // Ensure 16-byte alignment (ABI requirement before call)
    sp &= !0xF;
    
    crate::log!("[EXEC] Ready to execute at {:#x}, stack at {:#x} (argc={}, argv={:#x})", 
        elf.entry_point, sp, args.len(), argv_ptr);
    
    // ── Process lifecycle: spawn → run → exit → reap ────────────
    let proc_name = args.first().copied().unwrap_or("user");
    let pid = crate::process::spawn(proc_name).unwrap_or(0);
    let prev_pid = crate::process::current_pid();
    
    // Update memory layout in the process table
    crate::process::set_memory(pid, crate::process::MemoryLayout {
        code_start: elf.min_vaddr,
        code_end: elf.max_vaddr,
        heap_start: UserMemoryRegion::HEAP_START,
        heap_end: UserMemoryRegion::HEAP_START,
        stack_start: stack_base,
        stack_end: UserMemoryRegion::STACK_TOP,
        ..Default::default()
    });
    
    // Switch to user address space and jump to Ring 3
    let exit_code;
    
    // Open fd 0/1/2 (stdin/stdout/stderr) for the user process
    crate::vfs::setup_stdio();
    
    unsafe {
        // Save kernel CR3
        let kernel_cr3: u64;
        core::arch::asm!("mov {}, cr3", out(reg) kernel_cr3);
        
        // Publish process context for syscall / page-fault handlers
        CURRENT_USER_SPACE.store(&mut address_space as *mut AddressSpace, Ordering::Release);
        CURRENT_USER_BRK.store(UserMemoryRegion::HEAP_START, Ordering::SeqCst);
        CURRENT_USER_STACK_BOTTOM.store(stack_base, Ordering::SeqCst);
        
        // Mark as running in the process table
        crate::process::start_running(pid);
        
        // Activate user address space
        address_space.activate();
        
        crate::log!("[EXEC] PID {} entering Ring 3 at {:#x}...", pid, elf.entry_point);
        
        // Jump to Ring 3 — returns when user process calls exit()
        exit_code = crate::userland::exec_ring3_process(elf.entry_point, sp);
        
        // Clear process context
        CURRENT_USER_SPACE.store(core::ptr::null_mut(), Ordering::Release);
        
        // Restore kernel CR3 (safety — return_from_ring3 already does this)
        core::arch::asm!("mov cr3, {}", in(reg) kernel_cr3, options(nostack, preserves_flags));
    }
    
    // Record exit in process table and reap
    crate::process::finish(pid, exit_code);
    crate::process::reap(pid);
    
    // Clean up stdio descriptors
    crate::vfs::cleanup_stdio();
    
    // Restore previous PID (shell/kernel)
    crate::process::set_current(prev_pid);
    
    crate::log!("[EXEC] PID {} exited with code {}", pid, exit_code);
    ExecResult::Exited(exit_code)
}

/// Execute a Ring 3 hello world test program
///
/// Runs a small program in Ring 3 that:
/// 1. Calls write(1, "Hello from Ring 3!\n", 19) via syscall
/// 2. Calls exit(0) via syscall
/// 3. Returns to kernel with exit code 0
pub fn exec_test_program() -> ExecResult {
    crate::log!("[EXEC] Running Ring 3 hello world test...");
    
    // Create user address space (includes kernel mappings)
    let mut address_space = match AddressSpace::new_with_kernel() {
        Some(a) => a,
        None => {
            crate::log_error!("[EXEC] Failed to create address space");
            return ExecResult::MemoryError;
        }
    };
    
    let hhdm = hhdm_offset();
    
    // Machine code for Ring 3 hello world:
    //
    //   _start:                        ; offset 0x00
    //     mov rax, 1                   ; SYS_WRITE
    //     mov rdi, 1                   ; fd = stdout
    //     lea rsi, [rip + msg]         ; buf = &msg (RIP-relative)
    //     mov rdx, 19                  ; count = 19
    //     syscall
    //     mov rax, 60                  ; SYS_EXIT
    //     xor rdi, rdi                 ; code = 0
    //     syscall
    //     jmp $                        ; safety: infinite loop
    //   msg:
    //     db "Hello from Ring 3!", 10  ; 19 bytes with newline
    //
    // LEA displacement: msg is at offset 44, next instruction after LEA is at offset 21
    // displacement = 44 - 21 = 23 = 0x17
    let test_code: [u8; 63] = [
        // mov rax, 1 (SYS_WRITE)
        0x48, 0xC7, 0xC0, 0x01, 0x00, 0x00, 0x00,
        // mov rdi, 1 (stdout)
        0x48, 0xC7, 0xC7, 0x01, 0x00, 0x00, 0x00,
        // lea rsi, [rip + 0x17] (msg)
        0x48, 0x8D, 0x35, 0x17, 0x00, 0x00, 0x00,
        // mov rdx, 19 (count)
        0x48, 0xC7, 0xC2, 0x13, 0x00, 0x00, 0x00,
        // syscall
        0x0F, 0x05,
        // mov rax, 60 (SYS_EXIT)
        0x48, 0xC7, 0xC0, 0x3C, 0x00, 0x00, 0x00,
        // xor rdi, rdi (exit code 0)
        0x48, 0x31, 0xFF,
        // syscall
        0x0F, 0x05,
        // jmp $ (safety loop)
        0xEB, 0xFE,
        // "Hello from Ring 3!\n" (19 bytes)
        b'H', b'e', b'l', b'l', b'o', b' ', b'f', b'r', b'o', b'm',
        b' ', b'R', b'i', b'n', b'g', b' ', b'3', b'!', b'\n',
    ];
    
    // Allocate and map code page at 0x400000
    let code_vaddr: u64 = 0x400000;
    let code_phys = match alloc_physical_page() {
        Some(p) => p,
        None => return ExecResult::MemoryError,
    };
    
    crate::log!("[EXEC] Code page: phys={:#x}, vaddr={:#x}", code_phys, code_vaddr);
    
    // Copy code to physical page
    unsafe {
        let dest = (code_phys + hhdm) as *mut u8;
        core::ptr::write_bytes(dest, 0, 4096); // Zero first
        core::ptr::copy_nonoverlapping(test_code.as_ptr(), dest, test_code.len());
    }
    
    // Map code page as executable
    if address_space.map_page(code_vaddr, code_phys, PageFlags::USER_CODE).is_none() {
        crate::log_error!("[EXEC] Failed to map code page");
        return ExecResult::MemoryError;
    }
    
    // Allocate and map stack (16 KB) with guard page below
    let stack_top: u64 = 0x7FFFFFFF0000;
    let stack_pages = 4;
    let guard_page_addr = stack_top - (stack_pages as u64 + 1) * 4096;
    // Guard page at guard_page_addr: left unmapped — access triggers page fault
    
    for i in 0..stack_pages {
        let vaddr = stack_top - (i as u64 + 1) * 4096;
        let phys = match alloc_physical_page() {
            Some(p) => p,
            None => return ExecResult::MemoryError,
        };
        unsafe { core::ptr::write_bytes((phys + hhdm) as *mut u8, 0, 4096); }
        
        if address_space.map_page(vaddr, phys, PageFlags::USER_DATA).is_none() {
            crate::log_error!("[EXEC] Failed to map stack page");
            return ExecResult::MemoryError;
        }
    }
    
    let user_stack = stack_top - 8; // Align
    
    crate::log!("[EXEC] Jumping to Ring 3 at {:#x}, stack at {:#x}", code_vaddr, user_stack);
    
    // Switch to user address space and execute in Ring 3
    unsafe {
        let kernel_cr3: u64;
        core::arch::asm!("mov {}, cr3", out(reg) kernel_cr3);
        
        // Publish process context
        CURRENT_USER_SPACE.store(&mut address_space as *mut AddressSpace, Ordering::Release);
        CURRENT_USER_BRK.store(UserMemoryRegion::HEAP_START, Ordering::SeqCst);
        let test_stack_bottom = stack_top - (stack_pages as u64 * 4096);
        CURRENT_USER_STACK_BOTTOM.store(test_stack_bottom, Ordering::SeqCst);
        
        address_space.activate();
        
        let exit_code = crate::userland::exec_ring3_process(code_vaddr, user_stack);
        
        // Clear process context
        CURRENT_USER_SPACE.store(core::ptr::null_mut(), Ordering::Release);
        
        // Restore kernel CR3
        core::arch::asm!("mov cr3, {}", in(reg) kernel_cr3, options(nostack, preserves_flags));
        
        crate::log!("[EXEC] Ring 3 test exited with code {}", exit_code);
        ExecResult::Exited(exit_code)
    }
}

/// Embedded minimal ELF64 binary: "Hello from Ring 3!"
///
/// This is a complete, standalone ELF64 static executable that:
/// 1. Writes "Hello from Ring 3!\n" to stdout (fd 1) via write() syscall
/// 2. Exits with code 0 via exit() syscall
///
/// Can be loaded with exec_bytes() for a full ELF parse + Ring 3 execution test.
pub static HELLO_ELF: &[u8] = &{
    // ── ELF64 Header (64 bytes) ──
    let mut elf = [0u8; 183];
    
    // e_ident
    elf[0] = 0x7F; elf[1] = b'E'; elf[2] = b'L'; elf[3] = b'F'; // magic
    elf[4] = 2;    // ELFCLASS64
    elf[5] = 1;    // ELFDATA2LSB
    elf[6] = 1;    // EV_CURRENT
    // elf[7..16] = 0 (padding)
    
    // e_type = ET_EXEC (2)
    elf[16] = 2; elf[17] = 0;
    // e_machine = EM_X86_64 (62 = 0x3E)
    elf[18] = 0x3E; elf[19] = 0;
    // e_version = 1
    elf[20] = 1; elf[21] = 0; elf[22] = 0; elf[23] = 0;
    // e_entry = 0x400078 (code starts after headers)
    elf[24] = 0x78; elf[25] = 0x00; elf[26] = 0x40; elf[27] = 0x00;
    elf[28] = 0; elf[29] = 0; elf[30] = 0; elf[31] = 0;
    // e_phoff = 64 (program header right after ELF header)
    elf[32] = 64; elf[33] = 0; elf[34] = 0; elf[35] = 0;
    elf[36] = 0; elf[37] = 0; elf[38] = 0; elf[39] = 0;
    // e_shoff = 0
    // elf[40..48] = 0
    // e_flags = 0
    // elf[48..52] = 0
    // e_ehsize = 64
    elf[52] = 64; elf[53] = 0;
    // e_phentsize = 56
    elf[54] = 56; elf[55] = 0;
    // e_phnum = 1
    elf[56] = 1; elf[57] = 0;
    // e_shentsize = 64
    elf[58] = 64; elf[59] = 0;
    // e_shnum = 0, e_shstrndx = 0
    // elf[60..64] = 0
    
    // ── Program Header (56 bytes, offset 64) ──
    // p_type = PT_LOAD (1)
    elf[64] = 1; elf[65] = 0; elf[66] = 0; elf[67] = 0;
    // p_flags = PF_R | PF_X (5)
    elf[68] = 5; elf[69] = 0; elf[70] = 0; elf[71] = 0;
    // p_offset = 0
    // elf[72..80] = 0
    // p_vaddr = 0x400000
    elf[80] = 0x00; elf[81] = 0x00; elf[82] = 0x40; elf[83] = 0x00;
    elf[84] = 0; elf[85] = 0; elf[86] = 0; elf[87] = 0;
    // p_paddr = 0x400000
    elf[88] = 0x00; elf[89] = 0x00; elf[90] = 0x40; elf[91] = 0x00;
    elf[92] = 0; elf[93] = 0; elf[94] = 0; elf[95] = 0;
    // p_filesz = 183 (total file)
    elf[96] = 183; elf[97] = 0; elf[98] = 0; elf[99] = 0;
    elf[100] = 0; elf[101] = 0; elf[102] = 0; elf[103] = 0;
    // p_memsz = 183
    elf[104] = 183; elf[105] = 0; elf[106] = 0; elf[107] = 0;
    elf[108] = 0; elf[109] = 0; elf[110] = 0; elf[111] = 0;
    // p_align = 0x1000 (4096)
    elf[112] = 0x00; elf[113] = 0x10; elf[114] = 0; elf[115] = 0;
    elf[116] = 0; elf[117] = 0; elf[118] = 0; elf[119] = 0;
    
    // ── Code (starts at offset 120 = 0x78, vaddr 0x400078) ──
    // mov rax, 1 (SYS_WRITE)
    elf[120] = 0x48; elf[121] = 0xC7; elf[122] = 0xC0;
    elf[123] = 0x01; elf[124] = 0x00; elf[125] = 0x00; elf[126] = 0x00;
    // mov rdi, 1 (stdout)
    elf[127] = 0x48; elf[128] = 0xC7; elf[129] = 0xC7;
    elf[130] = 0x01; elf[131] = 0x00; elf[132] = 0x00; elf[133] = 0x00;
    // lea rsi, [rip + 0x17] → points to message at offset 164 (vaddr 0x4000A4)
    // Next instr at offset 141, message at 164, disp = 164 - 141 = 23 = 0x17
    elf[134] = 0x48; elf[135] = 0x8D; elf[136] = 0x35;
    elf[137] = 0x17; elf[138] = 0x00; elf[139] = 0x00; elf[140] = 0x00;
    // mov rdx, 19 (count)
    elf[141] = 0x48; elf[142] = 0xC7; elf[143] = 0xC2;
    elf[144] = 0x13; elf[145] = 0x00; elf[146] = 0x00; elf[147] = 0x00;
    // syscall
    elf[148] = 0x0F; elf[149] = 0x05;
    // mov rax, 60 (SYS_EXIT)
    elf[150] = 0x48; elf[151] = 0xC7; elf[152] = 0xC0;
    elf[153] = 0x3C; elf[154] = 0x00; elf[155] = 0x00; elf[156] = 0x00;
    // xor rdi, rdi (exit code 0)
    elf[157] = 0x48; elf[158] = 0x31; elf[159] = 0xFF;
    // syscall
    elf[160] = 0x0F; elf[161] = 0x05;
    // jmp $ (safety loop)
    elf[162] = 0xEB; elf[163] = 0xFE;
    // "Hello from Ring 3!\n" (19 bytes, offset 164 = 0xA4)
    elf[164] = b'H'; elf[165] = b'e'; elf[166] = b'l'; elf[167] = b'l';
    elf[168] = b'o'; elf[169] = b' '; elf[170] = b'f'; elf[171] = b'r';
    elf[172] = b'o'; elf[173] = b'm'; elf[174] = b' '; elf[175] = b'R';
    elf[176] = b'i'; elf[177] = b'n'; elf[178] = b'g'; elf[179] = b' ';
    elf[180] = b'3'; elf[181] = b'!'; elf[182] = b'\n';
    
    elf
};

/// Execute the embedded hello world ELF binary
pub fn exec_hello_elf() -> ExecResult {
    crate::log!("[EXEC] Running embedded hello world ELF...");
    exec_bytes(HELLO_ELF, &[])
}

/// Execute a Ring 3 memory-management test program
///
/// Tests from user-space:
/// 1. brk(0) to query current break
/// 2. brk(break + 0x1000) to extend heap by one page
/// 3. Write / read-back on newly mapped heap page
/// 4. mmap(0, 4096, PROT_RW, MAP_PRIVATE|MAP_ANON, -1, 0)
/// 5. Write / read-back on mmap'd page
/// 6. Prints "v0.3 OK\n" and exit(0) on success, "FAIL\n" and exit(1) on failure
pub fn exec_memtest() -> ExecResult {
    crate::log!("[EXEC] Running v0.3 memory test in Ring 3...");

    // Hand-assembled x86-64 machine code (194 bytes)
    //
    // Layout (all offsets from code start at vaddr 0x400000):
    //   0..29   — brk(0), save break in r12, brk(r12+0x1000)
    //  29..45   — write 0x42 to [r12], verify, jne fail
    //  45..80   — mmap(0, 0x1000, 3, 0x22, -1, 0)
    //  80..108  — check result, write 0x99 to [mmap'd], verify, jne fail
    // 108..143  — success: write "v0.3 OK\n", exit(0), jmp $
    // 143..181  — fail:   write "FAIL\n", exit(1), jmp $
    // 181..189  — "v0.3 OK\n"
    // 189..194  — "FAIL\n"
    let memtest_code: [u8; 194] = [
        // === brk(0) → r12 ===
        0xB8, 0x0C, 0x00, 0x00, 0x00,                     // mov eax, 12  (SYS_BRK)
        0x31, 0xFF,                                         // xor edi, edi
        0x0F, 0x05,                                         // syscall
        0x49, 0x89, 0xC4,                                   // mov r12, rax

        // === brk(r12 + 0x1000) ===
        0x4C, 0x89, 0xE7,                                   // mov rdi, r12
        0x48, 0x81, 0xC7, 0x00, 0x10, 0x00, 0x00,         // add rdi, 0x1000
        0xB8, 0x0C, 0x00, 0x00, 0x00,                     // mov eax, 12
        0x0F, 0x05,                                         // syscall

        // === Heap write / verify ===
        0x41, 0xC6, 0x04, 0x24, 0x42,                     // mov byte [r12], 0x42
        0x41, 0x80, 0x3C, 0x24, 0x42,                     // cmp byte [r12], 0x42
        0x0F, 0x85, 0x62, 0x00, 0x00, 0x00,               // jne fail (+98)

        // === mmap(0, 0x1000, PROT_RW=3, MAP_PRIV|ANON=0x22, fd=-1, off=0) ===
        0xB8, 0x09, 0x00, 0x00, 0x00,                     // mov eax, 9  (SYS_MMAP)
        0x31, 0xFF,                                         // xor edi, edi
        0xBE, 0x00, 0x10, 0x00, 0x00,                     // mov esi, 0x1000
        0xBA, 0x03, 0x00, 0x00, 0x00,                     // mov edx, 3
        0x41, 0xBA, 0x22, 0x00, 0x00, 0x00,               // mov r10d, 0x22
        0x49, 0xC7, 0xC0, 0xFF, 0xFF, 0xFF, 0xFF,         // mov r8, -1
        0x45, 0x31, 0xC9,                                   // xor r9d, r9d
        0x0F, 0x05,                                         // syscall

        // === Check mmap result ===
        0x48, 0x85, 0xC0,                                   // test rax, rax
        0x0F, 0x88, 0x36, 0x00, 0x00, 0x00,               // js fail (+54)

        // === Mmap write / verify ===
        0x49, 0x89, 0xC5,                                   // mov r13, rax
        0x41, 0xC6, 0x45, 0x00, 0x99,                     // mov byte [r13+0], 0x99
        0x41, 0x80, 0x7D, 0x00, 0x99,                     // cmp byte [r13+0], 0x99
        0x0F, 0x85, 0x23, 0x00, 0x00, 0x00,               // jne fail (+35)

        // === Success path ===
        0xB8, 0x01, 0x00, 0x00, 0x00,                     // mov eax, 1  (SYS_WRITE)
        0xBF, 0x01, 0x00, 0x00, 0x00,                     // mov edi, 1  (stdout)
        0x48, 0x8D, 0x35, 0x38, 0x00, 0x00, 0x00,         // lea rsi, [rip+56] → msg_ok
        0xBA, 0x08, 0x00, 0x00, 0x00,                     // mov edx, 8
        0x0F, 0x05,                                         // syscall
        0xB8, 0x3C, 0x00, 0x00, 0x00,                     // mov eax, 60 (SYS_EXIT)
        0x31, 0xFF,                                         // xor edi, edi
        0x0F, 0x05,                                         // syscall
        0xEB, 0xFE,                                         // jmp $

        // === Fail path (offset 143) ===
        0xB8, 0x01, 0x00, 0x00, 0x00,                     // mov eax, 1
        0xBF, 0x01, 0x00, 0x00, 0x00,                     // mov edi, 1
        0x48, 0x8D, 0x35, 0x1D, 0x00, 0x00, 0x00,         // lea rsi, [rip+29] → msg_fail
        0xBA, 0x05, 0x00, 0x00, 0x00,                     // mov edx, 5
        0x0F, 0x05,                                         // syscall
        0xB8, 0x3C, 0x00, 0x00, 0x00,                     // mov eax, 60
        0xBF, 0x01, 0x00, 0x00, 0x00,                     // mov edi, 1
        0x0F, 0x05,                                         // syscall
        0xEB, 0xFE,                                         // jmp $

        // === Data (offset 181) ===
        b'v', b'0', b'.', b'3', b' ', b'O', b'K', b'\n', // "v0.3 OK\n" (8)
        b'F', b'A', b'I', b'L', b'\n',                     // "FAIL\n"   (5)
    ];

    // Use the same approach as exec_test_program — map code at 0x400000

    let mut address_space = match AddressSpace::new_with_kernel() {
        Some(a) => a,
        None => {
            crate::log_error!("[EXEC] Failed to create address space");
            return ExecResult::MemoryError;
        }
    };

    let hhdm = hhdm_offset();

    // Map code page at 0x400000
    let code_vaddr: u64 = 0x400000;
    let code_phys = match alloc_physical_page() {
        Some(p) => p,
        None => return ExecResult::MemoryError,
    };
    unsafe {
        let dest = (code_phys + hhdm) as *mut u8;
        core::ptr::write_bytes(dest, 0, 4096);
        core::ptr::copy_nonoverlapping(memtest_code.as_ptr(), dest, memtest_code.len());
    }
    if address_space.map_page(code_vaddr, code_phys, PageFlags::USER_CODE).is_none() {
        return ExecResult::MemoryError;
    }

    // Allocate stack (4 pages + guard)
    let stack_top: u64 = 0x7FFFFFFF0000;
    let stack_pages = 4;
    for i in 0..stack_pages {
        let vaddr = stack_top - (i as u64 + 1) * 4096;
        let phys = match alloc_physical_page() {
            Some(p) => p,
            None => return ExecResult::MemoryError,
        };
        unsafe { core::ptr::write_bytes((phys + hhdm) as *mut u8, 0, 4096); }
        if address_space.map_page(vaddr, phys, PageFlags::USER_DATA).is_none() {
            return ExecResult::MemoryError;
        }
    }

    let user_stack = stack_top - 8;

    crate::log!("[EXEC] memtest: code at {:#x}, stack at {:#x}", code_vaddr, user_stack);

    unsafe {
        let kernel_cr3: u64;
        core::arch::asm!("mov {}, cr3", out(reg) kernel_cr3);

        // Publish process context
        CURRENT_USER_SPACE.store(&mut address_space as *mut AddressSpace, Ordering::Release);
        CURRENT_USER_BRK.store(UserMemoryRegion::HEAP_START, Ordering::SeqCst);
        let stack_bottom = stack_top - (stack_pages as u64 * 4096);
        CURRENT_USER_STACK_BOTTOM.store(stack_bottom, Ordering::SeqCst);

        address_space.activate();
        let exit_code = crate::userland::exec_ring3_process(code_vaddr, user_stack);

        CURRENT_USER_SPACE.store(core::ptr::null_mut(), Ordering::Release);
        core::arch::asm!("mov cr3, {}", in(reg) kernel_cr3, options(nostack, preserves_flags));

        crate::log!("[EXEC] memtest exited with code {}", exit_code);
        ExecResult::Exited(exit_code)
    }
}

/// Allocate a physical page (returns page-aligned physical address)
fn alloc_physical_page() -> Option<u64> {
    crate::memory::frame::alloc_frame_zeroed()
}

/// Execute a Ring 3 IPC pipe test
///
/// Creates a pipe via pipe2(), writes "PIPE" to it, reads it back,
/// and verifies the data matches. Prints "IPC OK\n" on success.
/// Returns exit code 0 on success, 1 on failure.
pub fn exec_pipe_test() -> ExecResult {
    crate::log!("[EXEC] Running Ring 3 IPC pipe test...");

    let mut address_space = match AddressSpace::new_with_kernel() {
        Some(a) => a,
        None => {
            crate::log_error!("[EXEC] Failed to create address space");
            return ExecResult::MemoryError;
        }
    };

    let hhdm = hhdm_offset();

    // Machine code for the pipe test (138 bytes):
    //
    //   sub rsp, 16                       ; stack: [rsp+0]=fds, [rsp+8]=data, [rsp+12]=readbuf
    //   mov dword [rsp+8], "PIPE"         ; test payload
    //
    //   ; pipe2(rsp, 0)
    //   mov rdi, rsp
    //   xor esi, esi
    //   mov eax, 293
    //   syscall
    //   test eax, eax
    //   jnz fail
    //
    //   ; write(write_fd, &"PIPE", 4)
    //   mov edi, [rsp+4]
    //   lea rsi, [rsp+8]
    //   mov edx, 4
    //   mov eax, 1
    //   syscall
    //   cmp eax, 4
    //   jne fail
    //
    //   ; read(read_fd, buf, 4)
    //   mov edi, [rsp]
    //   lea rsi, [rsp+12]
    //   mov edx, 4
    //   xor eax, eax
    //   syscall
    //   cmp eax, 4
    //   jne fail
    //
    //   ; compare read data with original
    //   mov eax, [rsp+12]
    //   cmp eax, [rsp+8]
    //   jne fail
    //
    //   ; write(1, "IPC OK\n", 7) then exit(0)
    //   ; fail: exit(1)
    //
    let pipe_test_code: [u8; 138] = [
        // sub rsp, 16
        0x48, 0x83, 0xEC, 0x10,
        // mov dword [rsp+8], 0x45504950 ("PIPE")
        0xC7, 0x44, 0x24, 0x08, 0x50, 0x49, 0x50, 0x45,
        // pipe2(rsp, 0) — syscall 293
        0x48, 0x89, 0xE7,                               // mov rdi, rsp
        0x31, 0xF6,                                     // xor esi, esi
        0xB8, 0x25, 0x01, 0x00, 0x00,                   // mov eax, 293
        0x0F, 0x05,                                     // syscall
        // test eax, eax / jnz fail
        0x85, 0xC0,
        0x75, 0x5B,
        // write(fd_write=[rsp+4], &[rsp+8], 4)
        0x8B, 0x7C, 0x24, 0x04,                         // mov edi, [rsp+4]
        0x48, 0x8D, 0x74, 0x24, 0x08,                   // lea rsi, [rsp+8]
        0xBA, 0x04, 0x00, 0x00, 0x00,                   // mov edx, 4
        0xB8, 0x01, 0x00, 0x00, 0x00,                   // mov eax, 1
        0x0F, 0x05,                                     // syscall
        // cmp eax, 4 / jne fail
        0x83, 0xF8, 0x04,
        0x75, 0x41,
        // read(fd_read=[rsp+0], &[rsp+12], 4)
        0x8B, 0x3C, 0x24,                               // mov edi, [rsp]
        0x48, 0x8D, 0x74, 0x24, 0x0C,                   // lea rsi, [rsp+12]
        0xBA, 0x04, 0x00, 0x00, 0x00,                   // mov edx, 4
        0x31, 0xC0,                                     // xor eax, eax
        0x0F, 0x05,                                     // syscall
        // cmp eax, 4 / jne fail
        0x83, 0xF8, 0x04,
        0x75, 0x2B,
        // compare [rsp+12] with [rsp+8]
        0x8B, 0x44, 0x24, 0x0C,                         // mov eax, [rsp+12]
        0x3B, 0x44, 0x24, 0x08,                         // cmp eax, [rsp+8]
        // jne fail
        0x75, 0x21,
        // write(1, "IPC OK\n", 7)
        0xB8, 0x01, 0x00, 0x00, 0x00,                   // mov eax, 1 (SYS_WRITE)
        0xBF, 0x01, 0x00, 0x00, 0x00,                   // mov edi, 1 (stdout)
        0x48, 0x8D, 0x35, 0x1C, 0x00, 0x00, 0x00,       // lea rsi, [rip+0x1C]
        0xBA, 0x07, 0x00, 0x00, 0x00,                   // mov edx, 7
        0x0F, 0x05,                                     // syscall
        // exit(0)
        0x31, 0xFF,                                     // xor edi, edi
        0xB8, 0x3C, 0x00, 0x00, 0x00,                   // mov eax, 60
        0x0F, 0x05,                                     // syscall
        // .fail: exit(1)
        0xBF, 0x01, 0x00, 0x00, 0x00,                   // mov edi, 1
        0xB8, 0x3C, 0x00, 0x00, 0x00,                   // mov eax, 60
        0x0F, 0x05,                                     // syscall
        // "IPC OK\n"
        b'I', b'P', b'C', b' ', b'O', b'K', b'\n',
    ];

    // Map code page at 0x400000
    let code_vaddr: u64 = 0x400000;
    let code_phys = match alloc_physical_page() {
        Some(p) => p,
        None => return ExecResult::MemoryError,
    };
    unsafe {
        let dest = (code_phys + hhdm) as *mut u8;
        core::ptr::write_bytes(dest, 0, 4096);
        core::ptr::copy_nonoverlapping(pipe_test_code.as_ptr(), dest, pipe_test_code.len());
    }
    if address_space.map_page(code_vaddr, code_phys, PageFlags::USER_CODE).is_none() {
        return ExecResult::MemoryError;
    }

    // Allocate stack (4 pages + guard)
    let stack_top: u64 = 0x7FFFFFFF0000;
    let stack_pages = 4;
    for i in 0..stack_pages {
        let vaddr = stack_top - (i as u64 + 1) * 4096;
        let phys = match alloc_physical_page() {
            Some(p) => p,
            None => return ExecResult::MemoryError,
        };
        unsafe { core::ptr::write_bytes((phys + hhdm) as *mut u8, 0, 4096); }
        if address_space.map_page(vaddr, phys, PageFlags::USER_DATA).is_none() {
            return ExecResult::MemoryError;
        }
    }

    let user_stack = stack_top - 8;

    crate::log!("[EXEC] pipe_test: code at {:#x}, stack at {:#x}", code_vaddr, user_stack);

    unsafe {
        let kernel_cr3: u64;
        core::arch::asm!("mov {}, cr3", out(reg) kernel_cr3);

        CURRENT_USER_SPACE.store(&mut address_space as *mut AddressSpace, Ordering::Release);
        CURRENT_USER_BRK.store(UserMemoryRegion::HEAP_START, Ordering::SeqCst);
        let stack_bottom = stack_top - (stack_pages as u64 * 4096);
        CURRENT_USER_STACK_BOTTOM.store(stack_bottom, Ordering::SeqCst);

        address_space.activate();
        let exit_code = crate::userland::exec_ring3_process(code_vaddr, user_stack);

        CURRENT_USER_SPACE.store(core::ptr::null_mut(), Ordering::Release);
        core::arch::asm!("mov cr3, {}", in(reg) kernel_cr3, options(nostack, preserves_flags));

        crate::log!("[EXEC] pipe_test exited with code {}", exit_code);
        ExecResult::Exited(exit_code)
    }
}

// ============================================================================
// Gap #4/#5 Integration Tests
// ============================================================================

/// Test Ring 3 exception safety: execute UD2 (invalid opcode) in user mode.
/// The kernel should NOT panic — it should catch the fault and kill the process
/// with exit code -4 (SIGILL).
pub fn exec_exception_safety_test() -> ExecResult {
    crate::log!("[EXEC] Running exception safety test (UD2 in Ring 3)...");

    let mut address_space = match AddressSpace::new_with_kernel() {
        Some(a) => a,
        None => return ExecResult::MemoryError,
    };
    let hhdm = hhdm_offset();

    // Machine code: just UD2 (invalid opcode) then safety loop
    //   ud2            ; 0F 0B — generates #UD exception
    //   jmp $          ; EB FE — safety (never reached)
    let code: [u8; 4] = [
        0x0F, 0x0B,  // ud2
        0xEB, 0xFE,  // jmp $
    ];

    let code_vaddr: u64 = 0x400000;
    let code_phys = match alloc_physical_page() {
        Some(p) => p,
        None => return ExecResult::MemoryError,
    };
    unsafe {
        let dest = (code_phys + hhdm) as *mut u8;
        core::ptr::write_bytes(dest, 0, 4096);
        core::ptr::copy_nonoverlapping(code.as_ptr(), dest, code.len());
    }
    if address_space.map_page(code_vaddr, code_phys, PageFlags::USER_CODE).is_none() {
        return ExecResult::MemoryError;
    }

    let stack_top: u64 = 0x7FFFFFFF0000;
    let stack_pages = 4usize;
    for i in 0..stack_pages {
        let vaddr = stack_top - (i as u64 + 1) * 4096;
        let phys = match alloc_physical_page() { Some(p) => p, None => return ExecResult::MemoryError };
        unsafe { core::ptr::write_bytes((phys + hhdm) as *mut u8, 0, 4096); }
        if address_space.map_page(vaddr, phys, PageFlags::USER_DATA).is_none() {
            return ExecResult::MemoryError;
        }
    }
    let user_stack = stack_top - 8;

    unsafe {
        let kernel_cr3: u64;
        core::arch::asm!("mov {}, cr3", out(reg) kernel_cr3);
        CURRENT_USER_SPACE.store(&mut address_space as *mut AddressSpace, Ordering::Release);
        CURRENT_USER_BRK.store(UserMemoryRegion::HEAP_START, Ordering::SeqCst);
        CURRENT_USER_STACK_BOTTOM.store(stack_top - (stack_pages as u64 * 4096), Ordering::SeqCst);
        address_space.activate();
        let exit_code = crate::userland::exec_ring3_process(code_vaddr, user_stack);
        CURRENT_USER_SPACE.store(core::ptr::null_mut(), Ordering::Release);
        core::arch::asm!("mov cr3, {}", in(reg) kernel_cr3, options(nostack, preserves_flags));
        crate::log!("[EXEC] exception_safety_test exited with code {}", exit_code);
        ExecResult::Exited(exit_code)
    }
}

/// Test signal syscalls from Ring 3: sigprocmask, kill(pid,0).
/// Prints "SIG OK\n" on success, "SIG FAIL\n" on failure.
pub fn exec_signal_test() -> ExecResult {
    crate::log!("[EXEC] Running signal syscall test...");

    let mut address_space = match AddressSpace::new_with_kernel() {
        Some(a) => a,
        None => return ExecResult::MemoryError,
    };
    let hhdm = hhdm_offset();

    // Machine code:
    //   getpid() → save pid in r12
    //   rt_sigprocmask(SIG_SETMASK=2, &zero, &old, 8) → check 0
    //   kill(pid, 0) → check 0  (existence check)
    //   write(1, "SIG OK\n", 7)
    //   exit(0)
    //   --- fail path ---
    //   write(1, "SIG FAIL\n", 9)
    //   exit(1)
    #[rustfmt::skip]
    let code: [u8; 158] = [
        // 0x00: getpid
        0xB8, 0x27, 0x00, 0x00, 0x00,              // mov eax, 39
        0x0F, 0x05,                                  // syscall
        0x49, 0x89, 0xC4,                            // mov r12, rax
        // 0x0A: sub rsp, 16 (space for mask + old)
        0x48, 0x83, 0xEC, 0x10,                      // sub rsp, 16
        0x48, 0xC7, 0x04, 0x24, 0x00, 0x00, 0x00, 0x00, // mov qword [rsp], 0
        // 0x16: rt_sigprocmask(SIG_SETMASK=2, &mask, &old, 8)
        0xB8, 0x0E, 0x00, 0x00, 0x00,              // mov eax, 14
        0xBF, 0x02, 0x00, 0x00, 0x00,              // mov edi, 2
        0x48, 0x8D, 0x34, 0x24,                     // lea rsi, [rsp]
        0x48, 0x8D, 0x54, 0x24, 0x08,              // lea rdx, [rsp+8]
        0x41, 0xBA, 0x08, 0x00, 0x00, 0x00,        // mov r10d, 8
        0x0F, 0x05,                                  // syscall
        0x85, 0xC0,                                  // test eax, eax
        0x75, 0x33,                                  // jnz fail (0x68)
        // 0x35: kill(pid, 0)
        0xB8, 0x3E, 0x00, 0x00, 0x00,              // mov eax, 62
        0x44, 0x89, 0xE7,                            // mov edi, r12d
        0x31, 0xF6,                                  // xor esi, esi
        0x0F, 0x05,                                  // syscall
        0x85, 0xC0,                                  // test eax, eax
        0x75, 0x23,                                  // jnz fail (0x68)
        // 0x45: write(1, "SIG OK\n", 7)
        0xB8, 0x01, 0x00, 0x00, 0x00,              // mov eax, 1
        0xBF, 0x01, 0x00, 0x00, 0x00,              // mov edi, 1
        0x48, 0x8D, 0x35, 0x38, 0x00, 0x00, 0x00,  // lea rsi, [rip+0x38] → msg
        0xBA, 0x07, 0x00, 0x00, 0x00,              // mov edx, 7
        0x0F, 0x05,                                  // syscall
        // 0x5D: exit(0)
        0xB8, 0x3C, 0x00, 0x00, 0x00,              // mov eax, 60
        0x31, 0xFF,                                  // xor edi, edi
        0x0F, 0x05,                                  // syscall
        0xEB, 0xFE,                                  // jmp $
        // 0x68: fail → write(1, "SIG FAIL\n", 9)
        0xB8, 0x01, 0x00, 0x00, 0x00,              // mov eax, 1
        0xBF, 0x01, 0x00, 0x00, 0x00,              // mov edi, 1
        0x48, 0x8D, 0x35, 0x1C, 0x00, 0x00, 0x00,  // lea rsi, [rip+0x1C] → fail_msg
        0xBA, 0x09, 0x00, 0x00, 0x00,              // mov edx, 9
        0x0F, 0x05,                                  // syscall
        0xB8, 0x3C, 0x00, 0x00, 0x00,              // mov eax, 60
        0xBF, 0x01, 0x00, 0x00, 0x00,              // mov edi, 1
        0x0F, 0x05,                                  // syscall
        0xEB, 0xFE,                                  // jmp $
        // 0x8E: data
        b'S', b'I', b'G', b' ', b'O', b'K', b'\n',              // "SIG OK\n"
        b'S', b'I', b'G', b' ', b'F', b'A', b'I', b'L', b'\n', // "SIG FAIL\n"
    ];

    let code_vaddr: u64 = 0x400000;
    let code_phys = match alloc_physical_page() {
        Some(p) => p,
        None => return ExecResult::MemoryError,
    };
    unsafe {
        let dest = (code_phys + hhdm) as *mut u8;
        core::ptr::write_bytes(dest, 0, 4096);
        core::ptr::copy_nonoverlapping(code.as_ptr(), dest, code.len());
    }
    if address_space.map_page(code_vaddr, code_phys, PageFlags::USER_CODE).is_none() {
        return ExecResult::MemoryError;
    }

    let stack_top: u64 = 0x7FFFFFFF0000;
    let stack_pages = 4usize;
    for i in 0..stack_pages {
        let vaddr = stack_top - (i as u64 + 1) * 4096;
        let phys = match alloc_physical_page() { Some(p) => p, None => return ExecResult::MemoryError };
        unsafe { core::ptr::write_bytes((phys + hhdm) as *mut u8, 0, 4096); }
        if address_space.map_page(vaddr, phys, PageFlags::USER_DATA).is_none() {
            return ExecResult::MemoryError;
        }
    }
    let user_stack = stack_top - 8;

    // Spawn a proper process so getpid() > 0 and signals are initialized
    let pid = crate::process::spawn("signal_test").unwrap_or(0);
    let prev_pid = crate::process::current_pid();

    // Set up stdio for the test process
    crate::vfs::setup_stdio();

    let exit_code;
    unsafe {
        let kernel_cr3: u64;
        core::arch::asm!("mov {}, cr3", out(reg) kernel_cr3);
        CURRENT_USER_SPACE.store(&mut address_space as *mut AddressSpace, Ordering::Release);
        CURRENT_USER_BRK.store(UserMemoryRegion::HEAP_START, Ordering::SeqCst);
        CURRENT_USER_STACK_BOTTOM.store(stack_top - (stack_pages as u64 * 4096), Ordering::SeqCst);
        crate::process::start_running(pid);
        address_space.activate();
        exit_code = crate::userland::exec_ring3_process(code_vaddr, user_stack);
        CURRENT_USER_SPACE.store(core::ptr::null_mut(), Ordering::Release);
        core::arch::asm!("mov cr3, {}", in(reg) kernel_cr3, options(nostack, preserves_flags));
    }

    crate::process::finish(pid, exit_code);
    crate::process::reap(pid);
    crate::vfs::cleanup_stdio();
    crate::process::set_current(prev_pid);
    crate::log!("[EXEC] signal_test exited with code {}", exit_code);
    ExecResult::Exited(exit_code)
}

/// Test stdio + getpid + clock_gettime from Ring 3.
/// Prints "IO OK\n" on success, "IO FAIL\n" on failure.
pub fn exec_stdio_test() -> ExecResult {
    crate::log!("[EXEC] Running stdio/time test...");

    let mut address_space = match AddressSpace::new_with_kernel() {
        Some(a) => a,
        None => return ExecResult::MemoryError,
    };
    let hhdm = hhdm_offset();

    // Machine code:
    //   getpid() → check > 0
    //   clock_gettime(MONOTONIC=1, &ts) → check == 0
    //   write(1, "IO OK\n", 6)
    //   exit(0)
    //   --- fail path ---
    //   write(1, "IO FAIL\n", 8)
    //   exit(1)
    #[rustfmt::skip]
    let code: [u8; 121] = [
        // 0x00: getpid
        0xB8, 0x27, 0x00, 0x00, 0x00,              // mov eax, 39
        0x0F, 0x05,                                  // syscall
        0x85, 0xC0,                                  // test eax, eax
        0x74, 0x3A,                                  // jz fail (0x45)
        // 0x0B: clock_gettime(1, &ts)
        0x48, 0x83, 0xEC, 0x10,                      // sub rsp, 16
        0xB8, 0xE4, 0x00, 0x00, 0x00,              // mov eax, 228
        0xBF, 0x01, 0x00, 0x00, 0x00,              // mov edi, 1
        0x48, 0x89, 0xE6,                            // mov rsi, rsp
        0x0F, 0x05,                                  // syscall
        0x85, 0xC0,                                  // test eax, eax
        0x75, 0x23,                                  // jnz fail (0x45)
        // 0x22: write(1, "IO OK\n", 6)
        0xB8, 0x01, 0x00, 0x00, 0x00,              // mov eax, 1
        0xBF, 0x01, 0x00, 0x00, 0x00,              // mov edi, 1
        0x48, 0x8D, 0x35, 0x38, 0x00, 0x00, 0x00,  // lea rsi, [rip+0x38] → msg
        0xBA, 0x06, 0x00, 0x00, 0x00,              // mov edx, 6
        0x0F, 0x05,                                  // syscall
        // 0x3A: exit(0)
        0xB8, 0x3C, 0x00, 0x00, 0x00,              // mov eax, 60
        0x31, 0xFF,                                  // xor edi, edi
        0x0F, 0x05,                                  // syscall
        0xEB, 0xFE,                                  // jmp $
        // 0x45: fail → write(1, "IO FAIL\n", 8)
        0xB8, 0x01, 0x00, 0x00, 0x00,              // mov eax, 1
        0xBF, 0x01, 0x00, 0x00, 0x00,              // mov edi, 1
        0x48, 0x8D, 0x35, 0x1B, 0x00, 0x00, 0x00,  // lea rsi, [rip+0x1B] → fail_msg
        0xBA, 0x08, 0x00, 0x00, 0x00,              // mov edx, 8
        0x0F, 0x05,                                  // syscall
        0xB8, 0x3C, 0x00, 0x00, 0x00,              // mov eax, 60
        0xBF, 0x01, 0x00, 0x00, 0x00,              // mov edi, 1
        0x0F, 0x05,                                  // syscall
        0xEB, 0xFE,                                  // jmp $
        // 0x6B: data
        b'I', b'O', b' ', b'O', b'K', b'\n',                  // "IO OK\n"
        b'I', b'O', b' ', b'F', b'A', b'I', b'L', b'\n',      // "IO FAIL\n"
    ];

    let code_vaddr: u64 = 0x400000;
    let code_phys = match alloc_physical_page() {
        Some(p) => p,
        None => return ExecResult::MemoryError,
    };
    unsafe {
        let dest = (code_phys + hhdm) as *mut u8;
        core::ptr::write_bytes(dest, 0, 4096);
        core::ptr::copy_nonoverlapping(code.as_ptr(), dest, code.len());
    }
    if address_space.map_page(code_vaddr, code_phys, PageFlags::USER_CODE).is_none() {
        return ExecResult::MemoryError;
    }

    let stack_top: u64 = 0x7FFFFFFF0000;
    let stack_pages = 4usize;
    for i in 0..stack_pages {
        let vaddr = stack_top - (i as u64 + 1) * 4096;
        let phys = match alloc_physical_page() { Some(p) => p, None => return ExecResult::MemoryError };
        unsafe { core::ptr::write_bytes((phys + hhdm) as *mut u8, 0, 4096); }
        if address_space.map_page(vaddr, phys, PageFlags::USER_DATA).is_none() {
            return ExecResult::MemoryError;
        }
    }
    let user_stack = stack_top - 8;

    // Spawn a proper process so getpid() > 0
    let pid = crate::process::spawn("stdio_test").unwrap_or(0);
    let prev_pid = crate::process::current_pid();

    // Set up stdio for the test process
    crate::vfs::setup_stdio();

    let exit_code;
    unsafe {
        let kernel_cr3: u64;
        core::arch::asm!("mov {}, cr3", out(reg) kernel_cr3);
        CURRENT_USER_SPACE.store(&mut address_space as *mut AddressSpace, Ordering::Release);
        CURRENT_USER_BRK.store(UserMemoryRegion::HEAP_START, Ordering::SeqCst);
        CURRENT_USER_STACK_BOTTOM.store(stack_top - (stack_pages as u64 * 4096), Ordering::SeqCst);
        crate::process::start_running(pid);
        address_space.activate();
        exit_code = crate::userland::exec_ring3_process(code_vaddr, user_stack);
        CURRENT_USER_SPACE.store(core::ptr::null_mut(), Ordering::Release);
        core::arch::asm!("mov cr3, {}", in(reg) kernel_cr3, options(nostack, preserves_flags));
    }

    crate::process::finish(pid, exit_code);
    crate::process::reap(pid);
    crate::vfs::cleanup_stdio();
    crate::process::set_current(prev_pid);
    crate::log!("[EXEC] stdio_test exited with code {}", exit_code);
    ExecResult::Exited(exit_code)
}

/// Check if a file is an executable ELF
pub fn is_executable(path: &str) -> bool {
    let fd = match crate::vfs::open(path, crate::vfs::OpenFlags(0)) {
        Ok(fd) => fd,
        Err(_) => return false,
    };
    
    let mut magic = [0u8; 4];
    let result = crate::vfs::read(fd, &mut magic).is_ok() 
        && magic == [0x7F, b'E', b'L', b'F'];
    
    crate::vfs::close(fd).ok();
    result
}
