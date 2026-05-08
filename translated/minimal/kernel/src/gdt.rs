
















use core::mem::size_of;


pub const KERNEL_CODE_SELECTOR: u16 = 0x08;

pub const KERNEL_DATA_SELECTOR: u16 = 0x10;

pub const ALG_: u16 = 0x18 | 3; 

pub const ALF_: u16 = 0x20 | 3; 

pub const TSS_SELECTOR: u16 = 0x28;


#[derive(Clone, Copy, Debug)]
#[repr(C, packed)]
pub struct GdtEntry {
    limit_low: u16,
    bxs: u16,
    base_middle: u8,
    access: u8,
    granularity: u8,
    bxr: u8,
}

impl GdtEntry {
    pub const fn null() -> Self {
        Self {
            limit_low: 0,
            bxs: 0,
            base_middle: 0,
            access: 0,
            granularity: 0,
            bxr: 0,
        }
    }
    
    
    pub const fn code_segment(dq: u8) -> Self {
        let access = if dq == 0 {
            0x9A 
        } else {
            0xFA 
        };
        
        Self {
            limit_low: 0xFFFF,
            bxs: 0,
            base_middle: 0,
            access,
            granularity: 0xAF, 
            bxr: 0,
        }
    }
    
    
    pub const fn hqq(dq: u8) -> Self {
        let access = if dq == 0 {
            0x92 
        } else {
            0xF2 
        };
        
        Self {
            limit_low: 0xFFFF,
            bxs: 0,
            base_middle: 0,
            access,
            granularity: 0xCF, 
            bxr: 0,
        }
    }
}


#[derive(Clone, Copy, Debug)]
#[repr(C, packed)]
pub struct TssEntry {
    length: u16,
    bxs: u16,
    base_middle: u8,
    flags1: u8,
    flags2: u8,
    bxr: u8,
    base_upper: u32,
    reserved: u32,
}

impl TssEntry {
    pub const fn null() -> Self {
        Self {
            length: 0,
            bxs: 0,
            base_middle: 0,
            flags1: 0,
            flags2: 0,
            bxr: 0,
            base_upper: 0,
            reserved: 0,
        }
    }
    
    
    pub fn new(ecs: u64) -> Self {
        let base = ecs;
        let jm = (size_of::<TaskStateSegment>() - 1) as u16;
        
        Self {
            length: jm,
            bxs: base as u16,
            base_middle: (base >> 16) as u8,
            flags1: 0x89, 
            flags2: 0x00,
            bxr: (base >> 24) as u8,
            base_upper: (base >> 32) as u32,
            reserved: 0,
        }
    }
}


#[derive(Clone, Copy, Debug)]
#[repr(C, packed)]
pub struct TaskStateSegment {
    reserved1: u32,
    
    pub rsp: [u64; 3],
    reserved2: u64,
    
    pub ist: [u64; 7],
    reserved3: u64,
    reserved4: u16,
    
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




#[repr(C, packed)]
pub struct Gdt {
    pub null: GdtEntry,
    pub kernel_code: GdtEntry,
    pub kernel_data: GdtEntry,
    pub user_data: GdtEntry,   
    pub user_code: GdtEntry,   
    pub tss: TssEntry,
}

impl Gdt {
    pub const fn new() -> Self {
        Self {
            null: GdtEntry::null(),
            kernel_code: GdtEntry::code_segment(0),
            kernel_data: GdtEntry::hqq(0),
            user_data: GdtEntry::hqq(3),   
            user_code: GdtEntry::code_segment(3),   
            tss: TssEntry::null(),
        }
    }
}


#[repr(C, packed)]
pub struct Zg {
    pub jm: u16,
    pub base: u64,
}


static mut Zb: Gdt = Gdt::new();
static mut Kt: TaskStateSegment = TaskStateSegment::new();


const AR_: usize = 64;
static mut AIE_: [Gdt; AR_] = {
    const Bm: Gdt = Gdt::new();
    [Bm; AR_]
};
static mut XM_: [TaskStateSegment; AR_] = {
    const Bm: TaskStateSegment = TaskStateSegment::new();
    [Bm; AR_]
};


pub fn init() {
    unsafe {
        
        
        let kernel_stack = efk();
        Kt.rsp[0] = kernel_stack; 
        
        
        let mun = efk();
        Kt.ist[0] = mun;
        
        
        let ecs = core::ptr::addr_of!(Kt) as u64;
        Zb.tss = TssEntry::new(ecs);
        
        
        let gdt_ptr = Zg {
            jm: (size_of::<Gdt>() - 1) as u16,
            base: core::ptr::addr_of!(Zb) as u64,
        };
        
        
        core::arch::asm!(
            "lgdt [{}]",
            in(reg) &gdt_ptr,
            options(readonly, nostack, preserves_flags)
        );
        
        
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
        
        
        core::arch::asm!(
            "mov ds, {0:x}",
            "mov es, {0:x}",
            "mov ss, {0:x}",
            in(reg) KERNEL_DATA_SELECTOR,
            options(nostack, preserves_flags)
        );
        
        
        core::arch::asm!(
            "ltr {0:x}",
            in(reg) TSS_SELECTOR,
            options(nostack, preserves_flags)
        );
    }
    
    crate::log_debug!("GDT initialized with Ring 0/3 support");
}


fn efk() -> u64 {
    use alloc::vec::Vec;
    
    const JS_: usize = 512 * 1024; 
    
    let dn: Vec<u8> = alloc::vec![0u8; JS_];
    let te = dn.as_ptr() as u64 + JS_ as u64;
    
    
    core::mem::forget(dn);
    
    te
}



pub fn jfg(te: u64) {
    let cpu_id = crate::cpu::smp::bll() as usize;
    unsafe {
        if cpu_id == 0 {
            Kt.rsp[0] = te;
        } else if cpu_id < AR_ {
            XM_[cpu_id].rsp[0] = te;
        }
    }
}



pub fn cau(cpu_id: u32) {
    let idx = cpu_id as usize;
    if idx == 0 || idx >= AR_ { return; }
    
    unsafe {
        
        let kernel_stack = efk();
        XM_[idx].rsp[0] = kernel_stack;
        
        
        let mum = efk();
        XM_[idx].ist[0] = mum;
        
        
        AIE_[idx] = Gdt::new();
        let ecs = core::ptr::addr_of!(XM_[idx]) as u64;
        AIE_[idx].tss = TssEntry::new(ecs);
        
        
        let gdt_ptr = Zg {
            jm: (size_of::<Gdt>() - 1) as u16,
            base: core::ptr::addr_of!(AIE_[idx]) as u64,
        };
        
        core::arch::asm!(
            "lgdt [{}]",
            in(reg) &gdt_ptr,
            options(readonly, nostack, preserves_flags)
        );
        
        
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
        
        
        core::arch::asm!(
            "mov ds, {0:x}",
            "mov es, {0:x}",
            "mov ss, {0:x}",
            in(reg) KERNEL_DATA_SELECTOR,
            options(nostack, preserves_flags)
        );
        
        
        core::arch::asm!(
            "ltr {0:x}",
            in(reg) TSS_SELECTOR,
            options(nostack, preserves_flags)
        );
    }
    
    crate::serial_println!("[GDT] AP {} GDT/TSS initialized", cpu_id);
}


pub fn fpw() -> u8 {
    let cs: u16;
    unsafe {
        core::arch::asm!("mov {:x}, cs", out(reg) cs, options(nomem, nostack));
    }
    (cs & 0x3) as u8
}


pub fn msv() -> bool {
    fpw() == 0
}


pub fn mub() -> bool {
    fpw() == 3
}
