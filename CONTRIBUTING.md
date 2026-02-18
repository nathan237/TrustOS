# Contributing to TrustOS

Thank you for your interest in contributing to TrustOS! This guide will help you get started.

## Table of Contents

- [Getting Started](#getting-started)
- [Build Requirements](#build-requirements)
- [Building TrustOS](#building-trustos)
- [Running in QEMU](#running-in-qemu)
- [Architecture Overview](#architecture-overview)
- [Adding a Shell Command](#adding-a-shell-command)
- [Adding a Driver](#adding-a-driver)
- [Adding a Syscall](#adding-a-syscall)
- [Integration Tests](#integration-tests)
- [Code Style](#code-style)
- [Pull Request Process](#pull-request-process)

## Getting Started

1. Fork the repository
2. Clone your fork: `git clone https://github.com/<you>/OSrust.git`
3. Create a feature branch: `git checkout -b feature/my-feature`
4. Make your changes
5. Test with `cargo build --release` and QEMU
6. Submit a pull request

## Build Requirements

| Tool | Version | Purpose |
|------|---------|---------|
| Rust nightly | See `rust-toolchain.toml` | Kernel compiler |
| `x86_64-unknown-none` target | — | Bare-metal target |
| QEMU | 7.0+ | Testing/emulation |
| OVMF | — | UEFI firmware for QEMU |
| Limine | 7.x | Bootloader |

### Quick Setup (Windows)

```powershell
# Install Rust nightly with the bare-metal target
rustup toolchain install nightly
rustup target add x86_64-unknown-none --toolchain nightly

# Build
cargo build --release

# Run in QEMU (after building ISO)
.\run-qemu.ps1
```

### Quick Setup (Linux)

```bash
# Install Rust nightly
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup toolchain install nightly
rustup target add x86_64-unknown-none

# Build
cargo build --release

# Run in QEMU
./build.sh && qemu-system-x86_64 ...
```

## Building TrustOS

```bash
# Standard release build (required — debug builds are too slow for a kernel)
cargo build --release

# The kernel binary is output to:
#   target/x86_64-unknown-none/release/kernel
```

The build uses a custom target (`x86_64-unknown-none`) defined in `rust-toolchain.toml`. Key compiler flags:
- `#![no_std]` — no standard library
- `#![no_main]` — custom entry point via Limine bootloader
- `#![feature(abi_x86_interrupt)]` — hardware interrupt handlers
- `extern crate alloc` — heap allocation via custom allocator

## Running in QEMU

```powershell
# GUI mode with networking
.\run-qemu-gui.ps1

# Serial-only mode (for CI/testing)
.\run-trustos-serial.ps1

# With network testing
.\run-trustos-network-test.ps1
```

QEMU flags used:
- `-machine q35` — modern chipset with PCIe
- `-cpu host -enable-kvm` (Linux) or `-accel whpx` (Windows) — hardware acceleration
- `-smp 4` — 4 CPU cores (SMP)
- `-m 512M` — 512 MB RAM
- `-device virtio-net-pci` — network card
- `-bios OVMF.fd` — UEFI firmware

## Architecture Overview

```
kernel/src/
├── main.rs              # Entry point, module declarations
├── serial.rs            # Serial port (COM1) output
├── framebuffer.rs       # VESA framebuffer + text console
├── keyboard.rs          # PS/2 keyboard driver
├── memory/              # Physical frame allocator + heap
├── interrupts.rs        # IDT, exception handlers
├── shell/               # Interactive shell
│   ├── mod.rs           #   Command dispatcher, pipes, redirects
│   ├── commands.rs      #   Core commands (ls, cat, help, inttest)
│   ├── unix.rs          #   POSIX utilities (sort, grep, export)
│   ├── vm.rs            #   VM, networking, hardware commands
│   ├── desktop.rs       #   GUI desktop commands
│   ├── network.rs       #   Browser, sandbox, container
│   ├── apps.rs          #   TrustLang, video, lab
│   ├── scripting.rs     #   Shell scripting engine (variables, if/for)
│   └── jarvis.rs        #   AI assistant
├── netstack/            # TCP/IP stack
│   ├── mod.rs           #   Ethernet frame processing
│   ├── arp.rs           #   ARP resolution
│   ├── ip.rs            #   IPv4 layer
│   ├── tcp.rs           #   TCP (full RFC 793 state machine)
│   ├── udp.rs           #   UDP
│   ├── dhcp.rs          #   DHCP client
│   ├── dns.rs           #   DNS resolver
│   ├── socket.rs        #   BSD socket API
│   ├── icmp.rs          #   ICMP (ping)
│   └── tls.rs           #   TLS 1.3 integration
├── netscan/             # Security toolkit
│   ├── port_scanner.rs  #   TCP SYN/connect port scanner
│   ├── sniffer.rs       #   Packet capture & analysis
│   ├── banner.rs        #   Service banner grabbing
│   ├── discovery.rs     #   Network host discovery
│   ├── traceroute.rs    #   ICMP traceroute
│   └── vuln.rs          #   Vulnerability scanner
├── drivers/             # Hardware drivers
│   ├── net/             #   Virtio-net, E1000, RTL8139
│   ├── usb.rs           #   USB (xHCI, mass storage)
│   └── gpu.rs           #   Virtual GPU emulation
├── hypervisor/          # Type-1 hypervisor
│   ├── vmx.rs           #   Intel VT-x
│   ├── svm.rs           #   AMD SVM
│   └── vmm.rs           #   Virtual machine monitor
├── cpu/                 # CPU management
│   ├── smp.rs           #   Symmetric multiprocessing
│   └── features.rs      #   CPUID, feature detection
├── tls13/               # TLS 1.3 implementation
├── httpd.rs             # HTTP server
├── trustpkg.rs          # Package manager
├── desktop.rs           # COSMIC desktop environment
├── trustlang.rs         # TrustLang programming language
├── nes.rs               # NES emulator
├── gameboy.rs           # Game Boy emulator
└── mario64.rs           # Mario 64 3D game
```

## Adding a Shell Command

### 1. Implement the command function

Add your function to the appropriate submodule in `kernel/src/shell/`:

```rust
// In kernel/src/shell/commands.rs (or unix.rs, vm.rs, etc.)
pub(super) fn cmd_mycommand(args: &[&str]) {
    if args.is_empty() {
        crate::println!("Usage: mycommand <arg>");
        return;
    }
    crate::println!("Hello from mycommand: {}", args[0]);
}
```

### 2. Register in the dispatch table

In `kernel/src/shell/mod.rs`, add the command to the `match` in `execute_single()`:

```rust
// In the big match statement in execute_single():
"mycommand" => commands::cmd_mycommand(args),
```

### 3. Add help text

In `cmd_help()` in `commands.rs`, add a line describing your command:

```rust
crate::println!("    mycommand <arg>     Description of what it does");
```

### 4. Add a man page entry

In `cmd_man()`, add a detailed help block:

```rust
"mycommand" => {
    crate::println!("NAME: mycommand — short description");
    crate::println!("USAGE: mycommand [options] <args>");
    crate::println!("OPTIONS:");
    crate::println!("  -v    Verbose output");
}
```

## Adding a Driver

1. Create `kernel/src/drivers/mydevice.rs`
2. Implement PCI device detection in `init()`:
   ```rust
   pub fn init(pci_devices: &[PciDevice]) {
       for dev in pci_devices {
           if dev.vendor_id == 0x1234 && dev.device_id == 0x5678 {
               // Initialize device
           }
       }
   }
   ```
3. Register the module in `main.rs`: `mod mydevice;`
4. Call `mydevice::init()` from the kernel initialization sequence

## Adding a Syscall

1. Add the syscall number in `kernel/src/syscall.rs`:
   ```rust
   const SYS_MYCALL: u64 = 500;
   ```
2. Add the handler in the syscall dispatch:
   ```rust
   SYS_MYCALL => handle_mycall(arg1, arg2),
   ```
3. Implement the handler function

## Integration Tests

TrustOS uses in-kernel integration tests run via the `inttest` shell command.

### Adding a test

In `kernel/src/shell/commands.rs`, inside `cmd_inttest()`:

```rust
// [N/30] My New Test
crate::println_color!(COLOR_CYAN, "[N/30] My Test Description");
{
    crate::print!("  Testing something... ");
    if some_condition {
        crate::println_color!(COLOR_GREEN, "[OK]");
        passed += 1;
    } else {
        crate::println_color!(COLOR_RED, "[FAIL]");
        failed += 1;
    }
}
```

### Running tests

```
TrustOS> inttest        # Run all integration tests
TrustOS> debugnew       # Run new feature tests
TrustOS> test           # Quick smoke test
```

### CI testing

Tests can be run automatically via QEMU with serial output:
```bash
qemu-system-x86_64 -serial stdio -nographic ... | grep "ALL.*PASSED"
```

## Code Style

- **No `std`**: Everything is `no_std` + `alloc`. No `std::` imports.
- **No `unsafe` without justification**: Document every `unsafe` block.
- **Use `crate::println!()` for output**: This dispatches to framebuffer or capture buffer.
- **Color output**: Use `crate::println_color!(COLOR_GREEN, "msg")` for colored text.
- **Error handling**: Return `Result` where possible. Use `Option` for lookups.
- **Module organization**: Keep shell commands in `shell/`, drivers in `drivers/`, networking in `netstack/`.
- **Documentation**: Add `//!` module doc comments and `///` function doc comments.

## Pull Request Process

1. Ensure `cargo build --release` passes with no errors
2. Run `inttest` in QEMU and verify all tests pass
3. Add integration tests for new features
4. Update help text and man pages
5. Add doc comments to public functions
6. Keep commits focused and well-described
7. Reference any related issues in the PR description

## License

By contributing, you agree that your contributions will be licensed under the same license as the project (see LICENSE file).
