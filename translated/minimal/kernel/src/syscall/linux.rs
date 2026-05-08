




use crate::memory::{ij, ux};
use crate::syscall::errno;
use alloc::string::String;
use alloc::vec::Vec;


pub mod nr {
    pub const Ba: u64 = 0;
    pub const Bh: u64 = 1;
    pub const Aby: u64 = 2;
    pub const Rf: u64 = 3;
    pub const Ael: u64 = 4;
    pub const Yq: u64 = 5;
    pub const Aas: u64 = 6;
    pub const Ace: u64 = 7;
    pub const Aaq: u64 = 8;
    pub const Tn: u64 = 9;
    pub const Abc: u64 = 10;
    pub const Tp: u64 = 11;
    pub const Rd: u64 = 12;
    pub const CTF_: u64 = 13;
    pub const CTG_: u64 = 14;
    pub const AJF_: u64 = 15;
    pub const Zy: u64 = 16;
    pub const Bai: u64 = 17;
    pub const Baq: u64 = 18;
    pub const Anr: u64 = 19;
    pub const Agm: u64 = 20;
    pub const Qy: u64 = 21;
    pub const Bad: u64 = 22;
    pub const Bbp: u64 = 23;
    pub const CUD_: u64 = 24;
    pub const Azg: u64 = 25;
    pub const Azh: u64 = 26;
    pub const Ayv: u64 = 27;
    pub const Ayq: u64 = 28;
    pub const Xo: u64 = 32;
    pub const Xp: u64 = 33;
    pub const Acb: u64 = 34;
    pub const Amv: u64 = 35;
    pub const Awk: u64 = 36;
    pub const Asm: u64 = 37;
    pub const Bby: u64 = 38;
    pub const So: u64 = 39;
    pub const Bbr: u64 = 40;
    pub const Apq: u64 = 41;
    pub const Ahu: u64 = 42;
    pub const Agt: u64 = 43;
    pub const Aov: u64 = 44;
    pub const Ans: u64 = 45;
    pub const Aou: u64 = 46;
    pub const Ant: u64 = 47;
    pub const Iu: u64 = 48;
    pub const Wm: u64 = 49;
    pub const Amh: u64 = 50;
    pub const Akp: u64 = 51;
    pub const Aki: u64 = 52;
    pub const Bct: u64 = 53;
    pub const Ape: u64 = 54;
    pub const Akq: u64 = 55;
    pub const Ahs: u64 = 56;
    pub const Yo: u64 = 57;
    pub const Arj: u64 = 58;
    pub const Yg: u64 = 59;
    pub const Oq: u64 = 60;
    pub const Arv: u64 = 61;
    pub const Ama: u64 = 62;
    pub const Vq: u64 = 63;
    pub const Ajp: u64 = 72;
    pub const Avw: u64 = 73;
    pub const Awb: u64 = 74;
    pub const Avs: u64 = 75;
    pub const Bdz: u64 = 76;
    pub const Awc: u64 = 77;
    pub const Awi: u64 = 78;
    pub const Sk: u64 = 79;
    pub const Wv: u64 = 80;
    pub const Avo: u64 = 81;
    pub const Bbf: u64 = 82;
    pub const Aba: u64 = 83;
    pub const Aoh: u64 = 84;
    pub const Atn: u64 = 85;
    pub const Ayk: u64 = 86;
    pub const Afq: u64 = 87;
    pub const Bdg: u64 = 88;
    pub const Acx: u64 = 89;
    pub const Ahp: u64 = 90;
    pub const Ajn: u64 = 91;
    pub const Ahq: u64 = 92;
    pub const Ajo: u64 = 93;
    pub const Amf: u64 = 94;
    pub const Ard: u64 = 95;
    pub const Aks: u64 = 96;
    pub const Akn: u64 = 97;
    pub const Awo: u64 = 98;
    pub const Bdk: u64 = 99;
    pub const Bdt: u64 = 100;
    pub const Sp: u64 = 102;
    pub const Bdl: u64 = 103;
    pub const Sn: u64 = 104;
    pub const Apf: u64 = 105;
    pub const Aoy: u64 = 106;
    pub const Sm: u64 = 107;
    pub const Sl: u64 = 108;
    pub const Aoz: u64 = 109;
    pub const Akl: u64 = 110;
    pub const Akk: u64 = 111;
    pub const Apd: u64 = 112;
    pub const Apb: u64 = 113;
    pub const Apa: u64 = 114;
    pub const Awj: u64 = 115;
    pub const Bbw: u64 = 116;
    pub const Bcc: u64 = 117;
    pub const Awn: u64 = 118;
    pub const Bcb: u64 = 119;
    pub const Awm: u64 = 120;
    pub const Akj: u64 = 121;
    pub const Bbv: u64 = 122;
    pub const Bbu: u64 = 123;
    pub const Ako: u64 = 124;
    pub const Atf: u64 = 125;
    pub const Atg: u64 = 126;
    pub const EGW_: u64 = 127;
    pub const EGZ_: u64 = 128;
    pub const EGX_: u64 = 129;
    pub const EGY_: u64 = 130;
    pub const Aph: u64 = 131;
    pub const Beo: u64 = 132;
    pub const Ayx: u64 = 133;
    pub const Bel: u64 = 134;
    pub const Baa: u64 = 135;
    pub const Ben: u64 = 136;
    pub const Bcy: u64 = 137;
    pub const Awa: u64 = 138;
    pub const Bdj: u64 = 139;
    pub const Awl: u64 = 140;
    pub const Bca: u64 = 141;
    pub const EHY_: u64 = 142;
    pub const EHS_: u64 = 143;
    pub const EHZ_: u64 = 144;
    pub const EHT_: u64 = 145;
    pub const EHU_: u64 = 146;
    pub const EHV_: u64 = 147;
    pub const EHW_: u64 = 148;
    pub const Ayz: u64 = 149;
    pub const Azi: u64 = 150;
    pub const Azb: u64 = 151;
    pub const Azj: u64 = 152;
    pub const Beu: u64 = 153;
    pub const DXZ_: u64 = 154;
    pub const EDM_: u64 = 155;
    pub const Anh: u64 = 157;
    pub const AAK_: u64 = 158;
    pub const Asl: u64 = 159;
    pub const Apc: u64 = 160;
    pub const Ahr: u64 = 161;
    pub const Aqc: u64 = 162;
    pub const Asj: u64 = 163;
    pub const Bcd: u64 = 164;
    pub const Aze: u64 = 165;
    pub const Bef: u64 = 166;
    pub const Apz: u64 = 167;
    pub const Apy: u64 = 168;
    pub const Um: u64 = 169;
    pub const Bbx: u64 = 170;
    pub const Bbt: u64 = 171;
    pub const Axo: u64 = 172;
    pub const Axn: u64 = 173;
    pub const Akr: u64 = 186;
    pub const Bbb: u64 = 187;
    pub const Bce: u64 = 188;
    pub const Awq: u64 = 191;
    pub const Aym: u64 = 194;
    pub const Bbe: u64 = 197;
    pub const Bdv: u64 = 200;
    pub const Bds: u64 = 201;
    pub const Ajz: u64 = 202;
    pub const CUC_: u64 = 203;
    pub const CUB_: u64 = 204;
    pub const EJV_: u64 = 205;
    pub const DVK_: u64 = 206;
    pub const DVI_: u64 = 207;
    pub const DVJ_: u64 = 208;
    pub const DVM_: u64 = 209;
    pub const DVH_: u64 = 210;
    pub const DQT_: u64 = 211;
    pub const DWL_: u64 = 212;
    pub const BWV_: u64 = 213;
    pub const EFX_: u64 = 216;
    pub const Akh: u64 = 217;
    pub const AJR_: u64 = 218;
    pub const EGF_: u64 = 219;
    pub const Bbq: u64 = 220;
    pub const Avm: u64 = 221;
    pub const ELM_: u64 = 222;
    pub const ELQ_: u64 = 223;
    pub const ELP_: u64 = 224;
    pub const ELO_: u64 = 225;
    pub const ELN_: u64 = 226;
    pub const DIT_: u64 = 227;
    pub const BPE_: u64 = 228;
    pub const DIL_: u64 = 229;
    pub const DIP_: u64 = 230;
    pub const ADN_: u64 = 231;
    pub const BXC_: u64 = 232;
    pub const BWX_: u64 = 233;
    pub const Bdr: u64 = 234;
    pub const Ber: u64 = 235;
    pub const Ayr: u64 = 237;
    pub const EJP_: u64 = 238;
    pub const DQO_: u64 = 239;
    pub const DYK_: u64 = 240;
    pub const DYN_: u64 = 241;
    pub const DYM_: u64 = 242;
    pub const DYL_: u64 = 243;
    pub const DYJ_: u64 = 244;
    pub const DYI_: u64 = 245;
    pub const DVY_: u64 = 246;
    pub const Bfk: u64 = 247;
    pub const DFZ_: u64 = 248;
    pub const EGD_: u64 = 249;
    pub const Aye: u64 = 250;
    pub const DVE_: u64 = 251;
    pub const DVD_: u64 = 252;
    pub const DUW_: u64 = 253;
    pub const DUV_: u64 = 254;
    pub const DUY_: u64 = 255;
    pub const DXU_: u64 = 256;
    pub const Abz: u64 = 257;
    pub const Ayw: u64 = 258;
    pub const Ayy: u64 = 259;
    pub const Avq: u64 = 260;
    pub const Awd: u64 = 261;
    pub const Abp: u64 = 262;
    pub const Beg: u64 = 263;
    pub const Bbg: u64 = 264;
    pub const Ayl: u64 = 265;
    pub const Bdh: u64 = 266;
    pub const Bbc: u64 = 267;
    pub const Avp: u64 = 268;
    pub const Avl: u64 = 269;
    pub const Ban: u64 = 270;
    pub const Bah: u64 = 271;
    pub const Beh: u64 = 272;
    pub const CWD_: u64 = 273;
    pub const CAF_: u64 = 274;
    pub const Bcv: u64 = 275;
    pub const Bdp: u64 = 276;
    pub const EKV_: u64 = 277;
    pub const Bff: u64 = 278;
    pub const DYB_: u64 = 279;
    pub const Bep: u64 = 280;
    pub const BXB_: u64 = 281;
    pub const Bcg: u64 = 282;
    pub const ELJ_: u64 = 283;
    pub const Ava: u64 = 284;
    pub const Avn: u64 = 285;
    pub const ELL_: u64 = 286;
    pub const ELK_: u64 = 287;
    pub const Asi: u64 = 288;
    pub const Bch: u64 = 289;
    pub const Avb: u64 = 290;
    pub const BWW_: u64 = 291;
    pub const Air: u64 = 292;
    pub const Anc: u64 = 293;
    pub const DUX_: u64 = 294;
    pub const Baj: u64 = 295;
    pub const Bar: u64 = 296;
    pub const EHA_: u64 = 297;
    pub const EDE_: u64 = 298;
    pub const Bbd: u64 = 299;
    pub const DPE_: u64 = 300;
    pub const DPF_: u64 = 301;
    pub const Ani: u64 = 302;
    pub const DYS_: u64 = 303;
    pub const EBB_: u64 = 304;
    pub const DIJ_: u64 = 305;
    pub const Bdi: u64 = 306;
    pub const Bbs: u64 = 307;
    pub const Bbz: u64 = 308;
    pub const Awh: u64 = 309;
    pub const EEU_: u64 = 310;
    pub const EEV_: u64 = 311;
    pub const Ayb: u64 = 312;
    pub const DPT_: u64 = 313;
    pub const EHX_: u64 = 314;
    pub const EHR_: u64 = 315;
    pub const Bbh: u64 = 316;
    pub const Bbm: u64 = 317;
    pub const Akm: u64 = 318;
    pub const DXL_: u64 = 319;
    pub const DVX_: u64 = 320;
    pub const Asz: u64 = 321;
    pub const Ave: u64 = 322;
    pub const Bem: u64 = 323;
    pub const Ayt: u64 = 324;
    pub const Aza: u64 = 325;
    pub const DKP_: u64 = 326;
    pub const Bak: u64 = 327;
    pub const Bas: u64 = 328;
    pub const EDQ_: u64 = 329;
    pub const EDO_: u64 = 330;
    pub const EDP_: u64 = 331;
    pub const Bcz: u64 = 332;
}






pub mod mmap_flags {
    pub const CIK_: u64 = 0x01;
    pub const BBH_: u64 = 0x02;
    pub const CIJ_: u64 = 0x10;
    pub const AGM_: u64 = 0x20;
    pub const DWO_: u64 = 0x100;
    pub const DWM_: u64 = 0x800;
    pub const DWN_: u64 = 0x1000;
    pub const DWQ_: u64 = 0x2000;
    pub const DWS_: u64 = 0x4000;
    pub const DWT_: u64 = 0x8000;
    pub const DWR_: u64 = 0x10000;
    pub const DWU_: u64 = 0x20000;
    pub const DWP_: u64 = 0x40000;
}


pub mod prot_flags {
    pub const COE_: u64 = 0x0;
    pub const COF_: u64 = 0x1;
    pub const XT_: u64 = 0x2;
    pub const AIL_: u64 = 0x4;
}

use spin::Mutex;
use core::sync::atomic::{AtomicU64, Ordering};


static EEW_: AtomicU64 = AtomicU64::new(0);


static CLA_: AtomicU64 = AtomicU64::new(0x4000_0000); 





pub fn fcb(addr: u64, length: u64, prot: u64, flags: u64, fd: i64, bkm: u64) -> i64 {
    use mmap_flags::*;
    use prot_flags::*;
    use crate::memory::paging::PageFlags;
    
    if length == 0 {
        return errno::Bw;
    }
    
    let xy = 4096u64;
    let bxj = (length + xy - 1) & !(xy - 1);
    
    
    let bug = if addr != 0 && (flags & CIJ_) != 0 {
        let asw = addr & !(xy - 1); 
        
        if !crate::memory::ux(asw) {
            return errno::Bw;
        }
        asw
    } else {
        
        CLA_.fetch_add(bxj, Ordering::SeqCst)
    };
    
    
    let mrx = (flags & AGM_) != 0 || fd < 0;
    if !mrx {
        crate::log_debug!("[MMAP] File-backed mmap not yet implemented");
        return errno::Gk;
    }
    
    
    let cr3: u64;
    #[cfg(target_arch = "x86_64")]
    unsafe { core::arch::asm!("mov {}, cr3", out(reg) cr3, options(nostack, preserves_flags)); }
    #[cfg(not(target_arch = "x86_64"))]
    { cr3 = 0; }
    
    let pso = (prot & 0x7) as u32; 
    
    crate::memory::vma::jty(cr3, crate::memory::vma::He {
        start: bug,
        end: bug + bxj,
        prot: pso,
        flags: crate::memory::vma::flags::AGM_ | crate::memory::vma::flags::BBH_,
    });
    
    crate::log_debug!("[MMAP] Lazy VMA {:#x}..{:#x} prot={:#x}", bug, bug + bxj, prot);
    bug as i64
}


pub fn fcc(addr: u64, length: u64) -> i64 {
    if addr == 0 || length == 0 {
        return errno::Bw;
    }
    
    let xy = 4096u64;
    let bxj = (length + xy - 1) & !(xy - 1);
    let bnw = (bxj / xy) as usize;
    let start = addr & !(xy - 1);
    
    
    let cr3: u64;
    #[cfg(target_arch = "x86_64")]
    unsafe { core::arch::asm!("mov {}, cr3", out(reg) cr3, options(nostack, preserves_flags)); }
    #[cfg(not(target_arch = "x86_64"))]
    { cr3 = 0; }
    crate::memory::vma::ofc(cr3, start, start + bxj);
    
    
    crate::exec::ffh(|space| {
        for i in 0..bnw {
            let virt = start + (i as u64 * xy);
            if let Some(phys) = space.translate(virt) {
                let coa = phys & !0xFFF;
                space.unmap_page(virt);
                crate::memory::frame::vk(coa);
            }
        }
    });
    
    crate::log_debug!("[MUNMAP] Unmapped {} pages at {:#x}", bnw, addr);
    0
}


pub fn pax(addr: u64, length: u64, prot: u64) -> i64 {
    use prot_flags::*;
    use crate::memory::paging::{PageFlags, PageTable};
    
    if addr == 0 || addr & 0xFFF != 0 {
        return errno::Bw;
    }
    
    let xy = 4096u64;
    let bxj = (length + xy - 1) & !(xy - 1);
    let bnw = (bxj / xy) as usize;
    
    
    let mut ccq = PageFlags::Bg | PageFlags::Cz;
    if (prot & XT_) != 0 {
        ccq |= PageFlags::Cg;
    }
    if (prot & AIL_) == 0 {
        ccq |= PageFlags::DT_;
    }
    let cna = PageFlags::new(ccq);
    
    crate::exec::ffh(|space| {
        let bz = crate::memory::hhdm_offset();
        let cr3 = space.cr3();
        
        for i in 0..bnw {
            let virt = addr + (i as u64 * xy);
            let lu = ((virt >> 39) & 0x1FF) as usize;
            let jc = ((virt >> 30) & 0x1FF) as usize;
            let iw   = ((virt >> 21) & 0x1FF) as usize;
            let mw   = ((virt >> 12) & 0x1FF) as usize;
            
            let pml4 = unsafe { &*((cr3 + bz) as *const PageTable) };
            if !pml4.entries[lu].is_present() { continue; }
            let jt = unsafe { &*((pml4.entries[lu].phys_addr() + bz) as *const PageTable) };
            if !jt.entries[jc].is_present() { continue; }
            let js = unsafe { &*((jt.entries[jc].phys_addr() + bz) as *const PageTable) };
            if !js.entries[iw].is_present() { continue; }
            let jd = unsafe { &mut *((js.entries[iw].phys_addr() + bz) as *mut PageTable) };
            if !jd.entries[mw].is_present() { continue; }
            
            let phys = jd.entries[mw].phys_addr();
            jd.entries[mw].set(phys, cna);
            #[cfg(target_arch = "x86_64")]
            unsafe { core::arch::asm!("invlpg [{}]", in(reg) virt, options(nostack, preserves_flags)); }
        }
    });
    
    crate::log_debug!("[MPROTECT] addr={:#x} len={:#x} prot={:#x}", addr, length, prot);
    0
}




pub fn fbz(addr: u64) -> i64 {
    use crate::memory::paging::{PageFlags, UserMemoryRegion};
    
    let bfr = crate::exec::bfr();
    
    if addr == 0 || bfr == 0 {
        
        if bfr == 0 {
            return UserMemoryRegion::CH_ as i64;
        }
        return bfr as i64;
    }
    
    
    if addr < UserMemoryRegion::CH_ || addr >= UserMemoryRegion::CCL_ {
        return bfr as i64;
    }
    
    let xy = 4096u64;
    
    if addr > bfr {
        
        let gkp = (bfr + xy - 1) & !(xy - 1); 
        let gje = (addr + xy - 1) & !(xy - 1);        
        
        if gje > gkp {
            let boc = ((gje - gkp) / xy) as usize;
            
            let ok = crate::exec::ffh(|space| {
                for i in 0..boc {
                    let virt = gkp + (i as u64 * xy);
                    let phys = match crate::memory::frame::aan() {
                        Some(aa) => aa,
                        None => return false,
                    };
                    if space.map_page(virt, phys, PageFlags::FM_).is_none() {
                        return false;
                    }
                }
                true
            });
            
            if ok != Some(true) {
                return bfr as i64; 
            }
        }
    }
    
    
    
    crate::exec::oor(addr);
    crate::log_debug!("[BRK] Set program break to {:#x}", addr);
    addr as i64
}






pub fn pam() -> i64 {
    crate::process::pe() as i64
}


pub fn pao() -> i64 {
    crate::process::pux(|aa| aa.ppid as i64)
        .unwrap_or(0)
}


pub fn jkt() -> i64 {
    crate::thread::current_tid() as i64
}


pub fn pau() -> i64 {
    let (uid, _, _, _) = crate::process::bfs();
    uid as i64
}


pub fn pai() -> i64 {
    let (_, gid, _, _) = crate::process::bfs();
    gid as i64
}


pub fn qyn() -> i64 {
    let (_, _, euid, _) = crate::process::bfs();
    euid as i64
}


pub fn qym() -> i64 {
    let (_, _, _, egid) = crate::process::bfs();
    egid as i64
}


pub fn pbt(uid: u32) -> i64 {
    let pid = crate::process::pe();
    match crate::process::jfm(pid, uid) {
        Ok(()) => 0,
        Err(_) => -1, 
    }
}


pub fn pbn(gid: u32) -> i64 {
    let pid = crate::process::pe();
    match crate::process::jff(pid, gid) {
        Ok(()) => 0,
        Err(_) => -1, 
    }
}


pub fn pbq(ruid: u32, euid: u32) -> i64 {
    let pid = crate::process::pe();
    
    if ruid != 0xFFFFFFFF {
        if crate::process::jfm(pid, ruid).is_err() { return -1; }
    }
    if euid != 0xFFFFFFFF {
        
        let mut bs = crate::process::AE_.write();
        if let Some(aa) = bs.processes.get_mut(&pid) {
            if aa.euid == 0 || euid == aa.uid || euid == aa.euid {
                aa.euid = euid;
            } else {
                return -1;
            }
        }
    }
    0
}


pub fn pbp(rgid: u32, egid: u32) -> i64 {
    let pid = crate::process::pe();
    if rgid != 0xFFFFFFFF {
        if crate::process::jff(pid, rgid).is_err() { return -1; }
    }
    if egid != 0xFFFFFFFF {
        let mut bs = crate::process::AE_.write();
        if let Some(aa) = bs.processes.get_mut(&pid) {
            if aa.euid == 0 || egid == aa.gid || egid == aa.egid {
                aa.egid = egid;
            } else {
                return -1;
            }
        }
    }
    0
}


pub fn pbz(mask: u32) -> i64 {
    let pid = crate::process::pe();
    crate::process::opt(pid, mask) as i64
}






pub fn pbo(pid: u32, pgid: u32) -> i64 {
    match crate::process::oph(pid, pgid) {
        Ok(()) => 0,
        Err(_) => -1,
    }
}


pub fn pal() -> i64 {
    crate::process::ibs(0) as i64
}


pub fn pbr() -> i64 {
    match crate::process::opy() {
        Ok(sid) => sid as i64,
        Err(_) => -1,
    }
}


pub fn pak(pid: u32) -> i64 {
    crate::process::ibs(pid) as i64
}


pub fn pap(pid: u32) -> i64 {
    crate::process::ibv(pid) as i64
}


pub fn ozs(path_ptr: u64) -> i64 {
    let path = match bdf(path_ptr, 256) {
        Some(j) => j,
        None => return -14, 
    };
    let pid = crate::process::pe();
    match crate::process::kkk(pid, &path) {
        Ok(()) => 0,
        Err(_) => -1, 
    }
}


pub fn ozr(path_ptr: u64, mode: u32) -> i64 {
    let path = match bdf(path_ptr, 256) {
        Some(aa) => aa,
        None => return -14, 
    };
    match crate::vfs::kkf(&path, mode) {
        Ok(()) => 0,
        Err(_) => -1, 
    }
}


pub fn pab(fd: i32, mode: u32) -> i64 {
    match crate::vfs::lum(fd, mode) {
        Ok(()) => 0,
        Err(_) => -1,
    }
}


pub fn jkn(path_ptr: u64, uid: u32, gid: u32) -> i64 {
    let path = match bdf(path_ptr, 256) {
        Some(aa) => aa,
        None => return -14,
    };
    
    let (_, _, euid, _) = crate::process::bfs();
    if euid != 0 { return -1; } 
    match crate::vfs::kkh(&path, uid, gid) {
        Ok(()) => 0,
        Err(_) => -1,
    }
}


pub fn pac(fd: i32, uid: u32, gid: u32) -> i64 {
    let (_, _, euid, _) = crate::process::bfs();
    if euid != 0 { return -1; }
    match crate::vfs::luo(fd, uid, gid) {
        Ok(()) => 0,
        Err(_) => -1,
    }
}






pub mod arch_prctl_codes {
    pub const AAL_: u64 = 0x1001;
    pub const MQ_: u64 = 0x1002;
    pub const MP_: u64 = 0x1003;
    pub const AAJ_: u64 = 0x1004;
}


static BJL_: AtomicU64 = AtomicU64::new(0);


pub fn gwz(code: u64, addr: u64) -> i64 {
    use arch_prctl_codes::*;
    
    match code {
        MQ_ => {
            
            BJL_.store(addr, Ordering::SeqCst);
            
            
            #[cfg(target_arch = "x86_64")]
            unsafe {
                
                core::arch::asm!(
                    "wrmsr",
                    in("ecx") 0xC0000100u32,
                    in("eax") (addr as u32),
                    in("edx") ((addr >> 32) as u32),
                );
            }
            crate::log_debug!("[ARCH_PRCTL] Set FS base to {:#x}", addr);
            0
        }
        AAL_ => {
            
            #[cfg(target_arch = "x86_64")]
            unsafe {
                
                core::arch::asm!(
                    "wrmsr",
                    in("ecx") 0xC0000101u32,
                    in("eax") (addr as u32),
                    in("edx") ((addr >> 32) as u32),
                );
            }
            0
        }
        MP_ => {
            if !ux(addr) {
                return errno::P;
            }
            let val = BJL_.load(Ordering::SeqCst);
            unsafe { *(addr as *mut u64) = val; }
            0
        }
        AAJ_ => {
            if !ux(addr) {
                return errno::P;
            }
            let val: u64;
            #[cfg(target_arch = "x86_64")]
            unsafe {
                core::arch::asm!(
                    "rdmsr",
                    in("ecx") 0xC0000101u32,
                    out("eax") _,
                    out("edx") _,
                );
                
                val = 0;
            }
            #[cfg(not(target_arch = "x86_64"))]
            { val = 0; }
            unsafe { *(addr as *mut u64) = val; }
            0
        }
        _ => errno::Bw,
    }
}





static BPD_: AtomicU64 = AtomicU64::new(0);


pub fn pbm(tidptr: u64) -> i64 {
    BPD_.store(tidptr, Ordering::SeqCst);
    jkt()
}






#[repr(C)]
pub struct Ns {
    pub sysname: [u8; 65],
    pub nodename: [u8; 65],
    pub release: [u8; 65],
    pub version: [u8; 65],
    pub machine: [u8; 65],
    pub domainname: [u8; 65],
}


pub fn gxm(buf: u64) -> i64 {
    if !ij(buf, core::mem::size_of::<Ns>(), true) {
        return errno::P;
    }
    
    let asq = unsafe { &mut *(buf as *mut Ns) };
    
    
    *asq = Ns {
        sysname: [0; 65],
        nodename: [0; 65],
        release: [0; 65],
        version: [0; 65],
        machine: [0; 65],
        domainname: [0; 65],
    };
    
    
    cvq(&mut asq.sysname, "TrustOS");
    cvq(&mut asq.nodename, "trustos");
    cvq(&mut asq.release, "1.0.0-trustos");
    cvq(&mut asq.version, "#1 SMP PREEMPT TrustOS");
    cvq(&mut asq.machine, "x86_64");
    cvq(&mut asq.domainname, "(none)");
    
    0
}

fn cvq(ik: &mut [u8; 65], j: &str) {
    let bytes = j.as_bytes();
    let len = bytes.len().min(64);
    ik[..len].copy_from_slice(&bytes[..len]);
}






#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct Fe {
    pub tv_sec: i64,
    pub tv_nsec: i64,
}


#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct Afj {
    pub tv_sec: i64,
    pub tv_usec: i64,
}


pub mod clock_ids {
    pub const DIR_: u32 = 0;
    pub const DIM_: u32 = 1;
    pub const DIQ_: u32 = 2;
    pub const DIU_: u32 = 3;
    pub const DIO_: u32 = 4;
    pub const DIS_: u32 = 5;
    pub const DIN_: u32 = 6;
    pub const DIK_: u32 = 7;
}


pub fn gxb(clock_id: u32, tp: u64) -> i64 {
    if !ij(tp, core::mem::size_of::<Fe>(), true) {
        return errno::P;
    }
    
    let gx = crate::time::yf();
    let abi = gx / 1000;
    let bul = (gx % 1000) * 1_000_000;
    
    let jy = unsafe { &mut *(tp as *mut Fe) };
    jy.tv_sec = abi as i64;
    jy.tv_nsec = bul as i64;
    
    0
}


pub fn pas(csb: u64, edb: u64) -> i64 {
    if csb != 0 {
        if !ij(csb, core::mem::size_of::<Afj>(), true) {
            return errno::P;
        }
        
        let gx = crate::time::yf();
        let abi = gx / 1000;
        let pql = (gx % 1000) * 1000;
        
        let jmt = unsafe { &mut *(csb as *mut Afj) };
        jmt.tv_sec = abi as i64;
        jmt.tv_usec = pql as i64;
    }
    
    
    0
}


pub fn gxj(bvk: u64, rem: u64) -> i64 {
    if !ij(bvk, core::mem::size_of::<Fe>(), false) {
        return errno::P;
    }
    
    let jy = unsafe { &*(bvk as *const Fe) };
    let dh = (jy.tv_sec * 1000 + jy.tv_nsec / 1_000_000) as u64;
    
    
    let start = crate::time::yf();
    while crate::time::yf().saturating_sub(start) < dh {
        crate::thread::ajc();
    }
    
    if rem != 0 && ij(rem, core::mem::size_of::<Fe>(), true) {
        let izl = unsafe { &mut *(rem as *mut Fe) };
        izl.tv_sec = 0;
        izl.tv_nsec = 0;
    }
    
    0
}






pub fn gxf(buf: u64, count: u64, bej: u64) -> i64 {
    if !ij(buf, count as usize, true) {
        return errno::P;
    }
    
    let buffer = unsafe { core::slice::from_raw_parts_mut(buf as *mut u8, count as usize) };
    
    
    for byte in buffer.iter_mut() {
        *byte = crate::rng::ixv();
    }
    
    count as i64
}






pub fn gxg(fd: i32, request: u64, db: u64) -> i64 {
    
    const Nk: u64 = 0x5401;
    const Vi: u64 = 0x5402;
    const Nl: u64 = 0x5413;
    const Aff: u64 = 0x5414;
    const Aju: u64 = 0x541B;
    
    match request {
        Nk | Vi => {
            
            0
        }
        Nl => {
            
            if db != 0 && ij(db, 8, true) {
                let winsize = unsafe { &mut *(db as *mut [u16; 4]) };
                winsize[0] = 25;  
                winsize[1] = 80;  
                winsize[2] = 0;   
                winsize[3] = 0;   
            }
            0
        }
        Aju => {
            
            if db != 0 && ij(db, 4, true) {
                unsafe { *(db as *mut i32) = 0; }
            }
            0
        }
        _ => {
            crate::log_debug!("[IOCTL] Unknown ioctl fd={} request={:#x}", fd, request);
            0 
        }
    }
}


pub fn pae(fd: i32, cmd: u32, db: u64) -> i64 {
    use alloc::collections::BTreeMap;
    use spin::Mutex;

    const BZL_: u32 = 0;
    const BZM_: u32 = 1;
    const BZO_: u32 = 2;
    const BZN_: u32 = 3;
    const BZP_: u32 = 4;
    const AUT_: u32 = 0x406;

    
    static NX_: Mutex<BTreeMap<(u32, i32), (u32, u32)>> = Mutex::new(BTreeMap::new());

    let pid = crate::process::pe();
    let key = (pid, fd);

    match cmd {
        BZL_ | AUT_ => {
            match crate::vfs::ftn(fd) {
                Ok(ue) => {
                    if cmd == AUT_ {
                        let mut flags = NX_.lock();
                        flags.insert((pid, ue), (1, 0)); 
                    }
                    ue as i64
                }
                Err(_) => -9, 
            }
        }
        BZM_ => {
            let flags = NX_.lock();
            flags.get(&key).map(|f| f.0 as i64).unwrap_or(0)
        }
        BZO_ => {
            let mut flags = NX_.lock();
            let entry = flags.entry(key).or_insert((0, 0));
            entry.0 = db as u32;
            0
        }
        BZN_ => {
            let flags = NX_.lock();
            flags.get(&key).map(|f| f.1 as i64).unwrap_or(0)
        }
        BZP_ => {
            
            let bxl = 0x400 | 0x800 | 0x2000;
            let mut flags = NX_.lock();
            let entry = flags.entry(key).or_insert((0, 0));
            entry.1 = (entry.1 & !bxl) | (db as u32 & bxl);
            0
        }
        _ => {
            crate::log_debug!("[FCNTL] fd={} cmd={:#x} arg={}", fd, cmd, db);
            0
        }
    }
}


#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct Stat {
    pub st_dev: u64,
    pub st_ino: u64,
    pub st_nlink: u64,
    pub st_mode: u32,
    pub st_uid: u32,
    pub st_gid: u32,
    pub _pad0: u32,
    pub st_rdev: u64,
    pub st_size: i64,
    pub st_blksize: i64,
    pub st_blocks: i64,
    pub st_atime: i64,
    pub st_atime_nsec: i64,
    pub st_mtime: i64,
    pub st_mtime_nsec: i64,
    pub st_ctime: i64,
    pub st_ctime_nsec: i64,
    pub _unused: [i64; 3],
}


pub mod stat_mode {
    pub const BJA_: u32 = 0o170000;
    pub const QS_: u32 = 0o100000;
    pub const AKJ_: u32 = 0o040000;
    pub const YX_: u32 = 0o020000;
    pub const DAX_: u32 = 0o010000;
    pub const AKK_: u32 = 0o120000;
    pub const DAY_: u32 = 0o140000;
}


fn lvf(qk: crate::vfs::FileType) -> u32 {
    match qk {
        crate::vfs::FileType::Regular    => stat_mode::QS_,
        crate::vfs::FileType::Directory  => stat_mode::AKJ_,
        crate::vfs::FileType::CharDevice => stat_mode::YX_,
        crate::vfs::FileType::Ak => 0o060000, 
        crate::vfs::FileType::Symlink    => stat_mode::AKK_,
        crate::vfs::FileType::Pipe       => stat_mode::DAX_,
        crate::vfs::FileType::Socket     => stat_mode::DAY_,
    }
}


fn jqc(vfs: &crate::vfs::Stat) -> Stat {
    Stat {
        st_dev: 1,
        st_ino: vfs.ino,
        st_nlink: 1,
        st_mode: lvf(vfs.file_type) | (vfs.mode & 0o7777),
        st_uid: vfs.uid,
        st_gid: vfs.gid,
        _pad0: 0,
        st_rdev: 0,
        st_size: vfs.size as i64,
        st_blksize: vfs.block_size as i64,
        st_blocks: ((vfs.size + 511) / 512) as i64,
        st_atime: vfs.atime as i64,
        st_atime_nsec: 0,
        st_mtime: vfs.mtime as i64,
        st_mtime_nsec: 0,
        st_ctime: vfs.ctime as i64,
        st_ctime_nsec: 0,
        _unused: [0; 3],
    }
}


pub fn jkr(fd: i32, statbuf: u64) -> i64 {
    if !ij(statbuf, core::mem::size_of::<Stat>(), true) {
        return errno::P;
    }
    
    let stat = unsafe { &mut *(statbuf as *mut Stat) };
    
    
    if fd >= 0 && fd <= 2 {
        *stat = Stat::default();
        stat.st_mode = stat_mode::YX_ | 0o666;
        stat.st_rdev = 0x0500; 
        stat.st_blksize = 4096;
        return 0;
    }
    
    
    match crate::vfs::lzw(fd) {
        Ok(vfs_stat) => {
            *stat = jqc(&vfs_stat);
            0
        }
        Err(_) => errno::Cp,
    }
}


pub fn eav(pathname: u64, statbuf: u64) -> i64 {
    let path = match bdf(pathname, 4096) {
        Some(j) => j,
        None => return errno::P,
    };
    
    if !ij(statbuf, core::mem::size_of::<Stat>(), true) {
        return errno::P;
    }
    
    let stat = unsafe { &mut *(statbuf as *mut Stat) };
    
    match crate::vfs::stat(&path) {
        Ok(vfs_stat) => {
            *stat = jqc(&vfs_stat);
            0
        }
        Err(_) => errno::Do,
    }
}


pub fn pay(dirfd: i32, pathname: u64, statbuf: u64, bej: u32) -> i64 {
    const AAM_: i32 = -100;
    const BNG_: u32 = 0x1000;
    
    
    if bej & BNG_ != 0 {
        let path = bdf(pathname, 4096);
        if path.is_none() || path.as_ref().map_or(false, |j| j.is_empty()) {
            if dirfd >= 0 {
                return jkr(dirfd, statbuf);
            }
        }
    }
    
    let path = match bdf(pathname, 4096) {
        Some(j) => j,
        None => return errno::P,
    };
    
    
    if path.starts_with('/') || dirfd == AAM_ {
        return eav(pathname, statbuf);
    }
    
    
    eav(pathname, statbuf)
}


pub fn gwy(pathname: u64, mode: u32) -> i64 {
    let jsq = match bdf(pathname, 256) {
        Some(j) => j,
        None => return errno::P,
    };
    
    
    
    0
}


pub fn pbd(pathname: u64, buf: u64, bufsiz: u64) -> i64 {
    let path = match bdf(pathname, 256) {
        Some(j) => j,
        None => return errno::P,
    };
    
    
    if path == "/proc/self/exe" {
        let hwu = "/bin/program";
        let len = hwu.len().min(bufsiz as usize);
        if ij(buf, len, true) {
            let dst = unsafe { core::slice::from_raw_parts_mut(buf as *mut u8, len) };
            dst.copy_from_slice(&hwu.as_bytes()[..len]);
            return len as i64;
        }
    }
    
    errno::Bw
}






pub fn pbg(sig: u32, act: u64, oldact: u64, sigsetsize: u64) -> i64 {
    let pid = crate::process::pe();
    crate::log_debug!("[SIGACTION] pid={} sig={} act={:#x} oldact={:#x}", pid, sig, act, oldact);

    
    if oldact != 0 && ij(oldact, core::mem::size_of::<crate::signals::SigAction>(), true) {
        if let Ok(qb) = crate::signals::get_action(pid, sig) {
            unsafe {
                core::ptr::write(oldact as *mut crate::signals::SigAction, qb);
            }
        }
    }

    
    if act != 0 && ij(act, core::mem::size_of::<crate::signals::SigAction>(), false) {
        let nip = unsafe { core::ptr::read(act as *const crate::signals::SigAction) };
        if let Err(e) = crate::signals::set_action(pid, sig, nip) {
            return e as i64;
        }
    }

    0
}


pub fn pbh(how: u32, set: u64, oldset: u64, sigsetsize: u64) -> i64 {
    let pid = crate::process::pe();

    let mut dvu: u64 = 0;
    let njr = if set != 0 && ij(set, 8, false) {
        unsafe { core::ptr::read(set as *const u64) }
    } else {
        0
    };

    if let Err(e) = crate::signals::ope(pid, how, njr, &mut dvu) {
        return e as i64;
    }

    
    if oldset != 0 && ij(oldset, sigsetsize as usize, true) {
        unsafe {
            core::ptr::write(oldset as *mut u64, dvu);
        }
    }

    0
}






#[repr(C)]
#[derive(Clone, Copy)]
pub struct Ado {
    pub rlim_cur: u64,
    pub rlim_max: u64,
}


pub mod rlimit_resource {
    pub const EGK_: u32 = 0;
    pub const EGL_: u32 = 1;
    pub const CSN_: u32 = 2;
    pub const CSP_: u32 = 3;
    pub const EGJ_: u32 = 4;
    pub const EGR_: u32 = 5;
    pub const EGQ_: u32 = 6;
    pub const CSO_: u32 = 7;
    pub const EGN_: u32 = 8;
    pub const CSM_: u32 = 9;
    pub const EGM_: u32 = 10;
    pub const EGU_: u32 = 11;
    pub const EGO_: u32 = 12;
    pub const EGP_: u32 = 13;
    pub const EGS_: u32 = 14;
    pub const EGT_: u32 = 15;
}

const QD_: u64 = !0;


pub fn jks(resource: u32, rlim: u64) -> i64 {
    if !ij(rlim, core::mem::size_of::<Ado>(), true) {
        return errno::P;
    }
    
    let jm = unsafe { &mut *(rlim as *mut Ado) };
    
    use rlimit_resource::*;
    match resource {
        CSP_ => {
            jm.rlim_cur = 8 * 1024 * 1024; 
            jm.rlim_max = QD_;
        }
        CSO_ => {
            jm.rlim_cur = 1024;
            jm.rlim_max = 1024 * 1024;
        }
        CSM_ | CSN_ => {
            jm.rlim_cur = QD_;
            jm.rlim_max = QD_;
        }
        _ => {
            jm.rlim_cur = QD_;
            jm.rlim_max = QD_;
        }
    }
    
    0
}


pub fn pbc(pid: u32, resource: u32, new_limit: u64, old_limit: u64) -> i64 {
    if old_limit != 0 {
        jks(resource, old_limit)
    } else {
        0
    }
}






pub fn qyl(status: i32) -> i64 {
    crate::log!("[EXIT_GROUP] status={}", status);
    crate::process::exit(status);
    0 
}


pub fn pbl(su: u64, len: u64) -> i64 {
    
    0
}


pub fn pag(pid: u32, head_ptr: u64, len_ptr: u64) -> i64 {
    0
}


pub fn pbb(option: u32, arg2: u64, aer: u64, cfw: u64, dhv: u64) -> i64 {
    const COH_: u32 = 15;
    const COG_: u32 = 16;
    
    match option {
        COH_ => {
            
            if let Some(name) = bdf(arg2, 16) {
                crate::log_debug!("[PRCTL] Set thread name: {}", name);
            }
            0
        }
        COG_ => {
            
            if ij(arg2, 16, true) {
                kxu(arg2, "trustos", 16);
            }
            0
        }
        _ => {
            crate::log_debug!("[PRCTL] Unknown option {}", option);
            0
        }
    }
}


pub fn pbj() -> i64 {
    crate::scheduler::dgw();
    0
}


pub fn pbi(pid: u32, cpusetsize: u64, mask: u64) -> i64 {
    if mask != 0 && ij(mask, cpusetsize as usize, true) {
        
        unsafe {
            core::ptr::write_bytes(mask as *mut u8, 0xFF, cpusetsize as usize);
        }
    }
    0
}






pub fn bdf(ptr: u64, max: usize) -> Option<String> {
    if ptr == 0 || !ux(ptr) {
        return None;
    }
    
    let mut j = String::new();
    for i in 0..max {
        let dkc = ptr + i as u64;
        if !ux(dkc) {
            return None;
        }
        
        let b = unsafe { *(dkc as *const u8) };
        if b == 0 { break; }
        j.push(b as char);
    }
    
    if j.is_empty() { None } else { Some(j) }
}


fn kxu(ptr: u64, j: &str, max: usize) {
    let len = j.len().min(max - 1);
    let dst = unsafe { core::slice::from_raw_parts_mut(ptr as *mut u8, max) };
    dst[..len].copy_from_slice(&j.as_bytes()[..len]);
    dst[len] = 0; 
}






#[repr(C)]
#[derive(Clone, Copy)]
pub struct Pe {
    pub iov_base: u64,
    pub iov_len: u64,
}


pub fn pcb(fd: i32, iov: u64, iovcnt: u32) -> i64 {
    if !ij(iov, (iovcnt as usize) * core::mem::size_of::<Pe>(), false) {
        return errno::P;
    }
    
    let gdm = unsafe { core::slice::from_raw_parts(iov as *const Pe, iovcnt as usize) };
    let mut av = 0i64;
    
    for iovec in gdm {
        if iovec.iov_len == 0 {
            continue;
        }
        if !ij(iovec.iov_base, iovec.iov_len as usize, false) {
            return errno::P;
        }
        
        let buf = unsafe { core::slice::from_raw_parts(iovec.iov_base as *const u8, iovec.iov_len as usize) };
        
        
        if fd == 1 || fd == 2 {
            for &b in buf {
                crate::serial_print!("{}", b as char);
            }
            av += iovec.iov_len as i64;
        } else {
            match crate::vfs::write(fd, buf) {
                Ok(ae) => av += ae as i64,
                Err(_) => return if av > 0 { av } else { errno::Lp },
            }
        }
    }
    
    av
}


pub fn pbe(fd: i32, iov: u64, iovcnt: u32) -> i64 {
    if !ij(iov, (iovcnt as usize) * core::mem::size_of::<Pe>(), false) {
        return errno::P;
    }
    
    let gdm = unsafe { core::slice::from_raw_parts(iov as *const Pe, iovcnt as usize) };
    let mut av = 0i64;
    
    for iovec in gdm {
        if iovec.iov_len == 0 {
            continue;
        }
        if !ij(iovec.iov_base, iovec.iov_len as usize, true) {
            return errno::P;
        }
        
        let buf = unsafe { core::slice::from_raw_parts_mut(iovec.iov_base as *mut u8, iovec.iov_len as usize) };
        
        match crate::vfs::read(fd, buf) {
            Ok(ae) => {
                av += ae as i64;
                if ae < iovec.iov_len as usize {
                    break; 
                }
            }
            Err(_) => return if av > 0 { av } else { errno::Lp },
        }
    }
    
    av
}






pub fn gxd(old_fd: i32) -> i64 {
    if crate::pipe::dab(old_fd) {
        return old_fd as i64; 
    }
    match crate::vfs::ftn(old_fd) {
        Ok(ue) => ue as i64,
        Err(_) => errno::Cp,
    }
}


pub fn fca(old_fd: i32, ue: i32) -> i64 {
    if old_fd == ue {
        return ue as i64;
    }
    if crate::pipe::dab(old_fd) {
        return old_fd as i64; 
    }
    match crate::vfs::hui(old_fd, ue) {
        Ok(fd) => fd as i64,
        Err(_) => errno::Cp,
    }
}






#[repr(C)]
#[derive(Clone, Copy)]
pub struct Acr {
    pub fd: i32,
    pub events: i16,
    pub revents: i16,
}

const Acf: i16 = 1;
const Acg: i16 = 4;
const Ane: i16 = 8;
const Anf: i16 = 16;
#[allow(dead_code)]
const Ang: i16 = 32;


pub fn jku(fds_ptr: u64, nfds: u32, timeout_ms: i32) -> i64 {
    if nfds == 0 { return 0; }
    let size = (nfds as usize) * core::mem::size_of::<Acr>();
    if !ij(fds_ptr, size, true) {
        return errno::P;
    }
    
    let fds = unsafe { core::slice::from_raw_parts_mut(fds_ptr as *mut Acr, nfds as usize) };
    
    
    let brr = if timeout_ms < 0 {
        u64::MAX 
    } else if timeout_ms == 0 {
        0 
    } else {
        crate::time::cbx().saturating_add((timeout_ms as u64) * 1_000_000)
    };
    
    loop {
        let mut ready = 0i64;
        for pfd in fds.iter_mut() {
            pfd.revents = 0;
            if pfd.fd < 0 { continue; }
            
            if let Some(status) = crate::vfs::ivm(pfd.fd) {
                if (pfd.events & Acf)  != 0 && status.readable { pfd.revents |= Acf; }
                if (pfd.events & Acg) != 0 && status.writable { pfd.revents |= Acg; }
                if status.error  { pfd.revents |= Ane; }
                if status.hangup { pfd.revents |= Anf; }
            } else {
                pfd.revents = Ang; 
            }
            
            if pfd.revents != 0 { ready += 1; }
        }
        
        if ready > 0 { return ready; }
        
        
        if timeout_ms == 0 { return 0; }
        
        
        let cy = crate::time::cbx();
        if cy >= brr { return 0; }
        
        
        
        
        let cds = brr.min(cy.saturating_add(10_000_000));
        crate::thread::cds(cds);
    }
}






pub fn pah(fd: i32, dirp: u64, count: u32) -> i64 {
    if !ij(dirp, count as usize, true) {
        return errno::P;
    }
    
    
    
    let path = crate::vfs::eof();
    
    let entries = match crate::vfs::readdir(&path) {
        Ok(e) => e,
        Err(_) => return errno::Aiz,
    };
    
    let buf = unsafe { core::slice::from_raw_parts_mut(dirp as *mut u8, count as usize) };
    let mut offset = 0usize;
    
    for entry in &entries {
        let agt = entry.name.as_bytes();
        
        let ddg = (8 + 8 + 2 + 1 + agt.len() + 1 + 7) & !7;
        
        if offset + ddg > count as usize { break; }
        
        let lbf: u8 = match entry.file_type {
            crate::vfs::FileType::Directory  => 4,
            crate::vfs::FileType::Regular    => 8,
            crate::vfs::FileType::CharDevice => 2,
            crate::vfs::FileType::Ak => 6,
            crate::vfs::FileType::Symlink    => 10,
            crate::vfs::FileType::Pipe       => 1,
            crate::vfs::FileType::Socket     => 12,
        };
        
        let ptr = &mut buf[offset..];
        if ptr.len() < ddg { break; }
        
        
        ptr[0..8].copy_from_slice(&entry.ino.to_le_bytes());
        
        let eva = (offset + ddg) as u64;
        ptr[8..16].copy_from_slice(&eva.to_le_bytes());
        
        ptr[16..18].copy_from_slice(&(ddg as u16).to_le_bytes());
        
        ptr[18] = lbf;
        
        let sj = 19;
        let aec = sj + agt.len();
        if aec < ptr.len() {
            ptr[sj..aec].copy_from_slice(agt);
            ptr[aec] = 0;
        }
        
        for i in (aec + 1)..ddg.min(ptr.len()) {
            ptr[i] = 0;
        }
        
        offset += ddg;
    }
    
    offset as i64
}






pub fn gxk(dirfd: i32, pathname: u64, flags: u32) -> i64 {
    const AAM_: i32 = -100;
    
    let path = match bdf(pathname, 256) {
        Some(j) => j,
        None => return errno::P,
    };
    
    
    if path.starts_with('/') || dirfd == AAM_ {
        return match crate::vfs::open(&path, crate::vfs::OpenFlags(flags)) {
            Ok(fd) => fd as i64,
            Err(_) => errno::Do,
        };
    }
    
    
    let kg = {
        let cwd = crate::vfs::eof();
        if cwd == "/" {
            alloc::format!("/{}", path)
        } else {
            alloc::format!("{}/{}", cwd, path)
        }
    };
    
    match crate::vfs::open(&kg, crate::vfs::OpenFlags(flags)) {
        Ok(fd) => fd as i64,
        Err(_) => errno::Do,
    }
}


pub fn pbx(path_ptr: u64) -> i64 {
    let (_, _, euid, _) = crate::process::bfs();
    if euid != 0 { return -1; } 
    let path = match bdf(path_ptr, 256) {
        Some(aa) => aa,
        None => return errno::P,
    };
    match crate::memory::swap::ozc(&path) {
        Ok(()) => 0,
        Err(_) => -1,
    }
}


pub fn pbw(path_ptr: u64) -> i64 {
    let (_, _, euid, _) = crate::process::bfs();
    if euid != 0 { return -1; }
    let path = match bdf(path_ptr, 256) {
        Some(aa) => aa,
        None => return errno::P,
    };
    match crate::memory::swap::ozb(&path) {
        Ok(()) => 0,
        Err(_) => -1,
    }
}





use alloc::collections::BTreeMap;


pub mod epoll_flags {
    pub const Ajb: u32 = 0x001;
    pub const Ajd: u32 = 0x004;
    pub const Yd: u32 = 0x008;
    pub const Ye: u32 = 0x010;
    pub const Aut: u32 = 0x2000;
    pub const Aus: u32 = 0x8000_0000;
    pub const Ajc: u32 = 0x4000_0000;
}


pub mod epoll_op {
    pub const BWY_: i32 = 1;
    pub const BWZ_: i32 = 2;
    pub const BXA_: i32 = 3;
}


#[repr(C, packed)]
#[derive(Clone, Copy)]
pub struct Hs {
    pub events: u32,
    pub data: u64,
}


#[derive(Clone)]
struct Ou {
    fd: i32,
    events: u32,
    data: u64,
}


pub struct Yj {
    
    interests: BTreeMap<i32, Ou>,
}


pub static KR_: Mutex<BTreeMap<i32, Yj>> = Mutex::new(BTreeMap::new());


static CKZ_: AtomicI32 = AtomicI32::new(500);

use core::sync::atomic::AtomicI32;


pub fn msn(fd: i32) -> bool {
    KR_.lock().contains_key(&fd)
}


pub fn jko(bej: u32) -> i64 {
    let fd = CKZ_.fetch_add(1, Ordering::SeqCst);
    let clg = Yj {
        interests: BTreeMap::new(),
    };
    KR_.lock().insert(fd, clg);
    crate::log_debug!("[EPOLL] created fd={}", fd);
    fd as i64
}


pub fn ozw(bek: i32) -> i64 {
    jko(0)
}


pub fn ozx(epfd: i32, op: i32, fd: i32, event_ptr: u64) -> i64 {
    use epoll_op::*;
    
    let mut bs = KR_.lock();
    let clg = match bs.get_mut(&epfd) {
        Some(i) => i,
        None => return errno::Cp,
    };
    
    match op {
        BWY_ => {
            if clg.interests.contains_key(&fd) {
                return errno::Aiu;
            }
            if event_ptr == 0 || !ij(event_ptr, core::mem::size_of::<Hs>(), false) {
                return errno::P;
            }
            let rt = unsafe { *(event_ptr as *const Hs) };
            clg.interests.insert(fd, Ou {
                fd,
                events: rt.events,
                data: rt.data,
            });
        }
        BXA_ => {
            if !clg.interests.contains_key(&fd) {
                return errno::Do;
            }
            if event_ptr == 0 || !ij(event_ptr, core::mem::size_of::<Hs>(), false) {
                return errno::P;
            }
            let rt = unsafe { *(event_ptr as *const Hs) };
            clg.interests.insert(fd, Ou {
                fd,
                events: rt.events,
                data: rt.data,
            });
        }
        BWZ_ => {
            if clg.interests.remove(&fd).is_none() {
                return errno::Do;
            }
        }
        _ => return errno::Bw,
    }
    
    0
}


pub fn jkp(epfd: i32, events_ptr: u64, maxevents: i32, timeout_ms: i32) -> i64 {
    if maxevents <= 0 {
        return errno::Bw;
    }
    let lrn = core::mem::size_of::<Hs>();
    let ate = (maxevents as usize) * lrn;
    if !ij(events_ptr, ate, true) {
        return errno::P;
    }
    
    let hws = unsafe {
        core::slice::from_raw_parts_mut(events_ptr as *mut Hs, maxevents as usize)
    };
    
    
    let brr = if timeout_ms < 0 {
        u64::MAX
    } else if timeout_ms == 0 {
        0
    } else {
        crate::time::cbx().saturating_add((timeout_ms as u64) * 1_000_000)
    };
    
    loop {
        
        let interests: Vec<Ou> = {
            let bs = KR_.lock();
            match bs.get(&epfd) {
                Some(inst) => inst.interests.values().cloned().collect(),
                None => return errno::Cp,
            }
        };
        
        let mut ready = 0usize;
        for interest in &interests {
            if ready >= maxevents as usize { break; }
            
            let mut revents = 0u32;
            if let Some(status) = crate::vfs::ivm(interest.fd) {
                if status.readable { revents |= epoll_flags::Ajb; }
                if status.writable { revents |= epoll_flags::Ajd; }
                if status.error    { revents |= epoll_flags::Yd; }
                if status.hangup   { revents |= epoll_flags::Ye; }
            }
            
            
            let jad = revents & (interest.events | epoll_flags::Yd | epoll_flags::Ye);
            if jad != 0 {
                hws[ready] = Hs {
                    events: jad,
                    data: interest.data,
                };
                ready += 1;
            }
        }
        
        if ready > 0 {
            
            let mut bs = KR_.lock();
            if let Some(inst) = bs.get_mut(&epfd) {
                for i in 0..ready {
                    let data = hws[i].data;
                    
                    for interest in inst.interests.values_mut() {
                        if interest.data == data && (interest.events & epoll_flags::Ajc) != 0 {
                            interest.events = 0; 
                        }
                    }
                }
            }
            return ready as i64;
        }
        
        
        if timeout_ms == 0 { return 0; }
        
        
        let cy = crate::time::cbx();
        if cy >= brr { return 0; }
        
        
        let cds = brr.min(cy.saturating_add(10_000_000));
        crate::thread::cds(cds);
    }
}


pub fn ozy(epfd: i32, events_ptr: u64, maxevents: i32, timeout_ms: i32, _sigmask: u64, _sigsetsize: u64) -> i64 {
    jkp(epfd, events_ptr, maxevents, timeout_ms)
}
