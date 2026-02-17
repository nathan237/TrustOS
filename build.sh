#!/usr/bin/env bash
# ═══════════════════════════════════════════════════════════════
# TrustOS — Build Script (Linux / macOS / WSL)
# Creates a bootable ISO from the Rust kernel using Limine
#
# Usage:
#   ./build.sh              Build kernel + create ISO
#   ./build.sh --run        Build + run in QEMU (UEFI)
#   ./build.sh --run-bios   Build + run in QEMU (BIOS)
#   ./build.sh --clean      Clean build artifacts
#   ./build.sh --check      Check dependencies
# ═══════════════════════════════════════════════════════════════
set -euo pipefail

# ── Colors ──
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
CYAN='\033[0;36m'
RESET='\033[0m'

# ── Configuration (override via environment variables) ──
KERNEL_PKG="${KERNEL_PKG:-trustos_kernel}"
TARGET="${TARGET:-x86_64-unknown-none}"
PROFILE="${PROFILE:-release}"
ISO_NAME="${ISO_NAME:-trustos.iso}"
QEMU="${QEMU:-qemu-system-x86_64}"
QEMU_MEMORY="${QEMU_MEMORY:-512M}"
QEMU_CPUS="${QEMU_CPUS:-4}"

# ── Derived paths (relative to script location, no hardcoded paths) ──
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

KERNEL_BIN="target/${TARGET}/${PROFILE}/${KERNEL_PKG}"
ISO_ROOT="iso_root"
ISO_FILE="${ISO_NAME}"

# ── Helper functions ──
info()  { echo -e "${CYAN}$*${RESET}"; }
ok()    { echo -e "${GREEN}✓ $*${RESET}"; }
warn()  { echo -e "${YELLOW}⚠ $*${RESET}"; }
err()   { echo -e "${RED}✗ $*${RESET}"; }

find_ovmf() {
    local candidates=(
        "$SCRIPT_DIR/OVMF.fd"
        "/usr/share/OVMF/OVMF_CODE.fd"
        "/usr/share/edk2/ovmf/OVMF_CODE.fd"
        "/usr/share/qemu/OVMF.fd"
        "/usr/share/edk2-ovmf/x64/OVMF_CODE.fd"
        "/opt/homebrew/share/qemu/edk2-x86_64-code.fd"
    )
    for f in "${candidates[@]}"; do
        if [ -f "$f" ]; then
            echo "$f"
            return 0
        fi
    done
    return 1
}

check_deps() {
    info "══ Checking dependencies ══"
    local all_ok=true
    for cmd in cargo rustup git xorriso "$QEMU"; do
        if command -v "$cmd" &>/dev/null; then
            ok "$cmd"
        else
            err "$cmd — not found"
            all_ok=false
        fi
    done
    # Check Limine
    if [ -f "$SCRIPT_DIR/limine/BOOTX64.EFI" ]; then
        ok "Limine bootloader"
    else
        warn "Limine not found (will be auto-downloaded on first build)"
    fi
    if OVMF=$(find_ovmf); then
        ok "OVMF ($OVMF)"
    else
        warn "OVMF not found (needed for UEFI boot only)"
        echo "  Install: sudo apt install ovmf  (Debian/Ubuntu)"
        echo "           sudo dnf install edk2-ovmf  (Fedora)"
        echo "           brew install qemu  (macOS)"
    fi
    rustc --version 2>/dev/null || true
    $all_ok
}

do_build() {
    info "══ Building TrustOS kernel ($PROFILE) ══"
    cargo build --"$PROFILE" -p "$KERNEL_PKG"
    if [ ! -f "$KERNEL_BIN" ]; then
        err "Kernel binary not found at $KERNEL_BIN"
        exit 1
    fi
    local ksize
    ksize=$(du -h "$KERNEL_BIN" | cut -f1)
    ok "Kernel built: $KERNEL_BIN ($ksize)"
}

ensure_limine() {
    local LIMINE_DIR="$SCRIPT_DIR/limine"
    local LIMINE_BRANCH="${LIMINE_BRANCH:-v8.x-binary}"

    if [ -f "$LIMINE_DIR/BOOTX64.EFI" ] && [ -f "$LIMINE_DIR/limine-bios.sys" ]; then
        ok "Limine bootloader found"
        return 0
    fi

    info "Limine bootloader not found — downloading..."

    if ! command -v git &>/dev/null; then
        err "git is required to fetch Limine. Install git and retry."
        exit 1
    fi

    if [ -d "$LIMINE_DIR" ]; then
        warn "Limine directory exists but is incomplete, re-cloning..."
        rm -rf "$LIMINE_DIR"
    fi

    git clone "https://github.com/limine-bootloader/limine.git" \
        --branch="$LIMINE_BRANCH" --depth=1 "$LIMINE_DIR"

    # Build the limine utility if a Makefile is present (needed for BIOS install)
    if [ -f "$LIMINE_DIR/Makefile" ]; then
        info "Building limine utility..."
        make -C "$LIMINE_DIR" 2>/dev/null || warn "limine utility build skipped (not critical for UEFI)"
    fi

    if [ ! -f "$LIMINE_DIR/BOOTX64.EFI" ]; then
        err "Limine download succeeded but BOOTX64.EFI not found."
        err "Check https://github.com/limine-bootloader/limine for the correct branch."
        exit 1
    fi

    ok "Limine bootloader ready"
}

do_iso() {
    do_build

    info "══ Creating bootable ISO ══"

    # Auto-download Limine if not present
    ensure_limine

    # Ensure iso_root structure
    mkdir -p "$ISO_ROOT/boot/limine"
    mkdir -p "$ISO_ROOT/EFI/BOOT"

    # Copy kernel
    cp "$KERNEL_BIN" "$ISO_ROOT/boot/$KERNEL_PKG"

    # Copy Limine bootloader files (fail loudly now — they must exist)
    cp -f limine/BOOTX64.EFI         "$ISO_ROOT/EFI/BOOT/BOOTX64.EFI"
    cp -f limine/limine-bios.sys     "$ISO_ROOT/boot/limine/limine-bios.sys"
    cp -f limine/limine-bios-cd.bin  "$ISO_ROOT/boot/limine/limine-bios-cd.bin"
    cp -f limine/limine-uefi-cd.bin  "$ISO_ROOT/boot/limine/limine-uefi-cd.bin"

    # Copy boot config
    cp -f limine.conf "$ISO_ROOT/limine.conf"              2>/dev/null || true
    cp -f limine.conf "$ISO_ROOT/boot/limine/limine.conf"  2>/dev/null || true

    # Create ISO with xorriso
    if ! command -v xorriso &>/dev/null; then
        err "xorriso not found! Install it:"
        echo "  sudo apt install xorriso    (Debian/Ubuntu)"
        echo "  sudo dnf install xorriso    (Fedora)"
        echo "  brew install xorriso        (macOS)"
        exit 1
    fi

    xorriso -as mkisofs \
        -b boot/limine/limine-bios-cd.bin \
        -no-emul-boot -boot-load-size 4 -boot-info-table \
        --efi-boot boot/limine/limine-uefi-cd.bin \
        -efi-boot-part --efi-boot-image --protective-msdos-label \
        -o "$ISO_FILE" "$ISO_ROOT"

    ok "ISO created: $ISO_FILE ($(du -h "$ISO_FILE" | cut -f1))"
}

do_run() {
    local mode="${1:-uefi}"
    do_iso

    if ! command -v "$QEMU" &>/dev/null; then
        err "QEMU not found! Install qemu-system-x86"
        exit 1
    fi

    if [ "$mode" = "uefi" ]; then
        OVMF=$(find_ovmf) || {
            err "OVMF firmware not found!"
            echo "  Install: sudo apt install ovmf"
            echo "           sudo dnf install edk2-ovmf"
            echo "           brew install qemu  (macOS)"
            echo ""
            echo "  Or run with BIOS: $0 --run-bios"
            exit 1
        }

        info "══ Starting QEMU (UEFI) ══"
        echo "  ISO:    $ISO_FILE"
        echo "  OVMF:   $OVMF"
        echo "  Memory: $QEMU_MEMORY  CPUs: $QEMU_CPUS"

        "$QEMU" \
            -cdrom "$ISO_FILE" \
            -m "$QEMU_MEMORY" \
            -machine q35 \
            -cpu max \
            -smp "$QEMU_CPUS" \
            -display gtk \
            -vga std \
            -device virtio-gpu-pci,xres=1280,yres=800 \
            -device virtio-net-pci,netdev=net0 \
            -netdev user,id=net0 \
            -drive "if=pflash,format=raw,readonly=on,file=$OVMF" \
            -rtc base=utc,clock=vm \
            -serial stdio \
            -no-reboot
    else
        info "══ Starting QEMU (BIOS) ══"
        echo "  ISO:    $ISO_FILE"
        echo "  Memory: $QEMU_MEMORY  CPUs: $QEMU_CPUS"

        "$QEMU" \
            -cdrom "$ISO_FILE" \
            -m "$QEMU_MEMORY" \
            -machine pc \
            -cpu max \
            -smp "$QEMU_CPUS" \
            -display gtk \
            -vga std \
            -serial stdio \
            -no-reboot
    fi
}

do_clean() {
    info "══ Cleaning ══"
    cargo clean
    rm -f "$ISO_FILE"
    ok "Cleaned"
}

# ── Main ──
case "${1:-}" in
    --run)       do_run uefi ;;
    --run-bios)  do_run bios ;;
    --clean)     do_clean ;;
    --check)     check_deps ;;
    --help|-h)
        echo ""
        echo "TrustOS Build Script"
        echo ""
        echo "Usage: $0 [option]"
        echo ""
        echo "  (no args)    Build kernel + create bootable ISO"
        echo "  --run        Build + run in QEMU (UEFI mode)"
        echo "  --run-bios   Build + run in QEMU (BIOS mode)"
        echo "  --clean      Clean build artifacts"
        echo "  --check      Check if dependencies are installed"
        echo "  --help       Show this help"
        echo ""
        echo "Environment variables:"
        echo "  QEMU_MEMORY=1G   Override QEMU RAM (default: 512M)"
        echo "  QEMU_CPUS=2      Override CPU count (default: 4)"
        echo "  QEMU=path        Override QEMU binary path"
        echo "  PROFILE=debug    Build in debug mode (default: release)"
        echo ""
        ;;
    "")          do_iso ;;
    *)
        err "Unknown option: $1"
        echo "Run '$0 --help' for usage"
        exit 1
        ;;
esac
