




use alloc::string::String;
use alloc::vec::Vec;
use alloc::collections::BTreeMap;
use alloc::boxed::Box;
use alloc::sync::Arc;
use spin::{RwLock, Mutex};
use core::sync::atomic::{AtomicU32, AtomicU64, Ordering};
use crate::memory::AddressSpace;


pub type Ah = u32;


pub const IT_: Ah = 0;
pub const IS_: Ah = 1;


#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProcessState {
    
    Cu,
    
    At,
    
    Ai,
    
    Hj,
    
    Bwo,
    
    Af,
    
    Vf,
    
    Ez,
}


#[derive(Clone, Copy, Debug)]
pub struct ProcessFlags(pub u32);

impl ProcessFlags {
    pub const Cq: u32 = 0;
    pub const Ps: u32 = 1 << 0;      
    pub const Cad: u32 = 1 << 1;      
    pub const Dm: u32 = 1 << 2;        
}


#[derive(Clone, Debug)]
pub struct Wp {
    pub gwc: i32,    
    pub flags: u32,     
}


#[derive(Clone, Debug, Default)]
pub struct MemoryLayout {
    pub dez: u64,
    pub kjr: u64,
    pub bjt: u64,
    pub njm: u64,
    pub caa: u64,
    pub ecv: u64,
    pub ibo: u64,
    pub ibm: u64,
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
    
    pub pc: u64,
    
    pub rflags: u64,
    
    pub aap: u64,
    pub rv: u64,
}


#[derive(Clone)]
pub struct Process {
    
    pub ce: Ah,
    
    pub bfb: Ah,
    
    pub j: String,
    
    pub g: ProcessState,
    
    pub flags: ProcessFlags,
    
    pub nz: i32,
    
    pub context: CpuContext,
    
    pub memory: MemoryLayout,
    
    pub buf: BTreeMap<i32, Wp>,
    
    bca: i32,
    
    pub jv: String,
    
    pub env: BTreeMap<String, String>,
    
    pub cdu: u64,
    
    pub zf: Vec<Ah>,
    
    pub jm: u64,
    
    pub ze: Option<Arc<Mutex<AddressSpace>>>,
    
    pub bai: u32,
    
    pub ary: u32,
    
    pub ffs: Option<u32>,
    
    pub grb: String,
    
    pub pi: u32,
    
    pub pw: u32,
    
    pub ahl: u32,
    
    pub bqj: u32,
    
    pub gvl: u32,
}

impl Process {
    
    pub fn new(ce: Ah, bfb: Ah, j: &str, flags: ProcessFlags) -> Self {
        let mut buf = BTreeMap::new();
        
        
        
        buf.insert(0, Wp { gwc: 0, flags: 0 });
        buf.insert(1, Wp { gwc: 1, flags: 0 });
        buf.insert(2, Wp { gwc: 2, flags: 0 });
        
        
        let (ze, jm) = if flags.0 & ProcessFlags::Ps != 0 {
            
            (None, crate::memory::paging::ade())
        } else {
            
            match AddressSpace::dtn() {
                Some(atm) => {
                    let jm = atm.jm();
                    (Some(Arc::new(Mutex::new(atm))), jm)
                }
                None => {
                    
                    (None, crate::memory::paging::ade())
                }
            }
        };
        
        Self {
            ce,
            bfb,
            j: String::from(j),
            g: ProcessState::Cu,
            flags,
            nz: 0,
            context: CpuContext::default(),
            memory: MemoryLayout::default(),
            buf,
            bca: 3,
            jv: String::from("/"),
            env: BTreeMap::new(),
            cdu: 0,
            zf: Vec::new(),
            jm,
            ze,
            bai: ce,
            ary: ce,
            ffs: None,
            grb: String::from("/"),
            pi: crate::auth::kne(),
            pw: crate::auth::kmu(),
            ahl: crate::auth::kne(),
            bqj: crate::auth::kmu(),
            gvl: 0o022,
        }
    }
    
    
    pub fn jzz(&mut self, gwc: i32) -> i32 {
        let da = self.bca;
        self.bca += 1;
        self.buf.insert(da, Wp { gwc, flags: 0 });
        da
    }
    
    
    pub fn yiu(&mut self, da: i32) -> Option<Wp> {
        self.buf.remove(&da)
    }
    
    
    pub fn yug(&self, da: i32) -> Option<i32> {
        self.buf.get(&da).map(|aa| aa.gwc)
    }
    
    
    pub fn znv(&mut self, bs: &str, bn: &str) {
        self.env.insert(String::from(bs), String::from(bn));
    }
    
    
    pub fn yul(&self, bs: &str) -> Option<&str> {
        self.env.get(bs).map(|e| e.as_str())
    }
    
    
    pub fn ksb(&mut self, bns: i32) -> Result<i32, &'static str> {
        let bt = self.buf.get(&bns).ok_or("Bad fd")?.clone();
        let anp = self.bca;
        self.bca += 1;
        self.buf.insert(anp, bt);
        Ok(anp)
    }
    
    
    pub fn noj(&mut self, bns: i32, anp: i32) -> Result<i32, &'static str> {
        if bns == anp { return Ok(anp); }
        let bt = self.buf.get(&bns).ok_or("Bad fd")?.clone();
        self.buf.remove(&anp);
        self.buf.insert(anp, bt);
        Ok(anp)
    }
}


pub struct ProcessTable {
    pub ye: BTreeMap<Ah, Process>,
    oqm: AtomicU32,
}

impl ProcessTable {
    const fn new() -> Self {
        Self {
            ye: BTreeMap::new(),
            oqm: AtomicU32::new(IS_),
        }
    }
    
    fn muz(&self) -> Ah {
        self.oqm.fetch_add(1, Ordering::SeqCst)
    }
}

pub static AD_: RwLock<ProcessTable> = RwLock::new(ProcessTable::new());
static APR_: AtomicU32 = AtomicU32::new(IT_);


pub fn init() {
    crate::log!("[PROC] Initializing process manager...");
    
    
    let ubb = Process::new(
        IT_,
        IT_,
        "kernel",
        ProcessFlags(ProcessFlags::Ps)
    );
    
    {
        let mut gg = AD_.write();
        gg.ye.insert(IT_, ubb);
    }
    
    crate::log_debug!("[PROC] Kernel process created (PID 0)");
    crate::log!("[OK] Process manager ready");
}


pub fn avp(j: &str, bfb: Ah) -> Result<Ah, &'static str> {
    let mut gg = AD_.write();
    
    let ce = gg.muz();
    let mut uf = Process::new(ce, bfb, j, ProcessFlags(ProcessFlags::Cq));
    
    
    if let Some(tu) = gg.ye.get(&bfb) {
        uf.jv = tu.jv.clone();
        uf.env = tu.env.clone();
    }
    
    
    if let Some(tu) = gg.ye.ds(&bfb) {
        tu.zf.push(ce);
    }
    
    uf.g = ProcessState::At;
    gg.ye.insert(ce, uf);
    
    crate::log_debug!("[PROC] Created process {} ({})", ce, j);
    Ok(ce)
}






pub fn svr() -> Result<Ah, &'static str> {
    let cv = aei();
    
    
    let (j, jv, env, buf, bca, huf, memory, pi, pw, ahl, bqj, gvl, bai, ary, ffs, grb) = {
        let gg = AD_.read();
        let tu = gg.ye.get(&cv)
            .ok_or("Current process not found")?;
        (
            tu.j.clone(),
            tu.jv.clone(),
            tu.env.clone(),
            tu.buf.clone(),
            tu.bca,
            tu.jm,
            tu.memory.clone(),
            tu.pi,
            tu.pw,
            tu.ahl,
            tu.bqj,
            tu.gvl,
            tu.bai,
            tu.ary,
            tu.ffs,
            tu.grb.clone(),
        )
    };
    
    
    let ade = crate::memory::paging::ade();
    let (ze, jm) = if huf != ade {
        match crate::memory::cow::rbt(huf) {
            Some(atm) => {
                let r = atm.jm();
                (Some(Arc::new(Mutex::new(atm))), r)
            }
            None => {
                
                match AddressSpace::dtn() {
                    Some(atm) => {
                        let r = atm.jm();
                        (Some(Arc::new(Mutex::new(atm))), r)
                    }
                    None => (None, ade)
                }
            }
        }
    } else {
        (None, ade)
    };
    
    
    let mut gg = AD_.write();
    let ce = gg.muz();
    
    let aeh = Process {
        ce,
        bfb: cv,
        j,
        g: ProcessState::At,
        flags: ProcessFlags(ProcessFlags::Cq),
        nz: 0,
        context: CpuContext::default(),
        memory,
        buf,   
        bca,
        jv,
        env,
        cdu: 0,
        zf: Vec::new(),
        jm,
        ze,
        bai,       
        ary,        
        ffs,
        grb,
        pi,
        pw,
        ahl,
        bqj,
        gvl,
    };
    
    if let Some(tu) = gg.ye.ds(&cv) {
        tu.zf.push(ce);
    }
    gg.ye.insert(ce, aeh);
    
    
    crate::signals::lef(ce);
    
    crate::log_debug!("[PROC] COW-fork: {} -> {}", cv, ce);
    Ok(ce)
}


pub fn cxn(aj: i32) {
    let cv = aei();
    let mut gg = AD_.write();
    
    if let Some(uf) = gg.ye.ds(&cv) {
        uf.g = ProcessState::Vf;
        uf.nz = aj;
        
        
        let zf: Vec<Ah> = uf.zf.bbk(..).collect();
        for inl in zf {
            if let Some(aeh) = gg.ye.ds(&inl) {
                aeh.bfb = IS_;
            }
            if let Some(init) = gg.ye.ds(&IS_) {
                init.zf.push(inl);
            }
        }
        
        crate::log_debug!("[PROC] Process {} exited with code {}", cv, aj);
    }
}


pub fn ccm(ce: Ah) -> Result<i32, &'static str> {
    let mut gg = AD_.write();
    
    let uf = gg.ye.get(&ce).ok_or("Process not found")?;
    
    if uf.g != ProcessState::Vf {
        return Err("Process not yet exited");
    }
    
    let nz = uf.nz;
    
    
    gg.ye.remove(&ce);
    
    Ok(nz)
}


pub fn aei() -> Ah {
    APR_.load(Ordering::Relaxed)
}


pub fn jos(ce: Ah) {
    APR_.store(ce, Ordering::SeqCst);
}


pub fn dfk() -> (u32, u32, u32, u32) {
    let gg = AD_.read();
    if let Some(ai) = gg.ye.get(&aei()) {
        (ai.pi, ai.pw, ai.ahl, ai.bqj)
    } else {
        (0, 0, 0, 0) 
    }
}


pub fn pji(ce: Ah, pi: u32) -> Result<(), &'static str> {
    let mut gg = AD_.write();
    let uf = gg.ye.ds(&ce).ok_or("No such process")?;
    
    if uf.ahl == 0 || pi == uf.pi {
        uf.pi = pi;
        uf.ahl = pi;
        Ok(())
    } else {
        Err("EPERM")
    }
}


pub fn pja(ce: Ah, pw: u32) -> Result<(), &'static str> {
    let mut gg = AD_.write();
    let uf = gg.ye.ds(&ce).ok_or("No such process")?;
    if uf.ahl == 0 || pw == uf.pw {
        uf.pw = pw;
        uf.bqj = pw;
        Ok(())
    } else {
        Err("EPERM")
    }
}


pub fn wjx(ce: Ah, hs: u32) -> u32 {
    let mut gg = AD_.write();
    if let Some(uf) = gg.ye.ds(&ce) {
        let aft = uf.gvl;
        uf.gvl = hs & 0o777;
        aft
    } else {
        0o022
    }
}


pub fn get(ce: Ah) -> Option<Process> {
    AD_.read().ye.get(&ce).abn()
}


pub fn cv() -> Option<Process> {
    get(aei())
}




#[inline]
pub fn ela<Ac, G: FnOnce(&Process) -> Ac>(ce: Ah, bb: G) -> Option<Ac> {
    let gg = AD_.read();
    gg.ye.get(&ce).map(bb)
}


#[inline]
pub fn xuv<Ac, G: FnOnce(&Process) -> Ac>(bb: G) -> Option<Ac> {
    ela(aei(), bb)
}


pub fn dsi(ce: Ah) -> bool {
    AD_.read().ye.get(&ce)
        .map(|ai| ai.g == ProcessState::Ai)
        .unwrap_or(false)
}


pub fn cbr(ce: Ah, g: ProcessState) {
    if let Some(uf) = AD_.write().ye.ds(&ce) {
        uf.g = g;
    }
}


pub fn aoy() -> Vec<(Ah, String, ProcessState)> {
    AD_.read()
        .ye
        .iter()
        .map(|(ce, uf)| (*ce, uf.j.clone(), uf.g))
        .collect()
}


pub fn az() -> usize {
    AD_.read().ye.len()
}


pub fn dsm(ce: Ah) -> Result<(), &'static str> {
    if ce == IT_ || ce == IS_ {
        return Err("Cannot kill kernel or init");
    }
    
    let mut gg = AD_.write();
    
    if let Some(uf) = gg.ye.ds(&ce) {
        uf.g = ProcessState::Ez;
        crate::log_debug!("[PROC] Process {} killed", ce);
        Ok(())
    } else {
        Err("Process not found")
    }
}




pub fn eys(j: &str) -> Result<Ah, &'static str> {
    let bfb = aei();
    let ce = avp(j, bfb)?;
    
    
    crate::signals::lef(ce);
    
    crate::log!("[PROC] Spawned process {} ({}) under parent {}", ce, j, bfb);
    Ok(ce)
}


pub fn mhj(ce: Ah) {
    cbr(ce, ProcessState::Ai);
    jos(ce);
}


pub fn eqi(ce: Ah, nz: i32) {
    let mut gg = AD_.write();
    if let Some(uf) = gg.ye.ds(&ce) {
        uf.g = ProcessState::Vf;
        uf.nz = nz;
        crate::log_debug!("[PROC] Process {} exited with code {}", ce, nz);
    }
}


pub fn lyd(ce: Ah) {
    let mut gg = AD_.write();
    
    
    if let Some(uf) = gg.ye.get(&ce) {
        let zf: Vec<Ah> = uf.zf.clone();
        for inl in zf {
            if let Some(aeh) = gg.ye.ds(&inl) {
                aeh.bfb = IT_;
            }
        }
    }
    
    gg.ye.remove(&ce);
    
    
    crate::signals::khu(ce);
    
    crate::log_debug!("[PROC] Reaped process {}", ce);
}


pub fn wjf(ce: Ah, memory: MemoryLayout) {
    if let Some(uf) = AD_.write().ye.ds(&ce) {
        uf.memory = memory;
    }
}


pub fn zgm() {
    let gg = AD_.read();
    
    fn oxu(gg: &ProcessTable, ce: Ah, eo: usize) {
        if let Some(uf) = gg.ye.get(&ce) {
            let crn: String = (0..eo).map(|_| "  ").collect();
            crate::serial_println!("{}[{}] {} ({:?})", crn, ce, uf.j, uf.g);
            for aeh in &uf.zf {
                oxu(gg, *aeh, eo + 1);
            }
        }
    }
    
    crate::serial_println!("Process tree:");
    oxu(&gg, IT_, 0);
}


pub fn fwl(ce: Ah) {
    let mut gg = AD_.write();
    if let Some(uf) = gg.ye.ds(&ce) {
        uf.g = ProcessState::Vf;
        uf.nz = -9; 
    }
}


pub fn qg(ce: Ah) {
    let mut gg = AD_.write();
    if let Some(uf) = gg.ye.ds(&ce) {
        uf.g = ProcessState::Af;
    }
}


pub fn anu(ce: Ah) {
    let mut gg = AD_.write();
    if let Some(uf) = gg.ye.ds(&ce) {
        if uf.g == ProcessState::Af {
            uf.g = ProcessState::At;
        }
    }
}


pub fn lsc(ce: Ah) -> Option<Ah> {
    let gg = AD_.read();
    gg.ye.get(&ce).map(|ai| ai.bfb)
}


pub fn ghz(ce: Ah) -> Option<CpuContext> {
    let gg = AD_.read();
    gg.ye.get(&ce).map(|ai| ai.context.clone())
}


pub fn meh(ce: Ah, be: &CpuContext) -> Result<(), &'static str> {
    let mut gg = AD_.write();
    if let Some(uf) = gg.ye.ds(&ce) {
        uf.context = be.clone();
        Ok(())
    } else {
        Err("Process not found")
    }
}






pub fn wjk(ce: Ah, bai: Ah) -> Result<(), &'static str> {
    let mut gg = AD_.write();
    let ejo = if ce == 0 { aei() } else { ce };
    let utk = if bai == 0 { ejo } else { bai };
    let uf = gg.ye.ds(&ejo).ok_or("No such process")?;
    uf.bai = utk;
    Ok(())
}


pub fn nyi(ce: Ah) -> u32 {
    let gg = AD_.read();
    let cd = if ce == 0 { aei() } else { ce };
    gg.ye.get(&cd).map(|ai| ai.bai).unwrap_or(0)
}


pub fn nyo(ce: Ah) -> u32 {
    let gg = AD_.read();
    let cd = if ce == 0 { aei() } else { ce };
    gg.ye.get(&cd).map(|ai| ai.ary).unwrap_or(0)
}



pub fn wkc() -> Result<u32, &'static str> {
    let ce = aei();
    let mut gg = AD_.write();
    let uf = gg.ye.ds(&ce).ok_or("No such process")?;
    
    if uf.bai == ce {
        
    }
    uf.ary = ce;
    uf.bai = ce;
    uf.ffs = None;
    Ok(ce)
}


pub fn pit(ce: Ah, iff: u32) {
    let mut gg = AD_.write();
    if let Some(uf) = gg.ye.ds(&ce) {
        uf.ffs = Some(iff);
    }
}


pub fn ysu(ce: Ah) -> Option<u32> {
    let gg = AD_.read();
    gg.ye.get(&ce).and_then(|ai| ai.ffs)
}


pub fn raq(ce: Ah, utr: &str) -> Result<(), &'static str> {
    let mut gg = AD_.write();
    let uf = gg.ye.ds(&ce).ok_or("No such process")?;
    
    if uf.ahl != 0 {
        return Err("EPERM");
    }
    uf.grb = String::from(utr);
    Ok(())
}


pub fn ytt(ce: Ah) -> String {
    let gg = AD_.read();
    gg.ye.get(&ce).map(|ai| ai.grb.clone()).unwrap_or_else(|| String::from("/"))
}


pub fn vht(bai: u32) -> Vec<Ah> {
    let gg = AD_.read();
    gg.ye.iter()
        .hi(|(_, ai)| ai.bai == bai)
        .map(|(&ce, _)| ce)
        .collect()
}
