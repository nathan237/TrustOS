



















use alloc::string::String;
use alloc::vec::Vec;
use alloc::format;
use alloc::boxed::Box;
use core::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use spin::Mutex;

use super::{HypervisorError, Result, CpuVendor, cpu_vendor};
use super::virtio_console::{self, VirtioConsole, ConsolePort};


pub const AGI_: u64 = 0xFFFF_FFFF_FFFF_0001;


pub const WB_: usize = 64;


#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LinuxState {
    
    NotStarted,
    
    Booting,
    
    Ready,
    
    Busy,
    
    Error,
    
    ShuttingDown,
}


#[derive(Debug, Clone)]
pub struct Tk {
    
    pub kernel_path: Option<String>,
    
    pub initramfs_path: Option<String>,
    
    pub cmdline: String,
    
    pub memory_mb: usize,
    
    pub serial_console: bool,
}

impl Default for Tk {
    fn default() -> Self {
        Self {
            kernel_path: None,
            initramfs_path: None,
            cmdline: String::from("console=hvc0 quiet"),
            memory_mb: WB_,
            serial_console: true,
        }
    }
}


#[derive(Debug, Clone)]
pub struct CommandResult {
    
    pub exit_code: i32,
    
    pub stdout: String,
    
    pub stderr: String,
    
    pub duration_ms: u64,
}

impl CommandResult {
    pub fn success(stdout: String) -> Self {
        Self {
            exit_code: 0,
            stdout,
            stderr: String::new(),
            duration_ms: 0,
        }
    }

    pub fn error(code: i32, stderr: String) -> Self {
        Self {
            exit_code: code,
            stdout: String::new(),
            stderr,
            duration_ms: 0,
        }
    }
}



#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
pub struct Aau {
    pub setup_sects: u8,
    pub root_flags: u16,
    pub syssize: u32,
    pub ram_size: u16,
    pub vid_mode: u16,
    pub root_dev: u16,
    pub boot_flag: u16,
    pub jump: u16,
    pub header: u32,           
    pub version: u16,
    pub realmode_swtch: u32,
    pub start_sys_seg: u16,
    pub kernel_version: u16,
    pub type_of_loader: u8,
    pub btz: u8,
    pub setup_move_size: u16,
    pub code32_start: u32,
    pub ramdisk_image: u32,
    pub ramdisk_size: u32,
    pub bootsect_kludge: u32,
    pub heap_end_ptr: u16,
    pub ext_loader_ver: u8,
    pub ext_loader_type: u8,
    pub cmd_line_ptr: u32,
    pub initrd_addr_max: u32,
    pub kernel_alignment: u32,
    pub relocatable_kernel: u8,
    pub min_alignment: u8,
    pub xloadflags: u16,
    pub fnk: u32,
    pub hardware_subarch: u32,
    pub hardware_subarch_data: u64,
    pub payload_offset: u32,
    pub payload_length: u32,
    pub setup_data: u64,
    pub pref_address: u64,
    pub init_size: u32,
    pub handover_offset: u32,
}


#[repr(C)]
#[derive(Clone)]
pub struct Ahn {
    pub screen_info: [u8; 64],
    pub apm_bios_info: [u8; 20],
    pub _pad1: [u8; 4],
    pub tboot_addr: u64,
    pub ist_info: [u8; 16],
    pub _pad2: [u8; 16],
    pub hd0_info: [u8; 16],
    pub hd1_info: [u8; 16],
    pub sys_desc_table: [u8; 16],
    pub olpc_ofw_header: [u8; 16],
    pub ext_ramdisk_image: u32,
    pub ext_ramdisk_size: u32,
    pub ext_cmd_line_ptr: u32,
    pub _pad3: [u8; 116],
    pub edid_info: [u8; 128],
    pub efi_info: [u8; 32],
    pub alt_mem_k: u32,
    pub scratch: u32,
    pub ftr: u8,
    pub eddbuf_entries: u8,
    pub edd_mbr_sig_buf_entries: u8,
    pub kbd_status: u8,
    pub secure_boot: u8,
    pub _pad4: [u8; 2],
    pub sentinel: u8,
    pub _pad5: [u8; 1],
    pub kp: Aau,
    pub _pad6: [u8; 36],
    pub edd_mbr_sig_buffer: [u32; 16],
    pub e820_table: [Ru; 128],
    pub _pad7: [u8; 48],
    pub eddbuf: [u8; 492],
    pub _pad8: [u8; 276],
}


#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct Ru {
    pub addr: u64,
    pub size: u64,
    pub entry_type: u32,
    pub _pad: u32,
}


pub mod e820_type {
    pub const Bax: u32 = 1;
    pub const Ada: u32 = 2;
    pub const Ask: u32 = 3;
    pub const Azt: u32 = 4;
    pub const Bej: u32 = 5;
}


#[derive(Debug, Clone)]
pub struct J {
    pub name: &'static str,
    pub version: &'static str,
    pub size_kb: u32,
    pub description: &'static str,
    pub deps: &'static [&'static str],
}


const QA_: &[J] = &[
    
    J { name: "vim", version: "9.0.2127-r0", size_kb: 1824, description: "Improved vi-style text editor", deps: &["ncurses-libs", "vim-common"] },
    J { name: "vim-common", version: "9.0.2127-r0", size_kb: 6240, description: "Vim common files", deps: &[] },
    J { name: "nano", version: "7.2-r1", size_kb: 612, description: "Nano text editor", deps: &["ncurses-libs"] },
    J { name: "emacs", version: "29.1-r0", size_kb: 48000, description: "GNU Emacs editor", deps: &[] },
    J { name: "micro", version: "2.0.13-r0", size_kb: 11264, description: "Modern terminal text editor", deps: &[] },
    J { name: "helix", version: "23.10-r0", size_kb: 24576, description: "Post-modern modal text editor", deps: &[] },
    
    J { name: "bash", version: "5.2.21-r0", size_kb: 1320, description: "The GNU Bourne Again shell", deps: &["ncurses-libs"] },
    J { name: "zsh", version: "5.9.0-r0", size_kb: 3200, description: "Z shell", deps: &["ncurses-libs"] },
    J { name: "fish", version: "3.7.0-r0", size_kb: 6400, description: "Friendly interactive shell", deps: &["ncurses-libs"] },
    J { name: "dash", version: "0.5.12-r0", size_kb: 96, description: "POSIX compliant shell", deps: &[] },
    
    J { name: "ncurses-libs", version: "6.4_p20231125-r0", size_kb: 308, description: "Ncurses libraries", deps: &[] },
    J { name: "openssl", version: "3.1.4-r5", size_kb: 7168, description: "Toolkit for SSL/TLS", deps: &["ca-certificates"] },
    J { name: "ca-certificates", version: "20240226-r0", size_kb: 680, description: "Common CA certificates PEM files", deps: &[] },
    J { name: "libcurl", version: "8.5.0-r0", size_kb: 512, description: "The multiprotocol file transfer library", deps: &["openssl"] },
    J { name: "libffi", version: "3.4.4-r3", size_kb: 52, description: "Portable foreign function interface library", deps: &[] },
    
    J { name: "curl", version: "8.5.0-r0", size_kb: 356, description: "URL retrieval utility and library", deps: &["ca-certificates", "libcurl"] },
    J { name: "wget", version: "1.21.4-r0", size_kb: 480, description: "Network utility to retrieve files from the Web", deps: &["openssl"] },
    J { name: "git", version: "2.43.0-r0", size_kb: 14336, description: "Distributed version control system", deps: &["openssl", "curl", "perl"] },
    J { name: "openssh", version: "9.6_p1-r0", size_kb: 3072, description: "Port of OpenBSD's free SSH release", deps: &["openssl"] },
    J { name: "nmap", version: "7.94-r0", size_kb: 5120, description: "Network exploration and security scanner", deps: &[] },
    J { name: "tcpdump", version: "4.99.4-r1", size_kb: 640, description: "Network packet analyzer", deps: &[] },
    J { name: "socat", version: "1.8.0.0-r0", size_kb: 384, description: "Multipurpose relay for binary protocols", deps: &[] },
    J { name: "iperf3", version: "3.16-r0", size_kb: 192, description: "Network bandwidth measurement tool", deps: &[] },
    J { name: "bind-tools", version: "9.18.24-r0", size_kb: 2048, description: "ISC BIND DNS tools (dig)", deps: &[] },
    J { name: "mtr", version: "0.95-r2", size_kb: 192, description: "Network diagnostic tool", deps: &[] },
    J { name: "wireguard-tools", version: "1.0.20210914-r3", size_kb: 64, description: "WireGuard VPN tools", deps: &[] },
    J { name: "openvpn", version: "2.6.8-r0", size_kb: 1024, description: "VPN solution", deps: &["openssl"] },
    J { name: "lynx", version: "2.8.9-r5", size_kb: 2048, description: "Text-mode web browser", deps: &[] },
    
    J { name: "python3", version: "3.11.8-r0", size_kb: 25600, description: "High-level scripting language", deps: &["libffi", "openssl"] },
    J { name: "perl", version: "5.38.2-r0", size_kb: 16384, description: "Larry Wall's Practical Extraction and Report Language", deps: &[] },
    J { name: "gcc", version: "13.2.1_git20231014-r0", size_kb: 102400, description: "The GNU Compiler Collection", deps: &["binutils", "musl-dev"] },
    J { name: "rust", version: "1.75.0-r0", size_kb: 204800, description: "The Rust programming language", deps: &["gcc", "musl-dev"] },
    J { name: "nodejs", version: "20.11.1-r0", size_kb: 30720, description: "JavaScript runtime built on V8", deps: &["openssl", "libffi"] },
    J { name: "go", version: "1.21.6-r0", size_kb: 143360, description: "Go programming language", deps: &[] },
    J { name: "ruby", version: "3.2.3-r0", size_kb: 12288, description: "Ruby programming language", deps: &[] },
    J { name: "php83", version: "8.3.2-r0", size_kb: 15360, description: "PHP programming language", deps: &[] },
    J { name: "lua5.4", version: "5.4.6-r2", size_kb: 256, description: "Lua programming language", deps: &[] },
    J { name: "zig", version: "0.11.0-r0", size_kb: 51200, description: "Zig programming language", deps: &[] },
    J { name: "nim", version: "2.0.2-r0", size_kb: 10240, description: "Nim programming language", deps: &[] },
    J { name: "openjdk17-jre", version: "17.0.10-r0", size_kb: 204800, description: "OpenJDK 17 Runtime", deps: &[] },
    J { name: "elixir", version: "1.16.1-r0", size_kb: 7680, description: "Elixir programming language", deps: &[] },
    J { name: "clang", version: "17.0.5-r0", size_kb: 81920, description: "C language family frontend for LLVM", deps: &[] },
    J { name: "cmake", version: "3.27.8-r0", size_kb: 9728, description: "Cross-platform build system", deps: &[] },
    
    J { name: "binutils", version: "2.41-r0", size_kb: 8192, description: "Tools necessary to build programs", deps: &[] },
    J { name: "musl-dev", version: "1.2.4_git20230717-r4", size_kb: 1024, description: "musl C library development files", deps: &[] },
    J { name: "make", version: "4.4.1-r2", size_kb: 272, description: "GNU make utility", deps: &[] },
    J { name: "nasm", version: "2.16.01-r0", size_kb: 640, description: "Netwide Assembler", deps: &[] },
    
    J { name: "gdb", version: "14.1-r0", size_kb: 12800, description: "GNU debugger", deps: &[] },
    J { name: "valgrind", version: "3.22.0-r0", size_kb: 22528, description: "Memory debugging tool", deps: &[] },
    J { name: "strace", version: "6.7-r0", size_kb: 580, description: "System call tracer", deps: &[] },
    J { name: "ltrace", version: "0.7.3-r8", size_kb: 384, description: "Library call tracer", deps: &[] },
    J { name: "lsof", version: "4.99.3-r0", size_kb: 320, description: "List open files", deps: &[] },
    
    J { name: "nginx", version: "1.24.0-r15", size_kb: 2048, description: "HTTP and reverse proxy server", deps: &["openssl"] },
    J { name: "apache2", version: "2.4.58-r0", size_kb: 5120, description: "Apache HTTP Server", deps: &[] },
    J { name: "haproxy", version: "2.8.5-r0", size_kb: 3072, description: "TCP/HTTP Load Balancer", deps: &[] },
    J { name: "dnsmasq", version: "2.90-r0", size_kb: 384, description: "Lightweight DNS/DHCP server", deps: &[] },
    J { name: "squid", version: "6.6-r0", size_kb: 7680, description: "HTTP caching proxy", deps: &[] },
    
    J { name: "redis", version: "7.2.4-r0", size_kb: 4096, description: "In-memory data structure store", deps: &[] },
    J { name: "postgresql16", version: "16.2-r0", size_kb: 15360, description: "PostgreSQL database server", deps: &[] },
    J { name: "mariadb", version: "10.11.6-r0", size_kb: 25600, description: "MariaDB database server", deps: &[] },
    J { name: "sqlite", version: "3.44.2-r0", size_kb: 1024, description: "SQLite database engine", deps: &[] },
    
    J { name: "docker-cli", version: "24.0.7-r0", size_kb: 50000, description: "Docker container runtime", deps: &[] },
    J { name: "podman", version: "4.8.3-r0", size_kb: 40960, description: "Daemonless container engine", deps: &[] },
    J { name: "helm", version: "3.14.0-r0", size_kb: 15360, description: "Kubernetes package manager", deps: &[] },
    J { name: "kubectl", version: "1.29.1-r0", size_kb: 20480, description: "Kubernetes CLI", deps: &[] },
    J { name: "terraform", version: "1.7.2-r0", size_kb: 81920, description: "Infrastructure as code", deps: &[] },
    J { name: "ansible", version: "9.2.0-r0", size_kb: 25600, description: "IT automation tool", deps: &[] },
    
    J { name: "coreutils", version: "9.4-r1", size_kb: 6400, description: "GNU core utilities", deps: &[] },
    J { name: "findutils", version: "4.9.0-r5", size_kb: 640, description: "GNU find utilities", deps: &[] },
    J { name: "grep", version: "3.11-r0", size_kb: 320, description: "GNU grep", deps: &[] },
    J { name: "sed", version: "4.9-r2", size_kb: 224, description: "GNU stream editor", deps: &[] },
    J { name: "gawk", version: "5.3.0-r0", size_kb: 1024, description: "GNU awk", deps: &[] },
    J { name: "diffutils", version: "3.10-r0", size_kb: 384, description: "GNU diff utilities", deps: &[] },
    J { name: "patch", version: "2.7.6-r10", size_kb: 128, description: "GNU patch", deps: &[] },
    J { name: "less", version: "643-r0", size_kb: 192, description: "Pager program", deps: &[] },
    J { name: "file", version: "5.45-r1", size_kb: 640, description: "File type identification", deps: &[] },
    J { name: "iproute2", version: "6.7.0-r0", size_kb: 1024, description: "IP routing utilities", deps: &[] },
    J { name: "util-linux", version: "2.39.3-r0", size_kb: 4096, description: "System utilities", deps: &[] },
    J { name: "procps", version: "4.0.4-r0", size_kb: 480, description: "Process monitoring utilities", deps: &[] },
    J { name: "shadow", version: "4.14.3-r0", size_kb: 480, description: "User/group management", deps: &[] },
    J { name: "e2fsprogs", version: "1.47.0-r5", size_kb: 2048, description: "Ext2/3/4 filesystem utilities", deps: &[] },
    
    J { name: "gzip", version: "1.13-r0", size_kb: 96, description: "GNU zip compression", deps: &[] },
    J { name: "bzip2", version: "1.0.8-r6", size_kb: 128, description: "Block-sorting compressor", deps: &[] },
    J { name: "xz", version: "5.4.5-r0", size_kb: 256, description: "XZ Utils compression", deps: &[] },
    J { name: "zstd", version: "1.5.5-r8", size_kb: 384, description: "Zstandard compression", deps: &[] },
    J { name: "zip", version: "3.0-r12", size_kb: 192, description: "Create ZIP archives", deps: &[] },
    J { name: "unzip", version: "6.0-r14", size_kb: 192, description: "Extract ZIP archives", deps: &[] },
    J { name: "p7zip", version: "17.05-r0", size_kb: 2048, description: "7-Zip file archiver", deps: &[] },
    
    J { name: "ffmpeg", version: "6.1.1-r0", size_kb: 20480, description: "Complete multimedia framework", deps: &[] },
    J { name: "imagemagick", version: "7.1.1-r0", size_kb: 15360, description: "Image manipulation tools", deps: &[] },
    J { name: "mpv", version: "0.37.0-r0", size_kb: 5120, description: "Media player", deps: &[] },
    
    J { name: "ripgrep", version: "14.1.0-r0", size_kb: 6144, description: "Fast recursive grep alternative (rg)", deps: &[] },
    J { name: "fd", version: "9.0.0-r0", size_kb: 3072, description: "Simple fast alternative to find", deps: &[] },
    J { name: "bat", version: "0.24.0-r0", size_kb: 5120, description: "Cat clone with syntax highlighting", deps: &[] },
    J { name: "exa", version: "0.10.1-r3", size_kb: 1536, description: "Modern replacement for ls", deps: &[] },
    J { name: "fzf", version: "0.44.1-r0", size_kb: 3072, description: "Fuzzy finder", deps: &[] },
    J { name: "dust", version: "0.8.6-r0", size_kb: 2048, description: "Intuitive version of du", deps: &[] },
    J { name: "hyperfine", version: "1.18.0-r0", size_kb: 2048, description: "Command-line benchmarking tool", deps: &[] },
    J { name: "tokei", version: "12.1.2-r4", size_kb: 3072, description: "Code statistics tool", deps: &[] },
    
    J { name: "mercurial", version: "6.6.3-r0", size_kb: 7680, description: "Mercurial version control", deps: &[] },
    J { name: "subversion", version: "1.14.3-r0", size_kb: 5120, description: "Subversion version control", deps: &[] },
    J { name: "fossil", version: "2.23-r0", size_kb: 3072, description: "Fossil version control", deps: &[] },
    
    J { name: "htop", version: "3.3.0-r0", size_kb: 216, description: "Interactive process viewer", deps: &["ncurses-libs"] },
    J { name: "neofetch", version: "7.1.0-r3", size_kb: 76, description: "CLI system information tool", deps: &["bash"] },
    J { name: "tree", version: "2.1.1-r0", size_kb: 48, description: "Directory listing in tree-like format", deps: &[] },
    J { name: "jq", version: "1.7.1-r0", size_kb: 312, description: "Lightweight JSON processor", deps: &[] },
    J { name: "tmux", version: "3.3a-r5", size_kb: 424, description: "Terminal multiplexer", deps: &["ncurses-libs"] },
    J { name: "screen", version: "4.9.1-r0", size_kb: 640, description: "Terminal multiplexer", deps: &[] },
    J { name: "bc", version: "1.07.1-r4", size_kb: 128, description: "Calculator language", deps: &[] },
    J { name: "ncdu", version: "2.3-r0", size_kb: 192, description: "NCurses disk usage", deps: &[] },
    J { name: "ranger", version: "1.9.3-r6", size_kb: 640, description: "Console file manager", deps: &[] },
    J { name: "mc", version: "4.8.31-r0", size_kb: 3072, description: "Midnight Commander file manager", deps: &[] },
    J { name: "cowsay", version: "3.04-r2", size_kb: 24, description: "Talking cow", deps: &[] },
    J { name: "figlet", version: "2.2.5-r3", size_kb: 128, description: "Large text banners", deps: &[] },
    J { name: "sl", version: "5.05-r0", size_kb: 24, description: "Steam locomotive", deps: &[] },
    J { name: "fortune", version: "0.1-r2", size_kb: 1024, description: "Fortune cookie program", deps: &[] },
    J { name: "py3-pip", version: "23.3.2-r0", size_kb: 5120, description: "Python package installer", deps: &["python3"] },
    J { name: "certbot", version: "2.8.0-r0", size_kb: 3072, description: "ACME client for Let's Encrypt", deps: &[] },
    J { name: "fail2ban", version: "1.0.2-r0", size_kb: 2048, description: "Intrusion prevention", deps: &[] },
];

fn dpp(name: &str) -> Option<&'static J> {
    QA_.iter().find(|aa| aa.name == name)
}


pub struct LinuxSubsystem {
    
    state: LinuxState,
    
    vm_id: u64,
    
    boot_params: Tk,
    
    console: Option<VirtioConsole>,
    
    pending_commands: Vec<String>,
    
    last_result: Option<CommandResult>,
    
    boot_time: u64,
    
    embedded_kernel: Option<&'static [u8]>,
    
    embedded_initramfs: Option<&'static [u8]>,
    
    installed_packages: Vec<(&'static str, &'static str)>,   
    
    repo_updated: bool,
    
    online_mode: bool,
    
    pkg_server: Option<String>,
    
    real_files_installed: u32,
    
    total_bytes_downloaded: usize,
}

impl LinuxSubsystem {
    pub const fn new() -> Self {
        Self {
            state: LinuxState::NotStarted,
            vm_id: AGI_,
            boot_params: Tk {
                kernel_path: None,
                initramfs_path: None,
                cmdline: String::new(),
                memory_mb: WB_,
                serial_console: true,
            },
            console: None,
            pending_commands: Vec::new(),
            last_result: None,
            boot_time: 0,
            embedded_kernel: None,
            embedded_initramfs: None,
            installed_packages: Vec::new(),
            repo_updated: false,
            online_mode: false,
            pkg_server: None,
            real_files_installed: 0,
            total_bytes_downloaded: 0,
        }
    }

    
    pub fn state(&self) -> LinuxState {
        self.state
    }

    
    pub fn is_ready(&self) -> bool {
        self.state == LinuxState::Ready
    }

    
    pub fn is_package_installed(&self, name: &str) -> bool {
        self.installed_packages.iter().any(|(ae, _)| *ae == name)
    }

    
    
    fn check_network(&self) -> bool {
        
        if crate::netstack::dhcp::clk() {
            return true;
        }
        
        let mjh = crate::drivers::net::aoh();
        let mki = crate::virtio_net::is_initialized();
        if !mjh && !mki {
            return false;
        }
        
        crate::netstack::dhcp::start();
        
        let start = crate::logger::eg();
        loop {
            crate::netstack::poll();
            if crate::netstack::dhcp::clk() {
                return true;
            }
            let bb = crate::logger::eg().saturating_sub(start);
            if bb > 3000 {
                return false;
            }
            for _ in 0..10000 { core::hint::spin_loop(); }
        }
    }

    
    fn detect_pkg_server(&self) -> Option<String> {
        use alloc::vec::Vec;
        let mut dkh: Vec<String> = Vec::new();

        
        if let Some((_ip, _mask, fz, _dns)) = crate::netstack::dhcp::ibj() {
            if fz != [0, 0, 0, 0] {
                dkh.push(alloc::format!("http://{}.{}.{}.{}:8080", fz[0], fz[1], fz[2], fz[3]));
            }
        }
        
        dkh.push(String::from("http://10.0.2.2:8080"));
        dkh.push(String::from("http://192.168.56.1:8080"));

        
        dkh.dedup();

        for ain in &dkh {
            let url = alloc::format!("{}/repo/index", ain);
            crate::serial_println!("[TSL-PKG] Trying server: {}", url);
            if let Ok(eo) = crate::netstack::http::get(&url) {
                if eo.status_code == 200 && eo.body.len() > 10 {
                    crate::serial_println!("[TSL-PKG] Server found: {} ({} bytes index)",
                        ain, eo.body.len());
                    return Some(ain.clone());
                }
            }
        }
        None
    }

    
    fn download_package(&self, ain: &str, name: &str) -> Option<Vec<u8>> {
        let url = alloc::format!("{}/repo/pool/{}.pkg", ain, name);
        crate::serial_println!("[TSL-PKG] Downloading: {}", url);
        match crate::netstack::http::get(&url) {
            Ok(eo) if eo.status_code == 200 && !eo.body.is_empty() => {
                crate::serial_println!("[TSL-PKG] Downloaded {} bytes for {}", eo.body.len(), name);
                Some(eo.body)
            }
            Ok(eo) => {
                crate::serial_println!("[TSL-PKG] Server returned {} for {}", eo.status_code, name);
                None
            }
            Err(e) => {
                crate::serial_println!("[TSL-PKG] Download failed for {}: {}", name, e);
                None
            }
        }
    }

    
    
    
    
    
    
    
    fn extract_package_to_ramfs(&mut self, data: &[u8]) -> u32 {
        let text = match core::str::from_utf8(data) {
            Ok(t) => t,
            Err(_) => return 0,
        };

        let mut emo = 0u32;
        let mut ht: Option<&str> = None;
        let mut cwb = String::new();

        for line in text.lines() {
            if line.starts_with("PKG ") {
                
                continue;
            } else if line.starts_with("FILE /") {
                
                if let Some(path) = ht {
                    if self.install_file_to_ramfs(path, cwb.as_bytes()) {
                        emo += 1;
                    }
                }
                ht = Some(&line[5..]);
                cwb = String::new();
            } else if line == "EOF" {
                
                if let Some(path) = ht {
                    if self.install_file_to_ramfs(path, cwb.as_bytes()) {
                        emo += 1;
                    }
                }
                break;
            } else {
                
                if ht.is_some() {
                    if !cwb.is_empty() {
                        cwb.push('\n');
                    }
                    cwb.push_str(line);
                }
            }
        }

        self.real_files_installed += emo;
        emo
    }

    
    fn install_file_to_ramfs(&self, path: &str, content: &[u8]) -> bool {
        crate::ramfs::bh(|fs| {
            
            let mut it = String::new();
            let au: Vec<&str> = path.split('/').filter(|j| !j.is_empty()).collect();
            if au.len() > 1 {
                for jn in &au[..au.len() - 1] {
                    it.push('/');
                    it.push_str(jn);
                    let _ = fs.mkdir(&it);
                }
            }
            
            if fs.touch(path).is_ok() {
                if fs.write_file(path, content).is_ok() {
                    crate::serial_println!("[TSL-PKG] Installed: {} ({} bytes)", path, content.len());
                    return true;
                }
            }
            false
        })
    }

    
    pub fn erp(&self) -> bool {
        self.online_mode
    }

    
    pub fn pyz(&self) -> usize {
        self.total_bytes_downloaded
    }
    
    
    pub fn has_kernel(&self) -> bool {
        self.embedded_kernel.is_some()
    }
    
    
    pub fn kernel_size(&self) -> usize {
        self.embedded_kernel.map(|k| k.len()).unwrap_or(0)
    }
    
    
    pub fn has_initramfs(&self) -> bool {
        self.embedded_initramfs.is_some()
    }
    
    
    pub fn initramfs_size(&self) -> usize {
        self.embedded_initramfs.map(|i| i.len()).unwrap_or(0)
    }
    
    
    pub fn kernel_version_string(&self) -> Option<String> {
        let ny = self.embedded_kernel?;
        if ny.len() < 0x210 {
            return None;
        }
        
        let kernel_version_offset = u16::from_le_bytes([ny[0x20E], ny[0x20F]]) as usize;
        if kernel_version_offset == 0 || kernel_version_offset + 0x200 >= ny.len() {
            return None;
        }
        
        let edr = kernel_version_offset + 0x200;
        let mut dgg = alloc::vec::Vec::new();
        for i in 0..80 {
            if edr + i >= ny.len() {
                break;
            }
            let c = ny[edr + i];
            if c == 0 {
                break;
            }
            dgg.push(c);
        }
        
        core::str::from_utf8(&dgg).ok().map(|j| String::from(j))
    }
    
    
    pub fn boot_protocol_version(&self) -> Option<(u8, u8)> {
        let ny = self.embedded_kernel?;
        if ny.len() < 0x208 {
            return None;
        }
        
        let magic = u32::from_le_bytes([
            ny[0x202],
            ny[0x203],
            ny[0x204],
            ny[0x205],
        ]);
        
        if magic != 0x53726448 {
            return None;
        }
        
        let version = u16::from_le_bytes([ny[0x206], ny[0x207]]);
        Some(((version >> 8) as u8, (version & 0xFF) as u8))
    }

    
    pub fn init(&mut self) -> Result<()> {
        if self.state != LinuxState::NotStarted {
            return Err(HypervisorError::AlreadyRunning);
        }

        crate::serial_println!("[TSL] Initializing TrustOS Subsystem for Linux...");
        
        
        self.console = Some(VirtioConsole::new(self.vm_id));
        
        
        self.boot_params.cmdline = String::from("console=hvc0 quiet init=/init");
        
        self.state = LinuxState::NotStarted;
        
        crate::serial_println!("[TSL] Subsystem initialized (waiting for kernel)");
        
        Ok(())
    }

    
    pub fn set_embedded_images(&mut self, ny: &'static [u8], initramfs: &'static [u8]) {
        self.embedded_kernel = Some(ny);
        self.embedded_initramfs = Some(initramfs);
        
        
        if ny.len() >= 0x210 {
            let magic = u32::from_le_bytes([
                ny[0x202],
                ny[0x203],
                ny[0x204],
                ny[0x205],
            ]);
            
            if magic == 0x53726448 {  
                let version = u16::from_le_bytes([ny[0x206], ny[0x207]]);
                crate::serial_println!("[TSL] Linux kernel: {} bytes, boot protocol v{}.{}", 
                    ny.len(), version >> 8, version & 0xFF);
            } else {
                crate::serial_println!("[TSL] Warning: Invalid kernel magic: {:#X}", magic);
            }
        }
        
        crate::serial_println!("[TSL] Initramfs: {} bytes", initramfs.len());
        crate::serial_println!("[TSL] Total Linux guest size: {} KB", 
            (ny.len() + initramfs.len()) / 1024);
    }

    
    pub fn boot(&mut self) -> Result<()> {
        if self.state == LinuxState::Ready || self.state == LinuxState::Booting {
            return Ok(());
        }

        crate::serial_println!("[TSL] Booting Linux VM...");
        self.state = LinuxState::Booting;

        
        let kernel_data = match self.embedded_kernel {
            Some(k) => k,
            None => {
                crate::serial_println!("[TSL] No kernel image available");
                crate::serial_println!("[TSL] Falling back to simulated mode");
                self.state = LinuxState::Ready;
                return Ok(());
            }
        };

        
        if kernel_data.len() < 0x210 {
            crate::serial_println!("[TSL] Kernel too small ({} bytes)", kernel_data.len());
            self.state = LinuxState::Ready;
            return Ok(());
        }

        let magic = u32::from_le_bytes([
            kernel_data[0x202],
            kernel_data[0x203],
            kernel_data[0x204],
            kernel_data[0x205],
        ]);

        if magic != 0x53726448 {  
            crate::serial_println!("[TSL] Invalid kernel magic: {:#X} (expected HdrS)", magic);
            crate::serial_println!("[TSL] Falling back to simulated mode");
            self.state = LinuxState::Ready;
            return Ok(());
        }

        let version = u16::from_le_bytes([kernel_data[0x206], kernel_data[0x207]]);
        crate::serial_println!("[TSL] Linux boot protocol version: {}.{}", 
            version >> 8, version & 0xFF);
            
        
        let setup_sects = kernel_data[0x1F1];
        let btz = kernel_data[0x211];
        let kernel_version_offset = u16::from_le_bytes([kernel_data[0x20E], kernel_data[0x20F]]) as usize;
        
        crate::serial_println!("[TSL] Setup sectors: {}", setup_sects);
        crate::serial_println!("[TSL] Load flags: {:#X}", btz);
        
        
        if kernel_version_offset > 0 && kernel_version_offset + 0x200 < kernel_data.len() {
            let edr = kernel_version_offset + 0x200;
            let mut dgg = alloc::vec::Vec::new();
            for i in 0..64 {
                if edr + i >= kernel_data.len() {
                    break;
                }
                let c = kernel_data[edr + i];
                if c == 0 {
                    break;
                }
                dgg.push(c);
            }
            if !dgg.is_empty() {
                if let Ok(j) = core::str::from_utf8(&dgg) {
                    crate::serial_println!("[TSL] Kernel version: {}", j);
                }
            }
        }
        
        
        if let Some(initrd) = self.embedded_initramfs {
            crate::serial_println!("[TSL] Initramfs: {} bytes ({} KB)", 
                initrd.len(), initrd.len() / 1024);
        }

        
        if kernel_data.len() > 0x1000 {
            
            match self.boot_real_linux() {
                Ok(()) => {
                    crate::serial_println!("[TSL] Linux VM booted successfully!");
                    self.state = LinuxState::Ready;
                    return Ok(());
                }
                Err(e) => {
                    crate::serial_println!("[TSL] Real boot failed: {:?}", e);
                    crate::serial_println!("[TSL] Falling back to simulated mode");
                }
            }
        }

        
        self.state = LinuxState::Ready;
        crate::serial_println!("[TSL] Linux VM ready (simulated mode)");

        Ok(())
    }
    
    
    fn boot_real_linux(&mut self) -> Result<()> {
        let kernel_data = self.embedded_kernel.ok_or(HypervisorError::InvalidState)?;
        let eqn = self.embedded_initramfs.ok_or(HypervisorError::InvalidState)?;
        
        
        let cmdline = &self.boot_params.cmdline;
        let vm_id = super::linux_vm::ehd(kernel_data, eqn, cmdline)?;
        
        crate::serial_println!("[TSL] Linux VM started with ID: {}", vm_id);
        
        Ok(())
    }

    
    pub fn execute(&mut self, command: &str) -> Result<CommandResult> {
        if self.state != LinuxState::Ready {
            
            if self.state == LinuxState::NotStarted {
                self.init()?;
            }
            
            
            if self.state != LinuxState::Ready {
                return self.simulate_command(command);
            }
        }

        self.state = LinuxState::Busy;

        
        if let Some(ref mut console) = self.console {
            let fms = format!("{}\n", command);
            console.write(fms.as_bytes());
        }

        
        let result = self.simulate_command(command)?;

        self.state = LinuxState::Ready;
        self.last_result = Some(result.clone());

        Ok(result)
    }

    
    fn simulate_command(&mut self, command: &str) -> Result<CommandResult> {
        let au: Vec<&str> = command.split_whitespace().collect();
        
        if au.is_empty() {
            return Ok(CommandResult::success(String::new()));
        }

        let cmd = au[0];
        let args = &au[1..];

        let output = match cmd {
            "uname" => {
                if args.contains(&"-a") {
                    String::from("Linux trustos-vm 6.1.0 #1 SMP x86_64 GNU/Linux")
                } else if args.contains(&"-r") {
                    String::from("6.1.0")
                } else {
                    String::from("Linux")
                }
            }
            "echo" => {
                args.join(" ")
            }
            "whoami" => {
                String::from("root")
            }
            "pwd" => {
                String::from("/")
            }
            "ls" => {
                if args.is_empty() || args.contains(&"/") {
                    String::from("bin  dev  etc  home  init  lib  mnt  proc  root  run  sbin  sys  tmp  usr  var")
                } else if args.contains(&"-la") || args.contains(&"-l") {
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
                if args.contains(&"/etc/os-release") {
                    String::from(
"NAME=\"TrustOS Linux\"
VERSION=\"1.0\"
ID=trustos
PRETTY_NAME=\"TrustOS Linux 1.0\"
HOME_URL=\"https://trustos.local\"")
                } else if args.contains(&"/proc/version") {
                    String::from("Linux version 6.1.0 (gcc version 12.2.0) #1 SMP Jan 31 2026")
                } else if args.contains(&"/proc/cpuinfo") {
                    String::from(
"processor	: 0
vendor_id	: AuthenticAMD
model name	: AMD Ryzen 7 5800X
cpu MHz		: 3800.000
cache size	: 512 KB
flags		: fpu vme de pse tsc msr pae mce cx8 apic sep mtrr pge mca cmov pat pse36 clflush mmx fxsr sse sse2 ht syscall nx mmxext fxsr_opt pdpe1gb rdtscp lm 3dnowext 3dnow pni cx16 sse4_1 sse4_2 popcnt aes xsave avx hypervisor lahf_lm svm")
                } else if args.contains(&"/proc/meminfo") {
                    String::from(
"MemTotal:          65536 kB
MemFree:           32768 kB
MemAvailable:      48000 kB
Buffers:            4096 kB
Cached:            16384 kB")
                } else {
                    return Ok(CommandResult::error(1, format!("cat: {}: No such file or directory", args.join(" "))));
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
                if args.contains(&"-h") {
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
                if args.contains(&"-h") {
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
                return self.simulate_apt(args);
            }
            "apk" => {
                return self.simulate_apk(args);
            }
            "dpkg" => {
                if args.contains(&"-l") || args.contains(&"--list") {
                    let mut out = String::from("Desired=Unknown/Install/Remove/Purge/Hold\n| Status=Not/Inst/Conf-files/Unpacked/halF-conf/Half-inst/trig-aWait/Trig-pend\n|/ Err?=(none)/Reinst-required (Status,Err: uppercase=bad)\n||/ Name                    Version          Architecture Description\n+++-=======================-================-============-=================================\n");
                    for (name, tu) in &self.installed_packages {
                        out.push_str(&format!("ii  {:<24}{:<17}x86_64       {}\n", name, tu,
                            dpp(name).map(|aa| aa.description).unwrap_or("")));
                    }
                    return Ok(CommandResult::success(out));
                }
                return Ok(CommandResult::error(1, String::from("dpkg: use apt-get to manage packages")));
            }
            "which" => {
                if args.is_empty() {
                    return Ok(CommandResult::error(1, String::from("which: missing argument")));
                }
                let dji = args[0];
                
                if self.installed_packages.iter().any(|(ae, _)| *ae == dji) {
                    return Ok(CommandResult::success(format!("/usr/bin/{}", dji)));
                }
                
                let fke = ["ls", "cat", "echo", "whoami", "pwd", "date", "uname", "sh", "ash"];
                if fke.contains(&dji) {
                    return Ok(CommandResult::success(format!("/bin/{}", dji)));
                }
                return Ok(CommandResult::error(1, format!("{} not found", dji)));
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
                return Ok(CommandResult::error(127, format!("{}: command not found", cmd)));
            }
        };

        Ok(CommandResult::success(output))
    }

    
    fn simulate_apt(&mut self, args: &[&str]) -> Result<CommandResult> {
        if args.is_empty() {
            return Ok(CommandResult::error(1, String::from(
                "Usage: apt-get [update|install|remove|list|search|upgrade] [packages...]")));
        }

        let je = args[0];
        let cct = &args[1..];

        match je {
            "update" => {
                let mut out = String::new();

                
                if self.check_network() {
                    if let Some(ain) = self.detect_pkg_server() {
                        out.push_str(&format!("Connected to package server: {}\n", ain));
                        out.push_str(&format!("Get:1 {}/repo/index Packages [online]\n", ain));
                        self.pkg_server = Some(ain);
                        self.online_mode = true;
                    } else {
                        out.push_str("No package server found, using built-in repository.\n");
                        self.online_mode = false;
                    }
                } else {
                    self.online_mode = false;
                }

                if !self.online_mode {
                    out.push_str("Hit:1 http://dl-cdn.alpinelinux.org/alpine/v3.19/main x86_64 Packages\n");
                    out.push_str("Hit:2 http://dl-cdn.alpinelinux.org/alpine/v3.19/community x86_64 Packages\n");
                    out.push_str("Hit:3 http://security.alpinelinux.org/alpine/v3.19/main x86_64 Packages\n");
                }
                out.push_str("Reading package lists... Done\n");
                out.push_str("Building dependency tree... Done\n");
                if self.online_mode {
                    
                    let mut jns = QA_.len();
                    if let Some(ref bvw) = self.pkg_server {
                        let mot = alloc::format!("{}/repo", bvw);
                        if let Ok(eo) = crate::netstack::http::get(&mot) {
                            
                            if let Some(pos) = eo.body.windows(17).position(|w| w == b"total_available\":") {
                                let ef = &eo.body[pos + 17..];
                                let nlt = ef.iter().position(|&b| !b.is_ascii_digit() && b != b' ').unwrap_or(ef.len());
                                let rw = core::str::from_utf8(&ef[..nlt]).unwrap_or("").trim();
                                if let Ok(ae) = rw.parse::<usize>() {
                                    jns = ae;
                                }
                            }
                        }
                    }
                    out.push_str(&format!("{} packages available (live).\n", jns));
                } else {
                    out.push_str(&format!("{} packages can be upgraded. Run 'apt-get upgrade' to see them.\n",
                        QA_.len().saturating_sub(self.installed_packages.len()).min(8)));
                }
                self.repo_updated = true;
                Ok(CommandResult::success(out))
            }

            "install" | "add" => {
                if cct.is_empty() {
                    return Ok(CommandResult::error(1, String::from(
                        "E: No packages specified for installation.")));
                }

                if !self.repo_updated {
                    return Ok(CommandResult::error(1, String::from(
                        "E: The package lists are not up to date. Run 'apt-get update' first.")));
                }

                let mut out = String::new();
                out.push_str("Reading package lists... Done\n");
                out.push_str("Building dependency tree... Done\n");

                
                let mut bec: Vec<&'static J> = Vec::new();
                let mut evg: Vec<&str> = Vec::new();
                let mut ela: Vec<&str> = Vec::new(); 

                for &name in cct {
                    
                    if name.starts_with('-') { continue; }
                    if let Some(gh) = dpp(name) {
                        
                        for dep_name in gh.deps {
                            if !self.installed_packages.iter().any(|(ae, _)| ae == dep_name)
                                && !bec.iter().any(|aa| aa.name == *dep_name)
                            {
                                if let Some(dep) = dpp(dep_name) {
                                    bec.push(dep);
                                }
                            }
                        }
                        
                        if !self.installed_packages.iter().any(|(ae, _)| *ae == gh.name)
                            && !bec.iter().any(|aa| aa.name == gh.name)
                        {
                            bec.push(gh);
                        }
                    } else if self.online_mode && self.pkg_server.is_some() {
                        
                        ela.push(name);
                    } else {
                        evg.push(name);
                    }
                }

                for nf in &evg {
                    out.push_str(&format!("E: Unable to locate package {}\n", nf));
                }

                
                let mut ftq = 0u32;
                if !ela.is_empty() {
                    let bvw = self.pkg_server.clone().unwrap_or_default();
                    for &dyn_name in &ela {
                        out.push_str(&format!("Resolving {} via package server...\n", dyn_name));
                        if let Some(data) = self.download_package(&bvw, dyn_name) {
                            let cwt = data.len();
                            self.total_bytes_downloaded += cwt;
                            let files = self.extract_package_to_ramfs(&data);
                            if files > 0 {
                                out.push_str(&format!("Get:1 {}/repo/pool/{}.pkg [{} B]\n", bvw, dyn_name, cwt));
                                out.push_str(&format!("  -> Downloaded {} bytes, extracted {} files\n", cwt, files));
                                
                                let pro = core::str::from_utf8(&data).unwrap_or("")
                                    .lines().next().unwrap_or("")
                                    .splitn(3, ' ').nth(2).unwrap_or("latest");
                                
                                let nhl = alloc::boxed::Box::leak(String::from(dyn_name).into_boxed_str());
                                let prn = alloc::boxed::Box::leak(String::from(pro).into_boxed_str());
                                self.installed_packages.push((nhl, prn));
                                ftq += 1;
                            } else {
                                out.push_str(&format!("E: Unable to locate package {}\n", dyn_name));
                            }
                        } else {
                            out.push_str(&format!("E: Unable to locate package {}\n", dyn_name));
                        }
                    }
                }

                if bec.is_empty() && ftq == 0 && evg.is_empty() && ela.is_empty() {
                    out.push_str("All requested packages are already installed.\n");
                    return Ok(CommandResult::success(out));
                }

                if bec.is_empty() && ftq == 0 {
                    return Ok(CommandResult::error(1, out));
                }

                
                let njg: Vec<&str> = bec.iter().map(|aa| aa.name).collect();
                let total_size: u32 = bec.iter().map(|aa| aa.size_kb).sum();

                out.push_str("The following NEW packages will be installed:\n  ");
                out.push_str(&njg.join(" "));
                out.push_str("\n");
                out.push_str(&format!("{} newly installed, 0 to remove and 0 not upgraded.\n",
                    bec.len()));
                out.push_str(&format!("Need to get {} kB of archives.\n", total_size));
                out.push_str(&format!("After this operation, {} kB of additional disk space will be used.\n",
                    total_size * 3));

                
                let ain = self.pkg_server.clone();
                let erp = self.online_mode && ain.is_some();

                for gh in &bec {
                    if erp {
                        let bvw = ain.as_deref().unwrap();
                        out.push_str(&format!("Get:1 {}/repo/pool/{}.pkg {} [{} kB]\n",
                            bvw, gh.name, gh.version, gh.size_kb));

                        
                        if let Some(data) = self.download_package(bvw, gh.name) {
                            let cwt = data.len();
                            self.total_bytes_downloaded += cwt;
                            let files = self.extract_package_to_ramfs(&data);
                            out.push_str(&format!("  -> Downloaded {} bytes, extracted {} files\n",
                                cwt, files));
                        } else {
                            out.push_str("  -> Download failed, using cached metadata\n");
                        }
                    } else {
                        out.push_str(&format!("Get:1 http://dl-cdn.alpinelinux.org/alpine/v3.19/main x86_64 {} {} [{} kB]\n",
                            gh.name, gh.version, gh.size_kb));
                    }
                }
                if erp {
                    out.push_str(&format!("Fetched {} bytes from {}\n",
                        self.total_bytes_downloaded, ain.as_deref().unwrap()));
                } else {
                    out.push_str(&format!("Fetched {} kB in 0s (internal)\n", total_size));
                }

                
                for gh in &bec {
                    out.push_str(&format!("Selecting previously unselected package {}.\n", gh.name));
                    out.push_str(&format!("Preparing to unpack {}_{}_amd64.deb ...\n", gh.name, gh.version));
                    out.push_str(&format!("Unpacking {} ({}) ...\n", gh.name, gh.version));
                }

                
                out.push_str("Setting up packages ...\n");
                for gh in &bec {
                    out.push_str(&format!("Setting up {} ({}) ...\n", gh.name, gh.version));
                    self.installed_packages.push((gh.name, gh.version));
                }

                out.push_str("Processing triggers for man-db ...\n");
                if erp {
                    out.push_str(&format!("[online] {} files installed to filesystem.\n",
                        self.real_files_installed));
                }

                if !evg.is_empty() {
                    return Ok(CommandResult { exit_code: 1, stdout: out, stderr: String::new(), duration_ms: 0 });
                }
                Ok(CommandResult::success(out))
            }

            "remove" | "purge" | "del" => {
                if cct.is_empty() {
                    return Ok(CommandResult::error(1, String::from(
                        "E: No packages specified for removal.")));
                }

                let mut out = String::new();
                out.push_str("Reading package lists... Done\n");
                out.push_str("Building dependency tree... Done\n");

                let mut ddj = 0u32;
                let mut iab = 0u32;
                for &name in cct {
                    if name.starts_with('-') { continue; }
                    if let Some(pos) = self.installed_packages.iter().position(|(ae, _)| *ae == name) {
                        let (biq, pver) = self.installed_packages.remove(pos);
                        let size = dpp(biq).map(|aa| aa.size_kb).unwrap_or(100);
                        out.push_str(&format!("Removing {} ({}) ...\n", biq, pver));
                        iab += size * 3;
                        ddj += 1;
                    } else {
                        out.push_str(&format!("Package '{}' is not installed, so not removed.\n", name));
                    }
                }

                if ddj > 0 {
                    out.push_str(&format!("{} packages removed, {} kB disk space freed.\n",
                        ddj, iab));
                }
                Ok(CommandResult::success(out))
            }

            "list" | "list-installed" => {
                let mut out = String::new();
                if self.installed_packages.is_empty() {
                    out.push_str("No packages installed.\n");
                    out.push_str("Use 'apt-get install <package>' to install packages.\n");
                } else {
                    out.push_str("Listing installed packages...\n");
                    out.push_str(&format!("{:<24} {:<24} {}\n", "Package", "Version", "Description"));
                    out.push_str(&format!("{:-<24} {:-<24} {:-<30}\n", "", "", ""));
                    for (name, tu) in &self.installed_packages {
                        let desc = dpp(name).map(|aa| aa.description).unwrap_or("");
                        out.push_str(&format!("{:<24} {:<24} {}\n", name, tu, desc));
                    }
                    out.push_str(&format!("\n{} packages installed.\n", self.installed_packages.len()));
                }
                Ok(CommandResult::success(out))
            }

            "search" => {
                if cct.is_empty() {
                    return Ok(CommandResult::error(1, String::from("Usage: apt-get search <keyword>")));
                }
                let ijd = cct[0].to_lowercase();
                
                let mut out = String::new();
                let mut count = 0;
                
                for gh in QA_ {
                    if gh.name.contains(ijd.as_str()) || gh.description.to_lowercase().contains(ijd.as_str()) {
                        let installed = if self.installed_packages.iter().any(|(ae, _)| *ae == gh.name) {
                            " [installed]"
                        } else {
                            ""
                        };
                        out.push_str(&format!("{}/{} {} x86_64{}\n  {}\n\n",
                            gh.name, gh.version, gh.size_kb, installed, gh.description));
                        count += 1;
                    }
                }
                
                if self.online_mode {
                    if let Some(ref bvw) = self.pkg_server {
                        let omn = alloc::format!("{}/repo/search?q={}", bvw, cct[0]);
                        if let Ok(eo) = crate::netstack::http::get(&omn) {
                            if eo.status_code == 200 {
                                let text = core::str::from_utf8(&eo.body).unwrap_or("");
                                for line in text.lines() {
                                    if line.is_empty() || line == "No results" { continue; }
                                    
                                    let au: Vec<&str> = line.splitn(7, ' ').collect();
                                    if au.len() >= 7 {
                                        let biq = au[0];
                                        
                                        if QA_.iter().any(|aa| aa.name == biq) { continue; }
                                        let installed = if self.installed_packages.iter().any(|(ae, _)| *ae == biq) {
                                            " [installed]"
                                        } else {
                                            ""
                                        };
                                        out.push_str(&format!("{}/{} {} {}{}\n  {}\n\n",
                                            biq, au[1], au[2], au[3], installed, au[6]));
                                        count += 1;
                                    }
                                }
                            }
                        }
                    }
                }
                if count == 0 {
                    out.push_str(&format!("No packages found matching '{}'.\n", cct[0]));
                } else {
                    out.push_str(&format!("{} packages found.\n", count));
                }
                Ok(CommandResult::success(out))
            }

            "upgrade" => {
                let mut out = String::new();
                out.push_str("Reading package lists... Done\n");
                out.push_str("Building dependency tree... Done\n");
                out.push_str("Calculating upgrade... Done\n");
                out.push_str("0 upgraded, 0 newly installed, 0 to remove and 0 not upgraded.\n");
                Ok(CommandResult::success(out))
            }

            _ => {
                Ok(CommandResult::error(1, format!("E: Invalid operation '{}'", je)))
            }
        }
    }

    
    fn simulate_apk(&mut self, args: &[&str]) -> Result<CommandResult> {
        if args.is_empty() {
            return Ok(CommandResult::error(1, String::from(
                "Usage: apk [update|add|del|list|search|info] [packages...]")));
        }

        
        let ggn: &[&str] = match args[0] {
            "add" => &["install"],
            "del" => &["remove"],
            "info" => &["search"],
            other => &[other],
        };

        let mut iah = ggn.to_vec();
        iah.extend_from_slice(&args[1..]);
        self.simulate_apt(&iah)
    }

    
    pub fn shutdown(&mut self) -> Result<()> {
        if self.state == LinuxState::NotStarted {
            return Ok(());
        }

        crate::serial_println!("[TSL] Shutting down Linux VM...");
        self.state = LinuxState::ShuttingDown;

        
        if let Some(ref mut console) = self.console {
            console.write(b"poweroff\n");
        }

        
        self.console = None;
        self.state = LinuxState::NotStarted;

        crate::serial_println!("[TSL] Linux VM stopped");

        Ok(())
    }

    
    pub fn console(&mut self) -> Option<&mut VirtioConsole> {
        self.console.as_mut()
    }
}


static IY_: Mutex<LinuxSubsystem> = Mutex::new(LinuxSubsystem::new());


pub fn init() -> Result<()> {
    IY_.lock().init()
}


pub fn boot() -> Result<()> {
    IY_.lock().boot()
}


pub fn execute(command: &str) -> Result<CommandResult> {
    IY_.lock().execute(command)
}


pub fn is_ready() -> bool {
    IY_.lock().is_ready()
}


pub fn state() -> LinuxState {
    IY_.lock().state()
}


pub fn shutdown() -> Result<()> {
    IY_.lock().shutdown()
}


pub fn acs() -> spin::MutexGuard<'static, LinuxSubsystem> {
    IY_.lock()
}
