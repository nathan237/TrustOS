<div align="center">

# TrustOS

### **Trust** the code. **Rust** is the reason.

**A fully auditable, bare-metal operating system -- 190,000+ lines of pure Rust. Zero C. Zero secrets.**

*One dev. One OS. Nothing to hide.*

**The first bare-metal OS with a built-in real-time CyberLab -- kernel introspection + network security toolkit.**

[![Build](https://img.shields.io/badge/build-passing-brightgreen?style=for-the-badge)]()
[![Rust](https://img.shields.io/badge/100%25%20Rust-F74C00?style=for-the-badge&logo=rust&logoColor=white)]()
[![Lines](https://img.shields.io/badge/code-190%2C000%2B%20lines-blue?style=for-the-badge)]()
[![Architectures](https://img.shields.io/badge/arch-x86__64%20%7C%20ARM64%20%7C%20RISC--V-blueviolet?style=for-the-badge)]()
[![Version](https://img.shields.io/badge/version-0.6.0--MultiArch-orange?style=for-the-badge)]()
[![Tests](https://img.shields.io/badge/tests-95%2F96%20(99%25)-brightgreen?style=for-the-badge)]()
[![Auditable](https://img.shields.io/badge/fully-auditable-00C853?style=for-the-badge)]()
[![License](https://img.shields.io/badge/license-MIT-green?style=for-the-badge)]()
[![Author](https://img.shields.io/badge/created%20by-Nated0ge-ff69b4?style=for-the-badge&logo=github&logoColor=white)](https://github.com/nathan237)

[![Watch the demo](https://img.shields.io/badge/%E2%96%B6%20Watch%20Demo-YouTube-red?style=for-the-badge&logo=youtube&logoColor=white)](https://youtu.be/RBJJi8jW1_g)

[Why "Trust"?](#-why-trustos) · [Run on Anything](#-run-on-anything) · [Features](#-features) · [Quick Start](#-quick-start) · [Architecture](#-architecture) · [Contributing](#-contributing)

---

</div>

## What's New (v0.6.0)

| Feature | Details |
|---------|---------|
| **Multi-Architecture** | TrustOS compiles for **x86_64**, **ARM64 (aarch64)**, and **RISC-V 64**. All 3 targets build with 0 errors. |
| **Android Boot** | Flash TrustOS to any Android phone via `fastboot flash boot`. Full boot.img v2 pipeline with SoC profiles (Pixel, OnePlus, RPi, QEMU). |
| **Raspberry Pi** | Boot from SD card on RPi 4/5. Auto-generated `kernel8.img` + `config.txt`. |
| **Universal Installer** | One script, 9 targets: `.\trustos-install.ps1`. PC USB, ISO, Android, RPi, ARM boards, RISC-V boards, QEMU (all 3 archs). |
| **Touch & Gesture** | Mobile desktop with multi-touch input, gesture recognition (swipe, pinch, rotate, long-press). |
| **GPU Emulation** | 1,770-line GPU compute emulation layer -- virtualizes CPU cores as GPU compute units. |

Previous highlights: [CyberLab v0.5.0](#v050----cyberlab) · [Emulators v0.4.0](#v040----emulators--gamelab) · [Shell + HTTP v0.4.1](#v041----shell-scripting--http--security)

---

## Why "TrustOS"?

The name says it all: **Trust** + **Rust**.

In a world where your operating system is a black box -- millions of lines of legacy C/C++, binary blobs, proprietary drivers, telemetry you can't disable -- **how do you know what your computer is actually doing?**

TrustOS is the answer: **every single line is open, readable, and auditable.**

- **Fully auditable** -- 190,000 lines of Rust, all on GitHub. No binary blobs. No hidden code.
- **Memory safe by design** -- Rust's ownership model prevents entire categories of vulnerabilities.
- **Zero dependencies on C** -- no libc, no glibc, no C runtime. Every driver, every protocol, every pixel is Rust.
- **Readable** -- one person wrote it. If one person can build it, one person can understand it.

> *"The only OS where you can trace every packet, every pixel, and every keystroke back to its source code."*

| Metric | Value |
|--------|-------|
| **Total code** | 190,000+ lines of Rust |
| **Source files** | 373 `.rs` files |
| **Architectures** | x86_64, aarch64 (ARM64), riscv64 (RISC-V) |
| **Boot targets** | PC, Android, Raspberry Pi, ARM boards, RISC-V boards |
| **Kernel modules** | 44+ independent modules |
| **Boot time** | < 1 second |
| **Desktop FPS** | 144 FPS (SSE2 SIMD) |
| **Auto-tests** | 95/96 passing (99%) |
| **C code** | 0 lines |
| **External deps** | 0 (everything from scratch) |

### TrustOS vs The World

| | Traditional OS | TrustOS |
|---|:---:|:---:|
| **Language** | C/C++ with 40 years of memory bugs | 100% Rust -- memory safe by design |
| **Codebase** | Millions of lines, impossible to audit | 190K lines, one person can read it all |
| **Platforms** | Tied to one architecture | x86_64 + ARM64 + RISC-V from one codebase |
| **Binary blobs** | Everywhere | None. Zero. |
| **Telemetry** | Opt-out (maybe) | Doesn't exist -- verify it yourself |
| **Build** | Complex cross-compilation toolchains | `cargo build` -- that's it |

### Vision

TrustOS is not another Linux clone -- it's a living laboratory. The goal is to build the first bare-metal environment where you can observe, understand, and experiment with every layer of an operating system in real time. TrustLab already lets you trace kernel events, inspect memory, and monitor virtual machines from the hypervisor level. TrustOS is becoming a full **open-source Cyber Range**: a training and research platform for cybersecurity where pentesters, researchers, and students can boot a guest OS, observe its processes agentlessly, trace exploits live, and analyze malware in a fully transparent environment. Every line is auditable Rust, every architectural decision is documented. The code is open. The vision is simple: **if you can't observe your system, you can't trust it.**

---

## Run on Anything

TrustOS boots on **PCs, phones, single-board computers, and VMs** -- from a single codebase.

```
.\trustos-install.ps1              # Interactive -- pick your target
.\trustos-install.ps1 -List        # Show all 9 targets
.\trustos-install.ps1 -Target <t>  # Direct install
```

| Target | Arch | Method | Status |
|--------|------|--------|--------|
| `pc-usb` | x86_64 | Limine UEFI + BIOS to USB drive | Production |
| `pc-iso` | x86_64 | Bootable ISO (CD/DVD/VM) | Production |
| `android` | ARM64 | `fastboot flash boot` (Pixel, OnePlus, Xiaomi...) | Ready |
| `rpi-sd` | ARM64 | SD card (RPi 4/5): kernel8.img + config.txt | Ready |
| `arm-generic` | ARM64 | Flat binary for U-Boot / JTAG / TFTP | Ready |
| `riscv` | RISC-V | Binary for OpenSBI + U-Boot (VisionFive, Milk-V) | Ready |
| `qemu-x86` | x86_64 | QEMU x86_64 UEFI test | Ready |
| `qemu-arm` | ARM64 | QEMU ARM64 virt machine | Ready |
| `qemu-riscv` | RISC-V | QEMU RISC-V virt machine | Ready |

**Android phones**: Requires unlocked bootloader (`fastboot flashing unlock`). Works on Pixel, OnePlus, Xiaomi, most Android devices. Samsung needs OEM unlock in Developer Settings first. Apple is not supported (hardware-locked bootchain).

**Raspberry Pi**: Auto-generates `kernel8.img`, `config.txt`, `cmdline.txt`. Copy to FAT32 SD card along with RPi firmware files. Supports RPi 4 and RPi 5.

---

## Features

### TrustLab -- Real-Time Kernel Introspection Laboratory

> **World's first: no other bare-metal OS has a built-in, real-time kernel introspection lab.**

7-panel interactive workspace that lets you **watch the OS kernel run in real-time -- from inside itself**.

| Panel | What it shows |
|-------|---------------|
| **Hardware Status** | CPU gauge, heap usage, IRQ rate, uptime, allocation stats |
| **Kernel Trace** | Scrolling event log with structured syscall tracing (50+ syscalls) |
| **Command Guide** | Searchable reference of ~55 commands with fuzzy search |
| **File System Tree** | Interactive VFS browser with expand/collapse |
| **TrustLang Editor** | Syntax-highlighted editor with F5 execution |
| **Execution Pipeline** | Real-time data flow visualization |
| **Hex Editor** | Raw byte inspection with color-coded display |

Full mouse interaction, zero-cost 512-slot trace bus, cinematic demo mode (`demo`). Launch: `trustlab` or from desktop.

### COSMIC2 Desktop Environment
- **Multi-layer GPU compositor** with 8 rendering layers, SSE2 SIMD, 144 FPS
- **Taskbar, dock, start menu**, window management, settings panel
- **14+ desktop apps**: Terminal, Files, TrustCode, Calculator, Network, Snake, Chess 3D, TrustBrowser, TrustEdit 3D...
- **Touch & gesture** input for mobile/tablet deployment
- **HoloMatrix 3D backgrounds**: volumetric wireframe scenes

### Network Stack (from scratch)
- **VirtIO-net** driver, **TCP/IP** (ARP, DHCP, DNS, TCP, UDP, ICMP), **IPv6 + ICMPv6**
- **TLS 1.3** -- full handshake, X.509 certs, crypto from scratch
- **HTTP/HTTPS client** (`curl`, `wget`) + **HTTP server** (`httpd start/stop/status`)
- **Live tested against google.com** -- nmap, curl, ping, DNS all verified on real internet

### TrustScan -- Network Security Toolkit
- Port scanner, packet sniffer, banner grabber, host discovery, traceroute, vulnerability scanner
- Commands: `netscan scan`, `netscan sniff`, `netscan banner`, `netscan discover`, `netscan trace`, `netscan vuln`

### Game Boy Color Emulator + GameLab
- **Full CGB emulator**: LR35902 CPU (all 501 opcodes), scanline-accurate PPU, dual VRAM banks, 8+8 palettes, MBC1/3/5
- **GameLab**: 2,000-line real-time analysis dashboard -- memory search, watch list, tile viewer, trace log, breakpoints, speed control, save/load state

### NES Emulator
- MOS 6502 CPU (151 official + 8 unofficial opcodes), 2C02 PPU, mappers 0-3, controller input

### Built-in Apps & Tools

| Category | Highlights |
|----------|-----------|
| **Browser** | HTML/CSS parser, JS engine, HTTPS, TLS 1.3 |
| **Code Editor** | TrustCode: Rust syntax highlighting, line numbers, file save/load |
| **Language** | TrustLang: Lexer, Parser, Compiler, Bytecode VM |
| **Video** | Custom `.tv` codec, fire/matrix/plasma effects, 60-72 FPS |
| **3D Engine** | Formula3D wireframe renderer, TrustEdit 3D model editor |
| **Chess** | Full 3D chess with AI (minimax depth 2), look-at camera |
| **Audio** | 8-voice polyphonic synthesizer, ADSR, pattern sequencer |
| **Packages** | TrustPkg: 30+ packages across 7 categories |
| **Scripting** | Shell scripting: variables, loops, if/else, command substitution |
| **Film** | Built-in 2-minute cinematic animated explainer (`film` command) |

### Hypervisor & Device Emulation
- **Intel VT-x (VMX)** and **AMD-V (SVM)** dual-backend
- **EPT/NPT**, VMCS, VMCB, VMI Engine for agentless guest monitoring
- **ACPI tables** (RSDP v2, XSDT, MADT, FADT, DSDT with AML)
- **PIC 8259A**, **PIT 8254**, **CMOS RTC**, **ACPI PM Timer** emulation

### Linux Compatibility Layer

| Component | Status |
|-----------|--------|
| Syscall interface | 70+ Linux syscalls |
| ELF64 loader | PATH search, shebang, auxiliary vector |
| Ring 3 userland | SYSCALL/SYSRET, IRETQ |
| Process model | fork/exit/wait, process groups, sessions |
| PTY/TTY | POSIX line discipline, pseudo-terminal pairs |
| Job control | SETPGID, SETSID, controlling TTY |
| chroot | Per-process root directory |

### Security
- **Ed25519 signatures** (RFC 8032) for kernel authentication
- **Capability-based** security model
- User auth: login, su, passwd, adduser
- File permissions: chmod, chown
- Ring 0/3 process isolation

---

## Quick Start

### Option A: Universal Installer (recommended)

```powershell
git clone https://github.com/nathan237/TrustOS.git
cd TrustOS
.\trustos-install.ps1           # Interactive mode -- pick your target
```

### Option B: Download the ISO

Grab the latest ISO from [**Releases**](https://github.com/nathan237/TrustOS/releases):

```bash
# QEMU (BIOS)
qemu-system-x86_64 -cdrom trustos.iso -m 512M -cpu max -smp 4 -display gtk -vga std -serial stdio

# QEMU (UEFI)
qemu-system-x86_64 -cdrom trustos.iso -m 512M -machine q35 -cpu max -smp 4 \
  -drive if=pflash,format=raw,readonly=on,file=/usr/share/OVMF/OVMF_CODE.fd \
  -display gtk -vga std -serial stdio
```

### Option C: Build from source

```bash
# Linux / macOS / WSL
git clone https://github.com/nathan237/TrustOS.git && cd TrustOS && ./setup.sh

# Windows
git clone https://github.com/nathan237/TrustOS.git; cd TrustOS; .\setup.ps1
```

<details>
<summary>Manual build steps</summary>

```bash
# Prerequisites: Rust nightly, QEMU, xorriso, OVMF

# Debian/Ubuntu
sudo apt install qemu-system-x86 xorriso ovmf

# Build & run
make run            # QEMU UEFI
make run-bios       # QEMU BIOS
make iso            # ISO only

# Multi-architecture
cargo build --target x86_64-unknown-none -p trustos_kernel
cargo build --target aarch64-unknown-none -p trustos_kernel
cargo build --target riscv64gc-unknown-none-elf -p trustos_kernel

# Windows
.\trustos-install.ps1 -Target qemu-x86    # Quick test
.\build-limine.ps1                          # Full ISO build
```
</details>

### First commands to try

| Command | What it does |
|---------|-------------|
| `showcase` | Automated feature tour |
| `desktop` | Launch COSMIC2 desktop |
| `trustlab` | Kernel introspection lab |
| `neofetch` | System info |
| `chess3d` | 3D chess vs AI |
| `gameboy` | Game Boy Color emulator |
| `nes` | NES emulator |
| `httpd start` | Built-in web server |
| `netscan scan` | Network port scanner |
| `help` | All 200+ commands |

---

## Architecture

```
+-------------------------------------------------------------+
|                     Applications                            |
|  TrustCode . TrustLang . TrustBrowser . Games . Terminal    |
|  Game Boy Color . NES . GameLab . TrustEdit 3D              |
+---------+---------+----------+---------+--------------------+
|         |         |          |         |                    |
|  COSMIC2 Desktop Compositor                                 |
|  8-layer GPU compositing . SSE2 SIMD . 144 FPS . Touch     |
+---------+---------+----------+---------+--------------------+
|  VFS    | Network | Linux    | Graphics| TrustVideo         |
|  ramfs  | TCP/IP  | Compat   | TrustGL |                    |
|  procfs | TLS 1.3 | 70+      | 3D Mesh | Fire/Plasma        |
|  devfs  | DHCP/DNS| syscalls | Formula | Matrix Rain        |
|  TrustFS| VirtIO  | ELF      | HoloMat |                    |
|  FAT32  | HTTP/S  | Alpine   |         |                    |
+---------+---------+----------+---------+--------------------+
|  TrustPkg . NetScan . Scripting . IPv6 . GPU Emu           |
+---------+---------+----------+---------+--------------------+
|                    TrustOS Kernel                           |
|  Memory . Scheduler . IPC . Security . Drivers . Syscalls  |
|  SSE2 SIMD . SMP . Hypervisor (VT-x/SVM) . Touch/Gesture  |
+---------+---------+----------+---------+--------------------+
|   x86_64 (Limine UEFI/BIOS)                                |
|   aarch64 (Android fastboot / RPi SD / U-Boot)             |
|   riscv64 (OpenSBI + U-Boot)                               |
+-------------------------------------------------------------+
```

### Key Modules

| Module | Lines | Description |
|--------|-------|-------------|
| `shell.rs` | ~17,000 | Command interpreter, 200+ commands, showcase |
| `desktop.rs` (x2) | ~18,000 | COSMIC2 desktop + mobile desktop |
| `hypervisor/` | ~10,000 | VT-x/SVM, EPT/NPT, VMI, ACPI, PIC/PIT/RTC |
| `network/` | ~5,000 | TCP/IP stack, TLS 1.3, HTTPS |
| `compositor/` | ~3,000 | Multi-layer GPU compositor |
| `browser/` | ~2,500 | HTML/CSS parser, JS engine |
| `linux/` | ~3,000 | Linux syscall emulation, ELF loader |
| `gameboy/` | ~1,870 | Game Boy Color emulator |
| `game_lab.rs` | ~2,025 | GameLab analysis dashboard |
| `gpu_emu.rs` | ~1,770 | GPU compute emulation layer |
| `nes/` | ~1,466 | NES emulator |
| `formula3d.rs` | ~1,500 | Wireframe 3D engine |
| `chess.rs` + `chess3d.rs` | ~2,100 | Chess engine + 3D renderer |
| `tls13/` | ~2,000 | TLS 1.3, crypto, X.509 |
| `netscan/` | ~900 | Network security toolkit |
| `gesture.rs` | ~730 | Touch gesture recognition |
| `ed25519.rs` | ~720 | Ed25519 signatures (RFC 8032) |
| `scripting.rs` | ~640 | Shell scripting engine |
| `httpd.rs` | ~410 | HTTP web server |
| `touch.rs` | ~400 | Multi-touch input layer |
| `android_boot.rs` | ~330 | Android boot.img parser |
| `trustpkg.rs` | ~290 | Package manager |
| `android_main.rs` | ~180 | Android aarch64 entry |

### Project Structure
```
kernel/src/
+-- main.rs              # Kernel entry point
+-- arch/
|   +-- x86_64/          # x86_64 boot, interrupts, memory, serial
|   +-- aarch64/         # ARM64 boot, android_entry.S, serial
|   +-- riscv64/         # RISC-V boot, interrupts, serial
+-- android_boot.rs      # Android boot.img v2/v4 parser
+-- android_main.rs      # Android bare-metal entry
+-- shell.rs             # 200+ commands + showcase
+-- desktop.rs           # COSMIC2 desktop manager
+-- touch.rs             # Multi-touch input
+-- gesture.rs           # Gesture recognition
+-- gpu_emu.rs           # GPU compute emulation
+-- compositor/          # 8-layer GPU compositor
+-- browser/             # HTML/CSS/JS browser engine
+-- network/             # TCP/IP, DHCP, DNS, IPv6, ICMPv6
+-- tls13/               # TLS 1.3, crypto, X.509
+-- netscan/             # Network security toolkit (6 modules)
+-- gameboy/             # Game Boy Color emulator
+-- game_lab.rs          # GameLab analysis dashboard
+-- nes/                 # NES emulator
+-- trustlang/           # Compiler + VM
+-- hypervisor/          # VT-x/SVM, EPT/NPT, VMI, ACPI emulation
+-- vfs/                 # TrustFS, FAT32, procfs, devfs
+-- linux_compat/        # 70+ Linux syscalls
+-- framebuffer/         # SSE2 SIMD rendering
+-- graphics/            # 3D, raytracer, HoloMatrix
+-- video/               # TrustVideo codec
+-- drivers/             # AHCI, USB, VirtIO, input
+-- security/            # Capability model, auth
+-- httpd.rs             # HTTP web server
+-- trustpkg.rs          # Package manager
+-- scripting.rs         # Shell scripting engine
+-- ed25519.rs           # Ed25519 signatures
+-- signature.rs         # Kernel signatures
+-- chess.rs / chess3d.rs
+-- model_editor.rs
+-- formula3d.rs
```

---

## 200+ Built-in Commands

<details>
<summary><strong>Filesystem (25+)</strong></summary>

`ls` `cd` `pwd` `mkdir` `rmdir` `touch` `rm` `cp` `mv` `cat` `head` `tail` `wc` `stat` `tree` `find` `grep` `ln` `readlink` `chmod` `chown` `sort` `uniq` `cut` `diff`
</details>

<details>
<summary><strong>Network (20+)</strong></summary>

`ifconfig` `ping` `curl` `wget` `nslookup` `arp` `route` `netstat` `traceroute` `browse` `download` `httpget` `tcpsyn` `ip` `dig` `httpd` `netscan`
</details>

<details>
<summary><strong>Security & Scanning (10+)</strong></summary>

`netscan scan` `netscan sniff` `netscan banner` `netscan discover` `netscan trace` `netscan vuln` `login` `su` `passwd` `adduser`
</details>

<details>
<summary><strong>Package Management</strong></summary>

`trustpkg list` `trustpkg search` `trustpkg install` `trustpkg remove` `trustpkg info` `trustpkg installed` `trustpkg update`
</details>

<details>
<summary><strong>System (25+)</strong></summary>

`ps` `top` `free` `df` `uname` `dmesg` `mount` `umount` `lspci` `lscpu` `lsblk` `lsmem` `lsusb` `lsmod` `lshw` `vmstat` `iostat` `sysctl` `kill` `killall` `nice` `bg` `fg` `strace` `lsof`
</details>

<details>
<summary><strong>Graphics & Media (10+)</strong></summary>

`desktop` `video` `benchmark` `holo` `matrix` `imgview` `imgdemo` `gterm` `theme` `fontsmooth`
</details>

<details>
<summary><strong>Development (15+)</strong></summary>

`trustlang` `transpile` `exec` `elfinfo` `hexdump` `strings` `base64` `md5sum` `sha256sum` `od` `export` `unset` `set` `env` `source`
</details>

---

## Changelog

### v0.6.0 -- Multi-Arch & Universal Boot (February 2026)

- **Multi-Architecture Support** -- TrustOS now compiles for 3 targets from a single codebase: `x86_64-unknown-none`, `aarch64-unknown-none`, `riscv64gc-unknown-none-elf`. Architecture-specific code in `kernel/src/arch/{x86_64,aarch64,riscv64}/` with shared kernel modules via `#[cfg(target_arch)]` gates. All 3 build with 0 errors.
- **Android Boot** -- Full Android boot.img v2 pipeline. Custom linker script (`linker-android.ld`, flat physical at 0x80080000), assembly entry (`android_entry.S`: EL2->EL1 drop, stack, BSS, early UART), Rust main (`android_main.rs`: PSCI, DTB validation). SoC profiles: QEMU, Pixel, OnePlus, RPi, generic. Build with `make-android-boot.ps1`.
- **Raspberry Pi SD Card** -- Boot TrustOS bare-metal on RPi 4/5. Auto-generates `kernel8.img` + `config.txt` (64-bit mode, PL011 UART, framebuffer 1920x1080, DTB pass-through).
- **Universal Installer** -- `trustos-install.ps1`: 9 targets (PC USB, PC ISO, Android, RPi SD, ARM generic, RISC-V, QEMU x86/ARM/RISC-V). Interactive target selection, auto-build, auto-extract binary, platform-specific deployment. One script for everything.
- **Touch & Gesture Input** -- Multi-touch input layer (`touch.rs`, 400 lines) + gesture recognition engine (`gesture.rs`, 730 lines). Supports tap, swipe, pinch-to-zoom, rotate, long-press. Mobile-ready desktop.
- **GPU Compute Emulation** -- 1,770-line GPU emulation layer (`gpu_emu.rs`): virtualizes CPU cores as GPU compute units for parallel workloads.
- 190,000+ lines, 373 source files, 3 architectures.

### v0.5.0 -- CyberLab (February 2026)

- **Live network scan verified on google.com** -- nmap SYN scan (80/443 open in 2s), nmap -A (99 ports + banner grab + vuln assessment), curl (real HTTP 301 from Google), ping (4/4 0% loss 12ms RTT), DNS, traceroute, packet sniffer. Full scantest: 11/11 live tests passed.
- **Code optimization** (-2,800 lines): logo_bitmap.rs compressed via `include_bytes!`, shared `draw_utils` module eliminates 24 duplicate functions, math dedup across 5 files.
- **6 architecture fixes**: futex blocking, syscall unification, sys_poll, null ptr->EFAULT, RwLock yield, disk.rs deprecation.
- 95/96 auto-tests (99%).

### v0.4.1 -- Shell Scripting + HTTP + Security (February 2026)

- **Shell scripting engine** -- Variables, arithmetic, if/elif/else/fi, for/while loops, command substitution, special vars.
- **HTTP server** (`httpd start/stop/status`) -- Dashboard, live status, RAMFS file browser, REST API.
- **TrustPkg package manager** -- 30+ packages, 7 categories, install/remove/search.
- **TrustScan security toolkit** -- Port scanner, sniffer, banner grabber, host discovery, traceroute, vuln scanner.
- **IPv6 + ICMPv6**, CONTRIBUTING.md, GitHub Actions CI, integration tests 26-30.

### v0.4.0 -- Emulators & GameLab (February 2026)

- **Game Boy Color emulator** -- Full CGB: LR35902 CPU (501 opcodes), scanline PPU, CGB color palettes, MBC1/3/5.
- **GameLab** -- 2,000-line dashboard: memory search, watch list, tile viewer, trace, breakpoints, save/load state.
- **NES emulator** -- 6502 CPU, 2C02 PPU, mappers 0-3.
- ~9,200 new lines across 23 files.

### v0.3.x -- Foundation (February 2026)

- **v0.3.5**: ACPI tables (RSDP v2, XSDT, MADT, FADT, DSDT), PIC 8259A, PIT 8254, CMOS RTC, ACPI PM Timer.
- **v0.3.4**: PTY/TTY, pseudo-terminals, job control, `/etc/passwd`, ELF improvements, chroot, NVMe swap.
- **v0.3.3**: Cinematic trailer (128 BPM, 14 scenes), SMP AP parking.

### v0.2.0 -- Userspace (February 2026)

- Ring 3 execution via IRETQ, embedded ELF64 loader, TrustFS block freeing.

### v0.1.x -- Initial Development (February 2026)

- TrustLab (7-panel introspection lab), COSMIC2 desktop, TrustOS Film, Ed25519, 3D Chess, TrustView binary analyzer, audio synthesizer, web sandbox, TrustEdit 3D editor.
- v0.1.0: Initial release at 99K lines.

---

## Comparison

| Feature | TrustOS | SerenityOS | Redox OS | TempleOS | Linux |
|---------|---------|------------|----------|----------|-------|
| Language | **Rust** | C++ | Rust | HolyC | C |
| Lines of code | **190K** | 800K+ | 200K+ | 100K | Millions |
| Architectures | **3** (x86/ARM/RISC-V) | 2 | 1 | 1 | 30+ |
| GUI Desktop | 144 FPS + touch | Yes | Yes | 16 colors | Via X11/Wayland |
| Web Browser | From scratch | Ladybird | Ported | No | Ported |
| **Kernel Introspection Lab** | **Yes (first)** | No | No | No | External tools |
| **Built-in Emulators** | **GBC + NES + GameLab** | No | No | No | No |
| **Network Security Scanner** | **TrustScan (6 tools)** | No | No | No | Via nmap |
| **HTTP Server** | **Built-in** | No | No | No | Apache/nginx |
| **Package Manager** | **TrustPkg** | Ports | No | No | apt/dnf |
| TLS 1.3 from scratch | Yes | No | No | No | Via OpenSSL |
| Hypervisor + VMI | VT-x/SVM | No | No | No | KVM |
| Memory safe | **Yes** | No | Yes | No | No |
| Fully auditable | **Yes** | Partially | Partially | Yes | No |

---

## Contributing

Contributions are welcome! TrustOS is designed to be **readable and hackable**.

> See [CONTRIBUTING.md](CONTRIBUTING.md) for the complete developer guide.

```bash
git clone https://github.com/YOUR_USERNAME/TrustOS.git
git checkout -b feature/my-feature
cargo build --target x86_64-unknown-none -p trustos_kernel
# Test in QEMU, then open a Pull Request
```

**Good first issues**: Add a shell command, add a TrustLang builtin, create a HoloMatrix scene, improve desktop UI.

### Contributor Signatures

TrustOS uses Ed25519 cryptographic signatures to recognize contributors:

1. Boot TrustOS, run `signature sign <name>`
2. Run `signature ed25519` for your public key fingerprint
3. Include in your PR -- you get registered in the developer registry

---

## License

MIT License -- see [LICENSE](LICENSE).

---

## Author

**Nated0ge** -- Sole creator & developer of TrustOS

- GitHub: [@nathan237](https://github.com/nathan237)
- Project: [TrustOS](https://github.com/nathan237/TrustOS)

> 190,000+ lines of Rust. 3 architectures. Zero C. Fully auditable.

---

<div align="center">

**Trust** the code. **Rust** is the reason.

Created by [Nated0ge](https://github.com/nathan237)

190,000+ lines | x86_64 + ARM64 + RISC-V | Zero C | Fully auditable

[Report Bug](https://github.com/nathan237/TrustOS/issues) · [Request Feature](https://github.com/nathan237/TrustOS/issues) · [Watch Demo](https://youtu.be/RBJJi8jW1_g)

</div>
