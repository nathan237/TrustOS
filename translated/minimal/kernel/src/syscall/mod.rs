




pub mod linux;

use crate::memory::{sw, aov};


pub mod nr {
    
    pub use super::linux::nr::*;
    
    
    pub const CVA_: u64 = 0x1000;
    pub const CWY_: u64 = 0x1001;
    pub const CWX_: u64 = 0x1002;
    pub const CWW_: u64 = 0x1003;
}


pub mod errno {
    pub const Cve: i64 = -1;
    pub const Il: i64 = -2;
    pub const Cvm: i64 = -3;
    pub const Cus: i64 = -4;
    pub const Abi: i64 = -5;
    pub const Cvd: i64 = -6;
    pub const Cun: i64 = -7;
    pub const Cva: i64 = -8;
    pub const Fu: i64 = -9;
    pub const Cbb: i64 = -10;
    pub const Bfc: i64 = -11;
    pub const Abl: i64 = -12;
    pub const Cuo: i64 = -13;
    pub const X: i64 = -14;
    pub const Cvc: i64 = -15;
    pub const Cup: i64 = -16;
    pub const Cbc: i64 = -17;
    pub const Cvr: i64 = -18;
    pub const Cuz: i64 = -19;
    pub const Cbh: i64 = -20;
    pub const Cuu: i64 = -21;
    pub const Er: i64 = -22;
    pub const Cuy: i64 = -23;
    pub const Cuv: i64 = -24;
    pub const Cbi: i64 = -25;
    pub const Cvn: i64 = -26;
    pub const Cur: i64 = -27;
    pub const Cvb: i64 = -28;
    pub const Cvl: i64 = -29;
    pub const Cvj: i64 = -30;
    pub const Cuw: i64 = -31;
    pub const Cvf: i64 = -32;
    pub const Cuq: i64 = -33;
    pub const Cvi: i64 = -34;
    pub const Pg: i64 = -38;
    pub const Cvq: i64 = Bfc;
}


pub fn init() {
    crate::log!("[SYSCALL] Linux-compatible syscall interface initialized");
}






pub fn yvu(num: u64, km: u64, oe: u64, vy: u64) -> u64 {
    
    let aux = oae(num, km, oe, vy, 0, 0, 0);

    
    crate::lab_mode::trace_bus::ktb(num, [km, oe, vy], aux);

    
    let ce = crate::process::aei();
    if ce > 0 {
        if let Some(qk) = crate::signals::qzs(ce) {
            
            
            if !crate::process::dsi(ce) && crate::userland::jbp() {
                unsafe { crate::userland::ctw(-(qk as i32)); }
            }
        }
    }

    aux as u64
}


pub fn oae(num: u64, km: u64, oe: u64, vy: u64, bfw: u64, fcf: u64, iik: u64) -> i64 {
    use linux::nr::*;
    
    match num {
        
        Cm => mjb(km as i32, oe, vy as usize),
        Db => mjf(km as i32, oe, vy as usize),
        Bnr => jse(km, oe as u32),
        App => mis(km as i32),
        Bsc | Bkp => linux::icn(km, oe),
        Bgo => linux::pqu(km as i32, oe),
        Bkn => mix(km as i32, oe as i64, vy as u32),
        Cjl => linux::wyq(km as i32, oe, vy as u32),
        Bwk => linux::wzn(km as i32, oe, vy as u32),
        Aos => linux::mio(km, oe as u32),
        Bpy => linux::wyp(km, oe, vy),
        Bjf => linux::miw(km as i32, oe, vy),
        Ccv => linux::wxs(km as i32, oe as u32, vy),
        Asr => miu(km, oe as usize),
        Bcy => miq(km),
        Blu => miy(km),
        Bva => mjd(km),
        Bed => linux::mit(km as i32),
        Bee => linux::jsb(km as i32, oe as i32),
        Cas => linux::jsb(km as i32, oe as i32),
        Bol => linux::pqx(km, oe as u32, vy as i32),
        Cdw => linux::wxv(km as i32, oe, vy as u32),
        Bns => linux::mja(km as i32, oe, vy as u32),
        Bnb => linux::wyk(km as i32, oe, vy, bfw as u32),
        
        
        Avd => linux::jsc(km, oe, vy, bfw, fcf as i64, iik),
        Avf => linux::jsd(km, oe),
        Blw => linux::wyj(km, oe, vy),
        Apl => linux::jsa(km),
        
        
        Asv => linux::wya(),
        Cea => linux::wyb(),
        Ceg => linux::pqw(),
        Asw | Ast => linux::wyg(),
        Asu | Ass => linux::wxw(),
        Clq => linux::wzf(km as u32),
        Clj => linux::wyz(km as u32),
        Clm => linux::wzc(km as u32, oe as u32),
        Cll => linux::wzb(km as u32, oe as u32),
        Clk => linux::wza(km as u32, oe as u32),
        Cdz => linux::wxz(),
        Clo => linux::wzd(),
        Cdy => linux::wxy(km as u32),
        Ced => linux::wyc(km as u32),
        Byy => linux::wxh(km),
        Coq => linux::wzl(km as u32),
        Byw => linux::wxg(km, oe as u32),
        Cct => linux::wxq(km as i32, oe as u32),
        Byx => linux::pqq(km, oe as u32, vy as u32),
        Ccu => linux::wxr(km as i32, oe as u32, vy as u32),
        Cgl => linux::pqq(km, oe as u32, vy as u32),
        Bgm => pqt(),
        Coz => pqt(),
        Byz => wxi(km, oe, vy),
        Bfp => wxo(km, oe, vy),
        Ahp => { 
            crate::process::cxn(km as i32);
            if crate::userland::jbp() { 
                
                
                if crate::userland::LN_.load(core::sync::atomic::Ordering::SeqCst) != 0 {
                    crate::userland::mop(km as i32);
                } else {
                    unsafe { crate::userland::ctw(km as i32); }
                }
            }
            0
        }
        ABX_ => { 
            crate::process::cxn(km as i32);
            if crate::userland::jbp() { 
                if crate::userland::LN_.load(core::sync::atomic::Ordering::SeqCst) != 0 {
                    crate::userland::mop(km as i32);
                } else {
                    unsafe { crate::userland::ctw(km as i32); }
                }
            }
            0
        }
        Cpz => wzm(km as i32, oe, vy as u32),
        Cgg => wyh(km as i32, oe as i32),
        AHU_ => linux::wyy(km),
        
        
        ZD_ => linux::mip(km, oe),
        
        
        Bae => linux::mjc(km),
        
        
        BMM_ => linux::mir(km as u32, oe),
        Ceh => linux::wyf(km, oe),
        Chr => linux::miz(km, oe),
        
        
        CPQ_ => linux::wys(km as u32, oe, vy, bfw),
        CPR_ => linux::wyt(km as u32, oe, vy, bfw),
        AHJ_ => 0,
        Cls => 0,
        
        
        CQM_ => linux::wyv(),
        CQK_ => linux::wyu(km as u32, oe, vy),
        CQL_ => 0,
        
        
        Cdg => wxt(km, oe as u32, vy as u32),
        CSM_ => linux::wyx(km, oe),
        BWZ_ => linux::wxu(km as u32, oe, vy),
        
        
        Cec => linux::pqv(km as u32, oe),
        Cln => 0,
        Ciu => linux::wyo(km as u32, oe as u32, vy, bfw),
        
        
        Ceb => linux::miv(km, oe, vy),
        
        
        Cmc => wzh(km as u16, oe as u32, vy as u32),
        Bzb => wxj(km as i32, oe, vy as usize),
        Bxe => wxe(km as i32, oe, vy),
        Clg => pqz(km as i32, oe, vy as usize, bfw as u32, fcf, iik as usize),
        Cjm => pqy(km as i32, oe, vy as usize, bfw as u32, fcf, iik),
        Uf => wzg(km as i32, oe as i32),
        Bch => wxf(km as i32, oe, vy as usize),
        Cgn => wyi(km as i32, oe as u32),
        Cee => wyd(km as i32, oe, vy),
        Cdx => wxx(km as i32, oe, vy),
        Clp => wze(km as i32, oe as i32, vy as i32, bfw, fcf as usize),
        Cef => wye(km as i32, oe as i32, vy as i32, bfw, fcf),
        Clf => wyw(km as i32, oe, vy as u32),
        Cjn => wyr(km as i32, oe, vy as u32),
        
        
        Cip => wym(km, oe as u32),
        
        
        BTZ_ => linux::wxl(km as i32),
        BUG_ => linux::pqs(km as i32, oe, vy as i32, bfw as i32),
        BUB_ => linux::wxm(km as i32, oe as i32, vy as i32, bfw),
        BUF_ => linux::wxn(km as i32, oe, vy as i32, bfw as i32, fcf, iik),
        BUA_ => linux::pqr(km as u32),
        
        
        Cit => linux::wyn(km as u32, oe, vy, bfw, fcf),
        Cml => linux::wzj(km),
        Cmk => linux::wzi(km),
        
        
        nr::CVA_ => wxk(km, oe as usize),
        nr::CWY_ => { crate::ipc::whk(km); 0 },
        nr::CWX_ => crate::ipc::vtc(km) as i64,
        nr::CWW_ => crate::ipc::rqk() as i64,
        
        _ => {
            crate::log_debug!("[SYSCALL] Unknown: {} (0x{:x})", num, num);
            errno::Pg
        }
    }
}

fn mjb(da: i32, aeg: u64, az: usize) -> i64 {
    if aeg == 0 || az == 0 {
        return errno::Er;
    }
    
    
    if !sw(aeg, az, true) {
        crate::log_warn!("[SYSCALL] read: invalid user pointer {:#x}", aeg);
        return errno::X;
    }
    
    
    if crate::pipe::gkh(da) {
        let bi = unsafe { core::slice::bef(aeg as *mut u8, az) };
        return crate::pipe::read(da, bi);
    }
    
    let bi = unsafe { core::slice::bef(aeg as *mut u8, az) };
    match crate::vfs::read(da, bi) {
        Ok(bo) => bo as i64,
        Err(_) => errno::Abi,
    }
}

fn mjf(da: i32, aeg: u64, az: usize) -> i64 {
    if aeg == 0 || az == 0 {
        return errno::Er;
    }
    
    
    if !sw(aeg, az, false) {
        crate::log_warn!("[SYSCALL] write: invalid user pointer {:#x}", aeg);
        return errno::X;
    }
    
    let bi = unsafe { core::slice::anh(aeg as *const u8, az) };
    
    
    if da == 1 || da == 2 {
        for &o in bi { crate::serial_print!("{}", o as char); }
        return az as i64;
    }
    
    
    if crate::pipe::gkh(da) {
        return crate::pipe::write(da, bi);
    }
    
    match crate::vfs::write(da, bi) {
        Ok(bo) => bo as i64,
        Err(_) => errno::Abi,
    }
}

fn jse(arq: u64, flags: u32) -> i64 {
    let path = match gqk(arq, 256) {
        Some(e) => e,
        None => return errno::X,
    };
    match crate::vfs::aji(&path, crate::vfs::OpenFlags(flags)) {
        Ok(da) => da as i64,
        Err(_) => errno::Il,
    }
}

fn mis(da: i32) -> i64 {
    
    if linux::txk(da) {
        crate::syscall::linux::JX_.lock().remove(&da);
        return 0;
    }
    
    if crate::pipe::gkh(da) {
        return crate::pipe::agj(da);
    }
    match crate::vfs::agj(da) {
        Ok(()) => 0,
        Err(_) => errno::Fu,
    }
}


fn wym(ovv: u64, ddp: u32) -> i64 {
    if !sw(ovv, 8, true) {
        return errno::X;
    }
    let (cbh, civ) = crate::pipe::avp();
    unsafe {
        let ptr = ovv as *mut i32;
        *ptr = cbh;
        *ptr.add(1) = civ;
    }
    crate::log_debug!("[SYSCALL] pipe2: read_fd={}, write_fd={}", cbh, civ);
    0
}

fn miy(arq: u64) -> i64 {
    let path = match gqk(arq, 256) {
        Some(e) => e,
        None => return errno::X,
    };
    match crate::vfs::ut(&path) {
        Ok(()) => 0,
        Err(_) => errno::Abi,
    }
}

fn mjd(arq: u64) -> i64 {
    let path = match gqk(arq, 256) {
        Some(e) => e,
        None => return errno::X,
    };
    match crate::vfs::cnm(&path) {
        Ok(()) => 0,
        Err(_) => errno::Il,
    }
}

fn wxk(aeg: u64, len: usize) -> i64 {
    if aeg == 0 { return errno::X; }
    
    
    if !sw(aeg, len, false) {
        return errno::X;
    }
    
    let bi = unsafe { core::slice::anh(aeg as *const u8, len) };
    for &o in bi { crate::serial_print!("{}", o as char); }
    len as i64
}


fn gqk(ptr: u64, am: usize) -> Option<alloc::string::String> {
    if ptr == 0 { return None; }
    
    
    if !aov(ptr) {
        crate::log_warn!("[SYSCALL] read_cstring: kernel address {:#x}", ptr);
        return None;
    }
    
    let mut e = alloc::string::String::new();
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









fn wxi(flags: u64, jo: u64, bt: u64) -> i64 {
    
    
    
    if jo == 0 || bt == 0 {
        return errno::Er;
    }
    
    
    if !aov(jo) || !aov(bt) {
        return errno::X;
    }
    
    let ce = crate::process::aei();
    let ni = crate::thread::pme(ce, "user_thread", bt, jo, 0);
    
    ni as i64
}






fn wxt(ag: u64, op: u32, ap: u32) -> i64 {
    const ACN_: u32 = 0;
    const ACO_: u32 = 1;
    const ACM_: u32 = 128;
    
    let op = op & !ACM_;
    
    if !aov(ag) {
        return errno::X;
    }
    
    match op {
        ACN_ => {
            
            let cv = unsafe { *(ag as *const u32) };
            if cv != ap {
                return errno::Bfc;
            }
            
            
            let ydl = crate::thread::bqd();
            
            
            crate::thread::cix();
            0
        }
        ACO_ => {
            
            
            
            0
        }
        _ => errno::Pg,
    }
}





fn mix(da: i32, l: i64, gwp: u32) -> i64 {
    match crate::vfs::uis(da, l, gwp) {
        Ok(u) => u as i64,
        Err(_) => errno::Er,
    }
}

fn miu(k: u64, aw: usize) -> i64 {
    if !sw(k, aw, true) {
        return errno::X;
    }
    
    let jv = crate::vfs::iwx();
    let bf = jv.as_bytes();
    let len = bf.len().v(aw - 1);
    
    unsafe {
        let cs = core::slice::bef(k as *mut u8, aw);
        cs[..len].dg(&bf[..len]);
        cs[len] = 0;
    }
    
    k as i64
}

fn miq(arq: u64) -> i64 {
    let path = match gqk(arq, 256) {
        Some(e) => e,
        None => return errno::X,
    };
    
    match crate::vfs::qyj(&path) {
        Ok(()) => 0,
        Err(_) => errno::Il,
    }
}

fn pqt() -> i64 {
    match crate::process::svr() {
        Ok(ce) => ce as i64,
        Err(_) => errno::Abl,
    }
}

fn wxo(clu: u64, fzk: u64, qbt: u64) -> i64 {
    let path = match gqk(clu, 256) {
        Some(e) => e,
        None => return errno::X,
    };
    
    
    let mut emg: alloc::vec::Vec<alloc::string::String> = alloc::vec::Vec::new();
    if fzk != 0 && aov(fzk) {
        for a in 0..256u64 {
            let oyi = fzk + a * 8;
            if !aov(oyi) { break; }
            let ptr = unsafe { *(oyi as *const u64) };
            if ptr == 0 { break; }
            if let Some(e) = gqk(ptr, 256) {
                emg.push(e);
            } else {
                break;
            }
        }
    }
    if emg.is_empty() {
        emg.push(path.clone());
    }
    let qkl: alloc::vec::Vec<&str> = emg.iter().map(|e| e.as_str()).collect();
    
    match crate::exec::sow(&path, &qkl, &[]) {
        Ok(()) => 0, 
        Err(_) => errno::Il,
    }
}

fn wzm(ce: i32, mri: u64, options: u32) -> i64 {
    let ejo = if ce > 0 { ce as u32 } else { 0 };
    let pzp = options & 1 != 0; 
    
    
    let umb: u32 = if pzp { 1 } else { 5000 };
    
    for _ in 0..umb {
        match crate::process::ccm(ejo) {
            Ok(status) => {
                if mri != 0 && sw(mri, 4, true) {
                    
                    unsafe { *(mri as *mut i32) = (status & 0xFF) << 8; }
                }
                return ejo as i64;
            }
            Err(_) => {
                if pzp { return 0; }
                crate::thread::cix();
            }
        }
    }
    errno::Cbb
}

fn wyh(ce: i32, sig: i32) -> i64 {
    if ce <= 0 {
        return errno::Er;
    }
    
    let gsa = crate::process::aei();
    
    match crate::signals::dsm(ce as u32, sig as u32, gsa) {
        Ok(()) => 0,
        Err(aa) => aa as i64,
    }
}






fn wzh(vh: u16, bif: u32, protocol: u32) -> i64 {
    match crate::netstack::socket::socket(vh, bif, protocol) {
        Ok(da) => da as i64,
        Err(aa) => aa as i64,
    }
}


fn wxj(da: i32, azf: u64, ely: usize) -> i64 {
    use crate::netstack::socket::SockAddrIn;
    
    if ely < SockAddrIn::Am {
        return errno::Er;
    }
    
    if !sw(azf, ely, false) {
        return errno::X;
    }
    
    let ag = unsafe { *(azf as *const SockAddrIn) };
    
    match crate::netstack::socket::ipa(da, &ag) {
        Ok(()) => 0,
        Err(aa) => aa as i64,
    }
}


fn wxf(da: i32, azf: u64, ely: usize) -> i64 {
    use crate::netstack::socket::SockAddrIn;
    
    if ely < SockAddrIn::Am {
        return errno::Er;
    }
    
    if !sw(azf, ely, false) {
        return errno::X;
    }
    
    let ag = unsafe { *(azf as *const SockAddrIn) };
    
    match crate::netstack::socket::kdj(da, &ag) {
        Ok(()) => 0,
        Err(aa) => aa as i64,
    }
}


fn wyi(da: i32, dea: u32) -> i64 {
    match crate::netstack::socket::ojr(da, dea) {
        Ok(()) => 0,
        Err(aa) => aa as i64,
    }
}


fn wxe(da: i32, azf: u64, bye: u64) -> i64 {
    match crate::netstack::socket::qes(da, azf, bye) {
        Ok(anp) => anp as i64,
        Err(aa) => aa as i64,
    }
}


fn pqz(da: i32, aeg: u64, len: usize, flags: u32, azf: u64, ely: usize) -> i64 {
    use crate::netstack::socket::SockAddrIn;
    
    if !sw(aeg, len, false) {
        return errno::X;
    }
    
    let f = unsafe { core::slice::anh(aeg as *const u8, len) };
    
    if azf != 0 && ely >= SockAddrIn::Am {
        
        if !sw(azf, ely, false) {
            return errno::X;
        }
        let ag = unsafe { *(azf as *const SockAddrIn) };
        match crate::netstack::socket::whr(da, f, flags, &ag) {
            Ok(bo) => bo as i64,
            Err(aa) => aa as i64,
        }
    } else {
        
        match crate::netstack::socket::baq(da, f, flags) {
            Ok(bo) => bo as i64,
            Err(aa) => aa as i64,
        }
    }
}


fn pqy(da: i32, aeg: u64, len: usize, flags: u32, azf: u64, bye: u64) -> i64 {
    if !sw(aeg, len, true) {
        return errno::X;
    }
    
    let k = unsafe { core::slice::bef(aeg as *mut u8, len) };
    
    
    match crate::netstack::socket::ehf(da, k, flags) {
        Ok(bo) => bo as i64,
        Err(aa) => aa as i64,
    }
}


fn wzg(da: i32, xzv: i32) -> i64 {
    match crate::netstack::socket::agj(da) {
        Ok(()) => 0,
        Err(aa) => aa as i64,
    }
}


fn wyd(da: i32, azf: u64, bye: u64) -> i64 {
    use crate::netstack::socket::{SockAddrIn, BA_};

    if azf == 0 || bye == 0 {
        return errno::X;
    }
    if !sw(bye, 4, true) {
        return errno::X;
    }

    let len = unsafe { *(bye as *const u32) } as usize;
    if len < SockAddrIn::Am || !sw(azf, SockAddrIn::Am, true) {
        return errno::Er;
    }

    let gg = BA_.lock();
    let su = match gg.get(&da) {
        Some(e) => e,
        None => return errno::Fu,
    };

    let ag = su.ljn.age();
    unsafe {
        *(azf as *mut SockAddrIn) = ag;
        *(bye as *mut u32) = SockAddrIn::Am as u32;
    }
    0
}


fn wxx(da: i32, azf: u64, bye: u64) -> i64 {
    use crate::netstack::socket::{SockAddrIn, BA_};

    if azf == 0 || bye == 0 {
        return errno::X;
    }
    if !sw(bye, 4, true) {
        return errno::X;
    }

    let len = unsafe { *(bye as *const u32) } as usize;
    if len < SockAddrIn::Am || !sw(azf, SockAddrIn::Am, true) {
        return errno::Er;
    }

    let gg = BA_.lock();
    let su = match gg.get(&da) {
        Some(e) => e,
        None => return errno::Fu,
    };

    let ag = match su.exp {
        Some(q) => q,
        None => return -107, 
    };
    unsafe {
        *(azf as *mut SockAddrIn) = ag;
        *(bye as *mut u32) = SockAddrIn::Am as u32;
    }
    0
}


fn wyw(da: i32, cax: u64, flags: u32) -> i64 {
    if cax == 0 || !sw(cax, 56, false) {
        return errno::X;
    }
    
    let lnk = unsafe { *(cax as *const u64) };
    let baf = unsafe { *((cax + 8) as *const u32) } as usize;
    let edr  = unsafe { *((cax + 16) as *const u64) };
    let cyy  = unsafe { *((cax + 24) as *const u64) } as usize;

    if cyy == 0 || edr == 0 {
        return 0;
    }
    
    if !sw(edr, 16, false) {
        return errno::X;
    }
    let ar = unsafe { *(edr as *const u64) };
    let len  = unsafe { *((edr + 8) as *const u64) } as usize;

    pqz(da, ar, len, flags, lnk, baf)
}


fn wyr(da: i32, cax: u64, flags: u32) -> i64 {
    if cax == 0 || !sw(cax, 56, true) {
        return errno::X;
    }
    let lnk = unsafe { *(cax as *const u64) };
    let urf = unsafe { (cax + 8) as u64 };
    let edr  = unsafe { *((cax + 16) as *const u64) };
    let cyy  = unsafe { *((cax + 24) as *const u64) } as usize;

    if cyy == 0 || edr == 0 {
        return 0;
    }
    if !sw(edr, 16, false) {
        return errno::X;
    }
    let ar = unsafe { *(edr as *const u64) };
    let len  = unsafe { *((edr + 8) as *const u64) } as usize;

    pqy(da, ar, len, flags, lnk, urf)
}


fn wze(da: i32, jy: i32, evr: i32, ctc: u64, evq: usize) -> i64 {
    if evq > 0 && !sw(ctc, evq, false) {
        return errno::X;
    }
    
    let f = if evq > 0 {
        unsafe { core::slice::anh(ctc as *const u8, evq) }
    } else {
        &[]
    };
    
    match crate::netstack::socket::wkd(da, jy, evr, f) {
        Ok(()) => 0,
        Err(aa) => aa as i64,
    }
}


fn wye(da: i32, jy: i32, evr: i32, ctc: u64, jhy: u64) -> i64 {
    if ctc == 0 || jhy == 0 {
        return errno::X;
    }
    
    if !sw(jhy, 4, true) {
        return errno::X;
    }
    
    let evq = unsafe { *(jhy as *const u32) } as usize;
    
    if evq > 0 && !sw(ctc, evq, true) {
        return errno::X;
    }
    
    let k = unsafe { core::slice::bef(ctc as *mut u8, evq) };
    
    match crate::netstack::socket::tfj(da, jy, evr, k) {
        Ok(len) => {
            unsafe { *(jhy as *mut u32) = len as u32; }
            0
        }
        Err(aa) => aa as i64,
    }
}
