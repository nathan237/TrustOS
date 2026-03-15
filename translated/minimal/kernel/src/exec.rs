




use alloc::vec::Vec;
use alloc::string::String;
use core::sync::atomic::{AtomicU64, AtomicPtr, Ordering};
use crate::elf::{Acr, ElfError, Ahr};
use crate::memory::paging::{AddressSpace, PageFlags, UserMemoryRegion};
use crate::memory::lr;


const AJN_: usize = 1024 * 1024;





static CK_: AtomicPtr<AddressSpace> = AtomicPtr::new(core::ptr::null_mut());


static FB_: AtomicU64 = AtomicU64::new(0);


static GF_: AtomicU64 = AtomicU64::new(0);



static BUQ_: spin::Mutex<()> = spin::Mutex::new(());






pub fn jwy<G, Ac>(bb: G) -> Option<Ac>
where
    G: FnOnce(&mut AddressSpace) -> Ac,
{
    let ptr = CK_.load(Ordering::Acquire);
    if ptr.abq() {
        return None;
    }
    
    
    Some(bb(unsafe { &mut *ptr }))
}


pub fn dfj() -> u64 {
    FB_.load(Ordering::Relaxed)
}


pub fn wio(den: u64) {
    FB_.store(den, Ordering::SeqCst);
}


pub fn rsb() -> u64 {
    GF_.load(Ordering::Relaxed)
}


#[derive(Debug)]
pub enum ExecResult {
    
    Dx(i32),
    
    In(&'static str),
    
    Xk(ElfError),
    
    Bf,
}



pub fn sow(path: &str, cjc: &[&str], qbt: &[&str]) -> Result<(), ExecResult> {
    match itf(path, cjc) {
        ExecResult::Dx(0) => Ok(()),
        gq => Err(gq),
    }
}


pub fn aqj(j: &str) -> Option<String> {
    
    if j.contains('/') {
        
        if crate::vfs::hm(j).is_ok() {
            return Some(String::from(j));
        }
        return None;
    }
    
    
    let hzf = ["/bin", "/usr/bin", "/sbin", "/usr/sbin", "/usr/local/bin"];
    for te in &hzf {
        let auh = alloc::format!("{}/{}", te, j);
        if crate::vfs::hm(&auh).is_ok() {
            return Some(auh);
        }
    }
    None
}


fn qzr(f: &[u8]) -> Option<(String, String)> {
    if f.len() < 4 || f[0] != b'#' || f[1] != b'!' {
        return None;
    }
    
    let ci = f.iter().qf(|&o| o == b'\n').unwrap_or(f.len().v(256));
    let line = core::str::jg(&f[2..ci]).bq()?;
    let line = line.em();
    
    let mut ek = line.eyv(2, ' ');
    let ahp = ek.next()?.em();
    if ahp.is_empty() { return None; }
    Some((String::from(ahp), String::new()))
}


pub fn itf(path: &str, n: &[&str]) -> ExecResult {
    crate::log!("[EXEC] Loading: {}", path);
    
    
    let bhv = aqj(path).unwrap_or_else(|| String::from(path));
    
    
    let elf = match crate::elf::ugx(&bhv) {
        Ok(aa) => aa,
        Err(aa) => {
            
            if let Ok(f) = crate::vfs::mq(&bhv) {
                if let Some((ahp, xzb)) = qzr(&f) {
                    
                    let mut opi = alloc::vec![ahp.as_str(), bhv.as_str()];
                    opi.bk(n);
                    
                    return itf(&ahp, &opi[1..]);
                }
            }
            crate::log_error!("[EXEC] Failed to load ELF: {:?}", aa);
            return ExecResult::Xk(aa);
        }
    };
    
    nrk(&elf, n)
}


pub fn soc(f: &[u8], n: &[&str]) -> ExecResult {
    
    let elf = match crate::elf::ljf(f) {
        Ok(aa) => aa,
        Err(aa) => {
            crate::log_error!("[EXEC] Failed to parse ELF: {:?}", aa);
            return ExecResult::Xk(aa);
        }
    };
    
    nrk(&elf, n)
}


fn nrk(elf: &Acr, n: &[&str]) -> ExecResult {
    crate::log!("[EXEC] Entry point: {:#x}", elf.mi);
    crate::log!("[EXEC] Address range: {:#x} - {:#x}", elf.foj, elf.gmk);
    
    
    let mut ze = match AddressSpace::dtn() {
        Some(q) => q,
        None => {
            crate::log_error!("[EXEC] Failed to create address space");
            return ExecResult::Bf;
        }
    };
    
    let hp = lr();
    
    
    for ie in &elf.jq {
        let duc = ((ie.aw as usize + 4095) / 4096).am(1);
        
        crate::log_debug!("[EXEC] Mapping segment: vaddr={:#x}, size={}, pages={}", 
            ie.uy, ie.aw, duc);
        
        for gor in 0..duc {
            let vd = (ie.uy & !0xFFF) + (gor as u64 * 4096);
            
            
            let fqs = match cja() {
                Some(ai) => ai,
                None => {
                    crate::log_error!("[EXEC] Out of memory");
                    return ExecResult::Bf;
                }
            };
            
            
            let jbh = (ie.flags & 1) != 0;  
            let rm = (ie.flags & 2) != 0; 
            
            let flags = if jbh {
                PageFlags::JG_
            } else if rm {
                PageFlags::EW_
            } else {
                PageFlags::DAC_
            };
            
            
            if ze.bnl(vd, fqs, flags).is_none() {
                crate::log_error!("[EXEC] Failed to map page at {:#x}", vd);
                return ExecResult::Bf;
            }
            
            
            let egd = fqs + hp;
            let mdf = gor * 4096;
            let nfx = mdf;
            let ros = ((ie.f.len()).v(mdf + 4096)).am(mdf);
            
            if nfx < ie.f.len() {
                let cy = &ie.f[nfx..ros];
                let aac = unsafe {
                    core::slice::bef(egd as *mut u8, 4096)
                };
                aac[..cy.len()].dg(cy);
                
                for o in &mut aac[cy.len()..] {
                    *o = 0;
                }
            } else {
                
                let aac = unsafe {
                    core::slice::bef(egd as *mut u8, 4096)
                };
                aac.vi(0);
            }
        }
    }
    
    
    if !elf.bwp.is_empty() {
        crate::log_debug!("[EXEC] Applying {} relocations (base={:#x})", elf.bwp.len(), elf.sm);
        for dbb in &elf.bwp {
            let psa = dbb.l + elf.sm;
            match dbb.fsp {
                8 => {
                    
                    let bn = elf.sm.cn(dbb.fcn as u64);
                    if let Some(ht) = ze.dmr(psa) {
                        unsafe { *((ht + hp) as *mut u64) = bn; }
                    }
                }
                1 | 6 | 7 => {
                    
                    let bn = elf.sm.cn(dbb.fcn as u64);
                    if let Some(ht) = ze.dmr(psa) {
                        unsafe { *((ht + hp) as *mut u64) = bn; }
                    }
                }
                _ => {
                    crate::log_debug!("[EXEC] Unsupported reloc type {}", dbb.fsp);
                }
            }
        }
    }
    
    
    let bfl = AJN_ / 4096;
    let dce = UserMemoryRegion::PP_ - (bfl as u64 * 4096);
    let thu = dce - 4096;
    
    crate::log_debug!("[EXEC] Guard page at {:#x}, stack: {:#x} - {:#x}", thu, dce, UserMemoryRegion::PP_);
    
    
    
    
    for a in 0..bfl {
        let vd = dce + (a as u64 * 4096);
        
        let fqs = match cja() {
            Some(ai) => ai,
            None => {
                crate::log_error!("[EXEC] Out of memory for stack");
                return ExecResult::Bf;
            }
        };
        
        
        let egd = fqs + hp;
        unsafe {
            core::ptr::ahx(egd as *mut u8, 0, 4096);
        }
        
        if ze.bnl(vd, fqs, PageFlags::EW_).is_none() {
            crate::log_error!("[EXEC] Failed to map stack at {:#x}", vd);
            return ExecResult::Bf;
        }
    }
    
    
    let mut sp = UserMemoryRegion::PP_;
    
    
    let mut kay: Vec<u64> = Vec::new();
    for ji in n.iter().vv() {
        let bf = ji.as_bytes();
        sp -= (bf.len() as u64) + 1; 
        
        let zpj = sp & !0xFFF;
        let huc = (sp - dce) as usize;
        let gor = huc / 4096;
        if gor < bfl {
            
            
            if let Some(ht) = ze.dmr(sp) {
                let aac = (ht + hp) as *mut u8;
                unsafe {
                    core::ptr::copy_nonoverlapping(bf.fq(), aac, bf.len());
                    *aac.add(bf.len()) = 0; 
                }
            }
        }
        kay.push(sp);
    }
    kay.dbh(); 
    
    
    sp &= !7;
    
    
    
    let qlo: [(u64, u64); 7] = [
        (6,  4096),                  
        (3,  elf.foj + 0x40),  
        (4,  56),                    
        (5,  elf.jq.len() as u64), 
        (9,  elf.mi),       
        (25, 0),                     
        (0,  0),                     
    ];
    
    
    for &(gzf, qlv) in qlo.iter().vv() {
        sp -= 8;
        if let Some(ht) = ze.dmr(sp) {
            unsafe { *((ht + hp) as *mut u64) = qlv; }
        }
        sp -= 8;
        if let Some(ht) = ze.dmr(sp) {
            unsafe { *((ht + hp) as *mut u64) = gzf; }
        }
    }
    
    
    sp -= 8;
    if let Some(ht) = ze.dmr(sp) {
        unsafe { *((ht + hp) as *mut u64) = 0; }
    }
    
    
    sp -= 8;
    if let Some(ht) = ze.dmr(sp) {
        unsafe { *((ht + hp) as *mut u64) = 0; }
    }
    
    
    for ag in kay.iter().vv() {
        sp -= 8;
        if let Some(ht) = ze.dmr(sp) {
            unsafe { *((ht + hp) as *mut u64) = *ag; }
        }
    }
    let qkk = sp; 
    
    
    sp -= 8;
    if let Some(ht) = ze.dmr(sp) {
        unsafe { *((ht + hp) as *mut u64) = n.len() as u64; }
    }
    
    
    sp &= !0xF;
    
    crate::log!("[EXEC] Ready to execute at {:#x}, stack at {:#x} (argc={}, argv={:#x})", 
        elf.mi, sp, n.len(), qkk);
    
    
    let vmh = n.fv().hu().unwrap_or("user");
    let ce = crate::process::eys(vmh).unwrap_or(0);
    let hvy = crate::process::aei();
    
    
    crate::process::wjf(ce, crate::process::MemoryLayout {
        dez: elf.foj,
        kjr: elf.gmk,
        caa: UserMemoryRegion::CF_,
        ecv: UserMemoryRegion::CF_,
        ibo: dce,
        ibm: UserMemoryRegion::PP_,
        ..Default::default()
    });
    
    
    let nz;
    
    
    crate::vfs::mfc();
    
    
    let qbv = BUQ_.lock();
    
    unsafe {
        
        let ade: u64;
        #[cfg(target_arch = "x86_64")]
        core::arch::asm!("mov {}, cr3", bd(reg) ade);
        #[cfg(not(target_arch = "x86_64"))]
        { ade = crate::arch::dle(); }
        
        
        CK_.store(&mut ze as *mut AddressSpace, Ordering::Release);
        FB_.store(UserMemoryRegion::CF_, Ordering::SeqCst);
        GF_.store(dce, Ordering::SeqCst);
        
        
        crate::process::mhj(ce);
        
        
        ze.fci();
        
        crate::log!("[EXEC] PID {} entering Ring 3 at {:#x}...", ce, elf.mi);
        
        
        nz = crate::userland::eqa(elf.mi, sp);
        
        
        CK_.store(core::ptr::null_mut(), Ordering::Release);
        
        
        #[cfg(target_arch = "x86_64")]
        core::arch::asm!("mov cr3, {}", in(reg) ade, options(nostack, preserves_flags));
        #[cfg(not(target_arch = "x86_64"))]
        crate::arch::dnj(ade);
    }
    
    
    drop(qbv);
    
    
    crate::process::eqi(ce, nz);
    crate::process::lyd(ce);
    
    
    crate::vfs::khv();
    
    
    crate::process::jos(hvy);
    
    crate::log!("[EXEC] PID {} exited with code {}", ce, nz);
    ExecResult::Dx(nz)
}







pub fn hil() -> ExecResult {
    crate::log!("[EXEC] Running Ring 3 hello world test...");
    
    
    let mut ze = match AddressSpace::dtn() {
        Some(q) => q,
        None => {
            crate::log_error!("[EXEC] Failed to create address space");
            return ExecResult::Bf;
        }
    };
    
    let hp = lr();
    
    
    
    
    
    
    
    
    
    
    
    
    
    
    
    
    
    
    let psf: [u8; 63] = [
        
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
    
    
    let bgl: u64 = 0x400000;
    let asn = match cja() {
        Some(ai) => ai,
        None => return ExecResult::Bf,
    };
    
    crate::log!("[EXEC] Code page: phys={:#x}, vaddr={:#x}", asn, bgl);
    
    
    unsafe {
        let aac = (asn + hp) as *mut u8;
        core::ptr::ahx(aac, 0, 4096); 
        core::ptr::copy_nonoverlapping(psf.fq(), aac, psf.len());
    }
    
    
    if ze.bnl(bgl, asn, PageFlags::JG_).is_none() {
        crate::log_error!("[EXEC] Failed to map code page");
        return ExecResult::Bf;
    }
    
    
    let alt: u64 = 0x7FFFFFFF0000;
    let bfl = 4;
    let yvp = alt - (bfl as u64 + 1) * 4096;
    
    
    for a in 0..bfl {
        let uy = alt - (a as u64 + 1) * 4096;
        let ht = match cja() {
            Some(ai) => ai,
            None => return ExecResult::Bf,
        };
        unsafe { core::ptr::ahx((ht + hp) as *mut u8, 0, 4096); }
        
        if ze.bnl(uy, ht, PageFlags::EW_).is_none() {
            crate::log_error!("[EXEC] Failed to map stack page");
            return ExecResult::Bf;
        }
    }
    
    let ais = alt - 8; 
    
    crate::log!("[EXEC] Jumping to Ring 3 at {:#x}, stack at {:#x}", bgl, ais);
    
    
    unsafe {
        let ade: u64;
        #[cfg(target_arch = "x86_64")]
        core::arch::asm!("mov {}, cr3", bd(reg) ade);
        #[cfg(not(target_arch = "x86_64"))]
        { ade = crate::arch::dle(); }
        
        
        CK_.store(&mut ze as *mut AddressSpace, Ordering::Release);
        FB_.store(UserMemoryRegion::CF_, Ordering::SeqCst);
        let xfb = alt - (bfl as u64 * 4096);
        GF_.store(xfb, Ordering::SeqCst);
        
        ze.fci();
        
        let nz = crate::userland::eqa(bgl, ais);
        
        
        CK_.store(core::ptr::null_mut(), Ordering::Release);
        
        
        #[cfg(target_arch = "x86_64")]
        core::arch::asm!("mov cr3, {}", in(reg) ade, options(nostack, preserves_flags));
        #[cfg(not(target_arch = "x86_64"))]
        crate::arch::dnj(ade);
        
        crate::log!("[EXEC] Ring 3 test exited with code {}", nz);
        ExecResult::Dx(nz)
    }
}








pub static ADJ_: &[u8] = &{
    
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


pub fn kui() -> ExecResult {
    crate::log!("[EXEC] Running embedded hello world ELF...");
    soc(ADJ_, &[])
}










pub fn nrl() -> ExecResult {
    crate::log!("[EXEC] Running v0.3 memory test in Ring 3...");

    
    
    
    
    
    
    
    
    
    
    
    let omx: [u8; 194] = [
        
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

    

    let mut ze = match AddressSpace::dtn() {
        Some(q) => q,
        None => {
            crate::log_error!("[EXEC] Failed to create address space");
            return ExecResult::Bf;
        }
    };

    let hp = lr();

    
    let bgl: u64 = 0x400000;
    let asn = match cja() {
        Some(ai) => ai,
        None => return ExecResult::Bf,
    };
    unsafe {
        let aac = (asn + hp) as *mut u8;
        core::ptr::ahx(aac, 0, 4096);
        core::ptr::copy_nonoverlapping(omx.fq(), aac, omx.len());
    }
    if ze.bnl(bgl, asn, PageFlags::JG_).is_none() {
        return ExecResult::Bf;
    }

    
    let alt: u64 = 0x7FFFFFFF0000;
    let bfl = 4;
    for a in 0..bfl {
        let uy = alt - (a as u64 + 1) * 4096;
        let ht = match cja() {
            Some(ai) => ai,
            None => return ExecResult::Bf,
        };
        unsafe { core::ptr::ahx((ht + hp) as *mut u8, 0, 4096); }
        if ze.bnl(uy, ht, PageFlags::EW_).is_none() {
            return ExecResult::Bf;
        }
    }

    let ais = alt - 8;

    crate::log!("[EXEC] memtest: code at {:#x}, stack at {:#x}", bgl, ais);

    unsafe {
        let ade: u64;
        #[cfg(target_arch = "x86_64")]
        core::arch::asm!("mov {}, cr3", bd(reg) ade);
        #[cfg(not(target_arch = "x86_64"))]
        { ade = crate::arch::dle(); }

        
        CK_.store(&mut ze as *mut AddressSpace, Ordering::Release);
        FB_.store(UserMemoryRegion::CF_, Ordering::SeqCst);
        let eiy = alt - (bfl as u64 * 4096);
        GF_.store(eiy, Ordering::SeqCst);

        ze.fci();
        let nz = crate::userland::eqa(bgl, ais);

        CK_.store(core::ptr::null_mut(), Ordering::Release);
        #[cfg(target_arch = "x86_64")]
        core::arch::asm!("mov cr3, {}", in(reg) ade, options(nostack, preserves_flags));
        #[cfg(not(target_arch = "x86_64"))]
        crate::arch::dnj(ade);

        crate::log!("[EXEC] memtest exited with code {}", nz);
        ExecResult::Dx(nz)
    }
}


fn cja() -> Option<u64> {
    crate::memory::frame::azg()
}






pub fn nrm() -> ExecResult {
    crate::log!("[EXEC] Running Ring 3 IPC pipe test...");

    let mut ze = match AddressSpace::dtn() {
        Some(q) => q,
        None => {
            crate::log_error!("[EXEC] Failed to create address space");
            return ExecResult::Bf;
        }
    };

    let hp = lr();

    
    
    
    
    
    
    
    
    
    
    
    
    
    
    
    
    
    
    
    
    
    
    
    
    
    
    
    
    
    
    
    
    
    
    
    
    
    
    
    let ovu: [u8; 138] = [
        
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

    
    let bgl: u64 = 0x400000;
    let asn = match cja() {
        Some(ai) => ai,
        None => return ExecResult::Bf,
    };
    unsafe {
        let aac = (asn + hp) as *mut u8;
        core::ptr::ahx(aac, 0, 4096);
        core::ptr::copy_nonoverlapping(ovu.fq(), aac, ovu.len());
    }
    if ze.bnl(bgl, asn, PageFlags::JG_).is_none() {
        return ExecResult::Bf;
    }

    
    let alt: u64 = 0x7FFFFFFF0000;
    let bfl = 4;
    for a in 0..bfl {
        let uy = alt - (a as u64 + 1) * 4096;
        let ht = match cja() {
            Some(ai) => ai,
            None => return ExecResult::Bf,
        };
        unsafe { core::ptr::ahx((ht + hp) as *mut u8, 0, 4096); }
        if ze.bnl(uy, ht, PageFlags::EW_).is_none() {
            return ExecResult::Bf;
        }
    }

    let ais = alt - 8;

    crate::log!("[EXEC] pipe_test: code at {:#x}, stack at {:#x}", bgl, ais);

    unsafe {
        let ade: u64;
        #[cfg(target_arch = "x86_64")]
        core::arch::asm!("mov {}, cr3", bd(reg) ade);
        #[cfg(not(target_arch = "x86_64"))]
        { ade = crate::arch::dle(); }

        CK_.store(&mut ze as *mut AddressSpace, Ordering::Release);
        FB_.store(UserMemoryRegion::CF_, Ordering::SeqCst);
        let eiy = alt - (bfl as u64 * 4096);
        GF_.store(eiy, Ordering::SeqCst);

        ze.fci();
        let nz = crate::userland::eqa(bgl, ais);

        CK_.store(core::ptr::null_mut(), Ordering::Release);
        #[cfg(target_arch = "x86_64")]
        core::arch::asm!("mov cr3, {}", in(reg) ade, options(nostack, preserves_flags));
        #[cfg(not(target_arch = "x86_64"))]
        crate::arch::dnj(ade);

        crate::log!("[EXEC] pipe_test exited with code {}", nz);
        ExecResult::Dx(nz)
    }
}








pub fn sod() -> ExecResult {
    crate::log!("[EXEC] Running exception safety test (UD2 in Ring 3)...");

    let mut ze = match AddressSpace::dtn() {
        Some(q) => q,
        None => return ExecResult::Bf,
    };
    let hp = lr();

    
    
    
    let aj: [u8; 4] = [
        0x0F, 0x0B,  
        0xEB, 0xFE,  
    ];

    let bgl: u64 = 0x400000;
    let asn = match cja() {
        Some(ai) => ai,
        None => return ExecResult::Bf,
    };
    unsafe {
        let aac = (asn + hp) as *mut u8;
        core::ptr::ahx(aac, 0, 4096);
        core::ptr::copy_nonoverlapping(aj.fq(), aac, aj.len());
    }
    if ze.bnl(bgl, asn, PageFlags::JG_).is_none() {
        return ExecResult::Bf;
    }

    let alt: u64 = 0x7FFFFFFF0000;
    let bfl = 4usize;
    for a in 0..bfl {
        let uy = alt - (a as u64 + 1) * 4096;
        let ht = match cja() { Some(ai) => ai, None => return ExecResult::Bf };
        unsafe { core::ptr::ahx((ht + hp) as *mut u8, 0, 4096); }
        if ze.bnl(uy, ht, PageFlags::EW_).is_none() {
            return ExecResult::Bf;
        }
    }
    let ais = alt - 8;

    unsafe {
        let ade: u64;
        #[cfg(target_arch = "x86_64")]
        core::arch::asm!("mov {}, cr3", bd(reg) ade);
        #[cfg(not(target_arch = "x86_64"))]
        { ade = crate::arch::dle(); }
        CK_.store(&mut ze as *mut AddressSpace, Ordering::Release);
        FB_.store(UserMemoryRegion::CF_, Ordering::SeqCst);
        GF_.store(alt - (bfl as u64 * 4096), Ordering::SeqCst);
        ze.fci();
        let nz = crate::userland::eqa(bgl, ais);
        CK_.store(core::ptr::null_mut(), Ordering::Release);
        #[cfg(target_arch = "x86_64")]
        core::arch::asm!("mov cr3, {}", in(reg) ade, options(nostack, preserves_flags));
        #[cfg(not(target_arch = "x86_64"))]
        crate::arch::dnj(ade);
        crate::log!("[EXEC] exception_safety_test exited with code {}", nz);
        ExecResult::Dx(nz)
    }
}



pub fn soh() -> ExecResult {
    crate::log!("[EXEC] Running signal syscall test...");

    let mut ze = match AddressSpace::dtn() {
        Some(q) => q,
        None => return ExecResult::Bf,
    };
    let hp = lr();

    
    
    
    
    
    
    
    
    
    #[rustfmt::chz]
    let aj: [u8; 158] = [
        
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

    let bgl: u64 = 0x400000;
    let asn = match cja() {
        Some(ai) => ai,
        None => return ExecResult::Bf,
    };
    unsafe {
        let aac = (asn + hp) as *mut u8;
        core::ptr::ahx(aac, 0, 4096);
        core::ptr::copy_nonoverlapping(aj.fq(), aac, aj.len());
    }
    if ze.bnl(bgl, asn, PageFlags::JG_).is_none() {
        return ExecResult::Bf;
    }

    let alt: u64 = 0x7FFFFFFF0000;
    let bfl = 4usize;
    for a in 0..bfl {
        let uy = alt - (a as u64 + 1) * 4096;
        let ht = match cja() { Some(ai) => ai, None => return ExecResult::Bf };
        unsafe { core::ptr::ahx((ht + hp) as *mut u8, 0, 4096); }
        if ze.bnl(uy, ht, PageFlags::EW_).is_none() {
            return ExecResult::Bf;
        }
    }
    let ais = alt - 8;

    
    let ce = crate::process::eys("signal_test").unwrap_or(0);
    let hvy = crate::process::aei();

    
    crate::vfs::mfc();

    let nz;
    unsafe {
        let ade: u64;
        #[cfg(target_arch = "x86_64")]
        core::arch::asm!("mov {}, cr3", bd(reg) ade);
        #[cfg(not(target_arch = "x86_64"))]
        { ade = crate::arch::dle(); }
        CK_.store(&mut ze as *mut AddressSpace, Ordering::Release);
        FB_.store(UserMemoryRegion::CF_, Ordering::SeqCst);
        GF_.store(alt - (bfl as u64 * 4096), Ordering::SeqCst);
        crate::process::mhj(ce);
        ze.fci();
        nz = crate::userland::eqa(bgl, ais);
        CK_.store(core::ptr::null_mut(), Ordering::Release);
        #[cfg(target_arch = "x86_64")]
        core::arch::asm!("mov cr3, {}", in(reg) ade, options(nostack, preserves_flags));
        #[cfg(not(target_arch = "x86_64"))]
        crate::arch::dnj(ade);
    }

    crate::process::eqi(ce, nz);
    crate::process::lyd(ce);
    crate::vfs::khv();
    crate::process::jos(hvy);
    crate::log!("[EXEC] signal_test exited with code {}", nz);
    ExecResult::Dx(nz)
}



pub fn soi() -> ExecResult {
    crate::log!("[EXEC] Running stdio/time test...");

    let mut ze = match AddressSpace::dtn() {
        Some(q) => q,
        None => return ExecResult::Bf,
    };
    let hp = lr();

    
    
    
    
    
    
    
    
    #[rustfmt::chz]
    let aj: [u8; 121] = [
        
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

    let bgl: u64 = 0x400000;
    let asn = match cja() {
        Some(ai) => ai,
        None => return ExecResult::Bf,
    };
    unsafe {
        let aac = (asn + hp) as *mut u8;
        core::ptr::ahx(aac, 0, 4096);
        core::ptr::copy_nonoverlapping(aj.fq(), aac, aj.len());
    }
    if ze.bnl(bgl, asn, PageFlags::JG_).is_none() {
        return ExecResult::Bf;
    }

    let alt: u64 = 0x7FFFFFFF0000;
    let bfl = 4usize;
    for a in 0..bfl {
        let uy = alt - (a as u64 + 1) * 4096;
        let ht = match cja() { Some(ai) => ai, None => return ExecResult::Bf };
        unsafe { core::ptr::ahx((ht + hp) as *mut u8, 0, 4096); }
        if ze.bnl(uy, ht, PageFlags::EW_).is_none() {
            return ExecResult::Bf;
        }
    }
    let ais = alt - 8;

    
    let ce = crate::process::eys("stdio_test").unwrap_or(0);
    let hvy = crate::process::aei();

    
    crate::vfs::mfc();

    let nz;
    unsafe {
        let ade: u64;
        #[cfg(target_arch = "x86_64")]
        core::arch::asm!("mov {}, cr3", bd(reg) ade);
        #[cfg(not(target_arch = "x86_64"))]
        { ade = crate::arch::dle(); }
        CK_.store(&mut ze as *mut AddressSpace, Ordering::Release);
        FB_.store(UserMemoryRegion::CF_, Ordering::SeqCst);
        GF_.store(alt - (bfl as u64 * 4096), Ordering::SeqCst);
        crate::process::mhj(ce);
        ze.fci();
        nz = crate::userland::eqa(bgl, ais);
        CK_.store(core::ptr::null_mut(), Ordering::Release);
        #[cfg(target_arch = "x86_64")]
        core::arch::asm!("mov cr3, {}", in(reg) ade, options(nostack, preserves_flags));
        #[cfg(not(target_arch = "x86_64"))]
        crate::arch::dnj(ade);
    }

    crate::process::eqi(ce, nz);
    crate::process::lyd(ce);
    crate::vfs::khv();
    crate::process::jos(hvy);
    crate::log!("[EXEC] stdio_test exited with code {}", nz);
    ExecResult::Dx(nz)
}


pub fn clc(path: &str) -> bool {
    let da = match crate::vfs::aji(path, crate::vfs::OpenFlags(0)) {
        Ok(da) => da,
        Err(_) => return false,
    };
    
    let mut sj = [0u8; 4];
    let result = crate::vfs::read(da, &mut sj).is_ok() 
        && sj == [0x7F, b'E', b'L', b'F'];
    
    crate::vfs::agj(da).bq();
    result
}
