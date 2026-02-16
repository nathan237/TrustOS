//! GDT - Global Descriptor Table for x86_64
//!
//! Implements proper segmentation with Ring 0/3 separation and TSS.
//!
//! GDT Layout (for SYSRET compatibility):
//! - 0x00: Null descriptor
//! - 0x08: Kernel Code (Ring 0)
//! - 0x10: Kernel Data (Ring 0)
//! - 0x18: User Data (Ring 3) ← Must be before User Code for SYSRET!
//! - 0x20: User Code (Ring 3)
//! - 0x28: TSS (16 bytes)
//!
//! SYSRET 64-bit loads:
//! - CS = STAR[63:48] + 16 (with RPL=3 forced)
//! - SS = STAR[63:48] + 8  (with RPL=3 forced)
//! So if STAR[63:48] = 0x10, then CS=0x20 (user code), SS=0x18 (user data) ✓

use core::mem::size_of;

/// Segment selector for kernel code (Ring 0)
pub const KERNEL_CODE_SELECTOR: u16 = 0x08;
/// Segment selector for kernel data (Ring 0)
pub const KERNEL_DATA_SELECTOR: u16 = 0x10;
/// Segment selector for user data (Ring 3) - BEFORE user code for SYSRET!
pub const USER_DATA_SELECTOR: u16 = 0x18 | 3; // RPL = 3
/// Segment selector for user code (Ring 3)
pub const USER_CODE_SELECTOR: u16 = 0x20 | 3; // RPL = 3
/// TSS selector
pub const TSS_SELECTOR: u16 = 0x28;

/// GDT entry (64-bit mode)
#[derive(Clone, Copy, Debug)]
#[repr(C, packed)]
pub struct GdtEntry {
    limit_low: u16,
    base_low: u16,
    base_middle: u8,
    access: u8,
    granularity: u8,
    base_high: u8,
}

impl GdtEntry {
    pub const fn null() -> Self {
        Self {
            limit_low: 0,
            base_low: 0,
            base_middle: 0,
            access: 0,
            granularity: 0,
            base_high: 0,
        }
    }
    
    /// Create a code segment descriptor
    pub const fn code_segment(ring: u8) -> Self {
        let access = if ring == 0 {
            0x9A // Present + DPL 0 + Code + Readable
        } else {
            0xFA // Present + DPL 3 + Code + Readable
        };
        
        Self {
            limit_low: 0xFFFF,
            base_low: 0,
            base_middle: 0,
            access,
            granularity: 0xAF, // Long mode + 4K granularity + limit high
            base_high: 0,
        }
    }
    
    /// Create a data segment descriptor
    pub const fn data_segment(ring: u8) -> Self {
        let access = if ring == 0 {
            0x92 // Present + DPL 0 + Data + Writable
        } else {
            0xF2 // Present + DPL 3 + Data + Writable
        };
        
        Self {
            limit_low: 0xFFFF,
            base_low: 0,
            base_middle: 0,
            access,
            granularity: 0xCF, // 4K granularity + limit high
            base_high: 0,
        }
    }
}

/// TSS entry in GDT (16 bytes for 64-bit TSS)
#[derive(Clone, Copy, Debug)]
#[repr(C, packed)]
pub struct TssEntry {
    length: u16,
    base_low: u16,
    base_middle: u8,
    flags1: u8,
    flags2: u8,
    base_high: u8,
    base_upper: u32,
    reserved: u32,
}

impl TssEntry {
    pub const fn null() -> Self {
        Self {
            length: 0,
            base_low: 0,
            base_middle: 0,
            flags1: 0,
            flags2: 0,
            base_high: 0,
            base_upper: 0,
            reserved: 0,
        }
    }
    
    /// Create TSS descriptor
    pub fn new(tss_addr: u64) -> Self {
        let base = tss_addr;
        let limit = (size_of::<TaskStateSegment>() - 1) as u16;
        
        Self {
            length: limit,
            base_low: base as u16,
            base_middle: (base >> 16) as u8,
            flags1: 0x89, // Present + 64-bit TSS (available)
            flags2: 0x00,
            base_high: (base >> 24) as u8,
            base_upper: (base >> 32) as u32,
            reserved: 0,
        }
    }
}

/// Task State Segment (64-bit)
#[derive(Clone, Copy, Debug)]
#[repr(C, packed)]
pub struct TaskStateSegment {
    reserved1: u32,
    /// Stack pointers for privilege levels 0-2
    pub rsp: [u64; 3],
    reserved2: u64,
    /// Interrupt Stack Table pointers
    pub ist: [u64; 7],
    reserved3: u64,
    reserved4: u16,
    /// I/O map base address
    pub iomap_base: u16,
}

impl TaskStateSegment {
    pub const fn new() -> Self {
        Self {
            reserved1: 0,
            rsp: [0; 3],
            reserved2: 0,
            ist: [0; 7],
            reserved3: 0,
            reserved4: 0,
            iomap_base: size_of::<TaskStateSegment>() as u16,
        }
    }
}

/// Complete GDT structure
/// Layout: null, kernel_code, kernel_data, user_data, user_code, tss
/// User data MUST come before user code for SYSRET to work correctly!
#[repr(C, packed)]
pub struct Gdt {
    pub null: GdtEntry,
    pub kernel_code: GdtEntry,
    pub kernel_data: GdtEntry,
    pub user_data: GdtEntry,   // 0x18 - BEFORE user_code for SYSRET!
    pub user_code: GdtEntry,   // 0x20
    pub tss: TssEntry,
}

impl Gdt {
    pub const fn new() -> Self {
        Self {
            null: GdtEntry::null(),
            kernel_code: GdtEntry::code_segment(0),
            kernel_data: GdtEntry::data_segment(0),
            user_data: GdtEntry::data_segment(3),   // 0x18
            user_code: GdtEntry::code_segment(3),   // 0x20
            tss: TssEntry::null(),
        }
    }
}

/// GDT pointer structure
#[repr(C, packed)]
pub struct GdtPtr {
    pub limit: u16,
    pub base: u64,
}

// Static GDT and TSS (BSP)
static mut GDT: Gdt = Gdt::new();
static mut TSS: TaskStateSegment = TaskStateSegment::new();

// Per-CPU GDT and TSS (APs) — each AP needs its own TSS for distinct RSP0
const MAX_CPUS: usize = 64;
static mut PER_CPU_GDT: [Gdt; MAX_CPUS] = {
    const INIT: Gdt = Gdt::new();
    [INIT; MAX_CPUS]
};
static mut PER_CPU_TSS: [TaskStateSegment; MAX_CPUS] = {
    const INIT: TaskStateSegment = TaskStateSegment::new();
    [INIT; MAX_CPUS]
};

/// Initialize GDT with Ring 0/3 support
pub fn init() {
    unsafe {
        // Set up TSS with kernel stack for interrupts from Ring 3
        // Use a dedicated kernel stack (allocate on heap)
        let kernel_stack = alloc_kernel_stack();
        TSS.rsp[0] = kernel_stack; // RSP0 - kernel stack for Ring 3 -> Ring 0
        
        // IST1 for double fault handler
        let ist1_stack = alloc_kernel_stack();
        TSS.ist[0] = ist1_stack;
        
        // Set TSS entry in GDT
        let tss_addr = core::ptr::addr_of!(TSS) as u64;
        GDT.tss = TssEntry::new(tss_addr);
        
        // Create GDT pointer
        let gdt_ptr = GdtPtr {
            limit: (size_of::<Gdt>() - 1) as u16,
            base: core::ptr::addr_of!(GDT) as u64,
        };
        
        // Load GDT
        core::arch::asm!(
            "lgdt [{}]",
            in(reg) &gdt_ptr,
            options(readonly, nostack, preserves_flags)
        );
        
        // Reload code segment
        core::arch::asm!(
            "push {sel}",
            "lea {tmp}, [rip + 2f]",
            "push {tmp}",
            "retfq",
            "2:",
            sel = in(reg) KERNEL_CODE_SELECTOR as u64,
            tmp = lateout(reg) _,
            options(preserves_flags)
        );
        
        // Reload data segments
        core::arch::asm!(
            "mov ds, {0:x}",
            "mov es, {0:x}",
            "mov ss, {0:x}",
            in(reg) KERNEL_DATA_SELECTOR,
            options(nostack, preserves_flags)
        );
        
        // Load TSS
        core::arch::asm!(
            "ltr {0:x}",
            in(reg) TSS_SELECTOR,
            options(nostack, preserves_flags)
        );
    }
    
    crate::log_debug!("GDT initialized with Ring 0/3 support");
}

/// Allocate a kernel stack (returns top of stack)
fn alloc_kernel_stack() -> u64 {
    use alloc::vec::Vec;
    
    const STACK_SIZE: usize = 16 * 1024; // 16 KB kernel stack
    
    let stack: Vec<u8> = alloc::vec![0u8; STACK_SIZE];
    let stack_top = stack.as_ptr() as u64 + STACK_SIZE as u64;
    
    // Leak the stack so it persists
    core::mem::forget(stack);
    
    stack_top
}

/// Update RSP0 in TSS (called during context switch)
/// Automatically selects the correct per-CPU TSS.
pub fn set_kernel_stack(stack_top: u64) {
    let cpu_id = crate::cpu::smp::current_cpu_id() as usize;
    unsafe {
        if cpu_id == 0 {
            TSS.rsp[0] = stack_top;
        } else if cpu_id < MAX_CPUS {
            PER_CPU_TSS[cpu_id].rsp[0] = stack_top;
        }
    }
}

/// Initialize GDT/TSS for an Application Processor (AP)
/// Creates a per-CPU GDT+TSS so each core has its own kernel stack (RSP0).
pub fn init_ap(cpu_id: u32) {
    let idx = cpu_id as usize;
    if idx == 0 || idx >= MAX_CPUS { return; }
    
    unsafe {
        // Allocate kernel stack for this AP
        let kernel_stack = alloc_kernel_stack();
        PER_CPU_TSS[idx].rsp[0] = kernel_stack;
        
        // IST1 for double fault handler
        let ist1 = alloc_kernel_stack();
        PER_CPU_TSS[idx].ist[0] = ist1;
        
        // Set up GDT with TSS entry pointing to this AP's TSS
        PER_CPU_GDT[idx] = Gdt::new();
        let tss_addr = core::ptr::addr_of!(PER_CPU_TSS[idx]) as u64;
        PER_CPU_GDT[idx].tss = TssEntry::new(tss_addr);
        
        // Load this AP's GDT
        let gdt_ptr = GdtPtr {
            limit: (size_of::<Gdt>() - 1) as u16,
            base: core::ptr::addr_of!(PER_CPU_GDT[idx]) as u64,
        };
        
        core::arch::asm!(
            "lgdt [{}]",
            in(reg) &gdt_ptr,
            options(readonly, nostack, preserves_flags)
        );
        
        // Reload code segment (far return)
        core::arch::asm!(
            "push {sel}",
            "lea {tmp}, [rip + 2f]",
            "push {tmp}",
            "retfq",
            "2:",
            sel = in(reg) KERNEL_CODE_SELECTOR as u64,
            tmp = lateout(reg) _,
            options(preserves_flags)
        );
        
        // Reload data segments
        core::arch::asm!(
            "mov ds, {0:x}",
            "mov es, {0:x}",
            "mov ss, {0:x}",
            in(reg) KERNEL_DATA_SELECTOR,
            options(nostack, preserves_flags)
        );
        
        // Load TSS
        core::arch::asm!(
            "ltr {0:x}",
            in(reg) TSS_SELECTOR,
            options(nostack, preserves_flags)
        );
    }
    
    crate::serial_println!("[GDT] AP {} GDT/TSS initialized", cpu_id);
}

/// Get current privilege level
pub fn current_ring() -> u8 {
    let cs: u16;
    unsafe {
        core::arch::asm!("mov {:x}, cs", out(reg) cs, options(nomem, nostack));
    }
    (cs & 0x3) as u8
}

/// Check if running in kernel mode
pub fn is_kernel_mode() -> bool {
    current_ring() == 0
}

/// Check if running in user mode
pub fn is_user_mode() -> bool {
    current_ring() == 3
}
