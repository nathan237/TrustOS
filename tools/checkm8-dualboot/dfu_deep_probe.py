#!/usr/bin/env python3
"""
DFU Deep Probe - Round 2
========================
Focused on findings from round 1:
- 0xC1 (vendor iface IN) responds to DFU commands -> explore full vendor space
- SET_FEATURE(2) accepted -> explore all features
- IMG4 DNLOAD crashed ROM -> careful DNLOAD probing with recovery
- Probes 6/7/8 never ran -> run them now
- 0x83 "other" recipient responds -> unusual, investigate
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
    print(f"{prefix} {msg}")
    if important:
        findings.append(msg)

def get_dev():
    dev = usb.core.find(idVendor=VID, idProduct=PID)
    if dev is None:
        print("ERROR: No DFU device found")
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

# ============================================================
# PROBE A: Vendor interface 0xC1 deep sweep
# ============================================================
def probe_vendor_deep(dev):
    print("\n" + "="*70)
    print("PROBE A: Vendor IN (0xC0/0xC1) deep sweep")
    print("="*70)
    
    apple_wvals = [
        0x0001, 0x0002, 0x0003, 0x0004, 0x0005, 0x0010, 0x0020,
        0x0040, 0x0080, 0x0100, 0x0200, 0x0400, 0x0800, 0x1000,
        0x4000, 0x8000, 0x8020, 0x000C, 0x05AC, 0x1227, 0x12A8, 0x1281,
        0xFFFF, 0xDEAD, 0xBEEF, 0xCAFE, 0xA12B, 0x1337,
        0x4442, 0x5347, 0x4A54, 0x5357, 0x5541, 0x4150, 0x504C,
    ]
    
    for bm in [0xC0, 0xC1]:
        bm_name = "0xC0 vendor-dev" if bm == 0xC0 else "0xC1 vendor-iface"
        print(f"\n  --- {bm_name} wValue sweep ---")
        for wVal in apple_wvals:
            for bReq in range(256):
                data = safe_in(dev, bm, bReq, wVal, 0, 256, 80)
                if data:
                    if bm == 0xC1 and bReq == 0x03 and len(data) == 6:
                        continue
                    if bm == 0xC1 and bReq == 0x05 and len(data) == 1:
                        continue
                    log(f"{bm_name} bReq=0x{bReq:02X} wVal=0x{wVal:04X}: {len(data)}B = {data[:24].hex()}", True)
            if not alive(dev):
                log("Device died during vendor sweep!", True)
                return
    
    # Vendor OUT
    print("\n  --- 0x40/0x41 vendor OUT sweep ---")
    for bm_out in [0x40, 0x41]:
        for bReq in range(256):
            ret = safe_out(dev, bm_out, bReq, 0, 0)
            if ret is not None:
                state = safe_in(dev, 0xA1, DFU_GETSTATE, 0, 0, 1)
                sval = state[0] if state else -1
                if sval != 2:
                    log(f"0x{bm_out:02X} bReq=0x{bReq:02X} OUT accepted! State={sval}", True)
                    reset_dfu(dev)
        if not alive(dev):
            log("Device died during vendor OUT sweep!", True)
            return

# ============================================================
# PROBE B: SET_FEATURE deep
# ============================================================
def probe_features_deep(dev):
    print("\n" + "="*70)
    print("PROBE B: SET_FEATURE / CLEAR_FEATURE deep")
    print("="*70)
    
    status = safe_in(dev, 0x80, 0x00, 0, 0, 2)
    print(f"  Device GET_STATUS: {status.hex() if status else 'None'}")
    
    print("\n  --- SET_FEATURE(device) 0-255 ---")
    for feat in range(256):
        ret = safe_out(dev, 0x00, 0x03, feat, 0)
        if ret is not None:
            new_status = safe_in(dev, 0x80, 0x00, 0, 0, 2)
            log(f"SET_FEATURE({feat}) accepted! Status={new_status.hex() if new_status else 'dead'}", feat != 2)
            if not alive(dev):
                log(f"SET_FEATURE({feat}) KILLED device!", True)
                return
    
    print("\n  --- SET_FEATURE(interface) ---")
    for feat in range(32):
        ret = safe_out(dev, 0x01, 0x03, feat, 0)
        if ret is not None:
            log(f"SET_FEATURE(iface, {feat}) accepted!", True)
    
    print("\n  --- SET_FEATURE(endpoint) ---")
    for feat in range(32):
        for ep in [0x00, 0x80]:
            ret = safe_out(dev, 0x02, 0x03, feat, ep)
            if ret is not None:
                log(f"SET_FEATURE(ep=0x{ep:02X}, feat={feat}) accepted!", True)
    
    # USB Test Mode
    print("\n  --- USB Test Mode selectors ---")
    for selector in range(1, 8):
        wIndex = selector << 8
        ret = safe_out(dev, 0x00, 0x03, 2, wIndex)
        if ret is not None:
            log(f"USB Test Mode selector {selector} ACCEPTED!", True)
            if not alive(dev):
                log(f"Test Mode {selector} made device unresponsive!", True)
                return
    
    print("\n  --- CLEAR_FEATURE ---")
    for feat in range(32):
        ret = safe_out(dev, 0x00, 0x01, feat, 0)
        if ret is not None:
            log(f"CLEAR_FEATURE({feat}) accepted!", feat > 2)

# ============================================================
# PROBE C: 'Other' recipient + Reserved types
# ============================================================
def probe_unusual_types(dev):
    print("\n" + "="*70)
    print("PROBE C: Unusual bmRequestType (other, reserved)")
    print("="*70)
    
    for bm in [0x83, 0xA3, 0xC3, 0xE3, 0xE0, 0xE1, 0xE2]:
        print(f"\n  --- 0x{bm:02X} IN scan ---")
        for bReq in range(256):
            for wVal in [0, 0x8020, 0xFFFF]:
                data = safe_in(dev, bm, bReq, wVal, 0, 256, 80)
                if data:
                    if bm == 0x83 and wVal == 0 and bReq in [0x00, 0x08]:
                        continue  # Already known
                    log(f"0x{bm:02X} bReq=0x{bReq:02X} wVal=0x{wVal:04X}: {len(data)}B = {data[:16].hex()}", True)
        if not alive(dev):
            log("Device died!", True)
            return
    
    for bm in [0x03, 0x23, 0x43, 0x63, 0x60, 0x61]:
        print(f"\n  --- 0x{bm:02X} OUT scan ---")
        for bReq in range(256):
            ret = safe_out(dev, bm, bReq, 0, 0)
            if ret is not None:
                state = safe_in(dev, 0xA1, DFU_GETSTATE, 0, 0, 1)
                sval = state[0] if state else -1
                log(f"0x{bm:02X} bReq=0x{bReq:02X} OUT accepted! State={sval}", True)
                if sval != 2:
                    reset_dfu(dev)
        if not alive(dev):
            return

# ============================================================
# PROBE D: DNLOAD magic (careful)
# ============================================================
def probe_dnload_careful(dev):
    print("\n" + "="*70)
    print("PROBE D: Careful DNLOAD magic probing")
    print("="*70)
    
    magics = {
        'DBUG_cmd': b'DBUG' + struct.pack('<I', 0x8020),
        'JTAG_cmd': b'JTAG' + struct.pack('<I', 0x8020),
        'UART_cmd': b'UART' + struct.pack('<I', 1),
        'SWD_cmd':  b'SWD\x00' + struct.pack('<I', 1),
        'TEST_cmd': b'TEST' + b'\x00' * 4,
        'APPL_cmd': b'APPL' + b'\x00' * 4,
        'SRTG_tag': b'SRTG[iBoot-3865.0.0.4.7]',
        'CPID_tag': b'CPID:8020',
        'ECID_val': struct.pack('<Q', 0x001C15D43C20802E),
        'zeros_16': b'\x00' * 16,
        'ones_16':  b'\xFF' * 16,
        'ARM64_nop': struct.pack('<I', 0xD503201F) * 4,
        'ARM64_svc': struct.pack('<I', 0xD4000001),
        'iBSS_hdr': b'ibss' + b'\x00' * 12,
        'iBEC_hdr': b'ibec' + b'\x00' * 12,
        'LLB_hdr':  b'illb' + b'\x00' * 12,
        'iBot_hdr': b'iBot' + b'\x00' * 12,
        'DER_seq':  bytes([0x30, 0x04, 0x16, 0x02, 0x41, 0x42]),
        'IM4P_hdr': b'\x30\x82\x00\x10' + b'IM4P' + b'\x00' * 8,
    }
    
    state_names = {0:'appIDLE', 2:'dfuIDLE', 3:'dnSYNC', 4:'dnBUSY', 
                   5:'dnIDLE', 6:'mfSYNC', 7:'MANIFEST', 10:'ERROR'}
    
    for name, magic in magics.items():
        if not alive(dev):
            log("Device dead, stopping", True)
            return
        
        reset_dfu(dev)
        
        # Send + immediate abort for DER/IMG stuff (known crash risk)
        if name in ['DER_seq', 'IM4P_hdr']:
            safe_out(dev, 0x21, DFU_DNLOAD, 0, 0, magic)
            safe_out(dev, 0x21, DFU_ABORT, 0, 0)
            time.sleep(0.01)
            state = safe_in(dev, 0xA1, DFU_GETSTATE, 0, 0, 1)
            if state is None:
                log(f"'{name}' crashed ROM even with abort!", True)
                return
            print(f"  {name}: abort-saved, state={state[0]}")
            continue
        
        # Safe ones: send + GETSTATUS to trigger processing
        ret = safe_out(dev, 0x21, DFU_DNLOAD, 0, 0, magic)
        status = safe_in(dev, 0xA1, DFU_GETSTATUS, 0, 0, 6)
        
        if status:
            bState = status[4]
            pollTimeout = status[1] | (status[2] << 8) | (status[3] << 16)
            sname = state_names.get(bState, f'UNK({bState})')
            
            if bState == 4:  # BUSY = processing
                log(f"'{name}': ROM PROCESSING timeout={pollTimeout}ms", True)
                time.sleep(pollTimeout / 1000.0 + 0.1)
                status2 = safe_in(dev, 0xA1, DFU_GETSTATUS, 0, 0, 6)
                if status2:
                    bState2 = status2[4]
                    sname2 = state_names.get(bState2, f'UNK({bState2})')
                    log(f"  After: state={sname2}", bState2 != 10)
                elif not alive(dev):
                    log(f"  '{name}' killed ROM during processing!", True)
                    return
            elif bState not in [3, 10]:
                log(f"'{name}': unexpected state={sname}", True)
            else:
                print(f"  {name}: state={sname} (normal)")
        elif not alive(dev):
            log(f"'{name}' crashed ROM!", True)
            return
        
        reset_dfu(dev)

# ============================================================
# PROBE E: Knock sequences
# ============================================================
def probe_knock_sequences(dev):
    print("\n" + "="*70)
    print("PROBE E: Knock sequences")
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
            elif op == 'sf':
                safe_out(dev, 0x00, 0x03, step[1], 0)
                results.append(f'sf:{step[1]}')
            elif op == 'sleep':
                time.sleep(step[1])
            time.sleep(0.002)
        
        state = safe_in(dev, 0xA1, DFU_GETSTATE, 0, 0, 1)
        final = state[0] if state else 'DEAD'
        print(f"  {name}: {' -> '.join(results)} -> final={final}")
        if final == 'DEAD':
            log(f"Sequence '{name}' killed device!", True)
        elif isinstance(final, int) and final not in [2, 10]:
            log(f"Sequence '{name}' -> unexpected state {final}!", True)
        return alive(dev)
    
    seqs = {
        'uaf_upload': [('dn', b'\x41'*256), ('gs',), ('ab',), ('up', 4096)],
        'multi_abort_up': [('dn', b'\x42'*512), ('gs',), ('ab',), ('ab',), ('ab',), ('up', 4096)],
        'double_dn': [('dn', b'\x43'*64), ('dn', b'\x44'*128), ('gs',), ('up', 4096)],
        'abort_mid': [('dn', b'\x45'*2048), ('gs',), ('gs',), ('ab',), ('up', 4096)],
        'detach_ops': [('det',), ('gs',), ('dn', b'\x46'*64), ('gs',), ('up', 256)],
        'feat_then_dfu': [('sf', 2), ('dn', b'\x47'*64), ('gs',), ('up', 256)],
        'alloc_stress': [
            ('dn', b'\x48'*0x800), ('ab',), ('dn', b'\x49'*0x800), ('ab',),
            ('dn', b'\x4A'*0x800), ('ab',), ('dn', b'\x4B'*0x800), ('ab',),
            ('up', 4096)
        ],
        'size_vary': [
            ('dn', b'\x50'*16), ('ab',), ('dn', b'\x51'*256), ('ab',),
            ('dn', b'\x52'*4096), ('ab',), ('dn', b'\x53'*16), ('ab',), ('up', 4096)
        ],
        'clr_spam': [('cl',), ('cl',), ('cl',), ('cl',), ('cl',), ('gs',), ('up', 256)],
        'slow_uaf': [('dn', b'\x60'*512), ('gs',), ('sleep', 0.5), ('ab',), ('sleep', 0.1), ('up', 4096)],
    }
    
    for name, seq in seqs.items():
        if not run_seq(name, seq):
            return

# ============================================================
# PROBE F: Extended strings
# ============================================================
def probe_strings(dev):
    print("\n" + "="*70)
    print("PROBE F: String descriptors 0-255")
    print("="*70)
    
    for idx in range(256):
        data = safe_in(dev, 0x80, 0x06, 0x0300 | idx, 0x0409, 256, 100)
        if data and len(data) > 2:
            try:
                text = data[2:].decode('utf-16-le', errors='replace')
                log(f"String #{idx}: \"{text}\"", idx > 3)
            except:
                log(f"String #{idx}: {data.hex()}", idx > 3)
    
    for lang in [0x0000, 0xFFFF, 0x8020]:
        for idx in [0, 1, 2, 3, 4, 5, 0xFE, 0xFF]:
            data = safe_in(dev, 0x80, 0x06, 0x0300 | idx, lang, 256, 100)
            if data and len(data) > 4:
                log(f"String #{idx} lang=0x{lang:04X}: {len(data)}B = {data[:16].hex()}", True)

# ============================================================
# MAIN
# ============================================================
def main():
    print("=" * 70)
    print("DFU DEEP PROBE - ROUND 2")
    print("=" * 70)
    
    dev = get_dev()
    serial = usb.util.get_string(dev, dev.iSerialNumber)
    print(f"\nDevice: {dev.idVendor:04X}:{dev.idProduct:04X}")
    print(f"Serial: {serial}")
    state = safe_in(dev, 0xA1, DFU_GETSTATE, 0, 0, 1)
    print(f"State: {state[0] if state else '?'} (2=dfuIDLE)\n")
    
    t0 = time.time()
    
    probe_strings(dev)                              # Quick
    if alive(dev): probe_features_deep(dev)         # SET_FEATURE was promising
    if alive(dev): probe_unusual_types(dev)         # 0x83 and reserved
    if alive(dev): probe_knock_sequences(dev)       # Sequences
    if alive(dev): probe_dnload_careful(dev)        # Careful DNLOAD
    if alive(dev): probe_vendor_deep(dev)           # Big sweep last
    
    elapsed = time.time() - t0
    
    print("\n" + "=" * 70)
    print(f"DEEP PROBE COMPLETE ({elapsed:.1f}s)")
    print("=" * 70)
    
    if findings:
        print(f"\n*** {len(findings)} FINDINGS ***\n")
        for i, f in enumerate(findings, 1):
            print(f"  [{i}] {f}")
    else:
        print("\nNo new findings.")

if __name__ == '__main__':
    main()
