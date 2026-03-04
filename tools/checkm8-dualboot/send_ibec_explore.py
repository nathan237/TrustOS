#!/usr/bin/env python3
"""
Send iBEC to iBSS via bulk EP 0x04 and explore the iBEC console.
Device must already be in Recovery mode running iBSS with auto-boot=false.
"""
import usb.core, usb.util, usb.backend.libusb1, libusb_package
import time, os, sys

be = usb.backend.libusb1.get_backend(find_library=libusb_package.find_library)
CACHE = "tools/checkm8-dualboot/cache"

# ================================================================
# Connect to iBSS in Recovery mode
# ================================================================
print("Connecting to iBSS (Recovery)...")
dev = usb.core.find(idVendor=0x05AC, idProduct=0x1281, backend=be)
if not dev:
    print("No Recovery device found!")
    sys.exit(1)

print(f"  PID=0x{dev.idProduct:04X}")
try:
    print(f"  Serial: {dev.serial_number[:60]}")
except:
    pass

try:
    dev.set_configuration()
except:
    pass

# Claim interface 0 (DFU/bulk endpoint)
try:
    usb.util.claim_interface(dev, 0)
except:
    pass

# ================================================================
# Send iBEC via bulk EP 0x04
# ================================================================
print("\nLoading iBEC...")
ibec_path = os.path.join(CACHE, "ibec_signed.img4")
with open(ibec_path, "rb") as f:
    ibec_data = f.read()
print(f"  {len(ibec_data)} bytes")

# First, initiate upload via ctrl 0x41 (FILE descriptor)
print("Sending FILE init (ctrl 0x41)...")
try:
    dev.ctrl_transfer(0x41, 0, 0, 0, b"", timeout=5000)
    print("  OK")
except Exception as e:
    print(f"  {e} (may be OK)")

# Send data in 512-byte chunks via bulk EP 0x04 (matches endpoint wMaxPacketSize)
CHUNK_SIZE = 0x8000  # 32KB chunks for speed
ep_out = 0x04
total = len(ibec_data)
sent = 0
t0 = time.time()

print(f"Sending iBEC via bulk EP 0x04...")
while sent < total:
    chunk = ibec_data[sent:sent + CHUNK_SIZE]
    try:
        written = dev.write(ep_out, chunk, timeout=10000)
        sent += written
    except usb.core.USBError as e:
        print(f"\n  Write error at {sent}/{total}: {e}")
        break
    
    pct = 100 * sent // total
    if (sent // CHUNK_SIZE) % 10 == 0 or sent >= total:
        dt = time.time() - t0
        rate = sent / dt / 1024 if dt > 0 else 0
        print(f"  {pct}% ({sent}/{total}) {rate:.0f} KB/s")

dt = time.time() - t0
print(f"  Sent {sent} bytes in {dt:.1f}s")

# Finalize - send ctrl 0x41 bReq=2 (execute)
print("Finalizing (ctrl execute)...")
try:
    dev.ctrl_transfer(0x41, 2, 0, 0, b"", timeout=5000)
    print("  Execute sent")
except usb.core.USBError as e:
    # This often disconnects the device
    print(f"  {e} (device may have rebooted)")

# ================================================================
# Wait for iBEC to appear in Recovery mode
# ================================================================
print("\nWaiting for iBEC Recovery mode...")
time.sleep(3)

for attempt in range(30):
    dev2 = usb.core.find(idVendor=0x05AC, idProduct=0x1281, backend=be)
    if dev2:
        try:
            sn = dev2.serial_number
            print(f"  Found! Serial: {sn[:80]}")
        except:
            print(f"  Found PID=0x1281")
        break
    if attempt % 5 == 0:
        print(f"  {attempt}s...")
    time.sleep(1)
else:
    print("  No Recovery device found after 30s")
    # Check for DFU
    dfu = usb.core.find(idVendor=0x05AC, idProduct=0x1227, backend=be)
    if dfu:
        print("  WARNING: Device fell back to DFU mode!")
    sys.exit(1)

# ================================================================
# Explore iBEC console
# ================================================================
print("\n" + "=" * 60)
print("iBEC CONSOLE EXPLORATION")
print("=" * 60)

dev2 = usb.core.find(idVendor=0x05AC, idProduct=0x1281, backend=be)
if not dev2:
    print("Lost device!")
    sys.exit(1)

try:
    dev2.set_configuration()
except:
    pass
try:
    usb.util.claim_interface(dev2, 0)
except:
    pass
try:
    usb.util.claim_interface(dev2, 1)
except:
    pass

def send_cmd(cmd):
    """Send command via ctrl 0x40."""
    data = (cmd + "\x00").encode("utf-8")
    try:
        dev2.ctrl_transfer(0x40, 0, 0, 0, data, timeout=5000)
        return True
    except Exception as e:
        print(f"  cmd error ({cmd}): {e}")
        return False

def read_response():
    """Read response from bulk EP 0x81 or ctrl 0xC0."""
    # Try bulk first (iBEC uses Interface 1, EP 0x81)
    try:
        resp = dev2.read(0x81, 0x10000, timeout=2000)
        return bytes(resp).decode("utf-8", errors="replace")
    except:
        pass
    # Fallback to ctrl
    try:
        resp = dev2.ctrl_transfer(0xC0, 0, 0, 0, 512, timeout=2000)
        return bytes(resp).split(b"\x00")[0].decode("utf-8", errors="replace")
    except:
        return None

# Test bgcolor blue
print("\nTest: bgcolor blue")
send_cmd("bgcolor 0 0 255")
print("  Check screen - should be BLUE!")

# Get env vars
print("\nEnvironment variables:")
env_vars = [
    "build-version", "build-style", "loadaddr", "config_board",
    "auto-boot", "boot-device", "display-color-space", "display-timing",
    "firmware-version", "platform-name", "region-info",
    "usb-enabled", "debug-enabled", "security-domain",
    "chip-epoch", "chip-id", "board-id",
]

for var in env_vars:
    send_cmd(f"getenv {var}")
    time.sleep(0.15)
    val = read_response()
    if val:
        print(f"  {var} = {val}")

# Try printenv for ALL variables
print("\n--- printenv (all variables) ---")
send_cmd("printenv")
time.sleep(0.5)
val = read_response()
if val:
    for line in val.split("\n")[:50]:
        print(f"  {line}")

# Try help command
print("\n--- help ---")
send_cmd("help")
time.sleep(0.5)
val = read_response()
if val:
    for line in val.split("\n")[:50]:
        print(f"  {line}")

# Memory commands
print("\n--- Memory exploration ---")
# Try reading SecureROM base (0x100000000 on T8020)
print("md 0x100000000 (SecureROM base):")
send_cmd("md 0x100000000")
time.sleep(0.3)
val = read_response()
if val:
    for line in val.split("\n")[:10]:
        print(f"  {line}")

# Try reading SRAM
print("\nmd 0x19C000000 (SRAM):")
send_cmd("md 0x19C000000")
time.sleep(0.3)
val = read_response()
if val:
    for line in val.split("\n")[:10]:
        print(f"  {line}")

# Try reading load address area
print("\nmd 0x802000000 (loadaddr):")
send_cmd("md 0x802000000")
time.sleep(0.3)
val = read_response()
if val:
    for line in val.split("\n")[:10]:
        print(f"  {line}")

# Device tree and other interesting commands
print("\n--- Other commands ---")
for cmd in ["devicetree", "version", "reboot"]:
    if cmd == "reboot":
        continue  # Don't reboot!
    print(f"\n{cmd}:")
    send_cmd(cmd)
    time.sleep(0.3)
    val = read_response()
    if val:
        for line in val.split("\n")[:10]:
            print(f"  {line}")

print("\n*** iBEC EXPLORATION COMPLETE ***")
print("Device is in iBEC Recovery mode with auto-boot=false.")
print("You can interact with it via USB commands.")
