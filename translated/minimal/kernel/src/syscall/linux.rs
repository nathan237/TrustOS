




use crate::memory::{sw, aov};
use crate::syscall::errno;
use alloc::string::String;
use alloc::vec::Vec;


pub mod nr {
    pub const Cm: u64 = 0;
    pub const Db: u64 = 1;
    pub const Bnr: u64 = 2;
    pub const App: u64 = 3;
    pub const Bsc: u64 = 4;
    pub const Bgo: u64 = 5;
    pub const Bkp: u64 = 6;
    pub const Bol: u64 = 7;
    pub const Bkn: u64 = 8;
    pub const Avd: u64 = 9;
    pub const Blw: u64 = 10;
    pub const Avf: u64 = 11;
    pub const Apl: u64 = 12;
    pub const CPQ_: u64 = 13;
    pub const CPR_: u64 = 14;
    pub const AHJ_: u64 = 15;
    pub const Bjf: u64 = 16;
    pub const Ddy: u64 = 17;
    pub const Deg: u64 = 18;
    pub const Cjl: u64 = 19;
    pub const Bwk: u64 = 20;
    pub const Aos: u64 = 21;
    pub const Ddt: u64 = 22;
    pub const Dgn: u64 = 23;
    pub const CQM_: u64 = 24;
    pub const Dcb: u64 = 25;
    pub const Dcc: u64 = 26;
    pub const Dbq: u64 = 27;
    pub const Dbl: u64 = 28;
    pub const Bed: u64 = 32;
    pub const Bee: u64 = 33;
    pub const Boi: u64 = 34;
    pub const Chr: u64 = 35;
    pub const Cxo: u64 = 36;
    pub const Crb: u64 = 37;
    pub const Dgw: u64 = 38;
    pub const Asv: u64 = 39;
    pub const Dgp: u64 = 40;
    pub const Cmc: u64 = 41;
    pub const Bzb: u64 = 42;
    pub const Bxe: u64 = 43;
    pub const Clg: u64 = 44;
    pub const Cjm: u64 = 45;
    pub const Clf: u64 = 46;
    pub const Cjn: u64 = 47;
    pub const Uf: u64 = 48;
    pub const Bch: u64 = 49;
    pub const Cgn: u64 = 50;
    pub const Cee: u64 = 51;
    pub const Cdx: u64 = 52;
    pub const Dhr: u64 = 53;
    pub const Clp: u64 = 54;
    pub const Cef: u64 = 55;
    pub const Byz: u64 = 56;
    pub const Bgm: u64 = 57;
    pub const Coz: u64 = 58;
    pub const Bfp: u64 = 59;
    pub const Ahp: u64 = 60;
    pub const Cpz: u64 = 61;
    pub const Cgg: u64 = 62;
    pub const Bae: u64 = 63;
    pub const Ccv: u64 = 72;
    pub const Cwv: u64 = 73;
    pub const Cwz: u64 = 74;
    pub const Cwr: u64 = 75;
    pub const Djp: u64 = 76;
    pub const Cxa: u64 = 77;
    pub const Cxm: u64 = 78;
    pub const Asr: u64 = 79;
    pub const Bcy: u64 = 80;
    pub const Cwn: u64 = 81;
    pub const Dfm: u64 = 82;
    pub const Blu: u64 = 83;
    pub const Cjz: u64 = 84;
    pub const Csp: u64 = 85;
    pub const Daz: u64 = 86;
    pub const Bva: u64 = 87;
    pub const Die: u64 = 88;
    pub const Bpy: u64 = 89;
    pub const Byw: u64 = 90;
    pub const Cct: u64 = 91;
    pub const Byx: u64 = 92;
    pub const Ccu: u64 = 93;
    pub const Cgl: u64 = 94;
    pub const Coq: u64 = 95;
    pub const Ceh: u64 = 96;
    pub const Cec: u64 = 97;
    pub const Cxs: u64 = 98;
    pub const Dii: u64 = 99;
    pub const Djj: u64 = 100;
    pub const Asw: u64 = 102;
    pub const Dij: u64 = 103;
    pub const Asu: u64 = 104;
    pub const Clq: u64 = 105;
    pub const Clj: u64 = 106;
    pub const Ast: u64 = 107;
    pub const Ass: u64 = 108;
    pub const Clk: u64 = 109;
    pub const Cea: u64 = 110;
    pub const Cdz: u64 = 111;
    pub const Clo: u64 = 112;
    pub const Clm: u64 = 113;
    pub const Cll: u64 = 114;
    pub const Cxn: u64 = 115;
    pub const Dgu: u64 = 116;
    pub const Dha: u64 = 117;
    pub const Cxr: u64 = 118;
    pub const Dgz: u64 = 119;
    pub const Cxq: u64 = 120;
    pub const Cdy: u64 = 121;
    pub const Dgt: u64 = 122;
    pub const Dgs: u64 = 123;
    pub const Ced: u64 = 124;
    pub const Csh: u64 = 125;
    pub const Csi: u64 = 126;
    pub const EDE_: u64 = 127;
    pub const EDH_: u64 = 128;
    pub const EDF_: u64 = 129;
    pub const EDG_: u64 = 130;
    pub const Cls: u64 = 131;
    pub const Dkn: u64 = 132;
    pub const Dbs: u64 = 133;
    pub const Dkk: u64 = 134;
    pub const Ddq: u64 = 135;
    pub const Dkm: u64 = 136;
    pub const Dhw: u64 = 137;
    pub const Cwy: u64 = 138;
    pub const Dih: u64 = 139;
    pub const Cxp: u64 = 140;
    pub const Dgy: u64 = 141;
    pub const EEF_: u64 = 142;
    pub const EDZ_: u64 = 143;
    pub const EEG_: u64 = 144;
    pub const EEA_: u64 = 145;
    pub const EEB_: u64 = 146;
    pub const EEC_: u64 = 147;
    pub const EED_: u64 = 148;
    pub const Dbu: u64 = 149;
    pub const Dcd: u64 = 150;
    pub const Dbw: u64 = 151;
    pub const Dce: u64 = 152;
    pub const Dkw: u64 = 153;
    pub const DUI_: u64 = 154;
    pub const DZV_: u64 = 155;
    pub const Cit: u64 = 157;
    pub const ZD_: u64 = 158;
    pub const Cra: u64 = 159;
    pub const Cln: u64 = 160;
    pub const Byy: u64 = 161;
    pub const Cmo: u64 = 162;
    pub const Cqy: u64 = 163;
    pub const Dhb: u64 = 164;
    pub const Dbz: u64 = 165;
    pub const Dkb: u64 = 166;
    pub const Cml: u64 = 167;
    pub const Cmk: u64 = 168;
    pub const Axh: u64 = 169;
    pub const Dgv: u64 = 170;
    pub const Dgr: u64 = 171;
    pub const Czd: u64 = 172;
    pub const Czc: u64 = 173;
    pub const Ceg: u64 = 186;
    pub const Dfi: u64 = 187;
    pub const Dhc: u64 = 188;
    pub const Cxu: u64 = 191;
    pub const Dbb: u64 = 194;
    pub const Dfl: u64 = 197;
    pub const Djl: u64 = 200;
    pub const Dji: u64 = 201;
    pub const Cdg: u64 = 202;
    pub const CQL_: u64 = 203;
    pub const CQK_: u64 = 204;
    pub const EGC_: u64 = 205;
    pub const DRR_: u64 = 206;
    pub const DRP_: u64 = 207;
    pub const DRQ_: u64 = 208;
    pub const DRT_: u64 = 209;
    pub const DRO_: u64 = 210;
    pub const DMZ_: u64 = 211;
    pub const DST_: u64 = 212;
    pub const BTZ_: u64 = 213;
    pub const ECH_: u64 = 216;
    pub const Cdw: u64 = 217;
    pub const AHU_: u64 = 218;
    pub const ECN_: u64 = 219;
    pub const Dgo: u64 = 220;
    pub const Cwl: u64 = 221;
    pub const EHV_: u64 = 222;
    pub const EHZ_: u64 = 223;
    pub const EHY_: u64 = 224;
    pub const EHX_: u64 = 225;
    pub const EHW_: u64 = 226;
    pub const DFA_: u64 = 227;
    pub const BMM_: u64 = 228;
    pub const DES_: u64 = 229;
    pub const DEW_: u64 = 230;
    pub const ABX_: u64 = 231;
    pub const BUG_: u64 = 232;
    pub const BUB_: u64 = 233;
    pub const Djh: u64 = 234;
    pub const Dkp: u64 = 235;
    pub const Dbm: u64 = 237;
    pub const EFW_: u64 = 238;
    pub const DMU_: u64 = 239;
    pub const DUT_: u64 = 240;
    pub const DUW_: u64 = 241;
    pub const DUV_: u64 = 242;
    pub const DUU_: u64 = 243;
    pub const DUS_: u64 = 244;
    pub const DUR_: u64 = 245;
    pub const DSF_: u64 = 246;
    pub const Dlw: u64 = 247;
    pub const DCE_: u64 = 248;
    pub const ECL_: u64 = 249;
    pub const Daq: u64 = 250;
    pub const DRL_: u64 = 251;
    pub const DRK_: u64 = 252;
    pub const DRC_: u64 = 253;
    pub const DRB_: u64 = 254;
    pub const DRE_: u64 = 255;
    pub const DUD_: u64 = 256;
    pub const Bns: u64 = 257;
    pub const Dbr: u64 = 258;
    pub const Dbt: u64 = 259;
    pub const Cwp: u64 = 260;
    pub const Cxb: u64 = 261;
    pub const Bnb: u64 = 262;
    pub const Dkc: u64 = 263;
    pub const Dfn: u64 = 264;
    pub const Dba: u64 = 265;
    pub const Dif: u64 = 266;
    pub const Dfj: u64 = 267;
    pub const Cwo: u64 = 268;
    pub const Cwk: u64 = 269;
    pub const Ded: u64 = 270;
    pub const Ddx: u64 = 271;
    pub const Dkd: u64 = 272;
    pub const CSM_: u64 = 273;
    pub const BWZ_: u64 = 274;
    pub const Dht: u64 = 275;
    pub const Djf: u64 = 276;
    pub const EHD_: u64 = 277;
    pub const Dlh: u64 = 278;
    pub const DUK_: u64 = 279;
    pub const Dko: u64 = 280;
    pub const BUF_: u64 = 281;
    pub const Dhe: u64 = 282;
    pub const EHS_: u64 = 283;
    pub const Cvo: u64 = 284;
    pub const Cwm: u64 = 285;
    pub const EHU_: u64 = 286;
    pub const EHT_: u64 = 287;
    pub const Cqx: u64 = 288;
    pub const Dhf: u64 = 289;
    pub const Cvp: u64 = 290;
    pub const BUA_: u64 = 291;
    pub const Cas: u64 = 292;
    pub const Cip: u64 = 293;
    pub const DRD_: u64 = 294;
    pub const Ddz: u64 = 295;
    pub const Deh: u64 = 296;
    pub const EDI_: u64 = 297;
    pub const DZN_: u64 = 298;
    pub const Dfk: u64 = 299;
    pub const DLP_: u64 = 300;
    pub const DLQ_: u64 = 301;
    pub const Ciu: u64 = 302;
    pub const DVB_: u64 = 303;
    pub const DXK_: u64 = 304;
    pub const DEQ_: u64 = 305;
    pub const Dig: u64 = 306;
    pub const Dgq: u64 = 307;
    pub const Dgx: u64 = 308;
    pub const Cxl: u64 = 309;
    pub const EBD_: u64 = 310;
    pub const EBE_: u64 = 311;
    pub const Dan: u64 = 312;
    pub const DLX_: u64 = 313;
    pub const EEE_: u64 = 314;
    pub const EDY_: u64 = 315;
    pub const Dfo: u64 = 316;
    pub const Dgk: u64 = 317;
    pub const Ceb: u64 = 318;
    pub const DTU_: u64 = 319;
    pub const DSE_: u64 = 320;
    pub const Crt: u64 = 321;
    pub const Cvs: u64 = 322;
    pub const Dkl: u64 = 323;
    pub const Dbo: u64 = 324;
    pub const Dbv: u64 = 325;
    pub const DGW_: u64 = 326;
    pub const Dea: u64 = 327;
    pub const Dei: u64 = 328;
    pub const DZZ_: u64 = 329;
    pub const DZX_: u64 = 330;
    pub const DZY_: u64 = 331;
    pub const Dhx: u64 = 332;
}






pub mod mmap_flags {
    pub const CFB_: u64 = 0x01;
    pub const AZG_: u64 = 0x02;
    pub const CFA_: u64 = 0x10;
    pub const AES_: u64 = 0x20;
    pub const DSW_: u64 = 0x100;
    pub const DSU_: u64 = 0x800;
    pub const DSV_: u64 = 0x1000;
    pub const DSY_: u64 = 0x2000;
    pub const DTA_: u64 = 0x4000;
    pub const DTB_: u64 = 0x8000;
    pub const DSZ_: u64 = 0x10000;
    pub const DTC_: u64 = 0x20000;
    pub const DSX_: u64 = 0x40000;
}


pub mod prot_flags {
    pub const CKV_: u64 = 0x0;
    pub const CKW_: u64 = 0x1;
    pub const WK_: u64 = 0x2;
    pub const AGR_: u64 = 0x4;
}

use spin::Mutex;
use core::sync::atomic::{AtomicU64, Ordering};


static EBF_: AtomicU64 = AtomicU64::new(0);


static CHR_: AtomicU64 = AtomicU64::new(0x4000_0000); 





pub fn jsc(ag: u64, go: u64, prot: u64, flags: u64, da: i64, dnv: u64) -> i64 {
    use mmap_flags::*;
    use prot_flags::*;
    use crate::memory::paging::PageFlags;
    
    if go == 0 {
        return errno::Er;
    }
    
    let aus = 4096u64;
    let emb = (go + aus - 1) & !(aus - 1);
    
    
    let efc = if ag != 0 && (flags & CFA_) != 0 {
        let ciz = ag & !(aus - 1); 
        
        if !crate::memory::aov(ciz) {
            return errno::Er;
        }
        ciz
    } else {
        
        CHR_.fetch_add(emb, Ordering::SeqCst)
    };
    
    
    let twr = (flags & AES_) != 0 || da < 0;
    if !twr {
        crate::log_debug!("[MMAP] File-backed mmap not yet implemented");
        return errno::Pg;
    }
    
    
    let jm: u64;
    #[cfg(target_arch = "x86_64")]
    unsafe { core::arch::asm!("mov {}, cr3", bd(reg) jm, options(nostack, preserves_flags)); }
    #[cfg(not(target_arch = "x86_64"))]
    { jm = 0; }
    
    let xsg = (prot & 0x7) as u32; 
    
    crate::memory::vma::qfp(jm, crate::memory::vma::Rf {
        ay: efc,
        ci: efc + emb,
        prot: xsg,
        flags: crate::memory::vma::flags::AES_ | crate::memory::vma::flags::AZG_,
    });
    
    crate::log_debug!("[MMAP] Lazy VMA {:#x}..{:#x} prot={:#x}", efc, efc + emb, prot);
    efc as i64
}


pub fn jsd(ag: u64, go: u64) -> i64 {
    if ag == 0 || go == 0 {
        return errno::Er;
    }
    
    let aus = 4096u64;
    let emb = (go + aus - 1) & !(aus - 1);
    let dtt = (emb / aus) as usize;
    let ay = ag & !(aus - 1);
    
    
    let jm: u64;
    #[cfg(target_arch = "x86_64")]
    unsafe { core::arch::asm!("mov {}, cr3", bd(reg) jm, options(nostack, preserves_flags)); }
    #[cfg(not(target_arch = "x86_64"))]
    { jm = 0; }
    crate::memory::vma::vva(jm, ay, ay + emb);
    
    
    crate::exec::jwy(|atm| {
        for a in 0..dtt {
            let ju = ay + (a as u64 * aus);
            if let Some(ht) = atm.dmr(ju) {
                let fqs = ht & !0xFFF;
                atm.xoj(ju);
                crate::memory::frame::apt(fqs);
            }
        }
    });
    
    crate::log_debug!("[MUNMAP] Unmapped {} pages at {:#x}", dtt, ag);
    0
}


pub fn wyj(ag: u64, go: u64, prot: u64) -> i64 {
    use prot_flags::*;
    use crate::memory::paging::{PageFlags, PageTable};
    
    if ag == 0 || ag & 0xFFF != 0 {
        return errno::Er;
    }
    
    let aus = 4096u64;
    let emb = (go + aus - 1) & !(aus - 1);
    let dtt = (emb / aus) as usize;
    
    
    let mut ewp = PageFlags::Cz | PageFlags::Gq;
    if (prot & WK_) != 0 {
        ewp |= PageFlags::Ff;
    }
    if (prot & AGR_) == 0 {
        ewp |= PageFlags::DL_;
    }
    let fot = PageFlags::new(ewp);
    
    crate::exec::jwy(|atm| {
        let hp = crate::memory::lr();
        let jm = atm.jm();
        
        for a in 0..dtt {
            let ju = ag + (a as u64 * aus);
            let wd = ((ju >> 39) & 0x1FF) as usize;
            let ru = ((ju >> 30) & 0x1FF) as usize;
            let rn   = ((ju >> 21) & 0x1FF) as usize;
            let yf   = ((ju >> 12) & 0x1FF) as usize;
            
            let wc = unsafe { &*((jm + hp) as *const PageTable) };
            if !wc.ch[wd].xo() { continue; }
            let ss = unsafe { &*((wc.ch[wd].ki() + hp) as *const PageTable) };
            if !ss.ch[ru].xo() { continue; }
            let sr = unsafe { &*((ss.ch[ru].ki() + hp) as *const PageTable) };
            if !sr.ch[rn].xo() { continue; }
            let se = unsafe { &mut *((sr.ch[rn].ki() + hp) as *mut PageTable) };
            if !se.ch[yf].xo() { continue; }
            
            let ht = se.ch[yf].ki();
            se.ch[yf].oj(ht, fot);
            #[cfg(target_arch = "x86_64")]
            unsafe { core::arch::asm!("invlpg [{}]", in(reg) ju, options(nostack, preserves_flags)); }
        }
    });
    
    crate::log_debug!("[MPROTECT] addr={:#x} len={:#x} prot={:#x}", ag, go, prot);
    0
}




pub fn jsa(ag: u64) -> i64 {
    use crate::memory::paging::{PageFlags, UserMemoryRegion};
    
    let dfj = crate::exec::dfj();
    
    if ag == 0 || dfj == 0 {
        
        if dfj == 0 {
            return UserMemoryRegion::CF_ as i64;
        }
        return dfj as i64;
    }
    
    
    if ag < UserMemoryRegion::CF_ || ag >= UserMemoryRegion::BZA_ {
        return dfj as i64;
    }
    
    let aus = 4096u64;
    
    if ag > dfj {
        
        let lpz = (dfj + aus - 1) & !(aus - 1); 
        let lnz = (ag + aus - 1) & !(aus - 1);        
        
        if lnz > lpz {
            let duc = ((lnz - lpz) / aus) as usize;
            
            let bq = crate::exec::jwy(|atm| {
                for a in 0..duc {
                    let ju = lpz + (a as u64 * aus);
                    let ht = match crate::memory::frame::azg() {
                        Some(ai) => ai,
                        None => return false,
                    };
                    if atm.bnl(ju, ht, PageFlags::EW_).is_none() {
                        return false;
                    }
                }
                true
            });
            
            if bq != Some(true) {
                return dfj as i64; 
            }
        }
    }
    
    
    
    crate::exec::wio(ag);
    crate::log_debug!("[BRK] Set program break to {:#x}", ag);
    ag as i64
}






pub fn wya() -> i64 {
    crate::process::aei() as i64
}


pub fn wyb() -> i64 {
    crate::process::xuv(|ai| ai.bfb as i64)
        .unwrap_or(0)
}


pub fn pqw() -> i64 {
    crate::thread::bqd() as i64
}


pub fn wyg() -> i64 {
    let (pi, _, _, _) = crate::process::dfk();
    pi as i64
}


pub fn wxw() -> i64 {
    let (_, pw, _, _) = crate::process::dfk();
    pw as i64
}


pub fn zqs() -> i64 {
    let (_, _, ahl, _) = crate::process::dfk();
    ahl as i64
}


pub fn zqr() -> i64 {
    let (_, _, _, bqj) = crate::process::dfk();
    bqj as i64
}


pub fn wzf(pi: u32) -> i64 {
    let ce = crate::process::aei();
    match crate::process::pji(ce, pi) {
        Ok(()) => 0,
        Err(_) => -1, 
    }
}


pub fn wyz(pw: u32) -> i64 {
    let ce = crate::process::aei();
    match crate::process::pja(ce, pw) {
        Ok(()) => 0,
        Err(_) => -1, 
    }
}


pub fn wzc(pen: u32, ahl: u32) -> i64 {
    let ce = crate::process::aei();
    
    if pen != 0xFFFFFFFF {
        if crate::process::pji(ce, pen).is_err() { return -1; }
    }
    if ahl != 0xFFFFFFFF {
        
        let mut gg = crate::process::AD_.write();
        if let Some(ai) = gg.ye.ds(&ce) {
            if ai.ahl == 0 || ahl == ai.pi || ahl == ai.ahl {
                ai.ahl = ahl;
            } else {
                return -1;
            }
        }
    }
    0
}


pub fn wzb(pdb: u32, bqj: u32) -> i64 {
    let ce = crate::process::aei();
    if pdb != 0xFFFFFFFF {
        if crate::process::pja(ce, pdb).is_err() { return -1; }
    }
    if bqj != 0xFFFFFFFF {
        let mut gg = crate::process::AD_.write();
        if let Some(ai) = gg.ye.ds(&ce) {
            if ai.ahl == 0 || bqj == ai.pw || bqj == ai.bqj {
                ai.bqj = bqj;
            } else {
                return -1;
            }
        }
    }
    0
}


pub fn wzl(hs: u32) -> i64 {
    let ce = crate::process::aei();
    crate::process::wjx(ce, hs) as i64
}






pub fn wza(ce: u32, bai: u32) -> i64 {
    match crate::process::wjk(ce, bai) {
        Ok(()) => 0,
        Err(_) => -1,
    }
}


pub fn wxz() -> i64 {
    crate::process::nyi(0) as i64
}


pub fn wzd() -> i64 {
    match crate::process::wkc() {
        Ok(ary) => ary as i64,
        Err(_) => -1,
    }
}


pub fn wxy(ce: u32) -> i64 {
    crate::process::nyi(ce) as i64
}


pub fn wyc(ce: u32) -> i64 {
    crate::process::nyo(ce) as i64
}


pub fn wxh(arq: u64) -> i64 {
    let path = match daz(arq, 256) {
        Some(e) => e,
        None => return -14, 
    };
    let ce = crate::process::aei();
    match crate::process::raq(ce, &path) {
        Ok(()) => 0,
        Err(_) => -1, 
    }
}


pub fn wxg(arq: u64, ev: u32) -> i64 {
    let path = match daz(arq, 256) {
        Some(ai) => ai,
        None => return -14, 
    };
    match crate::vfs::ral(&path, ev) {
        Ok(()) => 0,
        Err(_) => -1, 
    }
}


pub fn wxq(da: i32, ev: u32) -> i64 {
    match crate::vfs::srh(da, ev) {
        Ok(()) => 0,
        Err(_) => -1,
    }
}


pub fn pqq(arq: u64, pi: u32, pw: u32) -> i64 {
    let path = match daz(arq, 256) {
        Some(ai) => ai,
        None => return -14,
    };
    
    let (_, _, ahl, _) = crate::process::dfk();
    if ahl != 0 { return -1; } 
    match crate::vfs::ran(&path, pi, pw) {
        Ok(()) => 0,
        Err(_) => -1,
    }
}


pub fn wxr(da: i32, pi: u32, pw: u32) -> i64 {
    let (_, _, ahl, _) = crate::process::dfk();
    if ahl != 0 { return -1; }
    match crate::vfs::sri(da, pi, pw) {
        Ok(()) => 0,
        Err(_) => -1,
    }
}






pub mod arch_prctl_codes {
    pub const ZE_: u64 = 0x1001;
    pub const LT_: u64 = 0x1002;
    pub const LS_: u64 = 0x1003;
    pub const ZC_: u64 = 0x1004;
}


static BHH_: AtomicU64 = AtomicU64::new(0);


pub fn mip(aj: u64, ag: u64) -> i64 {
    use arch_prctl_codes::*;
    
    match aj {
        LT_ => {
            
            BHH_.store(ag, Ordering::SeqCst);
            
            
            #[cfg(target_arch = "x86_64")]
            unsafe {
                
                core::arch::asm!(
                    "wrmsr",
                    in("ecx") 0xC0000100u32,
                    in("eax") (ag as u32),
                    in("edx") ((ag >> 32) as u32),
                );
            }
            crate::log_debug!("[ARCH_PRCTL] Set FS base to {:#x}", ag);
            0
        }
        ZE_ => {
            
            #[cfg(target_arch = "x86_64")]
            unsafe {
                
                core::arch::asm!(
                    "wrmsr",
                    in("ecx") 0xC0000101u32,
                    in("eax") (ag as u32),
                    in("edx") ((ag >> 32) as u32),
                );
            }
            0
        }
        LS_ => {
            if !aov(ag) {
                return errno::X;
            }
            let ap = BHH_.load(Ordering::SeqCst);
            unsafe { *(ag as *mut u64) = ap; }
            0
        }
        ZC_ => {
            if !aov(ag) {
                return errno::X;
            }
            let ap: u64;
            #[cfg(target_arch = "x86_64")]
            unsafe {
                core::arch::asm!(
                    "rdmsr",
                    in("ecx") 0xC0000101u32,
                    bd("eax") _,
                    bd("edx") _,
                );
                
                ap = 0;
            }
            #[cfg(not(target_arch = "x86_64"))]
            { ap = 0; }
            unsafe { *(ag as *mut u64) = ap; }
            0
        }
        _ => errno::Er,
    }
}





static BML_: AtomicU64 = AtomicU64::new(0);


pub fn wyy(xgw: u64) -> i64 {
    BML_.store(xgw, Ordering::SeqCst);
    pqw()
}






#[repr(C)]
pub struct Afk {
    pub gtz: [u8; 65],
    pub gnv: [u8; 65],
    pub ehl: [u8; 65],
    pub dk: [u8; 65],
    pub czk: [u8; 65],
    pub gfd: [u8; 65],
}


pub fn mjc(k: u64) -> i64 {
    if !sw(k, core::mem::size_of::<Afk>(), true) {
        return errno::X;
    }
    
    let cin = unsafe { &mut *(k as *mut Afk) };
    
    
    *cin = Afk {
        gtz: [0; 65],
        gnv: [0; 65],
        ehl: [0; 65],
        dk: [0; 65],
        czk: [0; 65],
        gfd: [0; 65],
    };
    
    
    gdi(&mut cin.gtz, "TrustOS");
    gdi(&mut cin.gnv, "trustos");
    gdi(&mut cin.ehl, "1.0.0-trustos");
    gdi(&mut cin.dk, "#1 SMP PREEMPT TrustOS");
    gdi(&mut cin.czk, "x86_64");
    gdi(&mut cin.gfd, "(none)");
    
    0
}

fn gdi(sy: &mut [u8; 65], e: &str) {
    let bf = e.as_bytes();
    let len = bf.len().v(64);
    sy[..len].dg(&bf[..len]);
}






#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct Ml {
    pub ekg: i64,
    pub fxn: i64,
}


#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct Buk {
    pub ekg: i64,
    pub xnh: i64,
}


pub mod clock_ids {
    pub const DEY_: u32 = 0;
    pub const DET_: u32 = 1;
    pub const DEX_: u32 = 2;
    pub const DFB_: u32 = 3;
    pub const DEV_: u32 = 4;
    pub const DEZ_: u32 = 5;
    pub const DEU_: u32 = 6;
    pub const DER_: u32 = 7;
}


pub fn mir(yiq: u32, aaz: u64) -> i64 {
    if !sw(aaz, core::mem::size_of::<Ml>(), true) {
        return errno::X;
    }
    
    let qb = crate::time::ave();
    let dvm = qb / 1000;
    let efq = (qb % 1000) * 1_000_000;
    
    let wi = unsafe { &mut *(aaz as *mut Ml) };
    wi.ekg = dvm as i64;
    wi.fxn = efq as i64;
    
    0
}


pub fn wyf(fxm: u64, ifo: u64) -> i64 {
    if fxm != 0 {
        if !sw(fxm, core::mem::size_of::<Buk>(), true) {
            return errno::X;
        }
        
        let qb = crate::time::ave();
        let dvm = qb / 1000;
        let xpo = (qb % 1000) * 1000;
        
        let ptj = unsafe { &mut *(fxm as *mut Buk) };
        ptj.ekg = dvm as i64;
        ptj.xnh = xpo as i64;
    }
    
    
    0
}


pub fn miz(ehq: u64, rem: u64) -> i64 {
    if !sw(ehq, core::mem::size_of::<Ml>(), false) {
        return errno::X;
    }
    
    let wi = unsafe { &*(ehq as *const Ml) };
    let jn = (wi.ekg * 1000 + wi.fxn / 1_000_000) as u64;
    
    
    let ay = crate::time::ave();
    while crate::time::ave().ao(ay) < jn {
        crate::thread::cix();
    }
    
    if rem != 0 && sw(rem, core::mem::size_of::<Ml>(), true) {
        let pbp = unsafe { &mut *(rem as *mut Ml) };
        pbp.ekg = 0;
        pbp.fxn = 0;
    }
    
    0
}






pub fn miv(k: u64, az: u64, ddp: u64) -> i64 {
    if !sw(k, az as usize, true) {
        return errno::X;
    }
    
    let bi = unsafe { core::slice::bef(k as *mut u8, az as usize) };
    
    
    for hf in bi.el() {
        *hf = crate::rng::ozi();
    }
    
    az as i64
}






pub fn miw(da: i32, request: u64, ji: u64) -> i64 {
    
    const Aev: u64 = 0x5401;
    const Azq: u64 = 0x5402;
    const Aew: u64 = 0x5413;
    const Btx: u64 = 0x5414;
    const Cda: u64 = 0x541B;
    
    match request {
        Aev | Azq => {
            
            0
        }
        Aew => {
            
            if ji != 0 && sw(ji, 8, true) {
                let fbn = unsafe { &mut *(ji as *mut [u16; 4]) };
                fbn[0] = 25;  
                fbn[1] = 80;  
                fbn[2] = 0;   
                fbn[3] = 0;   
            }
            0
        }
        Cda => {
            
            if ji != 0 && sw(ji, 4, true) {
                unsafe { *(ji as *mut i32) = 0; }
            }
            0
        }
        _ => {
            crate::log_debug!("[IOCTL] Unknown ioctl fd={} request={:#x}", da, request);
            0 
        }
    }
}


pub fn wxs(da: i32, cmd: u32, ji: u64) -> i64 {
    use alloc::collections::BTreeMap;
    use spin::Mutex;

    const BWF_: u32 = 0;
    const BWG_: u32 = 1;
    const BWI_: u32 = 2;
    const BWH_: u32 = 3;
    const BWJ_: u32 = 4;
    const ASP_: u32 = 0x406;

    
    static MY_: Mutex<BTreeMap<(u32, i32), (u32, u32)>> = Mutex::new(BTreeMap::new());

    let ce = crate::process::aei();
    let bs = (ce, da);

    match cmd {
        BWF_ | ASP_ => {
            match crate::vfs::ksb(da) {
                Ok(anp) => {
                    if cmd == ASP_ {
                        let mut flags = MY_.lock();
                        flags.insert((ce, anp), (1, 0)); 
                    }
                    anp as i64
                }
                Err(_) => -9, 
            }
        }
        BWG_ => {
            let flags = MY_.lock();
            flags.get(&bs).map(|bb| bb.0 as i64).unwrap_or(0)
        }
        BWI_ => {
            let mut flags = MY_.lock();
            let bt = flags.bt(bs).gom((0, 0));
            bt.0 = ji as u32;
            0
        }
        BWH_ => {
            let flags = MY_.lock();
            flags.get(&bs).map(|bb| bb.1 as i64).unwrap_or(0)
        }
        BWJ_ => {
            
            let emd = 0x400 | 0x800 | 0x2000;
            let mut flags = MY_.lock();
            let bt = flags.bt(bs).gom((0, 0));
            bt.1 = (bt.1 & !emd) | (ji as u32 & emd);
            0
        }
        _ => {
            crate::log_debug!("[FCNTL] fd={} cmd={:#x} arg={}", da, cmd, ji);
            0
        }
    }
}


#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct Stat {
    pub pnh: u64,
    pub pnj: u64,
    pub pnn: u64,
    pub jrk: u32,
    pub pnq: u32,
    pub pni: u32,
    pub iig: u32,
    pub mhg: u64,
    pub gsz: i64,
    pub mhe: i64,
    pub pne: i64,
    pub pnc: i64,
    pub pnd: i64,
    pub pnk: i64,
    pub pnl: i64,
    pub pnf: i64,
    pub png: i64,
    pub qdv: [i64; 3],
}


pub mod stat_mode {
    pub const BGW_: u32 = 0o170000;
    pub const PV_: u32 = 0o100000;
    pub const AIN_: u32 = 0o040000;
    pub const XQ_: u32 = 0o020000;
    pub const CXF_: u32 = 0o010000;
    pub const AIO_: u32 = 0o120000;
    pub const CXG_: u32 = 0o140000;
}


fn ssg(agm: crate::vfs::FileType) -> u32 {
    match agm {
        crate::vfs::FileType::Ea    => stat_mode::PV_,
        crate::vfs::FileType::K  => stat_mode::AIN_,
        crate::vfs::FileType::Mv => stat_mode::XQ_,
        crate::vfs::FileType::Bj => 0o060000, 
        crate::vfs::FileType::Anh    => stat_mode::AIO_,
        crate::vfs::FileType::Yc       => stat_mode::CXF_,
        crate::vfs::FileType::Socket     => stat_mode::CXG_,
    }
}


fn pyg(vfs: &crate::vfs::Stat) -> Stat {
    Stat {
        pnh: 1,
        pnj: vfs.dd,
        pnn: 1,
        jrk: ssg(vfs.kd) | (vfs.ev & 0o7777),
        pnq: vfs.pi,
        pni: vfs.pw,
        iig: 0,
        mhg: 0,
        gsz: vfs.aw as i64,
        mhe: vfs.py as i64,
        pne: ((vfs.aw + 511) / 512) as i64,
        pnc: vfs.byi as i64,
        pnd: 0,
        pnk: vfs.bnp as i64,
        pnl: 0,
        pnf: vfs.cpq as i64,
        png: 0,
        qdv: [0; 3],
    }
}


pub fn pqu(da: i32, eja: u64) -> i64 {
    if !sw(eja, core::mem::size_of::<Stat>(), true) {
        return errno::X;
    }
    
    let hm = unsafe { &mut *(eja as *mut Stat) };
    
    
    if da >= 0 && da <= 2 {
        *hm = Stat::default();
        hm.jrk = stat_mode::XQ_ | 0o666;
        hm.mhg = 0x0500; 
        hm.mhe = 4096;
        return 0;
    }
    
    
    match crate::vfs::syo(da) {
        Ok(mpe) => {
            *hm = pyg(&mpe);
            0
        }
        Err(_) => errno::Fu,
    }
}


pub fn icn(clu: u64, eja: u64) -> i64 {
    let path = match daz(clu, 4096) {
        Some(e) => e,
        None => return errno::X,
    };
    
    if !sw(eja, core::mem::size_of::<Stat>(), true) {
        return errno::X;
    }
    
    let hm = unsafe { &mut *(eja as *mut Stat) };
    
    match crate::vfs::hm(&path) {
        Ok(mpe) => {
            *hm = pyg(&mpe);
            0
        }
        Err(_) => errno::Il,
    }
}


pub fn wyk(ges: i32, clu: u64, eja: u64, ddp: u32) -> i64 {
    const ZF_: i32 = -100;
    const BKS_: u32 = 0x1000;
    
    
    if ddp & BKS_ != 0 {
        let path = daz(clu, 4096);
        if path.is_none() || path.as_ref().efd(false, |e| e.is_empty()) {
            if ges >= 0 {
                return pqu(ges, eja);
            }
        }
    }
    
    let path = match daz(clu, 4096) {
        Some(e) => e,
        None => return errno::X,
    };
    
    
    if path.cj('/') || ges == ZF_ {
        return icn(clu, eja);
    }
    
    
    icn(clu, eja)
}


pub fn mio(clu: u64, ev: u32) -> i64 {
    let qdh = match daz(clu, 256) {
        Some(e) => e,
        None => return errno::X,
    };
    
    
    
    0
}


pub fn wyp(clu: u64, k: u64, qsx: u64) -> i64 {
    let path = match daz(clu, 256) {
        Some(e) => e,
        None => return errno::X,
    };
    
    
    if path == "/proc/self/exe" {
        let nri = "/bin/program";
        let len = nri.len().v(qsx as usize);
        if sw(k, len, true) {
            let cs = unsafe { core::slice::bef(k as *mut u8, len) };
            cs.dg(&nri.as_bytes()[..len]);
            return len as i64;
        }
    }
    
    errno::Er
}






pub fn wys(sig: u32, iit: u64, jht: u64, pku: u64) -> i64 {
    let ce = crate::process::aei();
    crate::log_debug!("[SIGACTION] pid={} sig={} act={:#x} oldact={:#x}", ce, sig, iit, jht);

    
    if jht != 0 && sw(jht, core::mem::size_of::<crate::signals::SigAction>(), true) {
        if let Ok(aft) = crate::signals::kyh(ce, sig) {
            unsafe {
                core::ptr::write(jht as *mut crate::signals::SigAction, aft);
            }
        }
    }

    
    if iit != 0 && sw(iit, core::mem::size_of::<crate::signals::SigAction>(), false) {
        let usk = unsafe { core::ptr::read(iit as *const crate::signals::SigAction) };
        if let Err(aa) = crate::signals::mec(ce, sig, usk) {
            return aa as i64;
        }
    }

    0
}


pub fn wyt(lco: u32, oj: u64, lqe: u64, pku: u64) -> i64 {
    let ce = crate::process::aei();

    let mut htn: u64 = 0;
    let uts = if oj != 0 && sw(oj, 8, false) {
        unsafe { core::ptr::read(oj as *const u64) }
    } else {
        0
    };

    if let Err(aa) = crate::signals::wje(ce, lco, uts, &mut htn) {
        return aa as i64;
    }

    
    if lqe != 0 && sw(lqe, pku as usize, true) {
        unsafe {
            core::ptr::write(lqe as *mut u64, htn);
        }
    }

    0
}






#[repr(C)]
#[derive(Clone, Copy)]
pub struct Brc {
    pub jmo: u64,
    pub jmp: u64,
}


pub mod rlimit_resource {
    pub const ECS_: u32 = 0;
    pub const ECT_: u32 = 1;
    pub const COY_: u32 = 2;
    pub const CPA_: u32 = 3;
    pub const ECR_: u32 = 4;
    pub const ECZ_: u32 = 5;
    pub const ECY_: u32 = 6;
    pub const COZ_: u32 = 7;
    pub const ECV_: u32 = 8;
    pub const COX_: u32 = 9;
    pub const ECU_: u32 = 10;
    pub const EDC_: u32 = 11;
    pub const ECW_: u32 = 12;
    pub const ECX_: u32 = 13;
    pub const EDA_: u32 = 14;
    pub const EDB_: u32 = 15;
}

const PG_: u64 = !0;


pub fn pqv(lzs: u32, pdt: u64) -> i64 {
    if !sw(pdt, core::mem::size_of::<Brc>(), true) {
        return errno::X;
    }
    
    let ul = unsafe { &mut *(pdt as *mut Brc) };
    
    use rlimit_resource::*;
    match lzs {
        CPA_ => {
            ul.jmo = 8 * 1024 * 1024; 
            ul.jmp = PG_;
        }
        COZ_ => {
            ul.jmo = 1024;
            ul.jmp = 1024 * 1024;
        }
        COX_ | COY_ => {
            ul.jmo = PG_;
            ul.jmp = PG_;
        }
        _ => {
            ul.jmo = PG_;
            ul.jmp = PG_;
        }
    }
    
    0
}


pub fn wyo(ce: u32, lzs: u32, zdj: u64, osh: u64) -> i64 {
    if osh != 0 {
        pqv(lzs, osh)
    } else {
        0
    }
}






pub fn zqq(status: i32) -> i64 {
    crate::log!("[EXIT_GROUP] status={}", status);
    crate::process::cxn(status);
    0 
}


pub fn wyx(ale: u64, len: u64) -> i64 {
    
    0
}


pub fn wxu(ce: u32, yww: u64, zap: u64) -> i64 {
    0
}


pub fn wyn(option: u32, agf: u64, bfx: u64, fcs: u64, gyx: u64) -> i64 {
    const CKY_: u32 = 15;
    const CKX_: u32 = 16;
    
    match option {
        CKY_ => {
            
            if let Some(j) = daz(agf, 16) {
                crate::log_debug!("[PRCTL] Set thread name: {}", j);
            }
            0
        }
        CKX_ => {
            
            if sw(agf, 16, true) {
                rov(agf, "trustos", 16);
            }
            0
        }
        _ => {
            crate::log_debug!("[PRCTL] Unknown option {}", option);
            0
        }
    }
}


pub fn wyv() -> i64 {
    crate::scheduler::gxc();
    0
}


pub fn wyu(ce: u32, ngv: u64, hs: u64) -> i64 {
    if hs != 0 && sw(hs, ngv as usize, true) {
        
        unsafe {
            core::ptr::ahx(hs as *mut u8, 0xFF, ngv as usize);
        }
    }
    0
}






pub fn daz(ptr: u64, am: usize) -> Option<String> {
    if ptr == 0 || !aov(ptr) {
        return None;
    }
    
    let mut e = String::new();
    for a in 0..am {
        let hbx = ptr + a as u64;
        if !aov(hbx) {
            return None;
        }
        
        let o = unsafe { *(hbx as *const u8) };
        if o == 0 { break; }
        e.push(o as char);
    }
    
    if e.is_empty() { None } else { Some(e) }
}


fn rov(ptr: u64, e: &str, am: usize) {
    let len = e.len().v(am - 1);
    let cs = unsafe { core::slice::bef(ptr as *mut u8, am) };
    cs[..len].dg(&e.as_bytes()[..len]);
    cs[len] = 0; 
}






#[repr(C)]
#[derive(Clone, Copy)]
pub struct Aji {
    pub jav: u64,
    pub cyy: u64,
}


pub fn wzn(da: i32, hon: u64, hoo: u32) -> i64 {
    if !sw(hon, (hoo as usize) * core::mem::size_of::<Aji>(), false) {
        return errno::X;
    }
    
    let lfr = unsafe { core::slice::anh(hon as *const Aji, hoo as usize) };
    let mut es = 0i64;
    
    for crr in lfr {
        if crr.cyy == 0 {
            continue;
        }
        if !sw(crr.jav, crr.cyy as usize, false) {
            return errno::X;
        }
        
        let k = unsafe { core::slice::anh(crr.jav as *const u8, crr.cyy as usize) };
        
        
        if da == 1 || da == 2 {
            for &o in k {
                crate::serial_print!("{}", o as char);
            }
            es += crr.cyy as i64;
        } else {
            match crate::vfs::write(da, k) {
                Ok(bo) => es += bo as i64,
                Err(_) => return if es > 0 { es } else { errno::Abi },
            }
        }
    }
    
    es
}


pub fn wyq(da: i32, hon: u64, hoo: u32) -> i64 {
    if !sw(hon, (hoo as usize) * core::mem::size_of::<Aji>(), false) {
        return errno::X;
    }
    
    let lfr = unsafe { core::slice::anh(hon as *const Aji, hoo as usize) };
    let mut es = 0i64;
    
    for crr in lfr {
        if crr.cyy == 0 {
            continue;
        }
        if !sw(crr.jav, crr.cyy as usize, true) {
            return errno::X;
        }
        
        let k = unsafe { core::slice::bef(crr.jav as *mut u8, crr.cyy as usize) };
        
        match crate::vfs::read(da, k) {
            Ok(bo) => {
                es += bo as i64;
                if bo < crr.cyy as usize {
                    break; 
                }
            }
            Err(_) => return if es > 0 { es } else { errno::Abi },
        }
    }
    
    es
}






pub fn mit(bns: i32) -> i64 {
    if crate::pipe::gkh(bns) {
        return bns as i64; 
    }
    match crate::vfs::ksb(bns) {
        Ok(anp) => anp as i64,
        Err(_) => errno::Fu,
    }
}


pub fn jsb(bns: i32, anp: i32) -> i64 {
    if bns == anp {
        return anp as i64;
    }
    if crate::pipe::gkh(bns) {
        return bns as i64; 
    }
    match crate::vfs::noj(bns, anp) {
        Ok(da) => da as i64,
        Err(_) => errno::Fu,
    }
}






#[repr(C)]
#[derive(Clone, Copy)]
pub struct Bpf {
    pub da: i32,
    pub events: i16,
    pub ctx: i16,
}

const Bom: i16 = 1;
const Bon: i16 = 4;
const Ciq: i16 = 8;
const Cir: i16 = 16;
#[allow(bgr)]
const Cis: i16 = 32;


pub fn pqx(nta: u64, lor: u32, sg: i32) -> i64 {
    if lor == 0 { return 0; }
    let aw = (lor as usize) * core::mem::size_of::<Bpf>();
    if !sw(nta, aw, true) {
        return errno::X;
    }
    
    let aho = unsafe { core::slice::bef(nta as *mut Bpf, lor as usize) };
    
    
    let eao = if sg < 0 {
        u64::O 
    } else if sg == 0 {
        0 
    } else {
        crate::time::evk().akq((sg as u64) * 1_000_000)
    };
    
    loop {
        let mut ack = 0i64;
        for dko in aho.el() {
            dko.ctx = 0;
            if dko.da < 0 { continue; }
            
            if let Some(status) = crate::vfs::owj(dko.da) {
                if (dko.events & Bom)  != 0 && status.bob { dko.ctx |= Bom; }
                if (dko.events & Bon) != 0 && status.bjb { dko.ctx |= Bon; }
                if status.zt  { dko.ctx |= Ciq; }
                if status.fkc { dko.ctx |= Cir; }
            } else {
                dko.ctx = Cis; 
            }
            
            if dko.ctx != 0 { ack += 1; }
        }
        
        if ack > 0 { return ack; }
        
        
        if sg == 0 { return 0; }
        
        
        let iu = crate::time::evk();
        if iu >= eao { return 0; }
        
        
        
        
        let eyp = eao.v(iu.akq(10_000_000));
        crate::thread::eyp(eyp);
    }
}






pub fn wxv(da: i32, nlq: u64, az: u32) -> i64 {
    if !sw(nlq, az as usize, true) {
        return errno::X;
    }
    
    
    
    let path = crate::vfs::iwx();
    
    let ch = match crate::vfs::brx(&path) {
        Ok(aa) => aa,
        Err(_) => return errno::Cbh,
    };
    
    let k = unsafe { core::slice::bef(nlq as *mut u8, az as usize) };
    let mut l = 0usize;
    
    for bt in &ch {
        let bko = bt.j.as_bytes();
        
        let gqo = (8 + 8 + 2 + 1 + bko.len() + 1 + 7) & !7;
        
        if l + gqo > az as usize { break; }
        
        let rta: u8 = match bt.kd {
            crate::vfs::FileType::K  => 4,
            crate::vfs::FileType::Ea    => 8,
            crate::vfs::FileType::Mv => 2,
            crate::vfs::FileType::Bj => 6,
            crate::vfs::FileType::Anh    => 10,
            crate::vfs::FileType::Yc       => 1,
            crate::vfs::FileType::Socket     => 12,
        };
        
        let ptr = &mut k[l..];
        if ptr.len() < gqo { break; }
        
        
        ptr[0..8].dg(&bt.dd.ho());
        
        let jgy = (l + gqo) as u64;
        ptr[8..16].dg(&jgy.ho());
        
        ptr[16..18].dg(&(gqo as u16).ho());
        
        ptr[18] = rta;
        
        let akj = 19;
        let bew = akj + bko.len();
        if bew < ptr.len() {
            ptr[akj..bew].dg(bko);
            ptr[bew] = 0;
        }
        
        for a in (bew + 1)..gqo.v(ptr.len()) {
            ptr[a] = 0;
        }
        
        l += gqo;
    }
    
    l as i64
}






pub fn mja(ges: i32, clu: u64, flags: u32) -> i64 {
    const ZF_: i32 = -100;
    
    let path = match daz(clu, 256) {
        Some(e) => e,
        None => return errno::X,
    };
    
    
    if path.cj('/') || ges == ZF_ {
        return match crate::vfs::aji(&path, crate::vfs::OpenFlags(flags)) {
            Ok(da) => da as i64,
            Err(_) => errno::Il,
        };
    }
    
    
    let wo = {
        let jv = crate::vfs::iwx();
        if jv == "/" {
            alloc::format!("/{}", path)
        } else {
            alloc::format!("{}/{}", jv, path)
        }
    };
    
    match crate::vfs::aji(&wo, crate::vfs::OpenFlags(flags)) {
        Ok(da) => da as i64,
        Err(_) => errno::Il,
    }
}


pub fn wzj(arq: u64) -> i64 {
    let (_, _, ahl, _) = crate::process::dfk();
    if ahl != 0 { return -1; } 
    let path = match daz(arq, 256) {
        Some(ai) => ai,
        None => return errno::X,
    };
    match crate::memory::swap::wwm(&path) {
        Ok(()) => 0,
        Err(_) => -1,
    }
}


pub fn wzi(arq: u64) -> i64 {
    let (_, _, ahl, _) = crate::process::dfk();
    if ahl != 0 { return -1; }
    let path = match daz(arq, 256) {
        Some(ai) => ai,
        None => return errno::X,
    };
    match crate::memory::swap::wwl(&path) {
        Ok(()) => 0,
        Err(_) => -1,
    }
}





use alloc::collections::BTreeMap;


pub mod epoll_flags {
    pub const Cbj: u32 = 0x001;
    pub const Cbl: u32 = 0x004;
    pub const Bfk: u32 = 0x008;
    pub const Bfl: u32 = 0x010;
    pub const Cvh: u32 = 0x2000;
    pub const Cvg: u32 = 0x8000_0000;
    pub const Cbk: u32 = 0x4000_0000;
}


pub mod epoll_op {
    pub const BUC_: i32 = 1;
    pub const BUD_: i32 = 2;
    pub const BUE_: i32 = 3;
}


#[repr(C, packed)]
#[derive(Clone, Copy)]
pub struct Sh {
    pub events: u32,
    pub f: u64,
}


#[derive(Clone)]
struct Aht {
    da: i32,
    events: u32,
    f: u64,
}


pub struct Bfy {
    
    dsc: BTreeMap<i32, Aht>,
}


pub static JX_: Mutex<BTreeMap<i32, Bfy>> = Mutex::new(BTreeMap::new());


static CHQ_: AtomicI32 = AtomicI32::new(500);

use core::sync::atomic::AtomicI32;


pub fn txk(da: i32) -> bool {
    JX_.lock().bgm(&da)
}


pub fn pqr(ddp: u32) -> i64 {
    let da = CHQ_.fetch_add(1, Ordering::SeqCst);
    let flr = Bfy {
        dsc: BTreeMap::new(),
    };
    JX_.lock().insert(da, flr);
    crate::log_debug!("[EPOLL] created fd={}", da);
    da as i64
}


pub fn wxl(dds: i32) -> i64 {
    pqr(0)
}


pub fn wxm(ggj: i32, op: i32, da: i32, ggl: u64) -> i64 {
    use epoll_op::*;
    
    let mut gg = JX_.lock();
    let flr = match gg.ds(&ggj) {
        Some(a) => a,
        None => return errno::Fu,
    };
    
    match op {
        BUC_ => {
            if flr.dsc.bgm(&da) {
                return errno::Cbc;
            }
            if ggl == 0 || !sw(ggl, core::mem::size_of::<Sh>(), false) {
                return errno::X;
            }
            let aiz = unsafe { *(ggl as *const Sh) };
            flr.dsc.insert(da, Aht {
                da,
                events: aiz.events,
                f: aiz.f,
            });
        }
        BUE_ => {
            if !flr.dsc.bgm(&da) {
                return errno::Il;
            }
            if ggl == 0 || !sw(ggl, core::mem::size_of::<Sh>(), false) {
                return errno::X;
            }
            let aiz = unsafe { *(ggl as *const Sh) };
            flr.dsc.insert(da, Aht {
                da,
                events: aiz.events,
                f: aiz.f,
            });
        }
        BUD_ => {
            if flr.dsc.remove(&da).is_none() {
                return errno::Il;
            }
        }
        _ => return errno::Er,
    }
    
    0
}


pub fn pqs(ggj: i32, ite: u64, gml: i32, sg: i32) -> i64 {
    if gml <= 0 {
        return errno::Er;
    }
    let snl = core::mem::size_of::<Sh>();
    let dzh = (gml as usize) * snl;
    if !sw(ite, dzh, true) {
        return errno::X;
    }
    
    let nrg = unsafe {
        core::slice::bef(ite as *mut Sh, gml as usize)
    };
    
    
    let eao = if sg < 0 {
        u64::O
    } else if sg == 0 {
        0
    } else {
        crate::time::evk().akq((sg as u64) * 1_000_000)
    };
    
    loop {
        
        let dsc: Vec<Aht> = {
            let gg = JX_.lock();
            match gg.get(&ggj) {
                Some(fi) => fi.dsc.alv().abn().collect(),
                None => return errno::Fu,
            }
        };
        
        let mut ack = 0usize;
        for flt in &dsc {
            if ack >= gml as usize { break; }
            
            let mut ctx = 0u32;
            if let Some(status) = crate::vfs::owj(flt.da) {
                if status.bob { ctx |= epoll_flags::Cbj; }
                if status.bjb { ctx |= epoll_flags::Cbl; }
                if status.zt    { ctx |= epoll_flags::Bfk; }
                if status.fkc   { ctx |= epoll_flags::Bfl; }
            }
            
            
            let pch = ctx & (flt.events | epoll_flags::Bfk | epoll_flags::Bfl);
            if pch != 0 {
                nrg[ack] = Sh {
                    events: pch,
                    f: flt.f,
                };
                ack += 1;
            }
        }
        
        if ack > 0 {
            
            let mut gg = JX_.lock();
            if let Some(fi) = gg.ds(&ggj) {
                for a in 0..ack {
                    let f = nrg[a].f;
                    
                    for flt in fi.dsc.xqp() {
                        if flt.f == f && (flt.events & epoll_flags::Cbk) != 0 {
                            flt.events = 0; 
                        }
                    }
                }
            }
            return ack as i64;
        }
        
        
        if sg == 0 { return 0; }
        
        
        let iu = crate::time::evk();
        if iu >= eao { return 0; }
        
        
        let eyp = eao.v(iu.akq(10_000_000));
        crate::thread::eyp(eyp);
    }
}


pub fn wxn(ggj: i32, ite: u64, gml: i32, sg: i32, ycv: u64, ycx: u64) -> i64 {
    pqs(ggj, ite, gml, sg)
}
