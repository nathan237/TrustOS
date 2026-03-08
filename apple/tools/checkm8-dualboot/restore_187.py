#!/usr/bin/env python3
"""
restore_187.py - Restore iPhone XR to iOS 18.7.5 via Recovery Mode
Patches USB backend to work with libusb_package on Windows.
"""
import sys
import os
import asyncio
import traceback
import zipfile

# MUST patch USB backend BEFORE importing pymobiledevice3
import libusb_package
import usb.core
import usb.util

_original_find = usb.core.find
def _patched_find(**kwargs):
    kwargs.setdefault('backend', libusb_package.get_libusb1_backend())
    results = _original_find(**kwargs)
    if kwargs.get('find_all'):
        return [d for d in results if d.idVendor == 0x05AC]
    return results
usb.core.find = _patched_find

# Patch usb.util.get_string to handle langid errors on Windows.
# When the default langid detection fails, use raw control transfer
# with langid 0x0409 (English US).
_original_get_string = usb.util.get_string
def _patched_get_string(dev, index, langid=None):
    try:
        return _original_get_string(dev, index, langid)
    except (ValueError, usb.core.USBError, NotImplementedError):
        pass
    # Fallback: raw control transfer with explicit langid
    try:
        buf = dev.ctrl_transfer(
            0x80,   # bmRequestType: IN, standard, device
            0x06,   # GET_DESCRIPTOR
            (0x03 << 8) | index,  # string descriptor type + index
            langid or 0x0409,     # langid
            255
        )
        if len(buf) > 2:
            return buf[2:].tobytes().decode('utf-16-le', errors='replace')
    except Exception:
        pass
    return ""
usb.util.get_string = _patched_get_string

# Also patch the property accessors on usb.core.Device that cache string reads
_orig_serial = usb.core.Device.serial_number.fget
def _patched_serial(self):
    try:
        return _orig_serial(self)
    except (ValueError, usb.core.USBError, NotImplementedError):
        sn = _patched_get_string(self, self.iSerialNumber)
        self._serial_number = sn
        return sn
usb.core.Device.serial_number = property(_patched_serial)

_orig_manufacturer = usb.core.Device.manufacturer.fget
def _patched_manufacturer(self):
    try:
        return _orig_manufacturer(self)
    except (ValueError, usb.core.USBError, NotImplementedError):
        m = _patched_get_string(self, self.iManufacturer)
        self._manufacturer = m
        return m
usb.core.Device.manufacturer = property(_patched_manufacturer)

print("[*] USB backend + string descriptor patches applied", flush=True)

# Now import pymobiledevice3
from pymobiledevice3.irecv import IRecv
from pymobiledevice3.restore.device import Device
from pymobiledevice3.restore.restore import Restore, Behavior

# Patch module-level local references in irecv.py
import pymobiledevice3.irecv as _irecv_mod
_irecv_mod.find = _patched_find
_irecv_mod.get_string = _patched_get_string

# No need for VID-based _find or _populate_device_info patches anymore
# since usb.util.get_string and Device.serial_number/manufacturer are all patched.
print("[*] All patches applied", flush=True)

IPSW_PATH = r"C:\Users\nathan\Downloads\iPhone11,8_18.7.5_22H311_Restore.ipsw"
IPSW_URL = "https://updates.cdn-apple.com/2025FallFCS/fullrestores/072-81361/6ECBA79D-5499-4E68-AA77-D34C0F9E0BE7/iPhone11,8_18.7.5_22H311_Restore.ipsw"


async def do_restore():
    print("=" * 60, flush=True)
    print("  iPhone XR Restore to iOS 18.7.5", flush=True)
    print("=" * 60, flush=True)

    # Step 1: Locate or download IPSW
    ipsw_path = IPSW_PATH
    if not os.path.exists(ipsw_path):
        print(f"[!] IPSW not found locally, will use pymobiledevice3 auto-download", flush=True)
        ipsw_path = None
    else:
        size_gb = os.path.getsize(ipsw_path) / (1024**3)
        print(f"[+] IPSW: {ipsw_path} ({size_gb:.2f} GB)", flush=True)

    # Step 2: Connect to device in Recovery
    print("[*] Connecting to device in Recovery mode...", flush=True)
    try:
        irecv = IRecv(timeout=15)
        print(f"[+] Connected! Mode: {irecv.mode}", flush=True)
        info = irecv._device_info
        print(f"    CPID: {info.get('CPID', '?')}", flush=True)
        print(f"    ECID: {info.get('ECID', '?')}", flush=True)
        print(f"    SRNM: {info.get('SRNM', '?')}", flush=True)
    except Exception as e:
        print(f"[-] Cannot connect to device: {e}", flush=True)
        traceback.print_exc()
        sys.exit(1)

    # Step 3: Create Device object from IRecv
    print("[*] Initializing restore device...", flush=True)
    device = Device(irecv=irecv)
    print(f"[+] Device initialized (product: {device.product_type})", flush=True)

    # Step 4: Open IPSW and start restore
    if ipsw_path:
        print("[*] Opening local IPSW...", flush=True)
        ipsw_ctx = zipfile.ZipFile(ipsw_path)
    else:
        # Use pymobiledevice3's auto-download for the product
        from pymobiledevice3.restore.ipsw.ipsw import IPSW
        from pymobiledevice3.cli.restore import query_ipswme, tempzip_download_ctx
        print(f"[*] Querying ipsw.me for {device.product_type}...", flush=True)
        url = query_ipswme(device.product_type)
        print(f"[+] IPSW URL: {url}", flush=True)
        print(f"[*] Downloading IPSW (this will take a while)...", flush=True)
        ipsw_ctx = tempzip_download_ctx(url).__enter__()

    with ipsw_ctx as ipsw:
        print(f"[+] IPSW ready ({len(ipsw.namelist())} entries)", flush=True)
        print("[*] Starting restore (update mode - no erase)...", flush=True)
        print("[*] This will take 10-30 minutes. Do NOT unplug the device!", flush=True)
        print("", flush=True)
        try:
            await Restore(
                ipsw=ipsw,
                device=device,
                tss=None,
                behavior=Behavior.Update,
                ignore_fdr=False,
            ).update()
            print("\n[+] Restore completed successfully!", flush=True)
        except Exception:
            traceback.print_exc()
            sys.exit(1)


if __name__ == '__main__':
    asyncio.run(do_restore(), debug=True)
