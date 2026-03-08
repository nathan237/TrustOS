#!/usr/bin/env python3
"""
Cert Size Scaling Test
======================
Tests if manifest processing time scales with cert chain data size.
If it does → cert data IS being read/copied/parsed.
If it doesn't → cert data is ignored, processing happens elsewhere.

Creates payloads with varying cert chain sizes:
  - 0 bytes (no cert)
  - 64 bytes
  - 256 bytes
  - 512 bytes  
  - 1024 bytes (near DFU max)
"""
import sys, time, struct, array, pathlib, json

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
        return D.raw_tlv(0x30, b''.join(items))
    @staticmethod
    def set_of(items):
        body = b''.join(items) if isinstance(items, list) else items
        return D.raw_tlv(0x31, body)
    @staticmethod
    def integer(val):
        if val == 0: return b'\x02\x01\x00'
        bs = []
        v = val
        while v > 0: bs.append(v & 0xff); v >>= 8
        bs.reverse()
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

def build_payload(cert_size):
    """IMG4 with variable cert chain size."""
    im4p = D.sequence([
        D.ia5_string("IM4P"),
        D.ia5_string("illb"),
        D.ia5_string("iBoot"),
        D.octet_string(b'\x00' * 16),
    ])
    
    manp = D.sequence([
        D.ia5_string("MANP"),
        D.set_of([
            D.sequence([D.ia5_string("CHIP"), D.integer(0x8020)]),
            D.sequence([D.ia5_string("ECID"), D.integer(0)]),
        ]),
    ])
    
    # cert chain: either empty or SEQUENCE of random-ish data
    if cert_size == 0:
        cert_data = D.sequence([])  # empty SEQUENCE
    else:
        cert_data = D.sequence([D.octet_string(b'\xAA' * cert_size)])
    
    im4m = D.sequence([
        D.ia5_string("IM4M"),
        D.integer(0),
        D.set_of(manp),
        cert_data,
        D.octet_string(b'\x00' * 32),  # sig
    ])
    
    return D.sequence([
        D.ia5_string("IMG4"),
        im4p,
        D.context_tag(0, im4m),
    ])

def find_dfu():
    import usb.core
    return usb.core.find(idVendor=0x05ac, idProduct=0x1227)

def dfu_dnload(dev, data):
    dev.ctrl_transfer(0x21, 1, 0, 0, array.array('B', data), timeout=5000)

def dfu_get_status(dev):
    try:
        r = dev.ctrl_transfer(0xa1, 3, 0, 0, 6, timeout=5000)
        return {"status": r[0], "poll_timeout": r[1]|(r[2]<<8)|(r[3]<<16), "state": r[4]}
    except: return None

def dfu_clear(dev):
    try: dev.ctrl_transfer(0x21, 4, 0, 0, 0, timeout=5000)
    except: pass

def dfu_abort(dev):
    try: dev.ctrl_transfer(0x21, 6, 0, 0, 0, timeout=5000)
    except: pass

def ensure_idle(dev, retries=15):
    for _ in range(retries):
        st = dfu_get_status(dev)
        if st is None: return False
        if st['state'] == 2: return True
        if st['state'] == 10: dfu_clear(dev); continue
        if st['state'] in (5,9,8): dfu_abort(dev); time.sleep(0.2); continue
        if st['state'] in (6,7): time.sleep(0.2); continue
        dfu_abort(dev); time.sleep(0.1)
    return False

def run_manifest(dev, payload):
    if not ensure_idle(dev):
        return None, 0
    
    t0 = time.perf_counter()
    dfu_dnload(dev, payload)
    st = dfu_get_status(dev)
    if st is None: return None, 0
    if st['state'] in (3,5):
        dfu_dnload(dev, b'')
    elif st['state'] == 10:
        dfu_clear(dev)
        return "REJECTED_EARLY", (time.perf_counter()-t0)*1000
    
    for _ in range(100):
        time.sleep(0.1)
        st = dfu_get_status(dev)
        if st is None: return "DEVICE_LOST", (time.perf_counter()-t0)*1000
        if st['state'] in (2,10):
            if st['state']==10: dfu_clear(dev)
            return "REJECTED", (time.perf_counter()-t0)*1000
        if st['state'] == 8:
            return "MANIFEST_ACCEPTED", (time.perf_counter()-t0)*1000
    
    return "TIMEOUT", (time.perf_counter()-t0)*1000

def main():
    dev = find_dfu()
    if not dev:
        print("No DFU device"); sys.exit(1)
    
    sizes = [0, 64, 256, 512, 1024]
    results = []
    ROUNDS = 2
    
    for sz in sizes:
        payload = build_payload(sz)
        # Truncate if > 0x800 (DFU max)
        if len(payload) > 0x800:
            print(f"  cert_size={sz}: payload too large ({len(payload)}), truncating to 0x800")
            payload = payload[:0x800]
        
        print(f"\ncert_size={sz:4d}  payload_total={len(payload):4d} bytes")
        
        for r in range(ROUNDS):
            dev = find_dfu()
            if not dev:
                time.sleep(2)
                dev = find_dfu()
                if not dev:
                    print(f"  Round {r+1}: DEVICE LOST"); continue
            
            result, ms = run_manifest(dev, payload)
            if result is None:
                print(f"  Round {r+1}: IDLE_FAIL")
                time.sleep(0.5)
                continue
            
            print(f"  Round {r+1}: {result:20s} {ms:7.1f}ms")
            results.append({"cert_size": sz, "payload_size": len(payload), "round": r+1, "result": result, "ms": round(ms, 2)})
            
            if result == "MANIFEST_ACCEPTED":
                dfu_abort(dev)
                time.sleep(0.5)
                dev2 = find_dfu()
                if dev2: dfu_clear(dev2); ensure_idle(dev2)
            time.sleep(0.5)
    
    print(f"\n{'='*60}")
    print("SIZE SCALING ANALYSIS")
    print(f"{'='*60}")
    for sz in sizes:
        runs = [r for r in results if r['cert_size'] == sz and r['ms'] > 0]
        if runs:
            avg = sum(r['ms'] for r in runs)/len(runs)
            print(f"  cert_size={sz:4d}  avg={avg:7.1f}ms  results={[r['result'] for r in runs]}")
    
    pathlib.Path("cert_size_scaling_results.json").write_text(json.dumps(results, indent=2))
    print("\nSaved to cert_size_scaling_results.json")

if __name__ == "__main__":
    main()
