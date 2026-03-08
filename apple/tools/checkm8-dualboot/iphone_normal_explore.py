#!/usr/bin/env python3
"""
iPhone Normal Mode Explorer via pymobiledevice3
Communicates through Apple's native usbmux driver (no libusb needed).
"""
import sys
import json
import os

try:
    from pymobiledevice3.usbmux import list_devices
    from pymobiledevice3.lockdown import LockdownClient, create_using_usbmux
except ImportError:
    print("pip install pymobiledevice3")
    sys.exit(1)

def main():
    print("=" * 60)
    print("  iPhone Normal Mode Explorer (via usbmux)")
    print("=" * 60)

    devices = list_devices()
    print(f"\nFound {len(devices)} device(s) via usbmux\n")

    if not devices:
        print("No devices found.")
        print("Make sure:")
        print("  - iPhone is connected and unlocked")
        print("  - iTunes or Apple Mobile Device Service is running")
        print("  - You trusted this computer on the iPhone")
        return

    for d in devices:
        print(f"Serial: {d.serial}")
        print(f"Connection: {d.connection_type}")
        print(f"Device ID: {d.device_id}")

        try:
            lockdown = create_using_usbmux(serial=d.serial)
            info = lockdown.all_values

            print(f"\n--- Device Info ---")
            print(f"  Name:          {info.get('DeviceName', '?')}")
            print(f"  Model:         {info.get('ProductType', '?')}")
            print(f"  iOS Version:   {info.get('ProductVersion', '?')}")
            print(f"  Build:         {info.get('BuildVersion', '?')}")
            print(f"  ECID:          {info.get('UniqueChipID', '?')}")
            print(f"  ChipID:        0x{info.get('ChipID', 0):X}")
            print(f"  HW Model:      {info.get('HardwareModel', '?')}")
            print(f"  Board ID:      {info.get('BoardId', '?')}")
            print(f"  WiFi MAC:      {info.get('WiFiAddress', '?')}")
            print(f"  BT MAC:        {info.get('BluetoothAddress', '?')}")
            print(f"  Serial #:      {info.get('SerialNumber', '?')}")
            print(f"  UDID:          {info.get('UniqueDeviceID', '?')}")
            print(f"  CPU Arch:      {info.get('CPUArchitecture', '?')}")
            print(f"  Firmware:      {info.get('FirmwareVersion', '?')}")
            print(f"  BasebandVer:   {info.get('BasebandVersion', '?')}")
            print(f"  DeviceClass:   {info.get('DeviceClass', '?')}")
            print(f"  DeviceColor:   {info.get('DeviceColor', '?')}")
            print(f"  DiskUsage:     {info.get('TotalDiskCapacity', '?')}")
            print(f"  Paired:        {info.get('PasswordProtected', '?')}")

            # ALL keys
            print(f"\n--- All {len(info)} keys ---")
            for k in sorted(info.keys()):
                v = info[k]
                if isinstance(v, bytes):
                    print(f"  {k}: <{len(v)} bytes>")
                elif isinstance(v, (dict, list)):
                    print(f"  {k}: {json.dumps(v, default=str)[:100]}")
                else:
                    print(f"  {k}: {v}")

            # Try to list available services
            print(f"\n--- Available Lockdown Services ---")
            known_services = [
                "com.apple.mobile.diagnostics_relay",
                "com.apple.syslog_relay",
                "com.apple.mobile.installation_proxy",
                "com.apple.instruments.remoteserver",
                "com.apple.mobile.house_arrest",
                "com.apple.crashreportcopier",
                "com.apple.mobile.file_relay",
                "com.apple.pcapd",
                "com.apple.dt.fetchsymbols",
                "com.apple.mobile.MCInstall",
                "com.apple.os_trace_relay",
                "com.apple.iosdiagnostics.relay",
                "com.apple.mobile.heartbeat",
                "com.apple.mobile.debug_image_mount",
                "com.apple.mobile.notification_proxy",
            ]
            for svc in known_services:
                try:
                    s = lockdown.start_lockdown_service(svc)
                    print(f"  [OK] {svc}")
                    s.close()
                except Exception as e:
                    err = str(e)
                    if "InvalidService" in err:
                        print(f"  [--] {svc} (not available)")
                    elif "password" in err.lower() or "pair" in err.lower() or "trust" in err.lower():
                        print(f"  [!!] {svc} (needs trust/pairing)")
                    else:
                        print(f"  [??] {svc}: {err[:60]}")

            # Try diagnostics relay for hardware info
            print(f"\n--- Diagnostics ---")
            try:
                from pymobiledevice3.services.diagnostics import DiagnosticsService
                diag = DiagnosticsService(lockdown)
                try:
                    ioregistry = diag.ioregistry_entry("IODeviceTree:/arm-io", "IOService")
                    print(f"  IODeviceTree arm-io: {json.dumps(ioregistry, default=str)[:200]}")
                except:
                    pass
                try:
                    battery = diag.get_battery()
                    print(f"  Battery: {battery}")
                except:
                    pass
                try:
                    mg = diag.mobilegestalt("UniqueChipID", "ChipID", "BoardId",
                                           "HWModelStr", "ArtworkTraits",
                                           "SupportedDeviceFamilies",
                                           "wi-fi", "bluetooth",
                                           "BasebandCertId", "BasebandKeyHashInformation",
                                           "DiskUsage", "UserAssignedDeviceName")
                    print(f"  MobileGestalt: {json.dumps(mg, default=str)[:300]}")
                except Exception as e:
                    print(f"  MobileGestalt: {e}")
            except Exception as e:
                print(f"  Diagnostics unavailable: {e}")

            # Try syslog
            print(f"\n--- Syslog (5 seconds) ---")
            try:
                from pymobiledevice3.services.syslog import SyslogService
                syslog = SyslogService(lockdown)
                import time
                start = time.time()
                count = 0
                for line in syslog.watch():
                    print(f"  {line.rstrip()[:120]}")
                    count += 1
                    if count >= 20 or time.time() - start > 5:
                        break
                syslog.close()
            except Exception as e:
                print(f"  Syslog: {e}")

        except Exception as e:
            print(f"Lockdown error: {e}")
            import traceback
            traceback.print_exc()

    # Save
    os.makedirs("results", exist_ok=True)
    print(f"\nDone.")

if __name__ == "__main__":
    main()
