#!/usr/bin/env python3
"""
DFU Hidden Door Prober
======================
Apple engineers need a way to debug their silicon through USB.
A backdoor won't be obvious - it's hidden in the combinatorial space
of USB control transfers that we haven't explored.

Strategy:
1. Non-standard GET_DESCRIPTOR types (not just device/config/string)
2. DFU_DNLOAD with known Apple magic headers (IMG4, IM4P, KBAG, etc.)
3. Vendor requests with EVERY bmRequestType direction/type combo
4. wValue/wIndex parameter sweeps on requests that DO respond
5. SET_FEATURE / SET_INTERFACE with non-standard values
6. Sequence-based probing (specific order of commands)
"""

import usb.core
import usb.util
import struct
import time
import sys

# Apple DFU
VID = 0x05AC
PID = 0x1227

# Standard DFU bRequests
DFU_DETACH    = 0
DFU_DNLOAD    = 1
DFU_UPLOAD    = 2
DFU_GETSTATUS = 3
DFU_CLRSTATUS = 4
DFU_GETSTATE  = 5
DFU_ABORT     = 6

# All 4 bmRequestType combinations for vendor/class
# Device-to-host = 0x80, Host-to-device = 0x00
# Type: Standard=0x00, Class=0x20, Vendor=0x40, Reserved=0x60
# Recipient: Device=0, Interface=1, Endpoint=2, Other=3
BM_REQUEST_TYPES = {
    'vendor_dev_in':   0xC0,  # vendor, device, IN
    'vendor_dev_out':  0x40,  # vendor, device, OUT
    'vendor_iface_in': 0xC1,  # vendor, interface, IN
    'vendor_iface_out':0x41,  # vendor, interface, OUT
    'vendor_ep_in':    0xC2,  # vendor, endpoint, IN
    'vendor_ep_out':   0x42,  # vendor, endpoint, OUT
    'class_dev_in':    0xA0,  # class, device, IN
    'class_dev_out':   0x20,  # class, device, OUT
    'class_iface_in':  0xA1,  # class, interface, IN
    'class_iface_out': 0x21,  # class, interface, OUT
    'std_dev_in':      0x80,  # standard, device, IN
    'std_dev_out':     0x00,  # standard, device, OUT
    'std_iface_in':    0x81,  # standard, interface, IN
    'std_iface_out':   0x01,  # standard, interface, OUT
    'reserved_dev_in': 0xE0,  # reserved, device, IN
    'reserved_dev_out':0x60,  # reserved, device, OUT
}

# Apple known image magic values
APPLE_MAGICS = {
    'IMG4': b'IMG4',                    # IMG4 container
    'IM4P': b'IM4P',                    # IMG4 payload
    'IM4M': b'IM4M',                    # IMG4 manifest
    'IM4R': b'IM4R',                    # IMG4 restore info
    'IMG3': b'Img3',                    # Legacy IMG3
    'iBootMagic': b'iBot',             # iBoot image magic
    'LLB':  b'illb',                    # Low Level Bootloader
    'iBSS': b'ibss',                    # iBoot Single Stage
    'iBEC': b'ibec',                    # iBoot Epoch Change
    'KBAG': b'KBAG',                    # Key Bag
    'SCAB': b'SCAB',                    # Secure CAB?
    'CERT': b'CERT',                    # Certificate
    'SHSH': b'SHSH',                    # SHSH blobs
    'ECID': b'ECID',                    # ECID tag
    'TYPE': b'TYPE',                    # Type descriptor
    'DATA': b'DATA',                    # Data section
    'SEPO': b'SEPO',                    # SEP OS
    'BORD': b'BORD',                    # Board ID
    'CHIP': b'CHIP',                    # Chip ID
    'SDOM': b'SDOM',                    # Security Domain
    'PROD': b'PROD',                    # Production status
    # DER/ASN.1 sequences (IMG4 is DER-encoded)
    'DER_SEQ': bytes([0x30, 0x82]),     # DER SEQUENCE with 2-byte length
    'DER_SEQ2': bytes([0x30, 0x80]),    # DER SEQUENCE indefinite length
    # Mach-O (what if it accepts raw executables?)
    'MachO64': struct.pack('<I', 0xFEEDFACF),  # Mach-O 64-bit
    'MachO32': struct.pack('<I', 0xFEEDFACE),  # Mach-O 32-bit
    'FAT':     struct.pack('>I', 0xCAFEBABE),  # Universal binary
    # ARM exception vectors (raw ARM64 code)
    'ARM64_NOP': struct.pack('<I', 0xD503201F),  # NOP instruction
    'ARM64_BRK': struct.pack('<I', 0xD4200000),  # BRK #0 (debug trap)
    'ARM64_RET': struct.pack('<I', 0xD65F03C0),  # RET
    # Potential debug/unlock magic values (speculative)
    'APPL': b'APPL',
    'DBUG': b'DBUG',
    'DDBG': b'DDBG',
    'SRTG': b'SRTG',                   # SecureROM tag (seen in exploits)
    'PWND': b'PWND',                    # pwned DFU marker
    'CPID': b'CPID',
    'BDID': b'BDID',
    'NONC': b'NONC',
    'SNON': b'SNON',
    'TEST': b'TEST',
    'JTAG': b'JTAG',
    'UART': b'UART',
    'SWD_': b'SWD\x00',
    'NULL16': b'\x00' * 16,
    'FF16':   b'\xFF' * 16,
    'DEAD':  struct.pack('>I', 0xDEADBEEF),
    'CAFE':  struct.pack('>I', 0xCAFECAFE),
}

findings = []

def log(msg, important=False):
    prefix = ">>>" if important else "   "
    print(f"{prefix} {msg}")
    if important:
        findings.append(msg)

def get_dev():
    dev = usb.core.find(idVendor=VID, idProduct=PID)
    if dev is None:
        print("ERROR: No device in DFU mode (05AC:1227)")
        sys.exit(1)
    try:
        dev.set_configuration()
    except:
        pass
    return dev

def safe_ctrl_in(dev, bmReq, bReq, wVal, wIdx, length, timeout=500):
    """Safe control IN transfer, returns data or None"""
    try:
        data = dev.ctrl_transfer(bmReq, bReq, wVal, wIdx, length, timeout)
        if len(data) > 0:
            return bytes(data)
    except usb.core.USBError:
        pass
    return None

def safe_ctrl_out(dev, bmReq, bReq, wVal, wIdx, data=None, timeout=500):
    """Safe control OUT transfer, returns bytes written or None"""
    try:
        if data is None:
            data = b''
        ret = dev.ctrl_transfer(bmReq, bReq, wVal, wIdx, data, timeout)
        return ret
    except usb.core.USBError:
        pass
    return None

def check_device_alive(dev):
    """Quick check if device still responds"""
    try:
        dev.ctrl_transfer(0xA1, DFU_GETSTATE, 0, 0, 1, 500)
        return True
    except:
        return False

# ============================================================
# PROBE 1: Hidden GET_DESCRIPTOR types
# ============================================================
def probe_hidden_descriptors(dev):
    """
    Standard descriptors: 1=Device, 2=Config, 3=String, 4=Interface, 5=Endpoint
    But USB spec defines up to 255. What if Apple responds to non-standard ones?
    GET_DESCRIPTOR = bRequest 0x06, wValue = (type << 8) | index
    """
    print("\n" + "="*70)
    print("PROBE 1: Hidden USB descriptor types (0x00-0xFF)")
    print("="*70)
    
    known = {1:'Device', 2:'Config', 3:'String', 4:'Interface', 5:'Endpoint',
             6:'DeviceQualifier', 7:'OtherSpeedConfig', 8:'InterfacePower',
             9:'OTG', 10:'Debug', 11:'InterfaceAssoc',
             0x21:'HID', 0x22:'Report', 0x29:'Hub', 0x30:'SuperSpeedEP',
             0x31:'SuperSpeedIsoEP', 0x0F:'BOS', 0x10:'DeviceCap'}
    
    for desc_type in range(0x00, 0x100):
        for idx in range(4):  # Try indices 0-3
            wValue = (desc_type << 8) | idx
            data = safe_ctrl_in(dev, 0x80, 0x06, wValue, 0, 256, 200)
            if data:
                name = known.get(desc_type, f"UNKNOWN-0x{desc_type:02X}")
                log(f"DESC type=0x{desc_type:02X} idx={idx} ({name}): {len(data)}B = {data[:32].hex()}", 
                    desc_type not in known or desc_type > 0x10)
                if desc_type not in known:
                    log(f"  *** NON-STANDARD DESCRIPTOR RESPONDS! type=0x{desc_type:02X} ***", True)
    
    # Also try with wIndex = language IDs
    for desc_type in [0x03, 0xFE, 0xFF]:  # String, and high types
        for lang in [0x0409, 0x0000, 0xFFFF, 0x0804]:
            for idx in range(8):
                wValue = (desc_type << 8) | idx
                data = safe_ctrl_in(dev, 0x80, 0x06, wValue, lang, 256, 200)
                if data and desc_type != 0x03:  # We know string descriptors work
                    log(f"DESC type=0x{desc_type:02X} idx={idx} lang=0x{lang:04X}: {len(data)}B", True)

# ============================================================
# PROBE 2: Full bmRequestType × bRequest matrix
# ============================================================
def probe_request_matrix(dev):
    """
    Previous scan only tried a limited set. Now try ALL bmRequestType values
    with all bRequests. The response space is: 256 bmReqTypes × 256 bRequests.
    We focus on types that might be Apple-specific.
    """
    print("\n" + "="*70)
    print("PROBE 2: bmRequestType × bRequest matrix (IN transfers)")
    print("="*70)
    
    # Key bmRequestTypes beyond what we already tested
    interesting_bm = [
        0x80, 0x81, 0x82, 0x83,  # Standard IN: device/iface/ep/other
        0xA0, 0xA1, 0xA2, 0xA3,  # Class IN: device/iface/ep/other
        0xC0, 0xC1, 0xC2, 0xC3,  # Vendor IN: device/iface/ep/other
        0xE0, 0xE1, 0xE2, 0xE3,  # Reserved IN: device/iface/ep/other
    ]
    
    for bm in interesting_bm:
        responded = []
        for bReq in range(256):
            data = safe_ctrl_in(dev, bm, bReq, 0, 0, 256, 100)
            if data:
                responded.append((bReq, data))
        
        if responded:
            bm_name = {0x80:'std_dev', 0x81:'std_iface', 0x82:'std_ep',
                      0xA0:'cls_dev', 0xA1:'cls_iface', 0xA2:'cls_ep',
                      0xC0:'vnd_dev', 0xC1:'vnd_iface', 0xC2:'vnd_ep',
                      0xE0:'rsv_dev', 0xE1:'rsv_iface', 0xE2:'rsv_ep'}.get(bm, f'0x{bm:02X}')
            log(f"bmRequestType 0x{bm:02X} ({bm_name}): {len(responded)} bRequests respond!", True)
            for bReq, data in responded:
                log(f"  bReq=0x{bReq:02X}: {len(data)}B = {data[:24].hex()}")
    
    if not check_device_alive(dev):
        log("DEVICE DIED during request matrix scan!", True)

# ============================================================
# PROBE 3: wValue/wIndex sweep on known-responding requests
# ============================================================
def probe_wvalue_windex(dev):
    """
    DFU requests (GETSTATUS, GETSTATE, UPLOAD) respond with wValue=0, wIndex=0.
    What if different wValue/wIndex changes behavior?  
    """
    print("\n" + "="*70)
    print("PROBE 3: wValue/wIndex sweep on DFU requests")
    print("="*70)
    
    # Baseline
    baseline_status = safe_ctrl_in(dev, 0xA1, DFU_GETSTATUS, 0, 0, 6)
    baseline_state = safe_ctrl_in(dev, 0xA1, DFU_GETSTATE, 0, 0, 1)
    print(f"  Baseline GETSTATUS: {baseline_status.hex() if baseline_status else 'None'}")
    print(f"  Baseline GETSTATE:  {baseline_state.hex() if baseline_state else 'None'}")
    
    # Sweep wValue on GETSTATUS
    print("\n  --- GETSTATUS wValue sweep ---")
    for wVal in [0x0001, 0x0002, 0x0100, 0x0200, 0xFF00, 0xFFFF, 
                 0x8020, 0x1227, 0x05AC, 0xDEAD, 0x4442, 0x5347]:
        data = safe_ctrl_in(dev, 0xA1, DFU_GETSTATUS, wVal, 0, 6)
        if data and data != baseline_status:
            log(f"GETSTATUS wValue=0x{wVal:04X} DIFFERENT: {data.hex()}", True)
    
    # Sweep wIndex on GETSTATUS
    print("  --- GETSTATUS wIndex sweep ---")
    for wIdx in [0x0001, 0x0002, 0x0100, 0x0200, 0xFF00, 0xFFFF,
                 0x8020, 0x1227, 0x05AC]:
        data = safe_ctrl_in(dev, 0xA1, DFU_GETSTATUS, 0, wIdx, 6)
        if data and data != baseline_status:
            log(f"GETSTATUS wIndex=0x{wIdx:04X} DIFFERENT: {data.hex()}", True)
    
    # Sweep wLength on GETSTATUS (normally 6 bytes - what if we ask for more?)
    print("  --- GETSTATUS extended length ---")
    for length in [7, 8, 16, 32, 64, 128, 256, 512, 1024, 4096]:
        data = safe_ctrl_in(dev, 0xA1, DFU_GETSTATUS, 0, 0, length)
        if data and len(data) > 6:
            log(f"GETSTATUS returns {len(data)} bytes (> 6)! Extra: {data[6:].hex()}", True)
    
    # Sweep on UPLOAD (might return ROM data with right params?)
    print("  --- UPLOAD parameter sweep ---")
    for wVal in range(0, 16):
        for wIdx in [0, 1, 2, 0x100]:
            data = safe_ctrl_in(dev, 0xA1, DFU_UPLOAD, wVal, wIdx, 256)
            if data and len(data) > 0:
                log(f"UPLOAD wVal={wVal} wIdx={wIdx}: {len(data)}B = {data[:32].hex()}", True)

# ============================================================
# PROBE 4: DFU_DNLOAD with Apple magic headers
# ============================================================
def probe_dnload_magics(dev):
    """
    The SecureROM parses DNLOAD data. Different magic bytes at the start
    might trigger different code paths. Most will be rejected, but maybe
    one activates something special before the signature check.
    """
    print("\n" + "="*70)
    print("PROBE 4: DFU_DNLOAD with magic headers")
    print("="*70)
    
    for name, magic in APPLE_MAGICS.items():
        # Build a payload: magic + padding
        payload = magic + b'\x00' * (64 - len(magic))
        
        # Reset to clean state
        safe_ctrl_out(dev, 0x21, DFU_ABORT, 0, 0)
        time.sleep(0.01)
        safe_ctrl_in(dev, 0xA1, DFU_GETSTATUS, 0, 0, 6)
        
        # Send DNLOAD with magic
        ret = safe_ctrl_out(dev, 0x21, DFU_DNLOAD, 0, 0, payload)
        
        # Check state transition
        status = safe_ctrl_in(dev, 0xA1, DFU_GETSTATUS, 0, 0, 6)
        state = safe_ctrl_in(dev, 0xA1, DFU_GETSTATE, 0, 0, 1)
        
        if status:
            bStatus = status[0]
            bState = status[4]
            state_names = {0:'appIDLE', 1:'appDETACH', 2:'dfuIDLE', 3:'dfuDNLOAD-SYNC',
                          4:'dfuDNBUSY', 5:'dfuDNLOAD-IDLE', 6:'dfuMANIFEST-SYNC',
                          7:'dfuMANIFEST', 8:'dfuMANIFEST-WT', 9:'dfuUPLOAD-IDLE', 10:'dfuERROR'}
            sname = state_names.get(bState, f'UNKNOWN({bState})')
            
            # Interesting if NOT dfuERROR and NOT dfuDNLOAD-SYNC
            if bState not in [3, 10]:
                log(f"DNLOAD magic '{name}': state={sname} status=0x{bStatus:02X} *** UNEXPECTED ***", True)
            elif bState == 3:  # Normal: went to DNLOAD-SYNC
                # Now check: does GETSTATUS after manifest do anything different?
                status2 = safe_ctrl_in(dev, 0xA1, DFU_GETSTATUS, 0, 0, 6)
                if status2:
                    bState2 = status2[4]
                    pollTimeout = status2[1] | (status2[2] << 8) | (status2[3] << 16)
                    sname2 = state_names.get(bState2, f'UNKNOWN({bState2})')
                    if bState2 not in [4, 10]:  # Not BUSY or ERROR = interesting
                        log(f"DNLOAD magic '{name}': after 2nd GETSTATUS state={sname2} timeout={pollTimeout}ms", True)
                    # If it entered BUSY with a long timeout, the ROM is actually processing it
                    if bState2 == 4 and pollTimeout > 100:
                        log(f"DNLOAD magic '{name}': ROM is PROCESSING (timeout={pollTimeout}ms)! ***", True)
        
        # Abort back to idle
        safe_ctrl_out(dev, 0x21, DFU_ABORT, 0, 0)
        time.sleep(0.01)
        
        if not check_device_alive(dev):
            log(f"DEVICE DIED after DNLOAD with '{name}' magic!", True)
            return

# ============================================================
# PROBE 5: SET_INTERFACE / SET_FEATURE / SET_ADDRESS tricks
# ============================================================
def probe_standard_tricks(dev):
    """
    Unusual standard USB requests that might trigger hidden behavior.
    """
    print("\n" + "="*70)
    print("PROBE 5: Standard USB request tricks")
    print("="*70)
    
    # SET_FEATURE on device (remote wakeup, test mode, etc.)
    print("  --- SET_FEATURE ---")
    for feature in range(0, 32):
        ret = safe_ctrl_out(dev, 0x00, 0x03, feature, 0)
        if ret is not None:
            log(f"SET_FEATURE({feature}) accepted! (ret={ret})", True)
    
    # GET_STATUS on device
    data = safe_ctrl_in(dev, 0x80, 0x00, 0, 0, 2)
    if data:
        log(f"GET_STATUS(device): {data.hex()}")
    
    # GET_STATUS on interface 0
    data = safe_ctrl_in(dev, 0x81, 0x00, 0, 0, 2)
    if data:
        log(f"GET_STATUS(iface 0): {data.hex()}")
    
    # SET_INTERFACE - try alternate settings
    print("  --- SET_INTERFACE alternate settings ---")
    for alt in range(0, 16):
        ret = safe_ctrl_out(dev, 0x01, 0x0B, alt, 0)
        if ret is not None and alt > 0:
            log(f"SET_INTERFACE alt={alt} ACCEPTED! Hidden alternate setting!", True)
    
    # SET_CONFIGURATION - try different configs
    print("  --- SET_CONFIGURATION ---")
    for cfg in range(0, 8):
        try:
            # Read back current config
            data = safe_ctrl_in(dev, 0x80, 0x08, 0, 0, 1)
            if data:
                log(f"  Current config before SET_CONFIG({cfg}): {data[0]}")
        except:
            pass
    
    # SYNCH_FRAME
    data = safe_ctrl_in(dev, 0x82, 0x0C, 0, 0, 2)
    if data:
        log(f"SYNCH_FRAME: {data.hex()}", True)
    
    # GET_CONFIGURATION
    data = safe_ctrl_in(dev, 0x80, 0x08, 0, 0, 1)
    if data:
        log(f"GET_CONFIGURATION: {data[0]}")

# ============================================================
# PROBE 6: Sequence-based knock patterns
# ============================================================
def probe_knock_sequences(dev):
    """
    Maybe a specific SEQUENCE of USB operations unlocks something.
    Try patterns that an engineer might use as a handshake.
    """
    print("\n" + "="*70)
    print("PROBE 6: Knock sequence patterns")
    print("="*70)
    
    sequences = {
        'triple_abort': [
            ('abort',), ('abort',), ('abort',), ('check_state',)
        ],
        'dnload_abort_upload': [
            ('dnload', b'\x00'*64), ('abort',), ('upload', 256), ('check_state',)
        ],
        'clrstatus_spam': [
            ('clrstatus',), ('clrstatus',), ('clrstatus',), ('getstatus',), ('check_state',)
        ],
        'abort_dnload_getstatus_x3': [
            ('abort',), ('dnload', b'AAAA'), ('getstatus',), 
            ('getstatus',), ('getstatus',), ('check_state',)
        ],
        'rapid_reset': [
            ('dnload', b'\x00'*16), ('getstatus',), ('abort',),
            ('dnload', b'\xFF'*16), ('getstatus',), ('abort',),
            ('dnload', b'\x00'*16), ('getstatus',), ('abort',),
            ('check_state',)
        ],
        'fill_then_upload': [
            # Fill buffer, abort (free), then try upload (read freed memory?)
            ('dnload', b'A'*2048), ('getstatus',), ('abort',), 
            ('upload', 4096), ('check_state',)
        ],
        'detach_sequence': [
            ('detach',), ('getstatus',), ('check_state',)
        ],
        'double_dnload': [
            ('dnload', b'DBUG' + b'\x00'*60), ('dnload', b'\x01'*64), 
            ('getstatus',), ('check_state',)
        ],
    }
    
    for name, seq in sequences.items():
        print(f"\n  --- Sequence: {name} ---")
        
        # Reset to clean state first
        safe_ctrl_out(dev, 0x21, DFU_ABORT, 0, 0)
        time.sleep(0.02)
        safe_ctrl_in(dev, 0xA1, DFU_GETSTATUS, 0, 0, 6)
        
        results = []
        for step in seq:
            if step[0] == 'abort':
                safe_ctrl_out(dev, 0x21, DFU_ABORT, 0, 0)
                results.append('abort:ok')
            elif step[0] == 'dnload':
                ret = safe_ctrl_out(dev, 0x21, DFU_DNLOAD, 0, 0, step[1])
                results.append(f'dn:{ret}')
            elif step[0] == 'upload':
                data = safe_ctrl_in(dev, 0xA1, DFU_UPLOAD, 0, 0, step[1])
                if data and len(data) > 0:
                    log(f"Sequence '{name}' UPLOAD returned {len(data)}B: {data[:32].hex()}", True)
                results.append(f'up:{len(data) if data else 0}')
            elif step[0] == 'getstatus':
                data = safe_ctrl_in(dev, 0xA1, DFU_GETSTATUS, 0, 0, 6)
                if data:
                    results.append(f'st:{data[4]}')
                else:
                    results.append('st:fail')
            elif step[0] == 'clrstatus':
                safe_ctrl_out(dev, 0x21, DFU_CLRSTATUS, 0, 0)
                results.append('clr:ok')
            elif step[0] == 'detach':
                safe_ctrl_out(dev, 0x21, DFU_DETACH, 0, 0)
                results.append('det:ok')
            elif step[0] == 'check_state':
                state = safe_ctrl_in(dev, 0xA1, DFU_GETSTATE, 0, 0, 1)
                if state:
                    results.append(f'state:{state[0]}')
                    # Check for unexpected state
                    if state[0] not in [2, 10]:  # Not dfuIDLE or dfuERROR
                        log(f"Sequence '{name}' ended in unexpected state {state[0]}!", True)
                else:
                    results.append('state:DEAD')
                    log(f"Sequence '{name}' KILLED the device!", True)
            time.sleep(0.005)
        
        print(f"    Results: {' → '.join(results)}")
        
        # Clean up
        safe_ctrl_out(dev, 0x21, DFU_ABORT, 0, 0)
        safe_ctrl_out(dev, 0x21, DFU_CLRSTATUS, 0, 0)
        
        if not check_device_alive(dev):
            log(f"Device died after sequence '{name}'!", True)
            return

# ============================================================
# PROBE 7: BOS Descriptor and USB 2.0 Extensions
# ============================================================
def probe_bos_and_extensions(dev):
    """
    BOS (Binary device Object Store) descriptor can reveal hidden capabilities.
    USB 2.0 Extension, SuperSpeed caps, Container ID, etc.
    """
    print("\n" + "="*70)
    print("PROBE 7: BOS descriptor and USB extensions")
    print("="*70)
    
    # BOS descriptor type = 0x0F
    bos = safe_ctrl_in(dev, 0x80, 0x06, 0x0F00, 0, 64)
    if bos:
        log(f"BOS descriptor: {len(bos)}B = {bos.hex()}", True)
        # If BOS exists, request full length
        if len(bos) >= 4:
            total_len = bos[2] | (bos[3] << 8)
            if total_len > len(bos):
                bos_full = safe_ctrl_in(dev, 0x80, 0x06, 0x0F00, 0, total_len)
                if bos_full:
                    log(f"BOS full ({total_len}B): {bos_full.hex()}", True)
    
    # Microsoft OS descriptors (0xEE string, vendor code)
    ms_os = safe_ctrl_in(dev, 0x80, 0x06, 0x03EE, 0, 32)
    if ms_os:
        log(f"MS OS Descriptor (string 0xEE): {ms_os.hex()}", True)
    
    # WebUSB descriptor (vendor request with specific wIndex)
    webusb = safe_ctrl_in(dev, 0xC0, 0x01, 0, 2, 64)
    if webusb:
        log(f"WebUSB-like response: {webusb.hex()}", True)
    
    # USB Debug descriptor (type 0x0A)
    debug_desc = safe_ctrl_in(dev, 0x80, 0x06, 0x0A00, 0, 16)
    if debug_desc:
        log(f"USB Debug Descriptor: {debug_desc.hex()}", True)

# ============================================================
# PROBE 8: Large wValue/wIndex space on vendor requests
# ============================================================
def probe_vendor_space(dev):
    """
    The previous scan tested bRequest 0-255 with wValue=0, wIndex=0.
    But Apple might use wValue/wIndex as a "password" parameter.
    Try common values on vendor requests.
    """
    print("\n" + "="*70)
    print("PROBE 8: Vendor request wValue/wIndex combinations")
    print("="*70)
    
    # Apple product IDs and chip IDs as potential wValue/wIndex
    apple_values = [
        0x8020,  # CPID T8020 (A12)
        0x000C,  # BDID
        0x8015,  # CPID T8015 (A11)
        0x8010,  # CPID T8010 (A10)
        0x7000,  # CPID S7002
        0x1227,  # DFU PID
        0x12A8,  # Normal PID
        0x1281,  # Recovery PID
        0x05AC,  # Apple VID
        0x0003,  # CPFM production
        0xFFFF,
        0x4442,  # 'DB' 
        0x5347,  # 'SG'
        0x4A54,  # 'JT' (JTAG?)
        0x5357,  # 'SW' (SWD?)
        0x5541,  # 'UA' (UART?)
        0xA12B,  # A12 Bionic?
    ]
    
    for bmReq in [0xC0, 0xC1]:  # Vendor IN device and interface
        bm_name = "vendor_dev" if bmReq == 0xC0 else "vendor_iface"
        for bReq in range(256):
            for wVal in apple_values:
                data = safe_ctrl_in(dev, bmReq, bReq, wVal, 0, 64, 100)
                if data:
                    log(f"{bm_name} bReq=0x{bReq:02X} wVal=0x{wVal:04X}: {len(data)}B = {data[:16].hex()}", True)
            
            for wIdx in apple_values:
                data = safe_ctrl_in(dev, bmReq, bReq, 0, wIdx, 64, 100)
                if data:
                    log(f"{bm_name} bReq=0x{bReq:02X} wIdx=0x{wIdx:04X}: {len(data)}B = {data[:16].hex()}", True)
        
        if not check_device_alive(dev):
            log(f"Device died during vendor space scan!", True)
            return

# ============================================================
# MAIN
# ============================================================
def main():
    print("=" * 70)
    print("DFU HIDDEN DOOR PROBER")
    print("A door exists. It's not meant to be obvious.")
    print("=" * 70)
    
    dev = get_dev()
    print(f"\nDevice: {dev.idVendor:04X}:{dev.idProduct:04X}")
    print(f"Serial: {usb.util.get_string(dev, dev.iSerialNumber)}")
    
    state = safe_ctrl_in(dev, 0xA1, DFU_GETSTATE, 0, 0, 1)
    if state:
        print(f"DFU State: {state[0]} (2=dfuIDLE)")
    
    print(f"\nStarting comprehensive probe...")
    t0 = time.time()
    
    probe_hidden_descriptors(dev)
    if check_device_alive(dev):
        probe_request_matrix(dev)
    if check_device_alive(dev):
        probe_wvalue_windex(dev)
    if check_device_alive(dev):
        probe_bos_and_extensions(dev)
    if check_device_alive(dev):
        probe_standard_tricks(dev)
    if check_device_alive(dev):
        probe_dnload_magics(dev)
    if check_device_alive(dev):
        probe_knock_sequences(dev)
    if check_device_alive(dev):
        probe_vendor_space(dev)
    
    elapsed = time.time() - t0
    
    print("\n" + "=" * 70)
    print(f"SCAN COMPLETE ({elapsed:.1f}s)")
    print("=" * 70)
    
    if findings:
        print(f"\n*** {len(findings)} INTERESTING FINDINGS ***\n")
        for i, f in enumerate(findings, 1):
            print(f"  [{i}] {f}")
    else:
        print("\nNo hidden doors found in this scan.")
    
    print(f"\nTotal probes across all categories completed.")

if __name__ == '__main__':
    main()
