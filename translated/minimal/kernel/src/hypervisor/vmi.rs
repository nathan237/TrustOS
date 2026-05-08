












use alloc::string::String;
use alloc::vec::Vec;
use alloc::format;
use core::sync::atomic::{AtomicBool, AtomicU64, Ordering};






static ALL_: AtomicBool = AtomicBool::new(false);


static CXG_: AtomicU64 = AtomicU64::new(0);


pub fn enable() {
    ALL_.store(true, Ordering::SeqCst);
    crate::serial_println!("[VMI] Introspection engine enabled");
    crate::lab_mode::trace_bus::bzh(0, "VMI engine ENABLED");
}


pub fn bbc() {
    ALL_.store(false, Ordering::SeqCst);
    crate::serial_println!("[VMI] Introspection engine disabled");
    crate::lab_mode::trace_bus::bzh(0, "VMI engine DISABLED");
}


pub fn lq() -> bool {
    ALL_.load(Ordering::Relaxed)
}






pub fn exv(vm_id: u64, gm: u64, len: usize) -> Option<Vec<u8>> {
    
    if let Some(data) = ocq(vm_id, gm, len) {
        return Some(data);
    }
    ocr(vm_id, gm, len)
}


fn ocq(vm_id: u64, gm: u64, len: usize) -> Option<Vec<u8>> {
    super::svm_vm::avv(vm_id, |vm| {
        vm.read_guest_memory(gm, len).map(|j| j.to_vec())
    }).flatten()
}


fn ocr(_vm_id: u64, _gpa: u64, _len: usize) -> Option<Vec<u8>> {
    
    
    None
}


pub fn exw(vm_id: u64, gm: u64) -> Option<u64> {
    let data = exv(vm_id, gm, 8)?;
    if data.len() < 8 { return None; }
    Some(u64::from_le_bytes([
        data[0], data[1], data[2], data[3],
        data[4], data[5], data[6], data[7],
    ]))
}


pub fn qsg(vm_id: u64, gm: u64) -> Option<u32> {
    let data = exv(vm_id, gm, 4)?;
    if data.len() < 4 { return None; }
    Some(u32::from_le_bytes([data[0], data[1], data[2], data[3]]))
}


pub fn qsf(vm_id: u64, gm: u64, aoo: usize) -> Option<String> {
    let max = if aoo > 256 { 256 } else { aoo };
    let data = exv(vm_id, gm, max)?;
    let end = data.iter().position(|&b| b == 0).unwrap_or(data.len());
    String::from_utf8(data[..end].to_vec()).ok()
}









pub fn mgo(vm_id: u64, guest_cr3: u64, vaddr: u64) -> Option<u64> {
    
    let nvx = guest_cr3 & 0x000F_FFFF_FFFF_F000;
    
    let lu = ((vaddr >> 39) & 0x1FF) as u64;
    let jc = ((vaddr >> 30) & 0x1FF) as u64;
    let iw   = ((vaddr >> 21) & 0x1FF) as u64;
    let mw   = ((vaddr >> 12) & 0x1FF) as u64;
    let offset   = vaddr & 0xFFF;
    
    
    let ivk = exw(vm_id, nvx + lu * 8)?;
    if ivk & 1 == 0 { return None; } 
    
    
    let nti = ivk & 0x000F_FFFF_FFFF_F000;
    let ewl = exw(vm_id, nti + jc * 8)?;
    if ewl & 1 == 0 { return None; }
    
    
    if ewl & (1 << 7) != 0 {
        let phys = (ewl & 0x000F_FFFF_C000_0000) | (vaddr & 0x3FFF_FFFF);
        return Some(phys);
    }
    
    
    let ntb = ewl & 0x000F_FFFF_FFFF_F000;
    let dcm = exw(vm_id, ntb + iw * 8)?;
    if dcm & 1 == 0 { return None; }
    
    
    if dcm & (1 << 7) != 0 {
        let phys = (dcm & 0x000F_FFFF_FFE0_0000) | (vaddr & 0x1F_FFFF);
        return Some(phys);
    }
    
    
    let cok = dcm & 0x000F_FFFF_FFFF_F000;
    let iwz = exw(vm_id, cok + mw * 8)?;
    if iwz & 1 == 0 { return None; }
    
    let phys = (iwz & 0x000F_FFFF_FFFF_F000) | offset;
    Some(phys)
}


pub fn cdd(vm_id: u64, guest_cr3: u64, vaddr: u64, len: usize) -> Option<Vec<u8>> {
    
    let mut result = Vec::with_capacity(len);
    let mut ck = len;
    let mut fpm = vaddr;
    
    while ck > 0 {
        let gm = mgo(vm_id, guest_cr3, fpm)?;
        let glv = (fpm & 0xFFF) as usize;
        let df = core::cmp::min(ck, 4096 - glv);
        
        let data = exv(vm_id, gm, df)?;
        result.extend_from_slice(&data);
        
        ck -= df;
        fpm += df as u64;
    }
    
    Some(result)
}






#[derive(Debug, Clone, Default)]
pub struct RegisterSnapshot {
    
    pub rax: u64,
    pub rbx: u64,
    pub rcx: u64,
    pub rdx: u64,
    pub rsi: u64,
    pub rdi: u64,
    pub rbp: u64,
    pub rsp: u64,
    pub r8:  u64,
    pub r9:  u64,
    pub r10: u64,
    pub r11: u64,
    pub r12: u64,
    pub r13: u64,
    pub r14: u64,
    pub r15: u64,
    
    pub rip: u64,
    pub rflags: u64,
    
    pub cr0: u64,
    pub cr3: u64,
    pub cr4: u64,
    
    pub cs: u16,
    pub ds: u16,
    pub ss: u16,
    pub es: u16,
}


pub fn ouj(vm_id: u64) -> Option<RegisterSnapshot> {
    super::svm_vm::avv(vm_id, |vm| {
        let mut jp = RegisterSnapshot::default();
        
        
        let regs = &vm.guest_regs;
        jp.rax = regs.rax;
        jp.rbx = regs.rbx;
        jp.rcx = regs.rcx;
        jp.rdx = regs.rdx;
        jp.rsi = regs.rsi;
        jp.rdi = regs.rdi;
        jp.rbp = regs.rbp;
        jp.r8  = regs.r8;
        jp.r9  = regs.r9;
        jp.r10 = regs.r10;
        jp.r11 = regs.r11;
        jp.r12 = regs.r12;
        jp.r13 = regs.r13;
        jp.r14 = regs.r14;
        jp.r15 = regs.r15;
        
        
        if let Some(ref vmcb) = vm.vmcb {
            use super::svm::vmcb::state_offsets;
            jp.rip = vmcb.read_state(state_offsets::Af);
            jp.rsp = vmcb.read_state(state_offsets::De);
            jp.rflags = vmcb.read_state(state_offsets::Ek);
            jp.cr0 = vmcb.read_state(state_offsets::Jn);
            jp.cr3 = vmcb.read_state(state_offsets::Jo);
            jp.cr4 = vmcb.read_state(state_offsets::Jp);
            jp.cs = vmcb.read_state(state_offsets::KO_) as u16;
            jp.ds = vmcb.read_state(state_offsets::NT_) as u16;
            jp.ss = vmcb.read_state(state_offsets::YO_) as u16;
            jp.es = vmcb.read_state(state_offsets::UF_) as u16;
        }
        
        jp
    })
}


pub fn oui(vm_id: u64) -> Option<RegisterSnapshot> {
    CXG_.fetch_add(1, Ordering::Relaxed);
    
    
    if let Some(jp) = ouj(vm_id) {
        crate::lab_mode::trace_bus::hvm(
            vm_id,
            jp.rip, jp.rsp, jp.rax, jp.rbx, jp.rcx, jp.rdx,
        );
        return Some(jp);
    }
    
    
    None
}






#[derive(Debug, Clone)]
pub struct St {
    
    pub pid: u32,
    
    pub comm: String,
    
    pub state: u8,
    
    pub ppid: u32,
    
    pub task_addr: u64,
    
    pub jqj: u64,
}



#[derive(Debug, Clone, Copy)]
pub struct LinuxOffsets {
    
    pub tasks_next: usize,
    
    pub pid: usize,
    
    pub comm: usize,
    
    pub state: usize,
    
    pub parent: usize,
    
    pub mm: usize,
    
    pub mm_total_vm: usize,
    
    pub init_task_addr: u64,
}

impl LinuxOffsets {
    
    pub fn myw() -> Self {
        LinuxOffsets {
            tasks_next: 0x498,    
            pid: 0x560,           
            comm: 0x6F0,          
            state: 0x00,          
            parent: 0x568,        
            mm: 0x478,            
            mm_total_vm: 0x80,    
            init_task_addr: 0,    
        }
    }
    
    
    pub fn myv() -> Self {
        LinuxOffsets {
            tasks_next: 0x3F0,
            pid: 0x4C8,
            comm: 0x670,
            state: 0x00,
            parent: 0x4D0,
            mm: 0x458,
            mm_total_vm: 0x80,
            init_task_addr: 0,
        }
    }
}





pub fn hwi(
    vm_id: u64,
    guest_cr3: u64,
    agv: &LinuxOffsets,
) -> Vec<St> {
    let mut processes = Vec::new();
    let ndh = 512; 
    
    if agv.init_task_addr == 0 {
        return processes;
    }
    
    let igv = agv.init_task_addr;
    let mut current = igv;
    
    for _ in 0..ndh {
        
        let pid = match cdd(vm_id, guest_cr3, current + agv.pid as u64, 4) {
            Some(data) if data.len() >= 4 => {
                u32::from_le_bytes([data[0], data[1], data[2], data[3]])
            }
            _ => break,
        };
        
        
        let comm = cdd(vm_id, guest_cr3, current + agv.comm as u64, 16)
            .and_then(|data| {
                let end = data.iter().position(|&b| b == 0).unwrap_or(data.len());
                String::from_utf8(data[..end].to_vec()).ok()
            })
            .unwrap_or_else(|| String::from("?"));
        
        
        let state = cdd(vm_id, guest_cr3, current + agv.state as u64, 1)
            .map(|d| d[0])
            .unwrap_or(0);
        
        
        let ppid = cdd(vm_id, guest_cr3, current + agv.parent as u64, 8)
            .and_then(|data| {
                if data.len() < 8 { return None; }
                let gmd = u64::from_le_bytes([
                    data[0], data[1], data[2], data[3],
                    data[4], data[5], data[6], data[7],
                ]);
                
                cdd(vm_id, guest_cr3, gmd + agv.pid as u64, 4)
                    .map(|d| u32::from_le_bytes([d[0], d[1], d[2], d[3]]))
            })
            .unwrap_or(0);
        
        
        let jqj = cdd(vm_id, guest_cr3, current + agv.mm as u64, 8)
            .and_then(|data| {
                if data.len() < 8 { return None; }
                let iny = u64::from_le_bytes([
                    data[0], data[1], data[2], data[3],
                    data[4], data[5], data[6], data[7],
                ]);
                if iny == 0 { return Some(0u64); } 
                cdd(vm_id, guest_cr3, iny + agv.mm_total_vm as u64, 8)
                    .map(|d| u64::from_le_bytes([
                        d[0], d[1], d[2], d[3], d[4], d[5], d[6], d[7],
                    ]))
            })
            .unwrap_or(0);
        
        processes.push(St {
            pid,
            comm,
            state,
            ppid,
            task_addr: current,
            jqj,
        });
        
        
        let nke = match cdd(
            vm_id, guest_cr3,
            current + agv.tasks_next as u64, 8
        ) {
            Some(data) if data.len() >= 8 => {
                u64::from_le_bytes([
                    data[0], data[1], data[2], data[3],
                    data[4], data[5], data[6], data[7],
                ])
            }
            _ => break,
        };
        
        
        current = nke.wrapping_sub(agv.tasks_next as u64);
        
        
        if current == igv {
            break;
        }
    }
    
    processes
}






#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MemoryRegionType {
    
    Ram,
    
    Mmio,
    
    Rom,
    
    Unmapped,
    
    Reserved,
    
    AcpiReclaimable,
}


#[derive(Debug, Clone)]
pub struct Fq {
    pub base: u64,
    pub size: u64,
    pub region_type: MemoryRegionType,
    pub label: &'static str,
}


pub fn fkc(memory_mb: usize) -> Vec<Fq> {
    let eug = (memory_mb * 1024 * 1024) as u64;
    let mut regions = Vec::new();
    
    
    
    regions.push(Fq {
        base: 0,
        size: 0xA_0000,
        region_type: MemoryRegionType::Ram,
        label: "Conventional",
    });
    
    
    regions.push(Fq {
        base: 0xA_0000,
        size: 0x6_0000,
        region_type: MemoryRegionType::Mmio,
        label: "VGA+ROM",
    });
    
    
    let ltl = if eug > 0x1_0000_0000 {
        
        0xC000_0000u64 
    } else {
        core::cmp::min(eug, 0xC000_0000)
    };
    
    regions.push(Fq {
        base: 0x10_0000,
        size: ltl - 0x10_0000,
        region_type: MemoryRegionType::Ram,
        label: "Extended",
    });
    
    
    regions.push(Fq {
        base: 0xC000_0000,
        size: 0x4000_0000,
        region_type: MemoryRegionType::Mmio,
        label: "PCI MMIO",
    });
    
    
    if eug > 0x1_0000_0000 {
        let jtb = eug - 0xC000_0000; 
        regions.push(Fq {
            base: 0x1_0000_0000,
            size: jtb,
            region_type: MemoryRegionType::Ram,
            label: "High Memory",
        });
    }
    
    
    regions.push(Fq {
        base: 0xFEC0_0000,
        size: 0x1000,
        region_type: MemoryRegionType::Mmio,
        label: "IO-APIC",
    });
    
    regions.push(Fq {
        base: 0xFEE0_0000,
        size: 0x1000,
        region_type: MemoryRegionType::Mmio,
        label: "Local APIC",
    });
    
    regions
}






#[derive(Debug, Clone)]
pub struct Afe {
    
    pub filter: Vec<u64>,
    
    pub active: bool,
    
    pub captured: Vec<Rj>,
    
    pub max_entries: usize,
}


#[derive(Debug, Clone)]
pub struct Rj {
    pub vm_id: u64,
    pub syscall_nr: u64,
    pub arg0: u64,
    pub arg1: u64,
    pub arg2: u64,
    pub rip: u64,
    pub timestamp: u64,
}

impl Afe {
    pub fn new() -> Self {
        Afe {
            filter: Vec::new(),
            active: false,
            captured: Vec::new(),
            max_entries: 1024,
        }
    }
    
    
    pub fn record(&mut self, vm_id: u64, nr: u64, abn: u64, eb: u64, fy: u64, rip: u64) {
        if !self.active { return; }
        
        
        if !self.filter.is_empty() && !self.filter.contains(&nr) {
            return;
        }
        
        if self.captured.len() >= self.max_entries {
            self.captured.remove(0); 
        }
        
        self.captured.push(Rj {
            vm_id,
            syscall_nr: nr,
            arg0: abn,
            arg1: eb,
            arg2: fy,
            rip,
            timestamp: crate::time::uptime_ms(),
        });
        
        
        crate::lab_mode::trace_bus::bzg(
            vm_id,
            "SYSCALL",
            rip,
            &format!("nr={} a0=0x{:X} a1=0x{:X}", nr, abn, eb),
        );
    }
    
    
    pub fn cpd(&self, count: usize) -> &[Rj] {
        let start = self.captured.len().saturating_sub(count);
        &self.captured[start..]
    }
    
    
    pub fn clear(&mut self) {
        self.captured.clear();
    }
}






#[derive(Debug, Clone)]
pub struct Agf {
    pub vm_id: u64,
    pub fep: String,
    pub regs: Option<RegisterSnapshot>,
    pub processes: Vec<St>,
    pub memory_map: Vec<Fq>,
    pub memory_mb: usize,
    pub state: &'static str,
}


pub fn qyt(vm_id: u64) -> Option<Agf> {
    if !lq() { return None; }
    
    
    let aen = super::svm_vm::dtn();
    let (name, acr, memory_mb) = {
        let nj = aen.iter().find(|(id, _, _)| *id == vm_id)?;
        let state = match nj.2 {
            super::svm_vm::SvmVmState::Created => "created",
            super::svm_vm::SvmVmState::Running => "running",
            super::svm_vm::SvmVmState::Stopped => "stopped",
            super::svm_vm::SvmVmState::Paused => "paused",
            _ => "unknown",
        };
        
        (nj.1.clone(), state, 0usize) 
    };
    
    
    let regs = oui(vm_id);
    
    
    let processes = if let Some(ref r) = regs {
        if r.cr3 != 0 {
            
            let nmk = LinuxOffsets::myw();
            let exb = hwi(vm_id, r.cr3, &nmk);
            if exb.is_empty() {
                let nmj = LinuxOffsets::myv();
                hwi(vm_id, r.cr3, &nmj)
            } else {
                exb
            }
        } else {
            Vec::new()
        }
    } else {
        Vec::new()
    };
    
    
    let memory_map = fkc(if memory_mb > 0 { memory_mb } else { 64 });
    
    Some(Agf {
        vm_id,
        fep: name,
        regs,
        processes,
        memory_map,
        memory_mb,
        state: acr,
    })
}


pub fn ikn() -> Vec<(u64, String, &'static str)> {
    let mut result = Vec::new();
    
    
    for (id, name, state) in super::svm_vm::dtn() {
        let acr = match state {
            super::svm_vm::SvmVmState::Created => "created",
            super::svm_vm::SvmVmState::Running => "running",
            super::svm_vm::SvmVmState::Stopped => "stopped",
            super::svm_vm::SvmVmState::Paused => "paused",
            _ => "unknown",
        };
        result.push((id, name, acr));
    }
    
    result
}
