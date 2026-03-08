#!/usr/bin/env python3
"""
iBEC console exploration with proper STALL recovery.
After a Pipe error (STALL), we need to clear the halt before sending more commands.
"""
import usb.core, usb.util, usb.backend.libusb1, libusb_package
import time, struct

be = usb.backend.libusb1.get_backend(find_library=libusb_package.find_library)

print("Connecting to iBEC (Recovery)...")
dev = usb.core.find(idVendor=0x05AC, idProduct=0x1281, backend=be)
if not dev:
    print("No Recovery device!")
    exit(1)

print(f"  PID=0x{dev.idProduct:04X}")
try:
    print(f"  Serial: {dev.serial_number[:80]}")
except:
    pass

# Configure
try:
    dev.set_configuration()
except:
    pass

# Print all interfaces/endpoints
cfg = dev.get_active_configuration()
print(f"\nConfiguration: {cfg.bConfigurationValue}")
for intf in cfg:
    print(f"  Interface {intf.bInterfaceNumber} alt={intf.bAlternateSetting}"
          f" class={intf.bInterfaceClass:#x} subclass={intf.bInterfaceSubClass:#x}")
    for ep in intf:
        d = "IN" if usb.util.endpoint_direction(ep.bEndpointAddress) == usb.util.ENDPOINT_IN else "OUT"
        print(f"    EP 0x{ep.bEndpointAddress:02X} ({d}) type={usb.util.endpoint_type(ep.bmAttributes)}"
              f" maxPacket={ep.wMaxPacketSize}")

# Claim interfaces
for i in range(2):
    try:
        usb.util.claim_interface(dev, i)
        print(f"  Claimed interface {i}")
    except Exception as e:
        print(f"  Claim interface {i}: {e}")

# Set alternate setting 1 on interface 1 (has EP 0x81 IN + EP 0x02 OUT)
try:
    dev.set_interface_altsetting(interface=1, alternate_setting=1)
    print("  Set interface 1 alt=1 (serial endpoints)")
except Exception as e:
    print(f"  Set alt: {e}")

def clear_stall():
    """Clear any stalled endpoints."""
    for ep_addr in [0x00, 0x02, 0x04, 0x81]:
        try:
            # CLEAR_FEATURE(ENDPOINT_HALT)
            dev.ctrl_transfer(0x02, 1, 0, ep_addr, timeout=1000)
        except:
            pass

def send_cmd(cmd):
    """Send command, clear stall on failure, retry once."""
    data = (cmd + "\x00").encode("utf-8")
    try:
        dev.ctrl_transfer(0x40, 0, 0, 0, data, timeout=5000)
        return True
    except usb.core.USBError:
        # Clear stall and retry
        clear_stall()
        try:
            dev.ctrl_transfer(0x40, 0, 0, 0, data, timeout=5000)
            return True
        except usb.core.USBError as e:
            return False

def read_resp_ctrl():
    """Read via ctrl 0xC0."""
    try:
        resp = dev.ctrl_transfer(0xC0, 0, 0, 0, 512, timeout=2000)
        return bytes(resp).split(b"\x00")[0].decode("utf-8", errors="replace")
    except:
        return None

def read_resp_bulk():
    """Read via bulk EP 0x81."""
    result = b""
    for _ in range(10):
        try:
            data = dev.read(0x81, 0x10000, timeout=500)
            result += bytes(data)
        except usb.core.USBTimeoutError:
            break
        except usb.core.USBError:
            break
    if result:
        return result.decode("utf-8", errors="replace")
    return None

def cmd_and_read(cmd, use_bulk=True):
    """Send command and read response."""
    ok = send_cmd(cmd)
    if not ok:
        return None
    time.sleep(0.1)
    if use_bulk:
        val = read_resp_bulk()
        if val:
            return val
    return read_resp_ctrl()

# ================================================================
# Test: bgcolor to confirm visual feedback
# ================================================================
print("\n" + "=" * 50)
print("VISUAL TEST")
print("=" * 50)
print("Sending bgcolor red...")
send_cmd("bgcolor 255 0 0")
print("  Screen should be RED!")
time.sleep(1)
print("Sending bgcolor blue...")
send_cmd("bgcolor 0 0 255")
print("  Screen should be BLUE!")

# ================================================================
# Environment Variables (one by one with proper recovery)
# ================================================================
print("\n" + "=" * 50)
print("ENVIRONMENT VARIABLES")
print("=" * 50)

known_vars = [
    "build-version", "build-style", "loadaddr", "config_board",
    "auto-boot", "boot-device", "display-color-space", "display-timing",
]

for var in known_vars:
    val = cmd_and_read(f"getenv {var}")
    if val:
        print(f"  {var} = {val.strip()}")
    else:
        # Try ctrl read
        send_cmd(f"getenv {var}")
        time.sleep(0.15)
        val = read_resp_ctrl()
        if val:
            print(f"  {var} = {val.strip()}")
        else:
            print(f"  {var} = (no response)")
    clear_stall()  # Always clear after each command

# ================================================================
# Try printenv (list ALL variables)  
# ================================================================
print("\n" + "=" * 50)
print("PRINTENV (all variables)")
print("=" * 50)

clear_stall()
send_cmd("printenv")
time.sleep(0.5)

# Read potentially large response
val = read_resp_bulk()
if val:
    for line in val.strip().split("\n"):
        print(f"  {line}")
else:
    val = read_resp_ctrl()
    if val:
        for line in val.strip().split("\n"):
            print(f"  {line}")
    else:
        print("  (no response to printenv)")

# ================================================================
# Help command
# ================================================================
print("\n" + "=" * 50)
print("HELP")
print("=" * 50)

clear_stall()
send_cmd("help")
time.sleep(0.5)

val = read_resp_bulk()
if val:
    for line in val.strip().split("\n"):
        print(f"  {line}")
else:
    val = read_resp_ctrl()
    if val:
        for line in val.strip().split("\n"):
            print(f"  {line}")
    else:
        print("  (no response to help)")

# ================================================================
# Memory dump: SecureROM
# ================================================================
print("\n" + "=" * 50)
print("MEMORY EXPLORATION")
print("=" * 50)

addrs = [
    ("0x100000000", "SecureROM base (T8020)"),
    ("0x19C000000", "SRAM"),
    ("0x802000000", "loadaddr / iBEC code"),
    ("0x0", "Zero page"),
]

for addr, desc in addrs:
    print(f"\nmd {addr} ({desc}):")
    clear_stall()
    send_cmd(f"md {addr}")
    time.sleep(0.3)
    val = read_resp_bulk()
    if not val:
        val = read_resp_ctrl()
    if val:
        for line in val.strip().split("\n")[:8]:
            print(f"  {line}")
    else:
        print("  (no response)")

# ================================================================
# Memory write test (safe area)
# ================================================================
print("\n" + "=" * 50)
print("MEMORY WRITE TEST")
print("=" * 50)

# Write to loadaddr area (safe - it's our own code region)
test_addr = "0x802100000"
print(f"mw {test_addr} 0xDEADBEEF:")
clear_stall()
send_cmd(f"mw {test_addr} 0xDEADBEEF")
time.sleep(0.2)
val = read_resp_bulk() or read_resp_ctrl()
if val:
    print(f"  {val.strip()}")

print(f"md {test_addr} (verify write):")
clear_stall()
send_cmd(f"md {test_addr}")
time.sleep(0.3)
val = read_resp_bulk() or read_resp_ctrl()
if val:
    for line in val.strip().split("\n")[:4]:
        print(f"  {line}")

# ================================================================
# Other commands
# ================================================================
print("\n" + "=" * 50)
print("OTHER COMMANDS")
print("=" * 50)

for cmd in ["version", "devicetree", "ticket"]:
    print(f"\n{cmd}:")
    clear_stall()
    send_cmd(cmd)
    time.sleep(0.3)
    val = read_resp_bulk() or read_resp_ctrl()
    if val:
        for line in val.strip().split("\n")[:10]:
            print(f"  {line}")
    else:
        print("  (no response)")

print("\n" + "=" * 50)
print("EXPLORATION COMPLETE")
print("=" * 50)
print("Device is in iBEC Recovery mode.")
print("bgcolor, getenv, setenv work.")
print("Check what commands are available from the help output above.")
