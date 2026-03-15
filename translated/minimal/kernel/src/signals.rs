




use alloc::collections::BTreeMap;
use alloc::vec::Vec;
use core::sync::atomic::{AtomicU64, Ordering};
use spin::Mutex;


pub mod sig {
    pub const Bro: u32 = 1;
    pub const Brq: u32 = 2;
    pub const Brs: u32 = 3;
    pub const Brp: u32 = 4;
    pub const Ayh: u32 = 5;
    pub const Brj: u32 = 6;
    pub const Brl: u32 = 7;
    pub const Brn: u32 = 8;
    pub const Ug: u32 = 9;  
    pub const Brw: u32 = 10;
    pub const Brt: u32 = 11;
    pub const Brx: u32 = 12;
    pub const Brr: u32 = 13;
    pub const Brk: u32 = 14;
    pub const Bru: u32 = 15;
    pub const Dhk: u32 = 16;
    pub const Ayf: u32 = 17;
    pub const Brm: u32 = 18;
    pub const Qq: u32 = 19; 
    pub const Brv: u32 = 20;
    pub const Clu: u32 = 21;
    pub const Clv: u32 = 22;
    pub const Clw: u32 = 23;
    pub const Dhm: u32 = 24;
    pub const Dhn: u32 = 25;
    pub const Dhl: u32 = 26;
    pub const Dhg: u32 = 27;
    pub const Clx: u32 = 28;
    pub const Dhd: u32 = 29;
    pub const Dhh: u32 = 30;
    pub const Clt: u32 = 31;
    
    
    pub const Dhj: u32 = 32;
    pub const Dhi: u32 = 64;
    
    pub const GT_: usize = 65;
}


pub mod sa_flags {
    pub const EDT_: u64 = 0x00000001;
    pub const EDU_: u64 = 0x00000002;
    pub const EDX_: u64 = 0x00000004;
    pub const EDV_: u64 = 0x08000000;
    pub const EDW_: u64 = 0x10000000;
    pub const CQE_: u64 = 0x40000000;
    pub const CQF_: u64 = 0x80000000;
    pub const CQG_: u64 = 0x04000000;
}


pub const XB_: u64 = 0;  
pub const BFW_: u64 = 1;  
pub const EGK_: u64 = u64::O;


#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct SigAction {
    
    pub jnk: u64,
    
    pub sa_flags: u64,
    
    pub pfb: u64,
    
    pub pfa: u64,
}

impl Default for SigAction {
    fn default() -> Self {
        Self {
            jnk: XB_,
            sa_flags: 0,
            pfb: 0,
            pfa: 0,
        }
    }
}


#[derive(Clone, Debug)]
pub struct Bou {
    pub qk: u32,
    pub gsa: u32,
    pub aea: u64,
}


pub struct SignalState {
    
    pub fch: [SigAction; sig::GT_],
    
    pub aln: AtomicU64,
    
    pub cdg: AtomicU64,
    
    pub egk: Vec<Bou>,
}

impl SignalState {
    pub fn new() -> Self {
        Self {
            fch: [SigAction::default(); sig::GT_],
            aln: AtomicU64::new(0),
            cdg: AtomicU64::new(0),
            egk: Vec::new(),
        }
    }
    
    
    pub fn mec(&mut self, qk: u32, hr: SigAction) -> Result<SigAction, i32> {
        if qk == 0 || qk as usize >= sig::GT_ {
            return Err(-22); 
        }
        
        
        if qk == sig::Ug || qk == sig::Qq {
            return Err(-22); 
        }
        
        let aft = self.fch[qk as usize];
        self.fch[qk as usize] = hr;
        Ok(aft)
    }
    
    
    pub fn kyh(&self, qk: u32) -> Option<&SigAction> {
        if qk as usize >= sig::GT_ {
            return None;
        }
        Some(&self.fch[qk as usize])
    }
    
    
    pub fn vkd(&mut self, qk: u32, gsa: u32) {
        if qk == 0 || qk as usize >= sig::GT_ {
            return;
        }
        
        
        self.aln.nth(1 << qk, Ordering::SeqCst);
        
        
        self.egk.push(Bou {
            qk,
            gsa,
            aea: crate::time::evk(),
        });
    }
    
    
    pub fn yzu(&self, qk: u32) -> bool {
        if qk as usize >= sig::GT_ {
            return false;
        }
        (self.aln.load(Ordering::Relaxed) & (1 << qk)) != 0
    }
    
    
    pub fn yzb(&self, qk: u32) -> bool {
        if qk as usize >= sig::GT_ {
            return false;
        }
        
        if qk == sig::Ug || qk == sig::Qq {
            return false;
        }
        (self.cdg.load(Ordering::Relaxed) & (1 << qk)) != 0
    }
    
    
    pub fn nxz(&self) -> Option<u32> {
        let aln = self.aln.load(Ordering::Relaxed);
        let cdg = self.cdg.load(Ordering::Relaxed);
        let nkh = aln & !cdg;
        
        if nkh == 0 {
            return None;
        }
        
        
        Some(nkh.pvv())
    }
    
    
    pub fn gcp(&mut self, qk: u32) {
        if qk as usize >= sig::GT_ {
            return;
        }
        self.aln.ntg(!(1 << qk), Ordering::SeqCst);
        self.egk.ajm(|e| e.qk != qk);
    }
    
    
    pub fn block(&self, hs: u64) {
        self.cdg.nth(hs, Ordering::SeqCst);
    }
    
    
    pub fn xod(&self, hs: u64) {
        self.cdg.ntg(!hs, Ordering::SeqCst);
    }
    
    
    pub fn wii(&self, hs: u64) {
        
        let hs = hs & !((1 << sig::Ug) | (1 << sig::Qq));
        self.cdg.store(hs, Ordering::SeqCst);
    }
}


static ER_: Mutex<BTreeMap<u32, SignalState>> = Mutex::new(BTreeMap::new());


pub fn lef(ce: u32) {
    ER_.lock().insert(ce, SignalState::new());
}


pub fn khu(ce: u32) {
    ER_.lock().remove(&ce);
}


pub fn dsm(ejo: u32, qk: u32, gsa: u32) -> Result<(), i32> {
    if qk == 0 {
        
        let aja = ER_.lock().bgm(&ejo);
        return if aja { Ok(()) } else { Err(-3) }; 
    }
    
    let mut yh = ER_.lock();
    let g = yh.ds(&ejo).ok_or(-3)?; 
    
    g.vkd(qk, gsa);
    
    
    if qk == sig::Ug || qk == sig::Qq {
        tjo(ejo, qk);
    }
    
    Ok(())
}


pub fn lhk(bai: u32, qk: u32) -> Result<(), i32> {
    let bsg = crate::process::aei();
    let ovo = crate::process::vht(bai);
    if ovo.is_empty() {
        return Err(-3); 
    }
    for ce in ovo {
        let _ = dsm(ce, qk, bsg);
    }
    Ok(())
}


fn tjo(ce: u32, qk: u32) {
    match qk {
        sig::Ug => {
            
            crate::process::fwl(ce);
        }
        sig::Qq => {
            
            crate::process::qg(ce);
        }
        _ => {}
    }
}


pub fn mec(ce: u32, qk: u32, hr: SigAction) -> Result<SigAction, i32> {
    let mut yh = ER_.lock();
    let g = yh.ds(&ce).ok_or(-3)?;
    g.mec(qk, hr)
}


pub fn kyh(ce: u32, qk: u32) -> Result<SigAction, i32> {
    let yh = ER_.lock();
    let g = yh.get(&ce).ok_or(-3)?;
    g.kyh(qk).hu().ok_or(-22)
}


pub fn wje(ce: u32, lco: u32, oj: u64, uxt: &mut u64) -> Result<(), i32> {
    let yh = ER_.lock();
    let g = yh.get(&ce).ok_or(-3)?;
    
    *uxt = g.cdg.load(Ordering::Relaxed);
    
    match lco {
        0 => g.block(oj),        
        1 => g.xod(oj),      
        2 => g.wii(oj),  
        _ => return Err(-22),
    }
    
    Ok(())
}


pub fn qzs(ce: u32) -> Option<u32> {
    let mut yh = ER_.lock();
    let g = yh.ds(&ce)?;
    
    if let Some(qk) = g.nxz() {
        let hr = &g.fch[qk as usize];
        
        match hr.jnk {
            BFW_ => {
                
                g.gcp(qk);
                None
            }
            XB_ => {
                
                g.gcp(qk);
                oad(ce, qk);
                Some(qk)
            }
            _ => {
                
                g.gcp(qk);
                Some(qk)
            }
        }
    } else {
        None
    }
}


fn oad(ce: u32, qk: u32) {
    match qk {
        
        sig::Bro | sig::Brq | sig::Ug | sig::Brr |
        sig::Brk | sig::Bru | sig::Brw | sig::Brx => {
            crate::process::fwl(ce);
        }
        
        
        sig::Brs | sig::Brp | sig::Brj | sig::Brn |
        sig::Brt | sig::Brl | sig::Clt => {
            tca(ce, qk);
            crate::process::fwl(ce);
        }
        
        
        sig::Qq | sig::Brv | sig::Clu | sig::Clv => {
            crate::process::qg(ce);
        }
        
        
        sig::Brm => {
            crate::process::anu(ce);
        }
        
        
        sig::Ayf | sig::Clw | sig::Clx => {
            
        }
        
        _ => {
            
            crate::process::fwl(ce);
        }
    }
}


pub fn woc(qk: u32) -> &'static str {
    match qk {
        sig::Bro => "SIGHUP",
        sig::Brq => "SIGINT",
        sig::Brs => "SIGQUIT",
        sig::Brp => "SIGILL",
        sig::Ayh => "SIGTRAP",
        sig::Brj => "SIGABRT",
        sig::Brl => "SIGBUS",
        sig::Brn => "SIGFPE",
        sig::Ug => "SIGKILL",
        sig::Brw => "SIGUSR1",
        sig::Brt => "SIGSEGV",
        sig::Brx => "SIGUSR2",
        sig::Brr => "SIGPIPE",
        sig::Brk => "SIGALRM",
        sig::Bru => "SIGTERM",
        sig::Ayf => "SIGCHLD",
        sig::Brm => "SIGCONT",
        sig::Qq => "SIGSTOP",
        sig::Brv => "SIGTSTP",
        _ => "UNKNOWN",
    }
}








#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct Ayt {
    
    pub vlb: u64,
    
    pub qk: u32,
    
    pub fzo: u32,
    
    pub mbw: u64,
    
    pub mbx: u64,
    
    pub pfv: u64,
    
    pub mbv: u64,
    
    pub pfr: u64,
}










pub fn xml(
    fxy: &mut u64,
    dxg: &mut u64,
    mon: &mut u64,
    wzo: u64,
) -> Option<u64> {
    let ce = crate::process::aei();
    
    let mut yh = ER_.lock();
    let g = yh.ds(&ce)?;
    
    let qk = g.nxz()?;
    let hr = g.fch[qk as usize];
    
    match hr.jnk {
        BFW_ => {
            g.gcp(qk);
            None
        }
        XB_ => {
            g.gcp(qk);
            drop(yh);
            oad(ce, qk);
            None
        }
        cfd => {
            
            g.gcp(qk);
            
            
            let htn = g.cdg.load(Ordering::Relaxed);
            let mut opx = htn | hr.pfa;
            
            if hr.sa_flags & sa_flags::CQE_ == 0 {
                opx |= 1 << qk;
            }
            g.cdg.store(opx, Ordering::SeqCst);
            
            
            if hr.sa_flags & sa_flags::CQF_ != 0 {
                g.fch[qk as usize].jnk = XB_;
            }
            
            drop(yh);
            
            
            let bzt = core::mem::size_of::<Ayt>() as u64;
            
            
            let loc = ((*dxg - bzt) & !0xF) - 8;
            
            
            if !crate::memory::aov(loc) {
                
                crate::process::fwl(ce);
                return None;
            }
            
            
            let frame = unsafe { &mut *(loc as *mut Ayt) };
            frame.vlb = if hr.sa_flags & sa_flags::CQG_ != 0 {
                hr.pfb
            } else {
                
                
                
                crate::log_debug!("[SIGNAL] No sa_restorer for signal {} — terminating PID {}", qk, ce);
                crate::process::fwl(ce);
                return None;
            };
            frame.qk = qk;
            frame.fzo = 0;
            frame.mbw = *fxy;
            frame.mbx = *dxg;
            frame.pfv = *mon;
            frame.mbv = wzo;
            frame.pfr = htn;
            
            
            *fxy = cfd;
            *dxg = loc;
            
            
            crate::log_debug!("[SIGNAL] Delivering signal {} to PID {} at handler {:#x}", qk, ce, cfd);
            
            Some(qk as u64)
        }
    }
}





pub fn wog(
    fxy: &mut u64,
    dxg: &mut u64,
    mon: &mut u64,
) -> i64 {
    let ce = crate::process::aei();
    
    
    
    
    
    
    
    
    
    
    
    
    
    
    
    
    
    
    
    
    
    let nvx = *dxg - 8;
    
    if !crate::memory::aov(nvx) {
        return -22; 
    }
    
    let frame = unsafe { &*(nvx as *const Ayt) };
    
    
    *fxy = frame.mbw;
    *dxg = frame.mbx;
    *mon = frame.pfv;
    
    
    if let Some(g) = ER_.lock().get(&ce) {
        g.cdg.store(frame.pfr, Ordering::SeqCst);
    }
    
    crate::log_debug!("[SIGNAL] sigreturn: restoring RIP={:#x} RSP={:#x} RAX={}", 
        frame.mbw, frame.mbx, frame.mbv as i64);
    
    frame.mbv as i64
}


fn tca(ce: u32, qk: u32) {
    use alloc::format;
    use alloc::vec;

    let be = match crate::process::ghz(ce) {
        Some(r) => r,
        None => return,
    };

    
    
    let mut core = vec![0u8; 0];

    
    core.bk(&[0x7f, b'E', b'L', b'F']); 
    core.push(2); 
    core.push(1); 
    core.push(1); 
    core.push(0); 
    core.bk(&[0u8; 8]); 
    core.bk(&4u16.ho()); 
    core.bk(&0x3Eu16.ho()); 
    core.bk(&1u32.ho()); 
    core.bk(&0u64.ho()); 
    core.bk(&64u64.ho()); 
    core.bk(&0u64.ho()); 
    core.bk(&0u32.ho()); 
    core.bk(&64u16.ho()); 
    core.bk(&56u16.ho()); 
    core.bk(&1u16.ho()); 
    core.bk(&0u16.ho()); 
    core.bk(&0u16.ho()); 
    core.bk(&0u16.ho()); 

    
    let bkp = b"CORE\0\0\0\0"; 
    let mut gny = vec![0u8; 0];
    
    gny.bk(&qk.ho()); 
    gny.bk(&ce.ho());   
    
    for &reg in &[
        be.r15, be.r14, be.r13, be.r12, be.rbp, be.rbx,
        be.r11, be.r10, be.r9, be.r8, be.rax, be.rcx,
        be.rdx, be.rsi, be.rdi, be.rax, 
        be.pc, be.aap, be.rflags, be.rsp, be.rv,
    ] {
        gny.bk(&reg.ho());
    }

    let uvi = 5u32; 
    let uvf = gny.len() as u32;
    let uvz = 1u32;

    let gnz = 64 + 56; 
    let ora = 12 + 8 + gny.len(); 

    
    core.bk(&4u32.ho()); 
    core.bk(&0u32.ho()); 
    core.bk(&(gnz as u64).ho()); 
    core.bk(&0u64.ho()); 
    core.bk(&0u64.ho()); 
    core.bk(&(ora as u64).ho()); 
    core.bk(&(ora as u64).ho()); 
    core.bk(&4u64.ho()); 

    
    core.bk(&uvi.ho());
    core.bk(&uvf.ho());
    core.bk(&uvz.ho());
    core.bk(bkp);
    core.bk(&gny);

    
    let path = format!("/tmp/core.{}", ce);
    let _ = crate::vfs::ns(&path, &core);

    crate::serial_println!("[COREDUMP] PID {} signal {} -> {} ({} bytes)",
        ce, woc(qk), path, core.len());
}
