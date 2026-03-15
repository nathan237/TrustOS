



















use alloc::string::String;
use alloc::vec::Vec;
use alloc::format;
use alloc::boxed::Box;
use core::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use spin::Mutex;

use super::{HypervisorError, Result, CpuVendor, avo};
use super::virtio_console::{self, VirtioConsole, ConsolePort};


pub const AEO_: u64 = 0xFFFF_FFFF_FFFF_0001;


pub const US_: usize = 64;


#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LinuxState {
    
    Ma,
    
    Agt,
    
    At,
    
    Rq,
    
    Q,
    
    Ays,
}


#[derive(Debug, Clone)]
pub struct Auv {
    
    pub ohs: Option<String>,
    
    pub oeq: Option<String>,
    
    pub wx: String,
    
    pub afc: usize,
    
    pub gsc: bool,
}

impl Default for Auv {
    fn default() -> Self {
        Self {
            ohs: None,
            oeq: None,
            wx: String::from("console=hvc0 quiet"),
            afc: US_,
            gsc: true,
        }
    }
}


#[derive(Debug, Clone)]
pub struct CommandResult {
    
    pub nz: i32,
    
    pub ejc: String,
    
    pub dwg: String,
    
    pub uk: u64,
}

impl CommandResult {
    pub fn vx(ejc: String) -> Self {
        Self {
            nz: 0,
            ejc,
            dwg: String::new(),
            uk: 0,
        }
    }

    pub fn zt(aj: i32, dwg: String) -> Self {
        Self {
            nz: aj,
            ejc: String::new(),
            dwg,
            uk: 0,
        }
    }
}



#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
pub struct Bli {
    pub boi: u8,
    pub zkj: u16,
    pub prf: u32,
    pub cbf: u16,
    pub zvi: u16,
    pub zkh: u16,
    pub ilz: u16,
    pub eeb: u16,
    pub dh: u32,           
    pub dk: u16,
    pub zio: u32,
    pub zpp: u16,
    pub zah: u16,
    pub pwt: u8,
    pub eet: u8,
    pub znx: u16,
    pub ffh: u32,
    pub ozh: u32,
    pub hwp: u32,
    pub ygu: u32,
    pub ywz: u16,
    pub ypy: u8,
    pub ypx: u8,
    pub nec: u32,
    pub gjy: u32,
    pub hpj: u32,
    pub pbo: u8,
    pub ong: u8,
    pub mrr: u16,
    pub kjm: u32,
    pub ywa: u32,
    pub ywb: u64,
    pub zew: u32,
    pub oux: u32,
    pub dlq: u64,
    pub gpq: u64,
    pub gjx: u32,
    pub yvx: u32,
}


#[repr(C)]
#[derive(Clone)]
pub struct Byq {
    pub zlt: [u8; 64],
    pub yey: [u8; 20],
    pub msr: [u8; 4],
    pub zrc: u64,
    pub zaf: [u8; 16],
    pub qdf: [u8; 16],
    pub ywt: [u8; 16],
    pub ywu: [u8; 16],
    pub zqp: [u8; 16],
    pub zeg: [u8; 16],
    pub ypz: u32,
    pub yqa: u32,
    pub ypv: u32,
    pub ybo: [u8; 116],
    pub yom: [u8; 128],
    pub yon: [u8; 32],
    pub yet: u32,
    pub pgn: u32,
    pub ksf: u8,
    pub yol: u8,
    pub yoi: u8,
    pub zag: u8,
    pub zmc: u8,
    pub ybp: [u8; 2],
    pub zmj: u8,
    pub ybq: [u8; 1],
    pub zj: Bli,
    pub ybr: [u8; 36],
    pub yoj: [u32; 16],
    pub yoa: [Arj; 128],
    pub ybs: [u8; 48],
    pub yok: [u8; 492],
    pub ybt: [u8; 276],
}


#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct Arj {
    pub ag: u64,
    pub aw: u64,
    pub avt: u32,
    pub fzo: u32,
}


pub mod e820_type {
    pub const Dfe: u32 = 1;
    pub const Bqb: u32 = 2;
    pub const Cqz: u32 = 3;
    pub const Dcw: u32 = 4;
    pub const Dkf: u32 = 5;
}


#[derive(Debug, Clone)]
pub struct E {
    pub j: &'static str,
    pub dk: &'static str,
    pub gs: u32,
    pub dc: &'static str,
    pub jw: &'static [&'static str],
}


const PD_: &[E] = &[
    
    E { j: "vim", dk: "9.0.2127-r0", gs: 1824, dc: "Improved vi-style text editor", jw: &["ncurses-libs", "vim-common"] },
    E { j: "vim-common", dk: "9.0.2127-r0", gs: 6240, dc: "Vim common files", jw: &[] },
    E { j: "nano", dk: "7.2-r1", gs: 612, dc: "Nano text editor", jw: &["ncurses-libs"] },
    E { j: "emacs", dk: "29.1-r0", gs: 48000, dc: "GNU Emacs editor", jw: &[] },
    E { j: "micro", dk: "2.0.13-r0", gs: 11264, dc: "Modern terminal text editor", jw: &[] },
    E { j: "helix", dk: "23.10-r0", gs: 24576, dc: "Post-modern modal text editor", jw: &[] },
    
    E { j: "bash", dk: "5.2.21-r0", gs: 1320, dc: "The GNU Bourne Again shell", jw: &["ncurses-libs"] },
    E { j: "zsh", dk: "5.9.0-r0", gs: 3200, dc: "Z shell", jw: &["ncurses-libs"] },
    E { j: "fish", dk: "3.7.0-r0", gs: 6400, dc: "Friendly interactive shell", jw: &["ncurses-libs"] },
    E { j: "dash", dk: "0.5.12-r0", gs: 96, dc: "POSIX compliant shell", jw: &[] },
    
    E { j: "ncurses-libs", dk: "6.4_p20231125-r0", gs: 308, dc: "Ncurses libraries", jw: &[] },
    E { j: "openssl", dk: "3.1.4-r5", gs: 7168, dc: "Toolkit for SSL/TLS", jw: &["ca-certificates"] },
    E { j: "ca-certificates", dk: "20240226-r0", gs: 680, dc: "Common CA certificates PEM files", jw: &[] },
    E { j: "libcurl", dk: "8.5.0-r0", gs: 512, dc: "The multiprotocol file transfer library", jw: &["openssl"] },
    E { j: "libffi", dk: "3.4.4-r3", gs: 52, dc: "Portable foreign function interface library", jw: &[] },
    
    E { j: "curl", dk: "8.5.0-r0", gs: 356, dc: "URL retrieval utility and library", jw: &["ca-certificates", "libcurl"] },
    E { j: "wget", dk: "1.21.4-r0", gs: 480, dc: "Network utility to retrieve files from the Web", jw: &["openssl"] },
    E { j: "git", dk: "2.43.0-r0", gs: 14336, dc: "Distributed version control system", jw: &["openssl", "curl", "perl"] },
    E { j: "openssh", dk: "9.6_p1-r0", gs: 3072, dc: "Port of OpenBSD's free SSH release", jw: &["openssl"] },
    E { j: "nmap", dk: "7.94-r0", gs: 5120, dc: "Network exploration and security scanner", jw: &[] },
    E { j: "tcpdump", dk: "4.99.4-r1", gs: 640, dc: "Network packet analyzer", jw: &[] },
    E { j: "socat", dk: "1.8.0.0-r0", gs: 384, dc: "Multipurpose relay for binary protocols", jw: &[] },
    E { j: "iperf3", dk: "3.16-r0", gs: 192, dc: "Network bandwidth measurement tool", jw: &[] },
    E { j: "bind-tools", dk: "9.18.24-r0", gs: 2048, dc: "ISC BIND DNS tools (dig)", jw: &[] },
    E { j: "mtr", dk: "0.95-r2", gs: 192, dc: "Network diagnostic tool", jw: &[] },
    E { j: "wireguard-tools", dk: "1.0.20210914-r3", gs: 64, dc: "WireGuard VPN tools", jw: &[] },
    E { j: "openvpn", dk: "2.6.8-r0", gs: 1024, dc: "VPN solution", jw: &["openssl"] },
    E { j: "lynx", dk: "2.8.9-r5", gs: 2048, dc: "Text-mode web browser", jw: &[] },
    
    E { j: "python3", dk: "3.11.8-r0", gs: 25600, dc: "High-level scripting language", jw: &["libffi", "openssl"] },
    E { j: "perl", dk: "5.38.2-r0", gs: 16384, dc: "Larry Wall's Practical Extraction and Report Language", jw: &[] },
    E { j: "gcc", dk: "13.2.1_git20231014-r0", gs: 102400, dc: "The GNU Compiler Collection", jw: &["binutils", "musl-dev"] },
    E { j: "rust", dk: "1.75.0-r0", gs: 204800, dc: "The Rust programming language", jw: &["gcc", "musl-dev"] },
    E { j: "nodejs", dk: "20.11.1-r0", gs: 30720, dc: "JavaScript runtime built on V8", jw: &["openssl", "libffi"] },
    E { j: "go", dk: "1.21.6-r0", gs: 143360, dc: "Go programming language", jw: &[] },
    E { j: "ruby", dk: "3.2.3-r0", gs: 12288, dc: "Ruby programming language", jw: &[] },
    E { j: "php83", dk: "8.3.2-r0", gs: 15360, dc: "PHP programming language", jw: &[] },
    E { j: "lua5.4", dk: "5.4.6-r2", gs: 256, dc: "Lua programming language", jw: &[] },
    E { j: "zig", dk: "0.11.0-r0", gs: 51200, dc: "Zig programming language", jw: &[] },
    E { j: "nim", dk: "2.0.2-r0", gs: 10240, dc: "Nim programming language", jw: &[] },
    E { j: "openjdk17-jre", dk: "17.0.10-r0", gs: 204800, dc: "OpenJDK 17 Runtime", jw: &[] },
    E { j: "elixir", dk: "1.16.1-r0", gs: 7680, dc: "Elixir programming language", jw: &[] },
    E { j: "clang", dk: "17.0.5-r0", gs: 81920, dc: "C language family frontend for LLVM", jw: &[] },
    E { j: "cmake", dk: "3.27.8-r0", gs: 9728, dc: "Cross-platform build system", jw: &[] },
    
    E { j: "binutils", dk: "2.41-r0", gs: 8192, dc: "Tools necessary to build programs", jw: &[] },
    E { j: "musl-dev", dk: "1.2.4_git20230717-r4", gs: 1024, dc: "musl C library development files", jw: &[] },
    E { j: "make", dk: "4.4.1-r2", gs: 272, dc: "GNU make utility", jw: &[] },
    E { j: "nasm", dk: "2.16.01-r0", gs: 640, dc: "Netwide Assembler", jw: &[] },
    
    E { j: "gdb", dk: "14.1-r0", gs: 12800, dc: "GNU debugger", jw: &[] },
    E { j: "valgrind", dk: "3.22.0-r0", gs: 22528, dc: "Memory debugging tool", jw: &[] },
    E { j: "strace", dk: "6.7-r0", gs: 580, dc: "System call tracer", jw: &[] },
    E { j: "ltrace", dk: "0.7.3-r8", gs: 384, dc: "Library call tracer", jw: &[] },
    E { j: "lsof", dk: "4.99.3-r0", gs: 320, dc: "List open files", jw: &[] },
    
    E { j: "nginx", dk: "1.24.0-r15", gs: 2048, dc: "HTTP and reverse proxy server", jw: &["openssl"] },
    E { j: "apache2", dk: "2.4.58-r0", gs: 5120, dc: "Apache HTTP Server", jw: &[] },
    E { j: "haproxy", dk: "2.8.5-r0", gs: 3072, dc: "TCP/HTTP Load Balancer", jw: &[] },
    E { j: "dnsmasq", dk: "2.90-r0", gs: 384, dc: "Lightweight DNS/DHCP server", jw: &[] },
    E { j: "squid", dk: "6.6-r0", gs: 7680, dc: "HTTP caching proxy", jw: &[] },
    
    E { j: "redis", dk: "7.2.4-r0", gs: 4096, dc: "In-memory data structure store", jw: &[] },
    E { j: "postgresql16", dk: "16.2-r0", gs: 15360, dc: "PostgreSQL database server", jw: &[] },
    E { j: "mariadb", dk: "10.11.6-r0", gs: 25600, dc: "MariaDB database server", jw: &[] },
    E { j: "sqlite", dk: "3.44.2-r0", gs: 1024, dc: "SQLite database engine", jw: &[] },
    
    E { j: "docker-cli", dk: "24.0.7-r0", gs: 50000, dc: "Docker container runtime", jw: &[] },
    E { j: "podman", dk: "4.8.3-r0", gs: 40960, dc: "Daemonless container engine", jw: &[] },
    E { j: "helm", dk: "3.14.0-r0", gs: 15360, dc: "Kubernetes package manager", jw: &[] },
    E { j: "kubectl", dk: "1.29.1-r0", gs: 20480, dc: "Kubernetes CLI", jw: &[] },
    E { j: "terraform", dk: "1.7.2-r0", gs: 81920, dc: "Infrastructure as code", jw: &[] },
    E { j: "ansible", dk: "9.2.0-r0", gs: 25600, dc: "IT automation tool", jw: &[] },
    
    E { j: "coreutils", dk: "9.4-r1", gs: 6400, dc: "GNU core utilities", jw: &[] },
    E { j: "findutils", dk: "4.9.0-r5", gs: 640, dc: "GNU find utilities", jw: &[] },
    E { j: "grep", dk: "3.11-r0", gs: 320, dc: "GNU grep", jw: &[] },
    E { j: "sed", dk: "4.9-r2", gs: 224, dc: "GNU stream editor", jw: &[] },
    E { j: "gawk", dk: "5.3.0-r0", gs: 1024, dc: "GNU awk", jw: &[] },
    E { j: "diffutils", dk: "3.10-r0", gs: 384, dc: "GNU diff utilities", jw: &[] },
    E { j: "patch", dk: "2.7.6-r10", gs: 128, dc: "GNU patch", jw: &[] },
    E { j: "less", dk: "643-r0", gs: 192, dc: "Pager program", jw: &[] },
    E { j: "file", dk: "5.45-r1", gs: 640, dc: "File type identification", jw: &[] },
    E { j: "iproute2", dk: "6.7.0-r0", gs: 1024, dc: "IP routing utilities", jw: &[] },
    E { j: "util-linux", dk: "2.39.3-r0", gs: 4096, dc: "System utilities", jw: &[] },
    E { j: "procps", dk: "4.0.4-r0", gs: 480, dc: "Process monitoring utilities", jw: &[] },
    E { j: "shadow", dk: "4.14.3-r0", gs: 480, dc: "User/group management", jw: &[] },
    E { j: "e2fsprogs", dk: "1.47.0-r5", gs: 2048, dc: "Ext2/3/4 filesystem utilities", jw: &[] },
    
    E { j: "gzip", dk: "1.13-r0", gs: 96, dc: "GNU zip compression", jw: &[] },
    E { j: "bzip2", dk: "1.0.8-r6", gs: 128, dc: "Block-sorting compressor", jw: &[] },
    E { j: "xz", dk: "5.4.5-r0", gs: 256, dc: "XZ Utils compression", jw: &[] },
    E { j: "zstd", dk: "1.5.5-r8", gs: 384, dc: "Zstandard compression", jw: &[] },
    E { j: "zip", dk: "3.0-r12", gs: 192, dc: "Create ZIP archives", jw: &[] },
    E { j: "unzip", dk: "6.0-r14", gs: 192, dc: "Extract ZIP archives", jw: &[] },
    E { j: "p7zip", dk: "17.05-r0", gs: 2048, dc: "7-Zip file archiver", jw: &[] },
    
    E { j: "ffmpeg", dk: "6.1.1-r0", gs: 20480, dc: "Complete multimedia framework", jw: &[] },
    E { j: "imagemagick", dk: "7.1.1-r0", gs: 15360, dc: "Image manipulation tools", jw: &[] },
    E { j: "mpv", dk: "0.37.0-r0", gs: 5120, dc: "Media player", jw: &[] },
    
    E { j: "ripgrep", dk: "14.1.0-r0", gs: 6144, dc: "Fast recursive grep alternative (rg)", jw: &[] },
    E { j: "fd", dk: "9.0.0-r0", gs: 3072, dc: "Simple fast alternative to find", jw: &[] },
    E { j: "bat", dk: "0.24.0-r0", gs: 5120, dc: "Cat clone with syntax highlighting", jw: &[] },
    E { j: "exa", dk: "0.10.1-r3", gs: 1536, dc: "Modern replacement for ls", jw: &[] },
    E { j: "fzf", dk: "0.44.1-r0", gs: 3072, dc: "Fuzzy finder", jw: &[] },
    E { j: "dust", dk: "0.8.6-r0", gs: 2048, dc: "Intuitive version of du", jw: &[] },
    E { j: "hyperfine", dk: "1.18.0-r0", gs: 2048, dc: "Command-line benchmarking tool", jw: &[] },
    E { j: "tokei", dk: "12.1.2-r4", gs: 3072, dc: "Code statistics tool", jw: &[] },
    
    E { j: "mercurial", dk: "6.6.3-r0", gs: 7680, dc: "Mercurial version control", jw: &[] },
    E { j: "subversion", dk: "1.14.3-r0", gs: 5120, dc: "Subversion version control", jw: &[] },
    E { j: "fossil", dk: "2.23-r0", gs: 3072, dc: "Fossil version control", jw: &[] },
    
    E { j: "htop", dk: "3.3.0-r0", gs: 216, dc: "Interactive process viewer", jw: &["ncurses-libs"] },
    E { j: "neofetch", dk: "7.1.0-r3", gs: 76, dc: "CLI system information tool", jw: &["bash"] },
    E { j: "tree", dk: "2.1.1-r0", gs: 48, dc: "Directory listing in tree-like format", jw: &[] },
    E { j: "jq", dk: "1.7.1-r0", gs: 312, dc: "Lightweight JSON processor", jw: &[] },
    E { j: "tmux", dk: "3.3a-r5", gs: 424, dc: "Terminal multiplexer", jw: &["ncurses-libs"] },
    E { j: "screen", dk: "4.9.1-r0", gs: 640, dc: "Terminal multiplexer", jw: &[] },
    E { j: "bc", dk: "1.07.1-r4", gs: 128, dc: "Calculator language", jw: &[] },
    E { j: "ncdu", dk: "2.3-r0", gs: 192, dc: "NCurses disk usage", jw: &[] },
    E { j: "ranger", dk: "1.9.3-r6", gs: 640, dc: "Console file manager", jw: &[] },
    E { j: "mc", dk: "4.8.31-r0", gs: 3072, dc: "Midnight Commander file manager", jw: &[] },
    E { j: "cowsay", dk: "3.04-r2", gs: 24, dc: "Talking cow", jw: &[] },
    E { j: "figlet", dk: "2.2.5-r3", gs: 128, dc: "Large text banners", jw: &[] },
    E { j: "sl", dk: "5.05-r0", gs: 24, dc: "Steam locomotive", jw: &[] },
    E { j: "fortune", dk: "0.1-r2", gs: 1024, dc: "Fortune cookie program", jw: &[] },
    E { j: "py3-pip", dk: "23.3.2-r0", gs: 5120, dc: "Python package installer", jw: &["python3"] },
    E { j: "certbot", dk: "2.8.0-r0", gs: 3072, dc: "ACME client for Let's Encrypt", jw: &[] },
    E { j: "fail2ban", dk: "1.0.2-r0", gs: 2048, dc: "Intrusion prevention", jw: &[] },
];

fn hjm(j: &str) -> Option<&'static E> {
    PD_.iter().du(|ai| ai.j == j)
}


pub struct LinuxSubsystem {
    
    g: LinuxState,
    
    fk: u64,
    
    gbg: Auv,
    
    console: Option<VirtioConsole>,
    
    vgp: Vec<String>,
    
    oif: Option<CommandResult>,
    
    gbh: u64,
    
    epr: Option<&'static [u8]>,
    
    ggb: Option<&'static [u8]>,
    
    cah: Vec<(&'static str, &'static str)>,   
    
    lzj: bool,
    
    dty: bool,
    
    fqw: Option<String>,
    
    lyb: u32,
    
    iee: usize,
}

impl LinuxSubsystem {
    pub const fn new() -> Self {
        Self {
            g: LinuxState::Ma,
            fk: AEO_,
            gbg: Auv {
                ohs: None,
                oeq: None,
                wx: String::new(),
                afc: US_,
                gsc: true,
            },
            console: None,
            vgp: Vec::new(),
            oif: None,
            gbh: 0,
            epr: None,
            ggb: None,
            cah: Vec::new(),
            lzj: false,
            dty: false,
            fqw: None,
            lyb: 0,
            iee: 0,
        }
    }

    
    pub fn g(&self) -> LinuxState {
        self.g
    }

    
    pub fn uc(&self) -> bool {
        self.g == LinuxState::At
    }

    
    pub fn ogm(&self, j: &str) -> bool {
        self.cah.iter().any(|(bo, _)| *bo == j)
    }

    
    
    fn qzh(&self) -> bool {
        
        if crate::netstack::dhcp::flz() {
            return true;
        }
        
        let tmi = crate::drivers::net::bzy();
        let tnl = crate::virtio_net::ky();
        if !tmi && !tnl {
            return false;
        }
        
        crate::netstack::dhcp::ay();
        
        let ay = crate::logger::lh();
        loop {
            crate::netstack::poll();
            if crate::netstack::dhcp::flz() {
                return true;
            }
            let ez = crate::logger::lh().ao(ay);
            if ez > 3000 {
                return false;
            }
            for _ in 0..10000 { core::hint::hc(); }
        }
    }

    
    fn rwt(&self) -> Option<String> {
        use alloc::vec::Vec;
        let mut hch: Vec<String> = Vec::new();

        
        if let Some((fcc, elo, nt, xyv)) = crate::netstack::dhcp::nxw() {
            if nt != [0, 0, 0, 0] {
                hch.push(alloc::format!("http://{}.{}.{}.{}:8080", nt[0], nt[1], nt[2], nt[3]));
            }
        }
        
        hch.push(String::from("http://10.0.2.2:8080"));
        hch.push(String::from("http://192.168.56.1:8080"));

        
        hch.rux();

        for bog in &hch {
            let url = alloc::format!("{}/repo/index", bog);
            crate::serial_println!("[TSL-PKG] Trying server: {}", url);
            if let Ok(lj) = crate::netstack::http::get(&url) {
                if lj.wt == 200 && lj.gj.len() > 10 {
                    crate::serial_println!("[TSL-PKG] Server found: {} ({} bytes index)",
                        bog, lj.gj.len());
                    return Some(bog.clone());
                }
            }
        }
        None
    }

    
    fn nmp(&self, bog: &str, j: &str) -> Option<Vec<u8>> {
        let url = alloc::format!("{}/repo/pool/{}.pkg", bog, j);
        crate::serial_println!("[TSL-PKG] Downloading: {}", url);
        match crate::netstack::http::get(&url) {
            Ok(lj) if lj.wt == 200 && !lj.gj.is_empty() => {
                crate::serial_println!("[TSL-PKG] Downloaded {} bytes for {}", lj.gj.len(), j);
                Some(lj.gj)
            }
            Ok(lj) => {
                crate::serial_println!("[TSL-PKG] Server returned {} for {}", lj.wt, j);
                None
            }
            Err(aa) => {
                crate::serial_println!("[TSL-PKG] Download failed for {}: {}", j, aa);
                None
            }
        }
    }

    
    
    
    
    
    
    
    fn nsl(&mut self, f: &[u8]) -> u32 {
        let text = match core::str::jg(f) {
            Ok(ab) => ab,
            Err(_) => return 0,
        };

        let mut ium = 0u32;
        let mut rp: Option<&str> = None;
        let mut gdy = String::new();

        for line in text.ak() {
            if line.cj("PKG ") {
                
                continue;
            } else if line.cj("FILE /") {
                
                if let Some(path) = rp {
                    if self.oeu(path, gdy.as_bytes()) {
                        ium += 1;
                    }
                }
                rp = Some(&line[5..]);
                gdy = String::new();
            } else if line == "EOF" {
                
                if let Some(path) = rp {
                    if self.oeu(path, gdy.as_bytes()) {
                        ium += 1;
                    }
                }
                break;
            } else {
                
                if rp.is_some() {
                    if !gdy.is_empty() {
                        gdy.push('\n');
                    }
                    gdy.t(line);
                }
            }
        }

        self.lyb += ium;
        ium
    }

    
    fn oeu(&self, path: &str, ca: &[u8]) -> bool {
        crate::ramfs::fh(|fs| {
            
            let mut te = String::new();
            let ek: Vec<&str> = path.adk('/').hi(|e| !e.is_empty()).collect();
            if ek.len() > 1 {
                for vu in &ek[..ek.len() - 1] {
                    te.push('/');
                    te.t(vu);
                    let _ = fs.ut(&te);
                }
            }
            
            if fs.touch(path).is_ok() {
                if fs.ns(path, ca).is_ok() {
                    crate::serial_println!("[TSL-PKG] Installed: {} ({} bytes)", path, ca.len());
                    return true;
                }
            }
            false
        })
    }

    
    pub fn jbn(&self) -> bool {
        self.dty
    }

    
    pub fn ygy(&self) -> usize {
        self.iee
    }
    
    
    pub fn oaq(&self) -> bool {
        self.epr.is_some()
    }
    
    
    pub fn bvc(&self) -> usize {
        self.epr.map(|eh| eh.len()).unwrap_or(0)
    }
    
    
    pub fn oao(&self) -> bool {
        self.ggb.is_some()
    }
    
    
    pub fn jaa(&self) -> usize {
        self.ggb.map(|a| a.len()).unwrap_or(0)
    }
    
    
    pub fn oht(&self) -> Option<String> {
        let acf = self.epr?;
        if acf.len() < 0x210 {
            return None;
        }
        
        let czd = u16::dj([acf[0x20E], acf[0x20F]]) as usize;
        if czd == 0 || czd + 0x200 >= acf.len() {
            return None;
        }
        
        let igk = czd + 0x200;
        let mut gwb = alloc::vec::Vec::new();
        for a in 0..80 {
            if igk + a >= acf.len() {
                break;
            }
            let r = acf[igk + a];
            if r == 0 {
                break;
            }
            gwb.push(r);
        }
        
        core::str::jg(&gwb).bq().map(|e| String::from(e))
    }
    
    
    pub fn mzu(&self) -> Option<(u8, u8)> {
        let acf = self.epr?;
        if acf.len() < 0x208 {
            return None;
        }
        
        let sj = u32::dj([
            acf[0x202],
            acf[0x203],
            acf[0x204],
            acf[0x205],
        ]);
        
        if sj != 0x53726448 {
            return None;
        }
        
        let dk = u16::dj([acf[0x206], acf[0x207]]);
        Some(((dk >> 8) as u8, (dk & 0xFF) as u8))
    }

    
    pub fn init(&mut self) -> Result<()> {
        if self.g != LinuxState::Ma {
            return Err(HypervisorError::Bxw);
        }

        crate::serial_println!("[TSL] Initializing TrustOS Subsystem for Linux...");
        
        
        self.console = Some(VirtioConsole::new(self.fk));
        
        
        self.gbg.wx = String::from("console=hvc0 quiet init=/init");
        
        self.g = LinuxState::Ma;
        
        crate::serial_println!("[TSL] Subsystem initialized (waiting for kernel)");
        
        Ok(())
    }

    
    pub fn piw(&mut self, acf: &'static [u8], buz: &'static [u8]) {
        self.epr = Some(acf);
        self.ggb = Some(buz);
        
        
        if acf.len() >= 0x210 {
            let sj = u32::dj([
                acf[0x202],
                acf[0x203],
                acf[0x204],
                acf[0x205],
            ]);
            
            if sj == 0x53726448 {  
                let dk = u16::dj([acf[0x206], acf[0x207]]);
                crate::serial_println!("[TSL] Linux kernel: {} bytes, boot protocol v{}.{}", 
                    acf.len(), dk >> 8, dk & 0xFF);
            } else {
                crate::serial_println!("[TSL] Warning: Invalid kernel magic: {:#X}", sj);
            }
        }
        
        crate::serial_println!("[TSL] Initramfs: {} bytes", buz.len());
        crate::serial_println!("[TSL] Total Linux guest size: {} KB", 
            (acf.len() + buz.len()) / 1024);
    }

    
    pub fn boot(&mut self) -> Result<()> {
        if self.g == LinuxState::At || self.g == LinuxState::Agt {
            return Ok(());
        }

        crate::serial_println!("[TSL] Booting Linux VM...");
        self.g = LinuxState::Agt;

        
        let abr = match self.epr {
            Some(eh) => eh,
            None => {
                crate::serial_println!("[TSL] No kernel image available");
                crate::serial_println!("[TSL] Falling back to simulated mode");
                self.g = LinuxState::At;
                return Ok(());
            }
        };

        
        if abr.len() < 0x210 {
            crate::serial_println!("[TSL] Kernel too small ({} bytes)", abr.len());
            self.g = LinuxState::At;
            return Ok(());
        }

        let sj = u32::dj([
            abr[0x202],
            abr[0x203],
            abr[0x204],
            abr[0x205],
        ]);

        if sj != 0x53726448 {  
            crate::serial_println!("[TSL] Invalid kernel magic: {:#X} (expected HdrS)", sj);
            crate::serial_println!("[TSL] Falling back to simulated mode");
            self.g = LinuxState::At;
            return Ok(());
        }

        let dk = u16::dj([abr[0x206], abr[0x207]]);
        crate::serial_println!("[TSL] Linux boot protocol version: {}.{}", 
            dk >> 8, dk & 0xFF);
            
        
        let boi = abr[0x1F1];
        let eet = abr[0x211];
        let czd = u16::dj([abr[0x20E], abr[0x20F]]) as usize;
        
        crate::serial_println!("[TSL] Setup sectors: {}", boi);
        crate::serial_println!("[TSL] Load flags: {:#X}", eet);
        
        
        if czd > 0 && czd + 0x200 < abr.len() {
            let igk = czd + 0x200;
            let mut gwb = alloc::vec::Vec::new();
            for a in 0..64 {
                if igk + a >= abr.len() {
                    break;
                }
                let r = abr[igk + a];
                if r == 0 {
                    break;
                }
                gwb.push(r);
            }
            if !gwb.is_empty() {
                if let Ok(e) = core::str::jg(&gwb) {
                    crate::serial_println!("[TSL] Kernel version: {}", e);
                }
            }
        }
        
        
        if let Some(apw) = self.ggb {
            crate::serial_println!("[TSL] Initramfs: {} bytes ({} KB)", 
                apw.len(), apw.len() / 1024);
        }

        
        if abr.len() > 0x1000 {
            
            match self.qqz() {
                Ok(()) => {
                    crate::serial_println!("[TSL] Linux VM booted successfully!");
                    self.g = LinuxState::At;
                    return Ok(());
                }
                Err(aa) => {
                    crate::serial_println!("[TSL] Real boot failed: {:?}", aa);
                    crate::serial_println!("[TSL] Falling back to simulated mode");
                }
            }
        }

        
        self.g = LinuxState::At;
        crate::serial_println!("[TSL] Linux VM ready (simulated mode)");

        Ok(())
    }
    
    
    fn qqz(&mut self) -> Result<()> {
        let abr = self.epr.ok_or(HypervisorError::Acg)?;
        let izz = self.ggb.ok_or(HypervisorError::Acg)?;
        
        
        let wx = &self.gbg.wx;
        let fk = super::linux_vm::ima(abr, izz, wx)?;
        
        crate::serial_println!("[TSL] Linux VM started with ID: {}", fk);
        
        Ok(())
    }

    
    pub fn bna(&mut self, ro: &str) -> Result<CommandResult> {
        if self.g != LinuxState::At {
            
            if self.g == LinuxState::Ma {
                self.init()?;
            }
            
            
            if self.g != LinuxState::At {
                return self.pkx(ro);
            }
        }

        self.g = LinuxState::Rq;

        
        if let Some(ref mut console) = self.console {
            let kiv = format!("{}\n", ro);
            console.write(kiv.as_bytes());
        }

        
        let result = self.pkx(ro)?;

        self.g = LinuxState::At;
        self.oif = Some(result.clone());

        Ok(result)
    }

    
    fn pkx(&mut self, ro: &str) -> Result<CommandResult> {
        let ek: Vec<&str> = ro.ayt().collect();
        
        if ek.is_empty() {
            return Ok(CommandResult::vx(String::new()));
        }

        let cmd = ek[0];
        let n = &ek[1..];

        let an = match cmd {
            "uname" => {
                if n.contains(&"-a") {
                    String::from("Linux trustos-vm 6.1.0 #1 SMP x86_64 GNU/Linux")
                } else if n.contains(&"-r") {
                    String::from("6.1.0")
                } else {
                    String::from("Linux")
                }
            }
            "echo" => {
                n.rr(" ")
            }
            "whoami" => {
                String::from("root")
            }
            "pwd" => {
                String::from("/")
            }
            "ls" => {
                if n.is_empty() || n.contains(&"/") {
                    String::from("bin  dev  etc  home  init  lib  mnt  proc  root  run  sbin  sys  tmp  usr  var")
                } else if n.contains(&"-la") || n.contains(&"-l") {
                    String::from(
"total 48
drwxr-xr-x   18 root root  4096 Jan 31 12:00 .
drwxr-xr-x   18 root root  4096 Jan 31 12:00 ..
drwxr-xr-x    2 root root  4096 Jan 31 12:00 bin
drwxr-xr-x    5 root root  4096 Jan 31 12:00 dev
drwxr-xr-x   10 root root  4096 Jan 31 12:00 etc
drwxr-xr-x    2 root root  4096 Jan 31 12:00 home
-rwxr-xr-x    1 root root   512 Jan 31 12:00 init
drwxr-xr-x    4 root root  4096 Jan 31 12:00 lib
drwxr-xr-x    2 root root  4096 Jan 31 12:00 mnt
dr-xr-xr-x  100 root root     0 Jan 31 12:00 proc
drwx------    2 root root  4096 Jan 31 12:00 root
drwxr-xr-x    2 root root  4096 Jan 31 12:00 sbin
dr-xr-xr-x   12 root root     0 Jan 31 12:00 sys
drwxrwxrwt    2 root root  4096 Jan 31 12:00 tmp
drwxr-xr-x    8 root root  4096 Jan 31 12:00 usr
drwxr-xr-x    8 root root  4096 Jan 31 12:00 var")
                } else {
                    String::from("bin  dev  etc  home  init  lib  mnt  proc  root  sbin  sys  tmp  usr  var")
                }
            }
            "cat" => {
                if n.contains(&"/etc/os-release") {
                    String::from(
"NAME=\"TrustOS Linux\"
VERSION=\"1.0\"
ID=trustos
PRETTY_NAME=\"TrustOS Linux 1.0\"
HOME_URL=\"https://trustos.local\"")
                } else if n.contains(&"/proc/version") {
                    String::from("Linux version 6.1.0 (gcc version 12.2.0) #1 SMP Jan 31 2026")
                } else if n.contains(&"/proc/cpuinfo") {
                    String::from(
"processor	: 0
vendor_id	: AuthenticAMD
model name	: AMD Ryzen 7 5800X
cpu MHz		: 3800.000
cache size	: 512 KB
flags		: fpu vme de pse tsc msr pae mce cx8 apic sep mtrr pge mca cmov pat pse36 clflush mmx fxsr sse sse2 ht syscall nx mmxext fxsr_opt pdpe1gb rdtscp lm 3dnowext 3dnow pni cx16 sse4_1 sse4_2 popcnt aes xsave avx hypervisor lahf_lm svm")
                } else if n.contains(&"/proc/meminfo") {
                    String::from(
"MemTotal:          65536 kB
MemFree:           32768 kB
MemAvailable:      48000 kB
Buffers:            4096 kB
Cached:            16384 kB")
                } else {
                    return Ok(CommandResult::zt(1, format!("cat: {}: No such file or directory", n.rr(" "))));
                }
            }
            "date" => {
                String::from("Fri Jan 31 12:00:00 UTC 2026")
            }
            "hostname" => {
                String::from("trustos-vm")
            }
            "uptime" => {
                String::from(" 12:00:00 up 0 min,  0 users,  load average: 0.00, 0.00, 0.00")
            }
            "free" => {
                if n.contains(&"-h") {
                    String::from(
"              total        used        free      shared  buff/cache   available
Mem:            64Mi        16Mi        32Mi       0.0Ki        16Mi        48Mi
Swap:            0B          0B          0B")
                } else {
                    String::from(
"              total        used        free      shared  buff/cache   available
Mem:          65536       16384       32768           0       16384       49152
Swap:             0           0           0")
                }
            }
            "df" => {
                if n.contains(&"-h") {
                    String::from(
"Filesystem      Size  Used Avail Use% Mounted on
rootfs           64M   16M   48M  25% /
devtmpfs         32M     0   32M   0% /dev
tmpfs            32M     0   32M   0% /tmp")
                } else {
                    String::from(
"Filesystem     1K-blocks  Used Available Use% Mounted on
rootfs             65536 16384     49152  25% /
devtmpfs           32768     0     32768   0% /dev
tmpfs              32768     0     32768   0% /tmp")
                }
            }
            "id" => {
                String::from("uid=0(root) gid=0(root) groups=0(root)")
            }
            "env" => {
                String::from(
"PATH=/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin
HOME=/root
TERM=linux
SHELL=/bin/sh
USER=root")
            }
            "apt-get" | "apt" => {
                return self.pkw(n);
            }
            "apk" => {
                return self.wor(n);
            }
            "dpkg" => {
                if n.contains(&"-l") || n.contains(&"--list") {
                    let mut bd = String::from("Desired=Unknown/Install/Remove/Purge/Hold\n| Status=Not/Inst/Conf-files/Unpacked/halF-conf/Half-inst/trig-aWait/Trig-pend\n|/ Err?=(none)/Reinst-required (Status,Err: uppercase=bad)\n||/ Name                    Version          Architecture Description\n+++-=======================-================-============-=================================\n");
                    for (j, axh) in &self.cah {
                        bd.t(&format!("ii  {:<24}{:<17}x86_64       {}\n", j, axh,
                            hjm(j).map(|ai| ai.dc).unwrap_or("")));
                    }
                    return Ok(CommandResult::vx(bd));
                }
                return Ok(CommandResult::zt(1, String::from("dpkg: use apt-get to manage packages")));
            }
            "which" => {
                if n.is_empty() {
                    return Ok(CommandResult::zt(1, String::from("which: missing argument")));
                }
                let hao = n[0];
                
                if self.cah.iter().any(|(bo, _)| *bo == hao) {
                    return Ok(CommandResult::vx(format!("/usr/bin/{}", hao)));
                }
                
                let kfn = ["ls", "cat", "echo", "whoami", "pwd", "date", "uname", "sh", "ash"];
                if kfn.contains(&hao) {
                    return Ok(CommandResult::vx(format!("/bin/{}", hao)));
                }
                return Ok(CommandResult::zt(1, format!("{} not found", hao)));
            }
            "help" | "--help" => {
                String::from(
"TrustOS Linux Subsystem - Available Commands:
  uname       - Print system information
  echo        - Display a line of text
  whoami      - Print effective userid
  pwd         - Print working directory
  ls          - List directory contents
  cat         - Concatenate files
  date        - Print current date and time
  hostname    - Print hostname
  uptime      - Show system uptime
  free        - Display memory usage
  df          - Show disk space usage
  id          - Print user identity
  env         - Print environment variables
  apt-get     - Debian/Ubuntu package manager
  apk         - Alpine Linux package manager
  dpkg -l     - List installed packages
  which       - Locate a command
  
Note: Running in simulated mode (no real Linux kernel)")
            }
            _ => {
                return Ok(CommandResult::zt(127, format!("{}: command not found", cmd)));
            }
        };

        Ok(CommandResult::vx(an))
    }

    
    fn pkw(&mut self, n: &[&str]) -> Result<CommandResult> {
        if n.is_empty() {
            return Ok(CommandResult::zt(1, String::from(
                "Usage: apt-get [update|install|remove|list|search|upgrade] [packages...]")));
        }

        let air = n[0];
        let ews = &n[1..];

        match air {
            "update" => {
                let mut bd = String::new();

                
                if self.qzh() {
                    if let Some(bog) = self.rwt() {
                        bd.t(&format!("Connected to package server: {}\n", bog));
                        bd.t(&format!("Get:1 {}/repo/index Packages [online]\n", bog));
                        self.fqw = Some(bog);
                        self.dty = true;
                    } else {
                        bd.t("No package server found, using built-in repository.\n");
                        self.dty = false;
                    }
                } else {
                    self.dty = false;
                }

                if !self.dty {
                    bd.t("Hit:1 http://dl-cdn.alpinelinux.org/alpine/v3.19/main x86_64 Packages\n");
                    bd.t("Hit:2 http://dl-cdn.alpinelinux.org/alpine/v3.19/community x86_64 Packages\n");
                    bd.t("Hit:3 http://security.alpinelinux.org/alpine/v3.19/main x86_64 Packages\n");
                }
                bd.t("Reading package lists... Done\n");
                bd.t("Building dependency tree... Done\n");
                if self.dty {
                    
                    let mut puw = PD_.len();
                    if let Some(ref eiv) = self.fqw {
                        let tsx = alloc::format!("{}/repo", eiv);
                        if let Ok(lj) = crate::netstack::http::get(&tsx) {
                            
                            if let Some(u) = lj.gj.ee(17).qf(|d| d == b"total_available\":") {
                                let kr = &lj.gj[u + 17..];
                                let uwi = kr.iter().qf(|&o| !o.atb() && o != b' ').unwrap_or(kr.len());
                                let ajh = core::str::jg(&kr[..uwi]).unwrap_or("").em();
                                if let Ok(bo) = ajh.parse::<usize>() {
                                    puw = bo;
                                }
                            }
                        }
                    }
                    bd.t(&format!("{} packages available (live).\n", puw));
                } else {
                    bd.t(&format!("{} packages can be upgraded. Run 'apt-get upgrade' to see them.\n",
                        PD_.len().ao(self.cah.len()).v(8)));
                }
                self.lzj = true;
                Ok(CommandResult::vx(bd))
            }

            "install" | "add" => {
                if ews.is_empty() {
                    return Ok(CommandResult::zt(1, String::from(
                        "E: No packages specified for installation.")));
                }

                if !self.lzj {
                    return Ok(CommandResult::zt(1, String::from(
                        "E: The package lists are not up to date. Run 'apt-get update' first.")));
                }

                let mut bd = String::new();
                bd.t("Reading package lists... Done\n");
                bd.t("Building dependency tree... Done\n");

                
                let mut dcu: Vec<&'static E> = Vec::new();
                let mut jhh: Vec<&str> = Vec::new();
                let mut isf: Vec<&str> = Vec::new(); 

                for &j in ews {
                    
                    if j.cj('-') { continue; }
                    if let Some(op) = hjm(j) {
                        
                        for kpd in op.jw {
                            if !self.cah.iter().any(|(bo, _)| bo == kpd)
                                && !dcu.iter().any(|ai| ai.j == *kpd)
                            {
                                if let Some(gem) = hjm(kpd) {
                                    dcu.push(gem);
                                }
                            }
                        }
                        
                        if !self.cah.iter().any(|(bo, _)| *bo == op.j)
                            && !dcu.iter().any(|ai| ai.j == op.j)
                        {
                            dcu.push(op);
                        }
                    } else if self.dty && self.fqw.is_some() {
                        
                        isf.push(j);
                    } else {
                        jhh.push(j);
                    }
                }

                for gns in &jhh {
                    bd.t(&format!("E: Unable to locate package {}\n", gns));
                }

                
                let mut kse = 0u32;
                if !isf.is_empty() {
                    let eiv = self.fqw.clone().age();
                    for &gft in &isf {
                        bd.t(&format!("Resolving {} via package server...\n", gft));
                        if let Some(f) = self.nmp(&eiv, gft) {
                            let gfb = f.len();
                            self.iee += gfb;
                            let sb = self.nsl(&f);
                            if sb > 0 {
                                bd.t(&format!("Get:1 {}/repo/pool/{}.pkg [{} B]\n", eiv, gft, gfb));
                                bd.t(&format!("  -> Downloaded {} bytes, extracted {} files\n", gfb, sb));
                                
                                let xrd = core::str::jg(&f).unwrap_or("")
                                    .ak().next().unwrap_or("")
                                    .eyv(3, ' ').goc(2).unwrap_or("latest");
                                
                                let urd = alloc::boxed::Box::fmu(String::from(gft).lfh());
                                let xrc = alloc::boxed::Box::fmu(String::from(xrd).lfh());
                                self.cah.push((urd, xrc));
                                kse += 1;
                            } else {
                                bd.t(&format!("E: Unable to locate package {}\n", gft));
                            }
                        } else {
                            bd.t(&format!("E: Unable to locate package {}\n", gft));
                        }
                    }
                }

                if dcu.is_empty() && kse == 0 && jhh.is_empty() && isf.is_empty() {
                    bd.t("All requested packages are already installed.\n");
                    return Ok(CommandResult::vx(bd));
                }

                if dcu.is_empty() && kse == 0 {
                    return Ok(CommandResult::zt(1, bd));
                }

                
                let utd: Vec<&str> = dcu.iter().map(|ai| ai.j).collect();
                let aay: u32 = dcu.iter().map(|ai| ai.gs).sum();

                bd.t("The following NEW packages will be installed:\n  ");
                bd.t(&utd.rr(" "));
                bd.t("\n");
                bd.t(&format!("{} newly installed, 0 to remove and 0 not upgraded.\n",
                    dcu.len()));
                bd.t(&format!("Need to get {} kB of archives.\n", aay));
                bd.t(&format!("After this operation, {} kB of additional disk space will be used.\n",
                    aay * 3));

                
                let bog = self.fqw.clone();
                let jbn = self.dty && bog.is_some();

                for op in &dcu {
                    if jbn {
                        let eiv = bog.ahz().unwrap();
                        bd.t(&format!("Get:1 {}/repo/pool/{}.pkg {} [{} kB]\n",
                            eiv, op.j, op.dk, op.gs));

                        
                        if let Some(f) = self.nmp(eiv, op.j) {
                            let gfb = f.len();
                            self.iee += gfb;
                            let sb = self.nsl(&f);
                            bd.t(&format!("  -> Downloaded {} bytes, extracted {} files\n",
                                gfb, sb));
                        } else {
                            bd.t("  -> Download failed, using cached metadata\n");
                        }
                    } else {
                        bd.t(&format!("Get:1 http://dl-cdn.alpinelinux.org/alpine/v3.19/main x86_64 {} {} [{} kB]\n",
                            op.j, op.dk, op.gs));
                    }
                }
                if jbn {
                    bd.t(&format!("Fetched {} bytes from {}\n",
                        self.iee, bog.ahz().unwrap()));
                } else {
                    bd.t(&format!("Fetched {} kB in 0s (internal)\n", aay));
                }

                
                for op in &dcu {
                    bd.t(&format!("Selecting previously unselected package {}.\n", op.j));
                    bd.t(&format!("Preparing to unpack {}_{}_amd64.deb ...\n", op.j, op.dk));
                    bd.t(&format!("Unpacking {} ({}) ...\n", op.j, op.dk));
                }

                
                bd.t("Setting up packages ...\n");
                for op in &dcu {
                    bd.t(&format!("Setting up {} ({}) ...\n", op.j, op.dk));
                    self.cah.push((op.j, op.dk));
                }

                bd.t("Processing triggers for man-db ...\n");
                if jbn {
                    bd.t(&format!("[online] {} files installed to filesystem.\n",
                        self.lyb));
                }

                if !jhh.is_empty() {
                    return Ok(CommandResult { nz: 1, ejc: bd, dwg: String::new(), uk: 0 });
                }
                Ok(CommandResult::vx(bd))
            }

            "remove" | "purge" | "del" => {
                if ews.is_empty() {
                    return Ok(CommandResult::zt(1, String::from(
                        "E: No packages specified for removal.")));
                }

                let mut bd = String::new();
                bd.t("Reading package lists... Done\n");
                bd.t("Building dependency tree... Done\n");

                let mut gqs = 0u32;
                let mut nwc = 0u32;
                for &j in ews {
                    if j.cj('-') { continue; }
                    if let Some(u) = self.cah.iter().qf(|(bo, _)| *bo == j) {
                        let (dkq, von) = self.cah.remove(u);
                        let aw = hjm(dkq).map(|ai| ai.gs).unwrap_or(100);
                        bd.t(&format!("Removing {} ({}) ...\n", dkq, von));
                        nwc += aw * 3;
                        gqs += 1;
                    } else {
                        bd.t(&format!("Package '{}' is not installed, so not removed.\n", j));
                    }
                }

                if gqs > 0 {
                    bd.t(&format!("{} packages removed, {} kB disk space freed.\n",
                        gqs, nwc));
                }
                Ok(CommandResult::vx(bd))
            }

            "list" | "list-installed" => {
                let mut bd = String::new();
                if self.cah.is_empty() {
                    bd.t("No packages installed.\n");
                    bd.t("Use 'apt-get install <package>' to install packages.\n");
                } else {
                    bd.t("Listing installed packages...\n");
                    bd.t(&format!("{:<24} {:<24} {}\n", "Package", "Version", "Description"));
                    bd.t(&format!("{:-<24} {:-<24} {:-<30}\n", "", "", ""));
                    for (j, axh) in &self.cah {
                        let desc = hjm(j).map(|ai| ai.dc).unwrap_or("");
                        bd.t(&format!("{:<24} {:<24} {}\n", j, axh, desc));
                    }
                    bd.t(&format!("\n{} packages installed.\n", self.cah.len()));
                }
                Ok(CommandResult::vx(bd))
            }

            "search" => {
                if ews.is_empty() {
                    return Ok(CommandResult::zt(1, String::from("Usage: apt-get search <keyword>")));
                }
                let ohx = ews[0].aqn();
                
                let mut bd = String::new();
                let mut az = 0;
                
                for op in PD_ {
                    if op.j.contains(ohx.as_str()) || op.dc.aqn().contains(ohx.as_str()) {
                        let adw = if self.cah.iter().any(|(bo, _)| *bo == op.j) {
                            " [installed]"
                        } else {
                            ""
                        };
                        bd.t(&format!("{}/{} {} x86_64{}\n  {}\n\n",
                            op.j, op.dk, op.gs, adw, op.dc));
                        az += 1;
                    }
                }
                
                if self.dty {
                    if let Some(ref eiv) = self.fqw {
                        let wfr = alloc::format!("{}/repo/search?q={}", eiv, ews[0]);
                        if let Ok(lj) = crate::netstack::http::get(&wfr) {
                            if lj.wt == 200 {
                                let text = core::str::jg(&lj.gj).unwrap_or("");
                                for line in text.ak() {
                                    if line.is_empty() || line == "No results" { continue; }
                                    
                                    let ek: Vec<&str> = line.eyv(7, ' ').collect();
                                    if ek.len() >= 7 {
                                        let dkq = ek[0];
                                        
                                        if PD_.iter().any(|ai| ai.j == dkq) { continue; }
                                        let adw = if self.cah.iter().any(|(bo, _)| *bo == dkq) {
                                            " [installed]"
                                        } else {
                                            ""
                                        };
                                        bd.t(&format!("{}/{} {} {}{}\n  {}\n\n",
                                            dkq, ek[1], ek[2], ek[3], adw, ek[6]));
                                        az += 1;
                                    }
                                }
                            }
                        }
                    }
                }
                if az == 0 {
                    bd.t(&format!("No packages found matching '{}'.\n", ews[0]));
                } else {
                    bd.t(&format!("{} packages found.\n", az));
                }
                Ok(CommandResult::vx(bd))
            }

            "upgrade" => {
                let mut bd = String::new();
                bd.t("Reading package lists... Done\n");
                bd.t("Building dependency tree... Done\n");
                bd.t("Calculating upgrade... Done\n");
                bd.t("0 upgraded, 0 newly installed, 0 to remove and 0 not upgraded.\n");
                Ok(CommandResult::vx(bd))
            }

            _ => {
                Ok(CommandResult::zt(1, format!("E: Invalid operation '{}'", air)))
            }
        }
    }

    
    fn wor(&mut self, n: &[&str]) -> Result<CommandResult> {
        if n.is_empty() {
            return Ok(CommandResult::zt(1, String::from(
                "Usage: apk [update|add|del|list|search|info] [packages...]")));
        }

        
        let lki: &[&str] = match n[0] {
            "add" => &["install"],
            "del" => &["remove"],
            "info" => &["search"],
            gq => &[gq],
        };

        let mut nwp = lki.ip();
        nwp.bk(&n[1..]);
        self.pkw(&nwp)
    }

    
    pub fn cbu(&mut self) -> Result<()> {
        if self.g == LinuxState::Ma {
            return Ok(());
        }

        crate::serial_println!("[TSL] Shutting down Linux VM...");
        self.g = LinuxState::Ays;

        
        if let Some(ref mut console) = self.console {
            console.write(b"poweroff\n");
        }

        
        self.console = None;
        self.g = LinuxState::Ma;

        crate::serial_println!("[TSL] Linux VM stopped");

        Ok(())
    }

    
    pub fn console(&mut self) -> Option<&mut VirtioConsole> {
        self.console.as_mut()
    }
}


static IE_: Mutex<LinuxSubsystem> = Mutex::new(LinuxSubsystem::new());


pub fn init() -> Result<()> {
    IE_.lock().init()
}


pub fn boot() -> Result<()> {
    IE_.lock().boot()
}


pub fn bna(ro: &str) -> Result<CommandResult> {
    IE_.lock().bna(ro)
}


pub fn uc() -> bool {
    IE_.lock().uc()
}


pub fn g() -> LinuxState {
    IE_.lock().g()
}


pub fn cbu() -> Result<()> {
    IE_.lock().cbu()
}


pub fn bcu() -> spin::Aki<'static, LinuxSubsystem> {
    IE_.lock()
}
