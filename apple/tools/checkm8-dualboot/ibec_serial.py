#!/usr/bin/env python3
"""iBEC serial communication via bulk endpoints"""
import usb.core
import usb.util
import time
import sys

dev = usb.core.find(idVendor=0x05AC, idProduct=0x1281)
if not dev:
    print("No Recovery device found")
    sys.exit(1)

# Claim interface 1
try:
    dev.set_configuration()
except:
    pass

for i in range(2):
    try:
        usb.util.claim_interface(dev, i)
    except:
        pass

# Set alt setting 1 on interface 1
try:
    dev.set_interface_altsetting(1, 1)
except Exception as e:
    print(f"Alt setting: {e}")

EP_IN = 0x81
EP_OUT = 0x02
TIMEOUT = 500  # ms

# === Step 1: Drain all pending data from bulk IN ===
print("=== DRAINING BUFFER ===")
total_drained = b""
while True:
    try:
        data = dev.read(EP_IN, 16384, timeout=TIMEOUT)
        chunk = bytes(data)
        total_drained += chunk
        print(f"  Read {len(chunk)} bytes")
    except usb.core.USBTimeoutError:
        break
    except Exception as e:
        print(f"  Read error: {e}")
        break

if total_drained:
    print(f"\nTotal drained: {len(total_drained)} bytes")
    # Show as text
    try:
        text = total_drained.decode('utf-8', errors='replace')
        print("--- START BUFFER ---")
        print(text)
        print("--- END BUFFER ---")
    except:
        print(total_drained[:500])
else:
    print("  (buffer empty)")

# === Step 2: Try sending commands in different formats ===
print("\n=== SENDING COMMANDS ===")

def send_and_read(label, cmd_bytes, wait=0.3):
    print(f"\n--- {label} ---")
    print(f"  Sending: {cmd_bytes!r}")
    try:
        written = dev.write(EP_OUT, cmd_bytes, timeout=2000)
        print(f"  Wrote {written} bytes")
    except Exception as e:
        print(f"  Write error: {e}")
        return None
    
    time.sleep(wait)
    
    # Read response
    response = b""
    while True:
        try:
            data = dev.read(EP_IN, 16384, timeout=TIMEOUT)
            chunk = bytes(data)
            response += chunk
            print(f"  Read {len(chunk)} bytes")
        except usb.core.USBTimeoutError:
            break
        except Exception as e:
            print(f"  Read error: {e}")
            break
    
    if response:
        try:
            text = response.decode('utf-8', errors='replace')
            print(f"  Response: {text[:500]}")
        except:
            print(f"  Response (hex): {response[:200].hex()}")
    else:
        print("  (no response)")
    return response

# Try various command formats
send_and_read("Empty newline", b"\n")
send_and_read("help (newline)", b"help\n")
send_and_read("help (null)", b"help\x00")
send_and_read("help (CR+LF)", b"help\r\n")
send_and_read("getenv build-version (newline)", b"getenv build-version\n")
send_and_read("printenv (newline)", b"printenv\n")
send_and_read("bgcolor 0 255 0 (newline)", b"bgcolor 0 255 0\n")

# === Step 3: Try ctrl_transfer with TYPE_VENDOR | RECIP_INTERFACE ===
print("\n=== CTRL TRANSFER VARIANTS ===")
for bmReq in [0x40, 0x41, 0x42, 0x43, 0xC0, 0xC1, 0xC2, 0xC3]:
    for bReq in [0, 1, 2]:
        try:
            if bmReq & 0x80:  # IN
                ret = dev.ctrl_transfer(bmReq, bReq, 0, 0, 256, timeout=500)
                print(f"  bmReq=0x{bmReq:02X} bReq={bReq}: {len(ret)} bytes: {bytes(ret)[:50]}")
            else:  # OUT
                ret = dev.ctrl_transfer(bmReq, bReq, 0, 0, b"help\n", timeout=500)
                print(f"  bmReq=0x{bmReq:02X} bReq={bReq}: wrote {ret}")
        except Exception as e:
            if "Pipe" in str(e):
                pass  # Skip pipe errors silently
            else:
                print(f"  bmReq=0x{bmReq:02X} bReq={bReq}: {e}")

print("\nDone.")
