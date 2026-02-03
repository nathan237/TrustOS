# 🦀 TrustOS - Secure Experimental Kernel in Rust

<div align="center">

**A bare-metal x86_64 operating system written entirely in Rust**

[![Build Status](https://img.shields.io/badge/build-passing-brightgreen)]()
[![Rust](https://img.shields.io/badge/rust-nightly-orange)]()
[![License](https://img.shields.io/badge/license-MIT-blue)]()
[![Platform](https://img.shields.io/badge/platform-x86__64-lightgrey)]()

[Features](#-features) • [Quick Start](#-quick-start) • [Screenshots](#-screenshots) • [Roadmap](#-roadmap)

</div>

---

## 🎯 What is TrustOS?

TrustOS is an **experimental operating system** built from scratch in Rust, designed to explore:
- **Memory safety** at the kernel level (no C, no undefined behavior)
- **Modern OS design** with microkernel-inspired architecture
- **Linux compatibility** through syscall emulation
- **AI-assisted development** with built-in Jarvis assistant

### Why TrustOS?

| Traditional OS | TrustOS |
|---------------|---------|
| C/C++ with memory bugs | 100% Rust - memory safe by design |
| Legacy compatibility debt | Clean slate, modern design |
| Complex build systems | Simple `cargo build` |
| Monolithic kernels | Modular, microkernel-inspired |

---

## ✨ Features

### 🖥️ Core Kernel
- **UEFI boot** via Limine bootloader
- **Memory management** with heap allocator (512MB+)
- **Preemptive multitasking** with cooperative scheduler
- **Virtual filesystem** (VFS) with ramfs, procfs, devfs
- **Process isolation** with Ring 0/3 support

### 🐧 Linux Compatibility
- **100+ syscalls** emulated (read, write, mmap, fork, exec...)
- **ELF binary execution** - run Linux binaries directly
- **Alpine Linux subsystem** - install packages with `apk`
- **Binary-to-Rust transpiler** - analyze and convert Linux binaries

### 🌐 Network Stack
- **VirtIO-net driver** with full networking
- **TCP/IP stack** with DHCP, DNS, ARP
- **HTTP/HTTPS** client (TLS 1.3 support)
- **curl/wget** commands built-in

### 🖼️ Graphics
- **Framebuffer console** with 1280x800 resolution
- **GUI desktop** (Windows 11-inspired design)
- **Compositor** with window management
- **TrustGL** - OpenGL-like software renderer

### 🛡️ Security
- **Capability-based security** model
- **User authentication** (login, su, passwd)
- **File permissions** (chmod, chown)

---

## 🚀 Quick Start

### Prerequisites
- Rust nightly toolchain
- QEMU (for x86_64 emulation)
- OVMF (UEFI firmware)

### Build & Run

```bash
# Clone the repository
git clone https://github.com/YOUR_USERNAME/TrustOS.git
cd TrustOS

# Build the kernel
cargo build --release

# Run in QEMU (Windows)
.\run-qemu.ps1

# Run in QEMU (Linux)
./run-qemu.sh
```

### Or download a pre-built release

```bash
# Download from releases and extract
unzip TrustOS-v0.1.0.zip -d trustos

# Run with QEMU
qemu-system-x86_64 \
  -bios OVMF.fd \
  -drive format=raw,file=fat:rw:trustos \
  -m 512M \
  -device virtio-net-pci,netdev=net0 \
  -netdev user,id=net0
```

---

## 📸 Demo

### Shell with Autocomplete
```
  _____ ____            _    ___      
 |_   _|  _ \ _   _ ___| |_ / _ \ ___ 
   | | | |_) | | | / __| __| | | / __|
   | | |  _ <| |_| \__ \ |_| |_| \__ \
   |_| |_| \_\\__,_|___/\__|\___/|___/

  T-RustOs v0.1.0 - Type 'help' for commands

[14:32:15] trustos:/$ help
```

### Linux Subsystem
```
[14:32:20] trustos:/$ linux shell
alpine:/# apk add curl
alpine:/# curl https://example.com
```

### GUI Desktop
```
[14:32:25] trustos:/$ gui
→ Launches graphical desktop with window management
```

---

## 📋 Available Commands

### 📁 File System (18 commands)
`ls` `cd` `pwd` `mkdir` `rmdir` `touch` `rm` `cp` `mv` `cat` `head` `tail` `stat` `tree` `find` `wc` `grep` `ln`

### 👤 User Management (9 commands)
`login` `logout` `su` `passwd` `adduser` `deluser` `users` `whoami` `id`

### ⚙️ System (12 commands)
`clear` `time` `date` `hostname` `env` `history` `uname` `free` `df` `ps` `top` `dmesg`

### 🌐 Network (10 commands)
`ifconfig` `ping` `curl` `wget` `nslookup` `arp` `route` `netstat` `traceroute` `browse`

### 🐧 Linux Subsystem (5 commands)
`linux shell` `linux extract` `linux exec` `alpine` `transpile`

---

## 🗺️ Roadmap

```
✅ Phase 0 - MVP Kernel (Complete)
   └─ Boot, memory, interrupts, scheduler, IPC, security

✅ Phase 1 - Core Userland (Complete)  
   └─ Shell, VFS, network stack, POSIX syscalls

✅ Phase 2 - Developer OS + AI (Complete)
   └─ Linux subsystem, Jarvis AI, transpiler

🔄 Phase 3 - GUI & UX (In Progress)
   └─ Wayland compositor, native apps, GPU acceleration

🔮 Phase 4 - Production Ready
   └─ Multi-core SMP, package manager, self-hosting
```

---

## 🏗️ Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    User Applications                        │
│   (Native Rust apps, Linux binaries, GUI applications)     │
├─────────────────────────────────────────────────────────────┤
│                      Shell / Desktop                        │
├─────────────────────────────────────────────────────────────┤
│     VFS      │   Network   │   Linux    │   Graphics       │
│   (ramfs,    │   (TCP/IP,  │ Subsystem  │  (Compositor,    │
│   procfs)    │   DHCP)     │  (syscalls)│   TrustGL)       │
├─────────────────────────────────────────────────────────────┤
│                    TrustOS Kernel                           │
│   Memory │ Scheduler │ IPC │ Security │ Drivers │ Syscalls │
├─────────────────────────────────────────────────────────────┤
│            Hardware (x86_64, UEFI, VirtIO)                  │
└─────────────────────────────────────────────────────────────┘
```

---

## 🤝 Contributing

Contributions are welcome!

```bash
# Fork the repo, then:
git checkout -b feature/amazing-feature
git commit -m 'Add amazing feature'
git push origin feature/amazing-feature
# Open a Pull Request
```

---

## 📄 License

This project is licensed under the MIT License.

---

## 🙏 Acknowledgments

- [Limine](https://github.com/limine-bootloader/limine) - Bootloader
- [Rust OSDev](https://os.phil-opp.com/) - Inspiration and learning
- [Alpine Linux](https://alpinelinux.org/) - Lightweight Linux for subsystem

---

<div align="center">

**Built with 🦀 Rust and ❤️**

*TrustOS - Because your kernel should be as safe as your code*

</div>
