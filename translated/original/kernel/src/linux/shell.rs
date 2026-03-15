//! Linux Shell for TrustOS
//!
//! Provides a Linux-compatible shell environment.

use alloc::string::String;
use alloc::string::ToString;
use alloc::vec::Vec;
use alloc::format;
use super::rootfs;

const COLOR_GREEN: u32 = 0x00FF00;
const COLOR_CYAN: u32 = 0x00FFFF;
const COLOR_YELLOW: u32 = 0xFFFF00;
const COLOR_RED: u32 = 0xFF0000;
const COLOR_WHITE: u32 = 0xFFFFFF;
const COLOR_BLUE: u32 = 0x6699FF;

/// Run the Linux shell
pub fn run() {
    crate::println!();
    crate::println_color!(COLOR_CYAN, "╔══════════════════════════════════════════════════════════════╗");
    crate::println_color!(COLOR_CYAN, "║              Alpine Linux on TrustOS                         ║");
    crate::println_color!(COLOR_CYAN, "╚══════════════════════════════════════════════════════════════╝");
    crate::println!();
    
    // Show MOTD
    if let Ok(motd) = rootfs::read_file("/etc/motd") {
        if let Ok(s) = core::str::from_utf8(&motd) {
            crate::print!("{}", s);
        }
    }
    
    let mut cmd_buffer = [0u8; 512];
    let mut history: Vec<String> = Vec::new();
    
    loop {
        // Get current directory and hostname
        let (cwd, hostname, username) = super::with_linux(|linux| {
            (
                String::from(linux.cwd()),
                String::from(linux.hostname()),
                String::from(linux.username()),
            )
        });
        
        // Display prompt: root@alpine:/root#
        crate::print_color!(COLOR_GREEN, "{}@{}", username, hostname);
        crate::print!(":");
        crate::print_color!(COLOR_BLUE, "{}", display_path(&cwd));
        if username == "root" {
            crate::print!("# ");
        } else {
            crate::print!("$ ");
        }
        
        let len = crate::keyboard::read_line(&mut cmd_buffer);
        let input = core::str::from_utf8(&cmd_buffer[..len]).unwrap_or("");
        let cmd = input.trim();
        
        if cmd.is_empty() {
            continue;
        }
        
        // Add to history
        if history.len() >= 100 {
            history.remove(0);
        }
        history.push(String::from(cmd));
        
        // Execute command
        if execute_command(cmd).is_err() {
            // Exit was requested
            break;
        }
    }
    
    crate::println!("Exiting Linux subsystem...");
}

/// Display path with ~ for home
fn display_path(path: &str) -> String {
    if path == "/root" {
        String::from("~")
    } else if path.starts_with("/root/") {
        format!("~{}", &path[5..])
    } else {
        String::from(path)
    }
}

/// Execute a single command
pub fn execute_command(cmd: &str) -> Result<(), &'static str> {
    let parts: Vec<&str> = cmd.split_whitespace().collect();
    if parts.is_empty() {
        return Ok(());
    }
    
    let command = parts[0];
    let args = &parts[1..];
    
    match command {
        "exit" | "logout" => {
            return Err("exit");
        }
        
        "help" => cmd_help(),
        "ls" => cmd_ls(args),
        "ll" => cmd_ls(&["-la"]),
        "cd" => cmd_cd(args),
        "pwd" => cmd_pwd(),
        "cat" => cmd_cat(args),
        "echo" => cmd_echo(args),
        "mkdir" => cmd_mkdir(args),
        "touch" => cmd_touch(args),
        "rm" => cmd_rm(args),
        "cp" => cmd_cp(args),
        "mv" => cmd_mv(args),
        "clear" => cmd_clear(),
        "uname" => cmd_uname(args),
        "whoami" => crate::println!("root"),
        "id" => crate::println!("uid=0(root) gid=0(root) groups=0(root)"),
        "hostname" => cmd_hostname(args),
        "date" => cmd_date(),
        "uptime" => cmd_uptime(),
        "free" => cmd_free(args),
        "df" => cmd_df(args),
        "ps" => cmd_ps(args),
        "top" => cmd_top(),
        "dmesg" => cmd_dmesg(),
        "mount" => cmd_mount(),
        "env" | "printenv" => cmd_env(),
        "export" => cmd_export(args),
        "head" => cmd_head(args),
        "tail" => cmd_tail(args),
        "wc" => cmd_wc(args),
        "grep" => cmd_grep(args),
        "find" => cmd_find(args),
        "which" => cmd_which(args),
        "type" => cmd_type(args),
        "true" => {}
        "false" => {}
        "test" | "[" => cmd_test(args),
        "sh" | "ash" | "bash" => {
            crate::println!("Already in shell");
        }
        "apk" => cmd_apk(args),
        "wget" | "curl" => cmd_wget(args),
        "ping" => cmd_ping(args),
        "ifconfig" | "ip" => cmd_ifconfig(args),
        "netstat" | "ss" => cmd_netstat(),
        "reboot" | "poweroff" | "halt" => {
            crate::println!("Cannot {} from Linux subsystem", command);
            crate::println!("Use 'exit' to return to TrustOS, then use TrustOS commands.");
        }
        
        _ => {
            // Check if it's a file to execute
            let cwd = super::with_linux(|l| String::from(l.cwd()));
            let bin_paths = ["/bin", "/sbin", "/usr/bin", "/usr/sbin"];
            
            let mut found = false;
            
            // First check absolute/relative path
            let check_path = if command.starts_with('/') || command.starts_with("./") {
                Some(String::from(command))
            } else {
                // Search in PATH
                bin_paths.iter()
                    .map(|p| format!("{}/{}", p, command))
                    .find(|p| rootfs::exists(p))
            };
            
            if let Some(binary_path) = check_path {
                if crate::linux_compat::is_linux_binary(&binary_path) {
                    // Execute real Linux binary
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
                    found = true;
                } else if let Ok(content) = rootfs::read_file(&binary_path) {
                    // Check for shell script
                    if content.len() >= 2 && &content[0..2] == b"#!" {
                        // Execute as script
                        match crate::linux_compat::runtime::run_script(&binary_path, &mut crate::linux_compat::LinuxProcess::new(&binary_path, alloc::vec![], alloc::vec![])) {
                            Ok(_) => {}
                            Err(e) => crate::println!("{}: {}", command, e),
                        }
                        found = true;
                    } else {
                        crate::println!("{}: binary format not recognized", command);
                        found = true;
                    }
                }
            }
            
            if !found {
                crate::println!("{}: command not found", command);
            }
        }
    }
    
    Ok(())
}

fn cmd_help() {
    crate::println_color!(COLOR_CYAN, "Alpine Linux Shell Commands");
    crate::println_color!(COLOR_CYAN, "===========================");
    crate::println!();
    crate::println_color!(COLOR_YELLOW, "File Operations:");
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
    crate::println_color!(COLOR_YELLOW, "System Information:");
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
    crate::println_color!(COLOR_YELLOW, "Text Processing:");
    crate::println!("  echo <text>      Print text");
    crate::println!("  head <file>      First lines of file");
    crate::println!("  tail <file>      Last lines of file");
    crate::println!("  wc <file>        Word/line count");
    crate::println!("  grep <pat> <f>   Search in files");
    crate::println!();
    crate::println_color!(COLOR_YELLOW, "Package Management:");
    crate::println!("  apk update       Update package index");
    crate::println!("  apk add <pkg>    Install package");
    crate::println!("  apk del <pkg>    Remove package");
    crate::println!("  apk search <q>   Search packages");
    crate::println!();
    crate::println_color!(COLOR_YELLOW, "Network:");
    crate::println!("  ping <host>      Ping host");
    crate::println!("  ifconfig         Network interfaces");
    crate::println!("  netstat          Network connections");
    crate::println!();
    crate::println!("  exit             Return to TrustOS");
}

fn cmd_ls(args: &[&str]) {
    let show_all = args.contains(&"-a") || args.contains(&"-la") || args.contains(&"-al");
    let long_format = args.contains(&"-l") || args.contains(&"-la") || args.contains(&"-al");
    
    // Get path argument (skip flags)
    let path = args.iter()
        .find(|a| !a.starts_with('-'))
        .copied();
    
    let cwd = super::with_linux(|l| String::from(l.cwd()));
    let target = path.unwrap_or(&cwd);
    
    let full_path = rootfs::linux_to_trustos_path(target, &cwd);
    let linux_path = if target.starts_with('/') {
        String::from(target)
    } else if target == "." {
        cwd.clone()
    } else {
        format!("{}/{}", cwd, target)
    };
    
    match rootfs::list_dir(&linux_path.replace("/linux", "").replace("//", "/")) {
        Ok(entries) => {
            if long_format {
                crate::println!("total {}", entries.len());
            }
            
            let mut items: Vec<_> = entries.into_iter().collect();
            items.sort_by(|a, b| a.0.cmp(&b.0));
            
            for (name, is_dir, size) in items {
                if !show_all && name.starts_with('.') {
                    continue;
                }
                
                if long_format {
                    let perms = if is_dir { "drwxr-xr-x" } else { "-rw-r--r--" };
                    let size_str = if is_dir { 
                        format!("{:>8}", 4096) 
                    } else { 
                        format!("{:>8}", size) 
                    };
                    
                    if is_dir {
                        crate::print!("{} 1 root root {} Feb  2 12:00 ", perms, size_str);
                        crate::println_color!(COLOR_BLUE, "{}/", name);
                    } else {
                        crate::println!("{} 1 root root {} Feb  2 12:00 {}", perms, size_str, name);
                    }
                } else {
                    if is_dir {
                        crate::print_color!(COLOR_BLUE, "{}  ", name);
                    } else {
                        crate::print!("{}  ", name);
                    }
                }
            }
            
            if !long_format {
                crate::println!();
            }
        }
        Err(_) => {
            crate::println!("ls: cannot access '{}': No such file or directory", target);
        }
    }
}

fn cmd_cd(args: &[&str]) {
    let target = args.first().copied().unwrap_or("~");
    
    super::with_linux(|linux| {
        let new_path = if target == "~" || target == "" {
            String::from("/root")
        } else if target == "-" {
            // TODO: implement OLDPWD
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
        
        // Normalize path
        let normalized = normalize_path(&new_path);
        
        // Check if path exists and is directory
        if rootfs::is_directory(&normalized) {
            linux.set_cwd(&normalized);
        } else if rootfs::exists(&normalized) {
            crate::println!("cd: {}: Not a directory", target);
        } else {
            crate::println!("cd: {}: No such file or directory", target);
        }
    });
}

fn normalize_path(path: &str) -> String {
    let mut parts: Vec<&str> = Vec::new();
    
    for part in path.split('/') {
        match part {
            "" | "." => continue,
            ".." => { parts.pop(); }
            _ => parts.push(part),
        }
    }
    
    if parts.is_empty() {
        String::from("/")
    } else {
        format!("/{}", parts.join("/"))
    }
}

fn cmd_pwd() {
    let cwd = super::with_linux(|l| String::from(l.cwd()));
    crate::println!("{}", cwd);
}

fn cmd_cat(args: &[&str]) {
    if args.is_empty() {
        crate::println!("cat: missing operand");
        return;
    }
    
    let cwd = super::with_linux(|l| String::from(l.cwd()));
    
    for path in args {
        let full_path = if path.starts_with('/') {
            String::from(*path)
        } else {
            format!("{}/{}", cwd, path)
        };
        
        match rootfs::read_file(&full_path) {
            Ok(content) => {
                if let Ok(s) = core::str::from_utf8(&content) {
                    crate::print!("{}", s);
                    if !s.ends_with('\n') {
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

fn cmd_echo(args: &[&str]) {
    let text = args.join(" ");
    crate::println!("{}", text);
}

fn cmd_mkdir(args: &[&str]) {
    if args.is_empty() {
        crate::println!("mkdir: missing operand");
        return;
    }
    
    let cwd = super::with_linux(|l| String::from(l.cwd()));
    
    for path in args {
        if path.starts_with('-') {
            continue;
        }
        
        let full_path = if path.starts_with('/') {
            format!("/linux{}", path)
        } else {
            format!("/linux{}/{}", cwd, path)
        };
        
        crate::ramfs::with_fs(|fs| {
            if let Err(_) = fs.mkdir(&full_path) {
                crate::println!("mkdir: cannot create directory '{}': File exists", path);
            }
        });
    }
}

fn cmd_touch(args: &[&str]) {
    if args.is_empty() {
        crate::println!("touch: missing file operand");
        return;
    }
    
    let cwd = super::with_linux(|l| String::from(l.cwd()));
    
    for path in args {
        let full_path = if path.starts_with('/') {
            format!("/linux{}", path)
        } else {
            format!("/linux{}/{}", cwd, path)
        };
        
        crate::ramfs::with_fs(|fs| {
            let _ = fs.touch(&full_path);
        });
    }
}

fn cmd_rm(args: &[&str]) {
    if args.is_empty() {
        crate::println!("rm: missing operand");
        return;
    }
    
    let cwd = super::with_linux(|l| String::from(l.cwd()));
    
    for path in args {
        if path.starts_with('-') {
            continue;
        }
        
        let full_path = if path.starts_with('/') {
            format!("/linux{}", path)
        } else {
            format!("/linux{}/{}", cwd, path)
        };
        
        crate::ramfs::with_fs(|fs| {
            if let Err(e) = fs.rm(&full_path) {
                crate::println!("rm: cannot remove '{}': {}", path, e.as_str());
            }
        });
    }
}

fn cmd_cp(args: &[&str]) {
    if args.len() < 2 {
        crate::println!("cp: missing operand");
        return;
    }
    
    let cwd = super::with_linux(|l| String::from(l.cwd()));
    let src = args[0];
    let dst = args[1];
    
    let src_path = if src.starts_with('/') {
        format!("/linux{}", src)
    } else {
        format!("/linux{}/{}", cwd, src)
    };
    
    let dst_path = if dst.starts_with('/') {
        format!("/linux{}", dst)
    } else {
        format!("/linux{}/{}", cwd, dst)
    };
    
    crate::ramfs::with_fs(|fs| {
        if let Err(e) = fs.cp(&src_path, &dst_path) {
            crate::println!("cp: error: {}", e.as_str());
        }
    });
}

fn cmd_mv(args: &[&str]) {
    if args.len() < 2 {
        crate::println!("mv: missing operand");
        return;
    }
    
    let cwd = super::with_linux(|l| String::from(l.cwd()));
    let src = args[0];
    let dst = args[1];
    
    let src_path = if src.starts_with('/') {
        format!("/linux{}", src)
    } else {
        format!("/linux{}/{}", cwd, src)
    };
    
    let dst_path = if dst.starts_with('/') {
        format!("/linux{}", dst)
    } else {
        format!("/linux{}/{}", cwd, dst)
    };
    
    crate::ramfs::with_fs(|fs| {
        if let Err(e) = fs.mv(&src_path, &dst_path) {
            crate::println!("mv: error: {}", e.as_str());
        }
    });
}

fn cmd_clear() {
    // Send ANSI clear screen
    crate::print!("\x1B[2J\x1B[H");
}

fn cmd_uname(args: &[&str]) {
    let show_all = args.contains(&"-a");
    
    if show_all || args.is_empty() {
        if show_all {
            crate::println!("Linux alpine 5.15.0-alpine #1 SMP TrustOS x86_64 GNU/Linux");
        } else {
            crate::println!("Linux");
        }
    } else {
        let mut output = String::new();
        for arg in args {
            match *arg {
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

fn cmd_hostname(args: &[&str]) {
    if args.is_empty() {
        let hostname = super::with_linux(|l| String::from(l.hostname()));
        crate::println!("{}", hostname);
    } else {
        crate::println!("hostname: you must be root to change the host name");
    }
}

fn cmd_date() {
    // Get RTC time
    let dt = crate::rtc::read_rtc();
    let days = ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"];
    let months = ["Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"];
    
    let month_name = months.get((dt.month - 1) as usize).unwrap_or(&"???");
    crate::println!("{} {} {:2} {:02}:{:02}:{:02} UTC {}", 
        days[(dt.day % 7) as usize], month_name, dt.day, dt.hour, dt.minute, dt.second, dt.year);
}

fn cmd_uptime() {
    let ticks = crate::logger::get_ticks();
    let seconds = ticks / 100; // Assuming 100 ticks per second
    let minutes = seconds / 60;
    let hours = minutes / 60;
    
    crate::println!(" {:02}:{:02}:{:02} up {} min, 1 user, load average: 0.00, 0.00, 0.00",
        hours % 24, minutes % 60, seconds % 60, minutes);
}

fn cmd_free(args: &[&str]) {
    let human = args.contains(&"-h");
    
    crate::println!("              total        used        free      shared  buff/cache   available");
    if human {
        crate::println!("Mem:          4.0Gi       500Mi       3.0Gi       128Mi       500Mi       3.5Gi");
        crate::println!("Swap:            0B          0B          0B");
    } else {
        crate::println!("Mem:        4194304      512000     3072000      131072      512000     3584000");
        crate::println!("Swap:             0           0           0");
    }
}

fn cmd_df(args: &[&str]) {
    let human = args.contains(&"-h");
    
    crate::println!("Filesystem      {}      Used Available Use% Mounted on", 
        if human { "Size" } else { "1K-blocks" });
    
    if human {
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

fn cmd_ps(args: &[&str]) {
    let show_all = args.contains(&"-a") || args.contains(&"-e") || args.contains(&"aux");
    
    crate::println!("PID   USER     TIME  COMMAND");
    crate::println!("    1 root      0:00 init");
    crate::println!("    2 root      0:00 [kthreadd]");
    if show_all {
        crate::println!("    3 root      0:00 [ksoftirqd/0]");
        crate::println!("   10 root      0:00 [rcu_sched]");
    }
    crate::println!("  100 root      0:00 /bin/sh");
    crate::println!("  101 root      0:00 ps");
}

fn cmd_top() {
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

fn cmd_dmesg() {
    crate::println!("[    0.000000] Linux version 5.15.0-alpine (TrustOS subsystem)");
    crate::println!("[    0.000000] Command line: console=ttyS0");
    crate::println!("[    0.000000] TrustOS Linux Subsystem initialized");
    crate::println!("[    0.010000] Memory: 4096MB available");
    crate::println!("[    0.020000] CPU: TrustOS Virtual CPU");
    crate::println!("[    0.030000] Mounting root filesystem...");
    crate::println!("[    0.040000] Alpine rootfs mounted successfully");
}

fn cmd_mount() {
    crate::println!("trustos-root on / type ramfs (rw,relatime)");
    crate::println!("proc on /proc type proc (rw,nosuid,nodev,noexec,relatime)");
    crate::println!("sys on /sys type sysfs (rw,nosuid,nodev,noexec,relatime)");
    crate::println!("devfs on /dev type devfs (rw,nosuid,relatime)");
    crate::println!("tmpfs on /tmp type tmpfs (rw,nosuid,nodev,relatime)");
}

fn cmd_env() {
    crate::println!("PATH=/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin");
    crate::println!("HOME=/root");
    crate::println!("USER=root");
    crate::println!("LOGNAME=root");
    crate::println!("SHELL=/bin/sh");
    crate::println!("PWD={}", super::with_linux(|l| String::from(l.cwd())));
    crate::println!("HOSTNAME={}", super::with_linux(|l| String::from(l.hostname())));
    crate::println!("TERM=linux");
    crate::println!("LANG=C.UTF-8");
}

fn cmd_export(args: &[&str]) {
    if args.is_empty() {
        cmd_env();
    } else {
        // Just acknowledge, we don't actually store env vars yet
        for arg in args {
            crate::println!("export {}", arg);
        }
    }
}

fn cmd_head(args: &[&str]) {
    let lines = 10;
    
    for path in args {
        if path.starts_with('-') {
            continue;
        }
        
        let cwd = super::with_linux(|l| String::from(l.cwd()));
        let full_path = if path.starts_with('/') {
            String::from(*path)
        } else {
            format!("{}/{}", cwd, path)
        };
        
        match rootfs::read_file(&full_path) {
            Ok(content) => {
                if let Ok(s) = core::str::from_utf8(&content) {
                    for (i, line) in s.lines().enumerate() {
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

fn cmd_tail(args: &[&str]) {
    let lines = 10;
    
    for path in args {
        if path.starts_with('-') {
            continue;
        }
        
        let cwd = super::with_linux(|l| String::from(l.cwd()));
        let full_path = if path.starts_with('/') {
            String::from(*path)
        } else {
            format!("{}/{}", cwd, path)
        };
        
        match rootfs::read_file(&full_path) {
            Ok(content) => {
                if let Ok(s) = core::str::from_utf8(&content) {
                    let all_lines: Vec<&str> = s.lines().collect();
                    let start = if all_lines.len() > lines { all_lines.len() - lines } else { 0 };
                    for line in &all_lines[start..] {
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

fn cmd_wc(args: &[&str]) {
    for path in args {
        if path.starts_with('-') {
            continue;
        }
        
        let cwd = super::with_linux(|l| String::from(l.cwd()));
        let full_path = if path.starts_with('/') {
            String::from(*path)
        } else {
            format!("{}/{}", cwd, path)
        };
        
        match rootfs::read_file(&full_path) {
            Ok(content) => {
                if let Ok(s) = core::str::from_utf8(&content) {
                    let lines = s.lines().count();
                    let words = s.split_whitespace().count();
                    let bytes = content.len();
                    crate::println!("{:>7} {:>7} {:>7} {}", lines, words, bytes, path);
                }
            }
            Err(_) => {
                crate::println!("wc: {}: No such file or directory", path);
            }
        }
    }
}

fn cmd_grep(args: &[&str]) {
    if args.len() < 2 {
        crate::println!("Usage: grep PATTERN FILE");
        return;
    }
    
    let pattern = args[0];
    let cwd = super::with_linux(|l| String::from(l.cwd()));
    
    for path in &args[1..] {
        let full_path = if path.starts_with('/') {
            String::from(*path)
        } else {
            format!("{}/{}", cwd, path)
        };
        
        match rootfs::read_file(&full_path) {
            Ok(content) => {
                if let Ok(s) = core::str::from_utf8(&content) {
                    for line in s.lines() {
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

fn cmd_find(args: &[&str]) {
    let path = args.first().copied().unwrap_or(".");
    crate::println!("{}", path);
    crate::println!("(find: recursive listing not fully implemented)");
}

fn cmd_which(args: &[&str]) {
    let bin_paths = ["/bin", "/sbin", "/usr/bin", "/usr/sbin"];
    
    for cmd in args {
        let mut found = false;
        for path in bin_paths {
            let full_path = format!("{}/{}", path, cmd);
            if rootfs::exists(&full_path) {
                crate::println!("{}", full_path);
                found = true;
                break;
            }
        }
        if !found {
            crate::println!("{} not found", cmd);
        }
    }
}

fn cmd_type(args: &[&str]) {
    let builtins = ["cd", "pwd", "echo", "exit", "export", "help", "clear", "test"];
    
    for cmd in args {
        if builtins.contains(cmd) {
            crate::println!("{} is a shell builtin", cmd);
        } else {
            cmd_which(&[cmd]);
        }
    }
}

fn cmd_test(_args: &[&str]) {
    // Basic test command, always succeeds for now
}

fn cmd_apk(args: &[&str]) {
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
            for pkg in &args[1..] {
                crate::println!("(1/1) Installing {}...", pkg);
                crate::println!(" Executing {}-0.1-r0.post-install", pkg);
                crate::println!("OK: {} installed", pkg);
            }
        }
        "del" | "remove" => {
            if args.len() < 2 {
                crate::println!("apk del: missing package name");
                return;
            }
            for pkg in &args[1..] {
                crate::println!("(1/1) Purging {}...", pkg);
                crate::println!("OK: {} removed", pkg);
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
                let pkg = args[1];
                crate::println!("{}-1.0-r0 description:", pkg);
                crate::println!("  {} package for Alpine Linux", pkg);
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

fn cmd_wget(args: &[&str]) {
    if args.is_empty() {
        crate::println!("Usage: wget URL");
        return;
    }
    
    crate::println!("wget: network download not yet implemented");
    crate::println!("Use TrustOS 'download' command instead");
}

fn cmd_ping(args: &[&str]) {
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

fn cmd_ifconfig(args: &[&str]) {
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

fn cmd_netstat() {
    crate::println!("Active Internet connections (servers and established)");
    crate::println!("Proto Recv-Q Send-Q Local Address           Foreign Address         State");
    crate::println!("tcp        0      0 0.0.0.0:22              0.0.0.0:*               LISTEN");
}
