<div align="center">

# TrustOS

### **Trust** the code. **Rust** is the reason.

**A fully auditable, bare-metal operating system -- 260,000+ lines of pure Rust. Zero C. Zero secrets.**

*One dev. One OS. Nothing to hide.*

[![Build](https://img.shields.io/badge/build-local%20passing-brightgreen?style=for-the-badge)]()
[![Rust](https://img.shields.io/badge/100%25%20Rust-F74C00?style=for-the-badge&logo=rust&logoColor=white)]()
[![Lines](https://img.shields.io/badge/code-260%2C000%2B%20lines-blue?style=for-the-badge)]()
[![Architectures](https://img.shields.io/badge/arch-x86__64%20%7C%20ARM64%20%7C%20RISC--V-blueviolet?style=for-the-badge)]()
[![Version](https://img.shields.io/badge/version-0.7.0--checkm8-orange?style=for-the-badge)]()
[![Tests](https://img.shields.io/badge/tests-95%2F96%20(99%25)-brightgreen?style=for-the-badge)]()
[![License](https://img.shields.io/badge/license-Apache%202.0-blue?style=for-the-badge)](LICENSE)
[![Author](https://img.shields.io/badge/created%20by-Nated0ge-ff69b4?style=for-the-badge&logo=github&logoColor=white)](https://github.com/nathan237)

[![Watch the demo](https://img.shields.io/badge/%E2%96%B6%20Watch%20Demo-YouTube-red?style=for-the-badge&logo=youtube&logoColor=white)](https://youtu.be/RBJJi8jW1_g)

[Why "Trust"?](#-why-trustos) | [JARVIS AI](#-jarvis--on-device-ai) | [Editions](#-editions) | [Features](#-features) | [Quick Start](#-quick-start) | [Architecture](#-architecture) | [Contributing](#-contributing)

---

</div>

## Editions

TrustOS ships in **two official editions**:

| Edition | Description | ISO Size |
|---------|-------------|----------|
| **TrustOS** | Full OS with desktop, network, emulators, CyberLab, and all tools. JARVIS AI engine included but starts untrained. | ~12 MB |
| **TrustOS-JarvisPack** | Everything in TrustOS + pretrained JARVIS brain weights (4.4M-param transformer). On-device AI inference ready out of the box. | ~29 MB |

```powershell
# Build base edition
.\build-trustos.ps1

# Build AI edition (with pretrained JARVIS)
.\build-trustos-jarvispack.ps1
```

---

## JARVIS -- On-Device AI

> **The first bare-metal OS with a fully integrated, kernel-resident AI that learns, reasons, and communicates -- without any external service.**

JARVIS is not a chatbot wrapper. It is a **4.4-million-parameter byte-level transformer** running entirely inside the TrustOS kernel, at Ring 0, with zero external dependencies. No cloud. No API calls. No internet required. The AI lives in the same address space as the kernel and has direct access to hardware telemetry.

### What makes JARVIS unique

| Feature | Details |
|---------|---------|
| **Architecture** | Custom transformer: 8 attention heads, 6 layers, 512-dim embeddings, byte-level tokenizer (256 vocab) |
| **Parameters** | 4.4 million trainable weights, all in kernel memory |
| **Training** | On-device backpropagation with Adam optimizer, runs bare-metal without any ML framework |
| **Inference** | Real-time text generation from kernel space, ~80 tokens in milliseconds |
| **Federated Learning** | Nodes can sync model weights over the network -- distributed training across TrustOS instances |
| **Mesh Networking** | Raft-consensus mesh: Leader election, peer discovery, weight synchronization |
| **PXE Replication** | JARVIS can replicate itself to new nodes over PXE boot -- self-propagating AI |
| **Hardware Awareness** | `jarvis_hw` module probes CPU, memory, PCI, network -- JARVIS knows its own hardware |
| **Guardian System** | The Pact: JARVIS cannot modify OS code without authorization from its guardians |
| **SIMD Optimized** | SSE2/AVX matrix operations for fast inference on x86_64 |
| **21,000+ lines** | 25 Rust modules: model, training, inference, backprop, mesh, federated, RPC, compression, guardian, and more |

### JARVIS Commands

```
jarvis status          # Show model info, parameter count, training state
jarvis chat <prompt>   # Generate text from the transformer
jarvis train <text>    # Train on new text (on-device backprop)
jarvis eval            # Evaluate model loss on test data
jarvis tokens          # Show tokenizer info
mesh start             # Start mesh networking (peer discovery + Raft)
mesh status            # Show mesh peers and roles
federated enable       # Enable federated weight sync
pxe start              # Start PXE replication server
propagate              # Auto-propagate JARVIS to discovered nodes
```

### The Pact

JARVIS has a hard-coded guardian system ([`kernel/src/jarvis/guardian.rs`](kernel/src/jarvis/guardian.rs)). Protected operations (training, weight push, model replace, PXE replicate) require explicit authorization. The Pact is enforced at the kernel level -- it cannot be bypassed by software.

### JARVIS Technical Summary

```
kernel/src/jarvis/             # 15,900 lines across 24 files
  model.rs                     # Transformer architecture (4.4M params)
  training.rs                  # On-device backpropagation
  backprop.rs                  # Gradient computation
  inference.rs                 # Text generation
  optimizer.rs                 # Adam optimizer
  tokenizer.rs                 # Byte-level BPE tokenizer
  mesh.rs                      # Raft-consensus mesh networking
  federated.rs                 # Federated learning (weight sync)
  rpc.rs                       # Remote procedure calls between nodes
  pxe_replicator.rs            # PXE boot self-replication
  guardian.rs                  # The Pact (authorization system)
  simd.rs                      # SSE2/AVX matrix acceleration
  compression.rs               # Model weight compression
  consensus.rs                 # Raft leader election
  compute.rs                   # Distributed compute scheduler
  corpus.rs                    # Training corpus management
  mentor.rs                    # Serial-port mentor protocol
  agent.rs                     # Autonomous agent capabilities
  task.rs                      # Task execution engine
  ...

kernel/src/jarvis_hw/          # 3,500 lines -- hardware intelligence
  probe/                       # CPU, memory, PCI, network hardware probing
  hw_corpus.rs                 # Hardware-aware training data

kernel/src/shell/jarvis.rs     # 1,600 lines -- shell integration
```

---

## What's New (v0.7.0)

| Feature | Details |
|---------|---------|
| **Two Official Editions** | TrustOS (base) and TrustOS-JarvisPack (AI). Separate build pipelines and output directories. |
| **Project Reorganization** | Clean structure: `scripts/`, `tools/`, `docs/`, `builds/`, `firmware/`, `logs/`, `media/`. Root reduced from 400+ files to 11. |
| **checkm8 SecureROM Exploit** | Bare-metal xHCI USB exploit for Apple A12 (T8020) DFU mode. Full exploit chain from kernel space. |
| **Apple Hardware Drivers** | Apple Interrupt Controller (AIC) + Apple UART for native Apple silicon support. |
| **ARM64 GICv2 + Exception Vectors** | Full interrupt controller driver + exception vector table for aarch64. |
| **COSMIC2 Desktop Refresh** | Redesigned window borders, transparency, icon set, and window sizing. |

Previous highlights: [Multi-Arch v0.6.0](#v060----multi-arch--universal-boot-february-2026) | [CyberLab v0.5.0](#v050----cyberlab) | [Emulators v0.4.0](#v040----emulators--gamelab)

---

## Why "TrustOS"?

The name says it all: **Trust** + **Rust**.

In a world where your operating system is a black box -- millions of lines of legacy C/C++, binary blobs, proprietary drivers, telemetry you can't disable -- **how do you know what your computer is actually doing?**

TrustOS is the answer: **every single line is open, readable, and auditable.**

- **Fully auditable** -- 260,000 lines of Rust, all on GitHub. No binary blobs. No hidden code.
- **Memory safe by design** -- Rust's ownership model prevents entire categories of vulnerabilities.
- **Zero dependencies on C** -- no libc, no glibc, no C runtime. Every driver, every protocol, every pixel is Rust.
- **Built-in AI** -- JARVIS is not a cloud wrapper. It's a real transformer running bare-metal in kernel space.

> *"The only OS where you can trace every packet, every pixel, every thought of the AI, and every keystroke back to its source code."*

| Metric | Value |
|--------|-------|
| **Total code** | 260,000+ lines of Rust |
| **Source files** | 441 `.rs` files |
| **Architectures** | x86_64, aarch64 (ARM64), riscv64 (RISC-V) |
| **JARVIS AI** | 4.4M-param transformer, 21,000 lines, 25 modules |
| **Boot time** | < 1 second |
| **Desktop FPS** | 144 FPS (SSE2 SIMD) |
| **Auto-tests** | 95/96 passing (99%) |
| **C code** | 0 lines |
| **External ML frameworks** | 0 (transformer from scratch) |

### TrustOS vs The World

| | Traditional OS | TrustOS |
|---|:---:|:---:|
| **Language** | C/C++ with 40 years of memory bugs | 100% Rust -- memory safe by design |
| **Codebase** | Millions of lines, impossible to audit | 260K lines, one person can read it all |
| **Built-in AI** | None | 4.4M-param transformer in kernel space |
| **Platforms** | Tied to one architecture | x86_64 + ARM64 + RISC-V from one codebase |
| **Binary blobs** | Everywhere | None. Zero. |
| **Telemetry** | Opt-out (maybe) | Doesn't exist -- verify it yourself |

---

## Run on Anything

TrustOS boots on **PCs, phones, single-board computers, and VMs** -- from a single codebase.

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

---

## Features

### TrustLab -- Real-Time Kernel Introspection

> **World's first: no other bare-metal OS has a built-in, real-time kernel introspection lab.**

7-panel interactive workspace: Hardware Status, Kernel Trace, Command Guide, File System Tree, TrustLang Editor, Execution Pipeline, Hex Editor. Full mouse interaction, zero-cost 512-slot trace bus. Launch: `trustlab` or from desktop.

### COSMIC2 Desktop Environment
- **Multi-layer GPU compositor** with 8 rendering layers, SSE2 SIMD, 144 FPS
- **Taskbar, dock, start menu**, window management, settings panel
- **14+ desktop apps**: Terminal, Files, TrustCode, Calculator, Network, Snake, Chess 3D, TrustBrowser, TrustEdit 3D...
- **Touch & gesture** input for mobile/tablet deployment

### Network Stack (from scratch)
- **VirtIO-net**, **TCP/IP** (ARP, DHCP, DNS, TCP, UDP, ICMP), **IPv6 + ICMPv6**
- **TLS 1.3** -- full handshake, X.509 certs, crypto from scratch
- **HTTP/HTTPS client** + **HTTP server** (`httpd start/stop/status`)
- **Live tested against google.com** -- nmap, curl, ping, DNS all verified on real internet

### TrustScan -- Network Security Toolkit
Port scanner, packet sniffer, banner grabber, host discovery, traceroute, vulnerability scanner.

### Game Boy Color Emulator + GameLab
Full CGB emulator (LR35902 CPU, all 501 opcodes, scanline PPU, MBC1/3/5) + 2,000-line real-time analysis dashboard.

### NES Emulator
MOS 6502 CPU (151 official + 8 unofficial opcodes), 2C02 PPU, mappers 0-3.

### Built-in Apps & Tools

| Category | Highlights |
|----------|-----------|
| **Browser** | HTML/CSS parser, JS engine, HTTPS, TLS 1.3 |
| **Code Editor** | TrustCode: Rust syntax highlighting, line numbers, file save/load |
| **Language** | TrustLang: Lexer, Parser, Compiler, Bytecode VM |
| **3D Engine** | Formula3D wireframe renderer, TrustEdit 3D model editor |
| **Chess** | Full 3D chess with AI (minimax depth 2), look-at camera |
| **Audio** | 8-voice polyphonic synthesizer, ADSR, pattern sequencer |
| **JARVIS AI** | Built-in transformer for on-device inference and learning |

### Hypervisor & Linux Compatibility
- **Intel VT-x** and **AMD-V** dual-backend with EPT/NPT
- **70+ Linux syscalls**, ELF64 loader, Ring 3 userland, PTY/TTY, job control

### Security
- **Ed25519 signatures** (RFC 8032) for kernel authentication
- **Capability-based** security model
- **JARVIS Guardian** -- hard-coded authorization for AI operations

---

## Quick Start

### Option A: Download the ISO

Grab the latest ISO from [**Releases**](https://github.com/nathan237/TrustOS/releases):

| Edition | Description |
|---------|-------------|
| `trustos.iso` | Base OS -- everything except pretrained AI |
| `trustos-jarvispack.iso` | Full OS + pretrained JARVIS brain |

```bash
# Run in QEMU
qemu-system-x86_64 -cdrom trustos.iso -m 512M -cpu max -smp 4 -display gtk -vga std -serial stdio
```

### Option B: Build from source

```powershell
git clone https://github.com/nathan237/TrustOS.git
cd TrustOS

# Base edition
.\build-trustos.ps1

# AI edition (with pretrained JARVIS)
.\build-trustos-jarvispack.ps1
```

### First commands to try

| Command | What it does |
|---------|-------------|
| `desktop` | Launch COSMIC2 desktop |
| `jarvis status` | Check JARVIS AI status |
| `jarvis chat hello` | Chat with the on-device AI |
| `showcase` | Automated feature tour |
| `trustlab` | Kernel introspection lab |
| `neofetch` | System info |
| `chess3d` | 3D chess vs AI |
| `gameboy` | Game Boy Color emulator |
| `netscan scan` | Network port scanner |
| `mesh start` | Start JARVIS mesh network |
| `help` | All 200+ commands |

---

## Project Structure

```
TrustOS/
  kernel/                   # Core bare-metal kernel (260K+ lines)
    src/
      jarvis/               # JARVIS AI (15,900 lines, 24 files)
      jarvis_hw/            # Hardware intelligence (3,500 lines)
      shell/                # 200+ commands + JARVIS integration
      desktop.rs            # COSMIC2 desktop manager
      network/              # TCP/IP, TLS 1.3, DHCP, DNS
      browser/              # HTML/CSS/JS browser engine
      gameboy/              # Game Boy Color emulator
      nes/                  # NES emulator
      hypervisor/           # VT-x/SVM, EPT/NPT
      vfs/                  # TrustFS, FAT32, procfs
      tls13/                # TLS 1.3, crypto, X.509
      netscan/              # Network security toolkit
      drivers/              # AHCI, USB, VirtIO, Apple, checkm8
      ...
  userland/                 # Userspace programs (init, shell, fs, jarvis...)
  scripts/
    build/                  # Build scripts (limine, multiarch)
    launch/                 # VM launch scripts (QEMU, VBox)
    test/                   # Test automation
    debug/                  # Serial debug tools
    jarvis/                 # AI training & demo scripts
  tools/                    # Python utilities, thumbnails, training
  docs/                     # Documentation, roadmaps, guides
  builds/
    trustos/                # Base edition ISO output
    trustos-jarvispack/     # JarvisPack edition ISO output
  firmware/                 # UEFI firmware files (OVMF)
  media/                    # Screenshots and recordings
  logs/                     # Build, test, and debug logs
  limine/                   # Bootloader binaries
  apple/                    # iOS security research tools
  sdk/                      # Cross-compilation SDK
```

---

## Documentation

A comprehensive usage guide covering all 200+ commands, desktop features, JARVIS AI, networking, emulators, and development workflows is maintained locally. If you want a copy, open an issue or contact the author.

| Document | Location |
|----------|----------|
| Developer Guide | [`docs/DEVELOPER_GUIDE.md`](docs/DEVELOPER_GUIDE.md) |
| Contributing | [`docs/CONTRIBUTING.md`](docs/CONTRIBUTING.md) |
| Roadmap | [`docs/ROADMAP_V2.md`](docs/ROADMAP_V2.md) |
| Release Notes | [`docs/RELEASE_NOTES.md`](docs/RELEASE_NOTES.md) |
| JARVIS Guardian Pact | [`kernel/src/jarvis/guardian.rs`](kernel/src/jarvis/guardian.rs) |

---

## Changelog

### v0.7.0 -- checkm8 & JARVIS (March 2026)

- **Two official editions**: TrustOS (base, ~12 MB) and TrustOS-JarvisPack (AI, ~29 MB)
- **Project reorganization**: Root cleaned from 400+ files to 11. Scripts, tools, docs, logs, media organized into dedicated directories.
- **JARVIS AI**: 4.4M-parameter byte-level transformer with on-device training, federated learning, mesh networking, PXE replication, and Guardian authorization system. 21,000+ lines across 25 modules.
- **checkm8 SecureROM exploit** -- Bare-metal xHCI USB exploit for Apple A12 DFU mode.
- **Apple hardware drivers** -- AIC + UART for native Apple silicon support.
- **ARM64 GICv2** -- Full interrupt controller driver + exception vectors for aarch64.
- **COSMIC2 Desktop refresh** -- Redesigned windows, borders, transparency, icons, default sizes.
- 260,000+ lines, 441 source files, 3 architectures.

### v0.6.0 -- Multi-Arch & Universal Boot (February 2026)

- **Multi-Architecture**: x86_64, aarch64, riscv64 from one codebase.
- **Android Boot**: boot.img v2 pipeline, `fastboot flash boot`.
- **Raspberry Pi SD**: Bare-metal RPi 4/5.
- **Universal Installer**: 9 targets, one script.
- **Touch & Gesture Input**: Multi-touch + gesture recognition.

### v0.5.0 -- CyberLab (February 2026)

- **Live network scan verified on google.com** -- 11/11 tests passed.
- **Code optimization** (-2,800 lines).
- 95/96 auto-tests (99%).

### v0.4.x -- Emulators & GameLab (February 2026)

- **Game Boy Color emulator** -- Full CGB: LR35902, scanline PPU, MBC1/3/5
- **GameLab** -- 2,000-line analysis dashboard
- **NES emulator** -- 6502 CPU, 2C02 PPU, mappers 0-3
- **Shell scripting**, **HTTP server**, **TrustPkg**, **TrustScan**, **IPv6**

### v0.3.x -- Foundation (February 2026)

- ACPI, PIC, PIT, RTC, PTY/TTY, job control, NVMe swap, SMP

### v0.2.0 -- Userspace (February 2026)

- Ring 3 execution, ELF64 loader, TrustFS

### v0.1.x -- Initial Development (February 2026)

- TrustLab, COSMIC2 desktop, Ed25519, 3D Chess, audio, web sandbox

---

## Contributing

Contributions are welcome. TrustOS is designed to be **readable and hackable**.

> See [`docs/CONTRIBUTING.md`](docs/CONTRIBUTING.md) for the complete guide.

```bash
git clone https://github.com/YOUR_USERNAME/TrustOS.git
git checkout -b feature/my-feature
cargo build --release -p trustos_kernel
# Test in QEMU, then open a Pull Request
```

---

## License

Apache License 2.0 -- see [LICENSE](LICENSE).

---

## Author

**Nated0ge** -- Sole creator & developer of TrustOS

- GitHub: [@nathan237](https://github.com/nathan237)
- Project: [TrustOS](https://github.com/nathan237/TrustOS)

> 260,000+ lines of Rust. 4.4M-param on-device AI. 3 architectures. Zero C. Fully auditable.

---

<div align="center">

**Trust** the code. **Rust** is the reason.

Created by [Nated0ge](https://github.com/nathan237)

260,000+ lines | JARVIS AI (4.4M params) | x86_64 + ARM64 + RISC-V | Zero C | Fully auditable

[Report Bug](https://github.com/nathan237/TrustOS/issues) | [Request Feature](https://github.com/nathan237/TrustOS/issues) | [Watch Demo](https://youtu.be/RBJJi8jW1_g)

</div>
