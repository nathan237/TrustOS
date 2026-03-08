#!/usr/bin/env python3
"""
TrustOS Live Test Harness — Automated iPhone Testing
=====================================================
Script automatise qui:
  1. Detecte l'iPhone en DFU automatiquement (polling USB)
  2. Execute checkm8 
  3. Lance les tests demandes
  4. Ecrit les resultats dans un fichier JSON/TXT pour analyse
  5. Boucle pour le prochain test

Pensé pour être piloté par Copilot: il lit les fichiers de résultats,
ajuste les paramètres, et relance les tests en boucle.

Usage:
  python live_test.py                    # Mode interactif (menu)
  python live_test.py --probe            # Detecter le device seulement
  python live_test.py --dump-rom         # Dump BootROM complet
  python live_test.py --read ADDR SIZE   # Lire memoire physique
  python live_test.py --test-all         # Tous les tests enchaines
  python live_test.py --watch            # Mode daemon: attend DFU et teste
"""

import sys
import os
import json
import time
import struct
import hashlib
import traceback
from datetime import datetime
from pathlib import Path

# Setup paths
SCRIPT_DIR = Path(__file__).parent.resolve()
RESULTS_DIR = SCRIPT_DIR / "results"
RESULTS_DIR.mkdir(exist_ok=True)

sys.path.insert(0, str(SCRIPT_DIR))

try:
    import usb.core
    import usb.util
    import libusb_package
    import usb.backend.libusb1
    HAS_USB = True
except ImportError:
    HAS_USB = False

from checkm8_t8030 import (
    Checkm8Exploit, DFUDevice, DFUDataReader,
    build_bootrom_dump_shellcode,
    T8030_ROM_BASE, T8030_ROM_SIZE, T8030_SRAM_BASE,
    APPLE_VID, DFU_PID
)

# ============================================================================
# Constants
# ============================================================================

# Result file that Copilot reads after each test
LIVE_RESULT_FILE    = RESULTS_DIR / "live_test_result.json"
LIVE_LOG_FILE       = RESULTS_DIR / "live_test_log.txt"
COMMAND_FILE        = RESULTS_DIR / "live_test_command.json"  # Copilot writes commands here
ROM_DUMP_FILE       = RESULTS_DIR / "t8030_bootrom.bin"
ROM_ANALYSIS_FILE   = RESULTS_DIR / "bootrom_analysis.json"

# Test status constants
STATUS_WAITING    = "waiting_for_device"
STATUS_CONNECTED  = "device_connected"
STATUS_EXPLOITING = "exploiting"
STATUS_PWNED      = "pwned"
STATUS_TESTING    = "running_test"
STATUS_DONE       = "test_complete"
STATUS_ERROR      = "error"
STATUS_IDLE       = "idle"

# ============================================================================
# Logging
# ============================================================================

class LiveLogger:
    """Dual logger: console + file for Copilot to read."""
    
    def __init__(self):
        self.log_path = LIVE_LOG_FILE
        self._clear_log()
    
    def _clear_log(self):
        with open(self.log_path, "w", encoding="utf-8") as f:
            f.write(f"=== TrustOS Live Test Log — {datetime.now().isoformat()} ===\n\n")
    
    def log(self, msg, level="INFO"):
        timestamp = datetime.now().strftime("%H:%M:%S.%f")[:-3]
        line = f"[{timestamp}] [{level}] {msg}"
        print(line)
        with open(self.log_path, "a", encoding="utf-8") as f:
            f.write(line + "\n")
    
    def info(self, msg):   self.log(msg, "INFO")
    def ok(self, msg):     self.log(msg, " OK ")
    def warn(self, msg):   self.log(msg, "WARN")
    def error(self, msg):  self.log(msg, " ERR")
    def data(self, msg):   self.log(msg, "DATA")

log = LiveLogger()

# ============================================================================
# Result Writer — Copilot reads this after each test
# ============================================================================

def write_result(test_name, status, data=None, error=None):
    """
    Write test result to JSON file that Copilot can read.
    
    Format:
    {
        "timestamp": "2026-03-02T13:50:00",
        "test": "probe_device",
        "status": "success|failure|error",
        "device": { "cpid": "0x8030", "serial": "...", ... },
        "data": { ... test-specific results ... },
        "error": "error message if any"
    }
    """
    result = {
        "timestamp": datetime.now().isoformat(),
        "test": test_name,
        "status": status,
        "data": data or {},
        "error": error,
    }
    
    with open(LIVE_RESULT_FILE, "w", encoding="utf-8") as f:
        json.dump(result, f, indent=2, ensure_ascii=False, default=str)
    
    log.info(f"Result written: {LIVE_RESULT_FILE}")
    return result


def read_command():
    """
    Read command file written by Copilot.
    
    Format:
    {
        "command": "read_memory",
        "args": { "address": "0x100000000", "size": 512 }
    }
    """
    if not COMMAND_FILE.exists():
        return None
    
    try:
        with open(COMMAND_FILE, "r", encoding="utf-8") as f:
            cmd = json.load(f)
        # Delete after reading (one-shot)
        COMMAND_FILE.unlink()
        return cmd
    except Exception:
        return None


# ============================================================================
# Device Probe (no exploit needed)
# ============================================================================

def probe_device():
    """
    Detect iPhone in DFU mode and report device info.
    No exploit — just USB enumeration.
    """
    log.info("=== PROBE: Detecting DFU device ===")
    
    if not HAS_USB:
        write_result("probe", "error", error="pyusb not installed")
        return None
    
    dfu = DFUDevice()
    if not dfu.find_device():
        log.warn("No DFU device found")
        write_result("probe", "no_device")
        return None
    
    # Parse device info
    info = {
        "vid": f"0x{APPLE_VID:04X}",
        "pid": f"0x{DFU_PID:04X}",
        "serial": dfu.serial or "N/A",
        "cpid": f"0x{dfu.cpid:04X}" if dfu.cpid else "unknown",
    }
    
    # Parse extended serial info
    if dfu.serial:
        for part in dfu.serial.split(" "):
            if ":" in part:
                key, val = part.split(":", 1)
                info[key.lower()] = val
    
    # Validate: is this an A13? (T8030 reports CPID:8020)
    is_a13 = dfu.cpid == 0x8020 if dfu.cpid else False
    info["is_a13"] = is_a13
    info["is_supported"] = is_a13
    
    if is_a13:
        log.ok(f"iPhone A13 (T8030) detected! CPID: {info['cpid']}")
    else:
        log.warn(f"Device detected but CPID={info['cpid']} — not A13!")
    
    # USB descriptor details
    try:
        cfg = dfu.dev.get_active_configuration()
        info["usb_config"] = cfg.bConfigurationValue
        info["usb_interfaces"] = cfg.bNumInterfaces
    except Exception:
        pass
    
    write_result("probe", "success" if is_a13 else "wrong_device", data=info)
    return dfu if is_a13 else None


# ============================================================================
# checkm8 Exploit Runner
# ============================================================================

def run_exploit():
    """
    Run checkm8 exploit and return pwned DFU device.
    """
    log.info("=== EXPLOIT: Running checkm8 on T8030 ===")
    
    shellcode = build_bootrom_dump_shellcode()
    log.info(f"Shellcode size: {len(shellcode)} bytes")
    
    exploit = Checkm8Exploit()
    
    # Quick device check first
    if not exploit.dfu.find_device():
        log.error("No DFU device — put iPhone in DFU mode first")
        write_result("exploit", "no_device", error="iPhone not in DFU mode")
        return None
    
    cpid = exploit.dfu.cpid
    if cpid and cpid != 0x8020:
        log.error(f"Wrong device: CPID=0x{cpid:04X}, need 0x8020 (T8030/A13)")
        write_result("exploit", "wrong_device", data={"cpid": f"0x{cpid:04X}"})
        return None
    
    log.info("Device found, starting exploit...")
    write_result("exploit", "in_progress")
    
    success = exploit.exploit(shellcode)
    
    if success:
        log.ok("checkm8 SUCCESS — device is pwned!")
        write_result("exploit", "success", data={
            "pwned": True,
            "serial": exploit.dfu.serial or "N/A"
        })
        return exploit
    else:
        log.error("checkm8 FAILED after all attempts")
        write_result("exploit", "failure", error="Exploit did not succeed")
        return None


# ============================================================================
# BootROM Dump
# ============================================================================

def dump_bootrom_live():
    """
    Full BootROM dump with detailed analysis.
    """
    log.info("=== DUMP: BootROM (SecureROM) T8030 ===")
    log.info(f"Target: 0x{T8030_ROM_BASE:X}, size: {T8030_ROM_SIZE} bytes ({T8030_ROM_SIZE//1024} KB)")
    
    # Step 1: Exploit
    exploit = run_exploit()
    if not exploit:
        return False
    
    # Step 2: Read ROM from SRAM (where shellcode copied it)
    log.info("Reading BootROM dump from device SRAM...")
    reader = DFUDataReader(exploit.dfu)
    
    rom_data = bytearray()
    chunk_size = 0x800
    total = T8030_ROM_SIZE
    start_time = time.time()
    
    for offset in range(0, total, chunk_size):
        remaining = min(chunk_size, total - offset)
        
        try:
            chunk = exploit.dfu.ctrl_transfer(
                0xA1, 2,  # DFU UPLOAD
                0, 0, remaining, timeout=5000
            )
            if chunk and len(chunk) > 0:
                rom_data.extend(chunk)
            else:
                log.warn(f"Empty read at offset 0x{offset:X}")
                rom_data.extend(b"\x00" * remaining)
        except Exception as e:
            log.warn(f"Read error at 0x{offset:X}: {e}")
            rom_data.extend(b"\x00" * remaining)
        
        # Progress every 64KB
        if offset > 0 and offset % 0x10000 == 0:
            pct = offset * 100 // total
            elapsed = time.time() - start_time
            speed = offset / elapsed if elapsed > 0 else 0
            log.info(f"  [{pct:3d}%] 0x{offset:X}/{total} — {speed/1024:.0f} KB/s")
    
    elapsed = time.time() - start_time
    log.ok(f"Dump complete: {len(rom_data)} bytes in {elapsed:.1f}s")
    
    # Step 3: Save raw dump
    with open(ROM_DUMP_FILE, "wb") as f:
        f.write(rom_data)
    log.ok(f"Saved: {ROM_DUMP_FILE}")
    
    # Step 4: Analyze dump
    analysis = analyze_bootrom(rom_data)
    
    # Step 5: Write analysis
    with open(ROM_ANALYSIS_FILE, "w", encoding="utf-8") as f:
        json.dump(analysis, f, indent=2, ensure_ascii=False)
    
    write_result("dump_bootrom", "success", data={
        "size": len(rom_data),
        "sha256": analysis["sha256"],
        "valid": analysis["appears_valid"],
        "file": str(ROM_DUMP_FILE),
        "analysis_file": str(ROM_ANALYSIS_FILE),
        "first_bytes": rom_data[:64].hex(),
    })
    
    return True


def analyze_bootrom(data):
    """
    Analyze a BootROM dump for validity and interesting content.
    Returns analysis dict for Copilot to review.
    """
    log.info("Analyzing BootROM dump...")
    
    analysis = {
        "size": len(data),
        "sha256": hashlib.sha256(data).hexdigest(),
        "md5": hashlib.md5(data).hexdigest(),
        "appears_valid": False,
        "is_all_zeros": data == bytes(len(data)),
        "is_all_ff": data == bytes([0xFF] * len(data)),
        "entropy": 0.0,
        "strings": [],
        "arm64_markers": [],
        "exception_vectors": {},
        "first_64_bytes": data[:64].hex(),
        "last_64_bytes": data[-64:].hex(),
    }
    
    # Entropy calculation
    if len(data) > 0:
        from collections import Counter
        byte_counts = Counter(data)
        import math
        entropy = 0
        for count in byte_counts.values():
            p = count / len(data)
            if p > 0:
                entropy -= p * math.log2(p)
        analysis["entropy"] = round(entropy, 4)
    
    # Extract ASCII strings (min 6 chars)
    current = bytearray()
    strings = []
    for i, b in enumerate(data):
        if 0x20 <= b < 0x7F:
            current.append(b)
        else:
            if len(current) >= 6:
                s = current.decode("ascii", errors="ignore")
                strings.append({"offset": i - len(current), "string": s})
            current = bytearray()
    analysis["strings"] = strings[:200]  # Limit to 200
    
    # Check for ARM64 exception vector table
    # SecureROM should start with exception vectors (branch instructions)
    if len(data) >= 0x800:
        for vec_idx in range(16):
            offset = vec_idx * 0x80
            if offset + 4 <= len(data):
                insn = struct.unpack("<I", data[offset:offset+4])[0]
                is_branch = (insn & 0xFC000000) == 0x14000000  # B imm
                is_nop = insn == 0xD503201F
                analysis["exception_vectors"][f"0x{offset:03X}"] = {
                    "instruction": f"0x{insn:08X}",
                    "is_branch": is_branch,
                    "target_offset": (insn & 0x3FFFFFF) * 4 if is_branch else None
                }
    
    # Check for known SecureROM signatures
    known_sigs = [
        b"iBoot",
        b"SecureROM",
        b"Apple",
        b"T8030",
        b"CPID:",
        b"SRTG:",
        b"USB",
        b"DFU",
        b"IMG4",
        b"CERT",
    ]
    for sig in known_sigs:
        idx = data.find(sig)
        if idx >= 0:
            analysis["arm64_markers"].append({
                "signature": sig.decode("ascii", errors="ignore"),
                "offset": f"0x{idx:X}"
            })
    
    # Validity assessment
    analysis["appears_valid"] = (
        analysis["entropy"] > 3.0 and  # Not all zeros/patterns
        not analysis["is_all_zeros"] and
        not analysis["is_all_ff"] and
        len(analysis["arm64_markers"]) > 0  # Has known strings
    )
    
    status = "VALID" if analysis["appears_valid"] else "SUSPECT"
    log.info(f"Analysis: {status}")
    log.info(f"  SHA256: {analysis['sha256']}")
    log.info(f"  Entropy: {analysis['entropy']} bits/byte")
    log.info(f"  Strings found: {len(analysis['strings'])}")
    log.info(f"  Known signatures: {len(analysis['arm64_markers'])}")
    
    for marker in analysis["arm64_markers"]:
        log.data(f"  Found '{marker['signature']}' at {marker['offset']}")
    
    return analysis


# ============================================================================
# Memory Read (arbitrary physical address)
# ============================================================================

def read_physical_memory(address, size):
    """
    Read arbitrary physical memory after checkm8 exploit.
    Useful for reading MMIO registers, SRAM, etc.
    """
    log.info(f"=== READ: 0x{address:X}, size={size} ===")
    
    # Build custom shellcode that reads from target address
    # and copies to SRAM for USB readback
    shellcode = build_memread_shellcode(address, size)
    
    exploit = run_exploit()
    if not exploit:
        return None
    
    # Read back from SRAM
    reader = DFUDataReader(exploit.dfu)
    data = reader.read_memory(T8030_SRAM_BASE, size)
    
    if data:
        log.ok(f"Read {len(data)} bytes from 0x{address:X}")
        
        # Save to file
        out_file = RESULTS_DIR / f"memread_0x{address:X}_{size}.bin"
        with open(out_file, "wb") as f:
            f.write(data)
        
        # Hex dump first 256 bytes
        hex_preview = ""
        for i in range(0, min(256, len(data)), 16):
            hex_line = " ".join(f"{b:02X}" for b in data[i:i+16])
            ascii_line = "".join(chr(b) if 0x20 <= b < 0x7F else "." for b in data[i:i+16])
            hex_preview += f"  0x{address+i:010X}: {hex_line:<48s} {ascii_line}\n"
        
        log.data(f"Preview:\n{hex_preview}")
        
        write_result("read_memory", "success", data={
            "address": f"0x{address:X}",
            "size": len(data),
            "file": str(out_file),
            "sha256": hashlib.sha256(data).hexdigest(),
            "hex_preview": data[:64].hex(),
        })
        return data
    else:
        write_result("read_memory", "failure", error="No data returned")
        return None


def build_memread_shellcode(src_addr, size):
    """Build shellcode to copy from src_addr to SRAM buffer."""
    code = bytearray()
    
    # STP X29, X30, [SP, #-0x10]!
    code += struct.pack("<I", 0xA9BF7BFD)
    
    # Load source address into X0
    # MOV X0, #imm (may need multiple MOVZ/MOVK)
    code += _mov_x_imm64(0, src_addr)
    
    # Load dest (SRAM buffer) into X1
    code += _mov_x_imm64(1, T8030_SRAM_BASE)
    
    # Load size into X2
    code += _mov_x_imm64(2, size)
    
    # Copy loop
    code += struct.pack("<I", 0xF8408403)  # LDR X3, [X0], #8
    code += struct.pack("<I", 0xF8008423)  # STR X3, [X1], #8
    code += struct.pack("<I", 0xF1002042)  # SUBS X2, X2, #8
    code += struct.pack("<I", 0x54FFFF81)  # B.NE -3
    
    # LDP X29, X30, [SP], #0x10
    code += struct.pack("<I", 0xA8C17BFD)
    # RET
    code += struct.pack("<I", 0xD65F03C0)
    
    return bytes(code)


def _mov_x_imm64(reg, value):
    """Generate MOVZ + MOVK sequence to load 64-bit immediate into Xreg."""
    code = bytearray()
    # MOVZ Xreg, #(value & 0xFFFF)
    imm16_0 = value & 0xFFFF
    code += struct.pack("<I", 0xD2800000 | (imm16_0 << 5) | reg)
    
    # MOVK Xreg, #((value >> 16) & 0xFFFF), LSL#16
    imm16_1 = (value >> 16) & 0xFFFF
    if imm16_1:
        code += struct.pack("<I", 0xF2A00000 | (imm16_1 << 5) | reg)
    
    # MOVK Xreg, #((value >> 32) & 0xFFFF), LSL#32
    imm16_2 = (value >> 32) & 0xFFFF
    if imm16_2:
        code += struct.pack("<I", 0xF2C00000 | (imm16_2 << 5) | reg)
    
    # MOVK Xreg, #((value >> 48) & 0xFFFF), LSL#48
    imm16_3 = (value >> 48) & 0xFFFF
    if imm16_3:
        code += struct.pack("<I", 0xF2E00000 | (imm16_3 << 5) | reg)
    
    return bytes(code)


# ============================================================================
# Watch Mode — Daemon that waits for DFU and auto-tests
# ============================================================================

def watch_mode():
    """
    Daemon mode: 
    1. Poll USB for DFU device every 2 seconds
    2. When found, run exploit + dump
    3. Write results for Copilot to analyze
    4. Wait for next DFU entry (after reboot)
    
    Copilot can also write commands to COMMAND_FILE to request
    specific tests between DFU cycles.
    """
    log.info("=== WATCH MODE: Monitoring for DFU devices ===")
    log.info(f"Results dir: {RESULTS_DIR}")
    log.info(f"Command file: {COMMAND_FILE}")
    log.info("Put iPhone in DFU mode to start...")
    log.info("Press Ctrl+C to stop")
    print()
    
    write_result("watch_mode", STATUS_WAITING)
    
    last_seen = False
    test_count = 0
    
    try:
        while True:
            # Check for Copilot command
            cmd = read_command()
            if cmd:
                log.info(f"Received command: {cmd['command']}")
                handle_command(cmd)
                continue
            
            # Poll for DFU device
            dfu = DFUDevice()
            found = dfu.find_device()
            
            if found and not last_seen:
                # New device detected!
                test_count += 1
                cpid = f"0x{dfu.cpid:04X}" if dfu.cpid else "unknown"
                log.ok(f"[Test #{test_count}] DFU device detected! CPID: {cpid}")
                
                write_result("watch_detect", STATUS_CONNECTED, data={
                    "cpid": cpid,
                    "serial": dfu.serial or "N/A",
                    "test_number": test_count
                })
                
                if dfu.cpid == 0x8030:
                    # Auto-run: exploit + dump
                    log.info("A13 detected — running auto-test sequence...")
                    
                    try:
                        # Run exploit + dump
                        dump_bootrom_live()
                    except Exception as e:
                        log.error(f"Auto-test error: {e}")
                        log.error(traceback.format_exc())
                        write_result("auto_test", STATUS_ERROR, error=str(e))
                else:
                    log.warn(f"Device CPID {cpid} is not A13 (0x8030)")
                
                last_seen = True
                
            elif not found and last_seen:
                log.info("Device disconnected, waiting for next DFU entry...")
                write_result("watch_mode", STATUS_WAITING, data={
                    "tests_completed": test_count
                })
                last_seen = False
            
            elif not found:
                # Still waiting — show heartbeat every 30s
                sys.stdout.write(".")
                sys.stdout.flush()
            
            time.sleep(2)
    
    except KeyboardInterrupt:
        log.info("\nWatch mode stopped by user")
        write_result("watch_mode", STATUS_IDLE, data={
            "tests_completed": test_count
        })


def handle_command(cmd):
    """Handle a command written by Copilot to the command file."""
    command = cmd.get("command", "")
    args = cmd.get("args", {})
    
    if command == "probe":
        probe_device()
    
    elif command == "dump_bootrom":
        dump_bootrom_live()
    
    elif command == "read_memory":
        addr = int(args.get("address", "0"), 0)
        size = int(args.get("size", 256))
        read_physical_memory(addr, size)
    
    elif command == "exploit_only":
        run_exploit()
    
    elif command == "stop":
        log.info("Stop command received")
        sys.exit(0)
    
    else:
        log.warn(f"Unknown command: {command}")
        write_result("command", "error", error=f"Unknown: {command}")


# ============================================================================
# Quick Tests (no exploit needed — USB only)
# ============================================================================

def quick_usb_scan():
    """
    Scan all USB devices — useful for debugging driver issues.
    """
    log.info("=== USB SCAN: All Apple devices ===")
    
    if not HAS_USB:
        log.error("pyusb not available")
        return
    
    try:
        backend = usb.backend.libusb1.get_backend(
            find_library=libusb_package.find_library
        )
    except Exception:
        backend = None
    
    devices = []
    
    # Find all Apple devices
    for dev in usb.core.find(find_all=True, idVendor=APPLE_VID, backend=backend):
        info = {
            "vid": f"0x{dev.idVendor:04X}",
            "pid": f"0x{dev.idProduct:04X}",
            "bus": dev.bus,
            "address": dev.address,
        }
        
        try:
            info["serial"] = dev.serial_number
        except Exception:
            info["serial"] = "N/A"
        
        try:
            info["product"] = dev.product
        except Exception:
            info["product"] = "N/A"
        
        try:
            info["manufacturer"] = dev.manufacturer
        except Exception:
            info["manufacturer"] = "N/A"
        
        # Identify mode
        if dev.idProduct == 0x1227:
            info["mode"] = "DFU"
        elif dev.idProduct == 0x1281:
            info["mode"] = "Recovery"
        elif 0x12A0 <= dev.idProduct <= 0x12AF:
            info["mode"] = "Normal (USB)"
        else:
            info["mode"] = "Unknown"
        
        devices.append(info)
        log.info(f"  {info['vid']}:{info['pid']} — {info['mode']} — {info['product']}")
    
    if not devices:
        log.warn("No Apple devices found on USB")
        log.info("Possible causes:")
        log.info("  1. iPhone not connected")
        log.info("  2. USB cable issue (try USB-A port)")
        log.info("  3. Driver issue (run 'zadig' setup)")
    
    write_result("usb_scan", "success", data={
        "apple_devices": devices,
        "count": len(devices)
    })
    
    return devices


# ============================================================================
# Entry Point
# ============================================================================

def main():
    print()
    print("╔══════════════════════════════════════════════════════╗")
    print("║  TrustOS Live Test Harness                          ║")  
    print("║  Automated testing for iPhone 11 Pro (A13)          ║")
    print("║  Results → tools/checkm8-dualboot/results/          ║")
    print("╚══════════════════════════════════════════════════════╝")
    print()
    
    if len(sys.argv) < 2:
        # Interactive menu
        print("Commands:")
        print("  --probe        Detecter l'iPhone (pas d'exploit)")
        print("  --usb-scan     Scanner tous les peripheriques Apple USB")
        print("  --dump-rom     Dump complet du BootROM")
        print("  --read A S     Lire memoire physique (addr, size en hex)")
        print("  --test-all     Enchainer tous les tests")
        print("  --watch        Mode daemon (attend DFU, teste en boucle)")
        print()
        print("Le mode --watch est ideal pour les tests en boucle.")
        print("Copilot lit les resultats dans results/live_test_result.json")
        print("et peut envoyer des commandes via results/live_test_command.json")
        print()
        return
    
    cmd = sys.argv[1]
    
    try:
        if cmd == "--probe":
            probe_device()
        
        elif cmd == "--usb-scan":
            quick_usb_scan()
        
        elif cmd == "--dump-rom":
            dump_bootrom_live()
        
        elif cmd == "--read":
            if len(sys.argv) < 4:
                print("Usage: --read ADDRESS SIZE  (both in hex, e.g. 0x100000000 0x80000)")
                return
            addr = int(sys.argv[2], 0)
            size = int(sys.argv[3], 0)
            read_physical_memory(addr, size)
        
        elif cmd == "--test-all":
            log.info("=== FULL TEST SEQUENCE ===")
            quick_usb_scan()
            probe_device()
            dump_bootrom_live()
        
        elif cmd == "--watch":
            watch_mode()
        
        else:
            print(f"Unknown command: {cmd}")
            main()
    
    except KeyboardInterrupt:
        log.info("\nInterrupted by user")
    except Exception as e:
        log.error(f"Fatal error: {e}")
        log.error(traceback.format_exc())
        write_result("fatal", STATUS_ERROR, error=str(e))


if __name__ == "__main__":
    main()
