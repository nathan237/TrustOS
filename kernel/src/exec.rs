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
    
    // Switch to user address space and jump to Ring 3
    unsafe {
        // Save kernel CR3
        let kernel_cr3: u64;
        core::arch::asm!("mov {}, cr3", out(reg) kernel_cr3);
        
        // Publish process context for syscall / page-fault handlers
        CURRENT_USER_SPACE.store(&mut address_space as *mut AddressSpace, Ordering::Release);
        CURRENT_USER_BRK.store(UserMemoryRegion::HEAP_START, Ordering::SeqCst);
        CURRENT_USER_STACK_BOTTOM.store(stack_base, Ordering::SeqCst);
        
        // Activate user address space
        address_space.activate();
        
        crate::log!("[EXEC] Entering Ring 3 at {:#x}...", elf.entry_point);
        
        // Jump to Ring 3 — returns when user process calls exit()
        let exit_code = crate::userland::exec_ring3_process(elf.entry_point, sp);
        
        // Clear process context
        CURRENT_USER_SPACE.store(core::ptr::null_mut(), Ordering::Release);
        
        // Restore kernel CR3 (safety — return_from_ring3 already does this)
        core::arch::asm!("mov cr3, {}", in(reg) kernel_cr3, options(nostack, preserves_flags));
        
        crate::log!("[EXEC] Process exited with code {}", exit_code);
        
        ExecResult::Exited(exit_code)
    }
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

/// Allocate a physical page (returns page-aligned physical address)
fn alloc_physical_page() -> Option<u64> {
    crate::memory::frame::alloc_frame_zeroed()
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

/// List executable files in a directory
pub fn list_executables(dir: &str) -> Vec<String> {
    let mut execs = Vec::new();
    
    crate::ramfs::with_fs(|fs| {
        if let Ok(entries) = fs.ls(Some(dir)) {
            for entry in entries {
                let full_path = if dir == "/" {
                    alloc::format!("/{}", entry.0)
                } else {
                    alloc::format!("{}/{}", dir, entry.0)
                };
                
                if is_executable(&full_path) {
                    execs.push(full_path);
                }
            }
        }
    });
    
    execs
}
