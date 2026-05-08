# TrustOS — Public Roadmap

> Living document. Dates are intent, not promises. Solo project, real hardware first.

## Now (Q2 2026)

- **GPU AMD Polaris** — full bring-up on RX 580X (BTC-250PRO board)
  - SDMA / CP / GMC / VM stable
  - User-visible `gpu` shell with diag, dump, regwatch
- **Audio Edition v0.12** — shipped: Intel HDA, TrustSynth, TrustDAW, Beat Studio
- **Repo hygiene** — feature gating clean, public ISO builds without `jarvis`

## Next (Q3 2026)

- **TrustWall** — opinionated firewall + network monitoring on bare metal
- **NTFS write support** (Sprint 2: journal + dirty bitmap)
- **Mining Edition** — dedicated build for BTC-250PRO class boards
- **GPU compute** — first wavefront dispatched on Polaris (compute pipeline)
- **CI smoke test** — boot kernel in QEMU headless, assert serial markers

## Later (Q4 2026 → 2027)

- **ARM64 / RISC-V** — beyond stub: real boot path on at least one board each
- **Wayland-ish compositor** — internal protocol, not Linux Wayland compat
- **JARVIS public release** — federated mode, on-device inference, opt-in
- **Robotics edition** — real-time scheduling, sensor stack, motor control
- **Self-hosting** — TrustOS able to rebuild itself

## Principles (won't change)

- **100% Rust, `no_std`** — zero C, zero binary blobs in kernel
- **One developer, one vision** — no committee, no design by consensus
- **Real hardware first** — every feature must boot on metal, not just QEMU
- **Transparent silicon** — every PCI device, MSR, ACPI table readable

## Not on the roadmap

- POSIX compatibility layer
- Linux ABI emulation
- Userland Wine / Windows compatibility
- Server / cloud edition
