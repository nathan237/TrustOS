# TrustOS — Cross-platform Makefile
# Works on Linux, macOS, and Windows (via WSL/MSYS2/Git Bash)
#
# Usage:
#   make build        — Build the kernel (release)
#   make iso          — Build kernel + create bootable ISO
#   make run          — Build + run in QEMU (UEFI)
#   make run-bios     — Build + run in QEMU (BIOS/legacy)
#   make clean        — Clean build artifacts
#   make help         — Show available targets
#
# Requirements:
#   - Rust nightly toolchain (see rust-toolchain.toml)
#   - xorriso (for ISO creation)
#   - qemu-system-x86_64 (for running)
#   - OVMF firmware (for UEFI boot in QEMU)

.PHONY: build iso run run-bios clean help check-deps

# ── Configuration (override via environment) ──
KERNEL_PKG    ?= trustos_kernel
TARGET        ?= x86_64-unknown-none
PROFILE       ?= release
ISO_NAME      ?= trustos.iso
QEMU          ?= qemu-system-x86_64
QEMU_MEMORY   ?= 512M
QEMU_CPUS     ?= 4

# ── Derived paths (no hardcoded absolute paths) ──
ROOT_DIR      := $(shell pwd)
KERNEL_BIN    := target/$(TARGET)/$(PROFILE)/$(KERNEL_PKG)
ISO_ROOT      := iso_root
ISO_FILE      := $(ISO_NAME)

# Detect OVMF location
OVMF_CODE     ?= $(shell \
	if [ -f "$(ROOT_DIR)/OVMF.fd" ]; then echo "$(ROOT_DIR)/OVMF.fd"; \
	elif [ -f /usr/share/OVMF/OVMF_CODE.fd ]; then echo "/usr/share/OVMF/OVMF_CODE.fd"; \
	elif [ -f /usr/share/edk2/ovmf/OVMF_CODE.fd ]; then echo "/usr/share/edk2/ovmf/OVMF_CODE.fd"; \
	elif [ -f /usr/share/qemu/OVMF.fd ]; then echo "/usr/share/qemu/OVMF.fd"; \
	elif [ -f /opt/homebrew/share/qemu/edk2-x86_64-code.fd ]; then echo "/opt/homebrew/share/qemu/edk2-x86_64-code.fd"; \
	else echo "OVMF_NOT_FOUND"; fi)

# ── Colors ──
GREEN  := \033[0;32m
YELLOW := \033[0;33m
RED    := \033[0;31m
CYAN   := \033[0;36m
RESET  := \033[0m

# ══════════════════════════════════════════════════
#  Targets
# ══════════════════════════════════════════════════

help:
	@echo ""
	@echo "$(CYAN)═══ TrustOS Build System ═══$(RESET)"
	@echo ""
	@echo "  $(GREEN)make build$(RESET)       Build the kernel (release)"
	@echo "  $(GREEN)make iso$(RESET)         Build kernel + create bootable ISO"
	@echo "  $(GREEN)make run$(RESET)         Build + run in QEMU (UEFI mode)"
	@echo "  $(GREEN)make run-bios$(RESET)    Build + run in QEMU (BIOS/legacy)"
	@echo "  $(GREEN)make clean$(RESET)       Clean build artifacts"
	@echo "  $(GREEN)make check-deps$(RESET)  Check if required tools are installed"
	@echo ""
	@echo "  Override variables:  QEMU_MEMORY=1G  QEMU_CPUS=2  make run"
	@echo ""

## Build the kernel
build:
	@echo "$(CYAN)══ Building TrustOS kernel ($(PROFILE)) ══$(RESET)"
	cargo build --$(PROFILE) -p $(KERNEL_PKG)
	@KSIZE=$$(du -h $(KERNEL_BIN) 2>/dev/null | cut -f1); \
	echo "$(GREEN)✓ Kernel built: $(KERNEL_BIN) ($$KSIZE)$(RESET)"

## Create bootable ISO with Limine
iso: build
	@echo "$(CYAN)══ Creating bootable ISO ══$(RESET)"
	@mkdir -p $(ISO_ROOT)/boot/limine
	@mkdir -p $(ISO_ROOT)/EFI/BOOT
	@# Copy kernel
	@cp $(KERNEL_BIN) $(ISO_ROOT)/boot/$(KERNEL_PKG)
	@# Copy Limine bootloader files
	@cp -f limine/BOOTX64.EFI    $(ISO_ROOT)/EFI/BOOT/BOOTX64.EFI     2>/dev/null || true
	@cp -f limine/limine-bios.sys    $(ISO_ROOT)/boot/limine/           2>/dev/null || true
	@cp -f limine/limine-bios-cd.bin $(ISO_ROOT)/boot/limine/           2>/dev/null || true
	@cp -f limine/limine-uefi-cd.bin $(ISO_ROOT)/boot/limine/           2>/dev/null || true
	@# Copy boot config
	@cp -f limine.conf $(ISO_ROOT)/limine.conf                          2>/dev/null || true
	@cp -f limine.conf $(ISO_ROOT)/boot/limine/limine.conf              2>/dev/null || true
	@# Create ISO
	xorriso -as mkisofs \
		-b boot/limine/limine-bios-cd.bin \
		-no-emul-boot -boot-load-size 4 -boot-info-table \
		--efi-boot boot/limine/limine-uefi-cd.bin \
		-efi-boot-part --efi-boot-image --protective-msdos-label \
		-o $(ISO_FILE) $(ISO_ROOT)
	@echo "$(GREEN)✓ ISO created: $(ISO_FILE)$(RESET)"

## Run in QEMU (UEFI mode)
run: iso
	@echo "$(CYAN)══ Starting QEMU (UEFI) ══$(RESET)"
	@if [ "$(OVMF_CODE)" = "OVMF_NOT_FOUND" ]; then \
		echo "$(RED)✗ OVMF firmware not found!$(RESET)"; \
		echo "  Install: sudo apt install ovmf  (Debian/Ubuntu)"; \
		echo "           sudo dnf install edk2-ovmf  (Fedora)"; \
		echo "           brew install qemu  (macOS, includes OVMF)"; \
		exit 1; \
	fi
	@echo "  ISO:    $(ISO_FILE)"
	@echo "  OVMF:   $(OVMF_CODE)"
	@echo "  Memory: $(QEMU_MEMORY)  CPUs: $(QEMU_CPUS)"
	$(QEMU) \
		-cdrom $(ISO_FILE) \
		-m $(QEMU_MEMORY) \
		-machine q35 \
		-cpu max \
		-smp $(QEMU_CPUS) \
		-display gtk \
		-vga std \
		-device virtio-gpu-pci,xres=1280,yres=800 \
		-device virtio-net-pci,netdev=net0 \
		-netdev user,id=net0 \
		-drive if=pflash,format=raw,readonly=on,file=$(OVMF_CODE) \
		-rtc base=utc,clock=vm \
		-serial stdio \
		-no-reboot

## Run in QEMU (BIOS/legacy mode — no OVMF needed)
run-bios: iso
	@echo "$(CYAN)══ Starting QEMU (BIOS) ══$(RESET)"
	$(QEMU) \
		-cdrom $(ISO_FILE) \
		-m $(QEMU_MEMORY) \
		-machine pc \
		-cpu max \
		-smp $(QEMU_CPUS) \
		-display gtk \
		-vga std \
		-serial stdio \
		-no-reboot

## Check if required tools are installed
check-deps:
	@echo "$(CYAN)══ Checking dependencies ══$(RESET)"
	@command -v cargo    >/dev/null 2>&1 && echo "$(GREEN)✓ cargo$(RESET)"    || echo "$(RED)✗ cargo    — install from https://rustup.rs$(RESET)"
	@command -v rustup   >/dev/null 2>&1 && echo "$(GREEN)✓ rustup$(RESET)"   || echo "$(RED)✗ rustup   — install from https://rustup.rs$(RESET)"
	@command -v xorriso  >/dev/null 2>&1 && echo "$(GREEN)✓ xorriso$(RESET)"  || echo "$(RED)✗ xorriso  — apt install xorriso / dnf install xorriso$(RESET)"
	@command -v $(QEMU)  >/dev/null 2>&1 && echo "$(GREEN)✓ qemu$(RESET)"     || echo "$(RED)✗ qemu     — apt install qemu-system-x86 / dnf install qemu$(RESET)"
	@if [ "$(OVMF_CODE)" != "OVMF_NOT_FOUND" ]; then \
		echo "$(GREEN)✓ OVMF ($(OVMF_CODE))$(RESET)"; \
	else \
		echo "$(YELLOW)⚠ OVMF not found (needed for UEFI boot only)$(RESET)"; \
	fi
	@rustc --version 2>/dev/null || true
	@echo ""

## Clean build artifacts
clean:
	cargo clean
	@rm -f $(ISO_FILE)
	@echo "$(GREEN)✓ Cleaned$(RESET)"
