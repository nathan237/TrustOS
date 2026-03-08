#!/usr/bin/env python3
"""Quick loader to send test payloads to DFU device."""
import sys, os, json, time
sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

try:
    import usb.core, usb.util, libusb_package, usb.backend.libusb1
    HAS_USB = True
except:
    HAS_USB = False
    print("WARNING: pyusb/libusb not available - dry run only")

APPLE_VID = 0x05AC; DFU_PID = 0x1227

def connect():
    if not HAS_USB:
        return None
    backend = usb.backend.libusb1.get_backend(find_library=libusb_package.find_library)
    dev = usb.core.find(idVendor=APPLE_VID, idProduct=DFU_PID, backend=backend)
    if dev:
        try: dev.set_configuration()
        except: pass
    return dev

def get_status(dev):
    try:
        data = dev.ctrl_transfer(0xA1, 3, 0, 0, 6, timeout=2000)
        return {"bStatus": data[0], "bState": data[4]}
    except:
        return None

def send_test(dev, payload, block=0):
    try:
        dev.ctrl_transfer(0x21, 1, block, 0, payload, timeout=5000)
        return True
    except:
        return False

def run_test_case(tc_file, dry_run=False):
    with open(tc_file, 'rb') as f:
        payload = f.read()
    print(f"  Payload: {tc_file} ({len(payload)} bytes)")
    print(f"  Preview: {payload[:32].hex()}")
    
    if dry_run or not HAS_USB:
        print("  [DRY RUN] Would send to DFU")
        return
    
    dev = connect()
    if not dev:
        print("  [ERROR] No DFU device found")
        return
    
    # Reset to dfuIDLE
    for _ in range(10):
        st = get_status(dev)
        if st and st['bState'] == 2: break
        if st and st['bState'] == 10:
            dev.ctrl_transfer(0x21, 4, 0, 0, 0)  # CLR_STATUS
        time.sleep(0.05)
    
    # Send payload in chunks
    CHUNK = 2048
    for i in range(0, len(payload), CHUNK):
        chunk = payload[i:i+CHUNK]
        ok = send_test(dev, chunk, block=i//CHUNK)
        if not ok:
            print(f"  [FAIL] DNLOAD block {i//CHUNK} failed")
            return
    
    # Send zero-length to trigger manifest
    send_test(dev, b'', block=(len(payload)//CHUNK)+1)
    
    # Poll status
    t0 = time.time()
    for _ in range(100):
        time.sleep(0.1)
        st = get_status(dev)
        if st:
            print(f"  bState={st['bState']} bStatus={st['bStatus']} t={time.time()-t0:.1f}s")
            if st['bState'] in (2, 10):  # IDLE or ERROR
                break
        else:
            print(f"  [CRASH?] Device not responding at t={time.time()-t0:.1f}s")
            break

if __name__ == "__main__":
    import argparse
    ap = argparse.ArgumentParser()
    ap.add_argument("test_file", nargs="?", help="Specific .bin to test")
    ap.add_argument("--all", action="store_true", help="Run all test cases")
    ap.add_argument("--dry", action="store_true", help="Dry run (no USB)")
    ap.add_argument("--category", type=int, help="Run specific category (1-11)")
    args = ap.parse_args()
    
    manifest = json.loads(open("test_manifest.json").read())
    
    if args.test_file:
        run_test_case(args.test_file, dry_run=args.dry)
    elif args.all:
        for tc in manifest["cases"]:
            print(f"\n--- TC {tc['id']}: {tc['name']} ({tc['severity']}) ---")
            run_test_case(tc["file"], dry_run=args.dry)
    elif args.category:
        cat_names = {
            1: "der_len", 2: "asn1_nesting", 3: "tag", 4: "truncat",
            5: "im4p", 6: "manifest", 7: "octet", 8: "negative",
            9: "target", 10: "cross", 11: "pad"
        }
        prefix = cat_names.get(args.category, "")
        for tc in manifest["cases"]:
            if prefix and prefix in tc["name"]:
                print(f"\n--- TC {tc['id']}: {tc['name']} ({tc['severity']}) ---")
                run_test_case(tc["file"], dry_run=args.dry)
    else:
        # Default: run CRITICAL cases only
        for tc in manifest["cases"]:
            if tc["severity"] == "CRITICAL":
                print(f"\n--- TC {tc['id']}: {tc['name']} ---")
                run_test_case(tc["file"], dry_run=args.dry)
