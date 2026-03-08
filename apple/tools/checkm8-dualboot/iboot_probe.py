#!/usr/bin/env python3
"""
iboot_probe.py - Probe iBoot attack surface via Recovery Mode USB
Target: iPhone XR (A12 T8020), iBoot-11881.0.193.0.1, iOS 18.5

Recovery mode iBoot exposes:
  - Interface 0: DFU class (0xFE/0x01/0x02) - for firmware upload 
  - Interface 1: Apple vendor (0xFF/0xFF/0x51) - for irecv commands

This script explores:
  1. iBoot environment variables (getenv)
  2. USB control transfer behavior
  3. iBoot command interface
  4. Image upload parsing surface
"""

import sys
import time
import struct
import json
from datetime import datetime


def probe_usb_descriptors():
    """Read all USB descriptors and check for anomalies."""
    print("\n=== USB Descriptor Analysis ===")
    import usb.core, usb.util, libusb_package
    backend = libusb_package.get_libusb1_backend()
    
    dev = usb.core.find(idVendor=0x05AC, idProduct=0x1281, backend=backend)
    if not dev:
        # Try other recovery PIDs
        for pid in [0x1280, 0x1282, 0x1283]:
            dev = usb.core.find(idVendor=0x05AC, idProduct=pid, backend=backend)
            if dev:
                break
    
    if not dev:
        print("[-] No device in Recovery mode")
        return None
    
    print(f"[+] Device: VID:{dev.idVendor:04X} PID:{dev.idProduct:04X}")
    print(f"    bcdDevice: {dev.bcdDevice:04X}")
    print(f"    bMaxPacketSize0: {dev.bMaxPacketSize0}")
    
    # Check for ZLP conditions (descriptor sizes mod 64)
    results = {'descriptors': []}
    for cfg in dev:
        print(f"\n  Config {cfg.bConfigurationValue}:")
        print(f"    wTotalLength: {cfg.wTotalLength} (mod64={cfg.wTotalLength % 64})")
        results['descriptors'].append({
            'config': cfg.bConfigurationValue,
            'totalLength': cfg.wTotalLength,
            'mod64': cfg.wTotalLength % 64
        })
        
        for intf in cfg:
            cls = f"{intf.bInterfaceClass:02X}/{intf.bInterfaceSubClass:02X}/{intf.bInterfaceProtocol:02X}"
            print(f"    Interface {intf.bInterfaceNumber} alt {intf.bAlternateSetting}: {cls}")
            
            for ep in intf:
                direction = "IN" if usb.util.endpoint_direction(ep.bEndpointAddress) == usb.util.ENDPOINT_IN else "OUT"
                print(f"      EP {ep.bEndpointAddress:#04x} ({direction}): maxPacket={ep.wMaxPacketSize}")
    
    return dev


def probe_irecv_env():
    """Read iBoot environment variables via pymobiledevice3."""
    print("\n=== iBoot Environment Variables ===")
    try:
        from pymobiledevice3.irecv import IRecv
        client = IRecv()
        
        # Comprehensive list of iBoot env vars
        env_vars = [
            # Device identity
            'CPID', 'CPRV', 'CPFM', 'SCEP', 'BDID', 'ECID', 'IBFL',
            'SRNM', 'IMEI', 'MEID', 'SRTG', 'TYPE',
            # Build info
            'build-version', 'build-style', 'firmware-version',
            'hardware-model', 'model-number', 'region-info',
            'serial-number', 'mlb-serial-number',
            # Boot config  
            'auto-boot', 'boot-command', 'boot-device', 'boot-partition',
            'boot-args', 'boot-script', 'idle-off',
            # Security
            'com.apple.System.boot-nonce', 'effective-production-status-ap',
            'effective-security-mode-ap', 'security-domain',
            'uid-cid-personalize-count', 'debug-uarts',
            # Display/hardware
            'darkboot', 'device-color', 'device-color-map',
            'display-rotation', 'display-scale',
            # Recovery specific
            'backlight-level', 'recoveryos-boot-mode',
            'obliteration', 'obliteration-begin-date',
            # Interesting for exploit
            'loadaddr', 'filesize', 'com.apple.System.tz0-size',
            'base-address', 'display-timing',
            'usb-enabled', 'ramdisk-delay',
            'diags', 'diag-log', 'debug-enabled',
            'development-cert',
            # NVRAM stuff
            'SystemAudioVolume', 'backlight-nits',
            'preferred-count', 'wifi-country-code',
        ]
        
        results = {}
        for var in env_vars:
            try:
                val = client.getenv(var)
                if val is not None:
                    # Clean up bytearray responses
                    if isinstance(val, (bytes, bytearray)):
                        val_str = val.rstrip(b'\x00').decode('utf-8', errors='replace')
                    else:
                        val_str = str(val)
                    results[var] = val_str
                    print(f"  {var}: {val_str}")
            except Exception:
                pass
        
        print(f"\n[+] Read {len(results)} variables")
        return results
        
    except Exception as e:
        print(f"[-] IRecv error: {e}")
        return {}


def probe_irecv_commands():
    """Test iBoot command responses."""
    print("\n=== iBoot Command Probing ===")
    try:
        from pymobiledevice3.irecv import IRecv
        client = IRecv()
        
        # Safe commands to test (read-only / info commands)
        safe_commands = [
            'help',           # List available commands
            'printenv',       # Print all env vars
            'version',        # iBoot version
            'devicetree',     # Device tree info
            'meminfo',        # Memory info
            'bgcolor',        # Background color (harmless)
        ]
        
        results = {}
        for cmd in safe_commands:
            try:
                print(f"\n  >> Sending: '{cmd}'")
                # Send command via USB control transfer
                resp = client.send_command(cmd)
                if resp:
                    resp_str = resp if isinstance(resp, str) else str(resp)
                    # Truncate long responses
                    if len(resp_str) > 500:
                        print(f"     Response ({len(resp_str)} chars): {resp_str[:500]}...")
                    else:
                        print(f"     Response: {resp_str}")
                    results[cmd] = resp_str
                else:
                    print(f"     No response / OK")
                    results[cmd] = "OK_NO_RESPONSE"
            except Exception as e:
                err = str(e)
                print(f"     Error: {err[:200]}")
                results[cmd] = f"ERROR: {err[:200]}"
        
        return results
        
    except Exception as e:
        print(f"[-] IRecv command error: {e}")
        return {}


def probe_control_transfers():
    """Probe USB control transfer behavior for anomalies."""
    print("\n=== USB Control Transfer Probing ===")
    import usb.core, libusb_package
    backend = libusb_package.get_libusb1_backend()
    
    dev = usb.core.find(idVendor=0x05AC, idProduct=0x1281, backend=backend)
    if not dev:
        print("[-] No device")
        return {}
    
    results = {}
    
    # Standard Apple recovery mode requests
    test_requests = [
        # (bmRequestType, bRequest, wValue, wIndex, length, description)
        (0xC0, 0x00, 0, 0, 4, "Vendor IN req 0x00"),
        (0xC0, 0x01, 0, 0, 4, "Vendor IN req 0x01"),  
        (0xC0, 0x02, 0, 0, 4, "Vendor IN req 0x02"),
        (0xC0, 0x03, 0, 0, 4, "Vendor IN req 0x03"),
        (0xC0, 0x04, 0, 0, 4, "Vendor IN req 0x04"),
        # DFU-like requests on recovery  
        (0xA1, 0x03, 0, 0, 6, "DFU_GETSTATUS"),
        (0xA1, 0x05, 0, 0, 1, "DFU_GETSTATE"),
        # GET_DESCRIPTOR for various types
        (0x80, 0x06, 0x0300, 0, 255, "String desc 0 (languages)"),
        (0x80, 0x06, 0x0301, 0x0409, 255, "String desc 1 (manufacturer)"),
        (0x80, 0x06, 0x0302, 0x0409, 255, "String desc 2 (product)"),
        (0x80, 0x06, 0x0303, 0x0409, 255, "String desc 3 (serial)"),
        (0x80, 0x06, 0x0304, 0x0409, 255, "String desc 4"),
        (0x80, 0x06, 0x0305, 0x0409, 255, "String desc 5"),
    ]
    
    for rt, req, val, idx, length, desc in test_requests:
        try:
            data = dev.ctrl_transfer(rt, req, val, idx, length, timeout=2000)
            hex_data = ' '.join(f'{b:02X}' for b in data[:32])
            print(f"  [{desc}] -> {len(data)} bytes: {hex_data}")
            results[desc] = {'size': len(data), 'data': hex_data}
        except Exception as e:
            err_str = str(e)[:60]
            print(f"  [{desc}] -> Error: {err_str}")
            results[desc] = {'error': err_str}
    
    # Test large control transfer reads (potential info leak)
    print("\n  --- Large read tests ---")
    for size in [256, 512, 1024, 2048, 4096]:
        try:
            data = dev.ctrl_transfer(0xC0, 0x01, 0, 0, size, timeout=2000)
            nonzero = sum(1 for b in data if b != 0)
            print(f"  Vendor req 0x01 size={size}: got {len(data)} bytes, {nonzero} non-zero")
            if nonzero > 0:
                hex_preview = ' '.join(f'{b:02X}' for b in data[:64])
                print(f"    Preview: {hex_preview}")
                results[f'large_read_{size}'] = {
                    'size': len(data), 'nonzero': nonzero,
                    'preview': hex_preview
                }
        except Exception as e:
            print(f"  Vendor req 0x01 size={size}: {str(e)[:60]}")
    
    return results


def main():
    print("=" * 60)
    print("  iBoot Recovery Mode Attack Surface Probe")
    print("  Target: iPhone XR (A12), iBoot-11881.0.193.0.1")
    print(f"  Date: {datetime.now().isoformat()}")
    print("=" * 60)
    
    all_results = {
        'target': 'iPhone XR A12 T8020',
        'iboot': 'iBoot-11881.0.193.0.1',
        'ios': '18.5',
        'timestamp': datetime.now().isoformat()
    }
    
    # Phase 1: USB descriptors
    dev = probe_usb_descriptors()
    if not dev:
        print("\n[!] No device detected. Exiting.")
        return
    
    # Phase 2: Environment variables
    env = probe_irecv_env()
    all_results['environment'] = env
    
    # Phase 3: iBoot commands
    cmds = probe_irecv_commands()
    all_results['commands'] = cmds
    
    # Phase 4: USB control transfers
    ctrl = probe_control_transfers()
    all_results['control_transfers'] = ctrl
    
    # Save results
    outfile = 'tools/checkm8-dualboot/results/iboot_probe.json'
    try:
        import os
        os.makedirs(os.path.dirname(outfile), exist_ok=True)
        with open(outfile, 'w') as f:
            json.dump(all_results, f, indent=2, default=str)
        print(f"\n[+] Results saved to {outfile}")
    except Exception as e:
        print(f"\n[-] Could not save: {e}")
    
    # Summary
    print("\n" + "=" * 60)
    print("  SUMMARY")
    print("=" * 60)
    print(f"  Environment vars read: {len(env)}")
    print(f"  Commands tested: {len(cmds)}")
    print(f"  Control transfers tested: {len(ctrl)}")
    
    # Highlight interesting findings
    interesting = []
    if env.get('debug-enabled'):
        interesting.append(f"debug-enabled = {env['debug-enabled']}")
    if env.get('debug-uarts'):
        interesting.append(f"debug-uarts = {env['debug-uarts']}")
    if env.get('development-cert'):
        interesting.append(f"development-cert present")
    if env.get('boot-args'):
        interesting.append(f"boot-args = {env['boot-args']}")
    if env.get('loadaddr'):
        interesting.append(f"loadaddr = {env['loadaddr']}")
    if env.get('diags'):
        interesting.append(f"diags = {env['diags']}")
    
    # Check command responses
    if cmds.get('help') and 'ERROR' not in cmds['help']:
        interesting.append(f"'help' command returned data")
    if cmds.get('meminfo') and 'ERROR' not in cmds['meminfo']:
        interesting.append(f"'meminfo' returned data")
    
    if interesting:
        print("\n  [!] Interesting findings:")
        for item in interesting:
            print(f"      - {item}")
    else:
        print("\n  No particularly interesting findings yet.")
    
    print("=" * 60)


if __name__ == '__main__':
    main()
