#!/usr/bin/env python3
"""
ECID Hypothesis Test
====================
Tests whether matching the device's real ECID (0x001C15D43C20802E)
allows the SecureROM to proceed past MANP validation and into the
certificate chain parser.

Creates 2 identical payloads differing ONLY in ECID value:
  - ecid_zero.bin   : ECID = 0           (control — known ~330ms)
  - ecid_real.bin   : ECID = 0x001C15D43C20802E  (device match)

Then sends each in manifest mode and compares timing.
If ECID-real shows significantly different timing (>380ms or crash),
the ECID gating hypothesis is confirmed.
"""
import sys, time, struct, array, pathlib, json

# ---------------------------------------------------------------------------
# DER builder (minimal, same as in generators)
# ---------------------------------------------------------------------------
class D:
    @staticmethod
    def length(n):
        if n < 0x80: return bytes([n])
        if n < 0x100: return b'\x81' + bytes([n])
        if n < 0x10000: return b'\x82' + struct.pack('>H', n)
        return b'\x83' + struct.pack('>I', n)[1:]

    @staticmethod
    def raw_tlv(tag, data):
        if isinstance(tag, int): tag = bytes([tag])
        return tag + D.length(len(data)) + data

    @staticmethod
    def sequence(items):
        body = b''.join(items)
        return D.raw_tlv(0x30, body)

    @staticmethod
    def set_of(items):
        if isinstance(items, list):
            body = b''.join(items)
        else:
            body = items
        return D.raw_tlv(0x31, body)

    @staticmethod
    def integer(val):
        if val == 0: return b'\x02\x01\x00'
        neg = val < 0
        if neg: val = -val - 1
        bs = []
        while val > 0:
            bs.append(val & 0xff)
            val >>= 8
        bs.reverse()
        if neg:
            bs = [b ^ 0xff for b in bs]
            if bs[0] < 0x80: bs.insert(0, 0xff)
        else:
            if bs[0] >= 0x80: bs.insert(0, 0x00)
        return D.raw_tlv(0x02, bytes(bs))

    @staticmethod
    def ia5_string(s):
        return D.raw_tlv(0x16, s.encode('ascii') if isinstance(s, str) else s)

    @staticmethod
    def octet_string(data):
        return D.raw_tlv(0x04, data)

    @staticmethod
    def context_tag(num, data):
        return D.raw_tlv(0xa0 | num, data)

# ---------------------------------------------------------------------------
# Payload builders
# ---------------------------------------------------------------------------
def build_payload(ecid_value):
    """Build a minimal but valid-structured IMG4 with given ECID."""
    # IM4P
    im4p = D.sequence([
        D.ia5_string("IM4P"),
        D.ia5_string("illb"),
        D.ia5_string("iBoot"),
        D.octet_string(b'\x00' * 16),
    ])

    # A small but nontrivial fake cert chain to parse if ECID passes
    # X.509 TBSCertificate-like structure with nested SEQUENCEs
    fake_cert_inner = D.sequence([
        D.context_tag(0, D.integer(2)),           # version v3
        D.integer(0x1234),                          # serialNumber
        D.sequence([D.raw_tlv(0x06, b'\x2a\x86\x48\x86\xf7\x0d\x01\x01\x0b')]),  # sha256WithRSA OID
        D.sequence([                                 # issuer
            D.set_of([D.sequence([
                D.raw_tlv(0x06, b'\x55\x04\x03'),   # CN OID
                D.raw_tlv(0x0c, b'TestCA'),
            ])]),
        ]),
        D.sequence([                                 # validity
            D.raw_tlv(0x17, b'200101000000Z'),       # notBefore
            D.raw_tlv(0x17, b'301231235959Z'),       # notAfter
        ]),
        D.sequence([                                 # subject
            D.set_of([D.sequence([
                D.raw_tlv(0x06, b'\x55\x04\x03'),
                D.raw_tlv(0x0c, b'TestLeaf'),
            ])]),
        ]),
        D.sequence([                                 # subjectPublicKeyInfo (RSA)
            D.sequence([D.raw_tlv(0x06, b'\x2a\x86\x48\x86\xf7\x0d\x01\x01\x01'), D.raw_tlv(0x05, b'')]),
            D.raw_tlv(0x03, b'\x00' + b'\x00' * 128),  # fake 1024-bit key
        ]),
    ])
    fake_cert = D.sequence([
        fake_cert_inner,
        D.sequence([D.raw_tlv(0x06, b'\x2a\x86\x48\x86\xf7\x0d\x01\x01\x0b'), D.raw_tlv(0x05, b'')]),
        D.raw_tlv(0x03, b'\x00' + b'\x00' * 128),  # fake signature
    ])

    # IM4M with MANP properties
    manp = D.sequence([
        D.ia5_string("MANP"),
        D.set_of([
            D.sequence([D.ia5_string("CHIP"), D.integer(0x8020)]),
            D.sequence([D.ia5_string("ECID"), D.integer(ecid_value)]),
            D.sequence([D.ia5_string("BDID"), D.integer(0x0C)]),
            D.sequence([D.ia5_string("CPFM"), D.integer(0x03)]),
            D.sequence([D.ia5_string("CPRV"), D.integer(0x11)]),
            D.sequence([D.ia5_string("SDOM"), D.integer(0x01)]),
        ]),
    ])

    im4m = D.sequence([
        D.ia5_string("IM4M"),
        D.integer(0),            # version
        D.set_of(manp),          # manifest body (SET of properties)
        fake_cert,               # certificate chain
        D.octet_string(b'\x00' * 64),  # manifest signature
    ])

    img4 = D.sequence([
        D.ia5_string("IMG4"),
        im4p,
        D.context_tag(0, im4m),
    ])

    return img4

# ---------------------------------------------------------------------------
# USB helpers
# ---------------------------------------------------------------------------
def find_dfu():
    import usb.core
    return usb.core.find(idVendor=0x05ac, idProduct=0x1227)

def dfu_dnload(dev, data):
    dev.ctrl_transfer(0x21, 1, 0, 0, array.array('B', data), timeout=5000)

def dfu_get_status(dev):
    try:
        r = dev.ctrl_transfer(0xa1, 3, 0, 0, 6, timeout=5000)
        return {"status": r[0], "poll_timeout": r[1] | (r[2]<<8) | (r[3]<<16), "state": r[4]}
    except Exception:
        return None

def ensure_idle(dev, max_retries=10):
    """Get device back to dfuIDLE (state 2)."""
    for _ in range(max_retries):
        st = dfu_get_status(dev)
        if st is None: return False
        if st['state'] == 2:  # dfuIDLE
            return True
        if st['state'] == 10:  # dfuERROR
            dev.ctrl_transfer(0x21, 4, 0, 0, 0, timeout=5000)  # DFU_CLRSTATUS
            continue
        if st['state'] in (5, 9):  # dfuDNLOAD-IDLE / dfuUPLOAD-IDLE
            dev.ctrl_transfer(0x21, 6, 0, 0, 0, timeout=5000)  # DFU_ABORT
            continue
        # Other states — try abort
        try:
            dev.ctrl_transfer(0x21, 6, 0, 0, 0, timeout=5000)
        except Exception:
            pass
        time.sleep(0.1)
    return False

def wait_device(timeout=10.0):
    t0 = time.time()
    while time.time() - t0 < timeout:
        dev = find_dfu()
        if dev is not None:
            return dev
        time.sleep(0.2)
    return None

def dfu_clear_status(dev):
    try:
        dev.ctrl_transfer(0x21, 4, 0, 0, 0, timeout=5000)
    except Exception:
        pass

def dfu_abort(dev):
    try:
        dev.ctrl_transfer(0x21, 6, 0, 0, 0, timeout=5000)
    except Exception:
        pass

STATE_NAMES = {0:"appIDLE",1:"appDETACH",2:"dfuIDLE",3:"dfuDNLOAD-SYNC",
    4:"dfuDNBUSY",5:"dfuDNLOAD-IDLE",6:"dfuMANIFEST-SYNC",
    7:"dfuMANIFEST",8:"dfuMANIFEST-WAIT-RESET",9:"dfuUPLOAD-IDLE",10:"dfuERROR"}

def run_manifest_test(dev, payload):
    """Send payload via DNLOAD, trigger manifest, poll for result.
    Returns (result_str, elapsed_ms, states_seen)."""
    if not ensure_idle(dev):
        return "IDLE_FAIL", 0, []
    
    t_start = time.perf_counter()
    
    # Send payload
    try:
        dfu_dnload(dev, payload)
    except Exception as e:
        return f"DNLOAD_ERROR:{e}", 0, []
    
    # GET_STATUS to advance past DNLOAD-SYNC
    st = dfu_get_status(dev)
    if st is None:
        return "NO_STATUS_AFTER_DNLOAD", 0, []
    
    if st['state'] in (3, 5):  # dfuDNLOAD-SYNC or dfuDNLOAD-IDLE
        # Trigger manifest with zero-length DNLOAD  
        dfu_dnload(dev, b'')
    elif st['state'] == 10:  # dfuERROR
        dfu_clear_status(dev)
        elapsed = (time.perf_counter() - t_start) * 1000.0
        return "REJECTED_EARLY", elapsed, [STATE_NAMES.get(10,"?")]
    else:
        elapsed = (time.perf_counter() - t_start) * 1000.0
        return f"UNEXPECTED_STATE_{st['state']}", elapsed, []
    
    # Poll for manifest result (up to 10s)
    states_seen = []
    for _ in range(100):
        time.sleep(0.1)
        st = dfu_get_status(dev)
        if st is None:
            elapsed = (time.perf_counter() - t_start) * 1000.0
            return "DEVICE_LOST", elapsed, states_seen
        
        sname = STATE_NAMES.get(st['state'], f"UNK_{st['state']}")
        if sname not in states_seen:
            states_seen.append(sname)
        
        if st['state'] == 2:  # dfuIDLE — done
            elapsed = (time.perf_counter() - t_start) * 1000.0
            return "REJECTED", elapsed, states_seen
        if st['state'] == 10:  # dfuERROR — rejected
            elapsed = (time.perf_counter() - t_start) * 1000.0
            dfu_clear_status(dev)
            return "REJECTED", elapsed, states_seen
        if st['state'] == 8:  # dfuMANIFEST-WAIT-RESET — accepted!
            elapsed = (time.perf_counter() - t_start) * 1000.0
            return "MANIFEST_ACCEPTED", elapsed, states_seen
    
    elapsed = (time.perf_counter() - t_start) * 1000.0
    return "TIMEOUT", elapsed, states_seen

# ---------------------------------------------------------------------------
# Main
# ---------------------------------------------------------------------------
def main():
    dev = find_dfu()
    if dev is None:
        print("ERROR: No DFU device found")
        sys.exit(1)
    print("DFU device found")
    
    # Build payloads
    ecid_zero = build_payload(0)
    ecid_real = build_payload(0x001C15D43C20802E)
    
    print(f"Payload size (ECID=0):    {len(ecid_zero)} bytes")
    print(f"Payload size (ECID=real): {len(ecid_real)} bytes")
    
    # Save them
    outdir = pathlib.Path("test_payloads_ecid")
    outdir.mkdir(exist_ok=True)
    (outdir / "ecid_zero.bin").write_bytes(ecid_zero)
    (outdir / "ecid_real.bin").write_bytes(ecid_real)
    
    results = []
    
    # Run multiple rounds for statistical significance
    ROUNDS = 3
    
    for label, payload in [("ECID=0", ecid_zero), ("ECID=real(0x001C15D43C20802E)", ecid_real)]:
        print(f"\n{'='*60}")
        print(f"Testing: {label}")
        print(f"{'='*60}")
        
        for i in range(ROUNDS):
            dev = find_dfu()
            if dev is None:
                print(f"  Round {i+1}: Device lost, waiting...")
                dev = wait_device(15.0)
                if dev is None:
                    print(f"  Round {i+1}: DEVICE LOST - significant!")
                    results.append({"label": label, "round": i+1, "result": "DEVICE_LOST", "ms": 0, "states": []})
                    continue
            
            result, ms, states = run_manifest_test(dev, payload)
            print(f"  Round {i+1}: result={result}, time={ms:.1f}ms, states={states}")
            results.append({"label": label, "round": i+1, "result": result, "ms": round(ms, 2), "states": states})
            
            # Recovery: if device stuck in manifest-wait-reset
            if result == "MANIFEST_ACCEPTED":
                dfu_abort(find_dfu() or dev)
                time.sleep(0.5)
                dev2 = find_dfu()
                if dev2:
                    dfu_clear_status(dev2)
                    ensure_idle(dev2)
            
            time.sleep(0.5)
    
    # Summary
    print(f"\n{'='*60}")
    print("RESULTS SUMMARY")
    print(f"{'='*60}")
    
    for label in ["ECID=0", "ECID=real(0x001C15D43C20802E)"]:
        runs = [r for r in results if r['label'] == label]
        times = [r['ms'] for r in runs if r['ms'] > 0]
        res_list = [r['result'] for r in runs]
        avg = sum(times)/len(times) if times else 0
        print(f"\n{label}:")
        print(f"  Results: {res_list}")
        print(f"  States:  {[r['states'] for r in runs]}")
        print(f"  Times:   {[r['ms'] for r in runs]}")
        print(f"  Average: {avg:.1f}ms")
    
    # Verdict
    zero_times = [r['ms'] for r in results if r['label'] == "ECID=0" and r['ms'] > 0]
    real_times = [r['ms'] for r in results if r['label'] == "ECID=real(0x001C15D43C20802E)" and r['ms'] > 0]
    
    if zero_times and real_times:
        avg_zero = sum(zero_times)/len(zero_times)
        avg_real = sum(real_times)/len(real_times)
        delta = avg_real - avg_zero
        print(f"\nDelta (real - zero): {delta:+.1f}ms")
        if abs(delta) > 50:
            print(">>> ECID HYPOTHESIS CONFIRMED: Significant timing difference!")
            print(">>> SecureROM proceeds deeper with matching ECID.")
        elif abs(delta) > 20:
            print(">>> Possible ECID effect — marginal difference. Need more rounds.")
        else:
            print(">>> ECID does NOT appear to be the gatekeeper (similar timing).")
    
    # Check for different result types
    zero_results = set(r['result'] for r in results if r['label'] == "ECID=0")
    real_results = set(r['result'] for r in results if r['label'] == "ECID=real(0x001C15D43C20802E)")
    if zero_results != real_results:
        print(f"\n>>> RESULT TYPE DIFFERENCE DETECTED!")
        print(f">>>   ECID=0 results: {zero_results}")
        print(f">>>   ECID=real results: {real_results}")
    
    # Save results
    outfile = pathlib.Path("ecid_hypothesis_results.json")
    outfile.write_text(json.dumps(results, indent=2))
    print(f"\nResults saved to {outfile}")

if __name__ == "__main__":
    main()
