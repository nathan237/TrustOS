#!/usr/bin/env python3
"""
A12 SecureROM — Precision Timing Oracle
==========================================
Goal: Build a high-resolution timing map of every DFU operation
to deduce CPU clock frequency and identify internal code paths.

Known timing signatures:
  - 73ms:   crash on <16B payload (hard fault / exception handler)
  - ~50ms:  GET_STATUS poll_ms (SecureROM's own timer tick)
  - 3562ms: manifest processing timeout
  - ~516ms: I/O error recovery between rapid cycles

A12 Bionic (T8020) specs:
  - Vortex (perf) cores: 2.49 GHz
  - Tempest (efficiency) cores: 1.59 GHz
  - SecureROM likely runs at fixed low frequency from crystal osc
  - Typical ARM boot clock: 24MHz (crystal) or PLL-derived

Strategy:
  1. Measure DNLOAD transfer time vs payload size → USB throughput → clock
  2. Measure GET_STATUS response precision → timer resolution
  3. Measure DNLOAD→manifest timing vs payload size → processing time per byte
  4. Measure control transfer round-trip → USB frame timing
  5. Repeated measurements for statistical significance
"""
import sys, time, struct, json, argparse, statistics
from datetime import datetime
from pathlib import Path

import usb.core, usb.util, libusb_package, usb.backend.libusb1

APPLE_VID = 0x05AC; DFU_PID = 0x1227

def backend():
    return usb.backend.libusb1.get_backend(find_library=libusb_package.find_library)

def log(msg):
    ts = datetime.now().strftime("%H:%M:%S.%f")[:-3]
    print(f"[{ts}] {msg}", flush=True)

class TimingOracle:
    def __init__(self):
        self.dev = None
        self.results = {}

    def connect(self):
        self.dev = usb.core.find(idVendor=APPLE_VID, idProduct=DFU_PID, backend=backend())
        if self.dev:
            try: self.dev.set_configuration()
            except: pass
            return True
        return False

    def get_status(self):
        try:
            r = self.dev.ctrl_transfer(0xA1, 3, 0, 0, 6, timeout=5000)
            return {"bStatus": r[0], "bState": r[4],
                    "poll_ms": r[1] | (r[2] << 8) | (r[3] << 16)} if len(r) >= 6 else None
        except:
            return None

    def reset_idle(self):
        for _ in range(20):
            st = self.get_status()
            if not st: return False
            if st["bState"] == 2: return True
            if st["bState"] == 10:
                self.dev.ctrl_transfer(0x21, 4, 0, 0, 0, timeout=2000)
            elif st["bState"] == 8:
                # manifestWaitReset — need device reset
                try: self.dev.ctrl_transfer(0x21, 6, 0, 0, 0, timeout=2000)
                except: pass
                try: self.dev.reset()
                except: pass
                time.sleep(2)
                try: usb.util.dispose_resources(self.dev)
                except: pass
                self.connect()
            else:
                try: self.dev.ctrl_transfer(0x21, 6, 0, 0, 0, timeout=2000)
                except: pass
            time.sleep(0.05)
        return False

    def precise_time(self):
        """High-resolution timer"""
        return time.perf_counter_ns()

    # ================================================================
    # TEST 1: GET_STATUS round-trip timing (measures USB latency baseline)
    # ================================================================
    def test_get_status_rtt(self, count=100):
        """
        Measure GET_STATUS round-trip time repeatedly.
        This gives us the USB control transfer baseline latency.
        Any deviation from this in other operations = processing time.
        """
        log(f"\n{'='*60}")
        log(f"TEST 1: GET_STATUS round-trip time ({count} samples)")
        log(f"{'='*60}")

        if not self.reset_idle():
            return {"error": "no_idle"}

        times_ns = []
        poll_ms_values = []

        for i in range(count):
            t0 = self.precise_time()
            st = self.get_status()
            t1 = self.precise_time()

            if st:
                dt_us = (t1 - t0) / 1000
                times_ns.append(t1 - t0)
                poll_ms_values.append(st["poll_ms"])
            else:
                log(f"  Sample {i}: FAILED")

        if not times_ns:
            return {"error": "no_data"}

        times_us = [t / 1000 for t in times_ns]
        result = {
            "test": "get_status_rtt",
            "samples": len(times_us),
            "mean_us": statistics.mean(times_us),
            "median_us": statistics.median(times_us),
            "stdev_us": statistics.stdev(times_us) if len(times_us) > 1 else 0,
            "min_us": min(times_us),
            "max_us": max(times_us),
            "poll_ms_values": list(set(poll_ms_values)),
            "all_times_us": [round(t, 1) for t in times_us],
        }

        log(f"  Mean:   {result['mean_us']:.1f} us")
        log(f"  Median: {result['median_us']:.1f} us")
        log(f"  Stdev:  {result['stdev_us']:.1f} us")
        log(f"  Range:  [{result['min_us']:.1f}, {result['max_us']:.1f}] us")
        log(f"  poll_ms values reported: {result['poll_ms_values']}")

        return result

    # ================================================================
    # TEST 2: DNLOAD transfer time vs payload size
    # ================================================================
    def test_dnload_transfer_time(self, sizes=None, repeats=10):
        """
        Measure DNLOAD time for various payload sizes.
        Time = USB transfer overhead + device processing.
        Linear regression → throughput → clock estimate.
        """
        if sizes is None:
            sizes = [16, 32, 64, 128, 256, 512, 1024, 2048, 4096]

        log(f"\n{'='*60}")
        log(f"TEST 2: DNLOAD transfer time vs size ({repeats} repeats)")
        log(f"{'='*60}")

        results = []

        for size in sizes:
            if not self.reset_idle():
                log(f"  {size}B: can't reset idle")
                continue

            times_us = []
            payload = bytes([0x41]) * size

            for r in range(repeats):
                if not self.reset_idle():
                    # Try harder
                    time.sleep(0.2)
                    self.connect()
                    if not self.reset_idle():
                        log(f"  {size}B rep{r}: can't idle, skipping")
                        continue

                t0 = self.precise_time()
                try:
                    self.dev.ctrl_transfer(0x21, 1, 0, 0, payload, timeout=5000)
                    t1 = self.precise_time()
                    dt_us = (t1 - t0) / 1000
                    times_us.append(dt_us)
                except usb.core.USBError as e:
                    log(f"  {size}B rep{r}: error {e}")

                # Proper reset: GET_STATUS first (moves to dnload-sync/idle)
                try:
                    st = self.get_status()
                    if st and st["bState"] == 5:
                        # dfuDNLOAD-IDLE → ABORT
                        self.dev.ctrl_transfer(0x21, 6, 0, 0, 0, timeout=2000)
                    elif st and st["bState"] == 3:
                        # dfuDNLOAD-SYNC → ABORT
                        self.dev.ctrl_transfer(0x21, 6, 0, 0, 0, timeout=2000)
                    elif st and st["bState"] == 4:
                        # dfuDNBUSY → wait then abort
                        time.sleep(0.1)
                        self.get_status()
                        self.dev.ctrl_transfer(0x21, 6, 0, 0, 0, timeout=2000)
                except: pass
                # Clear error state if needed
                try:
                    st = self.get_status()
                    if st and st["bState"] == 10:
                        self.dev.ctrl_transfer(0x21, 4, 0, 0, 0, timeout=2000)
                except: pass

            if times_us:
                entry = {
                    "size": size,
                    "mean_us": statistics.mean(times_us),
                    "median_us": statistics.median(times_us),
                    "stdev_us": statistics.stdev(times_us) if len(times_us) > 1 else 0,
                    "min_us": min(times_us),
                    "max_us": max(times_us),
                    "samples": len(times_us),
                    "all_us": [round(t, 1) for t in times_us],
                }
                results.append(entry)
                log(f"  {size:5d}B: mean={entry['mean_us']:.1f}us median={entry['median_us']:.1f}us "
                    f"stdev={entry['stdev_us']:.1f}us [{entry['min_us']:.1f}-{entry['max_us']:.1f}]")

        # Linear regression: time = a * size + b
        if len(results) >= 3:
            xs = [r["size"] for r in results]
            ys = [r["median_us"] for r in results]
            n = len(xs)
            sx = sum(xs); sy = sum(ys)
            sxx = sum(x*x for x in xs); sxy = sum(x*y for x, y in zip(xs, ys))
            denom = n * sxx - sx * sx
            if denom != 0:
                slope = (n * sxy - sx * sy) / denom  # us per byte
                intercept = (sy - slope * sx) / n     # base overhead us
                throughput_MBs = 1.0 / slope if slope > 0 else 0  # MB/s
                log(f"\n  Linear fit: time = {slope:.3f} * size + {intercept:.1f} us")
                log(f"  Throughput: {throughput_MBs:.2f} MB/s")
                log(f"  Base overhead: {intercept:.1f} us")
                if slope > 0:
                    # USB Full Speed = 12 Mbps = 1.5 MB/s
                    # USB High Speed = 480 Mbps = 60 MB/s
                    log(f"  (USB FS ~1.5 MB/s, USB HS ~60 MB/s)")
            else:
                slope = intercept = throughput_MBs = None
        else:
            slope = intercept = throughput_MBs = None

        return {
            "test": "dnload_transfer_time",
            "sizes": results,
            "slope_us_per_byte": slope,
            "intercept_us": intercept,
            "throughput_MBs": throughput_MBs,
        }

    # ================================================================
    # TEST 3: Manifest processing time vs payload content
    # ================================================================
    def test_manifest_timing(self, repeats=5):
        """
        Measure exact time from DNLOAD → GET_STATUS(BUSY) → GET_STATUS(complete).
        This measures how long SecureROM takes to process/validate the payload.
        Vary content to see if it causes different code paths.
        """
        log(f"\n{'='*60}")
        log(f"TEST 3: Manifest processing timing ({repeats} repeats)")
        log(f"{'='*60}")

        payloads = {
            "zeros_512":     bytes(512),
            "0x41_512":      b"\x41" * 512,
            "img4_header":   b"\x30\x82\x10\x00\x16\x04IMG4" + bytes(502),
            "im4p_header":   b"\x30\x82\x10\x00\x16\x04IM4P" + bytes(502),
            "im4m_header":   b"\x30\x82\x10\x00\x16\x04IM4M" + bytes(502),
            "ibss_magic":    b"\x30\x82\x10\x00\x16\x04IMG4\x16\x04ibss" + bytes(492),
            "ibot_magic":    b"\x30\x82\x10\x00\x16\x04IMG4\x16\x04ibot" + bytes(492),
            "illb_magic":    b"\x30\x82\x10\x00\x16\x04IMG4\x16\x04illb" + bytes(492),
            "random_512":    bytes(range(256)) * 2,
        }

        results = []

        for name, payload in payloads.items():
            timings = []

            for r in range(repeats):
                if not self.reset_idle():
                    log(f"  {name} rep{r}: can't idle")
                    continue

                # DNLOAD
                try:
                    self.dev.ctrl_transfer(0x21, 1, 0, 0, payload, timeout=5000)
                except usb.core.USBError as e:
                    log(f"  {name} rep{r}: DNLOAD err {e}")
                    continue

                # GET_STATUS → triggers manifest
                t0 = self.precise_time()
                st = self.get_status()
                t_first_status = self.precise_time()

                if not st:
                    log(f"  {name} rep{r}: no status")
                    continue

                first_state = st["bState"]
                first_poll = st["poll_ms"]

                # If BUSY (state 4), poll until done
                if first_state == 4:
                    while True:
                        time.sleep(max(first_poll / 1000.0, 0.01))
                        t_poll = self.precise_time()
                        st = self.get_status()
                        if not st or st["bState"] != 4:
                            break
                        if (t_poll - t0) / 1e9 > 10:
                            break  # 10s max

                t_done = self.precise_time()

                final_state = st["bState"] if st else None
                total_ms = (t_done - t0) / 1e6
                first_status_ms = (t_first_status - t0) / 1e6

                timings.append({
                    "total_ms": round(total_ms, 3),
                    "first_status_ms": round(first_status_ms, 3),
                    "first_state": first_state,
                    "first_poll_ms": first_poll,
                    "final_state": final_state,
                })

                # Reset
                if final_state == 10:
                    self.dev.ctrl_transfer(0x21, 4, 0, 0, 0, timeout=2000)
                elif final_state == 8:
                    self.reset_idle()

            if timings:
                totals = [t["total_ms"] for t in timings]
                entry = {
                    "name": name,
                    "mean_ms": round(statistics.mean(totals), 3),
                    "median_ms": round(statistics.median(totals), 3),
                    "stdev_ms": round(statistics.stdev(totals), 3) if len(totals) > 1 else 0,
                    "min_ms": round(min(totals), 3),
                    "max_ms": round(max(totals), 3),
                    "first_poll_ms": timings[0]["first_poll_ms"],
                    "first_state": timings[0]["first_state"],
                    "final_state": timings[0]["final_state"],
                    "all_timings": timings,
                }
                results.append(entry)
                log(f"  {name:20s}: mean={entry['mean_ms']:.3f}ms "
                    f"median={entry['median_ms']:.3f}ms stdev={entry['stdev_ms']:.3f}ms "
                    f"[{entry['min_ms']:.3f}-{entry['max_ms']:.3f}] "
                    f"poll={entry['first_poll_ms']}ms state:{entry['first_state']}→{entry['final_state']}")

        return {"test": "manifest_timing", "payloads": results}

    # ================================================================
    # TEST 4: GET_STATUS poll_ms as function of payload
    # ================================================================
    def test_poll_ms_oracle(self, repeats=3):
        """
        The poll_ms field in GET_STATUS tells us how long the device
        expects to be busy. This is set by SecureROM code —
        different code paths may report different poll values.
        """
        log(f"\n{'='*60}")
        log(f"TEST 4: poll_ms oracle (content-dependent?)")
        log(f"{'='*60}")

        test_payloads = {
            "16B_min":      b"\x00" * 16,
            "32B":          b"\x00" * 32,
            "64B":          b"\x00" * 64,
            "256B":         b"\x00" * 256,
            "512B":         b"\x00" * 512,
            "1024B":        b"\x00" * 1024,
            "2048B":        b"\x00" * 2048,
            "4096B":        b"\x00" * 4096,
            "img4_256":     b"\x30\x82\x10\x00\x16\x04IMG4" + b"\x00" * 246,
            "img4_2048":    b"\x30\x82\x10\x00\x16\x04IMG4" + b"\x00" * 2038,
            "der_nested":   b"\x30\x82\x08\x00" + b"\x30\x82\x04\x00" + b"\x00" * 248,
            "0xFF_512":     b"\xFF" * 512,
        }

        results = []

        for name, payload in test_payloads.items():
            polls = []
            states_after = []

            for r in range(repeats):
                if not self.reset_idle():
                    continue

                try:
                    self.dev.ctrl_transfer(0x21, 1, 0, 0, payload, timeout=5000)
                except:
                    continue

                # First GET_STATUS
                st = self.get_status()
                if st:
                    polls.append(st["poll_ms"])
                    states_after.append(st["bState"])

                    # If BUSY, get another status after poll delay
                    if st["bState"] == 4:
                        time.sleep(st["poll_ms"] / 1000.0 + 0.01)
                        st2 = self.get_status()
                        if st2:
                            states_after.append(st2["bState"])

                # Reset
                try: self.dev.ctrl_transfer(0x21, 6, 0, 0, 0, timeout=2000)
                except: pass
                st = self.get_status()
                if st and st["bState"] == 10:
                    self.dev.ctrl_transfer(0x21, 4, 0, 0, 0, timeout=2000)

            if polls:
                entry = {
                    "name": name,
                    "size": len(payload),
                    "poll_ms_values": polls,
                    "unique_polls": sorted(set(polls)),
                    "states": sorted(set(states_after)),
                }
                results.append(entry)
                log(f"  {name:20s} ({len(payload):5d}B): poll_ms={sorted(set(polls))} states={sorted(set(states_after))}")

        return {"test": "poll_ms_oracle", "results": results}

    # ================================================================
    # TEST 5: Precise crash timing for sub-16B payloads
    # ================================================================
    def test_crash_timing(self, repeats=3):
        """
        Measure exact timing for payloads that crash (<16B).
        Use partial overwrite technique: 512B primer then small overwrite.
        The crash timing tells us how many cycles to the exception.
        """
        log(f"\n{'='*60}")
        log(f"TEST 5: Sub-16B crash timing analysis")
        log(f"{'='*60}")

        sizes = [1, 2, 4, 8, 10, 12, 14, 15, 16, 17, 18, 20]
        results = []

        for size in sizes:
            timings = []

            for r in range(repeats):
                if not self.reset_idle():
                    continue

                # DNLOAD
                payload = bytes([0x41]) * size
                try:
                    self.dev.ctrl_transfer(0x21, 1, 0, 0, payload, timeout=5000)
                except:
                    continue

                # Trigger manifest and measure
                t0 = self.precise_time()
                st = self.get_status()
                t1 = self.precise_time()

                first_ms = (t1 - t0) / 1e6
                first_state = st["bState"] if st else None
                first_poll = st["poll_ms"] if st else None

                if st and st["bState"] == 4:
                    # Wait for processing
                    time.sleep(max(st["poll_ms"] / 1000.0, 0.01))
                    t2 = self.precise_time()
                    st2 = self.get_status()
                    t3 = self.precise_time()
                    total_ms = (t3 - t0) / 1e6
                    final_state = st2["bState"] if st2 else None
                else:
                    total_ms = first_ms
                    final_state = first_state

                timings.append({
                    "first_ms": round(first_ms, 3),
                    "total_ms": round(total_ms, 3),
                    "first_state": first_state,
                    "first_poll_ms": first_poll,
                    "final_state": final_state,
                })

                # Reset
                if final_state == 10:
                    try: self.dev.ctrl_transfer(0x21, 4, 0, 0, 0, timeout=2000)
                    except: pass
                elif final_state == 8:
                    self.reset_idle()

            if timings:
                totals = [t["total_ms"] for t in timings]
                entry = {
                    "size": size,
                    "mean_ms": round(statistics.mean(totals), 3),
                    "median_ms": round(statistics.median(totals), 3),
                    "stdev_ms": round(statistics.stdev(totals), 3) if len(totals) > 1 else 0,
                    "first_poll_ms": timings[0]["first_poll_ms"],
                    "first_state": timings[0]["first_state"],
                    "final_state": timings[0]["final_state"],
                    "all_timings": timings,
                }
                results.append(entry)
                log(f"  {size:3d}B: mean={entry['mean_ms']:.3f}ms stdev={entry['stdev_ms']:.3f}ms "
                    f"poll={entry['first_poll_ms']}ms state:{entry['first_state']}→{entry['final_state']}")

        return {"test": "crash_timing", "results": results}

    # ================================================================
    # TEST 6: Control transfer baseline (non-DFU)
    # ================================================================
    def test_usb_baseline(self, count=100):
        """
        Measure non-DFU control transfers to establish USB latency baseline.
        GET_DESCRIPTOR is always fast — pure USB overhead.
        """
        log(f"\n{'='*60}")
        log(f"TEST 6: USB baseline (GET_DESCRIPTOR) ({count} samples)")
        log(f"{'='*60}")

        times_us = []
        for i in range(count):
            t0 = self.precise_time()
            try:
                # GET_DESCRIPTOR: device descriptor
                data = self.dev.ctrl_transfer(0x80, 6, 0x0100, 0, 18, timeout=1000)
                t1 = self.precise_time()
                times_us.append((t1 - t0) / 1000)
            except:
                pass

        if not times_us:
            return {"error": "no_data"}

        result = {
            "test": "usb_baseline",
            "samples": len(times_us),
            "mean_us": round(statistics.mean(times_us), 1),
            "median_us": round(statistics.median(times_us), 1),
            "stdev_us": round(statistics.stdev(times_us), 1) if len(times_us) > 1 else 0,
            "min_us": round(min(times_us), 1),
            "max_us": round(max(times_us), 1),
        }
        log(f"  Mean:   {result['mean_us']:.1f} us")
        log(f"  Median: {result['median_us']:.1f} us")
        log(f"  Stdev:  {result['stdev_us']:.1f} us")
        log(f"  Range:  [{result['min_us']:.1f}, {result['max_us']:.1f}] us")

        return result

    # ================================================================
    # Main
    # ================================================================
    def run(self, tests="all"):
        log("=" * 60)
        log("A12 SecureROM — Precision Timing Oracle")
        log("=" * 60)

        if not self.connect():
            log("NO DFU DEVICE!"); return

        st = self.get_status()
        log(f"Device: state={st['bState']}, poll_ms={st['poll_ms']}")

        all_results = {"timestamp": datetime.now().isoformat()}

        test_map = {
            "baseline":  ("usb_baseline",      self.test_usb_baseline),
            "rtt":       ("get_status_rtt",     self.test_get_status_rtt),
            "dnload":    ("dnload_transfer",    self.test_dnload_transfer_time),
            "manifest":  ("manifest_timing",    self.test_manifest_timing),
            "poll":      ("poll_ms_oracle",     self.test_poll_ms_oracle),
            "crash":     ("crash_timing",       self.test_crash_timing),
        }

        if tests == "all":
            run_tests = ["baseline", "rtt", "dnload", "poll", "crash", "manifest"]
        else:
            run_tests = [t.strip() for t in tests.split(",")]

        for tname in run_tests:
            if tname not in test_map:
                log(f"Unknown test: {tname}")
                continue
            key, fn = test_map[tname]
            try:
                all_results[key] = fn()
            except Exception as e:
                log(f"Exception in {tname}: {e}")
                import traceback; traceback.print_exc()

            # Health check
            if not self.connect():
                log("Device gone, stopping")
                break

        # Clock estimation
        self._estimate_clock(all_results)

        # Save
        outf = Path(__file__).parent / "results" / "timing_oracle.json"
        outf.parent.mkdir(exist_ok=True)
        with open(outf, "w") as f:
            json.dump(all_results, f, indent=2)
        log(f"\nSaved: {outf}")

    def _estimate_clock(self, results):
        """Try to estimate SecureROM clock from collected timings"""
        log(f"\n{'='*60}")
        log(f"CLOCK ESTIMATION")
        log(f"{'='*60}")

        # USB baseline = pure USB overhead
        baseline_us = None
        if "usb_baseline" in results:
            baseline_us = results["usb_baseline"].get("median_us")
            log(f"  USB baseline (GET_DESCRIPTOR): {baseline_us:.1f} us")

        # GET_STATUS overhead
        status_us = None
        if "get_status_rtt" in results:
            status_us = results["get_status_rtt"].get("median_us")
            log(f"  GET_STATUS RTT: {status_us:.1f} us")
            if baseline_us:
                processing = status_us - baseline_us
                log(f"  GET_STATUS processing: {processing:.1f} us (RTT - baseline)")

        # DNLOAD throughput
        if "dnload_transfer" in results:
            slope = results["dnload_transfer"].get("slope_us_per_byte")
            if slope and slope > 0:
                log(f"  DNLOAD: {slope:.3f} us/byte = {1/slope:.2f} MB/s")
                # USB Full Speed max = ~1.216 MB/s for control (64B packets, 1ms frames)
                # USB High Speed max = ~48 MB/s for control (64B packets, 125us microframes)
                if 1/slope > 10:
                    log(f"  → Likely USB High Speed (480 Mbps)")
                else:
                    log(f"  → Likely USB Full Speed (12 Mbps)")

        # poll_ms analysis
        if "poll_ms_oracle" in results:
            all_polls = set()
            for r in results["poll_ms_oracle"].get("results", []):
                all_polls.update(r.get("unique_polls", []))
            log(f"  Unique poll_ms values: {sorted(all_polls)}")
            if all_polls:
                min_poll = min(all_polls)
                log(f"  Minimum timer tick: {min_poll}ms")
                if min_poll > 0:
                    # poll_ms likely derived from a hardware timer
                    # Common ARM timer divisors from 24MHz crystal:
                    # 24MHz / 24000 = 1kHz = 1ms tick
                    # 24MHz / 48000 = 500Hz = 2ms tick
                    log(f"  If 24MHz crystal: tick = 24000000 / {1000//min_poll} Hz")

        log(f"\n  Known A12 clocks:")
        log(f"    Crystal oscillator: 24 MHz (typical)")
        log(f"    Perf core (Vortex): 2.49 GHz")
        log(f"    Eff core (Tempest): 1.59 GHz")
        log(f"    SecureROM likely runs at fixed boot frequency")

if __name__ == "__main__":
    p = argparse.ArgumentParser()
    p.add_argument("--tests", default="all",
                   help="Comma-separated: baseline,rtt,dnload,poll,crash,manifest or all")
    args = p.parse_args()

    oracle = TimingOracle()
    oracle.run(args.tests)
