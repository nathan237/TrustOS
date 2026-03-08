#!/usr/bin/env python3
"""
iboot_recon.py - iBoot Recovery Mode Reconnaissance & Attack Surface Analysis

Target: iPhone XR (A12 T8020) running iBoot-11881.0.193.0.1 (iOS 18.5)
Purpose: Map all accessible iBoot commands, environment variables, and behaviors
         to identify potential exploitation vectors.

Recovery mode exposes an iBoot command interface via USB (Apple class 0xFF).
This script systematically probes that interface.
"""

import sys
import time
import json
import struct
import os
from datetime import datetime

# Results storage
RESULTS = {
    "timestamp": datetime.now().isoformat(),
    "device": {
        "cpid": "0x8020",
        "model": "iPhone XR",
        "iboot": "iBoot-11881.0.193.0.1",
        "ios_estimate": "18.5.x",
        "serial": "DX6CFCBNKXKN",
    },
    "env_vars": {},
    "commands_tested": {},
    "usb_interfaces": [],
    "attack_surface": [],
}


def get_usb_device():
    """Find and return the recovery mode USB device."""
    import usb.core
    import libusb_package
    backend = libusb_package.get_libusb1_backend()
    
    # Recovery mode PID for iPhone XR
    dev = usb.core.find(idVendor=0x05AC, idProduct=0x1281, backend=backend)
    if not dev:
        # Try other recovery PIDs
        for pid in [0x1280, 0x1282, 0x1283, 0x1284]:
            dev = usb.core.find(idVendor=0x05AC, idProduct=pid, backend=backend)
            if dev:
                break
    return dev


def get_irecv_client():
    """Get pymobiledevice3 IRecv client."""
    try:
        from pymobiledevice3.irecv import IRecv
        return IRecv()
    except Exception as e:
        print(f"[-] IRecv init failed: {e}")
        return None


# =====================================================
# Phase 1: Environment Variable Enumeration
# =====================================================
def phase1_env_enum(client):
    """Enumerate all known and potential iBoot environment variables."""
    print("\n" + "=" * 60)
    print("  PHASE 1: Environment Variable Enumeration")
    print("=" * 60)
    
    # Known iBoot env vars (comprehensive list)
    known_vars = [
        # Device identity
        "CPID", "CPRV", "CPFM", "SCEP", "BDID", "ECID", "IBFL",
        "SRNM", "IMEI", "MEID", "NONC", "SNON", "PWND",
        # Build info
        "build-version", "build-style", "firmware-version",
        "hardware-model", "model-number", "region-info",
        "serial-number", "mlb-serial-number",
        # Boot config
        "auto-boot", "boot-command", "boot-args", "boot-device",
        "boot-partition", "boot-path", "boot-ramdisk",
        # Security
        "com.apple.System.boot-nonce", "effective-production-status-ap",
        "effective-security-mode-ap", "security-domain",
        "cert-production-status", "cert-security-mode",
        "demoted-production-status", "uid-aes-key-generation",
        # Display / hardware
        "darkboot", "device-color", "device-color-policy",
        "backlight-level", "display-rotation", "display-scale",
        # DFU / Recovery
        "recoveryos-boot-mode", "recovery-boot-mode",
        "dfu-boot-mode", "force-dfu",
        # USB
        "usb-enabled", "usb-device-class",
        # Crypto / SEP
        "sep-debug-token", "sep-security-domain",
        # Filesystem
        "root-live-fs-upgrade", "upgrade-fallback-boot-command",
        # Misc
        "obliteration", "obliteration-willingness",
        "debug-enabled", "debug-gg", "development-cert",
        "allow-mix-and-match", "permit-unverified-boot",
        "preserve-debuggability", "research-enabled",
        "modified-device-key",
        # Potentially interesting undocumented
        "diags", "diag-log", "loadaddr", "filesize",
        "idle-off", "display-timing", "panic-info",
        "iBoot-version", "chip-id", "board-id",
        "nonce-seeds", "boot-manifest-hash",
        "SystemAudioVolume", "nvram-proxy-data",
        "ota-breadcrumbs",
    ]
    
    print(f"[*] Testing {len(known_vars)} environment variables...")
    found = 0
    
    for var in known_vars:
        try:
            val = client.getenv(var)
            if val is not None:
                # Decode if bytes
                if isinstance(val, (bytes, bytearray)):
                    try:
                        val_str = val.decode('utf-8', errors='replace').rstrip('\x00')
                    except:
                        val_str = val.hex()
                else:
                    val_str = str(val)
                
                RESULTS["env_vars"][var] = val_str
                found += 1
                
                # Highlight security-relevant vars
                security_relevant = [
                    "debug-enabled", "development-cert", "PWND",
                    "allow-mix-and-match", "permit-unverified-boot",
                    "preserve-debuggability", "research-enabled",
                    "demoted-production-status", "boot-args",
                    "effective-security-mode-ap", "security-domain",
                ]
                
                marker = " [!!! SECURITY RELEVANT]" if var in security_relevant else ""
                print(f"  [+] {var} = {val_str}{marker}")
        except Exception:
            pass
    
    print(f"\n[*] Found {found}/{len(known_vars)} variables")
    return found


# =====================================================
# Phase 2: USB Interface Analysis
# =====================================================
def phase2_usb_analysis(dev):
    """Deep analysis of USB interfaces and endpoints."""
    print("\n" + "=" * 60)
    print("  PHASE 2: USB Interface & Endpoint Analysis")
    print("=" * 60)
    
    import usb.util
    
    for cfg in dev:
        cfg_info = {
            "config_value": cfg.bConfigurationValue,
            "num_interfaces": cfg.bNumInterfaces,
            "interfaces": []
        }
        print(f"\n  Config {cfg.bConfigurationValue}:")
        
        for intf in cfg:
            intf_info = {
                "number": intf.bInterfaceNumber,
                "alt_setting": intf.bAlternateSetting,
                "class": f"0x{intf.bInterfaceClass:02X}",
                "subclass": f"0x{intf.bInterfaceSubClass:02X}",
                "protocol": f"0x{intf.bInterfaceProtocol:02X}",
                "endpoints": []
            }
            
            class_name = {
                0xFE: "Application Specific (DFU-like)",
                0xFF: "Vendor Specific (iBoot control)",
            }.get(intf.bInterfaceClass, "Unknown")
            
            protocol_name = {
                0x02: "DFU Protocol",
                0x51: "iBoot Serial Console",
            }.get(intf.bInterfaceProtocol, "Unknown")
            
            print(f"  Interface {intf.bInterfaceNumber} (alt {intf.bAlternateSetting}):")
            print(f"    Class: {intf.bInterfaceClass:#04x} ({class_name})")
            print(f"    SubClass: {intf.bInterfaceSubClass:#04x}")
            print(f"    Protocol: {intf.bInterfaceProtocol:#04x} ({protocol_name})")
            
            for ep in intf:
                direction = "IN" if usb.util.endpoint_direction(ep.bEndpointAddress) == usb.util.ENDPOINT_IN else "OUT"
                ep_info = {
                    "address": f"0x{ep.bEndpointAddress:02X}",
                    "direction": direction,
                    "type": usb.util.endpoint_type(ep.bmAttributes),
                    "max_packet_size": ep.wMaxPacketSize,
                }
                intf_info["endpoints"].append(ep_info)
                
                transfer_type = {
                    0: "Control", 1: "Isochronous", 2: "Bulk", 3: "Interrupt"
                }.get(usb.util.endpoint_type(ep.bmAttributes), "Unknown")
                
                print(f"    EP {ep.bEndpointAddress:#04x} [{direction}] {transfer_type} maxpkt={ep.wMaxPacketSize}")
            
            cfg_info["interfaces"].append(intf_info)
        
        RESULTS["usb_interfaces"].append(cfg_info)
    
    # Check DFU functional descriptor  
    print("\n  [*] Checking DFU attributes...")
    try:
        # Read DFU functional descriptor (type 0x21)
        dfu_desc = dev.ctrl_transfer(0x80 | 0x01, 0x06, 0x2100, 0, 64, timeout=1000)
        print(f"  [+] DFU Functional Descriptor: {dfu_desc.tobytes().hex()}")
        if len(dfu_desc) >= 7:
            attrs = dfu_desc[2]
            print(f"    bmAttributes: {attrs:#04x}")
            print(f"      bitCanDnload: {bool(attrs & 0x01)}")
            print(f"      bitCanUpload: {bool(attrs & 0x02)}")
            print(f"      bitManifestationTolerant: {bool(attrs & 0x04)}")
            print(f"      bitWillDetach: {bool(attrs & 0x08)}")
            detach_timeout = struct.unpack('<H', dfu_desc[3:5])[0]
            transfer_size = struct.unpack('<H', dfu_desc[5:7])[0]
            print(f"    wDetachTimeOut: {detach_timeout}")
            print(f"    wTransferSize: {transfer_size}")
            RESULTS["dfu_descriptor"] = {
                "can_download": bool(attrs & 0x01),
                "can_upload": bool(attrs & 0x02),
                "transfer_size": transfer_size,
            }
    except Exception as e:
        print(f"  [-] DFU descriptor read failed: {e}")


# =====================================================
# Phase 3: iBoot Command Probing
# =====================================================
def phase3_command_probe(client):
    """Probe iBoot for available commands and their responses."""
    print("\n" + "=" * 60)
    print("  PHASE 3: iBoot Command Probing")
    print("=" * 60)
    
    # Known iBoot commands (from public research & documentation)
    # These are sent via the USB control interface
    commands = [
        # Info commands (safe)
        ("getenv build-version", "Get iBoot build version"),
        ("getenv auto-boot", "Get auto-boot setting"),
        
        # Potentially dangerous - commented out for safety
        # ("setenv auto-boot false", "Disable auto-boot"),
        # ("saveenv", "Save environment to NVRAM"),
        # ("reboot", "Reboot device"),
        # ("reset", "Reset device"),
        
        # File/image commands (read-only probes)
        ("help", "List available commands"),
        ("version", "Get iBoot version"),
        ("printenv", "Print all environment variables"),
        
        # Memory commands (DANGEROUS - only probe existence)
        # These could crash the device if arguments are wrong
        # ("md 0x0 0x10", "Memory dump (test)"),
        # ("mw 0x0 0x0", "Memory write (test)"),
        
        # Boot chain
        ("bgcolor 0 0 0", "Set background color (harmless test)"),
    ]
    
    print(f"[*] Testing {len(commands)} iBoot commands...")
    print("[!] Using safe, read-only commands only\n")
    
    for cmd, desc in commands:
        print(f"  Testing: '{cmd}' ({desc})")
        try:
            response = client.send_command(cmd)
            if response:
                if isinstance(response, (bytes, bytearray)):
                    resp_str = response.decode('utf-8', errors='replace').rstrip('\x00')
                else:
                    resp_str = str(response)
                print(f"    Response: {resp_str[:200]}")
                RESULTS["commands_tested"][cmd] = {"status": "responded", "response": resp_str[:500]}
            else:
                print(f"    Response: (empty/none)")
                RESULTS["commands_tested"][cmd] = {"status": "empty_response"}
        except Exception as e:
            error_str = str(e)
            print(f"    Error: {error_str}")
            RESULTS["commands_tested"][cmd] = {"status": "error", "error": error_str}
        
        time.sleep(0.5)  # Don't overwhelm iBoot


# =====================================================
# Phase 4: Recovery Mode Protocol Analysis
# =====================================================
def phase4_protocol_analysis(dev):
    """Analyze the recovery mode USB protocol for anomalies."""
    print("\n" + "=" * 60)
    print("  PHASE 4: Recovery Mode Protocol Analysis")
    print("=" * 60)
    
    import usb.core
    
    # Test various control transfers that iBoot might accept
    test_requests = [
        # (bmRequestType, bRequest, wValue, wIndex, data_or_length, description)
        (0xC0, 0x00, 0, 0, 64, "Vendor IN: read status"),
        (0xC0, 0x01, 0, 0, 64, "Vendor IN: read data 0x01"),
        (0xC0, 0x02, 0, 0, 64, "Vendor IN: read data 0x02"),
        (0xC0, 0x03, 0, 0, 64, "Vendor IN: read data 0x03"),
        (0xC0, 0x04, 0, 0, 64, "Vendor IN: read data 0x04"),
        (0xC1, 0x00, 0, 0, 64, "Vendor+Intf IN: read 0x00"),
        (0xC1, 0x01, 0, 0, 64, "Vendor+Intf IN: read 0x01"),
        (0xA1, 0x03, 0, 0, 6, "DFU GETSTATUS"),
        (0xA1, 0x05, 0, 0, 1, "DFU GETSTATE"),
        (0x80, 0x06, 0x0300, 0, 255, "String Descriptor 0 (languages)"),
        (0x80, 0x06, 0x0301, 0x0409, 255, "String Descriptor 1"),
        (0x80, 0x06, 0x0302, 0x0409, 255, "String Descriptor 2"),
        (0x80, 0x06, 0x0303, 0x0409, 255, "String Descriptor 3"),
        (0x80, 0x06, 0x0304, 0x0409, 255, "String Descriptor 4"),
    ]
    
    print(f"[*] Testing {len(test_requests)} USB control transfers...\n")
    
    for req_type, req, val, idx, length, desc in test_requests:
        try:
            data = dev.ctrl_transfer(req_type, req, val, idx, length, timeout=2000)
            hex_data = data.tobytes().hex() if hasattr(data, 'tobytes') else bytes(data).hex()
            
            # Try to decode as string
            try:
                str_data = bytes(data).decode('utf-8', errors='replace')
            except:
                str_data = ""
            
            print(f"  [+] {desc}")
            print(f"      -> {len(data)} bytes: {hex_data[:80]}")
            if str_data and str_data.isprintable():
                print(f"      -> String: {str_data[:80]}")
            
            RESULTS.setdefault("protocol_responses", {})[desc] = {
                "success": True,
                "length": len(data),
                "hex": hex_data[:200],
            }
        except usb.core.USBError as e:
            if "timeout" in str(e).lower():
                print(f"  [-] {desc} -> TIMEOUT")
                status = "timeout"
            elif "pipe" in str(e).lower() or "stall" in str(e).lower():
                print(f"  [~] {desc} -> STALL (not supported)")
                status = "stall"
            else:
                print(f"  [-] {desc} -> {e}")
                status = str(e)
            
            RESULTS.setdefault("protocol_responses", {})[desc] = {
                "success": False,
                "status": status,
            }
        except Exception as e:
            print(f"  [-] {desc} -> {e}")
        
        time.sleep(0.2)


# =====================================================
# Phase 5: Attack Surface Assessment
# =====================================================
def phase5_attack_surface():
    """Analyze gathered data and produce attack surface assessment."""
    print("\n" + "=" * 60)
    print("  PHASE 5: Attack Surface Assessment")
    print("=" * 60)
    
    findings = []
    
    # Check for debug/demoted status
    env = RESULTS.get("env_vars", {})
    
    if env.get("debug-enabled") == "true":
        findings.append({
            "severity": "CRITICAL",
            "finding": "Debug mode enabled",
            "detail": "Device has debug-enabled=true, may allow additional attack vectors",
        })
    
    if env.get("PWND"):
        findings.append({
            "severity": "HIGH",
            "finding": "Device shows PWND flag",
            "detail": f"PWND={env['PWND']} - device was previously exploited",
        })
    
    if env.get("development-cert") == "true":
        findings.append({
            "severity": "HIGH",
            "finding": "Development certificate present",
            "detail": "Device may accept development-signed images",
        })
    
    if env.get("demoted-production-status"):
        findings.append({
            "severity": "HIGH",
            "finding": "Demoted production status",
            "detail": f"Status: {env['demoted-production-status']}",
        })
    
    if env.get("permit-unverified-boot"):
        findings.append({
            "severity": "CRITICAL",
            "finding": "Unverified boot permitted",
            "detail": "Device may boot unsigned images",
        })
    
    # Check boot-args for interesting flags
    boot_args = env.get("boot-args", "")
    if boot_args:
        interesting_args = ["-v", "debug=", "serial=", "cs_enforcement_disable",
                          "amfi_get_out_of_my_way", "PE_i_can_has_debugger"]
        for arg in interesting_args:
            if arg in boot_args:
                findings.append({
                    "severity": "HIGH",
                    "finding": f"Interesting boot arg: {arg}",
                    "detail": f"boot-args contains '{arg}'",
                })
    
    # Check CPFM (chip fusing mode)
    cpfm = env.get("CPFM", "03")
    if cpfm in ["00", "01"]:
        findings.append({
            "severity": "CRITICAL",
            "finding": "Device is NOT production-fused",
            "detail": f"CPFM={cpfm}, device may accept unsigned code",
        })
    else:
        findings.append({
            "severity": "INFO",
            "finding": "Device is production-fused",
            "detail": f"CPFM={cpfm}, standard production security",
        })
    
    # Check what commands worked
    cmds = RESULTS.get("commands_tested", {})
    for cmd, result in cmds.items():
        if result.get("status") == "responded":
            findings.append({
                "severity": "MEDIUM",
                "finding": f"iBoot command '{cmd}' is active",
                "detail": f"Response: {result.get('response', '')[:100]}",
            })
    
    # Check protocol responses
    protos = RESULTS.get("protocol_responses", {})
    for desc, result in protos.items():
        if result.get("success"):
            findings.append({
                "severity": "MEDIUM",
                "finding": f"USB request '{desc}' accepted",
                "detail": f"Returned {result.get('length', 0)} bytes",
            })
    
    # Known CVEs affecting this version
    known_cves = [
        {
            "severity": "CRITICAL",
            "finding": "CVE-2026-20700: Memory corruption (CISA KEV, exploited ITW)",
            "detail": "Memory corruption via improper state management. Fixed in iOS 26.3/18.7.5. "
                      "Requires memory write capability. Exploited in sophisticated targeted attacks. "
                      "Affects all iOS < 26.3 including our 18.5.",
        },
        {
            "severity": "HIGH",
            "finding": "CVE-2026-20677: Sandbox escape via symlink race (CVSS 9.0)",
            "detail": "Race condition in symbolic link handling allows sandbox bypass. "
                      "Fixed in iOS 26.3/18.7.5. Affects our 18.5.",
        },
        {
            "severity": "HIGH",
            "finding": "CVE-2026-20667: Sandbox breakout via logic flaw (CVSS 8.8)",
            "detail": "Logic issue allows app to break out of sandbox. "
                      "Fixed in iOS 26.3. Affects our 18.5.",
        },
        {
            "severity": "HIGH",
            "finding": "CVE-2026-20660: Arbitrary file write via path handling (CVSS 7.5)",
            "detail": "Path handling issue allows remote user to write arbitrary files. "
                      "Fixed in iOS 26.3/18.7.5. Affects our 18.5.",
        },
        {
            "severity": "MEDIUM",
            "finding": "CVE-2025-24198: WebKit Type Confusion (Aether Chain stage 1)",
            "detail": "Type confusion in WebKit. Fixed in 18.3.2. "
                      "Our 18.5 has this PATCHED. NOT exploitable.",
        },
        {
            "severity": "MEDIUM",
            "finding": "CVE-2025-24203: VM_BEHAVIOR_ZERO_WIRED_PAGES kernel bug",
            "detail": "Kernel vulnerability in VM subsystem. Fixed in 18.3.2. "
                      "Our 18.5 has this PATCHED. NOT exploitable.",
        },
    ]
    
    findings.extend(known_cves)
    RESULTS["attack_surface"] = findings
    
    # Print summary
    print("\n--- Attack Surface Findings ---\n")
    
    by_severity = {}
    for f in findings:
        sev = f["severity"]
        by_severity.setdefault(sev, []).append(f)
    
    for sev in ["CRITICAL", "HIGH", "MEDIUM", "LOW", "INFO"]:
        items = by_severity.get(sev, [])
        if items:
            print(f"\n  [{sev}] ({len(items)} findings)")
            for item in items:
                print(f"    - {item['finding']}")
                print(f"      {item['detail'][:120]}")
    
    return findings


def save_results():
    """Save all results to JSON."""
    results_dir = os.path.join(os.path.dirname(__file__), "results")
    os.makedirs(results_dir, exist_ok=True)
    
    output_path = os.path.join(results_dir, "iboot_recon_results.json")
    with open(output_path, 'w') as f:
        json.dump(RESULTS, f, indent=2, default=str)
    
    print(f"\n[+] Results saved to: {output_path}")
    return output_path


def main():
    print("=" * 60)
    print("  iBoot Recovery Mode Reconnaissance")
    print("  Target: iPhone XR - iBoot-11881.0.193.0.1 (iOS 18.5)")
    print("  Date: " + datetime.now().strftime("%Y-%m-%d %H:%M:%S"))
    print("=" * 60)
    
    # Get USB device
    print("\n[*] Connecting to device...")
    dev = get_usb_device()
    if not dev:
        print("[-] No device in Recovery mode found!")
        print("    Put device in Recovery mode first")
        return
    
    print(f"[+] Found device: VID:PID = {dev.idVendor:04X}:{dev.idProduct:04X}")
    
    # Get IRecv client
    client = get_irecv_client()
    if not client:
        print("[-] Could not connect via pymobiledevice3")
        print("    Falling back to USB-only analysis")
    
    # Run phases
    if client:
        phase1_env_enum(client)
        phase3_command_probe(client)
    
    phase2_usb_analysis(dev)
    phase4_protocol_analysis(dev)
    phase5_attack_surface()
    
    # Save
    output = save_results()
    
    print("\n" + "=" * 60)
    print("  RECONNAISSANCE COMPLETE")
    print("=" * 60)


if __name__ == '__main__':
    main()
