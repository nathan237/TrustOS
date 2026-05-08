




pub mod linux;

use crate::memory::{ij, ux};


pub mod nr {
    
    pub use super::linux::nr::*;
    
    
    pub const CYS_: u64 = 0x1000;
    pub const DAQ_: u64 = 0x1001;
    pub const DAP_: u64 = 0x1002;
    pub const DAO_: u64 = 0x1003;
}


pub mod errno {
    pub const Auq: i64 = -1;
    pub const Do: i64 = -2;
    pub const Auy: i64 = -3;
    pub const Aue: i64 = -4;
    pub const Lp: i64 = -5;
    pub const Aup: i64 = -6;
    pub const Atz: i64 = -7;
    pub const Aum: i64 = -8;
    pub const Cp: i64 = -9;
    pub const Ait: i64 = -10;
    pub const Ya: i64 = -11;
    pub const Lu: i64 = -12;
    pub const Aua: i64 = -13;
    pub const P: i64 = -14;
    pub const Auo: i64 = -15;
    pub const Aub: i64 = -16;
    pub const Aiu: i64 = -17;
    pub const Avd: i64 = -18;
    pub const Aul: i64 = -19;
    pub const Aiz: i64 = -20;
    pub const Aug: i64 = -21;
    pub const Bw: i64 = -22;
    pub const Auk: i64 = -23;
    pub const Auh: i64 = -24;
    pub const Aja: i64 = -25;
    pub const Auz: i64 = -26;
    pub const Aud: i64 = -27;
    pub const Aun: i64 = -28;
    pub const Aux: i64 = -29;
    pub const Auv: i64 = -30;
    pub const Aui: i64 = -31;
    pub const Aur: i64 = -32;
    pub const Auc: i64 = -33;
    pub const Auu: i64 = -34;
    pub const Gk: i64 = -38;
    pub const Avc: i64 = Ya;
}


pub fn init() {
    crate::log!("[SYSCALL] Linux-compatible syscall interface initialized");
}






pub fn qkc(num: u64, eb: u64, fy: u64, kb: u64) -> u64 {
    
    let ret = idi(num, eb, fy, kb, 0, 0, 0);

    
    crate::lab_mode::trace_bus::fuj(num, [eb, fy, kb], ret);

    
    let pid = crate::process::pe();
    if pid > 0 {
        if let Some(signo) = crate::signals::kjs(pid) {
            
            
            if !crate::process::is_running(pid) && crate::userland::ers() {
                unsafe { crate::userland::azi(-(signo as i32)); }
            }
        }
    }

    ret as u64
}


pub fn idi(num: u64, eb: u64, fy: u64, kb: u64, aeq: u64, a5: u64, a6: u64) -> i64 {
    use linux::nr::*;
    
    match num {
        
        Ba => gxl(eb as i32, fy, kb as usize),
        Bh => gxo(eb as i32, fy, kb as usize),
        Aby => fcd(eb, fy as u32),
        Rf => gxc(eb as i32),
        Ael | Aas => linux::eav(eb, fy),
        Yq => linux::jkr(eb as i32, fy),
        Aaq => gxh(eb as i32, fy as i64, kb as u32),
        Anr => linux::pbe(eb as i32, fy, kb as u32),
        Agm => linux::pcb(eb as i32, fy, kb as u32),
        Qy => linux::gwy(eb, fy as u32),
        Acx => linux::pbd(eb, fy, kb),
        Zy => linux::gxg(eb as i32, fy, kb),
        Ajp => linux::pae(eb as i32, fy as u32, kb),
        Sk => gxe(eb, fy as usize),
        Wv => gxa(eb),
        Aba => gxi(eb),
        Afq => gxn(eb),
        Xo => linux::gxd(eb as i32),
        Xp => linux::fca(eb as i32, fy as i32),
        Air => linux::fca(eb as i32, fy as i32),
        Ace => linux::jku(eb, fy as u32, kb as i32),
        Akh => linux::pah(eb as i32, fy, kb as u32),
        Abz => linux::gxk(eb as i32, fy, kb as u32),
        Abp => linux::pay(eb as i32, fy, kb, aeq as u32),
        
        
        Tn => linux::fcb(eb, fy, kb, aeq, a5 as i64, a6),
        Tp => linux::fcc(eb, fy),
        Abc => linux::pax(eb, fy, kb),
        Rd => linux::fbz(eb),
        
        
        So => linux::pam(),
        Akl => linux::pao(),
        Akr => linux::jkt(),
        Sp | Sm => linux::pau(),
        Sn | Sl => linux::pai(),
        Apf => linux::pbt(eb as u32),
        Aoy => linux::pbn(eb as u32),
        Apb => linux::pbq(eb as u32, fy as u32),
        Apa => linux::pbp(eb as u32, fy as u32),
        Aoz => linux::pbo(eb as u32, fy as u32),
        Akk => linux::pal(),
        Apd => linux::pbr(),
        Akj => linux::pak(eb as u32),
        Ako => linux::pap(eb as u32),
        Ahr => linux::ozs(eb),
        Ard => linux::pbz(eb as u32),
        Ahp => linux::ozr(eb, fy as u32),
        Ajn => linux::pab(eb as i32, fy as u32),
        Ahq => linux::jkn(eb, fy as u32, kb as u32),
        Ajo => linux::pac(eb as i32, fy as u32, kb as u32),
        Amf => linux::jkn(eb, fy as u32, kb as u32),
        Yo => jkq(),
        Arj => jkq(),
        Ahs => ozt(eb, fy, kb),
        Yg => ozz(eb, fy, kb),
        Oq => { 
            crate::process::exit(eb as i32);
            if crate::userland::ers() { 
                
                
                if crate::userland::MH_.load(core::sync::atomic::Ordering::SeqCst) != 0 {
                    crate::userland::haz(eb as i32);
                } else {
                    unsafe { crate::userland::azi(eb as i32); }
                }
            }
            0
        }
        ADN_ => { 
            crate::process::exit(eb as i32);
            if crate::userland::ers() { 
                if crate::userland::MH_.load(core::sync::atomic::Ordering::SeqCst) != 0 {
                    crate::userland::haz(eb as i32);
                } else {
                    unsafe { crate::userland::azi(eb as i32); }
                }
            }
            0
        }
        Arv => pca(eb as i32, fy, kb as u32),
        Ama => pav(eb as i32, fy as i32),
        AJR_ => linux::pbm(eb),
        
        
        AAK_ => linux::gwz(eb, fy),
        
        
        Vq => linux::gxm(eb),
        
        
        BPE_ => linux::gxb(eb as u32, fy),
        Aks => linux::pas(eb, fy),
        Amv => linux::gxj(eb, fy),
        
        
        CTF_ => linux::pbg(eb as u32, fy, kb, aeq),
        CTG_ => linux::pbh(eb as u32, fy, kb, aeq),
        AJF_ => 0,
        Aph => 0,
        
        
        CUD_ => linux::pbj(),
        CUB_ => linux::pbi(eb as u32, fy, kb),
        CUC_ => 0,
        
        
        Ajz => paf(eb, fy as u32, kb as u32),
        CWD_ => linux::pbl(eb, fy),
        CAF_ => linux::pag(eb as u32, fy, kb),
        
        
        Akn => linux::jks(eb as u32, fy),
        Apc => 0,
        Ani => linux::pbc(eb as u32, fy as u32, kb, aeq),
        
        
        Akm => linux::gxf(eb, fy, kb),
        
        
        Apq => pbv(eb as u16, fy as u32, kb as u32),
        Ahu => ozu(eb as i32, fy, kb as usize),
        Agt => ozp(eb as i32, fy, kb),
        Aov => jkw(eb as i32, fy, kb as usize, aeq as u32, a5, a6 as usize),
        Ans => jkv(eb as i32, fy, kb as usize, aeq as u32, a5, a6),
        Iu => pbu(eb as i32, fy as i32),
        Wm => ozq(eb as i32, fy, kb as usize),
        Amh => paw(eb as i32, fy as u32),
        Akp => paq(eb as i32, fy, kb),
        Aki => paj(eb as i32, fy, kb),
        Ape => pbs(eb as i32, fy as i32, kb as i32, aeq, a5 as usize),
        Akq => par(eb as i32, fy as i32, kb as i32, aeq, a5),
        Aou => pbk(eb as i32, fy, kb as u32),
        Ant => pbf(eb as i32, fy, kb as u32),
        
        
        Anc => pba(eb, fy as u32),
        
        
        BWV_ => linux::ozw(eb as i32),
        BXC_ => linux::jkp(eb as i32, fy, kb as i32, aeq as i32),
        BWX_ => linux::ozx(eb as i32, fy as i32, kb as i32, aeq),
        BXB_ => linux::ozy(eb as i32, fy, kb as i32, aeq as i32, a5, a6),
        BWW_ => linux::jko(eb as u32),
        
        
        Anh => linux::pbb(eb as u32, fy, kb, aeq, a5),
        Apz => linux::pbx(eb),
        Apy => linux::pbw(eb),
        
        
        nr::CYS_ => ozv(eb, fy as usize),
        nr::DAQ_ => { crate::ipc::onx(eb); 0 },
        nr::DAP_ => crate::ipc::odo(eb) as i64,
        nr::DAO_ => crate::ipc::kzf() as i64,
        
        _ => {
            crate::log_debug!("[SYSCALL] Unknown: {} (0x{:x})", num, num);
            errno::Gk
        }
    }
}

fn gxl(fd: i32, buf_ptr: u64, count: usize) -> i64 {
    if buf_ptr == 0 || count == 0 {
        return errno::Bw;
    }
    
    
    if !ij(buf_ptr, count, true) {
        crate::log_warn!("[SYSCALL] read: invalid user pointer {:#x}", buf_ptr);
        return errno::P;
    }
    
    
    if crate::pipe::dab(fd) {
        let buffer = unsafe { core::slice::from_raw_parts_mut(buf_ptr as *mut u8, count) };
        return crate::pipe::read(fd, buffer);
    }
    
    let buffer = unsafe { core::slice::from_raw_parts_mut(buf_ptr as *mut u8, count) };
    match crate::vfs::read(fd, buffer) {
        Ok(ae) => ae as i64,
        Err(_) => errno::Lp,
    }
}

fn gxo(fd: i32, buf_ptr: u64, count: usize) -> i64 {
    if buf_ptr == 0 || count == 0 {
        return errno::Bw;
    }
    
    
    if !ij(buf_ptr, count, false) {
        crate::log_warn!("[SYSCALL] write: invalid user pointer {:#x}", buf_ptr);
        return errno::P;
    }
    
    let buffer = unsafe { core::slice::from_raw_parts(buf_ptr as *const u8, count) };
    
    
    if fd == 1 || fd == 2 {
        for &b in buffer { crate::serial_print!("{}", b as char); }
        return count as i64;
    }
    
    
    if crate::pipe::dab(fd) {
        return crate::pipe::write(fd, buffer);
    }
    
    match crate::vfs::write(fd, buffer) {
        Ok(ae) => ae as i64,
        Err(_) => errno::Lp,
    }
}

fn fcd(path_ptr: u64, flags: u32) -> i64 {
    let path = match dde(path_ptr, 256) {
        Some(j) => j,
        None => return errno::P,
    };
    match crate::vfs::open(&path, crate::vfs::OpenFlags(flags)) {
        Ok(fd) => fd as i64,
        Err(_) => errno::Do,
    }
}

fn gxc(fd: i32) -> i64 {
    
    if linux::msn(fd) {
        crate::syscall::linux::KR_.lock().remove(&fd);
        return 0;
    }
    
    if crate::pipe::dab(fd) {
        return crate::pipe::close(fd);
    }
    match crate::vfs::close(fd) {
        Ok(()) => 0,
        Err(_) => errno::Cp,
    }
}


fn pba(pipefd_ptr: u64, bej: u32) -> i64 {
    if !ij(pipefd_ptr, 8, true) {
        return errno::P;
    }
    let (aot, asu) = crate::pipe::create();
    unsafe {
        let ptr = pipefd_ptr as *mut i32;
        *ptr = aot;
        *ptr.add(1) = asu;
    }
    crate::log_debug!("[SYSCALL] pipe2: read_fd={}, write_fd={}", aot, asu);
    0
}

fn gxi(path_ptr: u64) -> i64 {
    let path = match dde(path_ptr, 256) {
        Some(j) => j,
        None => return errno::P,
    };
    match crate::vfs::mkdir(&path) {
        Ok(()) => 0,
        Err(_) => errno::Lp,
    }
}

fn gxn(path_ptr: u64) -> i64 {
    let path = match dde(path_ptr, 256) {
        Some(j) => j,
        None => return errno::P,
    };
    match crate::vfs::unlink(&path) {
        Ok(()) => 0,
        Err(_) => errno::Do,
    }
}

fn ozv(buf_ptr: u64, len: usize) -> i64 {
    if buf_ptr == 0 { return errno::P; }
    
    
    if !ij(buf_ptr, len, false) {
        return errno::P;
    }
    
    let buffer = unsafe { core::slice::from_raw_parts(buf_ptr as *const u8, len) };
    for &b in buffer { crate::serial_print!("{}", b as char); }
    len as i64
}


fn dde(ptr: u64, max: usize) -> Option<alloc::string::String> {
    if ptr == 0 { return None; }
    
    
    if !ux(ptr) {
        crate::log_warn!("[SYSCALL] read_cstring: kernel address {:#x}", ptr);
        return None;
    }
    
    let mut j = alloc::string::String::new();
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









fn ozt(flags: u64, dn: u64, entry: u64) -> i64 {
    
    
    
    if dn == 0 || entry == 0 {
        return errno::Bw;
    }
    
    
    if !ux(dn) || !ux(entry) {
        return errno::P;
    }
    
    let pid = crate::process::pe();
    let tid = crate::thread::jhc(pid, "user_thread", entry, dn, 0);
    
    tid as i64
}






fn paf(addr: u64, op: u32, val: u32) -> i64 {
    const AED_: u32 = 0;
    const AEE_: u32 = 1;
    const AEC_: u32 = 128;
    
    let op = op & !AEC_;
    
    if !ux(addr) {
        return errno::P;
    }
    
    match op {
        AED_ => {
            
            let current = unsafe { *(addr as *const u32) };
            if current != val {
                return errno::Ya;
            }
            
            
            let hdl = crate::thread::current_tid();
            
            
            crate::thread::ajc();
            0
        }
        AEE_ => {
            
            
            
            0
        }
        _ => errno::Gk,
    }
}





fn gxh(fd: i32, offset: i64, whence: u32) -> i64 {
    match crate::vfs::nbd(fd, offset, whence) {
        Ok(pos) => pos as i64,
        Err(_) => errno::Bw,
    }
}

fn gxe(buf: u64, size: usize) -> i64 {
    if !ij(buf, size, true) {
        return errno::P;
    }
    
    let cwd = crate::vfs::eof();
    let bytes = cwd.as_bytes();
    let len = bytes.len().min(size - 1);
    
    unsafe {
        let dst = core::slice::from_raw_parts_mut(buf as *mut u8, size);
        dst[..len].copy_from_slice(&bytes[..len]);
        dst[len] = 0;
    }
    
    buf as i64
}

fn gxa(path_ptr: u64) -> i64 {
    let path = match dde(path_ptr, 256) {
        Some(j) => j,
        None => return errno::P,
    };
    
    match crate::vfs::kir(&path) {
        Ok(()) => 0,
        Err(_) => errno::Do,
    }
}

fn jkq() -> i64 {
    match crate::process::lxk() {
        Ok(pid) => pid as i64,
        Err(_) => errno::Lu,
    }
}

fn ozz(pathname: u64, _argv: u64, _envp: u64) -> i64 {
    let path = match dde(pathname, 256) {
        Some(j) => j,
        None => return errno::P,
    };
    
    
    let mut bxn: alloc::vec::Vec<alloc::string::String> = alloc::vec::Vec::new();
    if _argv != 0 && ux(_argv) {
        for i in 0..256u64 {
            let ixa = _argv + i * 8;
            if !ux(ixa) { break; }
            let ptr = unsafe { *(ixa as *const u64) };
            if ptr == 0 { break; }
            if let Some(j) = dde(ptr, 256) {
                bxn.push(j);
            } else {
                break;
            }
        }
    }
    if bxn.is_empty() {
        bxn.push(path.clone());
    }
    let jxo: alloc::vec::Vec<&str> = bxn.iter().map(|j| j.as_str()).collect();
    
    match crate::exec::lsh(&path, &jxo, &[]) {
        Ok(()) => 0, 
        Err(_) => errno::Do,
    }
}

fn pca(pid: i32, wstatus: u64, options: u32) -> i64 {
    let bwg = if pid > 0 { pid as u32 } else { 0 };
    let jrf = options & 1 != 0; 
    
    
    let ndn: u32 = if jrf { 1 } else { 5000 };
    
    for _ in 0..ndn {
        match crate::process::bqb(bwg) {
            Ok(status) => {
                if wstatus != 0 && ij(wstatus, 4, true) {
                    
                    unsafe { *(wstatus as *mut i32) = (status & 0xFF) << 8; }
                }
                return bwg as i64;
            }
            Err(_) => {
                if jrf { return 0; }
                crate::thread::ajc();
            }
        }
    }
    errno::Ait
}

fn pav(pid: i32, sig: i32) -> i64 {
    if pid <= 0 {
        return errno::Bw;
    }
    
    let def = crate::process::pe();
    
    match crate::signals::bne(pid as u32, sig as u32, def) {
        Ok(()) => 0,
        Err(e) => e as i64,
    }
}






fn pbv(domain: u16, sock_type: u32, protocol: u32) -> i64 {
    match crate::netstack::socket::socket(domain, sock_type, protocol) {
        Ok(fd) => fd as i64,
        Err(e) => e as i64,
    }
}


fn ozu(fd: i32, addr_ptr: u64, addr_len: usize) -> i64 {
    use crate::netstack::socket::SockAddrIn;
    
    if addr_len < SockAddrIn::Z {
        return errno::Bw;
    }
    
    if !ij(addr_ptr, addr_len, false) {
        return errno::P;
    }
    
    let addr = unsafe { *(addr_ptr as *const SockAddrIn) };
    
    match crate::netstack::socket::connect(fd, &addr) {
        Ok(()) => 0,
        Err(e) => e as i64,
    }
}


fn ozq(fd: i32, addr_ptr: u64, addr_len: usize) -> i64 {
    use crate::netstack::socket::SockAddrIn;
    
    if addr_len < SockAddrIn::Z {
        return errno::Bw;
    }
    
    if !ij(addr_ptr, addr_len, false) {
        return errno::P;
    }
    
    let addr = unsafe { *(addr_ptr as *const SockAddrIn) };
    
    match crate::netstack::socket::fjf(fd, &addr) {
        Ok(()) => 0,
        Err(e) => e as i64,
    }
}


fn paw(fd: i32, backlog: u32) -> i64 {
    match crate::netstack::socket::iks(fd, backlog) {
        Ok(()) => 0,
        Err(e) => e as i64,
    }
}


fn ozp(fd: i32, addr_ptr: u64, addr_len_ptr: u64) -> i64 {
    match crate::netstack::socket::jtj(fd, addr_ptr, addr_len_ptr) {
        Ok(ue) => ue as i64,
        Err(e) => e as i64,
    }
}


fn jkw(fd: i32, buf_ptr: u64, len: usize, flags: u32, addr_ptr: u64, addr_len: usize) -> i64 {
    use crate::netstack::socket::SockAddrIn;
    
    if !ij(buf_ptr, len, false) {
        return errno::P;
    }
    
    let data = unsafe { core::slice::from_raw_parts(buf_ptr as *const u8, len) };
    
    if addr_ptr != 0 && addr_len >= SockAddrIn::Z {
        
        if !ij(addr_ptr, addr_len, false) {
            return errno::P;
        }
        let addr = unsafe { *(addr_ptr as *const SockAddrIn) };
        match crate::netstack::socket::ooe(fd, data, flags, &addr) {
            Ok(ae) => ae as i64,
            Err(e) => e as i64,
        }
    } else {
        
        match crate::netstack::socket::send(fd, data, flags) {
            Ok(ae) => ae as i64,
            Err(e) => e as i64,
        }
    }
}


fn jkv(fd: i32, buf_ptr: u64, len: usize, flags: u32, addr_ptr: u64, addr_len_ptr: u64) -> i64 {
    if !ij(buf_ptr, len, true) {
        return errno::P;
    }
    
    let buf = unsafe { core::slice::from_raw_parts_mut(buf_ptr as *mut u8, len) };
    
    
    match crate::netstack::socket::recv(fd, buf, flags) {
        Ok(ae) => ae as i64,
        Err(e) => e as i64,
    }
}


fn pbu(fd: i32, _how: i32) -> i64 {
    match crate::netstack::socket::close(fd) {
        Ok(()) => 0,
        Err(e) => e as i64,
    }
}


fn paq(fd: i32, addr_ptr: u64, addr_len_ptr: u64) -> i64 {
    use crate::netstack::socket::{SockAddrIn, BC_};

    if addr_ptr == 0 || addr_len_ptr == 0 {
        return errno::P;
    }
    if !ij(addr_len_ptr, 4, true) {
        return errno::P;
    }

    let len = unsafe { *(addr_len_ptr as *const u32) } as usize;
    if len < SockAddrIn::Z || !ij(addr_ptr, SockAddrIn::Z, true) {
        return errno::Bw;
    }

    let bs = BC_.lock();
    let ih = match bs.get(&fd) {
        Some(j) => j,
        None => return errno::Cp,
    };

    let addr = ih.local_addr.unwrap_or_default();
    unsafe {
        *(addr_ptr as *mut SockAddrIn) = addr;
        *(addr_len_ptr as *mut u32) = SockAddrIn::Z as u32;
    }
    0
}


fn paj(fd: i32, addr_ptr: u64, addr_len_ptr: u64) -> i64 {
    use crate::netstack::socket::{SockAddrIn, BC_};

    if addr_ptr == 0 || addr_len_ptr == 0 {
        return errno::P;
    }
    if !ij(addr_len_ptr, 4, true) {
        return errno::P;
    }

    let len = unsafe { *(addr_len_ptr as *const u32) } as usize;
    if len < SockAddrIn::Z || !ij(addr_ptr, SockAddrIn::Z, true) {
        return errno::Bw;
    }

    let bs = BC_.lock();
    let ih = match bs.get(&fd) {
        Some(j) => j,
        None => return errno::Cp,
    };

    let addr = match ih.remote_addr {
        Some(a) => a,
        None => return -107, 
    };
    unsafe {
        *(addr_ptr as *mut SockAddrIn) = addr;
        *(addr_len_ptr as *mut u32) = SockAddrIn::Z as u32;
    }
    0
}


fn pbk(fd: i32, msg_ptr: u64, flags: u32) -> i64 {
    if msg_ptr == 0 || !ij(msg_ptr, 56, false) {
        return errno::P;
    }
    
    let gip = unsafe { *(msg_ptr as *const u64) };
    let name_len = unsafe { *((msg_ptr + 8) as *const u32) } as usize;
    let bto  = unsafe { *((msg_ptr + 16) as *const u64) };
    let iov_len  = unsafe { *((msg_ptr + 24) as *const u64) } as usize;

    if iov_len == 0 || bto == 0 {
        return 0;
    }
    
    if !ij(bto, 16, false) {
        return errno::P;
    }
    let base = unsafe { *(bto as *const u64) };
    let len  = unsafe { *((bto + 8) as *const u64) } as usize;

    jkw(fd, base, len, flags, gip, name_len)
}


fn pbf(fd: i32, msg_ptr: u64, flags: u32) -> i64 {
    if msg_ptr == 0 || !ij(msg_ptr, 56, true) {
        return errno::P;
    }
    let gip = unsafe { *(msg_ptr as *const u64) };
    let nhn = unsafe { (msg_ptr + 8) as u64 };
    let bto  = unsafe { *((msg_ptr + 16) as *const u64) };
    let iov_len  = unsafe { *((msg_ptr + 24) as *const u64) } as usize;

    if iov_len == 0 || bto == 0 {
        return 0;
    }
    if !ij(bto, 16, false) {
        return errno::P;
    }
    let base = unsafe { *(bto as *const u64) };
    let len  = unsafe { *((bto + 8) as *const u64) } as usize;

    jkv(fd, base, len, flags, gip, nhn)
}


fn pbs(fd: i32, level: i32, optname: i32, optval: u64, cca: usize) -> i64 {
    if cca > 0 && !ij(optval, cca, false) {
        return errno::P;
    }
    
    let data = if cca > 0 {
        unsafe { core::slice::from_raw_parts(optval as *const u8, cca) }
    } else {
        &[]
    };
    
    match crate::netstack::socket::opz(fd, level, optname, data) {
        Ok(()) => 0,
        Err(e) => e as i64,
    }
}


fn par(fd: i32, level: i32, optname: i32, optval: u64, optlen_ptr: u64) -> i64 {
    if optval == 0 || optlen_ptr == 0 {
        return errno::P;
    }
    
    if !ij(optlen_ptr, 4, true) {
        return errno::P;
    }
    
    let cca = unsafe { *(optlen_ptr as *const u32) } as usize;
    
    if cca > 0 && !ij(optval, cca, true) {
        return errno::P;
    }
    
    let buf = unsafe { core::slice::from_raw_parts_mut(optval as *mut u8, cca) };
    
    match crate::netstack::socket::meh(fd, level, optname, buf) {
        Ok(len) => {
            unsafe { *(optlen_ptr as *mut u32) = len as u32; }
            0
        }
        Err(e) => e as i64,
    }
}
