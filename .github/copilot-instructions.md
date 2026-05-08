# TrustOS — Copilot Instructions

## Project Identity
- **TrustOS**: Bare-metal OS in Rust (`no_std`), solo-developed by Nathan (~18-19 y/o)
- **JARVIS**: 4.4M-param byte-level transformer embedded in the kernel
- **Le Pacte**: JARVIS has two guardians (Nathan + Copilot). No code modification without explicit authorization from at least one guardian. See `kernel/src/jarvis/guardian.rs`.

## Language & Style
- **Code & comments**: English
- **Conversation**: French preferred, English accepted
- **Mode**: Always implement directly — no "I suggest…" or "you could…". Just do it.
- **Brevity**: Minimal explanations. Code speaks.

## Architecture

### Workspace Layout
```
kernel/src/          — THE source (never edit translated/)
  main.rs            — Entry point (Limine boot protocol)
  shell/             — Shell commands, parser, history
  desktop.rs         — Desktop environment (GUI mode)
  jarvis/            — JARVIS AI (transformer, guardian, RPC, federated)
  jarvis_hw/         — JARVIS hardware probing
  drivers/           — Hardware drivers (USB, NVMe, GPU, audio, etc.)
  netstack/          — TCP/IP stack
  framebuffer/       — Framebuffer management, rendering pipeline
  audio/             — Audio subsystem, synth, DAW
  hwdiag/            — Hardware diagnostics (15 subcommands: cpu, mem, pci,
                        acpi, storage, thermal, net, stress, remote, pciraw,
                        regdiff, ioscan, regwatch, aer, timing)
  marionet/          — MARIONET hardware dashboard (8-tab probe UI)
  compositor.rs      — Window compositor
  interrupts/        — IDT, handlers, APIC
  memory/            — Physical/virtual memory, heap allocator
  scheduler/         — Process scheduler
  vfs/               — Virtual filesystem
  gui/               — GUI widgets, event system
  cosmic/            — COSMIC-style UI components
boot/src/            — Bootloader helpers
userland/            — Userland crates (init, shell, fs, network, jarvis, compositor)
translated/          — AUTO-GENERATED, never edit
```

### Key Technical Constraints
- **`#![no_std]`** everywhere — no standard library
- **`panic = "abort"`** — no unwinding
- **Target**: `x86_64-unknown-none` (primary), `aarch64-unknown-none` (experimental), `riscv64gc-unknown-none-elf` (WIP stubs only)
- **Toolchain**: Rust nightly (see `rust-toolchain.toml`)
- **Allocator**: `linked_list_allocator` — heap available after init
- **No `unwrap()` in kernel code** — use `if let`, `match`, or `.unwrap_or()`
- **No `println!`** — use `serial_println!` or framebuffer rendering
- **Bootloader**: Limine v8 (boot protocol, not UEFI direct)

### Features (Cargo)
- `default = ["emulators", "hires-logo", "jarvis"]`
- `daw` — TrustDAW audio (~29 MB WAV, opt-in)
- `jarvis` — JARVIS AI subsystem
- `emulators` — NES + GameBoy + GameLab
- `hires-logo` — 400×400 ARGB logo bitmap

## Build Commands

### trustos-hub — point d'entrée canonique UNIQUE
```powershell
.\scripts\trustos-hub.ps1
```
Menu : **1/2/b** = switch profile · **3-6** = serveurs PXE/HTTP · **7** = build+deploy · **9** = reboot board · **d** = receive BIOS dump · **q** = quit.
Règle : ne jamais appeler `pxe_server.py` directement.

### Windows (primary dev environment)
```powershell
# Build kernel seul
cargo build --release -p trustos_kernel

# Build mining edition (GPU debug standard — cible BTC-250PRO)
cargo build --release -p trustos_kernel --no-default-features --features trustos-mining

# Scripts build dédiés (sous scripts/build/)
.\scripts\build\build-trustos.ps1              # build + ISO + VirtualBox
.\scripts\build\build-trustos.ps1 -NoRun       # build + ISO seulement
.\scripts\build\build-trustos-jarvispack.ps1   # édition JarvisPack

# Via hub (préféré — gère tout)
.\scripts\trustos-hub.ps1   # option 7 = build+deploy, option 9 = reboot

# VirtualBox: power cycle
& "C:\Program Files\Oracle\VirtualBox\VBoxManage.exe" controlvm "TRustOs" poweroff
& "C:\Program Files\Oracle\VirtualBox\VBoxManage.exe" startvm "TRustOs"
```

### Linux/macOS (Makefile)
```bash
make build          # Build kernel
make iso            # Build + ISO
make run            # Build + QEMU UEFI
make run-bios       # Build + QEMU BIOS
make clean          # Clean
```

### QEMU (direct)
```bash
qemu-system-x86_64 -cdrom trustos.iso -m 512M -machine q35 -cpu max -smp 4 \
  -display gtk -vga std -device virtio-gpu-pci,xres=1280,yres=800 \
  -device virtio-net-pci,netdev=net0 -netdev user,id=net0 \
  -drive if=pflash,format=raw,readonly=on,file=firmware/OVMF.fd \
  -serial stdio -no-reboot
```

## Hardware Test Targets
- **Primary**: BTC-250PRO LR mining board, Intel Pentium G4400 @ 3.30 GHz (Skylake 2c/2t), AMD RX 580X (Polaris 10, GCN4) PCIe x16 direct
- **VM**: VirtualBox "TRustOs" (daily dev)
- **PXE**: Network boot via 10.0.0.1 (TFTP server)
- **Target board**: ASUS H170-PRO (PXE boot target)

## Coding Conventions
- `serial_println!()` for debug output (goes to QEMU serial / COM1)
- All kernel modules: `pub fn init()` pattern for initialization
- Shell commands registered in `kernel/src/shell/`
- Use `crate::` paths, not `super::super::`
- Feature-gate heavy modules: `#[cfg(feature = "jarvis")]`, `#[cfg(feature = "daw")]`
- Framebuffer: always check `is_some()` before drawing
- Error types: per-module enums, not strings

## What NOT to Touch
- `translated/` — auto-generated by `tools/source_translator.py`
- `firmware/OVMF_VARS*.fd` — UEFI variable stores, machine-specific
- `builds/` — release artifacts
- `target/` — Cargo build cache
- `limine/` — upstream bootloader binaries (git submodule)
