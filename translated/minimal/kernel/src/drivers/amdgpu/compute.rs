

































use alloc::string::String;
use alloc::format;
use alloc::vec::Vec;
use core::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use spin::Mutex;

use super::{kj, ib, Hz};
use super::regs;
use crate::memory;






const DU_: usize = 1024;
const AJC_: usize = DU_ * 4;


const IN_: usize = 64 * 1024;


const ATX_: usize = IN_ - 16;


const AUA_: u64 = 0xDEAD_BEEF_CAFE_F00D;


const CBK_: u64 = 10_000_000;







#[inline]
fn dwq(opcode: u32, count: u32) -> u32 {
    (3 << 30) | ((opcode & 0xFF) << 8) | ((count - 1) & 0x3FFF)
}


fn nvu() -> [u32; 2] {
    [dwq(regs::CNG_, 1), 0]
}



fn coh(abg: u32, value: u32) -> [u32; 3] {
    [
        dwq(regs::BFJ_, 2),
        (abg - regs::BHZ_) >> 2, 
        value,
    ]
}


fn nvw(abg: u32, val0: u32, val1: u32) -> [u32; 4] {
    [
        dwq(regs::BFJ_, 3),
        (abg - regs::BHZ_) >> 2,
        val0,
        val1,
    ]
}



fn nvt(groups_x: u32, groups_y: u32, groups_z: u32) -> [u32; 5] {
    [
        dwq(regs::CNF_, 4),
        groups_x,
        groups_y,
        groups_z,
        1, 
    ]
}



fn nvv(eml: u64, fence_value: u64) -> [u32; 7] {
    [
        dwq(regs::CNH_, 6),
        
        (0x14) | (5 << 8) | (0 << 12), 
        
        (2 << 29), 
        (eml & 0xFFFFFFFF) as u32,          
        ((eml >> 32) & 0xFFFF) as u32,      
        (fence_value & 0xFFFFFFFF) as u32,              
        ((fence_value >> 32) & 0xFFFFFFFF) as u32,      
    ]
}













































pub static BLV_: &[u32] = &[
    
    
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
















pub static BLX_: &[u32] = &[
    
    0x02020082 | (0x12 << 25),
    
    0x7E040204,
    
    0xE0702000,
    0x80020100 | (1 << 8),
    
    0xBF8C0070,
    
    0xBF810000,
];















pub static BLW_: &[u32] = &[
    
    0x02020082 | (0x12 << 25),
    
    0xE0502000,
    0x80020100 | (1 << 8),
    
    0xBF8C0070,
    
    0xE0702000,
    0x80020100 | (1 << 8) | (1 << 16), 
    
    0xBF8C0070,
    
    0xBF810000,
];













fn ket(gpu_addr: u64, ayr: u32, stride: u32) -> [u32; 4] {
    let kae = (gpu_addr & 0xFFFFFFFF) as u32;
    let kac = ((gpu_addr >> 32) & 0xFFFF) as u32;
    
    let bza = kac | ((stride & 0x3FFF) << 16);
    
    
    
    let lmu: u32 = (4 << 15) |  
                   (4 << 19) |  
                   (0 << 24) |  
                   (4 << 0)  |  
                   (5 << 3)  |  
                   (6 << 6)  |  
                   (7 << 9);    
    [kae, bza, ayr, lmu]
}






#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgentKind {
    
    Incr,
    
    MemFill,
    
    MemCopy,
}

impl AgentKind {
    pub fn name(&self) -> &'static str {
        match self {
            AgentKind::Incr => "incr",
            AgentKind::MemFill => "memfill",
            AgentKind::MemCopy => "memcopy",
        }
    }
    
    pub fn description(&self) -> &'static str {
        match self {
            AgentKind::Incr => "Increment each u32 by 1 (proof-of-life)",
            AgentKind::MemFill => "Fill buffer with constant u32 value",
            AgentKind::MemCopy => "GPU-speed buffer copy (src → dst)",
        }
    }

    
    pub fn shader_code(&self) -> &'static [u32] {
        match self {
            AgentKind::Incr => BLV_,
            AgentKind::MemFill => BLX_,
            AgentKind::MemCopy => BLW_,
        }
    }
    
    
    pub fn sgpr_count(&self) -> u32 {
        match self {
            AgentKind::Incr => 4,     
            AgentKind::MemFill => 5,  
            AgentKind::MemCopy => 8,  
        }
    }
    
    
    pub fn vgpr_count(&self) -> u32 {
        match self {
            AgentKind::Incr => 3,    
            AgentKind::MemFill => 3,
            AgentKind::MemCopy => 3,
        }
    }
    
    
    pub fn user_sgpr_count(&self) -> u32 {
        match self {
            AgentKind::Incr => 4,
            AgentKind::MemFill => 5,
            AgentKind::MemCopy => 8,
        }
    }
}


pub const RU_: &[AgentKind] = &[
    AgentKind::Incr,
    AgentKind::MemFill,
    AgentKind::MemCopy,
];


struct Oi {
    initialized: bool,
    mmio_base: u64,
    
    ring_virt: u64,
    
    ring_phys: u64,
    
    data_virt: u64,
    
    data_phys: u64,
    
    code_virt: u64,
    
    code_phys: u64,
    
    wptr: u32,
    
    dispatch_count: u64,
}

static Hh: Mutex<Oi> = Mutex::new(Oi {
    initialized: false,
    mmio_base: 0,
    ring_virt: 0,
    ring_phys: 0,
    data_virt: 0,
    data_phys: 0,
    code_virt: 0,
    code_phys: 0,
    wptr: 0,
    dispatch_count: 0,
});

static ACB_: AtomicBool = AtomicBool::new(false);







fn aow(state: &mut Oi, data: &[u32]) -> usize {
    let dq = state.ring_virt as *mut u32;
    for (i, &qx) in data.iter().enumerate() {
        let idx = (state.wptr as usize + i) % DU_;
        unsafe {
            core::ptr::write_volatile(dq.add(idx), qx);
        }
    }
    state.wptr = (state.wptr + data.len() as u32) % DU_ as u32;
    data.len()
}


fn eyu(state: &Oi) {
    unsafe {
        
        let hco = (state.wptr as u32) * 4;
        ib(state.mmio_base, regs::ARF_, hco);
        ib(state.mmio_base, regs::ARE_, 0);
    }
}







pub fn init(mmio_base: u64) {
    crate::log!("[GPU-COMPUTE] ═══════════════════════════════════════════════");
    crate::log!("[GPU-COMPUTE] Phase 3/4: Bare-metal RDNA Compute Agent");
    crate::log!("[GPU-COMPUTE] ═══════════════════════════════════════════════");
    
    if mmio_base == 0 {
        crate::log!("[GPU-COMPUTE] No MMIO base — skipping");
        return;
    }
    
    
    let eyt = match alloc::alloc::Layout::from_size_align(AJC_, 4096) {
        Ok(l) => l,
        Err(_) => { crate::log!("[GPU-COMPUTE] ERROR: invalid ring layout"); return; }
    };
    let ring_virt = unsafe { alloc::alloc::alloc_zeroed(eyt) } as u64;
    let ring_phys = memory::lc(ring_virt).unwrap_or(0);
    
    if ring_phys == 0 {
        crate::log!("[GPU-COMPUTE] ERROR: Cannot get physical address for ring buffer");
        return;
    }
    crate::log!("[GPU-COMPUTE] Ring buffer: virt={:#X} phys={:#X} size={} dwords",
        ring_virt, ring_phys, DU_);
    
    
    let lbn = match alloc::alloc::Layout::from_size_align(IN_, 4096) {
        Ok(l) => l,
        Err(_) => { crate::log!("[GPU-COMPUTE] ERROR: invalid data layout"); return; }
    };
    let data_virt = unsafe { alloc::alloc::alloc_zeroed(lbn) } as u64;
    let data_phys = memory::lc(data_virt).unwrap_or(0);
    
    if data_phys == 0 {
        crate::log!("[GPU-COMPUTE] ERROR: Cannot get physical address for data buffer");
        return;
    }
    crate::log!("[GPU-COMPUTE] Data buffer: virt={:#X} phys={:#X} size={}KB",
        data_virt, data_phys, IN_ / 1024);
    
    
    let kuo = match alloc::alloc::Layout::from_size_align(4096, 256) {
        Ok(l) => l,
        Err(_) => { crate::log!("[GPU-COMPUTE] ERROR: invalid code layout"); return; }
    };
    let code_virt = unsafe { alloc::alloc::alloc_zeroed(kuo) } as u64;
    let code_phys = memory::lc(code_virt).unwrap_or(0);
    
    if code_phys == 0 {
        crate::log!("[GPU-COMPUTE] ERROR: Cannot get physical address for code buffer");
        return;
    }
    crate::log!("[GPU-COMPUTE] Code buffer: virt={:#X} phys={:#X}", code_virt, code_phys);
    
    
    let dqz = unsafe { kj(mmio_base, regs::LB_) };
    let kyi = unsafe { kj(mmio_base, regs::NM_) };
    crate::log!("[GPU-COMPUTE] GRBM_STATUS={:#010X} CP_ME_CNTL={:#010X}", dqz, kyi);
    
    let fzq = (dqz & regs::AVV_) != 0;
    let fos = (dqz & regs::AES_) != 0;
    crate::log!("[GPU-COMPUTE] GUI_ACTIVE={} CP_BUSY={}", fzq, fos);
    
    
    crate::log!("[GPU-COMPUTE] Configuring HQD for compute queue...");
    unsafe {
        
        ib(mmio_base, regs::ARC_, 0);
        
        
        let dxi = ring_phys >> 8;
        ib(mmio_base, regs::BRS_, (dxi & 0xFFFFFFFF) as u32);
        ib(mmio_base, regs::BRR_, ((dxi >> 32) & 0xFF) as u32);
        
        
        
        let nwr = (6 << 0) | (10 << 8);
        ib(mmio_base, regs::BRT_, nwr);
        
        
        ib(mmio_base, regs::ARD_, 0);
        ib(mmio_base, regs::ARF_, 0);
        ib(mmio_base, regs::ARE_, 0);
        
        
        ib(mmio_base, regs::ARC_, 1);
    }
    
    crate::log!("[GPU-COMPUTE] HQD configured: base={:#X} size={}dw", ring_phys, DU_);
    
    
    let mut state = Hh.lock();
    state.initialized = true;
    state.mmio_base = mmio_base;
    state.ring_virt = ring_virt;
    state.ring_phys = ring_phys;
    state.data_virt = data_virt;
    state.data_phys = data_phys;
    state.code_virt = code_virt;
    state.code_phys = code_phys;
    state.wptr = 0;
    state.dispatch_count = 0;
    drop(state);
    
    ACB_.store(true, Ordering::SeqCst);
    
    
    crate::log!("[GPU-COMPUTE] ───────────────────────────────────────────────");
    crate::log!("[GPU-COMPUTE] Available agents:");
    for agent in RU_ {
        crate::log!("[GPU-COMPUTE]   {} — {}", agent.name(), agent.description());
    }
    crate::log!("[GPU-COMPUTE] ───────────────────────────────────────────────");
    crate::log!("[GPU-COMPUTE] Compute engine ready — dispatch via `gpuexec`");
}














pub fn cwq(agent: AgentKind, ayr: u32, fill_value: u32) -> Result<u64, &'static str> {
    if !ACB_.load(Ordering::Relaxed) {
        return Err("GPU compute engine not initialized");
    }
    
    let mut state = Hh.lock();
    let mmio = state.mmio_base;
    
    
    let ncv = ((IN_ - 64) / 4) as u32;
    let ayr = ayr.min(ncv);
    
    
    let cwg = state.data_virt as *mut u32;
    match agent {
        AgentKind::Incr => {
            for i in 0..ayr {
                unsafe { core::ptr::write_volatile(cwg.add(i as usize), i); }
            }
        }
        AgentKind::MemFill => {
            
            for i in 0..ayr {
                unsafe { core::ptr::write_volatile(cwg.add(i as usize), 0); }
            }
        }
        AgentKind::MemCopy => {
            
            let cw = ayr / 2;
            for i in 0..cw {
                unsafe { core::ptr::write_volatile(cwg.add(i as usize), 0xA0A0_0000 + i); }
            }
            for i in cw..ayr {
                unsafe { core::ptr::write_volatile(cwg.add(i as usize), 0); }
            }
        }
    }
    
    
    let hxz = (state.data_virt + ATX_ as u64) as *mut u64;
    unsafe { core::ptr::write_volatile(hxz, 0); }
    
    
    let shader = agent.shader_code();
    let kur = state.code_virt as *mut u32;
    for (i, &insn) in shader.iter().enumerate() {
        unsafe { core::ptr::write_volatile(kur.add(i), insn); }
    }
    
    
    let ken = ket(state.data_phys, ayr, 4);
    
    
    
    let prz = (agent.vgpr_count() + 7) / 8;
    let oqo = (agent.sgpr_count() + 7) / 8;
    let ntv = ((prz.saturating_sub(1)) & 0x3F) |
                    (((oqo.saturating_sub(1)) & 0xF) << 6) |
                    (3 << 24); 
    
    
    let ntw = agent.user_sgpr_count() & 0x1F; 
    
    
    let jgb = state.code_phys >> 8;
    
    
    
    state.wptr = 0;
    
    
    let ntu = nvw(
        regs::BRF_,
        (jgb & 0xFFFFFFFF) as u32,
        ((jgb >> 32) & 0xFFFF) as u32,
    );
    aow(&mut state, &ntu);
    
    
    let ois = coh(regs::BRG_, ntv);
    aow(&mut state, &ois);
    
    
    let oit = coh(regs::BRH_, ntw);
    aow(&mut state, &oit);
    
    
    let piw = coh(regs::BRC_, 64);
    aow(&mut state, &piw);
    let pix = coh(regs::BRD_, 1);
    aow(&mut state, &pix);
    let piy = coh(regs::BRE_, 1);
    aow(&mut state, &piy);
    
    
    for (i, &qx) in ken.iter().enumerate() {
        let reg = regs::AQV_ + (i as u32) * 4;
        let fj = coh(reg, qx);
        aow(&mut state, &fj);
    }
    
    
    if agent == AgentKind::MemFill {
        let fj = coh(regs::AQV_ + 16, fill_value);
        aow(&mut state, &fj);
    }
    
    
    
    let irl = (ayr + 63) / 64;
    let lfg = nvt(irl, 1, 1);
    aow(&mut state, &lfg);
    
    
    let eml = state.data_phys + ATX_ as u64;
    let oer = nvv(eml, AUA_);
    aow(&mut state, &oer);
    
    
    let iqu = nvu();
    aow(&mut state, &iqu);
    
    
    crate::serial_println!("[GPU-COMPUTE] Submitting {} agent: {} elements, {} workgroups",
        agent.name(), ayr, irl);
    crate::serial_println!("[GPU-COMPUTE]   Ring WPTR: {} dwords", state.wptr);
    crate::serial_println!("[GPU-COMPUTE]   Shader: {} insns at phys {:#X}", shader.len(), state.code_phys);
    
    eyu(&state);
    
    
    let mut bb = 0u64;
    loop {
        let hpq = unsafe { core::ptr::read_volatile(hxz) };
        if hpq == AUA_ {
            break;
        }
        bb += 1;
        if bb >= CBK_ {
            crate::serial_println!("[GPU-COMPUTE] TIMEOUT after {} iterations (fence={:#X})",
                bb, hpq);
            
            let ckf = unsafe { kj(mmio, regs::LB_) };
            let oim = unsafe { kj(mmio, regs::ARD_) };
            crate::serial_println!("[GPU-COMPUTE]   GRBM_STATUS={:#010X} RPTR={}", ckf, oim);
            state.dispatch_count += 1;
            return Err("GPU dispatch timed out (fence not signaled)");
        }
        
        if bb % 100 == 0 {
            core::hint::spin_loop();
        }
    }
    
    state.dispatch_count += 1;
    crate::serial_println!("[GPU-COMPUTE] Dispatch complete in {} poll iterations", bb);
    
    Ok(bb)
}






pub fn dgf(agent: AgentKind, ayr: u32, fill_value: u32) -> (u32, u32) {
    let state = Hh.lock();
    let cwg = state.data_virt as *const u32;
    let mut gd = 0u32;
    let mut gv = 0u32;
    
    let dko = ayr.min(((IN_ - 64) / 4) as u32);
    
    for i in 0..dko {
        let bxh = unsafe { core::ptr::read_volatile(cwg.add(i as usize)) };
        let expected = match agent {
            AgentKind::Incr => i + 1, 
            AgentKind::MemFill => fill_value,
            AgentKind::MemCopy => {
                let cw = dko / 2;
                if i >= cw {
                    
                    0xA0A0_0000 + (i - cw)
                } else {
                    
                    0xA0A0_0000 + i
                }
            }
        };
        if bxh == expected {
            gd += 1;
        } else {
            gv += 1;
            
            if gv <= 8 {
                crate::serial_println!("[GPU-COMPUTE] VERIFY[{}]: expected {:#010X} got {:#010X}",
                    i, expected, bxh);
            }
        }
    }
    
    (gd, gv)
}


pub fn oco(index: u32) -> Option<u32> {
    let state = Hh.lock();
    if !state.initialized {
        return None;
    }
    let max = ((IN_ - 64) / 4) as u32;
    if index >= max {
        return None;
    }
    let ptr = state.data_virt as *const u32;
    Some(unsafe { core::ptr::read_volatile(ptr.add(index as usize)) })
}






pub fn is_ready() -> bool {
    ACB_.load(Ordering::Relaxed)
}


pub fn dispatch_count() -> u64 {
    Hh.lock().dispatch_count
}


pub fn summary() -> String {
    if is_ready() {
        let state = Hh.lock();
        format!("GPU Compute: {} agents, {} dispatches, ring@{:#X}",
            RU_.len(), state.dispatch_count, state.ring_phys)
    } else {
        String::from("GPU Compute: not initialized")
    }
}


pub fn info_lines() -> Vec<String> {
    let mut lines = Vec::new();
    
    if is_ready() {
        let state = Hh.lock();
        lines.push(String::from("╔══════════════════════════════════════════════════╗"));
        lines.push(String::from("║    GPU Compute Agent — Bare-metal RDNA Dispatch  ║"));
        lines.push(String::from("╠══════════════════════════════════════════════════╣"));
        lines.push(format!("║ Ring Buffer:  {:#X} ({} dwords)          ║", state.ring_phys, DU_));
        lines.push(format!("║ Data Buffer:  {:#X} ({}KB)              ║", state.data_phys, IN_/1024));
        lines.push(format!("║ Code Buffer:  {:#X}                     ║", state.code_phys));
        lines.push(format!("║ Dispatches:   {}                                  ║", state.dispatch_count));
        lines.push(format!("║ Ring WPTR:    {}                                  ║", state.wptr));
        lines.push(String::from("╠══════════════════════════════════════════════════╣"));
        lines.push(String::from("║ Available Agents:                                ║"));
        for agent in RU_ {
            lines.push(format!("║  {:10} — {}  ║", agent.name(), agent.description()));
        }
        lines.push(String::from("╚══════════════════════════════════════════════════╝"));
    } else {
        lines.push(String::from("GPU Compute Agent not initialized"));
        lines.push(String::from("(Requires AMD GPU with MMIO access)"));
    }
    
    lines
}
