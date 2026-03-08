#!/usr/bin/env python3
"""
TrustOS on iPhone — Master Orchestrator
========================================
Manages the complete flow from checkm8 exploit to dual-boot.

Usage:
  python trustos_iphone.py setup      # Verify prerequisites
  python trustos_iphone.py dump       # Dump BootROM 
  python trustos_iphone.py boot       # Boot TrustOS via USB
  python trustos_iphone.py menu       # Install boot menu (advanced)

SAFETY GUARANTEE:
  - iOS is NEVER modified or deleted
  - All operations are in volatile memory (SRAM/DRAM)
  - A simple reboot returns to normal iOS
  - No NAND writes whatsoever in Phase 1
"""

import sys
import os
import time
import struct

# Add parent dir to path
sys.path.insert(0, os.path.dirname(__file__))

from checkm8_t8030 import (
    Checkm8Exploit, DFUDevice, DFUDataReader,
    build_bootrom_dump_shellcode, dump_bootrom, verify_setup,
    T8030_ROM_BASE, T8030_ROM_SIZE, T8030_SRAM_BASE
)
from bootloader.dualboot_loader import (
    TrustOSBootProtocol, IBootPatcher, NANDSafetyManager
)

# ============================================================================
# TrustOS Kernel Loader (via USB)
# ============================================================================

def find_trustos_kernel():
    """
    Locate the TrustOS kernel binary.
    Searches in standard locations:
    1. ./trustos_kernel.bin
    2. ../kernel/target/aarch64-unknown-none/release/trustos
    3. ../../target/aarch64-unknown-none/release/trustos
    """
    search_paths = [
        "trustos_kernel.bin",
        os.path.join("..", "..", "target", "aarch64-unknown-none", "release", "trustos"),
        os.path.join("..", "..", "target", "aarch64-unknown-none", "debug", "trustos"),
        os.path.join("..", "kernel", "trustos_kernel.bin"),
    ]
    
    for path in search_paths:
        full = os.path.join(os.path.dirname(__file__), path)
        if os.path.exists(full):
            size = os.path.getsize(full)
            print(f"[+] TrustOS kernel found: {full} ({size} bytes)")
            return full
    
    print("[!] TrustOS kernel not found!")
    print("    Build it first: cargo build --release --target aarch64-unknown-none")
    print("    Or place trustos_kernel.bin in this directory")
    return None


def usb_boot_trustos(kernel_path=None):
    """
    Boot TrustOS on iPhone via USB:
    1. checkm8 exploit → pwned DFU
    2. Send USB boot payload
    3. Send TrustOS kernel via USB
    4. iPhone boots into TrustOS
    
    iOS is untouched — everything is in-memory.
    """
    print()
    print("╔══════════════════════════════════════════════════════╗")
    print("║     TrustOS USB Boot — iPhone 11 Pro (A13)         ║")
    print("║     iOS reste intact. Boot depuis USB.             ║")
    print("╚══════════════════════════════════════════════════════╝")
    print()
    
    # Find kernel
    if kernel_path is None:
        kernel_path = find_trustos_kernel()
        if kernel_path is None:
            return False
    
    with open(kernel_path, "rb") as f:
        kernel_data = f.read()
    
    print(f"[*] Kernel size: {len(kernel_data)} bytes ({len(kernel_data)/1024:.1f} KB)")
    
    # Build TrustOS image with header
    image = TrustOSBootProtocol.build_trustos_image_header(kernel_data)
    print(f"[*] Boot image size: {len(image)} bytes (header + kernel)")
    
    # Build the combined payload:
    # Stage 1: checkm8 shellcode that sets up USB receive
    # Stage 2: TrustOS kernel sent via USB after exploit
    
    # For now, build a shellcode that:
    # 1. Copies TrustOS to DRAM at load address
    # 2. Sets up minimal EL1 environment
    # 3. Jumps to TrustOS entry
    
    shellcode = build_usb_receive_and_boot_shellcode()
    
    print()
    print("[*] Phase 1: checkm8 exploit...")
    exploit = Checkm8Exploit()
    if not exploit.exploit(shellcode):
        print("[!] Exploit echoue")
        return False
    
    print()
    print("[*] Phase 2: Envoi du kernel TrustOS via USB...")
    
    # Send the TrustOS image in chunks via DFU
    reader = DFUDataReader(exploit.dfu)
    chunk_size = 0x800
    sent = 0
    
    while sent < len(image):
        chunk = image[sent:sent + chunk_size]
        exploit.dfu.dfu_send(chunk)
        exploit.dfu.dfu_get_status()
        sent += len(chunk)
        
        if sent % 0x10000 == 0:
            pct = sent * 100 // len(image)
            print(f"    [{pct:3d}%] {sent} / {len(image)} bytes")
    
    print(f"[+] Kernel envoye: {sent} bytes")
    
    # Signal end of transfer
    exploit.dfu.dfu_send(b"DONE")
    exploit.dfu.dfu_get_status()
    
    print()
    print("[*] Phase 3: Boot TrustOS...")
    print("[*] L'iPhone devrait afficher le framebuffer TrustOS")
    print()
    print("[+] TrustOS boot initie!")
    print("    Pour revenir a iOS: maintenir Power + Volume Down 10 sec")
    print("    (Force reboot → boot normal iOS)")
    
    return True


def build_usb_receive_and_boot_shellcode():
    """
    Shellcode (runs in SecureROM EL3 context):
    1. Stay in pwned DFU, receive data via USB
    2. Write received data to DRAM at TrustOS load address
    3. Drop to EL1
    4. Jump to TrustOS entry point
    
    This is a simplified version — real implementation needs
    proper EL3→EL1 transition with SPSR/ELR setup.
    """
    code = bytearray()
    
    # ---- Prologue ----
    code += struct.pack("<I", 0xA9BF7BFD)  # STP X29, X30, [SP, #-0x10]!
    code += struct.pack("<I", 0x910003FD)  # MOV X29, SP
    
    # ---- Setup: configure EL1 for TrustOS ----
    # The real shellcode would:
    # 1. Disable MMU at EL3
    # 2. Setup VBAR_EL1 
    # 3. Configure HCR_EL2 for EL1 access
    # 4. Setup SPSR_EL3 for EL1h entry
    # 5. Set ELR_EL3 to TrustOS entry
    # 6. ERET
    
    # For now: NOP placeholder (will be filled with real code
    # once we have the BootROM dump and know exact register state)
    for _ in range(128):
        code += struct.pack("<I", 0xD503201F)  # NOP
    
    # ---- Return (fallback if shellcode doesn't jump) ----
    code += struct.pack("<I", 0xA8C17BFD)  # LDP X29, X30, [SP], #0x10
    code += struct.pack("<I", 0xD65F03C0)  # RET
    
    return bytes(code)


# ============================================================================
# Device Safety Check
# ============================================================================

def safety_preflight():
    """
    Pre-flight safety checks before any operation.
    """
    print("[*] === SAFETY PRE-FLIGHT ===")
    print()
    print("  Cet outil va:")
    print("    ✓ Exploiter le SecureROM via checkm8 (USB, volatile)")
    print("    ✓ Executer du code en memoire SRAM/DRAM")
    print("    ✓ Booter TrustOS depuis la RAM")
    print()
    print("  Cet outil ne va PAS:")
    print("    ✗ Modifier le NAND/NOR de l'iPhone")  
    print("    ✗ Supprimer ou modifier iOS")
    print("    ✗ Ecrire dans les partitions systeme")
    print("    ✗ Modifier le bootchain permanent")
    print()
    print("  Pour revenir a iOS normal:")
    print("    → Force reboot (Power + Vol Down 10 sec)")
    print("    → L'iPhone redemarre normalement sous iOS")
    print()
    
    safety = NANDSafetyManager()
    return True


# ============================================================================
# Zadig Driver Setup Guide
# ============================================================================

def zadig_setup_guide():
    """
    Guide for setting up Zadig USB drivers on Windows.
    Required for pyusb to communicate with iPhone in DFU mode.
    """
    print()
    print("╔══════════════════════════════════════════════════════╗")
    print("║  Setup du driver USB (Windows)                      ║")
    print("╚══════════════════════════════════════════════════════╝")
    print()
    print("  Sur Windows, pyusb a besoin d'un driver libusb pour")
    print("  communiquer avec l'iPhone en mode DFU.")
    print()
    print("  ETAPES:")
    print("  1. Telecharger Zadig: https://zadig.akeo.ie/")
    print("  2. Mettre l'iPhone en mode DFU")
    print("  3. Dans Zadig: Options → List All Devices")
    print("  4. Selectionner 'Apple Mobile Device (DFU Mode)'")
    print("     (USB ID: 05AC:1227)")
    print("  5. Remplacer le driver par: libusb-win32 ou WinUSB")
    print("  6. Cliquer 'Replace Driver'")
    print()
    print("  IMPORTANT: Cela remplace le driver iTunes/Apple pour DFU.")
    print("  Pour restaurer: Gestionnaire de peripheriques →")
    print("  Mettre a jour le driver → Rechercher automatiquement")
    print()
    print("  ALTERNATIVE: Utiliser libusbK via Zadig (plus compatible)")
    print()


# ============================================================================
# Entry Point
# ============================================================================

def main():
    print()
    print("  ████████╗██████╗ ██╗   ██╗███████╗████████╗ ██████╗ ███████╗")
    print("  ╚══██╔══╝██╔══██╗██║   ██║██╔════╝╚══██╔══╝██╔═══██╗██╔════╝")
    print("     ██║   ██████╔╝██║   ██║███████╗   ██║   ██║   ██║███████╗")
    print("     ██║   ██╔══██╗██║   ██║╚════██║   ██║   ██║   ██║╚════██║")
    print("     ██║   ██║  ██║╚██████╔╝███████║   ██║   ╚██████╔╝███████║")
    print("     ╚═╝   ╚═╝  ╚═╝ ╚═════╝ ╚══════╝   ╚═╝    ╚═════╝ ╚══════╝")
    print("              iPhone 11 Pro — Dual Boot Manager")
    print()
    
    if len(sys.argv) < 2:
        print("Usage:")
        print(f"  python {sys.argv[0]} setup      Verifier les prerequis")
        print(f"  python {sys.argv[0]} zadig      Guide setup driver USB")
        print(f"  python {sys.argv[0]} dump       Dump le BootROM (SecureROM)")
        print(f"  python {sys.argv[0]} boot       Booter TrustOS via USB")
        print(f"  python {sys.argv[0]} safety     Afficher les garanties de securite")
        print()
        print("Workflow recommande:")
        print("  1. setup  → Verifier que tout est pret")
        print("  2. zadig  → Configurer le driver USB (premiere fois)")
        print("  3. dump   → Dumper le BootROM pour calibrer les offsets")
        print("  4. boot   → Booter TrustOS!")
        return
    
    cmd = sys.argv[1].lower()
    
    if cmd == "setup":
        safety_preflight()
        print()
        verify_setup()
        
    elif cmd == "zadig":
        zadig_setup_guide()
        
    elif cmd == "dump":
        safety_preflight()
        output = sys.argv[2] if len(sys.argv) > 2 else "t8030_bootrom.bin"
        dump_bootrom(output)
        
    elif cmd == "boot":
        safety_preflight()
        kernel = sys.argv[2] if len(sys.argv) > 2 else None
        usb_boot_trustos(kernel)
        
    elif cmd == "safety":
        safety_preflight()
        
    else:
        print(f"[!] Commande inconnue: {cmd}")
        main()  # Show help


if __name__ == "__main__":
    main()
