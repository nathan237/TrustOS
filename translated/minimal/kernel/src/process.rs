




use alloc::string::String;
use alloc::vec::Vec;
use alloc::collections::BTreeMap;
use alloc::boxed::Box;
use alloc::sync::Arc;
use spin::{RwLock, Mutex};
use core::sync::atomic::{AtomicU32, AtomicU64, Ordering};
use crate::memory::AddressSpace;


pub type X = u32;


pub const JL_: X = 0;
pub const JK_: X = 1;


#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProcessState {
    
    Created,
    
    Ready,
    
    Running,
    
    Blocked,
    
    Waiting,
    
    Stopped,
    
    Zombie,
    
    Dead,
}


#[derive(Clone, Copy, Debug)]
pub struct ProcessFlags(pub u32);

impl ProcessFlags {
    pub const Bc: u32 = 0;
    pub const Go: u32 = 1 << 0;      
    pub const Aie: u32 = 1 << 1;      
    pub const Bm: u32 = 1 << 2;        
}


#[derive(Clone, Debug)]
pub struct Ju {
    pub vfs_fd: i32,    
    pub flags: u32,     
}


#[derive(Clone, Debug, Default)]
pub struct MemoryLayout {
    pub code_start: u64,
    pub code_end: u64,
    pub data_start: u64,
    pub data_end: u64,
    pub heap_start: u64,
    pub heap_end: u64,
    pub stack_start: u64,
    pub stack_end: u64,
}


#[derive(Clone, Debug, Default)]
#[repr(C)]
pub struct CpuContext {
    
    pub rax: u64,
    pub rbx: u64,
    pub rcx: u64,
    pub rdx: u64,
    pub rsi: u64,
    pub rdi: u64,
    pub rbp: u64,
    pub rsp: u64,
    pub r8: u64,
    pub r9: u64,
    pub r10: u64,
    pub r11: u64,
    pub r12: u64,
    pub r13: u64,
    pub r14: u64,
    pub r15: u64,
    
    pub rip: u64,
    
    pub rflags: u64,
    
    pub cs: u64,
    pub ss: u64,
}


#[derive(Clone)]
pub struct Process {
    
    pub pid: X,
    
    pub ppid: X,
    
    pub name: String,
    
    pub state: ProcessState,
    
    pub flags: ProcessFlags,
    
    pub exit_code: i32,
    
    pub context: CpuContext,
    
    pub memory: MemoryLayout,
    
    pub fd_table: BTreeMap<i32, Ju>,
    
    next_fd: i32,
    
    pub cwd: String,
    
    pub env: BTreeMap<String, String>,
    
    pub cpu_time: u64,
    
    pub children: Vec<X>,
    
    pub cr3: u64,
    
    pub address_space: Option<Arc<Mutex<AddressSpace>>>,
    
    pub pgid: u32,
    
    pub sid: u32,
    
    pub controlling_tty: Option<u32>,
    
    pub root_dir: String,
    
    pub uid: u32,
    
    pub gid: u32,
    
    pub euid: u32,
    
    pub egid: u32,
    
    pub umask: u32,
}

impl Process {
    
    pub fn new(pid: X, ppid: X, name: &str, flags: ProcessFlags) -> Self {
        let mut fd_table = BTreeMap::new();
        
        
        
        fd_table.insert(0, Ju { vfs_fd: 0, flags: 0 });
        fd_table.insert(1, Ju { vfs_fd: 1, flags: 0 });
        fd_table.insert(2, Ju { vfs_fd: 2, flags: 0 });
        
        
        let (address_space, cr3) = if flags.0 & ProcessFlags::Go != 0 {
            
            (None, crate::memory::paging::kernel_cr3())
        } else {
            
            match AddressSpace::bnt() {
                Some(space) => {
                    let cr3 = space.cr3();
                    (Some(Arc::new(Mutex::new(space))), cr3)
                }
                None => {
                    
                    (None, crate::memory::paging::kernel_cr3())
                }
            }
        };
        
        Self {
            pid,
            ppid,
            name: String::from(name),
            state: ProcessState::Created,
            flags,
            exit_code: 0,
            context: CpuContext::default(),
            memory: MemoryLayout::default(),
            fd_table,
            next_fd: 3,
            cwd: String::from("/"),
            env: BTreeMap::new(),
            cpu_time: 0,
            children: Vec::new(),
            cr3,
            address_space,
            pgid: pid,
            sid: pid,
            controlling_tty: None,
            root_dir: String::from("/"),
            uid: crate::auth::fpz(),
            gid: crate::auth::fpp(),
            euid: crate::auth::fpz(),
            egid: crate::auth::fpp(),
            umask: 0o022,
        }
    }
    
    
    pub fn alloc_fd(&mut self, vfs_fd: i32) -> i32 {
        let fd = self.next_fd;
        self.next_fd += 1;
        self.fd_table.insert(fd, Ju { vfs_fd, flags: 0 });
        fd
    }
    
    
    pub fn qaf(&mut self, fd: i32) -> Option<Ju> {
        self.fd_table.remove(&fd)
    }
    
    
    pub fn qiw(&self, fd: i32) -> Option<i32> {
        self.fd_table.get(&fd).map(|e| e.vfs_fd)
    }
    
    
    pub fn qwr(&mut self, key: &str, value: &str) {
        self.env.insert(String::from(key), String::from(value));
    }
    
    
    pub fn qjb(&self, key: &str) -> Option<&str> {
        self.env.get(key).map(|j| j.as_str())
    }
    
    
    pub fn ftn(&mut self, old_fd: i32) -> Result<i32, &'static str> {
        let entry = self.fd_table.get(&old_fd).ok_or("Bad fd")?.clone();
        let ue = self.next_fd;
        self.next_fd += 1;
        self.fd_table.insert(ue, entry);
        Ok(ue)
    }
    
    
    pub fn hui(&mut self, old_fd: i32, ue: i32) -> Result<i32, &'static str> {
        if old_fd == ue { return Ok(ue); }
        let entry = self.fd_table.get(&old_fd).ok_or("Bad fd")?.clone();
        self.fd_table.remove(&ue);
        self.fd_table.insert(ue, entry);
        Ok(ue)
    }
}


pub struct ProcessTable {
    pub processes: BTreeMap<X, Process>,
    next_pid: AtomicU32,
}

impl ProcessTable {
    const fn new() -> Self {
        Self {
            processes: BTreeMap::new(),
            next_pid: AtomicU32::new(JK_),
        }
    }
    
    fn alloc_pid(&self) -> X {
        self.next_pid.fetch_add(1, Ordering::SeqCst)
    }
}

pub static AE_: RwLock<ProcessTable> = RwLock::new(ProcessTable::new());
static ART_: AtomicU32 = AtomicU32::new(JL_);


pub fn init() {
    crate::log!("[PROC] Initializing process manager...");
    
    
    let mvm = Process::new(
        JL_,
        JL_,
        "kernel",
        ProcessFlags(ProcessFlags::Go)
    );
    
    {
        let mut bs = AE_.write();
        bs.processes.insert(JL_, mvm);
    }
    
    crate::log_debug!("[PROC] Kernel process created (PID 0)");
    crate::log!("[OK] Process manager ready");
}


pub fn create(name: &str, ppid: X) -> Result<X, &'static str> {
    let mut bs = AE_.write();
    
    let pid = bs.alloc_pid();
    let mut jj = Process::new(pid, ppid, name, ProcessFlags(ProcessFlags::Bc));
    
    
    if let Some(parent) = bs.processes.get(&ppid) {
        jj.cwd = parent.cwd.clone();
        jj.env = parent.env.clone();
    }
    
    
    if let Some(parent) = bs.processes.get_mut(&ppid) {
        parent.children.push(pid);
    }
    
    jj.state = ProcessState::Ready;
    bs.processes.insert(pid, jj);
    
    crate::log_debug!("[PROC] Created process {} ({})", pid, name);
    Ok(pid)
}






pub fn lxk() -> Result<X, &'static str> {
    let current = pe();
    
    
    let (name, cwd, env, fd_table, next_fd, parent_cr3, memory, uid, gid, euid, egid, umask, pgid, sid, controlling_tty, root_dir) = {
        let bs = AE_.read();
        let parent = bs.processes.get(&current)
            .ok_or("Current process not found")?;
        (
            parent.name.clone(),
            parent.cwd.clone(),
            parent.env.clone(),
            parent.fd_table.clone(),
            parent.next_fd,
            parent.cr3,
            parent.memory.clone(),
            parent.uid,
            parent.gid,
            parent.euid,
            parent.egid,
            parent.umask,
            parent.pgid,
            parent.sid,
            parent.controlling_tty,
            parent.root_dir.clone(),
        )
    };
    
    
    let kernel_cr3 = crate::memory::paging::kernel_cr3();
    let (address_space, cr3) = if parent_cr3 != kernel_cr3 {
        match crate::memory::cow::klf(parent_cr3) {
            Some(space) => {
                let c = space.cr3();
                (Some(Arc::new(Mutex::new(space))), c)
            }
            None => {
                
                match AddressSpace::bnt() {
                    Some(space) => {
                        let c = space.cr3();
                        (Some(Arc::new(Mutex::new(space))), c)
                    }
                    None => (None, kernel_cr3)
                }
            }
        }
    } else {
        (None, kernel_cr3)
    };
    
    
    let mut bs = AE_.write();
    let pid = bs.alloc_pid();
    
    let pd = Process {
        pid,
        ppid: current,
        name,
        state: ProcessState::Ready,
        flags: ProcessFlags(ProcessFlags::Bc),
        exit_code: 0,
        context: CpuContext::default(),
        memory,
        fd_table,   
        next_fd,
        cwd,
        env,
        cpu_time: 0,
        children: Vec::new(),
        cr3,
        address_space,
        pgid,       
        sid,        
        controlling_tty,
        root_dir,
        uid,
        gid,
        euid,
        egid,
        umask,
    };
    
    if let Some(parent) = bs.processes.get_mut(&current) {
        parent.children.push(pid);
    }
    bs.processes.insert(pid, pd);
    
    
    crate::signals::gcp(pid);
    
    crate::log_debug!("[PROC] COW-fork: {} -> {}", current, pid);
    Ok(pid)
}


pub fn exit(code: i32) {
    let current = pe();
    let mut bs = AE_.write();
    
    if let Some(jj) = bs.processes.get_mut(&current) {
        jj.state = ProcessState::Zombie;
        jj.exit_code = code;
        
        
        let children: Vec<X> = jj.children.drain(..).collect();
        for child_pid in children {
            if let Some(pd) = bs.processes.get_mut(&child_pid) {
                pd.ppid = JK_;
            }
            if let Some(init) = bs.processes.get_mut(&JK_) {
                init.children.push(child_pid);
            }
        }
        
        crate::log_debug!("[PROC] Process {} exited with code {}", current, code);
    }
}


pub fn bqb(pid: X) -> Result<i32, &'static str> {
    let mut bs = AE_.write();
    
    let jj = bs.processes.get(&pid).ok_or("Process not found")?;
    
    if jj.state != ProcessState::Zombie {
        return Err("Process not yet exited");
    }
    
    let exit_code = jj.exit_code;
    
    
    bs.processes.remove(&pid);
    
    Ok(exit_code)
}


pub fn pe() -> X {
    ART_.load(Ordering::Relaxed)
}


pub fn faf(pid: X) {
    ART_.store(pid, Ordering::SeqCst);
}


pub fn bfs() -> (u32, u32, u32, u32) {
    let bs = AE_.read();
    if let Some(aa) = bs.processes.get(&pe()) {
        (aa.uid, aa.gid, aa.euid, aa.egid)
    } else {
        (0, 0, 0, 0) 
    }
}


pub fn jfm(pid: X, uid: u32) -> Result<(), &'static str> {
    let mut bs = AE_.write();
    let jj = bs.processes.get_mut(&pid).ok_or("No such process")?;
    
    if jj.euid == 0 || uid == jj.uid {
        jj.uid = uid;
        jj.euid = uid;
        Ok(())
    } else {
        Err("EPERM")
    }
}


pub fn jff(pid: X, gid: u32) -> Result<(), &'static str> {
    let mut bs = AE_.write();
    let jj = bs.processes.get_mut(&pid).ok_or("No such process")?;
    if jj.euid == 0 || gid == jj.gid {
        jj.gid = gid;
        jj.egid = gid;
        Ok(())
    } else {
        Err("EPERM")
    }
}


pub fn opt(pid: X, mask: u32) -> u32 {
    let mut bs = AE_.write();
    if let Some(jj) = bs.processes.get_mut(&pid) {
        let qb = jj.umask;
        jj.umask = mask & 0o777;
        qb
    } else {
        0o022
    }
}


pub fn get(pid: X) -> Option<Process> {
    AE_.read().processes.get(&pid).cloned()
}


pub fn current() -> Option<Process> {
    get(pe())
}




#[inline]
pub fn bwz<U, F: FnOnce(&Process) -> U>(pid: X, f: F) -> Option<U> {
    let bs = AE_.read();
    bs.processes.get(&pid).map(f)
}


#[inline]
pub fn pux<U, F: FnOnce(&Process) -> U>(f: F) -> Option<U> {
    bwz(pe(), f)
}


pub fn is_running(pid: X) -> bool {
    AE_.read().processes.get(&pid)
        .map(|aa| aa.state == ProcessState::Running)
        .unwrap_or(false)
}


pub fn apc(pid: X, state: ProcessState) {
    if let Some(jj) = AE_.write().processes.get_mut(&pid) {
        jj.state = state;
    }
}


pub fn list() -> Vec<(X, String, ProcessState)> {
    AE_.read()
        .processes
        .iter()
        .map(|(pid, jj)| (*pid, jj.name.clone(), jj.state))
        .collect()
}


pub fn count() -> usize {
    AE_.read().processes.len()
}


pub fn bne(pid: X) -> Result<(), &'static str> {
    if pid == JL_ || pid == JK_ {
        return Err("Cannot kill kernel or init");
    }
    
    let mut bs = AE_.write();
    
    if let Some(jj) = bs.processes.get_mut(&pid) {
        jj.state = ProcessState::Dead;
        crate::log_debug!("[PROC] Process {} killed", pid);
        Ok(())
    } else {
        Err("Process not found")
    }
}




pub fn spawn(name: &str) -> Result<X, &'static str> {
    let ppid = pe();
    let pid = create(name, ppid)?;
    
    
    crate::signals::gcp(pid);
    
    crate::log!("[PROC] Spawned process {} ({}) under parent {}", pid, name, ppid);
    Ok(pid)
}


pub fn gwd(pid: X) {
    apc(pid, ProcessState::Running);
    faf(pid);
}


pub fn finish(pid: X, exit_code: i32) {
    let mut bs = AE_.write();
    if let Some(jj) = bs.processes.get_mut(&pid) {
        jj.state = ProcessState::Zombie;
        jj.exit_code = exit_code;
        crate::log_debug!("[PROC] Process {} exited with code {}", pid, exit_code);
    }
}


pub fn gqo(pid: X) {
    let mut bs = AE_.write();
    
    
    if let Some(jj) = bs.processes.get(&pid) {
        let children: Vec<X> = jj.children.clone();
        for child_pid in children {
            if let Some(pd) = bs.processes.get_mut(&child_pid) {
                pd.ppid = JL_;
            }
        }
    }
    
    bs.processes.remove(&pid);
    
    
    crate::signals::flu(pid);
    
    crate::log_debug!("[PROC] Reaped process {}", pid);
}


pub fn opf(pid: X, memory: MemoryLayout) {
    if let Some(jj) = AE_.write().processes.get_mut(&pid) {
        jj.memory = memory;
    }
}


pub fn qrc() {
    let bs = AE_.read();
    
    fn iwo(bs: &ProcessTable, pid: X, depth: usize) {
        if let Some(jj) = bs.processes.get(&pid) {
            let axq: String = (0..depth).map(|_| "  ").collect();
            crate::serial_println!("{}[{}] {} ({:?})", axq, pid, jj.name, jj.state);
            for pd in &jj.children {
                iwo(bs, *pd, depth + 1);
            }
        }
    }
    
    crate::serial_println!("Process tree:");
    iwo(&bs, JL_, 0);
}


pub fn crk(pid: X) {
    let mut bs = AE_.write();
    if let Some(jj) = bs.processes.get_mut(&pid) {
        jj.state = ProcessState::Zombie;
        jj.exit_code = -9; 
    }
}


pub fn stop(pid: X) {
    let mut bs = AE_.write();
    if let Some(jj) = bs.processes.get_mut(&pid) {
        jj.state = ProcessState::Stopped;
    }
}


pub fn resume(pid: X) {
    let mut bs = AE_.write();
    if let Some(jj) = bs.processes.get_mut(&pid) {
        if jj.state == ProcessState::Stopped {
            jj.state = ProcessState::Ready;
        }
    }
}


pub fn gmc(pid: X) -> Option<X> {
    let bs = AE_.read();
    bs.processes.get(&pid).map(|aa| aa.ppid)
}


pub fn cyj(pid: X) -> Option<CpuContext> {
    let bs = AE_.read();
    bs.processes.get(&pid).map(|aa| aa.context.clone())
}


pub fn gug(pid: X, ab: &CpuContext) -> Result<(), &'static str> {
    let mut bs = AE_.write();
    if let Some(jj) = bs.processes.get_mut(&pid) {
        jj.context = ab.clone();
        Ok(())
    } else {
        Err("Process not found")
    }
}






pub fn oph(pid: X, pgid: X) -> Result<(), &'static str> {
    let mut bs = AE_.write();
    let bwg = if pid == 0 { pe() } else { pid };
    let njl = if pgid == 0 { bwg } else { pgid };
    let jj = bs.processes.get_mut(&bwg).ok_or("No such process")?;
    jj.pgid = njl;
    Ok(())
}


pub fn ibs(pid: X) -> u32 {
    let bs = AE_.read();
    let target = if pid == 0 { pe() } else { pid };
    bs.processes.get(&target).map(|aa| aa.pgid).unwrap_or(0)
}


pub fn ibv(pid: X) -> u32 {
    let bs = AE_.read();
    let target = if pid == 0 { pe() } else { pid };
    bs.processes.get(&target).map(|aa| aa.sid).unwrap_or(0)
}



pub fn opy() -> Result<u32, &'static str> {
    let pid = pe();
    let mut bs = AE_.write();
    let jj = bs.processes.get_mut(&pid).ok_or("No such process")?;
    
    if jj.pgid == pid {
        
    }
    jj.sid = pid;
    jj.pgid = pid;
    jj.controlling_tty = None;
    Ok(pid)
}


pub fn jfa(pid: X, ect: u32) {
    let mut bs = AE_.write();
    if let Some(jj) = bs.processes.get_mut(&pid) {
        jj.controlling_tty = Some(ect);
    }
}


pub fn qhj(pid: X) -> Option<u32> {
    let bs = AE_.read();
    bs.processes.get(&pid).and_then(|aa| aa.controlling_tty)
}


pub fn kkk(pid: X, new_root: &str) -> Result<(), &'static str> {
    let mut bs = AE_.write();
    let jj = bs.processes.get_mut(&pid).ok_or("No such process")?;
    
    if jj.euid != 0 {
        return Err("EPERM");
    }
    jj.root_dir = String::from(new_root);
    Ok(())
}


pub fn qik(pid: X) -> String {
    let bs = AE_.read();
    bs.processes.get(&pid).map(|aa| aa.root_dir.clone()).unwrap_or_else(|| String::from("/"))
}


pub fn nun(pgid: u32) -> Vec<X> {
    let bs = AE_.read();
    bs.processes.iter()
        .filter(|(_, aa)| aa.pgid == pgid)
        .map(|(&pid, _)| pid)
        .collect()
}
