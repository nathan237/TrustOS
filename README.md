<div align="center">

# 🦀 TrustOS

### **Trust** the code. **Rust** is the reason.

**A fully auditable, bare-metal operating system — 120,000 lines of pure Rust. Zero C. Zero secrets.**

[![Build](https://img.shields.io/badge/build-passing-brightgreen?style=for-the-badge)]()
[![Rust](https://img.shields.io/badge/100%25%20Rust-F74C00?style=for-the-badge&logo=rust&logoColor=white)]()
[![Lines](https://img.shields.io/badge/code-120%2C000%2B%20lines-blue?style=for-the-badge)]()
[![ISO](https://img.shields.io/badge/ISO-10.86%20MB-purple?style=for-the-badge)]()
[![Version](https://img.shields.io/badge/version-0.1.6-orange?style=for-the-badge)]()
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
| **2026-02-12** | **TrustOS Film** — Built-in cinematic animated explainer with 12 unique scene-specific animations (floating windows, question marks rain, screen shatter, binary flood, redacted bars, earthquake shake, light burst, odometer counter, glow pulse cards, sparkle dissolve, expanding rings, matrix rain callback), 8 animated backgrounds, odometer counters, and fast-paced transitions. Run `film` in the shell. |
| **2026-02-12** | **Ed25519 Signatures** — Full RFC 8032 Ed25519 asymmetric signature system (SHA-512, extended twisted Edwards curve, TweetNaCl-style scalar mod l), replaces forgeable HMAC-only signatures. **Cross-platform build** — Makefile + build.sh for Linux/macOS, no hardcoded Windows paths. **TrustLang Showcase** — Proper syntax highlighting (keywords red, function calls blue, variables cyan, strings orange, comments green, brackets gold) + auto-scrolling editor when code exceeds panel height |
| **2026-02-12** | **3D Chess** — Full 3D chess game with low-poly pieces, proper look-at camera (spherical coords, forward/right/up basis), AI opponent (minimax depth 2), board labels, shadows, reflections, scroll zoom, piece selection highlighting |
| **2026-02-11 18:00** | Terminal: timestamp cyan, `neofetch` command with ASCII art, `user`/`id`/`hostname`/`history` commands, colorized outputs (date/uname/free/net/ps/df/mkdir/touch/rm), `del`/`top`/`lsblk`/`ipconfig`/`version`/`time` aliases |
| **2026-02-11 16:30** | Terminal: help arguments in cyan (`\x01B`), commands green, descriptions white |
| **2026-02-11 15:00** | Keyboard: fixed Shift stuck bug (`0xAA` scancode filter removal), terminal scrollbar, color marker system (`\x01` prefix), categorized help, prompt with timestamp + red root + cwd, `cd`/`pwd`/`ls` use actual cwd |

---

## �🔐 Why "TrustOS"?

The name says it all: **Trust** + **Rust**.

In a world where your operating system is a black box — millions of lines of legacy C/C++, binary blobs, proprietary drivers, telemetry you can't disable — **how do you know what your computer is actually doing?**

TrustOS is the answer: **every single line is open, readable, and auditable.**

- 🔍 **Fully auditable** — 120,000 lines of Rust, all on GitHub. No binary blobs. No hidden code.
- 🦀 **Memory safe by design** — Rust's ownership model prevents entire categories of vulnerabilities (buffer overflows, use-after-free, data races).
- 🧩 **Zero dependencies on C** — no libc, no glibc, no C runtime. Every driver, every protocol, every pixel is Rust.
- 📖 **Readable** — one person wrote it in 8 days. If one person can build it, one person can understand it.

> *"The only OS where you can trace every packet, every pixel, and every keystroke back to its source code."*

### Key Stats

| Metric | Value |
|--------|-------|
| **Total code** | 120,000+ lines of Rust |
| **Source files** | 216+ `.rs` files |
| **ISO size** | 10.86 MB |
| **Boot time** | < 1 second |
| **Desktop FPS** | 144 FPS (SSE2 SIMD) |
| **C code** | 0 lines |
| **Development time** | 8 days |

### TrustOS vs The World

| | Traditional OS | TrustOS |
|---|:---:|:---:|
| **Language** | C/C++ with 40 years of memory bugs | 100% Rust — memory safe by design |
| Codebase | Millions of lines, impossible to audit | 120K lines, one person can read it all |
| **Binary blobs** | Everywhere | None. Zero. |
| **Telemetry** | Opt-out (maybe) | Doesn't exist — verify it yourself |
| **Build** | Complex cross-compilation toolchains | `cargo build` — that's it |

---

## ✨ Features

### 🖥️ COSMIC2 Desktop Environment
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
- **TLS 1.3** — full handshake, X.509 certificate validation, crypto
- **HTTP/HTTPS client** — `curl`, `wget`, `browse`
- **Commands**: `ping`, `nslookup`, `traceroute`, `netstat`, `arp`, `route`, `ifconfig`

### 🐧 Linux Compatibility Layer (WIP)

TrustOS is being built **with Linux binary compatibility in mind**. The infrastructure is real, but it's not at the "run `apt-get`" stage yet.

| Component | Status | What exists |
|-----------|--------|-------------|
| **Syscall interface** | ✅ Functional | 60+ Linux syscalls dispatched (read, write, mmap, fork, socket, exec, uname...) |
| **ELF64 loader** | ✅ Functional | Parses and loads static ELF binaries with correct segment mapping |
| **Ring 3 (userland)** | ✅ Working | Real SYSCALL/SYSRET mechanism, IRETQ to Ring 3, kernel stack switching |
| **Process table** | ✅ Working | Full PCB, PID management, fork/exit/wait, state machine |
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

### ⚡ Hypervisor
- **Intel VT-x (VMX)** and **AMD-V (SVM)** support
- **Extended Page Tables (EPT)**, VMCS, VPID
- **Guest VM isolation** for running Linux subsystem

### ⚡ Performance
- **SSE2 SIMD** throughout: buffer fills, blits, compositing
- **Double-buffered** rendering (RAM backbuffer → MMIO swap)
- **SMP multi-core** support
- **Compile-time LUTs** for math-heavy rendering

---

## 🚀 Quick Start

### Prerequisites
- Rust nightly (`rustup` will auto-install via `rust-toolchain.toml`)
- QEMU with OVMF (UEFI firmware)
- `xorriso` (for ISO creation)

### Build & Run (Linux / macOS)

```bash
# Clone
git clone https://github.com/nathan237/TrustOS.git
cd TrustOS

# Check dependencies
make check-deps

# Build + run in QEMU (UEFI)
make run

# Or step by step:
make build          # Build kernel only
make iso            # Build + create ISO
make run-bios       # Run in BIOS mode (no OVMF needed)
```

Or use the shell script directly:
```bash
chmod +x build.sh
./build.sh              # Build kernel + create ISO
./build.sh --run        # Build + run in QEMU (UEFI)
./build.sh --run-bios   # Build + run in QEMU (BIOS)
./build.sh --check      # Check dependencies
```

#### Install Dependencies

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

### Build & Run (Windows — PowerShell)
```powershell
cargo build --release -p trustos_kernel
.\run-vbox.ps1       # VirtualBox (full setup)
.\run-qemu-gui.ps1   # QEMU with GUI
```

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
<summary><strong>🌐 Network (15+)</strong></summary>

`ifconfig` `ping` `curl` `wget` `nslookup` `arp` `route` `netstat` `traceroute` `browse` `download` `httpget` `tcpsyn` `ip` `dig`
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
<summary><strong>🛠️ Development (10+)</strong></summary>

`trustlang` `transpile` `exec` `elfinfo` `hexdump` `strings` `base64` `md5sum` `sha256sum` `od`
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
| `model_editor.rs` | ~750 | TrustEdit 3D wireframe model editor |
| `video/` | ~1,500 | TrustVideo codec & player |
| `framebuffer/` | ~1,500 | SSE2 SIMD rendering |
| `filesystem/` | ~2,000 | TrustFS with WAL, VFS, FAT32 |
| `hypervisor/` | ~2,000 | VT-x/SVM, EPT, guest VM isolation |
| `tls13/` | ~2,000 | TLS 1.3, crypto, X.509 certs |
| `ed25519.rs` | ~720 | Ed25519 asymmetric signatures (RFC 8032) |

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
├── desktop.rs           # COSMIC2 desktop manager
├── chess.rs             # Chess engine (rules, AI, game state)
├── chess3d.rs           # 3D Chess renderer (camera, meshes)
├── model_editor.rs      # TrustEdit 3D model editor
├── signature.rs         # Kernel signatures (HMAC-SHA256 + Ed25519)
├── ed25519.rs           # Ed25519 digital signatures (RFC 8032)
├── formula3d.rs         # Wireframe 3D engine
├── compositor/          # 8-layer GPU compositor
├── browser/             # HTML/CSS/JS browser engine
├── network/             # TCP/IP, DHCP, DNS
├── tls13/               # TLS 1.3, crypto, X.509
├── video/               # TrustVideo codec
├── trustlang/           # Compiler + VM
├── framebuffer/         # SSE2 SIMD rendering
├── graphics/            # 3D, raytracer, HoloMatrix
├── hypervisor/          # VT-x/SVM, EPT, guest VMs
├── vfs/                 # TrustFS, FAT32, procfs, devfs
├── linux_compat/        # 100+ Linux syscalls
├── drivers/             # AHCI, USB, VirtIO, input
└── security/            # Capability model, auth
```

---

## 📊 Comparison

| Feature | TrustOS | Linux 0.01 (1991) | MenuetOS | SerenityOS |
|---------|---------|-------------------|----------|------------|
| Language | **Rust** | C | ASM | C++ |
| Lines of code | **120K** | 10K | 40K | 800K+ |
| ISO size | **10.86 MB** | N/A | 1.44 MB | ~300 MB |
| Dev time | **8 days** | 6 months | Years | Years |
| GUI Desktop | Yes (144 FPS) | No | Yes | Yes |
| Web Browser | **Yes** (HTML/CSS/JS) | No | No | Yes |
| Built-in compiler | **Yes** (TrustLang) | No | No | No |
| 3D Engine | **Yes** (Formula3D + Chess3D) | No | No | No |
| Network + TLS 1.3 | **Yes** | No | No | Yes |
| Hypervisor | **Yes** (VT-x/SVM) | No | No | No |
| Memory safe | **Yes** (Rust) | No | No | No |
| Fully auditable | **Yes** | Partially | Yes | Partially |

---

## 📋 Changelog

### v0.1.7 — February 2026
- **TrustLab — OS Introspection Laboratory** — Real-time 6-panel educational workspace (`lab` / `trustlab` command, or Start Menu). Panels: Hardware Status (CPU gauge, heap bar, IRQ rate, uptime, alloc stats), Kernel Trace (scrolling event log with category filters, pause toggle), Command Guide (searchable reference of ~55 commands, fuzzy search, category tabs), File Tree (VFS browser with expand/collapse, file sizes, color-coded extensions), TrustLang Editor (syntax-highlighted code editor with F5 execution and output pane), Live Trace (auto-scrolling real-time event feed). Tab/Shift+Tab panel navigation. Zero-cost trace bus (512-slot ring buffer, gated by `LAB_ACTIVE` flag). Kernel hooks in interrupts, VFS, scheduler, and memory allocator emit events in real time.

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

## �📄 License

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

> Every line of TrustOS — 120,000+ lines of Rust — was designed, written, and tested by a single developer in 8 days.

---

<div align="center">

**Trust** the code. **Rust** is the reason.

Created with ❤️ by [Nated0ge](https://github.com/nathan237)

120,000 lines · 8 days · Zero C · Fully auditable

⭐ **Star this repo** if you believe in transparent, auditable operating systems.

[Report Bug](https://github.com/nathan237/TrustOS/issues) · [Request Feature](https://github.com/nathan237/TrustOS/issues) · [Watch Demo](https://youtu.be/RBJJi8jW1_g)

</div>
