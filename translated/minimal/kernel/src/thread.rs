







use alloc::boxed::Box;
use alloc::string::String;
use alloc::vec::Vec;
use alloc::collections::BTreeMap;
use core::sync::atomic::{AtomicU64, AtomicU32, Ordering};
use spin::{Mutex, RwLock};


pub type Cs = u64;


pub const QB_: Cs = 0;


static VT_: AtomicU64 = AtomicU64::new(1);


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThreadState {
    
    At,
    
    Ai,
    
    Hj,
    
    Cnb,
    
    Ez,
}


#[derive(Debug, Clone, Copy)]
pub struct ThreadFlags(pub u32);

impl ThreadFlags {
    pub const Cq: u32 = 0;
    pub const Ps: u32 = 1 << 0;      
    pub const Ava: u32 = 1 << 1;        
    pub const Ctz: u32 = 1 << 2;    
}


#[cfg(target_arch = "x86_64")]
#[derive(Debug, Clone)]
#[repr(C, align(16))]
pub struct ThreadContext {
    
    pub rbx: u64,       
    pub rbp: u64,       
    pub r12: u64,       
    pub r13: u64,       
    pub r14: u64,       
    pub r15: u64,       
    
    
    pub rsp: u64,       
    
    
    pub pc: u64,       
    
    
    pub dxg: u64,  
    pub fxy: u64,  
    
    
    pub aap: u64,        
    pub rv: u64,        
    
    
    pub rflags: u64,    
    
    
    pub qbx: u64,  
    
    
    pub szt: [u8; 512],  
}

#[cfg(target_arch = "x86_64")]
impl Default for ThreadContext {
    fn default() -> Self {
        Self {
            rbx: 0, rbp: 0, r12: 0, r13: 0, r14: 0, r15: 0,
            rsp: 0, pc: 0, dxg: 0, fxy: 0,
            aap: 0, rv: 0, rflags: 0, qbx: 0,
            szt: [0; 512],
        }
    }
}


#[cfg(target_arch = "aarch64")]
#[derive(Debug, Clone, Default)]
#[repr(C)]
pub struct ThreadContext {
    
    pub qah: u64,
    pub qai: u64,
    pub xwc: u64,
    pub zxd: u64,
    pub zxe: u64,
    pub zxf: u64,
    pub zxg: u64,
    pub zxh: u64,
    pub zxi: u64,
    pub zxj: u64,
    pub ghm: u64,   
    pub aad: u64,   
    pub sp: u64,
}


#[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
#[derive(Debug, Clone, Default)]
#[repr(C)]
pub struct ThreadContext {
    ybw: u64,
}


pub struct Thread {
    
    pub ni: Cs,
    
    
    pub ce: u32,
    
    
    pub j: String,
    
    
    pub g: ThreadState,
    
    
    pub flags: ThreadFlags,
    
    
    pub context: ThreadContext,
    
    
    bhg: Option<Box<[u8; NS_]>>,
    
    
    pub etp: u64,
    
    
    pub jvc: u64,
    
    
    pub mi: u64,
    
    
    pub isv: u64,
    
    
    pub nz: i32,
    
    
    pub cdu: u64,
    
    
    pub eto: Option<Cs>,
}


const NS_: usize = 256 * 1024; 

impl Thread {
    
    pub fn opw(ce: u32, j: &str, bt: u64, ji: u64) -> Self {
        let ni = VT_.fetch_add(1, Ordering::SeqCst);
        
        
        let mut bhg = Box::new([0u8; NS_]);
        let alt = bhg.fq() as u64 + NS_ as u64;
        
        
        let mut context = ThreadContext::default();
        
        #[cfg(target_arch = "x86_64")]
        {
            
            context.rsp = alt - 8;
            context.pc = bt;
            context.rflags = 0x202; 
            context.aap = crate::gdt::NQ_ as u64;
            context.rv = crate::gdt::NR_ as u64;
            
            
            
            unsafe {
                let vyi = (alt - 8) as *mut u64;
                *vyi = idj as u64;
            }
            
            
            context.r12 = bt;
            context.r13 = ji;
            context.pc = idj as u64;
        }
        
        #[cfg(target_arch = "aarch64")]
        {
            context.sp = alt;
            context.aad = idj as u64;
            context.qah = bt;
            context.qai = ji;
        }
        
        Self {
            ni,
            ce,
            j: String::from(j),
            g: ThreadState::At,
            flags: ThreadFlags(ThreadFlags::Ps),
            context,
            bhg: Some(bhg),
            etp: alt,
            jvc: 0,
            mi: bt,
            isv: ji,
            nz: 0,
            cdu: 0,
            eto: None,
        }
    }
    
    
    pub fn zdl(ce: u32, j: &str, bt: u64, ais: u64, oge: bool) -> Self {
        let mut thread = if oge {
            Self::opw(ce, j, bt, 0)
        } else {
            Self::oqg(ce, j, bt, ais, 0)
        };
        thread.flags.0 |= ThreadFlags::Ava;
        thread
    }
    
    
    pub fn oqg(ce: u32, j: &str, bt: u64, ais: u64, ji: u64) -> Self {
        let ni = VT_.fetch_add(1, Ordering::SeqCst);
        
        
        let bhg = Box::new([0u8; NS_]);
        let etp = bhg.fq() as u64 + NS_ as u64;
        
        
        let mut context = ThreadContext::default();
        
        #[cfg(target_arch = "x86_64")]
        {
            
            context.dxg = ais;
            context.fxy = bt;
            context.rflags = 0x202; 
            context.aap = crate::gdt::AJK_ as u64;
            context.rv = crate::gdt::AJL_ as u64;
            
            
            context.rsp = etp;
            context.pc = moo as u64;
            
            
            context.r12 = bt;
            context.r13 = ais;
            context.r14 = ji;
        }
        
        #[cfg(target_arch = "aarch64")]
        {
            context.sp = etp;
            context.aad = moo as u64;
            context.qah = bt;
            context.qai = ais;
            context.xwc = ji;
        }
        
        Self {
            ni,
            ce,
            j: String::from(j),
            g: ThreadState::At,
            flags: ThreadFlags(ThreadFlags::Cq),
            context,
            bhg: Some(bhg),
            etp,
            jvc: ais,
            mi: bt,
            isv: ji,
            nz: 0,
            cdu: 0,
            eto: None,
        }
    }
    
    
    pub fn oge(&self) -> bool {
        self.flags.0 & ThreadFlags::Ps != 0
    }
    
    
    pub fn yzp(&self) -> bool {
        self.flags.0 & ThreadFlags::Ava != 0
    }
}



#[cfg(target_arch = "x86_64")]
#[unsafe(evb)]
extern "C" fn idj() {
    core::arch::evc!(
        
        
        "sti",
        
        
        "mov rdi, r13",      
        "call r12",          
        
        
        "mov rdi, rax",      
        "call {exit}",
        
        
        "ud2",
        
        cxn = aaw mkq,
    );
}

#[cfg(target_arch = "aarch64")]
#[unsafe(evb)]
extern "C" fn idj() {
    core::arch::evc!(
        "msr daifclr, #0xf",   
        "mov x0, x20",         
        "blr x19",             
        "bl {exit}",           
        "brk #0",              
        cxn = aaw mkq,
    );
}

#[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
extern "C" fn idj() {
    
}




#[cfg(target_arch = "x86_64")]
#[unsafe(evb)]
extern "C" fn moo() {
    core::arch::evc!(
        
        "mov rdi, r12",          
        "mov rsi, r13",          
        "mov rdx, r14",          
        "xor ecx, ecx",         
        "jmp {jump}",
        eeb = aaw crate::userland::ohk,
    );
}

#[cfg(not(target_arch = "x86_64"))]
extern "C" fn moo() {
    
}


#[no_mangle]
extern "C" fn mkq(nz: i32) {
    let ni = bqd();
    
    if let Some(mut thread) = Do.write().ds(&ni) {
        thread.g = ThreadState::Ez;
        thread.nz = nz;
        
        
        if let Some(uam) = thread.eto {
            if let Some(eto) = Do.write().ds(&uam) {
                if eto.g == ThreadState::Hj {
                    eto.g = ThreadState::At;
                }
            }
        }
    }
    
    crate::log_debug!("[THREAD] Thread {} exited with code {}", ni, nz);
    
    
    cix();
}





lazy_static::lazy_static! {
    
    static ref Do: RwLock<BTreeMap<Cs, Thread>> = RwLock::new(BTreeMap::new());
}


const KO_: usize = 64;
static SL_: [AtomicU64; KO_] = {
    const Dm: AtomicU64 = AtomicU64::new(QB_);
    [Dm; KO_]
};


const AWW_: Cs = 0x8000_0000_0000_0000;


fn ode(qq: usize) -> Cs {
    if qq == 0 { 0 } else { AWW_ + qq as u64 }
}


fn jbm(ni: Cs) -> bool {
    ni == 0 || ni >= AWW_
}


#[inline]
fn jns() -> usize {
    (crate::cpu::smp::ead() as usize).v(KO_ - 1)
}


pub fn bqd() -> Cs {
    SL_[jns()].load(Ordering::Relaxed)
}


pub fn wip(ni: Cs) {
    SL_[jns()].store(ni, Ordering::SeqCst);
}


pub fn jqu(j: &str, bt: fn(u64) -> i32, ji: u64) -> Cs {
    let ce = crate::process::aei();
    let thread = Thread::opw(ce, j, bt as *const () as u64, ji);
    let ni = thread.ni;
    
    Do.write().insert(ni, thread);
    fhs(ni);
    
    crate::log_debug!("[THREAD] Spawned kernel thread {} '{}'", ni, j);
    ni
}


pub fn pme(ce: u32, j: &str, bt: u64, ais: u64, ji: u64) -> Cs {
    let thread = Thread::oqg(ce, j, bt, ais, ji);
    let ni = thread.ni;
    
    Do.write().insert(ni, thread);
    fhs(ni);
    
    crate::log_debug!("[THREAD] Spawned user thread {} '{}'", ni, j);
    ni
}


pub fn cix() {
    dvk();
}


pub fn cxn(aj: i32) {
    mkq(aj);
}


pub fn wake(ni: Cs) {
    let mut axc = Do.write();
    if let Some(thread) = axc.ds(&ni) {
        if thread.g == ThreadState::Hj {
            thread.g = ThreadState::At;
            drop(axc);
            fhs(ni);
        }
    }
}


pub fn block(ni: Cs) {
    let mut axc = Do.write();
    if let Some(thread) = axc.ds(&ni) {
        thread.g = ThreadState::Hj;
    }
}





pub fn mzq() {
    let ni = bqd();
    if jbm(ni) {
        return; 
    }
    {
        let mut axc = Do.write();
        if let Some(thread) = axc.ds(&ni) {
            thread.g = ThreadState::Hj;
        }
    }
    
    dvk();
}




pub fn eyp(eao: u64) {
    let ni = bqd();
    if jbm(ni) {
        return;
    }

    
    crate::time::vuf(ni, eao);

    
    mzq();
}


pub fn wpl(shm: u64) {
    let ean = crate::time::evk().akq(shm);
    eyp(ean);
}





use alloc::collections::VecDeque;






struct PerCpuQueue {
    queue: Mutex<VecDeque<Cs>>,
}

impl PerCpuQueue {
    const fn new() -> Self {
        Self { queue: Mutex::new(VecDeque::new()) }
    }
    
    fn push(&self, ni: Cs) {
        self.queue.lock().agt(ni);
    }
    
    fn pop(&self) -> Option<Cs> {
        self.queue.lock().awp()
    }
    
    
    fn por(&self) -> Option<Cs> {
        self.queue.lock().owo()
    }
    
    fn len(&self) -> usize {
        self.queue.lock().len()
    }
    
    
    fn juf(&self, ni: Cs) -> bool {
        if let Some(mut fm) = self.queue.try_lock() {
            fm.agt(ni);
            true
        } else {
            false
        }
    }
    
    fn xms(&self) -> Option<Cs> {
        self.queue.try_lock()?.awp()
    }
    
    fn xmw(&self) -> Option<Cs> {
        self.queue.try_lock()?.owo()
    }
    
    fn xmp(&self) -> usize {
        self.queue.try_lock().efd(0, |fm| fm.len())
    }
}


static GY_: [PerCpuQueue; KO_] = {
    const Dm: PerCpuQueue = PerCpuQueue::new();
    [Dm; KO_]
};


static CHO_: AtomicU64 = AtomicU64::new(0);


lazy_static::lazy_static! {
    static ref CNL_: Mutex<VecDeque<Cs>> = Mutex::new(VecDeque::new());
}


fn fhs(ni: Cs) {
    let bcc = crate::cpu::smp::boc().am(1) as usize;
    
    
    let cih = (CHO_.fetch_add(1, Ordering::Relaxed) % bcc as u64) as usize;
    GY_[cih].push(ni);
    
    
    let rrv = jns();
    if cih != rrv && cih > 0 {
        crate::cpu::smp::phx(cih as u32);
    }
}


fn xmx(uqz: usize) -> Option<Cs> {
    let bcc = crate::cpu::smp::boc().am(1) as usize;
    
    
    let mut kcs = usize::O;
    let mut cjg = 0;
    
    for cpu in 0..bcc {
        if cpu == uqz { continue; }
        let len = GY_[cpu].xmp();
        if len > cjg {
            cjg = len;
            kcs = cpu;
        }
    }
    
    if kcs < KO_ && cjg > 1 {
        return GY_[kcs].xmw();
    }
    
    
    CNL_.try_lock()?.awp()
}


pub fn init() {
    crate::serial_println!("[THREAD] Creating idle thread...");
    
    crate::arch::cvh(|| {
        
        let trr = Thread {
            ni: 0,
            ce: 0,
            j: String::from("idle"),
            g: ThreadState::Ai,
            flags: ThreadFlags(ThreadFlags::Ps | ThreadFlags::Ava),
            context: ThreadContext::default(),
            bhg: None,
            etp: 0,
            jvc: 0,
            mi: 0,
            isv: 0,
            nz: 0,
            cdu: 0,
            eto: None,
        };
        
        Do.write().insert(0, trr);
        SL_[0].store(0, Ordering::SeqCst);
    });
    crate::serial_println!("[THREAD] Thread subsystem initialized");
}



pub fn ttb(qq: u32) {
    let w = qq as usize;
    let izj = ode(w);
    
    let fkz = Thread {
        ni: izj,
        ce: 0,
        j: String::from("idle-ap"),
        g: ThreadState::Ai,
        flags: ThreadFlags(ThreadFlags::Ps),
        context: ThreadContext::default(),
        bhg: None,
        etp: 0,
        jvc: 0,
        mi: 0,
        isv: 0,
        nz: 0,
        cdu: 0,
        eto: None,
    };
    
    Do.write().insert(izj, fkz);
    SL_[w].store(izj, Ordering::SeqCst);
    
    crate::serial_println!("[THREAD] AP {} idle thread created (TID={:#x})", qq, izj);
}


pub fn hto() {
    let ni = bqd();
    
    
    if let Some(mut axc) = Do.ifb() {
        if let Some(thread) = axc.ds(&ni) {
            thread.cdu += 1;
        }
    }
    
    
    static CXX_: AtomicU64 = AtomicU64::new(0);
    let qb = CXX_.fetch_add(1, Ordering::Relaxed);
    
    if qb % 10 == 0 {
        dvk();
    }
}


pub fn dvk() {
    let qq = jns();
    let cv = bqd();
    let fkz = ode(qq);
    
    
    
    if cv != QB_ && !jbm(cv) {
        let wnb = if let Some(axc) = Do.mnf() {
            axc.get(&cv).efd(false, |ab| ab.g == ThreadState::Ai)
        } else {
            return; 
        };
        if wnb {
            
            if !GY_[qq].juf(cv) {
                return; 
            }
            if let Some(mut axc) = Do.ifb() {
                if let Some(ab) = axc.ds(&cv) {
                    ab.g = ThreadState::At;
                }
            }
        }
    }
    
    
    let uuq = loop {
        
        if let Some(ni) = GY_[qq].xms() {
            let mbg = if let Some(axc) = Do.mnf() {
                axc.get(&ni).efd(false, |ab| ab.g == ThreadState::At || ab.g == ThreadState::Ai)
            } else {
                
                let _ = GY_[qq].juf(ni);
                return;
            };
            if mbg {
                break Some(ni);
            }
            continue; 
        }
        
        
        if let Some(ni) = xmx(qq) {
            let mbg = if let Some(axc) = Do.mnf() {
                axc.get(&ni).efd(false, |ab| ab.g == ThreadState::At || ab.g == ThreadState::Ai)
            } else {
                let _ = GY_[qq].juf(ni);
                return;
            };
            if mbg {
                break Some(ni);
            }
            continue;
        }
        
        
        break None;
    };
    
    match uuq {
        Some(next) if next != cv => {
            
            if let Some(mut axc) = Do.ifb() {
                if let Some(thread) = axc.ds(&next) {
                    thread.g = ThreadState::Ai;
                }
            } else {
                
                let _ = GY_[qq].juf(next);
                return;
            }
            
            
            nfs(cv, next);
        }
        None if !jbm(cv) && cv != QB_ => {
            
            nfs(cv, fkz);
        }
        _ => {
            
        }
    }
}


fn nfs(from: Cs, wh: Cs) {
    if from == wh {
        return;
    }
    
    
    let nwd: *mut ThreadContext;
    let ptt: *const ThreadContext;
    let jtj: u64;
    
    {
        let mut axc = match Do.ifb() {
            Some(ab) => ab,
            None => return, 
        };
        
        let syj = match axc.ds(&from) {
            Some(ab) => ab as *mut Thread,
            None => return,
        };
        
        let ptz = match axc.get(&wh) {
            Some(ab) => ab,
            None => return,
        };
        
        nwd = unsafe { &mut (*syj).context as *mut ThreadContext };
        ptt = &ptz.context as *const ThreadContext;
        jtj = ptz.etp;
    }
    
    
    #[cfg(target_arch = "x86_64")]
    if jtj != 0 {
        crate::gdt::pjb(jtj);
        
        
        
        unsafe {
            crate::userland::NT_ = jtj;
        }
    }
    
    
    wip(wh);
    
    
    unsafe {
        mii(nwd, ptt);
    }
}


#[cfg(target_arch = "x86_64")]
#[unsafe(evb)]
extern "C" fn mii(from: *mut ThreadContext, wh: *const ThreadContext) {
    core::arch::evc!(
        
        
        
        
        "fxsave [rdi + 0x70]",
        
        
        "mov [rdi + 0x00], rbx",
        "mov [rdi + 0x08], rbp",
        "mov [rdi + 0x10], r12",
        "mov [rdi + 0x18], r13",
        "mov [rdi + 0x20], r14",
        "mov [rdi + 0x28], r15",
        
        
        "mov [rdi + 0x30], rsp",
        
        
        "lea rax, [rip + 2f]",
        "mov [rdi + 0x38], rax",
        
        
        
        
        
        "fxrstor [rsi + 0x70]",
        
        
        "mov rbx, [rsi + 0x00]",
        "mov rbp, [rsi + 0x08]",
        "mov r12, [rsi + 0x10]",
        "mov r13, [rsi + 0x18]",
        "mov r14, [rsi + 0x20]",
        "mov r15, [rsi + 0x28]",
        
        
        "mov rsp, [rsi + 0x30]",
        
        
        "jmp [rsi + 0x38]",
        
        
        "2:",
        "ret",
    );
}


#[cfg(target_arch = "aarch64")]
#[unsafe(evb)]
extern "C" fn mii(msh: *mut ThreadContext, qdu: *const ThreadContext) {
    core::arch::evc!(
        
        "stp x19, x20, [x0, #0]",
        "stp x21, x22, [x0, #16]",
        "stp x23, x24, [x0, #32]",
        "stp x25, x26, [x0, #48]",
        "stp x27, x28, [x0, #64]",
        "stp x29, x30, [x0, #80]",
        "mov x9, sp",
        "str x9, [x0, #96]",
        
        "ldp x19, x20, [x1, #0]",
        "ldp x21, x22, [x1, #16]",
        "ldp x23, x24, [x1, #32]",
        "ldp x25, x26, [x1, #48]",
        "ldp x27, x28, [x1, #64]",
        "ldp x29, x30, [x1, #80]",
        "ldr x9, [x1, #96]",
        "mov sp, x9",
        "ret",
    );
}

#[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
extern "C" fn mii(msh: *mut ThreadContext, qdu: *const ThreadContext) {
    
}






pub struct Buh {
    caq: AtomicU32,
    awj: AtomicU64,
    cny: Mutex<VecDeque<Cs>>,
}

impl Buh {
    pub const fn new() -> Self {
        Self {
            caq: AtomicU32::new(0),
            awj: AtomicU64::new(QB_),
            cny: Mutex::new(VecDeque::new()),
        }
    }
    
    pub fn lock(&self) {
        let ni = bqd();
        
        loop {
            
            if self.caq.compare_exchange(0, 1, Ordering::Acquire, Ordering::Relaxed).is_ok() {
                self.awj.store(ni, Ordering::Relaxed);
                return;
            }
            
            
            {
                self.cny.lock().agt(ni);
                if let Some(thread) = Do.write().ds(&ni) {
                    thread.g = ThreadState::Hj;
                }
            }
            
            
            cix();
        }
    }
    
    pub fn xog(&self) {
        self.awj.store(QB_, Ordering::Relaxed);
        self.caq.store(0, Ordering::Release);
        
        
        if let Some(cnx) = self.cny.lock().awp() {
            if let Some(thread) = Do.write().ds(&cnx) {
                thread.g = ThreadState::At;
            }
            fhs(cnx);
        }
    }
    
    pub fn try_lock(&self) -> bool {
        let ni = bqd();
        
        if self.caq.compare_exchange(0, 1, Ordering::Acquire, Ordering::Relaxed).is_ok() {
            self.awj.store(ni, Ordering::Relaxed);
            true
        } else {
            false
        }
    }
}


pub struct Aml {
    az: AtomicU32,
    cny: Mutex<VecDeque<Cs>>,
}

impl Aml {
    pub const fn new(cfo: u32) -> Self {
        Self {
            az: AtomicU32::new(cfo),
            cny: Mutex::new(VecDeque::new()),
        }
    }
    
    pub fn ccm(&self) {
        loop {
            let az = self.az.load(Ordering::Relaxed);
            
            if az > 0 {
                if self.az.compare_exchange(az, az - 1, Ordering::Acquire, Ordering::Relaxed).is_ok() {
                    return;
                }
            } else {
                
                let ni = bqd();
                {
                    self.cny.lock().agt(ni);
                    if let Some(thread) = Do.write().ds(&ni) {
                        thread.g = ThreadState::Hj;
                    }
                }
                cix();
            }
        }
    }
    
    pub fn cug(&self) {
        self.az.fetch_add(1, Ordering::Release);
        
        
        if let Some(cnx) = self.cny.lock().awp() {
            if let Some(thread) = Do.write().ds(&cnx) {
                thread.g = ThreadState::At;
            }
            fhs(cnx);
        }
    }
}


pub struct Bzy {
    cny: Mutex<VecDeque<Cs>>,
}

impl Bzy {
    pub const fn new() -> Self {
        Self {
            cny: Mutex::new(VecDeque::new()),
        }
    }
    
    
    pub fn ccm(&self, oot: &Buh) {
        let ni = bqd();
        
        
        self.cny.lock().agt(ni);
        
        
        if let Some(thread) = Do.write().ds(&ni) {
            thread.g = ThreadState::Hj;
        }
        
        
        oot.xog();
        
        
        cix();
        
        
        oot.lock();
    }
    
    
    pub fn cug(&self) {
        if let Some(cnx) = self.cny.lock().awp() {
            if let Some(thread) = Do.write().ds(&cnx) {
                thread.g = ThreadState::At;
            }
            fhs(cnx);
        }
    }
    
    
    pub fn nad(&self) {
        let mut cny = self.cny.lock();
        while let Some(cnx) = cny.awp() {
            if let Some(thread) = Do.write().ds(&cnx) {
                thread.g = ThreadState::At;
            }
            fhs(cnx);
        }
    }
}





pub fn ufx() -> alloc::vec::Vec<(u64, u32, ThreadState, alloc::string::String)> {
    let axc = Do.read();
    let mut result = alloc::vec::Vec::new();
    
    for (ni, thread) in axc.iter() {
        result.push((*ni, thread.ce, thread.g, thread.j.clone()));
    }
    
    
    result.bxf(|(ni, _, _, _)| *ni);
    result
}
