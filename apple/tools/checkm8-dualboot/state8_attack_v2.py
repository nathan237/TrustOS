#!/usr/bin/env python3
"""
State 8 Attack Suite — Improved Recovery
==========================================
Tests DFU commands and attack patterns while device is in
dfuMANIFEST-WAIT-RESET (state 8).

Recovery: USB RESET → wait 2s → find device → verify IDLE.
"""
import time, json, array, traceback
from pathlib import Path
from datetime import datetime

import usb.core, usb.util, libusb_package, usb.backend.libusb1

BE = usb.backend.libusb1.get_backend(find_library=libusb_package.find_library)

def log(msg):
    print(f"[{datetime.now().strftime('%H:%M:%S.%f')[:-3]}] {msg}", flush=True)

def find():
    try:
        d = usb.core.find(idVendor=0x05ac, idProduct=0x1227, backend=BE)
        if d:
            try: d.set_configuration()
            except: pass
        return d
    except: return None

def gs(dev):
    try:
        r = dev.ctrl_transfer(0xa1, 3, 0, 0, 6, timeout=2000)
        return r[4] if len(r) >= 6 else None
    except: return None

def dn(dev, data):
    try:
        dev.ctrl_transfer(0x21, 1, 0, 0, array.array('B', data) if isinstance(data, bytes) else data, timeout=5000)
        return True
    except: return False

def ab(dev):
    try: dev.ctrl_transfer(0x21, 6, 0, 0, 0, timeout=2000); return True
    except: return False

def clr(dev):
    try: dev.ctrl_transfer(0x21, 4, 0, 0, 0, timeout=2000); return True
    except: return False

def up(dev, n=256):
    try: return bytes(dev.ctrl_transfer(0xa1, 2, 0, 0, n, timeout=2000))
    except: return None

def ctrl(dev, rt, req, val, idx, data, to=2000):
    try:
        r = dev.ctrl_transfer(rt, req, val, idx, data, timeout=to)
        return "ok", bytes(r) if hasattr(r, '__len__') else r
    except usb.core.USBError as e:
        return "usb_err", str(e)
    except Exception as e:
        return "exc", str(e)

def to_idle(dev, retries=20):
    for _ in range(retries):
        s = gs(dev)
        if s is None:
            time.sleep(0.3)
            dev = find()
            if not dev: return None
            continue
        if s == 2: return dev
        if s == 10: clr(dev); time.sleep(0.1); continue
        if s in (5, 9): ab(dev); time.sleep(0.5); dev = find(); continue
        if s == 8:
            try: dev.reset()
            except: pass
            time.sleep(2.5)
            dev = find()
            if not dev: return None
            continue
        if s in (3, 6, 7): time.sleep(0.2); continue
        ab(dev); time.sleep(0.5); dev = find()
        if not dev: return None
    return None

def enter_state8(dev):
    """DNLOAD + zero-length DNLOAD → poll for state 8."""
    if not dn(dev, bytes(2048)): return None, False
    s = gs(dev)
    if s != 5: return dev, False
    if not dn(dev, b''): return dev, False
    for _ in range(50):
        s = gs(dev)
        if s == 8: return dev, True
        if s in (2, 10): return dev, False
        time.sleep(0.01)
    return dev, (gs(dev) == 8)

def recover(dev):
    """Recover from any state back to IDLE."""
    try: dev.reset()
    except: pass
    time.sleep(2.5)
    for _ in range(10):
        dev = find()
        if dev:
            dev = to_idle(dev)
            if dev: return dev
        time.sleep(1)
    return None

def alive():
    try: return find() is not None
    except: return False

# ============================================================
results = []

def rec(test, detail, result, dev_alive, notes=""):
    e = {"test": test, "detail": detail, "result": str(result)[:80],
         "alive": dev_alive, "notes": notes}
    results.append(e)
    marker = " *** CRASH ***" if not dev_alive else (" [!]" if notes else "")
    log(f"  {test:30s} {detail:20s} → alive={dev_alive}{marker} {notes}")

# ============================================================
def main():
    log("State 8 Attack Suite v2")
    dev = find()
    if not dev:
        log("No device. Enter DFU mode."); return
    dev = to_idle(dev)
    if not dev:
        log("Cannot reach IDLE"); return
    log(f"Device ready, state={gs(dev)}")

    # ============================================================
    # TEST 1: Individual DFU commands in state 8
    # ============================================================
    log("\n=== TEST 1: DFU Commands in State 8 ===")
    
    commands = {
        "GET_STATUS":    lambda d: ctrl(d, 0xa1, 3, 0, 0, 6),
        "GET_STATE":     lambda d: ctrl(d, 0xa1, 5, 0, 0, 1),
        "ABORT":         lambda d: (ab(d), "ok" if ab(d) else "fail"),
        "CLR_STATUS":    lambda d: (clr(d), "ok" if clr(d) else "fail"),
        "DETACH":        lambda d: ctrl(d, 0x21, 0, 0, 0, 0),
        "UPLOAD_64":     lambda d: ("ok", up(d, 64)),
        "UPLOAD_2048":   lambda d: ("ok", up(d, 2048)),
        "DNLOAD_0":      lambda d: ("ok" if dn(d, b'') else "fail", None),
        "DNLOAD_16":     lambda d: ("ok" if dn(d, bytes(16)) else "fail", None),
        "DNLOAD_2048":   lambda d: ("ok" if dn(d, bytes(2048)) else "fail", None),
        "DNLOAD_0xDEAD": lambda d: ("ok" if dn(d, b'\xDE\xAD' * 1024) else "fail", None),
    }
    
    for name, action in commands.items():
        dev = to_idle(dev) or recover(dev) if dev else None
        if not dev:
            log("Lost device, waiting..."); dev = recover(find()); 
            if not dev: log("Can't recover"); break
        
        dev, ok = enter_state8(dev)
        if not ok:
            s = gs(dev) if dev else None
            log(f"  {name}: can't reach state 8 (state={s})"); 
            dev = recover(dev) if dev else None; continue
        
        # Execute command in state 8
        try:
            result = action(dev)
        except Exception as e:
            result = ("exc", str(e))
        
        time.sleep(0.1)
        s_after = gs(dev) if dev else None
        dev_alive = alive()
        
        notes = ""
        if s_after is not None and s_after != 8:
            notes = f"STATE_CHANGED→{s_after}!"
        
        # Check for upload data
        if "UPLOAD" in name and isinstance(result, tuple) and result[1] is not None:
            data = result[1]
            if isinstance(data, bytes) and any(b != 0 for b in data):
                notes += f" NONZERO_DATA:{data[:16].hex()}"
        
        rec(name, f"s_after={s_after}", result, dev_alive, notes)
        
        dev = recover(dev) if dev else None
    
    # ============================================================
    # TEST 2: Rapid-fire sequences in state 8
    # ============================================================
    log("\n=== TEST 2: Rapid-Fire Sequences ===")
    
    seqs = {
        "DN+AB+DN+AB": [
            (0x21, 1, 0, 0, bytes(2048)),
            (0x21, 6, 0, 0, 0),
            (0x21, 1, 0, 0, bytes(2048)),
            (0x21, 6, 0, 0, 0),
        ],
        "5xGS": [(0xa1, 3, 0, 0, 6)] * 5,
        "DN+UP": [
            (0x21, 1, 0, 0, b'\xDE\xAD\xBE\xEF' * 16),
            (0xa1, 2, 0, 0, 256),
        ],
        "UP+DN+UP": [
            (0xa1, 2, 0, 0, 256),
            (0x21, 1, 0, 0, bytes(2048)),
            (0xa1, 2, 0, 0, 256),
        ],
    }
    
    for name, seq in seqs.items():
        dev = to_idle(dev) or recover(dev) if dev else None
        if not dev: dev = recover(find()); 
        if not dev: break
        
        dev, ok = enter_state8(dev)
        if not ok: dev = recover(dev) if dev else None; continue
        
        seq_results = []
        for rt, req, val, idx, data in seq:
            s, r = ctrl(dev, rt, req, val, idx, data)
            seq_results.append(f"{s}")
        
        s_after = gs(dev) if dev else None
        dev_alive = alive()
        notes = ""
        if s_after is not None and s_after != 8: notes = f"STATE→{s_after}!"
        
        rec(name, f"s={s_after}", ";".join(seq_results), dev_alive, notes)
        dev = recover(dev) if dev else None
    
    # ============================================================
    # TEST 3: Vendor requests + weird USB in state 8
    # ============================================================
    log("\n=== TEST 3: Non-Standard USB in State 8 ===")
    
    weird = {
        "GET_DESC_DEV":    (0x80, 0x06, 0x0100, 0, 18),
        "GET_DESC_CFG":    (0x80, 0x06, 0x0200, 0, 64),
        "GET_DESC_STR0":   (0x80, 0x06, 0x0300, 0, 4),
        "GET_DESC_STR1":   (0x80, 0x06, 0x0301, 0x0409, 64),
        "SET_ADDRESS":     (0x00, 0x05, 0x02, 0, 0),
        "SET_FEAT_EP":     (0x02, 0x03, 0, 0x80, 0),   # SET_FEATURE(HALT) on EP0 IN
        "CLR_FEAT_EP":     (0x02, 0x01, 0, 0x80, 0),
        "VendorIN":        (0xc0, 0x01, 0, 0, 64),
        "VendorOUT":       (0x40, 0x01, 0, 0, bytes(64)),
        "DFU_wIdx1":       (0x21, 1, 0, 1, bytes(64)),   # DNLOAD with wrong wIndex
        "DFU_wIdx255":     (0x21, 1, 0, 0xff, bytes(64)),
        "DFU_badReq":      (0x21, 0x10, 0, 0, 0),
    }
    
    for name, (rt, req, val, idx, data) in weird.items():
        dev = to_idle(dev) or recover(dev) if dev else None
        if not dev: dev = recover(find())
        if not dev: break
        
        dev, ok = enter_state8(dev)
        if not ok: dev = recover(dev) if dev else None; continue
        
        s, r = ctrl(dev, rt, req, val, idx, data)
        s_after = gs(dev) if dev else None
        dev_alive = alive()
        notes = ""
        if s == "ok": notes = "ACCEPTED!"
        if s_after is not None and s_after != 8: notes += f" STATE→{s_after}"
        if not dev_alive: notes = "CRASHED!"
        
        # Check descriptor data
        if "GET_DESC" in name and s == "ok" and isinstance(r, bytes) and len(r) > 0:
            notes += f" data={r[:16].hex()}"
        
        rec(name, "", f"{s}:{str(r)[:40]}", dev_alive, notes)
        dev = recover(dev) if dev else None
    
    # ============================================================
    # TEST 4: Upload after DNLOAD pattern in state 8
    # ============================================================
    log("\n=== TEST 4: DNLOAD→UPLOAD in State 8 (data leak?) ===")
    
    for pattern_name, pattern in [("0xAA", b'\xAA'*2048), ("0xDEAD", b'\xDE\xAD'*1024)]:
        dev = to_idle(dev) or recover(dev) if dev else None
        if not dev: dev = recover(find())
        if not dev: break
        
        # First DNLOAD the pattern normally
        if not dn(dev, pattern): continue
        s = gs(dev)
        if s != 5: dev = recover(dev); continue
        ab(dev); time.sleep(0.5); dev = find()
        if not dev: continue
        dev = to_idle(dev)
        if not dev: continue
        
        # Now trigger manifest
        dev, ok = enter_state8(dev)
        if not ok: dev = recover(dev) if dev else None; continue
        
        # UPLOAD in state 8 — might read stale buffer?
        data = up(dev, 2048)
        dev_alive = alive()
        notes = ""
        if data:
            nonzero = sum(1 for b in data if b != 0)
            notes = f"nonzero={nonzero}/{len(data)}"
            if nonzero > 0:
                notes += f" first16={data[:16].hex()}"
        
        rec("UP_after_"+pattern_name, "", f"got {len(data) if data else 0}B", dev_alive, notes)
        dev = recover(dev) if dev else None
    
    # ============================================================
    # SUMMARY
    # ============================================================
    log(f"\n{'='*60}")
    log(f"RESULTS: {len(results)} tests")
    crashes = [r for r in results if not r['alive']]
    state_changes = [r for r in results if 'STATE' in r.get('notes','')]
    nonzero = [r for r in results if 'NONZERO' in r.get('notes','')]
    accepted = [r for r in results if 'ACCEPTED' in r.get('notes','')]
    
    log(f"  Crashes: {len(crashes)}")
    log(f"  State changes: {len(state_changes)}")
    log(f"  Non-zero upload data: {len(nonzero)}")
    log(f"  Unexpected accepts: {len(accepted)}")
    
    if crashes:
        log("\nCRASHES:")
        for c in crashes: log(f"  {c['test']} {c['detail']}")
    if state_changes:
        log("\nSTATE CHANGES:")
        for c in state_changes: log(f"  {c['test']} {c['notes']}")
    if nonzero:
        log("\nNON-ZERO UPLOADS:")
        for c in nonzero: log(f"  {c['test']} {c['notes']}")
    
    out = Path("results"); out.mkdir(exist_ok=True)
    (out / "state8_v2.json").write_text(json.dumps(results, indent=2))
    log(f"\nSaved to results/state8_v2.json")

if __name__ == "__main__":
    main()
