




use alloc::string::String;
use alloc::vec::Vec;
use alloc::vec;
use alloc::boxed::Box;
use alloc::format;
use core::sync::atomic::{AtomicBool, Ordering};
use spin::Mutex as SpinMutex;
use crate::framebuffer::{B_, G_, AU_, D_, A_, C_, Q_, CD_, DF_, L_};
use crate::ramfs::FileType;


static Atv: AtomicBool = AtomicBool::new(false);


pub fn etf() -> bool {
    Atv.load(Ordering::SeqCst)
}


pub fn hcw() {
    Atv.store(false, Ordering::SeqCst);
}


pub fn jpb() {
    Atv.store(true, Ordering::SeqCst);
}


pub(crate) static DE_: AtomicBool = AtomicBool::new(false);

static HO_: SpinMutex<String> = SpinMutex::new(String::new());


pub fn qwj(e: &str) {
    if DE_.load(core::sync::atomic::Ordering::Relaxed) {
        let mut k = HO_.lock();
        k.t(e);
    }
}


pub fn edu() -> bool {
    DE_.load(core::sync::atomic::Ordering::Relaxed)
}



pub fn jsk() -> String {
    let mut k = HO_.lock();
    let e = k.clone();
    k.clear();
    e
}



#[inline]
fn mfq() {
    let (bj, br) = crate::framebuffer::gia();
    crate::framebuffer::ah(
        (bj * 8) as u32, (br * 16) as u32, 8, 16, B_,
    );
}






#[repr(C)]
pub struct Tk {
    pub aeg: *mut u32,
    pub bjl: usize,
    pub z: u32,
    pub ac: u32,
    pub car: *const u8,              
    pub awc: *const i32,              
    pub hmz: *const u8,             
    pub gme: usize,                    
}

unsafe impl Send for Tk {}
unsafe impl Sync for Tk {}



pub(super) fn pbw(ay: usize, ci: usize, f: *mut u8) {
    let oi = unsafe { &*(f as *const Tk) };
    let aeg = oi.aeg;
    let bjl = oi.bjl;
    let z = oi.z;
    let ac = oi.ac;
    let obz = oi.hmz;
    let tmp = !obz.abq();
    let gme = oi.gme;
    
    let byt = 8u32;
    
    for bj in ay..ci {
        if bj >= 240 { break; }
        
        let b = bj as u32 * byt;
        if b >= z { continue; }
        
        let ale = unsafe { *oi.awc.add(bj) };
        let dwe = (ale - 5).am(0) as usize;
        let ktn = if ale + 30 < 0 { 0 } else { ((ale + 30) as usize).v(gme) };
        
        for br in dwe..ktn {
            let c = br as u32 * 16;
            if c >= ac { continue; }
            
            let la = br as i32 - ale;
            
            
            let gzs: u32 = if la < 0 {
                continue;
            } else if la == 0 {
                255  
            } else if la <= 12 {
                255 - (la as u32 * 8)
            } else if la <= 28 {
                
                let pv = ((la - 12) as u32).v(15) * 16;
                let yx = 255u32.ao(pv);
                (160 * yx) / 255
            } else {
                continue;
            };
            
            
            let kwn = bj * gme + br;
            
            
            
            let (sss, s) = if tmp {
                let tpq = unsafe { *obz.add(kwn) };
                if tpq >= 1 {
                    
                    ('#', 0xFF00FF00)
                } else {
                    
                    let r = unsafe { *oi.car.add(kwn) as char };
                    (r, 0xFF000000 | (gzs << 8))
                }
            } else {
                
                let r = unsafe { *oi.car.add(kwn) as char };
                (r, 0xFF000000 | (gzs << 8))
            };
            
            let ka = crate::framebuffer::font::ada(sss);
            
            
            unsafe {
                for (m, &fs) in ka.iter().cf() {
                    let x = c + m as u32;
                    if x >= ac { break; }
                    let afg = (x * z) as usize;
                    
                    if fs != 0 {
                        let bxy = b as usize;
                        if fs & 0x80 != 0 { let w = afg + bxy; if w < bjl { *aeg.add(w) = s; } }
                        if fs & 0x40 != 0 { let w = afg + bxy + 1; if w < bjl { *aeg.add(w) = s; } }
                        if fs & 0x20 != 0 { let w = afg + bxy + 2; if w < bjl { *aeg.add(w) = s; } }
                        if fs & 0x10 != 0 { let w = afg + bxy + 3; if w < bjl { *aeg.add(w) = s; } }
                        if fs & 0x08 != 0 { let w = afg + bxy + 4; if w < bjl { *aeg.add(w) = s; } }
                        if fs & 0x04 != 0 { let w = afg + bxy + 5; if w < bjl { *aeg.add(w) = s; } }
                        if fs & 0x02 != 0 { let w = afg + bxy + 6; if w < bjl { *aeg.add(w) = s; } }
                        if fs & 0x01 != 0 { let w = afg + bxy + 7; if w < bjl { *aeg.add(w) = s; } }
                    }
                }
            }
        }
    }
}


pub const AHV_: &[&str] = &[
    
    "help", "man", "info", "version", "uname",
    
    "ls", "dir", "cd", "pwd", "mkdir", "rmdir", "touch", "rm", "del", "cp", "copy",
    "mv", "move", "rename", "cat", "type", "head", "tail", "wc", "stat", "tree", "find",
    
    "echo", "grep",
    
    "clear", "cls", "time", "uptime", "date", "whoami", "hostname", "id", "env", "printenv",
    "history", "ps", "free", "df",
    
    "login", "su", "passwd", "adduser", "useradd", "deluser", "userdel", "users", "logout",
    
    "hwtest", "keytest", "hexdump", "xxd", "panic",
    
    "hwdiag", "cpudump", "stacktrace", "backtrace", "bootlog", "postcode",
    "ioport", "rdmsr", "wrmsr", "cpuid", "memmap", "watchdog",
    
    "desktop", "gui", "mobile", "cosmic", "open", "trustedit",
    
    "signature",
    
    "security",
    
    "linux", "distro", "alpine",
    
    "tasks", "jobs", "threads",
    
    "disk", "dd", "ahci", "fdisk", "partitions",
    
    "lspci", "lshw", "hwinfo", "gpu", "gpuexec", "sdma", "neural", "gpufw", "a11y",
    
    "lsusb", "checkm8",
    
    "beep", "audio", "synth", "play", "vizfx",
    
    "fan", "temp", "sensors", "cpufreq", "speedstep",
    
    "ifconfig", "ip", "ipconfig", "ping", "curl", "wget", "download",
    "nslookup", "dig", "arp", "route", "netstat",
    
    "which", "file", "chmod", "ln", "sort", "uniq", "cut",
    "kill", "top", "dmesg", "strings", "tar",
    "mount", "umount", "sync", "lsblk",
    
    "exit", "reboot", "shutdown", "poweroff",
    
    "exec", "run",
    
    "trustview", "tv",
    
    "lab", "trustlab",
    
    "hwscan", "trustprobe", "probe",
    
    "neofetch", "matrix", "cowsay", "rain",
    
    "showcase",
    "showcase3d",
    "filled3d",
    
    "demo", "tutorial", "tour",
    
    "hv", "hypervisor",
    
    "trailer", "trustos_trailer",
    
    "nano", "edit", "vi",
    
    "alias", "unalias", "bc", "diff", "md5sum", "sha256sum", "base64",
    "cut", "tr", "tee", "xargs", "chmod", "chown", "ln", "readlink",
    "watch", "timeout", "tar", "gzip", "zip", "unzip",
    "service", "systemctl", "crontab", "at", "read",
];


pub fn vw() -> ! {
    
    crate::framebuffer::meo();
    crate::framebuffer::clear();

    
    scripting::init();

    

    
    crate::interrupts::mee(true);

    vlf();

    
    unix::wbm();
    
    let mut hdd = [0u8; 512];
    
    loop {
        oxv();
        
        
        let len = vry(&mut hdd);
        
        
        let rio = core::str::jg(&hdd[..len]).unwrap_or("");
        hcw(); 
        azu(rio.em());
    }
}


fn vry(bi: &mut [u8]) -> usize {
    use crate::keyboard::{auw, jzh, lcd, lcc, lce,
                          V_, U_, AH_, AI_, CQ_, CP_, CX_,
                          AM_, AQ_};
    
    let mut u: usize = 0;
    let mut gi: usize = 0;
    let mut aer: Vec<&str> = Vec::new();
    let mut bcv: i32 = -1; 
    let mut ble = false;
    
    
    
    let (ycq, wen) = crate::framebuffer::yn();
    let brh = (wen as usize) / 16; 
    let mut abf = crate::framebuffer::gia().1;
    let bkb = crate::framebuffer::gia().0;
    const AIK_: usize = 4; 
    if brh > AIK_ && abf + AIK_ >= brh {
        let ojh = abf + AIK_ - brh + 1;
        for _ in 0..ojh {
            crate::framebuffer::dlm();
        }
        abf = abf.ao(ojh);
        crate::framebuffer::bld(bkb, abf);
    }
    
    
    let mut cwu = true;
    let mut byk: u32 = 0;
    const BLF_: u32 = 500000;
    
    lce();
    
    
    mfq();
    
    loop {
        if let Some(r) = auw() {
            
            if r != AM_ && r != AQ_ && crate::framebuffer::lgl() {
                let (qbo, br) = crate::framebuffer::pcw();
                abf = br;
                
                
                crate::framebuffer::bld(bkb + gi, abf);
            }
            
            
            let ift = if gi < u { bi[gi] as char } else { ' ' };
            crate::cgs!("{}", ift);
            crate::cgs!("\x08");
            cwu = true;
            byk = 0;
            
            match r {
                b'\n' | b'\r' => {
                    
                    crate::framebuffer::weu();
                    
                    
                    if bcv >= 0 && (bcv as usize) < aer.len() {
                        let na = aer[bcv as usize];
                        dev(abf, aer.len());
                        khx(bkb, abf, u);
                        let bf = na.as_bytes();
                        let len = bf.len().v(bi.len() - 1);
                        bi[..len].dg(&bf[..len]);
                        u = len;
                        gi = len;
                        for a in 0..u {
                            crate::print!("{}", bi[a] as char);
                        }
                    } else if ble {
                        dev(abf, aer.len());
                    }
                    crate::println!();
                    let cmd = core::str::jg(&bi[..u]).unwrap_or("");
                    if !cmd.em().is_empty() {
                        jzh(cmd);
                    }
                    break;
                }
                b'\t' => {
                    
                    if !aer.is_empty() {
                        let w = if bcv >= 0 { bcv as usize } else { 0 };
                        let na = aer[w];
                        dev(abf, aer.len());
                        khx(bkb, abf, u);
                        let bf = na.as_bytes();
                        let len = bf.len().v(bi.len() - 1);
                        bi[..len].dg(&bf[..len]);
                        u = len;
                        gi = len;
                        for a in 0..u {
                            crate::print!("{}", bi[a] as char);
                        }
                        bcv = -1;
                        ble = false;
                        aer.clear();
                        if u < bi.len() - 1 {
                            bi[u] = b' ';
                            u += 1;
                            gi += 1;
                            crate::print!(" ");
                        }
                    }
                }
                0x1B => {
                    
                    if ble {
                        dev(abf, aer.len());
                        aer.clear();
                        bcv = -1;
                        ble = false;
                    }
                }
                0x08 => {
                    
                    if gi > 0 {
                        
                        if ble {
                            dev(abf, aer.len());
                            crate::framebuffer::bld(bkb + gi, abf);
                        }
                        for a in gi..u {
                            bi[a - 1] = bi[a];
                        }
                        u = u.ao(1);
                        gi = gi.ao(1);
                        crate::cgs!("\x08");
                        for a in gi..u {
                            crate::print!("{}", bi[a] as char);
                        }
                        crate::cgs!(" ");
                        for _ in gi..=u {
                            crate::cgs!("\x08");
                        }
                        
                        juu(bi, u, &mut aer);
                        if !aer.is_empty() && u > 0 {
                            gsn(abf, &aer, bcv);
                            ble = true;
                            crate::framebuffer::bld(bkb + gi, abf);
                        } else {
                            ble = false;
                            bcv = -1;
                        }
                    }
                }
                V_ => {
                    if u == 0 {
                        
                        if let Some(vo) = lcd() {
                            let bf = vo.as_bytes();
                            let len = bf.len().v(bi.len() - 1);
                            bi[..len].dg(&bf[..len]);
                            u = len;
                            gi = len;
                            crate::print!("{}", &vo[..len]);
                        }
                    } else {
                        
                        if !ble {
                            
                            juu(bi, u, &mut aer);
                            if !aer.is_empty() {
                                bcv = 0;
                                gsn(abf, &aer, bcv);
                                ble = true;
                                
                                crate::framebuffer::bld(bkb + gi, abf);
                            }
                        } else if !aer.is_empty() {
                            
                            dev(abf, aer.len());
                            if bcv <= 0 {
                                bcv = aer.len() as i32 - 1;
                            } else {
                                bcv -= 1;
                            }
                            gsn(abf, &aer, bcv);
                            crate::framebuffer::bld(bkb + gi, abf);
                        }
                    }
                }
                U_ => {
                    if u == 0 {
                        
                        if let Some(next) = lcc() {
                            let bf = next.as_bytes();
                            let len = bf.len().v(bi.len() - 1);
                            bi[..len].dg(&bf[..len]);
                            u = len;
                            gi = len;
                            crate::print!("{}", &next[..len]);
                        } else {
                            khx(bkb, abf, u);
                            u = 0;
                            gi = 0;
                        }
                    } else if ble && !aer.is_empty() {
                        
                        dev(abf, aer.len());
                        bcv += 1;
                        if bcv >= aer.len() as i32 {
                            bcv = 0;
                        }
                        gsn(abf, &aer, bcv);
                        crate::framebuffer::bld(bkb + gi, abf);
                    }
                }
                AH_ => {
                    if gi > 0 {
                        gi -= 1;
                        crate::cgs!("\x08");
                    }
                }
                AI_ => {
                    if gi < u {
                        crate::print!("{}", bi[gi] as char);
                        gi += 1;
                    }
                }
                CQ_ => {
                    while gi > 0 {
                        crate::cgs!("\x08");
                        gi -= 1;
                    }
                }
                CP_ => {
                    while gi < u {
                        crate::print!("{}", bi[gi] as char);
                        gi += 1;
                    }
                }
                CX_ => {
                    if gi < u {
                        if ble {
                            dev(abf, aer.len());
                            crate::framebuffer::bld(bkb + gi, abf);
                        }
                        for a in gi..u.ao(1) {
                            bi[a] = bi[a + 1];
                        }
                        u = u.ao(1);
                        for a in gi..u {
                            crate::print!("{}", bi[a] as char);
                        }
                        crate::cgs!(" ");
                        for _ in gi..=u {
                            crate::cgs!("\x08");
                        }
                        juu(bi, u, &mut aer);
                        if !aer.is_empty() && u > 0 {
                            gsn(abf, &aer, bcv);
                            ble = true;
                            crate::framebuffer::bld(bkb + gi, abf);
                        } else {
                            ble = false;
                            bcv = -1;
                        }
                    }
                }
                AM_ => {
                    crate::framebuffer::wev(10);
                }
                AQ_ => {
                    crate::framebuffer::eid(10);
                    
                    
                    if !crate::framebuffer::lgl() {
                        let (_, br) = crate::framebuffer::gia();
                        abf = br;
                        crate::framebuffer::bld(bkb + gi, abf);
                    }
                }
                27 => {
                    
                    if crate::framebuffer::lgl() {
                        let (qbo, br) = crate::framebuffer::pcw();
                        abf = br;
                        
                        crate::framebuffer::bld(bkb + gi, abf);
                    }
                }
                3 => {
                    
                    if ble {
                        dev(abf, aer.len());
                        ble = false;
                    }
                    crate::gr!(A_, "^C");
                    crate::println!();
                    jpb();
                    u = 0;
                    break;
                }
                12 => {
                    
                    if ble {
                        dev(abf, aer.len());
                        ble = false;
                    }
                    crate::framebuffer::clear();
                    oxv();
                    for a in 0..u {
                        crate::print!("{}", bi[a] as char);
                    }
                    for _ in gi..u {
                        crate::cgs!("\x08");
                    }
                }
                r if r >= 0x20 && r < 0x7F && u < bi.len() - 1 => {
                    
                    
                    if ble {
                        dev(abf, aer.len());
                        crate::framebuffer::bld(bkb + gi, abf);
                    }
                    
                    if gi < u {
                        for a in (gi..u).vv() {
                            bi[a + 1] = bi[a];
                        }
                    }
                    bi[gi] = r;
                    u += 1;
                    gi += 1;
                    for a in gi - 1..u {
                        crate::print!("{}", bi[a] as char);
                    }
                    for _ in gi..u {
                        crate::cgs!("\x08");
                    }
                    
                    
                    juu(bi, u, &mut aer);
                    if !aer.is_empty() {
                        gsn(abf, &aer, bcv);
                        ble = true;
                        
                        crate::framebuffer::bld(bkb + gi, abf);
                    } else {
                        ble = false;
                        bcv = -1;
                    }
                }
                _ => {}
            }
            
            
            mfq();
        } else {
            
            byk += 1;
            if byk >= BLF_ {
                byk = 0;
                cwu = !cwu;
                
                if cwu {
                    mfq();
                } else {
                    let ift = if gi < u { bi[gi] as char } else { ' ' };
                    crate::cgs!("{}", ift);
                    crate::cgs!("\x08");
                }
            }
            
            {
                static CKF_: core::sync::atomic::AtomicU32 = core::sync::atomic::AtomicU32::new(0);
                let az = CKF_.fetch_add(1, core::sync::atomic::Ordering::Relaxed);
                if az % 5000 == 0 {
                    crate::netstack::poll();
                }
                
                if az % 100 == 0 && crate::jarvis::mesh::rl() {
                    crate::jarvis::ona();
                }
                
                if az % 10000 == 0 {
                    crate::jarvis::mentor::vjt();
                }
            }
            for _ in 0..100 { core::hint::hc(); }
        }
    }
    
    
    let ift = if gi < u { bi[gi] as char } else { ' ' };
    crate::cgs!("{}", ift);
    if gi < u { crate::cgs!("\x08"); }
    
    bi[u] = 0;
    u
}


fn juu(bi: &[u8], u: usize, aer: &mut Vec<&'static str>) {
    aer.clear();
    if u == 0 {
        return;
    }
    
    let input = match core::str::jg(&bi[..u]) {
        Ok(e) => e,
        Err(_) => return,
    };
    
    let cqo = input.ayt().next().unwrap_or("");
    if cqo.is_empty() || input.contains(' ') {
        return;
    }
    
    for cmd in AHV_ {
        if cmd.cj(cqo) && *cmd != cqo {
            aer.push(cmd);
            if aer.len() >= 8 {
                break;
            }
        }
    }
}



fn gsn(abf: usize, aer: &[&str], joh: i32) {
    if aer.is_empty() {
        return;
    }
    
    
    let (jym, ac) = crate::framebuffer::yn();
    let efh = (ac as usize) / 16; 
    let lp = crate::framebuffer::hlh();
    let ei = 0xFF000000u32; 
    
    for (a, cmd) in aer.iter().cf() {
        let br = abf + 1 + a;
        if br >= efh { break; } 
        crate::framebuffer::ndd(br);
        if a as i32 == joh {
            let adx = alloc::format!(" > {}", cmd);
            crate::framebuffer::krm(0, br, &adx, lp, ei);
        } else {
            let adx = alloc::format!("   {}", cmd);
            crate::framebuffer::krm(0, br, &adx, lp, ei);
        }
    }
}



fn dev(abf: usize, az: usize) {
    let (jym, ac) = crate::framebuffer::yn();
    let efh = (ac as usize) / 16; 
    for a in 0..az {
        let br = abf + 1 + a;
        if br >= efh { break; }
        crate::framebuffer::ndd(br);
    }
}



fn khx(bkb: usize, abf: usize, u: usize) {
    let (z, _) = crate::framebuffer::yn();
    let ec = (z as usize) / 8; 
    
    let rbe = (u + 2).v(ec.ao(bkb));
    
    let eyr: alloc::string::String = core::iter::afd(' ').take(rbe).collect();
    crate::framebuffer::krm(bkb, abf, &eyr, 0xFF000000, 0xFF000000);
    
    crate::framebuffer::bld(bkb, abf);
}

fn vlf() {
    crate::println!();
    crate::h!(G_, r" _____ ____            _    ___      ");
    crate::h!(G_, r"|_   _|  _ \ _   _ ___| |_ / _ \ ___ ");
    crate::h!(B_,        r"  | | | |_) | | | / __| __| | | / __|");
    crate::h!(B_,        r"  | | |  _ <| |_| \__ \ |_| |_| \__ \");
    crate::h!(AU_,   r"  |_| |_| \_\\__,_|___/\__\\___/|___/");
    crate::println!();
    crate::h!(C_, "  T-RustOs v0.2.0 - Type 'help' for commands");
    crate::println!();
}

fn oxv() {
    
    let os = crate::rtc::cgz();
    let jv = if crate::ramfs::ky() {
        crate::ramfs::fh(|fs| String::from(fs.dau()))
    } else {
        String::from("/")
    };
    
    crate::gr!(AU_, "[{:02}:{:02}:{:02}] ", os.bek, os.bri, os.chr);
    crate::gr!(G_, "trustos");
    crate::gr!(Q_, ":");
    crate::gr!(C_, "{}", jv);
    crate::gr!(B_, "$ ");
}


pub fn cts() -> alloc::string::String {
    let mut k = [0u8; 512];
    let len = crate::keyboard::cts(&mut k);
    core::str::jg(&k[..len]).unwrap_or("").into()
}


pub fn azu(cmd: &str) {
    if cmd.is_empty() {
        return;
    }

    
    let tg = scripting::cxo(cmd);
    let cmd = tg.as_str();
    
    
    let ovt = wrg(cmd);
    if ovt.len() > 1 {
        sor(&ovt);
        return;
    }
    
    
    kul(cmd, None);
}


fn wrg(cmd: &str) -> Vec<&str> {
    let mut jq = Vec::new();
    let mut ay = 0;
    let mut izt = false;
    let mut izr = false;
    let bf = cmd.as_bytes();
    
    for a in 0..bf.len() {
        match bf[a] {
            b'\'' if !izr => izt = !izt,
            b'"' if !izt => izr = !izr,
            b'|' if !izt && !izr => {
                
                if a + 1 < bf.len() && bf[a + 1] == b'|' {
                    continue; 
                }
                jq.push(cmd[ay..a].em());
                ay = a + 1;
            }
            _ => {}
        }
    }
    jq.push(cmd[ay..].em());
    jq
}


fn sor(jq: &[&str]) {
    let mut bvz: Option<String> = None;
    
    for (a, ie) in jq.iter().cf() {
        let fmd = a == jq.len() - 1;
        let input = bvz.take();
        
        if fmd {
            
            kul(ie, input);
        } else {
            
            bvz = Some(qwg(ie, input));
        }
    }
}


fn qwg(cmd: &str, bvz: Option<String>) -> String {
    
    DE_.store(true, core::sync::atomic::Ordering::SeqCst);
    {
        let mut k = HO_.lock();
        k.clear();
    }
    
    
    kul(cmd, bvz);
    
    
    DE_.store(false, core::sync::atomic::Ordering::SeqCst);
    let k = HO_.lock();
    k.clone()
}


fn kul(cmd: &str, bvz: Option<String>) {
    
    
    let (rgw, ehg) = {
        let mut pay: Option<usize> = None;
        let mut jip: i32 = 0;
        let mut gjt = false;
        let mut gjq = false;
        let bf = cmd.as_bytes();
        let mut a = 0;
        while a < bf.len() {
            let bm = bf[a] as char;
            match bm {
                '\'' if !gjq => gjt = !gjt,
                '"' if !gjt => gjq = !gjq,
                '(' if !gjt && !gjq => jip += 1,
                ')' if !gjt && !gjq => {
                    if jip > 0 { jip -= 1; }
                }
                '>' if !gjt && !gjq && jip == 0 => {
                    pay = Some(a);
                    break;
                }
                _ => {}
            }
            a += 1;
        }
        if let Some(u) = pay {
            let bte = cmd[u..].cj(">>");
            let file = if bte {
                cmd[u + 2..].em()
            } else {
                cmd[u + 1..].em()
            };
            (cmd[..u].em(), Some((file, bte)))
        } else {
            (cmd, None)
        }
    };
    
    
    let ek: Vec<&str> = rgw.ayt().collect();
    if ek.is_empty() {
        return;
    }
    
    
    let ro = ek[0];
    if let Some(mum) = unix::tct(ro) {
        
        let uso = if ek.len() > 1 {
            alloc::format!("{} {}", mum, ek[1..].rr(" "))
        } else {
            mum
        };
        azu(&uso);
        return;
    }
    let ro = ek[0];
    let n = &ek[1..];
    
    match ro {
        
        "help" => commands::kis(n),
        "man" => commands::nee(n),
        "info" => commands::rfj(),
        "version" => commands::rkf(),
        "uname" => commands::iom(n),
        "ls" | "dir" => commands::ioj(n),
        "cd" => commands::kig(n),
        "pwd" => commands::kjb(),
        "mkdir" => commands::iok(n),
        "rmdir" => commands::rhr(n),
        "touch" => commands::kji(n),
        "rm" | "del" => commands::kjc(n),
        "cp" | "copy" => commands::kii(n),
        "mv" | "move" | "rename" => commands::kix(n),
        "cat" | "type" => commands::hde(n, ehg, bvz.ahz()),
        "head" => commands::kir(n, bvz.ahz()),
        "tail" => commands::kjg(n, bvz.ahz()),
        "wc" => commands::kjk(n, bvz.ahz()),
        "stat" => commands::rim(n),
        "tree" => commands::kjj(n),
        "find" => commands::kip(n),
        "echo" => commands::kin(n, ehg),
        "grep" => commands::kiq(n, bvz.ahz()),
        "clear" | "cls" => commands::iof(),
        "time" | "uptime" => commands::rjj(),
        "date" => commands::kij(),
        "whoami" => commands::rks(),
        "hostname" => commands::kit(),
        "id" => commands::rff(),
        "env" | "printenv" => commands::iog(),
        "history" => commands::rez(),
        "ps" => commands::kja(),
        "free" => commands::ioh(),
        "df" => commands::kik(),
        "login" => commands::rfw(),
        "su" => commands::ris(n),
        "passwd" => commands::rgx(n),
        "adduser" | "useradd" => commands::rcb(n),
        "deluser" | "userdel" => commands::rdo(n),
        "users" => commands::rkd(),
        "hwtest" => commands::kjh(),
        "memtest" => commands::rgk(),
        "restest" => commands::rhp(),
        "inttest" => commands::rfk(),
        "debugnew" => commands::rdn(),
        "nvme" => commands::rgs(),
        "keytest" => commands::rfq(),
        "hexdump" | "xxd" => commands::neb(n),
        "panic" => commands::rgv(),
        "exit" | "logout" => commands::rfx(),
        "reboot" => commands::rhm(),
        "shutdown" | "halt" | "poweroff" => commands::rex(),
        "suspend" | "s3" => commands::kje(),
        "neofetch" => commands::kiy(),
        "matrix" => commands::rgg(),
        "rain" => {
            
            if n.is_empty() {
                let bc = crate::desktop::Aa.lock();
                let j = match bc.eup { 0 => "slow", 2 => "fast", _ => "mid" };
                drop(bc);
                crate::println!("Current rain preset: {}", j);
                crate::println!("Usage: rain <slow|mid|fast>");
            } else {
                let akl: u8 = match n[0] {
                    "slow" | "s" | "0" => 0,
                    "mid" | "m" | "1" | "medium" => 1,
                    "fast" | "f" | "2" => 2,
                    _ => {
                        crate::println!("Unknown preset '{}'. Use: slow, mid, fast", n[0]);
                        return;
                    }
                };
                crate::desktop::Aa.lock().mev(akl);
                let j = match akl { 0 => "slow", 2 => "fast", _ => "mid" };
                crate::println!("Rain speed set to: {}", j);
            }
        },
        "cowsay" => commands::rdb(n),

        
        "benchmark" | "bench" => desktop::kif(n),
        "showcase" => desktop::ric(n),
        "showcase-jarvis" | "jarvis-showcase" | "jdemo" => desktop::rid(n),
        "showcase3d" | "demo3d" => desktop::neh(),
        "demo" | "tutorial" | "tour" => desktop::rdp(n),
        "filled3d" => desktop::ndy(),
        "desktop" | "gui" => desktop::jcx(None),
        "mobile" => desktop::ucz(),
        "cosmic" => desktop::ndt(),
        "open" => desktop::rgu(n),
        "trustedit" | "edit3d" | "3dedit" => desktop::jcx(Some(("TrustEdit 3D", crate::desktop::WindowType::Fp, 100, 60, 700, 500))),
        "calculator" | "calc" => desktop::jcx(Some(("Calculator", crate::desktop::WindowType::Calculator, 300, 200, 320, 420))),
        "snake" => desktop::jcx(Some(("Snake", crate::desktop::WindowType::Io, 200, 100, 400, 400))),
        "signature" | "sig" => desktop::rie(n),
        "security" | "sec" | "caps" => desktop::rhy(n),

        
        "vm" | "linux" => {
            if n.is_empty() {
                vm::ioi();
            } else {
                match n[0] {
                    
                    "create" | "run" | "start" | "guests" | "inspect" | "mount" | "input"
                    | "debug" | "stack" | "regs" | "dump" | "linux" => vm::rki(n),
                    "status" => vm::rev(),
                    "install" => vm::reu(),
                    "console" | "shell" => vm::ioi(),
                    "stop" => vm::rkk(),
                    "list" => vm::rkj(),
                    "extract" => apps::nhi(),
                    "exec" => {
                        if n.len() > 1 {
                            let dyy = n[1];
                            let ilo: Vec<&str> = n[2..].ip();
                            match crate::linux_compat::exec(dyy, &ilo) {
                                Ok(aj) => crate::println!("[Exited with code {}]", aj),
                                Err(aa) => crate::h!(0xFF0000, "Error: {}", aa),
                            }
                        } else {
                            crate::println!("Usage: linux exec <binary> [args...]");
                            crate::println!("Example: linux exec /bin/busybox ls");
                        }
                    },
                    "help" | "--help" | "-h" => vm::nek(),
                    _ => vm::nek(),
                }
            }
        },
        "distro" | "distros" => {
            if n.is_empty() {
                vm::kil();
            } else {
                match n[0] {
                    "list" => vm::kil(),
                    "install" | "download" => {
                        if n.len() > 1 { vm::rdu(n[1]); }
                        else { vm::ndx(); }
                    },
                    "run" | "start" => {
                        if n.len() > 1 { vm::rdv(n[1]); }
                        else { crate::println!("Usage: distro run <id>"); }
                    },
                    "pick" | "select" => vm::ndx(),
                    _ => vm::kil(),
                }
            }
        },
        "glmode" | "compositor" => vm::reo(n),
        "theme" => vm::rjh(n),
        "anim" | "animations" => vm::rcf(n),
        "holo" | "holomatrix" => vm::rfa(n),
        "imgview" | "imageview" | "view" => vm::rfi(n),
        "imgdemo" | "imagedemo" => vm::rfh(n),
        "tasks" | "jobs" => vm::riz(),
        "threads" => vm::rji(),
        "alpine" => vm::rce(n),
        "apt-get" | "apt" | "apk" | "dpkg" => vm::rhb(ro, n),
        "persist" | "persistence" => vm::rha(n),
        "disk" => vm::rdt(),
        "dd" => vm::rdm(n),
        "ahci" => vm::rcc(n),
        "fdisk" | "partitions" => vm::reh(n),
        "lspci" => vm::rge(n),
        "lshw" | "hwinfo" => vm::rga(),
        "gpu" => vm::rep(n),
        "gpuexec" | "gpurun" | "gpuagent" => commands::req(n),
        "sdma" | "dma" => commands::rhx(n),
        "neural" | "nn" | "gemm" => commands::rgo(n),
        "gpufw" | "firmware" => commands::rer(n),
        "a11y" | "accessibility" => vm::rca(n),
        "beep" => vm::rco(n),
        "audio" => vm::rcj(n),
        "synth" => vm::riu(n),
        "play" => vm::rhc(n),
        "vizfx" | "liveviz" => vm::rkh(n),
        "daw" | "trustdaw" => vm::rdh(n),
        "ifconfig" | "ip" => vm::hdh(),
        "ipconfig" => vm::rfn(n),
        "ping" => vm::kiz(n),
        "tcpsyn" => vm::rja(n),
        "httpget" => vm::rfb(n),
        "curl" | "wget" => vm::ndv(n),
        "download" => vm::rdy(n),
        "nslookup" | "dig" => vm::rgr(n),
        "arp" => vm::rch(n),
        "route" => vm::rhs(n),
        "traceroute" | "tracert" => vm::rjn(n),
        "netstat" => vm::hdi(),
        "exec" | "run" | "./" => vm::rec(n, ro),
        "elfinfo" => vm::reb(n),
        "lsusb" => unix::rgf(),
        "checkm8" => {
            let qkj = n.rr(" ");
            let result = crate::drivers::checkm8::wbg(&qkj);
            crate::println!("{}", result);
        }
        "lscpu" => unix::ned(),
        "smpstatus" => unix::rig(),
        "smp" => unix::rif(n),
        "fontsmooth" => unix::rek(n),
        "hv" | "hypervisor" => vm::rfe(n),

        
        "nmap" | "portscan" | "scan" => vm::rgq(n),
        "discover" | "hostscan" | "arpscan" => vm::ndw(n),
        "banner" | "grabber" => vm::rck(n),
        "sniff" | "capture" | "tcpdump" => vm::rih(n),
        "vulnscan" | "vuln" => vm::rkm(n),
        "scantest" | "netscantest" => vm::rgn(n),

        
        "httpd" | "httpserv" | "webserv" => commands::kiu(n),

        
        "trustpkg" | "pkg" => commands::rjt(n),

        
        "browse" | "www" | "web" => network::rcs(n),
        "sandbox" | "websandbox" => network::rhv(n),
        "container" | "webcontainer" | "wc" => network::rda(n),

        
        "which" => unix::hdl(n),
        "whereis" => unix::rkr(n),
        "file" => unix::rei(n),
        "basename" => unix::rcm(n),
        "dirname" => unix::rds(n),
        "realpath" => unix::rhl(n),
        "sort" => unix::rii(n, bvz.ahz()),
        "uniq" => unix::rjz(n, bvz.ahz()),

        
        "nano" | "vi" | "edit" => editor::rgm(n),

        
        "alias" => unix::rcd(n),
        "unalias" => unix::rjy(n),
        "bc" => unix::rcn(n),
        "diff" => unix::rdr(n),
        "md5sum" => unix::rgh(n),
        "sha256sum" => unix::rib(n),
        "base64" => unix::rcl(n, bvz.ahz()),
        "cut" => unix::rdg(n, bvz.ahz()),
        "tr" => unix::rjm(n, bvz.ahz()),
        "tee" => unix::rjb(n, bvz.ahz()),
        "xargs" => unix::rku(n, bvz.ahz()),
        "chmod" => unix::rcw(n),
        "chown" => unix::rcy(n),
        "ln" => unix::rfv(n),
        "readlink" => unix::rhk(n),
        "watch" => unix::rkn(n),
        "timeout" => unix::rjl(n),
        "tar" => unix::riy(n),
        "gzip" => unix::rew(n),
        "zip" => unix::rkw(n),
        "unzip" => unix::rka(n),
        "service" => unix::kjd(n),
        "systemctl" => unix::rix(n),
        "crontab" => unix::rdf(n),
        "at" => unix::rci(n),
        "unset" => unix::nej(n),
        "read" => unix::rhj(n),

        "yes" => unix::rkv(n),
        "seq" => unix::rhz(n),
        "sleep" => unix::kje(n),
        "kill" => unix::rfr(n),
        "killall" => unix::rfs(n),
        "nice" => unix::rgp(n),

        "top" => unix::iol(),
        "htop" => unix::iol(),
        "vmstat" => unix::rkl(),
        "iostat" => unix::rfm(),
        "strace" => unix::rip(n),
        "dmidecode" => unix::rdw(),
        "hdparm" => unix::rey(n),
        "screenshot" | "scrot" => unix::rhw(n),
        "httpd" | "serve" => unix::kiu(n),
        "benchmark" | "bench" => unix::kif(),
        "uptime" => unix::rkc(),

        "lsof" => unix::rgd(n),

        "strings" => unix::riq(n),

        "mount" => unix::kiw(n),
        "umount" => unix::rjx(n),
        "fsck" => unix::rel(n),

        "sync" => unix::rit(),
        "lsblk" => unix::rfz(),
        "blkid" => unix::rcp(),

        "export" => unix::kio(n),

        "source" | "." => unix::rij(n),
        "set" => unix::ria(n),

        "printf" => unix::rhg(n),
        "test" | "[" => unix::rjd(n),
        "expr" => unix::red(n),

        "cal" => unix::rcu(n),

        "cmp" => unix::rcz(n),

        "od" => unix::rgt(n),
        "rev" => unix::rhq(n),
        "factor" => unix::ree(n),

        "tty" => unix::rjv(),
        "stty" => unix::rir(n),
        "reset" => unix::rho(),

        "lsmem" => unix::rgb(),

        "lsmod" => unix::rgc(),

        "sysctl" => unix::riw(n),
        "firewall" | "iptables" | "fw" => unix::rej(n),
        "du" => unix::rea(n),

        "dmesg" => unix::kim(n),
        "memdbg" | "heapdbg" => unix::rgi(),
        "perf" | "perfstat" => unix::rgz(),
        "irqstat" | "irqs" => unix::rfo(),
        "regs" | "registers" | "cpuregs" => unix::rhn(),
        "peek" | "memdump" => unix::rgy(n),
        "poke" | "memwrite" => unix::rhd(n),
        "devpanel" => unix::rdq(),
        "timecmd" => unix::rjk(n),

        
        "hwdiag" | "diagnostic" | "diag" => unix::rfc(),
        "cpudump" | "fullregs" => unix::rdc(),
        "stacktrace" | "backtrace" | "bt" => unix::rik(n),
        "bootlog" | "checkpoints" => unix::rcq(),
        "postcode" => unix::rhe(n),
        "ioport" => unix::rfl(n),
        "rdmsr" => unix::rhi(n),
        "wrmsr" => unix::rkt(n),
        "cpuid" => unix::rde(n),
        "memmap" => unix::rgj(),
        "watchdog" | "wdt" => unix::rko(n),

        
        "fan" => crate::drivers::thinkpad_ec::reg(n),
        "temp" | "sensors" => crate::drivers::thinkpad_ec::rjc(n),
        "cpufreq" | "speedstep" => crate::drivers::thinkpad_ec::rdd(n),

        
        "wayland" | "wl" => apps::rkp(n),
        "gterm" | "graphterm" => apps::res(n),
        "transpile" | "disasm" | "analyze" => apps::rjo(n),
        "rv-xlat" | "rvxlat" | "xlat" => apps::rhu(n),
        "rv-disasm" | "rvdisasm" => apps::rht(n),
        "trustview" | "tv" => apps::rju(n),
        "lab" | "trustlab" => apps::rft(n),
        "hwscan" | "trustprobe" | "probe" => apps::rfd(n),
        "trustlang" | "tl" => apps::rjp(n),
        "trustlang_showcase" | "tl_showcase" => apps::rjq(),
        "film" | "trustos_film" => apps::rjr(),
        "trailer" | "trustos_trailer" => trailer::rjs(),
        "video" => apps::rkg(n),

        
        "jarvis" | "j" | "ai" | "assistant" => jarvis::rfp(n),
        "mesh" | "jarvis-mesh" | "jmesh" => commands::rgl(n),
        "pxe" | "pxeboot" | "replicate" => commands::rhh(n),
        "guardian" | "pact" | "gardien" => commands::ret(n),

        "" => {}
        _ if unix::xmy(ro) => {}
        _ => {
            
            if vm::xmn(ro, n) {
                return;
            }
            crate::gr!(A_, "tsh: ");
            crate::print!("{}", ro);
            crate::h!(A_, ": command not found");
        }
    }
}





mod commands;       
pub(crate) mod desktop;    
mod vm;             
mod network;        
mod unix;           
mod apps;           
mod trailer;        
mod jarvis;         
pub(crate) mod scripting;  
mod editor;         
