




use alloc::string::String;
use alloc::vec::Vec;
use alloc::vec;
use alloc::boxed::Box;
use alloc::format;
use core::sync::atomic::{AtomicBool, Ordering};
use spin::Mutex as SpinMutex;
use crate::framebuffer::{B_, G_, AX_, D_, A_, C_, R_, CF_, DM_, K_};
use crate::ramfs::FileType;


static Sy: AtomicBool = AtomicBool::new(false);


pub fn cbc() -> bool {
    Sy.load(Ordering::SeqCst)
}


pub fn dks() {
    Sy.store(false, Ordering::SeqCst);
}


pub fn fag() {
    Sy.store(true, Ordering::SeqCst);
}


pub(crate) static DL_: AtomicBool = AtomicBool::new(false);

static IG_: SpinMutex<String> = SpinMutex::new(String::new());


pub fn khm(j: &str) {
    if DL_.load(core::sync::atomic::Ordering::Relaxed) {
        let mut buf = IG_.lock();
        buf.push_str(j);
    }
}


pub fn btp() -> bool {
    DL_.load(core::sync::atomic::Ordering::Relaxed)
}



pub fn fcj() -> String {
    let mut buf = IG_.lock();
    core::mem::take(&mut *buf)
}



#[inline]
fn guy() {
    let (col, row) = crate::framebuffer::cyk();
    crate::framebuffer::fill_rect(
        (col * 8) as u32, (row * 16) as u32, 8, 16, B_,
    );
}






#[repr(C)]
pub struct Ii {
    pub buf_ptr: *mut u32,
    pub buf_len: usize,
    pub width: u32,
    pub height: u32,
    pub matrix_chars: *const u8,              
    pub matrix_heads: *const i32,              
    pub holo_intensity: *const u8,             
    pub matrix_rows: usize,                    
}

unsafe impl Send for Ii {}
unsafe impl Sync for Ii {}



pub(super) fn izr(start: usize, end: usize, data: *mut u8) {
    let params = unsafe { &*(data as *const Ii) };
    let buf_ptr = params.buf_ptr;
    let buf_len = params.buf_len;
    let width = params.width;
    let height = params.height;
    let iey = params.holo_intensity;
    let mjn = !iey.is_null();
    let matrix_rows = params.matrix_rows;
    
    let ati = 8u32;
    
    for col in start..end {
        if col >= 240 { break; }
        
        let x = col as u32 * ati;
        if x >= width { continue; }
        
        let su = unsafe { *params.matrix_heads.add(col) };
        let bpd = (su - 5).max(0) as usize;
        let fuv = if su + 30 < 0 { 0 } else { ((su + 30) as usize).min(matrix_rows) };
        
        for row in bpd..fuv {
            let y = row as u32 * 16;
            if y >= height { continue; }
            
            let em = row as i32 - su;
            
            
            let dik: u32 = if em < 0 {
                continue;
            } else if em == 0 {
                255  
            } else if em <= 12 {
                255 - (em as u32 * 8)
            } else if em <= 28 {
                
                let ha = ((em - 12) as u32).min(15) * 16;
                let ln = 255u32.saturating_sub(ha);
                (160 * ln) / 255
            } else {
                continue;
            };
            
            
            let fxe = col * matrix_rows + row;
            
            
            
            let (final_char, color) = if mjn {
                let mmd = unsafe { *iey.add(fxe) };
                if mmd >= 1 {
                    
                    ('#', 0xFF00FF00)
                } else {
                    
                    let c = unsafe { *params.matrix_chars.add(fxe) as char };
                    (c, 0xFF000000 | (dik << 8))
                }
            } else {
                
                let c = unsafe { *params.matrix_chars.add(fxe) as char };
                (c, 0xFF000000 | (dik << 8))
            };
            
            let du = crate::framebuffer::font::ol(final_char);
            
            
            unsafe {
                for (r, &bits) in du.iter().enumerate() {
                    let o = y + r as u32;
                    if o >= height { break; }
                    let pq = (o * width) as usize;
                    
                    if bits != 0 {
                        let ani = x as usize;
                        if bits & 0x80 != 0 { let idx = pq + ani; if idx < buf_len { *buf_ptr.add(idx) = color; } }
                        if bits & 0x40 != 0 { let idx = pq + ani + 1; if idx < buf_len { *buf_ptr.add(idx) = color; } }
                        if bits & 0x20 != 0 { let idx = pq + ani + 2; if idx < buf_len { *buf_ptr.add(idx) = color; } }
                        if bits & 0x10 != 0 { let idx = pq + ani + 3; if idx < buf_len { *buf_ptr.add(idx) = color; } }
                        if bits & 0x08 != 0 { let idx = pq + ani + 4; if idx < buf_len { *buf_ptr.add(idx) = color; } }
                        if bits & 0x04 != 0 { let idx = pq + ani + 5; if idx < buf_len { *buf_ptr.add(idx) = color; } }
                        if bits & 0x02 != 0 { let idx = pq + ani + 6; if idx < buf_len { *buf_ptr.add(idx) = color; } }
                        if bits & 0x01 != 0 { let idx = pq + ani + 7; if idx < buf_len { *buf_ptr.add(idx) = color; } }
                    }
                }
            }
        }
    }
}


pub const AJS_: &[&str] = &[
    
    "help", "man", "info", "version", "uname",
    
    "ls", "dir", "cd", "pwd", "mkdir", "rmdir", "touch", "rm", "del", "cp", "copy",
    "mv", "move", "rename", "cat", "type", "head", "tail", "wc", "stat", "tree", "find",
    
    "echo", "grep",
    
    "clear", "cls", "time", "uptime", "date", "whoami", "hostname", "id", "env", "printenv",
    "history", "ps", "free", "df",
    
    "login", "su", "passwd", "adduser", "useradd", "deluser", "userdel", "users", "logout",
    
    "hwtest", "keytest", "hexdump", "xxd", "panic",
    
    "hwdiag", "cpudump", "stacktrace", "backtrace", "bootlog", "postcode",
    "ioport", "rdmsr", "wrmsr", "cpuid", "memmap", "watchdog", "drv",
    
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
    
    "ifconfig", "ip", "ipconfig", "wifi", "ping", "curl", "wget", "download",
    "nslookup", "dig", "arp", "route", "netstat",
    
    "which", "file", "chmod", "ln", "sort", "uniq", "cut",
    "kill", "top", "dmesg", "strings", "tar",
    "mount", "umount", "sync", "lsblk",
    
    "exit", "reboot", "shutdown", "poweroff",
    
    "exec", "run",
    
    "trustview", "tv",
    
    "lab", "trustlab",
    
    "hwscan", "trustprobe", "probe",
    
    "hwdbg", "hwdebug",
    
    "marionet", "mario",
    
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
    
    "netconsole", "nc", "strace",
];


pub fn run() -> ! {
    
    crate::framebuffer::guh();
    crate::framebuffer::clear();

    
    scripting::init();

    

    
    crate::interrupts::gue(true);

    nxd();

    
    unix::ojg();
    
    let mut dkv = [0u8; 512];
    
    loop {
        iwp();
        
        
        let len = oct(&mut dkv);
        
        
        let krz = core::str::from_utf8(&dkv[..len]).unwrap_or("");
        dks(); 
        aav(krz.trim());
    }
}


fn oct(buffer: &mut [u8]) -> usize {
    use crate::keyboard::{ya, fgf, gat, gas, gau,
                          T_, S_, AI_, AJ_, CW_, CV_, DE_,
                          AM_, AO_};
    
    let mut pos: usize = 0;
    let mut cursor: usize = 0;
    let mut pl: Vec<&str> = Vec::new();
    let mut acu: i32 = -1; 
    let mut ahb = false;
    
    
    
    let (_scr_w, ezo) = crate::framebuffer::kv();
    let xw = (ezo as usize) / 16; 
    let mut nl = crate::framebuffer::cyk().1;
    let agn = crate::framebuffer::cyk().0;
    const AKG_: usize = 4; 
    if xw > AKG_ && nl + AKG_ >= xw {
        let ikh = nl + AKG_ - xw + 1;
        for _ in 0..ikh {
            crate::framebuffer::scroll_up();
        }
        nl = nl.saturating_sub(ikh);
        crate::framebuffer::afr(agn, nl);
    }
    
    
    let mut cursor_visible = true;
    let mut blink_counter: u32 = 0;
    const BNX_: u32 = 500000;
    
    gau();
    
    
    guy();
    
    loop {
        if let Some(c) = ya() {
            
            if c != AM_ && c != AO_ && crate::framebuffer::geb() {
                let (_col, row) = crate::framebuffer::jam();
                nl = row;
                
                
                crate::framebuffer::afr(agn + cursor, nl);
            }
            
            
            let edg = if cursor < pos { buffer[cursor] as char } else { ' ' };
            crate::aru!("{}", edg);
            crate::aru!("\x08");
            cursor_visible = true;
            blink_counter = 0;
            
            match c {
                b'\n' | b'\r' => {
                    
                    crate::framebuffer::olw();
                    
                    
                    if acu >= 0 && (acu as usize) < pl.len() {
                        let selected = pl[acu as usize];
                        bfh(nl, pl.len());
                        flw(agn, nl, pos);
                        let bytes = selected.as_bytes();
                        let len = bytes.len().min(buffer.len() - 1);
                        buffer[..len].copy_from_slice(&bytes[..len]);
                        pos = len;
                        cursor = len;
                        for i in 0..pos {
                            crate::print!("{}", buffer[i] as char);
                        }
                    } else if ahb {
                        bfh(nl, pl.len());
                    }
                    crate::println!();
                    let cmd = core::str::from_utf8(&buffer[..pos]).unwrap_or("");
                    if !cmd.trim().is_empty() {
                        fgf(cmd);
                    }
                    break;
                }
                b'\t' => {
                    
                    if !pl.is_empty() {
                        let idx = if acu >= 0 { acu as usize } else { 0 };
                        let selected = pl[idx];
                        bfh(nl, pl.len());
                        flw(agn, nl, pos);
                        let bytes = selected.as_bytes();
                        let len = bytes.len().min(buffer.len() - 1);
                        buffer[..len].copy_from_slice(&bytes[..len]);
                        pos = len;
                        cursor = len;
                        for i in 0..pos {
                            crate::print!("{}", buffer[i] as char);
                        }
                        acu = -1;
                        ahb = false;
                        pl.clear();
                        if pos < buffer.len() - 1 {
                            buffer[pos] = b' ';
                            pos += 1;
                            cursor += 1;
                            crate::print!(" ");
                        }
                    }
                }
                0x1B => {
                    
                    if ahb {
                        bfh(nl, pl.len());
                        pl.clear();
                        acu = -1;
                        ahb = false;
                    }
                }
                0x08 => {
                    
                    if cursor > 0 {
                        
                        if ahb {
                            bfh(nl, pl.len());
                            crate::framebuffer::afr(agn + cursor, nl);
                        }
                        for i in cursor..pos {
                            buffer[i - 1] = buffer[i];
                        }
                        pos = pos.saturating_sub(1);
                        cursor = cursor.saturating_sub(1);
                        crate::aru!("\x08");
                        for i in cursor..pos {
                            crate::print!("{}", buffer[i] as char);
                        }
                        crate::aru!(" ");
                        for _ in cursor..=pos {
                            crate::aru!("\x08");
                        }
                        
                        fdy(buffer, pos, &mut pl);
                        if !pl.is_empty() && pos > 0 {
                            dem(nl, &pl, acu);
                            ahb = true;
                            crate::framebuffer::afr(agn + cursor, nl);
                        } else {
                            ahb = false;
                            acu = -1;
                        }
                    }
                }
                T_ => {
                    if pos == 0 {
                        
                        if let Some(prev) = gat() {
                            let bytes = prev.as_bytes();
                            let len = bytes.len().min(buffer.len() - 1);
                            buffer[..len].copy_from_slice(&bytes[..len]);
                            pos = len;
                            cursor = len;
                            crate::print!("{}", &prev[..len]);
                        }
                    } else {
                        
                        if !ahb {
                            
                            fdy(buffer, pos, &mut pl);
                            if !pl.is_empty() {
                                acu = 0;
                                dem(nl, &pl, acu);
                                ahb = true;
                                
                                crate::framebuffer::afr(agn + cursor, nl);
                            }
                        } else if !pl.is_empty() {
                            
                            bfh(nl, pl.len());
                            if acu <= 0 {
                                acu = pl.len() as i32 - 1;
                            } else {
                                acu -= 1;
                            }
                            dem(nl, &pl, acu);
                            crate::framebuffer::afr(agn + cursor, nl);
                        }
                    }
                }
                S_ => {
                    if pos == 0 {
                        
                        if let Some(next) = gas() {
                            let bytes = next.as_bytes();
                            let len = bytes.len().min(buffer.len() - 1);
                            buffer[..len].copy_from_slice(&bytes[..len]);
                            pos = len;
                            cursor = len;
                            crate::print!("{}", &next[..len]);
                        } else {
                            flw(agn, nl, pos);
                            pos = 0;
                            cursor = 0;
                        }
                    } else if ahb && !pl.is_empty() {
                        
                        bfh(nl, pl.len());
                        acu += 1;
                        if acu >= pl.len() as i32 {
                            acu = 0;
                        }
                        dem(nl, &pl, acu);
                        crate::framebuffer::afr(agn + cursor, nl);
                    }
                }
                AI_ => {
                    if cursor > 0 {
                        cursor -= 1;
                        crate::aru!("\x08");
                    }
                }
                AJ_ => {
                    if cursor < pos {
                        crate::print!("{}", buffer[cursor] as char);
                        cursor += 1;
                    }
                }
                CW_ => {
                    while cursor > 0 {
                        crate::aru!("\x08");
                        cursor -= 1;
                    }
                }
                CV_ => {
                    while cursor < pos {
                        crate::print!("{}", buffer[cursor] as char);
                        cursor += 1;
                    }
                }
                DE_ => {
                    if cursor < pos {
                        if ahb {
                            bfh(nl, pl.len());
                            crate::framebuffer::afr(agn + cursor, nl);
                        }
                        for i in cursor..pos.saturating_sub(1) {
                            buffer[i] = buffer[i + 1];
                        }
                        pos = pos.saturating_sub(1);
                        for i in cursor..pos {
                            crate::print!("{}", buffer[i] as char);
                        }
                        crate::aru!(" ");
                        for _ in cursor..=pos {
                            crate::aru!("\x08");
                        }
                        fdy(buffer, pos, &mut pl);
                        if !pl.is_empty() && pos > 0 {
                            dem(nl, &pl, acu);
                            ahb = true;
                            crate::framebuffer::afr(agn + cursor, nl);
                        } else {
                            ahb = false;
                            acu = -1;
                        }
                    }
                }
                AM_ => {
                    crate::framebuffer::olx(10);
                }
                AO_ => {
                    crate::framebuffer::scroll_down(10);
                    
                    
                    if !crate::framebuffer::geb() {
                        let (_, row) = crate::framebuffer::cyk();
                        nl = row;
                        crate::framebuffer::afr(agn + cursor, nl);
                    }
                }
                27 => {
                    
                    if crate::framebuffer::geb() {
                        let (_col, row) = crate::framebuffer::jam();
                        nl = row;
                        
                        crate::framebuffer::afr(agn + cursor, nl);
                    }
                }
                3 => {
                    
                    if ahb {
                        bfh(nl, pl.len());
                        ahb = false;
                    }
                    crate::bq!(A_, "^C");
                    crate::println!();
                    fag();
                    pos = 0;
                    break;
                }
                12 => {
                    
                    if ahb {
                        bfh(nl, pl.len());
                        ahb = false;
                    }
                    crate::framebuffer::clear();
                    iwp();
                    for i in 0..pos {
                        crate::print!("{}", buffer[i] as char);
                    }
                    for _ in cursor..pos {
                        crate::aru!("\x08");
                    }
                }
                c if c >= 0x20 && c < 0x7F && pos < buffer.len() - 1 => {
                    
                    
                    if ahb {
                        bfh(nl, pl.len());
                        crate::framebuffer::afr(agn + cursor, nl);
                    }
                    
                    if cursor < pos {
                        for i in (cursor..pos).rev() {
                            buffer[i + 1] = buffer[i];
                        }
                    }
                    buffer[cursor] = c;
                    pos += 1;
                    cursor += 1;
                    for i in cursor - 1..pos {
                        crate::print!("{}", buffer[i] as char);
                    }
                    for _ in cursor..pos {
                        crate::aru!("\x08");
                    }
                    
                    
                    fdy(buffer, pos, &mut pl);
                    if !pl.is_empty() {
                        dem(nl, &pl, acu);
                        ahb = true;
                        
                        crate::framebuffer::afr(agn + cursor, nl);
                    } else {
                        ahb = false;
                        acu = -1;
                    }
                }
                _ => {}
            }
            
            
            guy();
        } else {
            
            blink_counter += 1;
            if blink_counter >= BNX_ {
                blink_counter = 0;
                cursor_visible = !cursor_visible;
                
                if cursor_visible {
                    guy();
                } else {
                    let edg = if cursor < pos { buffer[cursor] as char } else { ' ' };
                    crate::aru!("{}", edg);
                    crate::aru!("\x08");
                }
            }
            
            
            
            {
                static CNO_: core::sync::atomic::AtomicU32 = core::sync::atomic::AtomicU32::new(0);
                let count = CNO_.fetch_add(1, core::sync::atomic::Ordering::Relaxed);
                if count > 500_000 && count % 5000 == 0 {
                    crate::netstack::poll();
                }
                
                if count % 100 == 0 && crate::jarvis::mesh::is_active() {
                    crate::jarvis::inl();
                }
                
                if count % 10000 == 0 {
                    crate::jarvis::mentor::nwa();
                }
            }
            for _ in 0..100 { core::hint::spin_loop(); }
        }
    }
    
    
    let edg = if cursor < pos { buffer[cursor] as char } else { ' ' };
    crate::aru!("{}", edg);
    if cursor < pos { crate::aru!("\x08"); }
    
    buffer[pos] = 0;
    pos
}


fn fdy(buffer: &[u8], pos: usize, pl: &mut Vec<&'static str>) {
    pl.clear();
    if pos == 0 {
        return;
    }
    
    let input = match core::str::from_utf8(&buffer[..pos]) {
        Ok(j) => j,
        Err(_) => return,
    };
    
    let first_word = input.split_whitespace().next().unwrap_or("");
    if first_word.is_empty() || input.contains(' ') {
        return;
    }
    
    for cmd in AJS_ {
        if cmd.starts_with(first_word) && *cmd != first_word {
            pl.push(cmd);
            if pl.len() >= 8 {
                break;
            }
        }
    }
}



fn dem(nl: usize, pl: &[&str], selected_idx: i32) {
    if pl.is_empty() {
        return;
    }
    
    
    let (_width, height) = crate::framebuffer::kv();
    let buh = (height as usize) / 16; 
    let fg = crate::framebuffer::dqp();
    let bg = 0xFF000000u32; 
    
    for (i, cmd) in pl.iter().enumerate() {
        let row = nl + 1 + i;
        if row >= buh { break; } 
        crate::framebuffer::hle(row);
        if i as i32 == selected_idx {
            let nm = alloc::format!(" > {}", cmd);
            crate::framebuffer::ftd(0, row, &nm, fg, bg);
        } else {
            let nm = alloc::format!("   {}", cmd);
            crate::framebuffer::ftd(0, row, &nm, fg, bg);
        }
    }
}



fn bfh(nl: usize, count: usize) {
    let (_width, height) = crate::framebuffer::kv();
    let buh = (height as usize) / 16; 
    for i in 0..count {
        let row = nl + 1 + i;
        if row >= buh { break; }
        crate::framebuffer::hle(row);
    }
}



fn flw(agn: usize, nl: usize, pos: usize) {
    let (width, _) = crate::framebuffer::kv();
    let cols = (width as usize) / 8; 
    
    let kkt = (pos + 2).min(cols.saturating_sub(agn));
    
    let cdt: alloc::string::String = core::iter::repeat(' ').take(kkt).collect();
    crate::framebuffer::ftd(agn, nl, &cdt, 0xFF000000, 0xFF000000);
    
    crate::framebuffer::afr(agn, nl);
}

fn nxd() {
    crate::println!();
    crate::n!(G_, r" _____ ____            _    ___      ");
    crate::n!(G_, r"|_   _|  _ \ _   _ ___| |_ / _ \ ___ ");
    crate::n!(B_,        r"  | | | |_) | | | / __| __| | | / __|");
    crate::n!(B_,        r"  | | |  _ <| |_| \__ \ |_| |_| \__ \");
    crate::n!(AX_,   r"  |_| |_| \_\\__,_|___/\__\\___/|___/");
    crate::println!();
    crate::n!(C_, "  T-RustOs v0.2.0 - Type 'help' for commands");
    crate::println!();
}

fn iwp() {
    
    let fm = crate::rtc::aou();
    let cwd = if crate::ramfs::is_initialized() {
        crate::ramfs::bh(|fs| String::from(fs.pwd()))
    } else {
        String::from("/")
    };
    
    crate::bq!(AX_, "[{:02}:{:02}:{:02}] ", fm.hour, fm.minute, fm.second);
    crate::bq!(G_, "trustos");
    crate::bq!(R_, ":");
    crate::bq!(C_, "{}", cwd);
    crate::bq!(B_, "$ ");
}


pub fn read_line() -> alloc::string::String {
    let mut buf = [0u8; 512];
    let len = crate::keyboard::read_line(&mut buf);
    core::str::from_utf8(&buf[..len]).unwrap_or("").into()
}


pub fn aav(cmd: &str) {
    if cmd.is_empty() {
        return;
    }

    
    let expanded = scripting::bbm(cmd);
    let cmd = expanded.as_str();
    
    
    let iuy = ovb(cmd);
    if iuy.len() > 1 {
        lsd(&iuy);
        return;
    }
    
    
    fvo(cmd, None);
}


fn ovb(cmd: &str) -> Vec<&str> {
    let mut segments = Vec::new();
    let mut start = 0;
    let mut eqj = false;
    let mut eqi = false;
    let bytes = cmd.as_bytes();
    
    for i in 0..bytes.len() {
        match bytes[i] {
            b'\'' if !eqi => eqj = !eqj,
            b'"' if !eqj => eqi = !eqi,
            b'|' if !eqj && !eqi => {
                
                if i + 1 < bytes.len() && bytes[i + 1] == b'|' {
                    continue; 
                }
                segments.push(cmd[start..i].trim());
                start = i + 1;
            }
            _ => {}
        }
    }
    segments.push(cmd[start..].trim());
    segments
}


fn lsd(segments: &[&str]) {
    let mut amf: Option<String> = None;
    
    for (i, segment) in segments.iter().enumerate() {
        let clo = i == segments.len() - 1;
        let input = amf.take();
        
        if clo {
            
            fvo(segment, input);
        } else {
            
            amf = Some(khk(segment, input));
        }
    }
}


fn khk(cmd: &str, amf: Option<String>) -> String {
    
    DL_.store(true, core::sync::atomic::Ordering::SeqCst);
    {
        let mut buf = IG_.lock();
        buf.clear();
    }
    
    
    fvo(cmd, amf);
    
    
    DL_.store(false, core::sync::atomic::Ordering::SeqCst);
    let mut buf = IG_.lock();
    core::mem::take(&mut *buf)
}


fn fvo(cmd: &str, amf: Option<String>) {
    
    
    let (cmd_part, redirect) = {
        let mut iyz: Option<usize> = None;
        let mut ewd: i32 = 0;
        let mut czp = false;
        let mut czo = false;
        let bytes = cmd.as_bytes();
        let mut i = 0;
        while i < bytes.len() {
            let ch = bytes[i] as char;
            match ch {
                '\'' if !czo => czp = !czp,
                '"' if !czp => czo = !czo,
                '(' if !czp && !czo => ewd += 1,
                ')' if !czp && !czo => {
                    if ewd > 0 { ewd -= 1; }
                }
                '>' if !czp && !czo && ewd == 0 => {
                    iyz = Some(i);
                    break;
                }
                _ => {}
            }
            i += 1;
        }
        if let Some(pos) = iyz {
            let append = cmd[pos..].starts_with(">>");
            let file = if append {
                cmd[pos + 2..].trim()
            } else {
                cmd[pos + 1..].trim()
            };
            (cmd[..pos].trim(), Some((file, append)))
        } else {
            (cmd, None)
        }
    };
    
    
    let au: Vec<&str> = cmd_part.split_whitespace().collect();
    if au.is_empty() {
        return;
    }
    
    
    let command = au[0];
    if let Some(alias_value) = unix::mco(command) {
        
        let nis = if au.len() > 1 {
            alloc::format!("{} {}", alias_value, au[1..].join(" "))
        } else {
            alias_value
        };
        aav(&nis);
        return;
    }
    let command = au[0];
    let args = &au[1..];
    
    match command {
        
        "help" => commands::chf(args),
        "man" => commands::hmb(args),
        "info" => commands::kou(),
        "version" => commands::ktq(),
        "uname" => commands::eim(args),
        "ls" | "dir" => commands::eij(args),
        "cd" => commands::fme(args),
        "pwd" => commands::fmz(),
        "mkdir" => commands::eik(args),
        "rmdir" => commands::krb(args),
        "touch" => commands::fng(args),
        "rm" | "del" => commands::fna(args),
        "cp" | "copy" => commands::fmg(args),
        "mv" | "move" | "rename" => commands::fmv(args),
        "cat" | "type" => commands::dkw(args, redirect, amf.as_deref()),
        "head" => commands::fmp(args, amf.as_deref()),
        "tail" => commands::fne(args, amf.as_deref()),
        "wc" => commands::fni(args, amf.as_deref()),
        "stat" => commands::krx(args),
        "tree" => commands::fnh(args),
        "find" => commands::fmn(args),
        "echo" => commands::fmm(args, redirect),
        "grep" => commands::fmo(args, amf.as_deref()),
        "clear" | "cls" => commands::eif(),
        "time" | "uptime" => commands::ksu(),
        "date" => commands::fmh(),
        "whoami" => commands::kud(),
        "hostname" => commands::fmq(),
        "id" => commands::bfj(),
        "env" | "printenv" => commands::eig(),
        "history" => commands::kol(),
        "ps" => commands::fmy(),
        "free" => commands::eih(),
        "df" => commands::fmi(),
        "login" => commands::kpg(),
        "su" => commands::ksd(args),
        "passwd" => commands::kqh(args),
        "adduser" | "useradd" => commands::kll(args),
        "deluser" | "userdel" => commands::kmx(args),
        "users" => commands::kto(),
        "hwtest" => commands::fnf(),
        "memtest" => commands::kpu(),
        "restest" => commands::kqz(),
        "inttest" => commands::kov(),
        "debugnew" => commands::kmw(),
        "nvme" => commands::kqd(),
        "keytest" => commands::kpb(),
        "hexdump" | "xxd" => commands::hlz(args),
        "panic" => commands::kqg(),
        "exit" | "logout" => commands::kph(),
        "reboot" => commands::kqw(),
        "shutdown" | "halt" | "poweroff" => commands::koj(),
        "suspend" | "s3" => commands::fnc(),
        "neofetch" => commands::fmw(),
        "matrix" => commands::kpq(),
        "rain" => {
            
            if args.is_empty() {
                let d = crate::desktop::S.lock();
                let name = match d.matrix_rain_preset { 0 => "slow", 2 => "fast", _ => "mid" };
                drop(d);
                crate::println!("Current rain preset: {}", name);
                crate::println!("Usage: rain <slow|mid|fast>");
            } else {
                let preset: u8 = match args[0] {
                    "slow" | "s" | "0" => 0,
                    "mid" | "m" | "1" | "medium" => 1,
                    "fast" | "f" | "2" => 2,
                    _ => {
                        crate::println!("Unknown preset '{}'. Use: slow, mid, fast", args[0]);
                        return;
                    }
                };
                crate::desktop::S.lock().set_rain_preset(preset);
                let name = match preset { 0 => "slow", 2 => "fast", _ => "mid" };
                crate::println!("Rain speed set to: {}", name);
            }
        },
        "cowsay" => commands::kml(args),

        
        "benchmark" | "bench" => desktop::fmd(args),
        "showcase" => desktop::krm(args),
        "showcase-jarvis" | "jarvis-showcase" | "jdemo" => desktop::krn(args),
        "showcase3d" | "demo3d" => desktop::hme(),
        "demo" | "tutorial" | "tour" => desktop::kmy(args),
        "filled3d" => desktop::hlw(),
        "desktop" | "gui" => desktop::esl(None),
        "mobile" => desktop::mxa(),
        "cosmic" => desktop::hlr(),
        "open" => desktop::kqf(args),
        "trustedit" | "edit3d" | "3dedit" => desktop::esl(Some(("TrustEdit 3D", crate::desktop::WindowType::ModelEditor, 100, 60, 700, 500))),
        "calculator" | "calc" => desktop::esl(Some(("Calculator", crate::desktop::WindowType::Calculator, 300, 200, 320, 420))),
        "snake" => desktop::esl(Some(("Snake", crate::desktop::WindowType::Game, 200, 100, 400, 400))),
        "signature" | "sig" => desktop::kro(args),
        "security" | "sec" | "caps" => desktop::krj(args),

        
        "vm" | "linux" => {
            if args.is_empty() {
                vm::eii();
            } else {
                match args[0] {
                    
                    "create" | "run" | "start" | "guests" | "inspect" | "mount" | "input"
                    | "debug" | "stack" | "regs" | "dump" | "linux" => vm::ktt(args),
                    "status" => vm::koh(),
                    "install" => vm::kog(),
                    "console" | "shell" => vm::eii(),
                    "stop" => vm::ktv(),
                    "list" => vm::ktu(),
                    "extract" => apps::hos(),
                    "exec" => {
                        if args.len() > 1 {
                            let bqr = args[1];
                            let egu: Vec<&str> = args[2..].to_vec();
                            match crate::linux_compat::exec(bqr, &egu) {
                                Ok(code) => crate::println!("[Exited with code {}]", code),
                                Err(e) => crate::n!(0xFF0000, "Error: {}", e),
                            }
                        } else {
                            crate::println!("Usage: linux exec <binary> [args...]");
                            crate::println!("Example: linux exec /bin/busybox ls");
                        }
                    },
                    "help" | "--help" | "-h" => vm::hmh(),
                    _ => vm::hmh(),
                }
            }
        },
        "distro" | "distros" => {
            if args.is_empty() {
                vm::fmk();
            } else {
                match args[0] {
                    "list" => vm::fmk(),
                    "install" | "download" => {
                        if args.len() > 1 { vm::knc(args[1]); }
                        else { vm::hlv(); }
                    },
                    "run" | "start" => {
                        if args.len() > 1 { vm::knd(args[1]); }
                        else { crate::println!("Usage: distro run <id>"); }
                    },
                    "pick" | "select" => vm::hlv(),
                    _ => vm::fmk(),
                }
            }
        },
        "glmode" | "compositor" => vm::koa(args),
        "theme" => vm::kss(args),
        "anim" | "animations" => vm::klp(args),
        "holo" | "holomatrix" => vm::kom(args),
        "imgview" | "imageview" | "view" => vm::kot(args),
        "imgdemo" | "imagedemo" => vm::kos(args),
        "tasks" | "jobs" => vm::ksk(),
        "threads" => vm::kst(),
        "alpine" => vm::klo(args),
        "apt-get" | "apt" | "apk" | "dpkg" => vm::kql(command, args),
        "persist" | "persistence" => vm::kqk(args),
        "disk" => vm::knb(),
        "dd" => vm::kmv(args),
        "ahci" => vm::klm(args),
        "fdisk" | "partitions" => vm::knu(args),
        "lspci" => vm::kpo(args),
        "lshw" | "hwinfo" => vm::kpk(),
        "gpu" => vm::kob(args),
        "gpuexec" | "gpurun" | "gpuagent" => commands::koc(args),
        "sdma" | "dma" => commands::kri(args),
        "neural" | "nn" | "gemm" => commands::kpz(args),
        "gpufw" | "firmware" => commands::kod(args),
        "a11y" | "accessibility" => vm::klk(args),
        "beep" => vm::kma(args),
        "audio" => vm::klt(args),
        "synth" => vm::ksf(args),
        "play" => vm::kqm(args),
        "vizfx" | "liveviz" => vm::kts(args),
        "daw" | "trustdaw" => vm::kmr(args),
        "ifconfig" | "ip" => vm::dkx(),
        "ipconfig" => vm::koy(args),
        "wifi" => commands::kue(args),
        "ping" => vm::fmx(args),
        "tcpsyn" => vm::ksl(args),
        "httpget" => vm::kon(args),
        "curl" | "wget" => vm::hlt(args),
        "download" => vm::knf(args),
        "nslookup" | "dig" => vm::kqc(args),
        "arp" => vm::klr(args),
        "route" => vm::krc(args),
        "traceroute" | "tracert" => vm::ksy(args),
        "netstat" => vm::dky(),
        "exec" | "run" | "./" => vm::knq(args, command),
        "elfinfo" => vm::knp(args),
        "lsusb" => unix::kpp(),
        "checkm8" => {
            let jxm = args.join(" ");
            let result = crate::drivers::checkm8::ojb(&jxm);
            crate::println!("{}", result);
        }
        "lscpu" => unix::hma(),
        "smpstatus" => unix::krq(),
        "smp" => unix::krp(args),
        "fontsmooth" => unix::knx(args),
        "hv" | "hypervisor" => vm::koq(args),

        
        "nmap" | "portscan" | "scan" => vm::kqb(args),
        "discover" | "hostscan" | "arpscan" => vm::hlu(args),
        "banner" | "grabber" => vm::klw(args),
        "sniff" | "capture" | "tcpdump" => vm::krs(args),
        "vulnscan" | "vuln" => vm::ktx(args),
        "scantest" | "netscantest" => vm::kpy(args),

        
        "httpd" | "httpserv" | "webserv" => commands::fmr(args),

        
        "trustpkg" | "pkg" => commands::kte(args),

        
        "browse" | "www" | "web" => network::kme(args),
        "sandbox" | "websandbox" => network::krf(args),
        "container" | "webcontainer" | "wc" => network::kmk(args),

        
        "which" => unix::dkz(args),
        "whereis" => unix::kuc(args),
        "file" => unix::knv(args),
        "basename" => unix::kly(args),
        "dirname" => unix::kna(args),
        "realpath" => unix::kqv(args),
        "sort" => unix::krt(args, amf.as_deref()),
        "uniq" => unix::ktk(args, amf.as_deref()),

        
        "nano" | "vi" | "edit" => editor::kpw(args),

        
        "alias" => unix::kln(args),
        "unalias" => unix::ktj(args),
        "bc" => unix::klz(args),
        "diff" => unix::fmj(args),
        "md5sum" => unix::kpr(args),
        "sha256sum" => unix::krl(args),
        "base64" => unix::klx(args, amf.as_deref()),
        "cut" => unix::kmq(args, amf.as_deref()),
        "tr" => unix::ksx(args, amf.as_deref()),
        "tee" => unix::ksm(args, amf.as_deref()),
        "xargs" => unix::kug(args, amf.as_deref()),
        "chmod" => unix::kmh(args),
        "chown" => unix::kmi(args),
        "ln" => unix::kpf(args),
        "readlink" => unix::kqu(args),
        "watch" => unix::kty(args),
        "timeout" => unix::ksw(args),
        "tar" => unix::ksj(args),
        "gzip" => unix::koi(args),
        "zip" => unix::kui(args),
        "unzip" => unix::ktl(args),
        "service" => unix::fnb(args),
        "systemctl" => unix::ksi(args),
        "crontab" => unix::kmp(args),
        "at" => unix::kls(args),
        "unset" => unix::hmg(args),
        "read" => unix::kqt(args),

        "yes" => unix::kuh(args),
        "seq" => unix::cmd_seq(args),
        "sleep" => unix::fnc(args),
        "kill" => unix::kpc(args),
        "killall" => unix::kpd(args),
        "nice" => unix::kqa(args),

        "top" => unix::eil(),
        "htop" => unix::eil(),
        "vmstat" => unix::ktw(),
        "iostat" => unix::kox(),
        "strace" => unix::ksa(args),
        "dmidecode" => unix::kne(),
        "hdparm" => unix::kok(args),
        "screenshot" | "scrot" => unix::krh(args),
        "httpd" | "serve" => unix::fmr(args),
        "benchmark" | "bench" => unix::fmd(),
        "uptime" => unix::ktn(),

        "lsof" => unix::kpn(args),

        "strings" => unix::ksb(args),

        "mount" => unix::fmu(args),
        "umount" => unix::kti(args),
        "fsck" => unix::kny(args),

        "sync" => unix::kse(),
        "lsblk" => unix::kpj(),
        "blkid" => unix::kmb(),

        "export" => unix::che(args),

        "source" | "." => unix::kru(args),
        "set" => unix::krk(args),

        "printf" => unix::kqq(args),
        "test" | "[" => unix::kso(args),
        "expr" => unix::knr(args),

        "cal" => unix::kmf(args),

        "cmp" => unix::kmj(args),

        "od" => unix::kqe(args),
        "rev" => unix::kra(args),
        "factor" => unix::kns(args),

        "tty" => unix::ktg(),
        "stty" => unix::ksc(args),
        "reset" => unix::kqy(),

        "lsmem" => unix::kpl(),

        "lsmod" => unix::kpm(),

        "sysctl" => unix::ksh(args),
        "firewall" | "iptables" | "fw" => unix::knw(args),
        "du" => unix::kno(args),

        "dmesg" => unix::fml(args),
        "memdbg" | "heapdbg" => unix::kps(),
        "perf" | "perfstat" => unix::kqj(),
        "irqstat" | "irqs" => unix::koz(),
        "regs" | "registers" | "cpuregs" => unix::kqx(),
        "peek" | "memdump" => unix::kqi(args),
        "poke" | "memwrite" => unix::kqn(args),
        "devpanel" => unix::kmz(),
        "timecmd" => unix::ksv(args),

        
        "hwdiag" | "diagnostic" | "diag" => unix::koo(),
        "cpudump" | "fullregs" => unix::kmm(),
        "stacktrace" | "backtrace" | "bt" => unix::krv(args),
        "bootlog" | "checkpoints" => unix::kmc(),
        "postcode" => unix::kqo(args),
        "ioport" => unix::kow(args),
        "rdmsr" => unix::kqs(args),
        "wrmsr" => unix::kuf(args),
        "cpuid" => unix::kmo(args),
        "memmap" => unix::kpt(),
        "watchdog" | "wdt" => unix::ktz(args),
        "drv" | "driver" => unix::kng(args),
        "netconsole" | "nc" => unix::kpx(args),

        
        "fan" => crate::drivers::thinkpad_ec::knt(args),
        "temp" | "sensors" => crate::drivers::thinkpad_ec::ksn(args),
        "cpufreq" | "speedstep" => crate::drivers::thinkpad_ec::kmn(args),

        
        "wayland" | "wl" => apps::kua(args),
        "gterm" | "graphterm" => apps::koe(args),
        "transpile" | "disasm" | "analyze" => apps::ksz(args),
        "rv-xlat" | "rvxlat" | "xlat" => apps::kre(args),
        "rv-disasm" | "rvdisasm" => apps::krd(args),
        "trustview" | "tv" => apps::ktf(args),
        "lab" | "trustlab" => apps::kpe(args),
        "hwscan" | "trustprobe" | "probe" => apps::kop(args),
        "hwdbg" | "hwdebug" => crate::hwdiag::idj(args),
        "marionet" | "mario" => crate::marionet::mhj(args),
        "trustlang" | "tl" => apps::kta(args),
        "trustlang_showcase" | "tl_showcase" => apps::ktb(),
        "film" | "trustos_film" => apps::ktc(),
        "trailer" | "trustos_trailer" => trailer::ktd(),
        "video" => apps::ktr(args),

        
        "jarvis" | "j" | "ai" | "assistant" => jarvis::kpa(args),
        "mesh" | "jarvis-mesh" | "jmesh" => commands::kpv(args),
        "pxe" | "pxeboot" | "replicate" => commands::kqr(args),
        "guardian" | "pact" | "gardien" => commands::kof(args),

        "" => {}
        _ if unix::pod(command) => {}
        _ => {
            
            if vm::pnv(command, args) {
                return;
            }
            crate::bq!(A_, "tsh: ");
            crate::print!("{}", command);
            crate::n!(A_, ": command not found");
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
