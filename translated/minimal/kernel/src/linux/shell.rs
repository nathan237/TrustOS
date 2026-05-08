



use alloc::string::String;
use alloc::string::ToString;
use alloc::vec::Vec;
use alloc::format;
use super::rootfs;

const B_: u32 = 0x00FF00;
const C_: u32 = 0x00FFFF;
const D_: u32 = 0xFFFF00;
const A_: u32 = 0xFF0000;
const R_: u32 = 0xFFFFFF;
const CF_: u32 = 0x6699FF;


pub fn run() {
    crate::println!();
    crate::n!(C_, "╔══════════════════════════════════════════════════════════════╗");
    crate::n!(C_, "║              Alpine Linux on TrustOS                         ║");
    crate::n!(C_, "╚══════════════════════════════════════════════════════════════╝");
    crate::println!();
    
    
    if let Ok(motd) = rootfs::read_file("/etc/motd") {
        if let Ok(j) = core::str::from_utf8(&motd) {
            crate::print!("{}", j);
        }
    }
    
    let mut dkv = [0u8; 512];
    let mut history: Vec<String> = Vec::new();
    
    loop {
        
        let (cwd, hostname, username) = super::akr(|linux| {
            (
                String::from(linux.cwd()),
                String::from(linux.hostname()),
                String::from(linux.username()),
            )
        });
        
        
        crate::bq!(B_, "{}@{}", username, hostname);
        crate::print!(":");
        crate::bq!(CF_, "{}", lfm(&cwd));
        if username == "root" {
            crate::print!("# ");
        } else {
            crate::print!("$ ");
        }
        
        let len = crate::keyboard::read_line(&mut dkv);
        let input = core::str::from_utf8(&dkv[..len]).unwrap_or("");
        let cmd = input.trim();
        
        if cmd.is_empty() {
            continue;
        }
        
        
        if history.len() >= 100 {
            history.remove(0);
        }
        history.push(String::from(cmd));
        
        
        if aav(cmd).is_err() {
            
            break;
        }
    }
    
    crate::println!("Exiting Linux subsystem...");
}


fn lfm(path: &str) -> String {
    if path == "/root" {
        String::from("~")
    } else if path.starts_with("/root/") {
        format!("~{}", &path[5..])
    } else {
        String::from(path)
    }
}


pub fn aav(cmd: &str) -> Result<(), &'static str> {
    let au: Vec<&str> = cmd.split_whitespace().collect();
    if au.is_empty() {
        return Ok(());
    }
    
    let command = au[0];
    let args = &au[1..];
    
    match command {
        "exit" | "logout" => {
            return Err("exit");
        }
        
        "help" => chf(),
        "ls" => eij(args),
        "ll" => eij(&["-la"]),
        "cd" => fme(args),
        "pwd" => fmz(),
        "cat" => dkw(args),
        "echo" => fmm(args),
        "mkdir" => eik(args),
        "touch" => fng(args),
        "rm" => fna(args),
        "cp" => fmg(args),
        "mv" => fmv(args),
        "clear" => eif(),
        "uname" => eim(args),
        "whoami" => crate::println!("root"),
        "id" => crate::println!("uid=0(root) gid=0(root) groups=0(root)"),
        "hostname" => fmq(args),
        "date" => fmh(),
        "uptime" => ktm(),
        "free" => eih(args),
        "df" => fmi(args),
        "ps" => fmy(args),
        "top" => eil(),
        "dmesg" => fml(),
        "mount" => fmu(),
        "env" | "printenv" => eig(),
        "export" => che(args),
        "head" => fmp(args),
        "tail" => fne(args),
        "wc" => fni(args),
        "grep" => fmo(args),
        "find" => fmn(args),
        "which" => dkz(args),
        "type" => kth(args),
        "true" => {}
        "false" => {}
        "test" | "[" => fnf(args),
        "sh" | "ash" | "bash" => {
            crate::println!("Already in shell");
        }
        "apk" => klq(args),
        "wget" | "curl" => kub(args),
        "ping" => fmx(args),
        "ifconfig" | "ip" => dkx(args),
        "netstat" | "ss" => dky(),
        "reboot" | "poweroff" | "halt" => {
            crate::println!("Cannot {} from Linux subsystem", command);
            crate::println!("Use 'exit' to return to TrustOS, then use TrustOS commands.");
        }
        
        _ => {
            
            let cwd = super::akr(|l| String::from(l.cwd()));
            let fjc = ["/bin", "/sbin", "/usr/bin", "/usr/sbin"];
            
            let mut nj = false;
            
            
            let kjk = if command.starts_with('/') || command.starts_with("./") {
                Some(String::from(command))
            } else {
                
                fjc.iter()
                    .map(|aa| format!("{}/{}", aa, command))
                    .find(|aa| rootfs::exists(aa))
            };
            
            if let Some(binary_path) = kjk {
                if crate::linux_compat::msy(&binary_path) {
                    
                    match crate::linux_compat::exec(&binary_path, args) {
                        Ok(code) => {
                            if code != 0 {
                                crate::serial_println!("[LINUX] {} exited with code {}", command, code);
                            }
                        }
                        Err(e) => {
                            crate::println!("{}: execution failed: {}", command, e);
                        }
                    }
                    nj = true;
                } else if let Ok(content) = rootfs::read_file(&binary_path) {
                    
                    if content.len() >= 2 && &content[0..2] == b"#!" {
                        
                        match crate::linux_compat::runtime::ojf(&binary_path, &mut crate::linux_compat::LinuxProcess::new(&binary_path, alloc::vec![], alloc::vec![])) {
                            Ok(_) => {}
                            Err(e) => crate::println!("{}: {}", command, e),
                        }
                        nj = true;
                    } else {
                        crate::println!("{}: binary format not recognized", command);
                        nj = true;
                    }
                }
            }
            
            if !nj {
                crate::println!("{}: command not found", command);
            }
        }
    }
    
    Ok(())
}

fn chf() {
    crate::n!(C_, "Alpine Linux Shell Commands");
    crate::n!(C_, "===========================");
    crate::println!();
    crate::n!(D_, "File Operations:");
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
    crate::n!(D_, "System Information:");
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
    crate::n!(D_, "Text Processing:");
    crate::println!("  echo <text>      Print text");
    crate::println!("  head <file>      First lines of file");
    crate::println!("  tail <file>      Last lines of file");
    crate::println!("  wc <file>        Word/line count");
    crate::println!("  grep <pat> <f>   Search in files");
    crate::println!();
    crate::n!(D_, "Package Management:");
    crate::println!("  apk update       Update package index");
    crate::println!("  apk add <pkg>    Install package");
    crate::println!("  apk del <pkg>    Remove package");
    crate::println!("  apk search <q>   Search packages");
    crate::println!();
    crate::n!(D_, "Network:");
    crate::println!("  ping <host>      Ping host");
    crate::println!("  ifconfig         Network interfaces");
    crate::println!("  netstat          Network connections");
    crate::println!();
    crate::println!("  exit             Return to TrustOS");
}

fn eij(args: &[&str]) {
    let cdr = args.contains(&"-a") || args.contains(&"-la") || args.contains(&"-al");
    let ggg = args.contains(&"-l") || args.contains(&"-la") || args.contains(&"-al");
    
    
    let path = args.iter()
        .find(|a| !a.starts_with('-'))
        .copied();
    
    let cwd = super::akr(|l| String::from(l.cwd()));
    let target = path.unwrap_or(&cwd);
    
    let kg = rootfs::myx(target, &cwd);
    let bni = if target.starts_with('/') {
        String::from(target)
    } else if target == "." {
        cwd.clone()
    } else {
        format!("{}/{}", cwd, target)
    };
    
    match rootfs::ikp(&bni.replace("/linux", "").replace("//", "/")) {
        Ok(entries) => {
            if ggg {
                crate::println!("total {}", entries.len());
            }
            
            let mut items: Vec<_> = entries.into_iter().collect();
            items.sort_by(|a, b| a.0.cmp(&b.0));
            
            for (name, is_dir, size) in items {
                if !cdr && name.starts_with('.') {
                    continue;
                }
                
                if ggg {
                    let bda = if is_dir { "drwxr-xr-x" } else { "-rw-r--r--" };
                    let td = if is_dir { 
                        format!("{:>8}", 4096) 
                    } else { 
                        format!("{:>8}", size) 
                    };
                    
                    if is_dir {
                        crate::print!("{} 1 root root {} Feb  2 12:00 ", bda, td);
                        crate::n!(CF_, "{}/", name);
                    } else {
                        crate::println!("{} 1 root root {} Feb  2 12:00 {}", bda, td, name);
                    }
                } else {
                    if is_dir {
                        crate::bq!(CF_, "{}  ", name);
                    } else {
                        crate::print!("{}  ", name);
                    }
                }
            }
            
            if !ggg {
                crate::println!();
            }
        }
        Err(_) => {
            crate::println!("ls: cannot access '{}': No such file or directory", target);
        }
    }
}

fn fme(args: &[&str]) {
    let target = args.first().copied().unwrap_or("~");
    
    super::akr(|linux| {
        let bcx = if target == "~" || target == "" {
            String::from("/root")
        } else if target == "-" {
            
            String::from(linux.cwd())
        } else if target.starts_with('/') {
            String::from(target)
        } else if target.starts_with("~/") {
            format!("/root/{}", &target[2..])
        } else {
            let cwd = linux.cwd();
            if cwd == "/" {
                format!("/{}", target)
            } else {
                format!("{}/{}", cwd, target)
            }
        };
        
        
        let bnu = normalize_path(&bcx);
        
        
        if rootfs::is_directory(&bnu) {
            linux.set_cwd(&bnu);
        } else if rootfs::exists(&bnu) {
            crate::println!("cd: {}: Not a directory", target);
        } else {
            crate::println!("cd: {}: No such file or directory", target);
        }
    });
}

fn normalize_path(path: &str) -> String {
    let mut au: Vec<&str> = Vec::new();
    
    for jn in path.split('/') {
        match jn {
            "" | "." => continue,
            ".." => { au.pop(); }
            _ => au.push(jn),
        }
    }
    
    if au.is_empty() {
        String::from("/")
    } else {
        format!("/{}", au.join("/"))
    }
}

fn fmz() {
    let cwd = super::akr(|l| String::from(l.cwd()));
    crate::println!("{}", cwd);
}

fn dkw(args: &[&str]) {
    if args.is_empty() {
        crate::println!("cat: missing operand");
        return;
    }
    
    let cwd = super::akr(|l| String::from(l.cwd()));
    
    for path in args {
        let kg = if path.starts_with('/') {
            String::from(*path)
        } else {
            format!("{}/{}", cwd, path)
        };
        
        match rootfs::read_file(&kg) {
            Ok(content) => {
                if let Ok(j) = core::str::from_utf8(&content) {
                    crate::print!("{}", j);
                    if !j.ends_with('\n') {
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

fn fmm(args: &[&str]) {
    let text = args.join(" ");
    crate::println!("{}", text);
}

fn eik(args: &[&str]) {
    if args.is_empty() {
        crate::println!("mkdir: missing operand");
        return;
    }
    
    let cwd = super::akr(|l| String::from(l.cwd()));
    
    for path in args {
        if path.starts_with('-') {
            continue;
        }
        
        let kg = if path.starts_with('/') {
            format!("/linux{}", path)
        } else {
            format!("/linux{}/{}", cwd, path)
        };
        
        crate::ramfs::bh(|fs| {
            if let Err(_) = fs.mkdir(&kg) {
                crate::println!("mkdir: cannot create directory '{}': File exists", path);
            }
        });
    }
}

fn fng(args: &[&str]) {
    if args.is_empty() {
        crate::println!("touch: missing file operand");
        return;
    }
    
    let cwd = super::akr(|l| String::from(l.cwd()));
    
    for path in args {
        let kg = if path.starts_with('/') {
            format!("/linux{}", path)
        } else {
            format!("/linux{}/{}", cwd, path)
        };
        
        crate::ramfs::bh(|fs| {
            let _ = fs.touch(&kg);
        });
    }
}

fn fna(args: &[&str]) {
    if args.is_empty() {
        crate::println!("rm: missing operand");
        return;
    }
    
    let cwd = super::akr(|l| String::from(l.cwd()));
    
    for path in args {
        if path.starts_with('-') {
            continue;
        }
        
        let kg = if path.starts_with('/') {
            format!("/linux{}", path)
        } else {
            format!("/linux{}/{}", cwd, path)
        };
        
        crate::ramfs::bh(|fs| {
            if let Err(e) = fs.rm(&kg) {
                crate::println!("rm: cannot remove '{}': {}", path, e.as_str());
            }
        });
    }
}

fn fmg(args: &[&str]) {
    if args.len() < 2 {
        crate::println!("cp: missing operand");
        return;
    }
    
    let cwd = super::akr(|l| String::from(l.cwd()));
    let src = args[0];
    let dst = args[1];
    
    let eag = if src.starts_with('/') {
        format!("/linux{}", src)
    } else {
        format!("/linux{}/{}", cwd, src)
    };
    
    let dnv = if dst.starts_with('/') {
        format!("/linux{}", dst)
    } else {
        format!("/linux{}/{}", cwd, dst)
    };
    
    crate::ramfs::bh(|fs| {
        if let Err(e) = fs.cp(&eag, &dnv) {
            crate::println!("cp: error: {}", e.as_str());
        }
    });
}

fn fmv(args: &[&str]) {
    if args.len() < 2 {
        crate::println!("mv: missing operand");
        return;
    }
    
    let cwd = super::akr(|l| String::from(l.cwd()));
    let src = args[0];
    let dst = args[1];
    
    let eag = if src.starts_with('/') {
        format!("/linux{}", src)
    } else {
        format!("/linux{}/{}", cwd, src)
    };
    
    let dnv = if dst.starts_with('/') {
        format!("/linux{}", dst)
    } else {
        format!("/linux{}/{}", cwd, dst)
    };
    
    crate::ramfs::bh(|fs| {
        if let Err(e) = fs.mv(&eag, &dnv) {
            crate::println!("mv: error: {}", e.as_str());
        }
    });
}

fn eif() {
    
    crate::print!("\x1B[2J\x1B[H");
}

fn eim(args: &[&str]) {
    let cdr = args.contains(&"-a");
    
    if cdr || args.is_empty() {
        if cdr {
            crate::println!("Linux alpine 5.15.0-alpine #1 SMP TrustOS x86_64 GNU/Linux");
        } else {
            crate::println!("Linux");
        }
    } else {
        let mut output = String::new();
        for db in args {
            match *db {
                "-s" => output.push_str("Linux "),
                "-n" => output.push_str("alpine "),
                "-r" => output.push_str("5.15.0-alpine "),
                "-v" => output.push_str("#1 SMP TrustOS "),
                "-m" => output.push_str("x86_64 "),
                "-o" => output.push_str("GNU/Linux "),
                _ => {}
            }
        }
        crate::println!("{}", output.trim());
    }
}

fn fmq(args: &[&str]) {
    if args.is_empty() {
        let hostname = super::akr(|l| String::from(l.hostname()));
        crate::println!("{}", hostname);
    } else {
        crate::println!("hostname: you must be root to change the host name");
    }
}

fn fmh() {
    
    let fm = crate::rtc::aou();
    let cic = ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"];
    let nge = ["Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"];
    
    let ngd = nge.get((fm.month - 1) as usize).unwrap_or(&"???");
    crate::println!("{} {} {:2} {:02}:{:02}:{:02} UTC {}", 
        cic[(fm.day % 7) as usize], ngd, fm.day, fm.hour, fm.minute, fm.second, fm.year);
}

fn ktm() {
    let gx = crate::logger::eg();
    let abi = gx / 100; 
    let ght = abi / 60;
    let aoi = ght / 60;
    
    crate::println!(" {:02}:{:02}:{:02} up {} min, 1 user, load average: 0.00, 0.00, 0.00",
        aoi % 24, ght % 60, abi % 60, ght);
}

fn eih(args: &[&str]) {
    let epv = args.contains(&"-h");
    
    crate::println!("              total        used        free      shared  buff/cache   available");
    if epv {
        crate::println!("Mem:          4.0Gi       500Mi       3.0Gi       128Mi       500Mi       3.5Gi");
        crate::println!("Swap:            0B          0B          0B");
    } else {
        crate::println!("Mem:        4194304      512000     3072000      131072      512000     3584000");
        crate::println!("Swap:             0           0           0");
    }
}

fn fmi(args: &[&str]) {
    let epv = args.contains(&"-h");
    
    crate::println!("Filesystem      {}      Used Available Use% Mounted on", 
        if epv { "Size" } else { "1K-blocks" });
    
    if epv {
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

fn fmy(args: &[&str]) {
    let cdr = args.contains(&"-a") || args.contains(&"-e") || args.contains(&"aux");
    
    crate::println!("PID   USER     TIME  COMMAND");
    crate::println!("    1 root      0:00 init");
    crate::println!("    2 root      0:00 [kthreadd]");
    if cdr {
        crate::println!("    3 root      0:00 [ksoftirqd/0]");
        crate::println!("   10 root      0:00 [rcu_sched]");
    }
    crate::println!("  100 root      0:00 /bin/sh");
    crate::println!("  101 root      0:00 ps");
}

fn eil() {
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

fn fml() {
    crate::println!("[    0.000000] Linux version 5.15.0-alpine (TrustOS subsystem)");
    crate::println!("[    0.000000] Command line: console=ttyS0");
    crate::println!("[    0.000000] TrustOS Linux Subsystem initialized");
    crate::println!("[    0.010000] Memory: 4096MB available");
    crate::println!("[    0.020000] CPU: TrustOS Virtual CPU");
    crate::println!("[    0.030000] Mounting root filesystem...");
    crate::println!("[    0.040000] Alpine rootfs mounted successfully");
}

fn fmu() {
    crate::println!("trustos-root on / type ramfs (rw,relatime)");
    crate::println!("proc on /proc type proc (rw,nosuid,nodev,noexec,relatime)");
    crate::println!("sys on /sys type sysfs (rw,nosuid,nodev,noexec,relatime)");
    crate::println!("devfs on /dev type devfs (rw,nosuid,relatime)");
    crate::println!("tmpfs on /tmp type tmpfs (rw,nosuid,nodev,relatime)");
}

fn eig() {
    crate::println!("PATH=/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin");
    crate::println!("HOME=/root");
    crate::println!("USER=root");
    crate::println!("LOGNAME=root");
    crate::println!("SHELL=/bin/sh");
    crate::println!("PWD={}", super::akr(|l| String::from(l.cwd())));
    crate::println!("HOSTNAME={}", super::akr(|l| String::from(l.hostname())));
    crate::println!("TERM=linux");
    crate::println!("LANG=C.UTF-8");
}

fn che(args: &[&str]) {
    if args.is_empty() {
        eig();
    } else {
        
        for db in args {
            crate::println!("export {}", db);
        }
    }
}

fn fmp(args: &[&str]) {
    let lines = 10;
    
    for path in args {
        if path.starts_with('-') {
            continue;
        }
        
        let cwd = super::akr(|l| String::from(l.cwd()));
        let kg = if path.starts_with('/') {
            String::from(*path)
        } else {
            format!("{}/{}", cwd, path)
        };
        
        match rootfs::read_file(&kg) {
            Ok(content) => {
                if let Ok(j) = core::str::from_utf8(&content) {
                    for (i, line) in j.lines().enumerate() {
                        if i >= lines {
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

fn fne(args: &[&str]) {
    let lines = 10;
    
    for path in args {
        if path.starts_with('-') {
            continue;
        }
        
        let cwd = super::akr(|l| String::from(l.cwd()));
        let kg = if path.starts_with('/') {
            String::from(*path)
        } else {
            format!("{}/{}", cwd, path)
        };
        
        match rootfs::read_file(&kg) {
            Ok(content) => {
                if let Ok(j) = core::str::from_utf8(&content) {
                    let fgr: Vec<&str> = j.lines().collect();
                    let start = if fgr.len() > lines { fgr.len() - lines } else { 0 };
                    for line in &fgr[start..] {
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

fn fni(args: &[&str]) {
    for path in args {
        if path.starts_with('-') {
            continue;
        }
        
        let cwd = super::akr(|l| String::from(l.cwd()));
        let kg = if path.starts_with('/') {
            String::from(*path)
        } else {
            format!("{}/{}", cwd, path)
        };
        
        match rootfs::read_file(&kg) {
            Ok(content) => {
                if let Ok(j) = core::str::from_utf8(&content) {
                    let lines = j.lines().count();
                    let um = j.split_whitespace().count();
                    let bytes = content.len();
                    crate::println!("{:>7} {:>7} {:>7} {}", lines, um, bytes, path);
                }
            }
            Err(_) => {
                crate::println!("wc: {}: No such file or directory", path);
            }
        }
    }
}

fn fmo(args: &[&str]) {
    if args.len() < 2 {
        crate::println!("Usage: grep PATTERN FILE");
        return;
    }
    
    let pattern = args[0];
    let cwd = super::akr(|l| String::from(l.cwd()));
    
    for path in &args[1..] {
        let kg = if path.starts_with('/') {
            String::from(*path)
        } else {
            format!("{}/{}", cwd, path)
        };
        
        match rootfs::read_file(&kg) {
            Ok(content) => {
                if let Ok(j) = core::str::from_utf8(&content) {
                    for line in j.lines() {
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

fn fmn(args: &[&str]) {
    let path = args.first().copied().unwrap_or(".");
    crate::println!("{}", path);
    crate::println!("(find: recursive listing not fully implemented)");
}

fn dkz(args: &[&str]) {
    let fjc = ["/bin", "/sbin", "/usr/bin", "/usr/sbin"];
    
    for cmd in args {
        let mut nj = false;
        for path in fjc {
            let kg = format!("{}/{}", path, cmd);
            if rootfs::exists(&kg) {
                crate::println!("{}", kg);
                nj = true;
                break;
            }
        }
        if !nj {
            crate::println!("{} not found", cmd);
        }
    }
}

fn kth(args: &[&str]) {
    let fke = ["cd", "pwd", "echo", "exit", "export", "help", "clear", "test"];
    
    for cmd in args {
        if fke.contains(cmd) {
            crate::println!("{} is a shell builtin", cmd);
        } else {
            dkz(&[cmd]);
        }
    }
}

fn fnf(_args: &[&str]) {
    
}

fn klq(args: &[&str]) {
    if args.is_empty() {
        crate::println!("apk-tools 2.14.0, compiled for x86_64.");
        crate::println!();
        crate::println!("usage: apk [<OPTIONS>...] COMMAND [<ARGUMENTS>...]");
        crate::println!();
        crate::println!("Commands: add, del, update, upgrade, search, info");
        return;
    }
    
    match args[0] {
        "update" => {
            crate::println!("fetch https://dl-cdn.alpinelinux.org/alpine/v3.19/main/x86_64/APKINDEX.tar.gz");
            crate::println!("fetch https://dl-cdn.alpinelinux.org/alpine/v3.19/community/x86_64/APKINDEX.tar.gz");
            crate::println!("v3.19.0-r0 [/etc/apk/cache]");
            crate::println!("OK: 23456 distinct packages available");
        }
        "add" => {
            if args.len() < 2 {
                crate::println!("apk add: missing package name");
                return;
            }
            for gh in &args[1..] {
                crate::println!("(1/1) Installing {}...", gh);
                crate::println!(" Executing {}-0.1-r0.post-install", gh);
                crate::println!("OK: {} installed", gh);
            }
        }
        "del" | "remove" => {
            if args.len() < 2 {
                crate::println!("apk del: missing package name");
                return;
            }
            for gh in &args[1..] {
                crate::println!("(1/1) Purging {}...", gh);
                crate::println!("OK: {} removed", gh);
            }
        }
        "search" => {
            let query = args.get(1).copied().unwrap_or("");
            crate::println!("Searching for '{}'...", query);
            crate::println!("{}-1.0-r0", query);
            crate::println!("{}-dev-1.0-r0", query);
            crate::println!("{}-doc-1.0-r0", query);
        }
        "info" => {
            if args.len() < 2 {
                crate::println!("Installed packages:");
                crate::println!("alpine-base-3.19.0-r0");
                crate::println!("busybox-1.36.0-r0");
                crate::println!("musl-1.2.4-r0");
            } else {
                let gh = args[1];
                crate::println!("{}-1.0-r0 description:", gh);
                crate::println!("  {} package for Alpine Linux", gh);
            }
        }
        "upgrade" => {
            crate::println!("Checking for updates...");
            crate::println!("OK: 0 packages upgraded");
        }
        _ => {
            crate::println!("apk: unknown command '{}'", args[0]);
        }
    }
}

fn kub(args: &[&str]) {
    if args.is_empty() {
        crate::println!("Usage: wget URL");
        return;
    }
    
    crate::println!("wget: network download not yet implemented");
    crate::println!("Use TrustOS 'download' command instead");
}

fn fmx(args: &[&str]) {
    if args.is_empty() {
        crate::println!("Usage: ping HOST");
        return;
    }
    
    let host = args[0];
    crate::println!("PING {} ({}) 56(84) bytes of data.", host, host);
    
    for i in 1..=4 {
        crate::println!("64 bytes from {}: icmp_seq={} ttl=64 time=0.1 ms", host, i);
    }
    
    crate::println!();
    crate::println!("--- {} ping statistics ---", host);
    crate::println!("4 packets transmitted, 4 received, 0% packet loss, time 3000ms");
    crate::println!("rtt min/avg/max/mdev = 0.100/0.100/0.100/0.000 ms");
}

fn dkx(args: &[&str]) {
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

fn dky() {
    crate::println!("Active Internet connections (servers and established)");
    crate::println!("Proto Recv-Q Send-Q Local Address           Foreign Address         State");
    crate::println!("tcp        0      0 0.0.0.0:22              0.0.0.0:*               LISTEN");
}
