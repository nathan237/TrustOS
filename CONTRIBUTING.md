# Contributing to TrustOS

Thanks for considering a contribution. TrustOS is a solo-maintained bare-metal OS, so contributions are welcome but reviewed strictly.

## Ground rules

TrustOS is `#![no_std]` everywhere with `panic = "abort"`. The following are **non-negotiable** in kernel code:

- ❌ No `unwrap()` or `expect()` — use `if let`, `match`, or `.unwrap_or(...)`.
- ❌ No `std::println!` — use `serial_println!` for COM1 / `crate::println!` for the framebuffer.
- ❌ No `std`, no `alloc` outside post-init code paths.
- ✅ Every `unsafe` block must justify its invariants in a comment.
- ✅ MMIO accesses use `read_volatile` / `write_volatile`.
- ✅ Bounds-check every external-data path.

## Toolchain

- Rust **nightly** (see `rust-toolchain.toml`).
- Components: `rust-src`, `llvm-tools-preview`.
- Target: `x86_64-unknown-none` (primary), `aarch64-unknown-none` (experimental).

## Build

```powershell
cargo build --release -p trustos_kernel
.\scripts\build\build-trustos.ps1              # kernel + ISO + VM
.\scripts\build\build-trustos.ps1 -NoRun       # kernel + ISO seulement
.\scripts\trustos-hub.ps1                      # hub — point d'entrée principal
```

Linux/macOS:
```bash
make build && make iso && make run
```

## Testing on real hardware

Most of TrustOS is validated by booting on real boards (ThinkPad T61, BTC-250PRO + RX 580X). PR descriptions should mention what was tested and how (QEMU, VirtualBox, PXE boot, USB).

## Pull request checklist

- [ ] `cargo build --release -p trustos_kernel` succeeds with no warnings.
- [ ] Touched feature still boots in QEMU.
- [ ] No `unwrap()` / `println!` regressions.
- [ ] Public APIs documented.
- [ ] Commit messages follow `type(scope): subject` (e.g. `feat(amdgpu): wire SDMA doorbell`).

## Code of conduct

Be civil. Disagreement is fine, hostility is not.
