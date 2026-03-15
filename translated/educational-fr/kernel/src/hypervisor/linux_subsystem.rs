//! TrustOS Subsystem for Linux (TSL)
//!
//! Provides the ability to run Linux commands from TrustOS by booting
//! a minimal Linux VM in the background and executing commands via
//! virtio-console communication.
//!
//! Architecture:
//! ```
//! ┌─────────────────────────────────────────────────┐
//! │                    TrustOS Shell                │
//! │  trustos:/$ linux ls -la                       │
//! └──────────────────────┬──────────────────────────┘
//!                        │ Virtio-Console
//!                        ▼
//! ┌─────────────────────────────────────────────────┐
//! │              Linux VM (Background)              │
//! │  Alpine Linux + BusyBox + Basic Tools          │
//! └─────────────────────────────────────────────────┘
//! ```

use alloc::string::String;
use alloc::vec::Vec;
use alloc::format;
use alloc::boxed::Box;
use core::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use spin::Mutex;

use super::{HypervisorError, Result, CpuVendor, cpu_vendor};
use super::virtio_console::{self, VirtioConsole, ConsolePort};

/// Linux VM ID (special reserved ID for the Linux subsystem)
pub // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const LINUX_VM_ID: u64 = 0xFFFF_FFFF_FFFF_0001;

/// Default memory for Linux VM (64 MB)
pub // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const LINUX_VM_MEMORY_MB: usize = 64;

/// Linux subsystem state
#[derive(Debug, Clone, Copy, PartialEq)]
// Énumération — un type qui peut être l'une de plusieurs variantes.
pub enum LinuxState {
    /// Not initialized
    NotStarted,
    /// Booting the Linux kernel
    Booting,
    /// Linux is ready to accept commands
    Ready,
    /// Executing a command
    Busy,
    /// Error state
    Error,
    /// Shutting down
    ShuttingDown,
}

/// Linux kernel boot parameters
#[derive(Debug, Clone)]
// Structure publique — visible à l'extérieur de ce module.
pub struct LinuxBootParams {
    /// Path to bzImage (kernel)
    pub kernel_path: Option<String>,
    /// Path to initramfs
    pub initramfs_path: Option<String>,
    /// Kernel command line
    pub cmdline: String,
    /// Memory in MB
    pub memory_mb: usize,
    /// Enable serial console
    pub serial_console: bool,
}

// Implémentation de trait — remplit un contrat comportemental.
impl Default for LinuxBootParams {
    fn default() -> Self {
        Self {
            kernel_path: None,
            initramfs_path: None,
            cmdline: String::from("console=hvc0 quiet"),
            memory_mb: LINUX_VM_MEMORY_MB,
            serial_console: true,
        }
    }
}

/// Command execution result
#[derive(Debug, Clone)]
// Structure publique — visible à l'extérieur de ce module.
pub struct CommandResult {
    /// Exit code (0 = success)
    pub exit_code: i32,
    /// Standard output
    pub stdout: String,
    /// Standard error
    pub stderr: String,
    /// Execution time in milliseconds
    pub duration_mouse: u64,
}

// Bloc d'implémentation — définit les méthodes du type ci-dessus.
impl CommandResult {
        // Fonction publique — appelable depuis d'autres modules.
pub fn success(stdout: String) -> Self {
        Self {
            exit_code: 0,
            stdout,
            stderr: String::new(),
            duration_mouse: 0,
        }
    }

        // Fonction publique — appelable depuis d'autres modules.
pub fn error(code: i32, stderr: String) -> Self {
        Self {
            exit_code: code,
            stdout: String::new(),
            stderr,
            duration_mouse: 0,
        }
    }
}

/// Linux Boot Protocol structures
/// See: https://www.kernel.org/doc/html/latest/x86/boot.html
#[repr(C, packed)]
// #[derive] — génère automatiquement les implémentations de traits à la compilation.
#[derive(Debug, Clone, Copy)]
// Structure publique — visible à l'extérieur de ce module.
pub struct LinuxSetupHeader {
    pub setup_sects: u8,
    pub root_flags: u16,
    pub syssize: u32,
    pub ram_size: u16,
    pub vid_mode: u16,
    pub root_device: u16,
    pub boot_flag: u16,
    pub jump: u16,
    pub header: u32,           // "HdrS" magic
    pub version: u16,
    pub realmode_swtch: u32,
    pub start_system_seg: u16,
    pub kernel_version: u16,
    pub type_of_loader: u8,
    pub loadflags: u8,
    pub setup_move_size: u16,
    pub code32_start: u32,
    pub ramdisk_image: u32,
    pub ramdisk_size: u32,
    pub bootsect_kludge: u32,
    pub heap_end_pointer: u16,
    pub ext_loader_ver: u8,
    pub ext_loader_type: u8,
    pub command_line_pointer: u32,
    pub initrd_address_maximum: u32,
    pub kernel_alignment: u32,
    pub relocatable_kernel: u8,
    pub minimum_alignment: u8,
    pub xloadflags: u16,
    pub cmdline_size: u32,
    pub hardware_subarch: u32,
    pub hardware_subarch_data: u64,
    pub payload_offset: u32,
    pub payload_length: u32,
    pub setup_data: u64,
    pub pref_address: u64,
    pub initialize_size: u32,
    pub handover_offset: u32,
}

/// Boot parameters passed to Linux kernel
#[repr(C)]
// #[derive] — génère automatiquement les implémentations de traits à la compilation.
#[derive(Clone)]
// Structure publique — visible à l'extérieur de ce module.
pub struct BootParams {
    pub screen_information: [u8; 64],
    pub apm_bios_information: [u8; 20],
    pub _pad1: [u8; 4],
    pub tboot_address: u64,
    pub ist_information: [u8; 16],
    pub _pad2: [u8; 16],
    pub hd0_information: [u8; 16],
    pub hd1_information: [u8; 16],
    pub system_descriptor_table: [u8; 16],
    pub olpc_ofw_header: [u8; 16],
    pub ext_ramdisk_image: u32,
    pub ext_ramdisk_size: u32,
    pub ext_command_line_pointer: u32,
    pub _pad3: [u8; 116],
    pub edid_information: [u8; 128],
    pub efi_information: [u8; 32],
    pub alt_memory_k: u32,
    pub scratch: u32,
    pub e820_entries: u8,
    pub eddbuf_entries: u8,
    pub edd_mbr_signal_buffer_entries: u8,
    pub kbd_status: u8,
    pub secure_boot: u8,
    pub _pad4: [u8; 2],
    pub sentinel: u8,
    pub _pad5: [u8; 1],
    pub header: LinuxSetupHeader,
    pub _pad6: [u8; 36],
    pub edd_mbr_signal_buffer: [u32; 16],
    pub e820_table: [E820Entry; 128],
    pub _pad7: [u8; 48],
    pub eddbuf: [u8; 492],
    pub _pad8: [u8; 276],
}

/// E820 memory map entry
#[repr(C)]
// #[derive] — génère automatiquement les implémentations de traits à la compilation.
#[derive(Debug, Clone, Copy, Default)]
// Structure publique — visible à l'extérieur de ce module.
pub struct E820Entry {
    pub address: u64,
    pub size: u64,
    pub entry_type: u32,
    pub _pad: u32,
}

/// E820 memory types
pub mod e820_type {
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const RAM: u32 = 1;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const RESERVED: u32 = 2;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const ACPI: u32 = 3;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const NVS: u32 = 4;
    pub     // Constante de compilation — évaluée à la compilation, coût zéro à l'exécution.
const UNUSABLE: u32 = 5;
}

/// Package metadata for simulated package manager
#[derive(Debug, Clone)]
// Structure publique — visible à l'extérieur de ce module.
pub struct PackageInformation {
    pub name: &'static str,
    pub version: &'static str,
    pub size_keyboard: u32,
    pub description: &'static str,
    pub deps: &'static [&'static str],
}

/// Available packages in the simulated repository
const REPO_PACKAGES: &[PackageInformation] = &[
    // Editors
    PackageInformation { name: "vim", version: "9.0.2127-r0", size_keyboard: 1824, description: "Improved vi-style text editor", deps: &["ncurses-libs", "vim-common"] },
    PackageInformation { name: "vim-common", version: "9.0.2127-r0", size_keyboard: 6240, description: "Vim common files", deps: &[] },
    PackageInformation { name: "nano", version: "7.2-r1", size_keyboard: 612, description: "Nano text editor", deps: &["ncurses-libs"] },
    PackageInformation { name: "emacs", version: "29.1-r0", size_keyboard: 48000, description: "GNU Emacs editor", deps: &[] },
    PackageInformation { name: "micro", version: "2.0.13-r0", size_keyboard: 11264, description: "Modern terminal text editor", deps: &[] },
    PackageInformation { name: "helix", version: "23.10-r0", size_keyboard: 24576, description: "Post-modern modal text editor", deps: &[] },
    // Shells
    PackageInformation { name: "bash", version: "5.2.21-r0", size_keyboard: 1320, description: "The GNU Bourne Again shell", deps: &["ncurses-libs"] },
    PackageInformation { name: "zsh", version: "5.9.0-r0", size_keyboard: 3200, description: "Z shell", deps: &["ncurses-libs"] },
    PackageInformation { name: "fish", version: "3.7.0-r0", size_keyboard: 6400, description: "Friendly interactive shell", deps: &["ncurses-libs"] },
    PackageInformation { name: "dash", version: "0.5.12-r0", size_keyboard: 96, description: "POSIX compliant shell", deps: &[] },
    // Libraries
    PackageInformation { name: "ncurses-libs", version: "6.4_p20231125-r0", size_keyboard: 308, description: "Ncurses libraries", deps: &[] },
    PackageInformation { name: "openssl", version: "3.1.4-r5", size_keyboard: 7168, description: "Toolkit for SSL/TLS", deps: &["ca-certificates"] },
    PackageInformation { name: "ca-certificates", version: "20240226-r0", size_keyboard: 680, description: "Common CA certificates PEM files", deps: &[] },
    PackageInformation { name: "libcurl", version: "8.5.0-r0", size_keyboard: 512, description: "The multiprotocol file transfer library", deps: &["openssl"] },
    PackageInformation { name: "libffi", version: "3.4.4-r3", size_keyboard: 52, description: "Portable foreign function interface library", deps: &[] },
    // Network tools
    PackageInformation { name: "curl", version: "8.5.0-r0", size_keyboard: 356, description: "URL retrieval utility and library", deps: &["ca-certificates", "libcurl"] },
    PackageInformation { name: "wget", version: "1.21.4-r0", size_keyboard: 480, description: "Network utility to retrieve files from the Web", deps: &["openssl"] },
    PackageInformation { name: "git", version: "2.43.0-r0", size_keyboard: 14336, description: "Distributed version control system", deps: &["openssl", "curl", "perl"] },
    PackageInformation { name: "openssh", version: "9.6_p1-r0", size_keyboard: 3072, description: "Port of OpenBSD's free SSH release", deps: &["openssl"] },
    PackageInformation { name: "nmap", version: "7.94-r0", size_keyboard: 5120, description: "Network exploration and security scanner", deps: &[] },
    PackageInformation { name: "tcpdump", version: "4.99.4-r1", size_keyboard: 640, description: "Network packet analyzer", deps: &[] },
    PackageInformation { name: "socat", version: "1.8.0.0-r0", size_keyboard: 384, description: "Multipurpose relay for binary protocols", deps: &[] },
    PackageInformation { name: "iperf3", version: "3.16-r0", size_keyboard: 192, description: "Network bandwidth measurement tool", deps: &[] },
    PackageInformation { name: "bind-tools", version: "9.18.24-r0", size_keyboard: 2048, description: "ISC BIND DNS tools (dig)", deps: &[] },
    PackageInformation { name: "mtr", version: "0.95-r2", size_keyboard: 192, description: "Network diagnostic tool", deps: &[] },
    PackageInformation { name: "wireguard-tools", version: "1.0.20210914-r3", size_keyboard: 64, description: "WireGuard VPN tools", deps: &[] },
    PackageInformation { name: "openvpn", version: "2.6.8-r0", size_keyboard: 1024, description: "VPN solution", deps: &["openssl"] },
    PackageInformation { name: "lynx", version: "2.8.9-r5", size_keyboard: 2048, description: "Text-mode web browser", deps: &[] },
    // Languages
    PackageInformation { name: "python3", version: "3.11.8-r0", size_keyboard: 25600, description: "High-level scripting language", deps: &["libffi", "openssl"] },
    PackageInformation { name: "perl", version: "5.38.2-r0", size_keyboard: 16384, description: "Larry Wall's Practical Extraction and Report Language", deps: &[] },
    PackageInformation { name: "gcc", version: "13.2.1_git20231014-r0", size_keyboard: 102400, description: "The GNU Compiler Collection", deps: &["binutils", "musl-dev"] },
    PackageInformation { name: "rust", version: "1.75.0-r0", size_keyboard: 204800, description: "The Rust programming language", deps: &["gcc", "musl-dev"] },
    PackageInformation { name: "nodejs", version: "20.11.1-r0", size_keyboard: 30720, description: "JavaScript runtime built on V8", deps: &["openssl", "libffi"] },
    PackageInformation { name: "go", version: "1.21.6-r0", size_keyboard: 143360, description: "Go programming language", deps: &[] },
    PackageInformation { name: "ruby", version: "3.2.3-r0", size_keyboard: 12288, description: "Ruby programming language", deps: &[] },
    PackageInformation { name: "php83", version: "8.3.2-r0", size_keyboard: 15360, description: "PHP programming language", deps: &[] },
    PackageInformation { name: "lua5.4", version: "5.4.6-r2", size_keyboard: 256, description: "Lua programming language", deps: &[] },
    PackageInformation { name: "zig", version: "0.11.0-r0", size_keyboard: 51200, description: "Zig programming language", deps: &[] },
    PackageInformation { name: "nim", version: "2.0.2-r0", size_keyboard: 10240, description: "Nim programming language", deps: &[] },
    PackageInformation { name: "openjdk17-jre", version: "17.0.10-r0", size_keyboard: 204800, description: "OpenJDK 17 Runtime", deps: &[] },
    PackageInformation { name: "elixir", version: "1.16.1-r0", size_keyboard: 7680, description: "Elixir programming language", deps: &[] },
    PackageInformation { name: "clang", version: "17.0.5-r0", size_keyboard: 81920, description: "C language family frontend for LLVM", deps: &[] },
    PackageInformation { name: "cmake", version: "3.27.8-r0", size_keyboard: 9728, description: "Cross-platform build system", deps: &[] },
    // Build tools
    PackageInformation { name: "binutils", version: "2.41-r0", size_keyboard: 8192, description: "Tools necessary to build programs", deps: &[] },
    PackageInformation { name: "musl-dev", version: "1.2.4_git20230717-r4", size_keyboard: 1024, description: "musl C library development files", deps: &[] },
    PackageInformation { name: "make", version: "4.4.1-r2", size_keyboard: 272, description: "GNU make utility", deps: &[] },
    PackageInformation { name: "nasm", version: "2.16.01-r0", size_keyboard: 640, description: "Netwide Assembler", deps: &[] },
    // Debug tools
    PackageInformation { name: "gdb", version: "14.1-r0", size_keyboard: 12800, description: "GNU debugger", deps: &[] },
    PackageInformation { name: "valgrind", version: "3.22.0-r0", size_keyboard: 22528, description: "Memory debugging tool", deps: &[] },
    PackageInformation { name: "strace", version: "6.7-r0", size_keyboard: 580, description: "System call tracer", deps: &[] },
    PackageInformation { name: "ltrace", version: "0.7.3-r8", size_keyboard: 384, description: "Library call tracer", deps: &[] },
    PackageInformation { name: "lsof", version: "4.99.3-r0", size_keyboard: 320, description: "List open files", deps: &[] },
    // Servers
    PackageInformation { name: "nginx", version: "1.24.0-r15", size_keyboard: 2048, description: "HTTP and reverse proxy server", deps: &["openssl"] },
    PackageInformation { name: "apache2", version: "2.4.58-r0", size_keyboard: 5120, description: "Apache HTTP Server", deps: &[] },
    PackageInformation { name: "haproxy", version: "2.8.5-r0", size_keyboard: 3072, description: "TCP/HTTP Load Balancer", deps: &[] },
    PackageInformation { name: "dnsmasq", version: "2.90-r0", size_keyboard: 384, description: "Lightweight DNS/DHCP server", deps: &[] },
    PackageInformation { name: "squid", version: "6.6-r0", size_keyboard: 7680, description: "HTTP caching proxy", deps: &[] },
    // Databases
    PackageInformation { name: "redis", version: "7.2.4-r0", size_keyboard: 4096, description: "In-memory data structure store", deps: &[] },
    PackageInformation { name: "postgresql16", version: "16.2-r0", size_keyboard: 15360, description: "PostgreSQL database server", deps: &[] },
    PackageInformation { name: "mariadb", version: "10.11.6-r0", size_keyboard: 25600, description: "MariaDB database server", deps: &[] },
    PackageInformation { name: "sqlite", version: "3.44.2-r0", size_keyboard: 1024, description: "SQLite database engine", deps: &[] },
    // Containers & cloud
    PackageInformation { name: "docker-cli", version: "24.0.7-r0", size_keyboard: 50000, description: "Docker container runtime", deps: &[] },
    PackageInformation { name: "podman", version: "4.8.3-r0", size_keyboard: 40960, description: "Daemonless container engine", deps: &[] },
    PackageInformation { name: "helm", version: "3.14.0-r0", size_keyboard: 15360, description: "Kubernetes package manager", deps: &[] },
    PackageInformation { name: "kubectl", version: "1.29.1-r0", size_keyboard: 20480, description: "Kubernetes CLI", deps: &[] },
    PackageInformation { name: "terraform", version: "1.7.2-r0", size_keyboard: 81920, description: "Infrastructure as code", deps: &[] },
    PackageInformation { name: "ansible", version: "9.2.0-r0", size_keyboard: 25600, description: "IT automation tool", deps: &[] },
    // System utilities
    PackageInformation { name: "coreutils", version: "9.4-r1", size_keyboard: 6400, description: "GNU core utilities", deps: &[] },
    PackageInformation { name: "findutils", version: "4.9.0-r5", size_keyboard: 640, description: "GNU find utilities", deps: &[] },
    PackageInformation { name: "grep", version: "3.11-r0", size_keyboard: 320, description: "GNU grep", deps: &[] },
    PackageInformation { name: "sed", version: "4.9-r2", size_keyboard: 224, description: "GNU stream editor", deps: &[] },
    PackageInformation { name: "gawk", version: "5.3.0-r0", size_keyboard: 1024, description: "GNU awk", deps: &[] },
    PackageInformation { name: "diffutils", version: "3.10-r0", size_keyboard: 384, description: "GNU diff utilities", deps: &[] },
    PackageInformation { name: "patch", version: "2.7.6-r10", size_keyboard: 128, description: "GNU patch", deps: &[] },
    PackageInformation { name: "less", version: "643-r0", size_keyboard: 192, description: "Pager program", deps: &[] },
    PackageInformation { name: "file", version: "5.45-r1", size_keyboard: 640, description: "File type identification", deps: &[] },
    PackageInformation { name: "iproute2", version: "6.7.0-r0", size_keyboard: 1024, description: "IP routing utilities", deps: &[] },
    PackageInformation { name: "util-linux", version: "2.39.3-r0", size_keyboard: 4096, description: "System utilities", deps: &[] },
    PackageInformation { name: "procps", version: "4.0.4-r0", size_keyboard: 480, description: "Process monitoring utilities", deps: &[] },
    PackageInformation { name: "shadow", version: "4.14.3-r0", size_keyboard: 480, description: "User/group management", deps: &[] },
    PackageInformation { name: "e2fsprogs", version: "1.47.0-r5", size_keyboard: 2048, description: "Ext2/3/4 filesystem utilities", deps: &[] },
    // Compression
    PackageInformation { name: "gzip", version: "1.13-r0", size_keyboard: 96, description: "GNU zip compression", deps: &[] },
    PackageInformation { name: "bzip2", version: "1.0.8-r6", size_keyboard: 128, description: "Block-sorting compressor", deps: &[] },
    PackageInformation { name: "xz", version: "5.4.5-r0", size_keyboard: 256, description: "XZ Utils compression", deps: &[] },
    PackageInformation { name: "zstd", version: "1.5.5-r8", size_keyboard: 384, description: "Zstandard compression", deps: &[] },
    PackageInformation { name: "zip", version: "3.0-r12", size_keyboard: 192, description: "Create ZIP archives", deps: &[] },
    PackageInformation { name: "unzip", version: "6.0-r14", size_keyboard: 192, description: "Extract ZIP archives", deps: &[] },
    PackageInformation { name: "p7zip", version: "17.05-r0", size_keyboard: 2048, description: "7-Zip file archiver", deps: &[] },
    // Media
    PackageInformation { name: "ffmpeg", version: "6.1.1-r0", size_keyboard: 20480, description: "Complete multimedia framework", deps: &[] },
    PackageInformation { name: "imagemagick", version: "7.1.1-r0", size_keyboard: 15360, description: "Image manipulation tools", deps: &[] },
    PackageInformation { name: "mpv", version: "0.37.0-r0", size_keyboard: 5120, description: "Media player", deps: &[] },
    // Modern CLI tools
    PackageInformation { name: "ripgrep", version: "14.1.0-r0", size_keyboard: 6144, description: "Fast recursive grep alternative (rg)", deps: &[] },
    PackageInformation { name: "fd", version: "9.0.0-r0", size_keyboard: 3072, description: "Simple fast alternative to find", deps: &[] },
    PackageInformation { name: "bat", version: "0.24.0-r0", size_keyboard: 5120, description: "Cat clone with syntax highlighting", deps: &[] },
    PackageInformation { name: "exa", version: "0.10.1-r3", size_keyboard: 1536, description: "Modern replacement for ls", deps: &[] },
    PackageInformation { name: "fzf", version: "0.44.1-r0", size_keyboard: 3072, description: "Fuzzy finder", deps: &[] },
    PackageInformation { name: "dust", version: "0.8.6-r0", size_keyboard: 2048, description: "Intuitive version of du", deps: &[] },
    PackageInformation { name: "hyperfine", version: "1.18.0-r0", size_keyboard: 2048, description: "Command-line benchmarking tool", deps: &[] },
    PackageInformation { name: "tokei", version: "12.1.2-r4", size_keyboard: 3072, description: "Code statistics tool", deps: &[] },
    // VCS
    PackageInformation { name: "mercurial", version: "6.6.3-r0", size_keyboard: 7680, description: "Mercurial version control", deps: &[] },
    PackageInformation { name: "subversion", version: "1.14.3-r0", size_keyboard: 5120, description: "Subversion version control", deps: &[] },
    PackageInformation { name: "fossil", version: "2.23-r0", size_keyboard: 3072, description: "Fossil version control", deps: &[] },
    // Misc
    PackageInformation { name: "htop", version: "3.3.0-r0", size_keyboard: 216, description: "Interactive process viewer", deps: &["ncurses-libs"] },
    PackageInformation { name: "neofetch", version: "7.1.0-r3", size_keyboard: 76, description: "CLI system information tool", deps: &["bash"] },
    PackageInformation { name: "tree", version: "2.1.1-r0", size_keyboard: 48, description: "Directory listing in tree-like format", deps: &[] },
    PackageInformation { name: "jq", version: "1.7.1-r0", size_keyboard: 312, description: "Lightweight JSON processor", deps: &[] },
    PackageInformation { name: "tmux", version: "3.3a-r5", size_keyboard: 424, description: "Terminal multiplexer", deps: &["ncurses-libs"] },
    PackageInformation { name: "screen", version: "4.9.1-r0", size_keyboard: 640, description: "Terminal multiplexer", deps: &[] },
    PackageInformation { name: "bc", version: "1.07.1-r4", size_keyboard: 128, description: "Calculator language", deps: &[] },
    PackageInformation { name: "ncdu", version: "2.3-r0", size_keyboard: 192, description: "NCurses disk usage", deps: &[] },
    PackageInformation { name: "ranger", version: "1.9.3-r6", size_keyboard: 640, description: "Console file manager", deps: &[] },
    PackageInformation { name: "mc", version: "4.8.31-r0", size_keyboard: 3072, description: "Midnight Commander file manager", deps: &[] },
    PackageInformation { name: "cowsay", version: "3.04-r2", size_keyboard: 24, description: "Talking cow", deps: &[] },
    PackageInformation { name: "figlet", version: "2.2.5-r3", size_keyboard: 128, description: "Large text banners", deps: &[] },
    PackageInformation { name: "sl", version: "5.05-r0", size_keyboard: 24, description: "Steam locomotive", deps: &[] },
    PackageInformation { name: "fortune", version: "0.1-r2", size_keyboard: 1024, description: "Fortune cookie program", deps: &[] },
    PackageInformation { name: "py3-pip", version: "23.3.2-r0", size_keyboard: 5120, description: "Python package installer", deps: &["python3"] },
    PackageInformation { name: "certbot", version: "2.8.0-r0", size_keyboard: 3072, description: "ACME client for Let's Encrypt", deps: &[] },
    PackageInformation { name: "fail2ban", version: "1.0.2-r0", size_keyboard: 2048, description: "Intrusion prevention", deps: &[] },
];

fn find_package(name: &str) -> Option<&'static PackageInformation> {
    REPO_PACKAGES.iter().find(|p| p.name == name)
}

/// Linux Subsystem manager
pub struct LinuxSubsystem {
    /// Current state
    state: LinuxState,
    /// VM ID for the Linux guest
    vm_id: u64,
    /// Boot parameters
    boot_params: LinuxBootParams,
    /// Virtio console for communication
    console: Option<VirtioConsole>,
    /// Command queue
    pending_commands: Vec<String>,
    /// Last command result
    last_result: Option<CommandResult>,
    /// Boot time
    boot_time: u64,
    /// Embedded Linux kernel (bzImage)
    embedded_kernel: Option<&'static [u8]>,
    /// Embedded initramfs
    embedded_initramfs: Option<&'static [u8]>,
    /// Installed packages (simulated)
    installed_packages: Vec<(&'static str, &'static str)>,   // (name, version)
    /// Whether repo index was fetched
    repo_updated: bool,
    /// Online mode: real HTTP downloads from package server
    online_mode: bool,
    /// Package server URL for real downloads (e.g. http://10.0.2.2:8080)
    pkg_server: Option<String>,
    /// Count of files extracted from real downloads
    real_files_installed: u32,
    /// Total bytes downloaded
    total_bytes_downloaded: usize,
}

// Bloc d'implémentation — définit les méthodes du type ci-dessus.
impl LinuxSubsystem {
    pub const fn new() -> Self {
        Self {
            state: LinuxState::NotStarted,
            vm_id: LINUX_VM_ID,
            boot_params: LinuxBootParams {
                kernel_path: None,
                initramfs_path: None,
                cmdline: String::new(),
                memory_mb: LINUX_VM_MEMORY_MB,
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

    /// Get current state
    pub fn state(&self) -> LinuxState {
        self.state
    }

    /// Check if Linux is ready
    pub fn is_ready(&self) -> bool {
        self.state == LinuxState::Ready
    }

    /// Check if a package is installed
    pub fn is_package_installed(&self, name: &str) -> bool {
        self.installed_packages.iter().any(|(n, _)| *n == name)
    }

    /// Check if the network is available for real package downloads.
    /// Check if network is ready. Starts DHCP if a NIC is present.
    fn check_network(&self) -> bool {
        // Already bound?
        if crate::netstack::dhcp::is_bound() {
            return true;
        }
        // Is there a real NIC driver active?
        let has_e1000 = crate::drivers::net::has_driver();
        let has_virtio = crate::virtio_net::is_initialized();
        if !has_e1000 && !has_virtio {
            return false;
        }
        // Start DHCP
        crate::netstack::dhcp::start();
        // Wait up to 3 seconds
        let start = crate::logger::get_ticks();
                // Boucle infinie — tourne jusqu'à un `break` explicite.
loop {
            crate::netstack::poll();
            if crate::netstack::dhcp::is_bound() {
                return true;
            }
            let elapsed = crate::logger::get_ticks().saturating_sub(start);
            if elapsed > 3000 {
                return false;
            }
            for _ in 0..10000 { core::hint::spin_loop(); }
        }
    }

    /// Detect package server: try DHCP gateway, QEMU user-net, VirtualBox host-only
    fn detect_pkg_server(&self) -> Option<String> {
        use alloc::vec::Vec;
        let mut candidates: Vec<String> = Vec::new();

        // 1. Try DHCP gateway (works for both QEMU SLIRP and VBox NAT)
        if let Some((_ip, _mask, gw, _dns)) = crate::netstack::dhcp::get_config() {
            if gw != [0, 0, 0, 0] {
                candidates.push(alloc::format!("http://{}.{}.{}.{}:8080", gw[0], gw[1], gw[2], gw[3]));
            }
        }
        // 2. Well-known addresses
        candidates.push(String::from("http://10.0.2.2:8080"));
        candidates.push(String::from("http://192.168.56.1:8080"));

        // De-duplicate
        candidates.dedup();

        for server in &candidates {
            let url = alloc::format!("{}/repo/index", server);
            crate::serial_println!("[TSL-PKG] Trying server: {}", url);
            if let Ok(response) = crate::netstack::http::get(&url) {
                if response.status_code == 200 && response.body.len() > 10 {
                    crate::serial_println!("[TSL-PKG] Server found: {} ({} bytes index)",
                        server, response.body.len());
                    return Some(server.clone());
                }
            }
        }
        None
    }

    /// Download package bundle from server, returns raw bytes
    fn download_package(&self, server: &str, name: &str) -> Option<Vec<u8>> {
        let url = alloc::format!("{}/repo/pool/{}.pkg", server, name);
        crate::serial_println!("[TSL-PKG] Downloading: {}", url);
                // Correspondance de motifs — branchement exhaustif de Rust.
match crate::netstack::http::get(&url) {
            Ok(response) if response.status_code == 200 && !response.body.is_empty() => {
                crate::serial_println!("[TSL-PKG] Downloaded {} bytes for {}", response.body.len(), name);
                Some(response.body)
            }
            Ok(response) => {
                crate::serial_println!("[TSL-PKG] Server returned {} for {}", response.status_code, name);
                None
            }
            Err(e) => {
                crate::serial_println!("[TSL-PKG] Download failed for {}: {}", name, e);
                None
            }
        }
    }

    /// Extract package bundle into ramfs.
    /// Bundle format:
    ///   PKG <name> <version>
    ///   FILE <path>
    ///   <content lines...>
    ///   EOF
    /// Returns number of files extracted.
    fn extract_package_to_ramfs(&mut self, data: &[u8]) -> u32 {
        let text = // Correspondance de motifs — branchement exhaustif de Rust.
match core::str::from_utf8(data) {
            Ok(t) => t,
            Err(_) => return 0,
        };

        let mut files_installed = 0u32;
        let mut current_path: Option<&str> = None;
        let mut current_content = String::new();

        for line in text.lines() {
            if line.starts_with("PKG ") {
                // Package header — skip
                continue;
            } else if line.starts_with("FILE /") {
                // Flush previous file
                if let Some(path) = current_path {
                    if self.install_file_to_ramfs(path, current_content.as_bytes()) {
                        files_installed += 1;
                    }
                }
                current_path = Some(&line[5..]);
                current_content = String::new();
            } else if line == "EOF" {
                // Flush last file
                if let Some(path) = current_path {
                    if self.install_file_to_ramfs(path, current_content.as_bytes()) {
                        files_installed += 1;
                    }
                }
                break;
            } else {
                // Content line
                if current_path.is_some() {
                    if !current_content.is_empty() {
                        current_content.push('\n');
                    }
                    current_content.push_str(line);
                }
            }
        }

        self.real_files_installed += files_installed;
        files_installed
    }

    /// Install a single file to ramfs, creating parent directories as needed
    fn install_file_to_ramfs(&self, path: &str, content: &[u8]) -> bool {
        crate::ramfs::with_filesystem(|fs| {
            // Create parent directories recursively
            let mut directory = String::new();
            let parts: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();
            if parts.len() > 1 {
                for part in &parts[..parts.len() - 1] {
                    directory.push('/');
                    directory.push_str(part);
                    let _ = fs.mkdir(&directory);
                }
            }
            // Create and write the file
            if fs.touch(path).is_ok() {
                if fs.write_file(path, content).is_ok() {
                    crate::serial_println!("[TSL-PKG] Installed: {} ({} bytes)", path, content.len());
                    return true;
                }
            }
            false
        })
    }

    /// Whether we're in online download mode
    pub fn is_online(&self) -> bool {
        self.online_mode
    }

    /// Total bytes downloaded from package server
    pub fn bytes_downloaded(&self) -> usize {
        self.total_bytes_downloaded
    }
    
    /// Check if kernel is loaded
    pub fn has_kernel(&self) -> bool {
        self.embedded_kernel.is_some()
    }
    
    /// Get kernel size
    pub fn kernel_size(&self) -> usize {
        self.embedded_kernel.map(|k| k.len()).unwrap_or(0)
    }
    
    /// Check if initramfs is loaded
    pub fn has_initramfs(&self) -> bool {
        self.embedded_initramfs.is_some()
    }
    
    /// Get initramfs size
    pub fn initramfs_size(&self) -> usize {
        self.embedded_initramfs.map(|i| i.len()).unwrap_or(0)
    }
    
    /// Get kernel version string
    pub fn kernel_version_string(&self) -> Option<String> {
        let kernel = self.embedded_kernel?;
        if kernel.len() < 0x210 {
            return None;
        }
        
        let kernel_version_offset = u16::from_le_bytes([kernel[0x20E], kernel[0x20F]]) as usize;
        if kernel_version_offset == 0 || kernel_version_offset + 0x200 >= kernel.len() {
            return None;
        }
        
        let version_start = kernel_version_offset + 0x200;
        let mut version_str = alloc::vec::Vec::new();
        for i in 0..80 {
            if version_start + i >= kernel.len() {
                break;
            }
            let c = kernel[version_start + i];
            if c == 0 {
                break;
            }
            version_str.push(c);
        }
        
        core::str::from_utf8(&version_str).ok().map(|s| String::from(s))
    }
    
    /// Get boot protocol version
    pub fn boot_protocol_version(&self) -> Option<(u8, u8)> {
        let kernel = self.embedded_kernel?;
        if kernel.len() < 0x208 {
            return None;
        }
        
        let magic = u32::from_le_bytes([
            kernel[0x202],
            kernel[0x203],
            kernel[0x204],
            kernel[0x205],
        ]);
        
        if magic != 0x53726448 {
            return None;
        }
        
        let version = u16::from_le_bytes([kernel[0x206], kernel[0x207]]);
        Some(((version >> 8) as u8, (version & 0xFF) as u8))
    }

    /// Initialize the Linux subsystem
    pub fn init(&mut self) -> Result<()> {
        if self.state != LinuxState::NotStarted {
            return Err(HypervisorError::AlreadyRunning);
        }

        crate::serial_println!("[TSL] Initializing TrustOS Subsystem for Linux...");
        
        // Create virtio console
        self.console = Some(VirtioConsole::new(self.vm_id));
        
        // Set default cmdline
        self.boot_params.cmdline = String::from("console=hvc0 quiet init=/init");
        
        self.state = LinuxState::NotStarted;
        
        crate::serial_println!("[TSL] Subsystem initialized (waiting for kernel)");
        
        Ok(())
    }

    /// Set embedded kernel and initramfs
    pub fn set_embedded_images(&mut self, kernel: &'static [u8], initramfs: &'static [u8]) {
        self.embedded_kernel = Some(kernel);
        self.embedded_initramfs = Some(initramfs);
        
        // Verify kernel magic and extract version info
        if kernel.len() >= 0x210 {
            let magic = u32::from_le_bytes([
                kernel[0x202],
                kernel[0x203],
                kernel[0x204],
                kernel[0x205],
            ]);
            
            if magic == 0x53726448 {  // "HdrS"
                let version = u16::from_le_bytes([kernel[0x206], kernel[0x207]]);
                crate::serial_println!("[TSL] Linux kernel: {} bytes, boot protocol v{}.{}", 
                    kernel.len(), version >> 8, version & 0xFF);
            } else {
                crate::serial_println!("[TSL] Warning: Invalid kernel magic: {:#X}", magic);
            }
        }
        
        crate::serial_println!("[TSL] Initramfs: {} bytes", initramfs.len());
        crate::serial_println!("[TSL] Total Linux guest size: {} KB", 
            (kernel.len() + initramfs.len()) / 1024);
    }

    /// Boot the Linux VM
    pub fn boot(&mut self) -> Result<()> {
        if self.state == LinuxState::Ready || self.state == LinuxState::Booting {
            return Ok(());
        }

        crate::serial_println!("[TSL] Booting Linux VM...");
        self.state = LinuxState::Booting;

        // Check if we have a kernel
        let kernel_data = // Correspondance de motifs — branchement exhaustif de Rust.
match self.embedded_kernel {
            Some(k) => k,
            None => {
                crate::serial_println!("[TSL] No kernel image available");
                crate::serial_println!("[TSL] Falling back to simulated mode");
                self.state = LinuxState::Ready;
                return Ok(());
            }
        };

        // Verify kernel magic
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

        if magic != 0x53726448 {  // "HdrS"
            crate::serial_println!("[TSL] Invalid kernel magic: {:#X} (expected HdrS)", magic);
            crate::serial_println!("[TSL] Falling back to simulated mode");
            self.state = LinuxState::Ready;
            return Ok(());
        }

        let version = u16::from_le_bytes([kernel_data[0x206], kernel_data[0x207]]);
        crate::serial_println!("[TSL] Linux boot protocol version: {}.{}", 
            version >> 8, version & 0xFF);
            
        // Extract more kernel info
        let setup_sects = kernel_data[0x1F1];
        let loadflags = kernel_data[0x211];
        let kernel_version_offset = u16::from_le_bytes([kernel_data[0x20E], kernel_data[0x20F]]) as usize;
        
        crate::serial_println!("[TSL] Setup sectors: {}", setup_sects);
        crate::serial_println!("[TSL] Load flags: {:#X}", loadflags);
        
        // Try to read kernel version string
        if kernel_version_offset > 0 && kernel_version_offset + 0x200 < kernel_data.len() {
            let version_start = kernel_version_offset + 0x200;
            let mut version_str = alloc::vec::Vec::new();
            for i in 0..64 {
                if version_start + i >= kernel_data.len() {
                    break;
                }
                let c = kernel_data[version_start + i];
                if c == 0 {
                    break;
                }
                version_str.push(c);
            }
            if !version_str.is_empty() {
                if let Ok(s) = core::str::from_utf8(&version_str) {
                    crate::serial_println!("[TSL] Kernel version: {}", s);
                }
            }
        }
        
        // Check initramfs
        if let Some(initrd) = self.embedded_initramfs {
            crate::serial_println!("[TSL] Initramfs: {} bytes ({} KB)", 
                initrd.len(), initrd.len() / 1024);
        }

        // Try to boot the real Linux kernel
        if kernel_data.len() > 0x1000 {
            // We have a kernel, try real boot
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

        // Fallback to simulated mode
        self.state = LinuxState::Ready;
        crate::serial_println!("[TSL] Linux VM ready (simulated mode)");

        Ok(())
    }
    
    /// Boot the real Linux kernel using the hypervisor
    fn boot_real_linux(&mut self) -> Result<()> {
        let kernel_data = self.embedded_kernel.ok_or(HypervisorError::InvalidState)?;
        let initramfs_data = self.embedded_initramfs.ok_or(HypervisorError::InvalidState)?;
        
        // Use the linux_vm module to boot
        let cmdline = &self.boot_params.cmdline;
        let vm_id = super::linux_vm::boot_linux(kernel_data, initramfs_data, cmdline)?;
        
        crate::serial_println!("[TSL] Linux VM started with ID: {}", vm_id);
        
        Ok(())
    }

    /// Execute a command in the Linux VM
    pub fn execute(&mut self, command: &str) -> Result<CommandResult> {
        if self.state != LinuxState::Ready {
            // Try to boot if not ready
            if self.state == LinuxState::NotStarted {
                self.init()?;
            }
            
            // If still not ready, simulate
            if self.state != LinuxState::Ready {
                return self.simulate_command(command);
            }
        }

        self.state = LinuxState::Busy;

        // Send command to VM via virtio-console
        if let Some(ref mut console) = self.console {
            let command_line = format!("{}\n", command);
            console.write(command_line.as_bytes());
        }

        // For now, simulate the response
        let result = self.simulate_command(command)?;

        self.state = LinuxState::Ready;
        self.last_result = Some(result.clone());

        Ok(result)
    }

    /// Simulate command execution (for testing without real Linux VM)
    fn simulate_command(&mut self, command: &str) -> Result<CommandResult> {
        let parts: Vec<&str> = command.split_whitespace().collect();
        
        if parts.is_empty() {
            return Ok(CommandResult::success(String::new()));
        }

        let cmd = parts[0];
        let args = &parts[1..];

        let output = // Correspondance de motifs — branchement exhaustif de Rust.
match cmd {
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
                    for (name, ver) in &self.installed_packages {
                        out.push_str(&format!("ii  {:<24}{:<17}x86_64       {}\n", name, ver,
                            find_package(name).map(|p| p.description).unwrap_or("")));
                    }
                    return Ok(CommandResult::success(out));
                }
                return Ok(CommandResult::error(1, String::from("dpkg: use apt-get to manage packages")));
            }
            "which" => {
                if args.is_empty() {
                    return Ok(CommandResult::error(1, String::from("which: missing argument")));
                }
                let bin = args[0];
                // Check if it's an installed package
                if self.installed_packages.iter().any(|(n, _)| *n == bin) {
                    return Ok(CommandResult::success(format!("/usr/bin/{}", bin)));
                }
                // Built-in commands
                let builtins = ["ls", "cat", "echo", "whoami", "pwd", "date", "uname", "sh", "ash"];
                if builtins.contains(&bin) {
                    return Ok(CommandResult::success(format!("/bin/{}", bin)));
                }
                return Ok(CommandResult::error(1, format!("{} not found", bin)));
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

    /// Simulate apt-get / apt package manager
    fn simulate_apt(&mut self, args: &[&str]) -> Result<CommandResult> {
        if args.is_empty() {
            return Ok(CommandResult::error(1, String::from(
                "Usage: apt-get [update|install|remove|list|search|upgrade] [packages...]")));
        }

        let subcmd = args[0];
        let pkg_args = &args[1..];

                // Correspondance de motifs — branchement exhaustif de Rust.
match subcmd {
            "update" => {
                let mut out = String::new();

                // Try real network fetch if network is available
                if self.check_network() {
                    if let Some(server) = self.detect_pkg_server() {
                        out.push_str(&format!("Connected to package server: {}\n", server));
                        out.push_str(&format!("Get:1 {}/repo/index Packages [online]\n", server));
                        self.pkg_server = Some(server);
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
                    // Query server for total available packages
                    let mut total_avail = REPO_PACKAGES.len();
                    if let Some(ref server) = self.pkg_server {
                        let information_url = alloc::format!("{}/repo", server);
                        if let Ok(response) = crate::netstack::http::get(&information_url) {
                            // Parse "total_available" from JSON response
                            if let Some(position) = response.body.windows(17).position(|w| w == b"total_available\":") {
                                let rest = &response.body[position + 17..];
                                let number_end = rest.iter().position(|&b| !b.is_ascii_digit() && b != b' ').unwrap_or(rest.len());
                                let number_str = core::str::from_utf8(&rest[..number_end]).unwrap_or("").trim();
                                if let Ok(n) = number_str.parse::<usize>() {
                                    total_avail = n;
                                }
                            }
                        }
                    }
                    out.push_str(&format!("{} packages available (live).\n", total_avail));
                } else {
                    out.push_str(&format!("{} packages can be upgraded. Run 'apt-get upgrade' to see them.\n",
                        REPO_PACKAGES.len().saturating_sub(self.installed_packages.len()).minimum(8)));
                }
                self.repo_updated = true;
                Ok(CommandResult::success(out))
            }

            "install" | "add" => {
                if pkg_args.is_empty() {
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

                // Resolve all requested packages + deps
                let mut to_install: Vec<&'static PackageInformation> = Vec::new();
                let mut not_found: Vec<&str> = Vec::new();
                let mut dynamic_pkgs: Vec<&str> = Vec::new(); // packages to fetch from server

                for &name in pkg_args {
                    // skip flags like -y
                    if name.starts_with('-') { continue; }
                    if let Some(pkg) = find_package(name) {
                        // Add deps first
                        for dep_name in pkg.deps {
                            if !self.installed_packages.iter().any(|(n, _)| n == dep_name)
                                && !to_install.iter().any(|p| p.name == *dep_name)
                            {
                                if let Some(dep) = find_package(dep_name) {
                                    to_install.push(dep);
                                }
                            }
                        }
                        // Add main package
                        if !self.installed_packages.iter().any(|(n, _)| *n == pkg.name)
                            && !to_install.iter().any(|p| p.name == pkg.name)
                        {
                            to_install.push(pkg);
                        }
                    } else if self.online_mode && self.pkg_server.is_some() {
                        // Package not in local repo — try downloading from server (Alpine CDN proxy)
                        dynamic_pkgs.push(name);
                    } else {
                        not_found.push(name);
                    }
                }

                for nf in &not_found {
                    out.push_str(&format!("E: Unable to locate package {}\n", nf));
                }

                // Try dynamic download for packages not in local repo
                let mut dynamic_installed = 0u32;
                if !dynamic_pkgs.is_empty() {
                    let server = self.pkg_server.clone().unwrap_or_default();
                    for &dyn_name in &dynamic_pkgs {
                        out.push_str(&format!("Resolving {} via package server...\n", dyn_name));
                        if let Some(data) = self.download_package(&server, dyn_name) {
                            let dl_size = data.len();
                            self.total_bytes_downloaded += dl_size;
                            let files = self.extract_package_to_ramfs(&data);
                            if files > 0 {
                                out.push_str(&format!("Get:1 {}/repo/pool/{}.pkg [{} B]\n", server, dyn_name, dl_size));
                                out.push_str(&format!("  -> Downloaded {} bytes, extracted {} files\n", dl_size, files));
                                // Extract version from first line of data ("PKG name version")
                                let ver_str = core::str::from_utf8(&data).unwrap_or("")
                                    .lines().next().unwrap_or("")
                                    .splitn(3, ' ').nth(2).unwrap_or("latest");
                                // Store as installed (we leak the string since installed_packages takes &'static str)
                                let name_box = alloc::boxed::Box::leak(String::from(dyn_name).into_boxed_str());
                                let ver_box = alloc::boxed::Box::leak(String::from(ver_str).into_boxed_str());
                                self.installed_packages.push((name_box, ver_box));
                                dynamic_installed += 1;
                            } else {
                                out.push_str(&format!("E: Unable to locate package {}\n", dyn_name));
                            }
                        } else {
                            out.push_str(&format!("E: Unable to locate package {}\n", dyn_name));
                        }
                    }
                }

                if to_install.is_empty() && dynamic_installed == 0 && not_found.is_empty() && dynamic_pkgs.is_empty() {
                    out.push_str("All requested packages are already installed.\n");
                    return Ok(CommandResult::success(out));
                }

                if to_install.is_empty() && dynamic_installed == 0 {
                    return Ok(CommandResult::error(1, out));
                }

                // Show what will be installed
                let new_names: Vec<&str> = to_install.iter().map(|p| p.name).collect();
                let total_size: u32 = to_install.iter().map(|p| p.size_keyboard).sum();

                out.push_str("The following NEW packages will be installed:\n  ");
                out.push_str(&new_names.join(" "));
                out.push_str("\n");
                out.push_str(&format!("{} newly installed, 0 to remove and 0 not upgraded.\n",
                    to_install.len()));
                out.push_str(&format!("Need to get {} kB of archives.\n", total_size));
                out.push_str(&format!("After this operation, {} kB of additional disk space will be used.\n",
                    total_size * 3));

                // Download packages — real or simulated
                let server = self.pkg_server.clone();
                let is_online = self.online_mode && server.is_some();

                for pkg in &to_install {
                    if is_online {
                        let server = server.as_deref().unwrap();
                        out.push_str(&format!("Get:1 {}/repo/pool/{}.pkg {} [{} kB]\n",
                            server, pkg.name, pkg.version, pkg.size_keyboard));

                        // Real HTTP download
                        if let Some(data) = self.download_package(server, pkg.name) {
                            let dl_size = data.len();
                            self.total_bytes_downloaded += dl_size;
                            let files = self.extract_package_to_ramfs(&data);
                            out.push_str(&format!("  -> Downloaded {} bytes, extracted {} files\n",
                                dl_size, files));
                        } else {
                            out.push_str("  -> Download failed, using cached metadata\n");
                        }
                    } else {
                        out.push_str(&format!("Get:1 http://dl-cdn.alpinelinux.org/alpine/v3.19/main x86_64 {} {} [{} kB]\n",
                            pkg.name, pkg.version, pkg.size_keyboard));
                    }
                }
                if is_online {
                    out.push_str(&format!("Fetched {} bytes from {}\n",
                        self.total_bytes_downloaded, server.as_deref().unwrap()));
                } else {
                    out.push_str(&format!("Fetched {} kB in 0s (internal)\n", total_size));
                }

                // Unpack
                for pkg in &to_install {
                    out.push_str(&format!("Selecting previously unselected package {}.\n", pkg.name));
                    out.push_str(&format!("Preparing to unpack {}_{}_amd64.deb ...\n", pkg.name, pkg.version));
                    out.push_str(&format!("Unpacking {} ({}) ...\n", pkg.name, pkg.version));
                }

                // Configure
                out.push_str("Setting up packages ...\n");
                for pkg in &to_install {
                    out.push_str(&format!("Setting up {} ({}) ...\n", pkg.name, pkg.version));
                    self.installed_packages.push((pkg.name, pkg.version));
                }

                out.push_str("Processing triggers for man-db ...\n");
                if is_online {
                    out.push_str(&format!("[online] {} files installed to filesystem.\n",
                        self.real_files_installed));
                }

                if !not_found.is_empty() {
                    return Ok(CommandResult { exit_code: 1, stdout: out, stderr: String::new(), duration_mouse: 0 });
                }
                Ok(CommandResult::success(out))
            }

            "remove" | "purge" | "del" => {
                if pkg_args.is_empty() {
                    return Ok(CommandResult::error(1, String::from(
                        "E: No packages specified for removal.")));
                }

                let mut out = String::new();
                out.push_str("Reading package lists... Done\n");
                out.push_str("Building dependency tree... Done\n");

                let mut removed = 0u32;
                let mut freed_keyboard = 0u32;
                for &name in pkg_args {
                    if name.starts_with('-') { continue; }
                    if let Some(position) = self.installed_packages.iter().position(|(n, _)| *n == name) {
                        let (pname, pver) = self.installed_packages.remove(position);
                        let size = find_package(pname).map(|p| p.size_keyboard).unwrap_or(100);
                        out.push_str(&format!("Removing {} ({}) ...\n", pname, pver));
                        freed_keyboard += size * 3;
                        removed += 1;
                    } else {
                        out.push_str(&format!("Package '{}' is not installed, so not removed.\n", name));
                    }
                }

                if removed > 0 {
                    out.push_str(&format!("{} packages removed, {} kB disk space freed.\n",
                        removed, freed_keyboard));
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
                    for (name, ver) in &self.installed_packages {
                        let desc = find_package(name).map(|p| p.description).unwrap_or("");
                        out.push_str(&format!("{:<24} {:<24} {}\n", name, ver, desc));
                    }
                    out.push_str(&format!("\n{} packages installed.\n", self.installed_packages.len()));
                }
                Ok(CommandResult::success(out))
            }

            "search" => {
                if pkg_args.is_empty() {
                    return Ok(CommandResult::error(1, String::from("Usage: apt-get search <keyword>")));
                }
                let keyword = pkg_args[0].to_lowercase();
                // keyword is lowercase String, we need to compare against &str
                let mut out = String::new();
                let mut count = 0;
                // Search local repo
                for pkg in REPO_PACKAGES {
                    if pkg.name.contains(keyword.as_str()) || pkg.description.to_lowercase().contains(keyword.as_str()) {
                        let installed = if self.installed_packages.iter().any(|(n, _)| *n == pkg.name) {
                            " [installed]"
                        } else {
                            ""
                        };
                        out.push_str(&format!("{}/{} {} x86_64{}\n  {}\n\n",
                            pkg.name, pkg.version, pkg.size_keyboard, installed, pkg.description));
                        count += 1;
                    }
                }
                // Search server (Alpine CDN proxy) for more results
                if self.online_mode {
                    if let Some(ref server) = self.pkg_server {
                        let search_url = alloc::format!("{}/repo/search?q={}", server, pkg_args[0]);
                        if let Ok(response) = crate::netstack::http::get(&search_url) {
                            if response.status_code == 200 {
                                let text = core::str::from_utf8(&response.body).unwrap_or("");
                                for line in text.lines() {
                                    if line.is_empty() || line == "No results" { continue; }
                                    // Format: "name version size_kb arch nfiles deps description"
                                    let parts: Vec<&str> = line.splitn(7, ' ').collect();
                                    if parts.len() >= 7 {
                                        let pname = parts[0];
                                        // Skip if already shown from local repo
                                        if REPO_PACKAGES.iter().any(|p| p.name == pname) { continue; }
                                        let installed = if self.installed_packages.iter().any(|(n, _)| *n == pname) {
                                            " [installed]"
                                        } else {
                                            ""
                                        };
                                        out.push_str(&format!("{}/{} {} {}{}\n  {}\n\n",
                                            pname, parts[1], parts[2], parts[3], installed, parts[6]));
                                        count += 1;
                                    }
                                }
                            }
                        }
                    }
                }
                if count == 0 {
                    out.push_str(&format!("No packages found matching '{}'.\n", pkg_args[0]));
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
                Ok(CommandResult::error(1, format!("E: Invalid operation '{}'", subcmd)))
            }
        }
    }

    /// Simulate apk (Alpine package manager) — delegates to same logic as apt
    fn simulate_apk(&mut self, args: &[&str]) -> Result<CommandResult> {
        if args.is_empty() {
            return Ok(CommandResult::error(1, String::from(
                "Usage: apk [update|add|del|list|search|info] [packages...]")));
        }

        // Map apk subcommands to apt-get equivalents
        let mapped: &[&str] = // Correspondance de motifs — branchement exhaustif de Rust.
match args[0] {
            "add" => &["install"],
            "del" => &["remove"],
            "info" => &["search"],
            other => &[other],
        };

        let mut full_args = mapped.to_vec();
        full_args.extend_from_slice(&args[1..]);
        self.simulate_apt(&full_args)
    }

    /// Shutdown the Linux VM
    pub fn shutdown(&mut self) -> Result<()> {
        if self.state == LinuxState::NotStarted {
            return Ok(());
        }

        crate::serial_println!("[TSL] Shutting down Linux VM...");
        self.state = LinuxState::ShuttingDown;

        // Send shutdown command
        if let Some(ref mut console) = self.console {
            console.write(b"poweroff\n");
        }

        // Clean up
        self.console = None;
        self.state = LinuxState::NotStarted;

        crate::serial_println!("[TSL] Linux VM stopped");

        Ok(())
    }

    /// Get console for direct access
    pub fn console(&mut self) -> Option<&mut VirtioConsole> {
        self.console.as_mut()
    }
}

/// Global Linux subsystem instance
static LINUX_SUBSYSTEM: Mutex<LinuxSubsystem> = Mutex::new(LinuxSubsystem::new());

/// Initialize the Linux subsystem
pub fn init() -> Result<()> {
    LINUX_SUBSYSTEM.lock().init()
}

/// Boot the Linux VM
pub fn boot() -> Result<()> {
    LINUX_SUBSYSTEM.lock().boot()
}

/// Execute a command in the Linux VM
pub fn execute(command: &str) -> Result<CommandResult> {
    LINUX_SUBSYSTEM.lock().execute(command)
}

/// Check if Linux subsystem is ready
pub fn is_ready() -> bool {
    LINUX_SUBSYSTEM.lock().is_ready()
}

/// Get current state
pub fn state() -> LinuxState {
    LINUX_SUBSYSTEM.lock().state()
}

/// Shutdown the Linux VM
pub fn shutdown() -> Result<()> {
    LINUX_SUBSYSTEM.lock().shutdown()
}

/// Get the Linux subsystem for direct access
pub fn subsystem() -> spin::MutexGuard<'static, LinuxSubsystem> {
    LINUX_SUBSYSTEM.lock()
}
