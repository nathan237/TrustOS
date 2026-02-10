<![CDATA[<div align="center">

# 🦀 TrustOS

**A complete bare-metal operating system written in 100% Rust — built in 7 days**

[![Build](https://img.shields.io/badge/build-passing-brightgreen?style=for-the-badge)]()
[![Rust](https://img.shields.io/badge/rust-nightly-F74C00?style=for-the-badge&logo=rust&logoColor=white)]()
[![Lines](https://img.shields.io/badge/code-99%2C000%2B%20lines-blue?style=for-the-badge)]()
[![ISO](https://img.shields.io/badge/ISO-6.25%20MB-purple?style=for-the-badge)]()
[![Platform](https://img.shields.io/badge/x86__64-UEFI-lightgrey?style=for-the-badge)]()
[![License](https://img.shields.io/badge/license-MIT-green?style=for-the-badge)]()

**GPU Compositing · 144 FPS Desktop · Built-in Compiler · TLS 1.3 · Filesystem**

[![Watch the demo](https://img.shields.io/badge/▶%20Watch%20Demo-YouTube-red?style=for-the-badge&logo=youtube&logoColor=white)](https://youtu.be/RBJJi8jW1_g)

[Features](#-features) · [Quick Start](#-quick-start) · [Screenshots](#-screenshots) · [Architecture](#-architecture) · [Contributing](#-contributing)

---

</div>

## 🎯 What is TrustOS?

TrustOS is a **complete operating system** built from scratch in Rust — no C, no libc, no unsafe dependencies. It boots on real x86_64 hardware (UEFI) and includes everything from a GPU-accelerated desktop to a built-in programming language compiler.

### Key Stats

| Metric | Value |
|--------|-------|
| **Total code** | 99,000+ lines of Rust |
| **Source files** | 207 `.rs` files |
| **ISO size** | 6.25 MB |
| **Kernel binary** | 2.34 MB |
| **Boot time** | ~2 seconds |
| **Desktop FPS** | 144 FPS (SSE2 SIMD) |
| **RAM usage** | 256 MB |
| **Development time** | 7 days |

### Why TrustOS?

| Traditional OS | TrustOS |
|:---:|:---:|
| C/C++ with memory bugs | **100% Rust** — memory safe by design |
| Millions of lines, decades old | **99K lines**, clean and readable |
| Complex toolchains | `cargo build` — that's it |
| Separate userland tools | **Everything built-in** — shell, editor, compiler, browser |

---

## ✨ Features

### 🖥️ COSMIC2 Desktop Environment
- **Multi-layer GPU compositor** with 8 independent rendering layers
- **SSE2 SIMD** optimized — 144 FPS with zero flickering
- **Dock, taskbar, window management**, start menu, settings panel
- **HoloMatrix 3D** background: volumetric wireframe scenes (cube, torus, DNA helix...)
- **Mouse + keyboard** driven with smooth cursor

### 📝 TrustCode — Built-in Code Editor
- **Rust syntax highlighting** with 60+ keywords
- **Line numbers**, cursor navigation, scrolling
- **File save/load** from TrustFS
- **Bracket matching** and auto-indentation

### 🔤 TrustLang — Integrated Programming Language
- **Rust-inspired syntax** with functions, recursion, loops, types
- **Full compiler pipeline**: Lexer → Parser → Compiler → Bytecode VM
- **Zero dependencies** — compiles and runs entirely in-kernel
- Commands: `trustlang run`, `trustlang eval`, `trustlang check`

### 📁 TrustFS — Persistent Filesystem
- **Block-based storage** with indirect block support
- **Write-Ahead Logging (WAL)** for crash safety
- **Block cache** for performance
- **VFS layer** unifying ramfs, procfs, devfs, and TrustFS

### 🎬 TrustVideo — Real-time Video Codec
- **Custom `.tv` format** with delta + RLE compression
- **Procedural demo engine**: fire, matrix rain, plasma effects
- **60-72 FPS** rendering with integer sine LUT (no floats)
- **RAM backbuffer** + SSE2 swap for instant display

### 🌐 Network Stack
- **VirtIO-net** driver with full packet handling
- **TCP/IP** stack from scratch (ARP, DHCP, DNS, TCP, UDP)
- **TLS 1.3** with HTTPS client support
- **Built-in commands**: `curl`, `wget`, `ping`, `nslookup`, `traceroute`, `netstat`

### 🐧 Linux Compatibility
- **100+ syscalls** emulated (read, write, mmap, fork, exec...)
- **ELF binary loader** — run Linux binaries directly
- **Alpine Linux subsystem** — `apk` package manager support
- **Binary-to-Rust transpiler** — analyze and convert Linux binaries

### 🛡️ Security
- **Capability-based** security model
- **User authentication** (login, su, passwd, adduser)
- **File permissions** (chmod, chown)
- **Process isolation** with Ring 0/3 separation

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

### Build & Run

```bash
# Clone
git clone https://github.com/nathan237/TrustOS.git
cd TrustOS

# Build kernel
cargo build --release -p trustos_kernel

# Create ISO
cp target/x86_64-unknown-none/release/trustos_kernel iso_root/boot/
xorriso -as mkisofs -b boot/limine/limine-bios-cd.bin \
  -no-emul-boot -boot-load-size 4 -boot-info-table \
  --efi-boot boot/limine/limine-uefi-cd.bin \
  -efi-boot-part --efi-boot-image --protective-msdos-label \
  iso_root -o trustos.iso
./limine/limine bios-install trustos.iso

# Run
qemu-system-x86_64 \
  -cdrom trustos.iso \
  -m 256M -machine q35 -smp 4 \
  -display gtk -vga std \
  -device virtio-gpu-pci \
  -device virtio-net-pci,netdev=net0 \
  -netdev user,id=net0 \
  -drive "if=pflash,format=raw,file=OVMF.fd" \
  -serial stdio
```

### Windows (PowerShell)
```powershell
cargo build --release -p trustos_kernel
.\run-qemu.ps1
```

---

## 📸 Screenshots

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

## 🏗️ Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                     Applications                            │
│  TrustCode · TrustLang · Browser · File Manager · Terminal  │
├─────────────────────────────────────────────────────────────┤
│              COSMIC2 Desktop Compositor                     │
│     8-layer GPU compositing · SSE2 SIMD · 144 FPS          │
├──────────┬──────────┬───────────┬──────────┬────────────────┤
│  VFS     │ Network  │  Linux    │ Graphics │  TrustVideo    │
│  ramfs   │ TCP/IP   │ Subsystem │ TrustGL  │  Codec +       │
│  procfs  │ TLS 1.3  │ 100+     │ Raytrace │  Procedural    │
│  devfs   │ DHCP/DNS │ syscalls  │ 3D Mesh  │  Renderer      │
│  TrustFS │ VirtIO   │ ELF      │ HoloMat  │  Fire/Plasma   │
├──────────┴──────────┴───────────┴──────────┴────────────────┤
│                    TrustOS Kernel                           │
│  Memory · Scheduler · IPC · Security · Drivers · Syscalls  │
│  SSE2 SIMD · SMP Multi-core · Double-buffered FB           │
├─────────────────────────────────────────────────────────────┤
│              Hardware (x86_64 · UEFI · VirtIO)              │
│              Limine Bootloader · MMIO · PCI                 │
└─────────────────────────────────────────────────────────────┘
```

### Module Breakdown

| Module | Lines | Description |
|--------|-------|-------------|
| `shell.rs` | ~13,500 | Command interpreter, 200+ commands |
| `compositor/` | ~3,000 | Multi-layer GPU compositor |
| `network/` | ~5,000 | Full TCP/IP stack with TLS |
| `video/` | ~1,500 | TrustVideo codec & player |
| `trustlang/` | ~2,000 | Compiler + bytecode VM |
| `framebuffer/` | ~1,500 | SSE2 SIMD rendering |
| `graphics/` | ~4,000 | HoloMatrix, raytracer, 3D mesh |
| `filesystem/` | ~2,000 | TrustFS with WAL |
| `linux/` | ~3,000 | Linux syscall emulation |

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
1. **System info** — neofetch, uname, memory stats
2. **Filesystem** — create files, directory tree
3. **TrustLang** — compile & run a Fibonacci program live
4. **Network** — interface config, connection status
5. **Video effects** — fire, matrix rain, plasma (auto-timed)
6. **Command overview** — categorized command summary

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

### Project Structure
```
kernel/src/
├── main.rs              # Kernel entry point
├── shell.rs             # Shell + 200+ commands
├── compositor/          # COSMIC2 desktop
├── network/             # TCP/IP, TLS, DHCP
├── video/               # TrustVideo codec
├── trustlang/           # Compiler + VM
├── framebuffer/         # SSE2 rendering
├── graphics/            # 3D, raytracer, HoloMatrix
├── filesystem/          # TrustFS persistence
└── linux/               # Syscall emulation
```

---

## 📊 Comparison

| Feature | TrustOS | Linux 0.01 (1991) | MenuetOS | SerenityOS |
|---------|---------|-------------------|----------|------------|
| Language | Rust | C | ASM | C++ |
| Lines of code | 99K | 10K | 40K | 800K+ |
| ISO size | 6.25 MB | N/A | 1.44 MB | ~300 MB |
| Dev time | 7 days | 6 months | Years | Years |
| GUI | Yes (144 FPS) | No | Yes | Yes |
| Built-in compiler | Yes (TrustLang) | No | No | No |
| Network + TLS | Yes (TLS 1.3) | No | No | Yes |
| Memory safe | Yes (Rust) | No | No | No |

---

## 📄 License

MIT License — see [LICENSE](LICENSE) for details.

---

## 🙏 Acknowledgments

- [Limine](https://github.com/limine-bootloader/limine) — Bootloader
- [Rust OSDev](https://os.phil-opp.com/) — Inspiration
- [Alpine Linux](https://alpinelinux.org/) — Linux subsystem base

---

<div align="center">

**Built with 🦀 Rust — 99,000 lines — 7 days — Zero C code**

⭐ **Star this repo** if you find it impressive!

[Report Bug](https://github.com/nathan237/TrustOS/issues) · [Request Feature](https://github.com/nathan237/TrustOS/issues)

</div>
]]>
