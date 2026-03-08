#!/usr/bin/env python3
"""Deep exploration of iBoot Recovery mode commands."""
import usb.core, usb.util, libusb_package, time

be = libusb_package.get_libusb1_backend()
dev = usb.core.find(idVendor=0x05AC, idProduct=0x1281, backend=be)
if not dev:
    print("No recovery device")
    exit(1)

dev.set_configuration(1)
try:
    usb.util.claim_interface(dev, 0)
except: pass
try:
    usb.util.claim_interface(dev, 1)
    dev.set_interface_altsetting(1, 1)
except: pass

def send_cmd(dev, cmd):
    """Send command, return (response_str, stalled)."""
    try:
        dev.ctrl_transfer(0x40, 0, 0, 0, cmd.encode() + b"\x00", timeout=5000)
    except usb.core.USBError as e:
        return f"(send error: {e})", True
    
    try:
        resp = dev.ctrl_transfer(0xC0, 0, 0, 0, 4096, timeout=3000)
        text = bytes(resp).rstrip(b"\x00").decode("utf-8", errors="replace").strip()
        return text, False
    except usb.core.USBError:
        return "", True

# Test many environment variables
print("=== ALL ENVIRONMENT VARIABLES ===")
envvars = [
    "build-version", "build-style", "auto-boot", "display-color-space",
    "display-timing", "idle-off", "debug-uarts", "debug-shmcon",
    "boot-device", "boot-partition", "boot-path", "loadaddr",
    "filesize", "bootdelay", "com.apple.System.boot-nonce",
    "obliteration", "effective-production-status-ap",
    "effective-security-mode-ap", "current-running-image",
    "usb-product-string", "device-material-color",
    "config_board", "boot-manifest-hash", "crypto-hash-method",
    "permit-random-gen-nonce", "allow-mix-and-match",
]

for var in envvars:
    val, stalled = send_cmd(dev, f"getenv {var}")
    if not stalled and val:
        print(f"  {var} = {val}")

# Test known iBoot commands 
print("\n=== COMMAND PROBING ===")
test_cmds = [
    ("version", "Get version string"),
    ("help", "List commands"),
    ("printenv", "Print all env vars"),
    ("devicetree", "Show device tree info"),
    ("meminfo", "Memory info"),
    ("mw 0 0 0", "Memory write test"),
    ("md 0x800000000 0x10", "Memory dump (try SRAM)"),
    ("md 0x100000000 0x10", "Memory dump (try ROM)"),
    ("go", "Execute loaded image"),
    ("reset", "Reset device (DON'T)"),
    ("reboot", "Reboot (DON'T)"),
]

for cmd, desc in test_cmds:
    if cmd in ("reset", "reboot", "go"):
        print(f"  [{cmd}] ({desc}) -- SKIPPED (destructive)")
        continue
    val, stalled = send_cmd(dev, cmd)
    if stalled:
        print(f"  [{cmd}] ({desc}) -- STALLED (not available)")
    else:
        # Truncate long output
        display = val[:200] if val else "(ok, empty)"
        print(f"  [{cmd}] ({desc}) => {display}")

# Try bgcolor to confirm visual control
print("\n=== VISUAL TEST ===")
print("Setting screen GREEN...")
send_cmd(dev, "bgcolor 0 255 0")
time.sleep(2)
print("Setting screen BLACK...")
send_cmd(dev, "bgcolor 0 0 0")

print("\nDone! Device is in iBSS Recovery mode, ready for iBEC.")
