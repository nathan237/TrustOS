



use alloc::string::String;
use alloc::vec::Vec;
use alloc::vec;
use super::LinuxProcess;


pub const CWK_: u64 = 0;
pub const CXD_: u64 = 1;
pub const CWF_: u64 = 2;
pub const CUZ_: u64 = 3;
pub const CWU_: u64 = 4;
pub const CVJ_: u64 = 5;
pub const CVY_: u64 = 6;
pub const CWI_: u64 = 7;
pub const CVX_: u64 = 8;
pub const CWB_: u64 = 9;
pub const CWC_: u64 = 10;
pub const CWD_: u64 = 11;
pub const CUW_: u64 = 12;
pub const CWO_: u64 = 13;
pub const CWP_: u64 = 14;
pub const CVW_: u64 = 16;
pub const CUU_: u64 = 21;
pub const CWH_: u64 = 22;
pub const CVB_: u64 = 32;
pub const CVC_: u64 = 33;
pub const CWE_: u64 = 35;
pub const CVR_: u64 = 39;
pub const CVI_: u64 = 57;
pub const CVD_: u64 = 59;
pub const CVE_: u64 = 60;
pub const CXC_: u64 = 61;
pub const CWZ_: u64 = 63;
pub const CVH_: u64 = 72;
pub const CVL_: u64 = 79;
pub const CUX_: u64 = 80;
pub const CVZ_: u64 = 83;
pub const CWN_: u64 = 84;
pub const CXA_: u64 = 87;
pub const CWL_: u64 = 89;
pub const CVV_: u64 = 102;
pub const CVO_: u64 = 104;
pub const CVN_: u64 = 107;
pub const CVM_: u64 = 108;
pub const CVS_: u64 = 110;
pub const CVQ_: u64 = 111;
pub const CWR_: u64 = 112;
pub const CVP_: u64 = 115;
pub const CWQ_: u64 = 116;
pub const CUV_: u64 = 158;
pub const CVU_: u64 = 186;
pub const CWV_: u64 = 201;
pub const CUY_: u64 = 228;
pub const CVF_: u64 = 231;
pub const CWG_: u64 = 257;
pub const CWA_: u64 = 258;
pub const CVK_: u64 = 262;
pub const CXB_: u64 = 263;
pub const CWM_: u64 = 267;
pub const CVG_: u64 = 269;
pub const CWT_: u64 = 218;
pub const CWS_: u64 = 273;
pub const CWJ_: u64 = 302;
pub const CVT_: u64 = 318;


pub const Pg: i64 = -38;
pub const Il: i64 = -2;
pub const Fu: i64 = -9;
pub const Er: i64 = -22;
pub const Abl: i64 = -12;






pub fn ixo(
    process: &mut LinuxProcess,
    ezk: u64,
    aai: u64,
    agf: u64,
    bfx: u64,
    fcs: u64,
    gyx: u64,
    kax: u64,
) -> i64 {
    match ezk {
        CWK_ => mjb(process, aai as i32, agf, bfx as usize),
        CXD_ => mjf(process, aai as i32, agf, bfx as usize),
        CWF_ => jse(process, aai, agf as i32, bfx as u32),
        CWG_ => mja(process, aai as i32, agf, bfx as i32, fcs as u32),
        CUZ_ => mis(process, aai as i32),
        CWU_ | CVJ_ | CVY_ | CVK_ => icn(process, aai, agf),
        CUW_ => jsa(process, aai),
        CWB_ => jsc(process, aai, agf, bfx as i32, fcs as i32, gyx as i32, kax),
        CWD_ => jsd(process, aai, agf),
        CWC_ => 0, 
        CVE_ | CVF_ => wxp(process, aai as i32),
        CWZ_ => mjc(aai),
        CVR_ => process.ce as i64,
        CVS_ => 1, 
        CVV_ | CVN_ => 0, 
        CVO_ | CVM_ => 0, 
        CVU_ => process.ce as i64,
        CVL_ => miu(process, aai, agf as usize),
        CUX_ => miq(process, aai),
        CUU_ | CVG_ => mio(process, aai),
        CVW_ => miw(process, aai as i32, agf, bfx),
        CVH_ => 0, 
        CVB_ => mit(process, aai as i32),
        CVC_ => jsb(process, aai as i32, agf as i32),
        CWH_ => wyl(process, aai),
        CWE_ => miz(aai, agf),
        CUY_ => mir(aai as i32, agf),
        CWV_ => wzk(aai),
        CVT_ => miv(aai, agf as usize, bfx as u32),
        CUV_ => mip(aai, agf),
        CWT_ => process.ce as i64,
        CWS_ => 0,
        CWJ_ => 0,
        CWO_ | CWP_ => 0, 
        CVZ_ | CWA_ => miy(process, aai, agf),
        CXA_ | CXB_ | CWN_ => mjd(process, aai),
        CWL_ | CWM_ => Er,
        CVP_ => 0,
        CWQ_ => 0,
        CVQ_ => process.ce as i64,
        CWR_ => process.ce as i64,
        CVI_ => Pg, 
        CVD_ => Pg, 
        CXC_ => Pg,
        CWI_ => crate::syscall::linux::pqx(aai, agf as u32, bfx as i32),
        CVX_ => mix(process, aai as i32, agf as i64, bfx as i32),
        _ => {
            crate::serial_println!("[LINUX] Unhandled syscall: {} (args: {:#x}, {:#x}, {:#x})", 
                ezk, aai, agf, bfx);
            Pg
        }
    }
}





fn mjb(process: &mut LinuxProcess, da: i32, k: u64, az: usize) -> i64 {
    if da < 0 || da >= 256 {
        return Fu;
    }
    
    match da {
        0 => {
            
            let gbr = unsafe { core::slice::bef(k as *mut u8, az) };
            let mut dlc = 0;
            
            
            while dlc < az {
                if let Some(r) = crate::keyboard::auw() {
                    gbr[dlc] = r;
                    dlc += 1;
                    if r == b'\n' {
                        break;
                    }
                } else {
                    if dlc > 0 {
                        break;
                    }
                    core::hint::hc();
                }
            }
            dlc as i64
        }
        _ => {
            
            Fu
        }
    }
}

fn mjf(process: &mut LinuxProcess, da: i32, k: u64, az: usize) -> i64 {
    if da < 0 || da >= 256 {
        return Fu;
    }
    
    let f = unsafe { core::slice::anh(k as *const u8, az) };
    
    match da {
        1 | 2 => {
            
            if let Ok(e) = core::str::jg(f) {
                crate::print!("{}", e);
            } else {
                
                for hf in f {
                    crate::print!("{:02x}", hf);
                }
            }
            az as i64
        }
        _ => {
            
            Fu
        }
    }
}

fn jse(process: &mut LinuxProcess, arq: u64, flags: i32, ev: u32) -> i64 {
    let path = match hxb(arq) {
        Ok(e) => e,
        Err(aa) => return aa,
    };
    crate::serial_println!("[LINUX] open({}, {:#x}, {:#o})", path, flags, ev);
    
    
    let da = (3..256).du(|&a| process.aho[a].is_none());
    
    match da {
        Some(da) => {
            process.aho[da] = Some(da as u32);
            da as i64
        }
        None => Abl
    }
}

fn mja(process: &mut LinuxProcess, ges: i32, arq: u64, flags: i32, ev: u32) -> i64 {
    
    jse(process, arq, flags, ev)
}

fn mis(process: &mut LinuxProcess, da: i32) -> i64 {
    if da < 0 || da >= 256 {
        return Fu;
    }
    process.aho[da as usize] = None;
    0
}

fn icn(jyi: &mut LinuxProcess, ybu: u64, wti: u64) -> i64 {
    
    let hm = unsafe { &mut *(wti as *mut LinuxStat) };
    *hm = LinuxStat::default();
    hm.jrk = 0o100644; 
    hm.gsz = 0;
    0
}

fn jsa(process: &mut LinuxProcess, ag: u64) -> i64 {
    
    let result = crate::syscall::linux::jsa(ag);
    
    if result > 0 {
        process.den = result as u64;
    }
    result
}

fn jsc(process: &mut LinuxProcess, ag: u64, go: u64, prot: i32, flags: i32, da: i32, l: u64) -> i64 {
    
    crate::syscall::linux::jsc(ag, go, prot as u64, flags as u64, da as i64, l)
}

fn jsd(jyi: &mut LinuxProcess, ag: u64, go: u64) -> i64 {
    
    crate::syscall::linux::jsd(ag, go)
}

fn wxp(process: &mut LinuxProcess, aj: i32) -> i64 {
    process.nz = Some(aj);
    crate::serial_println!("[LINUX] Process {} exited with code {}", process.ce, aj);
    aj as i64
}

fn mjc(k: u64) -> i64 {
    #[repr(C)]
    struct Afk {
        gtz: [u8; 65],
        gnv: [u8; 65],
        ehl: [u8; 65],
        dk: [u8; 65],
        czk: [u8; 65],
        gfd: [u8; 65],
    }
    
    let cin = unsafe { &mut *(k as *mut Afk) };
    
    fn gwy(buj: &mut [u8; 65], bn: &str) {
        let bf = bn.as_bytes();
        let len = bf.len().v(64);
        buj[..len].dg(&bf[..len]);
        buj[len] = 0;
    }
    
    gwy(&mut cin.gtz, "Linux");
    gwy(&mut cin.gnv, "trustos");
    gwy(&mut cin.ehl, "5.15.0-trustos");
    gwy(&mut cin.dk, "#1 SMP TrustOS");
    gwy(&mut cin.czk, "x86_64");
    gwy(&mut cin.gfd, "(none)");
    
    0
}

fn miu(process: &mut LinuxProcess, k: u64, aw: usize) -> i64 {
    let jv = process.jv.as_bytes();
    if jv.len() + 1 > aw {
        return Er;
    }
    
    let gbr = unsafe { core::slice::bef(k as *mut u8, aw) };
    gbr[..jv.len()].dg(jv);
    gbr[jv.len()] = 0;
    
    k as i64
}

fn miq(process: &mut LinuxProcess, arq: u64) -> i64 {
    let path = match hxb(arq) {
        Ok(e) => e,
        Err(aa) => return aa,
    };
    process.jv = path;
    0
}

fn mio(jyi: &mut LinuxProcess, arq: u64) -> i64 {
    let path = match hxb(arq) {
        Ok(e) => e,
        Err(aa) => return aa,
    };
    if crate::linux::rootfs::aja(&path) {
        0
    } else {
        Il
    }
}

fn miw(process: &mut LinuxProcess, da: i32, request: u64, ji: u64) -> i64 {
    
    const Aev: u64 = 0x5401;
    const Aew: u64 = 0x5413;
    
    match request {
        Aev => 0, 
        Aew => {
            
            #[repr(C)]
            struct Cqn {
                mrf: u16,
                mre: u16,
                mrg: u16,
                mrh: u16,
            }
            let ciw = unsafe { &mut *(ji as *mut Cqn) };
            ciw.mrf = 25;
            ciw.mre = 80;
            ciw.mrg = 0;
            ciw.mrh = 0;
            0
        }
        _ => 0
    }
}

fn mit(process: &mut LinuxProcess, efw: i32) -> i64 {
    if efw < 0 || efw >= 256 || process.aho[efw as usize].is_none() {
        return Fu;
    }
    
    let gnq = (0..256).du(|&a| process.aho[a].is_none());
    match gnq {
        Some(da) => {
            process.aho[da] = process.aho[efw as usize];
            da as i64
        }
        None => Abl
    }
}

fn jsb(process: &mut LinuxProcess, efw: i32, gnq: i32) -> i64 {
    if efw < 0 || efw >= 256 || gnq < 0 || gnq >= 256 {
        return Fu;
    }
    if process.aho[efw as usize].is_none() {
        return Fu;
    }
    
    process.aho[gnq as usize] = process.aho[efw as usize];
    gnq as i64
}

fn wyl(process: &mut LinuxProcess, vid: u64) -> i64 {
    
    let nsz = (3..256).du(|&a| process.aho[a].is_none());
    let srj = nsz.and_then(|nsp| (nsp+1..256).du(|&a| process.aho[a].is_none()));
    
    match (nsz, srj) {
        (Some(cbh), Some(civ)) => {
            process.aho[cbh] = Some(cbh as u32);
            process.aho[civ] = Some(civ as u32);
            
            let aho = unsafe { &mut *(vid as *mut [i32; 2]) };
            aho[0] = cbh as i32;
            aho[1] = civ as i32;
            0
        }
        _ => Abl
    }
}

fn miz(ehq: u64, ycd: u64) -> i64 {
    #[repr(C)]
    struct Ml {
        ekg: i64,
        fxn: i64,
    }
    
    let wi = unsafe { &*(ehq as *const Ml) };
    let csw = (wi.ekg as u64) * 1_000_000_000 + (wi.fxn as u64);
    
    
    crate::thread::wpl(csw);
    
    0
}

fn mir(yio: i32, aaz: u64) -> i64 {
    #[repr(C)]
    struct Ml {
        ekg: i64,
        fxn: i64,
    }
    
    let wi = unsafe { &mut *(aaz as *mut Ml) };
    let qb = crate::logger::lh();
    wi.ekg = (qb / 1000) as i64;
    wi.fxn = ((qb % 1000) * 1_000_000) as i64;
    
    0
}

fn wzk(ptp: u64) -> i64 {
    let time = (crate::logger::lh() / 1000) as i64;
    if ptp != 0 {
        unsafe { *(ptp as *mut i64) = time; }
    }
    time
}

fn miv(k: u64, nam: usize, ddp: u32) -> i64 {
    let bi = unsafe { core::slice::bef(k as *mut u8, nam) };
    
    
    for hf in bi.el() {
        *hf = crate::rng::ozi();
    }
    
    nam as i64
}

fn mip(aj: u64, ag: u64) -> i64 {
    const LT_: u64 = 0x1002;
    const LS_: u64 = 0x1003;
    const ZE_: u64 = 0x1001;
    const ZC_: u64 = 0x1004;
    
    match aj {
        LT_ => {
            
            unsafe {
                core::arch::asm!(
                    "wrfsbase {}",
                    in(reg) ag,
                );
            }
            0
        }
        ZE_ => {
            unsafe {
                core::arch::asm!(
                    "wrgsbase {}",
                    in(reg) ag,
                );
            }
            0
        }
        LS_ | ZC_ => 0,
        _ => Er
    }
}

fn miy(process: &mut LinuxProcess, arq: u64, ybh: u64) -> i64 {
    let path = match hxb(arq) {
        Ok(e) => e,
        Err(aa) => return aa,
    };
    let wo = alloc::format!("/linux{}", path);
    
    crate::ramfs::fh(|fs| {
        match fs.ut(&wo) {
            Ok(()) => 0,
            Err(_) => Il
        }
    })
}

fn mjd(process: &mut LinuxProcess, arq: u64) -> i64 {
    let path = match hxb(arq) {
        Ok(e) => e,
        Err(aa) => return aa,
    };
    let wo = alloc::format!("/linux{}", path);
    
    crate::ramfs::fh(|fs| {
        match fs.hb(&wo) {
            Ok(()) => 0,
            Err(_) => Il
        }
    })
}

fn mix(jyi: &mut LinuxProcess, da: i32, l: i64, gwp: i32) -> i64 {
    
    0
}





#[repr(C)]
#[derive(Default)]
struct LinuxStat {
    pnh: u64,
    pnj: u64,
    pnn: u64,
    jrk: u32,
    pnq: u32,
    pni: u32,
    xxo: u32,
    mhg: u64,
    gsz: i64,
    mhe: i64,
    pne: i64,
    pnc: i64,
    pnd: i64,
    pnk: i64,
    pnl: i64,
    pnf: i64,
    png: i64,
    xxp: [i64; 3],
}

fn hxb(ptr: u64) -> Result<String, i64> {
    if ptr == 0 {
        return Err(-14); 
    }
    
    let mut e = String::new();
    let mut ai = ptr as *const u8;
    
    unsafe {
        while *ai != 0 {
            e.push(*ai as char);
            ai = ai.add(1);
            if e.len() > 4096 {
                break;
            }
        }
    }
    
    Ok(e)
}
