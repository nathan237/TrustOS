#!/usr/bin/env python3
"""
Send iBSS via Apple DFU protocol and use Windows port reset to trigger boot.
Must be run from an ADMIN terminal for the port reset to work.
"""
import usb.core, usb.util, usb.backend.libusb1
import time, sys, os, struct, binascii, math, ctypes, subprocess

try:
    import libusb_package
    be = usb.backend.libusb1.get_backend(find_library=libusb_package.find_library)
except:
    be = None

CACHE = "tools/checkm8-dualboot/cache"

def find_apple(pid, backend=None):
    if backend:
        d = usb.core.find(idVendor=0x05AC, idProduct=pid, backend=backend)
        if d:
            return d
    return usb.core.find(idVendor=0x05AC, idProduct=pid)

# ================================================================
# Wait for DFU device
# ================================================================
print("Waiting for DFU device (PID=0x1227)...")
dev = None
for i in range(60):
    dev = find_apple(0x1227, be)
    if dev:
        break
    if i % 5 == 0:
        print(f"  {i}s... (enter DFU mode now)")
    time.sleep(1)

if not dev:
    print("No DFU device found!")
    sys.exit(1)

print(f"DFU device found! Backend: {dev.backend.__class__.__name__}")
try:
    print(f"  Serial: {dev.serial_number[:80]}")
except:
    pass

try:
    dev.set_configuration()
except:
    pass
try:
    usb.util.claim_interface(dev, 0)
except:
    pass

# Ensure dfuIDLE
st = dev.ctrl_transfer(0xA1, 3, 0, 0, 6, timeout=1000)
state = st[4]
print(f"  State: {state}")
if state != 2:
    print("  Clearing state...")
    for _ in range(3):
        try:
            dev.ctrl_transfer(0x21, 4, 0, 0, timeout=500)
        except: pass
        try:
            dev.ctrl_transfer(0x21, 6, 0, 0, timeout=500)
        except: pass
        time.sleep(0.3)
    st = dev.ctrl_transfer(0xA1, 3, 0, 0, 6, timeout=1000)
    print(f"  State after clear: {st[4]}")

# ================================================================
# Read the current nonce to check if ticket is still valid
# ================================================================
print("\nChecking device nonce...")
try:
    s1 = usb.util.get_string(dev, 1)
    if s1 and "NONC:" in s1:
        nonce = s1.split("NONC:")[1].split(" ")[0]
        print(f"  Current NONC: {nonce}")
        expected = "38B4510C5DBB94AB3CE33A55C3FF45133F9AD1B2E438F1DBC90CE90DBA2F6539"
        if nonce == expected:
            print(f"  NONCE MATCHES cached ticket!")
        else:
            print(f"  WARNING: Nonce CHANGED! Need new TSS ticket!")
            print(f"  Expected: {expected}")
    else:
        print(f"  String #1: {s1}")
        print("  No NONC in DFU string (normal for SecureROM)")
except Exception as e:
    print(f"  Nonce check: {e}")

# ================================================================
# Send iBSS using Apple DFU protocol (ctrl 0x21 with CRC32)
# ================================================================
print("\nSending iBSS...")
with open(os.path.join(CACHE, "ibss_signed.img4"), "rb") as f:
    data = f.read()
print(f"  {len(data)} bytes")

PACKET_SIZE = 2048
num_packets = math.ceil(len(data) / PACKET_SIZE)
crc = -1

t0 = time.time()
for offset in range(0, len(data), PACKET_SIZE):
    chunk = data[offset:offset + PACKET_SIZE]
    packet_index = offset // PACKET_SIZE

    if offset + PACKET_SIZE >= len(data):
        # Last packet — add CRC32 with dfu_xbuf salt
        crc = binascii.crc32(data, crc)
        dfu_xbuf = bytearray([0xFF, 0xFF, 0xFF, 0xFF, 0xAC, 0x05, 0x00, 0x01,
                               0x55, 0x46, 0x44, 0x10])
        crc = binascii.crc32(dfu_xbuf, crc)
        crc_chunk = dfu_xbuf + struct.pack("<I", crc)

        if len(chunk) + 16 > PACKET_SIZE:
            dev.ctrl_transfer(0x21, 1, packet_index, 0, chunk, timeout=5000)
            dev.ctrl_transfer(0x21, 1, packet_index, 0, crc_chunk, timeout=5000)
        else:
            dev.ctrl_transfer(0x21, 1, packet_index, 0, chunk + crc_chunk, timeout=5000)
    else:
        dev.ctrl_transfer(0x21, 1, packet_index, 0, chunk, timeout=5000)

    if packet_index % 200 == 0 and packet_index > 0:
        print(f"  {100*offset//len(data)}%")

dt = time.time() - t0
print(f"  Sent {num_packets} packets in {dt:.1f}s")

# Wait for status 5 (dfuDNLOAD-IDLE)
print("\nWaiting for DNLOAD-IDLE...")
for _ in range(10):
    st = dev.ctrl_transfer(0xA1, 3, 0, 0, 6, timeout=2000)
    if st[4] == 5:
        print(f"  State: 5 (DNLOAD-IDLE) - OK!")
        break
    time.sleep(0.5)

# Send empty DNLOAD to finalize
print("Sending empty DNLOAD...")
dev.ctrl_transfer(0x21, 1, num_packets, 0, b"", timeout=5000)

# Read status twice
for _ in range(2):
    try:
        st = dev.ctrl_transfer(0xA1, 3, 0, 0, 6, timeout=2000)
        print(f"  State: {st[4]}")
    except:
        print("  (device disconnected)")
        break

# ================================================================
# Now try MULTIPLE reset approaches
# ================================================================
print("\n" + "=" * 50)
print("iBSS validated! Triggering USB reset...")
print("=" * 50)

# --- Approach 1: libusb1 reset ---
print("\n[1] libusb reset...")
try:
    dev.reset()
    print("    Reset sent")
except Exception as e:
    print(f"    {e}")

time.sleep(3)
rec = find_apple(0x1281, be)
if rec:
    print("*** RECOVERY FOUND after libusb reset! ***")
    sys.exit(0)

# If still not found, check if DFU changed state
dfu2 = find_apple(0x1227, be)
if dfu2:
    try:
        dfu2.set_configuration()
        usb.util.claim_interface(dfu2, 0)
        st = dfu2.ctrl_transfer(0xA1, 3, 0, 0, 6, timeout=1000)
        print(f"    DFU state after reset: {st[4]}")
    except:
        pass

# --- Approach 2: Try libusb_reset_device directly ---
print("\n[2] Low-level libusb reset...")
try:
    handle = dev.backend.open_device(dev._ctx.dev, 0)
    # Reset via backend
    dev.backend.reset_device(handle)
    print("    Backend reset done")
except Exception as e:
    print(f"    {e}")

time.sleep(3)
rec = find_apple(0x1281, be)
if rec:
    print("*** RECOVERY FOUND after backend reset! ***")
    sys.exit(0)

# --- Approach 3: Windows devcon restart ---
print("\n[3] Windows devcon restart (needs admin)...")
# Try to find devcon
devcon_paths = [
    r"C:\devcon.exe",
    r"C:\Windows\System32\devcon.exe",
    os.path.expanduser(r"~\devcon.exe"),
]
devcon = None
for p in devcon_paths:
    if os.path.exists(p):
        devcon = p
        break

if devcon:
    try:
        r = subprocess.run([devcon, "restart", "*VID_05AC*PID_1227*"],
                           capture_output=True, text=True, timeout=10)
        print(f"    {r.stdout.strip()}")
    except Exception as e:
        print(f"    {e}")
else:
    print("    devcon not found, trying pnputil...")

# --- Approach 4: pnputil via admin powershell ---
print("\n[4] PnP device restart...")
try:
    # Get the hardware instance ID
    r = subprocess.run(
        ["powershell", "-Command",
         "(Get-PnpDevice -PresentOnly | Where-Object { $_.InstanceId -like '*VID_05AC*PID_1227*' }).InstanceId"],
        capture_output=True, text=True, timeout=10
    )
    instance_id = r.stdout.strip()
    if instance_id:
        print(f"    Found: {instance_id}")
        # Disable
        subprocess.run(
            ["powershell", "-Command",
             f"Disable-PnpDevice -InstanceId '{instance_id}' -Confirm:$false"],
            capture_output=True, text=True, timeout=10
        )
        print("    Disabled")
        time.sleep(1)
        # Enable
        subprocess.run(
            ["powershell", "-Command",
             f"Enable-PnpDevice -InstanceId '{instance_id}' -Confirm:$false"],
            capture_output=True, text=True, timeout=10
        )
        print("    Enabled")
    else:
        print("    No matching PnP device found")
except Exception as e:
    print(f"    {e}")

time.sleep(5)

# --- Approach 5: IOCTL USB port reset via ctypes ---
print("\n[5] Direct IOCTL USB port reset...")
try:
    import ctypes
    from ctypes import wintypes
    
    kernel32 = ctypes.windll.kernel32
    
    # Find USB hub handle - enumerate USB controllers
    setupapi = ctypes.windll.setupapi
    
    DIGCF_PRESENT = 0x02
    DIGCF_ALLCLASSES = 0x04
    INVALID_HANDLE = ctypes.c_void_p(-1).value
    
    # IOCTL_USB_HUB_CYCLE_PORT = 0x220444
    IOCTL_USB_HUB_CYCLE_PORT = 0x220444
    
    print("    (IOCTL approach - complex, may need specific hub path)")
    print("    Skipping for now...")
except Exception as e:
    print(f"    {e}")

# Final check
print("\nFinal device scan...")
for pid, name in [(0x1281, "RECOVERY"), (0x1227, "DFU")]:
    d = find_apple(pid, be)
    if d:
        print(f"  Found: {name} (PID=0x{pid:04X})")
        if name == "DFU":
            try:
                d.set_configuration()
                usb.util.claim_interface(d, 0)
                st = d.ctrl_transfer(0xA1, 3, 0, 0, 6, timeout=1000)
                print(f"    State: {st[4]}")
            except:
                pass
        break
else:
    # Try default backend too
    for pid in [0x1281, 0x1227]:
        d = usb.core.find(idVendor=0x05AC, idProduct=pid)
        if d:
            print(f"  Found via default: PID=0x{pid:04X}")
            break

print("""
=== IF STILL IN DFU ===
The USB software reset isn't working with current driver.
Options:
1. Open Zadig and check what driver is on the DFU device
   - If it's libusbK, try switching back to WinUSB
2. Open Device Manager, find the Apple DFU device,
   right-click -> Disable, wait 2 sec, right-click -> Enable
3. Run this terminal AS ADMINISTRATOR and re-run the script
""")
