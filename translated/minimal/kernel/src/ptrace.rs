




use alloc::collections::BTreeMap;
use spin::Mutex;
use core::sync::atomic::{AtomicU32, Ordering};


pub mod request {
    pub const CPR_: u32 = 0;
    pub const CPG_: u32 = 1;
    pub const CPF_: u32 = 2;
    pub const CPH_: u32 = 3;
    pub const CPJ_: u32 = 4;
    pub const CPI_: u32 = 5;
    pub const CPK_: u32 = 6;
    pub const CON_: u32 = 7;
    pub const COZ_: u32 = 8;
    pub const CPP_: u32 = 9;
    pub const COX_: u32 = 12;
    pub const CPO_: u32 = 13;
    pub const COW_: u32 = 14;
    pub const CPM_: u32 = 15;
    pub const COM_: u32 = 16;
    pub const COO_: u32 = 17;
    pub const EFE_: u32 = 18;
    pub const EFO_: u32 = 19;
    pub const CPQ_: u32 = 24;
    pub const CPN_: u32 = 0x4200;
    pub const COV_: u32 = 0x4201;
    pub const EFG_: u32 = 0x4202;
    pub const EFQ_: u32 = 0x4203;
    pub const EFF_: u32 = 0x4204;
    pub const EFP_: u32 = 0x4205;
    pub const CPL_: u32 = 0x4206;
    pub const COY_: u32 = 0x4207;
    pub const EFH_: u32 = 0x4208;
    pub const EFN_: u32 = 0x4209;
}


pub mod options {
    pub const EFL_: u32 = 0x00000001;
    pub const CPD_: u32 = 0x00000002;
    pub const CPE_: u32 = 0x00000004;
    pub const CPA_: u32 = 0x00000008;
    pub const CPB_: u32 = 0x00000010;
    pub const EFM_: u32 = 0x00000020;
    pub const CPC_: u32 = 0x00000040;
    pub const EFK_: u32 = 0x00000080;
    pub const EFI_: u32 = 0x00100000;
    pub const EFJ_: u32 = 0x00200000;
}


pub mod events {
    pub const COS_: u32 = 1;
    pub const COU_: u32 = 2;
    pub const COP_: u32 = 3;
    pub const COQ_: u32 = 4;
    pub const EFD_: u32 = 5;
    pub const COR_: u32 = 6;
    pub const EFC_: u32 = 7;
    pub const COT_: u32 = 128;
}


#[derive(Clone, Copy, Debug, PartialEq)]
pub enum TraceeState {
    
    NotTraced,
    
    Running,
    
    SyscallEntry,
    
    SyscallExit,
    
    SignalStop(u32),
    
    EventStop(u32),
    
    SingleStep,
    
    Seized,
}


#[derive(Clone)]
pub struct TraceState {
    
    pub tracer_pid: u32,
    
    pub state: TraceeState,
    
    pub options: u32,
    
    pub event_msg: u64,
    
    pub pending_signal: u32,
}

impl TraceState {
    pub fn new() -> Self {
        Self {
            tracer_pid: 0,
            state: TraceeState::NotTraced,
            options: 0,
            event_msg: 0,
            pending_signal: 0,
        }
    }
    
    pub fn is_traced(&self) -> bool {
        self.tracer_pid != 0
    }
}


#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct Nr {
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
    pub orig_rax: u64,
    pub rip: u64,
    pub cs: u64,
    pub rflags: u64,
    pub rsp: u64,
    pub ss: u64,
    pub fxw: u64,
    pub gs_base: u64,
    pub ds: u64,
    pub es: u64,
    pub fs: u64,
    pub gs: u64,
}


#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct FpRegs {
    pub cwd: u16,
    pub swd: u16,
    pub ftw: u16,
    pub fop: u16,
    pub rip: u64,
    pub rdp: u64,
    pub mxcsr: u32,
    pub mxcr_mask: u32,
    pub st_space: [u32; 32],   
    pub xmm_space: [u32; 64],  
    pub padding: [u32; 24],
}

impl Default for FpRegs {
    fn default() -> Self {
        Self {
            cwd: 0x37F,  
            swd: 0,
            ftw: 0,
            fop: 0,
            rip: 0,
            rdp: 0,
            mxcsr: 0x1F80,  
            mxcr_mask: 0,
            st_space: [0; 32],
            xmm_space: [0; 64],
            padding: [0; 24],
        }
    }
}


static BU_: Mutex<BTreeMap<u32, TraceState>> = Mutex::new(BTreeMap::new());


static IC_: AtomicU32 = AtomicU32::new(0);


pub fn gcp(pid: u32) {
    BU_.lock().insert(pid, TraceState::new());
}


pub fn flu(pid: u32) {
    let mut lb = BU_.lock();
    if let Some(state) = lb.remove(&pid) {
        if state.is_traced() {
            IC_.fetch_sub(1, Ordering::Relaxed);
        }
    }
}


pub fn ptrace(request: u32, pid: u32, addr: u64, data: u64) -> Result<u64, i32> {
    use request::*;
    
    match request {
        CPR_ => pmo(),
        CPG_ | CPF_ => peek(pid, addr),
        CPH_ => ntl(pid, addr),
        CPJ_ | CPI_ => gnq(pid, addr, data),
        CPK_ => nvz(pid, addr, data),
        CON_ => kxh(pid, data as u32),
        COZ_ => bne(pid),
        CPP_ => oth(pid, data as u32),
        COX_ => meg(pid, data as *mut Nr),
        CPO_ => opx(pid, data as *const Nr),
        COW_ => mef(pid, data as *mut FpRegs),
        CPM_ => opv(pid, data as *const FpRegs),
        COM_ => attach(pid),
        COO_ => lds(pid, data as u32),
        CPQ_ => syscall(pid, data as u32),
        CPN_ => opw(pid, data as u32),
        COV_ => mee(pid, data as *mut u64),
        CPL_ => one(pid, data as u32),
        COY_ => interrupt(pid),
        _ => Err(-22), 
    }
}


fn pmo() -> Result<u64, i32> {
    let pe = crate::process::pe();
    let gmc = crate::process::gmc(pe).ok_or(-3)?; 
    
    let mut lb = BU_.lock();
    let state = lb.get_mut(&pe).ok_or(-3)?;
    
    if state.is_traced() {
        return Err(-1); 
    }
    
    state.tracer_pid = gmc;
    state.state = TraceeState::Running;
    IC_.fetch_add(1, Ordering::Relaxed);
    
    Ok(0)
}


fn attach(pid: u32) -> Result<u64, i32> {
    let tracer_pid = crate::process::pe();
    
    
    if pid == tracer_pid {
        return Err(-1); 
    }
    
    let mut lb = BU_.lock();
    let state = lb.get_mut(&pid).ok_or(-3)?; 
    
    if state.is_traced() {
        return Err(-1); 
    }
    
    
    
    state.tracer_pid = tracer_pid;
    state.state = TraceeState::SignalStop(crate::signals::sig::Hb);
    IC_.fetch_add(1, Ordering::Relaxed);
    
    
    crate::process::stop(pid);
    
    Ok(0)
}


fn one(pid: u32, options: u32) -> Result<u64, i32> {
    let tracer_pid = crate::process::pe();
    
    if pid == tracer_pid {
        return Err(-1); 
    }
    
    let mut lb = BU_.lock();
    let state = lb.get_mut(&pid).ok_or(-3)?;
    
    if state.is_traced() {
        return Err(-1); 
    }
    
    state.tracer_pid = tracer_pid;
    state.state = TraceeState::Seized;
    state.options = options;
    IC_.fetch_add(1, Ordering::Relaxed);
    
    Ok(0)
}


fn lds(pid: u32, ash: u32) -> Result<u64, i32> {
    ans(pid)?;
    
    let mut lb = BU_.lock();
    let state = lb.get_mut(&pid).ok_or(-3)?;
    
    state.tracer_pid = 0;
    state.state = TraceeState::NotTraced;
    state.options = 0;
    IC_.fetch_sub(1, Ordering::Relaxed);
    
    drop(lb);
    
    
    if ash != 0 {
        let _ = crate::signals::bne(pid, ash, crate::process::pe());
    }
    crate::process::resume(pid);
    
    Ok(0)
}


fn kxh(pid: u32, ash: u32) -> Result<u64, i32> {
    ans(pid)?;
    
    let mut lb = BU_.lock();
    let state = lb.get_mut(&pid).ok_or(-3)?;
    state.state = TraceeState::Running;
    state.pending_signal = ash;
    drop(lb);
    
    crate::process::resume(pid);
    Ok(0)
}


fn oth(pid: u32, ash: u32) -> Result<u64, i32> {
    ans(pid)?;
    
    let mut lb = BU_.lock();
    let state = lb.get_mut(&pid).ok_or(-3)?;
    state.state = TraceeState::SingleStep;
    state.pending_signal = ash;
    drop(lb);
    
    
    if let Some(mut ab) = crate::process::cyj(pid) {
        ab.rflags |= 1 << 8; 
        let _ = crate::process::gug(pid, &ab);
    }
    
    crate::process::resume(pid);
    Ok(0)
}


fn syscall(pid: u32, ash: u32) -> Result<u64, i32> {
    ans(pid)?;
    
    let mut lb = BU_.lock();
    let state = lb.get_mut(&pid).ok_or(-3)?;
    
    
    state.state = match state.state {
        TraceeState::SyscallEntry => TraceeState::SyscallExit,
        _ => TraceeState::SyscallEntry,
    };
    state.pending_signal = ash;
    drop(lb);
    
    crate::process::resume(pid);
    Ok(0)
}


fn bne(pid: u32) -> Result<u64, i32> {
    ans(pid)?;
    
    let mut lb = BU_.lock();
    lb.remove(&pid);
    IC_.fetch_sub(1, Ordering::Relaxed);
    drop(lb);
    
    crate::process::crk(pid);
    Ok(0)
}


fn interrupt(pid: u32) -> Result<u64, i32> {
    ans(pid)?;
    
    let mut lb = BU_.lock();
    let state = lb.get_mut(&pid).ok_or(-3)?;
    
    if state.state != TraceeState::Seized {
        return Err(-22); 
    }
    
    state.state = TraceeState::EventStop(events::COT_);
    drop(lb);
    
    crate::process::stop(pid);
    Ok(0)
}


fn opw(pid: u32, options: u32) -> Result<u64, i32> {
    ans(pid)?;
    
    let mut lb = BU_.lock();
    let state = lb.get_mut(&pid).ok_or(-3)?;
    state.options = options;
    
    Ok(0)
}


fn mee(pid: u32, msg_ptr: *mut u64) -> Result<u64, i32> {
    ans(pid)?;
    
    let lb = BU_.lock();
    let state = lb.get(&pid).ok_or(-3)?;
    
    
    if !msg_ptr.is_null() {
        unsafe { *msg_ptr = state.event_msg; }
    }
    
    Ok(0)
}


fn peek(pid: u32, addr: u64) -> Result<u64, i32> {
    ans(pid)?;
    
    
    let value = crate::memory::odj(pid, addr)?;
    Ok(value)
}


fn ntl(pid: u32, offset: u64) -> Result<u64, i32> {
    ans(pid)?;
    
    let ab = crate::process::cyj(pid).ok_or(-3i32)?;
    
    let val = match offset {
        0   => ab.r15, 8   => ab.r14, 16  => ab.r13, 24  => ab.r12,
        32  => ab.rbp, 40  => ab.rbx, 48  => ab.r11, 56  => ab.r10,
        64  => ab.r9,  72  => ab.r8,  80  => ab.rax, 88  => ab.rcx,
        96  => ab.rdx, 104 => ab.rsi, 112 => ab.rdi, 120 => ab.rax, 
        128 => ab.rip, 136 => ab.cs,  144 => ab.rflags,
        152 => ab.rsp, 160 => ab.ss,
        _ => return Err(-14), 
    };
    Ok(val)
}


fn gnq(pid: u32, addr: u64, data: u64) -> Result<u64, i32> {
    ans(pid)?;
    
    
    crate::memory::pvf(pid, addr, data)?;
    Ok(0)
}


fn nvz(pid: u32, offset: u64, data: u64) -> Result<u64, i32> {
    ans(pid)?;
    
    let mut ab = crate::process::cyj(pid).ok_or(-3i32)?;
    match offset {
        0   => ab.r15 = data, 8   => ab.r14 = data,
        16  => ab.r13 = data, 24  => ab.r12 = data,
        32  => ab.rbp = data, 40  => ab.rbx = data,
        48  => ab.r11 = data, 56  => ab.r10 = data,
        64  => ab.r9 = data,  72  => ab.r8 = data,
        80  => ab.rax = data, 88  => ab.rcx = data,
        96  => ab.rdx = data, 104 => ab.rsi = data,
        112 => ab.rdi = data, 128 => ab.rip = data,
        136 => ab.cs = data,  144 => ab.rflags = data,
        152 => ab.rsp = data, 160 => ab.ss = data,
        _ => return Err(-14), 
    }
    crate::process::gug(pid, &ab).map_err(|_| -3i32)?;
    Ok(0)
}


fn meg(pid: u32, regs: *mut Nr) -> Result<u64, i32> {
    ans(pid)?;
    
    if regs.is_null() {
        return Err(-14); 
    }
    
    let ab = crate::process::cyj(pid).ok_or(-3i32)?;
    let aek = Nr {
        r15: ab.r15, r14: ab.r14, r13: ab.r13, r12: ab.r12,
        rbp: ab.rbp, rbx: ab.rbx, r11: ab.r11, r10: ab.r10,
        r9: ab.r9, r8: ab.r8, rax: ab.rax, rcx: ab.rcx,
        rdx: ab.rdx, rsi: ab.rsi, rdi: ab.rdi,
        orig_rax: ab.rax, 
        rip: ab.rip, cs: ab.cs, rflags: ab.rflags,
        rsp: ab.rsp, ss: ab.ss,
        fxw: 0, gs_base: 0, ds: 0, es: 0, fs: 0, gs: 0,
    };
    unsafe { *regs = aek; }
    
    Ok(0)
}


fn opx(pid: u32, regs: *const Nr) -> Result<u64, i32> {
    ans(pid)?;
    
    if regs.is_null() {
        return Err(-14); 
    }
    
    let aek = unsafe { &*regs };
    let mut ab = crate::process::cyj(pid).ok_or(-3i32)?;
    ab.r15 = aek.r15; ab.r14 = aek.r14;
    ab.r13 = aek.r13; ab.r12 = aek.r12;
    ab.rbp = aek.rbp; ab.rbx = aek.rbx;
    ab.r11 = aek.r11; ab.r10 = aek.r10;
    ab.r9 = aek.r9; ab.r8 = aek.r8;
    ab.rax = aek.rax; ab.rcx = aek.rcx;
    ab.rdx = aek.rdx; ab.rsi = aek.rsi;
    ab.rdi = aek.rdi; ab.rip = aek.rip;
    ab.cs = aek.cs; ab.rflags = aek.rflags;
    ab.rsp = aek.rsp; ab.ss = aek.ss;
    crate::process::gug(pid, &ab).map_err(|_| -3i32)?;
    
    Ok(0)
}


fn mef(pid: u32, regs: *mut FpRegs) -> Result<u64, i32> {
    ans(pid)?;
    
    if regs.is_null() {
        return Err(-14); 
    }
    
    let lyd = FpRegs::default();
    unsafe { *regs = lyd; }
    
    Ok(0)
}


fn opv(pid: u32, regs: *const FpRegs) -> Result<u64, i32> {
    ans(pid)?;
    
    if regs.is_null() {
        return Err(-14); 
    }
    
    
    Ok(0)
}


fn ans(pid: u32) -> Result<(), i32> {
    let pe = crate::process::pe();
    
    let lb = BU_.lock();
    let state = lb.get(&pid).ok_or(-3)?; 
    
    if state.tracer_pid != pe {
        return Err(-3); 
    }
    
    Ok(())
}


pub fn is_traced(pid: u32) -> bool {
    BU_.lock()
        .get(&pid)
        .map(|j| j.is_traced())
        .unwrap_or(false)
}


pub fn mdy(pid: u32) -> Option<u32> {
    BU_.lock()
        .get(&pid)
        .filter(|j| j.is_traced())
        .map(|j| j.tracer_pid)
}


pub fn qpt(pid: u32, event: u32, bk: u64) {
    let mut lb = BU_.lock();
    if let Some(state) = lb.get_mut(&pid) {
        if state.is_traced() && (state.options & lrp(event)) != 0 {
            state.state = TraceeState::EventStop(event);
            state.event_msg = bk;
            
            
            drop(lb);
            crate::process::stop(pid);
            
            
            if let Some(tracer) = mdy(pid) {
                let _ = crate::signals::bne(tracer, crate::signals::sig::Uu, pid);
            }
        }
    }
}


fn lrp(event: u32) -> u32 {
    match event {
        events::COS_ => options::CPD_,
        events::COU_ => options::CPE_,
        events::COP_ => options::CPA_,
        events::COQ_ => options::CPB_,
        events::COR_ => options::CPC_,
        _ => 0,
    }
}


pub fn qyo(pid: u32) -> bool {
    let mut lb = BU_.lock();
    if let Some(state) = lb.get_mut(&pid) {
        if state.state == TraceeState::SyscallEntry {
            state.state = TraceeState::SignalStop(crate::signals::sig::Uw);
            drop(lb);
            crate::process::stop(pid);
            return true;
        }
    }
    false
}


pub fn qyp(pid: u32) -> bool {
    let mut lb = BU_.lock();
    if let Some(state) = lb.get_mut(&pid) {
        if state.state == TraceeState::SyscallExit {
            state.state = TraceeState::SignalStop(crate::signals::sig::Uw);
            drop(lb);
            crate::process::stop(pid);
            return true;
        }
    }
    false
}


pub fn active_count() -> u32 {
    IC_.load(Ordering::Relaxed)
}
