



use alloc::string::String;
use alloc::vec::Vec;
use alloc::vec;
use super::LinuxProcess;


pub const DAC_: u64 = 0;
pub const DAV_: u64 = 1;
pub const CZX_: u64 = 2;
pub const CYR_: u64 = 3;
pub const DAM_: u64 = 4;
pub const CZB_: u64 = 5;
pub const CZQ_: u64 = 6;
pub const DAA_: u64 = 7;
pub const CZP_: u64 = 8;
pub const CZT_: u64 = 9;
pub const CZU_: u64 = 10;
pub const CZV_: u64 = 11;
pub const CYO_: u64 = 12;
pub const DAG_: u64 = 13;
pub const DAH_: u64 = 14;
pub const CZO_: u64 = 16;
pub const CYM_: u64 = 21;
pub const CZZ_: u64 = 22;
pub const CYT_: u64 = 32;
pub const CYU_: u64 = 33;
pub const CZW_: u64 = 35;
pub const CZJ_: u64 = 39;
pub const CZA_: u64 = 57;
pub const CYV_: u64 = 59;
pub const CYW_: u64 = 60;
pub const DAU_: u64 = 61;
pub const DAR_: u64 = 63;
pub const CYZ_: u64 = 72;
pub const CZD_: u64 = 79;
pub const CYP_: u64 = 80;
pub const CZR_: u64 = 83;
pub const DAF_: u64 = 84;
pub const DAS_: u64 = 87;
pub const DAD_: u64 = 89;
pub const CZN_: u64 = 102;
pub const CZG_: u64 = 104;
pub const CZF_: u64 = 107;
pub const CZE_: u64 = 108;
pub const CZK_: u64 = 110;
pub const CZI_: u64 = 111;
pub const DAJ_: u64 = 112;
pub const CZH_: u64 = 115;
pub const DAI_: u64 = 116;
pub const CYN_: u64 = 158;
pub const CZM_: u64 = 186;
pub const DAN_: u64 = 201;
pub const CYQ_: u64 = 228;
pub const CYX_: u64 = 231;
pub const CZY_: u64 = 257;
pub const CZS_: u64 = 258;
pub const CZC_: u64 = 262;
pub const DAT_: u64 = 263;
pub const DAE_: u64 = 267;
pub const CYY_: u64 = 269;
pub const DAL_: u64 = 218;
pub const DAK_: u64 = 273;
pub const DAB_: u64 = 302;
pub const CZL_: u64 = 318;


pub const Gk: i64 = -38;
pub const Do: i64 = -2;
pub const Cp: i64 = -9;
pub const Bw: i64 = -22;
pub const Lu: i64 = -12;






pub fn handle_syscall(
    process: &mut LinuxProcess,
    cec: u64,
    arg1: u64,
    arg2: u64,
    aer: u64,
    cfw: u64,
    dhv: u64,
    arg6: u64,
) -> i64 {
    match cec {
        DAC_ => gxl(process, arg1 as i32, arg2, aer as usize),
        DAV_ => gxo(process, arg1 as i32, arg2, aer as usize),
        CZX_ => fcd(process, arg1, arg2 as i32, aer as u32),
        CZY_ => gxk(process, arg1 as i32, arg2, aer as i32, cfw as u32),
        CYR_ => gxc(process, arg1 as i32),
        DAM_ | CZB_ | CZQ_ | CZC_ => eav(process, arg1, arg2),
        CYO_ => fbz(process, arg1),
        CZT_ => fcb(process, arg1, arg2, aer as i32, cfw as i32, dhv as i32, arg6),
        CZV_ => fcc(process, arg1, arg2),
        CZU_ => 0, 
        CYW_ | CYX_ => paa(process, arg1 as i32),
        DAR_ => gxm(arg1),
        CZJ_ => process.pid as i64,
        CZK_ => 1, 
        CZN_ | CZF_ => 0, 
        CZG_ | CZE_ => 0, 
        CZM_ => process.pid as i64,
        CZD_ => gxe(process, arg1, arg2 as usize),
        CYP_ => gxa(process, arg1),
        CYM_ | CYY_ => gwy(process, arg1),
        CZO_ => gxg(process, arg1 as i32, arg2, aer),
        CYZ_ => 0, 
        CYT_ => gxd(process, arg1 as i32),
        CYU_ => fca(process, arg1 as i32, arg2 as i32),
        CZZ_ => paz(process, arg1),
        CZW_ => gxj(arg1, arg2),
        CYQ_ => gxb(arg1 as i32, arg2),
        DAN_ => pby(arg1),
        CZL_ => gxf(arg1, arg2 as usize, aer as u32),
        CYN_ => gwz(arg1, arg2),
        DAL_ => process.pid as i64,
        DAK_ => 0,
        DAB_ => 0,
        DAG_ | DAH_ => 0, 
        CZR_ | CZS_ => gxi(process, arg1, arg2),
        DAS_ | DAT_ | DAF_ => gxn(process, arg1),
        DAD_ | DAE_ => Bw,
        CZH_ => 0,
        DAI_ => 0,
        CZI_ => process.pid as i64,
        DAJ_ => process.pid as i64,
        CZA_ => Gk, 
        CYV_ => Gk, 
        DAU_ => Gk,
        DAA_ => crate::syscall::linux::jku(arg1, arg2 as u32, aer as i32),
        CZP_ => gxh(process, arg1 as i32, arg2 as i64, aer as i32),
        _ => {
            crate::serial_println!("[LINUX] Unhandled syscall: {} (args: {:#x}, {:#x}, {:#x})", 
                cec, arg1, arg2, aer);
            Gk
        }
    }
}





fn gxl(process: &mut LinuxProcess, fd: i32, buf: u64, count: usize) -> i64 {
    if fd < 0 || fd >= 256 {
        return Cp;
    }
    
    match fd {
        0 => {
            
            let cul = unsafe { core::slice::from_raw_parts_mut(buf as *mut u8, count) };
            let mut biv = 0;
            
            
            while biv < count {
                if let Some(c) = crate::keyboard::ya() {
                    cul[biv] = c;
                    biv += 1;
                    if c == b'\n' {
                        break;
                    }
                } else {
                    if biv > 0 {
                        break;
                    }
                    core::hint::spin_loop();
                }
            }
            biv as i64
        }
        _ => {
            
            Cp
        }
    }
}

fn gxo(process: &mut LinuxProcess, fd: i32, buf: u64, count: usize) -> i64 {
    if fd < 0 || fd >= 256 {
        return Cp;
    }
    
    let data = unsafe { core::slice::from_raw_parts(buf as *const u8, count) };
    
    match fd {
        1 | 2 => {
            
            if let Ok(j) = core::str::from_utf8(data) {
                crate::print!("{}", j);
            } else {
                
                for byte in data {
                    crate::print!("{:02x}", byte);
                }
            }
            count as i64
        }
        _ => {
            
            Cp
        }
    }
}

fn fcd(process: &mut LinuxProcess, path_ptr: u64, flags: i32, mode: u32) -> i64 {
    let path = match dxl(path_ptr) {
        Ok(j) => j,
        Err(e) => return e,
    };
    crate::serial_println!("[LINUX] open({}, {:#x}, {:#o})", path, flags, mode);
    
    
    let fd = (3..256).find(|&i| process.fds[i].is_none());
    
    match fd {
        Some(fd) => {
            process.fds[fd] = Some(fd as u32);
            fd as i64
        }
        None => Lu
    }
}

fn gxk(process: &mut LinuxProcess, dirfd: i32, path_ptr: u64, flags: i32, mode: u32) -> i64 {
    
    fcd(process, path_ptr, flags, mode)
}

fn gxc(process: &mut LinuxProcess, fd: i32) -> i64 {
    if fd < 0 || fd >= 256 {
        return Cp;
    }
    process.fds[fd as usize] = None;
    0
}

fn eav(_process: &mut LinuxProcess, _path_ptr: u64, stat_buf: u64) -> i64 {
    
    let stat = unsafe { &mut *(stat_buf as *mut LinuxStat) };
    *stat = LinuxStat::default();
    stat.st_mode = 0o100644; 
    stat.st_size = 0;
    0
}

fn fbz(process: &mut LinuxProcess, addr: u64) -> i64 {
    
    let result = crate::syscall::linux::fbz(addr);
    
    if result > 0 {
        process.brk = result as u64;
    }
    result
}

fn fcb(process: &mut LinuxProcess, addr: u64, length: u64, prot: i32, flags: i32, fd: i32, offset: u64) -> i64 {
    
    crate::syscall::linux::fcb(addr, length, prot as u64, flags as u64, fd as i64, offset)
}

fn fcc(_process: &mut LinuxProcess, addr: u64, length: u64) -> i64 {
    
    crate::syscall::linux::fcc(addr, length)
}

fn paa(process: &mut LinuxProcess, code: i32) -> i64 {
    process.exit_code = Some(code);
    crate::serial_println!("[LINUX] Process {} exited with code {}", process.pid, code);
    code as i64
}

fn gxm(buf: u64) -> i64 {
    #[repr(C)]
    struct Ns {
        sysname: [u8; 65],
        nodename: [u8; 65],
        release: [u8; 65],
        version: [u8; 65],
        machine: [u8; 65],
        domainname: [u8; 65],
    }
    
    let asq = unsafe { &mut *(buf as *mut Ns) };
    
    fn dgt(field: &mut [u8; 65], value: &str) {
        let bytes = value.as_bytes();
        let len = bytes.len().min(64);
        field[..len].copy_from_slice(&bytes[..len]);
        field[len] = 0;
    }
    
    dgt(&mut asq.sysname, "Linux");
    dgt(&mut asq.nodename, "trustos");
    dgt(&mut asq.release, "5.15.0-trustos");
    dgt(&mut asq.version, "#1 SMP TrustOS");
    dgt(&mut asq.machine, "x86_64");
    dgt(&mut asq.domainname, "(none)");
    
    0
}

fn gxe(process: &mut LinuxProcess, buf: u64, size: usize) -> i64 {
    let cwd = process.cwd.as_bytes();
    if cwd.len() + 1 > size {
        return Bw;
    }
    
    let cul = unsafe { core::slice::from_raw_parts_mut(buf as *mut u8, size) };
    cul[..cwd.len()].copy_from_slice(cwd);
    cul[cwd.len()] = 0;
    
    buf as i64
}

fn gxa(process: &mut LinuxProcess, path_ptr: u64) -> i64 {
    let path = match dxl(path_ptr) {
        Ok(j) => j,
        Err(e) => return e,
    };
    process.cwd = path;
    0
}

fn gwy(_process: &mut LinuxProcess, path_ptr: u64) -> i64 {
    let path = match dxl(path_ptr) {
        Ok(j) => j,
        Err(e) => return e,
    };
    if crate::linux::rootfs::exists(&path) {
        0
    } else {
        Do
    }
}

fn gxg(process: &mut LinuxProcess, fd: i32, request: u64, db: u64) -> i64 {
    
    const Nk: u64 = 0x5401;
    const Nl: u64 = 0x5413;
    
    match request {
        Nk => 0, 
        Nl => {
            
            #[repr(C)]
            struct Asb {
                ws_row: u16,
                ws_col: u16,
                ws_xpixel: u16,
                ws_ypixel: u16,
            }
            let asv = unsafe { &mut *(db as *mut Asb) };
            asv.ws_row = 25;
            asv.ws_col = 80;
            asv.ws_xpixel = 0;
            asv.ws_ypixel = 0;
            0
        }
        _ => 0
    }
}

fn gxd(process: &mut LinuxProcess, oldfd: i32) -> i64 {
    if oldfd < 0 || oldfd >= 256 || process.fds[oldfd as usize].is_none() {
        return Cp;
    }
    
    let dbs = (0..256).find(|&i| process.fds[i].is_none());
    match dbs {
        Some(fd) => {
            process.fds[fd] = process.fds[oldfd as usize];
            fd as i64
        }
        None => Lu
    }
}

fn fca(process: &mut LinuxProcess, oldfd: i32, dbs: i32) -> i64 {
    if oldfd < 0 || oldfd >= 256 || dbs < 0 || dbs >= 256 {
        return Cp;
    }
    if process.fds[oldfd as usize].is_none() {
        return Cp;
    }
    
    process.fds[dbs as usize] = process.fds[oldfd as usize];
    dbs as i64
}

fn paz(process: &mut LinuxProcess, pipefd: u64) -> i64 {
    
    let hxx = (3..256).find(|&i| process.fds[i].is_none());
    let lup = hxx.and_then(|f1| (f1+1..256).find(|&i| process.fds[i].is_none()));
    
    match (hxx, lup) {
        (Some(aot), Some(asu)) => {
            process.fds[aot] = Some(aot as u32);
            process.fds[asu] = Some(asu as u32);
            
            let fds = unsafe { &mut *(pipefd as *mut [i32; 2]) };
            fds[0] = aot as i32;
            fds[1] = asu as i32;
            0
        }
        _ => Lu
    }
}

fn gxj(bvk: u64, _rem: u64) -> i64 {
    #[repr(C)]
    struct Fe {
        tv_sec: i64,
        tv_nsec: i64,
    }
    
    let jy = unsafe { &*(bvk as *const Fe) };
    let ayq = (jy.tv_sec as u64) * 1_000_000_000 + (jy.tv_nsec as u64);
    
    
    crate::thread::otp(ayq);
    
    0
}

fn gxb(clk_id: i32, tp: u64) -> i64 {
    #[repr(C)]
    struct Fe {
        tv_sec: i64,
        tv_nsec: i64,
    }
    
    let jy = unsafe { &mut *(tp as *mut Fe) };
    let gx = crate::logger::eg();
    jy.tv_sec = (gx / 1000) as i64;
    jy.tv_nsec = ((gx % 1000) * 1_000_000) as i64;
    
    0
}

fn pby(tloc: u64) -> i64 {
    let time = (crate::logger::eg() / 1000) as i64;
    if tloc != 0 {
        unsafe { *(tloc as *mut i64) = time; }
    }
    time
}

fn gxf(buf: u64, buflen: usize, bej: u32) -> i64 {
    let buffer = unsafe { core::slice::from_raw_parts_mut(buf as *mut u8, buflen) };
    
    
    for byte in buffer.iter_mut() {
        *byte = crate::rng::ixv();
    }
    
    buflen as i64
}

fn gwz(code: u64, addr: u64) -> i64 {
    const MQ_: u64 = 0x1002;
    const MP_: u64 = 0x1003;
    const AAL_: u64 = 0x1001;
    const AAJ_: u64 = 0x1004;
    
    match code {
        MQ_ => {
            
            unsafe {
                core::arch::asm!(
                    "wrfsbase {}",
                    in(reg) addr,
                );
            }
            0
        }
        AAL_ => {
            unsafe {
                core::arch::asm!(
                    "wrgsbase {}",
                    in(reg) addr,
                );
            }
            0
        }
        MP_ | AAJ_ => 0,
        _ => Bw
    }
}

fn gxi(process: &mut LinuxProcess, path_ptr: u64, _mode: u64) -> i64 {
    let path = match dxl(path_ptr) {
        Ok(j) => j,
        Err(e) => return e,
    };
    let kg = alloc::format!("/linux{}", path);
    
    crate::ramfs::bh(|fs| {
        match fs.mkdir(&kg) {
            Ok(()) => 0,
            Err(_) => Do
        }
    })
}

fn gxn(process: &mut LinuxProcess, path_ptr: u64) -> i64 {
    let path = match dxl(path_ptr) {
        Ok(j) => j,
        Err(e) => return e,
    };
    let kg = alloc::format!("/linux{}", path);
    
    crate::ramfs::bh(|fs| {
        match fs.rm(&kg) {
            Ok(()) => 0,
            Err(_) => Do
        }
    })
}

fn gxh(_process: &mut LinuxProcess, fd: i32, offset: i64, whence: i32) -> i64 {
    
    0
}





#[repr(C)]
#[derive(Default)]
struct LinuxStat {
    st_dev: u64,
    st_ino: u64,
    st_nlink: u64,
    st_mode: u32,
    st_uid: u32,
    st_gid: u32,
    __pad0: u32,
    st_rdev: u64,
    st_size: i64,
    st_blksize: i64,
    st_blocks: i64,
    st_atime: i64,
    st_atime_nsec: i64,
    st_mtime: i64,
    st_mtime_nsec: i64,
    st_ctime: i64,
    st_ctime_nsec: i64,
    __unused: [i64; 3],
}

fn dxl(ptr: u64) -> Result<String, i64> {
    if ptr == 0 {
        return Err(-14); 
    }
    
    let mut j = String::new();
    let mut aa = ptr as *const u8;
    
    unsafe {
        while *aa != 0 {
            j.push(*aa as char);
            aa = aa.add(1);
            if j.len() > 4096 {
                break;
            }
        }
    }
    
    Ok(j)
}
