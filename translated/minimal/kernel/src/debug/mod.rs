













use alloc::string::String;
use alloc::vec::Vec;
use alloc::format;
use core::sync::atomic::{AtomicU8, AtomicU64, AtomicBool, Ordering};
use spin::Mutex;






static AYV_: AtomicU8 = AtomicU8::new(0);



pub fn jjr(aj: u8) {
    AYV_.store(aj, Ordering::Relaxed);
    #[cfg(target_arch = "x86_64")]
    unsafe {
        core::arch::asm!("out dx, al", in("dx") 0x80u16, in("al") aj, options(nostack, preserves_flags));
        
        core::arch::asm!("out dx, al", in("dx") 0xE9u16, in("al") aj, options(nostack, preserves_flags));
    }
}


pub fn oie() -> u8 {
    AYV_.load(Ordering::Relaxed)
}






const AEV_: usize = 32;


struct CheckpointLog {
    ch: [(u64, u8, &'static str); AEV_],
    az: usize,
}

impl CheckpointLog {
    const fn new() -> Self {
        Self {
            ch: [(0, 0, ""); AEV_],
            az: 0,
        }
    }
    fn push(&mut self, bt: (u64, u8, &'static str)) {
        if self.az < AEV_ {
            self.ch[self.az] = bt;
            self.az += 1;
        }
    }
    fn iter(&self) -> core::slice::Dah<'_, (u64, u8, &'static str)> {
        self.ch[..self.az].iter()
    }
    fn is_empty(&self) -> bool {
        self.az == 0
    }
    fn gai(&self) -> &[(u64, u8, &'static str)] {
        &self.ch[..self.az]
    }
}

static Apo: Mutex<CheckpointLog> = Mutex::new(CheckpointLog::new());


pub const CKP_:     u8 = 0x10;
pub const EAU_:     u8 = 0x11;
pub const CKK_:             u8 = 0x20;
pub const CKL_:             u8 = 0x21;
pub const CKI_:      u8 = 0x22;
pub const CKG_:            u8 = 0x30;
pub const CKH_:            u8 = 0x31;
pub const CKR_:             u8 = 0x32;
pub const EAW_:          u8 = 0x40;
pub const EAV_:            u8 = 0x41;
pub const EAX_:          u8 = 0x42;
pub const CKN_:             u8 = 0x50;
pub const CKJ_:            u8 = 0x51;
pub const CKM_:         u8 = 0x60;
pub const CKS_:             u8 = 0x70;
pub const CKO_:         u8 = 0x80;
pub const CKQ_:     u8 = 0xAA;
pub const BDL_:           u8 = 0xFF;


pub fn cpc(aj: u8, j: &'static str) {
    jjr(aj);
    let tsc = ow();
    crate::serial_println!("[POST 0x{:02X}] {}", aj, j);

    
    if let Some(mut log) = Apo.try_lock() {
        log.push((tsc, aj, j));
    }
}


pub fn nxv() -> Vec<(u64, u8, &'static str)> {
    let log = Apo.lock();
    log.gai().ip()
}






#[cfg(target_arch = "x86_64")]
pub fn hab(ard: u32, uk: u32) {
    if ard == 0 { return; }
    let fgv = 1193180u32 / ard;

    unsafe {
        
        core::arch::asm!("out dx, al", in("dx") 0x43u16, in("al") 0xB6u8, options(nostack, preserves_flags));
        core::arch::asm!("out dx, al", in("dx") 0x42u16, in("al") (fgv & 0xFF) as u8, options(nostack, preserves_flags));
        core::arch::asm!("out dx, al", in("dx") 0x42u16, in("al") ((fgv >> 8) & 0xFF) as u8, options(nostack, preserves_flags));

        
        let ap: u8;
        core::arch::asm!("in al, dx", in("dx") 0x61u16, bd("al") ap, options(nostack, preserves_flags));
        let ea = ap | 0x03;
        core::arch::asm!("out dx, al", in("dx") 0x61u16, in("al") ea, options(nostack, preserves_flags));

        
        nat(uk);

        
        let dz = ap & !0x03;
        core::arch::asm!("out dx, al", in("dx") 0x61u16, in("al") dz, options(nostack, preserves_flags));
    }
}

#[cfg(not(target_arch = "x86_64"))]
pub fn hab(xzj: u32, xyx: u32) {}


pub fn yfs()    { hab(1000, 100); }  
pub fn yft()  { hab(500, 200); nat(100); hab(500, 200); } 
pub fn qoo() { hab(200, 500); } 







#[cfg(target_arch = "x86_64")]
pub fn pnt(lky: usize) -> Vec<(u64, u64)> {
    let mut vj = Vec::new();
    let mut rbp: u64;
    
    unsafe {
        core::arch::asm!("mov {}, rbp", bd(reg) rbp);
    }
    
    let jcb = 0xFFFF_8000_0000_0000u64; 
    
    for _ in 0..lky {
        if rbp == 0 || rbp < jcb {
            break;
        }
        
        
        let nvz = rbp as *const u64;
        
        
        let dbg = unsafe { core::ptr::read_volatile(nvz.add(1)) };
        let oxk = unsafe { core::ptr::read_volatile(nvz) };
        
        if dbg == 0 {
            break;
        }
        
        vj.push((dbg, rbp));
        
        
        if oxk <= rbp {
            break;
        }
        rbp = oxk;
    }
    
    vj
}

#[cfg(not(target_arch = "x86_64"))]
pub fn pnt(yau: usize) -> Vec<(u64, u64)> {
    Vec::new()
}


pub fn ivi(lky: usize) -> Vec<String> {
    let mut ak = Vec::new();
    ak.push(String::from("  Stack Backtrace:"));
    ak.push(String::from("  ─────────────────────────────────────────────"));
    
    let vj = pnt(lky);
    if vj.is_empty() {
        ak.push(String::from("  <no frames — frame pointers may be omitted>"));
        ak.push(String::from("  Hint: build with RUSTFLAGS=\"-Cforce-frame-pointers=yes\""));
    } else {
        for (a, (dbg, rbp)) in vj.iter().cf() {
            ak.push(format!("  #{:>2}: 0x{:016x}  (rbp=0x{:016x})", a, dbg, rbp));
        }
    }
    
    ak
}






#[cfg(target_arch = "x86_64")]
pub fn ivz() -> Vec<String> {
    let mut ak = Vec::new();
    
    let rax: u64; let rcx: u64; let rdx: u64;
    let rsi: u64; let rdi: u64; let rsp: u64; let rbp: u64;
    let r8: u64;  let r9: u64;  let r10: u64; let r11: u64;
    let r12: u64; let r13: u64; let r14: u64; let r15: u64;
    let rflags: u64;
    let akb: u64; let ngx: u64; let jm: u64; let cr4: u64;
    let aap: u16; let bjw: u16; let cqf: u16; let fs: u16; let ckx: u16; let rv: u16;
    
    unsafe {
        core::arch::asm!("mov {}, rax", bd(reg) rax);
        
        core::arch::asm!("mov {}, rcx", bd(reg) rcx);
        core::arch::asm!("mov {}, rdx", bd(reg) rdx);
        core::arch::asm!("mov {}, rsi", bd(reg) rsi);
        core::arch::asm!("mov {}, rdi", bd(reg) rdi);
        core::arch::asm!("mov {}, rsp", bd(reg) rsp);
        core::arch::asm!("mov {}, rbp", bd(reg) rbp);
        core::arch::asm!("mov {}, r8",  bd(reg) r8);
        core::arch::asm!("mov {}, r9",  bd(reg) r9);
        core::arch::asm!("mov {}, r10", bd(reg) r10);
        core::arch::asm!("mov {}, r11", bd(reg) r11);
        core::arch::asm!("mov {}, r12", bd(reg) r12);
        core::arch::asm!("mov {}, r13", bd(reg) r13);
        core::arch::asm!("mov {}, r14", bd(reg) r14);
        core::arch::asm!("mov {}, r15", bd(reg) r15);
        core::arch::asm!("pushfq; pop {}", bd(reg) rflags);
        core::arch::asm!("mov {}, cr0", bd(reg) akb);
        core::arch::asm!("mov {}, cr2", bd(reg) ngx);
        core::arch::asm!("mov {}, cr3", bd(reg) jm);
        core::arch::asm!("mov {}, cr4", bd(reg) cr4);
        core::arch::asm!("mov {:x}, cs", bd(reg) aap);
        core::arch::asm!("mov {:x}, ds", bd(reg) bjw);
        core::arch::asm!("mov {:x}, es", bd(reg) cqf);
        core::arch::asm!("mov {:x}, fs", bd(reg) fs);
        core::arch::asm!("mov {:x}, gs", bd(reg) ckx);
        core::arch::asm!("mov {:x}, ss", bd(reg) rv);
    }
    
    ak.push(String::from("  ╔══════════════════════════════════════════════════╗"));
    ak.push(String::from("  ║         FULL CPU STATE DUMP                     ║"));
    ak.push(String::from("  ╚══════════════════════════════════════════════════╝"));
    
    ak.push(String::from("  ── General Purpose Registers ──"));
    ak.push(format!("  RAX = 0x{:016x}   RBX = <LLVM reserved>", rax));
    ak.push(format!("  RCX = 0x{:016x}   RDX = 0x{:016x}", rcx, rdx));
    ak.push(format!("  RSI = 0x{:016x}   RDI = 0x{:016x}", rsi, rdi));
    ak.push(format!("  RSP = 0x{:016x}   RBP = 0x{:016x}", rsp, rbp));
    ak.push(format!("  R8  = 0x{:016x}   R9  = 0x{:016x}", r8, r9));
    ak.push(format!("  R10 = 0x{:016x}   R11 = 0x{:016x}", r10, r11));
    ak.push(format!("  R12 = 0x{:016x}   R13 = 0x{:016x}", r12, r13));
    ak.push(format!("  R14 = 0x{:016x}   R15 = 0x{:016x}", r14, r15));
    
    ak.push(String::from(""));
    ak.push(String::from("  ── RFLAGS ──"));
    ak.push(format!("  RFLAGS = 0x{:016x}", rflags));
    let mut flags = Vec::new();
    if rflags & (1 << 0) != 0 { flags.push("CF"); }
    if rflags & (1 << 2) != 0 { flags.push("PF"); }
    if rflags & (1 << 6) != 0 { flags.push("ZF"); }
    if rflags & (1 << 7) != 0 { flags.push("SF"); }
    if rflags & (1 << 8) != 0 { flags.push("TF"); }
    if rflags & (1 << 9) != 0 { flags.push("IF"); }
    if rflags & (1 << 10) != 0 { flags.push("DF"); }
    if rflags & (1 << 11) != 0 { flags.push("OF"); }
    if rflags & (1 << 14) != 0 { flags.push("NT"); }
    if rflags & (1 << 21) != 0 { flags.push("ID"); }
    ak.push(format!("           [{}]", flags.rr(" | ")));
    
    ak.push(String::from(""));
    ak.push(String::from("  ── Control Registers ──"));
    ak.push(format!("  CR0 = 0x{:016x}", akb));
    ak.push(format!("  CR2 = 0x{:016x}  (last page fault addr)", ngx));
    ak.push(format!("  CR3 = 0x{:016x}  (page table root)", jm));
    ak.push(format!("  CR4 = 0x{:016x}", cr4));
    
    ak.push(String::from(""));
    ak.push(String::from("  ── Segment Registers ──"));
    ak.push(format!("  CS=0x{:04x}  DS=0x{:04x}  ES=0x{:04x}  FS=0x{:04x}  GS=0x{:04x}  SS=0x{:04x}", aap, bjw, cqf, fs, ckx, rv));
    
    
    ak.push(String::from(""));
    ak.push(String::from("  ── Model Specific Registers ──"));
    
    let uqk: &[(u32, &str)] = &[
        (0xC000_0080, "IA32_EFER"),
        (0xC000_0081, "IA32_STAR"),
        (0xC000_0082, "IA32_LSTAR"),
        (0xC000_0083, "IA32_CSTAR"),
        (0xC000_0084, "IA32_FMASK"),
        (0xC000_0100, "IA32_FS_BASE"),
        (0xC000_0101, "IA32_GS_BASE"),
        (0xC000_0102, "IA32_KERNEL_GS_BASE"),
        (0x0000_0010, "IA32_TSC"),
        (0x0000_001B, "IA32_APIC_BASE"),
        (0x0000_0174, "IA32_SYSENTER_CS"),
        (0x0000_0175, "IA32_SYSENTER_ESP"),
        (0x0000_0176, "IA32_SYSENTER_EIP"),
        (0x0000_0277, "IA32_PAT"),
    ];
    
    for &(lmz, j) in uqk {
        match fsg(lmz) {
            Some(ap) => ak.push(format!("  0x{:08X} ({:<24}) = 0x{:016x}", lmz, j, ap)),
            None      => ak.push(format!("  0x{:08X} ({:<24}) = <GPF — not available>", lmz, j)),
        }
    }
    
    ak
}

#[cfg(not(target_arch = "x86_64"))]
pub fn ivz() -> Vec<String> {
    vec![String::from("  Full CPU dump only available on x86_64")]
}






#[cfg(target_arch = "x86_64")]
pub fn cfn(port: u16) -> u8 {
    let ap: u8;
    unsafe { core::arch::asm!("in al, dx", in("dx") port, bd("al") ap, options(nostack, preserves_flags)); }
    ap
}


#[cfg(target_arch = "x86_64")]
pub fn jar(port: u16) -> u16 {
    let ap: u16;
    unsafe { core::arch::asm!("in ax, dx", in("dx") port, bd("ax") ap, options(nostack, preserves_flags)); }
    ap
}


#[cfg(target_arch = "x86_64")]
pub fn jac(port: u16) -> u32 {
    let ap: u32;
    unsafe { core::arch::asm!("in eax, dx", in("dx") port, bd("eax") ap, options(nostack, preserves_flags)); }
    ap
}


#[cfg(target_arch = "x86_64")]
pub fn bkt(port: u16, ap: u8) {
    unsafe { core::arch::asm!("out dx, al", in("dx") port, in("al") ap, options(nostack, preserves_flags)); }
}


#[cfg(target_arch = "x86_64")]
pub fn jie(port: u16, ap: u16) {
    unsafe { core::arch::asm!("out dx, ax", in("dx") port, in("ax") ap, options(nostack, preserves_flags)); }
}


#[cfg(target_arch = "x86_64")]
pub fn jic(port: u16, ap: u32) {
    unsafe { core::arch::asm!("out dx, eax", in("dx") port, in("eax") ap, options(nostack, preserves_flags)); }
}

#[cfg(not(target_arch = "x86_64"))]
pub fn cfn(gxk: u16) -> u8 { 0 }
#[cfg(not(target_arch = "x86_64"))]
pub fn jar(gxk: u16) -> u16 { 0 }
#[cfg(not(target_arch = "x86_64"))]
pub fn jac(gxk: u16) -> u32 { 0 }
#[cfg(not(target_arch = "x86_64"))]
pub fn bkt(gxk: u16, msx: u8) {}
#[cfg(not(target_arch = "x86_64"))]
pub fn jie(gxk: u16, msx: u16) {}
#[cfg(not(target_arch = "x86_64"))]
pub fn jic(gxk: u16, msx: u32) {}






#[cfg(target_arch = "x86_64")]
pub fn fsg(msr: u32) -> Option<u64> {
    
    
    let hh: u32;
    let gd: u32;
    unsafe {
        core::arch::asm!(
            "rdmsr",
            in("ecx") msr,
            bd("eax") hh,
            bd("edx") gd,
            options(nostack, preserves_flags),
        );
    }
    Some(((gd as u64) << 32) | (hh as u64))
}

#[cfg(not(target_arch = "x86_64"))]
pub fn fsg(qde: u32) -> Option<u64> {
    None
}


#[cfg(target_arch = "x86_64")]
pub fn fbs(msr: u32, bn: u64) {
    let hh = bn as u32;
    let gd = (bn >> 32) as u32;
    unsafe {
        core::arch::asm!(
            "wrmsr",
            in("ecx") msr,
            in("eax") hh,
            in("edx") gd,
            options(nostack, preserves_flags),
        );
    }
}

#[cfg(not(target_arch = "x86_64"))]
pub fn fbs(qde: u32, msy: u64) {}






#[cfg(target_arch = "x86_64")]
pub fn ozl(awa: u32, bxj: u32) -> (u32, u32, u32, u32) {
    let eax: u32;
    let ebx: u32;
    let ecx: u32;
    let edx: u32;
    unsafe {
        
        core::arch::asm!(
            "push rbx",
            "cpuid",
            "mov {ebx_out:e}, ebx",
            "pop rbx",
            inout("eax") awa => eax,
            inout("ecx") bxj => ecx,
            ish = bd(reg) ebx,
            bd("edx") edx,
        );
    }
    (eax, ebx, ecx, edx)
}

#[cfg(not(target_arch = "x86_64"))]
pub fn ozl(yaj: u32, yde: u32) -> (u32, u32, u32, u32) {
    (0, 0, 0, 0)
}


pub fn ghj(awa: u32, bxj: u32) -> Vec<String> {
    let mut ak = Vec::new();
    let (eax, ebx, ecx, edx) = ozl(awa, bxj);
    ak.push(format!("  CPUID leaf=0x{:08x} subleaf=0x{:08x}", awa, bxj));
    ak.push(format!("  EAX = 0x{:08x}  ({:032b})", eax, eax));
    ak.push(format!("  EBX = 0x{:08x}  ({:032b})", ebx, ebx));
    ak.push(format!("  ECX = 0x{:08x}  ({:032b})", ecx, ecx));
    ak.push(format!("  EDX = 0x{:08x}  ({:032b})", edx, edx));
    
    
    match awa {
        0 => {
            
            let acs = [
                ebx.ho(),
                edx.ho(),
                ecx.ho(),
            ];
            let gvz: Vec<u8> = acs.iter().iva(|o| o.iter().hu()).collect();
            if let Ok(e) = core::str::jg(&gvz) {
                ak.push(format!("  → Vendor: \"{}\"  Max leaf: {}", e, eax));
            }
        }
        1 => {
            let bxi = eax & 0xF;
            let model = ((eax >> 4) & 0xF) | (((eax >> 16) & 0xF) << 4);
            let family = ((eax >> 8) & 0xF) + ((eax >> 20) & 0xFF);
            ak.push(format!("  → Family={} Model={} Stepping={}", family, model, bxi));
            ak.push(format!("  → Features: SSE3={} PCLMUL={} SSSE3={} SSE4.1={} SSE4.2={} AES={} AVX={}",
                ecx & 1, (ecx >> 1) & 1, (ecx >> 9) & 1, (ecx >> 19) & 1, (ecx >> 20) & 1, (ecx >> 25) & 1, (ecx >> 28) & 1));
            ak.push(format!("  → Features: FPU={} TSC={} MSR={} APIC={} SSE={} SSE2={} HTT={}",
                edx & 1, (edx >> 4) & 1, (edx >> 5) & 1, (edx >> 9) & 1, (edx >> 25) & 1, (edx >> 26) & 1, (edx >> 28) & 1));
        }
        0x8000_0002..=0x8000_0004 => {
            
            let bf: Vec<u8> = [eax, ebx, ecx, edx].iter()
                .iva(|p| p.ho())
                .collect();
            if let Ok(e) = core::str::jg(&bf) {
                ak.push(format!("  → Brand: \"{}\"", e.bdd('\0')));
            }
        }
        _ => {}
    }
    
    ak
}






pub fn vbm() {
    jjr(BDL_);
    
    crate::serial_println!("╔════════════════════════════════════════════════════════╗");
    crate::serial_println!("║              TRUSTOS CRASH DUMP                       ║");
    crate::serial_println!("╚════════════════════════════════════════════════════════╝");
    
    
    crate::serial_println!("── CPU Registers ──");
    for line in &ivz() {
        crate::serial_println!("{}", line);
    }
    
    
    crate::serial_println!("");
    for line in &ivi(32) {
        crate::serial_println!("{}", line);
    }
    
    
    #[cfg(target_arch = "x86_64")]
    {
        let rsp: u64;
        unsafe { core::arch::asm!("mov {}, rsp", bd(reg) rsp); }
        crate::serial_println!("");
        crate::serial_println!("  ── Stack Dump (RSP=0x{:016x}, 256 bytes) ──", rsp);
        let ahu = rsp as *const u8;
        for br in 0..16 {
            let l = br * 16;
            let ag = rsp + l as u64;
            let mut nu = String::new();
            let mut ascii = String::new();
            for bj in 0..16 {
                let hf = unsafe { core::ptr::read_volatile(ahu.add(l + bj)) };
                nu.t(&format!("{:02x} ", hf));
                if hf >= 0x20 && hf < 0x7f {
                    ascii.push(hf as char);
                } else {
                    ascii.push('.');
                }
            }
            crate::serial_println!("  {:016x}: {} |{}|", ag, nu, ascii);
        }
    }
    
    
    crate::serial_println!("");
    crate::serial_println!("  ── Boot Checkpoints ──");
    if let Some(log) = Apo.try_lock() {
        if log.is_empty() {
            crate::serial_println!("  <no checkpoints recorded>");
        }
        for (tsc, aj, j) in log.iter() {
            crate::serial_println!("  [TSC {:>16}] POST 0x{:02X}: {}", tsc, aj, j);
        }
    }
    
    
    crate::serial_println!("");
    crate::serial_println!("  ── Heap State ──");
    let cm = crate::devtools::jfu();
    crate::serial_println!("  allocs={}, deallocs={}, live={}, peak={}",
        cm.cok, cm.dpr, cm.czi, cm.gpe);
    
    crate::serial_println!("════════════════════════════════════════════════════════════");
    crate::serial_println!("Collect this output via serial cable (115200 8N1) for analysis.");
    
    
    qoo();
}






pub fn yob(e: &str) {
    #[cfg(target_arch = "x86_64")]
    for hf in e.bf() {
        unsafe {
            core::arch::asm!("out dx, al", in("dx") 0xE9u16, in("al") hf, options(nostack, preserves_flags));
        }
    }
}






pub fn nvq() -> Vec<String> {
    let mut ak = Vec::new();
    ak.push(String::from("  Physical Memory Map (from Limine bootloader):"));
    ak.push(String::from("  ─────────────────────────────────────────────────────────────"));
    ak.push(format!("  {:>18}  {:>18}  {:>12}  {}", "Start", "End", "Size", "Type"));
    
    
    let afx = crate::memory::tdz();
    if afx.is_empty() {
        ak.push(String::from("  <memory map not stored — add memory::store_memory_map() to boot>"));
    } else {
        let mut pvh: u64 = 0;
        let mut pvd: u64 = 0;
        for (ar, go, xnp) in &afx {
            let ci = ar + go;
            let gs = go / 1024;
            let bde = match xnp {
                0 => { pvh += go; "USABLE" },
                1 => { pvd += go; "RESERVED" },
                2 => "ACPI RECLAIM",
                3 => "ACPI NVS",
                4 => "BAD MEMORY",
                5 => "BOOTLOADER",
                6 => "KERNEL/MODULES",
                7 => "FRAMEBUFFER",
                _ => "UNKNOWN",
            };
            ak.push(format!("  0x{:016x}  0x{:016x}  {:>8} KB  {}", ar, ci, gs, bde));
        }
        ak.push(String::from("  ─────────────────────────────────────────────────────────────"));
        ak.push(format!("  Total usable: {} MB   Reserved: {} MB", pvh / 1024 / 1024, pvd / 1024 / 1024));
    }
    
    ak
}






pub fn syw() -> Vec<String> {
    let mut ak = Vec::new();
    
    ak.push(String::from("╔══════════════════════════════════════════════════════════════╗"));
    ak.push(String::from("║           TRUSTOS HARDWARE DIAGNOSTIC REPORT                ║"));
    ak.push(String::from("╚══════════════════════════════════════════════════════════════╝"));
    
    
    ak.push(String::from(""));
    ak.push(String::from("━━━ CPU IDENTIFICATION ━━━"));
    
    ak.lg(ghj(0, 0));
    ak.push(String::from(""));
    
    ak.lg(ghj(1, 0));
    ak.push(String::from(""));
    
    ak.lg(ghj(0x8000_0002, 0));
    ak.lg(ghj(0x8000_0003, 0));
    ak.lg(ghj(0x8000_0004, 0));
    
    
    ak.push(String::from(""));
    ak.push(String::from("━━━ CPU REGISTERS ━━━"));
    ak.lg(ivz());
    
    
    ak.push(String::from(""));
    ak.push(String::from("━━━ MEMORY MAP ━━━"));
    ak.lg(nvq());
    
    
    ak.push(String::from(""));
    ak.push(String::from("━━━ HEAP STATUS ━━━"));
    let cm = crate::devtools::jfu();
    ak.push(format!("  Allocations: {}   Deallocations: {}", cm.cok, cm.dpr));
    ak.push(format!("  Live allocs: {}   Peak heap: {}   Largest single: {}", cm.czi, cm.gpe, cm.etu));
    ak.push(format!("  Heap free: {} KB", crate::memory::heap::aez() / 1024));
    
    
    ak.push(String::from(""));
    ak.push(String::from("━━━ PCI DEVICES ━━━"));
    let jix = crate::pci::arx();
    if jix.is_empty() {
        ak.push(String::from("  <no PCI devices found>"));
    } else {
        for ba in &jix {
            ak.push(format!("  {:02x}:{:02x}.{} [{:04x}:{:04x}] class={:02x}{:02x} {}",
                ba.aq, ba.de, ba.gw,
                ba.ml, ba.mx,
                ba.ajz, ba.adl,
                ba.bpz()));
        }
    }
    
    
    ak.push(String::from(""));
    ak.push(String::from("━━━ BOOT CHECKPOINTS ━━━"));
    let gdm = nxv();
    if gdm.is_empty() {
        ak.push(String::from("  <no checkpoints recorded>"));
    } else {
        for (tsc, aj, j) in &gdm {
            ak.push(format!("  [TSC {:>16}] POST 0x{:02X}: {}", tsc, aj, j));
        }
    }
    
    
    ak.push(String::from(""));
    ak.push(String::from("━━━ CURRENT STACK TRACE ━━━"));
    ak.lg(ivi(16));
    
    
    ak.push(String::from(""));
    ak.push(String::from("━━━ SERIAL PORT STATUS ━━━"));
    #[cfg(target_arch = "x86_64")]
    {
        let eum = cfn(0x3F8 + 5);
        let umq = cfn(0x3F8 + 4);
        let izk = cfn(0x3F8 + 1);
        ak.push(format!("  COM1 (0x3F8): LSR=0x{:02x} MCR=0x{:02x} IER=0x{:02x}", eum, umq, izk));
        ak.push(format!("    Data Ready: {}  TX Empty: {}  Break: {}  Error: {}",
            eum & 1 != 0, eum & (1 << 5) != 0, eum & (1 << 4) != 0, eum & (1 << 7) != 0));
    }
    #[cfg(not(target_arch = "x86_64"))]
    ak.push(String::from("  <serial port status not available on this arch>"));
    
    ak.push(String::from(""));
    ak.push(String::from("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"));
    ak.push(String::from("Tip: Use serial cable (115200 8N1) to capture this output."));
    ak.push(String::from("     Run `hwdiag > serial` to send to serial only."));
    
    ak
}





static AJU_: AtomicBool = AtomicBool::new(false);
static YO_: AtomicU64 = AtomicU64::new(0);
static BIX_: AtomicU64 = AtomicU64::new(5000); 


pub fn xtw(sg: u64) {
    BIX_.store(sg, Ordering::Relaxed);
    YO_.store(0, Ordering::Relaxed);
    AJU_.store(true, Ordering::Relaxed);
    crate::serial_println!("[WATCHDOG] Enabled with {} ms timeout", sg);
}


pub fn xtx() {
    YO_.store(0, Ordering::Relaxed);
}


pub fn jwm(oog: u64) {
    if !AJU_.load(Ordering::Relaxed) {
        return;
    }
    let az = YO_.fetch_add(oog, Ordering::Relaxed) + oog;
    if az >= BIX_.load(Ordering::Relaxed) {
        YO_.store(0, Ordering::Relaxed);
        crate::serial_println!("!!! WATCHDOG TIMEOUT !!! System may be hung ({} ms)", az);
        
    }
}

pub fn xtv() {
    AJU_.store(false, Ordering::Relaxed);
    crate::serial_println!("[WATCHDOG] Disabled");
}






#[cfg(target_arch = "x86_64")]
fn ow() -> u64 {
    let hh: u32;
    let gd: u32;
    unsafe {
        core::arch::asm!("rdtsc", bd("eax") hh, bd("edx") gd, options(nostack, preserves_flags));
    }
    ((gd as u64) << 32) | (hh as u64)
}

#[cfg(not(target_arch = "x86_64"))]
fn ow() -> u64 { 0 }


fn nat(jn: u32) {
    
    
    let ay = ow();
    let cd = ay + (jn as u64) * 1_000_000; 
    while ow() < cd {
        core::hint::hc();
    }
}
