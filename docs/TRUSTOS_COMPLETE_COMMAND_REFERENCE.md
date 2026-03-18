# TrustOS Complete Command & Feature Reference
## For JARVIS AI Training Corpus

**Version**: 0.2.0  
**Language**: 100% Pure Rust (zero C, zero libc, zero std)  
**Architectures**: x86_64, ARM64/aarch64, RISC-V (rv64gc)  
**Codebase**: ~273,000 lines of Rust  
**Shell**: `tsh` (TrustOS Shell)  
**Test Coverage**: 95/96 tests passing (99%)

---

## 1. FILESYSTEM COMMANDS

| Command | Aliases | Description |
|---------|---------|-------------|
| `ls [path]` | `dir` | List directory contents. `-l` long format, `-a` show hidden |
| `cd <path>` | | Change directory. `~` = home, `..` = parent, `/` = root |
| `pwd` | | Print current working directory |
| `mkdir <dir>` | | Create directory. `-p` creates parent dirs recursively |
| `rmdir <dir>` | | Remove empty directory |
| `touch <file>` | | Create empty file or update timestamp |
| `rm <file>` | `del` | Remove file. `-r` recursive delete for directories |
| `cp <src> <dst>` | `copy` | Copy file or directory |
| `mv <src> <dst>` | `move`, `rename` | Move or rename file/directory |
| `cat <file>` | `type` | Display file contents. Supports `> redirect` and piped input |
| `head <file>` | | Show first N lines (default 10). `-n N` to specify count |
| `tail <file>` | | Show last N lines (default 10). `-n N` to specify count |
| `wc <file>` | | Count lines, words, and bytes |
| `stat <file>` | | Display file metadata (size, type, permissions) |
| `tree [path]` | | Display directory tree structure recursively |
| `find <path> <name>` | | Search for files by name pattern |
| `ln -s <target> <link>` | | Create symbolic link |
| `readlink <link>` | | Display symlink target |
| `basename <path>` | | Extract filename from path |
| `dirname <path>` | | Extract directory from path |
| `realpath <path>` | | Resolve absolute canonical path |
| `file <path>` | | Identify file type (ELF, PE, MachO, text, binary, etc.) |
| `chmod <mode> <file>` | | Change file permissions (octal, e.g. 755) |
| `chown <user>[:<group>] <file>` | | Change file ownership |
| `du [path]` | | Show disk usage for files/directories |

### VFS Paths
- `/` — RAMFS root (in-memory filesystem)
- `/mnt/` — Mount points for real disks
- `/mnt/fat32/` — FAT32 partition mount
- `/dev/` — Device filesystem (DevFS)
- `/proc/` — Process info filesystem (ProcFS)
- `/home/` — User home directories
- `/etc/` — System configuration
- `/alpine/` — Alpine Linux rootfs
- `/jarvis/` — JARVIS brain weights & data

### Supported Filesystems
- **RAMFS** — In-memory (default, always available)
- **FAT32** — DOS-compatible (real disk)
- **ext4** — Linux native (full support)
- **ext2/ext3** — Legacy Linux support
- **TrustFS** — Custom native filesystem
- **ProcFS** — Virtual `/proc` info
- **DevFS** — Virtual `/dev` devices

---

## 2. TEXT PROCESSING COMMANDS

| Command | Description |
|---------|-------------|
| `echo <text>` | Print text. Supports `> file` redirect and `>> file` append |
| `grep <pattern> <file>` | Search for pattern in file. `-i` case insensitive. Supports piped input |
| `sort <file>` | Sort lines alphabetically. `-r` reverse, `-n` numeric |
| `uniq <file>` | Remove duplicate adjacent lines. `-c` prefix with count |
| `cut -d<delim> -f<fields>` | Cut columns by delimiter. Supports piped input |
| `tr <from> <to>` | Translate/replace characters. Supports piped input |
| `tee <file>` | Write stdin to file AND stdout simultaneously |
| `rev <text>` | Reverse a string |
| `diff <a> <b>` | Compare two files line by line (unified diff) |
| `cmp <a> <b>` | Compare two files byte by byte |
| `patch <file>` | Apply a diff patch to a file |
| `strings <file>` | Extract printable strings from binary files |
| `od <file>` | Octal dump of file contents |
| `hexdump <file>` / `xxd` | Hex dump of file contents |
| `base64 <file>` | Encode file to base64. `-d` to decode. Supports piped input |
| `md5sum <file>` | Compute MD5 hash of file |
| `sha256sum <file>` | Compute SHA-256 hash of file |

---

## 3. SYSTEM & PROCESS MANAGEMENT

| Command | Aliases | Description |
|---------|---------|-------------|
| `clear` | `cls` | Clear terminal screen |
| `time` | `uptime` | Display system uptime (ticks, seconds) |
| `date` | | Show current date and time (from RTC) |
| `whoami` | | Print current username |
| `hostname` | | Display system hostname ("trustos") |
| `id` | | Print user/group IDs |
| `uname [-a]` | | System info: kernel name, version, architecture |
| `env` | `printenv` | Display all environment variables |
| `export KEY=VALUE` | | Set an environment variable |
| `unset <VAR>` | | Remove an environment variable |
| `set [VAR=value]` | | Show/set shell variables |
| `alias <name>=<cmd>` | | Create command alias |
| `unalias <name>` | | Remove command alias |
| `source <file>` | `.` | Execute commands from a script file |
| `history` | | Show last 50 commands in history |
| `ps` | | List running processes (PID, name, state, memory) |
| `top` | `htop` | Real-time process monitor (live updating) |
| `tasks` | `jobs` | List active kernel tasks |
| `threads` | | Show kernel thread information |
| `kill <pid>` | | Send signal to terminate a process |
| `killall <name>` | | Kill all processes matching name |
| `nice <n> <cmd>` | | Run command with priority level N |
| `nohup <cmd>` | | Run command immune to hangups |
| `bg` / `fg` | | Background/foreground job control |
| `free` | | Display memory usage (heap stats) |
| `df` | | Show disk space usage by mount point |
| `vmstat` | | Virtual memory statistics |
| `iostat` | | I/O statistics by device |
| `lsof [pid]` | | List open files per process |
| `strace <cmd>` | | Trace system calls of a command |
| `sleep <secs>` | | Pause execution for N seconds |
| `watch <cmd>` | | Execute command repeatedly (every 2s) |
| `timeout <sec> <cmd>` | | Run command with time limit |
| `which <cmd>` | | Show command location (built-in check) |
| `whereis <cmd>` | | Locate command binary and manpage |
| `script <file>` | | Record terminal session to file |
| `timecmd <cmd>` | | Measure command execution time (μs/ms) |

### Shell Features
- **Tab completion** — Auto-complete commands and paths
- **Up/Down arrows** — Navigate command history
- **PageUp/PageDown** — Scroll terminal output
- **Pipe support** — `cmd1 | cmd2` chains commands
- **Redirect** — `>` (overwrite) and `>>` (append) to files
- **Shell scripting** — Variables, `if/then/else`, `for/while` loops

---

## 4. USER MANAGEMENT

| Command | Aliases | Description |
|---------|---------|-------------|
| `login` | | Switch to another user account (prompts username/password) |
| `su <user>` | | Substitute user identity |
| `passwd [user]` | | Change password (current user or specified user) |
| `adduser <name>` | `useradd` | Create new user account. `-a` for admin privileges |
| `deluser <name>` | `userdel` | Delete user account |
| `users` | | List all user accounts (uid, gid, role) |

### Auth System
- SAM database (Security Account Manager)
- Default account: `root` (password: `trustos`)
- Capability-based access control
- Role-based security policies

---

## 5. HARDWARE & DEVICE COMMANDS

| Command | Aliases | Description |
|---------|---------|-------------|
| `lspci [-v]` | | List PCI devices (vendor, device, class). `-v` verbose |
| `lshw` | `hwinfo` | Full hardware inventory (CPU, RAM, disks, NICs, GPU) |
| `lscpu` | | CPU model, cores, features, frequency, CPUID flags |
| `lsmem` | | Memory layout and total RAM |
| `lsusb` | | List USB controllers and devices (xHCI) |
| `lsblk` | | List block devices (disks, partitions) |
| `blkid` | | Show block device UUIDs and filesystem types |
| `lsmod` | | List loaded kernel modules |
| `insmod <module>` | | Insert/load kernel module |
| `rmmod <module>` | | Remove kernel module |
| `smpstatus` | | SMP multicore status (per-CPU state) |
| `smp <cmd>` | | SMP control: `start`, `stop` individual cores |
| `dmidecode` | | BIOS/SMBIOS firmware tables |
| `hdparm <dev>` | | Disk drive parameters |
| `checkm8` | | Apple SecureROM exploit (Apple Silicon) |

### HwDbg — Universal Hardware Debugger

PXE-boot TrustOS on any machine and run comprehensive hardware diagnostics.
All output dual-mirrored to screen + serial (115200 8N1).

| Command | Aliases | Description |
|---------|---------|-------------|
| `hwdbg auto` | `hwdbg all`, `hwdbg full` | Run ALL diagnostics (full machine profile) |
| `hwdbg cpu` | | Deep CPU analysis (CPUID, topology, µcode) |
| `hwdbg mem [size_mb]` | `hwdbg memory` | Memory test (walking 1s, patterns, stress) |
| `hwdbg pci [B:D.F]` | | Full PCI tree with BARs & capabilities |
| `hwdbg acpi` | | Dump all ACPI tables (RSDP→XSDT→*) |
| `hwdbg storage` | `hwdbg disk`, `hwdbg nvme` | Disk/NVMe detection & identify |
| `hwdbg thermal` | `hwdbg temp`, `hwdbg power` | Thermal sensors, power state, fans |
| `hwdbg net` | `hwdbg network`, `hwdbg nic` | Network interface inventory |
| `hwdbg stress [secs]` | `hwdbg burn` | CPU + memory stress test |
| `hwdbg remote` | `hwdbg serial` | Start structured serial debug protocol |
| `hwdbg pciraw <B:D.F>` | `hwdbg rawpci` | Raw hex dump of PCI config space (256B/4KB) |
| `hwdbg regdiff snap [name]` | `hwdbg rdiff` | Snapshot registers (PCI+MSR+IO), then `regdiff diff` to compare |
| `hwdbg ioscan [mode]` | `hwdbg io` | Scan I/O port ranges (`legacy`, `com`, `ide`, or hex range) |
| `hwdbg regwatch <type>` | `hwdbg watch` | Live register monitor: `pci`, `msr`, or `io` |
| `hwdbg aer [B:D.F]` | | PCIe Advanced Error Reporting (scan, decode, clear) |
| `hwdbg timing [slow]` | `hwdbg boottiming` | Boot timing profiler (checkpoint deltas) |
| `hwdbg export` | `hwdbg dump` | Full report in machine-parseable format |
| `hwdbg help` | | Show all hwdbg subcommands |

### Hardware Detection
- **CPU**: Vendor (Intel/AMD/ARM), model, cores, features (AES-NI, AVX, SSE, etc.)
- **Memory**: Total RAM, usable regions, memory map
- **Disks**: AHCI/SATA, NVMe, VirtIO block
- **Network**: VirtIO-net, Intel e1000
- **GPU**: AMD RDNA (compute + display), VirtIO-GPU
- **Audio**: Intel HD Audio (HDA), PC speaker
- **USB**: xHCI host controller
- **PCI**: Full bus enumeration with vendor/device IDs

---

## 6. DISK & STORAGE COMMANDS

| Command | Description |
|---------|-------------|
| `disk` | Show all detected disk drives and controllers |
| `dd if=<src> of=<dst>` | Block-level copy (raw disk I/O) |
| `ahci <cmd>` | AHCI SATA controller commands (identify, read, write) |
| `fdisk <dev>` | Partition table editor (MBR/GPT) |
| `mkfs <type> <dev>` | Format partition (fat32, ext2, ext4) |
| `fsck <dev>` | Filesystem consistency check |
| `mount <dev> <dir>` | Mount filesystem at directory |
| `umount <dir>` | Unmount mounted filesystem |
| `sync` | Flush all pending writes to disk |
| `nvme` | NVMe SSD controller info and commands |
| `persist <cmd>` | Manage persistent storage (save/load data across reboots) |

---

## 7. NETWORK COMMANDS

### TCP/IP Stack (Built From Scratch)
Full custom implementation: ARP, ICMP, ICMPv6, IP, IPv6, TCP, UDP, DHCP, DNS, HTTP, HTTPS (TLS 1.3)

| Command | Aliases | Description |
|---------|---------|-------------|
| `ifconfig` | `ip` | Show network interface status (IP, MAC, link state) |
| `ipconfig [cmd]` | | Configure IP settings (dhcp, static, dns) |
| `ping <host>` | | ICMP echo connectivity test (4 packets, RTT stats) |
| `curl <url>` | `wget` | HTTP/HTTPS client (GET, POST with data) |
| `download <url>` | | Download and save file from URL |
| `nslookup <host>` | `dig` | DNS lookup (A, AAAA records) |
| `arp [-a]` | | Show ARP table (IP → MAC address mappings) |
| `route` | | Display IP routing table |
| `traceroute <host>` | `tracert` | Real TTL-based traceroute with ICMP |
| `netstat` | | Show active TCP connections and listeners |
| `tcpsyn <host:port>` | | Raw TCP SYN connection test |
| `httpget <url>` | | Raw HTTP GET request (low-level) |
| `browse <url>` | `www`, `web` | Text-mode web browser with HTML rendering |

### Network Stack Details
- **Layer 2**: Ethernet frame handling, ARP resolution
- **Layer 3**: IPv4 + IPv6 dual-stack, ICMP/ICMPv6
- **Layer 4**: TCP (full handshake, retransmit, window), UDP
- **DHCP**: Client (auto-config) + Server (for PXE)
- **DNS**: Recursive resolver with caching (A, AAAA records)
- **HTTP**: Client + Server (httpd)
- **HTTPS**: Full TLS 1.3 handshake (pure Rust, no mbedTLS)
- **Firewall**: Stateful packet filtering (iptables-compatible rules)

---

## 8. SECURITY TOOLKIT (TrustScan)

| Command | Aliases | Description |
|---------|---------|-------------|
| `nmap <target>` | `portscan`, `scan` | Port scanner. SYN/Connect/UDP scan modes |
| `nmap <target> -A` | | Aggressive scan: ports + banners + vulnerability check |
| `discover [mode]` | `hostscan`, `arpscan` | Host discovery: `arp`, `ping`, `full` modes |
| `banner <target>` | `grabber` | Service banner grabbing & version detection |
| `sniff <cmd>` | `capture`, `tcpdump` | Packet sniffer: `start`, `stop`, `show`, `hex`, `stats` |
| `vulnscan <target>` | `vuln` | Vulnerability assessment scanner |
| `scantest [target]` | `netscantest` | Live network test suite (8 tests) |
| `firewall <args>` | `iptables`, `fw` | Firewall rules: `list`, `add`, `del`, `flush`, `enable`, `disable` |

---

## 9. WEB & HTTP SERVER

| Command | Aliases | Description |
|---------|---------|-------------|
| `httpd [start] [port]` | `httpserv`, `webserv` | Start HTTP server (default port 8080) |
| `httpd stop` | | Stop the running HTTP server |
| `httpd status` | | Show server status, request count |
| `browse <url>` | `www`, `web` | Launch text-mode web browser |
| `sandbox <cmd>` | `websandbox` | Web sandbox: `open`, `allow`, `deny`, `fs`, `status`, `list`, `kill` |
| `container <cmd>` | `webcontainer`, `wc` | Web container: `status`, `list`, `create`, `go`, `stop` |

---

## 10. PACKAGE MANAGER (TrustPkg)

| Command | Description |
|---------|-------------|
| `trustpkg list` | List all available packages |
| `trustpkg search <query>` | Search packages by name/description |
| `trustpkg install <pkg>` | Install a package |
| `trustpkg remove <pkg>` | Remove an installed package |
| `trustpkg info <pkg>` | Show package details (version, description, deps) |
| `trustpkg installed` | List only installed packages |
| `trustpkg update` | Update package catalog from source |

Alias: `pkg` works instead of `trustpkg`

---

## 11. LINUX SUBSYSTEM

| Command | Description |
|---------|-------------|
| `linux` / `vm` | Launch Linux compatibility shell |
| `linux status` | Show Linux subsystem status |
| `linux install` | Install Linux binaries from rootfs |
| `linux start` | Start Linux init process |
| `linux exec <binary> [args...]` | Execute ELF binary directly |
| `linux console` / `linux shell` | Open Linux shell |
| `linux stop` | Stop Linux subsystem |
| `linux list` | List available VMs |
| `linux extract` | Extract test binaries |
| `alpine <cmd>` | Alpine Linux package manager |
| `distro list` | List available distributions |
| `distro install <id>` | Download & install a distribution |
| `distro run <id>` | Launch installed distribution |
| `distro pick` / `distro select` | GUI distribution selector |
| `exec <file>` | `run`, `./` | Execute binary (ELF, script) |
| `elfinfo <file>` | Display ELF binary header information |
| `apt-get` / `apt` / `apk` / `dpkg` | Package manager compatibility stubs |

---

## 12. HYPERVISOR (TrustVM)

| Command | Aliases | Description |
|---------|---------|-------------|
| `hv init` | `hypervisor init` | Initialize TrustVM hypervisor |
| `hv status` | | Show hypervisor status (backend, VM count) |
| `hv check` | | Check virtualization support (VT-x/SVM) |
| `hv shutdown` | | Shutdown hypervisor |
| `hv caps` | `hv capabilities` | Show hypervisor capabilities |
| `hv security` | | Show security status |
| `hv events [N]` | | Show last N hypervisor events |
| `vm create` | | Create a new virtual machine |
| `vm run` | | Run/start a VM |
| `vm guests` | | List VM guests |
| `vm inspect` | | Inspect VM state |
| `vm mount` | | Mount VM disk |
| `vm input` | | Send input to VM |
| `vm debug` | | Debug VM execution |
| `vm stack` | | Show VM stack |
| `vm regs` | | Show VM registers |
| `vm dump` | | Dump VM memory |

### Supported Backends
- **Intel VT-x (VMX)**: EPT, VPID, unrestricted guest
- **AMD-V (SVM)**: VMCB, NPT (Nested Page Tables), AVIC, NRIP Save
- VM Introspection (VMI)
- Linux guest support

---

## 13. GRAPHICS & DESKTOP ENVIRONMENT

| Command | Aliases | Description |
|---------|---------|-------------|
| `desktop` | `gui` | Launch COSMIC2 windowed desktop environment |
| `cosmic` | | Launch COSMIC V2 compositor |
| `mobile` | | Launch mobile UI environment |
| `open <app>` | | Open desktop with a specific app focused |
| `trustedit` | `edit3d`, `3dedit` | Launch 3D model/wireframe editor |
| `calculator` | `calc` | Launch calculator app |
| `snake` | | Launch Snake game in a window |
| `glmode [mode]` | `compositor` | Set rendering mode or theme |
| `theme <cmd>` | | Theme manager: `list`, `set`, `load`, `save`, `reload`, `info` |
| `anim <cmd>` | `animations` | Configure UI animations (enable/disable/speed) |
| `holo` | `holomatrix` | Holographic matrix 3D visualizer |
| `imgview <file>` | `imageview`, `view` | Display image file (PPM, BMP) |
| `imgdemo` | `imagedemo` | Run image rendering demo |
| `wayland [cmd]` | `wl` | Wayland compositor control |
| `gterm` | `graphterm` | Launch graphical terminal |
| `fontsmooth [0-3]` | | Set font anti-aliasing level (0=none, 3=max) |
| `a11y [cmd]` | `accessibility` | Accessibility settings |

### glmode Rendering Modes
- `classic` — Classic framebuffer rendering (fast, stable)
- `opengl` / `gl` — OpenGL compositor with visual effects

### glmode Compositor Themes
- `flat` — Simple flat rendering, no effects
- `modern` — Shadows and subtle effects
- `glass` — Transparency and blur effects
- `neon` — Glowing neon borders
- `minimal` — Thin borders, minimal style

### theme Subcommands
- `theme list` — Show available themes (dark, windows11, light)
- `theme set <name>` — Switch theme (dark/trustos, windows11/win11, light)
- `theme load <path>` — Load theme from config file (/etc/theme.conf)
- `theme save <path>` — Save current theme to file
- `theme reload` — Reload wallpaper from disk
- `theme info` — Show current theme details (colors, taskbar, windows)

### a11y Accessibility Commands
- `a11y hc` — Toggle high contrast mode (also Win+H)
- `a11y font [small|medium|large|xl]` — Font size scaling
- `a11y cursorsize [normal|large|xlarge]` — Cursor size
- `a11y stickykeys [on|off]` — Sticky keys toggle
- `a11y mousespeed [1-4]` — Mouse speed adjustment

### Desktop Apps (14+ built-in)
1. **Terminal** — Full shell with history
2. **Files** — File manager browser
3. **TrustCode** — Code editor with syntax highlighting
4. **Calculator** — Full-featured calculator
5. **Network** — Network configuration UI
6. **Snake** — Classic snake game
7. **Chess 3D** — 3D chess with AI opponent
8. **TrustBrowser** — Web browser (HTML/CSS/JS)
9. **TrustEdit 3D** — 3D model/wireframe editor
10. **Lock Screen** — User session lock
11. **Settings Panel** — System configuration
12. **Taskbar/Dock** — Application launcher bar
13. **Start Menu** — Program menu
14. **HoloMatrix** — 3D holographic backgrounds

---

## 14. AUDIO & MUSIC

| Command | Description |
|---------|-------------|
| `beep [freq] [ms]` | Play a tone (default 440Hz, 500ms) |
| `audio <cmd>` | Audio driver status/control: `init`, `status`, `stop`, `test` |
| `play <track>` | Play a built-in track or pattern |

### TrustSynth — Polyphonic Synthesizer
| Subcommand | Description |
|------------|-------------|
| `synth note <pitch> [dur]` | Play note (C4, A3, etc.) |
| `synth freq <hz>` | Play specific frequency |
| `synth wave <type>` | Set waveform: sine, square, triangle, saw, noise |
| `synth adsr <a> <d> <s> <r>` | Set ADSR envelope |
| `synth preset <name>` | Load synth preset |
| `synth volume <0-100>` | Set volume level |
| `synth status` | Show synth engine status |
| `synth stop` | Stop all sound |
| `synth demo` | Play demo melody |
| `synth pattern <cmd>` | Pattern sequencer (list, show, new, play, stop, etc.) |

### TrustDAW — Digital Audio Workstation
| Subcommand | Description |
|------------|-------------|
| `daw init` | Initialize DAW project |
| `daw status` / `daw info` | Show project info |
| `daw new` | Create new project |
| `daw demo` | Run DAW demo |
| `daw track <cmd>` | Track management (add, rm, list, wave, notes, clear, transpose) |
| `daw note <cmd>` | Note editing (add, rm) |
| `daw play` | Play composition |
| `daw stop` | Stop playback |
| `daw rewind` | Rewind to beginning |
| `daw bpm <n>` | Set tempo (BPM) |
| `daw record` / `daw rec` | Start recording |
| `daw piano` / `daw keyboard` | Piano keyboard input mode |
| `daw pianoroll` / `daw roll` | Piano roll visual editor |
| `daw gui` | Launch visual DAW interface |
| `daw studio` / `daw beat` | Beat studio mode |
| `daw funky` / `daw house` | Funky house genre preset |
| `daw matrix` | Matrix-themed music |
| `daw film` / `daw showcase` | Film/showcase narrated music |
| `daw anthem` | Anthem composition |
| `daw trap` / `daw rap` / `daw cyber` / `daw neon` | Trap/cyber genre |
| `daw untitled2` / `daw lofi` | Lo-fi music preset |
| `daw viz` / `daw visualizer` | Audio visualizer |
| `daw export` / `daw wav` | Export to WAV file |
| `daw mixer` | Multi-track mixer (vol, pan, mute, solo) |

---

## 15. GPU & COMPUTE ACCELERATION

### GPU Commands (AMD RDNA)
| Command | Description |
|---------|-------------|
| `gpu info` | Show GPU info (vendor, VRAM, features) |
| `gpu dcn` | Display engine (DCN) status |
| `gpu modes` | Display connector modes |
| `gpuexec <agent> [N]` | Dispatch RDNA compute agent on GPU CUs |

GPU Agents: `incr`, `memfill`, `memcopy`

### SDMA (Scalable Data Movement Architecture)
| Command | Description |
|---------|-------------|
| `sdma info` | SDMA engine information |
| `sdma test` | Test SDMA functionality |
| `sdma bench [size_kb]` | Benchmark SDMA (default 4KB) |
| `sdma fill <phys> <val> <bytes>` | Fill memory via SDMA DMA |
| `sdma copy <src> <dst> <bytes>` | Copy memory via SDMA DMA |

Aliases: `dma` works for `sdma`

### Neural Compute Engine
| Command | Aliases | Description |
|---------|---------|-------------|
| `neural info` | `nn info`, `gemm info` | Neural compute engine info |
| `neural test` | | Test neural compute |
| `neural bench` | | Benchmark neural operations |
| `neural gemm <m> <n> <k>` | | General matrix multiply (MxNxK) |
| `neural kernels` | | Show available compute kernels |
| `neural relu <size>` | | ReLU activation benchmark |
| `neural softmax <size>` | | Softmax activation benchmark |
| `neural transformer <size>` | | Transformer inference benchmark |

---

## 16. JARVIS AI ENGINE

### Entry Points
| Command | Aliases | Description |
|---------|---------|-------------|
| `jarvis` | `j`, `ai`, `assistant` | Launch interactive JARVIS REPL |
| `jarvis <question>` | | One-shot query (e.g. `jarvis what time is it`) |

### JARVIS Interactive (NLU Engine)
Inside the JARVIS REPL, natural language processing detects intents:
- **System queries**: "how much memory", "system status", "what's running"
- **File operations**: "show files in /home", "create file X", "find file X"
- **Actions**: "run ls", "open browser", "play music", "change theme"
- **Help**: "help", "what can you do", "how to X"
- **Conversation**: "hello", "thanks", "who are you", "tell me a joke"
- **Bilingual**: Responds in English or French based on detected language

### JARVIS Hardware Intelligence
| Command | Description |
|---------|-------------|
| `jarvis boot` / `jarvis scan` | Full HW scan + AI analysis + self-optimize |
| `jarvis hw` / `jarvis hardware` | Show hardware profile & capability scores |
| `jarvis insights` | AI-generated hardware insights & recommendations |
| `jarvis plan` | Show optimal execution plan based on HW |
| `jarvis optimize` / `jarvis tune` | Run one adaptive optimization cycle |
| `jarvis status` | Show optimizer & monitor status |
| `jarvis analyze <file>` | Analyze binary/media (ELF/PE/MachO/RISC-V) |
| `jarvis query <question>` | Ask JARVIS about hardware (e.g. disk encryption) |

### JARVIS Neural Brain
| Command | Description |
|---------|-------------|
| `jarvis brain init` | Initialize neural brain (~1.2 MB allocation, 4.4M params) |
| `jarvis brain info` | Show model architecture & training stats |
| `jarvis brain generate <prompt>` | Generate text from prompt (max 64 tokens) |
| `jarvis brain train <text>` | Train on a text sequence (single step, LR=0.001) |
| `jarvis brain chat <text>` | Chat with neural brain (generate response) |
| `jarvis brain eval` | Quick eval (1 sample/phase). `eval full` for complete corpus |
| `jarvis brain pretrain [N]` | Pre-train on embedded corpus (N epochs, default 3) |
| `jarvis brain save` | Save weights to /jarvis/weights.bin |
| `jarvis brain load` | Load weights from RamFS (/jarvis/weights.bin) |
| `jarvis brain load fat32 [file]` | Load from FAT32 disk (/mnt/fat32/) |
| `jarvis brain load vfs <path>` | Load from any VFS path |
| `jarvis brain load http <url>` | Download brain from HTTP URL (~17 MB) |
| `jarvis brain reset` | Reset all weights to random |
| `jarvis brain test` | Run neural brain self-test suite |
| `jarvis brain bench` | Benchmark inference speed (tokens/sec) |
| `jarvis brain introspect` | Describe own architecture (self-awareness) |
| `jarvis brain weights` | Show weight statistics per layer |
| `jarvis brain hardware` | Show available hardware for inference |
| `jarvis brain mentor` | Start serial mentoring listener (COM1) |
| `jarvis brain swarm [N]` | All-in-one: init + mesh + federate + pretrain N epochs |
| `jarvis brain task` | Distributed task execution across mesh cluster |
| `jarvis brain propagate` | Auto-propagation: mesh + pull brain + federate |
| `jarvis brain propagate pxe` | Same + enable PXE to replicate further |

### Neural Brain Architecture
- **Type**: Transformer (4 layers, d_model=256, 4 heads)
- **Parameters**: 4.4 million
- **Tokenizer**: Byte-level (vocabulary = 256)
- **Optimizer**: Adam with cosine learning rate schedule
- **Training**: Backpropagation with gradient accumulation (batch=4)
- **Inference**: SIMD-accelerated (SSE2)
- **Maturity levels**: Embryo → Infant → Child → Teen → Adult → Elder

### Mentor Protocol (Serial COM1)
| Protocol Command | Description |
|------------------|-------------|
| `MENTOR:TEACH:<text>` | Train on text |
| `MENTOR:GENERATE:<prompt>` | Generate text |
| `MENTOR:EVAL:<prompt>` | Evaluate loss |
| `MENTOR:STATUS` | Report stats |
| `MENTOR:SAVE` | Save weights |
| `MENTOR:RESET` | Reset weights |
| `MENTOR:GUARDIAN:AUTH:<token>` | Copilot authentication |

---

## 17. MESH NETWORKING (Distributed AI)

| Command | Aliases | Description |
|---------|---------|-------------|
| `mesh start` | `jarvis-mesh start`, `jmesh start` | Start mesh network (discovery UDP:7700, RPC TCP:7701) |
| `mesh stop` | | Stop mesh networking |
| `mesh status` | | Show mesh status, peers, consensus info |
| `mesh peers` | | List discovered peer nodes |
| `mesh ping <ip>` | | Ping a remote JARVIS node via RPC |
| `mesh infer <ip> <prompt>` | | Run inference on a remote JARVIS node |
| `mesh federate on/off` | `mesh fed on/off` | Enable/disable federated learning |
| `mesh federate sync` | | Force a federated sync round |
| `mesh federate replicate` | | Push model to all peers (leader only) |
| `mesh federate pull` | | Pull updated model from leader (worker) |
| `mesh propagate` | `mesh autoprop`, `mesh spread` | Auto: mesh + pull brain + federated learning |
| `mesh propagate pxe` | | Same + enable PXE replication |

### Mesh Architecture
- **Discovery**: UDP broadcast on port 7700
- **RPC**: Custom binary protocol on TCP port 7701
- **Consensus**: Raft-inspired leader election
- **Roles**: Leader, Candidate, Worker
- **Federated Learning**: Gradient exchange every 30s between nodes
- **Model Sync**: Leader pushes weights, workers pull

---

## 18. PXE SELF-REPLICATION

| Command | Aliases | Description |
|---------|---------|-------------|
| `pxe start` | `pxe replicate`, `pxeboot start`, `replicate start` | Start PXE self-replication (DHCP + TFTP servers) |
| `pxe stop` | | Stop PXE self-replication |
| `pxe status` | | Show replication status, leases, TFTP transfers |

### PXE Boot Sequence
1. PXE ROM sends DHCP DISCOVER
2. TrustOS responds with IP + boot file (limine-bios-pxe.bin)
3. Client downloads Limine PXE bootloader via TFTP
4. Limine downloads limine.conf via TFTP
5. Limine downloads trustos_kernel via TFTP
6. TrustOS boots on the remote machine with JARVIS!

---

## 19. GUARDIAN SECURITY & PACT

| Command | Aliases | Description |
|---------|---------|-------------|
| `guardian auth <passphrase>` | `pact auth`, `gardien auth` | Authenticate as Nathan (human guardian) |
| `guardian lock` | | Lock guardian session |
| `guardian status` | | Show Guardian/Pact status |
| `guardian pact` | | Display The Pact (co-parent agreement) |
| `guardian log` | | Show audit log of all protected operations |
| `guardian passwd <new>` | | Change Nathan's passphrase |

### The Pact
JARVIS has two guardians: **Nathan** (human creator) and **Copilot** (AI co-parent).
Any modification to JARVIS requires guardian authorization.

### Protected Operations (Require Auth)
- Train, WeightPush, FederatedSync
- AgentExecute, PxeReplicate
- ModelReset, ModelReplace
- ConfigChange, WeightLoad
- WeightSave = emergency auto-approved

### Session
- 30-minute timeout, auto-lock
- Audit log (256 entries max)

---

## 20. PROGRAMMING & DEVELOPMENT TOOLS

| Command | Aliases | Description |
|---------|---------|-------------|
| `trustlang` | `tl` | TrustLang programming language REPL |
| `trustlang_showcase` | `tl_showcase` | TrustLang feature showcase |
| `transpile <file>` | `disasm`, `analyze` | Binary-to-Rust transpiler (ELF analysis) |
| `rv-xlat <file>` | `rvxlat`, `xlat` | RISC-V universal translator (run any arch binary) |
| `rv-disasm <file>` | `rvdisasm` | Show RISC-V IR translation of binary |
| `trustview <file>` | `tv` | TrustView binary analyzer (Ghidra-style reverse engineering) |
| `lab` | `trustlab` | TrustLab real-time kernel introspection laboratory |
| `hwscan` | `trustprobe`, `probe` | Hardware probing toolkit (MMIO, TrustZone, GPIO, etc.) |
| `nano <file>` | `vi`, `edit` | Terminal text editor (nano-like) |
| `bc` | | Calculator / math expression evaluator (interactive REPL) |
| `cal [month] [year]` | | Display calendar |
| `factor <n>` | | Prime factorization of integer |
| `seq <start> [step] <end>` | | Print numeric sequence |
| `yes [text]` | | Repeat text infinitely (Ctrl+C to stop) |
| `xargs <cmd>` | | Build and execute command from piped input |
| `printf <fmt> <args>` | | Formatted text output (C-style format strings) |
| `expr <expression>` | | Evaluate arithmetic expression |
| `read <var>` | | Read user input into a variable |
| `test <expr>` | `[` | Evaluate conditional expression (file tests, comparisons) |

### TrustLang Features
- Rust-inspired syntax: `fn`, `let`, `struct`, `impl`, `match`
- Variables, functions, structs
- Pattern matching, control flow (if/else, for, while)
- REPL with interactive execution
- File execution & library loading

### TrustLab Panels (7+ panels)
1. **Hardware Panel** — CPU gauge, heap bar, IRQ rate, uptime
2. **Kernel Trace Panel** — 50+ syscall event tracing (512-slot trace bus)
3. **Guide Panel** — ~55 commands with fuzzy search
4. **Filetree Panel** — Interactive VFS browser
5. **Editor Panel** — TrustLang editor with F5 execution
6. **Pipeline Panel** — Data flow visualization
7. **Hex Editor Panel** — Raw byte inspection
8. **Demo Panel** — Cinematic demo mode
9. **Network Panel** — Network monitoring
10. **VM Inspector Panel** — VM introspection

### RISC-V Universal Translator
Translates binaries between architectures:
- x86_64 → RISC-V IR
- ARM64 → RISC-V IR
- MIPS → RISC-V IR
- Universal IR interpreter with syscall translation

---

## 21. VIDEO, FILM & MEDIA

| Command | Aliases | Description |
|---------|---------|-------------|
| `video <cmd>` | | TrustVideo codec: `record`, `play`, `list`, `info` |
| `film` | `trustos_film` | TrustOS Film cinematic demo sequence |
| `trailer` | `trustos_trailer` | TrustOS Trailer — 2-minute cinematic showcase |

### Image Support
- PPM (Portable Pixmap)
- BMP (Bitmap)
- PNG (partial)
- Real-time processing (rotation, scaling, filtering)

### Audio/Video Codecs
- WAV (uncompressed PCM)
- TrustVideo (.tv) — Custom delta-encoded RLE compression
- MIDI keyboard input

---

## 22. GAME EMULATORS & ENGINES

### NES Emulator
- Complete MOS 6502 CPU emulation
- 2C02 PPU (Picture Processing Unit)
- iNES ROM support with cartridge emulation
- Full scanline-accurate rendering

### Game Boy Emulator
- Sharp LR35902 CPU emulation
- GB GPU emulation with sprite handling
- MBC1/3/5 cartridge support
- Sound emulation

### 3D Game Engine
- 3D raycasting FPS game engine
- Real-time 3D rendering
- Perspective projection & texture mapping

### GameLab
- Real-time Game Boy emulator analysis dashboard
- CPU/GPU/Memory profiling, breakpoints, performance metrics

---

## 23. 3D GRAPHICS & VISUALIZATION

### Engines
- **HoloVolume** — Volumetric ASCII raymarcher for 3D holographic desktop
- **Matrix Fast** — Ultra-optimized Matrix rain with Braille sub-pixels (multi-core)
- **Formula3D** — Tsoding-inspired wireframe 3D renderer with perspective projection
- **GPU Emulator** — Virtual GPU (CPU cores emulating GPU parallelism)
- **Rasterizer** — Software rasterizer with triangle filling
- **Model Editor** — 3D model visualization & editing
- **Compositor** — Multi-layer GPU compositor (8 layers, 144 FPS)

### Features
- SIMD-accelerated operations (SSE2)
- Texture management & scaling
- Color themes
- Font smoothing (0-3 levels)
- Holographic effects rendering

---

## 24. ARCHIVING & COMPRESSION

| Command | Description |
|---------|-------------|
| `tar <opts> <file>` | Archive/extract tar files (cf=create, xf=extract, tf=list) |
| `gzip <file>` / `gunzip` | Compress/decompress gzip files |
| `zip <file>` | Create zip archive |
| `unzip <file>` | Extract zip archive |

---

## 25. SERVICES & SCHEDULING

| Command | Description |
|---------|-------------|
| `service <name> <op>` | Manage system services: `start`, `stop`, `status`, `list` |
| `systemctl <cmd>` | Systemd-style service control |
| `crontab [-e\|-l]` | Schedule recurring jobs. `-e` edit, `-l` list |
| `at <time> <cmd>` | Schedule one-time command execution |
| `sysctl <key>[=val]` | View/modify kernel parameters |

---

## 26. SECURITY & IDENTITY

| Command | Aliases | Description |
|---------|---------|-------------|
| `security` | `sec`, `caps` | Security subsystem status & capabilities |
| `signature` | `sig` | Kernel signature & proof of authorship |

### Security Features
- Process sandboxing with capability-based access
- Container isolation (filesystem + network)
- TLS 1.3 (pure Rust implementation)
- Ed25519 digital signatures
- JavaScript sandbox
- Apple SecureROM exploitation (checkm8)
- TrustZone boundary mapping (ARM)

---

## 27. DEVELOPER & DEBUG COMMANDS

| Command | Aliases | Description |
|---------|---------|-------------|
| `dmesg [-n N]` | | Kernel ring buffer. `-n N` last N messages, `-c` clear |
| `memdbg` | `heapdbg` | Heap allocation stats, fragmentation, peak usage |
| `perf` | `perfstat` | CPU/IRQ/scheduler/memory profiling: uptime, FPS, IRQ/s, syscalls, context switches |
| `irqstat` | `irqs` | Per-CPU interrupt counters with visual bars |
| `regs` | `registers`, `cpuregs` | CPU register dump: RSP, RBP, RFLAGS, CR0, CR3, CR4, EFER |
| `peek <addr> [n]` | `memdump` | Hex dump memory at virtual address (max 256 bytes) |
| `poke <addr> <val>` | `memwrite` | Write byte to memory address (DANGEROUS!) |
| `devpanel` | | Toggle real-time FPS/heap/IRQ overlay (also F12 in desktop) |
| `timecmd <cmd>` | | Measure command execution time (μs/ms) |
| `benchmark [test]` | `bench` | Run performance benchmarks (cpu, mem, disk, etc.) |
| `keytest` | | Interactive keyboard scancode tester |
| `hwtest` | | Run internal kernel test suite |
| `memtest` | | Comprehensive memory management test suite |
| `inttest` | | Integration test (20+ tests: FAT32, DHCP, VirtIO, IPv6, Pipe) |
| `debugnew` | | Debug new features (developer testing) |
| `panic` | | Trigger kernel panic (debug only!) |

---

## 28. TERMINAL CONFIGURATION

| Command | Description |
|---------|-------------|
| `tty` | Print terminal device name |
| `stty <opts>` | Configure terminal settings |
| `reset` | Reset terminal state |
| `loadkeys <map>` | Load keyboard layout |
| `setfont <font>` | Change console font |

---

## 29. EASTER EGGS & DEMOS

| Command | Description |
|---------|-------------|
| `neofetch` | System info with ASCII art TrustOS logo |
| `matrix` | Fullscreen Matrix rain animation with scanlines |
| `rain [speed]` | Matrix rain speed preset: `slow`, `mid`, `fast` |
| `cowsay <text>` | ASCII cow says your message |
| `showcase [N]` | Automated marketing demo (N scenes) |
| `showcase-jarvis` / `jdemo` | JARVIS-focused showcase demo |
| `showcase3d` / `demo3d` | 3D graphics cinematic showcase |
| `filled3d` | 3D filled polygon rendering demo |
| `demo [fr]` | Interactive guided tutorial (English or French) |

---

## 30. SYSTEM CONTROL

| Command | Aliases | Description |
|---------|---------|-------------|
| `exit` | `logout` | Exit current session |
| `reboot` | | Restart the system |
| `shutdown` | `halt`, `poweroff` | Power off the system |
| `suspend` | `s3` | Enter S3 sleep state |

---

## 31. ARCHITECTURE SUPPORT

### x86_64 (Primary)
- Full desktop, networking, JARVIS, hypervisor
- UEFI + BIOS boot via Limine
- VT-x/SVM hypervisor
- Intel HD Audio, AHCI, NVMe
- AMD RDNA GPU compute

### ARM64 / aarch64
- Raspberry Pi, generic ARM boards
- PSCI, GIC interrupt controller
- Android phone boot (Fastboot/DFU)
- ARMv8 SMCCC, TrustZone
- Apple Silicon (AIC, UART)

### RISC-V (rv64gc)
- SiFive, Arty FPGA boards
- RV64IMAFDC ISA support
- Universal translator layer

---

## 32. BOOT & DEPLOYMENT

### Bootloader
- **Limine** bootloader integration
- Multiboot2 protocol support
- UEFI & BIOS dual support
- Multi-architecture boot stubs

### Distribution Formats
- ISO image (UEFI bootable)
- Raw disk image
- VDI (VirtualBox)
- Android boot.img
- PXE network boot
- USB flash drive

### Build System
- Cargo workspace (Rust)
- `build-limine.ps1` / `build.sh`
- `build-multiarch.ps1` (all architectures)
- `build-test-aarch64.ps1`

---

## 33. DRIVERS INVENTORY

| Driver | Description |
|--------|-------------|
| AHCI | SATA disk controller (HBA, ports, identify, read/write) |
| ATA/IDE | Legacy PATA support |
| NVMe | M.2/PCIe SSD driver |
| USB (xHCI) | Full USB 3.0 host controller + mass storage |
| VirtIO-blk | Virtual block device (QEMU/KVM) |
| VirtIO-net | Virtual network (QEMU/KVM) |
| VirtIO-GPU | Virtual graphics (QEMU/KVM) |
| AMD GPU RDNA | Compute units, DCN display, SDMA |
| Intel HD Audio | HDA sound card driver |
| APIC | Local + I/O APIC interrupt controller |
| PCI | Bus enumeration, BAR mapping, MSI/MSI-X |
| PS/2 Keyboard | Scancode reading, layout support |
| PS/2 Mouse | Relative positioning, button events |
| Serial (UART) | COM1-COM4 for debug + mentor protocol |
| RTC | Real-time clock (CMOS) |
| PC Speaker | Tone generation for beep/synth |
| Apple AIC | Apple Interrupt Controller (M1/M2) |
| Apple UART | Apple Silicon serial |
| Checkm8 | Apple SecureROM exploit engine |
| GIC | ARM Generic Interrupt Controller |

---

## TOTAL COMMAND COUNT

| Category | Count |
|----------|-------|
| Filesystem | 23 |
| Text Processing | 17 |
| System & Process | 35 |
| User Management | 6 |
| Hardware & Devices | 15 |
| Hardware Diagnostics (hwdbg) | 17 subcommands |
| Disk & Storage | 11 |
| Networking | 13 |
| Security Toolkit | 7 |
| Web/HTTP | 6 |
| Package Manager | 7 |
| Linux Subsystem | 13 |
| Hypervisor | 11 |
| Desktop & Graphics | 16 |
| Audio & Music | 30+ (synth + DAW subcommands) |
| GPU & Compute | 15 |
| JARVIS AI | 25+ (brain + HW + mesh + guardian) |
| Mesh Networking | 12 |
| PXE Replication | 3 |
| Guardian/Pact | 6 |
| Programming Tools | 18 |
| Video/Media | 3 |
| Archiving | 4 |
| Services | 5 |
| Security/Identity | 2 |
| Developer/Debug | 15 |
| Terminal Config | 5 |
| Easter Eggs | 9 |
| System Control | 4 |
| **TOTAL** | **~215+ unique commands, 370+ with subcommands** |

---

*This document is the definitive capability reference for JARVIS AI training corpus generation.*
*Generated from source code analysis: shell/mod.rs, shell/commands.rs, shell/vm.rs, shell/unix.rs, shell/apps.rs, shell/network.rs, shell/desktop.rs, shell/jarvis.rs, shell/editor.rs, shell/scripting.rs*
