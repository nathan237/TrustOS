#!/usr/bin/env python3
"""
T8020 B1 SecureROM — COMPLETE STRUCTURAL MAP
=============================================
Goal: Map EVERY byte of the ROM. What's code? What's data? What's unknown?
For each function: what does it do? Have we analyzed it?

This produces:
1. Memory region map (code vs data vs padding vs unknown)
2. Complete function catalog with classification
3. String table extraction
4. Constant/lookup table identification
5. Cross-reference density map (hot zones vs cold zones)
6. UNEXPLORED REGIONS highlighted
"""

import struct
from collections import defaultdict
from capstone import *
from capstone.arm64 import *

ROM_PATH = "securerom/t8020_B1_securerom.bin"
ROM_BASE = 0x100000000

with open(ROM_PATH, "rb") as f:
    rom = f.read()

ROM_SIZE = len(rom)
md = Cs(CS_ARCH_ARM64, CS_MODE_ARM)
md.detail = True

WIDTH = 120
SEP = "=" * WIDTH

# ============================================================
# PHASE 1: Find ALL functions (STP x29, x30 prologues)
# ============================================================
print(SEP)
print("  T8020 B1 SecureROM — COMPLETE STRUCTURAL MAP")
print(f"  ROM Size: {ROM_SIZE} bytes ({ROM_SIZE//1024} KB)")
print(f"  ROM Base: 0x{ROM_BASE:X}")
print(f"  ROM End:  0x{ROM_BASE + ROM_SIZE:X}")
print(SEP)

print(f"\n{'#' * WIDTH}")
print(f"# PHASE 1: FUNCTION DISCOVERY")
print(f"{'#' * WIDTH}\n")

functions = []
# Pattern: STP x29, x30, [sp, #IMM]! or STP x29, x30, [sp, #IMM]
# Also: SUB sp, sp, #IMM as alternate prologue
for off in range(0, ROM_SIZE - 4, 4):
    insn = struct.unpack('<I', rom[off:off+4])[0]
    # STP x29, x30, [sp, #imm]! (pre-index) = 0xA9xx7BFD
    # STP x29, x30, [sp, #imm] (signed offset) = 0xA9xx7BFD but different encoding
    # More precisely: STP Xt1, Xt2, [Xn, #imm]! where t1=29, t2=30, n=31
    # Pre-index: 1x101001 1 xxxxxxx xxxxx xxxxx xxxxx
    # 0xA9BF7BFD = stp x29, x30, [sp, #-0x10]!
    
    # Check for STP with x29, x30 and sp
    if (insn & 0xFFC003E0) == 0xA90003E0:  # STP signed offset base=sp
        rt = insn & 0x1F
        rt2 = (insn >> 10) & 0x1F
        if rt == 29 and rt2 == 30:
            functions.append(ROM_BASE + off)
    elif (insn & 0xFFC003E0) == 0xA98003E0:  # STP pre-index base=sp
        rt = insn & 0x1F
        rt2 = (insn >> 10) & 0x1F
        if rt == 29 and rt2 == 30:
            functions.append(ROM_BASE + off)

# Also find SUB sp, sp, #imm as standalone function prologues
# These might be leaf functions or functions that save fp/lr differently
sub_sp_prologues = []
for off in range(0, ROM_SIZE - 4, 4):
    insn = struct.unpack('<I', rom[off:off+4])[0]
    # SUB sp, sp, #imm = 0xD10003FF with varying imm
    if (insn & 0xFF0003FF) == 0xD10003FF:
        addr = ROM_BASE + off
        if addr not in functions:
            # Check if previous instruction is NOT part of another function
            sub_sp_prologues.append(addr)

print(f"  Functions found (STP x29, x30): {len(functions)}")
print(f"  Additional SUB sp prologues: {len(sub_sp_prologues)}")

# Sort functions
functions.sort()

# ============================================================
# PHASE 2: Estimate function boundaries
# ============================================================
print(f"\n{'#' * WIDTH}")
print(f"# PHASE 2: FUNCTION BOUNDARIES & SIZE ESTIMATION")
print(f"{'#' * WIDTH}\n")

func_info = []
for i, func_addr in enumerate(functions):
    # End = start of next function or next RET, whichever comes first
    if i + 1 < len(functions):
        next_func = functions[i + 1]
    else:
        next_func = ROM_BASE + ROM_SIZE
    
    # Find RET within this function's range
    off = func_addr - ROM_BASE
    end_off = next_func - ROM_BASE
    
    func_size = 0
    has_ret = False
    num_calls = 0
    num_branches = 0
    
    for pos in range(off, min(end_off, off + 4000), 4):
        if pos + 4 > ROM_SIZE:
            break
        insn_bytes = struct.unpack('<I', rom[pos:pos+4])[0]
        func_size += 4
        
        # RET = 0xD65F03C0
        if insn_bytes == 0xD65F03C0:
            has_ret = True
        
        # BL = 0x94xxxxxx
        if (insn_bytes & 0xFC000000) == 0x94000000:
            num_calls += 1
        
        # B.cond and CBZ/CBNZ/TBZ/TBNZ
        if (insn_bytes & 0xFF000000) in (0x54000000, 0x34000000, 0x35000000, 0x36000000, 0x37000000, 0xB4000000, 0xB5000000):
            num_branches += 1
    
    estimated_size = min(func_size, (next_func - func_addr))
    func_info.append({
        'addr': func_addr,
        'size': estimated_size,
        'has_ret': has_ret,
        'calls': num_calls,
        'branches': num_branches,
    })

# ============================================================
# PHASE 3: Classify each function
# ============================================================
print(f"\n{'#' * WIDTH}")
print(f"# PHASE 3: FUNCTION CLASSIFICATION")
print(f"{'#' * WIDTH}\n")

# Known functions from our analysis
KNOWN_FUNCTIONS = {
    0x1000017A4: ("boot_main", "ANALYZED - Boot entry point, 333 instrs, 85 BL calls"),
    0x100001BC8: ("signature_gate", "ANALYZED - cbz w8 at 0x1BC8, signature verification branch"),
    0x100002994: ("usb_init", "ANALYZED - USB controller initialization, 91 instrs, no barriers"),
    0x100002A80: ("usb_alloc_with_canary", "ANALYZED - USB allocation with stack canary"),
    0x100002B04: ("usb_reset", "ANALYZED - USB controller reset, 42 instrs"),
    0x100002BB0: ("usb_detect", "ANALYZED - USB detection, 38 instrs"),
    0x100005480: ("img4_verify_internal", "DEEP TRACED - Real signature verifier, 600 instrs, 46 sub-calls"),
    0x100006754: ("log_event", "IDENTIFIED - Event logging function"),
    0x100006774: ("log_event_with_params", "IDENTIFIED - Parameterized event logging"),
    0x100006CDC: ("new_func_1_mmio_config", "ANALYZED - MMIO hardware register setup at 0x3D2D0000"),
    0x100006D4C: ("config_helper", "IDENTIFIED - Configuration helper"),
    0x100006D64: ("new_func_2_sram_base", "ANALYZED - SRAM base setup, stores to [0x19c00b+0xd58]"),
    0x100006D80: ("addr_validator", "DEEP ANALYZED - Address range validator 0x19C00-0x19C03, all 64-bit"),
    0x100006EE0: ("get_platform_info", "IDENTIFIED - Returns platform info struct"),
    0x100007008: ("mode_switch", "IDENTIFIED - Mode switching function"),
    0x100007108: ("get_chip_info", "IDENTIFIED - Returns chip info"),
    0x100007310: ("verify_result", "IDENTIFIED - 32-bit result verifier"),
    0x1000073AC: ("check_property", "IDENTIFIED - Property checker"),
    0x100007464: ("validate_header", "IDENTIFIED - Header validation"),
    0x1000075CC: ("get_boot_mode", "IDENTIFIED - Boot mode query"),
    0x100008370: ("gpio_config", "IDENTIFIED - GPIO configuration"),
    0x100008978: ("panic_handler", "ANALYZED - Panic/abort, called 136 times"),
    0x100008B58: ("logging_func", "IDENTIFIED - Logging infrastructure, called 69 times"),
    0x100008C94: ("security_config", "ANALYZED - Security configuration with ADDR_VALIDATOR call"),
    0x10000955C: ("new_func_4_heap_state", "ANALYZED - Heap/state management"),
    0x100009684: ("new_func_5_counter", "ANALYZED - Atomic counter increment"),
    0x100009BA8: ("delay_microseconds", "IDENTIFIED - Microsecond delay function"),
    0x100009BE8: ("hash_verify", "IDENTIFIED - Hash verification"),
    0x10000A704: ("img4_verify", "DEEP TRACED - img4 verification wrapper, 89 instrs"),
    0x10000A890: ("img4_create_manifest", "ANALYZED - Creates IMG4 manifest, calls heap alloc"),
    0x10000A8C0: ("img4_cleanup", "IDENTIFIED - IMG4 cleanup"),
    0x10000AA60: ("list_append", "IDENTIFIED - Linked list append"),
    0x10000AAF4: ("list_init", "IDENTIFIED - Linked list init"),
    0x10000AD7C: ("lock_release", "IDENTIFIED - Lock/mutex release"),
    0x10000AF24: ("session_create", "IDENTIFIED - Session/context creation"),
    0x10000BD0: ("memcpy_like", "IDENTIFIED - memcpy equivalent, called 76 times"),
    0x10000D540: ("string_alloc", "IDENTIFIED - String allocation from heap"),
    0x10000D6D4: ("buffer_alloc_pair", "IDENTIFIED - Allocates buffer pair, has SXTW!"),
    0x10000F1EC: ("heap_alloc", "ANALYZED - Heap allocator, 14 callers, returns x0 pointer"),
    0x10000F2E8: ("heap_get_pool", "IDENTIFIED - Gets heap pool descriptor"),
    0x10000F324: ("heap_calc_size", "IDENTIFIED - Calculates allocation size"),
    0x10000F35C: ("heap_link_block", "IDENTIFIED - Links block in free list"),
    0x10000F3E4: ("heap_validate", "IDENTIFIED - Validates heap block"),
    0x10000F468: ("heap_free_or_release", "IDENTIFIED - Heap free/release path"),
    0x10000FDB4: ("heap_split_block", "IDENTIFIED - Splits heap block"),
    0x10000FDEC: ("heap_get_bucket", "IDENTIFIED - Gets allocation bucket index"),
    0x10000FE34: ("heap_update_stats", "IDENTIFIED - Updates heap statistics"),
    0x10000FEBC: ("heap_coalesce", "IDENTIFIED - Coalesces free blocks"),
    0x10000FFE8: ("heap_alloc_large", "IDENTIFIED - Large allocation path"),
    0x100010050: ("heap_free", "IDENTIFIED - Heap free"),
    0x100010BD0: ("memcpy", "IDENTIFIED - Memory copy, 76 callers"),
    0x100010D80: ("memset_or_bzero", "IDENTIFIED - Memory clear"),
    0x100010E00: ("memset", "IDENTIFIED - memset implementation"),
    0x100010EA4: ("strlen_like", "IDENTIFIED - String length"),
    0x100011040: ("memmove_or_compare", "IDENTIFIED - Memory move/compare"),
    0x1000113B4: ("alloc_typed", "IDENTIFIED - Typed allocation"),
    0x1000115C4: ("buffer_setup", "IDENTIFIED - Buffer setup"),
    0x1000115F8: ("buffer_init", "IDENTIFIED - Buffer initialization"),
    0x100011760: ("table_count", "IDENTIFIED - Table entry counter"),
    0x1000117E8: ("table_alloc", "IDENTIFIED - Table allocation"),
    0x1000118D0: ("table_grow", "IDENTIFIED - Table resize/grow"),
    0x1000119CC: ("stack_clear", "IDENTIFIED - Stack buffer zeroing, called 5x from boot"),
    0x100011C70: ("lock_acquire", "IDENTIFIED - Lock/mutex acquire"),
    0x100011CBC: ("lock_release2", "IDENTIFIED - Lock/mutex release variant"),
    0x100011ED0: ("soc_get_prop_a", "IDENTIFIED - SoC property getter A"),
    0x100011ED4: ("soc_get_prop_b", "IDENTIFIED - SoC property getter B"),
    0x100011ED8: ("soc_get_prop_c", "IDENTIFIED - SoC property getter C"),
    0x100011EFC: ("soc_get_chipid", "IDENTIFIED - Gets chip ID, returns 0x5ac"),
    0x100011F00: ("soc_get_boardid", "IDENTIFIED - Gets board ID"),
    0x100011F0C: ("soc_get_ecid", "IDENTIFIED - Gets ECID"),
    0x100012558: ("asn1_parse_seq", "IDENTIFIED - ASN.1 sequence parser"),
    0x100012590: ("asn1_parse_tag", "IDENTIFIED - ASN.1 tag parser"),
    0x1000125C0: ("asn1_parse_int", "IDENTIFIED - ASN.1 integer parser"),
    0x1000125F8: ("asn1_parse_octet", "IDENTIFIED - ASN.1 octet string parser"),
    0x100012630: ("asn1_parse_set", "IDENTIFIED - ASN.1 SET parser"),
    0x1000127D0: ("asn1_validate", "IDENTIFIED - ASN.1 validation"),
    0x1000127F8: ("asn1_deep_parse", "IDENTIFIED - ASN.1 deep/recursive parser"),
    0x100012E9C: ("img4_parse_manifest", "IDENTIFIED - IMG4 manifest parser"),
    0x100012F58: ("img4_parse_payload", "IDENTIFIED - IMG4 payload parser"),
    0x100013AA4: ("der_check_type", "IDENTIFIED - DER type checking"),
    0x100013C50: ("cert_verify", "IDENTIFIED - Certificate verification"),
    0x100013D94: ("der_decode_value", "IDENTIFIED - DER value decoder"),
    0x100013DB0: ("der_get_contents", "IDENTIFIED - DER contents accessor"),
    0x100017A9C: ("bignum_or_ec", "IDENTIFIED - Bignum/EC math operation"),
    0x10001AC78: ("sha512_neon", "ANALYZED - SHA-512 NEON implementation, 300 instrs"),
    0x10001C094: ("entropy_source", "IDENTIFIED - Entropy/random source"),
}

# Cross-reference map: count how many times each function is called
call_xrefs = defaultdict(int)
for off in range(0, ROM_SIZE - 4, 4):
    insn_bytes = struct.unpack('<I', rom[off:off+4])[0]
    if (insn_bytes & 0xFC000000) == 0x94000000:
        imm26 = insn_bytes & 0x03FFFFFF
        if imm26 & 0x02000000:
            imm26 = struct.unpack('<i', struct.pack('<I', imm26 | 0xFC000000))[0]
        target = (ROM_BASE + off) + (imm26 * 4)
        call_xrefs[target] += 1

# Now classify ALL functions
ANALYSIS_STATUS = {
    "DEEP TRACED": [],    # Fully RE'd with return path analysis
    "ANALYZED": [],       # Disassembled and understood
    "IDENTIFIED": [],     # Name/purpose known but not deeply analyzed
    "UNKNOWN": [],        # Never looked at
}

for fi in func_info:
    addr = fi['addr']
    if addr in KNOWN_FUNCTIONS:
        name, status_str = KNOWN_FUNCTIONS[addr]
        if "DEEP" in status_str:
            ANALYSIS_STATUS["DEEP TRACED"].append((addr, name, fi))
        elif "ANALYZED" in status_str:
            ANALYSIS_STATUS["ANALYZED"].append((addr, name, fi))
        else:
            ANALYSIS_STATUS["IDENTIFIED"].append((addr, name, fi))
    else:
        ANALYSIS_STATUS["UNKNOWN"].append((addr, "???", fi))

for status, items in ANALYSIS_STATUS.items():
    print(f"  {status}: {len(items)} functions")

# ============================================================
# PHASE 4: UNKNOWN FUNCTIONS — THE GAPS
# ============================================================
print(f"\n{'#' * WIDTH}")
print(f"# PHASE 4: UNEXPLORED FUNCTIONS (sorted by xref count — most called first)")
print(f"# These are functions we have NEVER analyzed")
print(f"{'#' * WIDTH}\n")

unknown_sorted = sorted(ANALYSIS_STATUS["UNKNOWN"], 
                       key=lambda x: call_xrefs.get(x[0], 0), reverse=True)

print(f"  {'Address':>14s}  {'XRefs':>5s}  {'Size':>6s}  {'Calls':>5s}  {'Branches':>8s}  {'HasRET':>6s}  Region")
print(f"  {'-'*14}  {'-'*5}  {'-'*6}  {'-'*5}  {'-'*8}  {'-'*6}  {'-'*30}")

for addr, name, fi in unknown_sorted:
    xrefs = call_xrefs.get(addr, 0)
    offset = addr - ROM_BASE
    
    # Classify region
    if offset < 0x1800:
        region = "EARLY BOOT / RESET VECTOR"
    elif offset < 0x2900:
        region = "BOOT FLOW / INIT"
    elif offset < 0x3000:
        region = "USB CONTROLLER"
    elif offset < 0x5000:
        region = "DFU HANDLER (?)"
    elif offset < 0x6000:
        region = "IMG4 / SIGNATURE"
    elif offset < 0x7000:
        region = "PLATFORM CONFIG"
    elif offset < 0x8000:
        region = "VALIDATION / CHECKS"
    elif offset < 0x9000:
        region = "PANIC / LOGGING"
    elif offset < 0xA000:
        region = "SECURITY SERVICES"
    elif offset < 0xB000:
        region = "IMG4 VERIFY"
    elif offset < 0xD000:
        region = "IO / TRANSPORT"
    elif offset < 0xF000:
        region = "DER / ASN.1"
    elif offset < 0x11000:
        region = "HEAP / MEMORY"
    elif offset < 0x12000:
        region = "SYNC / SOC"
    elif offset < 0x15000:
        region = "ASN.1 / CERT / DER"
    elif offset < 0x18000:
        region = "CRYPTO (EC/RSA)"
    elif offset < 0x1B000:
        region = "CRYPTO (AES/HASH)"
    elif offset < 0x1D000:
        region = "CRYPTO (SHA/NEON)"
    elif offset < 0x1E000:
        region = "STRINGS / CONSTANTS"
    else:
        region = "DATA / PADDING"
    
    print(f"  0x{addr:012X}  {xrefs:5d}  {fi['size']:5d}B  {fi['calls']:5d}  {fi['branches']:8d}  {'Yes' if fi['has_ret'] else 'No':>6s}  {region}")

# ============================================================
# PHASE 5: Memory region map
# ============================================================
print(f"\n{'#' * WIDTH}")
print(f"# PHASE 5: MEMORY REGION MAP")
print(f"{'#' * WIDTH}\n")

# Classify each 4KB page
PAGE_SIZE = 0x1000
for page_start in range(0, ROM_SIZE, PAGE_SIZE):
    page_end = min(page_start + PAGE_SIZE, ROM_SIZE)
    page_addr = ROM_BASE + page_start
    
    # Count functions in this page
    funcs_in_page = [f for f in functions if page_start <= (f - ROM_BASE) < page_end]
    
    # Check for string data
    string_count = 0
    ascii_bytes = 0
    zero_bytes = 0
    nop_count = 0
    
    for b in range(page_start, page_end):
        byte = rom[b]
        if 0x20 <= byte <= 0x7E:
            ascii_bytes += 1
        if byte == 0:
            zero_bytes += 1
    
    # Count NOPs (0xD503201F)
    for off in range(page_start, page_end - 3, 4):
        if struct.unpack('<I', rom[off:off+4])[0] == 0xD503201F:
            nop_count += 1
    
    # Determine type
    ascii_pct = ascii_bytes * 100 // PAGE_SIZE
    zero_pct = zero_bytes * 100 // PAGE_SIZE
    
    if zero_pct > 95:
        page_type = "ZERO/PADDING"
    elif ascii_pct > 60:
        page_type = "STRING DATA"
    elif len(funcs_in_page) > 0:
        page_type = f"CODE ({len(funcs_in_page)} funcs)"
    elif nop_count > 100:
        page_type = "NOP SLED/PADDING"
    else:
        page_type = "DATA/CONSTANTS"
    
    # How many known vs unknown funcs
    known_count = sum(1 for f in funcs_in_page if f in KNOWN_FUNCTIONS)
    unknown_count = len(funcs_in_page) - known_count
    
    coverage = ""
    if len(funcs_in_page) > 0:
        if unknown_count == 0:
            coverage = " [FULLY MAPPED]"
        elif known_count == 0:
            coverage = " [*** UNEXPLORED ***]"
        else:
            coverage = f" [{known_count} known, {unknown_count} UNKNOWN]"
    
    print(f"  0x{page_addr:012X} - 0x{page_addr + PAGE_SIZE - 1:012X}: {page_type:25s}{coverage}")

# ============================================================
# PHASE 6: String extraction
# ============================================================
print(f"\n{'#' * WIDTH}")
print(f"# PHASE 6: ALL STRINGS IN ROM")
print(f"{'#' * WIDTH}\n")

strings = []
current_str = b""
str_start = 0
for i in range(ROM_SIZE):
    b = rom[i]
    if 0x20 <= b <= 0x7E:
        if not current_str:
            str_start = i
        current_str += bytes([b])
    else:
        if len(current_str) >= 4:  # Min length 4
            strings.append((ROM_BASE + str_start, current_str.decode('ascii', errors='replace')))
        current_str = b""

print(f"  Total strings found (>= 4 chars): {len(strings)}")
print(f"\n  Security-relevant strings:")
security_keywords = ['verify', 'sign', 'hash', 'cert', 'key', 'auth', 'valid', 'fail', 
                     'error', 'panic', 'boot', 'dfu', 'usb', 'img4', 'nonce', 'ecid',
                     'sep', 'tz', 'trust', 'secure', 'decrypt', 'encrypt', 'aes', 
                     'sha', 'rsa', 'ec', 'ecdsa', 'x509', 'der', 'asn', 'token',
                     'ap', 'ticket', 'manifest', 'payload', 'load', 'exec', 'jump']
for addr, s in strings:
    s_lower = s.lower()
    for kw in security_keywords:
        if kw in s_lower:
            region_off = addr - ROM_BASE
            print(f"    0x{addr:X} (offset 0x{region_off:05X}): \"{s}\"")
            break

# ============================================================
# PHASE 7: DATA TABLES / CONSTANTS
# ============================================================
print(f"\n{'#' * WIDTH}")
print(f"# PHASE 7: POTENTIAL DISPATCH/JUMP TABLES")
print(f"{'#' * WIDTH}\n")

# Look for sequences of pointers that look like function tables
for off in range(0, ROM_SIZE - 32, 8):
    # Check if 4 consecutive 8-byte values look like ROM pointers
    ptrs = []
    valid = True
    for j in range(4):
        val = struct.unpack('<Q', rom[off + j*8:off + j*8 + 8])[0]
        if ROM_BASE <= val < ROM_BASE + ROM_SIZE:
            ptrs.append(val)
        else:
            valid = False
            break
    if valid and len(ptrs) == 4:
        # Verify they point to plausible function starts
        all_funcs = all(p in functions for p in ptrs)
        print(f"  0x{ROM_BASE + off:X}: Pointer table -> {', '.join(f'0x{p:X}' for p in ptrs)}{' [ALL FUNCTIONS]' if all_funcs else ''}")

# ============================================================
# PHASE 8: CROSS-REFERENCE DENSITY (hot zones)
# ============================================================
print(f"\n{'#' * WIDTH}")
print(f"# PHASE 8: CROSS-REFERENCE DENSITY BY REGION")
print(f"# High xref = heavily used code. Low xref = specialized/rare code paths")
print(f"{'#' * WIDTH}\n")

# Aggregate xrefs by 4KB page
page_xrefs = defaultdict(int)
for target, count in call_xrefs.items():
    page = (target - ROM_BASE) // PAGE_SIZE
    page_xrefs[page] += count

for page_idx in sorted(page_xrefs.keys()):
    bar_len = min(page_xrefs[page_idx], 80)
    bar = "|" * bar_len
    print(f"  0x{ROM_BASE + page_idx * PAGE_SIZE:012X}: {page_xrefs[page_idx]:4d} xrefs {bar}")

# ============================================================
# PHASE 9: INTERRUPT/EXCEPTION VECTORS
# ============================================================
print(f"\n{'#' * WIDTH}")
print(f"# PHASE 9: EXCEPTION VECTOR TABLE (VBAR_EL3)")
print(f"{'#' * WIDTH}\n")

# ARM64 exception vector table: 16 entries, each 0x80 bytes apart
# Typically at beginning of ROM or referenced by MSR VBAR_EL3
# Search for MSR VBAR_ELx
for off in range(0, ROM_SIZE - 4, 4):
    insn_bytes = struct.unpack('<I', rom[off:off+4])[0]
    # MSR VBAR_EL1 = 0xD518C000 | Rt
    # MSR VBAR_EL3 = 0xD51EC000 | Rt  
    if (insn_bytes & 0xFFFFFFE0) in (0xD518C000, 0xD51EC000):
        el = "EL1" if (insn_bytes & 0xFFFFFFE0) == 0xD518C000 else "EL3"
        rt = insn_bytes & 0x1F
        print(f"  MSR VBAR_{el}, x{rt} at 0x{ROM_BASE + off:X}")
        # Try to find what x{rt} was set to
        # Look backwards for ADRP/ADR to this register
        for back in range(off - 4, max(off - 40, 0), -4):
            back_insn = struct.unpack('<I', rom[back:back+4])[0]
            # ADRP
            if (back_insn & 0x9F000000) == 0x90000000:
                rd = back_insn & 0x1F
                if rd == rt:
                    immhi = (back_insn >> 5) & 0x7FFFF
                    immlo = (back_insn >> 29) & 0x3
                    imm = (immhi << 2) | immlo
                    if imm & 0x100000:
                        imm |= ~0x1FFFFF
                        imm = imm & 0xFFFFFFFF
                        imm = struct.unpack('<i', struct.pack('<I', imm))[0]
                    page = ((ROM_BASE + back) & ~0xFFF) + (imm << 12)
                    print(f"    ADRP x{rt}, #0x{page:X} at 0x{ROM_BASE + back:X}")
                    print(f"    => VBAR points to 0x{page:X}")
                    
                    # Dump the vector table
                    vbar_off = page - ROM_BASE
                    if 0 <= vbar_off < ROM_SIZE - 0x800:
                        labels = [
                            "Sync EL1t/EL3t", "IRQ EL1t/EL3t", "FIQ EL1t/EL3t", "SError EL1t/EL3t",
                            "Sync EL1h/EL3h", "IRQ EL1h/EL3h", "FIQ EL1h/EL3h", "SError EL1h/EL3h",
                            "Sync Lower64", "IRQ Lower64", "FIQ Lower64", "SError Lower64",
                            "Sync Lower32", "IRQ Lower32", "FIQ Lower32", "SError Lower32",
                        ]
                        for vi in range(16):
                            vec_off = vbar_off + vi * 0x80
                            first_insn = struct.unpack('<I', rom[vec_off:vec_off+4])[0]
                            # Disassemble first instruction
                            dis = list(md.disasm(rom[vec_off:vec_off+8], page + vi * 0x80, 2))
                            if dis:
                                print(f"    [{vi:2d}] 0x{page + vi * 0x80:X} ({labels[vi]:16s}): {dis[0].mnemonic} {dis[0].op_str}")
                    break

# ============================================================
# PHASE 10: SYSTEM REGISTER ACCESS MAP
# ============================================================
print(f"\n{'#' * WIDTH}")
print(f"# PHASE 10: ALL SYSTEM REGISTER ACCESSES")
print(f"{'#' * WIDTH}\n")

sysreg_reads = []
sysreg_writes = []
for off in range(0, ROM_SIZE - 4, 4):
    insn_bytes = struct.unpack('<I', rom[off:off+4])[0]
    # MRS = 0xD5300000 | sys_reg | Rt
    # MSR = 0xD5100000 | sys_reg | Rt
    if (insn_bytes & 0xFFF00000) == 0xD5300000:
        sysreg_reads.append((ROM_BASE + off, insn_bytes))
    elif (insn_bytes & 0xFFF00000) == 0xD5100000:
        sysreg_writes.append((ROM_BASE + off, insn_bytes))

print(f"  MRS (system register reads): {len(sysreg_reads)}")
print(f"  MSR (system register writes): {len(sysreg_writes)}")

# Decode a few key ones
def decode_sysreg(insn):
    op0 = ((insn >> 19) & 0x1) + 2
    op1 = (insn >> 16) & 0x7
    crn = (insn >> 12) & 0xF
    crm = (insn >> 8) & 0xF
    op2 = (insn >> 5) & 0x7
    rt = insn & 0x1F
    
    # Common system registers
    key = (op0, op1, crn, crm, op2)
    names = {
        (3, 0, 1, 0, 0): "SCTLR_EL1",
        (3, 6, 1, 0, 0): "SCTLR_EL3",
        (3, 0, 12, 0, 0): "VBAR_EL1",
        (3, 6, 12, 0, 0): "VBAR_EL3",
        (3, 0, 4, 0, 0): "SPSR_EL1",
        (3, 6, 4, 0, 0): "SPSR_EL3",
        (3, 0, 4, 0, 1): "ELR_EL1",
        (3, 6, 4, 0, 1): "ELR_EL3",
        (3, 6, 1, 1, 0): "SCR_EL3",
        (3, 0, 2, 0, 0): "TTBR0_EL1",
        (3, 0, 2, 0, 2): "TCR_EL1",
        (3, 0, 5, 2, 0): "ESR_EL1",
        (3, 6, 5, 2, 0): "ESR_EL3",
        (3, 0, 0, 0, 0): "MIDR_EL1",
        (3, 0, 0, 0, 5): "MPIDR_EL1",
        (3, 3, 13, 0, 2): "TPIDR_EL0",
        (3, 3, 14, 0, 1): "CNTPCT_EL0",
        (3, 0, 2, 1, 0): "APIAKeyLo_EL1",
        (3, 0, 2, 1, 1): "APIAKeyHi_EL1",
        (3, 0, 2, 2, 0): "APIBKeyLo_EL1",
        (3, 0, 2, 2, 1): "APIBKeyHi_EL1",
        (3, 0, 2, 3, 0): "APDAKeyLo_EL1",
        (3, 0, 2, 3, 1): "APDAKeyHi_EL1",
        (3, 0, 2, 2, 2): "APGAKeyLo_EL1",
        (3, 0, 2, 2, 3): "APGAKeyHi_EL1",
    }
    name = names.get(key, f"S{op0}_{op1}_C{crn}_C{crm}_{op2}")
    return name, rt

print(f"\n  System register WRITES (MSR):")
msr_by_reg = defaultdict(list)
for addr, insn in sysreg_writes:
    name, rt = decode_sysreg(insn)
    msr_by_reg[name].append(addr)
for name in sorted(msr_by_reg.keys()):
    addrs = msr_by_reg[name]
    addr_str = ", ".join(f"0x{a:X}" for a in addrs[:5])
    if len(addrs) > 5:
        addr_str += f" (+{len(addrs)-5} more)"
    print(f"    {name:30s}: {len(addrs):2d}x at {addr_str}")

print(f"\n  System register READS (MRS):")
mrs_by_reg = defaultdict(list)
for addr, insn in sysreg_reads:
    name, rt = decode_sysreg(insn)
    mrs_by_reg[name].append(addr)
for name in sorted(mrs_by_reg.keys()):
    addrs = mrs_by_reg[name]
    addr_str = ", ".join(f"0x{a:X}" for a in addrs[:5])
    if len(addrs) > 5:
        addr_str += f" (+{len(addrs)-5} more)"
    print(f"    {name:30s}: {len(addrs):2d}x at {addr_str}")

# ============================================================
# PHASE 11: SUMMARY — What we know vs what we don't
# ============================================================
print(f"\n{'#' * WIDTH}")
print(f"# PHASE 11: COVERAGE SUMMARY — WHAT WE EXPLORED vs WHAT REMAINS")
print(f"{'#' * WIDTH}\n")

total_funcs = len(functions)
known = len(ANALYSIS_STATUS["DEEP TRACED"]) + len(ANALYSIS_STATUS["ANALYZED"]) + len(ANALYSIS_STATUS["IDENTIFIED"])
unknown = len(ANALYSIS_STATUS["UNKNOWN"])

print(f"  Total functions in ROM: {total_funcs}")
print(f"  Deep-traced (full RE):  {len(ANALYSIS_STATUS['DEEP TRACED'])}")
print(f"  Analyzed (understood):  {len(ANALYSIS_STATUS['ANALYZED'])}")
print(f"  Identified (named):    {len(ANALYSIS_STATUS['IDENTIFIED'])}")
print(f"  UNKNOWN (never looked): {unknown}")
print(f"  Coverage: {known}/{total_funcs} = {known*100//total_funcs}%")
print()

# Highlight the most important unknowns
print(f"  TOP 20 MOST-CALLED UNKNOWN FUNCTIONS:")
top_unknown = unknown_sorted[:20]
for addr, name, fi in top_unknown:
    xrefs = call_xrefs.get(addr, 0)
    off = addr - ROM_BASE
    print(f"    0x{addr:X} (off 0x{off:05X}): {xrefs} calls, {fi['size']}B, {fi['calls']} sub-calls, {fi['branches']} branches")

# Count code bytes vs data bytes vs explored
code_bytes = sum(fi['size'] for fi in func_info)
known_code_bytes = sum(fi['size'] for fi in func_info if fi['addr'] in KNOWN_FUNCTIONS)
print(f"\n  Total code bytes (in functions): {code_bytes} / {ROM_SIZE} = {code_bytes*100//ROM_SIZE}%")
print(f"  Known code bytes:                {known_code_bytes} / {code_bytes} = {known_code_bytes*100//max(code_bytes,1)}%")
print(f"  UNEXPLORED code bytes:           {code_bytes - known_code_bytes}")

print(f"\n{SEP}")
print(f"  STRUCTURAL MAP COMPLETE")
print(f"{SEP}")
