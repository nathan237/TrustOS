




use alloc::collections::BTreeMap;
use spin::Mutex;
use core::sync::atomic::{AtomicU32, Ordering};


pub mod request {
    pub const CMI_: u32 = 0;
    pub const CLX_: u32 = 1;
    pub const CLW_: u32 = 2;
    pub const CLY_: u32 = 3;
    pub const CMA_: u32 = 4;
    pub const CLZ_: u32 = 5;
    pub const CMB_: u32 = 6;
    pub const CLE_: u32 = 7;
    pub const CLQ_: u32 = 8;
    pub const CMG_: u32 = 9;
    pub const CLO_: u32 = 12;
    pub const CMF_: u32 = 13;
    pub const CLN_: u32 = 14;
    pub const CMD_: u32 = 15;
    pub const CLD_: u32 = 16;
    pub const CLF_: u32 = 17;
    pub const EBO_: u32 = 18;
    pub const EBY_: u32 = 19;
    pub const CMH_: u32 = 24;
    pub const CME_: u32 = 0x4200;
    pub const CLM_: u32 = 0x4201;
    pub const EBQ_: u32 = 0x4202;
    pub const ECA_: u32 = 0x4203;
    pub const EBP_: u32 = 0x4204;
    pub const EBZ_: u32 = 0x4205;
    pub const CMC_: u32 = 0x4206;
    pub const CLP_: u32 = 0x4207;
    pub const EBR_: u32 = 0x4208;
    pub const EBX_: u32 = 0x4209;
}


pub mod options {
    pub const EBV_: u32 = 0x00000001;
    pub const CLU_: u32 = 0x00000002;
    pub const CLV_: u32 = 0x00000004;
    pub const CLR_: u32 = 0x00000008;
    pub const CLS_: u32 = 0x00000010;
    pub const EBW_: u32 = 0x00000020;
    pub const CLT_: u32 = 0x00000040;
    pub const EBU_: u32 = 0x00000080;
    pub const EBS_: u32 = 0x00100000;
    pub const EBT_: u32 = 0x00200000;
}


pub mod events {
    pub const CLJ_: u32 = 1;
    pub const CLL_: u32 = 2;
    pub const CLG_: u32 = 3;
    pub const CLH_: u32 = 4;
    pub const EBN_: u32 = 5;
    pub const CLI_: u32 = 6;
    pub const EBM_: u32 = 7;
    pub const CLK_: u32 = 128;
}


#[derive(Clone, Copy, Debug, PartialEq)]
pub enum TraceeState {
    
    Bnk,
    
    Ai,
    
    Ani,
    
    Azp,
    
    Ayu(u32),
    
    Bge(u32),
    
    Cna,
    
    Bsl,
}


#[derive(Clone)]
pub struct TraceState {
    
    pub cnh: u32,
    
    pub g: TraceeState,
    
    pub options: u32,
    
    pub kug: u64,
    
    pub jje: u32,
}

impl TraceState {
    pub fn new() -> Self {
        Self {
            cnh: 0,
            g: TraceeState::Bnk,
            options: 0,
            kug: 0,
            jje: 0,
        }
    }
    
    pub fn etj(&self) -> bool {
        self.cnh != 0
    }
}


#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct Afi {
    pub r15: u64,
    pub r14: u64,
    pub r13: u64,
    pub r12: u64,
    pub rbp: u64,
    pub rbx: u64,
    pub r11: u64,
    pub r10: u64,
    pub r9: u64,
    pub r8: u64,
    pub rax: u64,
    pub rcx: u64,
    pub rdx: u64,
    pub rsi: u64,
    pub rdi: u64,
    pub uze: u64,
    pub pc: u64,
    pub aap: u64,
    pub rflags: u64,
    pub rsp: u64,
    pub rv: u64,
    pub kxh: u64,
    pub fjy: u64,
    pub bjw: u64,
    pub cqf: u64,
    pub fs: u64,
    pub ckx: u64,
}


#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct FpRegs {
    pub jv: u16,
    pub wwp: u16,
    pub syq: u16,
    pub svo: u16,
    pub pc: u64,
    pub vra: u64,
    pub lnd: u32,
    pub uqy: u32,
    pub wsb: [u32; 32],   
    pub xwn: [u32; 64],  
    pub ob: [u32; 24],
}

impl Default for FpRegs {
    fn default() -> Self {
        Self {
            jv: 0x37F,  
            wwp: 0,
            syq: 0,
            svo: 0,
            pc: 0,
            vra: 0,
            lnd: 0x1F80,  
            uqy: 0,
            wsb: [0; 32],
            xwn: [0; 64],
            ob: [0; 24],
        }
    }
}


static BS_: Mutex<BTreeMap<u32, TraceState>> = Mutex::new(BTreeMap::new());


static HK_: AtomicU32 = AtomicU32::new(0);


pub fn lef(ce: u32) {
    BS_.lock().insert(ce, TraceState::new());
}


pub fn khu(ce: u32) {
    let mut yh = BS_.lock();
    if let Some(g) = yh.remove(&ce) {
        if g.etj() {
            HK_.fetch_sub(1, Ordering::Relaxed);
        }
    }
}


pub fn ptrace(request: u32, ce: u32, ag: u64, f: u64) -> Result<u64, i32> {
    use request::*;
    
    match request {
        CMI_ => xla(),
        CLX_ | CLW_ => amm(ce, ag),
        CLY_ => vgl(ce, ag),
        CMA_ | CLZ_ => luq(ce, ag, f),
        CMB_ => vjs(ce, ag, f),
        CLE_ => rod(ce, f as u32),
        CLQ_ => dsm(ce),
        CMG_ => woy(ce, f as u32),
        CLO_ => tfi(ce, f as *mut Afi),
        CMF_ => wkb(ce, f as *const Afi),
        CLN_ => tfh(ce, f as *mut FpRegs),
        CMD_ => wjz(ce, f as *const FpRegs),
        CLD_ => dyl(ce),
        CLF_ => rwl(ce, f as u32),
        CMH_ => syscall(ce, f as u32),
        CME_ => wka(ce, f as u32),
        CLM_ => tfg(ce, f as *mut u64),
        CMC_ => wgp(ce, f as u32),
        CLP_ => gkb(ce),
        _ => Err(-22), 
    }
}


fn xla() -> Result<u64, i32> {
    let aei = crate::process::aei();
    let lsc = crate::process::lsc(aei).ok_or(-3)?; 
    
    let mut yh = BS_.lock();
    let g = yh.ds(&aei).ok_or(-3)?;
    
    if g.etj() {
        return Err(-1); 
    }
    
    g.cnh = lsc;
    g.g = TraceeState::Ai;
    HK_.fetch_add(1, Ordering::Relaxed);
    
    Ok(0)
}


fn dyl(ce: u32) -> Result<u64, i32> {
    let cnh = crate::process::aei();
    
    
    if ce == cnh {
        return Err(-1); 
    }
    
    let mut yh = BS_.lock();
    let g = yh.ds(&ce).ok_or(-3)?; 
    
    if g.etj() {
        return Err(-1); 
    }
    
    
    
    g.cnh = cnh;
    g.g = TraceeState::Ayu(crate::signals::sig::Qq);
    HK_.fetch_add(1, Ordering::Relaxed);
    
    
    crate::process::qg(ce);
    
    Ok(0)
}


fn wgp(ce: u32, options: u32) -> Result<u64, i32> {
    let cnh = crate::process::aei();
    
    if ce == cnh {
        return Err(-1); 
    }
    
    let mut yh = BS_.lock();
    let g = yh.ds(&ce).ok_or(-3)?;
    
    if g.etj() {
        return Err(-1); 
    }
    
    g.cnh = cnh;
    g.g = TraceeState::Bsl;
    g.options = options;
    HK_.fetch_add(1, Ordering::Relaxed);
    
    Ok(0)
}


fn rwl(ce: u32, cug: u32) -> Result<u64, i32> {
    byp(ce)?;
    
    let mut yh = BS_.lock();
    let g = yh.ds(&ce).ok_or(-3)?;
    
    g.cnh = 0;
    g.g = TraceeState::Bnk;
    g.options = 0;
    HK_.fetch_sub(1, Ordering::Relaxed);
    
    drop(yh);
    
    
    if cug != 0 {
        let _ = crate::signals::dsm(ce, cug, crate::process::aei());
    }
    crate::process::anu(ce);
    
    Ok(0)
}


fn rod(ce: u32, cug: u32) -> Result<u64, i32> {
    byp(ce)?;
    
    let mut yh = BS_.lock();
    let g = yh.ds(&ce).ok_or(-3)?;
    g.g = TraceeState::Ai;
    g.jje = cug;
    drop(yh);
    
    crate::process::anu(ce);
    Ok(0)
}


fn woy(ce: u32, cug: u32) -> Result<u64, i32> {
    byp(ce)?;
    
    let mut yh = BS_.lock();
    let g = yh.ds(&ce).ok_or(-3)?;
    g.g = TraceeState::Cna;
    g.jje = cug;
    drop(yh);
    
    
    if let Some(mut be) = crate::process::ghz(ce) {
        be.rflags |= 1 << 8; 
        let _ = crate::process::meh(ce, &be);
    }
    
    crate::process::anu(ce);
    Ok(0)
}


fn syscall(ce: u32, cug: u32) -> Result<u64, i32> {
    byp(ce)?;
    
    let mut yh = BS_.lock();
    let g = yh.ds(&ce).ok_or(-3)?;
    
    
    g.g = match g.g {
        TraceeState::Ani => TraceeState::Azp,
        _ => TraceeState::Ani,
    };
    g.jje = cug;
    drop(yh);
    
    crate::process::anu(ce);
    Ok(0)
}


fn dsm(ce: u32) -> Result<u64, i32> {
    byp(ce)?;
    
    let mut yh = BS_.lock();
    yh.remove(&ce);
    HK_.fetch_sub(1, Ordering::Relaxed);
    drop(yh);
    
    crate::process::fwl(ce);
    Ok(0)
}


fn gkb(ce: u32) -> Result<u64, i32> {
    byp(ce)?;
    
    let mut yh = BS_.lock();
    let g = yh.ds(&ce).ok_or(-3)?;
    
    if g.g != TraceeState::Bsl {
        return Err(-22); 
    }
    
    g.g = TraceeState::Bge(events::CLK_);
    drop(yh);
    
    crate::process::qg(ce);
    Ok(0)
}


fn wka(ce: u32, options: u32) -> Result<u64, i32> {
    byp(ce)?;
    
    let mut yh = BS_.lock();
    let g = yh.ds(&ce).ok_or(-3)?;
    g.options = options;
    
    Ok(0)
}


fn tfg(ce: u32, cax: *mut u64) -> Result<u64, i32> {
    byp(ce)?;
    
    let yh = BS_.lock();
    let g = yh.get(&ce).ok_or(-3)?;
    
    
    if !cax.abq() {
        unsafe { *cax = g.kug; }
    }
    
    Ok(0)
}


fn amm(ce: u32, ag: u64) -> Result<u64, i32> {
    byp(ce)?;
    
    
    let bn = crate::memory::vst(ce, ag)?;
    Ok(bn)
}


fn vgl(ce: u32, l: u64) -> Result<u64, i32> {
    byp(ce)?;
    
    let be = crate::process::ghz(ce).ok_or(-3i32)?;
    
    let ap = match l {
        0   => be.r15, 8   => be.r14, 16  => be.r13, 24  => be.r12,
        32  => be.rbp, 40  => be.rbx, 48  => be.r11, 56  => be.r10,
        64  => be.r9,  72  => be.r8,  80  => be.rax, 88  => be.rcx,
        96  => be.rdx, 104 => be.rsi, 112 => be.rdi, 120 => be.rax, 
        128 => be.pc, 136 => be.aap,  144 => be.rflags,
        152 => be.rsp, 160 => be.rv,
        _ => return Err(-14), 
    };
    Ok(ap)
}


fn luq(ce: u32, ag: u64, f: u64) -> Result<u64, i32> {
    byp(ce)?;
    
    
    crate::memory::xvu(ce, ag, f)?;
    Ok(0)
}


fn vjs(ce: u32, l: u64, f: u64) -> Result<u64, i32> {
    byp(ce)?;
    
    let mut be = crate::process::ghz(ce).ok_or(-3i32)?;
    match l {
        0   => be.r15 = f, 8   => be.r14 = f,
        16  => be.r13 = f, 24  => be.r12 = f,
        32  => be.rbp = f, 40  => be.rbx = f,
        48  => be.r11 = f, 56  => be.r10 = f,
        64  => be.r9 = f,  72  => be.r8 = f,
        80  => be.rax = f, 88  => be.rcx = f,
        96  => be.rdx = f, 104 => be.rsi = f,
        112 => be.rdi = f, 128 => be.pc = f,
        136 => be.aap = f,  144 => be.rflags = f,
        152 => be.rsp = f, 160 => be.rv = f,
        _ => return Err(-14), 
    }
    crate::process::meh(ce, &be).jd(|_| -3i32)?;
    Ok(0)
}


fn tfi(ce: u32, regs: *mut Afi) -> Result<u64, i32> {
    byp(ce)?;
    
    if regs.abq() {
        return Err(-14); 
    }
    
    let be = crate::process::ghz(ce).ok_or(-3i32)?;
    let bfo = Afi {
        r15: be.r15, r14: be.r14, r13: be.r13, r12: be.r12,
        rbp: be.rbp, rbx: be.rbx, r11: be.r11, r10: be.r10,
        r9: be.r9, r8: be.r8, rax: be.rax, rcx: be.rcx,
        rdx: be.rdx, rsi: be.rsi, rdi: be.rdi,
        uze: be.rax, 
        pc: be.pc, aap: be.aap, rflags: be.rflags,
        rsp: be.rsp, rv: be.rv,
        kxh: 0, fjy: 0, bjw: 0, cqf: 0, fs: 0, ckx: 0,
    };
    unsafe { *regs = bfo; }
    
    Ok(0)
}


fn wkb(ce: u32, regs: *const Afi) -> Result<u64, i32> {
    byp(ce)?;
    
    if regs.abq() {
        return Err(-14); 
    }
    
    let bfo = unsafe { &*regs };
    let mut be = crate::process::ghz(ce).ok_or(-3i32)?;
    be.r15 = bfo.r15; be.r14 = bfo.r14;
    be.r13 = bfo.r13; be.r12 = bfo.r12;
    be.rbp = bfo.rbp; be.rbx = bfo.rbx;
    be.r11 = bfo.r11; be.r10 = bfo.r10;
    be.r9 = bfo.r9; be.r8 = bfo.r8;
    be.rax = bfo.rax; be.rcx = bfo.rcx;
    be.rdx = bfo.rdx; be.rsi = bfo.rsi;
    be.rdi = bfo.rdi; be.pc = bfo.pc;
    be.aap = bfo.aap; be.rflags = bfo.rflags;
    be.rsp = bfo.rsp; be.rv = bfo.rv;
    crate::process::meh(ce, &be).jd(|_| -3i32)?;
    
    Ok(0)
}


fn tfh(ce: u32, regs: *mut FpRegs) -> Result<u64, i32> {
    byp(ce)?;
    
    if regs.abq() {
        return Err(-14); 
    }
    
    let swm = FpRegs::default();
    unsafe { *regs = swm; }
    
    Ok(0)
}


fn wjz(ce: u32, regs: *const FpRegs) -> Result<u64, i32> {
    byp(ce)?;
    
    if regs.abq() {
        return Err(-14); 
    }
    
    
    Ok(0)
}


fn byp(ce: u32) -> Result<(), i32> {
    let aei = crate::process::aei();
    
    let yh = BS_.lock();
    let g = yh.get(&ce).ok_or(-3)?; 
    
    if g.cnh != aei {
        return Err(-3); 
    }
    
    Ok(())
}


pub fn etj(ce: u32) -> bool {
    BS_.lock()
        .get(&ce)
        .map(|e| e.etj())
        .unwrap_or(false)
}


pub fn tex(ce: u32) -> Option<u32> {
    BS_.lock()
        .get(&ce)
        .hi(|e| e.etj())
        .map(|e| e.cnh)
}


pub fn zdz(ce: u32, id: u32, fr: u64) {
    let mut yh = BS_.lock();
    if let Some(g) = yh.ds(&ce) {
        if g.etj() && (g.options & snt(id)) != 0 {
            g.g = TraceeState::Bge(id);
            g.kug = fr;
            
            
            drop(yh);
            crate::process::qg(ce);
            
            
            if let Some(xlb) = tex(ce) {
                let _ = crate::signals::dsm(xlb, crate::signals::sig::Ayf, ce);
            }
        }
    }
}


fn snt(id: u32) -> u32 {
    match id {
        events::CLJ_ => options::CLU_,
        events::CLL_ => options::CLV_,
        events::CLG_ => options::CLR_,
        events::CLH_ => options::CLS_,
        events::CLI_ => options::CLT_,
        _ => 0,
    }
}


pub fn zqt(ce: u32) -> bool {
    let mut yh = BS_.lock();
    if let Some(g) = yh.ds(&ce) {
        if g.g == TraceeState::Ani {
            g.g = TraceeState::Ayu(crate::signals::sig::Ayh);
            drop(yh);
            crate::process::qg(ce);
            return true;
        }
    }
    false
}


pub fn zqu(ce: u32) -> bool {
    let mut yh = BS_.lock();
    if let Some(g) = yh.ds(&ce) {
        if g.g == TraceeState::Azp {
            g.g = TraceeState::Ayu(crate::signals::sig::Ayh);
            drop(yh);
            crate::process::qg(ce);
            return true;
        }
    }
    false
}


pub fn gxu() -> u32 {
    HK_.load(Ordering::Relaxed)
}
