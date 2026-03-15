



use super::tables::{Ei, Gj};


#[repr(C, packed)]
struct Cdi {
    dh: Ei,
    
    
    yqu: u32,
    
    dgt: u32,
    
    
    iii: u8,
    
    zgi: u8,
    
    wei: u16,
    
    fuy: u32,
    
    gxr: u8,
    
    jyz: u8,
    
    zko: u8,
    
    zgu: u8,
    
    
    gpl: u32,
    
    lul: u32,
    
    dul: u32,
    
    hvk: u32,
    
    zfp: u32,
    
    jjn: u32,
    
    yve: u32,
    
    yvh: u32,
    
    
    zfm: u8,
    
    zfl: u8,
    
    zfq: u8,
    
    lum: u8,
    
    yvf: u8,
    
    yvi: u8,
    
    yvg: u8,
    
    yky: u8,
    
    ygz: u16,
    
    yha: u16,
    
    yre: u16,
    
    yrf: u16,
    
    yny: u8,
    
    ynz: u8,
    
    ylf: u8,
    
    zda: u8,
    
    hcn: u8,
    
    
    ygr: u16,
    
    fzp: u8,
    
    flags: u32,
    
    
    gqu: Gj,
    
    hxp: u8,
    
    yfa: u16,
    
    yqf: u8,
    
    
    
    zxm: u64,
    
    zxl: u64,
    
    zxr: Gj,
    
    zxt: Gj,
    
    zxq: Gj,
    
    zxs: Gj,
    
    zxu: Gj,
    
    zxv: Gj,
    
    zxn: Gj,
    
    zxo: Gj,
    
    wpk: Gj,
    
    mgb: Gj,
}


#[derive(Debug, Clone)]
pub struct FadtInfo {
    
    pub grm: u16,
    
    pub fuy: u32,
    
    pub gxr: u8,
    
    pub jyz: u8,
    
    
    pub gpl: u32,
    
    pub lul: u32,
    
    pub dul: u32,
    
    pub hvk: u32,
    
    pub jjn: u32,
    
    
    pub nca: u8,
    
    
    pub gqu: Gj,
    
    pub hxp: u8,
    
    
    pub plh: Option<Gj>,
    
    pub mgb: Option<Gj>,
    
    
    pub flags: u32,
}

impl FadtInfo {
    
    pub const BVE_: u32 = 1 << 20;
    
    pub const BVF_: u32 = 1 << 21;
    
    pub const DLZ_: u32 = 1 << 0;
    
    pub const BVG_: u32 = 1 << 10;
    
    pub fn ogb(&self) -> bool {
        (self.flags & Self::BVE_) != 0
    }
    
    pub fn ppx(&self) -> bool {
        (self.flags & Self::BVG_) != 0
    }
}


pub fn parse(nst: u64, xzr: u64) -> Option<FadtInfo> {
    let dh = unsafe { &*(nst as *const Ei) };
    
    
    if &dh.signature != b"FACP" {
        return None;
    }
    
    let fadt = unsafe { &*(nst as *const Cdi) };
    
    
    let grm = unsafe { core::ptr::md(core::ptr::vf!(fadt.wei)) };
    let fuy = unsafe { core::ptr::md(core::ptr::vf!(fadt.fuy)) };
    let vjg = unsafe { core::ptr::md(core::ptr::vf!(fadt.gpl)) };
    let vji = unsafe { core::ptr::md(core::ptr::vf!(fadt.lul)) };
    let vjf = unsafe { core::ptr::md(core::ptr::vf!(fadt.dul)) };
    let vjh = unsafe { core::ptr::md(core::ptr::vf!(fadt.hvk)) };
    let jjm = unsafe { core::ptr::md(core::ptr::vf!(fadt.jjn)) };
    let flags = unsafe { core::ptr::md(core::ptr::vf!(fadt.flags)) };
    let gqu = unsafe { core::ptr::md(core::ptr::vf!(fadt.gqu)) };
    
    
    let tml = dh.go >= 244;
    
    let (mga, wpm) = if tml && dh.go >= 276 {
        let db = unsafe { core::ptr::md(core::ptr::vf!(fadt.wpk)) };
        let status = unsafe { core::ptr::md(core::ptr::vf!(fadt.mgb)) };
        (
            if db.cld() { Some(db) } else { None },
            if status.cld() { Some(status) } else { None }
        )
    } else {
        (None, None)
    };
    
    Some(FadtInfo {
        grm,
        fuy,
        gxr: fadt.gxr,
        jyz: fadt.jyz,
        gpl: vjg,
        lul: vji,
        dul: vjf,
        hvk: vjh,
        jjn: jjm,
        nca: fadt.hcn,
        gqu,
        hxp: fadt.hxp,
        plh: mga,
        mgb: wpm,
        flags,
    })
}


pub fn cbu(fadt: &FadtInfo) {
    crate::serial_println!("[ACPI] Attempting ACPI shutdown...");
    
    
    
    
    unsafe {
        
        x86_64::instructions::port::Port::<u16>::new(0x604).write(0x2000);
        for _ in 0..100_000 { core::hint::hc(); }
        
        
        x86_64::instructions::port::Port::<u16>::new(0xB004).write(0x2000);
        for _ in 0..100_000 { core::hint::hc(); }
        
        
        x86_64::instructions::port::Port::<u16>::new(0x4004).write(0x3400);
        for _ in 0..100_000 { core::hint::hc(); }
        
        
        x86_64::instructions::port::Port::<u8>::new(0x600).write(0x34);
        for _ in 0..100_000 { core::hint::hc(); }
    }
    
    crate::serial_println!("[ACPI] VM ports didn't work, trying ACPI PM1...");
    
    
    if fadt.fuy != 0 && fadt.gxr != 0 {
        unsafe {
            
            if fadt.dul != 0 {
                let cv = x86_64::instructions::port::Port::<u16>::new(fadt.dul as u16).read();
                if (cv & 0x0001) == 0 {
                    
                    crate::serial_println!("[ACPI] Enabling ACPI mode via SMI_CMD={:#x}", fadt.fuy);
                    x86_64::instructions::port::Port::<u8>::new(fadt.fuy as u16)
                        .write(fadt.gxr);
                    
                    for _ in 0..10_000_000 {
                        let ap = x86_64::instructions::port::Port::<u16>::new(fadt.dul as u16).read();
                        if (ap & 0x0001) != 0 { break; }
                        core::hint::hc();
                    }
                }
            }
        }
    }
    
    
    if fadt.ogb() {
        if let Some(ref mga) = fadt.plh {
            
            unsafe { mga.write(0x20); }
            for _ in 0..1_000_000 { core::hint::hc(); }
        }
    }
    
    
    
    
    const PO_: u16 = 1 << 13;
    
    let pex: [u16; 4] = [
        0x00 << 10,  
        0x05 << 10,  
        0x07 << 10,  
        0x02 << 10,  
    ];
    
    if fadt.dul != 0 {
        let port = fadt.dul as u16;
        
        for &dwb in &pex {
            unsafe {
                
                let cv = x86_64::instructions::port::Port::<u16>::new(port).read();
                let weh = cv & 0x0001;
                
                crate::serial_println!("[ACPI] S5: writing SLP_TYP={:#06x} | SLP_EN to PM1a_CNT port {:#x}",
                    dwb, port);
                
                x86_64::instructions::port::Port::<u16>::new(port)
                    .write(weh | dwb | PO_);
                
                
                for _ in 0..500_000 { core::hint::hc(); }
            }
        }
        
        
        if fadt.hvk != 0 {
            let vjw = fadt.hvk as u16;
            for &dwb in &pex {
                unsafe {
                    x86_64::instructions::port::Port::<u16>::new(vjw)
                        .write(dwb | PO_);
                    for _ in 0..200_000 { core::hint::hc(); }
                }
            }
        }
    }
    
    crate::serial_println!("[ACPI] All shutdown methods failed");
}


pub fn apa(fadt: &FadtInfo) {
    crate::serial_println!("[ACPI] Attempting ACPI reset...");
    
    if fadt.ppx() && fadt.gqu.cld() {
        unsafe {
            fadt.gqu.write(fadt.hxp as u64);
        }
        
        
        for _ in 0..1000000 { core::hint::hc(); }
    }
    
    crate::serial_println!("[ACPI] Reset register failed");
}






pub fn wwc(fadt: &FadtInfo) -> bool {
    crate::serial_println!("[ACPI] Attempting S3 suspend (sleep-to-RAM)...");

    
    const CTL_: u16 = 0x01 << 10;   
    const CTK_: u16 = 0x05 << 10;    
    const CTJ_: u16 = 0x03 << 10;     
    const PO_: u16 = 1 << 13;

    
    if fadt.gpl != 0 {
        unsafe {
            
            let skw = (fadt.gpl + 2) as u16;
            let skx: u16 = (1 << 0)   
                            | (1 << 8)   
                            | (1 << 5);  
            x86_64::instructions::port::Port::<u16>::new(skw).write(skx);
        }
    }

    if fadt.dul != 0 {
        let port = fadt.dul as u16;
        let alv = [CTL_, CTK_, CTJ_];

        for &dwb in &alv {
            unsafe {
                let cv = x86_64::instructions::port::Port::<u16>::new(port).read();
                let owy = cv & 0x0203; 

                crate::serial_println!("[ACPI] S3: writing 0x{:04X} to PM1a_CNT (port 0x{:03X})",
                    owy | dwb | PO_, port);

                
                core::arch::x86_64::ybf();

                x86_64::instructions::port::Port::<u16>::new(port)
                    .write(owy | dwb | PO_);

                
                
                for _ in 0..5_000_000 { core::hint::hc(); }
            }
        }

        crate::serial_println!("[ACPI] S3 suspend did not take effect");
        return false;
    }

    crate::serial_println!("[ACPI] No PM1a_CNT register — cannot suspend");
    false
}
