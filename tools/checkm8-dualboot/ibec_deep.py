#!/usr/bin/env python3
"""
Deep iBEC exploration - drain serial, test each command individually.
RELEASE iBECs have limited commands but we need to find what's available.
"""
import usb.core, usb.util, usb.backend.libusb1, libusb_package
import time

be = usb.backend.libusb1.get_backend(find_library=libusb_package.find_library)

dev = usb.core.find(idVendor=0x05AC, idProduct=0x1281, backend=be)
if not dev:
    print("No Recovery device!")
    exit(1)

print(f"Connected: PID=0x{dev.idProduct:04X}")
try:
    dev.set_configuration()
except: pass
try:
    usb.util.claim_interface(dev, 0)
    usb.util.claim_interface(dev, 1)
except: pass
try:
    dev.set_interface_altsetting(interface=1, alternate_setting=1)
except: pass

def drain_serial(timeout=300):
    """Drain all pending serial data."""
    total = 0
    while True:
        try:
            data = dev.read(0x81, 0x10000, timeout=timeout)
            total += len(data)
        except:
            break
    return total

def send_cmd(cmd):
    """Send command via ctrl 0x40."""
    data = (cmd + "\x00").encode("utf-8")
    try:
        dev.ctrl_transfer(0x40, 0, 0, 0, data, timeout=5000)
        return True
    except usb.core.USBError:
        # Clear stall and retry
        for ep in [0x00, 0x02, 0x04, 0x81]:
            try:
                dev.ctrl_transfer(0x02, 1, 0, ep, timeout=500)
            except: pass
        try:
            dev.ctrl_transfer(0x40, 0, 0, 0, data, timeout=5000)
            return True
        except:
            return False

def read_ctrl():
    """Read ctrl 0xC0 response."""
    try:
        resp = dev.ctrl_transfer(0xC0, 0, 0, 0, 0x2000, timeout=2000)
        return bytes(resp).split(b"\x00")[0].decode("utf-8", errors="replace")
    except:
        return None

def read_serial_after_cmd(wait=0.3, timeout=300):
    """Read serial output that appeared after sending a command."""
    time.sleep(wait)
    result = b""
    while True:
        try:
            data = dev.read(0x81, 0x10000, timeout=timeout)
            result += bytes(data)
        except:
            break
    return result.decode("utf-8", errors="replace") if result else ""

def test_command(cmd, description=""):
    """Send command, read both ctrl and serial responses."""
    drain_serial(100)  # Drain buffer first
    time.sleep(0.05)
    
    ok = send_cmd(cmd)
    status = "OK" if ok else "STALL"
    
    time.sleep(0.2)
    
    # Read ctrl response
    ctrl_val = read_ctrl()
    
    # Read new serial output
    serial_val = read_serial_after_cmd(0.1, 200)
    # Filter out common debug hash spam
    serial_lines = [l for l in serial_val.strip().split("\n") 
                    if l.strip() and "ea0f64a4253252:946" not in l
                    and "9905b4edc794469" not in l]
    
    result = f"[{status}]"
    if ctrl_val:
        result += f" ctrl={ctrl_val}"
    if serial_lines:
        result += f" serial={'; '.join(serial_lines[:5])}"
    
    desc = f" ({description})" if description else ""
    print(f"  {cmd}{desc}: {result}")
    return ctrl_val, serial_lines

# =============================================
print("=" * 60)
print("iBEC DEEP EXPLORATION")
print("=" * 60)

# Drain initial buffer
drained = drain_serial()
print(f"Drained {drained} bytes of buffered serial data\n")

# =============================================
# 1. All getenv variables we can find
# =============================================
print("--- ENVIRONMENT VARIABLES ---")
env_vars = [
    "build-version", "build-style", "loadaddr", "config_board",
    "auto-boot", "boot-device", "display-color-space", "display-timing",
    "platform-name", "chip-id", "board-id", "security-domain",
    "security-epoch", "production-mode", "debug-enabled",
    "usb-enabled", "dfu-loop-count", "boot-command",
    "boot-args", "idle-off", "boot-partition", "boot-path",
    "firmware-version", "region-info", "backlight-level",
    "com.apple.System.boot-nonce", "boot-manifest-hash",
    "current-trust-cache", "allow-mix-and-match",
    "effective-production-status-ap", "effective-security-mode-ap",
    "crypto-hash-method", "storage-type", "display-rotation",
    "device-color", "display-scale", "content-protect",
    "pwr-path", "voltage-states5-sram",
]

for var in env_vars:
    send_cmd(f"getenv {var}")
    time.sleep(0.08)
    val = read_ctrl()
    if val:
        print(f"  {var} = {val}")
    # Clear any stall
    for ep in [0x00]:
        try: dev.ctrl_transfer(0x02, 1, 0, ep, timeout=200)
        except: pass

# =============================================
# 2. Test all known iBoot/iBEC commands
# =============================================
print("\n--- COMMAND TESTING ---")
commands = [
    ("help", "List available commands"),
    ("?", "Help alias"),
    ("printenv", "Print all env vars"),
    ("version", "Show version"),
    ("bgcolor 128 0 255", "Set purple bg"),
    ("reboot", "DO NOT SEND"),
    ("reset", "Reset device"),
    ("fsboot", "Filesystem boot"),
    ("upgrade", "Trigger upgrade mode"),
    ("meminfo", "Memory info"),
    ("md 0x100000000 64", "SecureROM dump"),
    ("md 0x802000000 64", "loadaddr dump"),
    ("mw 0x802100000 0xDEADBEEF", "Memory write test"),
    ("go", "Execute at loadaddr"),
    ("jump", "Jump to address"),
    ("diags", "Enter diagnostics mode"),
    ("bootx", "Boot kernel"),
    ("ramdisk", "Load ramdisk"),
    ("ticket", "Show ticket info"),
    ("clearenv", "DO NOT SEND"),
    ("getenv build-version", "Verify getenv still works"),
]

for cmd, desc in commands:
    if "DO NOT SEND" in desc:
        print(f"  {cmd} ({desc}): SKIPPED")
        continue
    test_command(cmd, desc)

# =============================================
# 3. Try ctrl transfers with different bRequest values
# =============================================
print("\n--- CTRL TRANSFER PROBING ---")
for breq in range(8):
    for bmreq in [0x40, 0x41, 0xC0, 0xC1]:
        try:
            if bmreq & 0x80:  # IN
                resp = dev.ctrl_transfer(bmreq, breq, 0, 0, 512, timeout=500)
                val = bytes(resp)
                text = val.split(b"\x00")[0].decode("utf-8", errors="replace")
                if val and len(val) > 0:
                    print(f"  bmReq=0x{bmreq:02X} bReq={breq}: {len(val)} bytes"
                          f" = {text[:60]}")
            else:  # OUT
                dev.ctrl_transfer(bmreq, breq, 0, 0, b"test\x00", timeout=500)
                print(f"  bmReq=0x{bmreq:02X} bReq={breq}: OK (sent 'test')")
        except usb.core.USBError as e:
            if "Pipe" in str(e):
                pass  # STALL - command not supported
            elif "Timeout" in str(e):
                pass
            else:
                print(f"  bmReq=0x{bmreq:02X} bReq={breq}: {e}")

# =============================================
# 4. Try sending bulk commands via EP 0x02
# =============================================
print("\n--- BULK EP 0x02 COMMAND TESTING ---")
for cmd in ["help\n", "printenv\n", "version\n"]:
    drain_serial(100)
    try:
        dev.write(0x02, cmd.encode(), timeout=2000)
        time.sleep(0.3)
        resp = read_serial_after_cmd(0.1, 200)
        lines = [l for l in resp.strip().split("\n") 
                 if l.strip() and "ea0f64a" not in l]
        if lines:
            print(f"  EP0x02 '{cmd.strip()}': {'; '.join(lines[:5])}")
        else:
            print(f"  EP0x02 '{cmd.strip()}': (no new output)")
    except Exception as e:
        print(f"  EP0x02 '{cmd.strip()}': {e}")

print("\n" + "=" * 60)
print("EXPLORATION COMPLETE")
print("=" * 60)
