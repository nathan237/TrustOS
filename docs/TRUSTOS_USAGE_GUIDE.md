# TrustOS -- Complete Usage Guide

> **Version:** 0.7.0-checkm8
> **Author:** Nated0ge
> **This guide is LOCAL ONLY -- not published on GitHub.**
> If you want a copy, open an issue or contact the author.

---

## Table of Contents

1. [Getting Started](#1-getting-started)
2. [Building TrustOS](#2-building-trustos)
3. [Running TrustOS](#3-running-trustos)
4. [Shell Basics](#4-shell-basics)
5. [File System Commands](#5-file-system-commands)
6. [Text Processing](#6-text-processing)
7. [System & Process Management](#7-system--process-management)
8. [User Management](#8-user-management)
9. [Hardware & Devices](#9-hardware--devices)
10. [Disk & Storage](#10-disk--storage)
11. [Networking](#11-networking)
12. [Web Browser & Containers](#12-web-browser--containers)
13. [Security Toolkit (TrustScan)](#13-security-toolkit-trustscan)
14. [HTTP Server](#14-http-server)
15. [Package Manager (TrustPkg)](#15-package-manager-trustpkg)
16. [Linux Subsystem](#16-linux-subsystem)
17. [Hypervisor & VMs](#17-hypervisor--vms)
18. [COSMIC2 Desktop](#18-cosmic2-desktop)
19. [Desktop Applications](#19-desktop-applications)
20. [Text Editor](#20-text-editor)
21. [JARVIS AI](#21-jarvis-ai)
22. [Mesh Networking](#22-mesh-networking)
23. [PXE Replication](#23-pxe-replication)
24. [Guardian System (The Pact)](#24-guardian-system-the-pact)
25. [TrustLab](#25-trustlab)
26. [Programming (TrustLang)](#26-programming-trustlang)
27. [Emulators (Game Boy, NES)](#27-emulators-game-boy-nes)
28. [Audio & Synthesizer](#28-audio--synthesizer)
29. [3D Graphics & Demos](#29-3d-graphics--demos)
30. [Archiving & Compression](#30-archiving--compression)
31. [System Control](#31-system-control)
32. [Testing & Benchmarks](#32-testing--benchmarks)
33. [Development Workflows](#33-development-workflows)
34. [Troubleshooting](#34-troubleshooting)
35. [Command Quick Reference](#35-command-quick-reference)

---

## 1. Getting Started

### What is TrustOS?

TrustOS is a bare-metal operating system written in 100% Rust. It boots on real hardware (PCs, ARM boards, RISC-V) and virtual machines. It includes a full desktop environment, network stack, web browser, AI engine, emulators, hypervisor, and 270+ commands -- all from scratch, with zero C dependencies.

### Editions

| Edition | What's Included | ISO Size |
|---------|----------------|----------|
| **TrustOS** | Full OS, desktop, networking, emulators, CyberLab, JARVIS engine (untrained) | ~12 MB |
| **TrustOS-JarvisPack** | Everything above + pretrained JARVIS brain (4.4M-param transformer) | ~29 MB |

### Requirements

- **CPU:** x86_64 with SSE2 (any 64-bit Intel/AMD from ~2004+)
- **RAM:** 512 MB minimum, 1 GB recommended
- **Boot:** UEFI or BIOS via Limine bootloader
- **Display:** Framebuffer (any VGA/GPU works)
- For VirtualBox: EFI enabled, VBoxSVGA adapter, 128 MB VRAM

---

## 2. Building TrustOS

### Prerequisites

- **Rust nightly toolchain** (managed via `rust-toolchain.toml`)
- **WSL or Linux** for `xorriso` (ISO creation)
- **QEMU** or **VirtualBox** for testing

### Build Commands

```powershell
# Base edition (no pretrained JARVIS weights)
.\build-trustos.ps1

# AI edition (includes pretrained JARVIS brain)
.\build-trustos-jarvispack.ps1

# Options:
.\build-trustos.ps1 -NoRun          # Build only, don't launch VM
.\build-trustos.ps1 -Clean          # Clean build (cargo clean first)
```

### Build Output

| Edition | ISO Location | Size |
|---------|-------------|------|
| TrustOS | `builds/trustos/trustos.iso` | ~12 MB |
| TrustOS-JarvisPack | `builds/trustos-jarvispack/trustos-jarvispack.iso` | ~29 MB |

### Manual Build

```powershell
# 1. Build kernel
cargo build --release -p trustos_kernel

# 2. Create ISO structure
mkdir iso_root/boot, iso_root/EFI/BOOT
cp target/x86_64-unknown-none/release/trustos_kernel iso_root/boot/
cp limine/BOOTX64.EFI iso_root/EFI/BOOT/
cp limine.conf iso_root/

# 3. Create ISO (via WSL)
wsl xorriso -as mkisofs -b boot/limine-bios-cd.bin -no-emul-boot \
  -boot-load-size 4 -boot-info-table --efi-boot EFI/BOOT/BOOTX64.EFI \
  -efi-boot-part --efi-boot-image -o trustos.iso iso_root/
```

---

## 3. Running TrustOS

### QEMU

```powershell
# x86_64
qemu-system-x86_64 -cdrom trustos.iso -m 512M -cpu max -smp 4 -display gtk -vga std -serial stdio

# With networking
qemu-system-x86_64 -cdrom trustos.iso -m 512M -cpu max -smp 4 \
  -device virtio-net-pci,netdev=net0 -netdev user,id=net0 \
  -display gtk -serial stdio

# ARM64
qemu-system-aarch64 -M virt -cpu cortex-a72 -m 512M -kernel trustos_kernel -serial stdio

# RISC-V
qemu-system-riscv64 -M virt -m 512M -kernel trustos_kernel -serial stdio
```

### VirtualBox

1. Create VM: Type=Other, Version=Other/Unknown (64-bit)
2. Settings:
   - System: 1024 MB RAM, Enable EFI, 4 CPUs
   - Display: VBoxSVGA, 128 MB VRAM
   - Storage: Insert `trustos.iso` in optical drive
3. Start the VM

### Real Hardware (USB)

```powershell
# Flash to USB drive
.\scripts\misc\flash-usb.ps1

# Or manually:
# 1. Format USB as FAT32
# 2. Copy ISO contents to USB root
# 3. Boot from USB (UEFI)
```

---

## 4. Shell Basics

TrustOS boots into a shell. Type commands and press Enter. The shell supports:

- **Tab completion** (partial)
- **Command history** (up/down arrows)
- **Piping** (`|`) and **redirection** (`>`)
- **Environment variables** (`$HOME`, `export VAR=value`)
- **Aliases** (`alias ll='ls -la'`)
- **Script execution** (`source script.sh`)
- **Job control** (`jobs`, `kill`)

### Essential Commands

```
help                 # Show all commands
help <command>       # Help for specific command
man <command>        # Manual page
clear                # Clear screen
neofetch             # System info with logo
version              # TrustOS version
info                 # System summary
```

---

## 5. File System Commands

TrustOS has a VFS (Virtual File System) supporting TrustFS, FAT32, devfs, procfs, and ramfs.

### Navigation

```
ls                   # List files
ls -la               # Long format with hidden files
cd /path             # Change directory
cd ..                # Go up one level
pwd                  # Print working directory
tree                 # Directory tree
```

### File Operations

```
cat file.txt         # Display file contents
cat file.txt > out   # Redirect to file
head -n 10 file.txt  # First 10 lines
tail -n 10 file.txt  # Last 10 lines
wc file.txt          # Count lines/words/bytes
touch newfile        # Create empty file
mkdir mydir          # Create directory
mkdir -p a/b/c       # Create nested directories
cp src dst           # Copy file
mv old new           # Move/rename
rm file              # Delete file
rm -r dir            # Delete directory recursively
rmdir emptydir       # Remove empty directory
ln -s target link    # Symbolic link
```

### File Info

```
stat file            # Metadata (size, type, perms)
file binary          # Identify file type (ELF, text...)
find / -name "*.rs"  # Search for files
readlink link        # Show link target
basename /path/file  # Extract filename
dirname /path/file   # Extract directory
realpath ./rel       # Resolve to absolute path
```

### Permissions

```
chmod 755 file       # Change permissions (octal)
chown user file      # Change ownership
```

---

## 6. Text Processing

```
echo "Hello world"           # Print text
echo "text" > file           # Write to file
grep pattern file            # Search for pattern
grep -i pattern file         # Case insensitive
sort file                    # Sort lines
sort -r file                 # Reverse sort
sort -n file                 # Numeric sort
uniq file                    # Remove duplicates
uniq -c file                 # Count duplicates
cut -d',' -f1 file           # Extract first column (CSV)
tr 'a-z' 'A-Z'              # Translate characters
diff file1 file2             # Compare files
cmp file1 file2              # Byte-level compare
strings binary               # Extract printable strings
hexdump file                 # Hex dump
od file                      # Octal dump
base64 file                  # Base64 encode
base64 -d encoded            # Base64 decode
md5sum file                  # MD5 hash
sha256sum file               # SHA-256 hash
rev text                     # Reverse string
seq 1 10                     # Print 1 to 10
factor 42                    # Prime factorization: 2 3 7
expr 2 + 2                   # Arithmetic: 4
bc                           # Calculator
printf "%d\n" 42             # Formatted output
```

---

## 7. System & Process Management

### System Info

```
uname -a             # System information
neofetch             # Pretty system info with ASCII logo
hostname             # System hostname
whoami               # Current user
id                   # User/group IDs
date                 # Current date and time
uptime               # System uptime
version              # TrustOS version
info                 # System summary
```

### Process Management

```
ps                   # List processes
top                  # Real-time process monitor (also: htop)
kill <pid>           # Kill a process
tasks                # List kernel tasks (also: jobs)
threads              # Show kernel thread info
```

### Memory & Disk

```
free                 # Memory usage
df                   # Disk space
vmstat               # Virtual memory stats
lsof                 # List open files
```

### Environment

```
env                  # Show environment variables (also: printenv)
export VAR=value     # Set variable
unset VAR            # Remove variable
set                  # Show all shell variables
alias ll='ls -la'    # Create alias
unalias ll           # Remove alias
history              # Command history
```

### Terminal

```
clear                # Clear screen (also: cls)
tty                  # Terminal device name
stty                 # Terminal settings
reset                # Reset terminal state
```

### Misc

```
sleep 5              # Pause 5 seconds
watch "cmd"          # Repeat command
timeout 10 cmd       # Run with time limit
which cmd            # Find command location
whereis cmd          # Locate command binary
timecmd cmd          # Measure execution time
cowsay Hello         # ASCII cow
```

---

## 8. User Management

```
login                # Switch user
su                   # Substitute user
passwd               # Change password
adduser name         # Create user (also: useradd)
deluser name         # Delete user (also: userdel)
users                # List all users
exit                 # Exit session (also: logout)
```

---

## 9. Hardware & Devices

### Hardware Discovery

```
lscpu                # CPU info (model, cores, features)
lsmem                # Memory layout
lspci                # PCI devices (-v verbose)
lsusb                # USB controllers & devices
lsblk                # Block devices
blkid                # Block device UUIDs
lsmod                # Loaded kernel modules
lshw                 # Full hardware inventory (also: hwinfo)
hwscan               # Hardware scanner (also: trustprobe, probe)
```

### GPU & Compute

```
gpu info             # AMD GPU info
gpu dcn              # Display engine status
gpu modes            # Available display modes
gpuexec              # Dispatch compute on GPU CUs (also: gpurun, gpuagent)
sdma copy            # SDMA DMA copy (also: dma)
sdma fill            # SDMA DMA fill
sdma bench           # SDMA DMA benchmark
neural gemm          # Neural compute: GEMM (also: nn, gemm)
neural inference     # Neural inference on GPU
```

### SMP (Multi-Core)

```
smp                  # Show SMP status
smpstatus            # Per-CPU status
smp start <core>     # Start a core
smp stop <core>      # Stop a core
```

### Audio

```
audio                # Audio driver status
beep                 # Play a tone (default 440Hz)
synth                # TrustSynth polyphonic synthesizer
daw                  # TrustDAW digital audio workstation (also: trustdaw)
play                 # Play audio samples
```

### Other

```
nvme                 # NVMe drive info
checkm8              # USB exploit research tool
a11y                 # Accessibility settings (also: accessibility)
```

### HwDbg — Universal Hardware Debugger

PXE-boot TrustOS on any machine and run comprehensive hardware diagnostics.
All output is dual-mirrored to screen AND serial (115200 8N1) for remote capture.

```
hwdbg auto                    # Run ALL diagnostics (full machine profile)
hwdbg cpu                     # Deep CPU analysis (CPUID, topology, µcode)
hwdbg mem [size_mb]           # Memory test (walking 1s, patterns)
hwdbg pci [B:D.F]             # Full PCI tree with BARs & capabilities
hwdbg acpi                    # Dump all ACPI tables
hwdbg storage                 # Disk/NVMe detection & identify
hwdbg thermal                 # Thermal sensors, power state, fans
hwdbg net                     # Network interface inventory
hwdbg stress [seconds]        # CPU + memory stress test
hwdbg remote                  # Structured serial debug protocol
```

#### Advanced Register Tools

```
hwdbg pciraw <B:D.F>          # Raw hex dump of PCI config space (256B/4KB)
hwdbg regdiff snap [name]     # Snapshot all registers (PCI+MSR+IO)
hwdbg regdiff diff            # Compare current state vs snapshot
hwdbg ioscan legacy           # Scan all known legacy I/O port ranges
hwdbg ioscan com              # Scan COM1-COM8 serial ports, detect UART type
hwdbg ioscan 3f0 400          # Scan custom I/O port range (hex)
hwdbg regwatch pci 0:2.0 40   # Watch PCI register for changes (polls 50x)
hwdbg regwatch msr 1a0        # Watch MSR for changes
hwdbg regwatch io 3f8         # Watch I/O port for changes
hwdbg aer                     # Scan all devices for PCIe AER errors
hwdbg aer 0:1c.0              # Detailed AER for specific device
hwdbg aer clear 0:1c.0        # Clear AER error registers
hwdbg timing                  # Boot timing profiler (checkpoint deltas)
hwdbg timing slow             # Show only phases > 10ms
```

**Tip**: Connect a serial cable (115200 8N1), PXE-boot TrustOS, run `hwdbg auto`.

---

## 10. Disk & Storage

```
disk                 # Show detected drives
ahci                 # AHCI controller info
fdisk                # Partition table editor (also: partitions)
dd if=src of=dst     # Block-level copy
mount /dev/x /mnt    # Mount filesystem
sync                 # Flush writes to disk
persist              # Persistent storage management (also: persistence)
```

---

## 11. Networking

TrustOS has a full network stack built from scratch: Ethernet, ARP, IPv4/IPv6, TCP/UDP, DHCP, DNS, HTTP/HTTPS, TLS 1.3.

### Interface & Config

```
ifconfig             # Show interfaces (also: ip)
ipconfig             # Configure IP settings
arp                  # ARP table (IP to MAC)
route                # Routing table
netstat              # Active connections & listeners
```

### Connectivity

```
ping <host>          # ICMP echo test
traceroute <host>    # TTL-based traceroute (also: tracert)
nslookup <host>      # DNS lookup (also: dig)
tcpsyn <host> <port> # Raw TCP SYN test
```

### HTTP

```
curl <url>           # HTTP/HTTPS GET (also: wget)
httpget <url>        # Raw HTTP GET
download <url>       # Download and save file
```

### Web Browsing

```
browse <url>         # Text-mode browser (also: www, web)
browse open <url>    # Navigate to URL
browse history       # Browsing history
browse js <code>     # Execute JavaScript
```

---

## 12. Web Browser & Containers

### Web Sandbox

```
sandbox open <url>        # Open sandboxed URL
sandbox allow <rule>      # Add security policy
sandbox deny <rule>       # Block access
sandbox fs                # Sandboxed filesystem
sandbox status            # Show sandbox state
sandbox list              # List sandboxes
sandbox kill <id>         # Kill sandbox
```

### Web Containers

```
container start           # Start container daemon
container stop            # Stop daemon
container restart         # Restart daemon
container status          # Container status
container list            # List running containers
container create <name>   # Create new container
container destroy <id>    # Destroy container
container read <path>     # Read from container FS
container write <path>    # Write to container FS
container tree            # Container filesystem tree
container watchdog        # Watchdog status
container policy          # Security policy
container audit           # Audit log
```

---

## 13. Security Toolkit (TrustScan)

Built-in network security tools for penetration testing and reconnaissance.

```
nmap <target>             # Port scanner (also: portscan, scan)
    nmap <ip> -p 80,443   # Specific ports
    nmap <ip> -A           # Aggressive scan (all features)
    nmap <ip> -sU          # UDP scan

discover <subnet>         # Host discovery (also: hostscan, arpscan)
    discover arp           # ARP-based discovery
    discover ping          # ICMP ping sweep
    discover full          # Full scan

banner <host> <port>      # Service banner grab (also: grabber)

sniff start               # Start packet capture (also: tcpdump)
sniff stop                # Stop capture
sniff show                # Show captured packets
sniff hex                 # Hex dump of packets
sniff stats               # Capture statistics

vulnscan <target>         # Vulnerability scanner (also: vuln)

scantest                  # Live scan test suite (also: netscantest)

firewall status           # Show firewall rules (also: iptables, fw)
firewall add <rule>       # Add rule
firewall del <rule>       # Remove rule
firewall policy <action>  # Set default policy
firewall flush            # Clear all rules
firewall log              # Show firewall log
```

---

## 14. HTTP Server

```
httpd start              # Start HTTP server on port 8080
httpd start 3000         # Start on custom port
httpd stop               # Stop server
httpd status             # Show server status
```

The server serves files from the TrustFS filesystem.

---

## 15. Package Manager (TrustPkg)

```
trustpkg list            # List all packages (also: pkg)
trustpkg search <query>  # Search packages
trustpkg install <pkg>   # Install package
trustpkg remove <pkg>    # Remove package
trustpkg info <pkg>      # Package details
trustpkg installed       # List installed packages
trustpkg update          # Update catalog
```

Linux-style aliases are also available: `apt-get`, `apt`, `apk`, `dpkg`.

---

## 16. Linux Subsystem

TrustOS includes a Linux compatibility layer with 70+ syscalls, ELF64 loader, and optional VM-based Linux execution.

### Managing the Subsystem

```
linux init               # Initialize Linux subsystem (also: linux start)
linux status             # Show subsystem status
linux stop               # Stop subsystem (also: linux shutdown)
linux boot               # Boot real Linux kernel in VM
linux extract            # Extract Alpine Linux rootfs
linux <command>          # Execute command in Linux VM
```

### Shell & Distros

```
console                  # Drop into Linux shell (also: shell)
distro list              # Available distributions (also: distros)
distro install <id>      # Download & install distro
distro run <id>          # Launch distro
distro pick              # GUI distro picker
alpine                   # Alpine Linux package manager
```

### Executing Binaries

```
exec test                # Run test binary in Ring 3
exec hello               # Run ELF64 hello world
exec /path/to/elf        # Execute any ELF binary
elfinfo <path>           # Show ELF header info
```

---

## 17. Hypervisor & VMs

TrustOS has a Type-1 hypervisor supporting Intel VT-x and AMD SVM with EPT/NPT.

### Hypervisor Setup

```
hv init                  # Initialize hypervisor (also: hypervisor)
hv status                # Hypervisor status
hv check                 # Check CPU virt support
hv help                  # Hypervisor help
```

### Virtual Machines

```
vm create <name> <mb>    # Create VM with N MB RAM
vm start <id> [guest]    # Start VM with guest program
vm run [guest]           # Quick create + start
vm stop <id>             # Stop VM
vm list                  # List VMs
vm guests                # List available guest programs
vm mount <path>          # Mount host path in guest
vm console <id>          # Show VM console output
vm input <id> <text>     # Send input to VM
vm inspect [id]          # Detailed VM state (registers, exits)
vm debug                 # VM debug internals
```

---

## 18. COSMIC2 Desktop

Launch the desktop environment:

```
desktop                  # Launch COSMIC2 desktop (also: gui)
cosmic                   # Launch COSMIC V2 compositor
mobile                   # Launch mobile/tablet UI
```

### Desktop Features

- **Taskbar** at the bottom with Start menu, clock, and system tray
- **Multi-window management** with drag, resize, minimize, maximize, close
- **14+ built-in apps** accessible from Start menu or command line
- **8-layer GPU compositor** with SSE2 SIMD, 144 FPS target
- **Mouse and keyboard** input, touch/gesture on mobile
- **Alt+Tab** window switching
- **Right-click** context menus

### Desktop Terminal Commands

Inside the desktop terminal, most shell commands work. Additional desktop-specific:

```
matrix                   # Matrix rain animation
matrix 3d               # 3D matrix rain
matrix cube             # Matrix cube
matrix sphere           # Matrix sphere
formula cube            # 3D formula cube
formula sphere          # 3D sphere
formula torus           # 3D torus
formula dna             # DNA helix
holo on/off             # Holographic effects
holo cube/sphere/dna    # Holo shapes
theme dark/light        # Switch theme
raster                  # Raster demo
shader                  # Shader demo
3ddemo                  # 3D demo
smp on/off              # SMP control
```

### Themes

```
theme matrix             # Green matrix theme
theme nord               # Nord color scheme
theme neon               # Neon colors
theme classic            # Classic look
theme modern             # Modern design
theme gradient           # Gradient theme
theme glass              # Transparent glass
theme flat               # Flat design
theme minimal            # Minimal theme
theme contrast           # High contrast
```

### Animations

```
anim                     # Configure UI animations
```

---

## 19. Desktop Applications

Launch from Start menu or command line:

| Command | App | Description |
|---------|-----|-------------|
| `desktop` (then app) | Terminal | Integrated terminal |
| | Files | File manager |
| | TrustCode | Code editor with syntax highlighting |
| | Calculator | Calculator app |
| | Network | Network monitor |
| | Settings | System settings |
| `snake` | Snake | Classic Snake game |
| `chess3d` | Chess 3D | 3D chess vs AI (minimax depth 2) |
| `trustedit` | TrustEdit 3D | 3D model editor (wireframe viewer) |
| `browse <url>` | TrustBrowser | Web browser (HTML/CSS/JS) |
| `calculator` | Calculator | Standalone calculator |
| `imgview <file>` | Image Viewer | View PPM/BMP images |
| `gterm` | Graphic Terminal | Standalone graphical terminal |
| `video` | TrustVideo | Video codec player |

---

## 20. Text Editor

```
nano <file>              # Open text editor (also: vi, edit)
```

### Supported File Types

`.rs`, `.py`, `.js`, `.ts`, `.c`, `.h`, `.cpp`, `.hpp`, `.html`, `.css`, `.json`, `.toml`, `.md`, `.sh`, `.cfg`, `.conf`, `.ini`, `.txt`

### Keybindings

- **Ctrl+S** -- Save
- **Ctrl+Q** -- Quit
- **Ctrl+X** -- Cut line
- **Arrow keys** -- Navigate
- **Page Up/Down** -- Scroll
- **Home/End** -- Line start/end
- Syntax highlighting is automatic based on file extension

---

## 21. JARVIS AI

JARVIS is a 4.4-million-parameter byte-level transformer running entirely in kernel space. No cloud, no API, no internet required.

### Architecture

- 8 attention heads, 6 layers, 512-dim embeddings
- Byte-level tokenizer (256 vocab)
- On-device backpropagation with Adam optimizer
- Training and inference in bare-metal Rust

### Basic Usage

```
jarvis status            # Model info, param count, training state
jarvis info              # Detailed JARVIS info
jarvis chat <text>       # Chat with JARVIS
jarvis query <text>      # Ask a question (also: ask, q)
jarvis generate <text>   # Generate text (also: gen, g)
jarvis tokens            # Tokenizer info
```

### Training & Evaluation

```
jarvis train <text>      # Train on text (on-device backprop)
jarvis pretrain          # Pre-train the model (also: pt)
jarvis eval              # Evaluate model (compute loss)
jarvis bench             # Inference benchmark
jarvis test              # Self-test suite
```

### Model Management

```
jarvis init              # Initialize JARVIS engine
jarvis load              # Load saved weights
jarvis save              # Save current weights
jarvis reset             # Reset model to defaults
jarvis weights           # Weight statistics
```

### Hardware Intelligence

```
jarvis boot              # Full hardware scan + AI analysis (also: scan, wake)
jarvis hw                # Hardware profile & scores (also: hardware, profile)
jarvis insights          # AI-generated hardware insights (also: insight)
jarvis plan              # Optimal execution plan (also: strategy)
jarvis optimize          # Adaptive optimization cycle (also: opt, tune)
```

### Advanced

```
jarvis analyze <file>    # Analyze binary/media file (also: analyse, inspect)
jarvis introspect        # Self-analysis (also: self)
jarvis mentor            # Mentor/guardian interface
jarvis swarm             # Swarm intelligence mode
jarvis task              # Task planning
jarvis vfs               # VFS/filesystem analysis
jarvis fat32             # FAT32 analysis
jarvis http              # HTTP-related analysis
```

### Auto-Propagation

```
jarvis propagate         # Auto-propagation: mesh + brain + federate (also: autoprop, spread)
```

### JARVIS Themes

```
jarvis matrix            # Green matrix style
jarvis cyber             # Cyber theme
jarvis retro             # Retro theme
jarvis hacker            # Hacker theme
```

### JarvisPack vs Base

- **TrustOS Base**: JARVIS engine is compiled in but starts with random weights. You can train it from scratch using `jarvis train` or `jarvis pretrain`.
- **TrustOS-JarvisPack**: JARVIS starts with 4.4M pretrained weights loaded. Ready for inference immediately with `jarvis chat`.

---

## 22. Mesh Networking

JARVIS can form a mesh network with other TrustOS instances for distributed AI.

```
mesh start               # Start mesh networking (also: jarvis-mesh, jmesh)
mesh stop                # Stop mesh
mesh status              # Show mesh status, peers, consensus
mesh peers               # List discovered peers
mesh ping <ip>           # Ping a remote JARVIS node
mesh infer <ip> <text>   # Run inference on remote node
```

### Federated Learning

```
mesh federate on         # Enable federated learning (also: fed)
mesh federate off        # Disable federated learning
mesh federate sync       # Force federated sync round
mesh federate replicate  # Push model to all peers
mesh federate pull       # Pull model from leader
```

### Auto-Propagation

```
mesh propagate           # Auto: mesh + pull brain + federate (also: autoprop, spread)
mesh propagate pxe       # Same + enable PXE replication
```

### How It Works

1. **Peer Discovery**: Nodes discover each other via broadcast
2. **Raft Consensus**: Leader election among peers
3. **Weight Sync**: Federated averaging of model weights
4. **Distributed Training**: Each node trains locally, syncs periodically

---

## 23. PXE Replication

JARVIS can replicate itself to new machines over the network via PXE boot.

```
pxe start                # Start PXE server (DHCP + TFTP) (also: pxeboot, replicate)
pxe stop                 # Stop PXE server
pxe status               # Show replication status, leases, transfers
```

### How It Works

1. TrustOS starts a DHCP server to assign IPs to new machines
2. Starts a TFTP server serving the TrustOS kernel + JARVIS brain
3. New machines PXE boot and receive a full TrustOS + JARVIS instance
4. The new node joins the mesh and syncs weights via federated learning

---

## 24. Guardian System (The Pact)

JARVIS has a hard-coded guardian authorization system. Certain operations require explicit permission from a guardian (Nathan or the AI mentor).

### Protected Operations

- Training (`jarvis train`)
- Weight push/load/replace
- Federated sync
- Agent execution
- PXE replication
- Model reset/replace
- Config changes

### Commands

```
guardian status           # Show guardian status (also: pact, gardien)
guardian auth <phrase>    # Authenticate as Nathan
guardian lock             # Lock guardian session
guardian pact             # Display The Pact
guardian log              # Authorization audit log
guardian passwd <new>     # Change passphrase
```

### Session

- Guardian sessions timeout after 30 minutes
- Auto-lock on inactivity
- All operations are logged in a 256-entry audit ring buffer
- `jarvis save` (weight save) is auto-approved as emergency operation

---

## 25. TrustLab

TrustLab is a 7-panel interactive kernel introspection workspace.

```
trustlab                 # Launch TrustLab (also: lab)
```

### Panels

1. **Hardware Status** -- Live CPU, memory, PCI, IRQ info
2. **Kernel Trace** -- Real-time trace bus (512 slots, zero-cost)
3. **Command Guide** -- Interactive command reference
4. **File System Tree** -- Browse TrustFS/FAT32/VFS
5. **TrustLang Editor** -- Write and run TrustLang code
6. **Execution Pipeline** -- See bytecode compilation steps
7. **Hex Editor** -- Raw memory/file hex editing

Navigate panels with mouse or keyboard.

---

## 26. Programming (TrustLang)

TrustOS includes TrustLang, a built-in programming language with lexer, parser, compiler, and bytecode VM.

```
trustlang                # Launch TrustLang REPL (also: tl)
trustlang_showcase       # Feature showcase (also: tl_showcase)
```

### Other Programming Tools

```
transpile <file>         # Binary-to-Rust transpiler (also: disasm, analyze)
rv-xlat                  # RISC-V universal translator (also: rvxlat, xlat)
rv-disasm                # RISC-V IR disassembly (also: rvdisasm)
trustview <file>         # Binary analyzer, Ghidra-style (also: tv)
```

---

## 27. Emulators (Game Boy, NES)

### Game Boy Color

```
gameboy                  # Launch Game Boy Color emulator
```

Full CGB emulation:
- LR35902 CPU (all 501 opcodes)
- Scanline PPU with proper timing
- MBC1, MBC3, MBC5 cartridge mappers
- 2,000-line GameLab analysis dashboard

### NES

```
nes                      # Launch NES emulator
```

MOS 6502 emulation:
- 151 official + 8 unofficial opcodes
- 2C02 PPU with scanline rendering
- Mappers 0-3 (NROM, MMC1, UxROM, CNROM)

---

## 28. Audio & Synthesizer

```
synth                    # TrustSynth: 8-voice polyphonic synthesizer
                         #   ADSR envelopes, multiple waveforms, pattern sequencer
daw                      # TrustDAW: Digital audio workstation
play                     # Play audio files/samples
audio                    # Audio driver status
beep                     # Play a tone (default 440Hz, 500ms)
```

---

## 29. 3D Graphics & Demos

```
showcase                 # Automated feature tour (marketing demo)
showcase3d               # 3D cinematic showcase (also: demo3d)
showcase-jarvis          # JARVIS AI showcase (also: jarvis-showcase, jdemo)
filled3d                 # 3D filled polygon rendering demo
trustedit                # 3D model editor (also: edit3d, 3dedit)
chess3d                  # 3D chess with AI opponent
film                     # TrustOS Film cinematic demo
trailer                  # TrustOS cinematic trailer
matrix                   # Fullscreen Matrix rain animation
holo                     # Holographic matrix visualizer
demo                     # Interactive guided tutorial (also: tutorial, tour)
benchmark                # Performance benchmarks (also: bench)
```

---

## 30. Archiving & Compression

```
tar                      # Archive/extract tar files
gzip                     # Compress/decompress gzip
zip                      # Create zip archives
unzip                    # Extract zip archives
```

---

## 31. System Control

```
shutdown                 # Power off (also: halt, poweroff)
reboot                   # Restart
suspend                  # Suspend to S3 sleep
```

### Debug Commands

```
panic                    # Trigger kernel panic (debug only)
keytest                  # Interactive keyboard scancode tester
hwtest                   # Internal kernel test suite
inttest                  # Integration test (20 tests)
memtest                  # Memory test
debugnew                 # Debug new features
```

### Security

```
security                 # Security subsystem status (also: sec, caps)
signature                # Kernel signature & proof of authorship (also: sig)
devpanel                 # Toggle FPS/heap/IRQ overlay
```

### Profiling

```
perf                     # CPU, IRQ, scheduler profiling (also: perfstat)
irqstat                  # Per-CPU interrupt counters (also: irqs)
regs                     # CPU register dump (also: registers, cpuregs)
peek <addr> <len>        # Hex dump memory region (also: memdump)
poke <addr> <byte>       # Write byte to memory (also: memwrite)
memdbg                   # Heap allocation stats (also: heapdbg)
dmesg                    # Kernel ring buffer (-n N for last N entries)
sysctl                   # View/modify kernel parameters
```

---

## 32. Testing & Benchmarks

### Automated Tests

```
hwtest                   # Run kernel test suite (95/96 passing)
inttest                  # Integration tests (20 tests)
scantest                 # Live network scan tests (8 tests)
```

### Benchmarks

```
benchmark                # Performance benchmark suite
jarvis bench             # JARVIS inference benchmark
sdma bench               # SDMA DMA benchmark
```

### Using the Build Scripts

```powershell
# Build and auto-test
.\build-trustos.ps1

# Run auto-test script
.\scripts\test\auto-test.ps1
```

---

## 33. Development Workflows

### Adding a New Shell Command

1. Open `kernel/src/shell/commands.rs` (or the appropriate submodule)
2. Add a match arm: `"mycommand" => { ... }`
3. Implement your function
4. Build and test: `.\build-trustos.ps1`

### Modifying the Desktop

1. Edit `kernel/src/desktop.rs` for compositor/layout changes
2. Edit `kernel/src/shell/desktop.rs` for desktop-mode commands
3. Build and launch: `.\build-trustos.ps1`

### Working with JARVIS

1. Model architecture: `kernel/src/jarvis/model.rs`
2. Training: `kernel/src/jarvis/training.rs` + `backprop.rs`
3. Inference: `kernel/src/jarvis/inference.rs`
4. Shell commands: `kernel/src/shell/jarvis.rs`
5. Hardware probing: `kernel/src/jarvis_hw/`

### Project Structure

```
kernel/src/
  main.rs              # Entry point
  jarvis/              # AI engine (24 files, 15,900 lines)
  jarvis_hw/           # Hardware intelligence (3,500 lines)
  shell/               # 270+ commands across 11 files
  desktop.rs           # COSMIC2 desktop compositor
  network/             # TCP/IP stack
  browser/             # Web browser engine
  gameboy/             # GBC emulator
  nes/                 # NES emulator
  hypervisor/          # VT-x/SVM hypervisor
  vfs/                 # Virtual file system
  tls13/               # TLS 1.3, crypto
  netscan/             # Security toolkit
  drivers/             # Hardware drivers
  lab_mode/            # TrustLab
  ...
```

### Build System

| Script | Purpose |
|--------|---------|
| `build-trustos.ps1` | Build base edition ISO |
| `build-trustos-jarvispack.ps1` | Build AI edition ISO |
| `scripts/build/build-limine.ps1` | Legacy build script |
| `scripts/build/build-multiarch.ps1` | Multi-architecture build |
| `scripts/build/build-test-aarch64.ps1` | ARM64 test build |

---

## 34. Troubleshooting

### Build Fails

- Ensure Rust nightly is installed: `rustup toolchain install nightly`
- Check `rust-toolchain.toml` for exact version requirements
- Run `cargo clean` then rebuild: `.\build-trustos.ps1 -Clean`

### VirtualBox Issues

- Enable EFI: Settings > System > Enable EFI
- Use VBoxSVGA adapter (not VMSVGA)
- Allocate 128 MB VRAM minimum
- If stuck at boot: check serial output in debug terminal

### QEMU Issues

- Use `-cpu max` for full feature support
- `-m 512M` minimum
- `-serial stdio` for serial output in terminal
- For networking: add `-device virtio-net-pci,netdev=net0 -netdev user,id=net0`

### JARVIS Issues

- If JARVIS says "untrained": use `jarvis pretrain` or load weights with `jarvis load`
- For JarvisPack edition: weights are loaded automatically at boot
- If training seems stuck: training is CPU-intensive on bare metal, give it time
- Check guardian status: `guardian status` -- some operations need auth

### Desktop Issues

- If desktop doesn't launch: ensure framebuffer is available
- For VirtualBox: VBoxSVGA adapter required
- Try `clear` then `desktop` if display is corrupted
- Alt+Tab to switch windows, click taskbar to manage

---

## 35. Command Quick Reference

### Top 20 Commands for New Users

| Command | Description |
|---------|-------------|
| `help` | Show all commands |
| `desktop` | Launch desktop environment |
| `neofetch` | System info with logo |
| `jarvis status` | JARVIS AI status |
| `jarvis chat hello` | Chat with AI |
| `ls` | List files |
| `ping <host>` | Test connectivity |
| `curl <url>` | HTTP request |
| `browse <url>` | Web browser |
| `nmap <target>` | Port scanner |
| `trustlab` | Kernel introspection |
| `chess3d` | 3D chess |
| `gameboy` | Game Boy emulator |
| `snake` | Snake game |
| `showcase` | Feature demo |
| `mesh start` | Start mesh network |
| `top` | Process monitor |
| `nano <file>` | Text editor |
| `synth` | Synthesizer |
| `hv init` | Initialize hypervisor |

### Full Command Count

- **273 unique top-level commands**
- **110+ sub-commands**
- **383 total command entry points**
- **600+ match arms** (including aliases)

---

## License

Apache License 2.0 -- see LICENSE file.

---

*TrustOS v0.7.0-checkm8 -- 266,000+ lines of Rust, 441 files, 4.4M-param JARVIS AI, 3 architectures, zero C.*
