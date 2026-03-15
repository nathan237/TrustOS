

































use alloc::string::String;
use alloc::format;
use alloc::vec::Vec;
use core::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use spin::Mutex;

use super::{wr, sk, Sr};
use super::regs;
use crate::memory;






const DM_: usize = 1024;
const AHG_: usize = DM_ * 4;


const HT_: usize = 64 * 1024;


const ARV_: usize = HT_ - 16;


const ARY_: u64 = 0xDEAD_BEEF_CAFE_F00D;


const BYE_: u64 = 10_000_000;







#[inline]
fn hvl(opcode: u32, az: u32) -> u32 {
    (3 << 30) | ((opcode & 0xFF) << 8) | ((az - 1) & 0x3FFF)
}


fn vjk() -> [u32; 2] {
    [hvl(regs::CJX_, 1), 0]
}



fn frb(ban: u32, bn: u32) -> [u32; 3] {
    [
        hvl(regs::BDG_, 2),
        (ban - regs::BFV_) >> 2, 
        bn,
    ]
}


fn vjm(ban: u32, xqh: u32, xqi: u32) -> [u32; 4] {
    [
        hvl(regs::BDG_, 3),
        (ban - regs::BFV_) >> 2,
        xqh,
        xqi,
    ]
}



fn vjj(thq: u32, thr: u32, ths: u32) -> [u32; 5] {
    [
        hvl(regs::CJW_, 4),
        thq,
        thr,
        ths,
        1, 
    ]
}



fn vjl(iuf: u64, ntd: u64) -> [u32; 7] {
    [
        hvl(regs::CJY_, 6),
        
        (0x14) | (5 << 8) | (0 << 12), 
        
        (2 << 29), 
        (iuf & 0xFFFFFFFF) as u32,          
        ((iuf >> 32) & 0xFFFF) as u32,      
        (ntd & 0xFFFFFFFF) as u32,              
        ((ntd >> 32) & 0xFFFFFFFF) as u32,      
    ]
}













































pub static BJL_: &[u32] = &[
    
    
    0x7E020284,
    
    
    
    
    
    0x02020082 | (0x12 << 25),  
    
    
    
    0xE0502000, 
    0x80020100 | (1 << 8), 
    
    
    0xBF8C0070,
    
    
    
    0x02040081 | (0x25 << 25), 
    
    0xE0702000, 
    0x80020100 | (1 << 8), 
    
    0xBF8C0070,
    
    
    0xBF810000,
];
















pub static BJN_: &[u32] = &[
    
    0x02020082 | (0x12 << 25),
    
    0x7E040204,
    
    0xE0702000,
    0x80020100 | (1 << 8),
    
    0xBF8C0070,
    
    0xBF810000,
];















pub static BJM_: &[u32] = &[
    
    0x02020082 | (0x12 << 25),
    
    0xE0502000,
    0x80020100 | (1 << 8),
    
    0xBF8C0070,
    
    0xE0702000,
    0x80020100 | (1 << 8) | (1 << 16), 
    
    0xBF8C0070,
    
    0xBF810000,
];













fn qta(nzl: u64, csx: u32, oq: u32) -> [u32; 4] {
    let qnn = (nzl & 0xFFFFFFFF) as u32;
    let qnl = ((nzl >> 32) & 0xFFFF) as u32;
    
    let epl = qnl | ((oq & 0x3FFF) << 16);
    
    
    
    let shp: u32 = (4 << 15) |  
                   (4 << 19) |  
                   (0 << 24) |  
                   (4 << 0)  |  
                   (5 << 3)  |  
                   (6 << 6)  |  
                   (7 << 9);    
    [qnn, epl, csx, shp]
}






#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgentKind {
    
    It,
    
    Hv,
    
    Iw,
}

impl AgentKind {
    pub fn j(&self) -> &'static str {
        match self {
            AgentKind::It => "incr",
            AgentKind::Hv => "memfill",
            AgentKind::Iw => "memcopy",
        }
    }
    
    pub fn dc(&self) -> &'static str {
        match self {
            AgentKind::It => "Increment each u32 by 1 (proof-of-life)",
            AgentKind::Hv => "Fill buffer with constant u32 value",
            AgentKind::Iw => "GPU-speed buffer copy (src → dst)",
        }
    }

    
    pub fn fun(&self) -> &'static [u32] {
        match self {
            AgentKind::It => BJL_,
            AgentKind::Hv => BJN_,
            AgentKind::Iw => BJM_,
        }
    }
    
    
    pub fn jpo(&self) -> u32 {
        match self {
            AgentKind::It => 4,     
            AgentKind::Hv => 5,  
            AgentKind::Iw => 8,  
        }
    }
    
    
    pub fn jvl(&self) -> u32 {
        match self {
            AgentKind::It => 3,    
            AgentKind::Hv => 3,
            AgentKind::Iw => 3,
        }
    }
    
    
    pub fn xpw(&self) -> u32 {
        match self {
            AgentKind::It => 4,
            AgentKind::Hv => 5,
            AgentKind::Iw => 8,
        }
    }
}


pub const QZ_: &[AgentKind] = &[
    AgentKind::It,
    AgentKind::Hv,
    AgentKind::Iw,
];


struct Ahc {
    jr: bool,
    hv: u64,
    
    dlh: u64,
    
    bhy: u64,
    
    dpq: u64,
    
    cpu: u64,
    
    ffi: u64,
    
    asn: u64,
    
    ccn: u32,
    
    eox: u64,
}

static Rr: Mutex<Ahc> = Mutex::new(Ahc {
    jr: false,
    hv: 0,
    dlh: 0,
    bhy: 0,
    dpq: 0,
    cpu: 0,
    ffi: 0,
    asn: 0,
    ccn: 0,
    eox: 0,
});

static AAO_: AtomicBool = AtomicBool::new(false);







fn cbk(g: &mut Ahc, f: &[u32]) -> usize {
    let mz = g.dlh as *mut u32;
    for (a, &aix) in f.iter().cf() {
        let w = (g.ccn as usize + a) % DM_;
        unsafe {
            core::ptr::write_volatile(mz.add(w), aix);
        }
    }
    g.ccn = (g.ccn + f.len() as u32) % DM_ as u32;
    f.len()
}


fn jmn(g: &Ahc) {
    unsafe {
        
        let mqx = (g.ccn as u32) * 4;
        sk(g.hv, regs::APF_, mqx);
        sk(g.hv, regs::APE_, 0);
    }
}







pub fn init(hv: u64) {
    crate::log!("[GPU-COMPUTE] ═══════════════════════════════════════════════");
    crate::log!("[GPU-COMPUTE] Phase 3/4: Bare-metal RDNA Compute Agent");
    crate::log!("[GPU-COMPUTE] ═══════════════════════════════════════════════");
    
    if hv == 0 {
        crate::log!("[GPU-COMPUTE] No MMIO base — skipping");
        return;
    }
    
    
    let jmm = alloc::alloc::Layout::bjy(AHG_, 4096)
        .expect("ring layout");
    let dlh = unsafe { alloc::alloc::alloc_zeroed(jmm) } as u64;
    let bhy = memory::abw(dlh).unwrap_or(0);
    
    if bhy == 0 {
        crate::log!("[GPU-COMPUTE] ERROR: Cannot get physical address for ring buffer");
        return;
    }
    crate::log!("[GPU-COMPUTE] Ring buffer: virt={:#X} phys={:#X} size={} dwords",
        dlh, bhy, DM_);
    
    
    let rtl = alloc::alloc::Layout::bjy(HT_, 4096)
        .expect("data layout");
    let dpq = unsafe { alloc::alloc::alloc_zeroed(rtl) } as u64;
    let cpu = memory::abw(dpq).unwrap_or(0);
    
    if cpu == 0 {
        crate::log!("[GPU-COMPUTE] ERROR: Cannot get physical address for data buffer");
        return;
    }
    crate::log!("[GPU-COMPUTE] Data buffer: virt={:#X} phys={:#X} size={}KB",
        dpq, cpu, HT_ / 1024);
    
    
    let rld = alloc::alloc::Layout::bjy(4096, 256)
        .expect("code layout");
    let ffi = unsafe { alloc::alloc::alloc_zeroed(rld) } as u64;
    let asn = memory::abw(ffi).unwrap_or(0);
    
    if asn == 0 {
        crate::log!("[GPU-COMPUTE] ERROR: Cannot get physical address for code buffer");
        return;
    }
    crate::log!("[GPU-COMPUTE] Code buffer: virt={:#X} phys={:#X}", ffi, asn);
    
    
    let hlx = unsafe { wr(hv, regs::KI_) };
    let rpk = unsafe { wr(hv, regs::MN_) };
    crate::log!("[GPU-COMPUTE] GRBM_STATUS={:#010X} CP_ME_CNTL={:#010X}", hlx, rpk);
    
    let laq = (hlx & regs::ATR_) != 0;
    let kky = (hlx & regs::ADC_) != 0;
    crate::log!("[GPU-COMPUTE] GUI_ACTIVE={} CP_BUSY={}", laq, kky);
    
    
    crate::log!("[GPU-COMPUTE] Configuring HQD for compute queue...");
    unsafe {
        
        sk(hv, regs::APC_, 0);
        
        
        let hwt = bhy >> 8;
        sk(hv, regs::BPB_, (hwt & 0xFFFFFFFF) as u32);
        sk(hv, regs::BPA_, ((hwt >> 32) & 0xFF) as u32);
        
        
        
        let vkn = (6 << 0) | (10 << 8);
        sk(hv, regs::BPC_, vkn);
        
        
        sk(hv, regs::APD_, 0);
        sk(hv, regs::APF_, 0);
        sk(hv, regs::APE_, 0);
        
        
        sk(hv, regs::APC_, 1);
    }
    
    crate::log!("[GPU-COMPUTE] HQD configured: base={:#X} size={}dw", bhy, DM_);
    
    
    let mut g = Rr.lock();
    g.jr = true;
    g.hv = hv;
    g.dlh = dlh;
    g.bhy = bhy;
    g.dpq = dpq;
    g.cpu = cpu;
    g.ffi = ffi;
    g.asn = asn;
    g.ccn = 0;
    g.eox = 0;
    drop(g);
    
    AAO_.store(true, Ordering::SeqCst);
    
    
    crate::log!("[GPU-COMPUTE] ───────────────────────────────────────────────");
    crate::log!("[GPU-COMPUTE] Available agents:");
    for agent in QZ_ {
        crate::log!("[GPU-COMPUTE]   {} — {}", agent.j(), agent.dc());
    }
    crate::log!("[GPU-COMPUTE] ───────────────────────────────────────────────");
    crate::log!("[GPU-COMPUTE] Compute engine ready — dispatch via `gpuexec`");
}














pub fn gey(agent: AgentKind, csx: u32, eqg: u32) -> Result<u64, &'static str> {
    if !AAO_.load(Ordering::Relaxed) {
        return Err("GPU compute engine not initialized");
    }
    
    let mut g = Rr.lock();
    let mmio = g.hv;
    
    
    let ulc = ((HT_ - 64) / 4) as u32;
    let csx = csx.v(ulc);
    
    
    let gee = g.dpq as *mut u32;
    match agent {
        AgentKind::It => {
            for a in 0..csx {
                unsafe { core::ptr::write_volatile(gee.add(a as usize), a); }
            }
        }
        AgentKind::Hv => {
            
            for a in 0..csx {
                unsafe { core::ptr::write_volatile(gee.add(a as usize), 0); }
            }
        }
        AgentKind::Iw => {
            
            let iv = csx / 2;
            for a in 0..iv {
                unsafe { core::ptr::write_volatile(gee.add(a as usize), 0xA0A0_0000 + a); }
            }
            for a in iv..csx {
                unsafe { core::ptr::write_volatile(gee.add(a as usize), 0); }
            }
        }
    }
    
    
    let ntc = (g.dpq + ARV_ as u64) as *mut u64;
    unsafe { core::ptr::write_volatile(ntc, 0); }
    
    
    let bfg = agent.fun();
    let rlg = g.ffi as *mut u32;
    for (a, &tvf) in bfg.iter().cf() {
        unsafe { core::ptr::write_volatile(rlg.add(a), tvf); }
    }
    
    
    let qsp = qta(g.cpu, csx, 4);
    
    
    
    let xro = (agent.jvl() + 7) / 8;
    let wln = (agent.jpo() + 7) / 8;
    let vgz = ((xro.ao(1)) & 0x3F) |
                    (((wln.ao(1)) & 0xF) << 6) |
                    (3 << 24); 
    
    
    let vha = agent.xpw() & 0x1F; 
    
    
    let pkc = g.asn >> 8;
    
    
    
    g.ccn = 0;
    
    
    let vgy = vjm(
        regs::BOO_,
        (pkc & 0xFFFFFFFF) as u32,
        ((pkc >> 32) & 0xFFFF) as u32,
    );
    cbk(&mut g, &vgy);
    
    
    let wax = frb(regs::BOP_, vgz);
    cbk(&mut g, &wax);
    
    
    let way = frb(regs::BOQ_, vha);
    cbk(&mut g, &way);
    
    
    let xgk = frb(regs::BOL_, 64);
    cbk(&mut g, &xgk);
    let xgl = frb(regs::BOM_, 1);
    cbk(&mut g, &xgl);
    let xgm = frb(regs::BON_, 1);
    cbk(&mut g, &xgm);
    
    
    for (a, &aix) in qsp.iter().cf() {
        let reg = regs::AOV_ + (a as u32) * 4;
        let mt = frb(reg, aix);
        cbk(&mut g, &mt);
    }
    
    
    if agent == AgentKind::Hv {
        let mt = frb(regs::AOV_ + 16, eqg);
        cbk(&mut g, &mt);
    }
    
    
    
    let orq = (csx + 63) / 64;
    let ryk = vjj(orq, 1, 1);
    cbk(&mut g, &ryk);
    
    
    let iuf = g.cpu + ARV_ as u64;
    let vun = vjl(iuf, ARY_);
    cbk(&mut g, &vun);
    
    
    let oqw = vjk();
    cbk(&mut g, &oqw);
    
    
    crate::serial_println!("[GPU-COMPUTE] Submitting {} agent: {} elements, {} workgroups",
        agent.j(), csx, orq);
    crate::serial_println!("[GPU-COMPUTE]   Ring WPTR: {} dwords", g.ccn);
    crate::serial_println!("[GPU-COMPUTE]   Shader: {} insns at phys {:#X}", bfg.len(), g.asn);
    
    jmn(&g);
    
    
    let mut ez = 0u64;
    loop {
        let nig = unsafe { core::ptr::read_volatile(ntc) };
        if nig == ARY_ {
            break;
        }
        ez += 1;
        if ez >= BYE_ {
            crate::serial_println!("[GPU-COMPUTE] TIMEOUT after {} iterations (fence={:#X})",
                ez, nig);
            
            let fjx = unsafe { wr(mmio, regs::KI_) };
            let waq = unsafe { wr(mmio, regs::APD_) };
            crate::serial_println!("[GPU-COMPUTE]   GRBM_STATUS={:#010X} RPTR={}", fjx, waq);
            g.eox += 1;
            return Err("GPU dispatch timed out (fence not signaled)");
        }
        
        if ez % 100 == 0 {
            core::hint::hc();
        }
    }
    
    g.eox += 1;
    crate::serial_println!("[GPU-COMPUTE] Dispatch complete in {} poll iterations", ez);
    
    Ok(ez)
}






pub fn gwa(agent: AgentKind, csx: u32, eqg: u32) -> (u32, u32) {
    let g = Rr.lock();
    let gee = g.dpq as *const u32;
    let mut afu = 0u32;
    let mut ace = 0u32;
    
    let ncl = csx.v(((HT_ - 64) / 4) as u32);
    
    for a in 0..ncl {
        let elw = unsafe { core::ptr::read_volatile(gee.add(a as usize)) };
        let qy = match agent {
            AgentKind::It => a + 1, 
            AgentKind::Hv => eqg,
            AgentKind::Iw => {
                let iv = ncl / 2;
                if a >= iv {
                    
                    0xA0A0_0000 + (a - iv)
                } else {
                    
                    0xA0A0_0000 + a
                }
            }
        };
        if elw == qy {
            afu += 1;
        } else {
            ace += 1;
            
            if ace <= 8 {
                crate::serial_println!("[GPU-COMPUTE] VERIFY[{}]: expected {:#010X} got {:#010X}",
                    a, qy, elw);
            }
        }
    }
    
    (afu, ace)
}


pub fn vrn(index: u32) -> Option<u32> {
    let g = Rr.lock();
    if !g.jr {
        return None;
    }
    let am = ((HT_ - 64) / 4) as u32;
    if index >= am {
        return None;
    }
    let ptr = g.dpq as *const u32;
    Some(unsafe { core::ptr::read_volatile(ptr.add(index as usize)) })
}






pub fn uc() -> bool {
    AAO_.load(Ordering::Relaxed)
}


pub fn eox() -> u64 {
    Rr.lock().eox
}


pub fn awz() -> String {
    if uc() {
        let g = Rr.lock();
        format!("GPU Compute: {} agents, {} dispatches, ring@{:#X}",
            QZ_.len(), g.eox, g.bhy)
    } else {
        String::from("GPU Compute: not initialized")
    }
}


pub fn zl() -> Vec<String> {
    let mut ak = Vec::new();
    
    if uc() {
        let g = Rr.lock();
        ak.push(String::from("╔══════════════════════════════════════════════════╗"));
        ak.push(String::from("║    GPU Compute Agent — Bare-metal RDNA Dispatch  ║"));
        ak.push(String::from("╠══════════════════════════════════════════════════╣"));
        ak.push(format!("║ Ring Buffer:  {:#X} ({} dwords)          ║", g.bhy, DM_));
        ak.push(format!("║ Data Buffer:  {:#X} ({}KB)              ║", g.cpu, HT_/1024));
        ak.push(format!("║ Code Buffer:  {:#X}                     ║", g.asn));
        ak.push(format!("║ Dispatches:   {}                                  ║", g.eox));
        ak.push(format!("║ Ring WPTR:    {}                                  ║", g.ccn));
        ak.push(String::from("╠══════════════════════════════════════════════════╣"));
        ak.push(String::from("║ Available Agents:                                ║"));
        for agent in QZ_ {
            ak.push(format!("║  {:10} — {}  ║", agent.j(), agent.dc()));
        }
        ak.push(String::from("╚══════════════════════════════════════════════════╝"));
    } else {
        ak.push(String::from("GPU Compute Agent not initialized"));
        ak.push(String::from("(Requires AMD GPU with MMIO access)"));
    }
    
    ak
}
