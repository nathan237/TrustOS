#!/usr/bin/env python3
"""
Send iBSS then restart the USB port to trigger execution.
The problem: IRecv.send_buffer() transfers all data but the device.reset()
call doesn't actually cause the DFU firmware to process the image.
Solution: After sending, use Windows USB device restart to force bus re-enum.
"""
import usb.core, usb.util, usb.backend.libusb1
import time, sys, subprocess, os, struct, binascii, math

try:
    import libusb_package
    be = usb.backend.libusb1.get_backend(find_library=libusb_package.find_library)
except:
    be = None

CACHE = "tools/checkm8-dualboot/cache"

# ================================================================
# Step 1: Find DFU device
# ================================================================
print("Step 1: Finding DFU device...")
dev = usb.core.find(idVendor=0x05AC, idProduct=0x1227, backend=be)
if not dev:
    print("No DFU device!")
    sys.exit(1)

try:
    dev.set_configuration()
except:
    pass
try:
    usb.util.claim_interface(dev, 0)
except:
    pass

# Check state
st = dev.ctrl_transfer(0xA1, 3, 0, 0, 6, timeout=1000)
print(f"  DFU state: {st[4]} (need 2=dfuIDLE)")

if st[4] != 2:
    # Clear state
    print("  Clearing state...")
    try:
        dev.ctrl_transfer(0x21, 4, 0, 0, timeout=1000)  # CLRSTATUS
    except:
        pass
    try:
        dev.ctrl_transfer(0x21, 6, 0, 0, timeout=1000)  # ABORT
    except:
        pass
    time.sleep(1)
    # Reconnect
    dev = usb.core.find(idVendor=0x05AC, idProduct=0x1227, backend=be)
    if not dev:
        print("  Device lost during clear")
        sys.exit(1)
    try:
        dev.set_configuration()
    except:
        pass
    try:
        usb.util.claim_interface(dev, 0)
    except:
        pass
    st = dev.ctrl_transfer(0xA1, 3, 0, 0, 6, timeout=1000)
    print(f"  New state: {st[4]}")

# ================================================================
# Step 2: Send iBSS data using Apple DFU protocol with CRC32
# ================================================================
print("\nStep 2: Sending iBSS via Apple DFU protocol...")
with open(os.path.join(CACHE, "ibss_signed.img4"), "rb") as f:
    data = f.read()
print(f"  Data: {len(data)} bytes")

PACKET_SIZE = 2048  # DFU transfer size
num_packets = math.ceil(len(data) / PACKET_SIZE)
crc = -1

for offset in range(0, len(data), PACKET_SIZE):
    chunk = data[offset:offset + PACKET_SIZE]
    packet_index = offset // PACKET_SIZE

    if offset + PACKET_SIZE >= len(data):
        # Last packet - add CRC
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

    if packet_index % 100 == 0:
        pct = 100 * offset // len(data)
        print(f"  {pct}% ({offset}/{len(data)})")

print(f"  100% - all {num_packets} packets sent")

# ================================================================
# Step 3: Wait for status == 5 (dfuDNLOAD-IDLE)
# ================================================================
print("\nStep 3: Waiting for DFU status 5 (DNLOAD-IDLE)...")
for i in range(30):
    try:
        st = dev.ctrl_transfer(0xA1, 3, 0, 0, 6, timeout=1000)
        state = st[4]
        status = st[0]
        print(f"  Attempt {i}: state={state}, status={status}")
        if state == 5:
            break
        if state == 10:
            print("  ERROR state! Aborting.")
            sys.exit(1)
    except Exception as e:
        print(f"  Status read error: {e}")
        break
    time.sleep(0.5)
else:
    print("  Never reached status 5!")

# ================================================================
# Step 4: Send empty DNLOAD to finalize
# ================================================================
print("\nStep 4: Sending empty DNLOAD (finalize)...")
try:
    dev.ctrl_transfer(0x21, 1, num_packets, 0, b"", timeout=5000)
    print("  Empty DNLOAD sent")
except Exception as e:
    print(f"  {e}")

# Read status twice (as idevicerestore does)
for i in range(2):
    try:
        st = dev.ctrl_transfer(0xA1, 3, 0, 0, 6, timeout=2000)
        print(f"  Post-send status #{i}: state={st[4]}, status={st[0]}")
    except Exception as e:
        print(f"  Status #{i}: {e} (device may have disconnected)")

# ================================================================
# Step 5: Multiple reset approaches
# ================================================================
print("\nStep 5: Triggering USB reset...")

# Approach 1: pyusb reset
print("  5a: pyusb reset()...")
try:
    dev.reset()
    print("     Reset OK")
except Exception as e:
    print(f"     Reset: {e}")

time.sleep(2)

# Check if Recovery appeared
rec = usb.core.find(idVendor=0x05AC, idProduct=0x1281, backend=be)
if rec:
    print("\n*** RECOVERY MODE FOUND! ***")
    sys.exit(0)

# Approach 2: Release + re-find
print("  5b: Release interface + re-enumerate...")
try:
    usb.util.release_interface(dev, 0)
except:
    pass
try:
    usb.util.dispose_resources(dev)
except:
    pass

time.sleep(2)
rec = usb.core.find(idVendor=0x05AC, idProduct=0x1281, backend=be)
if rec:
    print("\n*** RECOVERY MODE FOUND! ***")
    sys.exit(0)

# Approach 3: Windows USB port restart via PowerShell
print("  5c: Windows USB port restart...")
ps_cmd = '''
$devices = Get-PnpDevice | Where-Object { $_.FriendlyName -like "*Apple*DFU*" -or $_.FriendlyName -like "*Apple Mobile*" -or $_.HardwareID -like "*VID_05AC*PID_1227*" }
foreach ($d in $devices) {
    Write-Host "Disabling: $($d.FriendlyName) ($($d.InstanceId))"
    Disable-PnpDevice -InstanceId $d.InstanceId -Confirm:$false -ErrorAction SilentlyContinue
    Start-Sleep -Seconds 1
    Write-Host "Enabling: $($d.FriendlyName)"
    Enable-PnpDevice -InstanceId $d.InstanceId -Confirm:$false -ErrorAction SilentlyContinue
}
'''
try:
    result = subprocess.run(
        ["powershell", "-Command", ps_cmd],
        capture_output=True, text=True, timeout=15
    )
    print(f"     stdout: {result.stdout.strip()}")
    if result.stderr:
        print(f"     stderr: {result.stderr.strip()[:200]}")
except Exception as e:
    print(f"     PS error: {e}")

time.sleep(5)

# Check again
print("\nStep 6: Final device check...")
for pid, name in [(0x1281, "RECOVERY"), (0x1227, "DFU")]:
    d = usb.core.find(idVendor=0x05AC, idProduct=pid, backend=be)
    if d:
        print(f"  Found: {name} (PID=0x{pid:04X})")
        if name == "RECOVERY":
            print("  *** SUCCESS! iBSS booted! ***")
            # Visual test
            try:
                d.set_configuration()
                usb.util.claim_interface(d, 0)
                d.ctrl_transfer(0x40, 0, 0, 0, b"bgcolor 255 0 0\x00", timeout=5000)
                print("  Sent bgcolor red - check screen!")
            except:
                pass
        elif name == "DFU":
            try:
                st2 = d.ctrl_transfer(0xA1, 3, 0, 0, 6, timeout=1000)
                print(f"  DFU state: {st2[4]}")
            except:
                pass
        break
else:
    print("  No Apple device found at all")

# Also check without specific backend
for pid in [0x1281, 0x1227]:
    d = usb.core.find(idVendor=0x05AC, idProduct=pid)
    if d:
        print(f"  (default backend) Found PID=0x{pid:04X} via {d.backend.__class__.__name__}")

print("\nDone.")
