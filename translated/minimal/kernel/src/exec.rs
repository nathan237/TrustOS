




use alloc::vec::Vec;
use alloc::string::String;
use core::sync::atomic::{AtomicU64, AtomicPtr, Ordering};
use crate::elf::{Mm, ElfError, Os};
use crate::memory::paging::{AddressSpace, PageFlags, UserMemoryRegion};
use crate::memory::hhdm_offset;


const ALI_: usize = 1024 * 1024;





static CP_: AtomicPtr<AddressSpace> = AtomicPtr::new(core::ptr::null_mut());


static FQ_: AtomicU64 = AtomicU64::new(0);


static GW_: AtomicU64 = AtomicU64::new(0);



static BXM_: spin::Mutex<()> = spin::Mutex::new(());






pub fn ffh<F, U>(f: F) -> Option<U>
where
    F: FnOnce(&mut AddressSpace) -> U,
{
    let ptr = CP_.load(Ordering::Acquire);
    if ptr.is_null() {
        return None;
    }
    
    
    Some(f(unsafe { &mut *ptr }))
}


pub fn bfr() -> u64 {
    FQ_.load(Ordering::Relaxed)
}


pub fn oor(brk: u64) {
    FQ_.store(brk, Ordering::SeqCst);
}


pub fn lap() -> u64 {
    GW_.load(Ordering::Relaxed)
}


#[derive(Debug)]
pub enum ExecResult {
    
    Exited(i32),
    
    Faulted(&'static str),
    
    LoadError(ElfError),
    
    MemoryError,
}



pub fn lsh(path: &str, argv: &[&str], _envp: &[&str]) -> Result<(), ExecResult> {
    match elt(path, argv) {
        ExecResult::Exited(0) => Ok(()),
        other => Err(other),
    }
}


pub fn resolve_path(name: &str) -> Option<String> {
    
    if name.contains('/') {
        
        if crate::vfs::stat(name).is_ok() {
            return Some(String::from(name));
        }
        return None;
    }
    
    
    let dyu = ["/bin", "/usr/bin", "/sbin", "/usr/sbin", "/usr/local/bin"];
    for it in &dyu {
        let xo = alloc::format!("{}/{}", it, name);
        if crate::vfs::stat(&xo).is_ok() {
            return Some(xo);
        }
    }
    None
}


fn kjr(data: &[u8]) -> Option<(String, String)> {
    if data.len() < 4 || data[0] != b'#' || data[1] != b'!' {
        return None;
    }
    
    let end = data.iter().position(|&b| b == b'\n').unwrap_or(data.len().min(256));
    let line = core::str::from_utf8(&data[2..end]).ok()?;
    let line = line.trim();
    
    let mut au = line.splitn(2, ' ');
    let interp = au.next()?.trim();
    if interp.is_empty() { return None; }
    Some((String::from(interp), String::new()))
}


pub fn elt(path: &str, args: &[&str]) -> ExecResult {
    crate::log!("[EXEC] Loading: {}", path);
    
    
    let afn = resolve_path(path).unwrap_or_else(|| String::from(path));
    
    
    let elf = match crate::elf::nac(&afn) {
        Ok(e) => e,
        Err(e) => {
            
            if let Ok(data) = crate::vfs::read_file(&afn) {
                if let Some((interp, _extra)) = kjr(&data) {
                    
                    let mut ipo = alloc::vec![interp.as_str(), afn.as_str()];
                    ipo.extend_from_slice(args);
                    
                    return elt(&interp, &ipo[1..]);
                }
            }
            crate::log_error!("[EXEC] Failed to load ELF: {:?}", e);
            return ExecResult::LoadError(e);
        }
    };
    
    hwv(&elf, args)
}


pub fn lru(data: &[u8], args: &[&str]) -> ExecResult {
    
    let elf = match crate::elf::gfw(data) {
        Ok(e) => e,
        Err(e) => {
            crate::log_error!("[EXEC] Failed to parse ELF: {:?}", e);
            return ExecResult::LoadError(e);
        }
    };
    
    hwv(&elf, args)
}


fn hwv(elf: &Mm, args: &[&str]) -> ExecResult {
    crate::log!("[EXEC] Entry point: {:#x}", elf.entry_point);
    crate::log!("[EXEC] Address range: {:#x} - {:#x}", elf.min_vaddr, elf.max_vaddr);
    
    
    let mut address_space = match AddressSpace::bnt() {
        Some(a) => a,
        None => {
            crate::log_error!("[EXEC] Failed to create address space");
            return ExecResult::MemoryError;
        }
    };
    
    let bz = hhdm_offset();
    
    
    for segment in &elf.segments {
        let boc = ((segment.size as usize + 4095) / 4096).max(1);
        
        crate::log_debug!("[EXEC] Mapping segment: vaddr={:#x}, size={}, pages={}", 
            segment.vaddr, segment.size, boc);
        
        for dce in 0..boc {
            let virt_addr = (segment.vaddr & !0xFFF) + (dce as u64 * 4096);
            
            
            let coa = match asx() {
                Some(aa) => aa,
                None => {
                    crate::log_error!("[EXEC] Out of memory");
                    return ExecResult::MemoryError;
                }
            };
            
            
            let erk = (segment.flags & 1) != 0;  
            let is_write = (segment.flags & 2) != 0; 
            
            let flags = if erk {
                PageFlags::JZ_
            } else if is_write {
                PageFlags::FM_
            } else {
                PageFlags::DDU_
            };
            
            
            if address_space.map_page(virt_addr, coa, flags).is_none() {
                crate::log_error!("[EXEC] Failed to map page at {:#x}", virt_addr);
                return ExecResult::MemoryError;
            }
            
            
            let page_virt = coa + bz;
            let gtn = dce * 4096;
            let hnq = gtn;
            let kxr = ((segment.data.len()).min(gtn + 4096)).max(gtn);
            
            if hnq < segment.data.len() {
                let src = &segment.data[hnq..kxr];
                let mt = unsafe {
                    core::slice::from_raw_parts_mut(page_virt as *mut u8, 4096)
                };
                mt[..src.len()].copy_from_slice(src);
                
                for b in &mut mt[src.len()..] {
                    *b = 0;
                }
            } else {
                
                let mt = unsafe {
                    core::slice::from_raw_parts_mut(page_virt as *mut u8, 4096)
                };
                mt.fill(0);
            }
        }
    }
    
    
    if !elf.relocations.is_empty() {
        crate::log_debug!("[EXEC] Applying {} relocations (base={:#x})", elf.relocations.len(), elf.base_addr);
        for bdg in &elf.relocations {
            let jlq = bdg.offset + elf.base_addr;
            match bdg.rel_type {
                8 => {
                    
                    let value = elf.base_addr.wrapping_add(bdg.addend as u64);
                    if let Some(phys) = address_space.translate(jlq) {
                        unsafe { *((phys + bz) as *mut u64) = value; }
                    }
                }
                1 | 6 | 7 => {
                    
                    let value = elf.base_addr.wrapping_add(bdg.addend as u64);
                    if let Some(phys) = address_space.translate(jlq) {
                        unsafe { *((phys + bz) as *mut u64) = value; }
                    }
                }
                _ => {
                    crate::log_debug!("[EXEC] Unsupported reloc type {}", bdg.rel_type);
                }
            }
        }
    }
    
    
    let aei = ALI_ / 4096;
    let bdt = UserMemoryRegion::QM_ - (aei as u64 * 4096);
    let mge = bdt - 4096;
    
    crate::log_debug!("[EXEC] Guard page at {:#x}, stack: {:#x} - {:#x}", mge, bdt, UserMemoryRegion::QM_);
    
    
    
    
    for i in 0..aei {
        let virt_addr = bdt + (i as u64 * 4096);
        
        let coa = match asx() {
            Some(aa) => aa,
            None => {
                crate::log_error!("[EXEC] Out of memory for stack");
                return ExecResult::MemoryError;
            }
        };
        
        
        let page_virt = coa + bz;
        unsafe {
            core::ptr::write_bytes(page_virt as *mut u8, 0, 4096);
        }
        
        if address_space.map_page(virt_addr, coa, PageFlags::FM_).is_none() {
            crate::log_error!("[EXEC] Failed to map stack at {:#x}", virt_addr);
            return ExecResult::MemoryError;
        }
    }
    
    
    let mut sp = UserMemoryRegion::QM_;
    
    
    let mut fhi: Vec<u64> = Vec::new();
    for db in args.iter().rev() {
        let bytes = db.as_bytes();
        sp -= (bytes.len() as u64) + 1; 
        
        let qxn = sp & !0xFFF;
        let glv = (sp - bdt) as usize;
        let dce = glv / 4096;
        if dce < aei {
            
            
            if let Some(phys) = address_space.translate(sp) {
                let mt = (phys + bz) as *mut u8;
                unsafe {
                    core::ptr::copy_nonoverlapping(bytes.as_ptr(), mt, bytes.len());
                    *mt.add(bytes.len()) = 0; 
                }
            }
        }
        fhi.push(sp);
    }
    fhi.reverse(); 
    
    
    sp &= !7;
    
    
    
    let jyo: [(u64, u64); 7] = [
        (6,  4096),                  
        (3,  elf.min_vaddr + 0x40),  
        (4,  56),                    
        (5,  elf.segments.len() as u64), 
        (9,  elf.entry_point),       
        (25, 0),                     
        (0,  0),                     
    ];
    
    
    for &(dia, aval) in jyo.iter().rev() {
        sp -= 8;
        if let Some(phys) = address_space.translate(sp) {
            unsafe { *((phys + bz) as *mut u64) = aval; }
        }
        sp -= 8;
        if let Some(phys) = address_space.translate(sp) {
            unsafe { *((phys + bz) as *mut u64) = dia; }
        }
    }
    
    
    sp -= 8;
    if let Some(phys) = address_space.translate(sp) {
        unsafe { *((phys + bz) as *mut u64) = 0; }
    }
    
    
    sp -= 8;
    if let Some(phys) = address_space.translate(sp) {
        unsafe { *((phys + bz) as *mut u64) = 0; }
    }
    
    
    for addr in fhi.iter().rev() {
        sp -= 8;
        if let Some(phys) = address_space.translate(sp) {
            unsafe { *((phys + bz) as *mut u64) = *addr; }
        }
    }
    let jxn = sp; 
    
    
    sp -= 8;
    if let Some(phys) = address_space.translate(sp) {
        unsafe { *((phys + bz) as *mut u64) = args.len() as u64; }
    }
    
    
    sp &= !0xF;
    
    crate::log!("[EXEC] Ready to execute at {:#x}, stack at {:#x} (argc={}, argv={:#x})", 
        elf.entry_point, sp, args.len(), jxn);
    
    
    let nyc = args.first().copied().unwrap_or("user");
    let pid = crate::process::spawn(nyc).unwrap_or(0);
    let dwv = crate::process::pe();
    
    
    crate::process::opf(pid, crate::process::MemoryLayout {
        code_start: elf.min_vaddr,
        code_end: elf.max_vaddr,
        heap_start: UserMemoryRegion::CH_,
        heap_end: UserMemoryRegion::CH_,
        stack_start: bdt,
        stack_end: UserMemoryRegion::QM_,
        ..Default::default()
    });
    
    
    let exit_code;
    
    
    crate::vfs::gul();
    
    
    let jsm = BXM_.lock();
    
    unsafe {
        
        let kernel_cr3: u64;
        #[cfg(target_arch = "x86_64")]
        core::arch::asm!("mov {}, cr3", out(reg) kernel_cr3);
        #[cfg(not(target_arch = "x86_64"))]
        { kernel_cr3 = crate::arch::biw(); }
        
        
        CP_.store(&mut address_space as *mut AddressSpace, Ordering::Release);
        FQ_.store(UserMemoryRegion::CH_, Ordering::SeqCst);
        GW_.store(bdt, Ordering::SeqCst);
        
        
        crate::process::gwd(pid);
        
        
        address_space.activate();
        
        crate::log!("[EXEC] PID {} entering Ring 3 at {:#x}...", pid, elf.entry_point);
        
        
        exit_code = crate::userland::bzn(elf.entry_point, sp);
        
        
        CP_.store(core::ptr::null_mut(), Ordering::Release);
        
        
        #[cfg(target_arch = "x86_64")]
        core::arch::asm!("mov cr3, {}", in(reg) kernel_cr3, options(nostack, preserves_flags));
        #[cfg(not(target_arch = "x86_64"))]
        crate::arch::bkc(kernel_cr3);
    }
    
    
    drop(jsm);
    
    
    crate::process::finish(pid, exit_code);
    crate::process::gqo(pid);
    
    
    crate::vfs::flv();
    
    
    crate::process::faf(dwv);
    
    crate::log!("[EXEC] PID {} exited with code {}", pid, exit_code);
    ExecResult::Exited(exit_code)
}







pub fn doy() -> ExecResult {
    crate::log!("[EXEC] Running Ring 3 hello world test...");
    
    
    let mut address_space = match AddressSpace::bnt() {
        Some(a) => a,
        None => {
            crate::log_error!("[EXEC] Failed to create address space");
            return ExecResult::MemoryError;
        }
    };
    
    let bz = hhdm_offset();
    
    
    
    
    
    
    
    
    
    
    
    
    
    
    
    
    
    
    let jlt: [u8; 63] = [
        
        0x48, 0xC7, 0xC0, 0x01, 0x00, 0x00, 0x00,
        
        0x48, 0xC7, 0xC7, 0x01, 0x00, 0x00, 0x00,
        
        0x48, 0x8D, 0x35, 0x17, 0x00, 0x00, 0x00,
        
        0x48, 0xC7, 0xC2, 0x13, 0x00, 0x00, 0x00,
        
        0x0F, 0x05,
        
        0x48, 0xC7, 0xC0, 0x3C, 0x00, 0x00, 0x00,
        
        0x48, 0x31, 0xFF,
        
        0x0F, 0x05,
        
        0xEB, 0xFE,
        
        b'H', b'e', b'l', b'l', b'o', b' ', b'f', b'r', b'o', b'm',
        b' ', b'R', b'i', b'n', b'g', b' ', b'3', b'!', b'\n',
    ];
    
    
    let aez: u64 = 0x400000;
    let code_phys = match asx() {
        Some(aa) => aa,
        None => return ExecResult::MemoryError,
    };
    
    crate::log!("[EXEC] Code page: phys={:#x}, vaddr={:#x}", code_phys, aez);
    
    
    unsafe {
        let mt = (code_phys + bz) as *mut u8;
        core::ptr::write_bytes(mt, 0, 4096); 
        core::ptr::copy_nonoverlapping(jlt.as_ptr(), mt, jlt.len());
    }
    
    
    if address_space.map_page(aez, code_phys, PageFlags::JZ_).is_none() {
        crate::log_error!("[EXEC] Failed to map code page");
        return ExecResult::MemoryError;
    }
    
    
    let te: u64 = 0x7FFFFFFF0000;
    let aei = 4;
    let qjx = te - (aei as u64 + 1) * 4096;
    
    
    for i in 0..aei {
        let vaddr = te - (i as u64 + 1) * 4096;
        let phys = match asx() {
            Some(aa) => aa,
            None => return ExecResult::MemoryError,
        };
        unsafe { core::ptr::write_bytes((phys + bz) as *mut u8, 0, 4096); }
        
        if address_space.map_page(vaddr, phys, PageFlags::FM_).is_none() {
            crate::log_error!("[EXEC] Failed to map stack page");
            return ExecResult::MemoryError;
        }
    }
    
    let user_stack = te - 8; 
    
    crate::log!("[EXEC] Jumping to Ring 3 at {:#x}, stack at {:#x}", aez, user_stack);
    
    
    unsafe {
        let kernel_cr3: u64;
        #[cfg(target_arch = "x86_64")]
        core::arch::asm!("mov {}, cr3", out(reg) kernel_cr3);
        #[cfg(not(target_arch = "x86_64"))]
        { kernel_cr3 = crate::arch::biw(); }
        
        
        CP_.store(&mut address_space as *mut AddressSpace, Ordering::Release);
        FQ_.store(UserMemoryRegion::CH_, Ordering::SeqCst);
        let phm = te - (aei as u64 * 4096);
        GW_.store(phm, Ordering::SeqCst);
        
        address_space.activate();
        
        let exit_code = crate::userland::bzn(aez, user_stack);
        
        
        CP_.store(core::ptr::null_mut(), Ordering::Release);
        
        
        #[cfg(target_arch = "x86_64")]
        core::arch::asm!("mov cr3, {}", in(reg) kernel_cr3, options(nostack, preserves_flags));
        #[cfg(not(target_arch = "x86_64"))]
        crate::arch::bkc(kernel_cr3);
        
        crate::log!("[EXEC] Ring 3 test exited with code {}", exit_code);
        ExecResult::Exited(exit_code)
    }
}








pub static AEZ_: &[u8] = &{
    
    let mut elf = [0u8; 183];
    
    
    elf[0] = 0x7F; elf[1] = b'E'; elf[2] = b'L'; elf[3] = b'F'; 
    elf[4] = 2;    
    elf[5] = 1;    
    elf[6] = 1;    
    
    
    
    elf[16] = 2; elf[17] = 0;
    
    elf[18] = 0x3E; elf[19] = 0;
    
    elf[20] = 1; elf[21] = 0; elf[22] = 0; elf[23] = 0;
    
    elf[24] = 0x78; elf[25] = 0x00; elf[26] = 0x40; elf[27] = 0x00;
    elf[28] = 0; elf[29] = 0; elf[30] = 0; elf[31] = 0;
    
    elf[32] = 64; elf[33] = 0; elf[34] = 0; elf[35] = 0;
    elf[36] = 0; elf[37] = 0; elf[38] = 0; elf[39] = 0;
    
    
    
    
    
    elf[52] = 64; elf[53] = 0;
    
    elf[54] = 56; elf[55] = 0;
    
    elf[56] = 1; elf[57] = 0;
    
    elf[58] = 64; elf[59] = 0;
    
    
    
    
    
    elf[64] = 1; elf[65] = 0; elf[66] = 0; elf[67] = 0;
    
    elf[68] = 5; elf[69] = 0; elf[70] = 0; elf[71] = 0;
    
    
    
    elf[80] = 0x00; elf[81] = 0x00; elf[82] = 0x40; elf[83] = 0x00;
    elf[84] = 0; elf[85] = 0; elf[86] = 0; elf[87] = 0;
    
    elf[88] = 0x00; elf[89] = 0x00; elf[90] = 0x40; elf[91] = 0x00;
    elf[92] = 0; elf[93] = 0; elf[94] = 0; elf[95] = 0;
    
    elf[96] = 183; elf[97] = 0; elf[98] = 0; elf[99] = 0;
    elf[100] = 0; elf[101] = 0; elf[102] = 0; elf[103] = 0;
    
    elf[104] = 183; elf[105] = 0; elf[106] = 0; elf[107] = 0;
    elf[108] = 0; elf[109] = 0; elf[110] = 0; elf[111] = 0;
    
    elf[112] = 0x00; elf[113] = 0x10; elf[114] = 0; elf[115] = 0;
    elf[116] = 0; elf[117] = 0; elf[118] = 0; elf[119] = 0;
    
    
    
    elf[120] = 0x48; elf[121] = 0xC7; elf[122] = 0xC0;
    elf[123] = 0x01; elf[124] = 0x00; elf[125] = 0x00; elf[126] = 0x00;
    
    elf[127] = 0x48; elf[128] = 0xC7; elf[129] = 0xC7;
    elf[130] = 0x01; elf[131] = 0x00; elf[132] = 0x00; elf[133] = 0x00;
    
    
    elf[134] = 0x48; elf[135] = 0x8D; elf[136] = 0x35;
    elf[137] = 0x17; elf[138] = 0x00; elf[139] = 0x00; elf[140] = 0x00;
    
    elf[141] = 0x48; elf[142] = 0xC7; elf[143] = 0xC2;
    elf[144] = 0x13; elf[145] = 0x00; elf[146] = 0x00; elf[147] = 0x00;
    
    elf[148] = 0x0F; elf[149] = 0x05;
    
    elf[150] = 0x48; elf[151] = 0xC7; elf[152] = 0xC0;
    elf[153] = 0x3C; elf[154] = 0x00; elf[155] = 0x00; elf[156] = 0x00;
    
    elf[157] = 0x48; elf[158] = 0x31; elf[159] = 0xFF;
    
    elf[160] = 0x0F; elf[161] = 0x05;
    
    elf[162] = 0xEB; elf[163] = 0xFE;
    
    elf[164] = b'H'; elf[165] = b'e'; elf[166] = b'l'; elf[167] = b'l';
    elf[168] = b'o'; elf[169] = b' '; elf[170] = b'f'; elf[171] = b'r';
    elf[172] = b'o'; elf[173] = b'm'; elf[174] = b' '; elf[175] = b'R';
    elf[176] = b'i'; elf[177] = b'n'; elf[178] = b'g'; elf[179] = b' ';
    elf[180] = b'3'; elf[181] = b'!'; elf[182] = b'\n';
    
    elf
};


pub fn fvl() -> ExecResult {
    crate::log!("[EXEC] Running embedded hello world ELF...");
    lru(AEZ_, &[])
}










pub fn hww() -> ExecResult {
    crate::log!("[EXEC] Running v0.3 memory test in Ring 3...");

    
    
    
    
    
    
    
    
    
    
    
    let inj: [u8; 194] = [
        
        0xB8, 0x0C, 0x00, 0x00, 0x00,                     
        0x31, 0xFF,                                         
        0x0F, 0x05,                                         
        0x49, 0x89, 0xC4,                                   

        
        0x4C, 0x89, 0xE7,                                   
        0x48, 0x81, 0xC7, 0x00, 0x10, 0x00, 0x00,         
        0xB8, 0x0C, 0x00, 0x00, 0x00,                     
        0x0F, 0x05,                                         

        
        0x41, 0xC6, 0x04, 0x24, 0x42,                     
        0x41, 0x80, 0x3C, 0x24, 0x42,                     
        0x0F, 0x85, 0x62, 0x00, 0x00, 0x00,               

        
        0xB8, 0x09, 0x00, 0x00, 0x00,                     
        0x31, 0xFF,                                         
        0xBE, 0x00, 0x10, 0x00, 0x00,                     
        0xBA, 0x03, 0x00, 0x00, 0x00,                     
        0x41, 0xBA, 0x22, 0x00, 0x00, 0x00,               
        0x49, 0xC7, 0xC0, 0xFF, 0xFF, 0xFF, 0xFF,         
        0x45, 0x31, 0xC9,                                   
        0x0F, 0x05,                                         

        
        0x48, 0x85, 0xC0,                                   
        0x0F, 0x88, 0x36, 0x00, 0x00, 0x00,               

        
        0x49, 0x89, 0xC5,                                   
        0x41, 0xC6, 0x45, 0x00, 0x99,                     
        0x41, 0x80, 0x7D, 0x00, 0x99,                     
        0x0F, 0x85, 0x23, 0x00, 0x00, 0x00,               

        
        0xB8, 0x01, 0x00, 0x00, 0x00,                     
        0xBF, 0x01, 0x00, 0x00, 0x00,                     
        0x48, 0x8D, 0x35, 0x38, 0x00, 0x00, 0x00,         
        0xBA, 0x08, 0x00, 0x00, 0x00,                     
        0x0F, 0x05,                                         
        0xB8, 0x3C, 0x00, 0x00, 0x00,                     
        0x31, 0xFF,                                         
        0x0F, 0x05,                                         
        0xEB, 0xFE,                                         

        
        0xB8, 0x01, 0x00, 0x00, 0x00,                     
        0xBF, 0x01, 0x00, 0x00, 0x00,                     
        0x48, 0x8D, 0x35, 0x1D, 0x00, 0x00, 0x00,         
        0xBA, 0x05, 0x00, 0x00, 0x00,                     
        0x0F, 0x05,                                         
        0xB8, 0x3C, 0x00, 0x00, 0x00,                     
        0xBF, 0x01, 0x00, 0x00, 0x00,                     
        0x0F, 0x05,                                         
        0xEB, 0xFE,                                         

        
        b'v', b'0', b'.', b'3', b' ', b'O', b'K', b'\n', 
        b'F', b'A', b'I', b'L', b'\n',                     
    ];

    

    let mut address_space = match AddressSpace::bnt() {
        Some(a) => a,
        None => {
            crate::log_error!("[EXEC] Failed to create address space");
            return ExecResult::MemoryError;
        }
    };

    let bz = hhdm_offset();

    
    let aez: u64 = 0x400000;
    let code_phys = match asx() {
        Some(aa) => aa,
        None => return ExecResult::MemoryError,
    };
    unsafe {
        let mt = (code_phys + bz) as *mut u8;
        core::ptr::write_bytes(mt, 0, 4096);
        core::ptr::copy_nonoverlapping(inj.as_ptr(), mt, inj.len());
    }
    if address_space.map_page(aez, code_phys, PageFlags::JZ_).is_none() {
        return ExecResult::MemoryError;
    }

    
    let te: u64 = 0x7FFFFFFF0000;
    let aei = 4;
    for i in 0..aei {
        let vaddr = te - (i as u64 + 1) * 4096;
        let phys = match asx() {
            Some(aa) => aa,
            None => return ExecResult::MemoryError,
        };
        unsafe { core::ptr::write_bytes((phys + bz) as *mut u8, 0, 4096); }
        if address_space.map_page(vaddr, phys, PageFlags::FM_).is_none() {
            return ExecResult::MemoryError;
        }
    }

    let user_stack = te - 8;

    crate::log!("[EXEC] memtest: code at {:#x}, stack at {:#x}", aez, user_stack);

    unsafe {
        let kernel_cr3: u64;
        #[cfg(target_arch = "x86_64")]
        core::arch::asm!("mov {}, cr3", out(reg) kernel_cr3);
        #[cfg(not(target_arch = "x86_64"))]
        { kernel_cr3 = crate::arch::biw(); }

        
        CP_.store(&mut address_space as *mut AddressSpace, Ordering::Release);
        FQ_.store(UserMemoryRegion::CH_, Ordering::SeqCst);
        let bvx = te - (aei as u64 * 4096);
        GW_.store(bvx, Ordering::SeqCst);

        address_space.activate();
        let exit_code = crate::userland::bzn(aez, user_stack);

        CP_.store(core::ptr::null_mut(), Ordering::Release);
        #[cfg(target_arch = "x86_64")]
        core::arch::asm!("mov cr3, {}", in(reg) kernel_cr3, options(nostack, preserves_flags));
        #[cfg(not(target_arch = "x86_64"))]
        crate::arch::bkc(kernel_cr3);

        crate::log!("[EXEC] memtest exited with code {}", exit_code);
        ExecResult::Exited(exit_code)
    }
}


fn asx() -> Option<u64> {
    crate::memory::frame::aan()
}






pub fn hwx() -> ExecResult {
    crate::log!("[EXEC] Running Ring 3 IPC pipe test...");

    let mut address_space = match AddressSpace::bnt() {
        Some(a) => a,
        None => {
            crate::log_error!("[EXEC] Failed to create address space");
            return ExecResult::MemoryError;
        }
    };

    let bz = hhdm_offset();

    
    
    
    
    
    
    
    
    
    
    
    
    
    
    
    
    
    
    
    
    
    
    
    
    
    
    
    
    
    
    
    
    
    
    
    
    
    
    
    let iuz: [u8; 138] = [
        
        0x48, 0x83, 0xEC, 0x10,
        
        0xC7, 0x44, 0x24, 0x08, 0x50, 0x49, 0x50, 0x45,
        
        0x48, 0x89, 0xE7,                               
        0x31, 0xF6,                                     
        0xB8, 0x25, 0x01, 0x00, 0x00,                   
        0x0F, 0x05,                                     
        
        0x85, 0xC0,
        0x75, 0x5B,
        
        0x8B, 0x7C, 0x24, 0x04,                         
        0x48, 0x8D, 0x74, 0x24, 0x08,                   
        0xBA, 0x04, 0x00, 0x00, 0x00,                   
        0xB8, 0x01, 0x00, 0x00, 0x00,                   
        0x0F, 0x05,                                     
        
        0x83, 0xF8, 0x04,
        0x75, 0x41,
        
        0x8B, 0x3C, 0x24,                               
        0x48, 0x8D, 0x74, 0x24, 0x0C,                   
        0xBA, 0x04, 0x00, 0x00, 0x00,                   
        0x31, 0xC0,                                     
        0x0F, 0x05,                                     
        
        0x83, 0xF8, 0x04,
        0x75, 0x2B,
        
        0x8B, 0x44, 0x24, 0x0C,                         
        0x3B, 0x44, 0x24, 0x08,                         
        
        0x75, 0x21,
        
        0xB8, 0x01, 0x00, 0x00, 0x00,                   
        0xBF, 0x01, 0x00, 0x00, 0x00,                   
        0x48, 0x8D, 0x35, 0x1C, 0x00, 0x00, 0x00,       
        0xBA, 0x07, 0x00, 0x00, 0x00,                   
        0x0F, 0x05,                                     
        
        0x31, 0xFF,                                     
        0xB8, 0x3C, 0x00, 0x00, 0x00,                   
        0x0F, 0x05,                                     
        
        0xBF, 0x01, 0x00, 0x00, 0x00,                   
        0xB8, 0x3C, 0x00, 0x00, 0x00,                   
        0x0F, 0x05,                                     
        
        b'I', b'P', b'C', b' ', b'O', b'K', b'\n',
    ];

    
    let aez: u64 = 0x400000;
    let code_phys = match asx() {
        Some(aa) => aa,
        None => return ExecResult::MemoryError,
    };
    unsafe {
        let mt = (code_phys + bz) as *mut u8;
        core::ptr::write_bytes(mt, 0, 4096);
        core::ptr::copy_nonoverlapping(iuz.as_ptr(), mt, iuz.len());
    }
    if address_space.map_page(aez, code_phys, PageFlags::JZ_).is_none() {
        return ExecResult::MemoryError;
    }

    
    let te: u64 = 0x7FFFFFFF0000;
    let aei = 4;
    for i in 0..aei {
        let vaddr = te - (i as u64 + 1) * 4096;
        let phys = match asx() {
            Some(aa) => aa,
            None => return ExecResult::MemoryError,
        };
        unsafe { core::ptr::write_bytes((phys + bz) as *mut u8, 0, 4096); }
        if address_space.map_page(vaddr, phys, PageFlags::FM_).is_none() {
            return ExecResult::MemoryError;
        }
    }

    let user_stack = te - 8;

    crate::log!("[EXEC] pipe_test: code at {:#x}, stack at {:#x}", aez, user_stack);

    unsafe {
        let kernel_cr3: u64;
        #[cfg(target_arch = "x86_64")]
        core::arch::asm!("mov {}, cr3", out(reg) kernel_cr3);
        #[cfg(not(target_arch = "x86_64"))]
        { kernel_cr3 = crate::arch::biw(); }

        CP_.store(&mut address_space as *mut AddressSpace, Ordering::Release);
        FQ_.store(UserMemoryRegion::CH_, Ordering::SeqCst);
        let bvx = te - (aei as u64 * 4096);
        GW_.store(bvx, Ordering::SeqCst);

        address_space.activate();
        let exit_code = crate::userland::bzn(aez, user_stack);

        CP_.store(core::ptr::null_mut(), Ordering::Release);
        #[cfg(target_arch = "x86_64")]
        core::arch::asm!("mov cr3, {}", in(reg) kernel_cr3, options(nostack, preserves_flags));
        #[cfg(not(target_arch = "x86_64"))]
        crate::arch::bkc(kernel_cr3);

        crate::log!("[EXEC] pipe_test exited with code {}", exit_code);
        ExecResult::Exited(exit_code)
    }
}








pub fn lrv() -> ExecResult {
    crate::log!("[EXEC] Running exception safety test (UD2 in Ring 3)...");

    let mut address_space = match AddressSpace::bnt() {
        Some(a) => a,
        None => return ExecResult::MemoryError,
    };
    let bz = hhdm_offset();

    
    
    
    let code: [u8; 4] = [
        0x0F, 0x0B,  
        0xEB, 0xFE,  
    ];

    let aez: u64 = 0x400000;
    let code_phys = match asx() {
        Some(aa) => aa,
        None => return ExecResult::MemoryError,
    };
    unsafe {
        let mt = (code_phys + bz) as *mut u8;
        core::ptr::write_bytes(mt, 0, 4096);
        core::ptr::copy_nonoverlapping(code.as_ptr(), mt, code.len());
    }
    if address_space.map_page(aez, code_phys, PageFlags::JZ_).is_none() {
        return ExecResult::MemoryError;
    }

    let te: u64 = 0x7FFFFFFF0000;
    let aei = 4usize;
    for i in 0..aei {
        let vaddr = te - (i as u64 + 1) * 4096;
        let phys = match asx() { Some(aa) => aa, None => return ExecResult::MemoryError };
        unsafe { core::ptr::write_bytes((phys + bz) as *mut u8, 0, 4096); }
        if address_space.map_page(vaddr, phys, PageFlags::FM_).is_none() {
            return ExecResult::MemoryError;
        }
    }
    let user_stack = te - 8;

    unsafe {
        let kernel_cr3: u64;
        #[cfg(target_arch = "x86_64")]
        core::arch::asm!("mov {}, cr3", out(reg) kernel_cr3);
        #[cfg(not(target_arch = "x86_64"))]
        { kernel_cr3 = crate::arch::biw(); }
        CP_.store(&mut address_space as *mut AddressSpace, Ordering::Release);
        FQ_.store(UserMemoryRegion::CH_, Ordering::SeqCst);
        GW_.store(te - (aei as u64 * 4096), Ordering::SeqCst);
        address_space.activate();
        let exit_code = crate::userland::bzn(aez, user_stack);
        CP_.store(core::ptr::null_mut(), Ordering::Release);
        #[cfg(target_arch = "x86_64")]
        core::arch::asm!("mov cr3, {}", in(reg) kernel_cr3, options(nostack, preserves_flags));
        #[cfg(not(target_arch = "x86_64"))]
        crate::arch::bkc(kernel_cr3);
        crate::log!("[EXEC] exception_safety_test exited with code {}", exit_code);
        ExecResult::Exited(exit_code)
    }
}



pub fn lry() -> ExecResult {
    crate::log!("[EXEC] Running signal syscall test...");

    let mut address_space = match AddressSpace::bnt() {
        Some(a) => a,
        None => return ExecResult::MemoryError,
    };
    let bz = hhdm_offset();

    
    
    
    
    
    
    
    
    
    #[rustfmt::skip]
    let code: [u8; 158] = [
        
        0xB8, 0x27, 0x00, 0x00, 0x00,              
        0x0F, 0x05,                                  
        0x49, 0x89, 0xC4,                            
        
        0x48, 0x83, 0xEC, 0x10,                      
        0x48, 0xC7, 0x04, 0x24, 0x00, 0x00, 0x00, 0x00, 
        
        0xB8, 0x0E, 0x00, 0x00, 0x00,              
        0xBF, 0x02, 0x00, 0x00, 0x00,              
        0x48, 0x8D, 0x34, 0x24,                     
        0x48, 0x8D, 0x54, 0x24, 0x08,              
        0x41, 0xBA, 0x08, 0x00, 0x00, 0x00,        
        0x0F, 0x05,                                  
        0x85, 0xC0,                                  
        0x75, 0x33,                                  
        
        0xB8, 0x3E, 0x00, 0x00, 0x00,              
        0x44, 0x89, 0xE7,                            
        0x31, 0xF6,                                  
        0x0F, 0x05,                                  
        0x85, 0xC0,                                  
        0x75, 0x23,                                  
        
        0xB8, 0x01, 0x00, 0x00, 0x00,              
        0xBF, 0x01, 0x00, 0x00, 0x00,              
        0x48, 0x8D, 0x35, 0x38, 0x00, 0x00, 0x00,  
        0xBA, 0x07, 0x00, 0x00, 0x00,              
        0x0F, 0x05,                                  
        
        0xB8, 0x3C, 0x00, 0x00, 0x00,              
        0x31, 0xFF,                                  
        0x0F, 0x05,                                  
        0xEB, 0xFE,                                  
        
        0xB8, 0x01, 0x00, 0x00, 0x00,              
        0xBF, 0x01, 0x00, 0x00, 0x00,              
        0x48, 0x8D, 0x35, 0x1C, 0x00, 0x00, 0x00,  
        0xBA, 0x09, 0x00, 0x00, 0x00,              
        0x0F, 0x05,                                  
        0xB8, 0x3C, 0x00, 0x00, 0x00,              
        0xBF, 0x01, 0x00, 0x00, 0x00,              
        0x0F, 0x05,                                  
        0xEB, 0xFE,                                  
        
        b'S', b'I', b'G', b' ', b'O', b'K', b'\n',              
        b'S', b'I', b'G', b' ', b'F', b'A', b'I', b'L', b'\n', 
    ];

    let aez: u64 = 0x400000;
    let code_phys = match asx() {
        Some(aa) => aa,
        None => return ExecResult::MemoryError,
    };
    unsafe {
        let mt = (code_phys + bz) as *mut u8;
        core::ptr::write_bytes(mt, 0, 4096);
        core::ptr::copy_nonoverlapping(code.as_ptr(), mt, code.len());
    }
    if address_space.map_page(aez, code_phys, PageFlags::JZ_).is_none() {
        return ExecResult::MemoryError;
    }

    let te: u64 = 0x7FFFFFFF0000;
    let aei = 4usize;
    for i in 0..aei {
        let vaddr = te - (i as u64 + 1) * 4096;
        let phys = match asx() { Some(aa) => aa, None => return ExecResult::MemoryError };
        unsafe { core::ptr::write_bytes((phys + bz) as *mut u8, 0, 4096); }
        if address_space.map_page(vaddr, phys, PageFlags::FM_).is_none() {
            return ExecResult::MemoryError;
        }
    }
    let user_stack = te - 8;

    
    let pid = crate::process::spawn("signal_test").unwrap_or(0);
    let dwv = crate::process::pe();

    
    crate::vfs::gul();

    let exit_code;
    unsafe {
        let kernel_cr3: u64;
        #[cfg(target_arch = "x86_64")]
        core::arch::asm!("mov {}, cr3", out(reg) kernel_cr3);
        #[cfg(not(target_arch = "x86_64"))]
        { kernel_cr3 = crate::arch::biw(); }
        CP_.store(&mut address_space as *mut AddressSpace, Ordering::Release);
        FQ_.store(UserMemoryRegion::CH_, Ordering::SeqCst);
        GW_.store(te - (aei as u64 * 4096), Ordering::SeqCst);
        crate::process::gwd(pid);
        address_space.activate();
        exit_code = crate::userland::bzn(aez, user_stack);
        CP_.store(core::ptr::null_mut(), Ordering::Release);
        #[cfg(target_arch = "x86_64")]
        core::arch::asm!("mov cr3, {}", in(reg) kernel_cr3, options(nostack, preserves_flags));
        #[cfg(not(target_arch = "x86_64"))]
        crate::arch::bkc(kernel_cr3);
    }

    crate::process::finish(pid, exit_code);
    crate::process::gqo(pid);
    crate::vfs::flv();
    crate::process::faf(dwv);
    crate::log!("[EXEC] signal_test exited with code {}", exit_code);
    ExecResult::Exited(exit_code)
}



pub fn lrz() -> ExecResult {
    crate::log!("[EXEC] Running stdio/time test...");

    let mut address_space = match AddressSpace::bnt() {
        Some(a) => a,
        None => return ExecResult::MemoryError,
    };
    let bz = hhdm_offset();

    
    
    
    
    
    
    
    
    #[rustfmt::skip]
    let code: [u8; 121] = [
        
        0xB8, 0x27, 0x00, 0x00, 0x00,              
        0x0F, 0x05,                                  
        0x85, 0xC0,                                  
        0x74, 0x3A,                                  
        
        0x48, 0x83, 0xEC, 0x10,                      
        0xB8, 0xE4, 0x00, 0x00, 0x00,              
        0xBF, 0x01, 0x00, 0x00, 0x00,              
        0x48, 0x89, 0xE6,                            
        0x0F, 0x05,                                  
        0x85, 0xC0,                                  
        0x75, 0x23,                                  
        
        0xB8, 0x01, 0x00, 0x00, 0x00,              
        0xBF, 0x01, 0x00, 0x00, 0x00,              
        0x48, 0x8D, 0x35, 0x38, 0x00, 0x00, 0x00,  
        0xBA, 0x06, 0x00, 0x00, 0x00,              
        0x0F, 0x05,                                  
        
        0xB8, 0x3C, 0x00, 0x00, 0x00,              
        0x31, 0xFF,                                  
        0x0F, 0x05,                                  
        0xEB, 0xFE,                                  
        
        0xB8, 0x01, 0x00, 0x00, 0x00,              
        0xBF, 0x01, 0x00, 0x00, 0x00,              
        0x48, 0x8D, 0x35, 0x1B, 0x00, 0x00, 0x00,  
        0xBA, 0x08, 0x00, 0x00, 0x00,              
        0x0F, 0x05,                                  
        0xB8, 0x3C, 0x00, 0x00, 0x00,              
        0xBF, 0x01, 0x00, 0x00, 0x00,              
        0x0F, 0x05,                                  
        0xEB, 0xFE,                                  
        
        b'I', b'O', b' ', b'O', b'K', b'\n',                  
        b'I', b'O', b' ', b'F', b'A', b'I', b'L', b'\n',      
    ];

    let aez: u64 = 0x400000;
    let code_phys = match asx() {
        Some(aa) => aa,
        None => return ExecResult::MemoryError,
    };
    unsafe {
        let mt = (code_phys + bz) as *mut u8;
        core::ptr::write_bytes(mt, 0, 4096);
        core::ptr::copy_nonoverlapping(code.as_ptr(), mt, code.len());
    }
    if address_space.map_page(aez, code_phys, PageFlags::JZ_).is_none() {
        return ExecResult::MemoryError;
    }

    let te: u64 = 0x7FFFFFFF0000;
    let aei = 4usize;
    for i in 0..aei {
        let vaddr = te - (i as u64 + 1) * 4096;
        let phys = match asx() { Some(aa) => aa, None => return ExecResult::MemoryError };
        unsafe { core::ptr::write_bytes((phys + bz) as *mut u8, 0, 4096); }
        if address_space.map_page(vaddr, phys, PageFlags::FM_).is_none() {
            return ExecResult::MemoryError;
        }
    }
    let user_stack = te - 8;

    
    let pid = crate::process::spawn("stdio_test").unwrap_or(0);
    let dwv = crate::process::pe();

    
    crate::vfs::gul();

    let exit_code;
    unsafe {
        let kernel_cr3: u64;
        #[cfg(target_arch = "x86_64")]
        core::arch::asm!("mov {}, cr3", out(reg) kernel_cr3);
        #[cfg(not(target_arch = "x86_64"))]
        { kernel_cr3 = crate::arch::biw(); }
        CP_.store(&mut address_space as *mut AddressSpace, Ordering::Release);
        FQ_.store(UserMemoryRegion::CH_, Ordering::SeqCst);
        GW_.store(te - (aei as u64 * 4096), Ordering::SeqCst);
        crate::process::gwd(pid);
        address_space.activate();
        exit_code = crate::userland::bzn(aez, user_stack);
        CP_.store(core::ptr::null_mut(), Ordering::Release);
        #[cfg(target_arch = "x86_64")]
        core::arch::asm!("mov cr3, {}", in(reg) kernel_cr3, options(nostack, preserves_flags));
        #[cfg(not(target_arch = "x86_64"))]
        crate::arch::bkc(kernel_cr3);
    }

    crate::process::finish(pid, exit_code);
    crate::process::gqo(pid);
    crate::vfs::flv();
    crate::process::faf(dwv);
    crate::log!("[EXEC] stdio_test exited with code {}", exit_code);
    ExecResult::Exited(exit_code)
}


pub fn is_executable(path: &str) -> bool {
    let fd = match crate::vfs::open(path, crate::vfs::OpenFlags(0)) {
        Ok(fd) => fd,
        Err(_) => return false,
    };
    
    let mut magic = [0u8; 4];
    let result = crate::vfs::read(fd, &mut magic).is_ok() 
        && magic == [0x7F, b'E', b'L', b'F'];
    
    crate::vfs::close(fd).ok();
    result
}
