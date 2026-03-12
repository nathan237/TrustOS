<div align="center">

# TrustOS

### The OS you can actually read.

**A bare-metal operating system written entirely in Rust — 257,000 lines, zero C, zero binary blobs, zero secrets.**

*Built by one developer. Auditable by anyone.*

[![Build](https://img.shields.io/badge/build-passing-brightgreen?style=for-the-badge)]()
[![Rust](https://img.shields.io/badge/100%25%20Rust-F74C00?style=for-the-badge&logo=rust&logoColor=white)]()
[![Lines](https://img.shields.io/badge/code-257%2C000%2B%20lines-blue?style=for-the-badge)]()
[![Architectures](https://img.shields.io/badge/arch-x86__64%20%7C%20ARM64%20%7C%20RISC--V-blueviolet?style=for-the-badge)]()
[![Version](https://img.shields.io/badge/version-0.8.0-orange?style=for-the-badge)]()
[![Tests](https://img.shields.io/badge/tests-96%2F96%20(100%25)-brightgreen?style=for-the-badge)]()
[![License](https://img.shields.io/badge/license-Apache%202.0-blue?style=for-the-badge)](LICENSE)
[![Author](https://img.shields.io/badge/created%20by-Nated0ge-ff69b4?style=for-the-badge&logo=github&logoColor=white)](https://github.com/nathan237)

[![Watch the demo](https://img.shields.io/badge/%E2%96%B6%20Watch%20Demo-YouTube-red?style=for-the-badge&logo=youtube&logoColor=white)](https://youtu.be/RBJJi8jW1_g)

[What's New](#-whats-new-in-v080) | [Download](#-download) | [Why TrustOS](#-why-trustos) | [JARVIS AI](#-jarvis----on-device-ai) | [Features](#-features) | [Quick Start](#-quick-start) | [Changelog](#-changelog)

---

</div>

## 🆕 What's New in v0.8.0

> **March 12, 2026** — Native x86_64 Compiler, Desktop Refresh, 96/96 Tests

- 🔥 **TrustLang now compiles to native x86_64 machine code** — write a program, compile it to raw Intel instructions, execute it bare-metal. No LLVM, no GCC, no external tools. Dual backend: bytecode VM + native.
- 🧪 **55+ automated compiler tests** — arithmetic, variables, control flow, recursion, bitwise, edge cases, cross-validation between VM and native output.
- 🎯 **New commands**: `trustlang compile`, `trustlang test`, `trustlang bench` — benchmark native vs interpreted.
- 🖥 **COSMIC2 desktop borders refined** — 4px chrome for a bolder, modern window look.
- ✅ **96/96 self-tests passing** (100%) — native compiler smoke test integrated.
- 📊 **257,000+ lines** across 473 source files, 3 architectures.

---

## 📥 Download

**Grab an ISO and boot it in 30 seconds:**

| Edition | ISO | Size | What's Inside | Download |
|---------|-----|------|---------------|----------|
| **TrustOS** | `trustos.iso` | ~12 MB | Full OS: desktop, networking, emulators, TrustLang, TrustLab, 200+ commands. JARVIS engine included (untrained). | [**⬇ Download**](https://github.com/nathan237/TrustOS/releases/latest/download/trustos.iso) |
| **TrustOS + JarvisPack** | `trustos-jarvispack.iso` | ~29 MB | Everything above + **pretrained JARVIS brain** (4.4M-param transformer). AI ready out of the box. | [**⬇ Download**](https://github.com/nathan237/TrustOS/releases/latest/download/trustos-jarvispack.iso) |

```bash
# Boot it right now in QEMU:
qemu-system-x86_64 -cdrom trustos.iso -m 512M -cpu max -smp 4 -display gtk -vga std -serial stdio
```

> 💡 Or flash to a USB drive and boot on real hardware — TrustOS runs bare-metal on x86_64 PCs, ARM64 phones/tablets, and RISC-V boards.

All releases: [**github.com/nathan237/TrustOS/releases**](https://github.com/nathan237/TrustOS/releases)

---

## 🔑 Why TrustOS?

TrustOS does things that **no other operating system on Earth** does. Here's why you should install it:

### 🧠 An AI that lives inside the kernel
JARVIS is a **4.4-million-parameter transformer** running in Ring 0 — bare-metal, zero cloud, zero API calls. It trains on-device, generates text, federates its weights across a mesh network, and **replicates itself to new machines over PXE boot**. No other OS has a real neural network baked into the kernel.

### ⚡ A compiler that generates native machine code — inside the OS
TrustLang compiles your programs to **raw x86_64 Intel instructions** and executes them, all from within the kernel. No LLVM. No GCC. No external toolchain. Write code → native binary → execute. The entire compiler is 3,555 lines of Rust.

### 🔍 257,000 lines — and you can read every single one
No binary blobs. No proprietary drivers. No hidden telemetry. **Every driver, every protocol, every pixel, every encryption algorithm, every line of AI** is open Rust. One developer built it. Anyone can audit it.

### 🌐 Self-replicating distributed AI
PXE boot a blank machine → TrustOS installs itself → JARVIS wakes up → joins the mesh network → starts federated learning with other nodes. Fully autonomous AI propagation over a LAN.

### 🖥 A real desktop OS, not a toy
144 FPS SIMD-accelerated desktop with 14+ apps, a browser with HTML/CSS/JS, Game Boy and NES emulators, 3D chess, a code editor, network security toolkit, and the most complete kernel introspection lab ever built into a bare-metal OS.

### 🏗 Everything from scratch — zero dependencies
TLS 1.3, TCP/IP, DNS, DHCP, HTTP/HTTPS, FAT32, EXT4, NVMe, AHCI, VirtIO, VT-x/AMD-V hypervisor, Ed25519 signatures, audio synthesizer — all written in pure Rust. Not a single line of C.

### 📱 Runs on 3 architectures from one codebase
x86_64 PCs, ARM64 (Android phones, Raspberry Pi), RISC-V — same source, same OS.

> *"The only OS where you can trace every packet, every pixel, every AI inference, and every keystroke back to its source code."*

---

## 🏆 Feature Highlights

These are things no other bare-metal OS does:

### 1. Native x86_64 Compiler Running Inside the OS

TrustLang is a full programming language **built into the kernel** — with a lexer, parser, AST compiler, bytecode VM, **and a native x86_64 machine code backend**. You can write a program, compile it to raw Intel instructions, and execute it — all from within TrustOS, with zero external tools.

```
trustlang compile my_app.tl   → Compiles to native x86_64 and executes
trustlang run my_app.tl       → Bytecode VM execution
trustlang bench               → Benchmark native vs VM (shows speedup)
trustlang test                → 55+ automated test suite
```

3,555 lines. Lexer → Parser → AST → Bytecode OR native x86_64. Dual execution backends.

### 2. Kernel-Resident AI Transformer (JARVIS)

A **4.4-million-parameter transformer** running in Ring 0 kernel space. Not an API wrapper. Not a cloud call. A real neural network with attention heads, backpropagation, and Adam optimizer — executing bare-metal on the CPU, trained and inferenced entirely on-device.

It can learn, generate text, federate its weights across a mesh network, and replicate itself to new machines via PXE boot.

14,443 lines of Rust. 24 modules. Zero ML framework dependencies.

### 3. Self-Replicating Over the Network

TrustOS can PXE boot new machines, push the kernel binary over TFTP, and bring up identical TrustOS instances on bare hardware — automatically. Combined with JARVIS mesh networking, this creates a distributed AI that spreads across a LAN.

### 4. Real-Time Kernel Introspection (TrustLab)

A 7-panel interactive lab that lets you watch the kernel at work in real-time: hardware status, kernel trace bus, file system tree, hex editor, TrustLang editor with syntax highlighting, execution pipeline. No other bare-metal OS ships this.

### 5. Everything From Scratch

TLS 1.3, TCP/IP, DNS, DHCP, HTTP, HTML/CSS parser, JavaScript engine, FAT32, EXT4, NVMe, AHCI, VirtIO, Game Boy emulator, NES emulator, 3D renderer, audio synthesizer, Ed25519 signatures, chess AI — all written in Rust, from zero, with no external libraries.

---

## 📦 Editions

TrustOS ships in **two official editions**:

| | **TrustOS** | **TrustOS + JarvisPack** |
|---|---|---|
| **ISO** | [`trustos.iso`](https://github.com/nathan237/TrustOS/releases/latest/download/trustos.iso) | [`trustos-jarvispack.iso`](https://github.com/nathan237/TrustOS/releases/latest/download/trustos-jarvispack.iso) |
| **Size** | ~12 MB | ~29 MB |
| **What's inside** | Full OS, desktop, network, emulators, TrustLang, TrustLab, 200+ commands. JARVIS engine (untrained). | Everything above + **pretrained JARVIS transformer** (4.4M params). AI inference out of the box. |
| **AI Status** | Engine present, no weights | Fully trained, ready to chat |
| **Use Case** | General-purpose OS, development, learning | AI workloads, mesh networking, federated learning |
| **Build** | `.\build-trustos.ps1` | `.\build-trustos-jarvispack.ps1` |

---

## 🤖 JARVIS — On-Device AI

> **The first bare-metal OS with a kernel-resident AI that learns, reasons, and communicates — without any external service.**

| Feature | Details |
|---------|---------|
| **Architecture** | Custom transformer: 8 attention heads, 6 layers, 512-dim embeddings, byte-level tokenizer (256 vocab) |
| **Parameters** | 4.4 million trainable weights, all in kernel memory |
| **Training** | On-device backpropagation with Adam optimizer — runs bare-metal, no ML framework |
| **Inference** | Real-time text generation from Ring 0, ~80 tokens in milliseconds |
| **Federated Learning** | Nodes sync model weights across the network — distributed training |
| **Mesh Networking** | Raft-consensus mesh: leader election, peer discovery, weight synchronization |
| **PXE Replication** | Self-propagating: JARVIS replicates to new nodes via PXE boot over TFTP |
| **Hardware Awareness** | `jarvis_hw` probes CPU, memory, PCI, network — JARVIS knows its own hardware |
| **Guardian System** | The Pact: AI cannot modify OS code without authorization from its guardians |
| **SIMD Optimized** | SSE2/AVX matrix operations for accelerated inference |

### JARVIS Commands

```
jarvis status          # Model info: parameters, layers, training state
jarvis chat <prompt>   # Generate text from the transformer
jarvis train <text>    # On-device backpropagation training
jarvis eval            # Evaluate model loss
mesh start             # Start Raft mesh networking
mesh status            # Show peers and roles
federated enable       # Enable federated learning
pxe start              # Start PXE replication server
propagate              # Auto-propagate to discovered nodes
```

### The Pact

JARVIS has a hard-coded guardian system ([`kernel/src/jarvis/guardian.rs`](kernel/src/jarvis/guardian.rs)). Protected operations (training, weight push, model replace, PXE replicate) require explicit authorization from its two guardians. This is enforced at the kernel level and cannot be bypassed.

### JARVIS Codebase

```
kernel/src/jarvis/            # 14,443 lines across 24 modules
  model.rs                    # Transformer architecture (4.4M params)
  training.rs                 # On-device backpropagation
  backprop.rs                 # Gradient computation
  inference.rs                # Text generation
  optimizer.rs                # Adam optimizer
  tokenizer.rs                # Byte-level tokenizer
  mesh.rs                     # Raft-consensus mesh networking
  federated.rs                # Federated learning (weight sync)
  rpc.rs                      # Remote procedure calls
  pxe_replicator.rs           # PXE boot self-replication
  guardian.rs                 # The Pact (authorization)
  simd.rs                     # SSE2/AVX matrix acceleration
  compression.rs              # Model weight compression
  consensus.rs                # Raft leader election
  compute.rs                  # Distributed compute scheduler
  corpus.rs                   # Training corpus management
  mentor.rs                   # Serial-port mentor protocol
  agent.rs                    # Autonomous agent capabilities
  task.rs                     # Task execution engine
  ...

kernel/src/jarvis_hw/         # 3,111 lines — hardware intelligence
  probe/                      # CPU, memory, PCI, network probing
  hw_corpus.rs                # Hardware-aware training data
```

---

## 🖥 Features

### COSMIC2 Desktop Environment
- Multi-layer GPU compositor: 8 rendering layers, SSE2 SIMD blitting, 144 FPS
- Taskbar, dock, start menu, window manager with 4px chrome borders
- 14+ desktop apps: Terminal, Files, TrustCode, Calculator, Network, Snake, Chess 3D, TrustBrowser, TrustEdit 3D…
- Touch & gesture input for mobile/tablet deployment

### TrustLang — Built-in Programming Language
- Rust-inspired syntax: `fn`, `let`, `mut`, `if/else`, `while`, `for`, `return`, `struct`
- Full pipeline: Lexer → Parser → AST → Compiler → Bytecode VM **+ native x86_64 backend**
- 20 builtin functions: `print`, `pixel`, `fill_rect`, `draw_circle`, `screen_w`, `sleep`…
- REPL, file execution, syntax checker, native benchmark
- 3,555 lines, dual execution mode (interpreted + native)

### Network Stack (all from scratch)
- VirtIO-net driver, TCP/IP (ARP, DHCP, DNS, TCP, UDP, ICMP), IPv6 + ICMPv6
- TLS 1.3 — full handshake, X.509 certificate parsing, all crypto from scratch
- HTTP/HTTPS client + HTTP server (`httpd start/stop/status`)
- Live-tested against real internet (google.com nmap + curl verified)

### TrustScan — Network Security Toolkit
Port scanner, packet sniffer, banner grabber, host discovery, traceroute, vulnerability scanner.

### TrustLab — Kernel Introspection
7-panel interactive workspace: Hardware Status, Kernel Trace, Command Guide, File System Tree, TrustLang Editor, Execution Pipeline, Hex Editor. Zero-cost 512-slot trace bus. Launch: `trustlab`

### Emulators
- **Game Boy Color** — Full CGB: LR35902 CPU, all 501 opcodes, scanline PPU, MBC1/3/5 + GameLab analysis dashboard
- **NES** — MOS 6502 (151 official + 8 unofficial opcodes), 2C02 PPU, mappers 0-3

### Hypervisor & Linux Compatibility
- **Intel VT-x** and **AMD-V** dual-backend with EPT/NPT
- **70+ Linux syscalls**, ELF64 loader, Ring 3 userland, PTY/TTY, job control

### Security & Crypto
- Ed25519 signatures (RFC 8032) — full public-key cryptography
- TLS 1.3 — handshake, AEAD, X.509, all from scratch
- Capability-based security model
- JARVIS Guardian — hardware-locked AI authorization

### Storage
- **TrustFS** — native bare-metal filesystem with WAL journal, indirect blocks
- **FAT32** — read/write for USB/disk interoperability
- **EXT4** — read-only for Linux compatibility
- **RamFS**, **DevFS**, **ProcFS** — in-memory filesystems
- **Persistence** — raw AHCI sector storage for cross-reboot file survival

### Multi-Architecture

| Target | Arch | Method | Status |
|--------|------|--------|--------|
| PC (USB/ISO) | x86_64 | Limine UEFI + BIOS | Production |
| Android | ARM64 | `fastboot flash boot` | Ready |
| Raspberry Pi | ARM64 | SD card (kernel8.img) | Ready |
| RISC-V boards | RISC-V | OpenSBI + U-Boot | Ready |
| QEMU (all 3 archs) | x86_64 / ARM64 / RISC-V | Virtual machine | Ready |

---

## 📊 By the Numbers

| | |
|---|---|
| **257,000+ lines** of pure Rust | **96/96** self-tests passing (100%) |
| **473** source files | **144 FPS** SIMD desktop |
| **3** architectures (x86_64, ARM64, RISC-V) | **< 1 sec** boot time |
| **4.4M** AI parameters in kernel space | **0** lines of C |
| **14,443** lines of AI code (24 modules) | **0** binary blobs |
| **3,555** lines of TrustLang (dual backend) | **0** external ML frameworks |

---

## 🚀 Quick Start

### Option A: Download & run (30 seconds)

1. Grab the ISO from [**Releases**](https://github.com/nathan237/TrustOS/releases) (or use the [Download](#-download) links above)
2. Boot it:

```bash
# TrustOS (base)
qemu-system-x86_64 -cdrom trustos.iso -m 512M -cpu max -smp 4 -display gtk -vga std -serial stdio

# TrustOS + JarvisPack (with pretrained AI)
qemu-system-x86_64 -cdrom trustos-jarvispack.iso -m 512M -cpu max -smp 4 -display gtk -vga std -serial stdio
```

> Or flash to USB with `dd` / Rufus and boot on real hardware.

### Option B: Build from source

```powershell
git clone https://github.com/nathan237/TrustOS.git
cd TrustOS

# Base edition
.\build-trustos.ps1

# AI edition (with pretrained JARVIS weights)
.\build-trustos-jarvispack.ps1
```

### First commands

| Command | What it does |
|---------|-------------|
| `desktop` | Launch COSMIC2 desktop environment |
| `jarvis status` | Check JARVIS AI status |
| `jarvis chat hello` | Chat with on-device AI |
| `trustlang demo` | Run TrustLang demo program |
| `trustlang test` | Run 55+ native backend tests |
| `trustlab` | Open kernel introspection lab |
| `showcase` | Automated feature tour |
| `neofetch` | System info |
| `chess3d` | 3D chess vs AI |
| `gameboy` | Game Boy Color emulator |
| `selftest` | Run 96 automated self-tests |
| `help` | All 200+ commands |

---

## 📁 Project Structure

```
TrustOS/
  kernel/                     # Core bare-metal kernel (253K+ lines)
    src/
      jarvis/                 # JARVIS AI (14,443 lines, 24 modules)
      jarvis_hw/              # Hardware intelligence (3,111 lines)
      trustlang/              # TrustLang compiler + native x86_64 (3,555 lines)
      shell/                  # 200+ commands
      desktop.rs              # COSMIC2 desktop manager
      network/                # TCP/IP, TLS 1.3, DHCP, DNS
      browser/                # HTML/CSS/JS browser engine
      gameboy/                # Game Boy Color emulator
      nes/                    # NES emulator
      hypervisor/             # VT-x/SVM, EPT/NPT
      vfs/                    # TrustFS, FAT32, EXT4, procfs
      tls13/                  # TLS 1.3, crypto, X.509
      netscan/                # Network security toolkit
      drivers/                # AHCI, USB, VirtIO, NVMe, Apple
  userland/                   # Userspace programs
  scripts/
    build/                    # Build scripts (limine, multiarch)
    launch/                   # VM launch scripts (QEMU, VBox)
    test/                     # Test automation
  tools/                      # Python utilities, training
  docs/                       # Documentation, roadmaps, guides
  builds/
    trustos/                  # Base edition ISO output
    trustos-jarvispack/       # JarvisPack edition ISO output
  firmware/                   # UEFI firmware (OVMF)
  limine/                     # Bootloader binaries
  apple/                      # iOS security research
  sdk/                        # Cross-compilation SDK
```

---

## 📋 Changelog

### v0.8.0 — Native Compiler & Desktop Polish (March 12, 2026)

- **TrustLang native x86_64 backend** — compile `.tl` programs directly to Intel machine code and execute in-kernel. Dual execution: bytecode VM + native. 55+ automated tests + cross-validation + benchmarking.
- **x86_64 assembler module** (`x86asm.rs`) — full instruction emitter: MOV, ADD, SUB, IMUL, IDIV, CMP, Jcc, SETcc, CALL, RET, function prologue/epilogue.
- **Desktop border refinement** — window borders thickened to 4px for a bolder, more modern look.
- **Shell commands**: `trustlang compile`, `trustlang test`, `trustlang bench`.
- **Selftest integration** — native compiler smoke test added to the 96-test diagnostic suite.
- 257,000+ lines, 473 source files, 3 architectures.

### v0.7.0 — checkm8 & JARVIS (March 2026)

- **Two official editions**: TrustOS (base, ~12 MB) and TrustOS-JarvisPack (AI, ~29 MB).
- **Project reorganization**: root cleaned from 400+ files to 11.
- **JARVIS AI**: 4.4M-parameter transformer with on-device training, federated learning, mesh networking, PXE replication, Guardian system.
- **checkm8 SecureROM exploit** — bare-metal xHCI USB exploit for Apple A12 DFU mode.
- **Apple hardware drivers** — AIC + UART for native Apple silicon.
- **ARM64 GICv2** — full interrupt controller + exception vectors.
- **COSMIC2 Desktop refresh** — redesigned windows, transparency, icons.

### v0.6.0 — Multi-Arch & Universal Boot (February 2026)

- Multi-Architecture: x86_64, aarch64, riscv64 from one codebase.
- Android boot: boot.img v2 pipeline, `fastboot flash boot`.
- Raspberry Pi SD: bare-metal RPi 4/5.
- Universal installer: 9 targets, one script.
- Touch & gesture input.

### v0.5.0 — CyberLab (February 2026)

- Live network scan verified on google.com — 11/11 tests passed.
- Code optimization (−2,800 lines).
- 95/96 auto-tests (99%).

### v0.4.x — Emulators & GameLab (February 2026)

- Game Boy Color emulator — full CGB: LR35902, scanline PPU, MBC1/3/5.
- GameLab — 2,000-line analysis dashboard.
- NES emulator — 6502 CPU, 2C02 PPU, mappers 0-3.
- Shell scripting, HTTP server, TrustPkg, TrustScan, IPv6.

### v0.3.x — Foundation (February 2026)

- ACPI, PIC, PIT, RTC, PTY/TTY, job control, NVMe swap, SMP.

### v0.2.0 — Userspace (February 2026)

- Ring 3 execution, ELF64 loader, TrustFS.

### v0.1.x — Initial Development (February 2026)

- TrustLab, COSMIC2 desktop, Ed25519, 3D Chess, audio synth, web sandbox.

---

## 📖 Documentation

| Document | Location |
|----------|----------|
| Developer Guide | [`docs/DEVELOPER_GUIDE.md`](docs/DEVELOPER_GUIDE.md) |
| Contributing | [`docs/CONTRIBUTING.md`](docs/CONTRIBUTING.md) |
| Roadmap | [`docs/ROADMAP_V2.md`](docs/ROADMAP_V2.md) |
| Release Notes | [`docs/RELEASE_NOTES.md`](docs/RELEASE_NOTES.md) |
| JARVIS Guardian Pact | [`kernel/src/jarvis/guardian.rs`](kernel/src/jarvis/guardian.rs) |
| Command Reference | [`docs/TRUSTOS_COMPLETE_COMMAND_REFERENCE.md`](docs/TRUSTOS_COMPLETE_COMMAND_REFERENCE.md) |

---

## 🤝 Contributing

Contributions welcome. TrustOS is designed to be readable and hackable.

> See [`docs/CONTRIBUTING.md`](docs/CONTRIBUTING.md) for the full guide.

```bash
git clone https://github.com/YOUR_USERNAME/TrustOS.git
git checkout -b feature/my-feature
cargo build --release -p trustos_kernel
# Test in QEMU, then open a Pull Request
```

---

## 📄 License

Apache License 2.0 — see [LICENSE](LICENSE).

---

## 👤 Author

**Nated0ge** — Sole creator & developer of TrustOS

- GitHub: [@nathan237](https://github.com/nathan237)
- Project: [TrustOS](https://github.com/nathan237/TrustOS)

---

<div align="center">

**Trust** the code. **Rust** is the reason.

Created by [Nated0ge](https://github.com/nathan237) — March 2026

257,000+ lines | Native x86_64 compiler | JARVIS AI (4.4M params) | x86_64 + ARM64 + RISC-V | Zero C

[Report Bug](https://github.com/nathan237/TrustOS/issues) | [Request Feature](https://github.com/nathan237/TrustOS/issues) | [Watch Demo](https://youtu.be/RBJJi8jW1_g)

</div>
