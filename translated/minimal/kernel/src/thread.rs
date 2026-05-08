







use alloc::boxed::Box;
use alloc::string::String;
use alloc::vec::Vec;
use alloc::collections::BTreeMap;
use core::sync::atomic::{AtomicU64, AtomicU32, Ordering};
use spin::{Mutex, RwLock};


pub type Bd = u64;


pub const QY_: Bd = 0;


static XC_: AtomicU64 = AtomicU64::new(1);


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThreadState {
    
    Ready,
    
    Running,
    
    Blocked,
    
    Sleeping,
    
    Dead,
}


#[derive(Debug, Clone, Copy)]
pub struct ThreadFlags(pub u32);

impl ThreadFlags {
    pub const Bc: u32 = 0;
    pub const Go: u32 = 1 << 0;      
    pub const Tl: u32 = 1 << 1;        
    pub const Atv: u32 = 1 << 2;    
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
    
    
    pub rip: u64,       
    
    
    pub user_rsp: u64,  
    pub user_rip: u64,  
    
    
    pub cs: u64,        
    pub ss: u64,        
    
    
    pub rflags: u64,    
    
    
    pub _fpu_pad: u64,  
    
    
    pub fxsave_area: [u8; 512],  
}

#[cfg(target_arch = "x86_64")]
impl Default for ThreadContext {
    fn default() -> Self {
        Self {
            rbx: 0, rbp: 0, r12: 0, r13: 0, r14: 0, r15: 0,
            rsp: 0, rip: 0, user_rsp: 0, user_rip: 0,
            cs: 0, ss: 0, rflags: 0, _fpu_pad: 0,
            fxsave_area: [0; 512],
        }
    }
}


#[cfg(target_arch = "aarch64")]
#[derive(Debug, Clone, Default)]
#[repr(C)]
pub struct ThreadContext {
    
    pub x19: u64,
    pub x20: u64,
    pub x21: u64,
    pub x22: u64,
    pub x23: u64,
    pub x24: u64,
    pub x25: u64,
    pub x26: u64,
    pub x27: u64,
    pub x28: u64,
    pub fp: u64,   
    pub lr: u64,   
    pub sp: u64,
}


#[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
#[derive(Debug, Clone, Default)]
#[repr(C)]
pub struct ThreadContext {
    _placeholder: u64,
}


pub struct Thread {
    
    pub tid: Bd,
    
    
    pub pid: u32,
    
    
    pub name: String,
    
    
    pub state: ThreadState,
    
    
    pub flags: ThreadFlags,
    
    
    pub context: ThreadContext,
    
    
    kernel_stack: Option<Box<[u8; OR_]>>,
    
    
    pub kernel_stack_top: u64,
    
    
    pub user_stack_top: u64,
    
    
    pub entry_point: u64,
    
    
    pub entry_arg: u64,
    
    
    pub exit_code: i32,
    
    
    pub cpu_time: u64,
    
    
    pub joiner: Option<Bd>,
}


const OR_: usize = 256 * 1024; 

impl Thread {
    
    pub fn iqa(pid: u32, name: &str, entry: u64, db: u64) -> Self {
        let tid = XC_.fetch_add(1, Ordering::SeqCst);
        
        
        let mut kernel_stack = Box::new([0u8; OR_]);
        let te = kernel_stack.as_ptr() as u64 + OR_ as u64;
        
        
        let mut context = ThreadContext::default();
        
        #[cfg(target_arch = "x86_64")]
        {
            
            context.rsp = te - 8;
            context.rip = entry;
            context.rflags = 0x202; 
            context.cs = crate::gdt::KERNEL_CODE_SELECTOR as u64;
            context.ss = crate::gdt::KERNEL_DATA_SELECTOR as u64;
            
            
            
            unsafe {
                let ogq = (te - 8) as *mut u64;
                *ogq = ebo as u64;
            }
            
            
            context.r12 = entry;
            context.r13 = db;
            context.rip = ebo as u64;
        }
        
        #[cfg(target_arch = "aarch64")]
        {
            context.sp = te;
            context.lr = ebo as u64;
            context.x19 = entry;
            context.x20 = db;
        }
        
        Self {
            tid,
            pid,
            name: String::from(name),
            state: ThreadState::Ready,
            flags: ThreadFlags(ThreadFlags::Go),
            context,
            kernel_stack: Some(kernel_stack),
            kernel_stack_top: te,
            user_stack_top: 0,
            entry_point: entry,
            entry_arg: db,
            exit_code: 0,
            cpu_time: 0,
            joiner: None,
        }
    }
    
    
    pub fn qpl(pid: u32, name: &str, entry: u64, user_stack: u64, ihz: bool) -> Self {
        let mut thread = if ihz {
            Self::iqa(pid, name, entry, 0)
        } else {
            Self::iqj(pid, name, entry, user_stack, 0)
        };
        thread.flags.0 |= ThreadFlags::Tl;
        thread
    }
    
    
    pub fn iqj(pid: u32, name: &str, entry: u64, user_stack: u64, db: u64) -> Self {
        let tid = XC_.fetch_add(1, Ordering::SeqCst);
        
        
        let kernel_stack = Box::new([0u8; OR_]);
        let kernel_stack_top = kernel_stack.as_ptr() as u64 + OR_ as u64;
        
        
        let mut context = ThreadContext::default();
        
        #[cfg(target_arch = "x86_64")]
        {
            
            context.user_rsp = user_stack;
            context.user_rip = entry;
            context.rflags = 0x202; 
            context.cs = crate::gdt::ALF_ as u64;
            context.ss = crate::gdt::ALG_ as u64;
            
            
            context.rsp = kernel_stack_top;
            context.rip = hay as u64;
            
            
            context.r12 = entry;
            context.r13 = user_stack;
            context.r14 = db;
        }
        
        #[cfg(target_arch = "aarch64")]
        {
            context.sp = kernel_stack_top;
            context.lr = hay as u64;
            context.x19 = entry;
            context.x20 = user_stack;
            context.x21 = db;
        }
        
        Self {
            tid,
            pid,
            name: String::from(name),
            state: ThreadState::Ready,
            flags: ThreadFlags(ThreadFlags::Bc),
            context,
            kernel_stack: Some(kernel_stack),
            kernel_stack_top,
            user_stack_top: user_stack,
            entry_point: entry,
            entry_arg: db,
            exit_code: 0,
            cpu_time: 0,
            joiner: None,
        }
    }
    
    
    pub fn ihz(&self) -> bool {
        self.flags.0 & ThreadFlags::Go != 0
    }
    
    
    pub fn qmo(&self) -> bool {
        self.flags.0 & ThreadFlags::Tl != 0
    }
}



#[cfg(target_arch = "x86_64")]
#[unsafe(naked)]
extern "C" fn ebo() {
    core::arch::naked_asm!(
        
        
        "sti",
        
        
        "mov rdi, r13",      
        "call r12",          
        
        
        "mov rdi, rax",      
        "call {exit}",
        
        
        "ud2",
        
        exit = sym thread_exit,
    );
}

#[cfg(target_arch = "aarch64")]
#[unsafe(naked)]
extern "C" fn ebo() {
    core::arch::naked_asm!(
        "msr daifclr, #0xf",   
        "mov x0, x20",         
        "blr x19",             
        "bl {exit}",           
        "brk #0",              
        exit = sym thread_exit,
    );
}

#[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
extern "C" fn ebo() {
    
}




#[cfg(target_arch = "x86_64")]
#[unsafe(naked)]
extern "C" fn hay() {
    core::arch::naked_asm!(
        
        "mov rdi, r12",          
        "mov rsi, r13",          
        "mov rdx, r14",          
        "xor ecx, ecx",         
        "jmp {jump}",
        jump = sym crate::userland::jump_to_ring3_with_args,
    );
}

#[cfg(not(target_arch = "x86_64"))]
extern "C" fn hay() {
    
}


#[no_mangle]
extern "C" fn thread_exit(exit_code: i32) {
    let tid = current_tid();
    
    if let Some(mut thread) = Bo.write().get_mut(&tid) {
        thread.state = ThreadState::Dead;
        thread.exit_code = exit_code;
        
        
        if let Some(joiner_tid) = thread.joiner {
            if let Some(joiner) = Bo.write().get_mut(&joiner_tid) {
                if joiner.state == ThreadState::Blocked {
                    joiner.state = ThreadState::Ready;
                }
            }
        }
    }
    
    crate::log_debug!("[THREAD] Thread {} exited with code {}", tid, exit_code);
    
    
    ajc();
}





lazy_static::lazy_static! {
    
    static ref Bo: RwLock<BTreeMap<Bd, Thread>> = RwLock::new(BTreeMap::new());
}


const LH_: usize = 64;
static TR_: [AtomicU64; LH_] = {
    const Bm: AtomicU64 = AtomicU64::new(QY_);
    [Bm; LH_]
};


const AYY_: Bd = 0x8000_0000_0000_0000;


fn ifp(cpu_id: usize) -> Bd {
    if cpu_id == 0 { 0 } else { AYY_ + cpu_id as u64 }
}


fn ero(tid: Bd) -> bool {
    tid == 0 || tid >= AYY_
}


#[inline]
fn ezm() -> usize {
    (crate::cpu::smp::bll() as usize).min(LH_ - 1)
}


pub fn current_tid() -> Bd {
    TR_[ezm()].load(Ordering::Relaxed)
}


pub fn oos(tid: Bd) {
    TR_[ezm()].store(tid, Ordering::SeqCst);
}


pub fn dzu(name: &str, entry: fn(u64) -> i32, db: u64) -> Bd {
    let pid = crate::process::pe();
    let thread = Thread::iqa(pid, name, entry as *const () as u64, db);
    let tid = thread.tid;
    
    Bo.write().insert(tid, thread);
    ciw(tid);
    
    crate::log_debug!("[THREAD] Spawned kernel thread {} '{}'", tid, name);
    tid
}


pub fn jhc(pid: u32, name: &str, entry: u64, user_stack: u64, db: u64) -> Bd {
    let thread = Thread::iqj(pid, name, entry, user_stack, db);
    let tid = thread.tid;
    
    Bo.write().insert(tid, thread);
    ciw(tid);
    
    crate::log_debug!("[THREAD] Spawned user thread {} '{}'", tid, name);
    tid
}


pub fn ajc() {
    boq();
}


pub fn exit(code: i32) {
    thread_exit(code);
}


pub fn wake(tid: Bd) {
    let mut zn = Bo.write();
    if let Some(thread) = zn.get_mut(&tid) {
        if thread.state == ThreadState::Blocked {
            thread.state = ThreadState::Ready;
            drop(zn);
            ciw(tid);
        }
    }
}


pub fn block(tid: Bd) {
    let mut zn = Bo.write();
    if let Some(thread) = zn.get_mut(&tid) {
        thread.state = ThreadState::Blocked;
    }
}





pub fn hig() {
    let tid = current_tid();
    if ero(tid) {
        return; 
    }
    {
        let mut zn = Bo.write();
        if let Some(thread) = zn.get_mut(&tid) {
            thread.state = ThreadState::Blocked;
        }
    }
    
    boq();
}




pub fn cds(brr: u64) {
    let tid = current_tid();
    if ero(tid) {
        return;
    }

    
    crate::time::oek(tid, brr);

    
    hig();
}


pub fn otp(duration_ns: u64) {
    let brq = crate::time::cbx().saturating_add(duration_ns);
    cds(brq);
}





use alloc::collections::VecDeque;






struct PerCpuQueue {
    queue: Mutex<VecDeque<Bd>>,
}

impl PerCpuQueue {
    const fn new() -> Self {
        Self { queue: Mutex::new(VecDeque::new()) }
    }
    
    fn push(&self, tid: Bd) {
        self.queue.lock().push_back(tid);
    }
    
    fn pop(&self) -> Option<Bd> {
        self.queue.lock().pop_front()
    }
    
    
    fn jit(&self) -> Option<Bd> {
        self.queue.lock().pop_back()
    }
    
    fn len(&self) -> usize {
        self.queue.lock().len()
    }
    
    
    fn try_push(&self, tid: Bd) -> bool {
        if let Some(mut q) = self.queue.try_lock() {
            q.push_back(tid);
            true
        } else {
            false
        }
    }
    
    fn try_pop(&self) -> Option<Bd> {
        self.queue.try_lock()?.pop_front()
    }
    
    fn try_steal(&self) -> Option<Bd> {
        self.queue.try_lock()?.pop_back()
    }
    
    fn try_len(&self) -> usize {
        self.queue.try_lock().map_or(0, |q| q.len())
    }
}


static HP_: [PerCpuQueue; LH_] = {
    const Bm: PerCpuQueue = PerCpuQueue::new();
    [Bm; LH_]
};


static CKX_: AtomicU64 = AtomicU64::new(0);


lazy_static::lazy_static! {
    static ref CQU_: Mutex<VecDeque<Bd>> = Mutex::new(VecDeque::new());
}


fn ciw(tid: Bd) {
    let num_cpus = crate::cpu::smp::ail().max(1) as usize;
    
    
    let target_cpu = (CKX_.fetch_add(1, Ordering::Relaxed) % num_cpus as u64) as usize;
    HP_[target_cpu].push(tid);
    
    
    let laj = ezm();
    if target_cpu != laj && target_cpu > 0 {
        crate::cpu::smp::jeo(target_cpu as u32);
    }
}


fn poc(my_cpu: usize) -> Option<Bd> {
    let num_cpus = crate::cpu::smp::ail().max(1) as usize;
    
    
    let mut fit = usize::MAX;
    let mut atb = 0;
    
    for cpu in 0..num_cpus {
        if cpu == my_cpu { continue; }
        let len = HP_[cpu].try_len();
        if len > atb {
            atb = len;
            fit = cpu;
        }
    }
    
    if fit < LH_ && atb > 1 {
        return HP_[fit].try_steal();
    }
    
    
    CQU_.try_lock()?.pop_front()
}


pub fn init() {
    crate::serial_println!("[THREAD] Creating idle thread...");
    
    crate::arch::bag(|| {
        
        let mnu = Thread {
            tid: 0,
            pid: 0,
            name: String::from("idle"),
            state: ThreadState::Running,
            flags: ThreadFlags(ThreadFlags::Go | ThreadFlags::Tl),
            context: ThreadContext::default(),
            kernel_stack: None,
            kernel_stack_top: 0,
            user_stack_top: 0,
            entry_point: 0,
            entry_arg: 0,
            exit_code: 0,
            cpu_time: 0,
            joiner: None,
        };
        
        Bo.write().insert(0, mnu);
        TR_[0].store(0, Ordering::SeqCst);
    });
    crate::serial_println!("[THREAD] Thread subsystem initialized");
}



pub fn mow(cpu_id: u32) {
    let idx = cpu_id as usize;
    let eqc = ifp(idx);
    
    let ckv = Thread {
        tid: eqc,
        pid: 0,
        name: String::from("idle-ap"),
        state: ThreadState::Running,
        flags: ThreadFlags(ThreadFlags::Go),
        context: ThreadContext::default(),
        kernel_stack: None,
        kernel_stack_top: 0,
        user_stack_top: 0,
        entry_point: 0,
        entry_arg: 0,
        exit_code: 0,
        cpu_time: 0,
        joiner: None,
    };
    
    Bo.write().insert(eqc, ckv);
    TR_[idx].store(eqc, Ordering::SeqCst);
    
    crate::serial_println!("[THREAD] AP {} idle thread created (TID={:#x})", cpu_id, eqc);
}


pub fn dvv() {
    let tid = current_tid();
    
    
    if let Some(mut zn) = Bo.try_write() {
        if let Some(thread) = zn.get_mut(&tid) {
            thread.cpu_time += 1;
        }
    }
    
    
    static DBP_: AtomicU64 = AtomicU64::new(0);
    let gx = DBP_.fetch_add(1, Ordering::Relaxed);
    
    if gx % 10 == 0 {
        boq();
    }
}


pub fn boq() {
    let cpu_id = ezm();
    let current = current_tid();
    let ckv = ifp(cpu_id);
    
    
    
    if current != QY_ && !ero(current) {
        let orv = if let Some(zn) = Bo.try_read() {
            zn.get(&current).map_or(false, |t| t.state == ThreadState::Running)
        } else {
            return; 
        };
        if orv {
            
            if !HP_[cpu_id].try_push(current) {
                return; 
            }
            if let Some(mut zn) = Bo.try_write() {
                if let Some(t) = zn.get_mut(&current) {
                    t.state = ThreadState::Ready;
                }
            }
        }
    }
    
    
    let nkj = loop {
        
        if let Some(tid) = HP_[cpu_id].try_pop() {
            let gsd = if let Some(zn) = Bo.try_read() {
                zn.get(&tid).map_or(false, |t| t.state == ThreadState::Ready || t.state == ThreadState::Running)
            } else {
                
                let _ = HP_[cpu_id].try_push(tid);
                return;
            };
            if gsd {
                break Some(tid);
            }
            continue; 
        }
        
        
        if let Some(tid) = poc(cpu_id) {
            let gsd = if let Some(zn) = Bo.try_read() {
                zn.get(&tid).map_or(false, |t| t.state == ThreadState::Ready || t.state == ThreadState::Running)
            } else {
                let _ = HP_[cpu_id].try_push(tid);
                return;
            };
            if gsd {
                break Some(tid);
            }
            continue;
        }
        
        
        break None;
    };
    
    match nkj {
        Some(next) if next != current => {
            
            if let Some(mut zn) = Bo.try_write() {
                if let Some(thread) = zn.get_mut(&next) {
                    thread.state = ThreadState::Running;
                }
            } else {
                
                let _ = HP_[cpu_id].try_push(next);
                return;
            }
            
            
            hnm(current, next);
        }
        None if !ero(current) && current != QY_ => {
            
            hnm(current, ckv);
        }
        _ => {
            
        }
    }
}


fn hnm(from: Bd, to: Bd) {
    if from == to {
        return;
    }
    
    
    let iac: *mut ThreadContext;
    let jmy: *const ThreadContext;
    let fda: u64;
    
    {
        let mut zn = match Bo.try_write() {
            Some(t) => t,
            None => return, 
        };
        
        let lzs = match zn.get_mut(&from) {
            Some(t) => t as *mut Thread,
            None => return,
        };
        
        let jnc = match zn.get(&to) {
            Some(t) => t,
            None => return,
        };
        
        iac = unsafe { &mut (*lzs).context as *mut ThreadContext };
        jmy = &jnc.context as *const ThreadContext;
        fda = jnc.kernel_stack_top;
    }
    
    
    #[cfg(target_arch = "x86_64")]
    if fda != 0 {
        crate::gdt::jfg(fda);
        
        
        
        unsafe {
            crate::userland::KERNEL_SYSCALL_STACK_TOP = fda;
        }
    }
    
    
    oos(to);
    
    
    unsafe {
        gwu(iac, jmy);
    }
}


#[cfg(target_arch = "x86_64")]
#[unsafe(naked)]
extern "C" fn gwu(from: *mut ThreadContext, to: *const ThreadContext) {
    core::arch::naked_asm!(
        
        
        
        
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
#[unsafe(naked)]
extern "C" fn gwu(_from: *mut ThreadContext, _to: *const ThreadContext) {
    core::arch::naked_asm!(
        
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
extern "C" fn gwu(_from: *mut ThreadContext, _to: *const ThreadContext) {
    
}






pub struct Afh {
    locked: AtomicU32,
    owner: AtomicU64,
    waiters: Mutex<VecDeque<Bd>>,
}

impl Afh {
    pub const fn new() -> Self {
        Self {
            locked: AtomicU32::new(0),
            owner: AtomicU64::new(QY_),
            waiters: Mutex::new(VecDeque::new()),
        }
    }
    
    pub fn lock(&self) {
        let tid = current_tid();
        
        loop {
            
            if self.locked.compare_exchange(0, 1, Ordering::Acquire, Ordering::Relaxed).is_ok() {
                self.owner.store(tid, Ordering::Relaxed);
                return;
            }
            
            
            {
                self.waiters.lock().push_back(tid);
                if let Some(thread) = Bo.write().get_mut(&tid) {
                    thread.state = ThreadState::Blocked;
                }
            }
            
            
            ajc();
        }
    }
    
    pub fn unlock(&self) {
        self.owner.store(QY_, Ordering::Relaxed);
        self.locked.store(0, Ordering::Release);
        
        
        if let Some(avt) = self.waiters.lock().pop_front() {
            if let Some(thread) = Bo.write().get_mut(&avt) {
                thread.state = ThreadState::Ready;
            }
            ciw(avt);
        }
    }
    
    pub fn try_lock(&self) -> bool {
        let tid = current_tid();
        
        if self.locked.compare_exchange(0, 1, Ordering::Acquire, Ordering::Relaxed).is_ok() {
            self.owner.store(tid, Ordering::Relaxed);
            true
        } else {
            false
        }
    }
}


pub struct Qg {
    count: AtomicU32,
    waiters: Mutex<VecDeque<Bd>>,
}

impl Qg {
    pub const fn new(are: u32) -> Self {
        Self {
            count: AtomicU32::new(are),
            waiters: Mutex::new(VecDeque::new()),
        }
    }
    
    pub fn bqb(&self) {
        loop {
            let count = self.count.load(Ordering::Relaxed);
            
            if count > 0 {
                if self.count.compare_exchange(count, count - 1, Ordering::Acquire, Ordering::Relaxed).is_ok() {
                    return;
                }
            } else {
                
                let tid = current_tid();
                {
                    self.waiters.lock().push_back(tid);
                    if let Some(thread) = Bo.write().get_mut(&tid) {
                        thread.state = ThreadState::Blocked;
                    }
                }
                ajc();
            }
        }
    }
    
    pub fn ash(&self) {
        self.count.fetch_add(1, Ordering::Release);
        
        
        if let Some(avt) = self.waiters.lock().pop_front() {
            if let Some(thread) = Bo.write().get_mut(&avt) {
                thread.state = ThreadState::Ready;
            }
            ciw(avt);
        }
    }
}


pub struct Aia {
    waiters: Mutex<VecDeque<Bd>>,
}

impl Aia {
    pub const fn new() -> Self {
        Self {
            waiters: Mutex::new(VecDeque::new()),
        }
    }
    
    
    pub fn bqb(&self, mutex: &Afh) {
        let tid = current_tid();
        
        
        self.waiters.lock().push_back(tid);
        
        
        if let Some(thread) = Bo.write().get_mut(&tid) {
            thread.state = ThreadState::Blocked;
        }
        
        
        mutex.unlock();
        
        
        ajc();
        
        
        mutex.lock();
    }
    
    
    pub fn ash(&self) {
        if let Some(avt) = self.waiters.lock().pop_front() {
            if let Some(thread) = Bo.write().get_mut(&avt) {
                thread.state = ThreadState::Ready;
            }
            ciw(avt);
        }
    }
    
    
    pub fn hiq(&self) {
        let mut waiters = self.waiters.lock();
        while let Some(avt) = waiters.pop_front() {
            if let Some(thread) = Bo.write().get_mut(&avt) {
                thread.state = ThreadState::Ready;
            }
            ciw(avt);
        }
    }
}





pub fn mzi() -> alloc::vec::Vec<(u64, u32, ThreadState, alloc::string::String)> {
    let zn = Bo.read();
    let mut result = alloc::vec::Vec::new();
    
    for (tid, thread) in zn.iter() {
        result.push((*tid, thread.pid, thread.state, thread.name.clone()));
    }
    
    
    result.sort_by_key(|(tid, _, _, _)| *tid);
    result
}
