#!/usr/bin/env bash
# ═══════════════════════════════════════════════════════════════
# TrustOS — One-Liner Setup (Linux / macOS / WSL)
#
# Usage:
#   curl -sSf https://raw.githubusercontent.com/nathan237/TrustOS/main/setup.sh | bash
#   # — or —
#   git clone https://github.com/nathan237/TrustOS.git && cd TrustOS && ./setup.sh
#
# What it does:
#   1. Installs Rust nightly toolchain (if missing)
#   2. Installs system dependencies (QEMU, xorriso, OVMF)
#   3. Clones Limine bootloader
#   4. Builds TrustOS kernel
#   5. Creates bootable ISO
#   6. (Optional) Launches in QEMU
# ═══════════════════════════════════════════════════════════════
set -euo pipefail

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
CYAN='\033[0;36m'
BOLD='\033[1m'
RESET='\033[0m'

info()  { echo -e "${CYAN}${BOLD}▸${RESET} $*"; }
ok()    { echo -e "${GREEN}✓${RESET} $*"; }
warn()  { echo -e "${YELLOW}⚠${RESET} $*"; }
err()   { echo -e "${RED}✗${RESET} $*"; exit 1; }

# ── Detect OS ──
detect_os() {
    if [ -f /etc/os-release ]; then
        . /etc/os-release
        case "$ID" in
            ubuntu|debian|pop|linuxmint|elementary|zorin) echo "debian" ;;
            fedora|rhel|centos|rocky|alma) echo "fedora" ;;
            arch|manjaro|endeavouros|artix) echo "arch" ;;
            opensuse*|sles) echo "suse" ;;
            void) echo "void" ;;
            alpine) echo "alpine" ;;
            *) echo "unknown" ;;
        esac
    elif [ "$(uname)" = "Darwin" ]; then
        echo "macos"
    elif grep -qi microsoft /proc/version 2>/dev/null; then
        echo "wsl"
    else
        echo "unknown"
    fi
}

DISTRO=$(detect_os)

echo ""
echo -e "${BOLD}${CYAN}═══════════════════════════════════════════${RESET}"
echo -e "${BOLD}${CYAN}       TrustOS — Automated Setup          ${RESET}"
echo -e "${BOLD}${CYAN}═══════════════════════════════════════════${RESET}"
echo -e "  Detected: ${BOLD}${DISTRO}${RESET}"
echo ""

# ── Step 1: Clone repo if not already inside it ──
if [ ! -f "Cargo.toml" ] || ! grep -q "trustos" Cargo.toml 2>/dev/null; then
    info "Cloning TrustOS repository..."
    if [ -d "TrustOS" ]; then
        cd TrustOS
        git pull --ff-only || true
    else
        git clone https://github.com/nathan237/TrustOS.git
        cd TrustOS
    fi
    ok "Repository ready"
else
    ok "Already in TrustOS directory"
fi

# ── Step 2: Install Rust nightly ──
install_rust() {
    if command -v rustup &>/dev/null; then
        ok "rustup already installed"
        info "Ensuring nightly toolchain..."
        rustup toolchain install nightly --profile minimal --component rust-src 2>/dev/null || true
    else
        info "Installing Rust via rustup..."
        curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain nightly
        source "$HOME/.cargo/env"
        rustup component add rust-src --toolchain nightly 2>/dev/null || true
        ok "Rust nightly installed"
    fi
}

install_rust

# ── Step 3: Install system dependencies ──
install_deps() {
    info "Installing system dependencies..."
    case "$DISTRO" in
        debian|wsl)
            sudo apt-get update -qq
            sudo apt-get install -y -qq qemu-system-x86 xorriso ovmf git make
            ok "Dependencies installed (apt)"
            ;;
        fedora)
            sudo dnf install -y qemu xorriso edk2-ovmf git make
            ok "Dependencies installed (dnf)"
            ;;
        arch)
            sudo pacman -Sy --noconfirm --needed qemu-full xorriso edk2-ovmf git make
            ok "Dependencies installed (pacman)"
            ;;
        suse)
            sudo zypper install -y qemu-x86 xorriso ovmf git make
            ok "Dependencies installed (zypper)"
            ;;
        void)
            sudo xbps-install -Sy qemu xorriso git make
            ok "Dependencies installed (xbps)"
            ;;
        alpine)
            sudo apk add qemu-system-x86_64 xorriso ovmf git make
            ok "Dependencies installed (apk)"
            ;;
        macos)
            if ! command -v brew &>/dev/null; then
                err "Homebrew not found. Install it first: https://brew.sh"
            fi
            brew install qemu xorriso
            ok "Dependencies installed (brew)"
            ;;
        *)
            warn "Unknown distro — please install manually:"
            echo "  - qemu-system-x86_64"
            echo "  - xorriso"
            echo "  - OVMF firmware"
            echo "  - git, make"
            ;;
    esac
}

install_deps

# ── Step 4: Fetch Limine bootloader ──
fetch_limine() {
    if [ -f "limine/BOOTX64.EFI" ]; then
        ok "Limine bootloader already present"
        return
    fi

    info "Downloading Limine bootloader..."
    rm -rf limine
    git clone https://github.com/limine-bootloader/limine.git \
        --branch=v8.x-binary --depth=1
    
    if [ -f "limine/Makefile" ]; then
        make -C limine 2>/dev/null || true
    fi

    if [ -f "limine/BOOTX64.EFI" ]; then
        ok "Limine bootloader ready"
    else
        err "Limine download failed — check https://github.com/limine-bootloader/limine"
    fi
}

fetch_limine

# ── Step 5: Build kernel ──
info "Building TrustOS kernel (release)..."
cargo build --release -p trustos_kernel 2>&1 | tail -5
if [ -f "target/x86_64-unknown-none/release/trustos_kernel" ]; then
    KSIZE=$(du -h "target/x86_64-unknown-none/release/trustos_kernel" | cut -f1)
    ok "Kernel built ($KSIZE)"
else
    err "Kernel build failed. Check errors above."
fi

# ── Step 6: Create ISO ──
info "Creating bootable ISO..."
mkdir -p iso_root/boot/limine iso_root/EFI/BOOT

cp target/x86_64-unknown-none/release/trustos_kernel iso_root/boot/trustos_kernel
cp -f limine/BOOTX64.EFI         iso_root/EFI/BOOT/BOOTX64.EFI
cp -f limine/limine-bios.sys     iso_root/boot/limine/
cp -f limine/limine-bios-cd.bin  iso_root/boot/limine/
cp -f limine/limine-uefi-cd.bin  iso_root/boot/limine/
cp -f limine.conf                iso_root/limine.conf
cp -f limine.conf                iso_root/boot/limine/limine.conf

xorriso -as mkisofs \
    -b boot/limine/limine-bios-cd.bin \
    -no-emul-boot -boot-load-size 4 -boot-info-table \
    --efi-boot boot/limine/limine-uefi-cd.bin \
    -efi-boot-part --efi-boot-image --protective-msdos-label \
    -o trustos.iso iso_root

ISOSIZE=$(du -h trustos.iso | cut -f1)
ok "ISO created: trustos.iso ($ISOSIZE)"

# ── Done! ──
echo ""
echo -e "${BOLD}${GREEN}═══════════════════════════════════════════${RESET}"
echo -e "${BOLD}${GREEN}       TrustOS is ready!                   ${RESET}"
echo -e "${BOLD}${GREEN}═══════════════════════════════════════════${RESET}"
echo ""
echo -e "  ${BOLD}Run in QEMU:${RESET}"
echo -e "    make run            # UEFI mode (needs OVMF)"
echo -e "    make run-bios       # BIOS mode (no OVMF needed)"
echo -e "    ./build.sh --run    # Alternative"
echo ""
echo -e "  ${BOLD}Or boot the ISO directly:${RESET}"
echo -e "    trustos.iso → VirtualBox / VMware / bare metal USB"
echo ""
echo -e "  ${BOLD}First commands to try:${RESET}"
echo -e "    showcase            # Automated feature tour"
echo -e "    desktop             # Launch desktop environment"
echo -e "    trustlab            # Kernel introspection laboratory"
echo -e "    neofetch            # System info"
echo ""

# ── Optional: auto-launch ──
if [ "${1:-}" = "--run" ]; then
    echo -e "${CYAN}Launching TrustOS in QEMU...${RESET}"
    make run
fi
