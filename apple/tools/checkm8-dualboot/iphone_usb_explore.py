#!/usr/bin/env python3
"""
iPhone USB Interface Explorer
==============================
Explores ALL USB communication paths available on an iPhone XR.
Tests Normal Mode, Recovery Mode, and DFU Mode interfaces.

Goal: Find every possible way to communicate with the device via USB
before resorting to bare-metal TrustOS.
"""

import sys
import time
import struct
import json
from datetime import datetime

# Try multiple USB libraries
try:
    import usb.core
    import usb.util
    HAS_PYUSB = True
except ImportError:
    HAS_PYUSB = False

try:
    import ctypes
    import libusb_package
    HAS_LIBUSB = True
except ImportError:
    HAS_LIBUSB = False

# Apple USB IDs
APPLE_VID = 0x05AC

# Known Apple PIDs
APPLE_PIDS = {
    # DFU Mode
    0x1227: "DFU Mode",
    # Recovery Mode
    0x1280: "Recovery Mode (old)",
    0x1281: "Recovery Mode",
    0x1282: "Recovery Mode (WTF)",
    # Normal Mode - various iPhone models
    0x12A8: "iPhone (Normal Mode - PTP)",
    0x12A9: "iPhone (Normal Mode - PTP+iTunes)",
    0x12AB: "iPhone (Normal Mode - Tethering)",
    # iPad
    0x12AA: "iPad",
    0x129E: "iPad DFU",
    # Generic
    0x1290: "Apple T2 DFU",
    0x1291: "Apple T2 Recovery",
}

# USB class codes
USB_CLASS_NAMES = {
    0x00: "Per-Interface",
    0x02: "Communications (CDC)",
    0x06: "Still Image",
    0x08: "Mass Storage",
    0x0A: "CDC-Data",
    0x0B: "Smart Card",
    0x0E: "Video",
    0xFE: "Application Specific",
    0xFF: "Vendor Specific",
}

# Apple vendor-specific subclass names (educated guesses from research)
APPLE_SUBCLASS_NAMES = {
    (0xFF, 0xFE, 0x02): "usbmuxd (TCP-over-USB)",
    (0xFF, 0xFD, 0x01): "Apple USB Multiplexer (legacy)",
    (0xFF, 0x01, 0x01): "Apple Valeria (Display)",
    (0xFF, 0x02, 0x01): "Apple Probe (Debug)",
    (0xFF, 0x03, 0x01): "Apple JTAG/SWD",
    (0xFF, 0xF0, 0x01): "Apple SWD Debug Probe",
    (0xFF, 0xFC, 0x01): "Apple KIS (Kernel Internal Services)",
    (0x06, 0x01, 0x01): "PTP (Photo Transfer)",
}

results = {
    "timestamp": datetime.now().isoformat(),
    "tool": "iphone_usb_explore.py",
    "apple_devices": [],
    "all_devices": [],
    "tests": {},
}

def log(msg):
    print(f"[*] {msg}")

def log_ok(msg):
    print(f"[+] {msg}")

def log_err(msg):
    print(f"[-] {msg}")

def log_info(msg):
    print(f"    {msg}")

def get_string_safe(dev, index):
    """Safely get a USB string descriptor"""
    if index == 0:
        return ""
    try:
        return usb.util.get_string(dev, index)
    except:
        return f"<string #{index} unreadable>"

def identify_interface(bClass, bSubClass, bProtocol):
    """Try to identify what an interface does"""
    key = (bClass, bSubClass, bProtocol)
    if key in APPLE_SUBCLASS_NAMES:
        return APPLE_SUBCLASS_NAMES[key]
    
    # Vendor-specific heuristics
    if bClass == 0xFF:
        if bSubClass == 0xFE:
            return f"usbmuxd variant (proto={bProtocol})"
        if bSubClass == 0xFC:
            return f"Apple KIS variant (proto={bProtocol})"
        if bSubClass == 0xFD:
            return f"Apple Mux legacy (proto={bProtocol})"
        if bSubClass == 0x01:
            return f"Apple Valeria/Display (proto={bProtocol})"
        if bSubClass == 0x02:
            return f"Apple Probe/Debug (proto={bProtocol})"
        if bSubClass == 0xF0:
            return f"Apple SWD/Debug (proto={bProtocol})"
        return f"Vendor-Specific (sub=0x{bSubClass:02X} proto=0x{bProtocol:02X})"
    
    class_name = USB_CLASS_NAMES.get(bClass, f"Class 0x{bClass:02X}")
    return f"{class_name} (sub=0x{bSubClass:02X} proto=0x{bProtocol:02X})"

def endpoint_desc(ep):
    """Describe an endpoint"""
    addr = ep.bEndpointAddress
    direction = "IN" if addr & 0x80 else "OUT"
    ep_num = addr & 0x0F
    
    transfer_types = {0: "CONTROL", 1: "ISOCHRONOUS", 2: "BULK", 3: "INTERRUPT"}
    xfer = transfer_types.get(ep.bmAttributes & 0x03, "UNKNOWN")
    
    return {
        "address": f"0x{addr:02X}",
        "direction": direction,
        "ep_num": ep_num,
        "type": xfer,
        "max_packet_size": ep.wMaxPacketSize,
        "interval": ep.bInterval,
        "desc": f"EP{ep_num} {direction} {xfer} (max={ep.wMaxPacketSize}B)"
    }


def scan_all_devices():
    """Scan all USB devices, highlight Apple devices"""
    log("=== Scanning ALL USB devices ===")
    
    all_devs = list(usb.core.find(find_all=True))
    log(f"Found {len(all_devs)} USB devices total")
    
    apple_devs = []
    
    for dev in all_devs:
        vid = dev.idVendor
        pid = dev.idProduct
        
        dev_info = {
            "vid": f"0x{vid:04X}",
            "pid": f"0x{pid:04X}",
            "bus": dev.bus,
            "address": dev.address,
        }
        
        try:
            dev_info["manufacturer"] = get_string_safe(dev, dev.iManufacturer)
            dev_info["product"] = get_string_safe(dev, dev.iProduct)
            dev_info["serial"] = get_string_safe(dev, dev.iSerialNumber)
        except:
            pass
        
        results["all_devices"].append(dev_info)
        
        if vid == APPLE_VID:
            mode = APPLE_PIDS.get(pid, f"Unknown Apple Device (PID=0x{pid:04X})")
            dev_info["mode"] = mode
            apple_devs.append((dev, dev_info))
            log_ok(f"APPLE DEVICE: {mode}")
            log_info(f"VID=0x{vid:04X} PID=0x{pid:04X}")
            if "manufacturer" in dev_info:
                log_info(f"Manufacturer: {dev_info.get('manufacturer', 'N/A')}")
                log_info(f"Product: {dev_info.get('product', 'N/A')}")
                log_info(f"Serial: {dev_info.get('serial', 'N/A')}")
        else:
            mfr = dev_info.get('manufacturer', '')
            prod = dev_info.get('product', '')
            # Check if "Apple" appears anywhere
            if 'apple' in str(mfr).lower() or 'apple' in str(prod).lower():
                apple_devs.append((dev, dev_info))
                log_ok(f"Apple-related device: {prod} (VID=0x{vid:04X} PID=0x{pid:04X})")
    
    return apple_devs


def deep_enumerate(dev, dev_info):
    """Deep-enumerate a device: all configs, interfaces, alt settings, endpoints"""
    log(f"\n=== Deep Enumeration: {dev_info.get('mode', 'Unknown')} ===")
    
    device_data = dict(dev_info)
    device_data["device_class"] = f"0x{dev.bDeviceClass:02X}"
    device_data["device_subclass"] = f"0x{dev.bDeviceSubClass:02X}"
    device_data["device_protocol"] = f"0x{dev.bDeviceProtocol:02X}"
    device_data["usb_version"] = f"{dev.bcdUSB >> 8}.{(dev.bcdUSB >> 4) & 0xF}{dev.bcdUSB & 0xF}"
    device_data["max_packet_size_ep0"] = dev.bMaxPacketSize0
    device_data["num_configurations"] = dev.bNumConfigurations
    device_data["configurations"] = []
    
    log_info(f"USB Version: {device_data['usb_version']}")
    log_info(f"Device Class: {device_data['device_class']}")
    log_info(f"EP0 Max Packet Size: {dev.bMaxPacketSize0}")
    log_info(f"Configurations: {dev.bNumConfigurations}")
    
    for cfg_idx in range(dev.bNumConfigurations):
        try:
            cfg = dev[cfg_idx]
        except Exception as e:
            log_err(f"  Cannot read config #{cfg_idx}: {e}")
            continue
        
        cfg_data = {
            "config_value": cfg.bConfigurationValue,
            "num_interfaces": cfg.bNumInterfaces,
            "max_power_mA": cfg.bMaxPower * 2,
            "self_powered": bool(cfg.bmAttributes & 0x40),
            "remote_wakeup": bool(cfg.bmAttributes & 0x20),
            "interfaces": [],
        }
        
        log(f"\n  Configuration #{cfg.bConfigurationValue}:")
        log_info(f"  Interfaces: {cfg.bNumInterfaces}, MaxPower: {cfg.bMaxPower*2}mA")
        
        for intf in cfg:
            intf_name = identify_interface(intf.bInterfaceClass, intf.bInterfaceSubClass, intf.bInterfaceProtocol)
            
            intf_data = {
                "number": intf.bInterfaceNumber,
                "alt_setting": intf.bAlternateSetting,
                "class": f"0x{intf.bInterfaceClass:02X}",
                "subclass": f"0x{intf.bInterfaceSubClass:02X}",
                "protocol": f"0x{intf.bInterfaceProtocol:02X}",
                "identified_as": intf_name,
                "num_endpoints": intf.bNumEndpoints,
                "endpoints": [],
            }
            
            try:
                intf_str = get_string_safe(dev, intf.iInterface)
                intf_data["interface_string"] = intf_str
            except:
                intf_str = ""
            
            log(f"\n    Interface #{intf.bInterfaceNumber} (alt={intf.bAlternateSetting}):")
            log_ok(f"    → {intf_name}")
            if intf_str:
                log_info(f"    String: \"{intf_str}\"")
            log_info(f"    Class=0x{intf.bInterfaceClass:02X} Sub=0x{intf.bInterfaceSubClass:02X} Proto=0x{intf.bInterfaceProtocol:02X}")
            log_info(f"    Endpoints: {intf.bNumEndpoints}")
            
            for ep in intf:
                ep_data = endpoint_desc(ep)
                intf_data["endpoints"].append(ep_data)
                log_info(f"      {ep_data['desc']}")
            
            cfg_data["interfaces"].append(intf_data)
        
        device_data["configurations"].append(cfg_data)
    
    results["apple_devices"].append(device_data)
    return device_data


def try_control_requests(dev, dev_info):
    """Try various control requests to probe the device"""
    log(f"\n=== Probing Control Requests ===")
    
    probe_results = {}
    
    # Standard requests
    standard_requests = [
        ("GET_STATUS (device)", 0x80, 0x00, 0, 0, 2),
        ("GET_STATUS (interface 0)", 0x81, 0x00, 0, 0, 2),
        ("GET_STATUS (ep0)", 0x82, 0x00, 0, 0, 2),
        # String descriptors (can reveal info)
        ("STRING desc #1 (Manufacturer)", 0x80, 0x06, 0x0301, 0x0409, 255),
        ("STRING desc #2 (Product)", 0x80, 0x06, 0x0302, 0x0409, 255),
        ("STRING desc #3 (Serial)", 0x80, 0x06, 0x0303, 0x0409, 255),
        ("STRING desc #4", 0x80, 0x06, 0x0304, 0x0409, 255),
        ("STRING desc #5", 0x80, 0x06, 0x0305, 0x0409, 255),
        ("STRING desc #6", 0x80, 0x06, 0x0306, 0x0409, 255),
        ("STRING desc #7", 0x80, 0x06, 0x0307, 0x0409, 255),
        ("STRING desc #8", 0x80, 0x06, 0x0308, 0x0409, 255),
        ("STRING desc #9", 0x80, 0x06, 0x0309, 0x0409, 255),
        ("STRING desc #10", 0x80, 0x06, 0x030A, 0x0409, 255),
        # Device qualifier (USB 2.0)
        ("DEVICE QUALIFIER", 0x80, 0x06, 0x0600, 0, 10),
        # BOS descriptor (USB 3.x)
        ("BOS descriptor", 0x80, 0x06, 0x0F00, 0, 64),
        # Microsoft OS descriptor
        ("MS OS String desc", 0x80, 0x06, 0x03EE, 0, 18),
    ]
    
    for name, bmReq, bReq, wVal, wIdx, wLen in standard_requests:
        try:
            data = dev.ctrl_transfer(bmReq, bReq, wVal, wIdx, wLen, timeout=1000)
            hex_data = ' '.join(f'{b:02X}' for b in data)
            
            # Decode strings
            decoded = ""
            if bmReq == 0x80 and bReq == 0x06 and (wVal >> 8) == 0x03:
                try:
                    decoded = bytes(data[2:]).decode('utf-16-le', errors='replace')
                except:
                    pass
            
            probe_results[name] = {
                "success": True,
                "length": len(data),
                "hex": hex_data[:120],
                "decoded": decoded,
            }
            
            if decoded:
                log_ok(f"{name}: \"{decoded}\" ({len(data)}B)")
            else:
                log_ok(f"{name}: {len(data)}B — {hex_data[:60]}")
                
        except usb.core.USBError as e:
            probe_results[name] = {"success": False, "error": str(e)}
            # Only log interesting failures
            if "STRING" not in name or "#4" in name or "#5" in name:
                log_err(f"{name}: {e}")
    
    # Apple-specific vendor requests (educated probing)
    log("\n--- Apple Vendor-Specific Requests ---")
    
    apple_requests = [
        # Vendor requests (bmRequestType = 0xC0 = device-to-host, vendor, device)
        ("Apple vendor 0x00", 0xC0, 0x00, 0, 0, 64),
        ("Apple vendor 0x01", 0xC0, 0x01, 0, 0, 64),
        ("Apple vendor 0x02", 0xC0, 0x02, 0, 0, 64),
        ("Apple vendor 0x03", 0xC0, 0x03, 0, 0, 64),
        ("Apple vendor 0x04", 0xC0, 0x04, 0, 0, 64),
        ("Apple vendor 0x05", 0xC0, 0x05, 0, 0, 64),
        ("Apple vendor 0x06", 0xC0, 0x06, 0, 0, 64),
        ("Apple vendor 0x10", 0xC0, 0x10, 0, 0, 64),
        ("Apple vendor 0x20", 0xC0, 0x20, 0, 0, 64),
        ("Apple vendor 0x40 (serial?)", 0xC0, 0x40, 0, 0, 256),
        ("Apple vendor 0x41", 0xC0, 0x41, 0, 0, 256),
        ("Apple vendor 0x80", 0xC0, 0x80, 0, 0, 64),
        ("Apple vendor 0xFF", 0xC0, 0xFF, 0, 0, 64),
        # Interface vendor requests
        ("Apple intf vendor 0x00", 0xC1, 0x00, 0, 0, 64),
        ("Apple intf vendor 0x01", 0xC1, 0x01, 0, 0, 64),
        ("Apple intf vendor 0x02", 0xC1, 0x02, 0, 0, 64),
        ("Apple intf vendor 0x40", 0xC1, 0x40, 0, 0, 256),
        ("Apple intf vendor 0x52", 0xC1, 0x52, 0, 0, 64),  # Recovery serial?
    ]
    
    for name, bmReq, bReq, wVal, wIdx, wLen in apple_requests:
        try:
            data = dev.ctrl_transfer(bmReq, bReq, wVal, wIdx, wLen, timeout=500)
            hex_data = ' '.join(f'{b:02X}' for b in data)
            
            # Try ASCII decode
            try:
                ascii_str = bytes(data).decode('ascii', errors='replace')
            except:
                ascii_str = ""
            
            probe_results[name] = {
                "success": True,
                "length": len(data),
                "hex": hex_data[:120],
                "ascii": ascii_str[:80],
            }
            log_ok(f"*** {name}: {len(data)}B — {hex_data[:80]}")
            if ascii_str and any(c.isalpha() for c in ascii_str):
                log_info(f"  ASCII: {ascii_str[:80]}")
                
        except usb.core.USBError as e:
            err = str(e)
            if "pipe" in err.lower() or "stall" in err.lower():
                probe_results[name] = {"success": False, "error": "STALL"}
            else:
                probe_results[name] = {"success": False, "error": err}
    
    results["tests"]["control_requests"] = probe_results


def try_dfu_commands(dev, dev_info):
    """If device is in DFU mode, try DFU-specific commands"""
    log(f"\n=== DFU Mode Commands ===")
    
    dfu_results = {}
    
    # DFU GETSTATUS
    try:
        data = dev.ctrl_transfer(0xA1, 3, 0, 0, 6, timeout=1000)
        status = data[0]
        poll_timeout = data[1] | (data[2] << 8) | (data[3] << 16)
        state = data[4]
        states = {0:"appIDLE", 1:"appDETACH", 2:"dfuIDLE", 3:"dfuDNLOAD-SYNC",
                  4:"dfuDNBUSY", 5:"dfuDNLOAD-IDLE", 6:"dfuMANIFEST-SYNC",
                  7:"dfuMANIFEST", 8:"dfuMANIFEST-WAIT-RESET", 9:"dfuUPLOAD-IDLE",
                  10:"dfuERROR"}
        dfu_results["GETSTATUS"] = {
            "status": status, "state": state,
            "state_name": states.get(state, "UNKNOWN"),
            "poll_timeout": poll_timeout,
        }
        log_ok(f"DFU GETSTATUS: status={status}, state={state} ({states.get(state, '?')}), poll_timeout={poll_timeout}ms")
    except Exception as e:
        dfu_results["GETSTATUS"] = {"error": str(e)}
        log_err(f"DFU GETSTATUS: {e}")
    
    # DFU GETSTATE
    try:
        data = dev.ctrl_transfer(0xA1, 5, 0, 0, 1, timeout=1000)
        dfu_results["GETSTATE"] = {"state": data[0]}
        log_ok(f"DFU GETSTATE: {data[0]}")
    except Exception as e:
        dfu_results["GETSTATE"] = {"error": str(e)}
    
    # DFU UPLOAD (read from device — may leak heap data!)
    try:
        data = dev.ctrl_transfer(0xA1, 2, 0, 0, 2048, timeout=2000)
        hex_preview = ' '.join(f'{b:02X}' for b in data[:64])
        nonzero = any(b != 0 for b in data)
        dfu_results["UPLOAD"] = {
            "length": len(data),
            "nonzero": nonzero,
            "preview": hex_preview,
        }
        log_ok(f"DFU UPLOAD: {len(data)}B, nonzero={nonzero}")
        if nonzero:
            log_info(f"  First 64B: {hex_preview}")
    except Exception as e:
        dfu_results["UPLOAD"] = {"error": str(e)}
        log_err(f"DFU UPLOAD: {e}")
    
    results["tests"]["dfu"] = dfu_results


def try_recovery_commands(dev, dev_info):
    """If device is in Recovery Mode, try iBoot console communication"""
    log(f"\n=== Recovery Mode (iBoot Console) ===")
    
    recovery_results = {}
    
    # Recovery Mode uses BULK endpoints for text I/O
    # Find bulk endpoints
    cfg = dev[0]
    bulk_in = None
    bulk_out = None
    
    for intf in cfg:
        for ep in intf:
            if usb.util.endpoint_type(ep.bmAttributes) == usb.util.ENDPOINT_TYPE_BULK:
                if usb.util.endpoint_direction(ep.bEndpointAddress) == usb.util.ENDPOINT_IN:
                    bulk_in = ep
                    log_ok(f"Found BULK IN: EP{ep.bEndpointAddress & 0x0F} (0x{ep.bEndpointAddress:02X})")
                else:
                    bulk_out = ep
                    log_ok(f"Found BULK OUT: EP{ep.bEndpointAddress & 0x0F} (0x{ep.bEndpointAddress:02X})")
    
    if not bulk_out:
        log_err("No BULK OUT endpoint found — cannot send commands")
        recovery_results["error"] = "No bulk endpoints"
        results["tests"]["recovery"] = recovery_results
        return
    
    # Try to claim the interface
    try:
        if dev.is_kernel_driver_active(0):
            dev.detach_kernel_driver(0)
    except:
        pass
    
    try:
        usb.util.claim_interface(dev, 0)
        log_ok("Claimed interface 0")
    except Exception as e:
        log_err(f"Cannot claim interface: {e}")
    
    # Apple Recovery Mode control request to get device info
    # irecovery uses bmRequestType=0xC0, bRequest=0x00
    log("\n--- Recovery control requests ---")
    
    # Get serial (Apple specific)
    recovery_ctrl_requests = [
        ("Recovery GETENV autoboot", 0x40, 0, 0, 0),  # OUT request
        ("Recovery vendor IN 0x00", 0xC0, 0x00, 0, 0),
        ("Recovery vendor IN 0x01", 0xC0, 0x01, 0, 0),
    ]
    
    # iBoot commands to try via BULK OUT
    iboot_commands = [
        "getenv build-version",
        "getenv build-style",
        "getenv chip-id",
        "getenv board-id",
        "getenv security-domain",
        "getenv device-name",
        "getenv model-number",
        "getenv region-info",
        "getenv serial-number",
        "getenv boot-args",
        "getenv auto-boot",
        "getenv debug-uarts",
        "getenv usb-enabled",
        "getenv display-timing",
        "getenv firmware-version",
        "version",
        "bgcolor 0 0 0",  # harmless - set background color to black
    ]
    
    log("\n--- Sending iBoot commands via BULK endpoint ---")
    
    for cmd in iboot_commands:
        try:
            # Send command + newline via BULK OUT
            cmd_bytes = (cmd + "\n").encode('ascii')
            bytes_sent = bulk_out.write(cmd_bytes, timeout=1000)
            
            # Read response via BULK IN
            response = b""
            if bulk_in:
                try:
                    for _ in range(5):  # Read multiple chunks
                        chunk = bulk_in.read(4096, timeout=500)
                        if chunk:
                            response += bytes(chunk)
                        if len(chunk) < 4096:
                            break
                except usb.core.USBTimeoutError:
                    pass
                except Exception as e:
                    pass
            
            resp_str = response.decode('ascii', errors='replace').strip()
            recovery_results[cmd] = {
                "sent": bytes_sent,
                "response_len": len(response),
                "response": resp_str[:500],
            }
            
            if resp_str:
                log_ok(f"[{cmd}] → \"{resp_str[:120]}\"")
            else:
                log_info(f"[{cmd}] → (no response, {bytes_sent}B sent)")
                
        except usb.core.USBTimeoutError:
            recovery_results[cmd] = {"error": "timeout"}
            log_err(f"[{cmd}] → timeout")
        except usb.core.USBError as e:
            recovery_results[cmd] = {"error": str(e)}
            log_err(f"[{cmd}] → {e}")
        
        time.sleep(0.1)
    
    results["tests"]["recovery"] = recovery_results


def try_normal_mode(dev, dev_info):
    """If device is in normal mode, try usbmuxd-style communication"""
    log(f"\n=== Normal Mode USB Communication ===")
    
    normal_results = {}
    
    # In normal mode, Apple devices expose:
    # - PTP interface (class 0x06 Still Image)
    # - Vendor-specific interface for usbmuxd (class 0xFF, sub 0xFE, proto 0x02)
    # - Possibly ACM/CDC for serial
    
    cfg = dev[0]
    
    usbmux_intf = None
    debug_intfs = []
    
    for intf in cfg:
        cls = intf.bInterfaceClass
        sub = intf.bInterfaceSubClass
        proto = intf.bInterfaceProtocol
        
        if cls == 0xFF and sub == 0xFE and proto == 0x02:
            usbmux_intf = intf
            log_ok(f"Found usbmuxd interface: #{intf.bInterfaceNumber}")
        elif cls == 0xFF:
            debug_intfs.append(intf)
            name = identify_interface(cls, sub, proto)
            log_ok(f"Found vendor-specific interface #{intf.bInterfaceNumber}: {name}")
    
    # Try to communicate via usbmuxd protocol
    if usbmux_intf:
        log("\n--- usbmuxd handshake ---")
        
        bulk_in = None
        bulk_out = None
        for ep in usbmux_intf:
            if usb.util.endpoint_type(ep.bmAttributes) == usb.util.ENDPOINT_TYPE_BULK:
                if usb.util.endpoint_direction(ep.bEndpointAddress) == usb.util.ENDPOINT_IN:
                    bulk_in = ep
                else:
                    bulk_out = ep
        
        if bulk_in and bulk_out:
            try:
                try:
                    if dev.is_kernel_driver_active(usbmux_intf.bInterfaceNumber):
                        dev.detach_kernel_driver(usbmux_intf.bInterfaceNumber)
                except:
                    pass
                
                usb.util.claim_interface(dev, usbmux_intf.bInterfaceNumber)
                log_ok(f"Claimed usbmuxd interface #{usbmux_intf.bInterfaceNumber}")
                
                # usbmuxd binary protocol: version exchange
                # Header: length(4) + version(4) + message_type(4) + tag(4)
                # Version exchange: type=3
                import plistlib
                
                # Try binary protocol first
                plist_data = plistlib.dumps({
                    "ClientVersionString": "TrustOS-Explorer 1.0",
                    "MessageType": "ListDevices",
                    "ProgName": "trustos",
                    "kLibUSBMuxVersion": 3,
                }, fmt=plistlib.FMT_XML)
                
                header = struct.pack('<IIII', len(plist_data) + 16, 1, 8, 1)
                
                bulk_out.write(header + plist_data, timeout=2000)
                log_ok("Sent usbmuxd plist handshake")
                
                # Read response
                try:
                    resp = bytes(bulk_in.read(65536, timeout=2000))
                    if len(resp) >= 16:
                        r_len, r_ver, r_type, r_tag = struct.unpack('<IIII', resp[:16])
                        payload = resp[16:]
                        log_ok(f"Response: len={r_len}, ver={r_ver}, type={r_type}, tag={r_tag}")
                        if payload:
                            try:
                                plist_resp = plistlib.loads(payload)
                                log_ok(f"Plist response: {plist_resp}")
                                normal_results["usbmuxd_response"] = str(plist_resp)
                            except:
                                log_info(f"Raw response ({len(payload)}B): {payload[:200]}")
                                normal_results["usbmuxd_raw"] = payload[:200].hex()
                    else:
                        log_info(f"Short response: {resp.hex()}")
                except usb.core.USBTimeoutError:
                    log_err("usbmuxd read timeout")
                    normal_results["usbmuxd"] = "timeout"
                    
            except usb.core.USBError as e:
                log_err(f"usbmuxd error: {e}")
                normal_results["usbmuxd_error"] = str(e)
    
    # Try vendor-specific debug interfaces
    for intf in debug_intfs:
        intf_num = intf.bInterfaceNumber
        name = identify_interface(intf.bInterfaceClass, intf.bInterfaceSubClass, intf.bInterfaceProtocol)
        log(f"\n--- Probing debug interface #{intf_num}: {name} ---")
        
        for ep in intf:
            ep_info = endpoint_desc(ep)
            log_info(f"  {ep_info['desc']}")
        
        # Try reading from bulk IN endpoints
        try:
            try:
                if dev.is_kernel_driver_active(intf_num):
                    dev.detach_kernel_driver(intf_num)
            except:
                pass
            
            usb.util.claim_interface(dev, intf_num)
            
            for ep in intf:
                if usb.util.endpoint_direction(ep.bEndpointAddress) == usb.util.ENDPOINT_IN:
                    try:
                        data = bytes(ep.read(4096, timeout=500))
                        if data:
                            log_ok(f"  Data from EP 0x{ep.bEndpointAddress:02X}: {len(data)}B")
                            log_info(f"    Hex: {data[:64].hex()}")
                            try:
                                log_info(f"    ASCII: {data[:64].decode('ascii', errors='replace')}")
                            except:
                                pass
                            normal_results[f"intf{intf_num}_ep{ep.bEndpointAddress:02X}"] = data[:256].hex()
                    except usb.core.USBTimeoutError:
                        log_info(f"  EP 0x{ep.bEndpointAddress:02X}: no data (timeout)")
                    except usb.core.USBError as e:
                        log_info(f"  EP 0x{ep.bEndpointAddress:02X}: {e}")
            
            usb.util.release_interface(dev, intf_num)
        except Exception as e:
            log_err(f"  Cannot probe interface #{intf_num}: {e}")
    
    results["tests"]["normal_mode"] = normal_results


def try_apple_diags(dev, dev_info):
    """Try Apple diagnostic/debug vendor requests that might work in any mode"""
    log("\n=== Apple Diagnostic Probing ===")
    
    diag_results = {}
    
    # Systematic scan of ALL possible vendor requests (0x00-0xFF)
    # bmRequestType = 0xC0 (IN, Vendor, Device)
    log("Scanning all vendor device-to-host requests (0x00-0xFF)...")
    
    found_count = 0
    for bReq in range(256):
        try:
            data = dev.ctrl_transfer(0xC0, bReq, 0, 0, 256, timeout=200)
            if len(data) > 0:
                hex_data = ' '.join(f'{b:02X}' for b in data[:32])
                try:
                    ascii_str = bytes(data).decode('ascii', errors='replace')
                except:
                    ascii_str = ""
                
                diag_results[f"vendor_0x{bReq:02X}"] = {
                    "length": len(data),
                    "hex": hex_data,
                    "ascii": ascii_str[:80],
                }
                log_ok(f"  bRequest=0x{bReq:02X}: {len(data)}B — {hex_data[:60]}")
                if ascii_str and any(c.isalpha() for c in ascii_str):
                    log_info(f"    ASCII: {ascii_str[:80]}")
                found_count += 1
        except:
            pass
    
    log(f"\nFound {found_count} responding vendor requests")
    
    # Also try wValue variations on responding requests
    if found_count > 0:
        log("\nProbing wValue variations on responding requests...")
        for bReq_hex, info in list(diag_results.items()):
            bReq = int(bReq_hex.split("_")[1], 16)
            for wVal in [1, 2, 3, 4, 0x100, 0x200, 0x300, 0x400, 0xFFFF]:
                try:
                    data = dev.ctrl_transfer(0xC0, bReq, wVal, 0, 256, timeout=200)
                    if len(data) > 0:
                        hex_data = ' '.join(f'{b:02X}' for b in data[:32])
                        key = f"vendor_0x{bReq:02X}_wVal=0x{wVal:04X}"
                        diag_results[key] = {"length": len(data), "hex": hex_data}
                        log_ok(f"  bReq=0x{bReq:02X} wVal=0x{wVal:04X}: {len(data)}B — {hex_data[:40]}")
                except:
                    pass
    
    # Try interface-level vendor requests on each interface
    log("\nScanning interface-level vendor requests...")
    try:
        cfg = dev[0]
        for intf in cfg:
            intf_num = intf.bInterfaceNumber
            for bReq in [0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x10, 0x20, 0x40, 0x52, 0x80, 0xFF]:
                try:
                    data = dev.ctrl_transfer(0xC1, bReq, 0, intf_num, 256, timeout=200)
                    if len(data) > 0:
                        hex_data = ' '.join(f'{b:02X}' for b in data[:32])
                        key = f"intf{intf_num}_vendor_0x{bReq:02X}"
                        diag_results[key] = {"length": len(data), "hex": hex_data}
                        log_ok(f"  Intf#{intf_num} bReq=0x{bReq:02X}: {len(data)}B — {hex_data[:60]}")
                except:
                    pass
    except:
        pass
    
    results["tests"]["diagnostics"] = diag_results


def main():
    print("=" * 70)
    print("  iPhone USB Interface Explorer")
    print("  Discovering ALL USB communication paths")
    print("=" * 70)
    print()
    
    if not HAS_PYUSB:
        print("ERROR: pyusb not installed. Run: pip install pyusb")
        sys.exit(1)
    
    # Use libusb backend
    backend = None
    if HAS_LIBUSB:
        try:
            backend = usb.backend.libusb1.get_backend(find_library=libusb_package.find_library)
            log("Using libusb_package backend")
        except:
            pass
    
    if backend:
        usb.core.find(backend=backend)  # Initialize
    
    # Step 1: Scan all USB devices
    apple_devs = scan_all_devices()
    
    if not apple_devs:
        log_err("\nNo Apple devices found!")
        log_info("Make sure your iPhone is connected via USB")
        log_info("Try these modes:")
        log_info("  Normal: Just plug in (may need to Trust this computer)")
        log_info("  Recovery: Hold Volume Up while connecting")
        log_info("  DFU: Volume+, Volume-, hold Side, then Side+VolDown 5s, release Side")
        
        # Still save results
        with open("results/usb_explore_results.json", "w") as f:
            json.dump(results, f, indent=2, default=str)
        return
    
    print()
    log(f"Found {len(apple_devs)} Apple device(s)")
    
    # Step 2: Deep enumerate each Apple device
    for dev, dev_info in apple_devs:
        try:
            device_data = deep_enumerate(dev, dev_info)
        except Exception as e:
            log_err(f"Enumeration failed: {e}")
            continue
        
        pid = dev.idProduct
        
        # Step 3: Try control request probing (all modes)
        try:
            try_control_requests(dev, dev_info)
        except Exception as e:
            log_err(f"Control request probing failed: {e}")
        
        # Step 4: Apple diagnostic probing (all modes)
        try:
            try_apple_diags(dev, dev_info)
        except Exception as e:
            log_err(f"Diagnostic probing failed: {e}")
        
        # Step 5: Mode-specific tests
        if pid == 0x1227:
            try:
                try_dfu_commands(dev, dev_info)
            except Exception as e:
                log_err(f"DFU test failed: {e}")
                
        elif pid in (0x1280, 0x1281, 0x1282):
            try:
                try_recovery_commands(dev, dev_info)
            except Exception as e:
                log_err(f"Recovery test failed: {e}")
                
        elif pid in (0x12A8, 0x12A9, 0x12AB, 0x12AA):
            try:
                try_normal_mode(dev, dev_info)
            except Exception as e:
                log_err(f"Normal mode test failed: {e}")
        
        else:
            # Unknown mode — try everything
            log("\nUnknown mode — trying all protocols...")
            try:
                try_dfu_commands(dev, dev_info)
            except:
                pass
            try:
                try_recovery_commands(dev, dev_info)
            except:
                pass
            try:
                try_normal_mode(dev, dev_info)
            except:
                pass
    
    # Save results
    import os
    os.makedirs("results", exist_ok=True)
    
    output_file = "results/usb_explore_results.json"
    with open(output_file, "w") as f:
        json.dump(results, f, indent=2, default=str)
    
    print()
    print("=" * 70)
    log_ok(f"Results saved to {output_file}")
    print("=" * 70)
    
    # Summary
    print("\n=== SUMMARY ===")
    for dev_data in results["apple_devices"]:
        mode = dev_data.get("mode", "Unknown")
        configs = dev_data.get("configurations", [])
        total_intfs = sum(len(c.get("interfaces", [])) for c in configs)
        total_eps = sum(
            len(i.get("endpoints", []))
            for c in configs
            for i in c.get("interfaces", [])
        )
        print(f"\n  Device: {mode}")
        print(f"  Configs: {len(configs)}, Interfaces: {total_intfs}, Endpoints: {total_eps}")
        
        for c in configs:
            for i in c.get("interfaces", []):
                name = i.get("identified_as", "?")
                eps = i.get("endpoints", [])
                ep_desc = ", ".join(e["desc"] for e in eps)
                print(f"    Interface #{i['number']}: {name}")
                if ep_desc:
                    print(f"      Endpoints: {ep_desc}")
    
    # Highlight findings
    tests = results.get("tests", {})
    diag = tests.get("diagnostics", {})
    if diag:
        responding = [k for k, v in diag.items() if isinstance(v, dict) and v.get("length", 0) > 0]
        if responding:
            print(f"\n  *** {len(responding)} vendor requests responded! ***")
            for r in responding[:10]:
                print(f"    {r}: {diag[r].get('hex', '')[:40]}")


if __name__ == "__main__":
    main()
