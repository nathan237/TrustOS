#!/usr/bin/env python3
"""
Chain B Phase 6: Virtual Dispatch Resolver + kernel_call Primitive Design
=========================================================================
Traces the multiplication core's virtual dispatch chain:
  max_check_compute (0xfffffff00a1d0384)
    -> BLRAA vtable+0x138 (div 0x2e3b) = actual multiply method
    -> BL 0xfffffff00861ed6c = kernel helper
    -> BLRAA vtable+0xb0 (div 0xbffa) = result store method
    
Resolves vtable entries to actual function addresses, disassembles them
to find the MUL/UMULL instruction, then designs the kernel_call primitive.
"""

import struct, json, os, sys
from capstone import Cs, CS_ARCH_ARM64, CS_MODE_ARM

# ─── Constants ───────────────────────────────────────────────────────────────
KC_PATH = os.path.join(os.path.dirname(__file__), "extracted", "kernelcache_iPhone12,3_26_3.raw")
KC_BASE = 0xfffffff007004000

# Known addresses from Phase 2-5
IOSURFACE_VTABLE   = 0xfffffff007f21fa0  # IOSurface vtable base
VTABLE_METHODS     = 0xfffffff007f21fb0  # methods start at +0x10
MAX_CHECK_COMPUTE  = 0xfffffff00a1d0384  # virtual dispatch wrapper
KERNEL_HELPER      = 0xfffffff00861ed6c  # BL target from compute
ML_PHYS_READ       = 0xfffffff00814f740
ML_PHYS_WRITE      = 0xfffffff00814f9f0
DISPATCH_TABLE     = 0xfffffff007f238e8
S_CREATE_SURFACE   = 0xfffffff00a1eba5c
IOSURFACE_MAX_CHECK = 0xfffffff00a1d02d0
IOSURFACE_ALLOCATE = 0xfffffff00a1cece8

# IOSurface kext TEXT_EXEC bounds
KEXT_TEXT_START = 0xfffffff00a1c5c80
KEXT_TEXT_END   = 0xfffffff00a1f75dc

# Vtable offsets we need to resolve (from Phase 5 - max_check_compute)
VTABLE_OFFSETS_TO_RESOLVE = {
    0x138: "multiply_method (div 0x2e3b)",
    0xb0:  "result_store_method (div 0xbffa)",
    0x20:  "method_0x20 (div 0x2e4a)",
    0x28:  "release_retain (div 0x3a87)",
    0x40:  "method_0x40 (div 0x1bf6)",
    0x78:  "method_0x78 (div 0x34f6)",
    0x90:  "getValue (div 0x29e8)",
    0xa8:  "method_0xa8 (div 0xba55)",
    0xb8:  "method_0xb8 (div 0x43aa)",
    0xe8:  "setValue (div 0xc302)",
}

def va_to_file(va):
    return va - KC_BASE

def file_to_va(off):
    return KC_BASE + off

def decode_chained_ptr(raw_val):
    """Decode DYLD_CHAINED_PTR_64_KERNEL_CACHE pointer"""
    is_auth = (raw_val >> 63) & 1
    if is_auth:
        target_off = raw_val & 0x3FFFFFFF  # bits[29:0]
        diversity = (raw_val >> 32) & 0xFFFF
        addr_div = (raw_val >> 48) & 1
        key = (raw_val >> 49) & 3
        resolved_va = KC_BASE + target_off
        key_names = {0: "IA", 1: "IB", 2: "DA", 3: "DB"}
        return {
            'va': resolved_va,
            'is_auth': True,
            'diversity': diversity,
            'addr_div': addr_div,
            'key': key_names.get(key, f"?{key}"),
            'raw': raw_val,
            'target_off': target_off
        }
    else:
        target_off = raw_val & 0x3FFFFFFF
        resolved_va = KC_BASE + target_off
        return {
            'va': resolved_va,
            'is_auth': False,
            'raw': raw_val,
            'target_off': target_off
        }

def disasm_at(data, va, count=80):
    """Disassemble at VA, return list of (addr, mnemonic, op_str, bytes)"""
    md = Cs(CS_ARCH_ARM64, CS_MODE_ARM)
    md.detail = True
    foff = va_to_file(va)
    chunk = data[foff:foff+count*4]
    result = []
    for insn in md.disasm(chunk, va):
        result.append((insn.address, insn.mnemonic, insn.op_str, insn.bytes))
        if len(result) >= count:
            break
    return result

def find_function_end(insns):
    """Find likely function end (RET or next function's STP/SUB SP)"""
    for i, (addr, mn, ops, _) in enumerate(insns):
        if i > 2 and mn == 'ret':
            return i + 1
        # PACIBSP/STP pattern after first few insns might be next function
        if i > 5 and mn == 'pacibsp':
            return i
    return len(insns)

def main():
    print("=" * 70)
    print("CHAIN B PHASE 6: Virtual Dispatch Resolver + kernel_call Design")
    print("=" * 70)
    
    with open(KC_PATH, 'rb') as f:
        data = f.read()
    
    print(f"\n  Kernelcache: {len(data):,} bytes")
    print(f"  KC_BASE: {KC_BASE:#018x}")
    print(f"  IOSurface vtable: {IOSURFACE_VTABLE:#018x}")
    print(f"  Methods start: {VTABLE_METHODS:#018x}")
    
    # ═══════════════════════════════════════════════════════════════════════
    # SECTION 1: Resolve ALL vtable entries
    # ═══════════════════════════════════════════════════════════════════════
    print(f"\n{'='*70}")
    print("1. IOSurface VTABLE RESOLUTION")
    print(f"   Reading vtable entries and decoding chained fixup pointers")
    print(f"{'='*70}")
    
    resolved_vtable = {}
    # Read a generous range of vtable entries (0x00 to 0x200, every 8 bytes)
    for off in range(0, 0x200, 8):
        foff = va_to_file(VTABLE_METHODS + off)
        if foff + 8 > len(data):
            break
        raw = struct.unpack_from('<Q', data, foff)[0]
        if raw == 0:
            continue
        decoded = decode_chained_ptr(raw)
        resolved_vtable[off] = decoded
    
    print(f"\n  Resolved {len(resolved_vtable)} vtable entries:")
    print(f"  {'Offset':<10} {'VA':<20} {'Auth':<6} {'Key':<4} {'Div':<8} {'Label'}")
    print(f"  {'-'*10} {'-'*20} {'-'*6} {'-'*4} {'-'*8} {'-'*30}")
    
    for off in sorted(resolved_vtable.keys()):
        entry = resolved_vtable[off]
        label = VTABLE_OFFSETS_TO_RESOLVE.get(off, "")
        auth_str = "YES" if entry['is_auth'] else "no"
        key_str = entry.get('key', '-') if entry['is_auth'] else '-'
        div_str = f"0x{entry.get('diversity', 0):04x}" if entry['is_auth'] else '-'
        print(f"  +0x{off:03x}     {entry['va']:#018x} {auth_str:<6} {key_str:<4} {div_str:<8} {label}")
    
    # ═══════════════════════════════════════════════════════════════════════
    # SECTION 2: Disassemble CRITICAL vtable methods
    # ═══════════════════════════════════════════════════════════════════════
    print(f"\n{'='*70}")
    print("2. CRITICAL VTABLE METHOD DISASSEMBLY")
    print(f"   Focus: vtable+0x138 (multiply) and vtable+0xb0 (result store)")
    print(f"{'='*70}")
    
    critical_offsets = [0x138, 0xb0]
    mul_function_va = None
    
    for vtoff in critical_offsets:
        if vtoff not in resolved_vtable:
            print(f"\n  [!] vtable+0x{vtoff:03x}: NOT FOUND in vtable!")
            continue
        
        entry = resolved_vtable[vtoff]
        target_va = entry['va']
        label = VTABLE_OFFSETS_TO_RESOLVE.get(vtoff, f"method_0x{vtoff:x}")
        
        print(f"\n  --- vtable+0x{vtoff:03x} -> {target_va:#018x} ({label}) ---")
        if entry['is_auth']:
            print(f"      Auth: key={entry['key']}, diversity=0x{entry['diversity']:04x}, addrDiv={entry['addr_div']}")
        
        # Check if target is a BTI+B trampoline
        foff = va_to_file(target_va)
        if foff + 8 <= len(data):
            first_insn = struct.unpack_from('<I', data, foff)[0]
            second_insn = struct.unpack_from('<I', data, foff+4)[0]
            
            # BTI c = 0xd503245f
            if first_insn == 0xd503245f:
                # Check if second is B (unconditional branch)
                if (second_insn >> 26) == 0x05:  # B instruction
                    imm26 = second_insn & 0x3FFFFFF
                    if imm26 & 0x2000000:
                        imm26 |= ~0x3FFFFFF
                    real_target = target_va + 4 + (imm26 * 4)
                    print(f"      [TRAMPOLINE] BTI C; B -> {real_target:#018x}")
                    target_va = real_target
        
        # Disassemble the actual function
        insns = disasm_at(data, target_va, 120)
        end = find_function_end(insns)
        insns = insns[:end]
        
        print(f"      Function at {target_va:#018x} ({len(insns)} instructions):")
        
        mul_ops = []
        bl_targets = []
        ldr_patterns = []
        str_patterns = []
        
        for addr, mn, ops, raw_bytes in insns:
            print(f"        {addr:#018x}: {mn:<12} {ops}")
            
            # Track MUL/MADD/UMULL/SMULL
            if mn in ('mul', 'madd', 'msub', 'umull', 'smull', 'umulh', 'smulh', 'umaddl', 'smaddl'):
                mul_ops.append((addr, mn, ops))
            
            # Track BL targets
            if mn == 'bl':
                try:
                    bl_target = int(ops.replace('#', ''), 16)
                    bl_targets.append((addr, bl_target))
                except:
                    pass
            
            # Track LDR from struct offsets
            if mn == 'ldr' and '+' in ops:
                ldr_patterns.append((addr, mn, ops))
            
            # Track STR to struct offsets  
            if mn == 'str' and '+' in ops:
                str_patterns.append((addr, mn, ops))
        
        if mul_ops:
            print(f"\n      *** MULTIPLY INSTRUCTIONS FOUND: {len(mul_ops)} ***")
            for addr, mn, ops in mul_ops:
                print(f"          {addr:#018x}: {mn} {ops}")
            if vtoff == 0x138:
                mul_function_va = target_va
        else:
            print(f"\n      No MUL instructions in this function")
        
        if bl_targets:
            print(f"\n      BL targets ({len(bl_targets)}):")
            for addr, tgt in bl_targets:
                in_kext = "IOSurface" if KEXT_TEXT_START <= tgt <= KEXT_TEXT_END else "external"
                print(f"          {addr:#018x}: BL {tgt:#018x} ({in_kext})")
        
        if ldr_patterns:
            print(f"\n      LDR from struct ({len(ldr_patterns)}):")
            for addr, mn, ops in ldr_patterns[:10]:
                print(f"          {addr:#018x}: {mn} {ops}")
    
    # ═══════════════════════════════════════════════════════════════════════
    # SECTION 3: Trace the kernel helper BL 0xfffffff00861ed6c
    # ═══════════════════════════════════════════════════════════════════════
    print(f"\n{'='*70}")
    print("3. KERNEL HELPER DISASSEMBLY")
    print(f"   Target: {KERNEL_HELPER:#018x} (called from max_check_compute)")
    print(f"{'='*70}")
    
    insns = disasm_at(data, KERNEL_HELPER, 60)
    end_idx = find_function_end(insns)
    insns = insns[:end_idx]
    
    print(f"\n  Function at {KERNEL_HELPER:#018x} ({len(insns)} instructions):")
    for addr, mn, ops, _ in insns:
        print(f"    {addr:#018x}: {mn:<12} {ops}")
    
    # ═══════════════════════════════════════════════════════════════════════
    # SECTION 4: Deep-trace sub-calls from vtable+0x138 method
    # If no MUL found directly, trace its BL targets
    # ═══════════════════════════════════════════════════════════════════════
    print(f"\n{'='*70}")
    print("4. DEEP TRACE: Sub-calls from vtable+0x138 method")
    print(f"   Looking for MUL/UMULL in callees")
    print(f"{'='*70}")
    
    if 0x138 in resolved_vtable:
        entry = resolved_vtable[0x138]
        target_va = entry['va']
        
        # Resolve trampoline
        foff = va_to_file(target_va)
        first = struct.unpack_from('<I', data, foff)[0]
        second = struct.unpack_from('<I', data, foff+4)[0]
        if first == 0xd503245f and (second >> 26) == 0x05:
            imm26 = second & 0x3FFFFFF
            if imm26 & 0x2000000:
                imm26 |= ~0x3FFFFFF
            target_va = target_va + 4 + (imm26 * 4)
        
        # Get BL targets from this function
        insns = disasm_at(data, target_va, 120)
        end_idx = find_function_end(insns)
        
        bl_targets = []
        for addr, mn, ops, _ in insns[:end_idx]:
            if mn == 'bl':
                try:
                    bl_targets.append(int(ops.replace('#', ''), 16))
                except:
                    pass
        
        for i, bl_tgt in enumerate(bl_targets):
            print(f"\n  --- Sub-call {i}: {bl_tgt:#018x} ---")
            sub_insns = disasm_at(data, bl_tgt, 80)
            sub_end = find_function_end(sub_insns)
            sub_insns = sub_insns[:sub_end]
            
            mul_found = []
            for addr, mn, ops, _ in sub_insns:
                print(f"    {addr:#018x}: {mn:<12} {ops}")
                if mn in ('mul', 'madd', 'msub', 'umull', 'smull', 'umulh', 'smulh', 'umaddl', 'smaddl'):
                    mul_found.append((addr, mn, ops))
            
            if mul_found:
                print(f"\n    *** MUL FOUND IN SUB-CALL {i}! ***")
                for addr, mn, ops in mul_found:
                    print(f"        {addr:#018x}: {mn} {ops}")
    
    # ═══════════════════════════════════════════════════════════════════════
    # SECTION 5: IOSurface_max_check full flow with resolved vtable
    # ═══════════════════════════════════════════════════════════════════════
    print(f"\n{'='*70}")
    print("5. IOSurface_max_check FULL FLOW ANALYSIS")
    print(f"   Tracing: check_call -> compute (vtable dispatch) -> overflow test")
    print(f"{'='*70}")
    
    # Re-disassemble max_check with annotations
    insns = disasm_at(data, IOSURFACE_MAX_CHECK, 80)
    end_idx = find_function_end(insns)
    insns = insns[:end_idx]
    
    print(f"\n  IOSurface_max_check @ {IOSURFACE_MAX_CHECK:#018x} ({len(insns)} insns):")
    for addr, mn, ops, _ in insns:
        annotation = ""
        if mn == 'bl' and str(hex(MAX_CHECK_COMPUTE)).replace('0x', '') in ops.lower():
            annotation = " <== CALLS max_check_compute (virtual dispatch)"
        elif mn == 'lsr' and '#32' in ops:
            annotation = " <== OVERFLOW CHECK (truncation to 32-bit)"
        elif mn == 'cbnz':
            annotation = " <== BRANCH if overflow detected"
        elif mn in ('str', 'stur') and 'w' in ops.split(',')[0]:
            annotation = " <== STORE truncated 32-bit result"
        elif 'adrp' in mn:
            annotation = " (string pointer setup)"
        print(f"    {addr:#018x}: {mn:<12} {ops}{annotation}")
    
    # ═══════════════════════════════════════════════════════════════════════
    # SECTION 6: kernel_call PRIMITIVE DESIGN
    # ═══════════════════════════════════════════════════════════════════════
    print(f"\n{'='*70}")
    print("6. kernel_call PRIMITIVE DESIGN")
    print(f"   Designing the R/W primitive from IOSurface overflow")
    print(f"{'='*70}")
    
    print("""
  === EXPLOITATION STRATEGY ===
  
  STEP 1: IOSurface Integer Overflow
  -----------------------------------
  Create IOSurface with crafted dimensions that overflow in max_check:
    width * height * bytes_per_element * element_width
  The multiplication happens via virtual dispatch through vtable+0x138.
  If result > 32 bits, LSR #32 should catch it, BUT:
    - The check is: if (result >> 32) != 0 → fail
    - We need the product to wrap to a SMALL value in low 32 bits
    - While the actual allocation uses the small (truncated) value
    - But the surface metadata stores the ORIGINAL large dimensions
  
  STEP 2: Heap Overflow via Size Mismatch
  -----------------------------------------
  Surface is allocated with truncated size (small)
  But operations (read/write) use original dimensions (large)
  This gives us an out-of-bounds read/write relative to the surface buffer
  
  STEP 3: Corrupt Adjacent Object
  ---------------------------------
  Spray IOSurface objects to get predictable layout
  Overflow from one surface into adjacent surface's metadata
  Target: Overwrite the adjacent surface's buffer pointer
  
  STEP 4: Arbitrary Read/Write 
  ------------------------------
  Set corrupted surface's buffer pointer to target address
  Use s_get_value (selector 5) to READ from arbitrary address
  Use s_set_value (selector 4) to WRITE to arbitrary address
  
  STEP 5: Call ml_phys_read for BootROM
  --------------------------------------
  Option A: Overwrite function pointer in IOSurface vtable
    - Requires PAC bypass (forge pointer with correct diversity)
    - PAC-IA diversity 0x705d for dispatch table entries
    
  Option B: Corrupt task port for kernel task port
    - Get kernel task port via arbitrary r/w
    - Use mach_vm_read to read kernel memory
    - Then call ml_phys_read via crafted kernel memory layout
    
  Option C: Direct physical read via IOMMU bypass  
    - Map BootROM physical range (0x100000000) via IOSurface DMA
    - Requires IOKit entitlements
""")
    
    # ═══════════════════════════════════════════════════════════════════════
    # SECTION 7: Analyze IOSurface_allocate for size computation
    # ═══════════════════════════════════════════════════════════════════════
    print(f"{'='*70}")
    print("7. IOSurface_allocate SIZE COMPUTATION")
    print(f"   How does allocate compute final buffer size?")
    print(f"{'='*70}")
    
    insns = disasm_at(data, IOSURFACE_ALLOCATE, 150)
    end_idx = find_function_end(insns)
    insns = insns[:end_idx]
    
    # Find MUL and allocation-related instructions
    mul_in_alloc = []
    lsl_in_alloc = []
    bl_in_alloc = []
    
    print(f"\n  IOSurface_allocate @ {IOSURFACE_ALLOCATE:#018x} ({len(insns)} insns):")
    for addr, mn, ops, _ in insns:
        annotation = ""
        if mn in ('mul', 'madd', 'umull', 'smull', 'umulh'):
            mul_in_alloc.append((addr, mn, ops))
            annotation = " <== MULTIPLY"
        elif mn == 'lsl':
            lsl_in_alloc.append((addr, mn, ops))
            annotation = " <== SHIFT"
        elif mn == 'bl':
            bl_in_alloc.append((addr, ops))
            annotation = " <== CALL"
        elif 'ldr' in mn and '#0x58' in ops or '#0x60' in ops or '#0x80' in ops or '#0x90' in ops or '#0x78' in ops:
            annotation = " <== READS DIMENSION"
        print(f"    {addr:#018x}: {mn:<12} {ops}{annotation}")
    
    if mul_in_alloc:
        print(f"\n  MUL ops in allocate: {len(mul_in_alloc)}")
        for a, m, o in mul_in_alloc:
            print(f"    {a:#018x}: {m} {o}")
    
    # ═══════════════════════════════════════════════════════════════════════
    # SECTION 8: Overflow Parameter Calculator
    # ═══════════════════════════════════════════════════════════════════════
    print(f"\n{'='*70}")
    print("8. OVERFLOW PARAMETER CALCULATOR")
    print(f"   Computing dimensions that trigger integer overflow")
    print(f"{'='*70}")
    
    # We need: width * height * bpe * elem_width to overflow 32 bits
    # but have low 32 bits = small value (e.g., 0x1000 = 4096)
    # Example: 0x100000001 * 0x1000 would give low32 = 0x1000, but that's
    # not how the check works exactly. Let's compute some candidates.
    
    print("\n  Target: width * height that wraps to small value in 32-bit")
    print("  Assuming bytes_per_element=4, element_width=1:")
    print()
    
    candidates = []
    # Try: width=0x4001, height=0x4001 → 0x4001 * 0x4001 = 0x10008001
    # That's > 32 bit? No, 0x10008001 fits in 32 bits. Need bigger.
    # width=0x10001, height=0x10001 → 0x100020001 → low32 = 0x20001, high32 = 1 → CAUGHT!
    # 
    # The check is: (result >> 32) != 0 → reject
    # So we need the 64-bit result to fit in 32 bits BUT intermediate truncation
    # causes a mismatch between checked size and allocated size.
    #
    # The REAL vulnerability would be if:
    # - max_check computes: checked_val = width * height (this is checked)
    # - allocate computes:  alloc_size = width * height * bpe (this overflows differently)
    # OR if the check truncates to 32-bit BEFORE comparing
    
    # Let's look for the classic pattern:
    # uint32_t size = (uint32_t)(width * height);  // truncating cast
    # if (size > MAX) return error;
    # alloc(size);  // small allocation
    # memset(buf, 0, width * height);  // full-size operation → overflow
    
    # The STR w8 (32-bit store) we found is the truncation point!
    # max_check stores the TRUNCATED 32-bit result
    # But does allocate re-compute or use the stored value?
    
    test_cases = [
        (0x10000, 0x10001, 4, 1),  # 0x10000 * 0x10001 * 4 = 0x400040000 → low32 = 0x40000
        (0x8000, 0x8001, 4, 1),    # 0x8000 * 0x8001 * 4 = 0x100020000 → low32 = 0x20000
        (0x4000, 0x10001, 4, 1),   # 0x4000 * 0x10001 * 4 = 0x100010000 → low32 = 0x10000  
        (0x10000, 0x10000, 1, 1),  # 0x100000000 → low32 = 0 (PERFECT wraparound!)
        (0x10001, 0x10000, 1, 1),  # 0x100010000 → low32 = 0x10000
        (0x20000, 0x8000, 1, 1),   # 0x100000000 → low32 = 0
        (0x40000, 0x4000, 1, 1),   # 0x100000000 → low32 = 0
        (0x100, 0x100, 0x100, 0x100), # 0x100000000 → low32 = 0
    ]
    
    print(f"  {'Width':<12} {'Height':<12} {'BPE':<6} {'ElemW':<6} {'Product (64-bit)':<20} {'Low32':<12} {'Detected?'}")
    print(f"  {'-'*12} {'-'*12} {'-'*6} {'-'*6} {'-'*20} {'-'*12} {'-'*10}")
    
    for w, h, bpe, ew in test_cases:
        product = w * h * bpe * ew
        low32 = product & 0xFFFFFFFF
        high32 = product >> 32
        detected = "YES (blocked)" if high32 != 0 else "NO (passes)"
        # Wait - if the check checks (product >> 32) != 0, then any overflow IS detected
        # The vuln must be that the check only does SOME of the multiplications
        # while allocate does ALL of them
        candidates.append((w, h, bpe, ew, product, low32, high32 != 0))
        print(f"  {w:#010x} {h:#010x} {bpe:<6} {ew:<6} {product:#018x}   {low32:#010x}  {detected}")
    
    print("""
  ANALYSIS:
  The LSR #32 + CBNZ check in IOSurface_max_check catches ALL 64-bit overflows.
  The vulnerability window depends on WHAT is checked vs WHAT is allocated:
  
  If max_check checks: width * height (2 factors)
  But allocate uses: width * height * bytes_per_row (3 factors)
  Then: pass check with legal w*h, but w*h*bpr overflows
  
  Or: max_check checks each dimension independently against a max,
  but doesn't check the PRODUCT of all dimensions together.
  The 10 call sites each pass a different label string, suggesting
  each call checks ONE dimension parameter against a maximum.
  
  KEY INSIGHT: Each of the 10 max_check calls checks a SINGLE
  dimension (width, height, bpe, etc.) against a per-field maximum.
  The PRODUCT of multiple valid dimensions can still overflow!
""")
    
    # ═══════════════════════════════════════════════════════════════════════
    # SECTION 9: Analyze what strings are passed to each max_check call
    # ═══════════════════════════════════════════════════════════════════════
    print(f"{'='*70}")
    print("9. max_check CALL SITE LABELS")
    print(f"   What dimension does each call site check?")
    print(f"{'='*70}")
    
    # IOSurface cstring section
    CSTRING_VA = 0xfffffff007789298
    CSTRING_SIZE = 0x2e5a
    cstring_foff = va_to_file(CSTRING_VA)
    cstring_data = data[cstring_foff:cstring_foff+CSTRING_SIZE]
    
    # Known call sites from Phase 5
    call_sites = [
        0xfffffff00a1cb810,
        0xfffffff00a1cb9dc,
        0xfffffff00a1cb9f0,
        0xfffffff00a1cba04,
        0xfffffff00a1cba18,
        0xfffffff00a1cbc5c,
        0xfffffff00a1cbc74,
        0xfffffff00a1cbc90,
        0xfffffff00a1cbcac,
        0xfffffff00a1ccdb4,
    ]
    
    for site_va in call_sites:
        # Look backwards for ADRP+ADD that sets x1 (label string)
        start = site_va - 40  # look up to 10 insns before
        insns = disasm_at(data, start, 20)
        
        label_str = "?"
        adrp_page = None
        
        for addr, mn, ops, _ in insns:
            if addr > site_va:
                break
            if mn == 'adrp' and 'x1' in ops:
                try:
                    page_str = ops.split('#')[-1]
                    adrp_page = int(page_str, 16)
                except:
                    pass
            elif mn == 'add' and 'x1' in ops and adrp_page is not None:
                try:
                    offset_str = ops.split('#')[-1]
                    page_off = int(offset_str, 16)
                    str_va = adrp_page + page_off
                    str_foff = va_to_file(str_va)
                    # Read null-terminated string
                    end = data.index(b'\x00', str_foff)
                    label_str = data[str_foff:end].decode('ascii', errors='replace')
                    adrp_page = None
                except:
                    pass
        
        print(f"  {site_va:#018x}: BL max_check  label=\"{label_str}\"")
    
    # ═══════════════════════════════════════════════════════════════════════
    # SECTION 10: Final Intelligence Report
    # ═══════════════════════════════════════════════════════════════════════
    print(f"\n{'='*70}")
    print("10. FINAL INTELLIGENCE REPORT")
    print(f"{'='*70}")
    
    # Collect all resolved vtable data
    report = {
        'vtable_entries': {},
        'multiply_method': None,
        'result_store_method': None, 
        'kernel_helper': hex(KERNEL_HELPER),
        'max_check_labels': [],
        'kernel_call_gadgets': 393,
        'pac_indirect_calls': 1098,
        'exploitation_approach': 'dimension_product_overflow'
    }
    
    for off, entry in resolved_vtable.items():
        report['vtable_entries'][f"+0x{off:03x}"] = {
            'va': hex(entry['va']),
            'auth': entry['is_auth'],
            'key': entry.get('key', None),
            'diversity': hex(entry.get('diversity', 0)) if entry['is_auth'] else None,
        }
        if off == 0x138:
            report['multiply_method'] = hex(entry['va'])
        if off == 0xb0:
            report['result_store_method'] = hex(entry['va'])
    
    out_path = os.path.join(os.path.dirname(__file__), "extracted", "CHAIN_B_PHASE6_VTABLE.json")
    with open(out_path, 'w', encoding='utf-8') as f:
        json.dump(report, f, indent=2, ensure_ascii=False)
    print(f"\n  Report saved: {out_path}")
    
    print(f"""
  ╔══════════════════════════════════════════════════════════════════╗
  ║            CHAIN B PHASE 6 — COMPLETE                           ║
  ╠══════════════════════════════════════════════════════════════════╣
  ║ Vtable entries resolved: {len(resolved_vtable):<4}                                 ║
  ║ Multiply method (vtable+0x138): {report['multiply_method'] or 'UNRESOLVED':<22}  ║
  ║ Result store (vtable+0xb0):     {report['result_store_method'] or 'UNRESOLVED':<22}  ║
  ║ Kernel helper: {KERNEL_HELPER:#018x}                       ║
  ║                                                                  ║
  ║ NEXT: Phase 7 — Full PoC parameter computation                  ║
  ║   - Identify exact overflow dimensions                          ║
  ║   - Map heap layout for adjacent object corruption              ║
  ║   - Build PAC forgery strategy or alternative kernel_call       ║
  ╚══════════════════════════════════════════════════════════════════╝
""")

if __name__ == '__main__':
    main()
