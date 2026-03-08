#!/usr/bin/env python3
"""
A12 SecureROM — Precision Timing Oracle v2
=============================================
Focused on clean measurements with proper DFU state management.

Approach: 
  - Measure DNLOAD time only (ctrl_transfer round-trip)
  - Then GET_STATUS → state should be 5 (dfuDNLOAD-IDLE) for data phase
  - Then ABORT → back to idle
  - No manifest trigger (avoid the 3.5s wait)

Key insight: DNLOAD ctrl_transfer returns when device ACKs the data.
The time includes: USB transfer + device copy-to-buffer time.
"""
import sys, time, struct, json, statistics
from datetime import datetime
from pathlib import Path

import usb.core, usb.util, libusb_package, usb.backend.libusb1

APPLE_VID = 0x05AC; DFU_PID = 0x1227

def be():
    return usb.backend.libusb1.get_backend(find_library=libusb_package.find_library)

def log(msg):
    ts = datetime.now().strftime("%H:%M:%S.%f")[:-3]
    print(f"[{ts}] {msg}", flush=True)

def ns():
    return time.perf_counter_ns()

class TimingV2:
    def __init__(self):
        self.dev = None

    def connect(self):
        self.dev = usb.core.find(idVendor=APPLE_VID, idProduct=DFU_PID, backend=be())
        if self.dev:
            try: self.dev.set_configuration()
            except: pass
            return True
        return False

    def status(self):
        try:
            r = self.dev.ctrl_transfer(0xA1, 3, 0, 0, 6, timeout=5000)
            return (r[4], r[0], r[1]|(r[2]<<8)|(r[3]<<16)) if len(r)>=6 else None
        except: return None

    def to_idle(self):
        """Reliable return to DFU idle (state 2)"""
        for attempt in range(30):
            s = self.status()
            if not s:
                time.sleep(0.1)
                self.connect()
                continue
            state, bstatus, poll = s
            if state == 2:
                return True
            elif state == 10:  # dfuERROR
                self.dev.ctrl_transfer(0x21, 4, 0, 0, 0, timeout=2000)  # CLRSTATUS
            elif state == 5:   # dfuDNLOAD-IDLE
                self.dev.ctrl_transfer(0x21, 6, 0, 0, 0, timeout=2000)  # ABORT
            elif state == 3:   # dfuDNLOAD-SYNC
                # Need to read status to advance state machine
                time.sleep(0.05)
            elif state == 4:   # dfuDNBUSY
                time.sleep(poll / 1000.0 + 0.01)
            elif state == 8:   # dfuMANIFEST-WAIT-RESET
                try: self.dev.reset()
                except: pass
                time.sleep(2)
                self.connect()
            else:
                try: self.dev.ctrl_transfer(0x21, 6, 0, 0, 0, timeout=2000)
                except: pass
            time.sleep(0.02)
        return False

    def run(self):
        log("="*60)
        log("A12 SecureROM — Precision Timing Oracle v2")
        log("="*60)

        if not self.connect():
            log("NO DFU DEVICE"); return

        results = {"timestamp": datetime.now().isoformat()}

        # ============================================================
        # 1. USB baseline: GET_DESCRIPTOR round-trip
        # ============================================================
        log(f"\n--- USB Baseline: GET_DESCRIPTOR x200 ---")
        times = []
        for _ in range(200):
            t0 = ns()
            try:
                self.dev.ctrl_transfer(0x80, 6, 0x0100, 0, 18, timeout=1000)
                times.append((ns()-t0)/1000)
            except: pass
        if times:
            times.sort()
            # Remove outliers (top/bottom 5%)
            trimmed = times[len(times)//20 : -len(times)//20] if len(times) > 40 else times
            results["usb_baseline_us"] = {
                "mean": round(statistics.mean(trimmed), 1),
                "median": round(statistics.median(trimmed), 1),
                "stdev": round(statistics.stdev(trimmed), 1) if len(trimmed) > 1 else 0,
                "p5": round(trimmed[0], 1),
                "p95": round(trimmed[-1], 1),
                "n": len(trimmed),
            }
            log(f"  Median: {results['usb_baseline_us']['median']} us  "
                f"Mean: {results['usb_baseline_us']['mean']} us  "
                f"Stdev: {results['usb_baseline_us']['stdev']} us")

        # ============================================================
        # 2. GET_STATUS round-trip
        # ============================================================
        log(f"\n--- GET_STATUS RTT x200 ---")
        self.to_idle()
        times = []
        for _ in range(200):
            t0 = ns()
            s = self.status()
            times.append((ns()-t0)/1000)
        if times:
            times.sort()
            trimmed = times[len(times)//20 : -len(times)//20] if len(times) > 40 else times
            results["get_status_us"] = {
                "mean": round(statistics.mean(trimmed), 1),
                "median": round(statistics.median(trimmed), 1),
                "stdev": round(statistics.stdev(trimmed), 1) if len(trimmed) > 1 else 0,
                "p5": round(trimmed[0], 1),
                "p95": round(trimmed[-1], 1),
                "n": len(trimmed),
            }
            log(f"  Median: {results['get_status_us']['median']} us  "
                f"Mean: {results['get_status_us']['mean']} us  "
                f"Stdev: {results['get_status_us']['stdev']} us")

        # ============================================================
        # 3. GET_STATE round-trip (1 byte response vs 6)
        # ============================================================
        log(f"\n--- GET_STATE RTT x200 ---")
        times = []
        for _ in range(200):
            t0 = ns()
            try:
                self.dev.ctrl_transfer(0xA1, 5, 0, 0, 1, timeout=1000)
                times.append((ns()-t0)/1000)
            except: pass
        if times:
            times.sort()
            trimmed = times[len(times)//20 : -len(times)//20] if len(times) > 40 else times
            results["get_state_us"] = {
                "mean": round(statistics.mean(trimmed), 1),
                "median": round(statistics.median(trimmed), 1),
                "stdev": round(statistics.stdev(trimmed), 1) if len(trimmed) > 1 else 0,
                "n": len(trimmed),
            }
            log(f"  Median: {results['get_state_us']['median']} us")

        # ============================================================
        # 4. DNLOAD round-trip for various sizes
        # ============================================================
        log(f"\n--- DNLOAD transfer time vs payload size ---")
        dnload_results = []

        for size in [16, 32, 48, 64, 96, 128, 192, 256, 384, 512, 768, 1024, 1536, 2048, 3072, 4096]:
            times = []
            payload = bytes([0x42]) * size
            
            for rep in range(10):
                if not self.to_idle():
                    log(f"  {size}B: idle failed at rep {rep}")
                    break

                t0 = ns()
                try:
                    self.dev.ctrl_transfer(0x21, 1, 0, 0, payload, timeout=5000)
                    t1 = ns()
                    times.append((t1-t0)/1000)
                except usb.core.USBError as e:
                    log(f"  {size}B rep{rep}: {e}")
                    break

                # Proper cleanup: GET_STATUS first
                s = self.status()
                if s:
                    state = s[0]
                    if state in (3, 5):
                        try: self.dev.ctrl_transfer(0x21, 6, 0, 0, 0, timeout=2000)
                        except: pass
                    elif state == 4:
                        time.sleep(s[2]/1000.0 + 0.01)
                        self.status()
                        try: self.dev.ctrl_transfer(0x21, 6, 0, 0, 0, timeout=2000)
                        except: pass

            if times:
                times.sort()
                entry = {
                    "size": size,
                    "mean_us": round(statistics.mean(times), 1),
                    "median_us": round(statistics.median(times), 1),
                    "stdev_us": round(statistics.stdev(times), 1) if len(times) > 1 else 0,
                    "min_us": round(min(times), 1),
                    "max_us": round(max(times), 1),
                    "n": len(times),
                }
                dnload_results.append(entry)
                log(f"  {size:5d}B: median={entry['median_us']:8.1f}us "
                    f"mean={entry['mean_us']:8.1f}us stdev={entry['stdev_us']:6.1f}us "
                    f"n={entry['n']}")

        results["dnload_times"] = dnload_results

        # Linear regression
        if len(dnload_results) >= 3:
            xs = [r["size"] for r in dnload_results]
            ys = [r["median_us"] for r in dnload_results]
            n = len(xs)
            sx=sum(xs); sy=sum(ys); sxx=sum(x*x for x in xs); sxy=sum(x*y for x,y in zip(xs,ys))
            denom = n*sxx - sx*sx
            if denom:
                slope = (n*sxy - sx*sy) / denom
                intercept = (sy - slope*sx) / n
                results["dnload_regression"] = {
                    "slope_us_per_byte": round(slope, 4),
                    "intercept_us": round(intercept, 1),
                    "throughput_MBps": round(1/slope, 2) if slope > 0 else None,
                }
                log(f"\n  Regression: time = {slope:.4f} * size + {intercept:.1f} us")
                log(f"  Throughput: {1/slope:.2f} MB/s" if slope > 0 else "  Throughput: N/A")

        # ============================================================
        # 5. poll_ms for different payloads
        # ============================================================
        log(f"\n--- poll_ms oracle ---")
        poll_results = []

        test_payloads = [
            ("zeros_16",   b"\x00" * 16),
            ("zeros_256",  b"\x00" * 256),
            ("zeros_512",  b"\x00" * 512),
            ("zeros_2048", b"\x00" * 2048),
            ("img4_256",   b"\x30\x82\x10\x00\x16\x04IMG4" + b"\x00" * 246),
            ("img4_2048",  b"\x30\x82\x10\x00\x16\x04IMG4" + b"\x00" * 2038),
            ("im4p_256",   b"\x30\x82\x10\x00\x16\x04IM4P" + b"\x00" * 246),
            ("0xff_256",   b"\xFF" * 256),
            ("0x41_256",   b"\x41" * 256),
        ]

        for name, payload in test_payloads:
            for rep in range(3):
                if not self.to_idle():
                    break
                try:
                    self.dev.ctrl_transfer(0x21, 1, 0, 0, payload, timeout=5000)
                except:
                    continue
                s = self.status()
                if s:
                    poll_results.append({
                        "name": name, "size": len(payload),
                        "state_after": s[0], "poll_ms": s[2], "bstatus": s[1]
                    })
                # Cleanup
                s2 = self.status()
                if s2 and s2[0] in (3, 5):
                    try: self.dev.ctrl_transfer(0x21, 6, 0, 0, 0, timeout=2000)
                    except: pass
                elif s2 and s2[0] == 4:
                    time.sleep(0.1)
                    self.status()
                    try: self.dev.ctrl_transfer(0x21, 6, 0, 0, 0, timeout=2000)
                    except: pass

        results["poll_oracle"] = poll_results
        if poll_results:
            for p in poll_results:
                log(f"  {p['name']:15s}: state={p['state_after']} poll={p['poll_ms']}ms status={p['bstatus']}")

        # ============================================================
        # 6. Manifest timing: DNLOAD + trigger via empty DNLOAD (block 0)
        # ============================================================
        log(f"\n--- Manifest timing for various payloads ---")
        manifest_results = []

        manifest_payloads = [
            ("zeros_64",    b"\x00" * 64),
            ("zeros_256",   b"\x00" * 256),
            ("zeros_512",   b"\x00" * 512),
            ("zeros_2048",  b"\x00" * 2048),
            ("img4_512",    b"\x30\x82\x10\x00\x16\x04IMG4" + b"\x00" * 502),
            ("im4p_512",    b"\x30\x82\x10\x00\x16\x04IM4P" + b"\x00" * 502),
            ("ibss_512",    b"\x30\x82\x10\x00\x16\x04IMG4\x16\x04ibss" + b"\x00" * 492),
        ]

        for name, payload in manifest_payloads:
            times_ms = []
            for rep in range(3):
                if not self.to_idle():
                    break

                # DNLOAD
                try:
                    self.dev.ctrl_transfer(0x21, 1, 0, 0, payload, timeout=5000)
                except:
                    continue

                # GET_STATUS → starts processing
                t0 = ns()
                s = self.status()
                if not s:
                    continue

                first_state = s[0]
                first_poll = s[2]

                # If state 5 (dfuDNLOAD-IDLE), send zero-length DNLOAD to trigger manifest
                if first_state == 5:
                    try:
                        self.dev.ctrl_transfer(0x21, 1, 0, 0, b"", timeout=5000)
                    except: continue
                    s = self.status()
                    if not s: continue
                    first_state = s[0]
                    first_poll = s[2]

                # Now wait for manifest completion
                if first_state == 4:  # dfuDNBUSY
                    time.sleep(first_poll / 1000.0)
                    s = self.status()

                # Wait for final state
                for _ in range(100):
                    if not s or s[0] not in (3, 4):
                        break
                    if s[0] == 4:
                        time.sleep(s[2] / 1000.0)
                    s = self.status()

                t_end = ns()
                total_ms = (t_end - t0) / 1e6
                final_state = s[0] if s else None
                times_ms.append(total_ms)

                manifest_results.append({
                    "name": name, "rep": rep,
                    "total_ms": round(total_ms, 3),
                    "first_poll_ms": first_poll,
                    "final_state": final_state,
                })
                log(f"  {name:15s} rep{rep}: {total_ms:.3f}ms → state={final_state} poll={first_poll}ms")

                # Reset
                self.to_idle()

        results["manifest_timing"] = manifest_results

        # ============================================================
        # ANALYSIS
        # ============================================================
        log(f"\n{'='*60}")
        log(f"ANALYSIS")
        log(f"{'='*60}")

        usb_base = results.get("usb_baseline_us", {}).get("median", 0)
        gs_rtt = results.get("get_status_us", {}).get("median", 0)
        log(f"  USB round-trip baseline: {usb_base:.1f} us")
        log(f"  GET_STATUS RTT:          {gs_rtt:.1f} us")
        log(f"  GET_STATUS processing:   {gs_rtt - usb_base:.1f} us")

        if "dnload_regression" in results:
            reg = results["dnload_regression"]
            log(f"  DNLOAD: {reg['slope_us_per_byte']:.4f} us/byte = {reg.get('throughput_MBps', 'N/A')} MB/s")
            log(f"  DNLOAD overhead: {reg['intercept_us']:.1f} us")

            # Clock estimation from known operations
            # If GET_STATUS processing = 451us and that's ~1000 instructions at 24MHz:
            # 1000 instructions * (1/24MHz) = 41.67us → too fast
            # Maybe ~10000 instructions at 24MHz = 416us → close to 451us!
            if gs_rtt > usb_base and usb_base > 0:
                proc_us = gs_rtt - usb_base
                for freq_mhz in [24, 48, 96, 192, 384, 768, 1590, 2490]:
                    cycles = proc_us * freq_mhz
                    log(f"    At {freq_mhz:5d} MHz: GET_STATUS = {cycles:,.0f} cycles")

        # Save
        outf = Path(__file__).parent / "results" / "timing_oracle_v2.json"
        outf.parent.mkdir(exist_ok=True)
        with open(outf, "w") as f:
            json.dump(results, f, indent=2)
        log(f"\nSaved: {outf}")

if __name__ == "__main__":
    t = TimingV2()
    t.run()
