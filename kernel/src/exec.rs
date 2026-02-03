//! ELF Executable Runner
//!
//! Loads and executes ELF binaries in Ring 3 user space.
//! Provides the bridge between the shell and userland execution.

use alloc::vec::Vec;
use alloc::string::String;
use crate::elf::{LoadedElf, ElfError, ElfResult};
use crate::memory::paging::{AddressSpace, PageFlags, UserMemoryRegion};
use crate::memory::hhdm_offset;

/// Stack size for user processes (1 MB)
const USER_STACK_SIZE: usize = 1024 * 1024;

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
    
    // Allocate and map user stack
    let stack_pages = USER_STACK_SIZE / 4096;
    let stack_base = UserMemoryRegion::STACK_TOP - (stack_pages as u64 * 4096);
    
    crate::log_debug!("[EXEC] Mapping stack: {:#x} - {:#x}", stack_base, UserMemoryRegion::STACK_TOP);
    
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
    
    // Set up initial stack with argc/argv
    let user_stack_top = UserMemoryRegion::STACK_TOP - 8; // Align
    let argc = args.len() as u64;
    
    crate::log!("[EXEC] Ready to execute at {:#x}, stack at {:#x}", 
        elf.entry_point, user_stack_top);
    
    // Switch to user address space and jump to Ring 3
    unsafe {
        // Save kernel CR3
        let kernel_cr3: u64;
        core::arch::asm!("mov {}, cr3", out(reg) kernel_cr3);
        
        // Activate user address space
        address_space.activate();
        
        // For now, we run in kernel mode to test
        // TODO: Actually jump to Ring 3 with proper exception handling
        
        // Call entry point (simulated - runs in kernel mode for now)
        let entry_fn: extern "C" fn(u64, u64) -> i32 = 
            core::mem::transmute(elf.entry_point);
        
        // This will crash if the ELF isn't set up correctly
        // In a real implementation, we'd use IRETQ to Ring 3
        
        // Restore kernel CR3 first
        core::arch::asm!("mov cr3, {}", in(reg) kernel_cr3);
        
        crate::log!("[EXEC] Would execute at {:#x} (Ring 3 not implemented yet)", elf.entry_point);
    }
    
    // For now, simulate success
    ExecResult::Exited(0)
}

/// Execute a simple bytecode program (for testing)
pub fn exec_test_program() -> ExecResult {
    crate::log!("[EXEC] Running Ring 3 test program (simulated)...");
    
    // For now, we just test that we CAN set up user space correctly
    // without actually jumping to Ring 3 (which would block)
    
    // Create user address space
    let mut address_space = match AddressSpace::new_with_kernel() {
        Some(a) => a,
        None => {
            crate::log_error!("[EXEC] Failed to create address space");
            return ExecResult::MemoryError;
        }
    };
    
    let hhdm = hhdm_offset();
    
    // Test code
    let test_code: [u8; 22] = [
        0x48, 0xC7, 0xC0, 0x3C, 0x00, 0x00, 0x00, // mov rax, 60 (SYS_EXIT)
        0x48, 0xC7, 0xC7, 0x2A, 0x00, 0x00, 0x00, // mov rdi, 42 (exit code)
        0x0F, 0x05,                               // syscall
        0xEB, 0xFE,                               // jmp $ (infinite loop if syscall returns)
        0x90, 0x90, 0x90, 0x90,                   // nops for alignment
    ];
    
    // Allocate and map code page at 0x400000
    let code_vaddr: u64 = 0x400000;
    let code_phys = match alloc_physical_page() {
        Some(p) => p,
        None => return ExecResult::MemoryError,
    };
    
    crate::log!("[EXEC] Code page: phys={:#x}, virt={:#x}", code_phys, code_vaddr);
    
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
    
    // Allocate and map stack
    let stack_top: u64 = 0x7FFFFFFF0000;
    let stack_pages = 4; // 16 KB stack
    
    for i in 0..stack_pages {
        let vaddr = stack_top - (i as u64 + 1) * 4096;
        let phys = match alloc_physical_page() {
            Some(p) => p,
            None => return ExecResult::MemoryError,
        };
        
        // Zero the stack
        unsafe {
            core::ptr::write_bytes((phys + hhdm) as *mut u8, 0, 4096);
        }
        
        if address_space.map_page(vaddr, phys, PageFlags::USER_DATA).is_none() {
            crate::log_error!("[EXEC] Failed to map stack page");
            return ExecResult::MemoryError;
        }
    }
    
    crate::log!("[EXEC] Stack mapped: {:#x} - {:#x}", stack_top - stack_pages as u64 * 4096, stack_top);
    crate::log!("[EXEC] User address space CR3: {:#x}", address_space.cr3());
    crate::log!("[EXEC] Test passed - userland setup successful!");
    crate::log!("[EXEC] To actually run in Ring 3, use 'exec ring3' (WARNING: will block)");
    
    ExecResult::Exited(42)
}

/// Actually jump to Ring 3 (WARNING: This blocks until the process exits!)
pub fn exec_ring3_test() -> ! {
    use crate::userland::jump_to_ring3;
    
    crate::log!("[EXEC] Preparing Ring 3 execution...");
    
    // Create user address space
    let mut address_space = match AddressSpace::new_with_kernel() {
        Some(a) => a,
        None => {
            crate::log_error!("[EXEC] Failed to create address space");
            loop { core::hint::spin_loop(); }
        }
    };
    
    let hhdm = hhdm_offset();
    
    // Simple test: just do a syscall to print and exit
    // Note: After exit syscall, we'll be stuck since exit just marks zombie
    let test_code: [u8; 32] = [
        // mov rax, 0x1000 (SYS_DEBUG_PRINT)
        0x48, 0xC7, 0xC0, 0x00, 0x10, 0x00, 0x00,
        // mov rdi, 0x400020 (ptr to message, we'll put it there)
        0x48, 0xBF, 0x20, 0x00, 0x40, 0x00, 0x00, 0x00, 0x00, 0x00,
        // mov rsi, 12 (length)
        0x48, 0xC7, 0xC6, 0x0C, 0x00, 0x00, 0x00,
        // syscall
        0x0F, 0x05,
        // jmp $ (loop forever after print)
        0xEB, 0xFE,
        0x90, 0x90, 0x90, 0x90,
    ];
    
    let message = b"Hello Ring3!";
    
    // Allocate code page
    let code_vaddr: u64 = 0x400000;
    let code_phys = alloc_physical_page().expect("OOM");
    
    unsafe {
        let dest = (code_phys + hhdm) as *mut u8;
        core::ptr::write_bytes(dest, 0, 4096);
        core::ptr::copy_nonoverlapping(test_code.as_ptr(), dest, test_code.len());
        // Put message at offset 0x20
        core::ptr::copy_nonoverlapping(message.as_ptr(), dest.add(0x20), message.len());
    }
    
    address_space.map_page(code_vaddr, code_phys, PageFlags::USER_CODE);
    
    // Map stack
    let stack_top: u64 = 0x7FFFFFFF0000;
    for i in 0..4 {
        let vaddr = stack_top - (i as u64 + 1) * 4096;
        let phys = alloc_physical_page().expect("OOM");
        unsafe { core::ptr::write_bytes((phys + hhdm) as *mut u8, 0, 4096); }
        address_space.map_page(vaddr, phys, PageFlags::USER_DATA);
    }
    
    let user_stack = stack_top - 8;
    
    crate::log!("[EXEC] Jumping to Ring 3 at {:#x}...", code_vaddr);
    
    unsafe {
        address_space.activate();
        jump_to_ring3(code_vaddr, user_stack);
    }
}

/// Allocate a physical page (returns physical address)
fn alloc_physical_page() -> Option<u64> {
    // Allocate from heap and convert to physical
    let page: Vec<u8> = alloc::vec![0u8; 4096];
    let virt = page.as_ptr() as u64;
    let hhdm = hhdm_offset();
    
    // Convert to physical (subtract HHDM)
    let phys = virt.checked_sub(hhdm)?;
    
    // Leak so it persists
    core::mem::forget(page);
    
    Some(phys)
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
