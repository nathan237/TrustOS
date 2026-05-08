




use alloc::collections::BTreeMap;
use alloc::vec::Vec;
use core::sync::atomic::{AtomicU64, Ordering};
use spin::Mutex;


pub mod sig {
    pub const Adx: u32 = 1;
    pub const Adz: u32 = 2;
    pub const Aeb: u32 = 3;
    pub const Ady: u32 = 4;
    pub const Uw: u32 = 5;
    pub const Ads: u32 = 6;
    pub const Adu: u32 = 7;
    pub const Adw: u32 = 8;
    pub const Iv: u32 = 9;  
    pub const Aef: u32 = 10;
    pub const Aec: u32 = 11;
    pub const Aeg: u32 = 12;
    pub const Aea: u32 = 13;
    pub const Adt: u32 = 14;
    pub const Aed: u32 = 15;
    pub const Bcm: u32 = 16;
    pub const Uu: u32 = 17;
    pub const Adv: u32 = 18;
    pub const Hb: u32 = 19; 
    pub const Aee: u32 = 20;
    pub const Apj: u32 = 21;
    pub const Apk: u32 = 22;
    pub const Apl: u32 = 23;
    pub const Bco: u32 = 24;
    pub const Bcp: u32 = 25;
    pub const Bcn: u32 = 26;
    pub const Bci: u32 = 27;
    pub const Apm: u32 = 28;
    pub const Bcf: u32 = 29;
    pub const Bcj: u32 = 30;
    pub const Api: u32 = 31;
    
    
    pub const Bcl: u32 = 32;
    pub const Bck: u32 = 64;
    
    pub const HK_: usize = 65;
}


pub mod sa_flags {
    pub const EHL_: u64 = 0x00000001;
    pub const EHM_: u64 = 0x00000002;
    pub const EHP_: u64 = 0x00000004;
    pub const EHN_: u64 = 0x08000000;
    pub const EHO_: u64 = 0x10000000;
    pub const CTT_: u64 = 0x40000000;
    pub const CTU_: u64 = 0x80000000;
    pub const CTV_: u64 = 0x04000000;
}


pub const YI_: u64 = 0;  
pub const BIA_: u64 = 1;  
pub const EKD_: u64 = u64::MAX;


#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct SigAction {
    
    pub sa_handler: u64,
    
    pub sa_flags: u64,
    
    pub sa_restorer: u64,
    
    pub sa_mask: u64,
}

impl Default for SigAction {
    fn default() -> Self {
        Self {
            sa_handler: YI_,
            sa_flags: 0,
            sa_restorer: 0,
            sa_mask: 0,
        }
    }
}


#[derive(Clone, Debug)]
pub struct Acm {
    pub signo: u32,
    pub def: u32,
    pub timestamp: u64,
}


pub struct SignalState {
    
    pub actions: [SigAction; sig::HK_],
    
    pub pending: AtomicU64,
    
    pub blocked: AtomicU64,
    
    pub pending_queue: Vec<Acm>,
}

impl SignalState {
    pub fn new() -> Self {
        Self {
            actions: [SigAction::default(); sig::HK_],
            pending: AtomicU64::new(0),
            blocked: AtomicU64::new(0),
            pending_queue: Vec::new(),
        }
    }
    
    
    pub fn set_action(&mut self, signo: u32, action: SigAction) -> Result<SigAction, i32> {
        if signo == 0 || signo as usize >= sig::HK_ {
            return Err(-22); 
        }
        
        
        if signo == sig::Iv || signo == sig::Hb {
            return Err(-22); 
        }
        
        let qb = self.actions[signo as usize];
        self.actions[signo as usize] = action;
        Ok(qb)
    }
    
    
    pub fn get_action(&self, signo: u32) -> Option<&SigAction> {
        if signo as usize >= sig::HK_ {
            return None;
        }
        Some(&self.actions[signo as usize])
    }
    
    
    pub fn post_signal(&mut self, signo: u32, def: u32) {
        if signo == 0 || signo as usize >= sig::HK_ {
            return;
        }
        
        
        self.pending.fetch_or(1 << signo, Ordering::SeqCst);
        
        
        self.pending_queue.push(Acm {
            signo,
            def,
            timestamp: crate::time::cbx(),
        });
    }
    
    
    pub fn qmt(&self, signo: u32) -> bool {
        if signo as usize >= sig::HK_ {
            return false;
        }
        (self.pending.load(Ordering::Relaxed) & (1 << signo)) != 0
    }
    
    
    pub fn qmc(&self, signo: u32) -> bool {
        if signo as usize >= sig::HK_ {
            return false;
        }
        
        if signo == sig::Iv || signo == sig::Hb {
            return false;
        }
        (self.blocked.load(Ordering::Relaxed) & (1 << signo)) != 0
    }
    
    
    pub fn get_deliverable(&self) -> Option<u32> {
        let pending = self.pending.load(Ordering::Relaxed);
        let blocked = self.blocked.load(Ordering::Relaxed);
        let hrf = pending & !blocked;
        
        if hrf == 0 {
            return None;
        }
        
        
        Some(hrf.trailing_zeros())
    }
    
    
    pub fn clear_pending(&mut self, signo: u32) {
        if signo as usize >= sig::HK_ {
            return;
        }
        self.pending.fetch_and(!(1 << signo), Ordering::SeqCst);
        self.pending_queue.retain(|j| j.signo != signo);
    }
    
    
    pub fn block(&self, mask: u64) {
        self.blocked.fetch_or(mask, Ordering::SeqCst);
    }
    
    
    pub fn unblock(&self, mask: u64) {
        self.blocked.fetch_and(!mask, Ordering::SeqCst);
    }
    
    
    pub fn set_blocked(&self, mask: u64) {
        
        let mask = mask & !((1 << sig::Iv) | (1 << sig::Hb));
        self.blocked.store(mask, Ordering::SeqCst);
    }
}


static FG_: Mutex<BTreeMap<u32, SignalState>> = Mutex::new(BTreeMap::new());


pub fn gcp(pid: u32) {
    FG_.lock().insert(pid, SignalState::new());
}


pub fn flu(pid: u32) {
    FG_.lock().remove(&pid);
}


pub fn bne(bwg: u32, signo: u32, def: u32) -> Result<(), i32> {
    if signo == 0 {
        
        let exists = FG_.lock().contains_key(&bwg);
        return if exists { Ok(()) } else { Err(-3) }; 
    }
    
    let mut lb = FG_.lock();
    let state = lb.get_mut(&bwg).ok_or(-3)?; 
    
    state.post_signal(signo, def);
    
    
    if signo == sig::Iv || signo == sig::Hb {
        mht(bwg, signo);
    }
    
    Ok(())
}


pub fn geu(pgid: u32, signo: u32) -> Result<(), i32> {
    let sender = crate::process::pe();
    let iut = crate::process::nun(pgid);
    if iut.is_empty() {
        return Err(-3); 
    }
    for pid in iut {
        let _ = bne(pid, signo, sender);
    }
    Ok(())
}


fn mht(pid: u32, signo: u32) {
    match signo {
        sig::Iv => {
            
            crate::process::crk(pid);
        }
        sig::Hb => {
            
            crate::process::stop(pid);
        }
        _ => {}
    }
}


pub fn set_action(pid: u32, signo: u32, action: SigAction) -> Result<SigAction, i32> {
    let mut lb = FG_.lock();
    let state = lb.get_mut(&pid).ok_or(-3)?;
    state.set_action(signo, action)
}


pub fn get_action(pid: u32, signo: u32) -> Result<SigAction, i32> {
    let lb = FG_.lock();
    let state = lb.get(&pid).ok_or(-3)?;
    state.get_action(signo).copied().ok_or(-22)
}


pub fn ope(pid: u32, how: u32, set: u64, old_set: &mut u64) -> Result<(), i32> {
    let lb = FG_.lock();
    let state = lb.get(&pid).ok_or(-3)?;
    
    *old_set = state.blocked.load(Ordering::Relaxed);
    
    match how {
        0 => state.block(set),        
        1 => state.unblock(set),      
        2 => state.set_blocked(set),  
        _ => return Err(-22),
    }
    
    Ok(())
}


pub fn kjs(pid: u32) -> Option<u32> {
    let mut lb = FG_.lock();
    let state = lb.get_mut(&pid)?;
    
    if let Some(signo) = state.get_deliverable() {
        let action = &state.actions[signo as usize];
        
        match action.sa_handler {
            BIA_ => {
                
                state.clear_pending(signo);
                None
            }
            YI_ => {
                
                state.clear_pending(signo);
                idh(pid, signo);
                Some(signo)
            }
            _ => {
                
                state.clear_pending(signo);
                Some(signo)
            }
        }
    } else {
        None
    }
}


fn idh(pid: u32, signo: u32) {
    match signo {
        
        sig::Adx | sig::Adz | sig::Iv | sig::Aea |
        sig::Adt | sig::Aed | sig::Aef | sig::Aeg => {
            crate::process::crk(pid);
        }
        
        
        sig::Aeb | sig::Ady | sig::Ads | sig::Adw |
        sig::Aec | sig::Adu | sig::Api => {
            mch(pid, signo);
            crate::process::crk(pid);
        }
        
        
        sig::Hb | sig::Aee | sig::Apj | sig::Apk => {
            crate::process::stop(pid);
        }
        
        
        sig::Adv => {
            crate::process::resume(pid);
        }
        
        
        sig::Uu | sig::Apl | sig::Apm => {
            
        }
        
        _ => {
            
            crate::process::crk(pid);
        }
    }
}


pub fn osq(signo: u32) -> &'static str {
    match signo {
        sig::Adx => "SIGHUP",
        sig::Adz => "SIGINT",
        sig::Aeb => "SIGQUIT",
        sig::Ady => "SIGILL",
        sig::Uw => "SIGTRAP",
        sig::Ads => "SIGABRT",
        sig::Adu => "SIGBUS",
        sig::Adw => "SIGFPE",
        sig::Iv => "SIGKILL",
        sig::Aef => "SIGUSR1",
        sig::Aec => "SIGSEGV",
        sig::Aeg => "SIGUSR2",
        sig::Aea => "SIGPIPE",
        sig::Adt => "SIGALRM",
        sig::Aed => "SIGTERM",
        sig::Uu => "SIGCHLD",
        sig::Adv => "SIGCONT",
        sig::Hb => "SIGSTOP",
        sig::Aee => "SIGTSTP",
        _ => "UNKNOWN",
    }
}








#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct Vb {
    
    pub pretcode: u64,
    
    pub signo: u32,
    
    pub _pad: u32,
    
    pub saved_rip: u64,
    
    pub saved_rsp: u64,
    
    pub saved_rflags: u64,
    
    pub saved_rax: u64,
    
    pub saved_mask: u64,
}










pub fn pnt(
    user_rip: &mut u64,
    user_rsp: &mut u64,
    user_rflags: &mut u64,
    syscall_rax: u64,
) -> Option<u64> {
    let pid = crate::process::pe();
    
    let mut lb = FG_.lock();
    let state = lb.get_mut(&pid)?;
    
    let signo = state.get_deliverable()?;
    let action = state.actions[signo as usize];
    
    match action.sa_handler {
        BIA_ => {
            state.clear_pending(signo);
            None
        }
        YI_ => {
            state.clear_pending(signo);
            drop(lb);
            idh(pid, signo);
            None
        }
        handler => {
            
            state.clear_pending(signo);
            
            
            let dvu = state.blocked.load(Ordering::Relaxed);
            let mut iqb = dvu | action.sa_mask;
            
            if action.sa_flags & sa_flags::CTT_ == 0 {
                iqb |= 1 << signo;
            }
            state.blocked.store(iqb, Ordering::SeqCst);
            
            
            if action.sa_flags & sa_flags::CTU_ != 0 {
                state.actions[signo as usize].sa_handler = YI_;
            }
            
            drop(lb);
            
            
            let frame_size = core::mem::size_of::<Vb>() as u64;
            
            
            let gjh = ((*user_rsp - frame_size) & !0xF) - 8;
            
            
            if !crate::memory::ux(gjh) {
                
                crate::process::crk(pid);
                return None;
            }
            
            
            let frame = unsafe { &mut *(gjh as *mut Vb) };
            frame.pretcode = if action.sa_flags & sa_flags::CTV_ != 0 {
                action.sa_restorer
            } else {
                
                
                
                crate::log_debug!("[SIGNAL] No sa_restorer for signal {} — terminating PID {}", signo, pid);
                crate::process::crk(pid);
                return None;
            };
            frame.signo = signo;
            frame._pad = 0;
            frame.saved_rip = *user_rip;
            frame.saved_rsp = *user_rsp;
            frame.saved_rflags = *user_rflags;
            frame.saved_rax = syscall_rax;
            frame.saved_mask = dvu;
            
            
            *user_rip = handler;
            *user_rsp = gjh;
            
            
            crate::log_debug!("[SIGNAL] Delivering signal {} to PID {} at handler {:#x}", signo, pid, handler);
            
            Some(signo as u64)
        }
    }
}





pub fn oss(
    user_rip: &mut u64,
    user_rsp: &mut u64,
    user_rflags: &mut u64,
) -> i64 {
    let pid = crate::process::pe();
    
    
    
    
    
    
    
    
    
    
    
    
    
    
    
    
    
    
    
    
    
    let hzv = *user_rsp - 8;
    
    if !crate::memory::ux(hzv) {
        return -22; 
    }
    
    let frame = unsafe { &*(hzv as *const Vb) };
    
    
    *user_rip = frame.saved_rip;
    *user_rsp = frame.saved_rsp;
    *user_rflags = frame.saved_rflags;
    
    
    if let Some(state) = FG_.lock().get(&pid) {
        state.blocked.store(frame.saved_mask, Ordering::SeqCst);
    }
    
    crate::log_debug!("[SIGNAL] sigreturn: restoring RIP={:#x} RSP={:#x} RAX={}", 
        frame.saved_rip, frame.saved_rsp, frame.saved_rax as i64);
    
    frame.saved_rax as i64
}


fn mch(pid: u32, signo: u32) {
    use alloc::format;
    use alloc::vec;

    let ab = match crate::process::cyj(pid) {
        Some(c) => c,
        None => return,
    };

    
    
    let mut core = vec![0u8; 0];

    
    core.extend_from_slice(&[0x7f, b'E', b'L', b'F']); 
    core.push(2); 
    core.push(1); 
    core.push(1); 
    core.push(0); 
    core.extend_from_slice(&[0u8; 8]); 
    core.extend_from_slice(&4u16.to_le_bytes()); 
    core.extend_from_slice(&0x3Eu16.to_le_bytes()); 
    core.extend_from_slice(&1u32.to_le_bytes()); 
    core.extend_from_slice(&0u64.to_le_bytes()); 
    core.extend_from_slice(&64u64.to_le_bytes()); 
    core.extend_from_slice(&0u64.to_le_bytes()); 
    core.extend_from_slice(&0u32.to_le_bytes()); 
    core.extend_from_slice(&64u16.to_le_bytes()); 
    core.extend_from_slice(&56u16.to_le_bytes()); 
    core.extend_from_slice(&1u16.to_le_bytes()); 
    core.extend_from_slice(&0u16.to_le_bytes()); 
    core.extend_from_slice(&0u16.to_le_bytes()); 
    core.extend_from_slice(&0u16.to_le_bytes()); 

    
    let agu = b"CORE\0\0\0\0"; 
    let mut dbv = vec![0u8; 0];
    
    dbv.extend_from_slice(&signo.to_le_bytes()); 
    dbv.extend_from_slice(&pid.to_le_bytes());   
    
    for &reg in &[
        ab.r15, ab.r14, ab.r13, ab.r12, ab.rbp, ab.rbx,
        ab.r11, ab.r10, ab.r9, ab.r8, ab.rax, ab.rcx,
        ab.rdx, ab.rsi, ab.rdi, ab.rax, 
        ab.rip, ab.cs, ab.rflags, ab.rsp, ab.ss,
    ] {
        dbv.extend_from_slice(&reg.to_le_bytes());
    }

    let nla = 5u32; 
    let nkx = dbv.len() as u32;
    let nlo = 1u32;

    let note_offset = 64 + 56; 
    let iqx = 12 + 8 + dbv.len(); 

    
    core.extend_from_slice(&4u32.to_le_bytes()); 
    core.extend_from_slice(&0u32.to_le_bytes()); 
    core.extend_from_slice(&(note_offset as u64).to_le_bytes()); 
    core.extend_from_slice(&0u64.to_le_bytes()); 
    core.extend_from_slice(&0u64.to_le_bytes()); 
    core.extend_from_slice(&(iqx as u64).to_le_bytes()); 
    core.extend_from_slice(&(iqx as u64).to_le_bytes()); 
    core.extend_from_slice(&4u64.to_le_bytes()); 

    
    core.extend_from_slice(&nla.to_le_bytes());
    core.extend_from_slice(&nkx.to_le_bytes());
    core.extend_from_slice(&nlo.to_le_bytes());
    core.extend_from_slice(agu);
    core.extend_from_slice(&dbv);

    
    let path = format!("/tmp/core.{}", pid);
    let _ = crate::vfs::write_file(&path, &core);

    crate::serial_println!("[COREDUMP] PID {} signal {} -> {} ({} bytes)",
        pid, osq(signo), path, core.len());
}
