#!/usr/bin/env python3
"""
DFU Round 4 — Safe probes only
No SET_FEATURE, no USB Test Mode, no IMG4 DNLOAD
Just: unusual types, knock sequences, safe DNLOAD magics, vendor sweep
"""

import usb.core
import usb.util
import struct
import time
import sys

VID = 0x05AC
PID = 0x1227

DFU_DETACH    = 0
DFU_DNLOAD    = 1
DFU_UPLOAD    = 2
DFU_GETSTATUS = 3
DFU_CLRSTATUS = 4
DFU_GETSTATE  = 5
DFU_ABORT     = 6

findings = []

def log(msg, important=False):
    prefix = ">>>" if important else "   "
    print(f"{prefix} {msg}", flush=True)
    if important:
        findings.append(msg)

def get_dev():
    dev = usb.core.find(idVendor=VID, idProduct=PID)
    if dev is None:
        print("ERROR: No DFU device")
        sys.exit(1)
    try:
        dev.set_configuration()
    except:
        pass
    return dev

def safe_in(dev, bm, bReq, wVal, wIdx, length, timeout=500):
    try:
        data = dev.ctrl_transfer(bm, bReq, wVal, wIdx, length, timeout)
        if len(data) > 0:
            return bytes(data)
    except:
        pass
    return None

def safe_out(dev, bm, bReq, wVal, wIdx, data=None, timeout=500):
    try:
        return dev.ctrl_transfer(bm, bReq, wVal, wIdx, data or b'', timeout)
    except:
        pass
    return None

def alive(dev):
    try:
        dev.ctrl_transfer(0xA1, DFU_GETSTATE, 0, 0, 1, 500)
        return True
    except:
        return False

def reset_dfu(dev):
    safe_out(dev, 0x21, DFU_ABORT, 0, 0)
    safe_out(dev, 0x21, DFU_CLRSTATUS, 0, 0)
    safe_out(dev, 0x21, DFU_ABORT, 0, 0)
    time.sleep(0.02)

def get_state(dev):
    s = safe_in(dev, 0xA1, DFU_GETSTATE, 0, 0, 1)
    return s[0] if s else None

STATE_NAMES = {0:'appIDLE', 2:'dfuIDLE', 3:'dnSYNC', 4:'dnBUSY',
               5:'dnIDLE', 6:'mfSYNC', 7:'MANIFEST', 10:'ERROR'}

# ============================================================
# PROBE 1: Unusual bmRequestType (other, reserved, class)
# ============================================================
def probe_unusual_types(dev):
    print("\n" + "="*70)
    print("PROBE 1: Unusual bmRequestType IN scan")
    print("="*70)
    
    for bm in [0xA0, 0xA2, 0xA3, 0xC0, 0xC2, 0xC3, 0xE0, 0xE1, 0xE2, 0xE3]:
        print(f"\n  --- 0x{bm:02X} IN ---", flush=True)
        count = 0
        for bReq in range(256):
            data = safe_in(dev, bm, bReq, 0, 0, 256, 80)
            if data:
                # Skip already-known responses
                if bm == 0xC1 and bReq in [0x03, 0x05]:
                    continue
                count += 1
                log(f"0x{bm:02X} bReq=0x{bReq:02X}: {len(data)}B = {data[:20].hex()}", True)
        if count == 0:
            print(f"      (nothing)")
        if not alive(dev):
            log("Device died during type scan!", True)
            return False
    
    # OUT scan on unusual types
    for bm in [0x03, 0x23, 0x43, 0x63, 0x60, 0x61]:
        print(f"\n  --- 0x{bm:02X} OUT ---", flush=True)
        for bReq in range(256):
            ret = safe_out(dev, bm, bReq, 0, 0)
            if ret is not None:
                state = get_state(dev)
                log(f"0x{bm:02X} bReq=0x{bReq:02X} OUT accepted! State={state}", True)
                if state != 2:
                    reset_dfu(dev)
        if not alive(dev):
            log("Device died during OUT scan!", True)
            return False
    return True

# ============================================================
# PROBE 2: Knock sequences (with and without HALT-free UAF attempts)
# ============================================================
def probe_knock_sequences(dev):
    print("\n" + "="*70)
    print("PROBE 2: Knock sequences")
    print("="*70)
    
    def run_seq(name, steps):
        reset_dfu(dev)
        results = []
        for step in steps:
            op = step[0]
            if op == 'dn':
                safe_out(dev, 0x21, DFU_DNLOAD, 0, 0, step[1])
                results.append('dn')
            elif op == 'ab':
                safe_out(dev, 0x21, DFU_ABORT, 0, 0)
                results.append('ab')
            elif op == 'gs':
                d = safe_in(dev, 0xA1, DFU_GETSTATUS, 0, 0, 6)
                results.append(f'gs:{d[4] if d else "X"}')
            elif op == 'up':
                d = safe_in(dev, 0xA1, DFU_UPLOAD, 0, 0, step[1])
                sz = len(d) if d else 0
                results.append(f'up:{sz}')
                if d and sz > 0:
                    log(f"Seq '{name}' UPLOAD={sz}B: {d[:32].hex()}", True)
            elif op == 'cl':
                safe_out(dev, 0x21, DFU_CLRSTATUS, 0, 0)
                results.append('cl')
            elif op == 'det':
                safe_out(dev, 0x21, DFU_DETACH, 0, 0)
                results.append('det')
            elif op == 'sleep':
                time.sleep(step[1])
            time.sleep(0.002)
        
        state = get_state(dev)
        final = state if state is not None else 'DEAD'
        print(f"  {name}: {' -> '.join(results)} -> final={final}")
        if final == 'DEAD':
            log(f"Seq '{name}' killed device!", True)
        elif isinstance(final, int) and final not in [2, 10]:
            log(f"Seq '{name}' -> unexpected state {final}!", True)
        return alive(dev)
    
    seqs = {
        'uaf_upload': [('dn', b'\x41'*256), ('gs',), ('ab',), ('up', 4096)],
        'multi_abort': [('dn', b'\x42'*512), ('gs',), ('ab',), ('ab',), ('ab',), ('up', 4096)],
        'double_dn': [('dn', b'\x43'*64), ('dn', b'\x44'*128), ('gs',), ('up', 4096)],
        'abort_mid': [('dn', b'\x45'*2048), ('gs',), ('gs',), ('ab',), ('up', 4096)],
        'alloc_free_5x': [
            ('dn', b'\x50'*0x800), ('ab',), ('dn', b'\x51'*0x800), ('ab',),
            ('dn', b'\x52'*0x800), ('ab',), ('dn', b'\x53'*0x800), ('ab',),
            ('dn', b'\x54'*0x800), ('ab',), ('up', 4096)
        ],
        'size_vary': [
            ('dn', b'\x60'*16), ('ab',), ('dn', b'\x61'*256), ('ab',),
            ('dn', b'\x62'*4096), ('ab',), ('dn', b'\x63'*16), ('ab',), ('up', 4096)
        ],
        'clr_spam': [('cl',), ('cl',), ('cl',), ('cl',), ('cl',), ('gs',), ('up', 256)],
        'slow_uaf': [('dn', b'\x70'*512), ('gs',), ('sleep', 0.5), ('ab',), ('sleep', 0.1), ('up', 4096)],
        'detach_ops': [('det',), ('gs',), ('dn', b'\x80'*64), ('gs',), ('up', 256)],
        'dn_gs_gs_gs_ab': [('dn', b'\x90'*2048), ('gs',), ('gs',), ('gs',), ('ab',), ('up', 4096)],
        'triple_abort': [('ab',), ('ab',), ('ab',), ('gs',), ('up', 256)],
        'dn_cl_dn': [('dn', b'\xA0'*64), ('gs',), ('cl',), ('dn', b'\xA1'*64), ('gs',), ('up', 256)],
    }
    
    for name, seq in seqs.items():
        if not run_seq(name, seq):
            return False
    return True

# ============================================================
# PROBE 3: Safe DNLOAD magics
# ============================================================
def probe_dnload_magics(dev):
    print("\n" + "="*70)
    print("PROBE 3: DNLOAD magic headers (safe ones only)")
    print("="*70)
    
    magics = {
        'DBUG': b'DBUG' + struct.pack('<I', 0x8020),
        'JTAG': b'JTAG' + struct.pack('<I', 0x8020),
        'UART': b'UART' + struct.pack('<I', 1),
        'SWD':  b'SWD\x00' + struct.pack('<I', 1),
        'TEST': b'TEST' + b'\x00' * 4,
        'APPL': b'APPL' + b'\x00' * 4,
        'SRTG': b'SRTG[iBoot-3865.0.0.4.7]',
        'CPID': b'CPID:8020',
        'ECID': struct.pack('<Q', 0x001C15D43C20802E),
        'zeros': b'\x00' * 16,
        'ones':  b'\xFF' * 16,
        'nop4': struct.pack('<I', 0xD503201F) * 4,
        'iBSS': b'ibss' + b'\x00' * 12,
        'iBEC': b'ibec' + b'\x00' * 12,
        'LLB':  b'illb' + b'\x00' * 12,
        'iBot': b'iBot' + b'\x00' * 12,
        'PWND': b'PWND:[checkm8]',
        'MEMC': b'MEMC' + b'\x00' * 12,
        'KBAG': b'KBAG' + b'\x00' * 12,
        'SHSH': b'SHSH' + b'\x00' * 12,
        'CERT': b'CERT' + b'\x00' * 12,
        'SCAB': b'SCAB' + b'\x00' * 12,
        'SEPO': b'SEPO' + b'\x00' * 12,
    }
    
    for name, magic in magics.items():
        if not alive(dev):
            log("Device dead, stopping", True)
            return False
        reset_dfu(dev)
        
        safe_out(dev, 0x21, DFU_DNLOAD, 0, 0, magic)
        status = safe_in(dev, 0xA1, DFU_GETSTATUS, 0, 0, 6)
        
        if status:
            bState = status[4]
            pt = status[1] | (status[2] << 8) | (status[3] << 16)
            sname = STATE_NAMES.get(bState, f'UNK({bState})')
            
            if bState == 4:  # BUSY
                log(f"'{name}': ROM PROCESSING timeout={pt}ms!", True)
                time.sleep(pt / 1000.0 + 0.1)
                status2 = safe_in(dev, 0xA1, DFU_GETSTATUS, 0, 0, 6)
                if status2:
                    sname2 = STATE_NAMES.get(status2[4], f'UNK({status2[4]})')
                    if status2[4] != 10:
                        log(f"  After processing: state={sname2} ***", True)
                    else:
                        print(f"      {name}: processed -> ERROR (rejected, normal)")
                elif not alive(dev):
                    log(f"'{name}' killed ROM!", True)
                    return False
            elif bState not in [3, 10]:
                log(f"'{name}': unexpected state={sname}", True)
            else:
                print(f"      {name}: state={sname} pt={pt}ms")
        elif not alive(dev):
            log(f"'{name}' crashed ROM!", True)
            return False
        
        reset_dfu(dev)
    return True

# ============================================================
# PROBE 4: Vendor request sweep
# ============================================================
def probe_vendor_sweep(dev):
    print("\n" + "="*70)
    print("PROBE 4: Vendor request sweep (0xC0/0xC1 with wValue)")
    print("="*70)
    
    apple_wvals = [
        0x0001, 0x0002, 0x0010, 0x0100, 0x0200, 0x8000,
        0x8020, 0x000C, 0x05AC, 0x1227, 0x12A8, 0x1281,
        0xFFFF, 0xDEAD, 0x4442, 0x5347, 0x4A54, 0x5357, 0x5541,
    ]
    
    for bm in [0xC0, 0xC1]:
        tag = "dev" if bm == 0xC0 else "iface"
        print(f"\n  --- 0x{bm:02X} ({tag}) ---", flush=True)
        for wVal in apple_wvals:
            for bReq in range(256):
                data = safe_in(dev, bm, bReq, wVal, 0, 256, 80)
                if data:
                    if bm == 0xC1 and bReq == 0x03 and len(data) == 6:
                        continue
                    if bm == 0xC1 and bReq == 0x05 and len(data) == 1:
                        continue
                    log(f"0x{bm:02X}({tag}) bReq=0x{bReq:02X} wVal=0x{wVal:04X}: {len(data)}B = {data[:16].hex()}", True)
            if not alive(dev):
                log("Device died during vendor sweep!", True)
                return False
    
    # Vendor OUT
    print("\n  --- Vendor OUT (0x40/0x41) ---", flush=True)
    for bm_out in [0x40, 0x41]:
        for bReq in range(256):
            ret = safe_out(dev, bm_out, bReq, 0, 0)
            if ret is not None:
                state = get_state(dev)
                if state != 2:
                    log(f"0x{bm_out:02X} bReq=0x{bReq:02X} OUT changed state to {state}!", True)
                    reset_dfu(dev)
        if not alive(dev):
            log("Device died during vendor OUT!", True)
            return False
    return True

# ============================================================
# MAIN
# ============================================================
def main():
    print("=" * 70)
    print("DFU ROUND 4 — SAFE PROBES")
    print("No SET_FEATURE, no Test Mode, no IMG4/DER DNLOAD")
    print("=" * 70)
    
    dev = get_dev()
    serial = usb.util.get_string(dev, dev.iSerialNumber)
    print(f"\nDevice: {dev.idVendor:04X}:{dev.idProduct:04X}")
    print(f"Serial: {serial}")
    state = get_state(dev)
    print(f"State: {state} (2=dfuIDLE)\n")
    
    t0 = time.time()
    
    ok = probe_unusual_types(dev)
    if ok and alive(dev): ok = probe_knock_sequences(dev)
    if ok and alive(dev): ok = probe_dnload_magics(dev)
    if ok and alive(dev): ok = probe_vendor_sweep(dev)
    
    elapsed = time.time() - t0
    
    print("\n" + "=" * 70)
    print(f"ROUND 4 COMPLETE ({elapsed:.1f}s)")
    print("=" * 70)
    
    if findings:
        print(f"\n*** {len(findings)} FINDINGS ***\n")
        for i, f in enumerate(findings, 1):
            print(f"  [{i}] {f}")
    else:
        print("\nNo new findings. The door is well hidden.")

if __name__ == '__main__':
    main()
