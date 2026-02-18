<div align="center">

# 🦀 TrustOS

### **Trust** the code. **Rust** is the reason.

**A fully auditable, bare-metal operating system — 165,000+ lines of pure Rust. Zero C. Zero secrets.**

*One dev. One OS. Nothing to hide.*

**🏆 The first bare-metal OS with a built-in real-time kernel introspection laboratory.**

[![Build](https://img.shields.io/badge/build-passing-brightgreen?style=for-the-badge)]()
[![Rust](https://img.shields.io/badge/100%25%20Rust-F74C00?style=for-the-badge&logo=rust&logoColor=white)]()
[![Lines](https://img.shields.io/badge/code-165%2C000%2B%20lines-blue?style=for-the-badge)]()
[![ISO](https://img.shields.io/badge/ISO-8.95%20MB-purple?style=for-the-badge)]()
[![Version](https://img.shields.io/badge/version-0.4.1-orange?style=for-the-badge)]()
[![Auditable](https://img.shields.io/badge/fully-auditable-00C853?style=for-the-badge)]()
[![License](https://img.shields.io/badge/license-MIT-green?style=for-the-badge)]()
[![Author](https://img.shields.io/badge/created%20by-Nated0ge-ff69b4?style=for-the-badge&logo=github&logoColor=white)](https://github.com/nathan237)

[![Watch the demo](https://img.shields.io/badge/▶%20Watch%20Demo-YouTube-red?style=for-the-badge&logo=youtube&logoColor=white)](https://youtu.be/RBJJi8jW1_g)
[![Watch the Film](https://img.shields.io/badge/▶%20TrustOS%20Film-YouTube-FF0000?style=for-the-badge&logo=youtube&logoColor=white)](https://youtu.be/FILM_ID)

[Why "Trust"?](#-why-trustos) · [Features](#-features) · [Quick Start](#-quick-start) · [Architecture](#-architecture) · [Contributing](#-contributing)

---

</div>

## 🆕 Recent Modifications

| Date | Changes |
|------|----------|
| **2026-02-19** | **v0.4.1 — Shell Scripting + HTTP Server + Package Manager + Network Security** — **Shell scripting engine** (variables `$VAR`/`${VAR:-default}`, arithmetic `$((expr))`, control flow `if/elif/else/fi`, `for/while` loops, command substitution `$(cmd)`, special vars `$?`/`$$`/`$#`, `source` scripts). **HTTP server** (`httpd start/stop/status`) with dashboard, live status, RAMFS file browser, REST API (`/api/info`, `/api/stats`, `/api/processes`). **TrustPkg package manager** (`trustpkg install/remove/search/list/info`) with 30+ packages across 7 categories. **TrustScan network security toolkit** (port scanner, packet sniffer, banner grabber, host discovery, traceroute, vulnerability scanner). **CONTRIBUTING.md** developer guide. **GitHub Actions CI pipeline** (build, clippy, QEMU integration tests). IPv6 + ICMPv6 protocol support. Integration tests 26-30 (20 sub-tests). |
| **2026-02-18** | **v0.4.0 — Game Boy Color Emulator + GameLab + NES + Mario 64** — Full CGB emulator (LR35902 CPU, all 245+256 CB opcodes, scanline-accurate PPU with CGB dual VRAM banks & 8+8 color palettes, MBC1/3/5, timer, joypad, OAM/HDMA DMA). NES emulator (6502 CPU with unofficial opcodes, 2C02 PPU, mappers 0-3). TrustMario64 3D platformer (software-rendered, 23 player actions, Bob-omb Battlefield, 3 enemy types, TAS engine). **GameLab** — 2,000-line real-time analysis dashboard for the GB emulator: 5-tab UI (Analyze, Search, Watch, Tiles, Trace), memory search (Cheat Engine-style), 16-slot watch list, save/load state, breakpoints + single-step, tile/sprite viewer, speed control (0.25x–4x), trace log (last 64 instructions), memory diff highlighting. ~9,200 new lines across 23 files. 165K total, 296 source files. |
| **2026-02-17** | **v0.3.5 — ACPI + Device Emulation** — Full ACPI table generation (RSDP v2, XSDT, MADT, FADT w/ PM timer + SCI, DSDT with AML bytecode), Intel 8259A PIC emulation (ICW1-4, OCW1-3, edge-triggered IRQs, cascaded master/slave, spurious IRQ detection), Intel 8254 PIT emulation (modes 0/2/3, channel 0-2, 1.193182 MHz, lobyte/hibyte access), CMOS RTC emulation (BCD time registers 0x00-0x09, Status A/B/C, NMI masking, century register), ACPI PM Timer (3.579545 MHz, 24/32-bit). Phase 0 infrastructure: setup.sh, setup.ps1, CI release workflow, Makefile auto-Limine, repo cleanup. |
| **2026-02-16** | **v0.3.4 — POSIX Process Model + Real Disk Swap** — PTY/TTY subsystem with POSIX line discipline (canonical mode, echo, signal chars), pseudo-terminal pairs (master/slave), job control syscalls (SETPGID/SETSID/GETPGID/GETSID), `/etc/passwd` persistence (load/sync to filesystem), ELF improvements (PATH search across 5 dirs, shebang `#!` support, auxiliary vector on stack), `chroot` syscall with per-process root dir, NVMe-backed swap (last 64MB of disk, 8 sectors/page, in-memory fallback), kernel stacks 16KB→64KB. 96/96 tests. 143K lines, 262 source files. |
| **2026-02-16** | **v0.3.3 — Cinematic Trailer + Visual Overhaul** — Beat-synced 128 BPM trailer (`trailer` command) with 14 scenes, 1984/Big Brother theme, feature showcase crescendo (17 cards with decreasing delay), glow/vignette/CRT effects, XOR plasma, optimized fire. SMP: APs safely parked (cli;hlt), BSP-only mode. 131K lines, 253 source files. |
| **2026-02-14** | **v0.2.0 — Ring 3 Userspace Execution** — Real CPL-3 process execution via IRETQ with clean kernel return. Embedded ELF64 binary loader. `exec test` and `exec hello` shell commands. TrustFS block freeing fixes (unlink, truncate). 130K+ lines. |
| **2026-02-14** | **TrustLab v3 — UX Overhaul** — Full mouse/click support on all 7 panels, structured syscall tracing (50+ syscall names, args, return values in Kernel Trace), event detail panel, in-kernel UX auto-test suite (9 tests via `labtest` command). 136K+ lines. |
| **2026-02-13** | **TrustLab v2 — Demo Mode** — Cinematic 27-second narrated demo with Matrix-themed Morpheus intro, 23 slides with panel-targeted narration, glitch transitions, red text overlay, progress bar, space-to-skip. PIT-based timing (100Hz). |
| **2026-02-13** | **TrustLab — 7-Panel Introspection Laboratory** — Added Hex Editor panel (7th panel), execution pipeline visualizer, zero-cost trace bus with 512-slot ring buffer. The **first bare-metal OS with real-time kernel introspection**. |
| **2026-02-12** | **TrustOS Film** — Built-in cinematic animated explainer with 12 unique scene-specific animations, 8 animated backgrounds. Run `film` in the shell. |
| **2026-02-12** | **Ed25519 Signatures** — Full RFC 8032 Ed25519 asymmetric signature system. **Cross-platform build** — Makefile + build.sh for Linux/macOS. **TrustLang Showcase** — Proper syntax highlighting + auto-scrolling editor. |
| **2026-02-12** | **3D Chess** — Full 3D chess game with low-poly pieces, proper look-at camera, AI opponent (minimax depth 2), board labels, shadows, reflections, scroll zoom. |
| **2026-02-11** | Terminal: timestamp cyan, `neofetch`, colorized outputs, categorized help, scrollbar, color marker system, Shift stuck fix |

---

## �🔐 Why "TrustOS"?

The name says it all: **Trust** + **Rust**.

In a world where your operating system is a black box — millions of lines of legacy C/C++, binary blobs, proprietary drivers, telemetry you can't disable — **how do you know what your computer is actually doing?**

TrustOS is the answer: **every single line is open, readable, and auditable.**

- 🔍 **Fully auditable** — 165,000 lines of Rust, all on GitHub. No binary blobs. No hidden code.
- 🦀 **Memory safe by design** — Rust's ownership model prevents entire categories of vulnerabilities (buffer overflows, use-after-free, data races).
- 🧩 **Zero dependencies on C** — no libc, no glibc, no C runtime. Every driver, every protocol, every pixel is Rust.
- 📖 **Readable** — one person wrote it in 8 days. If one person can build it, one person can understand it.

> *"The only OS where you can trace every packet, every pixel, and every keystroke back to its source code."*

### Key Stats

| Metric | Value |
|--------|-------|
| **Total code** | 165,000+ lines of Rust |
| **Source files** | 304+ `.rs` files |
| **Kernel modules** | 44 independent modules |
| **ISO size** | 8.95 MB |
| **Boot time** | < 1 second |
| **Desktop FPS** | 144 FPS (SSE2 SIMD) |
| **C code** | 0 lines |
| **External dependencies** | 0 (everything from scratch) |
| **Development time** | 12 days |

### TrustOS vs The World

| | Traditional OS | TrustOS |
|---|:---:|:---:|
| **Language** | C/C++ with 40 years of memory bugs | 100% Rust — memory safe by design |
| Codebase | Millions of lines, impossible to audit | 165K lines, one person can read it all |
| **Binary blobs** | Everywhere | None. Zero. |
| **Telemetry** | Opt-out (maybe) | Doesn't exist — verify it yourself |
| **Build** | Complex cross-compilation toolchains | `cargo build` — that's it |

### 🔭 Vision

TrustOS is not another Linux clone — it's a living laboratory. The goal was never to replace your daily driver, but to build the first bare-metal environment where you can observe, understand, and experiment with every layer of an operating system in real time. Today, TrustLab already lets you trace kernel events, inspect memory, and monitor virtual machines from the hypervisor level. Tomorrow, TrustOS will become a full **open-source Cyber Range**: a training and research platform for cybersecurity where pentesters, researchers, and students can boot a guest OS, observe its processes agentlessly, trace exploits live, and analyze malware in a fully transparent environment. Every line is auditable Rust, every architectural decision is documented, and the AI that accelerates development isn't a secret — it's an advantage. The code is open. The vision is simple: **if you can't observe your system, you can't trust it.**

---

## ✨ Features

### � TrustLab — Real-Time Kernel Introspection Laboratory

> **🏆 World's first: no other bare-metal OS has a built-in, real-time kernel introspection lab.**

TrustLab is a 7-panel interactive workspace that lets you **watch the OS kernel run in real-time — from inside itself**. No external debugger. No attached tools. The OS observes its own internals live.

| Panel | What it shows |
|-------|---------------|
| **Hardware Status** | Live CPU gauge, heap usage bar, IRQ rate, uptime, allocation stats — updated every tick |
| **Kernel Trace** | Scrolling event log with **structured syscall tracing** (50+ syscall names, args, return values), category filters, event detail panel, selected-row inspection |
| **Command Guide** | Searchable reference of ~55 commands with fuzzy search and category tabs |
| **File System Tree** | Interactive VFS browser with expand/collapse, file sizes, color-coded extensions (.rs = green, .tl = purple) |
| **TrustLang Editor** | Syntax-highlighted code editor with F5 execution and output pane |
| **Execution Pipeline** | Real-time data flow visualization through the kernel |
| **Hex Editor** | Raw byte inspection with color-coded display |

- **Full mouse interaction** — Click on any of the 7 panels to focus, interact with filters, select events, scroll stats, toggle pipeline stages. Every panel responds to clicks.
- **Structured syscall tracing** — Every syscall emits structured data: syscall number, human-readable name (50+ mapped), arguments, and return value. Click any event in Kernel Trace to inspect details.
- **Automated UX testing** — Type `labtest` in the TrustLab shell bar to run 9 automated usability tests (tab cycling, click dispatch, shell commands, trace propagation, syscall data, filters, hardware state, pipeline updates, guide search). Results appear in Kernel Trace + serial output.
- **Zero-cost trace bus** — 512-slot ring buffer, gated by `LAB_ACTIVE` flag. Kernel hooks in interrupts, VFS, scheduler, syscall handler, and memory allocator emit events **only when TrustLab is open** — zero overhead otherwise.
- **Cinematic demo mode** — Type `demo` inside TrustLab for a 27-second Matrix-themed narrated tour with Morpheus intro, glitch transitions, and panel-targeted narration.
- **Launch**: `lab` or `trustlab` in shell, or from desktop Start Menu.

---

### �🖥️ COSMIC2 Desktop Environment
- **Multi-layer GPU compositor** with 8 independent rendering layers
- **SSE2 SIMD optimized** — 144 FPS with zero flickering
- **Taskbar, dock, start menu**, window management, settings panel
- **14 built-in desktop apps**: Terminal, Files, TrustCode, Calculator, Network, Snake Game, Chess 3D, Settings, About, TrustGL 3D, TrustBrowser, TrustEdit 3D, and more
- **HoloMatrix 3D backgrounds**: volumetric wireframe scenes (cube, torus, DNA helix, character...)
- **Mouse + keyboard** driven with smooth cursor

### 🌐 TrustBrowser — Built-in Web Browser
- **HTML parser** — renders real web pages
- **CSS engine** — styling and layout
- **JavaScript engine** — basic script execution
- **HTTPS support** — TLS 1.3, handshake written from scratch
- Navigate to real websites from inside the OS

### 📝 TrustCode — Code Editor
- **Rust syntax highlighting** with 60+ keywords
- **Line numbers**, cursor navigation, scrolling
- **File save/load** from TrustFS
- **Bracket matching** and auto-indentation

### 🔤 TrustLang — Programming Language & Compiler
- **Rust-inspired syntax** with functions, recursion, loops, types
- **Full compiler pipeline**: Lexer → Parser → Compiler → Bytecode VM
- **Zero dependencies** — compiles and runs entirely in-kernel
- Commands: `trustlang run`, `trustlang eval`, `trustlang check`

### 🎬 TrustVideo — Real-time Video Engine
- **Custom `.tv` format** with delta + RLE compression
- **Procedural demo engine**: fire, matrix rain, plasma effects
- **60-72 FPS** rendering with integer sine LUT (no floats)
- **RAM backbuffer** + SSE2 swap for instant display

### 🔷 Formula3D — Wireframe 3D Engine
- **Real-time wireframe rendering** with perspective projection
- **Multiple mesh types**: cube, torus, sphere, DNA helix, character model
- **Per-edge coloring** with volumetric low-poly rendering
- **Scanline effects**, gradient backgrounds, floor grids

### 🎨 TrustEdit — 3D Model Editor
- **Wireframe 3D editor** with real-time orbital camera
- **5 editing tools**: Select, Move, Add Vertex, Add Edge, Delete
- **Preset models**: Cube, Pyramid, Diamond, Star, House, Rocket, Tree, Spaceship
- **Save/Load** models in `.t3d` format to TrustFS
- **Undo system** for safe editing
- **Mouse-driven** with toolbar, viewport, and status bar
- Launch with `trustedit` command or from the desktop Start Menu

### ♟️ Chess 3D — Full 3D Chess Game
- **Complete chess engine** — All rules: castling, en passant, promotion, check/checkmate
- **3D low-poly rendering** — Board, pieces (King, Queen, Rook, Bishop, Knight, Pawn) with mesh geometry
- **Look-at camera system** — Spherical coordinates, forward/right/up basis vectors, perspective projection
- **AI opponent** — Minimax algorithm (depth 2) with piece-value + positional evaluation
- **Board labels** — Algebraic notation (a-h, 1-8) along edges
- **Shadows & reflections** — Piece shadows on board, reflective board surface
- **Interactive controls** — Mouse piece selection, scroll zoom, camera orbit
- **Matrix theme** — Green wireframe aesthetic matching TrustOS visual identity
- Launch with `chess3d` command or from the desktop

### 🎮 Game Boy Color Emulator + GameLab

> **A complete CGB emulator running bare-metal — with a built-in real-time analysis dashboard.**

**Game Boy Color Emulator:**
- **Sharp LR35902 CPU** — All 245 base opcodes + 256 CB-prefix bit/shift/rotate ops, interrupt handling (5 vectors), HALT, DI/EI, DAA
- **Scanline-accurate PPU** — 160×144, 4-mode dot-cycle timing (OAM Search / Pixel Transfer / HBlank / VBlank), background + window + sprite rendering
- **Full CGB support** — Dual VRAM banks, per-tile attributes, 8 BG + 8 OBJ color palettes (RGB555), BCPS/BCPD/OCPS/OCPD auto-increment
- **4 MBC mappers** — MBC0, MBC1 (banking modes), MBC3 (ROM/RAM banks), MBC5 (9-bit ROM)
- **Timer, joypad, OAM DMA, HDMA, WRAM banking (32KB CGB), HRAM**
- ROM embedding at compile time via `include_bytes!()` — drop a `.gb` file in `kernel/roms/` and rebuild

**GameLab — Real-Time Analysis Dashboard (2,000 lines):**

| Tab | Feature | Description |
|-----|---------|-------------|
| **ANALYZE** | 6-panel live view | CPU registers/flags, GPU state (mode/LY/LCDC), memory hex dump, I/O registers, cartridge info, input display |
| **SEARCH** | Memory search | Cheat Engine-style: exact value, changed, unchanged, greater, less — snapshot-based narrowing, up to 256 results |
| **WATCH** | Watch list | Pin up to 16 addresses with labels, previous/current value tracking, change highlighting |
| **TILES** | Tile/Sprite viewer | 3 pages: tiles at $8000, tiles at $8800, OAM sprites — rendered from VRAM |
| **TRACE** | Trace log | Last 64 executed instructions with PC, opcode, A, F, SP |

**Additional toolbar features:**
- **Speed control** — 0.25×, 0.5×, 1×, 2×, 4× emulation speed
- **Breakpoints** — Up to 8 PC breakpoints, single-step, frame advance, pause/resume
- **Save/Load state** — Full snapshot/restore (CPU, GPU, VRAM×2, OAM, palettes, timer, cart RAM, WRAM, HRAM)
- **Memory diff** — Highlights changed bytes in the hex dump view

### 🕹️ NES Emulator
- **MOS 6502 CPU** — All 151 official opcodes + 8 common unofficial opcodes (LAX, SAX, DCP, ISB, SLO, RLA, SRE, RRA), correct cycle counts
- **2C02 PPU** — 256×240, scanline-accurate background + sprite rendering, scroll registers, sprite 0 hit, 64-color system palette
- **4 mappers** — NROM (0), MMC1 (1), UxROM (2), CNROM (3) — covers most common NES games
- **Controller input**, OAM DMA, nametable mirroring (horizontal/vertical/single/four-screen)
- Drop a `.nes` file in `kernel/roms/` to embed at compile time

### 🏔️ TrustMario64 — 3D Platformer
- **Software-rendered 3D** — No GPU, pure CPU rendering with perspective projection
- **23 player actions** — walk, run, jump, double/triple jump, long jump, backflip, side flip, wall kick, ground pound, dive, swim, ledge grab, and more
- **Bob-omb Battlefield** — 32×32 heightmap terrain with mountain, water, bridge, trees, coins, stars
- **3 enemy types** — Goomba (patrol/chase), Bob-omb (fuse/explode), Chain Chomp (tethered lunge)
- **TAS engine** — Save/load state (F1/F2), frame advance (F3), record/replay inputs (F5/F6), rewind (F7), ghost playback (F8), hitbox visualization (F10)
- **Lakitu camera** — Mouse orbit + scroll zoom + E/Q rotation

### 🎮 Interactive Desktop Apps
- **Calculator** — Full arithmetic with chained operations, keyboard & mouse input
- **Snake Game** — Real-time gameplay with arrow keys, scoring, progressive speed
- **TrustBrowser** — Keyboard-driven URL bar, page navigation

### 📁 TrustFS — Persistent Filesystem
- **Block-based storage** with indirect block support
- **Write-Ahead Logging (WAL)** for crash safety
- **Block cache** for performance
- **VFS layer** unifying ramfs, procfs, devfs, FAT32, and TrustFS

### 🌐 Network Stack (from scratch)
- **VirtIO-net** driver with full packet handling
- **TCP/IP** stack: ARP, DHCP, DNS, TCP, UDP, ICMP
- **IPv6 + ICMPv6** — Next-generation internet protocol support
- **TLS 1.3** — full handshake, X.509 certificate validation, crypto
- **HTTP/HTTPS client** — `curl`, `wget`, `browse`
- **HTTP server** — Built-in web server (`httpd start/stop/status`)
- **Commands**: `ping`, `nslookup`, `traceroute`, `netstat`, `arp`, `route`, `ifconfig`

### 🔒 TrustScan — Network Security Toolkit
- **Port scanner** — TCP SYN/connect scanning with service detection
- **Packet sniffer** — Real-time packet capture and analysis
- **Banner grabber** — Service identification via banner probes
- **Host discovery** — ARP-based and ICMP-based network scanning
- **Traceroute** — ICMP/UDP hop-by-hop path analysis
- **Vulnerability scanner** — Common vulnerability checks against discovered services
- Commands: `netscan scan`, `netscan sniff`, `netscan banner`, `netscan discover`, `netscan trace`, `netscan vuln`

### 🌐 HTTP Server (httpd)
- **Built-in web server** running on the TrustOS TCP stack
- **Dashboard** — System overview page at `/`
- **Live status** — Real-time OS stats at `/status`
- **File browser** — Browse RAMFS files at `/files/`
- **REST API** — `/api/info`, `/api/stats`, `/api/processes` (JSON)
- Commands: `httpd start [port]`, `httpd stop`, `httpd status`

### 📦 TrustPkg — Package Manager
- **30+ packages** across 7 categories (system, network, dev, security, graphics, games, utils)
- **Install/Remove** — `trustpkg install <pkg>`, `trustpkg remove <pkg>`
- **Search/Browse** — `trustpkg search <query>`, `trustpkg list [category]`
- **Package info** — `trustpkg info <pkg>` with description, version, size, dependencies
- **Update** — `trustpkg update` to refresh all installed packages

### 📜 Shell Scripting Engine
- **Variables** — `export VAR=value`, `$VAR`, `${VAR}`, `${VAR:-default}`
- **Arithmetic** — `$((expression))` with `+`, `-`, `*`, `/`
- **Control flow** — `if/elif/else/fi` with test conditions (`-f`, `-d`, `-z`, `-n`, `=`, `!=`)
- **Loops** — `for var in list; do ... done`, `while condition; do ... done`
- **Command substitution** — `$(command)` for inline execution
- **Special variables** — `$?` (last exit code), `$$` (PID), `$#` (arg count)
- **Script execution** — `source script.sh` to run shell scripts

### 🐧 Linux Compatibility Layer (WIP)

TrustOS is being built **with Linux binary compatibility in mind**. The infrastructure is real, but it's not at the "run `apt-get`" stage yet.

| Component | Status | What exists |
|-----------|--------|-------------|
| **Syscall interface** | ✅ Functional | 70+ Linux syscalls dispatched (read, write, mmap, fork, socket, exec, uname, setpgid, setsid, chroot...) |
| **ELF64 loader** | ✅ Functional | Parses and loads static ELF binaries, PATH search (5 dirs), shebang `#!` support, auxiliary vector (AT_PAGESZ, AT_PHDR, AT_ENTRY...) |
| **Ring 3 (userland)** | ✅ Working | Real SYSCALL/SYSRET mechanism, IRETQ to Ring 3, kernel stack switching |
| **Process table** | ✅ Working | Full PCB, PID management, fork/exit/wait, state machine, process groups, sessions, controlling TTY |
| **PTY/TTY subsystem** | ✅ Working | POSIX line discipline, pseudo-terminal master/slave pairs, termios, ioctls (TIOCGPGRP, TCGETS, TIOCGWINSZ...) |
| **Job control** | ✅ Working | SETPGID, GETPGRP, SETSID, GETPGID, GETSID syscalls, kill_process_group() |
| **chroot** | ✅ Working | Per-process root directory, sys_chroot syscall |
| **Wayland compositor** | 🔨 Internal | 2,700-line in-kernel compositor with surface management, window decorations, VT100 terminal. Protocol opcodes defined but no IPC socket transport — external Wayland clients can't connect yet |
| **x86_64 interpreter** | 🧪 Proof of concept | ~15 opcodes implemented (mov, push, jmp, call, syscall). Cannot run real binaries |
| **Alpine subsystem** | 📋 Planned | Infrastructure exists, needs full syscall coverage |

> **TL;DR:** The architecture is designed for Linux compat (ELF + Linux ABI + Wayland-style compositor), and the pieces are being connected. The Wayland mention in commits refers to a real in-kernel compositor module — not yet a full Wayland display server that external clients can talk to.

### 🛡️ Security & Auth
- **Ed25519 asymmetric signatures** — RFC 8032 digital signatures for unforgeable kernel authentication
- **Capability-based** security model
- **User authentication**: login, su, passwd, adduser
- **File permissions**: chmod, chown
- **Process isolation** with Ring 0/3 separation

### ⚡ Hypervisor & Device Emulation
- **Intel VT-x (VMX)** and **AMD-V (SVM)** dual-backend
- **Extended Page Tables (EPT/NPT)**, VMCS, VMCB, VPID
- **VMI Engine** — Virtual Machine Introspection for agentless guest monitoring
- **ACPI table generation** — RSDP v2, XSDT, MADT, FADT, DSDT with AML bytecode
- **PIC 8259A** emulation — cascaded master/slave, edge-triggered IRQs, ICW/OCW
- **PIT 8254** emulation — 1.193 MHz timer, modes 0/2/3, channel 0-2
- **CMOS RTC** — BCD time registers, NMI masking, century register
- **ACPI PM Timer** — 3.579 MHz, 24/32-bit counter
- **Guest VM isolation** for running Linux subsystem

### ⚡ Performance
- **SSE2 SIMD** throughout: buffer fills, blits, compositing
- **Double-buffered** rendering (RAM backbuffer → MMIO swap)
- **SMP multi-core** support
- **Compile-time LUTs** for math-heavy rendering

---

## 🚀 Quick Start

### Option A: Download the ISO (fastest)

Grab the latest prebuilt ISO from [**Releases**](https://github.com/nathan237/TrustOS/releases) and boot it in any VM:

```bash
# QEMU (BIOS — no extra firmware needed)
qemu-system-x86_64 -cdrom trustos.iso -m 512M -cpu max -smp 4 -display gtk -vga std -serial stdio

# QEMU (UEFI)
qemu-system-x86_64 -cdrom trustos.iso -m 512M -machine q35 -cpu max -smp 4 \
  -drive if=pflash,format=raw,readonly=on,file=/usr/share/OVMF/OVMF_CODE.fd \
  -display gtk -vga std -serial stdio
```

> Also works in VirtualBox, VMware, or on bare metal USB via `dd if=trustos.iso of=/dev/sdX`.

### Option B: One-liner setup (build from source)

**Linux / macOS / WSL:**
```bash
git clone https://github.com/nathan237/TrustOS.git && cd TrustOS && ./setup.sh
```
This automatically installs Rust nightly, QEMU, xorriso, OVMF, downloads Limine, builds the kernel, and creates the ISO.

**Windows (PowerShell):**
```powershell
git clone https://github.com/nathan237/TrustOS.git; cd TrustOS; .\setup.ps1
```

### Option C: Manual build

<details>
<summary>Step-by-step instructions</summary>

#### Prerequisites
- Rust nightly (`rustup` will auto-install via `rust-toolchain.toml`)
- QEMU with OVMF (UEFI firmware)
- `xorriso` (for ISO creation, Linux only)

#### Install dependencies
```bash
# Debian / Ubuntu
sudo apt install qemu-system-x86 xorriso ovmf

# Fedora
sudo dnf install qemu xorriso edk2-ovmf

# Arch
sudo pacman -S qemu-full xorriso edk2-ovmf

# macOS
brew install qemu xorriso
```

#### Build & Run
```bash
make run            # Build + run in QEMU (UEFI)
make run-bios       # Build + run in QEMU (BIOS)
make iso            # Build kernel + create ISO only
make check-deps     # Verify all tools are installed
```

#### Windows (PowerShell)
```powershell
cargo build --release -p trustos_kernel
.\run-qemu-gui.ps1     # QEMU with GUI
.\run-vbox.ps1          # VirtualBox (full setup)
```

</details>

### First commands to try
| Command | What it does |
|---------|-------------|
| `showcase` | Automated feature tour (great for screen recording) |
| `desktop` | Launch the COSMIC2 desktop environment |
| `trustlab` | Open the real-time kernel introspection laboratory |
| `neofetch` | System info display |
| `chess3d` | Play 3D chess against AI |
| `gameboy` | Launch the Game Boy Color emulator |
| `nes` | Launch the NES emulator |
| `mario64` | Launch the 3D Mario platformer |
| `httpd start` | Start the built-in HTTP web server |
| `trustpkg list` | Browse 30+ installable packages |
| `netscan scan` | Scan network ports |
| `help` | Show all 200+ commands |

---

## 📸 Screenshots

### COSMIC2 Desktop with Interactive Apps
![TrustOS Desktop](docs/screenshot_desktop.png)

### Shell with 200+ Commands
```
  _____ ____            _    ___      
 |_   _|  _ \ _   _ ___| |_ / _ \ ___ 
   | | | |_) | | | / __| __| | | / __|
   | | |  _ <| |_| \__ \ |_| |_| \__ \
   |_| |_| \_\\__,_|___/\__|\___/|___/

[14:32:15] trustos:/$ showcase
```

> **Tip**: Run `showcase` to see an automated demo of all features — perfect for screen recording!

### Available Demo Commands
| Command | Description |
|---------|-------------|
| `showcase` | Full automated feature tour (for video) |
| `showcase fast` | Quick version |
| `showcase slow` | Extended version with longer pauses |
| `desktop` | Launch COSMIC2 desktop environment |
| `video demo fire` | Real-time fire effect |
| `video demo matrix` | Matrix digital rain |
| `video demo plasma` | Psychedelic plasma |
| `trustlang demo` | TrustLang compiler demo |
| `benchmark` | GPU/SIMD performance benchmarks |
| `neofetch` | System info display |
| `trustedit` | Launch TrustEdit 3D model editor |
| `chess3d` | Launch 3D Chess game vs AI |

---

## 📋 200+ Built-in Commands

<details>
<summary><strong>📁 Filesystem (25+)</strong></summary>

`ls` `cd` `pwd` `mkdir` `rmdir` `touch` `rm` `cp` `mv` `cat` `head` `tail` `wc` `stat` `tree` `find` `grep` `ln` `readlink` `chmod` `chown` `sort` `uniq` `cut` `diff`
</details>

<details>
<summary><strong>🌐 Network (20+)</strong></summary>

`ifconfig` `ping` `curl` `wget` `nslookup` `arp` `route` `netstat` `traceroute` `browse` `download` `httpget` `tcpsyn` `ip` `dig` `httpd` `netscan`
</details>

<details>
<summary><strong>🔒 Security & Scanning (10+)</strong></summary>

`netscan scan` `netscan sniff` `netscan banner` `netscan discover` `netscan trace` `netscan vuln` `login` `su` `passwd` `adduser`
</details>

<details>
<summary><strong>📦 Package Management</strong></summary>

`trustpkg list` `trustpkg search` `trustpkg install` `trustpkg remove` `trustpkg info` `trustpkg installed` `trustpkg update`
</details>

<details>
<summary><strong>⚙️ System (25+)</strong></summary>

`ps` `top` `free` `df` `uname` `dmesg` `mount` `umount` `lspci` `lscpu` `lsblk` `lsmem` `lsusb` `lsmod` `lshw` `vmstat` `iostat` `sysctl` `kill` `killall` `nice` `bg` `fg` `strace` `lsof`
</details>

<details>
<summary><strong>👤 Users & Security (10+)</strong></summary>

`login` `logout` `su` `passwd` `adduser` `deluser` `users` `whoami` `id` `chmod` `chown`
</details>

<details>
<summary><strong>🎨 Graphics & Media (10+)</strong></summary>

`desktop` `video` `benchmark` `holo` `matrix` `imgview` `imgdemo` `gterm` `theme` `fontsmooth`
</details>

<details>
<summary><strong>🛠️ Development (15+)</strong></summary>

`trustlang` `transpile` `exec` `elfinfo` `hexdump` `strings` `base64` `md5sum` `sha256sum` `od` `export` `unset` `set` `env` `source`
</details>

<details>
<summary><strong>📦 Archives & Compression</strong></summary>

`tar` `gzip` `gunzip` `zip` `unzip`
</details>

---

## � TrustOS Film

TrustOS includes a **built-in cinematic animated explainer** — a 2-minute film that runs entirely inside the OS, rendered in real-time on the framebuffer.

**Run it:** type `film` in the TrustOS shell.

| ACT | Theme | Animation |
|-----|-------|-----------|
| **I — The Question** | "You use a computer every day..." | Floating windows, question marks rain, screen shatter |
| **II — The Problem** | "It controls EVERYTHING" | Binary flood, redacted classified document, earthquake bar chart |
| **III — The Solution** | "What if one person could understand ALL of it?" | Light burst with star rays, odometer counter 0→120,000 |
| **IV — The Proof** | Real packet journey through the stack | Animated packet with trail, circuit-board background |
| **V — The Future** | "TrustOS proves it." | Sparkle dissolve, expanding shockwave rings, matrix rain callback |

> 12 unique scene-specific animations, 8 animated backgrounds, all integer math (no_std compatible).

[![Watch on YouTube](https://img.shields.io/badge/▶%20Watch%20the%20Film-YouTube-FF0000?style=for-the-badge&logo=youtube&logoColor=white)](https://youtu.be/FILM_ID)

---

## �🏗️ Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                     Applications                            │
│  TrustCode · TrustLang · TrustBrowser · Games · Terminal    │
│  Game Boy Color · NES · Mario64 · GameLab                   │
├─────────────────────────────────────────────────────────────┤
│              COSMIC2 Desktop Compositor                     │
│     8-layer GPU compositing · SSE2 SIMD · 144 FPS          │
├──────────┬──────────┬───────────┬──────────┬────────────────┤
│  VFS     │ Network  │  Linux    │ Graphics │  TrustVideo    │
│  ramfs   │ TCP/IP   │ Subsystem │ TrustGL  │  Codec +       │
│  procfs  │ TLS 1.3  │ 100+     │ Raytrace │  Procedural    │
│  devfs   │ DHCP/DNS │ syscalls  │ 3D Mesh  │  Renderer      │
│  TrustFS │ VirtIO   │ ELF      │ Formula  │  Fire/Plasma   │
│  FAT32   │ HTTP/S   │ Alpine   │ HoloMat  │  Matrix Rain   │
│          │ httpd    │          │          │                │
├──────────┼──────────┼───────────┼──────────┼────────────────┤
│ TrustPkg │ NetScan  │ Scripting │ IPv6     │  CI Pipeline   │
│ 30+ pkgs │ Scanner  │ Engine    │ ICMPv6   │  GitHub        │
│ 7 categ. │ Sniffer  │ Variables │          │  Actions       │
│          │ Vuln     │ Loops     │          │                │
├──────────┴──────────┴───────────┴──────────┴────────────────┤
│                    TrustOS Kernel                           │
│  Memory · Scheduler · IPC · Security · Drivers · Syscalls  │
│  SSE2 SIMD · SMP Multi-core · Hypervisor (VT-x/SVM)       │
├─────────────────────────────────────────────────────────────┤
│              Hardware (x86_64 · UEFI · VirtIO)              │
│              Limine Bootloader · MMIO · PCI                 │
└─────────────────────────────────────────────────────────────┘
```

### Module Breakdown

| Module | Lines | Description |
|--------|-------|-------------|
| `shell.rs` | ~17,000 | Command interpreter, 200+ commands, cinematic showcase |
| `network/` | ~5,000 | Full TCP/IP stack with TLS 1.3, HTTPS |
| `graphics/` | ~4,000 | HoloMatrix, raytracer, 3D mesh, OpenGL emu |
| `compositor/` | ~3,000 | Multi-layer GPU compositor |
| `browser/` | ~2,500 | HTML/CSS parser, JS engine, page renderer |
| `linux/` | ~3,000 | Linux syscall emulation, ELF loader |
| `trustlang/` | ~2,000 | Compiler + bytecode VM |
| `formula3d.rs` | ~1,500 | Wireframe 3D engine with per-edge colors |
| `chess.rs` | ~1,030 | Full chess engine: rules, AI minimax, game state |
| `chess3d.rs` | ~1,093 | 3D chess renderer: look-at camera, low-poly meshes |
| `gameboy/` | ~1,870 | Game Boy Color emulator: CPU, PPU, MBC1/3/5, timer, CGB color |
| `game_lab.rs` | ~2,025 | GameLab analysis dashboard: search, watch, tiles, trace, breakpoints |
| `nes/` | ~1,466 | NES emulator: 6502 CPU, 2C02 PPU, mappers 0-3 |
| `mario64/` | ~3,840 | 3D platformer: player, physics, enemies, renderer, TAS engine |
| `model_editor.rs` | ~750 | TrustEdit 3D wireframe model editor |
| `video/` | ~1,500 | TrustVideo codec & player |
| `framebuffer/` | ~1,500 | SSE2 SIMD rendering |
| `filesystem/` | ~2,000 | TrustFS with WAL, VFS, FAT32 |
| `hypervisor/` | ~10,000 | VT-x/SVM dual-backend, EPT/NPT, VMI engine, ACPI tables, PIC/PIT/RTC/PM timer emulation |
| `tls13/` | ~2,000 | TLS 1.3, crypto, X.509 certs |
| `ed25519.rs` | ~720 | Ed25519 asymmetric signatures (RFC 8032) |
| `tty.rs` | ~330 | POSIX TTY layer with line discipline |
| `pty.rs` | ~196 | Pseudo-terminal master/slave pairs |
| `swap.rs` | ~466 | NVMe-backed page swap with fallback |
| `scripting.rs` | ~643 | Shell scripting engine (variables, loops, control flow) |
| `httpd.rs` | ~414 | Built-in HTTP web server with REST API |
| `trustpkg.rs` | ~290 | Package manager with 30+ packages |
| `netscan/` | ~900 | Network security toolkit (6 modules) |
| `ipv6.rs` | ~200 | IPv6 protocol implementation |
| `icmpv6.rs` | ~150 | ICMPv6 protocol support |

---

## 🎬 Create a Demo Video

TrustOS includes a built-in `showcase` command that automates a complete feature tour:

```bash
# Boot TrustOS, then type:
showcase          # Normal speed (~90 seconds)
showcase fast     # Quick demo (~45 seconds)  
showcase slow     # Extended demo (~3 minutes)
```

The showcase runs through:
1. **Cinematic intro** — 5 Matrix-style scenes with typed text & rain effects
2. **System info** — neofetch, uname, memory stats
3. **Filesystem** — create files, directory tree
4. **TrustLang** — compile & run a Fibonacci program live
5. **Network** — interface config, connection status
6. **Video effects** — fire, matrix rain, plasma (full-screen, auto-timed)
7. **Formula3D** — real-time wireframe 3D character with per-edge colors
8. **COSMIC2 Desktop + TrustBrowser** — desktop auto-demo with web browsing
9. **Command overview** — 200+ commands categorized

> Film it with OBS or any screen recorder for instant marketing content!

---

## 🤝 Contributing

Contributions are welcome! TrustOS is designed to be **readable and hackable**.

> 📖 **See [CONTRIBUTING.md](CONTRIBUTING.md) for the complete developer guide** — build requirements, architecture overview, how to add commands/drivers/syscalls, integration tests, code style, and PR process.

### Good First Issues
- Add a new shell command (follow the pattern in `shell.rs`)
- Add a new TrustLang built-in function
- Create a new HoloMatrix 3D scene
- Add a new TrustVideo procedural effect
- Improve the desktop UI (new widgets, themes)

### How to Contribute

```bash
# Fork, then:
git clone https://github.com/YOUR_USERNAME/TrustOS.git
git checkout -b feature/my-feature

# Make changes, build & test
cargo build --release -p trustos_kernel
# Run in QEMU to test

git commit -m "Add my feature"
git push origin feature/my-feature
# Open a Pull Request
```

### 🔐 Contributor Signatures & Developer Registry

TrustOS uses a **cryptographic signature system** to recognize contributors. When your PR is merged, you become a registered developer:

1. **Sign your build** — Boot TrustOS, run `signature sign <your_name>` with a secret passphrase
2. **Export your signature** — Run `signature export` to get your HMAC-SHA256 fingerprint
3. **Verify with Ed25519** — Run `signature ed25519` to see the asymmetric Ed25519 signature
4. **Include it in your PR** — Add your entry to [`SIGNATURES.md`](SIGNATURES.md)
4. **Get registered** — Once merged, you appear in the official developer registry

**What this means for contributors:**

| Benefit | Description |
|---------|-------------|
| **Developer entry** | Your name, GitHub, and fingerprint in the public registry |
| **Module attribution** | The modules you contributed are listed under your signature |
| **Cryptographic proof** | Your HMAC-SHA256 fingerprint proves you endorsed this build |
| **Immutable record** | Your entry is version-controlled in git — permanent history |

The **creator signature (#001)** is hardcoded in every kernel binary and can never be modified. Contributor signatures are co-signatures — they prove that a developer participated in TrustOS development and endorsed the code they contributed.

> Every contributor with an approved signature and a merged PR is integrated into the official developer list.  
> Your modules carry your signature. Your work is cryptographically attributed to you.

### Project Structure
```
kernel/src/
├── main.rs              # Kernel entry point
├── shell.rs             # Shell + 200+ commands + showcase
├── shell/scripting.rs   # Shell scripting engine
├── httpd.rs             # Built-in HTTP web server
├── trustpkg.rs          # Package manager (30+ packages)
├── netscan/             # Network security toolkit (6 modules)
├── desktop.rs           # COSMIC2 desktop manager
├── chess.rs             # Chess engine (rules, AI, game state)
├── chess3d.rs           # 3D Chess renderer (camera, meshes)
├── gameboy/             # Game Boy Color emulator (CPU, PPU, MBC, CGB)
├── game_lab.rs          # GameLab real-time analysis dashboard
├── nes/                 # NES emulator (6502, 2C02 PPU, mappers)
├── mario64/             # 3D platformer (player, physics, TAS, renderer)
├── embedded_roms.rs     # Compile-time ROM embedding
├── model_editor.rs      # TrustEdit 3D model editor
├── tty.rs               # POSIX TTY layer + line discipline
├── pty.rs               # Pseudo-terminal master/slave pairs
├── signature.rs         # Kernel signatures (HMAC-SHA256 + Ed25519)
├── ed25519.rs           # Ed25519 digital signatures (RFC 8032)
├── formula3d.rs         # Wireframe 3D engine
├── compositor/          # 8-layer GPU compositor
├── browser/             # HTML/CSS/JS browser engine
├── network/             # TCP/IP, DHCP, DNS, IPv6, ICMPv6
├── tls13/               # TLS 1.3, crypto, X.509
├── video/               # TrustVideo codec
├── trustlang/           # Compiler + VM
├── framebuffer/         # SSE2 SIMD rendering
├── graphics/            # 3D, raytracer, HoloMatrix
├── hypervisor/          # VT-x/SVM, EPT/NPT, VMI, ACPI, PIC/PIT/RTC emulation
├── vfs/                 # TrustFS, FAT32, procfs, devfs
├── linux_compat/        # 100+ Linux syscalls
├── drivers/             # AHCI, USB, VirtIO, input
└── security/            # Capability model, auth
```

---

## 📊 Comparison

| Feature | TrustOS | SerenityOS | Redox OS | TempleOS | Linux |
|---------|---------|------------|----------|----------|-------|
| Language | **Rust** | C++ | Rust | HolyC | C |
| Lines of code | **165K** | 800K+ | 200K+ | 100K | Millions |
| Contributors | **1** | 1,141 | Community | 1 | Thousands |
| Development time | **10 days** | 6+ years | 10+ years | ~10 years | 35+ years |
| GUI Desktop | ✅ (144 FPS) | ✅ | ✅ | ✅ (16 colors) | Via X11/Wayland |
| Web Browser | ✅ (from scratch) | ✅ (Ladybird) | Ported (NetSurf) | ❌ | Ported |
| Built-in IDE | ✅ (TrustCode) | ✅ (HackStudio) | ❌ | ✅ | ❌ |
| Built-in Language | ✅ (TrustLang) | ❌ | ❌ | ✅ (HolyC) | ❌ |
| **Kernel Introspection Lab** | **✅ (FIRST)** | ❌ | ❌ | ❌ | ❌ (external tools) |
| 3D Games | ✅ (FPS + Chess3D + Mario64) | ❌ (2D only) | ❌ | ✅ (16 colors) | Ported |
| Audio Synthesizer | ✅ (8-voice poly) | ❌ | ❌ | ❌ (single voice) | ❌ |
| **Built-in Emulators** | **✅ (GBC + NES + GameLab)** | ❌ | ❌ | ❌ | ❌ |
| **HTTP Server** | **✅ (built-in httpd)** | ❌ | ❌ | ❌ | Via Apache/nginx |
| **Package Manager** | **✅ (TrustPkg, 30+ pkgs)** | Ports | ❌ | ❌ | apt/dnf/pacman |
| **Shell Scripting** | **✅ (variables, loops, if/else)** | ✅ | ✅ | ✅ | ✅ (bash) |
| **Network Security Scanner** | **✅ (TrustScan, 6 tools)** | ❌ | ❌ | ❌ | Via nmap |
| TLS 1.3 from scratch | ✅ | ❌ | ❌ | ❌ | Via OpenSSL |
| Binary Analyzer | ✅ (TrustView) | ❌ | ❌ | ❌ | External (Ghidra) |
| Hypervisor + VMI | ✅ (VT-x/SVM + introspection) | ❌ | ❌ | ❌ | Via KVM (no VMI) |
| Memory safe | ✅ (Rust) | ❌ | ✅ (Rust) | ❌ | ❌ |
| Fully auditable | ✅ | Partially | Partially | ✅ | ❌ |

---

## 📋 Changelog

### v0.4.1 — February 2026
- **Shell Scripting Engine** — Full shell scripting with POSIX-style variables (`$VAR`, `${VAR}`, `${VAR:-default}`), arithmetic expansion (`$((expr))`), control flow (`if/elif/else/fi` with test conditions: `-f`, `-d`, `-z`, `-n`, `=`, `!=`), loops (`for var in list; do...done`, `while cond; do...done`), command substitution (`$(cmd)`), special variables (`$?`, `$$`, `$#`), and script execution via `source`. Integrated into the shell — all commands support variable expansion.
- **HTTP Server (httpd)** — Built-in web server using the existing TCP stack. Dashboard page (`/`), live status (`/status`), RAMFS file browser (`/files/`), REST API endpoints (`/api/info`, `/api/stats`, `/api/processes`). Start/stop/status control via `httpd` command.
- **TrustPkg Package Manager** — Package manager with 30+ packages across 7 categories (system, network, dev, security, graphics, games, utils). Install, remove, search, list, info, and update commands. Full dependency metadata and size tracking.
- **TrustScan Network Security Toolkit** — 6-module security suite: port scanner (TCP SYN/connect), packet sniffer (real-time capture), banner grabber (service identification), host discovery (ARP/ICMP scan), traceroute (hop-by-hop path), vulnerability scanner (CVE checks). Commands via `netscan`.
- **IPv6 + ICMPv6** — IPv6 protocol support with ICMPv6 for next-generation networking.
- **CONTRIBUTING.md** — Comprehensive developer guide: build requirements, architecture overview, how to add commands/drivers/syscalls, integration test guide, code style, PR process.
- **GitHub Actions CI Pipeline** — 3-job CI: Build (cargo build --release), Lint (clippy), Integration Test (QEMU boot + auto-test). Cargo cache, artifact upload.
- **Integration tests 26-30** — 20 new sub-tests covering shell scripting engine, HTTP server, package manager, network security scanner, and IPv6/ICMPv6. Total: 30 integration test groups.

### v0.4.0 — February 2026
- **Game Boy Color Emulator** — Complete CGB emulator running bare-metal. Sharp LR35902 CPU with all 245 base opcodes + 256 CB-prefix operations. Scanline-accurate PPU (160×144) with proper dot-cycle timing across 4 modes. Full CGB extensions: dual VRAM banks, per-tile attributes (palette, bank, flip), 8 BG + 8 OBJ color palettes (RGB555) via BCPS/BCPD/OCPS/OCPD. MBC0/MBC1/MBC3/MBC5 cartridge mappers (up to 4MB ROM, 128KB RAM). Timer (DIV/TIMA/TMA/TAC) with correct falling-edge detection. Joypad, OAM DMA, HDMA, 32KB WRAM with CGB bank switching, HRAM. Compile-time ROM embedding via `include_bytes!()`.
- **GameLab — Real-Time GB Analysis Dashboard** — 2,000-line interactive debugger/analysis tool with 5 tabs: **Analyze** (6-panel live view: CPU regs/flags, GPU state, memory hex dump, I/O regs, cart info, input), **Search** (Cheat Engine-style memory search: exact value, changed, unchanged, greater, less — snapshot-based narrowing, 256 results), **Watch** (16-slot address watch list with labels, previous/current values, change highlighting), **Tiles** (tile/sprite viewer: tiles $8000, tiles $8800, OAM sprites), **Trace** (last 64 instructions with PC, opcode, A, F, SP). Toolbar: speed control (0.25×–4×), breakpoints (8 PC breakpoints + single-step + frame advance), save/load state (full CPU/GPU/VRAM/OAM/palettes/timer/RAM snapshot), memory diff (highlights changed bytes in hex dump).
- **NES Emulator** — MOS 6502 CPU with all 151 official opcodes + common unofficial opcodes (LAX, SAX, DCP, ISB, SLO, RLA, SRE, RRA). 2C02 PPU with scanline-accurate background/sprite rendering, scroll registers, sprite 0 hit, 64-color palette. Mappers 0 (NROM), 1 (MMC1), 2 (UxROM), 3 (CNROM). Controller input, OAM DMA, nametable mirroring.
- **TrustMario64 — 3D Platformer** — Software-rendered 3D platformer with 23 player actions (walking, running, jumping, double/triple jump, long jump, backflip, side flip, wall kick, ground pound, dive, swim, ledge grab...), Bob-omb Battlefield level (32×32 heightmap, central mountain, water, bridge, trees, 6 stars, 24 coins), 3 enemy types (Goomba, Bob-omb, Chain Chomp with AI behaviors), TAS engine (save/load state, frame advance, record/replay, rewind, ghost playback, hitbox visualization), Lakitu-style camera with mouse orbit.
- **Desktop integration** — Game Boy, NES, and Mario64 run as windowed desktop apps. GameLab opens as a side panel next to the GB window. Desktop icons for all emulators. Multi-pass rendering keeps emulator content z-order consistent.
- ~9,200 new lines across 23 files. 165K total, 296 source files.

### v0.3.5 — February 2026
- **ACPI Table Generation** — Full RSDP v2 (20+16 byte structure with extended checksum), XSDT (dynamic entry array), MADT (Local APIC + I/O APIC + ISO entries, LINT0/1 NMI), FADT (PM timer port 0x608, SCI IRQ 9, SMI CMD, ACPI enable/disable, PM1a event/control blocks, GPE0, FACS + DSDT pointers, x_ 64-bit generic addresses), DSDT (minimal valid AML: `_S5` sleep object for clean ACPI shutdown).
- **PIC 8259A Emulation** — Full Intel 8259A Programmable Interrupt Controller. ICW1-4 initialization sequence, OCW1 (IMR), OCW2 (EOI: specific, non-specific, rotate), OCW3 (IRR/ISR read). Cascaded master/slave with IRQ2 cascade link. Edge-triggered mode. Spurious IRQ 7/15 detection. Priority rotation.
- **PIT 8254 Emulation** — Intel 8254 Programmable Interval Timer. 1.193182 MHz base. Modes 0 (interrupt on terminal count), 2 (rate generator), 3 (square wave). Channels 0-2. Lobyte/Hibyte/Word access modes. Latch command. Tick-based countdown with `advance()` and elapsed tick calculation.
- **CMOS RTC Emulation** — Motorola MC146818. Registers 0x00-0x09 (seconds→year) in BCD, Status Register A (UIP + divider + rate), B (24h mode, BCD, update-ended interrupt enable), C (interrupt flags read-clear). NMI mask via port 0x70 bit 7. Century register at 0x32.
- **ACPI PM Timer** — 3.579545 MHz counter. 24-bit and 32-bit configurable width. Tick accumulation from TSC delta. Read via port 0x608.
- **Phase 0 Infrastructure** — `setup.sh` (one-liner Linux/macOS/WSL setup), `setup.ps1` (Windows PowerShell), `.github/workflows/release.yml` (CI ISO build on tag), Makefile auto-Limine download, repo cleanup (48 junk files removed from tracking).

### v0.3.4 — February 2026
- **PTY/TTY Subsystem** — Full POSIX TTY layer with line discipline (canonical mode, echo, ISIG signal chars ^C/^Z/^\\). TTY_TABLE with named devices, ioctls (TIOCGPGRP, TIOCSPGRP, TIOCSCTTY, TIOCGSID, TIOCGWINSZ, TIOCSWINSZ, TCGETS, TCSETS). Termios struct with ECHO, ICANON, ISIG flags.
- **Pseudo-Terminal Pairs** — PTY master/slave architecture. `alloc_pty()` creates linked pairs with ring buffers. Master write → slave read (with line discipline), slave write → master read.
- **Job Control** — Process groups (pgid) and sessions (sid) on every PCB. New syscalls: SETPGID, GETPGRP, SETSID, GETPGID, GETSID. `kill_process_group()` for signal delivery to process groups. Controlling TTY per process.
- **`/etc/passwd` Persistence** — `load_from_filesystem()` reads `/etc/passwd` from ramfs on boot, parses `UserEntry` records, merges with defaults. `sync_to_filesystem()` writes current user database back to `/etc/passwd`.
- **ELF Improvements** — `resolve_path()` searches PATH across 5 directories (`/bin`, `/usr/bin`, `/sbin`, `/usr/sbin`, `/usr/local/bin`). `check_shebang()` for `#!` script detection. Auxiliary vector on user stack (AT_PAGESZ, AT_PHDR, AT_PHENT, AT_PHNUM, AT_ENTRY, AT_RANDOM, AT_NULL). Proper envp NULL terminator and argv parsing.
- **`chroot` Syscall** — Per-process `root_dir` field. `sys_chroot()` validates directory and updates process root. Inherited on `fork()`.
- **NVMe-Backed Swap** — Swap pages to last 64MB of NVMe disk (8 sectors/page). `write_swap_slot()` / `read_swap_slot()` try NVMe first, fallback to BTreeMap in-memory.
- **Kernel Stack Hardening** — All kernel stacks increased from 16KB to 64KB (GDT, thread, userland syscall stacks).
- **96/96 integration tests passing.** 143K+ lines, 262 source files.

### v0.2.0 — February 2026
- **Ring 3 Userspace Execution** — Real CPL-3 process execution via `IRETQ` with setjmp/longjmp-style kernel return (`exec_ring3_process` / `return_from_ring3`). Page-aligned physical memory allocation. SYSCALL/SYSRET for fast system calls. EXIT/EXIT_GROUP handlers for clean process termination.
- **Embedded ELF64 Binary** — 183-byte static ELF64 hello-world binary embedded in kernel. Full ELF loader maps LOAD segments to user address space at `0x400000`. Shell commands: `exec test` (raw machine code) and `exec hello` (ELF parse + load).
- **TrustFS Reliability** — `free_block()` clears bitmap + increments free count. `free_inode_blocks()` frees all direct + indirect blocks. `unlink()` properly reclaims storage when nlink=0. `truncate()` frees blocks beyond new size.

### v0.1.9 — February 2026
- **TrustLab v3 — UX Overhaul** — Full mouse/click interaction on all 7 panels. Previously only 3 panels handled clicks; now Hardware Status (scroll stats), Kernel Trace (filter toggles, event selection), Command Guide (category tabs, row selection), Pipeline (stage flash, flow scroll), File Tree, TrustLang Editor, and Hex Editor all respond to mouse input.
- **Structured syscall tracing** — New `emit_syscall()` in trace bus with `syscall_nr`, `syscall_args[3]`, `syscall_ret` fields. Human-readable syscall name mapping (50+ Linux x86_64 syscalls + TrustOS-specific 0x1000-0x1003). Every syscall now emits structured trace events. Kernel Trace shows syscall badges, args, and color-coded return values (green=success, red=error).
- **Event detail panel** — Click any event in Kernel Trace to see full message, syscall name + args + return value in a 4-line detail panel at bottom of trace view.
- **Automated UX test suite** — New `ux_test.rs` module with 9 tests: tab cycle (6 presses visit all panels), shell commands (7 cmds → correct panels), click focus (all 7 panel centers), trace event propagation, syscall structured data, filter key toggle, hardware live data, pipeline updates, guide search input. Triggered by `labtest` shell command. Results emitted to Kernel Trace + serial output.
- **Network fixes** — E1000 RX poll loop bounded to prevent serial flood, TCP/IP robustness improvements.

### v0.1.8 — February 2026
- **TrustLab Demo Mode** — Cinematic 27-second narrated demo with Matrix-themed Morpheus intro ("Are you ready to see the Matrix, Neo?"), 23 slides with panel-targeted narration, glitch transitions, red text overlay, progress bar with timer, space-to-skip navigation. PIT-based timing (100Hz) for reliable playback.
- **TrustLab v2** — Upgraded to 7 panels: added Hex Editor for raw byte inspection, Execution Pipeline visualizer. Improved zero-cost trace bus.
- **Audio synthesis engine** — TrustSynth: 8-voice polyphonic synthesizer (sine, square, sawtooth, triangle, noise), ADSR envelope generator, 48kHz 16-bit stereo, Q16.16 fixed-point DSP. Pattern sequencer (16 patterns, 64 steps, configurable BPM). Intel HDA driver.
- **Web Sandbox & Container** — Kernel-level sandboxed execution with `SandboxPolicy` presets, jailed filesystem (`SandboxFs`), JS threat scanner, DNS allow/deny `NetProxy`, capability tokens, watchdog timer, health checks, full audit trail.
- **TrustView binary analyzer** — Ghidra-style ELF64 parser + x86_64 disassembler with cross-reference analysis, function detection, string extraction. Desktop GUI + CLI.

### v0.1.7 — February 2026
- **TrustLab — OS Introspection Laboratory** — Real-time 7-panel educational workspace (`lab` / `trustlab` command, or Start Menu). Panels: Hardware Status (CPU gauge, heap bar, IRQ rate, uptime, alloc stats), Kernel Trace (scrolling event log with category filters, pause toggle), Command Guide (searchable reference of ~55 commands, fuzzy search, category tabs), File Tree (VFS browser with expand/collapse, file sizes, color-coded extensions), TrustLang Editor (syntax-highlighted code editor with F5 execution and output pane), Hex Editor (raw byte inspection), Execution Pipeline (data flow visualizer). Tab/Shift+Tab panel navigation. Zero-cost trace bus (512-slot ring buffer, gated by `LAB_ACTIVE` flag). Kernel hooks in interrupts, VFS, scheduler, and memory allocator emit events in real time.

### v0.1.6 — February 2026
- **TrustOS Film** — Built-in animated cinematic explainer (`film` command): 5-act narrative structure (The Question → The Problem → The Solution → The Proof → The Future) with 12 scene-specific animations: floating windows, question marks rain, screen shatter, binary flood, redacted bars, earthquake shake bar chart, light burst, odometer counter (0→120K), glow pulse feature cards, sparkle dissolve, expanding shockwave rings, matrix rain callback. 8 unique animated backgrounds (pulsing nebula, red scanlines, blueprint dot-grid, green sparks, starfield, circuit traces, sunrise gradient, matrix rain). All rendering integer-only (no_std compatible).
- **Ed25519 Asymmetric Signatures** — Full RFC 8032 implementation: SHA-512, extended twisted Edwards curve (GF(2^255-19) field reuse from TLS), TweetNaCl-style scalar mod l reduction. Replaces forgeable HMAC-only system with proper public-key cryptography. `signature ed25519` shell command for verification.
- **Cross-platform build system** — GNU Makefile + `build.sh` for Linux/macOS with auto-detected OVMF, no hardcoded Windows paths. `make run`, `make iso`, `make check-deps`.
- **TrustLang Showcase syntax highlighting** — Proper multi-category coloring: keywords (red), function calls (blue), variable declarations (cyan), string literals (orange), comments (green), numbers (green), brackets (gold). Replaced per-character word matching with full-line tokenizer.
- **TrustLang Showcase auto-scroll** — Editor panel now scrolls automatically when typed code exceeds visible area, with scrollbar indicator. Cursor line stays visible during typing animation.

### v0.1.5 — February 2026
- **3D Chess Game** — Full chess game with 3D low-poly pieces, proper look-at camera system (spherical coordinates, forward/right/up basis vectors), AI opponent (minimax depth 2), board labels, shadows, reflections, scroll zoom, piece selection highlighting, Matrix green theme
- **Chess engine** — Complete chess rules: castling, en passant, pawn promotion, check/checkmate detection, legal move validation
- **Camera redesign** — Replaced simple rotate-world projection with proper look-at camera: telephoto FOV, no edge distortion, centered projection

### v0.1.4 — February 2026
- **Terminal neofetch** — ASCII art TrustOS banner with OS/Kernel/Arch/Uptime/Memory/Shell/Display info
- **Terminal command aliases** — `user`/`users`/`id`, `hostname`, `history`, `del`, `top`, `lsblk`, `ipconfig`, `version`, `time`
- **Colorized terminal outputs** — date, uname, free, net, ps, df, mkdir/touch/rm all use color markers
- **Timestamp cyan** — Terminal prompt timestamps now in cyan instead of gold
- **Help arguments cyan** — Terminal help shows arguments in cyan for better readability

### v0.1.3 — February 2026
- **Keyboard Shift fix** — Fixed permanent Shift stuck caused by `0xAA` scancode filter
- **Terminal color system** — `\x01` prefix color markers parsed at render time (R/G/B/W/Y/M/H)
- **Terminal scrollbar** — Track + thumb, auto-scroll to bottom, unlimited history
- **Categorized help** — File System, System, Network, Graphics & Demos, Shell sections
- **Working cwd** — `cd`/`pwd`/`ls` now use actual current directory from ramfs
- **Colored prompt** — Timestamp + red root + cyan cwd path

### v0.1.2 — February 2026
- **Desktop shortcuts** — ESC close, Alt+Tab switch, Win+Arrows snap windows
- **Network auto-detection** — CPUID/ACPI/PCI platform detection, DHCP DNS
- **Browser improvements** — CSS/forms/HTTPS, chunked transfer, HTTP redirects
- **TrustDoom3D** — 3D FPS game integrated into desktop
- **TrustCode editor fixes** — Save with touch(), default file path

### v0.1.1 — June 2025
- **TrustEdit 3D Model Editor** — New wireframe 3D editor with 5 tools, 8 presets, save/load `.t3d`, undo system
- **Interactive Calculator** — Full arithmetic with keyboard & mouse, chained operations
- **Interactive Snake Game** — Real-time gameplay with arrow keys, scoring, speed progression
- **Browser keyboard input** — Type URLs, navigate, clear with Escape
- **Start Menu fix** — All 13 pinned apps now clickable and functional
- **`trustedit` shell command** — Launch TrustEdit directly from the shell
- **Desktop stability** — Fixed dual start menu conflict, improved window management

### v0.1.0 — June 2025
- Initial release: 99K+ lines, 207+ files, full desktop, shell, browser, network, TrustLang, hypervisor

---

## 📄 License

MIT License — see [LICENSE](LICENSE) for details.

---

## 🙏 Acknowledgments

- [Limine](https://github.com/limine-bootloader/limine) — Bootloader
- [Rust OSDev](https://os.phil-opp.com/) — Inspiration
- [Alpine Linux](https://alpinelinux.org/) — Linux subsystem base

---

## 👤 Author

**Nated0ge** — Sole creator & developer of TrustOS

- GitHub: [@nathan237](https://github.com/nathan237)
- Project: [TrustOS](https://github.com/nathan237/TrustOS)

> Every line of TrustOS — 165,000+ lines of Rust — was designed, written, and tested by a single developer. 13 days. Zero C. Zero compromises.

---

<div align="center">

**Trust** the code. **Rust** is the reason.

Created with ❤️ by [Nated0ge](https://github.com/nathan237)

165,000+ lines · 13 days · Zero C · Fully auditable

⭐ **Star this repo** if you believe in transparent, auditable operating systems.

[Report Bug](https://github.com/nathan237/TrustOS/issues) · [Request Feature](https://github.com/nathan237/TrustOS/issues) · [Watch Demo](https://youtu.be/RBJJi8jW1_g)

</div>
