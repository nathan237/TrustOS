#!/usr/bin/env python3
"""Explore iBoot Recovery mode - send commands and read responses."""
from pymobiledevice3.irecv import IRecv

print("Connecting via IRecv...")
irecv = IRecv()
print(f"Mode: {irecv.mode}")
print(f"ECID: {irecv.ecid:#x}")

# Try getenv commands
cmds = [
    "getenv build-version",
    "getenv build-style",
    "getenv firmware-version", 
    "getenv platform-name",
    "getenv chip-id",
    "getenv board-id",
    "getenv security-domain",
    "getenv auto-boot",
    "getenv dark-boot",
    "getenv boot-args",
]

print("\n=== Environment Variables ===")
for cmd in cmds:
    try:
        irecv.send_command(cmd)
        # Response via vendor ctrl IN
        try:
            resp = irecv.ctrl_transfer(0xC0, 0, 0, 0, data_or_wLength=512)
            text = bytes(resp).rstrip(b"\x00").decode("utf-8", errors="replace").strip()
            if text:
                varname = cmd.split(" ", 1)[1]
                print(f"  {varname}: {text}")
            else:
                print(f"  {cmd.split(' ',1)[1]}: (empty)")
        except Exception:
            try:
                resp = irecv._device.read(0x81, 512, timeout=1000)
                text = bytes(resp).decode("utf-8", errors="replace").strip()
                print(f"  {cmd.split(' ',1)[1]}: {text}")
            except:
                print(f"  {cmd.split(' ',1)[1]}: (no response)")
    except Exception as e:
        print(f"  {cmd}: error - {e}")

print("\n=== Trying 'help' ===")
try:
    irecv.send_command("help")
    try:
        resp = irecv.ctrl_transfer(0xC0, 0, 0, 0, data_or_wLength=4096)
        print(bytes(resp).rstrip(b"\x00").decode("utf-8", errors="replace"))
    except:
        try:
            resp = irecv._device.read(0x81, 4096, timeout=2000)
            print(bytes(resp).decode("utf-8", errors="replace"))
        except:
            print("  (no response)")
except Exception as e:
    print(f"  error: {e}")
