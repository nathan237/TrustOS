



use alloc::string::String;
use alloc::string::Gd;
use alloc::vec::Vec;
use alloc::format;
use super::rootfs;

const B_: u32 = 0x00FF00;
const C_: u32 = 0x00FFFF;
const D_: u32 = 0xFFFF00;
const A_: u32 = 0xFF0000;
const Q_: u32 = 0xFFFFFF;
const CD_: u32 = 0x6699FF;


pub fn vw() {
    crate::println!();
    crate::h!(C_, "╔══════════════════════════════════════════════════════════════╗");
    crate::h!(C_, "║              Alpine Linux on TrustOS                         ║");
    crate::h!(C_, "╚══════════════════════════════════════════════════════════════╝");
    crate::println!();
    
    
    if let Ok(upr) = rootfs::mq("/etc/motd") {
        if let Ok(e) = core::str::jg(&upr) {
            crate::print!("{}", e);
        }
    }
    
    let mut hdd = [0u8; 512];
    let mut adv: Vec<String> = Vec::new();
    
    loop {
        
        let (jv, ajc, ox) = super::bsz(|linux| {
            (
                String::from(linux.jv()),
                String::from(linux.ajc()),
                String::from(linux.ox()),
            )
        });
        
        
        crate::gr!(B_, "{}@{}", ox, ajc);
        crate::print!(":");
        crate::gr!(CD_, "{}", ryp(&jv));
        if ox == "root" {
            crate::print!("# ");
        } else {
            crate::print!("$ ");
        }
        
        let len = crate::keyboard::cts(&mut hdd);
        let input = core::str::jg(&hdd[..len]).unwrap_or("");
        let cmd = input.em();
        
        if cmd.is_empty() {
            continue;
        }
        
        
        if adv.len() >= 100 {
            adv.remove(0);
        }
        adv.push(String::from(cmd));
        
        
        if azu(cmd).is_err() {
            
            break;
        }
    }
    
    crate::println!("Exiting Linux subsystem...");
}


fn ryp(path: &str) -> String {
    if path == "/root" {
        String::from("~")
    } else if path.cj("/root/") {
        format!("~{}", &path[5..])
    } else {
        String::from(path)
    }
}


pub fn azu(cmd: &str) -> Result<(), &'static str> {
    let ek: Vec<&str> = cmd.ayt().collect();
    if ek.is_empty() {
        return Ok(());
    }
    
    let ro = ek[0];
    let n = &ek[1..];
    
    match ro {
        "exit" | "logout" => {
            return Err("exit");
        }
        
        "help" => kis(),
        "ls" => ioj(n),
        "ll" => ioj(&["-la"]),
        "cd" => kig(n),
        "pwd" => kjb(),
        "cat" => hde(n),
        "echo" => kin(n),
        "mkdir" => iok(n),
        "touch" => kji(n),
        "rm" => kjc(n),
        "cp" => kii(n),
        "mv" => kix(n),
        "clear" => iof(),
        "uname" => iom(n),
        "whoami" => crate::println!("root"),
        "id" => crate::println!("uid=0(root) gid=0(root) groups=0(root)"),
        "hostname" => kit(n),
        "date" => kij(),
        "uptime" => rkb(),
        "free" => ioh(n),
        "df" => kik(n),
        "ps" => kja(n),
        "top" => iol(),
        "dmesg" => kim(),
        "mount" => kiw(),
        "env" | "printenv" => iog(),
        "export" => kio(n),
        "head" => kir(n),
        "tail" => kjg(n),
        "wc" => kjk(n),
        "grep" => kiq(n),
        "find" => kip(n),
        "which" => hdl(n),
        "type" => rjw(n),
        "true" => {}
        "false" => {}
        "test" | "[" => kjh(n),
        "sh" | "ash" | "bash" => {
            crate::println!("Already in shell");
        }
        "apk" => rcg(n),
        "wget" | "curl" => rkq(n),
        "ping" => kiz(n),
        "ifconfig" | "ip" => hdh(n),
        "netstat" | "ss" => hdi(),
        "reboot" | "poweroff" | "halt" => {
            crate::println!("Cannot {} from Linux subsystem", ro);
            crate::println!("Use 'exit' to return to TrustOS, then use TrustOS commands.");
        }
        
        _ => {
            
            let jv = super::bsz(|dm| String::from(dm.jv()));
            let kdg = ["/bin", "/sbin", "/usr/bin", "/usr/sbin"];
            
            let mut aig = false;
            
            
            let qzj = if ro.cj('/') || ro.cj("./") {
                Some(String::from(ro))
            } else {
                
                kdg.iter()
                    .map(|ai| format!("{}/{}", ai, ro))
                    .du(|ai| rootfs::aja(ai))
            };
            
            if let Some(haq) = qzj {
                if crate::linux_compat::tya(&haq) {
                    
                    match crate::linux_compat::exec(&haq, n) {
                        Ok(aj) => {
                            if aj != 0 {
                                crate::serial_println!("[LINUX] {} exited with code {}", ro, aj);
                            }
                        }
                        Err(aa) => {
                            crate::println!("{}: execution failed: {}", ro, aa);
                        }
                    }
                    aig = true;
                } else if let Ok(ca) = rootfs::mq(&haq) {
                    
                    if ca.len() >= 2 && &ca[0..2] == b"#!" {
                        
                        match crate::linux_compat::runtime::wbl(&haq, &mut crate::linux_compat::LinuxProcess::new(&haq, alloc::vec![], alloc::vec![])) {
                            Ok(_) => {}
                            Err(aa) => crate::println!("{}: {}", ro, aa),
                        }
                        aig = true;
                    } else {
                        crate::println!("{}: binary format not recognized", ro);
                        aig = true;
                    }
                }
            }
            
            if !aig {
                crate::println!("{}: command not found", ro);
            }
        }
    }
    
    Ok(())
}

fn kis() {
    crate::h!(C_, "Alpine Linux Shell Commands");
    crate::h!(C_, "===========================");
    crate::println!();
    crate::h!(D_, "File Operations:");
    crate::println!("  ls [-la]         List directory contents");
    crate::println!("  cd <dir>         Change directory");
    crate::println!("  pwd              Print working directory");
    crate::println!("  cat <file>       Display file contents");
    crate::println!("  mkdir <dir>      Create directory");
    crate::println!("  touch <file>     Create empty file");
    crate::println!("  rm <file>        Remove file");
    crate::println!("  cp <src> <dst>   Copy file");
    crate::println!("  mv <src> <dst>   Move/rename file");
    crate::println!();
    crate::h!(D_, "System Information:");
    crate::println!("  uname [-a]       System information");
    crate::println!("  hostname         Show hostname");
    crate::println!("  whoami           Current user");
    crate::println!("  id               User/group IDs");
    crate::println!("  date             Current date/time");
    crate::println!("  uptime           System uptime");
    crate::println!("  free [-h]        Memory usage");
    crate::println!("  df [-h]          Disk usage");
    crate::println!("  ps               Process list");
    crate::println!("  top              Process monitor");
    crate::println!("  dmesg            Kernel messages");
    crate::println!();
    crate::h!(D_, "Text Processing:");
    crate::println!("  echo <text>      Print text");
    crate::println!("  head <file>      First lines of file");
    crate::println!("  tail <file>      Last lines of file");
    crate::println!("  wc <file>        Word/line count");
    crate::println!("  grep <pat> <f>   Search in files");
    crate::println!();
    crate::h!(D_, "Package Management:");
    crate::println!("  apk update       Update package index");
    crate::println!("  apk add <pkg>    Install package");
    crate::println!("  apk del <pkg>    Remove package");
    crate::println!("  apk search <q>   Search packages");
    crate::println!();
    crate::h!(D_, "Network:");
    crate::println!("  ping <host>      Ping host");
    crate::println!("  ifconfig         Network interfaces");
    crate::println!("  netstat          Network connections");
    crate::println!();
    crate::println!("  exit             Return to TrustOS");
}

fn ioj(n: &[&str]) {
    let eym = n.contains(&"-a") || n.contains(&"-la") || n.contains(&"-al");
    let lju = n.contains(&"-l") || n.contains(&"-la") || n.contains(&"-al");
    
    
    let path = n.iter()
        .du(|q| !q.cj('-'))
        .hu();
    
    let jv = super::bsz(|dm| String::from(dm.jv()));
    let cd = path.unwrap_or(&jv);
    
    let wo = rootfs::ufk(cd, &jv);
    let dss = if cd.cj('/') {
        String::from(cd)
    } else if cd == "." {
        jv.clone()
    } else {
        format!("{}/{}", jv, cd)
    };
    
    match rootfs::ojo(&dss.replace("/linux", "").replace("//", "/")) {
        Ok(ch) => {
            if lju {
                crate::println!("total {}", ch.len());
            }
            
            let mut pj: Vec<_> = ch.dse().collect();
            pj.bxe(|q, o| q.0.cmp(&o.0));
            
            for (j, ta, aw) in pj {
                if !eym && j.cj('.') {
                    continue;
                }
                
                if lju {
                    let dao = if ta { "drwxr-xr-x" } else { "-rw-r--r--" };
                    let als = if ta { 
                        format!("{:>8}", 4096) 
                    } else { 
                        format!("{:>8}", aw) 
                    };
                    
                    if ta {
                        crate::print!("{} 1 root root {} Feb  2 12:00 ", dao, als);
                        crate::h!(CD_, "{}/", j);
                    } else {
                        crate::println!("{} 1 root root {} Feb  2 12:00 {}", dao, als, j);
                    }
                } else {
                    if ta {
                        crate::gr!(CD_, "{}  ", j);
                    } else {
                        crate::print!("{}  ", j);
                    }
                }
            }
            
            if !lju {
                crate::println!();
            }
        }
        Err(_) => {
            crate::println!("ls: cannot access '{}': No such file or directory", cd);
        }
    }
}

fn kig(n: &[&str]) {
    let cd = n.fv().hu().unwrap_or("~");
    
    super::bsz(|linux| {
        let dag = if cd == "~" || cd == "" {
            String::from("/root")
        } else if cd == "-" {
            
            String::from(linux.jv())
        } else if cd.cj('/') {
            String::from(cd)
        } else if cd.cj("~/") {
            format!("/root/{}", &cd[2..])
        } else {
            let jv = linux.jv();
            if jv == "/" {
                format!("/{}", cd)
            } else {
                format!("{}/{}", jv, cd)
            }
        };
        
        
        let dto = bro(&dag);
        
        
        if rootfs::cfr(&dto) {
            linux.wiq(&dto);
        } else if rootfs::aja(&dto) {
            crate::println!("cd: {}: Not a directory", cd);
        } else {
            crate::println!("cd: {}: No such file or directory", cd);
        }
    });
}

fn bro(path: &str) -> String {
    let mut ek: Vec<&str> = Vec::new();
    
    for vu in path.adk('/') {
        match vu {
            "" | "." => continue,
            ".." => { ek.pop(); }
            _ => ek.push(vu),
        }
    }
    
    if ek.is_empty() {
        String::from("/")
    } else {
        format!("/{}", ek.rr("/"))
    }
}

fn kjb() {
    let jv = super::bsz(|dm| String::from(dm.jv()));
    crate::println!("{}", jv);
}

fn hde(n: &[&str]) {
    if n.is_empty() {
        crate::println!("cat: missing operand");
        return;
    }
    
    let jv = super::bsz(|dm| String::from(dm.jv()));
    
    for path in n {
        let wo = if path.cj('/') {
            String::from(*path)
        } else {
            format!("{}/{}", jv, path)
        };
        
        match rootfs::mq(&wo) {
            Ok(ca) => {
                if let Ok(e) = core::str::jg(&ca) {
                    crate::print!("{}", e);
                    if !e.pp('\n') {
                        crate::println!();
                    }
                } else {
                    crate::println!("cat: {}: Binary file", path);
                }
            }
            Err(_) => {
                crate::println!("cat: {}: No such file or directory", path);
            }
        }
    }
}

fn kin(n: &[&str]) {
    let text = n.rr(" ");
    crate::println!("{}", text);
}

fn iok(n: &[&str]) {
    if n.is_empty() {
        crate::println!("mkdir: missing operand");
        return;
    }
    
    let jv = super::bsz(|dm| String::from(dm.jv()));
    
    for path in n {
        if path.cj('-') {
            continue;
        }
        
        let wo = if path.cj('/') {
            format!("/linux{}", path)
        } else {
            format!("/linux{}/{}", jv, path)
        };
        
        crate::ramfs::fh(|fs| {
            if let Err(_) = fs.ut(&wo) {
                crate::println!("mkdir: cannot create directory '{}': File exists", path);
            }
        });
    }
}

fn kji(n: &[&str]) {
    if n.is_empty() {
        crate::println!("touch: missing file operand");
        return;
    }
    
    let jv = super::bsz(|dm| String::from(dm.jv()));
    
    for path in n {
        let wo = if path.cj('/') {
            format!("/linux{}", path)
        } else {
            format!("/linux{}/{}", jv, path)
        };
        
        crate::ramfs::fh(|fs| {
            let _ = fs.touch(&wo);
        });
    }
}

fn kjc(n: &[&str]) {
    if n.is_empty() {
        crate::println!("rm: missing operand");
        return;
    }
    
    let jv = super::bsz(|dm| String::from(dm.jv()));
    
    for path in n {
        if path.cj('-') {
            continue;
        }
        
        let wo = if path.cj('/') {
            format!("/linux{}", path)
        } else {
            format!("/linux{}/{}", jv, path)
        };
        
        crate::ramfs::fh(|fs| {
            if let Err(aa) = fs.hb(&wo) {
                crate::println!("rm: cannot remove '{}': {}", path, aa.as_str());
            }
        });
    }
}

fn kii(n: &[&str]) {
    if n.len() < 2 {
        crate::println!("cp: missing operand");
        return;
    }
    
    let jv = super::bsz(|dm| String::from(dm.jv()));
    let cy = n[0];
    let cs = n[1];
    
    let ibl = if cy.cj('/') {
        format!("/linux{}", cy)
    } else {
        format!("/linux{}/{}", jv, cy)
    };
    
    let hhc = if cs.cj('/') {
        format!("/linux{}", cs)
    } else {
        format!("/linux{}/{}", jv, cs)
    };
    
    crate::ramfs::fh(|fs| {
        if let Err(aa) = fs.bza(&ibl, &hhc) {
            crate::println!("cp: error: {}", aa.as_str());
        }
    });
}

fn kix(n: &[&str]) {
    if n.len() < 2 {
        crate::println!("mv: missing operand");
        return;
    }
    
    let jv = super::bsz(|dm| String::from(dm.jv()));
    let cy = n[0];
    let cs = n[1];
    
    let ibl = if cy.cj('/') {
        format!("/linux{}", cy)
    } else {
        format!("/linux{}/{}", jv, cy)
    };
    
    let hhc = if cs.cj('/') {
        format!("/linux{}", cs)
    } else {
        format!("/linux{}/{}", jv, cs)
    };
    
    crate::ramfs::fh(|fs| {
        if let Err(aa) = fs.euz(&ibl, &hhc) {
            crate::println!("mv: error: {}", aa.as_str());
        }
    });
}

fn iof() {
    
    crate::print!("\x1B[2J\x1B[H");
}

fn iom(n: &[&str]) {
    let eym = n.contains(&"-a");
    
    if eym || n.is_empty() {
        if eym {
            crate::println!("Linux alpine 5.15.0-alpine #1 SMP TrustOS x86_64 GNU/Linux");
        } else {
            crate::println!("Linux");
        }
    } else {
        let mut an = String::new();
        for ji in n {
            match *ji {
                "-s" => an.t("Linux "),
                "-n" => an.t("alpine "),
                "-r" => an.t("5.15.0-alpine "),
                "-v" => an.t("#1 SMP TrustOS "),
                "-m" => an.t("x86_64 "),
                "-o" => an.t("GNU/Linux "),
                _ => {}
            }
        }
        crate::println!("{}", an.em());
    }
}

fn kit(n: &[&str]) {
    if n.is_empty() {
        let ajc = super::bsz(|dm| String::from(dm.ajc()));
        crate::println!("{}", ajc);
    } else {
        crate::println!("hostname: you must be root to change the host name");
    }
}

fn kij() {
    
    let os = crate::rtc::cgz();
    let fgl = ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"];
    let upo = ["Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"];
    
    let upn = upo.get((os.caw - 1) as usize).unwrap_or(&"???");
    crate::println!("{} {} {:2} {:02}:{:02}:{:02} UTC {}", 
        fgl[(os.cjw % 7) as usize], upn, os.cjw, os.bek, os.bri, os.chr, os.ccq);
}

fn rkb() {
    let qb = crate::logger::lh();
    let dvm = qb / 100; 
    let lma = dvm / 60;
    let cad = lma / 60;
    
    crate::println!(" {:02}:{:02}:{:02} up {} min, 1 user, load average: 0.00, 0.00, 0.00",
        cad % 24, lma % 60, dvm % 60, lma);
}

fn ioh(n: &[&str]) {
    let iyz = n.contains(&"-h");
    
    crate::println!("              total        used        free      shared  buff/cache   available");
    if iyz {
        crate::println!("Mem:          4.0Gi       500Mi       3.0Gi       128Mi       500Mi       3.5Gi");
        crate::println!("Swap:            0B          0B          0B");
    } else {
        crate::println!("Mem:        4194304      512000     3072000      131072      512000     3584000");
        crate::println!("Swap:             0           0           0");
    }
}

fn kik(n: &[&str]) {
    let iyz = n.contains(&"-h");
    
    crate::println!("Filesystem      {}      Used Available Use% Mounted on", 
        if iyz { "Size" } else { "1K-blocks" });
    
    if iyz {
        crate::println!("trustos-root     32M       8M       24M  25% /");
        crate::println!("tmpfs           128M       0M      128M   0% /tmp");
        crate::println!("devfs             0M       0M        0M   0% /dev");
        crate::println!("procfs            0M       0M        0M   0% /proc");
    } else {
        crate::println!("trustos-root    32768     8192     24576  25% /");
        crate::println!("tmpfs          131072        0    131072   0% /tmp");
        crate::println!("devfs               0        0         0   0% /dev");
        crate::println!("procfs              0        0         0   0% /proc");
    }
}

fn kja(n: &[&str]) {
    let eym = n.contains(&"-a") || n.contains(&"-e") || n.contains(&"aux");
    
    crate::println!("PID   USER     TIME  COMMAND");
    crate::println!("    1 root      0:00 init");
    crate::println!("    2 root      0:00 [kthreadd]");
    if eym {
        crate::println!("    3 root      0:00 [ksoftirqd/0]");
        crate::println!("   10 root      0:00 [rcu_sched]");
    }
    crate::println!("  100 root      0:00 /bin/sh");
    crate::println!("  101 root      0:00 ps");
}

fn iol() {
    crate::println!("top - {:02}:{:02}:{:02} up 0 min,  1 user,  load average: 0.00, 0.00, 0.00",
        12, 0, 0);
    crate::println!("Tasks:   3 total,   1 running,   2 sleeping,   0 stopped,   0 zombie");
    crate::println!("%Cpu(s):  0.0 us,  0.0 sy,  0.0 ni,100.0 id,  0.0 wa,  0.0 hi,  0.0 si");
    crate::println!("MiB Mem :   4096.0 total,   3000.0 free,    500.0 used,    596.0 buff/cache");
    crate::println!("MiB Swap:      0.0 total,      0.0 free,      0.0 used.   3500.0 avail Mem");
    crate::println!();
    crate::println!("  PID USER      PR  NI    VIRT    RES    SHR S  %CPU  %MEM     TIME+ COMMAND");
    crate::println!("    1 root      20   0    1024    512    256 S   0.0   0.0   0:00.01 init");
    crate::println!("  100 root      20   0    2048   1024    512 S   0.0   0.0   0:00.05 sh");
    crate::println!("  101 root      20   0    1536    768    384 R   0.0   0.0   0:00.00 top");
    crate::println!();
    crate::println!("(Press 'q' to quit - not implemented in snapshot mode)");
}

fn kim() {
    crate::println!("[    0.000000] Linux version 5.15.0-alpine (TrustOS subsystem)");
    crate::println!("[    0.000000] Command line: console=ttyS0");
    crate::println!("[    0.000000] TrustOS Linux Subsystem initialized");
    crate::println!("[    0.010000] Memory: 4096MB available");
    crate::println!("[    0.020000] CPU: TrustOS Virtual CPU");
    crate::println!("[    0.030000] Mounting root filesystem...");
    crate::println!("[    0.040000] Alpine rootfs mounted successfully");
}

fn kiw() {
    crate::println!("trustos-root on / type ramfs (rw,relatime)");
    crate::println!("proc on /proc type proc (rw,nosuid,nodev,noexec,relatime)");
    crate::println!("sys on /sys type sysfs (rw,nosuid,nodev,noexec,relatime)");
    crate::println!("devfs on /dev type devfs (rw,nosuid,relatime)");
    crate::println!("tmpfs on /tmp type tmpfs (rw,nosuid,nodev,relatime)");
}

fn iog() {
    crate::println!("PATH=/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin");
    crate::println!("HOME=/root");
    crate::println!("USER=root");
    crate::println!("LOGNAME=root");
    crate::println!("SHELL=/bin/sh");
    crate::println!("PWD={}", super::bsz(|dm| String::from(dm.jv())));
    crate::println!("HOSTNAME={}", super::bsz(|dm| String::from(dm.ajc())));
    crate::println!("TERM=linux");
    crate::println!("LANG=C.UTF-8");
}

fn kio(n: &[&str]) {
    if n.is_empty() {
        iog();
    } else {
        
        for ji in n {
            crate::println!("export {}", ji);
        }
    }
}

fn kir(n: &[&str]) {
    let ak = 10;
    
    for path in n {
        if path.cj('-') {
            continue;
        }
        
        let jv = super::bsz(|dm| String::from(dm.jv()));
        let wo = if path.cj('/') {
            String::from(*path)
        } else {
            format!("{}/{}", jv, path)
        };
        
        match rootfs::mq(&wo) {
            Ok(ca) => {
                if let Ok(e) = core::str::jg(&ca) {
                    for (a, line) in e.ak().cf() {
                        if a >= ak {
                            break;
                        }
                        crate::println!("{}", line);
                    }
                }
            }
            Err(_) => {
                crate::println!("head: {}: No such file or directory", path);
            }
        }
    }
}

fn kjg(n: &[&str]) {
    let ak = 10;
    
    for path in n {
        if path.cj('-') {
            continue;
        }
        
        let jv = super::bsz(|dm| String::from(dm.jv()));
        let wo = if path.cj('/') {
            String::from(*path)
        } else {
            format!("{}/{}", jv, path)
        };
        
        match rootfs::mq(&wo) {
            Ok(ca) => {
                if let Ok(e) = core::str::jg(&ca) {
                    let jzw: Vec<&str> = e.ak().collect();
                    let ay = if jzw.len() > ak { jzw.len() - ak } else { 0 };
                    for line in &jzw[ay..] {
                        crate::println!("{}", line);
                    }
                }
            }
            Err(_) => {
                crate::println!("tail: {}: No such file or directory", path);
            }
        }
    }
}

fn kjk(n: &[&str]) {
    for path in n {
        if path.cj('-') {
            continue;
        }
        
        let jv = super::bsz(|dm| String::from(dm.jv()));
        let wo = if path.cj('/') {
            String::from(*path)
        } else {
            format!("{}/{}", jv, path)
        };
        
        match rootfs::mq(&wo) {
            Ok(ca) => {
                if let Ok(e) = core::str::jg(&ca) {
                    let ak = e.ak().az();
                    let aoh = e.ayt().az();
                    let bf = ca.len();
                    crate::println!("{:>7} {:>7} {:>7} {}", ak, aoh, bf, path);
                }
            }
            Err(_) => {
                crate::println!("wc: {}: No such file or directory", path);
            }
        }
    }
}

fn kiq(n: &[&str]) {
    if n.len() < 2 {
        crate::println!("Usage: grep PATTERN FILE");
        return;
    }
    
    let pattern = n[0];
    let jv = super::bsz(|dm| String::from(dm.jv()));
    
    for path in &n[1..] {
        let wo = if path.cj('/') {
            String::from(*path)
        } else {
            format!("{}/{}", jv, path)
        };
        
        match rootfs::mq(&wo) {
            Ok(ca) => {
                if let Ok(e) = core::str::jg(&ca) {
                    for line in e.ak() {
                        if line.contains(pattern) {
                            crate::println!("{}", line);
                        }
                    }
                }
            }
            Err(_) => {
                crate::println!("grep: {}: No such file or directory", path);
            }
        }
    }
}

fn kip(n: &[&str]) {
    let path = n.fv().hu().unwrap_or(".");
    crate::println!("{}", path);
    crate::println!("(find: recursive listing not fully implemented)");
}

fn hdl(n: &[&str]) {
    let kdg = ["/bin", "/sbin", "/usr/bin", "/usr/sbin"];
    
    for cmd in n {
        let mut aig = false;
        for path in kdg {
            let wo = format!("{}/{}", path, cmd);
            if rootfs::aja(&wo) {
                crate::println!("{}", wo);
                aig = true;
                break;
            }
        }
        if !aig {
            crate::println!("{} not found", cmd);
        }
    }
}

fn rjw(n: &[&str]) {
    let kfn = ["cd", "pwd", "echo", "exit", "export", "help", "clear", "test"];
    
    for cmd in n {
        if kfn.contains(cmd) {
            crate::println!("{} is a shell builtin", cmd);
        } else {
            hdl(&[cmd]);
        }
    }
}

fn kjh(elm: &[&str]) {
    
}

fn rcg(n: &[&str]) {
    if n.is_empty() {
        crate::println!("apk-tools 2.14.0, compiled for x86_64.");
        crate::println!();
        crate::println!("usage: apk [<OPTIONS>...] COMMAND [<ARGUMENTS>...]");
        crate::println!();
        crate::println!("Commands: add, del, update, upgrade, search, info");
        return;
    }
    
    match n[0] {
        "update" => {
            crate::println!("fetch https://dl-cdn.alpinelinux.org/alpine/v3.19/main/x86_64/APKINDEX.tar.gz");
            crate::println!("fetch https://dl-cdn.alpinelinux.org/alpine/v3.19/community/x86_64/APKINDEX.tar.gz");
            crate::println!("v3.19.0-r0 [/etc/apk/cache]");
            crate::println!("OK: 23456 distinct packages available");
        }
        "add" => {
            if n.len() < 2 {
                crate::println!("apk add: missing package name");
                return;
            }
            for op in &n[1..] {
                crate::println!("(1/1) Installing {}...", op);
                crate::println!(" Executing {}-0.1-r0.post-install", op);
                crate::println!("OK: {} installed", op);
            }
        }
        "del" | "remove" => {
            if n.len() < 2 {
                crate::println!("apk del: missing package name");
                return;
            }
            for op in &n[1..] {
                crate::println!("(1/1) Purging {}...", op);
                crate::println!("OK: {} removed", op);
            }
        }
        "search" => {
            let query = n.get(1).hu().unwrap_or("");
            crate::println!("Searching for '{}'...", query);
            crate::println!("{}-1.0-r0", query);
            crate::println!("{}-dev-1.0-r0", query);
            crate::println!("{}-doc-1.0-r0", query);
        }
        "info" => {
            if n.len() < 2 {
                crate::println!("Installed packages:");
                crate::println!("alpine-base-3.19.0-r0");
                crate::println!("busybox-1.36.0-r0");
                crate::println!("musl-1.2.4-r0");
            } else {
                let op = n[1];
                crate::println!("{}-1.0-r0 description:", op);
                crate::println!("  {} package for Alpine Linux", op);
            }
        }
        "upgrade" => {
            crate::println!("Checking for updates...");
            crate::println!("OK: 0 packages upgraded");
        }
        _ => {
            crate::println!("apk: unknown command '{}'", n[0]);
        }
    }
}

fn rkq(n: &[&str]) {
    if n.is_empty() {
        crate::println!("Usage: wget URL");
        return;
    }
    
    crate::println!("wget: network download not yet implemented");
    crate::println!("Use TrustOS 'download' command instead");
}

fn kiz(n: &[&str]) {
    if n.is_empty() {
        crate::println!("Usage: ping HOST");
        return;
    }
    
    let kh = n[0];
    crate::println!("PING {} ({}) 56(84) bytes of data.", kh, kh);
    
    for a in 1..=4 {
        crate::println!("64 bytes from {}: icmp_seq={} ttl=64 time=0.1 ms", kh, a);
    }
    
    crate::println!();
    crate::println!("--- {} ping statistics ---", kh);
    crate::println!("4 packets transmitted, 4 received, 0% packet loss, time 3000ms");
    crate::println!("rtt min/avg/max/mdev = 0.100/0.100/0.100/0.000 ms");
}

fn hdh(n: &[&str]) {
    crate::println!("eth0: flags=4163<UP,BROADCAST,RUNNING,MULTICAST>  mtu 1500");
    crate::println!("        inet 192.168.56.100  netmask 255.255.255.0  broadcast 192.168.56.255");
    crate::println!("        inet6 fe80::1  prefixlen 64  scopeid 0x20<link>");
    crate::println!("        ether 08:00:27:fb:be:aa  txqueuelen 1000  (Ethernet)");
    crate::println!("        RX packets 1000  bytes 100000 (97.6 KiB)");
    crate::println!("        TX packets 1000  bytes 100000 (97.6 KiB)");
    crate::println!();
    crate::println!("lo: flags=73<UP,LOOPBACK,RUNNING>  mtu 65536");
    crate::println!("        inet 127.0.0.1  netmask 255.0.0.0");
    crate::println!("        inet6 ::1  prefixlen 128  scopeid 0x10<host>");
    crate::println!("        loop  txqueuelen 1000  (Local Loopback)");
}

fn hdi() {
    crate::println!("Active Internet connections (servers and established)");
    crate::println!("Proto Recv-Q Send-Q Local Address           Foreign Address         State");
    crate::println!("tcp        0      0 0.0.0.0:22              0.0.0.0:*               LISTEN");
}
