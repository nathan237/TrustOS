
















use core::mem::size_of;


pub const NQ_: u16 = 0x08;

pub const NR_: u16 = 0x10;

pub const AJL_: u16 = 0x18 | 3; 

pub const AJK_: u16 = 0x20 | 3; 

pub const AJF_: u16 = 0x28;


#[derive(Clone, Copy, Debug)]
#[repr(C, packed)]
pub struct GdtEntry {
    liq: u16,
    emp: u16,
    gat: u8,
    vz: u8,
    hlw: u8,
    emo: u8,
}

impl GdtEntry {
    pub const fn null() -> Self {
        Self {
            liq: 0,
            emp: 0,
            gat: 0,
            vz: 0,
            hlw: 0,
            emo: 0,
        }
    }
    
    
    pub const fn dzo(mz: u8) -> Self {
        let vz = if mz == 0 {
            0x9A 
        } else {
            0xFA 
        };
        
        Self {
            liq: 0xFFFF,
            emp: 0,
            gat: 0,
            vz,
            hlw: 0xAF, 
            emo: 0,
        }
    }
    
    
    pub const fn njp(mz: u8) -> Self {
        let vz = if mz == 0 {
            0x92 
        } else {
            0xF2 
        };
        
        Self {
            liq: 0xFFFF,
            emp: 0,
            gat: 0,
            vz,
            hlw: 0xCF, 
            emo: 0,
        }
    }
}


#[derive(Clone, Copy, Debug)]
#[repr(C, packed)]
pub struct TssEntry {
    go: u16,
    emp: u16,
    gat: u8,
    nux: u8,
    nuy: u8,
    emo: u8,
    myc: u32,
    awt: u32,
}

impl TssEntry {
    pub const fn null() -> Self {
        Self {
            go: 0,
            emp: 0,
            gat: 0,
            nux: 0,
            nuy: 0,
            emo: 0,
            myc: 0,
            awt: 0,
        }
    }
    
    
    pub fn new(ife: u64) -> Self {
        let ar = ife;
        let ul = (size_of::<TaskStateSegment>() - 1) as u16;
        
        Self {
            go: ul,
            emp: ar as u16,
            gat: (ar >> 16) as u8,
            nux: 0x89, 
            nuy: 0x00,
            emo: (ar >> 24) as u8,
            myc: (ar >> 32) as u32,
            awt: 0,
        }
    }
}


#[derive(Clone, Copy, Debug)]
#[repr(C, packed)]
pub struct TaskStateSegment {
    pco: u32,
    
    pub rsp: [u64; 3],
    vxq: u64,
    
    pub lgs: [u64; 7],
    vxr: u64,
    vxs: u16,
    
    pub twj: u16,
}

impl TaskStateSegment {
    pub const fn new() -> Self {
        Self {
            pco: 0,
            rsp: [0; 3],
            vxq: 0,
            lgs: [0; 7],
            vxr: 0,
            vxs: 0,
            twj: size_of::<TaskStateSegment>() as u16,
        }
    }
}




#[repr(C, packed)]
pub struct Gdt {
    pub null: GdtEntry,
    pub uaz: GdtEntry,
    pub abr: GdtEntry,
    pub xpv: GdtEntry,   
    pub xpu: GdtEntry,   
    pub tss: TssEntry,
}

impl Gdt {
    pub const fn new() -> Self {
        Self {
            null: GdtEntry::null(),
            uaz: GdtEntry::dzo(0),
            abr: GdtEntry::njp(0),
            xpv: GdtEntry::njp(3),   
            xpu: GdtEntry::dzo(3),   
            tss: TssEntry::null(),
        }
    }
}


#[repr(C, packed)]
pub struct Bhv {
    pub ul: u16,
    pub ar: u64,
}


static mut Bhp: Gdt = Gdt::new();
static mut Za: TaskStateSegment = TaskStateSegment::new();


const AN_: usize = 64;
static mut AGK_: [Gdt; AN_] = {
    const Dm: Gdt = Gdt::new();
    [Dm; AN_]
};
static mut WD_: [TaskStateSegment; AN_] = {
    const Dm: TaskStateSegment = TaskStateSegment::new();
    [Dm; AN_]
};


pub fn init() {
    unsafe {
        
        
        let bhg = ijl();
        Za.rsp[0] = bhg; 
        
        
        let tzv = ijl();
        Za.lgs[0] = tzv;
        
        
        let ife = core::ptr::vf!(Za) as u64;
        Bhp.tss = TssEntry::new(ife);
        
        
        let ghv = Bhv {
            ul: (size_of::<Gdt>() - 1) as u16,
            ar: core::ptr::vf!(Bhp) as u64,
        };
        
        
        core::arch::asm!(
            "lgdt [{}]",
            in(reg) &ghv,
            options(awr, nostack, preserves_flags)
        );
        
        
        core::arch::asm!(
            "push {sel}",
            "lea {tmp}, [rip + 2f]",
            "push {tmp}",
            "retfq",
            "2:",
            fua = in(reg) NQ_ as u64,
            gup = lateout(reg) _,
            options(preserves_flags)
        );
        
        
        core::arch::asm!(
            "mov ds, {0:x}",
            "mov es, {0:x}",
            "mov ss, {0:x}",
            in(reg) NR_,
            options(nostack, preserves_flags)
        );
        
        
        core::arch::asm!(
            "ltr {0:x}",
            in(reg) AJF_,
            options(nostack, preserves_flags)
        );
    }
    
    crate::log_debug!("GDT initialized with Ring 0/3 support");
}


fn ijl() -> u64 {
    use alloc::vec::Vec;
    
    const IZ_: usize = 512 * 1024; 
    
    let jo: Vec<u8> = alloc::vec![0u8; IZ_];
    let alt = jo.fq() as u64 + IZ_ as u64;
    
    
    core::mem::forget(jo);
    
    alt
}



pub fn pjb(alt: u64) {
    let qq = crate::cpu::smp::ead() as usize;
    unsafe {
        if qq == 0 {
            Za.rsp[0] = alt;
        } else if qq < AN_ {
            WD_[qq].rsp[0] = alt;
        }
    }
}



pub fn eso(qq: u32) {
    let w = qq as usize;
    if w == 0 || w >= AN_ { return; }
    
    unsafe {
        
        let bhg = ijl();
        WD_[w].rsp[0] = bhg;
        
        
        let tzu = ijl();
        WD_[w].lgs[0] = tzu;
        
        
        AGK_[w] = Gdt::new();
        let ife = core::ptr::vf!(WD_[w]) as u64;
        AGK_[w].tss = TssEntry::new(ife);
        
        
        let ghv = Bhv {
            ul: (size_of::<Gdt>() - 1) as u16,
            ar: core::ptr::vf!(AGK_[w]) as u64,
        };
        
        core::arch::asm!(
            "lgdt [{}]",
            in(reg) &ghv,
            options(awr, nostack, preserves_flags)
        );
        
        
        core::arch::asm!(
            "push {sel}",
            "lea {tmp}, [rip + 2f]",
            "push {tmp}",
            "retfq",
            "2:",
            fua = in(reg) NQ_ as u64,
            gup = lateout(reg) _,
            options(preserves_flags)
        );
        
        
        core::arch::asm!(
            "mov ds, {0:x}",
            "mov es, {0:x}",
            "mov ss, {0:x}",
            in(reg) NR_,
            options(nostack, preserves_flags)
        );
        
        
        core::arch::asm!(
            "ltr {0:x}",
            in(reg) AJF_,
            options(nostack, preserves_flags)
        );
    }
    
    crate::serial_println!("[GDT] AP {} GDT/TSS initialized", qq);
}


pub fn knb() -> u8 {
    let aap: u16;
    unsafe {
        core::arch::asm!("mov {:x}, cs", bd(reg) aap, options(nomem, nostack));
    }
    (aap & 0x3) as u8
}


pub fn txv() -> bool {
    knb() == 0
}


pub fn tzj() -> bool {
    knb() == 3
}
