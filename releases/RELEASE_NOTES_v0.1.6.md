# TrustOS v0.1.6 Release Notes

**Release Date:** 2026-02-12

## 🎬 Highlight: TrustOS Film

This release features a **built-in cinematic animated explainer** — a 2-minute film that runs entirely inside the OS, rendered in real-time on the framebuffer.

Type `film` in the TrustOS shell to watch.

### 12 Scene-Specific Animations
| # | Text | Animation |
|---|------|-----------|
| 1 | "You use a computer every day" | Floating windows (app rectangles bouncing) |
| 2 | "Do you really know what it does?" | Question marks rain (accelerating) |
| 3 | "The honest answer... is no." | Screen shatter (fracture lines from center) |
| 4 | "It controls EVERYTHING" | Binary flood (0s and 1s cascading) |
| 5 | "Nobody knows what's inside" | Redacted bars (classified document) |
| 6 | Lines of Code comparison | Earthquake shake + flash on TrustOS bar |
| 7 | "What if one person could understand ALL of it?" | Light burst (16 star rays) |
| 8 | TrustOS stats | Odometer counter (0 → 120,000) |
| 9 | Feature grid (8 cards) | Glow pulse on card reveal |
| 10 | "Computing is not magic" | Sparkle dissolve → geometric shapes |
| 11 | "TrustOS proves it." | Expanding shockwave rings |
| 12 | Outro | Matrix rain callback |

### 8 Animated Backgrounds
Pulsing nebula, red scanlines, blueprint dot-grid, rising green sparks, deep-space starfield, circuit-board traces, sunrise gradient, matrix rain.

---

## 🔐 Ed25519 Asymmetric Signatures
- Full RFC 8032 implementation
- SHA-512, extended twisted Edwards curve
- GF(2^255-19) field reuse from TLS
- TweetNaCl-style scalar mod l reduction
- Replaces HMAC-only signatures with proper public-key cryptography
- `signature ed25519` shell command

## 🔧 Cross-platform Build
- GNU Makefile + `build.sh` for Linux/macOS
- Auto-detected OVMF paths
- `make run`, `make iso`, `make check-deps`

## 🎨 TrustLang Showcase
- Syntax highlighting: keywords (red), functions (blue), variables (cyan), strings (orange), comments (green), brackets (gold)
- Auto-scrolling editor panel with scrollbar

## 🎮 3D Chess
- Full 3D game with low-poly pieces
- Proper look-at camera (spherical coords)
- AI opponent (minimax depth 2)
- Board labels, shadows, reflections, scroll zoom

---

## 📦 Stats
- **120,000+** lines of pure Rust
- **216+** source files
- **10.86 MB** ISO
- **<1s** boot time
- **144 FPS** desktop
- **0** lines of C
- **8 days** development time
- **1** author

---

## 🚀 Quick Start

```powershell
# Clone and build
git clone https://github.com/nathan237/TrustOS.git
cd TrustOS
cargo build --release -p trustos_kernel

# Run with QEMU
qemu-system-x86_64 -bios OVMF.fd -drive format=raw,file=fat:rw:. -m 512M -serial stdio
```

Type `film` in the shell to watch the TrustOS Film.

---

*Trust the code. Rust is the reason.*
