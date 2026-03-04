#!/usr/bin/env python3
"""
T8020 B1 SecureROM -- DFU Payload Tester
==========================================
Sends malformed IMG4/DER payloads to a real DFU device and monitors behavior.

Two modes:
  DNLOAD (default, safe):
    DNLOAD payload -> GET_STATUS -> ABORT -> back to dfuIDLE
    Device stays in DFU. Can run all 79 payloads without losing device.
    Detects: crashes during DNLOAD, state anomalies, timing differences.

  MANIFEST (destructive, use for follow-up on interesting cases):
    DNLOAD payload -> zero-length DNLOAD -> manifest processing -> device resets
    Triggers actual IMG4 parser. Device goes to Recovery after each test.
    Requires manual DFU re-entry after each test.

Usage:
  python dfu_payload_tester.py --all            # Safe scan of all 79 payloads
  python dfu_payload_tester.py --severity HIGH  # DNLOAD-only HIGH+ cases
  python dfu_payload_tester.py --id 0 3 8       # Specific test IDs
  python dfu_payload_tester.py --mode manifest --id 13  # Full manifest test
"""

import usb.core, usb.util, usb.backend.libusb1
import libusb_package
import array, time, json, sys, os, argparse
from datetime import datetime
from pathlib import Path

# ============================================================================
# Constants
# ============================================================================
APPLE_VID     = 0x05AC
DFU_PID       = 0x1227

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
RESULTS_FILE = Path(__file__).parent / "dfu_test_results.json"
LOG_FILE = Path(__file__).parent / "dfu_test_log.txt"

# ============================================================================
# Logging
# ============================================================================
log_fh = None

def log(msg, level="INFO"):
    global log_fh
    ts = datetime.now().strftime('%H:%M:%S.%f')[:-3]
    line = f"[{ts}] [{level:5s}] {msg}"
    print(line, flush=True)
    if log_fh:
        log_fh.write(line + "\n")
        log_fh.flush()

# ============================================================================
# USB / DFU Operations (adapted from checkm8_t8020.py)
# ============================================================================
def get_backend():
    return usb.backend.libusb1.get_backend(find_library=libusb_package.find_library)

def find_dfu(retries=3, delay=0.3):
    for _ in range(retries):
        try:
            dev = usb.core.find(idVendor=APPLE_VID, idProduct=DFU_PID, backend=get_backend())
            if dev:
                try:
                    dev.set_configuration()
                except:
                    pass
                return dev
        except:
            pass
        time.sleep(delay)
    return None

def dfu_get_status(dev, timeout=2000):
    try:
        r = dev.ctrl_transfer(0xA1, DFU_GETSTATUS, 0, 0, 6, timeout=timeout)
        if len(r) >= 6:
            return {"bStatus": r[0], "bPollTimeout": r[1] | (r[2] << 8) | (r[3] << 16),
                    "bState": r[4], "iString": r[5],
                    "state_name": DFU_STATES.get(r[4], f"unknown({r[4]})")}
        return None
    except usb.core.USBTimeoutError:
        return "TIMEOUT"
    except usb.core.USBError as e:
        return f"USB_ERROR:{e.errno}"
    except:
        return None

def dfu_clear_status(dev):
    try:
        dev.ctrl_transfer(0x21, DFU_CLRSTATUS, 0, 0, 0, timeout=2000)
    except:
        pass

def dfu_abort(dev):
    try:
        dev.ctrl_transfer(0x21, DFU_ABORT, 0, 0, 0, timeout=500)
    except:
        pass

def dfu_dnload(dev, data, timeout=5000):
    try:
        if isinstance(data, (bytes, bytearray)):
            data = array.array('B', data)
        dev.ctrl_transfer(0x21, DFU_DNLOAD, 0, 0, data, timeout=timeout)
        return True
    except usb.core.USBTimeoutError:
        return "TIMEOUT"
    except usb.core.USBError as e:
        return f"USB_ERROR:{e.errno}"

def ensure_idle(dev):
    """Reset device to dfuIDLE. Returns device or None."""
    for attempt in range(20):
        st = dfu_get_status(dev)
        if st is None or isinstance(st, str):
            time.sleep(0.8)
            dev = find_dfu(retries=10, delay=0.5)
            if not dev:
                return None
            continue
        if st["bState"] == 2:  # dfuIDLE
            return dev
        elif st["bState"] == 10:  # dfuERROR
            dfu_clear_status(dev)
            time.sleep(0.1)
        elif st["bState"] in (6, 7, 8):  # dfuMANIFEST states
            # Wait for manifest to complete, then it should go to dfuERROR or dfuIDLE
            time.sleep(0.5)
            # If still in manifest after a few tries, try abort
            if attempt > 3:
                dfu_abort(dev)
                time.sleep(1.0)
                dev = find_dfu(retries=10, delay=0.5)
                if not dev:
                    return None
        else:
            dfu_abort(dev)
            time.sleep(0.8)
            dev = find_dfu(retries=10, delay=0.5)
            if not dev:
                return None
    return None

def wait_device_reappear(timeout_s=60):
    """Wait for device to reappear after crash/reset.
    SecureROM crash -> USB re-enumerate -> back to DFU. Can take up to ~30s.
    We wait up to 60s to avoid forcing manual DFU re-entry.
    """
    log(f"  Waiting for device to re-enumerate (max {timeout_s}s)...")
    t0 = time.time()
    attempt = 0
    while time.time() - t0 < timeout_s:
        attempt += 1
        dev = find_dfu(retries=1, delay=0.1)
        if dev:
            elapsed = time.time() - t0
            log(f"  Device recovered after {elapsed:.1f}s (attempt {attempt})")
            # Give it a moment to stabilize
            time.sleep(0.5)
            return dev
        time.sleep(0.2)
    log(f"  Device did NOT reappear after {timeout_s}s", "ERROR")
    return None

# ============================================================================
# Baseline measurement (NON-DESTRUCTIVE — safe for DNLOAD mode)
# ============================================================================
def measure_baseline(dev, n=5):
    """Measure baseline DNLOAD+ABORT round-trip WITHOUT triggering manifest.
    Sends a tiny valid payload, then ABORTs back to dfuIDLE.
    This exercises the same USB path as tests but never triggers the parser.
    """
    log("Measuring baseline (DNLOAD+ABORT, no manifest)...")
    
    # Phase 1: GET_STATUS RTT
    rtt_times = []
    for i in range(n):
        t0 = time.perf_counter()
        st = dfu_get_status(dev)
        dt = (time.perf_counter() - t0) * 1000
        if isinstance(st, dict):
            rtt_times.append(dt)
        time.sleep(0.02)
    
    rtt_avg = sum(rtt_times) / len(rtt_times) if rtt_times else 5.0
    log(f"  GET_STATUS RTT: avg={rtt_avg:.1f}ms")
    
    # Phase 2: DNLOAD+GET_STATUS+ABORT cycle (same as safe test, no manifest)
    cycle_times = []
    for i in range(n):
        dev = ensure_idle(dev)
        if not dev:
            log("  Lost device during baseline!", "ERROR")
            return None, 5000.0
        
        t0 = time.perf_counter()
        dfu_dnload(dev, b'\x30\x02\x02\x00')  # 4-byte DER
        st = dfu_get_status(dev)  # Advances to DNLOAD-SYNC/IDLE
        dfu_abort(dev)  # Back to dfuIDLE — NO manifest!
        dt = (time.perf_counter() - t0) * 1000
        
        state_desc = st['state_name'] if isinstance(st, dict) else str(st)
        cycle_times.append(dt)
        log(f"  Baseline {i+1}/{n}: {dt:.1f}ms (state after DNLOAD: {state_desc})")
        time.sleep(0.1)
    
    if not cycle_times:
        return dev, 5000.0
    
    avg = sum(cycle_times) / len(cycle_times)
    mx = max(cycle_times)
    # Threshold: 5x max cycle time, minimum 500ms (generous)
    threshold = max(mx * 5, 500.0)
    log(f"  Cycle: avg={avg:.1f}ms  max={mx:.1f}ms  threshold={threshold:.1f}ms")
    
    dev = ensure_idle(dev)
    return dev, threshold

# ============================================================================
# Test case runner — DNLOAD-only (safe) mode
# ============================================================================
def run_test_dnload(dev, tc, baseline_threshold):
    """Safe DNLOAD-only test. No manifest trigger, device stays in DFU.
    
    Flow: dfuIDLE -> DNLOAD(payload) -> GET_STATUS -> ABORT -> dfuIDLE
    
    What we can detect:
    - CRASH on DNLOAD (device disappears during USB transfer)
    - Unexpected state after DNLOAD (not dfuDNLOAD-SYNC/IDLE/ERROR)
    - Timing anomalies (DNLOAD takes much longer than baseline)
    - dfuERROR with unusual bStatus
    """
    tc_id = tc["id"]
    name = tc["name"]
    severity = tc["severity"]
    fpath = PAYLOAD_DIR / tc["file"]
    
    log(f"  TC {tc_id:3d} | {severity:8s} | {name} ({tc['size']}B)")
    
    if not fpath.exists():
        return {"id": tc_id, "name": name, "result": "SKIP", "reason": "file_not_found"}
    
    payload = fpath.read_bytes()
    
    dev = ensure_idle(dev)
    if not dev:
        dev = wait_device_reappear(60)
        if not dev:
            return {"id": tc_id, "name": name, "result": "DEVICE_LOST", "device": None}
        dev = ensure_idle(dev)
        if not dev:
            return {"id": tc_id, "name": name, "result": "DEVICE_STUCK", "device": None}
    
    t_start = time.perf_counter()
    
    # Send payload
    dn_result = dfu_dnload(dev, payload)
    t_dnload = (time.perf_counter() - t_start) * 1000
    
    if dn_result != True:
        # DNLOAD failed — check if device is still alive
        time.sleep(0.3)
        dev_check = find_dfu(retries=3, delay=0.5)
        if not dev_check:
            log(f"    *** CRASH ON DNLOAD *** {dn_result} t={t_dnload:.1f}ms", "CRIT")
            dev = wait_device_reappear(60)
            return {"id": tc_id, "name": name, "result": "CRASH_ON_DNLOAD",
                    "dnload_ms": round(t_dnload, 1), "error": str(dn_result),
                    "severity": severity, "device": dev}
        dev = dev_check
        log(f"    DNLOAD error but device alive: {dn_result}")
    
    # GET_STATUS to see what state the device transitioned to
    st = dfu_get_status(dev)
    t_status = (time.perf_counter() - t_start) * 1000
    
    if st is None:
        log(f"    *** DEVICE GONE after GET_STATUS *** t={t_status:.1f}ms", "CRIT")
        dev = wait_device_reappear(60)
        return {"id": tc_id, "name": name, "result": "CRASH_AFTER_STATUS",
                "total_ms": round(t_status, 1), "severity": severity, "device": dev}
    
    if isinstance(st, str):
        # Timeout or USB error
        log(f"    GET_STATUS: {st} t={t_status:.1f}ms", "WARN")
        dfu_abort(dev)
        dev = ensure_idle(dev)
        return {"id": tc_id, "name": name, "result": "STATUS_ERROR",
                "total_ms": round(t_status, 1), "status_error": st,
                "severity": severity, "device": dev}
    
    state = st["bState"]
    bstatus = st["bStatus"]
    state_name = st["state_name"]
    
    # ABORT back to dfuIDLE (safe — no manifest)
    dfu_abort(dev)
    time.sleep(0.05)
    
    # If we ended in dfuERROR, clear it
    if state == 10:
        dfu_clear_status(dev)
    
    total_ms = round(t_status, 1)
    
    # Classify
    result_data = {
        "id": tc_id, "name": name, "severity": severity,
        "total_ms": total_ms, "dnload_ms": round(t_dnload, 1),
        "bState": state, "state_name": state_name, "bStatus": bstatus,
    }
    
    if state == 10:  # dfuERROR — expected for malformed data
        result_data["result"] = "REJECTED"
    elif state in (3, 5):  # dfuDNLOAD-SYNC or dfuDNLOAD-IDLE — accepted data
        result_data["result"] = "ACCEPTED"  # Interesting! Data was buffered
    elif state == 2:  # dfuIDLE
        result_data["result"] = "IDLE"
    elif state in (6, 7, 8):  # dfuMANIFEST states
        result_data["result"] = "MANIFEST_TRIGGERED"  # Very interesting!
        log(f"    !!! MANIFEST STATE {state_name} without zero-length DNLOAD !!!", "CRIT")
    else:
        result_data["result"] = f"UNEXPECTED_STATE_{state}"
        log(f"    Unexpected state: {state_name}", "WARN")
    
    # Check timing anomaly
    if baseline_threshold and total_ms > baseline_threshold:
        result_data["timing"] = "SLOW"
        log(f"    SLOW: {total_ms:.1f}ms > threshold {baseline_threshold:.1f}ms", "WARN")
    
    log(f"    -> {result_data['result']:20s} bSt={bstatus} state={state_name} t={total_ms:.1f}ms")
    result_data["device"] = dev
    return result_data


# ============================================================================
# Test case runner — MANIFEST mode (destructive, for follow-up)
# ============================================================================
def run_test_manifest(dev, tc, baseline_threshold):
    """Full manifest test. Triggers IMG4 parser. Device will reset after.
    Use only for targeted follow-up on interesting DNLOAD results.
    """
    tc_id = tc["id"]
    name = tc["name"]
    severity = tc["severity"]
    fpath = PAYLOAD_DIR / tc["file"]
    
    log(f"\n{'='*70}")
    log(f"TC {tc_id:3d} | {severity:8s} | {name} [MANIFEST MODE]")
    log(f"  WARNING: Device will likely reset after this test!")
    
    if not fpath.exists():
        return {"id": tc_id, "name": name, "result": "SKIP"}
    
    payload = fpath.read_bytes()
    log(f"  Hex[0:32]: {payload[:32].hex()}")
    
    dev = ensure_idle(dev)
    if not dev:
        return {"id": tc_id, "name": name, "result": "DEVICE_LOST", "device": None}
    
    t_start = time.perf_counter()
    
    # Send payload
    dn_result = dfu_dnload(dev, payload)
    if dn_result != True:
        time.sleep(0.5)
        dev_check = find_dfu(retries=3, delay=0.5)
        if not dev_check:
            log(f"  *** CRASH ON DNLOAD ***", "CRIT")
            dev = wait_device_reappear(60)
            return {"id": tc_id, "name": name, "result": "CRASH_ON_DNLOAD", "device": dev}
        dev = dev_check
    
    # GET_STATUS to advance state
    st = dfu_get_status(dev)
    if isinstance(st, dict) and st["bState"] in (3, 5):
        # Trigger manifest with zero-length DNLOAD
        log(f"  Triggering manifest...")
        dfu_dnload(dev, b'')
    elif isinstance(st, dict) and st["bState"] == 10:
        log(f"  dfuERROR before manifest — parser rejected during DNLOAD")
        dfu_clear_status(dev)
        return {"id": tc_id, "name": name, "result": "REJECTED_EARLY",
                "total_ms": round((time.perf_counter() - t_start) * 1000, 1),
                "severity": severity, "device": dev}
    
    # Poll for result
    t_trigger = time.perf_counter()
    states_seen = []
    final_state = None
    device_lost = False
    
    for poll in range(100):  # Up to 10s
        time.sleep(0.1)
        st = dfu_get_status(dev)
        if st is None:
            device_lost = True
            break
        elif isinstance(st, str):
            states_seen.append(st)
            continue
        else:
            states_seen.append(st["state_name"])
            if st["bState"] in (2, 10):
                final_state = st
                break
            if st["bState"] == 8:
                final_state = st
                log(f"  !!! dfuMANIFEST-WAIT-RESET !!! PAYLOAD ACCEPTED!", "CRIT")
                break
    
    t_end = time.perf_counter()
    total_ms = round((t_end - t_start) * 1000, 1)
    
    result_data = {
        "id": tc_id, "name": name, "severity": severity,
        "total_ms": total_ms, "mode": "manifest",
        "states_seen": list(dict.fromkeys(states_seen)),
    }
    
    if device_lost:
        log(f"  Device lost after manifest (expected — goes to Recovery)")
        dev = wait_device_reappear(60)
        result_data["result"] = "DEVICE_RESET" if dev else "DEVICE_LOST"
        result_data["device"] = dev
        return result_data
    
    if final_state:
        result_data["bState"] = final_state["bState"]
        result_data["bStatus"] = final_state["bStatus"]
        if final_state["bState"] == 10:
            result_data["result"] = "REJECTED"
            dfu_clear_status(dev)
        elif final_state["bState"] == 8:
            result_data["result"] = "MANIFEST_ACCEPTED"
        else:
            result_data["result"] = "OK"
    else:
        result_data["result"] = "TIMEOUT"
    
    log(f"  Result: {result_data['result']} t={total_ms}ms")
    result_data["device"] = dev
    return result_data

# ============================================================================
# Main
# ============================================================================
def main():
    global log_fh
    
    parser = argparse.ArgumentParser(description="T8020 DFU Payload Tester")
    parser.add_argument("--all", action="store_true", help="Run all test cases")
    parser.add_argument("--severity", default=None, help="Min severity (CRITICAL/HIGH/MEDIUM/LOW)")
    parser.add_argument("--id", nargs="+", type=int, help="Specific test case IDs")
    parser.add_argument("--category", type=int, help="Category number (1-11)")
    parser.add_argument("--dry", action="store_true", help="Dry run")
    parser.add_argument("--no-baseline", action="store_true", help="Skip baseline measurement")
    parser.add_argument("--pause", type=float, default=0.3, help="Pause between tests (sec)")
    parser.add_argument("--no-sort", action="store_true", help="Don't sort, run in manifest order")
    parser.add_argument("--mode", default="dnload", choices=["dnload", "manifest"],
                        help="dnload=safe (no reset), manifest=full parser test (resets device)")
    args = parser.parse_args()
    
    # Default: --all for dnload mode, --severity CRITICAL for manifest mode
    if not args.all and not args.id and not args.category and args.severity is None:
        if args.mode == "dnload":
            args.all = True  # Safe mode: run everything
        else:
            args.severity = "CRITICAL"  # Manifest: only critical
    
    # Open log
    log_fh = open(LOG_FILE, "w", encoding="utf-8")
    
    log("=" * 70)
    log(f"T8020 B1 SecureROM -- DFU Payload Tester [{args.mode.upper()} mode]")
    log("=" * 70)
    if args.mode == "dnload":
        log("SAFE MODE: DNLOAD+ABORT only, device stays in DFU")
    else:
        log("MANIFEST MODE: Full parser test, device will reset after each test!")
    
    # Load manifest
    manifest_path = PAYLOAD_DIR / "test_manifest.json"
    if not manifest_path.exists():
        log("ERROR: test_manifest.json not found. Run img4_test_generator.py first.", "ERROR")
        return
    
    manifest = json.loads(manifest_path.read_text())
    all_cases = manifest["cases"]
    log(f"Loaded {len(all_cases)} test cases from manifest")
    
    # Filter cases
    sev_order = {"CRITICAL": 0, "HIGH": 1, "MEDIUM": 2, "LOW": 3}
    
    if args.id:
        cases = [tc for tc in all_cases if tc["id"] in args.id]
    elif args.category:
        cat_prefixes = {
            1: "der_len", 2: "asn1_nesting", 3: "img4_tag|im4p_payload|img4_context",
            4: "truncat|der_tag_only|der_tag_length", 5: "im4p_",
            6: "manifest_|full_2048|dfu_size", 7: "octet_string",
            8: "der_longform|der_len_sign|der_len_0x80", 9: "img4_memz|img4_deep|raw_magic|img4_full_manifest",
            10: "img4_child|img4_double", 11: "payload_|sram"
        }
        import re
        prefix = cat_prefixes.get(args.category, "")
        cases = [tc for tc in all_cases if re.search(prefix, tc["name"])]
    elif args.all:
        cases = all_cases
    elif args.severity:
        min_sev = sev_order.get(args.severity, 0)
        cases = [tc for tc in all_cases if sev_order.get(tc["severity"], 99) <= min_sev]
    else:
        cases = all_cases  # Fallback: run all
    
    log(f"Selected {len(cases)} test cases to run")
    if not cases:
        log("No test cases match filter!", "WARN")
        return
    
    # Sort by risk: smallest payloads first (less likely to crash)
    if not args.no_sort:
        cases.sort(key=lambda tc: tc["size"])
        log(f"Sorted by size (safest first): {cases[0]['size']}B -> {cases[-1]['size']}B")
    
    # Connect
    if not args.dry:
        dev = find_dfu(retries=5, delay=0.5)
        if not dev:
            log("ERROR: No DFU device found!", "ERROR")
            return
        
        try:
            serial = dev.serial_number
        except:
            serial = "?"
        log(f"Device: {serial}")
        
        st = dfu_get_status(dev)
        if isinstance(st, dict):
            log(f"State: {st['state_name']} (bStatus={st['bStatus']})")
        
        dev = ensure_idle(dev)
        if not dev:
            log("ERROR: Cannot reach dfuIDLE!", "ERROR")
            return
        
        # Baseline
        baseline_threshold = None
        if not args.no_baseline:
            dev, baseline_threshold = measure_baseline(dev)
            if not dev:
                log("ERROR: Lost device during baseline!", "ERROR")
                return
    else:
        dev = None
        baseline_threshold = None
    
    # Run tests
    results = []
    crashes = []
    interesting = []
    consecutive_crashes = 0
    MAX_CONSECUTIVE_CRASHES = 3
    
    test_fn = run_test_dnload if args.mode == "dnload" else run_test_manifest
    
    for i, tc in enumerate(cases):
        if args.mode == "dnload":
            log(f"\n[{i+1}/{len(cases)}]", "INFO")
        else:
            log(f"\n--- Manifest Test {i+1}/{len(cases)} ---")
        
        if args.dry:
            log(f"  TC {tc['id']:3d} | {tc['severity']:8s} | {tc['name']} [DRY RUN]")
            results.append({"id": tc["id"], "name": tc["name"], "result": "DRY_RUN"})
            continue
        
        res = test_fn(dev, tc, baseline_threshold)
        
        # Extract device handle (may have changed after recovery)
        if "device" in res:
            if res["device"] is not None:
                dev = res["device"]
            del res["device"]
        
        results.append(res)
        
        # Track crashes
        if "CRASH" in res.get("result", ""):
            crashes.append(res)
            consecutive_crashes += 1
            log(f"    !!! CRASH #{len(crashes)} (consec: {consecutive_crashes}) !!!", "CRIT")
            if dev is None:
                log(f"    STOPPING: Device permanently lost", "ERROR")
                break
            if consecutive_crashes >= MAX_CONSECUTIVE_CRASHES:
                log(f"    STOPPING: {MAX_CONSECUTIVE_CRASHES} consecutive crashes", "ERROR")
                break
            time.sleep(2.0)
        else:
            consecutive_crashes = 0
        
        # Track interesting results
        is_interesting = res.get("result", "") in (
            "ACCEPTED", "MANIFEST_TRIGGERED", "CRASH_ON_DNLOAD", "CRASH_AFTER_STATUS",
            "MANIFEST_ACCEPTED", "DEVICE_RESET"
        ) or res.get("timing") == "SLOW"
        if is_interesting:
            interesting.append(res)
        
        # Incremental save
        _save_incremental(results, crashes, interesting, baseline_threshold, args.mode)
        
        if not args.dry:
            time.sleep(args.pause)
    
    # Summary
    log(f"\n\n{'='*70}")
    log(f"RESULTS SUMMARY")
    log(f"{'='*70}")
    
    from collections import Counter
    result_counts = Counter(r["result"] for r in results)
    for result_type, count in result_counts.most_common():
        log(f"  {result_type:25s}: {count}")
    
    if crashes:
        log(f"\n  *** {len(crashes)} CRASHES DETECTED ***", "CRIT")
        for c in crashes:
            log(f"    TC {c['id']:3d} ({c['name']}): {c['result']} at {c.get('time_ms','?')}ms")
    
    if interesting:
        log(f"\n  {len(interesting)} INTERESTING RESULTS:", "WARN")
        for r in interesting:
            log(f"    TC {r['id']:3d} ({r.get('name','?')}): {r['result']} @ {r.get('total_ms','?')}ms")
    
    # Timing analysis
    timed = [r for r in results if "total_ms" in r]
    if timed:
        times = [r["total_ms"] for r in timed]
        avg = sum(times) / len(times)
        log(f"\n  Timing: avg={avg:.1f}ms  min={min(times):.1f}ms  max={max(times):.1f}ms")
        
        # Find outliers (>2x average)
        outliers = [r for r in timed if r["total_ms"] > avg * 2]
        if outliers:
            log(f"  Timing outliers (>2x avg):")
            for o in sorted(outliers, key=lambda x: -x["total_ms"]):
                log(f"    TC {o['id']:3d}: {o['total_ms']:.1f}ms ({o.get('name','')})")
    
    # Final save
    _save_incremental(results, crashes, interesting, baseline_threshold, args.mode)
    log(f"\nResults saved to: {RESULTS_FILE}")
    log(f"Log saved to: {LOG_FILE}")
    
    if log_fh:
        log_fh.close()


def _save_incremental(results, crashes, interesting, baseline_threshold, mode="dnload"):
    """Save results after each test so data isn't lost on crash."""
    from collections import Counter
    result_counts = Counter(r.get("result", "?") for r in results)
    output = {
        "date": datetime.now().isoformat(),
        "device": "T8020 B1 (CPID:8020 CPRV:11)",
        "mode": mode,
        "total_tests": len(results),
        "summary": dict(result_counts),
        "crashes": crashes,
        "interesting": interesting,
        "baseline_threshold_ms": baseline_threshold,
        "results": results
    }
    RESULTS_FILE.write_text(json.dumps(output, indent=2, default=str))

if __name__ == "__main__":
    main()
